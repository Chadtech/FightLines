pub mod action;
mod animation;
mod group_selected;
mod mode;
mod replenishment;
mod stage;
mod unit_selected;
mod view;

use crate::error::Error;
use crate::page::game::action::Action;
use crate::page::game::animation::Animation;
use crate::page::game::mode::Mode;
use crate::page::game::replenishment::Replenishment;
use crate::page::game::stage::animating_moves;
use crate::page::game::stage::taking_turn::Sidebar;
use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::Cell;
use crate::web_sys::HtmlCanvasElement;
use crate::{api, assets, core_ext, global, Row, Style, Toast};
use seed::app::CmdHandle;
use seed::prelude::{
    cmds, el_ref, streams, At, El, ElRef, Ev, IndexMap, JsCast, Node, Orders, St, StreamHandle,
    ToClasses, UpdateEl,
};
use seed::{attrs, canvas, div, style, C};
use shared::api::endpoint::Endpoint;
use shared::api::game::submit_turn;
use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::facing_direction::FacingDirection;
use shared::frame_count::FrameCount;
use shared::game::outcome::Outcome;
use shared::game::unit_index::Indexes;
use shared::game::{calculate_player_visibility, Game, GameId, Turn};
use shared::id::Id;
use shared::located::Located;
use shared::path::Path;
use shared::point::Point;
use shared::team_color::TeamColor;
use shared::tile::Tile;
use shared::unit::{Place, Unit, UnitId};
use shared::{game, located, tile, unit};
use std::cmp;
use std::collections::{HashMap, HashSet};
use web_sys::KeyboardEvent;

///////////////////////////////////////////////////////////////
// Helpers //
///////////////////////////////////////////////////////////////

fn wait_for_render_timeout(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(MIN_RENDER_TIME, || {
        Msg::MinimumRenderTimeExpired
    }))
}

fn wait_for_game_reload_timeout(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(4096, || Msg::GameReloadTimeExpired))
}

fn set_to_moving_unit_mode(model: &mut Model, unit_id: UnitId) -> Result<(), Error> {
    if let Stage::TakingTurn(sub_model) = &mut model.stage {
        let unit_model: &unit::Model = match model.game.get_unit(&unit_id) {
            Some(u) => u,
            None => {
                return Ok(());
            }
        };

        return match model.game.get_units_mobility(&unit_id.clone()) {
            Ok(mobility) => {
                if !unit_model.unit.is_supply_crate() {
                    sub_model.mode = Mode::MovingUnit(mode::moving::Model::init(unit_id, mobility));
                }

                draw_mode(model)
            }
            Err(err_msg) => Err(Error::new(
                "could not get mobility range of unit".to_string(),
                err_msg,
            )),
        };
    }

    Ok(())
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
    game_pixel_size: GamePixelSize,
    game_pos: Point<i16>,
    scroll_pos: Point<i16>,
    handle_minimum_framerate_timeout: CmdHandle,
    handle_game_reload_timeout: Option<CmdHandle>,
    frame_count: FrameCount,
    moves_index_by_unit: HashMap<UnitId, Action>,
    moves: Vec<Action>,
    changes: Vec<Change>,
    mouse_game_position: Option<Point<u32>>,
    stage: Stage,
    dialog: Option<Dialog>,
    status: Status,
    #[allow(dead_code)]
    key_press_stream: StreamHandle,
}

#[derive(Debug)]
struct GamePixelSize {
    width: u16,
    height: u16,
    width_fl: f64,
    height_fl: f64,
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
    fn travel_unit(
        &mut self,
        unit_id: &UnitId,
        path: Path,
        arrows: &[(Direction, Arrow)],
    ) -> Result<(), Error> {
        let taking_turn_model = match &mut self.stage {
            Stage::TakingTurn(m) => m,
            _ => {
                return Ok(());
            }
        };

        let unit = match self.game.indexes.by_id.get(unit_id) {
            Some(m) => m,
            None => {
                return Err(Error::new(
                    "travel unit".to_string(),
                    "could not find unit".to_string(),
                ));
            }
        };

        let action = Action::TraveledTo {
            unit_id: unit_id.clone(),
            path,
            arrows: arrows.to_owned(),
            dismounted_from: match &unit.place {
                Place::OnMap(_) => None,
                Place::InUnit(transport_id) => Some(transport_id.clone()),
            },
        };

        self.moves_index_by_unit.insert(unit_id.clone(), action);

        taking_turn_model.move_completed();

        clear_canvas(&self.mode_canvas, &self.game_pixel_size)
            .map_err(|msg| Error::new("clear mode canvas".to_string(), msg))?;

        Ok(())
    }

    fn get_units_move(&self, unit_id: &UnitId) -> Option<&Action> {
        self.moves_index_by_unit.get(unit_id)
    }

    fn clear_mode_and_sidebar(&mut self) -> Result<(), Error> {
        if let Stage::TakingTurn(sub_model) = &mut self.stage {
            sub_model.move_completed();

            clear_canvas(&self.mode_canvas, &self.game_pixel_size)
                .map_err(|msg| Error::new("clear mode canvas".to_string(), msg))?
        }

        Ok(())
    }

    fn all_units_moved(&self, player_id: Id) -> bool {
        let moved_units = self
            .moves_index_by_unit
            .keys()
            .cloned()
            .collect::<HashSet<UnitId>>();

        let movable_units = self.game.get_units_by_player_id(&player_id);

        let movable_units: HashSet<UnitId> = movable_units
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|(unit_id, unit)| {
                if unit.place.is_on_map() {
                    Some(unit_id.clone())
                } else {
                    None
                }
            })
            .collect::<HashSet<UnitId>>();

        movable_units.is_subset(&moved_units)
    }

    fn is_ready(&self) -> bool {
        self.status == Status::Ready
    }

    fn is_waiting_stage(&self) -> bool {
        matches!(self.stage, Stage::Waiting { .. })
    }
}

#[derive(Debug)]
enum Stage {
    TakingTurn(stage::taking_turn::Model),
    Waiting { indices: Indexes },
    AnimatingMoves(animating_moves::Model),
}

#[derive(PartialEq, Debug, Clone)]
enum Change {
    NameUnit { name: String, unit_id: UnitId },
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
    GameReloadTimeExpired,
    GroupSelectedSidebar(group_selected::Msg),
    UnitSelectedSidebar(unit_selected::Msg),
    MovingFlyout(mode::moving::ClickMsg),
    EnterPressed,
    EscapePressed,
    ScrolledCanvasContainer(Result<Point<i16>, String>),
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

    let game_pixel_width = flags.game.map.width * tile::PIXEL_WIDTH;
    let game_pixel_height = flags.game.map.height * tile::PIXEL_HEIGHT;

