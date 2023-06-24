use crate::facing_direction::FacingDirection;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn is_x_axis(&self) -> bool {
        match self {
            Direction::North => false,
            Direction::South => false,
            Direction::East => true,
            Direction::West => true,
        }
    }

    pub fn is_y_axis(&self) -> bool {
        !self.is_x_axis()
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    pub fn adjust_coord(&self, x: &mut u16, y: &mut u16) {
        match self {
            Direction::North => {
                if *y > 0 {
                    *y -= 1;
                }
            }
            Direction::South => {
                *y += 1;
            }
            Direction::East => {
                *x += 1;
            }
            Direction::West => {
                if *x > 0 {
                    *x -= 1;
                }
            }
        };
    }

    pub fn to_facing_dir(&self) -> Option<FacingDirection> {
        match self {
            Direction::North => None,
            Direction::South => None,
            Direction::East => Some(FacingDirection::Right),
            Direction::West => Some(FacingDirection::Left),
        }
    }
}

impl ToString for Direction {
    fn to_string(&self) -> String {
        match self {
            Direction::North => "north".to_string(),
            Direction::South => "south".to_string(),
            Direction::East => "east".to_string(),
            Direction::West => "west".to_string(),
        }
    }
}
