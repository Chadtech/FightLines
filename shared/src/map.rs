use crate::tile::Tile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Map {
    base_tile: Tile,
    features: HashMap<Spot, Tile>,
    pub grid: Vec<Vec<Cell>>,
    pub width: u8,
    pub height: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Cell {
    pub tile: Tile,
    // These x and y are positions within the width and height of the map.
    // They are u16 to make them more compatible with the rendering math,
    // which will be in terms of pixels on screens wider than what a u8 can hold
    pub x: u16,
    pub y: u16,
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
                    .map(|tile| tile.clone())
                    .unwrap_or_else(|| self.base_tile.clone());

                let cell = Cell {
                    tile: feature,
                    x: x as u16,
                    y: y as u16,
                };

                row.push(cell);
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
