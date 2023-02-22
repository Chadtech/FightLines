use crate::page::game::animation::Animation;
use shared::facing_direction::FacingDirection;
use shared::game::day::Time;
use shared::game::{calculate_player_visibility, unit_index, Indices};
use shared::id::Id;
use shared::located::Located;
use shared::map::Map;
use shared::unit::place::UnitPlace;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Model {
    pub indices: Indices,
    animations: Vec<Animation>,
    pub visibility: HashSet<Located<()>>,
    pub day: Time,
}

impl Model {
    pub fn init(
        indices: Indices,
        animations: Vec<Animation>,
        visibility: HashSet<Located<()>>,
        day: Time,
    ) -> Model {
        Model {
            indices,
            animations,
            visibility,
            day,
        }
    }
    pub fn progress_animation(&mut self, viewer_id: &Id, map: &Map) -> Result<bool, String> {
        let ret = match self.animations.first_mut() {
            None => Ok(true),
            Some(animation) => match animation {
                Animation::Travel {
                    unit_id,
                    path,
                    loads_into,
                } => {
                    let facing_dir = match self.indices.position_of_unit_or_transport(unit_id) {
                        Ok(facing_dir_loc) => facing_dir_loc.value,
                        Err(msg) => return Err(msg),
                    };

                    let unit = match self.indices.by_id.get_mut(unit_id) {
                        None => return Err("could not find unit".to_string()),
                        Some(unit) => unit,
                    };

                    match path.shift_first() {
                        Some(step) => {
                            let facing_dir = FacingDirection::from_directions(path.to_directions())
                                .unwrap_or(facing_dir);

                            let loc = Located {
                                x: step.x,
                                y: step.y,
                                value: facing_dir,
                            };
                            unit.place = UnitPlace::OnMap(loc);

                            self.indices.by_location =
                                unit_index::by_location::make(&self.indices.by_id);
                        }
                        None => {
                            if let Some(transport_id) = loads_into {
                                unit.place = UnitPlace::InUnit(transport_id.clone());
                            }

                            self.indices.by_location =
                                unit_index::by_location::make(&self.indices.by_id);

                            self.indices.by_transport =
                                unit_index::by_transport::make(&self.indices.by_id);

                            self.animations.remove(0);
                        }
                    }

                    Ok(false)
                }
            },
        };

        self.visibility = calculate_player_visibility(viewer_id, map, &self.indices.by_id);

        ret
    }
}
