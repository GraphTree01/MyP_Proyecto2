use std::error::Error;
use std::path::{PathBuf, Path};

use walkdir::WalkDir;


pub struct Minero {
    path: PathBuf,
}

impl Minero {

    pub fn nuevo(path: impl AsRef<Path>) -> Self {

        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn mina(&self) -> Result <(), Box<dyn Error>> {

        let buscador = WalkDir::new(&self.path).contents_first(true);

        for resultado in buscador {

            let entrada = resultado?;

            if !entrada.file_type().is_file() {
                continue;
            }

            let ruta = entrada.path();

            let mp3 = ruta
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("mp3"));

            if !mp3 {
                continue;
            }

        }

        Ok(())
    }

}