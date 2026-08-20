// Shared harness for the WebdriverIO specs in `specs/`. Non-spec harness modules
// live at the `e2e/` root — `display.js` is the precedent.
//
// WHY THIS EXISTS. The app boots on the startup choice screen and a character's
// type is fixed at creation, so there is no type selector to switch: the only way
// to a character of a given type is to create one. And creating one discards
// whatever came before, so no spec may inherit a character from an earlier `it`.
// Every spec therefore builds its own subject through `startCharacter`, from
// whatever state it finds itself in.
//
// (This comment used to add "the suite runs serially against one shared app
// instance". Serially, yes — `maxInstances: 1`. One instance, no: wdio spawns a
// worker per spec FILE and each opens its own WebDriver session, so every file
// gets a freshly launched app. Measured in M6/6b8d; see `e2e/README.md`. The
// advice above is unaffected, and is if anything stronger for it.)
//
// NAMING: this file must never be renamed to `*.test.js`. `ui/vitest.config.ts`
// includes `e2e/**/*.test.js` in `npm run test:unit` (it excludes only
// `e2e/specs/`), and this module imports `@wdio/globals`, which does not resolve
// outside a WebdriverIO worker — so that name would break the unit run.
//
// NO COMPANION UNIT TEST, unlike `display.js`, and that is deliberate rather than
// an oversight: `display.js` wraps a pure decision worth testing in isolation,
// whereas everything here is browser interaction (find, wait, click) with no pure
// logic to extract. A unit test could only assert against a mock of WebDriver.
// Its real coverage is every spec file in `specs/`, each of which fails loudly the
// moment it breaks. (No count here on purpose: the suite grows every slice.)

import { $, $$, browser } from '@wdio/globals';

const START_SCREEN = '[data-testid="start-screen"]';
const NEW_BUTTON = '[data-testid="new-button"]';
const DISCARD_CONFIRM = '[data-testid="discard-confirm"]';
// The editor's tab bar. Addressed by role rather than by one tab's testid: which
// tabs exist is a function of the type profile, but the bar itself always is.
const TAB_BAR = '[role="tablist"]';
// The guided wizard's step rail — the third screen. It has no tab bar (the flow's
// order is the point), so `returnToStartScreen` would otherwise wait out its full
// timeout on a wizard that is plainly on screen.
const WIZARD_RAIL = '[data-testid="wizard-rail"]';
const WIZARD_NEXT = '[data-testid="wizard-next"]';

// The first wait of a run also covers app start-up and the ruleset load over IPC
// (the create buttons are the loaded profiles, so they do not exist before it);
// every later wait is an ordinary DOM update. Same figures the specs use.
const BOOT_TIMEOUT = 30000;
const STEP_TIMEOUT = 10000;

/**
 * Bring the app to a freshly created character of `type` (`grog`, `companion`,
 * `mythic_companion`, `magus`, … — any id the loaded ruleset declares a profile
 * for) and leave it in the editor.
 *
 * Safe to call from any state: it returns to the startup screen first, answering
 * the discard prompt if the previous spec left unsaved edits behind.
 *
 * @param {string} type character-type id
 */
export async function startCharacter(type) {
  await returnToStartScreen();

  const create = await $(`[data-testid="start-create-${type}"]`);
  await create.waitForExist({ timeout: BOOT_TIMEOUT });
  await create.waitForClickable({ timeout: STEP_TIMEOUT });
  await create.click();

  await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
}

/**
 * Bring the app to a freshly created character of `type` and leave it in the
 * guided wizard, on its first step.
 *
 * Safe to call from any state, exactly like {@link startCharacter}.
 *
 * @param {string} type character-type id
 */
export async function startWizard(type) {
  await returnToStartScreen();

  const start = await $(`[data-testid="start-wizard-${type}"]`);
  await start.waitForExist({ timeout: BOOT_TIMEOUT });
  await start.waitForClickable({ timeout: STEP_TIMEOUT });
  await start.click();

  await $(WIZARD_RAIL).waitForExist({ timeout: STEP_TIMEOUT });
}

/**
 * The wizard rail's phase ids, in document order.
 *
 * @returns {Promise<string[]>}
 */
export async function wizardRailPhases() {
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

/**
 * The phase whose rail entry is marked current.
 *
 * @returns {Promise<string>}
 */
export async function currentWizardPhase() {
  const current = await $(`${WIZARD_RAIL} button[aria-current="step"]`);
  await current.waitForExist({ timeout: STEP_TIMEOUT });
  const testid = await current.getAttribute('data-testid');
  return testid.replace('wizard-step-', '');
}

/**
 * Walk the guided wizard forward to `phase`, pressing Next one step at a time and
 * waiting for the rail to follow each time — the flow has no way to jump to a step
 * it has not reached, so this is the only way to a later phase.
 *
 * A no-op when the wizard is already on `phase`. Fails loudly rather than spinning:
 * a phase the current character's rail does not declare is reported at once, and a
 * step whose gate blocks Next fails on the click (Next is disabled) or on the wait.
 *
 * @param {string} phase creation-phase id (`abilities`, `review`, …)
 */
export async function advanceWizardTo(phase) {
  const phases = await wizardRailPhases();
  if (!phases.includes(phase)) {
    throw new Error(`the wizard rail has no '${phase}' step; it offers ${phases.join(', ')}`);
  }

  let current = await currentWizardPhase();
  while (current !== phase) {
    const before = current;
    const nextButton = await $(WIZARD_NEXT);
    await nextButton.waitForClickable({
      timeout: STEP_TIMEOUT,
      timeoutMsg: `Next is not clickable on '${before}', so '${phase}' is unreachable`,
    });
    await nextButton.click();
    await browser.waitUntil(async () => (await currentWizardPhase()) !== before, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `the wizard did not advance past '${before}' on the way to '${phase}'`,
    });
    current = await currentWizardPhase();
  }
}

