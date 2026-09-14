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
// an oversight for the browser-interaction helpers: `display.js` wraps a pure
// decision worth testing in isolation, whereas startCharacter/advanceWizardTo/etc.
// are browser interaction (find, wait, click) with no pure logic to extract — a
// unit test could only assert against a mock of WebDriver. The one exception is
// `clean()` (V34): a one-line regex strip with no WebDriver dependency, moved here
// from 24 copy-pasted spec files. It stays untested on its own rather than gaining
// a dedicated unit test for a single `String.replace`, because it already gets the
// same "fails loudly the moment it breaks" coverage as the rest of this file: every
// spec that calls `textOf()` or `clean()` directly exercises it. Its real coverage
// is every spec file in `specs/`, each of which fails loudly the moment it breaks.
// (No count here on purpose: the suite grows every slice.)

import { $, $$, browser } from '@wdio/globals';

const START_SCREEN = '[data-testid="start-screen"]';
const DISCARD_CONFIRM = '[data-testid="discard-confirm"]';
// The editor's tab bar. Addressed by role rather than by one tab's testid: which
// tabs exist is a function of the type profile, but the bar itself always is.
const TAB_BAR = '[role="tablist"]';
// The guided wizard's step rail — the third screen. It has no tab bar (the flow's
// order is the point), so `returnToStartScreen` would otherwise wait out its full
// timeout on a wizard that is plainly on screen.
const WIZARD_RAIL = '[data-testid="wizard-rail"]';
const WIZARD_NEXT = '[data-testid="wizard-next"]';

// The settings dialog (C4) and the two controls that moved into it from the
// header. The ids are unchanged — the controls were relocated, not rewritten —
// so every spec that already named them keeps working once the dialog is open.
const SETTINGS_DIALOG = '[data-testid="settings-dialog"]';
const LANGUAGE_SELECT = '[data-testid="language-select"]';
const MODE_SELECT = '[data-testid="mode-select"]';
const THEME_SELECT = '[data-testid="theme-select"]';

// The first wait of a run also covers app start-up and the ruleset load over IPC
// (the create buttons are the loaded profiles, so they do not exist before it);
// every later wait is an ordinary DOM update. Same figures the specs use.
//
// Exported (V36) so specs import these instead of redefining the same two
// numbers locally. A spec with a genuinely different value (e.g.
// `wizard-flow.e2e.js`'s `opening a saved character into the guided wizard`
// describe, whose `STEP_TIMEOUT` is longer) keeps its own local const instead of
// importing this one — see that describe for why.
export const BOOT_TIMEOUT = 30000;
export const STEP_TIMEOUT = 10000;

// Fluent wraps interpolated values in Unicode bidi isolation marks (FSI/PDI);
// strip them so plain substring/equality matching on rendered text works.
// Extracted (V34) from 24 copy-pasted spec-local definitions.
export function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

/**
 * The visible, bidi-stripped text of one element.
 *
 * Extracted (V35) from 5 copy-pasted spec-local definitions, all built on
 * {@link clean}.
 *
 * @param {string} selector
 * @returns {Promise<string>}
 */
export async function textOf(selector) {
  return clean(await $(selector).getText());
}

// The native menu item each document action lives on — `arm_app::menu`'s
// `ACTION_IDS`, which `ui/src/lib/menu.ts` maps back to the store action.
const MENU_ITEM_IDS = {
  new: 'menu.new',
  open: 'menu.open',
  save: 'menu.save',
  saveAs: 'menu.save-as',
  export: 'menu.export',
  settings: 'menu.settings',
};

