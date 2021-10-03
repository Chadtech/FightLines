pub mod component_library;
pub mod not_found;
pub mod title;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub enum Page {
    Title(title::Model),
    ComponentLibrary(component_library::Model),
    NotFound,
    Blank,
}
