# Ars Magica 5th Edition Character Generator

## Project overview

A locally installed, cross-platform desktop character generator for Ars Magica
5th Edition. Builds **characters** (grog, companion, mythic companion, magus)
and **covenants** as first-class entity types with three input modes: guided
wizard, direct-validated, and direct-unchecked.

## Technical stack (locked)

| Concern            | Choice                                                     |
|--------------------|------------------------------------------------------------|
| Shell / packaging  | **Tauri 2** — native binary, OS webview                    |
| Core logic         | **Rust** — pure library crate (`arm-rules`)                |
| Frontend           | **Svelte 5 + Vite** (SPA, no SSR)                         |
| UI localization    | **Fluent** (`.ftl`): `fluent-bundle` (Rust) / `@fluent/bundle` (JS) |
| Rules + save format| **JSON** via `serde`                                       |
| Targets            | Windows, macOS, Linux                                      |

## Workspace layout

```
arm-char-gen/
  Cargo.toml                 # cargo workspace
  crates/
    arm-rules/               # PURE engine: types, parse, validate, evaluate. No tauri/IO.
    arm-app/                 # Tauri binary; depends on arm-rules; owns file IO + commands
  ui/                        # Svelte 5 + Vite frontend (SPA)
  rules/
    source/<lang>/*.md       # authoritative human-authored rules in Markdown (NOT loaded at runtime)
    core/*.json              # language-neutral mechanics
    i18n/<lang>/*.json       # translatable rules text, keyed by stable IDs
  locales/<lang>/*.ftl       # Fluent UI strings
  examples/                  # sample character and covenant saves for tests/demo
```

## Rules source files

All provided rulebooks are released under the **Ars Magica Open License**
(based on CC-BY-SA 4.0), so the Markdown source text may be redistributed with
the repository — no proprietary-copyright restrictions apply. Attribution and
share-alike terms still apply to derived content.

The authoritative Markdown in `rules/source/<lang>/` is the human-readable
rulebook text. JSON in `rules/core/` and `rules/i18n/` is **generated** from it
by extraction (hand-authoring the full catalogue is infeasible); the engine's
referential-integrity + serde validation is the trust gate. **English is the
source of truth** — IDs are derived from the English source; other languages
only fill i18n text against IDs that already exist.

Current English sources (`rules/source/en/`):

```
Ars Magica - Definitive Edition (Core Rules).md
Ars Magica 5e - Houses of Hermes - Mystery Cults.md
Ars Magica 5e - Houses of Hermes - Societates.md
Ars Magica 5e - Houses of Hermes - True Lineages.md
Ars Magica 5e - Magic - Hedge Magic (Revised).md
Ars Magica 5e - Realms of Power - Faerie.md
Ars Magica 5e - Realms of Power - Magic.md
Ars Magica 5e - Realms of Power - The Divine (Revised).md
Ars Magica 5e - Realms of Power - The Infernal.md
```

**Provenance is per-language.** Each item's `source` field (`SourceRef`:
`{ file, lines: [start, end] }`) records the file basename plus an inclusive
line range. The `source` in `rules/core/` always points at the **English**
file — the canonical source. German source files mirror the English ones
line-by-line **except in ordered lists** (e.g. Virtues, Flaws, Abilities — not
exhaustive), which are re-sorted per German rules. So German line numbers
diverge from English wherever an ordered list appears: a German line reference
must be computed against the German file, never reused from English. Such
per-language provenance, when needed, lives in the `rules/i18n/<lang>/` layer,
not in language-neutral `core/`.

## Architecture invariants

- **Engine purity.** `arm-rules` has NO dependency on `tauri`, filesystem, or UI.
  It operates on in-memory data (`&str` / `&[u8]`), parses rulesets and entities,
  evaluates them, and returns results. Fully testable with `cargo test`.
