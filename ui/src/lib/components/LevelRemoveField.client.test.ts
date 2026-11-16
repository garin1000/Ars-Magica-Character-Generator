import { flushSync, mount, unmount } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';

// manual-testing-findings-2026-11-15 #9 (F9): `LevelRemoveField` is the shared
// "level number + remove button" tail five hosts mount (`MagicPossessions`,
// `TalismanPanel`, `SupernaturalBeing`, `FamiliarPanel` — V30, full-audit round), and
// its `oninput` forwards `Number(raw)` unconditionally on every keystroke. Four of
// the five hosts pass an unsigned range (0..65535) where that is harmless, but
// `TalismanPanel`'s attunement bonus passes a SIGNED one (-128..127). A
// `type="number"` input's own value-sanitization algorithm reads an incomplete entry
// — a bare "-" typed on the way to "-4" — back as the empty string, so
// `event.currentTarget.value` is `''` at that instant even though the field still
// visibly shows the "-" (confirmed directly against happy-dom, which implements the
// same algorithm). `Number('')` is 0, so the current handler forwards a written 0 for
// it, and the host's own `clampInt` (`clamp.ts::clampInt`) plus its
// `value={levelValue}` binding then snaps the field back to "0", wiping the minus
// sign before the digit after it can ever be typed — the same "checks prevent proper
// typing" F9 reports, here in one shared component rather than five separate fixes.
// The fix is generic too: gate `oninput` on the field's own `levelMin`/`levelMax`
// props (already passed by every host) before calling `onLevelInput`, and forward the
// raw text unconditionally on commit so the host's own clamp still has the final
// word.
//
// A `client` test: the `input`/`change` listeners on a live element, asserting what
// `onLevelInput` is (and is not) called with — the component is host-agnostic, so
// this is the one place to prove the gate for every host at once, without the
// weight of mounting a full `TalismanPanel` + its store fixture. It takes no store
// or IPC dependency of its own (only `actions.ts::commitStored`), so neither needs
// mocking here.
import LevelRemoveField from './LevelRemoveField.svelte';

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function mountField(
  levelValue: number,
  levelMin: number,
  levelMax: number,
): { input: HTMLInputElement; onLevelInput: ReturnType<typeof vi.fn> } {
  const onLevelInput = vi.fn();
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(LevelRemoveField, {
    target,
    props: {
      levelLabel: 'Bonus',
      levelValue,
      levelMin,
      levelMax,
      levelTestid: 'level-field',
      onLevelInput,
      removeLabel: 'Remove',
      removeTestid: 'remove',
      onRemove: () => {},
    },
  });
  flushSync();
  const input = target.querySelector<HTMLInputElement>('[data-testid="level-field"]');
  expect(input).toBeTruthy();
  return { input: input!, onLevelInput };
}

function typeInto(input: HTMLInputElement, text: string): void {
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

describe('LevelRemoveField forwards only a value within its own range while typing (F9)', () => {
  // A number input reads the incomplete "-" back as '' (see the file banner); this
  // is that read, simulated directly.
  it('does not call onLevelInput on the empty read a mid-typed minus sign produces', () => {
    const { input, onLevelInput } = mountField(5, -128, 127);

    typeInto(input, '');
    expect(input.value, 'the field is left showing what the user typed').toBe('');
    expect(onLevelInput, 'no call for an unparsable prefix').not.toHaveBeenCalled();

    typeInto(input, '-4');
    expect(input.value).toBe('-4');
    expect(onLevelInput).toHaveBeenLastCalledWith(-4);
  });

  it('does not call onLevelInput past the max while typing, but does on commit', () => {
    const { input, onLevelInput } = mountField(5, -128, 127);

    typeInto(input, '200');
    expect(input.value, 'typing is never fought').toBe('200');
    expect(
      onLevelInput,
      'no call past the bound until the edit is committed',
    ).not.toHaveBeenCalled();

    commit(input);
    expect(onLevelInput).toHaveBeenCalledWith(200);
  });

  it('still forwards every keystroke of an ordinary in-range unsigned edit', () => {
    // The other four hosts pass 0..65535 — the gate must stay a no-op for them.
    const { input, onLevelInput } = mountField(0, 0, 65535);

    typeInto(input, '4');
    expect(onLevelInput).toHaveBeenLastCalledWith(4);
    typeInto(input, '45');
    expect(onLevelInput).toHaveBeenLastCalledWith(45);
  });
});
