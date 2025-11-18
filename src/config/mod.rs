use serde::{Deserialize, Serialize};
use std::env;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub solana_rpc_url: String,
    pub mongodb_uri: String,
    pub kafka_config: KafkaConfig,
    pub rpc_port: u16,
    pub websocket_port: u16,
    pub scan_interval_secs: u64,
    pub max_addresses: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub transaction_topic: String,
    pub client_id: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();

        let config = AppConfig {
            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            mongodb_uri: env::var("MONGODB_URI")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            kafka_config: KafkaConfig {
                brokers: env::var("KAFKA_BROKERS")
                    .unwrap_or_else(|_| "localhost:9092".to_string()),
                transaction_topic: env::var("KAFKA_TRANSACTION_TOPIC")
                    .unwrap_or_else(|_| "solana_transactions".to_string()),
                client_id: env::var("KAFKA_CLIENT_ID")
                    .unwrap_or_else(|_| "solana_scanner".to_string()),
            },
            rpc_port: env::var("RPC_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            websocket_port: env::var("WEBSOCKET_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .unwrap_or(8081),
            scan_interval_secs: env::var("SCAN_INTERVAL_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_addresses: env::var("MAX_ADDRESSES")
                .unwrap_or_else(|_| "100000".to_string())
                .parse()
                .unwrap_or(100000),
        };

        Ok(config)
    }
}