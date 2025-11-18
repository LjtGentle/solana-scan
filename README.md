# Solana Blockchain Scanner

A high-performance Rust-based Solana blockchain scanning service that monitors wallet addresses and records transactions to MongoDB with Kafka message streaming.

## Features

- **Blockchain Scanning**: Scan Solana blockchain with custom or latest block height
- **Wallet Monitoring**: Monitor up to 100,000 wallet addresses from database
- **Transaction Recording**: Record native coin and token transactions to MongoDB
- **Message Streaming**: Push transaction data to Apache Kafka
- **RESTful API**: Provide RPC endpoints for transaction queries and address management
- **WebSocket Support**: Real-time transaction notifications via WebSocket
- **High Performance**: Built with Rust and Tokio for high concurrency
- **Docker Support**: Complete containerized deployment with Docker Compose

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Solana RPC    │    │   WebSocket     │    │   REST API      │
│   Connection    │    │   Server        │    │   Server        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │  Blockchain     │
                    │  Scanner        │
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   MongoDB       │    │   Kafka         │    │   Wallet DB     │
│   Transactions    │    │   Message       │    │   Addresses     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust 1.70+ (for local development)
- Solana RPC endpoint

### Environment Configuration

Create a `.env` file with the following variables:

```bash
# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_RPC_TIMEOUT=30

# MongoDB Configuration
MONGODB_URI=mongodb://mongo:27017/solana_scanner

# Kafka Configuration
KAFKA_BROKERS=kafka:9092
KAFKA_TOPIC=solana_transactions

# Scanning Configuration
SCAN_BATCH_SIZE=100
SCAN_INTERVAL_SECONDS=10
MAX_CONCURRENT_REQUESTS=50

# Server Configuration
RPC_SERVER_PORT=8080
WEBSOCKET_PORT=8081
```

### Docker Deployment

1. Clone the repository:
```bash
git clone <repository-url>
cd solana-scan
```

2. Start all services with Docker Compose:
```bash
docker-compose up -d
```

3. Verify services are running:
```bash
docker-compose ps
```

4. Check logs:
```bash
docker-compose logs -f solana-scanner
```

### Local Development

1. Install Rust dependencies:
```bash
cargo build
```

2. Start MongoDB and Kafka locally:
```bash
docker-compose up -d mongo kafka zookeeper
```

3. Run the application:
```bash
cargo run
```

## API Endpoints

### Health Check
```http
GET /health
```

### Get Transactions
```http
GET /transactions?address=<address>&limit=<limit>&offset=<offset>
```

### Get Monitored Addresses
```http
GET /addresses
```

### Add Address to Monitor
```http
POST /addresses
Content-Type: application/json

{
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "label": "My Wallet"
}
```

### Remove Address from Monitoring
```http
DELETE /addresses/<address>
```

## WebSocket API

Connect to `ws://localhost:8081` for real-time transaction notifications.

### Subscribe to Address
```json
{
  "type": "subscribe",
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}
```

### Unsubscribe from Address
```json
{
  "type": "unsubscribe",
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}
```

## Performance Configuration

The application is optimized for high concurrency:

- **100,000+ Wallet Addresses**: Efficient database indexing and batch processing
- **High Concurrent Requests**: Configurable connection pooling and rate limiting
- **Real-time Processing**: WebSocket connections for live transaction updates
- **Scalable Architecture**: Microservice design with message queuing

## Database Schema

### Wallet Addresses Collection
```json
{
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "label": "My Wallet",
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Transactions Collection
```json
{
  "signature": "5Z2...",
  "block_number": 123456789,
  "block_hash": "ABC...",
  "from_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "to_address": "8yLM...",
  "amount": "1000000000",
  "token_address": "So11111111111111111111111111111111111111112",
  "transaction_type": "token_transfer",
  "timestamp": "2024-01-01T00:00:00Z",
  "created_at": "2024-01-01T00:00:00Z"
}
```

## Monitoring and Logging

The application uses structured logging with the `tracing` crate:

- **Info Level**: General application events and API requests
- **Debug Level**: Detailed blockchain scanning operations
- **Error Level**: Critical errors and failures
- **Performance Metrics**: Request timing and throughput statistics

## Security Considerations

- Rate limiting on API endpoints
- Input validation for all user inputs
- Secure WebSocket connections (WSS support)
- Database connection encryption
- Kafka message authentication (configurable)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
- Create an issue on GitHub
- Check the troubleshooting guide
- Review the API documentation