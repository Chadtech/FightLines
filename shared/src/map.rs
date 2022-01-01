use serde::{Deserialize, Serialize};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Map {
    base_tile: Tile,
    features: HashMap<Spot, Tile>,
    width: u8,
    height: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spot {
    x: u8,
    y: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Tile {
    GrassPlain,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Map {
    pub fn grass_square() -> Map {
        Map {
            base_tile: Tile::GrassPlain,
            features: HashMap::new(),
            width: 16,
            height: 16,
        }
    }
}

impl Tile {
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            Tile::GrassPlain => GRASS_PLAIN_BYTES,
        }
    }
}

const GRASS_PLAIN_BYTES: &[u8] = include_bytes!("sprites/grass_tile.png");
