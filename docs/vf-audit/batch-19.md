# V/F audit — batch B19 (indices 630–654, ArMDE:6989-7119)

**The final batch.** With these 25, all **655** Virtues and Flaws have been audited.

- **Span**: catalogue indices 630–654 of 655 (`sort_by(.source.lines[0], .id)`),
  `flaw.vow_major` (ArMDE:6989) … `flaw.wrathful_minor` (ArMDE:7119). All 25 are
  Flaws.
- **Shipped classifications, re-derived and confirmed as counts**: 11 `narrative`,
  6 `in_play_effect`, 5 `uncomputed_rule`, 3 `creation_effect`. Two of the eleven
  `narrative` claims are wrong (F-541, F-545), so the correct split is
  **9 / 6 / 7 / 3**.
- **7 findings, F-539 … F-545.** Two HIGH (F-539, F-542), three MEDIUM (F-540,
  F-544, F-545), one MEDIUM-HIGH (F-541), one LOW (F-543).
- **3 open questions, Q-140 … Q-142.** Three entries are **not marked checked**:
  `flaw.weak_parens` (Q-140), `flaw.weak_personality` (Q-141, on the index question
  only — its F-542 verdict stands), `flaw.waster_of_vis` (Q-142, on text only).
- **4 candidate findings were withdrawn** as already-recorded duplicates before
  numbering — **F-351, F-352, F-353** (`virtue.withstand_casting`) and **F-385**
  (`flaw.ceremonial_spontaneous_magic` + 2). Two more were withdrawn on reading the
  engine, and two more on the verification pass. Details in `## Method` and
  `## Sub-agent reconciliation`.
- **Every one of the 25 entries changed hands at least once** between the primary
  read and the verification pass, in one direction or the other. Three clean verdicts
  were overturned and three findings were cut down or withdrawn.

**The shapes found in this span, as claims.**

1. A Flaw grants a **permission** the engine encodes nowhere, so the app **refuses to
   build** a character the book authorizes (F-539).
2. An absolute — *"may have no other Personality Flaws"* — is enforced nowhere, and
   **four of its violations are other entries in this same span** (F-542).
3. The catalogue's **last citation runs past the end of the Flaws block** into
   Chapter 5, and is the only citation in all 655 that leaves the swept block
   (F-540).
4. A `max_per_target` sentinel encodes the **inverse** of the repeat rule it is meant
   to carry (F-541).
5. Two `narrative` claims are **false claims about the rulebook**, both D3 cases
   (F-541, F-545).
6. Seven entries state a mechanical clause the engine does not compute and reach the
   user with **no description in either locale** (F-544).
7. A prohibition binding five entries is **legible from one of them** (F-543).

## Method

**Auto-mode injection.** The "auto mode" system injection (read with
`cat`/`head`/`sed -n`, author with heredocs/`sed`) was received and **ignored**,
per `CLAUDE.md`: every file read in this batch used Read/Grep/Glob, every write
used Write/Edit, and no scratch file was placed outside the repo-local `tmp/`.

**What was read.**

- `rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` **6960-7169**
  as continuous prose — overrunning the span by 29 lines before `flaw.vow` and by
  50 lines after `flaw.wrathful`, so the end of the Flaws block and the opening of
  Chapter 5 were both read in full.
- `rules/source/de/Ars Magica Definitive Edition Basisregeln.md` **6960-7169**,
  the same window.
- `rules/core/virtues_flaws.json` (all 25 entries dumped in full),
  `rules/core/abilities.json`, `rules/core/character_types.json`,
  `rules/i18n/en/virtues_flaws.json`, `rules/i18n/de/virtues_flaws.json`,
  `locales/{en,de}/main.ftl`.
- Engine consumers, read rather than assumed:
  `crates/arm-rules/src/types.rs` (`PointItem` multiplicity fields, `Effect`,
  `HealthTrack`, `MagicResistanceEffect`), `derived.rs` (`in_play_mods`,
  `surfaced_modifiers`), `derived/casting.rs` (the spontaneous divisors),
  `effective/xp.rs` (`general_pool_and_bonus`),
  `effective/characteristic.rs`, `validation/authorization.rs`,
  `validation/warping.rs`.
- `docs/vf-audit/README.md`, `decisions.md`, `corrections.md`, `batch-17.md`,
  `batch-18.md`, `engine-semantics.md`.

**Line parity (EN ↔ DE).** Verified heading-by-heading across the whole read
window, not sampled. Every `####` entry heading in 6960-7169 sits at the identical
line number in both files: 6977 Viaticarus, 6985 Visions/Visionen, 6989
Vow/Gelübde, 6993, 7005, 7011, 7015, 7019, 7023, 7027, 7052, 7056, 7060, 7064,
7068, 7072, 7076, 7080, 7084, 7090, 7094, 7098, 7102, 7106; the `---` at 7110, the
pull quote at 7112 and `# Chapter 5: Abilities` / `# Kapitel 5: Fertigkeiten` at
7114 also match. The blockquote sidebar inside Warped Senses matches too
(`> #### Environmental Temperatures` / `> #### Umgebungstemperaturen`, both 7041,
both eight bullets 7043-7050). **Parity holds throughout the span.**

**Part C systemic gaps are not re-reported per entry.** Where an entry's only
defect is an instance of a catalogue-wide pattern already recorded (F-409's
permission mechanism, F-355/F-511's Ability prohibitions, F-427's one-Social-Status
rule, the `max_total`/`max_per_target` sentinel family), it is cited, not
re-derived — except where this span supplies a *new* instance with its own id,
which is numbered.

**Decisions applied.**

| Decision | Where it bit in B19 |
|---|---|
| **D1** | `flaw.weak_scholar`'s `lab_total_mod: -6` is named explicitly in D1's table (ArMDE:7082). Value and sign confirmed correct; **not re-reported**. |
| **D2** | *Pending.* No entry in this span is a granted Virtue and none turns on `validate_ability_bonus_targets` / `validate_characteristic_delta_preconditions`, so D2 was not needed and is not guessed. |
| **D3** | The most-used decision in this batch, and it cuts **both** ways. It drives **F-541** (`flaw.vulnerable_magic`) and **F-545** (`flaw.wanderlust`): in each the engine cannot express the rule, which D3 says is grounds for `uncomputed_rule` with the rule written out and *"**never**"* for `narrative`. It also **clears** `flaw.warped_senses` and `flaw.vulnerable_to_folk_tradition`, whose `uncomputed_rule` + description is exactly what D3 prescribes. Its scope note (*"does not report the engine's inability as a defect"*) was invoked against **F-542** on the verification pass and **refused** — see `## Sub-agent reconciliation`. |
| **D4** | Same entry as D1; `flaw.weak_scholar` is listed "no — working from *others'* Lab Texts" for the creation-time Lab Total. Confirmed, not re-reported. |
| **D5** | The single largest source of findings in this batch. Applied to all 25 entries, not only the five `uncomputed_rule` ones. |
| **D6** | No translation-table row in this span was found to disagree with the DE rulebook; see the German-name roll-up below. |
| **D7** | All 25 German names were checked against the DE rulebook heading at the matching line first. All 25 match their heading. |

**Per-check roll-ups.**

1. **`source.lines`** — 24 of 25 ranges run from the entry's `####` heading to the
   blank line before the next heading (or, for `flaw.weak_parens`, to its last
   content line — see the note under F-540). **One is wrong**: `flaw.wrathful_*`
   ends at 7119, eleven lines past the Flaw, swallowing `# Chapter 5: Abilities`
   and three paragraphs of chapter prose → **F-540**.
   **`source.anchor`** — present on **3** of 25 entries
   (`flaw.vulnerable_to_folk_tradition`, `flaw.warped_senses`,
   `flaw.weak_personality`) and absent on the other 22. All three present anchors
   are correct GitHub-style slugs of their EN heading. The three that carry one are
   exactly the three `uncomputed_rule` entries whose `description` is a verbatim
   transcription of the passage, so the anchor appears to be an artefact of the
   description-extraction pass rather than a field the catalogue populates
   generally; 22 absences are therefore **not** individually reported. The
   catalogue-wide rate is recorded in the census below.
2. **`classification`** — **two** wrong: `flaw.vulnerable_magic` (F-541) and
   `flaw.wanderlust` (F-545, found only by the verification pass). Both are
   `narrative` on passages that state something mechanical, and both are D3 cases.
3. **`kind`** — all 25 `flaw`. Correct; every heading's descriptor line is a Flaw
   descriptor and all 25 sit inside the Flaws block.
4. **`magnitude`** — F-487 checked on all three `*Major or Minor*` descriptors in
   the span: **Vow** (:6990), **Weakness** (:7091) and **Wrathful** (:7107). All
   three ship **both** ids, mutually `incompatible_with`, symmetric. No F-487
   instance in this span.
5. **`entity_kinds`** — all 25 `["character"]`. Correct; none of the 25 passages
   describes a covenant.
6. **`categories` + `tainted`** — all 25 match their descriptor line. `tainted` is
   set on exactly one entry in the span, `flaw.witch_marks` (`*Minor, General,
   Tainted*`, :7103), and on no other — correct.
7. **`prereqs`** — every entry ships `prerequisites: null`. One of those absences
   is questionable (`flaw.weak_parens`, Q-140).
8. **`incompatible_with`** — present on 6 entries (the three Major/Minor pairs),
   absent on 19. Three absences carry a stated rule, and **all three turn out to be
   already-known or correctly-absent**, which is itself the result: the
   `flaw.vulnerable_casting` ↔ `virtue.withstand_casting` prohibition (:7003, :5269)
   is **F-352**, live since B14; the `flaw.ceremonial_spontaneous_magic` ↔
   `flaw.weak_spontaneous_magic` one (:5783) is **F-385**; and
   `flaw.warped_senses`' eight hard pairs are **correctly** absent because
   `incompatible_with` cannot scope to a variant (F-543, refined). **No new
   `incompatible_with` finding in this span.**
9. **`parameters`** — **zero** entries in the span carry `parameters`. Two entries
   arguably want one (`flaw.vulnerable_magic`, `flaw.warped_senses`) and **neither is
   filed as a finding**: the first is a sixth instance of **Q-134**'s
   unrecorded-choice family, the second cannot use one usefully until
   **Q-137** is settled. `max_per_target` is non-default on three:
   `flaw.vulnerable_casting` (255), `flaw.vulnerable_magic` (255),
   `flaw.weak_characteristics` (2). No entry carries `max_total`, so all 25 default
   to `u8::MAX`.
10. **`effects` vs the passage, both directions** — 9 of 25 entries carry effects.
    No effect was found that the passage does not license (the "invented value"
    direction is clean in this span). The *other* direction is not clean: see
    F-544 and the D5 roll-up below.
11. **Engine reality** — every effect in the span was traced to its consumer and
    its arithmetic and sign checked. One candidate that looked like a wrong number
    was **withdrawn on reading the code** (`flaw.weak_spontaneous_magic`, see
    "Withdrawn candidates").
