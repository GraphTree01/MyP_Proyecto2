//! Analizador sintáctico: valida la secuencia de tokens y construye el AST.

use crate::modelo::error::ErrorCompilador;

use super::ast::{Literal, Nodo, Operador};
use super::lexer::{ClaseToken, Lexer, Token};

pub(super) fn analizar(texto: &str) -> Result<Nodo, ErrorCompilador> {
    let tokens = Lexer::new(texto).tokenizar()?;
    let mut parser = Parser::new(tokens);
    let raiz = parser.analizar_expresion()?;

    if !matches!(parser.actual().clase, ClaseToken::Fin) {
        return Err(parser.error_actual("Hay contenido después de la expresión"));
    }

    Ok(raiz)
}

/// Consume tokens y construye una expresión respetando la precedencia lógica.
struct Parser {
    tokens: Vec<Token>,
    indice: usize,
}

impl Parser {
    /// Crea un parser situado antes del primer token.
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, indice: 0 }
    }

    /// Analiza una expresión completa.
    fn analizar_expresion(&mut self) -> Result<Nodo, ErrorCompilador> {
        self.analizar_o()
    }

    /// Analiza disyunciones.
    fn analizar_o(&mut self) -> Result<Nodo, ErrorCompilador> {
        let mut izquierda = self.analizar_y()?;
        while self.coincide(&ClaseToken::O) {
            let derecha = self.analizar_y()?;
            izquierda = Nodo::O(Box::new(izquierda), Box::new(derecha));
        }
        Ok(izquierda)
    }

    /// Analiza conjunciones.
    fn analizar_y(&mut self) -> Result<Nodo, ErrorCompilador> {
        let mut izquierda = self.analizar_factor()?;
        while self.coincide(&ClaseToken::Y) {
            let derecha = self.analizar_factor()?;
            izquierda = Nodo::Y(Box::new(izquierda), Box::new(derecha));
        }
        Ok(izquierda)
    }

    /// Analiza negaciones, una expresión entre paréntesis o una condición.
    fn analizar_factor(&mut self) -> Result<Nodo, ErrorCompilador> {
        if self.coincide(&ClaseToken::No) {
            return Ok(Nodo::No(Box::new(self.analizar_factor()?)));
        }
        if self.coincide(&ClaseToken::AbreParentesis) {
            let expresion = self.analizar_expresion()?;
            if !self.coincide(&ClaseToken::CierraParentesis) {
                return Err(self.error_actual("Se esperaba ')'"));
            }
            return Ok(expresion);
        }
        self.analizar_condicion()
    }

    /// Analiza una condición con la forma `campo operador valor`.
    fn analizar_condicion(&mut self) -> Result<Nodo, ErrorCompilador> {
        let token = self.avanzar();
        let ClaseToken::Identificador(campo) = token.clase else {
            return Err(ErrorCompilador::Sintaxis {
                posicion: token.posicion,
                detalle: "Se esperaba el nombre de un campo".to_owned(),
            });
        };
        let token_operador = self.avanzar();
        let operador = match token_operador.clase {
            ClaseToken::Igual => Operador::Igual,
            ClaseToken::Distinto => Operador::Distinto,
            ClaseToken::Menor => Operador::Menor,
            ClaseToken::MenorIgual => Operador::MenorIgual,
            ClaseToken::Mayor => Operador::Mayor,
            ClaseToken::MayorIgual => Operador::MayorIgual,
            ClaseToken::Contiene => Operador::Contiene,
            _ => {
                return Err(ErrorCompilador::Sintaxis {
                    posicion: token_operador.posicion,
                    detalle: "Se esperaba un operador de comparación".to_owned(),
                });
            }
        };
        let token_valor = self.avanzar();
        let valor = match token_valor.clase {
            ClaseToken::Texto(texto) => Literal::Texto(texto),
            ClaseToken::Entero(numero) => Literal::Entero(numero),
            _ => {
                return Err(ErrorCompilador::Sintaxis {
                    posicion: token_valor.posicion,
                    detalle: "Se esperaba un texto entre comillas o un entero".to_owned(),
                });
            }
        };
        Ok(Nodo::Condicion {
            campo,
            operador,
            valor,
        })
    }

    /// Consume el token actual solo si pertenece a la categoría esperada.
    fn coincide(&mut self, esperado: &ClaseToken) -> bool {
        if std::mem::discriminant(&self.actual().clase) == std::mem::discriminant(esperado) {
            self.indice += 1;
            true
        } else {
            false
        }
    }

    /// Consulta el token actual sin consumirlo.
    fn actual(&self) -> &Token {
        &self.tokens[self.indice]
    }

    /// Devuelve el token actual y avanza, salvo que sea `Fin`.
    fn avanzar(&mut self) -> Token {
        let token = self.tokens[self.indice].clone();
        if !matches!(token.clase, ClaseToken::Fin) {
            self.indice += 1;
        }
        token
    }

    /// Crea un error sintáctico en la posición del token actual.
    fn error_actual(&self, detalle: &str) -> ErrorCompilador {
        ErrorCompilador::Sintaxis {
            posicion: self.actual().posicion,
            detalle: detalle.to_owned(),
        }
    }
}
