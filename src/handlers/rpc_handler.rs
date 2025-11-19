use axum::{
    extract::{Json, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::models::{RpcResponse, Transaction};
use crate::services::blockchain::BlockchainScanner;

#[derive(Deserialize)]
struct TransactionQuery {
    address: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Deserialize)]
struct AddAddressRequest {
    address: String,
    label: Option<String>,
}

#[derive(Serialize)]
struct AddressResponse {
    addresses: Vec<String>,
}

pub async fn start_rpc_server(scanner: Arc<RwLock<BlockchainScanner>>) {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/transactions", get(get_transactions))
        .route("/addresses", get(get_addresses))
        .route("/addresses", post(add_address))
        .route("/addresses/:address", axum::routing::delete(remove_address))
        .with_state(scanner);

    let addr: std::net::SocketAddr = "0.0.0.0:8080".parse().unwrap();
    info!("RPC server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> impl IntoResponse {
    Json(RpcResponse::success("healthy"))
}

async fn get_transactions(
    State(scanner): State<Arc<RwLock<BlockchainScanner>>>,
    Query(query): Query<TransactionQuery>,
) -> impl IntoResponse {
    match scanner
        .read()
        .await
        .get_transactions(query.address, query.limit, query.offset)
        .await
    {
        Ok(transactions) => Json(RpcResponse::success(transactions)),
        Err(e) => {
            error!("Failed to get transactions: {}", e);
            Json(RpcResponse::<Vec<Transaction>>::error(e.to_string()))
        }
    }
}

async fn get_addresses(State(scanner): State<Arc<RwLock<BlockchainScanner>>>) -> impl IntoResponse {
    let addresses = scanner.read().await.get_watched_addresses().await;
    Json(RpcResponse::success(AddressResponse { addresses }))
}

async fn add_address(
    State(scanner): State<Arc<RwLock<BlockchainScanner>>>,
    Json(request): Json<AddAddressRequest>,
) -> impl IntoResponse {
    match scanner
        .read()
        .await
        .add_watched_address(request.address.clone())
        .await
    {
        Ok(_) => Json(RpcResponse::success(
            "Address added successfully".to_string(),
        )),
        Err(e) => {
            error!("Failed to add address: {}", e);
            Json(RpcResponse::<String>::error(e.to_string()))
        }
    }
}

async fn remove_address(
    State(scanner): State<Arc<RwLock<BlockchainScanner>>>,
    axum::extract::Path(address): axum::extract::Path<String>,
) -> impl IntoResponse {
    match scanner
        .read()
        .await
        .remove_watched_address(address.clone())
        .await
    {
        Ok(_) => Json(RpcResponse::success(
            "Address removed successfully".to_string(),
        )),
        Err(e) => {
            error!("Failed to remove address: {}", e);
            Json(RpcResponse::<String>::error(e.to_string()))
        }
    }
}
