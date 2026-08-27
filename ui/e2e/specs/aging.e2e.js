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
//    character must roll on the aging table." (`:16565`) — so a grog of 40 owes
//    five rolls, for ages 36 through 40.
//   "AGING TOTAL: Stress die (no botch) + age/10 (round up) / - Living Conditions
//    modifier / - Longevity Ritual modifier" (`:16567-16569`).
//   "Modifiers marked with an asterisk are cumulative with each other" (`:16594`)
//    — a leper colony (-1) and a poor or unhealthy location (-2) stack to -3.
//
// For this grog, rolling for age 36 with a +1 ritual:
//
//   age modifier   ceil(36 / 10)      = +4
//   conditions     -(-3)              = +3   (subtracted, so a bad life costs)
//   ritual         -(+1)              = -1
//   standing total                    = +6
//   stress die 8   8 + 6              = 14   → "1 Aging Point in Quickness" (`:16603`)
//
// (Ars Magica - Definitive Edition (Core Rules).md.)
//
// NOTE: requires the production binary; the display comes from your desktop session
// or, when DISPLAY is unset, the Xvfb one WebdriverIO starts (see e2e/README.md).
// The wdio `onPrepare` hook builds `target/release/arm-app`, so this cannot run
// without that build step.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { advanceWizardTo, currentWizardPhase, startWizard } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const AGE_INPUT = '[data-testid="age-input"]';
const SCHEDULE = '[data-testid="aging-schedule"]';
const FIRST_ROLL_AGE = '[data-testid="aging-first-roll-age"]';
const ROLLS_NONE = '[data-testid="aging-rolls-none"]';
const ROLLS_OWED = '[data-testid="aging-rolls-owed"]';
const ROLLS_RECORDED = '[data-testid="aging-rolls-recorded"]';
const TOTAL_FORMULA = '[data-testid="aging-total-formula"]';
const CONDITIONS = '[data-testid="living-conditions"]';
const CONDITIONS_TOTAL = '[data-testid="living-conditions-total"]';
const CUMULATIVE_NOTE = '[data-testid="living-conditions-cumulative-note"]';
// The two asterisked rows this grog lives under (`:16588`, `:16591`).
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
const REVERT_36 = '[data-testid="aging-revert-36"]';
const LOG_EMPTY = '[data-testid="aging-log-empty"]';
const LOG_EFFECT_0 = '[data-testid="aging-log-effect-0"]';
const NEXT = '[data-testid="wizard-next"]';
const FINISH = '[data-testid="wizard-finish"]';
const TAB_BAR = '[role="tablist"]';
// The editor's own aging tab, mirroring the wizard's `aging` phase (#28).
const AGING_TAB = '[data-testid="tab-aging"]';
const DOC_STATUS = '[data-testid="doc-status"]';
// The docked step panel, scoped: other surfaces render `data-code` nodes too.
const DOCKED_ISSUES = '[data-testid="issue-list"]';

const STEP_TIMEOUT = 10000;
const BOOT_TIMEOUT = 30000;

/** Fluent wraps interpolated values in Unicode bidi isolation marks; strip them. */
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

/** The visible, bidi-stripped text of one element. */
async function textOf(selector) {
  return clean(await $(selector).getText());
}

/** How many findings of `code` the docked panel is showing. */
async function issueCount(code) {
  const found = await $$(`${DOCKED_ISSUES} [data-code="${code}"]`);
  return found.length;
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
    entries[characteristic] = await $(`[data-testid="aging-points-${characteristic}"]`).getValue();
  }
  return entries;
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

describe('the guided aging step', () => {
  it('reaches an aging step that owes a young character nothing', async () => {
    await startWizard('grog');
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
    // Typed HERE, on the aging step. Under flat (pool) funding this is the only age
    // field the wizard has — `life-stage-age-input` renders for life-stage funding
    // alone — so without the `AgeFields` extraction a guided grog could not enter
    // the one number the whole schedule hangs on.
    await $(AGE_INPUT).setValue('40');

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
    // rolls … before the game begins" (`:2232`) — filed on the AGING phase, which is
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

    // -1 and -2, cumulative with each other (`:16594`) — the ENGINE's resolved
    // modifier, not a sum the checklist made.
    await browser.waitUntil(async () => (await textOf(CONDITIONS_TOTAL)).includes('-3'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a leper colony and a poor location stack to -3',
    });
    // ASCII hyphen-minus, never the mathematical minus.
    expect(await textOf(CONDITIONS_TOTAL)).not.toContain('−');

    // Which rows stack is marked on the row and stated in words; the marking is
    // data (`cumulative` in `rules/core/aging.json`), so the test reads it back off
    // the very rows it ticked.
    expect(await $(condition(LEPER_COLONY)).getAttribute('data-cumulative')).toBe('true');
    expect(await $(condition(POOR_LOCATION)).getAttribute('data-cumulative')).toBe('true');
    expect(await $(CUMULATIVE_NOTE).isDisplayed()).toBe(true);

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
  });

  it('takes a Longevity Ritual bonus a grog did not make himself', async () => {
    // THE GAP THIS STEP CLOSES. "You can perform Longevity Rituals for others, even
    // for non-magi" (`:10672`), but the editor's only home for the ritual is the
    // Possessions tab, which is magus-gated — so for a grog this guided step is the
    // one place the bonus can be entered at all.
    await $(LONGEVITY_ADD).click();
    await $(LONGEVITY_BONUS).waitForExist({ timeout: STEP_TIMEOUT });
    await $(LONGEVITY_BONUS).setValue('1');

    // Subtracted like the conditions (`:16571`: "a high Longevity Ritual modifier
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
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(async () => !clean(await $(DOC_STATUS).getText()).startsWith('*'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'saving should clear the dirty marker before the die is typed',
    });

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

    // 14 is "1 Aging Point in Qik" in the book's shorthand (`:16603`) — and the
    // Characteristic reaches the player in words, through `characteristic-qik`.
    const outcome = await textOf(OUTCOME);
    expect(outcome).toContain('Quickness');
    expect(outcome).not.toContain('qik');
    // 14 is well past the apparent-age threshold of 3 (`:16577`).
    expect(outcome).toContain('Apparent age increases by one year.');

    // THE LOAD-BEARING ASSERTION: a total is not an outcome. Nothing is written to
    // the character until Apply, so no aging point has moved, the log is empty, and
    // the document is not even dirty.
    expect(Object.values(await agingPoints()).every((points) => points === '0')).toBe(true);
    expect(await $(LOG_EMPTY).isExisting()).toBe(true);
    expect(clean(await $(DOC_STATUS).getText()).startsWith('*')).toBe(false);
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
    await $(REVERT_36).click();

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
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });

    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    // Canonical serialization: the chosen conditions are ids, sorted.
    expect(saved.living_conditions).toEqual([LEPER_COLONY, POOR_LOCATION]);
    expect(saved.age).toBe(40);
    expect(saved.schema_version).toBe(15);
    // The widened log entry is the whole record of the year: what was rolled, what
    // it totalled, the conditions in force and the points it awarded. `year` is
    // absent because a grog with no birth year has no calendar year to write.
    expect(saved.aging_log).toEqual([
      {
        age: 36,
        effect: '',
        die: 8,
        total: 14,
        living_conditions: [LEPER_COLONY, POOR_LOCATION],
        points: { qik: 1 },
        apparent_age_increased: true,
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
    await $('[data-testid="open-button"]').click();
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
  // even for non-magi" (`:10672`), and the ritual this grog holds is a term of every
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
});
