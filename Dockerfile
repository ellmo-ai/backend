# Use the latest Rust nightly image as the base
FROM rust:1.78.0

# Set the working directory
WORKDIR /app

# Install cargo-watch
RUN cargo install cargo-watch

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to get dependencies cached
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build # --release

# Remove the dummy source file
RUN rm src/main.rs

# Copy the entire project
COPY . .

RUN cargo install diesel_cli --version 2.2.0 --no-default-features --features postgres

ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# Build the release binary
RUN cargo install --path .

# Expose the port your Rust server listens on
EXPOSE 3000

# Use cargo-watch to rebuild and run the server on changes
CMD sh -c "diesel setup --database-url $DATABASE_URL && cargo watch -x 'run'" #--release
