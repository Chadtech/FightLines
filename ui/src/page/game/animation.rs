use shared::game::outcome::Outcome;
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
    Expired {
        unit_id: UnitId,
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
            Outcome::Expired { unit_id } => Some(Animation::Expired { unit_id }),
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
        }
    }
}
