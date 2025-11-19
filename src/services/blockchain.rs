use anyhow::Result;
use chrono::Utc;
use mongodb::Database;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::UiTransactionEncoding;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use futures::stream::{self, StreamExt};
use tracing::{debug, error, info};

use crate::config::KafkaConfig;
use crate::db::{ScanStatusRepo, TransactionRepo, WalletAddressRepo};
use crate::models::{ScanStatus, Transaction, TransactionType};
use crate::services::websocket::WebSocketManager;
use crate::utils::kafka::KafkaProducer;

pub struct BlockchainScanner {
    rpc_client: RpcClient,
    db: Database,
    kafka_producer: Arc<KafkaProducer>,
    watched_addresses: Arc<RwLock<HashSet<String>>>,
    scan_status: Arc<RwLock<Option<ScanStatus>>>,
    ws_manager: Arc<RwLock<WebSocketManager>>, 
    max_concurrent_requests: usize,
}

impl BlockchainScanner {
    pub async fn new(
        rpc_url: String,
        db: Database,
        kafka_config: KafkaConfig,
        ws_manager: Arc<RwLock<WebSocketManager>>,
        max_concurrent_requests: usize,
    ) -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        let kafka_producer = Arc::new(KafkaProducer::new(&kafka_config).await?);

        let scanner = Self {
            rpc_client,
            db,
            kafka_producer,
            watched_addresses: Arc::new(RwLock::new(HashSet::new())),
            scan_status: Arc::new(RwLock::new(None)),
            ws_manager,
            max_concurrent_requests,
        };

        // 加载关注的钱包地址
        scanner.load_watched_addresses().await?;

        // 加载扫描状态
        scanner.load_scan_status().await?;

