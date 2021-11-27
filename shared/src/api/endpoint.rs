////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

use crate::id::Id;

pub enum Endpoint {
    CreateLobby,
    GetLobby(Id),
    JoinLobby(Id),
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

pub const LOBBY: &'static str = "/lobby";

pub const ROOT: &'static str = "/api";

impl Endpoint {
    fn to_pieces(self) -> Vec<String> {
        match self {
            Endpoint::CreateLobby => vec!["lobby".to_string(), "create".to_string()],
            Endpoint::GetLobby(id) => vec![
                "lobby".to_string(),
                "get".to_string(),
                id.clone().to_string(),
            ],
            Endpoint::JoinLobby(id) => vec![
                "lobby".to_string(),
                "join".to_string(),
                id.clone().to_string(),
            ],
        }
    }

    pub fn to_string(self) -> String {
        let mut buf = String::new();
        buf.push('/');

        let pieces = self.to_pieces();
        buf.push_str(pieces.join("/").as_str());

        buf
    }

    pub fn to_url(self) -> String {
        let mut buf = String::new();
        buf.push_str(ROOT);
        buf.push_str(self.to_string().as_str());
        buf
    }
}
