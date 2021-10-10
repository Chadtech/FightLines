mod button;
mod section;

use crate::route;
use crate::route::component_library::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::{Cell, Row};

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    active_section: Section,
}

#[derive(PartialEq, Clone)]
enum Section {
    Button,
}

///////////////////////////////////////////////////////////////
// Init //
///////////////////////////////////////////////////////////////

pub fn init(route: route::component_library::Route) -> Model {
    let active_section = match route {
        Route::Button => Section::Button,
    };

    Model { active_section }
}

///////////////////////////////////////////////////////////////
// Helpers //
///////////////////////////////////////////////////////////////

const ALL_SECTIONS: &[Section] = &[Section::Button];

impl Section {
    fn to_label(&self) -> &'static str {
        match self {
            Section::Button => "button",
        }
    }
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn view<Msg: 'static + Clone>(model: &Model) -> Vec<Row<Msg>> {
    vec![Row::from_cells(
        vec![Style::Grow],
        vec![
            Cell::from_rows(vec![Style::BorderR, Style::P3, Style::G3], nav_bar(model)),
            Cell::from_rows(vec![Style::P3, Style::G3], main_area(model)),
        ],
    )]
}

fn nav_bar<Msg: 'static>(model: &Model) -> Vec<Row<Msg>> {
    let mut ret_rows = Vec::new();

    let header_row = Row::from_cells(vec![], vec![Cell::from_str(vec![], "Component Library")]);

    ret_rows.push(header_row);

    for section in ALL_SECTIONS {
        let section_row = Row::from_cells(
            vec![],
            vec![Button::simple(section.to_label())
                .active(section.clone() == model.active_section)
                .cell()],
        );

        ret_rows.push(section_row);
    }

    ret_rows
}

fn main_area<Msg: 'static + Clone>(model: &Model) -> Vec<Row<Msg>> {
    match model.active_section {
        Section::Button => button::view(),
    }
}
