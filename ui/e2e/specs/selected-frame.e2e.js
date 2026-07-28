// End-to-end: the Selected list must scroll INSIDE the golden frame, never bleed
// out of it. All four list tabs share one skeleton (`.region-row` ->
// `.region-selected` -> `.selected-frame`), so this checks it from two sides:
// Virtues & Flaws (two grouped columns inside the frame) and Equipment (a single
// flat list). Geometry is the whole subject, so it can only be tested against a
// real layout engine — vitest renders these components to an SSR string and never
// resolves a box.
//
// Provenance: written for a manual-testing report that the chosen rows run
// downwards past the frame's bottom border on all four tabs. It did NOT reproduce
// that — measured against the release binary at window heights 800 and 600 with 32
// chosen rows, the frame is bounded (client 299 / scroll 2275 at 800px) and its
// bottom stays inside the tab area, both before and after the `grid-auto-rows`
// change that was proposed as the fix. So this spec is a standing invariant guard,
// not the regression test for that report, which is still open.
//
// NOTE: requires a display + the production binary (see e2e/README.md). The wdio
// `onPrepare` hook builds `target/release/arm-app`.

import { $, $$, browser, expect } from '@wdio/globals';

const TYPE_SELECT = '[data-testid="type-select"]';
const NAME_INPUT = '[data-testid="identity-name"]';

// The frame and the tab area carry no data-testid: they ARE layout elements, and
// what this spec asserts is precisely the geometry of `.selected-frame` inside
// `main.tab-content` (ui/src/app.css). So both are addressed by class, the way the
// tooltip specs address `.tooltip-pop`.
const FRAME = '.region-selected > .selected-frame';
const TAB_CONTENT = 'main.tab-content';

// Enough rows that the list cannot possibly fit: the window is 1100x800 (see
// crates/arm-app/tauri.conf.json), which leaves the frame well under 500px, while
// a selected V/F row is ~44px and an equipment row ~34px. The exact numbers do not
// matter — `expectContainedScrollport` asserts the overflow really happened, so an
// insufficient count fails loudly instead of passing vacuously.
const VF_ROWS = 16;
const EQUIPMENT_ROWS = 20;

/**
 * Live geometry of the selected frame and the tab area that must contain it.
 * `scrollHeight`/`clientHeight` say whether the frame is its own scrollport;
 * the rects say whether it stays inside the tab area.
 */
async function frameMetrics() {
  const metrics = await browser.execute(
    (frameSelector, contentSelector) => {
      const frame = document.querySelector(frameSelector);
      const content = document.querySelector(contentSelector);
      if (!frame || !content) return null;
      const frameRect = frame.getBoundingClientRect();
      const contentRect = content.getBoundingClientRect();
      return {
        scrollHeight: frame.scrollHeight,
        clientHeight: frame.clientHeight,
        frameTop: frameRect.top,
        frameBottom: frameRect.bottom,
        contentBottom: contentRect.bottom,
      };
    },
    FRAME,
    TAB_CONTENT,
  );
  if (metrics === null) throw new Error('the selected frame or the tab area is not rendered');
  return metrics;
}

/**
 * The three things a correct frame does, given a list too long to fit.
 * Assertion 0 is the precondition that makes the other two meaningful.
 */
function expectContainedScrollport(metrics) {
  // 0. The list genuinely does not fit: its content is taller than the tallest
  //    frame that could ever sit between the frame's top and the bottom of the tab
  //    area. Without this, a frame with a handful of rows satisfies everything
  //    below without exercising the bug at all.
  const tallestPossibleFrame = metrics.contentBottom - metrics.frameTop;
  expect(metrics.scrollHeight).toBeGreaterThan(tallestPossibleFrame);

  // 1. The frame is the scrollport — the overflow is scrollable INSIDE it. This is
  //    what fails when the grid row sizes to content: the frame then grows to the
  //    full list height, so scrollHeight === clientHeight and nothing scrolls.
  expect(metrics.scrollHeight).toBeGreaterThan(metrics.clientHeight);

  // 2. And the frame stays inside the tab area, so its bottom border is on screen
  //    rather than clipped by `.tab-content { overflow: hidden }`. One pixel of
  //    tolerance for sub-pixel rect rounding.
  expect(metrics.frameBottom).toBeLessThanOrEqual(metrics.contentBottom + 1);

  // 3. The frame did not collapse the other way either: a `1fr` row that resolved
  //    to zero would satisfy 1 and 2 while showing an empty frame. 40px is under
  //    one row's height, so this only rejects a collapse.
  expect(metrics.clientHeight).toBeGreaterThan(40);
}

