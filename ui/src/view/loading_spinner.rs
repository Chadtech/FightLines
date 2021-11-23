use crate::style::Style;
use crate::view::cell::{Cell, Row};
use seed::dom_entity_names::Tag;
use seed::prelude::{El, Node};
use std::borrow::Cow;

pub struct LoadingSpinner;

impl LoadingSpinner {
    pub fn row<Msg: 'static>() -> Row<Msg> {
        Row::from_cells(vec![], vec![LoadingSpinner::cell()])
    }

    pub fn cell<Msg: 'static>() -> Cell<Msg> {
        Cell::group(
            STYLES.to_vec(),
            vec![Cell::group(SPINNER_STYLES.to_vec(), vec![]).with_tag_name(TAG_NAME)],
        )
    }
}

const TAG_NAME: &'static str = "spinner";

const SPINNER_STYLES: [Style; 4] = [Style::BgContent4, Style::Absolute, Style::W7, HEIGHT_STYLE];

const STYLES: [Style; 6] = [
    Style::Inset,
    // These sizes are tied to the sizes in the spinners css animation.
    // So if you need to change them here, change the ones in the "slideBy"
    // animation as well
    Style::W8,
    HEIGHT_STYLE,
    Style::BgBackground0,
    Style::Relative,
    Style::OverflowHidden,
];

const HEIGHT_STYLE: Style = Style::H5;
