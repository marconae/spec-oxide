#!/usr/bin/env bash
set -euo pipefail

# Build and install spox, then update this project

echo "Building spox..."
cargo build --release

echo "Installing spox locally..."
cargo install --path . --force

echo "Updating project with spox init..."
spox init

echo "Done."
