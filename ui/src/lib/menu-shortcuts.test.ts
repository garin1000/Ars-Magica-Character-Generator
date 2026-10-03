import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import { chordMatches, menuActionEnabled, parseAccelerator } from './menu-shortcuts';
import type { MenuFlags } from './menu';

// U4 (try-out finding 16): on Windows the menu's accelerators are drawn but
// never fire, because WebView2's child window keeps the keys. The fix is a
// Windows-only webview mirror built from the list Rust's `menu_shortcuts`
// command returns — the very strings `accelerator_for` hands muda. These tests
// pin the pure half: reading muda's notation and matching it against a key
// event. The listener itself is `menu-shortcuts.client.test.ts`.

function source(relative: string): string {
  return readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf-8');
}

describe('parseAccelerator reads muda notation', () => {
  it('reads CmdOrCtrl as Ctrl, since the mirror only ever runs on Windows', () => {
    expect(parseAccelerator('CmdOrCtrl+O')).toEqual({
      key: 'o',
      ctrl: true,
      shift: false,
      alt: false,
    });
  });

  it('reads Shift as a modifier of its own', () => {
    expect(parseAccelerator('CmdOrCtrl+Shift+S')).toEqual({
      key: 's',
      ctrl: true,
      shift: true,
      alt: false,
    });
    expect(parseAccelerator('CmdOrCtrl+Shift+E')).toEqual({
      key: 'e',
      ctrl: true,
      shift: true,
      alt: false,
    });
  });

  it('reads Alt and a plain Ctrl', () => {
    expect(parseAccelerator('Ctrl+Alt+P')).toEqual({
      key: 'p',
      ctrl: true,
      shift: false,
      alt: true,
    });
  });

  it('reads a punctuation key', () => {
    expect(parseAccelerator('CmdOrCtrl+,')).toEqual({
      key: ',',
      ctrl: true,
      shift: false,
      alt: false,
    });
  });

  // A chord the mirror cannot read is skipped, never guessed at: guessing would
  // put a shortcut on the wrong key, which is worse than the missing one.
  it.each(['', 'CmdOrCtrl+', 'CmdOrCtrl+Shift', 'Hyper+O'])('rejects %j', (accelerator) => {
    expect(parseAccelerator(accelerator)).toBeNull();
  });

  // Ties the parser to the list it will really be fed. The strings are read out
  // of `accelerator_for` itself, so a chord added there that this parser cannot
  // read fails here instead of silently being skipped on Windows.
  it('reads every chord menu.rs declares', () => {
    const menu = source('../../../crates/arm-app/src/menu.rs');
    const start = menu.indexOf('fn accelerator_for');
    expect(start, 'menu.rs no longer defines accelerator_for').toBeGreaterThan(-1);
    const body = menu.slice(start, menu.indexOf('\n}\n', start));
    const chords = [...body.matchAll(/Some\("([^"]+)"\)/g)].map((match) => match[1]);
    expect(chords.length).toBeGreaterThan(0);

    for (const chord of chords) {
      expect(parseAccelerator(chord), chord).not.toBeNull();
    }
  });
});

describe('chordMatches compares a chord with a key event', () => {
  const press = (key: string, modifiers: Partial<KeyboardEventInit> = {}) => ({
    key,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    metaKey: false,
    ...modifiers,
  });

  it('matches the chord it names', () => {
    expect(chordMatches(parseAccelerator('CmdOrCtrl+O')!, press('o', { ctrlKey: true }))).toBe(
      true,
    );
  });

  // Shift makes the browser report the capital letter.
  it('matches a shifted letter whatever its case', () => {
    expect(
      chordMatches(
        parseAccelerator('CmdOrCtrl+Shift+S')!,
        press('S', { ctrlKey: true, shiftKey: true }),
      ),
    ).toBe(true);
  });

  it('keeps Ctrl+S and Ctrl+Shift+S apart', () => {
    const save = parseAccelerator('CmdOrCtrl+S')!;
    const saveAs = parseAccelerator('CmdOrCtrl+Shift+S')!;
    expect(chordMatches(save, press('S', { ctrlKey: true, shiftKey: true }))).toBe(false);
    expect(chordMatches(saveAs, press('s', { ctrlKey: true }))).toBe(false);
  });

  // AltGr on a Windows German keyboard arrives as Ctrl+Alt, so a chord without
  // Alt must not fire on it.
  it('does not fire on AltGr (Ctrl+Alt)', () => {
    expect(
      chordMatches(parseAccelerator('CmdOrCtrl+O')!, press('o', { ctrlKey: true, altKey: true })),
    ).toBe(false);
  });

  it('needs the modifier and refuses an extra Meta', () => {
    const open = parseAccelerator('CmdOrCtrl+O')!;
    expect(chordMatches(open, press('o'))).toBe(false);
    expect(chordMatches(open, press('o', { ctrlKey: true, metaKey: true }))).toBe(false);
  });
});

describe('menuActionEnabled reads the flags the native menu is built with', () => {
  const flags: MenuFlags = {
    new: true,
    open: false,
    save: true,
    saveAs: true,
    export: false,
    settings: true,
  };

  it('answers with the flag of the action the menu id names', () => {
    expect(menuActionEnabled('menu.open', flags)).toBe(false);
    expect(menuActionEnabled('menu.save-as', flags)).toBe(true);
    expect(menuActionEnabled('menu.export', flags)).toBe(false);
  });

  it('refuses an id no document action answers to', () => {
    expect(menuActionEnabled('menu.nonsense', flags)).toBe(false);
  });
});

// CLAUDE.md's one-owner rule: the chord is declared once, on the menu item. The
// Windows mirror must therefore READ that declaration, never restate it — a
// chord literal here would be a second owner that drifts silently.
describe('the webview holds no chord of its own', () => {
  const CHORD_LITERAL =
    /['"`][^'"`\n]*\b(CmdOrCtrl|CommandOrControl|Ctrl|Control|Cmd|Command|Super|Meta|Shift|Alt|Option)\s*\+[^'"`\n]*['"`]/;
  const SINGLE_KEY_COMPARISON = /\bkey\s*===?\s*['"`][^'"`]['"`]/;

  it.each(['./menu-shortcuts.ts', './ipc.ts', '../App.svelte'])('%s', (file) => {
    const text = source(file);
    expect(text).not.toMatch(CHORD_LITERAL);
    expect(text).not.toMatch(SINGLE_KEY_COMPARISON);
  });
});
