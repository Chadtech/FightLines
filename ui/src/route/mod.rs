use seed::Url;
use shared::game::GameId;
use shared::lobby::LobbyId;

pub mod component_library;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Route {
    Title,
    ComponentLibrary(component_library::Route),
    Lobby(LobbyId),
    Kicked,
    Game(GameId),
}

////////////////////////////////////////////////////////////////
// HELPERS //
////////////////////////////////////////////////////////////////

const LOBBY: &str = "lobby";

const KICKED: &str = "kicked";

const GAME: &str = "game";

////////////////////////////////////////////////////////////////
// API //
////////////////////////////////////////////////////////////////

impl ToString for Route {
    fn to_string(&self) -> String {
        let pieces = self.to_pieces();

        if pieces.is_empty() {
            String::new()
        } else {
            self.to_pieces().join("/")
        }
    }
}

impl Route {
    pub fn game(lobby_id: LobbyId) -> Route {
        Route::Game(GameId::from_lobby_id(lobby_id))
    }

    fn to_pieces(&self) -> Vec<String> {
        match self {
            Route::Title => vec![],
            Route::ComponentLibrary(_) => {
                let mut pieces = Vec::new();

                pieces.push(component_library::ROOT.to_string());

                pieces.append(&mut component_library::LANDING.to_pieces());

                pieces
            }
            Route::Lobby(id) => {
                vec![LOBBY.to_string(), id.to_string()]
            }
            Route::Kicked => {
                vec![KICKED.to_string()]
            }
            Route::Game(game_id) => {
                vec![GAME.to_string(), game_id.to_string()]
            }
        }
    }
    pub fn from_url(url: Url) -> Option<Route> {
        let mut path = url.path().iter();

        match path.next() {
            None => Some(Route::Title),
            Some(first) => {
                if first == LOBBY {
                    return path
                        .next()
                        .and_then(|id| LobbyId::from_string(id.clone()))
                        .map(Route::Lobby);
                }

                if first == GAME {
                    return path
                        .next()
                        .and_then(|id| GameId::from_string(id.clone()))
                        .map(Route::Game);
                }

                if first == component_library::ROOT {
                    let sub_route = component_library::Route::from_pieces(path)?;
                    return Some(Route::ComponentLibrary(sub_route));
                }

                if first == KICKED {
                    return Some(Route::Kicked);
                }

                None
            }
        }
    }

    pub fn to_url(&self) -> Url {
        Url::new().set_path(self.to_pieces())
    }
}
