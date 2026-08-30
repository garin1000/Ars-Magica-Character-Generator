// Shared driver for the four per-type wizard walks — `grog-wizard.e2e.js`,
// `companion-wizard.e2e.js`, `mythic-companion-wizard.e2e.js` and
// `magus-wizard.e2e.js` (milestone 6b8d).
//
// WHY THIS EXISTS. The four walks are the same walk: start the wizard, stand on
// every phase the character's type declares, put a real choice into it, and come
// out the far end with a complete, legal character. Only the *choices* differ, so
// they are the only thing a spec supplies — a `plan` object — while the driving
// lives here. Copy-pasting four near-identical walks would have meant fixing every
// step-rail change four times.
//
// THE PHASE LIST IS DATA, never a literal in a spec. `declaredPhases` reads
// `rules/core/character_types.json` — the very file the app loads — so adding a
// phase to a profile makes the walk visit it (and fail loudly if no filler knows
// how). The rail is then asserted to be exactly that list plus the wizard's own
// synthetic `review` step.
//
// EVERY PHASE IS FILLED, not Next-clicked past. After each filler the walk waits
// for the phase's incompleteness mark to clear, which is the engine's own
// `CompletenessReport` (6b8a) saying the input was accepted and stored. That is
// what makes this the first suite able to prove a character is *complete* rather
// than merely legal.
//
// NAMING: like `helpers.js` and `display.js` this is a non-spec harness module at
// the `e2e/` root and must never be renamed to `*.test.js` — `ui/vitest.config.ts`
// picks up `e2e/**/*.test.js` for the unit run, and `@wdio/globals` does not
// resolve outside a WebdriverIO worker.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { $, $$, browser, expect } from '@wdio/globals';

import {
  currentWizardPhase,
  satisfyMagusMinimums,
  startWizard,
  wizardRailPhases,
} from './helpers.js';
import { e2eFile } from './wdio.conf.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

// Re-exported so any importer of this module can still reach the fixture path
// through it, but `wdio.conf.js` is the single declaration (E11): every other
// consumer imports the same binding rather than re-declaring the literal.
export { e2eFile };

const NEXT = '[data-testid="wizard-next"]';
const FINISH = '[data-testid="wizard-finish"]';
const TAB_BAR = '[role="tablist"]';
const WIZARD_RAIL = '[data-testid="wizard-rail"]';

const STEP_TIMEOUT = 15000;

/**
 * The creation phases `typeId`'s profile declares, in its declared order, read
 * from the rules data rather than written down here.
 *
 * @param {string} typeId character-type id
 * @returns {string[]}
 */
export function declaredPhases(typeId) {
  const file = path.resolve(repoRoot, 'rules/core/character_types.json');
  const profiles = JSON.parse(fs.readFileSync(file, 'utf-8'));
  const profile = profiles.find((entry) => entry.id === typeId);
  if (!profile) {
    throw new Error(`no '${typeId}' profile in ${file}; it declares ${profiles.map((p) => p.id)}`);
  }
  return profile.creation_phases;
}

/** Raise one Characteristic to `score` and wait for the spinner to read it back. */
async function raiseCharacteristic(characteristic, score) {
  const inc = await $(`[data-testid="char-inc-${characteristic}"]`);
  await inc.waitForExist({ timeout: STEP_TIMEOUT });
  for (let i = 0; i < score; i++) await inc.click();
  await browser.waitUntil(
    async () =>
      (await $(`[data-testid="char-value-${characteristic}"]`).getText()).includes(`+${score}`),
    { timeout: STEP_TIMEOUT, timeoutMsg: `${characteristic} did not reach +${score}` },
  );
}

/**
 * Take one catalogue item off the Available list and wait for it to appear on the
 * Selected one.
 *
 * Every picker adds through `add-<id>` (`SourcePicker`), but the Selected side is
 * each tab's own markup: Virtues and Flaws remove through `remove-<id>-<i>` while
 * a spell removes through `spell-remove-<id>-<i>`, so the row's test-id prefix is
 * the caller's to name.
 *
 * @param {string} itemId the item's rules id
 * @param {string} [rowPrefix] test-id prefix of its Selected row
 */
