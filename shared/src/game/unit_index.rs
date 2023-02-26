use crate::facing_direction::FacingDirection;
use crate::game::UnitModel;
use crate::located::Located;
use crate::unit::place::UnitPlace;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod by_location;
pub mod by_player;
pub mod by_transport;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Indices {
    pub by_id: HashMap<UnitId, UnitModel>,
    pub by_location: by_location::Index,
    pub by_player: by_player::Index,
    pub by_transport: by_transport::Index,
}

impl Indices {
    pub fn position_of_unit_or_transport(
        &self,
        unit_id: &UnitId,
    ) -> Result<Located<FacingDirection>, String> {
        match self.by_id.get(unit_id) {
            None => Err("unit not found when getting units or transports location".to_string()),
            Some(unit_model) => Ok(match &unit_model.place {
                UnitPlace::OnMap(loc) => loc.clone(),
                UnitPlace::InUnit(transport_id) => {
                    self.position_of_unit_or_transport(transport_id)?
                }
            }),
        }
    }
}
