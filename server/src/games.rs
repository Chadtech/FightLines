use shared::game;
use shared::game::{Game, GameId};
use shared::id::Id;
use shared::lobby::Lobby;
use shared::name::Name;
use shared::player::Player;
use shared::rng::{RandGen, RandSeed};
use shared::team_color::TeamColor;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct Games {
    games: HashMap<GameId, Game>,
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

    pub fn init_dev(random_seed: RandSeed) -> Games {
        let mut games = HashMap::new();

        let mut rng = RandGen::from_seed(random_seed);

        let display_test: Game = {
            let mut lobby = Lobby::new(
                Id::Dev("red".to_string()),
                Player {
                    name: Name::from_str("red").unwrap(),
                    color: TeamColor::Red,
                },
            );

            let _ = lobby.add_guest(
                Id::Dev("blue".to_string()),
                Player {
                    name: Name::from_str("blue").unwrap(),
                    color: TeamColor::Blue,
                },
            );

            let new_seed: RandSeed = RandSeed::next(&mut rng);

            Game::try_from((lobby, &mut RandGen::from_seed(new_seed))).unwrap()
        };

        games.insert(GameId::DisplayTest, display_test);

        let final_seed: RandSeed = RandSeed::next(&mut rng);

        Games {
            games,
            random_seed: final_seed,
        }
    }

    pub fn get_game(&self, id: GameId) -> Option<&Game> {
        self.games.get(&id)
    }

    pub fn get_mut_game_and_seed(&mut self, id: GameId) -> Option<(&mut Game, RandSeed)> {
        match self.games.get_mut(&id) {
            None => None,
            Some(game) => {
                let mut rng = RandGen::from_seed(self.random_seed.clone());

                let new_seed_0: RandSeed = RandSeed::next(&mut rng);
                let new_seed_1: RandSeed = RandSeed::next(&mut rng);

                self.random_seed = new_seed_1;

                Some((game, new_seed_0))
            }
        }
    }

    pub fn upsert(&mut self, id: GameId, game: Game) {
        self.games.insert(id, game);
    }

    pub fn new_game_from_lobby(&mut self, lobby: Lobby) -> Result<Game, game::FromLobbyError> {
        let mut rand_gen = RandGen::from_seed(self.random_seed.clone());

        let game: Game = Game::try_from((lobby, &mut rand_gen))?;

        let new_seed: RandSeed = RandSeed::next(&mut rand_gen);

        self.random_seed = new_seed;

        Ok(game)
    }
}
