use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::Cell;
use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::facing_direction::FacingDirection;
use shared::game::unit_index;
use shared::located::Located;
use shared::path::Path;
use shared::point::Point;
use shared::tile;
use shared::unit::UnitId;
use std::collections::HashSet;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Model {
    pub unit_id: UnitId,
    pub mobility: HashSet<Located<()>>,
    pub arrows: Vec<ArrowStep>,
    pub ride_options: Option<Located<RideOptionsModel>>,
}

#[derive(Debug, Clone)]
pub struct ArrowStep {
    pub direction: Direction,
    pub arrow: Arrow,
}

impl From<&ArrowStep> for (Direction, Arrow) {
    fn from(step: &ArrowStep) -> Self {
        (step.direction.clone(), step.arrow.clone())
    }
}

impl Model {
    pub fn init(unit_id: UnitId, mobility: HashSet<Located<()>>) -> Model {
        Model {
            unit_id,
            mobility,
            arrows: Vec::new(),
            ride_options: None,
        }
    }

    pub fn with_options(
        &mut self,
        x: u16,
        y: u16,
        options: Vec<RideOption>,
        path: Path,
    ) -> &mut Model {
        let options_model = RideOptionsModel {
            ride_options: options,
            path,
        };

        self.ride_options = Some(Located {
            x,
            y,
            value: options_model,
        });

        self
    }

    pub fn path(
        &self,
        unit_id: &UnitId,
        unit_indexes: &unit_index::Indexes,
    ) -> Result<Path, String> {
        let loc = unit_indexes.position_of_unit_or_transport(unit_id)?;

        let path = Path::from_directions::<FacingDirection>(
            &loc,
            &self
                .arrows
                .clone()
                .into_iter()
                .map(|ArrowStep { direction, .. }| direction)
                .collect::<Vec<Direction>>(),
        );

        Ok(path)
    }
}

#[derive(Debug, Clone)]
pub struct RideOptionsModel {
    pub ride_options: Vec<RideOption>,
    pub path: Path,
}

#[derive(Debug, Clone)]

pub enum RideOption {
    LoadInto { unit_id: UnitId, unit_label: String },
    PickUp { unit_id: UnitId, unit_label: String },
    Replenish { only_self: bool },
    Attack,
}

impl RideOption {
    pub fn load_into(unit_id: UnitId, label: String) -> RideOption {
        RideOption::LoadInto {
            unit_id,
            unit_label: label,
        }
    }

    pub fn pick_up(unit_id: UnitId, label: String) -> RideOption {
        RideOption::PickUp {
            unit_id,
            unit_label: label,
        }
    }

    pub fn label(&self) -> String {
        match self {
            RideOption::LoadInto {
                unit_label: label, ..
            } => {
                format!("load into {}", label)
            }
            RideOption::PickUp { unit_label, .. } => {
                format!("pick up {}", unit_label)
            }
            RideOption::Replenish { only_self } => {
                if *only_self {
                    "replenish self".to_string()
                } else {
                    "replenish units".to_string()
                }
            }
            RideOption::Attack => "attack".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ClickMsg {
    LoadInto(UnitId),
    PickUp(UnitId),
    MoveTo,
    Replenish,
    Attack,
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn flyout_view(model: &Model, game_screen_pos: &Point<i16>) -> Cell<ClickMsg> {
    match &model.ride_options {
        None => Cell::none(),
        Some(loc_ride_options) => {
            let screen_x = {
                let game_pos_px = loc_ride_options.x * tile::PIXEL_WIDTH * 2;

                game_screen_pos.x + (game_pos_px as i16)
            };

            let screen_y = {
                let game_pos_px = (loc_ride_options.y + 1) * tile::PIXEL_HEIGHT * 2;

                game_screen_pos.y + (game_pos_px as i16) + 1
            };

            let mut move_buttons = vec![];

            move_buttons.push(Button::simple("move to").on_click(|_| ClickMsg::MoveTo));

            for ride_option in &loc_ride_options.value.ride_options {
                let msg = match ride_option {
                    RideOption::LoadInto { unit_id, .. } => ClickMsg::LoadInto(unit_id.clone()),
                    RideOption::PickUp { unit_id, .. } => ClickMsg::PickUp(unit_id.clone()),
                    RideOption::Replenish { .. } => ClickMsg::Replenish,
                    RideOption::Attack => ClickMsg::Attack,
                };

                let button = Button::simple(ride_option.label().as_str())
                    .on_click(|_| msg)
                    .full_width();

                move_buttons.push(button)
            }

            Cell::group(
                vec![
                    Style::Outset,
                    Style::BgContent1,
                    Style::P2,
                    Style::FlexCol,
                    Style::G3,
                    Style::W8,
                ],
                move_buttons
                    .into_iter()
                    .map(|button| button.full_width().cell())
                    .collect::<Vec<Cell<ClickMsg>>>(),
            )
            .at_screen_pos(Point {
                x: screen_x,
                y: screen_y,
            })
        }
    }
}
