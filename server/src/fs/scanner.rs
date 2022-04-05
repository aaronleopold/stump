use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use entity::{
    library, media,
    sea_orm::{self, ActiveModelTrait},
    series,
    util::FileStatus,
};

use sea_orm::{DatabaseConnection, Set};

use walkdir::{DirEntry, WalkDir};

use crate::{
    database::queries,
    event::{event::Event, handler::EventHandler},
    fs::{
        epub::process_epub, error::ProcessFileError, media_file::ProcessResult, rar::process_rar,
        zip::process_zip,
    },
    types::{
        comic::ComicInfo,
        dto::{GetMediaQuery, GetMediaQueryResult},
    },
    State,
};

// TODO: use ApiErrors here!

pub trait IgnoredFile {
    fn should_ignore(&self) -> bool;
}

impl IgnoredFile for Path {
    fn should_ignore(&self) -> bool {
        let filename = self
            .file_name()
            .unwrap_or_default()
            .to_str()
            .expect(format!("Malformed filename: {:?}", self.as_os_str()).as_str());

        if self.is_dir() {
            return true;
        } else if filename.starts_with(".") {
            return true;
        }

        false
    }
}

// TODO: error handling / return result
fn generate_series_model(path: &Path, library_id: i32) -> series::ActiveModel {
    let metadata = match path.metadata() {
        Ok(metadata) => Some(metadata),
        _ => None,
    };

    // TODO: remove the unsafe unwraps throughout this file
    let name = path.file_name().unwrap().to_str().unwrap().to_string();

    let mut updated_at: Option<NaiveDateTime> = None;

    if let Some(metadata) = metadata {
        // TODO: extract to fn somewhere
        updated_at = match metadata.modified() {
            Ok(st) => {
                let dt: DateTime<Utc> = st.clone().into();
                Some(dt.naive_utc())
            }
            Err(_) => Some(Utc::now().naive_utc()),
        };
    }

    series::ActiveModel {
        library_id: Set(library_id),
        title: Set(name),
        updated_at: Set(updated_at),
        // TODO: do I want this to throw an error?
        path: Set(path.to_str().unwrap_or("").to_string()),
        // FIXME: this should be handled by default but isn't, see https://github.com/SeaQL/sea-orm/issues/420 ?
        status: Set(FileStatus::Ready),
        ..Default::default()
    }
}

fn process_entry(entry: &DirEntry) -> ProcessResult {
    match entry.file_name().to_str() {
        Some(name) if name.ends_with("cbr") => process_rar(entry),
        Some(name) if name.ends_with("cbz") => process_zip(entry),
        // Some(name) if name.ends_with("epub") => process_epub(entry),
        _ => Err(ProcessFileError::UnsupportedFileType),
    }
}

// TODO: result return to handle error downstream
fn generate_media_model(entry: &DirEntry, series_id: i32) -> Option<media::ActiveModel> {
    let processed_info = process_entry(entry);

    if let Err(e) = processed_info {
        // log::info!("{:?}", e);
        return None;
    }

    let (info, pages) = processed_info.unwrap();

    let path = entry.path();

    let metadata = match entry.metadata() {
        Ok(metadata) => Some(metadata),
        _ => None,
    };

    let path_str = path.to_str().unwrap().to_string();
    let name = entry.file_name().to_str().unwrap().to_string();
    let ext = path.extension().unwrap().to_str().unwrap().to_string();

    let comic_info = match info {
        Some(info) => info,
        None => ComicInfo::default(),
    };

    let mut size: u64 = 0;
    let mut modified: Option<NaiveDateTime> = None;

    if let Some(metadata) = metadata {
        size = metadata.len();

        modified = match metadata.modified() {
            Ok(st) => {
                let dt: DateTime<Utc> = st.clone().into();
                Some(dt.naive_utc())
            }
            Err(_) => Some(Utc::now().naive_utc()),
        };
    }

    Some(media::ActiveModel {
        series_id: Set(series_id),
        name: Set(name),
        description: Set(comic_info.summary),
        size: Set(size as i64),
        extension: Set(ext),
        pages: Set(match comic_info.page_count {
            Some(count) => count as i32,
            None => pages.len() as i32,
        }),
        updated_at: Set(modified),
        path: Set(path_str),
        status: Set(FileStatus::Ready),
        ..Default::default()
    })
}

