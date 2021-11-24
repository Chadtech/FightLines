pub mod component_library;
pub mod error;
pub mod loading;
pub mod lobby;
pub mod not_found;
pub mod title;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub enum Page {
    Title(title::Model),
    ComponentLibrary(component_library::Model),
    Lobby(lobby::Model),
    Loading,
    Error(error::Model),
    NotFound,
    Blank,
}
