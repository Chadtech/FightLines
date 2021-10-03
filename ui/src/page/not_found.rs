use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::{Cell, Row};

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view<Msg: 'static>() -> Vec<Row<Msg>> {
    vec![
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Cell::from_str(
                vec![Style::JustifyCenter],
                "Sorry, this page does not seem to exist.",
            )],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::primary("go to title screen")
                .route(Route::Title)
                .to_cell()],
        ),
    ]
}

pub fn parent_styles() -> Vec<Style> {
    vec![Style::JustifyCenter, Style::G3]
}
