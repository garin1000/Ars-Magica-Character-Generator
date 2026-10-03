// End-to-end: the guided AGING step and its CRISIS branch, both driven on a GROG
// (the shortest rail the ruleset declares), the saga-year setting the aging
// arithmetic is measured against, and the `RunEvent::ExitRequested` bridge for a
// DIRTY document.
//
// A2 merged four previously separate spec files into this one. Each describe
// keeps its own local selectors and helpers scoped to its block, since several
// of them reused the same constant names (`SCHEDULE`, `DIE_INPUT`,
// `AGING_TOTAL`, `APPLY`, `REVERT_FIRST_YEAR`, `LONGEVITY_ADD`,
// `LONGEVITY_BONUS`, `LOG_EMPTY`, `AGE_INPUT`, `DOCKED_ISSUES`, `TAB_BAR`) with
// the same values but no reason to share a single module-level binding.
//
// `the saga year` is internally order-dependent (`it`#4 reads what `it`#2/#3
// set; `it`#5 reads what `it`#4 saved), writes persisted settings, and calls
// `window.location.reload()` — it stays an intact describe, placed after `the
// guided aging step`/`the aging crisis` (which do not touch persisted settings),
// with its own `after` hook restoring the default saga year kept describe-scoped.
//
// `RunEvent::ExitRequested bridge — unsaved changes` MUST stay last: with
// unsaved edits, `guard_blocks_quit` raises a native GTK confirmation dialog
// that WebDriver cannot dismiss, and that dialog is left open for the rest of
// this worker's app instance.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import {
  advanceWizardTo,
  BOOT_TIMEOUT,
  clean,
  currentWizardPhase,
  runDocumentAction,
  setWizardAge,
  SETTLE_TIMEOUT,
  startCharacter,
  startWizard,
  STEP_TIMEOUT,
  textOf,
  useFlatPoolFunding,
} from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

