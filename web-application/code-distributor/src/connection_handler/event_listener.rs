use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use warp::ws::{Message as WsMessage, WebSocket};

use crate::connection_handler::message::{Events, Message};
use crate::fragment_executor::FragmentExecutor;
use crate::fragment_registry::fragment::ExecutionLocation;

pub(crate) async fn handle_events(
    mut rx: SplitStream<WebSocket>,
    tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    fragment_executor: Arc<dyn FragmentExecutor>,
) {
    let tx = tx.clone();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let raw_message: Value =
                        serde_json::from_str(msg.to_str().unwrap()).unwrap_or_default();
                    let message =
                        serde_json::from_value::<Message<Value>>(raw_message.clone()).unwrap();
                    if let Events::ExecuteFunction = message.message_type {
                        log::info!("Received ExecuteFunction event");
                        let tx = tx.clone();
                        let fragment_executor = fragment_executor.clone();
                        tokio::spawn(async move {
                            handle_execute_function_event(tx, message, fragment_executor).await
                        });
                    }
                }
            }
            Err(err) => {
                eprintln!("Error receiving message: {}", err);
            }
        }
    }
}

async fn handle_execute_function_event(
    tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    message: Message<Value>,
    fragment_executor: Arc<dyn FragmentExecutor>,
) {
    match serde_json::from_value::<ExecuteFunctionData>(message.data) {
        Ok(execute_function_data) => {
            let result = fragment_executor
                .execute(
                    &execute_function_data.fragment_id,
                    &execute_function_data.function_name,
                    &execute_function_data.parameters,
                )
                .await;
            if let Ok(result) = result {
                let message = Message::new(message.message_id, Events::ExecuteFunction, result);
                let json_string = serde_json::to_string(&message)
                    .unwrap_or("Unable to serialize the result".to_string());
                {
                    tx.lock()
                        .await
                        .send(WsMessage::text(json_string))
                        .await
                        .ok();
                }
            }
        }
        Err(e) => {
            log::error!("Failed to parse UpdateFragmentData: {:?}", e);
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ExecuteFunctionData {
    fragment_id: String,
    function_name: String,
    parameters: Vec<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct UpdateFragmentData {
    pub id: String,
    pub execution_location: ExecutionLocation,
}
