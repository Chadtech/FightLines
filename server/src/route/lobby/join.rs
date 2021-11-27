use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::join::{Request, Response};
use shared::lobby::AddError;
use shared::player::Player;

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
    let guest = Player::new(req.guest_name);

    let mut lobbies = data.lobbies.lock().unwrap();

    match &mut lobbies.get_lobby(req.lobby_id.clone()) {
        None => HttpResponse::NotFound().body("Lobby not found"),
        Some(lobby) => {
            let lobby_result = lobby.add_guest(req.guest_id, guest);

            match lobby_result.map(|l| l.clone()) {
                Ok(lobby) => {
                    lobbies.upsert(req.lobby_id.clone(), lobby.clone());

                    match Response::init(req.lobby_id, lobby).to_bytes() {
                        Ok(response_bytes) => HttpResponse::Ok()
                            .header("Content-Type", "application/octet-stream")
                            .body(response_bytes),
                        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
                    }
                }
                Err(err) => match err {
                    AddError::LobbyIsFull => HttpResponse::Conflict().body("Lobby is full"),
                },
            }
        }
    }
}
