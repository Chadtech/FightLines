#![allow(clippy::wildcard_imports)]

mod global;
mod page;
mod route;
mod style;
mod view;

use crate::page::{component_library, not_found};
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
    TitleMsg(title::Msg),
    UrlChanged(subs::UrlChanged),
}

///////////////////////////////////////////////////////////////
// Init //
///////////////////////////////////////////////////////////////

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders
        .subscribe(Msg::UrlChanged)
        .notify(subs::UrlChanged(url.clone()));

    Model {
        page: Page::Blank,
        global: global::init(),
    }
}

///////////////////////////////////////////////////////////////
// Routing //
///////////////////////////////////////////////////////////////

fn handle_url_change(url: Url, model: &mut Model) {
    let maybe_route = Route::from_url(url);
    match maybe_route {
        None => {
            model.page = Page::NotFound;
        }

        Some(route) => handle_route_change(route, model),
    };
}

fn handle_route_change(route: Route, model: &mut Model) {
    let new_page = match route {
        Route::Title => Page::Title(title::init()),
        Route::ComponentLibrary(sub_route) => {
            Page::ComponentLibrary(component_library::init(sub_route))
        }
    };

    model.page = new_page;
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::TitleMsg(sub_msg) => {
            if let Page::Title(mut sub_model) = &model.page {
                page::title::update(sub_msg, &mut sub_model);
            }
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            handle_url_change(url, model);
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
            .map(|row| row.map_msg(Msg::TitleMsg))
            .collect(),
        Page::NotFound => not_found::view(),
        Page::ComponentLibrary(sub_model) => component_library::view(sub_model),
        Page::Blank => vec![],
    };

    let mut page_styles: Vec<Style> = Vec::new();

    match &model.page {
        Page::Title(_) => page_styles.append(&mut title::parent_styles()),
        Page::NotFound => page_styles.append(&mut not_found::parent_styles()),
        Page::ComponentLibrary(_) => {}
        Page::Blank => {}
    }

    page_styles.append(&mut vec![Style::Grow]);

    div![
        C!["page-container"],
        style::global_html(),
        Cell::from_rows(page_styles, body).to_html()
    ]
}

///////////////////////////////////////////////////////////////
// App //
///////////////////////////////////////////////////////////////

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
