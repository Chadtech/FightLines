use crate::game::unit_index;
use crate::located;
use crate::located::Located;
use crate::map::Map;
use crate::unit::UnitId;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Budget {
    mobility: f32,
    supply: i16,
}

pub fn get_units_mobility(
    map: &Map,
    unit_id: &UnitId,
    unit_indexes: &unit_index::Indexes,
) -> Result<HashSet<Located<()>>, String> {
    match unit_indexes.by_id.get(unit_id) {
        None => Err("unit not found when getting units mobility".to_string()),
        Some(unit_model) => {
            let mut mobility = HashSet::new();

            let loc = unit_indexes.position_of_unit_or_transport(unit_id)?;

            let mut search: HashMap<Located<()>, Budget> = HashMap::new();

            if !unit_model.unit.is_supply_crate() {
                search.insert(
                    located::unit(loc.x, loc.y),
                    Budget {
                        mobility: unit_model.unit.mobility_budget(),
                        supply: unit_model.supplies,
                    },
                );
            }

            while !search.is_empty() {
                for (search_loc, budget) in search.clone().into_iter() {
                    mobility.insert(search_loc.clone());
                    search.remove(&search_loc);

                    let mobility_budget = budget.mobility;
                    let supply_budget = budget.supply;

                    let x = search_loc.x;
                    let y = search_loc.y;

                    // north
                    if y > 0 {
                        let north_loc = located::unit(x, y - 1);
                        let north_tile = map.get_tile(&north_loc);

                        let mobility_cost = north_tile.mobility_cost(&unit_model.unit);
                        let supply_budget_cost = north_tile.travel_supply_cost(&unit_model.unit);

                        let mobility_budget_at_tile = mobility_budget - mobility_cost;
                        let supply_budget_at_tile = supply_budget - supply_budget_cost;

                        if mobility_budget_at_tile > 0.0 && supply_budget_at_tile > 0 {
                            search
                                .entry(north_loc)
                                .and_modify(|existing_budget| {
                                    if mobility_budget_at_tile > existing_budget.mobility {
                                        existing_budget.mobility = mobility_budget_at_tile;
                                    }

                                    if supply_budget_at_tile > existing_budget.supply {
                                        existing_budget.supply = supply_budget_at_tile;
                                    }
                                })
                                .or_insert(Budget {
                                    mobility: mobility_budget_at_tile,
                                    supply: supply_budget_at_tile,
                                });
                        }
                    }

                    // west
                    if x > 0 {
                        let west_loc = located::unit(x - 1, y);
                        let west_tile = map.get_tile(&west_loc);

                        let mobility_cost = west_tile.mobility_cost(&unit_model.unit);
                        let supply_budget_cost = west_tile.travel_supply_cost(&unit_model.unit);

                        let mobility_budget_at_tile = mobility_budget - mobility_cost;
                        let supply_budget_at_tile = supply_budget - supply_budget_cost;

                        if mobility_budget_at_tile > 0.0 && supply_budget_at_tile > 0 {
                            search
                                .entry(west_loc)
                                .and_modify(|existing_budget| {
                                    if mobility_budget_at_tile > existing_budget.mobility {
                                        existing_budget.mobility = mobility_budget_at_tile;
                                    }

                                    if supply_budget_at_tile > existing_budget.supply {
                                        existing_budget.supply = supply_budget_at_tile;
                                    }
                                })
                                .or_insert(Budget {
                                    mobility: mobility_budget_at_tile,
                                    supply: supply_budget_at_tile,
                                });
                        }
                    }

                    // south
                    {
                        let south_loc = located::unit(x, y + 1);
                        let south_tile = map.get_tile(&south_loc);

                        let mobility_cost = south_tile.mobility_cost(&unit_model.unit);
                        let supply_budget_cost = south_tile.travel_supply_cost(&unit_model.unit);

                        let mobility_budget_at_tile = mobility_budget - mobility_cost;
                        let supply_budget_at_tile = supply_budget - supply_budget_cost;

                        if mobility_budget_at_tile > 0.0 && supply_budget_at_tile > 0 {
                            search
                                .entry(south_loc)
                                .and_modify(|existing_budget| {
                                    if mobility_budget_at_tile > existing_budget.mobility {
                                        existing_budget.mobility = mobility_budget_at_tile;
                                    }

                                    if supply_budget_at_tile > existing_budget.supply {
                                        existing_budget.supply = supply_budget_at_tile;
                                    }
                                })
                                .or_insert(Budget {
                                    mobility: mobility_budget_at_tile,
                                    supply: supply_budget_at_tile,
                                });
                        }
                    }

                    // east
                    {
                        let east_loc = located::unit(x + 1, y);
                        let east_tile = map.get_tile(&east_loc);

                        let mobility_cost = east_tile.mobility_cost(&unit_model.unit);
                        let supply_budget_cost = east_tile.travel_supply_cost(&unit_model.unit);

                        let mobility_budget_at_tile = mobility_budget - mobility_cost;
                        let supply_budget_at_tile = supply_budget - supply_budget_cost;

                        if mobility_budget_at_tile > 0.0 && supply_budget_at_tile > 0 {
                            search
                                .entry(east_loc)
                                .and_modify(|existing_budget| {
                                    if mobility_budget_at_tile > existing_budget.mobility {
                                        existing_budget.mobility = mobility_budget_at_tile;
                                    }

                                    if supply_budget_at_tile > existing_budget.supply {
                                        existing_budget.supply = supply_budget_at_tile;
                                    }
                                })
                                .or_insert(Budget {
                                    mobility: mobility_budget_at_tile,
                                    supply: supply_budget_at_tile,
                                });
                        }
                    }
                }
            }

            Ok(mobility)
        }
    }
}