    let game_pixel_size = GamePixelSize {
        width: game_pixel_width,
        height: game_pixel_height,
        width_fl: game_pixel_width as f64,
        height_fl: game_pixel_height as f64,
    };

    let game_x = cmp::max(
        0,
        (((window_size.width - 384.0) / 2.0) - (game_pixel_width as f64)) as i16,
    );

    let game_y = cmp::max(
        0,
        ((window_size.height / 2.0) - (game_pixel_height as f64)) as i16,
    );

    orders.after_next_render(|_| Msg::RenderedFirstTime);
    let key_press_stream = orders.stream_with_handle(streams::window_event(Ev::KeyUp, |event| {
        let keyboard_event: KeyboardEvent = event.unchecked_into();
        let key = keyboard_event.key();
        match key.as_str() {
            "Enter" => Some(Msg::EnterPressed),
            "Escape" => Some(Msg::EscapePressed),
            _ => None,
        }
    }));

    let assets = assets::init()?;

    let game = flags.game;

    let stage = if game.waiting_on_player(&global.viewer_id()) {
        Stage::TakingTurn(stage::taking_turn::Model::init())
    } else {
        Stage::Waiting {
            indices: game.indexes.clone(),
        }
    };

    let moves: Vec<Action> = match game.get_turn(global.viewer_id()) {
        Ok(turn) => match turn {
            Turn::Waiting => Vec::new(),
            Turn::Turn { moves } => game_moves_to_page_actions(moves),
        },
        Err(error) => {
            global.toast(
                Toast::init("error", "could not get turn")
                    .error()
                    .with_more_info(error.as_str()),
            );

            Vec::new()
        }
    };

    let moves_index_by_unit = {
        let mut moves_index_ret = HashMap::new();

        for action in &moves {
            match action {
                Action::TraveledTo { unit_id, .. } => {
                    moves_index_ret.insert(unit_id.clone(), action.clone());
                }
                Action::LoadInto {
                    unit_id, load_into, ..
                } => {
                    moves_index_ret.insert(unit_id.clone(), action.clone());
                    moves_index_ret.insert(load_into.clone(), action.clone());
                }
                Action::PickUp {
                    unit_id, cargo_id, ..
                } => {
                    moves_index_ret.insert(unit_id.clone(), action.clone());
                    moves_index_ret.insert(cargo_id.clone(), action.clone());
                }
                Action::DropOff {
                    cargo_unit_loc: loc,
                    ..
                } => {
                    let (_, cargo_unit_id) = loc.value.clone();

                    moves_index_ret.insert(cargo_unit_id, action.clone());
                }
                Action::Replenish {
                    replenishing_unit_id,
                    ..
                } => {
                    moves_index_ret.insert(replenishing_unit_id.clone(), action.clone());
                }
            }
        }

        moves_index_ret
    };

    let model = Model {
        game,
        game_id: flags.game_id,
        game_pixel_size,
        map_canvas: ElRef::<HtmlCanvasElement>::default(),
        units_canvas: ElRef::<HtmlCanvasElement>::default(),
        visibility_canvas: ElRef::<HtmlCanvasElement>::default(),
        mode_canvas: ElRef::<HtmlCanvasElement>::default(),
        cursor_canvas: ElRef::<HtmlCanvasElement>::default(),
        assets,
        game_pos: Point {
            x: game_x,
            y: game_y,
        },
        scroll_pos: Point { x: 0, y: 0 },
        handle_minimum_framerate_timeout: wait_for_render_timeout(orders),
        handle_game_reload_timeout: None,
        frame_count: FrameCount::F1,
        moves_index_by_unit,
        moves,
        changes: Vec::new(),
        mouse_game_position: None,
        stage,
        dialog: None,
        status: Status::Ready,
        key_press_stream,
    };

    Ok(model)
}

fn mouse_screen_pos_to_game_pos(page_pos: Point<i16>, model: &Model) -> Point<i16> {
    let x_on_game = page_pos.x + model.scroll_pos.x - model.game_pos.x - 384;
    let x = x_on_game / ((tile::PIXEL_WIDTH * 2) as i16);

    let y_on_game = page_pos.y + model.scroll_pos.y - model.game_pos.y;
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
            if let Err(error) = handle_rendered_first_frame(model, global.viewer_id()) {
                global.toast_error(error);
            }
        }
        Msg::MouseDownOnScreen(_page_pos) => {}
        Msg::MouseUpOnScreen(page_pos) => {
            if let Stage::TakingTurn { .. } = model.stage {
                if let Err(error) = handle_click_on_screen_during_turn(
                    global.viewer_id(),
                    model,
                    mouse_screen_pos_to_game_pos(page_pos, model),
                ) {
                    global.toast_error(error);
                }
            }
        }
        Msg::MouseMoveOnScreen(page_pos) => {
            if let Err(error) = handle_mouse_move_on_screen(model, page_pos) {
                global.toast_error(error)
            }
        }
        Msg::MinimumRenderTimeExpired => {
            model.frame_count = model.frame_count.succ();

            if let Stage::AnimatingMoves(sub_model) = &mut model.stage {
                match sub_model
                    .progress_animation(&global.viewer_id(), &model.game.map)
                    .map_err(|err_msg| Error::new("progressing animation".to_string(), err_msg))
                {
                    Ok(finished) => {
                        if finished {
                            model.stage = Stage::TakingTurn(stage::taking_turn::Model::init());
                        };
                    }
                    Err(error) => global.toast_error(error),
                }
            }

            let viewer_id = global.viewer_id();

            if let Err(error) = draw(&viewer_id, model) {
                global.toast_error(error)
            }

            model.handle_minimum_framerate_timeout = wait_for_render_timeout(orders);
        }
        Msg::ClickedSubmitTurn => {
            if model.all_units_moved(global.viewer_id()) {
                submit_turn(global, model, orders);
            } else {
                model.dialog = Some(Dialog::ConfirmTurnSubmit);
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
                model.stage = Stage::Waiting {
                    indices: model.game.indexes.clone(),
                };
                model.status = Status::Ready;

                refetch_game(&global.viewer_id(), model, res.game, orders);
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
                refetch_game(&global.viewer_id(), model, res.get_game(), orders);
            }
            Err(err) => {
                global.toast(
                    Toast::init("error", "failed to fetch game")
                        .error()
                        .with_more_info(err),
                );
            }
        },
        Msg::GameReloadTimeExpired => {
            let url = Endpoint::make_get_game(model.game_id.clone());

            model.status = Status::Waiting;

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
        }
        Msg::UnitSelectedSidebar(sub_msg) => {
            if let Err(error) = handle_unit_selected_sidebar_msg(model, sub_msg) {
                global.toast_error(error);
            }
        }
        Msg::GroupSelectedSidebar(sub_msg) => {
            if let Err(error) = handle_group_selected_sidebar_msg(model, sub_msg) {
                global.toast_error(error);
            }
        }
        Msg::MovingFlyout(sub_msg) => {
            if let Err(error) = handle_moving_flyout_msg(&global.viewer_id(), model, sub_msg) {
                global.toast_error(error)
            }
        }
        Msg::EnterPressed => match &model.dialog {
            None => {}
            Some(dialog) => match dialog {
                Dialog::ConfirmTurnSubmit => {
                    submit_turn(global, model, orders);
                }
            },
        },
        Msg::EscapePressed => {
            model.dialog = None;
            match &mut model.stage {
                Stage::TakingTurn(sub_model) => {
                    sub_model.mode = Mode::None;

                    match &mut sub_model.sidebar {
                        Sidebar::None => {}
                        Sidebar::UnitSelected(unit_selected_model) => {
                            sub_model.sidebar = if let Some(group_selected_model) =
                                &unit_selected_model.from_group
                            {
                                Sidebar::GroupSelected(group_selected_model.clone())
                            } else {
                                Sidebar::None
                            };
                        }
                        Sidebar::GroupSelected(_) => {
                            sub_model.sidebar = Sidebar::None;
                        }
                    }

                    if let Err(error) = draw_mode(model) {
                        global.toast_error(error);
                    };
                }
                Stage::Waiting { .. } => {}
                Stage::AnimatingMoves(_) => {}
            }
        }
        Msg::ScrolledCanvasContainer(pos_result) => {
            if let Ok(pos) = pos_result {
                model.scroll_pos = pos;
            }
        }
    }
}

