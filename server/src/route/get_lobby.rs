use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::get_lobby::Response;
use shared::id::Id;

pub async fn handle(data: web::Data<Model>, url_id: web::Path<(String,)>) -> HttpResponse {
    match Id::from_string(url_id.into_inner().0) {
        Some(lobby_id) => {
            let lobbies = data.lobbies.lock().unwrap();

            let maybe_lobby = lobbies.get_lobby(lobby_id.clone());

            match maybe_lobby {
                None => HttpResponse::NotFound().body("lobby does not exist"),
                Some(lobby) => match Response::init(lobby_id, lobby).to_bytes() {
                    Ok(response_bytes) => HttpResponse::Ok()
                        .header("Content-Type", "application/octet-stream")
                        .body(response_bytes),
                    Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
                },
            }
        }
        None => HttpResponse::BadRequest().body("Invalid lobby id"),
    }
}