/**
 * Add one Ability row at `index` (its position in the character's Ability array),
 * name its instance where the Ability is parameterized, and raise it to 1.
 *
 * @param {string} ability ability id
 * @param {number} index expected row index
 * @param {string} [parameter] instance value, for a parameterized Ability
 */
async function addAbilityAtScoreOne(ability, index, parameter) {
  const add = await $(`[data-testid="add-${ability}"]`);
  await add.waitForExist({ timeout: STEP_TIMEOUT });
  await add.click();

  if (parameter !== undefined) {
    const field = await $(`[data-testid="ability-param-${ability}-${index}"]`);
    await field.waitForExist({ timeout: STEP_TIMEOUT });
    await field.setValue(parameter);
  }

  // `click` scrolls the element into view; `waitForClickable` does not, and the
  // Selected list is a scrolling box inside a squeezed step, so a freshly added row's
  // spinner can sit below the fold.
  const inc = await $(`[data-testid="ability-inc-${ability}-${index}"]`);
  await inc.waitForExist({ timeout: STEP_TIMEOUT });
  await inc.click();
  await browser.waitUntil(
    async () => (await $(`[data-testid="ability-score-${ability}-${index}"]`).getText()) === '1',
    { timeout: STEP_TIMEOUT, timeoutMsg: `${ability} did not reach a score of 1` },
  );
}

/**
 * Give the magus on screen the Abilities the Order demands of it, and the experience
 * to pay for them.
 *
 * "Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory 1,
 * Latin 1. Characters with lower scores would not be admitted to the Order."
 * (Ars Magica - Definitive Edition (Core Rules).md:2437.) Those three are BLOCKING
 * findings on the `abilities` phase for every magus, guided or flat — and an empty
 * experience pool leaves `not_enough_xp` blocking just as effectively — so any spec
 * walking a magus past that step has to settle both. Extracted rather than copied into
 * each spec, exactly like {@link advanceWizardTo}.
 *
 * Expects an Abilities surface on screen (the wizard's `abilities` step or the editor
 * tab) and a character with no Ability rows yet, so the three added take rows 0-2. The
 * experience pool is only filled where it is editable: under a life-stage plan the
 * engine forbids a typed pool, and apprenticeship funds these three many times over.
 *
 * @param {string} language the dead language to name (any dead language satisfies `:2437`)
 */
export async function satisfyMagusMinimums(language = 'Latin') {
  const pool = await $('[data-testid="xp-pool"]');
  if (await pool.isExisting()) {
    // Five experience points each off the advancement table; 30 leaves headroom.
    await pool.setValue('30');
  }

  await addAbilityAtScoreOne('ability.parma_magica', 0);
  await addAbilityAtScoreOne('ability.magic_theory', 1);
  await addAbilityAtScoreOne('ability.dead_language', 2, language);
}

/**
 * Leave the editor or the wizard for the startup screen through the New button,
 * confirming the discard prompt when there is something to discard. A no-op when
 * the startup screen is already showing.
 */
export async function returnToStartScreen() {
  // Decide only once the app has painted one of its three screens: on the very
  // first call it may still be starting up, and the New button lives on the
  // editor and wizard screens only.
  await browser.waitUntil(
    async () =>
      (await $(START_SCREEN).isExisting()) ||
      (await $(TAB_BAR).isExisting()) ||
      (await $(WIZARD_RAIL).isExisting()),
    {
      timeout: BOOT_TIMEOUT,
      timeoutMsg: 'the app rendered neither the startup screen, the editor, nor the wizard',
    },
  );
  if (await $(START_SCREEN).isExisting()) return;

  const newButton = await $(NEW_BUTTON);
  await newButton.waitForClickable({ timeout: STEP_TIMEOUT });
  await newButton.click();

  // New either lands on the startup screen at once (clean document) or raises the
  // discard prompt first (unsaved edits). Which of the two happens is state the
  // previous spec left behind, so wait for whichever arrives rather than sleeping
  // for a fixed time and guessing.
  await browser.waitUntil(
    async () => (await $(START_SCREEN).isExisting()) || (await $(DISCARD_CONFIRM).isExisting()),
    {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'New neither returned to the startup screen nor prompted to discard',
    },
  );

  const confirm = await $(DISCARD_CONFIRM);
  if (await confirm.isExisting()) {
    await confirm.click();
    await $(START_SCREEN).waitForExist({ timeout: STEP_TIMEOUT });
  }
}
