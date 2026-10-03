#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

cargo build -p game-free --target wasm32-unknown-unknown --release
rm -rf dist/wasm/assets
mkdir -p dist/wasm/assets/audio dist/wasm/assets/fonts dist/wasm/assets/textures
wasm-bindgen --out-dir dist/wasm --target web target/wasm32-unknown-unknown/release/game-free.wasm

# Keep the HTML import and the generated wasm-bindgen glue in the same cache
# version. Without this, a browser can combine an old JS glue file with a new
# WASM binary after a deployment.
WASM_VERSION="$(date -u +%Y%m%d%H%M%S)-$(sha256sum dist/wasm/game-free_bg.wasm | cut -c1-8)"
sed -i -E "s#\./game-free\.js(\?v=[^']*)?'#./game-free.js?v=${WASM_VERSION}'#" dist/wasm/index.html

# Only copy assets referenced by the WASM client. Source models and alternate
# JPG previews remain in assets/ but are not part of the initial web payload.
cp assets/audio/*.wav dist/wasm/assets/audio/
cp assets/fonts/*.ttf dist/wasm/assets/fonts/
for texture in background knight archer mage assassin cleric zhao_yun huang_zhong zhuge_liang zhang_he hua_tuo cao_cao dian_wei guo_jia sun_ce lu_xun da_qiao_xiao_qiao jia_xu; do
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