fn handle_group_selected_sidebar_msg(
    model: &mut Model,
    msg: group_selected::Msg,
) -> Result<(), Error> {
    match msg {
        group_selected::Msg::UnitRow(view::unit_row::Msg::Clicked(unit_id)) => {
            if model.get_units_move(&unit_id).is_some() {
                return Ok(());
            }

            let taking_turn_model = if let Stage::TakingTurn(m) = &mut model.stage {
                m
            } else {
                return Ok(());
            };

            let sub_model = if let Sidebar::GroupSelected(m) = &mut taking_turn_model.sidebar {
                m
            } else {
                return Ok(());
            };

            taking_turn_model.sidebar = Sidebar::UnitSelected(unit_selected::Model::init(
                unit_id.clone(),
                Some(sub_model.clone()),
            ));

            if let Stage::TakingTurn { .. } = &mut model.stage {
                return set_to_moving_unit_mode(model, unit_id);
            }

            Ok(())
        }
    }
}

fn handle_unit_selected_sidebar_msg(
    model: &mut Model,
    msg: unit_selected::Msg,
) -> Result<(), Error> {
    let taking_turn_model = if let Stage::TakingTurn(m) = &mut model.stage {
        m
    } else {
        return Ok(());
    };

    let sub_model = if let Sidebar::UnitSelected(m) = &mut taking_turn_model.sidebar {
        m
    } else {
        return Ok(());
    };

    match msg {
        unit_selected::Msg::UpdatedUnitNameField(new_field) => {
            sub_model.name_field = new_field;
        }
        unit_selected::Msg::ClickedSetName => {
            if !sub_model.name_submitted {
                sub_model.name_submitted = true;
                model.changes.push(Change::NameUnit {
                    name: sub_model.name_field.clone(),
                    unit_id: sub_model.unit_id.clone(),
                })
            }
        }
        unit_selected::Msg::ClickedBackToGroup => {
            if let Some(group_model) = sub_model.from_group.clone() {
                taking_turn_model.sidebar = Sidebar::GroupSelected(group_model);
            }
        }
        unit_selected::Msg::UnitRow(view::unit_row::Msg::Clicked(cargo_unit_id)) => {
            if let Some(cargo_unit_model) = model.game.get_unit(&cargo_unit_id) {
                if cargo_unit_model.unit.is_supply_crate() {
                    let unit_model = match model.game.get_unit(&sub_model.unit_id) {
                        Some(u) => u,
                        None => {
                            return Ok(());
                        }
                    };

                    let loc = match unit_model.place.clone() {
                        Place::OnMap(l) => l,
                        Place::InUnit(_) => {
                            return Ok(());
                        }
                    };

                    model.moves_index_by_unit.insert(
                        cargo_unit_id.clone(),
                        Action::DropOff {
                            cargo_unit_loc: Located {
                                x: loc.x,
                                y: loc.y,
                                value: (loc.value, cargo_unit_id.clone()),
                            },
                            transport_id: sub_model.unit_id.clone(),
                        },
                    );
                } else {
                    return set_to_moving_unit_mode(model, cargo_unit_id);
                }
            }
        }
    }

    Ok(())
}

fn handle_mouse_move_on_screen(model: &mut Model, page_pos: Point<i16>) -> Result<(), Error> {
    let Point { x, y } = mouse_screen_pos_to_game_pos(page_pos, model);

    model.mouse_game_position = if x < 0
        || y < 0
        || (model.game.map.width as i16) < x
        || (model.game.map.height as i16) < y
    {
        None
    } else {
        let mouse_loc = located::unit(x as u16, y as u16);

        handle_mouse_move_for_mode(model, mouse_loc)?;

        Some(Point {
            x: x as u32,
            y: y as u32,
        })
    };

    draw_cursor(model).map_err(|err_msg| Error::new("could not render cursor".to_string(), err_msg))
}

fn handle_rendered_first_frame(model: &mut Model, viewer_id: Id) -> Result<(), Error> {
    draw_terrain(model);

    draw(&viewer_id, model)
}

