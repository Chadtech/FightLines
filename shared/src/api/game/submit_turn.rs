use crate::game;
use crate::game::Game;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////
// Request //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub moves: Vec<game::action::Action>,
    pub changes: Vec<game::Change>,
}

impl Request {
    pub fn init(moves: Vec<game::action::Action>, changes: Vec<game::Change>) -> Request {
        Request { moves, changes }
    }

    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }
    pub fn from_bytes(byte_data: Vec<u8>) -> bincode::Result<Request> {
        bincode::deserialize(&byte_data[..])
    }
}

////////////////////////////////////////////////////////////////
// Response //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub game: Game,
}

impl Response {
    pub fn new(game: Game) -> Response {
        Response { game }
    }

    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(byte_data: Vec<u8>) -> bincode::Result<Response> {
        bincode::deserialize(&byte_data[..])
    }
}
