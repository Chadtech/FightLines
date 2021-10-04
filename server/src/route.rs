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

                let lobbies = data.lobbies.lock();

                let lobby_id = todo!("Make lobby id");

                // TODO put lobby in lobbies

                match create_lobby::Response::init(lobby_id).to_bytes() {
                    Ok(response_bytes) => hex::encode(response_bytes),
                    Err(error) => error.to_string(),
                }
                // let lobby = data.lobbies.lock().unwrap();

                // let mut model = mutex.lock().unwrap();
                //
                // let new_game = game::init_lobby(request.game_name(), request.host_name());
                // let game_id: u64 = model.add_game(new_game);
                // match start_game::Response::init(game_id).to_bytes() {
                //     Ok(response_bytes) => hex::encode(response_bytes),
                //     Err(error) => error.to_string(),
                // }
            }
            Err(error) => error.to_string(),
        },
        Err(error) => error.to_string(),
    }
}
