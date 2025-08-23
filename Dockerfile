# Stage 1: Builder using a secure, minimal base image
FROM rust:1.89-slim-bookworm AS builder

# Update packages and install build dependencies in a single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./ .sqlx/
COPY migrations ./migrations

# Create a dummy project to build and cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"Dependencies built!\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/metered_api_server*

# Copy the actual source code
COPY src ./src

# Build the application, leveraging the cached dependency layer
RUN cargo build --release

# ---
# Stage 2: Runtime using a minimal, non-root base image
FROM debian:bookworm-slim

# Install only necessary runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd --create-home --shell /bin/bash appuser

WORKDIR /home/appuser

# Copy the compiled binary and migrations from the builder stage
COPY --from=builder --chown=appuser:appuser /usr/src/app/target/release/metered-api-server .
COPY --from=builder --chown=appuser:appuser /usr/src/app/migrations ./migrations

# Switch to the non-root user
USER appuser

# Expose the application port
EXPOSE 3030

# Health check to ensure the application is running
HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3030/health || exit 1

# Command to run the application
CMD ["./metered-api-server"]