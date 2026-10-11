import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  beginFrontendBuild,
  commitFrontendBuild,
  e2eApplication,
  e2eBuildReusable,
  e2eTargetDir,
  frontendIsFresh,
  reuseRequested,
  runCli,
  rustInputsFingerprint,
  writeE2eBuildStamp,
} from './build-freshness.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

// A miniature repository under the repo-local, gitignored `tmp/`, laid out like
// the real one, so the freshness logic can be driven without touching the real
// `ui/dist` or the real stamps.
const FIXTURE_FILES = {
  'ui/index.html': '<div id="app"></div>',
  'ui/src/main.ts': 'export const app = 1;',
  'ui/src/lib/App.svelte': '<p>app</p>',
  'ui/vite.config.ts': 'export default {};',
  'ui/svelte.config.js': 'export default {};',
  'ui/tsconfig.json': '{}',
  'ui/package.json': '{"name":"ui"}',
  'ui/package-lock.json': '{"lockfileVersion":3}',
  'ui/public/favicon.png': 'png',
  'locales/en/main.ftl': 'app-title = App',
  'ui/e2e/helpers.js': 'export {};',
  'ui/README.md': '# ui',
  'Cargo.toml': '[workspace]',
  'Cargo.lock': 'version = 4',
  'crates/arm-app/Cargo.toml': '[package]',
  'crates/arm-app/build.rs': 'fn main() {}',
  'crates/arm-app/tauri.conf.json': '{}',
  'crates/arm-app/src/main.rs': 'fn main() {}',
  'crates/arm-app/capabilities/default.json': '{}',
  'crates/arm-app/gen/schemas/desktop-schema.json': '{}',
  'crates/arm-rules/Cargo.toml': '[package]',
  'crates/arm-rules/src/lib.rs': 'pub fn rules() {}',
};

let root;

