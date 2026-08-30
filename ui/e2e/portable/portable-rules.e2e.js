// End-to-end: the app boots and finds its rules in a PORTABLE layout (6b8d).
//
// The ordinary suite runs the binary from `target/release`, where Tauri takes it
// for a dev build and `BaseDirectory::Resource` already resolves to the exe's own
// directory — so `load_ruleset`'s second candidate, `./rules` beside the
// executable, is never reached. Outside `target/` (a USB stick, the archives
// `build-linux.sh`/`build-win.sh` produce) Resource resolves to `/usr/lib/<name>`,
// which does not exist, and that fallback is the only reason the app starts at
// all. This spec runs the same production binary from `tmp/portable/` and proves
// it.
//
// A smoke check on purpose: everything the app can do once its ruleset is loaded
// is covered by the specs in `specs/`, and re-running any of it here would say
// nothing about resolution. What is asserted is exactly the thing the layout puts
// at risk — that the rules were found, and that both halves of them arrived.
//
// SLICE 12 ADDS A SECOND RESOLVED PATH. The saga-year settings file
// (`crates/arm-app/src/settings.rs`) is read at launch and written on demand, so it
// is a path resolution of its own — and this layout is precisely where the project
// has been bitten before. It does NOT use `BaseDirectory::Resource`: the per-user
// config directory is computed from the environment rather than from where the bundle
// was installed, so it resolves the same here as in `target/release`. That is a claim
// worth checking rather than asserting in a comment, so the second test below reads
// the setting back in this layout.
//
//   cd ui && npm run test:e2e:portable
//
// NOTE: `wdio.portable.conf.js` builds the release binary and stages it; there is
// no separate setup step.

import { $, browser, expect } from '@wdio/globals';

const BOOT_TIMEOUT = 30000;
const STEP_TIMEOUT = 10000;

describe('portable layout', () => {
  it('boots outside target/ and loads its ruleset from beside the executable', async () => {
    // The startup screen renders one entry per character-type PROFILE, so a create
    // button existing at all means `rules/core/character_types.json` was found,
    // parsed, and passed the load-time referential-integrity check — which reads
    // every other file in `rules/core` too.
    const create = await $('[data-testid="start-create-magus"]');
    await create.waitForExist({ timeout: BOOT_TIMEOUT });

    // A failed load reports itself here rather than silently showing an empty
    // screen, so the absence of this banner is a claim, not a coincidence.
    expect(await $('[data-testid="start-error"]').isExisting()).toBe(false);

    // The other half of the rules — `rules/i18n/<lang>` — is what turns an id into
    // a name. Open a character and read one catalogue entry: it must exist, and it
    // must not be rendering its own slug back, which is what an unresolved
    // translation would look like.
    await create.click();
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: STEP_TIMEOUT });
    await vfTab.click();

    const virtue = await $('[data-testid="add-virtue.keen_vision"]');
    await virtue.waitForExist({ timeout: STEP_TIMEOUT });
    const label = await virtue.getText();
    expect(label.length).toBeGreaterThan(0);
    expect(label).not.toContain('virtue.keen_vision');

    expect(await $('[data-testid="error"]').isExisting()).toBe(false);
  });

  it('resolves its settings file here too, so the saga year survives (Slice 12)', async () => {
    // The wizard's Concept step is the only surface the setting is on. Reaching it
    // through the start screen keeps this independent of the test above.
    await $('[data-testid="new-button"]').click();
    const start = await $('[data-testid="start-wizard-grog"]');
    await start.waitForExist({ timeout: BOOT_TIMEOUT });
    await start.click();

    const sagaYear = await $('[data-testid="saga-year-input"]');
    await sagaYear.waitForExist({ timeout: BOOT_TIMEOUT });

    // The field renders only once `saga_year` has answered, so its presence already
    // means the settings path resolved and the read did not fail the launch. The value
    // is whatever this machine has stored — the DEFAULT if there is no file at all —
    // so what is asserted is that it is a year, never a specific one.
    const initial = await sagaYear.getValue();
    expect(Number(initial)).toBeGreaterThan(0);

    // And the write half: `set_saga_year` has to find a writable candidate from here
    // as well. Reading it back after a reload re-runs the launch-time read against
    // whatever the write produced.
    await sagaYear.setValue('1231');
    await browser.waitUntil(async () => (await sagaYear.getValue()) === '1231', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the typed saga year did not stay in the field',
    });
    expect(await $('[data-testid="error"]').isExisting()).toBe(false);

    await browser.execute(() => window.location.reload());
    await $('[data-testid="start-screen"]').waitForExist({ timeout: BOOT_TIMEOUT });
    const again = await $('[data-testid="start-wizard-grog"]');
    await again.waitForExist({ timeout: BOOT_TIMEOUT });
    await again.click();
    const reloaded = await $('[data-testid="saga-year-input"]');
    await reloaded.waitForExist({ timeout: BOOT_TIMEOUT });
    await browser.waitUntil(async () => (await reloaded.getValue()) === '1231', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `the saga year did not survive a relaunch in the portable layout; read ${await reloaded.getValue()}`,
    });

    // Put this machine's setting back where it was found, exactly as `saga-year.e2e.js`
    // does: it is real persisted state, not a fixture.
    await reloaded.setValue(initial);
    await browser.waitUntil(async () => (await reloaded.getValue()) === initial, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'restoring the saga year found on this machine',
    });
  });
});
