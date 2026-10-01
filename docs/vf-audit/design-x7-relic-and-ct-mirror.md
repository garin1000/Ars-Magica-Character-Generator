# X7 — design: the relic stub (F-256) and the D4 Casting Total mirror

Two independent, unfinished items from X7b-d. No code changed here.

## 1. F-256 — the relic-as-item mechanic (ArMDE:17619-17627)

**What ships today.** `virtue.relic`/`virtue.powerful_relic`
(`rules/core/virtues_flaws.json:6369-6378,6530-6538`) each carry one effect,
`Effect::RelicTrueFaith { score }` (`types.rs:2025-2038`), cited ArMDE:4852-4855
/ :4782-4787. It is deliberately excluded from `effective::true_faith`
(`effective/might.rs:46`) and from `magic_resistance`'s `true_faith_floor`
(`derived/casting.rs:495`), which read only `Effect::TrueFaithGrant` — the
original finding (a relic's score leaking onto the character's own True Faith
/ MR) is fixed and covered by `data_integrity.rs:3381-3391`. In `derived.rs`'s
exhaustive `in_play_mods` match, `RelicTrueFaith` sits in the "no in-play
modifier" arm (`:601-603`) — not merely unconsumed, **unsurfaced**: nothing
renders it anywhere except the Virtue's own stored description text. Both
entries carry `"classification": "creation_effect"`.

**What the book says a relic IS** (ArMDE:17619-17627, "Relics"). A relic has
its own True Faith score, which (1) **FAITH**, :17623 — gives it Faith Points
"usable by its bearer as Confidence", and grants the bearer Magic Resistance =
10× that score; "a person can only benefit from one relic at a time"; (2)
**DIVINE MIGHT**, :17624 — a separate Might-like pool gating how often the
relic's own powers fire, refreshed at sunrise; (3) **SCOURGING THE INFERNAL**,
:17625 — an area effect against Infernal creatures scaled to the relic's Might.
:17653 confirms relic MR stacks additively with a church officer's but, unlike
their own MR, "does not affect Soak". None of this is the character's own True
Faith Score (:17607, already respected).

