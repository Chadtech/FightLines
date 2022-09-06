use crate::id::Id;
use crate::rng::RandGen;
use crate::sprite::Sprite;
use serde::{Deserialize, Serialize};

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnitId(Id);

impl UnitId {
    pub fn new(rng: &mut RandGen) -> UnitId {
        let id = Id::new(rng);

        UnitId(id)
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Unit {
    Infantry,
}

impl Unit {
    pub fn to_sprite(&self) -> Sprite {
        match self {
            Unit::Infantry => Sprite::Infantry,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum FacingDirection {
    Left,
    Right,
}
