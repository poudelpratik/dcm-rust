use actix_web::web;

use crate::controllers::index_controller;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index_controller::index);
}