// End-to-end: the guided AGING step (slice 6b6), driven on a GROG — the suite's
// first grog wizard spec, and deliberately so: the grog rail is the shortest the
// ruleset declares (concept, type, characteristics, virtues & flaws, abilities,
// aging), so nothing between the start screen and the step under test can colour
// the result. It is also the type that proves the step stands on its own for a
// non-magus: it is where a grog first reaches the Longevity Ritual. Since Slice 3
// the ritual is part of `AgingPanel` itself, so the editor's Aging tab carries it
// for every type, and the closing `it` follows the ritual across the save into it.
//
// The arithmetic is the rulebook's:
//
//   "Characters begin aging in the Winter after they turn 35. Every year, a
//    character must roll on the aging table." (`ArMDE:16565`) — so a grog of 40 owes
//    five rolls, for ages 36 through 40.
//   "AGING TOTAL: Stress die (no botch) + age/10 (round up) / - Living Conditions
//    modifier / - Longevity Ritual modifier" (`ArMDE:16567-16569`).
//   "Modifiers marked with an asterisk are cumulative with each other" (`ArMDE:16594`)
//    — a leper colony (-1) and a poor or unhealthy location (-2) stack to -3.
//
// For this grog, rolling for age 36 with a +1 ritual:
//
//   age modifier   ceil(36 / 10)      = +4
//   conditions     -(-3)              = +3   (subtracted, so a bad life costs)
//   ritual         -(+1)              = -1
//   standing total                    = +6
//   stress die 8   8 + 6              = 14   → "1 Aging Point in Quickness" (`ArMDE:16603`)
describe('the guided aging step', () => {
  const AGE_INPUT = '[data-testid="age-input"]';
  const SCHEDULE = '[data-testid="aging-schedule"]';
  const FIRST_ROLL_AGE = '[data-testid="aging-first-roll-age"]';
  const ROLLS_NONE = '[data-testid="aging-rolls-none"]';
  const ROLLS_OWED = '[data-testid="aging-rolls-owed"]';
  const ROLLS_RECORDED = '[data-testid="aging-rolls-recorded"]';
  const TOTAL_FORMULA = '[data-testid="aging-total-formula"]';
  const CONDITIONS = '[data-testid="living-conditions"]';
  const CONDITIONS_TOTAL = '[data-testid="living-conditions-total"]';
  // The two asterisked rows this grog lives under (`ArMDE:16588`, `ArMDE:16591`).
  const LEPER_COLONY = 'living_condition.live_in_a_leper_colony';
  const POOR_LOCATION = 'living_condition.poor_or_unhealthy_location_typical_town';
  const condition = (id) => `[data-testid="living-condition-${id}"]`;
  const LONGEVITY_ADD = '[data-testid="longevity-add"]';
  const LONGEVITY_BONUS = '[data-testid="longevity-bonus"]';
  const LONGEVITY_HINT = '[data-testid="longevity-hint"]';
  const DIE_INPUT = '[data-testid="aging-die-input"]';
  const AGING_TOTAL = '[data-testid="aging-total"]';
  const TOTAL_PARTS = '[data-testid="aging-total-parts"]';
  const OUTCOME = '[data-testid="aging-outcome"]';
  const APPLY = '[data-testid="aging-apply"]';
  // manual-testing-findings-2026-09-03 #19: the calculator's list of per-year "Take
  // back age N" buttons is gone — one button per recorded year is fine at three and
  // unusable at forty. The undo is the log row's own ×, which on an engine-recorded
  // row hands the year to the very same `aging::revert_year`. This grog has exactly
  // one recorded year (age 36), so row 0 IS that year.
  const REVERT_FIRST_YEAR = '[data-testid="aging-log-remove-0"]';
  const LOG_EMPTY = '[data-testid="aging-log-empty"]';
  const LOG_EFFECT_0 = '[data-testid="aging-log-effect-0"]';
  const NEXT = '[data-testid="wizard-next"]';
  const FINISH = '[data-testid="wizard-finish"]';
  const TAB_BAR = '[role="tablist"]';
  // The editor's own aging tab, mirroring the wizard's `aging` phase (#28).
  const AGING_TAB = '[data-testid="tab-aging"]';
  // The docked step panel, scoped: other surfaces render `data-code` nodes too.
  const DOCKED_ISSUES = '[data-testid="issue-list"]';

  /** How many findings of `code` the docked panel is showing. */
  async function issueCount(code) {
    const found = await $$(`${DOCKED_ISSUES} [data-code="${code}"]`);
    return found.length;
  }

  /** Press the log's Add once and wait for row `index` before returning. */
  async function addAgingLogRow(index) {
    await $('[data-testid="aging-log-add"]').click();
    await $(`[data-testid="aging-log-year-${index}"]`).waitForExist({
      timeout: SETTLE_TIMEOUT,
      timeoutMsg: `the log Add click for row ${index} never rendered it`,
    });
  }

  /** One docked finding's sentence and the `data-severity` mirroring its rank. */
  async function issue(code) {
    const row = await $(`${DOCKED_ISSUES} [data-code="${code}"]`);
    return { text: clean(await row.getText()), severity: await row.getAttribute('data-severity') };
  }

  /** The accrued aging points, per Characteristic, as the record panel shows them. */
  async function agingPoints() {
    const entries = {};
    for (const characteristic of ['int', 'per', 'str', 'sta', 'pre', 'com', 'dex', 'qik']) {
      entries[characteristic] = await $(
        `[data-testid="aging-points-${characteristic}"]`,
      ).getValue();
    }
    return entries;
  }

  /**
   * Put every scrollport on the surface back to the top, so a "clears the fold"
   * measurement is the unscrolled one. Earlier tests in this file click controls, and
   * a click can scroll its ancestor to bring the target into view.
   */
  function scrollSurfaceToTop() {
    return browser.execute(() => {
      for (const box of document.querySelectorAll('.tab-content, .vf-tab, .tab-scroll')) {
        box.scrollTop = 0;
      }
      window.scrollTo(0, 0);
    });
  }

  /**
   * Geometry of one `.character-details` surface: how it is laid out, where each of
   * its blocks sits, and whether any of them overflows the cell it was given.
   *
   * The blocks are not the section's DOM children. `AgingPanel` and `AgingRecordPanel`
   * are `display: contents` (app.css), so this walks through any `contents` box rather
   * than reading `.children` once. Since #33 what that walk finds on the aging surface
   * is the heading and the three `.aging-column` wrappers — the grid's actual items.
   */
  function detailsMetrics(selector) {
    return browser.execute((sel) => {
      const panel = document.querySelector(sel);
      if (!panel) return null;
      const blocks = [];
      const collect = (element) => {
        for (const child of element.children) {
          if (getComputedStyle(child).display === 'contents') collect(child);
          else blocks.push(child);
        }
      };
      collect(panel);
      const style = getComputedStyle(panel);
      // Positions are measured RELATIVE TO THE PANEL, not the viewport: clicking a
      // control scrolls the tab area, which moves every viewport rect by the same
      // amount and would read as "everything shifted" when nothing was relaid out.
      const origin = panel.getBoundingClientRect();
      const log = panel.querySelector('[data-testid="aging-log-list"]');
      const effect = panel.querySelector('[data-testid="aging-log-effect-0"]');
      const logBlock = panel.querySelector('[data-testid="aging-log-block"]');
      return {
        display: style.display,
        // Resolved track sizes, so both the count and each width are real px.
        tracks: style.gridTemplateColumns.split(' ').map((track) => parseFloat(track)),
        // The grid's own content box: `clientWidth` includes the `.panel` padding the
        // tracks are laid out inside, so a full-width row is narrower than it.
        contentWidth:
          panel.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight),
        blocks: blocks.map((block) => ({
          id: block.dataset.testid ?? block.className,
          // Relative to `origin`, per the note above: these are compared across an
          // interaction that can scroll the tab area, and an absolute `top` would then
          // shift for every block at once and read as a whole-panel reflow when
          // nothing moved relative to the panel — the very false positive this
          // measurement exists to rule out.
          top: Math.round(block.getBoundingClientRect().top - origin.top),
          left: Math.round(block.getBoundingClientRect().left - origin.left),
          width: Math.round(block.getBoundingClientRect().width),
          // HEIGHT is what the column independence claim is made of (#33): a column
          // that grows must leave its neighbours' heights untouched, which is only
          // observable if the height is measured.
          height: Math.round(block.getBoundingClientRect().height),
          // Wider content than cell = something is cut off with no way to reach it.
          overflowX: block.scrollWidth - block.clientWidth,
        })),
        // Whether the log is on screen without scrolling — the #24/#25 guarantee, and
        // the reason the record column leads with it. Viewport-relative on purpose:
        // "above the fold" is a claim about the window, not about the panel.
        viewportHeight: window.innerHeight,
        logBlockTop: logBlock ? Math.round(logBlock.getBoundingClientRect().top) : null,
        logBlockBottom: logBlock ? Math.round(logBlock.getBoundingClientRect().bottom) : null,
        // The width the effect input would need for its own placeholder to render
        // whole: the string's advance in the input's OWN resolved font, plus the
        // input's horizontal padding and border. Measured rather than assumed, so the
        // floor holds in German (the longer string) and under whatever `system-ui`
        // resolves to on the box running this.
        effectPlaceholderPx: (() => {
          if (!effect) return null;
          const style = getComputedStyle(effect);
          const probe = document.createElement('span');
          probe.style.position = 'absolute';
          probe.style.visibility = 'hidden';
          probe.style.whiteSpace = 'pre';
          probe.style.fontFamily = style.fontFamily;
          probe.style.fontSize = style.fontSize;
          probe.style.fontWeight = style.fontWeight;
          probe.style.fontStyle = style.fontStyle;
          probe.style.letterSpacing = style.letterSpacing;
          probe.textContent = effect.placeholder;
          document.body.appendChild(probe);
          const advance = probe.getBoundingClientRect().width;
          probe.remove();
          return Math.ceil(
            advance +
              parseFloat(style.paddingLeft) +
              parseFloat(style.paddingRight) +
              parseFloat(style.borderLeftWidth) +
              parseFloat(style.borderRightWidth),
          );
        })(),
        logWidth: logBlock ? Math.round(logBlock.getBoundingClientRect().width) : null,
        logClientHeight: log ? log.clientHeight : null,
        logScrollHeight: log ? log.scrollHeight : null,
        logOverflowY: log ? getComputedStyle(log).overflowY : null,
        // WebKitGTK's horizontal overlay scrollbar claims hit area without taking
        // layout height, so nothing here may overflow sideways. Measured, not read off
        // the computed style: CSS turns a `visible` overflow-x into `auto` whenever the
        // other axis is not visible, so `overflow-y: auto` alone reports `auto` on both
        // — the guarantee has to be that there is nothing to scroll.
        logOverflowX: log ? log.scrollWidth - log.clientWidth : null,
        effectWidth: effect ? Math.round(effect.getBoundingClientRect().width) : null,
      };
    }, selector);
  }

  /** Type a stress die and wait for the engine's answer to come back. */
  async function rollDie(die, expectedTotal) {
    await $(DIE_INPUT).setValue(String(die));
    await browser.waitUntil(
      async () =>
        (await $(AGING_TOTAL).isExisting()) &&
        (await textOf(AGING_TOTAL)).includes(String(expectedTotal)),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: `a stress die of ${die} should total ${expectedTotal}`,
      },
    );
  }

  it('reaches an aging step that owes a young character nothing', async () => {
    await startWizard('grog');
    // B2/D41 (ArMDE:2816): every character must take one Social Status before
    // Next unblocks past `virtues_flaws` — free, so it does not touch this
    // spec's aging figures. This grog is reused by every later `it()` in this
    // describe (via `standOnWizardStep`), so this is the one place it needs it.
    await advanceWizardTo('virtues_flaws');
    const addStatus = await $('[data-testid="add-virtue.craftsman"]');
    await addStatus.waitForExist({ timeout: STEP_TIMEOUT });
    await addStatus.click();

    // Aging, not life-stage funding, is what this file is about — see
    // `useFlatPoolFunding`'s doc comment for why the guided default otherwise
    // blocks `Next` on `experience` before this test ever reaches `aging`.
    await useFlatPoolFunding();
    await advanceWizardTo('aging');
    expect(await currentWizardPhase()).toBe('aging');
    await $(SCHEDULE).waitForExist({ timeout: BOOT_TIMEOUT });

    // Nothing is owed before 36, and the step says so rather than showing an empty
    // list — a character who owes nothing is a legal state, not a blank one.
    expect(await $(ROLLS_NONE).isExisting()).toBe(true);
    expect(await $(ROLLS_OWED).isExisting()).toBe(false);

    // The threshold is read out of `rules/core/aging.json` — 35, and the first roll
    // at 36 — never printed as a literal and never as an engine field name.
    const firstRoll = await textOf(FIRST_ROLL_AGE);
    expect(firstRoll).toContain('35');
    expect(firstRoll).toContain('36');
    expect(firstRoll).not.toContain('first_roll_age');
    expect(firstRoll).not.toContain('begins_after_age');
    expect(await textOf(SCHEDULE)).not.toContain('start_age');

    // Owing nothing is not a reason to hold the player up.
    expect(await $(NEXT).isEnabled()).toBe(true);
  });

  it('owes one roll a year from 36 once an age is entered', async () => {
    // Typed on the CONCEPT step, three phases back, and read here. Slice 12 (#24)
    // gave the age one canonical home beside the birth year it is linked to, so this
    // step no longer offers a field of its own — which is exactly what makes the
    // assertion below worth making: the schedule this step is entirely about hangs on
    // a number entered somewhere else, and nothing but a round trip proves it arrives.
    expect(await $(AGE_INPUT).isExisting()).toBe(false);
    await setWizardAge(40);
    expect(await currentWizardPhase()).toBe('aging');

    await browser.waitUntil(async () => await $(ROLLS_OWED).isExisting(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a grog of 40 owes aging rolls',
    });
    const owed = await textOf(ROLLS_OWED);
    expect(owed).toContain('5');
    expect(owed).toContain('36');
    expect(owed).toContain('40');
    expect(await textOf(ROLLS_RECORDED)).toBe('0 of 5 recorded');
    expect(await $(ROLLS_NONE).isExisting()).toBe(false);

    // THE PHASE-ATTRIBUTION PROOF. "a character over the age of 35 must make aging
    // rolls … before the game begins" (`ArMDE:2232`) — filed on the AGING phase, which is
    // where the age is typed and the rolls are made. Counted EXACTLY: outside the
    // Review step only the docked panel is mounted, and it is filtered to the
    // current phase, so one finding here means this phase owns it.
    await browser.waitUntil(async () => (await issueCount('aging_rolls_pending')) === 1, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the pending aging rolls belong to the aging step',
    });
    const pending = await issue('aging_rolls_pending');
    expect(pending.severity).toBe('warning');
    expect(pending.text).not.toContain('aging_rolls_pending');
    expect(pending.text).toContain('40');
    // Advice, not admission: unrolled years must not trap the player on the step.
    expect(await $(NEXT).isEnabled()).toBe(true);
  });

  it('subtracts the Living Conditions modifier and says which rows stack', async () => {
    await $(CONDITIONS).waitForExist({ timeout: STEP_TIMEOUT });
    await $(condition(LEPER_COLONY)).click();
    await $(condition(POOR_LOCATION)).click();

    // -1 and -2, cumulative with each other (`ArMDE:16594`) — the ENGINE's resolved
    // modifier, not a sum the checklist made.
    await browser.waitUntil(async () => (await textOf(CONDITIONS_TOTAL)).includes('-3'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a leper colony and a poor location stack to -3',
    });
    // ASCII hyphen-minus, never the mathematical minus.
    expect(await textOf(CONDITIONS_TOTAL)).not.toContain('−');

    // Which rows stack is marked on the row itself; the marking is data (`cumulative`
    // in `rules/core/aging.json`), so the test reads it back off the very rows it
    // ticked. manual-testing-findings #21 removed the paragraph that also spelled the
    // rule out — each such row still carries its own `.sr-only` "cumulative" label, so
    // the marking is not asterisk-only for assistive tech.
    expect(await $(condition(LEPER_COLONY)).getAttribute('data-cumulative')).toBe('true');
    expect(await $(condition(POOR_LOCATION)).getAttribute('data-cumulative')).toBe('true');
    expect(await $('[data-testid="living-conditions-cumulative-note"]').isExisting()).toBe(false);
    // `.sr-only` text is clipped, so it is read out of the DOM rather than through
    // `getText()`ArMDE: every asterisked row still names the marking in words.
    const cumulativeLabels = await browser.execute(() =>
      [...document.querySelectorAll('[data-cumulative="true"]')].map(
        (input) => input.closest('li')?.querySelector('.sr-only')?.textContent?.trim() ?? '',
      ),
    );
    expect(cumulativeLabels.length).toBeGreaterThan(0);
    for (const label of cumulativeLabels) expect(label.length).toBeGreaterThan(0);

    // Every row is named through `rules/i18n/<lang>/aging.json`, never as its id.
    const checklist = await textOf(CONDITIONS);
    expect(checklist).toContain('Live in a leper colony');
    expect(checklist).not.toContain('living_condition.');

    // The standing half of the AGING TOTAL follows at once: +4 for the age, and the
    // conditions modifier SUBTRACTED, so -3 costs the character +3.
    await browser.waitUntil(async () => (await textOf(TOTAL_FORMULA)).includes('+7'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'age +4 and a subtracted -3 make a standing total of +7',
    });

    // AND THE SENTENCE ADDS UP. It used to name exactly the book's three terms while
    // stating a total that also included the Virtue/Flaw aging-roll modifiers, so a
    // character holding one read a contradiction (review #22). Every figure in the
    // sentence is checked as arithmetic, not merely as present: the named terms, then
    // the total they make.
    const formula = await textOf(TOTAL_FORMULA);
    expect(formula).toContain('Virtues and Flaws');
    const figures = [...formula.matchAll(/[+-]?\d+/g)].map((match) => Number(match[0]));
    expect(figures).toHaveLength(5);
    const stated = figures.pop();
    expect(figures.reduce((sum, term) => sum + term, 0)).toBe(stated);
    expect(stated).toBe(7);
  });

  // guided-creation-review-2026-08 #20, extended by manual-testing-findings-2026-09-03
  // #22/#24 and again by #33. Three arrangements, two of them reverted:
  //
  //  * CSS multi-column FLOWS content between columns, so every height change moved
  //    the break and blocks migrated to another column — ticking one checkbox relaid
  //    the whole panel and `AgingRecordPanel` sat permanently split across the break.
  //  * Grid auto-placement stopped the migration but still PAIRED blocks in a row, and
  //    a row is as tall as its tallest item, so the short schedule beside the tall roll
  //    calculator left a screen-third of emptiness. #22 answered that by spanning every
  //    block full width, which removed the columns rather than the gap.
  //  * #33 groups the blocks into three `.aging-column` WRAPPERS. Only the wrappers are
  //    grid items, so no two aging blocks share a row and the pairing cannot recur;
  //    nothing flows between wrappers, so the migration cannot either.
  //
  // What this test therefore proves is COLUMN INDEPENDENCE: growing one column changes
  // nothing about the other two, in either axis. Geometry is the subject, so only a
  // real layout engine can check it.
  it('keeps the columns independent when the aging log grows, and clips nothing', async () => {
    await scrollSurfaceToTop();
    const before = await detailsMetrics('[data-testid="aging-step"]');
    expect(before).not.toBe(null);

    // 1. THREE TRACKS at the default window — which is what the window was widened to
    //    1400x900 for (crates/arm-app/tauri.conf.json). At 1100px the 360px content
    //    floor admitted only two, so the third wrapper wrapped onto a second row below
    //    the taller of the other two, which is the below-the-fold problem #24/#25 was
    //    about. None of the tracks may be under the measured 360px floor.
    expect(before.display).toBe('grid');
    expect(before.tracks).toHaveLength(3);
    for (const track of before.tracks) expect(track).toBeGreaterThanOrEqual(360);

    // 2. The three wrappers are the grid's items, side by side in source order, each
    //    filling its own track. (`left` is measured from the panel's border box, so
    //    the first column's value is its padding, not 0.) The heading spans them all,
    //    so it is excluded — it is the one item that is not a column.
    const columnsOf = (metrics) => metrics.blocks.filter((b) => b.id.startsWith('aging-column-'));
    const columns = columnsOf(before);
    expect(columns.map((c) => c.id)).toEqual([
      'aging-column-schedule',
      'aging-column-roll',
      'aging-column-record',
    ]);
    // Strictly left to right: DOM order IS visual order, so reading, tab and focus
    // order cannot disagree with what the eye does.
    for (let i = 1; i < columns.length; i += 1) {
      expect(columns[i].left).toBeGreaterThan(columns[i - 1].left);
    }
    for (const [i, column] of columns.entries()) {
      expect(Math.abs(column.width - before.tracks[i])).toBeLessThanOrEqual(1);
    }

    // 3. Nothing is cut off: no block's content is wider than the cell it was given.
    //    A too-tight `minmax` floor shows up here, which is what bounds the judgement
    //    call behind it — the review's own complaint was a column too narrow to show
    //    a field.
    for (const block of before.blocks) {
      expect(block.overflowX).toBeLessThanOrEqual(1);
    }

    // 4. The log fills its own column and keeps its bounded scrollport, scrolling
    //    vertically only — and it is ON SCREEN WITHOUT SCROLLING, which is what
    //    leading the record column with it buys.
    const recordColumn = columns[2];
    expect(before.logWidth).toBeGreaterThanOrEqual(recordColumn.width - 2);
    expect(before.logOverflowY).toBe('auto');
    expect(before.logOverflowX).toBeLessThanOrEqual(1);
    expect(before.logBlockTop).toBeGreaterThanOrEqual(0);
    expect(before.logBlockBottom).toBeLessThanOrEqual(before.viewportHeight);

    // 5. THE LOAD-BEARING ASSERTION. Ten rows is a real height change of the kind that
    //    used to move blocks between columns — and past what the scrollport shows, so
    //    the log is now at its ceiling.
    //    One click, then its row, before the next: back-to-back clicks can lose one
    //    to a panel growing under the pointer (9632426, `waitForBalancePoints` in
    //    helpers.js).
    for (let i = 0; i < 10; i += 1) await addAgingLogRow(i);
    await $('[data-testid="aging-log-year-9"]').waitForExist({ timeout: STEP_TIMEOUT });
    await scrollSurfaceToTop();
    const grown = await detailsMetrics('[data-testid="aging-step"]');

    // The two columns that did not change are UNTOUCHED — position, width AND height.
    // Height is the new half of the claim and the whole point of #33: under the old
    // auto-placement a taller neighbour dictated a short block's row height, and under
    // #22's full-width rows a block above pushed everything below it down. A wrapper
    // owns its height, so the record column growing is invisible to the other two.
    const unchanged = (metrics) => metrics.blocks.filter((b) => b.id !== 'aging-column-record');
    expect(unchanged(grown)).toEqual(unchanged(before));
    // The record column itself keeps its place and its width; only its height is its
    // own business.
    const grownRecord = columnsOf(grown)[2];
    expect([grownRecord.id, grownRecord.left, grownRecord.width]).toEqual([
      recordColumn.id,
      recordColumn.left,
      recordColumn.width,
    ]);

    // 6. The log scrolls in place rather than pushing anything further: it is at its
    //    bounded height, so six more rows move NOTHING at all — the record column's
    //    own height included.
    expect(grown.logScrollHeight).toBeGreaterThan(grown.logClientHeight);
    // Vertically, and only vertically: the rows wrap rather than scroll sideways,
    // even once the vertical scrollbar has taken its width out of the row.
    expect(grown.logOverflowX).toBeLessThanOrEqual(1);
    for (let i = 10; i < 16; i += 1) await addAgingLogRow(i);
    await $('[data-testid="aging-log-year-15"]').waitForExist({ timeout: STEP_TIMEOUT });
    await scrollSurfaceToTop();
    const after = await detailsMetrics('[data-testid="aging-step"]');
    expect(after.logClientHeight).toBe(grown.logClientHeight);
    expect(after.blocks).toEqual(grown.blocks);
    // Still above the fold with sixteen rows in it.
    expect(after.logBlockBottom).toBeLessThanOrEqual(after.viewportHeight);

    // 7. THE EFFECT FIELD'S WIDTH FLOOR, re-derived for #33. The old threshold was a
    //    bare `> 300`, which only ever held because the log had the whole panel width;
    //    inside a 431px column the input is `year, effect, ×` on one wrapping flex
    //    line, and `min-width: 0` let it shrink without ever triggering the wrap, so
    //    the placeholder went back to "Describe the aging roll's e…".
    //    The floor is now the CONTENT's: the placeholder the field is currently
    //    showing, measured in the input's own resolved font, plus the input's own
    //    padding and border. That is the actual guarantee — "the label in the box fits
    //    in the box" — and it needs no second magic number to hold in whichever
    //    language is running. The CSS floor that delivers it is
    //    `min-width: min(22rem, 100%)` on `.aging-log-block .twilight-desc`ArMDE: measured
    //    here, the two shipped placeholders advance 178.375px (English) and 257.75px
    //    (German), the input's padding and border add 14.75px, and 22rem = 280.5px
    //    clears the German 272.5px. English measures 194px against a 326px field, so
    //    the assertion has real headroom in the locale it runs in and the binding case
    //    is pinned by the CSS comment beside the rule.
    expect(after.effectPlaceholderPx).toBeGreaterThan(0);
    expect(after.effectWidth).toBeGreaterThanOrEqual(after.effectPlaceholderPx);

    // Put the log back as it was: every added row is blank and identical, so
    // removing the first one sixteen times empties it again. Each removal waits for
    // the last row index to go before the next click, for the same reason as the adds.
    for (let i = 0; i < 16; i += 1) {
      await $('[data-testid="aging-log-remove-0"]').click();
      await $(`[data-testid="aging-log-year-${15 - i}"]`).waitForExist({
        reverse: true,
        timeout: SETTLE_TIMEOUT,
        timeoutMsg: `Remove click ${i + 1} of 16 never removed a log row`,
      });
    }
    await $(LOG_EMPTY).waitForExist({ timeout: STEP_TIMEOUT });
  });

  it('takes a Longevity Ritual bonus a grog did not make himself', async () => {
    // THE GAP THIS STEP CLOSES. "You can perform Longevity Rituals for others, even
    // for non-magi" (`ArMDE:10672`), but the editor's only home for the ritual is the
    // Possessions tab, which is magus-gated — so for a grog this guided step is the
    // one place the bonus can be entered at all.
    await $(LONGEVITY_ADD).click();
    await $(LONGEVITY_BONUS).waitForExist({ timeout: STEP_TIMEOUT });
    // Scroll it in, then wait for CLICKABLE, not merely existing. The guided aging
    // step nests three scrollports (`.tab-content` > `.vf-tab` > `.tab-scroll`) and
    // the ritual is the LAST block of the record column, so it is the one thing on
    // this surface still below the fold. #33 narrowed the margin but did not close it:
    // the record column starts 326px down and is 612px tall with an empty log, so its
    // foot lands at ~938px in a 900px window (before #33 the innermost scrollport
    // showed 218px of 1340px of content at 800px, so this is a large improvement and
    // still not enough). Its centre falls outside the visible box, the driver's hit-test lands
    // on an ancestor — `MAIN.tab-content.wizard-body` — and `setValue` reports the
    // element as never becoming interactable. A human scrolls to it, so the spec does
    // too. This stays the fix: Slice 11 measured removing the inner scrollport and it
    // changes nothing here (the step is 267px tall either way, and the same
    // scroll-then-wait is needed) — see `.tab-scroll`'s comment in `app.css`.
    // `waitForExist` alone was never enough — it means "in the DOM", not "usable" —
    // and the same two lines in the aging-crisis describe below failed the same way
    // (fixed in Slice 8; this call site was missed).
    await $(LONGEVITY_BONUS).scrollIntoView({ block: 'center' });
    await $(LONGEVITY_BONUS).waitForClickable({ timeout: STEP_TIMEOUT });
    await $(LONGEVITY_BONUS).setValue('1');

    // Subtracted like the conditions (`ArMDE:16571`ArMDE: "a high Longevity Ritual modifier
    // … indicate[s] longer life"), so +7 becomes +6.
    await browser.waitUntil(async () => (await textOf(TOTAL_FORMULA)).includes('+6'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a +1 ritual should take one off the standing total',
    });

    // The panel degrades for a non-magus exactly as intended: the bonus field is
    // there, the Creo Corpus suggestion behind it is not.
    expect(await $(LONGEVITY_HINT).isExisting()).toBe(false);
    expect(await $(LONGEVITY_BONUS).isExisting()).toBe(true);
  });

  it('totals a typed die, names the outcome in words, and records nothing yet', async () => {
    // Saved first, so the document is clean going in: what follows must not dirty
    // it, because the die is UI-only state the entity never holds.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(
      async () => !(await browser.execute(() => document.title)).startsWith('*'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'saving should clear the dirty marker before the die is typed',
      },
    );

    // The calculator defaults to the first year the log does not record — 36.
    await rollDie(8, 14);

    // Every term, as the engine reported it. A stress die explodes, so the input
    // carries no maximum; the total is asked of the engine, never derived here.
    const parts = await textOf(TOTAL_PARTS);
    expect(parts).toContain('+8');
    expect(parts).toContain('+4');
    expect(parts).toContain('+3');
    expect(parts).toContain('-1');
    expect(parts).not.toContain('−');

    // 14 is "1 Aging Point in Qik" in the book's shorthand (`ArMDE:16603`) — and the
    // Characteristic reaches the player in words, through `characteristic-qik`.
    const outcome = await textOf(OUTCOME);
    expect(outcome).toContain('Quickness');
    expect(outcome).not.toContain('qik');
    // 14 is well past the apparent-age threshold of 3 (`ArMDE:16577`).
    expect(outcome).toContain('Apparent age increases by one year.');

    // THE LOAD-BEARING ASSERTION: a total is not an outcome. Nothing is written to
    // the character until Apply, so no aging point has moved, the log is empty, and
    // the document is not even dirty.
    expect(Object.values(await agingPoints()).every((points) => points === '0')).toBe(true);
    expect(await $(LOG_EMPTY).isExisting()).toBe(true);
    expect((await browser.execute(() => document.title)).startsWith('*')).toBe(false);
  });

  it('applies a year, which settles the owed-rolls warning as the log fills', async () => {
    await $(APPLY).click();

    // The award lands where the table named it, and the log gains the year.
    await browser.waitUntil(async () => (await agingPoints()).qik === '1', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'applying a total of 14 should award one Aging Point in Quickness',
    });
    expect(await $(LOG_EMPTY).isExisting()).toBe(false);
    await $(LOG_EFFECT_0).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(ROLLS_RECORDED)).toBe('1 of 5 recorded');

    // The finding is keyed on the LOG, not on the points: a roll can legitimately
    // award nothing, so a well-rolled character would otherwise be nagged forever.
    await browser.waitUntil(async () => (await issueCount('aging_rolls_pending')) === 0, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a recorded aging year should settle the pending-rolls warning',
    });
  });

  it('reverts it exactly', async () => {
    // A pre-play catch-up can run to 25 rolls; one without an undo is not
    // shippable. `revert_year` subtracts precisely what the entry recorded.
    await $(REVERT_FIRST_YEAR).click();

    await browser.waitUntil(async () => (await agingPoints()).qik === '0', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'reverting age 36 should take its Aging Point back',
    });
    expect(await $(LOG_EMPTY).isExisting()).toBe(true);
    expect(await textOf(ROLLS_RECORDED)).toBe('0 of 5 recorded');

    // And the year is owed again, so the warning comes back with it.
    await browser.waitUntil(async () => (await issueCount('aging_rolls_pending')) === 1, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'taking the year back should restore the pending-rolls warning',
    });
  });

  it('carries the choices into the editor, the save and back', async () => {
    // Roll the year again, so there is a recorded year to carry across.
    await rollDie(8, 14);
    await $(APPLY).click();
    await browser.waitUntil(async () => (await agingPoints()).qik === '1', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 're-applying age 36 should award the Aging Point again',
    });

    await advanceWizardTo('review');
    // The Review step mounts the whole-character panel AND the docked one, both
    // `issue-list`, so a finding that IS there shows twice — which is why the exact
    // count above lives on the aging step. Nothing is owed here now either way.
    expect(await issueCount('aging_rolls_pending')).toBe(0);
    await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });
    await $(FINISH).click();

    // ONE SURFACE, TWO FLOWS: the editor's Aging tab mounts the very component the
    // guided step did (`AgingPanel`), so the choices are simply there.
    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(AGING_TAB).click();
    await $(CONDITIONS).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(condition(LEPER_COLONY)).isSelected()).toBe(true);
    expect(await $(condition(POOR_LOCATION)).isSelected()).toBe(true);
    expect(await $(LOG_EFFECT_0).isExisting()).toBe(true);
    expect((await agingPoints()).qik).toBe('1');

    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });

    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    // Canonical serialization: the chosen conditions are ids, sorted.
    expect(saved.living_conditions).toEqual([LEPER_COLONY, POOR_LOCATION]);
    expect(saved.age).toBe(40);
    // Derived from the age against the saga year, and NOT a schema change: both
    // fields have always been stored (Slice 12, #25 — no `SCHEMA_VERSION` bump).
    expect(saved.birth_year).toBe(1180);
    expect(saved.schema_version).toBe(22);
    // The widened log entry is the whole record of the year: what was rolled, what
    // it totalled, the conditions in force, the points it awarded — and now the
    // calendar year. The engine has always written `year` as `birth_year + age`
    // (`aging.rs`) and always omitted it for a character with no birth year, which
    // before the age ↔ birth-year link was every character this suite built. Typing
    // 40 in a 1220 saga puts this grog's birth at 1180, so age 36 fell in 1216.
    expect(saved.aging_log).toEqual([
      {
        age: 36,
        effect: '',
        die: 8,
        total: 14,
        living_conditions: [LEPER_COLONY, POOR_LOCATION],
        points: { qik: 1 },
        apparent_age_increased: true,
        year: 1216,
      },
    ]);

    // NO DRAFT STATE. The die and the outcome live in UI-only state; outside the
    // log entry's own recorded fields, neither may appear anywhere in the save.
    const withoutLog = { ...saved };
    delete withoutLog.aging_log;
    const rest = JSON.stringify(withoutLog);
    expect(rest).not.toContain('"die"');
    expect(rest).not.toContain('"outcome"');
    expect(rest).not.toContain('"distribution"');

    // Reload: everything is read back off the stored choices, with no reconciliation.
    await runDocumentAction('open');
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(AGING_TAB).click();
    await $(CONDITIONS).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(condition(LEPER_COLONY)).isSelected()).toBe(true);
    expect(await $(condition(POOR_LOCATION)).isSelected()).toBe(true);
    expect(await textOf(CONDITIONS_TOTAL)).toContain('-3');
    expect(await textOf(ROLLS_RECORDED)).toBe('1 of 5 recorded');
    expect((await agingPoints()).qik).toBe('1');
  });

  // Slice 6b8c, re-homed in Slice 3. "You can perform Longevity Rituals for others,
  // even for non-magi" (`ArMDE:10672`), and the ritual this grog holds is a term of every
  // aging total it will ever roll — but the editor's only home for one was the
  // magus-gated Possessions tab, so once the wizard was finished the bonus could no
  // longer be corrected. The ritual now lives inside `AgingPanel`, so the Aging tab
  // carries it for every type and the old `!is_magus` special case is gone.
  it('keeps the ritual reachable on the editor Aging tab, which every type has', async () => {
    // Still on the Aging tab of the reloaded grog from the test above, and the tab
    // that used to own the ritual is not even in this character's tab bar.
    expect(await $('[data-testid="tab-possessions"]').isExisting()).toBe(false);
    await browser.waitUntil(async () => (await $(LONGEVITY_BONUS).getValue()) === '1', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the editor should show the +1 ritual the guided step entered',
    });
    // The suggestion stays magus-only, exactly as it does on the guided step.
    expect(await $(LONGEVITY_HINT).isExisting()).toBe(false);
  });

  // `.character-details` is carried by FOUR mount sites: the wizard's aging step
  // (checked above), the editor's Aging tab, the editor's Details tab, and the
  // Personality & Reputations component that is both the wizard's step and the
  // editor's tab. #20 was reported against the aging step alone, but the class is
  // shared, so Slice 6 relays out all four — which is a win everywhere, and is
  // therefore verified everywhere rather than assumed. Since #33 the AGING surface
  // fills those tracks with three column wrappers instead of its raw blocks (see the
  // geometry spec above), so it is back to using the shared columns exactly as the
  // Details and Personality tabs do — and what this checks on all three is the one
  // thing they have in common: the same auto-fit grid, no track under the floor, and
  // nothing clipped.
  it('lays out every character-details surface as the same grid, clipping nothing', async () => {
    // Collected rather than asserted per tab, so a failure names the surface and the
    // block instead of just the first number that went wrong.
    const problems = [];
    for (const tab of ['tab-aging', 'tab-details', 'tab-personality_reputations']) {
      await $(`[data-testid="${tab}"]`).click();
      await $('.character-details').waitForExist({ timeout: STEP_TIMEOUT });
      const m = await detailsMetrics('.character-details');
      if (m == null) {
        problems.push(`${tab}: no .character-details surface`);
        continue;
      }
      if (m.display !== 'grid') problems.push(`${tab}: display is ${m.display}, not grid`);
      // No breakpoint list: `auto-fit` against the 28.25rem (360px) floor decides the
      // column count, so no track HOLDING ANYTHING may come out under the floor.
      // A 0px track is not a violation but `auto-fit` working: it collapses the tracks
      // no item was placed into and shares their width among the rest. That surfaced
      // the moment #33 widened the window to 1400 — Personality & Reputations has two
      // blocks and the wider panel offers three tracks, so the third reports 0px.
      // Anything strictly between 0 and the floor is the real failure, and still fails.
      for (const track of m.tracks.filter((width) => width > 0)) {
        if (track < 360) problems.push(`${tab}: a ${track}px track is under the floor`);
      }
      // Nothing overlapping, orphaned or wider than the cell it was given.
      for (const block of m.blocks) {
        if (block.overflowX > 1) {
          problems.push(`${tab}: ${block.id} overflows its cell by ${block.overflowX}px`);
        }
      }
    }
    expect(problems).toEqual([]);
  });
});

