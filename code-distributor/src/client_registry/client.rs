use crate::client_registry::client_event_listener::UpdateFragmentData;
use crate::connection_handler::message::{Events, Message};
use crate::fragment_registry::FragmentRegistry;
use crate::util::error::ApplicationError;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use serde::Serialize;
use serde_derive::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::ws::{Message as WsMessage, WebSocket};

#[derive(Debug, Clone)]
pub(crate) struct Client {
    pub(crate) uuid: Uuid,
    pub(crate) fragment_registry: Arc<Mutex<FragmentRegistry>>,
    pub(crate) ip_address: String,
    pub(crate) user_agent: String,
    pub(crate) tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
}

impl Client {
    pub(crate) fn new(
        fragment_registry: Arc<Mutex<FragmentRegistry>>,
        ip_address: String,
        user_agent: String,
        tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            fragment_registry,
            ip_address,
            user_agent,
            tx,
        }
    }

    pub async fn update_fragments(
        &mut self,
        update_fragments_data: Vec<UpdateFragmentData>,
    ) -> Result<(), ApplicationError> {
        self.fragment_registry
            .lock()
            .await
            .update_fragments(&update_fragments_data)
            .ok();
        self.send_message(update_fragments_data, Events::UpdateFragments)
            .await?;
        Ok(())
    }

    async fn send_message<T>(&mut self, data: T, event: Events) -> Result<(), ApplicationError>
    where
        T: Serialize,
    {
        let message = Message::new(Uuid::new_v4().to_string(), event, data);
        let json_string = serde_json::to_string(&message).unwrap();
        self.tx
            .lock()
            .await
            .send(WsMessage::text(json_string))
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ClientDto {
    pub(crate) uuid: Uuid,
    pub(crate) fragment_registry: FragmentRegistry,
    pub(crate) ip_address: String,
    pub(crate) user_agent: String,
}

impl ClientDto {
    pub fn new(
        uuid: Uuid,
        fragment_registry: FragmentRegistry,
        ip_address: String,
        user_agent: String,
    ) -> Self {
        Self {
            uuid,
            fragment_registry,
            ip_address,
            user_agent,
        }
    }
}
