use crate::lobby::Lobby;
use shared::id::Id;
use shared::rng::RandGen;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct Lobbies {
    lobbies: HashMap<Id, Lobby>,
    random_seed: usize,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Lobbies {
    pub fn init(random_seed: usize) -> Lobbies {
        Lobbies {
            lobbies: HashMap::new(),
            random_seed,
        }
    }

    pub fn insert_lobby(&mut self, id: Id, lobby: Lobby) {
        self.lobbies.insert(id, lobby);
    }

    pub fn new_lobby(&mut self, lobby: Lobby) -> Id {
        let mut rand_gen = RandGen::from_usize(self.random_seed);

        let new_id: Id = rand_gen.gen();

        self.insert_lobby(new_id.clone(), lobby);

        let new_seed: usize = rand_gen.gen();

        self.random_seed = new_seed;

        new_id
    }
}
