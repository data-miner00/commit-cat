FROM rust:latest AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src-tauri with both main.rs and lib.rs so workspace resolves
RUN mkdir -p src-tauri/src && \
    echo "fn main() {}" > src-tauri/src/main.rs && \
    echo "" > src-tauri/src/lib.rs
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml

# Copy only the cat sprites required by include_bytes! in badge.rs
# These are embedded at compile time; the runtime image does not need public/
COPY public/assets/cat/ public/assets/cat/

# Copy source code last (most frequently changed)
COPY crates/ crates/
COPY server/ server/

# Build server only
RUN cargo build --release -p commit-cat-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/commit-cat-server /usr/local/bin/

EXPOSE 3000
CMD ["commit-cat-server"]
