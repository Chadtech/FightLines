use crate::domain::point::Point;
use crate::view::cell::Cell;
use crate::web_sys::{HtmlCanvasElement, HtmlImageElement};
use crate::{assets, global, web_sys, Style, Toast};
use seed::app::{CmdHandle, RenderInfo};
use seed::prelude::{
    cmds, el_ref, At, El, ElRef, IndexMap, JsValue, Node, Orders, St, ToClasses, UpdateEl,
};
use seed::{attrs, canvas, log, style, C};
use shared::facing_direction::FacingDirection;
use shared::frame_count::FrameCount;
use shared::game::Game;
use shared::id::Id;
use shared::sprite::Sprite;
use shared::tile;
use shared::tile::Tile;
use shared::unit::Unit;

///////////////////////////////////////////////////////////////
// Helpers //
///////////////////////////////////////////////////////////////

fn wait_for_timeout(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(MIN_RENDER_TIME, || {
        Msg::MinimumRenderTimeExpired
    }))
}

const MIN_RENDER_TIME: u32 = 256;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    game: Game,
    game_id: Id,
    map_canvas: ElRef<HtmlCanvasElement>,
    units_canvas: ElRef<HtmlCanvasElement>,
    assets: assets::Model,
    game_pixel_width: u16,
    game_pixel_height: u16,
    game_x: i16,
    game_y: i16,
    dragging_map: Option<Point<i32>>,
    handle_minimum_framerate_timeout: CmdHandle,
    frame_count: FrameCount,
}

#[derive(Clone, Debug)]
pub enum Msg {
    RenderedFirstTime,
    MouseDownOnScreen(Point<i32>),
    MouseUpOnScreen(Point<i32>),
    MouseMoveOnScreen(Point<i32>),
    MinimumRenderTimeExpired,
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

    let window_size = global.window_size();

    let game_pixel_width = (flags.game.map.width.clone() as u16) * tile::PIXEL_WIDTH;
    let game_pixel_height = (flags.game.map.height.clone() as u16) * tile::PIXEL_HEIGHT;

    let mut game_x = window_size.width / 2.0;
    game_x -= (game_pixel_width as f64) / 2.0;

    let mut game_y = window_size.height / 2.0;
    game_y -= (game_pixel_height as f64) / 2.0;

    orders.after_next_render(|_| Msg::RenderedFirstTime);

    let assets = assets::init()?;

    let model = Model {
        game: flags.game.clone(),
        game_id: flags.game_id,
        map_canvas: ElRef::<HtmlCanvasElement>::default(),
        units_canvas: ElRef::<HtmlCanvasElement>::default(),
        assets,
        game_pixel_width,
        game_pixel_height,
        game_x: game_x as i16,
        game_y: game_y as i16,
        dragging_map: None,
        handle_minimum_framerate_timeout: wait_for_timeout(orders),
        frame_count: FrameCount::F1,
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
        Msg::RenderedFirstTime => {
            let map_draw_result = draw_map(&model);
            let unit_draw_result = draw_units(&model);

            if let Err(err) = map_draw_result {
                global.toast(
                    Toast::init("error", "map rendering problem")
                        .error()
                        .with_more_info(err.as_str()),
                );
            }

            if let Err(err) = unit_draw_result {
                global.toast(
                    Toast::init("error", "units rendering problem")
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
        Msg::MinimumRenderTimeExpired => {
            model.frame_count = model.frame_count.succ();

            let draw_result = draw_units(&model);

            if let Err(err) = draw_result {
                global.toast(
                    Toast::init("error", "units rendering problem")
                        .error()
                        .with_more_info(err.as_str()),
                );
            }
            model.handle_minimum_framerate_timeout = wait_for_timeout(orders);
        }
    }
}

fn draw_units(model: &Model) -> Result<(), String> {
    let canvas = model
        .units_canvas
        .get()
        .expect("could not get units canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = model.game_pixel_width as f64;
    let height = model.game_pixel_height as f64;

    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    for located_unit in model.game.units.values() {
        let x = (located_unit.x * tile::PIXEL_WIDTH) as f64;
        let y = (located_unit.y * tile::PIXEL_HEIGHT) as f64;

        let unit_model = located_unit.clone().value;

        let assets = &model.assets;
        let asset = match unit_model.unit {
            Unit::Infantry => match unit_model.facing {
                FacingDirection::Left => match model.frame_count {
                    FrameCount::F1 => &assets.infantry1_l,
                    FrameCount::F2 => &assets.infantry2_l,
                    FrameCount::F3 => &assets.infantry3_l,
                    FrameCount::F4 => &assets.infantry4_l,
                },
                FacingDirection::Right => match model.frame_count {
                    FrameCount::F1 => &assets.infantry1,
                    FrameCount::F2 => &assets.infantry2,
                    FrameCount::F3 => &assets.infantry3,
                    FrameCount::F4 => &assets.infantry4,
                },
            },
        };

        ctx.draw_image_with_html_image_element_and_dw_and_dh(
            asset,
            x,
            y,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
        )
        .map_err(|_| "Could not draw unit image on canvas".to_string())?;
    }

    Ok(())
}

fn draw_map(model: &Model) -> Result<(), String> {
    let canvas = model
        .map_canvas
        .get()
        .expect("could not get map canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = model.game_pixel_width as f64;
    let height = model.game_pixel_height as f64;

    // clear canvas
    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    let grid = &model.game.map.grid;

    for row in grid {
        for located_tile in row {
            let x = (located_tile.x * tile::PIXEL_WIDTH) as f64;
            let y = (located_tile.y * tile::PIXEL_HEIGHT) as f64;

            let tile_asset = match located_tile.value {
                Tile::GrassPlain => &model.assets.grass_tile,
            };

            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                tile_asset,
                x,
                y,
                tile::PIXEL_WIDTH_FL,
                tile::PIXEL_HEIGHT_FL,
            )
            .map_err(|_| "Could not draw tile image on canvas".to_string())?;
        }
    }

    Ok(())
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(model: &Model) -> Cell<Msg> {
    Cell::group(
        vec![],
        vec![
            map_canvas_cell(model),
            units_canvas_cell(model),
            click_screen(),
            overlay_view(model),
        ],
    )
}

fn units_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(vec![], vec![game_canvas(model, &model.units_canvas)])
}

fn map_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(vec![], vec![game_canvas(model, &model.map_canvas)])
}

fn game_canvas(model: &Model, r: &ElRef<HtmlCanvasElement>) -> Node<Msg> {
    canvas![
        C![
            Style::Absolute.css_classes().concat(),
            Style::W512px.css_classes().concat(),
            Style::H512px.css_classes().concat()
        ],
        attrs! {
            At::Width => px_u16(model.game_pixel_width).as_str(),
            At::Height => px_u16(model.game_pixel_height).as_str()
        },
        style! {
            St::Left => px_i16(model.game_x).as_str(),
            St::Top => px_i16(model.game_y).as_str()
        },
        el_ref(&r)
    ]
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
