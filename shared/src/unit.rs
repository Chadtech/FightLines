pub mod action;

use crate::id::Id;
use crate::rng::RandGen;
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
    Tank,
}

impl Unit {
    pub fn get_mobility_range(&self) -> usize {
        match self {
            Unit::Infantry => 2,
            Unit::Tank => 8,
        }
    }
}
