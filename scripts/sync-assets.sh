#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

mkdir -p dist/wasm/assets/audio dist/wasm/assets/fonts dist/wasm/assets/textures
cp assets/audio/*.wav dist/wasm/assets/audio/
cp assets/fonts/*.ttf dist/wasm/assets/fonts/
for texture in background knight archer mage assassin cleric; do
    cp "assets/textures/${texture}.png" dist/wasm/assets/textures/
done
