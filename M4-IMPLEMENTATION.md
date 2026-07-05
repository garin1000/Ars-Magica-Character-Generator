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
- the mythic free Minor status Virtue (+1, raising 20→21) is M8 catalogue data;
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
- Form bonus (Form score/5) is a derived combat stat — deferred (M5+).

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

Parallels the Magus House system (Phase 4): a Mythic Companion's *type* (Devil
Child, Faerie Doctor, Nephilim, Spirit Votary) is a data-driven profile that
grants a free "status" Virtue **plus** a free Minor Virtue and imposes a required
V/F package — the same auto-grant + free-Virtue machinery Houses already use
(`granted_selections`, `setHouse`/`setHouseChoice`). Phase 1 shipped only the
mythic budget ceiling (20 V / 10 F at 2:1) and deferred the free status Virtue to
"M8 catalogue data"; this phase pulls it forward so mythic-companion input is
genuinely complete in M4 (the milestone's own promise: "complete input for all
character types").

- [ ] Mythic-companion-type data model + registry (mirrors the House registry):
      each type carries its free status Virtue, its free Minor Virtue, and its
      required V/F package. Source: Core Rules.md:2635-2639
- [ ] Free "status" Virtue: 0-cost, mutually incompatible, incompatible with The
      Gift, forbidden to grogs (reuse the symmetric `incompatible_with` +
      profile permitted/forbidden check). Source: Core Rules.md:2637
- [ ] Auto-grant the type's free Minor Virtue (un-balanced, +1 → 21 V ceiling);
      reuses the Phase-4 free-Virtue grant path. Source: Core Rules.md:2638, 2847
- [ ] Required V/F packages count against the budget, incl. per-type bonus points
      (e.g. Devil Child +3 V / +7 F to offset the compulsory Major Flaw; Nephilim
      5 F required + 5 more granting 10 V). Source: Core Rules.md:2638, 2664, 2731
- [ ] `rules/core/mythic_companion_types.json` + i18n (en, de): the 4 core types
      with their packages; required Supernatural Virtues seeded as they exist in
      the core V/F chapter, long tail / full effects M8 (same graceful degradation
      as Mystery-House abilities). Source: Core Rules.md:2643-2765, 3329
- [ ] Mythic-companion V/F guidelines: ≥1 Social Status, ≤5 Minor Flaws, ≤1 Story
      Flaw, ≤2 Personality Flaws (≤1 Major). Source: Core Rules.md:2842-2851
- [ ] `MythicCompanionTypeSelector.svelte` (mirrors `HouseSelector`), gated on the
      profile — shown when the selected type is `mythic_companion`, read off the
      profile, never a hardcoded id
- [ ] e2e spec green; PLAN.md 4d box updated; gate passes

Notes:
- Supersedes the Phase-1 deferral ("the mythic free Minor status Virtue … is M8
  catalogue data"): the free status/Minor Virtue mechanism lands here; only the
  long tail of Supernatural-Virtue *effects* remains M8.
- The ~90-xp minimum-Ability set (Core Rules.md:2639) is a *guided* constraint
  and stays with the magus 90-xp min-ability work in M5, not here.
- Supplement Supernatural abilities (Demonic Might, Curse-Throwing, Blood of the
  Nephilim) whose full mechanics live in Realms of Power books are seeded
  structurally; their in-play effects are out of scope until those sources exist.

## Phase 6 — Spells (PLAN.md 4c)
e2e: `ui/e2e/specs/spells.e2e.js`

- [ ] Spell data model (T+F+level; uses Phase 2 Art registry) + spell-levels budget
- [ ] `rules/core/spells.json` seed + i18n (German names per translation tables)
- [ ] `SpellPicker.svelte` (add/remove, pick T+F+level), Fluent-labelled
- [ ] e2e spec green; PLAN.md 4c box updated; gate passes

## Phase 7 — Gift/supernatural rules + per-character fields (PLAN.md 4d-rest, 4e)
e2e: `ui/e2e/specs/character-fields.e2e.js`

- [ ] The Gift → one free Supernatural Ability (further ones need the Virtue)
- [ ] Ability selector greys an unavailable Supernatural Ability with the reason
      (engine surfaces *why*; selector reads it, no hardcoding)
- [ ] Age field + age→max-Ability-score cap (<30→5 … 46+→9), composes with caps
- [ ] Confidence default by type (1+3 companion/magi, none grog), V/F-modifiable
- [ ] Personality Traits ±3 (±6 with Major Personality Flaw); grog Loyal/Brave
- [ ] Reputations input only when granted by a V/F (score + content + type)
- [ ] e2e spec green; PLAN.md 4d+4e boxes updated; gate passes

---

## End-of-M4 acceptance

Build a legal character of each of the four types in direct-validated mode (and
the illegal-then-fixed flow under each ValidationMode), save, reload, confirm the
canonical JSON round-trips — end-to-end through the real binary.
