use crate::id::Id;
use crate::lobby::Lobby;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////
// Request //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Request {
    lobby_id: Id,
}

impl Request {
    pub fn init(lobby_id: Id) -> Request {
        Request { lobby_id }
    }

    pub fn lobby_id(&self) -> Id {
        self.lobby_id.clone()
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Response {
    lobby_id: Id,
    lobby: Lobby,
}

impl Response {
    pub fn init(lobby_id: Id, lobby: Lobby) -> Response {
        Response { lobby_id, lobby }
    }

    pub fn get_lobby_id(&self) -> Id {
        self.lobby_id.clone()
    }

    pub fn get_lobby(&self) -> Lobby {
        self.lobby.clone()
    }

    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(byte_data: Vec<u8>) -> bincode::Result<Response> {
        bincode::deserialize(&byte_data[..])
    }
}
