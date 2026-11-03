use crate::modelo::r#type::Type;

pub struct Performer {
    pub id_performer: Option<i64>,
    pub id_type: Type,
    pub name: String,
}
