#[macro_use]
extern crate rocket;

use rocket_session_store::{memory::MemoryStore, SessionStore};

use rocket::tokio::sync::broadcast::channel;
use rocket::{fs::FileServer, futures::executor::block_on, http::Method};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use types::dto::user::AuthenticatedUser;

use std::path::PathBuf;
use std::time::Duration;

#[cfg(debug_assertions)]
use dotenv::dotenv;

mod database;
mod fs;
mod guards;
mod logging;
mod opds;
mod routing;
mod state;
mod types;
mod utils;

use crate::{database::Database, logging::Log, types::rocket::UnauthorizedResponse};

pub type State = state::State;

#[catch(401)]
fn opds_unauthorized(_req: &rocket::Request) -> UnauthorizedResponse {
    UnauthorizedResponse {}
}

fn home_dir() -> PathBuf {
    dirs::home_dir().expect("Could not find home directory")
}

fn init_env() {
    let config_dir = match std::env::var("STUMP_CONFIG_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => home_dir().join(".stump"),
    };

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| panic!("Could not create config directory: {}", e));

    let database_url = format!(
        "sqlite:{}?mode=rwc",
        config_dir.join("stump.db").to_str().unwrap()
    );

    std::env::set_var("DATABASE_URL", database_url);
}

#[launch]
fn rocket() -> _ {
    #[cfg(debug_assertions)]
    dotenv().ok();

    init_env();

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .init();

    // FIXME: restrict this
    let allowed_origins = AllowedOrigins::all();

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Put, Method::Post, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Could not instantiate CORS configuration.");

    let db = block_on(Database::new());
    let state = state::AppState::new(db, channel::<Log>(1024).0);

    let session_name = std::env::var("SESSION_NAME").unwrap_or_else(|_| "stump-session".into());

    // TODO: I am not sure if I like this implementation yet. I might want to use my sqlite connection
    // to manage sessions. I did not want to deep dive into creating my own fairing for this *yet*,
    // but will likely do so in the future.
    let memory_store: MemoryStore<AuthenticatedUser> = MemoryStore::default();
    let store: SessionStore<AuthenticatedUser> = SessionStore {
        store: Box::new(memory_store),
        name: session_name,
        duration: Duration::from_secs(3600 * 24 * 3),
    };

    rocket::build()
        .manage(state)
        .attach(store.fairing())
        .attach(cors)
        .mount("/", FileServer::from("static/"))
        .mount(
            "/api",
            routes![
                // top level
                routing::api::scan,
                routing::api::log_listener,
                // auth
                routing::api::auth::login,
                routing::api::auth::register,
                // logs api
                routing::api::log::get_logs,
                // library api
                routing::api::library::get_libraries,
                routing::api::library::get_library,
                routing::api::library::insert_library,
                routing::api::library::update_library,
                routing::api::library::delete_library,
                routing::api::library::scan_library,
                // series api
                routing::api::series::get_series,
                routing::api::series::get_series_by_id,
                routing::api::series::get_series_thumbnail,
                // media api
                routing::api::media::get_media,
                routing::api::media::get_media_page,
                routing::api::media::get_media_thumbnail,
                routing::api::media::update_media_progress,
            ],
        )
        .mount(
            "/opds/v1.2",
            routes![
                routing::opds::catalog,
                routing::opds::keep_reading,
                // libraries
                routing::opds::libraries,
                routing::opds::library_by_id,
                // series
                routing::opds::series,
                routing::opds::series_latest,
                routing::opds::series_by_id,
                // books
                routing::opds::book_thumbnail,
                routing::opds::book_page
            ],
        )
        .register("/opds/v1.2", catchers![opds_unauthorized])
}
