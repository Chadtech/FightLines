use shared::direction::Direction;
use shared::point::Point;
use std::cmp::Ordering;
use std::collections::HashSet;

pub fn find(
    origin_pos: Point<i32>,
    dest_pos: Point<i32>,
    mobility: &HashSet<Point<i32>>,
    existing_directions: Vec<Direction>,
    mobility_budget: usize,
) -> Result<Vec<Direction>, String> {
    let existing_path: Vec<(Point<i32>, Direction)> = existing_directions
        .iter()
        .fold((origin_pos.clone(), vec![]), |(mut point, mut acc), dir| {
            match dir {
                Direction::North => {
                    point.y -= 1;
                }
                Direction::South => {
                    point.y += 1;
                }
                Direction::East => {
                    point.x += 1;
                }
                Direction::West => {
                    point.x -= 1;
                }
            }

            acc.push((point.clone(), dir.clone()));

            (point, acc)
        })
        .1;

    let mut path = existing_path;

    let mut escape_with: Option<Vec<Direction>> = None;

    let (mut pos, mut last_dir) = path
        .last()
        .map(|(p, dir)| (p.clone(), Some(dir.clone())))
        .unwrap_or_else(|| (origin_pos.clone(), None));

    let y_adjustments = |pos: &mut Point<i32>| match pos.y.cmp(&dest_pos.y) {
        Ordering::Less => {
            pos.y += 1;

            Some(Direction::South)
        }
        Ordering::Equal => None,
        Ordering::Greater => {
            pos.y -= 1;

            Some(Direction::North)
        }
    };

    let x_adjustments = |pos: &mut Point<i32>| match pos.x.cmp(&dest_pos.x) {
        Ordering::Less => {
            pos.x += 1;

            Some(Direction::East)
        }
        Ordering::Equal => None,
        Ordering::Greater => {
            pos.x -= 1;

            Some(Direction::West)
        }
    };

    while pos != dest_pos && escape_with.is_none() {
        if path.len() > mobility_budget {
            path = vec![];
            pos = origin_pos.clone()
        };

        let maybe_dir: Option<Direction> =
            if last_dir.clone().map(|d| d.is_x_axis()).unwrap_or(false) {
                y_adjustments(&mut pos).or_else(|| x_adjustments(&mut pos))
            } else {
                x_adjustments(&mut pos).or_else(|| y_adjustments(&mut pos))
            };

        if let Some(dir) = maybe_dir {
            last_dir = Some(dir.clone());

            if mobility.contains(&pos) {
                path.push((pos.clone(), dir));
            } else {
                escape_with = Some(search_and_find_movement_path(
                    origin_pos.clone(),
                    dest_pos.clone(),
                    mobility,
                    path.clone(),
                )?);
            }
        }
    }

    let filtered_for_double_backs = {
        let mut path = escape_with.unwrap_or_else(|| {
            path.iter()
                .map(|(_, dir)| dir.clone())
                .collect::<Vec<Direction>>()
        });

        let mut path_peek = path.iter().enumerate().clone().peekable();

        let mut deletable_steps = vec![];

        while let Some((index, step)) = path_peek.next() {
            if let Some((_, next)) = path_peek.peek() {
                match (step, next) {
                    (Direction::South, Direction::North) => {
                        deletable_steps.push(index);
                        deletable_steps.push(index + 1);
                        path_peek.next();
                    }
                    (Direction::North, Direction::South) => {
                        deletable_steps.push(index);
                        deletable_steps.push(index + 1);
                        path_peek.next();
                    }
                    (Direction::East, Direction::West) => {
                        deletable_steps.push(index);
                        deletable_steps.push(index + 1);
                        path_peek.next();
                    }
                    (Direction::West, Direction::East) => {
                        deletable_steps.push(index);
                        deletable_steps.push(index + 1);
                        path_peek.next();
                    }
                    _ => {}
                }
            }
        }

        deletable_steps.reverse();

        for step in deletable_steps {
            path.remove(step);
        }

        path
    };

    Ok(filtered_for_double_backs)
}

