use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum FacingDirection {
    Left,
    Right,
}

impl FacingDirection {
    pub fn to_file_name_str(&self) -> &str {
        match self {
            FacingDirection::Left => "-l",
            FacingDirection::Right => "",
        }
    }
    pub fn to_human_readable_label(&self) -> &str {
        match self {
            FacingDirection::Left => "left",
            FacingDirection::Right => "right",
        }
    }
}

pub const ALL: &[FacingDirection] = &[FacingDirection::Left, FacingDirection::Right];
