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

## D17 — the Redcap's 300 apprenticeship points replace, they do not supplement

**Question (Q-108).** `virtue.lone_redcap` ships `restricted_ability_xp: 300`,
which `effective/xp.rs::build_flow_pools` treats as **additive**. F-439 was
rated `high` if the 300 *replaces* a life-stage block and `nil` if it
*supplements* — the only finding in the audit outright blocked on a question.

**Resolved from the source, 2026-09-21. It replaces.** This is a rulebook
reading rather than a product decision, so it is recorded rather than put to a
choice; the evidence is below and can be overruled on evidence.

**The passages.** ArMDE:4848, the *base* `virtue.redcap`:

> "You **have spent fifteen years as an apprentice**, and gained a **total of 300
> experience points in those fifteen years**."

ArMDE:4321, `virtue.lone_redcap`:

> "You **still** begin with 300 experience points **for your fifteen years spent
> as an apprentice**."

Both describe the **same** 300 points for the **same** fifteen years. "Still"
means the Lone Redcap keeps what a Redcap has despite losing his Mercer House
ties — not that he gains 300 on top of anything.

**The structural proof, from `rules/core/life_stages.json`.** The engine already
models exactly this shape for magi:

| Stage | Years | XP |
|---|---|---|
| `childhood` | 5 | 75 native language + 45 spread = **120**, a block |
| `apprenticeship` | **15** | **240**, a block |
| `later_life` | — | 15 per year |

**A magus's apprenticeship is a fixed block that replaces per-year later-life XP
for those fifteen years.** The Redcap's is the identical structure with a
different figure — fifteen years, 300 instead of 240. Reading it as additive
would make the Redcap the only character in the game whose apprenticeship is
funded twice.

**The arithmetic, which is what F-439 is about.** A companion has no
`apprenticeship` creation phase, so a 25-year-old Lone Redcap is funded
childhood (120) + later life for years 5-25 (20 × 15 = 300) — and the Virtue
then adds **another 300**. The book gives him roughly five years of ordinary
later life plus the 300 apprenticeship block. **F-439 is confirmed at `high`**:
the character is over-funded by around 225 experience points.

### A second defect fell out of the same reading

**`virtue.redcap` does not encode its 300 at all.** It carries only
`item_level_budget: 50`, though ArMDE:4848 states the 300 as plainly as Lone
Redcap does. So the two entries are wrong in **opposite** directions: the
**Major** Redcap is *under*-funded by a whole apprenticeship block, while the
**Minor** Lone Redcap is *over*-funded by one. Neither question anticipated
this, and it is filed here rather than lost.

### Remedy — this is D13's third mode, and it must wait for it

The fix is **not** to delete Lone Redcap's pool and **not** to add a matching one
to Redcap. Both entries need the **replacement** shape D13 named and did not
build:

> *Additive grant* (modelled) · *earmark of the normal budget* (D13) ·
> **replacement of a life-stage block** (F-428, F-439, this ruling — unmodelled).

`flaw.feral_upbringing` (F-428) is the same mode at the childhood block; these
two are it at the apprenticeship block. **Design all three together**, per D13's
closing note, or a third incompatible spelling appears.

---

## D16 — an absolute the engine will not model is text; a *soft* restriction is a warning

Two questions, answered together because the pair sets the general rule: **how
hard the engine pushes must match how hard the book pushes.**

### Q-05 — sex restrictions: text only, no model

