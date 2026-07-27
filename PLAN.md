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
wizard (M6) has real phase content to orchestrate. No wizard yet.

Decisions made during M3 (see `crates/arm-rules/RULES.md` and the plan archive):
abilities store the **whole bought score + a single `unspent_xp` bank**, not
per-ability XP (ability XP is spent in whole points, so loose XP lives in the
bank; the effective score = bought + virtue bonuses is computed, never stored).
The effective-score & characteristic buy-limit layer (Puissant Ability's effective
bonus; Great/Poor Characteristic shifting the buy cap/floor) was pulled forward
into M3 (3d); the life-stage XP flow is deferred to the M6 wizard; `House`/`ArtMin`
evaluation and the Art registry move to M4 (4a/4b).

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
      — DEFERRED to the M6 wizard that drives it. Source: Core Rules.md:2364-2394

### 3b. Data — DONE
- [x] Characteristics core data (`characteristics.json`); labels in Fluent (enum)
- [x] Seed Abilities catalogue (`abilities.json`, 23 abilities across all 5
      categories + full childhood restricted list) + i18n (en, de)
- [~] Sample Childhood packages — DEFERRED to the M6 wizard (built with the
      childhood model, loading, integrity, and apply-flow, rather than shipping inert unvalidated
      data). Source: Core Rules.md:2380-2388

### 3c. Direct-entry UI — DONE
- [x] Characteristic point-buy component with live points readout + validation
- [x] Ability allocation component (whole-score steppers + specialty + banked-XP
      field) with live validation

### 3d. Effective-score & characteristic buy-limit layer (TDD) — DONE
Pulled forward from M4: direct entry already lets a character take score-boosting
Virtues and characteristic-limit Virtues/Flaws, so effective scores and buy
limits must be correct now (display, `AbilityMin`, caps/floors). No wizard
dependency.
- [x] Data-driven `Effect` model on `PointItem` (`ability_bonus` /
      `characteristic_limit`) + `max_per_target`; new
      `ParameterDomain::Characteristic`. Targets named by the selection's param
      value — no Virtue IDs in engine code.
- [x] `effective.rs`: ability effective score = bought + virtue bonuses;
      characteristic cap/floor = base bound widened by limit shifts, clamped to
      the `effective_max`/`effective_min` ceiling. Computed, never stored.
- [x] Puissant Ability (+2 effective, ≤1/Ability via the default
      `max_per_target` of 1); `AbilityMin` uses the effective score.
      Source: Core Rules.md:4814-4816
- [x] Great Characteristic — buy-limit shifter (+1 to the cap; base must already
      be at +3; ≤2/Characteristic; raises the buy cap up to the +5 `effective_max`
      ceiling; grants no effective bonus). Source: Core Rules.md:3987-3989
- [x] Poor Characteristic — buy-limit shifter (−1 to the floor; base must already
      be at −3; ≤2/Characteristic; lowers the buy floor down to the −5
      `effective_min`). Source: Core Rules.md:6598-6600
- [x] Multiplicity generalized in `validate_duplicate_selections` (default
      1/target, grouped by `(item, params)`); parameter-relative precondition in
      `validate_characteristic_limit_preconditions`; effect integrity at load
      (`ruleset.rs`)
- [x] UI: read-only "effective" badge beside abilities (Puissant); characteristic
      steppers widen their min/max from `characteristic_caps` /
      `characteristic_floors`; Great/Poor Characteristic target picker;
      Fluent + de i18n; real-binary e2e
- Deferred follow-ups, now both scheduled for **M4/4f**: *Improved Characteristics*
  (+3 point-buy pool, a budget modifier — not an effective bonus) and *Puissant
  Art* (+3, which needed the Art registry — moved forward to M4/4a with it).

## Milestone 4 — Complete input for all character types (direct entry)

Scope: make every one of the four character types — grog, companion, mythic
companion, magus — fully buildable in direct-validated and direct-unchecked
modes, including Arts, spells, Houses, Hermetic V/F, and all creation-relevant
V/F effect mechanics. Engine → data → direct-entry slices (mirroring M1→M3). No
wizard yet — the guided flow (M6) wraps these components afterward.

In scope: Arts, spells, Houses + specialisations, Hermetic V/F, all
creation-relevant V/F effect mechanics (see 4f), Confidence, Personality Traits,
Reputations, age + age→cap. Moved to **M5** (the pre-wizard completeness gate):
lab/casting/spell totals, Longevity Ritual, Decrepitude and Warping (points +
their effects), and the in-play-only V/F effects (Magical Focus, Method Caster,
study Source-Quality bonuses, Deficient Technique/Form total-halving) — direct
entry must represent them all before the wizard.

