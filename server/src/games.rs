use shared::facing_direction::FacingDirection;
use shared::game::{Game, GameId, GameInitFlags};
use shared::id::Id;
use shared::lobby::Lobby;
use shared::map::MapOpt;
use shared::name::Name;
use shared::player::Player;
use shared::rng::{RandGen, RandSeed};
use shared::team_color::TeamColor;
use shared::unit::{Place, Unit, UnitId};
use shared::{game, unit};
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

            lobby.set_map_choice(MapOpt::DisplayTest);

            let new_seed: RandSeed = RandSeed::next(&mut rng);

            Game::try_from(GameInitFlags::new(lobby, &mut RandGen::from_seed(new_seed))).unwrap()
        };

        games.insert(GameId::DisplayTest, display_test);

        let replenish_test: Game = {
            let red_player_id = Id::from_string("red".to_string(), true).unwrap();
            let mut lobby = Lobby::new(
                red_player_id.clone(),
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

            lobby.set_map_choice(MapOpt::ReplenishTest);

            let new_seed: RandSeed = RandSeed::next(&mut rng);

            let mut init_flags_rng = RandGen::from_seed(new_seed);

            let mut game_init_flags = GameInitFlags::new(lobby, &mut init_flags_rng);

            let mut infantry = unit::Model::new(
                Unit::Infantry,
                &red_player_id,
                Place::on_map(2, 2, FacingDirection::Right),
                &TeamColor::Red,
            );

            infantry.supplies = Unit::Infantry.max_supplies() / 3;

            let mut depleted_infantry = unit::Model::new(
                Unit::Infantry,
                &red_player_id,
                Place::on_map(2, 6, FacingDirection::Right),
                &TeamColor::Red,
            );

            depleted_infantry.supplies =
                (Unit::Infantry.active_supply_cost().unwrap() * 1.5).ceil() as i16;

            let mut truck = unit::Model::new(
                Unit::Truck,
                &red_player_id,
                Place::on_map(2, 4, FacingDirection::Right),
                &TeamColor::Red,
            );

            truck.supplies = Unit::Truck.max_supplies() / 2;

            let truck_id = UnitId::test("truck");
            game_init_flags.with_extra_units(&mut vec![
                (truck_id.clone(), truck),
                (
                    UnitId::test("supply crate"),
                    unit::Model::new(
                        Unit::SupplyCrate,
                        &red_player_id,
                        Place::InUnit(truck_id),
                        &TeamColor::Red,
                    ),
                ),
                (UnitId::test("infantry 1"), infantry.clone()),
                (UnitId::test("infantry 2"), infantry),
                (UnitId::test("depleted infantry"), depleted_infantry),
            ]);

            Game::try_from(game_init_flags).unwrap()
        };

        games.insert(GameId::ReplenishTest, replenish_test);

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

        let game: Game = Game::try_from(GameInitFlags::new(lobby, &mut rand_gen))?;

        let new_seed: RandSeed = RandSeed::next(&mut rand_gen);

        self.random_seed = new_seed;

        Ok(game)
    }
}
