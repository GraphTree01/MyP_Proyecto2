use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::modelo::compilador::Compilador;
use crate::modelo::error::ErrorModelo;
use crate::modelo::importador::Importador;
use crate::modelo::minero::Minero;
use crate::modelo::rola::Rolita;
use crate::modelo::rola_dao::RolaDao;

/// Clase que coordina las operaciones disponibles del modelo.
pub struct Operaciones {
    ruta_base: PathBuf,
    conexion: Connection,
    rola_dao: RolaDao,
}

impl Operaciones {
    /// Abre la base de datos que usará el controlador.
    pub fn nueva(ruta_base: impl AsRef<Path>) -> Result<Self, ErrorModelo> {
        let ruta_base = ruta_base.as_ref().to_path_buf();
        let conexion = Connection::open(&ruta_base)?;
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
}
