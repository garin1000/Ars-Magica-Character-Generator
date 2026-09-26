# A0 — design: the `is_magus` split (D56)

Design note for Phase 2 group A, slice A0. No code changed by this document.
Reviewed by the plan-reviewer, then the architect, before A1 starts.

## 1. The two facts

D56: *"`is_magus` becomes Hermetically trained and member of the Order. The
first four rows key on trained; Houses key on Order."*

| Fact | Derivation | Kind |
|---|---|---|
| **Hermetically trained** | `profile.hermetically_trained` (renamed from `is_magus`) **OR** the entity's present (bought ∪ granted) selections include an item carrying the new `Effect::ConfersHermeticTraining` marker | **Both** profile-level *and* entity-level — a union |
| **Member of the Order** | `profile.order_member` (the other half of the old `is_magus`) | Profile-level only — no entity override exists or is asked for |

Only the magus profile sets both to `true` today; every other profile sets
both to `false`. The split is invisible for every character *except* one whose
selections confer training without the profile declaring it — the Abandoned
Apprentice companion (`flaw.abandoned_apprentice`, ArMDE:5641-5650, filed under
D56/Q-84).

**Why a union, not two independent profile flags.** D56 is explicit that the
Flaw "confers *training*, not the Gift" — a *selection* effect, the same shape
as every other creation-time grant in this engine (`RestrictedAbilityXp`,
`AbilityScoreGrant`, `CharacteristicPoints`, …). Modelling it any other way
(e.g. a bespoke `Entity::hermetically_trained_override: bool` field) would put
a second, ad-hoc derivation path next to the one that already exists for every
other Virtue/Flaw effect, which is exactly the "third incompatible spelling"
D13 and D17 warn against for the XP modes. A new `Effect` variant is the
existing, reviewed mechanism, and zero new field — but it is **not** a
one-arm cost. `Effect` is matched exhaustively at every fold and every
referential-integrity check in the engine; § 1a enumerates every site this
variant forces a decision at, corrected from an earlier draft that
understated this to "one compile-forced match arm."

**Why Order membership stays profile-only for A1 — but this is an open risk,
not a closed one.** An earlier draft claimed outright that no book passage
proposes a selection granting Order membership without the magus profile.
That is wrong, and verified wrong: `virtue.redcap` (ArMDE:4842-4851,
`character_types.json`-permitted on the companion profile, `social_status`
category) states at ArMDE:4844 *"Although you do not have The Gift and cannot
work Hermetic magic, you are a full member of the Order of Hermes and of
House Mercere"* — an Order member who is explicitly **not** Hermetically
trained. `virtue.lone_redcap` (ArMDE:4319-4326) is the same figure without
Mercere ties: ArMDE:4321 *"You are a Redcap who does not maintain ties to a
Mercer House"*, still bound to the Order (ArMDE:4323, "Orbus" expulsion is
*from the Order*, via the House). Both are `creation_effect` companion-shaped
Virtues in `rules/core/virtues_flaws.json` today, with no `prerequisites`
field at all.

**This does not change A1's scope.** Houses remain the only consequence this
split gates on Order membership (`validation/magus.rs::validate_house`), and
neither Redcap entry grants a `house_specialisation` creation phase or any
House selection — a Redcap has no House pick to make, Order member or not. So
`order_member` stays **profile-only** through A1: there is still no reachable
site where an entity-level override of `order_member` would change today's
behavior. What changes is the confidence of the claim that motivated it —
this is now a recorded **open risk**, not a closed design decision, and it
must be re-checked the moment Redcap itself is encoded as a mechanic (D2/X5;
see the note below).

### Notes for D2/X5

D2 (D40's replacement-XP shape, `virtue.redcap`/`virtue.lone_redcap` named in
its own carriers list) and X5 (§ 3.6 prerequisites/eligibility) are the
slices that actually build out these two entries. When either does: **re-open
the entity-level-Order-membership question**, this time for real, rather than
inheriting A0's "profile-only" call unexamined. A Redcap is a companion who
is an Order member without training — the mirror image of the Abandoned
Apprentice this whole design is built around — so if any later mechanic asks
"is this entity in the Order" (a Tribunal-attendance rule, a House-adjacent
eligibility, anything ArMDE:4842-4851/4319-4326 might yet motivate), the
answer for a Redcap-shaped companion must be **yes**, and a profile-only
`order_member` cannot give it. Whether that ever becomes reachable is D2/X5's
call, not A0's — this note only obliges the question be asked there, not
answered here.

**Data-driven, per `CLAUDE.md`'s catalogue-size invariant.** The engine adds
one `Effect` variant and one derivation function; which catalogue entries carry
that effect is data. Today that is exactly one entry
(`flaw.abandoned_apprentice`); a future entry (translated content, a new
sourcebook) can carry it with zero code changes, exactly like every other
`Effect`.

**The union is load-bearing, not decorative — independent of the Abandoned
Apprentice entirely.** The magus profile's own `creation_phases` orders
`virtues_flaws` *before* `arts`/`spells` (`character_types.json`:
`["concept", "characteristics", "house_specialisation", "virtues_flaws",
"experience", "abilities", "arts", "spells", ...]`). A fresh magus entity, the
instant the player picks the `magus` type and before making a single V/F
selection, has nothing yet for a pure selection-fold to find. If
`is_hermetically_trained` dropped the `profile.hermetically_trained` term and
relied on `entity_confers_hermetic_training` alone, a brand-new magus would
read as untrained until his first relevant selection — hiding the Arts and
Spells tabs (once A2 wires them to this same fact) at the exact moment of
character creation they must be visible. The profile term is therefore not
only how the Abandoned Apprentice's *absence* of a profile flag is bridged;
it is how an *ordinary* magus's presence of one is honored before he has
selected anything at all.

### The shared derivation point

Lives in a new `effective/hermetic_training.rs`, sibling to
`effective/gift_confidence.rs::has_the_gift` (the module `has_the_gift`
already lives in, matching this fact's own role as a second necessary-but-
not-sufficient precondition alongside it — D24).

```rust
/// Single source of truth for "is this entity's Hermetic training real,
/// whether by profile or by selection?" Mirrors `has_the_gift`'s shape and
/// role (RULES.md's warning against re-deriving `gift_categories` logic
/// twice applies here too).
pub fn is_hermetically_trained(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: Option<&EntityTypeProfile>,
) -> bool {
    profile.is_some_and(|p| p.hermetically_trained)
        || entity_confers_hermetic_training(entity, ruleset)
}
```

Every "trained" production site in § 4 is obliged to call this function
instead of reading `profile.hermetically_trained` directly — **except** the
`profile-only` sites (§ 4, life-stage rows), which have a documented reason
to read the profile flag alone, and **except `PrereqCtx`**, which cannot use
this `bool`-returning helper at all — see the note below.

