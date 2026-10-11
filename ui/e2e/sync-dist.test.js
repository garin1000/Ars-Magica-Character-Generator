import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { syncDist } from './sync-dist.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

// `npm run build` has Vite write into `tmp/vite-dist`, then mirrors that into
// `ui/dist`, rewriting only what changed. arm-app embeds `ui/dist`, and cargo
// recompiles it whenever a file there gets a new mtime, so an unchanged
// frontend must leave `ui/dist` untouched. The result must still be exactly
// what Vite built, with its `emptyOutDir` semantics: nothing stale survives.
let root;
let source;
let dest;

function write(dir, relative, content) {
  const file = path.resolve(dir, relative);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

/** Every file under `dir` as `relative path -> content`; `{}` for no directory. */
function tree(dir) {
  const files = {};
  if (!fs.existsSync(dir)) return files;
  for (const relative of fs.readdirSync(dir, { recursive: true })) {
    const file = path.resolve(dir, relative);
    if (fs.statSync(file).isFile()) {
      files[relative.split(path.sep).join('/')] = fs.readFileSync(file, 'utf8');
    }
  }
  return files;
}

function mtimeOf(dir, relative) {
  return fs.statSync(path.resolve(dir, relative)).mtimeMs;
}

function backdate(dir, relative) {
  const past = new Date(Date.now() - 3600_000);
  fs.utimesSync(path.resolve(dir, relative), past, past);
}

/** What one `vite build` produced. */
function viteOutput(files) {
  fs.rmSync(source, { recursive: true, force: true });
  for (const [relative, content] of Object.entries(files)) write(source, relative, content);
}

beforeEach(() => {
  fs.mkdirSync(path.resolve(repoRoot, 'tmp'), { recursive: true });
  root = fs.mkdtempSync(path.resolve(repoRoot, 'tmp', 'sync-dist-test-'));
  source = path.resolve(root, 'vite-dist');
  dest = path.resolve(root, 'dist');
});

afterEach(() => {
  fs.rmSync(root, { recursive: true, force: true });
});

describe('syncDist', () => {
  it('creates ui/dist as an exact copy of the Vite output', () => {
    viteOutput({ 'index.html': 'html', 'assets/index-a1.js': 'js', 'favicon.png': 'png' });

    syncDist(source, dest);

    expect(tree(dest)).toEqual(tree(source));
  });

  it('leaves an unchanged file alone, mtime included', () => {
    viteOutput({ 'index.html': 'html', 'assets/index-a1.js': 'js' });
    syncDist(source, dest);
    expect(tree(dest)).toEqual(tree(source));
    backdate(dest, 'assets/index-a1.js');
    const before = mtimeOf(dest, 'assets/index-a1.js');

    viteOutput({ 'index.html': 'html', 'assets/index-a1.js': 'js' });
    syncDist(source, dest);

    expect(mtimeOf(dest, 'assets/index-a1.js')).toBe(before);
  });

  it('rewrites a file whose content changed', () => {
    viteOutput({ 'index.html': 'old html' });
    syncDist(source, dest);

    viteOutput({ 'index.html': 'new html' });
    syncDist(source, dest);

    expect(tree(dest)).toEqual({ 'index.html': 'new html' });
  });

  // Vite's `emptyOutDir`: an asset the new build no longer produces (an old
  // content hash) must not linger, nor must a directory left empty by it.
  it('removes files and directories the new build no longer produces', () => {
    viteOutput({ 'index.html': 'html', 'assets/index-a1.js': 'old', 'old/x.css': 'css' });
    syncDist(source, dest);

    viteOutput({ 'index.html': 'html', 'assets/index-b2.js': 'new' });
    syncDist(source, dest);

    expect(tree(dest)).toEqual({ 'index.html': 'html', 'assets/index-b2.js': 'new' });
    expect(fs.existsSync(path.resolve(dest, 'old'))).toBe(false);
  });

  it('replaces a directory with a file of the same name, and back', () => {
    viteOutput({ 'assets/a.js': 'js' });
    syncDist(source, dest);
    viteOutput({ assets: 'now a file' });
    syncDist(source, dest);
    expect(tree(dest)).toEqual({ assets: 'now a file' });

    viteOutput({ 'assets/a.js': 'js again' });
    syncDist(source, dest);
    expect(tree(dest)).toEqual({ 'assets/a.js': 'js again' });
  });

  // A failed Vite run leaves no output; that must fail the build, not empty ui/dist.
  it('refuses a missing Vite output and leaves ui/dist alone', () => {
    write(dest, 'index.html', 'html');

    expect(() => syncDist(source, dest)).toThrow(/vite-dist/);
    expect(tree(dest)).toEqual({ 'index.html': 'html' });
  });
});
