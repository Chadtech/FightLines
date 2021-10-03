use seed::Url;

pub mod component_library;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Route {
    Title,
    ComponentLibrary(component_library::Route),
}

////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////

impl ToString for Route {
    fn to_string(&self) -> String {
        self.to_pieces().join("/")
    }
}

impl Route {
    fn to_pieces(&self) -> Vec<String> {
        match self {
            Route::Title => vec!["/".to_string()],
            Route::ComponentLibrary(_) => vec![],
        }
    }
    pub fn from_url(url: Url) -> Option<Route> {
        let mut path = url.path().iter();

        match path.next() {
            None => Some(Route::Title),
            Some(first) => {
                if first == component_library::ROOT {
                    let sub_route = component_library::Route::from_pieces(path)?;
                    return Some(Route::ComponentLibrary(sub_route));
                }

                None
            }
        }
    }
}
