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
use shared::point::Point;
use shared::team_color::TeamColor;
use shared::tile;
use shared::unit::UnitId;
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
    mode_canvas: ElRef<HtmlCanvasElement>,
    cursor_canvas: ElRef<HtmlCanvasElement>,
    assets: assets::Model,
    game_pixel_width: u16,
    game_pixel_height: u16,
    game_pos: Point<i16>,
    handle_minimum_framerate_timeout: CmdHandle,
    frame_count: FrameCount,
    moved_units: HashSet<UnitId>,
    mouse_game_position: Option<Point<u32>>,
    mode: Mode,
}

enum Mode {
    None,
    MovingUnit {
        unit_id: UnitId,
        mobility: HashSet<Located<()>>,
    },
}

#[derive(Clone, Debug)]
pub enum Msg {
    RenderedFirstTime,
    MouseDownOnScreen(Point<i16>),
    MouseUpOnScreen(Point<i16>),
    MouseMoveOnScreen(Point<i16>),
    MinimumRenderTimeExpired,
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
        mode_canvas: ElRef::<HtmlCanvasElement>::default(),
        cursor_canvas: ElRef::<HtmlCanvasElement>::default(),
        assets,
        game_pixel_width,
        game_pixel_height,
        game_pos: Point {
            x: game_x as i16,
            y: game_y as i16,
        },
        handle_minimum_framerate_timeout: wait_for_timeout(orders),
        frame_count: FrameCount::F1,
        moved_units: HashSet::new(),
        mouse_game_position: None,
        mode: Mode::None,
    };

    Ok(model)
}

fn click_pos_to_game_pos(page_pos: Point<i16>, model: &Model) -> Point<i16> {
    let x_on_game = page_pos.x - model.game_pos.x;
    let x = x_on_game / ((tile::PIXEL_WIDTH * 2) as i16);

    let y_on_game = page_pos.y - model.game_pos.y;
    let y = y_on_game / ((tile::PIXEL_HEIGHT * 2) as i16);

    Point { x, y }
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

            let map_draw_result = draw_terrain(&model);

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
        Msg::MouseDownOnScreen(page_pos) => {}
        Msg::MouseUpOnScreen(page_pos) => {
            let Point { x, y } = click_pos_to_game_pos(page_pos, model);
            if x > 0 && y > 0 {
                let x = x as u16;
                let y = y as u16;
                if let Some(units_at_pos) = model.game.get_units_by_location(&Point { x, y }) {
                    if let Some((first, rest)) = units_at_pos.split_first() {
                        if rest.is_empty() {
                            let (unit_id, _) = first;

                            match model.game.get_units_mobility(unit_id) {
                                Ok(mobility) => {
                                    model.mode = Mode::MovingUnit {
                                        unit_id: unit_id.clone(),
                                        mobility,
                                    };

                                    if let Err((err_title, err_msg)) = draw_mode(model) {
                                        global.toast(
                                            Toast::init("error", err_title.as_str())
                                                .error()
                                                .with_more_info(err_msg.as_str()),
                                        );
                                    }
                                }
                                Err(err_msg) => {
                                    global.toast(
                                        Toast::init(
                                            "error",
                                            "could not get mobility range of unit",
                                        )
                                        .error()
                                        .with_more_info(err_msg.as_str()),
                                    );
                                }
                            }
                        }
                    }
                };
            }
        }
        Msg::MouseMoveOnScreen(page_pos) => {
            let Point { x, y } = click_pos_to_game_pos(page_pos, model);

            model.mouse_game_position = if x < 0
                || y < 0
                || (model.game.map.width as i16) < x
                || (model.game.map.height as i16) < y
            {
                None
            } else {
                Some(Point {
                    x: x as u32,
                    y: y as u32,
                })
            };

            let draw_result = draw_cursor(&model);

            if let Err(err_msg) = draw_result {
                global.toast(
                    Toast::init("error", "could not render cursor")
                        .error()
                        .with_more_info(err_msg.as_str()),
                );
            }
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

    draw_mode(&model)?;

    draw_visibility(visibility, &model)
        .map_err(|err_msg| ("visibility rendering problem".to_string(), err_msg))?;

    Ok(())
}

fn draw_mode(model: &Model) -> Result<(), (String, String)> {
    let canvas = model
        .mode_canvas
        .get()
        .expect("could not get mode canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = model.game_pixel_width as f64;
    let height = model.game_pixel_height as f64;

    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    match &model.mode {
        Mode::None => {}
        Mode::MovingUnit { mobility, .. } => {
            for mobility_space in mobility.into_iter() {
                ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &model.assets.sheet,
                    MISC_SPRITE_SHEET_COLUMN,
                    48.0,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                    mobility_space.x as f64 * tile::PIXEL_WIDTH_FL,
                    mobility_space.y as f64 * tile::PIXEL_HEIGHT_FL,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                )
                .map_err(|_| {
                    (
                        "rendering mobility range".to_string(),
                        "could not draw mobility image on canvas".to_string(),
                    )
                })?;
            }
        }
    }
    Ok(())
}

fn draw_cursor(model: &Model) -> Result<(), String> {
    let canvas = model
        .cursor_canvas
        .get()
        .expect("could not get cursor canvas element");
    let ctx = seed::canvas_context_2d(&canvas);

    let width = model.game_pixel_width as f64;
    let height = model.game_pixel_height as f64;

    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    if let Some(mouse_game_pos) = &model.mouse_game_position {
        ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &model.assets.sheet,
            MISC_SPRITE_SHEET_COLUMN,
            32.0,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
            mouse_game_pos.x as f64 * tile::PIXEL_WIDTH_FL,
            mouse_game_pos.y as f64 * tile::PIXEL_HEIGHT_FL,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
        )
        .map_err(|_| "Could not draw cursor image on canvas".to_string())?;
    }

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

                let sx = MISC_SPRITE_SHEET_COLUMN;
                let sy = 16.0;

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

    for (unit_id, located_unit) in model.game.units.iter() {
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
                let sx = {
                    let sprite_sheet_x = match model.frame_count {
                        FrameCount::F1 => 0.0,
                        FrameCount::F2 => 1.0,
                        FrameCount::F3 => 2.0,
                        FrameCount::F4 => 3.0,
                    };
                    match unit_model.facing {
                        FacingDirection::Left => SPRITE_SHEET_WIDTH - sprite_sheet_x - 1.0,
                        FacingDirection::Right => sprite_sheet_x,
                    }
                };

                let sy = match unit_model.color {
                    TeamColor::Red => 0.0,
                    TeamColor::Blue => 1.0,
                };

                (sx * tile::PIXEL_WIDTH_FL, sy * tile::PIXEL_WIDTH_FL)
            };

            if model.moved_units.contains(unit_id) {
                ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &model.assets.sheet,
                    sx + 64.0,
                    sy,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                    x,
                    y,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                )
                .map_err(|_| "Could not draw unit outline image on canvas".to_string())?;
            } else {
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
    }

    Ok(())
}