// NO CHORD TABLE LIVES HERE ANY MORE, and WebDriver is why (C7).
//
// This file used to press Ctrl+N/O/S, Ctrl+Shift+S and Ctrl+Shift+E, because
// C3c made those chords a webview keydown handler after the toolbar went. C7
// moved every one of them onto its menu item as an accelerator, declared there
// and nowhere else (`accelerator_for`, `crates/arm-app/src/menu.rs`), which is
// what lets the OS draw the chord beside the label and what removes the
// double-fire hazard of two owners.
//
// The accelerator is dispatched ABOVE the webview — GTK matches the toplevel's
// accel group in `gtk_window_key_press_event`, before the focused widget sees
// the key — and WebDriver cannot get there. `browser.keys` goes to
// WebKitWebDriver, which feeds a synthesized key into WebKit's own input
// pipeline rather than delivering it as an event on the window, so it reaches
// the page and stops. Measured in C7 rather than assumed: with the chords
// declared and the webview handler gone, every chord press was inert (F10 did
// not open the menubar either), while asking GTK directly —
// `gtk_accel_groups_activate` on the live window, for Ctrl+N — answered true.
// The accelerators work; they are simply not reachable from a spec.
//
// So a document action is driven by {@link runDocumentAction} below, which
// presses the menu item the accelerator is attached to.

/**
 * Press a native menu item, exactly as the OS handler would (C6).
 *
 * Fires `menu::forward_menu_action` — the single line `main.rs`'s
 * `on_menu_event` runs — so the id travels the real route to the frontend's
 * `runMenuAction`. Only exists in a build carrying the `e2e-testing` feature;
 * in the portable run the command is registered but inert.
 *
 * @param {string} id one of `arm_app::menu::ACTION_IDS`
 */
export async function activateMenuItem(id) {
  await browser.execute((itemId) => {
    window.__TAURI_INTERNALS__.invoke('activate_menu_item', { id: itemId });
  }, id);
}

/**
 * Invoke a document action by choosing its item off the native menu.
 *
 * WHY THE MENU AND NOT THE CHORD (C7). It is the same route: the chord is this
 * item's accelerator, so the OS turns a press into this very activation. It is
 * also the only one of the two a spec can take — see the block above. Addressing
 * the item rather than the keyboard is the better shape anyway: a spec that
 * means "save" says save, instead of silently becoming a test of key delivery.
 *
 * NOT available in the portable run, which builds without `e2e-testing`; the
 * command is registered there but inert. `portable-rules.e2e.js` therefore uses
 * no document action at all.
 *
 * @param {'new'|'open'|'save'|'saveAs'|'export'|'settings'} action
 */
export async function runDocumentAction(action) {
  const id = MENU_ITEM_IDS[action];
  if (id === undefined) {
    throw new Error(
      `no menu item for the document action '${action}'; it offers ${Object.keys(
        MENU_ITEM_IDS,
      ).join(', ')}`,
    );
  }
  await activateMenuItem(id);
}

/**
 * Wait until the app shell is interactive again after a document action.
 *
 * WHY THIS EXISTS (Erika E4, full-audit round 3). A document action leaves the
 * whole shell `inert` while it runs (`App.svelte`, keyed on `store.busy`), and a
 * click delivered into an `inert` subtree is swallowed in silence — no error, no
 * event, no handler. So "the file appeared on disk" is NOT the signal to click
 * on: Rust writes the file before the frontend's `save()` promise resolves, which
 * means the filesystem goes quiet strictly BEFORE the UI does. A spec that waits
 * only on the file and then clicks races the tail of its own save, and loses on a
 * loaded machine — round 2's gate run recorded exactly that, absorbed by
 * `specFileRetries`.
 *
 * `waitForClickable` is not the check either: it resolves through
 * `elementsFromPoint`, which returns the whole stack at a point, so the busy
 * scrim lying over the button does not make the button unclickable by that
 * definition. The scrim's own presence is the honest signal, and it renders if
 * and only if `store.busy` does.
 *
 * USE IT ONLY ONCE THE ACTION IS KNOWN TO BE IN FLIGHT — after a wait that
 * cannot pass until the app is busy, such as "the save file exists". Called the
 * instant an action is dispatched it proves nothing, because the scrim it waits
 * to see gone may not have been drawn yet.
 */
