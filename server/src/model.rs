use crate::flags::Flags;
use crate::lobbies::Lobbies;
use crate::setting::Setting;
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

        let random_seed: usize = 0;

        let lobbies_seed: usize = random_seed + 1;

        Ok(Model {
            ip_address: flags.ip_address,
            admin_password: flags.admin_password,
            port_number: flags.port_number,
            setting,
            lobbies: Mutex::new(Lobbies::init(lobbies_seed)),
        })
    }
}
