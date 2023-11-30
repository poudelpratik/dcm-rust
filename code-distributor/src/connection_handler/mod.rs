use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use warp::ws::{Message, WebSocket};
use warp::{ws, Filter};

use crate::client_registry::client_event_listener::ClientEventListener;
use crate::client_registry::ClientRegistry;
use crate::AppData;

pub mod message;

pub mod jwt;

pub(crate) async fn initialize(
    app_data: Arc<AppData>,
    client_registry: Arc<Mutex<ClientRegistry>>,
) {
    let config = app_data.config.clone();
    let host_addr = config.app_host.clone().unwrap_or("0.0.0.0".to_string());
    let server_port = config.app_port.unwrap_or(3030);
    let ws_path = "ws".to_string();
    let api_path = "api".to_string();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Content-Type", "X-Authorization"])
        .allow_methods(vec![
            "GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH", "HEAD",
        ]);

    let app_data = warp::any().map(move || app_data.clone()).boxed();
    let client_registry = warp::any().map(move || client_registry.clone()).boxed();
    let http_routes =
        crate::api::create_routes(app_data.clone(), client_registry.clone(), api_path);

    let ws_routes = warp::path(ws_path)
        .and(client_registry)
        .and(warp::query::<HashMap<String, String>>()) // Add this to extract query parameters
        .and(warp::ws())
        .and_then(
            move |client_registry, query_params: HashMap<String, String>, ws: ws::Ws| async move {
                match query_params.get("auth_token") {
                    Some(auth_token) => {
                        let auth_token = auth_token.to_string();
                        Ok(ws.on_upgrade(move |socket| {
                            handle_client_connection(socket, client_registry, auth_token.clone())
                        }))
                    }
                    None => Err(warp::reject::custom(WarpError)),
                }
            },
        )
        .recover(handle_rejection)
        .boxed();

    warp::serve(http_routes.or(ws_routes).with(cors))
        // .tls()
        // .cert_path(Path::new("cert.pem"))
        // .key_path(Path::new("key.pem"))
        .run((IpAddr::V4(host_addr.parse().unwrap()), server_port))
        .await;
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            "Not Found",
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else {
        Err(err)
    }
}

async fn handle_client_connection(
    ws: WebSocket,
    client_registry: Arc<Mutex<ClientRegistry>>,
    auth_token: String,
) {
    let (mut tx, rx) = ws.split();
    let client_opt = client_registry
        .lock()
        .await
        .get_client_ref_by_token(auth_token)
        .await;

    if let Some(client) = client_opt {
        let tx = Arc::new(Mutex::new(tx));
        client.lock().await.connected(tx.clone()).await;
        let mut client_event_listener = ClientEventListener::new(rx, tx);
        client_event_listener.handle_events().await;
        client.lock().await.disconnected();
    } else {
        tx.send(Message::close()).await.ok();
    }
}

#[derive(Debug)]
pub struct WarpError;

impl warp::reject::Reject for WarpError {}

// #[cfg(test)]
// mod tests {
//     use crate::client_registry::ClientRegistry;
//     use crate::configuration::Configuration;
//     use crate::fragment_registry::FragmentRegistry;
//     use crate::ApplicationContext;
//     use futures_util::{SinkExt, StreamExt};
//     use serde_json::json;
//     use std::sync::Arc;
//     use tokio::sync::Mutex;
//     use warp::ws::Message;
//
//     #[tokio::test]
//     async fn test_websocket_events() {
//         // Initialize the ApplicationContext
//         let config = Arc::new(Configuration::default());
//
//         let path = "ws".to_string();
//         let app_context = Arc::new(Mutex::new(ApplicationContext {
//             config: config.clone(),
//             client_registry: Arc::new(Mutex::new(ClientRegistry::new())),
//             fragment_registry: FragmentRegistry::new(vec![]),
//         }));
//
//         // Initialize the WebSocket server
//         tokio::spawn(async move {
//             super::initialize(app_context).await;
//         });
//
//         // Give server a moment to start
//         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//
//         // Client logic with a timeout
//         let client_duration = tokio::time::Duration::from_secs(20);
//         match tokio::time::timeout(client_duration, async {
//             let ws_url = format!("ws://127.0.0.1:3030/{}", path);
//             // Build a request with the custom header
//             // let request = http::Request::builder()
//             //     .uri(&ws_url)
//             //     .header("user-agent", "MyTestWebSocketClient/1.0")
//             //     .header("origin", "http://localhost:3000")
//             //     .body(())
//             //     .unwrap();
//
//             let (ws, _) = connect_async(ws_url).await.expect("Failed to connect");
//
//             // This is a handle to the connection for sending messages
//             let (mut tx, mut rx) = ws.split();
//
//             // Send a TextMessage event
//             let text_msg = json!({
//                 "event_type": "ExecuteFunction",
//                 "data": {
//                     "id": 8,
//                     "name": "add",
//                     "parameters": [1, 2]
//                 }
//             });
//
//             for _ in 0..2 {
//                 tx.send(Message::text(text_msg.to_string()))
//                     .await
//                     .expect("Failed to send message");
//
//                 if let Some(Ok(msg)) = rx.next().await {
//                     println!("Received response: {}", msg.into_text().unwrap());
//                 }
//             }
//         })
//         .await
//         {
//             Ok(_) => println!("Client finished successfully!"),
//             Err(_) => panic!("Client timed out!"),
//         }
//     }
// }
