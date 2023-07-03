use seed::{log, Url};
use shared::game::GameId;
use shared::id::Id;
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
    Game {
        game_id: GameId,
        dev_viewer_id: Option<Id>,
    },
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
        let query = self.to_query();

        if pieces.is_empty() {
            String::new()
        } else {
            let mut pieces_combined = self.to_pieces().join("/");

            if !query.is_empty() {
                pieces_combined.push('?');

                for (key, value) in query {
                    let mut param = key;

                    param.push('=');

                    param.push_str(value.as_str());

                    pieces_combined.push_str(param.as_str());
                }
            }

            pieces_combined
        }
    }
}

impl Route {
    pub fn game(lobby_id: LobbyId) -> Route {
        Route::Game {
            game_id: GameId::from_lobby_id(lobby_id),
            dev_viewer_id: None,
        }
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
            Route::Game { game_id, .. } => {
                vec![GAME.to_string(), game_id.to_string()]
            }
        }
    }

    fn to_query(&self) -> Vec<(String, String)> {
        match self {
            Route::Title => vec![],
            Route::ComponentLibrary(_) => vec![],
            Route::Lobby(_) => vec![],
            Route::Kicked => vec![],
            Route::Game { dev_viewer_id, .. } => match dev_viewer_id {
                None => vec![],
                Some(dev_viewer_id) => {
                    vec![("dev-viewer-id".to_string(), dev_viewer_id.to_string())]
                }
            },
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
                    let dev_viewer_id = url
                        .search()
                        .get("dev-viewer-id")
                        .and_then(|values| values.first())
                        .map(|value| Id::Dev(value.to_string()));

                    log!(&path);

                    return path
                        .next()
                        .and_then(|id| GameId::from_string(id.clone()))
                        .map(|game_id| {
                            log!(game_id);

                            Route::Game {
                                game_id,
                                dev_viewer_id,
                            }
                        });
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
