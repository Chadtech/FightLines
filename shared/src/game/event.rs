use crate::direction::Direction;
use crate::game;
use crate::game::action::Action;
use crate::game::unit_index;
use crate::id::Id;
use crate::located::Located;
use crate::path::Path;
use crate::rng::{RandGen, RandSeed};
use crate::unit::UnitId;

pub enum Event {
    ConsumedBaselineSupplies { unit_id: UnitId, cost: f32 }, // Travelled {
                                                             //     unit_id: UnitId,
                                                             //     path: Path,
                                                             // },
                                                             // Loaded {
                                                             //     cargo_id: UnitId,
                                                             //     transport_id: UnitId,
                                                             //     picked_up: bool,
                                                             //     loc: Located<()>,
                                                             // },
                                                             // Unloaded {
                                                             //     cargo_id: UnitId,
                                                             //     transport_id: UnitId,
                                                             //     loc: Located<()>,
                                                             // },
                                                             // Replenished {
                                                             //     unit_id: UnitId,
                                                             //     loc: Located<()>,
                                                             // },
                                                             // Battle {
                                                             //     attackers: Vec<(Direction, UnitId)>,
                                                             //     defenders: Vec<(Direction, UnitId)>,
                                                             // },
}

pub fn process_turn_into_events(
    rand_seed: RandSeed,
    player_moves: &mut Vec<(Id, Vec<Action>)>,
    indexes: unit_index::Indexes,
) -> Vec<Event> {
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

    events
}

fn baseline_supply_events(indexes: unit_index::Indexes) -> Vec<Event> {
    let mut events = vec![];

    for (unit_id, unit_model) in indexes.by_id.iter() {
        if let Some(supply_cost) = unit_model.unit.baseline_supply_cost() {
            events.push(Event::ConsumedBaselineSupplies {
                unit_id: unit_id.clone(),
                cost: supply_cost.clone(),
            });
        }
    }

    events
}