### 4a. Arts (engine + data + direct entry) ✅
- [x] Arts data model: whole bought Art score, bought from the shared `xp_pool`
      (one apprenticeship bank for Abilities + Arts); effective score = bought +
      bonuses, computed never stored
- [x] Art registry in the `Ruleset` so `art` parameter refs and `ArtMin` resolve;
      make `ParameterDomain::Art` registry-backed (replace the `true` stub in
      `validation.rs`), un-skip the integrity check in `ruleset.rs`, and evaluate
      `ArtMin` against the effective Art score
- [x] Puissant Art (+3): `art_bonus` effect (deferred from M3d "until the Art
      registry"); extend `validate_effect_refs` to accept an `art`-domain param.
      Source: Core Rules.md:4818-4820
- [x] `rules/core/arts.json` + i18n (en, de): all 15 Arts (5 Techniques + 10
      Forms). Technique/Form is the fixed taxonomy (enum surfaced via the
      ruleset); Art entries are data — assert structural invariants, never counts
- [x] Direct-entry Art allocation component — all 15 Arts always shown as the
      sheet's three columns (Techniques | Forms ×2) with whole-score steppers,
      labelled from `rules/i18n` Art names

### 4b. Houses, House specialisations & Hermetic V/F
- [x] House data model + registry (id, free Minor House Virtue, optional named
      specialisations/lineages); evaluate the `House` prereq (un-skip the
      deferral in `validation.rs`)
- [x] `rules/core/houses.json` + i18n: all 12 core Houses with their free Virtues.
      Source: Core Rules.md:2270-2283, 2859, 10986
- [x] Specialisation choice that determines the free Virtue, data-driven (no House
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
- [x] House selection + specialisation as direct-entry fields; selecting them
      auto-grants the free Minor House Virtue (engine/data, so direct entry yields
      a legal magus). The guided step wrapper is M6
- [x] `virtue_category_caps` (new, data-driven, mirroring `flaw_category_caps` /
      `FlawCategoryCap`) to enforce "≤1 Major Hermetic Virtue". Source: Core Rules.md:2857
- [x] Additional Hermetic-category V/F data (the special magus Virtues/Flaws)

### 4c. Spells (engine + data + direct entry) ✅
- [x] Spell data model (technique + form + level; depends on the 4a Art registry
      for T+F refs). `spell.rs` `Spell { technique, form, level: Option<u8>,
      requisites }` + `Entity::spells` (`SpellSelection`); `None` level = General.
      Source: Core Rules.md:12329-12358
- [x] Spell-levels budget concept for direct validation of a magus's spell list:
      120 levels (`EntityTypeProfile.spell_levels`) + per-spell cap ≤ Tech + Form
      + Int + Magic Theory + 3, both in `validate_spells`. Skilled/Weak Parens
      modify both the spell budget (`Effect::SpellLevels`) and the general XP pool
      (`Effect::GeneralXp`). Source: Core Rules.md:2215-2216, 2435, 2465, 4964-4966, 7072-7074
- [x] `rules/core/spells.json` seed (14 spells across Creo/Rego × several Forms) +
      i18n (en, de); German names from the `zauber-nach-form.md` table. Full
      catalogue lands in M5 (5d)
- [x] Spell direct-entry component (`SpellPicker.svelte`: Technique/Form filter,
      add/remove, General-level input, spell-levels bar), Fluent-labelled; Spells
      tab gated on the profile `is_magus`

### 4d. Character types: profiles, selector & type-specific V/F + Ability rules
- [x] All four type profiles in `character_types.json` with budgets/caps/required
      & forbidden traits. Source: Core Rules.md:2205-2222, 2293-2305, 2633-2639, 2855-2859
      (magus `≤1 Major Hermetic Virtue` + free House Virtue deferred to 4b)
      - Grog: ≤3 points, Minor only, no Story Flaws, no Gift. Source: Core Rules.md:2295
      - Companion: ≤10 points; Hermetic V/F only with The Gift (free). Source: Core Rules.md:2297-2299
      - Mythic companion (new): free Minor "status" Virtue + 10 Flaw points where
        each Flaw point is worth 2 Virtue points (max 21 V / 10 F). Source: Core Rules.md:2633-2639
      - Magus (new): must take The Gift + Hermetic Magus (both free), ≥1 Hermetic
        Flaw required, ≤5 Minor Flaws, ≤1 Major Hermetic Virtue, one free Minor
        House Virtue (un-balanced). Source: Core Rules.md:2303, 2855-2859
- [x] Mythic Companion *types* & free V/F system (parallels the Magus House
      system, 4b): each type (Devil Child, Faerie Doctor, Nephilim, Spirit Votary)
      is a data-driven profile granting a free "status" Virtue (mutually
      incompatible, incompatible with The Gift, not for grogs) + a free Minor
      Virtue (un-balanced, → 21 V ceiling) + a required V/F package with per-type
      bonus points; type selector gated on the profile. Source: Core Rules.md:2635-2639, 2643-2765, 2842-2851
- [x] The Gift → one free Supernatural Ability (without the granting Virtue;
      further ones still require the Virtue; a magus gets none — his free one is
      Hermetic magic). `validate_supernatural_abilities` +
      `effective::supernatural_free_slots`; companion `gift_policy` flipped to
      `allowed`. Source: Core Rules.md:2872, 2874
- [x] Ability selector greys an unavailable Supernatural Ability with the reason
      ("requires a Virtue"); the engine surfaces the free-slot counts
      (`EffectiveScores.supernatural_free_total/used`) + granting effects, and
      `AbilityPicker` reads them, never hardcoding which abilities are
      supernatural. Source: Core Rules.md:2392, 2874
- [x] Character-type selector UI (replaces the hardcoded `companion` in
      `App.svelte` / `newEntity()`). Type labels via Fluent (`type-<id>`), never
      the slug; the profile `is_magus` flag is wired for conditional sections.
      (The Arts/Spells/House sections themselves render as 4a/4c/4b land.)

### 4e. Remaining per-character input fields ✅
- [x] Age field + age → max-Ability-score cap validation: <30→5, 30-35→6, 36-40→7,
      41-45→8, 46+→9. `Entity.age` + `age_max_ability_score` + the cap check in
      `validate_abilities`; an Affinity Ability may exceed it by +2 (Core:3374),
      not without limit. Validation only — XP acquisition/aging is M6. Virtue
      cap-raisers deferred. Source: Core Rules.md:2366-2376
- [x] Confidence: derived (not stored) — default Score 1 + 3 points for companions
      /magi/mythic, none for grogs, modifiable by V/F (`ConfidenceBonus`; Self-
      Confident → 2/5). Surfaced read-only via `EffectiveScores`. Source: Core Rules.md:2520-2526
- [x] Personality Traits: `Entity.personality_traits` (name + ±value) list, range
      ±3, widened to ±6 by a Major Personality Flaw (one trait per flaw). Grog
      Loyal / warrior Brave soft "should" deferred to the M6 guided flow.
      Source: Core Rules.md:2500-2503
- [x] Reputations: `Entity.reputations` (score + content + type
      Local/Ecclesiastical/Hermetic), input only when a V/F grants one
      (`GrantsReputation`; Infamous/Black Sheep seeded). Source: Core Rules.md:1091-1101, 2512-2514

### 4f. Generalise the V/F mechanical effect model (all creation-relevant families) ✅
The engine today models only two `Effect` kinds (`ability_bonus`,
`characteristic_bonus`). Before the wizard, every V/F mechanic that changes a
creation number must be a data-driven `Effect`, so direct entry builds a correct
character and the XP-bank accounting is right. Extend the `Effect` model +
`validate_effect_refs` (`effective.rs`, `ruleset.rs`) and the
XP-spent-vs-available computation (`validation.rs`). Seed one or two
representative items per family; the full catalogue is wired in M5 (5a).
- [x] XP-COST modifier — Affinity with (Ability) and Affinity with (Art): XP put
      into the target counts as 1½× at creation (modelled as a reduced charged
      cost, `ceil(table_xp·2/3)`), and the target may exceed the age/recommended
      cap. `effective.rs::xp_allocation` applies the per-target multiplier;
      cap-exemption is implicit (read off the effect; age→cap is 4e). Source:
      Core Rules.md:3372-3374 (Ability), 3376-3378 (Art)
- [x] XP-GRANT restricted pools — Educated, Warrior, Privileged Upbringing:
      each adds a fixed XP pool spendable only on a defined Ability set
      (`restricted_ability_xp`, eligible by id OR category). The available-XP side
      is a bipartite max-flow over the general pool + restricted pools; leftover
      restricted XP warns (`restricted_xp_unspent`). The long tail is wired in M5 (5a).
      Source: Core Rules.md:3711-3713, 5227-5229, 4806-4808
- [x] Flat ART score bonus — Puissant Art (+3): the `art_bonus` effect (done in
      4a), alongside the existing Puissant Ability (+2). Source: Core Rules.md:4818-4820, 4814-4816
- [x] POINT-BUY pool — Improved Characteristics (`characteristic_points` +3 to
      the Characteristics budget, stackable); deferred from M3d. Source: Core Rules.md:4103-4105
- [x] STARTING-SCORE grant — `ability_score_grant` effect (fixed ability id, free
      floor, 0 XP): seeded Second Sight + Premonitions. Mystery-House grants reuse
      it in 4b; full catalogue M5 (5a). Source: Core Rules.md:4888-4890, 4788-4790
- [x] CAP composition: Great Characteristic's +5 ceiling already exists (M3d);
      the Affinity cap-exemption is carried implicitly so the age→cap (4e)
      composes with it when 4e lands
- [ ] Moved to **M5/5b** — the in-play-only effects (no creation-number change,
      but modelled at full scope there): Deficient Technique/Form (halves in-play
      totals — Deficient Technique stays in the seed as a valid Hermetic Flaw
      satisfying the magus "≥1 Hermetic Flaw" requirement), Magical Focus, Method
      Caster, and study Source-Quality bonuses (Apt Student, Book Learner, Free
      Study, Independent Study, Secondary Insight). Source: Core Rules.md:5909-5915

## Milestone 5 — Full mechanical & data completeness (direct-entry gate)

Scope: before the guided wizard, make **every** core-rules-conforming character
fully enterable AND fully computable in direct entry — no mechanic, catalogue, or
input field deferred past this point. M4 shipped only seed mechanics (~36 of 653
V/F wired); M5 closes the long tail so the wizard (M6) merely orchestrates. Engine
→ data → direct-entry slices (mirroring M1→M4); TDD + `RULES.md` provenance
throughout. Reviewed by a Fable agent (P1–P9 folded in).

Carve-out: the *guided* life-stage/aging derivation stays in M6, but every raw
value it would produce — final scores, Decrepitude/aging points, Warping points,
and the effects those carry — is directly enterable here.

### 5a. V/F audit + creation-effect wiring
- [x] Classify **all** V/F as narrative / creation-effect / in-play-effect with
      per-book `RULES.md` provenance; acceptance = no V/F unclassified. Personality,
      Story, and most Social-Status items are narrative by design — never given
      invented effects. (Done: narrative 445 / creation_effect 115 / in_play_effect 93.)
- [x] Wire the **creation-effect** subset: Supernatural-Ability starting scores,
      fixed-subject Puissant/Affinity, XP-grant Virtues, Confidence,
      size/characteristic deltas, and all reputation-granters (Famous,
      Hermetic Prestige, …). (68 of 79 wired in 5a-wire; **11 deferred** — 4 Might/
      power budgets and 2 nested grants from Realms-of-Power supplements with no
      Core creation number, Masterpiece's Lab-Total-derived item level, and 3
      XP-rate Virtues that are multiplicative or per-year — see RULES.md.)
      **Correction applied:** **Wealthy/Poor are narrative** — Core (`:5235-5237`,
      `:6594-6596`) defines only advancement *seasons* (3 free/yr, −1/yr), no
      creation XP-per-year figure; the invented "20/10 xp/yr" was dropped.
      **Hermetic Prestige reconciled against the file:** it *is* in Core at
      `:4071-4073`, a Hermetic Reputation at level **4** (the `:2518` Darius
      example's 3 is an errata slip); the stale "not in Core / Local-only" claim is
      removed in RULES.md.

### 5b. In-play-only V/F effects modeled at full scope
- [x] `Effect` variants + storage for Magical Focus, Method Caster, Deficient
      Technique/Form, and study Source-Quality bonuses — no creation-number change,
      but represented fully. Acceptance surface = the 5i casting/lab totals consume
      them (the exhaustive `match` enforces wiring). (Done: 13 `Effect` variants
      cover all 18 in-play families; all 93 `in_play_effect` V/F wired.)
      Source: Major/Minor Magical Focus Core Rules.md:4399-4422 / :4536-4542;
      Method Caster :4524-4527; Deficient Technique/Form :5909-5915.

### 5c. Elemental Magic — Art-XP redistribution
- [x] Model the creation-time Art-XP redistribution across the four elemental
      Forms. Requires reconciling with the settled whole-bought-score storage model
      (per-Art XP assignment vs derived leftover) — a design revision, not a routine
      `Effect`. Source: Core Rules.md:3731-3737. (Done as an XP-space `ArtBonus`;
      documented leftover-XP limitation — redistribution uses the table-XP of the
      bought score, so a by-hand assignment with leftover XP won't reproduce exactly.)

### 5d. Full core spell catalogue
- [x] Ship the **full core spell catalogue** (360 spells via a Python extractor,
      `scripts/extract_spells.py`); extend the `Spell` model with a `ritual` flag +
      the ritual creation-legality rule (cited). Source: Core Rules spell list.
      (**Roadmap correction:** the "pull the 78-ability catalogue / restore the
      78/21/50 counts" bullet was **dropped** — the full 78-ability catalogue
      already landed on `main` before M5 (commit `185d064`) and the `full-abilities`
      branch is obsolete; catalogue totals are no longer asserted in tests, per the
      catalogue-size-is-data invariant.)

### 5e. Enchanted devices, familiar, talisman & Longevity entry
- [x] `Entity` storage + direct-entry UI for starting enchanted devices (spending
      the item-level budget), the familiar (bond + Gold/Silver/Bronze cord scores,
      which feed 5i lab/Soak/aging totals), talisman attunements, and a Longevity
      Ritual/potion the character carries — a field (bonus + source), self-made or
      provided by another magus. 5i computes a self-made ritual's bonus; the field
      accepts an externally-provided one.

### 5f. Per-spell mastery input
- [x] UI control to spend the mastery-XP pool on individual spells (storage, pool,
      and floor already exist from M4).

### 5g. Directly-enterable state, effects & identity fields
- [x] Every point-bearing state enterable **with the effects it carries**, not just
      the count: Decrepitude/aging points + the resulting Characteristic reductions;
      Warping points + Twilight scars (schema change). Guided derivation stays M6;
      M5 lets the user type both directly, so an already-warped or already-aged
      character is fully representable. Source: Core Rules aging/warping.
- [x] Identity/flavor fields: name, gender, birth year, Wizard's sigil, covenant
      name, parens.

### 5h. Equipment / weapons / armor / encumbrance
- [x] New `rules/core/equipment.json` weapons/armor table (per-row provenance) +
      direct-entry input surface. Sequenced **before 5i** (feeds Soak/combat/
      encumbrance). Source: Core Rules combat/equipment.

### 5i. Derived totals computed in-engine (read-only)
- [x] In `arm-rules`, from cited source: **magic totals** (casting, penetration,
      lab total, Longevity-ritual bonus — gated on the aura input), then **combat**
      (Init/Atk/Def/Dam), **Soak**, **encumbrance**, Size-derived **wound-penalty
      ranges**, and Decrepitude/Warping **score** from points. Consumes 5b.
      (Done in `derived.rs`; families the app does not simulate — study/aging-roll/
      non-standard-casting/wound-recovery — are **surfaced** as labelled modifiers
      rather than folded into a number.)
- [x] A numeric **aura** input field (covenant auras arrive in M8).
- [x] Rendered in a **main-window read-out panel**; the M7 sheet window later
      re-renders the same computed values (the "UI computes no mechanics" invariant
      holds). Twilight-episode / >35-aging *rolls* are guided (M6) / out of app
      scope; their results are enterable in 5g.

### 5j. Provenance & full gate
- [x] `RULES.md` updated per book; full required gate (`cargo test --workspace`,
      clippy, fmt, `npm run test:unit`/lint/format, and the authoritative
      `cargo tauri build --no-bundle`).

## Milestone 5.5 — Magic Possessions depth (pre-wizard)

Scope: M5/5e shipped only *minimal* entry for the magus's magic possessions —
familiar (name + three cords), talisman (attunements list), Longevity Ritual
(source + externally-provided bonus). Manual testing showed each is thinner than
the rules warrant. This sub-milestone deepens the three to a rules-faithful model
**before** the wizard (M6) wraps them, so the wizard has complete surfaces to
orchestrate. Engine (where a mechanic changes) → data → direct-entry UI, TDD +
`RULES.md` provenance per slice.

### 5.5a. Longevity Ritual — player-entered value + live hint
- [x] The LR aging bonus becomes a **player-entered, stored** value used as-is by
      the engine (a frozen snapshot taken at the ritual's creation). **Drop the
      live Creo Corpus Lab-Total derivation** of the *stored* self-made bonus in
      `derived.rs::longevity_bonus` — raising Cr/Co or changing the aura after
      creation must NOT change the value. The engine passes the stored number
      through (as the `external` path already does); the self/external asymmetry
      collapses to one entered field. `LongevityBonus.entered` keeps an unfilled
      field distinct from a deliberate 0. Source: Core Rules.md:10662, :10668,
      :10670.
- [x] Show a **live hint** beside the input — current Creo Corpus Lab Total →
      suggested bonus (+1 per 5 points) — computed by repurposing the existing
      `creo_corpus_lab_total` / `longevity_bonus` logic as a *hint only*, never as
      the stored value. The player reads it and types the number in. The hint also
      applies the Deficient Creo/Corpus and Difficult Longevity Ritual halvings
      (the first read of `HalvableTotal::LabLongevity`), and the unsupported
      `aura != 0` gate is **removed**. Source: Core Rules.md:10662 (LR bonus),
      :10276-10278 (aura in the Lab Total), :17658 (no aura ⇒ no hindrance),
      :5909-5915, :5962-5964.
- [x] Add a free-text **focus** field (potion / bath / incense / fire …) and
      surface the **sterility** consequence. Source: Core Rules.md:10656.

### 5.5b. Talisman — item identity + instilled effects
- [x] Model the talisman as the magus's personal **enchanted item**, not just a
      list of attunements: a shape/material identity (free-text), an enchantment
      **capacity** read-out (highest Technique + highest Form, in Vim pawns), and
      its **instilled effects** (name + level, like enchanted devices), in
      addition to the existing shape/material **attunements** (description +
      bonus). `Entity.talisman: Option<Talisman>` replaces `talisman_attunements`
      (`SCHEMA_VERSION` 13 → 14, with a legacy fold in `load_entity_migrating`
      that **invents nothing**: attunements are copied verbatim, the identity and
      effects the old shape never stored stay empty, and a legacy value that cannot
      deserialize fails the load rather than folding in silence — see RULES.md
      M5.5b for the shapes it decides). The capacity is **read-only guidance**, and
      instilled effects are charged against **no** budget: `item_level_budget`
      comes only from the Redcap-only Virtues, and a Redcap "may not take The Gift"
      (:4850), so it can never fund a magus's talisman.
      Source: Core Rules.md:10603-10625 (esp. :10619 capacity, :10621 instilling,
      :10623/:10625 attunements), :4347-4349 + :4842-4850 (the budget non-goal).

### 5.5c. Familiar — magical-animal statblock
- [x] Expand `Familiar` from name + three cords into the rulebook's **Creature
      Format** (Core Rules.md:17787-17827), so a save reads like a statblock.
      **Scope decided and built**: `animal` (free text; *not* `species` — the rules
      reserve that word for the Imaginem term), **Magic Might** + realm,
      the eight **Characteristics** (the creature's own, never bought from the
      magus's points), signed **Size** (commonly negative), **Personality Traits**,
      and the bond-invested **powers** — atop the three cords already modeled.
      **Deferred**: the familiar's Abilities, Qualities, Virtues/Flaws and the
      Combat/Soak/Fatigue/Wound statlines — the creature lines the app enters for no
      creature yet. `Characteristic` keeps its fixed eight values: a bound familiar
      gains human intelligence at Int -3 (:10854), so no `Cunning` variant is
      warranted (the unbound creature's "Cun" display is recorded in `RULES.md` as a
      deferred display-only affordance). Every new field is additive
      `serde(default, skip_serializing_if)`, so `SCHEMA_VERSION` stays 14.
      Source: Core Rules.md:10766-10892, esp. :10774 (finding a Magic Might beast),
      :17787-17827 (Creature Format), :17829-17856 (Size examples).
- [x] Derive the bonding numbers as **read-only guidance** (no `ValidationIssue`,
      ever — the `masterpiece_item_cap` contract): the **binding level**
      (Might + 25 + 5 × Size, :10824/:10828), the best **bonding Lab Total**
      (:10818/:10826, reusing the Technique × Form grid — and unlike Masterpiece a
      **focus may apply** here, surfaced as a separate conditional figure), the
      **cord cost curve** 5/15/30/50/75 (:10836, clamped at the +5 maximum), and the
      total level of **invested powers** (:10866 — *no* limit, so deliberately no
      budget bar). Vis costs stay out of scope (the model holds no vis stock).
- [x] Surface the bond's effects as a **note, never auto-applied**: the Minor Virtue
      True Friend, Personality Traits of Loyal (partner) +3, and Intelligence -3 for
      a familiar that lacked human intelligence (:10852-10854). `virtue.true_friend`
      is not in the catalogue and `grant.rs` keys off House/mythic profiles, not off
      a familiar bond.

## Milestone 5.6 — Markdown character export

Scope: a standalone feature — export a built character as a single formatted
**Markdown** file the player can read/share. Independent of the wizard; delivered
here (pulled forward from M10's export item, which retains the PDF/Scribus path).

- [ ] Pure formatter in `arm-rules` (TDD): given entity + localized ruleset, emit
      Markdown covering identity/details, Virtues & Flaws, Abilities, Arts, Spells,
      Equipment, **combat values**, Longevity Ritual, Talisman, and Familiar.
      Include the character's chosen values and the key computed **combat / Soak**
      totals, but **exclude** the per-line derived casting/lab-total read-outs —
      those are noise in an export. Item **names** come through the `rules/i18n`
      layer; no user-facing string is hardcoded (CLAUDE.md data-kind separation).
- [ ] Section-header/label strings for the document: UI-owned Fluent (`.ftl`)
      keys, assembled at the call site — resolve the engine-pure vs. localized-UI
      split (engine emits structure/values keyed by id; UI supplies headings), so
      the `arm-rules` purity invariant holds.
- [ ] Tauri command + native **file-save dialog** + an **Export** button in the
      UI; write the `.md` to a user-chosen path (file IO stays in `arm-app`).
- [ ] Supersedes the Markdown half of M10's "Character sheet export" item.

## Milestone 6 — Guided creation wizard

Scope: wrap the full phase list for every character type in a guided flow,
reusing the direct-entry components from M2–M5. All input surfaces already exist;
this milestone adds orchestration, gating, and the guided life-stage flows.

### 6a. App entry & character-type lock (first M6 slice)

The startup restructure the guided flow plugs into: the app opens on a **choice
screen** rather than an already-instantiated blank character, and a character's
**type is fixed** for its whole life.

- [ ] Startup screen presenting the entry choices: **load existing**, **guided
      wizard** (the rest of M6), and **create new** — one option per character
      type (grog / companion / mythic companion / magus).
- [ ] Character **type is fixed at creation and immutable thereafter** — including
      on load. **Remove the in-view character-type selector** (from M4/4d); nothing
      may change a character's type once chosen.
- [ ] Show the type as a **read-only label** in a sensible spot in the main view
      (e.g. the character banner/header), rendered via Fluent, never a raw slug.
- [ ] Main view otherwise unchanged once a character is open.

### 6b. Guided wizard flow

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

## Milestone 7 — Character sheet window

Scope: an optional, read-only **second app window** that renders a formatted
character sheet and recomputes live as the character is edited in the main
window. Which calculated values appear is **driven by the character-type
profile**, not hardcoded per type. The derived values themselves are computed in
**M5** (`arm-rules`); this window only re-renders them. This is the interactive
in-app view; the static file export (PDF/Markdown) remains M10 and can reuse this
layout.

- [ ] Second Tauri window (label `character-sheet`), created hidden
      (`visible: false`) and opened/closed on demand. A toggle control in the
      main window ("show character sheet") is the only entry point — the window
      is optional. (`crates/arm-app/tauri.conf.json` windows array + a thin
      window command in `crates/arm-app/src/commands.rs`, or `WebviewWindow`
      from JS — currently there is no multi-window code at all.)
- [ ] Separate Vite entry point for the sheet (`ui/sheet.html` +
      `ui/src/sheet.ts` mounting a new `CharacterSheet.svelte` root), wired via
      Vite multi-page `build.rollupOptions.input`. Main window keeps `index.html`.
- [ ] State sync main→sheet: the main window emits the current `entity`
      (plus `lang` and `mode`) on change via Tauri events (`emit`/`listen`);
      the sheet window listens and re-renders. Each Svelte instance has its own
      rune store (`ui/src/lib/state.svelte.ts`), so cross-window state must go
      through events (or a shared backend cache) — events are the idiomatic
      Tauri 2 choice. The sheet is **read-only**: no editing surfaces.
- [ ] Shared sections rendered for every type, reusing existing engine output
      (no new computation): identity + type, Characteristics (score with
      effective cap/floor from `effective_scores`), Virtues & Flaws with point
      balance, Abilities (bought + effective score + specialty), XP pool.
- [ ] Type-dependent sections, gated by the `EntityTypeProfile` (e.g.
      `is_magus`, permitted categories, `creation_phases`) — never an
      `if type == "magus"` ladder in the UI:
      - companion / mythic companion / magus: Personality Traits, Reputations,
        Confidence (all land in M4/M6)
      - magus only: Arts (score + effective), House + specialisation/free Virtue,
        Spells (Technique+Form, level), age→max-score cap
      - grog: minimal subset (Characteristics, Abilities, V/F)
- [ ] All sheet labels via Fluent (`locales/{en,de}/main.ftl`, new `sheet-*`
      keys); no user-facing string hardcoded in Svelte/Rust, and no raw ID/enum
      slug rendered directly — derived-value and section labels map through
      Fluent keys. German labels follow the translation tables.
- [ ] Reuse existing commands (`load_ruleset`, `effective_scores`,
      `validate_entity`) for everything the sheet shows. If a genuinely
      sheet-only derived value is ever required (e.g. a combat/Soak total), the
      rule must first be added to `arm-rules` from the authoritative source with
      a citation — the sheet does not compute mechanics in the UI.
- [ ] Extend the tauri-driver e2e to open the window and assert it reflects an
      edit made in the main window (where feasible).

## Milestone 8 — Covenants

- [ ] Boons & Hooks data (same PointItem structure, EntityKind::Covenant)
- [ ] Covenant entity type profile
- [ ] Covenant resources model (Library, Vis, Specialists, etc.)
- [ ] Covenant wizard flow
- [ ] Covenant UI

## Milestone 9 — Full data population (descriptive text & supplements)

Mechanical completeness — the full V/F effect wiring + audit, the full ability +
spell catalogues, and all creation-relevant and in-play V/F effects — lands in
**M5**. M9 is now purely descriptive breadth and supplement content:

- [ ] Complete Arts descriptions / lab text — the 15 Arts and their mechanics ship
      in M4/M5; M9 adds their prose descriptions
- [ ] Complete Houses detail — all 12 core Houses ship in M4; M9 adds the
      Mystery/Societas House detail and any supplement-only Houses
- [ ] Supplement V/F beyond the core catalogue (the core catalogue and its effect
      wiring are M4/M5)
- [ ] Complete Boons & Hooks
- [ ] All data in en + de (+ additional languages as available)
- [ ] Markdown source files for all rules content

## Milestone 10 — Export & polish

- [ ] Character sheet export — **Markdown delivered in M5.6**; this item now
      covers the **PDF** path, reusing the M7 character-sheet window layout/
      components to render the same sections to a static file. PDF path may use the
      Scribus fillable template — see M11 / `docs/scribus-character-sheet.md`.
- [ ] Covenant sheet export
- [ ] Ruleset versioning & save migration
- [ ] Multiple rulebook/supplement support
- [ ] UI polish, accessibility
- [ ] CI pipeline (cargo test, clippy, fmt, frontend lint, e2e)
- [ ] Release packaging for Windows, macOS, Linux

## Milestone 11 — Scribus fillable-PDF character sheet (bilingual export target)

**Depends on: M6** (wizard complete). Reuses M7 sheet sections + engine derived
values; realizes the M10 PDF-export item via a hand-maintained fillable template.
Full detail, field inventory, and quirks: **`docs/scribus-character-sheet.md`**.

- [ ] Field-ID schema & map (DE `ANNAME` → English-ASCII ID → engine slug)
- [ ] Atomic rename of field names + all `getField()` references
- [ ] Audit & fix the AcroForm JS (math + Scribus calculation order)
- [ ] Extend missing fields vs the official copy-template
- [ ] Bilingual label skins (DE + EN; field names/JS shared)
- [ ] Export integration: arm-char-gen → FDF/XFDF keyed by the English field IDs

---

## Current focus: Milestone 5.6 (Markdown character export)

Milestones 0–5.5 complete: the direct-entry gate is closed — every core-rules
character is fully enterable and its combat/Soak/casting/lab totals computed
in-engine — and the magus's three magic possessions are now modeled at
rules-faithful depth. **M5.5 is done**: the Longevity Ritual stores a
player-entered bonus beside a live Creo Corpus hint (5.5a), the talisman is an
item with identity, capacity and instilled effects (5.5b, `SCHEMA_VERSION`
13 → 14 with a legacy fold that invents nothing), and the familiar is a
Creature-Format statblock with read-only bonding guidance (5.5c, no schema
bump). The guided
wizard **M6** gains a new first slice **6a** (startup screen + fixed character
type) ahead of the guided flow (6b).

Next: M5.6 — Markdown character export (a pure formatter in `arm-rules`,
UI-owned Fluent headings, and a Tauri file-save command), which now has the full
magic-possession content to export.

---

### Superseded focus note (M5, complete)

Milestones 0–4 complete: the Tauri app builds and launches, loads the ruleset,
validates live, round-trips canonical saves, and passes a real-binary tauri-driver
e2e. Every one of the four character types is buildable in direct-validated /
direct-unchecked mode — Characteristics, Abilities (whole score + `unspent_xp`
bank), Arts, spells, Houses + specialisation/free-Virtue, Hermetic V/F, the seed
V/F effect mechanics, the character-type selector, age + age→cap, Confidence,
Personality Traits, and Reputations. But M4 shipped only *seed* mechanics: ~36 of
653 V/F carry an `Effect`, and the full catalogues + derived totals were deferred.

The plan was reordered again (per user directive): **before** the guided wizard,
**everything** needed to enter AND compute any core-rules-conforming character must
be complete — no deferrals. New **M5** (this focus) is that completeness gate: full
V/F effect wiring + audit, in-play-only effects, Elemental Magic, the full ability
+ spell catalogues, enchanted-device/familiar/talisman/Longevity entry, per-spell
mastery, directly-enterable state (Decrepitude/Warping points + their effects) and
identity fields, equipment, and the derived combat/Soak/casting/lab totals. The
guided wizard is now **M6** (it only orchestrates these surfaces + adds the guided
life-stage/aging derivation); character-sheet window **M7**, Covenants **M8**, full
descriptive data **M9**, export **M10**, Scribus **M11**.

Next: M5/5a — the V/F audit + creation-effect wiring across the full catalogue.
