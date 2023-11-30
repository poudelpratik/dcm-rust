use std::sync::Arc;

use http::HeaderMap;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use log::info;
use serde_derive::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{Rejection, Reply};

use crate::client_registry::client::Client;
use crate::client_registry::client_event_listener::UpdateFragmentData;
use crate::client_registry::ClientRegistry;
use crate::connection_handler::jwt::Claims;
use crate::connection_handler::WarpError;
use crate::fragment_registry::FragmentRegistry;
use crate::ApplicationContext;

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
    let clients = client_registry.get_client_by_id(id).await;
    Ok(warp::reply::json(&clients))
}

/// This function updates a client by its id.
pub(crate) async fn update_client(
    id: Uuid,
    update_fragment_data: Vec<UpdateFragmentData>,
    app_context: Arc<Mutex<ApplicationContext>>,
) -> Result<impl Reply, Rejection> {
    let client_registry = app_context.lock().await.client_registry.clone();
    let client_registry = client_registry.lock().await;
    info!("Updating fragment information for client: {}", id);
    client_registry
        .update_client_fragments(id, update_fragment_data)
        .await
        .ok();
    Ok(warp::reply::json(&()))
}

/// For client authentication
pub(crate) async fn authenticate(
    headers: http::HeaderMap,
    app_context: Arc<Mutex<ApplicationContext>>,
) -> Result<impl Reply, Rejection> {
    let client_registry = app_context.lock().await.client_registry.clone();
    let fragment_registry = app_context.lock().await.fragment_registry.clone();

    let existing_token = headers
        .get("X-Authorization")
        .and_then(|hv| hv.to_str().ok())
        .map(|token| token.trim_start_matches("Bearer "));

    match existing_token {
        Some(token) => {
            // Validate the token
            let validation = Validation::default();
            let token_data = match decode::<Claims>(
                token,
                &DecodingKey::from_secret("secret".as_ref()),
                &validation,
            ) {
                Ok(data) => data,
                Err(_) => return Err(warp::reject::custom(WarpError)),
            };

            let client_id = token_data.claims.uuid;
            // let client_registry = client_registry.lock().await;
            if client_registry
                .lock()
                .await
                .get_client_by_id(Uuid::parse_str(client_id.as_str()).unwrap_or_default())
                .await
                .is_some()
            {
                Ok(warp::reply::json(&AuthResponse::new(
                    client_id,
                    token.to_string(),
                )))
            } else {
                let (uuid, auth_token) =
                    create_client(headers, fragment_registry, client_registry).await;
                Ok(warp::reply::json(&AuthResponse::new(
                    uuid.to_string(),
                    auth_token,
                )))
            }
        }
        None => {
            // Register new client and issue a new token
            let (uuid, auth_token) =
                create_client(headers, fragment_registry, client_registry).await;
            Ok(warp::reply::json(&AuthResponse::new(
                uuid.to_string(),
                auth_token,
            )))
        }
    }
}

async fn create_client(
    headers: HeaderMap,
    fragment_registry: FragmentRegistry,
    client_registry: Arc<Mutex<ClientRegistry>>,
) -> (Uuid, String) {
    let user_agent = headers
        .get("User-Agent")
        .map(|hv| hv.to_str().unwrap_or_default().to_string())
        .unwrap_or_default();
    let ip_address = headers
        .get("Origin")
        .map(|hv| hv.to_str().unwrap_or_default().to_string())
        .unwrap_or_default();

    let jwt_key = "secret";
    let uuid = Uuid::new_v4();
    let claims = Claims::new(uuid.to_string(), user_agent, ip_address, 10000000000);
    let auth_token = encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_key.as_ref()),
    )
    .unwrap_or_default();

    let fragment_registry = Arc::new(Mutex::new(fragment_registry));
    let mut client_registry = client_registry.lock().await;
    let client = Client::new(uuid, fragment_registry, auth_token.clone(), None);
    let client = Arc::new(Mutex::new(client));
    client_registry.register(uuid, client);
    (uuid, auth_token)
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub client_id: String,
    pub token: String,
}

impl AuthResponse {
    pub fn new(client_id: String, auth_token: String) -> Self {
        Self {
            client_id,
            token: auth_token,
        }
    }
}
