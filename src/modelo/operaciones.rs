use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::modelo::album::Album;
use crate::modelo::album_dao::AlbumDao;
use crate::modelo::compilador::Compilador;
use crate::modelo::error::ErrorModelo;
use crate::modelo::group::Group;
use crate::modelo::group_dao::GroupDao;
use crate::modelo::importador::Importador;
use crate::modelo::in_group::InGroup;
use crate::modelo::in_group_dao::InGroupDao;
use crate::modelo::minero::Minero;
use crate::modelo::performer::Performer;
use crate::modelo::performer_dao::PerformerDao;
use crate::modelo::person::Person;
use crate::modelo::person_dao::PersonDao;
use crate::modelo::rola::Rolita;
use crate::modelo::rola_dao::RolaDao;
use crate::modelo::r#type::Type;

/// Clase que coordina las operaciones disponibles del modelo.
pub struct Operaciones {
    ruta_base: PathBuf,
    conexion: Connection,
    rola_dao: RolaDao,
    album_dao: AlbumDao,
    performer_dao: PerformerDao,
    person_dao: PersonDao,
    group_dao: GroupDao,
    in_group_dao: InGroupDao,
}

impl Operaciones {
    /// Abre la base de datos que usará el controlador.
    pub fn nueva(ruta_base: impl AsRef<Path>) -> Result<Self, ErrorModelo> {
        let ruta_base = ruta_base.as_ref().to_path_buf();
        let conexion = Connection::open(&ruta_base)?;
        conexion.execute_batch("PRAGMA foreign_keys = ON;")?;
        let cantidad_tablas: i64 = conexion.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
            [],
            |fila| fila.get(0),
        )?;

        if cantidad_tablas == 0 {
            conexion.execute_batch(include_str!("../../rolas.sql"))?;
        }

