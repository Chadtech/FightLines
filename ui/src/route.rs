use seed::Url;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub enum Route {
    Title,
}

impl Route {
    pub fn from_url(url: Url) -> Option<Route> {
        let mut path = url.path().iter();

        match path.next() {
            None => Some(Route::Title),
            Some(first) => match first {
                _ => None,
            },
        }
    }
}
