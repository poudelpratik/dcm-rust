pub mod endpoints;

use crate::client_registry::client_event_listener::UpdateFragmentData;
use crate::ApplicationContext;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

pub(crate) fn create_routes(
    app_context: Arc<Mutex<ApplicationContext>>,
    api_base_path: String,
) -> BoxedFilter<(impl Reply + Sized,)> {
    let context = warp::any().map(move || app_context.clone()).boxed();

    let base_path = warp::path(api_base_path);

    let get_clients = base_path
        .clone()
        .and(warp::path("clients"))
        .and(warp::get())
        .and(context.clone())
        .and_then(endpoints::get_clients);

    let get_client = base_path
        .clone()
        .and(warp::path("clients"))
        .and(warp::path::param::<Uuid>())
        .and(warp::get())
        .and(context.clone())
        .and_then(endpoints::get_client);

    let update_client = base_path
        .clone()
        .and(warp::path("clients"))
        .and(warp::path::param::<Uuid>()) // Capture the client ID as a path parameter
        .and(warp::put()) // Use PUT method
        .and(warp::body::json::<Vec<UpdateFragmentData>>())
        .and(context.clone())
        .and_then(endpoints::update_client);

    get_client.or(update_client).or(get_clients).boxed()
}
