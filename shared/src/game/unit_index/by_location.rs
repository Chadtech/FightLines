use crate::facing_direction::FacingDirection;
use crate::game::unit_index;
use crate::id::Id;
use crate::located::Located;
use crate::unit;
use crate::unit::{Place, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Index(HashMap<Located<()>, Vec<(UnitId, FacingDirection, unit::Model)>>);

impl Index {
    pub fn get(&self, loc: &Located<()>) -> Option<&Vec<(UnitId, FacingDirection, unit::Model)>> {
        self.0.get(loc)
    }

    pub fn iter(&self) -> Iter<'_, Located<()>, Vec<(UnitId, FacingDirection, unit::Model)>> {
        self.0.iter()
    }

    pub fn filter_unit_id(&mut self, loc: &Located<()>, unit_id_to_remove: &UnitId) {
        if let Some(units) = self.0.get(loc) {
            let new_units = units
                .iter()
                .filter_map(|(unit_id, facing_dir, unit_model)| {
                    if unit_id != unit_id_to_remove {
                        Some((unit_id.clone(), facing_dir.clone(), unit_model.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(UnitId, FacingDirection, unit::Model)>>();

            self.0.insert(loc.clone(), new_units);
        }
    }

    pub fn insert(
        &mut self,
        loc: &Located<()>,
        unit_id: UnitId,
        facing_dir: FacingDirection,
        unit_model: unit::Model,
    ) {
        let entry = (unit_id, facing_dir, unit_model);

        let units = self.0.entry(loc.clone()).or_insert_with(Vec::new);

        units.push(entry);
    }

    pub fn get_replenishable_units<'a>(
        &'a self,
        viewer_id: &'a Id,
        loc: &Located<()>,
    ) -> Option<Vec<UnitId>> {
        self.0.get(loc).and_then(|units| {
            let filtered_units = units
                .iter()
                .filter_map(|(unit_id, _, unit_model)| {
                    let has_less_than_max_supplies =
                        unit_model.supplies < unit_model.unit.max_supplies();

                    if unit_model.owner == *viewer_id
                        && unit_model.unit.replenishable()
                        && has_less_than_max_supplies
                    {
                        Some(unit_id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<UnitId>>();

            if filtered_units.is_empty() {
                None
            } else {
                Some(filtered_units)
            }
        })
    }
}

pub fn make(units: &unit_index::by_id::Index) -> Index {
    let mut ret = HashMap::new();

    for (unit_id, unit) in units.iter() {
        if let Place::OnMap(loc_facing_dir) = unit.place.clone() {
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
