use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::view::text_field::TextField;
use crate::{api, core_ext, global};
use seed::browser::web_socket::CloseEvent;
use seed::prelude::{Orders, WebSocket, WebSocketMessage};
use seed::{log, spawn_local};
use shared::api::endpoint::Endpoint;
use shared::api::lobby::update as lobby_update;
use shared::id::Id;
use shared::lobby;
use shared::lobby::{Lobby, MAX_GUESTS};
use shared::player::Player;
use std::rc::Rc;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    lobby_id: Id,
    lobby: Lobby,
    name_field: String,
    host_model: Option<HostModel>,
    web_socket: WebSocket,
}

struct HostModel {
    game_name_field: String,
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

    // WebSocket
    OpenedWebSocket,
    ClosedWebSocket(CloseEvent),
    WebSocketErrored,
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
            let host_model = HostModel {
                game_name_field: lobby.name,
            };

            Some(host_model)
        } else {
            None
        }
    }
}

fn create_websocket(orders: &impl Orders<Msg>) -> WebSocket {
    let msg_sender = orders.msg_sender();

    let mut url = "ws://localhost:8080".to_string();
    url.push_str(Endpoint::LobbyWebsocket.to_url().as_str());

    WebSocket::builder(url, orders)
        .on_open(|| Msg::OpenedWebSocket)
        .on_message(move |msg| decode_message(msg, msg_sender))
        .on_close(Msg::ClosedWebSocket)
        .on_error(|| Msg::WebSocketErrored)
        .build_and_open()
        .unwrap()
}

fn decode_message(message: WebSocketMessage, msg_sender: Rc<dyn Fn(Option<Msg>)>) {
    spawn_local(async move {
        let bytes = message
            .bytes()
            .await
            .expect("WebsocketError on binary data");

        log!("Bytes!");
        log!(bytes);
        // let msg: shared::ServerMessage = rmp_serde::from_slice(&bytes).unwrap();
        // msg_sender(Some(Msg::BinaryMessageReceived(msg)));
    });
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

impl Model {
    pub fn init(global: &global::Model, flags: Flags, orders: &mut impl Orders<Msg>) -> Model {
        let name_field = global.viewer_name.to_string();
        Model {
            lobby_id: flags.lobby_id,
            lobby: flags.lobby.clone(),
            name_field,
            host_model: HostModel::init(global, flags.lobby),
            web_socket: create_websocket(orders),
        }
    }
    pub fn viewer_is_host(&self) -> bool {
        self.host_model.is_some()
    }
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

pub fn update(_global: &global::Model, msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClickedAddSlot => {
            send_updates(model.lobby_id.clone(), vec![lobby::Update::AddSlot], orders);
        }
        Msg::ClickedCloseSlot => {
            if !model.lobby.at_player_count_minimum() {
                send_updates(
                    model.lobby_id.clone(),
                    vec![lobby::Update::CloseSlot],
                    orders,
                );
            }
        }
        Msg::UpdatedLobby(result) => match result {
            Ok(res) => {
                model.lobby = res.get_lobby();
            }
            Err(_) => {}
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
                // TODO
            }
        }
        Msg::ClickedSavePlayerName => {
            // TODO
        }
        Msg::ClickedKickGuest(guest_id) => {
            // TODO
        }
        Msg::ClickedStart => {
            if let Some(host_model) = &model.host_model {
                // TODO
            }
        }
        Msg::OpenedWebSocket => {
            log!("WebSocket connection is open now");
        }
        Msg::ClosedWebSocket(close_event) => {
            log!("==================");
            log!("WebSocket connection was closed:");
            log!("Clean:", close_event.was_clean());
            log!("Code:", close_event.code());
            log!("Reason:", close_event.reason());
            log!("==================");
        }
        Msg::WebSocketErrored => {
            log!("WebSocket failed");
        }
    }
}

fn send_updates(lobby_id: Id, upts: Vec<lobby::Update>, orders: &mut impl Orders<Msg>) {
    let req = lobby_update::Request {
        lobby_id: lobby_id.clone(),
        updates: upts,
    };

    match req.to_bytes() {
        Ok(request_bytes) => {
            orders.skip().perform_cmd({
                async {
                    let result = match api::post(Endpoint::update_lobby().to_url(), request_bytes)
                        .await
                    {
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
        Err(_) => {
            // TODO
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
    viewer_name_field: &String,
    viewer_is_host: bool,
) -> Cell<Msg> {
    let guest_is_viewer = viewer_id == guest_id;

    let name_row = if guest_is_viewer {
        name_field(viewer_name_field)
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
        name_field(&model.name_field)
    } else {
        Row::from_str(model.lobby.host.name.to_string().as_str())
    };

    player_card(viewer_is_host, vec![name_row])
}

fn name_field(field: &String) -> Row<Msg> {
    Row::from_cells(
        vec![Style::G4],
        vec![
            TextField::simple(field.as_str(), |event| Msg::UpdatedNameField(event)).cell(),
            Button::simple("save")
                .on_click(|_| Msg::ClickedSavePlayerName)
                .cell(),
        ],
    )
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
