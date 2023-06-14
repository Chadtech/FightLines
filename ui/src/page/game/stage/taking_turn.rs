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

    pub fn move_completed(&mut self) {
        self.clear_mode();

        if let Sidebar::UnitSelected(unit_selected_model) = &mut self.sidebar {
            self.sidebar = match &unit_selected_model.from_group {
                None => Sidebar::None,
                Some(group_selected_model) => Sidebar::GroupSelected(group_selected_model.clone()),
            };
        }
    }

    fn clear_mode(&mut self) {
        self.mode = Mode::None;
    }
}
