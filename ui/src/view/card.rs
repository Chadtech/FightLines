use crate::style::Style;
use crate::view::cell::{Cell, Row};

pub struct Card;

impl Card {
    pub fn row<Msg: 'static>(mut styles: Vec<Style>, cells: Vec<Cell<Msg>>) -> Row<Msg> {
        Row::from_cells(
            vec![Style::Batch(STYLES.to_vec()), Style::Batch(styles)],
            cells,
        )
    }
    pub fn cell<Msg: 'static>(mut styles: Vec<Style>, cells: Vec<Cell<Msg>>) -> Cell<Msg> {
        Cell::group(
            vec![Style::Batch(STYLES.to_vec()), Style::Batch(styles)],
            cells,
        )
    }
    pub fn cell_from_rows<Msg: 'static>(mut styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        Cell::from_rows(
            vec![Style::Batch(STYLES.to_vec()), Style::Batch(styles)],
            rows,
        )
    }
}

const STYLES: [Style; 3] = [Style::BgContent1, Style::P4, Style::Outset];
