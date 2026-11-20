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

    /// Busca el álbum asociado a un directorio exacto.
    pub fn buscar_id_por_directorio(
        &self,
        conexion: &Connection,
        directorio: &Path,
    ) -> Result<Option<i64>, ErrorModelo> {
        let directorio = directorio
            .to_str()
            .ok_or_else(|| ErrorModelo::RutaNoRepresentable(directorio.to_path_buf()))?;

        Ok(conexion
            .query_row(
                "SELECT id_album FROM albums WHERE path = ?1 LIMIT 1",
                [directorio],
                |fila| fila.get(0),
            )
            .optional()?)
    }

    /// Inserta un álbum y devuelve el identificador generado.
    pub fn insertar(&self, conexion: &Connection, album: &Album) -> Result<i64, ErrorModelo> {
        let ruta = album
            .path
            .as_deref()
            .map(|ruta| {
                ruta.to_str()
                    .ok_or_else(|| ErrorModelo::RutaNoRepresentable(ruta.to_path_buf()))
            })
            .transpose()?;
        conexion.execute(
            "INSERT INTO albums (path, name, year) VALUES (?1, ?2, ?3)",
            params![ruta, album.name, album.year],
        )?;
        Ok(conexion.last_insert_rowid())
    }
}
