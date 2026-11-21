//! Clase que sigue el patron de diseño DAO para abstraer y encapsular el
// acceso a la tabla rolas del esquema de la base de datos.

use std::path::Path;

use rusqlite::{Connection, params};

use crate::modelo::compilador::ConsultaSql;
use crate::modelo::error::ErrorModelo;
use crate::modelo::metadatos::Metadatos;
use crate::modelo::rola::{Rola, Rolita};

type DatosFila = (String, String, String, Option<String>, String);

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

    /// Actualiza el álbum asociado a una pista ya almacenada.
    pub fn asignar_album_por_ruta(
        &self,
        conexion: &Connection,
        ruta: &Path,
        id_album: i64,
    ) -> Result<(), ErrorModelo> {
        let ruta = ruta
            .to_str()
            .ok_or_else(|| ErrorModelo::RutaNoRepresentable(ruta.to_path_buf()))?;

        conexion.execute(
            "UPDATE rolas SET id_album = ?1 WHERE path = ?2",
            params![id_album, ruta],
        )?;
        Ok(())
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

    /// Devuelve los datos de las canciones sin leer carátulas.
    pub fn listar(&self, conexion: &Connection) -> Result<Vec<Rolita>, ErrorModelo> {
        self.listar_con_caratulas(conexion, false)
    }

    /// Devuelve los datos de las canciones y carga las carátulas si se solicitan.
    pub fn listar_con_caratulas(
        &self,
        conexion: &Connection,
        incluir: bool,
    ) -> Result<Vec<Rolita>, ErrorModelo> {
        let columnas = if incluir {
            "NULL, r.path"
        } else {
            "r.genre, r.path"
        };
        let sql = format!(
            "SELECT r.title, p.name, a.name, {columnas}
             FROM rolas r
             JOIN performers p ON p.id_performer = r.id_performer
             JOIN albums a ON a.id_album = r.id_album
             ORDER BY r.title, p.name, a.name"
        );
        let mut consulta = conexion.prepare(&sql)?;
        let filas = consulta.query_map([], fila_a_datos)?;
        self.construir_rolitas(filas, incluir)
    }

    /// Ejecuta una consulta y devuelve las canciones que coiciden.
    pub fn buscar(
        &self,
        conexion: &Connection,
        consulta: &ConsultaSql,
    ) -> Result<Vec<Rolita>, ErrorModelo> {
        self.buscar_con_caratulas(conexion, consulta, false)
    }

    /// Ejecuta una consulta y carga carátulas solo si se solicitan.
    pub fn buscar_con_caratulas(
        &self,
        conexion: &Connection,
        consulta: &ConsultaSql,
        incluir: bool,
    ) -> Result<Vec<Rolita>, ErrorModelo> {
        let mut entrada = conexion.prepare(consulta.sql())?;
        let filas = entrada
            .query_map(rusqlite::params_from_iter(consulta.parametros()), |fila| {
                fila_a_datos(fila)
            })?;

        self.construir_rolitas(filas, incluir)
    }

    fn construir_rolitas(
        &self,
        filas: rusqlite::MappedRows<
            '_,
            impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<DatosFila>,
        >,
        incluir: bool,
    ) -> Result<Vec<Rolita>, ErrorModelo> {
        filas
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(titulo, performer, album, genero, ruta)| {
                Ok(Rolita {
                    path: ruta.clone().into(),
                    titulo,
                    performer,
                    album,
                    genero,
                    caratula: if incluir {
                        Metadatos::leer_caratula(Path::new(&ruta))?
                    } else {
                        None
                    },
                })
            })
            .collect()
    }
}

fn fila_a_datos(fila: &rusqlite::Row<'_>) -> rusqlite::Result<DatosFila> {
    Ok((
        fila.get(0)?,
        fila.get(1)?,
        fila.get(2)?,
        fila.get(3)?,
        fila.get(4)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modelo::compilador::Compilador;

    #[test]
    fn test_buscar() {
        let conexion = Connection::open_in_memory().unwrap();
        conexion
            .execute_batch(include_str!("../../rolas.sql"))
            .unwrap();
        conexion
            .execute_batch(
                "INSERT INTO performers (id_performer, id_type, name)
                 VALUES (1, 2, 'Artista');
                 INSERT INTO albums (id_album, path, name, year)
                 VALUES (1, NULL, 'Álbum', NULL);
                 INSERT INTO rolas (id_performer, id_album, path, title, year, genre)
                 VALUES
                    (1, 1, 'amor.mp3', 'Canción de amor', 2024, 'Rock'),
                    (1, 1, 'otra.mp3', 'Otra canción', 2020, 'Pop');",
            )
            .unwrap();

        let consulta = Compilador::compilar(r#"title contiene "amor""#).unwrap();
        let resultados = RolaDao::nuevo().buscar(&conexion, &consulta).unwrap();

        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].titulo, "Canción de amor");
    }
}
