// End-to-end: a short window must not make the tab area unusable. Found while
// measuring the list tabs' layout — at a window height of 600 the source (Available)
// picker's `.list-scroll` was flex-shrunk to a height of ZERO, so the whole
// selectable list vanished, and the panel's non-shrinkable head (title + filter bar)
// overflowed the panel by ~20px, which `.tab-content`'s `overflow: hidden` then
// clipped with no way to reach it.
//
// Two invariants, both about the same failure: the Available list keeps a usable
// height, and whatever still cannot fit stays REACHABLE — the tab area scrolls
// rather than clipping. Geometry is the subject, so only a real layout engine can
// check it.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, browser, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

// The default window is 1100x800 (crates/arm-app/tauri.conf.json). 600 is a
// plausible short window — a laptop screen with OS panels — and is where the source
// list measured zero.
const SHORT_HEIGHT = 600;
const DEFAULT_SIZE = { width: 1100, height: 800 };

// Under two source rows: this rejects the collapse, not a particular row height.
const MIN_USABLE_LIST = 60;

/** Geometry of the tab area and the source picker's scrolling list. */
function tabAreaMetrics() {
  return browser.execute(() => {
    const main = document.querySelector('main.tab-content');
    const list = document.querySelector('.region-source .list-scroll');
    const panel = document.querySelector('.region-source .panel');
    if (!main || !list || !panel) return null;
    return {
      mainScrollHeight: main.scrollHeight,
      mainClientHeight: main.clientHeight,
      // `hidden` means anything that does not fit is simply lost: no scrollbar, no
      // keyboard scroll, nothing to drag.
      mainOverflowY: getComputedStyle(main).overflowY,
      listClientHeight: list.clientHeight,
      panelClientHeight: panel.clientHeight,
      panelScrollHeight: panel.scrollHeight,
    };
  });
}

async function setWindowHeight(height) {
  await browser.setWindowSize(DEFAULT_SIZE.width, height);
  await browser.waitUntil(
    async () => (await browser.execute(() => window.innerHeight)) === height,
    {
      timeout: 10000,
      timeoutMsg: `the window should resize to ${height}px tall`,
    },
  );
}

describe('tab area at a short window height', () => {
  before(async () => {
    // A magus, whose Virtues & Flaws source list is the longest one the picker
    // renders — and freshly created, so the geometry measured below is that of an
    // untouched list rather than one the previous spec had filtered or filled.
    await startCharacter('magus');
    await $('[data-testid="tab-virtues_flaws"]').click();
    await $('.region-source .list-scroll').waitForExist({ timeout: 10000 });
  });

  // Always hand the following specs the window they expect, even on failure.
  after(async () => {
    await setWindowHeight(DEFAULT_SIZE.height);
  });

  it('keeps the Available list usable and anything clipped reachable', async () => {
    await setWindowHeight(SHORT_HEIGHT);
    const m = await tabAreaMetrics();
    expect(m).not.toBe(null);

    // 1. The selectable list did not collapse: a picker with no visible rows is a
    //    picker you cannot pick from.
    expect(m.listClientHeight).toBeGreaterThanOrEqual(MIN_USABLE_LIST);

    // 2. Whatever still does not fit is reachable. The panel's head (title + filter
    //    bar) has a min-content floor no amount of shrinking removes, so the tab area
    //    must scroll to it instead of clipping it away.
    if (m.mainScrollHeight > m.mainClientHeight) {
      expect(m.mainOverflowY).not.toBe('hidden');
    }
  });

  it('restores the full-height layout when the window grows back', async () => {
    await setWindowHeight(DEFAULT_SIZE.height);
    const m = await tabAreaMetrics();

    // At the default size everything fits, so no scrollbar appears on the tab area
    // and the list has room for many rows.
    expect(m.mainScrollHeight).toBeLessThanOrEqual(m.mainClientHeight + 1);
    expect(m.listClientHeight).toBeGreaterThan(MIN_USABLE_LIST);
    expect(m.panelScrollHeight).toBeLessThanOrEqual(m.panelClientHeight + 1);
  });
});
