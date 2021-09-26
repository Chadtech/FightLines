#![allow(clippy::wildcard_imports)]

mod global;
mod page;
mod route;
mod view;

use page::title;
use page::Page;
use route::Route;
use seed::{prelude::*, *};

///////////////////////////////////////////////////////////////
// Init
///////////////////////////////////////////////////////////////

fn init(url: Url, _: &mut impl Orders<Msg>) -> Model {
    let maybe_route = Route::from_url(url);

    let page = match maybe_route {
        None => Page::NotFound,
        Some(route) => match route {
            Route::Title => Page::Title(title::init()),
        },
    };

    Model {
        page,
        global: global::init(),
    }
}

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

struct Model {
    page: Page,
    global: global::Model,
}

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
#[derive(Copy, Clone)]
enum Msg {
    Msg,
}

///////////////////////////////////////////////////////////////
// Update
///////////////////////////////////////////////////////////////

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Msg => {}
    }
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    let body: Vec<Box<Msg>> = match &model.page {
        Page::Title(sub_model) => title::view(sub_model),
        Page::NotFound => Vec::new(),
    };
    // div![
    //     "This is a counter: ",
    //     C!["counter"],
    //     // button![model.counter, ev(Ev::Click, |_| Msg::Increment),],
    // ]

    div![nodes![body]].map_msg()
}

///////////////////////////////////////////////////////////////
// App
///////////////////////////////////////////////////////////////

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