fn dir_has_files(path: &Path) -> bool {
    let items = std::fs::read_dir(path);

    if items.is_err() {
        return false;
    }

    let items = items.unwrap();

    items
        .filter_map(|item| item.ok())
        .any(|f| !f.path().should_ignore())
}

struct Scanner<'a> {
    pub db: &'a DatabaseConnection,
    pub event_handler: &'a EventHandler,
    pub series: HashMap<String, series::Model>,
    pub media: HashMap<String, GetMediaQuery>,
}

impl<'a> Scanner<'a> {
    pub fn new(
        db: &'a DatabaseConnection,
        event_handler: &'a EventHandler,
        series: Vec<series::Model>,
        media: GetMediaQueryResult,
    ) -> Self {
        let mut media_map = HashMap::new();
        let mut series_map = HashMap::new();

        for m in media {
            // media_map.insert(media.checksum.clone(), media);
            media_map.insert(m.path.clone(), m);
        }

        for s in series {
            series_map.insert(s.path.clone(), s);
        }

        Self {
            db,
            event_handler,
            series: series_map,
            media: media_map,
        }
    }

    // FIXME: pass in &Model??
    // TODO: make me
    async fn analyze_media(&self, key: String) {
        let media = self.media.get(&key).unwrap();

        let id = media.id;

        println!("analyzing media: {:?}", media);

        if media.status == FileStatus::Missing {
            log::info!("Media found");
            self.set_media_status(id, FileStatus::Ready, media.path.clone())
                .await;
        }

        // TODO: more checks??
    }

    fn get_series(&self, path: &Path) -> Option<&series::Model> {
        self.series.get(path.to_str().expect("Invalid key"))
    }

    fn get_media(&self, key: &Path) -> Option<&GetMediaQuery> {
        self.media.get(key.to_str().expect("Invalid key"))
    }

    // TODO: I'm not sure I love this solution. I could check if the Path.parent is the series path, but
    // not sure about that either
    /// Get the series id for the given path. Used to determine the series of a media
    /// file at a given path.
    fn get_series_id(&self, path: &Path) -> Option<i32> {
        self.series
            .iter()
            .find(|(_, s)| path.to_str().unwrap_or("").to_string().contains(&s.path))
            .map(|(_, s)| s.id)
    }

    fn series_exists(&self, path: &Path) -> bool {
        self.get_series(path).is_some()
    }

    async fn set_media_status(&self, id: i32, status: FileStatus, path: String) {
        match queries::media::set_status(self.db, id, status).await {
            Ok(_) => {
                log::info!("set media status: {:?} -> {:?}", path, status);
                if status == FileStatus::Missing {
                    self.event_handler
                        .log_error(format!("Missing file: {}", path));
                }
            }
            Err(err) => {
                self.event_handler.log_error(err);
            }
        }
    }

    async fn set_series_status(&self, id: i32, status: FileStatus, path: String) {
        match queries::series::set_status(self.db, id, status).await {
            Ok(_) => {
                log::info!("set series status: {:?} -> {:?}", path, status);
                if status == FileStatus::Missing {
                    self.event_handler
                        .log_error(format!("Missing file: {}", path));
                }
            }
            Err(err) => {
                self.event_handler.log_error(err);
            }
        }
    }

    async fn create_series(&self, path: &Path, library_id: i32) -> Option<series::Model> {
        let series = generate_series_model(path, library_id);

        match series.insert(self.db).await {
            Ok(m) => {
                log::info!("Created new series: {:?}", m);
                self.event_handler
                    .emit_event(Event::series_created(m.clone()));
                Some(m)
            }
            Err(err) => {
                log::error!("Failed to create series: {:?}", err);
                self.event_handler.log_error(err.to_string());
                None
            }
        }
    }

    async fn create_media(&self, entry: &DirEntry, series_id: i32) -> Option<media::Model> {
        let media = generate_media_model(entry, series_id);

        if media.is_none() {
            return None;
        }

        let media = media.unwrap();

        log::info!("Creating media: {:?}", media);

        match media.insert(self.db).await {
            Ok(m) => {
                log::info!("Created new media: {:?}", m);
                self.event_handler
                    .emit_event(Event::media_created(m.clone()));
                Some(m)
            }
            Err(err) => {
                log::warn!("Failed to create media: {:?}", err);
                self.event_handler.log_error(err.to_string());
                None
            }
        }
    }

