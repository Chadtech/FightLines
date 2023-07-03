use crate::facing_direction::FacingDirection;
use crate::located::Located;
use crate::map::Map;
use crate::path::Path;
use crate::unit;
use crate::unit::Place;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::cmp;
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

pub struct CargoAndTransportIds<'a> {
    pub cargo_id: &'a UnitId,
    pub transport_id: &'a UnitId,
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

    pub fn pick_up(
        &mut self,
        units: CargoAndTransportIds<'_>,
        path: &Path,
        map: &Map,
    ) -> Result<(), String> {
        let cargo_unit_model = match self.by_id.get_mut(units.cargo_id) {
            Some(c) => c,
            None => {
                return Err("could not find cargo unit during pick up".to_string());
            }
        };

        let cargo_last_pos = match cargo_unit_model.place.clone() {
            Place::OnMap(loc) => loc.to_unit(),
            Place::InUnit(_) => {
                return Err("cargo not on map during pick up".to_string());
            }
        };

        cargo_unit_model.place = Place::InUnit(units.transport_id.clone());

        self.by_location
            .delete_unit(&cargo_last_pos, units.cargo_id);

        self.travel_unit(units.transport_id, path, map)?;

        Ok(())
    }

    pub fn replenish(&mut self, unit_id: &UnitId, amount: i16) -> Result<(), String> {
        let unit_model = match self.by_id.get_mut(unit_id) {
            None => return Err("could not get unit to replenish".to_string()),
            Some(u) => u,
        };

        unit_model.supplies = cmp::min(amount, unit_model.unit.max_supplies());

        Ok(())
    }

    pub fn deplete_supply_crate(&mut self, unit_id: &UnitId, amount: i16) -> Result<(), String> {
        let unit_model = match self.by_id.get_mut(unit_id) {
            None => {
                return Err("could not get unit to deplete supplies".to_string());
            }
            Some(u) => u,
        };

        let place = unit_model.place.clone();

        if !unit_model.unit.is_supply_crate() {
            return Err("trying to deplete supplies of non-supply crate unit".to_string());
        }

        let new_supply_level = unit_model.supplies - amount;

        if new_supply_level <= 0 {
            self.by_id.delete(unit_id);

            self.delete_by_place(unit_id, &place);
        }

        Ok(())
    }

    pub fn delete_by_place(&mut self, unit_id: &UnitId, place: &Place) {
        match place {
            Place::OnMap(loc) => {
                self.by_location.delete_unit(&loc.to_unit(), unit_id);
            }
            Place::InUnit(transport_id) => {
                self.by_transport.delete_unit(transport_id, unit_id);
            }
        }
    }

    pub fn unload(&mut self, cargo_id: &UnitId) -> Result<(), String> {
        let facing_dir = self.position_of_unit_or_transport(cargo_id)?;

        let cargo = match self.by_id.get_mut(cargo_id) {
            Some(u) => u,
            None => {
                return Err("could not find unit to unload".to_string());
            }
        };

        cargo.place = Place::OnMap(facing_dir.clone());

        self.by_location.insert(
            &facing_dir.to_unit(),
            cargo_id.clone(),
            facing_dir.value,
            cargo.clone(),
        );

        Ok(())
    }

    pub fn load_into(
        &mut self,
        units: CargoAndTransportIds<'_>,
        path: &Path,
        map: &Map,
    ) -> Result<(), String> {
        match self.by_id.get_mut(units.cargo_id) {
            None => Err("could not find cargo unit when loading into".to_string()),
            Some(cargo_model) => {
                let current_loc = match cargo_model.place.clone() {
                    Place::OnMap(loc) => loc,
                    Place::InUnit(_) => return Err("cargo unit was not on the map".to_string()),
                };

                cargo_model.supplies -= path.supply_cost(map, &cargo_model.unit);
                cargo_model.place = Place::InUnit(units.transport_id.clone());

                self.by_location
                    .delete_unit(&current_loc.to_unit(), units.cargo_id);

                Ok(())
            }
        }
    }

    pub fn travel_unit(&mut self, unit_id: &UnitId, path: &Path, map: &Map) -> Result<(), String> {
        let loc = match path.last_pos() {
            None => {
                return Ok(());
            }
            Some(loc) => loc,
        };

        let current_loc = match self.position_of_unit_or_transport(unit_id) {
            Ok(facing_dir_loc) => facing_dir_loc,
            Err(msg) => {
                return Err(msg);
            }
        };

        match self.by_id.get_mut(unit_id) {
            Some(unit_model) => {
                unit_model.supplies -= path.supply_cost(map, &unit_model.unit);

                let prev_place = unit_model.place.clone();

                let new_facing_dir = FacingDirection::from_directions(path.clone().to_directions())
                    .unwrap_or_else(|| current_loc.value.clone());

                unit_model.place = Place::OnMap(loc.with_value(new_facing_dir.clone()));

                self.by_location.insert(
                    &loc.to_unit(),
                    unit_id.clone(),
                    new_facing_dir,
                    unit_model.clone(),
                );

                self.delete_by_place(unit_id, &prev_place);

                Ok(())
            }
            None => Err("could not get unit when trying to travel it".to_string()),
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

    pub fn get_units_by_location(
        &self,
        key: &Located<()>,
    ) -> Option<&Vec<(UnitId, FacingDirection, unit::Model)>> {
        self.by_location.get(key)
    }
}
