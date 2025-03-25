# Stage 1: Build the Rust application
FROM rust:1.85.0-slim-bookworm AS builder

# Install dependencies required for building
RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --verbose
# RUN ls -al /app/target
# RUN ls -al /app/target/release || echo "Release directory not found"

# Stage 2: Create the final image
FROM debian:bookworm-slim

# Install runtime dependencies
# RUN apt-get update && apt-get install -y ca-certificates libpq-dev && libssl3 && rm -rf /var/lib/apt/lists/*
# Install runtime dependencies
RUN apt-get update && apt-get install -y \
  ca-certificates \
  libpq-dev \
  libssl3 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/tokencheck-backend /app/tokencheck-backend

# Expose the port the app will run on
EXPOSE 8080

# Set the entrypoint to run the binary
CMD ["./tokencheck-backend"]
