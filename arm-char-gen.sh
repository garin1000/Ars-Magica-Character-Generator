#!/usr/bin/env bash
#
# arm-char-gen.sh — build and launch the character generator in PRODUCTION mode,
# the way it behaves when shipped, for manual testing.
#
# Production mode means the app serves its bundled frontend from disk instead of
# the Vite dev server. A plain `cargo build --release` would still run in DEV
# mode (blank page, "Could not connect to localhost"), so we build through the
# Tauri CLI. `--no-bundle` skips installer packaging — we only need the runnable
# binary — while keeping the production code path identical to a shipped build.
#
# Usage: ./arm-char-gen.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

# The toolchain is not on PATH in fresh shells; bring cargo and node into scope.
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
export NVM_DIR="$HOME/.nvm"
# shellcheck disable=SC1091
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"

# shellcheck disable=SC1091
. "$ROOT/scripts/stage-rules.sh"

echo ">> Building production frontend + binary (cargo tauri build --no-bundle)..."
cargo tauri build --no-bundle

# A non-bundled binary resolves BaseDirectory::Resource to its own directory, so
# the rules data the app loads at startup must sit next to the executable. The
# bundle/installer would place these for us; here we stage them by hand.
echo ">> Staging rules resources next to the release binary..."
stage_rules "target/release/rules"

echo ">> Launching arm-char-gen (production)..."
exec "target/release/arm-app"
