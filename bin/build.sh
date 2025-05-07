#!/usr/bin/env bash

cargo build \
  --target x86_64-unknown-linux-musl \
  --target x86_64-unknown-linux-gnu \
  --release \
  --locked
