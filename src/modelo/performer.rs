use crate::modelo::r#type::Type;

/// Representa un registro de la tabla `performers`, en la base de datos.
pub struct Performer {
    /// Identificador generado por SQLite.
    pub id_performer: Option<i64>,
    /// Tipo de intérprete: persona, grupo o desconocido.
    pub id_type: Type,
    /// Nombre del intérprete.
    pub name: String,
}
