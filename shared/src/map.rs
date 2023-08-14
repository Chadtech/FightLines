use crate::facing_direction::FacingDirection;
use crate::located::Located;
use crate::tile::Tile;
use crate::unit::Unit;
use crate::{located, tile};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};

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

    pub fn display_test() -> Map {
        let mut features = HashMap::new();

        let block_size = 4;
        let block_size_plus_gap = block_size + 1;

        let size: u16 = ((tile::ALL.len() * block_size_plus_gap) + 1) as u16;

        for (index, tile) in tile::ALL.iter().enumerate() {
            for y in 0..block_size {
                for x in 0..block_size {
                    let loc_x = (index * block_size_plus_gap) + x;
                    features.insert(located::unit(loc_x as u16, y as u16), tile.clone());
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

    pub fn replenish_test() -> Map {
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum MapOpt {
    GrassSquare,
    TerrainTest,
    DisplayTest,
    ReplenishTest,
    ArrowTest,
    GamePlayTest,
    SingleUnitTest,
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
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::SupplyCrate),
                        x: 6,
                        y: 6,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::SupplyCrate),
                        x: 8,
                        y: 6,
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
            MapOpt::DisplayTest => StartingUnits {
                first_player_military: vec![
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Infantry),
                        x: 2,
                        y: 2,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Infantry),
                        x: 3,
                        y: 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Infantry),
                        x: 2,
                        y: 3,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Tank),
                        x: 2,
                        y: 4,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::Truck),
                        x: 2,
                        y: 6,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Right, Unit::SupplyCrate),
                        x: 2,
                        y: 10,
                    },
                ],
                second_player_military: vec![
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Infantry),
                        x: 4,
                        y: 2,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Tank),
                        x: 4,
                        y: 4,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::Truck),
                        x: 4,
                        y: 6,
                    },
                    Located::<(FacingDirection, Unit)> {
                        value: (FacingDirection::Left, Unit::SupplyCrate),
                        x: 4,
                        y: 10,
                    },
                ],
                rest_players_militatries: vec![],
            },
            MapOpt::ReplenishTest => StartingUnits {
                first_player_military: vec![],
                second_player_military: vec![Located::<(FacingDirection, Unit)> {
                    value: (FacingDirection::Right, Unit::Truck),
                    x: 4,
                    y: 8,
                }],
                rest_players_militatries: vec![],
            },
            MapOpt::ArrowTest => StartingUnits {
                first_player_military: vec![],
                second_player_military: vec![Located::<(FacingDirection, Unit)> {
                    value: (FacingDirection::Right, Unit::Truck),
                    x: 4,
                    y: 8,
                }],
                rest_players_militatries: vec![],
            },
            MapOpt::GamePlayTest => StartingUnits {
                first_player_military: vec![],
                second_player_military: vec![],
                rest_players_militatries: vec![],
            },
            MapOpt::SingleUnitTest => StartingUnits {
                first_player_military: vec![Located::<(FacingDirection, Unit)> {
                    value: (FacingDirection::Right, Unit::Infantry),
                    x: 0,
                    y: 0,
                }],
                second_player_military: vec![Located::<(FacingDirection, Unit)> {
                    value: (FacingDirection::Right, Unit::Infantry),
                    x: 0,
                    y: 1,
                }],
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
            MapOpt::DisplayTest => Map::display_test(),
            MapOpt::ReplenishTest => Map::replenish_test(),
            MapOpt::ArrowTest => DevFlags {
                base_tile: Tile::GrassPlain,
                src: r#"
FFFFFFFFFFFFFFFF
F      FF      F
F      FF      F
F      FF      F
F      FF      F
F              F
F      FF      F
F      FF      F
F      FF      F
F      FF      F
F      FF      F
F      FF      F
F      FF      F
F      FF      F
F      FF      F
FFFFFFFFFFFFFFFF
"#
                .to_string(),
            }
            .try_into()
            .unwrap(),
            MapOpt::GamePlayTest => DevFlags {
                base_tile: Tile::GrassPlain,
                src: r#"
FFF FFFFFF  FFFF
FFF  FF  FF  FFF
F     HHHH     G
F      HHF F   G
F   F  FF  F   G
F  FF      F   G
G      FF      G
G    H FF H    G
G    H FF H    G
G   HHHFFHHH   G
F      FHHHHH  G
F HHH   F  HH  G
F   HH      H  G
FF  HH FF      G
FFF  FF FF    FF
FFF  FF      FFF
"#
                .to_string(),
            }
            .try_into()
            .unwrap(),
            MapOpt::SingleUnitTest => DevFlags {
                base_tile: Tile::GrassPlain,
                src: r#"
GGGGGGGG
GGGGGGGG
GGGGGGGG
GGGGGGGG
GGGGGGGG
GGGGGGGG
GGGGGGGG
GGGGGGGG
"#
                .to_string(),
            }
            .try_into()
            .unwrap(),
        }
    }
}

pub struct DevFlags {
    pub base_tile: Tile,
    pub src: String,
}

impl TryFrom<DevFlags> for Map {
    type Error = String;

    fn try_from(flags: DevFlags) -> Result<Self, Self::Error> {
        let rows = flags
            .src
            .trim()
            .to_string()
            .split('\n')
            .map(|str| str.to_string())
            .collect::<Vec<String>>();

        let row_lengths = rows
            .iter()
            .map(|row| row.len())
            .collect::<HashSet<usize>>()
            .into_iter()
            .collect::<Vec<usize>>();

        if row_lengths.len() != 1 {
            return Err("not all rows are the same length".to_string());
        }

        let width = row_lengths.first().unwrap();

        let mut features = HashMap::new();
        for (ri, row) in rows.iter().enumerate() {
            for (ci, col) in row.chars().enumerate() {
                let tile = match col {
                    ' ' => flags.base_tile.clone(),
                    'G' => Tile::GrassPlain,
                    'H' => Tile::Hills,
                    'F' => Tile::Forest,
                    _ => {
                        let mut err_msg = "unrecognized char for making dev map: ".to_string();

                        err_msg.push(col);

                        return Err(err_msg);
                    }
                };

                features.insert(located::unit(ci as u16, ri as u16), tile);
            }
        }

        Ok(Map {
            base_tile: flags.base_tile,
            features,
            grid: vec![],
            width: *width as u16,
            height: rows.len() as u16,
        }
        .sync_grid())
    }
}
