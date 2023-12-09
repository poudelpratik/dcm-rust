pub mod configuration;
mod controllers;
mod routes;

#[allow(dead_code)]
mod shared;

use crate::configuration::{CodeDistributorConfiguration, Configuration};
use crate::routes::index_routes;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() {
    // set env var enable log
    std::env::set_var("RUST_LOG", "info");

    // initialize configuration
    let config = Configuration::default();
    config.init_logger();

    // write code distributor configuration to file
    let code_distributor_configuration: CodeDistributorConfiguration = config.clone().into();
    let code_distributor_configuration_json =
        serde_json::to_string(&code_distributor_configuration).unwrap();

    HttpServer::new(move || {
        let config_data = code_distributor_configuration_json.clone();
        App::new()
            .configure(index_routes::init_routes)
            .service(
                actix_files::Files::new(
                    "/static",
                    std::path::Path::new("src").join("frontend").join("static"),
                )
                .show_files_listing(),
            )
            // Adding a route for configuration.json
            .route(
                "configuration",
                web::get().to(move || serve_config(config_data.clone())),
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

async fn serve_config(config_data: String) -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(config_data)
}
