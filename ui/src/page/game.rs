use crate::domain::direction::Direction;
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
use shared::unit::{Unit, UnitId};
use std::collections::{HashMap, HashSet};

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
    moved_units: Vec<UnitId>,
    moves_index: HashMap<UnitId, Action>,
    mouse_game_position: Option<Point<u32>>,
    stage: Stage,
}

impl Model {
    fn get_units_move(&self, unit_id: &UnitId) -> Option<&Action> {
        self.moves_index.get(unit_id)
    }
}

enum Stage {
    TakingTurn { mode: Mode },
}

enum Mode {
    None,
    MovingUnit(MovingUnitModel),
}

struct MovingUnitModel {
    unit_id: UnitId,
    mobility: HashSet<Located<()>>,
    arrows: Vec<(Direction, Arrow)>,
}

#[derive(PartialEq, Debug, Clone)]
enum Action {
    TraveledTo {
        dest: Located<()>,
        arrows: Vec<(Direction, Arrow)>,
    },
}

#[derive(PartialEq, Debug, Clone)]
enum Arrow {
    EndLeft,
    EndDown,
    EndRight,
    EndUp,
    X,
    Y,
    RightUp,
    RightDown,
    LeftUp,
    LeftDown,
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
        moved_units: Vec::new(),
        moves_index: HashMap::new(),
        mouse_game_position: None,
        stage: Stage::TakingTurn { mode: Mode::None },
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

