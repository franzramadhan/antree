#!/usr/bin/env bash
# Build antree-server.wasm: the web build (dist/) embedded in a wasi:http
# component the runwasi wasmtime shim can run. wasip1 plus the wasmtime 36
# proxy adapter, because the shim serves WASI 0.2.6 and Rust's wasm32-wasip2
# std imports newer. Needs rustup targets wasm32-unknown-unknown and
# wasm32-wasip1, and wasm-tools on PATH.
set -euo pipefail
cd "$(dirname "$0")"

ADAPTER_VERSION=v36.0.10
ADAPTER_SHA256=7646182f7b40db5f6beb1516729423e6c88c7f59a1397c19ef496921f2d43a7e
adapter=target/wasi_snapshot_preview1.proxy-$ADAPTER_VERSION.wasm

if [ ! -f "$adapter" ]; then
  mkdir -p target
  curl -fsSL -o "$adapter.tmp" \
    "https://github.com/bytecodealliance/wasmtime/releases/download/$ADAPTER_VERSION/wasi_snapshot_preview1.proxy.wasm"
  echo "$ADAPTER_SHA256  $adapter.tmp" | shasum -a 256 -c - >/dev/null
  mv "$adapter.tmp" "$adapter"
fi

make web
cargo build --release --locked --target wasm32-wasip1 \
  --manifest-path server/Cargo.toml --target-dir target/server
wasm-tools component new target/server/wasm32-wasip1/release/antree-server.wasm \
  --adapt "wasi_snapshot_preview1=$adapter" -o antree-server.wasm

# The shim serves WASI 0.2.6; fail on anything newer.
if wasm-tools component wit antree-server.wasm | grep -E '@0\.2\.([7-9]|[1-9][0-9])'; then
  echo "antree-server.wasm imports a WASI version newer than 0.2.6" >&2
  exit 1
fi
ls -l antree-server.wasm
