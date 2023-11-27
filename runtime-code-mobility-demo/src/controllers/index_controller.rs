use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../frontend/index.html"))
}

#[get("/playground")]
pub async fn playground() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../frontend/playground.html"))
}
