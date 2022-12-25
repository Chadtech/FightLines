use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::start::{Request, Response};
use shared::game::FromLobbyError;

pub async fn handle(body: String, data: web::Data<Model>) -> HttpResponse {
    match hex::decode(body) {
        Ok(bytes) => match Request::from_bytes(bytes) {
            Ok(request) => from_req(request, data).await,
            Err(error) => HttpResponse::BadRequest().body(error.to_string()),
        },
        Err(error) => HttpResponse::BadRequest().body(error.to_string()),
    }
}

async fn from_req(req: Request, data: web::Data<Model>) -> HttpResponse {
    let mut lobbies = data.lobbies.lock().unwrap();
    let mut games = data.games.lock().unwrap();

    match &mut lobbies.get_lobby(req.lobby_id.clone()) {
        None => HttpResponse::NotFound().body("Lobby not found"),

        Some(lobby) => match games.new_game_from_lobby(lobby.clone()) {
            Ok(game) => {
                games.upsert(req.lobby_id.clone(), game.clone());

                lobby.started();
                lobbies.upsert(req.lobby_id, lobby.clone());

                response_to_http(Response::init(game))
            }
            Err(error) => match error {
                FromLobbyError::NotEnoughPlayers => {
                    HttpResponse::Conflict().body("Lobby does not have enough players")
                }
                FromLobbyError::CouldNotFindInitialMapMilitary { .. } => HttpResponse::Conflict().body(
                    "Map selected requires a different number of players than the number of players in the lobby"
                ),
            },
        },
    }
}

fn response_to_http(res: Response) -> HttpResponse {
    match res.to_bytes() {
        Ok(response_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(response_bytes),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
