use crate::style::Style;
use crate::view::cell::{Cell, Row};

pub fn view<Msg: 'static>(
    maybe_header_text: Option<&str>,
    maybe_descript: Option<&str>,
) -> Vec<Row<Msg>> {
    let header_row = match maybe_header_text {
        None => Row::none(),
        Some(header_text) => Row::from_cells(
            vec![],
            vec![Cell::from_str(vec![Style::TextContent5], header_text)],
        ),
    };

    let descript_row = match maybe_descript {
        None => Row::none(),
        Some(descript_text) => Row::from_cells(vec![], vec![Cell::from_str(vec![], descript_text)]),
    };

    vec![header_row, descript_row]
}
