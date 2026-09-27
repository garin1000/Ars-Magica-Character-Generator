# C0 — one parameter model for D14, D33, D34, D35, D48, D9 part 3, W2

**Slice.** `phase-2-plan.md` group C, C0. Design only, no code. Reviewed next by
the plan-reviewer, then the architect, before C1 starts.

**Mandate.** Six rulings all extend `ParameterDef`/`Effect`. Designed together so
C1–C5c produce one coherent model, not five that don't compose.

---

## 0. What already exists (read before designing on top of it)

| Piece | Where | Shape |
|---|---|---|
| `ParamType` | `types.rs::ParamType` | one variant (`Ref`), doc comment already says "retained... so a future non-ref parameter kind can be added without a wire change" |
| `ParameterDomain` | `types.rs::ParameterDomain` | `Ability, Art, Technique, Form, Characteristic, Item, Enumerated, Category, Realm, Text` |
| `ParameterDef` | `types.rs::ParameterDef` | `key, param_type, domain, values, at_most_one_of: Vec<BTreeSet<Id>>, max_per_value, require_categories: BTreeSet<String>, require_possessed, forbid_tainted, require_power` |
| `Selection` | `types.rs::Selection` | `item_ref: Id, params: BTreeMap<String, Id>` — **every stored parameter value is one bare `Id` today** |
| `AbilityInstanceRef` | `effective/xp.rs` | `{ ability: Id, parameter: Option<String> }` — **already the exact literal-instance shape D14 asks for**, just not reachable from rules JSON |
| `AbilityAuthorization` | `types.rs::Effect` | `{ abilities: Vec<Id>, categories: Vec<AbilityCategory> }` — id/category only, no instance, no gate |
| `RestrictedAbilityXp` | `types.rs::Effect` | `{ amount, abilities: Vec<Id>, categories: Vec<AbilityCategory> }` |
| `PoolEligibility::Ability` | `effective/xp.rs` | `{ abilities, categories, instances: Vec<AbilityInstanceRef>, exclude }`; `pool_covers` treats non-empty `instances` as **the only test** (not a union) |
| `ability_authorizations()` | `effective/xp.rs:371` | exhaustive `match` over every `Effect` variant, returns `(BTreeSet<Id>, BTreeSet<AbilityCategory>)` — id/category only |
| `validate_ability_authorization` | `validation/authorization.rs` | `authorized_categories.contains(category) \|\| authorized_abilities.contains(ability)` — **no instance check today**, which is F-349/F-16x's bug |
| Load-time integrity | `ruleset/integrity.rs::validate_parameter_defs` | the established pattern: a flag/list is rejected when its paired `domain` can't use it (`require_categories` needs `Item`, `require_power` needs `Text`, …) |
| Canonical multi-value precedent | `ParameterDef::at_most_one_of: Vec<BTreeSet<Id>>` | **`BTreeSet` already IS the project's canonical-set answer** — set equality, no separate "sort before compare" step needed |

`SCHEMA_VERSION` is 17 (confirmed in `migration.rs`). D10 (multiplicity default →
1) is **already landed** (`default_max_total`/`default_max_per_value` both
return `1` in the tree today) — the plan's "still `u8::MAX`" fact predates Q4.

---

## 1. The unified model

Two new small types carry all of D14/D33/D34/D35/D48/D9p3. Nothing else moves.

```rust
/// A value bound to a literal, or read from the SAME selection's own
/// parameter at evaluation time (D14's two forms).
#[serde(untagged)]
enum ParamValue {
    /// Fixed at authoring time: `ability.dead_language` + `language = "latin"`.
    Literal(String),
    /// Read from `Selection::params[param]` when this effect's owning
    /// selection is evaluated. `param` must be a key the SAME item declares
    /// (checked at load, see § 6).
    Bound { param: String },
}

/// An Ability id inside an authorization / restricted-XP-pool list, carrying
/// D14's two constraints. Deserializes from a bare string for the common,
/// unconstrained case, and serializes back to one whenever both fields are
/// absent — so all 99 existing ability-id entries across today's
/// `abilities: [...]` lists stay byte-identical.
#[serde(untagged)]
enum AbilityRef {
    Bare(Id),
    Scoped {
        ability: Id,
        /// D14 shape 1: restricts to ONE instance of a *parameterized*
        /// Ability (`ability.dead_language` + `instance: Literal("latin")`,
        /// or `ability.enchanting_ability` + `instance: Bound{param:"medium"}`
        /// for F-63).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instance: Option<ParamValue>,
        /// This entry is authorized/funded only when the OWN selection's
        /// `gate.param` equals `gate.equals`. Absent = unconditional.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate: Option<ParamGate>,
    },
}

/// An Ability *category* inside an authorization list, with the same gate
/// (D14 shape 2 / W2's exclusive choice, category-scoped rather than
/// id-scoped: Wise One, Custos).
#[serde(untagged)]
enum CategoryRef {
    Bare(AbilityCategory),
    Scoped { category: AbilityCategory, gate: Option<ParamGate> },
}

/// D14's binding, realized as conditional LIST MEMBERSHIP rather than value
/// substitution (see § 3 for why).
struct ParamGate {
    /// A parameter key the SAME item declares.
    param: String,
    /// The literal value that activates this entry.
    equals: Id,
}
```

`AbilityRef`/`CategoryRef` replace `Vec<Id>` / `Vec<AbilityCategory>` in
**both** `Effect::AbilityAuthorization` and `Effect::RestrictedAbilityXp`:

```rust
AbilityAuthorization { abilities: Vec<AbilityRef>, categories: Vec<CategoryRef> }
RestrictedAbilityXp {
    amount: u32,
    abilities: Vec<AbilityRef>,
    categories: Vec<AbilityCategory>,   // earmark/grant categories are never gated — unchanged
    /// D48: specific instances this pool funds (union, not exclusive — see § 5).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    instances: Vec<AbilityRef>,
}
```

