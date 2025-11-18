use serde::{Deserialize, Serialize};
use mongodb::bson::doc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    pub id: String,
    pub address: String,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl WalletAddress {
    pub fn new(address: String, label: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            address,
            label,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub signature: String,
    pub block_number: u64,
    pub transaction_type: TransactionType,
    pub from_address: String,
    pub to_address: Option<String>,
    pub amount: f64,
    pub token_mint: Option<String>,
    pub token_symbol: Option<String>,
    pub fee: f64,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Native,
    Token,
    Nft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Confirmed,
    Failed,
    Pending,
}

impl Transaction {
    pub fn new(
        signature: String,
        block_number: u64,
        transaction_type: TransactionType,
        from_address: String,
        to_address: Option<String>,
        amount: f64,
        token_mint: Option<String>,
        token_symbol: Option<String>,
        fee: f64,
        timestamp: DateTime<Utc>,
        status: TransactionStatus,
        raw_data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            signature,
            block_number,
            transaction_type,
            from_address,
            to_address,
            amount,
            token_mint,
            token_symbol,
            fee,
            timestamp,
            status,
            raw_data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatus {
    pub id: String,
    pub last_scanned_block: u64,
    pub last_scan_time: DateTime<Utc>,
    pub total_transactions_scanned: u64,
    pub is_scanning: bool,
}

impl ScanStatus {
    pub fn new(last_scanned_block: u64) -> Self {
        Self {
            id: "scan_status".to_string(),
            last_scanned_block,
            last_scan_time: Utc::now(),
            total_transactions_scanned: 0,
            is_scanning: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> RpcResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionQuery {
    pub address: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>, 
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[cfg(test)]
mod tests;