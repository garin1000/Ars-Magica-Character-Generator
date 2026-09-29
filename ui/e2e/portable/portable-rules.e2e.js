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
// SLICE 12 ADDS A SECOND RESOLVED PATH. The settings file
// (`crates/arm-app/src/settings.rs`) is read at launch and written on demand, so it
// is a path resolution of its own — and this layout is precisely where the project
// has been bitten before. It does NOT use `BaseDirectory::Resource`: the per-user
// config directory is computed from the environment rather than from where the bundle
// was installed, so it resolves the same here as in `target/release`. That is a claim
// worth checking rather than asserting in a comment, so the second test below proves
// the READ half of it in this layout.
//
// C8 CHANGED WHICH SETTING THAT IS. The saga year itself is document state now — it
// travels in the save, because a storyguide runs more than one saga and one
// machine-global number was wrong for all but one of them. The key still in the file
// is `default_saga_year`, the year a NEW document starts at.
//
// P2 (`docs/open-todos.md`, U2) retired the Settings dialog's header button
// outright, which is what this test used to drive the whole round trip
// through (see the second `it` below for why no UI path replaces it). It now
// plants a settings file directly and proves the launch-time READ finds it in
// this layout; the WRITE half is a Rust unit-test claim now, not a UI one —
// see that `it`'s own comment for the citations.
//
//   cd ui && npm run test:e2e:portable
//
// NOTE: `wdio.portable.conf.js` builds the release binary and stages it; there is
// no separate setup step.

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { workerConfigHome } from '../driver.js';

const BOOT_TIMEOUT = 30000;
const STEP_TIMEOUT = 10000;

const dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dirname, '../../..');

/**
 * The settings file path THIS run's Linux binary resolves — derived, not
 * guessed, by reading the same inputs the binary itself does rather than
 * retyping a path that could silently drift from them.
 *
 * `wdio.portable.conf.js`'s `beforeSession` spawns the app with
 * `XDG_CONFIG_HOME: workerConfigHome(repoRoot, process.env)` (`driver.js`) —
 * a per-worker directory under this repo's `tmp/`, NOT the developer's real
 * `~/.config`, so two concurrent workers (or a worker and a developer's own
 * machine) never race on the same file. This helper calls the exact same
 * function with the exact same `repoRoot`, so it resolves to the exact same
 * directory the app was actually launched against — never the real per-user
 * config dir. `workerConfigHome` reads `process.env.WDIO_WORKER_ID` (via
 * `workerSuffix`) to pick that directory, and this can never disagree with
 * what `beforeSession` read: `@wdio/local-runner` sets `WDIO_WORKER_ID` once,
 * at worker-process spawn, in the child's env (`runnerEnv` in
 * `@wdio/local-runner/build/index.js::startProcess`), and both `beforeSession`
 * and this spec body execute inside that same forked worker process for the
 * whole run.
 *
 * `crates/arm-app/src/commands.rs::settings_candidates` asks Tauri for
 * `app.path().app_config_dir()` first (the config-dir candidate always wins
 * when present — `settings::pick_settings_file` takes the first candidate
 * that exists, and `store_settings` tries existing candidates before fresh
 * ones). Tauri resolves that to `dirs::config_dir().join(identifier)`
 * (tauri-2.11.3/src/path/desktop.rs:238-242), where `identifier` is read out
 * of `tauri.conf.json` here rather than copied as a literal. On Linux
 * `dirs::config_dir()` honours an absolute `$XDG_CONFIG_HOME`
 * (dirs-6.0.0/src/lin.rs:9) — `workerConfigHome` always returns such an
 * absolute, already-created path, which is exactly what reproduces the
 * binary's own resolution here.
 */
