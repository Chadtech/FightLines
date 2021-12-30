use seed::Url;
use shared::id::Id;

pub mod component_library;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Route {
    Title,
    ComponentLibrary(component_library::Route),
    Lobby(Id),
    Kicked,
}

////////////////////////////////////////////////////////////////
// HELPERS //
////////////////////////////////////////////////////////////////

const LOBBY: &'static str = "lobby";

const KICKED: &'static str = "kicked";

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
                        .and_then(|id| Id::from_string(id.clone()))
                        .map(Route::Lobby);
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
