use crate::game::unit_index;
use crate::located;
use crate::located::Located;
use crate::map::Map;
use crate::unit::UnitId;
use std::collections::{HashMap, HashSet};

struct Budget {
    distance_budget: f32,
    supply_budget: i16,
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

            let mut search: HashMap<Located<()>, f32> = HashMap::new();

            if !unit_model.unit.is_supply_crate() {
                search.insert(
                    located::unit(loc.x, loc.y),
                    unit_model.unit.mobility_budget(),
                );
            }

            // let add_to_search = |budget_at_tile: f32, loc: Located<()>| {
            //     if budget_at_tile > 0.0 {
            //         search
            //             .entry(loc)
            //             .and_modify(|existing_budget| {
            //                 if budget_at_tile > *existing_budget {
            //                     *existing_budget = budget_at_tile;
            //                 }
            //             })
            //             .or_insert(budget_at_tile);
            //     }
            // };

            while !search.is_empty() {
                for (search_loc, mobility_budget) in search.into_iter() {
                    mobility.insert(search_loc.clone());
                    search.remove(&search_loc);

                    let x = search_loc.x;
                    let y = search_loc.y;

                    if y > 0 {
                        let north_loc = located::unit(x, y - 1);
                        let north_tile = map.get_tile(&north_loc);

                        let cost = north_tile.mobility_cost(&unit_model.unit);

                        let budget_at_tile = mobility_budget - cost;

                        if budget_at_tile > 0.0 {
                            search
                                .entry(north_loc)
                                .and_modify(|existing_budget| {
                                    if budget_at_tile > *existing_budget {
                                        *existing_budget = budget_at_tile;
                                    }
                                })
                                .or_insert(budget_at_tile);
                        }
                    }

                    if x > 0 {
                        let west_loc = located::unit(x - 1, y);
                        let west_tile = map.get_tile(&west_loc);

                        let cost = west_tile.mobility_cost(&unit_model.unit);

                        let budget_at_tile = mobility_budget - cost;

                        if budget_at_tile > 0.0 {
                            search
                                .entry(west_loc)
                                .and_modify(|existing_budget| {
                                    if budget_at_tile > *existing_budget {
                                        *existing_budget = budget_at_tile;
                                    }
                                })
                                .or_insert(budget_at_tile);
                        }
                    }

                    let south_loc = located::unit(x, y + 1);
                    let south_tile = map.get_tile(&south_loc);

                    let cost = south_tile.mobility_cost(&unit_model.unit);

                    let budget_at_tile = mobility_budget - cost;

                    if budget_at_tile > 0.0 {
                        search
                            .entry(south_loc)
                            .and_modify(|existing_budget| {
                                if budget_at_tile > *existing_budget {
                                    *existing_budget = budget_at_tile;
                                }
                            })
                            .or_insert(budget_at_tile);
                    }

                    let east_loc = located::unit(x + 1, y);
                    let east_tile = map.get_tile(&east_loc);

                    let cost = east_tile.mobility_cost(&unit_model.unit);

                    let budget_at_tile = mobility_budget - cost;

                    if budget_at_tile > 0.0 {
                        search
                            .entry(east_loc)
                            .and_modify(|existing_budget| {
                                if budget_at_tile > *existing_budget {
                                    *existing_budget = budget_at_tile;
                                }
                            })
                            .or_insert(budget_at_tile);
                    }
                }
            }

            Ok(mobility)
        }
    }
}
