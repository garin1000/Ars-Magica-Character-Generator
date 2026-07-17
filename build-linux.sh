#!/usr/bin/env bash
#
# build-linux.sh — build a PORTABLE Linux x64 build and package it as a tgz the
# user can extract and run on another Linux PC.
#
# This is the Linux counterpart to build-win.sh (which cross-compiles + zips a
# portable Windows build). Linux is the host platform, so no cross-compilation
# is needed: it builds the native production binary the same way arm-char-gen.sh
# does, stages the rules resources next to it, and tars the result.
#
# Requirements on the TARGET Linux machine: a glibc-based x86_64 distro with
# WebKitGTK 4.1 installed (see README.txt inside the archive for the package
# names per distro). The binary is dynamically linked, so the archive is not
# fully self-contained — the system WebView (WebKitGTK) must be present.
#
# Usage: ./build-linux.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

# The toolchain is not on PATH in fresh shells; bring cargo and node into scope.
# shellcheck disable=SC1091
. "$HOME/.cargo/env"
export NVM_DIR="$HOME/.nvm"
# shellcheck disable=SC1091
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"

# Build the production frontend + native release binary. --no-bundle yields just
# the runnable binary (no installer); we want a portable layout, not an
# installed one — same production code path as a shipped build.
echo ">> Building production frontend + binary (cargo tauri build --no-bundle)..."
cargo tauri build --no-bundle

# Read the product version from the Tauri config so the archive name tracks it.
VERSION="$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' \
  crates/arm-app/tauri.conf.json | head -1)"
VERSION="${VERSION:-0.0.0}"

NAME="arm-char-gen-${VERSION}-linux-x64-portable"
STAGE="dist-linux/${NAME}"

echo ">> Staging portable bundle at $STAGE..."
rm -rf "dist-linux"
mkdir -p "$STAGE/rules"
cp "target/release/arm-app" "$STAGE/arm-char-gen"
chmod +x "$STAGE/arm-char-gen"
# A non-bundled binary resolves BaseDirectory::Resource to its own directory, so
# the rules data the app loads at startup must sit next to the executable.
cp -R "rules/core" "$STAGE/rules/core"
cp -R "rules/i18n" "$STAGE/rules/i18n"

# Run instructions + system requirements travel inside the archive.
cat > "$STAGE/README.txt" <<EOF
Ars Magica 5th Edition Character Generator — Portable (Linux x64)
Version ${VERSION}

HOW TO RUN
  1. Extract this archive anywhere (home folder, USB stick, etc.):
       tar -xzf ${NAME}.tar.gz
  2. Run the launcher:
       ./${NAME}/arm-char-gen

  Keep the arm-char-gen binary and the "rules" folder together in the same
  directory — the app loads its rules data from ./rules at startup. Moving the
  binary out on its own will stop it from launching correctly.

REQUIREMENTS
  - A 64-bit (x86_64) glibc-based Linux distribution.
  - WebKitGTK 4.1 and its GTK 3 dependencies. This is the system WebView the
    app renders into; it is not bundled. Install it with your package manager:

      Debian/Ubuntu:  sudo apt install libwebkit2gtk-4.1-0
      Fedora:         sudo dnf install webkit2gtk4.1
      Arch:           sudo pacman -S webkit2gtk-4.1

    If the window stays blank or the binary fails to start with a message about
    a missing libwebkit2gtk / libgtk shared library, install the package above
    and relaunch.

NOTES
  - This is a portable build: it installs nothing and writes no system files.
    Delete the folder to remove it.
  - The binary is unsigned. If your desktop refuses to launch it from a file
    manager, mark it executable (chmod +x arm-char-gen) or run it from a
    terminal as shown above.
EOF

echo ">> Creating tarball..."
( cd "dist-linux" && tar -czf "${NAME}.tar.gz" "$NAME" )

echo ">> Done: dist-linux/${NAME}.tar.gz"
ls -lh "dist-linux/${NAME}.tar.gz"
