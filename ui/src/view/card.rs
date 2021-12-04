use crate::style::Style;
use crate::view::cell::{Cell, Row};

pub struct Card {
    disabled: bool,
    primary: bool,
}

impl Card {
    pub fn init() -> Card {
        Card {
            disabled: false,
            primary: false,
        }
    }

    pub fn disable(mut self, d: bool) -> Card {
        self.disabled = d;
        self
    }

    pub fn primary(mut self, p: bool) -> Card {
        self.primary = p;
        self
    }

    pub fn cell<Msg: 'static>(self, extra_styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        let mut styles = vec![Style::Batch(extra_styles), Style::Batch(STYLES.to_vec())];

        let bg_color = if self.disabled {
            Style::BgContent0
        } else {
            Style::BgContent1
        };

        if self.primary {
            styles.append(&mut vec![Style::OutsetImportant, Style::BgImportant1]);
        }

        styles.push(bg_color);

        Cell::from_rows(styles, rows)
    }
    pub fn cell_from_rows<Msg: 'static>(styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        Card::init().cell(styles, rows)
    }
}

const STYLES: [Style; 2] = [Style::P4, Style::Outset];
