# First stage: Build the server
FROM rust:1.78.0 AS builder

# Set the working directory
WORKDIR /app

# Set working directory to the sync crate
WORKDIR /app/sync

# Copy the entire server project
COPY ./sync ./

# Build the release binary for linux/amd64/v3
RUN cargo build --release

# Second stage: Create the final image for server
FROM rust:1.78.0-slim AS final

# Copy the built binary from the builder stage
COPY --from=builder /app/sync/target/release/sync /usr/local/bin/sync

# Expose the REST port
EXPOSE 3001

# Run the server
CMD ["sync"]