use std::path::PathBuf;

/// Representa un registro de la tabla `albums`.
pub struct Album {
    /// Identificador generado por SQLite.
    pub id_album: Option<i64>,
    /// Ruta asociada al álbum, si existe.
    pub path: Option<PathBuf>,
    /// Nombre del álbum.
    pub name: String,
    /// Año de publicación, si está disponible.
    pub year: Option<i32>,
}
