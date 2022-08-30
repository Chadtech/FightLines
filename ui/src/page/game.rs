use crate::domain::point::Point;
use crate::global::WindowSize;
use crate::view::cell::Cell;
use crate::web_sys::{HtmlCanvasElement, HtmlImageElement};
use crate::{global, web_sys, Style, Toast};
use seed::prelude::{el_ref, At, El, ElRef, IndexMap, JsValue, Orders, St, ToClasses, UpdateEl};
use seed::{attrs, canvas, log, style, C};
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
    game_pixel_width: u16,
    game_pixel_height: u16,
    game_x: i16,
    game_y: i16,
    dragging_map: Option<Point<i32>>,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Rendered,
    RenderedFirstTime,
    MouseDownOnScreen(Point<i32>),
    MouseUpOnScreen(Point<i32>),
    MouseMoveOnScreen(Point<i32>),
}

impl Model {
    pub fn adjust_map_position_by_mouse_position(&mut self, page_pos: Point<i32>) {
        if let Some(original_page_pos) = &self.dragging_map {
            let dx = page_pos.x - original_page_pos.x;
            let dy = page_pos.y - original_page_pos.y;

            self.game_x += dx as i16;
            self.game_y += dy as i16;

            self.dragging_map = Some(page_pos);
        }
    }
}
///////////////////////////////////////////////////////////////
// init
///////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Flags {
    pub game: Game,
    pub game_id: Id,
}

pub fn init(
    global: &global::Model,
    flags: Flags,
    orders: &mut impl Orders<Msg>,
) -> Result<Model, String> {
    let window = web_sys::window().ok_or("Cannot find window".to_string())?;

    let document = window.document().ok_or("Cannot get document".to_string())?;

    let grass_tile_asset: HtmlImageElement = HtmlImageElement::from(JsValue::from(
        document
            .get_element_by_id(Sprite::GrassTile.html_id().as_str())
            .ok_or("Cannot find grass tile asset".to_string())?,
    ));

    let window_size = global.window_size();

    let game_pixel_width = ((flags.game.map.width.clone() as u16) * tile::PIXEL_WIDTH);
    let game_pixel_height = ((flags.game.map.height.clone() as u16) * tile::PIXEL_HEIGHT);

    let mut game_x = window_size.width / 2.0;
    game_x -= (game_pixel_width as f64) / 2.0;

    let mut game_y = window_size.height / 2.0;
    game_y -= (game_pixel_height as f64) / 2.0;

    orders.after_next_render(|_| Msg::Rendered);
    orders.after_next_render(|_| Msg::RenderedFirstTime);

    let model = Model {
        game: flags.game.clone(),
        game_id: flags.game_id,
        canvas: ElRef::<HtmlCanvasElement>::default(),
        grass_tile_asset,
        game_pixel_width,
        game_pixel_height,
        game_x: game_x as i16,
        game_y: game_y as i16,
        dragging_map: None,
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
        Msg::MouseDownOnScreen(page_pos) => {
            model.dragging_map = Some(page_pos);
        }
        Msg::MouseUpOnScreen(page_pos) => {
            model.adjust_map_position_by_mouse_position(page_pos);
            model.dragging_map = None;
        }
        Msg::MouseMoveOnScreen(page_pos) => {
            model.adjust_map_position_by_mouse_position(page_pos);
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
        vec![map_canvas_cell(model), click_screen(), overlay_view(model)],
    )
}

fn map_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(
        vec![],
        vec![canvas![
            C![Style::Absolute.css_classes().join("")],
            attrs! {
                At::Width => px_u16(model.game_pixel_width).as_str(),
                At::Height => px_u16(model.game_pixel_height).as_str()
            },
            style! {
                St::Left => px_i16(model.game_x).as_str(),
                St::Top => px_i16(model.game_y).as_str()
            },
            el_ref(&model.canvas)
        ]],
    )
}

fn px_u16(n: u16) -> String {
    let mut n_str = n.to_string();
    n_str.push_str("px");

    n_str
}

fn px_i16(n: i16) -> String {
    let mut n_str = n.to_string();
    n_str.push_str("px");

    n_str
}

fn click_screen() -> Cell<Msg> {
    Cell::group(
        vec![
            Style::Absolute,
            Style::Left0,
            Style::Right0,
            Style::Top0,
            Style::Bottom0,
        ],
        vec![],
    )
    .on_mouse_down(|event| {
        Msg::MouseDownOnScreen(Point {
            x: event.page_x(),
            y: event.page_y(),
        })
    })
    .on_mouse_up(|event| {
        Msg::MouseUpOnScreen(Point {
            x: event.page_x(),
            y: event.page_y(),
        })
    })
    .on_mouse_move(|event| {
        Msg::MouseMoveOnScreen(Point {
            x: event.page_x(),
            y: event.page_y(),
        })
    })
}

fn overlay_view(model: &Model) -> Cell<Msg> {
    Cell::none()
}
