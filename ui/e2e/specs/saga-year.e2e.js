// End-to-end: the SAGA YEAR and the age ↔ birth-year link it derives
// (guided-creation-review-2026-08 #25, Slice 12).
//
// WHY THIS SPEC IS MANDATORY RATHER THAN NICE TO HAVE. Everything this slice adds on
// the Rust side is invisible to every other layer of the suite: two new commands
// (`saga_year` / `set_saga_year`) reading and writing a settings file resolved at
// LAUNCH, and two more (`derive_age` / `derive_birth_year`) that the store calls on
// every keystroke in the age or birth-year field. Unit tests cover the file and the
// arithmetic; only a real-binary run covers the IPC bridge, the startup read, and the
// fact that the frontend's derivation actually reaches the entity that gets saved.
//
// THE THREE CLAIMS:
//
//  1. Age and birth year are two views of one fact — edit either, the other follows,
//     against the saga year.
//  2. A saga year BEFORE the birth year clamps the derived age to 0 and says why,
//     rather than underflowing the entity's unsigned `age`
//     (`saga_year_before_birth_year`, a warning on the `concept` phase).
//  3. Editing the saga year alone changes NEITHER stored value and does NOT dirty the
//     document. This is the one most likely to be got wrong and the one with teeth:
//     a silent recompute would fabricate ages that skipped their aging rolls, and it
//     would also make a setting trip the mandatory unsaved-changes guard.
//
// The default is a rules value the engine owns —
// "That domination persists until the present day, 1220."
// (Ars Magica - Definitive Edition (Core Rules).md:597) — so 1220 is asserted as the
// year a fresh installation reports, never typed in as a magic number first.
//
// SIDE EFFECT, DELIBERATELY RESTORED. The saga year is real persisted app state, so
// this spec writes the developer's own settings file, exactly as the shipped app
// would. The `after` hook puts it back to 1220 so a later run starts where this one
// found it.
//
// NOTE: requires the production binary; the display comes from your desktop session
// or, when DISPLAY is unset, the Xvfb one WebdriverIO starts (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`, so this cannot run without
// that build step.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { BOOT_TIMEOUT, clean, standOnWizardStep, startWizard, STEP_TIMEOUT } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const AGE_INPUT = '[data-testid="age-input"]';
const BIRTH_YEAR_INPUT = '[data-testid="identity-birth-year"]';
const SAGA_YEAR_INPUT = '[data-testid="saga-year-input"]';
const SAGA_YEAR_HINT = '[data-testid="saga-year-hint"]';
const DOC_STATUS = '[data-testid="doc-status"]';
// The docked step panel, scoped: other surfaces render `data-code` nodes too.
const DOCKED_ISSUES = '[data-testid="issue-list"]';
const CLAMP_CODE = 'saga_year_before_birth_year';

/** The published setting's year, and so the default a fresh installation reports. */
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

describe('the saga year', () => {
  before(async () => {
    await startWizard('grog');
    await $(AGE_INPUT).waitForExist({ timeout: BOOT_TIMEOUT });
  });

  // Leave the machine's real settings where they were found.
  after(async () => {
    await standOnWizardStep('concept');
    await type(SAGA_YEAR_INPUT, DEFAULT_SAGA_YEAR);
    await expectValue(SAGA_YEAR_INPUT, DEFAULT_SAGA_YEAR, 'restoring the default saga year');
  });

  it('offers the age beside the identity, with the setting they are measured against', async () => {
    // #24: ONE canonical home for the age, mirroring the editor's Details tab. The
    // count spans the step, because a duplicate is exactly what this replaced.
    expect(await $$(AGE_INPUT)).toHaveLength(1);
    expect(await $$(BIRTH_YEAR_INPUT)).toHaveLength(1);

    // The default is the engine's, not this field's: a fresh installation has no
    // settings file, and the year it reports is the published setting's own.
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

  it('changes no stored value and does not dirty the document when the saga year moves', async () => {
    // A clean baseline first: the claim is that this edit moves nothing, which can
    // only be read off a document that had nothing outstanding.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });
    const status = await $(DOC_STATUS);
    await browser.waitUntil(async () => !clean(await status.getText()).startsWith('*'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the document should be clean immediately after a save',
    });

    await type(SAGA_YEAR_INPUT, 1230);
    await expectValue(SAGA_YEAR_INPUT, 1230, 'the typed saga year');

    // D3.3. Neither stored value moved — the saga year governs only what the NEXT
    // edit derives — so the document is still the one that was saved.
    await expectValue(AGE_INPUT, 30, 'the stored age must not be recomputed');
    await expectValue(BIRTH_YEAR_INPUT, 1190, 'the stored birth year must not be recomputed');
    expect(clean(await status.getText()).startsWith('*')).toBe(false);
    // And the file on disk still holds the pair the save wrote.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.age).toBe(30);
    expect(saved.birth_year).toBe(1190);

    // What it DOES govern: the next edit is measured against 1230.
    await type(AGE_INPUT, 40);
    await expectValue(BIRTH_YEAR_INPUT, 1190, 'age 40 in a 1230 saga is a birth year of 1190');
    await type(BIRTH_YEAR_INPUT, 1200);
    await expectValue(AGE_INPUT, 30, 'a birth year of 1200 in a 1230 saga is age 30');
  });

  it('remembers the saga year across a reload of the app', async () => {
    // The startup read, which is the whole reason this slice needs an e2e run: the
    // year came back over IPC from a settings file `arm-app` wrote at the previous
    // test's keystroke. Reloading the webview re-runs the store's `init()` against the
    // real backend, so a year that never reached the file cannot survive it.
    await type(SAGA_YEAR_INPUT, 1231);
    await expectValue(SAGA_YEAR_INPUT, 1231, 'the saga year to persist');

    await browser.execute(() => window.location.reload());
    // Back on the startup screen — a reload drops the in-memory document, so the
    // character has to be rebuilt to see the field again.
    await $('[data-testid="start-screen"]').waitForExist({ timeout: BOOT_TIMEOUT });
    await startWizard('grog');
    await $(SAGA_YEAR_INPUT).waitForExist({ timeout: BOOT_TIMEOUT });
    await expectValue(SAGA_YEAR_INPUT, 1231, 'the saga year after a relaunch');
  });
});
