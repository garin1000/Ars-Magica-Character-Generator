# Decisions taken during the V/F audit

Norbert's rulings on questions the audit could not settle from the source.
Recorded here so no batch agent re-litigates them and no correction slice
guesses. Each names the question, the ruling, and what it obliges.

---

## D1 — `lab_total_mod` and the spell-level cap

**Question.** `effective/spell.rs::spell_level_cap` computes
Te + Fo + Int + Magic Theory + 3, halves for a Deficient Art, and never adds
`lab_total_mod` — although its own doc comment quotes `ArMDE:2465` ("any
Virtues and Flaws your character has apply to this total if they would apply to
a Lab Total in play") and uses that sentence to justify the halving. RULES.md
records no exclusion. Nine entries carry the effect.

### First ruling — WITHDRAWN, it rested on a false premise

The first version of this decision asked Norbert to split the nine into
"unconditional" and "conditional" and he ruled: add the five unconditional
ones. **The premise was wrong and the question should never have been asked in
that form.** I had not read the passages. B01 caught it on the two entries in
its own range, and reading the other three confirmed it generalises:

| Entry | The condition the book states | Where |
|---|---|---|
| `virtue.inventive_genius` | "+3 … **if you are not using a Laboratory Text or being taught**. If you experiment, you get +6." | ArMDE:4153 |
| `virtue.adept_laboratory_student` | "+6 … **when working from the lab texts of others**, including when reinventing spells" | ArMDE:3370 |
| `virtue.aristotelian_training` | "+1 … **if attempting to synthesize the New Aristotle with Magic Theory**" | ArMDE:3442 |
| `flaw.creative_block` | "–3 … **unless you are using a Laboratory Text or being taught**" | ArMDE:5875 |
| `flaw.weak_scholar` | "–6 … **when working from the Lab Texts of others**" | ArMDE:7082 |
| `virtue.potent_magic_major/_minor` | within the chosen focus only | — |
| `virtue.cyclic_magic_positive` / `flaw.cyclic_magic_negative` | season-dependent | — |

**All nine are conditional.** There is no unconditional subset. Aristotelian
Training is the sharpest case: its condition is not spell invention at all, so
the withdrawn ruling would have introduced a *new* wrong number into a
creation-time total that is correct today.

### Ruling, as re-taken on the corrected facts

**Apply all nine flat to `spell_level_cap`, conditions ignored.** Norbert's
reasoning: this is only a *cap* — a ceiling on which spells may be chosen, not
a number printed as a result — so the generous, condition-free reading is
acceptable there.

**What this obliges.** One added term in
`effective/spell.rs::spell_level_cap`, summing the same flat `lab_mod` that
`derived/lab.rs::lab_totals` already computes. No data change and no
conditionality machinery — the whole point of the ruling is that the cap does
not model lab situations. Update the function's doc comment and RULES.md to
record that the conditions are deliberately ignored here, so the next reader
does not "fix" it back.

### Scope of this ruling — READ THIS BEFORE APPLYING IT

**D1 governs `effective/spell.rs::spell_level_cap` and nothing else.** It does
not extend to the in-play Lab Total, the longevity ritual bonus, the familiar
binding level, or the masterpiece cap. Those are D4, and they go the other way.
An earlier draft of this file generalised D1 across all four; that was wrong and
is corrected here.

---

## D6 — a translation-table error is fixed in BOTH projects, immediately

**Ruling (Norbert, standing for this session).** Any error found in
`rules/source/de/translation-tables/` is corrected **here and in the source
project `arm-de-translation`** in the same step. Not recorded as a todo in one
and fixed in the other — fixed in both.

**Why it is not a Phase 2 item like everything else this audit finds.** A wrong
table is a *generator* of future wrong data: CLAUDE.md makes the tables the
canonical EN→DE mapping, so the next agent producing German text reproduces the
error, and the error outlives the audit. Fixing the data alone leaves the
faucet running.

**Scope of a table's authority, which is what these errors keep violating.**
The tables are authoritative for **terminology** — which German word renders
which English term. They are **not** authoritative for **facts about the
rules**: which Virtue grants which Reputation, at which level, in which
magnitude class. Four of the six errors found so far were factual claims a
terminology table had no business making.

**Precedence when sources disagree — CORRECTED 2026-09-21, the first wording
was unscoped and contradicted the paragraph above it.** B13's F-444 caught it:
this file said flatly *"rulebook > thematic table > `tugenden-fehler.md`"*, which
read as a blanket rule and so cancelled the scope-of-authority paragraph
directly above, where the tables are authoritative for terminology. B13 applied
both readings inside one batch (F-433 against F-438) because the record
permitted both. The correct statement, matching
`rules/source/de/translation-tables/README.md:25-31` (*"gewinnt immer bei
**Sachaussagen**"*) and `RULES.md:788` (*"glossary wins on any term mismatch"*),
is **two rules, keyed on what kind of claim is in dispute**:

1. **A factual claim about the rules** — which Virtue grants which Reputation,
   at which level, in which magnitude class, from which book. **The rulebook
   wins, always.** A terminology table making a factual claim is the error shape
   this audit has now found seven times.
2. **A terminology claim** — which German word renders which English term.
   **The glossary wins**, in the order thematic table > `tugenden-fehler.md`
   (the broadest table, so the likeliest to be wrong). The DE rulebook heading is
   strong evidence of terminology but is not decisive, because the heading is one
   translator's rendering at one line and the glossary is the catalogue-wide
   reconciliation of all of them.

**The case that forced the split, and the reason rule 2 has to exist.**
`flaw.fettered_magic` (ArMDE:6114) and `virtue.tethered_magic` (ArMDE:5141) are
two different catalogue entries, and the **German rulebook gives both the same
heading** — `#### Gefesselte Magie`, at DE 6114 and DE 5141. The shipped data
follows the headings and so ships both under one name, leaving them
indistinguishable in the German UI. `tugenden-fehler.md` resolves it
(`:73` Tethered Magic → *Gefesselte Magie*; `:701` Fettered Magic → *Gekettete
Magie*). Under the old unscoped wording the rulebook won and the collision
stood, which is plainly the wrong answer: a name collision is a defect in the
rendering, not a rule the book states. **A rulebook heading that collides with
another entry's is evidence the heading is wrong, not evidence the glossary is.**

**And the finding underneath the findings: the copy drifts.** The tables are
*copied* into `arm-char-gen`, not referenced. The `Covenfolk` error was not an
error in the source project at all — `arm-de-translation` already carried
`Konventsbewohner` with a written rationale, and the copy here was simply
stale. So a table row disagreeing with the rulebook has two possible causes,
and they need different fixes: a real error (fix both) or a stale copy (re-sync
this one). **Check the source project before concluding the table is wrong.**

---

## D7 — which German name wins when the DE rulebook heading and the glossary disagree

**Question.** B16 escalated two entries (Q-129) rather than judging them, because
D6 rule 2 ("the glossary wins on terminology") appeared to make their shipped
names wrong. `flaw.palsied_hands` ships *Zitternde Hände* (DE 6578's heading)
against `tugenden-fehler.md`'s *Zittrige Hände*;
`flaw.primogeniture_lineage` ships *Erstgeburts-Abstammung* (DE 6634) against
`grundbegriffe.md`'s *Primogenitur-Abstammungslinie*;
`flaw.raised_from_the_dead` ships *Vom Tode auferstanden* (DE 6646) against
*Von den Toten Auferweckt*. `arm-de-translation` was checked and carries the
identical table values, so none of the three is a stale copy — the disagreement
is real.

**Ruling: the DE rulebook heading wins, and all three shipped names are
correct.** D6 rule 2 stands, but it was stated too broadly; this narrows it.

**Why, and this is the part that generalises.** Two of the three table rows are
**tagged to a different book** — `Palsied Hands` carries `HdH:WL` (*Häuser des
Hermes: Wahre Linien*) and `Raised from the Dead` carries `SdM:G` (*Sphären der
Macht*). Each is therefore evidence about how **that** book renders the term,
not about how the core book does. A row sourced from one book cannot override a
heading in another; the two renderings are both correct, for different books.
The third, `Primogeniture Lineage`, carries no book tag at all, and the core
book renders it at DE 6634 — so the heading is the better-sourced value there
too.

**The general rule, which supersedes D6 rule 2 wherever they touch:**

1. **The DE rulebook heading is the default winner for an entry's name**, because
   the user has that book open beside the app and the repo's line-parity
   invariant is the navigation contract between them. An app name that does not
   match the heading at the matching line is a name the user cannot look up.
2. **The glossary wins only where the heading is *defective*** — where it
   collides with another entry's heading (B13's F-444: the DE book heads both
   `flaw.fettered_magic` and `virtue.tethered_magic` `#### Gefesselte Magie`, and
   a collision is a defect on its face), or where the rulebook never renders the
   term at all.
3. **A glossary row tagged to a book other than the one the entry cites is not
   in the dispute.** It is that book's terminology and must not override the
   citing book's heading.

**What this obliges.** Nothing in `rules/i18n/de/` changes for these three.
Q-129's two entries move from *escalated* to **checked and clean** on the name
question. The three table rows are annotated in both projects so the next
German-writing agent sees which rendering belongs to which book rather than
reading a bare disagreement — per D6, in `arm-char-gen` and `arm-de-translation`
in the same step.

---

## D5 — the description obligation follows the rule, not the classification

**Question.** Today only `uncomputed_rule` must carry a `description`, enforced
in both locales by `uncomputed_clauses.rs`. An entry that computes *one*
mechanic and cannot compute *another* therefore falls through: no class obliges
the second to be written out. `virtue.faerie_magic` computes its main effect
while three mechanics behind its page-reference reach the user nowhere;
`virtue.faerie_blood` computes none of its seven blood types. In the data, 9 of
93 `in_play_effect` entries carry a description informally and **0 of 125
`creation_effect` entries do**.

**Ruling: any mechanical clause the engine does not compute must be written
into `description`, in both locales, whatever the entry's classification.**

**What this obliges.**

1. **The guard stops being class-keyed.** `uncomputed_clauses.rs` currently
   asks "is this `uncomputed_rule`?" It must instead ask "does this passage
   state a rule the effects do not implement?" — for all four classes.
2. **All 218 computed entries must be re-read for uncomputed leftovers.** This
   is not a separate pass: every batch already reads every passage in full, so
   the obligation is folded into check 10's "both directions" and reported per
   entry from B05 onward. B01-B04's parked findings are unparked and become
   plain findings.
3. **A `narrative` entry still carries no description** — it states nothing
   mechanical, so there is nothing to write. If it *does* have something to
   write, it was misclassified, which is a different finding.

**Why this and not reclassification.** Folding every partially-computed entry
into `uncomputed_rule` would have pulled it under the existing guard with no
new machinery, but it would redefine `classification` as "has at least one
uncomputed rule" rather than "what kind of thing this is" — and the whole audit
rests on `classification` being a claim about the rulebook. The obligation
moves; the taxonomy does not.

---

## D4 — conditional Lab Total modifiers everywhere except the cap

**Ruling: a Lab Total does not change unconditionally.** `derived/lab.rs::lab_totals`
folding all nine `lab_total_mod` rows into one flat `lab_mod` is a defect, and
it propagates into every figure computed from the grid — the displayed
Technique×Form totals, `longevity_bonus` (via `creo_corpus_lab_total`),
`familiar_readout`, and `masterpiece_item_cap`. A magus with Potent Magic
currently receives his focus-only bonus on his longevity ritual, his familiar
binding and his masterpiece cap, none of which lie within that focus.

**What this obliges.** The conditions resolve **statically** — no lab-activity
context, no runtime machinery, no new saved field. Each of the nine is decided
once, from its passage:

| Entry | Condition (ArMDE) | In a character-generation Lab Total? |
|---|---|---|
| `virtue.inventive_genius` | not using a Lab Text, not being taught (:4153) | **yes**, +3 |
| `flaw.creative_block` | unless using a Lab Text or being taught (:5875) | **yes**, −3 |
| `virtue.adept_laboratory_student` | working from *others'* lab texts (:3370) | no |
| `flaw.weak_scholar` | working from *others'* Lab Texts (:7082) | no |
| `virtue.aristotelian_training` | synthesizing the New Aristotle with Magic Theory (:3442) | no — see below |
| `virtue.cyclic_magic_positive` / `flaw.cyclic_magic_negative` | season-dependent | no; creation fixes no season |
| `virtue.potent_magic_major` / `_minor` | within the chosen focus | not in `total`; belongs in `within_focus` |

**The engine already has the shape for the focus case.** `derived/lab.rs`'s
`LabTotal` carries `total` and `within_focus` side by side, and its own comments
already record that a focus does not apply to a Longevity Ritual and applies
only to items within it. Potent Magic belongs in `within_focus`, exactly where
Magical Focus doubling already lives. This is an existing field, not a new
subsystem.

**`virtue.aristotelian_training` is a distinct case and needs its own verdict.**
Its +1 is conditioned on an activity described in *Art and Academe*, which is
not in `rules/source/en/`. The *rule* is properly sourced (ArMDE:3442 states
it), so this is not a provenance violation — but the condition can never be
satisfied by anything this app models, so the bonus can never legitimately fire
in any Lab Total. That points at `uncomputed_rule` with the rule written out,
rather than an `in_play_effect` carrying a `lab_total_mod` that must never
apply. Its other two clauses (+1 Artes Liberales for grammar/logic/rhetoric,
+1 Disputatio Totals, *Art and Academe* p.103) are in the same position.
**Flagged for Phase 2, not yet ruled on.**

---

## D2 — granted Virtues and the bought-only validators

**Question.** `validate_ability_bonus_targets` and
`validate_characteristic_delta_preconditions` read bought scores only, so a
Virtue granted by a House or a character type skips them — a granted Great
Characteristic bypasses the "already at ±3" precondition a bought one must
satisfy. The neighbouring case is documented as deliberate
(`validation/prereq.rs::PrereqCtx::build` says grants "must never reach" the
incompatibility and trait checks, citing review finding B1); these two carry no
note.

**Ruling: needs a rules read first.** Not to be settled by analogy to B1.

**What this obliges.** The batch that reaches `virtue.great_characteristic`
must read its passage verbatim, plus whatever the book says about
House-granted and type-granted Virtues, and report what the rules actually
require of a granted Virtue's preconditions. The decision is then made on that
evidence and appended here. Until it is, no batch marks either validator's
behaviour a defect, and no batch marks it correct.

---

## D3 — realm-scoped Magic Resistance

**Question.** `Susceptibility to Faerie Power` and `Susceptibility to Infernal
Power` halve Magic Resistance "against Faerie/Infernal power". The resistance
grid is per-Form only and structurally cannot express a realm-scoped halving,
so both Flaws compute nothing today and are surfaced as text.

**Ruling: surfaced-only is the honest answer.** No realm-scoped grid, no new
`Effect` variant.

**What this obliges.** Both entries are `uncomputed_rule`, with the halving
written out in `description` in **both** locales — the rule reaches the player
as text rather than as a number, which is exactly what `uncomputed_rule`
asserts. A batch reaching them classifies on that basis and does not report the
engine's inability as a defect.

**Precedent this sets.** An engine that structurally cannot express a rule is
grounds for `uncomputed_rule` with the rule written out — it is *never* grounds
for `narrative`. `narrative` remains a claim that the book states nothing
mechanical, and nothing about engine capability can make that claim true.
