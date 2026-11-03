use crate::modelo::metadatos::Metadatos;

pub struct Importador {
    metadatos: Metadatos,
}

impl Importador {
    pub fn nuevo(metadatos: Metadatos) -> Self {
        Self { metadatos }
    }

    pub fn procesar(&mut self, metadatos: Metadatos) {

    }
}