export async function waitForIdle() {
  await $('[data-testid="busy-overlay"]').waitForExist({
    reverse: true,
    timeout: STEP_TIMEOUT,
  });
}

/**
 * Open the settings dialog (C4) and leave it open.
 *
 * The header button rather than the native menu's Settings item: a native menu
 * is not in the webview, so WebDriver cannot click it, and the seam would take
 * this helper out of the portable run. Both routes call the same
 * `store.runDocumentAction('settings')`.
 */
export async function openSettings() {
  const dialog = await $(SETTINGS_DIALOG);
  if (await dialog.isExisting()) return;

  const button = await $('[data-testid="settings-button"]');
  await button.waitForExist({ timeout: BOOT_TIMEOUT });
  await button.waitForClickable({ timeout: STEP_TIMEOUT });
  await button.click();
  await dialog.waitForExist({ timeout: STEP_TIMEOUT });
}

/** Dismiss the settings dialog if it is open. */
export async function closeSettings() {
  const dialog = await $(SETTINGS_DIALOG);
  if (!(await dialog.isExisting())) return;

  await $('[data-testid="settings-close"]').click();
  await dialog.waitForExist({ timeout: STEP_TIMEOUT, reverse: true });
}

/**
 * Choose the UI language, through the settings dialog that holds it since C4,
 * and wait for the choice to land before returning.
 *
 * Every spec that switches language goes through here rather than naming the
 * select itself: the control moved once already, and one helper is one place to
 * follow it.
 *
 * @param {'en'|'de'} value
 */
export async function setLanguage(value) {
  await openSettings();
  await $(LANGUAGE_SELECT).selectByAttribute('value', value);
  await browser.waitUntil(async () => (await $(LANGUAGE_SELECT).getValue()) === value, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `language should switch to ${value}`,
  });
  await closeSettings();
}

/**
 * Choose the palette, through the settings dialog (C4).
 *
 * @param {'auto'|'light'|'dark'} value
 */
export async function setTheme(value) {
  await openSettings();
  await $(THEME_SELECT).selectByAttribute('value', value);
  await browser.waitUntil(async () => (await $(THEME_SELECT).getValue()) === value, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `theme should switch to ${value}`,
  });
  await closeSettings();
}

/**
 * Choose the validation mode, through the settings dialog that holds it since C4.
 *
 * @param {'enforced'|'advisory'|'silent'} value
 */
export async function setValidationMode(value) {
  await openSettings();
  await $(MODE_SELECT).selectByAttribute('value', value);
  await browser.waitUntil(async () => (await $(MODE_SELECT).getValue()) === value, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `validation mode should switch to ${value}`,
  });
  await closeSettings();
}

/** The validation mode currently chosen, read from the dialog. */
export async function validationMode() {
  await openSettings();
  const value = await $(MODE_SELECT).getValue();
  await closeSettings();
  return value;
}

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
 * Switch a fresh wizard character from its new life-stage default (M6/D2) back to
 * the flat, typed pool, and leave the wizard standing on `experience`.
 *
 * The guided wizard now defaults every new character to life-stage funding with an
 * allocated (empty) plan, so `Next` on `experience` blocks on
 * `life_stage_age_unset` / `life_stage_native_language_unset` until an age and a
 * native language are supplied. A spec that is not about life-stage funding at all
 * — general wizard flow, aging — has no reason to supply either, so it calls this
 * once, right after {@link startWizard}, to opt back into the simple pool exactly
 * as a player who does not want the guided plan would click the other radio.
 *
 * Advances to `experience` itself (a fresh character always starts before it), so
 * the caller does not need its own {@link advanceWizardTo} call first. A no-op
 * once the pool is already selected, and when the ruleset ships no life-stage
 * rules at all — the panel, and the choice, do not exist.
 */
