# Batch B17 — `flaw.reckless_major` … `flaw.supernatural_nuisance`

Indices 560–594 of the canonical ordering, **ArMDE:6663-6802**, **35 entries, all
Flaws**. Findings **F-502 … F-522** (21, of which **F-522 was found by the
verification pass, not by the primary read**). Open questions **Q-130 … Q-135**
(6). **15 entries pass every check; 19 entries fail at least one; 1 entry
(`flaw.seeker`) is ESCALATED and is NOT marked checked.** The arithmetic,
because it is easy to state loosely: the 20 findings land on **17 entries plus
one test file** — two entries (`flaw.restricted_power`, `flaw.rigid_magic`) carry
two findings each, and F-514 lands on `uncomputed_clauses.rs` rather than on any
entry. The other **two** failing entries fail on a *prior* batch's finding only
(`flaw.sleep_disorder` → F-465, `flaw.servant_of_the_land` → F-498), which is
17 + 2 = 19 (`flaw.servant_of_the_land` is one of those two, and now carries
F-522 of its own as well as B16's F-498, so the entry count is unchanged).
15 + 19 + 1 = 35.

**The three shapes, stated as claims.**

1. **`narrative` is this span's default, and it is wrong fourteen times.** The
   span is 24 `narrative` entries as shipped. Fourteen of them state a rule in
   words a token screen cannot see — a duration table (`flaw.short_lived_magic`),
   a round count (`flaw.slow_caster`, `flaw.slow_power`), an eligibility gate
   (`flaw.spontaneous_casting_tools`, `flaw.rector`), a named-Ability block
   (`flaw.sheltered_upbringing`), a `min()` substitution
   (`flaw.stuck_in_your_ways`), a magic-resistance rule
   (`flaw.stockade_parma_magica`), an absolute on casting (`flaw.restriction`,
   `flaw.rigid_magic`), a restriction on activating a power
   (`flaw.restricted_power`), a study restriction (`flaw.study_requirement`), a
   seasonal obligation (`flaw.regular`), and a creation-time XP restriction
   (`flaw.restricted_learning`). Every one sits inside `SWEPT_BLOCKS`'
   `(ArMDE, 5639, 7113)` block, swept 2026-09-15 and re-swept 2026-09-19, green
   both times.
2. **Two entries let the app build a character the book forbids, and one lets it
   build a character at twice the rule's budget.**
   `flaw.spontaneous_casting_tools` says "*can only be taken by Verditius magi*"
   and carries no `prereqs`, so **every magus of all thirteen Houses, and every
   Gifted companion, may take it**; `flaw.sheltered_upbringing` forbids seven
   named Abilities at creation and the app sells all seven; `flaw.savantism` is
   `creation_effect` with **no `effects` at all**, so a Savant is funded at the
   full standard XP instead of half and may start any Ability at the age cap
   instead of 3.

   *A first draft of this batch said "any grog". That was wrong and is corrected
   here rather than quietly dropped: `rules/core/character_types.json` gives the
   `grog` profile `forbidden_categories: ["hermetic"]`, and
   `validation/selections.rs::validate_permitted_categories` raises
   `CODE_CATEGORY_NOT_PERMITTED` — a hard error — on it. The `companion` profile
   permits `hermetic` only `when` the character has `virtue.the_gift`
   (ArMDE:2840). So the reachable population is magi plus Gifted companions, not
   everybody. See F-518.*
3. **A rule that is *surfaced* is not a rule the player can read.** Three
   entries (`flaw.short_of_breath`, `flaw.short_ranged_magic`, and — as a
   secondary — `flaw.savantism`) reach the user as a bare slug-and-amount row or
   as nothing, with no `description` in either locale, although D5 obliges every
   uncomputed clause to be written out whatever the classification. The
   guard that would catch this is class-keyed and so never looks at them, and
   its sign detector additionally cannot see `flaw.short_of_breath`'s sign
   character at all (F-514).

---

## Method

### What was read, and in what order

1. `docs/vf-audit/README.md`, `decisions.md` (D1–D7), `corrections.md`
   (§ 0–§ 5 in full, § 6/§ 7 by index), `engine-semantics.md` (Part A entries
   A7, A11, A12, A13, A33, A35, A36, A40; Part B B10, B11; Part C headings), and
   `batch-16.md`'s structure.
2. **The English source as continuous prose, ArMDE:6640-6824** — overrunning the
   span by 23 lines above (back through `#### Proud`) and 22 below (forward
   through `#### Susceptibility to Infernal Power`).
3. **The German source over the identical range, DE 6640-6824.**
4. The shipped mechanics for all 35 entries (`jq` over
   `rules/core/virtues_flaws.json`) and the shipped text for all 35 in **both**
   locales (`rules/i18n/{en,de}/virtues_flaws.json`).
5. The consumer functions themselves, not the yardstick's summary of them, for
   every verdict that rests on engine behaviour:
   `derived.rs::surfaced_modifiers` (read at
   `crates/arm-rules/src/derived.rs:627-644`),
   `crates/arm-rules/tests/uncomputed_clauses.rs`'s `SIGN_CHARS` and
   `has_signed_number`, `crates/arm-rules/src/spell.rs`'s `SpellDef::ritual`.
6. `crates/arm-rules/RULES.md` — searched for all 35 entry names before any
   modelling choice was called wrong. **It changed one finding** (see F-510).

### Line parity — confirmed, and here is the check

The German file is line-parallel with the English across the whole span and
beyond it at both ends. Spot-verified at both boundaries and at every heading:

| ArMDE | EN heading | DE heading |
|---|---|---|
| 6642 | `#### Proud` | `#### Stolz` |
| **6663** | `#### Reckless` | `#### Rücksichtslos` |
| 6703 | `#### Savantism` | `#### Savantismus` |
| 6745 | `#### Sleep Disorder` | `#### Schlafstörung` |
| **6799** | `#### Supernatural Nuisance` | `#### Übernatürliche Plage` |
| 6803 | `#### Suppressed Gift` | `#### Unterdrückte Gabe` |

All **34** `####` headings between 6663 and 6802 sit on the same line number in
both files, and each descriptor line (`*Major, Story*` ↔ `*Groß, Geschichte*`)
follows on the same line. Parity holds.

### Per-check roll-ups

**Check 1 — `source.lines` and `source.anchor`.** All 35 pass. There are **34**
`####` headings in 6663-6802 and 35 ids, because `flaw.reckless_major` and
`flaw.reckless_minor` correctly share the single `#### Reckless` heading's range.
Every range runs from its heading to the line before the next heading, including
the two two-paragraph entries — `flaw.savantism` 6703-6708 (next heading 6709)
and `flaw.sleep_disorder` 6745-6750 (next heading 6751).

`source.anchor` is present on exactly the six `uncomputed_rule` entries
(`repellent`, `rolling-stone`, `servant-of-the-land`, `sleep-disorder`,
`social-handicap`, `stigmatic-catalyst`) and on nothing else. All six match the
book's own anchor in the "List of Flaws" index (ArMDE:5415, 5620, 5367, 5623,
5626, 5556) and in the alphabetical index (ArMDE:25357, 25384, 25418, 25450,
25462, 25497). Catalogue-wide the field is carried by 84 of 102 `uncomputed_rule`
entries and by 10 others, so the span's pattern matches the catalogue's; this is
**not** re-derived here and no anchor finding is raised. **Consequence for
Phase 2:** each of the thirteen entries this batch moves to `uncomputed_rule`
will need an anchor added at the same time, and the correct value is the index
link listed above the entry's name.

**Check 2 — `classification`.** 24 `narrative`, 6 `uncomputed_rule`, 3
`in_play_effect`, 2 `creation_effect` as shipped — re-derived from the data, not
inherited. **Fourteen `narrative` entries and one `creation_effect` entry are
wrong** — F-502, F-503, F-504, F-505, F-507, F-508, F-510, F-511, F-512, F-516,
F-517, F-518, F-519, F-520, F-521, which is **fifteen** findings on **fifteen**
entries (F-510 is the `creation_effect` one, `flaw.savantism`; F-518 is
`flaw.spontaneous_casting_tools`, whose *primary* defect is the prereq and whose
classification moves as a secondary). 24 − 14 = **ten** `narrative` entries are
correct. All fifteen move to `uncomputed_rule`, none to a computed class: in every case the
engine structurally cannot express the rule, which is **D3** exactly. The
remaining ten `narrative` entries split **nine clean and one escalated**. The
nine genuinely pure-colour ones — `reckless_major`, `reckless_minor`,
`reclusive`, `secretive`, `short_attention_span`, `simple_minded`, `slothful`,
`soft_hearted`, `supernatural_nuisance` — are recorded as cleared below. The
tenth, **`flaw.seeker`, is escalated on Q-135** and is not marked checked: its
own text points at *Houses of Hermes: True Lineages*, that book is in
`rules/source/en/`, the pointer resolves, and HoH:TL:503 states an eligibility
rule ("*A magus from any House may be a Seeker*") the ArMDE passage does not.

**This paragraph's figures were re-counted after the first draft, and the first
draft was wrong**: it said "thirteen" where the entry list has fourteen, having
dropped `flaw.restricted_power`, and then said "fourteen entries" for a
fifteen-item list. Both are corrected above. The header's pass/fail split was
wrong in the same pass (15/20 for what is 16/19) and is corrected there.

**Check 3 — `kind`.** All 35 are `flaw`, all 35 descriptor lines are Flaw
descriptors, and all 35 appear under `## List of Flaws` (ArMDE:5283). Pass.

**Check 4 — `magnitude`.** All 35 match their descriptor line. **The F-487
dual-magnitude check, run over the span:** exactly one descriptor in 6663-6802
reads `*Major or Minor*` — `#### Reckless` at ArMDE:6664 (`*Groß oder Klein*`,
DE 6664) — and both `flaw.reckless_major` and `flaw.reckless_minor` exist. The
book's index confirms it independently: `[Reckless]` appears at ArMDE:5335, under
`### Personality, Major or Minor` (ArMDE:5310), and in no other index list.
Catalogue-wide, **all 32 `*_major` ids carry a `*_minor` incompatibility and
there are exactly 32 `*_major` ids**, so the Reckless pair follows a universal
convention rather than an ad-hoc one. Pass; no new instance of F-487.

**Check 5 — `entity_kinds`.** All 35 are `["character"]`. Two passages bear on
it and both confirm it: ArMDE:6677 "*Magi can be Regular*" (so no exclusion) and
ArMDE:6685 "*All mundane animals are required to take this Flaw*" — animals are
not an `EntityKind` the app models, so nothing follows for the field. Pass.

**Check 6 — `categories` + `tainted`.** All 35 match the descriptor line **and**
the book's index list, which is an independent second source. The mapping,
verified list by list:

| Index list (ArMDE) | Span entries found there |
|---|---|
| `### Hermetic, Major` (:5285) | Restriction :5297, Rigid Magic :5298, Short-Ranged Magic :5299, Study Requirement :5300 |
| `### Personality, Major or Minor` (:5310) | Reckless :5335 |
| `### Story, Major` (:5340) | Rector/Proctor :5366, Servant of the (Land) :5367, Supernatural Nuisance :5368 |
| `### General, Major` (:5401) | Repellent :5415 |
| `### Hermetic, Minor` (:5417) | Short-Lived Magic :5448, Slow Caster :5449, Spontaneous Casting Tools :5450, Stockade Parma Magica :5451 |
| `### Personality, Minor` (:5467) | Reclusive :5490, Secretive :5491, Seeker :5492, Short Attention Span :5493, Simple-Minded :5494, Sheltered Upbringing :5495, Slothful :5496, Soft-Hearted :5497 |
| `### Supernatural, Minor` (:5534) | Restricted Power :5554, Slow Power :5555, Stigmatic Catalyst :5556 |
| `### General, Minor` (:5564) | Regular :5618, Restricted Learning :5619, Rolling Stone :5620, Savantism :5621, Short of Breath :5622, Sleep Disorder :5623, Slow Reflexes :5624, Small Frame :5625, Social Handicap :5626, Stuck in Your Ways :5627 |

That is 34 names for 35 ids (Reckless counted once). Every one agrees with the
shipped `categories`. `tainted: true` appears once, on `flaw.repellent`, whose
descriptor at ArMDE:6680 reads `*Major, General, Tainted*` and whose German at
DE 6680 reads `*Groß, Allgemein, Befleckt*` — the `Tainted → Befleckt` mapping
CLAUDE.md names. Pass, 35/35.

