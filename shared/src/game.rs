use crate::arrow::Arrow;
use crate::direction::Direction;
use crate::facing_direction::FacingDirection;
use crate::game::FromLobbyError::CouldNotFindInitialMapMilitary;
use crate::id::Id;
use crate::lobby::{Lobby, LobbyId};
use crate::located::Located;
use crate::map::{Map, MapOpt};
use crate::owner::Owned;
use crate::player::Player;
use crate::point::Point;
use crate::rng::{RandGen, RandSeed};
use crate::team_color::TeamColor;
use crate::unit::{Unit, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GameId(Id);

impl ToString for GameId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl GameId {
    pub fn from_lobby_id(lobby_id: LobbyId) -> GameId {
        GameId(lobby_id.ambiguate())
    }

    pub fn from_string(s: String) -> Option<GameId> {
        Id::from_string(s).map(GameId)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Game {
    // host
    pub host: Player,
    pub host_id: Id,
    pub host_visibility: HashSet<Located<()>>,
    pub hosts_turn: Turn,
    // first guest
    pub first_guest: Player,
    pub first_guest_id: Id,
    pub first_guest_visibility: HashSet<Located<()>>,
    pub first_guests_turn: Turn,
    // remaining guests
    pub remaining_guests: Vec<(Id, Guest)>,
    //
    pub units: HashMap<UnitId, UnitModel>,
    pub units_by_location_index: HashMap<Point<u16>, Vec<(UnitId, UnitModel)>>,
    pub units_by_player_index: HashMap<Id, Vec<(UnitId, UnitModel)>>,
    pub map: Map,
    pub turn_number: u32,
    pub prev_outcomes: Vec<Outcome>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Turn {
    Waiting,
    Turn { moves: Vec<Action> },
}

impl Turn {
    pub fn is_waiting(&self) -> bool {
        match self {
            Turn::Waiting => true,
            Turn::Turn { .. } => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Action {
    Traveled {
        unit_id: UnitId,
        path: Vec<Located<Direction>>,
        arrows: Vec<(Direction, Arrow)>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Traveled {
        unit_id: UnitId,
        path: Vec<Located<Direction>>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct UnitModel {
    pub unit: Unit,
    pub place: UnitPlace,
    pub owner: Id,
    pub color: TeamColor,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]

pub enum UnitPlace {
    OnMap(Located<FacingDirection>),
    InUnit(UnitId),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Guest {
    player: Player,
    visibility: HashSet<Located<()>>,
    turn: Turn,
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
    pub fn all_players_turns(&self) -> Result<Vec<(Id, Vec<Action>)>, String> {
        let mut player_moves: Vec<(Id, Vec<Action>)> = Vec::new();

        match &self.hosts_turn {
            Turn::Waiting => {
                return Err("waiting on the host to submit their turn".to_string());
            }
            Turn::Turn { moves } => {
                player_moves.push((self.host_id.clone(), moves.clone()));
            }
        }

        match &self.first_guests_turn {
            Turn::Waiting => {
                return Err("waiting on the first guest to submit their turn".to_string());
            }
            Turn::Turn { moves } => {
                player_moves.push((self.first_guest_id.clone(), moves.clone()));
            }
        }

        for (n, (guest_id, guest)) in self.remaining_guests.iter().enumerate() {
            match &guest.turn {
                Turn::Waiting => {
                    return Err(format!("waiting on guest {} to submit their turn", n + 2));
                }
                Turn::Turn { moves } => player_moves.push((guest_id.clone(), moves.clone())),
            }
        }

        Ok(player_moves)
    }
    pub fn advance_turn(&mut self, seed: RandSeed) -> Result<bool, String> {
        let mut rng = RandGen::from_seed(seed);
        let mut player_moves: Vec<(Id, Vec<Action>)> = match &mut self.all_players_turns() {
            Ok(moves) => {
                let mut src_moves = moves.clone();
                let mut ret_moves = Vec::new();

                while !src_moves.is_empty() {
                    let max_index = src_moves.len() - 1;
                    let index = if max_index == 0 {
                        0
                    } else {
                        rng.gen::<usize>(0, max_index)
                    };

                    let players_moves = src_moves[index].clone();

                    ret_moves.push(players_moves);

                    src_moves.remove(index);
                }

                ret_moves
            }
            Err(_) => {
                return Ok(false);
            }
        };

        let mut outcomes = Vec::new();

        let mut player_index = 0;
        let mut cont = true;

        while cont {
            if let Some((_player_id, actions)) = player_moves.get_mut(player_index) {
                if let Some(first) = actions.first() {
                    match first {
                        Action::Traveled { unit_id, path, .. } => {
                            outcomes.push(Outcome::Traveled {
                                unit_id: unit_id.clone(),
                                path: path.clone(),
                            });
                        }
                    }

                    actions.remove(0);
                }
            }

            player_index = (player_index + 1) % player_moves.len();

            cont = !player_moves.iter().all(|(_, m)| m.is_empty());
        }

        self.turn_number += 1;
        self.consume_outcomes(outcomes.clone());
        self.prev_outcomes = outcomes.clone();
        self.units_by_location_index = index_units_by_location(&self.units);
        self.units_by_player_index = index_units_by_player(&self.units);
        self.host_visibility = calculate_player_visibility(&self.host_id, &self.map, &self.units);
        self.first_guest_visibility =
            calculate_player_visibility(&self.first_guest_id, &self.map, &self.units);
        self.hosts_turn = Turn::Waiting;
        self.first_guests_turn = Turn::Waiting;

        for (guest_id, guest) in &mut self.remaining_guests {
            guest.visibility = calculate_player_visibility(guest_id, &self.map, &self.units);
            guest.turn = Turn::Waiting;
        }

        Ok(true)
    }
    pub fn consume_outcomes(&mut self, outcomes: Vec<Outcome>) {
        for outcome in outcomes {
            match outcome {
                Outcome::Traveled { unit_id, path } => {
                    if let Some(loc_dir) = path.last() {
                        if let Some(unit) = self.units.get_mut(&unit_id) {
                            let new_facing_dir = FacingDirection::from_directions(
                                path.clone()
                                    .into_iter()
                                    .map(|loc_dir| loc_dir.value)
                                    .collect(),
                            )
                            .unwrap_or_else(|| {
                                match unit.place.clone() {
                                    UnitPlace::OnMap(loc_facing_dir) => loc_facing_dir.value,
                                    UnitPlace::InUnit(_) => {
                                        // This is a value of last resort after we have
                                        // exhausted all other information. This
                                        // code may be unreachable
                                        // - Chad Jan 8 2023
                                        FacingDirection::Right
                                    }
                                }
                            });

                            unit.place = UnitPlace::OnMap(Located {
                                x: loc_dir.x,
                                y: loc_dir.y,
                                value: new_facing_dir,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn get_turn(&self, player_id: Id) -> Result<Turn, String> {
        if player_id == self.host_id {
            return Ok(self.hosts_turn.clone());
        } else if player_id == self.first_guest_id {
            return Ok(self.first_guests_turn.clone());
        } else {
            for (guest_id, guest) in self.remaining_guests.iter() {
                if guest_id == &player_id {
                    return Ok(guest.turn.clone());
                }
            }
        }

        Err("Could not find player when getting turn".to_string())
    }

    pub fn set_turn(&mut self, player_id: Id, moves: Vec<Action>) -> Result<(), String> {
        if player_id == self.host_id {
            match self.hosts_turn {
                Turn::Waiting => {
                    self.hosts_turn = Turn::Turn { moves };

                    Ok(())
                }
                Turn::Turn { .. } => Err("Host has already moved this turn".to_string()),
            }
        } else if player_id == self.first_guest_id {
            match self.first_guests_turn {
                Turn::Waiting => {
                    self.first_guests_turn = Turn::Turn { moves };

                    Ok(())
                }
                Turn::Turn { .. } => Err("first guest has already moved this turn".to_string()),
            }
        } else {
            for (index, (guest_id, guest)) in self.remaining_guests.iter().enumerate() {
                if &player_id == guest_id {
                    return match guest.turn {
                        Turn::Waiting => {
                            self.remaining_guests[index].1.turn = Turn::Turn { moves };

                            Ok(())
                        }
                        Turn::Turn { .. } => Err(format!(
                            "guest number {} has already moved this turn",
                            (index + 2)
                        )),
                    };
                }
            }

            Err(format!(
                "Game does not have guest {}",
                player_id.to_string()
            ))
        }
    }
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
                 -> Vec<(UnitId, UnitModel)> {
                    let mut units_with_ids: Vec<(UnitId, UnitModel)> = vec![];

                    for located_unit in units {
                        let unit_id = UnitId::new(rng);

                        let (facing, unit) = located_unit.value;

                        let place: UnitPlace = UnitPlace::OnMap(Located {
                            x: located_unit.x,
                            y: located_unit.y,
                            value: facing,
                        });

                        let new_unit: UnitModel = UnitModel {
                            unit,
                            owner: owner_id.clone(),
                            place,
                            color: color.clone(),
                        };

                        units_with_ids.push((unit_id, new_unit));
                    }

                    units_with_ids
                };

                let mut remaining_guests_with_militaries: Vec<(UnitId, UnitModel)> = vec![];

                for (index, (guest_id, guest)) in rest.iter().enumerate() {
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
                    first_guest_id,
                    &first_guest.color,
                );

                let units: Vec<(UnitId, UnitModel)> = vec![
                    vec![host_units, first_guest_units].concat().to_vec(),
                    remaining_guests_with_militaries,
                ]
                .concat()
                .to_vec();

                let mut unit_hashmap: HashMap<UnitId, UnitModel> = HashMap::new();

                for (unit_id, unit) in units {
                    unit_hashmap.insert(unit_id, unit);
                }

                let host_id = lobby.host_id.clone();

                let remaining_guests: Vec<(Id, Guest)> = rest
                    .iter()
                    .map(|(guest_id, guest_player)| {
                        let guest = Guest {
                            player: guest_player.clone(),
                            visibility: calculate_player_visibility(guest_id, &map, &unit_hashmap),
                            turn: Turn::Waiting,
                        };

                        (guest_id.clone(), guest)
                    })
                    .collect();

                let game = Game {
                    host: lobby.host,
                    host_id: host_id.clone(),
                    host_visibility: calculate_player_visibility(&host_id, &map, &unit_hashmap),
                    hosts_turn: Turn::Waiting,
                    first_guest: first_guest.clone(),
                    first_guest_id: first_guest_id.clone(),
                    first_guest_visibility: calculate_player_visibility(
                        first_guest_id,
                        &map,
                        &unit_hashmap,
                    ),
                    first_guests_turn: Turn::Waiting,
                    remaining_guests,
                    units_by_location_index: index_units_by_location(&unit_hashmap),
                    units_by_player_index: index_units_by_player(&unit_hashmap),
                    units: unit_hashmap,
                    map,
                    turn_number: 0,
                    prev_outcomes: Vec::new(),
                };

                Ok(game)
            }
        }
    }

    pub fn dimensions(&self) -> (u8, u8) {
        self.map.dimensions()
    }

    pub fn get_players_visibility(&self, player_id: &Id) -> Result<&HashSet<Located<()>>, String> {
        if &self.host_id == player_id {
            return Ok(&self.host_visibility);
        }

        if &self.first_guest_id == player_id {
            return Ok(&self.first_guest_visibility);
        }

        let mut ret_guest_visibility: Result<&HashSet<Located<()>>, String> =
            Err("player not found when finding visibility".to_string());

        for (guest_id, guest) in self.remaining_guests.iter() {
            if guest_id == player_id {
                ret_guest_visibility = Ok(&guest.visibility);
            }
        }

        ret_guest_visibility
    }

    pub fn get_units_mobility(&self, unit_id: &UnitId) -> Result<HashSet<Located<()>>, String> {
        let maybe_unit = self.units.get(unit_id);

        match maybe_unit {
            None => Err("unit not found when getting units mobility".to_string()),
            Some(unit_model) => {
                let mut mobility = HashSet::new();

                if let UnitPlace::OnMap(loc_unit) = &unit_model.place {
                    let x = loc_unit.x;
                    let y = loc_unit.y;

                    let mut index = 0;
                    let unit_range = unit_model.unit.get_mobility_range() - 1;

                    let mut mobility_pre_filter: HashSet<Point<i16>> = HashSet::new();

                    let x = x as i16;
                    let y = y as i16;

                    mobility_pre_filter.insert(Point { x: x + 1, y });
                    mobility_pre_filter.insert(Point { x: x - 1, y });
                    mobility_pre_filter.insert(Point { x, y: y + 1 });
                    mobility_pre_filter.insert(Point { x, y: y - 1 });

                    while index < unit_range {
                        let mut new_points = vec![];
                        for p in mobility_pre_filter.iter() {
                            new_points.push(Point { x: p.x + 1, y: p.y });
                            new_points.push(Point { x: p.x - 1, y: p.y });
                            new_points.push(Point { x: p.x, y: p.y + 1 });
                            new_points.push(Point { x: p.x, y: p.y - 1 });
                        }

                        for p in new_points {
                            mobility_pre_filter.insert(p);
                        }

                        index += 1;
                    }

                    for p in mobility_pre_filter {
                        if p.x >= 0 && p.y >= 0 {
                            let loc = Located {
                                x: p.x as u16,
                                y: p.y as u16,
                                value: (),
                            };

                            mobility.insert(loc);
                        }
                    }
                }

                Ok(mobility)
            }
        }
    }

    pub fn waiting_on_player(&self, player_id: &Id) -> bool {
        let mut has_submitted: bool = false;

        if &self.host_id == player_id {
            has_submitted = self.hosts_turn.is_waiting();
        } else if &self.first_guest_id == player_id {
            has_submitted = self.first_guests_turn.is_waiting();
        } else {
            for (guest_id, guest) in self.remaining_guests.iter() {
                if guest_id == player_id {
                    has_submitted = guest.turn.is_waiting();
                }
            }
        }

        has_submitted
    }

    pub fn get_units_by_location(&self, key: &Point<u16>) -> Option<&Vec<(UnitId, UnitModel)>> {
        self.units_by_location_index.get(key)
    }

    pub fn num_players(&self) -> usize {
        2 + self.remaining_guests.len()
    }
}

fn index_units_by_player(
    units: &HashMap<UnitId, UnitModel>,
) -> HashMap<Id, Vec<(UnitId, UnitModel)>> {
    let mut ret = HashMap::new();

    for (unit_id, unit) in units.iter() {
        let key = unit.owner.clone();

        let val = || (unit_id.clone(), unit.clone());

        let entry = ret.entry(key).or_insert_with(Vec::new);

        entry.push(val());
    }

    ret
}

fn index_units_by_location(
    units: &HashMap<UnitId, UnitModel>,
) -> HashMap<Point<u16>, Vec<(UnitId, UnitModel)>> {
    let mut ret = HashMap::new();

    for (unit_id, unit) in units.iter() {
        if let UnitPlace::OnMap(loc_facing_dir) = unit.place.clone() {
            let key = Point {
                x: loc_facing_dir.x,
                y: loc_facing_dir.y,
            };

            let val = || (unit_id.clone(), unit.clone());

            let entry = ret.entry(key).or_insert_with(Vec::new);

            entry.push(val());
        }
    }

    ret
}

fn calculate_player_visibility(
    player_id: &Id,
    _map: &Map,
    units: &HashMap<UnitId, UnitModel>,
) -> HashSet<Located<()>> {
    let mut visible_spots = HashSet::new();

    for unit in units.values() {
        if let UnitPlace::OnMap(loc) = &unit.place {
            if &unit.owner == player_id {
                if loc.x > 0 {
                    visible_spots.insert(Located {
                        value: (),
                        x: loc.x - 1,
                        y: loc.y,
                    });
                }

                visible_spots.insert(Located {
                    value: (),
                    x: loc.x + 1,
                    y: loc.y,
                });

                if loc.y > 0 {
                    visible_spots.insert(Located {
                        value: (),
                        x: loc.x,
                        y: loc.y - 1,
                    });
                }

                visible_spots.insert(Located {
                    value: (),
                    x: loc.x,
                    y: loc.y + 1,
                });

                visible_spots.insert(Located {
                    value: (),
                    x: loc.x,
                    y: loc.y,
                });
            }
        }
    }

    visible_spots
}