fn handle_moving_flyout_msg(
    viewer_id: &Id,
    model: &mut Model,
    msg: mode::moving::ClickMsg,
) -> Result<(), Error> {
    let sub_model = if let Stage::TakingTurn(stage::taking_turn::Model {
        mode: Mode::MovingUnit(sub_model),
        ..
    }) = &mut model.stage
    {
        sub_model
    } else {
        return Ok(());
    };

    let error_title = "handle moving flyout msg".to_string();

    let unit_id = &sub_model.unit_id.clone();

    match msg {
        mode::moving::ClickMsg::PickUp(cargo_unit_id) => {
            let arrows = &sub_model.arrows.clone();

            let path = sub_model
                .path(unit_id, &model.game)
                .map_err(|err_msg| Error::new(error_title, err_msg))?;

            model.moves_index_by_unit.insert(
                sub_model.unit_id.clone(),
                Action::PickUp {
                    unit_id: unit_id.clone(),
                    cargo_id: cargo_unit_id,
                    arrows: arrows.clone(),
                    path,
                },
            );

            model.clear_mode_and_sidebar()
        }
        mode::moving::ClickMsg::LoadInto(rideable_unit_id) => {
            let arrows = &sub_model.arrows.clone();

            let path = sub_model
                .path(unit_id, &model.game)
                .map_err(|err_msg| Error::new(error_title, err_msg))?;

            model.moves_index_by_unit.insert(
                sub_model.unit_id.clone(),
                Action::LoadInto {
                    unit_id: unit_id.clone(),
                    load_into: rideable_unit_id,
                    arrows: arrows.clone(),
                    path,
                },
            );

            model.clear_mode_and_sidebar()
        }
        mode::moving::ClickMsg::MoveTo => {
            let arrows = &sub_model.arrows.clone();

            let path = sub_model
                .path(unit_id, &model.game)
                .map_err(|err_msg| Error::new(error_title, err_msg))?;

            model.travel_unit(unit_id, path, arrows)
        }
        mode::moving::ClickMsg::Replenish => {
            let path = sub_model
                .path(unit_id, &model.game)
                .map_err(|err_msg| Error::new(error_title.clone(), err_msg))?;

            let replenishment_pos = match path.last_pos() {
                Some(p) => p,
                None => {
                    return Err(Error::new(
                        error_title,
                        "replenishing unit does not travel".to_string(),
                    ));
                }
            };

            match Replenishment::calculate(
                viewer_id,
                &sub_model.unit_id,
                replenishment_pos,
                &model.game.indexes,
            ) {
                Ok(replenishment) => {
                    let arrows = &sub_model.arrows.clone();

                    model.moves_index_by_unit.insert(
                        sub_model.unit_id.clone(),
                        Action::Replenish {
                            replenishing_unit_id: sub_model.unit_id.clone(),
                            units: replenishment.replenished_units,
                            depleted_supply_crates: replenishment.depleted_supply_crates,
                            arrows: arrows.clone(),
                            path,
                        },
                    );

                    Ok(())
                }
                Err(err_msg) => Err(Error::new(error_title, err_msg)),
            }?;

            model.clear_mode_and_sidebar()
        }
    }
}

fn refetch_game(
    viewer_id: &Id,
    model: &mut Model,
    fetched_game: Game,
    orders: &mut impl Orders<Msg>,
) {
    if model.game.turn_number == fetched_game.turn_number {
        model.handle_game_reload_timeout = Some(wait_for_game_reload_timeout(orders));
    } else {
        model.stage = match &model.stage {
            Stage::Waiting { indices } => {
                let prev_outcomes = fetched_game.clone().prev_outcomes;

                let animations = outcomes_to_animations(prev_outcomes);

                let visibility =
                    calculate_player_visibility(viewer_id, &model.game.map, &indices.by_id);

                let sub_model = animating_moves::Model::init(
                    indices.clone(),
                    animations,
                    visibility,
                    model.game.day(),
                );

                Stage::AnimatingMoves(sub_model)
            }
            _ => Stage::TakingTurn(stage::taking_turn::Model::init()),
        };
        model.status = Status::Ready;
        model.moves_index_by_unit = HashMap::new();
        model.moves = Vec::new();
    }

    model.game = fetched_game;
}

fn submit_turn(global: &mut global::Model, model: &mut Model, orders: &mut impl Orders<Msg>) {
    model.dialog = None;

    let mut req_moves: Vec<game::action::Action> = model
        .moves_index_by_unit
        .iter()
        .map(|(unit_id, action)| match action {
            Action::TraveledTo {
                path,
                dismounted_from,
                ..
            } => game::action::Action::Travel {
                unit_id: unit_id.clone(),
                path: path.clone(),
                dismounted_from: dismounted_from.clone(),
            },
            Action::LoadInto {
                load_into, path, ..
            } => game::action::Action::LoadInto {
                unit_id: unit_id.clone(),
                load_into: load_into.clone(),
                path: path.clone(),
            },
            Action::PickUp { cargo_id, path, .. } => game::action::Action::PickUp {
                unit_id: unit_id.clone(),
                cargo_id: cargo_id.clone(),
                path: path.clone(),
            },
            Action::DropOff {
                cargo_unit_loc: loc,
                transport_id,
            } => game::action::Action::DropOff {
                cargo_unit_loc: loc.clone(),
                transport_id: transport_id.clone(),
            },
            Action::Replenish {
                replenishing_unit_id,
                units,
                depleted_supply_crates,
                path,
                ..
            } => game::action::Action::Replenish {
                replenishing_unit_id: replenishing_unit_id.clone(),
                units: units.clone(),
                depleted_supply_crates: depleted_supply_crates.clone(),
                path: path.clone(),
            },
        })
        .collect();

    let mut rng = global.new_rand_gen();
    game::action::order(&mut rng, &mut req_moves);

    let req_changes: Vec<game::Change> = model
        .changes
        .iter()
        .map(|change| match change {
            Change::NameUnit { unit_id, name } => game::Change::NameUnit {
                unit_id: unit_id.clone(),
                name: name.clone(),
            },
        })
        .collect();

    let req: submit_turn::Request = submit_turn::Request::init(req_moves, req_changes);

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
    viewer_id: Id,
    model: &mut Model,
    mouse_pos: Point<i16>,
) -> Result<(), Error> {
    let Point { x, y } = mouse_pos;

    if !(x >= 0 && y >= 0) {
        return Ok(());
    }

    let x = x as u16;
    let y = y as u16;

    if let Stage::TakingTurn(stage::taking_turn::Model { mode, .. }) = &mut model.stage {
        return match mode {
            Mode::None => handle_click_on_screen_when_no_mode(viewer_id, model, x, y),
            Mode::MovingUnit(moving_model) => {
                let mouse_loc = located::unit(x, y);

                if moving_model.mobility.contains(&mouse_loc) {
                    handle_click_on_screen_when_move_mode(viewer_id, model, &mouse_loc)
                } else {
                    model.clear_mode_and_sidebar()
                }
            }
        };
    }

    Ok(())
}

