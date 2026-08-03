FROM rust:1.77-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo manifests first for dependency caching
COPY plugins/solana-pos-core/pos-core-logic/Cargo.toml pos-core-logic/
COPY pos-backend/Cargo.toml pos-backend/

# Create dummy source files for dependency caching
RUN mkdir -p pos-core-logic/src && echo "pub fn placeholder() {}" > pos-core-logic/src/lib.rs && \
    mkdir -p pos-backend/src && echo "fn main() {}" > pos-backend/src/main.rs

# Build dependencies (cached layer)
RUN cd pos-backend && cargo build --release 2>/dev/null || true

# Copy actual source code
COPY plugins/solana-pos-core/pos-core-logic/ pos-core-logic/
COPY pos-backend/ pos-backend/

# Build the actual binary
RUN cd pos-backend && cargo build --release --bin pos-backend

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/pos-backend/target/release/pos-backend /app/pos-backend

# Create data directory
RUN mkdir -p /app/data && chmod 777 /app/data

EXPOSE 8080

CMD ["/app/pos-backend"]
