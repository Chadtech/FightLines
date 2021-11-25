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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod test_create_lobby {
    use crate::api::lobby::create::Response;
    use crate::id::Id;

    #[test]
    fn response_encodes_and_decodes() {
        let id = Id::from_int_test_only(0);

        let response = Response::init(id);

        let bytes = response.to_bytes().unwrap();

        let decoded_response = Response::from_bytes(bytes).unwrap();

        assert_eq!(response, decoded_response);
    }
}
