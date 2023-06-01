use crate::id::Id;
use crate::map::MapOpt;
use crate::name::Name;
use crate::player::Player;
use crate::rng::RandGen;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct LobbyId(Id);

impl ToString for LobbyId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl LobbyId {
    // When a lobby becomes a game, we reuse the id, so a lobby
    // id needs to transform into a game id
    // - Chad Dec 25 2022 (merry christmas!)
    pub fn ambiguate(&self) -> Id {
        self.0.clone()
    }

    pub fn from_int_test_only(n: u8) -> LobbyId {
        LobbyId(Id::from_int_test_only(n))
    }

    pub fn from_string(s: String) -> Option<LobbyId> {
        Id::from_string(s, false).map(LobbyId)
    }

    pub fn new(rng: &mut RandGen) -> LobbyId {
        LobbyId(Id::new(rng))
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Lobby {
    pub host: Player,
    pub host_id: Id,
    pub guests: HashMap<Id, Player>,
    pub num_players_limit: u8,
    pub name: Name,
    pub kicked_guests: HashSet<Id>,
    pub game_started: bool,
    pub map_choice: MapOpt,
}

#[derive(Debug)]
pub enum AddError {
    LobbyIsFull,
}

#[derive(Serialize, Deserialize)]
pub enum Update {
    AddSlot,
    CloseSlot,
    ChangeName(Name),
    ChangePlayerName { player_id: Id, new_name: Name },
    KickGuest { guest_id: Id },
    SetMapOption(MapOpt),
}

#[derive(Clone)]

pub enum UpdateError {
    AtMaximumSlots,
    NoOpenSlotToClose,
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
    pub fn new(host_id: Id, host: Player) -> Lobby {
        Lobby {
            host,
            host_id,
            guests: HashMap::new(),
            game_started: false,
            name: Name::new("game"),
            num_players_limit: 2,
            kicked_guests: HashSet::new(),
            map_choice: MapOpt::GrassSquare,
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

    pub fn add_guest(&mut self, guest_id: Id, guest: Player) -> Result<(), AddError> {
        let players = self.clone().players();

        if self.num_players() < MAX_PLAYERS {
            if !players.into_keys().any(|x| x == guest_id) {
                self.guests.insert(guest_id, guest);
            }

            Ok(())
        } else {
            Err(AddError::LobbyIsFull)
        }
    }

    pub fn many_updates(&mut self, upts: Vec<Update>) -> Result<(), UpdateError> {
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
            None => Ok(()),
        }
    }

    pub fn started(&mut self) {
        self.game_started = true;
    }

    pub fn set_map_choice(&mut self, map_opt: MapOpt) {
        self.map_choice = map_opt;
    }

    pub fn update(&mut self, upt: Update) -> Result<(), UpdateError> {
        match upt {
            Update::AddSlot => {
                if self.num_players_limit < MAX_PLAYERS {
                    self.num_players_limit += 1;
                } else {
                    return Err(UpdateError::AtMaximumSlots);
                }
            }
            Update::CloseSlot => {
                if MIN_PLAYERS < self.num_players_limit {
                    self.num_players_limit -= 1;
                } else {
                    return Err(UpdateError::NoOpenSlotToClose);
                }
            }
            Update::ChangeName(new_name) => {
                self.name = new_name;
            }
            Update::ChangePlayerName {
                player_id,
                new_name,
            } => {
                if self.host_id == player_id {
                    self.host.name = new_name;
                } else {
                    match self.guests.get_mut(&player_id) {
                        None => return Err(UpdateError::CannotFindPlayer),
                        Some(guest) => {
                            guest.name = new_name;
                        }
                    }
                }
            }
            Update::KickGuest { guest_id } => {
                self.guests.remove(&guest_id);
                self.kicked_guests.insert(guest_id);
            }
            Update::SetMapOption(map_opt) => {
                self.set_map_choice(map_opt);
            }
        }

        Ok(())
    }
}
