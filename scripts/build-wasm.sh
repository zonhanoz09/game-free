#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

cargo build -p game-free --target wasm32-unknown-unknown --release
mkdir -p dist/wasm
wasm-bindgen --out-dir dist/wasm --target web target/wasm32-unknown-unknown/release/game-free.wasm
mkdir -p dist/wasm/assets
cp -r assets/. dist/wasm/assets/
