use crate::rng::RandGen;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformInt, UniformSampler};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Id {
    Bytes { bytes: [u8; N] },
    Dev(String),
}

pub struct UniformId(UniformInt<u8>);

////////////////////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////////////////////

const N: usize = 16;

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

        Id::Bytes { bytes: id_bytes }
    }

    pub fn from_string(s: String) -> Option<Id> {
        match hex::decode(s) {
            Ok(bytes) => {
                if bytes.len() == N {
                    let mut new_id = [0; N];

                    for (i, byte) in bytes.iter().enumerate() {
                        new_id[i] = *byte;
                    }

                    Some(Id::Bytes { bytes: new_id })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn from_int_test_only(n: u8) -> Id {
        Id::Bytes {
            bytes: [n, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        match self {
            Id::Bytes { bytes } => {
                let mut buf = String::new();

                for byte in bytes {
                    let hex: String = format!("{:02X}", byte);

                    buf.push_str(hex.as_str());
                }

                buf
            }
            Id::Dev(dev_id) => dev_id.clone(),
        }
    }
}

impl Distribution<Id> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Id {
        let rand_bytes: [u8; N] = rng.gen();
        Id::Bytes { bytes: rand_bytes }
    }
}

impl UniformSampler for UniformId {
    type X = Id;
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let l = match low.borrow() {
            Id::Bytes { bytes } => bytes[0],
            Id::Dev(_) => 0,
        };

        let h = match high.borrow() {
            Id::Bytes { bytes } => bytes[0],
            Id::Dev(_) => 255,
        };
        UniformId(UniformInt::<u8>::new(l, h))
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Id::Bytes {
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
