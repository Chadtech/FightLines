use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Arrow {
    EndLeft,
    EndDown,
    EndRight,
    EndUp,
    X,
    Y,
    RightUp,
    RightDown,
    LeftUp,
    LeftDown,
}
