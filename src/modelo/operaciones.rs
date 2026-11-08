use std::error::Error;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

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
    pub fn nueva(ruta_base: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let ruta_base = ruta_base.as_ref().to_path_buf();

        Ok(Self {
            conexion: Connection::open(&ruta_base)?,
            ruta_base,
            rola_dao: RolaDao::nuevo(),
        })
    }

    /// Mina los archivos MP3 de un directorio y guarda las canciones nuevas.
    ///
    pub fn minar(&self, directorio: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
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
    pub fn listar_canciones(&self) -> Result<Vec<Rolita>, Box<dyn Error>> {
        self.rola_dao.listar(&self.conexion)
    }
}
