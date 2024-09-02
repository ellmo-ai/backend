# First stage: Build the server
FROM rust:1.78.0 as builder

# Set the working directory
WORKDIR /app

# Install dependencies for buf and protoc
RUN apt-get update && apt-get install -y \
    curl \
    unzip \
    protobuf-compiler

# Install buf (latest release)
RUN curl -sSL \
    "https://github.com/bufbuild/buf/releases/download/v1.36.0/buf-Linux-x86_64" \
    -o /usr/local/bin/buf && \
    chmod +x /usr/local/bin/buf

# Copy the proto crate
COPY ./proto ./proto

# Set working directory to the server crate
WORKDIR /app/server

# Copy the entire server project
COPY ./server ./

# Build the release binary
RUN cargo install --path .

# Second stage: Create the final image for server
FROM rust:1.78.0-slim as final

RUN apt-get update && apt-get install -y \
    libpq5

# Copy the built binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server

# Expose the server port
EXPOSE 3000

# Run the server
CMD ["server"]

