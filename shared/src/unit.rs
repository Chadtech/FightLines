use crate::direction::Direction;
use crate::facing_direction::FacingDirection;
use crate::id::Id;
use crate::located::Located;
use crate::rng::RandGen;
use crate::team_color::TeamColor;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Model {
    pub unit: Unit,
    pub place: Place,
    pub owner: Id,
    pub color: TeamColor,
    pub name: Option<String>,
    pub supplies: i16,
    pub entrenched_against: Option<Direction>,
    pub health: Health,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Health {
    Full,
    High,
    Medium,
    Low,
}

impl Model {
    pub fn supplies_percent(&self) -> f32 {
        (self.supplies as f32) / (self.unit.max_supplies() as f32)
    }

    pub fn new(unit: Unit, owner_id: &Id, place: Place, color: &TeamColor) -> Model {
        Model {
            unit: unit.clone(),
            owner: owner_id.clone(),
            place,
            color: color.clone(),
            name: None,
            supplies: unit.max_supplies(),
            entrenched_against: None,
            health: Health::Full,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Deleted {
    pub unit: Unit,
    pub owner: Id,
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

    pub fn on_map(x: u16, y: u16, facing_direction: FacingDirection) -> Place {
        Place::OnMap(Located {
            x,
            y,
            value: facing_direction,
        })
    }

    pub fn to_map_loc(&self) -> Option<&Located<FacingDirection>> {
        match self {
            Place::OnMap(loc) => Some(loc),
            Place::InUnit(_) => None,
        }
    }

    pub fn in_unit_loc(&self) -> Option<&UnitId> {
        match self {
            Place::OnMap(_) => None,
            Place::InUnit(unit_id) => Some(unit_id),
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

    pub fn test(s: &str) -> UnitId {
        UnitId(Id::from_string(s.to_string(), true).unwrap())
    }
}

impl ToString for UnitId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl PartialOrd<Self> for UnitId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnitId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
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

    pub fn replenishable(&self) -> bool {
        match self {
            Unit::Infantry => true,
            Unit::Tank => true,
            Unit::Truck => true,
            Unit::SupplyCrate => false,
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

    pub fn can_attack(&self) -> bool {
        match self {
            Unit::Infantry => true,
            Unit::Tank => true,
            Unit::Truck => false,
            Unit::SupplyCrate => false,
        }
    }

    pub fn can_pick_up_supply_crates(&self) -> bool {
        match self {
            Unit::Infantry => false,
            Unit::Tank => false,
            Unit::Truck => true,
            Unit::SupplyCrate => false,
        }
    }

    pub fn is_supply_crate(&self) -> bool {
        match self {
            Unit::Infantry => false,
            Unit::Tank => false,
            Unit::Truck => false,
            Unit::SupplyCrate => true,
        }
    }

    pub fn carrying_capacity(&self) -> usize {
        match self {
            Unit::Infantry => 0,
            Unit::Tank => 0,
            Unit::Truck => 1,
            Unit::SupplyCrate => 0,
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

    // The cost of moving one tile, without any adjustments
    // for terrain type
    pub fn active_supply_cost(&self) -> Option<f32> {
        match self {
            Unit::Infantry => Some(10.3333),
            Unit::Tank => Some(56.8889),
            Unit::Truck => Some(25.6),
            Unit::SupplyCrate => None,
        }
    }

    // The cost of just existing for one turn, whether
    // the unit did anything or not
    pub fn baseline_supply_cost(&self) -> Option<f32> {
        match self {
            Unit::Infantry => Some(21.3333),
            Unit::Tank => Some(64.0),
            Unit::Truck => Some(10.6667),
            Unit::SupplyCrate => None,
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

#[cfg(test)]
mod test_units {
    use crate::unit::Unit;
    use pretty_assertions::assert_eq;

    #[test]
    fn infantry_mobility() {
        let want = Some(21.3333);
        assert_eq!(want, Unit::Infantry.baseline_supply_cost());
    }
}
