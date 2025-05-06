#!/usr/bin/env bash
set -Eeuo pipefail

curl https://sh.rustup.rs -sSfkL | sh -s -- -y

~/.cargo/bin/rustc --version
~/.cargo/bin/cargo --version
