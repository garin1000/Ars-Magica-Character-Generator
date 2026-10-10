// End-to-end: the opt-in update check's consent prompt and its notice (M6.6).
//
// THE ONE SPEC THAT SEES THE PROMPT. Every other spec launches with a "Don't
// check" answer seeded into its worker's settings file (`driver.js`'s
// `seedUpdateCheckAnswer`, wired in by both wdio configs' `beforeSession`). This
// file is the exception: `wdio.conf.js` empties its worker's config home before
// the launch, so the first `it` below is a genuine first start, and hands the
// app `ARM_E2E_UPDATE_FEED` — a file standing in for GitHub's latest-release
// response (`fixtures/update-feed-newer.json`, `v99.0.0`). The `e2e-testing`
// build reads that file instead of the network, and its "Open release page" is
// a no-op; this spec never clicks it all the same.
//
// The `it`s continue one narrative in one app session, in this order: first
// start → decline → asked again for a newer version → enable → a later start.
// Each depends on the settings file the one before it left behind.

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { mergeWorkerSettings, workerConfigHome, workerSettingsFile } from '../driver.js';
import { BOOT_TIMEOUT, clean, closeSettings, openSettings, STEP_TIMEOUT } from '../helpers.js';

const PROMPT = '[data-testid="update-prompt"]';
const ENABLE = '[data-testid="update-prompt-enable"]';
const DECLINE = '[data-testid="update-prompt-decline"]';
const NOTICE = '[data-testid="update-notice"]';
const DISMISS = '[data-testid="update-notice-dismiss"]';
const TOGGLE = '[data-testid="update-check-toggle"]';
const START_CREATE = '[data-testid="start-create-magus"]';
const SHELL = '.app-shell';

// How long a "does not appear" claim waits. The prompt and the notice follow
// one IPC round trip each after start-up, so a few seconds is ample.
const ABSENCE_WINDOW = 3000;

// The worker-isolated config home this app was launched with — the same call,
// with the same inputs, that `wdio.conf.js`'s `beforeSession` made.
const configHome = workerConfigHome(
  path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..'),
  process.env,
);

/** The app's settings file as it stands on disk, or `{}` before any write. */
function storedSettings() {
  const file = workerSettingsFile(configHome);
  if (!fs.existsSync(file)) return {};
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

/** Wait until the app has written what `predicate` expects. */
async function waitForStored(predicate, description) {
  await browser.waitUntil(async () => predicate(storedSettings()), {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `the settings file never recorded ${description}; read ${JSON.stringify(storedSettings())}`,
  });
}

/** Reload the webview, which re-runs the start-up check, and wait for the start screen. */
async function restart() {
  await browser.execute(() => window.location.reload());
  await $(START_CREATE).waitForExist({ timeout: BOOT_TIMEOUT });
}

/** True when `selector` never shows up within {@link ABSENCE_WINDOW}. */
async function staysAbsent(selector) {
  try {
    await $(selector).waitForExist({ timeout: ABSENCE_WINDOW });
    return false;
  } catch {
    return true;
  }
}

/**
 * Whether the shell carries the `inert` attribute. Read in the page, not through
 * WebDriver's `getAttribute`: WebKitWebDriver answers a boolean attribute with its
 * property as a string, so a live shell reads `"false"` rather than `null`.
 */
async function shellInert() {
  return browser.execute(
    (selector) => document.querySelector(selector).hasAttribute('inert'),
    SHELL,
  );
}

describe('the update-check consent prompt', () => {
  it('asks on a first start, with focus inside it and the shell inert behind it', async () => {
    await $(PROMPT).waitForExist({ timeout: BOOT_TIMEOUT });
    await $(START_CREATE).waitForExist({ timeout: BOOT_TIMEOUT });

    // The attribute, not `isClickable`: WebDriver's clickability check does not
    // see `inert`, and a click into an inert subtree is swallowed in silence.
    expect(await shellInert()).toBe(true);
    await browser.waitUntil(
      async () =>
        browser.execute((selector) => {
          const dialog = document.querySelector(selector);
          return dialog !== null && dialog.contains(document.activeElement);
        }, PROMPT),
      { timeout: STEP_TIMEOUT, timeoutMsg: 'focus should move into the consent prompt' },
    );
  });

  it('records "Don\'t check", and a later start neither asks nor checks', async () => {
    await $(DECLINE).click();
    await $(PROMPT).waitForExist({ reverse: true, timeout: STEP_TIMEOUT });
    expect(await shellInert()).toBe(false);
    await waitForStored(
      (stored) =>
        stored.update_check === false && typeof stored.update_check_asked_version === 'string',
      'the declined answer',
    );

    await restart();
    expect(await staysAbsent(PROMPT)).toBe(true);
    expect(await staysAbsent(NOTICE)).toBe(true);
  });

  it('asks again once a newer version starts, and enabling shows the newer release', async () => {
    // Pretend the answer was given by an older version. `mergeWorkerSettings`
    // refuses any directory outside the repo's `tmp/`, so this can never reach
    // the developer's real settings.
    mergeWorkerSettings(configHome, { update_check_asked_version: '0.0.1' });

    await restart();
    await $(PROMPT).waitForExist({ timeout: STEP_TIMEOUT });
    await $(ENABLE).click();
    await $(PROMPT).waitForExist({ reverse: true, timeout: STEP_TIMEOUT });
    await waitForStored(
      (stored) => stored.update_check === true && stored.update_check_asked_version !== '0.0.1',
      'the enabled answer for the running version',
    );

    // `textContent` rather than `getText`, which reads empty for clipped text.
    const notice = await $(NOTICE);
    await notice.waitForExist({ timeout: STEP_TIMEOUT });
    expect(clean(await notice.getProperty('textContent'))).toContain('99.0.0');

    await $(DISMISS).click();
    await notice.waitForExist({ reverse: true, timeout: STEP_TIMEOUT });
  });

  it('checks again at the next start without asking, and Settings shows it enabled', async () => {
    await restart();
    expect(await staysAbsent(PROMPT)).toBe(true);
    // Dismissal lasts one session only.
    await $(NOTICE).waitForExist({ timeout: STEP_TIMEOUT });

    await openSettings();
    expect(await $(TOGGLE).isSelected()).toBe(true);
    await closeSettings();
  });
});
