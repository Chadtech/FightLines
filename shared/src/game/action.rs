use crate::facing_direction::FacingDirection;
use crate::located::Located;
use crate::path::Path;
use crate::rng::RandGen;
use crate::unit::UnitId;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Action {
    Travel {
        unit_id: UnitId,
        path: Path,
        dismounted_from: Option<UnitId>,
    },
    LoadInto {
        unit_id: UnitId,
        load_into: UnitId,
        path: Path,
    },
    PickUp {
        unit_id: UnitId,
        cargo_id: UnitId,
        path: Path,
    },
    DropOff {
        cargo_unit_loc: Located<(FacingDirection, UnitId)>,
        transport_id: UnitId,
    },
    Replenish {
        replenishing_unit_id: UnitId,
        units: Vec<(UnitId, i16)>,
        depleted_supply_crates: Vec<(UnitId, i16)>,
        path: Path,
    },
    Attack(Attack),
    Batch(Vec<Action>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Attack {
    pub unit_id: UnitId,
    pub path: Path,
}

impl Action {
    // None means no delete
    // Some means delete, and the vec is of the actions to replace the action with.
    // Some(vec![]) means delete and no replacement
    // A case of replacement might include a unit moving to load into a truck, but
    // the truck gets deleted. In this case the LoadInto action should be replaced
    // with a Travel action
    pub fn when_unit_deleted(&self, deleted_unit_id: UnitId) -> Option<Vec<Action>> {
        match self {
            Action::Travel { unit_id, .. } => {
                if deleted_unit_id == unit_id.clone() {
                    Some(vec![])
                } else {
                    None
                }
            }
            Action::LoadInto {
                unit_id,
                load_into,
                path,
            } => {
                if unit_id.clone() == deleted_unit_id {
                    Some(vec![])
                } else if deleted_unit_id == load_into.clone() {
                    Some(vec![Action::Travel {
                        unit_id: unit_id.clone(),
                        path: path.clone(),
                        dismounted_from: None,
                    }])
                } else {
                    None
                }
            }
            Action::PickUp {
                cargo_id,
                unit_id,
                path,
            } => {
                if unit_id.clone() == deleted_unit_id {
                    Some(vec![])
                } else if cargo_id.clone() == deleted_unit_id {
                    Some(vec![Action::Travel {
                        unit_id: unit_id.clone(),
                        path: path.clone(),
                        dismounted_from: None,
                    }])
                } else {
                    None
                }
            }
            Action::DropOff { cargo_unit_loc, .. } => {
                if cargo_unit_loc.value.1.clone() == deleted_unit_id {
                    Some(vec![])
                } else {
                    None
                }
            }
            Action::Replenish {
                replenishing_unit_id,
                ..
            } => {
                if deleted_unit_id == replenishing_unit_id.clone() {
                    Some(vec![])
                } else {
                    None
                }
            }
            Action::Attack(Attack { unit_id, .. }) => {
                if unit_id.clone() == deleted_unit_id {
                    Some(vec![])
                } else {
                    None
                }
            }
            Action::Batch(_) => None,
        }
    }
    pub fn to_priority(&self) -> u8 {
        match self {
            Action::Travel { .. } => 10,
            Action::LoadInto { .. } => 10,
            Action::PickUp { .. } => 10,
            Action::DropOff { .. } => 10,
            Action::Replenish { .. } => 0,
            Action::Attack { .. } => 5,
            Action::Batch(_) => 10,
        }
    }

    pub fn closest_crossing_attack_path<'a>(
        path: &Path,
        actions: &'a [Action],
    ) -> Option<(usize, Located<&'a Attack>)> {
        let mut closest_crossing_path: Option<(usize, Located<&'a Attack>)> = None;

        if let Some(origin) = path.first_pos() {
            let mut i = 0;

            while i < actions.len() {
                let action = actions.get(i).unwrap();

                if let Action::Attack(attack) = action {
                    if let Some(cross_loc) = path.crosses(&attack.path) {
                        match closest_crossing_path.clone() {
                            None => closest_crossing_path = Some((i, cross_loc.with_value(attack))),
                            Some((_, existing)) => {
                                if origin.distance_from(&existing)
                                    > origin.distance_from(&cross_loc)
                                {
                                    closest_crossing_path = Some((
                                        i,
                                        Located {
                                            x: cross_loc.x,
                                            y: cross_loc.y,
                                            value: attack,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }

                i += 1
            }
        }

        closest_crossing_path
    }

    pub fn attacking_path(&self) -> Option<&Path> {
        match self {
            Action::Attack(Attack { path, .. }) => Some(path),
            _ => None,
        }
    }
}

impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_priority().cmp(&other.to_priority())
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn unbatch(actions: &mut Vec<Action>) {
    let mut i = 0;

    while i < actions.len() {
        match actions.get(i).unwrap() {
            Action::Batch(batched_actions) => {
                let batched_actions = batched_actions.clone();
                let num_batched_actions = batched_actions.len();

                actions.splice(i..(i + 1), batched_actions.into_iter());

                i += num_batched_actions;
            }
            _ => {
                i += 1;
            }
        }
    }
}

pub fn order(rng: &mut RandGen, actions: &mut Vec<Action>) {
    let mut i = 0;

    actions.sort();

    while i < actions.len() {
        let action = actions[i].clone();

        match action {
            Action::Travel {
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
                            Action::Travel {
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
                            Action::PickUp { .. } => {}
                            Action::DropOff { .. } => {}
                            Action::Replenish { .. } => {}
                            Action::Attack { .. } => {}
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
                        Action::Travel {
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
                        Action::PickUp { .. } => {}
                        Action::DropOff { .. } => {}
                        Action::Replenish { .. } => {}
                        Action::Attack { .. } => {}
                    }

                    j += 1;
                }
            }
            Action::Batch(_) => {}
            Action::PickUp { .. } => {}
            Action::DropOff { .. } => {}
            Action::Replenish { .. } => {}
            Action::Attack { .. } => {}
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
    use crate::game::action::{order, unbatch, Action};
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
            Action::Travel {
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
                Action::Travel {
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
            Action::Travel {
                unit_id: truck_2_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(10, 10),
                    &vec![Direction::East, Direction::East, Direction::East],
                ),
                dismounted_from: None,
            },
            Action::Travel {
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
                    Action::Travel {
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
                    Action::Travel {
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
    fn unbatching_works() {
        let action_1 = Action::Travel {
            unit_id: UnitId::test("A"),
            path: Path::from_directions_test_only(&located::unit(1, 1), &vec![Direction::North]),
            dismounted_from: None,
        };

        let action_2 = Action::Travel {
            unit_id: UnitId::test("B"),
            path: Path::from_directions_test_only(&located::unit(1, 1), &vec![Direction::North]),
            dismounted_from: None,
        };

        let action_3 = Action::Travel {
            unit_id: UnitId::test("C"),
            path: Path::from_directions_test_only(&located::unit(1, 1), &vec![Direction::North]),
            dismounted_from: None,
        };

        let action_4 = Action::Travel {
            unit_id: UnitId::test("D"),
            path: Path::from_directions_test_only(&located::unit(1, 1), &vec![Direction::North]),
            dismounted_from: None,
        };

        let want = vec![
            action_1.clone(),
            action_2.clone(),
            action_3.clone(),
            action_4.clone(),
        ];

        let mut got = vec![action_1, Action::Batch(vec![action_2, action_3]), action_4];

        unbatch(&mut got);

        assert_eq!(got.clone(), want);
    }

    #[test]
    fn units_dismounting_from_trucks_dismount_before_truck_moves() {
        let mut rand_gen = RandGen::test();

        let truck_id = UnitId::new(&mut rand_gen);
        let infantry_id = UnitId::new(&mut rand_gen);

        let mut actions = vec![
            Action::Travel {
                unit_id: truck_id.clone(),
                path: Path::from_directions_test_only(
                    &located::unit(5, 5),
                    &vec![Direction::East, Direction::East, Direction::East],
                ),
                dismounted_from: None,
            },
            Action::Travel {
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
                Action::Travel {
                    unit_id: infantry_id.clone(),
                    path: Path::from_directions_test_only(
                        &located::unit(5, 5),
                        &vec![Direction::East, Direction::East],
                    ),
                    dismounted_from: Some(truck_id.clone()),
                },
                Action::Travel {
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
