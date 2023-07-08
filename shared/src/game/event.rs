use crate::facing_direction::FacingDirection;
use crate::game::action::Action;
use crate::game::replenishment::Replenishment;
use crate::game::{action, unit_index};
use crate::id::Id;
use crate::located::Located;
use crate::map::Map;
use crate::path::Path;
use crate::rng::{RandGen, RandSeed};
use crate::unit::{Place, UnitId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Event {
    ConsumedBaselineSupplies {
        unit_id: UnitId,
        cost: i16,
    },
    Travelled {
        unit_id: UnitId,
        path: Path,
    },
    Loaded {
        cargo_id: UnitId,
        transport_id: UnitId,
        path: Path,
    },
    PickedUp {
        cargo_id: UnitId,
        transport_id: UnitId,
        path: Path,
    },
    DroppedOff {
        cargo_id: UnitId,
        transport_id: UnitId,
        cargo_loc: Located<FacingDirection>,
    },
    ReplenishedUnits {
        path: Path,
        unit_id: UnitId,
        replenished_units: Vec<UnitId>,
    },
    WasReplenished {
        unit_id: UnitId,
        amount: i16,
    },
    DepletedCrate {
        unit_id: UnitId,
        amount: i16,
    },
    Perished {
        unit_id: UnitId,
    }, // Battle {},
}

pub struct ProcessedTurn {
    pub events: Vec<Event>,
    pub errors: Vec<String>,
}

pub fn process_turn(
    rand_seed: RandSeed,
    player_moves: &mut Vec<(Id, Vec<Action>)>,
    indexes: &mut unit_index::Indexes,
    map: &Map,
) -> ProcessedTurn {
    let mut rng = RandGen::from_seed(rand_seed);

    // These actions might come in orders that don't make sense,
    // such as a unit moving away from a transport before it has
    // been unloaded from the transport.
    for (_, moves) in &mut *player_moves {
        action::order(&mut rng, moves);
    }

    let mut ordered_actions = Vec::new();

    // The aggregation of all players actions should be ordered
    // such that player moves are evenly mixed together and the
    // first moves are not just from one player
    {
        let mut player_index = rng.gen::<usize>(0, player_moves.len());

        while !player_moves.is_empty() {
            if let Some((_, actions)) = player_moves.get_mut(player_index) {
                if let Some(first) = actions.first() {
                    ordered_actions.push(first.clone());
                    actions.remove(0);
                }

                if actions.is_empty() {
                    player_moves.remove(player_index);
                }
            }

            if !player_moves.is_empty() {
                player_index = (player_index + 1) % player_moves.len();
            }
        }
    }

    action::unbatch(&mut ordered_actions);

    let mut events = baseline_supply_events(indexes);
    let mut errors: Vec<String> = vec![];
    let mut event_index = 0;
    // while let Some(event) = events.first() {
    while event_index < events.len() {
        let event = events.get(event_index).unwrap();

        let mut event_error = |err: String| {
            let mut err_msg = "event error : ".to_string();

            err_msg.push_str(err.as_str());

            errors.push(err_msg);
        };

        match event {
            Event::ConsumedBaselineSupplies { unit_id, cost } => {
                match indexes.consume_base_supplies(unit_id, *cost) {
                    Ok(consume_baseline_supplies) => {
                        if consume_baseline_supplies.perished {
                            delete_actions_for_deleted_unit(unit_id.clone(), &mut ordered_actions);

                            events.push(Event::Perished {
                                unit_id: unit_id.clone(),
                            });
                        }
                    }
                    Err(err) => {
                        event_error(err);
                    }
                }
            }
            Event::Travelled { unit_id, path } => {
                if let Err(err) = indexes.travel_unit(unit_id, path, map) {
                    event_error(err);
                }
            }
            Event::Loaded {
                cargo_id,
                transport_id,
                path,
            } => {
                if let Err(err) = indexes.load_into(
                    unit_index::CargoAndTransportIds {
                        cargo_id,
                        transport_id,
                    },
                    path,
                    map,
                ) {
                    event_error(err)
                }
            }
            Event::PickedUp {
                cargo_id,
                transport_id,
                path,
            } => {
                if let Err(err) = indexes.pick_up(
                    unit_index::CargoAndTransportIds {
                        cargo_id,
                        transport_id,
                    },
                    path,
                    map,
                ) {
                    event_error(err);
                }
            }
            Event::DroppedOff { cargo_id, .. } => {
                if let Err(err) = indexes.unload(cargo_id) {
                    event_error(err);
                }
            }
            Event::ReplenishedUnits { .. } => {
                // This is only used for animation
            }
            Event::WasReplenished { unit_id, amount } => {
                if let Err(err) = indexes.replenish(unit_id, *amount) {
                    event_error(err)
                }
            }
            Event::DepletedCrate { unit_id, amount } => {
                if let Err(err) = indexes.deplete_supply_crate(unit_id, *amount) {
                    event_error(err)
                }
            }
            Event::Perished { .. } => {
                // This is only used for animation
            }
        }

        event_index += 1;

        if let Some(action) = ordered_actions.first() {
            let action = action.clone();
            ordered_actions.remove(0);

            if let Err(err) = process_action(action, &mut ordered_actions, indexes, &mut events) {
                let mut err_msg = "process action error : ".to_string();

                err_msg.push_str(err.as_str());

                errors.push(err_msg);
            };
        }
    }

    ProcessedTurn { errors, events }
}

fn delete_actions_for_deleted_unit(deleted_unit_id: UnitId, actions: &mut Vec<Action>) {
    let mut i = 0;
    while i < actions.len() {
        let action = actions.get(i).unwrap();

        if let Some(replacement_actions) = action.when_unit_deleted(deleted_unit_id.clone()) {
            let replacements_len = replacement_actions.len();
            actions.splice(i..i + 1, replacement_actions);
            i += replacements_len;
        }

        i += 1
    }
}

fn process_action(
    action: Action,
    remaining_actions: &mut Vec<Action>,
    indexes: &unit_index::Indexes,
    events: &mut Vec<Event>,
) -> Result<(), String> {
    match action {
        Action::Travel { path, unit_id, .. } => {
            events.push(Event::Travelled {
                unit_id: unit_id.clone(),
                path: path.clone(),
            });
        }
        Action::LoadInto {
            unit_id,
            load_into,
            path,
        } => {
            events.push(Event::Loaded {
                cargo_id: unit_id.clone(),
                transport_id: load_into.clone(),
                path: path.clone(),
            });
        }
        Action::PickUp {
            unit_id,
            cargo_id,
            path,
        } => {
            events.push(Event::PickedUp {
                cargo_id: cargo_id.clone(),
                transport_id: unit_id.clone(),
                path: path.clone(),
            });
        }
        Action::DropOff { cargo_id } => {
            let unit_model = match indexes.by_id.get(&cargo_id) {
                Some(u) => u,
                None => return Err("could not get unit when making drop off event".to_string()),
            };

            let transport_id = match &unit_model.place {
                Place::OnMap(_) => {
                    return Err("unit that is being dropped off is already on map".to_string())
                }
                Place::InUnit(t) => t,
            };

            let loc = indexes.position_of_unit_or_transport(&cargo_id)?;

            events.push(Event::DroppedOff {
                cargo_id: cargo_id.clone(),
                cargo_loc: loc,
                transport_id: transport_id.clone(),
            })
        }
        Action::Replenish {
            replenishing_unit_id,
            path,
            ..
        } => {
            let unit_model = match indexes.by_id.get(&replenishing_unit_id) {
                None => {
                    return Err("could not finding replenishing unit".to_string());
                }
                Some(u) => u,
            };

            let replenishment_pos = match path.last_pos() {
                Some(p) => p,
                None => {
                    return Err("replenishing unit does not travel".to_string());
                }
            };

            let replenishment = Replenishment::calculate(
                &unit_model.owner,
                &replenishing_unit_id,
                replenishment_pos,
                indexes,
            )?;

            let replenished_unit_ids = replenishment
                .replenished_units
                .iter()
                .map(|(unit_id, _)| unit_id.clone())
                .collect::<Vec<UnitId>>();

            for (unit_id, amount) in replenishment.replenished_units {
                events.push(Event::WasReplenished { unit_id, amount });
            }

            for (crate_id, amount) in replenishment.depleted_supply_crates {
                events.push(Event::DepletedCrate {
                    unit_id: crate_id,
                    amount,
                });
            }

            events.push(Event::ReplenishedUnits {
                unit_id: replenishing_unit_id.clone(),
                replenished_units: replenished_unit_ids,
                path: path.clone(),
            });
        }
        Action::Attack(attack) => {
            if let Some((index_of_closest_path_action, loc_unit_closest_path)) =
                Action::closest_crossing_path(&attack.path, remaining_actions)
            {};
        }
        Action::Batch(_) => {}
    }

    Ok(())
}

fn baseline_supply_events(indexes: &unit_index::Indexes) -> Vec<Event> {
    let mut events = vec![];

    for (unit_id, unit_model) in indexes.by_id.iter() {
        if let Some(supply_cost) = unit_model.unit.baseline_supply_cost() {
            events.push(Event::ConsumedBaselineSupplies {
                unit_id: unit_id.clone(),
                cost: supply_cost.ceil() as i16,
            });
        }
    }

    events
}

#[cfg(test)]
mod test_events {
    use crate::direction::Direction;
    use crate::facing_direction::FacingDirection;
    use crate::game::action::Action;
    use crate::game::event::process_turn;
    use crate::game::unit_index::Indexes;
    use crate::id::Id;
    use crate::located::Located;
    use crate::map::Map;
    use crate::path::Path;
    use crate::rng::RandSeed;
    use crate::team_color::TeamColor;
    use crate::unit::{Place, Unit, UnitId};
    use crate::{located, unit};
    use pretty_assertions::assert_eq;

    #[test]
    fn process_travel() {
        let rand_seed = RandSeed::test();

        let player_1 = Id::from_string("player 1".to_string(), true).unwrap();
        let infantry_id = UnitId::test("infantry");

        let path = Path::from_directions_test_only(&located::unit(2, 2), &vec![Direction::South]);

        let player_1_actions = vec![Action::Travel {
            unit_id: infantry_id.clone(),
            path: path.clone(),
            dismounted_from: None,
        }];

        let mut actions = vec![(player_1.clone(), player_1_actions)];

        let mut indexes = Indexes::make(vec![(
            infantry_id.clone(),
            unit::Model::new(
                Unit::Infantry,
                &player_1,
                Place::OnMap(Located {
                    x: 2,
                    y: 2,
                    value: FacingDirection::Right,
                }),
                &TeamColor::Red,
            ),
        )]);

        let map = Map::grass_square();

        let got_errors = process_turn(rand_seed, &mut actions, &mut indexes, &map).errors;

        let want_errors: Vec<String> = vec![];
        assert_eq!(want_errors, got_errors);

        let got_infantry_loc = indexes
            .by_id
            .get(&infantry_id)
            .unwrap()
            .place
            .to_map_loc()
            .unwrap();

        let want_loc = &Located {
            x: 2,
            y: 3,
            value: FacingDirection::Right,
        };

        assert_eq!(want_loc, got_infantry_loc);

        let got_units_by_loc = indexes.by_location.get(&want_loc.to_unit()).unwrap();

        let mut infantry_after_turn = unit::Model::new(
            Unit::Infantry,
            &player_1,
            Place::OnMap(Located {
                x: 2,
                y: 3,
                value: FacingDirection::Right,
            }),
            &TeamColor::Red,
        );

        infantry_after_turn.supplies = 982;

        let want_units = &vec![(infantry_id, FacingDirection::Right, infantry_after_turn)];

        assert_eq!(want_units, got_units_by_loc);
    }

    #[test]
    fn process_loading() {
        let rand_seed = RandSeed::test();

        let player_1 = Id::from_string("player 1".to_string(), true).unwrap();
        let infantry_id = UnitId::test("infantry");
        let truck_id = UnitId::test("truck");

        let player_1_loads_actions = vec![Action::LoadInto {
            unit_id: infantry_id.clone(),
            load_into: truck_id.clone(),
            path: Path::from_directions_test_only(
                &located::unit(2, 2),
                &vec![Direction::South, Direction::South],
            ),
        }];

        let mut actions = vec![(player_1.clone(), player_1_loads_actions)];

        let mut indexes = Indexes::make(vec![
            (
                infantry_id.clone(),
                unit::Model::new(
                    Unit::Infantry,
                    &player_1,
                    Place::OnMap(Located {
                        x: 2,
                        y: 2,
                        value: FacingDirection::Right,
                    }),
                    &TeamColor::Red,
                ),
            ),
            (
                truck_id.clone(),
                unit::Model::new(
                    Unit::Truck,
                    &player_1,
                    Place::OnMap(Located {
                        x: 2,
                        y: 4,
                        value: FacingDirection::Right,
                    }),
                    &TeamColor::Red,
                ),
            ),
        ]);

        let map = Map::grass_square();

        let got_errors = process_turn(rand_seed.clone(), &mut actions, &mut indexes, &map).errors;

        let want_errors: Vec<String> = vec![];
        assert_eq!(want_errors, got_errors);

        // After the turn, the infantry is located in the truck
        {
            let got_infantry_loc = indexes
                .by_id
                .get(&infantry_id)
                .unwrap()
                .place
                .in_unit_loc()
                .unwrap();

            let want_loc = truck_id.clone();

            assert_eq!(want_loc, got_infantry_loc.clone());
        }

        // After the turn, the infantry is no longer located a 2,2, where it was before
        {
            let got_units_by_loc = indexes.by_location.get(&located::unit(2, 2)).unwrap();

            let want_units: Vec<(UnitId, FacingDirection, unit::Model)> = vec![];

            assert_eq!(&want_units, got_units_by_loc);
        }

        // After the turn, we can get the infantry by id as we expect it
        {
            let mut infantry_after_turn = unit::Model::new(
                Unit::Infantry,
                &player_1,
                Place::InUnit(truck_id.clone()),
                &TeamColor::Red,
            );

            infantry_after_turn.supplies = 972;

            let got_infantry_by_id = indexes.by_id.get(&infantry_id.clone()).unwrap().clone();
            let want_infantry_by_id = infantry_after_turn;

            assert_eq!(want_infantry_by_id, got_infantry_by_id);
        }

        let player_1_unload_actions = vec![
            Action::Travel {
                unit_id: truck_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(2, 4),
                    &vec![
                        Direction::East,
                        Direction::East,
                        Direction::East,
                        Direction::East,
                    ],
                ),
                dismounted_from: None,
            },
            Action::Travel {
                unit_id: infantry_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(6, 4),
                    &vec![Direction::South, Direction::South],
                ),
                dismounted_from: Some(truck_id.clone()),
            },
        ];

        let mut actions = vec![(player_1.clone(), player_1_unload_actions)];

        let got_errors = process_turn(rand_seed, &mut actions, &mut indexes, &map).errors;

        let want_errors: Vec<String> = vec![];

        assert_eq!(want_errors, got_errors);

        // After the second turn, the truck has moved
        {
            let got_truck_loc = indexes
                .by_id
                .get(&truck_id)
                .unwrap()
                .place
                .to_map_loc()
                .unwrap();

            let want_loc = Located {
                x: 6,
                y: 4,
                value: FacingDirection::Right,
            };

            assert_eq!(want_loc, got_truck_loc.clone());
        }

        // After the second turn, the infantry is located south of the truck
        {
            let got_infantry_loc = indexes
                .by_id
                .get(&infantry_id)
                .unwrap()
                .place
                .to_map_loc()
                .unwrap();

            let want_loc = Located {
                x: 6,
                y: 6,
                value: FacingDirection::Right,
            };

            assert_eq!(want_loc, got_infantry_loc.clone());
        }
    }

    #[test]
    fn process_picking_up_crate() {
        let rand_seed = RandSeed::test();

        let player_1 = Id::from_string("player 1".to_string(), true).unwrap();
        let truck_id = UnitId::test("truck");
        let supply_crate_id = UnitId::test("supply crate");

        let player_1_pickup_actions = vec![Action::PickUp {
            unit_id: truck_id.clone(),
            cargo_id: supply_crate_id.clone(),
            path: Path::from_directions_test_only(
                &located::unit(4, 4),
                &vec![Direction::West, Direction::West],
            ),
        }];

        let mut actions = vec![(player_1.clone(), player_1_pickup_actions)];

        let mut indexes = Indexes::make(vec![
            (
                supply_crate_id.clone(),
                unit::Model::new(
                    Unit::SupplyCrate,
                    &player_1,
                    Place::OnMap(Located {
                        x: 2,
                        y: 4,
                        value: FacingDirection::Right,
                    }),
                    &TeamColor::Red,
                ),
            ),
            (
                truck_id.clone(),
                unit::Model::new(
                    Unit::Truck,
                    &player_1,
                    Place::OnMap(Located {
                        x: 4,
                        y: 4,
                        value: FacingDirection::Left,
                    }),
                    &TeamColor::Red,
                ),
            ),
        ]);

        let map = Map::grass_square();

        let got_errors = process_turn(rand_seed.clone(), &mut actions, &mut indexes, &map).errors;

        let want_errors: Vec<String> = vec![];
        assert_eq!(want_errors, got_errors);

        // After turn truck is located where the crate was
        {
            let got_truck_loc = indexes
                .by_id
                .get(&truck_id)
                .unwrap()
                .place
                .to_map_loc()
                .unwrap()
                .to_unit();

            let want_truck_loc = located::unit(2, 4);

            assert_eq!(want_truck_loc, got_truck_loc);

            let got_prev_loc = indexes.by_location.get(&located::unit(4, 4));
            let no_units: Vec<(UnitId, FacingDirection, unit::Model)> = vec![];
            let want_prev_loc: Option<&Vec<(UnitId, FacingDirection, unit::Model)>> =
                Some(&no_units);

            assert_eq!(want_prev_loc, got_prev_loc);

            let got_curr_loc = indexes.by_location.get(&located::unit(2, 4));

            let mut truck_after_turn = unit::Model::new(
                Unit::Truck,
                &player_1,
                Place::OnMap(Located {
                    x: 2,
                    y: 4,
                    value: FacingDirection::Left,
                }),
                &TeamColor::Red,
            );

            truck_after_turn.supplies = 1962;

            let one_unit: Vec<(UnitId, FacingDirection, unit::Model)> =
                vec![(truck_id.clone(), FacingDirection::Left, truck_after_turn)];

            let want_curr_loc: Option<&Vec<(UnitId, FacingDirection, unit::Model)>> =
                Some(&one_unit);

            assert_eq!(want_curr_loc, got_curr_loc);
        }

        // After turn crates location is inside truck
        {
            let got_crate_loc = indexes
                .by_id
                .get(&supply_crate_id)
                .unwrap()
                .place
                .in_unit_loc()
                .unwrap()
                .clone();

            let want_crate_loc = truck_id.clone();

            assert_eq!(want_crate_loc, got_crate_loc);
        }

        let player_1_drop_actions = vec![Action::DropOff {
            cargo_id: supply_crate_id.clone(),
        }];

        let mut actions = vec![(player_1.clone(), player_1_drop_actions)];

        let got_errors = process_turn(rand_seed.clone(), &mut actions, &mut indexes, &map).errors;

        let want_errors: Vec<String> = vec![];
        assert_eq!(want_errors, got_errors);

        // After second turn crates location is on map
        {
            let got_crate_loc = indexes
                .by_id
                .get(&supply_crate_id)
                .unwrap()
                .place
                .to_map_loc()
                .unwrap()
                .clone();

            let want_crate_loc = Located {
                x: 2,
                y: 4,
                value: FacingDirection::Left,
            };

            assert_eq!(want_crate_loc, got_crate_loc);

            let mut crate_after_turn = unit::Model::new(
                Unit::SupplyCrate,
                &player_1,
                Place::OnMap(Located {
                    x: 2,
                    y: 4,
                    value: FacingDirection::Left,
                }),
                &TeamColor::Red,
            );

            crate_after_turn.supplies = 8192;

            let mut truck_after_turn = unit::Model::new(
                Unit::Truck,
                &player_1,
                Place::OnMap(Located {
                    x: 2,
                    y: 4,
                    value: FacingDirection::Left,
                }),
                &TeamColor::Red,
            );

            truck_after_turn.supplies = 1962;

            let two_units: Vec<(UnitId, FacingDirection, unit::Model)> = vec![
                (truck_id.clone(), FacingDirection::Left, truck_after_turn),
                (
                    supply_crate_id.clone(),
                    FacingDirection::Left,
                    crate_after_turn,
                ),
            ];

            let want_curr_loc: Option<&Vec<(UnitId, FacingDirection, unit::Model)>> =
                Some(&two_units);

            let got_curr_loc = indexes.by_location.get(&located::unit(2, 4));

            assert_eq!(want_curr_loc, got_curr_loc);
        }
    }

    #[test]
    fn process_two_player_travel() {
        let rand_seed = RandSeed::test();

        let red_player_id = Id::from_string("red player".to_string(), true).unwrap();
        let red_infantry_id = UnitId::test("red infantry");

        let red_player_actions = vec![Action::Travel {
            unit_id: red_infantry_id.clone(),
            path: Path::from_directions_test_only(&located::unit(2, 2), &vec![Direction::South]),
            dismounted_from: None,
        }];

        let blue_player_id = Id::from_string("blue player".to_string(), true).unwrap();
        let blue_infantry_id = UnitId::test("blue infantry");

        let blue_player_actions = vec![Action::Travel {
            unit_id: blue_infantry_id.clone(),
            path: Path::from_directions_test_only(&located::unit(4, 2), &vec![Direction::South]),
            dismounted_from: None,
        }];

        let mut actions = vec![
            (red_player_id.clone(), red_player_actions),
            (blue_player_id.clone(), blue_player_actions),
        ];

        let mut indexes = Indexes::make(vec![
            (
                red_infantry_id.clone(),
                unit::Model::new(
                    Unit::Infantry,
                    &red_player_id,
                    Place::OnMap(Located {
                        x: 2,
                        y: 2,
                        value: FacingDirection::Right,
                    }),
                    &TeamColor::Red,
                ),
            ),
            (
                blue_infantry_id.clone(),
                unit::Model::new(
                    Unit::Infantry,
                    &blue_player_id,
                    Place::OnMap(Located {
                        x: 4,
                        y: 2,
                        value: FacingDirection::Right,
                    }),
                    &TeamColor::Blue,
                ),
            ),
        ]);

        let map = Map::grass_square();

        let got_errors = process_turn(rand_seed, &mut actions, &mut indexes, &map).errors;

        let want_errors: Vec<String> = vec![];
        assert_eq!(want_errors, got_errors);

        let got_red_infantry_loc = indexes
            .by_id
            .get(&red_infantry_id)
            .unwrap()
            .place
            .to_map_loc()
            .unwrap();

        let want_red_infantry_loc = &Located {
            x: 2,
            y: 3,
            value: FacingDirection::Right,
        };

        assert_eq!(want_red_infantry_loc, got_red_infantry_loc);

        let got_blue_infantry_loc = indexes
            .by_id
            .get(&blue_infantry_id)
            .unwrap()
            .place
            .to_map_loc()
            .unwrap();

        let want_blue_infantry_loc = &Located {
            x: 4,
            y: 3,
            value: FacingDirection::Right,
        };

        assert_eq!(want_blue_infantry_loc, got_blue_infantry_loc);

        let got_units_by_loc = indexes
            .by_location
            .get(&want_red_infantry_loc.to_unit())
            .unwrap();

        let mut infantry_after_turn = unit::Model::new(
            Unit::Infantry,
            &red_player_id,
            Place::OnMap(Located {
                x: 2,
                y: 3,
                value: FacingDirection::Right,
            }),
            &TeamColor::Red,
        );

        infantry_after_turn.supplies = 982;

        let want_units = &vec![(red_infantry_id, FacingDirection::Right, infantry_after_turn)];

        assert_eq!(want_units, got_units_by_loc);
    }
}
