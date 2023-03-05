pub mod action;
pub mod place;

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
    SupplyCrate,
}

impl Unit {
    pub fn mobility_budget(&self) -> f32 {
        match self {
            Unit::Infantry => 3.0,
            Unit::Tank => 6.0,
            Unit::Truck => 8.0,
            Unit::SupplyCrate => 0.0,
        }
    }

    pub fn is_rideable(&self) -> bool {
        match self {
            Unit::Infantry => false,
            Unit::Tank => false,
            Unit::Truck => true,
            Unit::SupplyCrate => false,
        }
    }

    pub fn can_carry(&self, carry_unit: &Unit) -> bool {
        match self {
            Unit::Infantry => false,
            Unit::Tank => false,
            Unit::Truck => match carry_unit {
                Unit::Infantry => true,
                Unit::Tank => false,
                Unit::Truck => false,
                Unit::SupplyCrate => true,
            },
            Unit::SupplyCrate => false,
        }
    }

    pub fn visibility_budget(&self) -> f32 {
        match self {
            Unit::Infantry => 3.5,
            Unit::Tank => 2.0,
            Unit::Truck => 3.0,
            Unit::SupplyCrate => 0.0,
        }
    }

    pub fn max_supplies(&self) -> u16 {
        match self {
            Unit::Infantry => 1024,
            Unit::Tank => 3072,
            Unit::Truck => 2048,
            Unit::SupplyCrate => 8192,
        }
    }
}

impl ToString for Unit {
    fn to_string(&self) -> String {
        match self {
            Unit::Infantry => "infantry".to_string(),
            Unit::Tank => "tank".to_string(),
            Unit::Truck => "truck".to_string(),
            Unit::SupplyCrate => "supply crate".to_string(),
        }
    }
}
