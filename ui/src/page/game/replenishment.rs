use shared::game::unit_index::Indexes;
use shared::id::Id;
use shared::unit;
use shared::unit::{Place, UnitId};
use std::cmp;
use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub struct Replenishment {
    replenished_units: Vec<(UnitId, i16)>,
    depleted_supply_crates: Vec<(UnitId, i16)>,
}

impl Replenishment {
    pub fn calculate(
        viewer_id: &Id,
        replenishing_unit_id: &UnitId,
        unit_indexes: &Indexes,
    ) -> Result<Replenishment, String> {
        let replenishing_unit_model = match unit_indexes.by_id.get(replenishing_unit_id) {
            Some(u) => u,
            None => {
                return Err("could not find replenishing unit".to_string());
            }
        };

        let replenishing_unit_pos = match &replenishing_unit_model.place {
            Place::OnMap(loc) => loc.with_value(()),
            Place::InUnit(_) => {
                return Err("replenishing unit not on map".to_string());
            }
        };

        let unit_ids_to_replenish: Vec<UnitId> = match unit_indexes
            .by_location
            .get_replenishable_units(&viewer_id, &replenishing_unit_pos)
        {
            Some(u) => u,
            None => {
                return Err("no units to replenish".to_string());
            }
        };

        let mut supply_crates: Vec<(UnitId, i16)> =
            match unit_indexes.by_transport.get(&replenishing_unit_id) {
                Some(cargo) => {
                    let supply_crates = cargo
                        .iter()
                        .filter_map(|(unit_id, unit_model)| {
                            if unit_model.unit.is_supply_crate() {
                                Some((unit_id.clone(), unit_model.supplies))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if supply_crates.is_empty() {
                        return Err("no supply crates".to_string());
                    } else {
                        supply_crates
                    }
                }
                None => {
                    return Err("could not find cargo".to_string());
                }
            };

        let units_to_replenish = unit_ids_to_replenish
            .iter()
            .filter_map(|unit_id| {
                unit_indexes
                    .by_id
                    .get(unit_id)
                    .map(|unit_model| (unit_id.clone(), unit_model))
            })
            .collect::<Vec<(UnitId, &unit::Model)>>();

        let mut units_in_need_of_supplies = units_to_replenish.len() as i16;
        let mut unit_adjustments: HashMap<UnitId, i16> = HashMap::new();
        let mut supply_crate_depletions: HashMap<UnitId, i16> = HashMap::new();

        let mut crate_index = 0;
        while crate_index < supply_crates.len() && units_in_need_of_supplies > 0 {
            let (crate_id, crate_supplies) = supply_crates.get_mut(crate_index).unwrap();

            let supplies_per_unit = crate_supplies.clone() / units_in_need_of_supplies;

            for (unit_id, unit_model) in units_to_replenish.clone() {
                let adjustment = unit_adjustments.entry(unit_id).or_insert(0);

                let capacity =
                    unit_model.unit.max_supplies() - (unit_model.supplies + adjustment.clone());

                if capacity == 0 {
                    units_in_need_of_supplies -= 1;
                }

                let delta = cmp::min(supplies_per_unit, capacity);

                *adjustment += delta;

                *crate_supplies -= delta;

                *supply_crate_depletions.entry(crate_id.clone()).or_insert(0) += delta;
            }

            if *crate_supplies <= 0 {
                crate_index += 1;
            }
        }

        Ok(Replenishment {
            replenished_units: unit_adjustments.into_iter().collect::<Vec<(UnitId, i16)>>(),
            depleted_supply_crates: supply_crate_depletions
                .into_iter()
                .collect::<Vec<(UnitId, i16)>>(),
        })
    }
}

#[cfg(test)]
mod test_replenishment {
    use crate::page::game::replenishment::Replenishment;
    use pretty_assertions::assert_eq;
    use shared::facing_direction::FacingDirection;
    use shared::game::unit_index::Indexes;
    use shared::id::Id;
    use shared::located::Located;
    use shared::team_color::TeamColor;
    use shared::unit;
    use shared::unit::{Place, Unit, UnitId};

    #[test]
    fn one_unit_one_crate() {
        let red_player_id = Id::from_string("red".to_string(), true).unwrap();

        let red_infantry_id = UnitId::test("red infantry");
        let red_infantry = {
            let mut u = unit::Model::new(
                Unit::Infantry,
                &red_player_id,
                Place::OnMap(Located {
                    x: 1,
                    y: 1,
                    value: FacingDirection::Right,
                }),
                &TeamColor::Red,
            );

            u.supplies = 1;

            u
        };

        let red_truck_id = UnitId::test("red truck");
        let red_truck = unit::Model::new(
            Unit::Truck,
            &red_player_id,
            Place::OnMap(Located {
                x: 1,
                y: 1,
                value: FacingDirection::Right,
            }),
            &TeamColor::Red,
        );

        let supply_crate_id = UnitId::test("supply crate");

        let units = vec![
            (red_infantry_id.clone(), red_infantry),
            (red_truck_id.clone(), red_truck),
            (
                supply_crate_id.clone(),
                unit::Model::new(
                    Unit::SupplyCrate,
                    &red_player_id,
                    Place::InUnit(red_truck_id.clone()),
                    &TeamColor::Red,
                ),
            ),
        ];

        let indexes = Indexes::make(units);

        let got = Replenishment::calculate(&red_player_id, &red_truck_id, &indexes).unwrap();

        let want = Replenishment {
            replenished_units: vec![(red_infantry_id, 1023)],
            depleted_supply_crates: vec![(supply_crate_id, 1023)],
        };

        assert_eq!(got, want)
    }

    #[test]
    fn two_units_one_crate() {
        let red_player_id = Id::from_string("red".to_string(), true).unwrap();

        let red_infantry_1_id = UnitId::test("red infantry 1");
        let red_infantry_2_id = UnitId::test("red infantry 2");
        let red_infantry = {
            let mut u = unit::Model::new(
                Unit::Infantry,
                &red_player_id,
                Place::OnMap(Located {
                    x: 1,
                    y: 1,
                    value: FacingDirection::Right,
                }),
                &TeamColor::Red,
            );

            u.supplies = 1;

            u
        };

        let red_truck_id = UnitId::test("red truck");
        let red_truck = unit::Model::new(
            Unit::Truck,
            &red_player_id,
            Place::OnMap(Located {
                x: 1,
                y: 1,
                value: FacingDirection::Right,
            }),
            &TeamColor::Red,
        );

        let supply_crate_id = UnitId::test("supply crate");

        let units = vec![
            (red_infantry_1_id.clone(), red_infantry.clone()),
            (red_infantry_2_id.clone(), red_infantry),
            (red_truck_id.clone(), red_truck),
            (
                supply_crate_id.clone(),
                unit::Model::new(
                    Unit::SupplyCrate,
                    &red_player_id,
                    Place::InUnit(red_truck_id.clone()),
                    &TeamColor::Red,
                ),
            ),
        ];

        let indexes = Indexes::make(units);

        let got = Replenishment::calculate(&red_player_id, &red_truck_id, &indexes).unwrap();

        let want = Replenishment {
            replenished_units: vec![(red_infantry_1_id, 1023), (red_infantry_2_id, 1023)],
            depleted_supply_crates: vec![(supply_crate_id, 2046)],
        };

        assert_eq!(got, want)
    }
}
