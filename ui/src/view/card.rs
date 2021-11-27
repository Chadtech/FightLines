use crate::style::Style;
use crate::view::cell::{Cell, Row};

pub struct Card {
    disabled: bool,
}

impl Card {
    pub fn init() -> Card {
        Card { disabled: false }
    }

    pub fn disable(mut self, d: bool) -> Card {
        self.disabled = d;
        self
    }

    pub fn cell<Msg: 'static>(self, styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        let bg_color = if self.disabled {
            Style::BgContent0
        } else {
            Style::BgContent1
        };

        Cell::from_rows(
            vec![
                bg_color,
                Style::Batch(styles),
                Style::Batch(STYLES.to_vec()),
            ],
            rows,
        )
    }
    pub fn cell_from_rows<Msg: 'static>(styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        Card::init().cell(styles, rows)
    }
}

const STYLES: [Style; 2] = [Style::P4, Style::Outset];
