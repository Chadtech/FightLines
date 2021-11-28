use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::{api, core_ext, global};
use seed::log;
use seed::prelude::Orders;
use shared::api::endpoint::Endpoint;
use shared::api::lobby::update as lobby_update;
use shared::id::Id;
use shared::lobby;
use shared::lobby::Lobby;
use shared::player::Player;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    lobby_id: Id,
    lobby: Lobby,
}

#[derive(Clone, Debug)]
pub enum Msg {
    ClickedAddSlot,
    ClickedCloseSlot,
    UpdatedLobby(Result<lobby_update::Response, String>),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub lobby_id: Id,
    pub lobby: Lobby,
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

impl Model {
    pub fn init(flags: Flags) -> Model {
        Model {
            lobby_id: flags.lobby_id,
            lobby: flags.lobby,
        }
    }
    pub fn viewer_is_host(&self, global: &global::Model) -> bool {
        self.lobby.host_id == global.viewer_id()
    }
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

pub fn update(_global: &global::Model, msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    log!(msg);
    match msg {
        Msg::ClickedAddSlot => {
            send_updates(model.lobby_id.clone(), vec![lobby::Update::AddSlot], orders);
        }
        Msg::ClickedCloseSlot => {
            send_updates(
                model.lobby_id.clone(),
                vec![lobby::Update::CloseSlot],
                orders,
            );
        }
        Msg::UpdatedLobby(result) => match result {
            Ok(res) => {
                model.lobby = res.get_lobby();
            }
            Err(_) => {}
        },
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
        Err(_) => {}
    };
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn view(global: &global::Model, model: &Model) -> Vec<Row<Msg>> {
    let mut rows = Vec::new();

    let viewer_is_host = model.viewer_is_host(global);

    let lobby = &model.lobby;

    let guests = &lobby.guests;

    rows.push(center(header(viewer_is_host, lobby.clone())));

    rows.push(center(host_card(viewer_is_host, model)));

    for (_, guest) in guests.into_iter() {
        let guest_row = center(guest_card(guest.clone()));

        rows.push(guest_row)
    }

    let max_num_guests = lobby.num_players_limit - 1;

    for _ in 0..(max_num_guests - lobby.num_guests()) {
        rows.push(center(open_slot(viewer_is_host)))
    }

    if lobby.num_guests() < max_num_guests && viewer_is_host {
        rows.push(center(add_slot_row()))
    }

    rows
}

fn header(viewer_is_host: bool, lobby: Lobby) -> Cell<Msg> {
    let mut msg = String::new();
    msg.push_str("Lobby for \"");
    msg.push_str(lobby.name.as_str());
    msg.push('"');

    Cell::from_str(vec![CARD_WIDTH], msg.as_str())
}

fn add_slot_row() -> Cell<Msg> {
    Cell::group(
        vec![CARD_WIDTH],
        vec![Button::simple("add slot")
            .on_click(|_| Msg::ClickedAddSlot)
            .cell()],
    )
}

fn open_slot(viewer_is_host: bool) -> Cell<Msg> {
    let cells = vec![
        Cell::from_str(
            vec![Style::FlexCol, Style::Grow, Style::JustifyCenter],
            "open slot",
        ),
        Button::simple("close")
            .on_click(|_| Msg::ClickedCloseSlot)
            .cell(),
    ];

    Cell::group(
        vec![Style::BorderContent2, Style::FlexRow, Style::P4, CARD_WIDTH],
        cells,
    )
}

pub fn guest_card(guest: Player) -> Cell<Msg> {
    let name_row = Row::from_str(guest.name.to_string().as_str());

    player_card(vec![name_row])
}

pub fn host_card(viewer_is_host: bool, model: &Model) -> Cell<Msg> {
    let name_row = if viewer_is_host {
        Row::from_cells(vec![], vec![])
    } else {
        Row::from_str(model.lobby.host.name.to_string().as_str())
    };

    player_card(vec![name_row])
}

pub fn player_card<Msg: 'static>(rows: Vec<Row<Msg>>) -> Cell<Msg> {
    Card::cell_from_rows(vec![CARD_WIDTH], rows)
}

fn center<Msg: 'static>(cell: Cell<Msg>) -> Row<Msg> {
    Row::from_cells(vec![Style::JustifyCenter], vec![cell])
}

pub const CARD_WIDTH: Style = Style::WA;

pub const PARENT_STYLES: [Style; 2] = [Style::JustifyCenter, Style::G3];
