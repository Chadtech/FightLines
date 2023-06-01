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
        dismounted_from: Option<UnitId>,
    },
    LoadInto {
        unit_id: UnitId,
        load_into: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
    PickUp {
        unit_id: UnitId,
        cargo_id: UnitId,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
}
