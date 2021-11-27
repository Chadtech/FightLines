use std::fs;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

use actix::prelude::*;

use actix;
use actix::Actor;
use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use notify::{raw_watcher, RecursiveMode, Watcher};

use route::lobby;
use shared::api::endpoint;

use crate::model::Model;
use actix::StreamHandler;
use actix_web::rt::time::Instant;
use std::time::Duration;

mod dev;
mod flags;
mod lobbies;
mod model;
mod route;
mod setting;

#[actix_web::main]
async fn main() -> Result<(), String> {
    let model = Model::init()?;

    let setting = model.setting.clone();

    if setting.is_dev() {
        thread::spawn(move || {
            build_frontend().unwrap();
            watch_and_recompile_ui();
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
            .route("/ws/", web::get().to(handle_websocket))
            .service(
                web::scope(endpoint::ROOT).service(
                    web::scope(endpoint::LOBBY)
                        .route("/create", web::post().to(lobby::create::handle))
                        .route("/get/{id}", web::get().to(lobby::get::handle))
                        .route("/join/{id}", web::post().to(lobby::join::handle)),
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

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// websocket connection is long running connection, it easier
/// to handle with an actor
struct MyWebSocket {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        println!("WS: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl MyWebSocket {
    fn new() -> Self {
        Self { hb: Instant::now() }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

async fn handle_websocket(
    r: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    println!("{:?}", r);
    let res = ws::start(MyWebSocket::new(), &r, stream);
    println!("{:?}", res);
    res
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
