use crate::client_registry::client::Client;
use crate::connection_handler::message::{Events, Message};
use crate::fragment_executor::wasmer_runtime::WasmerRuntime;
use crate::fragment_executor::FragmentExecutor;
use crate::fragment_registry::fragment::ExecutionLocation;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::ws::{Message as WsMessage, WebSocket};

#[derive(Debug)]
pub(crate) struct ClientEventListener {
    pub(crate) client: Arc<Mutex<Client>>,
    pub rx: SplitStream<WebSocket>,
    pub tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
}

impl ClientEventListener {
    pub(crate) fn new(
        client: Arc<Mutex<Client>>,
        rx: SplitStream<WebSocket>,
        tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    ) -> Self {
        Self {
            client,
            rx,
            tx,
        }
    }

    /// When a client connects, this function is called to listen to events and handle accordingly.
    pub(crate) async fn handle_events(&mut self) {
        let tx = self.tx.clone();
        while let Some(result) = self.rx.next().await {
            match result {
                Ok(msg) => {
                    if msg.is_text() {
                        let raw_message: Value =
                            serde_json::from_str(msg.to_str().unwrap()).unwrap_or_default();
                        let message =
                            serde_json::from_value::<Message<Value>>(raw_message.clone()).unwrap();
                        match message.message_type {
                            Events::ExecuteFunction => {
                                log::info!("Received ExecuteFunction event");
                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    handle_execute_function_event(tx, message).await
                                });
                            }
                            Events::UpdateFragments => {
                                log::info!("Received UpdateFragments event");
                                let tx = tx.clone();
                                handle_update_fragments(tx, message, self.client.clone()).await;
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error receiving message: {}", err);
                }
            }
        }
    }
}

async fn handle_update_fragments(
    tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    message: Message<Value>,
    client: Arc<Mutex<Client>>,
) {
    match serde_json::from_value::<Vec<UpdateFragmentData>>(message.data) {
        Ok(event_data) => {
            // Handle the async operation separately
            if client
                .lock()
                .await
                .fragment_registry
                .lock()
                .await
                .update_fragments(&event_data)
                .is_ok()
            {
                let message = Message::new(
                    message.message_id,
                    Events::ExecuteFunction,
                    "Fragment Registry updated.",
                );
                let json_string = serde_json::to_string(&message)
                    .unwrap_or("Unable to serialize the data".to_string());
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

async fn handle_execute_function_event(
    tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    message: Message<Value>,
) {
    match serde_json::from_value::<ExecuteFunctionData>(message.data) {
        Ok(execute_function_data) => {
            let result = WasmerRuntime::execute(
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
    pub fragment_id: String,
    pub execution_location: ExecutionLocation,
}
