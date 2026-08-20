import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { config as portableConfig } from './wdio.portable.conf.js';
import { config as standardConfig } from './wdio.conf.js';

// An e2e run is expensive — a release build plus a full drive of the real
// binary. Its output is therefore worth keeping rather than re-running the
// suite to recover a line that scrolled past, so both configs set wdio's
// `outputDir`. The location is the load-bearing part: it must be the
// repo-local, gitignored `tmp/`, never the system temp dir. A path under
// `/tmp` puts the artifact outside the repo, where `git status` cannot see it
// and it is shared with every other project on the machine.
//
// (The save/load fixture the specs round-trip is the one deliberate exception:
// it goes through `os.tmpdir()` because the app's own native file dialogs need
// an OS-native target. That is committed, intentional, and unrelated to logs.)
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const expectedLogDir = path.resolve(repoRoot, 'tmp/e2e-logs');

describe.each([
  ['standard', standardConfig],
  ['portable', portableConfig],
])('%s wdio config log output', (_name, config) => {
  it('writes run logs to the repo-local tmp/', () => {
    expect(config.outputDir).toBe(expectedLogDir);
  });

  it('does not write run logs to the system temp dir', () => {
    expect(config.outputDir.startsWith(os.tmpdir())).toBe(false);
  });
});
