use crate::global::WindowSize;
use crate::view::cell::Cell;
use crate::view::loading_spinner::LoadingSpinner;
use crate::web_sys::{HtmlCanvasElement, HtmlImageElement};
use crate::{global, web_sys};
use seed::prelude::{el_ref, At, El, ElRef, IndexMap, JsValue, Orders, UpdateEl};
use seed::{attrs, canvas};
use shared::game::Game;
use shared::id::Id;
use shared::sprite::Sprite;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub struct Model {
    game: Game,
    game_id: Id,
    canvas: ElRef<HtmlCanvasElement>,
    grass_tile_asset: HtmlImageElement,
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

pub fn init(flags: Flags, orders: &mut impl Orders<Msg>) -> Result<Model, String> {
    let window = web_sys::window().ok_or("Cannot find window".to_string())?;

    let document = window.document().ok_or("Cannot get document".to_string())?;

    let grass_tile_asset: HtmlImageElement = HtmlImageElement::from(JsValue::from(
        document
            .get_element_by_id(Sprite::GrassTile.html_id().as_str())
            .ok_or("Cannot find grass tile asset".to_string())?,
    ));

    orders.after_next_render(|_| Msg::Rendered);

    let model = Model {
        game: flags.game,
        game_id: flags.game_id,
        canvas: ElRef::<HtmlCanvasElement>::default(),
        grass_tile_asset,
    };

    Ok(model)
}

///////////////////////////////////////////////////////////////
// Update
///////////////////////////////////////////////////////////////

pub fn update(global: &global::Model, msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Rendered => {
            draw(global.window_size(), &model.canvas, &model.grass_tile_asset);
            orders
                .after_next_render(|_render_info| Msg::Rendered)
                .skip();
        }
    }
}

fn draw(
    window_size: WindowSize,
    canvas: &ElRef<HtmlCanvasElement>,
    grass_tile_asset: &HtmlImageElement,
) -> Result<(), String> {
    let canvas = canvas.get().expect("could not get canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = window_size.width;
    let height = window_size.height;

    // clear canvas
    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    // ctx.rect(0., 0., width, height);
    // ctx.set_fill_style(&JsValue::from_str("red"));
    // ctx.fill();
    //
    // ctx.move_to(0., 0.);
    // ctx.line_to(width, height);
    // ctx.stroke();

    ctx.draw_image_with_html_image_element(grass_tile_asset, 50.0, 50.0)
        .map_err(|_| "Could not draw image on canvas".to_string())?;

    Ok(())
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
