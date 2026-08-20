// Stages a PORTABLE layout of the app — the release binary with `rules/` beside
// it — outside every `target/` directory, so the portable rules-resolution path
// can be driven by WebdriverIO (`wdio.portable.conf.js`).
//
// WHY A SEPARATE LAYOUT AT ALL. `load_ruleset` (crates/arm-app/src/commands.rs)
// offers two candidate directories and takes whichever exists: the one Tauri's
// `BaseDirectory::Resource` names, and `./rules` next to the executable. Under
// `target/release` — where the ordinary e2e suite runs — Tauri treats the binary
// as a dev build and resolves Resource to the exe's own directory, so the first
// candidate already hits and the fallback is never exercised. A binary shipped
// anywhere else resolves Resource to `/usr/lib/<name>`, which does not exist, and
// the app boots only because of the fallback. That is the layout `build-linux.sh`
// and `build-win.sh` produce and the one a user on a USB stick runs, and until
// this staging existed nothing tested it.
//
// The layout below deliberately mirrors `build-linux.sh`: the binary, `rules/core`
// and `rules/i18n` beside it, and the attribution notice that has to travel with
// the rules text.
//
// Staging happens in Node rather than in a shell script so `npm run
// test:e2e:portable` is the whole command a human needs, on any platform.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dirname = path.dirname(fileURLToPath(import.meta.url));

/** Repository root, from `ui/e2e/`. */
export const repoRoot = path.resolve(dirname, '../..');

/**
 * Where the portable copy is staged. Under the gitignored `tmp/`, and — the whole
 * point — outside `target/`, so Tauri does not take it for a dev build.
 */
export const portableDir = path.resolve(repoRoot, 'tmp/portable');

/** The staged executable a wdio config points `tauri:options.application` at. */
export const portableApp = path.resolve(portableDir, 'arm-app');

/**
 * Build the production binary and stage the portable layout, replacing whatever
 * was there before.
 *
 * @param {{ build?: boolean }} [options] pass `{ build: false }` to stage an
 *   already-built `target/release/arm-app` without recompiling
 */
export function stagePortableApp({ build = true } = {}) {
  if (build) {
    const result = spawnSync('cargo', ['tauri', 'build', '--no-bundle'], {
      cwd: path.resolve(repoRoot, 'crates/arm-app'),
      stdio: 'inherit',
    });
    if (result.status !== 0) throw new Error('cargo tauri build failed');
  }

  const binary = path.resolve(repoRoot, 'target/release/arm-app');
  if (!fs.existsSync(binary)) {
    throw new Error(`no release binary at ${binary}; run \`cargo tauri build --no-bundle\` first`);
  }

  // A stale staging would hide a rules file that stopped shipping, so start clean.
  fs.rmSync(portableDir, { recursive: true, force: true });
  fs.mkdirSync(portableDir, { recursive: true });

  fs.copyFileSync(binary, portableApp);
  fs.chmodSync(portableApp, 0o755);

  for (const sub of ['core', 'i18n']) {
    fs.cpSync(path.resolve(repoRoot, 'rules', sub), path.resolve(portableDir, 'rules', sub), {
      recursive: true,
    });
  }
  // Attribution travels with the data it describes, exactly as build-linux.sh
  // arranges it for a shipped archive.
  fs.copyFileSync(
    path.resolve(repoRoot, 'rules/NOTICE.md'),
    path.resolve(portableDir, 'rules/NOTICE.md'),
  );

  return portableApp;
}
