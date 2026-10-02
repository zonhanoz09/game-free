#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

cargo build -p game-free --target wasm32-unknown-unknown --release
rm -rf dist/wasm/assets
mkdir -p dist/wasm/assets/audio dist/wasm/assets/fonts dist/wasm/assets/textures
wasm-bindgen --out-dir dist/wasm --target web target/wasm32-unknown-unknown/release/game-free.wasm

# Only copy assets referenced by the WASM client. Source models and alternate
# JPG previews remain in assets/ but are not part of the initial web payload.
cp assets/audio/*.wav dist/wasm/assets/audio/
cp assets/fonts/*.ttf dist/wasm/assets/fonts/
for texture in background knight archer mage assassin cleric; do
    cp "assets/textures/${texture}.png" "dist/wasm/assets/textures/"
done

cat > dist/wasm/assets/manifest.json <<'EOF'
{
  "version": 1,
  "initial": [
    "audio",
    "fonts",
    "textures/background.png",
    "textures/knight.png",
    "textures/archer.png",
    "textures/mage.png",
    "textures/assassin.png",
    "textures/cleric.png"
  ],
  "deferred": {
    "models": "/assets/models/",
    "preview_textures": "/assets/textures/"
  }
}
EOF
