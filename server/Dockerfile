FROM rust:1.83-slim AS builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY server/ server/
# Create dummy src-tauri so workspace resolves
RUN mkdir -p src-tauri/src && echo "fn main() {}" > src-tauri/src/main.rs
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml

# Build server
RUN cargo build --release -p commit-cat-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/commit-cat-server /usr/local/bin/

EXPOSE 3000
CMD ["commit-cat-server"]
