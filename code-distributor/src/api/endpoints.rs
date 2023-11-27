use crate::client_registry::client_event_listener::UpdateFragmentData;
use crate::ApplicationContext;
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{Rejection, Reply};

/// This function returns a list of all the clients connected to the server.
pub(crate) async fn get_clients(
    app_context: Arc<Mutex<ApplicationContext>>,
) -> Result<impl Reply, Rejection> {
    let client_registry = app_context.lock().await.client_registry.clone();
    let client_registry = client_registry.lock().await;
    info!("Getting all clients");
    let clients = client_registry.get_clients().await;
    Ok(warp::reply::json(&clients))
}

/// This function gets a client by its id.
pub(crate) async fn get_client(
    id: Uuid,
    app_context: Arc<Mutex<ApplicationContext>>,
) -> Result<impl Reply, Rejection> {
    let client_registry = app_context.lock().await.client_registry.clone();
    let client_registry = client_registry.lock().await;
    info!("Getting client information of client: {}", id);
    let clients = client_registry.get_client(id).await;
    Ok(warp::reply::json(&clients))
}

/// This function returns a list of all the clients connected to the server.
pub(crate) async fn update_client(
    id: Uuid,
    update_fragment_data: Vec<UpdateFragmentData>,
    app_context: Arc<Mutex<ApplicationContext>>,
) -> Result<impl Reply, Rejection> {
    let client_registry = app_context.lock().await.client_registry.clone();
    let client_registry = client_registry.lock().await;
    info!("Updating client information for client: {}", id);
    client_registry
        .update_client_fragments(id, update_fragment_data)
        .await
        .ok();
    Ok(warp::reply::json(&()))
}
