# First stage: Build the server
FROM rust:1.78.0 AS builder

# Set the working directory
WORKDIR /app

# Install dependencies for buf and protoc
RUN apt-get update && apt-get install -y \
    curl \
    unzip \
    protobuf-compiler \
    build-essential

# Install buf (latest release)
RUN curl -sSL \
    "https://github.com/bufbuild/buf/releases/download/v1.36.0/buf-Linux-x86_64" \
    -o /usr/local/bin/buf && \
    chmod +x /usr/local/bin/buf

# Copy the proto crate
COPY ./proto ./proto
COPY ./db ./db

# Set working directory to the server crate
WORKDIR /app/server

# Copy the entire server project
COPY ./server ./

RUN rustup target add x86_64-unknown-linux-gnu

# Build the release binary for linux/amd64/v3
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Second stage: Create the final image for server
FROM rust:1.78.0-slim AS final

RUN apt-get update && apt-get install -y \
    libpq5

# Copy the built binary from the builder stage
COPY --from=builder /app/server/target/x86_64-unknown-linux-gnu/release/server /usr/local/bin/server

# Expose the REST and gRPC ports
EXPOSE 3000
EXPOSE 50051

# Run the server
CMD ["server"]