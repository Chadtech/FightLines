use crate::global;
use crate::style::Style;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use seed::log;
use seed::prelude::Orders;
use shared::id::Id;
use shared::lobby::{Lobby, MAX_GUESTS};
use shared::player::Player;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    lobby_id: Id,
    lobby: Lobby,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Msg,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub lobby_id: Id,
    pub lobby: Lobby,
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

impl Model {
    pub fn init(flags: Flags) -> Model {
        Model {
            lobby_id: flags.lobby_id,
            lobby: flags.lobby,
        }
    }
    pub fn viewer_is_host(&self, global: &global::Model) -> bool {
        self.lobby.host.id == global.viewer_id()
    }
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

pub fn update(
    _global: &global::Model,
    msg: Msg,
    _model: &mut Model,
    _orders: &mut impl Orders<Msg>,
) {
    match msg {
        Msg::Msg => {}
    }
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn view(global: &global::Model, model: &Model) -> Vec<Row<Msg>> {
    let mut rows = Vec::new();

    let viewer_is_host = model.viewer_is_host(global);

    rows.push(center(host_card(global, viewer_is_host, model)));

    let guests = &model.lobby.guests;

    for guest in guests {
        let guest_row = center(guest_card(guest.clone()));

        rows.push(guest_row)
    }

    for _ in 0..(MAX_GUESTS - guests.len()) {
        rows.push(center(empty_slot()))
    }

    rows
}

fn empty_slot() -> Cell<Msg> {
    Card::cell_from_rows(vec![CARD_WIDTH], vec![])
}

pub fn guest_card(guest: Player) -> Cell<Msg> {
    let name_row = Row::from_str(guest.name.to_string().as_str());

    player_card(vec![name_row])
}

pub fn host_card(global: &global::Model, viewer_is_host: bool, model: &Model) -> Cell<Msg> {
    let name_row = if viewer_is_host {
        Row::from_cells(vec![], vec![])
    } else {
        Row::from_str(global.viewer_name.to_string().as_str())
    };

    player_card(vec![name_row])
}

pub fn player_card<Msg: 'static>(rows: Vec<Row<Msg>>) -> Cell<Msg> {
    Card::cell_from_rows(vec![CARD_WIDTH], rows)
}

fn center<Msg: 'static>(cell: Cell<Msg>) -> Row<Msg> {
    Row::from_cells(vec![Style::JustifyCenter], vec![cell])
}

pub const CARD_WIDTH: Style = Style::WA;

pub const PARENT_STYLES: [Style; 2] = [Style::JustifyCenter, Style::G3];