async function take(itemId, rowPrefix = 'remove-') {
  const add = await $(`[data-testid="add-${itemId}"]`);
  await add.waitForExist({ timeout: STEP_TIMEOUT });
  // `click` scrolls the button into view; the Available list is a scrolling box.
  await add.click();
  await browser.waitUntil(
    async () => (await $$(`[data-testid^="${rowPrefix}${itemId}"]`)).length > 0,
    {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `taking '${itemId}' did not put it on the Selected list`,
    },
  );
}

/** Add one Ability row at `index`, name its instance where parameterized, buy score 1. */
async function buyAbility(ability, index, parameter) {
  const add = await $(`[data-testid="add-${ability}"]`);
  await add.waitForExist({ timeout: STEP_TIMEOUT });
  await add.click();

  if (parameter !== undefined) {
    const field = await $(`[data-testid="ability-param-${ability}-${index}"]`);
    await field.waitForExist({ timeout: STEP_TIMEOUT });
    await field.setValue(parameter);
  }

  const inc = await $(`[data-testid="ability-inc-${ability}-${index}"]`);
  await inc.waitForExist({ timeout: STEP_TIMEOUT });
  await inc.click();
  await browser.waitUntil(
    async () => (await $(`[data-testid="ability-score-${ability}-${index}"]`).getText()) === '1',
    { timeout: STEP_TIMEOUT, timeoutMsg: `${ability} did not reach a score of 1` },
  );
}