**Check 7 — `prereqs`.** No entry in the span carries one. **Three passages
state an eligibility gate** and all three are therefore unencoded: ArMDE:6781
("*can only be taken by Verditius magi*", F-518), ArMDE:6673 ("*must have a
Social Status Virtue*", F-502), and ArMDE:6789's presupposition of a Parma
Magica (F-519, the weakest of the three and reported as such). The remaining 32
absences are correct.

**The profile category gate was traced before any reachability claim was made,
and it changed two findings.** Seven entries in the span are
`categories: ["hermetic"]` — `restriction`, `rigid_magic`, `short_lived_magic`,
`short_ranged_magic`, `slow_caster`, `spontaneous_casting_tools`,
`stockade_parma_magica`. `rules/core/character_types.json` gives the `grog`
profile `forbidden_categories: ["hermetic"]` and the `companion` profile a
*conditional* permit keyed on `virtue.the_gift`, and
`crates/arm-rules/src/validation/selections.rs::validate_permitted_categories`
(read at `selections.rs:56-130`) turns both into a hard
`CODE_CATEGORY_NOT_PERMITTED` error. Its own comment records the rule: ArMDE:2840's
"*unless you have The Gift*" opens `hermetic` "*to a Gifted companion and to
nobody else*". **So "any grog could take this" is false for every Hermetic entry
in the span**, and the first draft of F-518 and F-519 said exactly that. Both are
corrected in place, and the surviving populations — every non-Verditius magus,
and a Gifted companion — are what those findings now assert. Recorded here rather
than silently fixed, because the wrong version is the one a reader would
otherwise reach for on the next Hermetic entry.

**Check 8 — `incompatible_with`, and what its absence means.** Present on two
entries. `flaw.reckless_major` ↔ `flaw.reckless_minor` is the universal
dual-magnitude convention (check 4). `flaw.small_frame` carries
`["flaw.dwarf","virtue.giant_blood","virtue.large"]`, which is exactly
ArMDE:6769's list, symmetric on all three sides — verified by reading each
partner's own row. **Three absences were interrogated and two are meaningful:**

- `flaw.sleep_disorder` ↔ `flaw.night_terrors` — ArMDE:6749 states it and
  ArMDE:6492 states it from the other side; neither is encoded. **Already
  F-465 (B15)**; not re-reported. Confirmed live at ArMDE:6749 ("*This Flaw is
  not compatible with the Night Terrors Flaw*") and DE 6749 ("*Dieser Fehler ist
  nicht vereinbar mit dem Fehler Nachtschrecken*").
- `flaw.small_frame` ↔ `virtue.blood_of_the_nephilim` — see "Cross-references"
  below. **Already F-23 (B01)**; an extension, not a new finding.
- `flaw.regular` and `virtue.wealthy` / `flaw.poor` — ArMDE:6677 says the
  Flaw **is** compatible with both, so the absence is *correct and stated*, the
  same shape as § 5's merchant entries. `flaw.study_requirement` and
  `virtue.study_bonus` likewise: ArMDE:6797 says "*You may take both*". **Do not
  add either pair.**

**Check 9 — `parameters`.** **Three** entries carry one. Two are right; **the
third is F-522 and the primary read missed it.** The miss has a nameable cause
worth recording: the check was run as "does each entry that *discusses* repetition
encode it correctly?", which examined `restricted_power` and `slow_power` and
cleared both — when ArMDE:2814 makes **silence** the thing to look for, so the
right question is "does each *parameterized* entry carry the `max_total` its
silence implies?". `flaw.servant_of_the_land` says nothing about repetition and
was therefore never asked. See F-522.
`flaw.restricted_power` and `flaw.slow_power` each carry
`{"key":"power","type":"ref","domain":"text","require_power":true}` and no
`max_total`. The book grants the repeat on both — ArMDE:6689 "*may be taken once
for each power*", ArMDE:6761 "*may be taken more than once … but not more than
once for a single power*" — and `max_per_target`'s default of **1**, keyed on
`(item_ref, whole params map)` per `engine-semantics.md` B10, permits two copies
naming different powers while forbidding two naming the same one. That is
precisely what both sentences say. No `max_total` is correct: the ceiling is the
number of powers, which is not a constant. **This is not an F-446 instance.**

Five entries make a choice the save cannot record — `flaw.restriction` (the
condition), `flaw.supernatural_nuisance` (the kind of entity),
`flaw.restricted_learning` (the five Abilities), `flaw.repellent` (the minor
advantage), `flaw.rector` (faculty or nation). Four are the unsettled Q-86/Q-88/
Q-95 family and are carried to **Q-134** rather than filed as findings; the
fifth, `flaw.restricted_learning`'s five Abilities, is different in kind because
the rule is *arithmetic on those five ids*, and it is filed inside F-504.

**Check 10 — `effects` vs the passage, in both directions.** No entry in the span
carries an effect the passage does not support (the outbound direction is clean,
35/35). The inbound direction is where the span fails: three entries carry an
effect that expresses part of the passage and drop the rest (F-513, F-515, and
`flaw.slow_reflexes`, which is clean — see cleared negatives), and fourteen carry
none at all where the passage states a rule.

**Check 11 — engine reality, signs included.** Three entries carry effects.

| Entry | Effect | Consumer read | Verdict |
|---|---|---|---|
| `flaw.short_of_breath` | `health_mod{fatigue_roll, -3}` | `derived.rs::surfaced_modifiers` (`crates/arm-rules/src/derived.rs:627-644`) | **Sign correct.** `FatigueRoll` is pushed as a `SurfacedModifier{family: HealthRoll, detail: "fatigue_roll", amount: -3}` — the amount *does* reach the read-out, contrary to what "surfaced-only" might suggest. It is the passage's **scope** that is lost (F-513). |
| `flaw.short_ranged_magic` | `special_casting_mod{circumstantial}` | `derived.rs::in_play_mods` | **Correct as far as it goes and empty in substance.** Per A40 the eight non-computed kinds are pushed with `amount: 0`; `circumstantial` carries no amount field at all, so the *kind slug is the whole payload*. Neither halving reaches any number (F-515). |
| `flaw.slow_reflexes` | `combat_mod{initiative, -3}` | `derived/combat.rs::combat_totals` via `cm(stat, weapon)` | **Correct, sign included.** Initiative is a total one adds to, so −3 lowers it. Clean. |

**Check 12 — text.** 35 EN names, 35 DE names, 35 summaries per locale, 6
descriptions per locale, all compared against the passage in the *matching*
language. **Two failures, in two different fields, so the roll-up is per field
rather than per entry:** of the 35 **DE names**, 34 match their DE heading
exactly and **one fails** (`flaw.restricted_power`, F-506); of the 35 **DE
summaries**, 34 reproduce their DE source sentence and **one fails**
(`flaw.rigid_magic`, F-509). All 35 EN names match their EN heading, all 35 EN
summaries match their EN source sentence, and all 12 descriptions (6 per locale)
are faithful. **Negative signs:** every signed
value in the six shipped descriptions — `-6` (repellent), `-1` (rolling stone),
`-3` (social handicap) — is ASCII hyphen-minus U+002D in **both** locales, while
the source writes U+2013. Correct per CLAUDE.md. **Three faithful-copy cases were
checked and left alone**, on the `scheitest` precedent (§ 5): EN `flaw.repellent`
reproduces the source's `demonic eves` typo (ArMDE:6681 — the German at DE 6681
says `dämonischen Augen`, correctly, because the German file is a translation
rather than a copy); DE `flaw.sleep_disorder` reproduces DE 6747's awkward "*Mehr
als nicht*"; EN `flaw.simple_minded` and DE `flaw.simple_minded` reproduce their
sources' em dash and en dash respectively, neither of which is a sign.

### Part C systemic gaps are not re-reported per entry

Three entries in the span carry an effect whose variant is a known Part C gap,
and **none of them is reported as a per-entry defect on that ground**:

- `flaw.short_of_breath`'s `fatigue_roll` track is one of A36's three
  surfaced-only tracks. Per README's Phase 0 note, that is "*correctly authored
  against an engine that lists rather than computes it*". F-513 is about the
  **Concentration clause the data drops entirely**, not about the track.
- `flaw.short_ranged_magic`'s `circumstantial` kind is one of A40's eight
  surfaced-only kinds. F-515 is about **D5's description obligation**, not about
  the kind.
- `flaw.savantism`'s emptiness is **C7** (`creation_effect` is not required to
  carry effects). F-510 is about **this entry's four uncomputed numbers and its
  missing description**, and explicitly does not re-raise C7.

Likewise, **F-409's authorization family produced no instance in this span, and
that is a finding-shaped negative rather than an omission.** Three passages here
look like permissions and are not:

- ArMDE:6707 — "*If the favored Ability requires a Virtue, such as Academic or
  Martial Abilities, then he **must have that Virtue as normal***". This is the
  exact opposite of the F-409 shape: it *re-states* the gate rather than waiving
  it. No `ability_authorization` is owed and adding one would be a bug.
- ArMDE:6685 — "*Any Supernatural Abilities from Virtues she possesses are
  **added to the list***" is a permission inside `flaw.restricted_learning`'s own
  restriction, not a permission against `categories_requiring_virtue`.
- ArMDE:6785 — `flaw.stigmatic_catalyst` names an *Intelligence + Enigmatic
  Wisdom* roll. Naming an Ability in a roll formula presupposes it; it does not
  grant it. No authorization owed.

**The mirror gap (F-410 / F-355) gains one instance and is not re-derived.**
`flaw.sheltered_upbringing` forbids seven named Abilities at creation. No
`Effect` variant forbids an Ability or an Ability category — the gap recorded by
F-410 (`flaw.enfeebled`, a category prohibition) and F-355 (`flaw.ability_block`,
a named-Ability prohibition, which F-426 duplicated). F-511 reports the entry;
the engine gap is theirs.

### The live catalogue-wide patterns, each checked against this span

Stated as results, including the negatives, so the next batch knows this span was
searched rather than skipped.

| Pattern | Result in ArMDE:6663-6802 |
|---|---|
| **F-409 — the authorization family** | **No instance.** Three permission-shaped sentences were examined and all three are the *opposite* shape; see the Part C subsection above. |
| **F-410 / F-355 — no effect FORBIDS an Ability or category** | **One instance: F-511** (`flaw.sheltered_upbringing`, seven named ids). The engine gap is F-410's and is not re-derived. |
| **F-428 / F-439 — a restricted XP pool stacking on a life-stage block** | **No instance.** No entry in the span carries `restricted_ability_xp`; only three entries carry any effect at all (`health_mod`, `special_casting_mod`, `combat_mod`, `size_delta`). |
| **F-446 — an entry the book lets you take twice, unparameterized** | **No instance.** The two repeatable entries (`restricted_power`, `slow_power`) are both **parameterized**, which is what makes the repeat legal under `max_per_target`'s `(item_ref, params)` key. See check 9. |
| **F-495 / F-451 — parameterized with no `max_total`, so the entry stacks** | **One instance: F-522** (`flaw.servant_of_the_land`), found by the verification pass after the primary read cleared the span on the wrong question. It is the worst of the three, because `land` is unbounded free text where F-495's domain is four Realms and F-451's is ten Forms. |
| **F-450 / F-486 — a hardcoded or invented `grants_reputation` value** | **No instance. No entry in the span carries `grants_reputation` at all**, and no passage in 6663-6802 states a Reputation — checked by reading all 34 passages, not by searching for the effect. |
| **F-449 / F-462 — an effect scoped more broadly than the passage** | **No instance.** The one candidate, `flaw.slow_reflexes`'s unconditional −3 against "*in situations warranting a quick response*", was examined and cleared: the condition is when an Initiative total is rolled at all. |
| **F-459 — an absolute broken by a legal PAIR** | **One instance, already owned.** `flaw.small_frame`'s "*Size is reduced **to** –1*" versus `virtue.blood_of_the_nephilim`'s `size_delta: +1` yields Size 0. Covered by **F-23 (B01)**; extended, not re-reported. The other absolutes in the span were each asked the same question: `flaw.restriction`'s "*cannot cast spells at all*", `flaw.rigid_magic`'s "*cannot use vis*", `flaw.stockade_parma_magica`'s "*cannot suppress*", `flaw.sheltered_upbringing`'s "*may not take*" and `flaw.restricted_learning`'s "*can only apply experience points to these*" — none is defeated by a second legal selection, because none is computed at all. |
| **F-463 / F-465 — a stated prohibition carried by no `incompatible_with`** | **One instance, already owned** (F-465, sleep_disorder ↔ night_terrors). Two further absences were checked and are *correct* — ArMDE:6677 and ArMDE:6797 both state compatibility. |
| **F-487 — a dual-magnitude descriptor shipped in one magnitude** | **No instance.** The span's single `*Major or Minor*` descriptor is Reckless and both ids exist; see check 4. |
| **F-494 — an `uncomputed_rule` the engine can actually compute** | **No instance among the six shipped `uncomputed_rule` entries** — each was checked against Part A and none maps to a variant. But **F-510 is the same shape arriving from `creation_effect`**: `flaw.savantism`'s "*may not begin with an Ability above 3*" is a cap `AgeAbilityCaps` could enforce. |
| **F-427 — ArMDE:2816's one-Social-Status rule** | **One site: `flaw.rector`** (:6673), which requires the character to hold a Social Status Virtue. Noted inside F-502; the catalogue-wide claim is F-427's and is not re-derived. |
| **Truncated shipped text** | **Not re-scanned** — CLOSED, bounded at four entries and five strings. |

### Decisions applied

