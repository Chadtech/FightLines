use crate::page::game::action::Action;
use crate::page::game::view::unit_row;
use crate::style::Style;
use crate::view::cell::Cell;
use shared::game::Game;
use shared::unit::UnitId;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Model {
    pub units: Vec<UnitId>,
}

impl Model {
    pub fn init(units: Vec<UnitId>) -> Model {
        Model { units }
    }
}

#[derive(Clone, Debug)]
pub enum Msg {
    UnitRow(unit_row::Msg),
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn sidebar_content(
    model: &Model,
    moves_index: &HashMap<UnitId, Action>,
    game: &Game,
) -> Vec<Cell<Msg>> {
    let mut unit_rows = vec![];

    for unit_id in &model.units {
        if let Some(unit_model) = game.get_unit(unit_id) {
            unit_rows.push(unit_row::view(unit_id, unit_model, moves_index).map_msg(Msg::UnitRow));
        }
    }

    vec![Cell::group(
        vec![Style::FlexCol, Style::Inset, Style::BgBackground1],
        unit_rows,
    )]
}
