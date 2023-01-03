use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::Cell;
use crate::web_sys::HtmlCanvasElement;
use crate::{api, assets, core_ext, global, Row, Style, Toast};
use seed::app::CmdHandle;
use seed::prelude::{cmds, el_ref, At, El, ElRef, IndexMap, Node, Orders, St, ToClasses, UpdateEl};
use seed::{attrs, canvas, style, C};
use shared::api::endpoint::Endpoint;
use shared::api::game::submit_turn;
use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::facing_direction::FacingDirection;
use shared::frame_count::FrameCount;
use shared::game::{Game, GameId, Turn};
use shared::id::Id;
use shared::located::Located;
use shared::point::Point;
use shared::team_color::TeamColor;
use shared::unit::{Unit, UnitId};
use shared::{game, tile};
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
    game_id: GameId,
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
    dialog: Option<Dialog>,
    status: Status,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Status {
    Ready,
    Waiting,
}

enum Dialog {
    ConfirmTurnSubmit,
}

impl Model {
    fn get_units_move(&self, unit_id: &UnitId) -> Option<&Action> {
        self.moves_index.get(unit_id)
    }

    fn clear_mode(&mut self) -> Result<(), String> {
        self.stage = Stage::TakingTurn { mode: Mode::None };

        clear_mode_canvas(self)
    }

    fn all_units_moved(&self, player_id: Id) -> bool {
        let total_moved_units = self.moved_units.len();

        let total_units = self
            .game
            .units_by_player_index
            .get(&player_id)
            .unwrap_or(&vec![])
            .len();

        total_moved_units == total_units
    }

    fn is_ready(&self) -> bool {
        self.status == Status::Ready
    }

    fn is_waiting_stage(&self) -> bool {
        matches!(self.stage, Stage::Waiting)
    }
}

#[derive(Debug)]
enum Stage {
    TakingTurn { mode: Mode },
    Waiting,
}

#[derive(Debug)]
enum Mode {
    None,
    MovingUnit(MovingUnitModel),
}

#[derive(Debug)]
struct MovingUnitModel {
    unit_id: UnitId,
    mobility: HashSet<Located<()>>,
    arrows: Vec<(Direction, Arrow)>,
}

#[derive(PartialEq, Debug, Clone)]
enum Action {
    TraveledTo {
        path: Vec<Located<Direction>>,
        arrows: Vec<(Direction, Arrow)>,
    },
}

#[derive(Clone, Debug)]
pub enum Msg {
    RenderedFirstTime,
    MouseDownOnScreen(Point<i16>),
    MouseUpOnScreen(Point<i16>),
    MouseMoveOnScreen(Point<i16>),
    MinimumRenderTimeExpired,
    ClickedSubmitTurn,
    ClickedSubmitTurnConfirm,
    ClickedCancelSubmitTurn,
    GotTurnSubmitResponse(Box<Result<submit_turn::Response, String>>),
    GotGame(Box<Result<shared::api::game::get::Response, String>>),
}

///////////////////////////////////////////////////////////////
// init
///////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Flags {
    pub game: Game,
    pub game_id: GameId,
}

