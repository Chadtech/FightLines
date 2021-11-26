use crate::global;
use crate::view::cell::{Cell, Row};
use seed::log;
use seed::prelude::Orders;
use shared::id::Id;
use shared::lobby::Lobby;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    lobby_id: Id,
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
// Init //
///////////////////////////////////////////////////////////////

pub fn init(flags: Flags) -> Model {
    Model {
        lobby_id: flags.lobby_id,
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

pub fn view(model: &Model) -> Vec<Row<Msg>> {
    log!("A");
    vec![Row::from_str(model.lobby_id.to_string().as_str())]
}