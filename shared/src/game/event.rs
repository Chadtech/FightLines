use crate::game;
use crate::game::action::Action;
use crate::game::replenishment::Replenishment;
use crate::game::unit_index;
use crate::id::Id;
use crate::located::Located;
use crate::map::Map;
use crate::path::Path;
use crate::rng::{RandGen, RandSeed};
use crate::unit::UnitId;

#[derive(PartialEq, Eq, Debug)]
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
        picked_up: bool,
        loc: Located<()>,
    },
    Unloaded {
        cargo_id: UnitId,
        transport_id: UnitId,
        loc: Located<()>,
    },
    ReplenishedUnits {
        path: Path,
        unit_id: UnitId,
    },
    WasReplenished {
        unit_id: UnitId,
        amount: i16,
    },
    DepletedCrate {
        unit_id: UnitId,
        amount: i16,
    }, // Battle {},
}

pub fn process_turn_into_events(
    rand_seed: RandSeed,
    player_moves: &mut Vec<(Id, Vec<Action>)>,
    indexes: &mut unit_index::Indexes,
    map: &Map,
) -> Result<Vec<String>, String> {
    let mut rng = RandGen::from_seed(rand_seed);

    // These actions might come in orders that don't make sense,
    // such as a unit moving away from a transport before it has
    // been unloaded from the transport.
    for (_, moves) in &mut *player_moves {
        game::action::order(&mut rng, moves);
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
                    if actions.is_empty() {
                        player_moves.remove(player_index);
                    }
                }
            }

            if !player_moves.is_empty() {
                player_index = (player_index + 1) % player_moves.len();
            }
        }
    }

    let mut events = baseline_supply_events(indexes);
    let mut errors: Vec<String> = vec![];

    while let Some(event) = events.first() {
        let mut event_error = |err: String| {
            let mut err_msg = "event error : ".to_string();

            err_msg.push_str(err.as_str());

            errors.push(err_msg);
        };

        match event {
            Event::ConsumedBaselineSupplies { unit_id, cost } => {
                match indexes.consume_base_supplies(unit_id, cost.clone()) {
                    Ok(consume_baseline_supplies) => {
                        if consume_baseline_supplies.perished {
                            delete_actions_for_deleted_unit(unit_id.clone(), &mut ordered_actions);
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
            Event::Loaded { .. } => {}
            Event::Unloaded { .. } => {}
            Event::ReplenishedUnits { .. } => {}
            Event::WasReplenished { .. } => {}
            Event::DepletedCrate { .. } => {}
        }

        events.remove(0);

        if let Some(action) = ordered_actions.first() {
            if let Err(err) = process_action(&action, indexes, &mut events) {
                let mut err_msg = "process action error : ".to_string();

                err_msg.push_str(err.as_str());

                errors.push(err_msg);
            };

            ordered_actions.remove(0);
        }
    }

    Ok(errors)
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
    action: &Action,
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
            if let Some(pos) = path.last_pos() {
                events.push(Event::Loaded {
                    cargo_id: unit_id.clone(),
                    transport_id: load_into.clone(),
                    picked_up: false,
                    loc: pos,
                });
            }
        }
        Action::PickUp {
            unit_id,
            cargo_id,
            path,
        } => {
            if let Some(pos) = path.last_pos() {
                events.push(Event::Loaded {
                    cargo_id: cargo_id.clone(),
                    transport_id: unit_id.clone(),
                    picked_up: true,
                    loc: pos,
                });
            }
        }
        Action::DropOff {
            cargo_unit_loc,
            transport_id,
        } => events.push(Event::Unloaded {
            cargo_id: cargo_unit_loc.value.1.clone(),
            transport_id: transport_id.clone(),
            loc: cargo_unit_loc.to_unit(),
        }),
        Action::Replenish {
            replenishing_unit_id,
            path,
            ..
        } => {
            let unit_model = match indexes.by_id.get(replenishing_unit_id) {
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
                replenishing_unit_id,
                replenishment_pos,
                indexes,
            )?;

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
                path: path.clone(),
            });
        }
        Action::Attack(_) => {}
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
    use crate::game::event::process_turn_into_events;
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

        let got_errors =
            process_turn_into_events(rand_seed, &mut actions, &mut indexes, &map).unwrap();

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
}
