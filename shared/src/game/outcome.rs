use crate::facing_direction::FacingDirection;
use crate::game::action::Action;
use crate::id::Id;
use crate::located::Located;
use crate::path::Path;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Traveled {
        unit_id: UnitId,
        path: Path,
    },
    Placed {
        cargo_unit_loc: Located<(FacingDirection, UnitId)>,
        transport_id: UnitId,
    },
    LoadedInto {
        unit_id: UnitId,
        loaded_into: UnitId,
        path: Path,
    },
    PickUp {
        unit_id: UnitId,
        cargo_id: UnitId,
        path: Path,
    },
    NamedUnit {
        unit_id: UnitId,
        name: String,
    },
    Perished {
        unit_id: UnitId,
    },
    ConsumedSupplies {
        unit_id: UnitId,
        supplies: i16,
    },
}

impl Outcome {
    pub fn from_actions(player_moves: &mut Vec<(Id, Vec<Action>)>) -> Vec<Outcome> {
        let mut outcomes = Vec::new();

        let mut player_index = 0;
        let mut cont = true;

        while cont {
            if let Some((_, actions)) = player_moves.get_mut(player_index) {
                if let Some(first) = actions.first() {
                    let mut new_outcomes = outcomes_from_action(first);
                    outcomes.append(&mut new_outcomes);

                    actions.remove(0);
                }
            }

            player_index = (player_index + 1) % player_moves.len();

            cont = !player_moves.iter().all(|(_, m)| m.is_empty());
        }

        outcomes
    }
}

fn outcomes_from_action(action: &Action) -> Vec<Outcome> {
    match action {
        Action::Traveled { unit_id, path, .. } => {
            vec![Outcome::Traveled {
                unit_id: unit_id.clone(),
                path: path.clone(),
            }]
        }
        Action::LoadInto {
            unit_id,
            load_into,
            path,
        } => vec![Outcome::LoadedInto {
            unit_id: unit_id.clone(),
            loaded_into: load_into.clone(),
            path: path.clone(),
        }],
        Action::Batch(actions) => actions
            .iter()
            .map(outcomes_from_action)
            .collect::<Vec<Vec<Outcome>>>()
            .concat(),
        Action::PickedUp {
            unit_id,
            cargo_id,
            path,
        } => {
            vec![Outcome::PickUp {
                unit_id: unit_id.clone(),
                cargo_id: cargo_id.clone(),
                path: path.clone(),
            }]
        }
        Action::DropOff {
            cargo_unit_loc: loc,
            transport_id,
        } => {
            vec![Outcome::Placed {
                cargo_unit_loc: loc.clone(),
                transport_id: transport_id.clone(),
            }]
        }
    }
}
