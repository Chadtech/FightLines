use crate::id::Id;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////
// Request //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Request {
    player_id: Id,
}

impl Request {
    pub fn init(player_id: Id) -> Request {
        Request { player_id }
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
pub struct Response {}

impl Response {
    pub fn init() -> Response {
        Response {}
    }

    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(byte_data: Vec<u8>) -> bincode::Result<Response> {
        bincode::deserialize(&byte_data[..])
    }
}
