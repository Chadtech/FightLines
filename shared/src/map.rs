use crate::facing_direction::FacingDirection;
use crate::located;
use crate::located::Located;
use crate::tile::Tile;
use crate::unit::Unit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Map {
    base_tile: Tile,
    pub features: HashMap<Located<()>, Tile>,
    pub grid: Vec<Vec<Located<Tile>>>,
    pub width: u16,
    pub height: u16,
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

    pub fn terrain_test() -> Map {
        let mut features = HashMap::new();

        let size: u16 = 32;

        for x in 0..size {
            for y in 0..size {
                if x % 3 != 0 && y % 3 != 0 {
                    features.insert(Located { x, y, value: () }, Tile::Hills);
                }

                if (x + y) % 5 > 2 && y % 7 > 2 {
                    features.insert(Located { x, y, value: () }, Tile::Forest);
                }
            }
        }

        Map {
            base_tile: Tile::GrassPlain,
            features,
            grid: Vec::new(),
            width: size,
            height: size,
        }
        .sync_grid()
    }

    fn sync_grid(mut self) -> Map {
        let mut grid = Vec::with_capacity(self.height as usize);

        for y in 0..self.height {
            let mut row = Vec::with_capacity(self.width as usize);

            for x in 0..self.width {
                let feature = self.get_tile(&located::unit(x, y));

                let loc_tile = Located::<Tile> {
                    value: feature,
                    x,
                    y,
                };

                row.push(loc_tile);
            }

            grid.push(row);
        }

        self.grid = grid;

        self
    }

    pub fn get_tile(&self, loc: &Located<()>) -> Tile {
        self.features
            .get(loc)
            .cloned()
            .unwrap_or_else(|| self.base_tile.clone())
    }
}

pub enum MapOpt {
    GrassSquare,
    TerrainTest,
}

pub struct StartingUnits {
    pub first_player_military: Vec<Located<(FacingDirection, Unit)>>,
    pub second_player_military: Vec<Located<(FacingDirection, Unit)>>,
    pub rest_players_militatries: Vec<Vec<Located<(FacingDirection, Unit)>>>,
}

impl MapOpt {
    pub fn initial_units(&self) -> StartingUnits {
        let map = self.to_map();
        match self {
            MapOpt::GrassSquare => StartingUnits {
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
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Truck),
                        x: 4,
                        y: 2,
                    },
                ],
                second_player_military: vec![
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Infantry),
                        x: map.width - 3,
                        y: map.height - 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Tank),
                        x: map.width - 4,
                        y: map.height - 5,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Truck),
                        x: map.width - 5,
                        y: map.height - 3,
                    },
                ],
                rest_players_militatries: vec![],
            },
            MapOpt::TerrainTest => StartingUnits {
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
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Truck),
                        x: 4,
                        y: 2,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Truck),
                        x: 5,
                        y: 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Infantry),
                        x: 3,
                        y: 3,
                    },
                ],
                second_player_military: vec![
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Infantry),
                        x: map.width - 3,
                        y: map.height - 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Infantry),
                        x: map.width - 2,
                        y: map.height - 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Tank),
                        x: map.width - 4,
                        y: map.height - 5,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Truck),
                        x: map.width - 5,
                        y: map.height - 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Truck),
                        x: map.width - 6,
                        y: map.height - 3,
                    },
                ],
                rest_players_militatries: vec![],
            },
        }
    }

    pub fn player_count(&self) -> u8 {
        2 + (self.initial_units().rest_players_militatries.len() as u8)
    }

    pub fn to_map(&self) -> Map {
        match self {
            MapOpt::GrassSquare => Map::grass_square(),
            MapOpt::TerrainTest => Map::terrain_test(),
        }
    }
}
