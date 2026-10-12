# End-to-end tests (tauri-driver, real binary)

These drive the **real release binary** through the platform WebDriver, so they
verify true Tauri IPC, the bundled rules resources, and the save/load flow.

## Prerequisites

- `tauri-driver` — `cargo install tauri-driver`
- **Linux:** `WebKitWebDriver` — `sudo apt install webkit2gtk-driver`
  (macOS uses the system `safaridriver`; Windows needs `msedgedriver`.)
- A display — either a desktop session or, on Linux, `xvfb`
  (`sudo apt install xvfb`), which WebdriverIO uses automatically when there
  is none. See [Headful and headless](#headful-and-headless).

## Run

```bash
cd ui
npm run test:e2e            # builds the release binary, then runs the suite
npm run test:e2e:portable   # the portable-layout smoke check (see below)
npm run test:e2e:reuse -- --spec ./e2e/specs/app-shell.e2e.js   # fast local rerun
```

One command, both environments — no `xvfb-run` wrapper.

`wdio.conf.js` builds `arm-app` in release with `--features e2e-testing` into
its **own cargo target dir**, `target/e2e`, copies `rules/` beside
`target/e2e/release/arm-app`, spawns `tauri-driver`, and launches that binary.
The gate's plain `target/release/arm-app` is never overwritten with the test
seams, and each of the two builds stays a cargo no-op while its inputs are
unchanged. The first e2e run builds every dependency once (about 7 min). Native file dialogs can't be automated over WebDriver, so the app
honors the `ARM_E2E_FILE` environment seam (set by the config) to save/load a
fixed temp path instead of opening a dialog. See
`crates/arm-app/src/commands.rs`.

The same is true of the **native menu**, which the OS draws outside the webview:
WebDriver can neither click it nor read it. Two commands stand in, both gated
behind the `e2e-testing` Cargo feature and inert in the shipped binary
(`menu_test_seams_enabled`, `crates/arm-app/src/commands.rs`):

- `activate_menu_item(id)` — presses one of the six items, by firing the very
  `menu::forward_menu_action` the OS handler fires. The availability gate is
  untouched: `store.documentActionEnabled` still decides, so a disabled item
  does nothing.
- `installed_menu()` — reads back what Tauri actually installed
  (`AppHandle::menu()`), not the model that was pushed to it, so a menu that was
  never rebuilt is distinguishable from one that was. Tauri exposes no
  `is_enabled()` on a `PredefinedMenuItem` and no accessor for its role, so
  separators and OS-implemented items are reported by text and position only.

Neither seam exists in the portable run, which builds without the feature.

### What the harness sets in the environment

- **`NODE_ENV=production` for the build.** `@wdio/cli` sets `NODE_ENV=test` in
  the launcher, and Vite keeps an inherited `NODE_ENV`. So both configs' builds
  (`onPrepare`, `stage-portable.js`) force `production`, or the binary would
  embed a development frontend instead of the shipped one.
- **An unreachable session bus for the app.** `driver.js::startWorkerDriver`
  sets `DBUS_SESSION_BUS_ADDRESS` to a socket that does not exist. tao reads the
  colour scheme from the desktop portal with a blocking 5 s call, several times
  per launch. Headless, the user bus's portal never answers, and every session
  stalled ~30 s before its first command. A normal desktop launch is not
  affected (`docs/test-performance-2026-09.md` §3).
- **`ARM_E2E_WORKERS`** sets how many spec files run at once. The default is 6
  (`wdio.shared.conf.js`). Anything but a positive integer is refused.
- **`ARM_E2E_REUSE_BUILD=1`** (or `npm run test:e2e:reuse`, which needs no
  environment prefix) skips the cargo build entirely and prints a loud
  `REUSING` line. That happens only when the binary's size and mtime, the
  content of `crates/**` + `Cargo.toml` + `Cargo.lock`, and `ui/dist` all still
  match what the last successful e2e build recorded, and the frontend is current.
  Otherwise it builds as usual. Anything but `1` or unset is refused. The gate
  never uses it.

### What the build skips

- **`npm run build` leaves an unchanged `ui/dist` untouched.** Vite writes into
  `tmp/vite-dist`, and `sync-dist.js` mirrors only the changed files into
  `ui/dist` and deletes stale ones. arm-app embeds `ui/dist`, so new mtimes
  alone used to force a ~40 s recompile on every build.
- **The frontend stamp.** `build-freshness.js` (`begin`/`commit` around the
  build, stamps in `tmp/build-stamps/`) records content hashes of Vite's inputs
  and of `ui/dist`. It writes them only for a production build, and only if no
  input changed while the build ran. While they still match, the e2e build skips
  its frontend build.
- **One `--config` per target dir.** tauri-cli hands `--config` to cargo as
  `TAURI_CONFIG`, and tauri-build reruns whenever that changes. So the e2e build
  always passes the same override, removing `beforeBuildCommand` and running
  `npm run build` itself when needed. The portable build runs exactly the gate's
  `cargo tauri build --no-bundle`, whose `target/release` it shares.

### The update-check consent prompt (M6.6)

On a first start the app asks whether to check GitHub for updates, and that
modal makes the whole shell `inert`. So both configs' `beforeSession` seed
every worker's settings file with a "Don't check" answer for the running
version (`driver.js`'s `seedUpdateCheckAnswer`), which also keeps the portable
binary offline. The exceptions are the update-check specs, listed in
`wdio.conf.js`'s `SPEC_UPDATE_SETUPS`. All of them get `ARM_E2E_UPDATE_FEED`, a
file (`fixtures/update-feed-newer.json`) that the `e2e-testing` build reads
instead of the network. `update-check.e2e.js` gets an emptied config home for a
genuine first start. The two stall specs get "Check" already answered plus
`ARM_E2E_UPDATE_STALL_MS`, a hold before the feed is read that stands in for a
hanging network. A spec that writes settings itself must merge into the file,
never replace it.

### Keyboard accelerators cannot be pressed from a spec (C7)

Since C7 every document shortcut is declared as its menu item's **accelerator**
and nowhere else; `App.svelte` carries no keydown handler for any of them. The
accelerator is dispatched above the webview — GTK matches the toplevel window's
accel group in `gtk_window_key_press_event`, before the focused widget sees the
key — and **WebDriver cannot reach that**: `browser.keys` goes to
WebKitWebDriver, which feeds a synthesized key into WebKit's own input pipeline
rather than delivering it as an event on the window.

That was measured, not assumed. With the accelerators in place and the webview
handler gone, every chord press was inert (`F10` did not open the menubar
either), while asking GTK directly on the live window —
`gtk_accel_groups_activate(window, GDK_KEY_n, CONTROL_MASK)` — answered `true`,
and the items' `GtkAccelLabel`s reported accelerator widths of 37-76px once a
size request was forced, i.e. the chord really is drawn beside the label. The
shortcuts work; they are simply not drivable from a spec.

Two consequences:

- `helpers.js`'s `runDocumentAction` presses the **menu item** (through
  `activate_menu_item`), which is the same route the accelerator takes once the
  OS has turned the press into an activation.
- The portable spec, which has no seam, uses **no document action at all** — it
  reaches the startup screen with `window.location.reload()`. Building that run
  with `--features e2e-testing` was the alternative and was rejected: the
  portable suite exists to prove the artifact `build-linux.sh` ships works, and
  a binary built with a test feature is not that artifact.

What therefore stays unproven end-to-end is that a _physical_ key press
activates the item. The chord strings themselves are pinned and parse-checked
in `crates/arm-app/tests/menu.rs`, against muda's own parser, because
`MenuItem::with_id` drops an accelerator it cannot parse without a word.

## The portable-layout smoke check

`npm run test:e2e` runs the binary from `target/e2e/release`, where Tauri takes it
for a dev build and `BaseDirectory::Resource` already resolves to the executable's own
directory. A **portable** build — the archives `build-linux.sh` and `build-win.sh`
produce, the folder on a USB stick — sits outside `target/`, where Resource
resolves to `/usr/lib/<name>` instead; there the app only starts because
`load_ruleset` also looks for `./rules` beside the executable
(`crates/arm-app/src/commands.rs`). That fallback is unreachable from the standard
suite, so it has a run of its own:

```bash
cd ui
npm run test:e2e:portable
```

`wdio.portable.conf.js` builds the release binary, then `stage-portable.js` copies
it plus `rules/core`, `rules/i18n` and `rules/NOTICE.md` into the **gitignored**
`tmp/portable/` — mirroring `build-linux.sh`'s layout — and points the driver
there. The single spec in `portable/` asserts what the layout puts at risk and
nothing more: that a character-type profile loaded (so `rules/core` was found and
passed its integrity check), that a catalogue entry renders a name rather than its
own slug (so `rules/i18n` arrived too), and that no error banner is up. Everything
the app does _after_ its ruleset loads is the standard suite's job.

It is deliberately **not** part of `npm run test:e2e`: it needs the staging step,
and its specs live outside `specs/`, where the standard config's list does not
look.

## Headful and headless

Both modes come from the same command; WebdriverIO chooses between them:

| `DISPLAY` at launch | Mode                                                     |
| ------------------- | -------------------------------------------------------- |
| set                 | **headful** — renders on your desktop session, watchable |
| unset               | **headless** — the worker is wrapped in `xvfb-run`       |

`@wdio/local-runner` (via `@wdio/xvfb`) checks `DISPLAY` in the launcher process
and, when it is unset, spawns the test worker as
`xvfb-run --auto-servernum -- node …`, tearing the server down with the worker.
So on a remote/cron session the suite is already headless — no `xvfb-run`
wrapper of your own, no `DISPLAY` juggling in `wdio.conf.js`. Anything a session
hook spawns (`tauri-driver` → `WebKitWebDriver` → the app) inherits that display.

Two consequences worth knowing:

- The decision keys on **`DISPLAY` only**. A Wayland session without XWayland
  exports none, so it takes the Xvfb path too.
- It is decided **before any hook runs in the worker**, so a hook cannot observe
  "no display" and cannot usefully start its own X server.

What WebdriverIO does _not_ do well is a headless box with no xvfb installed: it
logs a warning, proceeds, and the run dies later inside WebDriver with an
unrelated-looking error. `onPrepare` therefore runs `preflightDisplay`
(`display.js`) in the launcher — the last place `DISPLAY` still reflects reality
— and fails immediately with the install hint instead. That check is pure and
unit-tested in `display.test.js`, which runs under `npm run test:unit`.

## Layout

```
e2e/
  wdio.conf.js           # the runner: build, display preflight, tauri-driver, the ARM_E2E_* seams
  wdio.reuse.conf.js     # the standard run with ARM_E2E_REUSE_BUILD on (npm run test:e2e:reuse)
  wdio.portable.conf.js  # the portable-layout run: build, stage outside target/, drive that copy
  stage-portable.js      # stages binary + rules/ into the gitignored tmp/portable/
  build-freshness.js     # frontend stamp + e2e reuse decision; also `npm run build`'s begin/commit
  sync-dist.js           # `npm run build`: mirrors tmp/vite-dist into ui/dist, changed files only
  display.js             # display preflight (pure; unit-tested by display.test.js)
  display.test.js        # runs under `npm run test:unit`, NOT under wdio
  helpers.js             # shared spec harness — `startCharacter()`, `startWizard()`, …
  wizard-walk.js         # shared driver for the four per-type guided walks
  fixtures/              # files the app reads through a seam (the update feed)
  specs/*.e2e.js         # the suite; listed in `wdio.conf.js`'s SPECS_LONGEST_FIRST
  portable/*.e2e.js      # the portable smoke check; not in the standard list
```

Non-spec harness modules live at the `e2e/` root, never under `specs/`. They must
not be named `*.test.js` unless they really are Vitest tests: `ui/vitest.config.ts`
includes `e2e/**/*.test.js` in `npm run test:unit` and excludes only `e2e/specs/`,
and anything importing `@wdio/globals` cannot resolve there. That is why
`display.test.js` is a unit test and `helpers.js` is not.

## Every spec builds its own character

The suite runs several spec files **in parallel** (`maxInstances`,
`wdio.shared.conf.js`), and since M6a the app boots on a startup choice screen
where a character's **type is fixed at creation** — there is no type selector to
switch, and creating a character discards whatever was being edited. So a spec
can never inherit a character from an earlier `it`, and — since each spec file
gets its own worker, its own WebDriver session and so its own app process —
certainly not from an earlier spec file.

`helpers.js` exports the one function that follows from that:

```js
import { startCharacter } from '../helpers.js';

await startCharacter('magus'); // 'grog' | 'companion' | 'mythic_companion' | 'magus' | …
```

It works from any state: it returns to the startup screen (answering the discard
prompt if there are unsaved edits), clicks that type's create button, and waits for
the editor. Call it at the start of every `it` that needs a character — except
where a block deliberately continues the narrative an earlier block in the same
file set up, which a comment should then say out loud.

App-wide state is **not** part of a character and does survive: the UI language and
the validation mode carry over, so a spec that depends on either must set it itself.

## Spec ordering

`wdio.conf.js` lists the spec files explicitly in `SPECS_LONGEST_FIRST`, longest
first by measured duration, so the long ones do not start last and leave
workers idle at the end. wdio hands them out in that order to up to
`maxInstances` workers (`wdio.shared.conf.js`, `ARM_E2E_WORKERS`). Which file
lands on which worker is scheduling detail.

**A new spec file must be added to that list**, at the position its measured
duration earns. `wdio.conf.test.js` fails while the list and the files under
`specs/` differ, so a forgotten entry is caught by `npm run test:unit` rather
than silently skipped. Each worker spawns its own
`tauri-driver` on its own port pair and its own app instance
(`driver.js`'s per-worker isolation, milestone A1), so every spec file gets a
**freshly launched app** in its own WebDriver session regardless of when it
runs or beside which other file. `app-shell.e2e.js`'s `app entry` describe
therefore does not need to sort first — any spec's first line sees a fresh
app — and its own header comment explains why it is instead the first
**describe** of its own file rather than relying on file-name sort order
(established in M6/6b8d, ahead of the A2 spec consolidation).

Do not write a spec that expects to inherit anything at all from the file
before it — not even the app process — and do not assume a particular
interleaving between files running in different workers.

**Within one file, order is not free.** A2 (spec consolidation) merged several
single-purpose files into shared-setup files, each with multiple top-level
`describe`s run in file order by the one worker that owns that file. Two kinds
of ordering constraint recur across those merges:

- **Terminal describes must be last.** `app-shell.e2e.js`'s
  `window.close() bridge — no unsaved changes`,
  `companion-editor.e2e.js`'s `window.close() bridge — unsaved changes`,
  `wizard-walks.e2e.js`'s `RunEvent::ExitRequested bridge — no unsaved changes`
  and `grog-wizard-aging.e2e.js`'s `RunEvent::ExitRequested bridge — unsaved
changes` each end the app process or leave a native dialog WebDriver cannot
  dismiss, so each is the final describe of its file and nothing may run after
  it in the same worker. So are the quit describes of the two update-check
  stall specs, which is why there are two of them.
- **Session-scoped state (language, window size) must be restored**, either by
  the describe's own `after` hook or by being the file's terminal describe, so
  the describes after it are not silently affected. See each merged file's own
  header comment for which describes do which.

## Specs

A2 (spec consolidation) merged 40 of the original 43 single-purpose files into 7
shared-setup files by feature domain, to cut the suite's app-launch count; three
files stayed as they were. Each file's own header comment, and each describe's,
states what it covers and, where the merge left a hazard (session-scoped
language/window state, a terminal describe), why it is ordered the way it is.

- `app-shell.e2e.js` — app-wide chrome: boot entry (M6a — the startup screen,
  creating a character, New's discard guard, Open loading a saved file's own
  type), the header's document-status readout, German localization of the
  chrome, the native menu (C6 — activating an item really runs the action, a
  disabled one does not, and a language switch rebuilds the _installed_ menu in
  German), the settings dialog, and the `window.close()` bridge for a clean
  document (terminal).
- `companion-editor.e2e.js` — the companion-shaped editor surface: Characteristic
  caps and Ability bonuses, illegal-selection reporting and its per-`ValidationMode`
  variants, mechanical Virtue/Flaw effects, mutual exclusion, warping-owed fills,
  the core tabbed edit/save/reload flow, per-character fields, and the
  `window.close()` bridge for a dirty document (terminal).
- `magus-editor.e2e.js` — editor surfaces: the selected-list frame's scroll
  containment, the Totals tab's duplicate-key regression guard, repeated
  parameterized Virtues, the max_total copy cap, Mythic Companion types, and
  per-type profile budgets/category rules.
- `magus-arts-houses.e2e.js` — a magus's Hermetic Arts and Hermetic Houses.
  Split out of `magus-editor.e2e.js` (2026-10-11), which had become the one
  spec every other worker waited on.
- `magus-possessions.e2e.js` — a magus's possessions: Spells, the familiar
  statblock, the talisman item, the Longevity Ritual, and the Markdown
  character-sheet export.
- `wizard-walks.e2e.js` — the four per-type guided-wizard walks (grog, companion,
  mythic companion, magus), each driven from the startup screen to Finish with
  **every** declared phase filled in, then asserted complete (no phase left
  marked untouched) and legal (no error-severity finding) and saved/reloaded
  unchanged — plus the `RunEvent::ExitRequested` bridge for a clean document
  (terminal). The phase list comes from `rules/core/character_types.json` and
  the shared driving from `wizard-walk.js`, so a profile that gains a phase
  makes all four walks visit it.
- `grog-wizard-aging.e2e.js` — the guided aging step and its crisis branch (both
  on a grog, the shortest rail), the saga-year setting the aging arithmetic is
  measured against, and the `RunEvent::ExitRequested` bridge for a dirty
  document (terminal).
- `magus-apprenticeship.e2e.js` — building a magus through its life stages:
  guided funding, the Gauntlet-age floor, the life-stage XP chips, the Hermetic
  minimums checklist, and the sticky XP bar at a short window height.
- `magus-post-gauntlet.e2e.js` — a magus built past its Gauntlet.
- `life-stage-childhood.e2e.js` — guided funding for a non-magus's childhood.
- `wizard-flow.e2e.js` — the guided wizard's own machinery: rail order and
  gating on a magus, resuming a saved mid-flow document, what the flow tells
  the player (guidance text, the Hermetic-minimums disclosure, unspent-budget
  warnings), layout stability under a step's first interaction, and the tab
  area at a short window height.
- `d81-features.e2e.js` — the D81 rules features through the release binary:
  the requisite fold in a Casting Total, "Add within focus" and the stale-mark
  warning, Incompatible Arts' four selects and "Unusable" cell, a
  Characteristic-floor prerequisite, and Merinita's conditional Warping Point.
- `save-load-fidelity.e2e.js` — save → New → Open round trips: the D81
  parameters (Incompatible Arts, a Magical Focus, a spell marked within focus),
  a Mythic Companion's type choices, an illegal companion kept in Silent mode,
  and the German names on screen plus a German companion export.
- `update-check.e2e.js` — the update-check consent prompt on a first start, a
  decline, being asked again by a newer version, enabling it, the notice for a
  newer release, and the Settings toggle (M6.6).
- `update-check-stall.e2e.js` — while a check hangs (a 60 s hold), real work
  completes before the check answers, and a clean quit ends the app within 5 s
  (terminal).
- `update-check-stall-dirty-quit.e2e.js` — while a check hangs (a 20 s hold),
  the unsaved-changes guard still holds a quit, and the notice still arrives
  behind the guard's dialog (terminal).
- `update-check-invalid-feed.e2e.js` — a malformed feed (`tag_name` that fails
  strict semver) never shows a notice, not even transiently, and the app stays
  usable and keeps the stored consent answer.

Every spec requires the same display + release-build prerequisites: none of them
can run without `WebKitWebDriver` and a display.

> The shipped sample ruleset (`rules/core/`) has too few selectable virtues to
> exceed the companion's 10-point budget through clicks alone, so the
> error-display spec uses a forbidden-category flaw (same render path) rather
> than an over-budget case. Over-budget rendering is covered at the engine level
> by `crates/arm-app/tests/commands.rs::over_budget_virtues_is_reported`.

Pure display helpers (`src/lib/derive.ts`) are unit-tested with Vitest, separate
from this webview-gated suite: `cd ui && npm run test:unit`.

## Status

The harness is complete but is **gated on `WebKitWebDriver`**, a system package
that must be installed separately (it is not bundled) — plus `Xvfb` where there
is no desktop session. Command logic and the canonical save/load round-trip are
independently covered by the Rust integration tests in
`crates/arm-app/tests/commands.rs`, which need no webview.
