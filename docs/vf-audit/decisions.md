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