fn handle_click_on_screen_when_move_mode(
    viewer_id: Id,
    model: &mut Model,
    mouse_loc: &Located<()>,
) -> Result<(), Error> {
    if let Stage::TakingTurn(stage::taking_turn::Model {
        mode: Mode::MovingUnit(moving_model),
        ..
    }) = &mut model.stage
    {
        let error_title = "handle move click".to_string();

        let unit_id = moving_model.unit_id.clone();

        let unit_model = match model.game.get_unit(&unit_id) {
            Some(unit_model) => unit_model,
            None => return Error::throw(error_title, "could not get unit".to_string()),
        };

        let unit_pos = match &unit_model.place {
            Place::OnMap(loc) => Some(loc.to_unit()),
            Place::InUnit(_) => None,
        };

        // If the user clicks back on the unit we should
        // exit out of the move mode
        if unit_pos == Some(mouse_loc.clone()) {
            return model.clear_mode_and_sidebar();
        }

        // If the unit clicks a unit that can carry it, we
        // need to bring up menu options to select
        let path = moving_model
            .path(&moving_model.unit_id, &model.game)
            .map_err(|err_msg| Error::new(error_title.clone(), err_msg))?;

        let mut ride_options = vec![];

        // The ride options for loading into another unit, like a truck
        if let Some(rideable_units) =
            model
                .game
                .get_rideable_units_by_location(&viewer_id, &unit_model.unit, mouse_loc)
        {
            let mut rideable_unit_options = rideable_units
                .iter()
                .map(|(rideable_unit_id, rideable_unit)| {
                    mode::moving::RideOption::load_into(
                        rideable_unit_id.clone(),
                        rideable_unit
                            .name
                            .clone()
                            .unwrap_or_else(|| rideable_unit.unit.to_string()),
                    )
                })
                .collect::<Vec<mode::moving::RideOption>>();

            ride_options.append(&mut rideable_unit_options);
        }

        // The ride options for picking up a unit, like a supply crate
        let visibility = model
            .game
            .get_players_visibility(&viewer_id)
            .map_err(|err| Error::new(error_title, err))?;

        if unit_model.unit.can_pick_up_supply_crates() && visibility.contains(mouse_loc) {
            if let Some(supply_crates) = model.game.get_supply_crates_by_location(mouse_loc) {
                ride_options.append(
                    &mut supply_crates
                        .iter()
                        .map(|(supply_crate_id, supply_crate)| {
                            mode::moving::RideOption::pick_up(
                                supply_crate_id.clone(),
                                supply_crate
                                    .name
                                    .clone()
                                    .unwrap_or_else(|| supply_crate.unit.to_string()),
                            )
                        })
                        .collect::<Vec<mode::moving::RideOption>>(),
                )
            }
        }

        // the option to replenish troops
        if let Some(cargo) = model.game.indexes.by_transport.get(&unit_id) {
            let crates = cargo
                .iter()
                .filter(|(_, cargo_unit_model)| cargo_unit_model.unit.is_supply_crate())
                .collect::<Vec<_>>();

            if !crates.is_empty() {
                ride_options.push(mode::moving::RideOption::Replenish)
            }
        }

        if ride_options.is_empty() {
            let arrows = moving_model.arrows.clone();

            model.travel_unit(&unit_id, path, &arrows)?;
        } else {
            moving_model.with_options(mouse_loc.x, mouse_loc.y, ride_options, path);
        }
    }

    Ok(())
}

fn handle_click_on_screen_when_no_mode(
    viewer_id: Id,
    model: &mut Model,
    x: u16,
    y: u16,
) -> Result<(), Error> {
    let taking_turn_model = match &mut model.stage {
        Stage::TakingTurn(m) => m,
        _ => {
            return Ok(());
        }
    };

    let units_at_pos = match model.game.get_units_by_location(&located::unit(x, y)) {
        Some(units) => units,
        None => {
            return Ok(());
        }
    };

    let (first, rest) = match units_at_pos.split_first() {
        Some(s) => s,
        None => {
            return Ok(());
        }
    };
    let (first_unit_id, _, unit_model) = first;

    if unit_model.owner != viewer_id {
        return Ok(());
    }

    if rest.is_empty() {
        taking_turn_model.sidebar = Sidebar::UnitSelected(unit_selected::Model::init(
            first_unit_id.clone(),
            None::<group_selected::Model>,
        ));

        set_to_moving_unit_mode(model, first_unit_id.clone())
    } else {
        let mut units = vec![];

        units.push(first_unit_id.clone());

        for (unit_id, _, _) in rest {
            units.push(unit_id.clone());
        }

        taking_turn_model.sidebar = if taking_turn_model.sidebar.is_group_at(located::unit(x, y)) {
            Sidebar::None
        } else {
            Sidebar::GroupSelected(group_selected::Model::init(units, located::unit(x, y)))
        };

        Ok(())
    }
}

fn handle_mouse_move_for_mode(model: &mut Model, mouse_loc: Located<()>) -> Result<(), Error> {
    let mode = if let Stage::TakingTurn(stage::taking_turn::Model { mode, .. }) = &mut model.stage {
        mode
    } else {
        return Ok(());
    };

    match mode {
        Mode::None => {}
        Mode::MovingUnit(moving_model) => {
            if moving_model.ride_options.is_some() {
                return Ok(());
            }

            if moving_model.mobility.contains(&mouse_loc) {
                let error_title = "handle mouse move in move mode".to_string();
                let unit_model = model.game.get_unit(&moving_model.unit_id).ok_or_else(|| {
                    Error::new(
                        error_title.clone(),
                        "could not find unit in moving model".to_string(),
                    )
                })?;

                let loc = model
                    .game
                    .position_of_unit_or_transport(&moving_model.unit_id)
                    .map_err(|err| Error::new(error_title, err))?;

                let mouse_point = Point {
                    x: mouse_loc.x as i32 - loc.x as i32,
                    y: mouse_loc.y as i32 - loc.y as i32,
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
                    unit_model.unit.mobility_budget() as usize,
                );

                moving_model.arrows = arrows;
            } else {
                moving_model.arrows = vec![];
            }
        }
    }

    draw_mode(model)?;

    Ok(())
}

fn draw(viewer_id: &Id, model: &Model) -> Result<(), Error> {
    let visibility = match &model.stage {
        Stage::AnimatingMoves(sub_model) => &sub_model.visibility,
        _ => model
            .game
            .get_players_visibility(viewer_id)
            .map_err(|err_msg| Error::new("visibility rendering problem".to_string(), err_msg))?,
    };

    draw_units(visibility, model);

    draw_visibility(visibility, model);

    Ok(())
}

fn clear_canvas(
    canvas_ref: &ElRef<HtmlCanvasElement>,
    game_pixel_size: &GamePixelSize,
) -> Result<(), String> {
    let html_canvas = canvas_ref
        .get()
        .ok_or_else(|| "could not get canvas element to clear".to_string())?;

    let ctx = seed::canvas_context_2d(&html_canvas);

    ctx.begin_path();
    ctx.clear_rect(0., 0., game_pixel_size.width_fl, game_pixel_size.height_fl);

    Ok(())
}

