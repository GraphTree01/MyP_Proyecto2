use std::error::Error;

use proyecto2::controlador::aplicacion::Aplicacion;

fn main() -> Result<(), Box<dyn Error>> {
    let aplicacion = Aplicacion::nueva()?;
    aplicacion.iniciar()?;
    Ok(())
}
