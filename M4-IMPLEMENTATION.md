# Milestone 4 — Implementation Tracker

Sequenced execution plan for M4 ("Complete input for all character types — direct
entry"). `PLAN.md` holds the high-level 4a–4f scope checkboxes; this file is the
session-resumable execution tracker with per-phase tasks, gate, and e2e spec.
Phases run in **dependency-optimized order** (not literal 4a→4f). Tick a box here
and the matching `PLAN.md` box in the same commit as green code.

## Required gate (end of every phase, before any "done" claim)

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
cd ui && npm run test:unit && npm run lint && npm run format:check && cd ..
cargo tauri build --no-bundle          # authoritative: type-checks UI + builds release
cd ui && npm run test:e2e              # the phase's new e2e spec, green against the real binary
```

TDD mandatory (red → green → refactor). Cite rules by source basename + line
range at the implementation site; update `crates/arm-rules/RULES.md` in the same
change. Bump `schema_version` when the Entity shape changes; keep canonical
key/array sorting. German labels must match the translation tables.

---

## Phase 1 — Char-type profiles + type selector (PLAN.md 4d-core) ✅
e2e: `ui/e2e/specs/character-types.e2e.js`

- [x] `magus` + `mythic_companion` profiles in `rules/core/character_types.json`
      (budgets/caps/required+forbidden traits, `creation_phases` incl. `arts`,
      `spells`, `house_specialisation`); numbers recorded in `RULES.md`
- [x] capability flag on the profile (`is_magus`) is on every profile; the
      selector + conditional sections read it, never a hardcoded type id
- [x] mythic-companion 2:1 flaw→virtue conversion via data-driven
      `PointBudget.virtue_points_per_flaw_point` in the balance calc
- [x] `CharacterTypeSelector.svelte` replaces hardcoded `companion` default;
      labels via Fluent `type-<id>`; `setType` validates immediately (discrete
      action, not debounced — avoids a stale-result race)
- [x] e2e spec green (3× stable); PLAN.md 4d boxes updated; full gate passes

Notes / deferred to later phases (as planned):
- The e2e asserts what is observable now — the selector switches type and the
  profile's budget (companion 10/10, grog 3/3, mythic 20/10 at 2:1) + category
  rules change. The "Arts/Spells/House sections appear for magus" assertions are
  added by Phases 2/4/5 when those sections exist.
- magus `≤1 Major Hermetic Virtue` (needs `virtue_category_caps`), the free
  House Virtue, and the `≥1 Hermetic Flaw` guideline land in Phase 4.
- the mythic free Minor status Virtue (+1, raising 20→21) is M5 catalogue data;
  the `virtue_points: 20` ceiling is the balanced max without it.

## Phase 2 — Arts (PLAN.md 4a) ✅
e2e: `ui/e2e/specs/arts.e2e.js`

- [x] Art data model: whole bought score; effective computed (`ArtScore`,
      `Entity::art_scores`; schema_version bumped 2→3). Arts draw from the shared
      `xp_pool` alongside Abilities (one apprenticeship bank, per the rules)
- [x] Art registry in `Ruleset`; `ParameterDomain::Art` registry-backed (dropped the
      `true` stub in `validation.rs`); un-skipped Art integrity check in `ruleset.rs`;
      `ArtMin` against effective score
- [x] Puissant Art (+3) `art_bonus` effect; `validate_effect_refs` accepts an
      `art`-domain param
- [x] `rules/core/arts.json` + i18n (en, de): 15 Arts (structural invariants only)
- [x] `ArtGrid.svelte` — all 15 Arts always shown (no pick step) as the sheet's
      three columns (Techniques | Forms 1-5 | Forms 6-10) with score steppers +
      effective badge; `ArtXpBar` shows the shared pool. Arts tab gated on the
      profile `is_magus`; ParameterPicker grows an `art`-domain select so Puissant
      Art targets a catalogue Art
- [x] e2e spec green (3× stable); PLAN.md 4a box updated; gate passes

Notes:
- Abilities and Arts share **one** `xp_pool` (the rules' single apprenticeship
  bank); `validate_xp_pool` checks the combined Ability + Art cost. Arts still
  price from the cheaper triangular curve (`art_advancement`).
- Form bonus (Form score/5) is a derived combat stat — deferred (M5/5i).

## Phase 3 — V/F mechanical effect model (PLAN.md 4f) ✅
e2e: `ui/e2e/specs/vf-effects.e2e.js`

- [x] XP-COST modifier — Affinity with (Ability)/(Art): creation XP "counts as
      1½×", modelled as `charged = ceil(table_xp·2/3)` (verified vs the Perdo
      37→56 worked example `:2443`). Two effect variants (`affinity_ability_cost`
      / `affinity_art_cost`) keep the one-domain-per-effect invariant
- [x] XP-GRANT restricted pools — Educated/Warrior/Privileged Upbringing
      (`restricted_ability_xp`, eligible by ability id OR category). Feasibility
      is a bipartite **max-flow** in `effective.rs::xp_allocation` (greedy is
      wrong under overlapping eligibility); leftover restricted XP → non-blocking
      `restricted_xp_unspent` warning
- [x] POINT-BUY pool — Improved Characteristics (`characteristic_points` +3,
      stackable); `validate_characteristics` budget = `start_points + granted`
- [x] STARTING-SCORE grant — `ability_score_grant` (fixed ability id, free
      floor, 0 XP); seeded Second Sight + Premonitions
- [x] Affinity cap-exemption is implicit (Phase-6 age-cap reads the effect's
      presence); permission-unlock (Academic/Martial purchasable) deferred
- [x] e2e spec green (3× stable); PLAN.md 4f box updated; full gate passes

Notes:
- Engine surfaces the authoritative XP spend (`xp_total_demand`, per-pool
  `used`/`amount`) and the granted floors via `EffectiveScores`; the XP bars and
  ability rows read those (no Affinity/flow recompute in TS). Restricted-pool
  labels go through Fluent (`ability-category-<id>`), never raw slugs.
- Latin is the parameterized `ability.dead_language`; Educated lists it +
  `ability.artes_liberales` (any Dead Language qualifies — a seed approximation).
- Full ability-grant UI (auto-conferring the ability, Gift→Supernatural) lands
  with the supernatural-ability work in Phase 6; Phase 3 shows the floor on a
  manually-added granted ability.

## Phase 4 — Houses, specialisations & Hermetic V/F (PLAN.md 4b)
e2e: `ui/e2e/specs/houses.e2e.js`

- [x] House data model + registry; evaluate the `House` prereq (un-skip)
- [x] `rules/core/houses.json` + i18n: 12 core Houses + free Virtues
- [x] data-driven specialisation→free-Virtue (incl. Mystery Houses seeding a
      Supernatural Ability at 1 via `ability_score_grant`)
- [x] `virtue_category_caps` (≤1 Major Hermetic Virtue); special-magus V/F data
- [x] `HouseSelector.svelte`; selecting auto-grants the free Minor House Virtue
- [x] e2e spec green; PLAN.md 4b box updated; gate passes

## Phase 5 — Mythic Companion types & free V/F system (PLAN.md 4d)
e2e: `ui/e2e/specs/mythic-companion.e2e.js`

**Status: DONE.** Engine, data, UI, and e2e complete; full gate + release build
+ `mythic-companion.e2e.js` green (11/11 suite). Committed across `cdabf2e`
(engine+plumbing), `699d43b` (sourced data), and the UI/enable commit.
Historical note (engine+plumbing landed first via `cdabf2e` with a dormant empty
registry). Done: `grant.rs` extraction (`HouseGrant`→`Grant`, shared
`resolve_grants`/`open_pick_satisfies`); `mythic_companion.rs`
(`MythicCompanionType`, `RequiredFlaw`, per-type `bonus_flaw_points` /
`bonus_free_virtue_points`); ruleset registry + `validate_mythic_type_refs`;
`Entity.mythic_type`/`mythic_choices` (schema 4→5); `effective::entity_grants`
union; `has_mythic_type` flag; effective-budget in `validate_balance`;
`validate_mythic_type` + issue codes + Fluent keys; `ruleset_io` load path.
**Remaining:** (a) author the 4 types + ~16 sourced V/F + 3 abilities + en/de
i18n + RULES.md — provenance corrections already found: **Tragic Life is `Major,
Story` (Tainted), not Supernatural** (Core:6855); Devil Child is `Special`/free
(Infernal:4144); Demonic Blood `Major, Supernatural, Tainted` (Infernal:4116);
authoritative per-type packages live in the RoP "Mythic Companions: X" sections
(Divine:3485, Faerie:6692, Magic: search "Votary"). (b) UI: `types.ts`, store
`setMythicType`/`setMythicRequiredFlaw`/`setMythicChoice`, a
`MythicCompanionTypeSelector.svelte` (cloned from `HouseSelector`, with a swap
dropdown per required flaw), App `mythic_type` tab gated on `has_mythic_type`,
Fluent chrome. (c) `mythic-companion.e2e.js`. Then set `has_mythic_type: true`
+ a `mythic_type` creation phase on the `mythic_companion` profile.


Parallels the Magus House system (Phase 4): a Mythic Companion's *type* (Devil
Child, Faerie Doctor, Nephilim, Spirit Votary) is a data-driven profile that
grants a free "status" Virtue **plus** a free Minor Virtue and imposes a required
V/F package — the same auto-grant + free-Virtue machinery Houses already use
(`granted_selections`, `setHouse`/`setHouseChoice`). Phase 1 shipped only the
mythic budget ceiling (20 V / 10 F at 2:1) and deferred the free status Virtue to
"M5 catalogue data"; this phase pulls it forward so mythic-companion input is
genuinely complete in M4 (the milestone's own promise: "complete input for all
character types").

- [x] Mythic-companion-type data model + registry (`mythic_companion.rs`,
      mirrors the House registry via the shared `grant.rs`): each type carries its
      free status Virtue, its free Minor Virtue, and its required V/F package.
      Source: Core Rules.md:2635-2639
- [x] Free "status" Virtue: 0-cost, mutually incompatible, incompatible with The
      Gift (symmetric `incompatible_with` web incl. `virtue.the_gift`); not for
      grogs (grants only via the mythic type, which grogs cannot pick).
      Source: Core Rules.md:2637
- [x] Auto-grant the type's free Minor Virtue (point-free grant; a `choice`
      free-Minor defaults to its first option); reuses the shared grant path.
      Source: Core Rules.md:2638, 2847
- [x] Required V/F packages count against the budget, incl. per-type bonus points
      folded into the balance ceilings (Devil Child +3 V / +7 F; Spirit Votary
      +7 F per RoP Magic:5486; Nephilim/Faerie Doctor none). Required Flaws are
      swappable for a substitute (advisory). Source: Core Rules.md:2638, 2664, 2731
- [x] `rules/core/mythic_companion_types.json` + i18n (en, de): the 4 core types
      with their packages; ~19 required/free V/F seeded structurally from Core +
      RoP Infernal/Divine/Faerie/Magic with verified citations (Tragic Life
      corrected to Major *Story*), full supernatural effects M9 (supplement). Source:
      Core Rules.md:2643-2765 + RoP books (see RULES.md)
- [~] Mythic-companion V/F guidelines: ≤5 Minor Flaws / ≤1 Story / ≤2 Personality
      (≤1 Major) enforced via the Phase-1 mythic profile budget caps; **≥1 Social
      Status is NOT enforced** (deferred — a guideline; the status Virtue is a
      grant, and no ≥1-category-count rule exists yet). Source: Core Rules.md:2842-2851
- [x] `MythicCompanionTypeSelector.svelte` (mirrors `HouseSelector`) + store
      `setMythicType`/`setMythicChoice`/`setMythicRequiredFlaw`, gated on the
      profile's `has_mythic_type` flag (never a hardcoded id); balance bar shows
      the engine-authoritative effective ceilings
- [x] e2e spec `mythic-companion.e2e.js` green (11/11 suite); PLAN.md 4d box
      updated; full gate + release build passes

Notes:
- Supersedes the Phase-1 deferral ("the mythic free Minor status Virtue … is M5
  catalogue data"): the free status/Minor Virtue mechanism lands here; only the
  long tail of core Supernatural-Virtue *effects* remains M5 (5a).
- The ~90-xp minimum-Ability set (Core Rules.md:2639) is a *guided* constraint
  and stays with the magus 90-xp min-ability work in M6, not here.
- Supplement Supernatural abilities (Demonic Might, Curse-Throwing, Blood of the
  Nephilim) whose full mechanics live in Realms of Power books are seeded
  structurally; their in-play effects are out of scope until those sources exist.

## Phase 6 — Spells (PLAN.md 4c) ✅
e2e: `ui/e2e/specs/spells.e2e.js`

**Status: DONE.** Engine, data, UI, and e2e complete; full gate + release build +
`spells.e2e.js` (7 tests) green, full e2e suite 12/12 files.

- [x] Spell data model (T+F+level; uses Phase 2 Art registry) + spell-levels budget.
      `spell.rs` (`Spell`, `SpellsFile`), `Entity::spells` (`SpellSelection`,
      schema 5→6), spells registry + `validate_spell_refs` in `ruleset.rs`,
      `validate_spells` (budget + per-spell cap + 5 issue codes) in `validation.rs`
- [x] `rules/core/spells.json` seed (14 spells, verified Core line ranges) + i18n
      (en, de); German names per `zauber-nach-form.md`. Skilled/Weak Parens V/F
      added with `spell_levels` + `general_xp` effects (both budgets), per user
      directive that all V/F effects be implemented in full
- [x] `SpellPicker.svelte` (Technique/Form filter, add/remove, General-level input,
      spell-levels bar), Fluent-labelled; Spells tab gated on `is_magus`
- [x] e2e spec green; PLAN.md 4c box updated; gate passes

Provenance facts (verified): magus spell budget **120 levels** (Core:2215-2216,
2435); per-spell cap **Tech + Form + Int + Magic Theory + 3** (Core:2465); General
spells learned at a chosen level, different levels are different spells
(Core:12349-12353); **Skilled Parens** Minor Hermetic +60 XP/+30 levels
(Core:4964-4966), **Weak Parens** Minor Hermetic −60/−30 (Core:7072-7074).

Architecture notes for Phase 7+: two new ref-free `Effect` variants (`SpellLevels`,
`GeneralXp`, signed, summed + clamped at 0) fold into `effective::spell_levels_budget`
and `xp_allocation`'s `general_pool` respectively; `PointItem.effects` being a
`Vec` lets one V/F carry both. Requisite-Art reduction in the per-spell cap is a
documented M4 approximation (out of scope; requisites stored for display only).

## Phase 7 — Gift/supernatural rules + per-character fields (PLAN.md 4d-rest, 4e) ✅
e2e: `ui/e2e/specs/character-fields.e2e.js`

**Status: DONE — M4 COMPLETE.** Engine, data, UI, e2e; full gate + release build
+ e2e 13/13 spec files green. All five features shipped in one phase.

- [x] The Gift → one free Supernatural Ability (further ones need the Virtue; a
      magus gets none — his free one is Hermetic magic). `validate_supernatural_abilities`
      + `effective::supernatural_free_slots`; companion `gift_policy` flipped
      `forbidden`→`allowed` (Core:2872) so a Gifted companion is legal
- [x] Ability selector greys an unavailable Supernatural Ability ("requires a
      Virtue"); `EffectiveScores.supernatural_free_total/used` surfaced, `AbilityPicker`
      reads them + selected virtues' grant effects (synchronous, no hardcoding)
- [x] Age field + age→max-Ability-score cap (Core:2366-2376); Affinity Ability
      exceeds by +2 (Core:3374), not exempt; surfaced as `age_ability_cap`
- [x] Confidence derived (type default 1/3, grog none) + `ConfidenceBonus`
      (Self-Confident → 2/5); read-only readout, hidden for grogs
- [x] Personality Traits ±3, widened to ±6 per selected Major Personality Flaw;
      grog Loyal/warrior Brave soft rule deferred to M6
- [x] Reputations gated on a granting V/F (`GrantsReputation`; Infamous/Black
      Sheep seeded, Local type); `ReputationType` = Local/Ecclesiastical/Hermetic
- [x] e2e spec green (6 tests); PLAN.md 4d-rest+4e boxes updated; gate passes

Provenance/architecture facts (verified): two ref-free `Effect` variants
`ConfidenceBonus {score,points}` + `GrantsReputation {kind,score}`; `ReputationType`
enum (fixed taxonomy like `ArtType`); Confidence is **derived, never stored**
(profile default + effects); the Supernatural "covered = has an `ability_score_grant`
floor" is a documented proxy for "has a granting Virtue" (exact for the seed;
`animal_ken` has no granting Virtue → free-slot only); `has_the_gift` extracted
and shared by `validate_gift_policy` + the free-slot count. schema 6→7.

---

## End-of-M4 acceptance ✅

All four character types (grog, companion, mythic companion, magus) are now fully
buildable in direct entry — Characteristics, Abilities, Arts, Spells, Houses,
Mythic types, V/F with the full effect model, age/Confidence/Personality/
Reputations, and the Gift/Supernatural gate — validated live, saved and reloaded
canonically, verified end-to-end through the real binary (e2e 13/13 spec files).
M5 (full mechanical & data completeness) is next — closing the effect-wiring,
catalogue, and derived-total gaps so any core-rules character is fully enterable
and computable. The guided wizard (M6) then wraps these direct-entry surfaces with
the life-stage XP flows.
