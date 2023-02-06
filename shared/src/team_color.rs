use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TeamColor {
    Red,
    Blue,
}

impl ToString for TeamColor {
    fn to_string(&self) -> String {
        match self {
            TeamColor::Red => "red".to_string(),
            TeamColor::Blue => "blue".to_string(),
        }
    }
}
