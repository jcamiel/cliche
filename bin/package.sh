#!/usr/bin/env bash

APP=cliche-0.1.0
HOST=x86_64-unknown-linux-gnu
PACKAGE="$APP-$HOST"

mkdir -p "target/$PACKAGE/bin"
cp target/release/cliche "target/$PACKAGE/bin"
tar -C target -czvf "target/$PACKAGE.tar.gz" "$PACKAGE"