**Why `is_hermetically_trained` returns `bool`, but `PrereqCtx` cannot use it
directly.** Every one of its callers (`derived.rs`, `effective/xp.rs`,
`validation/magus.rs::validate_spells`, `validation/authorization.rs`,
`effective/warping.rs`, `validation/warping.rs`,
`effective/reputation_and_caps.rs::supernatural_free_slots`,
`effective_dto.rs`) already receives `profile: Option<&EntityTypeProfile>`
and already collapses a missing profile to "not a magus" via
`.is_some_and(...)` **today** — that collapse is not new behavior the split
introduces, it is what every one of these sites already does. `bool` is
therefore the right return type for all of them: it preserves exactly
today's semantics for the one case (`None`) that changes nothing.

`validation/prereq.rs::PrereqCtx` is different in kind, not degree: it is the
**one** place in the engine that threads a genuine three-valued outcome —
`Tri::True` / `Tri::False` / `Tri::Unknown` — because
`validate_prerequisites` must distinguish "this prerequisite is definitely
unmet" (a hard `prereq_not_met` error) from "this cannot be evaluated with
the data on hand" (a soft `prereq_unevaluated` warning), and a missing type
profile is exactly the second case, not the first. `PrereqCtx::build`
therefore keeps its field `Option<bool>` and builds it by mapping *over* the
`Option`, not by collapsing it:

```rust
// `type_profile.map` short-circuits to `None` (→ Tri::Unknown) when the
// profile itself cannot be resolved, exactly mirroring how `is_magus` built
// today (`let is_magus = type_profile.map(|p| p.is_magus);`) — the OR-check
// only ever runs once a profile *is* in hand.
let trained: Option<bool> = type_profile
    .map(|p| p.hermetically_trained || entity_confers_hermetic_training(entity, ruleset));
let order: Option<bool> = type_profile.map(|p| p.order_member);
```

