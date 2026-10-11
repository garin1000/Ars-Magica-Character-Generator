// Mirrors Vite's output (`tmp/vite-dist`) into `ui/dist`, rewriting only the
// files whose bytes changed and deleting whatever the build no longer produces.
//
// arm-app embeds `ui/dist` (`generate_context!`), and cargo recompiles it
// (~40 s) whenever a file there carries a new mtime. `vite build` straight into
// `ui/dist` rewrites every file with identical bytes, so every release build
// recompiled arm-app even with nothing changed. Mirroring keeps `ui/dist`
// byte-identical to what Vite built, with `emptyOutDir`'s guarantee that
// nothing stale survives, while an unchanged file keeps its mtime.
//
// Called by `npm run build` as `node e2e/sync-dist.js`. A failure here fails
// the build: unlike the freshness stamp, `ui/dist` is the product.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function sameBytes(a, b) {
  return fs.readFileSync(a).equals(fs.readFileSync(b));
}

function isDirectory(file) {
  return fs.existsSync(file) && fs.statSync(file).isDirectory();
}

/** Makes `dest` hold exactly the files of `source`, touching only what differs. */
function mirror(source, dest) {
  if (fs.existsSync(dest) && !isDirectory(dest)) fs.rmSync(dest);
  fs.mkdirSync(dest, { recursive: true });

  const wanted = new Set(fs.readdirSync(source));
  for (const name of fs.readdirSync(dest)) {
    if (!wanted.has(name)) fs.rmSync(path.join(dest, name), { recursive: true });
  }

  for (const name of wanted) {
    const from = path.join(source, name);
    const to = path.join(dest, name);
    if (isDirectory(from)) {
      mirror(from, to);
      continue;
    }
    if (isDirectory(to)) fs.rmSync(to, { recursive: true });
    if (!fs.existsSync(to) || !sameBytes(from, to)) fs.copyFileSync(from, to);
  }
}

/**
 * @param {string} source Vite's output directory
 * @param {string} dest `ui/dist`
 */
export function syncDist(source, dest) {
  if (!isDirectory(source)) {
    throw new Error(`sync-dist: no Vite output at ${source}; ui/dist left as it was`);
  }
  mirror(source, dest);
}

const thisFile = fileURLToPath(import.meta.url);
if (process.argv[1] && path.resolve(process.argv[1]) === thisFile) {
  const repoRoot = path.resolve(path.dirname(thisFile), '../..');
  syncDist(path.resolve(repoRoot, 'tmp', 'vite-dist'), path.resolve(repoRoot, 'ui', 'dist'));
}