export async function useFlatPoolFunding() {
  await advanceWizardTo('experience');
  const funding = await $('[data-testid="ability-funding-pool"]');
  if (!(await funding.isExisting())) return;
  if (await funding.isSelected()) return;
  await funding.click();
  await browser.waitUntil(async () => funding.isSelected(), {
    timeout: STEP_TIMEOUT,
    timeoutMsg: 'switching to flat-pool funding did not select the radio',
  });
}

/**
 * Stand on the wizard's `phase`, whichever side of the current step it is on.
 *
 * A rail click rather than Next, because it is the only navigation that goes both
 * ways: {@link advanceWizardTo} can only move forward. Needed since Slice 2 split
 * the funding plan (`experience`) from the experience it prices (`abilities`), so a
 * spec about the two together has to move between the steps that own them.
 *
 * A no-op when the wizard is already there. Fails loudly rather than spinning: an
 * undeclared or unvisited phase is reported at once (its rail entry is disabled
 * until reached), and a forward jump over a phase holding an error clamps there —
 * `wizardGoTo` gates exactly as Next does — so the wait reports where it stopped.
 *
 * @param {string} phase creation-phase id
 */
export async function standOnWizardStep(phase) {
  if ((await currentWizardPhase()) === phase) return;

  const entry = await $(`${WIZARD_RAIL} [data-testid="wizard-step-${phase}"]`);
  await entry.waitForExist({ timeout: STEP_TIMEOUT });
  if (!(await entry.isEnabled())) {
    throw new Error(`the wizard rail has not reached '${phase}' yet, so it cannot be jumped to`);
  }
  // Re-click inside the wait, rather than clicking once and then waiting. A forward
  // rail jump is CLAMPED at the first blocking phase (`firstBlockedPhaseIndex`), and
  // validation settles on a round trip to Rust — so a jump issued in the frame after
  // an edit can be clamped short by a finding that is about to clear, and that single
  // click is then spent. Waiting alone would spin to the timeout while the rail sat
  // one step short. Clicking again each poll lets the jump land as soon as the
  // transient block lifts, and rail navigation is idempotent so a repeat is free.
  // (Observed twice on `life-stage-childhood`'s funding-switch test, both times
  // passing on the spec retry — a flake that was really a missing settle.)
  await browser.waitUntil(
    async () => {
      if ((await currentWizardPhase()) === phase) return true;
      await entry.click();
      return (await currentWizardPhase()) === phase;
    },
    {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `a rail jump to '${phase}' did not land there — a step in between stayed blocked`,
    },
  );
}

/**
 * Type the character's age, and come back to the step you were standing on.
 *
 * Slice 12 (#24) gave the age ONE home: the `concept` step, beside the birth year it
 * is now linked to. Before that it was typed on whichever surface needed it — the
 * aging step under flat funding, the life-stage panel under guided funding — so
 * several specs used to reach for a field on their own step. They cannot any more,
 * and hunting for the age is not what any of them is about, so the walk back and
 * forth lives here once instead of in each of them.
 *
 * The wait is on the input reading the value back rather than on a fixed pause: the
 * edit dirties the document and schedules a validate, so the round trip settles
 * before the caller's own step re-renders around the new age.
 *
 * @param {string|number} age the age to type
 */
