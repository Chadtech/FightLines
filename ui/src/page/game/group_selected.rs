use crate::page::game::action::Action;
use crate::style::Style;
use crate::view::cell::Cell;
use shared::api::endpoint::Endpoint;
use shared::game::Game;
use shared::unit::UnitId;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Model {
    pub units: Vec<UnitId>,
}

impl Model {
    pub fn init(units: Vec<UnitId>) -> Model {
        Model { units }
    }
}

#[derive(Clone, Debug)]

pub enum Msg {
    ClickedUnitInGroup(UnitId),
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn sidebar_content(
    model: &Model,
    moves_index: &HashMap<UnitId, Action>,
    game: &Game,
) -> Vec<Cell<Msg>> {
    let mut unit_rows = vec![];

    for unit_id in &model.units {
        if let Some(unit_model) = game.units.get(unit_id) {
            let label = unit_model
                .name
                .as_ref()
                .cloned()
                .unwrap_or_else(|| unit_model.unit.to_string());

            let clicked_unit_id = unit_id.clone();

            let unit_moved = moves_index.get(unit_id).is_some();

            let text_color = if unit_moved {
                Style::TextContent2
            } else {
                Style::none()
            };

            let background_color_hover = if unit_moved {
                Style::none()
            } else {
                Style::BgBackground4Hover
            };

            let unit_row = Cell::group(
                vec![
                    Style::CursorPointer,
                    background_color_hover,
                    Style::P4,
                    Style::G4,
                    Style::FlexRow,
                ],
                vec![
                    Cell::group(
                        vec![Style::FlexCol, Style::JustifyCenter],
                        vec![
                            Cell::empty(vec![Style::W5, Style::H5, Style::BgBackground4])
                                .with_img_src(
                                    Endpoint::ThumbnailAsset(
                                        unit_model.unit.clone(),
                                        unit_model.color.clone(),
                                    )
                                    .to_string(),
                                ),
                        ],
                    ),
                    Cell::from_str(
                        vec![
                            text_color,
                            Style::TextSelectNone,
                            Style::FlexCol,
                            Style::JustifyCenter,
                        ],
                        label.as_str(),
                    ),
                ],
            )
            .on_click(|_| Msg::ClickedUnitInGroup(clicked_unit_id));

            unit_rows.push(unit_row);
        }
    }

    vec![Cell::group(
        vec![Style::FlexCol, Style::Inset, Style::BgBackground1],
        unit_rows,
    )]
}
