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
        let max_supplies = unit.max_supplies() as f32;
        let mobility_budget = unit.mobility_budget();

        let supplies_per_tile_move = max_supplies / mobility_budget;

        let turns_until_out_of_supplies = match unit {
            Unit::Infantry => 18.0,
            Unit::Tank => 14.0,
            Unit::Truck => 16.0,
            Unit::SupplyCrate => 100000.0,
        };

        let cost_per_tile = supplies_per_tile_move / turns_until_out_of_supplies;

        let cost: f32 = match unit {
            Unit::Infantry => match self {
                Tile::GrassPlain => cost_per_tile,
                Tile::Hills => cost_per_tile,
                Tile::Forest => cost_per_tile,
            },
            Unit::Tank => {
                let base = match self {
                    Tile::GrassPlain => 1.0,
                    Tile::Hills => 1.25,
                    Tile::Forest => 1.5,
                };

                cost_per_tile * base
            }
            Unit::Truck => {
                let base = match self {
                    Tile::GrassPlain => 1.0,
                    Tile::Hills => 1.25,
                    Tile::Forest => 1.75,
                };

                cost_per_tile * base
            }
            Unit::SupplyCrate => 0.0,
        };

        let multiplier = 1.0;

        (multiplier * cost).floor() as i16
    }
}

pub const PIXEL_WIDTH: u16 = 16;
pub const PIXEL_HEIGHT: u16 = 16;

pub const PIXEL_WIDTH_FL: f64 = 16.0;
pub const PIXEL_HEIGHT_FL: f64 = 16.0;
