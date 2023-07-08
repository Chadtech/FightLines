use crate::facing_direction::FacingDirection;
use crate::game::unit_index;
use crate::id::Id;
use crate::located::Located;
use crate::path::Path;
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

    pub fn delete_unit(&mut self, loc: &Located<()>, unit_id_to_remove: &UnitId) {
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

    pub fn closest_enemy_units_in_path(
        &self,
        player_id: &Id,
        path: &Path,
    ) -> Option<Located<Vec<(UnitId, unit::Model)>>> {
        let origin = match path.first_pos() {
            None => {
                return None;
            }
            Some(o) => o,
        };

        let mut ret: Option<Located<Vec<(UnitId, unit::Model)>>> = None;

        let player_id = player_id.clone();

        for step in path.to_loc_directions() {
            if let Some(units_at_loc) = self.get(&step.to_unit()) {
                let non_supply_crate_units: Vec<(UnitId, unit::Model)> = units_at_loc
                    .iter()
                    .filter_map(|(unit_id, _, unit_model)| {
                        if unit_model.unit.is_supply_crate() || unit_model.owner == player_id {
                            None
                        } else {
                            Some((unit_id.clone(), unit_model.clone()))
                        }
                    })
                    .collect::<Vec<(UnitId, unit::Model)>>();

                if !non_supply_crate_units.is_empty() {
                    let loc = Located {
                        x: step.x,
                        y: step.y,
                        value: non_supply_crate_units,
                    };
                    match ret.clone() {
                        None => {
                            ret = Some(loc);
                        }
                        Some(existing_loc) => {
                            if origin.distance_from(&existing_loc) > origin.distance_from(&loc) {
                                ret = Some(loc);
                            }
                        }
                    }
                }
            }
        }

        ret
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

#[cfg(test)]
mod test_by_location_index {
    use crate::direction::Direction;
    use crate::facing_direction::FacingDirection;
    use crate::game::unit_index::by_id;
    use crate::game::unit_index::by_location;
    use crate::id::Id;
    use crate::located::Located;
    use crate::path::Path;
    use crate::team_color::TeamColor;
    use crate::unit;
    use crate::unit::{Place, Unit, UnitId};
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn basic_closest_units_in_path() {
        let mut units: HashMap<UnitId, unit::Model> = HashMap::new();

        let red_player_id = Id::test("red");
        let blue_player_id = Id::test("blue");

        let infantry_id = UnitId::test("blue infantry");
        let tank_id = UnitId::test("red tank");

        let infantry = unit::Model::new(
            Unit::Infantry,
            &blue_player_id,
            Place::OnMap(Located {
                x: 4,
                y: 2,
                value: FacingDirection::Left,
            }),
            &TeamColor::Blue,
        );

        units.insert(infantry_id.clone(), infantry.clone());

        let tank_loc = Located {
            x: 2,
            y: 2,
            value: FacingDirection::Right,
        };

        units.insert(
            tank_id,
            unit::Model::new(
                Unit::Tank,
                &red_player_id,
                Place::OnMap(tank_loc.clone()),
                &TeamColor::Red,
            ),
        );

        let by_id = by_id::Index::from_hash_map(units);

        let location_index = by_location::make(&by_id);

        let got = location_index.closest_enemy_units_in_path(
            &red_player_id,
            &Path::from_directions_test_only(
                &tank_loc.to_unit(),
                &vec![
                    Direction::East,
                    Direction::East,
                    Direction::East,
                    Direction::East,
                ],
            ),
        );

        let want = Some(Located {
            x: 4,
            y: 2,
            value: vec![(infantry_id, infantry)],
        });

        assert_eq!(got, want);
    }
}