pub fn init(
    global: &mut global::Model,
    flags: Flags,
    orders: &mut impl Orders<Msg>,
) -> Result<Model, String> {
    let window_size = global.window_size();

    let game_pixel_width = (flags.game.map.width as u16) * tile::PIXEL_WIDTH;
    let game_pixel_height = (flags.game.map.height as u16) * tile::PIXEL_HEIGHT;

    let game_x = (window_size.width / 2.0) - (game_pixel_width as f64);

    let game_y = (window_size.height / 2.0) - (game_pixel_height as f64);

    orders.after_next_render(|_| Msg::RenderedFirstTime);

    let assets = assets::init()?;

    let game = flags.game;

    let stage = if game.waiting_on_player(&global.viewer_id()) {
        Stage::TakingTurn { mode: Mode::None }
    } else {
        Stage::Waiting
    };

    let moves_index = match game.get_turn(global.viewer_id()) {
        Ok(turn) => match turn {
            Turn::Waiting => HashMap::new(),
            Turn::Turn { moves } => {
                let mut moves_ret = HashMap::new();

                for m in moves {
                    match m {
                        game::Action::Traveled {
                            unit_id,
                            path,
                            arrows,
                        } => {
                            moves_ret.insert(
                                unit_id.clone(),
                                Action::TraveledTo {
                                    path: path.clone(),
                                    arrows: arrows.clone(),
                                },
                            );
                        }
                    }
                }

                moves_ret
            }
        },
        Err(error) => {
            global.toast(
                Toast::init("error", "could not get turn")
                    .error()
                    .with_more_info(error.as_str()),
            );

            HashMap::new()
        }
    };

    let moved_units = moves_index
        .keys()
        .into_iter()
        .cloned()
        .collect::<Vec<UnitId>>();

    let model = Model {
        game,
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
        moved_units,
        moves_index,
        mouse_game_position: None,
        stage,
        dialog: None,
        status: Status::Ready,
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

            if let Err(err) = draw_terrain(model) {
                global.toast(
                    Toast::init("error", "map rendering problem")
                        .error()
                        .with_more_info(err.as_str()),
                );
            }

            if let Err((err_title, err_detail)) = draw(&viewer_id, model) {
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

            let draw_result = draw_cursor(model);

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

            if let Err((err_title, err_detail)) = draw(&viewer_id, model) {
                global.toast(
                    Toast::init("error", err_title.as_str())
                        .error()
                        .with_more_info(err_detail.as_str()),
                );
            }

            model.handle_minimum_framerate_timeout = wait_for_timeout(orders);
        }
        Msg::ClickedSubmitTurn => {
            if model.all_units_moved(global.viewer_id()) {
                submit_turn(global, model, orders);
            } else {
                model.dialog = Some(Dialog::ConfirmTurnSubmit)
            }
        }
        Msg::ClickedSubmitTurnConfirm => {
            submit_turn(global, model, orders);
        }
        Msg::ClickedCancelSubmitTurn => {
            model.dialog = None;
        }
        Msg::GotTurnSubmitResponse(result) => match *result {
            Ok(res) => {
                model.stage = Stage::Waiting;
                model.status = Status::Ready;

                refetch_game(model, res.game, orders);
            }
            Err(err) => {
                global.toast(
                    Toast::init("error", "failed to submit turn")
                        .error()
                        .with_more_info(err),
                );
            }
        },
        Msg::GotGame(result) => match *result {
            Ok(res) => {
                refetch_game(model, res.get_game(), orders);
            }
            Err(err) => {
                global.toast(
                    Toast::init("error", "failed to fetch game")
                        .error()
                        .with_more_info(err),
                );
            }
        },
    }
}

fn refetch_game(model: &mut Model, fetched_game: Game, orders: &mut impl Orders<Msg>) {
    if model.game.turn_number == fetched_game.turn_number {
        let url = Endpoint::make_get_game(model.game_id.clone());

        orders.skip().perform_cmd({
            async {
                let result = match api::get(url).await {
                    Ok(res_bytes) => shared::api::game::get::Response::from_bytes(res_bytes)
                        .map_err(|err| err.to_string()),
                    Err(error) => {
                        let fetch_error = core_ext::http::fetch_error_to_string(error);
                        Err(fetch_error)
                    }
                };

                Msg::GotGame(Box::new(result))
            }
        });

        model.status = Status::Waiting;
    } else {
        model.stage = Stage::TakingTurn { mode: Mode::None };
    }

    model.game = fetched_game;
}