#[cfg(test)]
mod test_replenishment {
    use crate::facing_direction::FacingDirection;
    use crate::game::mobility::get_units_mobility;
    use crate::game::unit_index::Indexes;
    use crate::id::Id;
    use crate::located::Located;
    use crate::map::Map;
    use crate::team_color::TeamColor;
    use crate::unit;
    use crate::unit::{Place, Unit, UnitId};
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    #[test]
    fn simple() {
        let player_id = Id::from_string("red".to_string(), true).unwrap();

        let unit_id = UnitId::test("red infantry");

        let infantry = unit::Model::new(
            Unit::Infantry,
            &player_id,
            Place::OnMap(Located {
                x: 8,
                y: 8,
                value: FacingDirection::Right,
            }),
            &TeamColor::Red,
        );

        let indexes = Indexes::make(vec![(unit_id.clone(), infantry)]);

        let got = get_units_mobility(&Map::grass_square(), &unit_id, &indexes).unwrap();

        let wanted_pos: Vec<(u16, u16)> = vec![
            (6, 8),
            //
            (7, 9),
            (7, 7),
            (7, 8),
            //
            (8, 6),
            (8, 7),
            (8, 8),
            (8, 9),
            (8, 10),
            //
            (9, 7),
            (9, 8),
            (9, 9),
            //
            (10, 8),
        ];

        let want = wanted_pos
            .into_iter()
            .map(|tuple| tuple.into())
            .collect::<HashSet<Located<()>>>();

        assert_eq!(got, want);
    }

    #[test]
    fn low_supplies() {
        let player_id = Id::from_string("red".to_string(), true).unwrap();

        let unit_id = UnitId::test("red infantry");

        let mut infantry = unit::Model::new(
            Unit::Infantry,
            &player_id,
            Place::OnMap(Located {
                x: 8,
                y: 8,
                value: FacingDirection::Right,
            }),
            &TeamColor::Red,
        );

        infantry.supplies = 16;

        let indexes = Indexes::make(vec![(unit_id.clone(), infantry)]);

        let got = get_units_mobility(&Map::grass_square(), &unit_id, &indexes).unwrap();

        let wanted_pos: Vec<(u16, u16)> = vec![(7, 8), (8, 7), (8, 8), (8, 9), (9, 8)];

        let want = wanted_pos
            .into_iter()
            .map(|tuple| tuple.into())
            .collect::<HashSet<Located<()>>>();

        assert_eq!(got, want);
    }
}
