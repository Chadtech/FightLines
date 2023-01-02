use crate::model::Model;
use actix_web::{web, HttpResponse};
use shared::api::game::submit_turn::{Request, Response};
use shared::game::{Game, GameId};
use shared::id::Id;
use shared::rng::RandSeed;

pub async fn handle(
    body: String,
    data: web::Data<Model>,
    params: web::Path<(String, String)>,
) -> HttpResponse {
    let (game_id_param, player_id_param) = params.into_inner();

    let game_id: GameId = match GameId::from_string(game_id_param) {
        Some(game_id) => game_id,
        None => {
            return HttpResponse::BadRequest().body("Invalid game id");
        }
    };

    let player_id: Id = match Id::from_string(player_id_param) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().body("Invalid player id");
        }
    };

    let req: Request = match hex::decode(body) {
        Ok(bytes) => match Request::from_bytes(bytes) {
            Ok(req) => req,
            Err(error) => {
                return HttpResponse::BadRequest().body(error.to_string());
            }
        },
        Err(error) => {
            return HttpResponse::BadRequest().body(error.to_string());
        }
    };

    let mut games = data.games.lock().unwrap();

    let (game, rand_seed): (&mut Game, RandSeed) = match games.get_mut_game_and_seed(game_id) {
        Some(game) => game,
        None => {
            return HttpResponse::NotFound().body("game does not exist");
        }
    };

    if let Err(err) = game.set_turn(player_id, req.moves) {
        return HttpResponse::BadRequest().body(err);
    };

    if let Err(err) = game.advance_turn(rand_seed) {
        return HttpResponse::BadRequest().body(err);
    };

    match Response::new(game.clone()).to_bytes() {
        Ok(res_bytes) => HttpResponse::Ok()
            .header("Content-Type", "application/octet-stream")
            .body(res_bytes),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
