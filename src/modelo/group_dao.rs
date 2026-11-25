use rusqlite::{Connection, OptionalExtension, params};

use crate::modelo::error::ErrorModelo;
use crate::modelo::fecha;
use crate::modelo::group::Group;

/// Acceso a los registros de `groups`.
pub struct GroupDao;

impl GroupDao {
    pub fn nuevo() -> Self {
        Self
    }

    pub fn buscar(
        &self,
        conexion: &Connection,
        id_group: i64,
    ) -> Result<Option<Group>, ErrorModelo> {
        Ok(conexion
            .query_row(
                "SELECT id_group, name, start_date, end_date
                 FROM groups WHERE id_group = ?1",
                [id_group],
                |fila| {
                    Ok(Group {
                        id_group: Some(fila.get(0)?),
                        name: fila.get(1)?,
                        start_date: fila.get(2)?,
                        end_date: fila.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn listar(&self, conexion: &Connection) -> Result<Vec<Group>, ErrorModelo> {
        let mut consulta = conexion.prepare(
            "SELECT id_group, name, start_date, end_date FROM groups ORDER BY name, id_group",
        )?;
        let filas = consulta.query_map([], |fila| {
            Ok(Group {
                id_group: Some(fila.get(0)?),
                name: fila.get(1)?,
                start_date: fila.get(2)?,
                end_date: fila.get(3)?,
            })
        })?;
        Ok(filas.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn insertar(&self, conexion: &Connection, group: &Group) -> Result<i64, ErrorModelo> {
        let inicio = fecha::validar(group.start_date.as_deref())?;
        let fin = fecha::validar(group.end_date.as_deref())?;
        validar_orden_fechas(inicio.as_deref(), fin.as_deref())?;
        conexion.execute(
            "INSERT INTO groups (id_group, name, start_date, end_date)
             VALUES (?1, ?2, ?3, ?4)",
            params![group.id_group, group.name, inicio, fin],
        )?;
        Ok(conexion.last_insert_rowid())
    }

    pub fn actualizar(&self, conexion: &Connection, group: &Group) -> Result<(), ErrorModelo> {
        let id_group = group.id_group.ok_or_else(|| {
            ErrorModelo::Validacion("No se puede actualizar un grupo sin ID.".to_owned())
        })?;
        let inicio = fecha::validar(group.start_date.as_deref())?;
        let fin = fecha::validar(group.end_date.as_deref())?;
        validar_orden_fechas(inicio.as_deref(), fin.as_deref())?;
        conexion.execute(
            "UPDATE groups SET name = ?1, start_date = ?2, end_date = ?3
             WHERE id_group = ?4",
            params![group.name, inicio, fin, id_group],
        )?;
        Ok(())
    }

    pub fn eliminar(&self, conexion: &Connection, id_group: i64) -> Result<bool, ErrorModelo> {
        let usada: bool = conexion.query_row(
            "SELECT EXISTS(SELECT 1 FROM in_group WHERE id_group = ?1)",
            [id_group],
            |fila| fila.get(0),
        )?;
        if usada {
            return Err(ErrorModelo::Validacion(
                "No se puede eliminar el grupo mientras tenga miembros.".to_owned(),
            ));
        }
        Ok(conexion.execute("DELETE FROM groups WHERE id_group = ?1", [id_group])? > 0)
    }
}

fn validar_orden_fechas(inicio: Option<&str>, fin: Option<&str>) -> Result<(), ErrorModelo> {
    if let (Some(inicio), Some(fin)) = (inicio, fin)
        && fin < inicio
    {
        return Err(ErrorModelo::Validacion(
            "La fecha final del grupo no puede ser anterior a su fecha de inicio.".to_owned(),
        ));
    }
    Ok(())
}
