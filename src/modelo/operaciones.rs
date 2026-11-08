use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::modelo::rola::Rolita;
use crate::modelo::rola_dao::RolaDao;

/// Clase que coordina las operaciones disponibles del modelo.
pub struct Operaciones {
    conexion: Connection,
    rola_dao: RolaDao,
}

impl Operaciones {
    /// Abre la base de datos que usará el controlador.
    pub fn nueva(ruta_base: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            conexion: Connection::open(ruta_base)?,
            rola_dao: RolaDao::nuevo(),
        })
    }

    /// Lista las canciones con los datos necesarios para presentarlas.
    ///
    /// Devuelve una lista vacía si la base de datos no contiene canciones.
    pub fn listar_canciones(&self) -> Result<Vec<Rolita>, Box<dyn Error>> {
        self.rola_dao.listar(&self.conexion)
    }
}
