use crate::id::Id;
use crate::lobby::Lobby;
use crate::map::Map;
use crate::player::Player;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Game {
    // host
    pub host: Player,
    pub host_id: Id,
    // first guest
    pub first_guest: Player,
    pub first_guest_id: Id,
    // remaining guests
    pub remaining_guests: Vec<(Id, Player)>,
    pub map: Map,
}

pub enum FromLobbyError {
    NotEnoughPlayers,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Game {
    pub fn from_lobby(lobby: Lobby) -> Result<Game, FromLobbyError> {
        let guests: Vec<(Id, Player)> = lobby.guests.into_iter().collect();

        match guests.split_first() {
            None => Err(FromLobbyError::NotEnoughPlayers),
            Some((first, rest)) => {
                let (first_guest_id, first_guest) = first;

                let game = Game {
                    host: lobby.host,
                    host_id: lobby.host_id,

                    first_guest: first_guest.clone(),
                    first_guest_id: first_guest_id.clone(),

                    remaining_guests: rest.to_vec(),
                    map: Map::grass_square(),
                };

                Ok(game)
            }
        }
    }

    pub fn dimensions(&self) -> (u8, u8) {
        self.map.dimensions()
    }
}
