use std::error::Error;

use rusqlite::{Connection, OptionalExtension, params};

use crate::modelo::performer::Performer;

pub struct PerformerDao;

impl PerformerDao {
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
                "SELECT id_performer FROM performers WHERE name = ?1 LIMIT 1",
                params![nombre],
                |fila| fila.get(0),
            )
            .optional()?)
    }

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
