use crate::domain::point::Point;
use crate::view::card::Card;
use crate::view::cell::Cell;
use crate::web_sys::HtmlCanvasElement;
use crate::{assets, global, Row, Style, Toast};
use seed::app::CmdHandle;
use seed::prelude::{cmds, el_ref, At, El, ElRef, IndexMap, Node, Orders, St, ToClasses, UpdateEl};
use seed::{attrs, canvas, style, C};
use shared::facing_direction::FacingDirection;
use shared::frame_count::FrameCount;
use shared::game::Game;
use shared::id::Id;
use shared::located::Located;
use shared::team_color::TeamColor;
use shared::tile;
use std::collections::HashSet;

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
    visibility_canvas: ElRef<HtmlCanvasElement>,
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
        visibility_canvas: ElRef::<HtmlCanvasElement>::default(),
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
            let viewer_id = global.viewer_id();

            let map_draw_result = draw_map(&model);

            if let Err(err) = map_draw_result {
                global.toast(
                    Toast::init("error", "map rendering problem")
                        .error()
                        .with_more_info(err.as_str()),
                );
            }

            if let Err((err_title, err_detail)) = draw(&viewer_id, &model) {
                global.toast(
                    Toast::init("error", err_title.as_str())
                        .error()
                        .with_more_info(err_detail.as_str()),
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

            let viewer_id = global.viewer_id();

            if let Err((err_title, err_detail)) = draw(&viewer_id, &model) {
                global.toast(
                    Toast::init("error", err_title.as_str())
                        .error()
                        .with_more_info(err_detail.as_str()),
                );
            }

            model.handle_minimum_framerate_timeout = wait_for_timeout(orders);
        }
    }
}

fn draw(viewer_id: &Id, model: &Model) -> Result<(), (String, String)> {
    let visibility = model
        .game
        .get_players_visibility(viewer_id)
        .map_err(|err_msg| ("visibility rendering problem".to_string(), err_msg))?;

    draw_units(visibility, &model)
        .map_err(|err_msg| ("units rendering problem".to_string(), err_msg))?;

    draw_visibility(visibility, &model)
        .map_err(|err_msg| ("visibility rendering problem".to_string(), err_msg))?;

    Ok(())
}

fn draw_visibility(visibility: &HashSet<Located<()>>, model: &Model) -> Result<(), String> {
    let canvas = model
        .visibility_canvas
        .get()
        .expect("could not get visibility canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = model.game_pixel_width as f64;
    let height = model.game_pixel_height as f64;

    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    for x in 0..model.game.map.width {
        for y in 0..model.game.map.height {
            let x_u16 = x as u16;
            let y_u16 = y as u16;

            let loc = Located {
                value: (),
                x: x_u16,
                y: y_u16,
            };

            if !visibility.contains(&loc) {
                let sheet = &model.assets.sheet;

                let sx = 0.0;
                let sy = 48.0;

                let x_fl = (x_u16 * tile::PIXEL_WIDTH) as f64;
                let y_fl = (y_u16 * tile::PIXEL_HEIGHT) as f64;

                ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    sheet,
                    sx,
                    sy,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                    x_fl,
                    y_fl,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                )
                .map_err(|_| "Could not draw unit image on canvas".to_string())?;
            }
        }
    }

    Ok(())
}

fn draw_units(visibility: &HashSet<Located<()>>, model: &Model) -> Result<(), String> {
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
        let location = Located {
            x: located_unit.x,
            y: located_unit.y,
            value: (),
        };

        if visibility.contains(&location) {
            let x = (located_unit.x * tile::PIXEL_WIDTH) as f64;
            let y = (located_unit.y * tile::PIXEL_HEIGHT) as f64;

            let unit_model = located_unit.clone().value;

            let sheet = match unit_model.facing {
                FacingDirection::Left => &model.assets.sheet_flipped,
                FacingDirection::Right => &model.assets.sheet,
            };

            let (sx, sy) = {
                let sx = match model.frame_count {
                    FrameCount::F1 => 0.0,
                    FrameCount::F2 => 1.0,
                    FrameCount::F3 => 2.0,
                    FrameCount::F4 => 3.0,
                };

                let sy = match unit_model.color {
                    TeamColor::Red => 0.0,
                    TeamColor::Blue => 1.0,
                };

                (sx * tile::PIXEL_WIDTH_FL, sy * tile::PIXEL_WIDTH_FL)
            };

            ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                sheet,
                sx,
                sy,
                tile::PIXEL_WIDTH_FL,
                tile::PIXEL_HEIGHT_FL,
                x,
                y,
                tile::PIXEL_WIDTH_FL,
                tile::PIXEL_HEIGHT_FL,
            )
            .map_err(|_| "Could not draw unit image on canvas".to_string())?;
        }
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

            ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &model.assets.sheet,
                0.0,
                32.0,
                tile::PIXEL_WIDTH_FL,
                tile::PIXEL_HEIGHT_FL,
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
            visibility_canvas_cell(model),
            click_screen(),
            overlay_view(model),
        ],
    )
}

fn visibility_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(vec![], vec![game_canvas(model, &model.visibility_canvas)])
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
    Card::cell_from_rows(
        vec![],
        vec![Row::from_cells(
            vec![],
            vec![Cell::from_str(vec![], "Waiting..")],
        )],
    )
}
