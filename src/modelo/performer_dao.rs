//! Clase que sigue el patron de diseño DAO para abstraer y encapsular el 
// acceso a la tabla perfomers del esquema de la base de datos.

use std::error::Error;

use rusqlite::{Connection, OptionalExtension, params};

use crate::modelo::performer::Performer;

/// Acceso a datos de la tabla `performers`.
pub struct PerformerDao;

impl PerformerDao {
    /// Crea un DAO de intérpretes.
    pub fn nuevo() -> Self {
        Self
    }

    /// Busca el identificador de un intérprete por su nombre.
    pub fn buscar_id_por_nombre(
        &self,
        conexion: &Connection,
        nombre: &str,
    ) -> Result<Option<i64>, Box<dyn Error>> {
        Ok(conexion
            .query_row(
                "SELECT id_performer FROM performers WHERE name = ?1 LIMIT 1",
                params![nombre],
                |fila| fila.get(0),
            )
            .optional()?)
    }

    /// Inserta un intérprete y devuelve el identificador generado.
    pub fn insertar(
        &self,
        conexion: &Connection,
        performer: &Performer,
    ) -> Result<i64, Box<dyn Error>> {
        conexion.execute(
            "INSERT INTO performers (id_type, name) VALUES (?1, ?2)",
            params![performer.id_type.id(), performer.name],
        )?;
        Ok(conexion.last_insert_rowid())
    }
}
