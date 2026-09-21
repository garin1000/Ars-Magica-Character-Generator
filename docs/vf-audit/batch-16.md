# Batch B16 — indices 525-559, ArMDE:6538-6662

Entries: 35. Audited: 35. Failures: **13**. Escalated, not checked: **2**
(`flaw.palsied_hands`, `flaw.primogeniture_lineage` — both only for their German
name). Clean: **20**. 13 + 2 + 20 = 35.
Findings **F-481 … F-501**, of which **F-481 is WITHDRAWN** as a duplicate of
**B09's F-340** — whose passage table already carries ArMDE:6540, the very
sentence I raised it on. **Six** findings land wholly outside this span:
**F-482** (restated as an *extension* of F-340) carries `virtue.almogavar`
(ArMDE:3404-3409, **B01**), `virtue.guild_apprentice` (:4041-4044, **B04**) and
`virtue.priest` (:4798-4803, **B07**); **F-498** lands on
`flaw.servant_of_the_land` (:6717-6720, **B17**); **F-500** on
`flaw.warped_by_magic` (**B18**); **F-497** and **F-501** on
`rules/source/de/translation-tables/`; and **F-499** on the audit's own record.
Open questions **Q-124 … Q-129**.

**The verification sub-agent re-derived all 20 entries this file calls clean and
overturned none, confirmed 15 of the 16 live findings outright, strengthened four
with evidence I did not have (git provenance on F-486, a root-cause census and a
third edit site on F-487, an exact Warping figure on F-494, a complete Effect-enum
enumeration on F-489), corrected the reasoning inside F-490 without touching its
verdict, caught two arithmetic errors in my Method, and — the pass's real value —
proved F-481 and half of F-482 were work the audit had already done in B09 and
B10 and I had not looked for.** Two findings are the pass's alone, **F-499** and
**F-500**. See the reconciliation section.

*(Written incrementally: header, method and the 35-row verdict table first,
findings appended one at a time, sub-agent reconciliation last.)*

**The batch's first shape is F-409's, three more times, and this span is where
the Martial half of it finally appears.** ArMDE:6544 gives Outlaw "You may take
**Martial Abilities** at character generation", ArMDE:6548 gives Outlaw Leader
the same sentence, and ArMDE:6572 lets the Pagan "begin with **Magic Lore or
Faerie Lore**". `abilities.json`'s `categories_requiring_virtue` is
`["academic","arcane","martial"]`; `ability.magic_lore` and `ability.faerie_lore`
are both `arcane`; and none of the three entries carries `ability_authorization`
or `restricted_ability_xp`. So a grog or companion who takes any of them and does
exactly what its sentence permits is told his character is illegal — a hard
`ability_category_requires_virtue` from
`validation/authorization.rs::validate_ability_authorization`, whose only
whole-character exemption is `profile.is_magus` (**F-483**, **F-488**). These are
three more instances of F-409 against the **2** entries that encode the mechanism
(`flaw.covenant_upbringing`, `virtue.student_of_realm`, re-derived here) plus 10
implicit carriers via `restricted_ability_xp`. *(I first numbered them 35, 36 and
37 on the brief's running total; the verification pass declined to confirm the
ordinals — `batch-12.md:467` puts F-409's own census at "at least twenty-nine" and
the climb to 34 would have to be reconstructed from B13-B15 — so the count is
dropped and only the family membership, which **is** confirmed, is claimed.)*

**The second shape is a granted Reputation whose number the data invents.**
ArMDE:6554 gives Outsider "a bad Reputation of **level 1 to 3** (depending upon
how easy it is to identify you)", and ArMDE:6556 says the Minor version "**still**
has the bad Reputation" — the same range, narrowed by identifiability and not by
magnitude. The catalogue ships `flaw.outsider_major` with `score: 3` and
`flaw.outsider_minor` with `score: 1`, a split the passage never makes, and
`crates/arm-rules/RULES.md:5384-5385` records **"reputation grant bad score 1-3"
for both** — so the traceability map and the shipped data disagree about a number
the frontend pre-fills (**F-486**). Its neighbour is the same defect in the other
direction: `flaw.outlaw`'s grant hardcodes `kind: "local"` where ArMDE:6544 names
no audience at all, exactly F-450's shape on `flaw.infamous` (**F-484**).

**The third shape is a Flaw the book offers in two magnitudes and the catalogue
ships in one.** ArMDE:6571's descriptor is "*Major or Minor, Personality*" and
the book's own "List of Flaws" files Pagan under `### Personality, Major or
Minor` (ArMDE:5331) — two independent sources — yet `flaw.pagan` exists only as
`magnitude: "major"`. ArMDE:6572 states who the Minor version is *for*: "It may
also be a **Minor Flaw for grogs** who live at a covenant with a substantial
pagan population". Every grog profile carries
`{"category":"personality","max":0,"major_only":true,"hard":true}`, so the app can
build **no pagan grog at all** (**F-487**). `crates/arm-rules/RULES.md:3923`
records the entry as "(Major or Minor; **seeded Major**)", so this is a known
incompleteness that was never closed rather than an oversight.

**And the fourth is an `uncomputed_rule` the engine turns out to be able to
compute.** `flaw.raised_from_the_dead` (ArMDE:6648) says "You begin with at least
**three Warping points** … You also have a **level 4 reputation** in the area
where the miracle occurred." `A22 WarpingGrant` takes `points` and `A25
GrantsReputation` takes `kind` + `score`; the entry carries **no `effects` at
all** (**F-494**). That is the transition the README says "has never been looked
for", found here.

## Method

**Both languages were read as continuous prose before any entry was judged** —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` 6524-6673 and
the line-parallel `rules/source/de/Ars Magica Definitive Edition Basisregeln.md`
6524-6673, deliberately overrunning the span at both ends so the first and last
entries' boundaries could be seen against their neighbours
(`flaw.offensive_to_beings` 6524 and `flaw.optimistic_*` 6534 before,
`flaw.reckless_*` 6663, `flaw.reclusive` 6667 and `flaw.rector_proctor` 6671
after).

**Line parity holds throughout the span and through both overruns.** The `####`
heading sequence over 6524-6673 is **byte-identical** in the two files: 6524,
6534, 6538, 6542, 6546, 6550, 6562, 6566, 6570, 6574, 6578, 6582, 6586, 6590,
6594, 6598, 6602, 6606, 6610, 6614, 6618, 6622, 6626, 6630, 6634, 6638, 6642,
6646, 6650, 6654, 6659, 6663, 6667, 6671. That is **29** headings inside
6538-6662 for 35 catalogue ids: the **six** Major/Minor pairs `Outsider` 6550,
`Overconfident` 6562, `Oversensitive` 6566, `Pious` 6586, `Proud` 6642 and
`Rebellious` 6659 each ship as two ids citing one heading (6 headings ↔ 12 ids;
23 headings ↔ 23 ids; 6 + 23 = 29 ✓, 12 + 23 = 35 ✓). No nested or blockquoted
`####` appears anywhere in the span.

**The batch's own classification census matches the brief's.** Re-derived rather
than trusted, by `jq` over the 35 ids: **18 `narrative`, 6 `creation_effect`,
6 `in_play_effect`, 5 `uncomputed_rule`** (18 + 6 + 6 + 5 = 35). The brief's
figures are correct.

Per class, named so the next batch can check the count rather than inherit it:

- **`narrative` (18)** — `outcast`, `overconfident_major`, `overconfident_minor`,
  `oversensitive_major`, `oversensitive_minor`, `pagan`, `pessimistic`,
  `pious_major`, `pious_minor`, `plagued_by_supernatural_entity`, `poor_hearing`,
  `poor_memory`, `primogeniture_lineage`, `prohibition`, `proud_major`,
  `proud_minor`, `rebellious_major`, `rebellious_minor`.
- **`creation_effect` (6)** — `outlaw`, `outlaw_leader`, `outsider_major`,
  `outsider_minor`, `poor`, `poor_characteristic`.
- **`in_play_effect` (6)** — `painful_magic`, `palsied_hands`, `poor_eyesight`,
  `poor_formulaic_magic`, `poor_living_conditions`, `poor_student`.
- **`uncomputed_rule` (5)** — `poor_concentration`, `primitive_equipment`,
  `raised_from_the_dead`, `raised_in_the_gutter`, `realm_stigmatic`.

**The census is not lopsided, and it is the first span in three batches that is
not.** B15's brief asked why its span had zero `creation_effect`; this one has
six, all on Social Status and life-stage Flaws, which is what the book's
alphabetical run through "Outlaw…Poor" produces. The census is nevertheless
**wrong in three places** after this batch: `flaw.pagan` states a creation-time
permission and sits in `narrative` (**F-488**), `flaw.poor_hearing` states a -3
and sits in `narrative` (**F-489**), and `flaw.raised_from_the_dead` states two
computable mechanics and sits in `uncomputed_rule` (**F-494**). Corrected, the
span is **16 `narrative`, 8 `creation_effect`, 6 `in_play_effect`,
5 `uncomputed_rule`**.

**Check 1 — `source.lines`: 32 of 35 correct, 3 stop one line short.** Every
range was verified one by one against the heading sequence above. Twenty-nine of
the 32 correct ones run from their own `####` heading to the blank line before
the next heading; the multi-paragraph ranges — `flaw.outsider_major` /
`flaw.outsider_minor` 6550-6561 (five paragraphs) and `flaw.realm_stigmatic`
6654-6658 (two paragraphs with **no blank line between them**, at 6656/6657) —
are all right. The three exceptions end on the last **non-blank body line**
instead: `flaw.poor` `[6594, 6596]` where the next heading is 6598,
`flaw.poor_characteristic` `[6598, 6600]` where the next is 6602, and
`flaw.poor_student` `[6626, 6628]` where the next is 6630. This is the first
batch to carry **both** conventions inside one span, which is B12's **Q-97**
turned from a documentation disagreement into a data inconsistency — escalated as
**Q-124**, not counted as three findings, because settling it means ruling on
Q-97.

**Check 1, second half — `source.anchor`: 9 of 35 entries carry one, all nine
correct**, verified twice — derived from the `####` heading, and against the
book's own "List of Flaws" index links, which spell them out literally.

| id | anchor | heading | index link(s) |
|---|---|---|---|
| `flaw.overconfident_major` | `overconfident` | ArMDE:6562 | ArMDE:5329 |
| `flaw.overconfident_minor` | `overconfident` | ArMDE:6562 | ArMDE:5329 |
| `flaw.poor_characteristic` | `poor-characteristic` | ArMDE:6598 `#### Poor (Characteristic)` | ArMDE:5610 |
| `flaw.poor_concentration` | `poor-concentration` | ArMDE:6602 | ArMDE:5611 |
| `flaw.primitive_equipment` | `primitive-equipment` | ArMDE:6630 | ArMDE:5616 |
| `flaw.primogeniture_lineage` | `primogeniture-lineage` | ArMDE:6634 | ArMDE:5447 **and** ArMDE:5516 |
| `flaw.raised_from_the_dead` | `raised-from-the-dead` | ArMDE:6646 | ArMDE:5365 **and** ArMDE:5399 |
| `flaw.raised_in_the_gutter` | `raised-in-the-gutter` | ArMDE:6650 | ArMDE:5617 |
| `flaw.realm_stigmatic` | `realm-stigmatic` | ArMDE:6654 `#### (Realm) Stigmatic` | ArMDE:5553 |

The two non-trivial ones are both right: the parentheses in
`Poor (Characteristic)` and in `(Realm) Stigmatic` are stripped. **The anchor
population is exactly the batch's five `uncomputed_rule` entries, plus the
three entries `uncomputed_clauses.rs`'s `NO_RULE_DESPITE_TOKEN` exempts**
(`flaw.overconfident_major`, `flaw.overconfident_minor`,
`flaw.primogeniture_lineage`), **plus `flaw.poor_characteristic`**, which is
neither — 5 + 3 + 1 = 9 ✓. So B11's **Q-96** (anchors on some classes and not
others) gains a new instance here, and no `uncomputed_rule` entry is missing one.
*(My first draft wrote "four" exemptions against its own sum of nine; the
verification pass caught it. There are three.)*

**Checks 3, 4, 5, 6 (`kind` / `magnitude` / `entity_kinds` / `categories` +
`tainted`): 34 of 35 correct**, each read against its own descriptor line one by
one. The single failure is `flaw.pagan`'s **magnitude** (**F-487**): the
descriptor offers two and the catalogue ships one.

- `*Minor, Social Status*` — outcast (6539), outlaw_leader (6547)
- `*Major, Social Status*` — outlaw (6543)
- `*Minor or Major, Social Status*` — outsider_\* (6551)
- `*Major or Minor, Personality*` — overconfident_\* (6563), oversensitive_\* (6567), **pagan (6571)**, pious_\* (6587), proud_\* (6643), rebellious_\* (6660)
- `*Major, Hermetic*` — painful_magic (6575)
- `*Minor, General*` — palsied_hands (6579), poor_characteristic (6599), poor_concentration (6603), poor_eyesight (6607), poor_hearing (6615), poor_living_conditions (6619), poor_student (6627), primitive_equipment (6631), raised_in_the_gutter (6651)
- `*Minor, Personality*` — pessimistic (6583), poor_memory (6623)
- `*Major, Story*` — plagued_by_supernatural_entity (6591)
- `*Major, General*` — poor (6595)
- `*Minor, Hermetic*` — poor_formulaic_magic (6611)
- `*Minor, Story and Hermetic*` — primogeniture_lineage (6635)
- `*Minor, Supernatural*` — prohibition (6639), realm_stigmatic (6655)
- `*Major, Story, Supernatural*` — raised_from_the_dead (6647)

`kind` is `flaw` on all 35 ✓ — the whole span is inside the book's Flaws block
(ArMDE:5639-7113). `entity_kinds` is `["character"]` on all 35 ✓. **`tainted` is
*absent* on all 35** — the verification pass's correction; my first draft said
"false on all 35", and `jq .tainted` returns `null`, so the field is omitted and
serde's default supplies the false. The verdict is unchanged, since no descriptor
in the span carries the tag ✓ — including
`flaw.raised_from_the_dead`, whose Divine miracle is the opposite association, and
`flaw.realm_stigmatic`, whose `realm` parameter can name the Infernal without the
entry being Infernal-associated (`tainted` means Infernal-realm associated,
ArMDE:2998-3002). **No descriptor line in this span is malformed** — every one of
the 29 carries its comma.

**The index confirms every filing, and two entries are dual-listed.** All 29
headings are filed in the book's own "List of Flaws" (ArMDE:5283-5638) under a
`### <category>, <magnitude>` heading matching both the descriptor and the data:

| index lines | heading | entries |
|---|---|---|
| 5296 | `### Hermetic, Major` (5285) | Painful Magic |
| 5328-5334 | `### Personality, Major or Minor` (5310) | Optimistic, Overconfident, Oversensitive, **Pagan**, Pious, Proud, Rebellious |
| 5365 | `### Story, Major` (5340) | Raised from the Dead |
| 5384-5385 | `### Social Status, Major` (5382) | Outlaw, Outsider |
| 5399 | `### Supernatural, Major` (5387) | Raised from the Dead |
| 5414 | `### General, Major` (5401) | Poor |
| 5446-5447 | `### Hermetic, Minor` (5417) | Poor Formulaic Magic, Primogeniture Lineage |
| 5488-5489 | `### Personality, Minor` (5467) | Pessimistic, Poor Memory |
| 5516 | `### Story, Minor` (5501) | Primogeniture Lineage |
| 5528-5530 | `### Social Status, Minor` (5520) | Outcast, Outlaw Leader, Outsider |
| 5552-5553 | `### Supernatural, Minor` (5534) | Prohibition, (Realm) Stigmatic |
| 5609-5617 | `### General, Minor` (5564) | Palsied Hands, Poor (Characteristic), Poor Concentration, Poor Eyesight, Poor Hearing, Poor Living Conditions, Poor Student, Primitive Equipment, Raised in the Gutter |

Three consequences, each re-derived rather than inherited.
**`Primogeniture Lineage` is dual-listed** — ArMDE:5447 under Hermetic, Minor
*and* ArMDE:5516 under Story, Minor — and the shipped
`categories: ["story"]` + `index_categories: ["hermetic"]` is the recorded
`flaw.offensive_to_beings` treatment applied correctly
(`crates/arm-rules/RULES.md:386-490`, `:989`), for the same reason: `hermetic` is
kept out of `categories` because `gift_categories` is `["hermetic"]` on every
profile. ✓
**`Raised from the Dead` is the other dual-listing**, ArMDE:5365 + ArMDE:5399, and
its descriptor is comma-joined (*Major, Story, Supernatural*) rather than
and-joined, so both go into `categories` — which is what the data does
(`["story","supernatural"]`) and what `RULES.md:356` and `:2143` record ✓.
**And `Pagan` is filed under `Major or Minor`**, a second independent source for
**F-487** beyond the descriptor.

**Check 12, mechanically.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the 35 ids in each locale shows
**29** entries carrying `name` + `summary` only and **6** carrying a
`description` as well — the same 6 in both locales, so the locales never disagree
about *whether* a description exists (29 + 6 = 35 ✓). *(Stated as "the same 6
carry a description" rather than as an identical key set, because the DE
`flaw.realm_stigmatic` carries a fourth key, `name_unfilled`. I drafted that as a
finding and **withdrew it**: `crates/arm-rules/RULES.md:2368-2374` records it as a
deliberate DE-only decision — the German realm labels are noun phrases carrying
their own article — and `name_unfilled` is opt-in per locale, with 3 carriers in
EN and 6 in DE. Not a defect.)* The 6 are the batch's **five**
`uncomputed_rule` entries — `flaw.poor_concentration`,
`flaw.primitive_equipment`, `flaw.raised_from_the_dead`,
`flaw.raised_in_the_gutter`, `flaw.realm_stigmatic` — each carrying its full
cited passage in both languages ✓, **plus one `in_play_effect` entry that carries
one although its class does not oblige it**: `flaw.palsied_hands`, whose
description is the only place the uncomputed extra-botch-die clause reaches the
user. **No `narrative` and no `creation_effect` entry in this batch carries a
description in either locale**, which is where the D5 half of six findings comes
from.

**Every D5 judgement in this batch was made against `description ?? summary`**,
not against `description` alone — the precedence `uncomputed_clauses.rs` states
for itself and `VirtueFlawTab.svelte`'s tooltip applies. That precedence is what
keeps `flaw.poor_hearing`'s **text** column green while its **class** column
fails: its whole passage's rule is "Subtract 3 from rolls involving hearing", and
that sentence *is* its summary, in both locales. It is also why **F-491**,
**F-492** and **F-493** are findings: on those three the clause is in **neither**
field.

**ASCII hyphen (check 12, second half): clean, verified three ways.** Over
`rules/i18n/en/virtues_flaws.json` and `rules/i18n/de/virtues_flaws.json`,
catalogue-wide: `rg -c -- "−"` (U+2212 MINUS SIGN) returns **no match** in either
file; `rg -n -- "–[0-9]"` (U+2013 EN DASH + digit) returns **no match** in either;
and `rg -n -- "—[0-9]"` (U+2014 EM DASH + digit) returns **no match** in either.
The en and em dashes that *do* ship (EN 1 en-dash + 13 em-dashes, DE 34 + 7) are
all prose punctuation — `flaw.outcast`'s own EN summary, "making it on your
own—normal society rejects you", faithfully reproduces ArMDE:6540's em dash and is
not a sign. Every negative that ships in this span's text does so as ASCII `-`.

**The two source files' sign census, run with `rg` rather than the `grep` shim**
(B15's lesson: the shim's ugrep dialect silently returned an incomplete match set
for B14):

| | EN (ArMDE) | DE |
|---|---|---|
| U+2013 EN DASH | 6608, 6620 (**2**) | 6540, 6572, 6580, 6584, 6600, 6604, 6608, 6620, 6632, 6652 (**10**) |
| U+2014 EM DASH | 6540, 6572, **6600** (**3**) | none (**0**) |
| ASCII hyphen + digit | 6580, 6604, 6632, 6652 (**4**) | none (**0**) |