fn submit_turn(global: &mut global::Model, model: &mut Model, orders: &mut impl Orders<Msg>) {
    model.dialog = None;

    let req_moves: Vec<game::Action> = model
        .moves_index
        .iter()
        .map(|(unit_id, action)| match action {
            Action::TraveledTo { path, arrows } => game::Action::Traveled {
                unit_id: unit_id.clone(),
                path: path.clone(),
                arrows: arrows.clone(),
            },
        })
        .collect();

    let req: submit_turn::Request = submit_turn::Request::init(req_moves);

    let url = Endpoint::submit_turn(model.game_id.clone(), global.viewer_id());

    match req.to_bytes() {
        Ok(bytes) => {
            model.status = Status::Waiting;

            orders.skip().perform_cmd({
                async {
                    let result = match api::post(url, bytes).await {
                        Ok(res_bytes) => submit_turn::Response::from_bytes(res_bytes)
                            .map_err(|err| err.to_string()),
                        Err(error) => {
                            let fetch_error = core_ext::http::fetch_error_to_string(error);
                            Err(fetch_error)
                        }
                    };

                    Msg::GotTurnSubmitResponse(Box::new(result))
                }
            })
        }
        Err(_err) => {
            todo!("Handle this error")
        }
    };
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
            Mode::MovingUnit(moving_model) => {
                let mouse_loc = Located {
                    x: x as u16,
                    y: y as u16,
                    value: (),
                };

                if moving_model.mobility.contains(&mouse_loc) {
                    if let Err((err_title, err_msg)) = handle_click_on_screen_when_move_mode(model)
                    {
                        global.toast(
                            Toast::init("error", err_title.as_str())
                                .error()
                                .with_more_info(err_msg.as_str()),
                        );
                    }
                } else if let Err(err_msg) = model.clear_mode() {
                    global.toast(
                        Toast::init("error", "clear mode canvas")
                            .error()
                            .with_more_info(err_msg.as_str()),
                    );
                }
            }
        },
        Stage::Waiting => {}
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

        let mut path: Vec<Located<Direction>> = Vec::new();

        let mut pos_x: u16 = unit_loc.x;
        let mut pos_y: u16 = unit_loc.y;

        for (dir, _) in &moving_model.arrows {
            path.push(Located {
                x: pos_x,
                y: pos_y,
                value: dir.clone(),
            });

            dir.adjust_coord(&mut pos_x, &mut pos_y);
        }

        let unit_id = moving_model.unit_id.clone();

        model.moved_units.push(unit_id.clone());
        model.moves_index.insert(
            unit_id,
            Action::TraveledTo {
                path,
                arrows: moving_model.arrows.clone(),
            },
        );

        model
            .clear_mode()
            .map_err(|msg| ("clear_mode_canvas".to_string(), msg))?;
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
        Stage::Waiting => {}
    }

    Ok(())
}

fn draw(viewer_id: &Id, model: &Model) -> Result<(), (String, String)> {
    let visibility = model
        .game
        .get_players_visibility(viewer_id)
        .map_err(|err_msg| ("visibility rendering problem".to_string(), err_msg))?;

    draw_units(visibility, model)
        .map_err(|err_msg| ("units rendering problem".to_string(), err_msg))?;

    draw_visibility(visibility, model)
        .map_err(|err_msg| ("visibility rendering problem".to_string(), err_msg))?;

    Ok(())
}

