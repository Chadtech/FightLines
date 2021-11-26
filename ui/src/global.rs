use crate::name::Name;
use rand::distributions::uniform::UniformInt;
use rand::Rng;
use seed::browser::web_storage::{LocalStorage, WebStorage, WebStorageError};
use seed::log;
use shared::id::Id;
use shared::rng::{RandGen, RandSeed};

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    viewer_id: Id,
    viewer_name: Name,
    random_seed: RandSeed,
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

const VIEWER_ID_KEY: &str = "fightlines-viewer-id";

const VIEWER_NAME_KEY: &str = "fightlines-viewer-name";

impl Model {
    pub fn init() -> Model {
        let mut rng = rand::thread_rng();
        let seed: RandSeed = rng.gen();

        let mut rand_gen = RandGen::from_seed(seed);

        let viewer_id = get_viewer_id(&mut rand_gen);
        LocalStorage::insert(VIEWER_ID_KEY, &viewer_id).expect("save viewer id to LocalStorage");

        let viewer_name = get_viewer_name(&mut rand_gen);
        LocalStorage::insert(VIEWER_NAME_KEY, &viewer_name.to_string())
            .expect("save viewer id to LocalStorage");

        let random_seed: RandSeed = RandSeed::next(&mut rand_gen);

        Model {
            viewer_id,
            viewer_name,
            random_seed,
        }
    }

    pub fn viewer_id(&self) -> Id {
        self.viewer_id.clone()
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