**Scope, re-counted catalogue-wide** (B01 saw six; there are more): **20**
entries say "This Virtue is only available to male characters" — ArMDE:3382,
:3438, :3482, :3531, :3535, :3539, :3543, :3553 and twelve further sites. One
states a female-only exception (`virtue.baccalaureus`, ArMDE:3474: "can be taken
by a female character, but only if she is (or was) studying to be a physician at
Salerno"), and ArMDE:3438 adds "who must also be eunuchs". **None of the 20
carries the clause in data or in either locale's text.**

**Ruling (Norbert, 2026-09-21): `uncomputed_rule` + `description` in both
locales. `Entity` does NOT gain a sex field, and nothing enforces the
restriction.**

**What was and was not open.** D3 already settled the *classification* — an
engine that cannot express a rule is grounds for `uncomputed_rule` with the rule
written out, never `narrative` — so these six-plus entries were misclassified on
that ground regardless. What was open was whether to build the model, which is a
product decision and not a rulebook reading, which is why B01 escalated rather
than guessing.

**The reasoning, recorded so it is not re-litigated.** The restriction reaches
the player as text and the troupe applies it. Building the model would mean the
generator **refuses to build a character on the basis of sex** — a call the table
is better placed to make than the tool. The accepted cost is that the app
silently permits combinations the book restricts; the rule is surfaced, not
enforced.

**This is a deliberate non-goal.** A future pass, a future book or a future agent
that finds another "only available to male characters" clause should apply this
ruling, not re-open it.

### Q-123 — soft restrictions: a warning, never an error

**The passages hedge, and the data must hedge with them.** ArMDE:6494
(`flaw.night_terrors`): "This Flaw **normally** makes seasonal Laboratory work
impossible, and so is **not suitable** for magi." ArMDE:6514
(`flaw.oath_of_fealty`) is the same shape. A profile `forbidden_traits` row is
the only mechanism the engine currently offers, and it is a **hard block** —
which says something the book does not.

**Ruling (Norbert, 2026-09-21): emit a warning, never an error.**

**No new machinery is needed.** `IssueSeverity::Warning` and
`ValidationIssue::warning` already exist (`validation/mod.rs:73`, `:844`). The
issue names the passage, appears in `Advisory` mode where the player wants
guidance, and **never blocks a character the rules permit**.

**Why not a hard block.** Making the tool stricter than the rulebook is the same
class of error as the invented aging floor `7f5605a` was reverted for — a
constraint with no passage behind it. "Normally" and "not suitable" are advice.

**Why not text alone.** Both entries already carry the sentence in
`description` in both locales, so the rule does reach the player — but it reaches
them on the sheet, not at the moment they are choosing the Flaw for a magus,
which is when it matters.

### The general rule this pair establishes

| The book says | The engine does |
|---|---|
| "may not", "cannot", "only available to" — **absolute** | hard error — *unless* the ruling is deliberately not to model it (Q-05), in which case `description` text |
| "normally", "not suitable", "should" — **hedged** | **warning**, never an error |

Apply this wherever the book hedges, rather than escalating each instance.

---

## D15 — the three Corrupted entries are one mechanic and get one treatment

**Question (Q-93).** ArMDE:5847-5864 states the same mechanic three times — +3
for selfish or sinful use, −3 for neutral or selfless, ±5 experience on a roll
the modifier swung, "you may only take this Flaw once, though it can affect
multiple …". The catalogue models it three ways:

| Entry | Class | Effects |
|---|---|---|
| `flaw.corrupted_abilities` | `uncomputed_rule` | none |
| `flaw.corrupted_arts` | `creation_effect` | **none** |
| `flaw.corrupted_spells` | `in_play_effect` | `special_casting_mod: circumstantial` |

At most one can be right.

**Ruling (Norbert, 2026-09-21): all three `uncomputed_rule`, with the full rule
in `description` in both locales, and `flaw.corrupted_spells`' effect deleted.**

**Why downward rather than upward.** The engine computes none of it, and saying
so is the honest classification:

- The **±3 is circumstantial** — "selfish or sinful" versus "neutral or
  selfless" is a table judgement no engine decides.
- The **±5 experience swing** is in-play bookkeeping triggered by a roll the
  modifier changed, which character generation never evaluates.
- **Corrupted Abilities' ±3 is on an *Ability roll*, not a Casting Total.**
  Unifying *upward* would give it a `special_casting_mod`, an effect naming the
  wrong total — and Phase 0 records that no effect can name an Ability and a
  roll modifier together. That gap is real but general, and inventing machinery
  for a modifier whose condition is a GM judgement would buy nothing.

`RULES.md:5226-5229` already reaches this conclusion for Corrupted Arts, so this
generalises an existing reading rather than introducing one.

**What this obliges.**

1. **`flaw.corrupted_arts`** moves `creation_effect` → `uncomputed_rule`. It was
   one of the five effect-less `creation_effect` entries `README.md` flagged as
   "a lead, not a conclusion" on day one; this resolves it.
2. **`flaw.corrupted_spells`** moves `in_play_effect` → `uncomputed_rule` and
   **loses its `special_casting_mod`** — the only thing any of the three
   currently computes. Deleting a shipped effect needs its own test, because the
   Casting Total it silently contributed to changes.
3. **`flaw.corrupted_abilities`** keeps `uncomputed_rule` and is the model the
   other two move to.
4. **All three carry the full passage in `description`, both locales**, per D5 —
   including `flaw.corrupted_spells`' prerequisite, *"has learned at least 30
   levels of formulaic spells"*, for which `Prereq` has no variant. Per D3 that
   is grounds for `uncomputed_rule` with the rule written out, which is exactly
   where this ruling lands it.

**Interaction with D9, which is orthogonal and must not be conflated.** Each
passage says the Flaw "can affect **multiple** Abilities / Arts / spells"
(ArMDE:5851, :5857, :5863). These three entries are D9's motivating cases for the
**multi-valued parameter type**, and they still need it: the *classification*
says the engine computes nothing, while the *parameter* records which Abilities
the player chose — data the save must round-trip. `uncomputed_rule` and a
multi-valued parameter are both correct here, for different reasons.

---

## D14 — an ability reference in an effect must be able to name its parameter

**Question.** Raised by Norbert on reading D13's citation of `RULES.md:6362`,
which blesses a known over-permission as a "documented approximation". It should
not be blessed; it should be fixed.

**The root cause.** `Effect::AbilityAuthorization` (and every other effect that
names an Ability) carries `abilities: Vec<Id>` — **an id and nothing else.** An
Ability id is not always the whole reference: `ability.dead_language` carries
`parameter: "language"`, so "Latin" is an id *plus a parameter value*. The effect
vocabulary cannot express that, and two entries are wrong as a result, in two
different ways.

**Shape 1 — a parameterized Ability, over-authorized.** ArMDE:5867 gives
`flaw.covenant_upbringing` "You may take **Latin** at character creation". The
data authorizes `ability.dead_language`, which is `academic` (gated) with
`parameter: "language"` — so it permits **every** dead language, Ancient Greek
and Hebrew included. `RULES.md:6362` records this as deliberate.

**Shape 2 — the entry's own parameter ignored, which is worse.**
`virtue.student_of_realm` carries a `realm` parameter, so the player picks **one**
realm — and its authorization lists **all four** realm Lores (`dominion`,
`faerie`, `infernal`, `magic`) unconditionally. A magus who takes Student of the
Magic Realm is authorized for **Infernal Lore**. Nothing links the authorization
to the choice the entry already records. This is not an id-level proxy
limitation; it is a missing binding.

**Ruling (Norbert, 2026-09-21): make ability references in effects
parameter-aware, and do it before F-409's remedy adds ~30 more authorizations.**

**What this obliges.**

1. **An ability reference in an effect gains an optional parameter constraint**,
   in two forms: a **literal** value (`ability.dead_language` + `language =
   latin`) and a **binding** to the selecting entry's own parameter
   (`ability.realm_lore`-style + `realm = {realm}`, so Student of (Magic)
   authorizes Magic Lore alone).
2. **No such mechanism exists today.** The `{land}` / `{realm}` placeholders in
   the catalogue are **i18n-layer name templates only**; no effect references a
   parameter. So the binding is genuinely new, and its syntax should be designed
   **once**, together with D9's multi-valued parameter type, rather than twice.
3. **`RULES.md:6362` is rewritten, not amended.** It currently documents the
   approximation as acceptable and cites `ability_score_grant` as precedent for
   the same looseness. Both halves stop being true.

**Scope, stated because it is smaller than it sounds.** Exactly **one** gated
parameterized Ability exists in the catalogue — `ability.dead_language`. So
shape 1 has one possible carrier today. Shape 2 has one. **The reason to do it
now is timing, not volume**: F-409's remedy writes ~30 new authorizations, and
writing them against a shape known to over-permit and retrofitting later is the
expensive order.

### Hard sequencing constraint: D14 lands before or with D13

**D13's earmark names Abilities, so it inherits this defect on a brand-new
effect.** ArMDE:5791 lists Church Upbringing's five as "Artes Liberales,
**Latin**, Music, **Organization Lore: Church**, or Theology" — and two of the
five are parameterized:

| Ability | Category | Parameter | The book means |
|---|---|---|---|
| `ability.dead_language` | academic | `language` | **Latin** |
| `ability.organization_lore` | general | `organization` | **Church** |

Written against the current id-only shape, the earmark would let the 25 points go
to **any** dead language and **any** organization's Lore. So D13 must not be
implemented until an ability reference can name its parameter — otherwise the
ruling that exists to model a passage *exactly* ships an approximation on day
one.

---

## D13 — an earmark of the normal budget is a third XP mode, and it gets modelled

**Question (Q-92).** `flaw.church_upbringing`, ArMDE:5791:

> "The player **must spend 25 experience points from the normal budget** on
> Artes Liberales, Latin, Music, Organization Lore: Church, or Theology.
> **Unless the character has a Virtue that permits it, no other experience points
> may be spent on Academic Abilities.**"

Two clauses, neither expressible. `RestrictedAbilityXp` **grants** new points,
which would turn a Flaw into a benefit; `AbilityAuthorization` is unbounded,
which would permit the unlimited Academic spending clause 2 forbids. The entry
ships `narrative` with no effects, so **a player who does what the Flaw mandates
gets a hard `ability_category_requires_virtue`** — a legal character the app
refuses to build.

**Ruling (Norbert, 2026-09-21): add a `from_normal_budget` earmark to
`RestrictedAbilityXp`.** Model both clauses exactly rather than approximating.

**Sizing, stated because it argues the other way and was overruled.** "from the
normal budget" appears **exactly once in the rulebook** — this entry. So this is
an engine change for one entry in 655, and the cheaper option (an
`ability_authorization` over the five named Abilities, with the rule as text) had
a documented precedent at `RULES.md:6362`, where Covenant Upbringing accepted the
same kind of over-permissive approximation. Norbert chose the exact model.

### Why it fits better than it looks

**Authorization comes free, and this is the part that makes the design tidy.**
`types.rs:1034-1035` already records that "`Effect::RestrictedAbilityXp` pool
already implies permission for what it funds … since the grant would otherwise be
unspendable." So an earmark naming the five Abilities **authorizes exactly those
five** — and every *other* Academic Ability stays gated by
`categories_requiring_virtue`. **That is clause 2, precisely, with no second
effect and no new authorization.**

### What this obliges

1. **`from_normal_budget: bool`** on `RestrictedAbilityXp`, `#[serde(default,
   skip_serializing_if)]` so the twelve existing carriers' JSON is unchanged.
2. **The flow graph gains one edge shape.** Today `effective/xp.rs::build_flow_pools`
   treats every restricted pool as a *source*. An earmark is not additional
   supply — it is a constraint on supply the character already has, so it draws
   from the general pool: `general → earmark → eligible spends`, rather than
   `source → pool → eligible spends`. The total budget must not rise.
3. **`restricted_xp_unspent` applies unchanged.** The earmark *must* be spent;
   the existing warning (`validation/life_stage.rs:745`) already says so, and
   here it is saying something the rulebook actually requires rather than
   nagging.
4. **`flaw.church_upbringing` becomes `creation_effect`** carrying the earmark,
   with the full passage in `description` in both locales per D5 — the "unless
   the character has a Virtue that permits it" escape is not modelled and must
   reach the player as text.
5. **Test-first.** The red that matters: a character with this Flaw who spends
   none of the 25 must fail, and the total budget must be unchanged by taking the
   Flaw.

### The larger thing this names

There are now **three distinct XP modes** in the rules, and the engine has had
only one of them:

| Mode | Example | Status |
|---|---|---|
| **Additive grant** — new points on top of the budget | Educated, Warrior, Privileged | modelled |
| **Earmark** — N of the *existing* budget, constrained | Church Upbringing (ArMDE:5791) | **this ruling** |
| **Replacement** — substitutes a life-stage block | Feral Upbringing (ArMDE:6112) | **F-428, unmodelled** |

F-428's defect is that `flaw.feral_upbringing`'s 120 XP *replaces* the 120-point
childhood block and the engine stacks them, yielding 240. That is the **third**
mode, not this one, and it needs its own shape — but the two should be designed
together, because a `from_normal_budget` flag and a `replaces_life_stage` flag
are the same kind of answer to the same kind of question, and building one in
ignorance of the other invites a third incompatible spelling.

---

## D12 — Hermetic V/F are intrinsic or trained, and trained requires a magus

**Question** (Q-90, "one ruling with 122 consequences").
`flaw.deficient_technique` carries `prerequisites: has virtue.hermetic_magus`;
its twin `flaw.deficient_form`, four lines away in the book, carries none.
Either the prerequisite is a stray to delete, or it is right and 121 other
Hermetic entries lack it.

**Ruling (Norbert, 2026-09-21): neither. Differentiate *intrinsic* from
*trained*. Techniques and Forms are available only to the hermetically trained
Gifted, so the Deficient Art Flaws are trained, the prerequisite is CORRECT, and
`flaw.deficient_form` is the entry that is missing it.**

**This reverses the reading I was about to record, and the error is worth
keeping.** ArMDE:2870 says a Gifted non-magus may take Hermetic V/F "which
relate to **intrinsic ability** rather than background or training", and I
classified a deficiency in a Technique or Form as intrinsic — it sounds like an
innate magical trait. It is not. **A Technique or Form only exists because of
Hermetic training**; a Gifted character who never had an apprenticeship has no
Arts at all, so there is nothing to be deficient *in*. I reasoned from the shape
of the words instead of from what the game object is — the same failure mode this
audit has caught repeatedly in its own agents.

**Confirmed structurally, not just textually:** only the **magus** profile has an
`arts` creation phase. `companion`, `grog` and `mythic_companion` do not
(`character_types.json`), so a Gifted companion cannot hold an Art in this app
under any circumstances.

### The criterion — mechanical, not a judgement call

> **Does this entry operate on a game object a character can only have after
> Hermetic training?**

- **Intrinsic** — it operates on **The Gift itself**. These are exactly the three
  entries that already gate on `virtue.the_gift`: `flaw.blatant_gift`,
  `virtue.gentle_gift`, `flaw.suppressed_gift`. **The catalogue already had the
  right instinct here and never generalised it.** A Gifted non-magus may take
  these.
- **Trained** — it operates on Techniques, Forms, spells, Casting Totals, Lab
  Totals, Parma Magica, Arcane Connections, certámen or Twilight. **Magus only.**

### What this obliges

1. **Classify all 122 `hermetic`-category entries** as intrinsic or trained.
   Scale, measured: **41** have effects touching arts/casting/lab/spell/
   resistance/warping and are mechanically obvious; **56** carry no effects at
   all and need their passage; the remaining ~25 need checking. **The audit has
   just read all 122**, so derive this from the batch files rather than
   re-reading the book.
2. **Every trained entry gets the magus gate.** `flaw.deficient_form` is the
   immediate, already-identified case — its twin has it and it does not.
3. **Use `Prereq::IsMagus`, not `Has(virtue.hermetic_magus)`.** Both spellings
   ship today — `flaw.primogeniture_lineage` uses `{"kind":"is_magus"}`,
   `flaw.deficient_technique` uses the `Has` form — and they are equivalent only
   because the magus profile lists `virtue.hermetic_magus` in `required_traits`.
   `IsMagus` reads the profile's own flag (`types.rs:2685`) and is the direct
   expression; normalise the one `Has` spelling to it.
4. **No new field and no new category.** The existing `Prereq` machinery carries
   it, so this is data plus one normalisation — no engine change.
5. **Two named entries ride along, and one of them is out of every batch span.**
   **D24** rules `flaw.weak_parens` and `virtue.skilled_parens` trained — a
   *parens* exists only through apprenticeship — so both take the gate in this
   pass. The Virtue appears in no batch record; do not let the pass skip it.
6. **Finish with the positive check D24 asks for.** After classifying, confirm
   that no entry reachable by a **Gifted non-magus** computes against a budget
   that only exists for a magus. Weak Parens is the worked example (it subtracts
   its −60 from `later_life_xp`); the pass must say whether it is the only one.

**What this does not settle.** Whether any *intrinsic* Hermetic entries exist
beyond the three Gift ones. The classification pass will answer that; if the
answer is "no others", then trained-versus-intrinsic collapses into "everything
Hermetic except the three Gift entries requires a magus", which would be simpler
still and should be stated that way if it holds.

---

## D11 — the Reputation model is right; the data is wrong, and `score` gets enforced

**Question** (Q-43, Q-73, Q-80, Q-127, and Phase 0's "is a granted Reputation's
`score` a ceiling, a fixed value or a suggestion?"). `corrections.md` § 3.11 was
blocked on all five.

**Ruling (Norbert, 2026-09-21): the model does not grow. The fix is data-only —
*plus* `score` semantics become deliberate and enforced instead of accidental.**

### Why the model does not grow — ArMDE:1093 reverses the premise

> "Reputations have **a score, a content, and a type**. … They don't determine
> how people react to characters they have heard of, **as that depends on what
> they think of what they've heard**."

The book gives a Reputation **three** components, and `types.rs::Reputation`
carries exactly those three. So:

- **Q-43 (polarity) is answered no.** "A bad Reputation at level 3" describes the
  **content** — *Unclean*, *Usurer* — not a fourth field. The book explicitly
  refuses to make good-versus-bad a mechanical property. Adding a polarity field
  would invent a mechanic the rulebook does not have, which is the error
  `7f5605a` was reverted for.
- **Q-80 (wildcards) is already solved.** `GrantsReputation.kind: Option<..>`
  with `None` means player-chosen, and `virtue.famous` (ArMDE:3861) uses it.
- **Q-73 (organization scope) is answered no.** ArMDE:1093 names Local,
  Ecclesiastical and Hermetic as the types, with Academic alongside, and the
  ease-factor table has exactly those columns. "Among Templars", "among members
  of his bloodline", "among the Jewish community" are **content**, not types —
  a new enum member would be an audience the engine has no distance rules for.

### `score` — enforced, exact by default, bounded where the book states a range

**ArMDE:2514 settles only entitlement** — "Characters only start with a
Reputation if they choose a Virtue or Flaw that grants one" — and says nothing
about the number. **The semantics are therefore per-entry, from each granting
passage**, and the passages were surveyed rather than assumed:

**31 entries grant a Reputation. Exactly one states a range**:
`flaw.outsider_major` / `_minor`, ArMDE:6554 — "a bad Reputation of **level 1 to
3** (depending upon how easy it is to identify you)". Every other passage gives
an exact level ("a level 4 bad Reputation", "a Reputation level of 3", "an
Academic Reputation of 1").

**What this obliges.**

1. **`validate_reputations` gains a score check.** Today it validates kind and
   count only (B18's F-525 confirmed this against `selections.rs:585-617`), so
   the grant's `score` is enforced nowhere — which is *why* Outsider's invented
   3 and 1 shipped without anything objecting (F-486).
2. **Exact is the default.** The entity's Reputation score must equal the
   grant's.
3. **An optional upper bound carries the one exception.** Add
   `max_score: Option<u8>` to `Effect::GrantsReputation`, `skip_serializing_if`
   so 30 of the 31 entries' JSON is unchanged. Absent = exact; present = the
   entity's score must lie in `[score, max_score]`. Outsider becomes
   `score: 1, max_score: 3` on **both** magnitudes — :6556's Minor version says
   "You **still** have the bad Reputation", i.e. the same range.
4. **The § 3.11 findings then become plain data fixes**: add the
   `grants_reputation` a passage states and the entry lacks (F-408
   `flaw.excommunicate`), correct invented levels (F-486), and stop hardcoding an
   audience the passage leaves open (F-450 `flaw.infamous` pins `kind: "local"`
   where ArMDE:6312 names none, while its twin `virtue.famous` correctly ships
   the wildcard).

**Consequence to expect, consistent with D10.** Enforcing a field that was never
checked **will reject characters that load cleanly today** — a save whose
Infamous Reputation was typed as 5 becomes invalid. Same treatment as D10:
validation reports it, `Enforced` blocks, no migration mutates the save, and the
player corrects it. No `SCHEMA_VERSION` bump is needed for the entity side; the
optional `max_score` is additive on the ruleset side only.

---

## D10 — once means once: invert the multiplicity default

**Ruling (Norbert, 2026-09-21): "if only once, it should be only once."** The
default must be **once**, and a repeat must be declared.

**The defect.** ArMDE:2814 is explicit in both directions:

> "A Virtue or Flaw may be taken more than once **only if the description
> explicitly allows it**. **Most Virtues and Flaws may only be taken once.**"

`PointItem::max_total` defaults to **`u8::MAX`** (`types.rs:2209-2212`,
`default_max_total`), and `ParameterDef::max_per_value` likewise. **The book's
default is once; the model's default is unlimited.** They are exactly inverted,
and the model's side is the one with no rulebook behind it.

**Why this is a ruling and not a bug report.** D9 originally handled this as
author discipline — "every added parameter carries `max_total: 1` unless the
passage permits repeats". That is the wrong shape, because it makes correctness
depend on remembering, on every entry, forever. It is also *demonstrably*
insufficient: forgetting it once is exactly how `flaw.servant_of_the_land`
(F-522) came to stack a **Major** Flaw without limit, and how
`flaw.vulnerable_magic` (F-541) came to license 255 identical copies. A default
that has to be overridden to be safe will be wrong again.

**What this obliges.**

1. **Invert `default_max_total` to 1**, and `default_max_per_value` with it.
   Absent means once. This is test-first and the reds are the point: every entry
   that legitimately repeats will fail until it declares so.
2. **Declare the repeats explicitly.** **42 entries** currently rely on the
   unlimited default (`jq`: `parameters != null and max_total == null`). Each
   must be read against its passage and either given an explicit
   `max_total`/`max_per_value` or left to the new default. The classic repeaters
   — `virtue.puissant_ability`, `virtue.affinity_ability`,
   `virtue.great_characteristic`, `virtue.minor_magical_focus`,
   `virtue.major_magical_focus`, `virtue.deft_form` — will need one; entries like
   `virtue.mythic_blood` and `flaw.servant_of_the_land` will not.
3. **`is_default_max_total`'s serialization skip flips meaning**, so the
   canonical-JSON output changes for every entry that gains an explicit value.
   Expect a large, mechanical diff and check it is exactly the intended set.
4. **This subsumes D9's part 2** and reaches further: D9's version covered only
   entries *gaining* a parameter, while this covers all 42 already on the
   default.

**Existing saves: validation reports it, and `Enforced` blocks the character.**
(Norbert, 2026-09-21.) A save written today may legitimately hold several copies
of an entry the new default caps at 1 — the player did nothing wrong, the model
permitted it. **No migration mutates that save.** It loads, validation reports
the excess, and the three existing modes do the rest: `Enforced` blocks,
`Advisory` warns, `Silent` suppresses. The player removes the extra copy
himself.

Rejected alternatives, and why: **migrating the save** — dropping the extras
automatically — is silent data corruption, which `CLAUDE.md` rates in the top
severity class, and it discards a choice the player made legitimately.
**Grandfathering** would make the same entry legal or illegal depending on when
the file was written, which is a rule no rulebook states and nothing could
explain to the user.

**This is why D10 needs no schema change.** The save *format* is untouched —
only what validation says about it — and the machinery already exists and is
already grant-aware: `validation/selections.rs:274-303` enforces `max_total` via
`CODE_TOO_MANY_SELECTIONS`, and `validate_duplicate_selections` (`:237-262`)
enforces `max_per_target` via `CODE_DUPLICATE_SELECTION`. **The whole of D10 is
a change to two default functions plus the explicit declarations on the entries
that legitimately repeat.** Only D9 part 3's multi-valued parameter type needs a
`SCHEMA_VERSION` bump; the two are independent and should not be bundled.

---

## D9 — every stated choice is recorded, and the parameter model gains a multi-valued type

**Question** (Q-86, Q-88, Q-31, Q-36, Q-95, Q-122, Q-128). The book repeatedly
tells the player to pick something — Deleterious Circumstances' circumstance
(ArMDE:5919, "a state, a target, or a place"), Curse of Slander's section of
society (:5883), Demonic Familiar's kind of demon (:5930), Ability Block's class
of Abilities (:5653). Most of those choices are recorded nowhere. Should every
such entry carry a parameter?

**Ruling (Norbert, 2026-09-21): yes — record every stated choice, *and* extend
the model with a multi-valued parameter type.**

**Starting state, verified by `jq`.** 51 of 655 entries carry parameters, across
ten domains (`text` 19, `form` 9, `ability` 7, `realm` 5, `enumerated` 4,
`art`/`category`/`characteristic`/`item` 2 each, `technique` 1). **Every
parameter in the catalogue is single-valued `"type": "ref"`.**

**What this obliges, in three parts.**

**1. Data — every entry whose passage tells the player to choose gets a
parameter.** Use `enumerated` where the book lists the options ("a warder,
teacher, or paramour"); use `text` only where the choice is genuinely open.
`corrections.md` § 3.7 lists the 26 findings already on this; it is unblocked by
this ruling but is **not** the full set — the ruling is catalogue-wide and the
slice must re-derive which entries state a choice.

**2. Multiplicity — superseded by D10, read that instead.** This ruling
originally required every added parameter to carry `max_total: 1` "unless the
passage explicitly permits repeats". Norbert rejected that shape on the spot —
*"if only once, it should be only once"* — and **D10 inverts the model default**
so absent means once and a repeat must be declared. That is strictly better and
strictly wider: author discipline was what failed in F-522 and F-541, and D10
also reaches the 42 entries already riding the unlimited default, which this
clause did not. **Do not add `max_total: 1` entry-by-entry under D9; land D10
first and let the default do it.**

**3. Engine + schema — a multi-valued parameter type.** This is the part that is
not a data change. The three Corrupted entries let the player pick an open-ended
*set*: ArMDE:5851 "you can choose to have it affect **multiple Abilities**",
:5857 "it can affect **multiple Arts**", :5863 "as many of the character's
**spells** as you wish". A single-valued `ref` misstates that rule in the
opposite direction to recording nothing. The ruling adds the type. It pulls in:

- `types.rs::ParameterDef` — a second `type` beyond `"ref"`, with an exhaustive
  `match` so every consumer is a compile error until handled.
- **`SCHEMA_VERSION` bump and a migration**, since a saved selection's parameter
  value changes shape.
- **UI** — a multi-select picker beside the existing single pickers.
- **`max_per_target` grouping semantics.** If a parameter holds a set, "the same
  tuple" needs a canonical form, or `{A, B}` and `{B, A}` read as two different
  selections and the once-only rule fails again. `CLAUDE.md`'s canonical
  serialization convention ("sort arrays by `id`/`ref` before writing JSON")
  already supplies the answer — apply it here explicitly rather than by
  coincidence.

**Why this over the cheaper options.** The choice not being in the save is
**data loss**, which `CLAUDE.md` rates in the top severity class alongside wrong
rules output; and a description-only fix (D5's shape) leaves two copies of a
Flaw indistinguishable and the Markdown export unable to print the choice at
all. YAGNI argues against a new type for three entries, and that argument was
heard and overruled: the alternative is shipping a model that cannot state what
the rulebook says.

**Sequencing.** Part 1 and part 2 are data and can proceed together. Part 3 is a
separate, test-first engine slice and should land **before** the three Corrupted
entries are touched, so those are worked once rather than twice.

---

## D8 — a roll-free supernatural capability is a rule, not colour

**Question** (Q-76, and with it Q-64, Q-17, Q-61, Q-126). A Supernatural Virtue
states a capability with no digit, no roll, no Ability grant and no reference to
a rules subsystem — `virtue.voice_of_the_land` (ArMDE:5221): *"The character can
speak with any creature whose natural habitat is a particular environment
associated with this Virtue (including animals and magic beings), and the
character is not normally perceived as either a threat or a prey object by these
creatures."* Is that `narrative` or `uncomputed_rule`? Nothing in
`rules/source/` decides it; it is a taxonomy question.

**Ruling (Norbert, 2026-09-21): `uncomputed_rule`. A capability is a rule.**

**Scale.** Of the **115** `supernatural`-category entries, **48** are
`narrative` (`creation_effect` 33, `in_play_effect` 9, `uncomputed_rule` 25) —
verified by `jq`. All 48 are in scope; this is one ruling, not 48 judgements,
because moving one entry would make the catalogue *less* consistent.

**Why.** Three independent supports, and they agree:

1. The book states something that **changes what the character can do** at the
   table — a thing the rules elsewhere require magic for. That is mechanical in
   the sense `narrative` denies.
2. **D3 already forbids** using the engine's incapability as grounds for
   `narrative`, and a roll-free capability is the purest case of a rule the
   engine cannot express.
3. **ArMDE:2960-2962 attaches a realm association and same-realm-aura Warping
   immunity to every Supernatural Virtue.** Read literally, no Supernatural
   Virtue states nothing mechanical — so the `narrative` claim is false for this
   cohort on grounds that do not depend on the capability argument at all.

**What this obliges.**

1. **Reclassify the 48** `supernatural` + `narrative` entries to
   `uncomputed_rule`. Enumerate them from the data, not from this list.
2. **A separate `description` is owed only where the `summary` does not already
   carry the rule.** `RULES.md:4728-4731` settles this: `flaw.missing_ear` gained
   no `description` because its passage is a single sentence its `summary`
   already carries in full, and a `description` would be a byte-identical
   duplicate. Most of these 48 are one-sentence capabilities. **So this is
   largely a relabelling, not 96 new pieces of text** — check each, do not
   assume either way.
3. **It lands on top of the screen problem, not beside it.** Every reclassified
   entry must satisfy `every_uncomputed_rule_entry_states_its_rule_in_every_locale`,
   and these entries carry **no signed number and no botch term** — so they
   depend entirely on `MECHANICAL_PHRASES`, which today has **no capability idiom
   in either language**. Per `corrections.md` § 2.1 the screen must grow first,
   and per § 2.1a the `flaw.wrathful_*` range must be fixed before the swept
   block widens. **This ruling adds a third requirement to that sequence: the
   screen needs a capability family before any of the 48 can land.**

**What it does not settle.** Whether `narrative` should exist at all for
Supernatural entries — 48 of 115 moving out leaves the class thinly populated
there, and the remaining ones deserve a look during Phase 2 rather than a
presumption of correctness.

---

## D84 — the second try-out's findings (N1–N11)

**Norbert, 2026-10-04** (`tmp/tryout-2026-10-04.md`).

1. **The native language matches by catalogue value, not by spelling** (N4a; a
   wrong-rules bug found while planning N4). The plan's `native_language` stays free
   text (N4b makes it a list value), but it is resolved once against the native
   Ability's catalogue names in every locale (`catalogue.rs::resolve_typed_instance`,
   trimmed and case-folded, the `instance_is` rule); text naming no value keeps a folded
   text match. The 75-XP pool, the missing-score warning, `SlotIsNativeLanguage` and the
   duplicate-slot key all compare through it, and `apply_package` writes `Catalogued`
   where the text names a value. Before, a list-picked Arabic row was unfunded by a plan
   saying "Arabic", and so was every typed one after the load fold. The UI mirror folds
   case and whitespace only (one locale's names); a cross-locale spelling is caught by the
   engine on Apply.

3. **A split "Minor or Major" entry shows its magnitude in square brackets** (N11).
   Where the book prints ONE entry taken at either magnitude and the catalogue splits
   it into two items, the name carries the magnitude in brackets: "Ambitious [Major]",
   "False Power [Minor]: {virtue}", DE „Ehrgeizig [Groß]"/„[Klein]". Brackets keep it
   apart from a parameter in parentheses ("Potent Magic [Minor] (Fire)"). A name whose
   book spelling contains the magnitude ("Minor Magical Focus") is unchanged. A display
   convention, not terminology: D31 is untouched, and `de_vf_names.rs` strips the bracket,
   so these names are now compared with the tables' base-name rows (no mismatch). Guard:
   `vf_magnitude_brackets.rs`.

---

## D83 — the post-deadline rulings and the try-out findings

**Norbert, 2026-10-03** (`tmp/questions-after-deadline.md`, `tmp/tryout-findings-2026-10-03.md`).

1. **Incompatible Arts copies may not share a combination** (amends D81.8;
   try-out findings 20, 21). Per ArMDE:6292, "may be taken repeatedly with
   different combinations". A copy naming any Technique+Form pair that another
   copy names, in either position, is an error
   (`param_group_shared_across_copies`), one per offending copy pair. A copy is
   judged on its completed combinations only. A whole repeated pair stays
   `duplicate_selection` alone. Single Arts may recur: grouped keys skip
   `max_per_value`, and the picker greys nothing.
3. **Potent Magic counts toward the creation cap only within its field**
   (amends D1; after-deadline answer 3). Its Lab Total bonus (+3 Minor / +6
   Major, ArMDE:4746, :4748) counts only for a spell marked
   `within_potent_field`, and only while a Potent Magic Virtue is held, so a
   stale mark adds nothing. Across Virtues the larger bonus applies, not the
   sum (ArMDE:4742). The other eight D1 entries stay flat. `spell_caps` gains
   `within_potent_field_cap` and `within_focus_and_potent_field_cap`. A save
   that was legal can become invalid; validation reports it, with no migration.
2. **Repeated free text compares folded** (amends D81.9; try-out finding 25).
   Every across-copies check (`max_per_target` duplicates, `max_per_value`)
   compares a free-text value, and a parameterized Ability's instance text,
   case- and whitespace-insensitively through `catalogue.rs::fold_free_text`.
   Ids and `multi_ref` sets compare exactly; values are stored as typed. Focus
   Power's `focus` declares `max_per_value: 255` (ArMDE:3903). Of the other ten
   repeatable text items, none may repeat a value. Norbert ruled the three unclear
   ones: Deteriorating Power, Potent Magic and Special Circumstances each take a
   different value per copy.
4. **An unset Characteristic is 0** (amends D81.2; ruling S2).
   `Prereq::CharacteristicMin` evaluates every Characteristic: the stored score,
   or 0 when there is no entry, plus all free deltas. The UI deletes the entry
   at 0, so "unset" and "0" are one state. Supernatural Beauty and Envied Beauty
   with Presence unset are now `prereq_not_met` (ArMDE:5095, :6014).
   Uncontrollable Strength with Strength unset is legal, and refused under Dwarf
   (ArMDE:6909, :5998).
5. **Ability and Art minimums test the held score** (ruling F-B).
   `AbilityMin`/`ArtMin` compare the bought score or a grant floor (Second Sight
   1, ArMDE:4890), never `ability_bonus`/`art_bonus`. Puissant applies "whenever
   you use it" (ArMDE:4816), and holding a minimum is not a use (ArMDE:4389).
   Broken Vessel's category and Art floors (ArMDE:5755) use the same held score.
   `AbilityMin{dead_language, 5}` stays instance-blind for now.
6. **The aging drop threshold is the actual Characteristic** (ruling F-A).
   ArMDE:16579 compares the aging points with "the absolute value of the
   Characteristic"; a free delta raises the Characteristic itself (ArMDE:3989).
   The threshold for each drop is |bought + delta - drops so far|, and the value
   after aging is bought + delta - drops. The legacy `aging_reductions` fold
   shares the helper. Aged saves with a free delta read higher than before.
7. **A Mythic Companion must choose a type, and only the type grants its
   status Virtue** (ruling F7; ArMDE:2846 "You must take the Free Virtue
   defining which type of Mythic Companion you are", :2637). An unset type is
   the error `mythic_type_unset`, filed under the Type step. Norbert: "Devil
   Child and all the other Vs for myth comps should only be granted by
   selecting the appropriate mythic companion type. not selectable by just
   anyone." The four status Virtues carry `mythic_status: true`. A bought copy
   is the error `mythic_status_virtue_bought`, and the picker greys them out.
8. **Post-Gauntlet lab seasons pack into full lab years** (ruling F1). A year
   holds four seasons, all of which may be lab work (ArMDE:2482), and at most
   three of them are charged. The stored total is read as packed into full
   years: charged = floor(s/4)*3 + min(s mod 4, 3), with s capped at 4 x years.
   The book's Darius is worth 240 (ArMDE:2486, :2488). Storing real per-year
   seasons is open-todos row 56 (a save-format change, later).
9. **Maximum age 500** (after-deadline answer 7). This is an app limit, not a
   rule. `rules/core/aging.json` declares `max_age: 500`, and a value below the
   first aging-roll age fails the load. The age, apparent-age and birth-year
   inputs take their bounds from it, as do the derived age and
   `aging_schedule`. Loading clamps age, apparent age and a too-early birth year
   together; the next save writes the clamped values, as with the aura clamp,
   and the schema is not bumped. Moving the saga year forward recomputes nothing
   (D3.3), so the birth year can fall out of step until the file is reopened.
10. **Dead and Living Language have separate catalogues** (amends D59; try-out
    finding 6). Dead = Latin, Hebrew, Gothic (ArMDE:3565); Living = Arabic,
    Greek, Persian, Aramaic. An Ability names its catalogue in an optional
    `catalogue` field. Educated (Islamic/Hebrew) now fund Living Language, and
    Turb Trained offers only the dead languages (ArMDE:5181). The migration to
    schema 22 moves a misplaced language in an older save to the Ability whose
    catalogue lists it. A collision keeps the higher score, and a one-time
    notice names each move. A newer save holding a value outside its catalogue
    gets `ability_parameter_outside_catalogue`.
12. **The language checks name the actual language** (try-out finding 8;
    ruling F5). The magus minimum and the recommended Latin 4 require Latin
    itself. The Academic requirement is Latin or Hebrew (Dead) or Greek or
    Arabic (Living) at 3 (ArMDE:7151, :7432). A typed name in any shipped locale
    counts as that catalogue language.
13. **Catalogue load notices only on a real upgrade** (try-out findings 13,
    14). Typed text naming a catalogue entry is normalised silently on every
    load. The "recognized" and "not recognized" notices fire only for a file
    saved before schema 18, judged by the version read before any fold runs.
    Free text such as "Native language" never warns in a current save.
14. **A fixed grant's text value is a catalogue value.** Tremere's focus
    (ArMDE:2281) is `magical_focus.certamen`, named "certamen" (EN) and
    "Certamen" (DE). A fixed grant may not carry literal text, and this is
    checked at load.
15. **The rulebook examples ship** (try-out finding 12). 24 book templates
    ship as `examples/<template>.armc` beside `rules/`, in the installers and
    the portable layout. They are in the current form (schema 22, catalogue
    ids, no notice on open), generated from the test fixtures and guarded byte
    for byte. A NOTICE gives the CC BY-SA attribution. The incomplete
    `magus_criamon` variant stays a test fixture only.
16. **The coverage target counts production code only** (amends D82.1;
    after-deadline answer 2). `tarpaulin.toml` measures `crates/*/src`. The
    target is 95%: 94.66% before this round, 95.73% after nine mock-runtime
    app tests. The rest is mostly Tauri-runtime and native-dialog code, which
    only e2e reaches.
11. **Spell codes show their requisites** (try-out finding 22; after-deadline
    answer 4). The engine builds a spell's code once in
    `LocalizedRuleset::spell_code`. Technique requisites go in parentheses after
    the Technique and Form requisites after the Form, several separated by ", ",
    in data order: "Cr(Re)Ig 30" (ArMDE:19301), "MuTe(Aq, Co, An) 25"
    (ArMDE:19241). The Spells tab and the export add the level a space apart.
    The export puts "· Focus"/"· Fokus" and "· Potent" after the code of a
    marked spell; there is no new column (D73.2).

---

## D82 — the coverage target and a dead Ruleset field

**Norbert, 2026-10-02.**
1. **D76's 95% means the whole tarpaulin report** (all workspace crates), not
   just `arm-rules/src`, which already stands at 97.1%.
2. **`Ruleset.ranges_beyond_touch` is removed** from the serialized Ruleset
   and the TS type. Since D81.5 the per-spell caps fold the beyond-Touch
   halving in the engine, so nothing reads the field.

---

## D81 — the night audits' exclusions and Merinita's Warping Point

**Norbert, 2026-10-02** (`tmp/incompat-audit.md`, `tmp/houses-audit.md`).
1. **Incompatible Arts excludes the Deficiencies outright** (ArMDE:6292, "may
   not be combined with a Deficiency"): a flat `incompatible_with` on both sides.
2. **A Characteristic-floor prerequisite** is built for Supernatural Beauty
   (ArMDE:5095), Envied Beauty (:6014) and Uncontrollable Strength (:6909).
3. **Broken Vessel** (ArMDE:5755) requires a score in a Supernatural Ability or
   an Art, through a new predicate.
4. **Merinita's Warping Point** (ArMDE:2280) is a conditional grant: it applies
   to a magus of the House who holds no faerie-related Virtue or Flaw.
5. **The creation level cap becomes per spell** (Q12). It folds the spell's
   requisites (ArMDE:2465, :12313) and, for a spell within the Magical Focus,
   applies the focus doubling (ArMDE:4403). The engine cannot match a spell to
   a free-text focus, so the player decides. The picker offers an "add within
   focus" action for a spell that fits only within the focus cap, so it is
   added already marked.
6. **Row 55 stays as built:** a granted copy's realm shows read-only, and a
   stated grant realm beats the concept realm.
7. **"In such as case" (ArMDE:4235)** is corrected here as a same-line edit,
   and recorded upstream.
8. **Incompatible Arts records its two combinations** (ArMDE:6292) as four
   selects per copy (Technique 1 + Form 1, Technique 2 + Form 2). A copy that
   repeats another's pair, in either order, is refused. The engine flags both
   combinations as unusable in the Casting and Lab totals and on the spells
   that use them, requisites included ("even if one or both are requisites").
9. **Focus Power** copies may share a scope or found separate powers
   (ArMDE:3903, "the points gained may be combined"); the current data stands.
10. **Mists of Change** keeps Duration Year (ArMDE:13634, "D: Sun & Year").
11. **MAG7 stays resolved:** a wind that carries the caster is weather, so
    Wings of the Soaring Wind is inside the Mercere's Weather focus.
12. **The D31 table revert is only recorded** upstream, not done. The seven
    German-name guard exceptions stay.
13. **Devil Child's required-Flaw slot** gains `require_categories: ["story"]`,
    like its siblings.
14. **"Faerie-related" for Merinita** (ArMDE:2280) means either an entry whose
    realm resolves to Faerie, or one of three entries outside the realm
    system, flagged explicitly: Faerie Friend (ArMDE:6052), Faerie Upbringing
    (:6056) and Susceptibility to Faerie Power (:6819).
    - The realm case covers Faerie Blood and Strong Faerie Blood, plus Bound
      to / Realm Stigmatic / Necessary Aura / Folk Magic when their realm is
      Faerie.
    - The House's own Faerie Magic grant does not count.
15. **A spell that uses a barred Incompatible Arts combination**, directly or
    through a requisite, is a creation-time ERROR (ArMDE:6292, "completely
    unable to use").
16. **One Incompatible Arts copy naming the same combination twice is an
    ERROR**: its "two combinations" (ArMDE:6292) must differ.
17. **A spell marked within a focus the character no longer holds** gets a
    non-blocking WARNING. The mark is inert (the cap and Casting Total ignore
    it) and stays saved.

---

## D80 — X6c's three open points (`tmp/x6c-verdicts.md`)

**Norbert, 2026-10-01.**
1. **Rector/Proctor takes no parameter.** A master leads his faculty and a
   student leads his nation (ArMDE:6673). The required university Social Status
   Virtue already says which he is, and the description states the rule.
2. **Demonic Familiar's role is free text.** The book's list is only examples
   (ArMDE:5930).
3. **Fida'i and Lasiq get an OPTIONAL "cover social status".** It applies only
   while "far from home on a mission" (ArMDE:4235; the Fida'i is covered by
   "As for a fida'i"). It is never reported missing. This is the one exception
   to D70's "every new parameter is required".

---

## D79 — Potent Magic gets its own "in field" marker; F-256 is text

**Norbert, 2026-10-01** (`docs/vf-audit/design-x7-relic-and-ct-mirror.md`).

1. **Potent Magic's +3/+6 applies only in its field** (ArMDE:4744-4748), and
   the field is a free-text theme, so the Arts cannot decide membership.
   - Today the Casting Total bonus reaches every spell. That is wrong output.
   - A known spell gains a second marker, "within the Potent Magic field",
     shown only when the character holds Potent Magic. Its Casting Total adds
     the bonus.
   - It is separate from X10c's Magical Focus marker, because the two fields
     can differ.
   - The Lab Totals grid gets its own "within the Potent Magic field" figure,
     so D4's combined within-focus figure separates too.
   - Both are additive fields, so there is no schema bump.
   - **Each spell marker shows only when its Virtue is taken**: the focus
     marker with a Magical Focus, the Potent Magic marker with Potent Magic
     (Norbert, same day).
2. **F-256: no relic machinery.**
   - Relic and Powerful Relic get a composed description from ArMDE:17619-17623,
     in the `COMPOSED_DESCRIPTIONS` shape (D70's True Faith precedent).
   - Both become `uncomputed_rule` (D67).

---

## D78 — X9c's sweep results (`tmp/x9c-verdicts*.md`)

**Norbert, 2026-10-01.**
1. **Companion Animal** ("Minor, Social Status, animals only", ArMDE:5806)
   gets D75's gate, the same animal character type no profile has.
2. **`flaw.pagan_minor` is added** as a Minor twin of `flaw.pagan`
   ("Major or Minor, Personality", ArMDE:6571), mutually incompatible with it.
3. **The RULES.md errata note** lists:
   - the four index omissions;
   - the three index disagreements;
   - the stray Folk Magic entry in the Flaw index;
   - for each, both citations and the side taken.

   The same errata, plus the dead index links and the descriptor typos, also go
   to the todo in `arm-de-translation/docs/todo.md`.

---

## D77 — X9c's three open points (`tmp/x9c-plan.md`)

**Norbert, 2026-10-01.**
1. **The extent guard is extended.** A blockquoted `> ####` heading that opens
   another entry's sidebar ends the previous entry's range, so
   `rules_source_provenance.rs` catches the `virtue.perfectus` pattern.
2. **A sidebar placed just before the heading it belongs to stays uncited**
   (e.g. ArMDE:3693). An entry's range is its heading plus its body; the sweep
   records such sidebars.
3. **F-500: `WarpingGrant.score` is dropped.** The engine derives the visible
   Warping Score from the point total (`effective/warping.rs::warping_score`),
   so a stored score is an unread second copy that can only disagree. A test
   pins that Warped by Magic shows Score 1 from its 5 points (ArMDE:7019-7021).

---

## D76 — the Friday review defers the coverage target

**Norbert, 2026-10-01.** Engine coverage is 91.51% against the full-review
skill's 95% floor. The gap of about 890 lines is spread across the crate,
mostly older code. The Friday review accepts 91.5% and converges on its other
findings. Raising coverage to 95% becomes its own slice in the next plan
section.

---

## D75 — F-556 and the covenant gap

**Norbert, 2026-09-30.**
1. **F-556:** add a generic `Prereq::CharacterType(id)`, as D38 foresaw.
   `virtue.domestic_animal` is gated on an animal type that no profile has, so
   humans can never take it.
2. **Covenants:** the UI has no covenant creation path. This is a known gap,
   out of scope before Saturday, and the try-out script drops it.

---

## D74 — D42's open points (`tmp/d42-realm-verdicts.md`)

**Norbert, 2026-09-30.** These extend D70 Q-X6-5/6.
1. **Blood of the Nephilim** (ArMDE:3507) and **Viaticarus** (:6979) are fixed
   Divine. **Sufi** (:5079) defaults to Divine and **Cursed Guile** (:5891) to
   Infernal; changing either warns, like Hex.
2. **Manifest Sin** (:6406): the realm domain may carry a narrowed value list
   (a general loader change). An unanswered subset realm is a warning, never a
   silent fallback.
3. **Bound to (Realm), (Realm) Stigmatic, Necessary (Realm) Aura:** the
   entry's realm defaults to its own named realm, overridable, with no
   warning.
4. **Accepted as recommended:**
   - a fixed realm is shown as read-only text;
   - Tainted entries derive Infernal from `tainted` (:3000);
   - granted copies carry a realm only where stated, and mythic-type grants
     default to the type's realm;
   - the weak doubtfuls stay free;
   - the Magic fallback has no marker.

---

## D73 — X10b/X10c (`docs/vf-audit/design-x10bc-save-format.md`)

**Norbert, 2026-09-30.**
1. A banked XP value at or above the next level's cost is a **warning**, not an
   error.
2. There is **no Casting Total column in the Markdown export** now. X10c covers
   the marker and the in-app totals.
3. **No SCHEMA_VERSION bump.** Both fields are additive with defaults, which is
   the plan's own criterion. This supersedes row 51's "(bump)" tags.

---

## D72 — X9a's anchor questions (`tmp/x9a-spike.md`)

**Norbert, 2026-09-30.**

1. **Table rows** anchor as `<heading>/<row>`, with an optional table key where
   one heading holds several tables (e.g. `aging/aging-roll/10-12`). The row
   key is the first cell, the text before `:`, or the first four words of
   prose, passed through the existing slug rule.
2. **Houses** anchor to each House's own section heading, not to the summary
   table row.
3. **Aging rows get ids.** This is a Rust type change, and the German sidecar
   stays keyed by id.
4. **Accepted as recommended:**
   - prose starts move onto a short section's heading; the four-word key is
     used only at ArMDE:16634;
   - the 27 `-N` anchors;
   - no provenance is added for catalogues without `source`;
   - Piercing the Magical Veil anchors on its Criamon template line;
   - only `anchor` becomes non-optional.

**Not pushed:** worktrees therefore keep branching from the stale
`origin/main`, and each one is fast-forwarded by hand.

---

## D71 — X8d's three held German-name items

**Norbert, 2026-09-30** (details in `docs/vf-audit/measurements.md` § 8 row 14).

1. **All four `SdM:M`-tagged rows are in the dispute.** The three kept rows
   (Homing Instinct, Magical Warder, Unaffected by The Gift) adopt the table
   name, and Deteriorating Power is reverted as D31 says. This applies D31 to
   those rows despite precedence rule 4.
2. **Hobbled stays "Humpelnd".** Norbert asked for the translation repository to
   be checked. Its reviewed core (`german-reviewed/…Basisregeln.md`) uses
   Humpelnd for Hobbled (:6328) and Verkrüppelt for Crippled (:5945). The table
   row "Hobbled → Verkrüppelt" (`grundbegriffe.md:331`) contradicts that current
   text, so it is treated as a table error, and the two Flaws keep distinct
   names. Nothing is filed upstream (D65 N7).
3. **Gender Shift adopts "Geschlechtswandel",** per the upstream table
   (`tugenden-fehler.md:924`) and the reviewed SdM:I translation (:4234). The
   row's note concerns the "Befleckt" label only.

---

## D70 — X6 scoping answers

**Norbert, 2026-09-29** (details in `tmp/x6-scope.md`).

- **Order of work.** The design note, the engine and the 16 rule-driving
  parameters come before Friday 2026-10-02. D42 follows before Friday if
  capacity remains. The 26 label-only parameters and the three retypes come
  after.
- **Q-X6-1.** Commanding Aura gets the four ranks (ArMDE:3585-3591) plus King
  (ArMDE:17651: MR 10, Soak +2). The wife rule (ArMDE:17652) stays text.
- **Q-X6-2.** Turb Trained gets a choice of dead language, a parameter over
  the catalogue's dead languages bound to the authorization (ArMDE:5181).
- **Q-X6-3.** Repellent gets an enumerated choice: natural weapons, scales
  (Soak +3, computed), dark sight, custom (ArMDE:6681).
- **Q-X6-4.** Every new parameter is required, so old saves report
  `missing_param` until it is filled (D10's policy). Retyped text values are not
  migrated. No schema bump.
- **Q-X6-5 (D42).** An entry's realm is resolved when read: the entry's
  override, else the concept's realm, else Magic (ArMDE:2960). Only overrides are
  stored.
- **Q-X6-6 (D42).** Fix the 17 fixed entries and the 1 subset, plus three
  entailed ones: Bee King Faerie (ArMDE:3486), Commanding Aura Divine
  (ArMDE:3581), Raised from the Dead Divine (ArMDE:6648). Spiritual Pact and
  Warped by Magic default to Magic, and Hex to Infernal; changing them warns.

**The X6-0 note's open points** (`docs/vf-audit/design-x6-parameters.md` § 5),
Norbert: two Commanding Aura MR bonuses add up. Ability Block's `custom`
choice stays text permanently. Warped Senses' incompatibilities are a hard
error while its -2 stays text. The orchestrator settles the rest from existing
rulings:
- the relic stub may land before X7b-d's F-256;
- Special Circumstances' +3 stays text, since there is no toggle (D4/D58);
- X6b checks the existing `catalogue.language` values for Turb Trained.

**X2d, Norbert:** True Faith's description (F-330) is composed from its own
entry plus the rules of the "True Faith" section (ArMDE:17603 onward), in
The Gift's `COMPOSED_DESCRIPTIONS` shape (D46).

**X5b, Norbert:** Master Bard's Profession clause is enforced as "some
Profession at 5", the id-level check already used for the Latin minimums.

**Scheduling, orchestrator:** Mercurian Magic's prerequisite goes to X5, so
X7b-d drops its F-196 test.

---

## D69 — X7b-e's three open items (row 42)

**Norbert, 2026-09-29** (details in `tmp/x7be-verdicts.md`).

1. **Raised from the Dead: the creation-time Warping and Reputation are
   computed** from a "years since resurrection" number parameter (D64's
   pattern). The yearly accrual after creation stays text.
2. **Viaticarus: text.** "Social interaction rolls" names no Ability list, and
   none is invented (D61's shape).
3. **Folk Magic's Casting Total: text.** The formula needs an Aura term the app
   does not track (row 49), so no partial total is shown.

4. **X4's entailed twins stay blocked** (D44 entailment): Outsider
   (ArMDE:6554, :6556), True Love the Flaw (ArMDE:6877), Amorphous
   (ArMDE:3412), Magian Lineage (ArMDE:4345). They are marked as entailed.
5. **Blood of the Nephilim's Size clause** (ArMDE:3517, "such as") becomes a
   new "affects Size" tag. Every entry that changes Size is tagged, and the tag
   is excluded.
6. **The same-choice constraint is built now in X4:** Student of (Realm) with
   Puissant Ability for the same Lore (ArMDE:5054), and Academic Concentration
   with Puissant Artes Liberales (ArMDE:3364).

**Scheduling, orchestrator:** Raised from the Dead belongs to X7b-e (D69.1),
and X7b-d drops its F-494 test. X7b-e owns Flawed Powers' "at least one Major
Supernatural Virtue" prerequisite. X4 keeps only its `requires_hermetic_arts`
import filter (D68.4).

---

## D68 — X3/X4/X10 scoping answers

**Norbert, 2026-09-29** (questions in `tmp/x3-scope.md`, `tmp/x10-verdicts.md`).

1. **The 15 ambiguous Hermetic entries are trained.** This applies D12's own
   criterion rather than a new ruling: raw vis, Parma, casting and lab totals
   (Longevity Rituals included) and Magical Focus are all trained objects. So
   D12 collapses to "every Hermetic entry except the three Gift ones requires
   a magus".
2. **Order-audience gates follow the wording.** `order_member` applies where the
   text names the Order, a House or the Gauntlet, and `hermetically_trained`
   applies otherwise.
3. **Training is stated twice:** the `trained` flag plus a `prerequisites`
   gate, with a guard that they agree. No engine change.
4. **Flawed Powers' import filter is a new predicate, `requires_hermetic_arts`.**
   It amends D23/D33. Deficient Technique and Unstructured Caster are in it;
   Restriction and Necessary Condition are not, so they are importable.
5. **Grapple** (ArMDE:18561, Natural Weapons Table) has no mounted twin (D66
   flag).
6. **`spell.piercing_the_magical_veil`** takes InVi 20 from ArMDE:1745, and its
   Range/Duration/Target are copied from Piercing the Faerie Veil, its named
   sibling (ArMDE:15709). This is an inference and is recorded as one in
   RULES.md.
7. **Priest's parish-priest ban on Poor** (ArMDE:4800, :4802) is text only.
8. **Major/Minor twin exclusions need their own passage per pair.** ArMDE:2814
   is not a blanket source for them. A pair without a passage loses its
   exclusion, and the integrity guard changes with it. ArMDE:4405 (Magical
   Focus) keeps its exclusion.
9. **Grog gates use a new `Prereq::IsGrog`**, a profile flag shaped like
   `IsCompanion`, with UI parity.
10. **The guild and university status prerequisites** (ArMDE:4441, :6673) become
   `any[...]` over named, book-defined id lists.
11. **Mythic Blood's hereditary Minor Personality Flaw** (ArMDE:4588) is an open
   grant. It costs no points but counts toward the Personality caps
   (ArMDE:2820).
12. **X3a's borderline six.** Masterpiece (the Gauntlet), Diedne Magic
   (hidden from the Order) and Gorgiastic (left House Criamon, still in the
   Order) gate on `order_member` as well as training. Exotic Casting, Mercurian
   Magic and Mythic Blood gate on training only.
13. **Bound Casting Tools** (ArMDE:5723-5726) gates on training only.
   "As used by House Verditius" explains the mechanic and is not an
   eligibility clause. Consumed Casting Tools keeps its explicit Verditius gate.

---

## D67 — how D46 reads a partly computed entry (X2 scoping, OQ-1/2/7)

**Orchestrator, 2026-09-29, applying D46 and D20. Norbert may override.**
The S3 guard (`data_integrity.rs::every_vf_is_classified`) rejects
`uncomputed_rule` on any entry that computes something. That contradicts D46,
which makes a partly computed `virtue.the_gift` `uncomputed_rule`, and D20,
which makes 19 entries with surfaced effects `uncomputed_rule`.

**Ruling.** An entry with any stated rule computed nowhere is `uncomputed_rule`,
whatever else it computes. The guard checks only what can be checked
mechanically: `narrative` computes nothing; `creation_effect`/`in_play_effect`
compute something; `uncomputed_rule` carries its text (the existing locale
test). A selection constraint the engine enforces (`prerequisites`,
`incompatible_with`, profile traits) counts as computed, following D46's
`hermetic_magus` precedent. So an entry whose only stated rule is such a
constraint is `creation_effect`. Whether a rule is fully computed stays a
per-entry reading, not a guard check.

---

## D66 — mounted combat lines follow the book's template, via an explicit weapon flag

**Norbert, 2026-09-29.** ArMDE:16839 adds Ride (max +3) to "his Attack and
Defense Totals" when mounted, and names no weapon. The Knight's statblock
(ArMDE:1467-1472) prints mounted twins for its two weapon lines only; Fist has
none.

**Ruling: reproduce the template.** An explicit weapon-catalogue field marks the
lines that get no mounted twin (Fist, Kick, Dodge). The engine reads that field,
never `min_strength` or any other unrelated property. This is a deliberate
presentation choice taken from the book's own template, not a rule the passage
states, and `RULES.md` says so.

---

## D65 — the plan's five open decisions (N1, N4–N7)

**Norbert, 2026-09-28.**

| # | Ruling | Slice |
|---|---|---|
| N1 | Markdown export: an `uncomputed_rule` V/F carries `description ?? summary`, and a computed V/F carries its `summary` | X8c |
| N4 | The V/F search indexes `summary` and the whole `description` | X8b |
| N5 | Row 51, all four in this round: (a) Tremere's granted Minor Magical Focus names certamen (ArMDE:2064, :2281), as data; (d) a Grapple equipment row (ArMDE:1774) and `spell.piercing_the_magical_veil` from the source's own figures; (b) banked XP beside Art and Ability scores (bump); (c) a per-spell "within the focus" marker (bump). If the source does not give a value, report it, never invent it | X10, X10b, X10c |
| N6 | `virtue.aristotelian_training` (ArMDE:3442): `uncomputed_rule` with the full rule as text, since every clause is a condition decided at the table (as D61) | X7a |
| N7 | Nothing is filed or edited in `arm-de-translation`. Upstream fixes are recorded here, and Norbert carries them over | X8d |

---

## D64 — an Abandoned Apprentice's years after abandonment may fund Arts

**Norbert, 2026-09-28.** ArMDE:5647 gives the years after abandonment
"experience points based on his age and other Virtues" and does not say where
they may go. The character "knows Hermetic magic" (ArMDE:5643), with his Arts
already opened.

**Ruling.** The ordinary later-life years **after** the truncated apprenticeship
may fund Arts as well as Abilities. The years **before** it stay Abilities-only,
since the Arts are not opened yet. The two spans are separate pools. The
apprenticeship starts at the character's Gauntlet age (default 25, ArMDE:1601)
minus `apprenticeship.years`. The age check requires age ≥ start +
`years_completed`.

---

## D63 — Feral Upbringing restricts its own first five years only (amends D60.2)

**Norbert, 2026-09-28:** *"Feral replaces first 5 years. Church comes later,
from the general pool"*. For example, a child who spends five years in the
wild, is then adopted and raised by the Church, and can still become a master
craftsman or a magus.

**Ruling.** ArMDE:6112's wilderness list and its "no Language" rule bind only
Feral's **120-XP pool for the first five years**, which replaces early
childhood (D40, plan slice D2). They do **not** bind later-life XP. D60.2's
creation-wide whitelist reading is withdrawn. Church Upbringing's earmark
(ArMDE:5791) is therefore spendable on a Feral character, from later-life
general XP.

**Consequences.**
- D2 models Feral as a restricted replacement pool only, with no
  `RestrictsAbilityCategoryToAbilities` effect.
- B1's `RestrictsAbilityCategoryToAbilities` has no carrier left, since no
  shipped data ever used it.
- D1 caps an earmark at the available general XP, so the total never rises.

---

## D62 — free seasons are not modelled; D49 becomes text (supersedes D49)

**Norbert, 2026-09-28**: "If none creation-relevant, it's not necessary to model."
Checked. **Non-magi:** later-life XP is a flat rate, *"15 experience points per
year … Wealthy … 20 … Poor … 10"* (ArMDE:2214, :2394), already data
(`later_life_xp_rate`). Seasons never enter it. **Magi:** post-Gauntlet lab
seasons are already capped at 3 per year (`life_stages.json`
`max_charged_lab_seasons_per_year`). The only magus-eligible season cost,
Regular (ArMDE:6677, "Magi can be Regular"), leaves 4 − 1 = 3, which does not
move that cap. The season rules of Landed Noble, License of Absence, Lone
Redcap, Redcap, Wealthy, Poor and Regular stay in `description`. **Plan slice
D4 is dropped.**

**D3 input, same day:** apprenticeship defaults to ages 10–25 (Gauntlet 25,
ArMDE:1601). An Abandoned Apprentice records the **years of apprenticeship
completed**, not an age, and the years after abandonment follow the later-life
rules.

---

## D61 — "rolls involving <a sense>" is a table call, so it stays text

**Norbert, 2026-09-28**, after B5 encoded Poor Hearing (ArMDE:6616, "Subtract 3
from rolls involving hearing") as -3 to Awareness rolls. That is wrong both ways:
it hits Awareness rolls by sight or smell, and misses hearing rolls that use no
Awareness (a Perception roll, following speech).

**Ruling.** A modifier conditioned on a sense, like D15's "selfish or sinful",
is a table judgement. It stays in `description` in both locales, with no
computed effect. That covers Poor Hearing, Sharp Ears (ArMDE:4952), Keen Vision
(ArMDE:4189), Susceptibility to Sunlight (ArMDE:6829, sight in bright light),
and the non-combat half of Poor Eyesight (ArMDE:6608). Poor Eyesight's attack
and defence -3 is stated outright, so it stays computed. A sheet list of
situational modifiers was offered and **not** chosen.

---

## D60 — True Love's reciprocity is text; Feral Upbringing's whitelist is creation-only

**Norbert, 2026-09-27**, on B0's open questions (`design-b0-ranging-and-predicates.md` § 9).

1. **F-334 (`virtue.true_love_pc`) stays text only.** ArMDE:5175's reciprocity
   constrains a second character, which a one-entity save cannot see. The
   reciprocity rule and ArMDE:5177's "True Friend" rename go into `description`
   in both locales (X2). B3 builds no `Prereq` for it.
2. **Feral Upbringing's Ability whitelist applies to beginning Abilities only**
   (ArMDE:6113), as Sheltered Upbringing leaves in-play learning open. If two
   such restrictions ever stack, their lists **intersect**.
3. **True Friend is a first-class entry** (amended the same day). The book
   allows the rename for both True Love (PC) (ArMDE:5177) and the True Love
   Story Flaw (ArMDE:6875), and names "the Minor Virtue True Friend" in its own
   right (ArMDE:10852). So `virtue.true_friend_pc`, `flaw.true_friend_major` and
   `flaw.true_friend_minor` copy their twins' mechanics and source lines, with
   the name *Wahrer Freund* (`tugenden-fehler.md`). A test pins twin parity.
   The book states no incompatibility with True Love, so none is added; the
   major/minor exclusion is mirrored. Plan slice **X2t**.

---

## D59 — a rule-recognised Ability parameter is a catalogue id, a Virtue link, or free text

**Norbert, 2026-09-27**, after C1/C4's literal instances (`language = latin`,
`profession = marshal`) turned out to match **free text**, so "Latin" and
"Latein" both silently failed.

**Ruling.**

1. Where a rule names one specific value (Latin, Profession: Marshal, Order of
   Hermes Lore), the Ability parameter draws from a **catalogue of ids** with
   localized names, with free text as the fallback. Old saves migrate by EN/DE
   name (case-insensitive), in their own schema bump.
2. Where a rule names **the character's own** organization or craft
   (Craft Guild Training's "Organization Lore: Guild", ArMDE:3615; Educated
   (Vernacular)'s "the character's company", ArMDE:3729; Forge Companion's
   "Crafts her master practices", ArMDE:3927), it is **not** a catalogue value.
   The Virtue records it as a parameter, and the Ability **links** to that
   parameter. **Link, not copy**: a rename on the Virtue follows through.
3. A link may target only a **once-only** item (selections have no stable id).
   It resolves against effective selections. When its Virtue is removed it
   becomes text, keeping the last value; it is never saved dangling.
4. The Ability parameter field is a **combo box whose options the engine
   builds**: catalogue values, linkable Virtue parameters, and free text.

**Spec:** `design-cv-catalogued-values.md`; plan slice **CV**.

---

## D58 — the goal is the scope test: build every character legal under the core rules

**Norbert, 2026-09-24:** *"Goal is an application which can build all characters
legal under core rules. … If mechanics are missing, the mechanics need to be
implemented."*

**This replaces "out of scope" as a category.** § 4a listed K3, K5 and W2 as
things the V/F round does not cover, which was true of the *pass* and was then
read as true of the *project*. It is not. A missing mechanic is work, not a
boundary.

### The test

**Does its absence prevent building — or misreport — a character legal under the
core rules?** If yes, it is in scope and the mechanic is implemented.

| Item | Verdict |
|---|---|
| **K5** `equipped` overloaded | the Knight loses two Combat rows → **in** |
| **W2** authorization cannot restrict | the app permits an illegal Wise One → **in** |
| **Q-51** parameter-gated effects | Magic Human's Reputation 3 reaches nobody → **in** |
| **Q-87**, **Q-11**, **D56** | block or misreport legal characters → **in** |
| **Q-56** Purity / Transcendence | *Realms of Power: The Divine*, not core → **out** |

**D22's core-book-only boundary is the goal stated precisely**, not a limitation
to apologise for. "Legal under **core rules**" excludes supplement content by
construction.

### The two answers the test needed

**1. In-play modifiers the sheet carries ARE in scope — if the sheet shows it, we
compute it.** The character sheet is the product, so a figure a player reads off
it must be right. K3's mounted combat, `flaw.enfeebled`'s doubled casting fatigue
and the surfaced-modifier family are all work, not text. **This does not reopen
Q-98 or D45**: a doubling still cannot be computed where the engine does not know
the base (`casting_fatigue` is surfaced, not computed), and D50 still governs
what reaches the player as text. It settles that *"it only matters in play"* is
**not** a reason to decline a computation the sheet displays.

**2. Animal characters are a deliberate non-goal.** Humans and covenants only.
`virtue.domestic_animal` is core-book and therefore describes a legal character,
so this is an **exception to the test, taken deliberately** — like **Q-05**'s
refusal of a sex model. **Consequence: F-556's gate is the permanent answer, not
a stopgap** — a human may not take the entry, the entry ships with its text
(D50), and **no `Cunning` characteristic and no animal profile are added.**

### What this obliges

- **Rewrite § 4a.** It must say *which pass* does the work, never *whether the
  project does it*. The Knight cluster stays as the worked example of the
  filing failure; the "not ours" framing goes.
- Anything currently resting on *"the engine cannot express it"* is re-read
  against the test. **D3 survives** — an engine that structurally cannot express
  a rule still yields `uncomputed_rule` **plus text** — but "cannot" now means
  *cannot even in principle*, not *does not today*.
- Record the two non-goals together where they can be found: **supplements**
  (D22) and **animals** (here), alongside Q-05's **sex model**.

---

## D57 — a free-text placeholder stands in apposition, because the app cannot know its gender

**Question (Q-77).** `virtue.voice_of_the_land`'s German `name` is
`Stimme des {land}` and `virtue.ways_of_the_land`'s is `Wege des {land}`. That is
right for masculine and neuter terrain nouns (*Stimme des Waldes*) and **wrong
for feminine ones** (*Stimme des Steppe*, for *der Steppe*).

**Ruling: apposition after a comma, uninflected** — the pattern `RULES.md:2356-2368`
already adopted for the four `realm` items and, before that, for Folk Magic.

**Why it generalises rather than being a second convention.** The realm fix was
made because *"every German template put the token somewhere that demands
agreement"*, producing *"Student der Das Göttliche"*. The free-text case is the
same failure with a worse cause: a **picked** label at least has a knowable
gender, while a **typed** one does not, so no fixed article can be right for
every input. B09 read *"a typed word inflects however the player typed it"* as a
reason free text differs; it is a reason to keep the word **out of** a slot that
demands agreement, which is what apposition does.

**The table is not overridden here, despite D31.** `tugenden-fehler.md:148` gives
`Stimme des/der (Land)`. That slash is an **editorial flag for a problem**, not a
string to ship — a user would read *"des/der"* in the middle of their own
character's Virtue. D31 makes the table authoritative on a **term**; it does not
turn a notation into a label.

**What this obliges.**

- Rewrite both German `name` templates into the apposition form, and check the
  rest of `rules/i18n/de/` for the same shape — **`RULES.md`'s own note says ~36
  mid-phrase templates exist**, and only the realm four have been fixed.
- **`name_unfilled` stays as it is on both.** `derive.ts::displayName` keeps the
  *"(Land)"* hint deliberately for mid-phrase templates; that is correct and is
  not part of this.
- The DE rulebook headings at 5219/5231 dodge the problem with a neuter
  placeholder (*Stimme des (Landes)*), so they are **not** evidence for the
  `des` form with a free-text value.

---

## D56 — `is_magus` splits: Hermetic training and Order membership are different facts

**Question (Q-84).** What entity or type holds `flaw.abandoned_apprentice`
(*Major, Story*, ArMDE:5641-5650)? *"He knows Hermetic magic and can cast spells
and enchant items like other magi. He is **not a member of the Order of Hermes**,
however."*

**The bind.** `is_magus` is one flag doing five jobs, and he needs four of them
and must not have the fifth:

| `is_magus` gates | He needs it? |
|---|---|
| the `arts` phase — the only profile that has one | **yes**, he casts spells |
| Arcane Abilities with no further Virtue (`validation/authorization.rs`) | **yes** |
| the apprenticeship XP shape (`effective/xp.rs`) | **partly** — see below |
| **D12**'s gate on *trained* Hermetic V/F | **yes** — Deficient Technique and its kin are his |
| **Houses** (`validation/magus.rs`) | **no** — he is in no Order |

**Ruling (Norbert, 2026-09-24): split the flag.** `is_magus` becomes *Hermetically
trained* and *member of the Order*. The first four rows key on **trained**;
Houses key on **Order**.

**He is a companion, and nothing about the budget changes.** Not a Mythic
Companion — the 2:1 V/F rule does not apply to him. He is an ordinary companion
at 1:1 who has taken a **Major Story Flaw worth 3 points**, consuming the Story
slot.

**The Flaw confers *training*, not the Gift.** The Gift is **selected**:
`gift_policy: "allowed"` on the companion profile, and `virtue.the_gift` is
`magnitude: "free"`, so it costs nothing. Under **D51** the passage presupposes
the Gift (*"He **knows** Hermetic magic"*) rather than giving it, so the encoding
is `Prereq::Has(virtue.the_gift)`. **Granting it would be a real defect**, not a
mislabel: `effective::has_the_gift` reads the Gift to decide whether a companion
may touch the `hermetic` category at all (D24), so a Flaw conferring it would
bootstrap its own permission — the circularity `RULES.md` warns about for
`gift_categories`.

**This also repairs D12's gate.** D12 made *trained* Hermetic entries require
`Prereq::IsMagus`. Under the split that becomes **requires Hermetic training**,
and an Abandoned Apprentice passes it — correctly — where `IsMagus` would have
refused him.

**Not post-gauntlet: he never had a Gauntlet.** ArMDE:5647 gives the
construction rule outright — *"**Decide at what age the character was abandoned.
Create the character as a regular apprentice up until that age**, and then give
him experience points based on his age and other Virtues for his life past being
abandoned."* So the shape is **truncated apprenticeship + later life**, and the
age is a **numeric parameter** (D35's type; a D9 instance).

**SETTLED (Norbert, 2026-09-24): a partial apprenticeship is priced by the
year, at the rate the lump implies.** ArMDE:2435 gives *"The fifteen years of
apprenticeship give the character **240 experience points, and 120 levels of
spells**"*, so:

| Per year | Value |
|---|---|
| Experience | **16** |
| Spell levels | **8** |

**Both** quantities are divided, not only the XP — the passage pairs them, and
ArMDE:5643 has him casting spells, so granting the first without the second would
leave a trained apprentice with none.

**FIXED DECISION — Norbert, 2026-09-24: *"16/8 is computed from the numbers.
Mark that as fixed decision until I say otherwise."*** The two figures are
**derived from the book's own**, ArMDE:2435's **240** and **120** over fifteen
years. They are not a house rule and are **not to be reopened** as one. The book
prices apprenticeship as a lump, which is why guided-creation issue **#15** was
marked *BLOCKED ON SOURCE* — **and this unblocks it**, since the division is
determinate rather than a judgement call.

**Spendable on what.** ArMDE:2435's own set: *"Arts or Abilities, including
Arcane, Academic, and Martial Abilities."*

**Parma Magica is allowed, and warned.** ArMDE:5648 does **not** forbid it —
*"If the character knows the Parma Magica, he must **join the Order or be
slain**"* — and ArMDE:5650 continues *"**Even if the character joins the
Order**, his background continues to cause problems"*, so a Parma-knowing
Abandoned Apprentice who joins is a character the book contemplates. Excluding
Parma from the spendable set was considered and rejected for making that concept
unbuildable; holding it raises an advisory instead (D16's shape for a stated
consequence).

**The truncated block must NOT enforce `apprenticeship.minimum_abilities`** —
Latin 1, Magic Theory 1 and Parma Magica 1 describe a *completed* apprenticeship.
A character abandoned at 14 cannot owe them.

**UI: conditional phases, using the mechanism that already exists.**
`creation_phases` is per-profile data, so a phase cannot depend on a selection
today. `permitted_categories` already solved exactly this with
`CategoryRule` — `{ category, when: <Prereq> }`, resolved once through
`categories_in_force` against the validator's own `PrereqCtx`, `Tri::Unknown`
treated as not-in-force. A phase rule takes the same shape, `{ phase, when }`, so
the Arts and spells tabs appear for anyone Hermetically trained, whether by being
a magus or by this Flaw. **One mechanism for "applies conditionally"**, which is
what D21 and D24 both argued for.

**No House-screen entry.** Houses belong to magi in the Order; he has none and
has nothing to choose there. The Flaw in the V/F phase is the whole of it.

**What this obliges.**

1. **Re-read every `is_magus` site for which half it meant** — ~33 in
   `validation/mod.rs`, ~19 in `derived.rs`, plus `types.rs`, `validation/magus.rs`,
   `prereq.rs`, `ruleset.rs`, `effective/xp.rs`, `integrity.rs`, `life_stage.rs`,
   `authorization.rs`. **This is the cost of the decision and it will be
   underestimated.** Added to § 8.
2. **`flaw.abandoned_apprentice` gains `Prereq::Has(virtue.the_gift)` — and that
   is a defect on today's data, independent of the split.** The reachable case is
   an **ungifted companion**: that profile permits `story` at `max: 1`, sets
   `max_major_flaws: null`, and has `gift_policy: "allowed"` — *allowed*, not
   required — so a companion who simply never takes The Gift collects **3 Flaw
   points** for Hermetic training he cannot have. **Not a grog**, corrected on
   Norbert's challenge: the grog profile blocks it four ways over — `story` is
   absent from `permitted_categories`, `flaw_category_caps` sets
   `{"category": "story", "max": 0}`, `max_major_flaws` is `0`, and
   `virtue.the_gift` is in `forbidden_traits` with `gift_policy: "forbidden"`.
   *Amended 2026-10-03 (Norbert, F10):* `story` is now permitted for grogs and the story cap is warning-only, per "should not" (ArMDE:1009, :2826); `max_major_flaws: 0` still blocks this Major Flaw.
3. The age parameter, the conditional-phase rule, and D12's gate reworded from
   `IsMagus` to *trained*.
4. *"If the character knows the Parma Magica, he must join the Order or be
   slain"* (ArMDE:5648) is a further rule this entry states and nothing carries —
   text at minimum (D50), and expressible as a warning on
   `Has(ability.parma_magica)`.

---

## D55 — a halving gets a factor; `amount: 0` must stop meaning two things

**Question (Q-113).** `RULES.md:5159` records `advancement_mod` `amount: 0` on
`flaw.incomprehensible` and `flaw.loose_magic` as a deliberate **"halve
advancement"** marker — the only two of fifteen `advancement_mod` rows carrying
it. `DerivedSurfacedModifiersSection.svelte:27` guards the value with
`{#if m.amount !== 0}`, which is right where 0 means *"no magnitude carried"*
(A37's four surfaced kinds, A40's eight) and **wrong here**, where it means
*halved*. The player sees a labelled row with nothing beside it, indistinguishable
from a modifier of no size.

**Ruling: give `advancement_mod` a factor, and stop overloading `amount: 0`.**

**Why not text, when Q-98 chose text for the same shape.** Q-98's doubling sits
on `casting_fatigue`, which **D45** established is *surfaced and never computed* —
the engine does not know what a spell costs, so it cannot double it. Advancement
is different: `advancement_mod` **is** computed and carries real values (−3 on
`flaw.poor_student`). A halving there can be computed, so gesturing at it with a
sentinel gives up a number the engine could get right.

**The overloading is the defect, more than the rendering is.** One field means
both *"no magnitude"* and *"halved"*, and nothing in the data distinguishes them —
a reader of `rules/core/` cannot tell which is meant, and the UI guard silently
picks the wrong reading for two entries.

**What this obliges.**

- A factor (or equivalent) on `advancement_mod`, and **`amount: 0` retired as a
  marker** on the two carriers — leaving 0 with exactly one meaning.
- **Check the halving's base before implementing**: what is halved, and whether
  it interacts with **D49**'s creation-time advancement work, which touches the
  same budget.
- **`flaw.incomprehensible` halves along *two* axes, not one** (found settling
  **Q-32**). ArMDE:6296: *"Anyone trying to learn **from you or from a book you
  have written** must halve their Advancement Total (or Lab Total, if you are a
  magus and have written Lab Texts on some spells or enchanted items)."* So it
  needs a `Teaching` row **and** an authoring row — the new `AdvancementSource`
  Q-32 adds — each carrying the factor. Encoding one of them and calling the
  entry done would drop half a stated rule.
- The UI then renders a real number and D45's `source` names the entry producing
  it; no separate rendering ruling is needed.
- Two carriers, measured — *"the only two of fifteen"* is B14's count and should
  be re-derived, per § 8's standing rule.

---

## D54 — a supplement-conditional category follows the app's *current* capability, and the flip is registered

**Question (Q-111).** ArMDE:6304 ends: *"If you are **not using the rules in City
and Guild** (page 73), treat this as a **Personality** Flaw."* The descriptor
(ArMDE:6303) and the book's index (ArMDE:5592) both say *General*, and
`flaw.independent_craftsman` ships `["general"]`. The category is not cosmetic:
`personality` is capped at 2 per profile (1 if Major), `general` is uncapped.

**Ruling: ship `personality` now, and register the flip.**

**Norbert's framing, which decides it.** A later version will let the troupe
select which supplements are supported in character generation; this Flaw then
switches — `general` when City and Guild is in play, `personality` when it is
not. Until that exists, **the app supports no supplements, so the book's
condition is satisfied today** and `general` is the value the book says is wrong
for our situation.

**Why not the alternatives.** Keeping `general` ships a value ArMDE:6304
contradicts for an app in this state. A `ParameterDomain::Category` (`taken_as`)
parameter would route a **saga-level** setting through an **entry-level** player
choice, and `RULES.md`'s 2026-09-13 ruling is explicit that `taken_as` models
*"**or**, and only *or*"* — a choice between two readings, which a conditional is
not.

**What this obliges.**

- `flaw.independent_craftsman` → `categories: ["personality"]`. One word. It
  **narrows** what is buildable, so saves legal today report and block per
  **D10**.
- **A standing register of supplement-conditional entries** — § 8a — so the
  future feature flips them by reading a list rather than by someone
  remembering. **This entry is invisible from that feature**, which is exactly
  the § 8 problem.
- **Measured: one instance.** A sweep for *"treat this as a"* over the core book
  returns two hits, and the second (ArMDE:18519) is a creature rule, not a
  recategorization. So no mechanism is warranted — a one-word data change plus
  the register is the whole of it.

---

## D53 — the glossary governs inside a verbatim quotation too

**Question (Q-54).** `rules/source/de/translation-tables/alterung-twilight.md:21`
writes *Living Conditions modifier* closed — **Lebensumständemodifikator**. Our
DE rulebook writes it hyphenated at ArMDE:4530 (*"einem +1-Bonus auf den
**Lebensumstände-Modifikator**"*), and the German `summary` quotes that sentence
verbatim. B06 let the rulebook govern *because* the summary is a quotation, and
flagged that CLAUDE.md's rule is unqualified.

**Ruling: the glossary governs, quotation included.**

**Why, and it turns on D31.** "Verbatim" here means faithful to
`rules/source/de/`, which **D31 established is an older copy than the tables**.
Fidelity to a stale source is fidelity to the wrong thing. The same reasoning
that made the table win on a *name* makes it win inside a quoted sentence.

**Scope, stated because it is large.** This decides the spelling in **every**
entry that quotes a glossary term in `summary` or `description` — which, after
D5's corrections and D50's reclassification wave, is most of the catalogue.

**What it does not license.** This is **not** permission to rewrite quoted rules
text generally. **D39** stands: a truncated English sentence is reconstructed
from the English and never back-translated, and the `scheitest` precedent keeps a
faithful copy of a real typo. D53 governs **terminology inside a quotation**, not
its wording, its facts or its structure — substitute the glossary's spelling of a
glossary term, change nothing else.

**What this obliges.**

- `virtue.mild_aging` is **not** clean on check 12 after all; B06 rated it clean
  on the rulebook-governs reading, which this reverses.
- The sweep belongs with **D36**'s, since both ask the same thing of German prose
  — added to § 8's row for the German text pass rather than as a separate one.
- **The German rulebook is not edited to match.** D18 forbids re-syncing it now,
  and the disagreement is evidence of the copy's age, not a defect in it.

---

## D52 — D4's Cyclic Magic row is amended: the Virtue and the Flaw are not symmetric

**Question (Q-91).** Does ArMDE:5895's third sentence change **D4**'s answer for
Cyclic Magic? D4's table carries one row for both entries —
*"`virtue.cyclic_magic_positive` / `flaw.cyclic_magic_negative` | season-dependent
| no; creation fixes no season"*.

**Ruling: yes. The row is wrong for the Flaw, and the book is deliberately
asymmetric.**

| Entry | Its Lab Total condition |
|---|---|
| Virtue, ArMDE:3637 | *"The bonus also applies to Lab Totals **if the positive part of the cycle covers the whole season**"* |
| Flaw, ArMDE:5895 | *"The penalty applies to Lab Totals **even if the negative period does not cover the whole of the season**"* |

Both passages also fix the two halves as **equal in length** (ArMDE:3637, :5896).
So for any cycle shorter than a season — *"solar, lunar"*, the book's own
examples — **every** season contains negative time: the penalty always applies,
while the bonus can never apply at all. Only a **seasonal** cycle makes the two
behave alike.

**What this obliges.**

- **D4's row splits.** The Virtue's *"no"* stands. The Flaw's answer is *"yes,
  unless the cycle is seasonal"*.
- **The cycle type is an unrecorded choice** — the book offers *"solar, lunar, or
  seasonal, for example"* and neither entry carries a parameter. That is a **D9**
  instance, and it is what the Flaw's answer depends on, so D9 must land before
  the Lab Total can be decided per character.
- **Filed as F-555.** F-45 rates the *Virtue's* scope and F-398 the *Flaw's*
  missing text; **no finding covered the asymmetry**, which is how one D4 row came
  to stand for two opposite rules.
- D4 is otherwise untouched — its other eight rows were each read from their own
  passage and none is a pair.

**Lesson.** D4 grouped two entries by their *name* rather than by their passages.
The two sentences differ by one word — *"if"* against *"even if"* — and that word
reverses the rule.

---

## D51 — prerequisite, grant or effects-only: the book says which by saying who gives it

**Question (Q-37, Q-48, Q-94).** When an entry "comes with" another item, is that
a `Prereq::Has` (the player takes it and its points count), a `grants_selection`
(budget-exempt), or neither? The two encodings differ by one Flaw point, and B06
would not pick between them.

**Ruling: read who the book says does the giving.**

| Wording | Encoding |
|---|---|
| *"the character **also has** …"*, *"can only be bought **if** the character also has …"* | **prerequisite** — `Prereq::Has`; the player takes it and its points count normally |
| *"**granting** …"*, *"provides …"* | **grant** — `grants_selection`, budget-exempt |
| *"… **at no extra cost**"* | confirms a grant; **its absence does not negate one** |
| *"includes the **effects of** …"* | **effects only** — the item is never held |

**The four worked instances.**

- ArMDE:4522 `virtue.mercurian_magic` — *"All known members of the Mercurian
  lineage **also have** the Minor Flaw Ceremonial Spontaneous Magic"*. A fact
  about the character, not something the Virtue hands over → **prerequisite**,
  and the player receives the Minor Flaw's point normally. This is F-196's
  proposal, now grounded.
- ArMDE:4588 `virtue.mythic_blood` — *"(both **at no extra cost**)"* → **grant**.
- ArMDE:4251 `virtue.leper_magus` — **both in one sentence**: *"can only be bought
  if the character **also has** the Leprosy Flaw"* → prerequisite, and
  *"**granting** the Life Boost Minor Virtue"* → grant. The entry is the proof
  that the distinction is the book's and not ours.
- ArMDE:5907 `flaw.a_deal_with_the_devil` — *"includes the **effects of** Plagued
  By Supernatural Entity"* → **effects only**.

**Q-94's second half falls out.** It asked whether a granted Story Flaw counts
toward the Story cap. Under the third row it never arises: the item is **not
held**, so there is nothing to count. The effects are copied; the Flaw is not.

**What this obliges.**

- Encode the four above accordingly, and **do not read "at no extra cost" as
  required** — Leper Magus would otherwise charge a point the book never asks for.
- **Effects-only needs checking against the engine**: copying an effect without
  holding the item may have no carrier today, and if it does not, that is a
  finding rather than a reason to reach for `grants_selection`.
- **Q-58 and Q-107 are not settled here.** Q-58 asks about *powers* rather than
  items, and Q-107 about an **open** grant; both may follow, but neither was read
  for this ruling and neither should be assumed.

---

## D50 — "mechanical" is the wrong test; the test is whether a player would get it wrong

**Question (the Q-83 family: Q-09, Q-60, Q-66, Q-83, Q-105, Q-118, Q-119).**
Seven open rows all ask the same thing in different clothes — is a hedged number
mechanical? a guaranteed storyguide intervention? an absolute about a resource
the engine does not model? a fiction condition?

**Ruling: drop "mechanical" as the boundary.** A clause belongs in
`uncomputed_rule`, owing its text in both locales, **iff the book states
something a player or storyguide must act on — something a reader would get wrong
by not knowing it.** `narrative` means **pure colour**: prose that states nothing
anyone must act on.

**Why the old framing failed.** It made the test turn on the *form* of the clause
(number? roll? cap?) when what matters is its *consequence*. The two extremes of
the family show it:

- ArMDE:4730, `virtue.personal_vis_source` — *"the yield **should be about one
  tenth** as much as the player covenant expects to gather per year"*. Hedged,
  and still a quantity the troupe must set.
- ArMDE:3599, `virtue.common_sense` — *"common sense (the storyguide) **alerts
  you** to the error"*. No number, no roll, and yet a storyguide who does not
  know it will not do it.

Under the old test the first is arguably in and the second is out. Under D20 —
*a rule must reach the player as a number or as text* — **both are in**, and so
are the other five.

**All seven resolve the same way:** `uncomputed_rule` + text. Q-60's hedged
tenth, Q-09's guaranteed intervention, Q-66's *"may overwhelm you"*, Q-83's
hedged monetary value, Q-105's *"1 pawn of Muto vis"*, Q-118's *"you cannot own
vis"* and Q-119's *"you cannot cast spells at all"*.

**This does not make hedged wording enforceable.** **D16** stands: hedged wording
is a warning and never an error. D50 governs *whether the rule is carried at
all*, not how strictly it is checked. A hedged rule is carried as text and
enforced as nothing.

**What this obliges.**

1. **The seven entries** reclassify and gain descriptions in both locales.
2. **The 335 `narrative` entries must be re-tested against this line.** That is
   the real cost and it is not optional — 102 are `uncomputed_rule` today, and
   D20's nineteen showed that a wrong `narrative` is how a stated rule reaches
   nobody. Added to § 8.
3. **The phrase screen cannot find these.** `MECHANICAL_PHRASES` matches signed
   numbers, botch terms and set phrases; *"the storyguide alerts you"* contains
   none. Per § 2's ordering constraint the screen must grow **before**
   reclassification lands, and D50 widens what it has to catch — or the re-test
   is a manual read, as the Virtues sweep already had to be.

---

## D49 — free seasons per year are a creation-time constraint, and they bind the plan

**Question (Q-44).** No `Effect` variant touches a per-year season count, though
three entries change one: `virtue.landed_noble` (ArMDE:4223, *"You must spend
every season managing it"*), `virtue.license_of_absence` (ArMDE:4293, *"an extra
free season each year … never more than four free seasons in a year"*) and
`virtue.lone_redcap` (ArMDE:4323, *"You must still devote two seasons each year
carrying messages"*, with explicit Wealthy/Poor interactions). B05 called it a
scope decision and recommended text.

**Norbert's correction: post-Gauntlet XP is not only an in-play quantity, and for
guided creation this matters.** Checked, and it holds:

- `life_stage.rs::LifeStagePlan::post_gauntlet_lab_seasons` is a **save-persisted
  character-creation plan field**, not an in-play tally.
- `life_stage.rs::post_gauntlet_points` subtracts `lab_season_cost` per charged
  season from the creation budget, and ArMDE:2471 makes each point *"an
  experience point in an Art or Ability or one level of spell"*.

So a Virtue that consumes every season, or frees an extra one, changes **what the
player may claim at creation** — and therefore his XP and spell levels.

**Ruling: derive a free-seasons-per-year value from the V/F and use it to
validate the plan.** Not a new computed axis on the sheet: the player already
enters `post_gauntlet_lab_seasons`, and these entries constrain that entry.

**What this obliges.**

- An effect expressing a per-year season delta, and a derived per-year
  free-season count with ArMDE:4293's **hard cap of four**.
- Validation against `post_gauntlet_lab_seasons`. Mind the field's own
  subtlety, documented on it: it counts **charged** seasons, and the fourth
  season of a year is free (ArMDE:2482 has reached 0 by the third) — so a cap on
  *available* seasons is not the same quantity and must not be compared naively.
- **F-131, F-146 and F-151 change remedy**: each resolved to "write it as text",
  which is now the *lesser* half. Text still owed (D20); the constraint is owed
  too.
- `virtue.lone_redcap`'s Wealthy/Poor interaction must be read with **D17**,
  which already rules on that entry's 300 apprenticeship points.
- **Sweep beyond B05's 35-entry window** before authoring — three in one span is
  a floor. Added to § 8.

---

## D48 — a restricted pool may name Ability *instances*, and eligibility becomes a union

**Question (Q-47).** `effective/xp.rs` hard-codes `instances: Vec::new()` for
every V/F grant, commented *"A V/F grant is id/category-scoped, never
instance-scoped."* So a pool naming `ability.profession` funds **every**
Profession. ArMDE:4453 ties `virtue.marshal`'s 50 points to **Profession:
Marshal**; today they buy Profession: Sailor.

**Ruling: add `instances` to a V/F grant, and make eligibility the *union* of
`abilities`, `categories` and `instances`** — replacing the current *"when
non-empty it is the ONLY test"*.

> **Correction (2026-09-26, C0 plan review):** the pool below is **240** points,
> not 50 — both the shipped entry and ArMDE:4461 say *"an extra 240 experience
> points"* — and its list also includes Faerie Lore and Magic Lore. The union
> argument is unaffected; the figure was wrong. Do not copy "50" into data.

**Why the union, and it is not a preference.** `virtue.master_bard` (ArMDE:4461)
has **one** 50-point pool funding *"Art of Memory, Profession: Storyteller,
Profession: Poet, any Area Lore, or any Organization Lore"* — two named instances
**and** two whole categories. Under instances-wins that entry cannot be expressed
at all, and splitting it into two pools would invent a division of the 50 the
book does not give.

**The change is verified behaviour-preserving.** The only existing `instances`
user is the childhood native-language pool, which builds
`abilities: Vec::new(), categories: Vec::new(), instances: vec![native]` — and a
union over two empty lists plus `instances` is exactly `instances`. Nothing else
reads the field.

**What this obliges.**

- `instances` on the V/F grant path, the union semantics, and the doc comments
  that currently assert the opposite as design.
- **Four entries are known, not three.** B06's span has `virtue.marshal`,
  `virtue.master_of_kennels` (each one named Profession) and `virtue.master_bard`
  (two, plus the Lores — whose unscoped funding is **correct** today). **B02's
  F-74 is a fourth**, outside that span: `virtue.forge_companion`'s 50 points are
  scoped by ArMDE:3927 to *"the particular Crafts **her master** practices"* and
  the data funds `ability.craft` unscoped, rated **H**. So "three" was never the
  count — **sweep the catalogue before authoring**, starting from these four.
- Instance scoping is **narrowing**, so a save that spent a Marshal's points on
  Profession: Sailor becomes invalid — report and block per **D10**, no
  migration.

---

## D47 — Guild Apprentice's nullification is computable, because the "stage" is a sibling Virtue

**Question (Q-26).** ArMDE:4043, `virtue.guild_apprentice`: *"The character is
**not able to benefit** from either the Poor Flaw or the Wealthy Virtue … **until
he moves to the journeyman stage**."* Both carry `later_life_xp_rate` (20 and
10), so the app computes a rate the book denies.

**Ruling: encode the suppression**, keyed on holding `virtue.guild_apprentice`,
and state the rule in `description` as well.

**This decision was first recorded the other way, and the first version was
wrong.** It ruled *text only*, reasoning that the nullification is bounded by a
life stage the engine does not model, so any encoding would apply permanently and
deny Wealthy's benefit for life. **Norbert asked whether a Journeyman Virtue
exists. It does** — `virtue.journeyman`, ArMDE:4159, *Minor, Social Status*,
alongside `virtue.guild_master`, `flaw.failed_journeyman` and
`flaw.failed_master`. The guild stages **are** modelled, as sibling Social Status
Virtues.

**So the condition is trivial.** **D41** establishes that a character holds
**one** Social Status, so *"until he moves to the journeyman stage"* means
exactly *"while he holds Guild Apprentice"* — moving on means holding a different
Social Status Virtue, which he cannot hold simultaneously. Nothing is permanent
and nothing is unmodelled.

**Reachability, checked.** Wealthy and Poor are Major and companion-only (D38),
so a **companion** with Guild Apprentice + Wealthy is legal and is shown 20 XP
per year that the book denies him.

**The sibling sweep found no second caller, and no new defect.** `virtue.journeyman`
(ArMDE:4161) and `flaw.failed_journeyman` (ArMDE:6062) state no nullification;
`flaw.failed_master` correctly ships `grants_reputation { local, 4 }` for
ArMDE:6066's *"bad Reputation of 4 in town"*; and `virtue.guild_master`'s
unencoded *"You may select Academic Abilities at character generation"* is
already **F-102**.

**What this obliges.**

- A suppression keyed on the holder, not a general "nullify any effect" variant —
  one caller, so keep it narrow (D38, D21).
- `incompatible_with` stays **rejected**: the book neutralises rather than
  forbids, and the field is symmetric, so it would drag `virtue.wealthy` and
  `flaw.poor` into naming Guild Apprentice back. F-100's `incompat` half is
  answered *against* encoding.
- The rule still goes in `description` in both locales — the in-play progression
  it describes is worth stating (D20).
- The second half of Q-26 (`virtue.indescribable_face`) follows from **D46** and
  **D9**: the passage states *"more perfect"* / *"less perfect"* and no mechanic,
  so it stays **`narrative`**, while the required choice is still recorded as a
  parameter (F-112 stands on its own).

**Lesson.** The first version reasoned confidently from *"the engine models no
stage progression"* without checking whether the stage existed as an entry. The
catalogue was one grep away.

---

## D46 — classification follows *what* is computed, never *where* it is computed

**Question (Q-23).** Does a rule modelled on the **type profile** rather than on
the Virtue make the Virtue a `creation_effect`? Raised for `virtue.the_gift` and
`virtue.hermetic_magus`, and B04 noted it recurs for every Virtue whose mechanic
lives on a profile.

**B04's precedent has since evaporated.** Its case for `creation_effect` was that
`virtue.devil_child` ships effect-less *"precisely because Devil Child's budget
bonus lives on the Mythic Companion type profile"*. **D32 deleted that bonus**
from the data *and* the model, so there is no profile-held mechanic for that
entry to point at. The question had to be decided on its own merits.

**Ruling: by what is computed, not where.**

- Every rule the entry states is computed — on the entry or on a profile —
  → **`creation_effect`**.
- Any stated rule is computed **nowhere** → **`uncomputed_rule`**, and it owes
  its text in both locales.

**The two instances.** `virtue.the_gift` becomes **`uncomputed_rule`**:
ArMDE:2870-2876's *"suffers all the penalties of The Gift"* reaches the player
through no number and no text today. `virtue.hermetic_magus` stays
**`creation_effect`** — *"All magi must take this as their Social Status, and
only magi may take it"* is computed, via the magus profile's `required_traits`.

**Why this is the only consistent line.** **D5** already holds that the
description obligation follows the *rule*, not the label, and **D20** made
`uncomputed_rule` the mechanism that *enforces* it. A location-based rule would
let an entry hold uncomputed rules and owe no text merely because something
*else* about it is computed elsewhere — which is the gap that produced D20's
nineteen entries.

**What this obliges.**

- Reclassify `virtue.the_gift` and write ArMDE:2870-2876's penalties into
  `description` in both locales.
- **Re-examine the five effect-less `creation_effect` entries.** `README.md`
  calls them *"a lead, not a conclusion"*, and each must now be tested against
  this rule rather than against each other.
- **Fix the guard's asymmetry.** `every_vf_is_classified` requires effects on
  `in_play_effect` and not on `creation_effect`, which is why an effect-less
  `creation_effect` passes silently today. The guard should express D46's test,
  not the presence of an `effects` array.

---

## D45 — the surfaced list is a diagnostic, so a modifier names its source

**Question (Q-100).** Six shipped Flaws render as the identical unattributed
string *"Special casting: Circumstantial"* (`amount: 0` is suppressed), and a
character holding two of them sees the same line twice with nothing to tell them
apart (F-423). Is `surfaced_modifiers` a **diagnostic**, where identity matters,
or a **summary**, where it does not?

**Ruling: a diagnostic.** `SurfacedModifier` gains `source: Id`, and the UI
renders the producing item's localized name beside the family.

**What changed since B12 asked.** Its strongest argument was that for the four
`circumstantial` carriers with no `description`, the anonymous row was *the only
thing the player is given*. **D20 removes that argument** — all 19 surfaced-only
entries now owe their rule in `description` in both locales. The ruling therefore
does not rest on it; it rests on the duplicate-row defect, which D20 does not
touch.

**What this obliges.**

- `source: Id` on the DTO, populated at every push site in
  `derived.rs::in_play_mods`, which already holds the item. Purely additive.
- **Render it through the label map, never as the id.** CLAUDE.md is explicit
  that a raw id or enum value must never reach the user; this is the same rule
  that governs `category-<id>` and `magnitude-<id>`.
- It widens a DTO that **crosses the IPC boundary**, so the frontend type mirror
  and the save/round-trip surface both need checking.
- **All five `ModifierFamily` values gain it**, not only `SpecialCasting` — the
  defect is the DTO's, not that family's.
- **De-duplication was considered and is not required.** Naming the source makes
  two rows legible, which is the defect. Collapsing identical families into one
  row listing every contributor is a further UI change and should be judged on
  its own, not smuggled in here.

---

## D44 — an entailed incompatibility may ship, but it must be marked as entailed

**Question (Q-19).** `virtue.gentle_gift` and `flaw.blatant_gift` declare each
other in `incompatible_with` (symmetric, so the load passes) and **neither
passage states it**. The book writes this exclusion explicitly when it means it —
ArMDE:5717 (Blatant Magical Air): *"a character may not have both Flaws"*;
ArMDE:6895 (Unbearable to Beings): *"it cannot be combined with the Blatant
Gift"* — both about *other* entries excluding Blatant Gift.

**Ruling: keep it, as a hard block.** The pair is definitionally contradictory —
a Gift cannot be both subtle and −6 on every interaction roll — so the exclusion
follows from the two descriptions even though neither names the other.

**The precedent, stated narrowly on purpose.** An incompatibility may be asserted
without a passage **only when each entry's own text contradicts the other's**.
That is *entailment*, not inference from theme, plausibility, or balance. "These
two feel like they shouldn't combine" is not this rule, and neither is "no
sensible character would take both".

**What this obliges.**

- **Mark entailed pairs in the data or in `RULES.md`**, with the two passages
  that contradict. An unmarked unsourced constraint is indistinguishable from an
  invented one, and the next auditor pays to re-derive it — which is exactly what
  happened here.
- **Measure the rest.** **81** `incompatible_with` declarations ship and nobody
  has counted how many cite a passage. B03 deliberately left that measurement to
  this ruling; it is now owed, and each unsourced pair must be shown to be
  entailed or removed.
- **Do not generalise to prerequisites.** This is about a *negative* constraint
  between two entries whose texts collide. A missing prerequisite is an absence
  of permission and stays "source or nothing".

---

## D43 — a restricted XP pool permits spending *itself*, and nothing more

**Question (Q-27).** `types.rs::Effect::AbilityAuthorization`'s doc comment makes
the over-reach deliberate: *"A `RestrictedAbilityXp` pool **already implies
permission** for what it funds … since the grant would otherwise be
unspendable."* ArMDE:4065 denies the wider half in the same breath as granting
the pool: *"you have an additional 50 experience points to spend on Order of
Hermes Lore, Magic Lore, or Latin. **You cannot spend other experience points on
Magic Lore or Latin** unless the character has another Virtue or Flaw permitting
this."*

**Ruling: scope the implied permission to the pool.** `RestrictedAbilityXp`
permits spending **its own** experience points on the listed Abilities. Spending
**general** XP on them requires an explicit `AbilityAuthorization`.

**A second entry states the same rule, and it was raised separately as Q-59.**
ArMDE:4808 (`virtue.privileged_upbringing`): *"You may not, however, buy Academic
or Martial Abilities with your **normal pool** of experience points unless you
have another Virtue or Flaw permitting that."* **F-234** records that the engine
grants exactly the permission this passage forbids. B07 offered three options and
its option (b) — separating the permission axis from the funding axis — is this
ruling; it also said the 28-carrier survey was beyond its batch, which is why
that survey is an obligation here. Two independent entries stating the rule is
the strongest evidence that the over-reach is the engine's and not a
per-entry authoring slip.

**The existing reasoning was half right, and that is why it survived.** "The
grant would otherwise be unspendable" is a sound argument for permitting the
*pool* — and no argument at all for permitting anything else. The doc comment
states the narrow warrant and then takes the wide permission.

**What this obliges, including the risk.**

- **28 entries carry `restricted_ability_xp`.** Every one must be re-read against
  its passage and given an `AbilityAuthorization` where the book grants general
  access — Educated's *"You may buy Academic Abilities during character
  generation"* is that shape and must keep working.
- **Until that pass is complete the change removes access characters legitimately
  have**, so the two land together: narrowing the implication and adding the
  authorizations is **one slice**, not two. Shipping the narrowing alone is a
  regression.
- Rewrite the `AbilityAuthorization` doc comment; it currently records the
  rejected reasoning as fact.
- Re-derive the 28 at implementation; assert behaviour, not the count.
- Note the asymmetry in ArMDE:4065 itself — it bars general XP on **Magic Lore
  and Latin** but says nothing about **Order of Hermes Lore**, the third Ability
  its own pool funds. Read that before encoding it; do not tidy it into a
  symmetric rule.

---

## D42 — the concept gains a default realm, and every Supernatural entry records one

**Question (Q-08).** ArMDE:2961: *"**All** Supernatural Virtues and Flaws are
associated with one of the four realms."* Today **4 of ~115** record it, through
an existing `realm` parameter domain.

**The first draft of this decision quoted the book's default and ignored its
condition** — *"this should be the choice if the character **concept** does not
suggest another option"*. Norbert caught it: the concept is **entirely free
text** (`name`, `description`, `concept`, `gender`, `birth_year`, every one
documented *"no mechanical effect"*), so the condition points at something the
model does not carry, and no per-entry default could honestly be said to follow
the book.

**Ruling, two parts.**

1. **The concept gains one optional default realm**, which seeds every
   Supernatural entry's realm.
2. **Every Supernatural entry records a realm** — seeded from the concept,
   overridable per entry, and **fixed** where the book fixes it (Faerie Blood
   and Strong Faerie Blood are always Faerie, ArMDE:2961).

**What it is not.** *Not* a character attribute with mechanical force. The book
never says a character has a realm — it says each Supernatural Virtue does, and
the concept is what *suggests* which. A character may legitimately hold a Faerie
Blood and a Magic-realm Second Sight at once, so the concept value is a **default
source and nothing else**. Anything that reads it as "the character's realm" is
wrong.

**Why record it at all, given the engine models no auras.** The consequences are
in-play — *"determines how it interacts with supernatural auras"* and immunity to
Warping from a same-realm aura — so nothing is computed at creation. The sheet
still has to state it, because the book asserts it of every such entry and the
player needs it at the table. D9 (record every stated choice) and D20 (the rule
must reach the player) both bite.

**What this obliges.**

- One optional field in the concept phase, with a realm domain, and the UI to set
  it. It must be **omittable** — an unset concept realm means each entry is
  answered on its own.
- ~111 entries gain the parameter; the book-fixed ones get a fixed value, not a
  default.
- **Establish which entries the book fixes before writing data.** ArMDE:2961
  names two examples and says *"A Virtue's description notes if it is limited in
  this way"* — so the list is discoverable per entry and must be read, not
  guessed.
- Assert behaviour, never the count of Supernatural entries (catalogue size is
  data).

---

## D41 — one Social Status is mandatory and hard; a second is a warning

**Question (Q-45).** ArMDE:2816: *"All characters **must take one** Social
Status, and may only take **more than one** if the descriptions of the Virtues or
Flaws **explicitly note that they are compatible**."* That is two rules, and a
`virtue_category_caps` row expresses neither — it cannot state a **minimum**, and
it cannot carry the per-entry exception.

**Ruling.**

1. **"Must take one" is a hard error.** It is absolute, universal and trivially
   checkable.
2. **A second Social Status raises a warning, never an error.** No per-entry flag.

**Why the exception is not modelled — the book's three cases decide it.**

| Entry | The stated exception |
|---|---|
| ArMDE:4325 (Redcap) | *"compatible with **any** other mundane Social Status that would **reasonably** allow you to do your job"* — hedged, examples only |
| ArMDE:4614 | *"compatible with any Social Status **normally restricted to men**, and any Social Status compatible with…"* — a predicate, and recursive |
| ArMDE:4441 | **requires** a second guild Social Status alongside its own |

None is an id list. ArMDE:4614's predicate needs a sex model, which **Q-05/D16
ruled out as a deliberate non-goal**, so it is unmodellable **by decision**, not
by omission. Redcap's *"reasonably"* is D16's hedged wording, which D16 says is a
warning and never an error. A flag would therefore hard-block pairings the book
permits, and its author would be guessing at what "reasonably" covers.

**What this obliges.**

- A category **minimum** in the profile budget — new, since only caps exist.
- A warning on a second `social_status`, carrying the entry names so the player
  can see what was paired.
- **Do not add a `compatible_with` list.** It would model none of the three cases
  and would look authoritative while being a guess.
- **This is D21's mechanism, not a new one**: D21 closes Q-07 and Q-102 as "the
  same ArMDE:2816 machinery". D41 says what that machinery must *say*, not how it
  ranges over a category.
- ArMDE:4441's *requirement* of a second status is a separate obligation and is
  **not** discharged here.

---

## D40 — replacement XP is one effect that names a life stage

**Question (Q-106).** `flaw.feral_upbringing` ships an **additive**
`restricted_ability_xp: 120`, but ArMDE:6113 says *"In your first five years you
gain 120 experience points"* — and the standard childhood block is **already**
120 (`native_language_xp: 75` + `spread_xp: 45` over the same five years). The
Flaw **replaces**; the data adds, so a feral character is funded **240** (F-428).

**Ruling: one effect that names a life stage** —
`replaces_life_stage_xp { stage, amount, abilities }` or equivalent. One concept,
one rule, one place in the data.

**It has two carriers, and they are the reason for the shape.** D13 named the
three XP modes — additive (modelled), earmark (D13), **replacement (unmodelled)**
— and said they must be designed together. Replacement's carriers are F-428
(`childhood`) and **F-439** (the Redcap's 300, D17, `apprenticeship`). Both name
a stage `rules/core/life_stages.json` already defines, so one variant covers
both. **Design against both; a variant shaped around `childhood` alone would not
fit apprenticeship's 15 years.**

**Why not the alternatives.** A *suppress flag plus the existing additive grant*
splits one book rule across two effects, and an entry could then carry the
suppression without the grant — silently costing a character 120 XP, which is the
same class of defect as the one being fixed. *Special-casing the two ids in the
engine* puts rules in code, the inversion CLAUDE.md's data-driven design exists
to prevent.

**What this obliges.**

- The variant, plus its consumer in the XP computation, plus `RULES.md`.
- **Not covered by it, and still owed:** ArMDE:6113's *"you may not start with a
  score in a Language"* and *"you may only choose beginning Abilities that you
  could have learned in the wilds"*. Neither is an XP rule; they are
  authorization rules and must not be smuggled into this effect.
- **Saves:** replacement *lowers* a funded total, so characters legal today can
  fail validation — same treatment as D10 and D28, reported and blocked in
  `Enforced`, no migration and no schema change.
- Re-derive both amounts at implementation rather than trusting 120/300 here.

---

## D39 — a truncated English sentence is reconstructed from the English, not translated back

**Question (Q-82).** ArMDE:5725 shipped *"Regular casting tools remain as Arcane
Connections for a few weeks, **his last years**."* — not a sentence. The
line-mirrored German is complete: *"…bleiben wenige Wochen als Arkane
Verbindungen erhalten, die zuletzt verwendeten bleiben es **jahrelang**."* May
the German be used to repair the English?

**Ruling (Norbert): use the English source and reconstruct accordingly.**
English stays the source of truth. The repair is an **English-side
reconstruction from the surviving fragment**; the German may *corroborate* the
reading but is never the source of the wording.

**DONE.** ArMDE:5725 now reads *"…for a few weeks, **but** his last **for**
years."* Two function words, both demanded by the fragment's own grammar and by
the contrast it already sets up ("a few weeks" against "years"). The shipped
`summary` carries only the entry's first sentence, so **no i18n change** was
needed and no German text moved.

**Why this is not a licence to back-translate.** The distinction is the whole
ruling: a reconstruction is constrained by the English words that survived, so it
can be checked against them. A back-translation invents English from German and
is checkable only against the translation — which would quietly make the German
the source of truth for English text, inverting the repository's rule.

**The test for the next one.** If the surviving English does not determine the
missing words, do **not** reconstruct. Write the rule into `description` instead
(D20), where it is honestly our prose rather than presented as the book's.

**Same class as D26**, and the same treatment: an OCR defect in the source is
corrected in the source, because the JSON is generated from the Markdown and an
i18n-only patch would be silently undone by re-extraction.

**What remains on F-370.** Only its other half — `narrative` over an
Arcane-Connection duration rule. The source defect is discharged.

---

## D38 — "only companions can take this" is stated on the entry, not in three profile lists

**Question (Q-75).** ArMDE:2394 closes with *"only companions can take this
Virtue or Flaw"* (Wealthy / Poor). Today the rule is enforced **twice explicitly
and once by accident**: `magus` and `mythic_companion` list both ids in
`forbidden_traits`, while a **grog** is refused only because both entries are
**Major** and the grog profile sets `max_major_virtues: 0` /
`max_major_flaws: 0`. F-339 is that asymmetry.

**Ruling: express it on the entry**, per **D24** — *"when an entry … needs a
narrower audience, the narrowing belongs on the entry as a `Prereq`, not on the
profile"*. One statement of the rule instead of three, and it cannot rot when a
profile changes.

**Why the incidental block is not good enough.** It is right by coincidence.
Make either entry Minor, or give grogs a Major allowance, and a rule the book
states stops being enforced — **and no test fails**, because nothing asserts the
rule, only its side effect. That is the shape this audit has hit repeatedly:
correct output, wrong reason, silent when the reason moves.

**What this obliges.**

- A `Prereq` that can name a **character type**. Only `IsMagus` exists today, and
  it reads the profile's own `is_magus` flag rather than an id — so this is a new
  variant, and `Prereq`'s exhaustive `match` makes adding one a compile error
  until every site handles it (which is the point).
- `virtue.wealthy` and `flaw.poor` carry it; **the two `forbidden_traits` entries
  come out** once it works, or the rule is stated in two places again.
- **The second caller is already known, and it changes the shape.** Sweeping the
  core book for sibling sentences found exactly **two**: ArMDE:2394 and
  **ArMDE:4375**, *"only a companion or **magus-level character** can take this
  Virtue"* (`virtue.magical_mount`, filed as **F-553** — unencoded today, so a
  grog may take it). That one names *two* audiences, so a single-type variant
  must compose under `Prereq::Any`. **Design against both, not against the first.**
  *"magus-level character"* is undefined in the passage and needs a reading before
  the data is written; it plausibly covers `mythic_companion` too.
- Reachability per **D2**: the check must see *granted* selections, not only
  bought ones.

---

## D37 — Magic Resistance competes, it does not stack: True Faith takes the higher total

**Question (Q-74).** ArMDE:17611 gives *"a True Faith Score gains Magic
Resistance equal to this score multiplied by ten"*, and
`derived/casting.rs::magic_resistance` computes nothing from it (F-329). Does it
**replace** the per-Form grid, **stack** with it, or **compete**?

**Settled from the source, not by choice.** ArMDE:2627 states the general rule:
*"If a character receives Magic Resistance from more than one source, these
totals **do not stack**, even if they derive from the same Realm … you simply use
the **higher total**."*

**Ruling: compete.** For each Form, resistance = **max(the Form's existing total,
True Faith score × 10)**.

**The book's exceptions confirm the default rather than undermining it** — they
are stated per entry, and both say "add" explicitly: ArMDE:4035, a guardian
angel's 15 *"is not compatible with a magus's Parma Magica … but it **does add**
to the Magic Resistance resulting from Faith Points"*; ArMDE:3583, Commanding
Aura's resistance *"is added to that of the relic"*. So an entry that adds must
say so, and silence means compete.

**What this obliges.**

- `derived/casting.rs::magic_resistance` folds `effective/might.rs::true_faith`
  in as a **floor across every Form**, not as an addend. The score is already
  modelled (`Effect::TrueFaithGrant`, summed and clamped), so this is a consumer
  change only — **F-329 needs no new data**.
- **Do not generalise the max to the two exceptions above.** They are additive by
  their own wording and belong to their own entries; folding them into the same
  rule would be the "one mechanism for two ideas" error D21 warns about.
- A character with **Faith Points but no True Faith Score** gets **nothing**
  (ArMDE:17611 says so in the same breath) — the floor keys on the *score*.

---

## D36 — a canonical German term binds wherever it *names a game element*, prose included

**Question (Q-63).** Does a translation table's term bind only on an entry's
**label**, or also **inside a sentence**? Three findings wait on it: F-65
(*Fähigkeit* for **Fertigkeit**), F-320 (*Widerstandsfähigkeit* for Soak) and
F-473 (Magical Air called by a name the app does not use).

**Ruling: it binds wherever the word names a game element the app shows** — a
label, a summary, a description, an export. **Ordinary-language use of the same
word is untouched.** All three findings are live and fixable.

**Why, and F-473 is the case that decides it.** A description that names a game
element by a word appearing nowhere in the UI is a **broken cross-reference
inside our own product**: the player reads the tooltip, looks for that thing in
the app, and it is not there. That is a defect, not a matter of style. F-65 and
F-320 are milder — *Fähigkeit* and *Widerstandsfähigkeit* are ordinary German a
reader understands — but they fail the same test, because both sentences are
pointing at something the app displays.

**The line, stated so it can be applied.** Ask whether the reader could want to
*find* the thing named. "Deine **Fertigkeit** steigt" names the Ability score the
app shows → binds. "Er ist eine **Fähigkeit**, die selten ist" used as plain
German, naming nothing on screen → free.

**Which term is canonical is D31's question, not this one.** D31 rules that on a
**name** the translation table beats the German rulebook heading, our
`rules/source/de/` copy being older. D36 only says *where* the winner binds.

**What this obliges.**

- Fix F-65, F-320, F-473 in `rules/i18n/de/`, and check the rest of the German
  prose for the same shape rather than only these three.
- **No guard.** A scanner matching table terms across German prose would fire on
  every ordinary-language use, and the audit has already learned that a noisy
  green guard proves nothing. This is enforced at review time, which is a
  deliberate choice and should not be "fixed" later by adding the noisy test.
- Both locales stay in scope of any slice that touches the text (standing rule).

---

## D35 — 30 XP per finished year is a *rate*, so the parameter model gains a number

**Question (Q-67).** `virtue.simple_student` (ArMDE:4960) grants *"30 experience
points **per finished year** that he can apply to Latin or Artes Liberales"*, and
states no year count. The entry ships `creation_effect` with **no effects at
all**, so it grants nothing today.

**Ruling, two parts.**

1. **Model it properly: a numeric parameter and an effect that scales by it.**
   `ParamType` has exactly one variant today (`Ref`); it gains a number variant
   with `min`/`max`, and `restricted_ability_xp` gains the ability to multiply
   its amount by a parameter value. Interim text-only was rejected — the XP has
   to reach the player.
2. **The cap is 2 finished years (60 XP), derived from the catalogue rather than
   invented.**

**Why 2, and why not an age formula.** The rate is a *family* mechanic, and the
degrees pin the year counts:

| Virtue | Years | XP | Source |
|---|---|---|---|
| Simple Student | 1–2 | 30/yr | ArMDE:4960 |
| Baccalaureus Artium | **3** | **90** | ArMDE:3472 — *"30 experience points per finished year"* |
| Magister in Artibus | **8** | **240** | ArMDE:4389 |
| Doctor in Faculty | **10** | **300** | ArMDE:3687 |

A Simple Student's **third** finished year completes the Baccalaureus, which is a
different Virtue with its own 90 XP — so 2 is the ceiling, and ArMDE:4960 says as
much: *"If he has finished his **second** year … he is in the liminal position of
either applying for work or continuing his education."* The ages corroborate
rather than drive it (Simple Student 14–16, Baccalaureus 16–19 after three
years, both implying entry around 14).

**An age formula was considered and rejected.** `age − 15` or similar breaks on
the perpetual student: a 30-year-old Simple Student would compute 15 finished
years and 450 restricted XP. The degree ladder has no such hole and is RAW.

**What this obliges.**

- `ParamType::Number { min, max }` — **designed with D9** (multi-valued
  parameters) and **D14** (constraints on parameter references), which extend the
  same struct. Three decisions touching `ParameterDef` must produce one coherent
  model, not three fields.
- A parameter-scaled `RestrictedAbilityXp`. Note this is the **additive** XP mode
  of D13, not the earmark or the replacement mode.
- `virtue.simple_student` gains the parameter (`years`, 1–2) and the effect, and
  keeps `restricted to Latin or Artes Liberales`.
- **The other three family members stay fixed totals** — their year counts are
  stated, so 90/240/300 are correct as constants. Do not parameterise them for
  symmetry.
- A UI number input, and the rule in `description` in both locales regardless
  (D20) — the cap's *reasoning* is not obvious from the entry alone.

---

## D34 — `flaw.false_power`'s domain is the Supernatural category plus two named ids

**Question (Q-103's remedy).** The entry's parameter carries
`require_categories: ["hermetic", "special", "supernatural"]`. ArMDE:6082 governs
with *"One of the character's **Supernatural Virtues**"* and then widens by
example: *"can apply to Supernatural Virtues that define the character's
background, **like** Faerie Blood, Diedne Magic, or even The Gift."* Those three
sit in three different categories (`supernatural`, `hermetic`, `special`), which
is why all three are admitted — at the cost of **56** Hermetic Virtues where the
book names one.

**Ruling: whitelist the named ids.** Domain = the `supernatural` category **plus
`virtue.diedne_magic` and `virtue.the_gift`**. Both derivations agreed 56 is
wrong; this is the remedy.

**What it costs, stated plainly.** The book's *"like"* is an **open** list and a
whitelist closes it, so a future background-defining Hermetic Virtue must be
added by hand. That is the deliberate trade: a missing id is **visible** the
moment someone looks for it, whereas 55 wrongly-admitted ones are invisible and
silently let a player build something the book never contemplated.

**What this obliges.**

- A **one-id whitelist** field on `ParameterDef` (`allow_ids`, or equivalent),
  additive to `require_categories`. ~1 field, and it **lands beside D14**, which
  is already adding constraint expressiveness to parameter references — design
  the two together.
- **Not** routed through **D23**'s predicate. A predicate would need a data
  property meaning "defines the character's background", which the catalogue does
  not carry and which nothing else would use.
- `virtue.faerie_blood` needs no entry — it is already `supernatural`.
- **Re-derive the 56 at implementation.** The figure is a census of the
  `hermetic` category, and the catalogue is data: assert the *behaviour* (Diedne
  Magic admitted, an arbitrary other Hermetic Virtue refused), never the count.

---

## D33 — ArMDE:6148 restricts what Flawed Powers *imports*; it is not an incompatibility

**Question (Q-138's residue).** `flaw.flawed_powers` makes the character *"suffer
the effects of a **Major Hermetic Flaw** … applied to her Supernatural Virtues
rather than to her Hermetic magic **(if any)**"*, then adds: *"Any Flaw that is
only appropriate to Hermetic Magic (for example, Deficient Technique or
Unstructured Caster) cannot be taken with this Flaw."* Does that constrain the
**imported** Flaw, or forbid **holding** such Flaws at all?

**Ruling: it constrains the import.** The character may still hold Deficient
Technique or Unstructured Caster in her own right.

**Why.** *"(if any)"* concedes the character may be a Hermetic magus. Under the
incompatibility reading that magus is barred from Deficient Technique for a
reason the passage never gives; under this reading the sentence says only what it
must — a Flaw about Techniques cannot be *applied to Supernatural Virtues*, so it
is not an eligible import.

**This changes what D23 builds, which is why it was worth asking.** An import
restriction is a **parameter constraint** (D14's shape: the imported Flaw is a
parameter value, and the constraint restricts its domain). An incompatibility
would have been **D23's exclusion predicate**. **D23 assumed the latter and was
wrong on this entry.** D23's mechanism stands for Q-137; it simply does not carry
this case.

**What this obliges.**

- `flaw.flawed_powers` gains a **parameter** naming the imported Major Hermetic
  Flaw — which it does not have today (it ships no `parameters` at all) — with a
  constraint excluding the only-Hermetic ones. This is a **D9** instance as well
  as a D14 one: a stated choice nobody records.
- The predicate itself is still D12's **trained** flag (D23's finding), so D12's
  classification pass remains the prerequisite. Only the *place* it attaches
  changes.
- **Do not** add an `incompatible_with` or a `Prereq::Nor` for this entry.

---

## D23 — exclusions may be predicates, not only lists of ids

**Question (Q-137, Q-138).** The book excludes Flaws by **description** where
`incompatible_with` can only hold **ids**.

- **Q-137** — ArMDE:6925, `flaw.university_dean`: "*can not have the Poor Flaw or
  **any other Flaw that grants a Bad Reputation***". That set is **16 Flaw ids**
  today and is **derivable** from the data — a Flaw either carries
  `grants_reputation` or it does not.
- **Q-138** — ArMDE:6148: "*Any Flaw that is **only appropriate to Hermetic
  Magic** (for example, Deficient Technique or Unstructured Caster) cannot be
  taken with this Flaw.*" This is **not** derivable: `categories: ["hermetic"]`
  also contains Flaws that are not *only* Hermetic.

**Ruling (Norbert, 2026-09-22): add predicate-valued exclusions.** One mechanism
for both, rather than ids-plus-a-regenerating-test for one and prose for the
other.

**Why not the id list.** It would work for Q-137 — a test could assert the list
still equals the derived set and fail when a seventeenth Flaw gains a Reputation
— but it freezes a *number* of entries into data, which sits badly with
`CLAUDE.md`'s "catalogue size is data, never code", and it does nothing at all
for Q-138.

**This is NOT what D21 added.** D21 gave the engine the ability to range over a
**category**; these are **predicates** — "grants a Reputation", "is only
appropriate to Hermetic Magic". Related machinery, different quantifier, and both
are needed. Design them together for the same reason D21 refuses to be split.

### Q-138's predicate already has a home — it is D12's `trained` flag

**This is the part worth not rediscovering.** D12 ruled that Hermetic V/F divide
into **intrinsic** (operating on The Gift itself) and **trained** (operating on
Techniques, Forms, spells, Casting or Lab Totals, Parma Magica, certámen or
Twilight — things that exist only after apprenticeship), and obliged a
classification pass over all 122 `hermetic` entries.

**"Only appropriate to Hermetic Magic" is that same predicate.** ArMDE:6148's two
worked examples are `flaw.deficient_technique` and `flaw.unstructured_caster` —
both `categories: ["hermetic"]`, and both squarely **trained** under D12's
criterion. So D12's classification pass **produces the data Q-138 needs as a
by-product**; no second judgement pass over 122 entries is required, and no new
flag needs inventing.

Q-138 therefore becomes: run D12's pass, then point the predicate at its result.

### What this obliges

1. **A predicate-valued exclusion shape**, designed alongside D21's category
   variant.
2. **Q-137's predicate is "carries a `grants_reputation` effect"** — derivable
   with no new data. Verify the count at implementation time rather than trusting
   16; B18 re-derived it from `jq` as **16 ids across 17 effect rows**, because
   `flaw.failed_monk` carries two.
3. **Q-138's predicate is D12's `trained`**, and so is **blocked on D12's
   classification pass**, not on this ruling.
4. **Reachability, again.** B15's F-466 correction applies here as it does to
   D21: `validate_incompatibilities` reads **bought** selections on both sides,
   so an exclusion that must also catch a *granted* Virtue needs `Prereq::Nor`.
   A predicate inherits the trap.
5. **`virtue.doctor_in_faculty` as a prerequisite is not blocked on any of
   this** — it is a plain `Prereq::Has` and can land with F-526's other clauses.

**Q-138 also unblocks an entry in another batch's span.** It sits on a **B13**
entry and is carried by no finding in B13, B17 or the index — recorded here so
that entry is not left silently unrated.

---

## D22 — cross-book rules bind in principle; the implementation is core-book-only

**Question (Q-135), two parts.** Is `flaw.seeker` magus-only? And — the policy
call — **does a rule in another book bind an entry whose `source` cites ArMDE?**

B17 escalated this rather than decide it, correctly: the second part has
consequences far beyond the entry.

### Part 2 — the policy

**Ruling (Norbert, 2026-09-22): cross-book rules DO bind in principle. But the
current implementation is core-book-only, and that boundary is stated
explicitly rather than left implicit.**

**What this obliges.**

1. **Every known cross-book rule is filed as a finding**, so the debt is visible
   instead of silent. Deferred work nobody wrote down is just a gap.
2. **No supplement sweep is undertaken now.** Honouring the principle properly
   means reading eight supplements for statements about core entries, which is
   unbounded — and the half-done version is *worse than either pure position*: a
   catalogue where some supplement rules are in and most are not, with no way to
   tell which.
3. **When it is undertaken, `source` must be able to hold a second citation.**
   It is a single `SourceRef` today and structurally cannot.

**The cost of deferring is currently one entry.** `flaw.seeker` is the **only**
cross-book instance the whole audit found. The four other findings of a similar
shape — F-442, F-452, F-453, F-478, "a rule the book states elsewhere about a
named entry is structurally invisible" (`corrections.md` § 3.1's second blind
spot) — are all ArMDE citing *itself*. So nothing is lost by waiting, while the
sweep that would find more is open-ended.

### Part 1 — `flaw.seeker` gets a magus prerequisite, on core-book evidence

**And the two parts turn out to be independent**, which B17's framing missed.
The entry does not need HoH:TL at all. ArMDE:6715, verbatim:

> "You are a self-proclaimed member of the Seekers, a loose organization of
> **competitive magi** searching for ancient magic and arcane artifacts. … Your
> interests may occasionally clash with other interests of **your House** or
> covenant."

**Two core-book signals, neither of them a stated restriction, both pointing the
same way:** the Seekers *are* an organization of magi, and the Flaw assumes the
character has a **House** — which grogs and companions do not.

**Ruling (Norbert): yes, a magus prerequisite.** Use `Prereq::IsMagus`, per D12's
normalisation — it reads the profile's own flag (`types.rs:2685`) rather than
requiring a specific Virtue to be selected.

**What the cross-book sentence was actually doing, recorded so nobody re-reads it
as the source of the restriction.** HoH:TL:503 says *"A magus from any House may
be a Seeker"* — that sentence's job is to **remove a House restriction**, not to
impose a magus one. It presupposes magus-hood only incidentally. So it is
*consistent* with this ruling and is not its warrant.

`flaw.seeker` moves from **escalated** to **checked**. Its classification stays
`narrative`: the passage states a drive and no mechanic, and the prerequisite is
an eligibility fact rather than a rule the entry states.

---

## D21 — the engine learns to talk about categories, not just entries

**Question (Q-132).** Should `Prereq` gain a variant that ranges over a
`categories` value? `flaw.rector` requires "a Social Status Virtue", and **99**
catalogue entries carry `social_status` — verified by `jq`.

**Ruling (Norbert, 2026-09-22): yes. Add the category variant, and its
Effect-side twin.**

**Why not the alternatives.** Enumerating the 99 ids in a `Prereq::Any` breaks
`CLAUDE.md`'s load-bearing invariant — *"Catalogue size is data, never code …
V/F are added by editing `rules/core/*.json` with **zero code changes**"* — and
the list would silently rot every time a Social Status is added. Text alone
enforces nothing: under **D20** the rule would reach the player, but the app
would keep building characters the book forbids.

**One mechanism closes six open items**, which is the real argument for it:

| Item | The rule it cannot express today |
|---|---|
| **Q-132** | `flaw.rector` — "requires a Social Status Virtue" |
| **F-427** | ArMDE:2816 — "All characters must take **one** Social Status", enforced nowhere; `social_status` appears in `character_types.json` only under `permitted_categories`, never in a caps array |
| **Q-07**, **Q-102** | the same ArMDE:2816 machinery |
| **F-542** | `flaw.weak_personality` (ArMDE:7078) — "may have **no other** Personality Flaws", with four legal-today pairs inside B19's own 25 |
| **F-355** | `flaw.ability_block` — a **category** block over Abilities (ArMDE:5653, "class of Abilities"), the Effect-side twin |

**Note F-355 and F-542 are the same missing mechanism, not two** — B19's closure
notes said so, and this ruling is where that observation pays off.

### What this obliges

1. **A `Prereq` variant ranging over a category.** `Prereq` is an exhaustive
   `match` by design, so adding a variant is a compile error until every consumer
   handles it — which is the intended safety and should not be worked around.
2. **An Effect-side twin that can *forbid*** a category, for F-355 and F-542.
   The engine today has a *grant* for an Ability category and no *forbid*.
3. **ArMDE:2816 needs a floor, not only a cap.** "Must take one" is not
   expressible as a `*_category_caps` row, which only bounds from above. Whatever
   shape this takes must express *at least one*, or F-427 stays open with new
   machinery sitting next to it.
4. **Check reachability, not just the passage.** B15's F-466 correction applies:
   `validate_incompatibilities` reads **bought** selections on both sides, so a
   prohibition that must also catch a *granted* Virtue needs `Prereq::Nor`, not
   `incompatible_with`. A category variant inherits that trap.

**Sequencing.** Independent of D19's screen work and of D14, so it can run in
parallel. It should *not* be split across the Prereq and Effect sides — they are
one design, and building one first invites a second incompatible spelling, the
same failure D13 warns about for the three XP modes.

---

## D20 — RAW fidelity is the test: every rule must reach the player somehow

**Question (Q-136).** Does a **surfaced-only** effect kind satisfy
`in_play_effect`, or is `uncomputed_rule` the honest class? It governs **19**
entries, and two batches set opposite precedents a week apart — B17's F-515 left
`flaw.short_ranged_magic` as `in_play_effect`; B18 escalated
`flaw.susceptibility_to_divine_power` rather than accept that.

**Norbert reframed it before ruling: *check each of them whether it is correct to
show a number at all*.** That read was done first, and it changed the answer.

### What the read established

**`amount: 0` is never displayed.** `DerivedSurfacedModifiersSection.svelte:27`
renders `{#if m.amount !== 0}`, so all 19 surface as a **bare label** with no
figure. The `0` is a "no number" sentinel, not an invented one — which clears the
worst suspicion, that the app was showing a number no passage supports (the
`7f5605a` error).

**Verdicts across the 19:** number right **14** · number **missing 5** · number
wrong **0** · number falsely shown **0**. The five missing:
`virtue.commanding_aura`, `virtue.life_boost`, `virtue.leper_magus`,
`virtue.special_circumstances` (mitigated — its +3 is in the summary),
`flaw.corrupted_spells`.

**The worst is `virtue.commanding_aura`.** ArMDE:3585-3591 states eight figures —
*"**Pope:** Magic Resistance 25, Soak bonus +5"* down to *"**Archbishop:** Magic
Resistance 10, Soak Bonus +2"*. The entry ships a bare
`magic_resistance_mod: aura_bonus` with no value, and the English summary stops
at *"…granted by either the pope, or the Divine directly"* — before any
mechanics. **None of the eight reaches the player anywhere.** The kind is wrong
too: `aura_bonus` means aura-conditional, while the passage gives a flat
rank-based resistance, and `derived.rs:296-298` cites this very range as its
warrant.

### The ruling

**RAW fidelity is the test: a rule the book states must reach the player
somehow — as a computed number, or as written text. If the engine cannot compute
it, text is the only route, and if neither happens the app has silently dropped
a rule.**

Measured against that, **all 19 fail today**, not only the five. The fourteen
"numerically correct" entries display a bare category label — *"Special casting:
Circumstantial"* — which names a taxonomy slot and tells the player nothing about
what happens. The book states a real rule; the player gets a word.

**So: all 19 become `uncomputed_rule`, and all 19 owe their rule in `description`
in both locales.** Five additionally owe the number they currently swallow.

**Why the class, given it is read by no production code.** `classification`
changes nothing the user sees; its only consequence is that `uncomputed_rule`
entries are *required* by `every_uncomputed_rule_entry_states_its_rule_in_every_locale`
to carry the text. **D5 already obliges all 19 regardless of label** — the
description obligation follows the rule, not the classification. Choosing
`uncomputed_rule` makes that obligation **enforced instead of aspirational**,
which is the whole reason to prefer it.

**This vindicates B18's instinct and corrects its example.** B18 escalated an
entry that was numerically clean — `RULES.md:4883-4884` is accurate for once
("carries no number of its own — it doubles whatever the scene's aura rating
happens to be", matching ArMDE:6817) — but that entry *was* dropping its rule, so
the escalation was right for a reason B18 did not name. B17's F-515 reading is
superseded.

### What this obliges

1. **19 reclassifications** to `uncomputed_rule`, enumerated in
   `tmp/q136-number-check.md`.
2. **19 descriptions in both locales**, each stating what actually happens rather
   than naming the effect family.
3. **Five number defects fixed independently of the class** — they are the real
   harm and reclassification alone would put none of them on screen.
   `virtue.commanding_aura` additionally needs its effect kind reconsidered
   (`aura_bonus` misdescribes a flat rank-based figure) and a rank parameter;
   `corrections.md` F-39/F-40 cover the Soak half and the missing parameter,
   **the eight MR figures and the summary truncation are new**.
4. **These land after D19's screen work** — `corrections.md` § 2.1. All 19
   currently sit outside what the screen can see, so reclassifying first would
   turn the guard red on exactly the entries being fixed.

**One `RULES.md` claim falls with this.** `RULES.md:4866` lists
`flaw.corrupted_spells` under "flat casting-total bonus/penalty"; its effect list
is only `special_casting_mod: circumstantial`. Another provenance record false as
written.

---

## D19 — the mechanical screen matches by regex, because German negation is discontinuous

**Question.** `MECHANICAL_PHRASES` in `crates/arm-rules/tests/uncomputed_clauses.rs`
is a list of **contiguous substrings**, left-word-boundary matched. B17 measured
that this **structurally cannot express German discontinuous negation**: four of
the five German prohibitions in its span put one or more words between the halves
— `kann … nicht anwenden`, `darf … nicht nehmen`. Adding those forms as literals
would leave `every_uncomputed_rule_entry_states_its_rule_in_every_locale` **red on
exactly the entries the reclassification is meant to fix**, and the pressure that
creates is to reword shipped rulebook text to satisfy a detector — which is what
`ecb5150` was written to stop.

**Ruling (Norbert, 2026-09-22): use regex.**

**Why the obvious objection does not apply.** The screen lives in
`crates/arm-rules/tests/`, so `regex` enters as a **dev-dependency and ships in
nothing**. `CLAUDE.md`'s desktop threat model rates `devDependencies` as a
build-integrity concern, not an end-user exposure, and this one does not reach
the bundle at all.

**What this gates.** `class` (166 findings) + `desc` (116) = **282 of 544** — over
half the correction list. `corrections.md` § 2.4 puts this first for that reason,
and § 3.1 states it must be decided before any data lands.

### What this obliges

1. **Convert before extending, and prove the conversion is inert.** The existing
   **33** literals in 15 bilingual groups must become patterns that match
   *exactly what they match today* — same left-word-boundary semantics — with the
   whole suite green **before** a single new family is added. A conversion that
   silently widens the screen would change which entries are flagged and
   invalidate the measurements B11, B12, B17 and B18 took.
2. **The bounded-gap form is the point.** `kann\b.{0,N}\bnicht` and its relatives.
   **N must be chosen from the German source, not guessed** — measure the actual
   distances in the passages B17 identified and pick a bound that covers them
   without spanning sentence boundaries.
3. **Every added German pattern is verified against the German rulebook, not
   against a plausible rendering of the English.** B18's F-537 found the mirror
   defect: a needle already in the list that matches **nothing the book writes** —
   so the list is not only incomplete, it is partly *inert*. A pattern that
   screens zero passages is worse than an absent one, because it looks like
   coverage.
4. **Case sensitivity gets decided rather than inherited.** The audit already hit
   this: `halbier` occurs 36 times case-insensitively and 31 case-sensitively,
   because 5 occurrences are capitalised. Regex makes `(?i)` free; state the
   choice explicitly either way.
5. **D8's capability family still has no word-form list.** Regex does not supply
   it. The 48 entries D8 reclassifies carry no signed number and no botch term, so
   they depend entirely on a family that does not yet exist — see
   `corrections.md` § 2.1b. This ruling makes that family *expressible*; it does
   not write it.

**What it does not change.** `NO_RULE_DESPITE_TOKEN` keeps its role and its
discipline — an exemption carries the written reading somebody had to produce.
Regex widens what the screen can see; it is not a licence to widen what counts as
a rule.

---

## D18 — the tables are NEWER than the rulebook copy, so a disagreement is not a table error

**This amends D7, whose central assumption turns out to be false.** Recorded
2026-09-21 on Norbert's ruling.

**D7 assumed the DE rulebook heading is the most current authority.** It is not.
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md` is an **older
edition**. The translation tables in `arm-de-translation` have been maintained
against a **newer** German rulebook, so **a table row that disagrees with the
heading may be a correction the shipped book has not caught up with** — not an
error.

**The two cases that established this**, both ruled by Norbert:

| Entry | Shipped / old heading | Current German edition | Ruling |
|---|---|---|---|
| `virtue.relic`, `virtue.powerful_relic` | *Relikt*, *Mächtiges Relikt* (DE :4782, :4852) | **Reliquie**, **Mächtige Reliquie** | table wins; data changed |
| `spell.curse_of_circe` | *Fluch der Kirke* | **Fluch der Circe** | table wins; data changed |

In both, the table was arguing *against* the heading and the table was right. The
Relic row even carried its reasoning ("**Nicht** ‚Relikt'"), which under D7 read
as a terminology table overreaching. It was a newer edition's correction.

### What this obliges

1. **A table–heading disagreement is now a three-way question, not two-way.**
   Before calling a row wrong, ask: is it a table error, a stale copy, **or a
   correction from a newer rulebook edition the repo has not received?** Only the
   first is fixed in the table.
2. **The remaining disagreements are NOT to be auto-fixed.** A sweep found **21**
   table-versus-shipped-data disagreements after the 2026-09-21 re-sync, and the
   obvious action — bring 20 table rows back into line with the heading — would
   have **reverted genuine corrections**. Each must be checked individually
   against the current edition, which only Norbert can consult. The list is in
   `tmp/table-sync-check.md`.
3. **D7's rule 2 stands, narrowed.** The rulebook heading still wins for a name
   *at equal currency*. It does not win against a later edition.

### DO NOT re-sync the rulebook sources — the line numbers will break

**Norbert's standing instruction, 2026-09-21.** A newer German (and English)
rulebook exists, and pulling it in would shift **every line number**. The entire
audit is built on them: 655 `source.lines` ranges, every `ArMDE:NNNN` citation in
code, `RULES.md` and all nineteen batch files, `SWEPT_BLOCKS`, and
`rulebook_citations.rs`'s range guard — which checks only that a range lands on
non-blank lines, so it would go **green on citations that now point at the wrong
text**.

**This makes the `source.anchor` backfill urgent rather than tidy.** B19 measured
coverage at **94 of 655 (14.4 %)**. The anchor is derived from the `####`
heading, so it survives a re-pagination; a line range does not, and fails
silently. **Backfilling anchors to 100 % is the prerequisite for ever accepting a
newer rulebook**, and should be treated as such in Phase 2 rather than as the
low-priority cleanup Q-96 filed it as.

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

**First ruling: needs a rules read first.** Not to be settled by analogy to B1.
**That read is now done — see below.**

### The rules read, 2026-09-21

**ArMDE:3989, `virtue.great_characteristic`, verbatim:**

> "You may **raise** any Characteristic that **already has a score of at least
> +3** by one point, to no more than +5. … You may take this Virtue twice for the
> same Characteristic, and for more than one Characteristic."

**"Already has" describes the Characteristic's state, not the act of purchase.**
The precondition is a statement about what the Virtue can legally operate on — a
Virtue cannot raise a +1 Characteristic however it was acquired, because there is
nothing at +3 to raise.

**The "take" versus "have" split that prompted the question is a red herring.**
ArMDE:3665 (`Demonic Might`) says "You may only **take** this Virtue if your
character has the Demonic Blood Virtue"; ArMDE:3669 (`Demonic Powers`), the very
next entry, states the *same* restriction as "Only a character with the Demonic
Blood Virtue may **have** Demonic Powers". Two adjacent entries, one
restriction, two verbs. The book is varying its prose, not drawing a technical
distinction, so no argument may rest on that pair.

### Reachability — the gap is latent, not live

Checked across all of `rules/core`. Exactly **three** entries carry the governed
effects: `virtue.great_characteristic`, `virtue.puissant_ability` and
`flaw.poor_characteristic` (`ability_bonus` / `characteristic_score_delta_param`).
**Nothing grants any of them** — not the seven House grants in `houses.json`,
not the seven fixed grants in `mythic_companion_types.json`, and not the seven
`grants_selection` targets in the catalogue. `mythic_type.nephilim` looked live
and is not: it lists both Great Characteristics under **`required_virtues`**,
which the player *buys*, so they reach the validators normally.

**This is why it still matters.** `CLAUDE.md` makes the V/F catalogue a
data-only extension point — "Abilities and V/F are added by editing
`rules/core/*.json` … with zero code changes". A future House or type that
grants Puissant Ability is a JSON edit, and **nothing would fail**.

### Ruling, as taken on that evidence

**Make both validators grant-aware now.** (Norbert, 2026-09-21.)
`validate_ability_bonus_targets` and `validate_characteristic_delta_preconditions`
must read **effective** selections — bought *and* granted — rather than bought
only.

**What this obliges, and the trap to avoid.** The change must **not** re-import
what review finding B1 deliberately excluded. `validation/prereq.rs::PrereqCtx::build`
records that grants "must never reach" the incompatibility and trait checks, and
that exemption is correct: those ask *may this character hold this Virtue at
all*, and a House grant is precisely what makes him eligible. **These two
validators ask a different question** — *is the thing this Virtue modifies in a
legal state* — and a grant says nothing about that. Great Characteristic's "+3
already" is not an entitlement gate; it is a description of its operand.
Implement the distinction explicitly, and say so in a comment at both sites,
because the next reader will otherwise see two validators disagreeing with B1
and "fix" one of them.

Both validators currently carry **no** note either way, which is what left this
open for the whole audit. Whichever way a future slice moves them, the reasoning
goes in the code.

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

---

## D24 — The Gift is necessary but not sufficient for a Hermetic Virtue or Flaw

**Question (Q-140).** `flaw.weak_parens` and `virtue.skilled_parens` ship
`prerequisites: null` and `categories: ["hermetic"]`. The `hermetic` category is
not a magus gate: the **companion** profile permits it conditionally,
`{"category": "hermetic", "when": {"kind": "has", "value": "virtue.the_gift"}}`
(`rules/core/character_types.json:18`). So a Gifted, non-magus companion may
legally select Weak Parens — and what follows is not a refusal but a **silent
budget change**. `effective/xp.rs::general_pool_and_bonus` builds `base_general`
from `apprenticeship_xp + post_gauntlet_xp` **only when `is_magus`**, otherwise
from `later_life_xp`, and folds the −60 in unconditionally. A Gifted companion
who never had a *parens* loses 60 XP from his later life and pockets a Minor
Flaw's points for it.

**Ruling, in two parts.**

1. **Both entries gain `Prereq::IsMagus`.** The Flaw and its mirror Virtue are
   treated identically, even though `virtue.skilled_parens` lies outside B19's
   span and appears in no batch record.
2. **Holding The Gift does not, by itself, open the `hermetic` category.** The
   Gift is a *necessary* condition, not a sufficient one. The profile's
   Gift-conditional permission **stays**; the second gate is the per-entry
   `IsMagus` that **D12** puts on every *trained* Hermetic entry.

**Why this is not a new judgement.** ArMDE:2880 states both halves in one
sentence: *"Only characters with The Gift can take these Virtues and Flaws, **and
some are only applicable to Hermetic magi who have already completed their
training**."* Part 1 of the ruling is the second clause; the profile condition is
the first. And `ArMDE:7074` settles which side Weak Parens falls on — its stated
result, *"a total of 180 experience points and 90 levels of spells"*, is an
**apprenticeship** figure and appears nowhere else. A *parens* exists only
through apprenticeship, so both entries are **trained** under D12's criterion and
inherit D12's gate. Q-140 therefore does not need a ruling of its own about the
entries; it needed one about the *profile*, which is part 2.

**The alternative was rejected on the book.** Dropping the companion's
Gift-conditional permission outright contradicts **ArMDE:2840** — *"You may not
take Hermetic Virtues and Flaws, **unless you have The Gift** (this would be
highly unusual)"* — and would re-block `flaw.suppressed_gift`, which
`RULES.md` row 20 made reachable on exactly that line, with `ArMDE:6805`/`:6809`
confirming such a character does hold The Gift.

**What this obliges.**

- Add `"prerequisites": {"kind": "is_magus"}` to `flaw.weak_parens` and
  `virtue.skilled_parens`. **Blocked on D12**, which decides the same thing for
  the other 120 `hermetic` entries — doing these two alone would split one pass
  in half.
- Leave `character_types.json:18` and `:25` **unchanged**. The companion
  profile's `CategoryRule` is correct as shipped; anyone reading it as
  over-permissive must be pointed at this decision.
- `tests/data_integrity.rs::a_companion_does_not_gift_himself_with_a_hermetic_virtue`
  stays green and stays load-bearing — it pins the non-circularity of the leaf,
  which part 2 depends on.
- D12's classification pass acquires a **positive check**: after it runs, no
  entry may be reachable by a Gifted non-magus *and* compute against a budget
  that only exists for a magus. Weak Parens is the worked example; the pass must
  say whether it is the only one.

**Precedent this sets.** A profile's `permitted_categories` condition is a
*membership* gate, never a *capability* gate. When an entry inside a permitted
category needs a narrower audience, the narrowing belongs on the **entry** as a
`Prereq`, not on the profile — otherwise one entry's requirement silently
rewrites what the whole category means for that character type. See **D12** for
the criterion and **D21** for the machinery that lets a rule range over a
category at all.

---

## D25 — where the book disagrees with itself, the descriptor wins and the error is recorded

**Question (Q-141).** `flaw.weak_personality` ships `categories:
["personality"]`. Its descriptor agrees — ArMDE:7077 reads *Minor,
**Personality***, DE:7077 likewise, and the text is pure Personality (*"all
Personality Traits must be between +1 and −1"*). But the book's own catalogue
index puts it somewhere else: **ArMDE:5518 lists it under `### Story, Minor`**,
and it is **absent** from the Personality, Minor list, which runs …Temperate,
Weak-Willed. So on the narrow question the data is right and **the book is
internally inconsistent**. The cost is user-facing, and CLAUDE.md rates that up:
a reader working from the book's Story list filters the app by Story and the
entry is not there.

**Ruling.** `["personality"]` **stands unchanged**, and the disagreement is
**recorded as a known source erratum** rather than left implicit.

**Why the descriptor wins.** The entry's content is not story-driven in any
respect — it constrains Personality Traits, personality rolls and social
modifiers, and nothing about it generates a story hook. D6 and D7 already place
a rulebook's *substantive* statement above a listing that merely points at it;
this is the same precedence one level down, inside a single book. Adding `story`
alongside would assert a category the descriptor denies, and it is not free:
`story` carries a hard cap of 1 for the companion profile, so the entry would
silently consume a Story slot.

**What this obliges.**

1. **A short `known source errata` note in `crates/arm-rules/RULES.md`**, naming
   the entry, both citations (ArMDE:7077 descriptor, ArMDE:5518 index) and the
   side taken. RULES.md is the right home because it is already the place that
   explains why a data value is what it is; this is that, for a value whose
   justification is a disagreement. **No new file and no new register** — the
   audit does not need an errata *system*, it needs this fact to survive.
2. **Establish whether it is the only one.** B19 found it in 25 entries and
   called it the only descriptor/index disagreement *in that span*; nothing has
   checked the other 630. The index lists are heading-delimited (`### <Category>,
   <Magnitude>`) and mechanically readable, so a one-off sweep can compare every
   entry's `categories`/`magnitude` against the list that names it. Run it once
   as part of the erratum note and state the result — if there are more, they all
   belong in the same note.
3. **Do not derive anything from the index lists.** They are link lines, and the
   audit already proved link hygiene unreliable: the anchor sweep found **134**
   link targets resolving to no heading. The sweep above is a *comparison* whose
   output a human reads, never a guard and never a source of data.

**Precedent this sets.** A defect that turns out to be the book's, not ours, is
still *written down* — at the implementation site, with both citations and the
side taken. An undocumented correct-looking value is indistinguishable from an
unexamined one, and the next audit pays to re-derive it.

---

## D26 — a scanno is corrected in the source, not preserved

**Question (Q-142).** `flaw.waster_of_vis` ships one string spelling the same
word two ways: ArMDE:7054 reads *"one **guarter** (rounded up)"* and, six clauses
later, *"one **quarter** of those you use"*. Does the `scheitest` precedent (a
shipped string is a faithful copy of its source, typo included) cover it?

**Ruling: no — it is a scanno, and it is corrected in `rules/source/en/` as well
as the shipped strings.** DONE, not deferred.

**Scope, as fixed.** ArMDE:7054 and ArMDE:3502 (`virtue.berserk`, *"give
guarter"* for the idiom *give quarter*) — the second is what makes the `q`→`g`
OCR error systematic rather than a slip. Both German lines were already correct
(*Viertel*, *Gnade*), so no DE change. `rules/i18n/en/virtues_flaws.json` re-synced.

**Where the limit is.** `scheitest` still governs a **real** typo in the source
that reads unambiguously. A scanno is not that: it is the OCR's error, not the
book's, and here one string contradicted itself. Fixing i18n alone was rejected —
the JSON is generated from the Markdown, so a re-extraction would revert it.

**What this obliges.** A todo is filed in `arm-de-translation`
(`docs/todo.md`) — its `original-english/reviewed/` copy carries both, at its own
line numbers. `docs/open-todos.md` row 39 is closed for this; its other half
(`ArMDE:6929` reads `-3penalty`, no space) stays open and is **not** covered here.

---

## D27 — the spell list models spells *known*, so Rigid Magic warns rather than blocks

**Question (Q-130).** ArMDE:6697 gives Rigid Magic *"you cannot … **cast** Ritual
magic"*. `SpellDef::ritual` exists, so the engine could refuse the selection.
Does the spell list model spells **known** or spells **usable**?

**Ruling: known.** Selecting a Ritual stays **legal**; a Rigid Magic character
holding one raises an **advisory warning**, never an error.

**Why.** The verb is *cast*. A magus may have learned a Ritual before acquiring
the Flaw, and a Ritual he cannot cast is still worth holding — he can teach it or
copy it out. Blocking would enforce something the book does not say.

**This does not soften D16.** D16 governs absolutes the engine *does* model.
Here the absolute is about casting, which a character generator does not do at
all; the warning is a helpfulness signal about a selection the book permits, not
a downgraded prohibition.

**What this obliges.** The restriction reaches the player as text on the entry
(D20), plus one validation rule: holds `flaw.rigid_magic` **and** a `ritual`
spell → warning. Reachability per D2 — the Flaw may be *granted*, so the check
must not read bought selections only.

---

## D28 — the spell level cap becomes range-aware for Short-Ranged Magic

**Question (Q-133).** ArMDE:6739 halves the Lab Total *"when designing an effect
or spell that has a range greater than Touch, **including Eye**"*.
`spell_level_cap` is that gate, and the Flaw ships only
`special_casting_mod: circumstantial`, so the halving is unmodelled.

**Ruling: model it.** The cap becomes range-aware; a `flaw.short_ranged_magic`
character's cap halves for spells whose range is beyond Touch.

**Why, and why D1 does not decide it.** D1 refused nine *conditional* Lab Total
modifiers because their conditions (experimenting, working from a text, a season)
are not creation-time facts. This one **is**: the condition is the spell's own
`range`, which `rules/core/spells.json` already records.

**What this obliges.**

1. **Re-key the cap.** `spell_level_cap` and `spell_level_caps` are keyed by
   `(technique, form)` today, on the stated ground that a cap depends only on
   Te/Fo. That ground no longer holds — the key gains a beyond-Touch flag, and
   the UI picker that consumes the vector follows.
2. **The predicate is by name, not by ordering.** Halving applies to `eye`,
   `voice`, `sight`, `arcane_connection`; not to `personal`, `touch`. The book
   says "including Eye" precisely because Eye's position is not obvious — do not
   derive this from a magnitude ordering.
3. **It lowers a cap, so saves legal today can become invalid.** Same treatment
   as D10: validation reports it, `Enforced` blocks it, no migration and no
   schema change.
4. **Reclassify.** The entry is not only `in_play_effect` — half of it is a
   creation-time rule now, and the casting half still needs its D20 text.

---

## D29 — one resolution point for an Ability's maximum score

**Question (Q-133's sibling).** Should `flaw.savantism`'s *"may not begin with an
Ability above 3"* (ArMDE:6706) run through `AgeAbilityCaps` /
`validate_ability_age_cap`, or sit beside it?

**Ruling: through it — a single resolution point** that folds the age band with
any V/F override, the same discipline `categories_in_force` applies to a gate
read from two places.

**Why it is not a style preference.** Savantism's two caps pull opposite ways.
The general cap **lowers** to 3; the one favored Ability *"is limited to a score
of 6 as a starting character"*, and the age table caps a character under 30 at
**5** — so the favored cap **raises** above the age cap. A validator beside the
age rule can lower but never raise, so it cannot express the 6 at all.

**What this obliges.** `max_ability_score` stops being the answer and becomes an
input: the resolution point takes the ability and the entity, folds the age band
and the overrides, and both the validator and the UI read it. The favored Ability
must be *recorded* to be capped at 6 — that is D9's parameter work. F-510 stays
live and covers the rest of the entry (half XP, halved Advancement Totals, the
+3 specialty).

---

## D30 — `source.anchor` is mandatory catalogue-wide, and a range ends on content

**Question (Q-96, Q-124/Q-97).** Is `source.anchor` mandatory or dropped? And
where does a `source.lines` range end — on the blank line before the next
heading, or on the last non-blank body line? Both forms ship.

**Ruling.**

1. **`anchor` is mandatory everywhere**, not only for Virtues and Flaws.
2. **A range ends on the last non-blank body line.**

**Why.** D18's argument — a line number dies on a rulebook re-sync, a heading
anchor survives — does not distinguish a spell from a Virtue. And a range that
ends on content can be *checked* for content, which is exactly what the
rulebook-citation guard already requires of code comments; a range ending on a
blank line can never be.

**Scale, measured.** `virtues_flaws.json` is at 655/655 (plus 655 German
anchors and two heading-derived guards). Every other core file is at **zero**:
360 refs in `spells.json`, ~200 more across abilities, equipment, aging, houses,
childhoods, arts, mythic companion types and spell mastery — **563 to backfill**.
For the range form, the only measurement is B16's span: **32 of 35** use the
blank-line form, so the minority form wins and most rows change. Nobody has
measured the other 620.

**What this obliges.**

- Backfill the 563, then make the field **non-optional** so a new entry cannot
  ship without one. The order matters: the type change must land last.
- Normalise every range to the non-blank form. Data-only, and re-derivable from
  the anchors — extend `regenerate_source_anchors` (currently `#[ignore]`d)
  rather than hand-editing.
- Fix `RULES.md` and check 1's stated convention to agree; the disagreement
  between them is what let the mixture survive three batches.

**A repair here is FORBIDDEN, not deferred — and that is now a project
invariant.** The concurrent book-template session found nine source-conversion
defects in `rules/source/en/` (`open-todos.md` row 54). Eight are in-place
single-line edits and safe. The ninth, a dropped line break at **ArMDE:1277**
that swallowed the Hunter's *"Personality Traits:"* heading into the Virtues
line, could only be fixed by **inserting a line**.

**Norbert's ruling, 2026-09-24: the number of lines in a `rules/source/` file is
never changed. Never — not for a defect, not for formatting, not as part of
anything else.** It is now in `CLAUDE.md` beside the provenance rule.

The reason is the one D30 exists for: an inserted line renumbers everything below
it and invalidates **every** citation into that file at once — `SourceRef` ranges
in `rules/core/` and `rules/i18n/`, `// Source:` comments, `RULES.md`, and every
dated record under `docs/`. And **nothing detects it**: `rulebook_citations.rs`
checks only that a range lands on non-blank lines, so after a shift every
citation still *passes* while pointing at the wrong text.

So ArMDE:1277 is **left in place and recorded**. Its repair is not scheduled
work, and it is not "blocked on the anchor backfill" — an earlier draft of this
paragraph said so and was wrong. A whole-file re-sync from upstream is a
different act; a hand edit is what is barred.

---

## D31 — the table wins on a NAME; the rulebook still wins on a RULE

**Amends D7.1 and completes D18.** D7.1 gives the DE rulebook heading precedence
over the glossary. That is right only while the rulebook we hold is **current**,
and it is not: `rules/source/de/` is older than `arm-de-translation`'s tables,
and the shipped `rules/i18n/de/` was generated from it.

**The error this corrects was mine, and it is circular.** Every "DE rulebook
heading" in `tmp/table-sync-check.md` comes from the stale copy, so every
"table wrong" verdict uses the old edition to convict the newer table. The
Hobbled case is the clearest: the collision argument reads *both* names out of
the stale copy. **All 21 verdicts have to be re-derived.**

**Ruling.**

1. A disagreement about a **name** resolves **for the table** — it is the most
   current German text in the repository. Adopt it into `rules/i18n/de/`.
2. A disagreement about a **rule** — which Virtue grants which Reputation, at
   which magnitude, in which category — still resolves **for the rulebook**
   (D6.1, unchanged). A translation revision renames; it does not reassign
   mechanics.

**The seven already-applied corrections are reverted.** Of the ten proven table
errors, seven were name rows decided against the stale copy: Deteriorating Power,
Disorientating Magic, Enfeebled, Environmental Magic Condition, Environmental
Sensitivity (`tugenden-fehler.md`), Vulnerable Magic and Vulnerable to Folk
Tradition (`grundbegriffe.md`). We edited the **newer** source to match the
**older** rulebook, in both projects. Restore the table wording and adopt it into
`rules/i18n/de/`. The other three were factual claims and stand.

**What this obliges.**

- Re-derive `tmp/table-sync-check.md`, then adopt the table's name for the 19
  live name disagreements and revert the 7 above — in `arm-char-gen` and
  `arm-de-translation` alike (D6), including the README correction log.
- Do **not** re-sync the rulebook to settle this. When a current German rulebook
  does land, D7.1 returns to full force and this decision retires.

---

## D32 — EVERY Mythic Companion has 10 Flaw points and 20 Virtue points. No exceptions, no bonuses.

**This decision is closed. Do not reopen it, do not re-derive it, and do not
"discover" a bonus in ArMDE:2664 or RoP:I:4924 — both sentences are read and
disposed of below.**

### The numbers, for every one of the four types

| | Value | Source |
|---|---|---|
| Flaw points | **10** | ArMDE:2638, RoP:I:4912 |
| Budgeted Virtue points | **20** | 10 × the 2:1 rate |
| Free Minor Virtue | **1**, uncharged | ArMDE:2638 |
| Virtue points *stated as a total* | **21** | = 20 budgeted + the free Minor |
| `bonus_flaw_points` | **0** | — |
| `bonus_free_virtue_points` | **0** | — |

Devil Child, Faerie Doctor, Nephilim and Spirit Votary are **identical** on all of
these. A type's compulsory package changes how the allowance is *spent*, never how
large it is.

### The decisive citation

**RoP:I:4912**, writing about Devil Child specifically:

> "The Devil Child Virtue is a Free Virtue (like The Gift) which, **like other
> Mythic Companion characters**, grants the player two points to spend on Virtues
> for every point that she spends on Flaws. It also grants a free Minor Virtue,
> **allowing a maximum of 21 points of Virtues for 10 points of Flaws**."

Twelve lines later, **RoP:I:4924** gives the disputed sentence — *"three more
points of Virtues at no cost … an additional seven points of Flaws"* — **inside
that stated maximum, in the same book, about the same character type.** Neither
number can therefore extend the cap. Both describe how the 21/10 is reached.

### The book's own worked example agrees

Malachi (RoP:I:4942) carries False Power (Major 3), Tragic Life (Major 3), Lesser
Malediction (Minor 1), Delusion (Minor 1), Proud (Minor 1), Tainted with Evil
(Minor 1) — **exactly 10 Flaw points**, not 17.

**His Virtue side proves nothing in either direction, and must not be cited.**
ArMDE:1163 fixes the order of a template's V/F line — Gift, Social Class, Major
Virtues, Minor Virtues, Major Flaws, Minor Flaws — and Malachi's semicolon groups
follow it, which places **Greater Immunity inside the Minor-Virtue group** while
ArMDE:4010 calls it *Major, Supernatural*. Read by the grouping he is one point
**under** budget; read by our magnitudes, one point **over**. The example is
internally inconsistent there, so neither number is evidence. An earlier draft of
this decision cited the "one point over" reading; that was an artefact of ignoring
ArMDE:1163 and is withdrawn. **The Flaw count above is independent of all this and
is what this decision rests on.**

### Rejected readings — each of these has been tried and is wrong

1. **"`an additional seven points of Flaws` is a bonus on top of the ten."**
   No. ArMDE:2638 and RoP:I:4912 both cap Flaws at ten, and the compulsory Major
   Flaw is 3 of them, so 3 + 7 = 10. This is the misreading that shipped.
2. **"`three more points of Virtues at no cost` is a grant on top of the twenty."**
   No. RoP:I:4912 states the 21-point maximum *before* the sentence that says it.
3. **"The 21-vs-20 in ArMDE:2638 is a contradiction to be resolved."**
   It is not a contradiction. **21 = 20 budgeted + the free Minor Virtue.** They
   are two different quantities in the same sentence.
4. **"Both readings give 19 free Virtue points, so the 3 is harmless."**
   That argument ignores that a cap is *stated*. It was made in this repository
   on 2026-09-22 and withdrawn the same day.
5. **"Spirit Votary's 7 is the unspent remainder, so it belongs in the data."**
   The remainder is already inside the 10; adding it to the 10 counts it twice.
   `RULES.md:3850-3857` derives it correctly and then encodes it wrongly.

### `RULES.md:3840-3857` is WITHDRAWN

It states *"Verified maxed budget: flaw 17, virtue 20 + 14 + 3 = 37"* as settled
fact. **No such decision was ever taken.** It is an earlier session's misreading
written up as a ruling, and it survived because nothing re-derived it. It also
contradicts itself: Devil Child's 7 sits *on top of* the 10 there, while Spirit
Votary's is derived as *"`10 − 3 = 7` points of the Flaw allowance **unspent**"* —
inside the 10 — and both ship the same value.

### What this obliges

- Set both `bonus_flaw_points` and `bonus_free_virtue_points` to **0** for every
  type in `rules/core/mythic_companion_types.json`; then **delete both fields
  from the model** unless something else needs them, so the mistake cannot be
  re-entered as data.
- **Pin it with a test.** A ruleset-level assertion that every mythic type yields
  a 10/20 effective budget is what stops this coming back a third time; a document
  alone has already failed once.
- Rewrite `RULES.md:3840-3857` — the table row, both derivations, and the
  "20-vs-21 inconsistency" note, which is itself wrong (see rejected reading 3).
- **Precedent:** a number in `RULES.md` presented as *verified* is not evidence
  that anyone verified it. Re-derive from the book, and prefer the supplement that
  states a cap over a core-book sentence that only describes a package.
