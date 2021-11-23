use std::slice::Iter;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub enum Route {
    Button,
}

///////////////////////////////////////////////////////////////
// Helpers //
///////////////////////////////////////////////////////////////

const BUTTON_PATH: &str = "button";

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

pub const ROOT: &str = "component-library";

pub const LANDING: Route = Route::Button;

impl ToString for Route {
    fn to_string(&self) -> String {
        self.to_pieces().join("/")
    }
}

impl Route {
    pub fn to_pieces(self) -> Vec<String> {
        match self {
            Route::Button => vec![BUTTON_PATH.to_string()],
        }
    }
    pub fn from_pieces(mut pieces: Iter<String>) -> Option<Route> {
        match pieces.next() {
            None => Some(LANDING),
            Some(piece) => {
                let mut ret_route = None;

                if piece == BUTTON_PATH {
                    ret_route = Some(Route::Button);
                }

                ret_route
            }
        }
    }
}
