use crate::style::Style;
use crate::view::cell::{Cell, Row};

pub struct Card;

impl Card {
    pub fn styles() -> Style {
        Style::Batch(vec![Style::BgContent1, Style::P4, Style::Inset])
    }
    pub fn row<Msg: 'static>(mut styles: Vec<Style>, cells: Vec<Cell<Msg>>) -> Row<Msg> {
        Row::from_cells(vec![Card::styles(), Style::Batch(styles)], cells)
    }
    pub fn cell<Msg: 'static>(mut styles: Vec<Style>, cells: Vec<Cell<Msg>>) -> Cell<Msg> {
        Cell::group(vec![Card::styles(), Style::Batch(styles)], cells)
    }
}
