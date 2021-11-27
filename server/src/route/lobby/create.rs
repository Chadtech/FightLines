use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::create::{Request, Response};
use shared::lobby::Lobby;
use shared::name::Name;
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

async fn from_req(request: Request, data: web::Data<Model>) -> HttpResponse {
    let host = Player::new(request.host_id(), request.host_name);
    let new_lobby = Lobby::init(host);

    let mut lobbies = data.lobbies.lock().unwrap();

    let lobby_id = lobbies.new_lobby(new_lobby.clone());

    match Response::init(lobby_id, new_lobby).to_bytes() {
        Ok(response_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(response_bytes),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