        Ok(Self {
            conexion,
            ruta_base,
            rola_dao: RolaDao::nuevo(),
            album_dao: AlbumDao::nuevo(),
            performer_dao: PerformerDao::nuevo(),
            person_dao: PersonDao::nuevo(),
            group_dao: GroupDao::nuevo(),
            in_group_dao: InGroupDao::nuevo(),
        })
    }

    /// Mina los archivos MP3 de un directorio y guarda las canciones nuevas.
    ///
    pub fn minar(&self, directorio: impl AsRef<Path>) -> Result<(), ErrorModelo> {
        let minero = Minero::nuevo(directorio);
        let mut importador = Importador::nuevo(&self.ruta_base)?;

        minero.mina(|metadatos| {
            importador.procesar(metadatos)?;
            Ok(())
        })?;

        Ok(())
    }

    /// Lista las canciones con los datos necesarios para presentarlas.
    ///
    /// Devuelve una lista vacía si la base de datos no contiene canciones.
    pub fn listar_canciones(&self) -> Result<Vec<Rolita>, ErrorModelo> {
        self.listar_canciones_con_caratulas(false)
    }

    /// Lista las canciones y carga sus carátulas solo si se solicitan.
    pub fn listar_canciones_con_caratulas(
        &self,
        incluir_caratulas: bool,
    ) -> Result<Vec<Rolita>, ErrorModelo> {
        self.rola_dao
            .listar_con_caratulas(&self.conexion, incluir_caratulas)
    }

    /// Busca canciones; una expresión vacía devuelve el listado completo.
    pub fn buscar_canciones(&self, expresion: &str) -> Result<Vec<Rolita>, ErrorModelo> {
        self.buscar_canciones_con_caratulas(expresion, false)
    }

    /// Busca canciones y carga carátulas solo si se solicitan.
    pub fn buscar_canciones_con_caratulas(
        &self,
        expresion: &str,
        incluir_caratulas: bool,
    ) -> Result<Vec<Rolita>, ErrorModelo> {
        if expresion.trim().is_empty() {
            return self.listar_canciones_con_caratulas(incluir_caratulas);
        }

        let consulta = if incluir_caratulas {
            Compilador::compilar_con_ruta(expresion)?
        } else {
            Compilador::compilar(expresion)?
        };
        self.rola_dao
            .buscar_con_caratulas(&self.conexion, &consulta, incluir_caratulas)
    }

    /// Devuelve la cantidad de canciones almacenadas en la base de datos.
    pub fn contar_canciones(&self) -> Result<i64, ErrorModelo> {
        self.rola_dao.contar(&self.conexion)
    }

    pub fn obtener_album(&self, id: i64) -> Result<Option<Album>, ErrorModelo> {
        self.album_dao.buscar(&self.conexion, id)
    }

    pub fn actualizar_album(
        &self,
        id: i64,
        nombre: &str,
        anio: Option<i32>,
    ) -> Result<bool, ErrorModelo> {
        self.album_dao.actualizar(&self.conexion, id, nombre, anio)
    }

    pub fn actualizar_rola(
        &self,
        id: i64,
        titulo: &str,
        id_performer: i64,
        id_album: i64,
    ) -> Result<bool, ErrorModelo> {
        self.rola_dao
            .actualizar(&self.conexion, id, titulo, id_performer, id_album)
    }

    pub fn obtener_performer(&self, id: i64) -> Result<Option<Performer>, ErrorModelo> {
        self.performer_dao.buscar(&self.conexion, id)
    }

    pub fn listar_performers(&self) -> Result<Vec<Performer>, ErrorModelo> {
        self.conexion
            .prepare("SELECT id_performer, id_type, name FROM performers ORDER BY name")?
            .query_map([], |fila| {
                let id_type: i64 = fila.get(1)?;
                let tipo = Type::from_id(id_type).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Integer,
                        Box::new(std::io::Error::other(format!(
                            "tipo de intérprete desconocido: {id_type}"
                        ))),
                    )
                })?;
                Ok(Performer {
                    id_performer: Some(fila.get(0)?),
                    id_type: tipo,
                    name: fila.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(ErrorModelo::from)
    }

    pub fn obtener_persona(&self, id: i64) -> Result<Option<Person>, ErrorModelo> {
        self.person_dao.buscar(&self.conexion, id)
    }

    pub fn listar_personas(&self) -> Result<Vec<Person>, ErrorModelo> {
        self.person_dao.listar(&self.conexion)
    }

    pub fn guardar_persona_independiente(&self, persona: &Person) -> Result<i64, ErrorModelo> {
        self.person_dao.insertar(&self.conexion, persona)
    }

    pub fn crear_persona(
        &self,
        persona: &Person,
        id_group: Option<i64>,
    ) -> Result<i64, ErrorModelo> {
        let transaccion = self.conexion.unchecked_transaction()?;
        let id_person = self.person_dao.insertar(&transaccion, persona)?;
        if let Some(id_group) = id_group {
            self.in_group_dao.agregar(
                &transaccion,
                &InGroup {
                    id_person,
                    id_group,
                },
            )?;
        }
        transaccion.commit()?;
        Ok(id_person)
    }

    pub fn actualizar_persona(&self, persona: &Person) -> Result<(), ErrorModelo> {
        self.person_dao.actualizar(&self.conexion, persona)
    }

    pub fn eliminar_persona(&self, id: i64) -> Result<bool, ErrorModelo> {
        self.person_dao.eliminar(&self.conexion, id)
    }

    pub fn obtener_grupo(&self, id: i64) -> Result<Option<Group>, ErrorModelo> {
        self.group_dao.buscar(&self.conexion, id)
    }

    pub fn listar_grupos(&self) -> Result<Vec<Group>, ErrorModelo> {
        self.group_dao.listar(&self.conexion)
    }

    pub fn actualizar_grupo(&self, grupo: &Group) -> Result<(), ErrorModelo> {
        self.group_dao.actualizar(&self.conexion, grupo)
    }

    pub fn guardar_performer_persona(
        &self,
        id: i64,
        nombre: &str,
        persona: &Person,
    ) -> Result<(), ErrorModelo> {
        let transaccion = self.conexion.unchecked_transaction()?;
        let tipo_actual = self
            .performer_dao
            .buscar(&transaccion, id)?
            .ok_or_else(|| {
                ErrorModelo::Validacion("No se encontró el intérprete seleccionado.".to_owned())
            })?
            .id_type;
        let persona_existente = self.person_dao.buscar(&transaccion, id)?;
        let grupo_existente = self.group_dao.buscar(&transaccion, id)?;
        if tipo_actual != Type::Person && persona_existente.is_some() {
            return Err(ErrorModelo::Validacion(
                "Ya existe una persona con este ID y no se puede asociar automáticamente al intérprete.".to_owned(),
            ));
        }
        if persona_existente.is_none() && grupo_existente.is_some() {
            return Err(ErrorModelo::Validacion(
                "Ya existe un grupo con este ID; no se puede crear una persona asociada sin resolver esa ambigüedad.".to_owned(),
            ));
        }
        self.performer_dao
            .actualizar(&transaccion, id, Type::Person, nombre)?;
        let persona = Person {
            id_person: Some(id),
            ..persona.clone()
        };
        if self.person_dao.buscar(&transaccion, id)?.is_some() {
            self.person_dao.actualizar(&transaccion, &persona)?;
        } else {
            self.person_dao.insertar(&transaccion, &persona)?;
        }
        transaccion.commit()?;
        Ok(())
    }

    pub fn guardar_performer_grupo(
        &self,
        id: i64,
        nombre: &str,
        grupo: &Group,
    ) -> Result<(), ErrorModelo> {
        let transaccion = self.conexion.unchecked_transaction()?;
        let tipo_actual = self
            .performer_dao
            .buscar(&transaccion, id)?
            .ok_or_else(|| {
                ErrorModelo::Validacion("No se encontró el intérprete seleccionado.".to_owned())
            })?
            .id_type;
        let grupo_existente = self.group_dao.buscar(&transaccion, id)?;
        let persona_existente = self.person_dao.buscar(&transaccion, id)?;
        if tipo_actual != Type::Group && grupo_existente.is_some() {
            return Err(ErrorModelo::Validacion(
                "Ya existe un grupo con este ID y no se puede asociar automáticamente al intérprete.".to_owned(),
            ));
        }
        if grupo_existente.is_none() && persona_existente.is_some() {
            return Err(ErrorModelo::Validacion(
                "Ya existe una persona con este ID; no se puede crear un grupo asociado sin resolver esa ambigüedad.".to_owned(),
            ));
        }
        self.performer_dao
            .actualizar(&transaccion, id, Type::Group, nombre)?;
        let grupo = Group {
            id_group: Some(id),
            ..grupo.clone()
        };
        if self.group_dao.buscar(&transaccion, id)?.is_some() {
            self.group_dao.actualizar(&transaccion, &grupo)?;
        } else {
            self.group_dao.insertar(&transaccion, &grupo)?;
        }
        transaccion.commit()?;
        Ok(())
    }

    pub fn guardar_performer_desconocido(
        &self,
        id: i64,
        nombre: &str,
    ) -> Result<bool, ErrorModelo> {
        self.performer_dao
            .actualizar(&self.conexion, id, Type::Unknown, nombre)
    }

    pub fn eliminar_performer(&self, id: i64) -> Result<bool, ErrorModelo> {
        let transaccion = self.conexion.unchecked_transaction()?;
        let Some(performer) = self.performer_dao.buscar(&transaccion, id)? else {
            return Ok(false);
        };
        match performer.id_type {
            Type::Person => {
                self.person_dao.eliminar(&transaccion, id)?;
            }
            Type::Group => {
                self.group_dao.eliminar(&transaccion, id)?;
            }
            Type::Unknown => {}
        }
        let eliminado = self.performer_dao.eliminar(&transaccion, id)?;
        transaccion.commit()?;
        Ok(eliminado)
    }

    pub fn eliminar_album(&self, id: i64) -> Result<bool, ErrorModelo> {
        self.album_dao.eliminar(&self.conexion, id)
    }

    pub fn eliminar_rola(&self, id: i64) -> Result<bool, ErrorModelo> {
        self.rola_dao.eliminar(&self.conexion, id)
    }

    pub fn crear_performer_persona(
        &self,
        nombre: &str,
        persona: &Person,
    ) -> Result<i64, ErrorModelo> {
        let transaccion = self.conexion.unchecked_transaction()?;
        let dao = PersonDao::nuevo();
        let id = PerformerDao::insertar_con_detalle(&transaccion, Type::Person, nombre, |id| {
            let persona = Person {
                id_person: Some(id),
                ..persona.clone()
            };
            dao.insertar(&transaccion, &persona)?;
            Ok(())
        })?;
        transaccion.commit()?;
        Ok(id)
    }

    pub fn crear_performer_grupo(&self, grupo: &Group) -> Result<i64, ErrorModelo> {
        let transaccion = self.conexion.unchecked_transaction()?;
        let dao = GroupDao::nuevo();
        let id =
            PerformerDao::insertar_con_detalle(&transaccion, Type::Group, &grupo.name, |id| {
                let grupo = Group {
                    id_group: Some(id),
                    ..grupo.clone()
                };
                dao.insertar(&transaccion, &grupo)?;
                Ok(())
            })?;
        transaccion.commit()?;
        Ok(id)
    }

    pub fn crear_performer_desconocido(&self, nombre: &str) -> Result<i64, ErrorModelo> {
        self.performer_dao.insertar(
            &self.conexion,
            &Performer {
                id_performer: None,
                id_type: Type::Unknown,
                name: nombre.to_owned(),
            },
        )
    }

    pub fn agregar_miembro(&self, id_person: i64, id_group: i64) -> Result<(), ErrorModelo> {
        self.in_group_dao.agregar(
            &self.conexion,
            &InGroup {
                id_person,
                id_group,
            },
        )
    }

    pub fn quitar_miembro(&self, id_person: i64, id_group: i64) -> Result<bool, ErrorModelo> {
        self.in_group_dao.quitar(
            &self.conexion,
            &InGroup {
                id_person,
                id_group,
            },
        )
    }

    pub fn listar_miembros(&self, id_group: i64) -> Result<Vec<Person>, ErrorModelo> {
        self.in_group_dao.listar_personas(&self.conexion, id_group)
    }
}

