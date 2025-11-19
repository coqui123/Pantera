# Multi-stage build for Rust application optimized for Fly.io

# Stage 1: Builder
FROM rust:1.82-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY templates ./templates

# Build argument for features (default: web-ui)
# Fly.io will pass this via build_args if configured, otherwise defaults to web-ui
ARG FEATURES=web-ui

# Build the application with web-ui feature enabled by default
# To build without web-ui: docker build --build-arg FEATURES="" .
RUN echo "Building with features: ${FEATURES:-web-ui}" && \
    if [ -z "${FEATURES}" ] || [ "${FEATURES}" = "" ]; then \
        echo "Building without features"; \
        cargo build --release; \
    else \
        echo "Building with features: ${FEATURES}"; \
        cargo build --release --features "${FEATURES}"; \
    fi

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1000 appuser

# Create app directory
WORKDIR /app

# Create data directory for SQLite database with proper permissions
RUN mkdir -p /app/data && \
    chown -R appuser:appuser /app && \
    chmod 755 /app && \
    chmod 755 /app/data

# Copy binary from builder
COPY --from=builder /app/target/release/mango-data-service /usr/local/bin/mango-data-service

# Copy templates (needed if web-ui feature is enabled)
COPY --from=builder /app/templates /app/templates

# Set ownership
RUN chown -R appuser:appuser /app /usr/local/bin/mango-data-service

# Switch to non-root user
USER appuser

# Expose port (Fly.io will set PORT env var automatically)
EXPOSE 8080

# Run the application
# Fly.io sets PORT automatically via environment variable
CMD ["/usr/local/bin/mango-data-service"]