export async function setWizardAge(age) {
  const returnTo = await currentWizardPhase();
  await standOnWizardStep('concept');

  const input = await $('[data-testid="age-input"]');
  await input.waitForExist({ timeout: STEP_TIMEOUT });
  await input.setValue(String(age));
  await browser.waitUntil(async () => (await input.getValue()) === String(age), {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `the age ${age} did not stay in the concept step's field`,
  });

  if (returnTo !== 'concept') await standOnWizardStep(returnTo);
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
 * experience pool is only filled where it is editable: under life-stage funding the
 * pools are derived and no input is offered, and apprenticeship funds these three
 * many times over anyway.
 *
 * @param {string} language the dead language to name (any dead language satisfies `:2437`)
 */
export async function satisfyMagusMinimums(language = 'Latin') {
  const pool = await $('[data-testid="xp-pool"]');
  if (await pool.isExisting()) {
    // Seed a pool only if the caller has not already set one. Five experience
    // points each off the advancement table, so 30 covers the three minimums with
    // headroom — but it must never CLOBBER a larger total: the guided walk sets its
    // own pool on the experience step, which now runs before this helper, and
    // overwriting it there silently starved the rest of the character and surfaced
    // as an unexplained `not_enough_xp` on the Review step.
    // Under life-stage funding the field is a read-only span, so `isExisting()`
    // already makes the whole block a no-op.
    const existing = await pool.getValue();
    if (existing === '' || existing === '0') await pool.setValue('30');
  }

  await addAbilityAtScoreOne('ability.parma_magica', 0);
  await addAbilityAtScoreOne('ability.magic_theory', 1);
  await addAbilityAtScoreOne('ability.dead_language', 2, language);
}

/**
 * Leave the editor or the wizard for the startup screen through New, confirming
 * the discard prompt when there is something to discard. A no-op when the
 * startup screen is already showing.
 */
export async function returnToStartScreen() {
  // A settings dialog a previous spec left open would make the shell `inert`,
  // so the New below would land on a screen the user cannot see past — `inert`
  // reaches neither a menu item nor the accel group above it. Every helper here
  // closes the dialog behind itself, so this is belt-and-braces against a spec
  // that opened it by hand.
  await closeSettings();

  // Decide only once the app has painted one of its three screens: on the very
  // first call it may still be starting up, and New only leaves a screen that
  // has been reached.
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

  await runDocumentAction('new');

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

/**
 * Resize the window and wait for the VIEWPORT to follow.
 *
 * The three copies this replaces all waited for `window.innerHeight === height`,
 * i.e. they assumed the window's outer height and its viewport were the same
 * number. That held until C3a (`44b005a`) gave the app a native menu bar: on
 * GTK the menubar lives INSIDE the window, so a window resized to 600 reports a
 * viewport of 573 and the equality can never be satisfied again. The three
 * specs that resize (`wizard-flow`'s two describes and `magus-apprenticeship`)
 * therefore timed out on the resize itself, before reaching a single assertion.
 *
 * The inset is measured rather than hardcoded — it is chrome the platform
 * decides, and a macOS build puts the same menu in the system bar and has none
 * — so the wait keeps its exact equality instead of degrading to a tolerance.
 * Measuring before the resize is safe: the inset is a property of the chrome,
 * not of the height.
 *
 * @param {number} width
 * @param {number} height outer window height to resize to
 */
export async function resizeWindowTo(width, height) {
  const before = await browser.getWindowSize();
  const inset = before.height - (await browser.execute(() => window.innerHeight));
  await browser.setWindowSize(width, height);
  await browser.waitUntil(
    async () => (await browser.execute(() => window.innerHeight)) === height - inset,
    {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `the window should resize to ${height}px tall (viewport ${height - inset}px)`,
    },
  );
}

/**
 * Whether a `SourcePicker` row (an `add-*` button — V/F, Abilities, Spells,
 * Equipment) is greyed out. These rows stay natively enabled and signal
 * "blocked" through `aria-disabled` instead of the `disabled` attribute, so
 * that a screen-reader/keyboard user can still reach the row and its tooltip
 * explaining why (see `SourcePicker.svelte`). WebDriver's own `isEnabled()`
 * only ever reflects the native `disabled` attribute — never `aria-disabled`
 * — so it always reports `true` for these rows regardless of their real
 * state; use this helper instead everywhere a spec needs to know whether an
 * `add-*` row is currently takeable.
 *
 * @param {WebdriverIO.Element} element the row's button element
 * @returns {Promise<boolean>}
 */
export async function isRowBlocked(element) {
  return (await element.getAttribute('aria-disabled')) === 'true';
}
