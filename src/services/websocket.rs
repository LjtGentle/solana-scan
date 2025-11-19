use axum::extract::ws::Message;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, RwLock};
use tracing::info;

pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    address_subscribers: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

pub struct WebSocketConnection {
    pub id: String,
    pub subscribed_addresses: HashMap<String, bool>,
    pub sender: UnboundedSender<Message>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            address_subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, connection_id: String, sender: UnboundedSender<Message>) {
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            subscribed_addresses: HashMap::new(),
            sender,
        };
        let mut connections = self.connections.write().await;
        connections.insert(connection_id.clone(), connection);
        info!("Added WebSocket connection: {}", connection_id);
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.remove(connection_id) {
            let mut index = self.address_subscribers.write().await;
            for address in conn.subscribed_addresses.keys() {
                if let Some(set) = index.get_mut(address) {
                    set.remove(connection_id);
                    if set.is_empty() {
                        index.remove(address);
                    }
                }
            }
        }
        info!("Removed WebSocket connection: {}", connection_id);
    }

    pub async fn subscribe_to_address(
        &self,
        connection_id: &str,
        address: String,
    ) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            let addr = address.clone();
            connection
                .subscribed_addresses
                .insert(address.clone(), true);
            drop(connections);
            let mut index = self.address_subscribers.write().await;
            index
                .entry(addr.clone())
                .or_default()
                .insert(connection_id.to_string());
            info!(
                "Connection {} subscribed to address {}",
                connection_id, addr
            );
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn unsubscribe_from_address(
        &self,
        connection_id: &str,
        address: &str,
    ) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.subscribed_addresses.remove(address);
            drop(connections);
            let mut index = self.address_subscribers.write().await;
            if let Some(set) = index.get_mut(address) {
                set.remove(connection_id);
                if set.is_empty() {
                    index.remove(address);
                }
            }
            info!(
                "Connection {} unsubscribed from address {}",
                connection_id, address
            );
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn broadcast_transaction(&self, transaction: &crate::models::Transaction) {
        let payload = serde_json::to_string(transaction).unwrap_or_else(|_| "{}".to_string());
        let mut targets: HashSet<String> = HashSet::new();
        let index = self.address_subscribers.read().await;
        if let Some(set) = index.get(&transaction.from_address) {
            targets.extend(set.iter().cloned());
        }
        if let Some(to) = transaction.to_address.as_ref() {
            if let Some(set) = index.get(to) {
                targets.extend(set.iter().cloned());
            }
        }
        drop(index);
        let connections = self.connections.read().await;
        for cid in targets {
            if let Some(conn) = connections.get(&cid) {
                let _ = conn.sender.send(Message::Text(payload.clone()));
            }
        }
    }

    pub async fn get_subscribed_addresses(&self) -> Vec<String> {
        let index = self.address_subscribers.read().await;
        index.keys().cloned().collect()
    }
}
