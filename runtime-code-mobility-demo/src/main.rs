pub mod configuration;
mod controllers;
mod routes;

#[allow(dead_code)]
mod shared;

use crate::configuration::Configuration;
use crate::routes::index_routes;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() {
    // set env var enable log
    std::env::set_var("RUST_LOG", "info");

    // initialize configuration
    let config = Configuration::default();
    config.init_logger();

    HttpServer::new(move || {
        App::new()
            .configure(index_routes::init_routes)
            .service(
                actix_files::Files::new(
                    "/static",
                    std::path::Path::new("src")
                        .join("frontend")
                        .join("static"),
                )
                .show_files_listing(),
            )
    })
    .bind((
        config.app_host.unwrap_or("0.0.0.0".to_string()),
        config.app_port.unwrap_or(8081),
    ))
    .expect("Failed to start web server")
    .run()
    .await
    .expect("Failed to start web server");
}
