use crate::rng::RandGen;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Id {
    bytes: [u8; N],
}

const N: usize = 16;

impl Id {
    pub fn new(mut rng: RandGen) -> Id {
        let bytes: [u8; 16] = rng.gen();

        Id { bytes }
    }
}

impl Distribution<Id> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Id {
        let rand_bytes: [u8; N] = rng.gen();
        Id { bytes: rand_bytes }
    }
}
