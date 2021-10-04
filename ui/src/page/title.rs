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
    vec![
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Cell::from_str(vec![Style::JustifyCenter], "Fightlines")],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::primary("start game")
                .full_width()
                .to_cell()
                .with_styles(vec![Style::W8])],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::simple("join game")
                .full_width()
                .to_cell()
                .with_styles(vec![Style::W8])],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::simple("custom game")
                .full_width()
                .to_cell()
                .with_styles(vec![Style::W8])],
        ),
    ]
}

pub fn parent_styles() -> Vec<Style> {
    vec![Style::JustifyCenter, GAP_SIZE]
}

const GAP_SIZE: Style = Style::G3;
