# Engine semantics — the yardstick for "implementation correct"

Phase 0 of the full Virtue/Flaw audit (`README.md`). This file answers, field by
field, **what a V/F entry's data actually does at runtime**. A batch agent
checking an entry asks two questions of the data — does it say what the passage
says, and *does the engine do what the data says* — and this document is the
only reference for the second.

Written 2026-09-19 by reading the code at `main` (`ecb5150`). Nothing here is
recalled; every claim is from a file read in that run. Where a claim could not
be settled, it is in **Open questions** at the end rather than stated.

## How to read this

- **Cross-references are `` `file.rs::symbol` ``** — the project's form (CLAUDE.md,
  enforced by `crates/arm-rules/tests/source_citations.rs`). Never a line number.
- **Rulebook citations are acronym + line** (`ArMDE:3362`).
- "Consumer" means a **production** call site (`crates/*/src`, `ui/src`). Test
  modules are named only where the test is the *only* thing reading something.
- Every effect fold in the engine iterates the same list:
  `effective.rs::selections_for_effects` — the entity's **bought** selections
  **++ granted** ones (House grants, Mythic-Companion-type grants,
  `grants_selection` expansions, resolved warping fills). So a granted copy of a
  Virtue contributes its effects exactly like a bought one. Three exceptions
  read a narrower list and are called out where they occur.
- A selection whose `item_ref` does not resolve in `ruleset.point_items` is
  **skipped** by every fold (the `let Some(item) … else { continue }` in
  `effective.rs::for_each_effect`), never an error at that layer.

## Three layers consume effects, and they are separate

| Layer | Entry point | What it computes | Which variants it reads |
|---|---|---|---|
| **effective** (`crates/arm-rules/src/effective.rs` + `effective/*.rs`) | many free functions | creation-legality numbers: effective scores, caps, budgets, XP allocation | the creation variants |
| **derived** (`crates/arm-rules/src/derived.rs` + `derived/*.rs`) | `derived.rs::derived_totals` | in-play play-stat totals: casting, lab, penetration, MR, combat, soak, fatigue, wounds | the in-play variants, via the single fold `derived.rs::in_play_mods` |
| **validation** (`crates/arm-rules/src/validation/*`) | `validation/mod.rs::validate` | issues (errors/warnings), never a number | a handful, listed per variant |

A fourth, **load-time**, layer (`ruleset/integrity.rs::validate_effect_refs`,
`::validate_item_ratios`) checks an effect's *references* resolve and its ratios
are non-degenerate. It changes no number and is not a runtime consumer, but it
is what makes a malformed effect fail the load rather than silently no-op — so
it is listed per variant where it applies.

**The exhaustive-match discipline, and its limit.** Nine folds match on `Effect`
exhaustively, so adding a variant is a compile error in all of them
(`effective.rs::irrelevant_effect_variants` is the shared ~40-variant "no-op"
tail five of them reuse). Exhaustiveness proves every variant was *classified*.
It does **not** prove any variant changes a number: an arm that reads
`Effect::X { .. } => {}` is exhaustive and inert. Part C is the census of exactly
that gap.

---

# Part A — the `Effect` enum, all 42 variants

Declared in `types.rs::Effect`, tagged `#[serde(tag = "type", rename_all = "snake_case")]`
— so the JSON tag is the variant name in `snake_case`, and every section below
states it explicitly rather than inferring it.

All 42 variants are used by at least one shipped entry. Usage counts are
occurrences in `rules/core/virtues_flaws.json` (an entry carrying the same
variant twice counts twice).

---

## A1 `AbilityBonus`

**Serde tag** `ability_bonus`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `param` | `String` | no | Key of the item's own `parameters` entry whose *value* names the target Ability id. The param must be declared and carry domain `ability` — load-time integrity (`integrity.rs::validate_effect_refs`) rejects the item otherwise. |
| `amount` | `i8` | no | Points added to the effective score. |

**Consumers**

- `effective/ability.rs::ability_bonus` — the sum.
- `effective/ability.rs::effective_ability_score` — adds it.
- `effective/ability.rs::ability_bonuses` — the per-instance list the UI shows.
- `validation/mod.rs::effect_target` → `validation/selections.rs::validate_ability_bonus_targets`.
- `ruleset/integrity.rs::validate_effect_refs` (load).

