import { afterEach, describe, expect, it, vi } from 'vitest';

import { installMenuShortcuts, type MenuShortcut } from './menu-shortcuts';

// U4 (try-out finding 16): the Windows-only webview mirror of the menu chords.
// A `client` test because it is about a live `keydown` listener on a real
// `window` and whether it cancels the event — nothing SSR can observe.
//
// The list is injected exactly as Rust's `menu_shortcuts` command would return
// it on Windows. On every other desktop that command returns `[]`, which is the
// "no listener at all" case below.

const WINDOWS_LIST: MenuShortcut[] = [
  { action: 'menu.new', accelerator: 'CmdOrCtrl+N' },
  { action: 'menu.open', accelerator: 'CmdOrCtrl+O' },
  { action: 'menu.save', accelerator: 'CmdOrCtrl+S' },
  { action: 'menu.save-as', accelerator: 'CmdOrCtrl+Shift+S' },
];

/** Dispatch a cancelable keydown on `window` and hand the event back. */
function press(key: string, modifiers: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent('keydown', {
    key,
    ctrlKey: true,
    bubbles: true,
    cancelable: true,
    ...modifiers,
  });
  window.dispatchEvent(event);
  return event;
}

let stop: (() => void) | undefined;

afterEach(() => {
  stop?.();
  stop = undefined;
  vi.restoreAllMocks();
});

describe('the Windows webview mirror of the menu chords', () => {
  it('runs the open action once on Ctrl+O and keeps the key from the webview', () => {
    const run = vi.fn();
    stop = installMenuShortcuts(window, WINDOWS_LIST, () => true, run);

    const event = press('o');

    expect(run).toHaveBeenCalledTimes(1);
    expect(run).toHaveBeenCalledWith('menu.open');
    expect(event.defaultPrevented).toBe(true);
  });

  it('runs Save As, not Save, on Ctrl+Shift+S', () => {
    const run = vi.fn();
    stop = installMenuShortcuts(window, WINDOWS_LIST, () => true, run);

    press('S', { shiftKey: true });

    expect(run).toHaveBeenCalledTimes(1);
    expect(run).toHaveBeenCalledWith('menu.save-as');
  });

  // The native menu greys a withheld item out and the OS refuses its
  // accelerator; the mirror must withhold it the same way — and leave the key
  // alone, since nothing claimed it.
  it('runs nothing and cancels nothing while the action is disabled', () => {
    const run = vi.fn();
    stop = installMenuShortcuts(window, WINDOWS_LIST, (action) => action !== 'menu.save', run);

    const event = press('s');

    expect(run).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
  });

  // The flags change at runtime (the startup screen withholds Save), so the
  // gate is asked at the press, not captured at install.
  it('asks the gate at the moment of the press', () => {
    const run = vi.fn();
    let saveEnabled = false;
    stop = installMenuShortcuts(window, WINDOWS_LIST, () => saveEnabled, run);

    press('s');
    expect(run).not.toHaveBeenCalled();

    saveEnabled = true;
    press('s');
    expect(run).toHaveBeenCalledTimes(1);
  });

  it('leaves a chord the menu does not claim to the platform', () => {
    const run = vi.fn();
    stop = installMenuShortcuts(window, WINDOWS_LIST, () => true, run);

    const event = press('e');

    expect(run).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
  });

  // Linux and macOS: the accelerator fires natively, so a listener here would
  // be a second owner of the chord and run every action twice.
  it('installs no listener at all for an empty list', () => {
    const add = vi.spyOn(window, 'addEventListener');
    const run = vi.fn();
    stop = installMenuShortcuts(window, [], () => true, run);

    const event = press('o');

    expect(add).not.toHaveBeenCalled();
    expect(run).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
  });

  it('stops answering once torn down', () => {
    const run = vi.fn();
    installMenuShortcuts(window, WINDOWS_LIST, () => true, run)();

    const event = press('o');

    expect(run).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
  });
});
