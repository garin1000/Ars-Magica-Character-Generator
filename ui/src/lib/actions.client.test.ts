import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { tooltip } from './actions';

// The `tooltip` action manipulates real DOM (`document.createElement`,
// `document.body.appendChild`, event listeners) entirely outside Svelte's own
// render cycle — `svelte/server`'s `render()` never mounts an action at all,
// so this can only be proven with a live `document`. Belongs in the `client`
// project (happy-dom).

/** Try-out finding 3 (Norbert): hover AND keyboard focus wait half a second. */
const OPEN_DELAY_MS = 500;
/** Try-out finding 2: how long the popup survives the pointer leaving it or its trigger. */
const CLOSE_GRACE_MS = 150;

let node: HTMLButtonElement;
let lifecycle: ReturnType<typeof tooltip> | undefined;

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  lifecycle?.destroy?.();
  lifecycle = undefined;
  node?.remove();
  document.querySelectorAll('.tooltip-pop').forEach((el) => el.remove());
  vi.restoreAllMocks();
  vi.useRealTimers();
});

function popup(): HTMLElement | null {
  return document.querySelector<HTMLElement>('.tooltip-pop');
}

function tooltipText(): HTMLElement {
  const text = document.querySelector<HTMLElement>('[data-testid="tooltip-text"]');
  expect(text, 'the open popup should carry its description').not.toBeNull();
  return text!;
}

const FLAME = { text: 'A bolt of flame.' };

/** A focusable trigger carrying the tooltip action. No default for `content`:
 *  an explicit `undefined` must reach the action as "no content". */
function mountTrigger(content: Parameters<typeof tooltip>[1]) {
  node = document.createElement('button');
  document.body.appendChild(node);
  lifecycle = tooltip(node, content);
}

function hover(el: Element): void {
  el.dispatchEvent(new MouseEvent('mouseenter'));
}

function unhover(el: Element): void {
  el.dispatchEvent(new MouseEvent('mouseleave'));
}

/** Focus the trigger and let the open delay run out. */
function openByFocus(): void {
  node.dispatchEvent(new FocusEvent('focusin'));
  vi.advanceTimersByTime(OPEN_DELAY_MS);
}

/** Hover the trigger and let the open delay run out; returns the opened popup. */
function openByHover(): HTMLElement {
  hover(node);
  vi.advanceTimersByTime(OPEN_DELAY_MS);
  const pop = popup();
  expect(pop, 'the popup should be open after the hover delay').not.toBeNull();
  return pop!;
}

/** Pin the trigger's viewport box, so the popup's placement can be measured. */
function placeTrigger(top: number, bottom: number): void {
  vi.spyOn(node, 'getBoundingClientRect').mockReturnValue({
    top,
    bottom,
    left: 50,
    right: 150,
    width: 100,
    height: bottom - top,
    x: 50,
    y: top,
    toJSON: () => ({}),
  } as DOMRect);
}