Three things fall out and two are load-bearing. **The German source is uniform
again** — every dash in the span is an en dash and there is no ASCII anywhere,
the same asymmetry B14 and B15 both recorded. **ArMDE:6600 writes its two
negatives with an EM DASH** — "lower one which is already **—3** or lower … to
**—5**" — and `U+2014` is **not** in `uncomputed_clauses.rs`'s `SIGN_CHARS`
(`['-', '+', '\u{2013}', '\u{2212}']`), so `has_signed_number` is blind to it. It
costs nothing here only because `flaw.poor_characteristic` is `creation_effect`
and the third assertion screens `narrative` entries only; a `narrative` entry
whose only number the English book spells with an em dash would pass the screen
silently. **Recorded for the screen-fix list: add `'\u{2014}'` to `SIGN_CHARS`.**

**Truncation scan: not re-run.** The brief records the truncated-shipped-text
sweep as **CLOSED** after B15 (four entries, five strings, one root cause), so no
scan of that family was performed and none of this batch's findings is one. The
one adjacent observation worth the line is that `flaw.poor` and
`flaw.poor_characteristic` ship *rewritten* summaries rather than first-sentence
extracts in both locales — which is how `flaw.poor` manages to carry its three
mechanical clauses in a single sentence — and that `flaw.poor_characteristic`'s
rewrite is where **F-490** lives.

**German names against the canonical tables.** Located by English key, **18** of
the 29 distinct English names in this batch have a row in
`rules/source/de/translation-tables/`. **Fifteen agree** with the shipped
`rules/i18n/de/` name and with the DE rulebook heading; **three disagree**
(**F-496**), and one further row makes a factual claim the rulebook contradicts
(**F-497**). The remaining **11** English names have no table row and were checked
directly against the DE rulebook heading on the parallel line — **all 11 match ✓**.
(18 + 11 = 29 ✓, and the 29 distinct names cover 35 ids because six are
Major/Minor pairs.)

