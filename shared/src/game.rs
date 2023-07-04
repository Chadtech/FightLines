pub mod action;
pub mod day;
pub mod event;
pub mod mobility;
pub mod replenishment;
pub mod unit_index;

use crate::facing_direction::FacingDirection;
use crate::game::action::Action;
use crate::game::day::Time;
use crate::game::event::Event;
use crate::game::unit_index::Indexes;
use crate::id::Id;
use crate::lobby::{Lobby, LobbyId};
use crate::located::Located;
use crate::map::Map;
use crate::owner::Owned;
use crate::player::Player;
use crate::rng::{RandGen, RandSeed};
use crate::team_color::TeamColor;
use crate::unit::{Place, Unit, UnitId};
use crate::{located, unit};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GameId {
    GameId(Id),
    Dev(DevGameId),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DevGameId {
    DisplayTest,
    ReplenishTest,
    ArrowTest,
    GamePlayTest,
}

impl ToString for GameId {
    fn to_string(&self) -> String {
        match self {
            GameId::GameId(id) => id.to_string(),
            GameId::Dev(dev_id) => dev_id.to_string(),
        }
    }
}

impl ToString for DevGameId {
    fn to_string(&self) -> String {
        match self {
            DevGameId::DisplayTest => DISPLAY_TEST.to_string(),
            DevGameId::ReplenishTest => REPLENISH_TEST.to_string(),
            DevGameId::ArrowTest => ARROW_TEST.to_string(),
            DevGameId::GamePlayTest => GAME_PLAY_TEST.to_string(),
        }
    }
}

impl From<DevGameId> for GameId {
    fn from(value: DevGameId) -> Self {
        GameId::Dev(value)
    }
}

impl From<&DevGameId> for GameId {
    fn from(value: &DevGameId) -> Self {
        GameId::Dev(value.clone())
    }
}

pub const ALL_DEV_IDS: &[DevGameId] = &[
    DevGameId::DisplayTest,
    DevGameId::ReplenishTest,
    DevGameId::ArrowTest,
    DevGameId::GamePlayTest,
];

impl GameId {
    pub fn from_lobby_id(lobby_id: LobbyId) -> GameId {
        GameId::GameId(lobby_id.ambiguate())
    }

    pub fn from_string(s: String) -> Option<GameId> {
        if s == DISPLAY_TEST {
            return Some(DevGameId::DisplayTest.into());
        }

        if s == REPLENISH_TEST {
            return Some(DevGameId::ReplenishTest.into());
        }

        if s == ARROW_TEST {
            return Some(DevGameId::ArrowTest.into());
        }

        if s == GAME_PLAY_TEST {
            return Some(DevGameId::GamePlayTest.into());
        }

        Id::from_string(s, false).map(GameId::GameId)
    }

    pub fn is_dev(&self) -> bool {
        match self {
            GameId::GameId(_) => false,
            GameId::Dev(_) => true,
        }
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
    pub indexes: Indexes,
    pub map: Map,
    pub turn_number: u32,
    pub turns_changes: Vec<Change>,
    pub prev_turns_events: Vec<Event>,
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
pub enum Change {
    NameUnit { name: String, unit_id: UnitId },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Guest {
    player: Player,
    visibility: HashSet<Located<()>>,
    turn: Turn,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Military(HashMap<UnitId, Located<Owned<Unit>>>);

#[derive(Debug)]
pub enum FromLobbyError {
    NotEnoughPlayers,
    CouldNotFindInitialMapMilitary {
        required_player_count: u8,
        found_player_count: u8,
    },
}

const DISPLAY_TEST: &str = "display-test";
const REPLENISH_TEST: &str = "replenish-test";
const ARROW_TEST: &str = "arrow-test";
const GAME_PLAY_TEST: &str = "game-play-test";

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Game {
    pub fn get_mut_unit(&mut self, unit_id: &UnitId) -> Option<&mut unit::Model> {
        self.indexes.by_id.get_mut(unit_id)
    }
    pub fn get_map(&self) -> &Map {
        &self.map
    }
    pub fn get_unit(&self, unit_id: &UnitId) -> Option<&unit::Model> {
        self.indexes.by_id.get(unit_id)
    }
    pub fn get_units_by_transport(&self, unit_id: &UnitId) -> Option<&Vec<(UnitId, unit::Model)>> {
        self.indexes.by_transport.get(unit_id)
    }
    pub fn get_units_by_player_id(&self, player_id: &Id) -> Option<&Vec<(UnitId, unit::Model)>> {
        self.indexes.by_player.get(player_id)
    }
    pub fn transport_index(&self) -> &unit_index::by_transport::Index {
        &self.indexes.by_transport
    }

    pub fn day(&self) -> Time {
        Time::from_turn(self.turn_number)
    }
    pub fn get_rideable_units_by_location(
        &self,
        owner_id: &Id,
        carrying_unit: &Unit,
        mouse_loc: &Located<()>,
    ) -> Option<Vec<(UnitId, unit::Model)>> {
        match self.indexes.get_units_by_location(mouse_loc) {
            Some(units) => {
                let rideable_units = units
                    .iter()
                    .filter_map(|(rideable_unit_id, _, possibly_rideable_unit)| {
                        if possibly_rideable_unit.unit.can_carry(carrying_unit)
                            && &possibly_rideable_unit.owner == owner_id
                        {
                            Some((rideable_unit_id.clone(), possibly_rideable_unit.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(UnitId, unit::Model)>>();

                if rideable_units.is_empty() {
                    None
                } else {
                    Some(rideable_units)
                }
            }
            None => None,
        }
    }
    pub fn get_supply_crates_by_location(
        &self,
        mouse_loc: &Located<()>,
    ) -> Option<Vec<(UnitId, unit::Model)>> {
        match self.indexes.get_units_by_location(mouse_loc) {
            Some(units) => {
                let supply_crates = units
                    .iter()
                    .filter_map(|(unit_id, _, possibly_supply_crate)| {
                        if possibly_supply_crate.unit.is_supply_crate() {
                            Some((unit_id.clone(), possibly_supply_crate.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(UnitId, unit::Model)>>();

                if supply_crates.is_empty() {
                    None
                } else {
                    Some(supply_crates)
                }
            }
            None => None,
        }
    }
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

    pub fn take_changes(&mut self, changes: &mut Vec<Change>) {
        self.turns_changes.append(changes);
    }

    pub fn advance_turn(&mut self, seed: RandSeed) -> Result<bool, String> {
        let mut rng = RandGen::from_seed(seed);
        let mut player_moves: Vec<(Id, Vec<Action>)> = match &mut self.all_players_turns() {
            Ok(moves) => {
                let mut src_moves: Vec<(Id, Vec<Action>)> = moves.clone();
                let mut ret_moves: Vec<(Id, Vec<Action>)> = Vec::new();

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

        self.turn_number += 1;
        self.process_changes();
        // self.process_outcomes(outcomes.clone())?;

        let event_rand_seed: RandSeed = RandSeed::next(&mut rng);
        let event::ProcessedTurn { events, .. } = event::process_turn(
            event_rand_seed,
            &mut player_moves,
            &mut self.indexes,
            &self.map,
        );

        self.prev_turns_events = events;
        self.indexes.by_location = unit_index::by_location::make(&self.indexes.by_id);
        self.indexes.by_player = unit_index::by_player::make(&self.indexes.by_id);
        self.indexes.by_transport = unit_index::by_transport::make(&self.indexes.by_id);
        self.host_visibility =
            calculate_player_visibility(&self.host_id, &self.map, &self.indexes.by_id);
        self.first_guest_visibility =
            calculate_player_visibility(&self.first_guest_id, &self.map, &self.indexes.by_id);
        self.hosts_turn = Turn::Waiting;
        self.first_guests_turn = Turn::Waiting;

        for (guest_id, guest) in &mut self.remaining_guests {
            guest.visibility =
                calculate_player_visibility(guest_id, &self.map, &self.indexes.by_id);
            guest.turn = Turn::Waiting;
        }

        Ok(true)
    }

    fn process_changes(&mut self) {
        for change in &mut self.turns_changes {
            match change {
                Change::NameUnit { unit_id, name } => {
                    if let Some(unit_model) = self.indexes.by_id.get_mut(unit_id) {
                        if unit_model.name.is_none() {
                            unit_model.name = Some(name.clone());
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

    pub fn num_players(&self) -> usize {
        2 + self.remaining_guests.len()
    }
}

pub fn calculate_player_visibility(
    player_id: &Id,
    map: &Map,
    units: &unit_index::by_id::Index,
) -> HashSet<Located<()>> {
    let mut visible_spots = HashSet::new();

    let player_id = player_id.clone();

    for unit_model in units.values() {
        if unit_model.owner.clone() == player_id && !unit_model.unit.is_supply_crate() {
            if let Place::OnMap(loc) = &unit_model.place {
                let budget = unit_model.unit.visibility_budget();

                let mut search: HashMap<Located<()>, f32> = HashMap::new();

                search.insert(located::unit(loc.x, loc.y), budget);

                while !search.is_empty() {
                    for (search_loc, spot_budget) in search.clone().into_iter() {
                        visible_spots.insert(search_loc.clone());
                        search.remove(&search_loc);

                        let x = search_loc.x;
                        let y = search_loc.y;

                        if y > 0 {
                            let north_loc = located::unit(x, y - 1);
                            let north_tile = map.get_tile(&north_loc);

                            let cost = north_tile.visibility_cost();

                            let budget_at_tile = spot_budget - cost;

                            if budget_at_tile > 0.0 {
                                search
                                    .entry(north_loc)
                                    .and_modify(|existing_budget| {
                                        if budget_at_tile > *existing_budget {
                                            *existing_budget = budget_at_tile;
                                        }
                                    })
                                    .or_insert(budget_at_tile);
                            }
                        }

                        if x > 0 {
                            let west_loc = located::unit(x - 1, y);
                            let west_tile = map.get_tile(&west_loc);

                            let cost = west_tile.visibility_cost();

                            let budget_at_tile = spot_budget - cost;

                            if budget_at_tile > 0.0 {
                                search
                                    .entry(west_loc)
                                    .and_modify(|existing_budget| {
                                        if budget_at_tile > *existing_budget {
                                            *existing_budget = budget_at_tile;
                                        }
                                    })
                                    .or_insert(budget_at_tile);
                            }
                        }

                        let south_loc = located::unit(x, y + 1);
                        let south_tile = map.get_tile(&south_loc);

                        let cost = south_tile.visibility_cost();

                        let budget_at_tile = spot_budget - cost;

                        if budget_at_tile > 0.0 {
                            search
                                .entry(south_loc)
                                .and_modify(|existing_budget| {
                                    if budget_at_tile > *existing_budget {
                                        *existing_budget = budget_at_tile;
                                    }
                                })
                                .or_insert(budget_at_tile);
                        }

                        let east_loc = located::unit(x + 1, y);
                        let east_tile = map.get_tile(&east_loc);

                        let cost = east_tile.visibility_cost();

                        let budget_at_tile = spot_budget - cost;

                        if budget_at_tile > 0.0 {
                            search
                                .entry(east_loc)
                                .and_modify(|existing_budget| {
                                    if budget_at_tile > *existing_budget {
                                        *existing_budget = budget_at_tile;
                                    }
                                })
                                .or_insert(budget_at_tile);
                        }
                    }
                }
            }
        }
    }

    visible_spots
}

pub struct GameInitFlags<'a> {
    pub lobby: Lobby,
    pub rng: &'a mut RandGen,
    pub extra_units: Vec<(UnitId, unit::Model)>,
}

impl GameInitFlags<'_> {
    pub fn new(lobby: Lobby, rng: &mut RandGen) -> GameInitFlags {
        GameInitFlags {
            lobby,
            rng,
            extra_units: vec![],
        }
    }

    pub fn with_extra_units(&mut self, more_extra_units: &mut Vec<(UnitId, unit::Model)>) {
        self.extra_units.append(more_extra_units);
    }
}

impl TryFrom<GameInitFlags<'_>> for Game {
    type Error = FromLobbyError;

    fn try_from(params: GameInitFlags) -> Result<Self, Self::Error> {
        let GameInitFlags {
            lobby,
            rng,
            extra_units,
        } = params;

        let num_players = lobby.num_players();
        let guests: Vec<(Id, Player)> = lobby.guests.into_iter().collect();

        let map_choice = lobby.map_choice;

        let map = map_choice.to_map();
        let initial_units = map_choice.initial_units();

        match guests.split_first() {
            None => Err(FromLobbyError::NotEnoughPlayers),
            Some((first, rest)) => {
                let (first_guest_id, first_guest) = first;

                let mut id_units = |units: Vec<Located<(FacingDirection, Unit)>>,
                                    owner_id: &Id,
                                    color: &TeamColor|
                 -> Vec<(UnitId, unit::Model)> {
                    let mut units_with_ids: Vec<(UnitId, unit::Model)> = vec![];

                    for located_unit in units {
                        let unit_id = UnitId::new(rng);

                        let (facing, unit) = located_unit.value;

                        let place: Place = Place::OnMap(Located {
                            x: located_unit.x,
                            y: located_unit.y,
                            value: facing,
                        });

                        let new_unit: unit::Model = unit::Model::new(unit, owner_id, place, color);

                        units_with_ids.push((unit_id, new_unit));
                    }

                    units_with_ids
                };

                let mut remaining_guests_with_militaries: Vec<(UnitId, unit::Model)> = vec![];

                for (index, (guest_id, guest)) in rest.iter().enumerate() {
                    let initial_military = initial_units
                        .rest_players_militatries
                        .get(index)
                        .ok_or(FromLobbyError::CouldNotFindInitialMapMilitary {
                            required_player_count: map_choice.player_count(),
                            found_player_count: num_players,
                        })?;

                    let mut military = id_units(initial_military.clone(), guest_id, &guest.color);

                    remaining_guests_with_militaries.append(&mut military);
                }

                let host_units = id_units(
                    initial_units.first_player_military,
                    &lobby.host_id,
                    &lobby.host.color,
                );

                let first_guest_units = id_units(
                    initial_units.second_player_military,
                    first_guest_id,
                    &first_guest.color,
                );

                let units: Vec<(UnitId, unit::Model)> = vec![
                    vec![host_units, first_guest_units, extra_units]
                        .concat()
                        .to_vec(),
                    remaining_guests_with_militaries,
                ]
                .concat()
                .to_vec();

                let indexes = Indexes::make(units);

                let host_id = lobby.host_id.clone();

                let remaining_guests: Vec<(Id, Guest)> = rest
                    .iter()
                    .map(|(guest_id, guest_player)| {
                        let guest = Guest {
                            player: guest_player.clone(),
                            visibility: calculate_player_visibility(guest_id, &map, &indexes.by_id),
                            turn: Turn::Waiting,
                        };

                        (guest_id.clone(), guest)
                    })
                    .collect();

                let host_visibility = calculate_player_visibility(&host_id, &map, &indexes.by_id);

                let first_guest_visibility =
                    calculate_player_visibility(first_guest_id, &map, &indexes.by_id);

                let game = Game {
                    host: lobby.host,
                    host_id,
                    host_visibility,
                    hosts_turn: Turn::Waiting,
                    first_guest: first_guest.clone(),
                    first_guest_id: first_guest_id.clone(),
                    first_guest_visibility,
                    first_guests_turn: Turn::Waiting,
                    remaining_guests,
                    indexes,
                    map,
                    turn_number: 0,
                    turns_changes: Vec::new(),
                    prev_turns_events: vec![],
                };

                Ok(game)
            }
        }
    }
}
