import { afterEach, describe, expect, it, vi } from 'vitest';

import { commitStored } from './actions';

// N2 (try-out 2026-10-04): a number field bound one-way to a clamping store setter
// showed what was typed, not what was stored, whenever the clamp left the stored
// value unchanged — at 500 the age field took a fifth digit and showed "5000",
// because Svelte had no new value to render. `commitStored` writes the stored value
// back into the field when the edit is committed (blur or Enter: the `change`
// event), and never while typing, so a prefix is never fought.
//
// A `client` test: the action listens on a live element and writes its `value`,
// which only a real `document` has; `svelte/server` never runs an action.

let node: HTMLInputElement;
let lifecycle: ReturnType<typeof commitStored> | undefined;

afterEach(() => {
  lifecycle?.destroy?.();
  lifecycle = undefined;
  node?.remove();
});

function mountInput(params: Parameters<typeof commitStored>[1]): HTMLInputElement {
  node = document.createElement('input');
  node.type = 'number';
  document.body.appendChild(node);
  lifecycle = commitStored(node, params);
  return node;
}

/** What a keystroke does: the field's text changes and `input` fires. */
function typeText(text: string): void {
  node.value = text;
  node.dispatchEvent(new Event('input', { bubbles: true }));
}

/** What blur or Enter does after an edit. */
function commitEdit(): void {
  node.dispatchEvent(new Event('change', { bubbles: true }));
}

describe('commitStored writes the stored value back on commit (N2)', () => {
  it('shows the stored value once the edit is committed, not the typed one', () => {
    let stored = 500;
    mountInput({ read: () => stored });
    typeText('5000');

    commitEdit();
    expect(node.value).toBe('500');
    stored = 7;
    expect(node.value, 'the write-back happens on commit, not on every read').toBe('500');
  });

  it('runs the commit with the typed text first, then shows what it stored', () => {
    let stored: number | null = 500;
    const commit = vi.fn((raw: string) => {
      stored = Math.min(500, Number(raw));
    });
    mountInput({ read: () => stored, commit });
    typeText('6000');

    commitEdit();
    expect(commit).toHaveBeenCalledTimes(1);
    expect(commit).toHaveBeenCalledWith('6000');
    expect(node.value).toBe('500');
  });

  it('shows an empty field for a stored null or undefined', () => {
    let stored: number | null | undefined = null;
    mountInput({ read: () => stored });
    typeText('0');
    commitEdit();
    expect(node.value).toBe('');

    stored = undefined;
    typeText('3');
    commitEdit();
    expect(node.value).toBe('');
  });

  it('puts the stored value back into a field the user emptied', () => {
    mountInput({ read: () => 1220 });
    typeText('');

    commitEdit();
    expect(node.value).toBe('1220');
  });

  it('neither commits nor rewrites while the user is typing', () => {
    const commit = vi.fn();
    mountInput({ read: () => 720, commit });

    typeText('1');
    typeText('11');
    typeText('119');
    expect(commit).not.toHaveBeenCalled();
    expect(node.value, 'a prefix must stay as typed').toBe('119');
  });

  // A loaded save can hold a value the setter would now clamp (a birth year after the
  // saga year). Focusing and leaving that field without typing must not rewrite it:
  // that would change the document — and dirty it — with nothing edited.
  it('skips the commit when the field still shows the stored value', () => {
    const commit = vi.fn();
    mountInput({ read: () => 1250, commit });
    node.value = '1250';

    commitEdit();
    expect(commit).not.toHaveBeenCalled();
    expect(node.value).toBe('1250');
  });

  it('uses the parameters of the latest update', () => {
    const first = vi.fn();
    const second = vi.fn();
    mountInput({ read: () => 1, commit: first });
    lifecycle?.update?.({ read: () => 2, commit: second });
    typeText('9');

    commitEdit();
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledWith('9');
    expect(node.value).toBe('2');
  });

  it('stops listening once destroyed', () => {
    const commit = vi.fn();
    mountInput({ read: () => 500, commit });
    lifecycle?.destroy?.();
    lifecycle = undefined;
    typeText('5000');

    commitEdit();
    expect(commit).not.toHaveBeenCalled();
    expect(node.value).toBe('5000');
  });
});
