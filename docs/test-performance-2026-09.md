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

**Current shape (2026-10-11):** 15 spec files (consolidated from 43 in
1055d0db), run by 4 parallel workers (`MAX_WORKERS` in
`ui/e2e/wdio.shared.conf.js`, per-worker ports and fixtures since 75a243e6).
**Each spec file still gets its own WebDriver session** — its own app launch and
teardown. A full run on 2026-10-11 took 5m52s: a 63s release build in
`onPrepare`, then a 4m47s suite. 13 of the 15 sessions waited a flat ~30.3s
before their first command, which was 43% of all worker time. §3 has the cause.

*Historical (2026-09-09, before consolidation and parallelism):* 43 spec files,
`maxInstances: 1`, strictly serial. Per-spec cycle ~40s, averaged over the first
20 specs (delta between consecutive `POST /session` timestamps).

The dominant cost was one blocking `POST /session` per spec at **~30s**.
Cleanest sample, `tmp/e2e-logs/app-quit-bridge-clean.e2e-0-3.log`:

```
21:13:12.020  [POST] http://127.0.0.1:4444/session
21:13:42.296  COMMAND getWindowHandle()      <- first test command
21:13:44.067  deleteSession()
```

**30.28s of setup for 1.8s of testing.** 43 × ~30s ≈ **22 min of the 41m43s**.

