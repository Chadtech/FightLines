use crate::page::game::animation::Animation;
use crate::view::cell::Cell;
use shared::facing_direction::FacingDirection;
use shared::game::day::Time;
use shared::game::unit_index::Indices;
use shared::game::{calculate_player_visibility, unit_index};
use shared::id::Id;
use shared::located::Located;
use shared::map::Map;
use shared::unit::Place;
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
                    picks_up,
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
                                value: step.value.to_facing_dir().unwrap_or(facing_dir),
                            };
                            unit.place = Place::OnMap(loc);

                            self.indices.by_location =
                                unit_index::by_location::make(&self.indices.by_id);
                        }
                        None => {
                            if let Some(transport_id) = loads_into {
                                unit.place = Place::InUnit(transport_id.clone());
                            }

                            if let Some(cargo_id) = picks_up {
                                let cargo = match self.indices.by_id.get_mut(cargo_id) {
                                    None => return Err("could not find cargo unit".to_string()),
                                    Some(u) => u,
                                };

                                cargo.place = Place::InUnit(unit_id.clone());
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
                Animation::Perish { .. } => {
                    self.animations.remove(0);

                    // Animate!

                    Ok(false)
                }
                Animation::DropOff {
                    cargo_unit: loc, ..
                } => {
                    let (facing_dir, cargo_id) = loc.value.clone();

                    let cargo_unit_model = match self.indices.by_id.get_mut(&cargo_id) {
                        Some(u) => u,
                        None => {
                            return Err("could not find cargo unit".to_string());
                        }
                    };

                    cargo_unit_model.place = Place::OnMap(loc.with_value(facing_dir));

                    self.indices.by_location = unit_index::by_location::make(&self.indices.by_id);

                    self.indices.by_transport = unit_index::by_transport::make(&self.indices.by_id);

                    self.animations.remove(0);

                    Ok(false)
                }
            },
        };

        self.visibility = calculate_player_visibility(viewer_id, map, &self.indices.by_id);

        ret
    }
}

pub fn sidebar_view<Msg: 'static>(
    unit_index: &unit_index::by_id::Index,
    model: &Model,
) -> Vec<Cell<Msg>> {
    match model.animations.first() {
        None => vec![],
        Some(current_animation) => match current_animation {
            Animation::Travel { unit_id, .. } => {
                let msg = match unit_index.get(unit_id) {
                    None => "error: could not find unit".to_string(),
                    Some(unit_model) => {
                        let mut unit_name_msg = unit_model
                            .name
                            .clone()
                            .unwrap_or_else(|| unit_model.unit.to_string());

                        unit_name_msg.push_str(" moved");

                        unit_name_msg
                    }
                };

                vec![Cell::from_str(vec![], msg.as_str())]
            }
            Animation::Perish { unit_id } => {
                let msg = match unit_index.get(unit_id) {
                    None => "error: could not find unit".to_string(),
                    Some(unit_model) => {
                        let mut unit_name_msg = unit_model
                            .name
                            .clone()
                            .unwrap_or_else(|| unit_model.unit.to_string());

                        unit_name_msg.push_str(" perished");

                        unit_name_msg
                    }
                };

                vec![Cell::from_str(vec![], msg.as_str())]
            }
            Animation::DropOff {
                cargo_unit: loc,
                transport_id,
            } => {
                let cargo_id = &loc.value.1;

                let msg = match (unit_index.get(transport_id), unit_index.get(cargo_id)) {
                    (Some(transport), Some(cargo)) => {
                        let mut dropoff_msg = transport
                            .name
                            .clone()
                            .unwrap_or_else(|| transport.unit.to_string());

                        dropoff_msg.push_str(" dropped off ");

                        dropoff_msg.push_str(
                            cargo
                                .name
                                .clone()
                                .unwrap_or_else(|| cargo.unit.to_string())
                                .as_str(),
                        );

                        dropoff_msg
                    }
                    (None, _) => "error: could not find transport unit".to_string(),
                    (_, None) => "error: could not find cargo unit".to_string(),
                };

                vec![Cell::from_str(vec![], msg.as_str())]
            }
        },
    }
}
