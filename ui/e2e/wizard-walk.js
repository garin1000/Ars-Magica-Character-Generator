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
  // The step's own field is the concept text; the name lives in the banner above
  // every view (`IdentityFields` says so), so the walk types both — the one the
  // step offers, and the one that proves at the end that Finish carried this very
  // character into the editor.
  concept: async (plan) => {
    const concept = await $('[data-testid="identity-concept"]');
    await concept.waitForExist({ timeout: STEP_TIMEOUT });
    await concept.setValue(plan.concept);
    const name = await $('[data-testid="identity-name"]');
    await name.waitForExist({ timeout: STEP_TIMEOUT });
    await name.setValue(plan.name);
  },

  // Read-only: the type was fixed before the wizard opened, so this step confirms
  // rather than asks. Its budget read-out is what proves the step rendered.
  type: async () => {
    await $('[data-testid="type-step-budget"]').waitForExist({ timeout: STEP_TIMEOUT });
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

  abilities: async (plan) => {
    if (plan.magusMinimums) await satisfyMagusMinimums();
    const pool = await $('[data-testid="xp-pool"]');
    await pool.waitForExist({ timeout: STEP_TIMEOUT });
    await pool.setValue(plan.xpPool);
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

  aging: async (plan) => {
    const age = await $('[data-testid="age-input"]');
    await age.waitForExist({ timeout: STEP_TIMEOUT });
    await age.setValue(plan.age);
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
  await browser.waitUntil(async () => (await $$('[data-severity="error"]')).length === 0, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: 'the finished character still holds error-severity findings',
  });
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
