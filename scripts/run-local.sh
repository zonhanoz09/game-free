#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

./scripts/build-wasm.sh
docker compose -f deploy/docker/docker-compose.yml up --build
