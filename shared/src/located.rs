use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Located<T> {
    pub value: T,
    // These x and y are positions within the width and height of the map.
    // They are u16 to make them more compatible with the rendering math,
    // which will be in terms of pixels on screens wider than what a u8 can hold
    pub x: u16,
    pub y: u16,
}

impl<T> Located<T> {
    pub fn is_west_of<U>(&self, other: &Located<U>) -> bool {
        self.x + 1 == other.x && self.y == other.y
    }

    pub fn is_east_of<U>(&self, other: &Located<U>) -> bool {
        self.x - 1 == other.x && self.y == other.y
    }

    pub fn is_north_of<U>(&self, other: &Located<U>) -> bool {
        self.x == other.x && self.y - 1 == other.y
    }

    pub fn is_south_of<U>(&self, other: &Located<U>) -> bool {
        self.x == other.x && self.y + 1 == other.y
    }

    pub fn is_same_pos_as<U>(&self, other: &Located<U>) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn to_unit(&self) -> Located<()> {
        unit(self.x, self.y)
    }

    pub fn with_value<U>(&self, u: U) -> Located<U> {
        Located {
            x: self.x,
            y: self.y,
            value: u,
        }
    }

    pub fn map_value<U>(&self, f: impl Fn(T) -> U + Clone) -> Located<U>
    where
        T: Clone,
    {
        Located {
            x: self.x,
            y: self.y,
            value: f(self.value.clone()),
        }
    }
}

impl<T: Clone> Iterator for Located<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.value.clone())
    }
}

pub fn unit(x: u16, y: u16) -> Located<()> {
    Located { x, y, value: () }
}
