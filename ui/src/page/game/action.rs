use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::path::Path;
use shared::unit::UnitId;

#[derive(Clone, Debug)]
pub enum Action {
    TraveledTo {
        unit_id: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
    LoadInto {
        unit_id: UnitId,
        load_into: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
}
