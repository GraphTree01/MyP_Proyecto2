use std::error::Error;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::modelo::metadatos::Metadatos;

pub struct Minero {
    path: PathBuf,
}

impl Minero {
    pub fn nuevo(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn mina<F>(&self, mut procesar: F) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(Metadatos) -> Result<(), Box<dyn Error>>,
    {
        let buscador = WalkDir::new(&self.path).contents_first(true);

        for resultado in buscador {
            let entrada = resultado?;

            if !entrada.file_type().is_file() {
                continue;
            }

            let ruta = entrada.path();

            if !es_mp3(ruta) {
                continue;
            }

            let metadatos = Metadatos::nuevo(ruta)?;
            procesar(metadatos)?;
        }

        Ok(())
    }
}

fn es_mp3(ruta: &Path) -> bool {
    ruta.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mp3"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use id3::{Tag, TagLike, Version};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn acepta_extensiones_validas() {
        assert!(es_mp3(Path::new("cancion.mp3")));
        assert!(es_mp3(Path::new("cancion.MP3")));
        assert!(es_mp3(Path::new("cancion.Mp3")));
    }

    #[test]
    fn rechaza_extensiones_invalidas() {
        assert!(!es_mp3(Path::new("imagen.jpg")));
        assert!(!es_mp3(Path::new("texto.txt")));
        assert!(!es_mp3(Path::new("sin_extension")));
    }

    #[test]
    fn recorre_carpetas() {
        let directorio = tempdir().unwrap();

        let mp3 = directorio.path().join("cancion.mp3");
        let notas = directorio.path().join("notas.txt");

        let subdirectorio = directorio.path()
            .join("subdirectorio")
            .join("otro_subdirectorio");

        fs::create_dir_all(&subdirectorio).unwrap();

        let mp3_anidado = subdirectorio.join("otra_cancion.mp3");

        fs::copy("beep.mp3", &mp3).unwrap();
        fs::copy("beep.mp3", &mp3_anidado).unwrap();
        fs::write(&notas, "Mis notas:....").unwrap();

        let minero = Minero::nuevo(directorio.path());
        let mut rutas = Vec::new();

        minero
            .mina(|metadatos| {
                rutas.push(metadatos.path().to_path_buf());
                Ok(())
            })
            .unwrap();

        rutas.sort();

        let mut esperadas = vec![mp3, mp3_anidado];
        esperadas.sort();

        assert_eq!(rutas, esperadas);
    }

    #[test]
    fn entrega_los_metadatos_esperados() {
        let directorio = tempdir().unwrap();
        let mp3 = directorio.path().join("cancion.mp3");

        fs::copy("beep.mp3", &mp3).unwrap();

        let mut etiqueta = Tag::new();
        etiqueta.set_title("Canción de prueba");
        etiqueta.set_artist("Artista de prueba");
        etiqueta.set_album("Álbum de prueba");
        etiqueta.set_genre("Rock");
        etiqueta.set_year(2024);
        etiqueta.set_track(7);
        etiqueta.write_to_path(&mp3, Version::Id3v24).unwrap();

        let minero = Minero::nuevo(directorio.path());
        let mut resultados = Vec::new();

        minero
            .mina(|metadatos| {
                resultados.push((
                    metadatos.title().map(str::to_owned),
                    metadatos.performer().map(str::to_owned),
                    metadatos.album().map(str::to_owned),
                    metadatos.genre().map(str::to_owned),
                    metadatos.year(),
                    metadatos.track(),
                ));
                Ok(())
            })
            .unwrap();

        assert_eq!(
            resultados,
            vec![(
                Some(String::from("Canción de prueba")),
                Some(String::from("Artista de prueba")),
                Some(String::from("Álbum de prueba")),
                Some(String::from("Rock")),
                Some(2024),
                Some(7),
            )]
        );
    }
}
