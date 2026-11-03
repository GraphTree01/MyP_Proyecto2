use std::path::PathBuf;

pub struct Album {
    pub id_album: Option<i64>,
    pub path: Option<PathBuf>,
    pub name: String,
    pub year: Option<i32>,
}
