//! Tipos que representan la estructura de una expresión ya analizada.

/// Nodo del árbol de sintaxis abstracta (AST).
#[derive(Debug)]
pub(super) enum Nodo {
    /// Comparación de un campo con un valor.
    Condicion {
        campo: String,
        operador: Operador,
        valor: Literal,
    },
    /// Negación lógica de una expresión.
    No(Box<Nodo>),
    /// Conjunción lógica de dos expresiones.
    Y(Box<Nodo>, Box<Nodo>),
    /// Disyunción lógica de dos expresiones.
    O(Box<Nodo>, Box<Nodo>),
}

/// Operadores de comparación aceptados por el lenguaje de búsqueda.
#[derive(Debug, Clone, Copy)]
pub(super) enum Operador {
    Igual,
    Distinto,
    Menor,
    MenorIgual,
    Mayor,
    MayorIgual,
    Contiene,
}

impl Operador {
    /// Devuelve la representación SQL equivalente.
    pub(super) fn sql(self) -> &'static str {
        match self {
            Self::Igual => "=",
            Self::Distinto => "!=",
            Self::Menor => "<",
            Self::MenorIgual => "<=",
            Self::Mayor => ">",
            Self::MayorIgual => ">=",
            Self::Contiene => "LIKE",
        }
    }

    /// Devuelve el nombre que se muestra en mensajes de error.
    pub(super) fn nombre(self) -> &'static str {
        match self {
            Self::Igual => "=",
            Self::Distinto => "!=",
            Self::Menor => "<",
            Self::MenorIgual => "<=",
            Self::Mayor => ">",
            Self::MayorIgual => ">=",
            Self::Contiene => "contiene",
        }
    }
}

/// Valor atómico de una condición, antes de validar su compatibilidad con el campo.
#[derive(Debug)]
pub(super) enum Literal {
    /// Texto delimitado por comillas en la entrada.
    Texto(String),
    /// Número entero escrito en la entrada.
    Entero(i64),
}
