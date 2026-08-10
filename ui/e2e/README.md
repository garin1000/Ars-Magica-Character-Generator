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
npm run test:e2e          # builds the release binary, then runs the suite
```

One command, both environments — no `xvfb-run` wrapper.

`wdio.conf.js` builds `arm-app` in release, spawns `tauri-driver`, and launches
the app. Native file dialogs can't be automated over WebDriver, so the app
honors the `ARM_E2E_FILE` environment seam (set by the config) to save/load a
fixed temp path instead of opening a dialog. See
`crates/arm-app/src/commands.rs`.

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

## Specs

- `create-character.e2e.js` — happy path: load rules, add a virtue, validate,
  save, reload.
- `validation-errors.e2e.js` — an illegal (forbidden-category) selection renders
  a localized, error-severity issue in the validation panel.
- `validation-modes.e2e.js` — the same illegal entity is reported differently
  under each `ValidationMode`: Enforced keeps errors, Advisory downgrades them to
  warnings, Silent clears the panel.

The two validation specs require the same display + release-build prerequisites
as the happy path; they cannot run without `WebKitWebDriver` and a display.

> The shipped sample ruleset (`rules/core/`) has too few selectable virtues to
> exceed the companion's 10-point budget through clicks alone, so the
> error-display spec uses a forbidden-category flaw (same render path) rather
> than an over-budget case. Over-budget rendering is covered at the engine level
> by `crates/arm-app/tests/commands.rs::over_budget_virtues_is_reported`.

Pure display helpers (`src/lib/derive.ts`) are unit-tested with Vitest, separate
from this webview-gated suite: `cd ui && npm run test`.

## Status

The harness is complete but is **gated on `WebKitWebDriver`**, a system package
that must be installed separately (it is not bundled) — plus `Xvfb` where there
is no desktop session. Command logic and the canonical save/load round-trip are
independently covered by the Rust integration tests in
`crates/arm-app/tests/commands.rs`, which need no webview.
