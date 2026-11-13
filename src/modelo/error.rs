use std::error::Error;
use std::fmt;
use std::path::PathBuf;

/// Errores que pueden producir las operaciones del modelo.
#[derive(Debug)]
pub enum ErrorModelo {
    BaseDatos(rusqlite::Error),
    Recorrido(walkdir::Error),
    LecturaId3 { ruta: PathBuf, fuente: id3::Error },
    RutaNoRepresentable(PathBuf),
    TituloNoDisponible(PathBuf),
}

/// Errores sintácticos o semánticos de una expresión de búsqueda.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorCompilador {
    Sintaxis { posicion: usize, detalle: String },
    CampoDesconocido(String),
    OperadorNoValido { campo: String, operador: String },
    TipoDeValorNoValido { campo: String, esperado: String },
    ValorFueraDeRango { campo: String, tipo: String },
}

impl fmt::Display for ErrorModelo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaseDatos(fuente) => {
                write!(formatter, "Error al acceder a la base de datos: {fuente}")
            }
            Self::Recorrido(fuente) => {
                write!(formatter, "Error al recorrer el directorio: {fuente}")
            }
            Self::LecturaId3 { ruta, fuente } => {
                write!(
                    formatter,
                    "No se pudieron leer los metadatos de '{}': {fuente}",
                    ruta.display()
                )
            }
            Self::RutaNoRepresentable(ruta) => write!(
                formatter,
                "La ruta '{}' no se puede representar como texto UTF-8",
                ruta.display()
            ),
            Self::TituloNoDisponible(ruta) => write!(
                formatter,
                "No se pudo determinar el título para '{}'",
                ruta.display()
            ),
        }
    }
}

impl fmt::Display for ErrorCompilador {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sintaxis { posicion, detalle } => {
                write!(
                    formatter,
                    "Error sintáctico en la posición {posicion}: {detalle}"
                )
            }
            Self::CampoDesconocido(campo) => {
                write!(formatter, "El campo '{campo}' no existe en la búsqueda")
            }
            Self::OperadorNoValido { campo, operador } => {
                write!(
                    formatter,
                    "El operador '{operador}' no es válido para '{campo}'"
                )
            }
            Self::TipoDeValorNoValido { campo, esperado } => {
                write!(formatter, "El campo '{campo}' requiere un valor {esperado}")
            }
            Self::ValorFueraDeRango { campo, tipo } => {
                write!(
                    formatter,
                    "El valor para '{campo}' está fuera del rango de {tipo}"
                )
            }
        }
    }
}

impl Error for ErrorModelo {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BaseDatos(fuente) => Some(fuente),
            Self::Recorrido(fuente) => Some(fuente),
            Self::LecturaId3 { fuente, .. } => Some(fuente),
            Self::RutaNoRepresentable(_) | Self::TituloNoDisponible(_) => None,
        }
    }
}

impl Error for ErrorCompilador {}

impl From<rusqlite::Error> for ErrorModelo {
    fn from(fuente: rusqlite::Error) -> Self {
        Self::BaseDatos(fuente)
    }
}

impl From<walkdir::Error> for ErrorModelo {
    fn from(fuente: walkdir::Error) -> Self {
        Self::Recorrido(fuente)
    }
}
