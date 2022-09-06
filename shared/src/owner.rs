use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Owned<T> {
    pub owned_by: Id,
    pub value: T,
}
