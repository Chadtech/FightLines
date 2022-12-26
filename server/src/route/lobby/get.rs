use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::get::Response;
use shared::lobby::LobbyId;

pub async fn handle(data: web::Data<Model>, url_id: web::Path<(String,)>) -> HttpResponse {
    let lobby_id: LobbyId = match LobbyId::from_string(url_id.into_inner().0) {
        Some(lobby_id) => lobby_id,
        None => {
            return HttpResponse::BadRequest().body("Invalid lobby id");
        }
    };

    let lobbies = data.lobbies.lock().unwrap();

    let lobby = match lobbies.get_lobby(lobby_id.clone()) {
        None => {
            return HttpResponse::NotFound().body("lobby does not exist");
        }
        Some(lobby) => lobby,
    };

    match Response::new(lobby_id, lobby.clone()).to_bytes() {
        Ok(res_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(res_bytes),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
