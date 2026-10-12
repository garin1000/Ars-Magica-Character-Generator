// End-to-end: a malformed update-feed response through the real binary (M6.6).
//
// CLAUDE.md's guarantee is that the release tag is parsed as strict semver and
// that a parse failure stays silent — no notice, no crash, no stuck state.
// `update_check.rs`'s unit tests already prove `parse_release_tag`/
// `parse_latest_release` reject a non-semver `tag_name`, but nothing end-to-end
// proved the real IPC + UI path degrades the same way through the shipped
// binary. `wdio.conf.js` launches this spec with "Check" already answered for
// the running version (so no consent prompt stands in the way) and
// `ARM_E2E_UPDATE_FEED` pointing at `fixtures/update-feed-invalid.json`
// (`tag_name: "not-a-version"`) instead of the well-formed
// `update-feed-newer.json` every other update-check spec uses
// (`SPEC_UPDATE_SETUPS`).

import { $, expect } from '@wdio/globals';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { workerConfigHome, workerSettingsFile } from '../driver.js';
import { startCharacter } from '../helpers.js';

const NOTICE = '[data-testid="update-notice"]';

// How long a "does not appear" claim waits. The check holds no artificial delay
// here (no `stallMs` in this spec's setup), so by the time the app is up the
// check has already resolved — this window is slack for the one IPC round trip.
const ABSENCE_WINDOW = 3000;

/** The app's settings file as it stands on disk. */
function storedSettings() {
  const configHome = workerConfigHome(
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..'),
    process.env,
  );
  const file = workerSettingsFile(configHome);
  if (!fs.existsSync(file)) return {};
  return JSON.parse(fs.readFileSync(file, 'utf8'));
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

describe('the update check against a malformed feed', () => {
  it('never shows a notice, and the app stays usable', async () => {
    // Confirms this worker really launched with the check enabled, i.e. the
    // check actually ran against the invalid feed rather than being skipped.
    expect(storedSettings().update_check).toBe(true);

    // The notice never appears, not even transiently: `check_for_update`
    // returns `None` for a tag strict semver rejects, same as no newer release.
    expect(await staysAbsent(NOTICE)).toBe(true);

    // The app itself is unaffected by the failed check — it still boots and
    // can build a character, and the stored answer survived unchanged.
    await startCharacter('grog');
    expect(await $(NOTICE).isExisting()).toBe(false);
    expect(storedSettings().update_check).toBe(true);
  });
});
