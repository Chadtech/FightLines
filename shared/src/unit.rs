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
impl ToString for UnitId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Unit {
    Infantry,
    Tank,
    Truck,
}

impl Unit {
    pub fn get_mobility_range(&self) -> usize {
        match self {
            Unit::Infantry => 2,
            Unit::Tank => 6,
            Unit::Truck => 7,
        }
    }
}

impl ToString for Unit {
    fn to_string(&self) -> String {
        match self {
            Unit::Infantry => "infantry".to_string(),
            Unit::Tank => "tanks".to_string(),
            Unit::Truck => "truck".to_string(),
        }
    }
}
