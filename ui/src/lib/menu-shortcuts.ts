// The Windows-only webview mirror of the native menu's keyboard chords (U4,
// try-out finding 16).
//
// Every chord is declared once, as the menu item's accelerator
// (`crates/arm-app/src/menu.rs::accelerator_for`). On Linux and macOS the OS
// dispatches it. On Windows the menu draws it but never receives the key,
// because WebView2's child window keeps it. So Rust's `menu_shortcuts`
// command hands the frontend that same list. On Windows the list is the menu's
// own; everywhere else it is empty, so this module installs nothing there. A
// matching press runs the same action a menu click runs, gated by the same
// flags the native menu is built with.
//
// This module holds NO chord of its own. It only reads muda's notation, so the
// chords can never drift from what the menu draws. `menu-shortcuts.test.ts`
// guards that.

import { MENU_ACTIONS, type MenuFlags } from './menu';

/** One chord the webview answers to, and the menu action id it runs. Mirrors `arm_app::menu::MenuShortcut`. */
export interface MenuShortcut {
  action: string;
  accelerator: string;
}

/** A parsed accelerator: the key (lower-cased) and the modifiers it needs. */
export interface Chord {
  key: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
}

type ChordEvent = Pick<KeyboardEvent, 'key' | 'ctrlKey' | 'shiftKey' | 'altKey' | 'metaKey'>;
type Modifier = 'ctrl' | 'shift' | 'alt';

/**
 * muda's modifier tokens, as this mirror reads them. `CmdOrCtrl` is Ctrl
 * because the mirror only ever runs on Windows. The macOS meaning is Cmd, but
 * on macOS the list is empty and nothing is parsed.
 */
const MODIFIERS: Record<string, Modifier> = {
  cmdorctrl: 'ctrl',
  commandorcontrol: 'ctrl',
  ctrl: 'ctrl',
  control: 'ctrl',
  shift: 'shift',
  alt: 'alt',
};

/**
 * Read an accelerator in muda's notation: modifiers, then the key, joined by
 * a plus sign. Returns
 * `null` for anything it cannot read: an unknown modifier, or a missing key. A
 * chord it cannot read is skipped rather than guessed, because a wrong guess
 * would put the shortcut on the wrong key.
 */
export function parseAccelerator(accelerator: string): Chord | null {
  const tokens = accelerator.split('+');
  const key = tokens.pop()?.toLowerCase() ?? '';
  if (key === '' || key in MODIFIERS) return null;
  const chord: Chord = { key, ctrl: false, shift: false, alt: false };
  for (const token of tokens) {
    const modifier = MODIFIERS[token.toLowerCase()];
    if (modifier === undefined) return null;
    chord[modifier] = true;
  }
  return chord;
}

/**
 * Whether `event` is exactly `chord`. Every modifier must match, so Ctrl+S and
 * Ctrl+Shift+S stay apart. AltGr, which arrives as Ctrl+Alt on Windows, never
 * fires a Ctrl chord. The key is compared without case, because Shift makes the
 * browser report the capital letter.
 */
export function chordMatches(chord: Chord, event: ChordEvent): boolean {
  return (
    event.key.toLowerCase() === chord.key &&
    event.ctrlKey === chord.ctrl &&
    event.shiftKey === chord.shift &&
    event.altKey === chord.alt &&
    !event.metaKey
  );
}

/**
 * Whether the menu item `action` names is enabled under `flags`, which are the
 * flags the native menu is built with (`store.menuFlags()`). An id that maps to
 * no document action is never enabled.
 */
export function menuActionEnabled(action: string, flags: MenuFlags): boolean {
  const documentAction = MENU_ACTIONS[action];
  return documentAction !== undefined && flags[documentAction];
}

/**
 * Answer `shortcuts` on `target`'s `keydown`. Returns the teardown.
 *
 * - An empty list installs no listener at all. That is the Linux and macOS
 *   case, where the OS already dispatches the chord, so a listener would make
 *   it run twice.
 * - `isEnabled` is asked at the moment of each press, because the flags change
 *   at runtime.
 * - A withheld or unclaimed chord is left alone, with no `preventDefault`.
 */
export function installMenuShortcuts(
  target: EventTarget,
  shortcuts: MenuShortcut[],
  isEnabled: (action: string) => boolean,
  run: (action: string) => void,
): () => void {
  const bindings = shortcuts.flatMap(({ action, accelerator }) => {
    const chord = parseAccelerator(accelerator);
    return chord === null ? [] : [{ action, chord }];
  });
  if (bindings.length === 0) return () => {};

  const onKeydown = (event: Event): void => {
    const keyEvent = event as KeyboardEvent;
    const binding = bindings.find(({ chord }) => chordMatches(chord, keyEvent));
    if (binding === undefined || !isEnabled(binding.action)) return;
    keyEvent.preventDefault();
    run(binding.action);
  };
  target.addEventListener('keydown', onKeydown);
  return () => target.removeEventListener('keydown', onKeydown);
}
