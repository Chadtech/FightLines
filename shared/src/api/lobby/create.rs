use crate::id::Id;
use crate::lobby::{Lobby, LobbyId};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub lobby_id: LobbyId,
    pub lobby: Lobby,
}

impl Response {
    pub fn new(lobby_id: LobbyId, lobby: Lobby) -> Response {
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
    use crate::lobby::{Lobby, LobbyId};
    use crate::name::Name;
    use crate::player::Player;
    use crate::team_color::TeamColor;
    use std::str::FromStr;

    #[test]
    fn response_encodes_and_decodes() -> Result<(), String> {
        let lobby_id = LobbyId::from_int_test_only(0);

        let host_name = Name::from_str("host").map_err(|err| err.to_string())?;

        let response = Response::new(
            lobby_id,
            Lobby::new(
                Id::from_int_test_only(1),
                Player::new(host_name, TeamColor::Red),
            ),
        );

        let bytes = response.to_bytes().unwrap();

        let decoded_response = Response::from_bytes(bytes).unwrap();

        assert_eq!(response, decoded_response);

        Ok(())
    }
}
