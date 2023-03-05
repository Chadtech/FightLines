use crate::unit;
use crate::unit::{Place, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Index(HashMap<UnitId, Vec<(UnitId, unit::Model)>>);

impl Index {
    pub fn get(&self, unit_id: &UnitId) -> Option<&Vec<(UnitId, unit::Model)>> {
        self.0.get(unit_id)
    }
}

pub fn make(units: &HashMap<UnitId, unit::Model>) -> Index {
    let mut ret = HashMap::new();

    for (unit_id, unit) in units.iter() {
        if let Place::InUnit(transport_id) = unit.place.clone() {
            let val = || (unit_id.clone(), unit.clone());

            let entry = ret.entry(transport_id).or_insert_with(Vec::new);

            entry.push(val());
        }
    }

    Index(ret)
}
