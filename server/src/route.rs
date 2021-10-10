use actix_web::{web, Responder};
use shared::api::create_lobby;

use crate::lobby::Lobby;
use crate::model::Model;
use crate::player::Player;
use shared::api::create_lobby::Response;

pub async fn create_lobby(body: String, data: web::Data<Model>) -> impl Responder {
    match hex::decode(body) {
        Ok(bytes) => match create_lobby::Request::from_bytes(bytes) {
            Ok(request) => {
                let host = Player::new(request.host_id());
                let new_lobby = Lobby::init(host);

                let mut lobbies = data.lobbies.lock().unwrap();

                let lobby_id = lobbies.new_lobby(new_lobby);

                match create_lobby::Response::init(lobby_id).to_bytes() {
                    Ok(response_bytes) => hex::encode(response_bytes),
                    Err(error) => error.to_string(),
                }
            }
            Err(error) => error.to_string(),
        },
        Err(error) => error.to_string(),
    }
}