// End-to-end: the aging CRISIS (slice 6b7c), driven on a GROG — the same rail
// the describe above uses, and for the same reason: the grog rail is the
// shortest the ruleset declares, so nothing between the start screen and the
// step under test can colour the result. What this spec adds is the second die,
// which until 6b7c the shipped app had nowhere to put: every Crisis it applied
// went in owed and unrolled, whatever the player had thrown.
//
// The arithmetic is the rulebook's, and both halves of it are hand-checkable:
//
//   "AGING TOTAL: Stress die (no botch) + age/10 (round up) / - Living Conditions
//    modifier / - Longevity Ritual modifier" (`ArMDE:16567-16569`).
//   "13 | Gain sufficient Aging Points (in any Characteristics) to reach the next
//    level in Decrepitude, and Crisis" (`ArMDE:16602`).
//   "**Crisis:** Increase the character's Decrepitude first, and then roll on the
//    Crisis Table." (`ArMDE:16619`)
//   "CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score" (`ArMDE:16621`).
//   "15 | **Minor illness**. Stamina stress roll against an Ease Factor of 3 or
//    CrCo20 to survive." (`ArMDE:16628`)
//
// For this grog of 40, rolling for age 36 with no Living Conditions and a ritual
// worth 0:
//
//   age modifier   ceil(36 / 10)      = +4
//   stress die 9   9 + 4              = 13   → next Decrepitude level, and a Crisis
//   five points placed                       → Decrepitude Score 1 (five xp buys 1)
//   simple die 10  10 + 4 + 1         = 15   → Minor illness, Ease Factor 3, CrCo20
//
// THE ORDER IS THE RULE. The five points are `ArMDE:16619`'s increase, so they are a term
// of the crisis total — which is why the panel withholds a reading until they are
// placed, and why this spec types the crisis die BEFORE placing them and asserts
// there is no total yet.
describe('the aging crisis', () => {
  const SCHEDULE = '[data-testid="aging-schedule"]';
  const DIE_INPUT = '[data-testid="aging-die-input"]';
  const AGING_TOTAL = '[data-testid="aging-total"]';
  const DISTRIBUTE_STA = '[data-testid="aging-distribute-sta"]';
  const APPLY = '[data-testid="aging-apply"]';
  // manual-testing-findings-2026-09-03 #19: the calculator's per-year "Take back age N"
  // list is gone (unusable once a magus owes forty years). The undo is the log row's
  // own ×, which on an engine-recorded row calls the same `aging::revert_year`. This
  // character has exactly one recorded year (age 36), so row 0 is that year.
  const REVERT_FIRST_YEAR = '[data-testid="aging-log-remove-0"]';
  const LONGEVITY_ADD = '[data-testid="longevity-add"]';
  const LONGEVITY_BONUS = '[data-testid="longevity-bonus"]';

  const CRISIS = '[data-testid="aging-crisis"]';
  const CRISIS_DIE = '[data-testid="crisis-die-input"]';
  const CRISIS_UNROLLED = '[data-testid="crisis-die-unrolled"]';
  const CRISIS_TOTAL = '[data-testid="crisis-total"]';
  const CRISIS_PARTS = '[data-testid="crisis-total-parts"]';
  const CRISIS_ROW = '[data-testid="crisis-row"]';
  const CRISIS_SURVIVAL = '[data-testid="crisis-survival"]';
  const CRISIS_EASE = '[data-testid="crisis-survival-ease-factor"]';
  const CRISIS_RITUAL = '[data-testid="crisis-survival-ritual"]';
  const CRISIS_ALLOWANCE = '[data-testid="crisis-allowance-0"]';
  const CRISIS_MODIFIER_0 = '[data-testid="crisis-modifier-0"]';
  const AGING_NOTE_0 = '[data-testid="aging-note-0"]';
  const LOG_CRISIS_0 = '[data-testid="aging-log-crisis-0"]';
  const LOG_EMPTY = '[data-testid="aging-log-empty"]';

  it('sends a grog of 40 to the Crisis Table and asks for the second die', async () => {
    await startWizard('grog');
    // B2/D41 (ArMDE:2816): every character must take one Social Status before
    // Next unblocks past `virtues_flaws` — free, so it does not touch this
    // spec's crisis figures. This grog is reused by every later `it()` in
    // this describe, so this is the one place it needs it.
    await advanceWizardTo('virtues_flaws');
    const addStatus = await $('[data-testid="add-virtue.craftsman"]');
    await addStatus.waitForExist({ timeout: STEP_TIMEOUT });
    await addStatus.click();

    await useFlatPoolFunding();
    await advanceWizardTo('aging');
    expect(await currentWizardPhase()).toBe('aging');
    await $(SCHEDULE).waitForExist({ timeout: BOOT_TIMEOUT });
    // Slice 12 (#24): the age lives on the `concept` step now, so it is typed there
    // and read here. `setWizardAge` walks back and returns to this step.
    await setWizardAge(40);
    await $(SCHEDULE).waitForExist({ timeout: STEP_TIMEOUT });

    // A ritual worth 0 leaves the AGING TOTAL alone and is still a ritual the Crisis
    // spends (`ArMDE:16573`). The ritual is part of `AgingPanel`, so this guided step and
    // the editor's Aging tab both reach it; the step is where the Crisis is resolved,
    // so this is where the note can be provoked.
    await $(LONGEVITY_ADD).waitForExist({ timeout: STEP_TIMEOUT });
    await $(LONGEVITY_ADD).click();
    // Clickable, not merely existing: the field appears the moment the ritual is
    // added above, so the aging grid is still relaying out around it and for a frame
    // the input is in the DOM but not yet interactable. `waitForExist` alone let
    // `setValue` race that frame — the whole spec failed on its first attempt and
    // passed on the retry, which is the shape a real defect hides in.
    await $(LONGEVITY_BONUS).waitForClickable({ timeout: STEP_TIMEOUT });
    await $(LONGEVITY_BONUS).setValue('0');

    // 9 + 4 = 13, which is the first Crisis row (`ArMDE:16602`).
    await $(DIE_INPUT).setValue('9');
    await browser.waitUntil(async () => (await textOf(AGING_TOTAL)).includes('13'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a stress die of 9 at age 36 should total 13',
    });
    // manual-testing-findings #21 removed the "this roll is a Crisis, place the points
    // first" paragraph. The Crisis panel appearing IS that statement now.
    expect(await $('[data-testid="aging-outcome-crisis"]').isExisting()).toBe(false);

    // The panel appears with the row that demanded it, never before.
    await $(CRISIS).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(CRISIS_DIE).isExisting()).toBe(true);
    // "a zero counts as ten" (`ArMDE:474`) — offered as the ruleset's own bounds.
    expect(await $(CRISIS_DIE).getAttribute('min')).toBe('1');
    expect(await $(CRISIS_DIE).getAttribute('max')).toBe('10');
  });

  it('withholds the reading until the Aging Points are placed', async () => {
    // THE LOAD-BEARING ASSERTION of the whole slice. "Increase the character's
    // Decrepitude first, and then roll on the Crisis Table" (`ArMDE:16619`) — those five
    // points ARE the increase, and they are a term of the crisis total. Read off the
    // character standing here, the same die answers 14; the app would then show 14
    // and write 15. So with the die typed and nothing placed, there is no reading.
    await $(CRISIS_DIE).setValue('10');
    await browser.waitUntil(async () => await $(CRISIS_UNROLLED).isExisting(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'an unplaced distribution should leave the Crisis unread',
    });
    expect(await $(CRISIS_TOTAL).isExisting()).toBe(false);

    // Placing them is what makes the reading honest — and it is 15, not 14.
    await $(DISTRIBUTE_STA).setValue('5');
    await browser.waitUntil(
      async () =>
        (await $(CRISIS_TOTAL).isExisting()) && (await textOf(CRISIS_TOTAL)).includes('15'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'a simple die of 10 with the year’s five points placed should total 15',
      },
    );

    // Every term, as the engine reported it: the die, the age, and the Decrepitude
    // this very year raised.
    const parts = await textOf(CRISIS_PARTS);
    expect(parts).toContain('+10');
    expect(parts).toContain('+4');
    expect(parts).toContain('+1');
    // ASCII hyphen-minus only, never the mathematical minus.
    expect(parts).not.toContain('−');
  });

  it('names the row and says what surviving it would take', async () => {
    // "15 | **Minor illness**." (`ArMDE:16628`) — the row's text comes from
    // `rules/i18n/<lang>/aging.json` keyed by its id, and the severity through
    // Fluent. Neither may reach the screen as a slug.
    const row = await textOf(CRISIS_ROW);
    expect(row).toContain('Minor illness');
    expect(row).not.toContain('crisis.');

    // "Stamina stress roll against an Ease Factor of 3 or CrCo20 to survive."
    await $(CRISIS_SURVIVAL).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(CRISIS_EASE)).toContain('3');
    expect(await textOf(CRISIS_RITUAL)).toContain('20');

    // "An Int + Medicine roll against an Ease Factor of 6 … if the doctor botches the
    // character must subtract 3" (`ArMDE:16634`) — stated, never scored, because the
    // Medicine belongs to a character this sheet does not hold.
    const allowance = await textOf(CRISIS_ALLOWANCE);
    expect(allowance).toContain('Medicine');
    expect(allowance).toContain('6');
    expect(allowance).toContain('-3');
    expect(allowance).not.toContain('ability.medicine');
    expect(allowance).not.toContain('−');

    // This grog carries no familiar and no aging Virtue, so he brings NOTHING to the
    // roll — and the panel shows no modifier line at all rather than a "+0".
    expect(await $(CRISIS_MODIFIER_0).isExisting()).toBe(false);
  });

  it('writes the Crisis onto the character, and says what it spent', async () => {
    await $(APPLY).click();

    // The five points land where the player placed them, and the log gains the year.
    await browser.waitUntil(
      async () => (await $('[data-testid="aging-points-sta"]').getValue()) === '5',
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'applying a total of 13 should award five Aging Points',
      },
    );
    expect(await $(LOG_EMPTY).isExisting()).toBe(false);

    // The log reads the Crisis back — the row named, and both figures beside it.
    const logged = await textOf(LOG_CRISIS_0);
    expect(logged).toContain('Minor illness');
    expect(logged).toContain('15');
    expect(logged).toContain('10');
    expect(logged).not.toContain('crisis.minor_illness');

    // "its power is spent, and the focal ritual must be performed again" (`ArMDE:16573`) —
    // reported, because the engine leaves the ritual entry exactly where it found it,
    // so this note is the only place the player can hear about it.
    await $(AGING_NOTE_0).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(AGING_NOTE_0)).not.toHaveLength(0);
    expect(await $(LONGEVITY_BONUS).isExisting()).toBe(true);
  });

  it('saves the resolved Crisis and takes the whole year back off', async () => {
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });

    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.aging_log).toEqual([
      {
        age: 36,
        effect: '',
        die: 9,
        total: 13,
        points: { sta: 5 },
        apparent_age_increased: true,
        crisis: true,
        crisis_die: 10,
        crisis_total: 15,
        crisis_row: 'crisis.minor_illness',
        crisis_severity: 'minor',
        // DATED, since Slice 12 (#25). The engine has always recorded a log entry's
        // calendar year as `birth_year + age` (`aging.rs`) and always left it out for
        // a character with no birth year — which, before the age ↔ birth-year link,
        // was every character this suite built. Typing 40 in a 1220 saga now derives
        // a birth year of 1180, so the roll for age 36 falls in 1216, and the log says
        // so. The entry is dated as a consequence of the link, not as a new mechanic.
        year: 1216,
      },
    ]);
    // The derived half of the pair really did reach the save, which is what makes the
    // year above meaningful rather than a magic number.
    expect(saved.birth_year).toBe(1180);
    expect(saved.age).toBe(40);
    // NO DRAFT STATE: both dice are the player's, and only the log entry's own
    // recorded fields may carry them.
    const withoutLog = { ...saved };
    delete withoutLog.aging_log;
    expect(JSON.stringify(withoutLog)).not.toContain('crisis');

    // A mistyped die has to be recoverable, and a Crisis is undone with the year that
    // caused it — never by editing a figure out from under the entry.
    await $(REVERT_FIRST_YEAR).click();
    await browser.waitUntil(
      async () => (await $('[data-testid="aging-points-sta"]').getValue()) === '0',
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'reverting age 36 should take its five Aging Points back',
      },
    );
    expect(await $(LOG_EMPTY).isExisting()).toBe(true);
    expect(await $(LOG_CRISIS_0).isExisting()).toBe(false);
  });
});

