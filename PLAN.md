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

## Milestone 3 — Characteristics & Abilities

Scope: the two foundational, type-agnostic trait domains every character needs.
Built as an engine → data → direct-entry slice (mirroring M1 → M2) so the guided
wizard (M4) has real phase content to orchestrate. No wizard yet.

Decisions made during M3 (see `crates/arm-rules/RULES.md` and the plan archive):
abilities store the **whole bought score + a single `unspent_xp` bank**, not
per-ability XP (ability XP is spent in whole points, so loose XP lives in the
bank; the effective score = bought + virtue bonuses is computed, never stored).
The life-stage XP flow and the effective-score (virtue-bonus) layer are deferred
to M4; `House`/`ArtMin` evaluation to M5.

### 3a. Engine models (TDD) — DONE
- [x] Characteristics model (Int, Per, Str, Sta, Pre, Com, Dex, Qik) with point-buy
      (`characteristics.rs`; cost table is data in `characteristics.json`)
- [x] Abilities model + 5 categories + XP advancement table (`ability.rs`)
- [x] Ability registry in the `Ruleset` (`from_core_json`) so `ability` parameter
      refs and `AbilityMin` resolve at load
- [x] Characteristic scores + whole ability scores + `unspent_xp` bank on `Entity`
      (schema_version 1→2, additive)
- [x] Evaluate `AbilityMin` against the entity's max bought ability score
      (`validation.rs`); `House`/`ArtMin` remain deferred (M5)
- [~] Life-stage XP acquisition (early childhood, 15/20/10 per year, age→max cap)
      — DEFERRED to M4 with the wizard that drives it. Source: Core Rules.md:2364-2394

### 3b. Data — DONE
- [x] Characteristics core data (`characteristics.json`); labels in Fluent (enum)
- [x] Seed Abilities catalogue (`abilities.json`, 23 abilities across all 5
      categories + full childhood restricted list) + i18n (en, de)
- [~] Sample Childhood packages — DEFERRED to M4 (built with the childhood model,
      loading, integrity, and apply-flow, rather than shipping inert unvalidated
      data). Source: Core Rules.md:2380-2388

### 3c. Direct-entry UI — DONE
- [x] Characteristic point-buy component with live points readout + validation
- [x] Ability allocation component (whole-score steppers + specialty + banked-XP
      field) with live validation

### 3d. Effective-score layer (TDD) — DONE
Pulled forward from M4: direct entry already lets a character take score-boosting
Virtues, so the effective score must be correct now (display, `AbilityMin`, caps).
No wizard dependency.
- [x] Data-driven `Effect` model on `PointItem` (`ability_bonus` / `characteristic_bonus`)
      + `max_per_target`; new `ParameterDomain::Characteristic`. Targets named by
      the selection's param value — no Virtue IDs in engine code.
- [x] `effective.rs`: bought + virtue bonuses, computed never stored
- [x] Puissant Ability (+2, ≤1/Ability); `AbilityMin` uses the effective score.
      Source: Core Rules.md:4814-4816
- [x] Great Characteristic (+1, base ≥ +3, ≤2/Characteristic, +5 effective ceiling
      via `effective_max`). Source: Core Rules.md:3987-3989
- [x] Multiplicity generalized in `validate_duplicate_selections`; effect
      integrity at load (`ruleset.rs`)
- [x] UI: read-only "effective" badge beside the base score; Great Characteristic
      target picker; Fluent + de i18n; real-binary e2e
- Deferred follow-ups: *Improved Characteristics* (+3 point-buy pool, a budget
  modifier — not an effective bonus); Puissant Art (+3) waits for the M5 Art registry.

## Milestone 4 — Guided creation wizard

Scope: wrap the full phase list in a guided flow, reusing the direct-entry
components from M2 (V/F) and M3 (characteristics, abilities).

- [ ] Wizard component driven by the character type's phase list
- [ ] Phase navigation (next/back/skip where allowed)
- [ ] Per-phase validation gating (enforced mode blocks advancing with errors)
- [ ] Characteristics + abilities phases wired to their M3 components
- [ ] Abilities phase offers two modes: simple flat allocation, and a
      "sophisticated" guided life-stage flow — early childhood (Native Language
      + the 45-xp restricted spread, with an optional Sample Childhood prefab),
      then 15 xp/year to the chosen age, enforcing the age → max-score cap
- [ ] Life-stage XP engine (deferred from M3): early-childhood 75+45 xp, later-life
      15/20/10 xp per year, age→max-Ability-score cap; feeds the `unspent_xp` bank
      and validates against it. Source: Core Rules.md:2364-2394
- [ ] Sample Childhood packages (deferred from M3): childhood model + registry +
      load-time integrity (ability refs resolve) + an "apply package" step.
      Source: Core Rules.md:2380-2388
- [ ] Companion wizard flow complete
- [ ] Grog wizard flow (subset of phases)

## Milestone 5 — Magus support

- [ ] Arts data model (Techniques + Forms, scores)
- [ ] Art registry in the `Ruleset` so `art` parameter refs and `ArtMin`
      resolve and evaluate (parallels the M3 ability registry)
- [ ] House data (id, required/granted virtues)
- [ ] Spell data model (basic: technique + form + level)
- [ ] Magus character type profile with extended phases
- [ ] House selection step (auto-grants free House Virtue)
- [ ] Arts allocation step
- [ ] Magus wizard flow (extends the M4 wizard framework)
- [ ] Additional V/F data for Hermetic category
- [ ] Magus life stages that spend Art XP (so they land here, not in M3):
      apprenticeship (240 xp across Arts + Abilities, 120 spell levels) and
      after-apprenticeship accrual (30 pts/year across Arts, Abilities, spells).
      Source: Core Rules.md:2433-2435, 2467-2471

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

## Current focus: Milestone 4

Milestones 0–3 complete, plus the effective-score layer (3d) pulled forward from
M4. The Tauri app builds and launches, loads the ruleset (virtues/flaws,
characteristics, abilities) from bundled resources, validates live, round-trips
canonical saves, and passes a real-binary tauri-driver e2e. Characters carry
point-buy Characteristics, whole bought Ability scores, and an `unspent_xp` bank,
edited through direct-entry components; score-boosting Virtues (Puissant Ability,
Great Characteristic) now show an effective score and gate `AbilityMin`/caps.

Next: the guided creation wizard (M4) wraps the full phase list and reuses the
M2/M3 direct-entry components — and lands the items still deferred out of M3: the
life-stage XP engine (which feeds the bank) and the Sample Childhood packages.
