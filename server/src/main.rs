mod flags;

use crate::flags::Flags;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

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

#[derive(Clone)]
enum Setting {
    Prod(ProdModelka),
    Dev(DevModelka),
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let flags = Flags::get()?;

    HttpServer::new(|| {
        App::new()
            .route("/package.js", web::get().to(js_asset_route))
            .route("/package_bg.wasm", web::get().to(wasm_asset_route))
            .default_service(web::get().to(frontend))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn wasm_asset_route() -> HttpResponse {
    let bytes: &'static [u8] = include_bytes!("assets/package_bg.wasm");
    HttpResponse::Ok().body(bytes)
}

async fn js_asset_route() -> HttpResponse {
    HttpResponse::Ok()
        .header("Content-Type", "text/javascript")
        .body(include_str!("assets/package.js"))
}

async fn frontend() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("assets/index.html"))
}
