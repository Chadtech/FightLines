use crate::id::Id;
use crate::player::Player;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Lobby {
    pub host: Player,
    pub guests: Vec<Player>,
    game_started: bool,
}

pub enum AddError {
    LobbyIsFull,
}

////////////////////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////////////////////

const MAX_PLAYERS: usize = MAX_GUESTS + 1;

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

pub const MAX_GUESTS: usize = 3;

impl Lobby {
    pub fn init(host: Player) -> Lobby {
        Lobby {
            host,
            guests: Vec::new(),
            game_started: false,
        }
    }

    pub fn players(mut self) -> Vec<Player> {
        let mut players = Vec::new();

        players.push(self.host);

        players.append(&mut self.guests);

        players
    }

    pub fn add_guest(&mut self, guest: Player) -> Result<&mut Lobby, AddError> {
        let players = self.clone().players();

        if players.len() < MAX_PLAYERS {
            let player_ids: Vec<Id> = players.into_iter().map(|p| p.id).collect();

            if !player_ids.contains(&guest.id) {
                self.guests.push(guest);
            }

            Ok(self)
        } else {
            Err(AddError::LobbyIsFull)
        }
    }
}
