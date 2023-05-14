use shared::game::outcome::Outcome;
use shared::path::Path;
use shared::unit::UnitId;

#[derive(Clone, Debug)]
pub enum Animation {
    Travel {
        unit_id: UnitId,
        path: Path,
        loads_into: Option<UnitId>,
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
            }),
            Outcome::NamedUnit { .. } => None,
            Outcome::Expired { unit_id } => Some(Animation::Expired { unit_id }),
            Outcome::ConsumedSupplies { .. } => None,
        }
    }
}
