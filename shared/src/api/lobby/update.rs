use crate::lobby::{Lobby, LobbyId, Update};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////
// Request //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub lobby_id: LobbyId,
    pub updates: Vec<Update>,
}

impl Request {
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
    lobby_id: LobbyId,
    lobby: Lobby,
}

impl Response {
    pub fn init(lobby_id: LobbyId, lobby: Lobby) -> Response {
        Response { lobby_id, lobby }
    }

    pub fn get_lobby_id(&self) -> LobbyId {
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
