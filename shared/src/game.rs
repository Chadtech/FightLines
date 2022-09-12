use crate::facing_direction::FacingDirection;
use crate::game::FromLobbyError::CouldNotFindInitialMapMilitary;
use crate::id::Id;
use crate::lobby::Lobby;
use crate::located::Located;
use crate::map::{Map, MapOpt};
use crate::owner::Owned;
use crate::player::Player;
use crate::rng::RandGen;
use crate::team_color::TeamColor;
use crate::unit::{Unit, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Game {
    // host
    pub host: Player,
    pub host_id: Id,
    // first guest
    pub first_guest: Player,
    pub first_guest_id: Id,
    // remaining guests
    pub remaining_guests: Vec<(Id, Player)>,
    pub units: HashMap<UnitId, Located<UnitModel>>,
    pub map: Map,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct UnitModel {
    pub unit: Unit,
    pub facing: FacingDirection,
    pub owner: Id,
    pub color: TeamColor,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Guest {
    player: Player,
    military: Military,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Military(HashMap<UnitId, Located<Owned<Unit>>>);

pub enum FromLobbyError {
    NotEnoughPlayers,
    CouldNotFindInitialMapMilitary {
        required_player_count: u8,
        found_player_count: u8,
    },
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Game {
    pub fn from_lobby(lobby: Lobby, rng: &mut RandGen) -> Result<Game, FromLobbyError> {
        let num_players = lobby.num_players();
        let guests: Vec<(Id, Player)> = lobby.guests.into_iter().collect();

        let map_choice = MapOpt::GrassSquare;
        let map = map_choice.to_map();
        let initial_militaries = map_choice.initial_militaries();

        match guests.split_first() {
            None => Err(FromLobbyError::NotEnoughPlayers),
            Some((first, rest)) => {
                let (first_guest_id, first_guest) = first;

                let mut id_units = |units: Vec<Located<(FacingDirection, Unit)>>,
                                    owner_id: &Id,
                                    color: &TeamColor|
                 -> Vec<(UnitId, Located<UnitModel>)> {
                    let mut units_with_ids: Vec<(UnitId, Located<UnitModel>)> = vec![];

                    for located_unit in units {
                        let unit_id = UnitId::new(rng);

                        let (facing, unit) = located_unit.value;

                        let new_located_unit = Located::<UnitModel> {
                            value: UnitModel {
                                unit,
                                owner: owner_id.clone(),
                                facing,
                                color: color.clone(),
                            },
                            x: located_unit.x,
                            y: located_unit.y,
                        };

                        units_with_ids.push((unit_id, new_located_unit));
                    }

                    units_with_ids
                };

                let mut remaining_guests_with_militaries: Vec<(UnitId, Located<UnitModel>)> =
                    vec![];

                for (index, (guest_id, guest)) in rest.into_iter().enumerate() {
                    let initial_military = initial_militaries
                        .rest_players_miliatries
                        .get(index)
                        .ok_or(CouldNotFindInitialMapMilitary {
                            required_player_count: map_choice.player_count(),
                            found_player_count: num_players,
                        })?;

                    let mut military = id_units(initial_military.clone(), guest_id, &guest.color);

                    remaining_guests_with_militaries.append(&mut military);
                }

                let host_units = id_units(
                    initial_militaries.first_player_military,
                    &lobby.host_id,
                    &lobby.host.color,
                );

                let first_guest_units = id_units(
                    initial_militaries.second_player_military,
                    &first_guest_id,
                    &first_guest.color,
                );

                let units: Vec<(UnitId, Located<UnitModel>)> = vec![
                    vec![host_units, first_guest_units].concat().to_vec(),
                    remaining_guests_with_militaries,
                ]
                .concat()
                .to_vec();

                let mut unit_hashmap: HashMap<UnitId, Located<UnitModel>> = HashMap::new();

                for (unit_id, unit) in units {
                    unit_hashmap.insert(unit_id, unit);
                }

                let game = Game {
                    host: lobby.host,
                    host_id: lobby.host_id.clone(),

                    first_guest: first_guest.clone(),
                    first_guest_id: first_guest_id.clone(),

                    remaining_guests: rest.to_vec(),
                    units: unit_hashmap,
                    map,
                };

                Ok(game)
            }
        }
    }

    pub fn dimensions(&self) -> (u8, u8) {
        self.map.dimensions()
    }
}
