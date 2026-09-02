// End-to-end: what the guided flow TELLS the player (Slice 11 —
// guided-creation-review-2026-08 #7, #12, #30), plus the geometry lock that came out
// of the same slice's re-measure.
//
// Four things only the running app can show:
//
//  - #7  The Virtues & Flaws step states the character type's own Story and
//        Personality Flaw caps, generated from the profile — so a grog and a magus
//        read different numbers off one code path, and no locale string carries a
//        rules figure. The magus's also states the Hermetic-Flaw recommendation.
//  - #12 The Hermetic minimums are one collapsed line that expands on demand, and its
//        count matches the rows it heads (it used to read "N of 7" above a list of
//        three). Only a real disclosure can be opened.
//  - #30 Leaving experience or spell levels unspent raises a WARNING on the owning
//        step and again on Review, and Finish stays available. This is the whole
//        finding pipeline — engine, IPC, phase filter, Fluent — so an `issue-<code>`
//        missing from a locale shows up here as a slug on screen.
//  - The floors `.region-row { min-height }` / `.list-scroll { min-height }` are still
//        load-bearing on the magus Abilities step at 800px, which is what the Slice 11
//        re-measure established (see `app.css`'s comments for the numbers). Locked so
//        the next reader does not have to re-derive it in a ten-minute run.
//
// NOTE: requires the production binary; the display comes from your desktop session
// or, when DISPLAY is unset, the Xvfb one WebdriverIO starts (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, $$, browser, expect } from '@wdio/globals';

import {
  advanceWizardTo,
  clean,
  satisfyMagusMinimums,
  standOnWizardStep,
  startWizard,
  STEP_TIMEOUT,
} from '../helpers.js';

const GUIDANCE = '[data-testid="wizard-guidance"]';
const MINIMUMS = '[data-testid="magus-minimums"]';
const SUMMARY = '[data-testid="magus-minimums-summary"]';
const FINISH = '[data-testid="wizard-finish"]';

/** The `data-code` of every finding the panel is currently showing. */
async function shownCodes() {
  const rows = await $$('[data-testid="issue-list"] li');
  const codes = [];
  for (const row of Array.from(rows)) codes.push(await row.getAttribute('data-code'));
  return codes;
}

/** Wait until `code` is on screen, and hand back its rendered text. */
async function waitForFinding(code) {
  let codes = [];
  await browser.waitUntil(
    async () => {
      codes = await shownCodes();
      return codes.includes(code);
    },
    { timeout: STEP_TIMEOUT, timeoutMsg: () => `'${code}' never appeared; showing ${codes}` },
  );
  return clean(await $(`[data-testid="issue-list"] li[data-code="${code}"]`).getText());
}

async function setWindowHeight(height) {
  await browser.setWindowSize(1100, height);
  await browser.waitUntil(
    async () => (await browser.execute(() => window.innerHeight)) === height,
    {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `the window should resize to ${height}px tall`,
    },
  );
}

