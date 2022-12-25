use shared::lobby::{Lobby, LobbyId};
use shared::rng::{RandGen, RandSeed};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct Lobbies {
    lobbies: HashMap<LobbyId, Lobby>,
    random_seed: RandSeed,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Lobbies {
    pub fn init(random_seed: RandSeed) -> Lobbies {
        Lobbies {
            lobbies: HashMap::new(),
            random_seed,
        }
    }

    pub fn get_lobby(&self, id: LobbyId) -> Option<Lobby> {
        self.lobbies.get(&id).cloned()
    }

    pub fn upsert(&mut self, id: LobbyId, lobby: Lobby) {
        self.lobbies.insert(id, lobby);
    }

    pub fn new_lobby(&mut self, lobby: Lobby) -> LobbyId {
        let mut rand_gen = RandGen::from_seed(self.random_seed.clone());

        let new_id: LobbyId = LobbyId::new(&mut rand_gen);

        self.upsert(new_id.clone(), lobby);

        let new_seed: RandSeed = RandSeed::next(&mut rand_gen);

        self.random_seed = new_seed;

        new_id
    }
}

#[cfg(test)]
mod test_lobbies {
    use crate::lobbies::Lobbies;
    use shared::id::Id;
    use shared::lobby::Lobby;
    use shared::name::Name;
    use shared::player::Player;
    use shared::rng::{RandGen, RandSeed};
    use shared::team_color::TeamColor;

    #[test]
    fn can_join_lobby() {
        let mut rng = RandGen::test();

        let mut lobbies = Lobbies::init(RandSeed::next(&mut rng));

        let host = Player::new(Name::new("host"), TeamColor::Red);
        let host_id = Id::new(&mut rng);

        let lobby_id = lobbies.new_lobby(Lobby::init(host_id.clone(), host.clone()));

        let guest = Player::new(Name::new("guest"), TeamColor::Blue);
        let guest_id = Id::new(&mut rng);

        let lobby = lobbies
            .get_lobby(lobby_id.clone())
            .unwrap()
            .add_guest(guest_id.clone(), guest.clone())
            .unwrap_or_else(|_| panic!("Lobby does not exist"))
            .clone();

        lobbies.upsert(lobby_id.clone(), lobby);

        let mut players: Vec<(Id, Player)> = lobbies
            .get_lobby(lobby_id)
            .unwrap()
            .players()
            .into_iter()
            .collect();

        players.sort_by_key(|(_, p)| p.name.clone());

        let mut expectation = vec![(guest_id, guest), (host_id, host)];

        expectation.sort_by_key(|(_, p)| p.name.clone());

        assert_eq!(players, expectation)
    }
}
