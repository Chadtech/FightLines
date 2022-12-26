use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::create::{Request, Response};
use shared::lobby::Lobby;
use shared::player::Player;
use shared::team_color::TeamColor;

pub async fn handle(body: String, data: web::Data<Model>) -> HttpResponse {
    let bytes = match hex::decode(body) {
        Ok(bytes) => bytes,
        Err(error) => {
            return HttpResponse::BadRequest().body(error.to_string());
        }
    };

    let req: Request = match Request::from_bytes(bytes) {
        Ok(req) => req,
        Err(error) => {
            return HttpResponse::BadRequest().body(error.to_string());
        }
    };

    let host = Player::new(req.host_name.clone(), TeamColor::Red);
    let new_lobby = Lobby::new(req.host_id(), host);

    let mut lobbies = data.lobbies.lock().unwrap();

    let lobby_id = lobbies.new_lobby(new_lobby.clone());

    match Response::new(lobby_id, new_lobby).to_bytes() {
        Ok(response_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(response_bytes),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
