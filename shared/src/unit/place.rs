use crate::facing_direction::FacingDirection;
use crate::located::Located;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum UnitPlace {
    OnMap(Located<FacingDirection>),
    InUnit(UnitId),
}

impl UnitPlace {
    pub fn is_on_map(&self) -> bool {
        match self {
            UnitPlace::OnMap(_) => true,
            UnitPlace::InUnit(_) => false,
        }
    }
}
