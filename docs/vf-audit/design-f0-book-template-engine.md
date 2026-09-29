# F0 — design: the K5 flag model, the K3 mount record, and the conditional-modifier shape

Design note for Phase 2 group F, slice F0. No code changed by this document.
Reviewed next by the plan-reviewer, then the architect, before F1 starts.

**Mandate.** `docs/vf-audit/phase-2-plan.md` Group F: K5 (`EquipmentSlot::equipped`
overloaded), K3 (mounted combat unmodelled), and — per D4/D58 — the
conditional-modifier shape for Berserk, Ways of the (Land), Cyclic Magic
(D52) and Special Circumstances. Sources: `docs/book-template-conformance.md`
§ K2/K3/K4/K5/B1/B2/MAG1/MAG7, `docs/open-todos.md` rows 48/49/52,
`docs/vf-audit/decisions.md` D4/D15/D52/D58/D61.

**Revision 2 (2026-09-28), after the plan-reviewer's REWORK verdict
(`tmp/f0-plan-review.md`): two BLOCKERs fixed, not patched.** (1) § 2c's
`circumstantial` flag/`ModifierFamily::Circumstantial` mechanism is deleted
outright — it was, as built, exactly the "sheet list of situational
modifiers" D61 records as "offered and **not** chosen." The five conditional
carriers now follow the **already-shipped D15 precedent**
(`flaw.corrupted_spells`): delete the conditional effect, reclassify per what
remains, full passage into `description` where the guards require it. This
moves F2 from new engine surface to mostly data + i18n. (2) The mounted-twin
design no longer doubles every `CombatLine` uniformly — the Knight's own
statblock prints no mounted Fist row, so the twin is now scoped to weapons
with a `min_strength` (excluding the three body attacks), confirmed against
the source and flagged as an open question with that recommendation, not
silently assumed. Two MAJOR fixes: `LoadoutState`'s derive list now matches
`EquipmentSlot`'s; F1's red-test claim is restated to what actually goes red
(no `deny_unknown_fields` exists, so the old red-test wording was wrong about
*why*). One MINOR: the id count is corrected to five. A dated summary table
closes the document.

**Revision 3 (2026-09-29), after the architect's "needs work" verdict on
Revision 2 (`tmp/f0-architect-review.md`) and Norbert's D66.** BLOCKER:
Revision 2's `Weapon::min_strength.is_none()` mounted-twin predicate is
dropped — `min_strength` documents an unrelated fact (can this weapon be
wielded at all), and reading a mounting exception out of it is exactly the
"rules meaning inferred from an unrelated field" pattern this project's own
reviews already reject (D52). **D66 (`decisions.md:831-842`) rules an
explicit weapon-catalogue field instead** — this revision names it
`Weapon::body_attack: bool`, justified in § 2b, set on `weapon.dodge`/
`weapon.fist`/`weapon.kick` only, with its own integrity check, TS mirror, and
`RULES.md` note that this reproduces the Knight's template rather than
stating a rule the passage itself contains. Two MAJOR fixes: (1) the great
sword is `two_handed`, so F1's fix widens `emitted` by **one** line, not two
— the "two" (mounted + on-foot) only exists after F2's twin lands on top;
(2) the Cyclic Magic guard red was mis-sequenced — `COMPUTED_ENTRY_COVERS_
WHOLE_PASSAGE` matches by id only and does not re-check its own prose claim,
so deleting the effect while leaving the exemption row in place produces
**no red at all**; the row must be **removed**, not left stale, before any
red appears. One MINOR: the F1 migration sketch now shows exactly where
`SCHEMA_VERSION` is stamped for the per-element fold shape (no single
top-level boolean exists here, unlike the four existing folds). Resolved
open questions are removed; a dated Revision 3 summary closes the document.

---

## 0. What already exists (read before designing on top of it)

