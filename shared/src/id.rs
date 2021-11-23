use crate::rng::RandGen;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Id {
    bytes: [u8; N],
}

pub struct UniformId(UniformInt<u8>);

////////////////////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////////////////////

impl Id {
    pub fn new(rng: &mut RandGen) -> Id {
        let id_bytes: [u8; 16] = [
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
            rng.gen::<u8>(0, 255),
        ];

        let new_id = Id { bytes: id_bytes };

        new_id
    }

    pub fn to_display_string(&self) -> String {
        let mut buf = String::new();

        for byte in self.bytes {
            let hex: String = format!("{:02X}", byte);

            buf.push_str(hex.as_str());
        }

        buf
    }

    pub fn from_int_test_only(n: u8) -> Id {
        Id {
            bytes: [n, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

impl Distribution<Id> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Id {
        let rand_bytes: [u8; N] = rng.gen();
        Id { bytes: rand_bytes }
    }
}

impl UniformSampler for UniformId {
    type X = Id;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformId(UniformInt::<u8>::new(
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
        Id {
            bytes: [
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
            ],
        }
    }
}

impl SampleUniform for Id {
    type Sampler = UniformId;
}

const N: usize = 16;
