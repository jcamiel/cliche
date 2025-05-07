#!/usr/bin/env bash
set -Eeuo pipefail

sudo apt-get update
sudo apt-get install --yes --no-install-recommends \
    curl

# Install Rust
echo "Installing rust"
curl https://sh.rustup.rs -sSfkL | sh -s -- -y
rustc --version
cargo --version

# Install for cross compilation `x86_64-unknown-linux-musl`
echo "Installing cross compilation"
sudo apt-get install --yes --no-install-recommends \
    musl-tools
rustup target add x86_64-unknown-linux-musl
