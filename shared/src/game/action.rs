use crate::path::Path;
use crate::rng::RandGen;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Action {
    Traveled {
        unit_id: UnitId,
        path: Path,
        dismounted_from: Option<UnitId>,
    },
    LoadInto {
        unit_id: UnitId,
        load_into: UnitId,
        path: Path,
    },
    PickedUp {
        unit_id: UnitId,
        cargo_id: UnitId,
        path: Path,
    },
    Batch(Vec<Action>),
}

pub fn order(rng: &mut RandGen, actions: &mut Vec<Action>) {
    let mut i = 0;

    while i < actions.len() {
        let action = actions[i].clone();

        match action {
            Action::Traveled {
                ref dismounted_from,
                ..
            } => {
                // A unit that travels out of a transport should travel
                // before its transport does
                if let Some(ref dismounted_from_unit_id) = dismounted_from {
                    let mut j = 0;
                    let mut continue_finding_transport = true;
                    while j < actions.len() && continue_finding_transport {
                        let possibly_travel_action = actions.get(j).unwrap().clone();

                        match possibly_travel_action {
                            Action::Traveled {
                                unit_id: ref possible_transport_id,
                                ..
                            } => {
                                if possible_transport_id == dismounted_from_unit_id {
                                    continue_finding_transport = false;

                                    let new_action = Action::Batch(vec![
                                        action.clone(),
                                        possibly_travel_action.clone(),
                                    ]);
                                    actions[i] = new_action;
                                    actions.remove(j);
                                    i = 0;
                                }
                            }
                            Action::LoadInto { .. } => {}
                            Action::Batch(_) => {}
                            Action::PickedUp { .. } => {}
                        }

                        j += 1;
                    }
                }
            }
            // A unit that loads into a transport, should load into
            // it before it travels away
            Action::LoadInto { ref load_into, .. } => {
                let mut j = 0;
                let mut continue_finding_transport = true;
                while j < actions.len() && continue_finding_transport {
                    let possibly_travel_action = actions.get(j).unwrap().clone();

                    match possibly_travel_action {
                        Action::Traveled {
                            unit_id: ref possible_transport_id,
                            ..
                        } => {
                            if possible_transport_id == load_into {
                                continue_finding_transport = false;

                                let new_action = Action::Batch(vec![
                                    action.clone(),
                                    possibly_travel_action.clone(),
                                ]);
                                actions[i] = new_action;
                                actions.remove(j);
                                i = 0;
                            }
                        }
                        Action::LoadInto { .. } => {}
                        Action::Batch(_) => {}
                        Action::PickedUp { .. } => {}
                    }

                    j += 1;
                }
            }
            Action::Batch(_) => {}
            Action::PickedUp { .. } => {}
        }

        i += 1;
    }

    let mut src_ordered = actions.clone();
    let mut random_ordered: Vec<Action> = Vec::new();

    while !src_ordered.is_empty() {
        let index = rng.gen::<usize>(0, src_ordered.len());

        random_ordered.push(src_ordered[index].clone());
        src_ordered.remove(index);
    }

    *actions = random_ordered;
}

#[cfg(test)]
mod test_game_actions {
    use crate::direction::Direction;
    use crate::game::action::{order, Action};
    use crate::located;
    use crate::path::Path;
    use crate::rng::RandGen;
    use crate::unit::UnitId;
    use pretty_assertions::assert_eq;