fn draw_mode(model: &Model) -> Result<(), Error> {
    let mode = if let Stage::TakingTurn(stage::taking_turn::Model { mode, .. }) = &model.stage {
        mode
    } else {
        return Ok(());
    };

    let canvas = match model.mode_canvas.get() {
        Some(c) => c,
        None => {
            return Ok(());
        }
    };

    let ctx = seed::canvas_context_2d(&canvas);

    ctx.begin_path();
    ctx.clear_rect(
        0.,
        0.,
        model.game_pixel_size.width_fl,
        model.game_pixel_size.height_fl,
    );

    match &mode {
        Mode::None => {}
        Mode::MovingUnit(moving_model) => {
            let error_title = "rendering mobility range".to_string();
            for mobility_space in moving_model.mobility.iter() {
                let _ = ctx
                    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        &model.assets.sheet,
                        MISC_SPRITE_SHEET_COLUMN,
                        48.0,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                        mobility_space.x as f64 * tile::PIXEL_WIDTH_FL,
                        mobility_space.y as f64 * tile::PIXEL_HEIGHT_FL,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                    );
            }

            let loc = model
                .game
                .position_of_unit_or_transport(&moving_model.unit_id)
                .map_err(|err| Error::new(error_title.clone(), err))?;

            let mut arrow_x = loc.x;
            let mut arrow_y = loc.y;
            for (dir, arrow) in &moving_model.arrows {
                draw_arrow(&ctx, model, arrow, dir, &mut arrow_x, &mut arrow_y, false);
            }
        }
    }

    Ok(())
}

fn draw_arrows(
    ctx: &web_sys::CanvasRenderingContext2d,
    model: &Model,
    game_pos: &Located<()>,
    arrows: &Vec<(Direction, Arrow)>,
) {
    let mut arrow_x = game_pos.x;
    let mut arrow_y = game_pos.y;
    for (dir, arrow) in arrows {
        draw_arrow(ctx, model, arrow, dir, &mut arrow_x, &mut arrow_y, true);
    }
}

fn draw_arrow(
    ctx: &web_sys::CanvasRenderingContext2d,
    model: &Model,
    arrow: &Arrow,
    dir: &Direction,
    arrow_x: &mut u16,
    arrow_y: &mut u16,
    moved: bool,
) {
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

    let _ = ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &model.assets.sheet,
        MISC_SPRITE_SHEET_COLUMN,
        sheet_row,
        tile::PIXEL_WIDTH_FL,
        tile::PIXEL_HEIGHT_FL,
        *arrow_x as f64 * tile::PIXEL_WIDTH_FL,
        *arrow_y as f64 * tile::PIXEL_HEIGHT_FL,
        tile::PIXEL_WIDTH_FL,
        tile::PIXEL_HEIGHT_FL,
    );
}

fn draw_cursor(model: &Model) -> Result<(), String> {
    let canvas = model
        .cursor_canvas
        .get()
        .ok_or_else(|| "could not get cursor canvas element".to_string())?;
    let ctx = seed::canvas_context_2d(&canvas);

    ctx.begin_path();
    ctx.clear_rect(
        0.,
        0.,
        model.game_pixel_size.width_fl,
        model.game_pixel_size.height_fl,
    );

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

fn draw_visibility(visibility: &HashSet<Located<()>>, model: &Model) {
    let canvas = match model.visibility_canvas.get() {
        Some(c) => c,
        None => {
            return;
        }
    };

    let ctx = seed::canvas_context_2d(&canvas);

    ctx.begin_path();
    ctx.clear_rect(
        0.,
        0.,
        model.game_pixel_size.width_fl,
        model.game_pixel_size.height_fl,
    );

    for x in 0..model.game.map.width {
        for y in 0..model.game.map.height {
            let loc = located::unit(x, y);

            if !visibility.contains(&loc) {
                let sheet = &model.assets.sheet;

                let sx = MISC_SPRITE_SHEET_COLUMN;
                let sy = 16.0;

                let x_fl = (x * tile::PIXEL_WIDTH) as f64;
                let y_fl = (y * tile::PIXEL_HEIGHT) as f64;

                let _ = ctx
                    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        sheet,
                        sx,
                        sy,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                        x_fl,
                        y_fl,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                    );
            }
        }
    }
}

fn draw_units(visibility: &HashSet<Located<()>>, model: &Model) {
    let canvas = match model.units_canvas.get() {
        Some(c) => c,
        None => {
            return;
        }
    };
    let ctx = seed::canvas_context_2d(&canvas);

    ctx.begin_path();
    ctx.clear_rect(
        0.,
        0.,
        model.game_pixel_size.width_fl,
        model.game_pixel_size.height_fl,
    );

    let indices = match &model.stage {
        Stage::AnimatingMoves(sub_model) => &sub_model.indices,
        _ => &model.game.indexes,
    };

    for (game_pos, units) in indices.by_location.iter() {
        let location = located::unit(game_pos.x, game_pos.y);

        let draw_units_move = |maybe_units_move: Option<&Action>| {
            if let Some(units_move) = maybe_units_move {
                match units_move {
                    Action::TraveledTo { arrows, .. } => {
                        draw_arrows(&ctx, model, game_pos, arrows);
                    }
                    Action::LoadInto { arrows, .. } => {
                        draw_arrows(&ctx, model, game_pos, arrows);
                    }
                    Action::PickUp { arrows, .. } => {
                        draw_arrows(&ctx, model, game_pos, arrows);
                    }
                    Action::DropOff { .. } => {}
                    Action::Replenish { arrows, .. } => {
                        draw_arrows(&ctx, model, game_pos, arrows);
                    }
                };
            }
        };

        let draw_passender_units_moves = |unit_id: &UnitId| {
            if let Some(loaded_units) = indices.by_transport.get(unit_id) {
                for (loaded_unit_id, _) in loaded_units {
                    draw_units_move(model.get_units_move(loaded_unit_id));
                }
            };
        };

        if visibility.contains(&location) {
            let x = (game_pos.x * tile::PIXEL_WIDTH) as f64;
            let y = (game_pos.y * tile::PIXEL_HEIGHT) as f64;

            if units.len() == 1 {
                let (unit_id, facing_direction, unit_model) = units.get(0).unwrap();

                let sheet_for_unit_draw = match facing_direction {
                    FacingDirection::Left => &model.assets.sheet_flipped,
                    FacingDirection::Right => &model.assets.sheet,
                };

                let maybe_units_move = model.get_units_move(unit_id);

                let (sx, sy) = {
                    let mut sx = {
                        let sprite_sheet_x = match model.frame_count {
                            FrameCount::F1 => 0.0,
                            FrameCount::F2 => 1.0,
                            FrameCount::F3 => 2.0,
                            FrameCount::F4 => 3.0,
                        };
                        match facing_direction {
                            FacingDirection::Left => SPRITE_SHEET_WIDTH - sprite_sheet_x - 1.0,
                            FacingDirection::Right => sprite_sheet_x,
                        }
                    };

                    if maybe_units_move.is_some() {
                        match facing_direction {
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
                        Unit::Truck => 4.0,
                        Unit::SupplyCrate => 6.0,
                    };

                    if unit_model.color == TeamColor::Blue {
                        sy += 1.0;
                    };

                    let sx_px = sx * tile::PIXEL_WIDTH_FL;
                    let sy_px = sy * tile::PIXEL_HEIGHT_FL;

                    (sx_px, sy_px)
                };

                let _ = ctx
                    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        sheet_for_unit_draw,
                        sx,
                        sy,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                        x,
                        y,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                    );

                draw_units_move(maybe_units_move);

                if let Some(loaded_units) = indices.by_transport.get(unit_id) {
                    let misc_sheet_row =
                        if loaded_units.len() >= unit_model.unit.carrying_capacity() {
                            28.0
                        } else {
                            26.0
                        };

                    let _ = ctx
                        .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                            &model.assets.sheet,
                            MISC_SPRITE_SHEET_COLUMN,
                            misc_sheet_row * tile::PIXEL_HEIGHT_FL,
                            tile::PIXEL_WIDTH_FL,
                            tile::PIXEL_HEIGHT_FL,
                            x,
                            y,
                            tile::PIXEL_WIDTH_FL,
                            tile::PIXEL_HEIGHT_FL,
                        );

                    for (loaded_unit_id, _) in loaded_units {
                        draw_units_move(model.get_units_move(loaded_unit_id));
                    }
                };

                let supplies_below_25_percent = 0.25 > unit_model.supplies_percent();

                match model.frame_count {
                    FrameCount::F1 | FrameCount::F2 => {
                        if supplies_below_25_percent {
                            let _ = ctx
                            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                &model.assets.sheet,
                                MISC_SPRITE_SHEET_COLUMN,
                                27.0 * tile::PIXEL_WIDTH_FL,
                                tile::PIXEL_WIDTH_FL,
                                tile::PIXEL_HEIGHT_FL,
                                x,
                                y,
                                tile::PIXEL_WIDTH_FL,
                                tile::PIXEL_HEIGHT_FL,
                            );
                        }
                    }
                    _ => {}
                }
            } else {
                let mut colors = HashSet::new();

                for (_, _, unit_model) in units {
                    colors.insert(unit_model.color.clone());
                }

                let colors_vec = colors.into_iter().collect::<Vec<TeamColor>>();

                let sy = if colors_vec.len() == 1 {
                    let color = colors_vec.get(0).unwrap();

                    let y = match color {
                        TeamColor::Red => 0.0,
                        TeamColor::Blue => 1.0,
                    };

                    y * tile::PIXEL_HEIGHT_FL
                } else {
                    todo!("Sprite for game pos with multiple teams on it")
                };

                let _ = ctx
                    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        &model.assets.sheet,
                        9.0 * tile::PIXEL_WIDTH_FL,
                        sy,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                        x,
                        y,
                        tile::PIXEL_WIDTH_FL,
                        tile::PIXEL_HEIGHT_FL,
                    );

                for (unit_id, _, _) in units {
                    draw_units_move(model.get_units_move(unit_id));
                    draw_passender_units_moves(unit_id);
                }
            }
        }
    }
}

