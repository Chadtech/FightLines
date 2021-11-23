use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::{Cell, Row};

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view<Msg: 'static>() -> Vec<Row<Msg>> {
    vec![
        center(Cell::from_str(
            vec![Style::JustifyCenter],
            "sorry, this page does not seem to exist.",
        )),
        center(
            Button::primary("go to title screen")
                .route(Route::Title)
                .cell(),
        ),
    ]
}

fn center<Msg: 'static>(cells: Cell<Msg>) -> Row<Msg> {
    Row::from_cells(vec![Style::JustifyCenter], vec![cells])
}

pub const PARENT_STYLES: [Style; 2] = [Style::JustifyCenter, Style::G3];
