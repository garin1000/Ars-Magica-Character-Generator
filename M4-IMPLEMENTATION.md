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

## Phase 2 — Arts (PLAN.md 4a)
e2e: `ui/e2e/specs/arts.e2e.js`

- [ ] Art data model: whole bought score + banked Art-XP; effective computed
- [ ] Art registry in `Ruleset`; `ParameterDomain::Art` registry-backed (drop the
      `true` stub in `validation.rs`); un-skip Art integrity check in `ruleset.rs`;
      `ArtMin` against effective score
- [ ] Puissant Art (+3) `art_bonus` effect; `validate_effect_refs` accepts an
      `art`-domain param
- [ ] `rules/core/arts.json` + i18n (en, de): 15 Arts (structural invariants only)
- [ ] `ArtPicker.svelte` (steppers + Art-XP bank), Fluent `art-<id>`, shown per flag
- [ ] e2e spec green; PLAN.md 4a box updated; gate passes

## Phase 3 — V/F mechanical effect model (PLAN.md 4f)
e2e: `ui/e2e/specs/vf-effects.e2e.js`

- [ ] XP-COST modifier — Affinity with (Ability)/(Art): +half XP cost, cap-exempt
- [ ] XP-GRANT restricted pools — seed Educated/Warrior/Privileged Upbringing
- [ ] POINT-BUY pool — Improved Characteristics (+3 to char pool, stackable)
- [ ] STARTING-SCORE grant — `ability_score_grant` effect (seed a couple)
- [ ] Puissant Art composes; Affinity cap-exemption composes with age→cap
- [ ] e2e spec green; PLAN.md 4f box updated; gate passes

## Phase 4 — Houses, specialisations & Hermetic V/F (PLAN.md 4b)
e2e: `ui/e2e/specs/houses.e2e.js`

- [ ] House data model + registry; evaluate the `House` prereq (un-skip)
- [ ] `rules/core/houses.json` + i18n: 12 core Houses + free Virtues
- [ ] data-driven specialisation→free-Virtue (incl. Mystery Houses seeding a
      Supernatural Ability at 1 via `ability_score_grant`)
- [ ] `virtue_category_caps` (≤1 Major Hermetic Virtue); special-magus V/F data
- [ ] `HouseSelector.svelte`; selecting auto-grants the free Minor House Virtue
- [ ] e2e spec green; PLAN.md 4b box updated; gate passes

## Phase 5 — Spells (PLAN.md 4c)
e2e: `ui/e2e/specs/spells.e2e.js`

- [ ] Spell data model (T+F+level; uses Phase 2 Art registry) + spell-levels budget
- [ ] `rules/core/spells.json` seed + i18n (German names per translation tables)
- [ ] `SpellPicker.svelte` (add/remove, pick T+F+level), Fluent-labelled
- [ ] e2e spec green; PLAN.md 4c box updated; gate passes

## Phase 6 — Gift/supernatural rules + per-character fields (PLAN.md 4d-rest, 4e)
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
