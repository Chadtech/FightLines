use crate::game::unit_index::by_id;
use crate::id::Id;
use crate::located::Located;
use crate::path::Path;
use crate::rng::RandGen;
use crate::unit;
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
        cargo_id: UnitId,
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

#[derive(Clone)]
pub struct ClosestCrossingEnemyPath<'a> {
    pub action_index: usize,
    pub action: &'a Action,
    pub unit_loc: Located<(UnitId, unit::Model)>,
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
            Action::DropOff { cargo_id } => {
                if cargo_id.clone() == deleted_unit_id {
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

    pub fn path(&self) -> Option<&Path> {
        match self {
            Action::Travel { path, .. } => Some(path),
            Action::LoadInto { path, .. } => Some(path),
            Action::PickUp { path, .. } => Some(path),
            Action::DropOff { .. } => None,
            Action::Replenish { path, .. } => Some(path),
            Action::Batch(_) => {
                // I don't think batch should ever be used in the contexts
                // where path() is called
                None
            }
            Action::Attack(Attack { path, .. }) => Some(path),
        }
    }

    pub fn moving_unit(&self) -> Option<&UnitId> {
        match self {
            Action::Travel { unit_id, .. } => Some(unit_id),
            Action::LoadInto { unit_id, .. } => Some(unit_id),
            Action::PickUp { unit_id, .. } => Some(unit_id),
            Action::DropOff { .. } => None,
            Action::Replenish {
                replenishing_unit_id,
                ..
            } => Some(replenishing_unit_id),
            Action::Attack(Attack { unit_id, .. }) => Some(unit_id),
            Action::Batch(_) => None,
        }
    }

    pub fn closest_crossing_enemy_path<'a>(
        by_id: &by_id::Index,
        path: &Path,
        player_id: Id,
        actions: &'a [Action],
    ) -> Result<Option<ClosestCrossingEnemyPath<'a>>, String> {
        let mut closest_crossing_path: Option<ClosestCrossingEnemyPath> = None;

        if let Some(origin) = path.first_pos() {
            let mut i = 0;

            while i < actions.len() {
                let action = actions.get(i).unwrap();

                let (unit_id, action_path) = if let (Some(unit_id), Some(action_path)) =
                    (action.moving_unit(), action.path())
                {
                    (unit_id, action_path)
                } else {
                    break;
                };

                let unit_model = match by_id.get(unit_id) {
                    None => return Err("could not find unit referenced by action".to_string()),
                    Some(unit_model) => {
                        if player_id == unit_model.owner {
                            i += 1;
                            continue;
                        }

                        unit_model
                    }
                };

                let cross_loc = match path.crosses(action_path) {
                    None => {
                        i += 1;
                        continue;
                    }
                    Some(l) => l,
                };

                match closest_crossing_path.clone().map(|c| c.unit_loc) {
                    None => {
                        closest_crossing_path = Some(ClosestCrossingEnemyPath {
                            action_index: i,
                            action,
                            unit_loc: cross_loc.with_value((unit_id.clone(), unit_model.clone())),
                        });
                    }
                    Some(existing) => {
                        if origin.distance_from(&existing) > origin.distance_from(&cross_loc) {
                            closest_crossing_path = Some(ClosestCrossingEnemyPath {
                                action_index: i,
                                action,
                                unit_loc: cross_loc
                                    .with_value((unit_id.clone(), unit_model.clone())),
                            });
                        }
                    }
                }

                i += 1
            }
        }

        Ok(closest_crossing_path)
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
    use crate::facing_direction::FacingDirection;
    use crate::game::action::{order, unbatch, Action, Attack};
    use crate::game::unit_index::by_id;
    use crate::id::Id;
    use crate::located::Located;
    use crate::path::Path;
    use crate::rng::RandGen;
    use crate::team_color::TeamColor;
    use crate::unit::{Place, Unit, UnitId};
    use crate::{located, unit};
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

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

    #[test]
    fn closest_crossing_enemy_path() {
        let mut units = HashMap::new();

        let red_player_id = Id::test("red player");
        let blue_player_id = Id::test("blue player");

        let red_tank_id = UnitId::test("red tank");

        let red_tank_loc = located::unit(2, 2);

        let red_tank = unit::Model::new(
            Unit::Tank,
            &red_player_id,
            Place::OnMap(red_tank_loc.with_value(FacingDirection::Right)),
            &TeamColor::Red,
        );

        units.insert(red_tank_id.clone(), red_tank);

        let blue_infantry_id = UnitId::test("blue infantry");

        let blue_infantry_loc = located::unit(3, 1);

        let blue_infantry = unit::Model::new(
            Unit::Infantry,
            &blue_player_id,
            Place::OnMap(blue_infantry_loc.with_value(FacingDirection::Right)),
            &TeamColor::Blue,
        );

        units.insert(blue_infantry_id.clone(), blue_infantry.clone());

        let by_id_index = by_id::Index::from_hash_map(units);

        let blue_infantry_travel_action = Action::Travel {
            unit_id: blue_infantry_id.clone(),
            path: Path::from_directions_test_only(
                &blue_infantry_loc,
                &vec![Direction::South, Direction::South],
            ),
            dismounted_from: None,
        };

        let actions = vec![
            Action::Attack(Attack {
                unit_id: red_tank_id,
                path: Path::from_directions_test_only(
                    &red_tank_loc,
                    &vec![
                        Direction::East,
                        Direction::East,
                        Direction::East,
                        Direction::East,
                    ],
                ),
            }),
            blue_infantry_travel_action.clone(),
        ];

        let got = Action::closest_crossing_enemy_path(
            &by_id_index,
            &Path::from_directions_test_only(
                &located::unit(2, 2),
                &vec![Direction::East, Direction::East, Direction::East],
            ),
            red_player_id,
            actions.as_slice(),
        )
        .unwrap();

        let want: Option<(usize, &Action, Located<(UnitId, unit::Model)>)> = Some((
            1,
            &blue_infantry_travel_action,
            Located {
                x: 3,
                y: 2,
                value: (blue_infantry_id, blue_infantry),
            },
        ));

        assert_eq!(want, got)
    }
}
