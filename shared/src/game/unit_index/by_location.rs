use crate::facing_direction::FacingDirection;
use crate::game::UnitModel;
use crate::located::Located;
use crate::unit::place::UnitPlace;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Index(HashMap<Located<()>, Vec<(UnitId, FacingDirection, UnitModel)>>);

impl Index {
    pub fn get(&self, loc: &Located<()>) -> Option<&Vec<(UnitId, FacingDirection, UnitModel)>> {
        self.0.get(loc)
    }

    pub fn iter(&self) -> Iter<'_, Located<()>, Vec<(UnitId, FacingDirection, UnitModel)>> {
        self.0.iter()
    }
}

pub fn make(units: &HashMap<UnitId, UnitModel>) -> Index {
    let mut ret = HashMap::new();

    for (unit_id, unit) in units.iter() {
        if let UnitPlace::OnMap(loc_facing_dir) = unit.place.clone() {
            let key = Located {
                x: loc_facing_dir.x,
                y: loc_facing_dir.y,
                value: (),
            };

            let val = || (unit_id.clone(), loc_facing_dir.value, unit.clone());

            let entry = ret.entry(key).or_insert_with(Vec::new);

            entry.push(val());
        }
    }

    Index(ret)
}
