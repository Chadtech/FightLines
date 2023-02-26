use crate::path::Path;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Action {
    Traveled {
        unit_id: UnitId,
        path: Path,
    },
    LoadInto {
        unit_id: UnitId,
        load_into: UnitId,
        path: Path,
    },
    Batch(Vec<Action>),
}

pub fn order(actions: &mut Vec<Action>) {
    let mut i = 0;

    while i < actions.len() {
        let action = actions[i].clone();

        match action {
            Action::Traveled { .. } => {}
            Action::LoadInto { ref load_into, .. } => {
                let mut j = 0;
                let mut continue_finding_transport = true;
                while j < actions.len() && continue_finding_transport {
                    let possibly_travel_action = actions[j].clone();

                    match possibly_travel_action {
                        Action::Traveled {
                            unit_id: ref possible_transport_id,
                            ..
                        } => {
                            if possible_transport_id == load_into {
                                continue_finding_transport = false;
                            }

                            let new_action =
                                Action::Batch(vec![action.clone(), possibly_travel_action.clone()]);
                            actions[i] = new_action;
                            actions.remove(j);
                        }
                        Action::LoadInto { .. } => {}
                        Action::Batch(_) => {}
                    }

                    j += 1;
                }
            }
            Action::Batch(_) => {}
        }

        i += 1;
    }
}

#[cfg(test)]
mod test_game_actions {
    use crate::direction::Direction;
    use crate::game::action::{order, Action};
    use crate::located;
    use crate::path::Path;
    use crate::rng::RandGen;
    use crate::unit::UnitId;

    #[test]
    fn can_be_sorted() {
        let mut rand_gen = RandGen::test();

        let truck_id = UnitId::new(&mut rand_gen);
        let infantry_id = UnitId::new(&mut rand_gen);

        let mut actions = vec![
            Action::Traveled {
                unit_id: truck_id.clone(),
                path: Path::from_directions(
                    &located::unit(5, 5),
                    &vec![Direction::East, Direction::East, Direction::East],
                ),
            },
            Action::LoadInto {
                unit_id: infantry_id.clone(),
                load_into: truck_id.clone(),
                path: Path::from_directions(
                    &located::unit(3, 5),
                    &vec![Direction::East, Direction::East],
                ),
            },
        ];

        order(&mut actions);

        assert_eq!(
            actions,
            vec![Action::Batch(vec![
                Action::LoadInto {
                    unit_id: infantry_id.clone(),
                    load_into: truck_id.clone(),
                    path: Path::from_directions(
                        &located::unit(3, 5),
                        &vec![Direction::East, Direction::East],
                    ),
                },
                Action::Traveled {
                    unit_id: truck_id.clone(),
                    path: Path::from_directions(
                        &located::unit(5, 5),
                        &vec![Direction::East, Direction::East, Direction::East],
                    ),
                },
            ])]
        )
    }
}
