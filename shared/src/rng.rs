use rand::distributions::uniform::{
    SampleBorrow, SampleUniform, UniformFloat, UniformInt, UniformSampler,
};
use rand::distributions::{Distribution, Standard, Uniform};
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

pub struct UniformRandSeed(UniformInt<u8>);

////////////////////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////////////////////

const LOW_SEED: RandSeed = RandSeed {
    bytes: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
};

const HIGH_SEED: RandSeed = RandSeed {
    bytes: [
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    ],
};

const N: usize = 32;

impl Distribution<RandSeed> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RandSeed {
        let rand_bytes: [u8; N] = rng.gen();
        RandSeed { bytes: rand_bytes }
    }
}
impl UniformSampler for UniformRandSeed {
    type X = RandSeed;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformRandSeed(UniformInt::<u8>::new(
            low.borrow().bytes[0],
            high.borrow().bytes[0],
        ))
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        let bytes = [
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
            self.0.sample(rng),
        ];

        let next = RandSeed { bytes };

        dbg!(bytes);

        next
    }
}
impl SampleUniform for RandSeed {
    type Sampler = UniformRandSeed;
}

////////////////////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////////////////////

impl RandSeed {
    pub fn from_bytes(bytes: [u8; N]) -> RandSeed {
        RandSeed { bytes }
    }
    pub fn next(rand_gen: &mut RandGen) -> RandSeed {
        let new_seed: RandSeed = rand_gen.gen(LOW_SEED, HIGH_SEED);

        new_seed
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
    pub fn gen<T>(&mut self, low: T, high: T) -> T
    where
        T: SampleUniform,
        Uniform<T>: Distribution<T>,
        Standard: Distribution<T>,
    {
        let (bytes, next_seed) = self.next();

        let mut rng = Pcg64::from_seed(bytes.clone());

        // let (low, high) = (LOW_SEED, HIGH_SEED);
        let uniform = Uniform::new(low, high);
        let val: T = uniform.sample(&mut rng);

        // let val: T = rng.gen();

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
