use crate::id::Id;
use crate::name::Name;
use crate::player::Player;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Lobby {
    pub host: Player,
    pub host_id: Id,
    pub guests: HashMap<Id, Player>,
    pub num_players_limit: u8,
    pub name: String,
    pub kicked_guests: HashSet<Id>,
    game_started: bool,
}

pub enum AddError {
    LobbyIsFull,
}

#[derive(Serialize, Deserialize)]
pub enum Update {
    AddSlot,
    CloseSlot,
    ChangeName(String),
    ChangePlayerName { player_id: Id, new_name: Name },
    KickGuest { guest_id: Id },
}

#[derive(Clone)]

pub enum UpdateError {
    AtMaximumSlots,
    NoOpenSlotToClose,
    GameNameCannotBeEmpty,
    CannotFindPlayer,
}

////////////////////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////////////////////

const MAX_PLAYERS: u8 = MAX_GUESTS + 1;

const MIN_PLAYERS: u8 = MIN_GUESTS + 1;

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

pub const MAX_GUESTS: u8 = 3;

pub const MIN_GUESTS: u8 = 1;

impl Lobby {
    pub fn init(host_id: Id, host: Player) -> Lobby {
        Lobby {
            host,
            host_id,
            guests: HashMap::new(),
            game_started: false,
            name: "new game".to_string(),
            num_players_limit: 2,
            kicked_guests: HashSet::new(),
        }
    }

    pub fn num_guests_limit(&self) -> u8 {
        self.num_players_limit - 1
    }
    pub fn num_guests(&self) -> u8 {
        self.guests.len() as u8
    }

    pub fn num_players(&self) -> u8 {
        self.num_guests() + 1
    }

    pub fn at_player_count_minimum(&self) -> bool {
        self.num_players_limit == MIN_PLAYERS
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

        if self.num_players() < MAX_PLAYERS {
            let player_ids: Vec<Id> = players.into_iter().map(|(id, _)| id).collect();

            if !player_ids.contains(&guest_id) {
                self.guests.insert(guest_id.clone(), guest);
            }

            Ok(self)
        } else {
            Err(AddError::LobbyIsFull)
        }
    }

    pub fn many_updates(&mut self, upts: Vec<Update>) -> Result<&mut Lobby, UpdateError> {
        let mut err: Option<UpdateError> = None;

        for upt in upts {
            let result = self.update(upt);

            if let Err(error) = result {
                err = Some(error);
                break;
            }
        }

        match err {
            Some(err) => Err(err),
            None => Ok(self),
        }
    }

    pub fn started(&mut self) {
        self.game_started = true;
    }

    pub fn update(&mut self, upt: Update) -> Result<&mut Lobby, UpdateError> {
        match upt {
            Update::AddSlot => {
                if self.num_players_limit < MAX_PLAYERS {
                    self.num_players_limit += 1;
                    Ok(self)
                } else {
                    Err(UpdateError::AtMaximumSlots)
                }
            }
            Update::CloseSlot => {
                if MIN_PLAYERS < self.num_players_limit {
                    self.num_players_limit -= 1;
                    Ok(self)
                } else {
                    Err(UpdateError::NoOpenSlotToClose)
                }
            }
            Update::ChangeName(new_name) => {
                if new_name.is_empty() {
                    Err(UpdateError::GameNameCannotBeEmpty)
                } else {
                    self.name = new_name;
                    Ok(self)
                }
            }
            Update::ChangePlayerName {
                player_id,
                new_name,
            } => {
                if self.host_id == player_id {
                    self.host.name = new_name;
                    Ok(self)
                } else {
                    match self.guests.get_mut(&player_id) {
                        None => Err(UpdateError::CannotFindPlayer),
                        Some(guest) => {
                            guest.name = new_name;

                            Ok(self)
                        }
                    }
                }
            }
            Update::KickGuest { guest_id } => {
                self.guests.remove(&guest_id);
                self.kicked_guests.insert(guest_id);

                Ok(self)
            }
        }
    }
}
