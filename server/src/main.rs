use std::fs;
use std::fs::ReadDir;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer};
use image::imageops;
use image::io::Reader as ImageReader;
use notify::{raw_watcher, RecursiveMode, Watcher};

use route::game;
use route::lobby;
use shared::api::endpoint;

use crate::flags::Flags;
use crate::model::Model;
use shared::api::endpoint::Endpoint;

mod dev;
mod flags;
mod games;
mod lobbies;
mod model;
mod route;
mod setting;

#[actix_web::main]
async fn main() -> Result<(), String> {
    let flags = Flags::get()?;

    match flags {
        Flags::Main(main_flags) => {
            let model = Model::init(main_flags);

            let setting = model.setting.clone();

            if setting.is_dev() {
                thread::spawn(move || {
                    if let Err(error) = build_frontend() {
                        panic!("Failed to build frontend: {}", error);
                    };
                    watch_and_recompile_ui();
                });
            };

            let web_model = actix_web::web::Data::new(model);

            HttpServer::new(move || {
                let cors = Cors::permissive();

                let app_serving_frontend_code = App::new()
                    .wrap(cors)
                    .app_data(web_model.clone())
                    .route("/package.js", web::get().to(js_asset_route))
                    .route("/package_bg.wasm", web::get().to(wasm_asset_route));

                app_serving_frontend_code
                    .service(route::assets::routes())
                    .service(
                        web::scope(endpoint::ROOT)
                            .route(
                                Endpoint::CreateLobby.to_string().as_str(),
                                web::post().to(lobby::create::handle),
                            )
                            .route(
                                Endpoint::template_get_lobby().to_string().as_str(),
                                web::get().to(lobby::get::handle),
                            )
                            .route(
                                Endpoint::template_join_lobby().to_string().as_str(),
                                web::post().to(lobby::join::handle),
                            )
                            .route(
                                Endpoint::update_lobby().to_string().as_str(),
                                web::post().to(lobby::update::handle),
                            )
                            .route(
                                Endpoint::StartGame.to_string().as_str(),
                                web::post().to(lobby::start::handle),
                            )
                            .route(
                                Endpoint::template_get_game().to_string().as_str(),
                                web::get().to(game::get::handle),
                            )
                            .route(
                                Endpoint::template_submit_turn().to_string().as_str(),
                                web::post().to(game::submit_turn::handle),
                            ),
                    )
                    .default_service(web::get().to(frontend))
            })
            .bind("127.0.0.1:8080")
            .map_err(|err| err.to_string())?
            .run()
            .await
            .map_err(|err| err.to_string())
        }
        Flags::Sprites => {
            flip_sprite_sheet()?;
            darken_units()?;
            move_sprites()?;

            Ok(())
        }
    }
}

fn flip_sprite_sheet() -> Result<(), String> {
    let sheet = ImageReader::open("./server/src/assets/sheet.png")
        .map_err(|err| err.to_string())?
        .decode()
        .map_err(|err| err.to_string())?;

    let flipped_sheet = imageops::flip_horizontal(&sheet);

    flipped_sheet
        .save("./server/src/assets/sheet-flipped.png")
        .map_err(|err| err.to_string())?;

    Ok(())
}

fn move_sprites() -> Result<(), String> {
    for path_result in read_sprite_dir() {
        let path = path_result.unwrap().path();

        let ext = path.extension().map(|ext| ext.to_str().unwrap());

        if ext == Some("png") {
            let img = ImageReader::open(path.to_str().unwrap())
                .map_err(|err| err.to_string())?
                .decode()
                .map_err(|err| err.to_string())?;

            let file_name_os_str = path.file_name().unwrap();

            let file_name = file_name_os_str.to_str().unwrap();

            let mut dest_file_path = "./server/src/assets/".to_string();
            dest_file_path.push_str(file_name);

            img.save(dest_file_path).map_err(|err| err.to_string())?;
        }
    }

    Ok(())
}

fn darken_units() -> Result<(), String> {
    for path_result in fs::read_dir("./shared/src/sprites/units").unwrap() {
        let path = path_result.unwrap().path();

        let ext = path.extension().unwrap().to_str().unwrap();

        let path_str = path.to_str().unwrap();
        let is_flip = path_str.ends_with("-l.png");
        let is_darkened = path_str.ends_with("_moved.png");

        if ext == "png" && !is_flip && !is_darkened {
            let img = ImageReader::open(path.to_str().unwrap())
                .map_err(|err| err.to_string())?
                .decode()
                .map_err(|err| err.to_string())?;

            let path_no_extension = path.with_extension("");

            let name = path_no_extension.to_str().unwrap();

            let darkened_img = imageops::brighten(&img, -64);

            let mut save_name = name.to_string();
            save_name.push_str("_moved.png");

            darkened_img
                .save(save_name)
                .map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

fn read_sprite_dir() -> ReadDir {
    fs::read_dir("./shared/src/sprites").unwrap()
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

////////////////////////////////////////////////////////////////////////////////
// DEV //
////////////////////////////////////////////////////////////////////////////////

fn build_frontend() -> Result<(), String> {
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
                match output.status.code() {
                    Some(1) => match std::str::from_utf8(output.stderr.as_slice()) {
                        Ok(str) => {
                            let mut buf = "\n".to_string();
                            buf.push_str(str);

                            dev::log(buf.as_str());
                            Ok(())
                        }
                        Err(error) => Err(error.to_string()),
                    },
                    _ => {
                        let mut buf = "failed to compiled frontend with status code : ".to_string();

                        buf.push_str(output.status.to_string().as_str());

                        Err(buf)
                    }
                }
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

fn watch_and_recompile_ui() {
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
                        Some("rs") => build_frontend(),
                        Some("css") => build_frontend(),
                        _ => Ok(()),
                    }
                }
            },
            Err(err) => Err(err.to_string()),
        };

        if let Err(err) = result {
            dev::error(err.as_str());
        };
    }
}