| English | table row | DE rulebook heading | shipped DE name | verdict |
|---|---|---|---|---|
| Outcast | `sphären-mächte.md:311` *Ausgestoßener* | 6538 Ausgestoßener | Ausgestoßener | ✓ |
| Outlaw | `reputationen.md:112-113` *Geächteter* | 6542 Geächteter | Geächteter | ✓ on the **name**; the row's magnitude and level columns contradict the book — **F-497** |
| Outsider | `reputationen.md:114` *Außenseiter* | 6550 Außenseiter | Außenseiter (Groß/Klein) | ✓, and its `1–3` column corroborates **F-486** |
| Overconfident | `tugenden-fehler.md:378` *Überheblich*; `persoenlichkeitseigenschaften.md:116` *Überheblich* | 6562 Überheblich | Überheblich (Groß/Klein) | ✓ — two rows, two *domains* (Flaw / Personality Trait), same German word, so not the two-magnitude-tables collision |
| Pagan | `tugenden-fehler.md:559` *Heide* | 6570 Heide | Heide | ✓ (note attributes the Flaw to *HM:RE*; the core book carries it at ArMDE:6570 — **Q-121** shape) |
| Painful Magic | `tugenden-fehler.md:699` *Schmerzhafte Magie* | 6574 Schmerzhafte Magie | Schmerzhafte Magie | ✓ (note attributes to *HoH:TL* — **Q-121** shape) |
| Palsied Hands | `tugenden-fehler.md:708` **Zittrige Hände** | 6578 **Zitternde Hände** | **Zitternde Hände** | **disagrees — F-496** (note also attributes to *HoH:TL*) |
| Pious | `persoenlichkeitseigenschaften.md:119` *Fromm* | 6586 Fromm | Fromm (Groß/Klein) | ✓ |
| Poor | `tugenden-fehler.md:441` *Arm* | 6594 Arm | Arm | ✓ (`RULES.md:6323-6325` already cites this row) |
| Poor Concentration | `tugenden-fehler.md:444` *Schlechte Konzentration* | 6602 Schlechte Konzentration | Schlechte Konzentration | ✓ (note claims the -3 for *SdM:M*; ArMDE:6604 states it — **Q-121** shape) |
| Poor Eyesight | `tugenden-fehler.md:445` *Schlechtes Sehvermögen* | 6606 Schlechtes Sehvermögen | Schlechtes Sehvermögen | ✓ |
| Poor Formulaic Magic | `tugenden-fehler.md:305` *Schwache Formulaische Magie* | 6610 Schwache Formulaische Magie | Schwache Formulaische Magie | ✓ |
| Poor Hearing | `tugenden-fehler.md:447` *Schlechtes Hörvermögen* | 6614 Schlechtes Hörvermögen | Schlechtes Hörvermögen | ✓ |
| Poor Memory | `tugenden-fehler.md:446` *Schlechtes Gedächtnis* | 6622 Schlechtes Gedächtnis | Schlechtes Gedächtnis | ✓ |
| Poor Student | `tugenden-fehler.md:448` *Schlechter Schüler* | 6626 Schlechter Schüler | Schlechter Schüler | ✓ |
| Primogeniture Lineage | `grundbegriffe.md:327` **Primogenitur-Abstammungslinie** | 6634 **Erstgeburts-Abstammung** | **Erstgeburts-Abstammung** | **disagrees — F-496** (the row's note also calls it "Kleiner **Hermetischer** Fehler", dropping the Story half) |
| Proud | `tugenden-fehler.md:379` *Stolz*; `persoenlichkeitseigenschaften.md:122` *Stolz* | 6642 Stolz | Stolz (Groß/Klein) | ✓ |
| Raised from the Dead | `tugenden-fehler.md:763` **Von den Toten Auferweckt** | 6646 **Vom Tode auferstanden** | **Vom Tode auferstanden** | **disagrees — F-496** (note also attributes to *SdM:G*) |

The **11** table-less names that match their DE heading: Outlaw Leader →
*Anführer von Geächteten*, Oversensitive → *Überempfindlich*, Pessimistic →
*Pessimistisch*, Plagued by Supernatural Entity → *Von übernatürlichem Wesen
geplagt*, Poor (Characteristic) → *Schlechte (Eigenschaft)*, Poor Living
Conditions → *Schlechte Lebensbedingungen*, Primitive Equipment → *Primitive
Ausrüstung*, Prohibition → *Verbot*, (Realm) Stigmatic →
*(Sphäre)-Stigmatisierter*, Raised in the Gutter → *In der Gosse aufgewachsen*,
Rebellious → *Aufsässig*.

One near-miss that is **not** a disagreement, checked and dismissed:
`flaw.realm_stigmatic` ships the DE name `Stigmatisierter, {realm}`, whose word
order differs from the DE heading `(Sphäre)-Stigmatisierter` — a **recorded
decision** at `crates/arm-rules/RULES.md:2372` (and argued at `:2356-2391`),
taken because the German realm labels are noun phrases carrying their own article.
Withdrawn, not counted. `crates/arm-rules/RULES.md` carries **no** corresponding
record for any of F-496's three, which is why they are reported rather than
withdrawn.

**`rules/core/abilities.json`, `rules/core/character_types.json`,
`rules/core/mythic_companion_types.json` and `rules/core/life_stages.json` were
consulted** for the Abilities, profiles, mythic types and rates these passages
name. Facts load-bearing for findings here, each re-derived:

- `categories_requiring_virtue` = `["academic","arcane","martial"]`;
  `ability.magic_lore` and `ability.faerie_lore` are both `arcane`;
  `ability.concentration`, `ability.charm`, `ability.guile` and
  `ability.etiquette` are all `general` (so `flaw.poor_concentration` and
  `flaw.raised_in_the_gutter` raise no authorization question).
- Grog `forbidden_traits` is `["virtue.the_gift"]` — it names
  **no** Flaw, which is **F-485**.
- Magus and mythic-companion `forbidden_traits` are both
  `["flaw.poor", "virtue.wealthy"]`; grog needs no entry because
  `max_major_flaws: 0` already blocks a Major Flaw (`RULES.md:6331-6337`) ✓.
- Every profile's `flaw_category_caps` covers `personality` and `story` only;
  **`social_status` appears in no caps array on any profile**, which is B12's
  **F-427** and is noted, not re-derived as a new claim.
- `life_stages.json`'s `later_life.xp_per_year` is **15**, so `flaw.poor`'s
  `later_life_xp_rate { amount: 10 }` replaces it with the figure ArMDE:2394
  states verbatim ✓.
- `mythic_companion_types.json`'s `mythic_type.spirit_votary` carries
  `required_flaws: [{ default: { ref: "flaw.pagan" }, constraint: { kind: flaw,
  magnitude: major, require_categories: ["personality"] } }]` — which is why
  **F-487**'s remedy has a second edit site.

### Part C systemic gaps are not re-reported per entry

**Six** of Part C's rows touch this batch and none is counted against an entry.
*(My first draft said five and omitted C3b, which the verification pass caught;
B09 named it explicitly at `batch-09.md:1659` when it met the same effect.)*

- **`LaterLifeXpRate` is unreachable without a life-stage plan** (C3b,
  `engine-semantics.md:2371-2379`) — that is `flaw.poor`. A character entered as a
  flat `xp_pool` never reaches `life_stage.rs::later_life_rate`, so the Flaw's
  `amount: 10` changes nothing at all on that path. The gap is the engine's,
  recorded once, and it is why `flaw.poor` is **clean** here rather than a
  finding: the entry is correctly authored against an engine that reads the effect
  on one of its two entry paths.

- **`HealthMod`'s three surfaced-only tracks** (C1) — that is `flaw.painful_magic`,
  the batch's only `health_mod` carrier, on the `casting_fatigue` track. The
  convention was re-derived rather than inherited: `casting_fatigue` has exactly
  three shipped carriers catalogue-wide — `flaw.painful_magic` **-1**,
  `flaw.vulnerable_casting` **-1** and `virtue.withstand_casting` **+1** — so a
  Flaw carrying a negative matches both its siblings and A36's stated convention
  ("positive reduces the penalty magnitude") ✓. That the track moves no number is
  the engine's gap, recorded once, and B12's **Q-98** (which way
  `HealthTrack::CastingFatigue`'s sign runs) is still the open question; this
  batch adds no evidence either way, because ArMDE:6576 states a *loss* rather
  than a modifier. What **is** counted against the entry is a different pair of
  clauses the effect never claimed to carry (**F-493**).
- **`AdvancementMod` is surfaced-only** (C1) — `flaw.poor_student` carries two
  rows (`taught` -3, `book` -3) and they are correct in sign, size and source
  against ArMDE:6628's "Subtract 3 from all Advancement Totals derived from
  teaching and books", with the carve-out list ("adventure experience, exposure,
  practice, studying from vis, or training") encoded by *omission* ✓. The gap is
  the engine's. What is counted is the floor clause (**F-492**).
- **`CombatMod` folds conditional amounts unconditionally** (A35) — touches
  `flaw.palsied_hands` and `flaw.poor_eyesight`, and on both the fold is
  *correct*, because neither figure is conditional on a weapon class: ArMDE:6580's
  -2 covers "all rolls involving holding or wielding an object … including weapon
  skills" and ArMDE:6608's -3 covers "rolls involving sight, including rolls to
  attack and defend". The Attack+Defense reading for both is argued in full and
  pinned at `RULES.md:5005-5065` and
  `data_integrity.rs::the_combat_roll_flaws_penalize_the_combat_ability_totals` ✓.
  ArMDE:6608's explicit *compatibility* with Missing Eye ("the penalties are
  cumulative") is satisfied by A35's plain sum, so the absent `incompatible_with`
  on both is correct ✓ — B15's reading of the same sentence from the other side,
  re-derived.
- **`incompatible_with` is validated over bought selections only** (C4a,
  documented as deliberate at `validation/prereq.rs::PrereqCtx::build`) — this
  touches **F-482** (and B09's **F-340**, which it extends), and it is
  **harmless for both**:
  `virtue.wealthy` and `flaw.poor` are reachable through no grant (neither appears
  in any `granted_selections`, any `grants_selection` effect, any House grant or
  any mythic-type grant — checked across `houses.json`,
  `mythic_companion_types.json`, `childhoods.json`, `life_stages.json` and
  `character_types.json`), so every copy a player can hold is **bought** and the
  proposed `incompatible_with` declarations would fire. B15's F-466 correction
  (`Prereq::Nor` where a grant is reachable) therefore does **not** apply here,
  and saying so is the point of the bullet.
- **`creation_effect` is not required to carry effects** (C7) — live in this
  batch: two of its recommendations (**F-488**, **F-494**) move entries *into*
  classes whose accompanying effect no guard will notice if it is forgotten.

Two further catalogue-wide gaps are named by clauses in this span and are
**recorded, not counted**: a Reputation's *negativity* is not representable
(B10's **F-380**), which affects **four** of this batch's five Reputation-bearing
clauses (`flaw.outlaw` "a Reputation at level 2 for whatever got you outlawed",
`flaw.outlaw_leader` "well known as an outlaw", `flaw.outsider_*` "a **bad**
Reputation", `flaw.raised_from_the_dead` "a level 4 reputation"); and there is no
`Effect` variant that names *which Ability* a roll modifier applies to — `A41
AbilityRollMod` carries only a free-text subject and no ability field at all — so
`flaw.poor_concentration`'s -3 Concentration, `flaw.poor_hearing`'s -3 hearing and
`flaw.raised_in_the_gutter`'s -3 Charm/Guile/Etiquette are **all** genuinely
inexpressible. That inexpressibility is D3 grounds for `uncomputed_rule`, which is
what two of the three already are and what **F-489** asks for the third.

### Decisions applied

`docs/vf-audit/decisions.md` was read in full and is binding. It carried D1-D6
when this batch read it, **including D6's 2026-09-21 correction**, read in its
corrected two-rule form.

- **D3** governs one entry here and is the whole basis of **F-489**:
  `flaw.poor_hearing`'s -3 is inexpressible (see the A41 note above), and
  inexpressibility is grounds for `uncomputed_rule` with the rule written out —
  never for `narrative`. It also *confirms* two existing classifications:
  `flaw.poor_concentration` and `flaw.raised_in_the_gutter` are `uncomputed_rule`
  for exactly this reason, with their full passages in both locales ✓.
- **D5** is the reason three entries that compute correctly still fail —
  `flaw.poor_eyesight` (**F-491**), `flaw.poor_student` (**F-492**) and
  `flaw.painful_magic` (**F-493**) — and the reason a fourth passes:
  `flaw.palsied_hands` ships its whole passage as `description` in both locales
  although its class does not oblige it, which is what carries the extra-botch-die
  clause the engine cannot compute ✓. D5 also adds the text half to
  **F-483**, **F-485**, **F-487**, **F-488** and to B09's **F-340** on
  `flaw.outcast`.
- **D6** governs nine table rows here, and **both** of its rules are used.
  Rule 2 (terminology → the glossary wins) puts three shipped German names in
  dispute with three table rows (**F-496**); per D6's closing paragraph these are
  reported as *"this repository's copy disagrees with the rulebook"* and the
  determination is the orchestrator's, because `arm-de-translation` cannot be
  read from here. Rule 1 (a factual claim about the rules → the rulebook wins)
  condemns `reputationen.md:112-113` outright (**F-497**) and, **as corrected
  after the verification pass**, the five `Anmerkung` cells making book-provenance
  claims the core book contradicts (`tugenden-fehler.md:444`, `:559`, `:699`,
  `:708`, `:763`) as well — **F-501**. My first draft escalated those five to
  B15's **Q-121** while condemning `:112-113`, which is two standards for one
  rule inside one batch: D6 rule 1 names "from which book" in the same breath as
  magnitude and level. Q-121 is not withdrawn, but it is now the broader question
  of what an `Anmerkung` is *for*, not the disposition of these five rows.
- **D1 and D4** touch nothing here: no entry in this batch carries
  `lab_total_mod`, and none of D1/D4's nine is in the span.
- **D2** touches one entry and is **not** triggered. `flaw.poor_characteristic`
  carries `characteristic_score_delta_param`, whose precondition check
  (`validate_characteristic_delta_preconditions`) is bought-only — but the
  Flaw is reachable through no grant, so no granted copy can bypass it, and D2's
  "no batch rates the bought-only validators either correct or defective" holds
  with nothing to rate. The entry's own encoding (`amount: -1`,
  `max_per_target: 2`, buy floor `-3` from `characteristics.json`) is argued in
  full at `RULES.md:1429-1450` and is correct ✓; its failure (**F-490**) is in the
  shipped *text*, not the data.

### Cross-references followed and rated

Every pointer in the span was followed and read, including on entries that
passed. **The inbound side was swept deliberately as a method step**: each of the
29 English headings in the span was searched for by name across the whole core
book, outside its own range and outside the "List of Flaws" index, and every hit
was read. That returned **31** inbound sites, counting ArMDE:2899-2900's two-row
table as one:

> ArMDE:2214, :2313, :2394, :2645, :2756, :2778, :2794, :2899-2900, :3400, :3406,
> :3547, :3573, :3611, :3623, :3631, :3689, :3845, :3961, :4043, :4197, :4293,
> :4323, :4391, :4484, :4494, :4504, :4508, :5135, :6436, :6719, :6983.

**Six of the 31 expose a defect, but only three of the six are new to the audit.**
ArMDE:3611, :3631 and :4494 turned out to be already in **B09's F-340**; the new
ones are **ArMDE:3406** and **:4043** (both **F-482**), **ArMDE:4800** (found by a
later `rg` sweep rather than by this list, and left to B07), and **ArMDE:6719**
(**F-498**). A further site, ArMDE:2756, exposes no defect but constrains
**F-487**'s remedy. The other 24 are suggestions, restatements, permissions or
already-encoded siblings; every
one is rated below, the twelve-strong Wealthy/Poor "control group" in a single row
because they say the same thing in the same words. Sample characters, creature
templates and Initiation scripts that merely *list* a Flaw were excluded as noise
after checking that none states a rule — ArMDE:1277, :1382, :1384, :1461, :1463,
:1486, :1502, :1504, :1597, :1668, :1766, :2014, :2117, :10037, :17519, :18430,
:19123, :19188, :19256, :19325, :20839, :23126. ArMDE:4225 was excluded on a
different ground: it names Wealthy alone, and `virtue.wealthy` is not in this
span.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the entry? |
|---|---|---|---|
| `flaw.outcast` | "**You may not take the Wealthy Virtue.**" (ArMDE:6540) / DE "**Du darfst nicht die Tugend Wohlhabend nehmen.**" | `virtue.wealthy` (ArMDE:5235-5238), whose `incompatible_with` is **absent** | **YES — but it is already B09's F-340, whose passage table lists this very line.** Neither side declares the edge, so the pair validates clean today. `virtue.wealthy` is bought-only, so a declared `incompatible_with` on both sides would fire — `ruleset/integrity.rs::validate_incompatibility_symmetry` requires both edges. My **F-481** is withdrawn as a duplicate; what the pointer genuinely adds is in **F-482**. |
| `flaw.outcast` | *(inbound)* ArMDE:6983, `flaw.viaticarus` (**B18**'s span) — "If the character's status is known in the community in which he lives, he **should** take the Outcast Social Status Flaw." | `flaw.viaticarus` (`uncomputed_rule`, empty `incompatible_with`) | **No.** A conditional recommendation ("If … should"), and it would additionally collide with ArMDE:2816's one-Social-Status rule, which is enforced nowhere (**F-427**). Followed, rated, discarded. |
| `flaw.outcast` / `flaw.outlaw` / `flaw.outlaw_leader` / `flaw.outsider_*` | *(inbound)* ArMDE:2899-2900, the "All Cultures" Social Status table — "**Minor Flaws**: Branded Criminal, Companion Animal, **Outcast, Outlaw Leader, Outsider**, Usurer / **Major Flaws**: **Outlaw, Outsider**" | the five Social Status Flaws in this span | **No new rule, and one useful confirmation plus one silence.** The table's magnitudes match every descriptor and every index filing ✓. What it does **not** carry is Outlaw Leader's grog exclusion, so ArMDE:6548 is the only site that states it — **F-485**. |
| `flaw.outlaw` | "You **may take Martial Abilities at character generation**, and have a **Reputation at level 2** for whatever got you outlawed." (ArMDE:6544) / DE "Du **darfst Kampffertigkeiten bei der Charaktererschaffung erwerben** und hast eine **Reputation der Stufe 2**" | `abilities.json`'s `categories_requiring_virtue` (`martial` is gated); `A14 AbilityAuthorization`; `A25 GrantsReputation` | **YES, twice — F-483 and F-484.** The permission is unencoded, so a grog or companion Outlaw who buys a weapon skill raises a hard `ability_category_requires_virtue`. And the grant hardcodes `kind: "local"` where the sentence names no audience at all, which is F-450's shape — `A25`'s `kind` is `Option<ReputationType>` and `None` means *player-chosen*, so the correct encoding is to omit it. |
| `flaw.outlaw` | "Outlaw followers created as grogs **should take** the Branded Criminal, Outcast or Wanderer social status." (ArMDE:6544) | `flaw.branded_criminal`, `flaw.outcast`, `virtue.wanderer` | **No.** Advice about the *followers*, who are separate characters; it attributes nothing to the Outlaw himself. Followed and discarded. |
| `flaw.outlaw_leader` | "You are well known as an outlaw **in the local area**, with a **Reputation level of 3**. … You **may take Martial Abilities at character generation**. **Grogs may not take this Flaw.**" (ArMDE:6548) | `A25 GrantsReputation` (`kind: "local"` ✓ — this one *does* name the audience); `A14`; grog `forbidden_traits` | **YES, twice — F-483 and F-485.** The Reputation is right in kind and score ✓, which is exactly what makes `flaw.outlaw`'s hardcoded `local` a defect rather than a convention: the book names the audience here and not there, and the data is identical. The Martial permission is the same live hard error. And "Grogs may not take this Flaw" is a flat prohibition on a *type*, expressible as a `forbidden_traits` row, encoded nowhere. |
| `flaw.outsider_major` / `flaw.outsider_minor` | "You have a bad Reputation of **level 1 to 3** (depending upon how easy it is to identify you) among members of the dominant social group of your area." (ArMDE:6554) and "You **still have the bad Reputation** among members of the dominant social group, but do not meet them so often." (ArMDE:6556) | `A25 GrantsReputation` (`score: u8`); `crates/arm-rules/RULES.md:5384-5385`; `rules/source/de/translation-tables/reputationen.md:114` | **YES — F-486.** The book gives **one** range to **both** versions and varies it by *identifiability*, not by magnitude; the data splits it into a fixed 3 and a fixed 1. RULES.md:5384-5385 records "bad score **1-3**" for both ids, so the traceability map describes data the catalogue does not contain — the same in-repo contradiction B15 found at `RULES.md:5161`. The translation table's `1–3` column agrees with the book, from a third source. |
| `flaw.outsider_*` | "As a general rule, **companions should take the Major version** … **Grogs may take the Minor version**." (ArMDE:6558) | `character_types.json`'s `forbidden_traits` | **No — a soft guideline, and it is Q-119's family.** "Should"/"may" is not a block, and `forbidden_traits` can only express a hard one. Not counted; added to **Q-126** rather than decided. |
| `flaw.outsider_*` | "the troupe should decide on a suitable Social Status … This should normally be a **Free Social Status Virtue**." (ArMDE:6560) | nothing in the engine | **No.** An in-play troupe decision about a saga that travels; no creation-time rule. Followed and discarded. |
| `flaw.outsider_*` | *(inbound)* ArMDE:4484 (`virtue.mobed`) and ArMDE:5135 (`virtue.templar_specialist`) — "you **may also possess** the Outsider Flaw", "who **should also take** the Outsider Flaw **or similar**" | the two Virtues | **No.** Both are suggestions, hedged twice in the second case. Followed and discarded. |
| `flaw.pagan` | "You **may begin with Magic Lore or Faerie Lore**, depending on the specifics of your faith." (ArMDE:6572) / DE "Je nach den Besonderheiten deines Glaubens **darfst du Magiekunde oder Feenkunde zu Beginn erwerben**." | `ability.magic_lore` and `ability.faerie_lore`, both `category: "arcane"`; `A14 AbilityAuthorization` | **YES — F-488**, F-409's **37th** stated instance. Identical in mechanism *and in wording* to B15's F-460 (`flaw.magical_fascination`, ArMDE:6394, "either Magic or Faerie Lore"), 178 lines later in the same book. Two entries, one idiom, one unencoded permission each. |
| `flaw.pagan` | "(… **It may also be a Minor Flaw for grogs** who live at a covenant with a substantial pagan population …)" (ArMDE:6572) + descriptor "*Major or Minor, Personality*" (ArMDE:6571) | the catalogue: one id, `magnitude: "major"`; `RULES.md:3923` | **YES — F-487.** The book states the Minor version twice (descriptor, and this sentence naming who it is for) and the index files it a third time at ArMDE:5331. No `flaw.pagan_minor` exists, and a grog cannot hold a Major Personality Flaw, so the very character the sentence describes is unbuildable. |
| `flaw.pagan` | *(inbound)* ArMDE:2756, the Spirit Votary mythic-companion type — "**Pagan (Major, Personality)**" as a required Flaw | `mythic_companion_types.json:69-84`, `required_flaws[0].default.ref = "flaw.pagan"`, `constraint.magnitude = "major"` | **No new rule, and it is F-487's second edit site.** The seeding is correct as it stands ✓ and `RULES.md:3855` prices it. But if the remedy renames `flaw.pagan` → `flaw.pagan_major`, this ref moves with it; if instead a `flaw.pagan_minor` is *added* beside the existing id, nothing here changes. Recorded so the correction pass chooses deliberately. |
| `flaw.pagan` | *(inbound)* ArMDE:2313 — "a pagan character does not need to have the Pagan Flaw — the Flaw indicates that their religion **creates stories or shapes their personality**" | the V/F chapter's general advice | **No.** A statement about concept-vs-Flaw that applies to the whole catalogue and is illustrated with this entry — B14's F-453 narrowing, applied. Followed, rated, not counted. |
| `flaw.poor` | *(inbound)* ArMDE:2394 — "Characters with the Wealthy Virtue get **20** experience points per year, while characters with the Poor Flaw get **10** … Note that **only companions** can take this Virtue or Flaw." (and ArMDE:2214 restates the 15/20/10 triple) | `A12 LaterLifeXpRate`; `life_stages.json`'s `later_life.xp_per_year: 15`; the four profiles' `forbidden_traits` | **No unrecorded rule — the entry is fully sourced and correct.** The rate *replaces* rather than adjusts, which is right because the passage states the whole rate ✓; magus and mythic companion carry the id in `forbidden_traits` and grog is blocked by `max_major_flaws: 0` ✓; all argued at `RULES.md:6312-6337`. The entry's own "one fewer season" is a consequence of the rate, not a second mechanic. |
| `flaw.poor` | *(inbound, ×5)* ArMDE:3406 `virtue.almogavar` — "He is supported by his unit, and **may not take the Poor Flaw or Wealthy Virtue**."; ArMDE:3611 `virtue.covenfolk` — "**You may not take the Wealthy Major Virtue or the Poor Major Flaw.**"; ArMDE:3631 `virtue.custos` — "you **may not take the Wealthy Virtue or Poor Flaw**."; ArMDE:4043 `virtue.guild_apprentice` — "The character is **not able to benefit from** either the Poor Flaw or the Wealthy Virtue"; ArMDE:4494 `virtue.mendicant_friar` — "**You may not take the Wealthy Virtue or Poor Flaw.**" | all five Virtues; `incompatible_with` on each is **empty**, and a catalogue-wide `jq` finds **zero** entries naming `virtue.wealthy` or `flaw.poor` in any `incompatible_with` | **Partly — and the honest answer is smaller than my first draft's.** Three of the five (:3611, :3631, :4494) are already in **B09's F-340**. Only **:3406** and **:4043** are new, and a later `rg` sweep added a third site, **:4800** (`virtue.priest`, conditional). What my pass genuinely contributes is those three plus the reachability proof that retires F-340's granted-Redcap caveat — see **F-482**, restated as an extension of F-340 rather than as a finding of its own. |
| `flaw.poor` | *(inbound, the control group ×13)* ArMDE:3400, :3547, :3573, :3623, :3689, :3845, :3961, :4197, :4293, :4323, :4391, :4504, :4508 — "The Wealthy Virtue and Poor Flaw **affect you normally**", "you may choose Wealthy or Poor", "unless you take the Poor flaw and must work a third season" | thirteen Social Status Virtues, and **at least twenty** once the verification pass's additions (:4600, :4604, :4618, :4622, :4802, :4858, :4914, :5225) are counted | **No — and this is what keeps the prohibition list honest.** At least twenty passages mention the pair and *permit* it, ten forbid it flatly. The distinction is lexical and unambiguous ("may not take" vs "affect you normally"), so the ten are not a sampling artefact. I did **not** enumerate the permissions exhaustively the way I did the prohibitions, hence "at least". ArMDE:3689's "if he is Poor, for which he receives a Bad Reputation at a level of 2" is a conditional grant belonging to `virtue.jurist`, noted and left to its own batch. |
| `flaw.outsider_*` / `flaw.poor` | *(inbound)* ArMDE:2214 — "**15** experience points per year … Characters with the Wealthy Major Virtue get **20** … those with the Poor Major Flaw get **10**"; ArMDE:2794 — "every character must have one of these [Social Statuses]"; ArMDE:5135 (`virtue.templar_specialist`) — "who should also take the Outsider Flaw **or similar**" | `life_stages.json`'s `later_life.xp_per_year: 15`; ArMDE:2816 / B12's **F-427**; `virtue.templar_specialist` | **No new rule on any of the three.** ArMDE:2214 restates ArMDE:2394's triple and confirms the base rate the data replaces ✓; ArMDE:2794 is ArMDE:2816's rule said twice and is F-427, noted not re-derived; ArMDE:5135 is a hedged suggestion. Followed, rated, discarded. |
| `flaw.poor_characteristic` | "lower one which is **already —3 or lower** by one point … You may take this Flaw **twice for a single Characteristic**, lowering it to —5, and **multiple times for different Characteristics**." (ArMDE:6600) | `max_per_target: 2` keyed on `(item_ref, params)`; `validation/selections.rs::validate_duplicate_selections`; `characteristics.json`'s `base_min` = -3; `RULES.md:1429-1450` | **No data defect — and this is ArMDE:2814 done right.** `max_per_target: 2` allows exactly two copies naming one Characteristic and, with `max_total` correctly absent, unlimited copies naming different ones. That is the sentence, both halves. The failure is in the shipped *summary* (**F-490**), which narrows "-3 **or lower**" to "at -3" and adds a points claim the passage does not make. |
| `flaw.poor_eyesight` | "This Flaw **can be combined with Missing Eye, but the penalties are cumulative**." (ArMDE:6608), mirrored at ArMDE:6436 | `flaw.missing_eye` (**B15**'s span); `A35 CombatMod`'s plain sum | **No new rule, and the encoding is right on this side.** An explicit *compatibility*, so the absent `incompatible_with` on both is correct ✓, and A35 sums unscoped figures so the two stack exactly as the sentence says ✓. The pair's arithmetic is right in melee (-3 + -1 = -4) and wrong at range, which is B15's **F-462** restated, not a second defect. |
| `flaw.poor_student` | "Subtract 3 from all Advancement Totals derived from teaching and books (that is, you have **no penalty** to adventure experience, exposure, practice, studying from vis, or training), but **do not reduce a total below one**." (ArMDE:6628) | `A39 AdvancementMod`'s nine `source` values; `derived.rs::surfaced_modifiers` | **YES — F-492, for the floor only.** The two encoded rows and the five-way carve-out are exactly right ✓ — the carve-out is encoded by omission, which is the only way A39 can express it. The floor of 1 is a clamp on a total the app never computes, so it is inexpressible (D3) and owed as text (D5), where it is absent in both locales. |
| `flaw.poor_living_conditions` | "The character has an **additional –1 Living Conditions Modifier**. This is **cumulative** with the character's base Living Conditions Modifier." (ArMDE:6620) | `A38 AgingMod { living_conditions }` → `aging.rs::living_conditions_modifier`; `virtue.mild_aging`'s **+1** at ArMDE:4530 | **No defect, and the sign was re-derived from both ends rather than assumed.** A38: `from_traits += amount`, and the aging total *subtracts* the summed modifier — so a stored **-1 raises** the total, which is the penalty the passage states ✓. ArMDE:4530 confirms the polarity from the opposite side ("a **+1 bonus** to the Living Conditions Modifier"), and the pair is pinned by `mild_aging_and_poor_living_conditions_move_the_total_in_opposite_directions`. "Cumulative" is A38's plain sum ✓. |
| `flaw.poor_formulaic_magic` | "Subtract 5 from every roll that you make to cast Formulaic spells. **This does not apply to Ritual spells.**" (ArMDE:6612) | `A30 CastingTotalMod { scope: formulaic }`; `derived.rs::CastType::matches` | **No defect — and the Ritual carve-out is the reason to read the consumer.** `CastType::Ritual` accepts `all | ritual | formulaic_ritual` and **not** `formulaic`, so `scope: "formulaic"` excludes Rituals exactly as the second sentence requires ✓. Had the data used `formulaic_ritual` — the scope that exists precisely for spells that are both — the carve-out would have been silently reversed. |
| `flaw.painful_magic` | "This reduces all your actions by the appropriate Fatigue penalty, which is **cumulative with any from actual fatigue or injuries** (though you do not suffer any physical damage from pain). You **recover these \"pain levels\" just like Fatigue levels**." (ArMDE:6576) | `A36 HealthMod`'s five `HealthTrack` members; `derived/combat.rs::fatigue_levels` | **YES — F-493.** Two rules about a quantity the engine does not model: pain levels are a *parallel* track that stacks with the Fatigue ladder and recovers on it, and none of the five members names a track that can hold them. Inexpressible (D3), computed nowhere, and in neither locale. |
| `flaw.primitive_equipment` | "the base **Craft Value (see City & Guild, page 67)** is raised by 1-3 … her workshop may not be improved with regards to **Innovation (City & Guild, page 65)**" (ArMDE:6632) | **Nowhere — *City & Guild* is not in `rules/source/en/`.** | **No rule this repository may implement.** The pointer cannot be followed; unlike B14's `flaw.independent_craftsman` the sentence states no fallback. Nothing recategorizes and nothing is owed beyond the text — which ships in full in `description` in both locales ✓, the 1-3 range included. Recorded so it is not re-chased. |
| `flaw.primogeniture_lineage` | "This Flaw **can only be taken by magi of House Verditius**" (ArMDE:6636) | `prerequisites: all(is_magus, house.verditius)`; `RULES.md:585-632`; `data_integrity.rs::primogeniture_lineage_requires_a_magus_of_house_verditius` | **No defect — the one restriction in this span that *is* encoded.** It is also `uncomputed_clauses.rs`'s own written reading of the entry (`NO_RULE_DESPITE_TOKEN`, line 434): the "at least three places removed" that trips the screen states nothing, and the real rule is computed rather than described. Re-derived at the prerequisite, and the exemption's reading **confirmed**. |
| `flaw.prohibition` | *(inbound)* ArMDE:6719, `flaw.servant_of_the_land` (**B17**'s span) — "while it remains incomplete the character has the **Minor Personality Flaw: Prohibition**, but this **does not count toward the character's total number of Virtues and Flaws**." | `A18 GrantsSelection` (budget-exempt, because `compute_balance` iterates **bought** selections only); `flaw.servant_of_the_land`'s **empty** `effects` | **YES for the target — F-498 — and a book self-contradiction for this entry.** The grant is the exact `virtue.ineslemen` shape B15 rated as the catalogue's one fully-correct inbound encoding, and `flaw.servant_of_the_land` carries no `grants_selection` at all. Separately, ArMDE:6719 calls Prohibition a *Personality* Flaw while ArMDE:6639 and the index at ArMDE:5552 both say *Supernatural* — two sources against one, so the shipped `categories: ["supernatural"]` is right ✓ and the contradiction is escalated as **Q-125**. |
| `flaw.raised_from_the_dead` | "You begin with **at least three Warping points**, plus **one Warping point for every year** that has passed since you were resurrected, and you automatically receive **another Warping point every year** you continue living. You also have a **level 4 reputation** in the area where the miracle occurred." (ArMDE:6648) | `A22 WarpingGrant { score, points }` → `effective/warping.rs::warping_points_total`; `A25 GrantsReputation`; `effective/warping.rs::WarpingOwed::from_score` | **YES — F-494, the batch's `uncomputed_rule` → `creation_effect` transition.** Two of the four clauses are expressible with variants that already exist and are used, and the entry carries no `effects` at all. The per-year accrual genuinely is not modelled (the app fixes no resurrection date), so it stays text — but "at least three" and "level 4" are not. |
| `flaw.realm_stigmatic` | "**Pick one** of the four Realms of Power; whenever he enters an **aura of strength 4 or more** aligned to that realm, or is affected by a power **greater than fourth magnitude (20th level)** from that realm, he suffers from the stigmata." (ArMDE:6656) | the `realm` parameter (domain `realm`) with **no** `max_total`; `validation/selections.rs::validate_duplicate_selections`; ArMDE:2814 | **YES — F-495.** "Pick one" is singular and ArMDE:2814 forbids a repeat by silence, but `max_per_target` defaults to 1 keyed on `(item_ref, params)`, so four copies naming four different Realms are four legal tuples. B15's **F-469** shape (`flaw.magical_being_companion`), in a new place — and the fix `flaw.necessary_realm_aura_for_ability` uses (`max_per_value: 1`) is the *wrong* one here, because the book permits no second copy at all: this needs `max_total: 1`. The aura and magnitude thresholds are correctly left uncomputed and ship in both locales ✓. |
| `flaw.plagued_by_supernatural_entity` | *(inbound, ×2)* ArMDE:2778 — "if you do not want to involve demons in your saga, no character can take **Plagued by Demon** as a Flaw"; ArMDE:2645 — "This often results in the **Plagued by an Angel** or Supernatural Nuisance Story Flaws" | the entry, which carries **no** parameter | **No rule, but a naming question — Q-128.** Both sites, plus two sample characters (ArMDE:1502 "Plagued by Angel", ArMDE:2014 "Plagued by Faerie"), write the Flaw as if it were parameterized by entity type, while its own heading (ArMDE:6590) is generic and its examples are prose. The *absence* of a parameter is what makes `max_per_target`'s default correctly forbid a second copy ✓, so there is no defect today. Escalated rather than decided. |
| `flaw.plagued_by_supernatural_entity` | *(inbound)* ArMDE:2778 — "if a Story, Social Status, or Personality Flaw will **not enhance stories**, a character should not be allowed to take it" | the V/F chapter | **No.** A troupe-judgement rule about a whole Flaw *class*, illustrated with this entry — the same construction as ArMDE:2782's "such as Blind or Mute" that B15 rated and did not count. Followed and discarded. |
| `flaw.pessimistic` / `flaw.pious_*` / `flaw.proud_*` / `flaw.overconfident_*` | *(inbound)* ArMDE:2804, :2806, :2968 — "You might take … **Pious** as the Major Personality Flaw"; "A **Pious** character who wants to go on a pilgrimage … [is] just as driven by [his] Personality Flaw as a character with Hatred" | the worked examples and the Major-Personality-Flaw advice | **No.** Concept suggestions and an illustration of ArMDE:2968's "a Major Personality Flaw should always be something that makes the character act". Followed and discarded. |

### ArMDE:2814, :2816, :2818, :2820 — instances in this batch

ArMDE:2812-2820 was re-read in full for this batch rather than taken from B10,
B12, B13, B14 or B15.

- **ArMDE:2814** ("A Virtue or Flaw may be taken more than once only if the
  description explicitly allows it. Most Virtues and Flaws may only be taken
  once.") — **one** explicit permission in the span and the data gets it right,
  plus one parameterized entry that the data gets wrong.
  `flaw.poor_characteristic` ArMDE:6600 ("twice for a single Characteristic …
  multiple times for different Characteristics") → a `characteristic` parameter
  with `max_per_target: 2` and no `max_total`, so two copies per Characteristic
  and any number across Characteristics ✓ — exactly the sentence.
  Against it, `flaw.realm_stigmatic` carries a `realm` parameter and **no
  `max_total`** where ArMDE:6656 says "Pick **one** of the four Realms" and grants
  no repeat (**F-495**). Stated no wider than checked: I read the passages of the
  two parameterized entries in my span only.
  **The other 33 entries are unparameterized with no `max_total`**, so
  `max_per_target`'s default of 1 keyed on `(item_ref, {})` already forbids a
  second copy, which is what the book requires — with no exception in this span.
- **ArMDE:2816** ("All characters must take one Social Status…") — **five
  instances**, the first Social Status Flaws the audit has reached and the densest
  such span in the book: `flaw.outcast`, `flaw.outlaw`, `flaw.outlaw_leader`,
  `flaw.outsider_major`, `flaw.outsider_minor`. B12's **F-427** is not re-derived
  here and gains no new *claim*; what is re-derived is the profile data behind it,
  and it holds: `social_status` appears in every profile's `permitted_categories`
  and in **no** profile's `flaw_category_caps`, so a character may hold five Social
  Status Flaws where the book requires exactly one. ArMDE:2794 states the rule a
  second time ("every character must have one of these") and ArMDE:2899-2900
  enumerates the legal set.
- **ArMDE:2818** ("A character should not have more than one Story Flaw") —
  **two** instances: `flaw.plagued_by_supernatural_entity` and
  `flaw.primogeniture_lineage`. Enforced on all four profiles, re-derived rather
  than inherited: every profile carries `{"category":"story","max":1}` (grog
  `max: 0`) ✓.
- **ArMDE:2820** ("A character may not have more than one Major Personality Flaw
  … should normally not have more than two Personality Flaws in total") — the
  span's densest rule, with **six** Major/Minor pairs and two Minor singles.
  `overconfident`, `oversensitive`, `pious`, `proud` and `rebellious` each carry a
  mutual `incompatible_with` ✓, additionally load-enforced by
  `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`. `outsider_major`
  / `outsider_minor` carry the same mutual pair ✓ although they are Social Status
  rather than Personality. Both halves of the cap are live and re-derived: every
  profile carries `{"category":"personality","max":1,"major_only":true,"hard":true}`
  and `{"category":"personality","max":2}` (grog 0 and 1) ✓. Two Minor singles
  join them (`pessimistic`, `poor_memory`), and **`flaw.pagan` is the one entry
  that should be a pair and is not** (**F-487**) — so it alone escapes
  `validate_magnitude_variant_exclusivity` for want of a partner.
- **ArMDE:2960-2962** (Supernatural realm association) — **three** instances:
  `flaw.prohibition`, `flaw.raised_from_the_dead`, `flaw.realm_stigmatic`. Only
  the last carries a `realm` parameter. Noted only; it is B11's standing
  observation, not a new one. `flaw.raised_from_the_dead` is the interesting case
  — a Divine miracle, in a book whose only realm marker for a V/F is the Infernal
  `tainted` boolean, so the association is unrecordable in either direction.

### The exact wording the phrase screen missed

The whole span sits inside `SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` block, swept
2026-09-15 and re-swept 2026-09-19, green both times. The third assertion
(`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause`) runs
`states_a_mechanical_rule` over the **English source passage**, not over the
shipped text — re-derived by reading `bracketed_passage`, which loads
`rules/source/en/<file>` — so the list below is what the screen read and passed.
Three of this batch's 18 `narrative` entries state a rule and the screen saw none
of them. Verbatim, in both locales, for the screen-fix list:

1. **`flaw.outcast` — the prohibition idiom** (B12's second named missing family).
   - EN (ArMDE:6540): `You may not take the Wealthy Virtue.`
   - DE (6540): `Du darfst nicht die Tugend Wohlhabend nehmen.`
   `states_a_mechanical_rule` is false: no sign (the passage's only dash is the
   U+2014 in "own—normal", not followed by a digit), no botch term, and
   `MECHANICAL_PHRASES` carries no prohibition form at all. A `"may not take"` /
   `"darfst nicht"` / `"dürfen nicht"` literal would catch it — and would also
   catch `flaw.outlaw_leader`'s "Grogs may not take this Flaw" (ArMDE:6548 / DE
   "Grogs dürfen diesen Fehler nicht nehmen"), which is in a `creation_effect`
   entry the third assertion never examines.

2. **`flaw.pagan` — the permission idiom** (B12's first named missing family, and
   B15's F-460 in the same words).
   - EN (ArMDE:6572): `You may begin with Magic Lore or Faerie Lore, depending on the specifics of your faith.`
   - DE (6572): `Je nach den Besonderheiten deines Glaubens darfst du Magiekunde oder Feenkunde zu Beginn erwerben.`
   No sign, no botch term, no phrase. The same sentence's companion — `It may
   also be a Minor Flaw for grogs who live at a covenant with a substantial pagan
   population` / `Es kann auch ein Kleiner Fehler für Grogs sein, die in einem
   Konvent mit einer erheblichen heidnischen Bevölkerung leben` — states a
   *magnitude*, which no token in the list quantifies either.

3. **`flaw.poor_hearing` — a new family: the bare imperative "Subtract N".**
   - EN (ArMDE:6616): `Subtract 3 from rolls involving hearing.`
   - DE (6616): `Ziehe 3 von Würfen ab, die das Hören beinhalten.`
   This is the sharpest miss in the batch, because the rule is the entry's *whole
   first sentence* and is a plain numeric modifier — it is invisible only because
   the book writes "Subtract 3" instead of "-3". `has_signed_number` requires a
   sign character immediately before the digit; there is none. The same idiom
   appears twice more in this span, in entries the assertion does not reach:
   `flaw.poor_student` ArMDE:6628 `Subtract 3 from all Advancement Totals` / DE
   `Ziehe 3 von allen Fortschrittssummen ab`, and `flaw.poor_formulaic_magic`
   ArMDE:6612 `Subtract 5 from every roll` / DE `Ziehe 5 von jedem Wurf ab`.
   **Recommended literals:** `"subtract "` and `"ziehe "` / `"ziehen "` — narrow
   enough to need no boundary argument, and the German `abziehen` is discontinuous
   so the stem must be the front half.

Two further screen observations, neither of which cost a finding here but both of
which are latent:

- **`SIGN_CHARS` omits U+2014 EM DASH**, which ArMDE:6600 uses for both of its
  negatives ("already —3 or lower", "lowering it to —5"). It is harmless in this
  span only because that entry is `creation_effect`.
- **`NO_RULE_DESPITE_TOKEN`'s two readings that touch this batch were checked
  against the passages and both are correct.** `flaw.overconfident_major` /
  `_minor` (ArMDE:6564, "If you actually botch, you come up with some
  rationalization as to what 'really' happened") states nothing about botch
  *dice* ✓, and `flaw.primogeniture_lineage`'s "at least three places removed"
  quantifies a fictional succession ✓ while its one real rule is computed as a
  prerequisite ✓. No finding against either exemption.

## Verdicts

35 rows, one per entry. `class` / `data` / `text` report checks 2, 3-11 and 12.
"OK" means every check in that column passed; `?` means escalated, not resolved.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.outcast` | 6538-6541 | narrative ✓ | **`incompatible_with` omits `virtue.wealthy`, stated at ArMDE:6540 and encoded on neither side** | the prohibition in neither locale | **B09's F-340** (row eight) — **F-481 withdrawn as its duplicate**; ArMDE:2816 instance |
| `flaw.outlaw` | 6542-6545 | creation_effect ✓ | **no `ability_authorization` for `martial`: buying the weapon skill the Flaw permits is a hard validator error today**; and `grants_reputation.kind: "local"` where the passage names no audience | the Martial permission and the Reputation in neither locale | **F-483**, **F-484**; ArMDE:2816 instance |
| `flaw.outlaw_leader` | 6546-6549 | creation_effect ✓ | **no `ability_authorization` for `martial`**; **"Grogs may not take this Flaw" is in no `forbidden_traits`**; the Reputation `{local, 3}` is correct ✓ | the Martial permission and the grog exclusion in neither locale | **F-483**, **F-485**; ArMDE:2816 instance |
| `flaw.outsider_major` | 6550-6561 | creation_effect ✓ | **`score: 3` against ArMDE:6554's "level 1 to 3"; RULES.md:5384 records 1-3** | the range and the identifiability rule in neither locale | **F-486**; ArMDE:2816 and :2820-pair instances |
| `flaw.outsider_minor` | 6550-6561 | creation_effect ✓ | **`score: 1` against the same "level 1 to 3", which ArMDE:6556 gives the Minor version unchanged** | as above | **F-486**; ArMDE:2816 instance |
| `flaw.overconfident_major` | 6562-6565 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance; `NO_RULE_DESPITE_TOKEN` reading confirmed |
| `flaw.overconfident_minor` | 6562-6565 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.oversensitive_major` | 6566-6569 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.oversensitive_minor` | 6566-6569 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.pagan` | 6570-6573 | narrative → creation_effect | **the Minor magnitude the descriptor and ArMDE:5331 both offer does not exist, so no pagan grog is buildable**; **no `ability_authorization` for `ability.magic_lore` / `ability.faerie_lore` (both `arcane`)** | the permission and the Minor-for-grogs clause in neither locale | **F-487**, **F-488**; ArMDE:2820 instance |
| `flaw.painful_magic` | 6574-6577 | in_play_effect ✓ | `health_mod {casting_fatigue, -1}` matches both siblings' convention ✓ (surfaced-only, Part C, not counted) | **the cumulativity with real fatigue and the pain-level recovery rule in neither locale** | **F-493** |
| `flaw.palsied_hands` | 6578-6581 | in_play_effect ✓ | `combat_mod {-2, attack}` + `{-2, defense}` argued out and pinned (`RULES.md:5058`) ✓ | full passage both locales, extra botch die included, ASCII ✓ — **but the shipped DE name disagrees with `tugenden-fehler.md:708`** **?** | **F-496**; **Q-129** |
| `flaw.pessimistic` | 6582-6585 | narrative ✓ | OK | OK | **clean**; ArMDE:2820 instance |
| `flaw.pious_major` | 6586-6589 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.pious_minor` | 6586-6589 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.plagued_by_supernatural_entity` | 6590-6593 | narrative ✓ | OK — unparameterized, so the default `max_per_target` correctly forbids a second copy ✓ | OK | **clean**; ArMDE:2818 instance; **Q-128** |
| `flaw.poor` | 6594-**6596** | creation_effect ✓ | `later_life_xp_rate {10}` is ArMDE:2394 verbatim ✓; magus + mythic-companion `forbidden_traits` ✓; grog blocked by `max_major_flaws: 0` ✓ | OK ✓ — the rewritten summary carries all three clauses in both locales | **clean**; **Q-124** (range ends one line short) |
| `flaw.poor_characteristic` | 6598-**6600** | creation_effect ✓ | `characteristic_score_delta_param {-1}` + `max_per_target: 2` is ArMDE:6600 exactly ✓ (`RULES.md:1429`) | **both locales narrow "-3 or lower" to "at -3" and add "no points are gained for it", which the passage does not say and `compute_balance` contradicts** | **F-490**; **Q-124**; ArMDE:2814 instance |
| `flaw.poor_concentration` | 6602-6605 | uncomputed_rule ✓ | OK — no effect names which Ability a roll modifier applies to (A41 carries no ability field) | OK ✓ — the -3 in `description` in both locales, ASCII ✓ | **clean** |
| `flaw.poor_eyesight` | 6606-6609 | in_play_effect ✓ | `combat_mod {-3, attack}` + `{-3, defense}` ✓; the explicit compatibility with Missing Eye correctly leaves `incompatible_with` empty ✓ | **the -3 on sight rolls *outside* combat reaches the user in neither locale** | **F-491** |
| `flaw.poor_formulaic_magic` | 6610-6613 | in_play_effect ✓ | `casting_total_mod {-5, formulaic}` ✓ — `CastType::Ritual` does not match `formulaic`, so ArMDE:6612's Ritual carve-out is honoured | OK ✓ — the -5 reaches the user as the `casting_mod_formulaic` addend | **clean** |
| `flaw.poor_hearing` | 6614-6617 | **narrative → uncomputed_rule** | the -3 is inexpressible (D3): A41 names no Ability | OK ✓ — the -3 *is* the summary, in both locales, ASCII ✓ | **F-489** |
| `flaw.poor_living_conditions` | 6618-6621 | in_play_effect ✓ | `aging_mod {living_conditions, -1}` correct in sign, re-derived from both ends (ArMDE:4530's +1) ✓ | OK ✓ | **clean** |
| `flaw.poor_memory` | 6622-6625 | narrative ✓ | OK | OK | **clean**; ArMDE:2820 instance |
| `flaw.poor_student` | 6626-**6628** | in_play_effect ✓ | `advancement_mod {taught, -3}` + `{book, -3}` ✓, carve-out encoded by omission ✓ (surfaced-only, Part C, not counted) | **the "do not reduce a total below one" floor in neither locale** | **F-492**; **Q-124** |
| `flaw.primitive_equipment` | 6630-6633 | uncomputed_rule ✓ | OK — the *City & Guild* clauses point outside `rules/source/en/` and are correctly not implemented | OK ✓ — the whole passage in both locales, ASCII ✓ | **clean** |
| `flaw.primogeniture_lineage` | 6634-6637 | narrative ✓ | `prerequisites: all(is_magus, house.verditius)` ✓; `["story"]` + `index_categories: ["hermetic"]` matches the dual index listing ✓ | **the shipped DE name disagrees with `grundbegriffe.md:327`** **?** | **F-496**; **Q-129**; ArMDE:2818 instance |
| `flaw.prohibition` | 6638-6641 | narrative ✓ | OK | OK | **clean**; **Q-125** (ArMDE:6719 calls it *Personality*; descriptor + index say *Supernatural*, and the data follows those two) |
| `flaw.proud_major` | 6642-6645 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.proud_minor` | 6642-6645 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.raised_from_the_dead` | 6646-6649 | **uncomputed_rule → creation_effect** | **no `effects` at all, although `warping_grant {points: 3}` and `grants_reputation {local, 4}` both express clauses the passage states** | the whole passage in `description` in both locales, ASCII ✓ — but the shipped DE name disagrees with `tugenden-fehler.md:763` | **F-494**, **F-496**; ArMDE:2960 instance; not separately escalated under Q-129 because the entry already fails on data |
| `flaw.raised_in_the_gutter` | 6650-6653 | uncomputed_rule ✓ | OK — the three -3s are inexpressible (D3), same A41 reason as `poor_concentration` | OK ✓ — the whole passage in both locales, ASCII ✓ | **clean** |
| `flaw.realm_stigmatic` | 6654-6658 | uncomputed_rule ✓ | **a `realm` parameter with no `max_total`, so four copies naming four Realms are legal against ArMDE:6656's "Pick one"** | OK ✓ — both paragraphs in both locales; the DE name's word order is a recorded decision (`RULES.md:2372`) ✓ | **F-495**; ArMDE:2814 and :2960 instances |
| `flaw.rebellious_major` | 6659-6662 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.rebellious_minor` | 6659-6662 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |

## Findings

### F-481 — WITHDRAWN: `flaw.outcast`'s Wealthy prohibition is B09's F-340, row eight

I raised this as a new finding and it is not one.
**`docs/vf-audit/batch-09.md:1664` is F-340** — *"`virtue.wealthy` — eight
passages state a Wealthy/Poor exclusion and `incompatible_with` is empty
catalogue-wide"* — and its passage table at `:1668-1677` already lists
**`| :6540 | flaw.outcast | "You may not take the Wealthy Virtue." |`** as one of
the eight. It already derives the zero-carriers result, with a *better* argument
than mine (`validate_incompatibility_symmetry` fails the load on a one-sided
declaration, and the ruleset loads, so an empty set on `virtue.wealthy` is by
itself proof that nothing anywhere names it); it already records the bought-only
caveat; and it already prescribes the symmetric `incompatible_with` remedy.
B10's **F-376** is the same defect on `flaw.branded_criminal` (ArMDE:5751),
explicitly framed as one of F-340's eight.

The verification sub-agent found this; I did not. The audit has no
cross-batch finding index — `docs/vf-audit/corrections.md` is declared in
`README.md:93` as "the accumulated correction list" and **does not exist**
(**F-499**) — so nothing but reading sixteen prior batch files would have caught
it, and this batch's failure to do so is the strongest argument for that file
existing.

**The verdict row for `flaw.outcast` points at F-340, not at a B16 number.** What
survives from my pass is folded into **F-482**, which is restated below as an
*extension* of F-340 rather than as a finding of its own.

*(Original text retained below the rule, so the reasoning is auditable and the
duplication is visible rather than silently deleted.)*

---

> "You have the rough task of making it on your own—normal society rejects you
> and you are not attached to a covenant. Perhaps you have a magical nature, a
> supernatural background, some disfigurement, or a tremendous scandal in your
> past. **You may not take the Wealthy Virtue.**"
> — ArMDE:6540

> "Du hast die schwierige Aufgabe, allein klarzukommen – die normale Gesellschaft
> lehnt dich ab, und du bist nicht an einen Konvent gebunden. Vielleicht hast du
> eine magische Natur, einen übernatürlichen Hintergrund, eine Entstellung oder
> einen ungeheuerlichen Skandal in deiner Vergangenheit. **Du darfst nicht die
> Tugend Wohlhabend nehmen.**"
> — DE 6540

**Current data.** `flaw.outcast`: `classification: "narrative"`, no `effects`, no
`parameters`, no `prerequisites`, **no `incompatible_with`**. Its counterpart
`virtue.wealthy` (ArMDE:5235-5238) carries `later_life_xp_rate { amount: 20 }`
and **no `incompatible_with` either**. Both locales carry `name` + `summary`
only, and the summary is sentence one — which stops before the prohibition.

**Why it is wrong.** The sentence is a flat, checkable, engine-expressible rule
about legal character construction, and the engine has the exact field for it.
`ruleset/integrity.rs::validate_incompatibility_symmetry` requires both edges, so
the correction touches both entries. The scope caveat that bit B15's F-466 does
**not** apply here: `validate_incompatibilities` reads **bought** selections on
both sides, and `virtue.wealthy` is reachable through no grant — it appears in no
`granted_selections`, no `grants_selection` effect, no House grant and no mythic
type — so every copy a player can hold is bought and a declared
`incompatible_with` fires.

Today a companion may hold Outcast (Minor, Social Status) and Wealthy (Major,
General) together, take the 20 XP/year later-life rate the Virtue grants, and
validation says nothing.

**Correct value.** `incompatible_with: ["virtue.wealthy"]` on `flaw.outcast` and
`incompatible_with: ["flaw.outcast"]` on `virtue.wealthy`. The classification
stays `narrative` once the rule is *computed* — that is the position
`uncomputed_clauses.rs`'s `flaw.primogeniture_lineage` exemption writes down
("the rule is enforced rather than merely described, and describing it again
would not be `uncomputed_rule`") — but it is not obviously the right position
while the rule is computed **nowhere**, which is **Q-126**.

**Severity: MEDIUM.** A stated prohibition that validation never raises, on a
pair whose Virtue changes a real creation-time number (the later-life XP rate).
Not a miscalculation; a legality gap.

### F-482 — an extension of B09's F-340: the passage list is short by three, and the "would not catch a granted Redcap" caveat can be retired *(outside this span)*

**This is not a new defect.** B09's **F-340** is the finding; this adds three
passages it misses and closes the one question it left open. Both additions came
out of following `flaw.poor`'s and `flaw.outcast`'s pointers, and the second came
out of the verification pass challenging my own five-passage count.

**The complete list, derived mechanically rather than by reading.** One `rg` over
the English core book for the prohibition's spellings —
`cannot take the Wealthy` / `may not take the Wealthy` / `not take the Poor` /
`cannot take the Poor` / `prohibited from being Wealthy` /
`benefit from either the Poor` — returns **eleven** passages, and nothing else in
the book states the rule:

| ArMDE | Entry | Forbids | In F-340? |
|---|---|---|---|
| :3406 | `virtue.almogavar` | both | **no — new** |
| :3611 | `virtue.covenfolk` | both | yes |
| :3631 | `virtue.custos` | both | yes |
| :4043 | `virtue.guild_apprentice` | both | **no — new** |
| :4494 | `virtue.mendicant_friar` | both | yes |
| :4634 | `virtue.perfectus` | Wealthy only | yes |
| :4800 | `virtue.priest` | Poor only, **conditionally** ("You may be a parish priest. **If you are**, you cannot take the Poor Flaw.") | **no — new, and needs its own verdict** |
| :4850 | `virtue.redcap` | both | yes |
| :5181 | `virtue.turb_trained` | both | yes |
| :5751 | `flaw.branded_criminal` | Wealthy only | yes (B10's F-376) |
| :6540 | `flaw.outcast` | Wealthy only | yes |

So F-340's "eight passages" is **ten flat prohibitions plus one conditional**.
The two flat additions are plain "may not take" / "not able to benefit from"
sentences in exactly F-340's shape and belong in its list without further
argument. **ArMDE:4800 is a different thing** and is flagged rather than folded
in: the prohibition binds only a *parish* priest, a fictional sub-role
`virtue.priest` does not model, and the same passage relaxes it two lines later
(ArMDE:4802), so `incompatible_with` would over-exclude. That is Q-126's family
and is left to **B07**, which owns the entry.

**What is new and load-bearing: the bought-only caveat can be retired.** F-340
records, correctly for what it knew, that an `incompatible_with` pair "would
**not** catch a *granted* Redcap beside a bought Wealthy". I checked, and the
premise does not hold in the shipped data. Across the whole of `rules/core/`,
`virtue.wealthy` and `flaw.poor` appear **only** as their own catalogue entries
plus two `forbidden_traits` rows in `character_types.json` — no House grant, no
mythic-type grant, no childhood, no life stage, no `granted_selections`, and none
of the seven `grants_selection` carriers names either (they grant only
`ability.dowsing`, `flaw.noncombatant`, `virtue.well_traveled`,
`virtue.social_contacts`, `virtue.second_sight` ×2, and
`virtue.brother_knight` + `virtue.temporal_influence`). **And neither is any
partner**: `virtue.almogavar`, `virtue.covenfolk`, `virtue.custos`,
`virtue.guild_apprentice`, `virtue.mendicant_friar`, `virtue.redcap`,
`virtue.turb_trained`, `virtue.perfectus` and `flaw.branded_criminal` are all
bought-only too. So `validate_incompatibilities`, which reads bought selections
on both sides (`validation/prereq.rs::PrereqCtx::build`), catches **every** pair a
player can actually assemble, and the plain `incompatible_with` remedy is
complete. `Prereq::Nor` — the correction B15's F-466 needed — is **not** required
here. That is the one question F-340 left open, answered.

**Why it still matters.** `flaw.poor` *changes a number*: its
`later_life_xp_rate { amount: 10 }` replaces the base 15 XP/year
(`life_stages.json`), and `virtue.wealthy`'s 20 does the same upward. So a
Covenfolk, a Custos, a Guild Apprentice, a Mendicant Friar, a Redcap or an
Almogavar who takes either gets a later-life experience pool the book says he may
not have, and nothing raises an issue.

**And the control group is what keeps the prohibition list honest.** At least
twenty *other* passages mention the same pair and explicitly **permit** it —
ArMDE:3400, :3547, :3573, :3623, :3689, :3845, :3961, :4197, :4293, :4323, :4391,
:4504, :4508, plus :4600, :4604, :4618, :4622, :4634 (partially), :4802, :4858,
:4914, :5225 — most with the formula "The Wealthy Virtue and Poor Flaw affect you
normally". My first draft said twelve; the verification pass counted more. The
*lexical* distinction between the two groups ("may not take" versus "affect you
normally") survives the recount and is in fact strengthened by it, but **the
count does not**, and is stated here as "at least twenty" because I did not
enumerate the permissions exhaustively the way I did the prohibitions.

**Correct value.** Add ArMDE:3406 and ArMDE:4043 to F-340's list and rate
ArMDE:4800 separately in B07. The remedy is F-340's own, unchanged and now
provably sufficient: symmetric `incompatible_with` edges on each pair, since
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
one-sided declaration.

**Severity: unchanged from F-340 — HIGH as F-340 states it** (the app permits an
illegal character on ten Virtues and Flaws at once, two of which move a
creation-time XP figure). The *extension* itself is MEDIUM: two more sites on a
known hole, plus the retirement of a caveat that would otherwise have made the
fix look incomplete.

### F-483 — Outlaw and Outlaw Leader both permit Martial Abilities, and the app refuses to build either character

> "You have been outlawed, and must live by your wits outside society. **You may
> take Martial Abilities at character generation**, and have a Reputation at
> level 2 for whatever got you outlawed."
> — ArMDE:6544

> "You are well known as an outlaw in the local area, with a Reputation level of
> 3. You are actively sought by the local lord, sheriff, or other such official.
> **You may take Martial Abilities at character generation.** Grogs may not take
> this Flaw."
> — ArMDE:6548

> "Du wurdest geächtet und musst außerhalb der Gesellschaft mit deinem Verstand
> überleben. **Du darfst Kampffertigkeiten bei der Charaktererschaffung
> erwerben** und hast eine Reputation der Stufe 2 für das, was dir die Ächtung
> eingebracht hat."
> — DE 6544
>
> "… **Du darfst Kampffertigkeiten bei der Charaktererschaffung erwerben.**
> Grogs dürfen diesen Fehler nicht nehmen."
> — DE 6548

**Current data.** `flaw.outlaw` and `flaw.outlaw_leader` each carry exactly one
effect, a `grants_reputation`, and neither carries `ability_authorization` nor
`restricted_ability_xp`. Both locales carry `name` + `summary` only, and both
summaries are sentence one — so the permission reaches the user nowhere either.

**Why it is wrong.** `rules/core/abilities.json`'s `categories_requiring_virtue`
is `["academic","arcane","martial"]`.
`validation/authorization.rs::validate_ability_authorization` errors with
`ability_category_requires_virtue` on any held Ability in a gated category unless
its id or its category appears in the union of `AbilityAuthorization` and
`RestrictedAbilityXp` lists, with a whole-character exemption **only** for a
profile whose `is_magus` is true. Both Flaws are *Social Status*, and
`social_status` is on the `permitted_categories` list of **all four** profiles.

So: a companion takes Outlaw, buys Single Weapon 3 as the passage expressly
permits, and the engine tells him his character is illegal. `flaw.outlaw_leader`
is the same sentence with the same consequence, minus grogs — who are excluded by
ArMDE:6548 but not by any data (**F-485**).

This is F-409's mechanism, **instances 35 and 36** against the **2** entries that
encode it (`flaw.covenant_upbringing`, `virtue.student_of_realm`, both re-derived
here) plus 10 implicit carriers via `restricted_ability_xp` on a gated category.
`rules/core/abilities.json` carries **four** Abilities with `category: "martial"`,
so the gate has real targets. Stated no wider than checked: the four F-409
instances whose passages I have actually read — B14's F-445, B15's F-460, F-461,
F-476 and F-477 — are all `arcane`, so these two are the first **Martial** ones I
can point at; I did not re-read B10-B13's instances to claim it of the whole
series.

**Correct value.** On each entry,
`{ "type": "ability_authorization", "categories": ["martial"] }`. `A14`'s
`categories` field takes an `AbilityCategory` enum, so `"martial"` is exactly the
shape. The `classification` of both stays `creation_effect` ✓ — they already
compute a creation-time effect — and note C7: nothing will fail if the added
effect is forgotten. The permission must also be written into `description` in
both locales under D5, since the summary stops before it.

**The exact wording the phrase screen missed**, for the screen-fix list:

- EN: `You may take Martial Abilities at character generation`
- DE: `Du darfst Kampffertigkeiten bei der Charaktererschaffung erwerben`

The screen's third assertion never examines these two at all — it filters to
`classification == "narrative"` and both are `creation_effect`. That is a gap in
the *guard*, not just in the vocabulary: the permission idiom would be invisible
here even if `MECHANICAL_PHRASES` carried it.

**Severity: HIGH.** Two characters the book describes and the app refuses to
build. Per `CLAUDE.md` this is product-integrity, not cosmetic.

### F-484 — `flaw.outlaw`'s granted Reputation hardcodes an audience the passage does not name

> "… and have a **Reputation at level 2 for whatever got you outlawed**."
> — ArMDE:6544

> "… und hast eine **Reputation der Stufe 2 für das, was dir die Ächtung
> eingebracht hat**."
> — DE 6544

**Current data.**
`effects: [{ "type": "grants_reputation", "kind": "local", "score": 2 }]`.

**Why it is wrong.** The sentence names a *level* and a *subject* ("for whatever
got you outlawed") and no **audience**. `A25 GrantsReputation`'s `kind` is
`Option<ReputationType>`, and `None` is not "unknown" — it is a defined value
meaning **player-chosen**, which `validation/scores.rs::validate_reputations`
consumes as a wildcard slot that any `Reputation` row may fill. So the data has a
correct way to say "the book does not fix the audience" and does not use it.

**Its own neighbour is the control.** `flaw.outlaw_leader`, four lines later, says
"You are well known as an outlaw **in the local area**, with a Reputation level of
3" — the audience *is* named — and ships the identical
`{ "kind": "local", "score": 3 }`. Two passages, one naming an audience and one
not, encoded the same way: the data cannot be reading the source on this field.

This is **F-450's second instance**. B14 found it on `flaw.infamous`, which pins
`kind: "local"` where its passage names no audience while its twin `virtue.famous`
correctly omits it. Every `grants_reputation` in this span was checked against
whether the passage names a type; the other three do
(`flaw.outlaw_leader` "in the local area", `flaw.outsider_*` "among members of the
dominant social group of your area", `flaw.raised_from_the_dead` "in the area
where the miracle occurred" — the last two being the audience the book gives, not
one of the four `ReputationType` members, which is B10's F-380 territory).

**RULES.md is silent rather than corroborating, and the silence is worth
recording.** `crates/arm-rules/RULES.md`'s "Reputation grant" list names an
audience or a polarity for most carriers — `:5377` "academic score 2", `:5380`
"hermetic score 3", `:5373` "bad score 4", `:5390` `virtue.famous`
"**player-chosen** score 4" — but `:5382` records `flaw.outlaw` as simply
"reputation grant score 2" and `:5383` records `flaw.outlaw_leader` as
"reputation grant score 3", with no audience on **either**. So the traceability
map does not distinguish the two cases the passages distinguish, which is why the
data's uniform `local` went unnoticed. It is also why this is rated LOW: nothing
in the repo asserts `local` is wrong, only the passage does.

**Correct value.** Omit `kind`, leaving `{ "type": "grants_reputation",
"score": 2 }`. Note that the audience the book *implies* (the people who know you
were outlawed) is a locality, so `local` is a defensible reading — but the audit's
standard is what the passage says, and F-450 was raised on exactly this
distinction.

**Severity: LOW.** `validate_reputations` matches kinds and counts, and a
concrete `local` slot is strictly *narrower* than a wildcard, so the effect is
that a player who wants his outlaw Reputation recorded as anything but Local gets
`reputation_not_granted`. No number is wrong; a legal choice is refused.

### F-485 — "Grogs may not take this Flaw" is in no profile's `forbidden_traits`

> "… **Grogs may not take this Flaw.**"
> — ArMDE:6548

> "… **Grogs dürfen diesen Fehler nicht nehmen.**"
> — DE 6548

**Current data.** `character_types.json`'s grog profile carries
`"forbidden_traits": ["virtue.the_gift"]` — one id, and it is a Virtue.
`flaw.outlaw_leader` appears in no profile's `forbidden_traits`, and its own
`entity_kinds` is `["character"]` (which does not distinguish character *types*).
Its shipped summary in both locales is sentence one, so the exclusion reaches the
user nowhere.

**Why it is wrong.** This is the one hard type restriction in the span — "may
not", not "should" — and `forbidden_traits` is the field that exists for it, used
today for exactly this purpose on three profiles (`virtue.the_gift` on grog,
`flaw.poor` + `virtue.wealthy` on magus and mythic companion). Nothing else in
the data blocks it: `social_status` is on grog's `permitted_categories`,
`flaw.outlaw_leader` is Minor so `max_major_flaws: 0` does not reach it, and
grog's `max_minor_flaws` is 3 so there is room.

So a grog may take Outlaw Leader today, command three to six outlaws, and carry a
level-3 Reputation, against a flat prohibition — and the same grog then hits
**F-483**'s hard error the moment he buys the weapon skill the same passage
grants him.

**Correct value.** Add `"flaw.outlaw_leader"` to the grog profile's
`forbidden_traits`, and write the exclusion into `description` in both locales
(D5).

**Severity: MEDIUM.** A stated prohibition validation never raises. Rated below
F-483 because no number is wrong and the illegal character is still internally
consistent.

### F-486 — both Outsider versions ship a fixed Reputation level where the book gives one range to both

> "If you live in society, take the Major version. You are shunned and often
> persecuted because of this, and your life and freedom may occasionally be in
> peril. **You have a bad Reputation of level 1 to 3 (depending upon how easy it
> is to identify you)** among members of the dominant social group of your area."
> — ArMDE:6554
>
> "If you spend most of your life in a closed group, such as a city ghetto or
> covenant, where people like you are accepted, take the Minor version, as you
> are only shunned if you leave your home, and have somewhere relatively safe to
> run to if you find trouble. **You still have the bad Reputation** among members
> of the dominant social group, but do not meet them so often."
> — ArMDE:6556

> "… **Du hast eine schlechte Reputation der Stufe 1 bis 3 (je nachdem, wie leicht
> du zu identifizieren bist)** unter Mitgliedern der dominanten sozialen Gruppe
> deines Gebiets."
> — DE 6554
>
> "… **Du hast immer noch die schlechte Reputation** unter Mitgliedern der
> dominanten sozialen Gruppe, begegnest ihnen jedoch nicht so oft."
> — DE 6556

**Current data.** `flaw.outsider_major`:
`grants_reputation { kind: "local", score: 3 }`. `flaw.outsider_minor`:
`grants_reputation { kind: "local", score: 1 }`. Both locales carry `name` +
`summary` only and neither summary mentions a Reputation at all.

**Why it is wrong, in three independent ways.**

1. **The book varies the level by *identifiability*, not by magnitude.** The
   parenthesis is explicit: "depending upon how easy it is to identify you". What
   distinguishes the Major version from the Minor one is stated separately and is
   about *where you live* — in society versus in a closed group — and ArMDE:6556
   says the Minor version keeps the Reputation unchanged ("still have"). The 3/1
   split maps the wrong axis onto the wrong variable, and it produces the
   backwards result for the book's own example: an obviously-identifiable person
   living in a ghetto is Minor with level 1, where the sentence gives him 3.
2. **The repo's own traceability map records the range.**
   `crates/arm-rules/RULES.md:5384-5385` reads
   `- flaw.outsider_major (ArMDE:6550-6561) — reputation grant bad score 1-3` and
   `- flaw.outsider_minor (ArMDE:6550-6561) — reputation grant bad score 1-3`.
   So RULES.md describes data the catalogue does not contain, for **both** ids —
   the same in-repo contradiction B15 found at `RULES.md:5161`, and this time it
   names the correct value.
3. **A third source agrees with the book.**
   `rules/source/de/translation-tables/reputationen.md:114` gives
   `Outsider (Major) | Außenseiter (Groß) | Lokal/Regional | 1–3 (–) |
   Zugehörigkeit zu gemiedener Gruppe`.

**Correct value.** Not determinable from the data model alone, which is why this
finding carries **Q-127**: `A25`'s `score` is a single `u8` and cannot hold a
range. The three candidate readings are (a) drop to the floor, `score: 1` on both,
(b) take the ceiling, `score: 3` on both, or (c) record the range in
`description` and let `score` carry whichever the engine treats as a pre-fill.
What is *not* in doubt is that the two ids must carry the **same** value, because
the book gives them the same Reputation. Under any reading the range must also be
written into `description` in both locales (D5), since neither summary mentions
the Reputation.

**How the wrong number got in, from git.** *(The verification pass's evidence,
not mine.)* The two ids were authored in `e0e80f3`, the Phase-A bulk pass, with
**no effects at all**. The `grants_reputation` rows were added later in
**`1b7a5eb`** (M5 5a-wire), whose commit message claims *"Every kind/score/amount
is source-cited against the Core Rules"* and offers no justification for a 3/1
split. `RULES.md`'s "bad score **1-3**" rows for both ids were written **earlier
the same day**, in `e2e9202`. So the wiring commit contradicted, within hours, the
traceability line it should have been reading — which is the same failure mode as
B15's `RULES.md:5161`, and the reason this batch treats a RULES.md/data
disagreement as evidence rather than as noise.

**Severity: MEDIUM.** `A25`'s `score` is **not enforced** —
`validate_reputations` compares kinds and counts only — so the figure reaches the
user as a frontend pre-fill rather than as a validated number. That caps it below
a miscalculation. But the pre-fill is real and was traced:
`ui/src/lib/components/Reputations.svelte:39` passes the grant's `score` straight
into `store.addReputation`, so the number the player gets handed on the sheet is
the wrong one unless he notices and overrides it — on a Flaw that is one of the
book's most commonly taken Social Statuses.

### F-487 — the book offers Pagan in two magnitudes, the catalogue ships one, and the missing one is the grog's

> "**Pagan** — *Major or Minor, Personality*"
> — ArMDE:6570-6571
>
> "… You may begin with Magic Lore or Faerie Lore, depending on the specifics of
> your faith. (This is not a Flaw in areas of Mythic Europe with substantial
> pagan populations, but by 1220 the only such areas are in parts of the Novgorod
> Tribunal. **It may also be a Minor Flaw for grogs who live at a covenant with a
> substantial pagan population that is an accepted and open part of the covenant
> community.**)"
> — ArMDE:6572

> "**Heide** — *Groß oder Klein, Persönlichkeit*"
> — DE 6570-6571
>
> "… (Dies ist in Gebieten des Mythischen Europas mit erheblichen heidnischen
> Bevölkerungen kein Fehler, aber bis 1220 gibt es solche Gebiete nur in Teilen
> des Novgorod-Tribunals. **Es kann auch ein Kleiner Fehler für Grogs sein, die
> in einem Konvent mit einer erheblichen heidnischen Bevölkerung leben, die ein
> akzeptierter und offener Teil der Konventsgemeinschaft ist.**)"
> — DE 6572

**Current data.** One catalogue entry, `flaw.pagan`, with
`magnitude: "major"`. Every other "*Major or Minor*" entry in this span ships two
ids with a mutual `incompatible_with` (`overconfident`, `oversensitive`, `pious`,
`proud`, `rebellious`, and `outsider` under "*Minor or Major*"), which is what
`ruleset/integrity.rs::validate_magnitude_variant_exclusivity` exists to enforce.

**Why it is wrong.** Three independent sources say the Flaw has two magnitudes —
the descriptor (ArMDE:6571), the book's own "List of Flaws" index, which files
Pagan at ArMDE:5331 under `### Personality, Major or Minor` (heading at
ArMDE:5310), and ArMDE:6572's own sentence naming who the Minor version is for.
And the consequence is not abstract: every profile's `flaw_category_caps` carries
`{"category":"personality","max":0,"major_only":true,"hard":true}` for grog, so a
grog cannot hold **any** Major Personality Flaw. The only character ArMDE:6572
names as a taker of the Minor version — a grog at a covenant with a pagan
community — is therefore the one character who cannot take the Flaw at all.

**This is a known incompleteness, not an oversight.**
`crates/arm-rules/RULES.md:3923` records the entry as
`| flaw.pagan (Major or Minor; seeded Major) | Major, Personality |
ArMDE:6570-6573 |`, in the M4/Phase-5 mythic-companion seeding table. The
parenthesis names the gap and nothing closed it.

**And the root cause is an off-by-one in a systematic pass, demonstrable three
ways.** *(The verification pass's evidence, and it is better than my "known
incompleteness" framing.)*

1. The English Virtues and Flaws blocks contain **33** dual-magnitude descriptors
   (`*Major or Minor*` / `*Minor or Major*`). Mapped onto the catalogue, **32 of
   them ship as `_major`/`_minor` pairs. `flaw.pagan` (ArMDE:6570) is the only
   exception in the entire catalogue** — not merely in this span.
2. The commit that performed the split, `e0e80f3`, says so in its own message:
   *"Dual-magnitude 'Major or Minor' items (**32**) split into _minor/_major
   entries"*. Thirty-two, against thirty-three descriptors.
3. `git show 699d43b` shows why the thirty-third was missed: `flaw.pagan` had
   already been hand-authored in M4 Phase 5, as `magnitude: "major"`,
   *specifically* to seed Spirit Votary's required Major Personality Flaw — and
   `e0e80f3` preserved pre-existing entries "byte-for-byte". So the entry was
   structurally excluded from the pass that would otherwise have split it.

That turns a vague "seeded Major" into a named mechanism, and it means the fix is
closing a known-size gap of exactly one rather than re-reading a descriptor.

**Correct value.** A second entry, plus a mutual `incompatible_with` — the same
shape the other six pairs in this span already use. Two remedy shapes are
possible and the choice has a second edit site:

- **Rename** `flaw.pagan` → `flaw.pagan_major` and add `flaw.pagan_minor`. This
  matches every other pair's id convention, but
  `rules/core/mythic_companion_types.json`'s `mythic_type.spirit_votary` carries
  `required_flaws: [{ default: { ref: "flaw.pagan" }, … }]` (sourced to
  ArMDE:2756, "Pagan (Major, Personality)"), which must move with it, and any
  existing save naming `flaw.pagan` needs a migration.
- **Keep** `flaw.pagan` as the Major and add `flaw.pagan_minor` beside it. No
  save migration and no mythic-type edit, but the id set is then inconsistent
  with the six neighbouring pairs.

Either way the `#### Pagan` heading is shared, so both ids cite
`ArMDE:6570-6573`, and the new entry needs `ability_authorization` too
(**F-488**).

**There is a third edit site, and it is a load-time failure rather than a
nicety.** `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`
(`:2082-2105`) detects `_major`/`_minor` pairs by id suffix and **fails the load**
unless each member names the other in `incompatible_with`. So the rename route
obliges the mutual declaration as well as the `spirit_votary` ref update — and,
conversely, this is why `flaw.pagan` escapes the guard today: with no `_major`
suffix there is no pair to detect, so nothing anywhere notices the missing half.

**Severity: HIGH.** A character the book explicitly describes — a pagan grog —
cannot be built at all, and a Minor Flaw the book offers is absent from the
catalogue, which also silently mis-prices any concept that wanted it (3 points
where the book charges 1).

### F-488 — `flaw.pagan` is `narrative` on a permission the app hard-errors on

> "… **You may begin with Magic Lore or Faerie Lore**, depending on the specifics
> of your faith."
> — ArMDE:6572

> "… **Je nach den Besonderheiten deines Glaubens darfst du Magiekunde oder
> Feenkunde zu Beginn erwerben.**"
> — DE 6572

**Current data.** `classification: "narrative"`, no `effects`, no `parameters`,
no `prerequisites`. Both locales carry `name` + `summary` only, and the summary
is a paraphrase of sentence one — *"You do not follow the Church and have never
been baptized."* / *"Du folgst nicht der Kirche und wurdest nie getauft."* — so
the only thing that reaches the user is the religious colour, with the Flaw's one
compensating benefit removed.

**Why it is wrong.** `rules/core/abilities.json` gives **both** named Abilities
the category `arcane` — `ability.magic_lore` and `ability.faerie_lore` — and
`categories_requiring_virtue` is `["academic","arcane","martial"]`.
`validation/authorization.rs::validate_ability_authorization` errors with
`ability_category_requires_virtue` unless the id or the category is authorized,
exempting only a profile whose `is_magus` is true. This Flaw is *Personality*,
permitted on all four profiles, so a companion, a mythic companion — and, once
**F-487** is fixed, a grog — may all take it and none is exempt. A player who
takes Pagan and does exactly what its sentence permits is told his character is
illegal.

Another F-409 instance against **2** encoded (the ordinal is deliberately not
claimed — see the header note). And it is literally B15's F-460 in the same
words: `flaw.magical_fascination` at
ArMDE:6394 permits "a score of 1 (but no more) in **either Magic or Faerie
Lore**", 178 lines earlier in the same book, and is unencoded for the same reason.
Two entries, one idiom, one gate.

**Correct value.** `classification: "creation_effect"`, with
`{ "type": "ability_authorization", "abilities": ["ability.magic_lore",
"ability.faerie_lore"] }`. Naming both ids is simpler than modelling "or" and is
strictly more permissive than the sentence — the same trade-off B15 raised as
**Q-122** for `flaw.magical_fascination`, which this entry should be resolved
together with. Unlike that entry, ArMDE:6572 states **no cap**, so nothing else
is inexpressible here. The permission must also be written into `description` in
both locales (D5).

**The exact wording the phrase screen missed** is recorded above under *"The
exact wording the phrase screen missed"*, item 2.

**Severity: HIGH.** A character the book describes and the app refuses to build,
plus a lost rule.

### F-489 — `flaw.poor_hearing` is `narrative` on a stated -3

> "**Subtract 3 from rolls involving hearing.** Speech that is hard for others to
> understand because of language, dialect, or accent is almost impossible for you
> to follow. You often pretend to be listening to people when in fact you are
> not."
> — ArMDE:6616

> "**Ziehe 3 von Würfen ab, die das Hören beinhalten.** Sprache, die für andere
> aufgrund von Sprache, Dialekt oder Akzent schwer zu verstehen ist, ist für dich
> fast unmöglich zu folgen. Du tust oft so, als würdest du Leuten zuhören, obwohl
> du es in Wirklichkeit nicht tust."
> — DE 6616

**Current data.** `classification: "narrative"`, no `effects`. Both locales carry
`name` + `summary` only, and the summary **is** the first sentence — *"Subtract 3
from rolls involving hearing."* / *"Ziehe 3 von Würfen ab, die das Hören
beinhalten."*

**Why it is wrong.** `narrative` asserts the book states **nothing mechanical**
for this entry. The entry's first sentence is a flat numeric modifier on a class
of rolls. That is as mechanical as a clause gets, and it is the *whole* rule —
the other two sentences are colour. The classification is simply false about the
rulebook.

**Why the class is `uncomputed_rule` and not `in_play_effect`.** No `Effect`
variant can express it, and the reason is structural rather than a missing row.
`A41 AbilityRollMod` is the only roll-modifier variant, and it carries
`param: String` (a free-text *subject*) and `amount: i8` — **no field naming
which Ability it modifies at all**; `engine-semantics.md`'s own entry says so
("the effect declares no *which ability* field"), its single shipped carrier is
`virtue.academic_concentration_subject`, and it is surfaced-only besides.
`A1 AbilityBonus` moves a *score*, not a roll, and "rolls involving hearing" is
not one Ability anyway. Per **D3** an engine that structurally cannot express a
rule is grounds for `uncomputed_rule` with the rule written out, and never for
`narrative`.

**The catalogue already contains the right answer twice, in this batch.**
`flaw.poor_concentration` (ArMDE:6604, "-3 penalty to Concentration rolls") and
`flaw.raised_in_the_gutter` (ArMDE:6652, three -3s on Charm, Guile and Etiquette)
are both `uncomputed_rule` with the rule in `description` in both locales, for
the identical reason. `flaw.poor_hearing` is the odd one out among three
neighbours.

**Correct value.** `classification: "uncomputed_rule"`. **No text change is
needed**: the rule is already the summary in both locales, with an ASCII "3" and
no sign at all, and `uncomputed_clauses.rs` builds its displayed text as
`description` if present else `summary` — the same precedence B14 established for
`flaw.loose_magic` and B15 re-derived for `flaw.missing_ear`. Adding a
`source.anchor` would bring it in line with the batch's other four
`uncomputed_rule` entries.

**The exact wording the phrase screen missed** is recorded above, item 3: the
bare imperative "Subtract 3" / "Ziehe 3 … ab" carries no sign character, so
`has_signed_number` is blind to it, and `MECHANICAL_PHRASES` has no entry for the
idiom. The entry sits inside `SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` block, swept
2026-09-15 and re-swept 2026-09-19, green both times.

**Severity: MEDIUM.** A lost-rule/provenance defect, not a miscalculation — and
unusually mild for its class, because the D5 harm that normally travels with a
wrong `narrative` does not apply: the rule already reaches the user in both
locales. What is wrong is the catalogue's claim about the rulebook, and the fact
that the next reader of this entry is told the book says nothing mechanical.

### F-490 — `flaw.poor_characteristic`'s shipped summary narrows the rule and adds a claim the passage does not make

> "You have an exceedingly bad Characteristic — **lower one which is already —3
> or lower by one point**. Describe what it is about you that makes this obvious,
> such as a feeble stature, hideous visage, or slack-jawed stupidity. You may take
> this Flaw twice for a single Characteristic, lowering it to —5, and multiple
> times for different Characteristics."
> — ArMDE:6600

> "Du hast eine außerordentlich schlechte Eigenschaft – **senke eine, die bereits
> –3 oder niedriger ist, um einen Punkt**. …"
> — DE 6600

**Current data.** The shipped summaries are rewrites, not extracts:

- EN: *"Lower a Characteristic already at -3 by one point, to no lower than -5;
  **no points are gained for it**."*
- DE: *"Senkt eine Eigenschaft, die bereits bei -3 liegt, um einen Punkt, auf
  mindestens -5; **Punkte gibt es dafür nicht**."*

**Why it is wrong, in two ways.**

1. **"or lower" is dropped, in both locales** — and the interesting part is
   *which* of the two the engine follows. *(This paragraph is the verification
   pass's correction; my first draft had the direction backwards and said a
   bought -4 was legal.)* The book's precondition is "already **—3 or lower**";
   both summaries say "already **at** -3" / "bereits bei -3".
   `characteristics.json` is `{"base_min": -3, "base_max": 3}` and its costs table
   bottoms out at `{"score": -3, "cost": -6}`, so a **bought** -4 does not exist —
   `validation/scores.rs::validate_characteristic_is_legal_score` raises
   `characteristic_out_of_range` for any score off the table. And
   `validate_characteristic_delta_preconditions` reads `entity.characteristics`,
   i.e. the **bought** score, which a second copy of the Flaw never changes. So on
   a plain character both copies see -3, both pass, and "at -3" is a *truthful*
   description of what the validator enforces. It is the **book** that is wider,
   not the summary that is narrower than the engine.
   **The narrowing still costs a legal character, through the effective score.**
   `flaw.dwarf` carries `characteristic_score_delta { str, -1 }` and
   `{ sta, -1 }`. A Dwarf who buys Strength at **-2** has an *effective* -3 and is
   "already -3 or lower" by ArMDE:6600 — but the precondition sees base -2, which
   is above the floor, and raises `characteristic_min_base_too_high`. That is the
   one reachable case where the dropped "or lower" is the difference between a
   legal character and a rejected one, and it is a bought-versus-effective
   mismatch in the validator, not only a wording defect in the text.
   **The mirror entry has the identical narrowing**, so the text fix belongs on
   both: ArMDE:3989 gives Great (Characteristic) "any Characteristic that already
   has a score of **at least +3**", and `virtue.great_characteristic`'s summary
   says "already **at** +3" in both locales.
2. **"no points are gained for it" is not in the passage, and its natural reading
   is false.** ArMDE:6598-6600 says nothing about points of any kind.
   `validation/balance.rs::compute_balance` iterates bought selections and adds
   `item.magnitude.points()` for every Flaw with **no exemption of any kind**, so
   a bought `flaw.poor_characteristic` yields its Minor Flaw point exactly like
   any other. In a Virtue/Flaw tooltip, "points" reads as Virtue/Flaw points, and
   under that reading the sentence is simply wrong.

**What the sentence was trying to say, and why that does not rescue it.** Its
sibling `virtue.great_characteristic` ships *"Raise a Characteristic already at
+3 by one **free** point"* / *"… **kostenlos** um einen Punkt an"*, and
`crates/arm-rules/RULES.md:1429` titles the Poor section *"a **free** −1, to a
minimum of −5"*. So the intended meaning is "no **Characteristic** points are
refunded" — the mirror of Great's "free". The Great summary says that
unambiguously with one adjective; the Poor summary says it with a noun phrase
that names the wrong currency. The two are meant to be sign-mirrors and are not.

**Correct value.** Both summaries restored to the passage: the precondition
"already at -3 **or lower**" / "bereits bei -3 **oder niedriger**", and the points
clause either deleted or replaced with Great's construction ("by one free point"
/ "kostenlos um einen Punkt"). The *data* is untouched — `max_per_target: 2`,
`characteristic_score_delta_param { amount: -1 }` and the `-3` buy floor in
`characteristics.json` are all correct and argued in full at `RULES.md:1429-1450`.

**Severity: MEDIUM.** Shipped user-facing rules text that states a rule the
source does not, in two locales, on an entry whose engine behaviour is correct —
so the defect is entirely in what the player is told. Under `CLAUDE.md`'s threat
model that is a wrong-rules-output defect of the text layer rather than of the
computation.

### F-491 — `flaw.poor_eyesight` computes the combat third of its -3 and drops the rest in both locales

> "Bleary vision impedes your performance. **Rolls involving sight, including
> rolls to attack and defend, are at –3.** New environments are disorienting and
> perhaps frightening for you. This Flaw can be combined with Missing Eye, but
> the penalties are cumulative."
> — ArMDE:6608

> "Verschwommenes Sehen beeinträchtigt deine Leistung. **Würfe, die Sehen
> beinhalten, einschließlich Angriffs- und Verteidigungswürfe, haben –3.** Neue
> Umgebungen sind für dich desorientierend und vielleicht beängstigend. Dieser
> Fehler kann mit Fehlendes Auge kombiniert werden, aber die Malus sind
> kumulativ."
> — DE 6608

**Current data.** `classification: "in_play_effect"`,
`effects: [{ combat_mod, -3, attack }, { combat_mod, -3, defense }]`, no
`description` in either locale, and both summaries are sentence one —
*"Bleary vision impedes your performance."* / *"Verschwommenes Sehen
beeinträchtigt deine Leistung."*

**The computed part is right and was re-derived at the consumer.** The sentence's
grammar is "**rolls involving sight**, *including* rolls to attack and defend" —
so the -3 on Attack and Defense is a named subset, and the two `combat_mod` rows
encode it exactly. `derived/combat.rs::combat_totals`' `cm` closure adds the
unscoped sum to every equipped weapon's Attack and Defense lines, which is what
"including rolls to attack **and defend**" asks for, and the reading is argued in
full at `RULES.md:5005-5019` and pinned by
`data_integrity.rs::the_combat_roll_flaws_penalize_the_combat_ability_totals` ✓.
The explicit compatibility with Missing Eye is satisfied by `A35`'s plain sum, so
the absent `incompatible_with` on both entries is correct ✓.

**Why it fails anyway (D5).** The *superset* — every other roll involving sight —
is the larger half of the rule, and no `Effect` variant can express it: `A41
AbilityRollMod` names no Ability (see F-489), and the Abilities a sight roll might
use (Awareness, Bows, Hunt, Craft, …) are not enumerable from the passage anyway.
That inexpressibility is grounds for the rule being **written out**, not for it
vanishing. Today it is in neither `description` nor `summary` in either locale, so
a player is shown a -3 on two combat lines and told nothing about the rule those
two lines are an instance of.

**Correct value.** `description` in both locales carrying the whole passage. The
classification stays `in_play_effect` ✓ — the combat half genuinely is computed
in play — which is exactly the case D5 exists for, and the catalogue already has
the pattern one entry away in `flaw.palsied_hands`.

**Severity: MEDIUM.** A lost rule on a very commonly taken Minor Flaw, with the
aggravating detail that the part which *does* reach the user is the narrow one,
so the display is not merely incomplete but misleading about the Flaw's scope.

### F-492 — `flaw.poor_student`'s floor of one reaches the user nowhere

> "You are bad at learning new things. Subtract 3 from all Advancement Totals
> derived from teaching and books (that is, you have no penalty to adventure
> experience, exposure, practice, studying from vis, or training), but **do not
> reduce a total below one**. If you could learn something without this Flaw, you
> still learn a bit."
> — ArMDE:6628

> "Du bist schlecht darin, neue Dinge zu lernen. Ziehe 3 von allen
> Fortschrittssummen ab, die aus Unterricht und Büchern stammen (das heißt, du
> hast keinen Malus auf Abenteuererfahrung, Exposition, Übung, Studium aus Vis
> oder Ausbildung), **aber reduziere eine Summe nicht unter eins**. Wenn du etwas
> ohne diesen Fehler lernen könntest, lernst du trotzdem ein wenig."
> — DE 6628

**Current data.** `classification: "in_play_effect"`,
`effects: [{ advancement_mod, source: "taught", -3 }, { advancement_mod,
source: "book", -3 }]`, no `description` in either locale, and both summaries are
rewrites that name no number — *"You are bad at learning from books and
teachers."* / *"Du lernst schlecht aus Büchern und von Lehrern."*

**The encoded part is right.** `AdvancementSource` has nine members and the two
chosen are exactly the two the sentence penalizes; the five-way carve-out
("adventure experience, exposure, practice, studying from vis, or training") is
encoded **by omission**, which is the only way `A39` can express it, since the
effect names what it modifies rather than what it spares ✓. `A39` is surfaced-only
(Part C C1) — the app does not simulate advancement — so the two rows reach the
user as labelled read-out rows and move no number, which is the engine's recorded
gap and is not counted here.

**Why it fails anyway (D5).** "Do not reduce a total below one" is a **clamp**,
and clamps are where this catalogue has been wrong before (`7f5605a`'s invented
aging floor). It is not computed — there is no advancement total to clamp — and it
is in neither locale, in neither field. The final sentence that explains it ("If
you could learn something without this Flaw, you still learn a bit") is gone too,
so the player is left with a -3 on two read-out rows and no way to know the book
stops it at 1.

**Correct value.** `description` in both locales carrying the whole passage, the
floor and the carve-out list included. The classification stays
`in_play_effect` ✓.

**Severity: MEDIUM.** A lost rule, and specifically a lost *floor* — the kind of
clause whose absence lets a later implementer compute a number the book forbids.

### F-493 — `flaw.painful_magic` drops the two clauses that say what a "pain level" is

> "Casting spells causes you to suffer the equivalent of one Fatigue level in
> pain for each spell you cast. **This reduces all your actions by the
> appropriate Fatigue penalty, which is cumulative with any from actual fatigue
> or injuries** (though you do not suffer any physical damage from pain). **You
> recover these "pain levels" just like Fatigue levels.**"
> — ArMDE:6576

> "Das Wirken von Zaubern verursacht dir das Äquivalent einer Erschöpfungsstufe
> an Schmerz für jeden Zauber, den du wirkst. **Dies verringert alle deine
> Aktionen um den entsprechenden Erschöpfungsmalus, der sich mit etwaigen Wunden
> oder tatsächlicher Erschöpfung kumuliert** (obwohl du durch Schmerz keinen
> körperlichen Schaden erleidest). **Du erholst dich von diesen „Schmerzstufen"
> genauso wie von Erschöpfungsstufen.**"
> — DE 6576

**Current data.** `classification: "in_play_effect"`,
`effects: [{ health_mod, track: "casting_fatigue", amount: -1 }]`, no
`description` in either locale, and both summaries are sentence one.

**The effect is right in sign and size, re-derived from its siblings rather than
assumed.** `casting_fatigue` has exactly **three** shipped carriers
catalogue-wide: `flaw.painful_magic` **-1**, `flaw.vulnerable_casting` **-1**
(ArMDE:6993-7004) and `virtue.withstand_casting` **+1** (ArMDE:5261-5282). A36's
stated convention is "positive reduces the penalty magnitude", so a Flaw carrying
a negative matches both its siblings and the convention ✓. The track is one of
A36's three **surfaced-only** members — listed labelled, folded into no number —
which is Part C C1, the engine's recorded gap, and is not counted against the
entry. B12's **Q-98** (which way `HealthTrack::CastingFatigue`'s sign runs) stays
open and this batch adds no evidence either way, because ArMDE:6576 states a
*loss* rather than a modifier.

**Why it fails anyway (D5).** Two rules the effect never claimed to carry:

- **Cumulativity.** Pain levels stack with "actual fatigue or injuries" on the
  same penalty ladder. No `HealthTrack` member expresses a parallel track that
  adds into the Fatigue and Wound penalties; the two *computed* tracks
  (`fatigue_penalty`, `wound_penalty`) move a tier's printed penalty, and each is
  clamped at 0 (`(base + delta).min(0)`).
- **Recovery.** Pain levels come back "just like Fatigue levels" — a recovery rule
  for a resource the engine does not hold.

Both are inexpressible (D3) and in neither locale, in neither field.

**Correct value.** `description` in both locales carrying the whole passage. The
classification stays `in_play_effect` ✓.

**Severity: LOW-MEDIUM.** A lost rule on a Major Hermetic Flaw, mitigated by the
fact that the headline mechanic — one Fatigue level per spell — *is* in the
summary in both locales, so the player is not left with nothing. What he is left
without is the part that says the penalty compounds with real fatigue, which is
the clause that makes the Flaw Major.

### F-494 — `flaw.raised_from_the_dead` is `uncomputed_rule` and carries no effects, although the engine can express two of its clauses

> "You died, and were brought back to life through a holy miracle. **You begin
> with at least three Warping points**, plus one Warping point for every year
> that has passed since you were resurrected, and you automatically receive
> another Warping point every year you continue living. **You also have a level 4
> reputation in the area where the miracle occurred.** You do not remember what
> happened to you while you were dead, although you may have virtuous impulses
> that you cannot explain."
> — ArMDE:6648

> "Du bist gestorben und durch ein heiliges Wunder ins Leben zurückgebracht
> worden. **Du beginnst mit mindestens drei Verzerrungspunkten**, zuzüglich eines
> Verzerrungspunkts für jedes Jahr, das seit deiner Auferstehung vergangen ist,
> und du erhältst automatisch jeden weiteren Jahr, den du weiterlebst, einen
> weiteren Verzerrungspunkt. **Du hast außerdem eine Reputation der Stufe 4 in
> dem Gebiet, in dem das Wunder stattfand.**"
> — DE 6648

**Current data.** `classification: "uncomputed_rule"`, **no `effects` key at
all**, no `parameters`, no `prerequisites`. Both locales carry the whole passage
in `description` ✓ — the text half is exemplary and is why this entry's **text**
column is green.

**Why the classification and the data are wrong.** `uncomputed_rule` asserts the
engine does **not** compute the rule. Two of the passage's four clauses are
expressible with variants that already exist and already ship:

1. **"at least three Warping points"** → `A22 WarpingGrant`, whose `points` field
   is read by `effective/warping.rs::warping_grant_points_in` and summed into
   `warping_points_total`. The catalogue's one existing carrier is
   `flaw.warped_by_magic`, so the shape is not an audit invention. Note A22's
   `score` field is **inert by design** (destructured as `score: _`) — the Warping
   Score is derived from points through the advancement curve — so the correct
   encoding is `{ "score": 0, "points": 3 }` or whatever the serde default
   permits, never a `score: 4` echoing the Reputation.
2. **"a level 4 reputation in the area where the miracle occurred"** →
   `A25 GrantsReputation`, `{ "kind": "local", "score": 4 }`. "In the area where
   the miracle occurred" names a locality, so unlike **F-484** the `kind` is
   fixed by the passage.

Only the *accrual* clauses are genuinely inexpressible — "one Warping point for
every year that has passed since you were resurrected" and "another Warping point
every year you continue living" need a resurrection date the app does not hold —
and those correctly stay text.

**The remedy is safe, and the figure was computed rather than assumed.**
`effective/warping.rs::WarpingOwed::from_score` makes a non-magus owe a Minor Flaw
at Warping Score ≥ 1 and a second at ≥ 3, so the question is what three points
score. `warping_score = advancement.score_for_xp(points_total)`, and the Ability
advancement curve's first step in `abilities.json` is **5 XP → score 1**.
So **3 points → Warping Score 0 → `WarpingOwed::from_score(0)` = `{0, 0, 0}`**:
nothing owed, no budget change. Two consequences to write into the correction
rather than discover afterwards:

- **The entry becomes ineligible as a warping-owed *fill*.**
  `effective/warping.rs::item_carries_warping_grant` is the deliberate recursion
  guard, and `warping_granted_selections` drops any chosen fill that itself
  carries a `WarpingGrant`. So adding the effect changes this entry's status in
  that list — arguably to the right one, since a Flaw that *causes* Warping should
  not also pay for it, but it is a behaviour change and not a no-op.
- **A player who enters stored Warping Points does cross the threshold.** The
  passage says "**at least** three", and at a points total of 5 the score reaches
  1 and a Minor Flaw becomes owed. That is the book's own rule firing, not a
  regression.

**And a trap sitting directly in this remedy's path — see F-500.** `A22`'s
`score` field is read by nothing (`warping_grant_points_in` destructures it as
`score: _`) and no load-time check ties it to `points`, so a `score` authored here
would be silently inert and could contradict the computed value indefinitely.

**Correct value.** `classification: "creation_effect"` with both effects added.
The `description` stays as it is — it is already complete and correct in both
locales, and under D5 the accrual clauses still need it.

**Severity: MEDIUM-HIGH.** Two creation-time numbers the book states and the app
computes nowhere, on an entry whose class asserts that is deliberate. It is the
`uncomputed_rule` → `creation_effect` transition the README records as never
having been looked for, which makes it worth more than its immediate impact.

### F-495 — `flaw.realm_stigmatic` is parameterized with no `max_total`, so four copies are legal

> "The character occasionally manifests physical phenomena in the presence of
> supernatural powers. **Pick one of the four Realms of Power**; whenever he
> enters an aura of strength 4 or more aligned to that realm, or is affected by a
> power greater than fourth magnitude (20th level) from that realm, he suffers
> from the stigmata."
> — ArMDE:6656

> "Der Charakter manifestiert gelegentlich in Gegenwart übernatürlicher Kräfte
> körperliche Phänomene. **Wähle eine der vier Sphären der Macht**; …"
> — DE 6656

**Current data.** `classification: "uncomputed_rule"`,
`parameters: [{ key: "realm", type: "ref", domain: "realm" }]`, **no
`max_total`**, no `max_per_target`, no `max_per_value`. Full passage in
`description` in both locales ✓.

**Why it is wrong.** `validation/selections.rs::validate_duplicate_selections`
keys `max_per_target` (default **1**) on the pair `(item_ref, the whole params
map)`, so two selections of this Flaw naming different Realms are two distinct
tuples and both are legal. A character may therefore hold Magic Stigmatic, Faerie
Stigmatic, Infernal Stigmatic **and** Divine Stigmatic simultaneously — four
copies of a *Minor, Supernatural* Flaw worth 4 points — against a passage whose
instruction is "Pick **one** of the four Realms" and against ArMDE:2814 ("A Virtue
or Flaw may be taken more than once **only if the description explicitly allows
it**"), which forbids the repeat by silence.

This is B15's **F-469** shape (`flaw.magical_being_companion`), B14's **F-451**,
B11's **F-400** and B10's **F-358** — a parameterized entry whose book grants no
repeat.

**And the remedy is not the one the neighbouring entry uses.**
`flaw.necessary_realm_aura_for_ability` answers the same *kind* of question with
`max_per_value: 1`, because ArMDE:6482 explicitly permits one copy "for any
particular Ability" — different Abilities are legal, the same one twice is not.
Here the book permits **no** second copy of any Realm, so `max_per_value` would be
exactly wrong (it would leave all four Realms open, which is the bug). The
correct field is **`max_total: 1`**, keyed on `item_ref` alone — the same value
`flaw.offensive_to_beings` carries for the same sentence shape.

**Correct value.** `"max_total": 1`. Nothing else about the entry changes: the
`realm` parameter is right, the aura-4 and fourth-magnitude thresholds are
correctly left uncomputed (no effect expresses an aura-conditional trigger), and
the passage ships in full in both locales ✓.

**Severity: MEDIUM.** A budget hole — four points of Flaws where the book allows
one — reachable by any player who simply picks the Flaw again. Not a wrong
number, but a legality gap that directly funds Virtues.

### F-496 — three shipped German names disagree with the translation tables *(D6 rule 2 — determination is the orchestrator's)*

| English | table row | shipped `rules/i18n/de/virtues_flaws.json` | DE rulebook heading |
|---|---|---|---|
| Primogeniture Lineage | `rules/source/de/translation-tables/grundbegriffe.md:327` — *Primogenitur-Abstammungslinie* | **Erstgeburts-Abstammung** (`:1003`) | **Erstgeburts-Abstammung** (DE 6634) |
| Palsied Hands | `rules/source/de/translation-tables/tugenden-fehler.md:708` — *Zittrige Hände* | **Zitternde Hände** (`:940`) | **Zitternde Hände** (DE 6578) |
| Raised from the Dead | `rules/source/de/translation-tables/tugenden-fehler.md:763` — *Von den Toten Auferweckt* | **Vom Tode auferstanden** (`:1019`) | **Vom Tode auferstanden** (DE 6646) |

**The dispute.** In all three the shipped name follows the **DE rulebook heading**
and the table says something else. Per **D6 rule 2** — a *terminology* claim, so
the glossary wins, in the order thematic table > `tugenden-fehler.md`, and "the DE
rulebook heading is strong evidence of terminology but is not decisive, because
the heading is one translator's rendering at one line and the glossary is the
catalogue-wide reconciliation" — the table's value is the one that should ship in
all three cases, and the data is wrong.

**But D6's closing paragraph is why this is reported rather than decided.** The
tables are *copied* into `arm-char-gen`, not referenced, and the `Covenfolk` case
proved the copy can be **stale** while the source project `arm-de-translation`
already carries the right value with a written rationale. That project is outside
this repository and the read is denied by argument-path containment, so I cannot
tell a real table error from a stale copy. Reported as **"this repository's copy
disagrees with the rulebook"**, per the brief.

**Two things that are settled and narrow the question.** `crates/arm-rules/RULES.md`
carries **no** record justifying any of the three shipped names — unlike
`flaw.realm_stigmatic`, whose word-order divergence from its DE heading *is*
recorded and argued at `RULES.md:2356-2391` and was withdrawn on that basis. And
none of the three is a name **collision** with another catalogue entry, so D6's
`flaw.fettered_magic` / `virtue.tethered_magic` reasoning (a heading colliding
with another entry's is evidence the heading is wrong) does not apply in either
direction.

**Two of the three rows also carry a provenance claim the core book contradicts** —
`tugenden-fehler.md:708` attributes Palsied Hands to *HoH:TL* and `:763`
attributes Raised from the Dead to *SdM:G* (RoP:D), while ArMDE:6578 and
ArMDE:6646 carry both — and `grundbegriffe.md:327` calls Primogeniture Lineage
"Kleiner **Hermetischer** Fehler", dropping the Story half of ArMDE:6635's
"*Minor, Story and Hermetic*". Those are D6 **rule 1** errors (a terminology table
making a factual claim about the rules) and are B15's **Q-121** shape — escalated
there, not counted separately here.

**Severity: LOW-MEDIUM.** A user-facing German label is either right or wrong and
every German-speaking user sees it every session, which `CLAUDE.md` rates up. But
none of the three is *unintelligible* or colliding, and the determination is not
mine to make — which is why `flaw.palsied_hands` and `flaw.primogeniture_lineage`
are recorded as **escalated, not checked** rather than as failures (**Q-129**).

### F-497 — `reputationen.md:112-113` states Outlaw's magnitude and Reputation level wrongly, in both rows *(D6 rule 1)*

> `| Outlaw (Minor) | Geächteter (Klein) | Lokal | 2 (–) | Ächtergrund (frei wählbar) |`
> `| Outlaw (Major) | Geächteter (Groß) | Lokal | 3 (–) | Bekannter Geächteter |`
> — `rules/source/de/translation-tables/reputationen.md:112-113`

**What the rulebook says.** There is no Minor Outlaw and no Major Outlaw with a
level-3 Reputation. ArMDE:6542-6544 gives **Outlaw** as *Major, Social Status* with
"a Reputation at level **2**", and ArMDE:6546-6548 gives **Outlaw Leader** as
*Minor, Social Status* with "a Reputation level of **3**". The book's own index
confirms both filings: Outlaw at ArMDE:5384 under `### Social Status, Major`
(heading ArMDE:5382) and Outlaw Leader at ArMDE:5529 under
`### Social Status, Minor` (heading ArMDE:5520), and ArMDE:2899-2900's "All
Cultures" table lists Outlaw under **Major Flaws** and Outlaw Leader under
**Minor Flaws**.

So the two rows have collapsed two different catalogue entries into one headword
and assigned each the other's magnitude: the level-2 Reputation belongs to the
**Major** Outlaw and the level-3 one to the **Minor** Outlaw *Leader*.

**Two narrowings the verification pass added, both of which make the row worse
rather than better.** First, the table does not merely swap two magnitudes — it
has **no row for Outlaw Leader at all**, and files that entry's number under a
*fictitious* "Outlaw (Minor)". My Method table lists Outlaw Leader among the
eleven table-less names, which is right in effect but understates what the table
is doing. Second, `reputationen.md:114` is **Outsider (Major) only**; there is no
Minor row, so my Method's "*Außenseiter (Groß/Klein)*" overstates its coverage. It
corroborates the 1–3 range for the Major version and is silent on the Minor —
still enough for **F-486**, since ArMDE:6556 gives the Minor version the same
Reputation.

**Why this is the table's error and not the data's.** D6 rule 1: "A factual claim
about the rules — which Virtue grants which Reputation, at which level, in which
magnitude class … **The rulebook wins, always.** A terminology table making a
factual claim is the error shape this audit has now found seven times." Magnitude
and Reputation level are exactly the listed examples. The shipped data has both
right — `flaw.outlaw` `{ score: 2 }`, `flaw.outlaw_leader` `{ score: 3 }` — so
nothing in `rules/` needs to change; the table does.

The German *name* column is not in dispute: `Geächteter` matches DE 6542's
heading and the shipped `flaw.outlaw` name ✓, and DE 6546's `Anführer von
Geächteten` (which the table omits entirely) matches the shipped
`flaw.outlaw_leader` name ✓.

**Correct value.** Two rows: `Outlaw (Major) | Geächteter | Lokal | 2` and
`Outlaw Leader (Minor) | Anführer von Geächteten | Lokal | 3`. Per **D6** the
correction is made **here and in the source project `arm-de-translation`** in the
same step, because a wrong table is a generator of future wrong data — but the
same "check whether the copy is stale first" caveat applies as in F-496.

**Severity: LOW for the shipped app, MEDIUM for the repository.** No user-facing
string is wrong today. What is wrong is a canonical reference that the next agent
generating German rules text will read, and it happens to contradict the book on
the very two numbers **F-486** shows the catalogue is already capable of getting
wrong.

### F-498 — `flaw.servant_of_the_land` grants a budget-exempt Prohibition and encodes neither half *(outside this span)*

> "Some powerful magical creature has saved the character from death or granted a
> similar boon, and in return the character has been set a task. … Usually the
> task is either difficult or time-consuming, and while it remains incomplete
> **the character has the Minor Personality Flaw: Prohibition, but this does not
> count toward the character's total number of Virtues and Flaws.**"
> — ArMDE:6719

> "… solange sie unvollendet bleibt, **hat der Charakter den Kleinen
> Persönlichkeits-Fehler: Prohibition, der jedoch nicht auf die Gesamtzahl der
> Tugenden und Fehler des Charakters angerechnet wird.**"
> — DE 6719

**Current data.** `flaw.servant_of_the_land` (ArMDE:6717-6720, **B17**'s span):
`classification: "uncomputed_rule"`, a `land` free-text parameter, and **no
`effects` key at all**.

**Why it is wrong.** This is `virtue.ineslemen`'s encoding, which B15 rated as the
catalogue's one fully-correct instance of the mechanism, stated in the same two
clauses:

- **The grant.** `A18 GrantsSelection` takes an `items` list, and
  `{ "type": "grants_selection", "items": ["flaw.prohibition"] }` is exactly what
  the sentence says.
- **The budget exemption.** It needs no encoding at all, and that is the point:
  `validation/balance.rs::compute_balance` iterates `entity.selections` — the
  **bought** list — while a granted item lands outside it, so a granted Prohibition
  automatically "does not count toward the character's total". `validate_caps`
  reads the same bought list. So the second clause is satisfied *by* using the
  first, and encoding the grant is the whole fix.

The clause reaches the user today only because it sits inside the shipped
`description` (the entry is `uncomputed_rule` and carries its full passage) — so
this is a data finding, not a D5 one.

**A book self-contradiction that the remedy must not propagate.** ArMDE:6719 calls
it "the Minor **Personality** Flaw: Prohibition", while the entry's own descriptor
at ArMDE:6639 says "*Minor, **Supernatural***" and the book's index files it at
ArMDE:5552 under `### Supernatural, Minor` (heading ArMDE:5534). Two sources
against one, so the shipped `categories: ["supernatural"]` is right ✓ and the
grant should name `flaw.prohibition` as it stands. Escalated as **Q-125**.

**And a German-source naming inconsistency, recorded so the correction pass sees
it.** DE 6719 leaves the Flaw's name **untranslated** — "den Kleinen
Persönlichkeits-Fehler: **Prohibition**" — where DE 6638's own heading for the
entry is "**Verbot**", which is also the shipped German name. Same shape as B15's
**F-473**, but in the *source* file rather than in shipped text, so it costs
nothing today and would cost something the moment that sentence is quoted into
`description`.

**Correct value.** Add
`"effects": [{ "type": "grants_selection", "items": ["flaw.prohibition"] }]` to
`flaw.servant_of_the_land`, and reclassify it — it would then compute a
creation-time effect, so `creation_effect`, with the passage's other clauses (the
one-season-per-year obligation and the curse) still owed as `description` under
D5, where they already are ✓. Note C7: nothing will fail if the effect is
forgotten when the class changes. The entry is **B17**'s to verify in full; this
finding records only what the inbound pointer from `flaw.prohibition` exposed.

**Severity: MEDIUM.** A grant the book states and the app does not make, on an
entry outside this span. Reached only because `flaw.prohibition` was swept for
inbound mentions.

### F-499 — `docs/vf-audit/corrections.md` is declared in the README and does not exist *(the audit's own record)*

`docs/vf-audit/README.md:93` lists it in the Files table:

> | `corrections.md` | The accumulated correction list — everything the batches
> found, which is the input to the fixing pass. |

There is no such file. Sixteen batches have now produced roughly five hundred
findings and there is **no accumulated list anywhere** — each batch file is the
only record of its own findings, and nothing indexes them across batches.

**This is not a tidiness complaint; it is the thing that let this batch duplicate
work.** **F-481** re-raised B09's F-340 on the identical passage, and half of
F-482's passage list re-raised three more of F-340's eight. The only way to have
caught that before the verification pass was to read sixteen prior batch files
looking for a Wealthy/Poor finding, which is exactly the job the declared file
exists to do. B10's F-376 avoided the same trap only because B10 happened to be
the batch immediately after B09.

**Correct value.** Either create `corrections.md` and back-fill it from B01-B16,
or delete the row from `README.md:93` so the Files table stops describing an
artefact the process does not produce. The first is obviously right while
thirty-nine batch-equivalents of findings still have to be applied; the second at
least stops the README lying.

**Severity: MEDIUM, and it compounds.** It costs duplicated audit effort now and
will cost a correction pass that has to reconstruct the work list from nineteen
prose documents. The cost is already realised once, in this batch.

*(Found by the verification sub-agent while checking F-481's novelty.)*

### F-500 — `WarpingGrant.score` is dead data with no integrity check, and F-494 would be its second carrier *(B18's span)*

`effective/warping.rs::warping_grant_points_in` destructures the field as
**`score: _`** and nothing anywhere reads it; `engine-semantics.md:894` already
records it as "**Never read**". The Warping Score is derived from `points` through
the Ability advancement curve (`score_for_xp`), so the stored `score` can say
anything at all.

**The one shipped carrier agrees with the computed value by coincidence, not by
construction.** `flaw.warped_by_magic` ships `{ "score": 1, "points": 5 }`, and 5
points does read as Score 1 on the curve — but there is **no load-time assertion**
in `ruleset/integrity.rs` tying the two together, so a future carrier can ship a
`score` that silently contradicts the score the engine computes, with no warning
and no test.

**Why it is B16's business.** **F-494** proposes adding the second carrier ever.
An author following the shape of the existing one would naturally write
`{ "score": 4, "points": 3 }` — echoing ArMDE:6648's level-4 Reputation, or
guessing — and the 4 would be discarded in silence while the character read as
Score 0. The trap is laid directly in this batch's remedy path, which is why it is
reported here rather than left to B18.

**Correct value.** Either drop the field (it is inert by design, per A22's own
documentation) or add a `ruleset/integrity.rs` assertion that
`score == score_for_xp(points)` for every carrier. Dropping it is the YAGNI answer
and would make F-494's correction unambiguous.

**Severity: LOW today** — nothing is currently wrong — **but it is a live trap
rather than a latent one**, because a correction this batch recommends walks into
it.

*(Found by the verification sub-agent while computing F-494's Warping figure.)*

### F-501 — five `Anmerkung` cells make provenance claims the core book contradicts, and the audit has been applying two standards to them *(D6 rule 1)*

| row | claim | what ArMDE says |
|---|---|---|
| `tugenden-fehler.md:444` | Poor Concentration's -3 is an "SdM:M-Hinweis" (RoP:M) | ArMDE:6604 states it |
| `tugenden-fehler.md:559` | Pagan is from *HM:RE* | ArMDE:6570-6573 carries it |
| `tugenden-fehler.md:699` | Painful Magic is from *HoH:TL* | ArMDE:6574-6577 carries it |
| `tugenden-fehler.md:708` | Palsied Hands is from *HoH:TL* | ArMDE:6578-6581 carries it |
| `tugenden-fehler.md:763` | Raised from the Dead is from *SdM:G* (RoP:D) | ArMDE:6646-6649 carries it |

**The inconsistency, which the verification pass caught in my own handling.** My
Method escalated these five to B15's **Q-121** while **F-497** condemns
`reputationen.md:112-113` outright — and D6 rule 1 names *"from which book"*
explicitly among the factual claims where the rulebook wins, in the same sentence
as magnitude and Reputation level. Applying Q-121 to one and a finding to the
other is two standards for one rule inside one batch, which is the exact shape
D6's own 2026-09-21 correction was written to stop. **Either both are findings or
both are Q-121**, and since F-497 is plainly a finding, so are these.

**Correct value.** The `Anmerkung` column drops or corrects the book attribution
on all five rows. Per **D6** the correction is made here **and** in
`arm-de-translation` in the same step — subject to the same stale-copy check as
F-496 and F-497, which cannot be performed from this repository.

**Two things this does not claim.** The German *names* in all five rows are
**correct** and agree with both the shipped data and the DE rulebook heading
(except Palsied Hands and Raised from the Dead, whose *name* dispute is F-496's,
not this finding's). And Q-121 is not withdrawn — it asks the broader question of
what a `tugenden-fehler.md` `Anmerkung` is *for*, which is worth a ruling
independently of these five rows being wrong.

**Severity: LOW for the shipped app** (no user-facing string is affected)
**, MEDIUM for the repository** — same reasoning as F-497: a canonical reference
that the next agent generating German rules text will read, telling it five
core-book Flaws come from supplements. The `:763` row is the worst of the five,
because *SdM:G* (*RoP: The Divine*) has **no German source file in
`rules/source/de/`** at all, so its claim cannot even be checked from the side the
table is written for.

*(Raised by me, at the verification pass's insistence that F-497 and Q-121 cannot
both stand as they were written.)*

## Open questions

**Q-124 — this span carries both `source.lines` conventions, which turns B12's
Q-97 from a documentation disagreement into a data inconsistency.** Thirty-two of
the batch's 35 ranges end on the **blank line** before the next heading
(`flaw.outcast` 6538-6541, next heading 6542; and so on). Three end on the last
**non-blank body line** instead: `flaw.poor` `[6594, 6596]` where 6597 is blank
and the next heading is 6598; `flaw.poor_characteristic` `[6598, 6600]` where 6601
is blank and the next is 6602; `flaw.poor_student` `[6626, 6628]` where 6629 is
blank and the next is 6630. Check 1's stated convention ("the entry's own `####`
heading to the line before the next heading") makes the 32 right and the 3 wrong;
`RULES.md:4471-4474`'s proposed convention ("stop at the last non-blank body
line") makes the 3 right and the 32 wrong. Either is defensible; **a span
containing both cannot be**, and this is the first batch to find them mixed inside
one span rather than one form in one batch and the other in another. B09, B10, B11
each met the tighter variant on one or two entries and called both forms correct,
which is how the mixture survived. **What would settle it:** the Q-97 ruling on
which form is canonical, after which these three (or the other 32) are a data-only
correction. Not counted as three findings here, because counting them would
pre-empt that ruling.

**Q-125 — the rulebook contradicts itself about Prohibition's category, and the
audit should record which side it is following.** ArMDE:6639's descriptor says
"*Minor, **Supernatural***" and the book's "List of Flaws" files it at ArMDE:5552
under `### Supernatural, Minor` (heading ArMDE:5534). ArMDE:6719
(`flaw.servant_of_the_land`) calls it "the Minor **Personality** Flaw:
Prohibition". The shipped `categories: ["supernatural"]` follows the two-against-one
majority and I have marked the entry clean on that basis — but the audit has no
recorded rule for a *book* that disagrees with itself, only for a repository
authority that does (B13's `decisions.md` self-contradiction) and for a table that
disagrees with the book (D6). **Why it matters beyond bookkeeping:** the two
categories behave differently. `supernatural` is on every profile's
`permitted_categories` with no cap; `personality` carries
`{"max":1,"major_only":true,"hard":true}` and `{"max":2}` on every profile (grog 0
and 1), so a Prohibition filed as Personality would consume one of a companion's
two Personality slots and would be **unavailable to a grog who already has one**.
And F-498's proposed grant would then push a granted Personality Flaw past a cap
that `validate_caps` reads off the bought list — so the two readings differ in a
way the engine can see. **What would settle it:** a ruling on whether a
descriptor + index majority beats a single in-prose restatement, recorded in
`decisions.md` so the next such case is not re-argued. (I believe the answer is
obviously yes; I am asking because the audit has no written rule and because
F-498's remedy depends on it.)

**Q-126 — is an entry whose only mechanical content is a *selection restriction*
`narrative`?** This adds to B15's **Q-118 / Q-119** rather than answering them,
and this span is dense with the shape. Four entries here state a restriction and
nothing else mechanical: `flaw.outcast` ("You may not take the Wealthy Virtue",
ArMDE:6540), `flaw.outlaw_leader` ("Grogs may not take this Flaw", ArMDE:6548),
`flaw.primogeniture_lineage` ("can only be taken by magi of House Verditius",
ArMDE:6636) and — softly — `flaw.outsider_*` ("companions **should** take the
Major version … Grogs **may** take the Minor version", ArMDE:6558). The repo has a
written position, and it is circular. `uncomputed_clauses.rs`'s
`NO_RULE_DESPITE_TOKEN` entry for `flaw.primogeniture_lineage` argues the entry is
correctly `narrative` because "the rule is **enforced** rather than merely
described, and describing it again would not be `uncomputed_rule`" — which makes
the classification depend on whether the restriction **happens to be encoded**,
i.e. on engine state, which the README says `classification` never is. Under that
position `flaw.outcast` is `uncomputed_rule` today and becomes `narrative` the
moment B09's F-340 is applied — the classification of an entry would change
because an edge was added to a *different* entry, which cannot be right. I have
followed B15's precedent (`flaw.no_sense_of_direction`,
whose only mechanical content is an unencoded incompatibility, was called
`narrative ✓` with the missing edge as a pure data finding) and kept all four
`narrative`, so that this batch does not silently set a different one. **What
would settle it:** a ruling on whether a prerequisite / incompatibility /
`forbidden_traits` restriction counts as "the book states something mechanical",
independent of whether the data encodes it — plus, if the answer is yes, a
rewrite of that exemption's rationale, which currently states the opposite.

**Q-127 — a granted Reputation's `score` cannot hold the range ArMDE:6554
states, and Phase 0's question about what `score` *means* is still open.** The
Phase-0 note asks "whether a granted Reputation's `score` is a ceiling, a fixed
value or a suggestion (`ArMDE:2512-2514`)" and records that the passage had not
been read. **It has now been read and it does not answer the question.**
ArMDE:2514 in full: *"Characters only start with a Reputation if they choose a
Virtue or Flaw that grants one, but all characters can develop them in play. See
page 28 for rules on Reputations."* That establishes the *gate* — which is
precisely what `validate_reputations` enforces (kinds and counts) — and says
nothing about the level. What this batch adds is a harder constraint from the
other direction: ArMDE:6554 gives Outsider "a bad Reputation of **level 1 to 3**
(depending upon how easy it is to identify you)", which is a **range**, and `A25`'s
`score` is a single `u8`. So whatever `score` means, it cannot mean "the value" for
every carrier. The three readings and their consequences for **F-486**: *floor*
→ both ids ship `score: 1`; *ceiling* → both ship `score: 3`; *pre-fill
suggestion* → either, with the range in `description`. **What would settle it:** a
ruling, plus — if the answer is "a range is real" — a decision on whether `A25`
gains a `score_max` or the range lives only in text. Note the shipped data already
answers "fixed value" implicitly on 33 carriers and answers it *inconsistently*
here (3 for Major, 1 for Minor, from one range).

**Q-128 — should `flaw.plagued_by_supernatural_entity` carry an `entity`
parameter?** The heading (ArMDE:6590) and descriptor are generic and the passage
gives its four kinds as prose examples ("a demon trying to corrupt you, an angel
trying to save you, a faerie playing games with you, or a ghost continuing the
agenda she had while alive"). But every cross-reference in the book writes it as
though it were parameterized: ArMDE:2778 "no character can take **Plagued by
Demon** as a Flaw", ArMDE:2645 "the **Plagued by an Angel** or Supernatural
Nuisance Story Flaws", and the sample characters at ArMDE:1502 ("**Plagued by
Angel**") and ArMDE:2014 ("**Plagued by Faerie**"). There is no defect today —
the *absence* of a parameter is what makes the default `max_per_target` correctly
forbid a second copy, and every profile caps Story Flaws at 1 anyway — so this is
a modelling question, not a correction. It matters because ArMDE:2778 states a
real rule scoped to one *value* ("if you do not want to involve demons in your
saga, no character can take Plagued by Demon"), which is unrecordable without the
parameter, and because the German name would then need a `{being}`-style template
like `flaw.offensive_to_beings` already uses. **What would settle it:** a ruling on
whether the book's cross-reference spelling or the entry's own heading governs
parameterization. Related to, but not the same as, B15's `flaw.magical_being_companion`
finding, where the parameter exists and the *multiplicity cap* is missing.

**Q-129 — which German name is canonical for Primogeniture Lineage and Palsied
Hands: the translation table's or the DE rulebook heading's?** This is **F-496**
restated as the reason two entries are marked *escalated, not checked* rather than
clean or failed. `flaw.primogeniture_lineage` ships **Erstgeburts-Abstammung**
(DE 6634's heading) against `grundbegriffe.md:327`'s *Primogenitur-Abstammungslinie*,
and `flaw.palsied_hands` ships **Zitternde Hände** (DE 6578's heading) against
`tugenden-fehler.md:708`'s *Zittrige Hände*. Every other check on both entries
passes. D6 rule 2 says the glossary wins on terminology, which would make both
shipped names wrong; D6's closing paragraph says a table row disagreeing with the
rulebook may instead be a **stale copy** of `arm-de-translation`, which cannot be
read from this repository. `crates/arm-rules/RULES.md` records no rationale for
either shipped name. **What would settle it:** the orchestrator comparing both
rows against `arm-de-translation`. (`flaw.raised_from_the_dead` has the same
dispute but is a failure on other grounds, so it is not escalated.)

## Sub-agent reconciliation

One verification sub-agent, briefed to **re-derive** rather than review, with the
20 clean verdicts and the ten highest-stakes findings named individually and the
ten Method censuses listed for independent recount. It wrote no files and
confirmed it received the "auto mode" injection and ignored it.

### What it overturned

**One finding, and it is the most important result of this batch.** **F-481** was
a re-report of **B09's F-340**, whose passage table already carries ArMDE:6540 —
the exact sentence I raised it on — and half of **F-482**'s five passages
(:3611, :3631, :4494) were three more of F-340's eight. B10's **F-376** is the
same defect on a third site. I had not read the prior batch files looking for a
Wealthy/Poor finding, and nothing in the repository indexes findings across
batches, because `corrections.md` does not exist (**F-499**, which is the pass's
finding and exists *because* of this duplication).

F-481 is withdrawn in place, with its original text retained below the withdrawal
so the duplication is visible rather than deleted. F-482 is restated as an
**extension** of F-340: two new flat prohibition sites (ArMDE:3406, :4043), one
new conditional one (:4800, left to B07), and the reachability proof that retires
F-340's open "would not catch a granted Redcap" caveat. The pass also made the
passage list **mechanical** rather than sampled — one `rg` over the prohibition's
six spellings returns eleven passages and nothing else in the book, so "eight" and
"five" both become "ten flat plus one conditional", stated as a closed set for the
first time.

### What it corrected without changing a verdict

- **F-490's first half had its reasoning backwards.** I wrote that a bought -4 is
  legal and the summary therefore narrows what the engine allows.
  `characteristics.json` is `{"base_min": -3}` with a costs table bottoming at -3,
  and `validate_characteristic_delta_preconditions` reads the **bought** score, so
  a bought -4 cannot exist and "already at -3" is a *truthful* description of the
  validator. The finding survives, but its bite moves: the reachable case is
  `flaw.dwarf`'s `characteristic_score_delta`, where a Dwarf with Strength bought
  at -2 is "already -3 or lower" *effectively* and is still refused. The pass also
  found the mirror narrowing on `virtue.great_characteristic` (ArMDE:3989 "at
  least +3" versus its summary's "at +3"), so the text fix belongs on both
  entries.
- **Two arithmetic errors in the Method.** "the four entries
  `NO_RULE_DESPITE_TOKEN` exempts" against my own sum of nine — there are three.
  And `tainted` is **absent** on all 35 rather than present-and-false; the verdict
  is unchanged, the wording was not accurate.
- **One Part C row omitted.** "Five of Part C's rows touch this batch" is six:
  **C3b**, `LaterLifeXpRate`'s unreachability without a life-stage plan, touches
  `flaw.poor` and is exactly why that entry is clean rather than a finding. B09
  named it when it met the same effect; I did not.
- **Two F-497 narrowings that make the table row worse.** `reputationen.md` has
  **no row for Outlaw Leader at all** — it files that entry's number under a
  fictitious "Outlaw (Minor)" — and `:114` is Outsider **(Major) only**, so my
  Method's "*Außenseiter (Groß/Klein)*" overstated its coverage.
- **A D6 inconsistency inside my own batch.** I escalated five `Anmerkung`
  provenance claims to Q-121 while condemning `reputationen.md:112-113` outright,
  although D6 rule 1 names "from which book" in the same sentence as magnitude and
  level. Raised as **F-501** rather than left as two standards for one rule.
- **The inbound-site count.** The pass declined to confirm my list site by site
  and proved the *sweep* incomplete within the families it did check, so "31
  inbound sites" is reported as derived-but-unverified, and the "six add a rule"
  figure is restated as "six expose a defect, three of them new to the audit".
- **The F-409 ordinals.** "instances 35, 36 and 37" rested on the brief's running
  total, which the pass could not reconstruct (`batch-12.md:467` says "at least
  twenty-nine"). Family membership is confirmed; the ordinals are dropped.

### What it strengthened

- **F-486 — git provenance.** The effects were added in `1b7a5eb`, whose message
  claims every score is source-cited, hours *after* `e2e9202` wrote the RULES.md
  "1-3" rows for both ids. And the wrong number is not inert:
  `ui/src/lib/components/Reputations.svelte:39` pre-fills it onto the sheet.
- **F-487 — root cause, demonstrated three ways.** 33 dual-magnitude descriptors
  exist, 32 ship as pairs, `flaw.pagan` is **the only exception in the entire
  catalogue**; `e0e80f3`'s own commit message says "(32)"; and `699d43b` shows the
  entry was hand-authored earlier for Spirit Votary and so excluded from the
  byte-for-byte-preserved split. Plus a **third edit site**:
  `validate_magnitude_variant_exclusivity` fails the load on a pair without mutual
  `incompatible_with`.
- **F-494 — the number I flagged as needing a check.** 3 points → Warping Score
  **0** (the curve's first step is 5 XP → 1), so `WarpingOwed::from_score(0)` is
  `{0,0,0}` and the remedy is safe; plus two side effects worth writing into the
  correction.
- **F-489 — exhaustive rather than argued.** The pass enumerated all 42 `Effect`
  variants and confirmed that the five naming an Ability all move a score, a cost
  or a permission, and **none** can name an Ability for a *roll* modifier.
- **F-496 — fifteen further citations checked.** Every other table-row line
  number in my Method table was spot-checked and all fifteen are exact.

### What it confirmed unchanged

All **20** clean verdicts, re-derived entry by entry — none overturned, including
the four I flagged as least confident (`flaw.poor`, `flaw.poor_formulaic_magic`
via `CastType::matches`, `flaw.poor_living_conditions` via `aging.rs:448`'s
subtraction, and `flaw.prohibition` via the ArMDE:5552/5534 index filing). Both
`NO_RULE_DESPITE_TOKEN` readings that touch this batch were checked against their
passages and **confirmed**, so there is no finding against an exemption. The
heading sequence, the 18/6/6/5 census, the three short `source.lines` ranges with
their blank lines, the nine anchors, the six descriptions, the full sign census,
`SIGN_CHARS`'s omission of U+2014, the guard's English-source screening, the
hand-run of `states_a_mechanical_rule` over all 18 narrative passages (three trip
nothing and state a rule; no fourth), the zero-carriers result, the bought-only
result, and every profile cap were each re-derived and matched.

### What it could not check, and why

`arm-de-translation` is outside the repository and denied by argument-path
containment, so **F-496**, **F-497**, **F-501** and **Q-129** all remain
"this repository's copy disagrees with the rulebook" and the stale-copy-versus-real-error
determination stays with the orchestrator. That is a confirmed blocker, not a
skipped check.

### The finding that refused to be withdrawn

I asked the pass to try to overturn things, and it reports one attempt that the
evidence stopped: the DE-only `name_unfilled` key on `flaw.realm_stigmatic`
looked like a locale asymmetry and is a recorded decision at
`crates/arm-rules/RULES.md:2368-2374`. It is noted here because a refused
withdrawal is the same kind of result as a successful one, and because it is the
second time in two batches that `RULES.md` has been what stopped a false finding
about that entry's German rendering.
