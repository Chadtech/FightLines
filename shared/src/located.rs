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
        self.x + 1 == other.x
    }

    pub fn is_east_of<U>(&self, other: &Located<U>) -> bool {
        self.x - 1 == other.x
    }
}
