use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler};
use rand::distributions::{Distribution, Standard, Uniform};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::fmt;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq)]
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

const LOW_SEED: RandSeed = RandSeed { bytes: [0; N] };

const HIGH_SEED: RandSeed = RandSeed { bytes: [255; N] };

const N: usize = 32;

impl fmt::Display for RandSeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

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

    pub fn to_display_string(&self) -> String {
        let mut buf = String::new();

        for byte in self.bytes {
            let hex: String = format!("{:02X}", byte);

            buf.push_str(hex.as_str());
        }

        buf
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

        let uniform = Uniform::new(low, high);
        let val: T = uniform.sample(&mut rng);

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

#[cfg(test)]
mod test_rng {
    use crate::rng::{RandGen, RandSeed, N};
    use std::ops::Index;

    #[test]
    fn more_than_255_seeds() {
        let seeds_count = 1024;
        let mut seeds: Vec<RandSeed> = Vec::with_capacity(seeds_count);
        seeds.push(RandSeed::from_bytes([0; N]));

        let mut index = 0;

        while index < (seeds_count - 1) {
            let mut rng = RandGen::from_seed(seeds[index].clone());

            let next_seed: RandSeed = RandSeed::next(&mut rng);

            seeds.push(next_seed);

            index += 1;
        }

        index = 0;

        while index < seeds_count {
            let mut j = index + 1;
            let this_seed = seeds[index].clone();

            while j < seeds_count {
                let next_seed = seeds[j].clone();

                if (this_seed == next_seed) {
                    for (i, seed) in seeds.iter().enumerate() {
                        println!("index {},  {}", i, seed);
                    }
                    panic!("Seeds are the same {} {} {}", index, j, this_seed);
                }
                j += 1;
            }

            index += 1;
        }
    }
}
