use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::{Cell, Row};

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
pub struct Model;

#[derive(Copy, Clone)]
pub enum Msg {}

///////////////////////////////////////////////////////////////
// Init
///////////////////////////////////////////////////////////////

pub fn init() -> Model {
    Model
}

///////////////////////////////////////////////////////////////
// Update
///////////////////////////////////////////////////////////////

pub fn update(_msg: Msg, _model: &mut Model) {}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(_model: &Model) -> Vec<Row<Msg>> {
    // vec![text("Fightlines")]
    vec![
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Cell::from_str(vec![Style::JustifyCenter], "Fightlines")],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::simple("New Game").to_cell()],
        ),
    ]
}

pub fn parent_styles() -> Vec<Style> {
    vec![Style::JustifyCenter, Style::G3]
}
