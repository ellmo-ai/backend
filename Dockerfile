# Use the latest Rust nightly image as the base
FROM rust:1.76.0

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to get dependencies cached
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Remove the dummy source file
RUN rm src/main.rs

# Copy the entire project
COPY . .

# Build the release binary
RUN cargo install --path .

# Expose the port your Rust server listens on
EXPOSE 3000

# Set the entry point to run the Rust server
CMD ["ollyllm"]