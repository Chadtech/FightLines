use crate::rng::RandGen;
use serde::{Deserialize, Serialize};

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Name(String);

///////////////////////////////////////////////////////////////
// Helpers //
///////////////////////////////////////////////////////////////

impl Name {
    pub fn random(rng: &mut RandGen) -> Name {
        let i = rng.gen::<u8>(0, 63);

        let name = RANDOM_NAMES[i as usize];

        Name(name.to_string())
    }

    pub fn from_string(s: String) -> Option<Name> {
        if s.is_empty() {
            None
        } else {
            Some(Name(s))
        }
    }

    pub fn from_str(s: &str) -> Name {
        Name(s.to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
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
    "Jones",
    "Huemer",
    "Friedman",
    "Cowen",
    "Tabarrok",
];
