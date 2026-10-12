import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { stagePortableApp } from './stage-portable.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

// `stagePortableApp` stages the release binary plus `rules/` and
// `examples/rulebook` into a layout outside `target/`, for `npm run
// test:e2e:portable` (see stage-portable.js's own header). It has real,
// silently-breakable file-staging logic — a binary-missing guard, which
// rules/ subdirectories get copied, a stale-directory wipe — with no test
// coverage short of a full e2e:portable run (E9, full-audit round 2).
//
// These tests drive it against an isolated fixture tree, mirroring
// sync-dist.test.js's shape, rather than the real repo paths.
let fixtureRoot;
let portableOut;

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

/** A fixture repo root with a release binary, rules/, and examples/rulebook. */
function buildFixtureRepo({ withBinary = true } = {}) {
  if (withBinary) write(path.resolve(fixtureRoot, 'target/release/arm-app'), 'fake binary');
  write(path.resolve(fixtureRoot, 'rules/core/abilities.json'), '{}');
  write(path.resolve(fixtureRoot, 'rules/i18n/en/abilities.json'), '{}');
  write(path.resolve(fixtureRoot, 'rules/NOTICE.md'), 'attribution');
  // Not shipped in a portable layout — build-linux.sh never stages rulebook
  // source text beside the binary.
  write(path.resolve(fixtureRoot, 'rules/source/en/core.md'), '# not shipped');
  write(path.resolve(fixtureRoot, 'examples/rulebook/grog.json'), '{}');
}

beforeEach(() => {
  fs.mkdirSync(path.resolve(repoRoot, 'tmp'), { recursive: true });
  const root = fs.mkdtempSync(path.resolve(repoRoot, 'tmp', 'stage-portable-test-'));
  fixtureRoot = path.resolve(root, 'repo');
  portableOut = path.resolve(root, 'portable');
});

afterEach(() => {
  fs.rmSync(path.dirname(fixtureRoot), { recursive: true, force: true });
});

describe('stagePortableApp', () => {
  it('throws a clear error when the release binary is missing', () => {
    buildFixtureRepo({ withBinary: false });

    expect(() =>
      stagePortableApp({ build: false, repoRoot: fixtureRoot, portableDir: portableOut }),
    ).toThrow(/no release binary/);
  });

  it('stages the binary executably and copies rules/core, rules/i18n and rules/NOTICE.md, but not rules/source', () => {
    buildFixtureRepo();

    stagePortableApp({ build: false, repoRoot: fixtureRoot, portableDir: portableOut });

    const staged = path.resolve(portableOut, 'arm-app');
    expect(fs.existsSync(staged)).toBe(true);
    expect(fs.statSync(staged).mode & 0o111).toBeTruthy();
    expect(fs.existsSync(path.resolve(portableOut, 'rules/core/abilities.json'))).toBe(true);
    expect(fs.existsSync(path.resolve(portableOut, 'rules/i18n/en/abilities.json'))).toBe(true);
    expect(fs.existsSync(path.resolve(portableOut, 'rules/NOTICE.md'))).toBe(true);
    expect(fs.existsSync(path.resolve(portableOut, 'rules/source'))).toBe(false);
  });

  it('copies examples/rulebook to <portableDir>/examples', () => {
    buildFixtureRepo();

    stagePortableApp({ build: false, repoRoot: fixtureRoot, portableDir: portableOut });

    // `examples/rulebook`'s own CONTENTS land directly under `<portableDir>/examples`
    // — mirroring build-linux.sh's `stage_examples`, not nested under `rulebook/`.
    expect(fs.existsSync(path.resolve(portableOut, 'examples/grog.json'))).toBe(true);
  });

  it('removes stale leftover content on a second call rather than merging', () => {
    buildFixtureRepo();
    stagePortableApp({ build: false, repoRoot: fixtureRoot, portableDir: portableOut });
    write(path.resolve(portableOut, 'stale-leftover.txt'), 'old');

    stagePortableApp({ build: false, repoRoot: fixtureRoot, portableDir: portableOut });

    expect(fs.existsSync(path.resolve(portableOut, 'stale-leftover.txt'))).toBe(false);
  });
});
