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
                Unit::SupplyCrate => 512.0,
            },
        }
    }

    pub fn travel_supply_cost(&self, unit: &Unit) -> i16 {
        let multiplier = 16.0;

        let cost: f32 = match unit {
            Unit::Infantry => {
                let infantry_cost = 4.0;
                match self {
                    Tile::GrassPlain => infantry_cost,
                    Tile::Hills => infantry_cost,
                    Tile::Forest => infantry_cost,
                }
            }
            Unit::Tank => match self {
                Tile::GrassPlain => 1.0,
                Tile::Hills => 1.25,
                Tile::Forest => 1.5,
            },
            Unit::Truck => match self {
                Tile::GrassPlain => 1.0,
                Tile::Hills => 1.25,
                Tile::Forest => 1.75,
            },
            Unit::SupplyCrate => 0.0,
        };

        (multiplier * cost).floor() as i16
    }
}

pub const PIXEL_WIDTH: u16 = 16;
pub const PIXEL_HEIGHT: u16 = 16;

pub const PIXEL_WIDTH_FL: f64 = 16.0;
pub const PIXEL_HEIGHT_FL: f64 = 16.0;
