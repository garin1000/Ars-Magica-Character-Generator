// End-to-end: quitting with unsaved changes while an update check hangs (M6.6).
//
// The companion of `update-check-stall.e2e.js`, which proves the app stays usable
// and a clean quit is prompt while the check hangs. Here the document is dirty: the
// unsaved-changes guard must still hold the quit, a hanging check notwithstanding,
// and the check must still land behind the guard's dialog once the hold ends.
//
// `wdio.conf.js` launches this spec with the check enabled, the newer-release feed,
// and a hold shorter than its sibling's (`SPEC_UPDATE_SETUPS`), since this file
// waits it out. A file of its own because its one describe is terminal as well:
// with unsaved edits the guard raises a native GTK dialog WebDriver cannot dismiss,
// left open for the rest of this app instance (see `grog-wizard-aging.e2e.js`'s
// `RunEvent::ExitRequested bridge — unsaved changes`, which this mirrors).

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { workerConfigHome, workerSettingsFile } from '../driver.js';
import { clean, startCharacter, STEP_TIMEOUT } from '../helpers.js';
import { SPEC_UPDATE_SETUPS } from '../wdio.conf.js';

const HOLD_MS = SPEC_UPDATE_SETUPS['update-check-stall-dirty-quit.e2e.js'].stallMs;

const NOTICE = '[data-testid="update-notice"]';
const NAME_INPUT = '[data-testid="identity-name"]';
const TAB_BAR = '[role="tablist"]';
const EDITED_NAME = 'Quit during a hanging check';

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

// MUST BE THE LAST (AND ONLY) DESCRIBE IN THIS FILE: the guard's native dialog stays
// open for the rest of this app instance. Its two `it`s continue one narrative.
describe('quitting a dirty document while the check hangs', () => {
  it('keeps the app, and the unsaved edit, behind the guard', async () => {
    expect(checkEnabledOnDisk()).toBe(true);
    await startCharacter('companion');
    await $(NAME_INPUT).setValue(EDITED_NAME);
    // The quit below is requested while the check is still in flight.
    expect(await $(NOTICE).isExisting()).toBe(false);

    await browser.execute(() => window.__TAURI_INTERNALS__.invoke('request_exit'));

    await browser.waitUntil(async () => (await $(NAME_INPUT).getValue()) === EDITED_NAME, {
      timeout: 5000,
      interval: 250,
      timeoutMsg:
        'the edit did not survive request_exit with unsaved changes — the app may have quit',
    });
    expect(await $(TAB_BAR).isExisting()).toBe(true);
  });

  it('still shows the newer release once the hold ends, behind the open dialog', async () => {
    // `textContent` rather than `getText`, which reads empty for clipped text.
    const notice = await $(NOTICE);
    await notice.waitForExist({ timeout: HOLD_MS + STEP_TIMEOUT });
    expect(clean(await notice.getProperty('textContent'))).toContain('99.0.0');
    // The process outlived the whole hold, so the guard really held the quit.
    expect(await $(NAME_INPUT).getValue()).toBe(EDITED_NAME);
  });
});
