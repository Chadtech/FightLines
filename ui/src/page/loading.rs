use crate::style::Style;
use crate::view::card::Card;
use crate::view::cell::Row;
use crate::view::loading_spinner::LoadingSpinner;

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view<Msg: 'static>() -> Vec<Row<Msg>> {
    let msg = "loading..";

    let card = Card::cell_from_rows(
        vec![Style::G4],
        vec![Row::from_str(msg), LoadingSpinner::row()],
    );

    vec![Row::from_cells(vec![Style::JustifyCenter], vec![card])]
}

pub const PARENT_STYLES: [Style; 2] = [Style::JustifyCenter, Style::G3];
