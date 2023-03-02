use crate::error::Error;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::{Card, Header};
use crate::view::cell::{Cell, Row};
use crate::view::textarea::Textarea;
use crate::view::toast;
use crate::view::toast::{OpenToast, Toast};
use rand::Rng;
use seed::browser::web_storage::{LocalStorage, WebStorage, WebStorageError};
use seed::prelude::{cmds, streams, CmdHandle, Ev, JsValue, Orders, StreamHandle};
use shared::id::Id;
use shared::name::Name;
use shared::rng::{RandGen, RandSeed};
use std::str::FromStr;

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    random_seed: RandSeed,

    // Viewer
    viewer_id: Id,
    pub viewer_name: Name,

    // toasts
    toasts: Vec<Toast>,
    open_toast: Option<OpenToast>,
    first_toast_hidden: bool,

    // window
    window_width: f64,
    window_height: f64,

    // timeouts
    handle_hide_toast_timeout: Option<CmdHandle>,
    handle_remove_toast_timeout: Option<CmdHandle>,
    handle_toast_check_timeout: CmdHandle,
    handle_localstorage_save_timeout: CmdHandle,

    // Streams
    #[allow(dead_code)]
    window_resize_stream: StreamHandle,
}

pub struct WindowSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone)]
pub enum Msg {
    HideToastTimeoutExpired,
    RemoveToastTimeoutExpired,
    CheckToastTimeoutExpired,
    SaveLocalStorageTimeoutExpired,
    ClickedGoBack,
    WindowResized,
}

///////////////////////////////////////////////////////////////
// HELPERS //
///////////////////////////////////////////////////////////////

fn wait_to_save_localstorage(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(1024, || Msg::SaveLocalStorageTimeoutExpired))
}

fn wait_to_check_toasts(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(1024, || Msg::CheckToastTimeoutExpired))
}

fn wait_to_progress_toasts(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(8192, || Msg::HideToastTimeoutExpired))
}

fn wait_to_remove_toast(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(128, || Msg::RemoveToastTimeoutExpired))
}

fn save_viewer_name(name: &Name) {
    LocalStorage::insert(VIEWER_NAME_KEY, &name.to_string())
        .expect("save viewer name to LocalStorage");
}

fn save_viewer_id(viewer_id: &Id) {
    LocalStorage::insert(VIEWER_ID_KEY, &viewer_id).expect("save viewer id to LocalStorage");
}

