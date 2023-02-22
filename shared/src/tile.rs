use crate::unit::Unit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Tile {
    GrassPlain,
    Hills,
    Forest,
}

impl Tile {
    pub fn visibility_cost(&self) -> f32 {
        match self {
            Tile::GrassPlain => 1.0,
            Tile::Hills => 2.0,
            Tile::Forest => 3.5,
        }
    }

    pub fn mobility_cost(&self, unit: &Unit) -> f32 {
        match self {
            Tile::GrassPlain => 1.0,
            Tile::Hills => 1.5,
            Tile::Forest => match unit {
                Unit::Infantry => 1.5,
                Unit::Tank => 2.5,
                Unit::Truck => 4.0,
            },
        }
    }
}

pub const PIXEL_WIDTH: u16 = 16;
pub const PIXEL_HEIGHT: u16 = 16;

pub const PIXEL_WIDTH_FL: f64 = 16.0;
pub const PIXEL_HEIGHT_FL: f64 = 16.0;
