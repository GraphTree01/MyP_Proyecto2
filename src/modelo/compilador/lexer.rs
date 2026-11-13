//! Analizador léxico: convierte caracteres de entrada en tokens.

use crate::modelo::error::ErrorCompilador;

/// Categoría de un token reconocido en la búsqueda.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ClaseToken {
    Identificador(String),
    Texto(String),
    Entero(i64),
    Igual,
    Distinto,
    Menor,
    MenorIgual,
    Mayor,
    MayorIgual,
    Contiene,
    Y,
    O,
    No,
    AbreParentesis,
    CierraParentesis,
    Fin,
}

/// Token junto con su posición inicial en el texto.
#[derive(Debug, Clone)]
pub(super) struct Token {
    pub(super) clase: ClaseToken,
    pub(super) posicion: usize,
}

/// Recorre una búsqueda sin copiar el texto original.
pub(super) struct Lexer<'a> {
    texto: &'a str,
    indice: usize,
}

impl<'a> Lexer<'a> {
    /// Crea un lexer cuya referencia al texto vive mientras el lexer lo necesite.
    pub(super) fn new(texto: &'a str) -> Self {
        Self { texto, indice: 0 }
    }

    /// Consume toda la entrada y devuelve sus tokens.
    pub(super) fn tokenizar(mut self) -> Result<Vec<Token>, ErrorCompilador> {
        let mut tokens = Vec::new();
        while let Some(caracter) = self.caracter_actual() {
            if caracter.is_whitespace() {
                self.avanzar_caracter();
                continue;
            }

            let posicion = self.indice;
            let clase = match caracter {
                '(' => {
                    self.avanzar_caracter();
                    ClaseToken::AbreParentesis
                }
                ')' => {
                    self.avanzar_caracter();
                    ClaseToken::CierraParentesis
                }
                '"' => ClaseToken::Texto(self.leer_texto()?),
                '=' => {
                    self.avanzar_caracter();
                    if self.caracter_actual() == Some('=') {
                        self.avanzar_caracter();
                    }
                    ClaseToken::Igual
                }
                '!' => {
                    self.avanzar_caracter();
                    if self.caracter_actual() != Some('=') {
                        return Err(self.error(posicion, "Se esperaba '=' después de '!'"));
                    }
                    self.avanzar_caracter();
                    ClaseToken::Distinto
                }
                '<' => {
                    self.avanzar_caracter();
                    match self.caracter_actual() {
                        Some('=') => {
                            self.avanzar_caracter();
                            ClaseToken::MenorIgual
                        }
                        Some('>') => {
                            self.avanzar_caracter();
                            ClaseToken::Distinto
                        }
                        _ => ClaseToken::Menor,
                    }
                }
                '>' => {
                    self.avanzar_caracter();
                    if self.caracter_actual() == Some('=') {
                        self.avanzar_caracter();
                        ClaseToken::MayorIgual
                    } else {
                        ClaseToken::Mayor
                    }
                }
                '-' if self
                    .siguiente_caracter()
                    .is_some_and(|siguiente| siguiente.is_ascii_digit()) =>
                {
                    ClaseToken::Entero(self.leer_entero(posicion)?)
                }
                caracter if caracter.is_ascii_digit() => {
                    ClaseToken::Entero(self.leer_entero(posicion)?)
                }
                caracter if es_inicio_identificador(caracter) => {
                    let identificador = self.leer_identificador();
                    match identificador.to_ascii_lowercase().as_str() {
                        "and" => ClaseToken::Y,
                        "or" => ClaseToken::O,
                        "not" => ClaseToken::No,
                        "contiene" => ClaseToken::Contiene,
                        _ => ClaseToken::Identificador(identificador),
                    }
                }
                _ => return Err(self.error(posicion, "Carácter no permitido")),
            };
            tokens.push(Token { clase, posicion });
        }
        tokens.push(Token {
            clase: ClaseToken::Fin,
            posicion: self.texto.len(),
        });
        Ok(tokens)
    }

    /// Lee una cadena entre comillas y, si aplica, convierte sus secuencias de escape.
    fn leer_texto(&mut self) -> Result<String, ErrorCompilador> {
        let inicio = self.indice;
        self.avanzar_caracter();
        let mut resultado = String::new();
        while let Some(caracter) = self.caracter_actual() {
            match caracter {
                '"' => {
                    self.avanzar_caracter();
                    return Ok(resultado);
                }
                '\\' => {
                    self.avanzar_caracter();
                    let Some(escape) = self.caracter_actual() else {
                        return Err(self.error(inicio, "Secuencia de escape incompleta"));
                    };
                    resultado.push(match escape {
                        'n' => '\n',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        otro => otro,
                    });
                    self.avanzar_caracter();
                }
                _ => {
                    resultado.push(caracter);
                    self.avanzar_caracter();
                }
            }
        }
        Err(self.error(inicio, "Cadena sin comilla de cierre"))
    }

    /// Lee un entero con signo y reporta error si no cabe en `i64`.
    fn leer_entero(&mut self, posicion: usize) -> Result<i64, ErrorCompilador> {
        let inicio = self.indice;
        if self.caracter_actual() == Some('-') {
            self.avanzar_caracter();
        }
        while self
            .caracter_actual()
            .is_some_and(|caracter| caracter.is_ascii_digit())
        {
            self.avanzar_caracter();
        }
        self.texto[inicio..self.indice]
            .parse()
            .map_err(|_| self.error(posicion, "El entero está fuera del rango permitido"))
    }

    /// Consume y devuelve un identificador desde la posición actual.
    fn leer_identificador(&mut self) -> String {
        let inicio = self.indice;
        while self.caracter_actual().is_some_and(es_parte_identificador) {
            self.avanzar_caracter();
        }
        self.texto[inicio..self.indice].to_owned()
    }

    /// Devuelve el carácter que señala el índice actual, si existe.
    fn caracter_actual(&self) -> Option<char> {
        self.texto[self.indice..].chars().next()
    }

    /// Devuelve el carácter posterior al actual, si existe.
    fn siguiente_caracter(&self) -> Option<char> {
        let mut caracteres = self.texto[self.indice..].chars();
        caracteres.next()?;
        caracteres.next()
    }

    /// Avanza el índice al siguiente carácter UTF-8.
    fn avanzar_caracter(&mut self) {
        if let Some(caracter) = self.caracter_actual() {
            self.indice += caracter.len_utf8();
        }
    }

    /// Construye un error sintáctico situado en el texto de entrada.
    fn error(&self, posicion: usize, detalle: &str) -> ErrorCompilador {
        ErrorCompilador::Sintaxis {
            posicion,
            detalle: detalle.to_owned(),
        }
    }
}

/// Indica si el carácter puede iniciar un identificador (operador lógico).
fn es_inicio_identificador(caracter: char) -> bool {
    caracter.is_ascii_alphabetic() || caracter == '_'
}

/// Indica si el carácter puede aparecer después del inicio de un identificador.
fn es_parte_identificador(caracter: char) -> bool {
    caracter.is_ascii_alphanumeric() || caracter == '_'
}
