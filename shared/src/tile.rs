use crate::sprite::Sprite;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Tile {
    GrassPlain,
}

impl Tile {
    pub fn to_sprite(&self) -> Sprite {
        match self {
            Tile::GrassPlain => Sprite::GrassTile,
        }
    }
}

pub const PIXEL_WIDTH: u16 = 16;
pub const PIXEL_HEIGHT: u16 = 16;

pub const PIXEL_WIDTH_FL: f64 = 16.0;
pub const PIXEL_HEIGHT_FL: f64 = 16.0;
