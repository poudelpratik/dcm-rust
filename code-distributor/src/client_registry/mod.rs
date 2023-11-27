use crate::client_registry::client::ClientDto;
use crate::client_registry::client_event_listener::UpdateFragmentData;
use crate::util::error::ApplicationError;
use client::Client;
use futures_util::future::join_all;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub mod client;
pub mod client_event_listener;

#[derive(Debug, Default)]
pub(crate) struct ClientRegistry {
    clients: HashMap<Uuid, Arc<Mutex<Client>>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        ClientRegistry {
            clients: HashMap::new(),
        }
    }

    pub fn register(&mut self, uuid: Uuid, client: Arc<Mutex<Client>>) {
        self.clients.insert(uuid, client);
        info!("Client connected: {:?}", &uuid);
        info!("Active clients: {}", self.clients.len());
    }

    pub fn unregister(&mut self, uuid: Uuid) {
        self.clients.remove(&uuid);
        info!("Client disconnected: {:?}", &uuid);
        info!("Active clients: {}", self.clients.len());
    }

    pub async fn get_clients(&self) -> Vec<ClientDto> {
        let client_handles: Vec<Arc<Mutex<Client>>> = self.clients.values().cloned().collect();
        let futures: Vec<_> = client_handles
            .into_iter()
            .map(|client_handle| async move {
                let client = client_handle.lock().await;
                let fragment_registry = client.fragment_registry.lock().await;
                ClientDto::new(
                    client.uuid,
                    fragment_registry.clone(), // Assuming FragmentRegistry implements Clone
                    client.ip_address.clone(),
                    client.user_agent.clone(),
                )
            })
            .collect();
        join_all(futures).await.into_iter().collect()
    }

    pub async fn get_client(&self, uuid: Uuid) -> ClientDto {
        let client: Arc<Mutex<Client>> = self.clients.get(&uuid).unwrap().clone();
        let client = client.lock().await.clone();
        let fragment_registry = client.fragment_registry.lock().await;
        ClientDto::new(
            client.uuid,
            fragment_registry.clone(), // Assuming FragmentRegistry implements Clone
            client.ip_address.clone(),
            client.user_agent.clone(),
        )
    }

    pub async fn update_client_fragments(
        &self,
        uuid: Uuid,
        update_fragments_data: Vec<UpdateFragmentData>,
    ) -> Result<(), ApplicationError> {
        let client_handle: Arc<Mutex<Client>> = self.clients.get(&uuid).cloned().unwrap();
        let mut client = client_handle.lock().await.clone();
        client.update_fragments(update_fragments_data).await?;
        Ok(())
    }
}
