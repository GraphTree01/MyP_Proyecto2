//! Metadatos generales asociados a archivos MP3.

use std::error::Error;
use std::path::{Path, PathBuf};

use id3::{Tag, TagLike};

/// Metadatos extraídos de la etiqueta ID3 de un archivo de audio.
///
/// Los campos que no estén presentes en la etiqueta se representan con
/// [`None`]. La ruta del archivo siempre se conserva para identificar el
/// origen de los datos.
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
    /// Lee la etiqueta ID3 del archivo indicado y crea sus metadatos.
    ///
    /// Devuelve un error si el archivo no puede abrirse o si su etiqueta no
    /// puede interpretarse.
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

    //Getters de la clase Metadatos

    /// Devuelve la ruta del archivo del que se obtuvieron los metadatos.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Devuelve el título de la canción, si está definido.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Devuelve el intérprete de la canción, si está definido.
    pub fn performer(&self) -> Option<&str> {
        self.performer.as_deref()
    }

    /// Devuelve el nombre del álbum, si está definido.
    pub fn album(&self) -> Option<&str> {
        self.album.as_deref()
    }

    /// Devuelve el género musical, si está definido.
    pub fn genre(&self) -> Option<&str> {
        self.genre.as_deref()
    }

    /// Devuelve el año de la canción, si está definido.
    pub fn year(&self) -> Option<i32> {
        self.year
    }

    /// Devuelve el número de pista, si está definido.
    pub fn track(&self) -> Option<u32> {
        self.track
    }
}