// One filler per creation phase, keyed by the engine's own phase slug. Every
// phase a shipped profile declares must have an entry — `fillPhase` throws
// otherwise, so a thirteenth phase fails these specs rather than walking through
// unfilled. Each filler records a real choice the phase owns; the walk then waits
// for the completeness mark to clear, so "the input was accepted" is checked once,
// centrally, rather than in eleven different ways here.
const FILLERS = {
  // The step's own fields are the concept text and — since Slice 12 (#24) — the
  // AGE, which now sits beside the birth year it is linked to instead of on the
  // aging step three phases later. The name lives in the banner above every view
  // (`IdentityFields` says so), so the walk types that too: it is what proves at the
  // end that Finish carried this very character into the editor.
  concept: async (plan) => {
    const concept = await $('[data-testid="identity-concept"]');
    await concept.waitForExist({ timeout: STEP_TIMEOUT });
    await concept.setValue(plan.concept);
    const name = await $('[data-testid="identity-name"]');
    await name.waitForExist({ timeout: STEP_TIMEOUT });
    await name.setValue(plan.name);
    const age = await $('[data-testid="age-input"]');
    await age.waitForExist({ timeout: STEP_TIMEOUT });
    await age.setValue(plan.age);
  },

  characteristics: async (plan) => {
    for (const [characteristic, score] of Object.entries(plan.characteristics)) {
      await raiseCharacteristic(characteristic, score);
    }
  },

  // Flaws first, then Virtues: in Enforced mode an unfunded Virtue is an error and
  // the step would block on the way out. Same order a player uses.
  virtues_flaws: async (plan) => {
    for (const flaw of plan.flaws) await take(flaw);
    for (const virtue of plan.virtues) await take(virtue);
  },

  // Where the character's experience comes from (Slice 2 split this off the
  // `abilities` step). The walk keeps the DEFAULT flat funding: the pool total is
  // typed on the `abilities` step's XP bar below, and a life-stage plan would retire
  // that input and re-price every later step, so the mode is what this step decides
  // and the funding fieldset with `pool` standing selected is what proves it
  // rendered and holds a choice.
  //
  // Read-only for the same reason the deleted `type` filler was: the step's default
  // IS the walk's answer. Unlike that one, though, this phase reports itself ENGAGED
  // only once something is stored for it — `completeness.rs`:
  // `life_stages.is_some() || xp_pool > 0` — and under flat funding the one control
  // that can store either, the XP bar's pool input, is mounted on the NEXT step. So
  // the walk cannot clear this step's untouched mark while standing on it, and
  // `expectPhaseComplete` below says so out loud rather than the walk skipping it:
  // a step that asks for a number it does not offer a field for is the defect, not
  // the assertion.
  // Where the character's experience comes from. The walk keeps the default flat
  // funding, so the answer is the pool total — typed here, on the step that asks the
  // question, not on the step that spends it. That split is the point of the phase:
  // `abilities` now only buys against a total this step already set.
  experience: async (plan) => {
    await $('[data-testid="life-stage-panel"]').waitForExist({ timeout: STEP_TIMEOUT });
    const funding = await $('[data-testid="ability-funding-pool"]');
    await funding.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await funding.isSelected()).toBe(true);
    const pool = await $('[data-testid="xp-pool"]');
    await pool.waitForExist({ timeout: STEP_TIMEOUT });
    await pool.setValue(plan.xpPool);
    // Confirm the total reached the STORE, not merely the input. `value={typedPool}`
    // is not a `bind:`, so the DOM keeps whatever was typed even if the change never
    // committed — reading the field back proves nothing. `Available` is derived from
    // the engine, so it moves only if the total actually landed. Nothing is spent
    // yet on this step, so it equals the total.
    await browser.waitUntil(
      async () => (await $('[data-testid="xp-available"]').getText()).includes(plan.xpPool),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: `the experience pool total ${plan.xpPool} never reached the store`,
      },
    );
  },

  abilities: async (plan) => {
    // The total set on the experience step must still be here — this step spends
    // against it, and the same `xp-pool` field is mounted on both.
    const pool = await $('[data-testid="xp-pool"]');
    await pool.waitForExist({ timeout: STEP_TIMEOUT });
    await browser.waitUntil(async () => (await pool.getValue()) === String(plan.xpPool), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `the abilities step lost the experience pool: expected ${plan.xpPool}`,
    });
    if (plan.magusMinimums) await satisfyMagusMinimums();
    // Rows are appended, so a row's index is its position in the plan — after the
    // three the Order demands of a magus, where those were bought.
    let index = plan.magusMinimums ? 3 : 0;
    for (const ability of plan.abilities) await buyAbility(ability, index++);
  },

  arts: async (plan) => {
    for (const [art, score] of Object.entries(plan.arts)) {
      const inc = await $(`[data-testid="art-inc-${art}"]`);
      await inc.waitForExist({ timeout: STEP_TIMEOUT });
      for (let i = 0; i < score; i++) await inc.click();
      await browser.waitUntil(
        async () => (await $(`[data-testid="art-score-${art}"]`).getText()) === String(score),
        { timeout: STEP_TIMEOUT, timeoutMsg: `${art} did not reach ${score}` },
      );
    }
  },

  spells: async (plan) => {
    for (const spell of plan.spells) await take(spell, 'spell-remove-');
  },

  house_specialisation: async (plan) => {
    const select = await $('[data-testid="house-select"]');
    await select.waitForExist({ timeout: STEP_TIMEOUT });
    await select.selectByAttribute('value', plan.house);
  },

  mythic_type: async (plan) => {
    const select = await $('[data-testid="mythic-type-select"]');
    await select.waitForExist({ timeout: STEP_TIMEOUT });
    await select.selectByAttribute('value', plan.mythicType);
  },

  // A Personality Trait is enough to engage the step; a Reputation needs a Virtue
  // that grants one, so asking for one here would be an error, not a choice.
  personality_reputations: async (plan) => {
    const add = await $('[data-testid="personality-add"]');
    await add.waitForExist({ timeout: STEP_TIMEOUT });
    await add.click();
    const name = await $('[data-testid="personality-name-0"]');
    await name.waitForExist({ timeout: STEP_TIMEOUT });
    await name.setValue(plan.personalityTrait);
    for (let i = 0; i < 3; i++) await $('[data-testid="personality-inc-0"]').click();
    await browser.waitUntil(
      async () => (await $('[data-testid="personality-value-0"]').getText()).includes('+3'),
      { timeout: STEP_TIMEOUT, timeoutMsg: 'the Personality Trait did not reach +3' },
    );
  },

  // Nothing to fill any more, and that IS the assertion. The age this step's whole
  // schedule hangs on was typed on `concept` (Slice 12, #24), and the engine's own
  // completeness rule for this phase is `entity.age.is_some()` — so the walk proving
  // the step reads as complete here, without touching it, proves the age really
  // travelled from the step that now owns it. Aging ROLLS are not owed at these ages
  // and have their own spec (`aging.e2e.js`).
  aging: async () => {
    await $('[data-testid="aging-panel"]').waitForExist({ timeout: STEP_TIMEOUT });
    // The copy this step used to carry is gone, not merely unused.
    expect(await $('[data-testid="age-input"]').isExisting()).toBe(false);
  },
};

