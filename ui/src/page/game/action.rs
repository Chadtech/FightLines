use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::facing_direction::FacingDirection;
use shared::located::Located;
use shared::path::Path;
use shared::unit::UnitId;

#[derive(Clone, Debug, PartialEq, Eq)]
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
    DropOff {
        cargo_unit_loc: Located<(FacingDirection, UnitId)>,
        transport_id: UnitId,
    },
    Replenish {
        replenishing_unit_id: UnitId,
        units: Vec<(UnitId, i16)>,
        depleted_supply_crates: Vec<(UnitId, i16)>,
        path: Path,
        arrows: Vec<(Direction, Arrow)>,
    },
}
