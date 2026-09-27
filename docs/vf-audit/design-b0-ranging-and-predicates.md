# B0 — ranging and predicates: design for D21/D23/D33/D40/D41 (Group B)

**Slice.** `phase-2-plan.md` Group B, B0. Design only, no code, no data. Reviewed
next by the plan-reviewer, then the architect, before B1 starts.

**Mandate.** Five findings/rulings all extend `Prereq`/`Effect`/`PointItem` in
the same family — "range over more than one id" (D21), "exclude by property,
not by id" (D23/D33), "fire only when a parameter holds a value" (Q-51),
"name a fixed Ability for a roll modifier" (F-489). Designed together so
B1–B5 produce one coherent addition, not five that don't compose — and so it
composes with C0/C0b's already-landed `ParamGate`/`AbilityRef`/`CategoryRef`/
`ItemPredicate` rather than duplicating them.

---

## 0. What already exists (read before designing on top of it)

| Piece | Where | Shape |
|---|---|---|
| `Prereq` | `types.rs::Prereq` | `All, Any, Nor, Has(Id), House(Id), AbilityMin{ability,score}, ArtMin{art,score}, HermeticallyTrained, OrderMember, IsCompanion` — adjacently tagged (`tag="kind", content="value"`), exhaustive `match` by design |
| `PrereqCtx` | `validation/prereq.rs` | `present_ids` (bought ++ granted, what `Has` tests), `trained`/`order`/`is_companion` (profile flags), `house`, `ability_scores`, `art_scores` — built once per validation pass |
| `advisory_prerequisites` | `types.rs::PointItem` (F-550/Q8, **landed**) | sibling field to `prerequisites`, same `Prereq` tree, `Tri::False` → `CODE_ADVISORY_PREREQ_NOT_MET` (warning) instead of `CODE_PREREQ_NOT_MET` (error); `Tri::Unknown` stays silent. **Deliberately not a wrapper `Prereq` variant** — so a new hard-tree `Prereq` variant needs no UI parity update for the advisory tree, and the two trees never interact under `All`/`Any`/`Nor` |
| `AbilityCategory` | `ability.rs` | closed 5-value enum: `General, Academic, Arcane, Martial, Supernatural` (`AbilityCategory::ALL`) |
| `PointItem::categories` | `types.rs` | `Vec<String>` — **free-form**, no closed registry (Virtue/Flaw categories: `hermetic`, `personality`, `social_status`, `story`, `general`, `supernatural`, `special`, …). Multi-category items resolve to the ONE category in force via `taken_as`/`categories_for` |
| `CategoryCap` | `types.rs`, on `EntityTypeProfile::virtue_category_caps`/`flaw_category_caps` | `{ category: String, max: u8, major_only: bool, hard: bool }` — **ceiling only**, `hard` toggles error vs warning |
| `CategoryRule` | `types.rs`, on `EntityTypeProfile::permitted_categories`/`forbidden_categories` | bare slug or `{ category, when: Prereq }` — **profile-level**, not per-selection; `validate_permitted_categories`/`validate_forbidden_categories` (`validation/selections.rs`) iterate **bought** `entity.selections` only |
| `ItemPredicate` | `docs/vf-audit/design-c0-parameter-model.md` § 7 (**designed, NOT built — verified**: `grep -rn "ItemPredicate" crates/arm-rules/src` returns zero hits; `exclude_if` appears only in a `types.rs:809` doc comment stating it is explicitly *not yet* routed. C2 is gated on B3/D33, so no C-slice has landed this despite the plan's "Group C done" status line) | `Trained` (D12's classification, blocked on X3 — and `PointItem` has no `trained` flag today either), `GrantsReputation` (derivable: `item.effects.iter().any(\|e\| matches!(e, Effect::GrantsReputation{..}))`) — **neither variant, the enum itself, `ParameterDef::exclude_if`, nor its `param_value_resolves` consumer exist in the tree; B3 builds all of it**, plus B0's own `PointItem`-level consumer (§ 2) |
| `Effect` (TS) | `ui/src/lib/types.ts:136-213` | **Real, live, and reachable** — a 32-member discriminated union typing `PointItem.effects?: Effect[]` (`types.ts:355`), with single-variant (non-exhaustive) consumers at `derive.ts:754` and `AbilityTab.svelte:67`. **Corrects an inherited premise**: `design-c0-parameter-model.md` § 10.1 argues no `Effect` TS enum exists to drift, and B0's first draft repeated that uncritically. It is a *curated subset* of Rust's `Effect` (omits e.g. `AbilityAuthorization`, `RestrictedAbilityXp`, `ScaledRestrictedAbilityXp`) — members are added only when a UI consumer needs to read that variant — so parity with Rust means **TS ⊆ Rust**, never full set equality the way `Prereq`'s mirror is |
| `ParamGate` | C0 § 1, **landed** (C1) | `{ param: String, equals: Id }` — activates an `AbilityRef`/`CategoryRef`/`AbilityBonusGated` entry only when the OWNING selection's own param equals a literal |
| D2 (**landed**) | `decisions.md` | grant-aware validators are the house style: read **effective** (bought ++ granted) selections, not bought-only, once a passage's own wording doesn't hinge on the *act* of buying. Q3 already retrofitted the two validators D2 named and left "B1 comment at both sites" as a forward marker for this note |
| Reachability trap (F-466, B15) | `decisions.md` D21/D23 | `validate_incompatibilities` (and, per this note's own reading, `validate_forbidden_categories`) iterate **bought** selections on both sides — a category/id prohibition that must also catch a *granted* item needs to read effective selections, not `Prereq::Nor` gymnastics (D2's route, not a new `Prereq` variant) |
| `SCHEMA_VERSION` | 19 (C5a landed, C0b→C5c) | Confirmed in `migration.rs`; `Selection::params` is `BTreeMap<String, SelectionParamValue>` (`Single(Id) \| Multi(BTreeSet<Id>)`) |

**Ability id check (verified against `rules/core/abilities.json`)**, load-bearing
for B1's Feral item below: `ability.dead_language` is `category: academic`
(gated — already inaccessible to a Flaw-only character), `ability.living_language`
is `category: general` (ungated — the one that matters). The nine "wilds"
Abilities (Area Lore, Animal Handling, Athletics, Awareness, Brawl, Hunt,
Stealth, Survival, Swim) are **all** `general`.

---

## 1. Findings and rulings, per slice

### B1 — D21 category prohibition (F-355, F-542, F-511), category prereq (F-502, F-427), D40's feral prohibitions

| Finding | Entry | Passage (verified) | Gap | Shape chosen |
|---|---|---|---|---|
| F-355 | `flaw.ability_block` | ArMDE:5651-5654: *"You are completely unable to learn a certain class of Abilities... This may be Martial Abilities, or a more limited set of the others... It must be possible for your character to learn the abilities in question in the absence of this Flaw... You may only take this Flaw once."* | no *negative* authorization over an **Ability category** | new `Effect::ForbidsAbilityCategory { category: AbilityCategory }` |
| F-542 (clause 1) | `flaw.weak_personality` | ArMDE:7076-7079: *"...all Personality Traits must be between +1 and -1... The character may have **no other Personality Flaws** or Virtues or Flaws that grant Personality Traits..."* | no *negative* authorization over a **V/F category** (`personality`) — the Ability-axis twin of F-355 (B19's consolidation note) | new `Effect::ForbidsItemCategory { category: String }` |
| F-542 (clause 2) | `flaw.weak_personality` | same passage, second half: *"...or Virtues or Flaws that **grant** Personality Traits"* | a **predicate**, not a category — any item (regardless of its own category) that grants a Personality Trait is excluded too; the "four legal-today pairs" B19 found are exactly items outside `personality` that grant one | reuses D23's `ItemPredicate`, new variant `GrantsPersonalityTrait` (derivable, no new data — same shape as `GrantsReputation`); consumed via B0's own `excluded_if_holds` (§ 2) |
| F-511 | `flaw.sheltered_upbringing` | ArMDE:6721-6724: *"You may not take Bargain, Charm, Etiquette, Folk Ken, Guile, Intrigue, or Leadership as beginning Abilities, but you may learn them in play."* | an **id-list** negative authorization — a third sub-shape, distinct from F-355/F-542's category one (corrections.md's own note) | new `Effect::ForbidsAbilities { abilities: BTreeSet<Id> }` |
| F-502 | `flaw.rector` | ArMDE:6671-6674: *"The character must have a Social Status Virtue dictating his place within the university."* | `Prereq` cannot range over a **category** (99 `social_status` entries — an enumerated `Prereq::Any` would freeze a catalogue count into code, against CLAUDE.md) | new `Prereq::HasCategory(String)` |
| F-427 | ArMDE:2816 | *"All characters **must take one** Social Status..."* | not a per-item prereq at all — a profile-wide **floor**, which `CategoryCap` cannot express (ceiling only) | `CategoryCap` gains `min`/`min_hard` (additive) — mechanism only; the `social_status` row with `min: 1` is **D41/B2's data**, and the general floor mechanism is what B1 owes |
| D40 residual (feral) | `flaw.feral_upbringing` | ArMDE:6110-6113 (already fetched for D40/F-428): *"You may only choose beginning Abilities that you could have learned in the wilds. In particular, you may not start with a score in a Language."* | **not** an XP rule (D40 says so explicitly) — an authorization restriction, and a novel one: it **narrows** the normally-open `general` category down to a named list, rather than adding or forbidding | new `Effect::RestrictsAbilityCategoryToAbilities { category: AbilityCategory, allowed: BTreeSet<Id> }` — flagged as architecturally new (§ 9 open question); **the "Language" sentence is very likely subsumed by the whitelist** (`ability.living_language` is `general` and not among the nine, so the whitelist alone already excludes it) — verify at implementation rather than also encoding a redundant `ForbidsAbilities` for `ability.dead_language`/`ability.living_language` |