describe('guided guidance and unspent-budget findings (slice 11)', () => {
  it("states a grog's own Flaw caps and claims no Story Flaw minimum (#7)", async () => {
    await startWizard('grog');
    await advanceWizardTo('virtues_flaws');
    const guidance = clean(await $(GUIDANCE).getText());

    // Real prose from the locale, never a Fluent key echoed back — which is what a
    // missing `wizard-guidance-<category>-flaw-cap` would look like on screen.
    expect(guidance).not.toContain('wizard-guidance');
    // Core Rules :2826-2827 — a grog should take NO Story Flaws and not more than one
    // Personality Flaw. Both clauses are generated from `flaw_category_caps`.
    expect(guidance).toContain('Story Flaws');
    expect(guidance).toContain('Personality Flaw');
    expect(guidance).toContain('should not');
    // A grog is no magus, so the Hermetic clause is absent…
    expect(guidance).not.toContain('Hermetic');
    // …and no clause anywhere claims a Story Flaw MINIMUM. The rules give Story Flaws
    // a recommended ceiling and no floor; the only "at least one" is the magus's
    // Hermetic Flaw.
    expect(guidance).not.toMatch(/at least one Story/i);
  });

  it("states the magus's own caps and its Hermetic-Flaw recommendation (#7)", async () => {
    await startWizard('magus');
    await advanceWizardTo('virtues_flaws');
    const guidance = clean(await $(GUIDANCE).getText());

    expect(guidance).not.toContain('wizard-guidance');
    // :2861-2862 — one Story Flaw and two Personality Flaws, different numbers off
    // the same code path the grog's sentence uses.
    expect(guidance).toMatch(/more than 1 Story Flaw/);
    expect(guidance).toMatch(/more than 2 Personality Flaws/);
    // :2860, worded exactly as the `missing_hermetic_flaw` warning.
    expect(guidance).toContain('at least one Hermetic Flaw');
    expect(guidance).not.toMatch(/at least one Story/i);
  });

  it('collapses the Hermetic minimums to one line that expands on demand (#12)', async () => {
    await standOnWizardStep('virtues_flaws');
    await advanceWizardTo('abilities');
    await $(MINIMUMS).waitForExist({ timeout: STEP_TIMEOUT });

    // Collapsed: the summary is on screen, the rows are not DISPLAYED. `<details>`
    // keeps them in the DOM, so displayed-ness is the property that matters.
    const parma = await $('[data-testid="magus-minimum-ability.parma_magica"]');
    expect(await $(SUMMARY).isDisplayed()).toBe(true);
    expect(await parma.isDisplayed()).toBe(false);

    // The count heads the whole checklist, so it equals the number of rows inside.
    const summaryText = clean(await $(SUMMARY).getText());
    const numbers = summaryText.match(/\d+/g);
    expect(numbers).toHaveLength(2);
    const rowCount = await browser.execute(
      () =>
        document.querySelectorAll(
          '[data-testid^="magus-minimum-ability."], [data-testid^="magus-recommended-ability."]',
        ).length,
    );
    expect(Number(numbers[1])).toBe(rowCount);
    // A fresh magus meets none of them, so every row is outstanding — which is also
    // what makes the "7 of 7 above a list of 3" bug impossible to reproduce silently.
    expect(Number(numbers[0])).toBe(rowCount);

    // Expanding reveals both groups; collapsing hides them again.
    await $(SUMMARY).click();
    await parma.waitForDisplayed({ timeout: STEP_TIMEOUT });
    expect(await $('[data-testid="magus-recommended-ability.parma_magica"]').isDisplayed()).toBe(
      true,
    );
    await $(SUMMARY).click();
    await parma.waitForDisplayed({ timeout: STEP_TIMEOUT, reverse: true });
  });

  it('keeps the Abilities lists usable at 800px, which is why the floors stay', async () => {
    // The Slice 11 re-measure: `.region-row`'s floor is STILL load-bearing on this
    // very step — removing it took the row from 180px to 54px at 800px and to 0 at
    // 600px, collapsing the Selected list with it. Structural, never a pixel count:
    // both lists must have room for rows.
    await setWindowHeight(800);
    const metrics = await browser.execute(() => {
      const height = (selector) => document.querySelector(selector)?.clientHeight ?? null;
      return {
        regionRow: height('.region-row'),
        listScroll: height('.region-source .list-scroll'),
        selectedScroll: height('.selected-scroll'),
      };
    });
    expect(metrics.regionRow).toBeGreaterThanOrEqual(120);
    expect(metrics.listScroll).toBeGreaterThanOrEqual(60);
    expect(metrics.selectedScroll).toBeGreaterThanOrEqual(60);
  });

  it('warns about unspent experience and spell levels without blocking Finish (#30)', async () => {
    // 30 experience points buys the three minimums at 5 each, so 15 are left over.
    await satisfyMagusMinimums();
    const xpWarning = await waitForFinding('general_xp_unspent');
    expect(xpWarning).not.toContain('issue-general_xp_unspent');
    expect(xpWarning).toContain('15');
    // Factual: a count, and no claim that the points are lost.
    expect(xpWarning.toLowerCase()).not.toContain('wast');
    expect(
      await $('[data-testid="issue-list"] li[data-code="general_xp_unspent"]').getAttribute(
        'data-severity',
      ),
    ).toBe('warning');
    // The Spells step's own budget is not reported here — each warning belongs to the
    // step that holds its budget.
    expect(await shownCodes()).not.toContain('spell_levels_unspent');

    // The whole 120-level grant is untouched on the Spells step.
    await advanceWizardTo('spells');
    const spellWarning = await waitForFinding('spell_levels_unspent');
    expect(spellWarning).not.toContain('issue-spell_levels_unspent');
    expect(spellWarning).toContain('120');
    expect(spellWarning.toLowerCase()).not.toContain('wast');

    // Review renders unfiltered, so both appear there — and neither gates Finish,
    // because a warning is advice and `canFinish` counts errors alone.
    await advanceWizardTo('review');
    await waitForFinding('general_xp_unspent');
    await waitForFinding('spell_levels_unspent');
    // The DISTINCT severities of every row for either code, wherever it is rendered.
    // Distinct, not a positional list: the Review step carries a validation panel of
    // its own AND the docked one below it, both unfiltered on the terminal step, so
    // each finding legitimately appears twice. What must hold is that no rendering of
    // either is an error — count them and the assertion breaks on a layout change
    // rather than on a severity change.
    const severities = await browser.execute(() => [
      ...new Set(
        [
          ...document.querySelectorAll(
            '[data-code="general_xp_unspent"], [data-code="spell_levels_unspent"]',
          ),
        ].map((row) => row.dataset.severity),
      ),
    ]);
    expect(severities).toEqual(['warning']);
    expect(await $(FINISH).isEnabled()).toBe(true);
  });
});
