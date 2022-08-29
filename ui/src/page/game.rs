use crate::global::WindowSize;
use crate::view::cell::Cell;
use crate::web_sys::{HtmlCanvasElement, HtmlImageElement};
use crate::{global, web_sys, Toast};
use seed::prelude::{el_ref, At, El, ElRef, IndexMap, JsValue, Orders, UpdateEl};
use seed::{attrs, canvas};
use shared::game::Game;
use shared::id::Id;
use shared::sprite::Sprite;
use shared::tile;

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
    RenderedFirstTime,
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
    orders.after_next_render(|_| Msg::RenderedFirstTime);

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

pub fn update(
    global: &mut global::Model,
    msg: Msg,
    model: &mut Model,
    orders: &mut impl Orders<Msg>,
) {
    match msg {
        Msg::Rendered => {
            orders
                .after_next_render(|_render_info| Msg::Rendered)
                .skip();
        }
        Msg::RenderedFirstTime => {
            let draw_result = draw_map(global.window_size(), &model);

            if let Err(err) = draw_result {
                global.toast(
                    Toast::init("error", "map rendering problem")
                        .error()
                        .with_more_info(err.as_str()),
                );
            }
        }
    }
}

fn draw_map(window_size: WindowSize, model: &Model) -> Result<(), String> {
    let canvas = model.canvas.get().expect("could not get canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = window_size.width;
    let height = window_size.height;

    // clear canvas
    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    let grid = &model.game.map.grid;

    for row in grid {
        for cell in row {
            let x = (cell.x * tile::PIXEL_WIDTH) as f64;
            let y = (cell.y * tile::PIXEL_HEIGHT) as f64;

            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                &model.grass_tile_asset,
                x,
                y,
                tile::PIXEL_WIDTH_FL,
                tile::PIXEL_HEIGHT_FL,
            )
            .map_err(|_| "Could not draw image on canvas".to_string())?;
        }
    }

    Ok(())
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(global: &global::Model, model: &Model) -> Cell<Msg> {
    Cell::group(
        vec![],
        vec![map_canvas_cell(global, model), overlay_view(model)],
    )
}

fn map_canvas_cell(global: &global::Model, model: &Model) -> Cell<Msg> {
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