function write(relative, content) {
  const file = path.resolve(root, relative);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

function append(relative, extra) {
  fs.appendFileSync(path.resolve(root, relative), extra);
}

/** What `vite build` leaves in `ui/dist`; the content stands for the mode. */
function writeDist(content = 'production bundle') {
  fs.rmSync(path.resolve(root, 'ui/dist'), { recursive: true, force: true });
  write('ui/dist/index.html', '<script src="assets/index.js"></script>');
  write('ui/dist/assets/index.js', content);
}

/** The `npm run build` sequence: begin, (svelte-check +) vite build, commit. */
function frontendBuild(env = { NODE_ENV: 'production' }, content) {
  beginFrontendBuild(root, env);
  writeDist(content);
  commitFrontendBuild(root);
}

/** Pushes a file's mtime into the past without touching its content. */
function touchOnly(relative) {
  const file = path.resolve(root, relative);
  const past = new Date(Date.now() - 3600_000);
  fs.utimesSync(file, past, past);
}

beforeEach(() => {
  fs.mkdirSync(path.resolve(repoRoot, 'tmp'), { recursive: true });
  root = fs.mkdtempSync(path.resolve(repoRoot, 'tmp', 'build-freshness-test-'));
  for (const [relative, content] of Object.entries(FIXTURE_FILES)) write(relative, content);
});

afterEach(() => {
  fs.rmSync(root, { recursive: true, force: true });
});

// The e2e and portable builds skip `beforeBuildCommand` (svelte-check + vite
// build) when `ui/dist` is already the production build of the current
// sources. "Current" has to be safe: a wrong "fresh" would drive the suite
// against a stale or development frontend while reporting green.
describe('frontendIsFresh', () => {
  it('is not fresh before any frontend build has been recorded', () => {
    writeDist();
    expect(frontendIsFresh(root)).toBe(false);
  });

  it('is fresh right after a production build', () => {
    frontendBuild();
    expect(frontendIsFresh(root)).toBe(true);
  });

  // The gate's `cargo tauri build` runs from Norbert's shell, where NODE_ENV is
  // unset; Vite then builds in production mode. An empty value counts as unset
  // (`process.env.NODE_ENV || config.mode` in Vite).
  it.each([[{}], [{ NODE_ENV: '' }]])('counts an unset NODE_ENV as production (%j)', (env) => {
    frontendBuild(env);
    expect(frontendIsFresh(root)).toBe(true);
  });

  // @wdio/cli and vitest set NODE_ENV=test, and Vite keeps an inherited one:
  // such a build is a development frontend and must never be reused.
  it.each(['test', 'development'])('is never fresh after a NODE_ENV=%s build', (mode) => {
    frontendBuild({ NODE_ENV: mode }, `${mode} bundle`);
    expect(frontendIsFresh(root)).toBe(false);
  });

  it('is not fresh once a non-production build replaces a production one', () => {
    frontendBuild();
    frontendBuild({ NODE_ENV: 'test' }, 'test bundle');
    expect(frontendIsFresh(root)).toBe(false);
  });

  // A production build that never committed (svelte-check or vite failed)
  // leaves no stamp from before it either.
  it('forgets an earlier build as soon as a new one begins', () => {
    frontendBuild();
    beginFrontendBuild(root, { NODE_ENV: 'production' });
    expect(frontendIsFresh(root)).toBe(false);
  });

  it('records nothing when commit runs without a begin', () => {
    writeDist();
    commitFrontendBuild(root);
    expect(frontendIsFresh(root)).toBe(false);
  });

  // The sources are fingerprinted when the build begins, so an edit made
  // while Vite was running cannot be stamped as already built.
  it('records nothing when a source changes while the build runs', () => {
    beginFrontendBuild(root, { NODE_ENV: 'production' });
    append('ui/src/main.ts', '\n// edited mid-build');
    writeDist();
    commitFrontendBuild(root);
    expect(frontendIsFresh(root)).toBe(false);
  });

  // Every input Vite reads: the entry HTML, the sources, the `.ftl` locales
  // pulled in from the repo root (`import.meta.glob`), `public/`, the build
  // configs, and the dependency lockfiles.
  it.each([
    'ui/index.html',
    'ui/src/main.ts',
    'ui/src/lib/App.svelte',
    'ui/vite.config.ts',
    'ui/svelte.config.js',
    'ui/tsconfig.json',
    'ui/package.json',
    'ui/package-lock.json',
    'ui/public/favicon.png',
    'locales/en/main.ftl',
  ])('is not fresh after %s changes', (input) => {
    frontendBuild();
    append(input, '\n ');
    expect(frontendIsFresh(root)).toBe(false);
  });

  it('is not fresh after a source file is added', () => {
    frontendBuild();
    write('ui/src/lib/New.svelte', '<p>new</p>');
    expect(frontendIsFresh(root)).toBe(false);
  });

  it('is not fresh after a source file is renamed', () => {
    frontendBuild();
    fs.renameSync(path.resolve(root, 'ui/src/main.ts'), path.resolve(root, 'ui/src/entry.ts'));
    expect(frontendIsFresh(root)).toBe(false);
  });

  it('is not fresh once ui/dist itself no longer holds what was built', () => {
    frontendBuild();
    write('ui/dist/assets/index.js', 'something else');
    expect(frontendIsFresh(root)).toBe(false);
  });

  it('is not fresh when ui/dist is gone', () => {
    frontendBuild();
    fs.rmSync(path.resolve(root, 'ui/dist'), { recursive: true });
    expect(frontendIsFresh(root)).toBe(false);
  });

  // Content, not timestamps: a checkout or a `vite build` that rewrites the
  // same bytes must not force a rebuild.
  it('stays fresh when only mtimes change', () => {
    frontendBuild();
    touchOnly('ui/src/main.ts');
    touchOnly('ui/dist/assets/index.js');
    expect(frontendIsFresh(root)).toBe(true);
  });

  it('stays fresh when a file Vite never reads changes', () => {
    frontendBuild();
    append('ui/e2e/helpers.js', '\n// harness edit');
    append('ui/README.md', '\nmore');
    append('crates/arm-app/src/main.rs', '\n// rust edit');
    expect(frontendIsFresh(root)).toBe(true);
  });
});

// `npm run build` calls this file as `node e2e/build-freshness.js begin|commit`.
describe('runCli', () => {
  it('records a production build through begin and commit', () => {
    runCli(['begin'], root, { NODE_ENV: 'production' });
    writeDist();
    runCli(['commit'], root, { NODE_ENV: 'production' });
    expect(frontendIsFresh(root)).toBe(true);
  });

  it('refuses an unknown command, naming it', () => {
    expect(() => runCli(['stamp'], root, {})).toThrow(/stamp/);
  });

  // `npm run build` is also the release pipeline's `beforeBuildCommand`. The
  // stamp is a cache, not a product step: failing to write it must warn and
  // leave the build alone. A file where the stamp directory belongs makes
  // every write there fail.
  it('only warns when the stamp cannot be written', () => {
    write('tmp/build-stamps', 'not a directory');
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    try {
      expect(() => runCli(['begin'], root, { NODE_ENV: 'production' })).not.toThrow();
      writeDist();
      expect(() => runCli(['commit'], root, { NODE_ENV: 'production' })).not.toThrow();
      expect(warn).toHaveBeenCalledTimes(2);
      expect(frontendIsFresh(root)).toBe(false);
    } finally {
      warn.mockRestore();
    }
  });

  // The release runners start from a fresh checkout, with no `tmp/` at all.
  it('creates the stamp directory when the repository has no tmp/', () => {
    expect(fs.existsSync(path.resolve(root, 'tmp'))).toBe(false);
    frontendBuild();
    expect(frontendIsFresh(root)).toBe(true);
  });
});

describe('the e2e build location', () => {
  // Its own cargo target dir, so the `e2e-testing` binary never overwrites the
  // plain `target/release/arm-app` the gate built (and the other way round).
  it('builds into target/e2e', () => {
    expect(e2eTargetDir(root)).toBe(path.resolve(root, 'target/e2e'));
  });

  it('runs the binary cargo leaves under target/e2e/release', () => {
    expect(e2eApplication(root)).toBe(path.resolve(root, 'target/e2e/release/arm-app'));
  });
});

describe('reuseRequested', () => {
  it.each([[{}], [{ ARM_E2E_REUSE_BUILD: '' }]])('is off by default (%j)', (env) => {
    expect(reuseRequested(env)).toBe(false);
  });

  it('is on for ARM_E2E_REUSE_BUILD=1', () => {
    expect(reuseRequested({ ARM_E2E_REUSE_BUILD: '1' })).toBe(true);
  });

  // A typo must not silently mean "rebuild" or "reuse".
  it.each(['0', 'true', 'yes', '2'])('refuses %j with a message naming the variable', (value) => {
    expect(() => reuseRequested({ ARM_E2E_REUSE_BUILD: value })).toThrow(/ARM_E2E_REUSE_BUILD/);
  });
});

// `ARM_E2E_REUSE_BUILD=1` skips the e2e cargo build entirely. That is only
// sound while the binary is exactly the one the last successful e2e build left,
// built from the Rust sources and the frontend that are there now.
describe('e2eBuildReusable', () => {
  const binary = 'target/e2e/release/arm-app';

  /** A successful e2e build: the frontend is current and cargo linked a binary. */
  function e2eBuild() {
    frontendBuild();
    const before = rustInputsFingerprint(root);
    write(binary, 'e2e binary');
    writeE2eBuildStamp(root, before);
  }

  it('is not reusable before any e2e build has been recorded', () => {
    frontendBuild();
    write(binary, 'e2e binary');
    expect(e2eBuildReusable(root)).toBe(false);
  });

  it('is reusable right after a successful e2e build', () => {
    e2eBuild();
    expect(e2eBuildReusable(root)).toBe(true);
  });

  it('is not reusable when the binary is gone', () => {
    e2eBuild();
    fs.rmSync(path.resolve(root, binary));
    expect(e2eBuildReusable(root)).toBe(false);
  });

  it('is not reusable when the binary was rebuilt since', () => {
    e2eBuild();
    write(binary, 'another e2e binary');
    expect(e2eBuildReusable(root)).toBe(false);
  });

  it('is not reusable when only the binary mtime moved', () => {
    e2eBuild();
    touchOnly(binary);
    expect(e2eBuildReusable(root)).toBe(false);
  });

  it.each([
    'crates/arm-app/src/main.rs',
    'crates/arm-app/Cargo.toml',
    'crates/arm-app/build.rs',
    'crates/arm-app/tauri.conf.json',
    'crates/arm-app/capabilities/default.json',
    'crates/arm-rules/src/lib.rs',
    'crates/arm-rules/Cargo.toml',
    'Cargo.toml',
    'Cargo.lock',
  ])('is not reusable after %s changes', (input) => {
    e2eBuild();
    append(input, '\n ');
    expect(e2eBuildReusable(root)).toBe(false);
  });

  it('is not reusable when a Rust source was edited while cargo ran', () => {
    frontendBuild();
    const before = rustInputsFingerprint(root);
    append('crates/arm-rules/src/lib.rs', '\n// edited mid-build');
    write(binary, 'e2e binary');
    writeE2eBuildStamp(root, before);
    expect(e2eBuildReusable(root)).toBe(false);
  });

  it('is not reusable once the frontend sources changed', () => {
    e2eBuild();
    append('ui/src/main.ts', '\n// edited');
    expect(e2eBuildReusable(root)).toBe(false);
  });

  // The frontend can be current again (rebuilt) and still not be the one this
  // binary embeds.
  it('is not reusable when ui/dist was rebuilt with different content', () => {
    e2eBuild();
    append('ui/src/main.ts', '\n// edited');
    frontendBuild({ NODE_ENV: 'production' }, 'a newer production bundle');
    expect(frontendIsFresh(root)).toBe(true);
    expect(e2eBuildReusable(root)).toBe(false);
  });

  // tauri-build regenerates these on every build; they are output, not input.
  it('stays reusable when only the generated schemas change', () => {
    e2eBuild();
    append('crates/arm-app/gen/schemas/desktop-schema.json', '\n ');
    expect(e2eBuildReusable(root)).toBe(true);
  });
});

// The npm scripts wire the stamp into every frontend build and cache lint.
describe('ui/package.json scripts', () => {
  const { scripts } = JSON.parse(
    fs.readFileSync(path.resolve(repoRoot, 'ui/package.json'), 'utf8'),
  );

  // Vite writes into tmp/vite-dist; `sync-dist.js` mirrors that into ui/dist,
  // rewriting only what changed, before the stamp records it.
  it('brackets the production frontend build with begin and commit', () => {
    expect(scripts.build).toBe(
      'node e2e/build-freshness.js begin && svelte-check --tsconfig ./tsconfig.json && vite build --outDir ../tmp/vite-dist --emptyOutDir && node e2e/sync-dist.js && node e2e/build-freshness.js commit',
    );
  });

  it('offers a reuse variant of the e2e run', () => {
    expect(scripts['test:e2e:reuse']).toBe('wdio run ./e2e/wdio.reuse.conf.js');
  });

  // Caches under the repo-local, gitignored tmp/, never the system temp dir.
  it('caches eslint and prettier under the repo-local tmp/', () => {
    expect(scripts.lint).toBe(
      'eslint . --cache --cache-location ../tmp/lint-cache/eslint.json && prettier --check . --cache --cache-location ../tmp/lint-cache/prettier.json',
    );
  });
});
