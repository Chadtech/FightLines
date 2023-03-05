use crate::id::Id;
use crate::unit;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Index(HashMap<Id, Vec<(UnitId, unit::Model)>>);

impl Index {
    pub fn get(&self, player_id: &Id) -> Option<&Vec<(UnitId, unit::Model)>> {
        self.0.get(player_id)
    }
}

pub fn make(units: &HashMap<UnitId, unit::Model>) -> Index {
    let mut ret = HashMap::new();

    for (unit_id, unit) in units.iter() {
        let key = unit.owner.clone();

        let val = || (unit_id.clone(), unit.clone());

        let entry = ret.entry(key).or_insert_with(Vec::new);

        entry.push(val());
    }

    Index(ret)
}
