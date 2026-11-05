/// Representa un registro de la tabla `groups`, en la base de datos.
pub struct Group {
    /// Identificador generado por SQLite.
    pub id_group: Option<i64>,
    /// Nombre del grupo musical.
    pub name: String,
    /// Fecha de inicio del grupo, si está disponible.
    pub start_date: Option<String>,
    /// Fecha de finalización del grupo, si está disponible.
    pub end_date: Option<String>,
}
