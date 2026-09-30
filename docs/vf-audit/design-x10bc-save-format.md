# X10bc — design: banked Art/Ability XP and the per-spell within-focus marker (D65 N5(b)/(c), row 51(b)/(c))

Design for X10b (banked XP beside a bought Art/Ability score) and X10c (a
per-spell "within the focus" marker). No code changed here. Findings:
`book-template-conformance.md` §§ MAG12/MAG8; ruling `decisions.md` D65 N5 —
"as data... if the source does not give a value, report it, never invent it."

## 0. What already exists

`AbilityScore` (`types.rs:4616-4630`) / `ArtScore` (`:4639-4644`): score only.
`ArtScore`'s doc (`:4636`) cites a nonexistent `Entity::art_xp_pool` — dead
link (one shared `Entity::xp_pool`, `:5432`); fix alongside X10b.
`charged_cost` (`effective/xp.rs:45-52`) is already the **fixed** formula —
S2/F-547's off-by-one is gone (`book_templates.rs:1752`). The Ability/Art
spend loop (`:1100-1178`) reads `xp_for_score(a.score)` as `table`, subtracts
a granted floor (Abilities), then `charged_cost(payable, affinity)` — the
site both slices touch. `CastingTotal`/`CastingWithinFocus`
(`derived/casting.rs:68-109,17-29`) already compute **both** figures for
every `(Technique,Form)` cell (`:288-309`); MAG8 is that nothing picks one
*per spell* today. `D4_WITHIN_FOCUS_ONLY` (`derived.rs:231`): Potent Magic
already folds into `within_focus` alone, never `total` (`:257-268,296-300`)
— reuse, don't re-derive. `Effect::MagicalFocus` (`types.rs:2193-2198`):
free-text `param`, `major` bool, one per magus. `SpellSelection`
(`:4653-4686`): no marker today. Fixtures standing in for the missing bank:
`magus_guernicus.json` (`xp_pool: 432`), `grog_specialist.json`
(`xp_pool: 330`); tests `book_templates.rs:657-699` / `:1747-1761`.

