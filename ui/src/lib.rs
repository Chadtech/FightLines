#![allow(clippy::wildcard_imports)]

mod global;
mod page;

use page::Page;
use seed::{prelude::*, *};

///////////////////////////////////////////////////////////////
// Init
///////////////////////////////////////////////////////////////

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        page: Page::Welcome,
        global: global::Model::init(),
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
    div![
        "This is a counter: ",
        C!["counter"],
        // button![model.counter, ev(Ev::Click, |_| Msg::Increment),],
    ]
}

///////////////////////////////////////////////////////////////
// App
///////////////////////////////////////////////////////////////

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