| Piece | Where | Shape |
|---|---|---|
| `EquipmentSlot` | `types.rs:4204-4224` | `{ item: Id, equipped: bool, specialization_applies: bool }` — `equipped` gates BOTH Load (`encumbrance`) and whether a Combat row is emitted (`combat_totals`), post-K2 |
| `encumbrance` | `derived/combat.rs:69-84` | sums Load over `entity.equipment.iter().filter(|s| s.equipped)` only (K2, Norbert's call) |
| `combat_totals` | `derived/combat.rs:146-260` | one/two `CombatLine`s per slot in `entity.equipment.iter().filter(|s| s.equipped)`; shields likewise filtered on `s.equipped` |
| `soak` | `derived/combat.rs:303-323` | armor Protection summed over `.filter(|s| s.equipped)` |
| `CombatLine` | `derived/combat.rs:110-134` | `{ weapon, shields, ability, initiative, attack?, defense, damage?, range? }` — no notion of "mounted" today |
| `Effect::CombatMod` | `types.rs:1992-2012` | `{ amount, target: CombatStat, weapon: Option<Id> }` — folded **unconditionally** into `InPlayMods::combat_mods`/`weapon_combat_mods` |
| `Effect::SoakMod` | `types.rs:1983-1986` | `{ amount }` — folded unconditionally into `InPlayMods::soak_mod` |
| `Effect::CastingTotalMod` | `types.rs:1919-1924` | `{ amount, scope: CastingScope }` — folded unconditionally into `InPlayMods::casting_mods: Vec<(i32, CastingScope)>`, which becomes `CastingTotal::casting_mod_addends` |
| `in_play_mods` | `derived.rs:207-`(exhaustive match) | the single fold point every `Effect` variant must be classified at; the **existing precedent for "cannot fold into a flat number, so surface it"** is `MagicResistanceEffect::AuraBonus \| SusceptibleFaerie \| SusceptibleInfernal \| ConditionalPenetrationWaiver` (`derived.rs:331-343`), which push a `SurfacedModifier` instead of touching a flat total |
| `SurfacedModifier` / `ModifierFamily` | `derived.rs:664-747` | `{ family, detail, amount, factor?, source?, ability? }`; `ModifierFamily` is a closed, Fluent-rendered enum (`Aging`, `Advancement`, `SpecialCasting`, `AbilityRoll`, `HealthRoll`, `MagicResistance`) |
| `DerivedTotals.surfaced_modifiers` | `derived.rs:832` (aka the "surfaced modifiers section") | `Vec<SurfacedModifier>`; rendered by `ui/src/lib/components/DerivedSurfacedModifiersSection.svelte`, one `derived-surfaced-<family>` + `derived-detail-<detail>` Fluent pair per row |
| `SpecialCasting::Circumstantial` | `types.rs` enum variant | a **pre-existing, separate** shape — an unquantified quirk (`amount` always 0) listed for the player to read up, predating D61 and **not yet audited against it** (plan-reviewer's own Tech Debt Ledger note). Revision 1 of this note mistook it for a precedent to imitate; it is not the model for Group F's fix (§ 1a/§ 2c) and this note does not touch it |
| `ability.ride` | `rules/core/abilities.json:95` | ordinary Ability, no special modelling |
| `effective_ability_score` | `effective/ability.rs:131` | resolver already used everywhere an Ability's effective score is needed |
| `migration.rs` fold idiom | `migration.rs:712-790` | dispatch on a legacy key's **absence**/presence (never on `schema_version`), fold, remove the legacy key, stamp `SCHEMA_VERSION` — used four times already (`aging_reductions`, `talisman_attunements`, `ability_funding`, `saga_year`) |
| `SCHEMA_VERSION` | `migration.rs:158` | **19** today |
| `book_templates.rs` | `the_knight_matches_the_book` (:1201), `the_berserker_matches_the_book` (:1510), `the_bjornaer_matches_the_book` (:244), `the_mercere_matches_the_book` (:768) | fixtures `companion_knight.json`, `grog_berserker.json`, `magus_bjornaer.json`, `magus_mercere.json` — every "wrong" figure is pinned as an explicit expectation today and must flip to the book's own figure |
| `EquipmentTab.svelte` / `state.svelte.ts` | `EquipmentTab.svelte:146-155`, `state.svelte.ts:2314` | one checkbox `equipment-equipped-{i}` → `store.setEquipmentEquipped(index, boolean)` |
| `export/sections.rs` | `write_combat` (:557-587), `combat_line_name` (:593-601) | Markdown export mirrors `combat_totals()` row-for-row; `combat_line_name` already composes weapon+shields labels, the natural place for a "(mounted)" suffix |
| `DerivedCombatSection.svelte` | UI mirror of `write_combat` | same composability point on the frontend |

---

## 1. Findings, per entry

### K5 — `equipped` conflates two independent facts

- **Passage:** the Knight's five Combat rows include two for the great sword
  (`ArMDE:1468-1472` `#### The Knight`) while his printed Encumbrance
  ("2 (3)", `ArMDE:1484`) counts only the wielded set (K2). *"You may take this
  Virtue multiple times"* is not the relevant clause here — the relevant fact
  is that the book wants the stowed great sword to **yield Combat rows** and
  **not contribute Load**, and one boolean cannot express both.
- **Chosen shape:** replace the boolean with a **three-state closed enum**,
  `LoadoutState { Stowed, Carried, Wielded }` — a fixed rules taxonomy
  (CLAUDE.md: "fixed taxonomies... stay Rust enums"), not two independent
  booleans. Two booleans would admit a meaningless fourth combination
  (contributes Load but yields no row); the enum cannot.

### K3 — mounted combat has no engine counterpart

- **Passage:** *"A mounted character adds his Ride score, to a maximum of +3,
  to his Attack and Defense Totals, due to higher position and control of a
  large animal."* (`ArMDE:16839` `#### Mounted Combat`, full passage
  `ArMDE:16837-16839`). The Knight's mounted lines are exactly his on-foot
  ones plus +3/+3 at Ride 5 (`ArMDE:1468`, `:1470`).
- **Chosen shape:** an additive `Entity.mounted: bool` ("the mount record") plus
  a `min(Ride, 3)` term computed into a **second, mounted** `CombatLine` per
  existing line. Not a richer per-mount catalogue entry (a horse's identity,
  encumbrance, or type changes nothing mechanically per this passage) — YAGNI.

### Berserk / Ways of the (Land) / Cyclic Magic / Special Circumstances — conditional modifiers folded unconditionally

Covered together per row 49: "one class with four witnesses" (the four
row-49 witness Virtues/Flaws — Berserk, Ways of the Land, Cyclic Magic,
Special Circumstances). Cyclic Magic's Virtue and Flaw are two distinct
catalogue ids for one witness, so the count is **four witnesses, five
carrier ids**: `virtue.berserk`, `virtue.ways_of_the_land`,
`virtue.cyclic_magic_positive`, `flaw.cyclic_magic_negative`,
`virtue.special_circumstances` (all five confirmed present in
`rules/core/virtues_flaws.json`). See § 1a for the per-entry verdict.

---

## 1a. The D61 reconciliation (mandatory per the brief) — Revision 2: D15's actual mechanism, not a new one

D61's ruling is narrower and stronger than "surface it, unfolded": *"It stays
in `description` in both locales, with **no computed effect**"*
(`decisions.md:911-913`). D61 invokes D15 by name for this exact class. D15's
own already-shipped fix for the identical shape (`flaw.corrupted_spells`) is
not "surface the effect instead of folding it" — it is **delete the effect
and reclassify**: `classification` moved `in_play_effect` → `uncomputed_rule`,
and the effect was removed from the catalogue entirely (`decisions.md:313-316`;
confirmed in data — `flaw.corrupted_spells` carries `"classification":
"uncomputed_rule"` and no `effects` key). Revision 1's `circumstantial: bool`
mechanism kept the effect, kept the fold (just redirected it to
`DerivedTotals.surfaced_modifiers`), and rendered it through a new
`ModifierFamily::Circumstantial` row — that computed *and displayed* row is
exactly the "sheet list of situational modifiers" D61 declined. This
revision replaces it with the D15 treatment throughout.

**D4/D58's "extra row where the template prints one, otherwise narrowed out
of the base total" is not in tension with this — narrowing out of the base
total does not require a field to narrow **with**; it can mean the catalogue
carries no effect for that clause at all**, which is the stronger, D61-clean
reading. Checked against each entry's own template, none of Group F's five
carrier ids has a template that ever prints a boosted figure as a second row.
K3 is the one entry in this slice with such a precedent, and it is not a
Virtue/Flaw at all (no catalogue change).

| Entry | Effects: kept vs. deleted | Resulting classification | Why | Ruling |
|---|---|---|---|---|
| **K3** Mounted Combat | n/a — not a catalogue entry | n/a | The Knight's own template prints mounted rows *beside* the on-foot ones (`ArMDE:1468`, `:1470`) — the one book-template precedent for a dual-row reading | D58 (in-scope test: "if the sheet shows it, we compute it") |
| **`virtue.berserk`** | **Delete all 3** (`combat_mod`×2, `soak_mod`) — none survives | `in_play_effect` → **`uncomputed_rule`** | B2: the printed statblock is the **not-berserk** figure (Soak +9, not the engine's current +11); no template anywhere shows the "+2/-2/+2 while berserk" state. "Are you currently berserk" is D61's exact shape | D61; D15 (delete + reclassify, not surface) |
| **`virtue.ways_of_the_land`** | **Delete the only effect** (`casting_total_mod`) | `in_play_effect` → **`uncomputed_rule`** | MAG1: Bjornaer's printed Casting Totals (`ArMDE:1643-1650`) are the unconditional base — no dual-row precedent | D61; D4's own Lab-Total row for this Virtue already answers "no" |
| **`virtue.cyclic_magic_positive`** | **Delete `casting_total_mod`; keep `lab_total_mod`** (X7a's problem, untouched here) | **stays `in_play_effect`** (one effect still computed) | MAG1 (Mercere): printed Casting Totals are base, unboosted, no dual-row precedent, for the Casting term. The Lab term's own correctness is D52/X7a's separate, still-open question — this note does not resolve it, only avoids conflating the two | D61 for the deleted Casting clause; D4/D52 + plan's own X7a assignment for the untouched Lab clause |
| **`flaw.cyclic_magic_negative`** | **Delete `casting_total_mod`; keep `lab_total_mod`** (X7a's problem, untouched here) | **stays `in_play_effect`** | Same reasoning, mirrored | Same |
| **`virtue.special_circumstances`** | **Delete `casting_total_mod`; keep `magic_resistance_mod: aura_bonus`** (pre-existing, already-correct D58 family — untouched) | **stays `in_play_effect`** | MAG1 (Mercere): same base-only printed figure for the Casting term. `aura_bonus` is a *decidable* realm/aura fact (D58's pre-existing surfaced-MR family, predates and is not reopened by D61) | D61 for the deleted Casting clause; D58 for the untouched MR clause |

**The classification split is not ad hoc — it is `Classification`'s own
documented rule.** `types.rs:224-227`: *"Carrying an `Effect` is what
distinguishes [`UncomputedRule`] from [`InPlayEffect`]: an entry the engine
*does* compute something for is `in_play_effect` even if its passage also
contains an uncomputable clause."* Berserk and Ways of the Land end up with
**zero** remaining effects → `uncomputed_rule`. The three Cyclic/Special-
Circumstances entries keep a real, correct, already-wired effect each → stay
`in_play_effect`, with the now-uncomputed Casting clause carried by
`description` instead (§ 2c walks the exact guard consequences).

**Net effect:** no new `Effect` field, no new `ModifierFamily`, no new Fluent
keys, no new `derived.rs` branch. Two entries lose all mechanical wiring
(pure data + i18n); three entries lose one effect each and keep the other
untouched; K3 remains the only genuine engine addition in this slice.

---

## 2. New/changed types

### 2a. `LoadoutState` (new enum) + `EquipmentSlot.loadout` (replaces `equipped`)

```rust
/// How a piece of equipment is currently carried. A fixed rules taxonomy
/// (CLAUDE.md), replacing the K2-era `equipped: bool`, which conflated two
/// independent facts (K5): whether the slot yields a Combat row, and whether
/// it contributes Load.
///
/// Derives **matched to `EquipmentSlot`'s own stack** (`types.rs:4203`:
/// `Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize`) —
/// `EquipmentSlot` is kept sorted by `Entity::normalize` (its own doc comment,
/// `types.rs:4200`), which needs every field, `loadout` included, to be
/// `Ord`. `Default` is added because `#[default]` requires it (Revision 2,
/// MAJOR #3 — Revision 1's sample derived neither and would not compile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoadoutState {
    /// Not currently carried on the character's person: no Combat row, no Load.
    #[default]
    Stowed,
    /// Carried and wieldable, but not currently wielded: yields a Combat row,
    /// contributes no Load (the Knight's stowed great sword, K5).
    Carried,
    /// Actively wielded/worn: yields a Combat row AND contributes Load (K2's
    /// existing behavior, unchanged).
    Wielded,
}

pub struct EquipmentSlot {
    pub item: Id,
    #[serde(default, skip_serializing_if = "is_stowed")]
    pub loadout: LoadoutState,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub specialization_applies: bool,
}
```

`encumbrance`/`soak` filter on `loadout == Wielded` (byte-identical to today's
`equipped == true`). `combat_totals` filters on `loadout != Stowed` (Carried OR
Wielded yield a row) — this is the whole K5 fix; shields keep needing
`Wielded` (no book template shows a "carried but not readied" shield).

**TS mirror**, `ui/src/lib/types.ts`:

```ts
export type LoadoutState = 'stowed' | 'carried' | 'wielded';

export interface EquipmentSlot {
  item: string;
  loadout?: LoadoutState; // omitted = 'stowed'
  specialization_applies?: boolean;
}
```

**Match-site table** (every place `.equipped`/`EquipmentSlot::equipped` is
read today, all of which move to `.loadout`):

| Site | Rust | Today's filter | New filter |
|---|---|---|---|
| Load | `derived/combat.rs::encumbrance` | `s.equipped` | `s.loadout == LoadoutState::Wielded` |
| Armor Soak | `derived/combat.rs::soak` | `s.equipped` | `s.loadout == LoadoutState::Wielded` |
| Combat rows | `derived/combat.rs::combat_totals` (slot loop) | `s.equipped` | `s.loadout != LoadoutState::Stowed` |
| Shield set | `derived/combat.rs::combat_totals` (`shield_ids`) | `s.equipped` | `s.loadout == LoadoutState::Wielded` |
| Specialization toggle gate | `EquipmentTab.svelte::specializationApplicable` | `slot.equipped` | `slot.loadout !== 'stowed'` (a Carried weapon's specialty still matters on its Combat row) |
| Equip control | `EquipmentTab.svelte` checkbox, `state.svelte.ts::setEquipmentEquipped` | boolean toggle | tri-state control, `store.setEquipmentLoadout(index, LoadoutState)` |
| Export | `export/sections.rs::write_combat`/`combat_line_name` | reads `combat_totals()` output, unaffected by the enum itself | no change needed beyond what F2's `mounted` suffix already touches |

### 2b. `Entity.mounted` (new field, "the K3 mount record") + `CombatLine.mounted`

```rust
pub struct Entity {
    // ...
    /// Whether the character is currently fighting mounted (K3). Additive:
    /// absent/false means every existing save's behavior is unchanged (no
    /// mounted lines, as today). Source: ArMDE:16837-16839.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub mounted: bool,
}
```

```rust
pub struct CombatLine {
    // ...existing fields unchanged...
    /// True for the mounted variant of this line (K3): `attack`/`defense` add
    /// `min(Ride, 3)`; `initiative`/`damage` are untouched, per the passage.
    /// Never part of a save — `CombatLine` is a derived read-out.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub mounted: bool,
}
```

**Revision 2 (BLOCKER #2 fix): the mounted twin is NOT emitted uniformly.**
The passage itself (`ArMDE:16837-16839`) states no exception, but the one
template that actually prints mounted rows — the Knight's — settles the
question by example: **five** rows, long sword+shield (mounted, on-foot),
great sword (mounted, on-foot), and a single **unmounted-only** Fist row
(`ArMDE:1467-1472`, re-verified verbatim for this revision). No mounted Fist
row exists anywhere in the book. Revision 1's uniform doubling would emit one,
which `the_knight_matches_the_book` (the test that exists specifically to
catch this) would then fail to reconcile against the book.

**Revision 3 (BLOCKER fix, D66): `min_strength` is dropped as the predicate.**
The architect's review (`tmp/f0-architect-review.md` finding #1) is right:
`min_strength` documents an unrelated fact — *"the minimum Strength score
needed to wield the weapon"* (`equipment.rs:95-98`) — and the three body
attacks lacking it is a **coincidence of this one data snapshot**, not a rule.
Reading a mounted-combat exception out of it is the same "meaning from an
unrelated field" mistake D52 already named once in this project. D66
(`decisions.md:831-842`, Norbert, 2026-09-29) rules explicitly against it:
*"An explicit weapon-catalogue field marks the lines that get no mounted
twin... The engine reads that field, never `min_strength` or any other
unrelated property."*

**New field: `Weapon::body_attack: bool`.** Name justified: it reuses a term
this codebase *already* uses for exactly this set — `min_strength`'s own doc
comment independently calls Dodge/Fist/Kick "body attacks" — so the field
introduces no new vocabulary, only a purpose-named carrier for it. It states
a fact about the weapon (an unarmed strike, not an implement) rather than a
narrow single-purpose presentation toggle (rejecting the alternative
`mounted_twin: bool` name for that reason: a field whose only conceivable
reader is one `if` in `combat_totals` invites the next unrelated reader to
misuse it exactly as `min_strength` was just misused). Default `false`,
`skip_serializing_if` for canonical JSON (only 3 of 32 entries need the key
at all) — additive to `rules/core/equipment.json` (ruleset data, not a save,
so **no `SCHEMA_VERSION` impact**, matching § 6's existing no-bump-for-F2
reasoning).

```rust
pub struct Weapon {
    // ...existing fields unchanged...
    /// True for an unarmed strike (Dodge, Fist, Kick) with no weapon in hand.
    /// Its ONLY consumer is K3's mounted-twin gate in `combat_totals`
    /// (D66) — it must never be read as a proxy for anything else
    /// (min_strength already exists for "can this be wielded at all," a
    /// different question). Default false; only the three body attacks set
    /// it. This reproduces the Knight's own printed template (five rows, no
    /// mounted Fist), not a rule the Mounted Combat passage itself states —
    /// see the `RULES.md` note under Mounted Combat.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub body_attack: bool,
}
```

```jsonc
// rules/core/equipment.json — the only three entries that change
{ "id": "weapon.dodge", "body_attack": true, /* ...unchanged fields... */ },
{ "id": "weapon.fist", "body_attack": true, /* ...unchanged fields... */ },
{ "id": "weapon.kick", "body_attack": true, /* ...unchanged fields... */ }
```

```rust
if entity.mounted {
    let ride = effective_ability_score(entity, ruleset, &Id::new("ability.ride"), None)
        .clamp(0, 3);
    let mounted_twin = |l: &CombatLine| CombatLine {
        attack: l.attack.map(|a| a + ride),
        defense: l.defense + ride,
        mounted: true,
        ..l.clone()
    };
    // D66: gated on the explicit `body_attack` flag alone, never `min_strength`.
    let twins: Vec<CombatLine> = out
        .iter()
        .filter(|l| ruleset.weapon(&l.weapon).is_some_and(|w| !w.body_attack))
        .map(mounted_twin)
        .collect();
    out.extend(twins);
}
```

(Illustrative — the implementer writes the real, borrow-checker-clean form;
the `.clamp(0, 3)` is load-bearing: "to a maximum of +3" and never negative.)

**Integrity check (D66's "specify the integrity check"):** a data-integrity
test, alongside the existing weapon-catalogue tests in `equipment.rs`,
asserting `weapon.dodge`/`weapon.fist`/`weapon.kick` each carry
`body_attack == true` and no other entry does — a **structural invariant on
named items**, not a catalogue-size assertion (CLAUDE.md: "tests assert
structural invariants... never exact catalogue totals"), so a future weapon
addition cannot silently drift the set without a test naming it explicitly.

**TS mirror:**

```ts
export interface Weapon {
  // ...existing fields...
  body_attack?: boolean;
}
```

**`RULES.md` note (D66's explicit requirement):** under *Mounted Combat*,
record that the Fist/Kick/Dodge exclusion is **not** stated by
`ArMDE:16837-16839` — it reproduces the one template that shows mounted
rows (the Knight's) rather than encoding a rule the passage itself contains,
exactly the "(d) unsettled, resolved as a presentation choice" shape K3 and
K5 already used elsewhere in this document.

**TS mirror:**

```ts
export interface Entity {
  // ...
  mounted?: boolean;
}
export interface CombatLine {
  // ...existing fields...
  mounted?: boolean;
}
```

**Match-site table:**

| Site | Change |
|---|---|
| `derived/combat.rs::combat_totals` | appends mounted twins as above |
| `export/sections.rs::combat_line_name` | append a localized "(mounted)" suffix when `line.mounted` |
| `DerivedCombatSection.svelte` | same suffix, client-side |
| `EquipmentTab.svelte` or a new `Identity`/`Combat` panel toggle | one new checkbox, `mounted-label` → `store.setMounted(boolean)` |

### 2c. No new `Effect` field. D15 treatment: delete + reclassify (Revision 2)

**Revision 1's `circumstantial: bool` field, `ModifierFamily::Circumstantial`,
and its five new Fluent keys are dropped entirely — BLOCKER #1.** They are not
part of this design. The fix is a **data change to five catalogue entries**
in `rules/core/virtues_flaws.json`, following D15's already-shipped shape
exactly (`flaw.corrupted_spells`, `decisions.md:313-316`): delete the
conditional effect(s), set `classification` to whatever the *remaining*
effects justify (per `types.rs:224-227`, quoted in § 1a), and ensure the
passage's dropped clause still reaches the player through `description` —
required by name at two different existing guards in
`crates/arm-rules/tests/uncomputed_clauses.rs`, both of which this data
change must keep green:

| Entry | Effect(s) deleted | Effect(s) kept | New `classification` | Guard(s) this change must satisfy |
|---|---|---|---|---|
| `virtue.berserk` | `combat_mod` ×2, `soak_mod` (all three) | none | `uncomputed_rule` | `every_uncomputed_rule_entry_states_its_rule_in_every_locale` (:913) — **today's `summary`** ("You are capable of entering a blinding rage...") **states no number**, so a `description` with the full passage (`ArMDE:3500-3503`) is *mandatory* in both locales, not optional |
| `virtue.ways_of_the_land` | `casting_total_mod` (the only one) | none | `uncomputed_rule` | Same guard — but **already satisfied**: `rules/i18n/{en,de}/virtues_flaws.json` already carry a full, verbatim `description` in both locales (`en:2767`, `de:2770`) from before this note existed. Zero i18n authoring needed; only the catalogue's `effects`/`classification` change |
| `virtue.cyclic_magic_positive` | `casting_total_mod` | `lab_total_mod` (X7a's problem, untouched) | **stays `in_play_effect`** | `no_swept_entry_drops_an_uncomputed_mechanical_clause` (:2678) — its `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` exemption row (`uncomputed_clauses.rs:1976-1980`, *"both stated bonuses... are computed via two effects"*) becomes **false** once only one effect remains, and must be rewritten (or removed) in the same change, with a `description` added (today it has none — `en:1701` is `summary`-only) covering at least the now-uncomputed Casting clause |
| `flaw.cyclic_magic_negative` | `casting_total_mod` | `lab_total_mod` (X7a's problem, untouched) | **stays `in_play_effect`** | Same guard, same exemption-row problem, mirrored (`uncomputed_clauses.rs:1836-1840`); same missing-`description` gap (`en:298-301`) |
| `virtue.special_circumstances` | `casting_total_mod` | `magic_resistance_mod: aura_bonus` (untouched, already-correct D58 family) | **stays `in_play_effect`** | **Already satisfied without any exemption row**: its `summary` (`en:2589`) already states *"gaining a +3 bonus to your Casting Scores and Magic Resistance"* — a mechanical phrase the screen accepts as-is. A `description` is recommended for consistency with the other four (and because `summary`'s number is about to describe an effect the catalogue no longer carries), but is not guard-mandatory |

**Nothing in `derived.rs`, `types.rs`, or any Fluent file changes for this
part of F2.** `combat_totals`, `soak`, and `casting.rs`'s fold already do the
right thing the moment the data no longer carries the deleted effects — the
Berserker's Soak/Attack/Defense and the Bjornaer's/Mercere's Casting Totals
drop back to the book's own printed figures with **zero** engine code
touched. This is the "materially smaller" redirection the plan-reviewer's
Estimation Adjustment called for.

**`ui/src/lib/types.ts::Effect`, `ui/src/lib/effect-parity.test.ts`: no
change.** No tag, no field, no variant is added or removed — the `Effect`
union stays byte-identical. The TS mirror only ever needed to change for a
new/changed serde shape, and there is none here.

### 2d. JSON examples

**Ruleset (`rules/core/virtues_flaws.json`), before/after — `virtue.berserk`:**

```jsonc
// before
"classification": "in_play_effect",
"effects": [
  { "type": "combat_mod", "amount": 2, "target": "attack" },
  { "type": "combat_mod", "amount": -2, "target": "defense" },
  { "type": "soak_mod", "amount": 2 }
]
// after — D15 shape: delete + reclassify, no `effects` key at all
"classification": "uncomputed_rule"
```

**`virtue.ways_of_the_land`**: identical treatment — its one `casting_total_mod`
effect and the `effects` key are deleted; `classification` becomes
`uncomputed_rule`. Its `description` (both locales) already exists and needs
no authoring.

**`virtue.cyclic_magic_positive`** (and `flaw.cyclic_magic_negative`,
mirrored):

```jsonc
// before
"classification": "in_play_effect",
"effects": [
  { "type": "casting_total_mod", "amount": 3, "scope": "all" },
  { "type": "lab_total_mod", "amount": 3 }
]
// after — only the Casting term is deleted; classification is unchanged
// because lab_total_mod still computes something (X7a owns its correctness)
"classification": "in_play_effect",
"effects": [
  { "type": "lab_total_mod", "amount": 3 }
]
```

**`virtue.special_circumstances`**: same pattern — `casting_total_mod` deleted,
`magic_resistance_mod: aura_bonus` kept, `classification` unchanged
(`in_play_effect`).

```jsonc
// before
"effects": [
  { "type": "casting_total_mod", "amount": 3, "scope": "all" },
  { "type": "magic_resistance_mod", "kind": "aura_bonus" }
]
// after
"effects": [
  { "type": "magic_resistance_mod", "kind": "aura_bonus" }
]
```

**Save, before/after — an equipment slot:**

```jsonc
// schema ≤ 19
{ "item": "weapon.sword_great", "equipped": false }
// schema 20
{ "item": "weapon.sword_great", "loadout": "carried" }
```

```jsonc
// schema 20, mounted
{ "mounted": true }   // new top-level Entity key, additive
```

---

## 3. Exhaustive-match sites

- **`Effect` match in `derived.rs::in_play_mods`**: **unaffected by F2**
  (Revision 2). No `Effect` field or variant changes, so no match arm changes
  — the five catalogue entries simply stop carrying the effects that used to
  reach `combat_mods`/`soak_mod`/`casting_mods`; the existing arms keep
  running, just over less data for those five ids.
- **`LoadoutState`**: a plain 3-variant enum with no consumer doing an
  exhaustive `match` on it yet (today's consumers are boolean-equality
  filters, per § 2a's table) — introducing it does not force any *other* file
  to change beyond that table.
- **`Entity.mounted`**: a new struct field, not a new enum variant — no
  exhaustive match is affected. `combat_totals` is the only reader.
- **`Weapon::body_attack`**: a new struct field (Revision 3/D66), not a new
  enum variant — no exhaustive match is affected. `combat_totals`'s
  mounted-twin filter is its only reader.

---

## 4. Evaluation and display

| Total | What changes |
|---|---|
| Encumbrance (`EncumbranceTotal`) | filter becomes `loadout == Wielded`; figure is byte-identical to today for every existing save once migrated (Wielded ≡ old `equipped: true`) |
| Combat rows (`Vec<CombatLine>`) | Carried slots now yield rows (K5 fix); `entity.mounted` doubles the row count, tagging the new half `mounted: true` |
| Soak | filter becomes `loadout == Wielded`; unchanged in value |
| Casting Totals | the deleted `casting_total_mod` entries (Ways of the Land, Cyclic ×2, Special Circumstances) no longer reach `casting_mod_addends`/`formulaic`/`ritual`/`spontaneous_*` — Bjornaer's and the Mercere's Casting Totals drop back to the book's own printed base figures. The Mercere's surviving `lab_total_mod` pair (Cyclic ±) is untouched, X7a's problem |
| Soak (Berserk) / Attack / Defense | the deleted `combat_mod`/`soak_mod` entries no longer reach `InPlayMods::soak_mod`/`combat_mods` — the Berserker's Soak and Pole Axe/Kick lines drop back to the book's printed not-berserk figures |
| Surfaced modifiers list | **no new rows** (Revision 2 drops the `Circumstantial` family). A deleted effect is gone from the character's mechanics, not relisted as text-in-the-derived-panel — its rule lives in the catalogue entry's own `description`/`summary`, read wherever the player is looking at *that Virtue*, not in the always-present derived-totals sidebar |

**Markdown export** (D65/N1): reclassification changes what N1's rule
selects for **two** of the five entries. `virtue.berserk` and
`virtue.ways_of_the_land` move to `uncomputed_rule`, so N1 now gives them
`description ?? summary` (their full rule text) instead of the `summary` a
computed entry gets — this is the *intended* effect of the reclassification,
not a side effect to route around. The three entries staying `in_play_effect`
(`virtue.cyclic_magic_positive`, `flaw.cyclic_magic_negative`,
`virtue.special_circumstances`) keep showing their `summary` under N1,
unchanged — which is exactly why § 2c requires their `description` (where
missing) to state the now-uncomputed Casting clause: N1's export path will
never surface it for these three, but the in-app tooltip
(`VirtueFlawTab.svelte:121`, `entry?.description ?? entry?.summary`, gated on
nothing) always will, so a `description` addition is not wasted even though
export does not consume it here.

---

## 5. Integrity checks

- **`LoadoutState`**: closed 3-variant enum; an unknown string fails to
  deserialize in both save and ruleset JSON — no additional load-time
  validator needed.
- **The five reclassified/effect-trimmed catalogue entries are gated by
  `crates/arm-rules/tests/uncomputed_clauses.rs`, which already exists and
  does the integrity job here — no new test file, no new mechanism (Revision
  2, replacing the "new `derived.rs` unit test" this section previously
  proposed):**
  - `every_uncomputed_rule_entry_states_its_rule_in_every_locale` (:913)
    catalogue-wide gate for `virtue.berserk`/`virtue.ways_of_the_land` once
    reclassified: their displayed text (`description ?? summary`) must state
    a mechanical rule (a signed number, a botch-dice term, ...) in **every**
    shipped locale. Berserk's current `summary` alone does not qualify — a
    `description` is mandatory, not optional, for this guard to stay green.
  - `no_swept_entry_drops_an_uncomputed_mechanical_clause` (:2678) — the
    guard for entries that stay `creation_effect`/`in_play_effect` but no
    longer have their *whole* passage covered by effects. This is the one
    that governs `virtue.cyclic_magic_positive`, `flaw.cyclic_magic_negative`,
    and (in principle, though already satisfied via `summary`)
    `virtue.special_circumstances`.
  - `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` (:1822) rows for
    `flaw.cyclic_magic_negative` (:1836-1840) and `virtue.cyclic_magic_positive`
    (:1976-1980) currently assert *"both stated bonuses... are computed via
    two effects"* — **false** the moment `casting_total_mod` is deleted, and
    the guard's own self-check
    (`exempted_entries_still_trip_the_screen`-style tests at :2853+) exists
    precisely to catch a stale row. Both rows must be rewritten in the same
    change (narrowed to "the Lab Total clause is computed via `lab_total_mod`;
    the Casting Total clause is not — see `description`") or removed if the
    added `description` alone is enough to satisfy
    `no_swept_entry_drops_an_uncomputed_mechanical_clause` without an
    exemption at all.
- **`Entity.mounted`**: plain bool, no integrity implication.
- **`Weapon::body_attack`** (Revision 3/D66): a targeted data-integrity test
  asserting exactly `weapon.dodge`/`weapon.fist`/`weapon.kick` carry
  `body_attack == true`, per § 2b — a structural invariant on named items,
  not a catalogue-size assertion.
- **No new referential edges, no new ids, no new Fluent keys.** Nothing here
  touches `ruleset::integrity`'s `has`/`incompatible_with`/`ref` resolution,
  and `ui/src/lib/i18n.test.ts`'s completeness guard has nothing new to cover.

---

## 6. Saves — schema impact

**One bump for F1 (`equipped` → `loadout`), zero for F2.** Per the plan's own
criterion ("the saved shape or meaning moves → bump... F1 (`equipped` changes
meaning)... F2 K3 mount record do[es] not, unless F0 finds otherwise" — this
note does not find otherwise):

| Slice | What moves | Bump? | Why |
|---|---|---|---|
| **F1** | `EquipmentSlot.equipped: bool` → `EquipmentSlot.loadout: LoadoutState` | **Yes, 19 → 20** | The field is renamed AND its represented values change shape (bool → 3-state enum) — a genuine meaning change, not an additive default |
| **F2** | `Entity.mounted: bool` (new) | **No** | Byte-compatible additive `serde(default)`, absent ≡ `false` ≡ today's behavior exactly, the same shape as `EquipmentSlot.specialization_applies`'s own addition |
| **F2** | Five `rules/core/virtues_flaws.json` entries lose effects / change `classification` | **No** | Ruleset data, never a save — `SCHEMA_VERSION` governs `Entity` shape only. Confirmed independent of BLOCKER #1's fix: deleting effects and reclassifying is, if anything, an even smaller/safer change than the dropped `circumstantial` mechanism, and touches no save field either way |

**F1's migration (test-first, untrusted `schema_version`), following the
existing dispatch-on-absence idiom exactly (`migration.rs:712-790`):**

```rust
// Revision 3 (MINOR #4 fix): the four EXISTING folds each capture a single
// top-level boolean BEFORE deserialization (`funding_absent`,
// `saga_year_absent`) and dispatch `entity.schema_version = SCHEMA_VERSION`
// on it afterward. There is no single top-level key here — the legacy shape
// is per equipment-array-ELEMENT — so this fold generalizes the same idiom
// one level down: capture, before the loop, whether ANY element still lacks
// `loadout` (this is the meaning-changing case the F1 bump exists for), then
// stamp once after the loop on that captured flag. `wrap_legacy_ability_
// parameters`'s per-element walk is a DIFFERENT case (value-driven,
// idempotent, deliberately version-less, migration.rs:659-689) and is not
// the precedent to follow here — it never represents a save-shape MEANING
// change, only a presentation normalization of the current shape.
let equipment_loadout_absent = value["equipment"]
    .as_array()
    .into_iter()
    .flatten()
    .any(|slot| slot.get("loadout").is_none());

for slot in value["equipment"].as_array_mut().into_iter().flatten() {
    let Some(obj) = slot.as_object_mut() else { continue };
    if obj.contains_key("loadout") {
        continue; // already current shape — untouched, matching every sibling fold
    }
    let was_equipped = obj.get("equipped").and_then(Value::as_bool).unwrap_or(false);
    obj.remove("equipped");
    obj.insert(
        "loadout".into(),
        json!(if was_equipped { "wielded" } else { "stowed" }),
    );
}

// (after deserializing `value` into `entity`, alongside the other
// `if <x>_absent { ...; entity.schema_version = SCHEMA_VERSION; }` blocks:)
if equipment_loadout_absent {
    entity.schema_version = SCHEMA_VERSION;
}
```

**What an old save becomes:** every equipped-`true` slot → `loadout: "wielded"`
(identical derived figures); every equipped-`false`-or-absent slot →
`loadout: "stowed"` (identical derived figures — the post-K2 behavior). No
existing save's *derived output* changes as a result of the migration itself;
only newly-authored `"carried"` values (which no old save can contain) unlock
the K5 fix.

**Red test, restated precisely (Revision 2, MAJOR #4 — the previous wording
conflated two different reds and the runtime half was wrong).** Neither
`EquipmentSlot` nor `Entity` carries `#[serde(deny_unknown_fields)]` (verified
by grep: the only such attribute in `types.rs`, at :4062, is on an unrelated
enum), so serde silently drops an unrecognized JSON key by default. Two
genuinely distinct reds, in order:

1. **Compile-time red.** Renaming `EquipmentSlot::equipped` to `loadout:
   LoadoutState` breaks every existing call site that reads `.equipped`
   (`derived/combat.rs`'s three filters, `EquipmentTab.svelte`'s TS mirror is
   separate). This is the legitimate "first red" CLAUDE.md's TDD note allows
   for a signature that does not exist yet — it must still be followed by
   seeing the *real* assertion fail once the code compiles again.
2. **Assertion-level red, the one that actually exercises the fold.** Take a
   schema-19 fixture `{"item": "weapon.sword_great", "equipped": true}` with
   no `loadout` key. Run it through `serde_json::from_value::<Entity>`
   *before the migration fold exists*: this **succeeds** (not an error) —
   `loadout` is filled by its `#[serde(default)]` (`Stowed`), and the JSON's
   `"equipped": true` is silently ignored, present or not. The red is the
   **assertion that follows**: `loadout == LoadoutState::Wielded` fails
   (actual: `Stowed`), because nothing yet reads the legacy key. Once the
   fold in § 6's code sketch lands, the same input produces `Wielded` and the
   assertion goes green. A second fixture with `equipped` absent entirely
   proves the `Stowed` default (already green today, since it needs no fold —
   recorded as a control case, not a red). A third fixture already at 20 with
   `loadout: "carried"` proves the fold is a no-op (idempotent, matching every
   sibling fold's own test) — `obj.contains_key("loadout")` short-circuits.
   A fourth, **not previously covered**, fixture carrying *both* keys
   (`{"equipped": true, "loadout": "carried"}`, e.g. hand-edited or a
   partial migration) proves the fold's documented, deliberate behavior: it
   is a no-op (dispatches on `loadout`'s presence, not `equipped`'s), so the
   stray legacy key is dropped on the next save and `loadout` keeps its own
   value (`Carried`, unaffected by the ignored `equipped: true`) — the same
   "a hand-edited save may carry either shape" tolerance the four existing
   sibling folds already have, not a new gap this design introduces.

**F1 and F2 cannot be one bump.** The plan rule is explicit ("never two per
slice") and the two changes are independently deployable: F1's rename needs a
migration; F2's two additions need none. Bumping once in F1 and shipping F2
on the same `SCHEMA_VERSION` (20) is correct and is what "F2... do not [bump]"
already presumes.

---

## 7. Data vs engine (Phase 3 deferred)

**Revision 2: F2 is now almost entirely data.** The only remaining *engine*
work in F2 is K3 (mount record + the mounted-twin pass); the four/five
conditional-modifier ids are a catalogue-JSON + i18n change with zero
`derived.rs`/`types.rs` touch, confirming the plan-reviewer's Estimation
Adjustment.

| Slice | Engine/type changes | Data changes | Deferred |
|---|---|---|---|
| **F1** | `LoadoutState` enum, `EquipmentSlot.loadout`, `derived/combat.rs` filters (3 sites), `migration.rs` fold + test, `EquipmentTab.svelte` tri-state control, `state.svelte.ts::setEquipmentLoadout` | fixture-only: `companion_knight.json`'s great sword moves from `equipped: false` to `loadout: "carried"` | — |
| **F2 (K3)** | `Entity.mounted`, `CombatLine.mounted`, `Weapon::body_attack` (new catalogue field, D66) + its integrity test, `combat_totals`'s mounted-twin pass (gated on `!w.body_attack`), mount UI toggle, `RULES.md` note (D66) | `rules/core/equipment.json`: `body_attack: true` on `weapon.dodge`/`weapon.fist`/`weapon.kick`; fixture: `companion_knight.json` gains `mounted: true` | — |
| **F2 (conditional modifiers)** | **none** — no `Effect` field, no new match arm, no new `ModifierFamily`, no new Fluent key | delete effects + reclassify (`virtue.berserk`, `virtue.ways_of_the_land`); delete one effect, keep classification (`virtue.cyclic_magic_positive`, `flaw.cyclic_magic_negative`, `virtue.special_circumstances`); rewrite/remove the two stale `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` rows; author `description` for Berserk (both locales — new prose) and the two Cyclic entries (both locales — new prose); fixtures `grog_berserker.json`, `magus_bjornaer.json`, `magus_mercere.json` updated to the book's own figures | **Ways of the Land / Cyclic / Special Circumstances' missing `combat_mod` clause** ("+3 bonus to all rolls, including combat...", `ArMDE:5233`) is a *different* defect (a missing effect, not an unconditional one) and is **not** in F2's scope — § 9 Q1; **Cyclic Magic's Lab Total term** stays exactly as D4/D52 left it, owned by **X7a** |
| **Row 45 (X1)** | — | Berserk's missing `ability_authorization` (Martial) is explicitly out of scope here per the plan ("the authorization half folds into row 45") | X1 |

---

## 8. Slice table

| Slice | Red tests (what fails today and why) | Dependencies | e2e/bump | UI strings (en/de) |
|---|---|---|---|---|
| **F1** | (1) `EquipmentSlot` gains `loadout`, loses `equipped` — every existing call site referencing `.equipped` fails to **compile** until updated (the legitimate "first red," per CLAUDE.md's TDD note on compile-fail reds); (2) `derived/combat.rs::only_equipped_gear_counts_toward_load`-style unit test asserting `Carried` yields a row + zero Load; (3) `the_knight_matches_the_book` — the `emitted` assertion (`:1285-1297`) currently pins **4** lines and must widen to **5** (Revision 3, MAJOR #2 fix: `weapon.sword_great` is `two_handed` — `rules/core/equipment.json:31` — so `combat_totals`'s `if weapon.two_handed { push(bare()); continue; }` branch, `derived/combat.rs:244-247`, emits exactly **one** new line, `("weapon.sword_great", false)`, never a shield-paired second one; the "two" great-sword rows the book prints (mounted + on-foot) only exist once F2's mounted twin lands on top of this single on-foot line); (4) `migration.rs` schema-19→20 fold tests (§ 6) | none (self-contained; the plan lists it directly after F0) | **Yes, bump 19→20.** Boundary e2e after F1 lands (plan § 1: "inside a slice only where it changes the save format" applies here) | `equipment-loadout-stowed` / `-carried` / `-wielded` replacing `equipment-equipped-label`; both locales |
| **F2** | (1) `virtues_flaws.json` data-integrity red: `uncomputed_clauses.rs::every_uncomputed_rule_entry_states_its_rule_in_every_locale` fails for `virtue.berserk` the moment it is reclassified `uncomputed_rule` with no `description` yet authored (its `summary` states no number); goes green once the full passage lands in both locales; (2) **restated, Revision 3 MAJOR #3 fix** — `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` (`uncomputed_clauses.rs:1822`) matches **by id only** and never re-checks its own prose claim, so deleting `casting_total_mod` from `virtue.cyclic_magic_positive`/`flaw.cyclic_magic_negative` while leaving the exemption row in place produces **no red at all** — the row's stale "both... via two effects" claim is invisible to the guard suite. The real red-checkpoint step is: **remove** the exemption row (not merely leave it stale) with no `description` yet authored → `no_swept_entry_drops_an_uncomputed_mechanical_clause` (:2678) now falls through to the missing-mechanical-text check and fails → add the `description` (both locales) → green. Delete the effect, remove the row, and add `description` in the **same** change, since nothing forces a second pass otherwise; (3) `the_berserker_matches_the_book` — Soak/Pole-Axe/Kick assertions (:1510+) currently pin the engine's over-applied figures (Soak 11, +15/+5, +8/+2) and must flip to the book's (Soak 9, +13/+7, +6/+4) — now purely a *data* consequence, no `derived.rs` change; (4) `the_bjornaer_matches_the_book` — Casting Totals (:244+) currently pin +22/+15/+13 and must flip to +19/+12/+10; (5) `the_mercere_matches_the_book` — Casting Totals (:768+) currently pin +29/+38 and must flip to +26/+35; (6) `crates/arm-rules/src/equipment.rs`-adjacent integrity test: `weapon.dodge`/`weapon.fist`/`weapon.kick` carry `body_attack == true`, no other entry does; (7) `combat_totals` mounted-twin test: `entity.mounted = true` + Ride 5 adds mounted twins only for lines whose weapon has `body_attack == false` (D66) — the Fist line stays singular, unmounted; the long-sword-with-shield and (post-F1) great-sword lines each gain a mounted twin at the book's own figures, so `the_knight_matches_the_book` asserts all five-becomes-eight lines by name via the existing `line()`/`stats()` helpers | **F1** (K3's mounted-twin pass composes with F1's `Carried` great-sword row: every non-body-attack line in `out`, regardless of `loadout`, gets a mounted twin — the great sword's single F1 line becomes two once F2 lands). The conditional-modifier data change has **no** dependency on F1 | **No bump** (schema stays 20, F1's — `Weapon::body_attack` is ruleset JSON, not a save field, same reasoning as § 6). Boundary e2e after Group F closes (plan § 1: "e2e at phase boundaries... after each X-block" — F is its own boundary) | K3 only: `mounted-label` (entity toggle), `derived-combat-mounted-suffix` (export + UI row suffix); both locales. **No Fluent keys for the conditional-modifier fix** — Revision 2 drops all five from Revision 1 |

---

## 9. Open questions for Norbert

**Both resolved 2026-09-29 (Norbert) as recommended.** (1) Ways of the (Land)'s
combat clause needs nothing more: the full rule already lives in `description`.
(2) `Entity.mounted: bool` is enough, and a richer mount record is not planned.

| # | Question | Recommendation |
|---|---|---|
| 1 | Ways of the (Land) also says *"+3 bonus to all rolls, including combat... that directly involve that area and its inhabitants"* (`ArMDE:5233`) — the data carries no `combat_mod` at all today, only `casting_total_mod`. Is the missing combat clause in scope for F2, or does it belong in the same bucket as row 45 (a missing-authorization-shaped gap, fixed alongside a broader data survey)? | Leave out of F2 (F2 fixes an *unconditional* application; this is a *missing* effect — a different defect shape) and file it as a new open-todos row for a Phase-3 data slice. Since the Virtue is going to `uncomputed_rule` anyway (§ 2c), the missing clause needs no effect at all — it is already covered by the same `description` that carries the Casting clause, so there is nothing to add beyond filing the observation. |
| 2 | Is a bare `Entity.mounted: bool` ("the mount record") sufficient, or does Norbert want a richer mount concept (a named/typed mount, tied into Magic Possessions' equipment modelling from M5.5) for a future slice? | Ship the bool now (YAGNI — the passage attaches no mechanic to *which* mount). Note the richer model as a non-blocking future idea, not a prerequisite for F2. |

**Resolved since Revision 2:** which lines get a mounted twin — **D66**
(`decisions.md:831-842`, Norbert, 2026-09-29) rules an explicit
`Weapon::body_attack` catalogue field, set on Fist/Kick/Dodge, reproducing
the Knight's template rather than inferring an exception from `min_strength`.
See § 2b. No longer open.

---

## References loaded

- `docs/vf-audit/phase-2-plan.md` (Group F, § 1 rules, § 4 critical path)
- `docs/book-template-conformance.md` (K2, K3, K4, K5, B1, B2, MAG1, MAG7,
  headline sections for grogs/companions/magi)
- `docs/open-todos.md` rows 42, 48, 49, 52
- `docs/vf-audit/decisions.md` D4, D15, D52, D58, D61, D66
- `docs/vf-audit/design-b0-ranging-and-predicates.md`,
  `docs/vf-audit/design-d0-xp-modes.md` (shape and lessons precedent)
- `tmp/f0-plan-review.md` (Revision 2's mandate — full findings list)
- `tmp/f0-architect-review.md` (Revision 3's mandate — full findings list)
- `crates/arm-rules/src/types.rs` (`Effect`, `ParamGate`, `AbilityRollMod`/
  `AbilityRollModParam`, `EquipmentSlot` incl. derive list at :4203,
  `Classification` incl. doc comment at :224-227, `CombatStat`,
  `CastingScope`, `deny_unknown_fields` grep)
- `crates/arm-rules/src/derived.rs` (`in_play_mods`, `SurfacedModifier`,
  `ModifierFamily`, `DerivedTotals`)
- `crates/arm-rules/src/derived/combat.rs` (`encumbrance`, `combat_totals`
  incl. the `two_handed` branch at :244-247, `soak`)
- `crates/arm-rules/src/derived/casting.rs` (`CastingTotal`)
- `crates/arm-rules/src/migration.rs` (`SCHEMA_VERSION`, fold idiom)
- `crates/arm-rules/src/equipment.rs` (`Weapon` struct, `min_strength` doc
  comment — superseded as the mounted-twin predicate in Revision 3, kept only
  as the source of the term "body attacks")
- `crates/arm-rules/tests/book_templates.rs` (Knight/Berserker/Bjornaer/Mercere
  fixtures and assertions)
- `crates/arm-rules/tests/uncomputed_clauses.rs`
  (`every_uncomputed_rule_entry_states_its_rule_in_every_locale`,
  `no_swept_entry_drops_an_uncomputed_mechanical_clause`,
  `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` rows for both Cyclic Magic ids)
- `rules/core/virtues_flaws.json` (`virtue.berserk`, `virtue.ways_of_the_land`,
  `virtue.cyclic_magic_positive`, `flaw.cyclic_magic_negative`,
  `virtue.special_circumstances`)
- `rules/core/abilities.json` (`ability.ride`)
- `rules/core/equipment.json` (all 32 `weapon.*` entries; `weapon.sword_great`'s
  `two_handed: true` at :31; the three body-attack entries `weapon.dodge`/
  `weapon.fist`/`weapon.kick` at :11/:13/:18)
- `rules/i18n/en/virtues_flaws.json`, `rules/i18n/de/virtues_flaws.json`
  (existing `summary`/`description` text for all five carrier ids and
  `flaw.corrupted_abilities`/`_arts`/`_spells` as the D15 precedent's own text)
- `rules/source/en/Ars Magica - Definitive Edition (Core Rules).md`
  (verified verbatim: `ArMDE:16837-16839`, `:3500-3503`, `:5231-5234`,
  `:3635-3638`, `:5893-5896`, `:4998-5001`, `:1460-1486` Knight statblock
  re-checked for Revision 2)
- `ui/src/lib/types.ts`, `ui/src/lib/effect-parity.test.ts`,
  `ui/src/lib/components/DerivedSurfacedModifiersSection.svelte`,
  `ui/src/lib/components/VirtueFlawTab.svelte`,
  `ui/src/lib/components/EquipmentTab.svelte`, `ui/src/lib/state.svelte.ts`,
  `ui/src/lib/i18n.test.ts`
- `crates/arm-rules/src/export.rs`, `crates/arm-rules/src/export/sections.rs`

## Verdict

COMPLETE — Revision 3 addresses all 4 architect-review findings (1 BLOCKER,
2 MAJOR, 1 MINOR), resolved per D66 where a Norbert ruling was needed; ready
for re-review before F1 starts.

---

## Revision 2 — dated summary (2026-09-28)

| # | Severity | Finding | Resolution |
|---|---|---|---|
| 1 | BLOCKER | `circumstantial` mechanism (§ 2c) was the "sheet list of situational modifiers" D61 declined | Dropped entirely. Replaced with D15's shipped precedent: delete the conditional effect, reclassify (`uncomputed_rule` for Berserk/Ways-of-the-Land, unchanged `in_play_effect` for the three entries keeping an untouched sibling effect), full passage into `description` where the guards require it. No new `Effect` field, `ModifierFamily`, or Fluent key |
| 2 | BLOCKER | Uniform mounted-twin doubling would emit a "Fist (mounted)" row the Knight's template does not print | Mounted twin now scoped to `Weapon::min_strength.is_some()` (excludes the three body attacks: Dodge/Fist/Kick), reproducing the Knight's five rows exactly. Recommended, not asserted — carried as Open Question 2 pending Norbert's confirmation, since the source rule itself states no exception |
| 3 | MAJOR | `LoadoutState`'s sample derive list omitted `Default` (required by `#[default]`) and `PartialOrd`/`Ord` (required by `EquipmentSlot`'s existing derive stack) | Both added, matching `EquipmentSlot`'s own stack exactly |
| 4 | MAJOR | F1's stated red ("fails to deserialize") is wrong: `EquipmentSlot`/`Entity` carry no `deny_unknown_fields`, so a legacy fixture deserializes silently | Restated as two distinct reds: a compile-time one (call sites referencing the renamed field) and an assertion-level one (`loadout` defaults to `Stowed` instead of translating `equipped: true` to `Wielded`, until the fold exists). Added a fourth fixture (both keys present) documenting the fold's deliberate no-op behavior |
| 5 | MINOR | Id count understated ("four ids") | Corrected: four row-49 witness Virtues/Flaws, **five** carrier catalogue ids (Cyclic Magic's Virtue and Flaw are two ids for one witness) |

---

## Revision 3 — dated summary (2026-09-29)

| # | Severity | Finding | Resolution |
|---|---|---|---|
| 1 | BLOCKER | Revision 2's mounted-twin exclusion, `Weapon::min_strength.is_none()`, infers a mounting exception from a field whose documented purpose is unrelated (can this weapon be wielded at all) — the "rules meaning from an unrelated field" mistake this project already rejected once (D52) | Dropped. **D66** (Norbert, 2026-09-29) rules an explicit, purpose-named weapon-catalogue field instead. This revision names it `Weapon::body_attack: bool` (reusing `min_strength`'s own "body attacks" terminology, so no new vocabulary), default `false`, set on `weapon.dodge`/`weapon.fist`/`weapon.kick` only. Ruleset JSON, no schema bump. New integrity test, TS mirror, and `RULES.md` note (D66's explicit requirement) specified in § 2b |
| 2 | MAJOR | F1's red-test claim said the great sword adds "two" lines; `weapon.sword_great` is `two_handed`, so `combat_totals` emits exactly **one** bare line, never a shield-paired second one | Restated: `emitted` widens from 4 to **5**. The "two" (mounted + on-foot) only exists after F2's mounted twin lands on top of that single line — already correctly stated in F2's own red test, now cross-referenced |
| 3 | MAJOR | F2's red-test claim said deleting `casting_total_mod` while leaving the stale `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` exemption row in place would trigger a guard failure — but that guard matches by id only and never re-checks its own prose claim, so no red occurs at that point | Restated: the guard stays silently green until the exemption row is **removed** (not merely left stale); removing it (with no `description` yet authored) is what produces the actual red, falling through to the missing-mechanical-text check. Sequenced as one change: delete effect + remove row + add `description`, together |
| 4 | MINOR | The F1 migration sketch's "stamps SCHEMA_VERSION, same as every other fold above" did not hold up — the existing folds key on a single top-level boolean, and this fold is per equipment-array-element | Sketch now captures `equipment_loadout_absent` (any element still lacking `loadout`) before the loop and stamps `entity.schema_version` on that flag afterward, the same idiom one level down, explicitly distinguished from the version-less `wrap_legacy_ability_parameters` precedent it must not be confused with |

Two open questions remain for Norbert (§ 9): the Ways-of-the-Land missing
combat-roll clause (Q1, unaffected by this revision), and whether
`Entity.mounted: bool` is sufficient or a richer mount concept is wanted
later (Q2). The mounted-twin exclusion predicate (formerly Q2/Revision 2) is
resolved by D66 and is no longer open.
