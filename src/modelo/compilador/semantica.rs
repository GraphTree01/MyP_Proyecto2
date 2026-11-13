//! Validación semántica y traducción del AST a una condición SQL.

use crate::modelo::error::ErrorCompilador;
use rusqlite::types::Value;

use super::ast::{Literal, Nodo, Operador};

pub(super) fn compilar(nodo: &Nodo) -> Result<(String, Vec<Value>), ErrorCompilador> {
    let mut parametros = Vec::new();
    let condicion = compilar_nodo(nodo, &mut parametros)?;
    Ok((condicion, parametros))
}

/// Campo permitido en las búsquedas y su correspondencia con el esquema SQL.
#[derive(Debug)]
enum Campo {
    Titulo,
    Interprete,
    Album,
    Genero,
    Año,
    Pista,
}

impl Campo {
    /// Normaliza el campo dado por el usuario a uno de los campos permitidos.
    fn analizar(nombre: &str) -> Option<Self> {
        match nombre.to_ascii_lowercase().as_str() {
            "title" => Some(Self::Titulo),
            "performer" => Some(Self::Interprete),
            "album" => Some(Self::Album),
            "genre" => Some(Self::Genero),
            "year" => Some(Self::Año),
            "track" => Some(Self::Pista),
            _ => None,
        }
    }

    /// Devuelve la columna SQL fija asociada al campo.
    fn sql(&self) -> &'static str {
        match self {
            Self::Titulo => "r.title",
            Self::Interprete => "p.name",
            Self::Album => "a.name",
            Self::Genero => "r.genre",
            Self::Año => "r.year",
            Self::Pista => "r.track",
        }
    }

    /// Indica si el campo buscado requiere texto como "argumento".
    fn es_texto(&self) -> bool {
        matches!(
            self,
            Self::Titulo | Self::Interprete | Self::Album | Self::Genero
        )
    }

    /// Devuelve el nombre del campo para mensajes de error.
    fn nombre(&self) -> &'static str {
        match self {
            Self::Titulo => "title",
            Self::Interprete => "performer",
            Self::Album => "album",
            Self::Genero => "genre",
            Self::Año => "year",
            Self::Pista => "track",
        }
    }
}

/// Valida un nodo y devuelve los campos a buscar en SQL.
fn compilar_nodo(nodo: &Nodo, parametros: &mut Vec<Value>) -> Result<String, ErrorCompilador> {
    match nodo {
        Nodo::Condicion {
            campo,
            operador,
            valor,
        } => {
            let campo_validado = Campo::analizar(campo)
                .ok_or_else(|| ErrorCompilador::CampoDesconocido(campo.clone()))?;

            if campo_validado.es_texto() {
                if !matches!(
                    operador,
                    Operador::Igual | Operador::Distinto | Operador::Contiene
                ) {
                    return Err(ErrorCompilador::OperadorNoValido {
                        campo: campo.clone(),
                        operador: operador.nombre().to_owned(),
                    });
                }
                let Literal::Texto(valor) = valor else {
                    return Err(ErrorCompilador::TipoDeValorNoValido {
                        campo: campo.clone(),
                        esperado: "texto entre comillas".to_owned(),
                    });
                };
                parametros.push(if matches!(operador, Operador::Contiene) {
                    Value::Text(formatear_patron(valor))
                } else {
                    Value::Text(valor.clone())
                });
            } else {
                if matches!(operador, Operador::Contiene) {
                    return Err(ErrorCompilador::OperadorNoValido {
                        campo: campo.clone(),
                        operador: operador.nombre().to_owned(),
                    });
                }
                let Literal::Entero(valor) = valor else {
                    return Err(ErrorCompilador::TipoDeValorNoValido {
                        campo: campo.clone(),
                        esperado: "entero".to_owned(),
                    });
                };
                validar_rango(&campo_validado, *valor)?;
                parametros.push(Value::Integer(*valor));
            }
            let marcador = format!("?{}", parametros.len());
            let escape = if matches!(operador, Operador::Contiene) {
                " ESCAPE '\\'"
            } else {
                ""
            };
            Ok(format!(
                "COALESCE(({} {} {marcador}{escape}), 0)",
                campo_validado.sql(),
                operador.sql()
            ))
        }
        Nodo::No(expresion) => Ok(format!("(NOT {})", compilar_nodo(expresion, parametros)?)),
        Nodo::Y(izquierda, derecha) => Ok(format!(
            "({} AND {})",
            compilar_nodo(izquierda, parametros)?,
            compilar_nodo(derecha, parametros)?
        )),
        Nodo::O(izquierda, derecha) => Ok(format!(
            "({} OR {})",
            compilar_nodo(izquierda, parametros)?,
            compilar_nodo(derecha, parametros)?
        )),
    }
}

/// Comprueba si la busqueda cae en los límites validos de los campos 
// 'year' y 'track'.
fn validar_rango(campo: &Campo, valor: i64) -> Result<(), ErrorCompilador> {
    match campo {
        Campo::Año if i32::try_from(valor).is_err() => Err(ErrorCompilador::ValorFueraDeRango {
            campo: campo.nombre().to_owned(),
            tipo: "i32".to_owned(),
        }),
        Campo::Pista if u32::try_from(valor).is_err() => Err(ErrorCompilador::ValorFueraDeRango {
            campo: campo.nombre().to_owned(),
            tipo: "u32".to_owned(),
        }),
        _ => Ok(()),
    }
}

/// Obliga a que carácteres especiales dados por el usuario aparezcan 
// literalmente y Rust no los interprete como comandos especiales.
fn formatear_patron(texto: &str) -> String {
    let scape_text = texto
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{scape_text}%")
}
