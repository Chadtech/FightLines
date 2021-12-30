use shared::game::Game;
use shared::id::Id;
use shared::rng::{RandGen, RandSeed};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct Games {
    games: HashMap<Id, Game>,
    random_seed: RandSeed,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Games {
    pub fn init(random_seed: RandSeed) -> Games {
        Games {
            games: HashMap::new(),
            random_seed,
        }
    }

    pub fn get_game(&self, id: Id) -> Option<Game> {
        self.games.get(&id).map(|game| game.clone())
    }

    pub fn upsert(&mut self, id: Id, game: Game) {
        self.games.insert(id, game);
    }

    pub fn new_game(&mut self, game: Game) -> Id {
        let mut rand_gen = RandGen::from_seed(self.random_seed.clone());

        let new_id: Id = Id::new(&mut rand_gen);

        self.upsert(new_id.clone(), game);

        let new_seed: RandSeed = RandSeed::next(&mut rand_gen);

        self.random_seed = new_seed;

        new_id
    }
}
