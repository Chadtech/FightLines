use crate::core_ext::route::go_to_route;
use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::view::text_field::TextField;
use crate::view::toast::Toast;
use crate::{api, core_ext, global};
use seed::prelude::{cmds, CmdHandle, Orders};
use shared::api::endpoint::Endpoint;
use shared::api::lobby::get as lobby_get;
use shared::api::lobby::start as lobby_start;
use shared::api::lobby::update as lobby_update;
use shared::game::{FromLobbyError, Game};
use shared::id::Id;
use shared::lobby;
use shared::lobby::{Lobby, MAX_GUESTS};
use shared::name::{Error, Name};
use shared::player::Player;
use std::str::FromStr;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    lobby_id: Id,
    lobby: Lobby,
    name_field: String,
    initial_name_field: String,
    host_model: Option<HostModel>,
    created_game: Option<Game>,
    handle_timeout: CmdHandle,
    load_failure_retries: u32,
}

struct HostModel {
    game_name_field: String,
    initial_name_field: String,
}

#[derive(Clone, Debug)]
pub enum Msg {
    ClickedAddSlot,
    ClickedCloseSlot,
    UpdatedLobby(Result<lobby_update::Response, String>),
    UpdatedNameField(String),
    UpdatedGameNameField(String),
    ClickedSaveGameName,
    ClickedSavePlayerName,
    ClickedKickGuest(Id),
    ClickedStart,

    //
    PollTimeoutExpired,
    GotLobbyResponse(Result<lobby_get::Response, String>),
    StartedGame(Result<lobby_start::Response, String>),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub lobby_id: Id,
    pub lobby: Lobby,
}

///////////////////////////////////////////////////////////////
// Helpers //
///////////////////////////////////////////////////////////////

