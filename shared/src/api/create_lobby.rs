use crate::id::Id;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////
// Request //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Request {
    host_id: Id,
}

impl Request {
    pub fn init(host_id: Id) -> Request {
        Request { host_id }
    }

    pub fn host_id(&self) -> Id {
        self.host_id.clone()
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Response {
    lobby_id: Id,
}

impl Response {
    pub fn init(lobby_id: Id) -> Response {
        Response { lobby_id }
    }

    pub fn get_lobby_id(&self) -> Id {
        self.lobby_id.clone()
    }

    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(byte_data: Vec<u8>) -> bincode::Result<Response> {
        bincode::deserialize(&byte_data[..])
    }
}