async function clickTab(id) {
  const tab = await $(`[data-testid="tab-${id}"]`);
  await tab.waitForExist({ timeout: 10000 });
  await tab.click();
}

/**
 * Back to an empty document. Specs share one app instance, so a marker name is
 * typed first: that guarantees the document is dirty, so New ALWAYS raises the
 * discard prompt and the reset needs no conditional branch (and, unlike saving
 * first, it never touches the JSON file the other specs round-trip).
 */
async function resetDocument() {
  await $(NAME_INPUT).setValue('selected-frame-reset');
  await $('[data-testid="new-button"]').click();
  const confirm = await $('[data-testid="discard-confirm"]');
  await confirm.waitForExist({ timeout: 10000 });
  await confirm.click();
  await browser.waitUntil(async () => (await $(NAME_INPUT).getValue()) === '', {
    timeout: 10000,
    timeoutMsg: 'New should reset the document',
  });
}

/**
 * The `data-testid`s of the first `count` ENABLED add controls in the active
 * source list, read in one round trip. Ids are never hardcoded: catalogue size and
 * contents are data, so the spec asks the rendered list what it can add.
 */
async function enabledAddIds(prefix, count) {
  const ids = await browser.execute(
    (testidPrefix, limit) =>
      Array.from(document.querySelectorAll(`[data-testid^="${testidPrefix}"]`))
        .filter((button) => !button.disabled)
        .slice(0, limit)
        .map((button) => button.getAttribute('data-testid')),
    prefix,
    count,
  );
  expect(ids.length).toBe(count);
  return ids;
}

describe('selected frame', () => {
  it('scrolls a long Virtues list inside the frame instead of past its bottom border', async () => {
    await $(TYPE_SELECT).waitForExist({ timeout: 30000 });
    await resetDocument();
    // A magus: the widest V/F catalogue, and the mandatory traits (The Gift,
    // Hermetic Magus) add their own rows to the selected side.
    await $(TYPE_SELECT).selectByAttribute('value', 'magus');

    await clickTab('virtues_flaws');
    const frame = await $(FRAME);
    await frame.waitForExist({ timeout: 10000 });

    // Fill the Virtues column past the frame's height. Over-budget is fine — the
    // engine reports it in the issues panel; nothing here depends on a legal build.
    // Spare candidates are fetched because a pick can grey a later one out (Major
    // vs Minor of the same Virtue and friends), and each is re-checked before the
    // click so an incompatible row is skipped rather than silently swallowed.
    for (const id of await enabledAddIds('add-virtue.', VF_ROWS + 8)) {
      if ((await $$('[data-testid^="remove-virtue."]')).length >= VF_ROWS) break;
      const add = await $(`[data-testid="${id}"]`);
      if (await add.isEnabled()) await add.click();
    }
    await browser.waitUntil(
      async () => (await $$('[data-testid^="remove-virtue."]')).length >= VF_ROWS,
      {
        timeout: 15000,
        timeoutMsg: `the selected side should hold ${VF_ROWS} chosen Virtues`,
      },
    );

    expectContainedScrollport(await frameMetrics());
  });

  it('scrolls a long Equipment list inside the frame instead of past its bottom border', async () => {
    await resetDocument();

    await clickTab('equipment');
    const frame = await $(FRAME);
    await frame.waitForExist({ timeout: 10000 });

    // Carried equipment is duplicable (`addEquipment` appends a slot per click), so
    // one catalogue entry clicked repeatedly fills the list — no assumption about
    // how many weapons/shields/armor the rules ship.
    const [addId] = await enabledAddIds('add-', 1);
    const add = await $(`[data-testid="${addId}"]`);
    for (let i = 0; i < EQUIPMENT_ROWS; i++) await add.click();
    await browser.waitUntil(
      async () => (await $$('[data-testid^="equipment-name-"]')).length >= EQUIPMENT_ROWS,
      {
        timeout: 15000,
        timeoutMsg: `the selected side should hold ${EQUIPMENT_ROWS} carried items`,
      },
    );

    expectContainedScrollport(await frameMetrics());

    // Leave a clean document behind: the specs after this one assert exact budgets
    // and would otherwise inherit these rows.
    await resetDocument();
  });
});
