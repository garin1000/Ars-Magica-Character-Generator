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
wizard (M5) has real phase content to orchestrate. No wizard yet.

Decisions made during M3 (see `crates/arm-rules/RULES.md` and the plan archive):
abilities store the **whole bought score + a single `unspent_xp` bank**, not
per-ability XP (ability XP is spent in whole points, so loose XP lives in the
bank; the effective score = bought + virtue bonuses is computed, never stored).
The effective-score (virtue-bonus) layer was pulled forward into M3 (3d); the
life-stage XP flow is deferred to the M5 wizard; `House`/`ArtMin` evaluation and
the Art registry move to M4 (4a/4b).

### 3a. Engine models (TDD) — DONE
- [x] Characteristics model (Int, Per, Str, Sta, Pre, Com, Dex, Qik) with point-buy
      (`characteristics.rs`; cost table is data in `characteristics.json`)
- [x] Abilities model + 5 categories + XP advancement table (`ability.rs`)
- [x] Ability registry in the `Ruleset` (`from_core_json`) so `ability` parameter
      refs and `AbilityMin` resolve at load
- [x] Characteristic scores + whole ability scores + `unspent_xp` bank on `Entity`
      (schema_version 1→2, additive)
- [x] Evaluate `AbilityMin` against the entity's max bought ability score
      (`validation.rs`); `House`/`ArtMin` remain deferred (M4)
- [~] Life-stage XP acquisition (early childhood, 15/20/10 per year, age→max cap)
      — DEFERRED to the M5 wizard that drives it. Source: Core Rules.md:2364-2394

### 3b. Data — DONE
- [x] Characteristics core data (`characteristics.json`); labels in Fluent (enum)
- [x] Seed Abilities catalogue (`abilities.json`, 23 abilities across all 5
      categories + full childhood restricted list) + i18n (en, de)
