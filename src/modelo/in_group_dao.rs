use rusqlite::{Connection, params};

use crate::modelo::error::ErrorModelo;
use crate::modelo::in_group::InGroup;
use crate::modelo::person::Person;

/// Acceso a las membresías almacenadas en `in_group`.
pub struct InGroupDao;

impl InGroupDao {
    pub fn nuevo() -> Self {
        Self
    }

    /// Devuelve las personas que pertenecen al grupo indicado.
    pub fn listar_personas(
        &self,
        conexion: &Connection,
        id_group: i64,
    ) -> Result<Vec<Person>, ErrorModelo> {
        let mut consulta = conexion.prepare(
            "SELECT p.id_person, p.stage_name, p.real_name, p.birth_date, p.death_date
             FROM persons p
             JOIN in_group ig ON ig.id_person = p.id_person
             WHERE ig.id_group = ?1
             ORDER BY COALESCE(p.stage_name, p.real_name), p.id_person",
        )?;
        let filas = consulta.query_map([id_group], |fila| {
            Ok(Person {
                id_person: Some(fila.get(0)?),
                stage_name: fila.get(1)?,
                real_name: fila.get(2)?,
                birth_date: fila.get(3)?,
                death_date: fila.get(4)?,
            })
        })?;
        Ok(filas.collect::<Result<Vec<_>, _>>()?)
    }

    /// Devuelve todas las relaciones persona-grupo.
    pub fn listar(&self, conexion: &Connection) -> Result<Vec<InGroup>, ErrorModelo> {
        let mut consulta = conexion
            .prepare("SELECT id_person, id_group FROM in_group ORDER BY id_group, id_person")?;
        let filas = consulta.query_map([], |fila| {
            Ok(InGroup {
                id_person: fila.get(0)?,
                id_group: fila.get(1)?,
            })
        })?;
        Ok(filas.collect::<Result<Vec<_>, _>>()?)
    }

    /// Agrega una persona al grupo como resultado de una acción del usuario.
    pub fn agregar(&self, conexion: &Connection, relacion: &InGroup) -> Result<(), ErrorModelo> {
        conexion.execute(
            "INSERT INTO in_group (id_person, id_group) VALUES (?1, ?2)",
            params![relacion.id_person, relacion.id_group],
        )?;
        Ok(())
    }

    /// Elimina una membresía específica como resultado de una acción del usuario.
    pub fn quitar(&self, conexion: &Connection, relacion: &InGroup) -> Result<bool, ErrorModelo> {
        let eliminadas = conexion.execute(
            "DELETE FROM in_group WHERE id_person = ?1 AND id_group = ?2",
            params![relacion.id_person, relacion.id_group],
        )?;
        Ok(eliminadas > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn lista_y_actualiza_miembros_solo_por_solicitud_explicita() {
        let conexion = Connection::open_in_memory().unwrap();
        conexion
            .execute_batch(include_str!("../../rolas.sql"))
            .unwrap();
        conexion.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conexion
            .execute(
                "INSERT INTO persons (id_person, stage_name, real_name)
                 VALUES (7, 'Ozzy Osbourne', 'John Michael Osbourne')",
                [],
            )
            .unwrap();
        conexion
            .execute(
                "INSERT INTO groups (id_group, name) VALUES (9, 'Banda')",
                [],
            )
            .unwrap();
        let dao = InGroupDao::nuevo();
        let relacion = InGroup {
            id_person: 7,
            id_group: 9,
        };

        assert!(dao.listar_personas(&conexion, 9).unwrap().is_empty());
        dao.agregar(&conexion, &relacion).unwrap();
        assert_eq!(
            dao.listar_personas(&conexion, 9).unwrap()[0]
                .stage_name
                .as_deref(),
            Some("Ozzy Osbourne")
        );
        assert_eq!(dao.listar(&conexion).unwrap(), vec![relacion.clone()]);
        assert!(dao.quitar(&conexion, &relacion).unwrap());
        assert!(dao.listar_personas(&conexion, 9).unwrap().is_empty());
    }
}