**Arithmetic.** `ability_bonus(entity, ruleset, ability, parameter)` walks every
`(selection, effect)` pair and adds `i32::from(amount)` for each
`AbilityBonus` whose `selection.params[param] == ability` **and** whose instance
matches. Instance matching: if the Ability's catalogue definition declares its
own `parameter` key (e.g. `(Area) Lore` → `"area"`), the selection must *also*
carry that key with the same value; a selection that omits it matches **nothing**
(not "every instance"). A plain Ability matches on id alone. Bonuses **stack**
across selections — two Puissant rows on one Ability give +4, and the engine does
not stop that (legality is `max_per_target`'s job).

`effective_ability_score = max(bought, granted_floor) + ability_bonus`, where
`granted_floor` is `AbilityScoreGrant` (A9). So the flat bonus is applied **after**
the floor, on top of it — Second Sight 1 plus Puissant Second Sight is 3.

**Order of operations vs. caps.** The age cap
(`validation/scores.rs::validate_ability_age_cap`) is checked against the
**bought** `AbilityScore.score`, not the effective score. A Puissant bonus
therefore never pushes an Ability over its age cap. Likewise the XP charge
(`effective/xp.rs::build_spends`) prices the bought score; a flat bonus is free.

**When it fires.** Character creation *and* in play — it is a change to the
effective score, which every derived total reads
(`derived.rs::ability`, `derived/lab.rs::lab_totals` etc. call
`effective_ability_score`). Classification-wise this is a `creation_effect`: it
moves a number on the character sheet at build time.

**Silently ignores.** Nothing of its own two fields. What it *cannot* express:
a bonus conditional on circumstance (that is `AbilityRollMod`, A41), a bonus to a
*category* of Abilities, or a bonus that also raises the age cap. A Puissant
naming a parameterized Ability with no instance key is inert in the arithmetic —
reported separately as `ability_bonus_dangling_target`, but only for **bought**
selections: `validate_ability_bonus_targets` iterates `entity.selections`, not
`selections_for_effects`, so a House-granted Puissant with a dangling target is
silently inert.

**Usage** 1 — `virtue.puissant_ability`.

---

## A2 `CharacteristicScoreDeltaParam`

**Serde tag** `characteristic_score_delta_param`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `param` | `String` | no | Param key whose value names the target Characteristic id (`characteristic.str`, …). Domain must be `characteristic`. |
| `amount` | `i8` | no | Free effective-score delta per selection; may be negative. |

**Consumers**

- `effective/characteristic.rs::characteristic_score_bonus` — the sum.
- `effective/characteristic.rs::effective_characteristic_score`, `::characteristic_bonuses`.
- `effective/warping.rs::effective_characteristic_after_aging` (via `characteristic_score_bonus`).
- `validation/mod.rs::effect_target` → `validation/scores.rs::validate_characteristic_delta_preconditions`.
- `ruleset/integrity.rs::validate_effect_refs` (load).

**Arithmetic.** Adds `amount` to the effective score of the named Characteristic.
It does **not** move the bought score and does **not** widen the buy range:
`characteristic_cap` / `characteristic_floor` return the ruleset's `base_max` /
`base_min` (±3) unconditionally, and take the Characteristic only to satisfy the
signature (`_characteristic`). Summed across selections, **no clamp of any kind**
— deliberately, because `ArMDE:3977` lets Giant Blood reach +6. Great's own "to no
more than +5" is an emergent property of `max_per_target: 2` over a bought score
capped at +3, not a clamp in code.

`effective_characteristic_score = bought + Σ deltas`.
`effective_characteristic_after_aging = bought.saturating_sub(aging_drops) + Σ deltas`
— the aging drop hits the bought score, the free delta is added after.

**Precondition check.** `validate_characteristic_delta_preconditions` enforces
"must already be at ±3": a positive `amount` errors
(`characteristic_max_base_too_low`) unless the bought score already equals
`base_max`; a negative one mirrors it. Derived from the sign of `amount` and the
ruleset's base cap/floor — no per-item data. It iterates **bought**
`entity.selections` only.

**When it fires.** Creation (and the resulting score is read in play).

**Silently ignores.** A selection whose `param` value is not one of the eight
Characteristics contributes 0 and is skipped by both the fold and the
precondition check (reported by `validate_parameters` as an unknown param value).

**Usage** 2 — `virtue.great_characteristic`, `flaw.poor_characteristic`.

---

## A3 `ArtBonus`

**Serde tag** `art_bonus`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `param` | `String` | no | Param key naming the target Art id; domain must be `art`. |
| `amount` | `i8` | no | Points added to the effective Art score. |

**Consumers**

- `effective/art.rs::art_bonus`, `::effective_art_score`, `::art_bonuses`.
- `ruleset/integrity.rs::validate_effect_refs` (load).

**Arithmetic.** `Σ amount` over selections whose `params[param]` equals the Art
id. Arts are never parameterized, so no instance matching. Stacks (two Puissant
Art rows on one Art give +6 — the engine computes it; legality is validation's).

`effective_art_score = bought + art_bonus + elemental_form_bonus` (A42). The Art
bonus reaches every in-play total through `effective_art_score`, and the
creation-time per-spell level cap through `effective/spell.rs::spell_level_cap`.

**When it fires.** Creation and in play.

**Silently ignores.** Nothing of its own fields. Unlike `AbilityBonus` there is
no dangling-target check for Arts — a `param` the selection never filled yields
`params.get(param) == None`, which matches no Art, and nothing reports it beyond
`validate_parameters`' generic `missing_param`.

**Usage** 1 — `virtue.puissant_art`.

---

## A4 `AffinityAbilityCost`

**Serde tag** `affinity_ability_cost`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `param` | `String` | no | Param key naming the target Ability id; domain `ability`. |
| `counts_as_num` | `u8` | no | Numerator of the "counts as num/den of itself" multiplier (Affinity = 3). |
| `counts_as_den` | `u8` | no | Denominator (Affinity = 2). |

**Consumers**

- `effective/xp.rs::ability_affinity` — resolves the multiplier for one instance.
- `effective/xp.rs::build_spends` → `::charged_cost` — applies it.
- `validation/scores.rs::validate_ability_age_cap` — presence raises the age cap by +2.
- `validation/mod.rs::effect_target` → `validation/selections.rs::validate_ability_bonus_targets` (dangling-target check, shared with `AbilityBonus`).
- `ruleset/integrity.rs::validate_item_ratios`, `::validate_effect_refs` (load).

**Arithmetic.** Instance matching is the same as `AbilityBonus`'s, except for one
difference: `ability_affinity` compares
`selection.params.get(key).map(Id::as_str) == parameter`, so a selection that
omits the instance key yields `None == parameter`, which is **true when the query
target is also `None`**. (`ability_bonus` instead hard-codes `None => false`.)
For a parameterized Ability the query always carries a `Some(instance)` for a
bought row, so in practice both behave the same; the asymmetry is real but not
reachable through a bought instance.

Several Affinities on one target do **not** stack: `xp.rs::best_affinity` keeps
the single most generous, comparing `n1·d2 ≥ n2·d1`.

The charge is `charged_cost(table_xp, Some((num, den))) = ceil(table_xp · den / num)`,
computed as `table_xp.saturating_mul(den).div_ceil(num)`. **Rounding is up.** The
book's worked example (`ArMDE:2443`): table 55, 3/2 → `ceil(55·2/3) = 37`.
`num == 0` or `den == 0` falls through to the **full** cost (fail-safe); both are
rejected at load anyway.

**Order.** The Affinity applies to the *payable* table cost, i.e. **after** the
`AbilityScoreGrant` floor is subtracted:
`payable = table(score) − table(floor)`, then `charged_cost(payable, affinity)`.

**Age cap.** `validate_ability_age_cap` adds a flat **+2** to the cap when
`ability_affinity` returns `Some` — regardless of the ratio. `GroupAffinityCost`
(A6) also satisfies this, since `ability_affinity` folds both.

**When it fires.** Character creation only (it is an XP price, and the app does
not simulate advancement) — plus the creation-time age cap.

**Silently ignores.** The ratio's *size* is invisible to the age-cap bonus (+2
either way). The XP-space reduction never reaches the effective score, so an
Affinity changes nothing a derived total reads.

**Usage** 1 — `virtue.affinity_ability`.

---

## A5 `AffinityArtCost`

**Serde tag** `affinity_art_cost`

**Fields** — identical shape to A4: `param` (domain `art`), `counts_as_num`,
`counts_as_den`.

**Consumers**

- `effective/xp.rs::art_affinity` (module-private) → `::build_spends` → `::charged_cost`.
- `ruleset/integrity.rs::validate_item_ratios`, `::validate_effect_refs` (load).

**Arithmetic.** As A4, matched by Art id alone. Most generous wins
(`best_affinity`); charge is `ceil(table · den / num)`, rounded up. Applied to the
full Art table cost — Arts have no granted floor to subtract.

**When it fires.** Character creation only.

**Silently ignores.** There is **no** Art analogue of the +2 age cap (that rule is
Ability-only in `ArMDE:3374`), and no dangling-target check: `effect_target`
classifies `AffinityArtCost` as `Other`, so an Affinity naming an Art the
character never bought is inert and unreported.

**Usage** 1 — `virtue.affinity_art`.

---

## A6 `GroupAffinityCost`

**Serde tag** `group_affinity_cost`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `abilities` | `BTreeSet<Id>` | no | The Ability ids the Affinity covers, matched by id, **any instance**. |
| `counts_as_num` | `u8` | no | Numerator (Linguist = 5). |
| `counts_as_den` | `u8` | no | Denominator (Linguist = 4). |

**Consumers**

- `effective/xp.rs::ability_affinity` (second arm) → `::build_spends` → `::charged_cost`.
- `validation/scores.rs::validate_ability_age_cap` (via `ability_affinity`, +2).
- `ruleset/integrity.rs::validate_item_ratios`, `::validate_effect_refs` (load: every id must resolve to a known Ability).

**Arithmetic.** Identical to A4 once matched, but the match is
`abilities.contains(ability)` with **no instance test at all** — so it covers
every instance of a parameterized Ability (every Living Language, not one).
Competes with `AffinityAbilityCost` through the same `best_affinity`.

**When it fires.** Character creation only.

**Silently ignores.** It cannot be scoped to one instance, and it cannot be
expressed over an Ability *category* — only an explicit id list. A group Affinity
over a category (say "all Academic") would need one id per Ability, which is a
data problem, not an engine one.

**Usage** 1 — `virtue.linguist`.

---

## A7 `RestrictedAbilityXp`

**Serde tag** `restricted_ability_xp`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `u32` | no | Points granted to this restricted pool. |
| `abilities` | `Vec<Id>` | yes (default `[]`) | Eligible Ability ids. |
| `categories` | `Vec<AbilityCategory>` | yes (default `[]`) | Eligible Ability categories. |

**Consumers**

- `effective/xp.rs::restricted_ability_xp_pools` → `::build_flow_pools` → `::xp_allocation`.
- `effective/xp.rs::ability_authorizations` — the grant **also confers permission** to buy the listed Abilities/categories.
- `validation/authorization.rs::validate_ability_authorization` (through `ability_authorizations`).
- `effective/xp.rs::xp_solve_scale` (counts toward the flow-solve node bound).
- `ruleset/integrity.rs::validate_effect_refs` (load: every `abilities` id must resolve; `categories` are **not** checked against a registry).

**Arithmetic.** One `FlowPool` per effect occurrence, capacity `amount`,
eligibility `Ability { abilities, categories, instances: [], exclude: [] }`. An
Ability spend qualifies if `abilities.contains(id) || categories.contains(cat)` —
an **OR**, not an AND. The pool funds **Ability spends only**: never an Art, never
Spell Mastery (`xp.rs::pool_covers`).

Allocation is a **two-phase max flow** (`xp.rs::two_phase_max_flow`): phase 1
solves with the general-pool edge closed, so restricted pools drain first; phase 2
opens the general pool and continues on the same residuals. Consequence: unused
restricted XP is wasted, and restricted XP is never left idle while an eligible
spend goes unfunded. Overlapping pools are resolved globally, not greedily.

Pools **stack** — two `Educated` selections give two 50-point pools.

**Permission side-effect.** `ability_authorizations` treats a restricted pool as
evidence that the character may own what it funds, so Warrior (Martial XP) also
legalises Martial Abilities with no separate `AbilityAuthorization` row. This is
load-bearing: adding `ability_authorization` alongside it would be redundant.

**When it fires.** Character creation only.

**Silently ignores.** Nothing of its three fields. It cannot express an
instance-scoped pool (that shape exists — `PoolEligibility::Ability.instances` —
but only life-stage blocks construct it; `restricted_ability_xp_pools` hard-codes
`instances: Vec::new(), exclude: Vec::new()`). It cannot fund Arts or Mastery.

**Usage** 28.

---

## A8 `CharacteristicPoints`

**Serde tag** `characteristic_points`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i8` | no | Points added to (negative: removed from) the Characteristic-buy budget, per selection. |

**Consumers**

- `effective/characteristic.rs::characteristic_points_granted`.
- `validation/scores.rs::validate_characteristic_point_spend` — `budget = start_points + granted`; `cost > budget` → `characteristic_overspent` (error), `cost < budget` → `characteristic_points_unspent` (warning).
- `arm-app/src/ruleset_io.rs::characteristic_fields` — surfaced as `EffectiveScores::characteristic_points_granted`; `ui/src/lib/components/CharacteristicPicker.svelte` shows `start_points + granted`.

**Arithmetic.** `Σ i32::from(amount)` across all selections and grants. Signed,
stacking, and **not clamped** at this layer — the net may be negative, which then
lowers the budget below the ruleset's `start_points`.

**When it fires.** Character creation only.

**Silently ignores.** Nothing. It has no target — it is a whole-character budget
adjustment and cannot be scoped to one Characteristic.

**Usage** 2 — `virtue.improved_characteristics`, `flaw.weak_characteristics`.

---

## A9 `AbilityScoreGrant`

**Serde tag** `ability_score_grant`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `ability` | `Id` | no | The Ability granted a free starting score. Fixed by the Virtue, **not** player-chosen — stored directly, never read from a param. |
| `amount` | `u8` | no | The free bought-score **floor**. |

**Consumers**

- `effective/ability.rs::granted_ability_floor` — the floor for one `(ability, parameter)`.
- `effective/ability.rs::effective_ability_score` — `max(bought, floor) + bonus`.
- `effective/ability.rs::ability_score_floors` — the per-ability list.
- `effective/xp.rs::build_spends` — subtracts the floor's table cost before charging.
- `effective/xp.rs::ability_authorizations` — the grant is permission to own the Ability.
- `effective/reputation_and_caps.rs::supernatural_free_slots` — a covered Ability does not consume the Gift's free slot.
- `validation/scores.rs::validate_supernatural_abilities` — a granted Supernatural Ability is "covered".
- `export/sections.rs` — lists granted-but-unbought Abilities.
- `ui/src/lib/components/AbilityTab.svelte` — reads `effect.type === 'ability_score_grant'` off the *selected* items to unlock the Supernatural picker synchronously.
- `ruleset/integrity.rs::validate_effect_refs` → `::validate_ability_ref` (load: the id must resolve).

**Arithmetic.** A **floor**, not an addend, and grants do **not** stack:
`granted_ability_floor = max over matching grants of amount` (a `max`, not a
`Σ`). It applies only to the **parameter-less** instance — `granted_ability_floor`
returns 0 immediately when `parameter.is_some()`, so a grant can never seed one
instance of `(Area) Lore`.

`effective_ability_score = max(bought, floor) + ability_bonus`. An Ability with no
bought row still reports the granted score.

**XP.** `build_spends` computes `payable = table(bought) − table(floor)`
(`saturating_sub`), then applies any Affinity. So the first `amount` points are
free and everything above is charged normally (`ArMDE:2639`). If the character's
bought score is *below* the floor, `payable` saturates to 0 — the row costs
nothing — and the floor still applies.

**When it fires.** Character creation (and the resulting score is read in play).

**Silently ignores.** Nothing of its two fields. It cannot target a parameterized
Ability instance. Two grants of different sizes on one Ability yield the larger,
silently — nothing warns that the smaller is inert.

**Usage** 24.

---

## A10 `SpellLevels`

**Serde tag** `spell_levels`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i16` | no | Spell levels added to (negative: removed from) the magus's budget, per selection. |

**Consumers**

- `effective/spell.rs::spell_levels_bonus` (its own hand-written exhaustive match, not the shared tail).
- `effective/spell.rs::spell_levels_budget`.
- `validation/magus.rs::validate_spell_levels_budget` and `arm-app/src/ruleset_io.rs` both read `spell_levels_budget`, so they cannot disagree.

**Arithmetic.**
`spell_levels_budget = clamp_to_u32(base + Σ amount + life_stage_spell_levels)`,
where `base` is `Entity::spell_levels_override` or the type profile's
`spell_levels`. Signed, summed, then clamped at **0** (a net-negative total floors
at 0, it does not underflow).

**When it fires.** Character creation only.

**Silently ignores.** Nothing. It has no scope — it cannot restrict the levels to
a Technique, Form or level band.

**Usage** 2 — `virtue.skilled_parens`, `flaw.weak_parens`.

---

## A11 `GeneralXp`

**Serde tag** `general_xp`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i16` | no | Experience added to (negative: removed from) the general pool, per selection. |

**Consumers**

- `effective/spell.rs::general_xp_bonus` (own exhaustive match).
- `effective/xp.rs::general_pool_and_bonus` → `::xp_allocation`.
- Surfaced as `XpAllocation::general_bonus` and `EffectiveScores::xp_general_bonus`.

**Arithmetic.** `general_pool = clamp_to_u32(base_general + Σ amount)`, clamped at
0. `base_general` depends on the funding mode:

| Character | `base_general` |
|---|---|
| magus with a life-stage plan | `apprenticeship_xp + post_gauntlet_xp` |
| non-magus with a life-stage plan | `later_life_xp` |
| no life-stage plan (flat entry) | `Entity::xp_pool` |

The general pool funds **any** spend (Ability, Art, Spell Mastery). It is opened
only in **phase 2** of the flow solve, after restricted pools have drained.

**When it fires.** Character creation only.

**Silently ignores.** Nothing of its own field. Note the deliberate
non-interaction documented in `general_xp_bonus`: `LaterLifeXpRate` is **not**
folded here — it multiplies out into the life-stage budget instead, and folding
both would double-count.

**Usage** 2 — `virtue.skilled_parens`, `flaw.weak_parens`.

---

## A12 `LaterLifeXpRate`

**Serde tag** `later_life_xp_rate`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `u32` | no | Experience points earned **per year** of later life. |

**Consumers**

- `life_stage.rs::LifeStageRules::later_life_rate` — the only reader.
- Downstream: the life-stage budget (`later_life_xp`), which becomes either the
  general pool (non-magus) or the `LaterLife` restricted pool (magus).

**Arithmetic.** **Replaces** the base rate; it is not additive. The base rate is
`LaterLifeRules::xp_per_year` and is *not* a candidate in the comparison — a named
rate replaces it outright. When several selections name a rate, the **lowest**
wins (`min`), deliberately, so the result does not depend on declaration order.
With no such effect the base rate stands.

**Only reachable through a life-stage plan.** `later_life_rate` is called by the
life-stage budget derivation. A character with no life-stage plan (flat `xp_pool`
entry) never reaches it, so on that path Wealthy/Poor change nothing.

**When it fires.** Character creation only.

**Silently ignores.** Nothing of its own field. The rule that both Wealthy and
Poor are companion-only is **not** in this effect — it is enforced by the type
profiles.

**Usage** 2 — `virtue.wealthy`, `flaw.poor`.

---

## A13 `LocalityAbilityCapFraction`

**Serde tag** `locality_ability_cap_fraction`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `num` | `u8` | no | Numerator of the surviving fraction (1 for "half"). |
| `den` | `u8` | no | Denominator (2). |

**Consumers**

- `effective/reputation_and_caps.rs::ability_age_cap` — the only reader.
- `validation/scores.rs::validate_ability_age_cap` (through it).
- `ruleset/integrity.rs::validate_item_ratios` (load: rejects `num == 0` and `den == 0`).

**Arithmetic.** It narrows the **age cap**, never the cost. Applies only to
Abilities the catalogue marks `locality_dependent` (`ability.rs::AbilityDef`);
for any other Ability `ability_age_cap` returns the base cap untouched, before
the fold runs at all.

`fractioned = ceil(base_cap · num / den)`, computed as
`(base·num + den − 1) / den` — **rounded up**, per `ArMDE:6160`. Several such
Flaws compose by taking the **smallest** result (`cap = cap.min(fractioned)`), not
by multiplying the fractions.

The `den > 0` guard skips the whole narrowing when the denominator is 0 (silently
permissive) — load-time integrity is what makes that unreachable.

`validate_ability_age_cap` then adds +2 if the Ability carries an Affinity, and
compares against the **bought** score.

**When it fires.** Character creation only.

**Silently ignores.** Which Abilities count as locality-dependent is entirely the
Ability catalogue's `locality_dependent` flag — the effect carries no list and
cannot narrow its own scope. It does not affect XP cost, effective score, or any
in-play total.

**Usage** 1 — `flaw.foreign_upbringing`.

---

## A14 `AbilityAuthorization`

**Serde tag** `ability_authorization`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `abilities` | `Vec<Id>` | yes (default `[]`) | Specific Abilities permitted. |
| `categories` | `Vec<AbilityCategory>` | yes (default `[]`) | Whole categories permitted. |

**Consumers**

- `effective/xp.rs::ability_authorizations` — folded together with `RestrictedAbilityXp`'s lists into one `(abilities, categories)` pair.
- `validation/authorization.rs::validate_ability_authorization` — gates *owning* a gated-category Ability (`ability_category_requires_virtue`).
- `effective/xp.rs::magus_later_life_pool` — widens which categories a magus's pre-apprenticeship pool may fund.

**Arithmetic.** No number. It is a set union: the permitted ids and categories are
extended into two `BTreeSet`s. `validate_ability_authorization` errors on a held
Ability whose category is in `ruleset.categories_requiring_virtue()` unless its id
or its category is in that union — with a whole-character exemption for any
profile whose `is_magus` is true.

**Note the redundancy.** A `RestrictedAbilityXp` pool already implies the same
permission, so this variant is only needed for a Virtue that permits *without*
funding.

**When it fires.** Character creation only.

**Silently ignores.** Nothing of its two fields. `categories` is not validated
against a registry at load — an unknown category string simply never matches.
(`AbilityCategory` is a Rust enum, so an unknown value fails serde at load
instead.)

**Usage** 2 — `flaw.covenant_upbringing`, `virtue.student_of_realm`.

---

## A15 `ConfidenceBonus`

**Serde tag** `confidence_bonus`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `score` | `i8` | no | Confidence Score added per selection. |
| `points` | `i8` | no | Confidence Points added per selection. |

**Consumers**

- `effective/gift_confidence.rs::confidence` — the only reader.
- `arm-app/src/ruleset_io.rs` — surfaced as the confidence fields of `EffectiveScores`.

**Arithmetic.** `score = clamp(base_score + Σ score)`,
`points = clamp(base_points + Σ points)`, where `clamp(n) = u8::try_from(max(n,0))`
— i.e. **floored at 0** and saturating at 255. `base_score`/`base_points` are the
type profile's defaults, passed in by the caller. Confidence is never stored on
the entity.

**When it fires.** Character creation (and it is a standing in-play figure, but no
derived total consumes it).

**Silently ignores.** Nothing of its two fields.

**Usage** 3 — `virtue.self_confident`, `virtue.ferocity`, `flaw.low_self_esteem`.

---

## A16 `SpellMasteryXp`

**Serde tag** `spell_mastery_xp`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `u16` | no | Mastery experience points granted per selection. |

**Consumers**

- `effective/spell.rs::spell_mastery_xp` — the sum.
- `effective/xp.rs::spell_mastery_flow_pool` → `::build_flow_pools` → `::xp_allocation`.
- `arm-app/src/ruleset_io.rs::spell_mastery_fields`.

**Arithmetic.** `Σ u32::from(amount)`, stacking. Becomes **one** `FlowPool` of that
total capacity with `PoolEligibility::Mastery`, which funds **only** Spell Mastery
spends — never an Ability, never an Art (`xp.rs::pool_covers`). Created only when
the total is `> 0`.

The mastery pool is **flow-only**: `assemble_restricted_pools` surfaces only
`PoolEligibility::Ability` pools, so it never appears in
`XpAllocation::restricted` and the XP bar shows no bar for it. Its `origin` is
nominal (`LifeStageBlock::ChildhoodSpread`) and is never read.

**When it fires.** Character creation only.

**Silently ignores.** Nothing of its own field. It cannot be scoped to particular
spells.

**Usage** 1 — `virtue.mastered_spells`.

---

## A17 `GrantsSpellMastery`

**Serde tag** `grants_spell_mastery`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `score` | `u8` | no | Mastery-score **floor** granted to every known spell. |
| `advancement_num` | `u8` | yes (default 1, omitted when 1) | Numerator of the mastery Advancement-Total multiplier. |
| `advancement_den` | `u8` | yes (default 1, omitted when 1) | Denominator. |

**Consumers**

- `effective/spell.rs::spell_mastery_floor` — reads `score` only (`{ score, .. }`).
- `effective/spell.rs::spell_mastery_advancement_affinity` — reads the ratio only.
- `effective/spell.rs::effective_spell_mastery` — `max(bought, floor)`.
- `effective/xp.rs::build_spends` — subtracts the floor's table cost and applies the ratio.
- `arm-app/src/ruleset_io.rs::spell_mastery_fields` — surfaces both the floor and the authored `(num, den)` pair (deliberately the ratio, not a boolean).
- `ruleset/integrity.rs::validate_item_ratios` (load: rejects a 0 in either half).

**Arithmetic.** Two independent things in one variant.

1. **Floor.** `spell_mastery_floor = max over grants of score` (a `max`, not a sum).
   `effective_spell_mastery(spell) = max(spell.mastery.unwrap_or(0), floor)`.
2. **Affinity.** `spell_mastery_advancement_affinity` collects `(num, den)` only
   from grants where `num > den` — the identity `1/1` contributes nothing — and
   picks the most generous via `best_affinity`. Applied in `build_spends` as
   `charged_cost(payable, affinity)` = `ceil(payable · den / num)`, rounded up.

XP: `payable = table(bought_mastery) − table(floor)`, saturating; a spell with
`bought == 0`, or with `cost == 0` after the reduction, produces no `Spend` at all.

**When it fires.** Character creation only (the floor is also a standing score, but
nothing in `derived.rs` reads mastery).

**Silently ignores.** The floor applies to **every** known spell with no way to
scope it. A grant with `num <= den` (including the default `1/1`) contributes no
Affinity — correct, but note that an author writing `advancement_num: 1,
advancement_den: 2` (a *penalty*) would be silently ignored rather than doubling
the cost.

**Usage** 1 — `virtue.flawless_magic`.

---

## A18 `GrantsSelection`

**Serde tag** `grants_selection`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `items` | `BTreeSet<Id>` | no | Virtue/Flaw ids granted for free (budget-exempt). |

**Consumers**

- `effective.rs::vf_granted_selections` → `::entity_grants_base` → `::entity_grants` → `::selections_for_effects`.
- `ruleset/integrity.rs::validate_effect_refs` → `::validate_item_list_effect` (load: every id must resolve to a point item).

**Arithmetic.** No number. Each id becomes a bare `Selection::new(id)` — **with no
params** — appended to the grant list, so the granted item's own effects are then
folded by every consumer exactly like a bought selection's.

**One level of nesting only.** `vf_granted_selections` scans
`entity.selections` (**bought only**), not `selections_for_effects`. A granted
item's own `grants_selection` is therefore **not** expanded. This is deliberate and
documented.

**Budget-exempt but not cap-exempt.** Granted rows are outside the V/F point
balance and the category/major-count caps (those read `entity.selections`), but
they **do** count toward `max_per_target`, `max_total` and `max_share_of_kind`,
which run against the folded list.

**When it fires.** Character creation and in play — whatever the granted items do.

**Silently ignores.** A granted `Selection` carries **no parameters**. So granting
an item that *requires* a param (Puissant Ability, Magical Focus, Deficient Art)
produces a selection whose param is unfilled: its effect resolves to nothing and
is inert. Nothing in `vf_granted_selections` warns about this.

**Usage** 7.

---

## A19 `ItemLevelBudget`

**Serde tag** `item_level_budget`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `u16` | no | Levels of enchanted devices added to the budget. |

**Consumers**

- `effective/gift_confidence.rs::item_level_budget` — the sum.
- `validation/might.rs::validate_devices` — compares against `item_level_used`.
- `arm-app/src/ruleset_io.rs` — surfaced for the budget bar.

**Arithmetic.** `Σ u32::from(amount)`, base 0, stacking, no clamp needed (unsigned).
Spent side is `item_level_used = Σ device.level`. Over-spend is an error.

**When it fires.** Character creation only.

**Silently ignores.** Nothing. Note that a **talisman**'s instilled effects are
deliberately charged against no budget, and `focus_powers` / `powers` are charged
against their own separate pools (A27, A28) — this budget covers `Entity::devices`
alone.

**Usage** 2 — `virtue.magic_items`, `virtue.redcap`.

---

## A20 `MasterpieceItem`

**Serde tag** `masterpiece_item` — a **unit variant**, serialized as
`{ "type": "masterpiece_item" }` with no other key.

**Fields** none.

**Consumers**

- `derived.rs::in_play_mods` — sets `InPlayMods::has_masterpiece`.
- `derived/lab.rs::masterpiece_item_cap` — the only reader of that flag.
- `ruleset/integrity.rs::validate_effect_refs` — falls in the "no ref to resolve" arm.

**Arithmetic.** A pure marker. When present, `masterpiece_item_cap` returns the
best cell of the already-built Lab-Total grid by `LabTotal::enchanting` (the
Weak-Enchanter-halved figure, equal to `total` without that Flaw), and reports
`cap = halve(enchanting)` — floor division (`div_euclid(2)`), from the
lesser-enchantment rule `Lab Total ≥ 2 × level` (`ArMDE:10410`). Ties break on the
**first** maximum in `(technique, form)` order.

`masterpiece_item_cap` is called only for a magus (`derived_totals` gates on
`is_magus`).

**When it fires.** In play / read-out only. It is a **read-only guidance** figure:
no device is created, no budget spent.

**Silently ignores.** Everything except its own presence. Two Masterpiece items
would produce one cap. It is a no-op in `effective.rs`, validation, and
referential checks.

**Usage** 1 — `virtue.masterpiece`.

---

## A21 `TrueFaithGrant`

**Serde tag** `true_faith_grant`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `score` | `u8` | no | True Faith Score added. |

**Consumers**

- `effective/might.rs::true_faith` — the only reader.
- `arm-app/src/ruleset_io.rs` — surfaced as `EffectiveScores::true_faith_score`.

**Arithmetic.** `Σ u32::from(score)`, base 0, then `u8::try_from(..).unwrap_or(u8::MAX)`
— saturating at 255. Derived, never stored.

**When it fires.** Character creation (a standing score). Nothing in `derived.rs`
or `validation/` consumes the resulting True Faith score — it is display-only.

**Silently ignores.** Everything True Faith *does* in the rules. The engine holds
the number and renders it; no total anywhere adds it.

**Usage** 3 — `virtue.true_faith`, `virtue.relic`, `virtue.powerful_relic`.

---

## A22 `WarpingGrant`

**Serde tag** `warping_grant`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `score` | `u8` | no | Warping Score "granted". **Never read** — see below. |
| `points` | `u8` | no | Warping Points added. |

**Consumers**

- `effective/warping.rs::warping_grant_points_in` — reads `points`; `score` is explicitly destructured as `score: _`.
- `effective/warping.rs::warping_points_total`, `::warping_points_for_owed`, `::warping_score`, `::warping`.
- `effective/warping.rs::item_carries_warping_grant` — a `matches!` predicate, reads neither field.
- `validation/warping.rs::validate_warping` — `warping_fill_ineligible`.
- `ui/src/lib/derive.ts` — filters warping-fill candidates by `e.type === 'warping_grant'`.

**Arithmetic.** The Warping **Score is derived from points**, never from the
`score` field:
`warping_points_total = entity.warping_points.saturating_add(Σ points)`, then
`warping_score = ruleset.advancement.score_for_xp(points_total)` — the Ability
advancement curve inverted (cumulative 5/15/30/50/75, so 15 points → Score 2).
The `score` field is inert by design, so the two can never disagree.

**Recursion guard.** `warping_owed` uses a *narrower* point total,
`warping_points_for_owed`, computed over bought selections plus non-warping grants
only — the owed warping fills are excluded. And
`warping_granted_selections` **drops** any chosen fill that itself carries a
`WarpingGrant`. So a warping-granting item can never inflate the count of items it
owes.

Owed V/F thresholds (`WarpingOwed::from_score`): Minor Flaws 1 at Score ≥ 1 and 2
at ≥ 3; one supernatural Minor Virtue at ≥ 5; Major Flaws `score.saturating_sub(5)`.
A **magus owes zero** (`profile.is_magus` → `WarpingOwed::default()`).

**When it fires.** Character creation (owed V/F, and the score) and the in-play
read-out (`DerivedTotals::warping_score` / `warping_points`).

**Silently ignores.** The `score` field, entirely — the clearest half-read variant
in the enum. An entry authoring `{"score": 3, "points": 5}` grants 5 points, which
the curve reads as Score 1, and the 3 is discarded without a warning.

**Usage** 1 — `flaw.warped_by_magic`.

---

## A23 `SizeDelta`

**Serde tag** `size_delta`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i8` | no | Size adjustment per selection; may be negative. |

**Consumers**

- `effective/characteristic.rs::size` — the only fold.
- `derived.rs::derived_totals` (`DerivedTotals::size`), `derived/combat.rs::wound_ranges`, `derived/familiar.rs` (via `size`).

**Arithmetic.** `Σ i32::from(amount)`, base 0, **no clamp**. Size is not a bought
Characteristic: no cost, no buy cap.

Downstream: wound bands use `u = max(1, size + 5)` and every band scales with it
(Light `1..u`, Medium `u+1..2u`, Heavy `2u+1..3u`, Incapacitating `3u+1..4u`, Dead
`4u+1..`).

**When it fires.** Character creation (the number) and in play (wound bands).

**Silently ignores.** Nothing of its own field. Note that Size does **not** feed
Soak, damage or Encumbrance in this engine — only the wound bands and the
familiar read-out.

**Usage** 5 — `virtue.giant_blood`, `virtue.large`, `virtue.blood_of_the_nephilim`,
`flaw.small_frame`, `flaw.dwarf`.

---

## A24 `CharacteristicScoreDelta`

**Serde tag** `characteristic_score_delta`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `characteristic` | `Id` | no | The fixed Characteristic id (`characteristic.str`, …). Stored directly, not read from a param. |
| `amount` | `i8` | no | Free effective-score bonus per selection; may be negative. |

**Consumers**

- `effective/characteristic.rs::characteristic_score_bonus` (first arm) — shares the fold with `CharacteristicScoreDeltaParam`.
- `effective/characteristic.rs::effective_characteristic_score`, `::characteristic_bonuses`.
- `effective/warping.rs::effective_characteristic_after_aging`.
- `ruleset/integrity.rs::validate_effect_refs` (load: the id must be one of the eight Characteristics).

**Arithmetic.** Identical to A2 once matched, except the target is fixed by id
rather than by a selection param — so it needs no parameter and cannot dangle.
Summed, **no ceiling** (Giant Blood's +1 Str/Sta may reach +6, `ArMDE:3977`).

Unlike A2 there is **no precondition check**: `effect_target` classifies this
variant as `Other`, so `validate_characteristic_delta_preconditions` never sees
it. Correct — Giant Blood does not require Str to be at +3 first.

**When it fires.** Character creation (and read in play through
`effective_characteristic_after_aging`, which every derived total uses).

**Silently ignores.** Nothing of its two fields. One effect targets exactly one
Characteristic, so an item affecting two carries two effects (Giant Blood and
Dwarf each carry two).

**Usage** 4 — `virtue.giant_blood` ×2, `flaw.dwarf` ×2.

---

## A25 `GrantsReputation`

**Serde tag** `grants_reputation`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `kind` | `Option<ReputationType>` | yes (omitted when `None`) | The audience the granted Reputation reaches. `None` = **player-chosen type** (a wildcard, e.g. Famous). |
| `score` | `u8` | no | The level of the granted Reputation. |

**Consumers**

- `effective/reputation_and_caps.rs::reputation_grants` — builds one `ReputationGrant { source, reputation_type, score }` per effect occurrence, tagging it with the granting item's id.
- `validation/scores.rs::validate_reputations` — the only enforcement.
- `arm-app/src/ruleset_io.rs` — surfaced as `EffectiveScores::reputation_grants`.

**Arithmetic.** Slot counting, not a number on a score. `validate_reputations`
builds `remaining: kind → count` from the concrete-kind grants plus a `wildcard`
counter from the `None` grants. Each `Entity::reputations` row consumes a matching
concrete slot **first**, falling back to a wildcard slot; a row with neither emits
`reputation_not_granted` (error).

**`score` is not enforced.** The grant's `score` reaches the frontend (so a picker
can pre-fill) but `validate_reputations` compares **kinds and counts only** — it
never checks `Reputation.score` against `ReputationGrant.score`. A character may
enter Local 10 backed by a grant of Local 3 with no issue raised.

An item with two audiences carries two `GrantsReputation` effects.

**When it fires.** Character creation only.

**Silently ignores.** The `score`, for validation purposes (see above). Also: a
grant does not *create* the Reputation — it authorizes one; the player still
enters it.

**Usage** 33 — the most-used variant in the catalogue.

---

## A26 `MightGrant`

**Serde tag** `might_grant`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `realm` | `Realm` | no | The Realm the granted Might is aligned to. |
| `score` | `u8` | no | Might Score points granted. **0 establishes the Realm without adding points.** |

**Consumers**

- `effective/might.rs::might_grants` (private) → `::effective_might`.
- `derived/casting.rs::magic_resistance` — a Might-being's blanket resistance.
- `derived.rs::derived_totals` — `has_might` gates the Magic Resistance grid for a non-magus.
- `validation/might.rs::validate_might` → `::ruleset_might_grant_realm` (reads `realm` only).

**Arithmetic.** The **Realm** is `entity.might.realm` if the entity carries a base
Might, else the realm of the **first** grant in fold order. The score is
`base + Σ score over grants whose realm equals the chosen realm`, saturating at
`u8::MAX`. Grants of a *different* Realm are silently excluded from the sum.
`effective_might` returns `None` — not a supernatural being — only when there is
neither a base Might nor any grant.

`validate_might` emits `might_realm_mismatch` (a **warning**) when the entity's
base Might realm differs from the first grant's realm.

Magic Resistance: for each Form,
`base = max(might, parma_for_form)` — Might and Parma do **not** stack; the higher
wins, compared per Form so a Flawed Parma lowers only the Parma side. Total is
`form_bonus + base`.

**When it fires.** Character creation (the score) and in play (Magic Resistance).

**Silently ignores.** A second grant in a *different* Realm contributes 0 to the
score (only a warning names the mismatch). Nothing charges Might against a budget.

**Usage** 3 — `virtue.demonic_blood`, `virtue.demonic_might`,
`virtue.strong_angelic_heritage`.

---

## A27 `PowerLevels`

**Serde tag** `power_levels`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `u16` | no | Levels of supernatural powers added to the budget. |

**Consumers**

- `effective/gift_confidence.rs::power_levels_budget` — the sum.
- `validation/might.rs::validate_powers` — `over_power_levels` when used > budget.
- `arm-app/src/ruleset_io.rs` — surfaced for the budget bar.

**Arithmetic.** `Σ u32::from(amount)`, base 0, stacking. The spent side is
`powers_used = Σ (power.level + power.penetration)` — **Penetration counts against
the same pool**, per `ArMDE:4019`.

**When it fires.** Character creation only.

**Silently ignores.** Nothing of its own field. It is deliberately a *different*
currency from `FocusPoints` (A28) — the two pools never cross.

**Usage** 7.

---

## A28 `FocusPoints`

**Serde tag** `focus_points`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `u16` | no | Focus Power points added to the pool. |

**Consumers**

- `effective/gift_confidence.rs::focus_points_budget` — the sum.
- `validation/might.rs::validate_focus_powers`.
- `derived/focus_power.rs::focus_power_lines` (the per-power read-out; reads `Entity::focus_powers`, not the effect).
- `arm-app/src/ruleset_io.rs`.

**Arithmetic.** `Σ u32::from(amount)`, base 0, stacking (`ArMDE:3903` permits taking
it more than once with the points combined). Spent side is
`focus_points_used = Σ (2 × max_level + penetration)` — **2 points per level of
effect, 1 per point of Penetration** (`ArMDE:3899`).

**When it fires.** Character creation only.

**Silently ignores.** Nothing of its own field.

**Usage** 1 — `virtue.focus_power`.

---

# The in-play block (A29–A42)

Everything from here down is a **5b in-play variant**. All of them are folded by
the *single* exhaustive match `derived.rs::in_play_mods`, which produces an
`InPlayMods` struct the per-area functions then read; all of them are explicit
no-ops in `effective.rs` (creation legality), with two deliberate exceptions:
`DeficientArt` also reaches `effective/spell.rs::spell_level_cap`, and
`AgingMod` also reaches `aging.rs` and `effective/warping.rs`.

**"Surfaced-only" means listed, not computed.** `in_play_mods` pushes a
`SurfacedModifier { family, detail, amount }` row that
`derived.rs::surfaced_modifiers` returns and the UI renders (through Fluent
`derived-detail-<slug>` keys). No number anywhere moves. When a variant below is
marked surfaced-only, the honest reading of an entry using it is: *the engine
displays that the modifier exists and its magnitude; it does not apply it.*

---

## A29 `MagicalFocus`

**Serde tag** `magical_focus`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `param` | `String` | no | Param key whose **free-text** value names the focus descriptor ("necromancy"). Domain must be `text` — a focus is sub-Art and may span Arts, so it is deliberately not an Art ref. |
| `major` | `bool` | no | `true` = Major Focus, `false` = Minor. |

**Consumers**

- `derived.rs::in_play_mods` — sets `InPlayMods::has_focus = true`. Reads **neither field**.
- `derived/casting.rs::casting_totals` (`within_focus`), `::penetration` (`within_focus`), `derived/lab.rs::lab_totals` (`within_focus`).
- `validation/selections.rs::validate_magical_focus` — counts occurrences.
- `ruleset/integrity.rs::validate_effect_refs` (load: `param` must exist with domain `text`).

**Arithmetic.** Within the focus, the **lower** of the pair's two effective Art
scores is added **again**: `score = sum(base terms) + min(te, fo)`, and the
Deficient-Art halving (if any) is applied **after** that addition. Surfaced as
`CastingTotal::within_focus`, `LabTotal::within_focus`,
`PenetrationLine::within_focus` — always alongside the ordinary figure, never
replacing it, because whether a given spell falls inside the descriptor cannot be
auto-derived. `None` on all three when `has_focus` is false.

**The one-focus limit.** `validate_magical_focus` counts every `MagicalFocus`
effect across the folded bought-plus-granted list and errors
(`multiple_magical_foci`) when the count exceeds 1. It counts the *effect*, not
pairwise `incompatible_with`, so two Minor foci with different descriptors are
caught.

**When it fires.** In play only.

**Silently ignores.** Both of its fields. `major` is **never read by any
consumer** — the within-focus computation is identical for a Major and a Minor
Focus, and the count-based limit does not distinguish them. `param` is only
validated at load; the descriptor text is never consulted, so the engine cannot
tell whether a given Technique/Form pair is inside the focus and simply shows
both figures for every cell.

**Usage** 3 — `virtue.major_magical_focus`, `virtue.minor_magical_focus`,
`virtue.mythic_blood`.

---

## A30 `CastingTotalMod`

**Serde tag** `casting_total_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i8` | no | Points added to (negative: removed from) the Casting Total. |
| `scope` | `CastingScope` | no | `all` / `formulaic` / `ritual` / `formulaic_ritual` / `spontaneous`. |

**Consumers**

- `derived.rs::in_play_mods` — pushes `(amount, scope)` onto `InPlayMods::casting_mods`.
- `derived.rs::InPlayMods::casting_mod_for` — sums the ones matching a `CastType`.
- `derived/casting.rs::casting_totals`, `::formulaic_casting_score`, `::penetration`.

**Arithmetic.** `casting_mod_for(cast) = Σ amount over mods whose scope matches`,
where the match is `derived.rs::CastType::matches`: Formulaic accepts
`all|formulaic|formulaic_ritual`; Ritual accepts `all|ritual|formulaic_ritual`;
Spontaneous accepts `all|spontaneous`.

It is a **Casting-Score** term, added with the other addends **before** the
Magical-Focus double, before the Deficient-Art halving, and before the
spontaneous divisor:
`formulaic = post(common + focus_add + formulaic_mod + extra, deficient)` where
`post` halves if deficient; `spont_non_fatiguing = spont_base.div_euclid(5)`,
`spont_fatiguing = halve(spont_base)` (or the ÷5 figure under Weak Spontaneous
Magic).

Surfaced in the breakdown as three always-present addends
(`casting_mod_formulaic`, `casting_mod_ritual`, `casting_mod_spontaneous`), even
at 0.

**When it fires.** In play only.

**Silently ignores.** Circumstantial conditions. Several shipped carriers are
conditional in the book (Cyclic Magic, Special Circumstances); the engine applies
the modifier **unconditionally** to every matching cell — the read-out is a
toggleable addend in the UI's presentation, not a condition in the engine.

**Usage** 9.

---

## A31 `LabTotalMod`

**Serde tag** `lab_total_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i8` | no | Points added to (negative: removed from) the Lab Total. |

**Consumers**

- `derived.rs::in_play_mods` — `InPlayMods::lab_mod += amount`.
- `derived/lab.rs::lab_totals` — as the `lab_mod` addend (always present, even at 0).
- `derived/lab.rs::creo_corpus_lab_total` — the Longevity hint's base.
- Indirectly `derived/lab.rs::masterpiece_item_cap` and `derived/familiar.rs` (both read the Lab-Total grid).

**Arithmetic.** A single summed scalar, added into the base Lab Total
`Int + Magic Theory + Technique + Form + Aura + lab_mod`, **before** the
Magical-Focus double and **before** any halving.

It does **not** reach `effective/spell.rs::spell_level_cap`, even though that cap
is declared a Lab Total by `ArMDE:2465`. That cap is
`Te + Fo + Int + Magic Theory + 3`, halved if deficient — with no `lab_mod` term.

**When it fires.** In play only.

**Silently ignores.** Nothing of its own field, but see the `spell_level_cap`
divergence above.

**Usage** 9.

---

## A32 `DeficientArt`

**Serde tag** `deficient_art`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `param` | `String` | no | Param key naming the deficient Technique or Form. The declared param's **domain** (`technique` or `form`) is what enforces the class restriction. |

**Consumers**

- `effective/art.rs::deficient_arts` — the single fold, guarded on `selection.params.contains_key(param)`.
- `derived.rs::in_play_mods` — seeds `InPlayMods::deficient_arts` from that fold (its own match arm is an explicit no-op).
- `derived.rs::InPlayMods::deficient` — `contains(technique) || contains(form)`.
- `derived/casting.rs::casting_totals`, `::formulaic_casting_score`; `derived/lab.rs::lab_totals`, `::creo_corpus_lab_total`.
- `effective/spell.rs::spell_level_cap` — the **creation-time** per-spell level cap.
- `ruleset/integrity.rs::validate_effect_refs` → `::validate_deficient_art_effect` (load).

**Arithmetic.** The set of deficient Art ids. A total is halved **once** for the
pair, however many of its two Arts are deficient. Halving is
`derived.rs::halve` = `div_euclid(2)` — **floor**, not truncation, per `ArMDE:547`;
this matters because these totals are routinely negative and `/ 2` would round a
negative one in the character's favour. The same `div_euclid(2)` is used by
`spell_level_cap`.

Order: the halving is applied **after** every Casting-Score/Lab-Total addend
including the Magical-Focus double and the non-standard-casting penalties, and
**before** the spontaneous divisors. Under Weak Enchanter the Deficiency halves
first and the enchanting halving stacks on top (`floor(base/4)`); the same for
Difficult Longevity Ritual.

**Magic Resistance is excluded.** `magic_resistance` never consults
`deficient_arts` — correct per the Deficient Form text.

**When it fires.** In play **and** character creation (the spell-level cap). This
is the one variant the in-play block shares with the creation layer.

**Silently ignores.** A selection that has not filled `param` names no Art and
falls through — reported only by `validate_parameters`' generic `missing_param`.

**Usage** 2 — `flaw.deficient_technique`, `flaw.deficient_form`.

---

## A33 `MagicTotalHalving`

**Serde tag** `magic_total_halving`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `total` | `HalvableTotal` | no | `spontaneous_casting` / `lab_enchanting` / `lab_longevity` / `penetration`. |

**Consumers**

- `derived.rs::in_play_mods` — inserts into `InPlayMods::halvings`.
- `derived/casting.rs::casting_totals` (`spontaneous_casting`), `::penetration` (`penetration`).
- `derived/lab.rs::lab_totals` (`lab_enchanting` → the `enchanting` field), `::creo_corpus_lab_total` (`lab_longevity`).

**Arithmetic.** A `BTreeSet`, so **idempotent** — two copies halve once.

- `penetration`: `halve(casting − level + penetration_ability)`, i.e. **after** the level subtraction.
- `lab_enchanting`: `halve(total)` where `total` is already Deficiency-halved — the order the Flaw's own text specifies. Surfaced as `LabTotal::enchanting`; `total`/`within_focus` stay the ordinary Lab Total.
- `lab_longevity`: base → Deficient → Difficult, both floor-halving, so `floor(base/4)` when both apply. (The compounding is an explicit inference, documented as such in `creo_corpus_lab_total`.)
- `spontaneous_casting`: **not** a halving at all. Weak Spontaneous Magic removes the fatiguing option, so `spontaneous_fatiguing` reports the same ÷5 figure as `spontaneous_non_fatiguing` rather than a made-up ÷4.

Magic Resistance is deliberately **not** a member of `HalvableTotal`.

**When it fires.** In play only.

**Silently ignores.** Nothing of its own field, but note the `spontaneous_casting`
member does not do what its name says (see above) — it is a mode change, not a
halving.

**Usage** 4 — `flaw.weak_spontaneous_magic`, `flaw.weak_enchanter`,
`flaw.difficult_longevity_ritual`, `flaw.weak_magic`.

---

## A34 `SoakMod`

**Serde tag** `soak_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i8` | no | Points added to (negative: removed from) Soak. |

**Consumers**

- `derived.rs::in_play_mods` — `InPlayMods::soak_mod += amount`.
- `derived/combat.rs::soak` — the `soak_mod` addend.

**Arithmetic.** `soak = stamina + armor + soak_mod + bronze_cord + form_bonus`,
where `stamina` is the **aging-adjusted** effective Characteristic and
`form_bonus` is a hard `0` placeholder. Summed, unconditional, no clamp.

**When it fires.** In play only.

**Silently ignores.** Nothing of its own field. Note `virtue.berserk` carries a
`soak_mod` — the Berserk bonus is applied unconditionally rather than only while
berserk; the engine has no condition to gate it on.

**Usage** 3 — `virtue.tough`, `flaw.frail`, `virtue.berserk`.

---

## A35 `CombatMod`

**Serde tag** `combat_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `amount` | `i8` | no | Points added to (negative: removed from) the combat total. |
| `target` | `CombatStat` | no | `initiative` / `attack` / `defense` / `damage`. |
| `weapon` | `Option<Id>` | yes (omitted when `None`) | Scopes the modifier to one weapon's lines and **replaces** the same item's unscoped figure for that weapon and `target`. |

**Consumers**

- `derived.rs::in_play_mods` — `InPlayMods::combat_mods` (unscoped) and `::weapon_combat_mods` (scoped, stored as a **delta**).
- `derived/combat.rs::combat_totals` — the `cm(stat, weapon)` closure.
- `ruleset/integrity.rs::validate_effect_refs` (load: a named weapon must resolve, when the ruleset ships an equipment catalogue).

**Arithmetic.** Unscoped figures sum per stat. A scoped figure is stored as
`amount − (this item's own unscoped sum for that stat)`, so
`cm(stat, weapon) = Σ unscoped(stat) + delta(weapon, stat)` yields the scoped
figure on that weapon while every *other* item's unscoped contribution survives.
This is what "Lame is -3 on Dodge **and -1 on other combat scores**" means: the -1
never applied to Dodge.

An item may carry several `CombatMod` effects, one per affected `target`.

**When it fires.** In play only.

**Silently ignores.** Circumstance. Berserk's combat bonuses apply always. A
scoped modifier on a weapon the character has not equipped simply never appears,
with no diagnostic beyond the load-time id check.

**Usage** 17.

---

## A36 `HealthMod`

**Serde tag** `health_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `track` | `HealthTrack` | no | `fatigue_penalty` / `wound_penalty` / `fatigue_roll` / `casting_fatigue` / `recovery`. |
| `amount` | `i8` | no | Signed modifier; **positive reduces the penalty magnitude**. |

**Consumers**

- `derived.rs::in_play_mods` — `InPlayMods::health_mods[track] += amount`.
- `derived/combat.rs::fatigue_levels` (`fatigue_penalty`), `::wound_ranges` (`wound_penalty`).
- `derived.rs::surfaced_modifiers` — the other three tracks, as `ModifierFamily::HealthRoll` rows.

**Arithmetic.** Two of the five tracks are **computed**:

- `fatigue_penalty`: each tier's printed penalty becomes `(base + delta).min(0)` — Weary −1, Tired −3, Dazed −5; Fresh/Winded are 0. **Clamped at 0**: a Virtue can never turn a penalty into a bonus.
- `wound_penalty`: `(base + delta).min(0)` on Light −1, Medium −3, Heavy −5. Incapacitating and Dead carry `penalty: None` and are **unaffected**.

The other three — `fatigue_roll`, `casting_fatigue`, `recovery` — are
**surfaced-only**: listed labelled, never folded into a number.

**When it fires.** In play only.

**Silently ignores.** Three of its five tracks change no number. Within the two
computed tracks, the modifier applies to **every** tier/band uniformly; it cannot
be scoped to one.

**Usage** 12.

---

## A37 `MagicResistanceMod`

**Serde tag** `magic_resistance_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `kind` | `MagicResistanceEffect` | no | `no_form_bonus` / `halved_parma` / `aura_bonus` / `susceptible_faerie` / `susceptible_infernal` / `conditional_penetration_waiver`. |
| `param` | `Option<String>` | yes | Param key naming the **Form** the modifier is scoped to. Used by the first two kinds only; `None` for the rest. |

**Consumers**

- `derived.rs::in_play_mods` — splits by `kind`.
- `derived/casting.rs::magic_resistance` — reads `no_form_bonus_forms` and `halved_parma_forms`.
- `derived.rs::surfaced_modifiers` — the other four kinds.

**Arithmetic.** Per Form:

```
form_bonus  = 0 if the Form is in no_form_bonus_forms, else effective_art_score(Form)
parma_mr    = 5 × effective Parma Magica
parma_for_form = halve(parma_mr) if the Form is in halved_parma_forms, else parma_mr
base        = might if might > parma_for_form else parma_for_form
total       = form_bonus + base
```

Halving is floor (`div_euclid(2)`). `halved_parma` touches **only** the Parma
term, never the Form bonus; a Might base is never halved.

The Form travels through `param` into `selection.params`. A copy that names **no
Form** (missing or unfilled `param`) applies to **nothing** — deliberately, so the
engine does not guess a Form the save never stored; `validate_parameters` asks for
the choice instead.

The other four kinds (`aura_bonus`, `susceptible_faerie`, `susceptible_infernal`,
`conditional_penetration_waiver`) are **surfaced-only**, pushed with `amount: 0`
— so even a magnitude is not carried for them.

**When it fires.** In play only.

**Silently ignores.** Four of six kinds compute nothing. Both realm
susceptibilities — which the book says *halve* resistance against that realm — move
no number; the per-Form grid has no realm axis. The `aura_bonus` figure is lost
entirely (pushed as 0, not as the carrier's amount — the variant carries no amount
field at all).

**Usage** 7.

---

## A38 `AgingMod`

**Serde tag** `aging_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `kind` | `AgingEffect` | no | One of eight (below). |
| `amount` | `i8` | no | Signed modifier; **0 when the kind is itself the whole effect** (an immunity or a marker). |

**Consumers** — this is the most widely-consumed variant; the reader depends on `kind`.

| `kind` | Consumer | What it does |
|---|---|---|
| `aging_roll` | `aging.rs::aging_total` | `trait_modifier += amount`, **added** to the AGING TOTAL with its stored sign. |
| `longevity_bonus` | `aging.rs::aging_total` | `longevity_bonus += amount`, **only if the character actually holds a ritual**; the total then subtracts it. |
| `living_conditions` | `aging.rs::living_conditions_modifier` | `from_traits += amount`; the total **subtracts** the summed Living Conditions Modifier. |
| `no_aging` | `effective/warping.rs::suppresses_characteristic_aging` → `::aging_drops` | Returns 0 drops: Aging Points still accrue and still build Decrepitude, but never lower a Characteristic. |
| `no_apparent_aging` | `aging.rs::suppresses_apparent_aging` → `::resolve_outcome` | `AgingOutcome::apparent_age_increases` is false at every total. |
| `crisis_survival` | `aging.rs::crisis_survival` | Pushes a `CrisisModifier::Trait` of `amount` into the survival roll. |
| `crisis_heavy_wound` | `aging.rs::carries_crisis_heavy_wound` (via `::is_crisis_heavy_wound`) | A **predicate**: the crisis read-out states a Heavy Wound is owed. `amount` is ignored; the engine never writes the wound. |
| `decrepitude` | **none** | Surfaced only — see Part C. |

Every kind is *also* pushed into `derived.rs::surfaced_modifiers` as a
`ModifierFamily::Aging` row carrying `kind.to_string()` and `amount`.

**Arithmetic (the AGING TOTAL).**

```
uncapped = die + ceil(age / age_divisor)
         - living_conditions.total
         - longevity_bonus
         + trait_modifier
total    = min(uncapped, clamp.max_total)  // only with a ritual and age < clamp.until_age
```

The sign convention is load-bearing and the file states it explicitly: the two
*named-as-subtracted* modifiers keep the book's sign and are negated exactly once
here, so Mild Aging's **+1** Living Conditions modifier **lowers** the total while
Poor Living Conditions' **−1** raises it. Aging-roll modifiers are **added** with
their stored sign (Faerie Blood's −1 lowers the total directly).

**The two rolls are sealed off from each other.** `ArMDE:16636` — aging-roll
modifiers never reach the crisis survival roll, and a `crisis_survival` grant
never reaches the aging total. Mild Aging proves it: one sentence grants both, and
each lands on exactly one roll.

**The two immunities are orthogonal.** `no_aging` (Characteristics do not drop)
and `no_apparent_aging` (the appearance does not advance) are separate tags,
because Bound to (Role) has the first without the second and a Bee King has the
second without the first.

**When it fires.** In play (the aging step). None of it touches creation legality.

**Silently ignores.** `decrepitude` moves nothing anywhere (Part C). `amount` is
ignored for `no_aging`, `no_apparent_aging` and `crisis_heavy_wound` (all three
ship as 0, asserted by `crates/arm-rules/tests/data_integrity.rs`). Two traits
carrying `crisis_heavy_wound` still cost one wound.

**Usage** 17.

---

## A39 `AdvancementMod`

**Serde tag** `advancement_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `source` | `AdvancementSource` | no | `taught` / `book` / `vis` / `practice` / `adventure` / `insight` / `teaching` / `spell_mastery` / `all`. |
| `amount` | `i8` | no | Signed modifier to that source's Source Quality / advancement total. |

**Consumers**

- `derived.rs::in_play_mods` — pushes a `SurfacedModifier { family: Advancement, detail: source.to_string(), amount }`.
- `derived.rs::surfaced_modifiers` → `DerivedTotals::surfaced_modifiers` → the UI's read-out list.

**Arithmetic.** **None.** Surfaced-only: the app does not simulate advancement, so
the modifier is listed labelled and folded into no number. Both fields reach the
read-out intact.

**When it fires.** Neither creation nor any computed in-play total — display only.

**Silently ignores.** The whole mechanic. An entry using this variant is correctly
*recorded* but nothing is computed from it.

**Usage** 15.

---

## A40 `SpecialCastingMod`

**Serde tag** `special_casting_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `kind` | `SpecialCasting` | no | One of eleven (below). |
| `param` | `Option<String>` | yes | For `deft_form` only: the param key naming the affected Form (domain `form`). `None` for the unscoped quirks. |

**Consumers**

- `derived.rs::in_play_mods` — three kinds computed, eight surfaced.
- `derived.rs::InPlayMods::residual_voice_penalty` / `::residual_gesture_penalty` → `derived/casting.rs::casting_totals` (`NonStandardCasting`).
- `ruleset/integrity.rs::validate_effect_refs` — `deft_form` **requires** a `param` with domain `form`; a param-less `deft_form` fails the load loudly.

**Arithmetic — the three computed kinds.**

```
residual_voice_penalty(form)   = 0 if form ∈ deft_forms, else min(-10 + Σ quiet_words·5, 0)
residual_gesture_penalty(form) = 0 if form ∈ deft_forms, else min(-5  + Σ subtle_gestures·5, 0)
```

The constants are `derived.rs::NO_VOICE_PENALTY` (−10),
`::NO_GESTURE_PENALTY` (−5), `::QUIET_MAGIC_VOICE_REDUCTION` (+5),
`::SUBTLE_MAGIC_GESTURE_REDUCTION` (+5). Two castings of Quiet Magic eliminate the
no-voice penalty; the `.min(0)` clamp means a Virtue can never turn a penalty into
a bonus. The penalties are **Casting-Score terms**, folded in *before* the
Deficient-Art halving: `halve(score + penalty)`.

They surface as `NonStandardCasting { voice_penalty, gesture_penalty, silent,
still, silent_and_still, deft_form }` on the Formulaic total only — Words and
Gestures penalties never apply to Rituals (`ArMDE:9236`).

**The other eight kinds** — `diedne`, `faerie_raised`, `life_linked_spontaneous`,
`spell_improvisation`, `mercurian`, `life_boost`, `circumstantial`,
`doubled_aura_penalty` — are **surfaced-only**, pushed with `amount: 0`.

**When it fires.** In play only.

**Silently ignores.** Eight of eleven kinds compute nothing. The variant carries
no amount at all, so even a surfaced row has no magnitude — the *kind* slug is the
whole payload. The UI mirror in `ui/src/lib/types.ts` omits the `param` field from
this variant's TypeScript shape (Part C).

**Usage** 17.

---

## A41 `AbilityRollMod`

**Serde tag** `ability_roll_mod`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `param` | `String` | no | Param key whose **free-text** value names the subject/field (domain `text`). |
| `amount` | `i8` | no | Points added to rolls of the ability in that subject. |

**Consumers**

- `derived.rs::in_play_mods` — pushes `SurfacedModifier { family: AbilityRoll, detail: <the param's value, or "">, amount }`.
- `derived.rs::surfaced_modifiers` → the UI read-out.
- `ruleset/integrity.rs::validate_effect_refs` (load: `param` must exist with domain `text`).

**Arithmetic.** **None.** Surfaced-only. It modifies *rolls*, not the
bought/effective Ability score, so it deliberately never perturbs creation.

Note the effect declares no *which ability* field at all — only a free-text
subject. Which Ability the bonus applies to is carried by the item's name and
description, not by the data.

**When it fires.** Display only.

**Silently ignores.** The Ability the modifier applies to (not modelled), and the
whole mechanic (nothing adds `amount` to anything). A selection with an unfilled
`param` yields `detail: ""` — an empty subject in the read-out, no diagnostic.

**Usage** 1 — `virtue.academic_concentration_subject`.

---

## A42 `ElementalMagic`

**Serde tag** `elemental_magic`

**Fields**

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `forms` | `BTreeSet<Id>` | no | The elemental Form ids the XP redistribution pools over (Aquam, Auram, Ignem, Terram — data, not hardcoded). |

**Consumers**

- `effective/art.rs::elemental_magic_forms` (private) → `::elemental_form_bonus` → `::effective_art_score`, `::art_bonuses`.
- `ruleset/integrity.rs::validate_effect_refs` → `::validate_art_list_effect` (load: every Form id must resolve to a known Art).

**Arithmetic.** An **XP-space** boost, nonlinear in the bought score — which is
why it is not a flat `ArtBonus`. For a Form in the set:

```
own_xp   = art_table.xp_for_score(bought(form))
bonus_xp = Σ over the other pooled Forms of ceil(other_xp / 2)     // div_ceil — rounds UP
boosted  = art_table.score_for_xp(own_xp + bonus_xp)
bonus    = boosted - bought(form)
```

`ArMDE:3731` worked example: 21 XP → 11 bonus, i.e. `ceil(21/2)` — rounding is
**up**. `effective_art_score = bought + art_bonus + elemental_form_bonus`, so it
composes additively with Puissant Art. 0 for a non-elemental Art or a character
without the marker. `elemental_magic_forms` returns the **first** such effect
found and stops (an early `return` out of the macro).

Redistribution operates on the table-XP of the **whole bought score** — storage
keeps no raw assigned XP — so leftover XP between score thresholds is not
represented. This is a documented approximation (see `crates/arm-rules/RULES.md`).

**When it fires.** Character creation (it changes the effective Art score, which
every in-play total then reads). It is a no-op everywhere else — it does not
change the XP *charged*.

**Silently ignores.** A second `ElementalMagic` effect on another selection
(`elemental_magic_forms` returns on the first). The between-threshold XP remainder.

**Usage** 1 — `virtue.elemental_magic`.

---

# Part B — every other field of a V/F entry

Derived from the Rust struct `types.rs::PointItem`, not from a JSON sample. The
on-disk shape goes through `types.rs::PointItemRepr`
(`#[serde(try_from = "PointItemRepr")]`), which exists so the two ways a
catalogue can fail to state `categories` are reported **with the offending id**
rather than as a bare `missing field`.

**The complete field list**, with its serde behaviour:

| Field | Type | Required | Default / omission |
|---|---|---|---|
| `id` | `Id` | **yes** | — |
| `kind` | `ItemKind` | **yes** | — |
| `magnitude` | `Magnitude` | **yes** | — |
| `categories` | `Vec<String>` | **yes** | a missing or empty list fails the load, by id |
| `index_categories` | `Vec<String>` | no | `[]`, omitted when empty |
| `classification` | `Classification` | **yes** | no serde default — an unclassified entry fails to load |
| `tainted` | `bool` | no | `false`, omitted when false |
| `entity_kinds` | `BTreeSet<EntityKind>` | no | `{}`, omitted when empty (= any kind) |
| `prerequisites` | `Option<Prereq>` | no | `None`, omitted |
| `incompatible_with` | `BTreeSet<Id>` | no | `{}`, omitted when empty |
| `parameters` | `Vec<ParameterDef>` | no | `[]`, omitted when empty |
| `effects` | `Vec<Effect>` | no | `[]`, omitted when empty |
| `max_per_target` | `u8` | no | **1**, omitted when 1 |
| `max_total` | `u8` | no | **255** (`u8::MAX` = "no stated ceiling"), omitted when 255 |
| `max_share_of_kind` | `Option<Share>` | no | `None`, omitted |
| `source` | `Option<SourceRef>` | no | `None`, omitted |

A key the struct does not declare is a **hard load error** only for the one
removed key `category` (singular), which `PointItemRepr` accepts solely in order
to reject it by name. Other unknown keys are ignored by serde.

---

## B1 `classification` — **read by no production code at all**

This is the section the audit's severity ratings hang on, so it is stated
flatly and the evidence is given.

**The finding: nothing in `crates/*/src` or `ui/src` reads
`PointItem::classification` at runtime.** It gates nothing, filters nothing, and
changes no computed number or rendered string.

**How that was established.** Three passes, two tools:

1. `rg -n "\.classification|Classification::"` over `crates/arm-rules/src`,
   `crates/arm-app/src`, `crates/*/tests`. Every hit in `src` is in
   `types.rs` — the enum declaration, its doc comments, and its `Display` impl —
   plus `types.rs`'s own `#[cfg(test)]` module. Every other hit is in
   `crates/arm-rules/tests/data_integrity.rs`.
2. `grep -ra -l "classification"` (with `-a`, so a NUL-bearing file could not be
   silently skipped) over the same roots plus `ui/src`. It lists ~19 `.rs` files,
   but every one of those beyond `types.rs` matches only the string
   `"classification"` **inside a JSON test fixture** in a `#[cfg(test)]` module —
   confirmed by pass 1 finding no field access in any of them.
3. `rg -a "classification" ui/src --glob '*.svelte'` → **no hits at all**. In
   `ui/src` the only non-test occurrence is the type declaration
   `ui/src/lib/types.ts` line "classification: Classification;" plus the
   `Classification` type alias. No component, store, or helper reads it.

**Who does read it, then:**

- **serde, at load.** It is required (no `#[serde(default)]`), so an entry
  omitting it fails the whole ruleset load. That is the *only* runtime
  consequence of the field.
- **The IPC boundary.** `PointItem` is serialized whole into the frontend's
  `Ruleset.point_items`, so the value crosses to the UI and is typed in
  `ui/src/lib/types.ts` — and is then read by nothing.
- **`crates/arm-rules/tests/data_integrity.rs::every_vf_is_classified`** — the
  partition guard. It asserts: all four classes are non-empty; the catalogue is
  larger than 600 entries (never an exact total); **an entry carrying `effects`
  is neither `narrative` nor `uncomputed_rule`**; and **every `in_play_effect`
  entry carries at least one effect**. Note the asymmetry — there is **no**
  assertion that a `creation_effect` entry carries effects, which is why the five
  effect-less `creation_effect` entries named in `README.md` are green today.
- **`crates/arm-rules/tests/uncomputed_clauses.rs`** — reads the field out of the
  raw JSON (`item["classification"] == …`), not through the Rust type. It asserts
  that every `uncomputed_rule` entry's displayed text (`description`, else
  `summary`) contains a mechanical token, **in every shipped locale**; and, scoped
  to `SWEPT_BLOCKS`, that no `narrative` entry's cited passage carries one.

**What this means for a batch agent's severity rating.** A misclassification
cannot produce a wrong number, because no number depends on it. Its real cost is
twofold and both halves are test-side:

- Reclassifying **into** `uncomputed_rule` obliges you to write the rule into
  `description` in **both** locales, or `uncomputed_clauses.rs` goes red. That is
  the mechanism by which the class stays honest.
- A wrong `narrative` is the defect the audit exists for: it asserts the rulebook
  said nothing mechanical, which suppresses the obligation to carry the rule in
  the description — so the rule leaves the application silently and the entry
  still looks complete.

So: **`classification` is documentation plus a hook for two test-side guards.**
Rate a misclassification as a data/provenance defect and a lost-rule risk, never
as a miscalculation. It is still high-value — a `narrative` that should be
`uncomputed_rule` means a real rule reaches no user — but the mechanism is the
missing *text*, not a missing *computation*.

**The four values** (`types.rs::Classification`, serde `snake_case`):

| Value | The claim it makes |
|---|---|
| `narrative` | The cited passage states **no mechanical clause at all**. The doc is explicit that this last condition is load-bearing and easy to misread: an entry whose text states a signed modifier, a botch-dice change or a cap is **not** narrative even when the engine computes nothing. |
| `uncomputed_rule` | The passage states a real rule that is *genuinely uncomputable at character-generation time* — botch dice, scene-/activity-contingent modifiers, GM judgement, open-ended magnitudes. Carries **no** effects, like `narrative`; the difference is entirely about whether the **rulebook** said something. |
| `creation_effect` | Changes a character-creation number or state. |
| `in_play_effect` | Does not change a creation number, but modifies an in-play/derived total the engine computes. |

The line between `uncomputed_rule` and `in_play_effect` is stated in the enum
doc: **carrying an `Effect` is what decides it** — an entry the engine computes
*something* for is `in_play_effect` even if its passage also contains an
uncomputable clause.

---

## B2 `kind`

**Type** `ItemKind` — `virtue` | `flaw` | `boon` | `hook` (serde `snake_case`).
Required.

**Consumers**

- `validation/balance.rs::compute_balance` — `item.kind.is_positive()` decides which side of the balance the points land on. Virtue and Boon are positive; Flaw and Hook are negative.
- `validation/caps.rs::validate_caps` — the three hard count caps (`max_major_virtues`, `max_major_flaws`, `max_minor_flaws`) and the data-driven per-category caps all match on `kind`.
- `validation/caps.rs::validate_tainted_cap`, `::validate_share_of_kind_cap` — split their totals by kind.
- `grant.rs::GrantConstraint` — an open grant (warping fill, House/Mythic slot) requires a matching `kind`.
- The UI's V/F tab partitions the picker by kind.

**Arithmetic.** No number of its own; it selects which running total a
magnitude's points join.

**Ignores.** Nothing. Note that `boon`/`hook` are the covenant-side kinds and
behave exactly as virtue/flaw for balance.

---

## B3 `magnitude`

**Type** `Magnitude` — `free` | `minor` | `major` (serde `snake_case`). Required.

**Point values.** `Magnitude::points()` — `free` = **0**, `minor` = **1**,
`major` = **3**. Surfaced to the frontend as `Ruleset.magnitude_points` so the UI
never re-hardcodes them (a CLAUDE.md invariant).

**Consumers**

- `validation/balance.rs::compute_balance` — the points summed per side.
- `validation/caps.rs::validate_caps` — the Major/Minor count caps match on it.
- `validation/caps.rs::validate_tainted_cap` — points, not headcount, so a `free` item never affects the ratio.
- `validation/caps.rs::validate_share_of_kind_cap` — likewise points.
- `grant.rs::GrantConstraint::magnitude` — an open grant may require one.
- `ruleset/integrity.rs::validate_magnitude_variant_exclusivity` — `<stem>_major`/`<stem>_minor` (and `major_<stem>`/`minor_<stem>`) pairs must list each other in `incompatible_with`, checked only when both members exist.

**Budget arithmetic** (`validation/balance.rs::effective_budget`):

```
rate            = profile.budget.virtue_points_per_flaw_point      // default 1
virtue_ceiling  = profile.virtue_points + bonus_flaw·rate + bonus_free_virtue
flaw_ceiling    = profile.flaw_points   + bonus_flaw
funded(flaw)    = flaw·rate + bonus_free_virtue
```

Three separate errors: `over_budget_virtues`, `over_budget_flaws`, and
`unbalanced_virtues` (virtues must be *funded* by flaws — `ArMDE:2774`). The
mythic bonuses apply only when the profile's `has_mythic_type` is set, so a stray
`mythic_type` on a hand-edited save cannot inflate a budget.

**Scope.** `compute_balance` and `validate_caps` iterate **`entity.selections`
(bought only)** — House/Mythic/`grants_selection` grants are deliberately
budget-exempt and category-cap-exempt. `validate_share_of_kind_cap` takes the
**folded** list, deliberately, because a free copy is still a copy.

---

## B4 `entity_kinds`

**Type** `BTreeSet<EntityKind>`. Optional; **empty means "any kind"**.

**Consumers**

- `validation/selections.rs::validate_entity_kind_applicability` — the only reader. Errors `wrong_entity_kind` when the set is non-empty and does not contain `entity.entity_kind`.
- The UI's Available picker filters on it.

**Arithmetic.** A membership test on bought `entity.selections` only.

**Ignores.** Grants. A House- or warping-granted item of the wrong entity kind is
not flagged.

---

## B5 `categories` and `index_categories`

**`categories`: `Vec<String>`, required, non-empty, no repeats** (load-time
integrity rejects both). Order is the descriptor's own and is deliberately
**exempt from canonical sorting**.

**All listed categories are equally real.** Membership tests read the **whole
list**: permitted/forbidden categories, category caps, grant constraints, Gift
detection, and the UI's Available picker (which offers a dual-category item under
both headings). `categories[0]` carries **no rules meaning** — it is only a
deterministic tie-break in the two places that structurally have room for exactly
one (`types.rs::PointItem::first_listed_category`).

**Consumers**

- `validation/selections.rs::validate_permitted_categories`, `::validate_forbidden_categories` — against the type profile's lists (with conditional `CategoryRule`s evaluated through `PrereqCtx`).
- `validation/caps.rs::validate_caps` — the data-driven per-category caps (`too_many_<category>_<flaws|virtues>`, or `too_many_major_<category>_…`). The code is *derived from the slug*, so no category is hardcoded in Rust.
- `effective/gift_confidence.rs::has_the_gift` — via the profile's `gift_categories`.
- `grant.rs::GrantConstraint::require_categories` / `forbid_categories`.
- `types.rs::ParameterDef::require_categories` (for `item`-domain params).
- `ruleset/accessors.rs::items_by_category`, the UI picker, the Markdown export's Type cell.

**`taken_as` awareness.** Five membership sites resolve a multi-category item
through `types.rs::PointItem::categories_for`, which consults a
`ParameterDomain::Category` param recording which single reading the player chose
(Sufi as Social Status *or* Supernatural, `ArMDE:5083`). Browsing surfaces
(`items_by_category`, the picker, the export Type cell) deliberately stay
whole-list.

**`index_categories`: `Vec<String>`, optional, canonically sorted.**
**Provenance, not membership.** It records headings the book's own *index* files
the entry under, beyond its descriptor's membership categories. **Exactly one
consumer**: `validation/magus.rs::validate_house`'s `ArMDE:2860` "at least one
Hermetic Flaw" guideline. Every membership surface, every browsing surface and
`categories_for` are deliberately blind to it. Load-time integrity rejects a
repeat and rejects a slug already in `categories`.

---

## B6 `prerequisites`

**Type** `Option<Prereq>`, a recursive boolean tree. Variants (`types.rs::Prereq`,
adjacently tagged — `{ "kind": …, "value": … }`):
`all` | `any` | `none` (the Rust variant is `Nor`) | `has` | `house` |
`ability_min` | `art_min` | `is_magus`.

**Consumers**

- `validation/prereq.rs::validate_prerequisites` → `::evaluate_prereq`.
- `validation/selections.rs::categories_in_force` — conditional `CategoryRule`s reuse the same evaluator.
- `validation/selections.rs::validate_possessed_param_targets` — reuses `PrereqCtx::present_ids`.
- `ruleset/integrity.rs::validate_prereq_refs` (load: every referenced id must resolve; nesting past `types.rs::PREREQ_MAX_DEPTH` — 32 — is rejected).
- `ui/src/lib/derive.ts` mirrors the evaluation for the picker (guarded by `ui/src/lib/prereq-parity.test.ts`).

**Evaluation is tri-state** (`Tri::True` / `False` / `Unknown`), not boolean:

| Variant | Semantics |
|---|---|
| `all` | AND. Short-circuits **False** on a False child; all-known → True. |
| `any` | OR. Short-circuits **True** on a True child; all-known → False. |
| `none` (`Nor`) | Short-circuits **False** on a True child; all-known → True. |
| `has` | `present_ids.contains(id)` — the **grants-inclusive** set (bought ++ House/Mythic/`grants_selection`/warping-fill grants). Never `Unknown`. |
| `house` | matches → True, differs → False, **no house at all → `Unknown`**. |
| `ability_min` | Compared against the max **effective** score (bought + Puissant), so a boosted Ability satisfies it. An unheld Ability is 0. Never `Unknown`. |
| `art_min` | Same, for Arts. |
| `is_magus` | `profile.is_magus`; **`Unknown` when the profile is missing**. |

A surviving `Unknown` makes the whole expression `Unknown`.

**Outcomes.** `False` → `prereq_not_met` (**error**). `Unknown` **and** it hinged
on genuinely missing data → `prereq_unevaluated` (**warning**). `Unknown` that did
not → nothing. `True` → nothing.

Depth is capped at `PREREQ_MAX_DEPTH`; exceeding it yields `Unknown` rather than
overflowing the stack (load-time validation already rejects such a tree).

**Scope.** Iterates **bought** `entity.selections`. A granted item's own
prerequisites are not checked.

---

## B7 `incompatible_with`

**Type** `BTreeSet<Id>`. Optional. **Must be symmetric** — `ruleset/integrity.rs::validate_incompatibility_symmetry`
fails the load on a one-sided declaration.

**Consumers**

- `validation/prereq.rs::validate_incompatibilities` — the only runtime reader. Emits `incompatible` (**error**), with the pair order normalised so a mutual declaration is reported **once**.
- `ruleset/integrity.rs::validate_incompatibility_symmetry` and `::validate_magnitude_variant_exclusivity` (load).

**Arithmetic.** A set-membership test of each bought item's list against
`selected_ids`.

**Scope — this one matters.** `selected_ids` is built in
`validation/mod.rs::validate` as `entity.selections.iter().map(|s| &s.item_ref)`
— **bought only**. So an incompatibility between a bought item and a *granted*
one is **not** detected, even though `Prereq::Has` in the same validation run
*does* see grants (`PrereqCtx::present_ids`). The divergence is deliberate and
documented for the forbidden-trait check; for incompatibilities it is recorded
here as a scope fact (see Part C).

**Magnitude-variant pairs.** `<stem>_major`/`<stem>_minor` and
`major_<stem>`/`minor_<stem>` must be mutually incompatible, enforced at load —
only when both members exist, and only inspecting the Major side so each pair is
reported once.

---

## B8 `parameters`

**Type** `Vec<ParameterDef>`. Optional.

### `ParameterDef`'s own fields

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `key` | `String` | no | The key the selection's `params` map must use. |
| `type` (`param_type`) | `ParamType` | yes (defaults to `Ref`) | The only current variant. |
| `domain` | `ParameterDomain` | no | What the value resolves against. |
| `values` | `Vec<Id>` | yes | The closed list — **`enumerated` domain only**. Load-time integrity rejects an empty list here *and* a list on any other domain. |
| `at_most_one_of` | `Vec<BTreeSet<Id>>` | yes | Groups of which at most one may be named across **all copies** of the item. Each group must name ≥ 2 values, each resolving in the domain. Enforced by `validation/selections.rs::validate_exclusive_param_values` → `exclusive_param_values`. |
| `max_per_value` | `u8` | yes (default **255** = no stated ceiling) | How many copies may name one and the same value **for this key**. Enforced by `validation/selections.rs::validate_per_value_cap` → `too_many_for_param_value`. |
| `require_categories` | `BTreeSet<String>` | yes | **`item` domain only.** The named point item must carry one of these categories. |
| `require_possessed` | `bool` | yes | **`item` domain only.** The named item must be one the entity *holds* (grants-inclusive, `PrereqCtx::present_ids`), and each held target may be claimed **once**. Enforced by `validation/selections.rs::validate_possessed_param_targets` → `param_target_not_possessed` / `param_target_already_claimed`. |
| `forbid_tainted` | `bool` | yes | **`item` domain only.** The named item may not be `tainted`. Reported as `unknown_param_value`. |
| `require_power` | `bool` | yes | **`text` domain only.** The value must name an `Entity::powers` entry exactly (trimmed, case-sensitive). Enforced by `validation/selections.rs::validate_power_targets` → `power_dangling_target`, on the Review phase. |

Load-time integrity rejects each of `require_categories` / `require_possessed` /
`forbid_tainted` on any domain but `item`, and `require_power` on any domain but
`text` — precisely so a restriction cannot *look* enforced while nothing reads it.

### The ten domains (`types.rs::ParameterDomain`, serde `snake_case`)

`ability` · `art` · `technique` (Art, additionally required to be a Technique) ·
`form` (Art, required to be a Form) · `characteristic` (parsed via
`Characteristic::from_id`, no registry) · `item` (the point-item catalogue) ·
`enumerated` (the parameter's own `values`) · `category` (the *declaring item's
own* `categories`; must also cap `max_total` at 1) · `realm` (`Realm::from_id`) ·
`text` (free text; **any value with non-whitespace content is legal**, and an
empty/whitespace-only value is reported as `missing_param`, not as an unknown
value).

### How a parameter's choice reaches an effect

A `Selection` carries `params: BTreeMap<String, Id>`. An effect names a **key**,
and the engine resolves `selection.params.get(param)` at fold time. **Eleven of
the 42 effect variants are parameter-relative:**

| Variant | Required domain (enforced at load) |
|---|---|
| `ability_bonus` | `ability` |
| `characteristic_score_delta_param` | `characteristic` |
| `art_bonus` | `art` |
| `affinity_ability_cost` | `ability` |
| `affinity_art_cost` | `art` |
| `magical_focus` | `text` |
| `ability_roll_mod` | `text` |
| `deficient_art` | `technique` **or** `form` (the declared domain fixes the class) |
| `special_casting_mod` with `kind: deft_form` | `form`, and the `param` is **mandatory** |
| `magic_resistance_mod` with `kind: no_form_bonus` / `halved_parma` | the `param` is optional in the type; when absent or unfilled the modifier applies to **nothing** |
| *(none other)* | — |

`ruleset/integrity.rs::validate_effect_refs` is the gate: for each of these it
requires the named key to exist on the item **and** to carry the expected domain,
failing the load otherwise. The other 31 variants either carry a directly-stored
ref (validated separately — `ability_score_grant`, `restricted_ability_xp`,
`group_affinity_cost`, `grants_selection`, `elemental_magic`,
`characteristic_score_delta`, `combat_mod`'s `weapon`) or need no resolution at
all.

### What happens to a parameter no effect consumes

**Nothing at all, and nothing warns.** The relationship is one-directional: an
*effect* must name a declared parameter with a matching domain, but a *parameter*
need not be named by any effect. Declaring a parameter that no effect reads is
legal, loads cleanly, and produces a picker the player must fill whose value is
then used only by:

- `validate_parameters` / `validate_selection_parameters` (the value must resolve
  in its domain, and every declared key must be present and non-blank);
- `max_per_target`'s duplicate key (the *whole* `(item_ref, params)` tuple), so an
  unconsumed parameter still distinguishes two copies as separate targets;
- `max_per_value` and `at_most_one_of`, if declared;
- `categories_for`, for the `category` domain;
- display (the picker and the Markdown export render the chosen value).

This is the normal and intended shape for the ~large majority of shipped
parameterized entries, whose parameter exists to record a player choice the
engine does not compute from (a (Realm), a (Land), an (Ability) subject). It is
**not** evidence of a bug on its own — but it *is* the shape a genuinely
mis-wired entry takes, so a batch agent must check the passage, not the data
alone.

---

## B9 `tainted`

**Type** `bool`. Optional, defaults `false`, omitted from JSON when false.

**Meaning.** The descriptor's "Type" tag: associated with the Infernal realm
(`ArMDE:2998-3002`); any Supernatural Ability such an item grants is an Infernal
power.

**Consumers**

- `validation/caps.rs::validate_tainted_cap` — no more than half a character's Virtue points (and likewise Flaw points) may be Tainted. A **warning**, because the book says "should". Compared as `2·tainted > total` (integer, no rounding). Measured against **points actually taken**, not the budget — so a `free` item never affects the ratio. Iterates bought `entity.selections`.
- `validation/selections.rs::param_value_resolves` — via a parameter's `forbid_tainted`.
- The UI badges it.

---

## B10 `max_per_target`, `max_total`, `max_share_of_kind`

Three *different* multiplicity axes. Getting them mixed up is the commonest
authoring error, so the keys are:

| Field | Grouping key | Default | Validator | Issue code |
|---|---|---|---|---|
| `max_per_target` | `(item_ref, **whole** params map)` | **1** | `validation/selections.rs::validate_duplicate_selections` | `duplicate_selection` (error) |
| `max_total` | `item_ref` alone | **255** (sentinel: no stated ceiling) | `validation/selections.rs::validate_total_selection_cap` | `too_many_selections` (error) |
| `ParameterDef::max_per_value` | `(item_ref, one named key's value)` | **255** | `validation/selections.rs::validate_per_value_cap` | `too_many_for_param_value` (error) |
| `max_share_of_kind` | points held by one `item_ref`, vs. its own kind's total | `None` | `validation/caps.rs::validate_share_of_kind_cap` | `too_large_share` (**warning**) |

**All four are grant-aware** — they take the folded bought-plus-granted list,
because a free copy is still a copy. (This is the deliberate divergence from
`compute_balance` and `validate_caps`, which are bought-only.)

`max_share_of_kind` is a `Share { numerator, denominator }`, compared as
`points · denominator > total · numerator` — integer, no rounding choice.
Load-time integrity rejects a zero denominator and a numerator above its
denominator.

`max_per_value` **tests for the 255 sentinel** rather than comparing against it,
so a crafted save holding 256 copies of one value cannot trip a ceiling the rules
never state. It also counts **distinct parameter tuples** (capped at
`max_per_target` per tuple), so an identical repeat stays `max_per_target`'s
finding and one mistake draws one finding.

---

## B11 `source`

**Type** `Option<SourceRef>`. Optional.

**`SourceRef`'s fields** (alphabetical, so the serialized form is canonical
without a custom `Serialize`):

| Field | Type | Optional | Meaning |
|---|---|---|---|
| `anchor` | `Option<String>` | yes | The Markdown heading anchor (`abandoned-apprentice` for `#### Abandoned Apprentice`) — **the durable half**. Always the **English** anchor. Present only for entries a sweep has already read. |
| `file` | `String` | no | Basename of the Markdown source file, relative to `rules/source/<lang>/`. Always the **English** file. |
| `lines` | `LineRange` | no | Inclusive `[start, end]`, 1-based. Serialized as a two-element array. |

**Consumers.** None at runtime — it changes no number and gates nothing. It is
read by the provenance guards (`crates/arm-rules/tests/rules_source_provenance.rs`,
`rules_md_citations.rs`) and by the audit's own batch ordering
(`sort_by(.source.lines[0], .id)`).

**Why `anchor` exists alongside `lines`.** `lines` is a *derived* coordinate: an
edit anywhere above the item shifts it, and the guards can only prove a range
lands on non-blank lines, never that it lands on the rule the citation claims. An
anchor survives an edit elsewhere in the book and, when it does break (a renamed
heading), fails **loudly**. That asymmetry is the whole argument for carrying
both — and it is directly relevant to this audit: a batch agent verifying a
passage must confirm the *anchor* names the entry, not just that the line range
is non-blank.

---

## B12 `id`

**Type** `Id` (a newtype over `String`). Required. Slug-style, never translated:
`virtue.puissant_ability`, `flaw.deficient_technique`.

It is the join key for everything: the `rules/i18n/<lang>/` text layer, every
`has` / `incompatible_with` / parameter `ref`, `grants_selection`, House and
Mythic-type grants, and the frontend's `point_items` map. The `virtue.` / `flaw.`
prefix carries **no** meaning in code — `kind` is the field that decides that.
`ruleset/integrity.rs::validate_magnitude_variant_exclusivity` is the one place
the id's *shape* is parsed (for `_major`/`_minor` stem pairing).

---

# Part C — the census of systemic gaps

A defect here makes **every** entry using the variant wrong at once, while each
entry's own data looks perfectly correct. That is why it is found here and not
entry by entry.

Every claim below states what was searched for and why the absence is real.
**Every negative was confirmed with two tools** — the native `rg`-backed Grep and
a `grep -a` (which overrides the injected `grep`'s hardcoded `-I`, so a
NUL-bearing file cannot be silently skipped) — because an empty `grep` result is
not proof of absence in this environment.

**Then the whole of Part C was attacked.** A separate pass was run with the sole
brief of falsifying every claim here, independently, and it overturned or
sharpened six of them plus contributed one finding the draft had missed (C2-e).
See the **Correction log** at the end for exactly what changed. Claims that
survived that pass are marked nowhere special — they are simply the ones left.

## C1 — Dead variants (declared and parsed, no consumer)

**None.** All 42 variants have at least one production reader.

*Evidence.* `grep -rHo "Effect::[A-Z][A-Za-z]*" crates/arm-rules/src
crates/arm-app/src | sort -u` produced the full file × variant matrix; every
variant appears in at least one non-test file, and each was then read in place to
confirm the arm is not an inert `=> {}`. Cross-checked against
`jq` over `rules/core/virtues_flaws.json`, which shows all 42 tags in use.

**But five *values inside* variants are dead**, which has the same
entry-level consequence. An entry authoring one of these is inert:

| Dead value | Variant | Only "reader" |
|---|---|---|
| `AgingEffect::Decrepitude` | `aging_mod` | The generic `surfaced` push in `derived.rs::in_play_mods` (`kind.to_string()`). Both `aging.rs::aging_total` and `::crisis_survival` list it in an explicit `=> {}` arm; no third reader exists. |
| `MagicResistanceEffect::AuraBonus` | `magic_resistance_mod` | Surfaced with `amount: 0` — and the variant carries no amount field, so the bonus's *size* is not even representable. |
| `MagicResistanceEffect::SusceptibleFaerie` / `::SusceptibleInfernal` | `magic_resistance_mod` | Same. The book halves resistance against that realm; `derived/casting.rs::magic_resistance` has no realm axis, so nothing halves. |
| `MagicResistanceEffect::ConditionalPenetrationWaiver` | `magic_resistance_mod` | Same. Documented as necessarily surfaced-only (its inputs are scene facts). |
| `HealthTrack::FatigueRoll` / `::CastingFatigue` / `::Recovery` | `health_mod` | `derived.rs::surfaced_modifiers` only. Note the mechanism: `derived.rs::in_play_mods` folds **every** track into `InPlayMods::health_mods`; the three die one level later, where `surfaced_modifiers` routes them to the read-out and sends `FatiguePenalty \| WoundPenalty` to an empty arm. `derived/combat.rs` reads exactly those latter two. |
| 8 of 11 `SpecialCasting` kinds (`diedne`, `faerie_raised`, `life_linked_spontaneous`, `spell_improvisation`, `mercurian`, `life_boost`, `circumstantial`, `doubled_aura_penalty`) | `special_casting_mod` | Surfaced with `amount: 0`. |

*Evidence for `AgingEffect::Decrepitude`:*
`rg -rn "CrisisHeavyWound\|AgingEffect::Decrepitude" crates/ ui/src` returns four
hits, all of them the two explicit no-op arms in `aging.rs` plus one test fixture
and one doc comment — no computing reader. `CrisisHeavyWound`, searched the same
way, *does* have one (`aging.rs::carries_crisis_heavy_wound`), which is what makes
the absence for `Decrepitude` meaningful rather than a search artefact.

**Which of those dead values actually have shipped entries**, so the census is not
inflated. Per `jq` over the catalogue, `aging_mod` kinds in use are: `aging_roll`
8, `living_conditions` 3, `no_aging` 2, `no_apparent_aging` 2,
`crisis_heavy_wound` 1, `crisis_survival` 1 — and **`decrepitude` 0**. So
`AgingEffect::Decrepitude` is the one row in the table above with **no entry-level
consequence at all**: no shipped entry authors it. Every other dead value *is*
authored (`aura_bonus` 2, each susceptibility 1, `conditional_penetration_waiver`
1, `fatigue_roll` 3, `casting_fatigue` 3, `recovery` 2, the eight surfaced-only
`SpecialCasting` kinds 14 between them), so those do reach real entries.

`AgingEffect::LongevityBonus` is a different case again: **0 shipped uses**, but it
*is* consumed (`aging.rs::aging_total` moves the ritual term with it). Live, merely
unexercised by the data.

**The four closed taxonomies not listed above were swept and are clean**, so a
later reader need not redo it: every value of `CastingScope`
(`derived.rs::CastType::matches`), `HalvableTotal` (`derived/casting.rs` +
`derived/lab.rs`) and `CombatStat` (`derived/combat.rs`) is consumed.
`AdvancementSource` is uniformly dead-as-a-number, which is C1-e's finding about
`advancement_mod` restated at value level rather than a separate one.

**Two whole variants compute nothing anywhere** (they are not dead — they are
read into the read-out — but they move no number):

- `advancement_mod` (**15 uses**) — surfaced-only by design.
- `ability_roll_mod` (**1 use**) — surfaced-only by design.

## C2 — Half-read variants (a consumer exists but ignores a field)

| Variant | Ignored | Consequence |
|---|---|---|
| **`warping_grant`** | `score` | `effective/warping.rs::warping_grant_points_in` destructures it as `score: _`. The Warping Score is derived from **points** through the advancement curve, so an authored `score` is discarded silently. Deliberate and documented — but an entry whose data says `score: 3` gets whatever the curve makes of its `points`. |
| **`magical_focus`** | `major` — **and `param` at runtime** | No production site binds `major`: every pattern is `{ .. }` or `{ param, .. }`. Verified twice (`rg -a "MagicalFocus"` and `grep -ran "MagicalFocus"`), 13 hits, none binding it. So a **Major** and a **Minor** Focus compute *identically* — both add `min(te, fo)` once. `param` is validated at load and then never read: the descriptor text is never matched against a Technique/Form pair, so the within-focus figure is offered for **every** cell. |
| **`grants_reputation`** | `score`, for enforcement | `reputation_grants` carries it to the frontend, but `validation/scores.rs::validate_reputations` counts **kinds and slots only**. A character may enter a Local Reputation of 10 backed by a grant of 3 and nothing objects. 33 uses — the most-used variant in the catalogue. |
| **`aging_mod`** | `amount`, for three kinds | `no_aging`, `no_apparent_aging` and `crisis_heavy_wound` are markers; `amount` is never read for them. Deliberate, ships as 0, and `crates/arm-rules/tests/data_integrity.rs` pins that. Listed for completeness, not as a defect. |
| **`special_casting_mod`** | `param`, for every kind but `deft_form` | `derived.rs::in_play_mods` reads `param` only in the `DeftForm` arm. `ruleset/integrity.rs::validate_effect_refs` requires a `form`-domain param for `deft_form` and *rejects* a param-less one — but a `param` on any **other** kind falls into the catch-all `Effect::SpecialCastingMod { .. } => continue` arm and is neither validated nor read. Latent only: **no** shipped non-`deft_form` entry carries a `param`. |

**C2-e — `magic_resistance_mod`'s `param` is read at runtime but validated at
load by nothing. This is the exact inverse of C2-d's asymmetry, and unlike C2-d it
is live in the shipped data.**

`derived.rs::in_play_mods` resolves `param` through `selection.params` for
`NoFormBonus` and `HalvedParma`, and those two Forms are what
`derived/casting.rs::magic_resistance` scopes the Flaw to. But
`ruleset/integrity.rs::validate_effect_refs` routes
`Effect::MagicResistanceMod { .. }` into its **"no parameter or ref to resolve"**
catch-all arm — so the key is never checked to exist on the item and never
checked to carry the `form` domain.

Compare `deft_form`, one variant over: it gets a *required*-param error **and** a
`ParameterDomain::Form` check, justified in the integrity source by exactly the
argument that applies here — "a param-less `deft_form` would silently never apply
its waiver". A `magic_resistance_mod` naming a key the item does not declare, or
one carrying the wrong domain, applies to **no Form at all** and loads clean.

Live in the data: `flaw.flawed_parma_magica` and `flaw.limited_magic_resistance`
each declare a `form`-domain parameter keyed `"form"` and each carry
`"param": "form"`. Correct today — and nothing would catch a typo.

## C3 — Phase-limited variants (consumed in one phase where the data implies both)

**C3a — `lab_total_mod` does not reach the creation-time spell-level cap.
This is the strongest finding in Part C, and the engine's own comment is the
argument against it.**

`effective/spell.rs::spell_level_cap` computes
`Te + Fo + Int + Magic Theory + 3`, halved if either Art is Deficient. Its doc
comment justifies the halving by quoting `ArMDE:2465`:

> "This is the appropriate Lab Total, assuming an aura modifier of +3, and thus
> any Virtues and Flaws your character has apply to this total if they would
> apply to a Lab Total in play"

By that sentence, **everything** that moves a Lab Total in play should move this
cap. Three things that do move `derived/lab.rs::lab_totals` do **not** appear in
`spell_level_cap`:

1. `lab_total_mod` (the `mods.lab_mod` addend) — **9 uses**, e.g. Inventive Genius +3.
2. `magical_focus` (the `min(te, fo)` double).
3. `magic_total_halving` with `lab_enchanting` / `lab_longevity`.

Of these, (2) and (3) have defensible reasons to be absent (a focus applies only
within its descriptor, and the two halvings are scoped to enchanting and to
longevity rituals, neither of which is learning a spell). **(1) has none**:
`lab_total_mod` is an unconditional flat modifier to every Lab Total, and
Deficiency — which *is* applied here — is cited under the same sentence.

*Evidence.* `spell_level_cap` was read in full; its `base` expression names five
terms and `mods`/`in_play_mods` is never called from `effective/spell.rs`
(`rg -n "in_play_mods" crates/arm-rules/src/effective/` → no hits; the only
`effective`-side reader of the in-play fold is `deficient_arts`, which lives in
`effective/art.rs` and is called *by* `derived.rs`, not the reverse).
`spell_level_cap` has exactly two callers —
`effective/spell.rs::spell_level_caps` and
`validation/magus.rs::validate_spell_level_cap` — and neither adds `lab_mod`.

**And `crates/arm-rules/RULES.md` records no decision to exclude it.** Its
entry for this cap quotes the same `ArMDE:2465` sentence, justifies the Deficiency
halving with it, and names exactly **one** acknowledged approximation —
*requisite-Art reduction* — saying nothing about `lab_total_mod`. The two places
RULES.md does mention `LabTotalMod` describe the in-play totals only. So the
omission is undocumented in **both** the source and the traceability map, which is
the strongest available evidence that it is an oversight rather than a reading.

**C3b — `later_life_xp_rate` is unreachable on the flat XP-pool path.**
`life_stage.rs::LifeStageRules::later_life_rate` is called from exactly one place,
`::budget`, which returns `None` immediately when
`entity.ability_funding == AbilityFunding::Pool`. A directly-entered character
using `Entity::xp_pool` therefore gets **nothing** from Wealthy or Poor. This is
consistent with the design (later life is a life-stage concept) but it means the
same two entries are computed or inert depending on a mode the player chose
elsewhere.

*Evidence.* `rg -an "later_life_rate" crates/ ui/src` — 14 hits; the only
non-test, non-doc call is `life_stage.rs:548` inside `budget`.

**C3c — `confidence_bonus` and `true_faith_grant` produce a number nothing
consumes.** Both are computed (`effective/gift_confidence.rs::confidence`,
`effective/might.rs::true_faith`) and serialized to the frontend for display, but
no derived total, validator or budget reads the result. Confidence is spent per
roll and True Faith has its own rules; neither is modelled. Not a defect — a scope
boundary — but it means "the engine computes it" is true only in the weakest
sense.

## C4 — Variants with more than one consumer that disagree

**C4a — the computing consumer folds grants; the validating consumer does not.**
This is one pattern, and it recurs:

| Variant | Computes over | Validates over | Consequence |
|---|---|---|---|
| `ability_bonus`, `affinity_ability_cost` | `selections_for_effects` (bought ++ granted) | `validate_ability_bonus_targets` iterates `entity.selections` | A **House- or Mythic-granted** Puissant/Affinity whose target the character never bought applies nothing and is never reported as dangling. |
| `characteristic_score_delta_param` | `selections_for_effects` | `validate_characteristic_delta_preconditions` iterates `entity.selections` | A granted Great/Poor Characteristic grants its free ±1 with **no** "must already be at ±3" check. |
| `incompatible_with` (Part B field) | — | `validate_incompatibilities` gets `selected_ids`, built as `entity.selections.iter().map(…)` | An incompatibility between a bought item and a **granted** one is not detected — while `Prereq::Has` in the same run *does* see grants (`PrereqCtx::present_ids`). |

*Evidence.* `validation/mod.rs::validate` was read at the dispatch point: it
builds both `selected_ids` (bought) and `effective_selections` (folded) and hands
each validator one or the other explicitly. `validate_duplicate_selections`,
`validate_total_selection_cap`, `validate_per_value_cap`,
`validate_exclusive_param_values`, `validate_share_of_kind_cap`,
`validate_possessed_param_targets`, `validate_magical_focus`,
`validate_power_targets` and `validate_might` get the **folded** list.
`validate_incompatibilities`, `validate_required_traits`,
`validate_forbidden_traits`, `validate_ability_bonus_targets` and
`validate_characteristic_delta_preconditions` do **not**.

**Three of those five are documented as deliberate, and two are not.**
`validation/prereq.rs::PrereqCtx::build` carries a comment beside the
`present_ids` fold stating that the grants-inclusive set is "deliberately distinct
from the bought-only `selected_ids` that the forbidden-trait / **incompatibility**
validators use — grants must never reach those (review finding B1)". So
`validate_incompatibilities`, `validate_required_traits` and
`validate_forbidden_traits` are a recorded decision, not an oversight, and the
`incompatible_with` row in the table above should be read as *scope fact,
deliberate* rather than as a gap.

`validate_ability_bonus_targets` and
`validate_characteristic_delta_preconditions` carry **no such note**, at their
call sites or in their own doc comments. Those two are the genuinely
undocumented divergence.

**C4b — two instance-matchers for the same idea differ in the `None` case.**
`effective/ability.rs::ability_bonus` matches a parameterized Ability's instance
with an explicit `None => false` (a selection that omits the instance key targets
**nothing**). `effective/xp.rs::ability_affinity` writes the same test as
`selection.params.get(key).map(Id::as_str) == parameter`, which is `true` when
both sides are `None`. The two therefore disagree for a query with
`parameter: None` against a selection with no instance key.

**It is reachable, not merely latent.** `validation/scores.rs::validate_ability_age_cap`
calls `ability_affinity(entity, ruleset, &entry.ability, entry.parameter.as_deref())`.
An `AbilityScore` row for a *parameterized* Ability whose `parameter` is `None` is
exactly what a direct-unchecked or hand-edited save can hold — only a validator
objects to it, and under `ValidationMode::Silent` nothing does. Pair that with a
selection that omits the instance key and the divergent branch fires: the Affinity
is found, the **+2 age-cap widening is granted**, and `ability_bonus` on the same
pair would have refused. Reachable-under-`Silent`, and it fails **permissive** —
it widens a cap rather than narrowing one.

**C4c — considered and found NOT to disagree.** `grants_spell_mastery` is read by
two functions, but they read disjoint fields (`score` vs the ratio).
`warping_grant` is folded over two different selection lists
(`warping_points_total` vs `warping_points_for_owed`), which is the documented
recursion guard, not a disagreement. `deficient_art` has exactly one fold
(`effective/art.rs::deficient_arts`), deliberately shared by the creation cap and
the in-play totals so they cannot diverge.

## C5 — Validation-only variants (read only to raise an issue)

**None, strictly.** The closest case is **`grants_reputation`** — and it is very
close. Its only two readers are
`effective/reputation_and_caps.rs::reputation_grants`, which builds a display list
and performs no arithmetic, and `validation/scores.rs::validate_reputations`,
which raises an issue. It moves **no number anywhere**. It escapes this category
only on the technicality that a display list is not a validator.

(`ability_authorization` might look like the candidate, but it is not: besides
raising `ability_category_requires_virtue` it widens
`effective/xp.rs::magus_later_life_pool`'s eligible categories, which changes the
flow solve's `max_flow` — a real number.)

Two variants are read to raise an issue *in addition to* computing:
`might_grant` (→ `might_realm_mismatch`, a warning) and `magical_focus`
(→ `multiple_magical_foci`, an error).

## C6 — The frontend `Effect` union omits seven variants

`ui/src/lib/types.ts` mirrors the Rust `Effect` enum **by hand**, and
**seven of the 42 tags are missing**:

`later_life_xp_rate` · `locality_ability_cap_fraction` · `ability_authorization` ·
`masterpiece_item` · `might_grant` · `power_levels` · `focus_points`

Plus one incomplete variant: `special_casting_mod` is declared as
`{ type: 'special_casting_mod'; kind: SpecialCasting }` — the optional `param`
field is absent.

*Evidence.* The 42 serde tags were derived from `types.rs::Effect` (variant name
→ `snake_case`) and each searched in `ui/src/lib/types.ts`. The seven above return
no hit as a `type: '…'` literal; the only textual near-misses are
`power_levels_budget`, `power_levels_used`, `focus_points_budget`,
`focus_points_used` and `invested_power_levels`, which are `EffectiveScores`
fields, not effect tags.

**Why this is not currently a runtime bug, and why it is still a gap.** Only two
places in `ui/src` read an effect's `type` at all:
`ui/src/lib/components/AbilityTab.svelte` (`ability_score_grant`) and
`ui/src/lib/derive.ts` (`warping_grant`) — both of which *are* in the union. A
missing arm in a TS discriminated union does not crash; it just means the value
is untyped where it appears. But there is **no guard**: `crates/arm-app/tests/commands.rs`
contains hand-mirror tests for `CreationPhase`, `CrisisSeverity`, `AgingEffect`
and several DTO key sets, and **none for the `Effect` union**
(`rg -n "types.ts" crates/*/tests/*.rs` — no row checks effect tags). So the drift
was invisible and will stay invisible.

## C7 — `creation_effect` is not required to carry effects

`crates/arm-rules/tests/data_integrity.rs::every_vf_is_classified` asserts that
**`in_play_effect` entries must carry at least one effect**, and that an entry
carrying effects is neither `narrative` nor `uncomputed_rule`. It does **not**
assert the converse for `creation_effect`. That asymmetry is exactly why the five
effect-less `creation_effect` entries named in `README.md`
(`flaw.corrupted_arts`, `flaw.savantism`, `virtue.devil_child`,
`virtue.nephilim`, `virtue.simple_student`) are green today. The guard cannot tell
a `creation_effect` whose mechanic is modelled by *something other than an effect*
(Devil Child's budget bonus lives on the Mythic Companion type profile, not on the
Virtue) from one that simply lost its effect.

---

# Open questions

Things this document could not settle from the code, listed rather than guessed.
A wrong confident answer here poisons 655 downstream verdicts.

1. **Is the `lab_total_mod` absence from `spell_level_cap` (C3a) a bug or a
   decision?** Narrowed, not closed. It is **not a recorded decision**: an
   adversarial re-check read `crates/arm-rules/RULES.md`'s entry for this cap and
   found it quotes the same `ArMDE:2465` sentence, justifies the Deficiency
   halving with it, and names exactly one acknowledged approximation
   (requisite-Art reduction) — nothing about `lab_total_mod`. So both the source
   and the traceability map are silent. What remains open is only whether the
   right fix is to add the term or to record the exclusion, which is a rules
   reading someone must make. **Still: do not mark an entry carrying
   `lab_total_mod` as mis-implemented on the strength of this — the defect, if it
   is one, is in `spell_level_cap`, not in the entry's data.**

2. **Is `magical_focus`'s unread `major` field a defect?** The two magnitudes of
   Magical Focus are separate catalogue entries with different point costs, and
   the *within-focus arithmetic* the book gives (add the lower Art again) is
   identical for both — so `major` may be genuinely redundant at the computation
   layer and exist only as data. I did not find a rulebook passage in this run
   that gives a Major Focus different arithmetic. *Tried:* read `types.rs::Effect`'s
   doc for the variant (cites `ArMDE:4399-4422` Major and `ArMDE:4536-4542` Minor)
   and every consumer; did not open the two passages. **A batch agent reaching
   `virtue.major_magical_focus` should read `ArMDE:4399-4422` and settle it.**

3. **Should `grants_reputation`'s `score` be enforced (C2)?** Whether a granted
   Reputation's level is a ceiling, a fixed value, or a suggestion is a rules
   question I did not resolve. `ArMDE:2512-2514` is cited at `reputation_grants`
   but was not read in this pass.

4. **Do the two realm susceptibilities have a home the engine could compute in?**
   `MagicResistanceEffect::SusceptibleFaerie` / `SusceptibleInfernal` halve
   resistance "against Faerie/Infernal power", which the per-Form Magic Resistance
   grid cannot express — but whether a realm-scoped second grid is the right
   model, or whether surfaced-only is the honest answer, is a design question, not
   a code fact.

5. **Is the bought-only scope of `validate_ability_bonus_targets` and
   `validate_characteristic_delta_preconditions` (C4a) deliberate?** Neither
   carries a note, at its call site or in its own doc comment, and neither shares
   the recorded rationale that covers the incompatibility and trait checks. A
   granted Great Characteristic really does skip the "already at ±3" gate; whether
   that is intended (grants are off-budget, so perhaps off-precondition too) or an
   oversight is not answerable from the code.

   *(An earlier draft asked the same question about `validate_incompatibilities`.
   That one is settled and the answer is "deliberate":
   `validation/prereq.rs::PrereqCtx::build` names the incompatibility validator
   explicitly as one of the checks grants "must never reach", citing review
   finding B1. Recorded here because the wrong version of this question was in
   the draft a verification pass overturned.)*

---

# Correction log

This document was written, then attacked. One verification pass was run against
Part C with the sole brief of falsifying it. It overturned or sharpened six
things, all now folded in above and listed here so a reader knows which claims
have been stress-tested and which merely asserted:

| What changed | Why |
|---|---|
| C4a's "not documented as deliberate" rider | **Was wrong** for `validate_incompatibilities` / the two trait checks — `PrereqCtx::build` documents them. Narrowed to the two validators that genuinely lack a note. |
| Open question 5 | **Deleted and replaced** — it asked about the one case that turned out to be settled. |
| C4b "latent, not reachable" | **Too generous.** Reachable through `validate_ability_age_cap` under `ValidationMode::Silent`, and it fails permissive. |
| C5's "closest case" | **Wrong example.** `ability_authorization` does move a number; `grants_reputation` is the real near-miss. |
| C1's `HealthTrack` mechanism | **Imprecise.** All five tracks are folded; three die one level later in `surfaced_modifiers`. |
| Open question 1 (RULES.md) | **Narrowed.** RULES.md records no exclusion, which makes C3a stronger, not weaker. |
| C2-e (new) | **Found by the pass, not by the draft.** `magic_resistance_mod`'s runtime-read `param` has no load-time validation — and unlike C2-d's mirror case, it is live in shipped data. |
| C1's shipped-usage counts | **Added.** `AgingEffect::Decrepitude` has 0 shipped uses, so it is dead *and* unused; every other dead value is authored by real entries. |

One thing the pass found that belongs to the **source**, not to this document, and
so is recorded only here: `types.rs::Effect`'s `AgingMod` doc comment opens
"**Three of the five kinds are consumed**". There are **eight** kinds
(`AgingEffect::ALL`), and seven are consumed. The prose undercounts both sides. It
is the comment a reader consults to decide whether an aging kind is live, so it is
worth fixing — but fixing it is a code change, outside this audit's Phase 0.