(The old `wdio.shared.conf.js` comment claiming "Specs run serially against one
shared app instance" has since been corrected.)

## 3. The 30s — root-caused 2026-10-11 (see the end of this section)

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

### The discriminator — answered 2026-10-11

**Does the release binary start fast when NOT under WebDriver?** Yes: Norbert
confirmed that a plain release launch on his desktop starts immediately. The
experiments below show the trigger is neither automation mode nor `xvfb-run` as
such, but the environment's session bus.

### 2026-10-11: cause found — a non-answering desktop portal

Setup: temporary edits to `ui/e2e/driver.js::startWorkerDriver` only (reverted).
tauri-driver's stderr, which carries the app's, went to a timestamped file under
`tmp/`, and per-variant env was added to the spawn. Each row is one run of 4 short
specs on 4 parallel workers, so it gives 4 samples. Raw data:
`tmp/phase0-findings.md`. The runs were headless (`DISPLAY` unset), and the
session bus was the real user bus `/run/user/1000/bus`. No desktop session was
running (no X0 or Wayland socket).

| Variant | Gaps (4 samples) | Result |
|---|---|---|
| baseline | 30.28, 30.32, 30.31, 30.31s | slow |
| `G_MESSAGES_DEBUG=all` | 30.30–30.33s | slow; no debug lines reach the log (GLib writes debug to stdout, which is not forwarded) |
| `+ WEBKIT_DEBUG=all` | suite time unchanged | slow; no output either |
| `NO_AT_BRIDGE=1` | 30.29–30.32s | slow |
| `GTK_A11Y=none GIO_USE_VFS=local GSETTINGS_BACKEND=memory` | 30.27–30.33s | slow |
| **`DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent…`** | **0.26, 0.26, 0.26, 0.28s** | **fast** |
| **`dbus-run-session -- tauri-driver …`** (private bus) | **0.42, 0.50, 0.45, 0.44s** | **fast** |

**Cause:** tao is Tauri's windowing layer. Its `portal.rs::theme` (tao 0.35.3)
makes a **blocking** `org.freedesktop.portal.Settings.Read` call for the
colour scheme, with a **5s timeout**:

- It is called once at window creation.
- That call fails, so the cached theme stays unset. From then on every
  `Window::theme()` call repeats the 5s call, and tauri-runtime-wry makes one
  of those at webview creation and one in its theme getter.

The private-bus run shows the request directly: the dbus-daemon log reads
`Activating service name='org.freedesktop.portal.Desktop' requested by … arm-app`.
On the user bus, with no live desktop session behind the portal's backend, that
call does not answer within 5s. So 30.3s ≈ 6 × 5s (*inference*). The old 9.5s
and 21.0s outliers that were released at the same instant fit a portal that
started answering partway through.

Norbert's plain launch is fast because his live session's portal answers.
**User-facing note:** the same ~30s start would hit a Linux user whose
xdg-desktop-portal is installed but whose backend hangs. That is upstream tao
behaviour, not this app's code.

**Verification:** one full `npm run test:e2e` with the unreachable-bus
address passed 15 of 15 specs in **3m02s** (4m47s before). Every per-spec gap was
0.27–0.43s, with update-check-stall at 0.79s. The fix was reverted afterwards;
Phase 1 implements it test-first. The private-bus variant also works, but it
starts a portal, gvfs and at-spi per worker, and whether those are reaped was
not checked.

## 3a. Incremental Rust and release builds (2026-10-11)

Method: append one word to an existing comment, time the build, revert, then time
the rebuild. The first command of each pair was a confirmed no-op (0.13–0.37s).
**Caveat:** another workload was keeping the 8-CPU box at a load of about 7
during part of this, so the spread is wide.

| Edit | Command | Edit | Revert rebuild |
|---|---|---|---|
| (a) `arm-rules/src/lib.rs` comment | `cargo test --workspace --no-run` | 3m53s; repeat **10m24s** | 10m21s; repeat 10m23s |
| (b) `arm-rules/tests/de_vf_names.rs` doc comment | `cargo test -p arm-rules --test de_vf_names --no-run` | **0.54s** | 0.46s |
| (c) `arm-app/src/commands.rs` comment | `cargo test --workspace --no-run` | 8.9s | 1m28s |

- **(a) dominates.** A comment-only change to the engine library recompiles
  arm-rules and every integration-test crate that depends on it, about 4–10 min.
  So the number of integration-test binaries is the lever.
- **(b) is cheap**, but only for a comment. A real code edit in a test file still
  has to be codegen'd and linked, so the cost of a test-file edit inside a merged
  test binary remains unmeasured.
- **`-p` and `--workspace` do not share artifacts.** Right after a fully built
  `--workspace` state, the "no-op" `cargo test -p arm-rules --test de_vf_names
  --no-run` recompiled arm-rules for **6m21s**; the next run took 0.13s. So each
  switch between the two forms pays a rebuild.
- **(d) `cargo tauri build --no-bundle` twice in a row:** the second still
  recompiled arm-app (38.6s). Across the two runs `ui/dist` kept the same hashed
  asset names and byte sizes, with new mtimes. That is **mtime churn** from
  `vite build`, not a content change.
- **The e2e and portable builds embed a non-production frontend.**
  `@wdio/cli/bin/wdio.js` sets `NODE_ENV=test` when it is unset. `onPrepare`'s
  `cargo tauri build` inherits it, and Vite 6 keeps an inherited `NODE_ENV`, so
  `isProduction` is false. The e2e-built `index-*.js` was 704,921 bytes, against
  638,151 for the plain build. This is source-confirmed and consistent with the
  sizes, but the bundle content itself was not inspected. It touches the "e2e
  drives the shipped production code path" claim.

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

- **Never start an e2e run while another is going.** The driver ports are
  shared, and `onPrepare` rewrites `target/e2e` (since Phase 2, §7).
- **~~`target/release` may carry the e2e seams.~~ Resolved 2026-10-11 (§7).**
  The `--features e2e-testing` build now goes into its own cargo target dir,
  `target/e2e`, and the suite runs `target/e2e/release/arm-app`. The plain
  `target/release/arm-app` is only ever written by plain builds (the gate, the
  portable staging). Releases were never affected: `.github/workflows/release.yml`
  builds from a fresh checkout with a plain `cargo tauri build`, and the feature
  is off by default (`crates/arm-app/Cargo.toml:51-52`), verified 2026-09-10.
- **Raw data survives.** `tmp/e2e-logs/` holds 169 logs including the full
  2026-09-09 43-spec run, so the numbers above are re-derivable. Caveat:
  `app-quit-bridge-clean.e2e-0-0.log` was overwritten by single-spec
  experiments; the suite original is `-0-3.log` and is intact.

## 6. Phase 1 results (2026-10-11): e2e harness

Changes, all in `ui/e2e/`:
- The spawned app gets an unreachable `DBUS_SESSION_BUS_ADDRESS` (§3).
- Both builds run with `NODE_ENV=production`. The e2e and portable `index-*.js`
  are now byte-identical to a plain `cargo tauri build` (638,151 B, same hash).
- The specs run from an explicit longest-first list, guarded against drift by
  `wdio.conf.test.js`.
- `ARM_E2E_WORKERS` sets the worker count; the default is now **6**.
- `magus-editor.e2e.js` was split, moving Arts and Houses to
  `magus-arts-houses.e2e.js`. 23 `it`s before and after.

| Run | Workers | Specs | Test phase | Total | Retries |
|---|---|---|---|---|---|
| baseline (§2, before the fix) | 4 | 15 | 4:47 | 5:52 | — |
| bus + NODE_ENV fix, alphabetical | 4 | 15 | 2:45 | 3:49 | 0 |
| + longest first | 4 | 15 | 2:44 | 3:41 | 0 |
| | 6 | 15 | 2:16 | 3:13 | 0 |
| | 8 | 15 | 2:15 | 3:12 | 0 |
| + split | 8 | 16 | 2:09 | 3:05 | 0 |
| | 6 | 16 | 2:15 | 3:11 | 0 |
| confirm | 8 | 16 | 2:41 | 3:37 | **1** (magus-possessions, interactability) |
| confirm ×3 | **6** | 16 | 2:15 / 2:16 / 2:17 | 3:11 / 3:12 / 3:13 | 0 / 0 / 0 |
| portable | — | 1 | 0:05 (was 0:35) | — | 0 |

- **Load inflates every spec.** companion-editor took 84s at 4 workers, 113s
  at 6 and 130s at 8. Past 6, the longest spec sets the floor, not the worker
  count.
- After the split, companion-editor and wizard-flow are the floor (~105–130s).
  Splitting them is the next lever, if one is wanted.
- **Longest-first order** gained nothing at 4 workers: magus-editor already
  started first alphabetically. It matters at 6+, where the floor spec must
  start at t=0.
- **The dirty-quit hold stays at 20 s.** Its first `it` ends 1.9–3.8s after
  session start (4–8 workers). 8 s would leave only a 2.1–2.8× margin, under
  the 3× required, and the spec is not on the critical path.

## 7. Phase 2 results (2026-10-11): stop rebuilding what's already built

Changes (`ui/e2e/`, `ui/package.json`):
- **`npm run build` no longer churns `ui/dist`.** Vite writes into `tmp/vite-dist`,
  and `sync-dist.js` mirrors only the changed files into `ui/dist`, deleting stale
  ones. So an unchanged frontend leaves every mtime alone, and cargo no longer
  recompiles arm-app (§3a (d)). Checked: same hashed names and bytes as a plain
  `vite build`, `diff -r` identical.
- **The e2e build has its own target dir, `target/e2e`** (`CARGO_TARGET_DIR`;
  tauri-cli honours it). Rules are copied beside `target/e2e/release/arm-app`.
  Resource still resolves to the exe dir there, since tauri keys on `.cargo-lock`.
  After an e2e run the plain binary has 0 `ARM_E2E_FILE` strings and the e2e
  binary has 1. The cost is a one-time cold build of 7m13s, and 1.3 GB on disk.
- **Frontend stamp** (`build-freshness.js`, `tmp/build-stamps/`). `npm run build`
  records content hashes of Vite's inputs and of `ui/dist`, but only for a
  production build, and only if the inputs did not change while it ran. When
  they still match, the e2e build skips its own `npm run build`. A stamp IO
  failure only warns, because `npm run build` is the release `beforeBuildCommand`.
- **TAURI_CONFIG trap.** tauri-cli passes `--config` to cargo as `TAURI_CONFIG`,
  and tauri-build reruns on any change to it (`rerun-if-env-changed`), which costs
  an arm-app recompile (~45 s). So a target dir must always see the same
  `--config`:
  - the e2e build always removes `beforeBuildCommand` and runs the frontend
    itself, when it is stale;
  - the portable build runs exactly the gate's command.
- **`ARM_E2E_REUSE_BUILD=1` / `npm run test:e2e:reuse`** skips cargo when the
  binary, the Rust inputs and the frontend still match the last e2e build. It is
  opt-in, prints a loud line, and is never used by the gate.
- **Lint** caches eslint and prettier in `tmp/lint-cache/`. `format:check` was
  dropped from the documented gate, because `lint` already runs `prettier --check`.
  CI still runs both; that was left as it is.

Gate sequence, warm, no source change (one run each; e2e at 6 workers):

| Step | Before | After |
|---|---|---|
| `cargo tauri build --no-bundle` | 55–58 s (arm-app recompiled) | **14.4 s** (frontend 14 s, cargo 0.28 s) |
| `npm run test:e2e` | 3:12 (build ~57 s) | **2:23** (build ~1 s, cargo 0.28 s) |
| `npm run test:e2e:portable` | ~61 s (build 56 s) | **26 s** (frontend ~14 s, cargo 0.28 s) |
| one-spec rerun (`--spec update-check`) | ~70 s | **16 s**; 14.8 s with reuse |
| `npm run lint` | 18 s | 18.8 s cold cache, **1.8–2.0 s** warm |

- **A comment edit in `ui/src`.** The gate rebuilt the frontend. The bundle was
  byte-identical, so `ui/dist` stayed untouched and cargo was a no-op, and the
  stamp moved to the new inputs. e2e and portable then took the same times as
  above.
- **The e2e build after a revert, with no gate build in between.** The stamp was
  stale, so e2e ran `npm run build` itself: 34 s against 16 s for the one-spec rerun.
- **One unexplained recompile.** The first warm e2e build after the cold one
  recompiled arm-app once (37 s), though no input was newer. Every later e2e
  build was a 0.3 s no-op, including straight after a gate build.
- **Reuse is now worth only ~1.5 s,** because the default path already costs
  about 1 s of build. It stays as a cheap opt-in.

## 8. Phase 3 results (2026-10-11): Rust test build and run

Changes:
- **Three slow tests fixed** (test code only, same assertions):
  `de_vf_names` computes each table row's match key once (13.7 s -> 0.18 s for
  the file); `rules_source_provenance`'s range check reads each book once
  (6.68 s -> 0.06 s); `source_citations`' symbol sweep reads each file once and
  skips lines without the symbol (3.31 s -> 0.95 s).
