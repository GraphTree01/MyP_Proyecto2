use rusqlite::{Connection, OptionalExtension, params};

use crate::modelo::error::ErrorModelo;
use crate::modelo::fecha;
use crate::modelo::person::Person;

/// Acceso a los registros de `persons`.
pub struct PersonDao;

impl PersonDao {
    pub fn nuevo() -> Self {
        Self
    }

    pub fn buscar(
        &self,
        conexion: &Connection,
        id_person: i64,
    ) -> Result<Option<Person>, ErrorModelo> {
        Ok(conexion
            .query_row(
                "SELECT id_person, stage_name, real_name, birth_date, death_date
                 FROM persons WHERE id_person = ?1",
                [id_person],
                |fila| {
                    Ok(Person {
                        id_person: Some(fila.get(0)?),
                        stage_name: fila.get(1)?,
                        real_name: fila.get(2)?,
                        birth_date: fila.get(3)?,
                        death_date: fila.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn listar(&self, conexion: &Connection) -> Result<Vec<Person>, ErrorModelo> {
        let mut consulta = conexion.prepare(
            "SELECT id_person, stage_name, real_name, birth_date, death_date
             FROM persons ORDER BY COALESCE(stage_name, real_name), id_person",
        )?;
        let filas = consulta.query_map([], |fila| {
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

    pub fn insertar(&self, conexion: &Connection, person: &Person) -> Result<i64, ErrorModelo> {
        let nacimiento = fecha::validar(person.birth_date.as_deref())?;
        let fallecimiento = fecha::validar(person.death_date.as_deref())?;
        validar_orden_fechas(nacimiento.as_deref(), fallecimiento.as_deref())?;
        conexion.execute(
            "INSERT INTO persons (id_person, stage_name, real_name, birth_date, death_date)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                person.id_person,
                person.stage_name,
                person.real_name,
                nacimiento,
                fallecimiento
            ],
        )?;
        Ok(conexion.last_insert_rowid())
    }

    pub fn actualizar(&self, conexion: &Connection, person: &Person) -> Result<(), ErrorModelo> {
        let id_person = person.id_person.ok_or_else(|| {
            ErrorModelo::Validacion("No se puede actualizar una persona sin ID.".to_owned())
        })?;
        let nacimiento = fecha::validar(person.birth_date.as_deref())?;
        let fallecimiento = fecha::validar(person.death_date.as_deref())?;
        validar_orden_fechas(nacimiento.as_deref(), fallecimiento.as_deref())?;
        conexion.execute(
            "UPDATE persons
             SET stage_name = ?1, real_name = ?2, birth_date = ?3, death_date = ?4
             WHERE id_person = ?5",
            params![
                person.stage_name,
                person.real_name,
                nacimiento,
                fallecimiento,
                id_person
            ],
        )?;
        Ok(())
    }

    pub fn eliminar(&self, conexion: &Connection, id_person: i64) -> Result<bool, ErrorModelo> {
        let usada: bool = conexion.query_row(
            "SELECT EXISTS(SELECT 1 FROM in_group WHERE id_person = ?1)",
            [id_person],
            |fila| fila.get(0),
        )?;
        if usada {
            return Err(ErrorModelo::Validacion(
                "No se puede eliminar la persona mientras pertenezca a un grupo.".to_owned(),
            ));
        }
        Ok(conexion.execute("DELETE FROM persons WHERE id_person = ?1", [id_person])? > 0)
    }
}

fn validar_orden_fechas(
    nacimiento: Option<&str>,
    fallecimiento: Option<&str>,
) -> Result<(), ErrorModelo> {
    if let (Some(nacimiento), Some(fallecimiento)) = (nacimiento, fallecimiento)
        && fallecimiento < nacimiento
    {
        return Err(ErrorModelo::Validacion(
            "La fecha de fallecimiento no puede ser anterior a la de nacimiento.".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn valida_fechas_antes_de_insertar_o_actualizar_personas() {
        let conexion = Connection::open_in_memory().unwrap();
        conexion
            .execute_batch(include_str!("../../rolas.sql"))
            .unwrap();
        let dao = PersonDao::nuevo();
        let persona = Person {
            id_person: Some(5),
            stage_name: Some("Artista".to_owned()),
            real_name: Some("Nombre real".to_owned()),
            birth_date: Some("1980-02-29".to_owned()),
            death_date: None,
        };

        dao.insertar(&conexion, &persona).unwrap();
        assert_eq!(
            dao.buscar(&conexion, 5)
                .unwrap()
                .unwrap()
                .birth_date
                .as_deref(),
            Some("1980-02-29")
        );
        assert!(
            dao.actualizar(
                &conexion,
                &Person {
                    birth_date: Some("1981-01-01".to_owned()),
                    death_date: Some("1980-12-31".to_owned()),
                    ..persona.clone()
                }
            )
            .is_err()
        );
        assert!(
            dao.actualizar(
                &conexion,
                &Person {
                    birth_date: Some("1980-02-30".to_owned()),
                    ..persona
                }
            )
            .is_err()
        );
    }
}
