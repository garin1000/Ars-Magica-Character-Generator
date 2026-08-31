import { afterEach, describe, expect, it } from 'vitest';

import { tooltip } from './actions';

// The `tooltip` action manipulates real DOM (`document.createElement`,
// `document.body.appendChild`, event listeners) entirely outside Svelte's own
// render cycle — `svelte/server`'s `render()` never mounts an action at all,
// so this can only be proven with a live `document`. Belongs in the `client`
// project (happy-dom).

let node: HTMLButtonElement;
let lifecycle: ReturnType<typeof tooltip> | undefined;

afterEach(() => {
  lifecycle?.destroy?.();
  lifecycle = undefined;
  node?.remove();
  document.querySelectorAll('.tooltip-pop').forEach((el) => el.remove());
});

function popup(): Element | null {
  return document.querySelector('.tooltip-pop');
}

// S2 (full-audit a11y): the shared tooltip action (used on every picker row
// app-wide) opened on hover/focus but offered no keyboard way to dismiss it
// short of moving focus elsewhere — a sighted keyboard user had no Escape.
describe('tooltip action Escape dismissal (S2)', () => {
  it('shows the popup on focusin', () => {
    node = document.createElement('button');
    document.body.appendChild(node);
    lifecycle = tooltip(node, { text: 'A bolt of flame.' });

    node.dispatchEvent(new FocusEvent('focusin'));
    expect(popup()).not.toBeNull();
  });

  it('hides the popup on Escape while the trigger holds it open', () => {
    node = document.createElement('button');
    document.body.appendChild(node);
    lifecycle = tooltip(node, { text: 'A bolt of flame.' });

    node.dispatchEvent(new FocusEvent('focusin'));
    expect(popup()).not.toBeNull();

    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    expect(popup()).toBeNull();
  });

  it('clears the aria-describedby link Escape leaves behind, same as any other hide', () => {
    node = document.createElement('button');
    document.body.appendChild(node);
    lifecycle = tooltip(node, { text: 'A bolt of flame.' });

    node.dispatchEvent(new FocusEvent('focusin'));
    expect(node.hasAttribute('aria-describedby')).toBe(true);

    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    expect(node.hasAttribute('aria-describedby')).toBe(false);
  });

  it('ignores every other key, never dismissing the popup by accident', () => {
    node = document.createElement('button');
    document.body.appendChild(node);
    lifecycle = tooltip(node, { text: 'A bolt of flame.' });

    node.dispatchEvent(new FocusEvent('focusin'));
    expect(popup()).not.toBeNull();

    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }));
    expect(popup()).not.toBeNull();
  });

  it('is a no-op with no content, exactly like the other handlers', () => {
    node = document.createElement('button');
    document.body.appendChild(node);
    lifecycle = tooltip(node, undefined);

    node.dispatchEvent(new FocusEvent('focusin'));
    expect(popup()).toBeNull();

    // Must not throw with nothing open to hide.
    expect(() =>
      node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true })),
    ).not.toThrow();
  });

  // A leaked document-level listener would be a worse bug than the one this
  // finding fixes — the Escape handler must be wired into the SAME
  // add/removeEventListener pair on `node` as mouseenter/focusin, so
  // `destroy()` tears it down exactly like the others.
  it('removes the keydown listener on destroy, not just mouseenter/focusin', () => {
    node = document.createElement('button');
    document.body.appendChild(node);
    lifecycle = tooltip(node, { text: 'A bolt of flame.' });

    node.dispatchEvent(new FocusEvent('focusin'));
    expect(popup()).not.toBeNull();

    lifecycle?.destroy?.();
    lifecycle = undefined;

    // The popup from before destroy is still in the document (destroy calls
    // hide() itself, matching the other listeners' teardown) — the point here
    // is that a keydown AFTER destroy no longer does anything, proving the
    // listener was actually removed rather than merely made a no-op.
    const stray = document.createElement('div');
    stray.className = 'tooltip-pop';
    document.body.appendChild(stray);
    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    expect(document.querySelector('.tooltip-pop')).toBe(stray);
    stray.remove();
  });
});
