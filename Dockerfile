FROM rust:1.75 as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟项目以缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# 复制源代码
COPY src ./src

# 重新构建项目
RUN touch src/main.rs && cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/solana-scan /app/solana-scan

# 复制配置文件
COPY .env /app/.env

EXPOSE 8080 8081

CMD ["./solana-scan"]