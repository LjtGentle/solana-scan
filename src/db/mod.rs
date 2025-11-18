use mongodb::{Client, Database};
use anyhow::Result;

pub mod repos;

pub use repos::*;

pub async fn init_mongodb(uri: &str) -> Result<Database> {
    let client = Client::with_uri_str(uri).await?;
    let database = client.database("solana_scanner");
    
    // 创建索引
    create_indexes(&database).await?;
    
    Ok(database)
}

async fn create_indexes(database: &Database) -> Result<()> {
    use mongodb::IndexModel;
    use mongodb::bson::doc;
    
    // 钱包地址索引
    let wallet_collection = database.collection::<mongodb::bson::Document>("wallet_addresses");
    let wallet_index = IndexModel::builder()
        .keys(doc! { "address": 1 })
        .options(mongodb::options::IndexOptions::builder()
            .unique(true)
            .build())
        .build();
    wallet_collection.create_index(wallet_index, None).await?;
    
    // 交易索引
    let transaction_collection = database.collection::<mongodb::bson::Document>("transactions");
    
    // 签名索引
    let signature_index = IndexModel::builder()
        .keys(doc! { "signature": 1 })
        .options(mongodb::options::IndexOptions::builder()
            .unique(true)
            .build())
        .build();
    transaction_collection.create_index(signature_index, None).await?;
    
    // 地址和时间索引
    let address_time_index = IndexModel::builder()
        .keys(doc! { 
            "from_address": 1,
            "timestamp": -1 
        })
        .build();
    transaction_collection.create_index(address_time_index, None).await?;
    
    let to_address_time_index = IndexModel::builder()
        .keys(doc! { 
            "to_address": 1,
            "timestamp": -1 
        })
        .build();
    transaction_collection.create_index(to_address_time_index, None).await?;
    
    Ok(())
}