fn draw_terrain(model: &Model) {
    let canvas = match model.map_canvas.get() {
        Some(c) => c,
        None => {
            return;
        }
    };
    let ctx = seed::canvas_context_2d(&canvas);

    ctx.begin_path();
    ctx.clear_rect(
        0.,
        0.,
        model.game_pixel_size.width_fl,
        model.game_pixel_size.height_fl,
    );

    let grid = &model.game.map.grid;

    for row in grid {
        for loc_tile in row {
            let x = (loc_tile.x * tile::PIXEL_WIDTH) as f64;
            let y = (loc_tile.y * tile::PIXEL_HEIGHT) as f64;

            let sheet_row = match loc_tile.value {
                Tile::GrassPlain => 0.0,
                Tile::Hills => 24.0,
                Tile::Forest => 25.0,
            };

            let _ = ctx
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &model.assets.sheet,
                    MISC_SPRITE_SHEET_COLUMN,
                    sheet_row * tile::PIXEL_HEIGHT_FL,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                    x,
                    y,
                    tile::PIXEL_WIDTH_FL,
                    tile::PIXEL_HEIGHT_FL,
                );
        }
    }
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(global: &global::Model, model: &Model) -> Vec<Cell<Msg>> {
    let canvases = Cell::group(
        vec![
            Style::Absolute,
            Style::Scroll,
            Style::Top0,
            Style::Bottom0,
            Style::Right0,
            Style::Left8P5,
        ],
        vec![
            map_canvas_cell(model),
            units_canvas_cell(model),
            visibility_canvas_cell(model),
            mode_canvas_cell(model),
            cursor_canvas_cell(model),
            click_screen(model),
            flyout_view(model),
        ],
    )
    .with_html_id(CANVAS_CONTAINER_HTML_ID)
    .on_scroll(|_| {
        let maybe_pos: Option<Point<i16>> = seed::document()
            .get_element_by_id(CANVAS_CONTAINER_HTML_ID)
            .map(|el| {
                let scroll_top = el.scroll_top() as i16;
                let scroll_left = el.scroll_left() as i16;

                Point {
                    x: scroll_left,
                    y: scroll_top,
                }
            });

        let pos_result = match maybe_pos {
            None => Err("could not find canvas container element".to_string()),
            Some(pos) => Ok(pos),
        };

        Msg::ScrolledCanvasContainer(pos_result)
    });

    vec![
        sidebar_view(global, model),
        canvases,
        snackbar_view(model),
        day_view(model),
        dialog_view(model),
    ]
}

const CANVAS_CONTAINER_HTML_ID: &str = "canvases_container";

fn mode_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(
        vec![],
        vec![game_canvas(model, &model.mode_canvas, "mode".to_string())],
    )
}

fn cursor_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(
        vec![],
        vec![game_canvas(
            model,
            &model.cursor_canvas,
            "cursor".to_string(),
        )],
    )
}

fn visibility_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(
        vec![],
        vec![game_canvas(
            model,
            &model.visibility_canvas,
            "visibility".to_string(),
        )],
    )
}

fn units_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(
        vec![],
        vec![game_canvas(model, &model.units_canvas, "units".to_string())],
    )
}

fn map_canvas_cell(model: &Model) -> Cell<Msg> {
    Cell::from_html(
        vec![],
        vec![game_canvas(model, &model.map_canvas, "map".to_string())],
    )
}

