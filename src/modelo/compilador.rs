//! Compilador que analiza búsquedas y las convierte en consultas SQL válidas.

use crate::modelo::error::ErrorCompilador;
use rusqlite::types::Value;

mod ast;
mod lexer;
mod parser;
mod semantica;

use ast::Nodo;

/// Convierte expresiones de búsqueda en una consulta SQL.
///
/// Acepta los campos `title`, `performer`, `album`, `genre`, `year` y `track`;
/// los operadores `=`, `!=`, `<`, `<=`, `>` y `>=`, además de `contiene` para
/// texto. Las expresiones se combinan con `NOT`, `AND`, `OR` y paréntesis.
/// Además, se respeta la precedencia de los operadores lógicos.
pub struct Compilador;

/// Resultado del análisis sintáctico, listo para la validación semántica.
#[derive(Debug)]
pub struct Expresion {
    raiz: Nodo,
}

/// Consulta lista para ejecutarse con parámetros separados del SQL.
#[derive(Debug)]
pub struct ConsultaSql {
    sql: String,
    parametros: Vec<Value>,
}

impl ConsultaSql {
    /// Devuelve la instrucción SQL preparada.
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Devuelve los valores que se deben enlazar a los marcadores SQL.
    pub fn parametros(&self) -> &[Value] {
        &self.parametros
    }
}

impl Compilador {
    /// Analiza la sintaxis de `texto` y construye su árbol de expresión.
    pub fn analizar_sintaxis(texto: &str) -> Result<Expresion, ErrorCompilador> {
        Ok(Expresion {
            raiz: parser::analizar(texto)?,
        })
    }

    /// Valida los campos, operadores y valores, y genera una consulta sql.
    pub fn analizar_semantica(expresion: Expresion) -> Result<ConsultaSql, ErrorCompilador> {
        let (condicion, parametros) = semantica::compilar(&expresion.raiz)?;

        Ok(ConsultaSql {
            sql: format!(
                "SELECT r.title, p.name, a.name, r.genre
                 FROM rolas r
                 JOIN performers p ON p.id_performer = r.id_performer
                 JOIN albums a ON a.id_album = r.id_album
                 WHERE {condicion}
                 ORDER BY r.title, p.name, a.name"
            ),
            parametros,
        })
    }

    /// Ejecuta el análisis sintáctico y semántico de principio a fin.
    pub fn compilar(texto: &str) -> Result<ConsultaSql, ErrorCompilador> {
        Self::analizar_semantica(Self::analizar_sintaxis(texto)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compila_busqueda_no_trivial() {
        let consulta = Compilador::compilar(
            r#"title contiene "love" AND (performer = "The Beatles" OR genre = "Rock")"#,
        )
        .unwrap();

        assert!(consulta.sql().contains("r.title LIKE ?1"));
        assert!(consulta.sql().contains("p.name = ?2"));
        assert!(consulta.sql().contains("r.genre = ?3"));
        assert_eq!(
            consulta.parametros(),
            &[
                Value::Text("%love%".to_owned()),
                Value::Text("The Beatles".to_owned()),
                Value::Text("Rock".to_owned())
            ]
        );
    }

    #[test]
    fn analiza_busqueda_numerica_negada() {
        let consulta = Compilador::compilar("NOT year < 1980 OR track >= 3").unwrap();

        assert!(consulta.sql().contains("NOT COALESCE"));
        assert_eq!(
            consulta.parametros(),
            &[Value::Integer(1980), Value::Integer(3)]
        );
    }

    #[test]
    fn separa_error_sintactico_de_error_semantico() {
        let expresion = Compilador::analizar_sintaxis("artist = \"X\"").unwrap();
        assert_eq!(
            Compilador::analizar_semantica(expresion).unwrap_err(),
            ErrorCompilador::CampoDesconocido("artist".to_owned())
        );

        assert!(matches!(
            Compilador::analizar_sintaxis("title = ").unwrap_err(),
            ErrorCompilador::Sintaxis { .. }
        ));
    }

    #[test]
    fn valida_tipos_operadores_y_rangos() {
        assert!(matches!(
            Compilador::compilar("year contiene 2000").unwrap_err(),
            ErrorCompilador::OperadorNoValido { .. }
        ));
        assert!(matches!(
            Compilador::compilar("genre = 5").unwrap_err(),
            ErrorCompilador::TipoDeValorNoValido { .. }
        ));
        assert!(matches!(
            Compilador::compilar("track = -1").unwrap_err(),
            ErrorCompilador::ValorFueraDeRango { .. }
        ));
    }

    #[test]
    fn mantiene_valores_fuera_del_sql_y_escapa_patrones() {
        let consulta = Compilador::compilar(r#"title contiene "%_\"'""#).unwrap();

        assert!(!consulta.sql().contains("%_\"'"));
        assert_eq!(
            consulta.parametros(),
            &[Value::Text("%\\%\\_\"'%".to_owned())]
        );
    }

    #[test]
    fn ejecuta_la_consulta_generada_en_sqlite() {
        let conexion = rusqlite::Connection::open_in_memory().unwrap();
        conexion
            .execute_batch(include_str!("../../rolas.sql"))
            .unwrap();
        conexion
            .execute_batch(
                "INSERT INTO performers (id_performer, id_type, name)
                 VALUES (1, 2, 'Artista');
                 INSERT INTO albums (id_album, path, name, year)
                 VALUES (1, NULL, 'Álbum', NULL);
                 INSERT INTO rolas (id_performer, id_album, path, title, year, genre)
                 VALUES (1, 1, 'cancion.mp3', 'Love song', 2024, 'Rock');",
            )
            .unwrap();

        let consulta = Compilador::compilar(r#"title contiene "love" AND year >= 2000"#).unwrap();
        let mut sentencia = conexion.prepare(consulta.sql()).unwrap();
        let titulos = sentencia
            .query_map(rusqlite::params_from_iter(consulta.parametros()), |fila| {
                fila.get::<_, String>(0)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(titulos, vec!["Love song"]);
    }
}
