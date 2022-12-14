#!/usr/bin/env bash
set -Eeuo pipefail

cargo build --release --target wasm32-wasi
for NODE in ./rust/*; do
    cp -v ./target/wasm32-wasi/release/${NODE##*/}.wasm $NODE/
done