fn game_canvas(model: &Model, r: &ElRef<HtmlCanvasElement>, html_id: String) -> Node<Msg> {
    canvas![
        C![Style::Absolute.css_classes().concat()],
        attrs! {
            At::Width => px_u16(model.game_pixel_size.width).as_str(),
            At::Height => px_u16(model.game_pixel_size.height).as_str()
            At::Id => html_id.as_str()
        },
        style! {
            St::Left => px_i16(model.game_pos.x).as_str(),
            St::Top => px_i16(model.game_pos.y).as_str(),
            St::Width => px_u16(model.game_pixel_size.width * 2).as_str(),
            St::Height => px_u16(model.game_pixel_size.height * 2).as_str()
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

    let screen: Node<Msg> = div![
        C![Style::Absolute.css_classes().concat()],
        attrs! {
            At::Width => px_u16(model.game_pixel_size.width).as_str(),
            At::Height => px_u16(model.game_pixel_size.height).as_str()
        },
        style! {
            St::Left => px_i16(model.game_pos.x).as_str(),
            St::Top => px_i16(model.game_pos.y).as_str(),
            St::Width => px_u16(model.game_pixel_size.width * 2).as_str(),
            St::Height => px_u16(model.game_pixel_size.height * 2).as_str()
        },
    ];

    Cell::from_html(vec![cursor_style], vec![screen])
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

fn sidebar_view(global: &global::Model, model: &Model) -> Cell<Msg> {
    let submit_button = {
        let label = "submit turn";

        Button::simple(label)
            .set_primary(model.all_units_moved(global.viewer_id()))
            .disable(!model.is_ready() || model.is_waiting_stage())
            .on_click(|_| Msg::ClickedSubmitTurn)
    };

    Cell::group(
        vec![
            Style::W8P5,
            Style::Absolute,
            Style::Bottom0,
            Style::Top0,
            Style::Left0,
            Style::BgContent1,
            Style::BorderRContent0,
            Style::P4,
            Style::FlexCol,
        ],
        vec![
            Cell::group(
                vec![Style::Grow, Style::FlexCol, Style::G4],
                sidebar_content(model),
            ),
            submit_button.cell(),
        ],
    )
}

fn sidebar_content(model: &Model) -> Vec<Cell<Msg>> {
    match &model.stage {
        Stage::TakingTurn(taking_turn_model) => match &taking_turn_model.sidebar {
            Sidebar::None => {
                vec![]
            }
            Sidebar::GroupSelected(sub_model) => {
                group_selected::sidebar_content(sub_model, &model.moves_index_by_unit, &model.game)
                    .into_iter()
                    .map(|cell| cell.map_msg(Msg::GroupSelectedSidebar))
                    .collect::<Vec<_>>()
            }
            Sidebar::UnitSelected(sub_model) => match model.game.get_unit(&sub_model.unit_id) {
                None => {
                    vec![Cell::from_str(vec![], "error: could not find unit")]
                }
                Some(unit_model) => unit_selected::sidebar_content(
                    sub_model,
                    model.game.transport_index(),
                    unit_model,
                    &model.moves_index_by_unit,
                    &model.game,
                )
                .into_iter()
                .map(|cell| cell.map_msg(Msg::UnitSelectedSidebar))
                .collect::<Vec<_>>(),
            },
        },
        Stage::Waiting { .. } => {
            vec![]
        }
        Stage::AnimatingMoves(sub_model) => {
            animating_moves::sidebar_view(&model.game.indexes.by_id, sub_model)
        }
    }
}

fn day_view(model: &Model) -> Cell<Msg> {
    let day = match &model.stage {
        Stage::AnimatingMoves(sub_model) => sub_model.day.clone(),
        _ => model.game.day(),
    };

    Cell::from_str(
        vec![
            Style::Absolute,
            Style::Right0,
            Style::Top0,
            Style::P4,
            Style::BgContent1,
            Style::BorderBContent0,
            Style::BorderLContent2,
        ],
        day.to_string().as_str(),
    )
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

fn flyout_view(model: &Model) -> Cell<Msg> {
    if let Stage::TakingTurn(stage::taking_turn::Model {
        mode: Mode::MovingUnit(moving_model),
        ..
    }) = &model.stage
    {
        mode::moving::flyout_view(moving_model, &model.game_pos).map_msg(Msg::MovingFlyout)
    } else {
        Cell::none()
    }
}

fn snackbar_view(model: &Model) -> Cell<Msg> {
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
const SPRITE_SHEET_WIDTH: f64 = 10.0;

fn calc_arrows(
    mouse_pos: Point<i32>,
    maybe_existing_path: Option<&Vec<Direction>>,
    range_limit: usize,
) -> Vec<(Direction, Arrow)> {
    let directions = calc_movement_path(mouse_pos, maybe_existing_path, range_limit);

    shared::path::path_with_arrows(&directions)
}

fn outcomes_to_animations(outcomes: Vec<Outcome>) -> Vec<Animation> {
    outcomes
        .into_iter()
        .flat_map(Animation::from_outcome)
        .collect::<Vec<Animation>>()
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

fn game_moves_to_page_actions(moves: Vec<game::action::Action>) -> Vec<Action> {
    let mut moves_ret = Vec::new();

    for action in moves {
        match action {
            game::action::Action::Travel {
                unit_id,
                path,
                dismounted_from,
            } => {
                moves_ret.push(Action::TraveledTo {
                    unit_id: unit_id.clone(),
                    path: path.clone(),
                    arrows: path.with_arrows(),
                    dismounted_from: dismounted_from.clone(),
                });
            }
            game::action::Action::LoadInto {
                unit_id,
                load_into,
                path,
            } => moves_ret.push(Action::LoadInto {
                unit_id: unit_id.clone(),
                load_into: load_into.clone(),
                arrows: path.with_arrows(),
                path: path.clone(),
            }),
            game::action::Action::Batch(more_moves) => {
                let mut more_moves_ret = game_moves_to_page_actions(more_moves);
                moves_ret.append(&mut more_moves_ret);
            }
            game::action::Action::PickUp {
                unit_id,
                path,
                cargo_id,
                ..
            } => {
                moves_ret.push(Action::PickUp {
                    unit_id: unit_id.clone(),
                    cargo_id: cargo_id.clone(),
                    path: path.clone(),
                    arrows: path.with_arrows(),
                });
            }
            game::action::Action::DropOff {
                cargo_unit_loc: loc,
                transport_id,
            } => moves_ret.push(Action::DropOff {
                cargo_unit_loc: loc.clone(),
                transport_id: transport_id.clone(),
            }),
            game::action::Action::Replenish {
                replenishing_unit_id,
                units,
                depleted_supply_crates,
                path,
                ..
            } => moves_ret.push(Action::Replenish {
                replenishing_unit_id,
                units: units.clone(),
                depleted_supply_crates,
                path: path.clone(),
                arrows: path.with_arrows(),
            }),
        }
    }

    moves_ret
}

#[cfg(test)]
mod test_movement_arrow {
    use crate::game::{calc_movement_path, Arrow};
    use pretty_assertions::assert_eq;
    use shared::direction::Direction;
    use shared::path::path_with_arrows;
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
