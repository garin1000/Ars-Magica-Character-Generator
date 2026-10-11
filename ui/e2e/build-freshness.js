// Decides when the e2e and portable builds may skip work that is already done.
//
// Two questions, both answered by CONTENT hashes recorded in stamps under the
// repo-local, gitignored `tmp/build-stamps/` — never by timestamps alone,
// because `vite build` rewrites `ui/dist` with identical bytes and new mtimes:
//
// 1. Is `ui/dist` the production build of the frontend sources as they are now?
//    `npm run build` brackets svelte-check + `vite build` with `begin` and
//    `commit` (the CLI at the bottom). `begin` forgets any earlier stamp and,
//    only for a production build, fingerprints the inputs; `commit` stamps the
//    built `ui/dist` only if the inputs did not change while it ran. When the
//    stamp still matches, the e2e build skips its own `npm run build`.
//
// 2. Is the e2e binary exactly what an e2e build of the current sources would
//    produce? Only asked when ARM_E2E_REUSE_BUILD=1 (opt-in, never the gate).
//
// The stamp is a cache, not a product step: `npm run build` is also the release
// pipeline's `beforeBuildCommand`, so a stamp that cannot be written only warns.

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Everything `vite build` reads, relative to the repository root: the entry
 * HTML, the sources, `public/` (copied verbatim), the `.ftl` locales pulled in
 * from the repo root by `import.meta.glob` (`src/lib/i18n.ts`), the build
 * configs, and the dependency lockfiles. `ui/.env*` files are added when present.
 */
const FRONTEND_INPUTS = [
  'ui/index.html',
  'ui/src',
  'ui/public',
  'ui/vite.config.ts',
  'ui/svelte.config.js',
  'ui/tsconfig.json',
  'ui/package.json',
  'ui/package-lock.json',
  'ui/node_modules/.package-lock.json',
  'locales',
];

/** Everything cargo compiles into arm-app, besides the embedded `ui/dist`. */
const RUST_INPUTS = ['Cargo.toml', 'Cargo.lock', 'crates'];

/** tauri-build regenerates these on every build: output, not input. */
const RUST_GENERATED = ['crates/arm-app/gen'];

const DIST = 'ui/dist';

function stampDir(repoRoot) {
  return path.resolve(repoRoot, 'tmp', 'build-stamps');
}

function stampFile(repoRoot, name) {
  return path.resolve(stampDir(repoRoot), `${name}.json`);
}

/** The stamp's content, or `undefined` when there is none. Other IO errors throw. */
function readStamp(repoRoot, name) {
  try {
    return JSON.parse(fs.readFileSync(stampFile(repoRoot, name), 'utf8'));
  } catch (error) {
    if (error.code === 'ENOENT') return undefined;
    throw error;
  }
}

function writeStamp(repoRoot, name, content) {
  fs.mkdirSync(stampDir(repoRoot), { recursive: true });
  fs.writeFileSync(stampFile(repoRoot, name), JSON.stringify(content, null, 2));
}

function removeStamp(repoRoot, name) {
  fs.rmSync(stampFile(repoRoot, name), { force: true });
}

/** Every file under the given repo-relative entries, as sorted `/`-separated paths. */
function filesUnder(repoRoot, entries, excluded = []) {
  const skip = new Set(excluded.map((entry) => path.resolve(repoRoot, entry)));
  const files = [];
  const visit = (absolute) => {
    if (skip.has(absolute) || !fs.existsSync(absolute)) return;
    if (fs.statSync(absolute).isDirectory()) {
      for (const name of fs.readdirSync(absolute)) visit(path.join(absolute, name));
    } else {
      files.push(path.relative(repoRoot, absolute).split(path.sep).join('/'));
    }
  };
  for (const entry of entries) visit(path.resolve(repoRoot, entry));
  return files.sort();
}

/** A content hash over the files' paths and bytes; mtimes play no part. */
function fingerprint(repoRoot, entries, excluded) {
  const hash = crypto.createHash('sha256');
  for (const file of filesUnder(repoRoot, entries, excluded)) {
    const content = fs.readFileSync(path.resolve(repoRoot, file));
    hash.update(`${file}\0${content.length}\0`);
    hash.update(content);
  }
  return hash.digest('hex');
}

function frontendInputsFingerprint(repoRoot) {
  const uiDir = path.resolve(repoRoot, 'ui');
  const envFiles = fs.existsSync(uiDir)
    ? fs.readdirSync(uiDir).filter((name) => name.startsWith('.env'))
    : [];
  return fingerprint(repoRoot, [...FRONTEND_INPUTS, ...envFiles.map((name) => `ui/${name}`)]);
}

function distFingerprint(repoRoot) {
  return fingerprint(repoRoot, [DIST]);
}

/** Vite builds in production mode unless an inherited NODE_ENV says otherwise. */
function isProductionBuild(env) {
  return !env.NODE_ENV || env.NODE_ENV === 'production';
}

/**
 * Called before svelte-check + `vite build`: forgets the previous stamp and,
 * for a production build, records the inputs the build is about to read.
 */
