use crate::core_ext::route::go_to_route;
use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::view::error_card::ErrorCard;
use crate::view::loading_spinner::LoadingSpinner;
use crate::{api, core_ext, global};
use seed::prelude::Orders;
use shared::api::endpoint::Endpoint;
use shared::api::lobby::create;
use shared::id::Id;
use shared::lobby::Lobby;

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
    NewGameLobbyCreated { lobby_id: Id, lobby: Lobby },
}

enum NewGameError {
    FailedToSend(String),
    RemoteError(String),
}

#[derive(Clone, Debug)]
pub enum Msg {
    ClickedStartGame,
    LoadedLobby(Result<create::Response, String>),
    ClickedGoBackToTitle,
    ClickedGoToNewGame,
}

///////////////////////////////////////////////////////////////
// Init
///////////////////////////////////////////////////////////////

impl Model {
    pub fn init() -> Model {
        Model {
            status: Status::Ready,
        }
    }

    pub fn just_created_lobby(&self) -> Option<(&Id, &Lobby)> {
        match &self.status {
            Status::NewGameLobbyCreated { lobby_id, lobby } => Some((lobby_id, lobby)),
            _ => None,
        }
    }
}

///////////////////////////////////////////////////////////////
// Update
///////////////////////////////////////////////////////////////

pub fn update(global: &global::Model, msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClickedStartGame => {
            model.status = Status::WaitingForNewGame;

            match create::Request::init(global.viewer_id(), global.viewer_name.clone()).to_bytes() {
                Ok(request_bytes) => {
                    orders.skip().perform_cmd({
                        async {
                            let result = match api::post(Endpoint::CreateLobby, request_bytes).await
                            {
                                Ok(response_bytes) => create::Response::from_bytes(response_bytes)
                                    .map_err(|err| err.to_string()),
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
            Ok(res) => {
                let lobby_id = res.lobby_id;

                model.status = Status::NewGameLobbyCreated {
                    lobby_id: lobby_id.clone(),
                    lobby: res.lobby,
                };

                go_to_route(orders, Route::Lobby(lobby_id));
            }
            Err(err) => {
                model.status = Status::CouldNotMakeNewGame(NewGameError::RemoteError(err));
            }
        },
        Msg::ClickedGoBackToTitle => {
            go_to_route(orders, Route::Title);
        }
        Msg::ClickedGoToNewGame => {
            if let Status::NewGameLobbyCreated { lobby_id, lobby: _ } = &mut model.status {
                go_to_route(orders, Route::Lobby(lobby_id.clone()));
            }
        }
    }
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(model: &Model) -> Vec<Row<Msg>> {
    match &model.status {
        Status::Ready => ready_view(),
        Status::WaitingForNewGame => waiting_for_new_game_view(),
        Status::CouldNotMakeNewGame(error) => new_game_error_view(error),
        Status::NewGameLobbyCreated {
            lobby_id: _,
            lobby: _,
        } => new_game_view(),
    }
}

fn new_game_view() -> Vec<Row<Msg>> {
    let title = "new game is ready";

    let msg = "if this page does not redirect to the new games lobby, click the button below.";

    let card = Card::cell_from_rows(
        vec![Style::G4],
        vec![
            Row::from_str(title),
            Row::from_str(msg),
            Row::from_cells(
                vec![Style::JustifyEnd],
                vec![Button::primary("go back to title page")
                    .on_click(|_| Msg::ClickedGoToNewGame)
                    .cell()],
            ),
        ],
    );

    vec![center(vec![card])]
}

fn new_game_error_view(error: &NewGameError) -> Vec<Row<Msg>> {
    let title = match error {
        NewGameError::FailedToSend(_) => "failed to send request",
        NewGameError::RemoteError(_) => "failed to start new game",
    };

    let msg = match error {
        NewGameError::FailedToSend(s) => s.as_str(),
        NewGameError::RemoteError(s) => s.as_str(),
    };

    let card = ErrorCard::from_title(title)
        .with_msg(msg)
        .with_buttons(vec![
            Button::primary("go back to title page").on_click(|_| Msg::ClickedGoBackToTitle)
        ])
        .cell();

    vec![center(vec![card])]
}

fn waiting_for_new_game_view() -> Vec<Row<Msg>> {
    let msg = "starting new game..";

    let card = Card::cell_from_rows(
        vec![Style::G4],
        vec![Row::from_str(msg), LoadingSpinner::row()],
    );

    vec![center(vec![card])]
}

fn ready_view() -> Vec<Row<Msg>> {
    vec![
        center(vec![Cell::from_str(
            vec![Style::JustifyCenter],
            "Fightlines",
        )]),
        center_button(Button::primary("start game").on_click(|_| Msg::ClickedStartGame)),
        center_button(Button::simple("join game")),
        center_button(Button::simple("custom game")),
    ]
}

fn center_button(button: Button<Msg>) -> Row<Msg> {
    center(vec![button
        .full_width()
        .cell()
        .with_styles(vec![Style::W8])])
}

fn center(cells: Vec<Cell<Msg>>) -> Row<Msg> {
    Row::from_cells(vec![Style::JustifyCenter], cells)
}

pub const PARENT_STYLES: [Style; 2] = [Style::JustifyCenter, GAP_SIZE];

const GAP_SIZE: Style = Style::G3;
