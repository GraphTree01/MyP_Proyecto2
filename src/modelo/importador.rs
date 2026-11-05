use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::modelo::album::Album;
use crate::modelo::album_dao::AlbumDao;
use crate::modelo::metadatos::Metadatos;
use crate::modelo::performer::Performer;
use crate::modelo::performer_dao::PerformerDao;
use crate::modelo::rola::Rola;
use crate::modelo::rola_dao::RolaDao;
use crate::modelo::r#type::Type;

/// Coordina la importación de metadatos hacia la base de datos.
///
/// Cada canción se procesa dentro de una transacción. El importador aprovecha
/// que la base de datos es relacional para evitar repetir canciones.
pub struct Importador {
    conexion: Connection,
    rola_dao: RolaDao,
    album_dao: AlbumDao,
    performer_dao: PerformerDao,
}

impl Importador {
    /// Instancia la base de datos y activa la validación de claves foráneas.
    pub fn nuevo(ruta_base: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let conexion = Connection::open(ruta_base)?;
        conexion.execute_batch("PRAGMA foreign_keys = ON;")?;

        Ok(Self {
            conexion,
            rola_dao: RolaDao::nuevo(),
            album_dao: AlbumDao::nuevo(),
            performer_dao: PerformerDao::nuevo(),
        })
    }

    /// Inserta una canción y sus relaciones si la ruta aún no existe.
    ///
    /// Devuelve `true` cuando se inserta una canción nueva y `false` cuando
    /// ya había una canción con la misma ruta.
    pub fn procesar(&mut self, metadatos: Metadatos) -> Result<bool, Box<dyn Error>> {
        let transaccion = self.conexion.transaction()?;

        if self.rola_dao.existe(&transaccion, metadatos.path())? {
            transaccion.rollback()?;
            return Ok(false);
        }

        let titulo = metadatos
            .title()
            .map(String::from)
            .or_else(|| {
                metadatos
                    .path()
                    .file_stem()
                    .and_then(|nombre| nombre.to_str())
                    .map(String::from)
            })
            .ok_or("No se pudo determinar el título de la rola")?;

        let performer_nombre = metadatos.performer().unwrap_or("Unknown");
        let id_performer = match self
            .performer_dao
            .buscar_id_por_nombre(&transaccion, performer_nombre)?
        {
            Some(id) => id,
            None => self.performer_dao.insertar(
                &transaccion,
                &Performer {
                    id_performer: None,
                    id_type: Type::Unknown,
                    name: performer_nombre.to_owned(),
                },
            )?,
        };

        let album_nombre = metadatos.album().unwrap_or("Unknown");
        let id_album = match self
            .album_dao
            .buscar_id_por_nombre(&transaccion, album_nombre)?
        {
            Some(id) => id,
            None => self.album_dao.insertar(
                &transaccion,
                &Album {
                    id_album: None,
                    path: None,
                    name: album_nombre.to_owned(),
                    year: metadatos.year(),
                },
            )?,
        };

        self.rola_dao.insertar(
            &transaccion,
            &Rola {
                id_rola: None,
                id_performer,
                id_album,
                path: metadatos.path().to_path_buf(),
                title: titulo,
                track: metadatos.track(),
                year: metadatos.year(),
                genre: metadatos.genre().map(String::from),
            },
        )?;

        transaccion.commit()?;
        Ok(true)
    }
}
