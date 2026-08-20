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
```

One command, both environments — no `xvfb-run` wrapper.

`wdio.conf.js` builds `arm-app` in release, spawns `tauri-driver`, and launches
the app. Native file dialogs can't be automated over WebDriver, so the app
honors the `ARM_E2E_FILE` environment seam (set by the config) to save/load a
fixed temp path instead of opening a dialog. See
`crates/arm-app/src/commands.rs`.

## The portable-layout smoke check

`npm run test:e2e` runs the binary from `target/release`, where Tauri takes it for
a dev build and `BaseDirectory::Resource` already resolves to the executable's own
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
and its specs live outside `specs/` so the standard config's glob cannot reach
them.

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
  wdio.portable.conf.js  # the portable-layout run: build, stage outside target/, drive that copy
  stage-portable.js      # stages binary + rules/ into the gitignored tmp/portable/
  display.js             # display preflight (pure; unit-tested by display.test.js)
  display.test.js        # runs under `npm run test:unit`, NOT under wdio
  helpers.js             # shared spec harness — `startCharacter()`, `startWizard()`, …
  wizard-walk.js         # shared driver for the four per-type guided walks
  specs/*.e2e.js         # the suite; wdio globs `specs/**/*.e2e.js`
  portable/*.e2e.js      # the portable smoke check; the standard glob cannot reach it
```

Non-spec harness modules live at the `e2e/` root, never under `specs/`. They must
not be named `*.test.js` unless they really are Vitest tests: `ui/vitest.config.ts`
includes `e2e/**/*.test.js` in `npm run test:unit` and excludes only `e2e/specs/`,
and anything importing `@wdio/globals` cannot resolve there. That is why
`display.test.js` is a unit test and `helpers.js` is not.

## Every spec builds its own character

The suite runs **serially**, one spec file at a time, and since M6a the app boots on a
startup choice screen where a character's **type is fixed at creation** — there is no
type selector to switch, and creating a character discards whatever was being edited.
So a spec can never inherit a character from an earlier `it`, and — since each spec
file gets its own session and so its own app process — certainly not from an earlier
spec file.

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

wdio globs `specs/**/*.e2e.js` and runs the files in sorted order, one at a time
(`maxInstances: 1`). `app-entry.e2e.js` was named to sort first, so that it — and only
it — could assert the app's **boot** state.

**Neither half of that is true any more, and it does not matter** (established in
M6/6b8d):

- It no longer sorts first. `aging-crisis.e2e.js` and `aging.e2e.js`, added in
  6b6/6b7, sort ahead of it (`ag` before `ap`), so it runs **third**.
- It does not need to. wdio spawns a **worker per spec file**, and each worker opens
  its own WebDriver session — so every spec file gets a **freshly launched app**.
  (Visible in the reporter as `#0-0 … #0-34`, one `Session ID` per file.) The boot
  state is therefore not one file's privilege; any spec's first line sees a fresh app.

So do not rename a file believing the order protects an assertion, and do not write a
spec that expects to inherit anything at all from the file before it — not even the
app process.

## Specs

- `app-entry.e2e.js` — M6a entry: the app boots on the startup screen; creating a
  magus lands in the editor with its mandatory free traits and a read-only type
  label; New returns to the startup screen through the discard guard; Open loads a
  saved character with that file's type.
- `create-character.e2e.js` — happy path: load rules, add a virtue, validate,
  save, reload.
- `character-types.e2e.js` — each created type carries its own profile budget and
  category rules, and the type cannot be changed afterwards.
- `validation-errors.e2e.js` — an illegal (forbidden-category) selection renders
  a localized, error-severity issue in the validation panel.
- `validation-modes.e2e.js` — the same illegal entity is reported differently
  under each `ValidationMode`: Enforced keeps errors, Advisory downgrades them to
  warnings, Silent clears the panel.
- `wizard.e2e.js` — the guided flow's machinery on a magus: the rail's order, the
  per-step gate and the validation mode that lifts it, back/forward navigation,
  the untouched mark, and Finish landing in the editor.
- `grog-wizard.e2e.js`, `companion-wizard.e2e.js`,
  `mythic-companion-wizard.e2e.js`, `magus-wizard.e2e.js` — one per character
  type, each walking that type from the startup screen to Finish with **every**
  declared phase filled in, then asserting the character is complete (no phase
  left marked untouched) and legal (no error-severity finding) and that it
  survives a save and a reload. The phase list comes from
  `rules/core/character_types.json`, and the shared driving from `wizard-walk.js`,
  so a profile that gains a phase makes all four walks visit it.
- …and the rest of `specs/`, one file per mechanic or reported defect (Arts,
  Spells, Houses, familiar, talisman, longevity ritual, Markdown export, the
  layout/geometry specs, …). Each file's header comment states what it is for.

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
