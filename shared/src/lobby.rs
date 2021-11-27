use crate::id::Id;
use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Lobby {
    pub host: Player,
    pub host_id: Id,
    pub guests: HashMap<Id, Player>,
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
    pub fn init(host_id: Id, host: Player) -> Lobby {
        Lobby {
            host,
            host_id,
            guests: HashMap::new(),
            game_started: false,
        }
    }

    pub fn players(self) -> HashMap<Id, Player> {
        let mut players = HashMap::new();

        players.insert(self.host_id, self.host);

        for (id, guest) in self.guests.into_iter() {
            players.insert(id, guest);
        }

        players
    }

    pub fn add_guest(&mut self, guest_id: Id, guest: Player) -> Result<&mut Lobby, AddError> {
        let players = self.clone().players();

        if players.len() < MAX_PLAYERS {
            let player_ids: Vec<Id> = players.into_iter().map(|(id, _)| id).collect();

            if !player_ids.contains(&guest_id) {
                self.guests.insert(guest_id.clone(), guest);
            }

            Ok(self)
        } else {
            Err(AddError::LobbyIsFull)
        }
    }
}
