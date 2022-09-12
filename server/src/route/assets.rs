use actix_web::{web, HttpResponse};
use shared::api::endpoint::Endpoint;
use shared::facing_direction::FacingDirection;
use shared::frame_count::FrameCount;
use shared::sprite::Sprite;

pub fn routes() -> actix_web::Scope {
    web::scope("/asset")
        .route("/sheet.png", web::get().to(sheet_route))
        .route("/sheet-flipped.png", web::get().to(sheet_flipped_route))
}

async fn sheet_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/sheet.png")).await
}

async fn sheet_flipped_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/sheet-flipped.png")).await
}

async fn sprite_route(bytes: &'static [u8]) -> HttpResponse {
    HttpResponse::Ok()
        .header("Content-Type", "image/png")
        .body(bytes)
}
