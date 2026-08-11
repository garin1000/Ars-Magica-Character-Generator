// End-to-end: the guided creation wizard, driven on a magus — the longest flow
// and the only one that exercises the House step's placement before Virtues &
// Flaws. Covers entry from the startup screen, the rail's order, back/forward
// navigation, per-step gating (and the validation mode that lifts it), and
// finishing into the editor. Drives the real binary.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`, so this cannot run without that build step.

import { $, $$, browser, expect } from '@wdio/globals';

import { startWizard } from '../helpers.js';

const WIZARD_RAIL = '[data-testid="wizard-rail"]';
const TAB_BAR = '[role="tablist"]';
const CHARACTER_TYPE = '[data-testid="character-type"]';
const BACK = '[data-testid="wizard-back"]';
const NEXT = '[data-testid="wizard-next"]';
const FINISH = '[data-testid="wizard-finish"]';
const NAME_INPUT = '[data-testid="identity-name"]';
const MODE_SELECT = '[data-testid="mode-select"]';
// A Minor Virtue with no prerequisites and no Flaws to fund it, so the V/F step
// carries exactly one deterministic error: `unbalanced_virtues`.
const ADD_VIRTUE = '[data-testid="add-virtue.keen_vision"]';
const REMOVE_VIRTUE = '[data-testid^="remove-virtue.keen_vision"]';

const BOOT_TIMEOUT = 30000;
const STEP_TIMEOUT = 10000;

/** Fluent wraps interpolated values in Unicode bidi isolation marks; strip them. */
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

/** The rail's phase ids, in document order. */
async function railPhases() {
  // Index-based loop: in webdriverio v9 the awaited `$$` result's `.map` does not
  // yield a plain iterable, so `Promise.all(items.map(...))` throws.
  const items = await $$(`${WIZARD_RAIL} button`);
  const phases = [];
  for (let i = 0; i < items.length; i++) {
    const testid = await items[i].getAttribute('data-testid');
    phases.push(testid.replace('wizard-step-', ''));
  }
  return phases;
}

/** Click Next and wait for the rail's current step to move on. */
async function next() {
  const before = await currentPhase();
  await $(NEXT).click();
  await browser.waitUntil(async () => (await currentPhase()) !== before, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `the wizard did not advance past '${before}'`,
  });
}

/** The phase whose rail entry is marked current. */
async function currentPhase() {
  const current = await $(`${WIZARD_RAIL} button[aria-current="step"]`);
  const testid = await current.getAttribute('data-testid');
  return testid.replace('wizard-step-', '');
}

describe('guided creation wizard', () => {
  it('walks a magus through its declared phases and finishes in the editor', async () => {
    await startWizard('magus');

    // The wizard replaces the tab bar: the flow's order is the whole point, so
    // there is no way to jump around it.
    await $(WIZARD_RAIL).waitForExist({ timeout: BOOT_TIMEOUT });
    expect(await $(TAB_BAR).isExisting()).toBe(false);

    // It edits a real character, so the banner is above it and the document
    // controls are reachable — the unsaved-changes guard promises the work can be
    // saved rather than lost.
    const label = clean(await $(CHARACTER_TYPE).getText());
    expect(label).toContain('Magus');
    expect(label).not.toContain('type-magus');
    await expect($('[data-testid="save-button"]')).toExist();
    await expect($(MODE_SELECT)).toExist();

    // The rail follows the ruleset, and the magus profile deliberately places the
    // House step BEFORE Virtues & Flaws — its free Virtue lands in that budget.
    const phases = await railPhases();
    expect(phases).toContain('house_specialisation');
    expect(phases).toContain('virtues_flaws');
    expect(phases.indexOf('house_specialisation')).toBeLessThan(phases.indexOf('virtues_flaws'));
    // The wizard appends its own closing step, which no profile declares.
    expect(phases.at(-1)).toBe('review');

    // Back is dead on the first step; Next advances and the rail follows.
    expect(await $(BACK).isEnabled()).toBe(false);
    expect(await currentPhase()).toBe('concept');
    await next();
    expect(await currentPhase()).toBe('type');
    expect(await $(BACK).isEnabled()).toBe(true);

    // Walk to Virtues & Flaws and break it: a Minor Virtue with no Flaws to fund
    // it is `unbalanced_virtues`, an error.
    while ((await currentPhase()) !== 'virtues_flaws') await next();
    const addVirtue = await $(ADD_VIRTUE);
    await addVirtue.waitForExist({ timeout: STEP_TIMEOUT });
    await addVirtue.click();

    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'an unfunded Virtue did not block the Virtues & Flaws step',
    });
    // The rail marks the offending step, and says why Next is dead.
    const vfStep = await $('[data-testid="wizard-step-virtues_flaws"]');
    expect(await vfStep.getAttribute('data-blocked')).toBe('true');
    await expect($('[data-testid="wizard-blocked-hint"]')).toExist();

    // The step's own findings are shown; a finding from another phase is not.
    await expect($('[data-code="unbalanced_virtues"]')).toExist();
    expect(await $('[data-code="house_unset"]').isExisting()).toBe(false);

    // Advisory downgrades every error to a warning, so the gate lifts with the
    // illegal state standing — the validation mode is the intended escape hatch.
    await $(MODE_SELECT).selectByAttribute('value', 'advisory');
    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Advisory mode did not lift the step gate',
    });
    await $(MODE_SELECT).selectByAttribute('value', 'enforced');
    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'returning to Enforced did not restore the step gate',
    });

    // Drop the Virtue: the step unblocks.
    await $(REMOVE_VIRTUE).click();
    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'removing the unfunded Virtue did not unblock the step',
    });

    // Type a name here, to prove Finish carries the character over untouched.
    await $(NAME_INPUT).setValue('Marcus of Bonisagus');

    // Walk back two steps, then jump forward through the rail to a step already
    // visited — the two halves of the flow's navigation.
    await $(BACK).click();
    await $(BACK).click();
    const twoBack = await currentPhase();
    await $(`[data-testid="wizard-step-virtues_flaws"]`).click();
    await browser.waitUntil(async () => (await currentPhase()) === 'virtues_flaws', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `a rail jump forward from '${twoBack}' did not land on Virtues & Flaws`,
    });

    // On to the closing step: Next gives way to Finish.
    while ((await currentPhase()) !== 'review') await next();
    expect(await $(NEXT).isExisting()).toBe(false);
    await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });

    await $(FINISH).click();

    // Finishing lands in the ordinary editor, with the character the wizard built.
    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(WIZARD_RAIL).isExisting()).toBe(false);
    expect(await $(NAME_INPUT).getValue()).toBe('Marcus of Bonisagus');
  });
});
