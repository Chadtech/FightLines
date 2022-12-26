use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::lobby::update::{Request, Response};
use shared::lobby::{Lobby, UpdateError};

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

    let mut lobbies = data.lobbies.lock().unwrap();

    let lobby: &mut Lobby = match lobbies.get_mut_lobby(req.lobby_id.clone()) {
        None => {
            return HttpResponse::NotFound().body("Lobby not found");
        }
        Some(lobby) => lobby,
    };

    if let Err(error) = lobby.many_updates(req.updates) {
        let res = match error {
            UpdateError::AtMaximumSlots => {
                HttpResponse::Conflict().body("Cannot add slot. Lobby is at its maximum size")
            }
            UpdateError::NoOpenSlotToClose => HttpResponse::Conflict()
                .body("Cannot close slot. Lobby must have at least two players"),
            UpdateError::CannotFindPlayer => {
                HttpResponse::InternalServerError().body("Cannot find player")
            }
        };

        return res;
    }

    match Response::new(req.lobby_id, lobby.clone()).to_bytes() {
        Ok(response_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(response_bytes),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