// End-to-end: the SAGA YEAR and the age ↔ birth-year link it derives
// (guided-creation-review-2026-08 #25, Slice 12; moved onto the document by C8).
//
// WHY THIS SPEC IS MANDATORY RATHER THAN NICE TO HAVE. `derive_age` /
// `derive_birth_year` are called by the store on every keystroke in the age or
// birth-year field, and since C8 the year they are measured against travels in the
// save. Unit tests cover the arithmetic and the migration; only a real-binary run
// covers the IPC bridge and the fact that the frontend's derivation actually reaches
// the entity that gets written to disk.
//
// THE FOUR CLAIMS:
//
//  1. Age and birth year are two views of one fact — edit either, the other follows,
//     against this document's saga year.
//  2. A saga year BEFORE the birth year clamps the derived age to 0 and says why,
//     rather than underflowing the entity's unsigned `age`
//     (`saga_year_before_birth_year`, a warning on the `concept` phase).
//  3. Editing the saga year rewrites NEITHER stored value. A silent recompute would
//     fabricate ages that skipped their aging rolls (D3.3).
//  4. But it DOES dirty the document and it IS written to the file — that is the
//     whole of C8. Before it, the year lived in a machine-global settings file, so a
//     storyguide running a 1220 Rhine saga and a 1197 Iberia saga had one number that
//     was wrong for one of them, and every age derived from it was wrong with it.
//
// A new character starts at the configured default, which a worker that has chosen
// none gets from the engine's own rules value —
// "That domination persists until the present day, 1220."
// (ArMDE:597) — so 1220 is asserted as the
// year a fresh installation stamps, never typed in as a magic number first. The
// settings half of the split (`default_saga_year`, and that it seeds the NEXT
// document without touching the open one) is covered in `app-shell.e2e.js`'s
// settings-dialog block, beside the other three persisted settings.
//
// This describe no longer writes a settings file at all — the year it types goes into
// the document — so the `after` hook that used to restore 1220 is gone with it.
// Internally order-dependent: `it`#4 reads what `it`#2/#3 set, and `it`#5 reopens
// what `it`#4 saved.
describe('the saga year', () => {
  const AGE_INPUT = '[data-testid="age-input"]';
  const BIRTH_YEAR_INPUT = '[data-testid="identity-birth-year"]';
  const SAGA_YEAR_INPUT = '[data-testid="saga-year-input"]';
  const SAGA_YEAR_HINT = '[data-testid="saga-year-hint"]';
  // The docked step panel, scoped: other surfaces render `data-code` nodes too.
  const DOCKED_ISSUES = '[data-testid="issue-list"]';
  const CLAMP_CODE = 'saga_year_before_birth_year';

  /** The published setting's year, and so the year a fresh installation stamps. */
  const DEFAULT_SAGA_YEAR = '1220';

  /** Type `value` into `selector`, replacing whatever was there. */
  async function type(selector, value) {
    const field = await $(selector);
    await field.waitForClickable({ timeout: STEP_TIMEOUT });
    await field.setValue(String(value));
  }

  /** Wait for a field to read back `value`, so the store has taken it. */
  async function expectValue(selector, value, what) {
    await browser.waitUntil(async () => (await $(selector).getValue()) === String(value), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `${what}: expected ${value}, got ${await $(selector).getValue()}`,
    });
  }

  /** How many findings of `code` the docked panel is showing. */
  async function issueCount(code) {
    const found = await $$(`${DOCKED_ISSUES} [data-code="${code}"]`);
    return found.length;
  }

  before(async () => {
    await startWizard('grog');
    await $(AGE_INPUT).waitForExist({ timeout: BOOT_TIMEOUT });
  });

  it('offers the age beside the identity, with the year they are measured against', async () => {
    // #24: ONE canonical home for the age, mirroring the editor's Details tab. The
    // count spans the step, because a duplicate is exactly what this replaced.
    expect(await $$(AGE_INPUT)).toHaveLength(1);
    expect(await $$(BIRTH_YEAR_INPUT)).toHaveLength(1);

    // The year a fresh character is stamped with is the engine's, not this field's:
    // a worker that has chosen no default gets the published setting's own year.
    await expectValue(SAGA_YEAR_INPUT, DEFAULT_SAGA_YEAR, 'the default saga year');

    // manual-testing-findings #21: the sentence explaining what the setting does and
    // does not do is gone, and its `aria-describedby` with it — the field's label is
    // the whole of what it says. What it DOES is asserted for real by the two tests
    // below, which type into it and read the age and birth year back.
    expect(await $(SAGA_YEAR_HINT).isExisting()).toBe(false);
    expect(await $(SAGA_YEAR_INPUT).getAttribute('aria-describedby')).toBe(null);
  });

  it('derives the age from a typed birth year, and the birth year from a typed age', async () => {
    await type(BIRTH_YEAR_INPUT, 1190);
    await expectValue(AGE_INPUT, 30, 'a birth year of 1190 in a 1220 saga is age 30');

    // And back the other way, which is the half that proves they are two views rather
    // than one field feeding the other.
    await type(AGE_INPUT, 45);
    await expectValue(BIRTH_YEAR_INPUT, 1175, 'age 45 in a 1220 saga is a birth year of 1175');
  });

  it('clamps the age to zero and says why when the saga year precedes the birth year', async () => {
    // `birth_year` is i32 and `age` is u32, so this is the one pair that could
    // underflow. It is an advisory, not an error: impossible, but not illegal.
    await type(BIRTH_YEAR_INPUT, 1250);
    await expectValue(AGE_INPUT, 0, 'a character not yet born reads as age 0');

    await browser.waitUntil(async () => (await issueCount(CLAMP_CODE)) === 1, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'an impossible age/birth-year pair must be reported on the concept step',
    });
    const row = await $(`${DOCKED_ISSUES} [data-code="${CLAMP_CODE}"]`);
    expect(await row.getAttribute('data-severity')).toBe('warning');
    const text = clean(await row.getText());
    // A sentence naming both years, never its own code.
    expect(text).not.toContain(CLAMP_CODE);
    expect(text).toContain('1220');
    expect(text).toContain('1250');

    // Fixing either half clears it.
    await type(BIRTH_YEAR_INPUT, 1190);
    await browser.waitUntil(async () => (await issueCount(CLAMP_CODE)) === 0, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a possible pair must clear the advisory',
    });
    await expectValue(AGE_INPUT, 30, 'the age follows the corrected birth year');
  });

  it('dirties the document when the saga year moves, and recomputes neither half', async () => {
    // A clean baseline first: the claim is about what THIS edit moves, which can
    // only be read off a document that had nothing outstanding.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });
    await browser.waitUntil(
      async () => !(await browser.execute(() => document.title)).startsWith('*'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'the document should be clean immediately after a save',
      },
    );
    // The save wrote the year too — it is stored state, not a preference (C8).
    expect(JSON.parse(fs.readFileSync(e2eFile, 'utf-8')).saga_year).toBe(1220);

    await type(SAGA_YEAR_INPUT, 1230);
    await expectValue(SAGA_YEAR_INPUT, 1230, 'the typed saga year');

    // D3.3. Neither stored value moved — the saga year governs only what the NEXT
    // edit derives.
    await expectValue(AGE_INPUT, 30, 'the stored age must not be recomputed');
    await expectValue(BIRTH_YEAR_INPUT, 1190, 'the stored birth year must not be recomputed');

    // C8's inversion, and the assertion with teeth: the year is part of what an
    // unsaved file would lose, so the unsaved-changes guard has to see it move.
    await browser.waitUntil(
      async () => (await browser.execute(() => document.title)).startsWith('*'),
      { timeout: STEP_TIMEOUT, timeoutMsg: 'moving the saga year must dirty the document' },
    );
    // The file on disk is still the one that was saved, unchanged.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.age).toBe(30);
    expect(saved.birth_year).toBe(1190);
    expect(saved.saga_year).toBe(1220);

    // What it DOES govern: the next edit is measured against 1230.
    await type(AGE_INPUT, 40);
    await expectValue(BIRTH_YEAR_INPUT, 1190, 'age 40 in a 1230 saga is a birth year of 1190');
    await type(BIRTH_YEAR_INPUT, 1200);
    await expectValue(AGE_INPUT, 30, 'a birth year of 1200 in a 1230 saga is age 30');
  });

  it('carries the saga year through a save and an open, not through a relaunch', async () => {
    // The whole of C8 against the shipped binary. Before it, this test reloaded the
    // webview and expected the year back from a settings file; that is exactly the
    // behaviour that made one number wrong for every saga but one. Now the year
    // belongs to the document, so it must survive a WRITE and a READ of that
    // document — and a brand-new character must NOT inherit it.
    await type(SAGA_YEAR_INPUT, 1231);
    await expectValue(SAGA_YEAR_INPUT, 1231, 'the saga year to persist');

    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(
      () =>
        fs.existsSync(e2eFile) && JSON.parse(fs.readFileSync(e2eFile, 'utf-8')).saga_year === 1231,
      { timeout: STEP_TIMEOUT, timeoutMsg: 'the saga year did not reach the saved file' },
    );

    // A fresh character starts at the configured default, NOT at 1231 — the year is
    // the document's, not the installation's.
    await startWizard('grog');
    await $(SAGA_YEAR_INPUT).waitForExist({ timeout: BOOT_TIMEOUT });
    await expectValue(SAGA_YEAR_INPUT, DEFAULT_SAGA_YEAR, 'a new character starts at the default');

    // And opening the saved file brings 1231 back, through the real load path. An
    // open lands in the editor, so the year is read off the Details tab — C8's other
    // new surface, and the one direct entry uses.
    await runDocumentAction('open');
    // Entering the wizard records its progress, so the fresh grog may be dirty;
    // discard it the way `magus-possessions.e2e.js`'s legacy-open test does.
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();
    const detailsTab = await $('[data-testid="tab-details"]');
    await detailsTab.waitForExist({ timeout: BOOT_TIMEOUT });
    await detailsTab.click();
    await $(SAGA_YEAR_INPUT).waitForExist({ timeout: BOOT_TIMEOUT });
    await expectValue(SAGA_YEAR_INPUT, 1231, 'the saga year the opened document carries');
  });
});