fn get_window_size() -> Result<(f64, f64), String> {
    let inner_width_json: JsValue = seed::window().inner_width().map_err(|err| {
        err.as_string()
            .unwrap_or_else(|| "cannot unwrap inner width error".to_string())
    })?;
    let inner_height_json: JsValue = seed::window().inner_height().map_err(|err| {
        err.as_string()
            .unwrap_or_else(|| "cannot unwrap inner height error".to_string())
    })?;

    let inner_width = inner_width_json.as_f64().ok_or("no inner width")?;
    let inner_height = inner_height_json.as_f64().ok_or("no inner height")?;

    Ok((inner_width, inner_height))
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

const VIEWER_ID_KEY: &str = "fightlines-viewer-id";

const VIEWER_NAME_KEY: &str = "fightlines-viewer-name";

impl Model {
    pub fn init(orders: &mut impl Orders<Msg>) -> Result<Model, String> {
        let (inner_width, inner_height) = get_window_size()?;

        let window_resize_stream =
            orders.stream_with_handle(streams::window_event(Ev::Resize, |_| Msg::WindowResized));

        let mut rng = rand::thread_rng();
        let seed: RandSeed = rng.gen();

        let mut rand_gen = RandGen::from_seed(seed);

        let viewer_id = get_viewer_id(&mut rand_gen);
        save_viewer_id(&viewer_id);

        let viewer_name = get_viewer_name(&mut rand_gen);
        save_viewer_name(&viewer_name);

        let random_seed: RandSeed = RandSeed::next(&mut rand_gen);

        let model = Model {
            viewer_id,
            viewer_name,
            random_seed,
            toasts: Vec::new(),
            open_toast: None,
            first_toast_hidden: false,
            handle_hide_toast_timeout: None,
            handle_remove_toast_timeout: None,
            handle_toast_check_timeout: wait_to_check_toasts(orders),
            handle_localstorage_save_timeout: wait_to_save_localstorage(orders),
            window_width: inner_width,
            window_height: inner_height,
            window_resize_stream,
        };

        Ok(model)
    }

    pub fn new_rand_gen(&mut self) -> RandGen {
        let mut rand_gen = RandGen::from_seed(self.random_seed.clone());
        let new_seed = RandSeed::next(&mut rand_gen);

        self.random_seed = new_seed;

        rand_gen
    }

    pub fn viewer_id(&self) -> Id {
        self.viewer_id.clone()
    }

    pub fn toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn toast_error(&mut self, error: Error) {
        self.toast(Toast::from_error(error));
    }

    pub fn toasts(&self) -> &Vec<Toast> {
        &self.toasts
    }

    pub fn first_toast_hidden(&self) -> bool {
        self.first_toast_hidden
    }

    pub fn set_viewer_name(&mut self, name: Name) {
        self.viewer_name = name;
    }

    pub fn window_size(&self) -> WindowSize {
        WindowSize {
            width: self.window_width,
            height: self.window_height,
        }
    }
}

fn get_viewer_name(rand_gen: &mut RandGen) -> Name {
    let viewer_name_result: Result<Name, WebStorageError> = LocalStorage::get(VIEWER_NAME_KEY)
        .and_then(|name_str: String| {
            match Name::from_str(name_str.as_str()) {
                Ok(name) => Ok(name),
                Err(_) => {
                    // This shouldnt be possible
                    Err(WebStorageError::StorageNotFoundError)
                }
            }
        });

    match viewer_name_result {
        Ok(name) => name,
        Err(_) => {
            let new_viewer_name: Name = Name::random(rand_gen);

            new_viewer_name
        }
    }
}

fn get_viewer_id(rand_gen: &mut RandGen) -> Id {
    let viewer_id_result: Result<Id, WebStorageError> = LocalStorage::get(VIEWER_ID_KEY);

    let new_viewer_id: Id = Id::new(rand_gen);

    viewer_id_result.unwrap_or(new_viewer_id)
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

pub fn update(msg: Msg, global: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::HideToastTimeoutExpired => {
            if global.toasts.is_empty() {
                global.first_toast_hidden = false;
                global.handle_hide_toast_timeout = None;
                global.handle_remove_toast_timeout = None;
                global.handle_toast_check_timeout = wait_to_check_toasts(orders);
            } else {
                global.first_toast_hidden = true;

                global.handle_hide_toast_timeout = Some(wait_to_progress_toasts(orders));
                global.handle_remove_toast_timeout = Some(wait_to_remove_toast(orders));
            }
        }
        Msg::RemoveToastTimeoutExpired => {
            global.first_toast_hidden = false;

            if let Some((_, rest)) = global.toasts.split_first() {
                global.toasts = rest.to_vec();
            }
        }
        Msg::ClickedGoBack => {
            global.open_toast = None;
        }
        Msg::CheckToastTimeoutExpired => {
            if global.toasts.is_empty() {
                global.handle_toast_check_timeout = wait_to_check_toasts(orders);
            } else {
                global.handle_hide_toast_timeout = Some(wait_to_progress_toasts(orders));
            }
        }
        Msg::SaveLocalStorageTimeoutExpired => {
            save_viewer_id(&global.viewer_id);
            save_viewer_name(&global.viewer_name);
            global.handle_localstorage_save_timeout = wait_to_save_localstorage(orders);
        }
        Msg::WindowResized => {
            if let Ok((inner_width, inner_height)) = get_window_size() {
                global.window_width = inner_width;
                global.window_height = inner_height;
            }
        }
    }
}

pub fn update_from_toast_msg(msg: toast::Msg, global: &mut Model) {
    match msg {
        toast::Msg::ClickedOpenToast(toast_index) => {
            global.open_toast = global
                .toasts
                .get(toast_index)
                .and_then(|toast| toast.to_open_toast());
        }
        toast::Msg::ClickedClose(toast_index) => {
            global.toasts.remove(toast_index);
        }
    }
}

pub fn open_toast_view(global: &Model) -> Option<Cell<Msg>> {
    global.open_toast.as_ref().map(|open_toast| {
        let header = Header::from_title(open_toast.title.as_str());

        let text_cell = Cell::from_str(vec![], open_toast.text.as_str());

        let card = Card::init()
            .with_header(header)
            .with_body_styles(vec![Style::G4])
            .cell(
                vec![Style::AbsoluteCenter],
                vec![
                    Row::from_cells(vec![], vec![text_cell]),
                    Row::from_cells(
                        vec![],
                        vec![Textarea::simple(open_toast.info.clone()).cell(vec![Style::WFull])],
                    ),
                    Row::from_cells(
                        vec![],
                        vec![Button::primary("close")
                            .on_click(|_| Msg::ClickedGoBack)
                            .cell()],
                    ),
                ],
            );

        Cell::group(vec![Style::WFull], vec![card])
    })
}
