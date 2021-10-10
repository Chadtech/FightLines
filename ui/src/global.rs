use rand::Rng;
use seed::browser::web_storage::{LocalStorage, WebStorage, WebStorageError};
use shared::id::Id;
use shared::rng::{RandGen, RandSeed};

///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    viewer_id: Id,
    random_seed: RandSeed,
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

pub const VIEWER_ID_KEY: &str = "fightlines-viewer-id";

impl Model {
    pub fn init() -> Model {
        let mut rng = rand::thread_rng();
        let seed: RandSeed = rng.gen();

        let mut rand_gen = RandGen::from_seed(seed);

        let viewer_id_result: Result<Id, WebStorageError> = LocalStorage::get(VIEWER_ID_KEY);

        let viewer_id = viewer_id_result.unwrap_or_else(|_err| {
            let id: Id = rand_gen.gen();

            id
        });

        let random_seed: RandSeed = rand_gen.gen();

        Model {
            viewer_id,
            random_seed,
        }
    }

    pub fn viewer_id(&self) -> Id {
        self.viewer_id.clone()
    }
}
