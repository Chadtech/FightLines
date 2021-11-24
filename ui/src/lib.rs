#![allow(clippy::wildcard_imports)]

mod api;
mod core_ext;
mod global;
mod page;
mod route;
mod style;
mod view;

use crate::page::{component_library, loading, lobby, not_found};
use crate::view::cell::{Cell, Row};
use page::title;
use page::Page;
use route::Route;
use seed::{prelude::*, *};
use style::Style;

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

    //
    UrlChanged(subs::UrlChanged),
}

///////////////////////////////////////////////////////////////
// Init //
///////////////////////////////////////////////////////////////

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders
        .subscribe(Msg::UrlChanged)
        .notify(subs::UrlChanged(url));

    Model {
        page: Page::Blank,
        global: global::Model::init(),
    }
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
    let new_page = match route {
        Route::Title => Page::Title(title::init()),
        Route::ComponentLibrary(sub_route) => {
            Page::ComponentLibrary(component_library::init(sub_route))
        }
        Route::Lobby(_) => {
            // TODO fetch lobby
            orders;

            Page::Loading
        }
    };

    model.page = new_page;
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

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
                page::lobby::update(
                    &model.global,
                    sub_msg,
                    sub_model,
                    &mut orders.proxy(Msg::Lobby),
                );
            }
        }
    }
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

fn view(model: &Model) -> Node<Msg> {
    let body: Vec<Row<Msg>> = match &model.page {
        Page::Title(sub_model) => title::view(sub_model)
            .into_iter()
            .map(|row| row.map_msg(Msg::Title))
            .collect(),

        Page::ComponentLibrary(sub_model) => component_library::view(sub_model),
        Page::Lobby(sub_model) => lobby::view(sub_model)
            .into_iter()
            .map(|row| row.map_msg(Msg::Lobby))
            .collect(),
        Page::NotFound => not_found::view(),
        Page::Blank => vec![],
        Page::Loading => loading::view(),
    };

    let mut page_styles: Vec<Style> = Vec::new();

    match &model.page {
        Page::Title(_) => page_styles.append(&mut title::PARENT_STYLES.to_vec()),
        Page::NotFound => page_styles.append(&mut not_found::PARENT_STYLES.to_vec()),
        Page::Lobby(_) => {}
        Page::ComponentLibrary(_) => {}
        Page::Blank => {}
        Page::Loading => page_styles.append(&mut loading::PARENT_STYLES.to_vec()),
    }

    page_styles.append(&mut vec![Style::Grow]);

    div![
        C!["page-container"],
        style::global_html(),
        Cell::from_rows(page_styles, body).html()
    ]
}

///////////////////////////////////////////////////////////////
// App //
///////////////////////////////////////////////////////////////

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
