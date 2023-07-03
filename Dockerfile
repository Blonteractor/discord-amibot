# Stage 1: Build the bot and the server to later shift to a minimal image
FROM rust:1.69 AS builder
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /app

# Copy the important files to the container
COPY bot bot
COPY amizone amizone
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# Clone the protobuf dependencies
RUN <<EOF
 [ "$(ls -A ./amizone/proto/googleapis)" ] && echo "Found protobuf deps" || git clone https://www.github.com/googleapis/googleapis.git ./amizone/proto/googleapis
EOF

# Build the Bot
RUN cargo build --release

# Stage 2: Create minimal ubuntu image, for running the bot
FROM ubuntu:latest

# Install necessary dependencies
RUN apt-get update && apt-get install -y curl

ENV GO_AMIZONE_VERSION 0.8.0

# Expose the port for Discord communication
EXPOSE 443

WORKDIR /app

# Copy the .env file from the build context into the Docker image
COPY .env /app/.env

# Copy the built Bot from the builder stage
COPY --from=builder /app/target/release/bot /app/bot
RUN chmod 755 /app/bot

# Download go-amizone using curl
RUN curl -LO https://github.com/ditsuke/go-amizone/releases/download/v$GO_AMIZONE_VERSION/amizone-api-server_linux_amd64 && \ 
mv amizone-api-server_linux_amd64 /app/amizone-api-server && chmod 755 /app/amizone-api-server

# Create entrypoint script
COPY <<EOF /app/entrypoint.sh
/app/amizone-api-server &
/app/bot
EOF
RUN chmod +x /app/entrypoint.sh


CMD ["/bin/sh", "/app/entrypoint.sh"]
