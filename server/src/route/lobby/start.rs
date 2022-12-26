use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::start::{Request, Response};
use shared::game::{FromLobbyError, Game, GameId};
use shared::lobby::Lobby;

pub async fn handle(body: String, data: web::Data<Model>) -> HttpResponse {
    let bytes = match hex::decode(body) {
        Ok(bytes) => bytes,
        Err(error) => {
            return HttpResponse::BadRequest().body(error.to_string());
        }
    };

    let req: Request = match Request::from_bytes(bytes) {
        Ok(request) => request,
        Err(error) => {
            return HttpResponse::BadRequest().body(error.to_string());
        }
    };

    let mut lobbies = data.lobbies.lock().unwrap();
    let mut games = data.games.lock().unwrap();

    let lobby: &mut Lobby = match lobbies.get_mut_lobby(req.lobby_id.clone()) {
        Some(lobby) => lobby,
        None => {
            return HttpResponse::NotFound().body("Lobby not found");
        }
    };

    let game: Game = match games.new_game_from_lobby(lobby.clone()) {
        Ok(game) => game,
        Err(error) => {
            let res = match error {
                FromLobbyError::NotEnoughPlayers => {
                    HttpResponse::Conflict().body("Lobby does not have enough players")
                }
                FromLobbyError::CouldNotFindInitialMapMilitary { .. } => HttpResponse::Conflict().body(
                    "Map selected requires a different number of players than the number of players in the lobby"
                ),
            };

            return res;
        }
    };

    games.upsert(GameId::from_lobby_id(req.lobby_id), game.clone());

    lobby.started();

    let res: Response = Response::new(game);

    match res.to_bytes() {
        Ok(res_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(res_bytes),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