- **One test binary per crate, files unmoved.** `autotests = false`;
  arm-rules has one target, `all` (`tests/all.rs`, a `mod` line per file);
  arm-app has `app` (`tests/app.rs`) and `commands` (alone, since it sets env
  vars). `test_targets.rs` fails when a test file is in no root, or in two.
  One binary was enough: a code edit in one test file rebuilds it in ~5 s.
  Run one file with `cargo test --workspace --test all de_vf_names::`.
- **`[profile.dev] debug = "line-tables-only"`**, and no debug info for
  dependencies. Backtraces keep file:line. `[profile.release]` is unchanged.

| Measure | Before | After |
|---|---|---|
| cold `--no-run`, after `cargo clean -p arm-rules -p arm-app` | 14m36s (§3a era) | 55.5 s one binary; **43.3 s** + light debug |
| full cold build, empty target dir | 3m35s (one binary, full debug) | **2m38s** |
| arm-rules lib comment edit, `--workspace --no-run` | 3m21s; 9m40s | **19.0 s**; 19.0 s |
| test-file code edit | 0.6 s (its own small binary) | 4.6 s |
| arm-app `src` comment edit | 9.1 s; 24.5 s | **3.6 s**; 3.7 s |
| `cargo test --workspace`, run phase | 47.0 s | **15.1 s** |
| target dir after a clean full build | 5.5 GB | **2.7 GB** |
| tests passed / ignored | 3,141 / 12 | 3,144 / 12 (+3 guard tests) |

