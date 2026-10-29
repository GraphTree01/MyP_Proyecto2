use std::error::Error;
use std::path::{PathBuf, Path};

use id3::{Tag, TagLike};

pub struct Metadatos {

    path: PathBuf,
    title: Option<String>,
    performer: Option<String>,
    album: Option<String>,
    genre: Option<String>,
    year: Option<i32>,
    track: Option<u32>,
}

impl Metadatos {

    pub fn nuevo(path: &Path) -> Result<Self, Box<dyn Error>> {

        let etiqueta = Tag::read_from_path(path)?;
        let title = etiqueta.title().map(String::from);
        let performer = etiqueta.artist().map(String::from);
        let album = etiqueta.album().map(String::from);
        let genre = etiqueta.genre().map(String::from);
        let year = etiqueta.year();
        let track = etiqueta.track();

        Ok(Self {
            path: path.to_path_buf(),
            title,
            performer,
            album,
            genre,
            year,
            track,
        })

    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn performer(&self) -> Option<&str> {
        self.performer.as_deref()
    }

    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    pub fn genre(&self) -> Option<&str> {
        self.genre.as_deref()
    }

    pub fn year(&self) -> Option<i32> {
        self.year
    }

    pub fn track(&self) -> Option<u32> {
        self.track
    }
}
