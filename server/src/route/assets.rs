use actix_web::{web, HttpResponse};

pub fn routes() -> actix_web::Scope {
    web::scope("/asset")
        .route("/sheet.png", web::get().to(sheet_route))
        .route("/sheet-flipped.png", web::get().to(sheet_flipped_route))
        .route("/infantry-red.png", web::get().to(infantry_red_route))
        .route("/tank-red.png", web::get().to(tank_red_route))
        .route("/truck-red.png", web::get().to(truck_red_route))
        .route(
            "/supply-crate-red.png",
            web::get().to(supply_crate_red_route),
        )
        .route("/infantry-blue.png", web::get().to(infantry_blue_route))
        .route("/tank-blue.png", web::get().to(tank_blue_route))
        .route("/truck-blue.png", web::get().to(truck_blue_route))
        .route(
            "/supply-crate-blue.png",
            web::get().to(supply_crate_blue_route),
        )
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

async fn infantry_red_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/infantry_red1.png")).await
}

async fn tank_red_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/tank_red1.png")).await
}

async fn truck_red_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/truck_red1.png")).await
}

async fn supply_crate_red_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/supply_crate_red1.png")).await
}

async fn infantry_blue_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/infantry_blue1.png")).await
}

async fn tank_blue_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/tank_blue1.png")).await
}

async fn truck_blue_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/truck_blue1.png")).await
}

async fn supply_crate_blue_route() -> HttpResponse {
    sprite_route(include_bytes!("../assets/supply_crate_blue1.png")).await
}
