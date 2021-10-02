mod dev;
mod flags;

use crate::flags::Flags;
use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use notify::{raw_watcher, RecursiveMode, Watcher};
use std::fs;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;
////////////////////////////////////////////////////////////////////////////////
// TYPES //
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct Model {
    pub ip_address: String,
    pub admin_password: String,
    pub port_number: u64,
    pub setting: Setting,
}

impl Model {
    fn init() -> Result<Model, String> {
        let flags = Flags::get()?;

        let setting: Setting = if flags.dev_mode {
            Setting::Dev(DevModel)
        } else {
            Setting::Prod(ProdModel)
        };

        Ok(Model {
            ip_address: flags.ip_address,
            admin_password: flags.admin_password,
            port_number: flags.port_number,
            setting,
        })
    }
}

#[derive(Clone)]
enum Setting {
    Prod(ProdModel),
    Dev(DevModel),
}
impl Setting {
    pub fn is_dev(&self) -> bool {
        match self {
            Setting::Dev(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
struct DevModel;

#[derive(Clone)]
struct ProdModel;

#[actix_web::main]
async fn main() -> Result<(), String> {
    let model = Model::init()?;

    let setting = model.setting.clone();

    if setting.is_dev() {
        thread::spawn(move || {
            watch_and_recompile_ui(&setting);
        });
    };

    let web_model = actix_web::web::Data::new(model);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web_model.clone())
            .route("/package.js", web::get().to(js_asset_route))
            .route("/package_bg.wasm", web::get().to(wasm_asset_route))
            .default_service(web::get().to(frontend))
    })
    .bind("127.0.0.1:8080")
    .map_err(|err| err.to_string())?
    .run()
    .await
    .map_err(|err| err.to_string())
}

async fn wasm_asset_route(model: web::Data<Model>) -> HttpResponse {
    if model.setting.is_dev() {
        match fs::read("./src/assets/package_bg.wasm") {
            Ok(bytes) => HttpResponse::Ok()
                .header("Content-Type", "application/wasm")
                .body(bytes),
            Err(_) => HttpResponse::InternalServerError().body("Could not find wasm file"),
        }
    } else {
        let bytes: &'static [u8] = include_bytes!("assets/package_bg.wasm");

        HttpResponse::Ok()
            .header("Content-Type", "application/wasm")
            .body(bytes)
    }
}

async fn js_asset_route(model: web::Data<Model>) -> HttpResponse {
    if model.setting.is_dev() {
        match fs::read("./src/assets/package.js") {
            Ok(bytes) => HttpResponse::Ok()
                .header("Content-Type", "text/javascript")
                .body(bytes),
            Err(_) => HttpResponse::InternalServerError().body("Could not find js file"),
        }
    } else {
        HttpResponse::Ok()
            .header("Content-Type", "text/javascript")
            .body(include_str!("assets/package.js"))
    }
}

async fn frontend() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("assets/index.html"))
}

// async fn wasm_asset_route(modelka: web::Data<Modelka>) -> HttpResponse {
//     match &modelka.get_ref().okoli {
//         Okoli::Dev(_) => match read_elm_file() {
//             Ok(elm_file) => HttpResponse::Ok().body(elm_file),
//             Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
//         },
//         Okoli::Prod(prod_model) => match &prod_model.elm_file {
//             Ok(file_str) => HttpResponse::Ok().body(file_str),
//             Err(_) => HttpResponse::InternalServerError().body("elm file was missing"),
//         },
//     }
// }

////////////////////////////////////////////////////////////////////////////////
// DEV //
////////////////////////////////////////////////////////////////////////////////

fn watch_and_recompile_ui(setting: &Setting) {
    let (sender, receiver) = channel();

    let mut watcher = raw_watcher(sender).unwrap();

    watcher
        .watch("../ui/src", RecursiveMode::Recursive)
        .unwrap();

    loop {
        let result: Result<(), String> = match receiver.recv() {
            Ok(event) => match event.path {
                None => Ok(()),
                Some(filepath) => {
                    let file_extension = filepath.extension().and_then(|ext| ext.to_str());

                    match file_extension {
                        Some("rs") => {
                            dev::log("Building frontend..");

                            let build_result = Command::new("cargo")
                                .current_dir("../ui")
                                .args(&["make", "build"])
                                .output();

                            match build_result {
                                Ok(output) => {
                                    if output.status.success() {
                                        dev::succeed("Done");
                                        Ok(())
                                    } else {
                                        let mut buf =
                                            "failed to compiled frontend with status code : "
                                                .to_string();

                                        buf.push_str(output.status.to_string().as_str());

                                        Err(buf)
                                    }
                                }
                                Err(err) => Err(err.to_string()),
                            }
                        }
                        _ => Ok(()),
                    }
                }
            },
            Err(err) => Err(err.to_string()),
        };

        if let Err(err) = result {
            panic!("{}", err);
        };
    }
}