`SCHEMA_VERSION = 20` (`migration.rs:174`). `load_entity_migrating`
(`:778-906`) dispatches on legacy-key **presence**, never the recorded
version — one direct jump, no per-version chain (`:759-761`). Numeric-field
precedent, by kind: a **reshaped legacy field** (talisman `bonus`,
`aging_reductions`) **rejects** the load on a bad shape
(`:1493-1583,1592-1668`); a **live field merely out of range** (`aura`) is
**clamped silently, no stamp** (`:832`, `:1143-1246`). Neither new field here
is either kind — both are plain additive optionals, the shape of
`mastery_abilities` ("stays 13", `:4681`) and `from_normal_budget` ("no
bump", D0 §2).

## 1. X10b — banked XP on `AbilityScore`/`ArtScore`

New field on both, **appended last**: `banked_xp: u32`,
`#[serde(default, skip_serializing_if = "is_zero")]`. Doc: XP already
acquired toward the NEXT score, in the raw table-XP currency `xp_for_score`
reads — not charged XP, not `Entity::xp_pool`. Printed "X (Z)"
(ArMDE:1177/:1179): X = score, Z = this field. **Last, not after `score`**:
both derive `Ord` positionally and every pre-existing save has `banked_xp ==
0` throughout, so a trailing tie-break key leaves `Entity::normalize`'s sort
byte-identical for old saves — earlier, it would re-order a character holding
two same-scored specialties of one Ability (legal today).

**Engine** (`effective/xp.rs`): Ability loop (`:1100-1133`) —
`payable = table.saturating_sub(floor_table).saturating_add(a.banked_xp)`,
unchanged `charged_cost(payable, affinity)`. Art loop (`:1169-1178`) —
`table = xp_for_score(a.score)?.saturating_add(a.banked_xp)`, unchanged
`charged_cost`. Banked joins the raw table total **before** `charged_cost`:
Guernicus's In 12 (raw 78) + 5 banked = 83, charged once at 2/3 to 55, not
`52 + 5` (§ MAG12). `saturating_add` closes the hostile-input path
(`banked_xp: u32::MAX`): no panic.

**Validation (new).** A `banked_xp` at/over the delta to the next score is a
self-contradiction (that IS the next score, mis-recorded), not a crash —
`saturating_add` already stops the crash. New warning
`banked_xp_at_or_above_next_level` (Abilities/Arts phase; `ability`/`art`,
`banked`, `needed`), firing when `banked_xp >= xp_for_score(score+1) -
xp_for_score(score)`, or at any `banked_xp > 0` at the ceiling score.
Severity warning, matching `CODE_GENERAL_XP_UNSPENT` (open question 7.2).

**Saves / SCHEMA_VERSION: no bump.** Ordinary `serde(default,
skip_serializing_if)` on an existing Vec-item struct, identical in shape to
`mastery_abilities`/`from_normal_budget`, both shipped bump-free. An old save
omits the key, defaults to `0`, round-trips byte-identically — no legacy
*shape* to reconcile. **Overrides row 51's "(bump)" tag** — question 7.3.

## 2. X10c — per-spell within-focus marker on `SpellSelection`

New field, **appended last**: `within_focus: bool`,
`#[serde(default, skip_serializing_if = "crate::types::is_false")]`. Doc: the
player's own claim this spell is within the character's Magical Focus
(ArMDE:4399-4422) — free-text `focus` can't supply this (MAG8), so it's a
recorded choice. Harmless if the character has no Focus.

**Engine.** New function beside `casting_totals` in `derived/casting.rs` (not
`effective/spell.rs` — `effective` never depends on `derived`, per D0's own
dependency check):
```rust
pub fn spell_casting_total(chosen: &SpellSelection, entity: &Entity, ruleset: &Ruleset) -> Option<i32> {
    let spell = ruleset.spell(&chosen.spell)?;
    let cell = casting_totals(entity, ruleset).into_iter()
        .find(|c| c.technique == spell.technique && c.form == spell.form)?;
    Some(if chosen.within_focus { cell.within_focus.map(|wf| wf.formulaic).unwrap_or(cell.formulaic) } else { cell.formulaic })
}
```
Formulaic only — the figure the Format table prints per spell (ArMDE:1179's "TeFo X/+Y"); Ritual/Spontaneous stay grid-only.

**Interaction with `D4_WITHIN_FOCUS_ONLY`: none needed.** Potent Magic already
folds into `within_focus.formulaic` and nowhere else; this function is a pure
*selector* over the two numbers `casting_totals` already computes, so
`within_focus: true` picks up Potent Magic automatically — why D4 built the
split as two fields on one struct rather than a second total. `within_focus:
true` with no Focus falls back to `cell.formulaic` silently — reachable only
via a hand-edited save, since the UI (§ 3) offers the toggle only with a Focus.

**Validation: none.** Whether a Technique/Form lies "within" a free-text descriptor is table judgement (MAG8: "a capability gap, and arguably the right design"); the engine records the claim, it does not adjudicate it.

**Saves / SCHEMA_VERSION**: no bump, identical reasoning to § 1.

## 3. UI

`ArtGrid.svelte:60-114` — number input beside the score spinner (`:92`),
bound to `banked_xp`. `AbilityTab.svelte:404-479` — same, in the row snippet
(spinner `:452-454`). `SpellTab.svelte` — a toggle per spell, bound to
`within_focus`, shown only when the character holds a Focus: reuse
`CastingTotal.within_focus` (`ui/src/lib/types.ts:1084`, already `| null` per
cell) as the "has a Focus" signal, rather than add a new DTO field — the same
test `derived/casting.rs`'s `mods.has_focus` already gates on.
`DerivedLabCastingSection.svelte:148-154` (full grid) untouched. Fluent, both
`locales/{en,de}/main.ftl`: `art-banked-xp-label`, `ability-banked-xp-label`,
`spell-within-focus-label`, `issue-banked_xp_at_or_above_next_level`.

## 4. Book-template tests that become exact

**Guernicus** (`:657-699`): `art.intellego` gains `"banked_xp": 5`; `xp_pool`
432 → 435, error/warning sets stay empty — MAG12 goes from "exact only
because the pool is under-funded to match" to actually exact. **Specialist**
(`:1747-1761`): `ability.bows` gains `"banked_xp": 2`; `warning_codes` drops
`general_xp_unspent` to empty — S2 is already resolved, so this fixture
becomes fully exact. **The MAG8 quartet** (`:492,599,813,945` — Ex
Miscellanea, Flambeau, Mercere, Tremere): each spell list gains
`"within_focus": true` on exactly the spells the book prints focused
(Flambeau: all five; Ex Miscellanea: two stone spells; Mercere: one weather
spell; Tremere: none — nothing on his list is certamen). New assertions call
`spell_casting_total` per spell and match the book's one printed figure.
§ MAG8's capability-gap paragraph stays (the engine still can't *derive*
membership) — only the "wrong" verdict gains a resolved footnote in both
§ MAG12 and § MAG8.

## 5. Export and round-trip

`write_abilities`/`write_arts` (`export/sections.rs:245-324,410-455`): append
`" ({banked})"` to the score cell only when `banked_xp > 0` — the book's own
"X (Z)"; omitted at 0 keeps existing exports byte-stable. `write_spells`
(`:459-500`) **has no Casting Total column at all today**, for any spell,
confirmed by direct read — a bare `within_focus` bool prints nothing new by
itself; recommend **not** bundling a new column into X10c (question 7.1).
Round-trip: both fields are ordinary struct fields covered by
`Entity::normalize`'s blanket sort and the existing byte-identity tests
(`types.rs:8531,8876`-style); one new round-trip test per field.

## 6. Test plan and sub-slice order

**X10b first** (no X10c dependency): (1) RED, compile-forced — fixture
literal `"banked_xp": 5` fails to deserialize pre-field; (2) RED — Guernicus
at `banked_xp: 5`/`xp_pool: 435` still reports `not_enough_xp`
pre-engine-change; (3) GREEN — `saturating_add` before `charged_cost` in both
loops, Guernicus and Specialist land exact; (4) RED→GREEN — the new warning,
table-driven {just under: none; at/over: warning; `u32::MAX`: warning, no
panic}; (5) round-trip test, UI inputs, both Fluent keys, the Markdown
`" (Z)"` column.

**X10c after X10b**: (1) RED, compile-forced — `within_focus` doesn't exist;
(2) RED — a Flambeau-shaped entity with `within_focus: true` on a flame spell
reads 29 (grid base), not the book's 41, pre-`spell_casting_total`;
(3) GREEN — the function, the four MAG8 fixtures, rewritten assertions;
(4) round-trip test, `SpellTab` toggle, both Fluent keys.

**e2e/gate.** Save-format change: full `npm run test:e2e` **and**
`test:e2e:portable` once after X10c, not per slice. Full gate (`cargo test
--workspace`, clippy, fmt, `ui` checks, `cargo tauri build --no-bundle`) after
each slice.

## 7. Open questions for Norbert

**7.1 — a Casting Total column in Markdown export.** The book prints one, the app prints none for any spell today. Recommend leaving it out of X10c as its own export slice, not folded into a "record a choice" slice.

**7.2 — severity of the new banked-XP validation.** Recommend **warning**
(mirrors `CODE_GENERAL_XP_UNSPENT`); an **error** case exists too — the
figure names a score the character should already have, closer to
`CODE_DUPLICATE_ABILITY`.

**7.3 — no SCHEMA_VERSION bump, against row 51's assumption.** Recommend
confirming this (both fields additive, §§ 1-2, the D0/`mastery_abilities`
precedent) over a bump that documents no actual shape change.

## References loaded

`decisions.md` D65; `open-todos.md` row 51; `book-template-conformance.md`
§§ MAG8, MAG10-12; `design-d0-xp-modes.md`; `types.rs` (`AbilityScore`,
`ArtScore`, `SpellSelection`, `MagicalFocus`, `Entity::xp_pool`);
`effective/xp.rs` (`charged_cost`, spend loop); `derived/casting.rs`
(`CastingTotal`, `CastingWithinFocus`, `variant`); `derived.rs:229-300`;
`migration.rs` (`SCHEMA_VERSION`, `load_entity_migrating`, reject/clamp
tests); `export/sections.rs`; `tests/book_templates.rs`
(`:657-699,1747-1761,492,599,813,945`) + fixtures; `ui/src/lib/components/
{ArtGrid,AbilityTab,SpellTab,DerivedLabCastingSection}.svelte`;
`ui/src/lib/types.ts:1038-1099`; `locales/{en,de}/main.ftl`; rulebook source
verified against `Ars Magica - Definitive Edition (Core Rules).md`:
ArMDE:1177-1179, :1326, :1881, :4399-4422.

## Verdict
COMPLETE
