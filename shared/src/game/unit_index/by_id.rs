use crate::unit;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Iter, Values};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Index {
    pub units: HashMap<UnitId, unit::Model>,
    deleted: HashMap<UnitId, unit::Deleted>,
}

impl Index {
    pub fn get(&self, unit_id: &UnitId) -> Option<&unit::Model> {
        self.units.get(unit_id)
    }

    pub fn delete(&mut self, unit_id: &UnitId) {
        if let Some(unit_model) = self.get(unit_id) {
            let deleted_unit: unit::Deleted = unit_model.into();

            self.units.remove(unit_id);
            self.deleted.insert(unit_id.clone(), deleted_unit);
        }
    }

    pub fn get_mut(&mut self, unit_id: &UnitId) -> Option<&mut unit::Model> {
        self.units.get_mut(unit_id)
    }

    pub fn iter(&self) -> Iter<'_, UnitId, unit::Model> {
        self.units.iter()
    }

    pub fn from_hash_map(hash_map: HashMap<UnitId, unit::Model>) -> Index {
        Index {
            units: hash_map,
            deleted: HashMap::new(),
        }
    }

    pub fn values(&self) -> Values<'_, UnitId, unit::Model> {
        self.units.values()
    }
}
