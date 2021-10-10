use rand::distributions::{Distribution, Standard};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

pub struct RngSeed(pub [u8; N]);
pub struct RandGen(RngSeed);

////////////////////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////////////////////

const N: usize = 32;

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl RandGen {
    pub fn from_usize(n: usize) -> RandGen {
        let n0 = n as u8;
        let n1 = (n / 2) as u8;
        let n2 = (n / 4) as u8;
        let n3 = (n / 16) as u8;

        RandGen(RngSeed([
            n0, n1, n2, n3, n0, n1, n2, n3, n0, n1, n2, n3, n0, n1, n2, n3, n0, n1, n2, n3, n0, n1,
            n2, n3, n0, n1, n2, n3, n0, n1, n2, n3,
        ]))
    }

    pub fn gen<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        let RandGen(RngSeed(seed)) = self;

        let mut rng = Pcg64::from_seed(*seed);

        // let next_seed: [u8; N] = rng.gen();

        // self = &mut RandGen(RngSeed(next_seed));

        let val: T = rng.gen();
        val
    }
}

impl Default for RngSeed {
    fn default() -> RngSeed {
        RngSeed([0; N])
    }
}

impl SeedableRng for RandGen {
    type Seed = RngSeed;

    fn from_seed(seed: RngSeed) -> RandGen {
        RandGen(seed)
    }
}
impl AsMut<[u8]> for RngSeed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// impl RandGen {
//     pub fn from_seed(seed: usize) -> RandGen {
//         let seed_slice: &[usize] = &[seed];
//
//         let mut std_rng: StdRng = SeedableRng::from_seed(seed_slice);
//
//         RandGen(std_rng)
//     }
//
//     pub fn gen<T: Rand>(&mut self) -> T
//     where
//         Self: Sized,
//     {
//         let RandGen(std_rng) = self;
//
//         let val: T = std_rng.gen();
//
//         val
//     }
// }

// use rand_core::SeedableRng;
//
// const N: usize = 64;
// pub struct MyRngSeed(pub [u8; N]);
// pub struct MyRng(MyRngSeed);
//
// impl Default for MyRngSeed {
//     fn default() -> MyRngSeed {
//         MyRngSeed([0; N])
//     }
// }
//
// impl AsMut<[u8]> for MyRngSeed {
//     fn as_mut(&mut self) -> &mut [u8] {
//         &mut self.0
//     }
// }
//
// impl SeedableRng for MyRng {
//     type Seed = MyRngSeed;
//
//     fn from_seed(seed: MyRngSeed) -> MyRng {
//         MyRng(seed)
//     }
// }
