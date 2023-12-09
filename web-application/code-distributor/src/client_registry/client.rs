use std::sync::Arc;

use crate::connection_handler::event_listener::UpdateFragmentData;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use log::info;
use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::ws::{Message as WsMessage, WebSocket};

use crate::connection_handler::message::{Events, Message};
use crate::fragment_registry::fragment::Fragment;
use crate::fragment_registry::FragmentRegistry;
use crate::util::error::ApplicationError;

#[derive(Debug, Clone)]
pub(crate) struct Client {
    pub(crate) uuid: Uuid,
    pub(crate) fragment_registry: FragmentRegistry,
    pub(crate) connected: bool,
    pub(crate) tx: Option<Arc<Mutex<SplitSink<WebSocket, WsMessage>>>>,
}

impl Client {
    pub(crate) fn new(
        uuid: Uuid,
        fragment_registry: FragmentRegistry,
        tx: Option<Arc<Mutex<SplitSink<WebSocket, WsMessage>>>>,
    ) -> Self {
        Self {
            uuid,
            fragment_registry,
            connected: false,
            tx,
        }
    }

    pub async fn update_fragments(
        &mut self,
        update_fragments_data: Vec<UpdateFragmentData>,
    ) -> Result<(), ApplicationError> {
        self.fragment_registry
            .update_fragments(&update_fragments_data)
            .ok();
        self.send_message(update_fragments_data, Events::UpdateFragments)
            .await?;
        Ok(())
    }

    pub async fn connected(&mut self, tx: Arc<Mutex<SplitSink<WebSocket, WsMessage>>>) {
        self.connected = true;
        info!("Client connected: {:?}", &self.uuid);
        self.tx = Some(tx);
        let update_fragments: Vec<UpdateFragmentData> = self
            .fragment_registry
            .fragments
            .clone()
            .into_iter()
            .map(|fragment| UpdateFragmentData {
                id: fragment.id,
                execution_location: fragment.execution_location,
            })
            .collect();
        self.send_message(update_fragments, Events::UpdateFragments)
            .await
            .ok();
    }

    pub fn disconnected(&mut self) {
        self.connected = false;
        info!("Client disconnected: {:?}", &self.uuid);
    }

    async fn send_message<T>(&mut self, data: T, event: Events) -> Result<(), ApplicationError>
    where
        T: Serialize,
    {
        let message = Message::new(Uuid::new_v4().to_string(), event, data);
        let json_string = serde_json::to_string(&message).unwrap();
        self.tx
            .clone()
            .unwrap()
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
    pub(crate) fragments: Vec<Fragment>,
    pub(crate) connected: bool,
}

impl From<Client> for ClientDto {
    fn from(client: Client) -> Self {
        Self {
            uuid: client.uuid,
            fragments: client.fragment_registry.fragments,
            connected: client.connected,
        }
    }
}
