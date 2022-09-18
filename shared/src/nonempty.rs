use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Nonempty<T> {
    pub first: T,
    pub rest: Vec<T>,
}
