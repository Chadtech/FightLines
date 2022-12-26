use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::join::{Request, Response};
use shared::lobby::{AddError, Lobby};
use shared::player::Player;
use shared::team_color::TeamColor;

pub async fn handle(body: String, data: web::Data<Model>) -> HttpResponse {
    let body_bytes = match hex::decode(body) {
        Ok(bytes) => bytes,
        Err(error) => {
            return HttpResponse::BadRequest().body(error.to_string());
        }
    };

    let req: Request = match Request::from_bytes(body_bytes) {
        Ok(req) => req,
        Err(error) => {
            return HttpResponse::BadRequest().body(error.to_string());
        }
    };

    let guest = Player::new(req.guest_name, TeamColor::Blue);

    let mut lobbies = data.lobbies.lock().unwrap();

    let lobby: &mut Lobby = match lobbies.get_mut_lobby(req.lobby_id.clone()) {
        None => {
            return HttpResponse::NotFound().body("Lobby not found");
        }
        Some(lobby) => lobby,
    };

    if let Err(err) = lobby.add_guest(req.guest_id, guest) {
        match err {
            AddError::LobbyIsFull => {
                return HttpResponse::Conflict().body("Lobby is full");
            }
        }
    };
    let res: Response = Response::new(req.lobby_id, lobby.clone());

    match res.to_bytes() {
        Ok(res_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(res_bytes),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