**Why NOT new machinery.** The repo already has the identical shape of
problem on `Talisman`: `TalismanAttunement`'s bonus is "stored, deliberately
not computed" (`types.rs:5332-5343`) because it applies only "while touching
the talisman" — a moment of play the model does not represent — and is
surfaced as text the player applies at the table, not folded into any total.
A relic's bearer-MR is the same shape, with an extra wrinkle the book states
explicitly: *only one relic benefits the bearer at a time*, a per-scene player
choice with no "currently carried" state on `Entity` to decide it from, and a
piety/behavior condition on `virtue.powerful_relic`'s own text
(:4784-4786, "if you ever behave impiously ... your relic will cease to
function"). Computing a blanket "+10×score MR" onto the sheet would be
**wrong rules output** the moment a character owns a second relic, multiple
party members share one, or the storyguide rules impiety — exactly the
product-integrity failure this project treats as high severity. Minor Relic
and Powerful Relic a magus might also stack with Parma are a second,
unmodelled interaction (RoP:D governs relic/Might stacking, out of scope: no
RoP:D source file in `rules/source/en/`).

**Minimal model: composed description, not a computed field.** This repo
already has the precedent for "a Virtue's own clause is one sentence, the
mechanic lives in the chapter it points to": `virtue.true_faith` composes its
`description` from its own entry plus the "True Faith" section
(`ArMDE:5169-5172` + `:17603-17617`) via `COMPOSED_DESCRIPTIONS`
(`tests/x2_reclassification.rs:450-453`), per D46/X2d. `virtue.relic`'s own
sentence ends "See Chapter 12: Realms for rules for relics and True Faith" —
it is *already* pointing at exactly the passage that states the uncomputed
rule. Proposed: extend each entry's `description` (both locales) with the
verbatim "Relics" heading through the FAITH clause, ArMDE:17619-17623 (not
:17624-17627 — Divine Might/Scourging are the relic's *own* defenses, not a
rule about the bearer's sheet, and stay out of scope for this finding); add
`(17619, 17623)` to each entry's `COMPOSED_DESCRIPTIONS` row; update `source`
to the two-range form `virtue.true_faith` already uses.

**Classification must follow.** D67: "an entry with any stated rule computed
nowhere is `uncomputed_rule`, regardless of what else it computes" — the exact
rule X7b-d already applied to `virtue.mythic_blood` (F-205) in this same batch.
Once the description states the bearer-MR/Confidence clause, `creation_effect`
is wrong for both entries; reclassify to `uncomputed_rule`. `RULES.md` gets a
new row for both ids citing ArMDE:4852-4855/:4782-4787 + :17619-17623.

**No save-format question here.** Nothing on `Entity` changes — this is rules
catalogue data (`rules/core/`, `rules/i18n/`), not a save field. No
`SCHEMA_VERSION` bump, no migration.

**Test plan.** (1) `x2_reclassification.rs::x2_shipped_descriptions_match_their_cited_passage_verbatim`
picks up both ids once added to `COMPOSED_DESCRIPTIONS` — RED until the
description text is the exact concatenation, GREEN once it is. (2)
`uncomputed_clauses.rs`'s classification guard — RED on the stale
`creation_effect` tag once the new clause is detected as stated-but-uncomputed
(if the guard scans by classification rather than content, add the two ids to
its pending/expected list instead). (3) `rulebook_citations.rs` /
`source_citations.rs` need no change (acronym + range already covered by the
guard's existing roots). (4) No UI change and no new Fluent keys — the
existing Virtue-detail view already renders `description` verbatim for every
entry, so the longer text appears automatically.

## 2. The D4 Casting Total mirror

**Status: owed, and currently wrong, not merely undone.** D4 fixed
`derived/lab.rs::lab_totals`'s Potent Magic defect (a focus-only Lab Total
bonus leaking into the unconditional total) by giving `Effect::LabTotalMod` a
`scope: LabTotalModScope` (`InPlayGrid` / `WithinFocusOnly` / `NeverAtCreation`,
X7a) and folding `WithinFocusOnly` separately
(`derived.rs::in_play_lab_total_mod_within_focus:262-284`) into
`InPlayMods::lab_mod_within_focus`, never into `lab_mod`. The X7b-d handover
(item 14) explicitly flagged that `casting_total_mod{scope:"all"}` on the same
two Potent Magic entries "shares the same defect and is not separately
pinned" — and it is still unfixed. Confirmed directly in code: both
`virtue.potent_magic_major`/`_minor` (`virtues_flaws.json:6344,6364`) carry
`{"type":"casting_total_mod","amount":6|3,"scope":"all"}` with no within-focus
gate at all — `CastingScope` (`types.rs:2927-2942`) only distinguishes
Formulaic/Ritual/Spontaneous/FormulaicRitual/All, an orthogonal axis to D4's
in-focus/out-of-focus one. `derived.rs::in_play_mods`'s
`Effect::CastingTotalMod` arm (`:332-334`) folds every carrier into
`InPlayMods::casting_mods` unconditionally, which `casting_totals`
(`derived/casting.rs:194-196,224-241`) reads via `casting_mod_for` into
`common`/`base` — i.e. into `cell.formulaic`/`ritual`/`spontaneous_*` — **and**
into `within_focus` (`variant(true, 0)` at `:288-289` reuses the same
`formulaic_mod`/`ritual_mod`/`spontaneous_mod`). So today a magus with Potent
Magic gets the +6/+3 on **every** spell, inside or outside her field — exactly
the Lab Total bug, un-mirrored.

**This also corrects the X10bc design note.** `docs/vf-audit/design-x10bc-save-format.md`
§2 ("Interaction with `D4_WITHIN_FOCUS_ONLY`: none needed... Potent Magic
already folds into `within_focus.formulaic` and nowhere else") is wrong for
Casting (it is true for Lab Total, the half D4 actually fixed). Harmless so
far only because no shipped book-template fixture
(`tests/book_templates.rs`) combines Potent Magic with `within_focus: true` —
grep confirms zero `potent_magic` hits there — so X10c's four MAG8 fixtures
are unaffected by this fix.

**Rulebook basis for the split.** ArMDE:4746-4748: Potent Magic "grants a +3
[/+6] bonus to Lab Totals **and Casting Score**" in "her field of magic ...
much as in a Magical Focus" (:4740-4742) — the same "within this narrow field
only" framing D4 already read for the Lab Total half of this exact entry.

**Minimal change.** Add `within_focus_only: bool` (`#[serde(default)]`) to
`Effect::CastingTotalMod` — orthogonal to the existing `scope: CastingScope`
cast-type axis, so no new enum and no touch to any *other* carrier (Method
Caster, Cyclic Magic, Special Circumstances all default `false`, unchanged
behavior). Potent Magic's two rows gain `"within_focus_only": true` beside
their existing `"scope":"all"`. Engine: `InPlayMods` gains
`casting_mods_within_focus: Vec<(i32, CastingScope)>` and a
`casting_mod_within_focus_for(CastType)` helper mirroring
`casting_mod_for` (`derived.rs:653-659`); the `Effect::CastingTotalMod` arm
routes to one Vec or the other on the new flag. In
`derived/casting.rs::casting_totals`'s `variant` closure (`:218-265`), the
three per-type sums gain a `focus_mod = if focused { casting_mod_within_focus_for(cast) } else { 0 }`
term beside the existing `focus_add` — added only when `focused`, so
`base`/`cell.formulaic` etc. (computed via `variant(false, ...)`) are
untouched and only `within_focus` (`variant(true, 0)`, `:288-289`) gains it.

**No save-format question here either.** `CastingTotalMod` lives in rules
catalogue data (`rules/core/virtues_flaws.json`), not on `Entity` — no
`SCHEMA_VERSION` impact.

**Test plan.** (1) RED, compile-forced — the new field doesn't exist. (2) RED
— a hand-built entity with `virtue.potent_magic_major` and no Magical Focus:
today `cell.formulaic` includes +6 it should not; assert it does not once the
flag lands (this is the first test ever to combine Potent Magic with a
casting total, since none exists). (3) RED — the same entity with a Magical
Focus covering the cell: `cell.within_focus.formulaic` must include both the
doubled Art and the +6. (4) GREEN — the `variant` closure change. (5) Extend
`derived.rs::lab_total_mod_carriers_match_the_d4_table`-style assertion (or a
sibling `casting_total_mod_carriers_match_the_d4_table`) so the table is
pinned the same way D4's Lab Total table already is, closing the
"not separately pinned" gap the handover flagged. (6) RULES.md: update the
existing Potent Magic row to note the mirrored split.

## Questions for Norbert

1. **F-256 composed-description range.** Recommend ArMDE:17619-17623 (Relics
   heading + intro + FAITH clause only), leaving Divine Might/Scourging
   (:17624-17627) out as the relic's own defenses, not a bearer-facing rule.
   Alternative: the full :17619-17627 block, if you'd rather the player see
   the complete Relics rules from the Virtue itself. Recommend the shorter
   range — it's the minimum that makes the classification claim true.
2. **F-256 reclassification.** Recommend `creation_effect` → `uncomputed_rule`
   for both entries, per D67, once the description states the bearer-MR
   clause. Alternative: leave classification as-is and accept the
   inconsistency — not recommended, it's the same drift D67 was written to
   close.
3. **D4 Casting Total mirror — do it now or batch it?** It is a two-line data
   change plus a small, mechanical engine mirror of work already done once
   for Lab Total, with no save-format risk. Recommend landing it as its own
   small slice rather than deferring further — it has been "flagged, not
   done" since X7b-d, and the defect is live (wrong numbers for any character
   who takes Potent Magic), not merely undocumented.
4. **X10bc's incorrect claim.** Recommend a one-line correction to
   `design-x10bc-save-format.md` §2 once the mirror lands, noting the claim
   held only for Lab Total.

## References loaded

`tmp/x7bd-handover.md` (items 14, 15, and the Phase-2 status/caveat on
F-256); `decisions.md` D70 (X6 scoping, "the relic stub may land before
X7b-d's F-256"), D4 (conditional Lab Total modifiers), D69/D44/D46/D67 (cited
via the handover); `corrections.md` row F-256; `design-x10bc-save-format.md`
§2; `types.rs` (`Effect::RelicTrueFaith`, `Effect::CastingTotalMod`,
`CastingScope`, `Talisman`/`TalismanAttunement`); `derived.rs` (`in_play_mods`,
`InPlayMods`, `in_play_lab_total_mod_within_focus`, `casting_mod_for`);
`derived/casting.rs` (`casting_totals`, `magic_resistance`, `CastingWithinFocus`);
`effective/might.rs::true_faith`; `rules/core/virtues_flaws.json` (relic and
Potent Magic entries); `tests/data_integrity.rs:3381-3391`;
`tests/x2_reclassification.rs` (`COMPOSED_DESCRIPTIONS`); `tests/book_templates.rs`
(grep confirms no `potent_magic` fixture); rulebook source verified against
`Ars Magica - Definitive Edition (Core Rules).md`: ArMDE:4740-4855,
:17603-17653.

## Verdict
COMPLETE
