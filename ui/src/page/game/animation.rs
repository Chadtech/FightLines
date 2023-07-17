use shared::facing_direction::FacingDirection;
use shared::game::event::Event;
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
        cargo_id: UnitId,
        cargo_loc: Located<FacingDirection>,
        transport_id: UnitId,
    },
    Replenish {
        replenishing_unit_id: UnitId,
        units: Vec<UnitId>,
    },
}

impl Animation {
    pub fn subject_unit_id(&self) -> &UnitId {
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
    pub fn moving_subject_unit_id(&self) -> Option<Located<&UnitId>> {
        match self {
            Animation::Travel { unit_id, path, .. } => {
                path.first_pos().map(|loc| loc.with_value(unit_id))
            }
            _ => None,
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

    pub fn from_event(event: Event) -> Vec<Animation> {
        match event {
            Event::ConsumedBaselineSupplies { .. } => {
                vec![]
            }
            Event::Travelled { unit_id, path, .. } => vec![Animation::Travel {
                unit_id,
                path,
                loads_into: None,
                picks_up: None,
            }],
            Event::Loaded {
                cargo_id,
                transport_id,
                path,
                ..
            } => vec![Animation::Travel {
                unit_id: cargo_id,
                path,
                loads_into: Some(transport_id),
                picks_up: None,
            }],
            Event::PickedUp {
                cargo_id,
                transport_id,
                path,
                ..
            } => vec![Animation::Travel {
                unit_id: transport_id,
                path,
                loads_into: None,
                picks_up: Some(cargo_id),
            }],
            Event::DroppedOff {
                cargo_id,
                transport_id,
                cargo_loc,
            } => vec![Animation::DropOff {
                cargo_id,
                transport_id,
                cargo_loc,
            }],
            Event::ReplenishedUnits {
                unit_id,
                replenished_units,
                path,
            } => {
                vec![
                    Animation::Travel {
                        unit_id: unit_id.clone(),
                        path,
                        loads_into: None,
                        picks_up: None,
                    },
                    Animation::Replenish {
                        replenishing_unit_id: unit_id,
                        units: replenished_units,
                    },
                ]
            }
            Event::WasReplenished { .. } => vec![],
            Event::DepletedCrate { .. } => vec![],
            Event::Perished { unit_id } => {
                vec![Animation::Perish { unit_id }]
            }
        }
    }
}
