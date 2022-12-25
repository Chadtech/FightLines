use crate::game::{Game, GameId};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////
// Response //
////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Response {
    game_id: GameId,
    game: Game,
}

impl Response {
    pub fn init(game_id: GameId, game: Game) -> Response {
        Response { game_id, game }
    }

    pub fn get_game_id(&self) -> GameId {
        self.game_id.clone()
    }

    pub fn get_game(&self) -> Game {
        self.game.clone()
    }

    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(byte_data: Vec<u8>) -> bincode::Result<Response> {
        bincode::deserialize(&byte_data[..])
    }
}
