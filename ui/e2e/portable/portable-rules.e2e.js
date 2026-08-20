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
// is covered by the 32 specs in `specs/`, and re-running any of it here would say
// nothing about resolution. What is asserted is exactly the thing the layout puts
// at risk — that the rules were found, and that both halves of them arrived.
//
//   cd ui && npm run test:e2e:portable
//
// NOTE: `wdio.portable.conf.js` builds the release binary and stages it; there is
// no separate setup step.

import { $, expect } from '@wdio/globals';

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
});