This is a **ruleset-JSON shape change, not a save-format change** — `Effect`
lives in `rules/core/virtues_flaws.json`, which the repo re-authors wholesale
in the same commit. It needs no `SCHEMA_VERSION` bump (matches the plan's "only
C5a bumps").

### D35 — `ParamType::Number`

```rust
enum ParamType {
    #[default]
    Ref,
    MultiRef,                       // D9 part 3, § 5
    Number { min: i32, max: i32 },  // D35
}
```

Paired with a new `ParameterDomain::Number` (nothing to resolve; exists so
`domain` stays a required field with no `Option` wire change, and so integrity
can reject the mismatch the same way it already rejects `require_categories`
on a non-`item` domain). **Doc-comment obligation, binding on whoever writes
this variant (C3):** `ParameterDomain::Number` is the **redundant half of the
`ParamType::Number` pair** — it exists solely so `domain` stays non-optional,
and carries no resolution logic of its own (unlike every other domain, which
resolves a value against a registry). Its doc comment must say so explicitly,
on the precedent of `ParamType`'s own current comment ("today it carries no
behavior... retained so a future non-ref parameter kind can be added without a
wire change") — a future reader must not go looking in `Number`'s `Display`
arm or a resolution match for logic that lives entirely on `ParamType::Number`
instead.

```json
{ "key": "years", "type": { "number": { "min": 1, "max": 2 } }, "domain": "number" }
```

New effect, parameter-scaled additive grant (D13's family, **additive** mode
only — not the earmark or replacement modes):

```rust
RestrictedAbilityXp { .. }   // unchanged shape; scaling is a NEW effect, not a flag on it:

ScaledRestrictedAbilityXp {
    /// Parameter key (domain = number) whose value is the per-unit count.
    param: String,
    /// XP per unit (Simple Student: 30 per finished year).
    per_unit: u32,
    abilities: Vec<AbilityRef>,
    categories: Vec<AbilityCategory>,
}
```

Kept as a **separate** variant rather than a `Option<(String, u32)>` field
bolted onto `RestrictedAbilityXp`, on the same precedent as
`CharacteristicScoreDelta`/`CharacteristicScoreDeltaParam`: a fixed-amount
effect and a parameter-scaled one are already a "Foo"/"FooParam" pair
elsewhere in this file, and the flow-graph code that reads `RestrictedAbilityXp`
(`effective/xp.rs::restricted_ability_xp_pools`) must not silently pass through
an unscaled `amount` for an entry that actually needs `per_unit * bound value`.

`virtue.simple_student` (ArMDE:4960 — *"He receives 30 experience points per
finished year that he can apply to Latin or Artes Liberales... If he has
finished his second year of studies, he is in the liminal position..."*):

```json
{
  "id": "virtue.simple_student",
  "parameters": [{ "key": "years", "type": { "number": { "min": 1, "max": 2 } }, "domain": "number" }],
  "effects": [
    { "type": "scaled_restricted_ability_xp", "param": "years", "per_unit": 30,
      "abilities": ["ability.artes_liberales", { "ability": "ability.dead_language", "instance": { "literal": "latin" } }] }
  ]
}
```

Note the `dead_language` entry already uses D14's literal-instance form in the
same edit — Simple Student was never one of D14's named carriers, but it
authorizes/funds Latin specifically (not any dead language), so it needs the
same fix the moment it is written. The cap of 2 (60 XP) is `max: 2` on the
parameter — enforced by the existing numeric-range integrity/validation path,
no new machinery.

**D3's truncated-apprenticeship reuse.** `flaw.abandoned_apprentice`
(ArMDE:5647, D56) needs "age abandoned" as a **number** parameter feeding the
16-XP/8-level-per-year computation — same `ParamType::Number` variant, a
different consumer (a life-stage truncation, not an XP grant). C0 only needs
to confirm the type is general enough; D3 (group D) owns wiring it to the
truncation math.

---

## 1a. Every exhaustive `match Effect` site — no-op-or-real verdict, by slice

Three new variants are introduced across the C-group:
`ScaledRestrictedAbilityXp` (**C3**, D35), `AbilityBonusGated` (**C1**, § 3 —
**kept as a sibling variant, decided**: the realm value bound by Student of
Realm's gate is not an Ability id, so `AbilityBonus`'s existing
"target-IS-my-own-param-value" reading cannot carry it; stretching
`AbilityBonus` itself was the alternative this note's first draft left open,
and the architect review closes it), and `AbilityScoreGrantParam` (**C5c**,
§ 2 — F-63; it does not exist before C5c, so nothing before C5c can or should
reference it).

**Walked by symbol, not by grep-guessing**: the crate has exactly one shared
macro (`effective.rs::irrelevant_effect_variants!`) plus **seven** hand-written
functions that each contain their own exhaustive `match` over `Effect` (found
by locating every site listing `Effect::AbilityBonus { .. }` in an
"everything else" tail — the macro's own definition is one, the other six are
hand-written copies predating it or deliberately not sharing it). No wildcards
exist at any of these eight sites; each names every variant.

| # | Site (`file.rs::fn`) | Exhaustive via | `ScaledRestrictedAbilityXp` | `AbilityBonusGated` | `AbilityScoreGrantParam` |
|---|---|---|---|---|---|
| 1 | `effective.rs::irrelevant_effect_variants!` (macro; propagates to sites 2–6 below) | macro tail | no-op — add to tail (same family as `RestrictedAbilityXp`, already there) | no-op **at every site this macro serves except `ability_bonus` itself** (site 2) — safe to list in the shared tail because its own arm in `ability_bonus` is *guarded*, exactly the precedent `AbilityBonus{..}` already sets (a guarded interesting arm + the same variant in the shared tail is sound; only an *unguarded* interesting arm may not reuse the macro — see the macro's own doc comment) | no-op (same family as `AbilityScoreGrant`, already there) |
| 2 | `effective/ability.rs::ability_bonus` | macro (site 1) + own arms | (via macro) no-op | **real** — new guarded arm, C1: for each active target (gate holds or absent) naming this `(ability, parameter)` instance, sum `amount` | (via macro) no-op |
| 3 | `effective/art.rs::art_bonus` | macro (site 1) | (via macro) no-op | (via macro) no-op — `AbilityBonusGated` targets Abilities only, never Arts | (via macro) no-op |
| 4 | `effective/art.rs::deficient_arts` | macro (site 1) | (via macro) no-op | (via macro) no-op | (via macro) no-op |
| 5 | `effective/characteristic.rs::characteristic_score_bonus` | macro (site 1) | (via macro) no-op | (via macro) no-op | (via macro) no-op |
| 6 | `effective/xp.rs::ability_affinity` | macro (site 1) | (via macro) no-op | (via macro) no-op | (via macro) no-op |
| 7 | `effective/xp.rs::art_affinity` | macro (site 1) | (via macro) no-op | (via macro) no-op | (via macro) no-op |
| 8 | `effective/spell.rs::spell_levels_bonus` | hand tail (own copy, not the macro — an unconditional interesting arm) | no-op — add to tail | no-op — add to tail | no-op — add to tail |
| 9 | `effective/spell.rs::general_xp_bonus` | hand tail | no-op — add to tail | no-op — add to tail | no-op — add to tail |
| 10 | `effective/spell.rs::spell_mastery_advancement_affinity` | hand tail | no-op — add to tail | no-op — add to tail | no-op — add to tail |
| 11 | `derived.rs::in_play_mods` | hand tail (~30-variant "creation-effect, not in-play" list) | no-op — add to tail (a creation-time XP grant, not an in-play total) | no-op — add to tail | no-op — add to tail |
| 12 | `effective/xp.rs::ability_authorizations` → **renamed/refactored to `resolve_ability_refs` in C4** (§ 6) | hand-written, real logic | **real, C3**: contributes its `abilities`/`categories` to the authorized set exactly like `RestrictedAbilityXp` (an earmark is itself permission) | **real, C1**: contributes each gate-active category/ability, same fold as `AbilityAuthorization` | no-op — a floor grant is not an authorization path (matches `AbilityScoreGrant`'s existing no-op here) |
| 13 | `ruleset/integrity.rs::validate_effect_refs` | hand-written, real logic, **no wildcard** | **real, C3**: `param` must resolve to a declared **`Number`**-domain parameter (not `Ref`/`MultiRef`) — a new arm, not the bare-marker tail | **real, C1**: each `targets[].gate.param` (if present) must resolve on the SAME item, each `targets[].ability` must resolve in the ability catalogue | **real, C5c**: `ability` must resolve; `instance` (`Bound{param}`) must resolve to a **non-`MultiRef`** parameter on the same item (§ 9) |
| 14 | `validation/mod.rs::effect_target` | hand-written, real logic, **no wildcard** | `Other` — no-op, matches `RestrictedAbilityXp`'s existing classification (a pool grant is not a dangling-target-checkable bonus) | `Other` — no-op: targets are fixed catalogue ids, never a player-typed instance, so there is nothing here that can *dangle* the way `AbilityBonus`'s free param can | `Other` — no-op, matches `AbilityScoreGrant`'s existing classification |

**Non-exhaustive but real consumers, found by tracing the symbol
`AbilityScoreGrant`/`RestrictedAbilityXp`, not by a compiler-forced match.**
These use `if let` (not `match`), so a new sibling variant is **not** a
compile error here — each is a genuine, easy-to-miss site this note is
recording precisely so C5c does not ship `AbilityScoreGrantParam` blind to
its own family's other consumers:

| Site | Today's role | `AbilityScoreGrantParam` treatment | Slice |
|---|---|---|---|
| `effective/ability.rs::granted_ability_floor` | Floors a **fixed, unparameterized** target only — `if parameter.is_some() { return 0; }` short-circuits before ever reading `AbilityScoreGrant` | Must resolve the gated/bound instance and floor **that specific `(ability, parameter)` pair** when `parameter.is_some()` — this is the function whose early return currently makes a parameter-bound floor grant impossible at all | C5c |
| `effective/ability.rs::ability_score_floors` | Lists every ability granted a free starting score, for the UI | Must also list the resolved instance, so the UI badge lands on the right row (same reasoning `ability_bonuses` already documents for Puissant on a parameterized Ability) | C5c |
| `validation/prereq.rs` (`AbilityMin` floor reading, ~line 172) | Counts an `AbilityScoreGrant` floor toward `Prereq::AbilityMin` | Must count `AbilityScoreGrantParam`'s resolved-instance floor the same way | C5c |

`RestrictedAbilityXp`'s own non-exhaustive consumers
(`effective/xp.rs::restricted_ability_xp_pools`, the pool-builder) are already
covered in § 5/§ 8's worked examples and are not repeated here.

---

## 2. Ability-reference shape — which `Effect` variants gain it

| Variant | Gains `AbilityRef`/`CategoryRef`? | Why / why not |
|---|---|---|
| `AbilityAuthorization` | **Yes** — `abilities: Vec<AbilityRef>`, `categories: Vec<CategoryRef>` | The primary carrier of D14 shape 1 (`covenant_upbringing`), shape 2 (`student_of_realm`), and W2 (`wise_one`, `custos`) |
| `RestrictedAbilityXp` | **Yes** — `abilities: Vec<AbilityRef>` (categories stay bare; instances new, D48) | D13's earmark needs literal instances (Church Upbringing: Latin, Organization Lore: Church); D48's four+ known instance-scoped pools (Marshal, Master of Kennels, Master Bard, Forge Companion, …) |
| `ScaledRestrictedAbilityXp` (new, D35) | **Yes** — same `abilities: Vec<AbilityRef>` | Simple Student's Latin needs the literal-instance form from day one |
| `AbilityScoreGrant` | **No — stays `ability: Id`** | Fixed target, never player-chosen (Second Sight). No known carrier needs a gate or instance |
| `AbilityBonus`, `AffinityAbilityCost` | **No change** | Already parameter-relative via their own `param: String` — the target IS the selection's own parameter, which is a *different*, already-correct pattern (the whole point of the selection is picking that ability) |
| `GroupAffinityCost` | **No change** | `abilities: BTreeSet<Id>` deliberately names a **fixed group of whole ids** (Linguist: every language, parameterized or not) — D14's defect is an *unwanted* over-permission; Linguist's breadth is *intended*, so gating it would be a regression, not a fix |
| `AbilityRollMod` | **No change (out of C-group)** | Already parameter-relative (free-text subject). F-489's 42 entries need a **new**, id-based roll-modifier variant — that's B5's job; when it lands it should reuse `AbilityRef` for consistency, but C0 does not create it |

**New small effect needed for F-63** (`virtue.enchanting_ability`, ArMDE:3747-3750:
*"Choosing this Virtue confers the Ability Enchanting (Ability) 1"*, where
*(Ability)* is a player-chosen medium — music, dance, drawing, storytelling,
"even craftwork"): a parameter-bound sibling of `AbilityScoreGrant`, the same
"Foo"/"FooParam" pattern as above:

```rust
AbilityScoreGrantParam { ability: Id, instance: Option<ParamValue>, amount: u8 }
```

`virtue.enchanting_ability`'s own parameter is `domain: text` (the passage's
"even craftwork" hedges the list as illustrative, not exhaustive, so D9's
"text only where the choice is genuinely open" applies), and the grant reads
`instance: Bound{param:"medium"}`. **This is C5c's F-63 item, not C1's** — flagged
here only so the shape exists before C5c needs it.

---

## 3. Exclusive choice, and why it cannot over-permit (F-349)

**The trap, restated.** F-349 recorded only "a two-category gated Ability
permission" for `virtue.wise_one` — not that the permission is *exclusive*. The
naive fix — `AbilityAuthorization { categories: [academic, arcane] }` — makes
the under-permission (today: neither is authorized, ArMDE:5259 "You may take
either Arcane or Academic Abilities, but not both" is unmodelled and the app
wrongly refuses the book's own Witch template) into an **over-permission**
(both, unconditionally) — worse, because it is silent.

**The fix: a gate, not a value substitution.** D14's own wording — "a binding
to the selecting entry's own parameter... `ability.realm_lore`-style" — reads
as if one parameterized ability could absorb the whole family. The catalogue
does not support that reading for either W2 or Student of Realm: the four
realm Lores are four **separate, non-parameterized** ability ids
(`ability.dominion_lore`, `ability.faerie_lore`, `ability.infernal_lore`,
`ability.magic_lore` — confirmed in `rules/core/virtues_flaws.json:6108-6112`),
and Wise One/Custos's two/three options are **categories**, which have no
"instance" axis at all. **Substituting a value into a target reference does
not fit either case.** What both cases need instead is: *list every candidate,
and activate the ones the player's own choice selects.*

```json
// virtue.wise_one
{
  "parameters": [{ "key": "study", "type": "ref", "domain": "enumerated", "values": ["academic", "arcane"] }],
  "effects": [{
    "type": "ability_authorization",
    "categories": [
      { "category": "academic", "gate": { "param": "study", "equals": "academic" } },
      { "category": "arcane",   "gate": { "param": "study", "equals": "arcane" } }
    ],
    "abilities": [{ "ability": "ability.dead_language", "instance": { "literal": "latin" } }]
  }]
}
```

(The trailing unconditional `abilities` entry is ArMDE:5259's own carve-out:
*"If you choose Martial or Arcane Abilities, you may still learn to speak
Latin"* on `virtue.custos` — showing gated and ungated entries coexist in one
list with no special-casing.)

**The proof this cannot over-permit — scoped to one selection.**
`ability_authorizations()` (§ 4) folds each `CategoryRef`/`AbilityRef` in only
when its `gate` is `None` **or** `selection.params.get(gate.param) ==
Some(gate.equals)`. With `study = "academic"` selected, the `arcane` entry's
gate fails and it contributes **nothing** to the authorized set — by
construction, not by a downstream filter that could be forgotten. There is no
code path that authorizes an entry whose gate does not hold: the fold either
includes an entry or it does not, there is no third "include but mark
restricted" state to get wrong. This is exactly what closes F-349's trap
**within the one `Selection` that carries the gate**, and it is checkable by
one test per mutually-exclusive family: *selecting academic must leave arcane
Abilities gated, and vice versa* — the negative half F-349 shows is easy to
skip.

**The claim does NOT extend across two selections of the same item.** A
character holding a **bought** `wise_one(study=academic)` selection **and** a
**granted** `wise_one(study=arcane)` selection (e.g. from a future House or
Mystery grant) would union to both — each selection's fold runs independently
and the authorized-set union happens *after*, so two selections voting
oppositely both survive. **Not live today**: no entry in the catalogue grants
`wise_one`, `custos`, `templar_specialist`, or `student_of_realm` — all four
are bought-only. **C1 owes a test that pins this absence**
(`no_gated_authorization_item_is_ever_granted` or equivalent, scanning
`GrantsSelection`/House/Mythic-type/warping-fill targets), so the catalogue
cannot silently reopen the trap by adding a grant of one of these four without
anyone noticing. The moment any of them becomes grantable, the fix is
grant-aware deduplication — D2's territory (both validators already read
*effective* selections; the gate fold would need the same "collapse to one
per `(item_ref, params)`" treatment before folding, not after), not a defect
in this design. Recorded here as a coupling risk, not fixed here.

**Same mechanism, three more consumers**, so it is not built for one entry:

| Entry | Finding | Choice parameter | Gated list |
|---|---|---|---|
| `virtue.wise_one` | W2 | `study` ∈ {academic, arcane} | 2 `CategoryRef`s |
| `virtue.custos` | F-42 / W2 | `study` ∈ {martial, academic, arcane} | 3 `CategoryRef`s + 1 ungated `AbilityRef` (spoken Latin) |
| `virtue.templar_specialist` | F-317 | `study` ∈ **open set** — see below | N `CategoryRef`s, N to be decided |
| `virtue.student_of_realm` | D14 shape 2, row 50(a) | `realm` ∈ {dominion, faerie, infernal, magic} (**already** the entry's own `realm`-domain parameter) | 4 `AbilityRef`s, one per Lore |

**Templar Specialist's set is open, and this note does not close it.**
ArMDE:5133-5136: *"filling a crucial role... such as craftsmen, blacksmiths,
artisans, notaries, squires, soldiers, scribes, or translators. You may take
**one restricted group of Abilities** during character creation, **such as**
Academic or Martial Abilities."* Both "such as" hedges mean the passage gives
**examples**, not an exhaustive enumeration — unlike Wise One's "either...or"
and Custos's "either...or...or", which genuinely close the set at two and
three. Do **not** model this as a two- or three-member `enumerated` parameter
by analogy with its neighbours. **C1 owes this decision, with its
justification recorded in `RULES.md`**, choosing one of:

- **Derive a closed set from the passage's own examples**, reading each named
  role (craftsman → General/`craft`-bearing categories, notary/scribe →
  Academic, soldier/squire → Martial, translator → the `living_language`
  Ability specifically) and enumerating the categories those roles actually
  imply — accepting that this is an interpretive narrowing of an open list,
  the same trade D34 already made and named explicitly ("what it costs, stated
  plainly").
- **Model it as the full category domain** (`enumerated` over
  `AbilityCategory::ALL` minus `Supernatural`, per `CLAUDE.md`'s "the engine
  surfaces taxonomies, the UI never re-hardcodes them"), letting the player
  pick any one restricted category rather than guessing which examples the
  book meant to be exhaustive.
- **Leave it `uncomputed_rule` with the choice as free `text`**, if neither
  reading survives scrutiny — matching D9's "text only where the choice is
  genuinely open."

This note takes no position among the three; each is defensible and the
choice is C1's to make and record, not C0's to pre-empt.

**Row 50(a)'s +2 Lore bonus rides the same gate.** ArMDE:5054: *"you have a +2
bonus on all uses of the appropriate Lore"* — an `ability_bonus`-shaped effect,
but `AbilityBonus.param` already means "the target IS this selection's own
parameter value" (§ 2's "no change" row) — which is **exactly** `realm`'s
value read as an Ability id, except `realm`'s domain is `Realm`, not `Ability`
(a Realm id, e.g. `realm.magic`, is not an Ability id). So the +2 needs the
**same gated-list shape** as the authorization, not `AbilityBonus`:

```json
{
  "type": "ability_bonus_gated",
  "targets": [
    { "ability": "ability.dominion_lore", "gate": { "param": "realm", "equals": "realm.dominion" } },
    { "ability": "ability.faerie_lore",   "gate": { "param": "realm", "equals": "realm.faerie" } },
    { "ability": "ability.infernal_lore", "gate": { "param": "realm", "equals": "realm.infernal" } },
    { "ability": "ability.magic_lore",    "gate": { "param": "realm", "equals": "realm.magic" } }
  ],
  "amount": 2
}
```

One more small variant (`AbilityBonusGated`, using the same `AbilityRef` list
+ `gate`), landing in C1 beside the authorization fix since it is the same
entry and the same mechanism — not a second design.

---

## 4. Evaluation — what changes in the engine

**`ability_authorizations()` (`effective/xp.rs:371`) gains the selection's own
`params` as an input to the fold**, since a gate can only be read off the
selection that carries it:

```rust
for selection in selections_for_effects(entity, ruleset).iter() {
    let Some(item) = ruleset.point_items.get(&selection.item_ref) else { continue };
    for effect in &item.effects {
        match effect {
            Effect::AbilityAuthorization { abilities, categories } => {
                for a in abilities { if a.active_for(selection) { authorized.insert(a.resolved(selection)); } }
                for c in categories { if c.active_for(selection) { categories_out.insert(c.category); } }
            }
            ...
```

`AuthorizedAbility` (the new element type, replacing bare `Id` in the returned
set) carries the resolved instance constraint:

```rust
struct AuthorizedAbility { ability: Id, instance: Option<String> }
```

and `validate_ability_authorization`'s membership test becomes an OR-match
(`instance.is_none()` on the authorized side = any instance; `Some(x)` requires
the bought `AbilityScore::parameter == x`) instead of today's bare
`.contains(&entry.ability)` — **this is the actual F-349/F-16x fix**: Covenant
Upbringing's authorized set becomes `{(dead_language, Some("latin"))}`, so
buying Ancient Greek at that instance is correctly refused.

`AbilityRef::active_for`/`resolved` and `CategoryRef::active_for` are the only
new logic; `ParamGate` evaluation is one `BTreeMap::get` + `==`, no new
control flow.

---

## 5. D48 — pool instances become a union, not an override

Change **one line's meaning** in `effective/xp.rs::pool_covers`:

```rust
// BEFORE — instances non-empty is the ONLY test:
if !instances.is_empty() { return instances.iter().any(|i| i.matches(...)); }
abilities.contains(ability) || categories.contains(category)

// AFTER — union:
(!instances.is_empty() && instances.iter().any(|i| i.matches(ability, parameter)))
    || abilities.contains(ability)
    || categories.contains(category)
```

(`exclude` still applies first and short-circuits, unchanged.)

`restricted_ability_xp_pools()` stops hardcoding `instances: Vec::new()` and
instead resolves `RestrictedAbilityXp.instances: Vec<AbilityRef>` (§ 1) against
the owning selection's `params`, exactly as `ability_authorizations()` does —
one shared helper, `fn resolve_ability_refs(refs: &[AbilityRef], selection: &Selection) -> Vec<AbilityInstanceRef>`,
used by both call sites so the gate/instance semantics cannot drift between
"what may I own" and "what may this pool fund".

**Verified behaviour-preserving** (decisions.md D48): the only existing
`instances` producer, `childhood_native_language_pool`, builds
`abilities: [], categories: [], instances: [native]` — union over two empty
lists plus `instances` is exactly `instances`, so the childhood pool's
behaviour does not change.

`virtue.master_bard` (ArMDE:4457-4462, one **240**-point pool funding
*"Art of Memory, Profession: Storyteller, Profession: Poet, any Area Lore, or
any Organization Lore, or any Faerie Lore, or any Magic Lore"* — the shipped
entry's `amount` is **240**, not 50; this note's first draft copied the wrong
figure, and the same slip sits in `decisions.md` D48's own worked example
(Norbert is annotating that separately, not this note):

```json
{
  "type": "restricted_ability_xp",
  "amount": 240,
  "abilities": [
    "ability.area_lore", "ability.art_of_memory", "ability.faerie_lore",
    "ability.magic_lore", "ability.organization_lore"
  ],
  "instances": [
    { "ability": "ability.profession", "instance": { "literal": "storyteller" } },
    { "ability": "ability.profession", "instance": { "literal": "poet" } }
  ]
}
```

"Any Area Lore", "any Organization Lore", "any Faerie Lore" and "any Magic
Lore" are **whole (parameterized or plain) Abilities with no instance
restriction**, which is already `abilities: [...]` with bare `AbilityRef::Bare`
entries, no `instance` — **no new field needed**, and critically
`ability.faerie_lore`/`ability.magic_lore` **must stay in `abilities`**, not
move to `instances`: they are not instances of one parameterized Ability, they
are two of the four distinct realm-Lore ids (§ 3). **C4 owes two regression
tests, not just the four/twelve-pool re-derivation**: `virtue.master_bard`'s
`amount` stays `240`, and the pool still names all **six** distinct Abilities
the passage grants — five staying in `abilities` (`area_lore`,
`art_of_memory`, `faerie_lore`, `magic_lore`, `organization_lore`) and the
sixth (`profession`) moving to the two `instances` entries — after the
`instances`/union rewrite. D48's fix must touch only how `profession` is
scoped, not drop or relocate any of the other five.

Twelve pools total confirmed instance-naming by measurements row 12 (four
known + eight "candidates to confirm" — `virtue.senior_bard`,
`virtue.falconer`, `virtue.craft_guild_training`, `virtue.educated`,
`virtue.baccalaureus`, `virtue.hermetic_experience`, `virtue.clan_ilfetu`,
`virtue.rosh_beth_din`). **C4 re-derives and confirms each against its own
passage before writing data — the count is not to be trusted from this note.**

---

## 6. D34 — the id whitelist

Additive to `require_categories`, same domain restriction (`item` only),
following the exact integrity-check precedent already in
`ruleset::integrity::validate_parameter_defs`:

```rust
/// One-id whitelist, additive to `require_categories`: a value resolves if
/// EITHER test passes. `flaw.false_power`'s domain is `supernatural` plus
/// two named ids the category axis cannot reach (Diedne Magic is `hermetic`,
/// The Gift is `special`).
#[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
pub allow_ids: BTreeSet<Id>,
```

```json
{
  "key": "target",
  "type": "ref",
  "domain": "item",
  "require_categories": ["supernatural"],
  "allow_ids": ["virtue.diedne_magic", "virtue.the_gift"],
  "forbid_tainted": true
}
```

(`flaw.false_power`'s existing `require_possessed: true` is unaffected.)
**Not** routed through D33's predicate (D34 says so explicitly) — a whitelist
is a closed, hand-maintained list; D33 needs an open-ended computed test. Two
different tools for two different shapes, kept apart on purpose.

---

## 7. D33 — the predicate interface, shared with B0/B3

**C0's job is the interface only; B3 implements it, gated on D12/X3.** One
small shared vocabulary, so a predicate is spelled once regardless of which of
the two mechanisms (a parameter's domain-narrowing, or a point-item's
exclusion) consumes it:

```rust
/// A property-based test over a point item, for exclusions the book states
/// by description rather than by id (D23) or category (D21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemPredicate {
    /// D12's intrinsic/trained classification: "operates on Techniques, Forms,
    /// spells, Casting/Lab Totals, Parma Magica, certámen or Twilight" — reads
    /// a `trained: bool` flag D12's classification pass adds to `PointItem`
    /// (X3's output; this predicate is BLOCKED on that pass, not on C0/C2).
    Trained,
    /// Carries a `grants_reputation` effect (Q-137, `virtue.university_dean`).
    /// Derivable with no new data — `item.effects.iter().any(|e| matches!(e, Effect::GrantsReputation{..}))`.
    GrantsReputation,
}
```

**C0's one consumer** — `ParameterDef` gains a negative filter, paired with
`domain: item` exactly like `allow_ids`/`require_categories`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub exclude_if: Option<ItemPredicate>,
```

`flaw.flawed_powers` (ArMDE:6148 — *"Any Flaw that is only appropriate to
Hermetic Magic... cannot be taken with this Flaw"*), which today ships **no
`parameters` at all**:

```json
{
  "key": "imported_flaw",
  "type": "ref",
  "domain": "item",
  "require_categories": ["hermetic"],
  "exclude_if": "trained"
}
```

resolution: `param_value_resolves` gains one more test, symmetric with
`forbid_tainted`'s — a value resolves only if `!item.trained` when
`exclude_if == Some(Trained)`. Reported as the existing
`CODE_UNKNOWN_PARAM_VALUE`, on `forbid_tainted`'s own precedent (the
narrowing IS the domain, no second code for one idea).

**B0's consumer (out of C0's scope, named so B3 does not reinvent the enum):**
a `PointItem`-level exclusion for D23's other shape (`university_dean` vs
*any* Flaw granting a Reputation) most likely wants
`excluded_if_holds: Vec<ItemPredicate>` sitting beside `incompatible_with`, or
a new `Prereq::Nor`-adjacent variant if it must also catch a *granted* Flaw
(B0/B15's F-466 reachability trap). **That shape is B0's to finish** — C0 only
fixes the vocabulary (`ItemPredicate`) both mechanisms draw from, per the
plan's explicit "co-designed with B0."

`flaw.false_power`'s domain stays whitelist-only (§ 6) — D34 says explicitly
not to route it through this predicate, and it is a closed, human-picked list
(2 ids), not a computed property.

---

## 8. D9 part 3 — the multi-valued parameter

**Type.** `ParamType::MultiRef` (§ 1). Stored value: `BTreeSet<Id>`, not
`Vec<Id>` — reusing the exact container `ParameterDef::at_most_one_of` already
uses for this reason. A `BTreeSet` makes `{A,B}` and `{B,A}` the same value by
construction (`PartialEq`/`Ord` on the container, not on write order), so
`max_per_target`'s duplicate-detection key `(item_ref, params)` needs **no
separate canonicalization step** — the canonical form falls out of the type
choice, which is the answer `CLAUDE.md`'s "sort arrays... before writing JSON"
convention was already pointing at.

**`Selection::params` value type** moves from `BTreeMap<String, Id>` to:

```rust
#[serde(untagged)]
enum ParamValue2 {   // name TBD at implementation; avoid colliding with § 1's ParamValue
    Single(Id),
    Multi(BTreeSet<Id>),
}
pub params: BTreeMap<String, ParamValue2>,
```

A bare-string existing value still deserializes as `Single` with **zero byte
change** — this is genuinely additive at the wire level for every save that
predates the type. The `SCHEMA_VERSION` bump exists for a narrower reason (§
below), not because old saves need folding.

**A new domain, not just a new type — and settled against the character's own
spells, not the catalogue.** ArMDE:5859-5863 (Corrupted Spells, verified
directly): *"The character has **learned at least 30 levels of formulaic
spells** from a source that has been corrupted in some way... You may only
take this Flaw once, though it can affect **as many of the character's
spells** as you wish."* Both clauses name spells the character has **already
learned** — the 30-level prerequisite only makes sense read against
`Entity::spells`, and "the character's spells" is not "any spell in the
rules" — so `ParameterDomain::Spell` resolves against **`Entity::spells`**
(the entity's own `SpellSelection` list, by `spell` id), not the ruleset's
whole spell catalogue. This is `require_possessed`'s "must be on the sheet"
idea (§ 0's table), ported to a domain with no point-item registry to check
against — so `ParameterDomain::Spell` is **inherently** possession-scoped
(there is no "any spell" reading to opt out of with a flag, unlike `Item`'s
optional `require_possessed`), and needs no sibling flag. Without the domain
at all, Corrupted Spells has a `MultiRef` type with nothing to validate its
members against; without the possession scoping, a save could name a spell
the character never learned, which the passage's own prerequisite rules out.

**Worked examples, one per Corrupted entry (D15):**

```json
// flaw.corrupted_abilities — ArMDE:5847-5852, "you can choose to have it affect multiple Abilities"
{ "parameters": [{ "key": "targets", "type": "multi_ref", "domain": "ability" }] }

// flaw.corrupted_arts — ArMDE:5853-5858, "it can affect multiple Arts"
{ "parameters": [{ "key": "targets", "type": "multi_ref", "domain": "art" }] }

// flaw.corrupted_spells — ArMDE:5859-5864, "as many of the character's spells as you wish"
{ "parameters": [{ "key": "targets", "type": "multi_ref", "domain": "spell" }] }
```

All three stay `uncomputed_rule` (D15) — the parameter records the choice for
save round-trip and Markdown export; the ±3/±5 arithmetic is not computed
(D15's own reasoning, orthogonal to this).

**`max_per_target` grouping** now works unmodified: two `Corrupted Abilities`
selections naming `{ability.awareness, ability.brawl}` and
`{ability.brawl, ability.awareness}` compare equal (same `BTreeSet`), so they
correctly collide as "the same target" under `max_per_target: 1` — matching
"you may only take this Flaw once, though it can affect multiple [x]"
(ArMDE:5851 et al.: one target *set* per character, not one entry per
Ability).

**F-504's shape decision — five keys, not `MultiRef`.** Restricted Learning's
five chosen Abilities have no precedent for a single multi-valued field (of
655 entries, only two carry more than one parameter, and in both the domains
*differ*). B17's recommendation stands and this note ratifies it: **`ability_1`
… `ability_5`, all domain `ability`, single-valued `Ref`** — five is the rule
(ArMDE-stated), not a catalogue size, so hardcoding the count is not a
`CLAUDE.md` violation. `MultiRef` would also be *wrong* here: the book asks for
five **independent** choices (each could legitimately repeat, or not, per its
own passage), not one set.

**F-42/F-63/F-317 do NOT use `MultiRef`.** Despite corrections.md's terse gloss
("a player-chosen restricted Ability group is unrepresentable"), each of the
three resolves to a mechanism already built in C1: F-42 (Custos) and F-317
(Templar Specialist) are the **exclusive-choice gate** (§ 3); F-63 (Enchanting
Ability) is the **parameter-bound `AbilityScoreGrantParam`** (§ 2). None needs
a set-valued parameter. **This note deliberately narrows the plan's C5c
grouping** — the three findings are scheduled together for convenience, not
because they share `MultiRef`'s machinery; flagging this now so C5c's
implementer does not build a sixth mechanism for entries that already have a
home.

### Migration (bump 18 → 19)

**Renumbered** (architect review of `design-cv-catalogued-values.md`): CV
(catalogued parameter values) now claims the 17 → 18 bump ahead of this slice,
so C5a's own bump moves to 18 → 19. No other content below changes — only the
version numbers.

**Implementation finding (C5a phase 2), overriding this section's original
"what must fold" premise: no canonicalizing fold is needed at all.** The
premise below assumed `MultiRef`'s canonical (sorted, deduplicated) form would
need code to enforce it. It does not: `SelectionParamValue::Multi` is a
`BTreeSet<Id>` (landed in C0b, ahead of this slice), and `BTreeSet`
deserialization already sorts and deduplicates on insert — `["ability.brawl",
"ability.awareness", "ability.brawl"]` parses straight into the canonical
two-element set `{ability.awareness, ability.brawl}` with zero lines of fold
code. The RED this section originally proposed for that shape is therefore
**already green the moment `ParamType::MultiRef` exists** — proven, not just
argued, by `crates/arm-rules/tests/c5a_multi_ref_parameter.rs`'s
`multi_values_built_in_either_order_serialize_identically_and_sorted` and
`two_selections_naming_the_same_set_in_different_order_collide_as_duplicates`.
**Why the bump is still owed even though nothing folds:** D9's own schema
criterion is about *validation*, not wire shape — a v19 save may contain a
`Multi` value under a key whose ruleset parameter is genuinely `multi_ref`,
and only a v19-aware validator knows to shape-check it
(`CODE_PARAM_WRONG_SHAPE`, added this slice). An older (pre-19) build has
neither the type nor the check, so it must refuse the load outright
(`entity.schema_version > SCHEMA_VERSION`) rather than silently accepting or
misreading that shape — the same forward-compatibility reasoning
`crate::migration::SCHEMA_VERSION`'s own doc comment gives for the 14 → 15 and
16 → 17 bumps. The bump is therefore a **pure version marker**: see
`crates/arm-rules/src/migration.rs::schema_version_is_19` and
`a_clean_schema_18_save_loads_unchanged_under_the_19_build`, which pins that a
save with nothing to migrate keeps its recorded version exactly as read,
becoming 19 only on the next save (`ruleset_io.rs::save_entity_to_path`'s
unconditional stamp) — precisely like every other marker-only bump (10 → 11,
11 → 12, 12 → 13) before it.

The original text below is kept for its still-relevant half — the
`CODE_PARAM_WRONG_SHAPE` reasoning — with only the canonicalization RED struck:

- ~~RED: a hand-crafted save with `"targets": ["ability.brawl", "ability.awareness", "ability.brawl"]`
  (unsorted, duplicated) must load with `targets == {ability.awareness, ability.brawl}`
  (a two-element `BTreeSet`) — fails today because the type does not exist.~~
  **Superseded above: this is already true with no fold, the moment the type exists.**
- RED: a save with `"targets": "ability.brawl"` (old-shape single value on a
  key that is now `multi_ref` in the ruleset) must be reported under a
  **new, distinct code — `CODE_PARAM_WRONG_SHAPE`, not `CODE_MISSING_PARAM`**.
  The two are different defects: `missing_param` means the key is **absent**
  (a choice not yet made); this value is **present** and parses (a valid
  `Single(Id)`), it is simply the wrong shape for a `multi_ref` slot — closer
  to `CODE_UNKNOWN_PARAM_VALUE`'s family (the value does not belong to this
  parameter's domain) than to an unmade choice. Reusing `missing_param` would
  tell the player "you haven't chosen yet" about a choice they made under the
  entry's *previous* (pre-`multi_ref`) shape, which is actively misleading.
  Proves, incidentally, that the untagged enum does not quietly reinterpret a
  scalar as a one-element set.
- GREEN once `ParamValue2` exists and the fold runs.

No other fold rides on this bump — D34/allow_ids, D33/exclude_if, D35/Number,
D48/instances are all `#[serde(default)]`-additive on the **ruleset** side,
which is not schema-versioned at all (only `Entity`/`Selection` saves are).

---

## 9. Integrity checks at load

Every new field gets a load-time check in `ruleset::integrity`, on the
existing pattern (a field is rejected when its paired `domain`/context cannot
use it — see § 0's table and the `validate_parameter_defs` excerpt read at
`ruleset/integrity.rs:2282-2408`):

| Field | Rejected when | Message names |
|---|---|---|
| `ParamGate.param` (on any `AbilityRef`/`CategoryRef`) | the named `param` is not a key the SAME item's `parameters` declares | the declaring item's id, the dangling `param` key |
| `ParamGate.param` | the named param's `param_type` **is** `MultiRef` — a gate reads exactly one value out of `Selection::params[param]` at fold time, and a multi-valued slot has no single value to read | item id, param key, the offending `MultiRef` param it points at |
| `ParamGate.param` | the named param's `domain` **is** `Number` — gating on numeric equality is a different, unaddressed feature no current ruling needs; if a future ruling needs it, it should compare against `min..=max` rather than a single `equals`, which is a new mechanism, not this one. **Rejected outright for now**, not silently accepted | item id, param key |
| `ParamGate.equals` | not a member of that parameter's own `values` (`Enumerated`) or not a valid `Realm`/`Id` for its domain | item id, param key, the bad `equals` value |
| `ParamValue::Bound{param}` (on `AbilityRef.instance` / `AbilityScoreGrantParam.instance`) | the named `param` is dangling (as `ParamGate.param` above), **or** its `param_type` is `MultiRef` (same one-value-per-read reason) | item id, field name, `param` key |
| **`AbilityRef` setting both `instance` and `gate` naming the SAME `param` key** — i.e. `instance == Bound{param: p}` **and** `gate.param == p` for the same `p` | one key cannot simultaneously *gate this entry's activation* and *supply its instance value* — the two roles conflict, and nothing in D14/D35/D48/W2 needs it. (A `Literal` instance alongside an unrelated `gate`, or a `Bound` instance keyed on a *different* param than the gate, are both fine and unaffected by this rule — Custos's own ungated Latin entry already shows `instance` with no `gate` at all) | item id, the shared `param` key |
| `allow_ids` (D34) | non-`item` domain (mirrors `require_categories`) | item id, param key, domain |
| `allow_ids` member | does not resolve to a real point item | item id, param key, offending id |
| `exclude_if` (D33) | non-`item` domain | item id, param key, domain |
| `ParamType::Number{min,max}` | `min > max`, or paired with a `domain` other than `Number` | item id, param key, min/max |
| `ParamType::MultiRef` | paired with a domain that itself makes no sense multi-valued (`Text`, `Number`) — reject the combination explicitly rather than leave it silently meaningless | item id, param key, domain |
| `Effect::ScaledRestrictedAbilityXp.param` | the named param's `param_type` is not `Number` specifically (a scaled-XP `param` must resolve to a count, not an id or a set) | item id, effect index, param key |
| `Effect::AbilityScoreGrantParam.instance`'s `Bound.param` | the named param's domain does not match the domain the *target ability itself* expects for its instance (e.g. `ability.enchanting_ability`'s medium is `text`-domain, so `Bound.param` must resolve to a `text`-domain parameter) — a mismatch here would silently never apply, the same reasoning `validate_deficient_art_effect` already applies to Technique/Form | item id, effect index, param key |
| `RestrictedAbilityXp.instances` / `AbilityAuthorization.abilities` entries | an `AbilityRef.ability` that does not resolve in the ability catalogue | item id, offending ability id |
| `CategoryRef.category` | not a member of `AbilityCategory` — already caught by serde at parse time (closed Rust enum), no separate check needed | — |
| whitelist/predicate/gate all empty but field present (e.g. `allow_ids: []`) | `Vec`/`BTreeSet::is_empty` already `skip_serializing_if`s these away on write; on **read** an explicit empty list is data noise, not an error — reject only if the intent is ambiguous (an empty `allow_ids` with empty `require_categories` too, i.e. a domain-`item` parameter with no narrowing at all declared through this mechanism) | item id, param key |

Every message names the offending id(s), per `CLAUDE.md`'s "fail loudly with
clear error listing offending IDs" and the existing `integrity.rs` style
(string-built, not typed errors — matches file convention).

---

## 10. Slice breakdown, C0b, C1–C5c

| Slice | Content | First failing test | New UI control | Bumps schema |
|---|---|---|---|---|
| **C0b** (new, architect review) | **`Selection::params` moves from `BTreeMap<String, Id>` to `BTreeMap<String, ParamValue2>` (`Single(Id) \| Multi(BTreeSet<Id>)`), producing only `Single` today.** Wire-compatible (an existing bare-string value still deserializes as `Single`), so **no schema bump** — the bump stays with C5a's array *fold*. TS mirror (`ui/src/lib/types.ts::Selection.params`) widens to `Record<string, string \| string[]>` in the **same slice**, before any consumer needs it. Exists solely so C1 is written once, against the final shape, rather than against a type C5a would later change under it | a red asserting a hand-written `"targets": "ability.brawl"` save still round-trips as `Single` and a *hypothetical* (not-yet-producible) array value round-trips as `Multi` — proves the type exists and both shapes parse, before any producer or consumer reads it | No — type-level change only, no picker behaviour changes (nothing produces `Multi` yet) | No |
| **C1** | D14 `AbilityRef`/`CategoryRef`/`ParamGate`/`ParamValue`, written **once** against C0b's final `Selection::params` shape; `covenant_upbringing` Latin instance; `student_of_realm` binding + `AbilityBonusGated` (+2 Lore, row 50a, kept as a sibling variant — decided, § 1a); W2 exclusive choice (`wise_one`, `custos`, F-42, F-317, its open set flagged in § 3); `RULES.md`'s "Documented approximation" note rewritten; `ability_bonus`'s new guarded arm (§ 1a site 2); `validate_effect_refs`'s new `AbilityAuthorization`/`RestrictedAbilityXp` arms (§ 1a site 13) | `an_authorizing_virtue_permits_one_ability_without_granting_xp`-style test asserting Covenant Upbringing does **not** authorize `ability.dead_language` at any instance but Latin; **and** a test on the actual serde error text for a malformed untagged `AbilityRef` (e.g. `{"ability": 5}` or an object missing `ability`) — pins today's message so a future serde/dependency bump that degrades it to a generic "data did not match any variant" is noticed, not silently accepted (`AbilityRef`'s `#[serde(untagged)]` is exactly the shape known to produce poor error text) | No (ruleset JSON + Rust only; `ParameterPicker.svelte` needs no change — no entry uses `gate` in a way the UI must newly render beyond the existing enumerated dropdown) | No |
| **C2** | D34 `allow_ids`; `flaw.false_power`/`_minor`'s domain narrowed from 227 to 56+2; **opportunistic doc fix**: `ParameterDef::max_per_value`'s doc comment still states the pre-D10 default of 255 — correct it to 1 while this slice already touches the struct's neighbouring doc comments | a test asserting `virtue.diedne_magic` resolves and an arbitrary other `hermetic` Virtue does not | No | No |
| **C3** | D35 `ParamType::Number`, `ParameterDomain::Number` (with its redundant-half doc comment, § 1), `ScaledRestrictedAbilityXp`; `virtue.simple_student`; the 6 no-op sites + `validate_effect_refs`'s new real arm for this variant (§ 1a) | a red asserting 1 finished year → 30 XP, 2 → 60, and the parameter rejects 3 | **Yes** — a number `<input>` picker (new UI control, no prior numeric parameter existed); `ParamType`/`ParameterDomain` TS mirrors gain `number` — **first slice where a Rust/TS parity test earns its keep** (see § 10.1) | No |
| **C4** | D48 `instances` field + union `pool_covers`; re-derive and confirm the 12 pools (measurements row 12); **`virtue.master_bard`'s two regression tests (§ 5)**; **refactor `ability_authorizations` into the shared `resolve_ability_refs` used by both the authorization fold and the pool-eligibility fold (§ 1a site 12), so there is one resolution path for gate/instance evaluation, not two that could drift** | a red on `virtue.marshal` funding Profession: Sailor today (should fail once instance-scoped); a green pinning `virtue.master_bard`'s `amount == 240` and its full 6-Ability set unchanged | No | No |
| **C5a** | D9 part 3 engine only: `ParamType::MultiRef`, `ParameterDomain::Spell` (character-scoped, § 8), `CODE_PARAM_WRONG_SHAPE`. **No canonicalizing migration exists or is needed** — `SelectionParamValue::Multi`'s `BTreeSet<Id>` (C0b) already sorts/dedupes on deserialize for free, so the 18 → 19 bump is a documented **no-op version marker** (see § 8's implementation-finding note) | the wrong-shape red in § 8, plus the three load-time integrity reds (a gate/bound naming a `MultiRef` param; `MultiRef` paired with `text`/`number`) | No (engine + migration only — C0b already moved the type, so no C1 consumer needs touching here) | **Yes — 18 → 19** (CV takes 17 → 18 ahead of this slice), pure version marker, no fold |
| **C5b** | Multi-select UI picker (`ParameterPicker.svelte`), keyed on `param.param_type == MultiRef`; `selection-workflow.svelte.ts` and `art-workflow.svelte.ts`'s param-write helpers widen to accept a set (§ 10.1) | a `.client.test.ts` mounting the picker and asserting a checked/unchecked toggle writes/removes a set member | **Yes** | No |
| **C5c** | D15's three Corrupted entries (multi-valued data); F-42/F-63/F-317 (reusing C1's gate — **not** `MultiRef`); **`AbilityScoreGrantParam` is introduced here** (F-63, § 1a) — it does not exist before this slice, so nothing in C1/C3/C4/C5a/C5b references it | per-entry reds: Corrupted Abilities round-trips a 2-element set; Custos authorizes exactly one of Martial/Academic/Arcane; Enchanting Ability grants a floor in the player-chosen medium instance | No | No |

**e2e:** per the plan's "at phase boundaries... and inside a slice only where
it changes the save format or IPC shape" — only **C5a** changes the save
format, so only C5a needs an e2e boundary run inside the C-group (in addition
to the group-boundary run after C5c, per the plan's own phase-boundary rule).

### 10.1 UI/TS mirror work, named per slice

Seven non-test UI files read or write `Selection.params` today:
`ui/src/lib/types.ts` (the TS mirror of `ParamType`/`ParameterDomain`/
`Selection.params` itself), `ParameterPicker.svelte` (the general picker and
its `write()` sink), `selection-workflow.svelte.ts` (the plain add/remove/
parameter surface the V/F picker drives — confirmed by its own header comment
as "not the only writer of `entity.selections`, deliberately"),
`art-workflow.svelte.ts::ArtWorkflow` (the same surface for an Art-domain
parameter specifically — relevant because `flaw.corrupted_arts` is
`domain: art`), `derive.ts` (`paramValueUsage`'s sibling-equality check,
`selectionCategories`, and the selection sort/compare used for canonical
ordering), `VirtueFlawTab.svelte` (renders a selection's name from its
params), and `HouseSelector.svelte`/`MythicCompanionTypeSelector.svelte`
(render a **granted** selection's params the same way, read-only).

| Slice | Files touched | What changes |
|---|---|---|
| **C0b** | `ui/src/lib/types.ts` | `Selection.params` widens to `Record<string, string \| string[]>`. Nothing else changes — no producer emits an array yet, so every other file's existing string-only logic still type-checks against the union's `string` half unchanged |
| **C1** | none | Gated/bound entries render through the picker's **existing** enumerated/text controls; `gate` and `instance` are invisible to the UI, which only ever sees a `ParameterDef` + its resolved option list, never the `Effect`-level constraint |
| **C2**, **C4** | none | Ruleset-internal narrowing (`allow_ids`) and pool-eligibility (`instances`) are both invisible to the picker — same reasoning as C1 |
| **C3** | `ui/src/lib/types.ts` (`ParamType`/`ParameterDomain` gain `number`/`Number`), `ParameterPicker.svelte` (new numeric `<input min max>` control) | First slice where the TS mirror must learn a genuinely new *kind*, not just widen a value type — see the parity-test question below |
| **C5a** | none | `ParameterDomain::Spell` and `ParamType::MultiRef` are engine/migration-only until C5b gives them a picker; C0b already covers the `params` value-type widening |
| **C5b** | `ParameterPicker.svelte` (multi-select control + `write()`'s signature widens from `Record<string, string>` to accept a `string[]` per key), `selection-workflow.svelte.ts` and `art-workflow.svelte.ts` (their param-write helpers currently do `{ ...params, [key]: trimmed }` — a single scalar per call — and must widen to write/toggle a set member), `derive.ts` (`paramValueUsage`'s sibling check and the selection sort/compare must treat a `Multi` value as a set, matching the engine's `BTreeSet` canonicalization so client-side grouping cannot disagree with server-side `max_per_target`), `VirtueFlawTab.svelte` (join multiple names for display, e.g. "Awareness, Brawl") | The one slice that is genuinely UI-heavy across most of the seven files |
| **C5c** | none beyond exercising C5b's control with real data (`AbilityScoreGrantParam`'s medium parameter is `domain: text`, already renderable by the existing text input — no new control) | — |

**Is a `prereq-parity.test.ts`-style Rust/TS parity test warranted?** **Yes, for
`ParamType` and `ParameterDomain` specifically** — these are exactly the kind
of closed, engine-owned taxonomy `CLAUDE.md`'s architecture invariant names
("the engine surfaces them... so there is a single source of truth; the UI
must not re-hardcode their values"), and `prereq-parity.test.ts` already
exists for the structurally identical risk on `Prereq`. **Not** for `Effect`
itself — the UI never receives a raw `Effect`, only a `ParameterDef` plus the
already-resolved option list an IPC command hands it, so there is no TS-side
enum for `Effect`'s variants to drift out of sync with. Introduce the test
when **C3** first adds a variant the TS side must also grow
(`ParamType::Number`/`ParameterDomain::Number` — C0b's change was a value-type
widening, not a new taxonomy member, so it does not yet need this), and extend
it in **C5a** for `MultiRef`/`Spell`.

---

## 11. Risks and open points

**Risks (implementation, not product):**

1. **`ability_authorizations()`'s exhaustive `match`** (`effective/xp.rs:382-469`)
   changes shape for two arms (`AbilityAuthorization`, `RestrictedAbilityXp`)
   and gains a new field (`instances` on the latter) — the compiler forces
   every call site, but the **fold semantics** (gate evaluation, instance
   resolution) are new logic the exhaustiveness check cannot verify is
   *correct*, only that it is *present*. C1's test plan must include the
   negative case (§ 3's "gate fails ⇒ contributes nothing") for every gated
   family, not just the positive one.
2. **Two structurally similar but distinct mechanisms** (`gate` = conditional
   list membership, `instance` = literal/bound value restriction) sit on the
   same `AbilityRef` type. An implementer reaching for "binding" from D14's
   prose alone could conflate them (as this note's own drafting nearly did for
   Student of Realm) — the design doc's § 3 "why a gate, not a substitution"
   argument should be re-read at C1's start, not just at review time.
3. **`BTreeSet<Id>` for `MultiRef`** solves canonicalization for free but means
   `ParamValue2`'s `Multi` variant has no room for a *repeated* value with
   different weight/count — not needed by any of the three Corrupted entries
   or F-504 (which uses five keys instead), but worth a one-line note in the
   type's doc comment so a future author does not "just add a `Vec`" beside it.
4. **C1↔C5a coupling, resolved by re-sequencing rather than by rework.**
   `AbilityRef`/`CategoryRef::active_for`/`resolved` (§ 4) and
   `AbilityScoreGrantParam`'s instance resolution (§ 2) all read
   `selection.params.get(key)` expecting a single `Id` back — the exact shape
   C5a would otherwise change out from under them. **Orchestrator decision:**
   a new micro-slice **C0b**, landing before C1, moves `Selection::params`
   from `BTreeMap<String, Id>` to `BTreeMap<String, ParamValue2>`
   (`Single(Id) | Multi(BTreeSet<Id>)`) **producing only `Single` values** —
   wire-compatible, no `SCHEMA_VERSION` bump (the bump stays with C5a's array
   *fold*, which is the part that actually needs migrating, per § 8). C1's
   gate/`Bound` consumers are then written **once**, against the final shape,
   and C5a no longer touches them at all (§ 10's C0b/C1/C5a rows). The residual
   risk is narrower: a future read site could still unwrap a `Multi` value
   carelessly rather than rejecting it — which is exactly what § 9's new
   integrity rule (`ParamGate.param`/`ParamValue::Bound.param` must name a
   **non-`MultiRef`** parameter) catches at load time, so a mismatch cannot
   reach the fold silently even though the ordering hazard itself is gone.

**Open points for Norbert (not decided here):**

- **Whether `ItemPredicate` belongs on `ParameterDef` at all, or should be a
  `Prereq`-family concept.** § 7 puts `exclude_if` on `ParameterDef` because
  D33 is explicitly a parameter-domain narrowing, not an item-level
  incompatibility — but B0 may find that D23's point-item-level predicate
  exclusion wants to share more machinery with `exclude_if` than this note
  anticipates (e.g. a single `fn item_satisfies(predicate, item) -> bool` used
  by both). That convergence is B0's to find; this note only guarantees the
  **enum** (`ItemPredicate`) is shared, not the calling convention around it.

---

## References loaded

`docs/vf-audit/phase-2-plan.md`; `docs/vf-audit/decisions.md` §§ D9, D10, D12,
D13, D14, D15, D23, D33, D34, D35, D48; `docs/vf-audit/corrections.md` §§ 2.5,
2.6, 2.7, 3.2, 3.2a, 3.7, 3.15, 3.15a, 3.15b, 3.17; `docs/vf-audit/measurements.md`
§ 8 rows 2, 7, 9, 12; `docs/open-todos.md` rows 45, 50, 52, 53;
`crates/arm-rules/src/types.rs` (`Id`, `ParamType`, `ParameterDomain`,
`ParameterDef`, `Effect`, `PointItem`, `Selection`, `SpellSelection`);
`crates/arm-rules/src/effective/xp.rs` (pools, `AbilityInstanceRef`,
`ability_authorizations`, `pool_covers`, `build_flow_pools`);
`crates/arm-rules/src/validation/authorization.rs`;
`crates/arm-rules/src/migration.rs` (`SCHEMA_VERSION`, fold pattern);
`crates/arm-rules/src/ruleset/integrity.rs` (`validate_parameter_defs`);
`ui/src/lib/components/ParameterPicker.svelte`; `ui/src/lib/types.ts`;
`rules/core/virtues_flaws.json` (`flaw.corrupted_abilities/_arts/_spells`,
`virtue.student_of_realm`, `flaw.covenant_upbringing`); rulebook source,
verified directly: ArMDE:3629-3634 (Custos), ArMDE:3747-3750 (Enchanting
(Ability)), ArMDE:4958-4963 (Simple Student), ArMDE:5052-5055 (Student of
(Realm)), ArMDE:5133-5136 (Templar Specialist), ArMDE:5257-5259 (Wise One),
ArMDE:5865-5867 (Covenant Upbringing), ArMDE:6080-6084 (False Power),
ArMDE:6146-6148 (Flawed Powers), ArMDE:4457-4462 (Master Bard),
ArMDE:5847-5864 (the three Corrupted entries, full passage).

## Verdict

COMPLETE — revised per the plan-reviewer's "approve after fixes" (round 1)
and the architect's "approve after fixes" (round 2). Ready for sign-off before
C1 starts.