            if let Err(err) = draw_terrain(&model) {
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
        Msg::MouseDownOnScreen(_page_pos) => {}
        Msg::MouseUpOnScreen(page_pos) => {
            handle_click_on_screen_during_turn(
                global,
                model,
                click_pos_to_game_pos(page_pos, model),
            );
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
                let mouse_loc = Located {
                    value: (),
                    x: x as u16,
                    y: y as u16,
                };

                if let Err((err_title, err_msg)) = handle_mouse_move_for_mode(model, mouse_loc) {
                    global.toast(
                        Toast::init("error", err_title.as_str())
                            .error()
                            .with_more_info(err_msg.as_str()),
                    );
                }

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

fn handle_click_on_screen_during_turn(
    global: &mut global::Model,
    model: &mut Model,
    mouse_pos: Point<i16>,
) {
    let Point { x, y } = mouse_pos;

    if !(x > 0 && y > 0) {
        return;
    }

    let x = x as u16;
    let y = y as u16;

    match &mut model.stage {
        Stage::TakingTurn { mode } => match mode {
            Mode::None => handle_click_on_screen_when_no_mode(global, model, x, y),
            Mode::MovingUnit(_) => {
                if let Err((err_title, err_msg)) = handle_click_on_screen_when_move_mode(model) {
                    global.toast(
                        Toast::init("error", err_title.as_str())
                            .error()
                            .with_more_info(err_msg.as_str()),
                    );
                }
            }
        },
    }
}

fn handle_click_on_screen_when_move_mode(model: &mut Model) -> Result<(), (String, String)> {
    if let Stage::TakingTurn {
        mode: Mode::MovingUnit(moving_model),
    } = &model.stage
    {
        let unit_loc = model.game.units.get(&moving_model.unit_id).ok_or((
            "handle move click".to_string(),
            "Could not find unit when moving unit SirttBHL".to_string(),
        ))?;

        let pos = path_to_pos(
            &moving_model
                .arrows
                .iter()
                .map(|(dir, _)| dir.clone())
                .collect::<Vec<_>>(),
        );

        let dest = Located {
            x: unit_loc.x + (pos.x as u16),
            y: unit_loc.y + (pos.y as u16),
            value: (),
        };

        let unit_id = moving_model.unit_id.clone();

        model.moved_units.push(unit_id.clone());
        model.moves_index.insert(
            unit_id,
            Action::TraveledTo {
                dest,
                arrows: moving_model.arrows.clone(),
            },
        );

        model.stage = Stage::TakingTurn { mode: Mode::None };

        clear_mode_canvas(model)?;
    }

    Ok(())
}

fn handle_click_on_screen_when_no_mode(
    global: &mut global::Model,
    model: &mut Model,
    x: u16,
    y: u16,
) {
    if let Some(units_at_pos) = model.game.get_units_by_location(&Point { x, y }) {
        if let Some((first, rest)) = units_at_pos.split_first() {
            if rest.is_empty() {
                let (unit_id, _) = first;

                match model.game.get_units_mobility(unit_id) {
                    Ok(mobility) => {
                        model.stage = Stage::TakingTurn {
                            mode: Mode::MovingUnit(MovingUnitModel {
                                unit_id: unit_id.clone(),
                                mobility,
                                arrows: Vec::new(),
                            }),
                        };

                        if let Err((err_title, err_msg)) = draw_mode_from_mouse_event(model) {
                            global.toast(
                                Toast::init("error", err_title.as_str())
                                    .error()
                                    .with_more_info(err_msg.as_str()),
                            );
                        }
                    }
                    Err(err_msg) => {
                        global.toast(
                            Toast::init("error", "could not get mobility range of unit")
                                .error()
                                .with_more_info(err_msg.as_str()),
                        );
                    }
                }
            } else {
                todo!("Clicked on many units")
            }
        }
    };
}

fn handle_mouse_move_for_mode(
    model: &mut Model,
    mouse_loc: Located<()>,
) -> Result<(), (String, String)> {
    match &mut model.stage {
        Stage::TakingTurn { mode } => match mode {
            Mode::None => {}
            Mode::MovingUnit(moving_model) => {
                if moving_model.mobility.contains(&mouse_loc) {
                    let unit_loc = model.game.units.get(&moving_model.unit_id).ok_or((
                        "handle mouse move in move mode".to_string(),
                        "Could not find unit in moving model".to_string(),
                    ))?;

                    let mouse_point = Point {
                        x: mouse_loc.x as i32 - unit_loc.x as i32,
                        y: mouse_loc.y as i32 - unit_loc.y as i32,
                    };

                    let arrows = calc_arrows(
                        mouse_point,
                        Some(
                            &moving_model
                                .arrows
                                .iter()
                                .map(|(dir, _)| dir.clone())
                                .collect::<Vec<_>>(),
                        ),
                        unit_loc.value.unit.get_mobility_range(),
                    );

                    moving_model.arrows = arrows;
                } else {
                    moving_model.arrows = vec![];
                }

                draw_mode_from_mouse_event(model)?;
            }
        },
    }

    Ok(())
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

fn clear_mode_canvas(model: &Model) -> Result<(), (String, String)> {
    let canvas = model.mode_canvas.get().ok_or((
        "clear mode canvas".to_string(),
        "could not get mode canvas element to clear".to_string(),
    ))?;

    let ctx = seed::canvas_context_2d(&canvas);

    let width = model.game_pixel_width as f64;
    let height = model.game_pixel_height as f64;

    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    Ok(())
}

fn draw_mode_from_mouse_event(model: &Model) -> Result<(), (String, String)> {
    let canvas = model.mode_canvas.get().ok_or((
        "draw mode from mouse event".to_string(),
        "could not get mode canvas element".to_string(),
    ))?;

    let ctx = seed::canvas_context_2d(&canvas);

    let width = model.game_pixel_width as f64;
    let height = model.game_pixel_height as f64;

    ctx.begin_path();
    ctx.clear_rect(0., 0., width, height);

    match &model.stage {
        Stage::TakingTurn { mode } => match mode {
            Mode::None => {}
            Mode::MovingUnit(moving_model) => {
                for mobility_space in moving_model.mobility.iter() {
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

                let unit = model.game.units.get(&moving_model.unit_id).ok_or((
                    "rendering mobility range".to_string(),
                    "Could not get unit in moving mode".to_string(),
                ))?;

                let mut arrow_x = unit.x;
                let mut arrow_y = unit.y;
                for (dir, arrow) in &moving_model.arrows {
                    draw_arrows(&ctx, model, arrow, dir, &mut arrow_x, &mut arrow_y)
                        .map_err(|err_msg| ("rendering mobility range".to_string(), err_msg))?;
                }
            }
        },
    }

    Ok(())
}

fn draw_arrows(
    ctx: &web_sys::CanvasRenderingContext2d,
    model: &Model,
    arrow: &Arrow,
    dir: &Direction,
    arrow_x: &mut u16,
    arrow_y: &mut u16,
) -> Result<(), String> {
    dir.adjust_coord(arrow_x, arrow_y);

    let sheet_row = match arrow {
        Arrow::EndLeft => 96.0,
        Arrow::EndDown => 144.0,
        Arrow::EndRight => 64.0,
        Arrow::EndUp => 112.0,
        Arrow::X => 80.0,
        Arrow::Y => 128.0,
        Arrow::RightUp => 160.0,
        Arrow::RightDown => 176.0,
        Arrow::LeftUp => 192.0,
        Arrow::LeftDown => 208.0,
    };

    ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &model.assets.sheet,
        MISC_SPRITE_SHEET_COLUMN,
        sheet_row,
        tile::PIXEL_WIDTH_FL,
        tile::PIXEL_HEIGHT_FL,
        *arrow_x as f64 * tile::PIXEL_WIDTH_FL,
        *arrow_y as f64 * tile::PIXEL_HEIGHT_FL,
        tile::PIXEL_WIDTH_FL,
        tile::PIXEL_HEIGHT_FL,
    )
    .map_err(|_| "could not draw arrow image on canvas".to_string())?;

    Ok(())
}

fn draw_cursor(model: &Model) -> Result<(), String> {
    let canvas = model
        .cursor_canvas
        .get()
        .ok_or("could not get cursor canvas element".to_string())?;
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
        .ok_or("could not get visibility canvas element".to_string())?;
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
        .ok_or("could not get units canvas element")?;
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

                let mut sy = match unit_model.unit {
                    Unit::Infantry => 0.0,
                    Unit::Tank => 2.0,
                };

                if unit_model.color == TeamColor::Blue {
                    sy += 1.0;
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

                match model.get_units_move(unit_id) {
                    None => {}
                    Some(action) => match action {
                        Action::TraveledTo { arrows, .. } => {
                            let mut arrow_x = located_unit.x;
                            let mut arrow_y = located_unit.y;
                            for (dir, arrow) in arrows {
                                draw_arrows(&ctx, model, arrow, dir, &mut arrow_x, &mut arrow_y)?;
                            }
                        }
                    },
                }
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
        .ok_or("could not get map canvas element")?;
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

fn calc_arrows(
    mouse_pos: Point<i32>,
    maybe_existing_path: Option<&Vec<Direction>>,
    range_limit: usize,
) -> Vec<(Direction, Arrow)> {
    let directions = calc_movement_path(mouse_pos, maybe_existing_path, range_limit);

    path_with_arrows(&directions)
}

fn calc_movement_path(
    mouse_pos: Point<i32>,
    maybe_existing_path: Option<&Vec<Direction>>,
    range_limit: usize,
) -> Vec<Direction> {
    match maybe_existing_path {
        None => {
            if mouse_pos.x == 0 && mouse_pos.y == 0 {
                vec![]
            } else {
                let (direction, next_loc) = if mouse_pos.y.abs() < mouse_pos.x.abs() {
                    let is_east = mouse_pos.x > 0;

                    let next_loc = Point {
                        x: if is_east {
                            mouse_pos.x - 1
                        } else {
                            mouse_pos.x + 1
                        },
                        y: mouse_pos.y,
                    };

                    let direction = if is_east {
                        Direction::East
                    } else {
                        Direction::West
                    };

                    (direction, next_loc)
                } else {
                    let is_south = mouse_pos.y > 0;

                    let next_loc = Point {
                        x: mouse_pos.x,
                        y: if is_south {
                            mouse_pos.y - 1
                        } else {
                            mouse_pos.y + 1
                        },
                    };

                    let direction = if is_south {
                        Direction::South
                    } else {
                        Direction::North
                    };

                    (direction, next_loc)
                };

                let mut base_ret = vec![direction];

                base_ret.append(&mut calc_movement_path(next_loc, None, range_limit));

                base_ret
            }
        }
        Some(existing_path) => {
            if existing_path.len() > range_limit {
                calc_movement_path(mouse_pos, None, range_limit)
            } else {
                let new_origin = path_to_pos(&existing_path);
                let mut ret = existing_path.clone();

                let new_mouse_pos = Point {
                    x: mouse_pos.x - new_origin.x,
                    y: mouse_pos.y - new_origin.y,
                };

                ret.append(&mut calc_movement_path(new_mouse_pos, None, range_limit));

                let positions = path_to_positions(&ret);
                let positions_set: HashSet<Point<i32>> = {
                    let mut set = HashSet::new();

                    for p in &positions {
                        set.insert(p.clone());
                    }

                    set
                };

                // If it the arrow visits the same position twice, then scrap it
                if positions.len() == positions_set.len() {
                    ret.clone()
                } else {
                    calc_movement_path(mouse_pos, None, range_limit)
                }
            }
        }
    }
}

fn path_to_pos(path: &Vec<Direction>) -> Point<i32> {
    let mut x = 0;
    let mut y = 0;

    for step in path {
        match step {
            Direction::North => {
                y -= 1;
            }
            Direction::South => {
                y += 1;
            }
            Direction::East => {
                x += 1;
            }
            Direction::West => {
                x -= 1;
            }
        }
    }

    Point { x, y }
}

fn path_to_positions(path: &Vec<Direction>) -> Vec<Point<i32>> {
    let mut ret = vec![];

    let mut x = 0;
    let mut y = 0;

    for step in path {
        match step {
            Direction::North => {
                y -= 1;
            }
            Direction::South => {
                y += 1;
            }
            Direction::East => {
                x += 1;
            }
            Direction::West => {
                x -= 1;
            }
        }

        ret.push(Point {
            x: x.clone(),
            y: y.clone(),
        });
    }

    ret
}

fn path_with_arrows(path: &Vec<Direction>) -> Vec<(Direction, Arrow)> {
    let mut filtered_path = path.into_iter().collect::<Vec<_>>();

    let mut index = 0;
    while index < filtered_path.len() {
        let dir = path[index.clone()].clone();
        if let Some(next) = path.get(index.clone() + 1) {
            if dir == next.opposite() {
                if (index + 1) < filtered_path.len() {
                    filtered_path.remove(index.clone() + 1);
                }

                filtered_path.remove(index.clone());

                index = 0;
            }
        }
        index += 1;
    }

    let mut filtered_path_peek = filtered_path.into_iter().peekable();

    let mut arrows = vec![];

    while let Some(dir) = filtered_path_peek.next() {
        let maybe_next = filtered_path_peek.peek();

        let arrow = match maybe_next {
            None => match dir {
                Direction::North => Arrow::EndUp,
                Direction::South => Arrow::EndDown,
                Direction::East => Arrow::EndRight,
                Direction::West => Arrow::EndLeft,
            },
            Some(next) => match (dir, next) {
                (Direction::North, Direction::North) => Arrow::Y,
                (Direction::North, Direction::East) => Arrow::LeftDown,
                (Direction::North, Direction::South) => {
                    panic!("Cannot move up and then down")
                }
                (Direction::North, Direction::West) => Arrow::RightDown,
                (Direction::East, Direction::North) => Arrow::RightUp,
                (Direction::East, Direction::East) => Arrow::X,
                (Direction::East, Direction::South) => Arrow::RightDown,
                (Direction::East, Direction::West) => {
                    panic!("Cannot move right then left")
                }
                (Direction::South, Direction::North) => {
                    panic!("Cannot move down then up")
                }
                (Direction::South, Direction::East) => Arrow::LeftUp,
                (Direction::South, Direction::South) => Arrow::Y,
                (Direction::South, Direction::West) => Arrow::RightUp,
                (Direction::West, Direction::North) => Arrow::LeftUp,
                (Direction::West, Direction::East) => {
                    panic!("Cannot move left then right")
                }
                (Direction::West, Direction::South) => Arrow::LeftDown,
                (Direction::West, Direction::West) => Arrow::X,
            },
        };

        arrows.push((dir.clone(), arrow));
    }

    arrows
}

#[cfg(test)]
mod test_movement_arrow {
    use crate::domain::direction::Direction;
    use crate::game::{calc_movement_path, path_with_arrows, Arrow};
    use pretty_assertions::assert_eq;
    use shared::point::Point;

    fn path_to_arrows(path: &Vec<Direction>) -> Vec<Arrow> {
        path_with_arrows(path)
            .into_iter()
            .map(|(_, arrow)| arrow)
            .collect::<Vec<_>>()
    }

    #[test]
    fn no_path_for_origin() {
        let want: Vec<Direction> = vec![];
        assert_eq!(want, calc_movement_path(Point { x: 0, y: 0 }, None, 16));
    }

    #[test]
    fn east_path_for_mouse_east() {
        let want: Vec<Direction> = vec![Direction::East];
        assert_eq!(want, calc_movement_path(Point { x: 1, y: 0 }, None, 16));
    }

    #[test]
    fn many_east_path_for_mouse_very_east() {
        let want: Vec<Direction> = vec![
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
        ];
        assert_eq!(want, calc_movement_path(Point { x: 8, y: 0 }, None, 16));
    }

    #[test]
    fn many_west_path_for_mouse_very_west() {
        let want: Vec<Direction> = vec![
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::West,
        ];
        assert_eq!(want, calc_movement_path(Point { x: -8, y: 0 }, None, 16));
    }

    #[test]
    fn many_north_path_for_mouse_very_north() {
        let want: Vec<Direction> = vec![
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
            Direction::North,
        ];
        assert_eq!(want, calc_movement_path(Point { x: 0, y: -8 }, None, 16));
    }

    #[test]
    fn many_south_path_for_mouse_very_south() {
        let want: Vec<Direction> = vec![
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::South,
        ];
        assert_eq!(want, calc_movement_path(Point { x: 0, y: 8 }, None, 16));
    }

    #[test]
    fn path_can_go_diagonal() {
        let want: Vec<Direction> = vec![Direction::South, Direction::East];

        assert_eq!(want, calc_movement_path(Point { x: 1, y: 1 }, None, 16));
    }

    #[test]
    fn path_can_go_diagonal_far() {
        let want: Vec<Direction> = vec![
            Direction::North,
            Direction::East,
            Direction::North,
            Direction::East,
            Direction::North,
            Direction::East,
            Direction::North,
            Direction::East,
        ];

        assert_eq!(want, calc_movement_path(Point { x: 4, y: -4 }, None, 16));
    }

    #[test]
    fn path_can_go_diagonal_irregular() {
        let want: Vec<Direction> = vec![
            Direction::West,
            Direction::West,
            Direction::North,
            Direction::West,
            Direction::North,
            Direction::West,
        ];

        assert_eq!(want, calc_movement_path(Point { x: -4, y: -2 }, None, 16));
    }

    #[test]
    fn path_can_work_off_existing_path() {
        let want: Vec<Direction> = vec![
            Direction::South,
            Direction::South,
            Direction::South,
            Direction::West,
            Direction::West,
            Direction::West,
            Direction::South,
            Direction::West,
        ];

        assert_eq!(
            want,
            calc_movement_path(
                Point { x: -4, y: 4 },
                Some(&vec![Direction::South, Direction::South, Direction::South]),
                16
            )
        );
    }

    #[test]
    fn east_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::X, Arrow::X, Arrow::EndRight];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::East, Direction::East, Direction::East])
        );
    }

    #[test]
    fn south_west_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::RightUp, Arrow::LeftDown, Arrow::EndDown];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::South, Direction::West, Direction::South])
        );
    }

    #[test]
    fn north_west_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::RightDown, Arrow::LeftUp, Arrow::EndUp];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::North, Direction::West, Direction::North])
        );
    }

    #[test]
    fn north_east_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::LeftDown, Arrow::RightUp, Arrow::EndUp];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::North, Direction::East, Direction::North])
        );
    }

    #[test]
    fn south_east_path_to_arrow() {
        let want: Vec<Arrow> = vec![Arrow::LeftUp, Arrow::RightDown, Arrow::EndDown];

        assert_eq!(
            want,
            path_to_arrows(&vec![Direction::South, Direction::East, Direction::South])
        );
    }

    #[test]
    fn west_south_path_to_arrow() {
        let want: Vec<Arrow> = vec![
            Arrow::LeftDown,
            Arrow::RightUp,
            Arrow::LeftDown,
            Arrow::RightUp,
            Arrow::EndLeft,
        ];

        assert_eq!(
            want,
            path_to_arrows(&vec![
                Direction::West,
                Direction::South,
                Direction::West,
                Direction::South,
                Direction::West
            ])
        );
    }

    #[test]
    fn path_to_arrows_filters_double_backs() {
        let want: Vec<Arrow> = vec![Arrow::X, Arrow::EndRight];

        assert_eq!(
            want,
            path_to_arrows(&vec![
                Direction::East,
                Direction::East,
                Direction::East,
                Direction::West
            ])
        )
    }

    #[test]
    fn path_to_arrows_filters_returns() {
        let want: Vec<Direction> = vec![Direction::East, Direction::East];

        assert_eq!(
            want,
            calc_movement_path(
                Point { x: 2, y: 0 },
                Some(&vec![
                    Direction::East,
                    Direction::East,
                    Direction::South,
                    Direction::West,
                    Direction::North
                ]),
                16
            )
        );
    }

    #[test]
    fn edge_of_range_can_be_approached_from_north() {
        let want: Vec<Direction> = vec![Direction::East, Direction::South];

        assert_eq!(
            want,
            calc_movement_path(Point { x: 1, y: 1 }, Some(&vec![Direction::East]), 2)
        );
    }

    #[test]
    fn edge_of_range_can_be_approached_from_west() {
        let want: Vec<Direction> = vec![Direction::South, Direction::East];

        assert_eq!(
            want,
            calc_movement_path(Point { x: 1, y: 1 }, Some(&vec![Direction::South]), 2)
        );
    }
}