impl HostModel {
    pub fn init(global: &global::Model, lobby: Lobby) -> Option<HostModel> {
        if global.viewer_id() == lobby.host_id {
            let name_str = lobby.name.to_string();

            let host_model = HostModel {
                game_name_field: name_str.clone(),
                initial_name_field: name_str,
            };

            Some(host_model)
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

pub enum InitError {
    PlayerIsKicked,
}

impl Model {
    pub fn init(
        global: &global::Model,
        flags: Flags,
        orders: &mut impl Orders<Msg>,
    ) -> Result<Model, InitError> {
        let lobby = flags.lobby.clone();

        if lobby.kicked_guests.contains(&global.viewer_id()) {
            go_to_route(orders, Route::Kicked);
            Err(InitError::PlayerIsKicked)
        } else {
            let name_field = global.viewer_name.to_string();

            let load_failure_retries = 0;

            let model = Model {
                lobby_id: flags.lobby_id,
                lobby,
                name_field: name_field.clone(),
                initial_name_field: name_field,
                host_model: HostModel::init(global, flags.lobby),
                handle_timeout: wait_to_poll_lobby(load_failure_retries, orders),
                created_game: None,
                load_failure_retries,
            };

            Ok(model)
        }
    }
    pub fn viewer_is_host(&self) -> bool {
        self.host_model.is_some()
    }

    pub fn started_game(&self) -> Option<(Id, &Game)> {
        self.created_game
            .as_ref()
            .map(|game| (self.lobby_id.clone(), game))
    }
}

fn wait_to_poll_lobby(retries: u32, orders: &mut impl Orders<Msg>) -> CmdHandle {
    let wait_time = 2048 * (2 ^ retries);

    orders.perform_cmd_with_handle(cmds::timeout(wait_time, || Msg::PollTimeoutExpired))
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

pub fn update(
    global: &mut global::Model,
    msg: Msg,
    model: &mut Model,
    orders: &mut impl Orders<Msg>,
) {
    match msg {
        Msg::ClickedAddSlot => {
            send_updates(
                global,
                model.lobby_id.clone(),
                vec![lobby::Update::AddSlot],
                orders,
            );
        }
        Msg::ClickedCloseSlot => {
            if !model.lobby.at_player_count_minimum() {
                send_updates(
                    global,
                    model.lobby_id.clone(),
                    vec![lobby::Update::CloseSlot],
                    orders,
                );
            }
        }
        Msg::UpdatedLobby(result) => match result {
            Ok(res) => {
                let lobby = res.get_lobby();

                handle_updated_lobby(&global, model, &lobby, orders);
            }
            Err(error) => global.toast(
                Toast::init("error", "could not load lobby")
                    .error()
                    .with_more_info(error.as_str()),
            ),
        },
        Msg::UpdatedNameField(field) => {
            model.name_field = field;
        }
        Msg::UpdatedGameNameField(field) => {
            if let Some(host_model) = &mut model.host_model {
                host_model.game_name_field = field;
            }
        }
        Msg::ClickedSaveGameName => {
            if let Some(host_model) = &mut model.host_model {
                match validate_name_field(
                    &host_model.game_name_field,
                    &host_model.initial_name_field,
                ) {
                    Ok(name) => {
                        host_model.initial_name_field = host_model.game_name_field.clone();

                        send_updates(
                            global,
                            model.lobby_id.clone(),
                            vec![lobby::Update::ChangeName(name)],
                            orders,
                        )
                    }
                    Err(err) => global.toast(Toast::validation_error(err.as_str())),
                }
            }
        }
        Msg::ClickedSavePlayerName => match Name::from_str(model.name_field.as_str()) {
            Err(error) => {
                let error_msg = match error {
                    Error::NameCannotBeEmpty => "player name cannot be blank",
                };
                global.toast(Toast::validation_error(error_msg))
            }
            Ok(name) => {
                global.set_viewer_name(name.clone());

                model.initial_name_field = model.name_field.clone();

                send_updates(
                    global,
                    model.lobby_id.clone(),
                    vec![lobby::Update::ChangePlayerName {
                        player_id: global.viewer_id(),
                        new_name: name,
                    }],
                    orders,
                );
            }
        },
        Msg::ClickedKickGuest(guest_id) => {
            if model.host_model.is_some() {
                send_updates(
                    global,
                    model.lobby_id.clone(),
                    vec![lobby::Update::KickGuest { guest_id }],
                    orders,
                )
            }
        }
        Msg::ClickedStart => {
            if model.host_model.is_some() {
                attempt_start_game(global, model, orders)
            }
        }
        Msg::PollTimeoutExpired => {
            model.handle_timeout = wait_to_poll_lobby(model.load_failure_retries, orders);

            let lobby_id = model.lobby_id.clone();

            orders.skip().perform_cmd({
                async {
                    let result = match api::get(Endpoint::make_get_lobby(lobby_id)).await {
                        Ok(response_bytes) => lobby_get::Response::from_bytes(response_bytes)
                            .map_err(|err| err.to_string()),
                        Err(error) => Err(core_ext::http::fetch_error_to_string(error)),
                    };

                    Msg::GotLobbyResponse(result)
                }
            });
        }
        Msg::GotLobbyResponse(result) => match result {
            Ok(res) => {
                model.load_failure_retries = 0;

                let lobby = res.get_lobby();

                handle_updated_lobby(&global, model, &lobby, orders);
            }
            Err(error) => {
                model.load_failure_retries += 1;
                model.handle_timeout = wait_to_poll_lobby(model.load_failure_retries, orders);

                global.toast(
                    Toast::init("error", "could not load lobby")
                        .error()
                        .with_more_info(error.as_str()),
                )
            }
        },
        Msg::StartedGame(result) => match result {
            Ok(res) => {
                model.created_game = Some(res.game);

                // Games use the same id as the lobby they were
                // created from. Should that ever change, this code
                // should change too.
                go_to_route(orders, Route::Game(model.lobby_id.clone()));
            }
            Err(error) => global.toast(
                Toast::init("error", "could not load new game")
                    .error()
                    .with_more_info(error.as_str()),
            ),
        },
    }
}

fn attempt_start_game(
    global: &mut global::Model,
    model: &mut Model,
    orders: &mut impl Orders<Msg>,
) {
    match Game::from_lobby(model.lobby.clone()) {
        Ok(_) => {
            let req = lobby_start::Request {
                player_id: global.viewer_id(),
                lobby_id: model.lobby_id.clone(),
            };

            match req.to_bytes() {
                Ok(bytes) => {
                    orders.skip().perform_cmd({
                        async {
                            let result = match api::post(Endpoint::StartGame, bytes).await {
                                Ok(response_bytes) => {
                                    lobby_start::Response::from_bytes(response_bytes)
                                        .map_err(|err| err.to_string())
                                }
                                Err(error) => Err(core_ext::http::fetch_error_to_string(error)),
                            };

                            Msg::StartedGame(result)
                        }
                    });
                }
                Err(error) => {
                    global.toast(
                        Toast::init(
                            "error",
                            "could not start game, could not encode start request",
                        )
                        .error()
                        .with_more_info(error.to_string().as_str()),
                    );
                }
            }
        }
        Err(error) => {
            let text = match error {
                FromLobbyError::NotEnoughPlayers => "could not start game, not enough players",
            };

            global.toast(Toast::init("error", text).error());
        }
    }
}

fn handle_updated_lobby(
    global: &global::Model,
    model: &mut Model,
    lobby: &Lobby,
    orders: &mut impl Orders<Msg>,
) {
    // Redirect out of here, if the viewer is kicked
    if lobby.kicked_guests.contains(&global.viewer_id()) {
        go_to_route(orders, Route::Kicked);
    } else {
        model.lobby = lobby.clone();

        if lobby.game_started {
            go_to_route(orders, Route::Game(model.lobby_id.clone()))
        }
    }
}

fn send_updates(
    global: &mut global::Model,
    lobby_id: Id,
    upts: Vec<lobby::Update>,
    orders: &mut impl Orders<Msg>,
) {
    let req = lobby_update::Request {
        lobby_id: lobby_id.clone(),
        updates: upts,
    };

    match req.to_bytes() {
        Ok(request_bytes) => {
            orders.skip().perform_cmd({
                async {
                    let result = match api::post(Endpoint::update_lobby(), request_bytes).await {
                        Ok(response_bytes) => lobby_update::Response::from_bytes(response_bytes)
                            .map_err(|err| err.to_string()),
                        Err(error) => {
                            let fetch_error = core_ext::http::fetch_error_to_string(error);

                            Err(fetch_error)
                        }
                    };

                    Msg::UpdatedLobby(result)
                }
            });
        }
        Err(error) => {
            global.toast(
                Toast::init("error", "could not send update, could not encode update")
                    .error()
                    .with_more_info(error.to_string().as_str()),
            );
        }
    };
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn view(global: &global::Model, model: &Model) -> Vec<Row<Msg>> {
    let mut rows = Vec::new();

    let viewer_is_host = model.viewer_is_host();

    let lobby = &model.lobby;

    let guests = &lobby.guests;

    rows.push(center(header(model.host_model.as_ref(), lobby.clone())));

    rows.push(center(host_card(viewer_is_host, model)));

    for (guest_id, guest) in guests.into_iter() {
        let guest_row = center(guest_card(
            global.viewer_id(),
            guest_id.clone(),
            guest.clone(),
            &model.name_field,
            &model.initial_name_field,
            viewer_is_host,
        ));

        rows.push(guest_row)
    }

    let num_guests_limit = lobby.num_guests_limit();

    for _ in 0..(num_guests_limit - lobby.num_guests()) {
        rows.push(center(open_slot(
            viewer_is_host,
            lobby.at_player_count_minimum(),
        )))
    }

    if num_guests_limit < MAX_GUESTS && viewer_is_host {
        rows.push(center(add_slot_row()))
    }

    rows
}

fn header(maybe_host_model: Option<&HostModel>, lobby: Lobby) -> Cell<Msg> {
    match maybe_host_model {
        Some(host_model) => Cell::group(
            vec![CARD_WIDTH, Style::G4, Style::FlexRow],
            vec![
                Cell::from_str(vec![Style::FlexCol, Style::JustifyCenter], "game name"),
                TextField::simple(host_model.game_name_field.as_str(), |field| {
                    Msg::UpdatedGameNameField(field)
                })
                .cell(),
                Button::simple("save")
                    .disable(!can_save_name_field(
                        &host_model.game_name_field,
                        &host_model.initial_name_field,
                    ))
                    .on_click(|_| Msg::ClickedSaveGameName)
                    .cell(),
                Cell::group(
                    vec![Style::Grow, Style::FlexRow, Style::JustifyEnd],
                    vec![Button::primary("start")
                        .on_click(|_| Msg::ClickedStart)
                        .cell()],
                ),
            ],
        ),
        None => {
            let mut msg = String::new();
            msg.push_str("Lobby for \"");
            msg.push_str(lobby.name.as_str());
            msg.push('"');

            Cell::from_str(vec![CARD_WIDTH], msg.as_str())
        }
    }
}

fn add_slot_row() -> Cell<Msg> {
    Cell::group(
        vec![CARD_WIDTH],
        vec![Button::simple("add slot")
            .on_click(|_| Msg::ClickedAddSlot)
            .cell()],
    )
}

fn open_slot(viewer_is_host: bool, at_player_count_minimum: bool) -> Cell<Msg> {
    let button_cell = if viewer_is_host {
        let label = "close";

        let button_base = if at_player_count_minimum {
            Button::disabled(label)
        } else {
            Button::simple(label)
        };

        button_base.on_click(|_| Msg::ClickedCloseSlot).cell()
    } else {
        Cell::none()
    };

    let cells = vec![
        Cell::from_str(
            vec![Style::FlexCol, Style::Grow, Style::JustifyCenter],
            "open slot",
        ),
        button_cell,
    ];

    Cell::group(
        vec![Style::BorderContent2, Style::FlexRow, Style::P4, CARD_WIDTH],
        cells,
    )
}

pub fn guest_card(
    viewer_id: Id,
    guest_id: Id,
    guest: Player,
    user_name_field: &String,
    initial_user_name_field: &String,
    viewer_is_host: bool,
) -> Cell<Msg> {
    let guest_is_viewer = viewer_id == guest_id;

    let name_row = if guest_is_viewer {
        name_field(user_name_field, &initial_user_name_field)
    } else {
        let remove_player_button = if viewer_is_host {
            Button::destructive("kick")
                .on_click(|_| Msg::ClickedKickGuest(guest_id))
                .cell()
        } else {
            Cell::none()
        };

        Row::from_cells(
            vec![Style::G4],
            vec![
                Cell::from_str(
                    vec![Style::FlexCol, Style::JustifyCenter, Style::Grow],
                    guest.name.to_string().as_str(),
                ),
                remove_player_button,
            ],
        )
    };

    player_card(guest_is_viewer, vec![name_row])
}

pub fn host_card(viewer_is_host: bool, model: &Model) -> Cell<Msg> {
    let name_row = if viewer_is_host {
        name_field(&model.name_field, &model.initial_name_field)
    } else {
        Row::from_str(model.lobby.host.name.to_string().as_str())
    };

    player_card(viewer_is_host, vec![name_row])
}

fn name_field(field: &String, initial_name: &String) -> Row<Msg> {
    Row::from_cells(
        vec![Style::G4],
        vec![
            TextField::simple(field.as_str(), |event| Msg::UpdatedNameField(event)).cell(),
            Button::simple("save")
                .disable(!can_save_name_field(field, initial_name))
                .on_click(|_| Msg::ClickedSavePlayerName)
                .cell(),
        ],
    )
}

fn can_save_name_field(field: &String, initial_field: &String) -> bool {
    let changed = field != initial_field;

    let is_not_empty = !field.is_empty();

    changed && is_not_empty
}

fn validate_name_field(field: &String, initial_field: &String) -> Result<Name, String> {
    let changed = field != initial_field;

    if changed {
        Name::from_str(field.as_str()).map_err(|err| err.to_string())
    } else {
        Err("Name is unchaged".to_string())
    }
}

fn player_card<Msg: 'static>(player_is_viewer: bool, rows: Vec<Row<Msg>>) -> Cell<Msg> {
    let styles = vec![CARD_WIDTH];

    Card::init().primary(player_is_viewer).cell(styles, rows)
}

fn center<Msg: 'static>(cell: Cell<Msg>) -> Row<Msg> {
    Row::from_cells(vec![Style::JustifyCenter], vec![cell])
}

pub const CARD_WIDTH: Style = Style::WA;

pub const PARENT_STYLES: [Style; 2] = [Style::JustifyCenter, Style::G3];
