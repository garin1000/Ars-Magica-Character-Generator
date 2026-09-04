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

// The whole-app language control in the header.
const LANG_SELECT = '[data-testid="language-select"]';

/** Geometry of the tab area and the source picker's scrolling list. */
function tabAreaMetrics() {
  return browser.execute(() => {
    const main = document.querySelector('main.tab-content');
    const list = document.querySelector('.region-source .list-scroll');
    const panel = document.querySelector('.region-source .panel');
    const tabbar = document.querySelector('.tabbar');
    if (!main || !list || !panel || !tabbar) return null;
    return {
      mainScrollHeight: main.scrollHeight,
      mainClientHeight: main.clientHeight,
      // `hidden` means anything that does not fit is simply lost: no scrollbar, no
      // keyboard scroll, nothing to drag.
      mainOverflowY: getComputedStyle(main).overflowY,
      listClientHeight: list.clientHeight,
      panelClientHeight: panel.clientHeight,
      panelScrollHeight: panel.scrollHeight,
      // The strip sits above every panel, so its height is subtracted from theirs.
      tabbarScrollHeight: tabbar.scrollHeight,
      tabbarClientHeight: tabbar.clientHeight,
      tabbarRight: tabbar.getBoundingClientRect().right,
      // Whether each tab's own centre hit-tests to that tab: the property a click
      // depends on, and the one an overflow scrollbar's hit area destroys.
      tabsHittableAtCentre: [...tabbar.querySelectorAll('[role="tab"]')].map((tab) => {
        const rect = tab.getBoundingClientRect();
        const hit = document.elementFromPoint(
          rect.left + rect.width / 2,
          rect.top + rect.height / 2,
        );
        return [tab.id, rect.right, hit ? tab.contains(hit) || hit === tab : false];
      }),
      // `.tab` is `overflow: hidden` + `text-overflow: ellipsis` on one nowrap line,
      // so a label wider than its box is exactly a label wearing an ellipsis. Each
      // entry carries the rendered text too, so a failure names the offender rather
      // than only its id.
      tabsTruncated: [...tabbar.querySelectorAll('[role="tab"]')]
        .filter((tab) => tab.scrollWidth > tab.clientWidth + 1)
        .map((tab) => [tab.id, tab.textContent.trim(), tab.scrollWidth, tab.clientWidth]),
      // Both dimensions of the smallest tab, against WCAG 2.5.8's 24x24 CSS px.
      smallestTabBox: [...tabbar.querySelectorAll('[role="tab"]')].reduce(
        (smallest, tab) => {
          const rect = tab.getBoundingClientRect();
          return [Math.min(smallest[0], rect.width), Math.min(smallest[1], rect.height)];
        },
        [Infinity, Infinity],
      ),
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

  // Slice 3 (#28) took the magus tab count to thirteen, whose labels are ~1350px of
  // text against ~1050px of room. Two failure modes were measured here at the
  // default window, and this test rejects both:
  //
  //  1. Left to the flex defaults the buttons shrink below their text width and the
  //     text wraps INSIDE them, so the strip gains a second line and every panel
  //     below it loses that height — which is what broke the full-height assertion
  //     below by 25px.
  //  2. Making the strip scroll sideways instead fixes the height and breaks the
  //     controls: WebKitGTK's overlay horizontal scrollbar claims the hit area
  //     across the bottom of the scroll container while taking no layout height, so
  //     on a 40px strip everything below ~19px stopped hit-testing to the button and
  //     the lower half of every tab became unclickable.
  //
  // Both invariants are structural — no vertical overflow, every tab hit-testable at
  // its own centre and inside the strip — never a pixel count.
  it('keeps the tab strip one line tall with every tab clickable', async () => {
    await setWindowHeight(DEFAULT_SIZE.height);
    const m = await tabAreaMetrics();

    expect(m.tabbarScrollHeight).toBeLessThanOrEqual(m.tabbarClientHeight + 1);
    // The offenders are collected rather than asserted one by one, so a failure
    // names the tabs instead of stopping at the first.
    expect(m.tabsHittableAtCentre.filter(([, , hit]) => !hit).map(([id]) => id)).toEqual([]);
    expect(
      m.tabsHittableAtCentre.filter(([, right]) => right > m.tabbarRight + 1).map(([id]) => id),
    ).toEqual([]);
  });

  // manual-testing-findings-2026-09 #1: the strip fitting is not the same claim as
  // the strip being ONE LINE — the test above was green throughout, because the
  // labels were ellipsizing rather than wrapping. GERMAN is the binding locale (the
  // magus set is ~146 characters of label against English's ~130), and it is the one
  // no earlier assertion here ever rendered, so English fitting proved nothing about
  // the case that actually failed. `app.css.test.ts` budgets this arithmetically
  // from the same `.ftl` strings; only a real engine can confirm the arithmetic.
  it('shows every tab label in full, in German as well as English', async () => {
    await setWindowHeight(DEFAULT_SIZE.height);

    const english = await tabAreaMetrics();
    expect(english.tabsTruncated).toEqual([]);
    // The type shrank to fit; the button must not have shrunk with it.
    expect(english.smallestTabBox[0]).toBeGreaterThanOrEqual(24);
    expect(english.smallestTabBox[1]).toBeGreaterThanOrEqual(24);

    await $(LANG_SELECT).selectByAttribute('value', 'de');
    await browser.waitUntil(async () => (await $(LANG_SELECT).getValue()) === 'de', {
      timeout: 5000,
      timeoutMsg: 'the language should switch to German',
    });
    // Wait for a label the two locales spell differently, so the assertions below
    // cannot race the re-render and measure English boxes.
    await browser.waitUntil(
      async () => (await $('[data-testid="tab-abilities"]').getText()).trim() === 'Fertigkeiten',
      { timeout: 5000, timeoutMsg: 'the tab strip should re-render in German' },
    );

    const german = await tabAreaMetrics();
    expect(german.tabsTruncated).toEqual([]);
    expect(german.tabbarScrollHeight).toBeLessThanOrEqual(german.tabbarClientHeight + 1);
    expect(
      german.tabsHittableAtCentre
        .filter(([, right]) => right > german.tabbarRight + 1)
        .map(([id]) => id),
    ).toEqual([]);
    expect(german.smallestTabBox[0]).toBeGreaterThanOrEqual(24);
    expect(german.smallestTabBox[1]).toBeGreaterThanOrEqual(24);

    // Hand the next spec in this file the language it expects.
    await $(LANG_SELECT).selectByAttribute('value', 'en');
    await browser.waitUntil(async () => (await $(LANG_SELECT).getValue()) === 'en', {
      timeout: 5000,
      timeoutMsg: 'the language should switch back to English',
    });
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