// S2 (full-audit a11y): the shared tooltip action (used on every picker row
// app-wide) opened on hover/focus but offered no keyboard way to dismiss it
// short of moving focus elsewhere — a sighted keyboard user had no Escape.
describe('tooltip action Escape dismissal (S2)', () => {
  it('shows the popup once focus has rested on the trigger', () => {
    mountTrigger(FLAME);

    openByFocus();
    expect(popup()).not.toBeNull();
  });

  it('hides the popup on Escape while the trigger holds it open', () => {
    mountTrigger(FLAME);

    openByFocus();
    expect(popup()).not.toBeNull();

    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    expect(popup()).toBeNull();
  });

  it('clears the aria-describedby link Escape leaves behind, same as any other hide', () => {
    mountTrigger(FLAME);

    openByFocus();
    expect(node.hasAttribute('aria-describedby')).toBe(true);

    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    expect(node.hasAttribute('aria-describedby')).toBe(false);
  });

  it('ignores every other key, never dismissing the popup by accident', () => {
    mountTrigger(FLAME);

    openByFocus();
    expect(popup()).not.toBeNull();

    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true }));
    expect(popup()).not.toBeNull();
  });

  it('is a no-op with no content, exactly like the other handlers', () => {
    mountTrigger(undefined);

    openByFocus();
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
    mountTrigger(FLAME);

    openByFocus();
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

// Try-out finding 3 (Norbert, decided): passing the pointer over a list flashed
// one popup per row. The popup now waits half a second — for hover AND for
// keyboard focus alike — and leaving or blurring before then cancels it.
describe('tooltip open delay (finding 3)', () => {
  it('opens on hover only once the pointer has rested for 500 ms', () => {
    mountTrigger(FLAME);

    hover(node);
    vi.advanceTimersByTime(OPEN_DELAY_MS - 1);
    expect(popup(), 'no popup before the delay has run out').toBeNull();

    vi.advanceTimersByTime(1);
    expect(popup()).not.toBeNull();
    expect(node.getAttribute('aria-describedby')).toBe(popup()!.id);
  });

  it('opens on keyboard focus only after the same 500 ms', () => {
    mountTrigger(FLAME);

    node.dispatchEvent(new FocusEvent('focusin'));
    vi.advanceTimersByTime(OPEN_DELAY_MS - 1);
    expect(popup(), 'no popup before the delay has run out').toBeNull();

    vi.advanceTimersByTime(1);
    expect(popup()).not.toBeNull();
    expect(node.getAttribute('aria-describedby')).toBe(popup()!.id);
  });

  it('never opens when the pointer leaves before the delay runs out', () => {
    mountTrigger(FLAME);

    hover(node);
    vi.advanceTimersByTime(300);
    expect(popup()).toBeNull();
    unhover(node);

    vi.advanceTimersByTime(OPEN_DELAY_MS * 2);
    expect(popup()).toBeNull();
  });

  it('never opens when focus leaves before the delay runs out', () => {
    mountTrigger(FLAME);

    node.dispatchEvent(new FocusEvent('focusin'));
    vi.advanceTimersByTime(300);
    expect(popup()).toBeNull();
    node.dispatchEvent(new FocusEvent('focusout'));

    vi.advanceTimersByTime(OPEN_DELAY_MS * 2);
    expect(popup()).toBeNull();
  });

  // `vi.getTimerCount()` is no witness here: happy-dom schedules timers of its own
  // when the DOM mutates, so the count moves with the popup. What matters is the
  // effect a leaked timer would have — a popup appearing for a destroyed trigger.
  it('destroy cancels a pending open, so no popup appears for a destroyed trigger', () => {
    mountTrigger(FLAME);

    node.dispatchEvent(new FocusEvent('focusin'));
    vi.advanceTimersByTime(300);
    expect(popup(), 'the open is still pending').toBeNull();

    lifecycle?.destroy?.();
    lifecycle = undefined;

    vi.advanceTimersByTime(OPEN_DELAY_MS * 2);
    expect(popup()).toBeNull();
  });

  it('destroy removes a popup in its close grace at once, and nothing fires later', () => {
    mountTrigger(FLAME);
    openByHover();

    unhover(node);
    expect(popup(), 'the close is still pending').not.toBeNull();

    lifecycle?.destroy?.();
    lifecycle = undefined;
    expect(popup()).toBeNull();
    expect(node.hasAttribute('aria-describedby')).toBe(false);

    expect(() => vi.advanceTimersByTime(CLOSE_GRACE_MS * 2)).not.toThrow();
    expect(popup()).toBeNull();
  });
});

// Try-out finding 2: a long description scrolls inside the popup, but the popup
// ignored the pointer and sat 6px below its trigger, closing the moment the
// pointer left the trigger to travel there — its scrollbar was unreachable.
describe('tooltip hover bridge (finding 2)', () => {
  it('opens flush against the trigger, leaving no gap the pointer must cross', () => {
    mountTrigger(FLAME);
    placeTrigger(100, 120);

    const pop = openByHover();
    const top = parseFloat(pop.style.top);
    expect(top, 'the popup must touch the trigger (overlap of 1px at most)').toBeLessThanOrEqual(
      120,
    );
    expect(top).toBeGreaterThanOrEqual(119);
  });

  it('flipped above a trigger near the bottom edge, it still touches the trigger', () => {
    const POP_HEIGHT = 200;
    vi.spyOn(HTMLElement.prototype, 'offsetHeight', 'get').mockImplementation(function (
      this: HTMLElement,
    ) {
      return this.classList.contains('tooltip-pop') ? POP_HEIGHT : 0;
    });
    mountTrigger(FLAME);
    const triggerTop = window.innerHeight - 40;
    placeTrigger(triggerTop, triggerTop + 20);

    const pop = openByHover();
    const popBottom = parseFloat(pop.style.top) + POP_HEIGHT;
    expect(popBottom, 'the flipped popup must touch the trigger').toBeGreaterThanOrEqual(
      triggerTop,
    );
    expect(popBottom).toBeLessThanOrEqual(triggerTop + 1);
  });

  it('stays open while the pointer travels from the trigger into the popup', () => {
    mountTrigger(FLAME);
    const pop = openByHover();

    unhover(node);
    vi.advanceTimersByTime(CLOSE_GRACE_MS - 50);
    expect(popup(), 'still open inside the close grace').not.toBeNull();

    hover(pop);
    vi.advanceTimersByTime(OPEN_DELAY_MS * 4);
    expect(popup(), 'stays open while the pointer is over it').toBe(pop);
  });

  it('closes once the grace runs out when the pointer leaves the trigger for elsewhere', () => {
    mountTrigger(FLAME);
    openByHover();

    unhover(node);
    vi.advanceTimersByTime(CLOSE_GRACE_MS - 1);
    expect(popup(), 'still open inside the close grace').not.toBeNull();

    vi.advanceTimersByTime(1);
    expect(popup()).toBeNull();
    expect(node.hasAttribute('aria-describedby')).toBe(false);
  });

  it('closes after the grace once the pointer leaves the popup', () => {
    mountTrigger(FLAME);
    const pop = openByHover();
    unhover(node);
    hover(pop);

    unhover(pop);
    vi.advanceTimersByTime(CLOSE_GRACE_MS - 1);
    expect(popup(), 'still open inside the close grace').not.toBeNull();

    vi.advanceTimersByTime(1);
    expect(popup()).toBeNull();
  });

  it('stays open when the pointer returns from the popup to the trigger', () => {
    mountTrigger(FLAME);
    const pop = openByHover();
    unhover(node);
    hover(pop);

    unhover(pop);
    vi.advanceTimersByTime(CLOSE_GRACE_MS - 100);
    hover(node);
    vi.advanceTimersByTime(OPEN_DELAY_MS * 4);
    // The same popup, never closed and rebuilt behind a fresh open delay.
    expect(popup()).toBe(pop);
  });

  it('keeps the popup open while its own text is scrolled', () => {
    mountTrigger(FLAME);
    openByHover();

    // `scroll` does not bubble; the action's window listener sees it in the
    // capture phase, exactly as it sees a page scroll.
    tooltipText().dispatchEvent(new Event('scroll'));
    expect(popup(), 'scrolling inside the popup must not close it').not.toBeNull();
  });

  it('still closes on a page scroll outside the popup', () => {
    mountTrigger(FLAME);
    openByHover();

    document.dispatchEvent(new Event('scroll'));
    expect(popup()).toBeNull();
  });
});

// U1 follow-up: the action used to `removeAttribute('aria-describedby')` on every
// close, wiping a static description the element carried itself (an error hint,
// a "why disabled" reason). It must add its popup id beside that value while
// open and put the previous value back on close and on destroy.
describe('tooltip keeps an element’s own aria-describedby', () => {
  function mountDescribedTrigger(): void {
    mountTrigger(FLAME);
    node.setAttribute('aria-describedby', 'static-hint');
  }

  it('adds the popup beside the static value while open', () => {
    mountDescribedTrigger();
    openByFocus();

    const ids = (node.getAttribute('aria-describedby') ?? '').split(/\s+/);
    expect(ids).toContain('static-hint');
    expect(ids).toContain(popup()!.id);
  });

  it('restores the static value when the popup closes', () => {
    mountDescribedTrigger();
    openByFocus();

    node.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    expect(node.getAttribute('aria-describedby')).toBe('static-hint');
  });

  it('restores the static value when destroyed while open', () => {
    mountDescribedTrigger();
    openByFocus();

    lifecycle?.destroy?.();
    lifecycle = undefined;
    expect(node.getAttribute('aria-describedby')).toBe('static-hint');
  });

  it('leaves the static value alone when destroyed while closed', () => {
    mountDescribedTrigger();

    lifecycle?.destroy?.();
    lifecycle = undefined;
    expect(node.getAttribute('aria-describedby')).toBe('static-hint');
  });
});

// Try-out finding 2, keyboard half (decided): a keyboard user cannot drag a
// scrollbar, so while the popup is open from focus PageUp/PageDown scroll its
// text. The full text also stays reachable through `aria-describedby`.
describe('tooltip keyboard scrolling (finding 2)', () => {
  /** Give the open popup's text a scrollable box: 100px tall, 1000px of content. */
  function makeTextScrollable(): HTMLElement {
    const text = tooltipText();
    Object.defineProperty(text, 'clientHeight', { configurable: true, value: 100 });
    Object.defineProperty(text, 'scrollHeight', { configurable: true, value: 1000 });
    return text;
  }

  function press(key: string): KeyboardEvent {
    const event = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true });
    node.dispatchEvent(event);
    return event;
  }

  it('scrolls the open popup text down with PageDown and back up with PageUp', () => {
    mountTrigger(FLAME);
    openByFocus();
    const text = makeTextScrollable();

    const down = press('PageDown');
    expect(text.scrollTop, 'PageDown should scroll the popup text').toBeGreaterThan(0);
    expect(down.defaultPrevented, 'the page itself must not scroll too').toBe(true);
    expect(popup(), 'scrolling by key must not close the popup').not.toBeNull();

    const scrolled = text.scrollTop;
    const up = press('PageUp');
    expect(text.scrollTop).toBeLessThan(scrolled);
    expect(up.defaultPrevented).toBe(true);
  });

  it('leaves PageUp/PageDown alone while no popup is open', () => {
    mountTrigger(FLAME);

    // Focused, but the open delay has not run out yet: nothing to scroll.
    node.dispatchEvent(new FocusEvent('focusin'));
    expect(press('PageDown').defaultPrevented).toBe(false);
    expect(press('PageUp').defaultPrevented).toBe(false);
  });
});