        Ok(scanner)
    }

    async fn load_watched_addresses(&self) -> Result<()> {
        let repo = WalletAddressRepo::new(self.db.clone());
        let addresses = repo.get_all_active_addresses().await?;

        let mut watched = self.watched_addresses.write().await;
        for addr in addresses {
            watched.insert(addr.address.clone());
        }

        info!("Loaded {} watched addresses", watched.len());
        Ok(())
    }

    async fn load_scan_status(&self) -> Result<()> {
        let repo = ScanStatusRepo::new(self.db.clone());
        let status = repo.get_scan_status().await?;

        let mut scan_status = self.scan_status.write().await;
        *scan_status = status;

        Ok(())
    }

    pub async fn start_scanning(&self) -> Result<()> {
        info!("Starting blockchain scanning...");

        let mut scan_interval = interval(Duration::from_millis(200));

        loop {
            scan_interval.tick().await;

            if let Err(e) = self.scan_blocks().await {
                error!("Error scanning blocks: {}", e);
            }
        }
    }

    async fn scan_blocks(&self) -> Result<()> {
        let current_slot = self.rpc_client.get_slot()?;
        let start_slot = {
            let scan_status = self.scan_status.read().await;
            if let Some(status) = scan_status.as_ref() {
                status.last_scanned_block + 1
            } else {
                current_slot.saturating_sub(300)
            }
        };

        if start_slot > current_slot {
            debug!("No new blocks to scan");
            return Ok(());
        }

        info!("Scanning blocks from {} to {}", start_slot, current_slot);

        let concurrency = std::cmp::max(1, self.max_concurrent_requests);
        stream::iter(start_slot..=current_slot)
            .map(|slot| async move { (slot, self.scan_block(slot).await) })
            .buffer_unordered(concurrency)
            .for_each(|res| async move {
                let (slot, outcome) = res;
                match outcome {
                    Ok(_) => { let _ = self.update_scan_status(slot).await; }
                    Err(e) => { error!("Error scanning block {}: {}", slot, e); }
                }
            })
            .await;

        Ok(())
    }

    async fn scan_block(&self, slot: u64) -> Result<()> {
        debug!("Scanning block {}", slot);

        let block = self.rpc_client.get_block_with_config(
            slot,
            solana_client::rpc_config::RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
                rewards: Some(false),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )?;

        if let Some(transactions) = block.transactions {
            for tx in transactions {
                // 这里需要正确处理交易数据
                // 简化处理，实际需要解析transaction数据
                if let Err(e) = self
                    .process_transaction(slot, &tx.transaction, tx.meta.as_ref())
                    .await
                {
                    error!("Error processing transaction: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn process_transaction(
        &self,
        slot: u64,
        transaction: &solana_transaction_status::EncodedTransaction,
        meta: Option<&solana_transaction_status::UiTransactionStatusMeta>,
    ) -> Result<()> {
        let watched = self.watched_addresses.read().await;
        match transaction {
            solana_transaction_status::EncodedTransaction::Json(ui_tx) => {
                let signature = ui_tx.signatures.get(0).cloned().unwrap_or_default();
                match &ui_tx.message {
                    solana_transaction_status::UiMessage::Parsed(message) => {
                        let account_keys: Vec<String> = message
                            .account_keys
                            .iter()
                            .map(|k| k.pubkey.clone())
                            .collect();
                        let involved = account_keys.iter().any(|k| watched.contains(k));
                        if !involved {
                            return Ok(());
                        }
                        let fee_lamports = meta.map(|m| m.fee as f64).unwrap_or(0.0);
                        let fee_sol = fee_lamports / 1_000_000_000f64;
                        for instr in &message.instructions {
                            if let solana_transaction_status::UiInstruction::Parsed(parsed_ins) =
                                instr
                            {
                                match parsed_ins {
                                    solana_transaction_status::UiParsedInstruction::Parsed(pi) => {
                                        let program = pi.program.as_str();
                                        let parsed_val = &pi.parsed;
                                        if program == "system" {
                                            if parsed_val.get("type").and_then(|v| v.as_str())
                                                == Some("transfer")
                                            {
                                                if let Some(info) = parsed_val.get("info") {
                                                    let from = info
                                                        .get("source")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let to = info
                                                        .get("destination")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| s.to_string());
                                                    let lamports = info
                                                        .get("lamports")
                                                        .and_then(|v| v.as_u64())
                                                        .unwrap_or(0);
                                                    let amount =
                                                        (lamports as f64) / 1_000_000_000f64;
                                                    if watched.contains(&from)
                                                        || to
                                                            .as_ref()
                                                            .map(|t| watched.contains(t))
                                                            .unwrap_or(false)
                                                    {
                                                        let tx_record = Transaction::new(
                                                            signature.clone(),
                                                            slot,
                                                            TransactionType::Native,
                                                            from,
                                                            to,
                                                            amount,
                                                            None,
                                                            None,
                                                            fee_sol,
                                                            Utc::now(),
                                                            if meta
                                                                .map(|m| m.err.is_none())
                                                                .unwrap_or(false)
                                                            {
                                                                crate::models::TransactionStatus::Confirmed
                                                            } else {
                                                                crate::models::TransactionStatus::Failed
                                                            },
                                                            Some(parsed_val.clone()),
                                                        );
                                                        let tx_repo =
                                                            TransactionRepo::new(self.db.clone());
                                                        let _ = tx_repo
                                                            .insert_transaction(&tx_record)
                                                            .await;
                                                        self.dispatch_transaction(tx_record);
                                                    }
                                                }
                                            }
                                        } else if program == "spl-token"
                                            || program == "spl-token-2022"
                                        {
                                            let t = parsed_val
                                                .get("type")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            if t == "transfer" || t == "transferChecked" {
                                                if let Some(info) = parsed_val.get("info") {
                                                    let from = info
                                                        .get("source")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let to = info
                                                        .get("destination")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| s.to_string());
                                                    let mint = info
                                                        .get("mint")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| s.to_string());
                                                    let amount_raw = info.get("amount");
                                                    let decimals = info
                                                        .get("decimals")
                                                        .and_then(|v| v.as_u64())
                                                        .unwrap_or(0);
                                                    let mut amount = 0f64;
                                                    if let Some(v) = amount_raw {
                                                        if let Some(s) = v.as_str() {
                                                            amount =
                                                                s.parse::<f64>().unwrap_or(0.0);
                                                        } else if let Some(n) = v.as_u64() {
                                                            amount = n as f64;
                                                        } else if let Some(n) = v.as_f64() {
                                                            amount = n;
                                                        }
                                                    }
                                                    if decimals > 0 {
                                                        amount =
                                                            amount / 10f64.powi(decimals as i32);
                                                    }
                                                    let tx_type = if decimals == 0
                                                        && (amount - 1.0).abs() < f64::EPSILON
                                                    {
                                                        TransactionType::Nft
                                                    } else {
                                                        TransactionType::Token
                                                    };
                                                    if watched.contains(&from)
                                                        || to
                                                            .as_ref()
                                                            .map(|t| watched.contains(t))
                                                            .unwrap_or(false)
                                                    {
                                                        let tx_record = Transaction::new(
                                                            signature.clone(),
                                                            slot,
                                                            tx_type,
                                                            from,
                                                            to,
                                                            amount,
                                                            mint,
                                                            None,
                                                            fee_sol,
                                                            Utc::now(),
                                                            if meta
                                                                .map(|m| m.err.is_none())
                                                                .unwrap_or(false)
                                                            {
                                                                crate::models::TransactionStatus::Confirmed
                                                            } else {
                                                                crate::models::TransactionStatus::Failed
                                                            },
                                                            Some(parsed_val.clone()),
                                                        );
                                                        let tx_repo =
                                                            TransactionRepo::new(self.db.clone());
                                                        let _ = tx_repo
                                                            .insert_transaction(&tx_record)
                                                            .await;
                                                        self.dispatch_transaction(tx_record);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn dispatch_transaction(&self, tx: Transaction) {
        let kafka = self.kafka_producer.clone();
        let ws = self.ws_manager.clone();
        tokio::spawn(async move {
            let _ = kafka.send_transaction(&tx).await;
            let _ = ws.read().await.broadcast_transaction(&tx).await;
        });
    }

    async fn update_scan_status(&self, last_block: u64) -> Result<()> {
        let repo = ScanStatusRepo::new(self.db.clone());

        let scan_status = ScanStatus::new(last_block);
        let _ = repo.update_scan_status(&scan_status).await;

        let mut current_status = self.scan_status.write().await;
        *current_status = Some(scan_status);

        Ok(())
    }

    pub async fn add_watched_address(&self, address: String) -> Result<()> {
        let mut watched = self.watched_addresses.write().await;
        watched.insert(address.clone());

        let repo = WalletAddressRepo::new(self.db.clone());
        let _ = repo.insert_address(&address, None).await;

        Ok(())
    }

    pub async fn remove_watched_address(&self, address: String) -> Result<()> {
        let mut watched = self.watched_addresses.write().await;
        watched.remove(&address);

        let repo = WalletAddressRepo::new(self.db.clone());
        let _ = repo.deactivate_address(&address).await;

        Ok(())
    }

    pub async fn get_watched_addresses(&self) -> Vec<String> {
        let watched = self.watched_addresses.read().await;
        watched.iter().cloned().collect()
    }

    pub async fn get_transactions(
        &self,
        address: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Transaction>> {
        let tx_repo = TransactionRepo::new(self.db.clone());
        let _ = tx_repo.get_transactions(address, limit, offset).await;
        Ok(vec![])
    }
}
