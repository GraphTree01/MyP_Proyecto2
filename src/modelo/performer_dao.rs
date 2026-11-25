//! Clase que sigue el patron de diseño DAO para abstraer y encapsular el
// acceso a la tabla perfomers del esquema de la base de datos.

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::modelo::error::ErrorModelo;
use crate::modelo::performer::Performer;
use crate::modelo::r#type::Type;

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
    ) -> Result<Option<i64>, ErrorModelo> {
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
    ) -> Result<i64, ErrorModelo> {
        let id_performer = match performer.id_performer {
            Some(id_performer) => id_performer,
            None => Self::siguiente_id_compartido(conexion)?,
        };
        conexion.execute(
            "INSERT INTO performers (id_performer, id_type, name) VALUES (?1, ?2, ?3)",
            params![id_performer, performer.id_type.id(), performer.name],
        )?;
        Ok(id_performer)
    }

    /// Devuelve un intérprete y convierte explícitamente su tipo del esquema.
    pub fn buscar(
        &self,
        conexion: &Connection,
        id_performer: i64,
    ) -> Result<Option<Performer>, ErrorModelo> {
        let fila = conexion
            .query_row(
                "SELECT id_performer, id_type, name
                 FROM performers WHERE id_performer = ?1",
                [id_performer],
                |fila| {
                    Ok((
                        fila.get::<_, i64>(0)?,
                        fila.get::<_, i64>(1)?,
                        fila.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        fila.map(|(id_performer, id_type, name)| {
            let id_type = Type::from_id(id_type).ok_or_else(|| {
                ErrorModelo::Validacion(format!(
                    "El intérprete {id_performer} tiene un tipo desconocido ({id_type})."
                ))
            })?;
            Ok(Performer {
                id_performer: Some(id_performer),
                id_type,
                name,
            })
        })
        .transpose()
    }

    /// Actualiza el nombre y tipo de un intérprete.
    pub fn actualizar(
        &self,
        conexion: &Connection,
        id_performer: i64,
        tipo: Type,
        nombre: &str,
    ) -> Result<bool, ErrorModelo> {
        Ok(conexion.execute(
            "UPDATE performers SET id_type = ?1, name = ?2 WHERE id_performer = ?3",
            params![tipo.id(), nombre, id_performer],
        )? > 0)
    }

    /// Elimina un intérprete solo si no está asociado a una pista.
    pub fn eliminar(&self, conexion: &Connection, id_performer: i64) -> Result<bool, ErrorModelo> {
        let usado: bool = conexion.query_row(
            "SELECT EXISTS(SELECT 1 FROM rolas WHERE id_performer = ?1)",
            [id_performer],
            |fila| fila.get(0),
        )?;
        if usado {
            return Err(ErrorModelo::Validacion(
                "No se puede eliminar el intérprete mientras tenga canciones.".to_owned(),
            ));
        }
        Ok(conexion.execute(
            "DELETE FROM performers WHERE id_performer = ?1",
            [id_performer],
        )? > 0)
    }

    /// Obtiene el siguiente ID libre entre intérpretes y detalles para mantener
    /// la convención de IDs iguales sin sobrescribir registros históricos.
    pub fn siguiente_id_compartido(conexion: &Connection) -> Result<i64, ErrorModelo> {
        Ok(conexion.query_row(
            "SELECT MAX(id) + 1 FROM (
                SELECT COALESCE(MAX(id_performer), 0) AS id FROM performers
                UNION ALL SELECT COALESCE(MAX(id_person), 0) FROM persons
                UNION ALL SELECT COALESCE(MAX(id_group), 0) FROM groups
             )",
            [],
            |fila| fila.get(0),
        )?)
    }

    /// Inserta un intérprete y el detalle asociado dentro de una transacción.
    pub fn insertar_con_detalle<F>(
        transaccion: &Transaction<'_>,
        tipo: Type,
        nombre: &str,
        detalle: F,
    ) -> Result<i64, ErrorModelo>
    where
        F: FnOnce(i64) -> Result<(), ErrorModelo>,
    {
        if tipo == Type::Unknown {
            return Err(ErrorModelo::Validacion(
                "Un intérprete desconocido no lleva un detalle personal o grupal.".to_owned(),
            ));
        }
        let id = Self::siguiente_id_compartido(transaccion)?;
        transaccion.execute(
            "INSERT INTO performers (id_performer, id_type, name) VALUES (?1, ?2, ?3)",
            params![id, tipo.id(), nombre],
        )?;
        detalle(id)?;
        Ok(id)
    }
}
