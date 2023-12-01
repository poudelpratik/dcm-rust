use std::collections::HashMap;
use std::sync::Arc;

use futures_util::future::join_all;
use log::info;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::client_registry::client::Client;

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
        info!("New Client Registered: {:?}", &uuid);
    }

    // pub async fn unregister(&mut self, uuid: Uuid) {
    //     if let Some(client) = self.clients.remove(&uuid) {
    //         let client_guard = client.lock().await;
    //         self.auth_tokens.remove(&client_guard.auth_token);
    //         info!("Client Unregistered: {:?}", &uuid);
    //     }
    // }

    pub async fn get_all_clients(&self) -> Vec<Client> {
        let client_handles: Vec<Arc<Mutex<Client>>> = self.clients.values().cloned().collect();
        let futures: Vec<_> = client_handles
            .into_iter()
            .map(|client_handle| async move {
                let client = client_handle.lock().await;
                client.clone()
            })
            .collect();
        join_all(futures).await.into_iter().collect()
    }

    pub async fn get_client_ref_by_id(&self, uuid: Uuid) -> Option<Arc<Mutex<Client>>> {
        self.clients.get(&uuid).cloned()
    }
}