fn clear_mode_canvas(model: &Model) -> Result<(), String> {
    let canvas = model
        .mode_canvas
        .get()
        .ok_or_else(|| "could not get mode canvas element to clear".to_string())?;

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
                    draw_arrows(&ctx, model, arrow, dir, &mut arrow_x, &mut arrow_y, false)
                        .map_err(|err_msg| ("rendering mobility range".to_string(), err_msg))?;
                }
            }
        },
        Stage::Waiting => {}
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
    moved: bool,
) -> Result<(), String> {
    dir.adjust_coord(arrow_x, arrow_y);

    let mut sheet_row = match arrow {
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

    if moved {
        sheet_row += 160.0;
    }

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
        .ok_or_else(|| "could not get cursor canvas element".to_string())?;
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
        .ok_or_else(|| "could not get visibility canvas element".to_string())?;
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

            let has_moved = model.moved_units.contains(unit_id);

            let (sx, sy) = {
                let mut sx = {
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

                if has_moved {
                    match unit_model.facing {
                        FacingDirection::Left => {
                            sx -= 4.0;
                        }
                        FacingDirection::Right => {
                            sx += 4.0;
                        }
                    }
                }

                let mut sy = match unit_model.unit {
                    Unit::Infantry => 0.0,
                    Unit::Tank => 2.0,
                };

                if unit_model.color == TeamColor::Blue {
                    sy += 1.0;
                };

                (sx * tile::PIXEL_WIDTH_FL, sy * tile::PIXEL_WIDTH_FL)
            };

            if has_moved {
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
                .map_err(|_| "Could not draw unit outline image on canvas".to_string())?;

                match model.get_units_move(unit_id) {
                    None => {}
                    Some(action) => match action {
                        Action::TraveledTo { arrows, .. } => {
                            let mut arrow_x = located_unit.x;
                            let mut arrow_y = located_unit.y;
                            for (dir, arrow) in arrows {
                                draw_arrows(
                                    &ctx,
                                    model,
                                    arrow,
                                    dir,
                                    &mut arrow_x,
                                    &mut arrow_y,
                                    true,
                                )?;
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
            overlay_view(model),
            primary_options_view(global, model),
            dialog_view(model),
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
        el_ref(r)
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

fn dialog_view(model: &Model) -> Cell<Msg> {
    match &model.dialog {
        None => Cell::none(),
        Some(dialog) => match dialog {
            Dialog::ConfirmTurnSubmit => {
                let message = "You have not moved all your units, are you sure you would like to submit your turn?";

                let card_content =
                    Row::from_cells(vec![Style::W9], vec![Cell::from_str(vec![], message)]);

                Cell::group(
                    vec![
                        Style::Absolute,
                        Style::Left0,
                        Style::Top0,
                        Style::Right0,
                        Style::Bottom0,
                        Style::BgBackDrop,
                    ],
                    vec![Cell::group(
                        vec![Style::AbsoluteCenter],
                        vec![Card::cell_from_rows(
                            vec![Style::G4, Style::FlexCol],
                            vec![
                                card_content,
                                Row::from_cells(
                                    vec![Style::G4, Style::FlexRow],
                                    vec![
                                        Button::primary("submit")
                                            .on_click(|_| Msg::ClickedSubmitTurnConfirm)
                                            .cell(),
                                        Button::simple("cancel")
                                            .on_click(|_| Msg::ClickedCancelSubmitTurn)
                                            .cell(),
                                    ],
                                ),
                            ],
                        )],
                    )],
                )
            }
        },
    }
}

fn primary_options_view(global: &global::Model, model: &Model) -> Cell<Msg> {
    if model.dialog.is_none() {
        let submit_move_button = {
            let label = "submit turn";

            Button::simple(label)
                .set_primary(model.all_units_moved(global.viewer_id()))
                .disable(!model.is_ready() || model.is_waiting_stage())
                .on_click(|_| Msg::ClickedSubmitTurn)
        };

        Cell::group(
            vec![Style::Absolute, Style::Bottom4, Style::Left4],
            vec![Card::cell_from_rows(
                vec![],
                vec![Row::from_cells(vec![], vec![submit_move_button.cell()])],
            )],
        )
    } else {
        Cell::none()
    }
}

fn overlay_view(model: &Model) -> Cell<Msg> {
    if !model.is_waiting_stage() || model.dialog.is_some() {
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
                let new_origin = path_to_pos(existing_path);
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

        ret.push(Point { x, y });
    }

    ret
}

fn path_with_arrows(path: &[Direction]) -> Vec<(Direction, Arrow)> {
    let mut filtered_path = path.iter().collect::<Vec<_>>();

    let mut index = 0;
    while index < filtered_path.len() {
        let dir = path[index].clone();
        if let Some(next) = path.get(index + 1) {
            if dir == next.opposite() {
                if (index + 1) < filtered_path.len() {
                    filtered_path.remove(index + 1);
                }

                filtered_path.remove(index);

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
    use crate::game::{calc_movement_path, path_with_arrows, Arrow};
    use pretty_assertions::assert_eq;
    use shared::direction::Direction;
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
