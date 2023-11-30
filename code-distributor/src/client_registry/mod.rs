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
        info!("Client authenticated: {:?}", &uuid);
    }

    pub async fn handle_connection(&mut self, uuid: Uuid) {
        let client = self.clients.get(&uuid);
        if let Some(client) = client {
            let mut client = client.lock().await;
            client.connected = true;
        }
        info!("Client connected: {:?}", &uuid);
    }

    pub async fn handle_disconnection(&mut self, uuid: Uuid) {
        let client = self.clients.get(&uuid);
        if let Some(client) = client {
            let mut client = client.lock().await;
            client.connected = false;
        }
        info!("Client disconnected: {:?}", &uuid);
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
                )
            })
            .collect();
        join_all(futures).await.into_iter().collect()
    }

    pub async fn get_client_by_id(&self, uuid: Uuid) -> Option<ClientDto> {
        let client = self.clients.get(&uuid);
        match client {
            None => None,
            Some(client) => {
                let client = client.lock().await;
                let fragment_registry = client.fragment_registry.lock().await;
                Some(ClientDto::new(
                    client.uuid,
                    fragment_registry.clone(), // Assuming FragmentRegistry implements Clone
                ))
            }
        }
    }

    pub async fn get_client_by_token(&self, auth_token: String) -> Option<Client> {
        let auth_token = Arc::new(auth_token);
        let client_handles: Vec<Arc<Mutex<Client>>> = self.clients.values().cloned().collect();
        let futures: Vec<_> = client_handles
            .into_iter()
            .map(|client_handle| {
                let auth_token = Arc::clone(&auth_token); // Clone the Arc for each async block
                async move {
                    let client = client_handle.lock().await;
                    if client.auth_token == *auth_token {
                        Some(client.clone())
                    } else {
                        None
                    }
                }
            })
            .collect();
        let results: Vec<Option<Client>> = join_all(futures).await;
        results.into_iter().flatten().next()
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
