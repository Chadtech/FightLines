use rand::{Rng, StdRng};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone)]
pub struct Id {
    bytes: [u8; 16],
}

impl Id {
    pub fn new(mut rng: StdRng) -> Id {
        let mut bytes: [u8; 16] = rng.gen();

        Id { bytes }
    }
}
