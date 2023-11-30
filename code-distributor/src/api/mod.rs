use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

use crate::client_registry::client_event_listener::UpdateFragmentData;
use crate::client_registry::ClientRegistry;
use crate::AppData;

pub mod endpoints;

pub(crate) fn create_routes(
    app_data: BoxedFilter<(Arc<AppData>,)>,
    client_registry: BoxedFilter<(Arc<Mutex<ClientRegistry>>,)>,
    api_base_path: String,
) -> BoxedFilter<(impl Reply + Sized,)> {
    let base_path = warp::path(api_base_path);

    let get_clients = base_path
        .clone()
        .and(warp::path("clients"))
        .and(warp::get())
        .and(client_registry.clone())
        .and_then(endpoints::get_clients);

    let get_client = base_path
        .clone()
        .and(warp::path("clients"))
        .and(warp::path::param::<Uuid>())
        .and(warp::get())
        .and(client_registry.clone())
        .and_then(endpoints::get_client);

    let update_client = base_path
        .clone()
        .and(warp::path("clients"))
        .and(warp::path::param::<Uuid>()) // Capture the client ID as a path parameter
        .and(warp::put()) // Use PUT method
        .and(warp::body::json::<Vec<UpdateFragmentData>>())
        .and(client_registry.clone())
        .and_then(endpoints::update_client);

    let authenticate = base_path
        .clone()
        .and(warp::path("auth"))
        .and(warp::post()) // Use POST method
        .and(warp::header::headers_cloned())
        .and(app_data.clone())
        .and(client_registry.clone())
        .and_then(endpoints::authenticate);

    get_client
        .or(update_client)
        .or(get_clients)
        .or(authenticate)
        .boxed()
}
