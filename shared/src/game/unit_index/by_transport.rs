use crate::game::UnitModel;
use crate::unit::place::UnitPlace;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Index(HashMap<UnitId, Vec<(UnitId, UnitModel)>>);

impl Index {
    pub fn get(&self, unit_id: &UnitId) -> Option<&Vec<(UnitId, UnitModel)>> {
        self.0.get(unit_id)
    }
}

pub fn make(units: &HashMap<UnitId, UnitModel>) -> Index {
    let mut ret = HashMap::new();

    for (unit_id, unit) in units.iter() {
        if let UnitPlace::InUnit(transport_id) = unit.place.clone() {
            let val = || (unit_id.clone(), unit.clone());

            let entry = ret.entry(transport_id).or_insert_with(Vec::new);

            entry.push(val());
        }
    }

    Index(ret)
}
