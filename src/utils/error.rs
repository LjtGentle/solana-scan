use thiserror::Error;
use anyhow::Result;

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Solana RPC error: {0}")]
    SolanaRpcError(String),
    
    #[error("Kafka error: {0}")]
    KafkaError(String),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl From<mongodb::error::Error> for ScannerError {
    fn from(error: mongodb::error::Error) -> Self {
        ScannerError::DatabaseError(error.to_string())
    }
}

impl From<solana_client::client_error::ClientError> for ScannerError {
    fn from(error: solana_client::client_error::ClientError) -> Self {
        ScannerError::SolanaRpcError(error.to_string())
    }
}

impl From<rdkafka::error::KafkaError> for ScannerError {
    fn from(error: rdkafka::error::KafkaError) -> Self {
        ScannerError::KafkaError(error.to_string())
    }
}

pub type ScannerResult<T> = Result<T, ScannerError>;