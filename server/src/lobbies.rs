use shared::id::Id;
use shared::lobby::Lobby;
use shared::rng::{RandGen, RandSeed};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct Lobbies {
    lobbies: HashMap<Id, Lobby>,
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

    pub fn get_lobby(&self, id: Id) -> Option<Lobby> {
        self.lobbies.get(&id).map(|lobby| lobby.clone())
    }

    pub fn upsert(&mut self, id: Id, lobby: Lobby) {
        self.lobbies.insert(id, lobby);
    }

    pub fn new_lobby(&mut self, lobby: Lobby) -> Id {
        let mut rand_gen = RandGen::from_seed(self.random_seed.clone());

        let new_id: Id = Id::new(&mut rand_gen);

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
    use shared::player::Player;
    use shared::rng::{RandGen, RandSeed};

    #[test]
    fn can_join_lobby() {
        let mut rng = RandGen::test();

        let mut lobbies = Lobbies::init(RandSeed::next(&mut rng));

        let host = Player::new(Id::new(&mut rng));

        let lobby_id = lobbies.new_lobby(Lobby::init(host.clone()));

        let guest = Player::new(Id::new(&mut rng));

        let lobby = lobbies
            .get_lobby(lobby_id.clone())
            .unwrap()
            .add_guest(guest.clone())
            .unwrap_or_else(|_| panic!("Lobby does not exist"))
            .clone();

        lobbies.upsert(lobby_id.clone(), lobby);

        let players = lobbies.get_lobby(lobby_id).unwrap().players();

        assert_eq!(players, vec![host, guest])
    }
}
