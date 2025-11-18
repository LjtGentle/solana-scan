# Solana 区块链扫描器

一个高性能的、基于 Rust 的 Solana 区块链扫描服务，用于监控钱包地址并将交易记录写入 MongoDB，同时通过 Kafka 进行消息流式传输。

## 特性

- 区块扫描：支持从自定义高度或最新高度进行扫描
- 钱包监控：从数据库监控最多 100,000 个钱包地址
- 交易记录：将原生币与代币交易记录写入 MongoDB
- 消息流：将交易数据推送到 Apache Kafka
- RESTful API：提供交易查询与地址管理的 RPC 接口
- WebSocket：实时交易通知
- 高性能：基于 Rust 与 Tokio 的高并发架构
- Docker 支持：通过 Docker Compose 完整容器化部署

## 架构

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
│   Transactions  │    │   Message       │    │   Addresses     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 快速开始

### 前置条件

- Docker 与 Docker Compose
- Rust 1.70+（用于本地开发）
- Solana RPC 端点

### 环境配置

创建一个 `.env` 文件并填入以下变量：

```bash
# Solana 配置
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_RPC_TIMEOUT=30

# MongoDB 配置
MONGODB_URI=mongodb://mongo:27017/solana_scanner

# Kafka 配置
KAFKA_BROKERS=kafka:9092
KAFKA_TOPIC=solana_transactions

# 扫描配置
SCAN_BATCH_SIZE=100
SCAN_INTERVAL_SECONDS=10
MAX_CONCURRENT_REQUESTS=50

# 服务配置
RPC_SERVER_PORT=8080
WEBSOCKET_PORT=8081
```

### Docker 部署

1. 克隆仓库：
```bash
git clone <repository-url>
cd solana-scan
```

2. 使用 Docker Compose 启动全部服务：
```bash
docker-compose up -d
```

3. 验证服务是否运行：
```bash
docker-compose ps
```

4. 查看日志：
```bash
docker-compose logs -f solana-scanner
```

### 本地开发

1. 安装 Rust 依赖：
```bash
cargo build
```

2. 本地启动 MongoDB 与 Kafka：
```bash
docker-compose up -d mongo kafka zookeeper
```

3. 运行应用：
```bash
cargo run
```

## API 接口

### 健康检查
```http
GET /health
```

### 获取交易列表
```http
GET /transactions?address=<address>&limit=<limit>&offset=<offset>
```

### 获取已监控地址
```http
GET /addresses
```

### 添加监控地址
```http
POST /addresses
Content-Type: application/json

{
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "label": "我的钱包"
}
```

### 移除监控地址
```http
DELETE /addresses/<address>
```

## WebSocket 接口

连接到 `ws://localhost:8081` 获取实时交易通知。

### 订阅地址
```json
{
  "type": "subscribe",
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}
```

### 取消订阅地址
```json
{
  "type": "unsubscribe",
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
}
```

## 性能配置

该应用针对高并发进行了优化：

- 100,000+ 地址监控：高效的数据库索引与批处理
- 高并发请求：可配置的连接池与限流策略
- 实时处理：通过 WebSocket 提供实时交易更新
- 可扩展架构：基于消息队列的微服务设计

## 数据库结构

### 钱包地址集合
```json
{
  "address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
  "label": "我的钱包",
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### 交易集合
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

## 监控与日志

应用使用 `tracing` 进行结构化日志：

- Info：应用事件与 API 请求
- Debug：区块链扫描的详细过程
- Error：关键错误与故障
- 性能指标：请求耗时与吞吐统计

## 安全考量

- API 接口限流
- 全量输入校验
- 支持安全 WebSocket（WSS）
- 数据库连接加密
- Kafka 消息鉴权（可配置）

## 贡献指南

1. Fork 本仓库
2. 创建功能分支
3. 完成修改
4. 为新功能添加测试
5. 提交 Pull Request

## 许可证

本项目采用 MIT 许可证，详情参见 LICENSE 文件。

## 支持

若遇到问题或有疑问：
- 在 GitHub 创建 Issue
- 查阅故障排查指南
- 查看 API 文档