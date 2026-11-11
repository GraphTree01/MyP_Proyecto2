use std::rc::Rc;

use crate::modelo::error::ErrorModelo;
use crate::modelo::operaciones::Operaciones;
use crate::vista::interfaz::Vista;

/// Punto de entrada del flujo de la aplicación.
pub struct Aplicacion {
    vista: Vista,
    operaciones: Rc<Operaciones>,
}

impl Aplicacion {
    /// Crea una aplicación lista para iniciar.
    pub fn nueva() -> Result<Self, ErrorModelo> {
        Ok(Self {
            vista: Vista::nueva(),
            operaciones: Rc::new(Operaciones::nueva("rolas.db")?),
        })
    }

    /// Consulta la biblioteca y entrega su conteo inicial a la vista.
    pub fn iniciar(&self) -> Result<(), ErrorModelo> {
        let cantidad_rolas = self.operaciones.contar_canciones()?;
        let operaciones = Rc::clone(&self.operaciones);
        self.vista.iniciar(cantidad_rolas, move |directorio| {
            operaciones
                .minar(&directorio)
                .and_then(|()| operaciones.contar_canciones())
                .map_err(|error| error.to_string())
        });
        Ok(())
    }
}