**Reconciling "one mechanism" (corrections.md § 3.15a) with "two variants" (this note).** The consolidation note says *"one `Effect` variant carrying a category-scoped prohibition closes both [F-355, F-542]."* B0 ships two — `ForbidsAbilityCategory { category: AbilityCategory }` and `ForbidsItemCategory { category: String }` — and does so deliberately: `AbilityCategory` is a closed 5-value enum and `PointItem::categories` is free-form `Vec<String>` (§ 0), so one untagged sum type across both would be genuinely ambiguous at the wire (both domains legally contain the bare string `"general"`, so an untagged deserializer cannot tell which axis a value belongs to without a discriminant — which is itself a second variant in disguise). **"One mechanism, not one variant"** is the reading this note stands behind: one concept (a per-selection category-scoped prohibition), realized as a "Foo"/"Foo"-for-the-other-domain sibling pair, on the same precedent as every other Ability-axis/domain-specific pair already in `Effect` (`AbilityScoreGrant`/`CharacteristicScoreDelta`, not one type for both). Not a literal reading of corrections.md's wording, and stated here so the gap is closed by design rather than by omission.

### B2 — D41 minimum + second-status warning; the second-guild-status requirement D41 leaves open

Pure **data + validator wiring** on top of B1's `CategoryCap.min`/`min_hard` —
no new type. D41 (`decisions.md`) already rules the shape:

| Clause | ArMDE:2816 text | `CategoryCap` row |
|---|---|---|
| Floor | *"must take one"* | `{ category: "social_status", min: Some(1), min_hard: true }` |
| Ceiling | a second is legal only if the entries say so; D41: *"no per-entry flag... a second Social Status raises a warning, never an error"* | `{ category: "social_status", max: Some(1), hard: false }` — the **existing** ceiling machinery, unchanged, just given a `false` hard flag and a message that names the paired entries |

