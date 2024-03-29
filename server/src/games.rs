use shared::facing_direction::FacingDirection;
use shared::game::{DevGameId, Game, GameId, GameInitFlags};
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

pub struct Flags {
    pub dev: bool,
    pub rand_seed: RandSeed,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl From<Flags> for Games {
    fn from(flags: Flags) -> Self {
        let mut rng = RandGen::from_seed(flags.rand_seed);

        let games = if flags.dev {
            let dev_seed: RandSeed = RandSeed::next(&mut rng);

            Games::dev_games(dev_seed)
        } else {
            HashMap::new()
        };

        let final_seed: RandSeed = RandSeed::next(&mut rng);

        Games {
            games,
            random_seed: final_seed,
        }
    }
}

impl Games {
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

    fn dev_games(rand_seed: RandSeed) -> HashMap<GameId, Game> {
        let mut rng = RandGen::from_seed(rand_seed);

        let mut games = HashMap::new();

        // This does nothing, but if you are
        // brought here by a compile error,
        // you probably need to add your new
        // dev id to ALL_DEV_IDS
        match DevGameId::DisplayTest {
            DevGameId::DisplayTest => {}
            DevGameId::ReplenishTest => {}
            DevGameId::ArrowTest => {}
            DevGameId::GamePlayTest => {}
            DevGameId::SingleUnitTest => {}
        }

        for dev_game_id in game::ALL_DEV_IDS {
            let game = match dev_game_id {
                DevGameId::DisplayTest => {
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

                    Game::try_from(GameInitFlags::new(lobby, &mut RandGen::from_seed(new_seed)))
                        .unwrap()
                }
                DevGameId::ReplenishTest => {
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
                }
                DevGameId::ArrowTest => {
                    let red_player_id = Id::from_string("red".to_string(), true).unwrap();
                    let mut lobby = Lobby::new(
                        red_player_id.clone(),
                        Player {
                            name: Name::from_str("red").unwrap(),
                            color: TeamColor::Red,
                        },
                    );

                    let blue_player_id = Id::from_string("blue".to_string(), true).unwrap();

                    let _ = lobby.add_guest(
                        blue_player_id.clone(),
                        Player {
                            name: Name::from_str("blue").unwrap(),
                            color: TeamColor::Blue,
                        },
                    );

                    lobby.set_map_choice(MapOpt::ArrowTest);

                    let new_seed: RandSeed = RandSeed::next(&mut rng);

                    let mut init_flags_rng = RandGen::from_seed(new_seed);

                    let mut game_init_flags = GameInitFlags::new(lobby, &mut init_flags_rng);

                    let truck_1 = unit::Model::new(
                        Unit::Truck,
                        &red_player_id,
                        Place::on_map(6, 6, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let truck_2 = unit::Model::new(
                        Unit::Truck,
                        &red_player_id,
                        Place::on_map(1, 1, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let blue_truck = unit::Model::new(
                        Unit::Truck,
                        &blue_player_id,
                        Place::on_map(6, 8, FacingDirection::Right),
                        &TeamColor::Blue,
                    );

                    game_init_flags.with_extra_units(&mut vec![
                        (UnitId::test("red truck 1"), truck_1),
                        (UnitId::test("red truck 2"), truck_2),
                        (UnitId::test("blue truck"), blue_truck),
                    ]);

                    Game::try_from(game_init_flags).unwrap()
                }
                DevGameId::GamePlayTest => {
                    let red_player_id = Id::from_string("red".to_string(), true).unwrap();
                    let mut lobby = Lobby::new(
                        red_player_id.clone(),
                        Player {
                            name: Name::from_str("red").unwrap(),
                            color: TeamColor::Red,
                        },
                    );

                    let blue_player_id = Id::from_string("blue".to_string(), true).unwrap();

                    let _ = lobby.add_guest(
                        blue_player_id.clone(),
                        Player {
                            name: Name::from_str("blue").unwrap(),
                            color: TeamColor::Blue,
                        },
                    );

                    lobby.set_map_choice(MapOpt::GamePlayTest);

                    let new_seed: RandSeed = RandSeed::next(&mut rng);

                    let mut init_flags_rng = RandGen::from_seed(new_seed);

                    let mut game_init_flags = GameInitFlags::new(lobby, &mut init_flags_rng);

                    let red_truck_1 = unit::Model::new(
                        Unit::Truck,
                        &red_player_id,
                        Place::on_map(1, 2, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let red_truck_2 = unit::Model::new(
                        Unit::Truck,
                        &red_player_id,
                        Place::on_map(2, 2, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let red_infantry_1 = unit::Model::new(
                        Unit::Infantry,
                        &red_player_id,
                        Place::on_map(3, 2, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let red_infantry_2 = unit::Model::new(
                        Unit::Infantry,
                        &red_player_id,
                        Place::on_map(2, 4, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let red_tank_1 = unit::Model::new(
                        Unit::Tank,
                        &red_player_id,
                        Place::on_map(3, 3, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let red_tank_2 = unit::Model::new(
                        Unit::Tank,
                        &red_player_id,
                        Place::on_map(3, 5, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let red_supply_crate_1 = unit::Model::new(
                        Unit::SupplyCrate,
                        &red_player_id,
                        Place::on_map(3, 10, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let red_supply_crate_2 = unit::Model::new(
                        Unit::SupplyCrate,
                        &red_player_id,
                        Place::on_map(8, 5, FacingDirection::Right),
                        &TeamColor::Red,
                    );

                    let blue_truck_1 = unit::Model::new(
                        Unit::Truck,
                        &blue_player_id,
                        Place::on_map(14, 12, FacingDirection::Left),
                        &TeamColor::Blue,
                    );

                    let blue_truck_2 = unit::Model::new(
                        Unit::Truck,
                        &blue_player_id,
                        Place::on_map(13, 13, FacingDirection::Left),
                        &TeamColor::Blue,
                    );

                    let blue_infantry_1 = unit::Model::new(
                        Unit::Infantry,
                        &blue_player_id,
                        Place::on_map(12, 12, FacingDirection::Left),
                        &TeamColor::Blue,
                    );

                    let blue_infantry_2 = unit::Model::new(
                        Unit::Infantry,
                        &blue_player_id,
                        Place::on_map(13, 11, FacingDirection::Left),
                        &TeamColor::Blue,
                    );

                    let blue_tank_1 = unit::Model::new(
                        Unit::Tank,
                        &blue_player_id,
                        Place::on_map(11, 12, FacingDirection::Left),
                        &TeamColor::Blue,
                    );

                    let blue_tank_2 = unit::Model::new(
                        Unit::Tank,
                        &blue_player_id,
                        Place::on_map(11, 10, FacingDirection::Left),
                        &TeamColor::Blue,
                    );

                    let blue_supply_crate_1 = unit::Model::new(
                        Unit::SupplyCrate,
                        &red_player_id,
                        Place::on_map(6, 12, FacingDirection::Right),
                        &TeamColor::Blue,
                    );

                    let blue_supply_crate_2 = unit::Model::new(
                        Unit::SupplyCrate,
                        &red_player_id,
                        Place::on_map(11, 5, FacingDirection::Right),
                        &TeamColor::Blue,
                    );

                    game_init_flags.with_extra_units(&mut vec![
                        (UnitId::test("red truck 1"), red_truck_1),
                        (UnitId::test("red truck 2"), red_truck_2),
                        (UnitId::test("blue truck 1"), blue_truck_1),
                        (UnitId::test("blue truck 2"), blue_truck_2),
                        (UnitId::test("red infantry 1"), red_infantry_1),
                        (UnitId::test("red infantry 2"), red_infantry_2),
                        (UnitId::test("blue infantry 1"), blue_infantry_1),
                        (UnitId::test("blue infantry 2"), blue_infantry_2),
                        (UnitId::test("red tank 1"), red_tank_1),
                        (UnitId::test("red tank 2"), red_tank_2),
                        (UnitId::test("blue tank 1"), blue_tank_1),
                        (UnitId::test("blue tank 2"), blue_tank_2),
                        (UnitId::test("red supply crate 1"), red_supply_crate_1),
                        (UnitId::test("red supply crate 2"), red_supply_crate_2),
                        (UnitId::test("blue supply crate 1"), blue_supply_crate_1),
                        (UnitId::test("blue supply crate 2"), blue_supply_crate_2),
                    ]);

                    Game::try_from(game_init_flags).unwrap()
                }
                DevGameId::SingleUnitTest => {
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

                    lobby.set_map_choice(MapOpt::SingleUnitTest);

                    let new_seed: RandSeed = RandSeed::next(&mut rng);

                    Game::try_from(GameInitFlags::new(lobby, &mut RandGen::from_seed(new_seed)))
                        .unwrap()
                }
            };

            games.insert(dev_game_id.into(), game);
        }

        games
    }
}
