#!/bin/bash

# Define an array of child crate names
CRATES=("server" "exec")

# Function to clean up background processes on exit
cleanup() {
  echo "Terminating background processes..."
  kill 0
}

# Trap SIGINT (Ctrl+C) and SIGTERM signals to run the cleanup function
trap cleanup SIGINT SIGTERM

# Loop through each crate in the array and run cargo watch concurrently
for CRATE in "${CRATES[@]}"; do
  (
    cd "$CRATE" || exit
    cargo watch -x "run --bin $CRATE"
  ) &
done

# Wait for all cargo watch processes to finish
wait

