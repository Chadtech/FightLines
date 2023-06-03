use crate::game::GameId;
use crate::id::Id;
use crate::lobby::LobbyId;
use crate::team_color::TeamColor;
use crate::unit::Unit;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum Endpoint {
    CreateLobby,
    GetLobby(Param<LobbyId>),
    JoinLobby(Param<LobbyId>),
    UpdateLobby,
    StartGame,
    GetGame(Param<GameId>),
    SubmitTurn(Param<GameId>, Param<Id>),
    ThumbnailAsset(Unit, TeamColor),
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

pub const ROOT: &str = "/api";

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
            Endpoint::StartGame => {
                vec!["lobby".to_string(), "start".to_string()]
            }
            Endpoint::GetGame(param) => {
                vec!["game".to_string(), "get".to_string(), param.to_string()]
            }
            Endpoint::SubmitTurn(game_id, player_id) => {
                vec![
                    "game".to_string(),
                    "submit-turn".to_string(),
                    game_id.to_string(),
                    player_id.to_string(),
                ]
            }
            Endpoint::ThumbnailAsset(unit, team_color) => {
                let mut buf = unit.to_string();
                buf.push('-');
                buf.push_str(team_color.to_string().as_str());

                buf = buf.replace(' ', "-");

                buf.push_str(".png");

                vec!["asset".to_string(), buf]
            }
        }
    }

    pub fn make_get_game(id: GameId) -> Endpoint {
        Endpoint::GetGame(Param::Value(id))
    }

    pub fn template_get_game() -> Endpoint {
        Endpoint::GetGame(Param::Template("id".to_string()))
    }

    pub fn template_submit_turn() -> Endpoint {
        Endpoint::SubmitTurn(
            Param::Template("game_id".to_string()),
            Param::Template("player_id".to_string()),
        )
    }

    pub fn update_lobby() -> Endpoint {
        Endpoint::UpdateLobby
    }

    pub fn join_lobby(lobby_id: LobbyId) -> Endpoint {
        Endpoint::JoinLobby(Param::Value(lobby_id))
    }

    pub fn make_get_lobby(id: LobbyId) -> Endpoint {
        Endpoint::GetLobby(Param::Value(id))
    }

    pub fn submit_turn(game_id: GameId, player_id: Id) -> Endpoint {
        Endpoint::SubmitTurn(Param::Value(game_id), Param::Value(player_id))
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
