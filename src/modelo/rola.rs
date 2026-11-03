use std::path::PathBuf;

pub struct Rola {
    pub id_rola: Option<i64>,
    pub id_performer: i64,
    pub id_album: i64,
    pub path: PathBuf,
    pub title: String,
    pub track: Option<u32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
}
