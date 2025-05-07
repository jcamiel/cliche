#!/usr/bin/env bash

app=cliche
version=0.1.0


targets=("x86_64-unknown-linux-musl" "x86_64-unknown-linux-gnu")

for target in "${targets[@]}"; do
  package="$app-$version-$target"
  mkdir -p "target/package/$package/bin"
  cp "target/$target/release/cliche" "target/package/$package/bin"
  tar -C target/package -czvf "target/$package.tar.gz" "$package"
done