fn search_and_find_movement_path(
    origin_pos: Point<i32>,
    dest_pos: Point<i32>,
    mobility: &HashSet<Point<i32>>,
    existing_path: Vec<(Point<i32>, Direction)>,
) -> Result<Vec<Direction>, String> {
    let mut queue: Vec<Vec<(Point<i32>, Direction)>> = vec![existing_path];
    let mut found: Option<Vec<Direction>> = None;

    while !queue.is_empty() && found.is_none() {
        let mut tries_to_delete = vec![];

        for (index, path) in queue.clone().iter().enumerate() {
            let last_pos = path
                .last()
                .map(|(pos, _)| pos.clone())
                .unwrap_or_else(|| origin_pos.clone());

            if last_pos.x == dest_pos.x && last_pos.y == dest_pos.y {
                found = Some(
                    path.iter()
                        .map(|(_, dir)| dir.clone())
                        .collect::<Vec<Direction>>(),
                );
            } else if mobility.contains(&last_pos) {
                let north_path = {
                    let mut ret = path.clone();
                    let new_pos = Point {
                        x: last_pos.x,
                        y: last_pos.y - 1,
                    };

                    ret.push((new_pos, Direction::North));

                    ret
                };

                queue.push(north_path);

                let west_path = {
                    let mut ret = path.clone();
                    let new_pos = Point {
                        x: last_pos.x - 1,
                        y: last_pos.y,
                    };

                    ret.push((new_pos, Direction::West));

                    ret
                };

                queue.push(west_path);

                let south_path = {
                    let mut ret = path.clone();
                    let new_pos = Point {
                        x: last_pos.x,
                        y: last_pos.y + 1,
                    };

                    ret.push((new_pos, Direction::South));

                    ret
                };

                queue.push(south_path);

                let east_path = {
                    let mut ret = path.clone();
                    let new_pos = Point {
                        x: last_pos.x + 1,
                        y: last_pos.y,
                    };

                    ret.push((new_pos, Direction::East));

                    ret
                };

                queue.push(east_path);
            }

            tries_to_delete.push(index);
        }

        tries_to_delete.reverse();

        for try_to_delete in tries_to_delete {
            queue.remove(try_to_delete);
        }
    }

    match found {
        Some(path) => Ok(path),
        None => Err("search finished without finding path".to_string()),
    }
}

#[cfg(test)]
mod test_movement_arrow {
    use crate::game::Arrow;
    use crate::page::game::movement_path::{find, search_and_find_movement_path};
    use pretty_assertions::assert_eq;
    use shared::direction::Direction;
    use shared::path::path_with_arrows;
    use shared::point::Point;
    use std::collections::HashSet;

    fn path_to_arrows(path: &Vec<Direction>) -> Vec<Arrow> {
        path_with_arrows(path)
            .into_iter()
            .map(|(_, arrow)| arrow)
            .collect::<Vec<_>>()
    }

    fn large_mobility() -> HashSet<Point<i32>> {
        let mut ret = HashSet::new();

        for x in -32..32 {
            for y in -32..32 {
                ret.insert(Point { x, y });
            }
        }

        ret
    }

    fn mobility_from_str(s: &str) -> HashSet<Point<i32>> {
        s.trim()
            .split('\n')
            .enumerate()
            .map(|(ri, row)| {
                let row_of_pos: Vec<Point<i32>> = row
                    .chars()
                    .enumerate()
                    .filter_map(|(ci, char)| {
                        if char == '#' || char == 'U' || char == 'T' {
                            Some(Point {
                                x: (ci as i32),
                                y: (ri as i32),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Point<i32>>>();

                row_of_pos
            })
            .flatten()
            .collect::<HashSet<Point<i32>>>()
    }

    #[test]
    fn no_path_for_origin() {
        let want: Vec<Direction> = vec![];
        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 0, y: 0 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn east_path_for_mouse_east() {
        let want: Vec<Direction> = vec![Direction::East];
        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn many_east_path_for_mouse_very_east() {
        let want: Vec<Direction> = vec![
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
        ];
        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 8, y: 0 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn many_west_path_for_mouse_very_west() {
        let want: Vec<Direction> = vec![
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
        ];
        assert_eq!(
            want,
            find(
                Point { x: 8, y: 0 },
                Point { x: 0, y: 0 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn many_north_path_for_mouse_very_north() {
        let want: Vec<Direction> = vec![
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
        ];
        assert_eq!(
            want,
            find(
                Point { x: 0, y: 8 },
                Point { x: 0, y: 0 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn many_south_path_for_mouse_very_south() {
        let want: Vec<Direction> = vec![
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
        ];
        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 0, y: 8 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn path_can_go_diagonal() {
        let want: Vec<Direction> = vec![Direction::East, Direction::South];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 1, y: 1 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn path_can_go_diagonal_far() {
        let want: Vec<Direction> = vec![
            Direction::East,
            Direction::North,
            Direction::East,
            Direction::North,
            Direction::East,
            Direction::North,
            Direction::East,
            Direction::North,
        ];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 4 },
                Point { x: 4, y: 0 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn path_can_go_diagonal_irregular() {
        let want: Vec<Direction> = vec![
            Direction::West,
            Direction::North,
            Direction::West,
            Direction::North,
            Direction::West,
            Direction::West,
        ];

        assert_eq!(
            want,
            find(
                Point { x: 4, y: 4 },
                Point { x: 0, y: 2 },
                &large_mobility(),
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn path_can_work_off_existing_path() {
        let want: Vec<Direction> = vec![
            Direction::South,
            Direction::South,
            Direction::West,
            Direction::South,
            Direction::West,
            Direction::South,
            Direction::West,
            Direction::West,
        ];

        assert_eq!(
            want,
            find(
                Point { x: 4, y: 0 },
                Point { x: 0, y: 4 },
                &large_mobility(),
                vec![Direction::South, Direction::South],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn east_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::X, Arrow::X, Arrow::EndRight];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::East, Direction::East, Direction::East])
        );
    }

    #[test]
    fn south_west_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::RightUp, Arrow::LeftDown, Arrow::EndDown];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::South, Direction::West, Direction::South])
        );
    }

    #[test]
    fn north_west_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::RightDown, Arrow::LeftUp, Arrow::EndUp];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::North, Direction::West, Direction::North])
        );
    }

    #[test]
    fn north_east_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::LeftDown, Arrow::RightUp, Arrow::EndUp];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::North, Direction::East, Direction::North])
        );
    }

    #[test]
    fn south_east_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::LeftUp, Arrow::RightDown, Arrow::EndDown];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::South, Direction::East, Direction::South])
        );
    }

