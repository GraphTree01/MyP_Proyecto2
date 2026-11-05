use std::path::PathBuf;

/// Representa un registro de la tabla `rolas`, en la base de datos.
pub struct Rola {
    /// Identificador generado por SQLite.
    pub id_rola: Option<i64>,
    /// Identificador del intérprete relacionado.
    pub id_performer: i64,
    /// Identificador del álbum relacionado.
    pub id_album: i64,
    /// Ruta del archivo de audio.
    pub path: PathBuf,
    /// Título de la canción.
    pub title: String,
    /// Número de pista, si está disponible.
    pub track: Option<u32>,
    /// Año de la canción, si está disponible.
    pub year: Option<i32>,
    /// Género musical, si está disponible.
    pub genre: Option<String>,
}