So the engine ships **two** related functions, not one, and each is correct
for its own callers precisely because they answer different questions
("what should this validator/derivation assume" vs. "what does the evidence
actually show, including its absence"):

| Function | Returns | Used by |
|---|---|---|
| `entity_confers_hermetic_training(entity, ruleset) -> bool` | pure selection fold, no profile involved | both of the below |
| `is_hermetically_trained(entity, ruleset, profile: Option<&_>) -> bool` | collapses a missing profile to `false`, matching every existing non-`PrereqCtx` call site | every "trained" row in § 4 except `PrereqCtx` |
| `PrereqCtx::build`'s inline `type_profile.map(...)` | preserves `None` as `Tri::Unknown` | `Prereq::HermeticallyTrained`/`Prereq::OrderMember` evaluation only |

### 1a. Every `match` on `Effect` this variant touches

`Effect::ConfersHermeticTraining` is a bare marker variant (no fields — it
names no ability, no amount, nothing to resolve), matching the existing shape
of `Effect::MasterpieceItem`, `Effect::ForbidsAbilitySpecialties`,
`Effect::ForbidsRitualCasting`, and `Effect::WaivesAbilityAgeCap`. Verified by
reading every `match` over `Effect` in the tree (not by counting the macro's
own call sites, which undercounts — see the correction below).

**One shared tail absorbs six of them automatically.**
`effective.rs::irrelevant_effect_variants` is the ~40-variant "everything
else is a no-op" macro five folds share; adding
`Effect::ConfersHermeticTraining` to *that one list* is the only edit needed
for:

| Symbol | Why it is a no-op there |
|---|---|
| `effective/art.rs::art_bonus` | not an Art-bonus effect |
| `effective/art.rs::deficient_arts` | not a Deficiency |
| `effective/ability.rs::ability_bonus` | not an Ability-bonus effect |
| `effective/characteristic.rs::characteristic_score_bonus` | not a Characteristic delta |
| `effective/xp.rs::ability_affinity` | not an Affinity-cost effect |
| `effective/xp.rs::art_affinity` | not an Affinity-cost effect |

**Seven sites hand-list their own tail (per the macro's own documented
exception for an unconditional interesting arm) and each needs one new,
explicitly-reasoned arm, landed in the same commit as the variant:**

| # | Symbol | New arm | No-op reason (comment style matches the surrounding code) |
|---|---|---|---|
| 1 | `effective/spell.rs::spell_levels_bonus` | `| Effect::ConfersHermeticTraining => None,` (joins its existing `|`-chain) | "Not a spell-levels contribution — training is a creation-legality fact, not a levels grant." |
| 2 | `effective/spell.rs::general_xp_bonus` | `| Effect::ConfersHermeticTraining => None,` | "Not a general-XP contribution — the apprenticeship *shape* this confers is folded in `effective/xp.rs`, not here, or it would double-count exactly as the file's own `LaterLifeXpRate` comment warns against." |
| 3 | `effective/spell.rs::spell_mastery_advancement_affinity` | `| Effect::ConfersHermeticTraining => None,` | "Not a Spell Mastery Affinity — grants no advancement multiplier." |
| 4 | `effective/xp.rs::ability_authorizations` | `| Effect::ConfersHermeticTraining => {}` (or absorbed into its existing no-op `|`-chain) | "Grants Hermetic training as a fact, not permission to own a specific Ability or category — Arcane authorization for a trained non-magus is a **profile/entity-level** gate in `validation/authorization.rs`, not a per-effect grant here." |
| 5 | `ruleset/integrity.rs::validate_effect_refs` | joins the existing `| ... => { continue; }` "bare marker: no parameter, no ref to resolve" group beside `ForbidsAbilitySpecialties`/`ForbidsRitualCasting`/`WaivesAbilityAgeCap` | Identical shape: no parameter, no ref, nothing to validate. |
| 6 | `validation/mod.rs::effect_target` | joins the existing `| ... => EffectTarget::Other` tail | "No ability/characteristic creation-time target — a training marker, not a score effect." |
| 7 | `derived.rs::in_play_mods` | `Effect::ConfersHermeticTraining => {}` (new standalone arm, beside `Effect::DeficientArt { .. } => {}`) | "A creation-time training marker, not an in-play total; no-op here — Casting/Lab Totals themselves are unaffected by *how* training was acquired." |

**One site needs no edit at all, and is worth recording so it is not
mistaken for a gap:** `ruleset/integrity.rs::validate_item_ratios` ends its
`match effect` in `_ => continue`, so the new variant is silently and
*correctly* absorbed — it carries no ratio, so the wildcard is right, not a
smell. (A second `match effect { .. }`-shaped site in the same file,
`validate_aging_row_effect`, matches `AgingRowEffect`, an unrelated enum, and
is not a real `Effect` site at all — excluded from the count above.)

**Net correction to "zero enum churn."** Slice 1 (§ 5) lands the variant, all
seven hand-written arms above, and the one-line macro addition **in the same
commit** — eight edits in total, not one, before anything else in sub-slice 1
can compile.

## 2. What replaces `Prereq::IsMagus`

| | Old | New |
|---|---|---|
| Variant | `Prereq::IsMagus` | `Prereq::HermeticallyTrained`, `Prereq::OrderMember` |
| Serde tag | `{"kind": "is_magus"}` | `{"kind": "hermetically_trained"}`, `{"kind": "order_member"}` |
| `PrereqCtx` field | `is_magus: Option<bool>` | `trained: Option<bool>`, `order: Option<bool>` |
| Evaluator arm | `Prereq::IsMagus => ctx.is_magus.map(...)` | one arm per variant, same `Option<bool>` → `Tri` shape |

`Prereq` is deliberately exhaustive (`types.rs::Prereq`), so removing `IsMagus`
and adding two variants is a **compile error at every match site** until
handled — this is the invariant working as designed, not a risk to route
around.

**No serde alias for the retired tag.** `EntityTypeProfile` and `Prereq` live
in `rules/core/*.json`, which is developer-authored data shipped in lockstep
with the binary and referenced by `ruleset.id`+`version`
(`RulesetRef` on `Entity`) — it is not migrated forward the way a *save* is.
There is no "old ruleset file in the wild" to stay compatible with, so
`{"kind": "is_magus"}` is edited in place, not aliased. (Contrast saves: § 3.)

### Rules-JSON migration

Exactly **one** production data site uses the old tag —
`grep -rn '"kind": "is_magus"' rules/core` finds only:

```
rules/core/virtues_flaws.json:2293  flaw.primogeniture_lineage
  "prerequisites": { "kind": "all", "value": [
    { "kind": "is_magus" }, { "kind": "house", "value": "house.verditius" } ] }
```

This is a Verditius-only Flaw (`RULES.md` documents it at the `All([IsMagus,
House(house.verditius)])` shape, `:446`, `:594`). Migrated form:

```json
"prerequisites": { "kind": "all", "value": [
  { "kind": "order_member" }, { "kind": "house", "value": "house.verditius" } ] }
```

**Not `hermetically_trained`.** House membership is D56's own worked example
of an *Order* concern; `Prereq::House` alone would not gate a Gifted-but-Order-
less character the way `RULES.md:603` already argues `IsMagus` had to, and
under the split it is `OrderMember` that carries that job.

`EntityTypeProfile.is_magus: bool` splits into two booleans on the same struct
(`character_types.json`'s four profile objects each replace one key with two):

```json
// before
"is_magus": true,
// after
"hermetically_trained": true,
"order_member": true,
```

Both keep the existing field's serde shape
(`#[serde(default, skip_serializing_if = "is_false")]`) — an omitted key still
means `false`, so `grog`/`companion`/`mythic_companion`'s existing terse JSON
(which never writes `"is_magus": false` … actually today's data **does** write
it explicitly for clarity, see `character_types.json:29,63,145` — kept as
explicit `false` for both new keys, matching current style) is unchanged in
spirit.

### Exhaustive-match impact

Every non-test production site that pattern-matches `Prereq` (the evaluator in
`validation/prereq.rs`, the parity-shape walk in `ui/src/lib/derive.ts`, and
nothing else — `conflicts_with_house` already treats `IsMagus` in a catch-all
arm alongside `Has`/`AbilityMin`/`ArtMin`, so `types.rs:373-376` needs only its
comment reworded, not restructuring) goes red at compile time until handled.
This is the intended cost, not a defect to work around.

### `ui/src/lib/prereq-parity.test.ts`

Reads variant names mechanically from both `types.rs`'s `Prereq` enum body and
`types.ts`'s `Prereq` union — it hardcodes no kind string itself, so it needs
**no edit**. It goes red the moment the Rust side changes and stays red until
`types.ts` gains the two new `{kind: 'hermetically_trained'}` /
`{kind: 'order_member'}` members and `houseOnlyValue`'s switch gets an
explicit case for each (its own test forbids a catch-all). This is a
**cross-cutting compile-time-equivalent gate that spans the Rust/TS boundary**
— see § 5's ordering note.

### `CLAUDE.md`'s Prereq quick reference

The `enum Prereq` code block under "Data model quick reference" lists
`IsMagus` as a bare unit variant. A1 replaces that one line with two:

```rust
enum Prereq {
    ...
    HermeticallyTrained,
    OrderMember,
}
```

## 3. Saves

`crates/arm-rules/src/types.rs::Entity` carries **no** `is_magus`-shaped field.
The flag lives only on `EntityTypeProfile` (ruleset data, looked up at load
time via `Entity::type_id`) and in the transient `EffectiveScores` DTO
(derived fresh every load, never stored). Confirmed by reading the full
`Entity` struct (`types.rs::Entity`) — its persisted fields are
`schema_version`, `ruleset`, `entity_kind`, `type_id`, `selections`,
`characteristics`, `xp_pool`, `life_stages`, `ability_funding`,
`wizard_furthest_phase`, `art_scores`, … — none of them is a magus/trained/
order boolean.

**Per the plan's bump criterion** ("shape or meaning moves → bump… A1 only if
A0 finds the entity stores the flag") — **it does not. No `SCHEMA_VERSION`
bump for A1.** The new `Effect::ConfersHermeticTraining` marker lives on
`rules/core/virtues_flaws.json`'s `effects` array (rules data, keyed by
`ruleset.version`, not by `schema_version`), not on any saved `Selection`.

The age parameter for the truncated apprenticeship (D56 "What this obliges" #3,
D35's numeric type) *does* touch `Selection.params`, but that is D3's slice,
not A1's, and D3 makes its own bump call independently — nothing here commits
D3 to "no bump."

## 4. Per-site verdict: all 52 files

Legend: **trained** = keys on `is_hermetically_trained`; **order** = keys on
`profile.order_member`; **both** = the file has sites needing each, separately;
**neither** = fixture/comment only, no behavior keys on either fact;
**profile-only** = correctly stays keyed on `profile.hermetically_trained`
alone (the union would be a defect here — flagged explicitly).

Counts are `total | prod | inline-test`, from § 8 row 12c's re-derivation
against today's tree (2026-09-26), which corrects one stale row (below).

### Rust `crates/arm-rules/src`, `crates/arm-app/src` (18 files, now 18 — see correction)

| # | File | Counts | Verdict | Reason |
|---|---|---|---|---|
| 1 | `validation/mod.rs` | 41 \| 1 \| 40 | both | The 1 prod site (`:967`) is a doc comment ("prerequisites' `is_magus` resolution") — reword to name both facts. The 40 inline-test sites are `Prereq::IsMagus` fixtures (`:6503-6731`) exercising the old single flag; split into trained-only, order-only, and a new Abandoned-Apprentice-shaped case per fact. |
| 2 | `types.rs` | 23 \| 9 \| 14 | both | This *is* the split point: `Prereq::IsMagus` variant (`:321`), the evaluator's catch-all arm (`:373-376`, comment only), `EntityTypeProfile.is_magus` field (`:2846`) and its doc (`:2811,2844,2849`). Splits into the two variants and the two profile booleans. Inline tests: serde roundtrip + `conflicts_with_house` + profile-flag roundtrip, each duplicated per new field/variant. |
| 3 | `derived.rs` | 21 \| 13 \| 8 | trained | Every prod site gates a magus-only derived-totals bundle (arts/casting/lab/penetration/magic_resistance/longevity/talisman/masterpiece/familiar) plus the `EffectiveScores.is_magus` DTO field itself (`:707`) that 8 UI files key off. All are D12 "trained" objects (Techniques/Forms/spells/Casting/Lab/Parma). Rename the DTO field to `hermetically_trained`; switch every gate from `profile.is_magus` to `is_hermetically_trained(entity, ruleset, profile)`. |
| 4 | `validation/prereq.rs` | 13 \| 9 \| 4 | both | The evaluator itself: `PrereqCtx.is_magus` splits into `trained`+`order`; the `Prereq::IsMagus => …` arm splits into two arms. Inline tests duplicate per fact (`magus_type`/`non_magus_type`/`unknown`/`independent_of_gift` × 2). |
| 5 | `validation/magus.rs` | 11 \| 10 \| 1 | both | Two independent site groups in one file: `validate_house` (`:41`, `if !profile.is_magus { return }`) is **order** — Houses are structurally Order-only, no entity ever overrides it. `validate_spells`/`validate_spell_level_cap`/`validate_spell_levels_budget` (`:439-470,644,704`) are **trained** — D56's own row 1 ("arts phase — yes, he casts spells"); these already receive `entity`, so the rewrite to `is_hermetically_trained` is a straight signature-compatible swap, not a new parameter. |
| 6 | `ruleset.rs` | 8 \| 0 \| 8 | both | Test-only (all 8 below `#[cfg(test)]`). Fixture JSON building magus-shaped profiles for unrelated integrity/apprenticeship/spell-cap tests; each becomes `"hermetically_trained": true, "order_member": true` (a real magus is both). |
| 7 | `ruleset/integrity.rs` | 8 \| 8 \| 0 | trained | `validate_engine_required_roles` (`:616`), `validate_apprenticeship_refs` (`:736`), `validate_post_apprenticeship_rules` (`:839`) — all three ask "does any profile declare itself hermetically trained by default", i.e. must the ruleset ship the Arts catalogue / apprenticeship / post-apprenticeship life-stage blocks. This is squarely D12's "trained" domain (spells, Lab, Arcane Connections, Casting), not Houses. **Profile-level only** (`profile.hermetically_trained`), since these are structural ruleset-shape checks, not per-entity — an entity-level union would be a category error here (the check runs with no entity in scope at all). |
| 8 | `effective/xp.rs` | 6 \| 6 \| 0 | trained | `base_general`'s apprenticeship+post-Gauntlet budget shape (D56 row 3, "partly"). Switch the branch condition to `is_hermetically_trained`. **The truncated per-year shape itself (16 XP/8 levels) is D3's slice, out of scope here** — A0's obligation is only that the entity-level fact exists and is available to `effective/xp.rs`; D3 builds the actual truncation branch on top of it. |
| 9 | `life_stage.rs` | 4 \| 2 \| 2 | **profile-only** | `magus_minimum_abilities` (`:642`) gates whether `apprenticeship.minimum_abilities` is checked at all. D56: *"The truncated block must NOT enforce `apprenticeship.minimum_abilities`… describe a completed apprenticeship."* Reading the **unioned** trained fact here would be the bug D56 explicitly forbids — an Abandoned Apprentice is trained but must **not** be held to Latin 1/Magic Theory 1/Parma 1. Keep `profile.hermetically_trained` (unchanged behavior for the magus profile, no change needed beyond the rename). |
| 10 | `validation/authorization.rs` | 3 \| 2 \| 1 | trained | `:61`, Arcane-Ability authorization "off the profile's `is_magus` flag" — D56 row 2 explicitly: the Abandoned Apprentice needs Arcane Abilities with no further Virtue. **This is a real behavior change, not a rename**: today an ungifted-profile companion with the Flaw is refused; after the split he is authorized. `entity` is already in scope at this call site. |
| 11 | `effective.rs` | 3 \| 0 \| 3 | both | Test-only fixtures (spell-levels-base doc references the magus exemption path). Rename only. |
| 12 | `validation/life_stage.rs` | 2 \| 1 \| 1 | **profile-only** | `:87`, same `magus_minimum_abilities`-adjacent warning path as row 9 — same reasoning, same caveat. |
| 13 | `effective/warping.rs` | 2 \| 2 \| 0 | trained | "Hermetic magi are exempt" from Warping/Twilight accretion (`:159-164`). Twilight exposure is a trained-only mechanic (Parma, Arcane Connections) per D12. **Behavior change**: an Abandoned Apprentice casting spells is equally exposed and must get the same exemption a magus gets. |
| 14 | `crates/arm-app/src/ruleset_io.rs` | **stale — see correction below** | — | — |
| 15 | `validation/warping.rs` | 1 \| 1 \| 0 | trained | Sibling of row 13, same Twilight/Warping exemption reasoning, same behavior change. |
| 16 | `export.rs` | 1 \| 0 \| 1 | both | Markdown-export test fixture, magus-shaped. Rename only. |
| 17 | `effective/reputation_and_caps.rs` | 1 \| 1 \| 0 | trained | `supernatural_free_slots` (`:76`): *"a magus gets none (his free ability is Hermetic magic itself)"* — this is a trained-only exemption dressed as `!profile.is_magus`. **Behavior change**: an Abandoned Apprentice's free Gift-slot must also be 0, exactly like a real magus, or D24's "Weak Parens is the worked example, is it the only one?" positive check gains a second failure. |
| 18 | `completeness.rs` | 1 \| 0 \| 1 | both | D58 scope-test fixture (`gift_policy: required`), magus-shaped. Rename only. |

**Correction to § 8 row 12c: `ruleset_io.rs` → `effective_dto.rs`.** Live-tree
recheck (`grep -c is_magus crates/arm-app/src/ruleset_io.rs` → **0**;
`crates/arm-app/src/effective_dto.rs` → **2**) shows the U0 slice (Phase 1,
"`EffectiveScores` DTO → `effective_dto.rs`", already landed ahead of Group A
per the plan's ordering) moved these two sites before A0 ran. This is exactly
the kind of drift `CLAUDE.md`'s "read the code itself" instruction exists to
catch; the 52-file *count* is unaffected (one file swaps for another), but the
row's filename in `measurements.md` is now wrong and should be corrected in
the same pass that consumes this table (X-something, or a one-line fix
alongside A1).

| # | File (corrected) | Counts | Verdict | Reason |
|---|---|---|---|---|
| 14′ | `crates/arm-app/src/effective_dto.rs` | 2 \| 2 \| 0 | trained | `:467`, spell-level-caps DTO field, gated "only for a magus" — per-Te/Fo Casting/Lab cap is D12 trained territory. Switch to `is_hermetically_trained`; once A2 exposes the Spells phase conditionally, an Abandoned Apprentice's caps compute correctly instead of coming back empty. |

**18 Rust files, 67 production occurrences** confirmed against today's tree
(matches the plan's "Facts re-checked" figure exactly).

### UI `ui/src` — production components (15 files, 27 occurrences)

| # | File | Counts | Verdict | Reason |
|---|---|---|---|---|
| 19 | `lib/types.ts` | 5 \| 5 | both | TS mirror of `Prereq`'s `is_magus` kind (`:268`) and `EntityTypeProfile.is_magus`/comment near `has_mythic_type` (`:957,1040,1042`). Gains the two new `Prereq` kinds and the two renamed profile booleans. The profile-boolean half has **no automated parity test today** (unlike the `Prereq` half) — flagged as a risk in § 7. |
| 20 | `App.svelte` | 5 \| 5 | both — **the site D56 warns will be underestimated** | One `isMagus` const (`:62-64`) feeds three unrelated jobs, conflated: (a) `hasMight` (`:71-72`, *"Magi never have Might"*) → **trained**; (b) the Arts+Spells tabs (`:95-100`) → **trained**, and per D56 this is literally the case A2's conditional-phases mechanism must drive, not a hand-rolled boolean; (c) the Possessions+House-specialisation tabs (`:106-111`) are **two different facts wired to one flag today** — Possessions (magic items/Talisman) is **trained**, `house_specialisation` is **order**. This last one is a genuine bug this split must fix, not just rename: today a hypothetical trained-non-Order character (none exists yet, but nothing stops the Flaw + a future variant) would wrongly see a House tab, or a hypothetical Order-only… (moot until such a character exists, but the *coupling* is real and worth recording). |
| 21 | `lib/components/SupernaturalBeing.svelte` | 3 \| 3 | trained | `:37`, `{#if !isMagus}` — *"Magi never have Might… reaches this tab only through Focus Power"* — same fact as row 20(a). Behavior change: Abandoned Apprentice hides the Might block too. |
| 22 | `lib/components/LifeStagePanel.svelte` | 3 \| 3 | **profile-only** | `:24,36`, `showPostGauntlet` — the Abandoned Apprentice never had a Gauntlet (D56: *"Not post-gauntlet… he never had a Gauntlet"*), so this must stay keyed on `profile.hermetically_trained` (the normal apprenticeship→Gauntlet→post-Gauntlet flow), **not** the unioned unclear fact. Mirrors row 9/12's caveat on the frontend side. |
| 23 | `lib/derive.ts` | 1 \| 1 | both | `:613`, `case 'is_magus':` inside the Prereq-to-text renderer. Needs one case per new kind, with distinct wording ("must be Hermetically trained" vs "must belong to the Order of Hermes"). |
| 24 | `lib/components/SpellBudgetBar.svelte` | 1 \| 1 | neither | Comment states the component is deliberately gate-free ("no `is_magus` test of its own"); no behavior change. |
| 25 | `lib/components/MagusMinimumAbilities.svelte` | 1 \| 1 | neither | Comment: the backend (`magus_minimum_abilities`) already returns empty for anyone off the normal apprenticeship track (row 9's profile-only fix handles this); component needs no change once row 9 lands. |
| 26 | `lib/components/DerivedTotalsPanel.svelte` | 1 \| 1 | trained | `:41`, `{#if d.is_magus}` — the DTO gate itself (renamed alongside `derived.rs` row 3). |
| 27 | `lib/components/DerivedPenetrationSection.svelte` | 1 \| 1 | trained | Comment only ("parent mounts this only inside its own `{#if d.is_magus}`"); rename in lockstep with row 26. |
| 28 | `lib/components/DerivedMasterpieceSection.svelte` | 1 \| 1 | trained | Same as row 27. |
| 29 | `lib/components/DerivedMagicResistanceSection.svelte` | 1 \| 1 | trained | Same as row 27. |
| 30 | `lib/components/DerivedLongevitySection.svelte` | 1 \| 1 | trained | Same as row 27. |
| 31 | `lib/components/DerivedLabCastingSection.svelte` | 1 \| 1 | trained | Same as row 27. |
| 32 | `lib/components/DerivedFamiliarSection.svelte` | 1 \| 1 | trained | Same as row 27. |
| 33 | `lib/components/DerivedAuraField.svelte` | 1 \| 1 | trained | Same as row 27. |

### UI co-located tests (19 files, 39 occurrences)

| # | File | Count | Verdict | Reason |
|---|---|---|---|---|
| 34 | `DerivedTotalsPanel.test.ts` | 8 | trained | Exercises the renamed DTO gate (row 26); add magus/non-magus cases already exist, extend with a trained-non-magus case once A2 lands (not required in A1). |
| 35 | `LifeStagePanel.test.ts` | 4 | profile-only | Mirrors row 22. |
| 36 | `VirtueFlawTab.test.ts` | 3 | trained (primary) | Magus-shaped fixtures for hermetic-category V/F selection and magus-gated entries (e.g. Deficient Form). Check during A1 for any House-specific assertion, which would be order instead. |
| 37 | `CharacterDetails.test.ts` | 3 | both | `:33` states outright: *"`is_magus` is the flag every gate keys off — never the type id"* — a general statement this design revises; `:118` mentions a ritual gated on `!is_magus` (needs inspection to place trained vs order, deferred to A1's per-assertion read). |
| 38 | `state.svelte.test.ts` | 2 | neither | Dirty-flag/store test using a magus-shaped ruleset fixture; unrelated to trained/order semantics. Rename only. |
| 39 | `VirtueFlawTab.client.test.ts` | 2 | trained (primary) | Client-mounted counterpart of row 36. |
| 40 | `AgingPanel.test.ts` | 2 | trained | `:30`, *"One profile per capability the aging surfaces read; `is_magus` is the flag"* — aging/longevity is trained territory (row 3/13/15). |
| 41 | `AbilityTab.test.ts` | 2 | trained | `:230`, *"Empty for every type but a magus… needs no `is_magus` test of its own"* — mirrors row 10's authorization site; comment wording updates once row 10's behavior change lands. |
| 42 | `App.test.ts` | 2 | both | Tracks `App.svelte`'s three-way split (row 20); resolved together with it. |
| 43 | `App.client.test.ts` | 2 | both | Same as row 42, client-mounted. |
| 44 | `WizardStep.test.ts` | 1 | neither | Magus-shaped fixture; the component itself carries no `is_magus` reference. Natural future home for an A2 conditional-phase test, not required now. |
| 45 | `WizardShell.test.ts` | 1 | neither | Same as row 44. |
| 46 | `SupernaturalBeing.test.ts` | 1 | trained | Mirrors row 21. |
| 47 | `SpellBudgetBar.test.ts` | 1 | neither | Mirrors row 24's "no gate" note. |
| 48 | `ParameterPicker.test.ts` | 1 | neither | Component (not in the 15-file production list) carries no `is_magus` reference; fixture boilerplate only. |
| 49 | `MagusMinimumAbilities.test.ts` | 1 | neither | Mirrors row 25. |
| 50 | `ExperienceStep.test.ts` | 1 | neither | Fixture contrasts a non-magus XP shape; component itself has no gate. Natural future home for a D3 truncated-apprenticeship UI test. |
| 51 | `DerivedTotalsPanel.client.test.ts` | 1 | trained | Client-mounted counterpart of row 34. |
| 52 | `CharacterBanner.test.ts` | 1 | neither | Component (not in the 15-file list) carries no `is_magus` reference; fixture boilerplate only. |

**Verdict counts (52/52 rows, all accounted for):**

| Verdict | Files |
|---|---|
| trained | 22 |
| order | 0 (standalone) — order appears only inside "both" rows 5 and 20 |
| both | 15 |
| profile-only (a flavor of trained) | 4 |
| neither | 11 |
| **Total** | **52** |

Not in the 52 (outside the `src` roots, per measurements.md's own scoping) but
touched by the same rename: `crates/arm-rules/tests/data_integrity.rs` (9
hits, including D24's load-bearing
`a_companion_does_not_gift_himself_with_a_hermetic_virtue`),
`crates/arm-rules/tests/uncomputed_clauses.rs` (1),
`crates/arm-rules/tests/roundtrip_proptest.rs` (1),
`ui/e2e/specs/grog-wizard-aging.e2e.js` (1). These are renamed alongside
whichever production site they exercise; none needs its own design decision.

## 5. A1 sub-slice order

Adopts the plan's own ordering — **validation → derived → effective/xp →
prereq/ruleset/integrity → arm-app → ui** — because it lets the two new
*profile* booleans and the shared `is_hermetically_trained` helper exist and
compile from sub-slice 1 onward, while deferring the actual `Prereq` enum
split (the exhaustive-match-forcing change on `Prereq`) to sub-slice 4. Slices
2–3 are then pure "read the right flag" rewrites with no further enum churn.
**Slice 1 is not churn-free itself** — it pays the `Effect` enum's cost
(§ 1a's eight edits) up front, in the same commit that introduces
`Effect::ConfersHermeticTraining` — and slice 4 pays the `Prereq` side's
"every match site goes red" cost once, separately, in one place.

**Shipped data is deliberately decoupled from the code that reads it, for the
whole of Group A — not just sub-slices 1–2.** Every sub-slice below, sub-slice
3 included, proves its behavior exclusively with **test-only** `Ruleset`
fixtures that attach `Effect::ConfersHermeticTraining` to a throwaway test
item id. **The shipped `rules/core/virtues_flaws.json` entry for
`flaw.abandoned_apprentice` is not touched anywhere in Group A.** It gains the
effect for the first time in **slice D3**, in the same commit as the
truncated-apprenticeship block (16 XP/8 levels per year) — this is the
orchestrator's decision, superseding this note's earlier "attach in sub-slice
3" design and the two mitigation options that went with it (resequencing D3
and the `NO_RULE_DESPITE_TOKEN` gate are both withdrawn: resequencing breaks
D3's real dependency on C3's numeric age parameter (D35), and the token list
certifies "states no rule" — this passage states one, so the token is the
wrong instrument for it).

**Consequence: until D3, the shipped Abandoned Apprentice behaves exactly as
he does on `main` today** — refused Arcane Authorization, no Casting/Lab/
Twilight-exemption, no apprenticeship-shaped XP — because nothing in his
shipped data yet says he is trained. This is **not a regression Group A
introduces and not a new interim window it opens**; it is today's status quo,
continuing unchanged. It is, however, a **known defect**, since D56 already
rules that he must have all of these — tracked here and closed at D3, not
before. State this plainly rather than letting it read as silently fine: the
Abandoned Apprentice is **not correctly buildable at all until D3 ships**,
Group A only having built and proved the machinery D3 then switches on.

| Sub-slice | Scope | First failing test | Must stay green |
|---|---|---|---|
| 1. validation | `types.rs` field split (`hermetically_trained`/`order_member`), the new `Effect::ConfersHermeticTraining` variant **plus all eight edits § 1a enumerates** (the macro addition and the seven hand-written arms — landed together, in this slice, because the variant must compile everywhere the moment it exists), `is_hermetically_trained()` + `entity_confers_hermetic_training()` helpers, rewrite `validation/magus.rs` (Houses→order, spells→trained), `authorization.rs`, `life_stage.rs`+`validation/life_stage.rs` (profile-only), `effective/warping.rs`+`validation/warping.rs`. `flaw.primogeniture_lineage`'s JSON migrated in the same commit (trivial, avoids a known-wrong interim state). **`rules/core/virtues_flaws.json`'s shipped `flaw.abandoned_apprentice` entry is NOT touched here** — every red/green pair below the fold uses a test-only fixture item carrying the effect. | `abandoned_apprentice_shaped_test_fixture_is_authorized_for_arcane_abilities_without_a_further_virtue` — a **test ruleset's** entity holding a fixture item with `ConfersHermeticTraining` passes where today's profile-only `IsMagus` gate refuses him; the real Flaw is unaffected. | Every existing magus/companion/grog/mythic_companion validation test (renamed fixtures, unchanged real-magus behavior); `flaw.primogeniture_lineage`'s Verditius-only refusal; the real `flaw.abandoned_apprentice` continues authorizing nothing extra. |
| 2. derived | Rename `EffectiveScores.is_magus` → `hermetically_trained`; switch `derived.rs`'s 9 magus-only bundles to the helper. **Still test-fixture-only**, same reason as slice 1. | `derived_totals_for_a_trained_non_magus_test_fixture_include_casting_lab_and_penetration` — currently-empty sections become populated for the test fixture; the real Flaw is unaffected. | Existing DTO fixtures for a real magus (renamed field, same values); the real `flaw.abandoned_apprentice` still computes empty Casting/Lab/Penetration, matching its still-unchanged shipped data. |
| 3. effective/xp | Switch `effective/xp.rs`'s `base_general` branch condition to the helper. **Test-fixture-only, same reason as slices 1–2 — the shipped `flaw.abandoned_apprentice` entry is NOT edited in Group A at all.** The data edit (`"effects": [{"type": "confers_hermetic_training"}]`) moves to **slice D3**, landed there in the same commit as the truncated 16-XP/8-levels-per-year block, because only D3 can fund what sub-slice 3 merely makes the branch capable of selecting. | `apprenticeship_shaped_test_fixture_selects_the_apprenticeship_xp_branch_when_trained_off_profile` — a **test ruleset's** entity gets the apprenticeship-shaped pool; the real Flaw is unaffected and keeps computing exactly as it does on `main` today. | Every existing magus XP-pool test; the real `flaw.abandoned_apprentice` continues funding nothing extra, matching its still-unchanged shipped data — this is expected, not a gap, until D3. |
| 4. prereq/ruleset/integrity | The real `Prereq` enum split (`IsMagus` → `HermeticallyTrained`+`OrderMember`), `PrereqCtx` field split, every compiler-forced match site (`ruleset.rs` test fixtures), `ruleset/integrity.rs`'s three structural checks (rename only, unchanged meaning), `RULES.md` and `CLAUDE.md`'s Prereq reference. **Also lands the minimal `ui/src/lib/types.ts` `Prereq` union edit** (just the two kind strings) so `prereq-parity.test.ts` does not go red across a slice boundary — see § 7 risk 2. | `data_integrity.rs`'s existing `IsMagus`-named tests fail to *compile* under the new variant names (a legitimate first red per `CLAUDE.md`'s TDD rule) until renamed; new `prereq_order_member_and_trained_are_independent` exercises all four `(trained, order)` combinations via an Abandoned-Apprentice-shaped fixture. | `rulebook_citations.rs`, `rules_md_citations.rs`, `source_citations.rs` guards (RULES.md/CLAUDE.md edits keep citation shape valid); `prereq-parity.test.ts` (green, not just not-yet-red). |
| 5. arm-app | `effective_dto.rs`'s 2 sites (row 14′) → helper. | An arm-app-level DTO test: Abandoned Apprentice's spell-level caps list is non-empty. | Every existing magus DTO test. |
| 6. ui | Full `types.ts`, `derive.ts`, `App.svelte`'s three-way split (row 20), `SupernaturalBeing.svelte`, `LifeStagePanel.svelte` (profile-only), the 8 Derived* components (pure rename), and every co-located test per § 4's table. | `App.client.test.ts`, against a **test-only** trained-non-magus fixture (the real Flaw is still not trained until D3): sees the Possessions tab but not `house_specialisation` — proves the row-20(c) bug is actually fixed, not just renamed past. | Every "neither" and "profile-only" row's test (renamed-only, no behavior asserted to change). |

**Hand-off note for D3's brief.** D3 (D56's truncated apprenticeship, 16
XP/8 levels per year, gated on C3's numeric age parameter) must itself add,
as part of its own scope — not inherited pre-built from Group A:

1. The shipped-data edit: `rules/core/virtues_flaws.json`'s
   `flaw.abandoned_apprentice` entry gains
   `"effects": [{"type": "confers_hermetic_training"}]`, in the same commit as
   the per-year truncation logic.
2. Its own first failing test against the **real** Flaw (not a fixture):
   an Abandoned Apprentice built at a chosen age gets exactly
   `16 × years` XP and `8 × years` spell levels, Parma allowed-with-warning,
   `apprenticeship.minimum_abilities` **not** enforced (D56) — landing all at
   once, since before this commit he is not buildable as trained at all.
3. Confirmation that every Group-A site this note lists as `trained` (§ 4)
   now activates correctly for the real Flaw, not only for Group A's test
   fixtures — this is D3's acceptance check that the machinery Group A built
   was actually wired to something in the end.

There is consequently **no over-funding window and no regression to track**:
today's shipped Abandoned Apprentice (not buildable as trained at all) is a
known defect, already ruled by D56 and left unfixed on purpose until D3 —
Group A changes nothing about what he can do on `main`, only what the engine
is *capable of* once D3 tells it to.

## 6. A2 outline: conditional `creation_phases`

**Schema.** `EntityTypeProfile.creation_phases: Vec<PhaseRule>` where
`PhaseRule` mirrors the existing, already-reviewed `CategoryRule` shape
exactly:

```rust
#[serde(untagged)]
enum PhaseRule {
    Always(CreationPhase),
    When { phase: CreationPhase, when: Prereq },
}
```

Backward-compatible: today's bare-string phase lists deserialize unchanged
into the `Always` arm (identical to how `permitted_categories` already
supports a bare category slug beside a `{category, when}` object).

**Resolution.** A new `phases_in_force(entity, ruleset, profile) ->
Vec<CreationPhase>`, evaluated through the same `PrereqCtx` and the same
`Tri::Unknown`-is-not-in-force convention `categories_in_force` already
established — "one mechanism for 'applies conditionally'", exactly D56's and
D21's argument.

**Data.** The magus profile's `"arts"` and `"spells"` entries become
`{"phase": "arts", "when": {"kind": "hermetically_trained"}}` /
`{"phase": "spells", "when": {"kind": "hermetically_trained"}}` (still always
true for the magus profile itself). The companion profile gains the *same*
two entries (absent today), so they appear once `ctx.trained` is true. The
magus profile's `"possessions"` phase gets the same `hermetically_trained`
condition; `"house_specialisation"` gets `{"kind": "order_member"}` instead —
this is where the App.svelte row-20(c) conflation actually gets fixed at the
data layer, not patched around in the component.

**UI.** Today's `App.svelte` re-derives `isMagus`/tab visibility **client-side
from the raw profile flag** — there is no TS equivalent of
`categories_in_force`/`phases_in_force` (`grep -rl categories_in_force
ui/src` finds nothing), unlike the `Prereq` *shape*, which is mirrored and
kept in parity by `prereq-parity.test.ts`. Re-implementing a second Prereq
*evaluator* in TypeScript for phases would be exactly the duplication that
test exists to prevent for the shape half — so A2 should **not** hand-roll a
client-side evaluator. Instead: extend the DTO (`effective_dto.rs`) with a
resolved `phases_in_force: Vec<String>` (ordered, already evaluated
server-side), and have `App.svelte`'s tab list intersect its static tab
metadata against that resolved list rather than deriving `isMagus` itself.
This also directly retires the `isMagus`/`hasMythicType` consts in favor of
list membership, which is a net simplification, not just a rename.

**Integrity: `creation_phases[].when` must be walked at load, exactly like
`permitted_categories`/`forbidden_categories`.**
`ruleset/integrity.rs::validate_category_rule_conditions` already exists
precisely for this shape — it walks every `CategoryRule::when` condition on a
profile through `validate_prereq_refs`, because (per its own doc comment) "a
profile-borne `Prereq` is authored in the `rules/` directory beside the
binary — the project's declared hostile-input surface — so it must fail the
load on a dangling ref rather than sit there permanently unevaluable." A
`PhaseRule::when` is the identical shape and sits at the identical trust
boundary, so A2 must add a sibling —
`validate_phase_rule_conditions(type_id, profile, errors)`, called from
`validate_integrity` beside `validate_category_rule_conditions` — walking
`profile.creation_phases` the same way. Without it, a dangling ref in a
`{"phase": "arts", "when": {...}}` clause would not fail to load; it would
silently resolve to `Tri::Unknown` at evaluation time and (per
`categories_in_force`'s own "Unknown is not-in-force" convention) hide the
phase forever, with no error anywhere naming the typo.

**First failing test.** A ruleset fixture with
`"creation_phases": [{"phase": "arts", "when": {"kind": "has", "value": "virtue.no_such_id"}}]`
must fail `Ruleset::validate_integrity` with an error naming the profile, the
phase, and the dangling id — mirroring the existing
`validate_category_rule_conditions` test shape exactly (same fixture pattern,
`permitted_categories` swapped for `creation_phases`).

## 7. Risks and open points

| # | Risk | Why it matters | What decides it |
|---|---|---|---|
| 1 | **Resolved by the orchestrator (superseding this note's earlier draft).** The shipped `flaw.abandoned_apprentice` entry is not edited anywhere in Group A; the data edit moves to slice D3, in the same commit as the truncated apprenticeship block. | Group A therefore opens **no over-funding window**: the shipped Abandoned Apprentice behaves exactly as on `main` today (not buildable as trained) until D3, where every trained-gated site turns on together, correctly funded from that first commit. Today's behavior is itself a known defect (D56), left open on purpose until D3 — not a regression Group A causes or must mitigate. | Closed. D3's brief carries the data edit and its own first failing test — see § 5's hand-off note. (Rejected alternatives, for the record: resequencing D3 right after A breaks its real dependency on C3's D35 age parameter; a `NO_RULE_DESPITE_TOKEN` entry is the wrong tool because that list certifies "states no rule," and this passage states one.) |
| 2 | Cross-language compile-equivalent gate: sub-slice 4 changing the Rust `Prereq` enum makes `prereq-parity.test.ts` red in `ui/src`, which is sub-slice 6's territory. | Violates "every commit is green" if slice 4 lands alone. | Sub-slice 4 must carry the minimal `types.ts` edit (two kind strings + `houseOnlyValue` cases) as a rider, deferring the *rest* of the UI work to slice 6. Flagged explicitly in § 5's table; the plan-reviewer should confirm this rider is acceptable scope creep into a "prereq" slice rather than pushing the parity fix to slice 6 with a temporarily-red gate (not acceptable — no red commits). |
| 3 | `EntityTypeProfile`'s profile-boolean split (`hermetically_trained`/`order_member`) has **no automated Rust/TS parity test**, unlike `Prereq`'s kinds. **A2 widens the same gap**: `phases_in_force`'s resolved strings (§ 6) must match `App.svelte`'s static tab-id metadata, and no mechanical check found covers that either — checked directly: `ui/src/lib/types.ts`'s `CreationPhase` union carries a comment claiming "a Rust test pins this union against `CreationPhase::ALL`", but the only matching Rust tests (`types.rs::creation_phase_all_has_no_type_phase`, `::creation_phase_experience_serializes_as_experience`) assert properties of `CreationPhase::ALL` alone — count, absence of a retired slug, relative ordering — none of them reads `types.ts`, so the comment's claim does not hold today; no TS-side test reading `CreationPhase::ALL` was found either. | A future third profile-level boolean, or a `phases_in_force` string with no matching tab id (or vice versa), could drift between `types.rs`/`character_types.json` and `types.ts`/`App.svelte` silently — the same class of bug `prereq-parity.test.ts` was built to catch for `Prereq`, twice over now. | Out of A0's scope to fix, but worth a follow-up open-todo, widened by this round: generalize `prereq-parity.test.ts`'s technique (mechanical extraction from both sources) to (a) `EntityTypeProfile`'s field list and (b) `CreationPhase::ALL` vs. `App.svelte`'s tab-id set once A2 lands `phases_in_force`. A2 should not ship without at least (b), since it is the direct mechanism the row-20(c) fix (§ 6) depends on being kept honest. |
| 4 | Two "profile-only" sites (`life_stage.rs`, `validation/life_stage.rs`, and their UI mirror `LifeStagePanel.svelte`) are the **one place** where using the shared `is_hermetically_trained()` helper would be a regression, not a simplification. | Easy to "clean up" mechanically during a later refactor by someone who does not know the D56 exemption exists, silently re-imposing `minimum_abilities` on a truncated apprenticeship. | A1 should leave an explicit doc comment at each of the three sites citing D56 and this file, not just a code comment saying "profile-only, don't touch." |
| 5 | `App.svelte` row 20(c)'s Possessions/House conflation is a **real, if currently unreachable, bug**: nothing today builds a trained-non-Order character who reaches the UI, so it has never manifested. | Once the Abandoned Apprentice ships (end of A2), it manifests immediately as either a missing Possessions tab or a spurious House tab. | Resolved by A2's data-level fix (§ 6), not by A1; A1 should not attempt a UI-only patch that A2 will then redo. |
| 6 | § 8 row 12c in `measurements.md` names `ruleset_io.rs` where the tree now has `effective_dto.rs` (§ 4's correction). | A1 (or whoever consumes the measurement) could go looking in the wrong file and either miss the two sites or falsely conclude row 12c overcounted. | This design note's § 4 correction is the fix; propagate it into `measurements.md` in the same pass that implements A1, or file it as a one-line follow-up. |

### Notes for E1

E1 (D38's character-type `Prereq`, F-553) is about to add a *third* kind of
type-naming variant — D38 itself flags `virtue.magical_mount`'s "companion or
**magus-level character**" as naming two audiences and warns "design against
both, not against the first." Before E1 adds a dedicated
`Prereq::CharacterType(Id)`-shaped variant, it should check whether
`Prereq::OrderMember` (this design) already **is** the right encoding for
"magus-level" in D38/F-553's specific sentence — Order membership is,
structurally, the one fact that is true of a magus and (per D56) not true of
a mythic companion or an Abandoned Apprentice companion, which may be exactly
what "magus-level" is reaching for, or may not (F-553's own reading is stated
as undecided). Reusing `OrderMember` where it fits avoids a third spelling
for what might be the same underlying question `Prereq::Any([OrderMember,
CharacterType(mythic_companion)])` could already answer; inventing a new
variant first and discovering the overlap after is the "third incompatible
spelling" pattern D13/D17 already warn against elsewhere in this note.

**Redcap is a counter-example to reusing an unqualified `OrderMember` here,
and E1 must weigh it before reusing the encoding above.** `virtue.redcap`
(ArMDE:4844) is a full Order member with no Hermetic training and, per
"Notes for D2/X5" above, is a companion — exactly the shape D38/F-553's
"magus-level character" sentence is *not* trying to reach (a Redcap is not
what ArMDE:4375 means by it). If `order_member` is ever widened past
profile-only (the open risk those notes record), a bare
`Prereq::OrderMember` would then also admit a Redcap wherever E1 used it for
"magus-level" — silently, since nothing about the Prereq's name signals the
narrower intent. E1 should therefore pair it with an explicit exclusion (or
prefer `Prereq::Any([OrderMember, CharacterType(mythic_companion)])` only
once it has independently confirmed a Redcap must be excluded from whatever
"magus-level" is gating) rather than reuse `OrderMember` bare on the
assumption that today's profile-only scoping makes the question moot forever.

## Citation guard check

```
cargo test -p arm-rules --test rulebook_citations 2>&1 | grep -E "test result|FAILED"
```

Output:

```
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 52.94s
```

This file lives at `docs/vf-audit/design-a0-is-magus-split.md`, one directory
below the guard's scanned root (`docs_markdown_files()` reads `docs/`
non-recursively, per D1c — `docs/vf-audit/` is not walked), so the guard does
not scan it either way; the pass above is the guard's existing, unaffected
baseline. Most `ArMDE:` citations here reuse line ranges already stated and
signed off in `decisions.md` D12/D24/D56 (ArMDE:2435, 2465, 2471, 2816, 2840,
2853-2861, 2870, 2880, 4375, 5641-5650) rather than re-deriving them
independently. The Redcap citations added for the "Notes for D2/X5" and
"Notes for E1" sections (ArMDE:4319-4326, 4321, 4323, 4842-4851, 4844) are new
to this note and were independently verified against
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` by reading
those exact ranges before citing them (the `#### Lone Redcap` and `#### Redcap`
headings land at :4319 and :4842 respectively) — this design does not
re-litigate settled rulings, only sequences their implementation.
