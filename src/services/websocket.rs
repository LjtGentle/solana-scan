use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;

pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
}

pub struct WebSocketConnection {
    pub id: String,
    pub subscribed_addresses: HashMap<String, bool>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, connection_id: String) {
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            subscribed_addresses: HashMap::new(),
        };

        let mut connections = self.connections.write().await;
        let conn_id = connection_id.clone();
        connections.insert(connection_id, connection);
        
        info!("Added WebSocket connection: {}", conn_id);
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
        
        info!("Removed WebSocket connection: {}", connection_id);
    }

    pub async fn subscribe_to_address(&self, connection_id: &str, address: String) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(connection_id) {
            let addr = address.clone();
            connection.subscribed_addresses.insert(address, true);
            info!("Connection {} subscribed to address {}", connection_id, addr);
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn unsubscribe_from_address(&self, connection_id: &str, address: &str) -> Result<(), String> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.subscribed_addresses.remove(address);
            info!("Connection {} unsubscribed from address {}", connection_id, address);
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub async fn broadcast_transaction(&self, transaction: &crate::models::Transaction) {
        let connections = self.connections.read().await;
        let from_address = &transaction.from_address;
        let to_address = transaction.to_address.as_ref();

        for (connection_id, connection) in connections.iter() {
            let should_notify = 
                connection.subscribed_addresses.contains_key(from_address) ||
                to_address.map_or(false, |addr| connection.subscribed_addresses.contains_key(addr));

            if should_notify {
                // 这里需要通过WebSocket连接发送消息
                // 实际实现中需要存储WebSocket连接的sender
                info!("Would broadcast transaction to connection {}: {}", connection_id, transaction.signature);
            }
        }
    }

    pub async fn get_subscribed_addresses(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        let mut addresses = HashMap::new();

        for connection in connections.values() {
            for address in connection.subscribed_addresses.keys() {
                addresses.insert(address.clone(), true);
            }
        }

        addresses.keys().cloned().collect()
    }
}