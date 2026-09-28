# D0 — design: the four XP modes (D13, D40, D56/D3, D49)

Design note for Phase 2 group D, slice D0. No code changed by this document.
Reviewed next by the plan-reviewer, then the architect, before D1 starts.

**Mandate.** D13 (earmark), D40 (replacement), D56 (truncated apprenticeship,
landing as slice D3), D49 (free seasons) all extend the shared XP pool /
life-stage machinery in `effective/xp.rs` and `life_stage.rs`. D13's own text
says the earmark and replacement modes "should be designed together, because a
`from_normal_budget` flag and a `replaces_life_stage` flag are the same kind of
answer to the same kind of question" — this note designs all four together so
D1–D4 produce one coherent addition, not four that don't compose.

**Revision 3 note: D49/D4 is superseded by D62 and dropped (§ 5) — this is
now a three-mode, three-slice design (D1, D2, D3).** The title and this
paragraph are kept as the note's original framing rather than rewritten,
since `corrections.md`-style history is worth preserving; § 5 carries the
supersession in full.

---

## 0. What already exists (read before designing on top of it)

| Piece | Where | Shape |
|---|---|---|
| `Effect::RestrictedAbilityXp` | `types.rs:1366-1386` | `{ amount, abilities: Vec<Id>, categories: Vec<AbilityCategory>, instances: Vec<AbilityRef> }` — D48's union landed; **no** `from_normal_budget` field yet |
| `Effect::ScaledRestrictedAbilityXp` | `types.rs:1413-1427` | D35's parameter-scaled sibling, landed (C3) |
| `LifeStageBlock` | `effective/xp.rs:216-246` | `ChildhoodNativeLanguage \| ChildhoodSpread \| LaterLife` — **lives in `effective/xp.rs`, not `life_stage.rs`** (architectural issue, § 1) |
| `LifeStageBudget` | `life_stage.rs:420-470` | `childhood_native_xp, childhood_spread_xp, later_life_years, later_life_rate, later_life_xp, apprenticeship_years, apprenticeship_xp, gauntlet_age, post_gauntlet_years, post_gauntlet_points, post_gauntlet_spell_levels, post_gauntlet_xp` |
| `LifeStageRules::budget()` | `life_stage.rs:520-579` | single funnel for every mode-sensitive path; reads `Entity::ability_funding`, not the plan's presence |
| `LifeStageRules::later_life_years(stop_age, apprenticeship_years)` | `life_stage.rs:632-634` | **already generalized**: takes an arbitrary "years to carve out of later life" parameter, not hardcoded to a real magus's own block — D2 and D3 both reuse this signature, not fork it |
| `LifeStageRules::apprenticeship_of()` | `life_stage.rs:650-662` | reads `profile.hermetically_trained` **alone**, never the entity-level union (D56/A0's own documented exemption) — `None` for every companion profile, Abandoned Apprentice included |
| `general_pool_and_bonus()` | `effective/xp.rs:1479-1498` | `base_general = apprenticeship_xp + post_gauntlet_xp` if `is_hermetically_trained`, else `later_life_xp`, else `entity.xp_pool`; **the branch this note must widen for D3** |
| `two_phase_max_flow` | `effective/xp.rs:1619-1642` | restricted-only max-flow first (source→general edge closed), then general opens — this is what already makes a restricted pool "preferred" over general for its own eligible spends, with no ambiguity about which edge a spend routes through |
| `build_capacity_matrix` | `effective/xp.rs:1583-1617` | every `FlowPool` gets `cap[SOURCE][pool_node] = amount` — **an unconditional source-fed edge**, which is exactly what D13 must NOT reuse verbatim for an earmark (§ 2) |
| `restricted_ability_xp_pools()` | `effective/xp.rs:1114-1195` | one `FlowPool` per `RestrictedAbilityXp`/`ScaledRestrictedAbilityXp` grant; the `RestrictedAbilityXp` match arm (`:1131-1136`) names all four fields with **no trailing `..`** — the one compile-forced site a fifth field touches (§ 2) |
| `CODE_RESTRICTED_XP_UNSPENT` | `validation/magus.rs:938-957` | already fires as a **warning** for ANY `FlowPool` with `used < amount`, regardless of origin — D13's "must be spent" obligation is met by wiring the earmark into `flow_pools`, no new validator needed |
| `spell_levels_base`/`spell_levels_budget`/`life_stage_spell_levels` | `effective/spell.rs:195-235` | base from `profile.spell_levels` (0 for every companion profile); post-Gauntlet levels folded in via `budget.post_gauntlet_spell_levels` — the one selector both the validator and the DTO read, so D3's truncated levels must fold in **here**, not a second place |
| `ApprenticeshipRules` | `life_stage.rs:57-91` | `years: u32, xp: u32, minimum_abilities, recommended_abilities, recommended_xp, default_gauntlet_age` — ruleset data, `RULES.md`-documented |
| `ParamType::Number{min,max}` | C0/C3, landed | the numeric parameter type D3's "years of apprenticeship completed" needs (Revision 3, D62) |
| `ParamGate`, `AbilityRef` | C0/C1, landed | D13's earmark reuses `instances: Vec<AbilityRef>` for Latin/Organization Lore: Church exactly as D48's pools already do |
| `is_hermetically_trained`/`entity_confers_hermetic_training` | `effective/hermetic_training.rs`, A0/A1 landed | the union D56 built; `flaw.abandoned_apprentice` does **not yet carry** `Effect::ConfersHermeticTraining` — attaching it is D3's own first act (A0 § 5 hand-off note) |
| `Effect::ConfersHermeticTraining` | landed (A1) | bare marker, already threaded through every exhaustive `match Effect` site (A0 § 1a) |
| `flaw.church_upbringing` | `virtues_flaws.json:347-354` | `narrative`, no `effects`, no `parameters` — D13's carrier, untouched |
| `flaw.feral_upbringing` | `virtues_flaws.json:1071-1079` | `creation_effect`, ships `restricted_ability_xp: 120` **additively** — F-428's bug, D40's carrier |
| `virtue.redcap` | `virtues_flaws.json:5786-5794` | ships only `item_level_budget: 50`, **no XP effect at all** — under-funded (D17's "second defect") |
| `virtue.lone_redcap` | `virtues_flaws.json:5017-5029` | ships `restricted_ability_xp: 300` **additively**, broad categories — F-439's bug |
| `flaw.abandoned_apprentice` | `virtues_flaws.json:12-19` | `narrative`, `prerequisites: Has(virtue.the_gift)` (landed, D56), no `parameters`, no `effects` — D3's carrier |
| `virtue.landed_noble`, `virtue.license_of_absence` | `virtues_flaws.json:4852-4859,4959-4966` | both `narrative`, no effects — D49's carriers |
| `validate_life_stage_plan` | `validation/life_stage.rs:66-117` | gates every post-Gauntlet check on `magus = type_profile.hermetically_trained` (**profile-only**, D56/A0's documented exemption) — D3 must NOT read the union here either |
| `abandoned_apprentice_xp_shape_is_unchanged_pending_d3` | `data_integrity.rs:5817` | **the pinned baseline D3 flips**: asserts `allocation.general_pool == 225` (15yr later-life, untrained shape) for a 20-year-old companion holding the Flaw — must change once `ConfersHermeticTraining` lands |

---

## 1. Architectural correction required before D1: `LifeStageBlock` must move

`Effect::ReplacesLifeStageXp` (§ 3) has to **name** a life-stage block as data
— it is the whole point of D40's "one effect that names a life stage." The
only existing taxonomy for "which block" is `LifeStageBlock`, and it lives in
`crate::effective::xp` (`effective/xp.rs:216`). `Effect` lives in `types.rs`,
and `types.rs` already depends on `life_stage` (`use crate::life_stage::
LifeStagePlan;`, `types.rs:17`) — the dependency runs `types → life_stage`,
never `types → effective` (confirmed: `effective/xp.rs` opens with `use
super::*`, i.e. it depends on `types`, the reverse direction). `types.rs`
reaching into `effective::xp` for `LifeStageBlock` would be a real layering
violation — the kind CLAUDE.md's engine-purity/module-boundary intent forbids
— not a style nitpick.

**Fix, required groundwork for D1 (not its own slice): relocate the
`LifeStageBlock` enum's definition from `effective/xp.rs` to `life_stage.rs`.**
Pure code motion, no behavior change — mechanical, like U0's `EffectiveScores`
DTO move (Phase 1). **Revision 2 correction (plan-review finding D0-6): the
footprint is three files with a real edit, not six.** Verified by reading
every reference, not by counting the `grep -rln` hit list: `effective.rs`'s
own `pub use xp::*;` (`effective.rs:43`) is what currently re-exports
`LifeStageBlock` out of the `xp` submodule, and its handful of uses
(`effective.rs:3476,3776-3796`) sit inside `#[cfg(test)]`, reached via that
same glob — so moving the definition needs exactly:

1. `life_stage.rs` — gains the enum + its `impl`s (the move itself).
2. `effective/xp.rs` — loses them; its own remaining uses resolve unchanged
   through `use super::*` (`:19`), which already glob-imports whatever
   `effective.rs` re-exports, once (2) below exists.
3. `effective.rs` — gains one explicit `pub use crate::life_stage::
   LifeStageBlock;`, replacing what the `xp::*` glob used to carry.

**Zero further edits**, not "a one-line edit each": `lib.rs:80`'s crate-root
`use crate::effective::{..., LifeStageBlock, ...}` keeps resolving once (3)
exists, since the public path `effective::LifeStageBlock` is unchanged;
`export.rs:3392`'s `crate::effective::LifeStageBlock::ALL` (a fully-qualified
path) resolves for the identical reason; `validation/magus.rs:963` and
`validation/life_stage.rs:1205` name the type only inside a rustdoc
intra-doc link (`` [`LifeStageBlock`] ``, no `use` statement anywhere in
either file — confirmed, `grep -rn "use.*LifeStageBlock"
crates/arm-rules/src` finds nothing), which rustdoc resolves against the
type's public path regardless of which module defines it, so neither file
needs touching at all.

This also gives `LifeStageBlock` the conceptually correct home: it is a
life-stage taxonomy, not an XP-flow-graph implementation detail, and D2/D3
both need to extend it (§ 3, § 4) from code that must not create a new
`types → effective` edge either.

---

## 2. D13 — the earmark, and why it is NOT a new source-fed pool

**Finding.** `flaw.church_upbringing`, ArMDE:5789-5791 (verified directly):

> "The player must spend 25 experience points from the normal budget on Artes
> Liberales, Latin, Music, Organization Lore: Church, or Theology. Unless the
> character has a Virtue that permits it, no other experience points may be
> spent on Academic Abilities."

Ruling: D13 (`decisions.md:407-482`). Add `from_normal_budget: bool` to
`RestrictedAbilityXp`; model both clauses, budget unchanged.

### Why a naive `FlowPool` is wrong

`build_capacity_matrix` gives every `FlowPool` an **unconditional** edge
`cap[SOURCE][pool_node] = amount` (`effective/xp.rs:1604`) — that is
*additional* supply, on top of `general_pool`. Wiring the earmark this way
would over-fund the character by 25, which is exactly the defect D13's
obligation #2 forbids ("the total budget must not rise").

D13's own prose floats "the flow graph gains one edge shape:
`general → earmark → eligible spends`, rather than `source → pool → eligible
spends`." Read literally (`cap[GENERAL][pool_node] = amount`, closing
`cap[SOURCE][pool_node]`), this **does not work** under
`two_phase_max_flow`: phase 1 runs with `cap[SOURCE][GENERAL]` closed
(`effective/xp.rs:1638`), so a pool fed only from `GENERAL` has zero supply in
phase 1 and cannot be preferentially filled — it would compete with every
other general-eligible spend on equal footing in phase 2, and because the
earmarked Abilities are *also* reachable via the ordinary `GENERAL → spend`
edge, which path the max-flow solve picks is an implementation accident of
augmenting-path order, not a rule. `RestrictedXpPool.used` would then report
an arbitrary, non-reproducible split between "funded via the earmark" and
"funded via general" for the same spend — a real regression against the XP
bar's existing (and tested) determinism.

### The shape actually chosen

Reduce `general_pool`'s own base by the earmarked total, and fund the earmark
as an **ordinary, `SOURCE`-fed** restricted pool of that same size — i.e. shift
capacity from `general` to a new pool node rather than opening a new edge
shape:

```
base_general' = base_general - Σ(earmark.amount for every from_normal_budget=true grant)
flow_pools    += one ordinary FlowPool per earmark (SOURCE-fed, same as Educated/Warrior)
```

Total supply (`base_general' + Σ earmarks + other pools`) is algebraically
identical to today's `base_general + other pools` — **capacity does not
move.** The earmark pool now goes through `two_phase_max_flow`'s **existing**
phase-1 preference (restricted-only, before general opens), which is the
*proven* mechanism that already makes Educated/Warrior/Privileged report a
determinate `used` figure — reusing it rather than inventing a second one is
the point.

**Revision 2 correction (plan-review finding D0-4): "capacity does not move"
is not "spendable total does not move," and the note's first draft conflated
the two.** If an earmark's `amount` exceeds what its eligible Abilities can
realistically absorb (a score cap, or a future earmark naming a narrower set
than its point total), `base_general` still shrinks by the **full** `amount`
— the subtraction in § "Engine change" step 4 is unconditional, it does not
know or care whether the earmark pool can actually spend that much — while
the earmark pool itself reports `used < amount`. The shortfall is **not**
returned to `general`: the character ends up with a strictly smaller
*spendable* total than before the earmark existed, even though total
*capacity* (the sum the flow graph can route) is unchanged. `flaw.church_
upbringing`'s own five Abilities can absorb far more than 25 XP between them,
so this is inert for the one shipped carrier D1 authors — but `from_normal_
budget` is a general-purpose flag, so a future earmark with a tighter set
could hit it for real.

**D13 does not rule on this edge case — it is an open question, not a
silent design gap.** D13's text (`decisions.md:407-482`) states the earmark
"must be spent" (obligation #3) and that "the total budget must not rise"
(obligation #2, a ceiling), but never contemplates an earmark that *cannot*
be spent in full. Two readings, neither decided here:

- **(a) Accept the shortfall, surfaced only via `CODE_RESTRICTED_XP_UNSPENT`**
  — the same warning every other under-spent restricted pool already gets
  (Educated naming an Ability the player never buys behaves identically
  today). No new mechanism; the character is simply told, not silently
  short-changed.
- **(b) Refund an unabsorbable remainder back to `general`** — would need a
  second pass after the flow solve (how much of the earmark pool's *capacity*
  proved unreachable given the character's actual Ability scores/caps, not
  just "unspent because the player hasn't gotten there yet" — the two are
  observationally identical to the solve) and reopens exactly the
  over-funding risk D13 rules out, since a partially-refunded earmark plus a
  later purchase raising the score cap would then need to claw the refund
  back.

**Recommendation: (a).** It costs nothing beyond what D1 already builds, is
visible rather than silent (meeting D16/D50's "an absolute the engine will
not model is text, a *soft* restriction is a warning" standard), and (b)'s
"how much was truly unabsorbable" question has no clean answer without
re-running the solve under a hypothetical. Flagged in § 6 as open question
6.3 for Norbert to confirm or override before D1 ships.

**Authorization comes free, unchanged from D13's own reasoning**
(`decisions.md:435-441`): `ability_authorizations()`'s existing
`RestrictedAbilityXp` arm (`effective/xp.rs:688-713`, already `..`-tolerant —
see below) folds in the earmark's `abilities`/`instances` regardless of
`from_normal_budget`, so the five named Abilities are authorized and every
*other* Academic Ability stays refused by `categories_requiring_virtue`. That
is clause 2. No new authorization code.

### Type

```rust
RestrictedAbilityXp {
    amount: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    abilities: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    categories: Vec<AbilityCategory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    instances: Vec<AbilityRef>,
    /// D13: this grant is not additional supply — it earmarks part of the
    /// GENERAL pool the character already has. `general_pool_and_bonus`
    /// subtracts every earmark's `amount` from the general base before this
    /// pool is built as an ordinary source-fed `FlowPool`, so the character's
    /// total budget is unchanged; only which Abilities the earmarked slice may
    /// fund narrows. `false` (the default) preserves every existing carrier's
    /// current, additive meaning — Educated, Warrior, Privileged Upbringing
    /// are real grants, not earmarks, and must not lose XP by this change.
    #[serde(default, skip_serializing_if = "crate::types::is_false")]
    from_normal_budget: bool,
}
```

JSON — `flaw.church_upbringing` (Latin and Organization Lore: Church are
instance-scoped per D14/D48's existing mechanism; Artes Liberales, Music,
Theology are bare ids):

```json
{
  "id": "flaw.church_upbringing",
  "classification": "creation_effect",
  "effects": [{
    "type": "restricted_ability_xp",
    "amount": 25,
    "from_normal_budget": true,
    "abilities": ["ability.artes_liberales", "ability.music", "ability.theology"],
    "instances": [
      { "ability": "ability.dead_language", "instance": { "literal": "latin" } },
      { "ability": "ability.organization_lore", "instance": { "literal": "church" } }
    ]
  }]
}
```

(Verify `"latin"`/`"church"` against CV's `rules/i18n/<lang>/` catalogues at
implementation — D1 does not invent new catalogue ids, it reuses whichever the
CV pass already minted for Simple Student/Covenant Upbringing, or adds
`"church"` to the `organization` catalogue if D59/CV has not yet.)

### Engine change

1. `types.rs`: add the field (above).
2. `effective/xp.rs::restricted_ability_xp_pools` (`:1131-1136`): the
   `RestrictedAbilityXp { amount, abilities, categories, instances }` arm names
   all four fields with no `..` — **compile-forced** the moment the field
   exists. Add `from_normal_budget: _,` (unread here; the flag only matters to
   the general-pool computation below).
3. New function `earmarked_general_xp(entity, ruleset) -> u32`: same
   `selections_for_effects` walk `restricted_ability_xp_pools` already does,
   filtered to `Effect::RestrictedAbilityXp { amount, from_normal_budget: true,
   .. }`, summing `amount`. A short, independent walk, on the file's own
   established precedent ("both are cheap, pure lookups... recomputing costs
   nothing", `effective/xp.rs:1475-1478`) rather than threading a value out of
   `restricted_ability_xp_pools`.
4. `general_pool_and_bonus` (`:1479-1498`): `base_general` gains
   `.saturating_sub(earmarked_general_xp(entity, ruleset))` before
   `general_bonus` is applied, in **every** branch (life-stage or `xp_pool`) —
   an earmark narrows the same budget regardless of funding mode.
5. `ability_authorizations`'s `RestrictedAbilityXp` arm (`:688-693`) already
   ends `..` — **no change forced**, confirmed by direct read.

### Validation

`CODE_RESTRICTED_XP_UNSPENT` already fires (warning) for any `FlowPool` with
`used < amount` — the earmark is an ordinary `FlowPool`, so D13's "the earmark
*must* be spent" obligation is met with **zero new validator code**, exactly
as `decisions.md:452-455` argues. Severity stays a warning (D13 does not ask
for an error, and the general shape of every other unspent-restricted-pool
finding is a warning).

### Integrity

No new check: `from_normal_budget` is a bare `bool`, no invalid state. The
earmark's `abilities`/`instances` get the existing `validate_ability_list_effect`/
`validate_gated_ability_refs` checks `RestrictedAbilityXp` already receives
(`ruleset/integrity.rs:2441-2460`), unaffected by the new field (its arm
already ends `..`).

### Saves

No `SCHEMA_VERSION` bump. `from_normal_budget` lives on `Effect`, in
`rules/core/virtues_flaws.json` — ruleset data, re-authored wholesale, not a
saved `Selection` field. A saved `Selection` naming `flaw.church_upbringing`
round-trips byte-identically; only how the engine funds it changes.

---

## 3. D40 — replacement, and why childhood and apprenticeship need different arithmetic under one variant

**Findings.**

| Carrier | Passage (verified) | Bug |
|---|---|---|
| `flaw.feral_upbringing` | ArMDE:6110-6113: *"You may only choose beginning Abilities that you could have learned in the wilds. In particular, you may not start with a score in a Language. In your first five years you gain 120 experience points, which must be split between (Area) Lore, Animal Handling, Athletics, Awareness, Brawl, Hunt, Stealth, Survival, and Swim."* | Ships `restricted_ability_xp: 120` **additively** on top of the standard 120-point childhood block → 240 (F-428) |
| `virtue.redcap` | ArMDE:4842-4851 (verified; `:4848` — *"You have spent fifteen years as an apprentice, and gained a total of 300 experience points in those fifteen years"*) | Ships **no** XP effect at all — under-funded by a whole block |
| `virtue.lone_redcap` | ArMDE:4319-4326 (verified; `:4321` — *"You still begin with 300 experience points for your fifteen years spent as an apprentice"*) | Ships `restricted_ability_xp: 300` **additively** on top of ordinary later-life for the same 15 years → over-funded by ~225 (F-439, resolved D17) |

Ruling: D40 (`decisions.md:1801-1839`), reading D17 (`decisions.md:130-198`).
One effect, naming a life stage. Two carriers, two different shapes.

### The two shapes are structurally different — this is why one variant, not one arithmetic formula

**Childhood (Feral).** Childhood is a **flat, unscaled block**: 120 XP over a
fixed 5-year span the character lives regardless. The replacement swaps
*eligibility* (9 named Abilities, no native-language carve-out — "you may not
start with a score in a Language" removes the native/spread split entirely)
for the *same total* (120). No year-count is carved out of anything else;
childhood's 5-year span is unaffected.

**Apprenticeship-shaped (Redcap/Lone Redcap).** This is a **years-carving**
replacement: it is the identical structural move a real magus's own
apprenticeship already makes on later life
(`later_life_years(stop_age, apprenticeship_years)`, `life_stage.rs:632`,
already generalized to accept an arbitrary `apprenticeship_years`) — 15 years
are removed from the later-life span, and a fixed 300-XP restricted block
replaces what those 15 years would otherwise have earned as ordinary later
life. **Reuses the existing function signature, does not fork it.**

One `Effect` variant covers both because both are "this selection replaces a
named block's normal grant" — the *consumer* differences are exactly the kind
`LifeStageBlock`'s three existing variants already have (`childhood_native_
language_pool`/`childhood_spread_pool`/`magus_later_life_pool` are three
separate functions sharing one enum) — not evidence the concept needs two
variants.

### Type

```rust
/// Replaces a named life-stage block's normal grant with a different total
/// and eligibility, rather than adding to it (D40). Consumed differently per
/// `stage` — see `LifeStageRules::budget`/`build_flow_pools` — exactly as
/// `LifeStageBlock`'s three pre-existing variants already are.
ReplacesLifeStageXp {
    /// Which block this replaces.
    stage: LifeStageBlock,
    /// The replacement's total XP.
    amount: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    abilities: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    categories: Vec<AbilityCategory>,
    /// Years carved out of later life in place of this block — meaningful
    /// ONLY for `LifeStageBlock::Apprenticeship` (below); `0` (skipped) for a
    /// flat block replacement (childhood), which does not touch any span.
    /// Authored per entry rather than derived from the ruleset's own magus
    /// `ApprenticeshipRules.years`, because the carrier's OWN passage states
    /// its span independently (ArMDE:4321: "your fifteen years") and a
    /// companion-shaped ruleset need not ship a magus block at all for this
    /// effect to be well-formed.
    #[serde(default, skip_serializing_if = "crate::types::is_zero")]
    years: u32,
}
```

`LifeStageBlock` gains one variant (relocated per § 1 first):

```rust
pub enum LifeStageBlock {
    ChildhoodNativeLanguage,
    ChildhoodSpread,
    LaterLife,
    /// A replacement-effect-driven, apprenticeship-shaped block for a
    /// character who is NOT hermetically trained by profile (Redcap, Lone
    /// Redcap) — distinct from a real magus's own apprenticeship, which
    /// needs no slug of its own (it folds straight into the general pool;
    /// see `LifeStageBudget::apprenticeship_xp`'s own doc comment).
    Apprenticeship,
}
```

**Childhood's replacement reuses the EXISTING `ChildhoodSpread` tag, not a new
one.** `XpBar.svelte` already has a "spread-only" display branch
(`xp-pool-block-early-childhood-spread-only`, `XpBar.svelte:207-210`), used
today whenever `childhoodNative` is absent. A Feral character's replacement
pool, tagged `origin: LifeStage{ block: ChildhoodSpread }`, produces exactly
that state (no native-language pool is built at all — see below) — **the
existing UI renders it correctly with zero new code.** Do not invent a fourth
tag for this case.

JSON:

```json
// flaw.feral_upbringing
{
  "effects": [{
    "type": "replaces_life_stage_xp",
    "stage": "childhood_spread",
    "amount": 120,
    "abilities": ["ability.animal_handling", "ability.area_lore", "ability.athletics",
                  "ability.awareness", "ability.brawl", "ability.hunt", "ability.stealth",
                  "ability.survival", "ability.swim"]
  }]
}

// virtue.redcap — 15 years, 300 XP, ArMDE:4848's broad categories
{
  "effects": [{
    "type": "replaces_life_stage_xp",
    "stage": "apprenticeship",
    "amount": 300,
    "years": 15,
    "categories": ["academic", "arcane", "general", "martial", "supernatural"]
  }]
}

// virtue.lone_redcap — identical shape, same passage (ArMDE:4321 "still")
{
  "effects": [
    { "type": "replaces_life_stage_xp", "stage": "apprenticeship", "amount": 300, "years": 15,
      "categories": ["academic", "arcane", "general", "martial", "supernatural"] },
    { "type": "grants_reputation", "kind": "hermetic", "score": 2 },
    { "type": "grants_selection", "items": ["virtue.well_traveled"] }
  ]
}
```

### Engine change

**Childhood.** `build_flow_pools` (`effective/xp.rs:1327-1348`): if any
effective selection carries `ReplacesLifeStageXp{stage: ChildhoodSpread, ..}`,
skip `childhood_native_language_pool` and `childhood_spread_pool` entirely and
push **one** `FlowPool` built from the replacement's own `amount`/`abilities`/
`categories`, `origin: LifeStage{block: ChildhoodSpread}`. `LifeStageBudget`'s
`childhood_native_xp`/`childhood_spread_xp` fields are set to `0` in this case
(so `LifeStageBudget::total()` does not double-count — the replacement's total
is reported only through the pool, exactly like every other restricted grant).

**Apprenticeship-shaped.** New helper,
`fn extra_apprenticeship_years(entity, ruleset) -> u32`, returning whichever
of (a) a real magus's `apprenticeship.years` or (b) a `ReplacesLifeStageXp{
stage: Apprenticeship, years, ..}` effect's `years` applies — mutually
exclusive in practice (nothing in the catalogue lets a character be both), so
`.max()` rather than `.saturating_add()` is the defensive choice. D3 (§4)
extends this SAME function with a third candidate — see § 4's own engine
change and § 7's corrected sequencing (Revision 3, F4).

**Revision 3 correction (architect finding F2) — this widened value feeds
`later_life_years` ONLY, never the `LifeStageBudget.apprenticeship_years`
struct field.** The first draft's instruction — "`budget()`'s existing
`apprenticeship_years` local becomes `apprenticeship.map_or(0, |a|
a.years).max(extra_apprenticeship_years(...))`" — read as a wholesale
redefinition of the ONE local that ALSO gets shorthand-assigned straight into
the struct field at `life_stage.rs:569`. That field's own doc comment states
"15 for a magus, 0 for anyone else" (`:432-433`), is serialized to the
frontend (`ui/src/lib/types.ts:568`), and is asserted on directly by four
existing tests — a Redcap or Abandoned-Apprentice-shaped character would
silently start reporting `apprenticeship_years: 15` despite serving no
apprenticeship in the sense that doc comment states, alongside
`apprenticeship_xp: 0` (a DTO pair that never occurred before and nothing
reads defensively), and D3's own purpose-built `truncated_training_years`
field would then duplicate the same information, correctly scoped —
"two mechanisms where one idiom exists," by accident.

**Fix: two separate locals, not one repurposed.** `budget()` keeps a local
sourced ONLY from the real magus block —
`let magus_apprenticeship_years = apprenticeship.map_or(0, |a| a.years);` —
which is what the struct field (`apprenticeship_years: magus_apprenticeship_
years`) and `apprenticeship_xp` both read, unchanged from today. A SEPARATE,
struct-field-blind local —
`let later_life_carve_years = magus_apprenticeship_years.max(extra_
apprenticeship_years(entity, ruleset));` — is the ONLY value passed to
`later_life_years(gauntlet_age, later_life_carve_years)`. A Redcap's
later-life span still correctly shrinks by 15 years, with **no change to
`later_life_years`'s own signature**, and `apprenticeship_years` keeps
meaning exactly what its doc comment says for every character, D2/D3's new
carriers included.

`build_flow_pools` pushes an ordinary `FlowPool` for the `amount`/eligibility,
`origin: LifeStage{block: Apprenticeship}` — sourced from `SOURCE`, never
`general` (Redcap's 300 does not fund Arts and is not part of the general
pool; he has none to spend on).

### Validation

No new validator. `CODE_RESTRICTED_XP_UNSPENT` covers an unspent replacement
pool exactly as it covers any other. `flaw.feral_upbringing`'s authorization
half (no Language score at all, only the 9 named Abilities as beginning
picks) is **explicitly not this effect's job** (D40: "not covered by it, and
still owed... authorization rules, must not be smuggled into this effect") —
it is B1's `Effect::RestrictsAbilityCategoryToAbilities` (already designed,
`design-b0-ranging-and-predicates.md` § 1, B1), landed independently. D2's
brief must add the second effect to the same entry, not invent a third.

### Integrity

| Field | Rejected when |
|---|---|
| `ReplacesLifeStageXp.years` | non-zero when `stage` is `ChildhoodNativeLanguage`/`ChildhoodSpread`/`LaterLife` (only `Apprenticeship` carves years) |
| `ReplacesLifeStageXp.years` | zero when `stage == Apprenticeship` (a years-carving replacement with nothing to carve is dead data) |
| `ReplacesLifeStageXp.abilities`/`categories` | same checks `RestrictedAbilityXp` already gets (ability ids resolve; categories are the closed enum, serde-checked) |

### Saves

No bump. `ReplacesLifeStageXp` lives on `Effect`, ruleset data. No
`Entity`/`Selection` field's shape or meaning moves.

### Exhaustive-match sites this variant touches

One new `Effect` variant → the same 16-site checklist B0/C0 established
(`design-c0-parameter-model.md` § 1a, `design-b0-ranging-and-predicates.md`
§ 3b), walked here rather than re-derived:

| # | Site | Verdict |
|---|---|---|
| 1 | `effective.rs` macro tail (`irrelevant_effect_variants!`) | no-op — add to the tail (same family as `RestrictedAbilityXp`) |
| 2-7 | `ability_bonus`, `art_bonus`, `deficient_arts`, `characteristic_score_bonus`, `ability_affinity`, `art_affinity` | no-op (via macro) |
| 8 | `effective/spell.rs::spell_levels_bonus` | no-op (hand tail) — a life-stage XP replacement is not a spell-levels contribution |
| 9 | `effective/spell.rs::general_xp_bonus` | no-op — folded in `effective/xp.rs`, not here (same reasoning `LaterLifeXpRate` already documents) |
| 10 | `effective/spell.rs::spell_mastery_advancement_affinity` | no-op |
| 11 | `derived.rs::in_play_mods` | no-op — creation-time grant, not an in-play total |
| 12 | `effective/xp.rs::ability_authorizations` | **real**: a replacement is itself permission for what it funds, same reasoning as `RestrictedAbilityXp` — new arm folding `abilities`/`categories` into the authorized set |
| 13 | `ruleset/integrity.rs::validate_effect_refs` | **real**: the two checks in the table above |
| 14 | `validation/mod.rs::effect_target` | `Other` — matches `RestrictedAbilityXp`'s own classification (a pool grant, no dangling-target concern) |
| 15 | `ruleset/integrity.rs::validate_item_ratios` | no-op (not ratio-shaped; trailing wildcard absorbs it) |
| 16 | `ui/src/lib/types.ts::Effect` | **Revision 3 correction (architect finding F5): omit, not "real."** The first draft marked this "real" with a justification ("no UI consumer reads it... the XP bar reads the resolved `RestrictedXpPool`") that is verbatim the reasoning D3's and D49's own row 16 use to conclude "omit" — stating the premise and reaching the opposite conclusion, with no actual consumer named. No component reads `ReplacesLifeStageXp` directly; **omit from the TS union**, matching D3's row 16 and B0's own "curated subset" rule |

---

## 4. D3 — the truncated apprenticeship (D56), landing as slice D3

**Finding.** `flaw.abandoned_apprentice`, ArMDE:5641-5650 (verified). Already
extensively designed by D56/A0; this note pins the parts A0 explicitly left
open ("D3 owns wiring it to the truncation math").

**16 XP / 8 spell levels per year, FIXED DECISION (`decisions.md:1060-1066`),
derived from ArMDE:2435's 240/120 over 15 years — not to be reopened.**

**Revision 3 rewrite (Norbert's D62 ruling, `decisions.md:844-847`; two
inputs applied together with the architect's F1/F2/F3/F4 fixes, since D62's
own change to the parameter reshapes exactly the code F1-F4 touch).** D62:
*"apprenticeship defaults to ages 10-25 (Gauntlet 25, ArMDE:1601). An
Abandoned Apprentice records the years of apprenticeship completed, not an
age, and the years after abandonment follow the later-life rules."* This
**closes former open question § 6.1** outright (no longer open — dropped, not
merely answered "(a)"), and it **simplifies the whole design**: the parameter
is now the truncation year-count itself, not an age needing a subtraction.

### Where the two rate numbers live: data, derived once, not re-derived at runtime

D56 is explicit the ratio is fixed. Two candidate homes were considered:

1. **Derive at runtime** from `ApprenticeshipRules.xp / years` (240/15=16) and
   the magus profile's `spell_levels / apprenticeship.years` (120/15=8).
   **Rejected**: the second half needs the *magus* profile's `spell_levels`
   specifically, but an Abandoned Apprentice is a `companion`-profiled entity
   — reading a **different** type profile's field by convention (which one is
   "the" magus profile?) is a fragile cross-profile lookup with no existing
   precedent, and breaks for a companion-only ruleset with no magus profile at
   all (which could otherwise still define a `flaw.abandoned_apprentice`-alike
   Story Flaw, however unlikely in practice).
2. **New explicit fields on `ApprenticeshipRules`** —
   `truncated_xp_per_year: u32`, `truncated_spell_levels_per_year: u32` (16,
   8) — **chosen**. Ordinary ruleset data, `RULES.md`-documented as "derived
   from `xp`/`years` and the magus profile's `spell_levels`/`years`
   respectively — FIXED per `decisions.md` D56, not to be reopened as a house
   rule." Required fields (not `#[serde(default)]`): every ruleset that ships
   an `apprenticeship` block must now also state these two, on the same
   footing as `xp`/`years` themselves. `SHIPPED_WITH_APPRENTICESHIP` (the test
   fixture, `life_stage.rs:735-766`) and `rules/core/life_stages.json` both
   gain the two keys in the same commit — a ruleset-JSON edit, not a schema
   change.

### The life-stage spans, and which is which (Revision 3, simplified per D62)

D62's parameter is the truncation itself — the years-before-apprenticeship
span (5-10 by default, per the ruleset's own `default_gauntlet_age(25) −
apprenticeship.years(15) = 10`) and the years-after-abandonment span are
**mechanically identical** (both are ordinary later life, same rate, same
Abilities-only eligibility), so they need no separate modelling at all: they
are simply the total years NOT spent in truncated training, all folded into
one ordinary `later_life_years` computation.

| Span | Years | Funding |
|---|---|---|
| Childhood | 5 (unchanged) | 120 XP, restricted (unchanged — "create the character as a **regular apprentice**" invokes the standard template) |
| Ordinary later life (before AND after the truncated block, merged into one bucket) | `(entity.age − childhood.years) − years_completed` | ordinary later-life rate (Wealthy/Poor-adjusted), **general** for a companion — settled by D62, § 6.1 dropped |
| Truncated training | `years_completed` (the parameter's own value — no subtraction needed) | `16 × years_completed` XP, `8 × years_completed` spell levels | **general** — ArMDE:2435's "Arts or Abilities" set is the same passage this Flaw's own construction rule invokes |

### Parameter (Revision 3: `years_completed`, replacing `age_abandoned`)

```json
{ "key": "years_completed", "type": { "number": { "min": 1, "max": 14 } }, "domain": "number" }
```

`min: 1` (must have completed at least one year to have learned anything);
`max: 14` = `apprenticeship.years(15) − 1` (completing all 15 is not
"abandoned," it is a finished apprenticeship — D62's own framing, "not an
age"). **Revision 3: this bound-consistency check is now MANDATORY, not the
"nice-to-have" the first draft proposed for the retired `age_abandoned`
bound** — Norbert's own words, "taken from the ruleset rather than
hardcoded," read as a firm constraint. `ruleset/integrity.rs` gains a load
check that this parameter's authored `max` equals `apprenticeship.years − 1`
exactly, rejecting the ruleset otherwise — the same "recompute rather than
trust a transcribed number" discipline `ApprenticeshipRules.recommended_xp`'s
own doc comment already states for a sibling field ("Carried as data so the
load can re-price the list against it — the trust gate on transcribed
numbers").

### Type

A new `Effect` variant is required — reusing
`Effect::ScaledRestrictedAbilityXp`'s "per-unit × parameter value" shape
(C0/C3/D35) is tempting but **wrong** here: that variant funds a *restricted*
pool, and the truncated block is **general** (Arts+Abilities). Distinct from
`ReplacesLifeStageXp` (D40), which replaces a RESTRICTED block's total, this
one funds the GENERAL pool and additionally carves years out of later life —
no counterpart in D40's shape:

```rust
TruncatedApprenticeshipXp {
    /// Parameter key (Number domain) naming the years of apprenticeship
    /// completed before abandonment (D62).
    param: String,
}
```

(`xp_per_year`/`spell_levels_per_year` are **not** fields here — ruleset data
on `ApprenticeshipRules`, per above.)

### F1 (BLOCKER, architect) — `ConfersHermeticTraining` must not resolve independently of `years_completed`

**The bug, restated precisely.** `entity_confers_hermetic_training`
(`effective/hermetic_training.rs:28-41`) folds in `ConfersHermeticTraining`
from an item's `effects` list alone, blind to whether any of that SAME item's
*parameters* are filled in. The instant a player selects `flaw.abandoned_
apprentice` — any input mode, guided included — `is_hermetically_trained`
flips `true` before `years_completed` is answered. At that moment:
`apprenticeship_of()` still returns `None` (companion profile, correct);
`apprenticeship_xp == 0`; `post_gauntlet_xp == 0`; and `budget.truncated_
training_xp == 0` (no value to read yet). `general_pool_and_bonus`'s trained
branch computes `0+0+0 = 0` — replacing the ordinary companion later-life
total (225 in the pinned baseline's own 20-year-old example) with **nothing**,
silently, in every input mode, including `Silent` validation (where
`missing_param` is never even surfaced). Not a crafted-file edge case — the
sheet's normal, momentary state between "pick the Flaw" and "answer its one
question."

**Fix, chosen (Revision 4, architect finding R3-1): a sibling variant, not an
in-place widening.** Revision 3's first draft widened the existing bare
`ConfersHermeticTraining` marker with an optional `requires_param` field —
correct logic, wrong idiom. The note's own § 4 already cites this codebase's
established "`Foo`/`FooParam`-style precedent... for 'same concept, different
shape'" (`AbilityScoreGrant`/`AbilityScoreGrantParam`,
`AbilityBonus`/`AbilityBonusGated`) as the reason a narrow, purpose-built
mechanism beats stretching an existing one — and then didn't apply that same
precedent to the variant it was actually changing. Widening the bare marker
forces `{ .. }` onto **seven** production sites that have nothing to do with
D3 and will never read the new field — a permanent, unread tax on working
code for a one-carrier feature. A sibling variant pays only the ordinary
per-variant cost every other D0 effect already pays (one arm added to each
exhaustive match), and leaves the bare `ConfersHermeticTraining`'s existing
seven sites **completely untouched**:

```rust
/// The conditional sibling of the bare `ConfersHermeticTraining` marker
/// (same "Foo"/"FooParam"-style precedent as `AbilityScoreGrant`/
/// `AbilityScoreGrantParam`): confers Hermetic training only once the OWNING
/// selection's own `params[param]` resolves to a value (any value — a
/// PRESENCE test, not `ParamGate`'s equality test, so the two stay
/// deliberately separate mechanisms; B4's `ParamGate{param, equals: Id}`
/// cannot express "any resolved value" without widening `equals` to
/// `Option<Id>`, which would ripple through every EXISTING `ParamGate`
/// consumer — `AbilityRef::Scoped`, `CategoryRef::Scoped`,
/// `AbilityBonusGated`, B4's own two gated variants — for a semantics none
/// of them need). D3's own carrier is the first (and, today, only) user:
/// the training marker and `TruncatedApprenticeshipXp`'s own payoff must
/// resolve TOGETHER, or an Abandoned Apprentice reads as
/// trained-but-funded-with-nothing the instant the Flaw is picked (F1,
/// `tmp/d0-architect-review.md`).
ConfersHermeticTrainingIf {
    /// Parameter key on the SAME item; presence (not any particular value)
    /// activates this marker.
    param: String,
}
```

`entity_confers_hermetic_training` gains a **second** match arm alongside its
existing bare-unit one, rather than widening that arm's payload:

```rust
item.effects.iter().any(|effect| match effect {
    Effect::ConfersHermeticTraining => true,
    Effect::ConfersHermeticTrainingIf { param } => selection.params.get(param).is_some(),
    _ => false,
})
```

`flaw.abandoned_apprentice`'s own entry: `{ "type": "confers_hermetic_training_if", "param": "years_completed" }`.

**Consequence, and why it is the correct one.** Before `years_completed` is
answered: this arm returns `false` for this item →
`is_hermetically_trained` stays `false` (profile is also `false`) →
`general_pool_and_bonus` takes the ORDINARY companion branch
(`later_life_xp`) → the sheet shows exactly the same total it would show with
no Flaw's XP effect at all yet — never zero, never silent. The instant the
player types a value, both effects switch on together: the training marker
counts, and `TruncatedApprenticeshipXp` has a value to fund from. There is no
window where one is "on" and the other's payoff is "zero."

**Exhaustive-match sites `ConfersHermeticTrainingIf` touches — the ordinary
16-site walk every other new D0 variant pays, NOT a widening ripple:**

| # | Site | Verdict |
|---|---|---|
| 1 | `effective.rs` macro tail | no-op — add to the tail (same family as the bare marker) |
| 2-7 | `ability_bonus`, `art_bonus`, `deficient_arts`, `characteristic_score_bonus`, `ability_affinity`, `art_affinity` | no-op (via macro) |
| 8-10 | `effective/spell.rs`'s three hand tails | no-op — a training marker, not a spell-levels/general-XP/mastery-affinity contribution |
| 11 | `derived.rs::in_play_mods` | no-op — creation-time, not an in-play total |
| 12 | `effective/xp.rs::ability_authorizations` | no-op — same reasoning as the bare marker's own arm there: grants training as a fact, not Ability/category ownership |
| 13 | `ruleset/integrity.rs::validate_effect_refs` | **real**: `param` must resolve to a declared parameter on the SAME item — no domain restriction (a presence test is domain-agnostic; unlike `TruncatedApprenticeshipXp.param`, this one is not required to be `Number`-domain, since it only ever asks "was anything chosen") |
| 14 | `validation/mod.rs::effect_target` | `Other` — matches the bare marker's own classification |
| 15 | `ruleset/integrity.rs::validate_item_ratios` | no-op |
| 16 | `ui/src/lib/types.ts::Effect` | **omit** — no UI consumer reads it, matching `TruncatedApprenticeshipXp`'s own row 16 |

The bare `ConfersHermeticTraining`'s seven existing production sites
(`effective/xp.rs:887`, `derived.rs:258`, `effective/hermetic_training.rs:38`,
`ruleset/integrity.rs:2619`, `validation/mod.rs:1327`, `effective.rs:405`,
`effective/spell.rs:77,158,557`) are **untouched** — each gains a sibling arm
for the new variant, none is retrofitted.

**Red test owed (architect's own note, test-verifier Erika): the transition
state itself**, in `Advisory` AND `Silent` modes — `flaw.abandoned_apprentice`
selected, `years_completed` unanswered: `is_hermetically_trained() == false`,
`general_pool_and_bonus` returns the ordinary companion `later_life_xp`
(matching a 20-year-old's pre-Flaw total, e.g. `225`), not `0`. This is the
FIRST red D3 must show — before the fully-answered case the note already
proposed.

### Engine change

**Revision 3 renumbering — F4 (architect): D3 depends on D2, stated plainly.**
Step 2 below extends `extra_apprenticeship_years` (§ 3's `.max()` helper),
which D2 introduces. The first draft's §7 claimed D3 was independent and
"may run... first or last" — that claim is withdrawn; see § 7.

1. `LifeStageBudget` gains three fields: `truncated_training_years: u32`,
   `truncated_training_xp: u32`, `truncated_training_spell_levels: u32`
   (all `0` for every character not carrying the effect, including a real
   magus).
2. `LifeStageRules::budget()`: when an effective selection carries
   `TruncatedApprenticeshipXp{param}` alongside a resolved
   `ConfersHermeticTrainingIf{param}` on the SAME item (F1/R3-1's gate having
   resolved, `entity_confers_hermetic_training` therefore TRUE) **and**
   `apprenticeship_of()` returns `None` (companion-shaped, trained by
   selection — the D56 asymmetry), read `years_completed` directly off the
   selection's own `param` (D62 — no subtraction from an age; the parameter
   IS the year count) and set the three new fields. **F3 (architect,
   MAJOR): explicit saturating arithmetic, matching `ScaledRestrictedAbilityXp`'s
   own precedent (`effective/xp.rs:1176`, `per_unit.saturating_mul(units)`)
   verbatim, and the same fallible parse `ScaledRestrictedAbilityXp` already
   uses** (`effective/xp.rs:1167-1174`: `.and_then(|v|
   v.as_str().parse::<u32>().ok())`, skipping the effect on failure to
   parse):
   ```rust
   let Some(years) = selection.params.get(param)
       .and_then(SelectionParamValue::as_single)
       .and_then(|v| v.as_str().parse::<u32>().ok())
   else { /* unanswered — F1's gate already keeps `trained` false here too */ return default; };
   let truncated_training_xp = rules.truncated_xp_per_year.saturating_mul(years);
   let truncated_training_spell_levels = rules.truncated_spell_levels_per_year.saturating_mul(years);
   ```
   `budget()` is reachable on a plain load, NOT behind `MAX_XP_SOLVE_NODES`'s
   guard (`build_capacity_matrix`'s guard is a different call path) — a
   hand-edited save naming an absurd `years_completed` (bypassing the
   parameter's own `1..=14` bound, enforced only by the display-only
   validator) must not panic under `overflow-checks = true`
   (`Cargo.toml:17-26`). `saturating_mul` guarantees this regardless of how
   large `years` claims to be; the resulting (nonsensically huge) total is
   simply what an out-of-range value already reports as illegal via the
   existing parameter-range finding — correctness of a rejected value is not
   this arithmetic's job, not panicking is.
   `extra_apprenticeship_years` (§ 3) gains `years_completed` as a third
   candidate in its `.max()` chain, feeding ONLY `later_life_carve_years`
   (§ 3's F2 fix) — **never** `LifeStageBudget.apprenticeship_years`, which
   stays `magus_apprenticeship_years` alone, unaffected by D2 or D3.
   **Red test owed (F3): a crafted `years_completed` at `u32::MAX` (a legal
   parse, illegal per the declared range) must not panic** — asserts
   `truncated_training_xp == u32::MAX` (saturated, not wrapped), no crash,
   under `overflow-checks = true`.
3. `general_pool_and_bonus`'s `if trained` branch (`effective/xp.rs:1489-1491`)
   widens from `apprenticeship_xp.saturating_add(post_gauntlet_xp)` to also
   add `.saturating_add(budget.truncated_training_xp)` — a real magus has `0`
   here (unaffected), an Abandoned Apprentice with `years_completed` answered
   has `0` for the other two terms (unaffected in the other direction). This
   is the line that **flips the pinned baseline test** — but only once F1's
   gate has resolved `trained` to `true`, i.e. only in the fully-answered
   state; the transition state (F1's own red test) never reaches this branch
   at all.
4. `effective/spell.rs::life_stage_spell_levels` (`:209-214`) widens from
   `budget.post_gauntlet_spell_levels` to
   `budget.post_gauntlet_spell_levels.saturating_add(budget.
   truncated_training_spell_levels)` — the single selector both
   `validate_spells` and the DTO read.
5. `validation/life_stage.rs::validate_life_stage_plan`'s `magus` flag
   (`:95`) **stays profile-only, unchanged** — D56/A0's own exemption already
   keeps `apprenticeship.minimum_abilities` from being enforced against him.
6. Parma advisory (D56: *"If the character knows the Parma Magica, he must
   join the Order or be slain"*, ArMDE:5647): `flaw.abandoned_apprentice`
   gains `advisory_prerequisites: { kind: "nor", value: [{ kind: "has",
   value: "ability.parma_magica" }] }` — F-550/Q8's existing advisory
   machinery, zero new mechanism. Text for the passage's full consequence
   lives in `description` (D50).

### Validation

`magus_minimum_abilities` stays empty for him (profile-only gate,
unaffected). The Parma advisory surfaces as `CODE_ADVISORY_PREREQ_NOT_MET`,
existing infrastructure. The unanswered-parameter state itself already
produces the generic `missing_param` display finding — F1's fix (§ above) is
about what the ENGINE computes meanwhile, not a new validator.

**New hard error, Revision 4 (architect finding R3-2): `years_completed` must
not exceed what the character's own age can have lived.** `validate_life_
stage_age_meets_minimum` (`validation/life_stage.rs:149-190`) already raises
`CODE_LIFE_STAGE_AGE_BEFORE_GAUNTLET` for a real magus (`gauntlet_age <
minimum_gauntlet_age()`) and `CODE_LIFE_STAGE_AGE_BEFORE_CHILDHOOD` for
everyone else (`age < childhood.years`) — but nothing checks the companion-
shaped analogue: a 6-year-old companion could legally set `years_completed:
14` (the parameter's own declared max), an impossible timeline (14 years of
training completed after only 1 year past childhood), and the merged
later-life formula (`(age − childhood.years) − years_completed`) simply
`saturating_sub`s to `0` **silently** — F3's fail-safe arithmetic correctly
not panicking, but here masking a genuinely invalid character rather than an
attack, which is exactly the class of defect F1 was.

**Fix: widen the SAME function with a third branch**, reusing the `budget`
parameter it already receives (which will carry D3's new
`truncated_training_years` field, § "Engine change" step 1) rather than
adding a new parameter:

```rust
let (subject_age, min_age) = if magus {
    (budget.map_or(age, |b| b.gauntlet_age), rules.minimum_gauntlet_age())
} else if budget.is_some_and(|b| b.truncated_training_years > 0) {
    (age, rules.childhood.years + budget.unwrap().truncated_training_years)
} else {
    (age, rules.childhood.years)
};
```

New code, **error** severity (matching both existing siblings, not a
warning — an impossible timeline is exactly as hard a defect as a magus
gauntleted before the minimum age), `CreationPhase::Experience`, args
`age`/`min`:

```rust
pub const CODE_LIFE_STAGE_AGE_BEFORE_TRUNCATION: &'static str =
    "life_stage_age_before_truncation";
```

Fluent, both locales, matching the sibling keys' exact style:

```ftl
# en
issue-life_stage_age_before_truncation = Age { $age } is too young to have completed { $min } years of childhood and apprenticeship before being abandoned.

# de
issue-life_stage_age_before_truncation = Alter { $age } ist zu jung, um vor der Verstoßung { $min } Jahre aus Kindheit und Lehrzeit durchlaufen zu haben.
```

**Red test (the architect's own worked example): a 6-year-old companion
holding `flaw.abandoned_apprentice` with `years_completed: 14`** must raise
`CODE_LIFE_STAGE_AGE_BEFORE_TRUNCATION` with `age: "6"`, `min: "19"`
(`childhood.years(5) + years_completed(14)`) — error severity, `Experience`
phase — and must NOT silently accept the character with `later_life_years`
saturated to `0` and no finding at all, which is today's (Revision 3's)
behavior.

### Integrity

- `TruncatedApprenticeshipXp.param` must resolve to a declared `Number`-domain
  parameter on the same item — rejected otherwise, item id + param key named.
- `ConfersHermeticTrainingIf.param` must resolve to a declared parameter on
  the same item, any domain (R3-1's own table).
- **Mandatory (Revision 3, no longer "recommended"): `years_completed`'s
  authored `max` must equal `apprenticeship.years − 1`** — rejected
  otherwise, per Norbert's own "taken from the ruleset rather than
  hardcoded."
- **Revision 4 addition (architect finding R3-3): when the ruleset ships an
  item carrying `TruncatedApprenticeshipXp`/`ConfersHermeticTrainingIf` but
  ships NO `apprenticeship` block at all**, `apprenticeship.years` does not
  exist for the bound check above to compare against. **Rejected outright,
  not silently skipped**: an item declaring this parameter shape is
  meaningless without the block that bounds it, so the same integrity pass
  refuses the ruleset with an error naming the offending item and stating
  that it requires an `apprenticeship` block. Not reachable in the shipped
  ruleset (which always ships one), but a homebrew ruleset that copies
  `flaw.abandoned_apprentice`-shaped data without the block must fail loudly
  at load, per `CLAUDE.md`'s "fail loudly with clear error listing offending
  IDs" — silently skipping the check would let the parameter's `max` go
  unverified forever on such a ruleset.

### Exhaustive-match sites `TruncatedApprenticeshipXp` touches

Same 16-site walk § 3 uses (`ConfersHermeticTrainingIf`'s own 16-site table
is under F1 above — two different new variants, two separate tables):

| # | Site | Verdict |
|---|---|---|
| 1 | `effective.rs` macro tail | no-op — add to the tail |
| 2-7 | `ability_bonus`, `art_bonus`, `deficient_arts`, `characteristic_score_bonus`, `ability_affinity`, `art_affinity` | no-op (via macro) |
| 8 | `effective/spell.rs::spell_levels_bonus` | no-op — the 8×years spell-levels term is folded via `LifeStageBudget.truncated_training_spell_levels` (Engine change step 4), a SEPARATE selector. Adding it here too would double-count |
| 9 | `effective/spell.rs::general_xp_bonus` | no-op — the 16×years XP term is folded via `general_pool_and_bonus` (Engine change step 3), not this per-effect fold |
| 10 | `effective/spell.rs::spell_mastery_advancement_affinity` | no-op |
| 11 | `derived.rs::in_play_mods` | no-op — a creation-time life-stage grant, not an in-play total |
| 12 | `effective/xp.rs::ability_authorizations` | no-op — an Abandoned Apprentice's Arcane/Academic/Martial access comes from `is_hermetically_trained`'s whole-character exemption (`validation/authorization.rs:28-53`, fed by either `ConfersHermeticTraining` or `ConfersHermeticTrainingIf`), not from this variant, which only sizes a pool the exemption has already opened |
| 13 | `ruleset/integrity.rs::validate_effect_refs` | **real**: `param` must resolve to a declared `Number`-domain parameter on the same item |
| 14 | `validation/mod.rs::effect_target` | `Other` — matches `RestrictedAbilityXp`'s own classification |
| 15 | `ruleset/integrity.rs::validate_item_ratios` | no-op |
| 16 | `ui/src/lib/types.ts::Effect` | **omit** — no UI consumer reads it; the Abandoned Apprentice's totals reach the UI through the already-DTO'd `LifeStageBudget`/`RestrictedXpPool` fields |

### Saves

No bump. `years_completed` is an ordinary `Selection.params` entry (already
`BTreeMap<String, SelectionParamValue>`, landed by C0b) — the *type* is
unchanged, only a new legal parameter key on one catalogue entry. The shipped
`flaw.abandoned_apprentice` JSON edit is the only "migration": per A0 § 5,
this is deliberately **not** touched by Group A and lands here.

### Data vs engine

| Touches | This slice (engine) | This slice (data) |
|---|---|---|
| D3 | `LifeStageBudget` fields, `budget()`'s widened computation (saturating), `general_pool_and_bonus`'s widened branch, `life_stage_spell_levels`'s widened sum, `TruncatedApprenticeshipXp` variant + integrity, `ConfersHermeticTrainingIf` sibling variant + integrity (Revision 4, R3-1), the new `CODE_LIFE_STAGE_AGE_BEFORE_TRUNCATION` validator (Revision 4, R3-2), the missing-`apprenticeship`-block rejection (Revision 4, R3-3), the advisory-prereq machinery (already built) | `flaw.abandoned_apprentice`'s `parameters`+`effects`+`advisory_prerequisites` edit; `ApprenticeshipRules`'s two new ruleset fields + their values (16, 8) in `rules/core/life_stages.json`; `RULES.md` |

---

## 5. D49 is superseded by D62 — D4 is dropped

**Revision 3 (Norbert, D62, `decisions.md:831-847`): "If none creation-relevant,
it's not necessary to model."** Checked and confirmed: non-magi's later-life
XP is already a flat rate (`later_life_xp_rate`, landed), seasons never enter
it; magi's post-Gauntlet lab seasons are already capped at 3/year
(`max_charged_lab_seasons_per_year`), and the only magus-eligible season cost
(Regular, ArMDE:6677, "Magi can be Regular") leaves `4 − 1 = 3`, which does not
move that cap. **No shipped or plausible carrier is creation-relevant.** D4,
`Effect::FreeSeasonsPerYear`, the widened lab-season validator, and open
questions § 6.2 (the cross-item conditional) and § 6.4 (`points_per_year`)
are all withdrawn — none of this note's Revision 2 engine design ships. The
season rules of Landed Noble, License of Absence, Lone Redcap, Redcap,
Wealthy, Poor and Regular stay exactly what they already were: `description`
text, both locales, no computed effect.

---

## 6. Open questions for Norbert

**Revision 3: this section shrinks to one item.** Former § 6.1 (what funds an
Abandoned Apprentice's years after abandonment) is **closed by D62**
("the years after abandonment follow the later-life rules") — dropped, not
merely answered. Former §§ 6.2 and 6.4 were both D4/D49-scoped and are
withdrawn along with D4 itself (§ 5). Former § 6.5 (the parameter's
bound-consistency check) is promoted from "nice to have" to a mandatory
integrity check in § 4, per Norbert's own "taken from the ruleset rather than
hardcoded" — settled, not open.

### 6.1 — When an earmark's `amount` exceeds what its eligible Abilities can absorb (§ 2)

D13 does not rule on this — see § 2's "capacity does not move" correction for
the full argument. Recommendation: accept the shortfall, surfaced only via
the existing `CODE_RESTRICTED_XP_UNSPENT` warning, rather than building a
refund mechanism back to `general`. Restated here only as an index entry; § 2
carries the reasoning.

---

## 7. Slice table

**Revision 3: no D4 row.** D4 is dropped (§ 5, D62). D0 is now a three-slice
design (D1, D2, D3).

| Slice | Content | Red tests (fail today, and why) | Depends on | e2e / bump | Locale strings (both) |
|---|---|---|---|---|---|
| **D0-step0** (architect F6) | `LifeStageBlock` relocation ALONE, per § 1 — its own green commit, full suite passing, unchanged, landing BEFORE any red test below | none — this step is deliberately behavior-free; the suite must stay 100% green through it, so a reviewer can tell "broke because of the move" from "red because a field doesn't exist yet" (CLAUDE.md's red-checkpoint discipline) | none | none | none |
| **D1** (D13) | `from_normal_budget` field; `earmarked_general_xp`; widened `general_pool_and_bonus`; `flaw.church_upbringing`'s data | a hand-authored earmark pool: total demand fundable is unchanged (general shrinks by exactly the earmark's amount) — fails today (field does not exist, and today's `RestrictedAbilityXp` would over-fund by the amount if wired naively); `restricted_ability_xp_pools`'s pattern fails to **compile** the moment the field is added (no `..`) until the arm is updated (legitimate first red per CLAUDE.md's TDD rule) | D0-step0; C1, C3 (landed) | none (no save/IPC shape change) | none new — `flaw.church_upbringing`'s full passage already belongs in `description` per D5/D20, both locales, landing in the same commit as the data edit |
| **D2** (D40) | `ReplacesLifeStageXp` variant + `Apprenticeship` `LifeStageBlock` variant; `extra_apprenticeship_years`; childhood-collapse branch in `build_flow_pools`; `flaw.feral_upbringing`, `virtue.redcap`, `virtue.lone_redcap` data (XP half only — the authorization half is B1's `RestrictsAbilityCategoryToAbilities`, coordinate so one commit does not orphan the other per D40's own note) | a Feral-Upbringing-shaped fixture: total childhood funding is 120, not 240 (F-428) — fails today; a Redcap-shaped fixture at age 25: total budget is 495 (120+300+75), not 720 (F-439) — fails today; a Lone-Redcap fixture: `virtue.redcap` itself funds 300 where it funds 0 today | D1 (shares `general_pool_and_bonus`/`earmarked_general_xp` locality, run after so the two touch the same function once) | e2e only at the D-group boundary — no save/IPC shape change | none new beyond the existing `description` fields (F-428/F-439's passages are already the entries' own text) |
| **D3** (D56 truncated) | `ApprenticeshipRules`'s two new ruleset fields; `TruncatedApprenticeshipXp` variant; `ConfersHermeticTrainingIf` sibling variant (Revision 4, R3-1 — bare `ConfersHermeticTraining` untouched); `LifeStageBudget`'s three new fields; widened `budget()` (saturating, F3), `general_pool_and_bonus`, `life_stage_spell_levels`; the mandatory `years_completed`-bound integrity check + its missing-block rejection (R3-3); the new `CODE_LIFE_STAGE_AGE_BEFORE_TRUNCATION` validator (R3-2); `flaw.abandoned_apprentice`'s `parameters`+`effects`+`advisory_prerequisites` edit; a numeric `<input>` for `years_completed` (reuses C3's existing number-parameter control — no new UI control) | **the transition-state red (F1, new — must land FIRST)**: the Flaw selected, `years_completed` unanswered, in `Advisory` and `Silent` modes — `is_hermetically_trained() == false`, general pool reads the ordinary companion `later_life_xp`, never `0`; **the age-consistency red (R3-2)**: a 6-year-old companion with `years_completed: 14` must raise `CODE_LIFE_STAGE_AGE_BEFORE_TRUNCATION` (`age: 6`, `min: 19`), not silently saturate to `0` with no finding; **the pinned baseline itself**: `abandoned_apprentice_xp_shape_is_unchanged_pending_d3` (`data_integrity.rs:5817`) must now show the fully-answered `general_pool == 16×years_completed`, not `225`; a new test: a 20-year-old companion who completed 7 years of apprenticeship gets `16×7=112` general XP and `8×7=56` spell levels, Parma allowed-with-warning, `magus_minimum_abilities` empty; **the crafted-input red (F3)**: `years_completed` at `u32::MAX` must not panic under `overflow-checks=true`, `truncated_training_xp` saturates rather than wraps | **D3 depends on D2** (architect F4) — step 2 of § 4's engine change extends `extra_apprenticeship_years`, the `.max()` helper D2 introduces. D0-step0; D2; C3 (`ParamType::Number`, landed); A1 (`ConfersHermeticTraining`, landed) | e2e at the D-group boundary; **portable e2e not implicated** (no `arm-app`/load-path change) | Parma-advisory warning text (new `CODE_ADVISORY_PREREQ_NOT_MET` context, reused code); the new `issue-life_stage_age_before_truncation` key (R3-2, both locales); the full ArMDE:5647-5650 passage in `description`, both locales |

**Sequencing.** D0-step0 (the `LifeStageBlock` relocation, its own green
commit) first, always. Then D1 before D2 (both touch `general_pool_and_bonus`'s
same few lines; landing D1 first means D2's diff is smaller and does not
re-litigate the earmark subtraction). **D3 after D2 — not independent, not
"may run first or last"** (the first draft's claim, withdrawn per F4): D3
extends `extra_apprenticeship_years`, code D2 introduces, so D3 cannot be
implemented as specified until D2's version of that helper exists on disk.
True order: D0-step0 → D1 → D2 → D3.

**e2e.** Per the plan's phase-boundary rule, one run after D3 (the D-group
boundary), none inside any individual slice — none of D1-D3 changes the save
format or an IPC shape.

---

## References loaded

`docs/vf-audit/phase-2-plan.md`; `docs/vf-audit/decisions.md` §§ D12, D13,
D17, D35, D40, D43, D48, D49, D56; `docs/vf-audit/design-a0-is-magus-split.md`
(in full — the hand-off note for D3, `is_hermetically_trained`'s two-function
split, the profile-only life-stage exemption); `docs/vf-audit/design-c0-
parameter-model.md` (in full — `ParamType::Number`, `AbilityRef`/`ParamGate`,
the 16-site exhaustive-match table, the schema-bump criterion); `docs/vf-
audit/design-b0-ranging-and-predicates.md` (in full — the 16-site table
reused here, the "curated TS subset" rule, Revision 2/3's wrapper-rejection
lesson); `crates/arm-rules/src/effective/xp.rs` (in full — `LifeStageBlock`,
`RestrictedXpPool`, `FlowPool`, `PoolEligibility`, `build_flow_pools`,
`restricted_ability_xp_pools`, `general_pool_and_bonus`, `build_capacity_
matrix`, `two_phase_max_flow`, `ability_authorizations`, `MAX_XP_SOLVE_NODES`);
`crates/arm-rules/src/life_stage.rs` (in full through its test module —
`LifeStageRules`, `ApprenticeshipRules`, `LifeStageBudget`, `budget()`,
`later_life_years`, `apprenticeship_of`, `minimum_gauntlet_age`,
`later_life_rate`); `crates/arm-rules/src/validation/life_stage.rs`
(`validate_life_stage_plan`'s profile-only `magus` gate, `restricted_xp_
unspent`'s occurrence at `:782-783`); `crates/arm-rules/src/validation/
magus.rs:880-959` (`validate_xp_pool`, `CODE_RESTRICTED_XP_UNSPENT`);
`crates/arm-rules/src/effective/spell.rs:195-235` (`spell_levels_base`,
`life_stage_spell_levels`, `spell_levels_budget`); `crates/arm-rules/src/
types.rs:1330-1530` (`Effect`'s XP-family variants, verified field-by-field);
`crates/arm-rules/src/validation/mod.rs:1249-1330` (`effect_target`);
`crates/arm-rules/src/ruleset/integrity.rs:2354-2480` (`validate_effect_refs`'s
`RestrictedAbilityXp`/`ScaledRestrictedAbilityXp` arms, both already
`..`-tolerant); `crates/arm-rules/src/effective.rs:318` (the macro);
`crates/arm-rules/src/derived.rs:207` (`in_play_mods`); `crates/arm-rules/
tests/data_integrity.rs:5755-5850` (`abandoned_apprentice_requires_the_gift`,
the pinned baseline test D3 flips, both read directly); `rules/core/
virtues_flaws.json` (every carrier id's exact current shape, verified by
direct read: `flaw.abandoned_apprentice:12-19`, `flaw.church_upbringing:347-
354`, `flaw.feral_upbringing:1071-1079`, `virtue.landed_noble:4852-4859`,
`virtue.license_of_absence:4959-4966`, `virtue.lone_redcap:5017-5029`,
`virtue.redcap:5786-5794`, `flaw.poor:2183-2192`); `rules/core/character_
types.json` (all four profiles, confirming A2's conditional `creation_phases`
and B2's `virtue_category_caps.min` are already landed); `ui/src/lib/
components/XpBar.svelte` (in full — the existing childhood spread-only
display branch this note's D2 reuses, the `xp-pool-block-*` Fluent-key
family); `locales/en/*.ftl` (exact existing key text, confirmed no new key is
needed for D2's childhood case); rulebook source, verified directly against
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md`:
ArMDE:5789-5791 (Church Upbringing), ArMDE:6110-6113 (Feral Upbringing),
ArMDE:4219-4228 (Landed Noble), ArMDE:4291-4294 (License of Absence),
ArMDE:4319-4326 (Lone Redcap), ArMDE:4842-4851 (Redcap), ArMDE:5641-5650
(Abandoned Apprentice), ArMDE:2433-2482 (the four-period apprenticeship/
post-apprenticeship block, Darius example, lab-season charging).

**Revision 3 additions**: `docs/vf-audit/decisions.md` D62 (`:831-847`, full);
`tmp/d0-architect-review.md` (in full); `crates/arm-rules/src/effective/
hermetic_training.rs` (in full — `entity_confers_hermetic_training`,
`is_hermetically_trained`); `crates/arm-rules/src/life_stage.rs:432-433,527,
569` (`LifeStageBudget.apprenticeship_years`'s doc comment and both its
readers, re-verified); `crates/arm-rules/src/effective/xp.rs:1-17` (K3/K9
header doc), `:1167-1176` (`ScaledRestrictedAbilityXp`'s existing
`saturating_mul`/fallible-parse precedent, re-verified as the pattern D3's
own arithmetic now cites verbatim); `Cargo.toml:17-26` (`overflow-checks =
true`); `grep -rn "Effect::ConfersHermeticTraining" crates/arm-rules/src`
(every production match site, confirming all are bare-unit patterns).

**Revision 4 additions**: `tmp/d0-architect-review.md`'s "Focused Re-Review:
Revision 3" section (in full); `crates/arm-rules/src/types.rs:1081-1086`
(`ParamGate{param, equals: Id}`), `:971-987` (`required_if` on
`ParameterDef`, confirmed a validation-time concept, not fold-time — not
reusable here); `crates/arm-rules/src/effective.rs:76-100`
(`selections_for_effects`, confirmed it folds every selection's effects
unconditionally with no `missing_param`/`required_if` awareness);
`crates/arm-rules/src/validation/life_stage.rs:149-190`
(`validate_life_stage_age_meets_minimum`, the function R3-2's fix extends);
`locales/en/main.ftl:1170-1171`, `locales/de/main.ftl:1237-1238` (the sibling
`issue-life_stage_age_before_*` keys' exact naming/wording convention).

## Revision 2 (2026-09-28)

Applied against the plan-reviewer's "approve after fixes" verdict
(`tmp/d0-plan-review.md`): 4 MAJOR, 2 MINOR. All six applied in place, listed
here rather than re-stated (each fix carries its own "Revision 2" marker at
its actual location):

| # | Finding | What changed |
|---|---|---|
| D0-1 (MAJOR) | D49's `min(max_charged_lab_seasons_per_year, free_seasons_per_year)` formula conflated the lab-season charging ceiling with the ambient 30/year rate, contradicting D49's own naive-comparison warning | § 5 "Validation" rewritten: the ceiling formula is kept (it is what D49 actually names) but scoped explicitly to *chargeable seasons only*; whether `points_per_year` itself should shrink is split out as new open question § 6.4, with a recommendation (no) rather than a silent assumption; a red test for `free_seasons_per_year < 4` added to § 7's D4 row |
| D0-2 (MAJOR) | `virtue.redcap`'s own unconditional "two seasons per year" (ArMDE:4850) missing from the carrier sweep | Added as a fourth carrier in § 5's Findings table, ships unconditionally in D4 (no §6.2 cross-item block — the entry itself excludes Wealthy/Poor), threaded through the JSON example, Data-vs-engine table, and § 7's slice table |
| D0-3 (MAJOR) | D3's `TruncatedApprenticeshipXp` had no exhaustive-match-site table, unlike D2/D4 | Added the 16-site table to § 4, including the explicit TS-omission call the documentation reviewer asked for |
| D0-4 (MAJOR) | D1's "the budget does not move" claim conflated capacity with spendable total | § 2 "The shape actually chosen" now distinguishes the two explicitly; new open question § 6.3 (D13 does not rule on an earmark exceeding absorbable capacity) with a recommendation |
| D0-5 (MINOR) | D3's "Type" heading read "No new `Effect` variant," contradicting its own body | Heading text corrected in place |
| D0-6 (MINOR) | § 1 overstated the `LifeStageBlock` relocation's footprint as six files, one-line each | Corrected to the actual three-file footprint, verified by reading every reference rather than counting `grep` hits |

Open-question numbering shifted: former § 6.3 (the `age_abandoned`
bound-consistency check) is now § 6.5; §§ 6.1-6.2 are unchanged in content.

## Revision 3 (2026-09-28)

Applied against the architect's "needs work" verdict (`tmp/d0-architect-
review.md`: 1 BLOCKER, 3 MAJOR, 2 MINOR) and two rulings from Norbert
(`decisions.md` D62). Two structural changes plus the six findings:

| # | Source | What changed |
|---|---|---|
| D62 (1) | Norbert | **D4 dropped.** § 5 rewritten to a short supersession paragraph: neither non-magi's later-life rate nor magi's 3/year lab-season cap is creation-relevant to any season effect, Regular included. `Effect::FreeSeasonsPerYear`, its validator, and open questions former § 6.2/§ 6.4 are withdrawn. Season rules stay `description` text. |
| D62 (2) | Norbert | **D3's parameter replaced.** `age_abandoned` → `years_completed` (an integer, 1…`apprenticeship.years − 1`, bound taken from the ruleset — now a MANDATORY integrity check, not a deferred nice-to-have). The years-before-apprenticeship and years-after-abandonment spans merge into one ordinary later-life bucket — this closes and DROPS former § 6.1 (not merely answers it). § 4 rewritten throughout. |
| F1 (BLOCKER) | architect | `ConfersHermeticTraining` gains an optional presence gate (`requires_param: Option<String>`, distinct from B4's equality-testing `ParamGate` — deliberately not reused, see § 4's own reasoning) so the training marker and `TruncatedApprenticeshipXp`'s payoff resolve TOGETHER. Before `years_completed` is answered, the character reads as untrained and gets the ordinary companion later-life total — never a silent zero. New transition-state red test in `Advisory`/`Silent` modes. |
| F2 (MAJOR) | architect | `LifeStageBudget.apprenticeship_years` (the struct field) now stays sourced ONLY from a real magus's own block, matching its doc comment for every D2/D3 carrier too. A separate, unnamed local (`later_life_carve_years`) carries the `.max()`-widened value D2/D3 need, feeding `later_life_years()` alone. Fixed in § 3's engine-change text. |
| F3 (MAJOR) | architect | D3's `16 × years`/`8 × years` arithmetic now explicit: `saturating_mul`, fallible `.parse::<u32>()` (mirroring `ScaledRestrictedAbilityXp`'s own existing precedent verbatim) — `budget()` is reachable on a plain load, not behind `MAX_XP_SOLVE_NODES`'s guard, and the release profile enables `overflow-checks`. New crafted-input red test at `u32::MAX`. |
| F4 (MAJOR) | architect | § 7's independence claim for D3 withdrawn — D3 extends `extra_apprenticeship_years`, code D2 introduces, so the true order is D1 → D2 → D3, stated plainly in the corrected slice table and sequencing paragraph. |
| F5 (MINOR) | architect | § 3's row 16 (`ReplacesLifeStageXp`) flipped from "real" to "omit," matching D3's and the (now-withdrawn) D49's own row 16 and the reasoning all three actually state. |
| F6 (MINOR) | architect | The `LifeStageBlock` relocation is now its own named slice-table row (D0-step0), a green, behaviour-free commit landing before D1's compile-forced red — not bundled into D1's diff. |

Open-question numbering collapsed to a single item: former §§ 6.1 (D62-closed),
6.2/6.4 (D4-withdrawn), and 6.5 (now mandatory, not open) are all gone; the
surviving earmark-capacity question (former § 6.3) is renumbered § 6.1.

## Revision 4 (2026-09-28)

Applied against the architect's focused re-review of Revision 3
(`tmp/d0-architect-review.md`, "Focused Re-Review: Revision 3" section):
F1-F6 confirmed closed against current code; 2 new MAJOR + 1 new MINOR.

| # | Finding | What changed |
|---|---|---|
| R3-1 (MAJOR) | Revision 3's `ConfersHermeticTraining.requires_param` widened the existing bare-marker variant in place, forcing `{ .. }` onto seven production sites that will never read the new field — inconsistent with the note's own cited `Foo`/`FooParam` sibling-variant idiom, which would touch none of them. | Reverted the widening. New sibling variant `ConfersHermeticTrainingIf { param: String }`; the bare `ConfersHermeticTraining` and its seven existing sites are untouched. `entity_confers_hermetic_training` gains a second match arm, not a widened first one. Full 16-site table added under § 4's F1 section, replacing the old 7-site ripple table. Integrity/JSON/engine-change text updated to the new variant name throughout. |
| R3-2 (MAJOR) | No validator ties a companion's `years_completed` to its own age — a 6-year-old could legally set `years_completed: 14`, silently saturating `later_life_years` to `0` with no finding, the same "wrong rules output silently accepted" class as F1. | New hard error `CODE_LIFE_STAGE_AGE_BEFORE_TRUNCATION`, added as a third branch of the EXISTING `validate_life_stage_age_meets_minimum` (reusing the `budget` parameter it already receives, no new function parameter): `age < childhood.years + truncated_training_years` fails, `Experience` phase, both-locale Fluent keys specified, red test at age 6 / `years_completed: 14` (expects `age: 6`, `min: 19`). |
| R3-3 (MINOR) | The mandatory `years_completed`-bound integrity check didn't state its behavior when a ruleset ships no `apprenticeship` block at all. | Specified: rejected outright, not silently skipped — an item declaring this parameter shape with no block to bound it against is meaningless data, so integrity refuses the ruleset naming the offending item, per `CLAUDE.md`'s "fail loudly" standard. |

No new open questions; § 6.1 (the earmark-capacity question) is unchanged.

## Verdict

COMPLETE — ready for re-review. One item remains genuinely open for Norbert
(§ 6.1) and is not an invented ruling.

## New Learnings

- **A restricted-XP "earmark" cannot be modelled as `general → pool` in this
  engine's max-flow solve.** `two_phase_max_flow` closes `SOURCE→GENERAL`
  during phase 1 specifically so restricted pools are preferentially drained
  before general opens; a pool fed FROM `GENERAL` has no supply in phase 1 and
  competes on equal footing with ordinary general spends in phase 2, making
  its reported `used` figure a non-deterministic function of augmenting-path
  order. The correct shape is: shrink the general base by the earmark amount,
  and fund the earmark as an ordinary `SOURCE`-fed pool — algebraically
  identical total, but reuses the existing (and tested) restricted-pool
  preference machinery instead of fighting it.
- **`LifeStageBlock` belongs in `life_stage.rs`, not `effective/xp.rs`** — it
  was placed in the XP-solve module before any `Effect` needed to name it as
  data; once one does (D40's `stage` field), leaving it where it is would
  force `types.rs` to depend on `effective`, inverting the crate's own
  established `types → effective` direction. A relocation (pure code motion)
  is cheap; discovering the layering violation only at that point is not.
- **Reusing an existing `LifeStageBlock` tag for a new UI state, rather than
  minting a new one, can retire a whole slice's UI work for free** — Feral
  Upbringing's replacement pool, tagged with the SAME `ChildhoodSpread` origin
  the ordinary spread pool already uses, renders through `XpBar.svelte`'s
  existing "spread-only" branch (built for a different reason — no native
  language named yet) with zero new Svelte/Fluent work.
- **A ruling that warns "these two quantities must not be compared naively"
  is not itself the formula — do not reach for the nearest-looking combinator
  (`min()`) and call the warning satisfied.** D49 named the trap precisely and
  still got walked into it in this note's first draft, because a validator
  ceiling and a rate are both "numbers about seasons" and `min()` type-checks
  against either. Split the question explicitly (which existing quantity, if
  any, does the new one actually constrain?) before writing the consumer, and
  if the ruling itself does not say, that split is exactly where the open
  question belongs — not folded silently into a formula that looks complete.
- **A design note's own self-imposed conventions (here: one exhaustive-match
  table per new `Effect` variant) need a checklist pass across every
  variant the note introduces, not a check-as-you-go while drafting each
  one** — the very variant with no existing sibling to copy from (D3's
  `TruncatedApprenticeshipXp`, unlike D1/D2's reuse of landed patterns) was
  the one that skipped it, which is exactly backwards: a novel variant is
  where the table earns its keep most.
- **When a sibling `Effect` grants a base-switching fact (not an additive
  bonus), an unconditional marker on the same item is a real, ordinary-use
  bug, not a crafted-input edge case.** Every prior parameterized grant in
  this engine (`ScaledRestrictedAbilityXp` included) is additive on top of an
  otherwise-unaffected base, so an unanswered parameter costs nothing —
  `TruncatedApprenticeshipXp` was the first case where a SIBLING effect
  (`ConfersHermeticTraining`) switches which base applies at all, and nothing
  about "additive effects fail safe" carries over to that shape. Check
  whether a new effect changes WHICH branch a consumer takes, not just what a
  branch computes, before assuming an unanswered parameter is harmless.
- **A `.max()`-widened local that is also shorthand-assigned into a
  documented struct field is a trap the shorthand itself hides.** `budget()`'s
  `apprenticeship_years` read as one honest local right up until two
  unrelated slices (D2, D3) each had a real reason to widen it for a
  DIFFERENT consumer (`later_life_years()`) than the one reading the struct
  field. `let x = ...; Struct { x, .. }` looks like a single, simple
  assignment — actually check every reader of a local before widening its
  definition, not just the one motivating the change.
- **"Ordinary use, not just a crafted save" and "no `MAX_XP_SOLVE_NODES` guard
  on this call path" are two independent robustness questions, and a design
  note has to ask both.** `budget()` sits entirely outside the flow-solve's
  own hardened guard, so a saturating-arithmetic commitment has to be stated
  explicitly at every new multiplication reachable from it, not inherited
  from a neighbouring function's own guard.
- **Citing an idiom as the reason a design choice is correct, and then not
  applying that same idiom to the choice actually being made, is a specific
  and checkable inconsistency — check it explicitly, not just "does a
  precedent exist somewhere."** Revision 3 named `Foo`/`FooParam` sibling
  variants as the reason a narrow field beats a generic mechanism, then
  widened an existing bare-marker variant in place instead of adding the
  sibling its own argument called for. The tell was available in the note's
  own text, not hidden in the code.
- **A `saturating_sub` that quietly floors an impossible timeline to zero is
  not automatically "safe" — it can convert a genuinely invalid character
  into a silently accepted one with no finding at all.** F3's fix (don't
  panic) and R3-2's fix (don't silently accept) are two different
  obligations that can look like the same "handle the edge case" line item;
  a design note has to name both, because satisfying one does not satisfy
  the other.
