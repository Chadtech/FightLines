////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub enum Endpoint {
    CreateLobby,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

pub const CREATE_LOBBY: &'static str = "/lobby/create";

pub const ROOT: &'static str = "/api";

impl Endpoint {
    fn to_pieces<'a>(self) -> &'a [&'a str] {
        match self {
            Endpoint::CreateLobby => &["lobby", "create"],
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
