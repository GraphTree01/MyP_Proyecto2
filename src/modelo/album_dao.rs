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

    /// Busca los datos editables de un álbum.
    pub fn buscar(
        &self,
        conexion: &Connection,
        id_album: i64,
    ) -> Result<Option<Album>, ErrorModelo> {
        Ok(conexion
            .query_row(
                "SELECT id_album, path, name, year FROM albums WHERE id_album = ?1",
                [id_album],
                |fila| {
                    let ruta: Option<String> = fila.get(1)?;
                    Ok(Album {
                        id_album: Some(fila.get(0)?),
                        path: ruta.map(Into::into),
                        name: fila.get(2)?,
                        year: fila.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    /// Actualiza el nombre y año del álbum sin modificar su ruta.
    pub fn actualizar(
        &self,
        conexion: &Connection,
        id_album: i64,
        nombre: &str,
        anio: Option<i32>,
    ) -> Result<bool, ErrorModelo> {
        if id_album == 0 {
            return Err(ErrorModelo::Validacion(
                "El álbum desconocido no se puede modificar.".to_owned(),
            ));
        }
        Ok(conexion.execute(
            "UPDATE albums SET name = ?1, year = ?2 WHERE id_album = ?3",
            params![nombre, anio, id_album],
        )? > 0)
    }

    /// Elimina un álbum si ninguna pista lo utiliza.
    pub fn eliminar(&self, conexion: &Connection, id_album: i64) -> Result<bool, ErrorModelo> {
        if id_album == 0 {
            return Err(ErrorModelo::Validacion(
                "El álbum desconocido no se puede eliminar.".to_owned(),
            ));
        }
        let usado: bool = conexion.query_row(
            "SELECT EXISTS(SELECT 1 FROM rolas WHERE id_album = ?1)",
            [id_album],
            |fila| fila.get(0),
        )?;
        if usado {
            return Err(ErrorModelo::Validacion(
                "No se puede eliminar el álbum mientras tenga canciones.".to_owned(),
            ));
        }
        Ok(conexion.execute("DELETE FROM albums WHERE id_album = ?1", [id_album])? > 0)
    }
}
