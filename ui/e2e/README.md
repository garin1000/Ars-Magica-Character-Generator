# End-to-end tests (tauri-driver, real binary)

These drive the **real release binary** through the platform WebDriver, so they
verify true Tauri IPC, the bundled rules resources, and the save/load flow.

## Prerequisites

- `tauri-driver` — `cargo install tauri-driver`
- **Linux:** `WebKitWebDriver` — `sudo apt install webkit2gtk-driver`
  (macOS uses the system `safaridriver`; Windows needs `msedgedriver`.)
- A display. In headless CI wrap the command with `xvfb-run`.

## Run

```bash
cd ui
npm run test:e2e          # builds the release binary, then runs the suite
# headless CI:
xvfb-run -a npm run test:e2e
```

`wdio.conf.js` builds `arm-app` in release, spawns `tauri-driver`, and launches
the app. Native file dialogs can't be automated over WebDriver, so the app
honors the `ARM_E2E_FILE` environment seam (set by the config) to save/load a
fixed temp path instead of opening a dialog. See
`crates/arm-app/src/commands.rs`.

## Status

The harness is complete but is **gated on `WebKitWebDriver`**, a system package
that must be installed separately (it is not bundled). Command logic and the
canonical save/load round-trip are independently covered by the Rust
integration tests in `crates/arm-app/tests/commands.rs`, which need no webview.
