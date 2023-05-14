pub mod action;

use crate::facing_direction::FacingDirection;
use crate::id::Id;
use crate::located::Located;
use crate::rng::RandGen;
use crate::team_color::TeamColor;
use serde::{Deserialize, Serialize};

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Model {
    pub unit: Unit,
    pub place: Place,
    pub owner: Option<Id>,
    pub color: TeamColor,
    pub name: Option<String>,
    pub supplies: i16,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Deleted {
    pub unit: Unit,
    pub owner: Option<Id>,
    pub color: TeamColor,
    pub name: Option<String>,
}

impl From<Model> for Deleted {
    fn from(unit_model: Model) -> Deleted {
        Deleted {
            unit: unit_model.unit,
            owner: unit_model.owner,
            color: unit_model.color,
            name: unit_model.name,
        }
    }
}

impl From<&Model> for Deleted {
    fn from(unit_model: &Model) -> Deleted {
        Deleted {
            unit: unit_model.unit.clone(),
            owner: unit_model.owner.clone(),
            color: unit_model.color.clone(),
            name: unit_model.name.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Place {
    OnMap(Located<FacingDirection>),
    InUnit(UnitId),
}

impl Place {
    pub fn is_on_map(&self) -> bool {
        match self {
            Place::OnMap(_) => true,
            Place::InUnit(_) => false,
        }
    }
}

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

    pub fn max_supplies(&self) -> i16 {
        match self {
            Unit::Infantry => 1024,
            Unit::Tank => 3072,
            Unit::Truck => 2048,
            Unit::SupplyCrate => 8192,
        }
    }
    // The idealized number of turns a unit would
    // last if it did nothing at all
    pub fn supply_lifespan(&self) -> f32 {
        match self {
            Unit::Infantry => 18.0,
            Unit::Tank => 14.0,
            Unit::Truck => 16.0,
            Unit::SupplyCrate => 100000.0,
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
