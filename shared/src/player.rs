use crate::id::Id;
use crate::name::Name;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Player {
    pub id: Id,
    pub name: Name,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Player {
    pub fn new(id: Id, name: Name) -> Player {
        Player { id, name }
    }
}
