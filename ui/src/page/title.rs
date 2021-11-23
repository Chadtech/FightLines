use crate::style::Style;
use crate::view::button::Button;
use crate::view::card;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::{core_ext, global};
use seed::log;
use seed::prelude::{fetch, Method, Orders, Request};
use shared::api::create_lobby;
use shared::api::create_lobby::Response;
use shared::api::endpoint::Endpoint;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub struct Model {
    status: Status,
}

enum Status {
    Ready,
    WaitingForNewGame,
    CouldNotMakeNewGame(NewGameError),
}

enum NewGameError {
    FailedToSend(String),
}

#[derive(Clone, Debug)]
pub enum Msg {
    ClickedStartGame,
    LoadedLobby(Result<create_lobby::Response, String>),
}

///////////////////////////////////////////////////////////////
// Init
///////////////////////////////////////////////////////////////

pub fn init<'a>() -> Model {
    Model {
        status: Status::Ready,
    }
}

///////////////////////////////////////////////////////////////
// Update
///////////////////////////////////////////////////////////////

pub fn update(global: &global::Model, msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClickedStartGame => {
            model.status = Status::WaitingForNewGame;

            let url = Endpoint::CreateLobby.to_url();

            let lobby_request = create_lobby::Request::init(global.viewer_id());
            match lobby_request.to_bytes() {
                Ok(request_bytes) => {
                    orders.skip().perform_cmd({
                        async {
                            let result = match send_request(url, request_bytes).await {
                                Ok(response_bytes) => {
                                    match create_lobby::Response::from_bytes(response_bytes) {
                                        Ok(response) => Ok(response),
                                        Err(error) => Err(error.to_string()),
                                    }
                                }
                                Err(error) => {
                                    let fetch_error = core_ext::http::fetch_error_to_string(error);
                                    Err(fetch_error)
                                }
                            };

                            Msg::LoadedLobby(result)
                        }
                    });
                }
                Err(err) => {
                    model.status =
                        Status::CouldNotMakeNewGame(NewGameError::FailedToSend(err.to_string()))
                }
            };
        }
        Msg::LoadedLobby(result) => match result {
            Ok(response) => {
                log!(response.get_lobby_id().to_display_string());
            }
            Err(_) => {}
        },
    }
}

async fn send_request(url: String, bytes: Vec<u8>) -> fetch::Result<Vec<u8>> {
    Request::new(url.as_str())
        .method(Method::Post)
        .text(hex::encode(bytes))
        .fetch()
        .await?
        .check_status()?
        .bytes()
        .await
}
///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(model: &Model) -> Vec<Row<Msg>> {
    match &model.status {
        Status::Ready => ready_view(),
        Status::WaitingForNewGame => waiting_for_new_game_view(),
        Status::CouldNotMakeNewGame(error) => new_game_error_view(error),
    }
}

fn new_game_error_view(error: &NewGameError) -> Vec<Row<Msg>> {
    vec![]
}

fn waiting_for_new_game_view() -> Vec<Row<Msg>> {
    vec![Row::from_cells(
        vec![Style::JustifyCenter],
        vec![Card::cell(
            vec![Style::W8],
            vec![Cell::from_str(vec![], "Waiting..")],
        )],
    )]
}

fn ready_view() -> Vec<Row<Msg>> {
    vec![
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Cell::from_str(vec![Style::JustifyCenter], "Fightlines")],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::primary("start game")
                .on_click(|_| Msg::ClickedStartGame)
                .full_width()
                .cell()
                .with_styles(vec![Style::W8])],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::simple("join game")
                .full_width()
                .cell()
                .with_styles(vec![Style::W8])],
        ),
        Row::from_cells(
            vec![Style::JustifyCenter],
            vec![Button::simple("custom game")
                .full_width()
                .cell()
                .with_styles(vec![Style::W8])],
        ),
    ]
}

pub fn parent_styles() -> Vec<Style> {
    vec![Style::JustifyCenter, GAP_SIZE]
}

const GAP_SIZE: Style = Style::G3;
