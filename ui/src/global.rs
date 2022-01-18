use crate::assets::Assets;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::{Card, Header};
use crate::view::cell::{Cell, Row};
use crate::view::textarea::Textarea;
use crate::view::toast;
use crate::view::toast::{OpenToast, Toast};
use rand::Rng;
use seed::browser::web_storage::{LocalStorage, WebStorage, WebStorageError};
use seed::prelude::{cmds, CmdHandle, Orders};
use shared::id::Id;
use shared::name::Name;
use shared::rng::{RandGen, RandSeed};

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

    // game assets
    assets: Assets,

    // timeouts
    handle_hide_toast_timeout: Option<CmdHandle>,
    handle_remove_toast_timeout: Option<CmdHandle>,
    handle_toast_check_timeout: CmdHandle,
    handle_localstorage_save_timeout: CmdHandle,
}

#[derive(Clone)]
pub enum Msg {
    HideToastTimeoutExpired,
    RemoveToastTimeoutExpired,
    CheckToastTimeoutExpired,
    SaveLocalStorageTimeoutExpired,
    ClickedGoBack,
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

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

const VIEWER_ID_KEY: &str = "fightlines-viewer-id";

const VIEWER_NAME_KEY: &str = "fightlines-viewer-name";

impl Model {
    pub fn init(orders: &mut impl Orders<Msg>) -> Model {
        let mut rng = rand::thread_rng();
        let seed: RandSeed = rng.gen();

        let mut rand_gen = RandGen::from_seed(seed);

        let viewer_id = get_viewer_id(&mut rand_gen);
        save_viewer_id(&viewer_id);

        let viewer_name = get_viewer_name(&mut rand_gen);
        save_viewer_name(&viewer_name);

        let random_seed: RandSeed = RandSeed::next(&mut rand_gen);

        Model {
            viewer_id,
            viewer_name,
            random_seed,
            assets: Assets::init(),
            toasts: Vec::new(),
            open_toast: None,
            first_toast_hidden: false,
            handle_hide_toast_timeout: None,
            handle_remove_toast_timeout: None,
            handle_toast_check_timeout: wait_to_check_toasts(orders),
            handle_localstorage_save_timeout: wait_to_save_localstorage(orders),
        }
    }

    pub fn viewer_id(&self) -> Id {
        self.viewer_id.clone()
    }

    pub fn toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
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
}

fn get_viewer_name(rand_gen: &mut RandGen) -> Name {
    let viewer_name_result: Result<Name, WebStorageError> = LocalStorage::get(VIEWER_NAME_KEY)
        .map(Name::from_string)
        .and_then(|maybe_name| {
            match maybe_name {
                None => {
                    // This shouldnt be possible
                    Err(WebStorageError::StorageNotFoundError)
                }
                Some(name) => Ok(name),
            }
        });

    let new_viewer_name: Name = Name::random(rand_gen);

    viewer_name_result.unwrap_or_else(|_err| new_viewer_name)
}

fn get_viewer_id(rand_gen: &mut RandGen) -> Id {
    let viewer_id_result: Result<Id, WebStorageError> = LocalStorage::get(VIEWER_ID_KEY);

    let new_viewer_id: Id = Id::new(rand_gen);

    viewer_id_result.unwrap_or_else(|_err| new_viewer_id)
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
                vec![Style::Absolute, Style::AbsoluteCenter],
                vec![
                    Row::from_cells(vec![], vec![text_cell]),
                    Row::from_cells(
                        vec![],
                        vec![Textarea::simple(open_toast.text.clone()).cell(vec![Style::WFull])],
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
