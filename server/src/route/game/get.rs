use actix_web::{web, HttpResponse};

use crate::model::Model;
use shared::api::game::get::Response;
use shared::game::GameId;

pub async fn handle(data: web::Data<Model>, url_id: web::Path<(String,)>) -> HttpResponse {
    match GameId::from_string(url_id.into_inner().0) {
        Some(game_id) => {
            let games = data.games.lock().unwrap();

            match games.get_game(game_id.clone()) {
                None => HttpResponse::NotFound().body("game does not exist"),
                Some(game) => match Response::init(game_id, game.clone()).to_bytes() {
                    Ok(response_bytes) => HttpResponse::Ok()
                        .header("Content-Type", "application/octet-stream")
                        .body(response_bytes),
                    Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
                },
            }
        }
        None => HttpResponse::BadRequest().body("Invalid game id"),
    }
}
