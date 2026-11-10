//! Clase que sigue el patron de diseño DAO para abstraer y encapsular el
// acceso a la tabla albums del esquema de la base de datos.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::modelo::album::Album;
use crate::modelo::error::ErrorModelo;

/// Acceso a datos de la tabla `albums`.
pub struct AlbumDao;

impl AlbumDao {
    /// Crea un DAO de álbumes.
    pub fn nuevo() -> Self {
        Self
    }

    /// Busca el identificador de un álbum por su nombre.
    pub fn buscar_id_por_nombre(
        &self,
        conexion: &Connection,
        nombre: &str,
    ) -> Result<Option<i64>, ErrorModelo> {
        Ok(conexion
            .query_row(
                "SELECT id_album FROM albums WHERE name = ?1 LIMIT 1",
                params![nombre],
                |fila| fila.get(0),
            )
            .optional()?)
    }

    /// Inserta un álbum y devuelve el identificador generado.
    pub fn insertar(&self, conexion: &Connection, album: &Album) -> Result<i64, ErrorModelo> {
        let ruta = album.path.as_deref().and_then(Path::to_str);
        conexion.execute(
            "INSERT INTO albums (path, name, year) VALUES (?1, ?2, ?3)",
            params![ruta, album.name, album.year],
        )?;
        Ok(conexion.last_insert_rowid())
    }
}
