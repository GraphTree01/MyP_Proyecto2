//! Clase que sigue el patron de diseño DAO para abstraer y encapsular el
// acceso a la tabla rolas del esquema de la base de datos.

use std::path::Path;

use rusqlite::{Connection, params};

use crate::modelo::error::ErrorModelo;
use crate::modelo::rola::{Rola, Rolita};

/// Acceso a datos de la tabla `rolas`.
pub struct RolaDao;

impl RolaDao {
    /// Crea un DAO de canciones.
    pub fn nuevo() -> Self {
        Self
    }

    /// Comprueba si ya existe una canción con la ruta indicada.
    pub fn existe(&self, conexion: &Connection, ruta: &Path) -> Result<bool, ErrorModelo> {
        let ruta = ruta
            .to_str()
            .ok_or_else(|| ErrorModelo::RutaNoRepresentable(ruta.to_path_buf()))?;

        Ok(conexion.query_row(
            "SELECT EXISTS(SELECT 1 FROM rolas WHERE path = ?1)",
            params![ruta],
            |fila| fila.get(0),
        )?)
    }

    /// Inserta una canción y devuelve el identificador generado.
    pub fn insertar(&self, conexion: &Connection, rola: &Rola) -> Result<i64, ErrorModelo> {
        let ruta = rola
            .path
            .to_str()
            .ok_or_else(|| ErrorModelo::RutaNoRepresentable(rola.path.clone()))?;
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

    /// Devuelve la cantidad de canciones almacenadas.
    pub fn contar(&self, conexion: &Connection) -> Result<i64, ErrorModelo> {
        Ok(conexion.query_row("SELECT COUNT(*) FROM rolas", [], |fila| fila.get(0))?)
    }

    /// Devuelve las canciones e identificadores de la base de datos.
    pub fn listar(&self, conexion: &Connection) -> Result<Vec<Rolita>, ErrorModelo> {
        let mut consulta = conexion.prepare(
            "SELECT r.title, p.name, a.name, r.genre
             FROM rolas r
             JOIN performers p ON p.id_performer = r.id_performer
             JOIN albums a ON a.id_album = r.id_album
             ORDER BY r.title, p.name, a.name",
        )?;

        let filas = consulta.query_map([], |fila| {
            Ok(Rolita {
                titulo: fila.get(0)?,
                performer: fila.get(1)?,
                album: fila.get(2)?,
                genero: fila.get(3)?,
            })
        })?;

        Ok(filas.collect::<Result<Vec<_>, _>>()?)
    }
}