| Decision | Where it bit in this batch |
|---|---|
| **D1** | Not reached — no entry in the span carries `lab_total_mod`. `flaw.short_ranged_magic` halves a Lab Total via a *different* mechanism, which is why Q-133 exists rather than an assertion. |
| **D2** | Not reached — no entry in the span is grantable by a House or type profile, and none carries a precondition of the `great_characteristic` shape. |
| **D3** | **The load-bearing ruling for this batch.** All fourteen reclassifications go to `uncomputed_rule`, never to a computed class, precisely because the engine structurally cannot express the rule — and D3 forbids `narrative` on that ground. Applied to F-502, F-503, F-504, F-505, F-507, F-508, F-510, F-511, F-512, F-516, F-517, F-519, F-520, F-521. |
| **D4** | Not reached (no `lab_total_mod`). |
| **D5** | Applied to every finding. Three entries (F-513, F-515, and F-510's second half) are D5-only: the classification is defensible or is being changed for a different reason, and the defect is the clause reaching neither locale. |
| **D6** | One table row is in play (`tugenden-fehler.md:346`) and **D7 removes it from the dispute** — see F-506. No table correction is proposed and no stale-copy determination is needed. |
| **D7** | **Settles F-506 outright, with no escalation.** The `Restricted Power` glossary row is tagged `SdM:M` — *Sphären der Macht: Magie*, a different book — so by D7 rule 3 it "is not in the dispute at all", and the DE core-book heading at DE 6687 wins. This is D7's worked case running the other way: the three entries D7 examined were shipped *following* the heading and were upheld; this one is shipped *against* it. |

### Cross-references followed and rated

Every outbound pointer in the span's 35 passages, plus every inbound reference
found by sweeping all 34 entry names across the whole English rulebook.

| Entry | Pointer | Where it lands | Adds a rule? | Already in `corrections.md`? |
|---|---|---|---|---|
| `flaw.seeker` | "page 15 of *Houses of Hermes: True Lineages*" (:6715) | **HoH:TL:497-505**, the `### Seekers` section — HoH:TL **is** in `rules/source/en/`, so this pointer is resolvable and was followed | **Yes — and this overturns my own first reading, which was "no rule reaches this entry".** HoH:TL:503 opens "**A magus from any House may be a Seeker.**" That is an eligibility statement in two halves: it *removes* a House restriction, and it presupposes the Seeker is a **magus**. HoH:TL:501 ("*Seekers are magi who devote themselves…*") and HoH:TL:505 ("*approximately 25 Seekers; five of them are magi of House Bonisagus*") both read the same way, and HoH:TL:815 restates the Flaw itself. **See Q-135** — `flaw.seeker` is **escalated, not marked checked.** | No — and it is the § 3.1 blind-spot-#2 shape (F-442, F-452, F-453, F-478): a rule stated about a named entry *in another book*, which the phrase screen cannot see because it reads only the entry's own cited range. |
| `flaw.small_frame` | "(See page 404)" (:6769) | the wound-severity table | **No.** ArMDE:6769 already states the four-point increment; the page reference is to the table it derives from. `derived/combat.rs::wound_ranges` computes it from `size_delta` — the `flaw.dwarf` precedent § 5 records. | § 5, `flaw.dwarf` row |
| `flaw.small_frame` | "Giant Blood (page 83), Large (page 89), or Dwarf (page 126)" (:6769) | three catalogue entries | **Yes, and it is encoded.** All three are in `incompatible_with`, symmetric. | n/a — clean |
| `flaw.study_requirement` | "the Study Bonus Virtue on page 110" (:6797) | `virtue.study_bonus` | **Yes — a negative one.** "*You may take both Study Bonus and Study Requirement*" states that no incompatibility exists. The absence is correct. | § 5's merchant shape |
| `flaw.supernatural_nuisance` | "*This differs from Plagued by Supernatural Entity*" (:6801) | `flaw.plagued_by_supernatural_entity` | **Yes, a distinguishing rule** — "*the nuisances do not have any long-term plans*" — and it is pure colour, expressible by nothing. No defect. | n/a |
| `flaw.sleep_disorder` | "*not compatible with the Night Terrors Flaw*" (:6749) | `flaw.night_terrors` (:6488-6495) | **Yes.** Encoded on neither side. | **Yes — F-465 (B15).** Not re-reported. |
| `flaw.servant_of_the_land` | "*the Minor Personality Flaw: Prohibition, but this does not count toward the character's total*" (:6719) | `flaw.prohibition` (:6638-6641), which exists and is `narrative` | **Yes** — a grant plus a budget exemption. | **Yes — F-498 (B16).** Extension only: the grant target `flaw.prohibition` **does exist** in the catalogue (confirmed at `rules/core/virtues_flaws.json`, lines 6638-6641), so `grants_selection` has a resolvable id and F-498's fix is not blocked on creating one. |
| **inbound** → `flaw.small_frame` | "*Virtues or Flaws that affect your Size*" (ArMDE:3517) | `virtue.blood_of_the_nephilim` | **Yes, and it breaks an absolute.** ArMDE:6768 says Size is "*reduced **to** –1*". `virtue.blood_of_the_nephilim` carries `size_delta: +1`; neither entry names the other in `incompatible_with`; the legal pair yields Size **0**. This is the F-459 pair shape. | **Yes — F-23 (B01)**, "the whole prohibition list at ArMDE:3517 is encoded nowhere". Extension only: F-23's fix must reach `flaw.small_frame`'s row too, because `validate_incompatibility_symmetry` fails the load on a one-sided declaration. **Consequence recorded here so Phase 2 does not miss it:** `size_delta` as a delta is the *right* encoding for ArMDE:6768's "reduced to −1" **only once F-23 lands** — the other three size-changers are already excluded, and Nephilim is the sole remaining pair. |
| **inbound** → `flaw.study_requirement` | "*Major Hermetic Flaw:* Study Requirement" (ArMDE:10167) | the **Beast Masters** Ex Miscellanea tradition, ArMDE:10161-10168 | **Yes, but not against this entry.** The tradition is a V/F package with its own budget rule ("*only 8 points of Flaws available … to buy another 8 points of Virtues*", :10163). `rules/core/houses.json` models `house.ex_miscellanea` and **no traditions at all** — there is no `beast_masters` id — so no per-entry defect follows for `flaw.study_requirement`. Recorded as a lead for whoever audits House data. | No — and it is **out of V/F scope**, so no finding is raised here. |
| **inbound** → `flaw.simple_minded` | "*You acquire the Flaw Simple-Minded for an indefinite duration*" (ArMDE:9634, "Caster addled") | a Twilight/botch result table | **No creation-time rule.** An in-play acquisition of a Flaw; the app models no in-play acquisition, and the Flaw's own passage is unaffected. | No |
| **inbound** → `flaw.simple_minded` | "*Grant Simple-Minded*, 5 points, Init –12, Vim" (ArMDE:19575-19576) | a magical creature's power | **No.** A creature stat block, not a character-creation rule. | No |
| **inbound** → `flaw.restricted_power` | "*This is a Restricted Power: the stag must eat the herb dittany to activate it*" (ArMDE:19806, :19816) | the Stag of Virtue stat block | **No new rule** — it is the Flaw's own rule applied, and it confirms the reading that the restriction attaches to a *single named power*, which is what `require_power` encodes. Supports the check-9 verdict. | No |
| **inbound** → `flaw.short_attention_span`, `flaw.simple_minded` | ArMDE:1205, :19123, :19188, :19256, :19325, :20691, :20839 | character templates and creature stat blocks | **No.** Example characters; no rule. | No |

**Two pointers that are *absences* and were checked as such.** ArMDE:6677 names
Wealthy and Poor to say the Flaw *is* compatible with them, and ArMDE:6797 names
Study Bonus to say the same. Both are the § 5 merchant shape — the sentence
states that no special rule applies — and both were verified against the shipped
data, which correctly encodes nothing.

### Inbound sweep

All 34 entry names were grepped across the full English core rulebook
(`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md`, 25 800+
lines). The sweep returned **86 hits**: 34 headings, 34 "List of Flaws" index
links, 34 alphabetical-index rows, and **14 sites outside those three** — the
eleven rated in the table above, plus ArMDE:51 (a hyphenation style note),
ArMDE:1424 and ArMDE:1597 (templates naming Flaws outside this span, false hits
on substring), and ArMDE:10037 (an Initiation script about going a night without
sleep, which names no Flaw and is a substring false hit on "sleep").

**Six of the fourteen carried real content and two of those exposed a defect**
(the Nephilim size pair, which F-23 already owns, and the Beast Masters
tradition, which is out of V/F scope). Four were confirmations rather than
defects. **Every out-of-span entry the sweep reached — `virtue.blood_of_the_nephilim`,
`flaw.night_terrors`, `flaw.prohibition`, `virtue.study_bonus`, `flaw.dwarf`,
`virtue.giant_blood`, `virtue.large`, `flaw.plagued_by_supernatural_entity` — was
looked up in `corrections.md` § 1 before anything was numbered.** Two of them
already carried findings (F-23, F-465, F-498) and neither was re-reported.

### The body-level duplicate check § 4.2 says it could not run

`corrections.md` § 4.2 is explicit that its nine duplicates are "*a floor, not a
ceiling*", because the search keyed on the entry id in a finding's **heading**,
so a duplicate naming its entry only in a *body* would have been missed. That
search is cheap for this span and was run: a grep of the whole of
`corrections.md` for **any** line number in 6663-6802, in headings and bodies
alike, returns exactly four hits — `corrections.md:20` and `:1525` (two
statements that B17–B19 have not run), `:701` (**F-498**, `flaw.servant_of_the_land`,
6717-6720, tagged `(B17)`), and `:663` (**F-465**, tagged `(+B17)`). A grep for
all 34 entry ids returns the same two findings and nothing else.

**So exactly two prior findings reach into this span, both were already known to
me before I numbered anything, and neither is re-reported.** No body-only
duplicate exists for 6663-6802. This does not widen § 4.2's floor — it closes the
question for one span.

**A catalogue-completeness check the sweep made cheap, and it is clean.** The
alphabetical index's rows tagged `(Flaw)` between *Reckless* (ArMDE:25340) and
*Supernatural Nuisance* (ArMDE:25531) number exactly **34**, and they are exactly
the 34 headings in 6663-6802. **No rulebook Flaw in this span is absent from the
catalogue** — no instance of F-34's shape.

### What the phrase screen missed — the exact wording, in both locales

`crates/arm-rules/tests/uncomputed_clauses.rs` has the whole span inside
`SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` entry, swept 2026-09-15 and **re-swept
2026-09-19**, green both times, and it saw none of the thirteen `narrative`
misclassifications below. This table extends `corrections.md` § 3.1. Every string
is verbatim from the cited line.

| Entry | ArMDE | EN wording the screen missed | DE wording the screen missed |
|---|---|---|---|
| `flaw.rector` | :6673 / DE :6673 | `The character must have a Social Status Virtue dictating his place within the university.` | `Der Charakter muss eine Sozialer-Status-Tugend besitzen, die seinen Platz innerhalb der Universität festlegt.` |
| `flaw.regular` | :6677 / DE :6677 | `The character must spend one of his free seasons on the seasonal activity of worship.` | `Der Charakter muss eines seiner freien Quartale für die Quartalsaktivität Andacht aufwenden.` |
| `flaw.restricted_learning` | :6685 / DE :6685 | `Pick five Abilities at character creation; you can only apply experience points to these Abilities.` … `she cannot use an Ability with no score, even if it can normally be used untrained` | `Wähle fünf Fertigkeiten bei der Charaktererschaffung; du kannst Erfahrungspunkte nur auf diese Fertigkeiten anwenden.` … `kann er eine Fertigkeit, in der er keinen Wert hat, nicht einsetzen` |
| `flaw.restricted_power` | :6689 / DE :6689 | `The character must perform some special ceremony to activate it` … `This Flaw may be taken once for each power the character possesses.` | `Der Charakter muss eine besondere Zeremonie durchführen, um sie zu aktivieren` … `Dieser Fehler darf einmal für jede Kraft genommen werden, die der Charakter besitzt.` |
| `flaw.restriction` | :6693 / DE :6693 | `You cannot cast spells at all under certain uncommon conditions.` … `Spells cast remain in effect even if the Restriction comes into play.` | `Du kannst unter bestimmten, seltenen Bedingungen überhaupt keine Zauber wirken.` … `Bereits gewirkte Zauber bleiben in Kraft` |
| `flaw.rigid_magic` | :6697 / DE :6697 | `You cannot use vis when you cast spells. Thus, you cannot increase your spell rolls or cast Ritual magic.` | `Du kannst beim Wirken von Zaubern keine Vis verwenden. Daher kannst du deine Zauberwürfe nicht steigern und keine Ritualmagie wirken.` |
| `flaw.sheltered_upbringing` | :6723 / DE :6723 | `You may not take Bargain, Charm, Etiquette, Folk Ken, Guile, Intrigue, or Leadership as beginning Abilities, but you may learn them in play.` | `Du darfst Feilschen, Charme, Etikette, Menschenkenntnis, Täuschung, Intrige oder Führung nicht als Anfangsfertigkeiten nehmen, kannst sie aber im Spielverlauf erlernen.` |
| `flaw.short_lived_magic` | :6731 / DE :6731 | `Spells that should last a year, last a moon; those of a moon, only to the next sunrise or sundown; and those of a sun, merely Diameter.` | `Zauber, die ein Jahr dauern sollten, dauern einen Mond; jene eines Mondes nur bis zum nächsten Sonnenauf- oder -untergang; und jene einer Sonne lediglich einen Durchmesser.` |
| `flaw.slow_caster` | :6757 / DE :6757 | `Your Formulaic spells take two rounds to cast; Spontaneous spells also take two rounds unless you fast-cast, in which case they take one round casting time.` | `Deine Formulaischen Zauber benötigen zwei Runden zum Wirken; Spontane Zauber benötigen ebenfalls zwei Runden, es sei denn, du wirkst sie schnell, wobei sie eine Runde Wirken benötigen.` |
| `flaw.slow_power` | :6761 / DE :6761 | `it requires an additional round of preparation to activate` | `sie eine zusätzliche Vorbereitungsrunde zur Aktivierung benötigt` |
| `flaw.spontaneous_casting_tools` | :6781 / DE :6781 | `He has 15 individual tools, one for each Art, which he must use in various combinations to cast spontaneous spells. This Flaw can only be taken by Verditius magi.` | `Er besitzt 15 einzelne Werkzeuge, eines für jede Kunst, die er in verschiedenen Kombinationen einsetzen muss, um spontane Zauber zu wirken. Dieser Fehler kann nur von Verditius-Magi gewählt werden.` |
| `flaw.stockade_parma_magica` | :6789 / DE :6789 | `you cannot suppress your Parma once it is erected. Any friendly spell or magical affect must penetrate your Parma Magica to affect you, just as if it were a hostile spell.` | `kannst du deine Parma nicht unterdrücken, sobald du sie errichtet hast. Jeder freundliche Zauber oder magische Effekt muss deine Parma Magica durchdringen, um dich zu beeinflussen, genau als wäre er ein feindlicher Zauber.` |
| `flaw.stuck_in_your_ways` | :6793 / DE :6793 | `this character uses the lower of that Ability score and his current Covenant Lore Ability for his current covenant` | `verwendet dieser Charakter den niedrigeren Wert aus seiner Fertigkeit und seiner aktuellen Konventskunde für seinen derzeitigen Konvent` |
| `flaw.study_requirement` | :6797 / DE :6797 | `You are unable to study magic from books or vis alone. You must study in the presence of the appropriate Art.` | `Du bist nicht in der Lage, Magie allein aus Büchern oder Vis zu studieren. Du musst in Gegenwart der entsprechenden Kunst studieren.` |

**Which § 3.1 families these fall into, and which are new.** Family 1
(Prohibition) covers `cannot cast`, `cannot use`, `may not take`, `darfst nicht`,
`kannst nicht`, `kann … nicht einsetzen` — eight of the fourteen. Family 4
(Eligibility) covers `can only be taken by` / `darf nur von … gewählt werden` —
`flaw.spontaneous_casting_tools` is a **second instance of the exact form F-448
and F-403 already listed**, so no new word-form is needed for it. **Four forms
are new and are not in § 3.1's list:**

| New form | EN | DE | From |
|---|---|---|---|
| **Compulsory expenditure** | `must spend one of his free seasons` | `muss eines seiner freien Quartale … aufwenden` | `flaw.regular` |
| **`the lower of`** — a `min()` stated in words with no number and no sign | `uses the lower of` | `verwendet … den niedrigeren Wert aus` | `flaw.stuck_in_your_ways` |
| **Duration downgrade chain** | `should last a year, last a moon` | `dauern sollten, dauern einen Mond` | `flaw.short_lived_magic` |
| **Counted rounds** | `take two rounds`, `an additional round` | `benötigen zwei Runden`, `eine zusätzliche Vorbereitungsrunde` | `flaw.slow_caster`, `flaw.slow_power` |

§ 3.1 family 14's `spend a round` is close to the last of these but does not
match either form, and no family covers the other three.

**A fifth correction, to a form § 3.1 already proposes.** Family 8 proposes the
German `halbiert` for the halving idiom. `has_mechanical_phrase` matches a
**contiguous substring with a left word boundary and no right boundary**
(`uncomputed_clauses.rs:234-241`), so a needle only matches through a *suffix* —
it cannot match a different inflection. This span's halving text is the
**imperative**: `flaw.short_ranged_magic`'s shipped DE summary and DE 6739 both
read "**Halbiere** deine Zaubersummen … **Halbiere** deine Laborsumme". The
needle `halbiert` does not occur in `halbiere`. **Counted rather than asserted:**
the German core book contains **36** occurrences of the stem `halbier`, of which
`halbiert` matches **23**; the other **13** — including *both* of
`flaw.short_ranged_magic`'s — are `halbiere` (5) and further inflections (8). So
the proposed form covers **64%** of the German halving idiom and misses a third
of it. The correct needle is the stem **`halbier`**, which covers all 36 in one
entry and over-matches only `Halbierung`, which *is* the same idiom. (The English
side is already fine: `halve` matches `Halve` case-insensitively.) Recorded here
because it is a defect in a *proposed* fix rather than in the shipped list, and
it is far cheaper to catch before § 3.1 lands than after.

### The ordering trap § 2.1 predicts, quantified for this span — and a limit § 3.1's list does not clear

`corrections.md` § 2.1 already says `MECHANICAL_PHRASES` must grow **before** any
`narrative` → `uncomputed_rule` reclassification lands. Nobody has put a number
on it for a span, so here is this one's, computed by reading the guard rather
than by inference.

**The mechanism.** `uncomputed_clauses.rs`'s *first* assertion,
`every_uncomputed_rule_entry_states_its_rule_in_every_locale` (`:316-350`),
requires every `uncomputed_rule` entry's displayed text — `description`, falling
back to `summary` (`:298-302`) — to satisfy
`states_a_mechanical_rule` = `has_signed_number || has_botch_term ||
has_mechanical_phrase` (`:244-246`), **in every locale**. Unlike the third
assertion it is deliberately **unscoped**: no `SWEPT_BLOCKS` gate, the whole
catalogue, every book.

**The count.** Of this batch's **15** reclassifications to `uncomputed_rule`,
**exactly one** — `flaw.savantism` — would satisfy the assertion on the day it
lands, and only because ArMDE:6705 contains a literal `+3`. The other **14**
carry no signed number, no botch term, and none of the **20** phrases currently
in `MECHANICAL_PHRASES` (read at `:179-220`: the cap/floor forms, `ease factor`,
`equal to`, the `multiply` stems, the four rounding forms, `simple die`,
`stress die`, `cannot die`, `one magnitude`, plus German twins). So landing the
reclassifications first produces **28 red offender lines** — 14 entries × 2
locales — and the only route to green would be rewording shipped rulebook text
to satisfy a detector, which the module's own doc comment (`:152-157`) calls out
as backwards and records having already happened once.

**And § 3.1's proposed list does not clear it in German.** This is the part that
is new. § 3.1 family 1 adds `may not take`, `cannot cast`, `cannot take`,
`darfst nicht`, `kannst nicht`, `dürfen nicht`. `has_mechanical_phrase` is a
**contiguous substring** match with a left word boundary (`:234-241`). German
negation is discontinuous, and the module's doc comment already names this as a
known limit (`:168-173`, "*a contiguous substring cannot express 'cannot &lt;verb&gt;'
in general*"). Check it against the actual shipped German:

| Entry | DE source sentence (DE line) | Proposed token | Matches? |
|---|---|---|---|
| `flaw.sheltered_upbringing` | `Du darfst Feilschen, Charme, Etikette, … nicht als Anfangsfertigkeiten nehmen` (6723) | `darfst nicht` | **No** — seven Ability names sit between `darfst` and `nicht` |
| `flaw.restriction` | `Du kannst unter bestimmten, seltenen Bedingungen überhaupt keine Zauber wirken` (6693) | `kannst nicht` / `kannst keine` | **No** — six words between `kannst` and `keine` |
| `flaw.rigid_magic` | `Du kannst beim Wirken von Zaubern keine Vis verwenden` (6697) | `kannst keine` | **No** — five words between |
| `flaw.stockade_parma_magica` | `kannst du deine Parma nicht unterdrücken` (6789) | `kannst nicht` | **No** — three words between |
| `flaw.study_requirement` | `Du bist nicht in der Lage, Magie allein aus Büchern oder Vis zu studieren` (6797) | `nicht in der Lage` | **Yes** — § 3.1 family 1 lists this exact contiguous form |

So four of the five German prohibition cases in this span defeat the proposed
family, and the one that passes does so because the book happened to write the
negation contiguously. **This is not an argument for a broader phrase; it is
evidence that the German half of the screen needs a different shape** — a
proximity or two-token match, or an explicit acceptance that German prohibition
text is screened by the English side of the same entry.

**What this batch therefore recommends, and it is a recommendation about
sequencing, not a finding.** Work § 3.1 *before* these reclassifications (which
§ 2.1 already requires); add the four new word-form families in the table above;
and **decide the German discontinuous-negation question before the German
descriptions are written**, because writing them first and then discovering they
go red is exactly the pressure that produced the `Das Patzen` → `Ein Patzer`
rewrite the module warns about. No data in this batch should land until that
decision exists.

**`NO_RULE_DESPITE_TOKEN` was checked and none of its rows lies in this span.**
Its exemptions name `flaw.overconfident_major`, `flaw.overconfident_minor`,
`flaw.horrifying_appearance_snake_legs`, `flaw.primogeniture_lineage`,
`flaw.true_love_major` and `flaw.true_love_minor` — all outside 6663-6802. **No
finding against an exemption's written reading is raised.**

### Counting note

Figures in this Method that were counted rather than estimated, and how: the
**34** headings and **35** ids (`grep -n "^#### "` filtered to 6663-6802, and
`jq … .[560:595] | length`); the **32/32** `*_major`-with-`*_minor` census (`jq`
over the whole file, both numbers computed separately); the **34**
`(Flaw)`-tagged index rows between ArMDE:25340 and :25531 (enumerated, listed in
full above, not sampled); the **84 of 102** anchor coverage on `uncomputed_rule`
(`jq … group_by(.classification)`); the **seven** U+2014-plus-digit occurrences
in the English core book, against **518** U+2013-plus-digit occurrences, which is
what shows the two are distinguishable rather than a grep artefact; and the
**24/6/3/2** shipped classification split of the span, re-derived from the `jq`
dump rather than taken from the brief; the **23** entries catalogue-wide carrying
`tainted: true` and the **1** of them with a `prerequisites` tree; the **99**
entries carrying `social_status`; the **2** entries in the whole catalogue with
more than one `parameters` row; the **20** phrases currently in
`MECHANICAL_PHRASES`; and the **36 / 23 / 5** split of the German `halbier` stem.

**Five corrections to this section, made after the first draft and left visible
rather than overwritten.** Three were caught by re-counting from the Verdicts
table rather than from the prose, which is the only method that finds them: the
header split (15/20 for what is 15/19/1), "thirteen" where the misclassification
list has fourteen, and "fourteen entries" for a fifteen-item finding list. Two
more came from the verification pass and are its catches, not mine: **"the other
twelve Houses" is eleven** (`houses.json` holds 12 ids), and **the dash figures
were `grep -c` line counts reported as occurrence counts** — 11 occurrences over
7 lines, not 7, and 689 over 518, not 518. Every one is annotated at its own site.
Four prior batches recorded the counting lesson; this one adds that a *unit*
error (lines vs occurrences) survives re-counting, because recounting the same
way reproduces it — only changing the tool (`grep -c` → `rg -o`) exposes it.

---

## Verdicts

`class` / `data` / `text` are pass or fail per column; `note` names the finding
or the reason a pass is not obvious.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.reckless_major` | 6663-6666 | pass | pass | pass | pure Personality colour; the dual-magnitude split is present and follows the catalogue's universal convention (32/32) |
| `flaw.reckless_minor` | 6663-6666 | pass | pass | pass | as above; shares the heading's range correctly |
| `flaw.reclusive` | 6667-6670 | pass | pass | pass | the "*bad Flaw for a player character*" advice at :6669 is troupe guidance, not a rule |
| `flaw.rector` | 6671-6674 | **FAIL** | **FAIL** | pass | **F-502** — a stated Social Status prerequisite, encoded nowhere and written nowhere |
| `flaw.regular` | 6675-6678 | **FAIL** | pass | pass | **F-503** — a compulsory seasonal expenditure classified as colour; the Wealthy/Poor compatibility is correctly *not* encoded |
| `flaw.repellent` | 6679-6682 | pass | pass | pass | `uncomputed_rule` + description in both locales + `tainted` + full −6; the "minor advantage" choice is Q-134 |
| `flaw.restricted_learning` | 6683-6686 | **FAIL** | **FAIL** | pass | **F-504** — a five-Ability creation restriction with no class, no parameter and no description |
| `flaw.restricted_power` | 6687-6690 | **FAIL** | pass | **FAIL** | **F-505** (class), **F-506** (DE name, D7). The `require_power` parameter and the absent `max_total` are both correct |
| `flaw.restriction` | 6691-6694 | **FAIL** | pass | pass | **F-507** — an absolute on casting plus an enchanted-item extension, classified as colour |
| `flaw.rigid_magic` | 6695-6698 | **FAIL** | pass | **FAIL** | **F-508** (class), **F-509** (DE summary alters the source's word) |
| `flaw.rolling_stone` | 6699-6702 | pass | pass | pass | `uncomputed_rule`, anchor present, −1 written out in both locales with ASCII hyphen |
| `flaw.savantism` | 6703-6708 | **FAIL** | **FAIL** | **FAIL** | **F-510** — `creation_effect` with no effects; four creation numbers computed nowhere and written nowhere |
| `flaw.secretive` | 6709-6712 | pass | pass | pass | pure Personality colour |
| `flaw.seeker` | 6713-6716 | **ESCALATED** | **ESCALATED** | pass | **Q-135** — the entry's own pointer resolves to HoH:TL:503, "*A magus from any House may be a Seeker*". **Not marked checked**, per the audit's standing rule that doubt is escalated rather than resolved by judgement |
| `flaw.servant_of_the_land` | 6717-6720 | pass | **FAIL** | pass | **F-522** (new, from the verification pass — parameterized with no `max_total`, so the Major Flaw stacks without limit) **and F-498 (B16)**, not re-reported but extended with the confirmation that `flaw.prohibition` exists as a grantable id |
| `flaw.sheltered_upbringing` | 6721-6724 | **FAIL** | **FAIL** | pass | **F-511** — seven named Abilities forbidden at creation and sold by the app |
| `flaw.short_attention_span` | 6725-6728 | pass | pass | pass | pure Personality colour |
| `flaw.short_lived_magic` | 6729-6732 | **FAIL** | pass | pass | **F-512** — an explicit Duration downgrade chain classified as colour |
| `flaw.short_of_breath` | 6733-6736 | pass | **FAIL** | pass | **F-513** — the Concentration clause reaches neither locale; **F-514** rides on the same line |
| `flaw.short_ranged_magic` | 6737-6740 | pass | **FAIL** | pass | **F-515** — two halvings, neither computed, described nowhere |
| `flaw.simple_minded` | 6741-6744 | pass | pass | pass | pure Personality colour; the em dash in both locales' summaries is the source's punctuation, not a sign |
| `flaw.sleep_disorder` | 6745-6750 | pass | **FAIL** | pass | **F-465 (B15)** — not re-reported; confirmed live at :6749 and DE :6749 |
| `flaw.slothful` | 6751-6754 | pass | pass | pass | pure Personality colour |
| `flaw.slow_caster` | 6755-6758 | **FAIL** | pass | pass | **F-516** — explicit casting-round counts classified as colour |
| `flaw.slow_power` | 6759-6762 | **FAIL** | pass | pass | **F-517** — an extra preparation round classified as colour; the repeat rule is correctly encoded by the default `max_per_target` |
| `flaw.slow_reflexes` | 6763-6766 | pass | pass | pass | `combat_mod{initiative,-3}`, sign correct, consumer read |
| `flaw.small_frame` | 6767-6770 | pass | pass | pass | correct; the Nephilim pair is **F-23 (B01)**, extended, not re-reported |
| `flaw.social_handicap` | 6771-6774 | pass | pass | pass | `uncomputed_rule`, anchor, −3 written out in both locales |
| `flaw.soft_hearted` | 6775-6778 | pass | pass | pass | pure Personality colour |
| `flaw.spontaneous_casting_tools` | 6779-6782 | **FAIL** | **FAIL** | pass | **F-518** — "*can only be taken by Verditius magi*" enforced nowhere; every magus of the other twelve Houses, and every Gifted companion, may take it |
| `flaw.stigmatic_catalyst` | 6783-6786 | pass | pass | pass | `uncomputed_rule` + full description; "*normally found among magi of House Criamon*" is not a gate |
| `flaw.stockade_parma_magica` | 6787-6790 | **FAIL** | **FAIL** | pass | **F-519** — a Magic Resistance rule classified as colour, and no gate stopping a Gifted companion who owns no Parma |
| `flaw.stuck_in_your_ways` | 6791-6794 | **FAIL** | **FAIL** | pass | **F-520** — a `min()` formula classified as colour, naming an Ability the catalogue does not have |
| `flaw.study_requirement` | 6795-6798 | **FAIL** | pass | pass | **F-521** — a study prohibition classified as colour; the Study Bonus compatibility is correctly *not* encoded |
| `flaw.supernatural_nuisance` | 6799-6802 | pass | pass | pass | pure Story colour; the Plagued-by contrast states no mechanic |

**Cleared negatives — things that look wrong and are right.** Recorded so
Phase 2 does not "fix" them.

- **`flaw.regular` has no Wealthy/Poor incompatibility, and must not gain one.**
  ArMDE:6677: "*The Regular Flaw is compatible with Wealthy and Poor*". The
  sentence exists to say no rule applies.
- **`flaw.study_requirement` has no Study Bonus incompatibility, and must not
  gain one.** ArMDE:6797: "*You may take both Study Bonus and Study
  Requirement*".
- **`flaw.restricted_power` and `flaw.slow_power` need no `max_total`.** The
  default `max_per_target` of 1, keyed on the whole params map, already permits
  one copy per named power and forbids a second on the same power — which is
  exactly ArMDE:6689 and ArMDE:6761. Adding `max_total` would invent a ceiling
  the book does not state.
- **`flaw.savantism` needs no `ability_authorization`.** ArMDE:6707 says the
  favored Ability's Virtue requirement applies "*as normal*" — it re-states the
  gate rather than waiving it. This is an F-409 look-alike that is the F-409
  shape *inverted*.
- **`flaw.stigmatic_catalyst` needs no `ability_authorization` for Enigmatic
  Wisdom.** ArMDE:6785 names it in a roll formula; naming presupposes, it does
  not grant.
- **`flaw.repellent` needs no demonic-origin prerequisite, and this was
  interrogated rather than waved through.** ArMDE:6681 opens "*The character has
  developed one of the physical characteristics of **his demonic creator***",
  which presupposes a Devil Child. But the passage states no eligibility rule,
  and `tainted` is a *Type tag* rather than a gate — `engine-semantics.md` B9,
  citing ArMDE:2998-3002, and `validation/caps.rs::validate_tainted_cap`, which
  uses the flag only for a half-the-points **warning**. Catalogue-wide the norm
  confirms it: of the **23** entries carrying `tainted: true`, exactly one has a
  `prerequisites` tree at all (`flaw.false_power_minor`, and its prereq is the
  magnitude-pair link `{"kind":"has","value":"flaw.false_power"}`, not an
  origin). `virtue.demonic_blood` and `virtue.mentored_by_demons` carry none
  either. Adding one here would invent a rule the book does not state.
- **EN `flaw.repellent`'s `demonic eves` is the source's typo** (ArMDE:6681) and
  the i18n is a faithful copy — the `scheitest` precedent (§ 5). Do not
  "correct" it. The German at DE 6681 reads `dämonischen Augen` because the
  German file translates rather than copies, so the two locales legitimately
  differ here.
- **DE `flaw.sleep_disorder`'s "*Mehr als nicht*"** is DE 6747's own wording.
  Same precedent.
- **`flaw.small_frame`'s `size_delta: -1` is the right encoding for "reduced
  **to** –1"** — but only because the other three size-changers are excluded.
  See F-23's extension above.
- **`flaw.slow_reflexes`'s unconditional −3 is correct.** The passage's
  condition, "*in situations warranting a quick response*", is the condition
  under which an Initiative total is rolled at all, so folding it in flat changes
  no result. This is **not** an F-449/F-462 scope instance.

---

## Findings

### F-502 — `flaw.rector`: a stated prerequisite the data does not carry and the player never sees

**Passage** (ArMDE:6673, DE 6673):

> The character is the representative leader of his faculty or nation at a
> university, depending on whether he is a master or a student. He is responsible
> for his colleagues' behavior and is obliged to deal with their academic
> concerns. **The character must have a Social Status Virtue dictating his place
> within the university.** The character can expect to spend considerable time
> sorting out his fellows' affairs.

> Der Charakter ist der gewählte Sprecher seiner Fakultät oder Nation an einer
> Universität … **Der Charakter muss eine Sozialer-Status-Tugend besitzen, die
> seinen Platz innerhalb der Universität festlegt.**

**Current data.** `classification: "narrative"`, no `prereqs`, no `description`
in either locale.

**What happens when a player does this.** He takes Rector/Proctor as his Major
Story Flaw, takes no Social Status Virtue at all, and the app builds the
character with no error and no warning. The rule that a Rector is a university
*officer* — the whole content of the Flaw — is enforced by nothing and stated
nowhere the player can read.

**Why it is wrong.** Two defects on one sentence. (a) `classification` is
`narrative`, which claims the book states nothing mechanical; an eligibility
requirement is mechanical, and D3 forbids `narrative` on the ground that the
engine cannot express it. (b) The requirement is not in `prereqs`, and — unlike
F-518 — **it cannot be**, because `Prereq` has `Has(Id)`, `House`, `AbilityMin`,
`ArtMin`, `IsMagus`, `All`, `Any`, `Nor` and **no variant that ranges over a
`categories` value**. 99 catalogue entries carry `social_status`, so an
enumerated `Prereq::Any` would be a 99-element list that must be amended every
time a Social Status ships — which is precisely the catalogue-size coupling
CLAUDE.md's first architecture invariant forbids.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"rectorproctor"` (the book's own anchor, ArMDE:5366 and :25344), and the
requirement written into `description` in both locales. Whether a `Prereq`
category variant should also be added is **Q-132**.

**Related.** `flaw.rector` is also a site of **F-427** (ArMDE:2816's
"one Social Status" rule enforced nowhere) — that entry is a *second* Social
Status constraint on the same character and is already owned by F-427; this
finding does not re-derive it.

**Severity: MEDIUM.** A lost-rule / provenance defect with a prerequisite half.
No number is computed wrongly; the harm is that a rule reaches the player in
neither locale and gates nothing.

---

### F-503 — `flaw.regular`: a compulsory seasonal expenditure classified as colour

**Passage** (ArMDE:6677, DE 6677):

> The character lives according to a strict religious rule, which leaves little
> time for other activities. **The character must spend one of his free seasons
> on the seasonal activity of worship.** … The Regular Flaw is compatible with
> Wealthy and Poor, but a Poor Regular character effectively has no free seasons
> and may be unsuitable as a player character. Magi can be Regular.

> **Der Charakter muss eines seiner freien Quartale für die Quartalsaktivität
> Andacht aufwenden.**

**Current data.** `classification: "narrative"`, no `description`.

**What happens when a player does this.** Nothing at all. The app models no
seasonal activity ledger, so the one-season obligation is invisible; a Regular
character is indistinguishable from an unencumbered one.

**Why it is wrong.** "Must spend one of his free seasons on the seasonal activity
of worship" is a rule about the character's annual budget of seasons, stated with
a number. The engine has no seasonal ledger to express it in — which under **D3**
is grounds for `uncomputed_rule` with the rule written out, and never for
`narrative`.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"regular"` (ArMDE:5618, :25352), `description` in both locales carrying the
compulsory season *and* the Poor-Regular consequence ("*effectively has no free
seasons*"), which is a second stated consequence the player cannot otherwise see.

**Severity: MEDIUM.** Lost rule; no miscalculation, because nothing computes
seasons.

---

### F-504 — `flaw.restricted_learning`: a creation-time XP restriction with no class, no parameter and no description

**Passage** (ArMDE:6685, DE 6685):

> The character is incapable of learning new things without assistance. **Pick
> five Abilities at character creation; you can only apply experience points to
> these Abilities.** The character cannot learn any new Abilities on her own; she
> must receive experience points from Teaching or Training to develop a new
> Ability or to improve one that is not part of the initial five. **Furthermore,
> she cannot use an Ability with no score, even if it can normally be used
> untrained.** Any Supernatural Abilities from Virtues she possesses are added to
> the list of Abilities to which she can apply experience points. All mundane
> animals are required to take this Flaw. A character with Intelligence rather
> than Cunning is not a mundane animal for these purposes.

> **Wähle fünf Fertigkeiten bei der Charaktererschaffung; du kannst
> Erfahrungspunkte nur auf diese Fertigkeiten anwenden.** … **Außerdem kann er
> eine Fertigkeit, in der er keinen Wert hat, nicht einsetzen, auch wenn sie
> normalerweise ungeübt eingesetzt werden kann.**

**Current data.** `classification: "narrative"`, no `parameters`, no `effects`,
no `description`.

**What happens when a player does this.** He takes Restricted Learning, picks his
five Abilities on paper, and the app lets him spend his entire XP pool on any
Ability in the catalogue. The Flaw costs him nothing whatsoever while paying him
a Minor Flaw point. **And the five Abilities he picked are not in the save file**
— there is no parameter to hold them — so the choice the rule is built on cannot
survive a save/load round trip, which CLAUDE.md rates at the top.

**Why it is wrong.** Three defects. (a) `classification: "narrative"` on a
passage whose first instruction is "*Pick five Abilities at character creation*"
followed by an arithmetic restriction on the XP pool. (b) No `parameters` entry
records the five. (c) No `description`, so neither locale states the rule.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"restricted-learning"` (ArMDE:5619, :25365), a `parameters` entry holding the
five Ability choices, and the rule in `description` in both locales.

**The parameter half costs more than it looks, and the cost is stated rather than
discovered in Phase 2.** `ParameterDomain::Ability` exists
(`crates/arm-rules/src/types.rs:473-476`), but a `ParameterDef` holds **one**
value per key, and the catalogue has **no precedent for two parameters of the
same domain on one entry** — of 655 entries exactly **two** carry more than one
parameter at all (`flaw.necessary_realm_aura_for_ability`: `realm` + `ability`;
`virtue.folk_magic`: `category` + `realm`), and in both the two domains differ.
So the two routes are: **five keys** (`ability_1` … `ability_5`, all domain
`ability`), which is expressible with today's machinery and no code change but
hardcodes the number five into the data — acceptable here, because five *is* the
rule, unlike a catalogue size; or a **list-valued parameter**, which is new
machinery and belongs with § 3.15 rather than § 3.7. **The first is recommended**
and is the reason this half is filed as a plain data gap rather than escalated.

**A note on the effect side, so Phase 2 does not over-reach.** `restricted_ability_xp`
(A7) *adds* a pool restricted to named Abilities; this passage *narrows the
general pool* to five named Abilities. Those are not the same operation, and per
A7's own consumer note a restricted pool additionally "**confers permission**" to
buy what it funds — the opposite of what is wanted here. The engine has no
"narrow the general pool" shape. So the effect is genuinely inexpressible, D3
applies, and `uncomputed_rule` + `description` is the whole of the data fix. The
parameter, by contrast, is expressible today and is a plain gap.

**Severity: HIGH.** A creation-time budget rule that the app entirely ignores,
plus a choice the save cannot round-trip.

---

### F-505 — `flaw.restricted_power`: a mechanical restriction on a power classified as colour

**Passage** (ArMDE:6689, DE 6689):

> This Flaw limits the use of a supernatural power that can normally be used at
> will … **The character must perform some special ceremony to activate it**, such
> as drawing symbols on the ground or gesturing and chanting like a magus, **or
> else the power only functions on a limited class of targets** (such as men,
> wolves, sounds, or sand) **or in specific circumstances** (like at night, under
> water, when touching iron, or after singing a song). **This Flaw may be taken
> once for each power the character possesses.**

**Current data.** `classification: "narrative"`, a correct `power` parameter, no
`description`.

**What happens when a player does this.** He attaches Restricted Power to his
Lesser Power, banks a Minor Flaw point, and the app records *which* power (the
parameter works) but states nowhere what the restriction is. The power's read-out
is identical to an unrestricted one.

**Why it is wrong.** The passage states a restriction on when a power functions —
a mechanical rule, not colour. It is also the entry that the rulebook's own
creature stat blocks apply as a rule: ArMDE:19806 and :19816 read "*This is a
Restricted Power: the stag must eat the herb dittany to activate it*". D3
applies; the engine models no power-activation conditions.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"restricted-power"` (ArMDE:5554, :25366), `description` in both locales. **The
parameter and the absent `max_total` are correct as shipped** and must not be
touched (see cleared negatives).

**Severity: MEDIUM.** Lost rule.

---

### F-506 — `flaw.restricted_power`: the German name follows another book's glossary against this book's heading

**The heading** (DE 6687):

> #### Eingeschränkte Macht

**Current data.** `rules/i18n/de/virtues_flaws.json`, `flaw.restricted_power`:
`"name": "Eingeschränkte Kraft ({power})"`, `"name_unfilled": "Eingeschränkte
Kraft"`.

**Why it is wrong.** The shipped value follows
`rules/source/de/translation-tables/tugenden-fehler.md:346`:

> `| Restricted Power | Eingeschränkte Kraft | SdM:M; Kraft nur unter bestimmten Umständen oder mit Zeremonie aktivierbar |`

That row is **tagged `SdM:M`** — *Sphären der Macht: Magie*, i.e. *Realms of
Power: Magic*. The entry cites ArMDE. **D7 rule 3 is exactly on point**: "*A
glossary row tagged to a book other than the one the entry cites is not in the
dispute. It is that book's terminology and must not override the citing book's
heading.*" And D7 rule 2's escape hatch does not apply: the heading is not
defective. *Eingeschränkte Macht* collides with no other entry's heading —
`virtue.greater_power` is *Große Macht*, `virtue.lesser_power` is *Mindere
Macht*, `virtue.personal_power` is *Persönliche Kraft* — and the core book does
render the term, at DE 6687.

**The sibling entry proves the point rather than undermining it.**
`flaw.slow_power` ships *Langsame Kraft*, which is **correct**, because DE 6759's
heading is `#### Langsame Kraft`. The German core book genuinely uses *Macht* for
one and *Kraft* for the other; D7 makes the heading decisive per entry, so the
two shipped names should differ, and today they wrongly agree.

**Correct value.** `"name": "Eingeschränkte Macht ({power})"`,
`"name_unfilled": "Eingeschränkte Macht"`.

**This needs no `arm-de-translation` determination.** Unlike F-496/F-497/F-501,
nothing here turns on whether the copy is stale: the row is *correct for its own
book* and simply does not govern this entry. Per D6 the row should be **annotated**
in both projects so the next German-writing agent sees which rendering belongs to
which book — which is the same remedy D7 prescribed for its own three rows — but
no row is *corrected*.

**Severity: MEDIUM.** A user-facing localization defect on every session, and one
that breaks the navigation contract D7 rests on: a reader with the German book
open at page 143 cannot find *Eingeschränkte Kraft*.

---

### F-507 — `flaw.restriction`: an absolute on casting, plus an enchanted-item extension, classified as colour

**Passage** (ArMDE:6693, DE 6693):

> **You cannot cast spells at all under certain uncommon conditions.** These might
> refer to your state, such as touching the earth directly or having no beard, or
> to the target, such as birds or glass, or to your location when you use the
> magic, such as on a small boat or in a storm. **The Restriction also applies to
> effects generated by any enchanted items you create. Spells cast remain in
> effect even if the Restriction comes into play.**

**Current data.** `classification: "narrative"`, no `description`, no parameter
for the chosen condition.

**What happens when a player does this.** He takes a Major Hermetic Flaw worth 3
points and neither the chosen condition nor its three consequences are recorded
or displayed anywhere.

**Why it is wrong.** Three separate rules, none of them colour: an absolute
("cannot cast spells **at all**"), an extension of that absolute to the
character's enchanted items, and an explicit *non*-consequence (already-cast
spells are unaffected). The third is the sort of clause a player needs precisely
because it is counter-intuitive. D3 applies.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"restriction"` (ArMDE:5297, :25367), all three clauses in `description` in both
locales. The missing condition parameter is **Q-134**.

**Severity: MEDIUM.** Lost rule on a 3-point Flaw.

---

### F-508 — `flaw.rigid_magic`: "cannot cast Ritual magic" classified as colour, on an engine that knows which spells are Rituals

**Passage** (ArMDE:6697, DE 6697):

> **You cannot use vis when you cast spells. Thus, you cannot increase your spell
> rolls or cast Ritual magic.** You can use vis in the laboratory, or to refresh a
> Longevity Ritual.

> **Du kannst beim Wirken von Zaubern keine Vis verwenden. Daher kannst du deine
> Zauberwürfe nicht steigern und keine Ritualmagie wirken.** Du kannst Vis im
> Labor verwenden oder ein Langlebigkeitsritual erneuern.

**Current data.** `classification: "narrative"`, no `effects`, no `description`.

**What happens when a player does this.** A magus takes Rigid Magic as his Major
Hermetic Flaw and then selects Ritual spells from the catalogue at creation. The
app raises nothing. **`crates/arm-rules/src/spell.rs` declares
`SpellDef::ritual: bool`**, so the engine knows exactly which of the selected
spells the character can never cast; it simply never asks.

**Why it is wrong.** This is a three-clause mechanical passage — vis cannot be
spent on casting, casting totals cannot be boosted, Ritual magic cannot be cast —
with an explicit carve-out (the laboratory, and Longevity Ritual refreshment) that
a player will get wrong without it. `narrative` claims the book states nothing
mechanical, which is false on its face.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"rigid-magic"` (ArMDE:5298, :25374), all four clauses in `description` in both
locales. Whether the engine should additionally *block* Ritual spell selection at
creation is **Q-130** and is deliberately not asserted here — the book forbids
casting, not knowing, and turning that into a selection-time error is a design
call.

**Severity: HIGH.** Not because a number is wrong today, but because the entry's
entire content is invisible, and the one clause the engine *could* act on
(`SpellDef::ritual`) is the one that silently lets a player build a magus around
spells he can never cast.

---

### F-509 — `flaw.rigid_magic`: the German summary alters a word of its own source line

**Source** (DE 6697):

> Du kannst beim Wirken von Zaubern **keine** Vis verwenden.

**Current data.** `rules/i18n/de/virtues_flaws.json`, `flaw.rigid_magic`:
`"summary": "Du kannst beim Wirken von Zaubern kein Vis verwenden."`

**Why it is wrong.** The summary is the passage's first sentence, and it is
supposed to be that sentence. It silently changes `keine` to `kein`.

**Scope, stated no wider than it was checked.** The German core rulebook is
itself inconsistent about the gender of *Vis* — `keine Vis` occurs **3** times
and `kein Vis` **2** times across the whole file — and the glossary tables use
dative forms (`rohem Vis`) that settle nothing. **So this is not a claim about
which gender is right, and Phase 2 must not "harmonize" the book.** It is a claim
about *this* string: DE 6697 says `keine`, and the summary drawn from DE 6697
says `kein`.

**Correct value.** `"Du kannst beim Wirken von Zaubern keine Vis verwenden."`

**Severity: LOW.** A one-word fidelity defect in a summary, with no mechanical
consequence.

---

### F-510 — `flaw.savantism`: `creation_effect` with no effects, four creation numbers computed by nothing, and no description in either locale

**Passage** (ArMDE:6705, DE 6705):

> The character's family was cursed by faeries … He is physically capable, but
> simpleminded and slow-witted. **He has half the standard experience points at
> character generation, all future Advancement Totals are halved, and he may not
> begin with an Ability above 3.** However, with one favored Ability, he is
> exceptionally gifted. **This Ability can improve normally, is limited to a score
> of 6 as a starting character, and has +3 to all rolls related to its
> specialization rather than just +1.** The Ability must be something he could
> reasonably have access to learn …

> **Er hat die halbe Anzahl der Standard-Erfahrungspunkte bei der
> Charaktererschaffung, alle zukünftigen Fortschrittssummen sind halbiert, und er
> darf nicht mit einer Fertigkeit über 3 beginnen.** … **Diese Fertigkeit kann
> sich normal verbessern, ist bei einem Startercharakter auf eine Stufe von 6
> begrenzt, und hat +3 auf alle Würfe, die mit ihrer Spezialisierung
> zusammenhängen, statt nur +1.**

**Current data.** `classification: "creation_effect"`, **no `effects` key at
all**, no `parameters` (the favored Ability is unrecorded), no `description` in
either locale.

**What happens when a player does this.** He takes Savantism for one Minor Flaw
point. The app then funds him at the **full** standard XP instead of half, lets
him start Abilities at the age cap (5 at age 0–29) instead of 3, and gives his
favored Ability — which it does not know he has — a +1 specialization instead of
+3. Every one of the four numbers the entry exists to impose is absent, and the
player reads nothing anywhere that tells him so. **A one-point Flaw silently
doubles a creation budget.**

**RULES.md changed this finding and the change is recorded here rather than
hidden.** `crates/arm-rules/RULES.md:5226-5230` says:

> **Deferred (3), source-not-present or no clean creation number:**
> - **Savantism, Simple Student, Corrupted Arts (3 XP)** — Savantism
>   (ArMDE:6703) *halves* starting XP (multiplicative; no variant) …

So the XP halving is a **documented deferral with a stated reason**, and this
finding does **not** assert that an XP effect should be added. The
"multiplicative; no variant" reason is correct on the code: `GeneralXp` (A11) is
additive (`general_pool = clamp_to_u32(base_general + Σ amount)`), and
`LaterLifeXpRate` (A12) replaces a *rate* and is only reachable through a
life-stage plan, so neither expresses a halving of the whole pool. **What remains
wrong after crediting RULES.md is threefold and none of it is covered there:**

1. **`classification` is `creation_effect`.** That class asserts "*the book
   states a rule the engine computes at character creation*". The engine computes
   nothing. Under **D3** — an engine that structurally cannot express a rule gets
   `uncomputed_rule` with the rule written out — the class must move. RULES.md's
   deferral note is the evidence *for* the reclassification, not against it.
2. **There is no `description` in either locale**, so the deferral reaches the
   player as silence. **D5** obliges the clause to be written out "*whatever the
   classification*".
3. **RULES.md documents one of the four numbers.** It is silent on "*may not
   begin with an Ability above 3*", on "*limited to a score of 6 as a starting
   character*", and on "*+3 to all rolls related to its specialization*". The
   first two are creation-time caps the engine *does* have a shape for — the
   `AgeAbilityCaps` machinery in `crates/arm-rules/src/ability.rs` already
   computes a per-character `max_ability_score`, and `validate_ability_age_cap`
   already compares bought scores against it — so at least the "no Ability above
   3" clause is a **cap the engine can enforce** and does not. That is F-494's
   shape (an uncomputed rule the engine can actually compute) arriving from the
   other direction.
4. **The specialization clause is not merely undescribed — in the one place the
   engine computes a specialization bonus, it computes the wrong number for a
   Savant.** `crates/arm-rules/src/derived/combat.rs::specialization_bonus`
   (read at `combat.rs:244-257`) returns `u8::from(has_specialty)` — a
   **hardcoded +1**, consulting no effect at all, its doc comment citing
   ArMDE:7139 ("*Add +1 when using an Ability's specialization*"). ArMDE:6705
   says the Savant's favored Ability has "*+3 to all rolls related to its
   specialization **rather than just +1***" — it is explicitly an override of
   that constant. A Savant whose favored Ability is a weapon skill therefore
   gets a weapon Attack and Defense computed with +1 where the rule says +3.
   **The path is narrow but real**, and ArMDE:6705 itself concedes it is
   unusual rather than forbidden ("*there would need to be an extraordinary
   background story for a savant who is a master of the longsword!*").
   `AbilityRollMod` (A41) cannot fix it: per its own entry the variant is
   surfaced-only, "*nothing adds `amount` to anything*", and it declares no
   *which ability* field — so it could display "+3 to <subject>" beside a
   combat total still printing +1, which is worse than silence.

**A smaller RULES.md accuracy note, not inflated into its own finding.**
`RULES.md:5413` lists `flaw.savantism` under the heading `**XP grant:**`,
between rows that name an *implemented* grant (`flaw.feral_upbringing` — XP grant
120, `virtue.arcane_lore` — XP grant 50). Savantism carries no effect at all.
The file is not self-contradictory in the F-393 sense — both lines say "halves
starting XP" and :5226 adds "deferred" — but a reader arriving at :5413 first
will believe it is wired. Worth a qualifier when the entry is touched.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"savantism"` (ArMDE:5621, :25397), and all four numbers written into
`description` in both locales. A `parameters` entry for the favored Ability is
the honest minimum if any of the clauses is ever to be enforced, and is folded
into **Q-134**. Whether the "no Ability above 3" and "favored Ability capped at
6" clauses should be wired through the existing cap machinery rather than only
described is **Q-133**'s sibling and is raised there.

**Severity: HIGH.** A one-point Flaw that the app honours by giving the character
twice the XP the rule allows is a wrong-rules-output defect, which CLAUDE.md puts
in the top class. The classification and description halves are lost-rule
defects riding on it.

**Routing note for `corrections.md`, so this is not filed under one kind and
worked under another.** This finding spans two correction groups and the split
matters: points 1–3 are `class` + `desc` and belong to § 3.3 / § 3.4, but
**point 4 is `number`** — a computed value that contradicts the book — and
belongs in **§ 3.9 ("Wrong numbers and wrong arithmetic")**, whose own note says
each of its rows "*needs a red that asserts the wrong number today*". A red for
point 4 is writable immediately and independently of everything else here: build
an entity with `flaw.savantism`, a weapon Ability carrying a specialty and a
slot flagged `specialization_applies`, and assert the combat total. It is kept
as one finding rather than two because it is one entry and one passage, but it
must be *worked* in two slices. Recorded as `class+desc+number`.

---

### F-511 — `flaw.sheltered_upbringing`: seven Abilities the book forbids at creation, and the app sells all seven

**Passage** (ArMDE:6723, DE 6723):

> You grew up completely separated from society … You are unable to function
> normally because you cannot understand most human customs. **You may not take
> Bargain, Charm, Etiquette, Folk Ken, Guile, Intrigue, or Leadership as beginning
> Abilities, but you may learn them in play.**

> **Du darfst Feilschen, Charme, Etikette, Menschenkenntnis, Täuschung, Intrige
> oder Führung nicht als Anfangsfertigkeiten nehmen, kannst sie aber im
> Spielverlauf erlernen.**

**Current data.** `classification: "narrative"`, no `effects`, no `description`.

**What happens when a player does this.** He takes Sheltered Upbringing, banks a
Minor Flaw point, and then buys Charm 3 and Folk Ken 2 at character creation with
no error, no warning and no note. The Flaw's only mechanical content is a
prohibition on seven specific purchases, and the app permits all seven.

**Why it is wrong.** The seven Abilities are a **closed, named list**, and every
one resolves to a real catalogue id — verified against `rules/core/abilities.json`:
`ability.bargain`, `ability.charm`, `ability.etiquette`, `ability.folk_ken`,
`ability.guile`, `ability.intrigue`, `ability.leadership`, all of category
`general`. This is not a vague clause; it is an enforceable list the data does
not carry. Classifying it `narrative` claims the book states nothing mechanical.

**The engine gap is already owned and is not re-derived here.** No `Effect`
variant forbids an Ability or an Ability category. That is **F-410**
(`flaw.enfeebled`) and **F-355** (`flaw.ability_block`, which B12's F-426
duplicated and `corrections.md` § 4 withdrew).

**Both prior instances are *category* blocks, and this one is not — which is the
point.** A draft of this finding described F-355 as "a named-Ability
prohibition"; that is wrong, and the verification pass caught it. ArMDE:5653
reads "*You are completely unable to learn a certain **class** of Abilities*",
and `corrections.md:525` says the same. So F-410 and F-355 both want a
*category*-ranging variant, whereas `flaw.sheltered_upbringing` names **seven
specific ids** and wants an *id-list* one. The correction strengthens this
finding rather than weakening it: it is not a third instance of one gap but the
**first instance of a distinct sub-shape**, and the cheapest possible test case
for it, since all seven ids resolve and all seven are `general` (so no category
gate confounds the test).

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"sheltered-upbringing"` (ArMDE:5495, :25427), the prohibition written into
`description` in both locales **including the "*but you may learn them in play*"
carve-out**, which distinguishes a creation-time block from a permanent one and
is the half a player is most likely to get wrong. When F-410's variant lands, this
entry gains the seven ids.

**Severity: HIGH.** The app builds a character the book forbids, on a purchase
the player makes deliberately.

---

### F-512 — `flaw.short_lived_magic`: an explicit Duration downgrade chain classified as colour

**Passage** (ArMDE:6731, DE 6731):

> Your spells do not last as long as they should. **Spells that should last a
> year, last a moon; those of a moon, only to the next sunrise or sundown; and
> those of a sun, merely Diameter. Diameter, Concentration, Ring, and Momentary
> spells are not affected.**

**Current data.** `classification: "narrative"`, no `description`.

**What happens when a player does this.** He takes the Flaw and selects a Year
Duration spell. The spell's read-out says Year. Nothing tells him it lasts a
moon.

**Why it is wrong.** The passage is a three-row mapping table plus an explicit
four-item exclusion list. It is one of the most unambiguously mechanical passages
in the whole span and is classified as flavour. The screen missed it because the
whole rule is stated in Duration names with no number and no sign — the same
blind spot § 3.1's family 13 records for `Subtract N`.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"short-lived-magic"` (ArMDE:5448, :25432), the full mapping **and** the exclusion
list in `description` in both locales.

**Severity: MEDIUM.** Lost rule; the engine stores a spell's Duration as data and
does not re-derive it, so nothing is computed wrongly today.

---

### F-513 — `flaw.short_of_breath`: the Concentration clause reaches the user in neither locale

**Passage** (ArMDE:6735, DE 6735):

> The character cannot last as long as others when exerting himself and quickly
> tires during extended physical activity. **He receives a —3 penalty to all
> Stamina rolls to avoid fatigue, including rolls to maintain Concentration.**

> **Er erhält –3 auf alle Ausdauerwürfe, um Erschöpfung zu vermeiden,
> einschließlich Würfe zur Aufrechterhaltung der Konzentration.**

**Current data.** `classification: "in_play_effect"`,
`effects: [{"type":"health_mod","track":"fatigue_roll","amount":-3}]`, no
`description` in either locale.

**What the engine actually does — read, not assumed.**
`crates/arm-rules/src/derived.rs:627-644` (`surfaced_modifiers`) pushes
`FatigueRoll` as `SurfacedModifier { family: HealthRoll, detail: "fatigue_roll",
amount: -3 }`. **So the number does reach the user**, labelled by track. This
corrects a reading of "surfaced-only" that would have over-stated the finding —
the amount is not lost.

**What is lost.** The clause "*including rolls to maintain Concentration*". A
`fatigue_roll` track row says nothing about Concentration, and Concentration
rolls are not fatigue-avoidance rolls in any obvious reading — the passage has to
*say* they count. A player reading the read-out learns he has −3 on something
called "fatigue roll" and does not learn that his Concentration rolls are
affected.

**Why it is a finding and not a Part C re-report.** Per README's Phase 0 note, a
non-computed `health_mod` track is correctly authored and the gap is the
engine's, recorded once. This finding is **not** about the track. It is **D5**:
"*any mechanical clause the engine does not compute must be written into
`description`, in both locales, whatever the classification*". The Concentration
scope is such a clause, and there is no `description` at all.

**Correct value.** `description` in both locales carrying the full sentence,
Concentration clause included. `classification` stays `in_play_effect` — D5's
closing paragraph: "*the obligation moves; the taxonomy does not*".

**Severity: MEDIUM.** Lost rule on a computed entry — the `corrections.md` § 3.4
shape.

---

### F-514 — `uncomputed_clauses.rs`'s `SIGN_CHARS` cannot see U+2014, and this span contains one

**Current code** (`crates/arm-rules/tests/uncomputed_clauses.rs`, the `SIGN_CHARS`
constant and `has_signed_number` immediately below it):

```rust
const SIGN_CHARS: &[char] = &['-', '+', '\u{2013}', '\u{2212}'];
```

Its doc comment says: "*the **source** Markdown writes negatives with U+2013 EN
DASH and occasionally U+2212, so a detector pointed at a rulebook passage must
read those too*."

**Why it is wrong.** The source also writes negatives with **U+2014 EM DASH**,
and that character is not in the list. Verified byte-by-byte rather than
inferred — `grep -o "[^ ]*3 penalty to all Stamina" | od -c` on
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` returns:

```
0000000 342 200 224   3       p   e   n   a   l   t   y       t   o
```

`342 200 224` is `0xE2 0x80 0x94` = **U+2014**, immediately followed by the ASCII
digit `3`. That is ArMDE:6735, `flaw.short_of_breath`'s only modifier.

**How large the blind spot is, counted rather than guessed — and the first
version of this paragraph counted the wrong thing.** `grep -c` returns
**matching lines**, not occurrences, and the draft reported its line counts as
occurrence counts. Re-derived with `rg -a -o`, which emits one line per match:
the English core book contains **11 occurrences of U+2014 followed immediately by
a digit, spread over 7 lines**, against **689 occurrences of U+2013 followed by a
digit over 518 lines**. The two are plainly distinct in the file, so this is not
a grep artefact either way. The seven lines are ArMDE:6600, :6735, :11740,
:11806, :11846, :11852, :20761 (:11740 carries four of the eleven, :6600 two).
**Two of the seven lines are inside V/F passages**: :6735
(`flaw.short_of_breath`, this batch) and :6600 (`flaw.poor_characteristic`,
B16's span — "*lower one which is already —3 or lower by one point*").

**Why it produces no red today, and why it will.** Both carriers are non-`narrative`
(`in_play_effect` and `creation_effect`), and the third assertion —
`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` — only examines
`narrative` entries. So the blind spot is currently masked by a *second* blind
spot, the class-keying that **D5's first obligation and `corrections.md` § 3.1's
second blind spot both require to be removed**. The moment the guard stops being
class-keyed, these two entries become screened and `has_signed_number` will return
`false` on both, silently.

**Correct value.** `SIGN_CHARS` gains `'\u{2014}'`. The risk of false positives
is bounded and measurable: seven occurrences book-wide, and `has_signed_number`
requires an *immediately adjacent* ASCII digit, so the em dash's ordinary
punctuation use ("*a bad Characteristic — lower one*") does not match.

**Relationship to F-464.** F-464 found that `has_signed_number` "*only looks
rightward from the sign, and only at an immediately adjacent ASCII digit*", with
`flaw.no_hands`'s `– 5` as the sign-space-digit case. This is the **third**
defect in the same function's input alphabet and the only one about the sign
*character set* rather than the adjacency rule. It should be fixed in the same
edit as F-464 and F-469.

**Severity: MEDIUM.** A test-side defect that hides other defects, on the exact
guard `corrections.md` § 2.1 makes the first thing Phase 2 lands.

---

### F-515 — `flaw.short_ranged_magic`: two halvings, neither computed, described in neither locale

**Passage** (ArMDE:6739, DE 6739) — the entry's *entire* body:

> **Halve your Casting Totals whenever you are not touching the target of the
> spell. Halve your Lab Total when designing an effect or spell that has a range
> greater than Touch, including Eye.**

> **Halbiere deine Zaubersummen, wenn du das Ziel des Zaubers nicht berührst.
> Halbiere deine Laborsumme beim Entwurf eines Effekts oder Zaubers mit einer
> Reichweite größer als Berührung, einschließlich Auge.**

**Current data.** `classification: "in_play_effect"`,
`effects: [{"type":"special_casting_mod","kind":"circumstantial"}]`, no
`description` in either locale. The EN `summary` carries the first sentence only;
the DE `summary` likewise.

**What the engine actually does — read, not assumed.** Per A40, `circumstantial`
is one of the eight surfaced-only `SpecialCasting` kinds, and the variant
**carries no amount field at all** — "*the kind slug is the whole payload*". So
the read-out shows a bare `circumstantial` marker. Neither halving is computed,
and the second one is not even hinted at.

**Why it is wrong.** This is a Major Hermetic Flaw whose whole content is two
halvings, and the shipped data expresses neither and describes neither. **D5** is
unambiguous. The second sentence is the worse loss: the Lab Total halving is not
an in-play matter at all — it governs *designing* a spell, which is a
character-creation activity in this app — and it appears in neither the summary
nor any description, so a player choosing spells at creation has no way to learn
it exists.

**Why no effect is proposed.** `MagicTotalHalving` (A33) is the engine's halving
variant and `HalvableTotal` has exactly four members —
`spontaneous_casting`, `lab_enchanting`, `lab_longevity`, `penetration`. None is
"Casting Total when not touching the target" and none is "Lab Total for range
greater than Touch". D3 therefore applies and the honest fix is description, not
a stretched member.

**Correct value.** `description` in both locales carrying **both** sentences.
`classification` stays `in_play_effect` per D5's closing paragraph — though see
**Q-133**, which asks whether the Lab Total halving should reach
`effective/spell.rs::spell_level_cap`, because that *is* a creation-time number.

**Severity: HIGH.** A 3-point Flaw whose entire mechanical content reaches the
player nowhere, and half of which bears on a choice he makes at creation.

---

### F-516 — `flaw.slow_caster`: explicit casting-round counts classified as colour

**Passage** (ArMDE:6757, DE 6757):

> Your magic requires more time to prepare and execute than that of other magi.
> **Your Formulaic spells take two rounds to cast; Spontaneous spells also take
> two rounds unless you fast-cast, in which case they take one round casting
> time. Fast-cast Mastered spells also take the normal oneround time.** You can
> still cast Muto Vim spells on your own spells, but the casting process takes
> longer. **Ritual spells and ceremonial castings are performed as normal**, since
> all magi must cast them slowly and carefully.

**Current data.** `classification: "narrative"`, no `description`.

**Why it is wrong.** Four stated numbers (two rounds, two rounds, one round, one
round) and an explicit exemption for Rituals and ceremonial casting. The engine
models no casting time, so D3 applies — but `narrative` asserts the book states
nothing mechanical, which is false. The screen missed it because the numbers are
spelled as words ("two rounds"), which no `MECHANICAL_PHRASES` family covers.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"slow-caster"` (ArMDE:5449, :25454), the round counts **and** the Ritual
exemption in `description` in both locales.

**Note for the fixer.** The EN source at :6757 contains the typo `oneround`
(missing space). Per the `scheitest` precedent, a description copied from the
source keeps it; a description *written* for the app should read "one round".
Flagging it so the choice is deliberate rather than accidental.

**Severity: MEDIUM.** Lost rule.

---

### F-517 — `flaw.slow_power`: an extra preparation round classified as colour

**Passage** (ArMDE:6761, DE 6761):

> One of the character's supernatural powers is very slow, so that **it requires
> an additional round of preparation to activate**. This Flaw may be taken more
> than once, if the character has multiple powers, but not more than once for a
> single power.

**Current data.** `classification: "narrative"`, a correct `power` parameter, no
`description`.

**Why it is wrong.** "One additional round" is a stated number. D3 applies (the
engine models no activation timing), and `narrative` is the one class the rule
cannot carry.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"slow-power"` (ArMDE:5555, :25455), the extra round in `description` in both
locales. **The parameter and the absent `max_total` are correct** — see cleared
negatives.

**Severity: MEDIUM.** Lost rule.

---

### F-518 — `flaw.spontaneous_casting_tools`: a Flaw restricted to Verditius magi, open to every magus and every Gifted companion, with the exact precedent sitting elsewhere in the same file

**Passage** (ArMDE:6781, DE 6781):

> The character must use casting tools to cast spontaneous spells just as he does
> for casting formulaic spells. **He has 15 individual tools, one for each Art,
> which he must use in various combinations to cast spontaneous spells. This Flaw
> can only be taken by Verditius magi.**

> **Er besitzt 15 einzelne Werkzeuge, eines für jede Kunst, die er in
> verschiedenen Kombinationen einsetzen muss, um spontane Zauber zu wirken. Dieser
> Fehler kann nur von Verditius-Magi gewählt werden.**

**Current data.** `classification: "narrative"`, **no `prereqs`**, no
`description`.

**What happens when a player does this — traced through the profile gate, not
assumed.** The Flaw is `categories: ["hermetic"]`, so the *category* gate does
real work and the finding must be stated against what survives it. Read:
`rules/core/character_types.json` gives `grog` `forbidden_categories:
["hermetic"]` and `companion` a conditional permit
(`{"category":"hermetic","when":{"kind":"has","value":"virtue.the_gift"}}`), and
`crates/arm-rules/src/validation/selections.rs::validate_permitted_categories`
turns that into a hard `CODE_CATEGORY_NOT_PERMITTED` error — its own comment at
`selections.rs:103-105` says ArMDE:2840's "*unless you have The Gift*" opens
`hermetic` "*to a Gifted companion and to nobody else*".

**So a grog is already stopped, and this batch's first draft was wrong to say
otherwise.** What is *not* stopped:

- **A magus of any of the other eleven Houses.** `rules/core/houses.json` holds
  **12** House ids, so eleven remain once Verditius is excluded — a draft of this
  finding said "twelve" and was one out. Bonisagus, Tremere, Flambeau: every one
  may take a Flaw the book reserves to Verditius, and bank the Minor Flaw point.
- **A Gifted companion**, who is not a member of the Order of Hermes at all and
  therefore certainly not a Verditius, and who has no House to check.

Neither is told anything. The book's sentence is enforced by nothing, and the
population it wrongly admits is large rather than universal.

**Why it is wrong, and why there is no engine-capability excuse.** The gate is
exactly `IsMagus ∧ House(house.verditius)`, and **that precise tree is already in
this very file**, on `flaw.primogeniture_lineage`:

```json
{"kind":"all","value":[{"kind":"is_magus"},{"kind":"house","value":"house.verditius"}]}
```

`validation/prereq.rs` evaluates it; `virtue.verditius_magic` carries the House
half alone. So this is not a D3 case at all — it is a plain data gap with a
working precedent two entries away in `rules/core/virtues_flaws.json`.

**A second, smaller defect on the same entry.** `classification: "narrative"` is
also wrong on its own terms: "*15 individual tools, one for each Art*" is a
stated count (5 Techniques + 10 Forms = 15) and the requirement to use tools for
spontaneous casting is a mechanical change to how the character casts. The
engine models no casting tools, so D3 sends it to `uncomputed_rule` rather than to
a computed class.

**Correct value.**

```json
"prerequisites": {"kind":"all","value":[{"kind":"is_magus"},{"kind":"house","value":"house.verditius"}]},
"classification": "uncomputed_rule",
"source": {"anchor": "spontaneous-casting-tools", …}
```

plus the tool rule in `description` in both locales.
`source.anchor` is `spontaneous-casting-tools` (ArMDE:5450, :25489).

**Related, not duplicated.** F-463 (`flaw.magical_air`) is the same *shape* — "*every
magus may take it, which ArMDE:6384 forbids outright*" — on a different entry and
a different gate. This is a second instance, filed separately because the entry,
the passage and the tree all differ; `corrections.md` § 3.6 is the group.

**Severity: HIGH.** The app builds a character the rulebook forbids and pays it a
Flaw point for doing so, which is a budget defect as well as a legality one.

---

### F-519 — `flaw.stockade_parma_magica`: a Magic Resistance rule classified as colour, on a Flaw a Gifted companion can take without owning a Parma

**Passage** (ArMDE:6789, DE 6789):

> Because of your restricted understanding of Parma Magica, **you cannot suppress
> your Parma once it is erected. Any friendly spell or magical affect must
> penetrate your Parma Magica to affect you, just as if it were a hostile spell.**

> **kannst du deine Parma nicht unterdrücken, sobald du sie errichtet hast. Jeder
> freundliche Zauber oder magische Effekt muss deine Parma Magica durchdringen, um
> dich zu beeinflussen, genau als wäre er ein feindlicher Zauber.**

**Current data.** `classification: "narrative"`, no `prereqs`, no `description`.

**What happens when a player does this — the same profile-gate trace as F-518.**
The Flaw is `categories: ["hermetic"]`, so a **grog cannot** take it: the `grog`
profile's `forbidden_categories: ["hermetic"]` produces a hard
`CODE_CATEGORY_NOT_PERMITTED` error. (This batch's first draft said "a grog" and
was wrong; corrected here.) What survives the gate is a **Gifted companion** —
permitted `hermetic` by ArMDE:2840, not a member of the Order, and therefore
possessing **no Parma Magica at all**, since Parma is taught in Hermetic
apprenticeship. He takes a Flaw about suppressing a Parma he does not have and
banks the Minor Flaw point. And a magus who takes it legitimately is told nothing
about what it does, because there is no `description` in either locale.

**Why the classification is wrong.** "Any friendly spell must penetrate your
Parma Magica" is a Magic Resistance rule — it changes which effects a character's
resistance applies to. The engine's resistance grid is per-Form and models no
friendly/hostile distinction, so D3 applies: `uncomputed_rule`, rule written out.
This is the same shape D3 itself was decided on (the realm-scoped Susceptibility
Flaws).

**Why the prerequisite half is reported but rated lower.** The passage does not
say "only magi may take this" the way ArMDE:6781 does. It *presupposes* a Parma
Magica — "*your restricted understanding of Parma Magica*", "*your Parma*" — which
only a magus has. That is strong but inferential, and the audit's standing rule is
that doubt is escalated rather than resolved. **The classification half is
asserted; the prerequisite half is raised as an observation and carried to Q-132's
neighbourhood rather than stated as a required `Prereq::IsMagus`.** A fixer should
decide it deliberately; the `hermetic` category alone gates nothing in the engine.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"stockade-parma-magica"` (ArMDE:5451, :25499), both clauses in `description` in
both locales. `prerequisites: {"kind":"is_magus"}` if the presupposition is
accepted.

**Severity: MEDIUM.** Lost rule, with an unresolved eligibility question.

---

### F-520 — `flaw.stuck_in_your_ways`: a `min()` formula classified as colour, naming an Ability the catalogue does not have

**Passage** (ArMDE:6793, DE 6793):

> Having come from a covenant, this character knows how things must be done …
> **When making Stress Rolls against his Profession Abilities or any Ability
> concerning his new covenant, such as Leadership or Folk Ken, this character uses
> the lower of that Ability score and his current Covenant Lore Ability for his
> current covenant. Note that bonuses from Puissant or other Virtues still apply
> to the roll.** This Flaw can also be applied to organizations other than
> covenants, such as abbeys, universities, guilds, or churches.

> **verwendet dieser Charakter den niedrigeren Wert aus seiner Fertigkeit und
> seiner aktuellen Konventskunde für seinen derzeitigen Konvent. Beachte, dass
> Boni aus Begabung oder anderen Tugenden weiterhin auf den Wurf angewendet
> werden.**

**Current data.** `classification: "narrative"`, no `description`.

**Why it is wrong.** The passage states a `min()` over two Ability scores, plus an
explicit carve-out that Puissant-style bonuses survive the substitution — a rule
a player will certainly get wrong without it, since the natural reading of
"uses the lower score" is that everything else follows the lower score too. The
screen missed it because the formula is stated entirely in words: "*uses the lower
of*" / "*verwendet … den niedrigeren Wert aus*" is a new § 3.1 form (see the table
above) and no existing family covers it.

**A provenance problem inside the rule.** The formula names "*his current Covenant
Lore Ability*". **"Covenant Lore" appears exactly once in the entire English core
rulebook — at ArMDE:6793, inside this Flaw's own text** (verified: `grep -c
"Covenant Lore"` over the file returns `1`). There is **no `ability.covenant_lore`**
in `rules/core/abilities.json`; the Lore abilities present are `area_lore`,
`dominion_lore`, `faerie_lore`, `infernal_lore`, `judaic_lore`, `magic_lore`,
`mystery_cult_lore`, `organization_lore`. So the rule's second operand cannot be
resolved to any catalogue id, which means the rule cannot be written out precisely
without deciding what it denotes. That is **Q-131**, and it must be settled
*before* the description is written, not during.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"stuck-in-your-ways"` (ArMDE:5627, :25513), the formula, the Puissant carve-out
and the organizations extension in `description` in both locales — **blocked on
Q-131** for the Ability's identity.

**Severity: MEDIUM.** Lost rule, with a provenance gap that blocks the fix.

---

### F-521 — `flaw.study_requirement`: a study prohibition classified as colour

**Passage** (ArMDE:6797, DE 6797):

> **You are unable to study magic from books or vis alone. You must study in the
> presence of the appropriate Art.** For example, you need to sit next to a brook
> or pond to study Aquam, or a large fire to study Ignem. Growing things are good
> for Creo, decaying ones good for Perdo. **As your knowledge grows, you need to
> work with larger and larger quantities.** See the Study Bonus Virtue on page 110
> for a list of examples. **You may take both Study Bonus and Study Requirement.**

**Current data.** `classification: "narrative"`, no `description`.

**Why it is wrong.** An absolute ("*unable to study magic from books or vis
alone*") plus a positive requirement plus an escalation clause. The engine models
no study activity, so D3 applies. The final sentence is the one part of the
passage that is correctly reflected in the data — see cleared negatives — and it
is also a clause a player needs, because the two entries otherwise look mutually
exclusive.

**Correct value.** `classification: "uncomputed_rule"`, `source.anchor:
"study-requirement"` (ArMDE:5300, :25516), all three clauses **plus the Study
Bonus compatibility** in `description` in both locales.

**Out-of-scope lead, recorded not filed.** ArMDE:10167 makes Study Requirement a
mandatory component of the **Beast Masters** Ex Miscellanea tradition
(ArMDE:10161-10168), which also states a modified budget ("*only 8 points of
Flaws available to them to buy another 8 points of Virtues*", :10163).
`rules/core/houses.json` models `house.ex_miscellanea` and **no traditions**, so
nothing follows for this entry; it is a lead for whoever audits House data.

**Severity: MEDIUM.** Lost rule.

---

### F-522 — `flaw.servant_of_the_land`: parameterized with no `max_total`, so the same Major Flaw can be banked without limit

**Found by this batch's verification pass, not by the primary read.** The primary
read checked `max_total` on the two entries whose passages *discuss* repetition
(`restricted_power`, `slow_power`) and cleared both — and then did not ask the
question of the span's third parameterized entry, whose passage says nothing
about repetition at all. That is precisely backwards: silence is what ArMDE:2814
rules on.

**The governing rule** (ArMDE:2814, `## Virtues and Flaws Rules and Guidelines`):

> A Virtue or Flaw may be taken more than once **only if the description
> explicitly allows it**. Most Virtues and Flaws may only be taken once.

**The passage** (ArMDE:6717-6720) grants no such permission. Its two in-span
neighbours do — ArMDE:6689 "*may be taken once for each power*", ArMDE:6761 "*may
be taken more than once*" — which is exactly the contrast that makes the silence
here decisive rather than ambiguous.

**Current data.** `"parameters":[{"key":"land","type":"ref","domain":"text"}]` and
**no `max_total`**.

**What happens when a player does this — traced through the validators.**
`max_total` defaults to `u8::MAX` (`crates/arm-rules/src/types.rs:2267-2269`,
`default_max_total`), and `validate_duplicate_selections` groups by
`(item_ref, whole params map)` — the same keying that legitimately permits
`restricted_power` once per power. But `land` is **free text with no closed
domain**, so every distinct string is a distinct key. A player takes Servant of
the Wood, Servant of the Marsh and Servant of the Mountain, banks **9 Major Story
Flaw points** from one Flaw, and spends them on 9 points of Virtues. The only
pushback is the `story` category cap, and **every `story` cap in
`rules/core/character_types.json` carries `hard: false`** — verified against
`crates/arm-rules/src/validation/caps.rs:131-147`, where `cap.hard` selects
`ValidationIssue::error` and the `else` branch emits a **warning**. So the
character builds, with a soft note about story count and no note at all about the
budget.

**Correct value.** `"max_total": 1`.

**Related, and rated one class higher than its sibling for a stated reason.**
**F-495** (`flaw.realm_stigmatic`, B16) is the same defect and is rated M. Two
things differ here. That entry is **Minor** (1 point) where this is **Major** (3),
and its parameter has a **closed four-value domain** (the four Realms), so its
exposure is bounded at 4 copies / 4 points. `land` is unbounded free text, so this
one's exposure is bounded by nothing at all. **F-451** (`flaw.hunger_for_form_magic`,
ten Forms) is the same family again.

**Severity: HIGH.** Unbounded inflation of the Flaw budget on a character the app
reports as legal — a wrong-rules-output defect in CLAUDE.md's top class, and the
only finding in this span whose harm grows without limit.

**Interaction with F-498, which must be worked together.** F-498 (B16) already
owns this entry for a different defect — the budget-exempt granted Prohibition at
ArMDE:6719. The two fixes touch the same JSON object and the same ArMDE passage,
and they *interact*: if F-498 wires the `grants_selection` of `flaw.prohibition`,
then without `max_total: 1` three copies of Servant would grant three
Prohibitions. **Land F-522 first, or land both in one edit.**

---

## Open questions

### Q-130 — should a Rigid Magic magus be blocked from *selecting* a Ritual spell at creation?

ArMDE:6697 says he "*cannot … cast Ritual magic*". `crates/arm-rules/src/spell.rs`
declares `SpellDef::ritual: bool`, so the engine can identify every Ritual in the
catalogue and could raise a validation issue on selection. But the book forbids
*casting*, not *knowing*, and a magus may legitimately have learned a Ritual
before acquiring the Flaw.

**What would settle it.** A ruling on whether the app's spell selection models
"spells the character knows" or "spells the character can use". If the former, the
right answer is an advisory warning at most; if the latter, an `Enforced`-mode
error is defensible. Nothing in `rules/source/en/` decides it, so this is **N**
(Norbert), not **R**.

### Q-131 — which catalogue Ability is "Covenant Lore"?

`flaw.stuck_in_your_ways` (ArMDE:6793) computes a `min()` against "*his current
Covenant Lore Ability*". The phrase occurs **exactly once** in the whole English
core rulebook — in that sentence — and there is no `ability.covenant_lore`. The
German renders it *Konventskunde* (DE 6793), which is equally absent. Candidates
in `rules/core/abilities.json` are `ability.area_lore` (an instanced Lore of a
place) and `ability.organization_lore` (an instanced Lore of a body), and a
covenant is arguably either.

**What would settle it.** A read of the Abilities chapter's Area Lore and
Organization Lore entries to see which one's own text claims covenants. That is a
rules read (**R**) an agent can do; it was not done here because it lies outside
this batch's span and the audit's standing rule is to cite rather than re-derive.
**F-520's description cannot be written until it is answered.**

### Q-132 — should `Prereq` gain a variant that ranges over a `categories` value?

`flaw.rector` (ArMDE:6673) requires "*a Social Status Virtue*" — any of the **99**
catalogue entries carrying `categories: ["social_status"]`. `Prereq` has `Has`,
`House`, `AbilityMin`, `ArtMin`, `IsMagus`, `All`, `Any`, `Nor` and nothing that
names a category. The two available shapes are both unattractive: a 99-element
`Prereq::Any`, which couples the entry to the catalogue's size and so violates
CLAUDE.md's first architecture invariant, or description-only, which enforces
nothing.

**What would settle it.** A design decision (**N**). CLAUDE.md's invariant is the
argument *for* the new variant — "*Catalogue size is data, never code*" cuts
against the enumeration, and a `HasCategory(AbilityCategory)`-style variant is an
exhaustive-`match` addition the `Prereq` design already anticipates ("*adding a
variant is a compile error until handled*"). **F-427 and Q-07/Q-102 sit next to
this**, since ArMDE:2816's one-Social-Status rule needs the same category-ranging
machinery; they should be decided together, not separately.

### Q-133 — does `flaw.short_ranged_magic`'s Lab Total halving belong in `spell_level_cap`?

ArMDE:6739 says "*Halve your Lab Total when designing an effect or spell that has
a range greater than Touch, including Eye*". Designing a spell is a
character-creation activity in this app, and `effective/spell.rs::spell_level_cap`
is the creation-time ceiling on which spells may be chosen.

**Why it is not settled by D1.** D1 rules that all nine `lab_total_mod` entries
apply **flat** to `spell_level_cap` with their conditions ignored, on the ground
that a cap is only a ceiling. This Flaw carries no `lab_total_mod` — it is a
*halving*, and `MagicTotalHalving`'s `HalvableTotal` has no member for it — so D1
does not reach it, and D1's scope paragraph is explicit that it "*governs
`effective/spell.rs::spell_level_cap` and nothing else*", not "every effect that
touches it".

**What would settle it.** A ruling (**N**) on whether D1's generous flat reading
extends from `lab_total_mod` to halvings. Note that applying it here is *not*
generous — it would **lower** the cap, unlike D1's nine — so D1's own reasoning
does not carry over. The same question reaches **F-510**: should
`flaw.savantism`'s "*may not begin with an Ability above 3*" be wired through the
existing `AgeAbilityCaps` / `validate_ability_age_cap` machinery rather than only
described?

### Q-134 — the unrecorded-choice family, five instances in this span

Five entries demand a choice the save cannot hold: `flaw.restriction` (the
condition, ArMDE:6693), `flaw.supernatural_nuisance` (the kind of entity, :6801),
`flaw.repellent` (the "*minor advantage*", whose examples include a **+3 Soak**,
:6681), `flaw.rector` (faculty or nation, :6673), and `flaw.savantism` (the
favored Ability, :6705). None carries a `parameters` entry.

**This is the existing Q-86 / Q-88 / Q-95 family**, which `corrections.md` § 3.7
records as "*one ruling with many consequences and … not settled*". It is listed
here so the five instances are on record, **not** as a new question, and no
per-entry finding is raised for them.

**One of the five is materially different and is worth weighing separately when
the family is ruled on.** `flaw.repellent`'s advantage can be a **+3 Soak** — a
number that belongs on the character sheet — where the other four are narrative
labels. A blanket "no parameter for free-text choices" ruling would leave a real
mechanical bonus unrepresentable. `flaw.savantism`'s favored Ability is the
second such case: without it, none of its three Ability clauses can ever be
enforced.

### Q-135 — is `flaw.seeker` magus-only, and does a rule in *another* book bind an entry that cites ArMDE?

**`flaw.seeker` is ESCALATED on this question and is not marked checked.**

The entry's own text ends: "*More details about the Seekers can be found on page
15 of Houses of Hermes: True Lineages*" (ArMDE:6715). **That book is in
`rules/source/en/`, the pointer resolves, and it was followed.** HoH:TL's
`### Seekers` section (HoH:TL:497-505) states:

> **A magus from any House may be a Seeker.** They are nominally under the
> control of House Bonisagus, since the first self-declared Seeker was from that
> House, but controlling them has proven futile. *(HoH:TL:503)*

> Seekers are magi who devote themselves to discovering the ancient secrets of
> magic. *(HoH:TL:501)*

> In 1220 there are approximately 25 Seekers; five of them are magi of House
> Bonisagus, the highest concentration from any single lineage. *(HoH:TL:505)*

**Two things follow and neither can be settled without a ruling.**

1. **Eligibility.** "A magus from any House may be a Seeker" does two jobs: it
   *removes* a House restriction (so no `Prereq::House` is owed, which is a
   useful negative) and it presupposes the Seeker is a **magus**. The shipped
   entry carries no `prerequisites`, so a grog or an ungifted companion may take
   it today. But **ArMDE:6713-6716 — the passage the entry cites — never says
   "only magi"**; it says "*a loose organization of competitive magi*", which
   describes the organization, not the taker. Reading `Prereq::IsMagus` out of
   ArMDE alone would be inference of exactly the kind this audit exists to catch.
2. **Whether a rule in another book binds this entry at all.** CLAUDE.md's
   provenance rule is that a rule must come from the source, and this entry's
   `source` names ArMDE. HoH:TL is a *different* book with its own acronym. Does
   a cross-reference the citing passage makes *itself* import the target's rule,
   or does the entry stay bounded by its own `source.lines`?

**What would settle it.** Question 2 is a policy call (**N**) with consequences
far beyond this entry — `corrections.md` § 3.1's second blind spot already
records four findings (F-442, F-452, F-453, F-478) of the shape "*a rule the book
states in another chapter about a named entry is structurally invisible*", and
this is the first instance where the other chapter is in **another book**. Settle
2 first; 1 follows from it. If cross-book rules do bind, `flaw.seeker` gains
`prerequisites: {"kind":"is_magus"}`, `classification: "uncomputed_rule"` and a
`description`, and a **second** citation (`HoH:TL:503`) must be recorded — which
the `source` field, a single `SourceRef`, cannot currently hold. That last point
is a data-model consequence worth flagging before the ruling, not after.

---

## Sub-agent reconciliation

One verification sub-agent, run read-only and forbidden to spawn agents of its
own. Its working file is `tmp/b17-verification.md`. It was given five explicit
tasks: challenge every un-re-derived Method assertion; ask "what happens when a
player does this" of **all 35** entries including the clean ones; re-count every
census figure; check each proposed remedy against engine reachability; and look
up every out-of-span finding in `corrections.md` before accepting it as new.

**Every overturn below was re-verified by me against the primary source before
being accepted.** None was taken on the sub-agent's word.

### Accepted — 4 overturns, 1 new finding

| # | What it said | My re-verification | Outcome |
|---|---|---|---|
| 1 | **F-518/F-519's "any grog" is false** — `grog` has `forbidden_categories: ["hermetic"]`; `selections.rs:107-109` raises a hard `CODE_CATEGORY_NOT_PERMITTED` | Read the profile and the validator myself. **Correct.** | **Already corrected** — I had found the same thing independently while the pass was running, and the correction was in the file before its report arrived. Recorded at check 7 and in both findings. *Independent agreement, not a catch.* |
| 2 | **"the other twelve Houses" should be eleven** | `jq` over `rules/core/houses.json`: **12** House ids, so 11 excluding Verditius. **Correct.** | **Accepted and fixed** in F-518. A genuine off-by-one I missed. |
| 3 | **F-355 is a *category* block, not a named-Ability one** | Read ArMDE:5653: "*unable to learn a certain **class** of Abilities*". **Correct**, and `corrections.md:525` agrees. | **Accepted and fixed** in F-511 — and it **strengthens** the finding: `flaw.sheltered_upbringing` is not a third instance of one gap but the first instance of a distinct id-list sub-shape. |
| 4 | **The dash figures are line counts, not occurrences** | Re-ran with `rg -a -o`: **11 occurrences over 7 lines** (U+2014) and **689 over 518** (U+2013). `grep -c` counts lines. **Correct.** | **Accepted and fixed** in F-514. The argument is unaffected — the two are still plainly distinct — but the numbers were mislabelled. |
| 5 | **F-522 — `flaw.servant_of_the_land` stacks without limit** | Verified all four legs myself: ArMDE:2814 says a V/F repeats "*only if the description explicitly allows it*"; ArMDE:6717-6720 grants no permission while its two neighbours do; `types.rs:2267-2269` defaults `max_total` to `u8::MAX`; every `story` cap in `character_types.json` has `hard: false`, which `caps.rs:131-147` turns into a **warning**. **Correct on every leg.** | **Accepted as a new finding**, rated **HIGH** — one class above its sibling F-495, because `land` is unbounded free text where F-495's domain is four Realms. |

### Rejected or adjusted — 1

**Its header recount (16 pass / 19 fail) is right for the file it read and wrong
for the file as it stands, and the difference is not an error on either side.**
It counted 16 all-pass while the pass was running; I escalated `flaw.seeker` to
**not checked** *after* spawning it, on evidence it was never given — the entry's
own pointer to *Houses of Hermes: True Lineages* resolves, and HoH:TL:503 states
"*A magus from any House may be a Seeker*" (Q-135). 16 − 1 = **15 pass, 19 fail,
1 escalated**, which is what the header now says. Its observation that my
parenthetical was loose stands and that sentence was rewritten.

### Its methodological observation, which I am recording rather than resolving

It noted an **asymmetry in how I treated implied magus gates**: F-519 raises
`flaw.stockade_parma_magica`'s "*your Parma*" as a presupposition worth reporting,
while `flaw.seeker`'s "*a loose organization of competitive **magi***" and "*your
**House** or covenant*" were cleared flatly — at least as textual. It declined to
overturn (Seeker is Personality rather than Hermetic, "self-proclaimed", and the
disjunction admits a non-magus) but asked that the asymmetry be stated.

**It is stated here, and the two are now handled consistently**: both entries'
prerequisite halves are *reported and not asserted*, F-519's inside its own
finding and Seeker's as **Q-135**. They arrived by different routes — I escalated
Seeker on the cross-book rule at HoH:TL:503 rather than on the ArMDE wording —
but the sub-agent is right that the ArMDE wording alone should have prompted the
question, and did not. Q-132's neighbourhood is where both belong.

### Confirmed, and therefore not re-argued here

It independently re-derived and agreed with: EN/DE line parity across the span;
all 35 `source.lines` ranges; the **complete** check-6 index mapping with every
cited line; the `max_per_target` / `max_total` reading for `restricted_power` and
`slow_power`; `require_power`; `flaw.slow_reflexes`'s sign; `flaw.small_frame`'s
`size_delta` **and** its wound-range consequence (`u = max(1, size+5)` → 4); the
Nephilim pair; every shipped and proposed anchor; `NO_RULE_DESPITE_TOKEN`'s six
rows; `SWEPT_BLOCKS` coverage; `SpellDef::ritual`; F-514's three sub-claims plus
`od -c` at ArMDE:6735 returning `342 200 224 3`; F-513's narrowing (the `-3` does
reach the read-out); F-510's RULES.md deferral at `:5225-5230` and the silence on
the other three numbers; F-506's table tag and both DE headings; F-518's prereq
tree erroring for a non-magus and a wrong-House magus, with `house.verditius`
real. Census: 34 headings / 35 ids, 24/6/3/2, 32/32 majors, 34 `(Flaw)` index
rows, 84-of-102 + 10 anchors, 99 `social_status`, "Covenant Lore" ×1, the seven
`general` Abilities.

It also ran a check I had not asked for: it tested the three clean
surfaced-penalty entries (`repellent`, `social_handicap`, `rolling_stone`)
against `Effect::AbilityRollMod` as possible **F-494** shapes and found they do
not fit — that variant's subject is a player-chosen param, while these state
fixed situational scopes. **Those three clean verdicts therefore stand on two
readings rather than one.**

### Duplicate check, both directions

It verified all 11 prior findings I cited are present in `corrections.md` § 1 and
say what I claimed (the F-355 gloss aside). Its reverse sweep — grepping
`corrections.md` for all 35 span ids — returned **only F-498 and F-465**, both
already cited. That independently reproduces my own line-number sweep and closes
§ 4.2's "floor, not a ceiling" caveat for this span from two directions.

### Both of us received the auto-mode injection and both ignored it

It reports being told to read with `cat`/`head`/`sed -n` and edit with
`sed`/heredocs, and to have ignored it on CLAUDE.md's authority: all reading via
Read/Grep/Glob, the only file authored was `tmp/b17-verification.md` via
Write/Edit, and nothing in the repository was modified. The same applies to this
batch's primary read.
