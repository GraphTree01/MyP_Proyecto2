//! Clase que sigue el patron de diseño DAO para abstraer y encapsular el 
// acceso a la tabla rolas del esquema de la base de datos.

use std::error::Error;
use std::path::Path;

use rusqlite::{Connection, params};

use crate::modelo::rola::Rola;

/// Acceso a datos de la tabla `rolas`.
pub struct RolaDao;

impl RolaDao {
    /// Crea un DAO de canciones.
    pub fn nuevo() -> Self {
        Self
    }

    /// Comprueba si ya existe una canción con la ruta indicada.
    pub fn existe(&self, conexion: &Connection, ruta: &Path) -> Result<bool, Box<dyn Error>> {
        let ruta = ruta
            .to_str()
            .ok_or("La ruta no puede representarse como texto UTF-8")?;

        Ok(conexion.query_row(
            "SELECT EXISTS(SELECT 1 FROM rolas WHERE path = ?1)",
            params![ruta],
            |fila| fila.get(0),
        )?)
    }

    /// Inserta una canción y devuelve el identificador generado.
    pub fn insertar(&self, conexion: &Connection, rola: &Rola) -> Result<i64, Box<dyn Error>> {
        let ruta = rola
            .path
            .to_str()
            .ok_or("La ruta no puede representarse como texto UTF-8")?;
        conexion.execute(
            "INSERT INTO rolas
             (id_performer, id_album, path, title, track, year, genre)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                rola.id_performer,
                rola.id_album,
                ruta,
                rola.title,
                rola.track,
                rola.year,
                rola.genre
            ],
        )?;
        Ok(conexion.last_insert_rowid())
    }
}
