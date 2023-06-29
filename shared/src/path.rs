use crate::arrow::Arrow;
use crate::direction::Direction;
use crate::located::Located;
use crate::map::Map;
use crate::unit::Unit;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Path {
    steps: Vec<Located<Step>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
struct Step {
    pub direction: Direction,
}

impl From<Direction> for Step {
    fn from(dir: Direction) -> Self {
        Step { direction: dir }
    }
}

impl Path {
    pub fn crosses(&self, other: &Path) -> Option<Located<()>> {
        for self_step in self.steps.iter() {
            for other_step in other.steps.iter() {
                if self_step.is_same_pos_as(other_step) {
                    return Some(self_step.to_unit());
                }
            }
        }

        None
    }

    pub fn shift_first(&mut self) -> Option<Located<Direction>> {
        if !self.steps.is_empty() {
            let removed = self.steps.remove(0);

            Some(removed.map_value(|step| step.direction))
        } else {
            None
        }
    }
    pub fn supply_cost(&self, map: &Map, unit: &Unit) -> i16 {
        let mut cost: i16 = 0;

        for loc_step in self.steps.iter() {
            let tile = map.get_tile(&loc_step.to_unit());

            cost += tile.travel_supply_cost(unit);
        }

        cost
    }
    pub fn last_pos(&self) -> Option<Located<()>> {
        self.steps.last().map(|loc_step| loc_step.to_unit())
    }

    pub fn to_directions(&self) -> Vec<Direction> {
        self.steps
            .clone()
            .into_iter()
            .map(|loc_dir| loc_dir.value.direction)
            .collect::<Vec<Direction>>()
    }

    pub fn with_arrows(&self) -> Vec<(Direction, Arrow)> {
        path_with_arrows(self.to_directions().as_slice())
    }

    pub fn from_directions<T>(loc: &Located<T>, dirs: &Vec<Direction>) -> Path {
        let mut path: Vec<Located<Step>> = Vec::new();

        let mut pos_x = loc.x;
        let mut pos_y = loc.y;

        if let Some(dir) = dirs.first() {
            path.push(Located {
                x: pos_x,
                y: pos_y,
                value: Step {
                    direction: dir.clone(),
                },
            });
        }

        for dir in dirs {
            dir.adjust_coord(&mut pos_x, &mut pos_y);

            path.push(Located {
                x: pos_x,
                y: pos_y,
                value: Step {
                    direction: dir.clone(),
                },
            });
        }

        Path { steps: path }
    }

    pub fn from_directions_test_only<T>(loc: &Located<T>, dirs: &Vec<Direction>) -> Path {
        Path::from_directions(loc, dirs)
    }
}

pub fn path_with_arrows(path: &[Direction]) -> Vec<(Direction, Arrow)> {
    let mut filtered_path = path.iter().collect::<Vec<_>>();

    let mut index = 0;
    while index < filtered_path.len() {
        let dir = path[index].clone();
        if let Some(next) = path.get(index + 1) {
            if dir == next.opposite() {
                if (index + 1) < filtered_path.len() {
                    filtered_path.remove(index + 1);
                }

                filtered_path.remove(index);

                index = 0;
            }
        }
        index += 1;
    }

    let mut filtered_path_peek = filtered_path.into_iter().peekable();

    let mut arrows = vec![];

    while let Some(dir) = filtered_path_peek.next() {
        let maybe_next = filtered_path_peek.peek();

        let arrow = match maybe_next {
            None => match dir {
                Direction::North => Arrow::EndUp,
                Direction::South => Arrow::EndDown,
                Direction::East => Arrow::EndRight,
                Direction::West => Arrow::EndLeft,
            },
            Some(next) => match (dir, next) {
                (Direction::North, Direction::North) => Arrow::Y,
                (Direction::North, Direction::East) => Arrow::LeftDown,
                (Direction::North, Direction::South) => {
                    panic!("Cannot move up and then down")
                }
                (Direction::North, Direction::West) => Arrow::RightDown,
                (Direction::East, Direction::North) => Arrow::RightUp,
                (Direction::East, Direction::East) => Arrow::X,
                (Direction::East, Direction::South) => Arrow::RightDown,
                (Direction::East, Direction::West) => {
                    panic!("Cannot move right then left")
                }
                (Direction::South, Direction::North) => {
                    panic!("Cannot move down then up")
                }
                (Direction::South, Direction::East) => Arrow::LeftUp,
                (Direction::South, Direction::South) => Arrow::Y,
                (Direction::South, Direction::West) => Arrow::RightUp,
                (Direction::West, Direction::North) => Arrow::LeftUp,
                (Direction::West, Direction::East) => {
                    panic!("Cannot move left then right")
                }
                (Direction::West, Direction::South) => Arrow::LeftDown,
                (Direction::West, Direction::West) => Arrow::X,
            },
        };

        arrows.push((dir.clone(), arrow));
    }

    arrows
}

#[cfg(test)]
mod test_path {
    use crate::direction::Direction;
    use crate::game::action::{order, unbatch, Action};
    use crate::located;
    use crate::located::Located;
    use crate::path::{Path, Step};
    use crate::rng::RandGen;
    use crate::tile::Tile;
    use crate::unit::UnitId;
    use pretty_assertions::assert_eq;

    #[test]
    fn crosses() {
        let path_1 = Path::from_directions_test_only(
            &located::unit(2, 0),
            &vec![
                Direction::South,
                Direction::South,
                Direction::South,
                Direction::South,
                Direction::South,
            ],
        );

        let path_2 = Path::from_directions_test_only(
            &located::unit(0, 2),
            &vec![
                Direction::East,
                Direction::East,
                Direction::East,
                Direction::East,
                Direction::East,
            ],
        );

        let got = path_1.crosses(&path_2);

        let want = Some(located::unit(2, 2));

        assert_eq!(want, got);
    }

    #[test]
    fn do_notcross() {
        let path_1 = Path::from_directions_test_only(
            &located::unit(2, 0),
            &vec![
                Direction::South,
                Direction::South,
                Direction::South,
                Direction::South,
                Direction::South,
            ],
        );

        let path_2 = Path::from_directions_test_only(
            &located::unit(4, 0),
            &vec![
                Direction::South,
                Direction::South,
                Direction::South,
                Direction::South,
                Direction::South,
            ],
        );

        let got = path_1.crosses(&path_2);

        let want = None;

        assert_eq!(want, got);
    }
}
