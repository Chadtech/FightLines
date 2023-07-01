use crate::facing_direction::FacingDirection;
use crate::located::Located;
use crate::unit;
use crate::unit::Place;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod by_id;
pub mod by_location;
pub mod by_player;
pub mod by_transport;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Indexes {
    pub by_id: by_id::Index,
    pub by_location: by_location::Index,
    pub by_player: by_player::Index,
    pub by_transport: by_transport::Index,
}

pub struct ConsumeBaselineSupplies {
    pub perished: bool,
}

impl Indexes {
    pub fn make(units: Vec<(UnitId, unit::Model)>) -> Indexes {
        let mut unit_hashmap: HashMap<UnitId, unit::Model> = HashMap::new();

        for (unit_id, unit) in units {
            unit_hashmap.insert(unit_id, unit);
        }

        let units_by_id = by_id::Index::from_hash_map(unit_hashmap);

        let by_location_index = by_location::make(&units_by_id);
        let by_player_index = by_player::make(&units_by_id);
        let by_transport_index = by_transport::make(&units_by_id);

        Indexes {
            by_id: units_by_id,
            by_location: by_location_index,
            by_player: by_player_index,
            by_transport: by_transport_index,
        }
    }

    pub fn perish(&mut self, unit_id: &UnitId) -> Result<(), String> {
        self.by_id.delete(unit_id);

        if self.by_transport.contains(unit_id) {
            let facing_dir_loc = match self.position_of_unit_or_transport(unit_id) {
                Ok(l) => l,
                Err(msg) => {
                    return Err(msg);
                }
            };

            for (_, cargo_model) in self.by_transport.get_mut(unit_id).unwrap() {
                cargo_model.place = Place::OnMap(facing_dir_loc.clone());
            }
        }

        Ok(())
    }

    pub fn consume_base_supplies(
        &mut self,
        unit_id: &UnitId,
        cost: i16,
    ) -> Result<ConsumeBaselineSupplies, String> {
        match self.by_id.get_mut(unit_id) {
            None => Err("could not find unit for consuming base supplies".to_string()),
            Some(unit_model) => {
                let new_supplies = unit_model.supplies - cost;

                let perished = new_supplies <= 0;

                if perished {
                    self.perish(unit_id)?
                } else {
                    unit_model.supplies = new_supplies;
                }

                Ok(ConsumeBaselineSupplies { perished })
            }
        }
    }

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
