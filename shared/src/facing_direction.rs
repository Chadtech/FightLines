use crate::direction::Direction;
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
    pub fn from_directions(dirs: Vec<Direction>) -> Option<FacingDirection> {
        let mut ret = None;

        let mut dirs = dirs.clone();
        dirs.reverse();

        for dir in dirs {
            match dir {
                Direction::North | Direction::South => {}
                Direction::East => {
                    ret = Some(FacingDirection::Right);
                    break;
                }
                Direction::West => {
                    ret = Some(FacingDirection::Left);
                    break;
                }
            }
        }

        ret
    }
}

pub const ALL: &[FacingDirection] = &[FacingDirection::Left, FacingDirection::Right];
