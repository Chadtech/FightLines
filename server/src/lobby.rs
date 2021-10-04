use crate::player::Player;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct Lobby {
    host: Player,
    guests: Vec<Player>,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Lobby {
    pub fn init(host: Player) -> Lobby {
        Lobby {
            host,
            guests: Vec::new(),
        }
    }
}
