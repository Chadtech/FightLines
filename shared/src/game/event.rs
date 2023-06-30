use crate::game;
use crate::game::action::Action;
use crate::game::replenishment::Replenishment;
use crate::game::unit_index;
use crate::id::Id;
use crate::located::Located;
use crate::path::Path;
use crate::rng::{RandGen, RandSeed};
use crate::unit::UnitId;

pub enum Event {
    ConsumedBaselineSupplies {
        unit_id: UnitId,
        cost: f32,
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
    Replenished {
        unit_id: UnitId,
        amount: i16,
    },
    // Battle {},
}

pub fn process_turn_into_events(
    rand_seed: RandSeed,
    player_moves: &mut Vec<(Id, Vec<Action>)>,
    indexes: &mut unit_index::Indexes,
) -> Result<Vec<Event>, String> {
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
                    ordered_actions.remove(0);
                    if actions.is_empty() {
                        player_moves.remove(player_index);
                    }
                }
            }

            player_index = (player_index + 1) % player_moves.len();
        }
    }

    let mut events = baseline_supply_events(indexes);

    let mut i = 0;

    while i < ordered_actions.len() {
        let action = ordered_actions.get(i).unwrap();

        let _ = process_action(action, &mut ordered_actions, &mut events);

        i += 1;
    }

    // while let Some(event) = events.first() {
    //     match event {
    //         Event::ConsumedBaselineSupplies { unit_id, cost } => {
    //             if let Some(unit_model) = indexes.by_id.get_mut(unit_id) {
    //                 let new_supplies = unit_model.supplies - (cost.ceil() as i16);
    //
    //                 if new_supplies <= 0 {
    //                     let _ = indexes.perish(unit_id);
    //                 } else {
    //                     unit_model.supplies = new_supplies;
    //                 }
    //             }
    //         }
    //         Event::Travelled { .. } => {}
    //     }
    //
    //     events.remove(0);
    //
    //     if let Some((action, rest)) = ordered_actions.split_first() {
    //         match action {
    //             Action::Travel { path, unit_id, .. } => {
    //                 events.push(Event::Travelled {
    //                     unit_id: unit_id.clone(),
    //                     path: path.clone(),
    //                 });
    //             }
    //             Action::LoadInto { .. } => {}
    //             Action::PickUp { .. } => {}
    //             Action::DropOff { .. } => {}
    //             Action::Replenish { .. } => {}
    //             Action::Attack { .. } => {}
    //             Action::Batch(_) => {}
    //         }
    //     }
    // }

    Ok(events)
}

fn process_action(
    action: &Action,
    indexes: &unit_index::Indexes,
    actions: &mut Vec<Action>,
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
            ..
        } => {
            let unit_model = match indexes.by_id.get(replenishing_unit_id) {
                Some(u) => u,
                None => {
                    return Err("could not find replenishing unit".to_string());
                }
            };

            // let replenishment_result = Replenishment::calculate()
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
                cost: supply_cost,
            });
        }
    }

    events
}
