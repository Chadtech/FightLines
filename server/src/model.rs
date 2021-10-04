use crate::flags::Flags;
use crate::lobby::Lobby;
use crate::setting::Setting;
use rand::{Rng, SeedableRng, StdRng};
use shared::id::Id;
use std::collections::HashMap;
use std::sync::{LockResult, Mutex};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Model {
    pub ip_address: String,
    pub admin_password: String,
    pub port_number: u64,
    pub setting: Setting,
    pub lobbies: Mutex<HashMap<Id, Lobby>>,
    randomness_seed: Mutex<usize>,
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl Model {
    pub fn init() -> Result<Model, String> {
        let flags = Flags::get()?;

        let setting: Setting = if flags.dev_mode {
            Setting::init_dev()
        } else {
            Setting::init_prod()
        };

        Ok(Model {
            ip_address: flags.ip_address,
            admin_password: flags.admin_password,
            port_number: flags.port_number,
            setting,
            lobbies: Mutex::new(HashMap::new()),
            randomness_seed: Mutex::new(0),
        })
    }

    // pub fn get_lobbies(&self) -> Result<> {
    //     let lobbies_guard = self.lobbies.lock().map_err(|err| err.to_string())?;
    //     let mut lobbies = *lobbies_guard;
    //     lobbies
    // }

    pub fn new_lobby(&mut self, lobby: Lobby) -> Result<Id, String> {
        let lobbies_guard = self.lobbies.lock().map_err(|err| err.to_string())?;
        let mut lobbies = *lobbies_guard;
        let rng = self.get_rng()?;

        let lobby_id = Id::new(rng);
        lobbies.insert(lobby_id, lobby);

        Ok(lobby_id.clone())
    }
    fn rand_seed(&self) -> Result<usize, String> {
        self.randomness_seed
            .lock()
            .map(|seed_guard| {
                let seed = *seed_guard;
                seed
            })
            .map_err(|err| err.to_string())
    }

    fn get_rng(&mut self) -> Result<StdRng, String> {
        self.rand_seed().map(|seed_num| {
            let seed: &[usize] = &[seed_num];

            let mut rng: StdRng = SeedableRng::from_seed(seed);

            let new_randomness_seed: usize = rng.gen();

            self.set_seed(new_randomness_seed);

            rng
        })
    }
}
