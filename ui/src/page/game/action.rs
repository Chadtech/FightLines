use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::game;
use shared::path::Path;
use shared::unit::UnitId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    TraveledTo {
        unit_id: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
        dismounted_from: Option<UnitId>,
    },
    LoadInto {
        unit_id: UnitId,
        load_into: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
    PickUp {
        unit_id: UnitId,
        cargo_id: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
    DropOff {
        cargo_id: UnitId,
    },
    Replenish {
        replenishing_unit_id: UnitId,
        units: Vec<(UnitId, i16)>,
        depleted_supply_crates: Vec<(UnitId, i16)>,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
    Attack {
        unit_id: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
}

impl Action {
    pub fn from_game_actions(moves: Vec<game::action::Action>) -> Vec<Action> {
        let mut moves_ret = Vec::new();

        for action in moves {
            match action {
                game::action::Action::Travel {
                    unit_id,
                    path,
                    dismounted_from,
                } => {
                    moves_ret.push(Action::TraveledTo {
                        unit_id: unit_id.clone(),
                        path: path.clone(),
                        arrows: path.with_arrows(),
                        dismounted_from: dismounted_from.clone(),
                    });
                }
                game::action::Action::LoadInto {
                    unit_id,
                    load_into,
                    path,
                } => moves_ret.push(Action::LoadInto {
                    unit_id: unit_id.clone(),
                    load_into: load_into.clone(),
                    arrows: path.with_arrows(),
                    path: path.clone(),
                }),
                game::action::Action::Batch(more_moves) => {
                    let mut more_moves_ret = Action::from_game_actions(more_moves);
                    moves_ret.append(&mut more_moves_ret);
                }
                game::action::Action::PickUp {
                    unit_id,
                    path,
                    cargo_id,
                    ..
                } => {
                    moves_ret.push(Action::PickUp {
                        unit_id: unit_id.clone(),
                        cargo_id: cargo_id.clone(),
                        path: path.clone(),
                        arrows: path.with_arrows(),
                    });
                }
                game::action::Action::DropOff { cargo_id } => {
                    moves_ret.push(Action::DropOff { cargo_id })
                }
                game::action::Action::Replenish {
                    replenishing_unit_id,
                    units,
                    depleted_supply_crates,
                    path,
                    ..
                } => moves_ret.push(Action::Replenish {
                    replenishing_unit_id,
                    units: units.clone(),
                    depleted_supply_crates,
                    path: path.clone(),
                    arrows: path.with_arrows(),
                }),
                game::action::Action::Attack(game::action::Attack { unit_id, path }) => moves_ret
                    .push(Action::Attack {
                        unit_id: unit_id.clone(),
                        path: path.clone(),
                        arrows: path.with_arrows(),
                    }),
            }
        }

        moves_ret
    }

    pub fn arrows(&self) -> Option<&Vec<(Direction, Arrow)>> {
        match self {
            Action::TraveledTo { arrows, .. } => Some(arrows),
            Action::LoadInto { arrows, .. } => Some(arrows),
            Action::PickUp { arrows, .. } => Some(arrows),
            Action::DropOff { .. } => None,
            Action::Replenish { arrows, .. } => Some(arrows),
            Action::Attack { arrows, .. } => Some(arrows),
        }
    }
}