function resolveSettingsPath() {
  const conf = JSON.parse(
    fs.readFileSync(path.join(repoRoot, 'crates/arm-app/tauri.conf.json'), 'utf-8'),
  );
  const configDir = workerConfigHome(repoRoot, process.env);
  return path.join(configDir, conf.identifier, 'settings.json');
}

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

  it('resolves its settings file here too, so a launch-time read finds it (Slice 12, C8)', async () => {
    // P2 (`docs/open-todos.md`, U2, 2026-09-13) retired the Settings dialog's
    // header button outright, which is what this test used to drive a whole
    // read-then-write-then-reread round trip through — the ONLY affordance
    // that worked in this build in the first place, because it is ordinary
    // shipped code: this run builds WITHOUT `e2e-testing`
    // (`wdio.portable.conf.js`), so C6's `activate_menu_item` seam is
    // registered but inert, and C7 moved every document chord onto the native
    // menu's own accelerator, which WebDriver cannot press either (GTK
    // dispatches it above the webview). P2 stands — no Settings affordance
    // comes back for this run's sake — so this is a verification-strategy
    // change, not a workaround: plant the settings file directly (this
    // spec's own `resolveSettingsPath`, which reads the exact same
    // worker-isolated `XDG_CONFIG_HOME` — `workerConfigHome`, `driver.js` —
    // that `wdio.portable.conf.js`'s `beforeSession` handed this run's app,
    // never the developer's real `~/.config`), reload, and prove the
    // launch-time READ finds it in THIS layout by creating a document and
    // reading the saga year it was stamped with — the one thing only a real
    // binary in this exact layout can prove.
    //
    // The WRITE half (`store_settings`/`write_settings` finding a writable
    // candidate, replacing rather than truncating the file, preserving every
    // other key) is not retested here — it was always a Rust-level claim
    // wearing a UI costume, and the actual unit coverage already exists:
    // `settings.rs::storing_writes_to_the_file_that_already_exists`,
    // `settings.rs::storing_falls_through_to_the_next_candidate_when_one_cannot_be_written`,
    // `settings.rs::writing_creates_the_settings_directory_on_first_run`, and
    // `settings.rs::candidate_paths_are_offered_in_order_without_touching_the_disk`
    // for the candidate list itself. The one thing NONE of those cover is
    // `commands.rs::settings_candidates`'s own call to the live
    // `app.path().app_config_dir()` — that needs a real Tauri app handle,
    // which is exactly what this e2e run has and a `cargo test` does not —
    // so the READ assertion below is this claim's only witness, and is kept.
    //
    // Nothing here dirties or discards anything: creating a character is not
    // a destructive action, and this test writes no save file.
    const settingsPath = resolveSettingsPath();

    // Belt-and-suspenders against ever writing outside this repo again — the
    // bug this spec's own history had: an earlier version of
    // `resolveSettingsPath` derived the developer's REAL `~/.config` instead
    // of the worker-isolated directory `wdio.portable.conf.js` actually
    // launches the app with, and planted a settings file there. Refuse before
    // touching disk if that ever regresses.
    const tmpRoot = path.resolve(repoRoot, 'tmp') + path.sep;
    if (!settingsPath.startsWith(tmpRoot)) {
      throw new Error(
        `refusing to write settings outside ${tmpRoot}: resolved path was ${settingsPath}`,
      );
    }

    const backup = fs.existsSync(settingsPath) ? fs.readFileSync(settingsPath) : null;
    try {
      fs.mkdirSync(path.dirname(settingsPath), { recursive: true });
      fs.writeFileSync(settingsPath, `${JSON.stringify({ default_saga_year: 1231 })}\n`);

      await browser.execute(() => window.location.reload());
      await $('[data-testid="start-screen"]').waitForExist({ timeout: BOOT_TIMEOUT });
      const create = await $('[data-testid="start-create-magus"]');
      await create.waitForExist({ timeout: BOOT_TIMEOUT });
      await create.click();

      // `details` is the editor's default tab (`App.svelte`), but this clicks
      // it explicitly rather than relying on that, the same way every other
      // spec in this suite names the tab it wants.
      const detailsTab = await $('[data-testid="tab-details"]');
      await detailsTab.waitForExist({ timeout: STEP_TIMEOUT });
      await detailsTab.click();

      const sagaYear = await $('[data-testid="saga-year-input"]');
      await browser.waitUntil(async () => (await sagaYear.getValue()) === '1231', {
        timeout: STEP_TIMEOUT,
        timeoutMsg: `the new document's saga year did not pick up the planted setting; read ${await sagaYear.getValue()}`,
      });
      expect(await $('[data-testid="error"]').isExisting()).toBe(false);
    } finally {
      // Put this worker's isolated settings file back exactly as found — a
      // rerun in the same `tmp/e2e-config*` directory should not see a stale
      // planted year from a previous run — or remove the one this test
      // created if there was none, so a failure never leaves it behind.
      if (backup === null) {
        fs.rmSync(settingsPath, { force: true });
      } else {
        fs.writeFileSync(settingsPath, backup);
      }
    }
  });
});
