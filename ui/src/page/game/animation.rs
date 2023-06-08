use shared::facing_direction::FacingDirection;
use shared::game::outcome::Outcome;
use shared::located::Located;
use shared::path::Path;
use shared::unit::UnitId;

#[derive(Clone, Debug)]
pub enum Animation {
    Travel {
        unit_id: UnitId,
        path: Path,
        loads_into: Option<UnitId>,
        picks_up: Option<UnitId>,
    },
    Perish {
        unit_id: UnitId,
    },
    DropOff {
        cargo_unit: Located<(FacingDirection, UnitId)>,
        transport_id: UnitId,
    },
    Replenish {
        replenishing_unit_id: UnitId,
        units: Vec<UnitId>,
    },
}

impl Animation {
    pub fn from_outcome(outcome: Outcome) -> Option<Animation> {
        match outcome {
            Outcome::Traveled { unit_id, path, .. } => Some(Animation::Travel {
                unit_id,
                path,
                loads_into: None,
                picks_up: None,
            }),
            Outcome::LoadedInto {
                unit_id,
                path,
                loaded_into,
                ..
            } => Some(Animation::Travel {
                unit_id,
                path,
                loads_into: Some(loaded_into),
                picks_up: None,
            }),
            Outcome::NamedUnit { .. } => None,
            Outcome::Perished { unit_id } => Some(Animation::Perish { unit_id }),
            Outcome::ConsumedSupplies { .. } => None,
            Outcome::PickUp {
                unit_id,
                path,
                cargo_id,
            } => Some(Animation::Travel {
                unit_id,
                path,
                loads_into: None,
                picks_up: Some(cargo_id),
            }),
            Outcome::Placed {
                cargo_unit_loc: loc,
                transport_id,
            } => Some(Animation::DropOff {
                cargo_unit: loc,
                transport_id,
            }),
            Outcome::Replenished {
                replenishing_unit_id,
                units,
                ..
            } => Some(Animation::Replenish {
                units: units
                    .iter()
                    .map(|unit| unit.0.clone())
                    .collect::<Vec<UnitId>>(),
                replenishing_unit_id,
            }),
        }
    }
}
