#!/usr/bin/env bash
#
# build-win.sh — cross-compile a PORTABLE Windows x64 build from Linux and
# package it as a zip the user can unzip and run on a Windows PC.
#
# This is the Windows counterpart to arm-char-gen.sh (which builds + launches
# the Linux production binary). It cross-compiles the `x86_64-pc-windows-msvc`
# target with cargo-xwin, stages the rules resources next to the .exe, and zips
# the result.
#
# One-time host setup (Ubuntu/Debian), run manually with sudo:
#     sudo apt-get install -y clang lld llvm zip
# Everything else (Rust target, cargo-xwin, the clang-cl shim) is provisioned
# automatically below — none of it needs root.
#
# Requirements on the TARGET Windows machine: Windows 10 (64-bit) or newer with
# the Microsoft WebView2 runtime (preinstalled on Win11 and current Win10).
#
# Usage: ./build-win.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

TARGET="x86_64-pc-windows-msvc"

# The toolchain is not on PATH in fresh shells; bring cargo and node into scope.
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
export NVM_DIR="$HOME/.nvm"
# shellcheck disable=SC1091
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"

# cargo-xwin drives clang in its MSVC-compatible "cl" mode. Clang selects that
# mode from the name it is invoked as, but Ubuntu's clang package ships no
# `clang-cl` file — so provide one as a symlink on PATH (~/.cargo/bin is already
# on PATH and writable without root).
if ! command -v clang-cl >/dev/null 2>&1; then
  echo ">> Creating clang-cl shim in ~/.cargo/bin..."
  ln -sf "$(command -v clang)" "$HOME/.cargo/bin/clang-cl"
fi

# Ensure the Windows Rust target and cross-compiler are present (no-ops if so).
echo ">> Ensuring Rust target $TARGET is installed..."
rustup target add "$TARGET" >/dev/null

if ! command -v cargo-xwin >/dev/null 2>&1; then
  echo ">> Installing cargo-xwin (first run only)..."
  cargo install --locked cargo-xwin
fi

# Cross-compile. cargo-xwin downloads + caches the MSVC CRT and Windows SDK on
# first use. --no-bundle yields just the runnable .exe (no installer); we want
# a portable layout, not an installed one.
echo ">> Cross-compiling Windows binary ($TARGET)..."
cargo tauri build --runner cargo-xwin --target "$TARGET" --no-bundle

# Read the product version from the Tauri config so the zip name tracks it.
VERSION="$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' \
  crates/arm-app/tauri.conf.json | head -1)"
VERSION="${VERSION:-0.0.0}"

# Hyphenated form of the Tauri productName: the installers get the spaced name,
# but the portable layout keeps hyphens so paths stay easy to type.
APP="Ars-Magica-Character-Generator"
NAME="${APP}-${VERSION}-win-x64-portable"
STAGE="dist-win/${NAME}"

echo ">> Staging portable bundle at $STAGE..."
rm -rf "dist-win"
mkdir -p "$STAGE/rules"
cp "target/$TARGET/release/arm-app.exe" "$STAGE/${APP}.exe"
# A non-bundled binary resolves BaseDirectory::Resource to its own directory, so
# the rules data the app loads at startup must sit next to the executable.
cp -R "rules/core" "$STAGE/rules/core"
cp -R "rules/i18n" "$STAGE/rules/i18n"
# Attribution has to travel with the data it describes. Bundled installers get
# these from tauri.conf.json (bundle.resources + licenseFile), which this
# --no-bundle path never runs, so copy them explicitly. The per-directory
# LICENSE files ride along with the cp -R above.
cp "rules/NOTICE.md" "$STAGE/rules/NOTICE.md"
cp "LICENSE" "$STAGE/LICENSE.txt"

# Run instructions + system requirements travel inside the zip.
cat > "$STAGE/README.txt" <<EOF
Ars Magica 5th Edition Character Generator — Portable (Windows x64)
Version ${VERSION}

HOW TO RUN
  1. Unzip this folder anywhere (Desktop, USB stick, etc.).
  2. Double-click  ${APP}.exe

  Keep ${APP}.exe and the "rules" folder together in the same
  directory — the app loads its rules data from .\\rules at startup.
  Moving the .exe out on its own will stop it from launching correctly.

REQUIREMENTS
  - Windows 10 (64-bit) or newer.
  - Microsoft WebView2 Runtime. This is preinstalled on Windows 11 and
    on up-to-date Windows 10, so normally nothing extra is needed.
    If the window stays blank, install the Evergreen WebView2 Runtime
    (free, from Microsoft) and relaunch:
    https://developer.microsoft.com/microsoft-edge/webview2/

NOTES
  - This is a portable build: it installs nothing and writes no registry
    entries. Delete the folder to remove it.
  - Windows SmartScreen may warn on first launch because the binary is
    unsigned ("More info" -> "Run anyway").

LICENSING
  The application is MIT licensed (see LICENSE.txt), and so is the form of the
  rules data — its JSON schema, identifiers and the mechanics in rules\\core.
  The Ars Magica rules TEXT in rules\\i18n is (c) 1993-2024 Trident, Inc. d/b/a
  Atlas Games, used under CC BY-SA 4.0. See rules\\NOTICE.md for the full
  attribution and terms.
EOF

echo ">> Zipping..."
( cd "dist-win" && zip -r -q "${NAME}.zip" "$NAME" )

echo ">> Done: dist-win/${NAME}.zip"
ls -lh "dist-win/${NAME}.zip"
