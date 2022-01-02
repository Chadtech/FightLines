use serde::{Deserialize, Serialize};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Map {
    base_tile: Tile,
    features: HashMap<Spot, Tile>,
    pub grid: Vec<Vec<Tile>>,
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
                    .map(|tile| tile.clone())
                    .unwrap_or_else(|| self.base_tile.clone());

                row.push(feature);
            }

            grid.push(row);
        }

        self.grid = grid;

        self
    }
}

// impl Tile {
//     pub fn to_bytes(&self) -> &[u8] {
//         match self {
//             Tile::GrassPlain => GRASS_PLAIN_BYTES,
//         }
//     }
// }
