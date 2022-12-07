#!/usr/bin/env bash

# rust
for NODE in $(ls | grep -v .sh); do
    echo "Building $NODE"
    $(cd $NODE && cargo build --release --target wasm32-wasi && cp target/wasm32-wasi/release/$NODE.wasm ../$NODE)
done