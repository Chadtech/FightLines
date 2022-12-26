use crate::style::Style;
use crate::view::cell::{Cell, Row};

pub struct Card {
    variant: Variant,
    body_styles: Vec<Style>,
    header: Option<Header>,
}

enum Variant {
    Normal,
    Primary,
    Problem,
}

pub struct Header {
    title: String,
}

impl Header {
    pub fn from_title(title: &str) -> Header {
        Header {
            title: title.to_string(),
        }
    }
    pub fn to_cell<Msg: 'static>(&self) -> Cell<Msg> {
        Cell::from_str(
            vec![Style::BgContent4, Style::M2, Style::TextContent1, Style::P2],
            self.title.as_str(),
        )
    }
}

impl Card {
    pub fn init() -> Card {
        Card {
            variant: Variant::Normal,
            body_styles: Vec::new(),
            header: None,
        }
    }

    pub fn with_header(mut self, header: Header) -> Card {
        self.header = Some(header);
        self
    }

    pub fn problem(self, p: bool) -> Card {
        self.with_variant(Variant::Problem, p)
    }

    pub fn primary(self, p: bool) -> Card {
        self.with_variant(Variant::Primary, p)
    }

    fn with_variant(mut self, variant: Variant, cond: bool) -> Card {
        if cond {
            self.variant = variant;
        }

        self
    }

    pub fn with_body_styles(mut self, styles: Vec<Style>) -> Card {
        self.body_styles = styles;

        self
    }

    pub fn cell<Msg: 'static>(self, extra_styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        let mut styles = vec![Style::Batch(extra_styles), Style::Batch(STYLES.to_vec())];

        let bg_color = match self.variant {
            Variant::Normal => Style::BgContent1,
            Variant::Primary => Style::BgImportant1,
            Variant::Problem => Style::BgProblem1,
        };

        styles.push(bg_color);

        Cell::group(
            styles,
            vec![
                self.header.map(|h| h.to_cell()).unwrap_or_else(Cell::none),
                Cell::from_rows(vec![Style::P4, Style::Batch(self.body_styles)], rows),
            ],
        )
    }
    pub fn cell_from_rows<Msg: 'static>(styles: Vec<Style>, rows: Vec<Row<Msg>>) -> Cell<Msg> {
        Card::init().with_body_styles(styles).cell(vec![], rows)
    }
}

const STYLES: [Style; 2] = [Style::Outset, Style::FlexCol];
