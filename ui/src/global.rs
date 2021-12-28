use crate::view::toast::Toast;
use rand::Rng;
use seed::browser::web_storage::{LocalStorage, WebStorage, WebStorageError};
use seed::log;
use seed::prelude::{cmds, CmdHandle, Orders};
use shared::id::Id;
use shared::name::Name;
use shared::rng::{RandGen, RandSeed};

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    viewer_id: Id,
    pub viewer_name: Name,
    random_seed: RandSeed,
    toasts: Vec<Toast>,
    first_toast_hidden: bool,
    handle_toast_timeout: Option<CmdHandle>,
}

#[derive(Clone)]
pub enum Msg {
    ToastTimeoutExpired,
}

///////////////////////////////////////////////////////////////
// HELPERS //
///////////////////////////////////////////////////////////////

fn wait_to_progress_toasts(orders: &mut impl Orders<Msg>) -> CmdHandle {
    orders.perform_cmd_with_handle(cmds::timeout(8192, || Msg::ToastTimeoutExpired))
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
        LocalStorage::insert(VIEWER_ID_KEY, &viewer_id).expect("save viewer id to LocalStorage");

        let viewer_name = get_viewer_name(&mut rand_gen);
        LocalStorage::insert(VIEWER_NAME_KEY, &viewer_name.to_string())
            .expect("save viewer id to LocalStorage");

        let random_seed: RandSeed = RandSeed::next(&mut rand_gen);

        let mut toasts = Vec::new();

        toasts.push(Toast::from_str(
            "TEST toast that has a decently long message",
        ));

        toasts.push(Toast::from_str(
            "TEST toast that has a decently long message",
        ));

        Model {
            viewer_id,
            viewer_name,
            random_seed,
            toasts,
            first_toast_hidden: false,
            handle_toast_timeout: Some(wait_to_progress_toasts(orders)),
        }
    }

    pub fn viewer_id(&self) -> Id {
        self.viewer_id.clone()
    }

    pub fn toast_from_text(&mut self, text: &str) -> &mut Model {
        self.toasts.push(Toast::from_str(text));
        self
    }

    pub fn toasts(&self) -> &Vec<Toast> {
        &self.toasts
    }

    pub fn first_toast_hidden(&self) -> bool {
        self.first_toast_hidden
    }
}

fn get_viewer_name(rand_gen: &mut RandGen) -> Name {
    let viewer_name_result: Result<Name, WebStorageError> =
        LocalStorage::get(VIEWER_NAME_KEY).map(Name::from_string);

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
        Msg::ToastTimeoutExpired => match global.toasts.split_first() {
            None => {
                log!("???? Somhoew");
                global.first_toast_hidden = false;
                global.handle_toast_timeout = None;
            }
            Some((_, rest)) => {
                log!("Timeout expired!");
                if global.first_toast_hidden {
                    global.toasts = rest.to_vec();
                } else {
                    global.first_toast_hidden = true;
                }

                global.handle_toast_timeout = Some(wait_to_progress_toasts(orders));
            }
        },
    }
}
