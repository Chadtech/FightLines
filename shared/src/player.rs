use crate::name::Name;
use crate::team_color::TeamColor;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Player {
    pub name: Name,
    pub color: TeamColor,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Player {
    pub fn new(name: Name, team_color: TeamColor) -> Player {
        Player {
            name,
            color: team_color,
        }
    }

    // only used for development
    pub fn red_player() -> Player {
        Player {
            name: Name::from_str("red").unwrap(),
            color: TeamColor::Red,
        }
    }

    // only used for development
    pub fn blue_player() -> Player {
        Player {
            name: Name::from_str("blue").unwrap(),
            color: TeamColor::Blue,
        }
    }
}
