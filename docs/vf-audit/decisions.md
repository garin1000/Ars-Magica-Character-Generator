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
