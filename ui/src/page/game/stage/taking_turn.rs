use crate::page::game::mode::Mode;
use crate::page::game::{group_selected, unit_selected};

#[derive(Debug)]
pub struct Model {
    pub mode: Mode,
    pub sidebar: Sidebar,
}

#[derive(Debug)]
pub enum Sidebar {
    None,
    UnitSelected(unit_selected::Model),
    GroupSelected(group_selected::Model),
}

impl Model {
    pub fn init() -> Model {
        Model {
            mode: Mode::None,
            sidebar: Sidebar::None,
        }
    }

    pub fn clear_mode(&mut self) {
        self.mode = Mode::None;
    }
}
