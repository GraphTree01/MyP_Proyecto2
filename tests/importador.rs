//!Pruebas de integración del Importador; componente del Modelo del proyecto.

use std::fs;

use id3::{Tag, TagLike, Version};
use proyecto2::modelo::importador::Importador;
use proyecto2::modelo::metadatos::Metadatos;
use rusqlite::Connection;
use std::path::PathBuf;
use tempfile::tempdir;

/// Verifica el flujo completo de lectura, importación y persistencia de una
/// canción, incluyendo la prevención de duplicados.
#[test]
fn importa_canciones() {
    let directorio = tempdir().unwrap();
    let ruta_base = directorio.path().join("prueba.db");
    let ruta_mp3 = directorio.path().join("cancion.mp3");
    let archivo_mp3 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("beep.mp3");

    fs::copy(archivo_mp3, &ruta_mp3).unwrap();

    let mut etiqueta = Tag::new();
    etiqueta.set_title("Canción génerica");
    etiqueta.set_artist("Artista del momento");
    etiqueta.set_album("Álbum cualquiera");
    etiqueta.set_genre("Rock");
    etiqueta.set_year(1980);
    etiqueta.set_track(10);
    etiqueta.write_to_path(&ruta_mp3, Version::Id3v24).unwrap();

    let conexion = Connection::open(&ruta_base).unwrap();
    conexion
        .execute_batch(include_str!("../rolas.sql"))
        .unwrap();
    drop(conexion);

    let mut importador = Importador::nuevo(&ruta_base).unwrap();
    let metadatos = Metadatos::nuevo(&ruta_mp3).unwrap();

    assert!(importador.procesar(metadatos).unwrap());

    let metadatos_repetidos = Metadatos::nuevo(&ruta_mp3).unwrap();
    assert!(!importador.procesar(metadatos_repetidos).unwrap());

    let conexion = Connection::open(&ruta_base).unwrap();
    let fila = conexion
        .query_row(
            "SELECT r.title, p.name, a.name, r.genre, r.year, r.track
             FROM rolas r
             JOIN performers p ON p.id_performer = r.id_performer
             JOIN albums a ON a.id_album = r.id_album",
            [],
            |fila| {
                Ok((
                    fila.get::<_, String>(0)?,
                    fila.get::<_, String>(1)?,
                    fila.get::<_, String>(2)?,
                    fila.get::<_, Option<String>>(3)?,
                    fila.get::<_, Option<i32>>(4)?,
                    fila.get::<_, Option<u32>>(5)?,
                ))
            },
        )
        .unwrap();

    assert_eq!(
        fila,
        (
            "Canción génerica".to_owned(),
            "Artista del momento".to_owned(),
            "Álbum cualquiera".to_owned(),
            Some("Rock".to_owned()),
            Some(1980),
            Some(10),
        )
    );

    let cantidad: i64 = conexion
        .query_row("SELECT COUNT(*) FROM rolas", [], |fila| fila.get(0))
        .unwrap();
    assert_eq!(cantidad, 1);
}
