#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

use page::Page;
use route::Route;
use shared::api::endpoint::Endpoint;
use shared::api::game::get as get_game;
use shared::api::lobby::join as join_lobby;
use shared::lobby::Lobby;
use style::Style;

use crate::page::lobby::InitError;
use crate::page::{component_library, error, game, kicked, loading, lobby, not_found, title};
use crate::view::cell::{Cell, Row};
use crate::view::toast;
use crate::view::toast::Toast;

mod api;
mod assets;
mod core_ext;
mod domain;
mod global;
mod page;
mod route;
mod style;
mod view;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

struct Model {
    page: Page,
    global: global::Model,
}

#[derive(Clone)]
enum Msg {
    // Pages
    Title(title::Msg),
    Lobby(lobby::Msg),
    Error(error::Msg),
    Kicked(kicked::Msg),
    Game(game::Msg),

    // Page Loads
    LoadedLobby(Box<Result<lobby::Flags, String>>),
    LoadedGame(Box<Result<game::Flags, String>>),
    //
    UrlChanged(subs::UrlChanged),
    Global(global::Msg),
    Toast(toast::Msg),
    Assets(assets::Msg),
}

///////////////////////////////////////////////////////////////
// Init //
///////////////////////////////////////////////////////////////

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Result<Model, String> {
    orders
        .subscribe(Msg::UrlChanged)
        .notify(subs::UrlChanged(url));

    let global_result = global::Model::init(&mut orders.proxy(Msg::Global));

    global_result.map(|global| Model {
        page: Page::Blank,
        global,
    })
}

///////////////////////////////////////////////////////////////
// Routing //
///////////////////////////////////////////////////////////////

fn handle_url_change(url: Url, model: &mut Model, orders: &mut impl Orders<Msg>) {
    let maybe_route = Route::from_url(url);
    match maybe_route {
        None => {
            model.page = Page::NotFound;
        }

        Some(route) => handle_route_change(route, model, orders),
    };
}