// End-to-end: the no-unsaved-changes case to this describe is
// `wizard-walks.e2e.js`'s `RunEvent::ExitRequested bridge — no unsaved changes`
// — RunEvent::ExitRequested (macOS Cmd+Q / app-level quit) coverage.
//
// `app-shell.e2e.js`'s `window.close() bridge — no unsaved changes` and
// `companion-editor.e2e.js`'s `window.close() bridge — unsaved changes`
// cover WindowEvent::CloseRequested, but that is a structurally different Tauri
// event from RunEvent::ExitRequested — the one Cmd+Q actually raises (see
// `crates/arm-app/src/main.rs`'s `request_exit` doc comment for the full story:
// window.close() always resolves to CloseRequested, never ExitRequested, so the
// window-close bridge cannot be reused to reach this path). No WebDriver
// capability available to this project can synthesize a real OS-level quit
// signal either, so this spec drives `request_exit` — a narrow, e2e-only IPC
// command (gated behind the `e2e-testing` Cargo feature this suite already
// builds with, see `wdio.conf.js`) that runs the exact same `guard_blocks_quit`
// call the real `RunEvent::ExitRequested` handler makes, triggered by IPC
// instead of a real quit signal.
//
// MUST BE THE LAST DESCRIBE IN THIS FILE: with unsaved edits, `guard_blocks_quit`
// raises a native GTK confirmation dialog that WebDriver cannot dismiss. That
// dialog is left open for the rest of this worker's app instance, so nothing may
// run after this describe in this file.
describe('RunEvent::ExitRequested bridge — unsaved changes', () => {
  const NAME_INPUT = '[data-testid="identity-name"]';
  const TAB_BAR = '[role="tablist"]';

  it('a dirty entity survives a request_exit IPC call', async () => {
    await startCharacter('companion');
    await $(NAME_INPUT).setValue('Quit bridge test dirty');

    // Invokes the same low-level call `window_close_shadow_script` uses
    // (`withGlobalTauri` is false, so there is no `window.__TAURI__` convenience
    // global — see `main.rs`'s `window_close_bridge_plugin` doc comment).
    await browser.execute(() => window.__TAURI_INTERNALS__.invoke('request_exit'));

    // `guard_blocks_quit` raises the native confirmation dialog and calls
    // `prevent_exit()` while the entity is dirty, so the process — and this
    // WebDriver session riding on it — must survive. Poll rather than a fixed
    // sleep: confirm the app stays alive and the edit survives once the IPC round
    // trip has had time to land.
    await browser.waitUntil(
      async () => (await $(NAME_INPUT).getValue()) === 'Quit bridge test dirty',
      {
        timeout: 5000,
        interval: 250,
        timeoutMsg:
          'the edit did not survive request_exit with unsaved changes — the app may have quit',
      },
    );
    expect(await $(TAB_BAR).isExisting()).toBe(true);
  });
});
