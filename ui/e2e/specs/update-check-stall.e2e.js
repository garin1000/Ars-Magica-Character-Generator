// End-to-end: the app while an update check hangs (M6.6).
//
// The check is meant to stay off the user's way: `AppStore.init` fires it without
// awaiting it (`ui/src/lib/state.svelte.ts`), and the backend runs the blocking
// fetch on the blocking pool (`crates/arm-app/src/commands.rs::check_for_update`).
// This file proves, against the real release binary, what that buys: a check that
// hangs holds up neither work in the app nor a quit.
//
// `wdio.conf.js` launches this spec with the check already enabled for the running
// version (so no prompt), the newer-release feed, and a hold
// (`SPEC_UPDATE_SETUPS`): the `e2e-testing` build waits that long before it reads
// the feed (`commands.rs::e2e_update_stall`), standing in for a network that drops
// packets until the timeout. The hold is far longer than the shipped 10 s timeout,
// so the notice cannot show while this file runs, and its absence at a given moment
// is the proof that the check is still in flight then.
//
// `quitting a clean document while the check hangs` MUST stay last: it ends the app
// process and this worker's WebDriver session.

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { workerConfigHome, workerSettingsFile } from '../driver.js';
import {
  clean,
  closeSettings,
  openSettings,
  runDocumentAction,
  startCharacter,
  STEP_TIMEOUT,
  waitForIdle,
} from '../helpers.js';
import { e2eFile, SPEC_UPDATE_SETUPS } from '../wdio.conf.js';

const HOLD_MS = SPEC_UPDATE_SETUPS['update-check-stall.e2e.js'].stallMs;

const NOTICE = '[data-testid="update-notice"]';
const TOGGLE = '[data-testid="update-check-toggle"]';
const STA_INC = '[data-testid="char-inc-sta"]';
const STA_VALUE = '[data-testid="char-value-sta"]';
const SOAK = '[data-testid="derived-soak"]';

// How long a clean quit may take: a small fraction of the 60 s hold, so a quit
// that waited for the check would blow it, and clear of the shipped 10 s timeout.
const QUIT_BUDGET_MS = 5000;

/** Whether this launch really runs the check: "Check" on file for this worker. */
function checkEnabledOnDisk() {
  const configHome = workerConfigHome(
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..'),
    process.env,
  );
  const file = workerSettingsFile(configHome);
  if (!fs.existsSync(file)) return false;
  return JSON.parse(fs.readFileSync(file, 'utf8')).update_check === true;
}

/** Whether the newer-release notice is on screen, i.e. the check has answered. */
async function noticeShown() {
  return $(NOTICE).isExisting();
}

async function clickTab(id) {
  const tab = await $(`[data-testid="tab-${id}"]`);
  await tab.waitForExist({ timeout: STEP_TIMEOUT });
  await tab.click();
}

/** The Soak total on the Totals tab, as a number. */
async function soakOnTotalsTab() {
  await clickTab('totals');
  const soak = await $(SOAK);
  await soak.waitForExist({ timeout: STEP_TIMEOUT });
  return Number(clean(await soak.getText()));
}

/** Save through the ARM_E2E_FILE seam and wait until the app is idle again. */
async function save() {
  if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
  await runDocumentAction('save');
  await browser.waitUntil(() => fs.existsSync(e2eFile), {
    timeout: STEP_TIMEOUT,
    timeoutMsg: 'save did not write the file',
  });
  await waitForIdle();
}

describe('using the app while the check hangs', () => {
  it('creates, edits, recomputes, saves and opens Settings before the check answers', async () => {
    expect(checkEnabledOnDisk()).toBe(true);
    const started = Date.now();
    expect(await noticeShown()).toBe(false);

    await startCharacter('companion');

    // An edit, and a derived value following it: Soak counts Stamina.
    const soakBefore = await soakOnTotalsTab();
    await clickTab('characteristics');
    const staInc = await $(STA_INC);
    await staInc.waitForExist({ timeout: STEP_TIMEOUT });
    await staInc.click();
    await browser.waitUntil(async () => clean(await $(STA_VALUE).getText()) === '+1', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Stamina should read +1 after one increment',
    });
    await browser.waitUntil(async () => (await soakOnTotalsTab()) === soakBefore + 1, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `Soak should follow Stamina from ${soakBefore} to ${soakBefore + 1}`,
    });

    await openSettings();
    expect(await $(TOGGLE).isSelected()).toBe(true);
    await closeSettings();

    await save();

    const elapsed = Date.now() - started;
    expect(elapsed).toBeLessThan(HOLD_MS);
    // Everything above happened while the check was still hanging.
    expect(await noticeShown()).toBe(false);
  });
});

// MUST BE THE LAST DESCRIBE IN THIS FILE: a clean quit ends this app instance and
// its WebDriver session. Drives `request_exit`, the e2e-only IPC seam that runs the
// same guard the real `RunEvent::ExitRequested` handler runs
// (`crates/arm-app/src/main.rs::request_exit`), exactly as `wizard-walks.e2e.js`'s
// `RunEvent::ExitRequested bridge — no unsaved changes` does without a hanging check.
describe('quitting a clean document while the check hangs', () => {
  it('ends the app at once, without waiting for the check', async () => {
    // A fresh character is not dirty, so no confirmation stands in the way.
    await startCharacter('grog');
    expect(await noticeShown()).toBe(false);

    const requested = Date.now();
    // The session can die while `execute` is still in flight, so the call itself
    // rejects; that rejection is the evidence the quit happened.
    let sessionGone = false;
    try {
      await browser.execute(() => window.__TAURI_INTERNALS__.invoke('request_exit'));
    } catch {
      sessionGone = true;
    }
    if (!sessionGone) {
      await browser.waitUntil(
        async () => {
          try {
            await browser.getWindowHandles();
            return false;
          } catch {
            return true;
          }
        },
        {
          timeout: QUIT_BUDGET_MS,
          timeoutMsg: 'request_exit did not quit the app while the check was hanging',
        },
      );
    }
    expect(Date.now() - requested).toBeLessThan(QUIT_BUDGET_MS);
  });
});
