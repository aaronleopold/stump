use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[sea_orm(table_name = "series")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// The id of the library this series belongs to.
    pub library_id: i32,
    /// The title of the series. This is generated from a fs scan, and will be the directory name.
    pub title: String,
    /// The number of media files in the series.
    pub book_count: i32,
    /// The date in which the series was last updated in the FS. ex: "2020-01-01"
    pub updated_at: String,
    /// The url of the series. ex: "/home/user/media/comics/The Amazing Spider-Man"
    pub path: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::media::Entity")]
    Media,

    #[sea_orm(
        belongs_to = "super::library::Entity",
        from = "Column::LibraryId"
        to="super::library::Column::Id"
    )]
    Library,
}

impl Related<super::media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

impl Related<super::library::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Library.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
