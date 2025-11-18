use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use tracing_subscriber;

mod config;
mod models;
mod services;
mod handlers;
mod utils;
mod db;

use config::AppConfig;
use services::{blockchain::BlockchainScanner, websocket::WebSocketManager};
use handlers::{rpc_handler, websocket_handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Solana blockchain scanner service...");

    // 加载配置
    let config = AppConfig::load()?;
    
    // 初始化数据库连接
    let db_client = db::init_mongodb(&config.mongodb_uri).await?;
    
    // 创建区块链扫描器
    let scanner = Arc::new(RwLock::new(BlockchainScanner::new(
        config.solana_rpc_url.clone(),
        db_client.clone(),
        config.kafka_config.clone(),
    ).await?));

    // 创建WebSocket管理器
    let ws_manager = Arc::new(RwLock::new(WebSocketManager::new()));

    // 启动区块链扫描任务
    let scanner_clone = scanner.clone();
    let scan_task = tokio::spawn(async move {
        if let Err(e) = scanner_clone.read().await.start_scanning().await {
            error!("Blockchain scanning error: {}", e);
        }
    });

    // 启动WebSocket服务
    let ws_manager_clone = ws_manager.clone();
    let ws_task = tokio::spawn(async move {
        websocket_handler::start_websocket_server(ws_manager_clone).await;
    });

    // 启动RPC服务
    let rpc_task = tokio::spawn(async move {
        rpc_handler::start_rpc_server(scanner.clone()).await;
    });

    // 等待所有任务完成
    tokio::select! {
        _ = scan_task => info!("Scanner task completed"),
        _ = ws_task => info!("WebSocket task completed"),
        _ = rpc_task => info!("RPC task completed"),
    }

    Ok(())
}
