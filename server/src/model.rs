use crate::flags::Flags;
use crate::lobbies::Lobbies;
use crate::setting::Setting;
use shared::rng::{RandGen, RandSeed};
use std::sync::Mutex;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

// #[derive(Clone)]
pub struct Model {
    pub ip_address: String,
    pub admin_password: String,
    pub port_number: u64,
    pub setting: Setting,
    pub lobbies: Mutex<Lobbies>,
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

        let random_seed: RandSeed = RandSeed::from_bytes([
            125, 106, 161, 45, 228, 43, 117, 234, 218, 111, 245, 253, 177, 14, 76, 151, 40, 229,
            183, 121, 69, 23, 25, 227, 72, 149, 42, 114, 251, 186, 158, 131,
        ]);

        let mut rand_gen: RandGen = RandGen::from_seed(random_seed);

        let lobbies_seed: RandSeed = rand_gen.gen();

        Ok(Model {
            ip_address: flags.ip_address,
            admin_password: flags.admin_password,
            port_number: flags.port_number,
            setting,
            lobbies: Mutex::new(Lobbies::init(lobbies_seed)),
        })
    }
}
