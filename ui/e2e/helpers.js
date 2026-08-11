// Shared harness for the WebdriverIO specs in `specs/`. Non-spec harness modules
// live at the `e2e/` root — `display.js` is the precedent.
//
// WHY THIS EXISTS. The app boots on the startup choice screen and a character's
// type is fixed at creation, so there is no type selector to switch: the only way
// to a character of a given type is to create one. And the suite runs serially
// against one shared app instance (`wdio.conf.js`), where creating a character
// discards whatever came before — so no spec may inherit a character from an
// earlier `it` or an earlier file. Every spec therefore builds its own subject
// through `startCharacter`, from whatever state its predecessor left behind.
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
// Its real coverage is the 25 spec files that fail loudly the moment it breaks.

import { $, browser } from '@wdio/globals';

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
