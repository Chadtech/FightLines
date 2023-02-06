use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::located::Located;

pub enum Action {
    TraveledTo {
        path: Vec<Located<Direction>>,
        arrows: Vec<(Direction, Arrow)>,
    },
}
