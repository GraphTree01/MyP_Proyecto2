/// Relación entre una persona y un grupo musical.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InGroup {
    pub id_person: i64,
    pub id_group: i64,
}