    #[test]
    fn west_south_path_to_arrow() {
        let want: Vec<Arrow> = vec![
            Arrow::LeftDown,
            Arrow::RightUp,
            Arrow::LeftDown,
            Arrow::RightUp,
            Arrow::EndLeft,
        ];

        assert_eq!(
            want,
            path_to_arrows(&vec![
                Direction::West,
                Direction::South,
                Direction::West,
                Direction::South,
                Direction::West
            ])
        );
    }

    #[test]
    fn path_to_arrows_filters_double_backs() {
        let want: Vec<Arrow> = vec![Arrow::X, Arrow::EndRight];

        assert_eq!(
            want,
            path_to_arrows(&vec![
                Direction::East,
                Direction::East,
                Direction::East,
                Direction::West
            ])
        )
    }

    #[test]
    fn edge_of_range_can_be_approached_from_north() {
        let want: Vec<Direction> = vec![Direction::East, Direction::South];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 1, y: 1 },
                &large_mobility(),
                vec![Direction::East],
                32,
            )
            .unwrap()
        );
    }

    #[test]
    fn edge_of_range_can_be_approached_from_west() {
        let want: Vec<Direction> = vec![Direction::South, Direction::East];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 1, y: 1 },
                &large_mobility(),
                vec![Direction::South],
                32,
            )
            .unwrap()
        );
    }

    #[test]
    fn search_and_find_north_line_path() {
        let mut mobility = HashSet::new();

        for y in 0..5 {
            mobility.insert(Point { x: 0, y });
        }

        let got = search_and_find_movement_path(
            Point { x: 0, y: 4 },
            Point { x: 0, y: 0 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_and_find_west_line_path() {
        let mut mobility = HashSet::new();

        for x in 0..5 {
            mobility.insert(Point { x, y: 0 });
        }

        let got = search_and_find_movement_path(
            Point { x: 4, y: 0 },
            Point { x: 0, y: 0 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_and_find_north_west_turn_path() {
        let mut mobility = HashSet::new();

        for x in 0..5 {
            for y in 0..5 {
                mobility.insert(Point { x, y });
            }
        }

        let got = search_and_find_movement_path(
            Point { x: 4, y: 4 },
            Point { x: 0, y: 0 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_and_find_north_west_limited_turn_path_1() {
        let mut mobility = HashSet::new();

        for y in 0..5 {
            mobility.insert(Point { x: 0, y });
        }

        for x in 0..5 {
            mobility.insert(Point { x, y: 4 });
        }

        let got = search_and_find_movement_path(
            Point { x: 4, y: 4 },
            Point { x: 0, y: 0 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_and_find_north_west_limited_turn_path_2() {
        let mut mobility = HashSet::new();

        for y in 0..5 {
            mobility.insert(Point { x: 4, y });
        }

        for x in 0..5 {
            mobility.insert(Point { x, y: 0 });
        }

        let got = search_and_find_movement_path(
            Point { x: 4, y: 4 },
            Point { x: 0, y: 0 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_spiral_path_1() {
        let mobility = mobility_from_str(
            r#"
###
# #
"#,
        );

        let got = search_and_find_movement_path(
            Point { x: 2, y: 1 },
            Point { x: 0, y: 1 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::North,
            Direction::West,
            Direction::West,
            Direction::South,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_spiral_path_2() {
        let mobility = mobility_from_str(
            r#"
###
# #
#
#
#
"#,
        );

        let got = search_and_find_movement_path(
            Point { x: 2, y: 1 },
            Point { x: 0, y: 4 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::North,
            Direction::West,
            Direction::West,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_spiral_path_3() {
        let mobility = mobility_from_str(
            r#"
###
# #
#
#
##
"#,
        );

        let got = search_and_find_movement_path(
            Point { x: 2, y: 1 },
            Point { x: 1, y: 4 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::North,
            Direction::West,
            Direction::West,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::East,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn search_spiral_path_4() {
        let mobility = mobility_from_str(
            r#"
###
# #
#
# #
###
"#,
        );

        let got = search_and_find_movement_path(
            Point { x: 2, y: 1 },
            Point { x: 2, y: 3 },
            &mobility,
            vec![],
        )
        .unwrap();

        let want = vec![
            Direction::North,
            Direction::West,
            Direction::West,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::East,
            Direction::East,
            Direction::North,
        ];

        assert_eq!(want, got);
    }

    #[test]
    fn discontinuous_spiral_path_does_not_stack_overflow() {
        let mobility = mobility_from_str(
            r#"
# #
"#,
        );

        let got = search_and_find_movement_path(
            Point { x: 2, y: 0 },
            Point { x: 0, y: 0 },
            &mobility,
            vec![],
        )
        .is_err();

        let want = true;

        assert_eq!(want, got);
    }

    #[test]
    fn arrow_calc_that_needs_search_1() {
        let mobility = mobility_from_str(
            r#"
###
U T
    "#,
        );

        let want: Vec<Direction> = vec![
            Direction::North,
            Direction::East,
            Direction::East,
            Direction::South,
        ];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 1 },
                Point { x: 2, y: 1 },
                &mobility,
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn find_movement_path_east() {
        let mobility = mobility_from_str(
            r#"
####
"#,
        );

        let want: Vec<Direction> = vec![Direction::East, Direction::East, Direction::East];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 3, y: 0 },
                &mobility,
                vec![],
                32,
            )
            .unwrap()
        );
    }

    #[test]
    fn find_movement_path_south() {
        let mobility = mobility_from_str(
            r#"
#
#
#
#
"#,
        );

        let want: Vec<Direction> = vec![Direction::South, Direction::South, Direction::South];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 0, y: 3 },
                &mobility,
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn find_movement_path_east_existing_path() {
        let mobility = mobility_from_str(
            r#"
####
"#,
        );

        let want: Vec<Direction> = vec![Direction::East, Direction::East, Direction::East];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 3, y: 0 },
                &mobility,
                vec![Direction::East],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn find_movement_path_south_existing_path() {
        let mobility = mobility_from_str(
            r#"
#
#
#
#
"#,
        );

        let want: Vec<Direction> = vec![Direction::South, Direction::South, Direction::South];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 0, y: 3 },
                &mobility,
                vec![Direction::South],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn find_movement_path_east_more_existing_path() {
        let mobility = mobility_from_str(
            r#"
####
"#,
        );

        let want: Vec<Direction> = vec![Direction::East, Direction::East, Direction::East];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 3, y: 0 },
                &mobility,
                vec![Direction::East, Direction::East],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn find_movement_path_south_more_existing_path() {
        let mobility = mobility_from_str(
            r#"
#
#
#
#
"#,
        );

        let want: Vec<Direction> = vec![Direction::South, Direction::South, Direction::South];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 0, y: 3 },
                &mobility,
                vec![Direction::South, Direction::South],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn find_movement_path_south_east_more_existing_path() {
        let mobility = mobility_from_str(
            r#"
#
#
#
####
"#,
        );

        let want: Vec<Direction> = vec![
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::East,
            Direction::East,
            Direction::East,
        ];

        assert_eq!(
            want,
            find(
                Point { x: 0, y: 0 },
                Point { x: 3, y: 3 },
                &mobility,
                vec![Direction::South, Direction::South],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn arrow_calc_that_needs_search_going_south() {
        let mobility = mobility_from_str(
            r#"
###  ###
###  ###
########
##U  ###
###  #T#
###  ###
"#,
        );

        let want: Vec<Direction> = vec![
            Direction::North,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::South,
            Direction::South,
        ];

        assert_eq!(
            want,
            find(
                Point { x: 2, y: 3 },
                Point { x: 6, y: 4 },
                &mobility,
                vec![],
                32
            )
            .unwrap()
        );
    }

    #[test]
    fn arrow_calc_that_needs_search_going_north() {
        let mobility = mobility_from_str(
            r#"
###  #T#
###  ###
########
##U  ###
###  ###
###  ###
"#,
        );

        let want: Vec<Direction> = vec![
            Direction::North,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::North,
            Direction::North,
        ];

        assert_eq!(
            want,
            find(
                Point { x: 2, y: 3 },
                Point { x: 6, y: 0 },
                &mobility,
                vec![],
                32
            )
            .unwrap()
        );
    }
}
