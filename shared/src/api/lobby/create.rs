use crate::id::Id;
use crate::lobby::Lobby;
use crate::name::Name;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////
// Request //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize)]
pub struct Request {
    host_id: Id,
    pub host_name: Name,
}

impl Request {
    pub fn init(host_id: Id, host_name: Name) -> Request {
        Request { host_id, host_name }
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
    pub lobby_id: Id,
    pub lobby: Lobby,
}

impl Response {
    pub fn init(lobby_id: Id, lobby: Lobby) -> Response {
        Response { lobby_id, lobby }
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
    use crate::lobby::Lobby;
    use crate::name::Name;
    use crate::player::Player;

    #[test]
    fn response_encodes_and_decodes() {
        let lobby_id = Id::from_int_test_only(0);

        let response = Response::init(
            lobby_id,
            Lobby::init(
                Id::from_int_test_only(1),
                Player::new(Name::from_str("host")),
            ),
        );

        let bytes = response.to_bytes().unwrap();

        let decoded_response = Response::from_bytes(bytes).unwrap();

        assert_eq!(response, decoded_response);
    }
}