- **Entity-generic design.** Characters and covenants share one buildable-entity
  abstraction (`EntityKind`). No character-specific assumptions in the engine;
  covenant support = new data + thin UI, never an engine rewrite.
- **Character types are data-driven profiles.** Each type (grog, companion,
  mythic companion, magus) is a JSON profile defining: V/F budget, caps,
  permitted/forbidden categories, required/forbidden traits, Gift policy, and
  ordered creation phases.
- **One evaluation path.** The engine always computes validation results.
  `ValidationMode` (Enforced / Advisory / Silent) governs enforcement at the
  caller level. No separate "without rule checking" branch.
- **Prerequisites as recursive boolean expressions.** `Prereq` enum with
  `All`, `Any`, `None`, `Has`, `House`, `AbilityMin`, `ArtMin`, `IsMagus` —
  exhaustive `match` so adding a variant is a compile error until handled.
- **Two-file separation per rules domain.** Language-neutral mechanics in
  `rules/core/`, translatable text in `rules/i18n/<lang>/`, joined by stable
  slug-style IDs (`virtue.puissant_ability`).
- **Strict separation of data kinds.** Mechanics files: zero translatable strings.
  UI strings: only in Fluent `.ftl`. Rules text: only in `rules/i18n/<lang>/`.
  No user-facing string hardcoded in Rust or Svelte source.

## Engineering conventions

- **TDD mandatory.** Red → green → refactor. No implementation code without a
  preceding failing test. Commits should reflect the cycle.
- **Human-readable code.** Intention-revealing names, small single-purpose
  functions, early returns over deep nesting. A domain expert should be able to
  follow the rules engine by reading it.
- **Code style enforced.**
  - Rust: `rustfmt` + `clippy` (warnings as errors: `#![deny(clippy::all)]`)
  - Svelte/TS: `prettier` + `eslint`
- **English** for all code, identifiers, comments, and commit messages.
- **Current dependencies.** Latest stable versions. Pin below latest only with
  documented reason.
- **Canonical serialization.** Sort object keys and arrays by `id`/`ref` before
  writing JSON. Use `BTreeMap` / explicit sort. Zero-noise git diffs.
- **Referential integrity validated at load.** Every `has`, `incompatible_with`,
  and parameter `ref` must resolve. Incompatibilities must be symmetric. Fail
  loudly with clear error listing offending IDs.
- **Saves store choices, not resolved values.** Record ruleset `id` + `version`.
  Schema-versioned (`schema_version` field).
- **YAGNI / KISS.** Build what is needed now, nothing speculative.

## Build & test commands

```bash
# Rust engine tests
cargo test -p arm-rules

# Rust workspace (all crates)
cargo test --workspace

# Clippy (warnings as errors)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check

# Frontend dev server
cd ui && npm run dev

# Frontend lint/format
cd ui && npm run lint && npm run format:check

# Full Tauri dev build
cargo tauri dev

# E2E tests (headless)
cd ui && npm run test:e2e
```

## Data model quick reference

### IDs
Slug-style, never translated: `virtue.puissant_ability`, `flaw.deficient_technique`,
`ability.awareness`, `art.creo`, `house.bjornaer`.

### Magnitudes
`free` = 0 points, `minor` = 1 point, `major` = 3 points.
Combined with `kind` (virtue costs, flaw grants) for balance computation.

### Prereq enum (Rust)
```rust
enum Prereq {
    All(Vec<Prereq>),
    Any(Vec<Prereq>),
    None(Vec<Prereq>),       // NOR — none may be present
    Has(Id),
    House(Id),
    AbilityMin { ability: Id, score: u8 },
    ArtMin { art: Id, score: u8 },
    IsMagus,
}
```

### ValidationMode
```rust
enum ValidationMode { Enforced, Advisory, Silent }
```
- `Enforced` — guided + direct-validated modes (blocks illegal states)
- `Advisory` — shows violations as non-blocking warnings
- `Silent` — suppresses validation display (unchecked mode)