- **Noise.** Some runs stalled with the CPU idle: a lib edit once took 2m32s
  with 40 s of CPU time, against 19 s otherwise. The CPU time is the stable
  figure: about 3m20s per lib edit before, about 50 s after.
- **`-p arm-rules` vs `--workspace` is a feature-unification split.** Inside
  the workspace, tauri adds features to arm-rules' dependencies: serde and
  serde_core get `alloc` and `rc`, serde_json gets `alloc` and `raw_value`
  (`unbounded_depth` is only on the build-script copy), and memchr, once_cell
  and bitflags differ too. The proc-macro stack differs as well (syn `full`,
  proc-macro2 `span-locations`). So each form keeps its own arm-rules build.
  Both builds stay cached, so a switch without an edit costs nothing (0.2 s).
  After an engine edit, the second form rebuilds arm-rules again, ~17 s.
  Making the two equal would take a workspace-hack: about eight pinned
  dependencies, including host-side syn features that follow tauri's macro
  crates. That is not small and would break silently on tauri upgrades, so it
  was not done. Use `--workspace` everywhere.

## 9. Phase 4 results (2026-10-11): vitest

`npm run test:unit`, three runs each. The phase figures are summed across workers.

| Measure | Before | After |
|---|---|---|
| wall (Duration) | 29.04 / 28.92 s | **17.30 / 17.41 / 17.49 s** |
| tests phase | 66.4 s | 16.5 s |
| collect phase | 62 s | 97 s (more workers in parallel, see below) |
| environment phase | 19.6 s | 4.3 s |
| files / tests | 133 / 2,807 | 133 / 2,808 (+1, the TDD test below) |
| slowest files | App.client 5.7 s, VirtueFlawTab 4.9, WizardStep 4.8, App 4.7, WizardShell 3.0 | App.client 1.8, wdio.conf 1.75, prereq-text 0.7, AbilityTab.client 0.6, state 0.5 |

