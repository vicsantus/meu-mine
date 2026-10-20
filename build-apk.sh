#!/bin/sh
# Builds target/android-artifacts/release/apk/meu-mine.apk (debug-signed) with Docker.
# The notfl3/cargo-apk image ships Rust 1.79, too old for current macroquad deps,
# so a newer toolchain is installed inside the container first.
# cargo-quad-apk only reads Cargo.lock `version = 3`; newer cargo rewrites it to 4,
# so it is downgraded here (no git deps, so both versions hold the same data).
set -e
cd "$(dirname "$0")"
sed 's/^version = 4$/version = 3/' Cargo.lock > Cargo.lock.tmp && mv Cargo.lock.tmp Cargo.lock
docker run --rm --platform linux/amd64 -v "$PWD":/root/src -w /root/src notfl3/cargo-apk sh -c \
  'rustup toolchain install 1.90.0 --profile minimal --target aarch64-linux-android >/dev/null &&
   rustup default 1.90.0 && cargo quad-apk build --release'
