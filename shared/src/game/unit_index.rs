use crate::facing_direction::FacingDirection;
use crate::located::Located;
use crate::unit::Place;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};

pub mod by_id;
pub mod by_location;
pub mod by_player;
pub mod by_transport;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Indices {
    pub by_id: by_id::Index,
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
                Place::OnMap(loc) => loc.clone(),
                Place::InUnit(transport_id) => self.position_of_unit_or_transport(transport_id)?,
            }),
        }
    }
}
