use crate::page::game::action::Action;
use crate::page::game::group_selected;
use crate::page::game::view::unit_row;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::Cell;
use crate::view::text_field::TextField;
use shared::game;
use shared::game::{Game, UnitModel};
use shared::unit::UnitId;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////
pub struct Model {
    pub unit_id: UnitId,
    pub name_field: String,
    pub name_submitted: bool,
    pub from_group: Option<group_selected::Model>,
}

impl Model {
    pub fn init(unit_id: UnitId, from_group: Option<group_selected::Model>) -> Model {
        Model {
            unit_id,
            name_field: String::new(),
            name_submitted: false,
            from_group,
        }
    }
}

#[derive(Clone, Debug)]

pub enum Msg {
    UpdatedUnitNameField(String),
    ClickedSetName,
    ClickedBackToGroup,
    UnitRow(unit_row::Msg),
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn sidebar_content(
    model: &Model,
    transport_index: &game::unit_index::by_transport::Index,
    unit_model: &UnitModel,
    moves_index: &HashMap<UnitId, Action>,
    game: &Game,
) -> Vec<Cell<Msg>> {
    let back_button_row = match model.from_group {
        None => Cell::none(),
        Some(_) => Button::simple("back to group")
            .on_click(|_| Msg::ClickedBackToGroup)
            .cell(),
    };
    let name_view = match &unit_model.name {
        Some(name) => Cell::from_str(vec![], name.as_str()),
        None => {
            let save_name_button = Button::simple("save")
                .on_click(|_| Msg::ClickedSetName)
                .disable(model.name_submitted)
                .cell();

            Cell::group(
                vec![Style::FlexRow, Style::G4],
                vec![
                    TextField::simple(model.name_field.as_str(), Msg::UpdatedUnitNameField)
                        .with_placeholder("unit name".to_string())
                        .cell(),
                    save_name_button,
                ],
            )
        }
    };

    let maybe_loaded_units = transport_index.get(&model.unit_id);

    let transporting_label = if maybe_loaded_units.is_some() {
        Cell::from_str(vec![], "deploy")
    } else {
        Cell::none()
    };

    let transporting_view = match maybe_loaded_units {
        Some(loaded_units) => {
            let mut unit_rows = Vec::new();

            for (unit_id, _) in loaded_units {
                if let Some(unit_model) = game.units.get(unit_id) {
                    unit_rows.push(
                        unit_row::view(unit_id, unit_model, moves_index).map_msg(Msg::UnitRow),
                    );
                }
            }
            Cell::group(
                vec![Style::FlexCol, Style::Inset, Style::BgBackground1],
                unit_rows,
            )
        }
        None => Cell::none(),
    };

    vec![
        back_button_row,
        name_view,
        transporting_label,
        transporting_view,
    ]
}
