pub mod message;

use crate::client_registry::client::Client;
use crate::client_registry::client_event_listener::ClientEventListener;
use crate::ApplicationContext;
use futures_util::StreamExt;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::ws::WebSocket;
use warp::{ws, Filter};

pub(crate) async fn initialize(app_context: Arc<Mutex<ApplicationContext>>) {
    let config = app_context.lock().await.config.clone();
    let host_addr = config.app_host.clone().unwrap_or("0.0.0.0".to_string());
    let server_port = config.app_port.unwrap_or(3030);
    let ws_path = "ws".to_string();
    let api_path = "api".to_string();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["user-agent", "content-type"])
        .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"]);

    let http_routes = crate::api::create_routes(app_context.clone(), api_path);
    let context = warp::any().map(move || app_context.clone()).boxed();

    let ws_routes = warp::path(ws_path)
        .and(warp::addr::remote())
        .and(context)
        .and(warp::header::optional::<String>("user-agent"))
        .and(warp::ws())
        .map(
            move |remote_addr: Option<SocketAddr>,
                  application_context,
                  user_agent: Option<String>,
                  ws: ws::Ws| {
                ws.on_upgrade(move |socket| {
                    handle_client_connection(socket, application_context, remote_addr, user_agent)
                })
            },
        )
        .boxed();

    warp::serve(http_routes.or(ws_routes).with(cors))
        // .tls()
        // .cert_path(Path::new("cert.pem"))
        // .key_path(Path::new("key.pem"))
        .run((IpAddr::V4(host_addr.parse().unwrap()), server_port))
        .await;
}

async fn handle_client_connection(
    ws: WebSocket,
    app_context: Arc<Mutex<ApplicationContext>>,
    remote_addr: Option<SocketAddr>,
    user_agent: Option<String>,
) {
    let (tx, rx) = ws.split();
    let tx = Arc::new(Mutex::new(tx));
    let fragment_registry = { app_context.lock().await.fragment_registry.clone() };
    let fragment_registry = Arc::new(Mutex::new(fragment_registry));
    // Create a new client and register it
    let client = Client::new(
        fragment_registry,
        remote_addr.map_or(String::new(), |addr| addr.to_string()),
        user_agent.unwrap_or_default(),
        tx.clone(),
    );
    let uuid = client.uuid;
    let client = Arc::new(Mutex::new(client));
    let mut client_event_listener = ClientEventListener::new(client.clone(), rx, tx);
    app_context
        .lock()
        .await
        .client_registry
        .lock()
        .await
        .register(uuid, client);
    client_event_listener.handle_events().await;
    app_context
        .lock()
        .await
        .client_registry
        .lock()
        .await
        .unregister(uuid);
}

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
