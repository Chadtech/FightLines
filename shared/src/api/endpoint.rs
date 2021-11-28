////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

use crate::id::Id;

#[derive(Clone)]
pub enum Endpoint {
    CreateLobby,
    GetLobby(Param<Id>),
    JoinLobby(Param<Id>),
    UpdateLobby,
}

#[derive(Clone)]

pub enum Param<P>
where
    P: ToString + Clone,
{
    Value(P),
    Template(String),
}

////////////////////////////////////////////////////////////////////////////////
// HELPERS //
////////////////////////////////////////////////////////////////////////////////

impl<P: ToString + Clone> ToString for Param<P> {
    fn to_string(&self) -> String {
        match self {
            Param::Value(v) => v.to_string(),
            Param::Template(template) => {
                let mut ret = String::new();
                ret.push('{');
                ret.push_str(template.as_str());
                ret.push('}');

                ret
            }
        }
    }
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        buf.push('/');

        let pieces = self.to_pieces();
        buf.push_str(pieces.join("/").as_str());

        buf
    }
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

pub const ROOT: &'static str = "/api";

impl Endpoint {
    fn to_pieces(&self) -> Vec<String> {
        match self {
            Endpoint::CreateLobby => vec!["lobby".to_string(), "create".to_string()],
            Endpoint::GetLobby(param) => {
                vec!["lobby".to_string(), "get".to_string(), param.to_string()]
            }
            Endpoint::JoinLobby(param) => {
                vec!["lobby".to_string(), "join".to_string(), param.to_string()]
            }
            Endpoint::UpdateLobby => vec!["lobby".to_string(), "update".to_string()],
        }
    }

    pub fn update_lobby() -> Endpoint {
        Endpoint::UpdateLobby
    }

    pub fn join_lobby(lobby_id: Id) -> Endpoint {
        Endpoint::JoinLobby(Param::Value(lobby_id))
    }

    pub fn template_get_lobby() -> Endpoint {
        Endpoint::GetLobby(Param::Template("id".to_string()))
    }

    pub fn template_join_lobby() -> Endpoint {
        Endpoint::JoinLobby(Param::Template("id".to_string()))
    }

    pub fn to_url(self) -> String {
        let mut buf = String::new();
        buf.push_str(ROOT);
        buf.push_str(self.to_string().as_str());
        buf
    }
}
