use crate::core_ext::route::go_to_route;
use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::view::error_card::ErrorCard;
use crate::view::loading_spinner::LoadingSpinner;
use crate::web_sys::HtmlCanvasElement;
use crate::{api, core_ext, global};
use seed::prelude::{el_ref, El, ElRef, JsValue, Orders, UpdateEl};
use seed::{canvas, log};
use shared::api::endpoint::Endpoint;
use shared::api::lobby::create;
use shared::game::Game;
use shared::id::Id;
use shared::lobby::Lobby;
use shared::map::Tile;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub struct Model {
    game: Game,
    game_id: Id,
    canvas: ElRef<HtmlCanvasElement>,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Rendered,
}

///////////////////////////////////////////////////////////////
// init
///////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Flags {
    pub game: Game,
    pub game_id: Id,
}

pub fn init(flags: Flags, orders: &mut impl Orders<Msg>) -> Model {
    orders.after_next_render(|_| Msg::Rendered);

    Model {
        game: flags.game,
        game_id: flags.game_id,
        canvas: ElRef::<HtmlCanvasElement>::default(),
    }
}

///////////////////////////////////////////////////////////////
// Update
///////////////////////////////////////////////////////////////

pub fn update(global: &global::Model, msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered => {
            draw(&model.canvas);
            orders
                .after_next_render(|_render_info| Msg::Rendered)
                .skip();
        }
    }
}

fn draw(canvas: &ElRef<HtmlCanvasElement>) {
    let canvas = canvas.get().expect("get canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    // clear canvas
    ctx.begin_path();
    ctx.clear_rect(0., 0., 400., 200.);

    let width = 200.;
    let height = 100.;

    ctx.rect(0., 0., width, height);
    ctx.set_fill_style(&JsValue::from_str("red"));
    ctx.fill();

    ctx.move_to(0., 0.);
    ctx.line_to(width, height);
    ctx.stroke();
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(model: &Model) -> Cell<Msg> {
    Cell::group(vec![], vec![canvas_cell(model), overlay_view(model)])
}

fn canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(vec![], vec![canvas![el_ref(&model.canvas)]])
}

fn overlay_view(mode: &Model) -> Cell<Msg> {
    Cell::none()
}