The e2e-harness tests had already dropped out of the top list (wdio.conf.test.js
~0.95 s) before this phase.

Kept:
- **One Fluent bundle per language** (`i18n.ts::buildBundle`, TDD). Under
  Svelte's server build a `$derived` created outside a render is not memoized,
  so the store's `#bundle` reparsed the 114 KB `main.ftl` on **every**
  `store.t()` call in an SSR test, about 100 ms per render. This took the tests
  phase from 66 s to 20 s and the wall from 29 s to 22.3 s. The client build was
  never affected.
- **`client` project on `vmThreads`.** It took the wall from 21.7 s to 17.4 s.
  About 1.2 s of that comes from a second pool running beside `forks`, because
  vitest sizes every pool to cpus-1; a plain `threads` client pool gave 20.45 s.
  The rest comes from reusing the happy-dom set-up (environment 19 s -> 4 s).
- **`deps.optimizer` (ssr + web)**: 18.43 s -> 17.45 s with the vm pool, and
  21.66 s against 22.27 s without it.
- **App.client.test.ts polls `vi.waitFor` every 1 ms** instead of 50 ms. The
  condition and the 1 s timeout are unchanged. The file went from 4.7 s to 1.0 s
  of tests; there was no wall gain, because the `ssr` pool is the critical path.

Rejected:
- **`pool: 'threads'` for both projects**: 22.80 s against 22.27 s.
- **`isolate: false`**: a run hung for over 300 s and was stopped. Without
  isolation vitest skips `mocker.reset()` and `resetModules` between files, and
  90 files use `vi.mock`.
- **`vmThreads` for `ssr`**: 15.8 s, but 3 `toStrictEqual` failures in
  `state.svelte.test.ts`. `structuredClone` returns main-realm objects. Making it
  pass would mean weakening assertions.
- **More forks** (10 or 12): 17.25 / 17.38 s. The run is CPU-bound.