fn handle_route_change(route: Route, model: &mut Model, orders: &mut impl Orders<Msg>) {
    let new_page: Page = match route {
        Route::Title => Page::Title(title::Model::init()),
        Route::ComponentLibrary(sub_route) => {
            Page::ComponentLibrary(component_library::init(sub_route))
        }
        Route::Lobby(lobby_id) => {
            let mut already_loaded_lobby: Option<Lobby> = None;

            let mut join_lobby = || {
                let req: join_lobby::Request = join_lobby::Request {
                    guest_id: model.global.viewer_id(),
                    guest_name: model.global.viewer_name.clone(),
                    lobby_id: lobby_id.clone(),
                };

                let url = Endpoint::join_lobby(lobby_id.clone());

                match req.to_bytes() {
                    Ok(bytes) => {
                        orders.skip().perform_cmd({
                            async {
                                let result = match api::post(url, bytes).await {
                                    Ok(res_bytes) => {
                                        match join_lobby::Response::from_bytes(res_bytes) {
                                            Ok(res) => {
                                                let flags = lobby::Flags {
                                                    lobby_id: res.lobby_id,
                                                    lobby: res.lobby,
                                                };

                                                Ok(flags)
                                            }
                                            Err(err) => Err(err.to_string()),
                                        }
                                    }
                                    Err(error) => {
                                        let fetch_error =
                                            core_ext::http::fetch_error_to_string(error);
                                        Err(fetch_error)
                                    }
                                };

                                Msg::LoadedLobby(Box::new(result))
                            }
                        });
                    }

                    Err(_err) => {
                        todo!("Handle this error somehow")
                    }
                }
            };

            match &model.page {
                Page::Title(sub_model) => match sub_model.just_created_lobby() {
                    Some((existing_lobby_id, lobby)) => {
                        if existing_lobby_id != &lobby_id {
                            join_lobby()
                        } else {
                            already_loaded_lobby = Some(lobby.clone());
                        }
                    }
                    None => {
                        join_lobby();
                    }
                },
                _ => {
                    join_lobby();
                }
            };

            match already_loaded_lobby {
                Some(lobby) => {
                    let flags = lobby::Flags { lobby_id, lobby };

                    match lobby::Model::init(&model.global, flags, &mut orders.proxy(Msg::Lobby)) {
                        Err(error) => match error {
                            InitError::PlayerIsKicked => Page::Kicked,
                        },

                        Ok(sub_model) => Page::Lobby(sub_model),
                    }
                }
                None => Page::Loading,
            }
        }
        Route::Kicked => Page::Kicked,
        Route::Game(id) => {
            let maybe_game =
                match &model.page {
                    Page::Lobby(sub_model) => {
                        sub_model.started_game().and_then(|(game_id, game)| {
                            if game_id == id {
                                Some(game)
                            } else {
                                None
                            }
                        })
                    }
                    _ => None,
                };

            match maybe_game {
                None => {
                    let url = Endpoint::make_get_game(id.clone());

                    orders.skip().perform_cmd({
                        async {
                            let result = match api::get(url).await {
                                Ok(res_bytes) => match get_game::Response::from_bytes(res_bytes) {
                                    Ok(res) => {
                                        let flags = game::Flags {
                                            game_id: res.get_game_id(),
                                            game: res.get_game(),
                                        };

                                        Ok(flags)
                                    }
                                    Err(err) => Err(err.to_string()),
                                },
                                Err(error) => {
                                    let fetch_error = core_ext::http::fetch_error_to_string(error);
                                    Err(fetch_error)
                                }
                            };

                            Msg::LoadedGame(Box::new(result))
                        }
                    });
                    Page::Loading
                }
                Some(game) => {
                    let sub_model_result = game::init(
                        &model.global,
                        game::Flags {
                            game: game.clone(),
                            game_id: id.clone(),
                        },
                        &mut orders.proxy(Msg::Game),
                    );

                    match sub_model_result {
                        Ok(sub_model) => Page::Game(sub_model),
                        Err(err) => {
                            let flags =
                                error::Flags::from_title("Failed to load game").with_msg(err);

                            Page::Error(error::init(flags))
                        }
                    }
                }
            }
        }
    };

    model.page = new_page;
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

fn super_update(msg: Msg, model_result: &mut Result<Model, String>, orders: &mut impl Orders<Msg>) {
    if let Ok(model) = model_result {
        update(msg, model, orders)
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Title(sub_msg) => {
            if let Page::Title(sub_model) = &mut model.page {
                page::title::update(
                    &model.global,
                    sub_msg,
                    sub_model,
                    &mut orders.proxy(Msg::Title),
                );
            }
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            handle_url_change(url, model, orders);
        }
        Msg::Lobby(sub_msg) => {
            if let Page::Lobby(sub_model) = &mut model.page {
                lobby::update(
                    &mut model.global,
                    sub_msg,
                    sub_model,
                    &mut orders.proxy(Msg::Lobby),
                );
            }
        }
        Msg::LoadedLobby(result) => match *result {
            Ok(flags) => {
                match lobby::Model::init(&model.global, flags, &mut orders.proxy(Msg::Lobby)) {
                    Ok(sub_model) => {
                        model.page = Page::Lobby(sub_model);
                    }
                    Err(error) => match error {
                        InitError::PlayerIsKicked => {
                            model.page = Page::Kicked;
                        }
                    },
                }
            }
            Err(error) => {
                let flags = error::Flags::from_title("could not load lobby").with_msg(error);

                model.page = Page::Error(error::init(flags));
            }
        },
        Msg::Error(sub_msg) => {
            error::update(sub_msg, &mut orders.proxy(Msg::Error));
        }
        Msg::Global(sub_msg) => {
            global::update(sub_msg, &mut model.global, &mut orders.proxy(Msg::Global));
        }
        Msg::Toast(sub_msg) => {
            global::update_from_toast_msg(sub_msg, &mut model.global);
        }
        Msg::Kicked(sub_msg) => kicked::update(sub_msg, &mut orders.proxy(Msg::Kicked)),
        Msg::Game(sub_msg) => {
            if let Page::Game(sub_model) = &mut model.page {
                game::update(
                    &mut model.global,
                    sub_msg,
                    sub_model,
                    &mut orders.proxy(Msg::Game),
                )
            }
        }
        Msg::Assets(_) => {}
        Msg::LoadedGame(result) => match *result {
            Ok(flags) => {
                let new_page = match game::init(&model.global, flags, &mut orders.proxy(Msg::Game))
                {
                    Ok(sub_model) => Page::Game(sub_model),
                    Err(err) => Page::Error(error::init(
                        error::Flags::from_title("Failed to load game page").with_msg(err),
                    )),
                };
                model.page = new_page;
            }
            Err(error) => {
                let flags = error::Flags::from_title("could not load game").with_msg(error);

                model.page = Page::Error(error::init(flags));
            }
        },
    }
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

fn super_view(model_result: &Result<Model, String>) -> Node<Msg> {
    match model_result {
        Ok(model) => view(model),
        Err(err) => {
            div![err]
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    let page_body = global::open_toast_view(&model.global)
        .map(|cell| cell.map_msg(Msg::Global))
        .unwrap_or_else(|| {
            let body: Vec<Row<Msg>> = match &model.page {
                Page::Title(sub_model) => title::view(sub_model)
                    .into_iter()
                    .map(|row| row.map_msg(Msg::Title))
                    .collect(),

                Page::ComponentLibrary(sub_model) => component_library::view(sub_model),
                Page::Lobby(sub_model) => lobby::view(&model.global, sub_model)
                    .into_iter()
                    .map(|row| row.map_msg(Msg::Lobby))
                    .collect(),
                Page::NotFound => not_found::view(),
                Page::Blank => vec![],
                Page::Loading => loading::view(),
                Page::Error(sub_model) => error::view(sub_model)
                    .into_iter()
                    .map(|row| row.map_msg(Msg::Error))
                    .collect(),
                Page::Kicked => kicked::view()
                    .into_iter()
                    .map(|row| row.map_msg(Msg::Kicked))
                    .collect(),
                Page::Game(sub_model) => vec![Row::from_cells(
                    vec![],
                    vec![game::view(&model.global, sub_model).map_msg(Msg::Game)],
                )],
            };

            let mut page_styles: Vec<Style> = Vec::new();

            let mut from_page = match &model.page {
                Page::Title(_) => title::PARENT_STYLES.to_vec(),
                Page::NotFound => not_found::PARENT_STYLES.to_vec(),
                Page::Lobby(_) => lobby::PARENT_STYLES.to_vec(),
                Page::ComponentLibrary(_) => vec![],
                Page::Blank => vec![],
                Page::Loading => loading::PARENT_STYLES.to_vec(),
                Page::Error(_) => error::PARENT_STYLES.to_vec(),
                Page::Kicked => kicked::PARENT_STYLES.to_vec(),
                Page::Game(_) => vec![],
            };

            page_styles.append(&mut from_page);
            page_styles.append(&mut vec![Style::Grow]);

            Cell::from_rows(page_styles, body)
        });

    div![
        C!["page-container"],
        style::global_html(),
        page_body.html(),
        Toast::many_to_html(model.global.first_toast_hidden(), model.global.toasts())
            .map_msg(Msg::Toast),
        assets::view().map_msg(Msg::Assets).html(),
    ]
}

///////////////////////////////////////////////////////////////
// App //
///////////////////////////////////////////////////////////////

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, super_update, super_view);
}