    pub async fn scan_library(&mut self, library: &library::Model) {
        let library_path = PathBuf::from(&library.path);

        let mut visited_series = HashMap::<i32, bool>::new();
        let mut visited_media = HashMap::<i32, bool>::new();

        for entry in WalkDir::new(&library.path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            log::info!("Current: {:?}", path);

            let series = self.get_series(&path);
            let series_exists = series.is_some();

            if path.is_dir() && !series_exists {
                if path.to_path_buf().eq(&library_path) && !dir_has_files(path) {
                    log::info!("Skipping library directory - contains no files.");
                    continue;
                }

                log::info!("Creating new series: {:?}", path);

                match self.create_series(path, library.id).await {
                    Some(s) => {
                        visited_series.insert(s.id, true);
                        self.series.insert(s.path.clone(), s);
                    }
                    // Error handled in the function call
                    None => {}
                }

                continue;
            }

            if series_exists {
                let series = series.unwrap();
                log::info!("Existing series: {:?}", series);
                visited_series.insert(series.id, true);
                continue;
            } else if path.should_ignore() {
                // log::info!("Ignoring: {:?}", path);
                continue;
            }

            if let Some(media) = self.get_media(&path) {
                // log::info!("Existing media: {:?}", media);
                visited_media.insert(media.id, true);
                // self.analyze_media(media).await;
                continue;
            }

            // TODO: don't do this :)
            let series_id = self.get_series_id(&path).expect(&format!(
                "Could not determine series for new media: {:?}",
                path
            ));

            log::info!("New media at {:?} in series {:?}", &path, series_id);

            match self.create_media(&entry, series_id).await {
                Some(m) => {
                    visited_media.insert(m.id, true);
                    // FIXME: ruh roh, this won't work but *do I need it to??*
                    // self.media.insert(m.path.clone(), m);
                }
                // Error handled in the function call
                None => {}
            }
        }

        for (_, s) in self.series.iter() {
            match visited_series.get(&s.id) {
                Some(true) => {
                    if s.status == FileStatus::Missing {
                        self.set_series_status(s.id, FileStatus::Ready, s.path.clone())
                            .await;
                    }
                }
                _ => {
                    if s.library_id == library.id {
                        log::info!("MOVED/MISSING SERIES: {}", s.path);
                        self.set_series_status(s.id, FileStatus::Missing, s.path.clone())
                            .await;
                    }
                }
            }
        }

        for media in self.media.values() {
            match visited_media.get(&media.id) {
                Some(true) => {
                    if media.status == FileStatus::Missing {
                        self.set_media_status(media.id, FileStatus::Ready, media.path.clone())
                            .await;
                    }
                }
                _ => {
                    if media.library_id == library.id {
                        log::info!("MOVED/MISSING MEDIA: {}", media.path);
                        self.set_media_status(media.id, FileStatus::Missing, media.path.clone())
                            .await;
                    }
                }
            }
        }
    }
}

pub async fn scan(state: &State, library_id: Option<i32>) -> Result<(), String> {
    let conn = state.get_connection();
    let event_handler = state.get_event_handler();

    let libraries: Vec<library::Model> = match library_id {
        Some(id) => queries::library::get_library_by_id(conn, id)
            .await?
            .map(|l| l.into())
            .into_iter()
            .collect(),
        None => queries::library::get_libraries(conn).await?,
    };

    if libraries.is_empty() {
        let mut message = "No libraries configured.".to_string();

        if library_id.is_some() {
            message = format!("No library with id: {}", library_id.unwrap());
        }

        event_handler.log_error(message.clone());

        return Err(message);
    }

    let series = queries::series::get_series_in_library(conn, library_id).await?;

    // println!("{:?}", series);

    let media = queries::media::get_media_with_library_and_series(conn, library_id).await?;

    // println!("{:?}", media);

    let mut scanner = Scanner::new(conn, event_handler, series, media);

    for library in libraries {
        scanner.scan_library(&library).await;
    }

    Ok(())
}
