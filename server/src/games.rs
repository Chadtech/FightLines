use shared::game;
use shared::game::Game;
use shared::id::Id;
use shared::lobby::Lobby;
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

    pub fn new_game_from_lobby(&mut self, lobby: Lobby) -> Result<Game, game::FromLobbyError> {
        let mut rand_gen = RandGen::from_seed(self.random_seed.clone());

        let game = Game::from_lobby(lobby, &mut rand_gen)?;

        let new_seed: RandSeed = RandSeed::next(&mut rand_gen);

        self.random_seed = new_seed;

        Ok(game)
    }
}