/**
 * Record this phase's choices from `plan`.
 *
 * @param {string} phase creation-phase id
 * @param {object} plan the spec's choices for its character type
 */
export async function fillPhase(phase, plan) {
  const filler = FILLERS[phase];
  if (!filler) {
    throw new Error(
      `no filler for the '${phase}' creation phase; the walk knows ${Object.keys(FILLERS).join(', ')}`,
    );
  }
  await filler(plan);
}

/** Wait for the rail to stop marking `phase` as holding no choices yet. */
async function expectPhaseComplete(phase) {
  await browser.waitUntil(
    async () => !(await $(`[data-testid="wizard-incomplete-${phase}"]`).isExisting()),
    {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `the '${phase}' step still reads as untouched after its choices were entered`,
    },
  );
}

/** Press Next and wait for the rail to follow, failing loudly on a blocked step. */
async function stepForward(phase) {
  const next = await $(NEXT);
  await next.waitForClickable({
    timeout: STEP_TIMEOUT,
    timeoutMsg: `Next is dead on '${phase}', so the walk cannot leave it`,
  });
  await next.click();
  await browser.waitUntil(async () => (await currentWizardPhase()) !== phase, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `the wizard did not advance past '${phase}'`,
  });
}

/**
 * Walk one character of `typeId` from the startup screen through every phase its
 * profile declares, filling each in, and leave the wizard standing on `review`.
 *
 * @param {string} typeId character-type id
 * @param {object} plan the choices to record, one property per phase that asks
 * @returns {Promise<string[]>} the phases the profile declared, in order
 */
export async function walkEveryPhase(typeId, plan) {
  const declared = declaredPhases(typeId);
  await startWizard(typeId);

  // The rail IS the profile, plus the closing step the wizard appends itself.
  expect(await wizardRailPhases()).toEqual([...declared, 'review']);

  for (const phase of declared) {
    expect(await currentWizardPhase()).toBe(phase);
    await fillPhase(phase, plan);
    await expectPhaseComplete(phase);
    await stepForward(phase);
  }

  expect(await currentWizardPhase()).toBe('review');
  await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });
  return declared;
}

/**
 * Assert the closing step reports a character that is both legal and finished:
 * no error-severity finding anywhere (the Review panel is unfiltered, so this is
 * the whole character, including the phases no step owns), and no phase left
 * marked incomplete.
 */
export async function expectCompleteAndErrorFree() {
  // Findings arrive on a round trip, so wait for the panel to settle rather than
  // reading it a frame early.
  // Name the offenders on failure. "still holds error-severity findings" alone sends
  // the next reader back into a ten-minute run just to learn which ones, and the
  // codes are already in the DOM.
  let errorCodes = [];
  try {
    await browser.waitUntil(
      async () => {
        const rows = await $$('[data-severity="error"]');
        errorCodes = [];
        for (const row of Array.from(rows)) errorCodes.push(await row.getText());
        return errorCodes.length === 0;
      },
      { timeout: STEP_TIMEOUT, timeoutMsg: 'still holds error-severity findings' },
    );
  } catch {
    throw new Error(
      `the finished character still holds error-severity findings: ${errorCodes.join(', ')}`,
    );
  }
  await expect($('[data-testid="wizard-review-complete"]')).toExist();
  expect(await $('[data-testid="wizard-review-incomplete"]').isExisting()).toBe(false);
  expect(await $(FINISH).isEnabled()).toBe(true);
}

/** Finish the wizard and land in the ordinary editor. */
export async function finishIntoEditor() {
  await $(FINISH).click();
  await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
  expect(await $(WIZARD_RAIL).isExisting()).toBe(false);
}

/**
 * Save the character in the editor and read the file back, then reopen it.
 *
 * @returns {Promise<object>} the saved JSON, so a spec can assert on the choices
 */
export async function saveAndReopen() {
  if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
  await $('[data-testid="save-button"]').click();
  await browser.waitUntil(() => fs.existsSync(e2eFile), {
    timeout: STEP_TIMEOUT,
    timeoutMsg: 'save did not write the file',
  });
  const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));

  await $('[data-testid="open-button"]').click();
  const discard = await $('[data-testid="discard-confirm"]');
  if (await discard.isExisting()) await discard.click();
  await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
  return saved;
}
