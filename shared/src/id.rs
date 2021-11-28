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
        let id_bytes: [u8; N] = [
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

    pub fn from_string(s: String) -> Option<Id> {
        match hex::decode(s) {
            Ok(bytes) => {
                if bytes.len() == N {
                    let mut new_id = Id::empty();

                    for (i, byte) in bytes.iter().enumerate() {
                        new_id.bytes[i] = byte.clone();
                    }

                    Some(new_id)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn empty() -> Id {
        Id { bytes: [0; N] }
    }

    pub fn from_int_test_only(n: u8) -> Id {
        Id {
            bytes: [n, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = String::new();

        for byte in self.bytes {
            let hex: String = format!("{:02X}", byte);

            buf.push_str(hex.as_str());
        }

        write!(f, "{}", buf)
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

#[cfg(test)]
mod test_id {
    use crate::id::Id;

    #[test]
    fn can_make_id_from_string() {
        let id_string = "6D5B5DBFF37475EFE4C09C075A968A54".to_string();
        let maybe_id = Id::from_string(id_string.clone());

        if maybe_id.is_none() {}

        match maybe_id {
            None => {
                panic!("Could not make id from string");
            }
            Some(id) => {
                assert_eq!(id.to_string(), id_string);
            }
        }
    }

    #[test]
    fn cannot_make_id_from_short_string() {
        if Id::from_string("6D5B5DBFF37475EFE4C09C075A968A5".to_string()).is_some() {
            panic!("Could make id from string");
        }

        if Id::from_string("6D5B5DBFF37475EFE4C09C075A968A".to_string()).is_some() {
            panic!("Could make id from string");
        }
    }

    #[test]
    fn cannot_make_id_from_long_string() {
        if Id::from_string("6D5B5DBFF37475EFE4C09C075A968A5FF".to_string()).is_some() {
            panic!("Could make id from string");
        }

        if Id::from_string("6D5B5DBFF37475EFE4C09C075A968AF".to_string()).is_some() {
            panic!("Could make id from string");
        }
    }
}