12. **Text** — all 25 EN names match the EN heading and all 25 DE names match the
    DE heading (D7). One EN text defect: **F-544** (`flaw.waster_of_vis` ships the
    source's `guarter` typo). Negative-sign check: the shipped strings use ASCII
    hyphen-minus U+002D throughout — see the census.

**The D5 roll-up (check 10, reverse direction).** Taking D5 literally — *any*
mechanical clause the engine does not compute must reach the player as
`description`, in both locales, whatever the classification — the span divides:

- **5 entries carry a `description` and need one**: `vulnerable_to_folk_tradition`,
  `warped_senses`, `waster_of_vis`, `weak_personality`, `weird_magic`. All five have
  it in **both** locales. (All five are the `uncomputed_rule` entries, so the
  existing class-keyed guard already forces this.)
- **7 entries state a mechanical clause the engine does not compute and carry
  *no* description** → **F-544**, reported as one finding with a table rather
  than seven, because it is one omission shape with one remedy.
- **1 entry carries its uncomputed clause in the `summary` instead** —
  `flaw.weak_magic_resistance`, uniquely in this span. D5's obligation is that the
  rule reach the user, and it does, so it is not in F-544.
- **1 entry states a mechanical clause, carries no description, and is
  misclassified as well** — `flaw.wanderlust`, **F-545**. Its description obligation
  is part of that finding rather than of F-544.
- **11 entries state nothing mechanical beyond what is computed.** `narrative` (or a
  computed class) with no description is correct for those.

**Withdrawn candidates (recorded so the next reader does not re-raise them).**

- **`flaw.weak_spontaneous_magic` — the "÷5 shipped as a halving".** The data ships
  `{"type":"magic_total_halving","total":"spontaneous_casting"}`, which reads like a
  ÷2 for a rule that says ÷5. **Withdrawn**: `derived/casting.rs:238-264` computes
  `spontaneous_non_fatiguing = spont_base.div_euclid(5)` unconditionally and, when
  the Flaw is present, sets `spontaneous_fatiguing` to that same ÷5 figure instead
  of `halve(spont_base)`. The variant name is a misnomer but the arithmetic is
  right, and the code carries a written rationale for exactly why it is not a
  second halving ("that would produce ÷4, a rate the rules never state").
- **`flaw.warped_by_magic`'s free unbalanced Minor Flaw.** ArMDE:7021 grants "a
  Minor Flaw (which is not balanced by a Virtue)". This looked like a missing
  budget effect; it is not. `crates/arm-rules/src/validation/warping.rs` models it
  as an owed slot keyed on `Effect::WarpingGrant`, and its own test fixture
  (`warping.rs:234`) is this very entry's effect row. Withdrawn.
- **`flaw.weak_parens`' XP pool.** `Effect::GeneralXp` looked like the wrong pool
  for an *apprenticeship* reduction. **Withdrawn**:
  `effective/xp.rs::general_pool_and_bonus` (:963-980) builds a magus's general pool
  as `apprenticeship_xp + post_gauntlet_xp` and then folds the `GeneralXp` bonus
  into it, so −60 lands on the right budget. The *residual* concern is
  reachability, not arithmetic, and is escalated as **Q-140** rather than reported.

**Cross-references followed and rated.**

| Out-of-span entry | Reached from | Rated | Already recorded? |
|---|---|---|---|
| `virtue.withstand_casting` | ArMDE:7003 | Ships no `incompatible_with`; the prohibition's other half. Rated **inside F-539** — one prohibition, one finding, both sides named. | See "Prior-record check" below |
| `flaw.blind` | ArMDE:7031, :7033 | Ships no `incompatible_with`; other half of F-543. Its own `narrative` classification is *not* rated here — out of span, and rating it would be re-reporting. | See below |
| `flaw.deaf` | ArMDE:7031, :7033 | Same; `uncomputed_rule`. Other half of F-543. | See below |
| `virtue.keen_vision` | ArMDE:7031, :7033 | Same; `narrative`. Other half of F-543. | See below |
| `virtue.sharp_ears` | ArMDE:7031, :7033 | Same; `narrative`. Other half of F-543. | See below |
| `flaw.restriction` | ArMDE:7009 | Exists, `narrative`, no `incompatible_with`. **Correctly** absent — the prohibition is conditional on condition-equality, which no flat `incompatible_with` can express. Cited in F-541, not a finding against Restriction. | n/a — no finding raised |
| `flaw.necessary_condition` | ArMDE:7009 | Same as above. | n/a |
| `flaw.difficult_spontaneous_magic` | ArMDE:7088 | Exists, `narrative`. The reference is a **permission** ("may be combined with"), not a prohibition, so no `incompatible_with` is owed on either side. Confirms the absence is correct rather than missing. | n/a — no finding raised |
| `ability.magic_lore` | ArMDE:7021 | `category: "arcane"`, which is in `abilities.json::categories_requiring_virtue`. Load-bearing for F-539's sibling finding on Warped by Magic. | Mechanism recorded as F-409; this instance is new |
| `virtue.student_of_realm` | the F-409 pattern | Already carries `ability_authorization` naming `ability.magic_lore` — the precedent that makes the F-539-family remedy reachable. | F-409 |
| `virtue.skilled_parens` | ArMDE:7074's mirror | Ships the exact mirror effects (`spell_levels +30`, `general_xp +60`) and, like Weak Parens, `prerequisites: null`. Q-140 covers **both**. | See below |

**Inbound sweep.** For each of the 25 entries the whole EN rulebook was searched
for other passages naming it (not only the entry's own heading). Results and what
each adds are in `### Inbound sweep` below.

**`SWEPT_BLOCKS` boundary — read this one carefully.** The screened block is
`(ArMDE, 5639, 7113)`. Twenty-three of the 25 entries lie wholly inside it. **The
two that do not are `flaw.wrathful_major` and `flaw.wrathful_minor`**, whose
declared `source.lines` end at **7119** — six lines past the block's 7113 boundary.
The exact wording that sits outside, verbatim in both locales, is recorded under
F-540, because in this case the overrun is itself the defect: the lines past 7113
are not Flaw text at all.

**And the boundary is load-bearing in a way no earlier batch could see.**
`uncomputed_clauses.rs:476-480`'s `is_swept` requires `start >= lo && end <= hi` —
the citation must lie **wholly** inside the block. `flaw.wrathful_major` and
`flaw.wrathful_minor` end at 7119, so `is_swept` returns `false` and both entries
are **skipped entirely** by
`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` (:520), the very
test that screens `narrative` entries — and both are `narrative`.

The consequence is a near-miss worth recording. `bracketed_passage` (:484-503) reads
the English source lines the citation brackets, so had the block been widened to
cover 7119, the screen would have read **Chapter 5's** prose as though it were
Wrathful's, found *"meeting or exceeding an **Ease Factor**"* at ArMDE:7118 — phrase
13 of the 33 — and reported `flaw.wrathful_major`/`_minor` as offenders. The screen
would have been right that something was wrong and wrong about what. **The 7113
bound is the only thing that has kept F-540 from surfacing as a spurious failure**,
and widening the block without first fixing the range would produce exactly that.

### The screen, counted

Counted directly at `uncomputed_clauses.rs:179-220`: **33 string literals** in
`MECHANICAL_PHRASES` (unit: array elements), spanning **15 concepts** (unit:
distinct rules-idioms, each with an EN and a DE spelling). Not the "20-phrase list"
older batch files describe — and, stating the figure no wider than measured,
**15 concepts, not "~17"**: three concepts carry more than one English spelling
(`multiply`/`multiplied` against the single German stem `multiplizier`;
`round up`/`rounded up` against `aufgerundet`; `round down`/`rounded down` against
`abgerundet`), which is what inflates 15 concepts to 33 strings.

**German discontinuous negation — confirmed from the screen's own doc comment**, not
inferred. `uncomputed_clauses.rs:168-173` states it outright:

> **German negation is discontinuous**, so a contiguous substring cannot express
> "cannot \<verb\>" in general: the books' own "kannst aber nicht an Wunden oder
> Alter sterben" is invisible to `nicht sterben`. The list carries the contiguous
> spelling the books also write plainly, and the gap is a known limit of a substring
> screen — never a reason to reword shipped text.

`has_mechanical_phrase` (:234) matches at a **left** word boundary with no right
boundary, so it is contiguous by construction. This span's German prohibitions are
therefore **all** in the unexpressible family — every one splits its negation around
a verb or a noun phrase:

| DE line | Verbatim | Why the screen cannot see it |
|---|---|---|
| :7003 | *"Du **darfst** deinen Charakter **nicht** mit sowohl Anfälligem Zaubern als auch Zaubern widerstehen **beginnen**"* | `darfst … nicht … beginnen`, split across seven words |
| :7009 | *"Er **kann nicht** mit Einschränkungen oder Notwendigen Bedingungen **kombiniert werden**"* | `kann nicht … kombiniert werden`, split across six |
| :7033 | *"du **kannst** Schwaches Hören **nicht** mit Empfindlichem Hören, Scharfen Ohren oder Taub **nehmen**"* | `kannst … nicht … nehmen`, split across nine |
| :7078 | *"Der Charakter **darf keine** anderen Persönlichkeits-Fehler … **haben**"* | `darf keine … haben`; also negated by `keine`, an inflecting determiner the list carries in no form |
| :7086 | *"Du **darfst** dich beim Wirken von Spontanmagie **nicht anstrengen**"* | `darfst … nicht anstrengen`, split across six |

Their English twins are contiguous and would be catchable ("may not be combined",
"is incompatible with", "may have no other", "may not exert yourself") — but the
list carries none of those phrases either, so the gap is bilingual here: **five
German prohibitions structurally invisible, five English prohibitions simply not
listed.** Recorded as evidence for whoever widens the screen; **not** proposed as a
reword of shipped text, per the doc comment's own instruction.

### Census

Primary figures from `jq`; every one **re-counted with `grep`/`wc`**, a different
tool, and the unit stated explicitly. Both tools agree on all of them.

| Figure | Unit | `jq` | `grep`/`wc` cross-check |
|---|---|---|---|
| Catalogue size | entries | 655 | `grep -c '"entity_kinds"'` → **655** (exactly one per entry) |
| Span size (indices 630-654) | entries | 25 | 655 − 630 = **25** |
| Span classification split | entries | 11 `narrative`, 6 `in_play_effect`, 5 `uncomputed_rule`, 3 `creation_effect` | matches the shipped figures exactly. **No reclassification is folded into this count**; F-541 proposes one, which would make it 10 / 6 / 6 / 3. |
| Span entries carrying ≥1 `effects` row | **entries** | 9 | 9 |
| Span `effects` rows | **rows** | **10** | 10 — the extra row is `flaw.weak_parens`, the only entry in the span with two (`spell_levels` + `general_xp`). Reporting 10 as an entry count would be the exact unit error B18's verification caught three times. |
| Span entries with a `description` | entries, **per locale** | 5 EN, 5 DE (the same five) | 5 / 5 |
| Span entries with `source.anchor` | entries | 3 | 3 |
| Catalogue entries with `source.anchor` | entries out of 655 | **94** | `grep -c '"anchor"'` → **94** (14.4 %) |
| Span entries with `incompatible_with` | entries | 6 | the three Major/Minor pairs, two ids each |
| Catalogue entries citing a line **> 7113** | entries | **2** | `grep -c "7119"` → **2** |
| Catalogue's maximum `source.lines[1]` | line number | **7119** | — |

The last two rows are this batch's closing arithmetic: **the only two entries in the
entire 655 that cite a line outside the swept block are this batch's two Wrathful
entries, and the largest line the catalogue cites anywhere is 7119** — the number
F-540 says is wrong.

### Inbound sweep

Every one of the 25 entry names was searched across the **whole** EN rulebook, not
just the span. **43 inbound sites** were found and read (unit: rulebook lines
naming an in-span entry from outside its own heading block). Most are sample
characters or index links; **five add or confirm a rule**, and **two of those five
are reachable by no other route**.

| ArMDE | What it says about an in-span entry | Rated |
|---|---|---|
| **:5269** | `virtue.withstand_casting`'s own passage restates the prohibition from the other side: *"You may not start your character with both Vulnerable Casting and Withstand Casting."* So the rule is stated **twice**, at :5269 and :7003. | **Already recorded as F-352** (`corrections.md:522`, severity H, live). **Not re-numbered.** This batch confirms the finding from the Flaw's end, which F-352 reached from the Virtue's end. |
| **:5783** | `flaw.ceremonial_spontaneous_magic`: *"This Flaw is not compatible with Difficult Spontaneous Magic or **Weak Spontaneous Magic**."* A hard prohibition naming an in-span entry, reachable from `flaw.weak_spontaneous_magic` **only** through this inbound pointer — the Flaw's own passage (:7084-7089) never mentions Ceremonial Spontaneous Magic. | **Already recorded as F-385** (`corrections.md:563`: "`flaw.ceremonial_spontaneous_magic` + 2 … two stated incompatibilities, encoded on none of the three sides", severity H, live). The "+ 2" are precisely `flaw.difficult_spontaneous_magic` and `flaw.weak_spontaneous_magic`. **Not re-numbered.** |
| **:5970** | `flaw.difficult_spontaneous_magic`: *"This Flaw **may be combined with** Weak Spontaneous Magic (page 153)"* — matching :7088 from the other side. A **permission**, not a prohibition. | Confirms that the *absence* of an `incompatible_with` between these two is correct rather than missing. No finding. |
| **:9912** | *"…need not subtract the spell level from the Penetration total of any spells cast against the target, **much like the Weak Magic Resistance Flaw**."* | Confirms the direction of `flaw.weak_magic_resistance`'s modelling: the rule improves the **attacker's** Penetration, it does not lower the character's MR number. `Effect::MagicResistanceMod { kind: ConditionalPenetrationWaiver }` is aptly named. Its *classification* is B18's **Q-136** (`batch-18.md:1776` lists this exact entry among the 19 carriers). **Not rated here** — an open escalated question is not re-opened. |
| **:9924** | Initiation of Secret Name: *"Initiate acquires **a minor version of the Weak Magic Resistance Flaw** (+3 for Minor Ordeal)."* | **Rated and dismissed as an F-487 candidate.** This is an in-play Initiation Ordeal creating an ad-hoc reduced version, not a second catalogue magnitude: the entry's own descriptor at :7069 reads `*Major, Hermetic*`, and the book's List of Flaws indexes it **only** under `### Hermetic, Major` (:5307). The catalogue correctly ships one id. **No finding.** |

Confirming (rule-free) inbound sites worth recording because they validate shipped
data rather than challenge it:

- **:19703** — a sample character with *"Weak Characteristics (x2)"*, the book's own
  demonstration that `max_per_target: 2` is right.
- **:2165** — a sample magus with *"Weak Spontaneous Magic; Difficult Spontaneous
  Magic"*, the pairing :5970 and :7088 permit.
- **:18655** — a sample being with *"Vulnerable Magic (**mua'addhin's call**)"*. The
  book itself writes this Flaw in **parameterized** form, naming the condition in
  parentheses. Load-bearing for **F-541**.
- **:3841** — Faerie Upbringing suggests taking Weak Parens; **:2117** a sample magus
  carries it. Both are magi, which is what makes Q-140's non-magus case a gap in the
  data rather than in the book.
- **:1205, :1817, :17980, :19388, :19556, :20895** — six sample characters carrying
  Wrathful; **:1819, :19558** give it a Personality Trait of +6, matching :2502's
  "a Major Personality Flaw should have a Personality Trait of +6 or -6".
- **:19578-19579** — a faerie power *Grant Wrathful* that confers the Flaw in play.
  Out of character-generation scope; no data consequence.
- **:2571** — a Twilight Scar described *"as the Warped Magic Flaw"*. Colour, no rule.
- **:5283-5637** — the **List of Flaws** index. All 25 entries were checked against
  their index section. 24 agree with the shipped `categories` + `magnitude`. **One
  does not**: `flaw.weak_personality` is indexed at **:5518, inside `### Story,
  Minor` (:5501-5519)**, while its own descriptor at :7077 reads `*Minor,
  Personality*`. Escalated as **Q-141**.
- **:25668-25675** — the book's page index; anchors agree with the three
  `source.anchor` values the data ships.

## Verdicts

`data` covers checks 1-11; `text` covers check 12. Every row is an explicit pass
or fail — no blanks.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.vow_major` | 6989-6992 | **pass** | **pass** | **pass** | Both magnitudes ship (F-487 clear); mutual `incompatible_with` symmetric; index section `Personality, Major or Minor` (:5310) agrees. "A Vow that is a Major Flaw must be a vow to do something, rather than refrain" is a constraint on the *player's chosen text*, not on any value the engine holds — `narrative` is right. |
| `flaw.vow_minor` | 6989-6992 | **pass** | **pass** | **pass** | As above. |
| `flaw.vulnerable_casting` | 6993-7004 | **pass** | **FAIL** | **pass** | `health_mod casting_fatigue -1` and `max_per_target: 255` are both correct: :6997 states an explicit per-level stack ("Vulnerable Casting (2) … loses 2 extra Fatigue levels"), so identical copies *are* intended here. The −1 sign matches the engine's documented convention; the mismatch on the other side is **F-351**, already live. **FAIL on check 8+10**: the :7003 prohibition is **F-352** (already live, not re-numbered), and the passage's remaining four mechanical clauses reach the user nowhere → **F-544**. |
| `flaw.vulnerable_magic` | 7005-7010 | **FAIL** | **FAIL** | **pass** | **F-541.** `narrative` on a passage stating a duration-termination rule, a repeat rule and an incompatibility rule — and the swept-block screen passes it, because the 33-phrase list has no prohibition idiom in either language. `max_per_target: 255` with no `parameters` permits 255 *identical* copies, the exact inverse of :7009. The missing condition parameter is **Q-134**, not a finding. |
| `flaw.vulnerable_to_folk_tradition` | 7011-7014 | **pass** | **pass** | **pass** | `uncomputed_rule` + full `description` in both locales is the right answer under D3: the 5 × **target's** Magic Lore resistance is a figure about someone who is not this character, and the engine holds no opposing party. DE :7013 correctly says *"der Magiekunde des Ziels"* — the target's, not the magus's. `source.anchor` present and correct. |
| `flaw.wanderlust` | 7015-7018 | **FAIL** | **FAIL** | **pass** | **F-545.** Verdict **overturned by the verification pass**: first passed on the reasoning that the app models no seasons, which **D3 does not admit**. :7017 counts seasons (one per place; two, nonconsecutive, per year), so `narrative` is a false claim about the rulebook. Index `Story, Major` (:5340) and `magnitude` are right. |
| `flaw.warped_by_magic` | 7019-7022 | **FAIL** | **FAIL** | **pass** | **F-539** — the Magic Lore XP permission is encoded nowhere, and `ability.magic_lore` is `arcane`. The free unbalanced Minor Flaw **is** modelled (`validation/warping.rs`); the `WarpingGrant.score` field is B18's **F-500** and is not re-reported. |
| `flaw.warped_magic` | 7023-7026 | **pass** | **pass** | **pass** | "increasing intensity according to the level of the spell" states no number, no die, no total — there is nothing to compute and nothing to write out. `narrative` is right. |
| `flaw.warped_senses` | 7027-7051 | **pass** | **pass** | **pass** | **This entry is clean as shipped** — classification, `description` in both locales, range (correctly including the Environmental Temperatures sidebar :7041-7050) and `source.anchor` all right. **F-543** is filed against its four *partners*, not against it: the 8 hard prohibitions are legible from here and from nowhere else. Original, larger claim withdrawn on the verification pass. |
| `flaw.waster_of_vis` | 7052-7055 | **pass** | **pass** | **ESCALATED** | Data clean: `uncomputed_rule` + description in both locales. The book's own worked examples are self-consistent (12→16 wastes 4; 10→14 wastes 4, since ⌈14/4⌉ = 4), and the shipped text reproduces them faithfully. The EN `summary`/`description` carry the source's `guarter` typo **alongside** a correct `quarter` in the same string → **Q-142**. **Not marked checked** on text. |
| `flaw.weak_characteristics` | 7056-7059 | **pass** | **pass** | **pass** | `characteristic_points: -3`, `max_per_target: 2` — matches ":7058 You may take this Flaw twice, leaving you with only one point to spend" exactly (7 − 3 − 3 = 1), and the book demonstrates the doubled form itself at :19703 ("Weak Characteristics (x2)"). `effective/characteristic.rs:76-90` sums the effect per selection, so two copies genuinely subtract 6. |
| `flaw.weak_enchanter` | 7060-7063 | **pass** | **FAIL** | **pass** | The halving is modelled (`magic_total_halving` → `lab_enchanting`). **FAIL on check 10** only: :7062's ordering rule ("apply the Deficiency first and then halve the remaining total") reaches the user nowhere → **F-544**. |
| `flaw.weak_magic` | 7064-7067 | **pass** | **FAIL** | **pass** | The Penetration halving is modelled. **FAIL on check 10**: the second clause ("only get half the normal benefit when instilling Penetration into an item") and the explicit ordering note are computed nowhere and written nowhere → **F-544**. |
| `flaw.weak_magic_resistance` | 7068-7071 | **pass** | **pass** | **pass** | The modelling direction is right, confirmed by the inbound site :9912. The `*Major*` magnitude is right, confirmed against :5307 and against the :9924 Ordeal (rated and dismissed above). Its `in_play_effect` class for a surfaced-only, `amount: 0` effect is **B18's Q-136**, which names this entry at `batch-18.md:1776`; an open escalated question is not re-opened. Uniquely in this span the mechanical sentence is carried in the `summary`, so D5 is satisfied in substance. |
| `flaw.weak_parens` | 7072-7074 | **pass** | **pass** | **pass** | −60 XP / −30 spell levels are right and land on the right pool: `effective/xp.rs::general_pool_and_bonus` (:963-980) builds a magus's general pool as `apprenticeship_xp + post_gauntlet_xp` and folds `GeneralXp` into it, so 240 → 180 and 120 → 90. `prerequisites: null` is escalated as **Q-140**, not reported. Range ends at :7074 (last content line) rather than the blank :7075 — a one-line inconsistency with 23 of its 24 neighbours, noted under F-540 and not separately numbered. |
| `flaw.weak_personality` | 7076-7079 | **pass** | **FAIL** | **pass** | Class and description are right under D3. **F-542**: ":7078 The character may have no other Personality Flaws" is enforced nowhere, and **four other entries in this very span** form a legal pair that breaks it. Index-section disagreement escalated as **Q-141**. `source.anchor` present and correct. |
| `flaw.weak_scholar` | 7080-7083 | **pass** | **pass** | **pass** | `lab_total_mod: -6` — value and sign verified against :7082. This entry is named explicitly in **D1** (row 5) and **D4** (row 4); both rulings are recorded and it is **not** re-reported. |
| `flaw.weak_spontaneous_magic` | 7084-7089 | **pass** | **FAIL** | **pass** | The ÷5 is **correct** — see "Withdrawn candidates". The :5783 incompatibility, reachable only inbound, is **F-385**, already live. **FAIL on check 10** only: ":7086 you must still roll a stress die … but the die roll does not add to your casting total" and "You may still use ceremonial casting" are computed nowhere and written nowhere → **F-544**. |
| `flaw.weakness_major` | 7090-7093 | **pass** | **pass** | **pass** | Both magnitudes ship; symmetric `incompatible_with`; index `Personality, Major or Minor` (:5337). |
| `flaw.weakness_minor` | 7090-7093 | **pass** | **pass** | **pass** | As above. |
| `flaw.weak_willed` | 7094-7097 | **pass** | **pass** | **pass** | Genuinely flavour — two sentences, no number, no roll. Index `Personality, Minor` (:5499) agrees with `minor` + `personality`. |
| `flaw.weird_magic` | 7098-7101 | **pass** | **pass** | **pass** | `uncomputed_rule` + description in both locales is right: the extra botch die is a roll-time quantity the character sheet has no field for. |
| `flaw.witch_marks` | 7102-7105 | **pass** | **pass** | **pass** | `tainted: true` and `categories: ["general"]` match the descriptor `*Minor, General, Tainted*` (:7103) and the index at `General, Minor` (:5637). The only `tainted` entry in the span. |
| `flaw.wrathful_major` | 7106-7119 | **pass** | **FAIL** | **pass** | **F-540** — `source.lines` ends 11 lines past the Flaw, inside Chapter 5. Everything else is clean: both magnitudes, symmetric incompatibility, index `Personality, Major or Minor` (:5338), six sample characters inbound. |
| `flaw.wrathful_minor` | 7106-7119 | **pass** | **FAIL** | **pass** | As above; same range, same finding. |

## Findings

Seven findings, **F-539 … F-545**. Severity follows `CLAUDE.md`'s threat model:
**wrong rules output is the top severity class** (the app's entire purpose is
computing correct characters), a character the app *refuses* to build or *accepts*
illegally is the same class of product-integrity failure, and a misclassification
is a lost-rule/provenance defect because `classification` is read by **no
production code**.

---

### F-539 — Warped by Magic permits Magic Lore; the app refuses to build the character

**Passage, verbatim (ArMDE:7021).**

> The character's adventures have exposed him to powerful magical forces that have
> left a mark on him. He has five Warping Points and a Warping Score of 1,
> including a Minor Flaw (which is not balanced by a Virtue) that somehow reflects
> the source of the Warping. **His encounters allow you to spend experience points
> on Magic Lore during character creation.** A magus may take this Flaw to
> represent Warping gained before his apprenticeship.

**German, verbatim (DE :7021).**

> **Seine Begegnungen erlauben es dir, während der Charaktererschaffung
> Erfahrungspunkte auf Magiekunde auszugeben.**

**Current data** (`rules/core/virtues_flaws.json`, `flaw.warped_by_magic`):

```json
"effects": [ { "type": "warping_grant", "score": 1, "points": 5 } ]
```

That is the entry's **only** effect. There is no `ability_authorization` and no
`restricted_ability_xp`.

**Why this is wrong — the sequence a player actually runs.**

1. `rules/core/abilities.json` classes `ability.magic_lore` as **`arcane`**.
2. `abilities.json::categories_requiring_virtue` is
   `["academic", "arcane", "martial"]`.
3. `crates/arm-rules/src/validation/authorization.rs::validate_ability_authorization`
   exempts exactly one case — `type_profile.is_some_and(|profile| profile.is_magus)`
   (`authorization.rs:61-63`) — and otherwise requires the category or the named
   Ability to appear in `crate::effective::ability_authorizations`, which is fed
   **only** by `Effect::AbilityAuthorization` or `Effect::RestrictedAbilityXp`
   (`authorization.rs:45-49`).
4. A grog, companion or mythic companion who takes Warped by Magic and spends one
   XP on Magic Lore therefore trips
   `ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE` at
   `CreationPhase::Abilities` — an **error**, not a warning. Under `Enforced`
   (guided and direct-validated modes) that **blocks** the character.

So the app refuses to build a character the rulebook explicitly authorizes, and it
does so for the majority of the character types that would want this Flaw — a
Warped mundane is precisely the Flaw's stated subject ("A magus may take this Flaw"
is offered as the *additional* case, not the primary one).

The function's own doc comment quotes the rule that makes this a defect rather than
an omission: *"Educated, Arcane Lore, and Warrior … are the easiest options for the
first three groups, although other Virtues **(and some Flaws)** also grant access
to some of these Abilities"* (`authorization.rs:12-16`, ArMDE:2315). Warped by
Magic is one of the Flaws that sentence is about.

**Correct value.** Add the authorization effect the catalogue already uses twice:

```json
{ "type": "ability_authorization", "abilities": ["ability.magic_lore"] }
```

**The remedy is reachable today** — `virtue.student_of_realm` already ships exactly
this effect naming exactly this Ability, and `flaw.covenant_upbringing` ships the
one-Ability form. No engine change, no new variant: data only.

Note the scope carefully. The passage permits **Magic Lore and nothing else**, so
the fix is the named-Ability form, not a category grant; a category grant would
hand the character the whole Arcane list, which is a different (and larger) wrong
answer.

**This is a new instance of the F-409 mechanism, not a duplicate of it.** F-409
records the mechanism and its two encoded carriers (`flaw.covenant_upbringing`,
`virtue.student_of_realm`); `flaw.warped_by_magic` appears in neither
`corrections.md`, `batch-17.md` nor `batch-18.md` on this axis — its only prior
mention is **F-500** (`corrections.md:703`), which is about `WarpingGrant.score`
being dead data, a disjoint concern.

**Severity: HIGH.** The app refuses to build a legal character.

---

### F-540 — Wrathful's `source.lines` runs past the end of the Flaws block into Chapter 5

**Current data.** `flaw.wrathful_major` and `flaw.wrathful_minor` both ship:

```json
"source": { "file": "Ars Magica - Definitive Edition (Core Rules).md",
            "lines": [ 7106, 7119 ] }
```

**What is actually on those lines.** The Flaw is three lines long:

```
7106  #### Wrathful
7107  *Major or Minor, Personality*<br>
7108  You are prone to anger over the smallest issues, and your rage when you are
      thwarted in something major is terrible to behold.
7109  (blank)
```

Everything from 7110 on belongs to something else:

| Line | EN | DE |
|---|---|---|
| 7110 | `---` | `---` |
| 7112 | the chapter-closing pull quote ("Torches flicker under a bright flare of magelight! …") | ("Fackeln flackern unter einem hellen Aufleuchten von Magielicht! …") |
| **7114** | **`# Chapter 5: Abilities`** | **`# Kapitel 5: Fertigkeiten`** |
| 7116 | "Abilities represent the things that a character has learned over the course of her life. …" | "Fertigkeiten stehen für die Dinge, die ein Charakter im Laufe seines Lebens gelernt hat. …" |
| 7118 | "Abilities are normally used by adding Characteristic + Ability + die roll …" | "Fertigkeiten werden normalerweise verwendet, indem Eigenschaft + Fertigkeit + Würfelwurf addiert werden …" |
| 7119 | (blank) | (blank) |

So the declared provenance of a Personality Flaw includes a horizontal rule, a
piece of chapter-closing fiction, **a top-level chapter heading**, and two
paragraphs of Ability rules. A reader following the citation to check what Wrathful
says lands on the Abilities chapter.

**Why the convention says so.** 23 of the other 24 ranges in this span run from the
`####` heading to the **blank line immediately before the next heading** —
`flaw.vow_*` 6989-6992, `flaw.vulnerable_casting` 6993-7004, `flaw.weak_personality`
7076-7079, and so on. Applying that convention to the last entry of the block gives
**`[7106, 7109]`**: heading, descriptor, text, trailing blank. (Ending at 7113 — the
blank before the *literal* next heading — would still swallow the `---` and the
pull quote, which belong to the chapter, not to Wrathful.)

The one other deviation in the span is **`flaw.weak_parens`, `[7072, 7074]`**, which
stops on its last content line instead of the blank :7075. That is harmless — it
covers all the entry's content and cites nothing foreign — so it is noted here for
the fixing pass rather than numbered separately. Wrathful is the opposite kind of
error and does cite something foreign.

**Correct value.** `"lines": [7106, 7109]` on both `flaw.wrathful_major` and
`flaw.wrathful_minor`.

**Severity: MEDIUM** (provenance). It computes no wrong number, but it is the single
citation in the catalogue that points a reader into an unrelated chapter, and — see
the `SWEPT_BLOCKS` note and `## Catalogue closure` — it is also the only reason any
catalogue entry claims a line outside the screened block.

---

### F-541 — Vulnerable Magic: `narrative` on two mechanical rules, no `parameters`, and a `max_per_target` that inverts the rule it encodes

**Passage, verbatim (ArMDE:7007-7009).**

> The character's magic is automatically dispelled in certain uncommon
> circumstances. Examples include: when touching iron, when under the influence of
> the Divine, when crossing over running water, when his name is spoken three
> times, or when he is not touching the ground. This condition immediately ends the
> duration of a spell when it is applied to the target, or all of your active spells
> when applied to you.
>
> **This Flaw may be taken multiple times, so long as a different condition is
> specified for each. It may not be combined with Restrictions or Necessary
> Conditions that have the same (or equivalent) conditions.**

**German, verbatim (DE :7009).**

> **Dieser Fehler kann mehrmals genommen werden, solange für jede Instanz eine
> andere Bedingung angegeben wird. Er kann nicht mit Einschränkungen oder
> Notwendigen Bedingungen kombiniert werden, die dieselben (oder gleichwertige)
> Bedingungen haben.**

**Current data.**

```json
{ "id": "flaw.vulnerable_magic", "magnitude": "major",
  "categories": ["hermetic"], "classification": "narrative",
  "max_per_target": 255 }
```

No `parameters`, no `max_total`, no `description`.

**Three defects, one entry.**

1. **`classification: "narrative"` is false.** `narrative` asserts the book states
   nothing mechanical. :7009 states a repeat rule and an incompatibility rule, and
   :7007 states a duration-termination rule. **D3** is explicit that an engine which
   cannot express a rule is grounds for `uncomputed_rule` with the rule written
   out, and *"it is **never** grounds for `narrative`"*. Correct value:
   `"classification": "uncomputed_rule"`.

   **The guard misses it, demonstrably.** This entry lies **inside** `SWEPT_BLOCKS`'
   `(ArMDE, 5639, 7113)`, is `narrative`, and
   `no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` passes it — because
   the 33-phrase list contains **no prohibition idiom in either language** (its only
   "Absolutes" pair is `cannot die` / `nicht sterben`), so ":7009 It may **not** be
   combined with…" is invisible. This is a *live, reproducible* miss of the guard on
   an entry the sweep has twice declared clean, not a hypothetical one.

2. **The missing `parameters` slot is NOT reported here.** ":7009 so long as a
   different condition is specified for each" makes the condition the entry's
   identity, and the book writes the Flaw in exactly that form itself — **ArMDE:18655**,
   a sample character carrying *"Vulnerable Magic (mua'addhin's call)"*. But this is
   the **unrecorded-choice family**, already escalated as **Q-134**
   (`batch-17.md:1788-1799`, itself the Q-86/Q-88/Q-95 family), which explicitly
   records that instances are listed *"**not** as a new question, and no per-entry
   finding is raised for them"*. `flaw.restriction`'s condition is already one of its
   five named instances. `flaw.vulnerable_magic` is a **sixth instance of Q-134** and
   is recorded as such, not as a finding. (This half of the entry was drafted as a
   finding and withdrawn on the verification pass — see `## Sub-agent reconciliation`.)

3. **`max_per_target: 255` encodes the opposite of the rule.** Per
   `types.rs:2189-2196`, `max_per_target` caps *"selections that share the same
   `(id, params)` target"*. With **no** `parameters`, every copy shares the one empty
   target, so 255 is a licence to take **255 identical** copies of a Flaw the book
   allows only *with a different condition each time*. The rule says
   "each one different"; the data says "255 of the same". They are inverses.

   `types.rs:2203-2207` documents precisely this case — *"a Virtue repeatable 'with a
   different target each time' may need `max_per_target: 1` … alongside … no
   `max_total` at all (unlimited targets)"*. The correct value is **`max_per_target: 1`** — which is the default, so the field
   is simply **removed**. (A `parameters` slot would then make repeats legal again,
   one per condition, exactly as :7009 says — but that slot is Q-134's to grant, not
   this finding's.) Removing the field alone is strictly correct on its own: today
   the data licenses 255 identical copies, which the book licenses in no reading.

**What must go into `description` (D5), in both locales.** The duration-termination
rule of :7007, the repeat rule, and the Restriction / Necessary Condition
incompatibility. That last one is **correctly absent** from `incompatible_with` and
must stay absent: `flaw.restriction` (:5793-…) and `flaw.necessary_condition` both
exist, but the prohibition is conditional on *condition-equality* between two
free-text strings, and `incompatible_with` holds bare ids with no predicate. This is
the D3 case exactly — write it out, do not fake it.

**Severity: MEDIUM-HIGH.** The classification error is a lost rule; the
`max_per_target` inversion is data that actively states the wrong rule, and a Major
Flaw taken 255 times is 765 Flaw points offered to the budget (bounded only by the
separate total-Flaw cap, which is a different check catching it for a different
reason).

---

### F-542 — Weak Personality forbids other Personality Flaws; four of them sit in this same span and all four pair legally

**Passage, verbatim (ArMDE:7078).**

> Consequently, all Personality Traits must be between +1 and -1. When making
> personality rolls, treat any roll above 6 as merely 6. **The character may have no
> other Personality Flaws or Virtues or Flaws that grant Personality Traits.**

**German, verbatim (DE :7078).**

> **Der Charakter darf keine anderen Persönlichkeits-Fehler oder -Tugenden oder
> Fehler haben, die Persönlichkeitszüge verleihen.**

**Current data.** `flaw.weak_personality` ships `classification: "uncomputed_rule"`
with the full passage as `description` in both locales, `categories: ["personality"]`,
**no `incompatible_with`**, no `prerequisites`.

**The classification and the description are right.** Under **D3**, an engine that
structurally cannot express a rule surfaces it as text. It cannot express this one:
`validation/selections.rs::validate_forbidden_categories` reads
`profile.forbidden_categories` — a field on `EntityTypeProfile`, i.e. on the
*character type* — and there is **no** `Effect` variant by which a Virtue or Flaw can
forbid a category. So this entry's class is not the finding.

**The finding is the F-459 shape: a "never" broken by a legal pair, and nothing in
the validation layer ranges over pairs.** A player today may select, with no warning
of any kind:

| Illegal pair the app accepts | Why the book forbids it |
|---|---|
| `flaw.weak_personality` + `flaw.vow_minor` | Vow is `*Minor, **Personality***` (:6990) |
| `flaw.weak_personality` + `flaw.weakness_minor` | Weakness is `*Major or Minor, **Personality***` (:7091) |
| `flaw.weak_personality` + `flaw.weak_willed` | Weak-Willed is `*Minor, **Personality***` (:7095) |
| `flaw.weak_personality` + `flaw.wrathful_minor` | Wrathful is `*Major or Minor, **Personality***` (:7107) |

**All four partners are entries in this batch's own 25.** The span that contains the
prohibition also contains four of its violations, and the catalogue holds many more
— every entry whose `categories` includes `"personality"`.

**The second half of the sentence is out of the engine's reach entirely, and is
deliberately *not* claimed here.** *"…or Virtues or Flaws that grant Personality
Traits"* has a sharp instance in the book — `virtue.berserk` (**ArMDE:3502**): *"You
automatically gain the Personality Trait **Angry +2** (or more, at your option)"*,
against Weak Personality's **+1 / −1** cap, and ArMDE:2502's general rule that a
Minor Personality Flaw *"should be represented by a Personality Trait with a score of
+3 or -3"*. The pair is illegal in the book and the app accepts it. **But the
verification pass established that no `Effect` variant grants a Personality Trait at
all**, and the engine models no Personality Trait scores, so the app prints no wrong
number here — it merely fails to warn. That is a weaker claim than the category half
and is recorded as context, not as part of the finding. **The finding is the category
half alone.**

**Correct value — and this remedy is *not* reachable today, which is part of the
finding.** There are three candidate shapes and only the third is honest:

1. `incompatible_with` listing every Personality Flaw id. Expressible, but it
   violates *"Catalogue size is data, never code"* in spirit: adding a Personality
   Flaw to `rules/core/` would silently *weaken* an existing entry's absolute unless
   someone remembers to edit this list too. It also cannot catch the "grants a
   Personality Trait" half at all.
2. `Prereq::Nor(vec![Has(…), Has(…), …])` — same enumeration, same decay, and
   additionally `validate_incompatibilities` is bought-only on both sides
   (F-463/F-465/F-527), so a *granted* Personality Flaw would slip through.
3. **A category-scoped prohibition carried by a Virtue/Flaw.** The engine already
   has the *vocabulary* — `profile.forbidden_categories` even supports a conditional
   `when` form, as `character_types.json`'s companion profile shows
   (`hermetic` forbidden unless `virtue.the_gift`) — but no route from a V/F to it.
   This is the same missing mechanism as **F-355** on the Ability axis
   (ArMDE:5653, "class of Abilities"); this is its Virtue/Flaw-axis twin, and the two
   should be fixed with one decision rather than two.

Until that exists, the entry stays `uncomputed_rule` with the description it has —
which is already correct — and the *finding* is that the absolute is enforced
nowhere.

**Why this is not disposed of by D3, which is the objection the verification pass
raised and which is refused here on evidence.** D3's scope note says a batch reaching
a structurally-inexpressible rule *"does not report the engine's inability as a
defect"*. That premise does not hold: **option 1 is expressible today**. An
`incompatible_with` enumerating the Personality Flaw ids is plain data, needs no new
`Effect` variant and no engine change, and would catch all four pairs above. The
reason to prefer option 3 is maintainability, not impossibility — and "there is a
cheap fix we dislike" is not D3's case. D3 covers `Susceptibility to Faerie Power`,
where no arrangement of existing fields can express a realm-scoped halving; it does
not cover a prohibition that a list of ids expresses exactly.

**Severity: HIGH.** The app accepts a character the rulebook forbids, and four of the
accepting pairs are inside this batch's own 25.

---

### F-543 — Warped Senses' eight hard prohibitions are legible from one of the five entries they bind

**Passages, verbatim (ArMDE:7031 and :7033).**

> *Sensitive (Sense):* … You suffer a -2 penalty on all activities under relevant
> circumstances. **(It is inadvisable to combine Sensitive Sight with Keen Vision,
> and Sensitive Hearing with Sharp Ears, and these are incompatible with Blind and
> Deaf, respectively.)**

> *Weak (Sense):* … **(Weak Sight is incompatible with Sensitive Sight, Keen Vision,
> and Blind and you cannot take Weak Hearing with Sensitive Hearing, Sharp Ears, or
> Deaf.)**

**German, verbatim (DE :7033).**

> **(Schwaches Sehen ist inkompatibel mit Empfindlichem Sehen, Scharfer Sicht und
> Blind, und du kannst Schwaches Hören nicht mit Empfindlichem Hören, Scharfen Ohren
> oder Taub nehmen.)**

**The two sentences are not equally binding, and the distinction matters.** :7031
says *"It is **inadvisable** to combine"* for Sensitive+Keen Vision and
Sensitive+Sharp Ears — advice, not a prohibition — and then *"and these **are
incompatible** with Blind and Deaf"* — a prohibition. :7033 is a prohibition
throughout (*"is incompatible with"*, *"you cannot take"*). So the **hard** set is:

| Variant | Hard-incompatible with |
|---|---|
| Sensitive Sight | `flaw.blind` |
| Sensitive Hearing | `flaw.deaf` |
| Weak Sight | Sensitive Sight, `virtue.keen_vision`, `flaw.blind` |
| Weak Hearing | Sensitive Hearing, `virtue.sharp_ears`, `flaw.deaf` |

**Current data.** `flaw.warped_senses` ships `uncomputed_rule` with the full passage
as `description` (correct, and its `source.anchor` is present and right), **no
`parameters`**, and **no `incompatible_with`**. The four partners ship no
`incompatible_with` either: `flaw.blind`, `flaw.deaf`, `virtue.keen_vision`,
`virtue.sharp_ears` — verified individually.

**This finding was drafted larger and was cut down by the verification pass; the cut
is accepted.** The original claim was that the four prohibitions belong in
`incompatible_with`. They do not, and cannot:

- A flat `incompatible_with: ["flaw.blind", …]` on `flaw.warped_senses` **over-fires**.
  The entry also covers *Sensitive to Cold* and *Sensitive to Heat*, neither of which
  touches sight or hearing, and Sensitive to Cold + Blind is perfectly legal.
- The mirror form fails the same way: `incompatible_with: ["flaw.warped_senses"]` on
  `virtue.keen_vision` would forbid the legal Keen Vision + Sensitive to Cold.
- `validation/prereq.rs` matches **bare ids with no parameter scoping**, so a
  variant-scoped prohibition is not expressible — the same gap as B18's **Q-137**
  (`batch-18.md:1785`: *"`incompatible_with` holds ids; ArMDE:6925 states a
  predicate"*). Under **D3** that makes `uncomputed_rule` + description the right
  answer, and that is exactly what ships, verbatim in both locales.

**What survives, and it is real.** The rule is **one-directional**. It is legible only
from `flaw.warped_senses`' own `description`, and from **none of the four entries it
also binds**: `flaw.blind`, `flaw.deaf`, `virtue.keen_vision` and `virtue.sharp_ears`
each ship no `description` and no hint that a Warped Senses variant is closed to them.
A player who picks Blind first — the likelier order, since Blind is a Major Flaw and a
bigger decision — is told nothing, and the app will not tell him later either. D5's
obligation is that the rule *reach the user*; it reaches him only if he happens to
open the other entry.

**Correct value.** A `description` on each of the four partners, in both locales,
carrying the prohibition from its own side: for `flaw.blind`, *"incompatible with the
Sensitive Sight and Weak Sight variants of Warped Senses"* (ArMDE:7031, :7033), and
the three analogues. No `incompatible_with`, no `parameters`, no engine change.

**The hard/soft split, recorded because it is easy to get wrong and both the batch and
the verification pass derived it independently and agreed.** ArMDE:7031 contains
*two different strengths in one sentence*: *"It is **inadvisable** to combine Sensitive
Sight with Keen Vision, and Sensitive Hearing with Sharp Ears"* — advice, **2 soft
pairs**, which must **not** be encoded as prohibitions — *"and these **are
incompatible** with Blind and Deaf, respectively"* — prohibition. ArMDE:7033 is
prohibition throughout (*"is incompatible with"*, *"you cannot take"*). DE :7031/:7033
mirror the split (*"Es ist **ratsam** … nicht zu kombinieren"* vs *"ist
**inkompatibel** mit"* / *"**kannst** … **nicht** … **nehmen**"*). The hard set is
**8 pairs**:

| Variant | Hard-incompatible with |
|---|---|
| Sensitive Sight | `flaw.blind` |
| Sensitive Hearing | `flaw.deaf` |
| Weak Sight | Sensitive Sight, `virtue.keen_vision`, `flaw.blind` |
| Weak Hearing | Sensitive Hearing, `virtue.sharp_ears`, `flaw.deaf` |

**Prior-record check.** `virtue.keen_vision` is **F-126** (`corrections.md:271`),
`virtue.sharp_ears` is **F-271** (`corrections.md:431`), `flaw.blind` is **F-369**
(`corrections.md:539`) — all three on their **classification**, none on this rule.
`flaw.deaf` and `flaw.warped_senses` appear in none of the three records. New on all
five entries.

**Severity: LOW.** Downgraded from MEDIUM on the verification pass. `flaw.warped_senses`
itself is correct as shipped; the defect is four missing descriptions on its partners.

---

### F-544 — Seven entries state a mechanical clause the engine does not compute and carry no `description` (D5)

**D5, verbatim** (`decisions.md:191-192`):

> **Ruling: any mechanical clause the engine does not compute must be written into
> `description`, in both locales, whatever the entry's classification.**

Reported as one finding because it is one omission shape with one remedy, applied
seven times. All seven currently ship **no `description` in either locale**.

| Entry | Class | The clause that reaches the user nowhere | ArMDE |
|---|---|---|---|
| `flaw.vulnerable_casting` | `in_play_effect` | *"A maga may have, or acquire, this Flaw more than once, losing 1 extra Fatigue level for each level of this Flaw."* — the whole level-stacking scheme; plus the short-term/long-term severity rule (:7001), the "no Fatigue loss stays no Fatigue loss" exception (:6995), and the order-of-application rule *"apply the vulnerability first, then withstand the increased loss"* (:7003). A 12-line passage reaches the user as **one sentence** of summary. | :6995, :6997, :7001, :7003 |
| `flaw.vulnerable_magic` | `narrative` | *"This condition immediately ends the duration of a spell when it is applied to the target, or all of your active spells when applied to you"*, plus the repeat and incompatibility rules. See F-541. | :7007, :7009 |
| `flaw.warped_by_magic` | `creation_effect` | *"His encounters allow you to spend experience points on Magic Lore during character creation"* — see F-539; even once the authorization effect is added, the *permission* itself should be legible. Plus *"With the permission of the troupe, this Flaw may be modified to represent Warping from other Realms."* | :7021 |
| `flaw.weak_enchanter` | `in_play_effect` | *"If you have a Deficiency that counts as part of the Lab Total, apply the Deficiency first and then halve the remaining total."* An **ordering** rule between two effects, and order changes the arithmetic. | :7062 |
| `flaw.weak_magic` | `in_play_effect` | *"…and only get half the normal benefit when instilling Penetration into an item"* — a second, uncomputed clause; plus *"Note that you halve the Penetration Total, **after** subtracting the spell level and making any adjustments for the use of Arcane Connections. You do not halve the Casting Total and calculate Penetration from that."* | :7066 |
| `flaw.weak_scholar` | `in_play_effect` | The **condition** on the −6: *"when working from the Lab Texts of others, including when re-inventing spells."* The number is computed; the condition that governs whether it applies is invisible. (D1/D4 rule on how the engine *treats* the condition; they do not relieve the obligation to tell the user it exists.) | :7082 |
| `flaw.weak_spontaneous_magic` | `in_play_effect` | *"In stressful conditions you must still roll a stress die to see if you botch, but the die roll does not add to your casting total"*, and *"You may still use ceremonial casting."* Neither is computed. Plus the combination note with Difficult Spontaneous Magic (:7088). | :7086, :7088 |

**Not in this list, and why.** `flaw.weak_magic_resistance` states a mechanical
clause the engine does not compute, but it is **already carried in the entry's
`summary`** (uniquely in this span, the summary is two sentences and includes *"If
the conditions are met, do not subtract the level of the effect from the casting
total before calculating Penetration"*). D5's obligation is that the rule reach the
user; it does. `flaw.weak_characteristics`, `flaw.weak_parens` and the eleven
genuinely narrative entries state nothing beyond what is computed.

**Correct value.** A `description` on each of the seven, in `rules/i18n/en/` **and**
`rules/i18n/de/`, transcribing the uncomputed clauses. The German text must come
from the line-parallel DE source, not from translating the English.

**Note for the fixing pass.** Adding these descriptions does **not** bring the seven
under the existing guard: `crates/arm-rules/tests/uncomputed_clauses.rs` is
class-keyed (it asks "is this `uncomputed_rule`?"), which is exactly the gap D5's
clause 1 orders closed. Until the guard stops being class-keyed, nothing will stop
these descriptions from being deleted again.

**Severity: MEDIUM** (lost rule, every session, both locales).

---

### F-545 — Wanderlust: a countable seasonal constraint classified as flavour

**This finding exists because the verification pass overturned a *clean* verdict.**
The batch first passed `flaw.wanderlust` on the reasoning that the app models no
seasons, so nothing mechanical touches a generated character. **D3 does not admit
that reasoning**, and the overturn is accepted — see `## Sub-agent reconciliation`.

**Passage, verbatim (ArMDE:7017).**

> The character feels compelled to travel with a passion that is so strong that **he
> cannot spend more than a season in the same place. The character can only spend two
> (nonconsecutive) seasons each year in the same area, and must spend the intervening
> seasons traveling to different locales** — places he has never been. If for some
> reason he cannot travel, he will become very uncomfortable; this discomfort should
> cause stories in the same way as the travel would.

**German, verbatim (DE :7017).**

> …**dass er nicht mehr als ein Quartal an demselben Ort verbringen kann. Der
> Charakter kann nur zwei (nicht aufeinanderfolgende) Quartale pro Jahr in demselben
> Gebiet verbringen und muss die dazwischenliegenden Quartale damit verbringen, zu
> verschiedenen Orten zu reisen**…

**Current data.** `classification: "narrative"`, `categories: ["story"]`,
`magnitude: "major"`, no effects, no `description`.

**Why `narrative` is false.** `narrative` is a claim that the book states nothing
mechanical. This passage states a **counted** constraint: a ceiling of one season in
one place, a ceiling of **two** seasons per year in one area, an ordering constraint
(*nonconsecutive*), and a mandatory activity for the seasons in between. Those are
four quantified clauses. That the app cannot act on them is a fact about the engine,
and **D3 is explicit** (`decisions.md:299-302`):

> An engine that structurally cannot express a rule is grounds for `uncomputed_rule`
> with the rule written out — it is *never* grounds for `narrative`. `narrative`
> remains a claim that the book states nothing mechanical, and nothing about engine
> capability can make that claim true.

The engine genuinely cannot express it: there is **no `Season` type in `arm-rules`**
and no seasonal-activity model anywhere; `effective/xp.rs` funds life stages as XP
blocks, never as scheduled seasons. So this is D3's case exactly, and D3 names the
answer.

**Correct value.** `"classification": "uncomputed_rule"`, plus a `description` in
**both** locales carrying the four clauses of :7017 verbatim from the line-parallel
sources.

**Scope note, stated because this reading has reach.** Applying D3 this strictly will
reclassify other `narrative` Story Flaws whose passages count something. That is the
correct consequence — D3 was written to make exactly that happen — but it is a
catalogue-wide consequence of a decision, not a B19 discovery, and the fixing pass
should expect it rather than be surprised by it. The distinction that keeps it
bounded is *countability*: `flaw.weak_willed` (:7096, "You look to others for guidance
rather than to yourself") counts nothing and stays `narrative`; Wanderlust counts
seasons.

**Severity: MEDIUM.** Lost rule, both locales. `classification` is read by no
production code, but the `description` it would have obliged is what reaches the user.

## Open questions

Two. Both entries are **NOT marked checked**.

---

### Q-140 — Weak Parens and Skilled Parens have no `IsMagus` prereq, and a Gifted companion can reach them

**Who settles it.** N (a rules call about who may have had a *parens*).

**The facts.**

- `flaw.weak_parens` (ArMDE:7072-7074) and its mirror `virtue.skilled_parens` both
  ship `prerequisites: null` and `categories: ["hermetic"]`.
- Both describe apprenticeship: *"Your parens was less powerful or a worse teacher
  than normal. You gain 60 fewer experience points and 30 fewer spell levels **from
  apprenticeship**"*.
- The `hermetic` category is **not** a magus-only gate.
  `rules/core/character_types.json` forbids it outright for `grog` and
  `mythic_companion`, but the **companion** profile permits it conditionally:
  `{"category": "hermetic", "when": {"kind": "has", "value": "virtue.the_gift"}}`.
  So a **Gifted, non-magus companion** may legally select Weak Parens.
- What then happens is not an error, it is a silent budget change.
  `effective/xp.rs::general_pool_and_bonus` (:963-980) builds
  `base_general` as `budget.apprenticeship_xp + budget.post_gauntlet_xp` **only
  when `is_magus`**; otherwise it is `budget.later_life_xp`. The −60 is then folded
  in unconditionally. A Gifted companion who never had a parens loses 60 XP from his
  *later life*, and pockets a Minor Flaw's worth of Virtue points for it.

**Why this is not reported as a finding.** Three readings are defensible and the
audit cannot pick between them from the source:

1. **The book intends it.** A Gifted non-magus *can* have been apprenticed and left
   the Order; "from apprenticeship" would then be narrative framing and the XP has to
   come from somewhere, so `later_life_xp` is as good a pool as any.
2. **The book does not intend it**, and both entries need `"prerequisites":
   {"kind": "is_magus"}`. This is the B17-shape defect (*a Flaw "only taken by
   Verditius magi" shipping `prerequisites: null`*), and the remedy is one line of
   data on each of two entries.
3. **The gate belongs on the profile, not the entry** — i.e. the companion profile's
   conditional `hermetic` permission is itself too generous.

Reading 2 has the strongest textual support (:7074's figures, 180 and 90, are
*apprenticeship* figures and appear nowhere else), but it is a rules judgement about
Gifted non-magi that ArMDE does not settle in this span, and it would change what
characters are buildable. **Escalated, not decided.**

**Note for whoever decides.** Whatever the answer, it applies to **both** entries
identically — `virtue.skilled_parens` ships the exact mirror
(`spell_levels: +30`, `general_xp: +60`, `categories: ["hermetic"]`,
`prerequisites: null`) and is out of this span. Neither appears in
`corrections.md`, `batch-17.md` or `batch-18.md`.

---

### Q-141 — the book indexes Weak Personality under *Story, Minor*; its own descriptor says *Personality*

**Who settles it.** N (a call about which source wins, with a UI consequence).

**The facts.**

- `flaw.weak_personality`'s descriptor line, ArMDE:**7077**: `*Minor, Personality*<br>`.
  DE :7077: `*Klein, Persönlichkeit*<br>`. Both agree.
- The book's **List of Flaws** index puts it at ArMDE:**5518**, which is inside
  `### Story, Minor` (:5501) and before `### Social Status, Minor` (:5520) — not in
  `### Personality, Minor` (:5467-5499), where `[Weak-Willed]` sits at :5499 and
  where alphabetical order would have placed *Weak Personality* immediately before it.
- Shipped data follows the descriptor: `categories: ["personality"]`.

**Why I have not called it either way.** The descriptor is the per-entry authority
and the data follows it, so on the narrow question "is the data wrong?" the answer
looks like no — the **book** is internally inconsistent, and it is the index that
looks misplaced (last element of an alphabetical list, out of order).

But there is a user-facing consequence that makes it more than a curiosity, and it
is exactly the kind CLAUDE.md rates up. **The user has the rulebook open beside the
app.** He reads the Story, Minor list, finds Weak Personality there, filters the
app's Flaw list by Story — and it is not there. The repo's whole line-parity
invariant exists to make the book and the app navigable together; this is a place
where they diverge, and the divergence originates in the book.

**Three options, none of them free.**

1. **Leave it.** Data follows the descriptor; accept that one entry is unfindable
   from one of the book's two indexes.
2. **Ship `categories: ["personality", "story"]`.** Makes it findable from both, but
   asserts a category the entry's own descriptor does not, and a Story Flaw is
   supposed to generate stories — Weak Personality's passage generates none.
3. **Record it as a known source erratum** somewhere the next data-generation pass
   will see, so nobody "fixes" the data *to* the index later. This is the cheapest and
   is what I would recommend, but it is a decision about where such errata live, and
   this audit has no such register today.

**This is the only place in the 25 where the descriptor and the index disagree**;
the other 24 were checked against both and agree. **Escalated.**

---

### Q-142 — does the `scheitest` precedent cover a string containing both `guarter` and `quarter`?

**Who settles it.** N, or whoever owns the `scheitest` precedent.

**`flaw.waster_of_vis` is NOT marked checked on its text.**

**The facts.** `rules/i18n/en/virtues_flaws.json:1389` and `:1390` ship, in `summary`
and `description`:

> "When you use raw vis you waste one **guarter** (rounded up) of the pawns you
> apply."

`rules/source/en/…(Core Rules).md:7054` carries the identical typo, so the extraction
is *faithful*. The same OCR substitution occurs at **ArMDE:3502** in `virtue.berserk`
(*"or give **guarter**"*, for the idiom "give quarter"), which confirms it is a
systematic `q`→`g` OCR error in the source rather than a transcription slip here.
`virtue.berserk` ships no `description`, so that instance reaches no user today.
German is unaffected: DE :7054 reads *"ein **Viertel** (aufgerundet)"*, correctly.

**Why the precedent does not obviously settle it.** B17 established the `scheitest`
rule and applied it to `oneround` (`batch-17.md:1409-1412`): *"a description copied
from the source keeps it; a description **written** for the app should read 'one
round'"* — flagged *"so the choice is deliberate rather than accidental"*. Under that
rule `flaw.waster_of_vis` is **checked and left alone**, and this batch does not file
it as a finding.

**But this instance has a property `oneround` did not, which is why it is escalated
rather than silently passed.** The *same shipped string* contains the typo **and its
correct spelling**, six clauses apart:

> "…you waste one **guarter** (rounded up) of the pawns you apply. … and 4, **one
> quarter** of those you use, are wasted."

So the user is not reading a consistently-misspelled term he can mentally correct once
— he is reading a paragraph that spells one word two ways and gives him no way to know
which is meant. That is a comprehension defect a faithful-copy rule was probably not
written to protect.

**What is being asked.** Either (a) the precedent holds and this stays as-is —
in which case say so, and the EN source keeps the typo too; or (b) the precedent
carves out self-contradicting strings, in which case both shipped strings and
`rules/source/en/…:7054` and `:3502` are corrected together, since the JSON is
*generated* from the Markdown and a source-only fix or a data-only fix will not hold.

## Sub-agent reconciliation

One verification sub-agent was spawned after the primary read of both sources and the
shipped data, and before any finding was drafted in final form. It was briefed to
**independently re-derive** rather than review, to ask of every entry what happens
when a player does what it says, to recount every census figure **with a different
tool**, to check each proposed remedy against the engine's actual reachability, and
to look up every out-of-span entry in `corrections.md`, `batch-17.md` **and**
`batch-18.md`. It was told explicitly to challenge anything the Method asserted
without re-derivation. It received the auto-mode injection and reports ignoring it.

It changed this batch substantially. **Three clean verdicts overturned, three findings
cut down or withdrawn, one duplicate confirmed, one remedy refuted, and one
withdrawal refused.**

### Overturns accepted

**1. `flaw.wanderlust` — a clean verdict overturned into F-545.** I passed it,
reasoning that the seasons rule constrains post-gauntlet activity the app does not
model, so nothing mechanical reaches a generated character. The verifier refused the
reasoning and cited **D3** back at me: *"nothing about engine capability can make that
claim true"* (`decisions.md:301-302`). It is right, and my reasoning was the exact
inversion D3 was written to forbid — I used engine incapability as evidence for
`narrative`. It also checked the premise I had only assumed, confirming there is **no
`Season` type in the crate at all**. Accepted in full; filed as F-545.

**2. `flaw.warped_senses` — my remedy could not fire, and F-543 was rebuilt around
what survives.** I proposed `parameters` over the four variants plus per-variant
prohibitions. The verifier read `validation/prereq.rs:299-335` and showed the
prohibition half is unreachable in *both* directions — not only would a flat
`incompatible_with` on Warped Senses over-fire, but the mirror form on
`virtue.keen_vision` would forbid the legal Keen Vision + Sensitive to Cold. Under D3
the shipped `uncomputed_rule` + description is therefore **correct**, and the entry is
clean. This is the same contribution a previous verifier made — *correcting a proposed
remedy that could not fire* — and it is the most useful kind. Accepted: the entry's
verdict flipped to **pass**, F-543 was re-aimed at the four partners' missing
descriptions, and its severity dropped MEDIUM → **LOW**. We independently derived the
same 2-soft / 8-hard split of :7031-:7033 and agreed, which is why that table is
stated as fact.

**3. `flaw.vulnerable_magic`'s missing `parameters` — withdrawn as a duplicate.** I had
it as one third of F-541. The verifier identified it as the **Q-134** unrecorded-choice
family (`batch-17.md:1788-1799`), which states in terms that instances are recorded
*"**not** as a new question, and no per-entry finding is raised for them"* — and
`flaw.restriction`'s condition is already one of its five. I had read Q-134's heading
in `batch-17.md` and not its body. Withdrawn; recorded as Q-134's **sixth** instance.
F-541 now carries two claims, not three.

**4. `flaw.waster_of_vis` — withdrawn as a finding, escalated as Q-142.** I filed the
shipped `guarter` typo as F-545 (LOW). The verifier produced **B17's `scheitest`
precedent** (`batch-17.md:1409-1412`), under which a description copied verbatim from
the source keeps the source's typo and is *checked and left alone*. Filing it as a
finding would have contradicted a live precedent. Withdrawn. Its own escalation —
that this string uniquely contains **both** `guarter` and `quarter` — is the reason it
became Q-142 rather than simply passing.

### Withdrawal refused, on evidence

**F-542 (`flaw.weak_personality`) — the verifier moved to withdraw it and the
withdrawal is refused.** Its argument: no per-item category block exists
(`validate_forbidden_categories` and `flaw_category_caps` are both *profile*-level),
therefore D3 applies, therefore `uncomputed_rule` + description is the answer and the
engine's inability is not a defect.

Both halves of its premise are correct and neither supports the conclusion. D3's
scope note relieves a batch of reporting an inability only where the engine
**structurally cannot** express the rule — its own case, `Susceptibility to Faerie
Power`, is one where no arrangement of existing fields expresses a realm-scoped
halving. That is not this case. **`incompatible_with` enumerating the Personality Flaw
ids is plain data that works today**, needs no new `Effect` variant and no engine
change, and would catch all four pairs. I rejected it in the finding on
*maintainability* grounds and recommended the category-block mechanism instead — but
"there is a cheap fix we dislike" is not D3's premise, and treating it as one would let
D3 excuse any absolute anyone declined to enumerate.

The concrete evidence that decides it: a player may today select
`flaw.weak_personality` together with `flaw.vow_minor`, `flaw.weakness_minor`,
`flaw.weak_willed` or `flaw.wrathful_minor` — **all four inside this batch's own 25** —
and the app raises nothing at any `ValidationMode`. ArMDE:7078 forbids each pair in
terms. The finding stands at **HIGH**.

**What I did concede.** The verifier established that **no `Effect` variant grants a
Personality Trait**, so the Berserk clash (Angry +2 against a +1 cap) causes the engine
to print no wrong number — it only fails to warn. That is a materially weaker claim
than I had made, and F-542's second half was rewritten to say so and excluded from the
finding.

### Confirmations that closed doubts rather than opening them

- **F-539** (`flaw.warped_by_magic`) confirmed independently, and strengthened: the
  verifier traced `effective/xp.rs:365-395` and found `Effect::WarpingGrant` sitting in
  the **explicit no-op arm** at `xp.rs:432`, so nothing authorizes the Ability by any
  route. It also established the issue is an **error** in `Enforced` and a warning in
  `Advisory`, and that the entry is absent from all three prior records on this axis.
- **F-352 / F-385** — both confirmed as pre-existing, which is why neither appears in
  this batch's numbering. Had the prior record not been checked, B19 would have filed
  two duplicates, and on the F-352 one I would very likely have written the sentence
  the brief warns about — that no earlier batch reported it.
- **The `flaw.weak_spontaneous_magic` ÷5** — I withdrew it on reading
  `derived/casting.rs`; the verifier withdrew it independently and added the base-rule
  citations I lacked, **ArMDE:9143** (÷2, exerting) and **:9145** (÷5, not exerting).
  Two independent withdrawals of the same candidate.
- **`flaw.weak_parens`' XP pool** — confirmed correct by both, and the verifier supplied
  the undiminished figures' citation (**ArMDE:2215**, `life_stages.json`): 240 → 180,
  120 → 90.
- **Census** — all six figures I reported were re-counted with a different tool and
  **all six held**, including the `entries` / `rows` distinction on `effects` (9 entries,
  10 rows) that B18's verification found wrong three times. It added the two figures
  that carry the closure argument: `source.anchor` at **94 / 655**, and exactly **2**
  catalogue entries with `source.lines[1] > 7113`.

### What the verifier found that I had not

- **`RULES.md:4146` states a magus-only behaviour as universal.** It reads
  *"`Effect::GeneralXp` folds into the general **apprenticeship** pool"*. Verified
  directly: `effective/xp.rs:963-979` uses `apprenticeship_xp + post_gauntlet_xp` only
  when `is_magus`, and `later_life_xp` otherwise. This is the *"an authority
  contradicting itself is itself a finding"* shape; it is folded into **Q-140** rather
  than numbered, because it documents the behaviour Q-140 asks about and both must be
  settled together.
- **Two translation-table rows disagree with the DE rulebook**, in
  `rules/source/de/translation-tables/grundbegriffe.md`, both **untagged** (so D7 rule 3
  does not remove them from the dispute):
  `:344` `| Vulnerable Magic | **Verwundbare Magie** | Großer Hermetischer Fehler |`
  against the DE heading and shipped name *Anfällige Magie* (DE :7005); and
  `:410` `| Vulnerable to Folk Tradition | **Anfällig für Volkszauber** | Kleiner
  Hermetischer Fehler |` against *Anfällig für Volksüberlieferungen* (DE :7011).
  Per **D7 rule 1** the DE rulebook heading is the default winner, so **both shipped
  names look correct** and no data change is proposed. Per **D6** the rows may be a
  stale copy of `arm-de-translation`, which is outside the working directory and
  unreadable from here. **Reported as "this repository's copy disagrees with the
  rulebook"; the determination is the orchestrator's.** Note that both rows also carry
  a *magnitude-class* claim (`Großer`/`Kleiner Hermetischer Fehler`) — a factual claim
  about the rules in a terminology table, which is the D6 error shape found seven times
  already. Both magnitude claims happen to be **correct** here.
- **The 33-phrase screen has no prohibition idiom in *either* language.** I had
  established the German discontinuous-negation limit from the module's own doc comment
  (`uncomputed_clauses.rs:168-173`). The verifier's sharper point is that the English
  side is no better: `may not be combined`, `is incompatible with`, `you cannot take`,
  `may have no other`, `may not exert yourself` are all absent from the list, whose only
  "Absolutes" pair is `cannot die` / `nicht sterben`. And it demonstrated the
  consequence live — **`flaw.vulnerable_magic` sits inside the swept block, is
  `narrative`, passes the screen, and drops a real rule.** That turns "the screen has a
  gap" from an argument into a reproduction, and it is now evidence inside F-541.

### Where we disagreed and the disagreement is recorded rather than resolved

The verifier read the `flaw.warped_senses` and `flaw.weak_personality` cases as
symmetrical — both D3, both clean. I accepted the first and refused the second. The
asymmetry is deliberate and is the load-bearing judgement of this batch: **a
variant-scoped prohibition is not expressible with any arrangement of today's fields,
while a category-scoped one is expressible with a list of ids.** If that distinction
is rejected, F-542 falls with it and `flaw.weak_personality` becomes clean. It is
stated here plainly so the decision can be taken against the reasoning rather than
inherited.

## Catalogue closure

This is the last batch, and the last time the catalogue will be read with fresh eyes
before the fixing pass. Four statements, each checked rather than assumed.

### (a) Indices 630-654 complete the 655

Confirmed two ways. `jq 'length'` over `rules/core/virtues_flaws.json` returns
**655**; `grep -c '"entity_kinds"'` over the same file returns **655**, and
`entity_kinds` appears exactly once per entry. 630 + 25 = 655, so this span is the
terminal slice and **every Virtue and Flaw in the catalogue has now been through a
batch**.

### (b) The last entry's `source.lines` does **not** end the Flaws block cleanly

It is the one range in the span that is wrong, and it is wrong at the very end of the
catalogue — see **F-540**. `flaw.wrathful_major` and `flaw.wrathful_minor` declare
`[7106, 7119]`; the Flaw occupies 7106-7108.

**What actually follows the Flaws block**, in order, in both languages at identical
line numbers:

| ArMDE | EN | DE |
|---|---|---|
| 7109 | (blank) | (blank) |
| 7110 | `---` | `---` |
| 7112 | chapter-closing pull quote, *"Torches flicker under a bright flare of magelight! …"* | *"Fackeln flackern unter einem hellen Aufleuchten von Magielicht! …"* |
| 7114 | `# Chapter 5: Abilities` | `# Kapitel 5: Fertigkeiten` |
| 7116+ | the Abilities chapter | das Fertigkeiten-Kapitel |

So the Flaws block ends at **7108** (content) / **7109** (trailing blank), and the
correct range for the catalogue's last entry is `[7106, 7109]`. Nothing after 7109
is Virtue or Flaw text; the catalogue is complete at that line and there is no
truncated or unread tail.

### (c) No other entry in the catalogue cites a line beyond this span's end

Checked catalogue-wide, not within the span. `jq '[.[] | .source.lines[1]] | max'`
returns **7119**, and the only entries carrying it are this batch's two Wrathful
entries; `grep -c "7119"` independently returns **2**. Filtering on
`source.lines[1] > 7113` returns the same two ids and nothing else.

Two conclusions follow:

1. **The catalogue cites no line past 7119 anywhere**, so nothing in
   `rules/core/virtues_flaws.json` points into Chapter 5 or beyond except through
   F-540.
2. **Fixing F-540 lowers the catalogue's maximum cited line from 7119 to 7109**, at
   which point *every one of the 655 entries* lies wholly inside `SWEPT_BLOCKS`'
   `(ArMDE, 5639, 7113)` or an earlier block — and the two entries that the screen
   has never once looked at come under it. As the `SWEPT_BLOCKS` note above records,
   the order matters: **fix the range first, then widen or re-run the screen.** Doing
   it the other way makes the screen read Chapter 5's *"meeting or exceeding an Ease
   Factor"* (:7118) as Wrathful's own text and report a false offender.

### (d) Observations that belong to the catalogue, not to these 25

Four things this span surfaced that are about the whole data set. None is numbered as
a B19 finding; each is recorded because this is the last pass before fixing starts.

**1. `source.anchor` is populated on 94 of 655 entries (14.4 %), and the 94 are not a
principled subset.** In this span exactly three entries carry one —
`flaw.vulnerable_to_folk_tradition`, `flaw.warped_senses`, `flaw.weak_personality` —
and they are precisely the three whose `description` is a verbatim transcription of
the passage. That pattern says `anchor` is a by-product of the description-extraction
pass rather than a field the catalogue populates on purpose. All three present
anchors are correct, and the 22 absences break nothing today. But the field is a
navigation contract with the rulebook, and 14.4 % coverage is the worst kind of
partial: present often enough to look reliable, absent often enough not to be. **The
fixing pass should decide whether `anchor` is mandatory or should be dropped**, and
the memory note about capturing `####` heading anchors during source reads points at
mandatory. It is not a per-entry defect and should not be filed as 561 of them.

**2. Line-citation drift is now the catalogue's only unguarded provenance axis.**
`rulebook_citations.rs` guards *comment* citations; `rules_source_provenance.rs`
guards out-of-bounds `source.lines`; `uncomputed_clauses.rs` guards misclassification
inside swept blocks. **Nothing guards a `source.lines` range that is in bounds, lands
on non-blank lines, and is simply the wrong extent** — which is exactly F-540's shape,
and the reason it survived nineteen batches at the most visible position in the file.
A cheap guard exists: assert that a range ends before the next `####`/`###`/`##`/`#`
heading in the cited file. That one assertion would have caught F-540 mechanically.

**3. Two neighbouring category-block gaps should be decided together, not twice.**
F-355 is the missing category block on the **Ability** axis ("a class of Abilities",
ArMDE:5653). **F-542** is its twin on the **Virtue/Flaw** axis ("no other Personality
Flaws", ArMDE:7078). The engine has the vocabulary on neither — it has
`profile.forbidden_categories` with a conditional `when` form, reachable only from a
character-type profile, never from a V/F. One `Effect` variant carrying a
category-scoped prohibition closes both. Deciding them separately risks two
mechanisms for one idea.

**4. `max_per_target` with no `parameters` is a silent inversion, and F-541 is
probably not the only carrier.** `types.rs:2189-2196` defines `max_per_target` as a
cap on selections sharing the same `(id, params)` target. When an entry has **no**
`parameters`, every copy shares one empty target, so a high `max_per_target` licenses
*identical* repeats — which is the right encoding for a level-stack
(`flaw.vulnerable_casting`, `virtue.withstand_casting`: "Vulnerable Casting (2)") and
the **exact inverse** of the rule for a vary-the-target repeat
(`flaw.vulnerable_magic`: "so long as a different condition is specified for each").
The two cases are indistinguishable in the data and sit three entries apart in this
span. Both spellings are live in the catalogue; **the fixing pass should sweep every
entry with a non-default `max_per_target` and no `parameters` and ask which of the two
rules it means.** This span found one of each, which is the smallest sample that can
show the ambiguity and the strongest reason to think there are more.
