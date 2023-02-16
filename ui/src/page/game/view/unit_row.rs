use crate::page::game::action::Action;
use crate::style::Style;
use crate::view::cell::Cell;
use shared::api::endpoint::Endpoint;
use shared::game::UnitModel;
use shared::unit::UnitId;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum Msg {
    Clicked(UnitId),
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn view(
    unit_id: &UnitId,
    unit_model: &UnitModel,
    moves_index: &HashMap<UnitId, Action>,
) -> Cell<Msg> {
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

    Cell::group(
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
                    Cell::empty(vec![Style::W5, Style::H5, Style::BgBackground4]).with_img_src(
                        Endpoint::ThumbnailAsset(unit_model.unit.clone(), unit_model.color.clone())
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
    .on_click(|_| Msg::Clicked(clicked_unit_id))
}
