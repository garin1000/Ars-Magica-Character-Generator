# `scripts/` — developer / build tooling

These scripts are **dev/build tools**. They are **never loaded at runtime** by
`arm-rules` or `arm-app`. The data extractors regenerate hand-checkable JSON from
the authoritative Markdown in `rules/source/`, consistent with CLAUDE.md's rule
that "JSON is generated from the Markdown by extraction"; the engine's load-time
referential-integrity + serde validation remains the trust gate over whatever
they emit. The packaging scripts post-process release artifacts and run in the
release workflow.

## `extract_spells.py`

Extracts the full Hermetic spell catalogue from the Spells chapter of
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` and writes:

- `rules/core/spells.json` — language-neutral mechanics (id, technique, form,
  level, requisites, ritual, range/duration/target, creates_lasting, source
  line-range).
- `rules/i18n/en/spells.json` — English name + description prose, keyed by id.
- `rules/i18n/de/spells.json` — German names from the canonical translation table
  `rules/source/de/translation-tables/zauber-nach-form.md`. A spell absent from
  the table falls back to its English name (documented policy — the extractor
  reports every fallback); German is never invented.

Run it from anywhere:

```bash
python3 scripts/extract_spells.py
```

Properties:

- **Deterministic & canonical.** Output is sorted by id with stable formatting,
  so re-running produces a zero-noise diff.
- **In-loop checks fail loudly.** Technique resolves to a Technique-class Art and
  Form to a Form-class Art (against `rules/core/arts.json`); requisites are known
  arts; ritual ⇒ level ≥ 20; non-ritual ⇒ level ≤ 50; Year ⇒ ritual; Boundary ⇒
  ritual; ids unique; source ranges within the chapter. On any violation it
  prints the offending ids and exits non-zero without writing.
- **Reports.** Total spell count, per-Technique and per-Form counts, ritual
  count, and DE coverage (with the list of any English fallbacks).

The script tolerates the source's formatting inconsistencies (mixed CRLF/LF,
stray `.`/missing `,` in R/D/T lines, compound durations like "Sun & Year", a
`## Creo Mentem Spells` heading with two hashes, and the "Svells" typo). See
`crates/arm-rules/RULES.md` (Spells section) for the provenance and mapping
detail.

## `patch-appimage-wayland.sh` + `appimage/wayland-compat-hook.sh`

Post-build patch for the Linux AppImage, run by `.github/workflows/release.yml`
right after `cargo tauri build`.

**The problem.** Tauri's bundler lets linuxdeploy follow `libgtk-3`'s
dependencies, which copies `libwayland-client/-cursor/-egl/-server` into the
AppImage, and the app binary carries `RUNPATH=$ORIGIN/../lib` so those copies win
over the host's. `libEGL` is *not* bundled, so the host's Mesa is what actually
runs — and a current Mesa driving a libwayland from the (deliberately old) build
image fails:

```
Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...
```

`WebKitWebProcess` then aborts and the user sees an empty white window. This is a
Tauri-wide bundler defect ([tauri#15665](https://github.com/tauri-apps/tauri/issues/15665));
until it is fixed upstream we patch our own artifact.

**The fix.** `wayland-compat-hook.sh` is installed into the AppImage's
`apprun-hooks/` and sourced by `AppRun` before the app launches. Under a Wayland
session it preloads the host's libwayland so those sonames resolve to the system
copies that match the host's Mesa. It is deliberately conservative:

- Outside Wayland it does nothing.
- If the host has no libwayland, it does nothing — the bundled copies stay in
  the AppImage as a fallback, so the AppImage never gets *worse* than before.
- An existing `LD_PRELOAD` is preserved.
- `ARM_WAYLAND_LIB_DIRS` (colon-separated) overrides the search path.

```bash
./scripts/patch-appimage-wayland.sh                    # every AppImage in target/release/bundle/appimage
./scripts/patch-appimage-wayland.sh path/to.AppImage   # a specific one, patched in place
./scripts/patch-appimage-wayland.sh --appdir path/to.AppDir
./scripts/test-appimage-wayland-patch.sh               # tests (synthetic AppDir; no build, no display)
```

Repacking needs appimagetool, taken from `$APPIMAGETOOL`, from `PATH`, or from
`~/.cache/tauri/linuxdeploy-plugin-appimage.AppImage` (which a Tauri AppImage
build downloads anyway). Patching is idempotent.
