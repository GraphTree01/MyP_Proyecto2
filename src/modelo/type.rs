#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Person,
    Group,
    Unknown,
}

impl Type {
    pub fn id(self) -> i64 {
        match self {
            Self::Person => 0,
            Self::Group => 1,
            Self::Unknown => 2,
        }
    }

    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            0 => Some(Self::Person),
            1 => Some(Self::Group),
            2 => Some(Self::Unknown),
            _ => None,
        }
    }
}
