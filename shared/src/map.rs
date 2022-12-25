use crate::facing_direction::FacingDirection;
use crate::located::Located;
use crate::tile::Tile;
use crate::unit::Unit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Map {
    base_tile: Tile,
    features: HashMap<Spot, Tile>,
    pub grid: Vec<Vec<Located<Tile>>>,
    pub width: u8,
    pub height: u8,
}

impl Map {
    pub fn grass_square() -> Map {
        let features = HashMap::new();

        Map {
            base_tile: Tile::GrassPlain,
            features,
            grid: Vec::new(),
            width: 16,
            height: 16,
        }
        .sync_grid()
    }

    fn sync_grid(mut self) -> Map {
        let mut grid = Vec::with_capacity(self.height as usize);

        for y in 0..self.height {
            let mut row = Vec::with_capacity(self.width as usize);

            for x in 0..self.width {
                let feature = self
                    .features
                    .get(&Spot { x, y })
                    .cloned()
                    .unwrap_or_else(|| self.base_tile.clone());

                let loc_tile = Located::<Tile> {
                    value: feature,
                    x: x as u16,
                    y: y as u16,
                };

                row.push(loc_tile);
            }

            grid.push(row);
        }

        self.grid = grid;

        self
    }

    pub fn dimensions(&self) -> (u8, u8) {
        (self.width, self.height)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Spot {
    x: u8,
    y: u8,
}

pub enum MapOpt {
    GrassSquare,
}

pub struct Militaries {
    pub first_player_military: Vec<Located<(FacingDirection, Unit)>>,
    pub second_player_military: Vec<Located<(FacingDirection, Unit)>>,
    pub rest_players_miliatries: Vec<Vec<Located<(FacingDirection, Unit)>>>,
}

impl MapOpt {
    pub fn initial_militaries(&self) -> Militaries {
        let map = self.to_map();
        match self {
            MapOpt::GrassSquare => Militaries {
                first_player_military: vec![
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Infantry),
                        x: 2,
                        y: 2,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Tank),
                        x: 3,
                        y: 4,
                    },
                ],
                second_player_military: vec![
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Infantry),
                        x: (map.width as u16) - 3,
                        y: (map.height as u16) - 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Tank),
                        x: (map.width as u16) - 4,
                        y: (map.height as u16) - 5,
                    },
                ],
                rest_players_miliatries: vec![],
            },
        }
    }

    pub fn player_count(&self) -> u8 {
        2 + (self.initial_militaries().rest_players_miliatries.len() as u8)
    }

    pub fn to_map(&self) -> Map {
        match self {
            MapOpt::GrassSquare => Map::grass_square(),
        }
    }
}
