use crate::lobby::Lobby;
use shared::id::Id;
use shared::rng::{RandGen, RandSeed};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct Lobbies {
    lobbies: HashMap<Id, Lobby>,
    random_seed: RandSeed,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Lobbies {
    pub fn init(random_seed: RandSeed) -> Lobbies {
        Lobbies {
            lobbies: HashMap::new(),
            random_seed,
        }
    }

    pub fn insert_lobby(&mut self, id: Id, lobby: Lobby) {
        self.lobbies.insert(id, lobby);
    }

    pub fn new_lobby(&mut self, lobby: Lobby) -> Id {
        let mut rand_gen = RandGen::from_seed(self.random_seed.clone());

        let new_id: Id = Id::new(&mut rand_gen);

        println!("New Lobby Id {}", new_id);

        self.insert_lobby(new_id.clone(), lobby);

        let new_seed: RandSeed = RandSeed::next(&mut rand_gen);

        self.random_seed = new_seed;

        new_id
    }
}