    #[test]
    fn units_loaded_into_trucks_load_before_truck_moves() {
        let mut rand_gen = RandGen::test();

        let truck_id = UnitId::new(&mut rand_gen);
        let infantry_id = UnitId::new(&mut rand_gen);

        let mut actions = vec![
            Action::Traveled {
                unit_id: truck_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(5, 5),
                    &vec![Direction::East, Direction::East, Direction::East],
                ),
                dismounted_from: None,
            },
            Action::LoadInto {
                unit_id: infantry_id.clone(),
                load_into: truck_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(3, 5),
                    &vec![Direction::East, Direction::East],
                ),
            },
        ];

        order(&mut rand_gen, &mut actions);

        assert_eq!(
            actions,
            vec![Action::Batch(vec![
                Action::LoadInto {
                    unit_id: infantry_id.clone(),
                    load_into: truck_id.clone(),
                    path: Path::from_directions_test_only(
                        &located::unit(3, 5),
                        &vec![Direction::East, Direction::East],
                    ),
                },
                Action::Traveled {
                    unit_id: truck_id.clone(),
                    path: Path::from_directions_test_only(
                        &located::unit(5, 5),
                        &vec![Direction::East, Direction::East, Direction::East],
                    ),
                    dismounted_from: None
                },
            ])]
        )
    }

    #[test]
    fn multiple_units_loaded_into_multiple_trucks_load_before_trucks_moves() {
        let mut rand_gen = RandGen::test();

        let truck_1_id = UnitId::new(&mut rand_gen);
        let infantry_1_id = UnitId::new(&mut rand_gen);

        let truck_2_id = UnitId::new(&mut rand_gen);
        let infantry_2_id = UnitId::new(&mut rand_gen);

        let mut actions = vec![
            Action::Traveled {
                unit_id: truck_2_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(10, 10),
                    &vec![Direction::East, Direction::East, Direction::East],
                ),
                dismounted_from: None,
            },
            Action::Traveled {
                unit_id: truck_1_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(5, 5),
                    &vec![Direction::East, Direction::East, Direction::East],
                ),
                dismounted_from: None,
            },
            Action::LoadInto {
                unit_id: infantry_1_id.clone(),
                load_into: truck_1_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(3, 5),
                    &vec![Direction::East, Direction::East],
                ),
            },
            Action::LoadInto {
                unit_id: infantry_2_id.clone(),
                load_into: truck_2_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(8, 10),
                    &vec![Direction::East, Direction::East],
                ),
            },
        ];

        order(&mut rand_gen, &mut actions);

        assert_eq!(
            actions,
            vec![
                Action::Batch(vec![
                    Action::LoadInto {
                        unit_id: infantry_1_id.clone(),
                        load_into: truck_1_id.clone(),
                        path: Path::from_directions_test_only(
                            &located::unit(3, 5),
                            &vec![Direction::East, Direction::East],
                        ),
                    },
                    Action::Traveled {
                        unit_id: truck_1_id.clone(),
                        path: Path::from_directions_test_only(
                            &located::unit(5, 5),
                            &vec![Direction::East, Direction::East, Direction::East],
                        ),
                        dismounted_from: None
                    },
                ]),
                Action::Batch(vec![
                    Action::LoadInto {
                        unit_id: infantry_2_id.clone(),
                        load_into: truck_2_id.clone(),
                        path: Path::from_directions_test_only(
                            &located::unit(8, 10),
                            &vec![Direction::East, Direction::East],
                        ),
                    },
                    Action::Traveled {
                        unit_id: truck_2_id.clone(),
                        path: Path::from_directions_test_only(
                            &located::unit(10, 10),
                            &vec![Direction::East, Direction::East, Direction::East],
                        ),
                        dismounted_from: None
                    },
                ])
            ]
        )
    }

    #[test]
    fn units_dismounting_from_trucks_dismount_before_truck_moves() {
        let mut rand_gen = RandGen::test();

        let truck_id = UnitId::new(&mut rand_gen);
        let infantry_id = UnitId::new(&mut rand_gen);

        let mut actions = vec![
            Action::Traveled {
                unit_id: truck_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(5, 5),
                    &vec![Direction::East, Direction::East, Direction::East],
                ),
                dismounted_from: None,
            },
            Action::Traveled {
                unit_id: infantry_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(5, 5),
                    &vec![Direction::East, Direction::East],
                ),
                dismounted_from: Some(truck_id.clone()),
            },
        ];

        order(&mut rand_gen, &mut actions);

        assert_eq!(
            actions,
            vec![Action::Batch(vec![
                Action::Traveled {
                    unit_id: infantry_id.clone(),
                    path: Path::from_directions_test_only(
                        &located::unit(5, 5),
                        &vec![Direction::East, Direction::East],
                    ),
                    dismounted_from: Some(truck_id.clone()),
                },
                Action::Traveled {
                    unit_id: truck_id.clone(),
                    path: Path::from_directions_test_only(
                        &located::unit(5, 5),
                        &vec![Direction::East, Direction::East, Direction::East],
                    ),
                    dismounted_from: None,
                },
            ])]
        )
    }
}
