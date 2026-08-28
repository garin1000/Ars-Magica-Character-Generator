// End-to-end: opening an EXISTING character into the guided wizard (Slice 5, #31).
// Two documents, two behaviours, both driven through the shipped binary:
//
//   1. a character saved part-way through the flow reopens on the step it was left
//      on, with the rail still stopping there — the wizard's progress survived the
//      save, which before this slice it never did;
//   2. a character built in the EDITOR opens with every rail step reachable and no
//      gating at all, because it never passed those gates in the first place.
//
// It also checks the unsaved-changes guard's new interaction on the real binary: a
// rail click still costs nothing, while a Next onto a step never reached before
// marks the document changed (the header's ASCII `*`).
//
// A grog is the subject: the shortest declared flow, and `virtues_flaws` is a step
// a fresh grog can reach with nothing filled in.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import {
  advanceWizardTo,
  currentWizardPhase,
  returnToStartScreen,
  standOnWizardStep,
  startCharacter,
  startWizard,
} from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const WIZARD_RAIL = '[data-testid="wizard-rail"]';
const START_OPEN_WIZARD = '[data-testid="start-open-wizard"]';
const SAVE_BUTTON = '[data-testid="save-button"]';
const STATUS = '[data-testid="doc-status"]';
const FINISH = '[data-testid="wizard-finish"]';
const NAME_INPUT = '[data-testid="identity-name"]';

const BOOT_TIMEOUT = 30000;
const STEP_TIMEOUT = 15000;

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

/** Save through the ARM_E2E_FILE seam and hand back the JSON that landed on disk. */
async function saveAndRead() {
  if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
  await $(SAVE_BUTTON).click();
  await browser.waitUntil(() => fs.existsSync(e2eFile), {
    timeout: STEP_TIMEOUT,
    timeoutMsg: 'save did not write the file',
  });
  return JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
}

/** Come back to the startup screen and re-enter through the guided-open button. */
async function reopenIntoWizard() {
  await returnToStartScreen();
  const open = await $(START_OPEN_WIZARD);
  await open.waitForClickable({ timeout: STEP_TIMEOUT });
  await open.click();
  await $(WIZARD_RAIL).waitForExist({ timeout: STEP_TIMEOUT });
}

/** Whether the header shows the ASCII dirty marker. */
async function isDirty() {
  return clean(await $(STATUS).getText()).startsWith('*');
}

describe('opening a saved character into the guided wizard', () => {
  it('resumes a mid-flow save on the step it was left on', async () => {
    await startWizard('grog');
    await $(WIZARD_RAIL).waitForExist({ timeout: BOOT_TIMEOUT });

    // A fresh wizard character is clean; the first Next onto a step never reached
    // before is what records progress, and therefore what marks the document
    // changed. That is the DECIDED trade: rail browsing stays free.
    expect(await isDirty()).toBe(false);
    await advanceWizardTo('characteristics');
    await browser.waitUntil(async () => await isDirty(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'advancing a wizard step should mark the document changed',
    });

    await advanceWizardTo('virtues_flaws');
    const saved = await saveAndRead();

    // The progress is in the file, as the phase SLUG — not an index into a phase
    // list that is ruleset data.
    expect(saved.wizard_furthest_phase).toBe('virtues_flaws');
    expect(saved.type_id).toBe('grog');

    await reopenIntoWizard();

    // It lands where it was left, not at the start.
    expect(await currentWizardPhase()).toBe('virtues_flaws');
    // And a just-loaded document is the saved one: no edit, no marker.
    expect(await isDirty()).toBe(false);

    // The rail still stops there: nothing past the stored phase was ever reached
    // during the original run, so nothing past it may be jumped to now.
    const experience = await $(`${WIZARD_RAIL} [data-testid="wizard-step-experience"]`);
    await experience.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await experience.isEnabled()).toBe(false);

    // Rail navigation across the restored steps costs nothing — the whole reason
    // only `furthest` is persisted and never the current step.
    await standOnWizardStep('concept');
    await standOnWizardStep('virtues_flaws');
    expect(await isDirty()).toBe(false);
  });

  it('opens an editor-built character with every step reachable and ungated', async () => {
    await startCharacter('grog');
    await $(NAME_INPUT).setValue('Rolf the Turb');

    // A Minor Virtue with no Flaw to fund it is `unbalanced_virtues`, an error — so
    // this document carries a BLOCKING finding on a step in the middle of the rail.
    // A wizard run would have been held at it; a character assembled in the editor
    // never passed that gate, so the flow must not lock the steps behind it.
    await $('[data-testid="tab-virtues_flaws"]').click();
    const addVirtue = await $('[data-testid="add-virtue.keen_vision"]');
    await addVirtue.waitForExist({ timeout: STEP_TIMEOUT });
    await addVirtue.click();
    await $('[data-testid^="remove-virtue.keen_vision"]').waitForExist({ timeout: STEP_TIMEOUT });

    const saved = await saveAndRead();
    // Nothing walked this character through the flow, so the file records no
    // progress at all — which is exactly what makes it the ungated case.
    expect(saved.wizard_furthest_phase).toBeUndefined();
    expect(saved.selections.some((s) => s.ref === 'virtue.keen_vision')).toBe(true);

    await reopenIntoWizard();

    // Every step of the rail is reachable, not just the first. Collect the refusals
    // and assert on the list, so a failure names the steps that stayed shut instead
    // of only the first one.
    const steps = await $$(`${WIZARD_RAIL} button`);
    expect(steps.length).toBeGreaterThan(1);
    const unreachable = [];
    for (let i = 0; i < steps.length; i++) {
      if (await steps[i].isEnabled()) continue;
      unreachable.push(await steps[i].getAttribute('data-testid'));
    }
    expect(unreachable).toEqual([]);

    // The finding is still SHOWN — the exemption is from enforcement, not from
    // being told. (The closing step's panel is the unfiltered one.)
    await standOnWizardStep('review');
    expect(await currentWizardPhase()).toBe('review');
    await $('[data-code="unbalanced_virtues"]').waitForExist({ timeout: STEP_TIMEOUT });

    // ...and it gates nothing: the jump above crossed the offending step instead of
    // clamping at it, and Finish is live.
    await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(FINISH).isEnabled()).toBe(true);
  });
});
