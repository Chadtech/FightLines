use crate::rng::RandGen;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Name(String);

impl Name {
    pub fn random(rng: &mut RandGen) -> Name {
        let i = rng.gen::<u8>(0, 63);

        let name = RANDOM_NAMES[i as usize];

        Name(name.to_string())
    }

    pub fn new(s: &str) -> Name {
        let mut buf = String::new();
        buf.push_str("new ");
        buf.push_str(s);

        Name(buf)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Name {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(Error::NameCannotBeEmpty)
        } else {
            Ok(Name(s.to_string()))
        }
    }
}

impl ToString for Name {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub enum Error {
    NameCannotBeEmpty,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::NameCannotBeEmpty => "name cannot be empty".to_string(),
        }
    }
}

///////////////////////////////////////////////////////////////
// Helpers //
///////////////////////////////////////////////////////////////

const RANDOM_NAMES: [&str; 64] = [
    "Partch",
    "Cage",
    "Lang",
    "Reich",
    "Glass",
    "Hardin",
    "Hamilton",
    "Mason",
    "Madison",
    "Franklin",
    "Ferguson",
    "Gaethje",
    "Thompson",
    "Hall",
    "Prochazka",
    "Vettori",
    "Oliveira",
    "Teixeira",
    "Oleynik",
    "Lewis",
    "Frege",
    "Russell",
    "Anscombe",
    "Moore",
    "Ramsey",
    "James",
    "Horikoshi",
    "Kripke",
    "Truman",
    "Deng",
    "Jiang",
    "Zhang",
    "Hess",
    "Menhennette",
    "Ziesel",
    "Zietner",
    "Meier",
    "Stonborough",
    "Sayn",
    "Kalmus",
    "Schemp",
    "Nischwitz",
    "Strasser",
    "Vitow",
    "Marshall",
    "Roberts",
    "Thomas",
    "Jay",
    "White",
    "Black",
    "Brandeis",
    "Kopf",
    "Kerr",
    "Volokh",
    "Barnett",
    "Greenfield",
    "Baude",
    "Caplan",
    "Hansen",
    "Huemer",
    "Friedman",
    "Cowen",
    "Tabarrok",
    "Hsu",
];
