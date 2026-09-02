# scripts/stage-rules.sh — copy rules/core and rules/i18n from the repo root into
# a destination "rules" directory, replacing anything already there.
#
# Shared by arm-char-gen.sh, build-linux.sh, and build-win.sh: all three build a
# --no-bundle binary, and a non-bundled binary needs the rules data staged next
# to it by hand (the bundle/installer would otherwise place it for us — see the
# callers for the per-platform reasoning about BaseDirectory::Resource).
#
# Sourced, not executed: it runs under the caller's `set -euo pipefail` and from
# the caller's working directory (the repo root, after each caller's own `cd
# "$ROOT"`), so paths here are relative to the repo root exactly like the rest
# of each caller's script.
#
# Usage:
#   . "$ROOT/scripts/stage-rules.sh"
#   stage_rules "target/release/rules"
stage_rules() {
  local dest="$1"
  rm -rf "$dest"
  mkdir -p "$dest"
  cp -R "rules/core" "$dest/core"
  cp -R "rules/i18n" "$dest/i18n"
}