- [~] Sample Childhood packages — DEFERRED to the M5 wizard (built with the
      childhood model, loading, integrity, and apply-flow, rather than shipping inert unvalidated
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
- Deferred follow-ups, now both scheduled for **M4/4f**: *Improved Characteristics*
  (+3 point-buy pool, a budget modifier — not an effective bonus) and *Puissant
  Art* (+3, which needed the Art registry — moved forward to M4/4a with it).

## Milestone 4 — Complete input for all character types (direct entry)

Scope: make every one of the four character types — grog, companion, mythic
companion, magus — fully buildable in direct-validated and direct-unchecked
modes, including Arts, spells, Houses, Hermetic V/F, and all creation-relevant
V/F effect mechanics. Engine → data → direct-entry slices (mirroring M1→M3). No
wizard yet — the guided flow (M5) wraps these components afterward.

In scope: Arts, spells, Houses + specialisations, Hermetic V/F, all
creation-relevant V/F effect mechanics (see 4f), Confidence, Personality Traits,
Reputations, age + age→cap. Deferred (sheet/export, M8+): lab totals,
casting/spell totals, Twilight, Longevity Ritual, Decrepitude-in-play, and the
in-play-only V/F effects (Magical Focus, Method Caster, study Source-Quality
bonuses, Deficient Technique/Form total-halving) — these don't change a creation
number.

### 4a. Arts (engine + data + direct entry)
- [ ] Arts data model: whole bought Art score + a banked Art-XP total; effective
      score = bought + bonuses, computed never stored (mirrors the Ability
      `xp_pool` precedent)
- [ ] Art registry in the `Ruleset` so `art` parameter refs and `ArtMin` resolve;
      make `ParameterDomain::Art` registry-backed (replace the `true` stub in
      `validation.rs`), un-skip the integrity check in `ruleset.rs`, and evaluate
      `ArtMin` against the effective Art score
- [ ] Puissant Art (+3): `art_bonus` effect (deferred from M3d "until the Art
      registry"); extend `validate_effect_refs` to accept an `art`-domain param.
      Source: Core Rules.md:4818-4820
- [ ] `rules/core/arts.json` + i18n (en, de): all 15 Arts (5 Techniques + 10
      Forms). Technique/Form is the fixed taxonomy (enum surfaced via the
      ruleset); Art entries are data — assert structural invariants, never counts
- [ ] Direct-entry Art allocation component (whole-score steppers + Art-XP bank),
      labelled via Fluent keys (`art-<id>`)

### 4b. Houses, House specialisations & Hermetic V/F
- [ ] House data model + registry (id, free Minor House Virtue, optional named
      specialisations/lineages); evaluate the `House` prereq (un-skip the
      deferral in `validation.rs`)
- [ ] `rules/core/houses.json` + i18n: all 12 core Houses with their free Virtues.
      Source: Core Rules.md:2270-2283, 2859, 10986
- [ ] Specialisation choice that determines the free Virtue, data-driven (no House
      IDs hardcoded in engine logic):
      - Bonisagus → Bonisagus (Puissant Magic Theory) vs Trianomae (Puissant
        Intrigue). Source: Core Rules.md:2271; Houses of Hermes — True Lineages.md:509, 531
      - Mercere → Puissant Creo or Muto. Source: Core Rules.md:2278; True Lineages.md:2975
      - Flambeau → Puissant Ignem or Perdo. Source: Core Rules.md:2275; Societates.md:1134
      - Jerbiton → a choice of Minor Virtue. Source: Core Rules.md:2277
      - Ex Miscellanea → free Minor Hermetic Virtue + free Major non-Hermetic
        Virtue + compulsory Major Hermetic Flaw. Source: Core Rules.md:2274
      - Mystery Houses (Bjornaer, Criamon, Merinita, Verditius) grant a Virtue
        that also sets a starting Supernatural-Ability score of 1. Source:
        Core Rules.md:4061, 3761, 3827, 5217; Mystery Cults.md:1378
- [ ] House selection + specialisation as direct-entry fields; selecting them
      auto-grants the free Minor House Virtue (engine/data, so direct entry yields
      a legal magus). The guided step wrapper is M5
- [ ] `virtue_category_caps` (new, data-driven, mirroring `flaw_category_caps` /
      `FlawCategoryCap`) to enforce "≤1 Major Hermetic Virtue". Source: Core Rules.md:2857
- [ ] Additional Hermetic-category V/F data (the special magus Virtues/Flaws)

### 4c. Spells (engine + data + direct entry)
- [ ] Spell data model (technique + form + level; depends on the 4a Art registry
      for T+F refs)
- [ ] Spell-levels budget concept for direct validation of a magus's spell list
- [ ] `rules/core/spells.json` seed + i18n (full catalogue stays M7; German spell
      names must follow the curated translation tables)
- [ ] Spell direct-entry component (add/remove, pick T+F+level), Fluent-labelled

### 4d. Character types: profiles, selector & type-specific V/F + Ability rules
- [ ] All four type profiles in `character_types.json` with budgets/caps/required
      & forbidden traits. Source: Core Rules.md:2205-2222, 2293-2305, 2633-2639, 2855-2859
      - Grog: ≤3 points, Minor only, no Story Flaws, no Gift. Source: Core Rules.md:2295
      - Companion: ≤10 points; Hermetic V/F only with The Gift (free). Source: Core Rules.md:2297-2299
      - Mythic companion (new): free Minor "status" Virtue + 10 Flaw points where
        each Flaw point is worth 2 Virtue points (max 21 V / 10 F). Source: Core Rules.md:2633-2639
      - Magus (new): must take The Gift + Hermetic Magus (both free), ≥1 Hermetic
        Flaw required, ≤5 Minor Flaws, ≤1 Major Hermetic Virtue, one free Minor
        House Virtue (un-balanced). Source: Core Rules.md:2303, 2855-2859
- [ ] The Gift → one free Supernatural Ability (without the granting Virtue;
      further ones still require the Virtue). Engine validation, available in
      direct entry. Source: Core Rules.md:2874
- [ ] Ability selector reflects selectability of Supernatural Abilities: one the
      character cannot currently take is shown disabled/greyed with the reason
      (e.g. "requires <Virtue>") rather than hidden. The engine surfaces why an
      ability is unavailable; the selector reads that, never hardcoding which
      abilities are supernatural. Source: Core Rules.md:2392, 2874
- [ ] Character-type selector UI (replaces the hardcoded `companion` in
      `App.svelte` / `newEntity()`). Direct-entry sections render Arts/Spells/House
      only for the magus profile, driven by the profile (e.g. `is_magus`), not
      hardcoded. Type labels via Fluent, never the slug

### 4e. Remaining per-character input fields
- [ ] Age field + age → max-Ability-score cap validation: <30→5, 30-35→6, 36-40→7,
      41-45→8, 46+→9 (raised by some Virtues). Validation only — the XP
      acquisition and aging rolls are M5. Source: Core Rules.md:2366-2374
- [ ] Confidence: default Score 1 + 3 points for companions & magi, none for
      grogs, modifiable by V/F. Source: Core Rules.md:2221, 2520-2526
- [ ] Personality Traits: input list, range ±3 (±6 with a Major Personality
      Flaw); grogs should have Loyal, warrior grogs Brave. Source: Core Rules.md:2217-2219, 1071-1089, 2500-2506
- [ ] Reputations: input only when granted by a Virtue/Flaw (score + content +
      type Local/Ecclesiastical/Hermetic). Source: Core Rules.md:2220, 1091-1101, 2512-2518

### 4f. Generalise the V/F mechanical effect model (all creation-relevant families)
The engine today models only two `Effect` kinds (`ability_bonus`,
`characteristic_bonus`). Before the wizard, every V/F mechanic that changes a
creation number must be a data-driven `Effect`, so direct entry builds a correct
character and the XP-bank accounting is right. Extend the `Effect` model +
`validate_effect_refs` (`effective.rs`, `ruleset.rs`) and the
XP-spent-vs-available computation (`validation.rs`). Seed one or two
representative items per family; the full catalogue is data-only (M7).
- [ ] XP-COST modifier — Affinity with (Ability) and Affinity with (Art): XP put
      into the target is increased by half (rounded up) at creation (the target
      is cheaper), and the target may exceed the normal age/recommended cap. The
      XP-spent calc applies the per-target multiplier + cap-exemption. Source:
      Core Rules.md:3372-3374 (Ability), 3376-3378 (Art)
- [ ] XP-GRANT restricted pools — Educated, Warrior, Privileged Upbringing,
      Arcane Lore, Well-Traveled, …: each adds a fixed XP pool spendable only on a
      defined Ability set; the available-XP side tracks restricted pools, not just
      the single `unspent_xp` bank. Seed Educated/Warrior/Privileged Upbringing;
      the ~30-item long tail is M7 data. Source: Core Rules.md:3711-3713,
      5227-5229, 4806-4808, 3430-3432, 5239-5241
- [ ] Flat ART score bonus — Puissant Art (+3): the `art_bonus` effect from 4a,
      alongside the existing Puissant Ability (+2). Source: Core Rules.md:4818-4820, 4814-4816
- [ ] POINT-BUY pool — Improved Characteristics (+3 to the Characteristics pool,
      stackable); deferred from M3d. Source: Core Rules.md:4103-4105
- [ ] STARTING-SCORE grant — `ability_score_grant` effect: a V/F that confers an
      Ability at score 1 (Mystery-House Virtues, overlapping 4b; plus the
      Supernatural-Ability Virtues: Second Sight, Premonitions, Dowsing, Animal
      Ken, …). Seed a couple; full catalogue M7. Source: Core Rules.md:3414-3416,
      4059-4061, 4888-4890
- [ ] CAP composition: Great Characteristic's +5 ceiling already exists (M3d);
      ensure the Affinity cap-exemption and the age→cap (4e) compose correctly
- [ ] NOT modelled here (no creation-number effect; selectable but inert at
      creation): Deficient Technique/Form (halve in-play totals only — Deficient
      Technique stays in the seed as a valid Hermetic Flaw satisfying the magus
      "≥1 Hermetic Flaw" requirement), Magical Focus, Method Caster, and study
      Source-Quality bonuses (Apt Student, Book Learner, Free Study, Independent
      Study, Secondary Insight). Source: Core Rules.md:5909-5915

## Milestone 5 — Guided creation wizard

Scope: wrap the full phase list for every character type in a guided flow,
reusing the direct-entry components from M2–M4. All input surfaces already exist;
this milestone adds orchestration, gating, and the guided life-stage flows.

- [ ] Wizard component driven by the character type's `creation_phases` list
      (already inert data on the profile; the wizard is its first consumer)
- [ ] Phase navigation (next/back/skip where allowed)
- [ ] Per-phase validation gating (enforced mode blocks advancing with errors)
- [ ] Phases wired to their M2–M4 direct-entry components (V/F, characteristics,
      abilities, Arts, spells, House+specialisation)
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
- [ ] Magus life stages that spend Art XP: apprenticeship (240 xp across Arts +
      Abilities, 120 spell levels, min Parma/Magic Theory/Latin) and
      after-apprenticeship accrual (30 pts/year across Arts, Abilities, spells).
      Source: Core Rules.md:2433-2435, 2467-2471
- [ ] Aging engine for characters over 35: aging rolls, Characteristic loss,
      Decrepitude accrual — the guided age/life-stage computation. Source:
      Core Rules.md:16563-16640
- [ ] Guided House+specialisation step and guided Arts allocation step (auto-grant
      logic already in M4)
- [ ] Grog wizard flow (subset of phases)
- [ ] Companion wizard flow complete
- [ ] Mythic companion wizard flow
- [ ] Magus wizard flow

## Milestone 6 — Covenants

- [ ] Boons & Hooks data (same PointItem structure, EntityKind::Covenant)
- [ ] Covenant entity type profile
- [ ] Covenant resources model (Library, Vis, Specialists, etc.)
- [ ] Covenant wizard flow
- [ ] Covenant UI

## Milestone 7 — Full data population

- [ ] Complete V/F catalogue from ArM5 core book
- [ ] Complete Abilities catalogue
      — **staged on the `full-abilities` branch**: the full 78-ability Core Rules
      catalogue (en + de descriptions, specialties, `requires_training` flags) is
      already authored there. The engine, i18n schema, and UI on `main` already
      support it; `main` currently ships only the 23-ability seed set. Pulling it
      over is a data-only widening — re-add the dropped ability entries to
      `rules/core/abilities.json` and `rules/i18n/{en,de}/abilities.json`, then
      restore the `78`/`21`/`50` counts in `tests/data_integrity.rs` and the `78`
      in `crates/arm-app/tests/commands.rs`.
- [ ] Complete Arts text — all 15 Arts ship in M4; M7 adds their descriptions /
      lab text (the mechanics are already complete)
- [ ] Complete Houses detail — all 12 core Houses ship in M4; M7 adds the
      Mystery/Societas House detail and any supplement-only Houses
- [ ] Complete V/F effect-mechanic catalogues — the effect families land in M4/4f
      with seed items; M7 fills the long tails (the ~30 XP-grant Virtues, the
      Supernatural-Ability starting-score Virtues) as data-only widening
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

The plan was reordered so that **all input for all four character types is
possible before the guided wizard is built**. M4 (renamed from "Guided creation
wizard") now completes every input surface — Arts, spells, Houses + their
specialisation/free-Virtue choice, Hermetic V/F, the full set of creation-relevant
V/F effect mechanics (Affinity, restricted XP-grant pools, Improved
Characteristics, starting-score grants, Puissant Art), the magus + mythic-companion
profiles, a character-type selector, age + age→cap, Confidence, Personality Traits,
and Reputations — all in direct-validated/direct-unchecked mode. The guided wizard
moves to **M5**, where it wraps these surfaces and adds the guided life-stage flows
(life-stage XP engine, Sample Childhood packages, magus apprenticeship XP, and the
aging engine for characters over 35). Covenants remain M6.

Next: M4/4a — the Arts engine (registry, Art-XP bank, `ArtMin`/`art`-domain
resolution, Puissant Art) and its direct-entry component.
