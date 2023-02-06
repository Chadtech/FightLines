use crate::page::game::group_selected;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::Cell;
use crate::view::text_field::TextField;
use shared::game::UnitModel;
use shared::unit::UnitId;

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
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn sidebar_content(model: &Model, unit_model: &UnitModel) -> Vec<Cell<Msg>> {
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

    vec![back_button_row, name_view]
}
