/// Representa un registro de la tabla `persons`, en la base de datos.
pub struct Person {
    /// Identificador generado por SQLite.
    pub id_person: Option<i64>,
    /// Nombre artístico de la persona.
    pub stage_name: Option<String>,
    /// Nombre real de la persona.
    pub real_name: Option<String>,
    /// Fecha de nacimiento, si está disponible.
    pub birth_date: Option<String>,
    /// Fecha de fallecimiento, si está disponible.
    pub death_date: Option<String>,
}
