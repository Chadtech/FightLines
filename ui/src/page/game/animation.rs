use shared::facing_direction::FacingDirection;
use shared::game::outcome::Outcome;
use shared::game::unit_index;
use shared::id::Id;
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
    fn subject_unit_id(self) -> UnitId {
        match self {
            Animation::Travel { unit_id, .. } => unit_id,
            Animation::Perish { unit_id } => unit_id,
            Animation::DropOff { transport_id, .. } => transport_id,
            Animation::Replenish {
                replenishing_unit_id,
                ..
            } => replenishing_unit_id,
        }
    }
    pub fn sidebar_can_list_for_user(
        self,
        player_id: Id,
        unit_index: &unit_index::by_id::Index,
    ) -> Result<bool, String> {
        let unit_id = self.subject_unit_id();

        match unit_index.get(&unit_id) {
            Some(unit_model) => Ok(unit_model.owner == player_id),
            None => Err("could not find unit".to_string()),
        }
    }
    pub fn from_outcome(outcome: Outcome) -> Vec<Animation> {
        match outcome {
            Outcome::Traveled { unit_id, path, .. } => vec![Animation::Travel {
                unit_id,
                path,
                loads_into: None,
                picks_up: None,
            }],
            Outcome::LoadedInto {
                unit_id,
                path,
                loaded_into,
                ..
            } => vec![Animation::Travel {
                unit_id,
                path,
                loads_into: Some(loaded_into),
                picks_up: None,
            }],
            Outcome::NamedUnit { .. } => vec![],
            Outcome::Perished { unit_id } => vec![Animation::Perish { unit_id }],
            Outcome::ConsumedSupplies { .. } => vec![],
            Outcome::PickUp {
                unit_id,
                path,
                cargo_id,
            } => vec![Animation::Travel {
                unit_id,
                path,
                loads_into: None,
                picks_up: Some(cargo_id),
            }],
            Outcome::Placed {
                cargo_unit_loc: loc,
                transport_id,
            } => vec![Animation::DropOff {
                cargo_unit: loc,
                transport_id,
            }],
            Outcome::Replenished {
                replenishing_unit_id,
                units,
                path,
                ..
            } => vec![
                Animation::Travel {
                    unit_id: replenishing_unit_id.clone(),
                    path,
                    loads_into: None,
                    picks_up: None,
                },
                Animation::Replenish {
                    units: units
                        .iter()
                        .map(|unit| unit.0.clone())
                        .collect::<Vec<UnitId>>(),
                    replenishing_unit_id,
                },
            ],
        }
    }
}
