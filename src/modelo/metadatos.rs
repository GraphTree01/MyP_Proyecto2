//! Metadatos generales asociados a archivos MP3.

use std::path::{Path, PathBuf};

use id3::frame::PictureType;
use id3::{ErrorKind, Tag, TagLike};

use crate::modelo::error::ErrorModelo;

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
    caratula: Option<Vec<u8>>,
}

impl Metadatos {
    /// Lee la etiqueta ID3 del archivo indicado y crea sus metadatos.
    ///
    /// Devuelve un error si el archivo no puede abrirse o si su etiqueta no
    /// puede interpretarse.
    pub fn nuevo(path: &Path) -> Result<Self, ErrorModelo> {
        let etiqueta = Tag::read_from_path(path).map_err(|fuente| ErrorModelo::LecturaId3 {
            ruta: path.to_path_buf(),
            fuente,
        })?;
        let title = etiqueta.title().map(String::from);
        let performer = etiqueta.artist().map(String::from);
        let album = etiqueta.album().map(String::from);
        let genre = etiqueta.genre().map(String::from);
        let year = etiqueta.year();
        let track = etiqueta.track();
        let caratula = extraer_caratula(&etiqueta);

        Ok(Self {
            path: path.to_path_buf(),
            title,
            performer,
            album,
            genre,
            year,
            track,
            caratula,
        })
    }

    /// Lee la portada de un archivo para incluirla en los resultados de la vista.
    ///
    /// Si el archivo ya no existe o no tiene etiqueta ID3, no hay portada
    /// disponible.
    pub fn leer_caratula(path: &Path) -> Result<Option<Vec<u8>>, ErrorModelo> {
        let etiqueta = match Tag::read_from_path(path) {
            Ok(etiqueta) => etiqueta,
            Err(error) if matches!(&error.kind, ErrorKind::NoTag) => return Ok(None),
            Err(error)
                if matches!(
                    &error.kind,
                    ErrorKind::Io(fuente)
                        if fuente.kind() == std::io::ErrorKind::NotFound
                ) =>
            {
                return Ok(None);
            }
            Err(fuente) => {
                return Err(ErrorModelo::LecturaId3 {
                    ruta: path.to_path_buf(),
                    fuente,
                });
            }
        };
        Ok(extraer_caratula(&etiqueta))
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

    /// Devuelve los bytes de la portada ID3, si está disponible.
    pub fn caratula(&self) -> Option<&[u8]> {
        self.caratula.as_deref()
    }
}

fn extraer_caratula(etiqueta: &Tag) -> Option<Vec<u8>> {
    etiqueta
        .pictures()
        .filter(|imagen| !imagen.data.is_empty())
        .find(|imagen| imagen.picture_type == PictureType::CoverFront)
        .or_else(|| etiqueta.pictures().find(|imagen| !imagen.data.is_empty()))
        .map(|imagen| imagen.data.clone())
}
