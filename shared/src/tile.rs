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
        match unit.active_supply_cost() {
            Some(cost_per_tile) => {
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
            None => 0,
        }
    }
}

pub const PIXEL_WIDTH: u16 = 16;
pub const PIXEL_HEIGHT: u16 = 16;

pub const PIXEL_WIDTH_FL: f64 = 16.0;
pub const PIXEL_HEIGHT_FL: f64 = 16.0;

pub const ALL: [Tile; 3] = [Tile::GrassPlain, Tile::Hills, Tile::Forest];

#[cfg(test)]
mod test_tiles {
    use crate::tile::Tile;
    use crate::unit::Unit;
    use pretty_assertions::assert_eq;

    #[test]
    fn infantry_mobility_cost() {
        let want = 42;
        assert_eq!(want, Tile::GrassPlain.travel_supply_cost(&Unit::Infantry));
    }
}
