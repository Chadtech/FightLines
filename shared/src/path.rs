use crate::arrow::Arrow;
use crate::direction::Direction;
use crate::located::Located;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Path {
    steps: Vec<Located<Direction>>,
}

impl Path {
    pub fn last(&self) -> Option<&Located<Direction>> {
        self.steps.last()
    }

    pub fn to_directions(&self) -> Vec<Direction> {
        self.steps
            .clone()
            .into_iter()
            .map(|loc_dir| loc_dir.value)
            .collect::<Vec<Direction>>()
    }

    pub fn with_arrows(&self) -> Vec<(Direction, Arrow)> {
        path_with_arrows(self.to_directions().as_slice())
    }

    pub fn from_directions<T>(loc: &Located<T>, dirs: &Vec<Direction>) -> Path {
        let mut path: Vec<Located<Direction>> = Vec::new();

        let mut pos_x = loc.x;
        let mut pos_y = loc.y;

        if let Some(dir) = dirs.first() {
            path.push(Located {
                x: pos_x,
                y: pos_y,
                value: dir.clone(),
            });
        }

        for dir in dirs {
            dir.adjust_coord(&mut pos_x, &mut pos_y);

            path.push(Located {
                x: pos_x,
                y: pos_y,
                value: dir.clone(),
            });
        }

        Path { steps: path }
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
