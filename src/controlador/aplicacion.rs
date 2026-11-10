use crate::modelo::error::ErrorModelo;
use crate::modelo::operaciones::Operaciones;
use crate::vista::interfaz::Vista;

/// Punto de entrada del flujo de la aplicación.
pub struct Aplicacion {
    vista: Vista,
    operaciones: Operaciones,
}

impl Aplicacion {
    /// Crea una aplicación lista para iniciar.
    pub fn nueva() -> Result<Self, ErrorModelo> {
        Ok(Self {
            vista: Vista::nueva(),
            operaciones: Operaciones::nueva("rolas.db")?,
        })
    }

    /// Consulta la biblioteca y entrega su conteo inicial a la vista.
    pub fn iniciar(&self) -> Result<(), ErrorModelo> {
        let cantidad_rolas = self.operaciones.contar_canciones()?;
        self.vista.iniciar(cantidad_rolas);
        Ok(())
    }
}
