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

German sources (`rules/source/de/`) are the **source of truth for German
translations** — both the full rulebook text and the term mappings used to
fill `rules/i18n/de/`. The German rulebook file set does **not** match English
1:1:

```
Ars Magica Definitive Edition Basisregeln.md            # ↔ Core Rules
Ars Magica 5e - Häuser des Hermes - Mysterienkulte.md   # ↔ Houses of Hermes - Mystery Cults
Ars Magica 5e - Häuser des Hermes - Societates.md       # ↔ Houses of Hermes - Societates
Ars Magica 5e - Häuser des Hermes - Wahre Linien.md     # ↔ Houses of Hermes - True Lineages
Ars Magica 5e - Magie - Heckenzauber (Überarbeitet).md  # ↔ Magic - Hedge Magic (Revised)
Ars Magica 5e - Sphären der Macht - Magie.md            # ↔ Realms of Power - Magic
Ars Magica 5e - Wächter des Waldes - Das Rhein-Tribunal.md  # Rhine Tribunal — no English source in repo
```

Not yet available in German: Realms of Power — Faerie, The Divine, The
Infernal. Conversely, the Rhine Tribunal book has no English counterpart here.
Because English is the source of truth for IDs, German-only books cannot
introduce new IDs; they only supply i18n text against existing English-derived
IDs (or wait until the English source is added).

### German translation tables

`rules/source/de/translation-tables/` holds 18 thematic, hand-curated **EN↔DE
glossary tables** (Markdown tables: `Englisch (EN) | Deutsch (DE) | Anmerkung`),
distilled from three master tables (main glossary of 22 sections, 30 additional
terms, 362 spell names). These are the **canonical EN→DE terminology mapping**:
when generating `rules/i18n/de/` text, the German label for any term whose
English form appears in a table MUST match the table's `Deutsch (DE)` value.
`README.md` indexes the tables and records resolved naming conflicts;
`uebersetzungsregeln.md` gives the prose/formatting rules (natural German
syntax, `n/a`→`n/v`, Latin terms kept untranslated, etc.). Key conventions:
Latin terms tagged `(Lat.)` stay untranslated; the type-label `Tainted` →
`Befleckt` (but the flaw name `Depraved` → `Verdorben`); supernatural abilities
are marked `*`. The `Anmerkung` column carries gender/number and disambiguation
notes.

**Provenance is per-language.** Each item's `source` field (`SourceRef`:
`{ file, lines: [start, end] }`) records the file basename plus an inclusive
line range. The `source` in `rules/core/` always points at the **English**
file — the canonical source. German source files mirror the English ones
**line-by-line throughout** — even lists that are alphabetically ordered in
English (e.g. Virtues, Flaws, Abilities) are **not** re-sorted into German
alphabetical order; each German entry keeps the same line position as its
English counterpart. So a German line number corresponds to the same item as
the English line number. Such per-language provenance, when needed, lives in
the `rules/i18n/<lang>/` layer, not in language-neutral `core/`.

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
  No user-facing string hardcoded in Rust or Svelte source. A raw ID or enum
  value (category, magnitude, kind, …) must **never** be rendered directly as a
  user-facing label — always map it through a Fluent key
  (`category-<id>`, `magnitude-<id>`). Rendering the slug itself is the same
  violation as hardcoding a string.

## Engineering conventions

- **TDD mandatory.** Red → green → refactor. No implementation code without a
  preceding failing test. Commits should reflect the cycle.
- **Human-readable code.** Intention-revealing names, small single-purpose
  functions, early returns over deep nesting. A domain expert should be able to
  follow the rules engine by reading it.
- **Code style enforced.**
  - Rust: `rustfmt` + `clippy` (warnings as errors: `#![deny(clippy::all)]`)
  - Svelte/TS: `prettier` + `eslint`
- **English** for all code, identifiers, comments, commit messages, and all
  assistant communication (chat responses, PR descriptions, status updates) —
  regardless of the language the user writes in.
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
- **Committing on `main` is fine.** This repo's workflow commits directly to
  `main`; do not branch first or open a PR unless explicitly asked.
- **Keep `README.md` current.** `README.md` is the GitHub-facing project
  overview. When a change alters what the project does, its status, the tech
  stack, or the build/run commands, update `README.md` in the same change so it
  never drifts from reality.

## Rules provenance

- **Rules backed by source, never memory.** Every rule implemented in code or
  encoded as data MUST be taken from the authoritative Markdown in
  `rules/source/<lang>/` (English is the source of truth). This applies to
  *every* sourcebook, not just the core rules — e.g. hedge-wizard mechanics come
  from *Hedge Magic (Revised)*, House mysteries from *Houses of Hermes — Mystery
  Cults*. Implementing a rule from training-data recollection is prohibited: if
  the passage is not in the source files, the rule is not implemented until the
  source is added. A rule from a book with no English source in
  `rules/source/en/` yet (e.g. the Rhine Tribunal book) cannot be implemented,
  because English is the source of truth for IDs.
- **Cite the source at the implementation site, by book.** When implementing a
  mechanic in Rust, add a comment citing the **source file basename** + inclusive
  line range — the basename identifies which book, e.g.
  `// Source: Ars Magica - Definitive Edition (Core Rules).md:2774`. Never cite
  bare line numbers; they are meaningless without the book. Verify every line
  range against the actual file before committing it — do not trust recalled
  numbers.
- **Maintain the traceability map.** `crates/arm-rules/RULES.md` maps each rule
  → verbatim excerpt → source file:line → implementing function/file (and the
  JSON data value where the rule's number lives), organized by book. Update it
  in the same change as any mechanic. JSON files carry no comments, so RULES.md
  is the provenance home for rule values encoded as data (e.g. character-type
  budgets in `rules/core/character_types.json`).

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

# Frontend type-check (svelte-check) — vitest does NOT type-check
cd ui && npm run check

# Frontend lint/format
cd ui && npm run lint && npm run format:check

# Frontend unit tests
cd ui && npm run test:unit

# Full Tauri dev build
cargo tauri dev

# E2E tests (headless)
cd ui && npm run test:e2e
```

### Required gate (must pass before any commit / "done" claim)

A change is not verified until **all** of these pass. The last command is the
authoritative one and is mandatory — it is the only step that exercises the
shipped production code path.

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
cd ui && npm run test:unit && npm run lint && npm run format:check && cd ..

# FULL RELEASE APP COMPILE — non-negotiable final gate.
# Runs the frontend build (svelte-check type-check + vite) via beforeBuildCommand
# AND compiles the release binary. `cargo test`/`clippy` and `npm run test:unit`
# (vitest) do NOT type-check the frontend or build the production app, so type
# errors and production-only breakage slip past every other gate — only this
# command catches them. This is what `./arm-char-gen.sh` runs to launch.
cargo tauri build --no-bundle
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
    Nor(Vec<Prereq>),        // NOR — none may be present (serde tag "none")
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
