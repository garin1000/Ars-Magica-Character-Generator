# Implementation Plan

## Guiding principle

**Slice-first.** Prove the full architecture on a thin vertical slice before
broadening. Every engine type is entity-kind-agnostic from the start.

---

## Milestone 0 — Project scaffolding

- [x] Write `CLAUDE.md`
- [x] Write `PLAN.md`
- [x] Initialize Cargo workspace with `arm-rules` crate (lib, no deps beyond serde)
- [x] Initialize Tauri 2 app (`arm-app` crate + `ui/` Svelte 5 + Vite)
- [x] Configure tooling: rustfmt, clippy deny warnings
- [x] Configure tooling: prettier, eslint
- [x] Create directory structure: `rules/core/`, `rules/i18n/`, `rules/source/`,
      `locales/`, `examples/`
- [x] Initialize git repository

## Milestone 1 — Thin vertical slice (engine)

Scope: one entity kind (character), one type (companion), ~5 Virtues/Flaws,
two languages (en, de). Engine only — no UI yet.

### 1a. Core types & serialization (TDD) — DONE
- [x] Define `Id` newtype (slug string)
- [x] Define `EntityKind` enum (`Character`, `Covenant`)
- [x] Define `Magnitude` enum (`Free`, `Minor`, `Major`) with point values
- [x] Define `ItemKind` enum (`Virtue`, `Flaw`, `Boon`, `Hook`)
- [x] Define `PointItem` struct (id, kind, magnitude, category, entity_kinds,
      prerequisites, incompatible_with, parameters, source)
- [x] Define `Prereq` enum (All, Any, None, Has, House, AbilityMin, ArtMin, IsMagus)
- [x] Define `CharacterType` profile struct (budget, caps, permitted/forbidden
      categories, required/forbidden traits, gift policy, creation phases)
- [x] Serde round-trip tests for all types (14 unit tests)
- [x] Write seed data: `rules/core/virtues_flaws.json` (10 items),
      `rules/core/character_types.json` (companion + grog profiles)
- [x] Write i18n data: `rules/i18n/en/virtues_flaws.json`,
      `rules/i18n/de/virtues_flaws.json`

### 1b. Ruleset loading & referential integrity (TDD) — DONE
- [x] Define `Ruleset` struct (id, version, point_items, character_types)
- [x] Parse `core` + `i18n` JSON into `Ruleset`
- [x] Validate referential integrity: all prereq/incompatible refs resolve,
      incompatibilities are symmetric
- [x] Error type with clear messages listing offending IDs
- [x] Tests: valid load, missing ref, asymmetric incompatibility, unknown ID in prereq (5 unit tests)

### 1c. Entity model & validation (TDD) — DONE
- [x] Define `Entity` struct (kind, type_id, selections: Vec<Selection>)
- [x] Define `Selection` struct (ref to point_item id, params)
- [x] Point-balance computation (sum magnitudes by kind against type budget)
- [x] Prerequisite evaluation (recursive Prereq match)
- [x] Incompatibility checking
- [x] Cap enforcement (e.g. max Major Virtues)
- [x] Forbidden category enforcement
- [x] `ValidationResult` struct with list of `ValidationIssue` (severity, message key, context)
- [x] `ValidationMode` enum (Enforced, Advisory, Silent) — filtering layer
- [x] Tests: balanced entity, over-budget, missing prereq, incompatible pair,
      cap exceeded, unknown ref, all three validation modes (12 unit tests)

### 1d. Save/load round-trip (TDD) — DONE
- [x] Define save format (schema_version, ruleset id+version, entity data)
- [x] Serialize entity to canonical JSON (sorted keys via BTreeMap)
- [x] Deserialize and validate against ruleset
- [x] Tests: round-trip identity, canonical output stability (2 tests)
- [x] Integration tests against shipped data files (7 tests)

## Milestone 2 — Thin vertical slice (UI + Tauri integration)

Scope: wire the engine to a minimal Svelte UI via Tauri commands. Two languages,
companion type, direct-entry mode only.

### 2a. Tauri commands
- [x] `load_ruleset` command — read JSON files from `rules/`, parse, return metadata
- [x] `validate_entity` command — accept entity JSON, return validation results
- [x] `save_entity` / `load_entity` commands — file dialog, canonical JSON
- [x] Integration tests (Rust side) for commands (10 tests, webview-free)

### 2b. Svelte UI — direct entry
- [x] Language selector (en/de) with Fluent integration
- [x] Basic Fluent `.ftl` files for UI chrome (en, de)
- [x] Entity editor: list selected V/F, add/remove, parameter selection
- [x] Live validation display (issues list, colored by severity)
- [x] Validation mode toggle (Enforced / Advisory / Silent)
- [x] Save / Load buttons wired to Tauri commands

### 2c. E2E test
- [x] e2e: load ruleset → add V/F → see validation → save → reload → verify.
      WebdriverIO + tauri-driver against the real production binary
      (`cargo tauri build --no-bundle`), with an `ARM_E2E_FILE` dialog seam for
      determinism. Passing. Requires the `webkit2gtk-driver` system package; see
      `ui/e2e/README.md`.

## Milestone 3 — Guided creation wizard

- [ ] Wizard component driven by character type's phase list
- [ ] Phase navigation (next/back/skip where allowed)
- [ ] Per-phase validation gating (enforced mode blocks advancing with errors)
- [ ] Companion wizard flow complete
- [ ] Grog wizard flow (subset of phases)

## Milestone 4 — Magus support

- [ ] Arts data model (Techniques + Forms, scores)
- [ ] House data (id, required/granted virtues)
- [ ] Spell data model (basic: technique + form + level)
- [ ] Magus character type profile with extended phases
- [ ] House selection step (auto-grants free House Virtue)
- [ ] Arts allocation step
- [ ] Apprenticeship calculation
- [ ] Magus wizard flow
- [ ] Additional V/F data for Hermetic category

## Milestone 5 — Characteristics & Abilities

- [ ] Characteristics model (Int, Per, Str, Sta, Pre, Com, Dex, Qik) with point-buy
- [ ] Abilities model (categories, specialties, XP-to-score table)
- [ ] Characteristic allocation step in wizard
- [ ] Ability allocation step in wizard
- [ ] Age & experience system (later life stages)

## Milestone 6 — Covenants

- [ ] Boons & Hooks data (same PointItem structure, EntityKind::Covenant)
- [ ] Covenant entity type profile
- [ ] Covenant resources model (Library, Vis, Specialists, etc.)
- [ ] Covenant wizard flow
- [ ] Covenant UI

## Milestone 7 — Full data population

- [ ] Complete V/F catalogue from ArM5 core book
- [ ] Complete Abilities catalogue
- [ ] Complete Arts
- [ ] Complete Houses
- [ ] Complete Boons & Hooks
- [ ] All data in en + de (+ additional languages as available)
- [ ] Markdown source files for all rules content

## Milestone 8 — Export & polish

- [ ] Character sheet export (PDF and/or Markdown)
- [ ] Covenant sheet export
- [ ] Ruleset versioning & save migration
- [ ] Multiple rulebook/supplement support
- [ ] UI polish, accessibility
- [ ] CI pipeline (cargo test, clippy, fmt, frontend lint, e2e)
- [ ] Release packaging for Windows, macOS, Linux

---

## Current focus: Milestone 3

Milestones 0, 1, and 2 complete. The Tauri app builds and launches, loads the
ruleset from bundled resources, validates live, round-trips canonical saves, and
passes a real-binary tauri-driver e2e. Next: guided creation wizard driven by
the character type's phase list.
