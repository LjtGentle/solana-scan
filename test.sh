#!/bin/bash

# Test script for Solana blockchain scanner

echo "=== Solana Blockchain Scanner Test ==="
echo

# Create test environment file
cat > .env.test << EOF
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_RPC_TIMEOUT=30
MONGODB_URI=mongodb://localhost:27017/solana_scanner_test
KAFKA_BROKERS=localhost:9092
KAFKA_TOPIC=solana_transactions_test
SCAN_BATCH_SIZE=10
SCAN_INTERVAL_SECONDS=5
MAX_CONCURRENT_REQUESTS=10
RPC_SERVER_PORT=8080
WEBSOCKET_PORT=8081
EOF

echo "✓ Created test environment file"

# Check if Rust is available
if command -v cargo &> /dev/null; then
    echo "✓ Rust/Cargo is available"
    
    # Run tests
    echo "Running cargo tests..."
    cargo test
    
    if [ $? -eq 0 ]; then
        echo "✓ All tests passed"
    else
        echo "✗ Tests failed"
        exit 1
    fi
    
    # Check if the project builds
    echo "Building project..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        echo "✓ Project builds successfully"
    else
        echo "✗ Build failed"
        exit 1
    fi
    
else
    echo "✗ Rust/Cargo not found"
    exit 1
fi

echo
echo "=== Test Summary ==="
echo "✓ Environment setup complete"
echo "✓ Tests passing"
echo "✓ Project builds successfully"
echo
echo "To run the application:"
echo "1. Set up MongoDB and Kafka (see docker-compose.yml)"
echo "2. Copy .env.test to .env and modify as needed"
echo "3. Run: cargo run"
echo
echo "API endpoints will be available at:"
echo "- Health check: http://localhost:8080/health"
echo "- Transactions: http://localhost:8080/transactions"
echo "- Addresses: http://localhost:8080/addresses"
echo "- WebSocket: ws://localhost:8081"