# Test-suite performance — measurements and open leads

Where the test wall-clock actually goes, what has been measured, and what has
been **ruled out by measurement** so it is not investigated twice. Written
2026-09-10, consolidating a prior session's e2e investigation (which lived only
in a transcript) with fresh measurements of the other layers.

## 1. The breakdown — e2e is ~99% of it

| Layer | Wall clock | Measured | Note |
|---|---|---|---|
| `cargo test --workspace` | **~11.5s** | 2026-09-10, warm | Slowest: `rules_source_provenance` 6.58s, `arm_rules` lib 3.47s (1104 tests) |
| `npm run test:unit` (both vitest projects) | **~16.4s** | 2026-09-10 | 70 files, 1332 tests |
| `npm run test:e2e` | **~41m43s** | 2026-09-09 run | 43 specs |

Neither Rust nor vitest is worth optimizing. **Every speed-up lives in e2e.**

`svelte-check`, `eslint`/`prettier` and the `cargo tauri build --no-bundle` gate
are **unmeasured** — treat as unexplored, not as "found nothing". The e2e
`onPrepare` build was observed at ~35s even warm.

## 2. Where the e2e time goes

43 spec files, `maxInstances: 1` (`ui/e2e/wdio.shared.conf.js:23`), strictly
serial. **Each spec file gets its own WebDriver session** — its own app launch
and teardown. Per-spec cycle ~40s, averaged over the first 20 specs of the
2026-09-09 run (delta between consecutive `POST /session` timestamps).

The dominant cost is one blocking `POST /session` per spec at **~30s**.
Cleanest sample, `tmp/e2e-logs/app-quit-bridge-clean.e2e-0-3.log`:

```
21:13:12.020  [POST] http://127.0.0.1:4444/session
21:13:42.296  COMMAND getWindowHandle()      <- first test command
21:13:44.067  deleteSession()
```

**30.28s of setup for 1.8s of testing.** 43 × ~30s ≈ **22 min of the 41m43s**.

`wdio.shared.conf.js:24` claims "Specs run serially against one shared app
instance". That is **wrong** — every spec log carries its own session id and
ends in `deleteSession()`. Correct it in the same pass as any e2e work.

## 3. The 30s — narrowed, NOT root-caused

Ruled out as the source, each by inspection or measurement:

- **wdio** — issues one `POST /session` and blocks; the time is on the far side.
- **tauri-driver** — no sleeps or retries in its spawn path (`src/main.rs`,
  `src/webdriver.rs`).
- **WebKitWebDriver** — `SessionHostGlib.cpp` at the webkitgtk-2.52.6 tag
  (matching the installed 2.52.6) has **no startup timeout**; it retries the
  inspector socket every 100ms indefinitely.
- **The app's ruleset load** — `main.rs` does nothing blocking at startup,
  `load_ruleset` is a frontend-invoked command, and `rules/` is only ~1.1MB.

So the app genuinely takes ~30.27s from spawn to opening its WebKit inspector
socket. **The timer is inside the app's WebKitGTK startup, and it has not been
found.**

### Experiments that did NOT work

Each row is a full run; gap measured from the worker log.

| Variant | Result |
|---|---|
| baseline | 30.26–30.30s |
| `WEBKIT_DISABLE_COMPOSITING_MODE=1` | 30.263s |
| `LIBGL_ALWAYS_SOFTWARE=1` | 30.237s |
| `WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1` | 10.16s, **then 30.28s, 30.27s** |
| sandbox + software GL | 30.27s |
| `GTK_USE_PORTAL=0` | 30.28s |

**Red herring:** every run prints `libEGL warning: DRI3 error: Could not get
DRI3 device`. Forcing software GL removes the warning and changes nothing.

**Methodological warning — the most important line in this document.** The
sandbox variant was called "found it" off that single 10.16s sample, and two
repeats came back at 30.28s and 30.27s. The baseline has real outliers: the
43-spec run contained specs at 12.8s and 18.2s. **A single measurement here
proves nothing. Repeat everything at least 3×.**

### The discriminator not yet run

**Does the release binary start fast when NOT under WebDriver?** If a plain
launch is ~1s, automation mode is the trigger; if it is also ~30s, it is
WebKitGTK on this box and end users see it too. One measurement, and it splits
the remaining search space. Run this before any further variant-hunting.

## 4. Speed-ups, with expected savings

| Lever | Saving | Risk | Note |
|---|---|---|---|
| **Consolidate 43 spec files → ~10** | **~20 min** | low | ~33 fewer startups at ~40s. No infrastructure change, no coverage loss — specs already run serially against a fresh app each, so merging only changes how often that app starts. This is `docs/open-todos.md` row 21 (`42c8dac`). |
| **`maxInstances > 1` (parallelism)** | large, multiplicative | medium | **Blocked as the config stands:** `beforeSession` spawns tauri-driver on the fixed port 4444 and `afterSession` kills it, so concurrent workers collide. Needs a per-worker port first — and per-worker `ARM_E2E_FILE` / `ARM_E2E_EXPORT_FILE` fixture paths, which are currently one shared path each in `os.tmpdir()` (`wdio.conf.js:29,34`). |
| **Replace the hardcoded 2s sleep with a port poll** | ~1.4 min | trivial | `wdio.conf.js:105` — `setTimeout(resolve, 2000)` per session, 43 × 2s. |
| Root-cause the 30s | up to ~22 min | unbounded | Highest ceiling, least predictable. Timebox it behind the discriminator above. |
| `specFileRetries: 1` | — | — | **Do not remove** — it absorbs real flakes. Just be aware every flake re-pays the full ~40s: one logged run took 3 retries, a variant with 21 failures paid it 21 times. |

Reusing one session across spec files is not a separate lever: wdio's model is a
session per spec file, so it is the same thing as consolidation.

## 5. Operational cautions

- **Never start an e2e run while another is going.** Fixed port 4444, and
  `onPrepare` rewrites `target/release`.
- **`target/release` may carry the e2e seams.** `ui/e2e/wdio.conf.js` builds
  with `--features e2e-testing`, so after any e2e run the local binary has the
  `ARM_E2E_FILE` / `ARM_E2E_EXPORT_FILE` seams compiled in. Rebuild clean before
  manual testing. This does **not** affect releases: `.github/workflows/release.yml`
  builds from a fresh checkout with a plain `cargo tauri build`, and the feature
  is off by default (`crates/arm-app/Cargo.toml:51-52`) — verified 2026-09-10
  against the v0.3.0 release path.
- **Raw data survives.** `tmp/e2e-logs/` holds 169 logs including the full
  2026-09-09 43-spec run, so the numbers above are re-derivable. Caveat:
  `app-quit-bridge-clean.e2e-0-0.log` was overwritten by single-spec
  experiments; the suite original is `-0-3.log` and is intact.
