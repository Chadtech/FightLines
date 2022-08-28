use crate::global;
use crate::global::WindowSize;
use crate::view::cell::Cell;
use crate::view::loading_spinner::LoadingSpinner;
use crate::web_sys::HtmlCanvasElement;
use seed::prelude::{el_ref, At, El, ElRef, IndexMap, JsValue, Orders, UpdateEl};
use seed::{attrs, canvas};
use shared::game::Game;
use shared::id::Id;

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
            draw(global.window_size(), &model.canvas);
            orders
                .after_next_render(|_render_info| Msg::Rendered)
                .skip();
        }
    }
}

fn draw(window_size: WindowSize, canvas: &ElRef<HtmlCanvasElement>) {
    let canvas = canvas.get().expect("could not get canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = window_size.width;
    let height = window_size.height;

    // clear canvas
    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

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

pub fn view(global: &global::Model, model: &Model) -> Cell<Msg> {
    Cell::group(
        vec![],
        vec![canvas_cell(global, model), overlay_view(model)],
    )
}

fn canvas_cell(global: &global::Model, model: &Model) -> Cell<Msg> {
    let window_size = global.window_size();

    let px = |n: f64| -> String {
        let mut n_str = n.to_string();
        n_str.push_str("px");

        n_str
    };

    Cell::from_html(
        vec![],
        vec![canvas![
            attrs! {
                At::Width => px(window_size.width).as_str(),
                At::Height => px(window_size.height).as_str()
            },
            el_ref(&model.canvas)
        ]],
    )
}

fn overlay_view(model: &Model) -> Cell<Msg> {
    Cell::none()
}