fn draw_terrain(model: &Model) -> Result<(), String> {
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
                MISC_SPRITE_SHEET_COLUMN,
                0.0,
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

pub fn view(global: &global::Model, model: &Model) -> Cell<Msg> {
    Cell::group(
        vec![],
        vec![
            map_canvas_cell(model),
            units_canvas_cell(model),
            visibility_canvas_cell(model),
            mode_canvas_cell(model),
            cursor_canvas_cell(model),
            click_screen(model),
            overlay_view(global, model),
        ],
    )
}

fn mode_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(vec![], vec![game_canvas(model, &model.mode_canvas)])
}

fn cursor_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(vec![], vec![game_canvas(model, &model.cursor_canvas)])
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
            St::Left => px_i16(model.game_pos.x).as_str(),
            St::Top => px_i16(model.game_pos.y).as_str()
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

fn click_screen(model: &Model) -> Cell<Msg> {
    let cursor_style = if model.mouse_game_position.is_none() {
        Style::none()
    } else {
        Style::CursorNone
    };

    Cell::group(
        vec![
            Style::Absolute,
            Style::Left0,
            Style::Right0,
            Style::Top0,
            Style::Bottom0,
            cursor_style,
        ],
        vec![],
    )
    .on_mouse_down(|event| {
        Msg::MouseDownOnScreen(Point {
            x: event.page_x() as i16,
            y: event.page_y() as i16,
        })
    })
    .on_mouse_up(|event| {
        Msg::MouseUpOnScreen(Point {
            x: event.page_x() as i16,
            y: event.page_y() as i16,
        })
    })
    .on_mouse_move(|event| {
        Msg::MouseMoveOnScreen(Point {
            x: event.page_x() as i16,
            y: event.page_y() as i16,
        })
    })
}

fn overlay_view(global: &global::Model, model: &Model) -> Cell<Msg> {
    if model.game.waiting_on_player(&global.viewer_id()) {
        Cell::none()
    } else {
        Cell::group(
            vec![Style::Absolute, Style::Bottom4, Style::Left50Pct],
            vec![Card::cell_from_rows(
                vec![],
                vec![Row::from_cells(
                    vec![],
                    vec![Cell::from_str(vec![], "waiting for other players..")],
                )],
            )],
        )
    }
}

const MISC_SPRITE_SHEET_COLUMN: f64 = 128.0;
const SPRITE_SHEET_WIDTH: f64 = 9.0;
