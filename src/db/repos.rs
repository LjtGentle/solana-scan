use anyhow::Result;
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Collection, Database};

use crate::models::{ScanStatus, Transaction, WalletAddress};

pub struct WalletAddressRepo {
    collection: Collection<WalletAddress>,
}

impl WalletAddressRepo {
    pub fn new(database: Database) -> Self {
        let collection = database.collection("wallet_addresses");
        Self { collection }
    }

    pub async fn insert_address(&self, address: &str, label: Option<&str>) -> Result<()> {
        let wallet_address = WalletAddress::new(address.to_string(), label.map(|s| s.to_string()));
        self.collection.insert_one(&wallet_address, None).await?;
        Ok(())
    }

    pub async fn get_all_active_addresses(&self) -> Result<Vec<WalletAddress>> {
        let cursor = self
            .collection
            .find(doc! { "is_active": true }, None)
            .await?;

        let addresses: Vec<WalletAddress> = cursor.try_collect().await?;
        Ok(addresses)
    }

    pub async fn deactivate_address(&self, address: &str) -> Result<()> {
        self.collection
            .update_one(
                doc! { "address": address },
                doc! {
                    "$set": {
                        "is_active": false,
                        "updated_at": mongodb::bson::DateTime::now()
                    }
                },
                None,
            )
            .await?;
        Ok(())
    }
}

pub struct TransactionRepo {
    collection: Collection<Transaction>,
}

impl TransactionRepo {
    pub fn new(database: Database) -> Self {
        let collection = database.collection("transactions");
        Self { collection }
    }

    pub async fn insert_transaction(&self, transaction: &Transaction) -> Result<()> {
        self.collection.insert_one(transaction, None).await?;
        Ok(())
    }

    pub async fn get_transactions(
        &self,
        address: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Transaction>> {
        let mut filter = doc! {};

        if let Some(addr) = address {
            filter = doc! {
                "$or": [
                    { "from_address": &addr },
                    { "to_address": &addr }
                ]
            };
        }

        let mut options = mongodb::options::FindOptions::default();

        if let Some(limit) = limit {
            options.limit = Some(limit as i64);
        }

        if let Some(offset) = offset {
            options.skip = Some(offset as u64);
        }
        let cursor = self.collection.find(filter, options).await?;
        let transactions: Vec<Transaction> = cursor.try_collect().await?;

        Ok(transactions)
    }

    pub async fn get_transaction_by_signature(
        &self,
        signature: &str,
    ) -> Result<Option<Transaction>> {
        let transaction = self
            .collection
            .find_one(doc! { "signature": signature }, None)
            .await?;

        Ok(transaction)
    }
}

pub struct ScanStatusRepo {
    collection: Collection<ScanStatus>,
}

impl ScanStatusRepo {
    pub fn new(database: Database) -> Self {
        let collection = database.collection("scan_status");
        Self { collection }
    }

    pub async fn get_scan_status(&self) -> Result<Option<ScanStatus>> {
        let status = self
            .collection
            .find_one(doc! { "id": "scan_status" }, None)
            .await?;

        Ok(status)
    }

    pub async fn update_scan_status(&self, status: &ScanStatus) -> Result<()> {
        self.collection
            .replace_one(
                doc! { "id": "scan_status" },
                status,
                mongodb::options::ReplaceOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await?;

        Ok(())
    }
}
