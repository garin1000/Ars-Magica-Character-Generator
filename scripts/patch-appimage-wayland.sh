#!/usr/bin/env bash
#
# patch-appimage-wayland.sh — post-build patch that keeps the AppImage working
# on modern Wayland desktops.
#
# Tauri's bundler (via linuxdeploy) over-bundles libwayland into the AppImage,
# which makes the host's Mesa libEGL fail with "Could not create default EGL
# display: EGL_BAD_PARAMETER" and leaves the user with a white window. This
# script installs scripts/appimage/wayland-compat-hook.sh into the AppImage's
# apprun-hooks/ and makes AppRun source it before launching the app; the hook
# itself explains the mechanism.
#
# Usage:
#   ./scripts/patch-appimage-wayland.sh                 # patch every AppImage
#                                                       # in target/release/bundle/appimage
#   ./scripts/patch-appimage-wayland.sh path/to.AppImage
#   ./scripts/patch-appimage-wayland.sh --appdir path/to/Some.AppDir
#
# Repacking needs appimagetool. It is taken from $APPIMAGETOOL, from PATH, or —
# since a Tauri AppImage build downloads it anyway — from
# ~/.cache/tauri/linuxdeploy-plugin-appimage.AppImage.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOK_SOURCE="$ROOT/scripts/appimage/wayland-compat-hook.sh"
HOOK_NAME="arm-wayland-compat.sh"

die() {
  echo "patch-appimage-wayland: $1" >&2
  exit 1
}

# Installs the hook into an extracted AppDir and wires it into AppRun. Safe to
# run twice: the second run finds the source line already there and stops.
patch_appdir() {
  local appdir="$1"
  [ -d "$appdir" ] || die "not a directory: $appdir"
  [ -f "$appdir/AppRun" ] || die "no AppRun in $appdir — not an AppDir?"
  [ -d "$appdir/apprun-hooks" ] || die "no apprun-hooks/ in $appdir — not a linuxdeploy AppDir?"

  cp "$HOOK_SOURCE" "$appdir/apprun-hooks/$HOOK_NAME"
  chmod +x "$appdir/apprun-hooks/$HOOK_NAME"

  if grep -q "$HOOK_NAME" "$appdir/AppRun"; then
    echo "  AppRun already sources $HOOK_NAME — hook refreshed"
    return 0
  fi

  # The hook has to run before AppRun execs the app, so insert the source line
  # ahead of the exec line rather than appending it.
  grep -q "^exec " "$appdir/AppRun" || die "no exec line in $appdir/AppRun — unexpected AppRun layout"
  awk -v hook="$HOOK_NAME" '
    /^exec / && !inserted {
      print "source \"$this_dir\"/apprun-hooks/\"" hook "\""
      print ""
      inserted = 1
    }
    { print }
  ' "$appdir/AppRun" > "$appdir/AppRun.patched"
  chmod --reference="$appdir/AppRun" "$appdir/AppRun.patched"
  mv "$appdir/AppRun.patched" "$appdir/AppRun"
  echo "  installed $HOOK_NAME and wired it into AppRun"
}

find_appimagetool() {
  if [ -n "${APPIMAGETOOL:-}" ]; then
    echo "$APPIMAGETOOL"
    return 0
  fi
  if command -v appimagetool > /dev/null 2>&1; then
    command -v appimagetool
    return 0
  fi
  local cached="$HOME/.cache/tauri/linuxdeploy-plugin-appimage.AppImage"
  if [ -x "$cached" ]; then
    echo "$cached"
    return 0
  fi
  die "no appimagetool found — set APPIMAGETOOL=/path/to/appimagetool"
}

repack() {
  local appdir="$1"
  local output="$2"
  local tool
  tool="$(find_appimagetool)"

  # linuxdeploy-plugin-appimage takes --appdir and names its output from
  # $LDAI_OUTPUT (older builds: $OUTPUT — set both); a plain appimagetool takes
  # the AppDir and the target path as positional arguments.
  if [[ "$tool" == *linuxdeploy-plugin-appimage* ]]; then
    ARCH="$(uname -m)" LDAI_OUTPUT="$output" OUTPUT="$output" "$tool" --appdir "$appdir" > /dev/null
  else
    ARCH="$(uname -m)" "$tool" "$appdir" "$output" > /dev/null
  fi
  [ -f "$output" ] || die "repacking produced no AppImage at $output"
}

patch_appimage() {
  local appimage="$1"
  [ -f "$appimage" ] || die "no such AppImage: $appimage"
  appimage="$(readlink -f "$appimage")"
  echo "Patching $(basename "$appimage")"

  local work
  work="$(mktemp -d)"
  # shellcheck disable=SC2064 # $work is intentionally expanded now, not at exit
  trap "rm -rf '$work'" RETURN

  chmod +x "$appimage"
  ( cd "$work" && "$appimage" --appimage-extract > /dev/null )
  patch_appdir "$work/squashfs-root"
  repack "$work/squashfs-root" "$work/patched.AppImage"

  mv "$work/patched.AppImage" "$appimage"
  chmod +x "$appimage"
  echo "  repacked $(basename "$appimage")"
}

main() {
  [ -f "$HOOK_SOURCE" ] || die "missing hook source: $HOOK_SOURCE"

  if [ "${1:-}" = "--appdir" ]; then
    [ -n "${2:-}" ] || die "--appdir needs a path"
    patch_appdir "$2"
    return 0
  fi

  if [ "$#" -gt 0 ]; then
    local appimage
    for appimage in "$@"; do
      patch_appimage "$appimage"
    done
    return 0
  fi

  local found=0
  local appimage
  for appimage in "$ROOT"/target/release/bundle/appimage/*.AppImage; do
    [ -f "$appimage" ] || continue
    patch_appimage "$appimage"
    found=1
  done
  [ "$found" = 1 ] || die "no AppImage in target/release/bundle/appimage — run 'cargo tauri build' first"
}

main "$@"
