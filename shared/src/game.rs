pub mod action;
pub mod day;
pub mod outcome;
pub mod unit_index;

use crate::facing_direction::FacingDirection;
use crate::game::action::Action;
use crate::game::day::Time;
use crate::game::outcome::Outcome;
use crate::game::unit_index::Indices;
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
use std::collections::hash_map::Iter;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GameId {
    GameId(Id),
    DisplayTest,
}

impl ToString for GameId {
    fn to_string(&self) -> String {
        match self {
            GameId::GameId(id) => id.to_string(),
            GameId::DisplayTest => DISPLAY_TEST.to_string(),
        }
    }
}

impl GameId {
    pub fn from_lobby_id(lobby_id: LobbyId) -> GameId {
        GameId::GameId(lobby_id.ambiguate())
    }

    pub fn from_string(s: String) -> Option<GameId> {
        if s == DISPLAY_TEST {
            return Some(GameId::DisplayTest);
        }

        Id::from_string(s, false).map(GameId::GameId)
    }

    pub fn is_dev(&self) -> bool {
        match self {
            GameId::GameId(_) => false,
            GameId::DisplayTest => true,
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
    pub indices: Indices,
    pub map: Map,
    pub turn_number: u32,
    pub turns_changes: Vec<Change>,
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

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Game {
    pub fn get_mut_unit(&mut self, unit_id: &UnitId) -> Option<&mut unit::Model> {
        self.indices.by_id.get_mut(unit_id)
    }
    pub fn get_unit(&self, unit_id: &UnitId) -> Option<&unit::Model> {
        self.indices.by_id.get(unit_id)
    }
    pub fn units_by_location(
        &self,
    ) -> Iter<'_, Located<()>, Vec<(UnitId, FacingDirection, unit::Model)>> {
        self.indices.by_location.iter()
    }
    pub fn get_units_by_transport(&self, unit_id: &UnitId) -> Option<&Vec<(UnitId, unit::Model)>> {
        self.indices.by_transport.get(unit_id)
    }
    pub fn get_units_by_player_id(&self, player_id: &Id) -> Option<&Vec<(UnitId, unit::Model)>> {
        self.indices.by_player.get(player_id)
    }
    pub fn transport_index(&self) -> &unit_index::by_transport::Index {
        &self.indices.by_transport
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
        match self.indices.by_location.get(mouse_loc) {
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
        match self.indices.by_location.get(mouse_loc) {
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

    pub fn outcomes(&mut self, player_moves: &mut Vec<(Id, Vec<Action>)>) -> Vec<Outcome> {
        vec![self.consume_supplies(), Outcome::from_actions(player_moves)].concat()
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

        let outcomes = self.outcomes(&mut player_moves);

        self.turn_number += 1;
        self.process_changes();
        self.process_outcomes(outcomes.clone())?;

        self.prev_outcomes = outcomes;
        self.indices.by_location = unit_index::by_location::make(&self.indices.by_id);
        self.indices.by_player = unit_index::by_player::make(&self.indices.by_id);
        self.indices.by_transport = unit_index::by_transport::make(&self.indices.by_id);
        self.host_visibility =
            calculate_player_visibility(&self.host_id, &self.map, &self.indices.by_id);
        self.first_guest_visibility =
            calculate_player_visibility(&self.first_guest_id, &self.map, &self.indices.by_id);
        self.hosts_turn = Turn::Waiting;
        self.first_guests_turn = Turn::Waiting;

        for (guest_id, guest) in &mut self.remaining_guests {
            guest.visibility =
                calculate_player_visibility(guest_id, &self.map, &self.indices.by_id);
            guest.turn = Turn::Waiting;
        }

        Ok(true)
    }

    fn process_changes(&mut self) {
        for change in &mut self.turns_changes {
            match change {
                Change::NameUnit { unit_id, name } => {
                    if let Some(unit_model) = self.indices.by_id.get_mut(unit_id) {
                        if unit_model.name.is_none() {
                            unit_model.name = Some(name.clone());
                        }
                    }
                }
            }
        }
    }

    fn consume_supplies(&self) -> Vec<Outcome> {
        let mut outcomes = Vec::new();

        for (unit_id, unit_model) in self.indices.by_id.iter() {
            if let Some(supply_cost) = unit_model.unit.baseline_supply_cost() {
                let supply_cost = supply_cost.ceil() as i16;

                let new_supplies = unit_model.supplies - supply_cost;

                let outcome = if new_supplies < 0 {
                    Outcome::Perished {
                        unit_id: unit_id.clone(),
                    }
                } else {
                    Outcome::ConsumedSupplies {
                        unit_id: unit_id.clone(),
                        supplies: supply_cost,
                    }
                };

                outcomes.push(outcome);
            }
        }

        outcomes
    }

    pub fn process_outcomes(&mut self, outcomes: Vec<Outcome>) -> Result<(), String> {
        for outcome in outcomes {
            match outcome {
                Outcome::Traveled { unit_id, path } => {
                    if let Some(loc_dir) = path.last_pos() {
                        let facing_dir = match self.indices.position_of_unit_or_transport(&unit_id)
                        {
                            Ok(facing_dir_loc) => facing_dir_loc.value,
                            Err(msg) => return Err(msg),
                        };

                        if let Some(unit) = self.get_mut_unit(&unit_id) {
                            unit.supplies -= path.supply_cost(&unit.unit);

                            let new_facing_dir =
                                FacingDirection::from_directions(path.clone().to_directions())
                                    .unwrap_or(facing_dir);

                            unit.place = Place::OnMap(loc_dir.with_value(new_facing_dir));
                        }
                    }
                }
                Outcome::NamedUnit { unit_id, name } => {
                    if let Some(unit) = self.get_mut_unit(&unit_id) {
                        unit.name = Some(name);
                    }
                }
                Outcome::LoadedInto {
                    unit_id,
                    loaded_into,
                    path,
                    ..
                } => {
                    if let Some(unit) = self.get_mut_unit(&unit_id) {
                        unit.supplies -= path.supply_cost(&unit.unit);
                        unit.place = Place::InUnit(loaded_into.clone());
                    }
                }
                Outcome::Perished { unit_id } => {
                    self.indices.by_id.delete(&unit_id);

                    if self.indices.by_transport.contains(&unit_id) {
                        let facing_dir_loc =
                            match self.indices.position_of_unit_or_transport(&unit_id) {
                                Ok(l) => l,
                                Err(msg) => {
                                    return Err(msg);
                                }
                            };

                        for (_, cargo_model) in self.indices.by_transport.get_mut(&unit_id).unwrap()
                        {
                            cargo_model.place = Place::OnMap(facing_dir_loc.clone());
                        }
                    }
                }
                Outcome::ConsumedSupplies { unit_id, supplies } => {
                    if let Some(unit) = self.get_mut_unit(&unit_id) {
                        unit.supplies -= supplies;
                    }
                }
                Outcome::PickUp {
                    unit_id,
                    cargo_id,
                    path,
                } => {
                    if self.get_unit(&cargo_id).is_none()
                        || self.get_unit(&unit_id).is_none()
                        || path.last_pos().is_none()
                    {
                        return Ok(());
                    }

                    let facing_dir = match self.indices.position_of_unit_or_transport(&unit_id) {
                        Ok(facing_dir_loc) => facing_dir_loc.value,
                        Err(msg) => return Err(msg),
                    };

                    let cargo_unit = match self.get_mut_unit(&cargo_id) {
                        Some(cargo) => cargo,
                        None => {
                            return Ok(());
                        }
                    };

                    cargo_unit.place = Place::InUnit(unit_id.clone());

                    let unit = match self.get_mut_unit(&unit_id) {
                        Some(u) => u,
                        None => {
                            return Ok(());
                        }
                    };

                    unit.supplies -= path.supply_cost(&unit.unit);

                    if let Some(loc_dir) = path.last_pos() {
                        let new_facing_dir =
                            FacingDirection::from_directions(path.clone().to_directions())
                                .unwrap_or(facing_dir);

                        unit.place = Place::OnMap(loc_dir.with_value(new_facing_dir));
                    }
                }
                Outcome::Placed { cargo_unit_loc, .. } => {
                    let (facing_dir, unit_id) = cargo_unit_loc.value.clone();

                    let unit = match self.get_mut_unit(&unit_id) {
                        Some(u) => u,
                        None => {
                            return Ok(());
                        }
                    };

                    unit.place = Place::OnMap(cargo_unit_loc.with_value(facing_dir));
                }
            }
        }

        Ok(())
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

    pub fn position_of_unit_or_transport(
        &self,
        unit_id: &UnitId,
    ) -> Result<Located<FacingDirection>, String> {
        self.indices.position_of_unit_or_transport(unit_id)
    }

    pub fn get_units_mobility(&self, unit_id: &UnitId) -> Result<HashSet<Located<()>>, String> {
        let maybe_unit = self.get_unit(unit_id);

        match maybe_unit {
            None => Err("unit not found when getting units mobility".to_string()),
            Some(unit_model) => {
                let mut mobility = HashSet::new();

                let loc = self.position_of_unit_or_transport(unit_id)?;
                let budget = unit_model.unit.mobility_budget();

                let mut search: HashMap<Located<()>, f32> = HashMap::new();

                if !unit_model.unit.is_supply_crate() {
                    search.insert(located::unit(loc.x, loc.y), budget);
                }

                let map = &self.map;

                while !search.is_empty() {
                    for (search_loc, spot_budget) in search.clone().into_iter() {
                        mobility.insert(search_loc.clone());
                        search.remove(&search_loc);

                        let x = search_loc.x;
                        let y = search_loc.y;

                        if y > 0 {
                            let north_loc = located::unit(x, y - 1);
                            let north_tile = map.get_tile(&north_loc);

                            let cost = north_tile.mobility_cost(&unit_model.unit);

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

                            let cost = west_tile.mobility_cost(&unit_model.unit);

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

                        let cost = south_tile.mobility_cost(&unit_model.unit);

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

                        let cost = east_tile.mobility_cost(&unit_model.unit);

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

    pub fn get_units_by_location(
        &self,
        key: &Located<()>,
    ) -> Option<&Vec<(UnitId, FacingDirection, unit::Model)>> {
        self.indices.by_location.get(key)
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

impl TryFrom<(Lobby, &mut RandGen)> for Game {
    type Error = FromLobbyError;

    fn try_from(params: (Lobby, &mut RandGen)) -> Result<Self, Self::Error> {
        let (lobby, rng) = params;

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

                        let new_unit: unit::Model = unit::Model {
                            unit: unit.clone(),
                            owner: owner_id.clone(),
                            place,
                            color: color.clone(),
                            name: None,
                            supplies: unit.max_supplies(),
                        };

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
                    vec![host_units, first_guest_units].concat().to_vec(),
                    remaining_guests_with_militaries,
                ]
                .concat()
                .to_vec();

                let mut unit_hashmap: HashMap<UnitId, unit::Model> = HashMap::new();

                for (unit_id, unit) in units {
                    unit_hashmap.insert(unit_id, unit);
                }

                let host_id = lobby.host_id.clone();

                let units_by_id = unit_index::by_id::Index::from_hash_map(unit_hashmap);

                let remaining_guests: Vec<(Id, Guest)> = rest
                    .iter()
                    .map(|(guest_id, guest_player)| {
                        let guest = Guest {
                            player: guest_player.clone(),
                            visibility: calculate_player_visibility(guest_id, &map, &units_by_id),
                            turn: Turn::Waiting,
                        };

                        (guest_id.clone(), guest)
                    })
                    .collect();

                let by_location_index = unit_index::by_location::make(&units_by_id);
                let by_player_index = unit_index::by_player::make(&units_by_id);
                let by_transport_index = unit_index::by_transport::make(&units_by_id);

                let host_visibility = calculate_player_visibility(&host_id, &map, &units_by_id);

                let first_guest_visibility =
                    calculate_player_visibility(first_guest_id, &map, &units_by_id);

                let indices = Indices {
                    by_id: units_by_id,
                    by_location: by_location_index,
                    by_player: by_player_index,
                    by_transport: by_transport_index,
                };

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
                    indices,
                    map,
                    turn_number: 0,
                    turns_changes: Vec::new(),
                    prev_outcomes: Vec::new(),
                };

                Ok(game)
            }
        }
    }
}
