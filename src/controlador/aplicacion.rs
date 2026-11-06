use crate::vista::vista::Vista;

/// Punto de entrada del flujo de la aplicación.
pub struct Aplicacion {
    vista: Vista,
}

impl Aplicacion {
    /// Crea una aplicación lista para iniciar.
    pub fn nueva() -> Self {
        Self {
            vista: Vista::nueva(),
        }
    }

    /// Delega el inicio del programa a la capa de vista.
    pub fn iniciar(&self) {
        self.vista.iniciar();
    }
}
