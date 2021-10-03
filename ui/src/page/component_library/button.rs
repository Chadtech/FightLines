use crate::page::component_library::section;
use crate::view::button::Button;
use crate::view::cell::Row;

pub fn view<Msg: 'static + Clone>() -> Vec<Row<Msg>> {
    let row = |button: Button<Msg>| Row::from_cells(vec![], vec![button.to_cell()]);

    vec![
        vec![row(Button::simple("button"))],
        section::view(
                Some("active"),
                None
        ),
        vec![row(Button::simple("active").active(true))],
        section::view(
                Some("primary"),
                Some(r#"
                    The button type for when there is a singular button that represents the primary intent of a view
                "#)
        ),
        vec![row(Button::primary("primary"))]
    ]
    .concat()
}
