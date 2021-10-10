use rand::distributions::{Distribution, Standard};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RandSeed {
    pub bytes: [u8; N],
}

#[derive(Clone)]
pub struct RandGen {
    seed: RandSeed,
}

////////////////////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////////////////////

const N: usize = 32;

impl Distribution<RandSeed> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RandSeed {
        let rand_bytes: [u8; N] = rng.gen();
        RandSeed { bytes: rand_bytes }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////
impl RandSeed {
    pub fn from_bytes(bytes: [u8; N]) -> RandSeed {
        RandSeed { bytes }
    }
}
impl RandGen {
    pub fn from_seed(seed: RandSeed) -> RandGen {
        RandGen { seed }
    }

    fn next(&mut self) -> ([u8; N], RandSeed) {
        let RandGen {
            seed: RandSeed { bytes },
        } = self;
        let mut rng = Pcg64::from_seed(*bytes);

        let ret: ([u8; N], RandSeed) = rng.gen();

        ret
    }
    pub fn gen<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        let (bytes, next_seed) = self.next();

        let mut rng = Pcg64::from_seed(bytes.clone());

        let val: T = rng.gen();

        self.seed = next_seed;

        val
    }
}

impl Default for RandSeed {
    fn default() -> RandSeed {
        RandSeed { bytes: [0; N] }
    }
}

impl SeedableRng for RandGen {
    type Seed = RandSeed;

    fn from_seed(seed: RandSeed) -> RandGen {
        RandGen { seed }
    }
}
impl AsMut<[u8]> for RandSeed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}