export function beginFrontendBuild(repoRoot, env) {
  removeStamp(repoRoot, 'frontend');
  removeStamp(repoRoot, 'frontend.pending');
  if (!isProductionBuild(env)) return;
  writeStamp(repoRoot, 'frontend.pending', { inputs: frontendInputsFingerprint(repoRoot) });
}

/**
 * Called after a successful `vite build`: stamps `ui/dist`, unless no
 * production build was begun or its inputs changed while it ran.
 */
export function commitFrontendBuild(repoRoot) {
  const pending = readStamp(repoRoot, 'frontend.pending');
  if (pending === undefined) return;
  removeStamp(repoRoot, 'frontend.pending');
  if (pending.inputs !== frontendInputsFingerprint(repoRoot)) return;
  writeStamp(repoRoot, 'frontend', { inputs: pending.inputs, dist: distFingerprint(repoRoot) });
}

/** Whether `ui/dist` is the production build of the current frontend sources. */
export function frontendIsFresh(repoRoot) {
  try {
    const stamp = readStamp(repoRoot, 'frontend');
    return (
      stamp !== undefined &&
      fs.existsSync(path.resolve(repoRoot, DIST)) &&
      stamp.inputs === frontendInputsFingerprint(repoRoot) &&
      stamp.dist === distFingerprint(repoRoot)
    );
  } catch {
    return false;
  }
}

/**
 * `cargo tauri build` arguments removing `build.beforeBuildCommand` (a JSON
 * merge-patch `null`). tauri-cli passes `--config` to cargo as TAURI_CONFIG,
 * and tauri-build reruns whenever that changes, so a target dir must get the
 * same `--config` on every build or arm-app recompiles: only the e2e build,
 * alone in `target/e2e`, uses this, and always.
 */
export const NO_BEFORE_BUILD_COMMAND = [
  '--config',
  JSON.stringify({ build: { beforeBuildCommand: null } }),
];

/** `npm run build`'s entry: `begin` or `commit`. A stamp failure only warns. */
export function runCli(argv, repoRoot, env) {
  const [command] = argv;
  const actions = {
    begin: () => beginFrontendBuild(repoRoot, env),
    commit: () => commitFrontendBuild(repoRoot),
  };
  if (!Object.hasOwn(actions, command)) {
    throw new Error(`build-freshness: unknown command ${JSON.stringify(command)}`);
  }
  try {
    actions[command]();
  } catch (error) {
    console.warn(
      `build-freshness: ${command} could not record the frontend stamp (${error.message}); ` +
        'the next e2e build simply rebuilds the frontend',
    );
  }
}

/** The e2e build's own cargo target dir, apart from the gate's `target/release`. */
export function e2eTargetDir(repoRoot) {
  return path.resolve(repoRoot, 'target', 'e2e');
}

/** The binary the e2e build leaves, and the suite drives. */
export function e2eApplication(repoRoot) {
  return path.resolve(e2eTargetDir(repoRoot), 'release', 'arm-app');
}

/** Taken before cargo starts, so an edit made while it compiles is never stamped. */
export function rustInputsFingerprint(repoRoot) {
  return fingerprint(repoRoot, RUST_INPUTS, RUST_GENERATED);
}

/** Records a successful e2e build: the binary, its Rust inputs, and the embedded dist. */
export function writeE2eBuildStamp(repoRoot, rustFingerprint) {
  const { size, mtimeMs } = fs.statSync(e2eApplication(repoRoot));
  writeStamp(repoRoot, 'e2e-build', {
    binary: { size, mtimeMs },
    rust: rustFingerprint,
    dist: distFingerprint(repoRoot),
  });
}

/** Whether the e2e binary is still exactly what the last e2e build of these sources left. */
export function e2eBuildReusable(repoRoot) {
  try {
    const stamp = readStamp(repoRoot, 'e2e-build');
    if (stamp === undefined) return false;
    const { size, mtimeMs } = fs.statSync(e2eApplication(repoRoot));
    return (
      stamp.binary.size === size &&
      stamp.binary.mtimeMs === mtimeMs &&
      stamp.rust === rustInputsFingerprint(repoRoot) &&
      stamp.dist === distFingerprint(repoRoot) &&
      frontendIsFresh(repoRoot)
    );
  } catch {
    return false;
  }
}

/** `ARM_E2E_REUSE_BUILD`: `1` is on, unset or empty is off, anything else is refused. */
export function reuseRequested(env) {
  const value = env.ARM_E2E_REUSE_BUILD;
  if (value === undefined || value === '') return false;
  if (value === '1') return true;
  throw new Error(`ARM_E2E_REUSE_BUILD must be 1 or unset, got ${JSON.stringify(value)}`);
}

const thisFile = fileURLToPath(import.meta.url);
if (process.argv[1] && path.resolve(process.argv[1]) === thisFile) {
  runCli(process.argv.slice(2), path.resolve(path.dirname(thisFile), '../..'), process.env);
}
