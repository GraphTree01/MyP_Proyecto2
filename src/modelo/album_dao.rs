use std::error::Error;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::modelo::album::Album;

pub struct AlbumDao;

impl AlbumDao {
    pub fn nuevo() -> Self {
        Self
    }

    pub fn buscar_id_por_nombre(
        &self,
        conexion: &Connection,
        nombre: &str,
    ) -> Result<Option<i64>, Box<dyn Error>> {
        Ok(conexion
            .query_row(
                "SELECT id_album FROM albums WHERE name = ?1 LIMIT 1",
                params![nombre],
                |fila| fila.get(0),
            )
            .optional()?)
    }

    pub fn insertar(&self, conexion: &Connection, album: &Album) -> Result<i64, Box<dyn Error>> {
        let ruta = album.path.as_deref().and_then(Path::to_str);
        conexion.execute(
            "INSERT INTO albums (path, name, year) VALUES (?1, ?2, ?3)",
            params![ruta, album.name, album.year],
        )?;
        Ok(conexion.last_insert_rowid())
    }
}
