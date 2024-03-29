use crate::page::game::action::Action;
use crate::page::game::group_selected;
use crate::page::game::unit_change::UnitChange;
use crate::page::game::view::unit_row;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::Cell;
use crate::view::text_field::TextField;
use shared::game::Game;
use shared::unit::UnitId;
use shared::{game, unit};
use std::collections::HashMap;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Model {
    pub unit_id: UnitId,
    pub name_field: String,
    pub name_submitted: bool,
    pub from_group: Option<group_selected::Model>,
}

pub struct Flags {
    pub unit_id: UnitId,
    pub existing_name: Option<String>,
    pub name_already_submitted: bool,
    pub from_group: Option<group_selected::Model>,
}

impl From<Flags> for Model {
    fn from(flags: Flags) -> Self {
        Model {
            unit_id: flags.unit_id,
            name_field: flags.existing_name.unwrap_or_default(),
            name_submitted: flags.name_already_submitted,
            from_group: flags.from_group,
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
    unit_model: &unit::Model,
    moves_index: &HashMap<UnitId, Action>,
    unit_changes: &HashMap<UnitId, UnitChange>,
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
        Cell::from_str(vec![], "unload")
    } else {
        Cell::none()
    };

    let transporting_view = match maybe_loaded_units {
        Some(loaded_units) => {
            let mut unit_rows = Vec::new();

            for (unit_id, _) in loaded_units {
                if let Some(unit_model) = game.get_unit(unit_id) {
                    unit_rows.push(
                        unit_row::view(unit_id, unit_model, moves_index, unit_changes)
                            .map_msg(Msg::UnitRow),
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

    let supplies_label = Cell::from_str(vec![], "supplies");
    let supply_view = {
        let supply_block_num: u16 = {
            let percent_of_max: f32 = if unit_model.supplies > 0 {
                ((unit_model.supplies as f32) / (unit_model.unit.max_supplies() as f32)) * 16.0
            } else {
                0.0
            };

            percent_of_max.ceil() as u16
        };

        let supply_bar_width = match supply_block_num {
            0 => Style::W0D16,
            1 => Style::W1D16,
            2 => Style::W2D16,
            3 => Style::W3D16,
            4 => Style::W4D16,
            5 => Style::W5D16,
            6 => Style::W6D16,
            7 => Style::W7D16,
            8 => Style::W8D16,
            9 => Style::W9D16,
            10 => Style::W10D16,
            11 => Style::W11D16,
            12 => Style::W12D16,
            13 => Style::W13D16,
            14 => Style::W14D16,
            15 => Style::W15D16,
            16 => Style::W16D16,
            _ => Style::W0D16,
        };

        let supply_block_color = if supply_block_num < 5 {
            Style::BgProblem5
        } else if supply_block_num < 9 {
            Style::BgImportant4
        } else {
            Style::BgContent4
        };

        let supply_block = Cell::empty(vec![supply_bar_width, Style::H4, supply_block_color]);

        Cell::group(
            vec![
                Style::FlexRow,
                Style::Inset,
                Style::BgBackground1,
                Style::G3,
                Style::P2,
                Style::Relative,
            ],
            vec![supply_block],
        )
    };

    vec![
        back_button_row,
        name_view,
        transporting_label,
        transporting_view,
        supplies_label,
        supply_view,
    ]
}
