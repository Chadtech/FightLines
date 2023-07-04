use crate::game::unit_index;
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

    pub fn get_mut(&mut self, unit_id: &UnitId) -> Option<&mut Vec<(UnitId, unit::Model)>> {
        self.0.get_mut(unit_id)
    }

    pub fn contains(&mut self, unit_id: &UnitId) -> bool {
        self.0.contains_key(unit_id)
    }

    pub fn delete_unit(&mut self, transport_id: &UnitId, cargo_id_to_delete: &UnitId) {
        if let Some(cargo) = self.get_mut(transport_id) {
            *cargo = cargo
                .iter()
                .filter(|(cargo_id, _)| cargo_id != cargo_id_to_delete)
                .cloned()
                .collect::<Vec<(UnitId, unit::Model)>>();
        }
    }
}

pub fn make(units: &unit_index::by_id::Index) -> Index {
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

#[cfg(test)]
mod test_index_by_transport {
    use crate::facing_direction::FacingDirection;
    use crate::game::unit_index::{by_id, by_transport};
    use crate::id::Id;
    use crate::located::Located;
    use crate::team_color::TeamColor;
    use crate::unit;
    use crate::unit::{Place, Unit, UnitId};
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn transport_delete_by_unit_id() {
        let player_id = Id::from_string("player".to_string(), true).unwrap();

        let infantry_id = UnitId::test("infantry");
        let truck_id = UnitId::test("truck");

        let infantry = unit::Model::new(
            Unit::Infantry,
            &player_id.clone(),
            Place::InUnit(truck_id.clone()),
            &TeamColor::Red,
        );

        let truck = unit::Model::new(
            Unit::Truck,
            &player_id.clone(),
            Place::OnMap(Located {
                x: 2,
                y: 2,
                value: FacingDirection::Right,
            }),
            &TeamColor::Red,
        );

        let units = vec![(infantry_id.clone(), infantry), (truck_id.clone(), truck)]
            .into_iter()
            .collect::<HashMap<UnitId, unit::Model>>();

        let by_id_index = by_id::Index::from_hash_map(units);

        let mut by_transport_index = by_transport::make(&by_id_index);

        by_transport_index.delete_unit(&truck_id, &infantry_id);

        let got = by_transport_index.get(&truck_id);

        let want = &vec![];
        let want = Some(want);

        assert_eq!(got, want);
    }
}
