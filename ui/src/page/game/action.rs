use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::located::Located;
use shared::unit::UnitId;

pub enum Action {
    TraveledTo {
        path: Vec<Located<Direction>>,
        arrows: Vec<(Direction, Arrow)>,
    },
    LoadInto {
        load_into: UnitId,
    },
}