D41 explicitly rules out a `compatible_with` list (would model none of ArMDE:4325/4614/4441's three cases and "would look authoritative while being a guess"). **ArMDE:4441's *requirement* of a second guild status is a separate obligation D41 does not discharge** — it is the mirror problem (a **floor** raised by holding a *specific* entry, not the profile's own baseline floor) and is data-only once B1's `Prereq::HasCategory`/`CategoryCap.min` exist.

**Carrier identified (verified directly): `virtue.male_guild_sponsor`.** ArMDE:4439-4442: *"#### Male Guild Sponsor — Free, Social Status — ...The character must select a **separate guild Social Status Virtue** as well as this free Virtue to represent her status in the guild system..."* — an exact match for D41's own quoted text. Already in the catalogue: `rules/core/virtues_flaws.json:5198` (`"id": "virtue.male_guild_sponsor"`, `"magnitude": "free"`, `"categories": ["social_status"]`, `"classification": "narrative"`), zero `prerequisites` today. **Folded into B2's data scope** (was left as an open question in an earlier draft; a two-minute lookup answers it, so it does not wait for X5):

```json
{ "id": "virtue.male_guild_sponsor", "prerequisites": { "kind": "has_category", "value": "social_status" } }
```

Whether this needs to name a *second, distinct* Social Status specifically (rather than "any", which the character already has by ArMDE:2816's own floor) is B2's remaining data call, not an identification gap — `HasCategory` alone cannot distinguish "holds one" from "holds a second, different one" without also excluding the entry's own row, so B2 should read `flaw.rector`'s worked example (§ 1) for the same-shape precedent before deciding whether `uncomputed_rule` text is the honest fallback here.

### B3 — D23 predicates (F-526), D33's domain predicate, F-334's parameter-comparing prereq

**Correction (plan-review finding #1, BLOCKER): C0 *designed* `ItemPredicate`/`exclude_if`, it did not *build* either.** B0's first draft said "already fully specified by C0 § 7 — B3 implements it" and "already landed (C0/D33)" — both wrong against the tree (`grep -rn "ItemPredicate" crates/arm-rules/src` is empty; `types.rs:809`'s doc comment says `exclude_if` is explicitly "out of this scope" so far). C0 § 7 is an **interface design**, not code: no C-slice (C0b–C5c) built the enum, the `ParameterDef.exclude_if` field, its `param_value_resolves` consumer arm, or its integrity check, despite the plan's "Group C done" status line. **B3's scope is therefore larger than the first draft stated** — it must build, in order:

1. The `ItemPredicate` enum itself (`Trained`, `GrantsReputation` — C0 § 7's shapes, unbuilt) plus this note's own `GrantsPersonalityTrait` (§ 2).
2. `ParameterDef::exclude_if: Option<ItemPredicate>` (D33's consumer) and its `param_value_resolves` resolution arm (C0 § 7).
3. `PointItem::excluded_if_holds: Vec<ItemPredicate>` (D23's consumer, B0's own — § 2) and its grant-aware validator (§ 4).
4. Both fields' load-time integrity checks (§ 5) — `exclude_if`/`excluded_if_holds` must reject a non-`item` domain and a `Trained` reference ahead of D12's classification pass, mirroring C0 § 9's own table.
5. Red tests for all of the above, not just for B0's `excluded_if_holds` consumer — including `flaw.flawed_powers`' own D33 case (a hand-authored entry with `exclude_if: "trained"` must resolve `virtue.diedne_magic`... no, must resolve a non-Hermetic-only Flaw and refuse `flaw.deficient_technique`), which today fails to **parse at all**, not merely to validate.

**Sizing (plan-review's estimation-adjustment ask): M still holds.** The added scope is one small enum (3 variants), one `Option<ItemPredicate>` field + one resolution arm, one `Vec<ItemPredicate>` field + one grant-aware validator, and two integrity checks — comparable in shape and size to B1's four `Effect` variants (also M), not a step up to L.

| Finding | Entry | Passage (verified) | Gap | Shape chosen |
|---|---|---|---|---|
| F-526 (Q-137) | `flaw.university_dean` | ArMDE:6923-6926 (already verified in D23, quoted there): *"can not have the Poor Flaw or any other Flaw that grants a Bad Reputation"* | exclusion by **description** (grants a Reputation), not by id — the derivable predicate D23 names | `ItemPredicate::GrantsReputation` (built by B3, not C0 — see above) through the new `excluded_if_holds` field (§ 2); `Poor` itself stays a plain `incompatible_with` id (it does not "grant a Reputation", it's the other named exclusion) |
| D33 domain predicate | `flaw.flawed_powers` | ArMDE:6146-6148 (D33, quoted above) | **Interface designed by C0 § 7, NOT built** (`ParameterDef.exclude_if`, `ItemPredicate::Trained`) | B3 builds the enum + `exclude_if` field + its resolver + integrity check (above), then lands the C0-specified JSON; the `trained` flag itself stays blocked on X3's classification pass (per D23/D33 both) — B3 can build and test the *machinery* against a hand-authored fixture before X3 lands the real flag |
| F-334 | `virtue.true_love_pc` | ArMDE:5175 (already fetched in batch-09/F-334): *"Your True Love is another player character, who must also have this Virtue. True Love is never one-sided."* | see **§ 9, open question** — the original ruling (batch-09's own "D3", D1-D18 era) closed this as **unencodable, full stop**: a cross-*entity* requirement, and no entity in this engine's model ever references a second entity's sheet. The plan's phrase "F-334's parameter-comparing prereq" does not obviously square with that closure | **not designed here** — flagged, not invented (§ 9) |

### B4 — Q-51 parameter-gated effects

| Finding | Entry | Passage (verified) | Gap | Shape chosen |
|---|---|---|---|---|
| Q-51 | `virtue.magical_blood` | ArMDE:4359-4371 (fetched directly): four bloodline variants behind an `enumerated` parameter (F-165); only *Magic Human* states a determinate mechanic — *"The character may increase one of his Characteristics by 1, but not above +3... also has a positive Reputation at level 3 among others of his bloodline"* (ArMDE:4367) | an effect firing **for every copy of the item regardless of the parameter's value** would wrongly apply Magic Human's Characteristic/Reputation grants to Magic Animal/Spirit/Thing too (F-45/F-20 shape, rated HIGH elsewhere) — the missing machinery is *"this effect applies only when parameter X holds value Y"*, **gated**, not **valued** (`characteristic_score_delta_param` already is the latter) | an optional `gate: Option<ParamGate>` field added directly to the two concrete carriers, `Effect::CharacteristicScoreDeltaParam` and `Effect::GrantsReputation` (§ 2) — **revised in Revision 3**, see below |
| Q-51's second caller | `flaw.flawed_powers` (D33) | — | D33's imported Flaw's effects "apply through a parameter" too | same `gate` field, on whichever effect variant D33's data ends up naming — **not** built alone, per Q-51's own text ("design the gate with D33, D21 and D23 rather than alone") |

**Wrapper rejected, field-per-variant adopted (Revision 3, architect finding #1).**
The first two drafts of this note added a generic `Effect::Gated { gate, effect: Box<GatedEffect> }` wrapper. The architect review found this a second, structurally distinct gating idiom beside the one C0/C1 already established — a gate embedded directly on the item that needs conditional activation (`AbilityRef::Scoped.gate`, `CategoryRef::Scoped.gate`, `AbilityBonusGated`'s per-target gate). Both of B4's concrete needs are already-existing `Effect` variants with no field-name collision risk, so adding `gate: Option<ParamGate>` straight to each does the identical job with **zero** new variant, no `Box`, no new closed enum, no new macro-tail row, and no §3b re-walk at all. The wrapper's stated justification — reusability for "any inner effect" — was undercut by its own design choice to close `GatedEffect` to exactly two members (Revision 2's own §9 superseded-note already conceded this was not actually generic): CLAUDE.md's YAGNI cuts against the wrapper, not for it, once the "any effect" generality is not real. **Adopted, not merely considered**: this revision replaces the wrapper everywhere it appeared.

Q-51's own ruling is explicit: text for all four Magical Blood variants **until** a parameter-gated effect exists. Once B4 lands the mechanism, `virtue.magical_blood`'s Magic Human clause **may** move from `uncomputed_rule` to a gated `CharacteristicScoreDeltaParam` + a gated `GrantsReputation` — Animal/Spirit/Thing stay `uncomputed_rule` regardless (F-164's "make up your own" reading has no computable content). No X-slice names this data work explicitly; B4 (sized M) should carry it as its own worked example rather than leave it stranded.

### B5 — F-489, an `Effect` naming an Ability for a roll modifier (the shape of 42 entries)

| Finding | Entry (worked example) | Passage (verified) | Gap |
|---|---|---|---|
| F-489 | `flaw.poor_hearing` | ArMDE:6614-6617: *"Subtract 3 from rolls involving hearing..."* | **no `Effect` variant names a fixed Ability for a roll-only modifier.** The existing `AbilityRollMod { param: String, amount: i8 }` targets the Ability the OWNING selection's own free-text param names (Academic Concentration's chosen field of study) — it cannot express "always Awareness, -3", because there is no parameter to read the target from; the target is fixed by the Flaw itself |

**Naming collision, flagged rather than silently resolved.** Every other
fixed/parameterized pair in this file follows "base name = fixed target,
`...Param` = parameter-relative target" (`AbilityScoreGrant`/`AbilityScoreGrantParam`,
`CharacteristicScoreDelta`/`CharacteristicScoreDeltaParam`). `AbilityRollMod`
already exists and is the **parameter-relative** one, so it does not follow that
convention — it predates it. Recommended fix, not decided here (B5's call, no
Norbert decision needed): **rename** the existing variant's wire tag from
`ability_roll_mod` to `ability_roll_mod_param` (ruleset-JSON-only, one known
carrier — Academic Concentration — updated in the same commit; **no**
`SCHEMA_VERSION` bump, `Effect` lives in `rules/core/`, not in saves), and give
the **new**, fixed-target, 42-entry-carrier shape the plain name:

```rust
AbilityRollMod { ability: Id, amount: i8 }        // NEW meaning: fixed target
AbilityRollModParam { param: String, amount: i8 } // RENAMED from today's AbilityRollMod
```

The Ability each of the 42 entries targets (Poor Hearing → Awareness, or
whichever Ability governs a hearing-dependent roll in this ruleset) is **data**,
authored per entry in Phase 3 (X7b-e names "row 42" as its own item-by-item
pass) — B5 only builds the Effect shape and its one worked example.

---

## 2. New/changed types

```rust
// B1 — F-355 / D21 Ability-axis twin
AbilityCategoryBlock... // see below, folded into Effect
Effect::ForbidsAbilityCategory { category: AbilityCategory }

// B1 — F-542 clause 1 / D21 V/F-axis twin
Effect::ForbidsItemCategory { category: String }

// B1 — F-511 (+ available for Feral's "Language" clause if the whitelist below
// turns out NOT to subsume it)
Effect::ForbidsAbilities { abilities: BTreeSet<Id> }

// B1 — D40 residual (Feral Upbringing); flagged novel, see § 9
Effect::RestrictsAbilityCategoryToAbilities {
    category: AbilityCategory,
    allowed: BTreeSet<Id>,
}

// B1 — F-502 / F-427's Prereq half
enum Prereq {
    ...
    /// The entity must hold (bought or granted) at least one item whose
    /// in-force category is `String` — the category-ranging twin of `Has`,
    /// evaluated the same grant-aware way (D21; `flaw.rector`'s "must have a
    /// Social Status Virtue").
    HasCategory(String),
}

// B1 — F-427's floor half, additive on the EXISTING type
struct CategoryCap {
    category: String,
    max: u8,               // unchanged
    major_only: bool,      // unchanged
    hard: bool,            // unchanged — governs the CEILING only
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min: Option<u8>,       // NEW (D21/F-427): "must take one" needs a floor,
                           // which a ceiling-only cap cannot express
    #[serde(default, skip_serializing_if = "is_false")]
    min_hard: bool,        // NEW: independent hardness from the ceiling's
                           // `hard` — D41 needs floor=hard, ceiling=soft on
                           // the SAME row (Social Status)
}

// B3 — D23's B0-owned consumer, alongside `incompatible_with`
struct PointItem {
    ...
    incompatible_with: BTreeSet<Id>,       // unchanged, symmetric
    /// One-directional (unlike `incompatible_with`): this item is illegal
    /// while ANY effective (bought or granted) OTHER item satisfies one of
    /// these predicates. `university_dean` names `GrantsReputation`; the 16
    /// Reputation-granting Flaws need no reciprocal declaration (D23/Q-137).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    excluded_if_holds: Vec<ItemPredicate>,
}

// B3 — the enum C0 § 7 DESIGNED but did not build (plan-review finding #1);
// B3 builds all three variants, the type itself, `ParameterDef::exclude_if`,
// and `PointItem::excluded_if_holds` in the same slice.
enum ItemPredicate {
    Trained,            // shape from C0 § 7; blocked on X3's classification pass
    GrantsReputation,   // shape from C0 § 7; derivable today, no blocker
    /// Carries a `GrantsReputation`-shaped effect for Personality Traits
    /// (F-542 clause 2: "Virtues or Flaws that grant Personality Traits").
    /// Needs the same derivable-predicate treatment `GrantsReputation` gets —
    /// no new data, scanned off existing effects once the granting shape
    /// itself has one (see the open question in § 9 on whether such an
    /// Effect exists yet).
    GrantsPersonalityTrait,
}

// B4 — Q-51's parameter gate, REVISED in Revision 3 (architect finding #1):
// no wrapper variant. An optional `gate` field added directly to the two
// concrete carriers Q-51 names, on the SAME embedded-gate idiom C0/C1 already
// established for `AbilityRef::Scoped`/`CategoryRef::Scoped`/`AbilityBonusGated`
// — one gating idiom in the codebase, not two.
enum Effect {
    ...
    CharacteristicScoreDeltaParam {
        param: String,
        amount: i8,
        /// This grant applies only when the OWNING selection's own gate
        /// holds (Magical Blood: only for the chosen bloodline). Absent for
        /// every OTHER existing carrier — additive, byte-compatible, no
        /// `SCHEMA_VERSION` bump (`Effect` lives in ruleset JSON anyway; see
        /// § 6).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
    GrantsReputation {
        kind: Option<ReputationType>,
        score: u8,
        max_score: Option<u8>,
        /// Same meaning as above.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
}
```

**Rejected: the generic wrapper.** Revisions 1–2 had a separate
`Effect::Gated { gate: ParamGate, effect: Box<GatedEffect> }` wrapping a
closed two-member inner enum. Dropped per the architect's review: Q-51 names
exactly two concrete carriers, both already-existing `Effect` variants with
no field-name collision risk, so wrapping bought nothing a direct field
doesn't already give — it added a variant, a `Box`, a second closed enum, and
an entire new §3b exhaustive-match re-walk for a "generality" the design's
own closure to two members admitted was never real. CLAUDE.md's YAGNI ("build
what is needed now, nothing speculative") argues directly against the
wrapper once that's seen. The parse-time-recursion discussion the wrapper
needed (Revision 2, §5) no longer applies at all: there is nothing to nest —
a `gate` field is a leaf, not a box.

**`ParamGate::holds` visibility (architect finding #2).** `fn holds`
(`types.rs:1028`) carries no visibility modifier today — it is private to
the `types` module, reachable only via the `pub(crate)` wrappers
`AbilityRef::active_for`/`CategoryRef::active_for` defined in that SAME
module. B4's two consumer arms live in `effective/characteristic.rs` and
`effective/reputation_and_caps.rs` — a different module — so calling
`gate.holds(selection)` directly will not compile as written. **B4 makes
`holds` `pub(crate)`** (matching `ParamGate`'s own struct visibility and the
precedent every other `pub(crate)` method in this file already sets, rather
than adding a third wrapper for a field two external call sites need
directly).

```rust
// B5 — F-489 (see § 1 for the naming-collision recommendation)
enum Effect {
    ...
    AbilityRollMod { ability: Id, amount: i8 },        // NEW meaning
    AbilityRollModParam { param: String, amount: i8 }, // renamed from today's AbilityRollMod
}
```

### JSON examples

```json
// flaw.ability_block (F-355) — one worked category (Martial), data authors the rest
{ "id": "flaw.ability_block", "effects": [
  { "type": "forbids_ability_category", "category": "martial" }
] }

// flaw.weak_personality (F-542) — both clauses on one entry
{ "id": "flaw.weak_personality", "effects": [
  { "type": "forbids_item_category", "category": "personality" }
], "excluded_if_holds": ["grants_personality_trait"] }

// flaw.sheltered_upbringing (F-511)
{ "id": "flaw.sheltered_upbringing", "effects": [
  { "type": "forbids_abilities", "abilities": [
    "ability.bargain", "ability.charm", "ability.etiquette", "ability.folk_ken",
    "ability.guile", "ability.intrigue", "ability.leadership"
  ] }
] }

// flaw.feral_upbringing (D40 residual — the authorization half only; the XP
// half is D40/F-428/group D's `replaces_life_stage_xp`, not this note's)
{ "id": "flaw.feral_upbringing", "effects": [
  { "type": "restricts_ability_category_to_abilities", "category": "general", "allowed": [
    "ability.area_lore", "ability.animal_handling", "ability.athletics",
    "ability.awareness", "ability.brawl", "ability.hunt", "ability.stealth",
    "ability.survival", "ability.swim"
  ] }
] }

// flaw.rector (F-502)
{ "id": "flaw.rector", "prerequisites": { "kind": "has_category", "value": "social_status" } }

// character_types.json — companion/magus/etc. budget (F-427/D41, B2's data)
{ "virtue_category_caps": [
  { "category": "social_status", "min": 1, "min_hard": true, "max": 1, "hard": false }
] }

// flaw.university_dean (F-526/Q-137)
{ "id": "flaw.university_dean", "incompatible_with": ["flaw.poor"],
  "excluded_if_holds": ["grants_reputation"] }

// virtue.magical_blood (Q-51, Magic Human clause — worked example)
{ "id": "virtue.magical_blood", "parameters": [
  { "key": "bloodline", "type": "ref", "domain": "enumerated",
    "values": ["magic_animal", "magic_human", "magic_spirit", "magic_thing"] }
], "effects": [
  { "type": "characteristic_score_delta_param", "param": "characteristic", "amount": 1,
    "gate": { "param": "bloodline", "equals": "magic_human" } },
  { "type": "grants_reputation", "score": 3,
    "gate": { "param": "bloodline", "equals": "magic_human" } }
] }

// flaw.poor_hearing (F-489)
{ "id": "flaw.poor_hearing", "effects": [
  { "type": "ability_roll_mod", "ability": "ability.awareness", "amount": -3 }
] }
```

### `ui/src/lib/types.ts::Effect` — the TS union every new/renamed variant touches (plan-review finding #2)

Corrected premise (§ 0): this union is real and reachable, so every B-group
variant that a UI surface can encounter needs a TS member, added in the SAME
slice as the Rust variant:

```ts
// B1
| { type: 'forbids_ability_category'; category: string }
| { type: 'forbids_item_category'; category: string }
| { type: 'forbids_abilities'; abilities: string[] }
| { type: 'restricts_ability_category_to_abilities'; category: string; allowed: string[] }
// B4 (Revision 3) — WIDENS two EXISTING members (types.ts:138, :167), no new
// union member at all: an optional `gate` field on each, mirroring § 2's Rust
// field-per-variant revision
| { type: 'characteristic_score_delta_param'; param: string; amount: number; gate?: { param: string; equals: string } }
| { type: 'grants_reputation'; kind?: ReputationType; score: number; max_score?: number; gate?: { param: string; equals: string } }
// B5 — replaces today's single `ability_roll_mod` line (types.ts:197)
| { type: 'ability_roll_mod'; ability: string; amount: number }       // NEW meaning
| { type: 'ability_roll_mod_param'; param: string; amount: number }   // renamed from today's entry
```

**A new parity test, `ui/src/lib/effect-parity.test.ts`, lands in B1 before B1's first new variant** — otherwise B1 is the slice that reintroduces exactly the drift finding #2 caught. Unlike `prereq-parity.test.ts` (full set equality — `Prereq` has no curated subset), this one asserts the **correct** relationship for a deliberately partial mirror: every `type` literal in the TS `Effect` union resolves to a real Rust `Effect` variant tag (**TS ⊆ Rust**, checked by parsing both sources exactly as `prereq-parity.test.ts` already does), never the reverse — Rust is allowed variants the UI never renders. This catches the one drift class that matters here: a TS entry left stale after a Rust rename (exactly B5's `ability_roll_mod`/`ability_roll_mod_param` swap) or typo'd after a copy-paste, which today nothing would flag since neither side's match is exhaustive.

---

## 3. Exhaustive match sites

### 3a. `Prereq::HasCategory` — every site that must handle it

`Prereq` is adjacently tagged and matched exhaustively in exactly the sites
`prereq-parity.test.ts` already polices plus the Rust ones it mirrors — a new
variant is a compile error at every one of these until touched:

| # | Site | Verdict |
|---|---|---|
| 1 | `types.rs::Prereq::house_only_value` (+ `fold_house_only`) | **real, but a no-op answer**: `HasCategory` is outside this function's remit exactly like `Has`/`AbilityMin`/etc. — undecided (`None`), never excludes an item from the open-grant menu. Add to the `Prereq::Has(_) \| Prereq::AbilityMin{..} \| ... => None` tail |
| 2 | `validation/prereq.rs::evaluate_prereq` | **real**: new arm reading `ctx.held_categories.contains(category.as_str())` (§ 4) — `Tri::True`/`Tri::False`, never `Unknown` (an item's own category is static, not conditional) |
| 3 | `ruleset/integrity.rs::validate_prereq_refs` | **real**: new arm checking the named category string is used by **at least one** point item in the catalogue (catches a typo'd category the same way `Has`/`House` catch a dangling id) — free-form strings have no closed registry to check against otherwise |
| 4 | `ui/src/lib/types.ts::Prereq` (TS union) | **real**: `| { kind: 'has_category'; value: string }` |
| 5 | `ui/src/lib/derive.ts::houseOnlyValue` | **real, no-op answer**: add `case 'has_category': return undefined;` to the exhaustive switch, on the same reasoning as site 1 |
| 6 | `ui/src/lib/prereq-parity.test.ts` | **no code change** — it diffs sites 1/4/5 against each other as text; it fails loudly if any of the three is missed, which is its job |

### 3b. `Effect` sites — B1's four new variants, plus B4's revised (no-new-variant) scope, plus B5's rename

Reusing C0's own 14-site enumeration (`crates/arm-rules/src/effective.rs::irrelevant_effect_variants!`
macro plus 13 hand-written exhaustive/real-logic sites), **plus one Rust site
C0 did not enumerate and one TS site neither C0 nor B0's first draft
enumerated** (rows 15/16 below). None of B1's four new variants
(`ForbidsAbilityCategory`, `ForbidsItemCategory`, `ForbidsAbilities`,
`RestrictsAbilityCategoryToAbilities`) touches a computed score, XP pool, or
spell total — they are flag-shaped, on the exact precedent
`Effect::ForbidsAbilitySpecialties`/`Effect::RigidMagic`-class effects already
set (consumed only by a dedicated validator). **B4 no longer belongs in this
table at all** (Revision 3): since it adds no new `Effect` variant, it never
touches an exhaustive match's arm COUNT — see the separate, much shorter
site list below instead.

| # | Site | B1's four (forbid/restrict) | `AbilityRollMod` (B5, renamed pair) |
|---|---|---|---|
| 1 | `effective.rs` macro tail | no-op — add all four | no-op — add both names to the tail (roll mods are surfaced-only, like today's single `AbilityRollMod`) |
| 2 | `effective/ability.rs::ability_bonus` | no-op (via macro) | no-op (via macro) |
| 3 | `effective/art.rs::art_bonus` | no-op | no-op |
| 4 | `effective/art.rs::deficient_arts` | no-op | no-op |
| 5 | `effective/characteristic.rs::characteristic_score_bonus` | no-op | no-op |
| 6 | `effective/xp.rs::ability_affinity` | no-op | no-op |
| 7 | `effective/xp.rs::art_affinity` | no-op | no-op |
| 8 | `effective/spell.rs::spell_levels_bonus` | no-op (hand tail) | no-op |
| 9 | `effective/spell.rs::general_xp_bonus` | no-op | no-op |
| 10 | `effective/spell.rs::spell_mastery_advancement_affinity` | no-op | no-op |
| 11 | `derived.rs::in_play_mods` | no-op (creation-time flags, not in-play) | **real** — a roll modifier is exactly this list's business (5i "surfaces it labelled"); add both names |
| 12 | `effective/xp.rs::resolve_ability_refs`/`ability_authorizations` | no-op — these are prohibitions, not authorizations, and stay independent (§ 4) | no-op |
| 13 | `ruleset/integrity.rs::validate_effect_refs` | **real** for all four: `category` (Ability axis) needs no check beyond serde (closed enum); `category` (item axis, `ForbidsItemCategory`) checked like `Prereq::HasCategory`'s new arm (§ 3a site 3); `abilities`/`allowed` members must resolve in the ability catalogue; `allowed` must be a subset of `category`'s actual members (else a name in the list can never apply) | **real**: `ability` (new meaning) must resolve in the ability catalogue; `param` (renamed variant) checked exactly as today's `AbilityRollMod` arm is |
| 14 | `validation/mod.rs::effect_target` | **`Other`** — no-op, matches `AbilityScoreGrant`'s classification: these name fixed ids/categories, never a player-chosen dangling param | `Other` — no-op, same reasoning as B1's four |
| 15 | `ruleset/integrity.rs::validate_item_ratios` (~line 439) | no-op | no-op |
| 16 | `ui/src/lib/types.ts::Effect` (§ 2's TS section) | **real** — 4 new union members | **real** — both renamed/new members |

**Row 15, out of scope, stated rather than silently skipped (plan-review
finding #5).** `validate_item_ratios` is a genuinely 15th `match Effect` site
C0's own 14-site count missed — but it is a deliberately **non-exhaustive**
match (trailing `_`, per its own doc comment: "nothing outside this list is
checked at load"), guarding only the ratio-bearing variants
(`AffinityAbilityCost`/`AffinityArtCost`/`GrantsSpellMastery`/one more). None
of this note's seven new/renamed variants is ratio-shaped, so no action is
owed here — recorded so the re-walk is accurate rather than overstated by one
site.

### 3c. B4's revised site list (Revision 3, architect finding #1) — a widened field, not a new variant

Because `gate: Option<ParamGate>` is added directly to two ALREADY-EXISTING
`Effect` variants, B4 touches no exhaustive-match ARM COUNT anywhere — the
macro tail's `Effect::CharacteristicScoreDeltaParam { .. }` /
`Effect::GrantsReputation { .. }` entries (used at 9 of the 12 numeric-fold
call sites, confirmed by direct grep) stay `{ .. }` and compile unchanged.
Only sites that pattern-match these two variants **by named field, without
`..`** are compiler-forced to change — verified directly, not assumed, by
grepping every `Effect::CharacteristicScoreDeltaParam`/`Effect::GrantsReputation`
occurrence in `crates/arm-rules/src`:

| Site | Today's pattern | Forced? | What changes |
|---|---|---|---|
| `effective/characteristic.rs::characteristic_score_bonus` (~line 135) | `Effect::CharacteristicScoreDeltaParam { param, amount } if selection.params.get(param)... == Some(characteristic)` | **yes** — named fields, no `..` | guard becomes `if gate.as_ref().is_none_or(\|g\| g.holds(selection)) && selection.params.get(param)...` — the ONE real consumer of the Characteristic-delta gate |
| `validation/mod.rs::effect_target` (~line 1195) | `Effect::CharacteristicScoreDeltaParam { param, amount } => EffectTarget::CharacteristicParamDelta { param, amount: *amount }` | **yes** — named fields, no `..` | add `gate: _` (or `..`) to the pattern — the gate carries no dangling-target concern of its own beyond what § 5's integrity check already covers, so this site's classification is unchanged, only its pattern needs to compile |
| `effective/reputation_and_caps.rs::reputation_grants` (~line 38) | `if let Effect::GrantsReputation { kind, score, max_score } = effect` | **yes** — named fields, no `..` | becomes `if let Effect::GrantsReputation { kind, score, max_score, gate } = effect` guarded by `if gate.as_ref().is_none_or(\|g\| g.holds(selection))` — `selection` is already in scope here via the `for_each_effect!` macro this function already uses — the ONE real consumer of the Reputation gate |
| `ruleset/integrity.rs` (~line 2278, `validate_item_ratios`) | `Effect::CharacteristicScoreDeltaParam { param, .. }` | no (already `..`) | **voluntary addition, not compiler-forced**: `validate_effect_refs` (§ 5, not this site) gains a new check resolving `gate.param` on the SAME item, reusing the identical check `AbilityRef`'s gate already has (C0 § 9) — `ParamGate` needs no second integrity rule invented for it |
| every other `{ .. }` site (9 sites: `effective.rs:321,361`; `derived.rs:414,440`; `effective/xp.rs:835,855`; `effective/spell.rs:23,49,89,118,484,509`) | `Effect::CharacteristicScoreDeltaParam { .. }` / `Effect::GrantsReputation { .. }` | no | correctly stay no-ops — these folds (Affinity, spell levels, in-play surfacing, …) have no reason to read a gate that only ever conditions THIS effect's own applicability, already fully resolved by the two real consumers above |

Three sites, one of them voluntary — a small fraction of the 16-row/two-enum
cost the rejected wrapper would have carried into this same section.

---

## 4. Evaluation semantics

**Blocking vs advisory.** `Prereq::HasCategory` sits on whichever tree the
data author puts it on: `prerequisites` (hard — F-502's "must have", stated as
an absolute) or `advisory_prerequisites` (F-550/Q8's hedge carrier — not
needed by any B-group finding today, but available with **zero extra work**
since it is the same `Prereq` tree evaluated twice). The four new `Effect`
forbids are **not** `Prereq` at all and carry no severity choice by
construction — matching `Effect::ForbidsAbilitySpecialties`'s existing
precedent, a forbid raises a **hard error** (`CODE_...`) when violated, full
stop; D41's "warning, never an error" case (a *second* Social Status) is
**not** modelled as a forbid-effect at all — it is the `CategoryCap` ceiling
with `hard: false`, which already has warning semantics built in. **No B-group
finding needs a *soft* forbid-effect**, so none is designed; if one ever does,
it is a `hard: bool` field on the forbid effects themselves, on `CategoryCap`'s
own precedent, not a new mechanism.

**Grant-awareness (D2).** Every new validator this note implies —
`Prereq::HasCategory`'s evaluation (via `PrereqCtx.held_categories`, folded the
same bought-++-granted way `present_ids` already is) and the four forbid/
restrict effects' consumer (a new function, `validate_category_effect_prohibitions`
or equivalent, in `validation/selections.rs` beside `validate_forbidden_categories`)
— reads **effective** selections on both sides: whether the forbidding item is
itself in effect (bought OR granted), and whether the forbidden target
(an Ability score bought, or another V/F selection) is present at all (bought
OR granted). This is D2's fix applied prospectively rather than retrofitted,
per Q3's "B1 comment at both sites" forward marker. `excluded_if_holds` (§ 2)
follows the identical rule: scanned against effective selections, not bought
selections, closing the F-466 trap at design time instead of leaving it for a
later finding.

**Doc-comment cross-references B1 owes, named explicitly (architect findings
#3/#4) so neither pair of same-shaped helpers gets conflated by a future
implementer.** (1) `validate_category_effect_prohibitions` and
`validate_forbidden_categories` (`validation/selections.rs:165`) solve
genuinely different axes — item-authored `Effect` list vs. profile-declared
`CategoryRule` list — which is the right reason to keep them as two
functions, not one; each function's doc comment must say so and name the
other, so "which validator do I extend" is answered in the code, not only in
this note. (2) `PrereqCtx.held_categories` (grant-aware, bought ++ granted)
and the existing `categories_in_force` (`validation/selections.rs:46`,
confirmed **bought-only** — `validate_permitted_categories`/
`validate_forbidden_categories` both iterate `entity.selections` alone) are
same-shaped "what categories does this entity hold" helpers with different
scope. `categories_in_force`'s own doc comment gains one line: "bought-only
by design; see `PrereqCtx.held_categories` for the grant-aware twin `Has`-style
prerequisites need" — otherwise a future reader has no signal that reaching
for the wrong one is even possible.

**`gate.holds(selection)`'s evaluation** (§ 2, § 3c) is one `BTreeMap::get` +
`==` (`ParamGate::holds`, made `pub(crate)` per the architect's finding #2)
guarding the two real consumers directly — no recursion, no wrapper, and no
new control flow beyond the one extra `&&` each of § 3c's two real sites
already needs for its own logic.

**Amendment, 2026-09-27 (B2/ArMDE:4441).** `Prereq::HasCategory`, when
evaluated as an item's OWN `prerequisites`/`advisory_prerequisites`
(`PrereqCtx::evaluate_for_item`), excludes that item's own contributed
categories — a prerequisite states what the REST of the character holds, and
`virtue.male_guild_sponsor` is itself `social_status`, so a self-blind
evaluation would be trivially satisfied by itself and never actually require
the SEPARATE guild status ArMDE:4441 demands. `held_categories` is now keyed
on category → contributing `item_ref`s (not a bare set) so this exclusion is
possible; every OTHER caller (`CategoryRule.when`, a profile-level gate
belonging to no single selection) keeps using the non-excluding
`PrereqCtx::evaluate`.

**Amendment, 2026-09-27 (B2/ArMDE:2816, second note).** ArMDE:2816 reads
"Virtues **or** Flaws", and the catalogue ships Social Status as both kinds
(`virtue.gentleman`; `flaw.outlaw`) — so a Virtue-only count on the
`social_status` `virtue_category_caps` row would wrongly refuse a character
represented solely by a Social Status Flaw. `CategoryCap` gains
`both_kinds: bool` (default `false`, preserving every OTHER shipped cap's
existing kind-scoped behavior — a Major Hermetic Flaw like
`flaw.suppressed_gift` must NOT start counting toward the Major Hermetic
Virtue cap): `true` on this one row makes its count span both Virtues and
Flaws sharing the category, regardless of which array the row itself lives
in.

---

## 5. Load-time integrity checks

Every new field/variant gets a check in `ruleset::integrity`, on the
established pattern (C0 § 9's own table is the precedent to match stylistically):

| Field/variant | Rejected when | Message names |
|---|---|---|
| `Prereq::HasCategory(category)` | `category` is not a category any point item in the catalogue declares | the declaring item id (the Flaw carrying the prereq), the unrecognised category |
| `Effect::ForbidsAbilityCategory.category` | never (closed enum, serde-checked) | — |
| `Effect::ForbidsItemCategory.category` | same as `HasCategory` above | item id, category |
| `Effect::ForbidsAbilities.abilities` member | does not resolve in the ability catalogue | item id, offending ability id |
| `Effect::RestrictsAbilityCategoryToAbilities.allowed` member | does not resolve, **or** its own `Ability.category` is not `category` (a whitelist naming an ability outside its own stated category can never apply — silent dead data otherwise) | item id, offending ability id, its actual category |
| `CategoryCap.min` | `min > max` when both are present (an unsatisfiable range) | item id (well, profile id here), category, min/max |
| `CategoryCap.min`/`min_hard` present with `min` absent | `min_hard` on a cap with no floor is meaningless | profile id, category |
| `PointItem.excluded_if_holds` member | — (closed `ItemPredicate` enum, serde-checked; `Trained` additionally requires D12's classification pass to have run — same precondition C0 § 7 states for `exclude_if`) | — |
| `Effect::CharacteristicScoreDeltaParam.gate.param` / `Effect::GrantsReputation.gate.param` (Revision 3) | dangling, or resolves to a `MultiRef`-typed parameter (one-value-per-read) | item id, effect kind, gate param key |
| `Effect::AbilityRollMod.ability` (new meaning) | does not resolve in the ability catalogue | item id, offending ability id |
| `Effect::AbilityRollModParam.param` (renamed) | unchanged from today's `AbilityRollMod` check | item id, param key |

**`gate.param`'s check reuses C1's `ParamGate` rule verbatim, not a new
invention (Revision 3).** `AbilityRef::Scoped.gate`/`CategoryRef::Scoped.gate`
already carry exactly this check (C0 § 9: "the named `param` is not a key the
SAME item's `parameters` declares" / "is a `MultiRef` param"), landed in C1.
B4's two `gate` fields are validated by the SAME check function, not a
second copy of it — one rule for what a `ParamGate` may point at, regardless
of which field holds it. (Revision 2's parse-time-recursion discussion for
the rejected `Gated` wrapper is removed here — there is nothing to nest, so
the question does not arise for a leaf `Option<ParamGate>` field.)

---

## 6. Saves — schema impact

**No `SCHEMA_VERSION` bump.** Every type touched by this note —
`Prereq`, `Effect`, `PointItem::excluded_if_holds`, `CategoryCap::min`/`min_hard`,
`ItemPredicate` — lives in **ruleset JSON** (`rules/core/*.json`), which is
re-authored wholesale in the same commit as the code that reads it, exactly as
C0 § 1 notes for its own `AbilityRef`/`CategoryRef` change. None of it touches
`Entity`/`Selection`, which is the only save-format surface `SCHEMA_VERSION`
governs. The plan's own schema criterion ("saved shape or meaning moves →
bump") does not fire: a saved `Selection` naming `flaw.ability_block` or
`virtue.magical_blood` round-trips byte-identically before and after this
note's changes — what changes is how the ENGINE interprets that same id's
effects, not what the save stores. **Saves store choices, not resolved
values** (CLAUDE.md) — B0 changes no choice's representation.

---

## 7. Data vs engine

| Slice | Engine work (this note) | Data work (deferred to Phase 3) |
|---|---|---|
| B1 | 4 new `Effect` variants + their TS mirror, 1 new `Prereq` variant + its TS mirror, `CategoryCap.min`/`min_hard`, the grant-aware validator, `effect-parity.test.ts` | `flaw.ability_block`'s actual category (X2, reclassification pass); `flaw.sheltered_upbringing`'s 7-id list (worked example here, but the entry's own JSON edit is data); `flaw.rector`'s prereq (X5); the `social_status` `CategoryCap` row (X5, D41); `flaw.feral_upbringing`'s authorization half (X5 or D-group, alongside D40's XP half — **coordinate with group D so the one entry isn't touched by two slices out of order**) |
| B2 | none (pure data/validator-wiring on B1's mechanism) | the `social_status` cap row itself (min+max, per D41); **`virtue.male_guild_sponsor`'s prerequisite** (identified this revision — no longer waiting on X5 to name the carrier, only to author the JSON) |
| B3 | **build** `ItemPredicate` (`Trained`, `GrantsReputation`, `GrantsPersonalityTrait`), `ParameterDef.exclude_if` + resolver, `PointItem.excluded_if_holds` + grant-aware validator — none of this exists today (§ 1 correction) | `flaw.university_dean`'s `excluded_if_holds` (worked example given here; entry edit is data); D33's own `flaw.flawed_powers` JSON is **already C1/C2's** to author once B3's machinery exists; D12's `trained` flag itself is X3's |
| B4 | `gate: Option<ParamGate>` on `CharacteristicScoreDeltaParam`/`GrantsReputation` (Revision 3 — no new variant) + their TS field widening + `gate.param` integrity check (reusing C1's) + the two real consumer arms (`characteristic_score_bonus`, `reputation_and_caps.rs::reputation_grants`) + `ParamGate::holds` → `pub(crate)` | `virtue.magical_blood`'s Magic Human clause (worked example given here; B4 should land it directly per § 1, not strand it) |
| B5 | the renamed/new `AbilityRollMod` pair + integrity | all 42 entries' per-Ability data — **X7b-e**, "each row-42 item... gets a D58 compute/text verdict", explicitly Phase 3's job |

---

## 8. Slice table

| Slice | Red tests (fail today, and why) | Depends on | e2e / bump | Locale strings (both) |
|---|---|---|---|---|
| **B1** | (a) a hand-authored `flaw.ability_block` with a `forbids_ability_category` effect: buying a Martial Ability afterward must be refused — fails today (variant does not exist, compile error until handled); (b) `flaw.rector` with `has_category` prereq: an entity with no `social_status` selection must fail `prereq_not_met` — fails (variant does not exist); (c) `CategoryCap{min:1,min_hard:true}` on a fresh companion with zero Social Status selections must raise a hard error; a second one must raise only a warning when `max:1,hard:false` — fails (`min` field does not exist); (d) `ui/src/lib/effect-parity.test.ts` (new): every TS `Effect` `type` tag resolves to a real Rust variant — passes trivially today (nothing to check yet), then is the FIRST thing that must go green after adding B1's four TS union members | C1 (per plan's critical path: `C1+D1 → X1`, and B1 reads C0's landed `AbilityRef`/`CategoryRef` conventions for its own new types' style) | e2e at the **B-group boundary** (after B5, per the plan's phase-boundary rule) — B1 alone changes no save/IPC shape, so no boundary run *inside* B1 | new `CODE_CATEGORY_FORBIDDEN_BY_EFFECT`/`CODE_ABILITY_FORBIDDEN_BY_EFFECT`/`CODE_PREREQ_NOT_MET` (existing code, new cause) validation messages — both `.ftl` locales |
| **B2** | a companion with a `social_status` selection AND a second `social_status` selection: `validate` must report exactly one **warning** (not an error), naming both entries; `virtue.male_guild_sponsor` with the new prerequisite: an entity with no OTHER `social_status` selection fails `prereq_not_met` | B1 | none (data + wiring only) | the warning's message string, both locales |
| **B3** | (a) `ItemPredicate`/`exclude_if`/`excluded_if_holds` **do not exist and must be built first** (§ 1 correction) — a hand-authored `flaw.flawed_powers` with `exclude_if: "trained"` fails to **parse** today, not merely to validate; (b) once built: `flaw.university_dean` with `excluded_if_holds: ["grants_reputation"]`, alongside a bought/granted Flaw carrying `Effect::GrantsReputation`: must fail load/validate; the SAME pairing via a **granted** (not bought) Flaw must ALSO fail (closing F-466); (c) `flaw.weak_personality` with both `forbids_item_category` and `excluded_if_holds:["grants_personality_trait"]`: a second `personality` Flaw fails via the category route, an unrelated Virtue granting a Personality Trait fails via the predicate route | C0's § 7 **design** (not its landing — nothing to depend on in the tree), B1 (style precedent for the grant-aware validator) | none new (D33's own data already lands in C1/C2 once B3's machinery exists) | the new exclusion message, both locales |
| **B4** | `virtue.magical_blood` with `bloodline=magic_human`: the +1 Characteristic and the level-3 Reputation apply; `bloodline=magic_animal`: neither applies (the SAME two gated effects on the entry, gate held vs not) — fails today (`gate` field does not exist on either variant, and `ParamGate::holds` is private so the consumer arms cannot even compile against it); a load-time red asserting a `gate.param` naming a dangling/`MultiRef` param is refused, reusing C1's existing `ParamGate` integrity test fixture | C0/C1 (`ParamGate`) | none | none new (Magical Blood's text already exists in both locales as `uncomputed_rule`; moving Magic Human to a computed effect does not need new prose, only a values check) |
| **B5** | `flaw.poor_hearing` with the renamed-pair split: a roll-modifier query for `ability.awareness` returns `-3`, surfaced-only (does not touch the bought/effective score) — fails today (no fixed-target variant exists); a regression test pins `virtue.academic_concentration` still resolves under the renamed `ability_roll_mod_param` tag, in BOTH Rust and TS (`effect-parity.test.ts` catches a stale TS tag left behind by the rename) | none beyond landed C-group | e2e at the **group boundary only** (Ability roll modifiers are a `DerivedSurfacedModifiersSection` item, not an IPC/save shape change) | `flaw.poor_hearing`'s `uncomputed_rule`→computed reclassification note in RULES.md (not a new Fluent string — the existing description already states "-3", per corrections.md's own read) |

**Sequencing inside the group.** B1 before B2 (B2 is pure data on B1's
mechanism) and before B3 (B3's grant-aware validator style follows B1's). B4
and B5 are independent of B1–B3 and of each other — both only need C0/C1's
landed `ParamGate` — so they may run in either order or, given the plan's
1-agent cap, back-to-back in either sequence.

---

## 9. Open questions for Norbert

**Both resolved 2026-09-27 as recommended — see `decisions.md` D60.** F-334 is
text only (X2), so B3 builds no parameter-comparing `Prereq`; Feral Upbringing's
whitelist is creation-only, intersecting if stacked.

1. **F-334 — does the plan's "parameter-comparing prereq" supersede the
   original "cannot be, full stop" closure?** The D1–D18-era ruling
   (`batch-09.md`, corrections.md § 3.15a) is unambiguous: a cross-*character*
   requirement has no carrier in an engine where `Entity` never references a
   second entity, "there is no approximation available either — a
   self-referential `Has` would be trivially satisfied." B3's plan text names
   F-334 as if a `Prereq` addition were owed. **Recommendation:** treat this as
   **not superseded** — B3 does not build a cross-entity `Prereq`, and the
   correct fix for F-334 stays what F-333/F-334's own "Correct value" already
   says: `description` in both locales carrying the reciprocity requirement
   and the True Friend rename as **text**, which is an X2/i18n fix, not an
   engine one. If the plan intends something narrower (e.g., a `beloved`
   free-text parameter with no engine check, matching D9's "text only where
   the choice is genuinely open"), that is compatible with this
   recommendation and adds no new `Prereq`. Needs your call before B3 starts,
   since it changes B3's shape (a text/i18n fix has zero engine surface; a new
   `Prereq` variant has the full § 3a cost).

2. **`RestrictsAbilityCategoryToAbilities` (Feral Upbringing) is a genuinely
   new shape — is the semantics right?** Every other effect this note (and
   C0) adds either **forbids** something normally open or **authorizes**
   something normally gated. This one **narrows** a category that is normally
   *unconditionally* open (`general`) down to a named subset — the first
   "positive override of the default-permitted set" the engine will have.
   Two design questions ride on this, and both are policy calls rather than
   engineering ones: (a) does it apply **only** to the general-XP-pool /
   creation-time purchase path, leaving a later in-play purchase of, say,
   Chirurgy untouched (ArMDE:6113 does not restrict "in play", only "beginning
   Abilities" — consistent with `flaw.sheltered_upbringing`'s own "but you may
   learn them in play" carve-out, so the answer is very likely yes); (b) if a
   future entry ever stacked a SECOND such restriction (not live in the
   catalogue today), do the two whitelists **intersect** (stricter) or does
   the later one **replace** the earlier (last-wins)? **Recommendation:**
   (a) yes, creation-time only, on Sheltered Upbringing's own explicit
   precedent; (b) intersect — a character under two "only these" constraints
   plausibly means "only in both", and replace-semantics would make
   evaluation order load-bearing, which nothing else in this engine's effect
   folding does. Flagging because this is the one new mechanism in the group
   with no existing sibling to copy, unlike every other addition above.

**Dropped (plan-review finding #3): "where does ArMDE:4441's carrier entry
live, by id?"** Identified this revision — `virtue.male_guild_sponsor`
(§ 1, B2). No longer open.

---

## References loaded

`docs/vf-audit/phase-2-plan.md`; `docs/vf-audit/decisions.md` §§ D2, D21, D23,
D33, D40, D41; `docs/vf-audit/corrections.md` §§ 3.15a, 3.15b (+ the F-334,
F-355/F-410/F-426/F-511/F-542 consolidation notes), index rows F-334, F-355,
F-427, F-489, F-502, F-511, F-526, F-542, and rows Q-51/Q-102/Q-132/Q-137;
`docs/vf-audit/batch-09.md` (F-334's original ruling, verbatim);
`docs/vf-audit/design-c0-parameter-model.md` (in full — `ItemPredicate`,
`ParamGate`, `AbilityRef`/`CategoryRef`, the § 1a exhaustive-match table, its
own § 7/§ 11 notes naming B0 as the owner of the point-item-level predicate
consumer); `crates/arm-rules/src/types.rs` (`Prereq`, `PrereqCtx`-adjacent
doc comments, `Effect`, `PointItem`, `CategoryCap`, `CategoryRule`,
`advisory_prerequisites`); `crates/arm-rules/src/ability.rs`
(`AbilityCategory`); `crates/arm-rules/src/validation/prereq.rs`
(`evaluate_prereq`, `PrereqCtx`); `crates/arm-rules/src/validation/selections.rs`
(`categories_in_force`, `validate_permitted_categories`,
`validate_forbidden_categories` — confirmed bought-selections-only today);
`crates/arm-rules/src/validation/authorization.rs` (`categories_requiring_virtue`
gating, confirms `general` is never gated); `crates/arm-rules/src/effective.rs`
(`irrelevant_effect_variants!` macro + doc comment, the landed
`AbilityBonusGated` guarded-arm precedent at `ability_bonus`);
`crates/arm-rules/src/effective/reputation_and_caps.rs` (`reputation_grants`,
the real `GrantsReputation` consumer B4's `gate` field must reach);
`crates/arm-rules/src/effective/characteristic.rs` (`characteristic_score_bonus`,
the real `CharacteristicScoreDeltaParam` consumer); `crates/arm-rules/src/validation/mod.rs`
(`effect_target`, confirmed a named-field, non-`..` match on
`CharacteristicScoreDeltaParam`); `crates/arm-rules/src/ruleset/integrity.rs`
(`validate_item_ratios`, its non-exhaustive trailing wildcard and doc comment;
its own `CharacteristicScoreDeltaParam { param, .. }` arm, confirmed already
`..`-tolerant); `ui/src/lib/types.ts`
(`Prereq`; `Effect`, confirmed real and reachable at lines 136-213/355, NOT
absent as design-c0 § 10.1 and this note's own first draft assumed);
`ui/src/lib/derive.ts` (`houseOnlyValue`; the `warping_grant` check at line
754); `ui/src/lib/components/AbilityTab.svelte` (the `ability_score_grant`
check at line 67); `ui/src/lib/prereq-parity.test.ts` (the
exhaustiveness-parity mechanism itself, and the model for the new
`effect-parity.test.ts`); `rules/core/abilities.json` (verified
`ability.dead_language`/`ability.living_language`/the nine "wilds" Abilities'
categories, and the seven `flaw.sheltered_upbringing` ability ids);
`rules/core/virtues_flaws.json` (verified every point-item id this note names
exists: `flaw.ability_block`, `flaw.feral_upbringing`, `flaw.poor`,
`flaw.rector`, `flaw.sheltered_upbringing`, `flaw.university_dean`,
`flaw.weak_personality`, `virtue.magical_blood`, `virtue.male_guild_sponsor`
at line 5198 with its current zero-`prerequisites` state); rulebook source,
verified directly: ArMDE:5651-5654 (Ability Block), ArMDE:6671-6674
(Rector/Proctor), ArMDE:6721-6724 (Sheltered Upbringing), ArMDE:7076-7079
(Weak Personality), ArMDE:6110-6113 (Feral Upbringing), ArMDE:2816 (already
verified for D41), ArMDE:6146-6148 (Flawed Powers, already verified for D33),
ArMDE:6923-6926 (University Dean, already verified for D23), ArMDE:5173-5178
(True Love (PC)), ArMDE:6614-6617 (Poor Hearing), ArMDE:4359-4371 (Magical
Blood, full passage, all four variants), ArMDE:4439-4442 (Male Guild
Sponsor, this revision).

## Verdict

COMPLETE — ready for the plan-reviewer, then the architect, per the plan's
"design notes... reviewed... before their first implementation slice." Two
items remain genuinely open (§ 9.1, § 9.2); the third open question from the
first draft is resolved and dropped (§ 9).

---

## Revision 2 (2026-09-27)

Applied all 7 plan-review findings (`tmp/b0-plan-review.md`):

1. **BLOCKER** — corrected §§ 0/1/2/7/8: `ItemPredicate`/`exclude_if` are
   designed by C0 § 7 but NOT built (verified: `grep -rn ItemPredicate
   crates/arm-rules/src` is empty); re-scoped B3 to build the enum, the
   `ParameterDef.exclude_if` field, its resolver, and `PointItem
   .excluded_if_holds`, alongside B0's own `GrantsPersonalityTrait` work. Size
   stays M (§ 1).
2. **MAJOR** — added `ui/src/lib/types.ts::Effect` as a real, curated (TS ⊆
   Rust) mirror (§ 0, § 2, § 3b row 16); every new/renamed variant now lists
   its TS member; specified `ui/src/lib/effect-parity.test.ts`, landing in
   B1 before B1's first new variant (§ 2, § 8).
3. **MAJOR** — identified `virtue.male_guild_sponsor` (ArMDE:4439-4442,
   `rules/core/virtues_flaws.json:5198`) as ArMDE:4441's carrier; folded into
   B2's data scope; dropped the open question (§ 1, § 9).
4. **MAJOR** — reworded § 3b's `Gated` rows to mirror the landed
   `AbilityBonusGated` precedent explicitly (macro-tail entry PLUS one guarded
   arm at `characteristic_score_bonus`, not "tail is wrong"); added the
   non-exhaustive `reputation_and_caps.rs` consumer B4 must also touch.
5. **MINOR** — added `validate_item_ratios` as § 3b row 15 (out of scope, one
   sentence).
6. **MINOR** — resolved by construction rather than caveat: `Gated`'s inner
   field is now `Box<GatedEffect>`, a closed two-member enum with no `Gated`
   tag, which bounds parse-time recursion at the type level (§ 2, § 5) —
   stronger than the load-time-only check the first draft had.
7. **MINOR** — added an explicit reconciling paragraph in § 1 (B1): "one
   mechanism, not one variant" for F-355/F-542, given the `AbilityCategory`/
   `String` domain mismatch.

---

## Revision 3 (2026-09-27)

Applied all 4 architect-review findings (`tmp/b0-architect-review.md`):

1. **MAJOR** — dropped `Effect::Gated`/`GatedEffect` entirely (§ 1, § 2, § 3b,
   § 3c new, § 4, § 5, § 7, § 8). B4 now adds `gate: Option<ParamGate>`
   directly to the two concrete carriers, `CharacteristicScoreDeltaParam` and
   `GrantsReputation` (`#[serde(default, skip_serializing_if =
   "Option::is_none")]` — additive, no bump), on the SAME embedded-gate idiom
   `AbilityRef::Scoped`/`CategoryRef::Scoped`/`AbilityBonusGated` already use.
   Re-derived from scratch by grepping every real occurrence of both variants
   in `crates/arm-rules/src`: only three sites are compiler-forced or owed
   (`characteristic_score_bonus`, `validation/mod.rs::effect_target`,
   `reputation_and_caps.rs::reputation_grants`, plus a voluntary integrity
   check reusing C1's existing `ParamGate` rule) — a small fraction of the
   dropped wrapper's 16-row/two-enum cost. The parse-time-recursion
   discussion (Revision 2, § 5) is removed: nothing is boxed, so there is
   nothing to nest. Rejection recorded in § 2 (YAGNI, one gating idiom).
2. **MINOR** — `ParamGate::holds` (`types.rs:1028`, private today) is made
   `pub(crate)` by B4, stated explicitly in § 2/§ 4/§ 7 — the two real
   consumer arms live outside `types.rs` and cannot compile against a private
   method.
3. **MINOR** — added a cross-referencing doc-comment requirement (§ 4)
   between the new `validate_category_effect_prohibitions` and the existing
   `validate_forbidden_categories`, each naming the axis the other does NOT
   cover.
4. **LOW** — added a cross-referencing doc-comment requirement (§ 4) on
   `categories_in_force` ("bought-only by design; see `held_categories` for
   the grant-aware twin"), so the two same-shaped helpers are not conflated.
