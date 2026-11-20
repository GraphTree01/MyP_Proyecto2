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

    /// Consulta la base de datos y entrega su conteo inicial a la vista.
    pub fn iniciar(&self) -> Result<(), ErrorModelo> {
        let rolas = self.operaciones.listar_canciones()?;
        let busqueda = Rc::clone(&self.operaciones);
        let minado = Rc::clone(&busqueda);
        self.vista.iniciar(
            rolas,
            move |directorio| minado.minar(&directorio).map_err(|error| error.to_string()),
            move |expresion, incluir_caratulas| {
                busqueda
                    .buscar_canciones_con_caratulas(&expresion, incluir_caratulas)
                    .map_err(|error| error.to_string())
            },
        );
        Ok(())
    }
}
