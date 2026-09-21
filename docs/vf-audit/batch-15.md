# Batch B15 — indices 490-524, ArMDE:6370-6537

Entries: 35. Audited: 35. Failures: **13**. Escalated, not checked: **3**
(`flaw.monastic_vows_hermetic`, `flaw.necessary_condition`,
`flaw.oath_of_fealty`). Clean: **19**. 13 + 3 + 19 = 35.
Findings **F-460 … F-480** — **nine** reach outside the batch: **F-465** carries
`flaw.sleep_disorder` from **B17**'s span (ArMDE:6745-6750), **F-466** carries
`virtue.well_traveled` from **B09**'s span (ArMDE:5239-5242), **F-472** lands
entirely on `virtue.cyclic_magic_positive` in **B02**'s span (ArMDE:3635-3638),
**F-474** on `flaw.blatant_magical_air` in **B10**'s span (ArMDE:5715-5718),
**F-475** on `virtue.alluring_to_beings` in **B01**'s span (ArMDE:3388-3395),
**F-476** on `virtue.blood_of_the_nephilim` (ArMDE:3504-3518, **B01**),
**F-477** on `virtue.demonic_blood` (ArMDE:3649-3662, **B02**), **F-479** on
`virtue.mendicant_friar` (ArMDE:4488-4495, **B06**), and **F-480** on
`rules/source/de/translation-tables/tugenden-fehler.md`. Four of those
(F-474 … F-477) were reached through an *inbound* pointer naming an entry in this
span and are invisible to a reader of ArMDE:6370-6537 alone; two (F-472, F-479)
were reached by a catalogue-wide scan this batch's Method introduces.
Open questions **Q-118 … Q-123**, of which **Q-120 is closed** on the enum's own
doc comment.

**The verification sub-agent re-derived all 19 entries this file calls clean and
overturned none, confirmed all three of the withdrawals, confirmed every finding
— narrowing one (F-469), correcting the proposed fix on another (F-466), and
strengthening a third with git evidence (F-462) — caught one census error my own
`rg` re-run had not reached, closed one open question, and found the batch's
third truncated summary, which is in **English**.** Two findings are the pass's
alone: **F-479** and **F-480**. See the reconciliation section.

**The batch's first shape is the one the brief named and it is here twice, once
in each of the two classes that can hide it.** ArMDE:6394 lets Magical
Fascination's bearer "have a score of 1 (but no more) in **either Magic or
Faerie Lore** at character creation", and ArMDE:6456 lets Monstrous Blood's
bearer "**learn Magic Lore during character creation**". `ability.magic_lore`
and `ability.faerie_lore` are both `arcane`, `abilities.json`'s
`categories_requiring_virtue` is `["academic","arcane","martial"]`, and neither
entry carries `ability_authorization` or `restricted_ability_xp`. So a grog,
companion or mythic companion who takes either Flaw and does exactly what its
sentence permits is told his character is illegal — a hard
`ability_category_requires_virtue` from
`validation/authorization.rs::validate_ability_authorization`, whose only
whole-character exemption is `profile.is_magus` (**F-460**, **F-461**).
That is F-409's mechanism, instances **31** and **32** against the **2** entries
that encode it (`flaw.covenant_upbringing`, `virtue.student_of_realm`, both
re-derived here) plus **10** implicit carriers. **It is also the whole answer to
the brief's zero-`creation_effect` question**: the span does state a
creation-time mechanic, twice, and both statements are sitting in another class —
one in `narrative`, one in `in_play_effect`.

**The second shape is a stated prohibition that no `incompatible_with` carries,
three times, and one of the three is reachable by every magus in the game.**
ArMDE:6384: "You may not take this Flaw if you actually do have The Gift" —
`flaw.magical_air`'s `incompatible_with` holds `flaw.offensive_to_beings` and
not `virtue.the_gift`, and since `virtue.the_gift` is **bought** (the magus
profile requires `virtue.hermetic_magus`, whose prerequisite is
`Has(virtue.the_gift)`; nothing grants it), the bought-only
`validate_incompatibilities` would catch the pair if it were declared and
catches nothing today (**F-463**). ArMDE:6492 and ArMDE:6749 state the Night
Terrors / Sleep Disorder exclusion **from both sides** and neither entry encodes
it (**F-465**). ArMDE:6502 states the No Sense of Direction / Well-Traveled
exclusion from one side and neither entry encodes it (**F-466**).

**The third shape is the batch's one live wrong number, and it is a penalty the
book scopes to one kind of weapon being applied to every kind.** ArMDE:6436
gives Missing Eye "**-3 on Attack rolls for missiles** and targeting rolls for
spells. **In melee combat you suffer -1** on Attack rolls". The entry ships a
single unscoped `combat_mod { -1, attack }`, and
`derived/combat.rs::combat_totals`' `cm` closure adds the unscoped sum to
**every** equipped weapon's line — so a character with Missing Eye and a long
bow is shown Attack -1 where the book says -3, and the -3 reaches no total at
all (**F-462**). The weapon-scoping machinery this needs already exists and
`flaw.lame` already uses it, so this is not a Part C gap; and
`crates/arm-rules/RULES.md:5161` asserts the opposite, naming "missing_eye's
ranged −3" among the amounts that *are* folded.

**The fourth shape is where more than a third of this batch's findings came
from: the rule is in the *other* entry.** Following every pointer out of the
span — including inbound ones — turned up four defects no reader of
ArMDE:6370-6537 could see. ArMDE:3392 states an eligibility gate naming Magical
Air that `virtue.alluring_to_beings` does not encode, while three siblings in the
same family encode the identical sentence (**F-475**). ArMDE:3517 forbids Blood
of the Nephilim to a Lycanthrope and `incompatible_with` is empty (**F-476**).
ArMDE:5717 restricts Blatant Magical Air to characters with Magical Air or The
Gift and carries no `prerequisites` at all (**F-474**). ArMDE:9906 makes the
lycanthrope immune to Mentem spells and reaches the user in neither locale
(**F-478**). And two further F-409 instances fell out of the same reads —
ArMDE:3511's Dominion Lore and ArMDE:3655's Infernal Lore, both phrased
"**without needing to take the Arcane Lore Minor Virtue**", both unencoded
(**F-476**, **F-477**). That idiom is `grep`-able, which makes it the cheapest
route yet to F-409's remaining population.

*(Written incrementally: header, method and the 35-row verdict table first,
findings appended one at a time, sub-agent reconciliation last.)*

## Method

**Both languages were read as continuous prose before any entry was judged** —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` 6360-6549 and
the line-parallel `rules/source/de/Ars Magica Definitive Edition Basisregeln.md`
6360-6549, deliberately overrunning the span at both ends so the first and last
entries' boundaries could be seen against their neighbours
(`flaw.low_self_esteem` 6362 and `flaw.low_tolerance` 6366 before,
`flaw.outcast` 6538, `flaw.outlaw` 6542 and `flaw.outlaw_leader` 6546 after).

**Line parity holds throughout the span and through both overruns.**
`grep -n "^#### "` over each file, restricted to 6360-6560, returns
**byte-identical line-number sequences**: 6362, 6366, 6370, 6378, 6382, 6386,
6392, 6396, 6408, 6414, 6418, 6422, 6426, 6430, 6434, 6438, 6442, 6446, 6450,
6454, 6468, 6472, 6476, 6480, 6488, 6496, 6500, 6504, 6508, 6512, 6516, 6520,
6524, 6534, 6538, 6542, 6546, 6550. That is **32** headings inside 6370-6537 for
35 catalogue ids: the three Major/Minor pairs `Meddler` 6422, `Obsessed` 6520
and `Optimistic` 6534 each ship as two ids citing one heading. No nested or
blockquoted `####` appears anywhere in the span.

**The batch's own classification census matches the brief's.** Re-derived rather
than trusted, by `jq` over the 35 ids: **22 `narrative`, 9 `uncomputed_rule`, 4
`in_play_effect`, 0 `creation_effect`** (22+9+4+0 = 35). The brief's figures are
correct.

**The zero-`creation_effect` observation, answered.** The span is not free of
creation-time mechanics; two of them are filed elsewhere. `flaw.magical_fascination`
(`narrative`) and `flaw.monstrous_blood` (`in_play_effect`) each state a
creation-time permission to buy a gated Ability, which `A14 AbilityAuthorization`
expresses exactly and neither carries — see **F-460** and **F-461**. Both
belong in `creation_effect` once the effect is added (`flaw.monstrous_blood`
under **Q-120**, since it would then compute in *both* phases and the taxonomy
admits one answer). No other entry in the span states anything a creation-layer
consumer reads: nothing carries or should carry a budget, an XP pool, a
Characteristic delta, a Reputation grant, a Size delta or a spell-level figure.
So the correct census for this span is **2 `creation_effect`, not 0**, and the
absence was a symptom rather than a fact about the rulebook.

**Check 1 — `source.lines`: 35 of 35 correct.** Every range runs from its own
`####` heading to the line before the next heading, verified one by one against
the heading sequence above. The **five** multi-paragraph ranges are all right:
`flaw.manifest_sin` 6396-6407 (five paragraphs), `flaw.monstrous_blood`
6454-6467 (six), `flaw.necessary_realm_aura_for_ability` 6480-6487 (three),
`flaw.night_terrors` 6488-6495 (three) and `flaw.offensive_to_beings` 6524-6533
(four). No range stops short of the blank line before the next heading, so there
is no new instance of B12's **Q-97** here.

**Check 1, second half — `source.anchor`: 9 of 35 entries carry one, all nine
correct**, verified twice — derived from the `####` heading, and against the
book's own "List of Flaws" index links, which spell them out literally.

| id | anchor | heading | index link |
|---|---|---|---|
| `flaw.lycanthrope` | `lycanthrope` | ArMDE:6370 | ArMDE:5398 |
| `flaw.magic_addiction` | `magic-addiction` | ArMDE:6378 | ArMDE:5293 |
| `flaw.manifest_sin` | `manifest-sin` | ArMDE:6396 | ArMDE:5549 |
| `flaw.missing_ear` | `missing-ear` | ArMDE:6430 | ArMDE:5600 |
| `flaw.mute` | `mute` | ArMDE:6472 | ArMDE:5412 |
| `flaw.necessary_realm_aura_for_ability` | `necessary-realm-aura-for-ability` | ArMDE:6480 `#### Necessary (Realm) Aura for (Ability)` | ArMDE:5551 |
| `flaw.night_terrors` | `night-terrors` | ArMDE:6488 | ArMDE:5604 |
| `flaw.nocturnal` | `nocturnal` | ArMDE:6504 | ArMDE:5606 |
| `flaw.offensive_to_beings` | `offensive-to-beings` | ArMDE:6524 `#### Offensive to (Beings)` | ArMDE:5445 **and** ArMDE:5608 |

The two non-trivial ones are both right: the parentheses in
`Necessary (Realm) Aura for (Ability)` and in `Offensive to (Beings)` are
stripped. **The anchor population is exactly the batch's nine `uncomputed_rule`
entries** — the first batch in a while where the two sets coincide, so B11's
**Q-96** (anchors on some classes and not others) gains no new instance here and
no `uncomputed_rule` entry is missing one.

**Checks 3, 4, 5, 6 (`kind` / `magnitude` / `entity_kinds` / `categories` +
`tainted`): 35 of 35 correct**, each read against its own descriptor line one by
one:

- `*Major, Supernatural*` — lycanthrope (6371)
- `*Major, Hermetic*` — magic_addiction (6379), monastic_vows_hermetic (6451), necessary_condition (6477)
- `*Major, General*` — magical_air (6383), mute (6473), no_hands (6497)
- `*Minor, Story*` — magical_being_companion (6387), mentor (6427)
- `*Minor, Story, Tainted*` — manufactured_ignorance (6409)
- `*Minor, Personality*` — magical_fascination (6393), noncombatant (6509)
- `*Minor, Supernatural*` — manifest_sin (6397), monstrous_blood (6455), necessary_realm_aura_for_ability (6481)
- `*Major, Story*` — many_marriageable_daughters (6415), mistaken_identity (6443), monastic_vows (6447), oath_of_fealty (6513)
- `*Minor, General*` — master_of_none (6419), missing_ear (6431), missing_eye (6435), missing_hand (6439), motion_sickness (6469), night_terrors (6489), no_sense_of_direction (6501), nocturnal (6505), obese (6517)
- `*Major or Minor, Personality*` — meddler_\* (6423), obsessed_\* (6521), optimistic_\* (6535)
- `*Minor, Hermetic and General*` — offensive_to_beings (6525)

`kind` is `flaw` on all 35 ✓ — the whole span is inside the book's Flaws block
(ArMDE:5639-7113). `entity_kinds` is `["character"]` on all 35 ✓. **`tainted` is
`true` on exactly the one entry whose descriptor carries the tag**
(`flaw.manufactured_ignorance`, ArMDE:6409 `*Minor, Story, Tainted*` / DE 6409
`*Klein, Geschichte, Befleckt*`) and `false` on the other 34 ✓. One entry invites
the mistake and does not get it: `flaw.manifest_sin` closes with "This Flaw
originates from **either the Divine or the Infernal Realm**" (ArMDE:6406), which
is a half-Infernal association its descriptor deliberately does not tag —
`tainted` means Infernal-realm associated (ArMDE:2998-3002), and following the
descriptor is correct. **No descriptor line in this span is malformed** — every
one of the 32 carries its comma.

**The index confirms every filing, and one entry is dual-listed.** All 32
headings are filed in the book's own "List of Flaws" (ArMDE:5283-5638) under a
`### <category>, <magnitude>` heading matching both the descriptor and the data:

| index lines | heading | entries |
|---|---|---|
| 5293, 5294, 5295 | `### Hermetic, Major` (5285) | Magic Addiction, Monastic Vows (Hermetic), Necessary Condition |
| 5326-5328 | `### Personality, Major or Minor` (5310) | Meddler, Obsessed, Optimistic |
| 5360-5363 | `### Story, Major` (5340) | Many Marriageable Daughters, Mistaken Identity, Monastic Vows, Oath of Fealty |
| 5398 | `### Supernatural, Major` (5387) | Lycanthrope |
| 5411-5413 | `### General, Major` (5401) | Magical Air, Mute, No Hands |
| 5445 | `### Hermetic, Minor` (5417) | Offensive to (Beings) |
| 5486, 5487 | `### Personality, Minor` (5467) | Magical Fascination, Noncombatant |
| 5513-5515 | `### Story, Minor` (5501) | Magical (Being) Companion, Manufactured Ignorance, Mentor |
| 5549-5551 | `### Supernatural, Minor` (5534) | Manifest Sin, Monstrous Blood, Necessary (Realm) Aura for (Ability) |
| 5599-5608 | `### General, Minor` (5564) | Master of None, Missing Ear, Missing Eye, Missing Hand, Motion Sickness, Night Terrors, No Sense of Direction, Nocturnal, Obese, Offensive to (Beings) |

**`Offensive to (Beings)` is the dual-listed one — ArMDE:5445 under Hermetic,
Minor *and* ArMDE:5608 under General, Minor — and the shipped `categories:
["general"]` + `index_categories: ["hermetic"]` is a recorded decision, not a
defect.** I had this drafted as a check-6 failure and **withdrew it on
`crates/arm-rules/RULES.md:386-490` and `:943-996`**, which is B10's lesson
applied: the split is Norbert's ruling of 2026-09-13 (an *and*-joined descriptor
means "either route"), `hermetic` is deliberately kept out of `categories`
because `gift_categories` is `["hermetic"]` on every profile and adding it would
make an unGifted companion count as Gifted (pinned by
`data_integrity.rs::a_companion_holding_offensive_to_beings_is_not_gifted`), and
`index_categories` exists precisely to carry the book's index heading for
`validate_house`'s ArMDE:2860 guideline. The entry is clean.

**Check 12, mechanically.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the 35 ids in each locale returns
`["name","summary"]` for **25** entries and `["description","name","summary"]`
for **10** — the same 10 in both locales, so the locales never disagree about
*whether* a description exists. The 10 are **8 of the batch's 9
`uncomputed_rule` entries** (`lycanthrope`, `magic_addiction`, `manifest_sin`,
`mute`, `necessary_realm_aura_for_ability`, `night_terrors`, `nocturnal`,
`offensive_to_beings`), each carrying its full cited passage in both languages
✓, **plus two `in_play_effect` entries that carry one although their class does
not oblige it** — `flaw.missing_eye` and `flaw.missing_hand`. The ninth
`uncomputed_rule` entry, `flaw.missing_ear`, carries **no** description and does
not need one: its whole passage is a single sentence which *is* its summary,
`-3` included, in both locales ✓ — B14's `flaw.loose_magic` reading, re-derived.
**No `narrative` entry in this batch carries a description in either locale**,
which is where the D5 half of six findings comes from.

**And the precedence that makes `flaw.missing_ear` legitimate is stated in the
guard itself**, so it is cited rather than assumed: `uncomputed_clauses.rs`
builds an entry's displayed rules text as `description` **if present, else
`summary`** — "exactly the precedence `VirtueFlawTab.svelte`'s tooltip applies".
Every D5 judgement in this batch was therefore made against `description ??
summary`, not against `description` alone; on F-461, F-463, F-464, F-467, F-468,
F-469 and F-470 the clause is in **neither** field.

**One source typo is reproduced faithfully and is not a finding.** ArMDE:6492's
last sentence ends with no full stop — "…may not also take the Sleep Disorder
Flaw" — and `flaw.night_terrors`' shipped English description copies it exactly.
Correct behaviour for an extraction; worth the line so a later reader does not
"fix" the data away from its source.

*(One method correction worth recording, because it nearly produced two false
findings. My first pass piped `jq`'s per-entry dump through `grep -A3`, which
truncates a multi-paragraph description at three lines — and `flaw.lycanthrope`
and `flaw.manifest_sin` therefore looked like English descriptions cut to their
first paragraph while the German ones were complete. They are not:
a character-length and paragraph-count comparison of **every** `flaw.*`
description in both locale files returns **zero** rows where the EN and DE
paragraph counts differ, and `flaw.lycanthrope` is 3/3 and `flaw.manifest_sin`
5/5. The tool truncated, not the data.)*

**ASCII hyphen (check 12, second half): clean, verified two ways.**
`grep -c "−"` (U+2212) over `rules/i18n/en/virtues_flaws.json` and
`rules/i18n/de/virtues_flaws.json` returns **0** for both, catalogue-wide; and
`rg -c -- "–[0-9]"` (U+2013 followed by a digit) returns **no match** in either
file, so no en dash that does ship is used as a *sign*. The conversion works in
this batch's direction too: the DE source writes every negative in the span with
an en dash (DE 6376 `–1`/`+2`/`+3`, 6402 `–1`, 6432 `–3`, 6436 `–3`/`–1`, 6440
`–3`, 6456 `–1`, 6460 `–3`, 6462 `–3`, 6482 `–3`, 6490 `–1`/`–3`, 6498 `–5`,
6506 `–1`, 6518 `–1`/`–3`, 6528 `–3`) and every one that ships does so as ASCII
`-` (visible in the shipped DE descriptions of `lycanthrope`, `manifest_sin`,
`missing_eye`, `missing_hand`, `necessary_realm_aura_for_ability`,
`night_terrors`, `nocturnal` and `offensive_to_beings`).

**The two source files' sign census, re-derived with `rg` rather than the `grep`
shim** — B14's equivalent list was wrong in three places because the shim's
ugrep dialect returned an incomplete match set, and my own first draft of this
paragraph was wrong in three places for the same reason before I re-ran it:

| | EN (ArMDE) | DE |
|---|---|---|
| en dash + digit | 6376, 6432, 6436, 6460, 6462, 6506, 6518, 6528 (**8**) | 6376, 6402, 6432, 6436, 6440, 6456, 6460, 6462, 6474, 6482, 6490, **6498**, 6506, 6518, 6528 (**15**) |
| ASCII hyphen + digit | 6402, 6440, 6456, 6474, 6482, 6490 (**6**) | none (**0**) |
| `+` + digit | 6376, 6380 | 6376, 6380 |
| en dash **not** followed by a digit, used as a sign or operator | **6390** (`10 – Size`), **6498** (`– 5`) | 6390 only |

Three things fall out of that table and all three are load-bearing. **The German
source is uniform** — every negative in the span is an en dash, with no ASCII
anywhere, which is the same asymmetry B14 recorded in the other direction.
**ArMDE:6498 is in neither of the English digit rows**: it writes `– 5`, en dash
**plus a space** before the digit, where DE 6498 writes `–5` with no space. That
single space is what defeats the phrase screen, and because the screen reads the
English file only, the German spelling that *would* have tripped it never gets a
vote — see **F-464**. And **ArMDE:6390 is in no digit row in either language**:
`10 – Size` puts the only digit on the *left* of the operator, which is a second,
independent way past `has_signed_number` — see **F-469**. The shipped EN and DE
text carries ASCII throughout regardless ✓, but note that **ArMDE:6390 and
ArMDE:6462 are both en dashes in clauses this batch asks to be written into
`description`** (F-469 and F-461), so the correction pass must convert them.

*(The fourth row of this table is the verification pass's, not mine. My first
draft had 6462 and 6528 on the ASCII side and omitted 6474 — the ugrep-shim
failure `CLAUDE.local.md` warns about, which I caught and fixed with `rg` before
the pass ran; the pass then caught that even the corrected table had no row for
an en dash **not** followed by a digit, and so silently dropped ArMDE:6390.)*

**Truncation scan: this batch has one, and it is the shape B14 predicted and
said its own scan could not see.** The literal-ellipsis scan
(`jq` over both locale files for `...` or `…` in any `name`/`summary`/
`description`) returns exactly the same **one** id in each, `flaw.exciting_experimentation`
— B12's F-415, outside this span. So I ran two scans the ellipsis one cannot do:
a summary ending in a German or English abbreviation, and a summary containing an
**unclosed parenthesis**. Those return **two** rows, both German:
`flaw.lycanthrope` (**F-471**, in this span) and `virtue.cyclic_magic_positive`
(**F-472**, B02's span) — summaries cut mid-sentence at a `z.B.` / `z. B.` whose
period the extractor read as a sentence end.

**The verification pass found a third, and it is English — so the census is three
and my "both are German" was wrong.** `virtue.mendicant_friar`'s **English**
summary is the twenty-five-character fragment `"You are a follower of St."`
against ArMDE:4490's full sentence (**F-479**, B06's span), truncated at `St.`
by the same mechanism. **The scan that found it independently of guessing the
abbreviation is the one to adopt**: an **EN/DE summary length ratio above 2×**,
which measures locale asymmetry rather than a lexical shape and needs no
vocabulary at all. Catalogue-wide it returns exactly the two truncations plus one
benign row (`virtue.tough`, whose English summary is legitimately the four-word
"+3 to Soak."); below about 1.4× it goes noisy, because German is naturally
longer. Four further scans returned **zero** rows each and are recorded so the
next batch can skip them: a summary not ending in sentence punctuation; an
unbalanced bracket in either locale; an EN/DE `description` paragraph-count
mismatch; and an EN/DE description-presence mismatch. Stated exactly as checked:
five scans over the two shipped locale files, which together would still not see
a summary truncated at a full stop that genuinely ends a sentence.

**German names against the canonical tables.** Located by English key, **14** of
the 32 distinct English names in this batch have a row in
`rules/source/de/translation-tables/`. **All 14 agree** with the shipped
`rules/i18n/de/` name and with the DE rulebook heading; **none disagrees**, so
this batch raises no D6 name dispute. The remaining **18** English names have no
table row and were checked directly against the DE rulebook heading on the
parallel line — **all 18 match ✓**. (14 + 18 = 32 ✓, and the 32 distinct names
cover 35 ids because three are Major/Minor pairs.)

| English | table row | DE rulebook heading | shipped DE name | verdict |
|---|---|---|---|---|
| Lycanthrope | `tugenden-fehler.md:376` *Lykanthrop* | 6370 Lykanthrop | Lykanthrop | ✓ (row's note attributes the Flaw to *HdH:S*; the core book carries it at ArMDE:6370 — see **Q-121**) |
| Magic Addiction | `tugenden-fehler.md:301` *Magiesucht* | 6378 Magiesucht | Magiesucht | ✓ |
| Magical Air | `grundbegriffe.md:200`, `sphären-mächte.md:152` *Magische Ausstrahlung* | 6382 Magische Ausstrahlung | Magische Ausstrahlung | ✓ — and this row is what makes **F-473** a source error rather than a naming choice |
| Meddler | `tugenden-fehler.md:377` *Einmischer* | 6422 Einmischer | Einmischer (Groß/Klein) | ✓ (note attributes to *GotF* — **Q-121**) |
| Mentor | `tugenden-fehler.md:402` *Mentor* | 6426 Mentor | Mentor | ✓ |
| Missing Ear | `tugenden-fehler.md:438` *Fehlendes Ohr* | 6430 Fehlendes Ohr | Fehlendes Ohr | ✓ |
| Missing Eye | `tugenden-fehler.md:439` *Fehlendes Auge* | 6434 Fehlendes Auge | Fehlendes Auge | ✓ |
| Missing Hand | `tugenden-fehler.md:440` *Fehlende Hand* | 6438 Fehlende Hand | Fehlende Hand | ✓ |
| Mistaken Identity | `tugenden-fehler.md:403` *Verwechslung* | 6442 Verwechslung | Verwechslung | ✓ |
| Monstrous Blood | `tugenden-fehler.md:331` **and** `:346` *Monströses Blut* | 6454 Monströses Blut | Monströses Blut | ✓ on the name — but **two rows for one English headword**, and `:346` states a magnitude and an effect: **F-480**, with its provenance fragment left to **Q-121** |
| Necessary Condition | `tugenden-fehler.md:302` *Notwendige Bedingung* | 6476 Notwendige Bedingung | Notwendige Bedingung | ✓ |
| Noncombatant | `tugenden-fehler.md:441` *Nichtkämpfer* | 6508 Nichtkämpfer | Nichtkämpfer | ✓ |
| Oath of Fealty | `tugenden-fehler.md:404` *Treueeid* | 6512 Treueeid | Treueeid | ✓ |
| Offensive to (Beings) | `tugenden-fehler.md:444` *Abstoßend für (Wesen)* | 6524 Abstoßend für (Wesen) | Abstoßend für {being} | ✓ (attributes to *SdM:M* — **Q-121**) |

The **18** table-less names that match their DE heading: Magical (Being)
Companion → *Magischer (Wesen)-Gefährte*, Magical Fascination → *Magische
Faszination*, Manifest Sin → *Offenbarte Sünde*, Manufactured Ignorance →
*Fabrizierte Unwissenheit*, Many Marriageable Daughters → *Viele heiratsfähige
Töchter*, Master of None → *Meister von nichts*, Monastic Vows →
*Klostergelübde*, Monastic Vows (Hermetic) → *Klostergelübde (Hermetisch)*,
Motion Sickness → *Reisekrankheit*, Mute → *Stumm*, Necessary (Realm) Aura for
(Ability) → *Notwendige (Sphäre)-Aura für (Fertigkeit)*, Night Terrors →
*Nachtschrecken*, No Hands → *Keine Hände*, No Sense of Direction → *Kein
Orientierungssinn*, Nocturnal → *Nachtmensch*, Obese → *Fettleibig*, Obsessed →
*Besessen*, Optimistic → *Optimistisch*.

Two near-misses that are **not** disagreements, checked and dismissed:
`tugenden-fehler.md:378` gives *Obsession → Besessenheit*, a different English
headword from this batch's `Obsessed`, whose shipped *Besessen (Groß/Klein)* is
the uninflected adjective the repo's standalone-label convention asks for; and
`flaw.necessary_realm_aura_for_ability` ships the DE name `Notwendige Aura für
{ability}, {realm}`, whose word order differs from the DE heading — a **recorded
decision** at `crates/arm-rules/RULES.md:2356-2391`, taken because the German
realm labels are noun phrases carrying their own article (*Das Göttliche*) and
every template that put the token before a noun rendered ungrammatical German.
Withdrawn, not counted.

**One German-source grammatical error sits in this span and does *not* ship
today — but two of this batch's findings would make it ship.** DE 6420 reads
"Erfahrungspunkte, die durch **einen** Fortschrittssumme erlangt wurden" — `die
Summe` is feminine, so it must be *eine*. It is the identical error B14 recorded
at DE 6356 and DE 6296 (**F-458**). It is in sentence two of
`flaw.master_of_none`, so today's summary-only text escapes it; the moment
**F-467** puts the passage into `description` it ships. Recorded here so the
correction pass fixes it in the same step.

**`rules/core/abilities.json`, `rules/core/character_types.json` and
`rules/core/equipment.json` were consulted** for the Abilities, profiles and
weapons these passages name: `ability.magic_lore` and `ability.faerie_lore`
(both category `arcane`), `categories_requiring_virtue` =
`["academic","arcane","martial"]`, the four profiles' `permitted_categories` /
`forbidden_categories` / `forbidden_traits` / `budget.flaw_category_caps`, and
the seven missile weapons (`weapon.bow_long`, `weapon.bow_short`,
`weapon.sling`, `weapon.javelin`, `weapon.axe_throwing`, `weapon.knife_thrown`,
`weapon.stone`) that make **F-462**'s scoped encoding expressible. Two profile
facts are load-bearing for findings here: `virtue.the_gift` appears on **no**
profile's `granted_selections` and on grog's `forbidden_traits`, and the magus
profile's `required_traits` is `["virtue.hermetic_magus"]`, whose own
`prerequisites` is `Has(virtue.the_gift)` — so The Gift is always a **bought**
selection, which is exactly what makes **F-463**'s proposed `incompatible_with`
effective despite that validator's bought-only scope.

### Part C systemic gaps are not re-reported per entry

Five of Part C's rows touch this batch and none is counted against an entry:

- **`HealthMod`'s three surfaced-only tracks** (C1) — that is `flaw.obese`,
  the batch's only `health_mod` carrier, on the `fatigue_roll` track. Its
  amount is correct in sign and size and the convention was re-derived rather
  than inherited: `fatigue_roll` has exactly three shipped carriers,
  `flaw.obese` **-3**, `flaw.short_of_breath` **-3** and `virtue.long_winded`
  **+3**, so a Flaw-negative value matches both its siblings and ArMDE:6518's
  "-3 to all Fatigue rolls" ✓. That the track moves no number is the engine's
  gap, recorded once. What **is** counted against the entry is a different
  clause the effect never claimed to carry (**F-470**).
- **`AdvancementMod` is surfaced-only** (C1) — **no entry in this batch carries
  it**, which is worth the line because `flaw.master_of_none` (ArMDE:6420) looks
  like a carrier and is not one: its rule forbids *applying* experience to a
  target already advanced this year, which is not an amount on a source, so no
  `advancement_mod` could express it. That inexpressibility is D3 grounds and is
  **F-467**'s basis, not a Part C excuse.
- **`CombatMod` folds conditional amounts unconditionally** (A35) — the tempting
  reading of **F-462**, and the wrong one. `flaw.missing_eye`'s two figures are
  not one conditional amount; they are **two different amounts for two different
  weapon classes**, and `Effect::CombatMod`'s `weapon` field expresses exactly
  that, is already used by `flaw.lame` for `weapon.dodge`, and is pinned by
  `derived.rs::weapon_scoped_combat_mod_replaces_general_on_that_weapon_only`.
  Part C is therefore *not* the reason this entry fails, and saying it was would
  have buried a live wrong number under a known gap — B14's lesson on
  `flaw.low_tolerance`, applied.
- **`incompatible_with` is validated over bought selections only** (C4a, and
  documented as deliberate at `validation/prereq.rs::PrereqCtx::build`) — this
  touches **F-463**, **F-465** and **F-466**, and it **splits two ways, which my
  first draft got wrong and the verification pass corrected.**
  For **F-463** and **F-465** the scope fact is harmless: `virtue.the_gift` and
  `flaw.sleep_disorder` are reachable through no House, Mythic-type or
  `grants_selection` grant (checked across `houses.json`,
  `mythic_companion_types.json`, `childhoods.json`, `life_stages.json`,
  `character_types.json` and every `grants_selection` effect), so every copy a
  player can hold is bought and the proposed `incompatible_with` declarations
  fire.
  For **F-466** it is **not** harmless: `virtue.lone_redcap` carries
  `grants_selection: { items: ["virtue.well_traveled"] }` (sourced to ArMDE:4321,
  "receive the benefits of the Well-Traveled virtue"), so a Lone Redcap holds a
  **granted** Well-Traveled that `validate_incompatibilities` cannot see —
  `validation/mod.rs::validate` builds `selected_ids` from `entity.selections`
  alone and `validation/prereq.rs::validate_incompatibilities` iterates the same
  bought list, so the pair escapes on *both* sides. F-466's remedy is therefore
  `Prereq::Nor`, whose `PrereqCtx::present_ids` **is** grants-inclusive — the
  exact reach this bullet's first draft warned a correction pass away from. The
  warning stands for F-463 and F-465 and is withdrawn for F-466.
- **`creation_effect` is not required to carry effects** (C7) — inert in this
  batch today, since the span has **zero** `creation_effect` entries. Recorded
  because two of this batch's recommendations (**F-460**, **F-461**) move
  entries *into* that class, and C7 is why no guard will notice if the
  accompanying `ability_authorization` is forgotten when the class is changed.

Two further catalogue-wide gaps are named by clauses in this span and are
**recorded, not counted**: a Reputation's *negativity* is not representable
(B10's **F-380**) and `ReputationType` has only four members
(`local`/`ecclesiastical`/`hermetic`/`academic`), so `flaw.monstrous_blood`'s
"poor Reputation of level 3 **among other magic beings**" (ArMDE:6462) is
doubly inexpressible — wrong polarity *and* no such audience. That clause is
part of **F-461**'s D5 half, not a separate effect defect.

### Decisions applied

`docs/vf-audit/decisions.md` was read in full and is binding. It carried D1-D6
when this batch read it, **including D6's 2026-09-21 correction**, read in its
corrected two-rule form.

- **D3** is the load-bearing one again and governs six entries. On each,
  `narrative` is forbidden and the rule must be written into both locales:
  `flaw.magical_air` (no effect makes one character's social reactions those of
  a Gifted character; `A14`/`A25`/`A41` are all the wrong shape and
  `ability_roll_mod` is surfaced-only anyway),
  `flaw.magical_being_companion` (nothing models a companion creature's Might,
  `A26 MightGrant` being a grant to the *bearer*),
  `flaw.master_of_none` (no effect constrains which target advancement XP may be
  applied to),
  `flaw.motion_sickness` (no effect expresses fatigue lost on a journey; the five
  `HealthTrack` members are penalties and rolls, not losses),
  `flaw.no_hands` (the -5 is the no-gestures penalty, and the engine's
  gesture machinery is a *per-casting display choice*
  (`derived.rs::residual_gesture_penalty`), with no way to say "this character
  always pays it" — the identical inexpressibility `flaw.mute` faces, where the
  catalogue already answers `uncomputed_rule` with the rule written out), and
  `flaw.magical_fascination`'s **cap** half ("but no more" — no effect caps one
  named Ability at a fixed score; `A13 LocalityAbilityCapFraction` is a fraction
  of the age cap scoped to `locality_dependent` Abilities). Each entry's
  **permission** or **incompatibility** half is a different matter and *is*
  expressible, which is why F-460 recommends `creation_effect` with an effect
  added rather than `uncomputed_rule`, and why F-463/F-465/F-466 are data
  findings rather than classification ones.
- **D5** is the reason two entries that compute correctly still fail:
  `flaw.monstrous_blood` (**F-461** — the four Magic-being deformity blocks, the
  Characteristic decrease, the level-3 Reputation and the Int/Per -3, none of
  them anywhere in either locale) and `flaw.obese` (**F-470** — the -1 on rolls
  involving moving quickly or gracefully, in neither locale). It is also the
  reason two *other* computed entries pass: `flaw.missing_hand` and
  `flaw.missing_eye` each ship the whole passage as `description` in both
  locales although nothing obliges them to — which on `flaw.missing_eye` is what
  keeps its **text** column green while its **data** column fails.
- **D6** governs nothing here in the disputed sense: **no** table row in this
  batch disagrees with the rulebook about a German name, so there is no
  "this repository's copy disagrees with the rulebook" row to hand the
  orchestrator. What the tables do carry is four `Anmerkung` cells making
  **provenance** claims the core book contradicts, which is D6 rule 1's error
  shape — escalated as **Q-121** rather than counted, following B13's and B14's
  handling of the same shape.
- **D1 and D4** touch nothing here: no entry in this batch carries
  `lab_total_mod`, and none of D1/D4's nine is in the span.
- **D2** touches nothing here: no entry is a granted Virtue and none carries
  `characteristic_score_delta` or `characteristic_score_delta_param`.
  `flaw.monstrous_blood`'s "must decrease one of his Characteristics by 1, but
  not below -3" (ArMDE:6462) is the nearest thing and is **not** a D2 case: it is
  a *player-chosen* decrease of an unnamed Characteristic, which
  `characteristic_score_delta_param` cannot express (its `amount` is fixed by
  the item, and its precondition check is the opposite gate), so it is part of
  F-461's D5 half.

### Cross-references followed and rated

Every pointer in the span was followed and read, including on entries that
passed. **The inbound side was swept deliberately rather than opportunistically**:
each of the 32 English headings in the span was searched for by name across the
whole core book, outside its own range and outside the "List of Flaws" index,
and every hit was read. That returned **twenty-one** inbound sites — ArMDE:2782,
:2804, :3392, :3517, :3655, :4125, :4139, :4221, :4494, :4498, :5079, :5185,
:5717, :6148, :6608, :6749, :6895, :7009, :9906, :18641, :21130 — of which **six
add a rule or a defect** (F-463's third clause, F-465, F-474, F-475, F-476,
F-478) and fifteen are suggestions, restatements, illustrative mentions or
already-encoded siblings. Every one of the twenty-one is rated below. Sample
characters and creature templates that merely *list* a Flaw were excluded from
the sweep as noise — ArMDE:1310, :1382, :1461, :1539, :1716, :1766, :1817,
:1913, :18061, :18165, :18430, :18655, :18699, :20022, :20561, :20794 — after
checking that none of them states a rule. This is the first batch to run that sweep as a *method step*
rather than as a reaction to a pointer noticed in the text, and it is where more
than a third of the batch's findings came from.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the entry? |
|---|---|---|---|
| `flaw.magical_fascination` | "he is allowed to have **a score of 1 (but no more) in either Magic or Faerie Lore at character creation**" (ArMDE:6394) / DE "darf er bei der Charaktererschaffung einen **Wert von 1 (aber nicht mehr) in Magie- oder Feenkunde** haben" | `rules/core/abilities.json` — `ability.magic_lore` and `ability.faerie_lore` both carry `category: "arcane"`, and `arcane` is one of the three `categories_requiring_virtue` | **YES — this is F-460.** A permission gate the entry does not encode, so a grog, companion or mythic companion doing exactly what the sentence permits raises a hard `ability_category_requires_virtue` (`validation/authorization.rs::validate_ability_authorization`; magi exempt by `profile.is_magus`). F-409's 31st instance. The cap half is separately inexpressible (D3). |
| `flaw.monstrous_blood` | "and **may learn Magic Lore during character creation**" (ArMDE:6456) / DE "außerdem darf er **Magiekunde bereits bei der Charaktererschaffung erlernen**" | same file — `ability.magic_lore`, category `arcane` | **YES — this is F-461**, and it is the sharper of the two because the sentence exists *only* to grant the permission: it is not a side remark, it is the Flaw's one compensating benefit beside the aging bonus. F-409's 32nd instance. |
| `flaw.magical_air` | "People and animals react to you **as if you had The Gift**." (ArMDE:6384) / DE "Menschen und Tiere reagieren auf dich, **als hättest du die Gabe**." | ArMDE:8751, `## The Gift` — "she suffers the **-3 penalty to all rolls and totals based on social interaction**, including training" | **YES — a signed modifier imported by reference, and half of F-463.** The entry states a -3 without printing one, which is why the phrase screen cannot see it and why `narrative` survived two sweeps. Its own sibling `flaw.offensive_to_beings` spells the same -3 out for one class of being (ArMDE:6528) and is correctly `uncomputed_rule` with the rule in `description` in both locales — so the catalogue already contains the right answer for this shape, one entry away. |
| `flaw.magical_air` | "**You may not take this Flaw if you actually do have The Gift**; see The Blatant Gift (page 120) instead." (ArMDE:6384) / DE "**Du kannst diesen Fehler nicht wählen, wenn du tatsächlich die Gabe besitzt**; stattdessen siehe Auffällige Gabe" | `virtue.the_gift` (ArMDE:3967-3970); `flaw.blatant_gift` (the "instead" target) | **YES — the other half of F-463, and the one with a live consequence.** `flaw.magical_air`'s `incompatible_with` is `["flaw.offensive_to_beings"]` and `virtue.the_gift`'s is `["virtue.devil_child","virtue.faerie_doctor","virtue.failed_apprentice","virtue.nephilim","virtue.spirit_votary"]` — neither names the other. Since every magus must hold The Gift, **every magus in the app may take a Major Flaw the book forbids him outright**, and validation says nothing. |
| `flaw.magical_air` | *(inbound)* ArMDE:5717, `#### Blatant Magical Air` (*Major, Supernatural*, **B10**'s span) — "**Only characters with a Magical Air or The Gift may take this Flaw.** … This effect is the same as the Blatant Gift, and **a character may not have both Flaws**." | `flaw.blatant_magical_air`; `Prereq::Any([Has(virtue.the_gift), Has(flaw.magical_air)])`; `flaw.blatant_gift` | **YES — F-474, and it lands outside this span.** Two stated restrictions, neither encoded: `flaw.blatant_magical_air` carries **no `prerequisites` and an empty `incompatible_with`**, and `flaw.blatant_gift`'s `incompatible_with` is `["flaw.unbearable_to_beings","virtue.gentle_gift"]` — no reverse edge. The engine already writes this exact prerequisite twice, on `flaw.unbearable_to_beings` and `virtue.inoffensive_to_beings` (`RULES.md:921-923`), so the shape is not an audit invention. Invisible to any screen reading only ArMDE:6382-6385. |
| `flaw.magical_air` | "characters who are Offensive to more than one kind of being **should take Magical Air instead** … Characters with Magical Air may not take it at all." (ArMDE:6530) | `flaw.offensive_to_beings` | **No new rule — and the encoding is right.** The mutual `incompatible_with` is present on both entries ✓, `max_total: 1` on Offensive matches "You may not take this Flaw more than once" ✓, and the Gift/Gentle-Gift condition is `Any([Nor([Has(virtue.the_gift)]), Has(virtue.gentle_gift)])` ✓, all three recorded at `RULES.md:900-928`. What the pointer *does* surface is a text defect: the shipped German description renders Magical Air as **"Magisches Auftreten"** while the entry's own shipped German name is **"Magische Ausstrahlung"** (**F-473**). |
| `flaw.missing_eye` | "-3 on Attack rolls **for missiles** and targeting rolls for spells. **In melee combat** you suffer -1 on Attack rolls" (ArMDE:6436) / DE "-3 auf **Angriffswürfe mit Fernkampfwaffen** … Im **Nahkampf** erleidest du -1" | `derived/combat.rs::combat_totals`'s `cm` closure; `rules/core/equipment.json`'s seven missile weapons; `crates/arm-rules/RULES.md:5161` | **YES — F-462, the batch's wrong number.** The unscoped `combat_mod { -1, attack }` is added to every equipped weapon's Attack line, so the bow reads -1 rather than -3 and the -3 reaches nothing. RULES.md:5161 names "missing_eye's ranged −3" among the amounts that *are* folded, so the traceability map describes data the catalogue does not contain — an in-repo authority contradicting the shipped file. |
| `flaw.missing_eye` | "This Flaw **can be combined with Poor Eyesight, but the penalties are cumulative**." (ArMDE:6436), mirrored at ArMDE:6608 | `flaw.poor_eyesight` (ArMDE:6606-6609, **B16**'s span) — `combat_mod { -3, attack }` + `combat_mod { -3, defense }` | **No new rule, and the target checks out.** An explicit *compatibility*, so the absent `incompatible_with` on both is correct, and `A35` sums unscoped figures, so the two stack exactly as the sentence says ✓. Poor Eyesight's own "-3 to rolls involving sight, **including rolls to attack and defend**" matches its two effects ✓. The pair's arithmetic is right in melee (-1 + -3 = -4) and wrong at range (-4 where the book gives -6), which is F-462 restated, not a second defect. |
| `flaw.missing_hand` | "Climbing, **combat**, and other activities normally requiring both hands are at a penalty of **-3 or greater**" (ArMDE:6440) | `crates/arm-rules/RULES.md:5048-5057`; `data_integrity.rs::the_combat_roll_flaws_penalize_the_combat_ability_totals` | **No unrecorded rule.** RULES.md argues the encoding out in full — "combat" reaches both Combat-Ability totals, ArMDE:16656's standard configuration ("a weapon and a shield") needs both hands, and "-3 **or greater**" is taken at its floor because that is the only figure the source fixes. Corrected 2026-09-14 from Attack-only, and pinned. The whole passage also ships in `description` in both locales ✓. Entry passes. |
| `flaw.no_hands` | "magi with this Flaw take a **– 5 penalty to all Casting Scores**. This **may be offset by taking the Subtle Magic Virtue** (page 110) or the **Still Casting Spell Mastery** ability." (ArMDE:6498) | `virtue.subtle_magic` (ArMDE:5073-5076) = `special_casting_mod { subtle_gestures }` → `derived.rs::NO_GESTURE_PENALTY` (-5) and `::SUBTLE_MAGIC_GESTURE_REDUCTION` (+5) | **YES — F-464, and the target is what identifies the rule.** The -5 is the engine's own no-gestures constant and Subtle Magic is already the +5 that cancels it, so the book's sentence and `derived.rs::residual_gesture_penalty` are describing the same arithmetic — but that penalty is a *per-casting display option*, with no marker for a character who can never use hands, so the rule is inexpressible (D3) and belongs in `description`. `flaw.mute` is the exact mirror (ArMDE:6474's -10 = `NO_VOICE_PENALTY`, offset by Quiet Magic) and the catalogue gets **that** one right: `uncomputed_rule`, full passage in both locales. |
| `flaw.no_sense_of_direction` | "This Flaw is **incompatible with the Well Traveled Virtue**." (ArMDE:6502) / DE "Dieser Fehler ist **unvereinbar mit der Tugend Vielgereist**." | `virtue.well_traveled`, ArMDE:5239-5242 (**B09**'s span) | **YES — F-466.** Both `incompatible_with` sets are empty, so the pair validates clean today. The target was read in full and rated: ArMDE:5241 states the exclusion from **neither** side — it is a one-sided statement in the book — but `ruleset/integrity.rs::validate_incompatibility_symmetry` requires both edges, so the correction touches both entries. Nothing else about `virtue.well_traveled` is wrong: its fifty bonus experience points ship as `restricted_ability_xp` (out of scope here, checked only enough to rate this pointer). |
| `flaw.night_terrors` | "A character with this Flaw **may not also take the Sleep Disorder Flaw**" (ArMDE:6492) / DE "Ein Charakter mit diesem Fehler **darf nicht zusätzlich den Fehler Schlafstörung nehmen**." | `flaw.sleep_disorder`, ArMDE:6745-6750 (**B17**'s span) | **YES — F-465, stated from both sides and encoded on neither.** The target was read in full: ArMDE:6749 closes "This Flaw is **not compatible with the Night Terrors Flaw**", so the book says it twice and the catalogue says it nowhere — `incompatible_with` is absent on both ids. A doubly-sourced exclusion is the strongest kind this audit finds, and it is the only one so far that the book itself declares symmetrically. |
| `flaw.night_terrors` | "This Flaw normally makes seasonal Laboratory work impossible, and so is **not suitable for magi**." (ArMDE:6494) | `entity_kinds`; `character_types.json`'s `forbidden_traits` | **Undecided — escalated as Q-123**, together with `flaw.oath_of_fealty`'s ArMDE:6514. A soft restriction ("normally", "not suitable") on one character type, expressible as a profile `forbidden_traits` row but only as a hard block, which is not what either sentence says. Not counted against the entry; the clause ships in `description` in both locales ✓. |
| `flaw.magical_being_companion` | "The creature has a **Magic Might score of 10 – Size**." (ArMDE:6390) / DE "Das Geschöpf hat einen **Machtwert von 10 – Größe**." | `A26 MightGrant` (a grant to the *bearer*, not to a companion NPC); `Realms of Power: Magic`, named at ArMDE:6388 and **not** in `rules/source/en/` | **YES — F-469.** A closed formula for a figure the app does not model at all, stated in a `narrative` entry whose summary stops at sentence one. The *RoP:M* pointer cannot be followed and is not needed: the formula is stated in the core book, in this entry's own range. |
| `flaw.magical_being_companion` | *(multiplicity)* the entry declares a `being` free-text parameter and no `max_total`, against ArMDE:2814 | `validation/selections.rs::validate_duplicate_selections`; `max_per_target` default 1 keyed on `(item_ref, whole params map)` | **YES — F-469's second half.** Two copies naming different beings are two tuples and legal, so a character may hold Magical (Ferret) Companion *and* Magical (Crow) Companion. ArMDE:6386-6391 grants no repeat and the passage is singular throughout ("**an** intelligent magical being"), so ArMDE:2814 forbids it by silence. B10's **F-358** / B11's **F-400** / B14's **F-451** shape, in a new place. |
| `flaw.necessary_realm_aura_for_ability` | "A character may take this Flaw **once for any particular Ability**." (ArMDE:6482) / DE "Ein Charakter darf diesen Fehler **einmal für jede bestimmte Fertigkeit** nehmen." | `ParameterDef::max_per_value`; `validation/selections.rs::validate_per_value_cap` | **No defect — and this is the entry the rest of the batch should be measured against.** The `ability` parameter carries `max_per_value: 1`, which caps copies naming one and the same Ability at one while leaving different Abilities free, and `max_total` is correctly absent. That is precisely the sentence. Re-derived at the consumer rather than taken from the field name. |
| `flaw.necessary_realm_aura_for_ability` | "reducing the number of **Labor Points (as described in City & Guild)** he accumulates each season" (ArMDE:6484) | **Nowhere — *City & Guild* is not in `rules/source/en/`.** | **No rule this repository may implement.** The pointer cannot be followed, and unlike B14's `flaw.independent_craftsman` the sentence states no fallback, so nothing recategorizes and nothing is owed beyond the text — which ships in full in `description` in both locales ✓. Recorded so it is not re-chased. |
| `flaw.monstrous_blood` | "*Magic Human:* … The character also has a **poor Reputation of level 3 among other magic beings**." (ArMDE:6462) | `A25 GrantsReputation`; `types.rs::ReputationType` (`local`/`ecclesiastical`/`hermetic`/`academic`); B10's **F-380** | **No new *effect* obligation, and the reason is doubly structural.** `Reputation::score` is a `u8` with no polarity (F-380, recorded once), and there is no "magic beings" audience among the four `ReputationType` members — so the clause is inexpressible twice over and is owed as `description` text (D5), which is **F-461**'s second half. |
| `flaw.monstrous_blood` | "receives a **-1 bonus to all Aging rolls**" (ArMDE:6456) | `aging.rs::aging_total` via `A38 AgingMod { aging_roll }` | **No new rule, and the sign was re-derived rather than assumed.** A38: an `aging_roll` amount is **added** to the AGING TOTAL with its stored sign, and a lower total is a better outcome — so a stored **-1** is a benefit, which is what "bonus" means here. The catalogue's five other `aging_roll` carriers with a non-zero amount are all **Virtues** at -1 or -3 (`faerie_blood`, `magical_blood`, `magian_lineage_major`, `magian_lineage_minor`, `strong_faerie_blood`), so this Flaw carrying a negative is correct rather than inverted ✓. |
| `flaw.mute` | "magi with this Flaw get a **-10 penalty to all spellcasting**, although this **may be offset by taking the Quiet Magic Virtue** (page 105)" (ArMDE:6474) | `virtue.quiet_magic` = `special_casting_mod { quiet_words }`; `derived.rs::NO_VOICE_PENALTY` (-10), `::QUIET_MAGIC_VOICE_REDUCTION` (+5) | **No unrecorded rule — and this entry is the control case for F-464.** The -10 is the engine's no-voice constant and Quiet Magic is its +5 offset, but the residual is a per-casting figure with no always-on marker, so the rule is inexpressible; the entry answers `uncomputed_rule` and ships the whole passage, -10 included, in both locales ✓. `casting_total_mod { -10, all }` was considered and rejected: Quiet Magic feeds `residual_voice_penalty`, a different pipeline, so the offset the sentence promises would not apply and the figure would be wrong in the other direction. |
| `flaw.magical_air` / `flaw.offensive_to_beings` | "see **The Blatant Gift (page 120)** instead" (ArMDE:6384) | `flaw.blatant_gift` (`uncomputed_rule`, `incompatible_with` = `["flaw.unbearable_to_beings","virtue.gentle_gift"]`) | **No — a redirection, not a rule about this entry.** Followed and read; it tells a Gifted player which Flaw to take instead and attributes nothing to Magical Air. Rated and discarded. |
| `flaw.lycanthrope` | "You have the normal **physical characteristics of a shapeshifter (see page 107)**, except that **+3 is added to your Soak score** (in animal form only)" (ArMDE:6376) | the shapeshifter rules; `derived/combat.rs::soak` | **No new rule the entry can be held to, and the whole clause ships.** The +3 is scoped to a form the app does not model (`A34 SoakMod` would apply it always, which is the opposite of "in animal form only"), so it is inexpressible (D3) and is correctly carried as text — the full three-paragraph passage is in `description` in **both** locales, Size range, +3 Soak and full healing included ✓. The entry's only defect is its truncated German *summary* (**F-471**). |
| `flaw.master_of_none` | "Experience points gained through other means, such as **Secondary Insight**, are unaffected" (ArMDE:6420) | `virtue.secondary_insight` (ArMDE:4892-4895), an `advancement_mod` carrier | **No new rule attributed to this entry**, and the target confirms the reading: Secondary Insight is a real catalogue entry whose own mechanic is an advancement modifier, so the carve-out names a live interaction rather than flavour. It is part of the rule **F-467** says must be written out, not a separate finding. |
| `flaw.monastic_vows_hermetic` | "you **cannot own vis** and must only possess **functional magic devices**. You cannot marry, and many magi might interpret that as **prohibiting binding a familiar**." (ArMDE:6452) | `types.rs::Entity` — **no vis field exists**; `Entity::devices` + `A19 ItemLevelBudget`; `Entity::familiar` (`Option<Familiar>`), which **is** modelled | **Undecided — escalated as Q-118.** The vis prohibition has no field to attach to, the devices clause constrains a collection the engine does hold, and the familiar clause is hedged twice ("many magi **might** interpret"). The whole first clause reaches the user, since it is inside sentence one and therefore inside the shipped summary in both locales. Not counted against the entry. |
| `flaw.motion_sickness` | "you suffer **double the fatigue loss** on long journeys, with a **minimum loss of two Fatigue levels**" (ArMDE:6470) | the five `HealthTrack` members (`fatigue_penalty`, `wound_penalty`, `fatigue_roll`, `casting_fatigue`, `recovery`) | **YES — F-468.** A multiplier and a floor, on a quantity — Fatigue levels lost to travel — that no `HealthTrack` member names: the two computed tracks move a tier's *penalty*, not a *loss*. Inexpressible (D3), stated in neither locale, and classified `narrative`. |
| `flaw.magical_air` | *(inbound)* ArMDE:3392, `#### Alluring to (Beings)` (*Minor, General*, **B01**'s span) — "Characters who are Offensive to beings of this sort **cannot take this Virtue, including those who have The Gift or Magical Air**, though characters who are Inoffensive to them or have the Gentle Gift may." | `virtue.alluring_to_beings`; `Prereq::Any`/`Nor` over `virtue.the_gift`, `flaw.magical_air`, `flaw.offensive_to_beings`, `virtue.gentle_gift`, `virtue.inoffensive_to_beings` | **YES — F-475, and it lands outside this span.** The entry carries **no `prerequisites` and an empty `incompatible_with`**, although it is the fourth member of the Beings family and the other three all encode the identical sentence (`RULES.md:921-923`). The fix is a prerequisite on the B01 entry, so no edge lands on `flaw.magical_air` — but only a reader of `flaw.magical_air` would ever have gone looking. |
| `flaw.magical_air` | *(inbound)* ArMDE:21130 — "Specific Flaws (such as Offensive to Divine Beings and Unbearable to Divine Beings) that modify or simulate the effects of The Gift may cause creatures with Divine Might to react negatively to the character with the Flaw, **although Magical Air does not**." | the Divine-Might chapter; this entry and `flaw.offensive_to_beings` | **YES — a named carve-out, and F-463's third clause.** It matters because the entry's own text says people react "as if you had The Gift"; this sentence is where that analogy stops. Owed in `description` under D5. The same sentence's first half belongs to `flaw.offensive_to_beings` for its `being.divine` value and is noted there rather than counted, being value-scoped in a way the entry cannot express. |
| `flaw.magical_air` | *(inbound, ×2)* ArMDE:4139, `virtue.inoffensive_to_beings` — "**UnGifted characters may take this Virtue only if they have the Flaw Magical Air.**"; and ArMDE:6895, `flaw.unbearable_to_beings` — "She suffers an **additional –3 penalty** on all social interactions with them, which **adds to the –3 penalty normally associated with The Gift**. **Only characters with The Gift or Magical Air may take this Flaw**, and it cannot be combined with the Blatant Gift." | `virtue.inoffensive_to_beings` and `flaw.unbearable_to_beings`, both carrying `prerequisites: Any([Has(virtue.the_gift), Has(flaw.magical_air)])` (`RULES.md:921-923`) | **No new rule, and two useful confirmations.** Both eligibility gates naming Magical Air **are** encoded ✓, which is what makes `flaw.blatant_magical_air`'s bare data (F-474) and `virtue.alluring_to_beings`' bare data (F-475) omissions rather than capability gaps. ArMDE:6895 also corroborates F-463's classification half from a third site: it says the Gift's penalty "normally associated" is **-3**, and treats a Magical Air holder as subject to the same family of effects. |
| `flaw.magical_air` | *(inbound)* ArMDE:5185, `virtue.unaffected_by_the_gift` — "The character is **not affected by the negative effects of The Gift or Magical Air** in others." | `virtue.unaffected_by_the_gift` (`narrative`, ArMDE:5183-5186, **B09**'s span) | **No rule attributed to this entry — but it is corroboration for F-463's classification half.** The immunity belongs to the Virtue, not to the Flaw. What it establishes independently is that the book treats Magical Air as having "**negative effects**" of the same kind as The Gift's — which is the -3 at ArMDE:8751 — so `narrative` on ArMDE:6384 is contradicted by a third site. Followed, rated, and not counted against the Virtue, whose own passage states only this immunity and nothing computable. |
| `flaw.lycanthrope` | *(inbound)* ArMDE:9906, the blockquoted sidebar `#### Ringing the Changes` — "**An exception is the lycanthrope, who does not retain his human mind when transformed, and therefore cannot be affected by Mentem spells**, although Corpus spells still work." | the shapechanger rules; `derived/casting.rs::magic_resistance` (per-Form resistance, not immunity) | **YES — F-478.** A Flaw-specific carve-out out of a rule the sidebar has just stated for the whole shapechanger class, naming the Flaw. Same construction B14 **confirmed** at ArMDE:7397 for `flaw.lesser_malediction`, and **not** the construction B14 narrowed F-453 on. Inexpressible (no effect makes a Form ineffective against the bearer) and therefore owed as `description` text in both locales — where it is absent. |
| `flaw.lycanthrope` | *(inbound)* ArMDE:18641 — "**Lycanthropes are an exception to this** — when transforming back into human form, all wounds taken while an animal are healed, although wounds suffered as a human remain." | the shapechanger wound rules | **No — a restatement, and it is already covered.** This is ArMDE:6376's own healing clause said twice, and ArMDE:6376 ships in the entry's `description` in both locales ✓. Followed and discarded, and recorded because it is the near-miss that keeps F-478 honest: not every inbound mention of a Flaw adds a rule. |
| `flaw.lycanthrope` | *(inbound)* ArMDE:3517, `virtue.blood_of_the_nephilim` (**B01**'s span) — "You may not take … **Flaws such as Age Quickly or Lycanthrope**, or Virtues or Flaws that affect your Size." | `virtue.blood_of_the_nephilim`; `incompatible_with` | **YES — F-476.** An exclusion naming this Flaw, with `incompatible_with` empty on both sides. Reading the target in full also turned up **F-409's thirty-third instance** in the same entry (ArMDE:3511's Dominion Lore) and four uncomputed aging clauses **B01** should re-examine — which is exactly why the brief asks that a pointer's target be *read*, not merely resolved. |
| `flaw.manufactured_ignorance` | *(inbound)* ArMDE:3655, `virtue.demonic_blood` (**B02**'s span) — "she **probably** has either the Delusion or Manufactured Ignorance Flaw to explain her remarkable capabilities" | `virtue.demonic_blood` | **No for this entry, YES for the target — F-477.** The mention is a suggestion and attributes nothing to `flaw.manufactured_ignorance`, whose `narrative` classification and empty data stand ✓. But the same paragraph carries "may learn **Infernal Lore** during character creation **without needing to take the Arcane Lore Minor Virtue**", unencoded on an entry that already has two effects — F-409's thirty-fourth instance. |
| `flaw.manufactured_ignorance` | *(inbound)* ArMDE:4498, `virtue.mentored_by_demons` — "Characters trained to this extreme know that their teachers are supernatural figures, but **often have the Manufactured Ignorance Flaw**." | `virtue.mentored_by_demons` (a `restricted_ability_xp` carrier) | **No.** "Often have" is a suggestion, exactly as ArMDE:3655's "probably has". Followed and discarded. |
| `flaw.necessary_condition` | *(inbound)* ArMDE:7009, `flaw.vulnerable_magic` (**B19**'s span) — "It may not be combined with Restrictions or **Necessary Conditions that have the same (or equivalent) conditions**." | `flaw.vulnerable_magic` (`narrative`, `incompatible_with` empty) | **No flat rule, and the hedge is the reason.** The exclusion is scoped to copies whose *condition* matches, and both entries record their condition as free text (or not at all — `flaw.necessary_condition` has no parameter), so `incompatible_with` would over-exclude and no mechanism narrows it. Genuinely inexpressible, which ties it to **Q-119**. Followed, rated, not counted. |
| `flaw.noncombatant` | *(inbound)* ArMDE:4125, `virtue.ineslemen` (*Minor, Social Status*) — "The character **begins with the Minor Flaw Noncombatant, which does not yield any points for buying Virtues**." | `virtue.ineslemen`'s `effects`; `A18 GrantsSelection` (budget-exempt); `validation/balance.rs::compute_balance`, which iterates **bought** selections only | **No defect — and it is the batch's one fully-correct inbound encoding.** `virtue.ineslemen` carries `grants_selection: { items: ["flaw.noncombatant"] }`, and because `compute_balance` and `validate_caps` read `entity.selections` (bought) while the effect folds land on the grant list, the granted copy is exactly "does not yield any points" ✓. It also carries a `restricted_ability_xp` naming six Lores including three gated ones, which authorizes them ✓ — the correct shape of the mechanism F-460/F-461/F-476/F-477 are missing, encoded on one entry, for reference. `flaw.noncombatant`'s own empty data is right: the grant belongs to the granting Virtue. |
| `flaw.mute` | *(inbound)* ArMDE:2782 — "Some Flaws, **such as Blind or Mute**, could be fixed using Hermetic magic. A character with easy access to such magic **can only take such a Flaw if there is some reason why it cannot be fixed**, such as that it is part of a character's Essential Nature." | the V/F rules chapter; no `Effect` or `Prereq` shape for "a reason why it cannot be fixed" | **No rule attributed to this entry, and the reasoning is B14's F-453 narrowing applied.** "Such as Blind or Mute" **illustrates** a general rule about all curable Flaws; it does not state a Mute-specific one, unlike ArMDE:9906's "An exception is **the lycanthrope**" (F-478), which names one Flaw and carves it out. The general rule is also a storyguide judgement with no engine hook. Followed, rated, and not counted — the distinction is the same one B14 drew between ArMDE:7397's two clauses. |
| `flaw.mentor` | *(inbound)* ArMDE:5079, `virtue.sufi` — "you should choose an appropriate Minor Story Flaw, **such as Mentor**, which does not yield any points for buying Virtues." | `virtue.sufi` (the catalogue's one `taken_as` carrier, `ArMDE:5083`) | **No.** "Such as" again, and unlike ArMDE:4125's Ineslemen this is a *choice* the player makes rather than a grant, so there is nothing for `grants_selection` to name and the budget exemption cannot be encoded on the Virtue either. A real gap in expressiveness, but it belongs to `virtue.sufi` (ArMDE:5077-5084, **B08**'s span) and not to `flaw.mentor`, whose own `narrative` classification and empty data stand ✓. Recorded so it is not re-chased. |
| `flaw.necessary_condition` | *(inbound)* ArMDE:6148, `flaw.flawed_powers` (**B13**'s span) — "She suffers the effects of a Major Hermetic Flaw (**commonly Restriction or Necessary Condition**)" | `flaw.flawed_powers` (`uncomputed_rule`) | **No.** "Commonly" is illustrative and attributes nothing. Followed and discarded. |
| `flaw.oath_of_fealty` | *(inbound)* ArMDE:4221, `virtue.landed_noble` — "You have sworn an Oath of Fealty, and so it would be **reasonable** to balance this Virtue with that Flaw. **You get the normal points for Oath of Fealty if you do.**" | `virtue.landed_noble`; `validation/balance.rs::compute_balance` | **No new rule — and the second sentence is worth recording as a non-finding.** "You get the normal points" is the *default* behaviour: a bought Flaw always scores its magnitude's points. The sentence exists to pre-empt a reader who might think the Virtue's implied oath makes the Flaw free, so the data's silence is correct ✓. |
| `flaw.monastic_vows` | *(inbound)* ArMDE:4494 and ArMDE:2804 — "vows of poverty, chastity, and obedience, which **could together constitute a Major Story Flaw (Monastic Vows, see page 138)**, and which would be a natural choice if you take this Virtue"; "You might take Monastic Vows as your Major Story Flaw" | `virtue.mendicant_friar` (ArMDE:4488-4495, `narrative`); the worked example at ArMDE:2804 | **No.** Both are suggestions about character concept. Followed and discarded. |

### ArMDE:2814, :2816, :2818, :2820 — instances in this batch

ArMDE:2812-2820 was re-read in full for this batch rather than taken from B10,
B12, B13 or B14.

- **ArMDE:2814** ("A Virtue or Flaw may be taken more than once only if the
  description explicitly allows it. Most Virtues and Flaws may only be taken
  once.") — **two** explicit permissions in the span and the data gets **both**
  right, plus one silent entry that the data gets wrong.
  `flaw.necessary_realm_aura_for_ability` ArMDE:6482 ("once for any particular
  Ability") → an `ability` parameter carrying `max_per_value: 1`, so copies
  naming different Abilities are legal and a second copy naming the same one is
  `too_many_for_param_value` ✓ — exactly the sentence, and the first entry in
  five batches to use `max_per_value` for this.
  `flaw.offensive_to_beings` ArMDE:6530 ("You may not take this Flaw more than
  once") → `max_total: 1`, keyed on `item_ref` alone, so no second copy of any
  being class ✓.
  Against those, **one entry is parameterized where the book grants no repeat at
  all**: `flaw.magical_being_companion` carries a `being` free-text parameter and
  no `max_total`, so two differently-named companions are two tuples and legal
  (**F-469**). Stated no wider than checked: I read the passages of the three
  parameterized entries in my span only.
  **The other 32 entries are unparameterized with no `max_total`**, so
  `max_per_target`'s default of 1 keyed on `(item_ref, {})` already forbids a
  second copy, which is what the book requires — with no exception in this span.
- **ArMDE:2816** ("All characters must take one Social Status…") — **zero
  instances**: no entry in this batch carries the `social_status` category. B12's
  **F-427** is therefore not re-derived here and gains no instance; the first
  Social Status Flaws of the block start at ArMDE:6538 (`flaw.outcast`), one
  line past the span, in **B16**.
- **ArMDE:2818** ("A character should not have more than one Story Flaw") —
  **seven** instances, the densest Story span the audit has hit and nearly
  double B14's four: `flaw.magical_being_companion`, `flaw.manufactured_ignorance`,
  `flaw.many_marriageable_daughters`, `flaw.mentor`, `flaw.mistaken_identity`,
  `flaw.monastic_vows`, `flaw.oath_of_fealty`. Enforced on all four profiles,
  re-derived rather than inherited: every profile carries
  `{"category":"story","max":1}` (grog `max: 0`) ✓.
- **ArMDE:2820** ("A character may not have more than one Major Personality Flaw
  … should normally not have more than two Personality Flaws in total") — three
  Major/Minor pairs here (`meddler`, `obsessed`, `optimistic`), each with a
  mutual `incompatible_with` ✓, additionally load-enforced by
  `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`. Both halves are
  live and re-derived: every profile carries
  `{"category":"personality","max":1,"major_only":true,"hard":true}` and
  `{"category":"personality","max":2}` (grog 0 and 1) ✓. Two Minor singles join
  them (`magical_fascination`, `noncombatant`).
- **ArMDE:2960-2962** (Supernatural realm association) — **four** instances:
  `flaw.lycanthrope`, `flaw.manifest_sin`, `flaw.monstrous_blood`,
  `flaw.necessary_realm_aura_for_ability`. Only the last carries a `realm`
  parameter. Noted only; it is B11's standing observation, not a new one.
  `flaw.manifest_sin` is the interesting case — ArMDE:6406 names **two** realms
  ("either the Divine or the Infernal"), which a single `realm` parameter could
  record and `tainted` could not, since `tainted` is a boolean and only one of
  the two readings is Infernal.

## Verdicts

35 rows, one per entry. `class` / `data` / `text` report checks 2, 3-11 and 12.
"OK" means every check in that column passed; `?` means escalated, not resolved.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.lycanthrope` | 6370-6377 | uncomputed_rule ✓ | OK — the Size range, the +3 Soak and the healing are all inexpressible (D3) | **the DE summary is cut mid-sentence at "(z.B." with no ellipsis**; and **ArMDE:9906's Mentem immunity, attributed to the Flaw by name from outside its range, is in neither locale** | **F-471**, **F-478**; ArMDE:2960 instance |
| `flaw.magic_addiction` | 6378-6381 | uncomputed_rule ✓ | OK | OK ✓ — Ease Factor, the +3 and the botch clause in both locales, ASCII ✓ | **clean** |
| `flaw.magical_air` | 6382-6385 | narrative → uncomputed_rule | **`incompatible_with` omits `virtue.the_gift`, so every magus may take a Flaw ArMDE:6384 forbids him**; the imported -3 is inexpressible (D3) | the prohibition and the imported -3 in neither locale | **F-463**; **F-474** (inbound) |
| `flaw.magical_being_companion` | 6386-6391 | narrative → uncomputed_rule | **parameterized with no `max_total`, so two magical companions are legal** (ArMDE:2814) | the Magic Might formula in neither locale | **F-469**; ArMDE:2818 instance |
| `flaw.magical_fascination` | 6392-6395 | narrative → creation_effect | **no `ability_authorization` for `ability.magic_lore` / `ability.faerie_lore` (both `arcane`): buying the score the Flaw grants is a hard validator error today**; the "but no more" cap is inexpressible (D3) | the permission and the cap in neither locale | **F-460**; **Q-122**; ArMDE:2820 instance |
| `flaw.manifest_sin` | 6396-6407 | uncomputed_rule ✓ | OK — `tainted: false` follows the descriptor ✓ | OK ✓ — all five paragraphs in both locales, the -1 social penalty and the Faith Point clause included, ASCII ✓ | **clean**; ArMDE:2960 instance |
| `flaw.manufactured_ignorance` | 6408-6413 | narrative ✓ | OK — `tainted: true` ✓ | OK | **clean**; ArMDE:2818 instance |
| `flaw.many_marriageable_daughters` | 6414-6417 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.master_of_none` | 6418-6421 | narrative → uncomputed_rule | the advancement restriction is inexpressible (D3); no `advancement_mod` shape fits | the whole rule in neither locale; **and DE 6420 carries a gender error that would ship with the fix** | **F-467** |
| `flaw.meddler_major` | 6422-6425 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.meddler_minor` | 6422-6425 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.mentor` | 6426-6429 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.missing_ear` | 6430-6433 | uncomputed_rule ✓ | OK | OK ✓ — the -3 is the summary itself in both locales, ASCII ✓ | **clean**; the one `uncomputed_rule` entry with no `description`, correctly |
| `flaw.missing_eye` | 6434-6437 | in_play_effect ✓ | **the unscoped -1 reaches every weapon, so a bow reads -1 where ArMDE:6436 says -3, and the -3 is encoded nowhere**; the spell-targeting clause is inexpressible | OK ✓ — full passage both locales, ASCII ✓ | **F-462** |
| `flaw.missing_hand` | 6438-6441 | in_play_effect ✓ | `combat_mod {-3, attack}` + `{-3, defense}` argued out and pinned (`RULES.md:5048`) ✓ | OK ✓ — full passage both locales | **clean** |
| `flaw.mistaken_identity` | 6442-6445 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.monastic_vows` | 6446-6449 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.monastic_vows_hermetic` | 6450-6453 | narrative **?** | the vis prohibition has no field; the devices clause constrains `Entity::devices`; the familiar clause is hedged **?** | the whole first clause is inside sentence one, so it ships in both locales ✓ | **Q-118** |
| `flaw.monstrous_blood` | 6454-6467 | in_play_effect → creation_effect | `aging_mod {aging_roll, -1}` correct in sign ✓ — but **no `ability_authorization` for `ability.magic_lore` (`arcane`): learning it is a hard validator error today** | **the four Magic-being deformity blocks reach the user in neither locale** | **F-461**; Q-120 **closed** — `creation_effect` by the enum's own doc; ArMDE:2960 instance; F-380 carrier, not counted |
| `flaw.motion_sickness` | 6468-6471 | narrative → uncomputed_rule | the doubled loss and the two-level floor are inexpressible (D3) | the whole rule in neither locale | **F-468** |
| `flaw.mute` | 6472-6475 | uncomputed_rule ✓ | OK — the -10 is `NO_VOICE_PENALTY` but has no always-on marker (D3) | OK ✓ — full passage both locales, the -10 included, ASCII ✓ | **clean**; the control case for F-464 |
| `flaw.necessary_condition` | 6476-6479 | narrative **?** | "If you cannot perform the action, you cannot cast spells at all" is an absolute under a fiction condition **?** | the absolute is not in sentence one, so it ships in neither locale | **Q-119** |
| `flaw.necessary_realm_aura_for_ability` | 6480-6487 | uncomputed_rule ✓ | `max_per_value: 1` on the `ability` param is exactly ArMDE:6482 ✓ | OK ✓ — all three paragraphs in both locales, ASCII ✓ | **clean**; ArMDE:2814 and :2960 instances |
| `flaw.night_terrors` | 6488-6495 | uncomputed_rule ✓ | **the Sleep Disorder exclusion, stated from both sides, is encoded on neither entry** | OK ✓ — all three paragraphs in both locales, Ease Factor 9 included, ASCII ✓ | **F-465**; **Q-123** |
| `flaw.no_hands` | 6496-6499 | narrative → uncomputed_rule | the -5 is `NO_GESTURE_PENALTY` and has no always-on marker (D3) | **the -5 reaches the user in neither locale**; ArMDE:6498's `– 5` (sign + space) is what the screen missed | **F-464** |
| `flaw.no_sense_of_direction` | 6500-6503 | narrative ✓ | **the stated Well-Traveled exclusion is encoded on neither entry** | the exclusion in neither locale | **F-466** |
| `flaw.nocturnal` | 6504-6507 | uncomputed_rule ✓ | OK | OK ✓ — the -1 dawn-to-midday in both locales, ASCII ✓ | **clean** |
| `flaw.noncombatant` | 6508-6511 | narrative ✓ | OK | OK | **clean**; ArMDE:2820 instance |
| `flaw.oath_of_fealty` | 6512-6515 | narrative **?** | "Magi are forbidden … by the Hermetic Code. Some don't let that stop them." — a restriction the same sentence permits breaking **?** | the Code clause is not in sentence one, so it ships in neither locale | **Q-123**; ArMDE:2818 instance |
| `flaw.obese` | 6516-6519 | in_play_effect ✓ | `health_mod {fatigue_roll, -3}` correct in sign and size ✓ (surfaced-only, Part C, not counted) | **the -1 on rolls involving moving quickly or gracefully in neither locale** | **F-470** |
| `flaw.obsessed_major` | 6520-6523 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.obsessed_minor` | 6520-6523 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.offensive_to_beings` | 6524-6533 | uncomputed_rule ✓ | OK — `["general"]` + `index_categories: ["hermetic"]`, `max_total: 1`, the Gift/Gentle-Gift prereq and the Magical Air incompatibility are all correct and recorded (`RULES.md:386-490`, `:900-996`) ✓ | **the shipped DE description names Magical Air "Magisches Auftreten" while the entry's own shipped DE name is "Magische Ausstrahlung"** | **F-473**; check-6 finding drafted and **withdrawn** on RULES.md |
| `flaw.optimistic_major` | 6534-6537 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.optimistic_minor` | 6534-6537 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |

## Findings

### F-460 — `flaw.magical_fascination` is `narrative` on a permission the app hard-errors on, and on a cap it expresses nowhere

> "To represent the scraps of knowledge he has, **he is allowed to have a score
> of 1 (but no more) in either Magic or Faerie Lore at character creation**. But
> most of the time, he is filling in the blanks himself and making it up as he
> goes along, enthusiastically."
> — ArMDE:6394

> "Um die Bruchstücke seines Wissens darzustellen, **darf er bei der
> Charaktererschaffung einen Wert von 1 (aber nicht mehr) in Magie- oder
> Feenkunde haben**. Aber meistens füllt er die Lücken selbst und improvisiert
> enthusiastisch."
> — DE 6394

**Current data.** `classification: "narrative"`, no `effects`, no `parameters`,
no `prerequisites`. `rules/i18n/en|de/virtues_flaws.json` carry `name` +
`summary` only, and the summary stops at the first sentence — *"This character
is intensely interested in supernatural and magical phenomena, to the exclusion
of all other motivations."* — so the only thing that reaches the user is the
personality colour, with the Flaw's one compensating benefit removed.

**Why it is wrong, in two independent ways.**

1. **The permission is a live false hard error.** `rules/core/abilities.json`
   gives **both** named Abilities the category `arcane` — `ability.magic_lore`
   and `ability.faerie_lore` — and `categories_requiring_virtue` is
   `["academic","arcane","martial"]`.
   `validation/authorization.rs::validate_ability_authorization` errors with
   `ability_category_requires_virtue` on any held Ability in a gated category
   unless its id or its category appears in the union of `AbilityAuthorization`
   and `RestrictedAbilityXp` lists, with a whole-character exemption **only**
   for a profile whose `is_magus` is true. This Flaw is *Minor, Personality*,
   and `personality` is on the permitted list of **all four** profiles, so a
   grog, a companion and a mythic companion may all take it and none of them is
   exempt. A player who takes the Flaw and does exactly what its sentence
   permits — buy Magic Lore 1, or Faerie Lore 1 — is told his character is
   illegal.
   This is F-409's mechanism, **thirty-first** stated instance against **2**
   encoded (`flaw.covenant_upbringing`, `virtue.student_of_realm`, both
   re-derived here) plus 10 implicit via `restricted_ability_xp` on a gated
   category.
2. **The cap is a stated rule that reaches the user nowhere.** "(but no more)"
   is a hard ceiling of 1 on a named Ability. No `Effect` variant expresses it:
   `A9 AbilityScoreGrant` grants a floor rather than capping, and
   `A13 LocalityAbilityCapFraction` is a fraction of the *age* cap scoped to the
   catalogue's `locality_dependent` Abilities. That inexpressibility is grounds
   for the rule being written into `description` (D3/D5), never for `narrative`.

**Correct value.** `classification: "creation_effect"`, with an
`ability_authorization` effect — see **Q-122** for whether the `abilities` list
should name both ids (simple, but more permissive than "either … or") or the
either/or should be recorded as a player-chosen parameter. The cap sentence must
be written into `description` in **both** locales either way. Note C7:
`creation_effect` is not required to carry effects, so nothing will fail if the
effect is forgotten when the class is changed.

**The exact wording the phrase screen missed**, for the screen-fix list:

- EN: `he is allowed to have a score of 1 (but no more) in either Magic or Faerie Lore at character creation`
- DE: `darf er bei der Charaktererschaffung einen Wert von 1 (aber nicht mehr) in Magie- oder Feenkunde haben`

`states_a_mechanical_rule` is false here: `has_signed_number` sees no sign on
"1", there is no botch term, and `MECHANICAL_PHRASES` carries `"no more than"` /
`"nicht mehr als"` but **not** the bare `"no more"` / `"nicht mehr"` this passage
uses. The entry sits inside `SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` block, swept
2026-09-15 and re-swept 2026-09-19, green both times. **This is the identical
miss B14 recorded on `flaw.imagined_folk_tradition_vulnerability` (F-445)**,
whose ArMDE:6282 says "purchase a score of 1 (but no more) in Faerie Lore" —
two entries, 112 lines apart, using the same idiom for the same mechanic, and
the screen blind to both. The **permission** idiom B12 named ("is allowed to
have", "allows the character to purchase", "darf … haben", "erlaubt … zu
kaufen") would have caught both, and so would a `"(but no more"` /
`"(aber nicht mehr"` literal.

**Severity: HIGH.** A character the book describes and the app refuses to build,
plus a lost rule. Per `CLAUDE.md` this is product-integrity, not cosmetic.

### F-461 — `flaw.monstrous_blood` computes its aging bonus, hard-errors on its other benefit, and drops four mechanical blocks in both locales

> "The character is more resistant to age and receives a -1 bonus to all Aging
> rolls, and **may learn Magic Lore during character creation**."
> — ArMDE:6456

> "*Magic Animal:* … These are not usable by the character, and give a **–3
> penalty** to appropriate activities (such as running or climbing) …
> *Magic Human:* The character **must decrease one of his Characteristics by 1,
> but not below –3** … The character also has a **poor Reputation of level 3
> among other magic beings**.
> *Magic Spirit:* … **Subtract 3 from all of the character's Intelligence and
> Perception rolls** that do not involve a Supernatural Ability or Art.
> *Magic Thing:* The character has a **constant Lesser Power** … the character
> can **spend a Confidence Point** to prevent it from triggering."
> — ArMDE:6460, :6462, :6464, :6466

> "Der Charakter ist widerstandsfähiger gegen das Altern und erhält einen –1
> Bonus auf alle Alterungswürfe; **außerdem darf er Magiekunde bereits bei der
> Charaktererschaffung erlernen**."
> — DE 6456

**Current data.** `classification: "in_play_effect"`,
`effects: [{ "type": "aging_mod", "kind": "aging_roll", "amount": -1 }]`, no
`parameters`, no `prerequisites`. Both locales carry `name` + `summary` only,
and the summary is sentence one — which stops *before* the aging bonus and the
Magic Lore permission. So of a fourteen-line passage stating six separate
mechanics, exactly one reaches the user, as an unlabelled number in a derived
read-out.

**The aging effect is correct, and the sign was re-derived rather than
assumed.** Per A38 an `aging_roll` amount is **added** to the AGING TOTAL with
its stored sign, and a *lower* total is the better outcome, so a stored `-1` is
the benefit ArMDE:6456 calls a "bonus" ✓. The convention is confirmed by the
neighbours: the catalogue's five other non-zero `aging_roll` carriers are all
**Virtues** (`virtue.faerie_blood` -1, `virtue.magical_blood` -1,
`virtue.magian_lineage_major` -1, `virtue.magian_lineage_minor` -1,
`virtue.strong_faerie_blood` -3), so a Flaw carrying a negative here is right
rather than inverted. `RULES.md:5067` and `:5139` both name this entry and its
-1 explicitly.

**Why it fails anyway, in two ways.**

1. **The Magic Lore permission is a live false hard error**, by the identical
   chain as F-460: `ability.magic_lore` is `arcane`, `arcane` is in
   `categories_requiring_virtue`, the entry carries neither
   `ability_authorization` nor `restricted_ability_xp`, and the Flaw is *Minor,
   Supernatural* — a category permitted on all four profiles, three of which are
   not `is_magus`. **F-409's thirty-second instance**, and the sharper of this
   batch's two, because the clause exists *only* to grant the permission: it is
   the second half of the sentence that states the Flaw's compensating benefit,
   not an aside. Note the German is even more explicit — "darf er Magiekunde
   **bereits** bei der Charaktererschaffung erlernen".
2. **Four mechanical blocks reach the user in neither locale (D5).** ArMDE:6458
   makes one of the four Magic-being types mandatory ("the character **also
   gains** a minor physical deformity appropriate to the Magic being type"), so
   these are not optional colour. None is computed and none is written down:
   - the Magic Animal **-3** on appropriate activities — no effect scopes a
     penalty to an activity class;
   - the Magic Human **Characteristic decrease of 1, floor -3** — a
     *player-chosen* Characteristic, which `A2 CharacteristicScoreDeltaParam`
     cannot express (its `amount` is fixed by the item, and its precondition
     validator gates the opposite direction), so this is **not** a D2 case;
   - the Magic Human **poor Reputation of level 3 among other magic beings** —
     doubly inexpressible: `types.rs::Reputation::score` is a `u8` with no
     polarity (B10's **F-380**) and `types.rs::ReputationType` has only
     `local` / `ecclesiastical` / `hermetic` / `academic`, with no such
     audience;
   - the Magic Spirit **-3 on Intelligence and Perception rolls** — `A41
     AbilityRollMod` is keyed on a free-text *subject* and is surfaced-only
     anyway, and nothing modifies a Characteristic *roll*.

**Correct value.** `classification: "creation_effect"` (see **Q-120** — the
entry would then compute in both phases and `types.rs::Classification` admits
one answer), keeping the `aging_mod` and adding
`{ "type": "ability_authorization", "abilities": ["ability.magic_lore"] }` — the
`abilities` form rather than `categories`, because the passage permits one named
Ability and not the whole `arcane` category. All four deformity blocks, plus the
aging bonus and the permission themselves, must be written into `description` in
**both** locales.

**The exact wording the phrase screen missed** — and this one is a *different*
miss from F-460's, because the entry is not `narrative`, so the screen never
looked at it at all. The screen's third assertion is scoped to `narrative`
entries; an `in_play_effect` entry that computes one clause and silently drops
five is invisible to every assertion in `uncomputed_clauses.rs`. That is D5's
whole point, and it is why this finding could not have come from the screen:

- EN: `may learn Magic Lore during character creation`
- DE: `außerdem darf er Magiekunde bereits bei der Charaktererschaffung erlernen`
- EN: `must decrease one of his Characteristics by 1, but not below –3`
- DE: `muss eine seiner Eigenschaften um 1 senken, jedoch nicht unter –3`

(For the record, the first pair carries **no** token the screen knows — the
permission idiom again — while the second pair *does* carry a signed number
(`–3`) and would have tripped the screen had the entry been `narrative`. The
class, not the vocabulary, is what hid this one.)

**Severity: HIGH.** A character the book describes and the app refuses to build,
plus five lost rules on one entry — the largest D5 deficit this batch found.

### F-462 — `flaw.missing_eye` applies the melee penalty to missile weapons and drops the missile penalty entirely

> "You cannot judge close distances easily and get **-3 on Attack rolls for
> missiles** and targeting rolls for spells. **In melee combat you suffer -1 on
> Attack rolls** because your field of vision is limited. You also have a blind
> side from which people can approach unseen. This Flaw can be combined with
> Poor Eyesight, but the penalties are cumulative."
> — ArMDE:6436

> "Du kannst kurze Abstände nicht leicht einschätzen und erhältst **–3 auf
> Angriffswürfe mit Fernkampfwaffen** und Zielwürfe für Zauber. **Im Nahkampf
> erleidest du –1 auf Angriffswürfe**, weil dein Sichtfeld eingeschränkt ist."
> — DE 6436

**Current data.** `classification: "in_play_effect"`, one effect:
`{ "type": "combat_mod", "amount": -1, "target": "attack" }`, **unscoped** (no
`weapon` field). Both locales carry the full passage as `description` ✓, which
is what keeps the text column green.

**Why it is wrong.** The passage states **two different amounts for two
different weapon classes**, and the data states one amount for all of them.
`derived/combat.rs::combat_totals` builds

```
cm(stat, weapon) = Σ unscoped(stat) + delta(weapon, stat)
```

and iterates **every equipped weapon slot**, so the unscoped `-1` lands on the
Attack line of `weapon.bow_long`, `weapon.bow_short`, `weapon.sling`,
`weapon.javelin`, `weapon.axe_throwing`, `weapon.knife_thrown` and
`weapon.stone` — all seven of which `rules/core/equipment.json` ships, each with
`ability.bows` or `ability.thrown_weapon`. A character with Missing Eye and a
long bow is therefore shown an Attack total two points too high, and the -3 the
book gives him reaches no total anywhere.

**This is not a Part C gap and must not be filed as one.** `A35`'s "silently
ignores: circumstance" covers a *conditional* amount folded unconditionally.
This is not one amount under a condition; it is two amounts for two weapon
classes, and `Effect::CombatMod`'s `weapon` field expresses exactly that. The
mechanism is live, used, and pinned: `flaw.lame` carries a `-3` Defense scoped
to `weapon.dodge` beside its unscoped `-1`, `derived.rs::in_play_mods` stores
the scoped figure as a **delta** over the item's own unscoped sum so the scoped
line reads -3 and not -4, and
`derived.rs::weapon_scoped_combat_mod_replaces_general_on_that_weapon_only`
asserts it. Everything F-462 asks for already exists one entry away.

**And an in-repo authority asserts the opposite.**
`crates/arm-rules/RULES.md:5161` lists, among the amounts that *are* folded,
"**missing_eye's ranged −3**". No such amount is in the catalogue. RULES.md's
per-item paragraphs at `:5023-5061` itemise `flaw.hobbled`, `flaw.lame`,
`flaw.missing_hand` and `flaw.palsied_hands` and argue each encoding out; this
entry is **not** among them, and `data_integrity.rs::the_combat_roll_flaws_penalize_the_combat_ability_totals`
does not pin it either. So the one place that describes this entry's encoding
describes an encoding that does not ship, and no test would notice. That is the
same shape B14's F-449 found at `engine-semantics.md:1455` — a yardstick stating
correct behaviour as a fact about code that does not implement it — in a second
document.

**And RULES.md's claim was false at birth, not by drift.** The verification pass
traced it: `git log -S"missing_eye's ranged"` puts the phrase into RULES.md at
`acc1657` ("M5 slice 5b"), and `git show acc1657:rules/core/virtues_flaws.json`
shows `flaw.missing_eye.effects` was **already** `[{ "type": "combat_mod",
"amount": -1, "target": "attack" }]` at that same commit; the entry had no
effects at all before it. So the "ranged −3" sentence has never described any
shipped revision of the data. That is a sharper statement than "the map drifted"
and it says something about the map: a traceability line can be written wrong and
stay wrong for as long as nothing reads it against the file it describes.

**Correct value.** Keep the unscoped `{ -1, attack }` (it is the melee figure
and "other" weapons is what the -1 is for), and add a `weapon`-scoped
`{ -3, attack, weapon: <id> }` for **each** of the seven missile weapons, which
the delta fold turns into -3 on those lines while leaving -1 everywhere else —
exactly `flaw.lame`'s pattern. The "targeting rolls for spells" clause has no
consumer (no effect modifies a targeting roll) and stays as `description` text,
where it already is ✓. RULES.md needs a per-item paragraph for this entry beside
the other four, and `:5161`'s claim must be corrected or removed.

**The remedy is not a one-line data edit, and the correction pass must know
why.** `Effect::CombatMod`'s `weapon` field scopes to a **single weapon id**, so
ArMDE:6436 needs **seven** rows — one per missile weapon — and an eighth missile
weapon added to `equipment.json` later would silently fall back to the melee -1.
That sits badly with `CLAUDE.md`'s "**catalogue size is data, never code**"
invariant: the correctness of a V/F would depend on the size of the equipment
catalogue. Two honest options, and the choice is the orchestrator's: **(a)** the
seven rows plus a guard asserting that every weapon whose `ability` is
`ability.bows` or `ability.thrown_weapon` is covered, so adding a bow fails
loudly; or **(b)** an ability-scoped `CombatMod` — a new optional field beside
`weapon`, scoping by the weapon's Ability rather than its id, which is how the
book actually divides missile from melee and which would need one row instead of
seven. (b) is a code change and therefore out of scope for a data correction
pass, but it is the shape the rule is really in.

**Severity: HIGH — this is the batch's only wrong number.** Per `CLAUDE.md`,
wrong rules output is the top severity class: the app's entire purpose is
computing correct Ars Magica characters, and a bowman's printed Attack total is
two points better than the rules allow. The error is also **permissive**, which
is the worse direction, and it compounds with `flaw.poor_eyesight`, whose
passage explicitly invites the combination ("the penalties are cumulative"): the
pair should read -6 at range and reads -4.

### F-463 — every magus may take `flaw.magical_air`, which ArMDE:6384 forbids him outright, and the Flaw's own -3 reaches nobody

> "People and animals react to you **as if you had The Gift**. **You may not take
> this Flaw if you actually do have The Gift**; see The Blatant Gift (page 120)
> instead."
> — ArMDE:6384 (the entry's entire text)

> "Menschen und Tiere reagieren auf dich, **als hättest du die Gabe**. **Du
> kannst diesen Fehler nicht wählen, wenn du tatsächlich die Gabe besitzt**;
> stattdessen siehe Auffällige Gabe ([Seite 120](#auffällige-gabe))."
> — DE 6384

**Current data.** `classification: "narrative"`, no `effects`, no
`prerequisites`, `incompatible_with: ["flaw.offensive_to_beings"]`. Both locales
carry `name` + `summary` only, and the summary is sentence one — so the
prohibition, which is the entry's *second half and half its word count*, ships
nowhere.

**Why it is wrong, in two ways.**

1. **The prohibition is enforced by nothing, and the build it permits is the
   commonest build in the game.** `virtue.the_gift`'s `incompatible_with` is
   `["virtue.devil_child","virtue.faerie_doctor","virtue.failed_apprentice","virtue.nephilim","virtue.spirit_votary"]`
   — it does not name this Flaw, and this Flaw does not name it. So
   `validation/prereq.rs::validate_incompatibilities` has nothing to test and a
   character holding both validates clean.
   **And every magus holds The Gift.** The magus profile's `required_traits` is
   `["virtue.hermetic_magus"]`; `virtue.hermetic_magus`'s own `prerequisites` is
   `Has(virtue.the_gift)`; and **no** profile lists `virtue.the_gift` in
   `granted_selections` — it appears only on grog's `forbidden_traits`. The Gift
   is therefore always a **bought** selection, which matters because
   `validate_incompatibilities` reads the bought-only `selected_ids`
   (`validation/mod.rs::validate`, a scope documented as deliberate at
   `validation/prereq.rs::PrereqCtx::build`): the declaration this finding asks
   for would fire, and nothing weaker is needed. Nothing else blocks the pair —
   `magical_air` is `general`, which is on every profile's permitted list, and
   `effective/gift_confidence.rs::has_the_gift` reads `gift_categories`
   (`["hermetic"]`), so the Flaw is invisible to Gift detection too.
2. **`narrative` is wrong on a passage that imports a signed modifier.**
   "People and animals react to you as if you had The Gift" is a reference to a
   number: ArMDE:8751 gives the Gifted "a **-3 penalty** to all rolls and totals
   based on social interaction, including training". The entry states a -3
   without printing one. No effect expresses it either — the penalty attaches to
   how *others* react, and `A41 AbilityRollMod` is a free-text subject and
   surfaced-only — so per **D3** this is `uncomputed_rule` with the rule written
   out, never `narrative`.
   The catalogue already contains the right answer for this exact shape, one
   entry away: `flaw.offensive_to_beings` says the same thing narrowed to one
   class of being, spells its -3 out at ArMDE:6528, and ships `uncomputed_rule`
   with the whole passage in `description` in both locales.

3. **And a third clause is attributed to the Flaw by name from outside its
   range.** ArMDE:21130 (the Divine-Might chapter): "Specific Flaws (such as
   **Offensive to Divine Beings** and Unbearable to Divine Beings) that modify or
   simulate the effects of The Gift may cause creatures with Divine Might to
   react negatively to the character with the Flaw, **although Magical Air does
   not**." That is a carve-out naming this Flaw, and it matters precisely because
   clause 1 says the Flaw makes people react "as if you had The Gift" — the
   reader needs to know the analogy stops at Divine Might beings. It is an
   inbound pointer, 14,700 lines past the entry, invisible to its cited range
   6382-6385 and to the screen. It is owed in `description` under **D5** along
   with the rest, and the same sentence's first half is owed to
   `flaw.offensive_to_beings` for its `being.divine` value — noted there rather
   than counted, since that entry's description already carries its own passage
   and the clause is value-scoped in a way the entry cannot express.

**Correct value.** `classification: "uncomputed_rule"`;
`incompatible_with: ["flaw.offensive_to_beings", "virtue.the_gift"]`, with the
reverse edge added to `virtue.the_gift` (required by
`ruleset/integrity.rs::validate_incompatibility_symmetry`); and the full
passage — the "as if you had The Gift" reaction, the prohibition, **and**
ArMDE:21130's Divine-Might carve-out — in `description` in both locales.

**The exact wording the phrase screen missed**, for the screen-fix list:

- EN: `People and animals react to you as if you had The Gift. You may not take this Flaw if you actually do have The Gift`
- DE: `Menschen und Tiere reagieren auf dich, als hättest du die Gabe. Du kannst diesen Fehler nicht wählen, wenn du tatsächlich die Gabe besitzt`

Neither carries a sign, a botch term, or any of the twenty `MECHANICAL_PHRASES`.
Two idiom families would have caught it and neither is in the list: the
**prohibition** idiom B12 named ("you may not take this Flaw", "du kannst diesen
Fehler nicht wählen"), and a **by-reference** idiom this entry hands over free —
"as if you had The Gift" / "als hättest du die Gabe", a construction that by
construction imports another rule's numbers and appears in at least three
entries of this block (ArMDE:6384, :6528, :5717).

**Severity: HIGH.** A hard rule the app does not enforce, on a build every magus
can reach, plus a lost rule. Rated up rather than down per `CLAUDE.md`: this is
wrong rules output in the permissive direction, not a cosmetic classification
slip.

### F-464 — `flaw.no_hands` is `narrative` on a -5 Casting Score penalty, and the phrase screen missed it because of a space

> "You have no hands. Any activity requiring hands is impossible, and **magi
> with this Flaw take a – 5 penalty to all Casting Scores**. This may be offset
> by taking the **Subtle Magic Virtue** (page 110) or the **Still Casting Spell
> Mastery** ability."
> — ArMDE:6498 (the entry's entire text)

> "Du hast keine Hände. Jede Tätigkeit, die Hände erfordert, ist unmöglich, und
> **Magi mit diesem Fehler erhalten –5 auf alle Zauberwerte**. Dies kann durch
> die Tugend **Subtile Magie** ([Seite 110](#subtile-magie)) oder die
> Meisterschaftsfähigkeit **Regloses Zaubern** ausgeglichen werden."
> — DE 6498

**Current data.** `classification: "narrative"`, no `effects`, no
`prerequisites`, no `incompatible_with`. Both locales carry `name` + `summary`
only, and the summary is the four-word first sentence — *"You have no hands." /
"Du hast keine Hände."* So a Major Flaw whose whole mechanical content is a -5
to every Casting Score ships as a four-word statement of fact.

**Why it is wrong.** The -5 is a real number the book states plainly, so
`narrative` is false on its face. It is also **the engine's own constant**:
`derived.rs::NO_GESTURE_PENALTY` is `-5`, and `virtue.subtle_magic` — the very
Virtue ArMDE:6498 names as the offset — ships
`special_casting_mod { subtle_gestures }`, which
`derived.rs::SUBTLE_MAGIC_GESTURE_REDUCTION` turns into the matching `+5`. The
book's sentence and `derived.rs::residual_gesture_penalty` are computing the
same thing.

**And it is nevertheless inexpressible, which is what makes `uncomputed_rule`
the right class rather than `in_play_effect`.** `residual_gesture_penalty` is a
*per-casting display option*: it is surfaced on `NonStandardCasting` beside the
ordinary Formulaic total, because whether a given casting uses gestures is a
choice made at the table. There is no marker for a character who can **never**
use them, so the engine cannot make the penalty the character's standing figure.
`casting_total_mod { amount: -5, scope: all }` was considered and rejected: it
would move the right number, but Subtle Magic feeds `residual_gesture_penalty`,
a different pipeline, so the offset the passage promises would not apply and the
figure would then be wrong in the other direction for every magus who takes
both. Per **D3**, an engine that structurally cannot express a rule is grounds
for `uncomputed_rule` with the rule written out — never for `narrative`.

**The catalogue already gets the mirror case right, which is the decisive
evidence.** `flaw.mute` (ArMDE:6474) states "-10 penalty to all spellcasting …
may be offset by taking the Quiet Magic Virtue" — the identical shape, one
constant over (`derived.rs::NO_VOICE_PENALTY` is `-10`,
`::QUIET_MAGIC_VOICE_REDUCTION` is `+5`, `virtue.quiet_magic` is
`special_casting_mod { quiet_words }`). `flaw.mute` ships `uncomputed_rule` with
its full passage, the -10 included, in **both** locales. Two Major General Flaws
from the same block, four headings apart, stating the same mechanic against the
same pair of engine constants, and the catalogue answers them differently.

**Correct value.** `classification: "uncomputed_rule"`, with the full passage —
the -5 and both named offsets — in `description` in **both** locales, exactly as
`flaw.mute` carries its own.

**The exact wording the phrase screen missed**, for the screen-fix list, and the
reason it missed it:

- EN: `magi with this Flaw take a – 5 penalty to all Casting Scores`
- DE: `Magi mit diesem Fehler erhalten –5 auf alle Zauberwerte`

`uncomputed_clauses.rs::has_signed_number` walks the string and returns true only
when a `SIGN_CHARS` member is followed **immediately** by an ASCII digit. The
English source at ArMDE:6498 writes `342 200 223` (U+2013 EN DASH), then **a
space**, then `5` — verified byte-by-byte with `od -c`. So the detector sees no
signed number, `MECHANICAL_PHRASES` matches nothing in the sentence, there is no
botch term, and the entry passed both sweeps of its `SWEPT_BLOCKS` block.
**This is the only sign-space-digit occurrence in the whole Flaws block**: `rg -n
-- "[–−+-] [0-9]"` over ArMDE:5639-7113 returns exactly this one line. And the
screen reads the **English** file only (`uncomputed_clauses.rs::bracketed_passage`
joins `rules_dir()/source/en/<file>`), so the German source — which writes `–5`
with no space and *would* have tripped it — never gets a vote. The fix is a
one-character change to `has_signed_number`: skip optional whitespace between
the sign and the digit. That is cheaper and more general than any phrase, and it
is the first time this audit has found the detector defeated by typography
rather than by vocabulary.

**Severity: HIGH.** A Major Flaw's entire mechanical content reaches the user in
neither locale, and the screen that exists to catch exactly this was defeated by
a space. The classification defect is a lost rule (`classification` is read by
no production code); the *screen* defect is the wider problem, because it is
catalogue-wide and silent.

### F-465 — the Night Terrors / Sleep Disorder exclusion is stated from both sides and encoded on neither

> "If a night of terrors renders the character Unconscious, he slips into deep,
> dreamless sleep. He sleeps for an entire day and night, and wakes up at
> Winded, ending the string of Night Terrors. **A character with this Flaw may
> not also take the Sleep Disorder Flaw**"
> — ArMDE:6492

> "Once he has lost 3 Long Term Fatigue Levels, he automatically sleeps through
> the night, due to sheer exhaustion. **This Flaw is not compatible with the
> Night Terrors Flaw.**"
> — ArMDE:6749 (`#### Sleep Disorder`, *Minor, General*, B17's span)

> "Ein Charakter mit diesem Fehler **darf nicht zusätzlich den Fehler
> Schlafstörung nehmen**."
> — DE 6492

**Current data.** `flaw.night_terrors`: `classification: "uncomputed_rule"`,
full three-paragraph passage in `description` in both locales ✓,
**`incompatible_with` absent**. `flaw.sleep_disorder`: `classification:
"uncomputed_rule"`, **`incompatible_with` absent**. Neither names the other.

**Why it is wrong.** A character holding both validates clean today:
`validation/prereq.rs::validate_incompatibilities` tests each bought item's
declared set against `selected_ids`, and both sets are empty, so there is
nothing to test. Both Flaws are *Minor, General*, `general` is on every
profile's permitted list, and neither is capped by any
`budget.flaw_category_caps` row (the four profiles cap only `personality` and
`story`). So the pair is reachable by every character type in the app, for two
points, and the book forbids it **twice**.

**What makes this the strongest exclusion finding so far.** Every previous
instance of this shape in the audit rested on one sentence in one entry. This
one is stated independently in *both* entries' own passages, in the book's own
words, which removes the only reading under which the omission could have been a
judgement call: there is no side from which the rule is invisible.

**Correct value.** `incompatible_with: ["flaw.sleep_disorder"]` on
`flaw.night_terrors` and `incompatible_with: ["flaw.night_terrors"]` on
`flaw.sleep_disorder` — both edges, because
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
one-sided declaration. No classification change: both are already
`uncomputed_rule` and both already carry their passages. The correction touches
**B17**'s entry as well as this one and is recorded here so it is not found
twice.

**The exact wording the phrase screen missed** — *nothing*, and that is the
point worth recording. `flaw.night_terrors` is `uncomputed_rule`, so the
narrative screen never examines it; and its `description` passes the
`uncomputed_rule` assertion easily on "Ease Factor of 9" and "-1 to -3". The
screen is structurally incapable of noticing an *unencoded* incompatibility,
because it inspects text and this defect is an absence in the data. There is no
guard anywhere that reads a passage for the words "may not also take" and checks
`incompatible_with` — which is why three of this batch's findings (F-463, F-465,
F-466) are the same missing mechanism, and why a screen fix for them would have
to be a different kind of screen. For the record, the phrases are:

- EN: `A character with this Flaw may not also take the Sleep Disorder Flaw` / `This Flaw is not compatible with the Night Terrors Flaw`
- DE: `Ein Charakter mit diesem Fehler darf nicht zusätzlich den Fehler Schlafstörung nehmen` (DE 6492) / `Dieser Fehler ist nicht vereinbar mit dem Fehler Nachtschrecken.` (DE 6749)

**Severity: MEDIUM-HIGH.** An illegal build the app validates as legal. Not the
top class, because nothing prints a wrong *number* — but it is an enforcement
gap on an explicit rule, reachable by every character type, and both sides of
it were written down.

### F-466 — the No Sense of Direction / Well-Traveled exclusion is stated and encoded on neither entry

> "You frequently get lost while traveling unfamiliar paths by yourself, or with
> others following your lead, and often have to reason your way home or to your
> destination from first principles. **This Flaw is incompatible with the Well
> Traveled Virtue.**"
> — ArMDE:6502 (the entry's last sentence)

> "Du verläufst dich häufig auf unbekannten Wegen … und musst dich oft durch
> Überlegung nach Hause oder zu deinem Ziel vorarbeiten. **Dieser Fehler ist
> unvereinbar mit der Tugend Vielgereist.**"
> — DE 6502

**Current data.** `flaw.no_sense_of_direction`: `classification: "narrative"`,
no `effects`, **`incompatible_with` absent**, `name` + `summary` only in both
locales, and the summary is sentence one — *"You are completely unable to follow
directions."* — so the exclusion ships nowhere. `virtue.well_traveled`
(ArMDE:5239-5242, **B09**'s span): **`incompatible_with` absent** too.

**Why it is wrong.** As F-465: two empty sets mean
`validate_incompatibilities` has nothing to test, and the pair validates clean.
Both are *Minor, General*, so every character type can reach them and no
category cap intervenes.

**The target was read in full and rated, and it changes the shape of the
fix.** `virtue.well_traveled` (ArMDE:5241) reads: "You have journeyed
extensively in this part of the world and find it easy to get along with people
throughout the area. You have fifty bonus experience points to spend on living
languages, Area Lores, and Bargain, Carouse, Charm, Etiquette, Folk Ken, or
Guile." It states the exclusion from **neither** side — unlike F-465, the book
says it once. That does not make the data's omission defensible, but it does
mean the reverse edge is an artefact of
`ruleset/integrity.rs::validate_incompatibility_symmetry` rather than of the
rulebook, and the correction pass should know which of the two declarations is
sourced and which is structural.

**Correct value — and `incompatible_with` is the wrong tool here, which my first
draft got wrong and the verification pass caught.** `virtue.lone_redcap` carries
`grants_selection: { items: ["virtue.well_traveled"] }`, sourced to ArMDE:4321
("receive the benefits of the Well-Traveled virtue"). A Lone Redcap therefore
holds a **granted** Well-Traveled, and `validate_incompatibilities` sees only
bought selections on *both* sides — `validation/mod.rs::validate` builds
`selected_ids` from `entity.selections` and
`validation/prereq.rs::validate_incompatibilities` iterates the same list — so
the declared pair would fire for a bought Well-Traveled and stay silent for the
Redcap, which is the commoner build of the two. The remedy that works is
`prerequisites: Nor([Has(virtue.well_traveled)])` on
`flaw.no_sense_of_direction`, because `PrereqCtx::present_ids` **is**
grants-inclusive by design. This is the one place in this batch where the
documented bought-only scope of `incompatible_with` (Part C, C4a) is not merely a
scope fact but decides the fix. `classification: "narrative"` **stands**: the
passage states no computed quantity, and an encoded `incompatible_with` on a
`narrative` entry is the catalogue's established shape — **62** of the 335
`narrative` entries carry a non-empty one, including this batch's
`flaw.magical_air` and the three Major/Minor pairs. The exclusion should
nevertheless be added to the `summary` or a `description`, because a hard
validator error that the item's own text never predicted is a poor experience;
that half is a recommendation rather than a D5 obligation, since the rule *is*
computed once the edge exists.

**The exact wording the phrase screen missed:**

- EN: `This Flaw is incompatible with the Well Traveled Virtue.`
- DE: `Dieser Fehler ist unvereinbar mit der Tugend Vielgereist.`

No sign, no botch term, no `MECHANICAL_PHRASES` member. This is the third
instance in this batch of an idiom family the screen has no word for — the
**exclusion** idiom ("is incompatible with", "may not also take", "may not take
… if", "ist unvereinbar mit", "darf nicht zusätzlich", "kannst … nicht wählen,
wenn") — and unlike the permission and prohibition families B12 named, this one
would not even be the right tool, since the defect is an absent field rather
than an absent sentence. Recorded for the screen-fix list all the same, because a
phrase that reliably marks "this entry needs an `incompatible_with`" is worth
having whichever guard eventually reads it.

**Severity: MEDIUM.** An illegal build the app validates as legal, on a
one-sided rule. Lower than F-465 only because the book states it once.

### F-467 — `flaw.master_of_none` is `narrative` on an advancement rule that reaches the user in neither locale

> "Either by choice or circumstance, this character never seems to be able to
> stick at a task or job for longer than a season, usually following the work as
> it moves through the year. As a result, **this character can't apply any
> experience points earned through an Advancement Total to an Ability or Art
> that they have already applied experience points to this year. Where they
> can't be applied to a different Ability or Art, the experience points are
> lost.** Experience points gained through other means, such as Secondary
> Insight, are unaffected and may be applied normally."
> — ArMDE:6420 (the entry's entire text)

> "Infolgedessen **kann dieser Charakter keine Erfahrungspunkte, die durch einen
> Fortschrittssumme erlangt wurden, auf eine Fertigkeit oder Kunst anwenden, auf
> die er in diesem Jahr bereits Erfahrungspunkte angewendet hat. Wenn sie nicht
> auf eine andere Fertigkeit oder Kunst angewendet werden können, gehen die
> Erfahrungspunkte verloren.**"
> — DE 6420

**Current data.** `classification: "narrative"`, no `effects`. Both locales
carry `name` + `summary` only, and the summary is sentence one — the colour
about not sticking at a job — so **the whole rule**, two of the passage's three
sentences, ships nowhere.

**Why it is wrong.** The passage states a hard constraint on how experience may
be spent, plus a consequence ("the experience points are lost") that destroys a
resource. That is mechanical by any reading. It is also inexpressible:
`A39 AdvancementMod` is `{ source, amount }` — a signed modifier on a Source
Quality — and this rule is not an amount at all but a restriction on which
*target* an amount may be applied to. No other variant is closer. Per **D3**
that is grounds for `uncomputed_rule` with the rule written out, never for
`narrative`.

**The carve-out was followed and confirms the reading.** "Experience points
gained through other means, such as **Secondary Insight**, are unaffected" names
`virtue.secondary_insight` (ArMDE:4892-4895), a real catalogue entry that is
itself an `advancement_mod` carrier — so the sentence is describing a live
interaction between two modelled entries, not flavour.

**Correct value.** `classification: "uncomputed_rule"`, with the full passage —
the restriction, the loss, and the Secondary Insight carve-out — in
`description` in **both** locales.

**A German-source error rides along with the fix, and must be corrected in the
same step.** DE 6420 reads "Erfahrungspunkte, die durch **einen**
Fortschrittssumme erlangt wurden". `die Summe` is feminine, so the article must
be *eine*. It is the identical error B14 recorded twice (**F-458** at DE 6356,
and the same slip at DE 6296), which makes three known instances of one
translator's mistake on one noun. Today it does not ship, because it is in
sentence two and the shipped German text is sentence one; the moment this
finding's `description` is written it does. Recorded here rather than as a
separate finding because the two corrections are one edit.

**The exact wording the phrase screen missed**, for the screen-fix list:

- EN: `this character can't apply any experience points earned through an Advancement Total to an Ability or Art that they have already applied experience points to this year`
- DE: `kann dieser Charakter keine Erfahrungspunkte, die durch einen Fortschrittssumme erlangt wurden, auf eine Fertigkeit oder Kunst anwenden, auf die er in diesem Jahr bereits Erfahrungspunkte angewendet hat`

No sign, no botch term, no `MECHANICAL_PHRASES` member. The **prohibition**
idiom again ("can't apply", "kann … nicht anwenden"), plus a resource-loss idiom
worth its own entry — "the experience points are **lost**" / "gehen die
Erfahrungspunkte **verloren**", a construction that never describes colour.
Note also that a naming token would have caught this one where a phrase list
struggles: the passage says "**Advancement Total**", a proper rulebook term the
screen does not know, exactly as it knows "Ease Factor".

**Severity: MEDIUM-HIGH.** A whole Minor Flaw's mechanical content lost in both
locales. Not the top class — nothing computes a wrong number — but the user who
takes this Flaw is given no way to learn what it does.

### F-468 — `flaw.motion_sickness` is `narrative` on a doubled Fatigue loss and a two-level floor

> "Riding a horse, in a cart, or sailing on a ship makes you violently ill. When
> not traveling on foot, **you suffer double the fatigue loss on long journeys,
> with a minimum loss of two Fatigue levels**. Violent jostling over a period of
> a few hours could conceivably lead to unconsciousness."
> — ArMDE:6470 (the entry's entire text)

> "Wenn du dich nicht zu Fuß fortbewegst, **erleidest du auf langen Reisen
> doppelten Erschöpfungsverlust, wobei der Mindestverlust zwei
> Erschöpfungsstufen beträgt**."
> — DE 6470

**Current data.** `classification: "narrative"`, no `effects`. Both locales
carry `name` + `summary` only, and the summary is sentence one, so the
multiplier and the floor ship nowhere.

**Why it is wrong.** "Double the fatigue loss" is a multiplier and "a minimum
loss of two Fatigue levels" is a floor — two of the sharpest mechanical
statements a rulebook makes. It is also inexpressible: `A36 HealthMod`'s five
`HealthTrack` members are `fatigue_penalty`, `wound_penalty`, `fatigue_roll`,
`casting_fatigue` and `recovery`, and the two that compute anything move a
*tier's printed penalty*, not a *quantity of levels lost*; none of the five
names a loss, and none carries a multiplier — every `health_mod` is an additive
`amount`. Per **D3**, `uncomputed_rule` with the rule written out.

**Correct value.** `classification: "uncomputed_rule"`, with the full passage in
`description` in **both** locales.

**The exact wording the phrase screen missed**, for the screen-fix list:

- EN: `you suffer double the fatigue loss on long journeys, with a minimum loss of two Fatigue levels`
- DE: `erleidest du auf langen Reisen doppelten Erschöpfungsverlust, wobei der Mindestverlust zwei Erschöpfungsstufen beträgt`

No sign (both quantities are words, not digits), no botch term, and no
`MECHANICAL_PHRASES` member — note that the list carries `"at least"` /
`"mindestens"` but not `"minimum"` / `"Mindest"`, and carries `"multiply"` /
`"multiplied"` / `"multiplizier"` but not `"double"` / `"doppelt"`. **Two new
idiom families fall out of this one entry**, and both are cheap and safe to add:
a **floor** family (`minimum`, `Mindest`) and a **multiplier-in-words** family
(`double`, `doppelt`, `twice`, `halve`/`halbiert` — the last already reachable
through other entries in this block). Neither can match colour: a rulebook does
not say "a minimum loss of two" about anything but a quantity. This is the first
entry in the audit whose rule is invisible because **both of its numbers are
spelled as words**, which is a gap the `has_signed_number` fix from F-464 would
*not* close.

**Severity: MEDIUM-HIGH.** A whole Minor Flaw's mechanical content lost in both
locales.

### F-469 — `flaw.magical_being_companion` is `narrative` on a Might formula, and permits a repeat the book does not

> "**Realms of Power: Magic** contains extensive rules for designing such
> characters, but they can also be made following the guidelines and examples in
> Chapter 13 of this book. …
> A particularly common example is a magical animal that's smart enough to
> follow your orders or to disobey them on its own initiative. The smaller and
> more innocuous the creature, the more intelligent it is. A ferret or crow is
> as intelligent as a human, a wolf is very cunning, and an animal the size of a
> horse is simply more intelligent than normal. **The creature has a Magic Might
> score of 10 – Size.**"
> — ArMDE:6388, :6390

> "**Das Geschöpf hat einen Machtwert von 10 – Größe.**"
> — DE 6390

**Current data.** `classification: "narrative"`, no `effects`, no `max_total`,
`parameters: [{ "key": "being", "type": "ref", "domain": "text" }]`. Both locales
carry `name` + `summary` only, and the summary is the first sentence of the
first paragraph, so the formula ships nowhere.

**Why it is wrong, in two ways.**

1. **`narrative` on a closed formula.** `Magic Might = 10 − Size` is as
   mechanical as a statement gets: two named quantities and an operator. It is
   inexpressible — `A26 MightGrant` grants Might to the **bearer**, and nothing
   in the engine models a companion creature at all — so per **D3** it is
   `uncomputed_rule` with the rule written out, never `narrative`. The *RoP:M*
   pointer at ArMDE:6388 cannot be followed (*Realms of Power: Magic* is in
   `rules/source/en/` as a separate book but the passage names no page and the
   rule this entry needs is stated right here), and it is not needed: the
   formula is in the core book, inside this entry's own cited range.
2. **A parameterized entry with no `max_total`, against ArMDE:2814.** The
   `being` parameter is free text, and `validation/selections.rs::validate_duplicate_selections`
   keys `max_per_target` on `(item_ref, **whole** params map)` — so *Magical
   (Ferret) Companion* and *Magical (Crow) Companion* are two distinct tuples
   and both are legal today. ArMDE:2814 is explicit that "a Virtue or Flaw may be
   taken more than once **only if the description explicitly allows it**", and
   this description does not: it is singular throughout — "accompanied by **an**
   intelligent magical being … which the character must regularly look after".
   This is the B10 **F-358** / B11 **F-400** / B14 **F-451** shape in a new
   place.
   **Narrowed by the verification pass, and the narrowing is worth stating
   exactly.** The entry is a **Story** Flaw, and `validation/caps.rs::validate_caps`
   counts *selections*, not distinct items, against every profile's
   `{"category":"story","max":1}` — so two copies **do** produce
   `too_many_story_flaws`. That cap carries no `"hard": true`, so it is a
   **warning**, not an error. The precise claim is therefore: two Magical (Being)
   Companions raise **no blocking check and nothing that names this item** — only
   the generic non-blocking Story-cap warning that any two Story Flaws would
   raise. "Two tuples and legal" overstated it; the multiplicity itself is
   still unreported, which is the defect.

**Correct value.** `classification: "uncomputed_rule"`, with the whole passage —
both paragraphs, the intelligence-by-size guidance and the Might formula — in
`description` in **both** locales; and `max_total: 1`, so the `being` parameter
keeps recording *which* companion without licensing a second one.

**The exact wording the phrase screen missed**, for the screen-fix list:

- EN: `The creature has a Magic Might score of 10 – Size.`
- DE: `Das Geschöpf hat einen Machtwert von 10 – Größe.`

This is the second entry in this batch defeated by **typography rather than
vocabulary**, and by a *different* typographic accident from F-464's. The
sentence contains a U+2013 EN DASH, which is in `SIGN_CHARS` — but
`has_signed_number` requires an ASCII **digit** immediately after the sign, and
what follows here is a space and then the word `Size`. The formula's *only*
digit, `10`, sits on the **left** of the operator. So a subtraction written
`<number> – <name>` is invisible to a detector that only looks rightward from a
sign, in both locales, and `MECHANICAL_PHRASES` has no member for it either —
note that `"equal to"` / `"entspricht"` is in the list but this formula is
stated with a bare "has a … score of". The cheap fix is a
**score-formula** idiom (`Might score of`, `score of 10`, `Machtwert von`)
rather than another change to the sign detector; a more general one is to treat
a digit *followed* by a sign-and-word as a formula too.

**Severity: MEDIUM-HIGH.** A lost rule in both locales, plus a multiplicity the
book forbids by silence. Not the top class: the Might formula describes an NPC
the app does not build, so no printed number is wrong — but a player who takes
this Flaw is given no way to learn what his companion is.

### F-470 — `flaw.obese`'s movement penalty reaches the user in neither locale

> "You are large because of fat, not muscle. **You are at –1 to all rolls that
> involve moving quickly or gracefully** and at –3 to all Fatigue rolls. You are
> not so large that your Size is increased, and **you may take this Flaw along
> with the Virtues and Flaws that change your Size**."
> — ArMDE:6518 (the entry's entire text)

> "Du hast **–1 auf alle Würfe, die schnelles oder anmutiges Bewegen
> erfordern**, und –3 auf alle Erschöpfungswürfe. Du bist nicht so groß, dass
> deine Größe erhöht wird, und du kannst diesen Fehler zusammen mit den Tugenden
> und Fehlern nehmen, die deine Größe verändern."
> — DE 6518

**Current data.** `classification: "in_play_effect"`, one effect:
`{ "type": "health_mod", "track": "fatigue_roll", "amount": -3 }`. Both locales
carry `name` + `summary` only, and the summary is the six-word first sentence —
*"You are large because of fat, not muscle."* So of the passage's four
mechanical or quasi-mechanical clauses, **one** is in the data and **none** is
in the text.

**The effect that is there is correct, and the sign was re-derived.**
`fatigue_roll` has exactly three shipped carriers catalogue-wide —
`flaw.obese` **-3**, `flaw.short_of_breath` **-3**, `virtue.long_winded` **+3** —
so a Flaw-negative value matches both siblings and ArMDE:6518's "-3 to all
Fatigue rolls" ✓. That the track computes nothing is Part C's row (C1: three of
five `HealthTrack` members die in `derived.rs::surfaced_modifiers`), recorded
once and **not** counted against this entry: the value is correctly authored
against an engine that lists rather than folds it, and it does reach the
read-out labelled with its magnitude.

**Why it fails anyway (D5).** The **-1 on all rolls that involve moving quickly
or gracefully** is a second signed modifier, on a different subject, that the
effects do not implement and no locale carries. It is inexpressible — no effect
scopes a penalty to a class of *rolls* (`A41 AbilityRollMod` is keyed on a
free-text subject, is surfaced-only, and names no Ability anyway; `A35 CombatMod`
reaches only the four combat stats) — and **D5** is explicit that any mechanical
clause the engine does not compute must be written into `description`, in both
locales, **whatever the classification**. An `in_play_effect` entry owes this
exactly as an `uncomputed_rule` one does, and the guard cannot see it: the
`uncomputed_clauses.rs` description assertion is keyed on `uncomputed_rule`, so
a computed entry that drops a second clause is invisible to every assertion in
that file. `flaw.missing_eye` and `flaw.missing_hand` in this same batch show
the right shape — both `in_play_effect`, both carrying their whole passage.

The two remaining clauses are also owed: "You are not so large that your **Size
is increased**" (a statement about a computed quantity — `A23 SizeDelta` — which
this entry correctly does *not* carry, but which the text should say, since a
reader may reasonably expect an Obese character to gain Size), and "you **may
take this Flaw along with the Virtues and Flaws that change your Size**" (an
explicit *compatibility*, which correctly corresponds to an empty
`incompatible_with` ✓ and is worth stating so nobody later "completes" the data
with an exclusion the book denies).

**Correct value.** `classification: "in_play_effect"` **stands** — the entry does
carry a computed effect, and per `types.rs::Classification`'s own doc the line
between `uncomputed_rule` and `in_play_effect` is exactly "does it carry an
`Effect`". What changes is the text: the full passage into `description` in
**both** locales.

**The exact wording the phrase screen missed** — *nothing*, again, and for the
same structural reason as F-465: this entry is `in_play_effect`, so no assertion
in `uncomputed_clauses.rs` examines it. Recorded for completeness, because the
sentence would have tripped the screen instantly had the entry been `narrative`:

- EN: `You are at –1 to all rolls that involve moving quickly or gracefully`
- DE: `Du hast –1 auf alle Würfe, die schnelles oder anmutiges Bewegen erfordern`

Both carry an unambiguous `–1` that `has_signed_number` reads without difficulty.
**So the screen's blind spot here is not its vocabulary but its scope**, and this
is the concrete case for D5's first obligation — "the guard stops being
class-keyed". A screen that asked "does this passage state a rule the effects do
not implement?" of all four classes would have flagged this entry, and
`flaw.monstrous_blood` (F-461), on the first pass.

**Severity: MEDIUM.** One lost signed modifier plus two lost statements, on an
entry whose computed half is correct.

### F-471 — `flaw.lycanthrope`'s German summary is cut mid-sentence, with no ellipsis

> "Du wurdest verflucht, dich bei Vollmond (oder ähnlichen, monatlichen
> astronomischen Ereignissen) **in ein gefährliches Raubtier (z.B. Wolf, Luchs
> oder Bär) zu verwandeln.**"
> — DE 6372

**Current data.** `rules/i18n/de/virtues_flaws.json` → `flaw.lycanthrope`:

```
"summary": "Du wurdest verflucht, dich bei Vollmond (oder ähnlichen, monatlichen astronomischen Ereignissen) in ein gefährliches Raubtier (z.B."
```

The sentence stops after the abbreviation `z.B.`, mid-clause, with an **unclosed
parenthesis** and without the verb the whole sentence is built around
(`zu verwandeln`). The English summary is the complete sentence and is correct;
the German `description` is the complete three-paragraph passage and is also
correct. Only the German summary is damaged.

**Why it happened, and why it matters.** The summary is evidently produced by
splitting the passage at the first sentence-final period, and `z.B.` ends in a
period. So a German passage whose first sentence contains *any* abbreviation is
truncated at that abbreviation. The result is what the user actually sees:
`summary` is the text the V/F picker and the Available list render, and
`description` is only shown where the UI offers it — so a German player browsing
for this Major Supernatural Flaw is shown a fragment that does not say what the
Flaw does. It ends on a bare `(z.B.` — "e.g." — which reads as a rendering fault
rather than as text.

**And it is exactly the case B14's truncation scan said it could not see.**
B14's scan was a literal-ellipsis scan (`...` or `…`) over both locale files, and
it said so explicitly. This summary carries no ellipsis, so it passed. B14's
verification pass spotted it one entry past its own span and flagged it as a
lead for B15; it is confirmed here from the source, and the scan that finds it
is recorded in this batch's Method so the next batch can re-run it.

**Correct value.** The complete first sentence, matching DE 6372:

```
"Du wurdest verflucht, dich bei Vollmond (oder ähnlichen, monatlichen astronomischen Ereignissen) in ein gefährliches Raubtier (z.B. Wolf, Luchs oder Bär) zu verwandeln."
```

**Severity: MEDIUM.** A localization defect that every German user hits on every
visit to this entry, and `CLAUDE.md` rates localization defects up rather than
down ("real users, every session, on every platform"). It is not the top class
because no number is wrong and no rule is lost — the `description` carries the
whole passage — but the entry's primary user-facing string is broken text.

### F-472 — `virtue.cyclic_magic_positive`'s German summary is cut after two characters of an abbreviation

> "Deine Magie ist auf einen Naturzyklus abgestimmt **(z. B. solar, lunar oder
> saisonal)** und ist daher zu bestimmten Zeiten besonders potent."
> — DE 3637

**Current data.** `rules/i18n/de/virtues_flaws.json` → `virtue.cyclic_magic_positive`:

```
"summary": "Deine Magie ist auf einen Naturzyklus abgestimmt (z."
```

The English summary is the full sentence — *"Your magic is attuned to some cycle
of nature (solar, lunar, or seasonal, for example) and as such, is more potent at
specific times."* — so the two locales are not merely different lengths; the
German one is unreadable. It stops **two characters into** the abbreviation,
because DE 3637 spells it `z. B.` with a space, so the split lands on the `z.`
rather than on the `z.B.` F-471 hit.

**This entry is outside B15's span and is rated here rather than left.**
`virtue.cyclic_magic_positive` sorts to index **49** by the audit's canonical
ordering (`sort_by(.source.lines[0], .id)`), which is **B02**'s range (35-69),
and its `source.lines` is `[3635, 3638]`. B02 did not report it. It is recorded
in this batch because it was found by this batch's truncation scan and the brief
requires cross-reference and scan hits to be rated fully wherever they land —
carrying it here means it is not found a second time, and it means the two
truncations are corrected as one job, since they have one cause.

**Correct value.** The complete first sentence, matching DE 3637:

```
"Deine Magie ist auf einen Naturzyklus abgestimmt (z. B. solar, lunar oder saisonal) und ist daher zu bestimmten Zeiten besonders potent."
```

**The scan that finds both, stated so it can be re-run.** Over both shipped
locale files, for every `virtue.*` and `flaw.*` entry: a `summary` ending in a
German or English abbreviation (`z.B.`, `z. B.`, `bzw.`, `usw.`, `d.h.`, `ca.`,
`vgl.`, `e.g.`, `i.e.`, `etc.`, `cf.`), **or** containing an unclosed `(`.
Catalogue-wide that returns exactly **two** rows and both are German:
`flaw.lycanthrope` and `virtue.cyclic_magic_positive`. Stated no wider than
checked — it would not see a summary truncated at a period that genuinely ends a
sentence, so a third shape may still exist. `virtue.cyclic_magic_negative`, the
obvious sibling, was checked and is **not** affected.

**Severity: MEDIUM.** As F-471, and slightly worse as text: `(z.` is not even a
recognisable abbreviation.

### F-473 — the shipped German description of `flaw.offensive_to_beings` calls Magical Air by a name the app does not use

> "Du darfst diesen Fehler nicht mehr als einmal nehmen; Charaktere, die
> gegenüber mehr als einer Art von Wesen abstoßend sind, sollten stattdessen
> **Magisches Auftreten** nehmen. … Charaktere mit **Magischem Auftreten**
> dürfen ihn überhaupt nicht nehmen."
> — DE 6530, shipped verbatim as `flaw.offensive_to_beings`' German `description`

**Current data.** `rules/i18n/de/virtues_flaws.json`:

| id | German `name` |
|---|---|
| `flaw.magical_air` | **Magische Ausstrahlung** |
| `flaw.offensive_to_beings` | Abstoßend für {being} |

and `flaw.offensive_to_beings`' German `description` refers to the first of
those, twice, as **Magisches Auftreten**.

**Why it is wrong.** Three independent sources agree on *Magische Ausstrahlung*
and none supports *Magisches Auftreten*: the German rulebook's own heading at
**DE 6382** (`#### Magische Ausstrahlung`), the canonical glossary at
`rules/source/de/translation-tables/grundbegriffe.md:200`
(`| Magical Air | Magische Ausstrahlung | …`), and its duplicate at
`sphären-mächte.md:152`. The shipped `name` follows all three and is correct.
So the German source contradicts itself inside one book — heading at DE 6382,
body reference at DE 6530 — and the extraction reproduced the body reference
verbatim into the shipped text.

**The user-visible consequence is precise and bad.** A German player reading
Abstoßend für (Wesen) is told two things about a Flaw named *Magisches
Auftreten*: that he should take it instead if he is offensive to more than one
kind of being, and that he may not take this one at all if he has it. Neither
name exists anywhere in the app's UI. He cannot find the Flaw the text sends him
to, and he cannot check the exclusion the text warns him about — which is
especially unfortunate because that exclusion **is** correctly enforced
(`incompatible_with` is mutual on the two entries ✓), so the app will refuse the
combination while naming it something the app never showed him.

**This is a D6 question only in the weakest sense, and the answer does not
depend on the source project.** No translation-table row disagrees with the
rulebook here; the table and the rulebook *heading* agree, and it is a single
sentence of the rulebook *body* that dissents. Per D6's corrected rule 2, a
terminology claim is settled by the glossary, and the glossary is unambiguous.
Per D6's own worked example, a German heading colliding with — or here,
diverging from — another entry's established name "is evidence the heading is
wrong, not evidence the glossary is", and the same logic applies a fortiori to a
body reference.

**Correct value.** `flaw.offensive_to_beings`' German `description` must render
both occurrences as **Magische Ausstrahlung**. This is a deliberate divergence
from the German source at DE 6530 and should be recorded as such, in the same
way B14's **F-458** records a German-source grammatical error the shipped text
must not reproduce. Whether DE 6530 itself should be corrected in
`rules/source/de/` is the orchestrator's call; the shipped text must not wait
for it.

**Severity: MEDIUM.** A localization defect that makes two cross-references
unfollowable for every German user of this entry, on a Flaw whose text is three
quarters cross-reference. Rated up from cosmetic per `CLAUDE.md`'s treatment of
localization defects, and down from HIGH because no rule and no number is lost —
the exclusion it names is enforced.

### F-474 — `flaw.blatant_magical_air`'s prerequisite and its Blatant Gift exclusion are encoded nowhere

> "**Only characters with a Magical Air or The Gift may take this Flaw.** The
> character is especially disturbing to others, so much that they can barely
> tolerate its presence. The character suffers a –6 penalty to social actions,
> and is immediately hated and feared by members of the mundane population. This
> effect is the same as the Blatant Gift, and **a character may not have both
> Flaws.**"
> — ArMDE:5717 (`#### Blatant Magical Air`, *Major, Supernatural*)

**This finding lands outside B15's span and was reached through an inbound
pointer.** `flaw.blatant_magical_air` sits at ArMDE:5715-5718, which is **B10**'s
range (ArMDE:5257-5780); its first sentence names this batch's `flaw.magical_air`
by name, and no screen reading only ArMDE:6382-6385 can see it. B10 did not
report either restriction. It is rated here, from the source rather than from B11's record,
because the brief requires inbound pointers to be followed and rated wherever
they land.

**Current data.** `flaw.blatant_magical_air`: `classification: "uncomputed_rule"`
(correct — the -6 is stated plainly and the screen caught it), no `effects`,
**no `prerequisites`**, **`incompatible_with` empty**. `flaw.blatant_gift`:
`incompatible_with: ["flaw.unbearable_to_beings", "virtue.gentle_gift"]` — it
does not name `flaw.blatant_magical_air` either.

**Why it is wrong, in two ways.**

1. **The eligibility restriction is expressible and the engine already writes it
   twice.** `RULES.md:921-923` records exactly this prerequisite on two
   neighbouring entries:
   `flaw.unbearable_to_beings` carries
   `Any([Has(virtue.the_gift), Has(flaw.magical_air)])` (sourced to ArMDE:6895,
   "Only characters with The Gift or Magical Air may take this Flaw"), and
   `virtue.inoffensive_to_beings` carries the same tree (sourced to ArMDE:4139).
   ArMDE:5717's sentence is word-for-word the same restriction, and this entry
   carries no `prerequisites` at all. So an ordinary grog with neither The Gift
   nor Magical Air may take a Major Supernatural Flaw the book reserves for two
   kinds of character.
2. **The stated exclusion with Blatant Gift is encoded on neither side.** "a
   character may not have both Flaws" is as flat as ArMDE:6892's twin, which
   *is* encoded (`flaw.unbearable_to_beings` ↔ `flaw.blatant_gift`). A character
   holding Blatant Magical Air and Blatant Gift validates clean today — two
   Major Flaws whose passages say they are the same effect.

**Correct value.** On `flaw.blatant_magical_air`:
`prerequisites: Any([Has(virtue.the_gift), Has(flaw.magical_air)])` and
`incompatible_with: ["flaw.blatant_gift"]`; on `flaw.blatant_gift`, the
symmetric `flaw.blatant_magical_air` edge, required at load by
`validate_incompatibility_symmetry`. No classification change:
`uncomputed_rule` is right and the -6 already reaches the user.

**Why it matters that this came from an inbound pointer.** It is the third
consecutive batch in which the *only* route to a real defect ran through a
sentence outside the entry's own cited range — B13's F-442 from ArMDE:7397,
B14's F-452/F-453 from the same line and F-136's re-discovery from ArMDE:4251,
and now this. The pattern is specific enough to name: **an entry's eligibility
rule is often stated in the *other* entry**, and `SWEPT_BLOCKS`' screen, which
reads only `source.lines`, is structurally blind to every one of them.

**Severity: MEDIUM-HIGH.** Two unenforced hard restrictions on a Major Flaw, one
of which (the eligibility gate) the engine already implements twice for
identically-worded siblings — so this is an omission rather than a capability
gap, and the correction is two data edits.

### F-475 — `virtue.alluring_to_beings`' eligibility restriction, which names `flaw.magical_air`, is encoded nowhere

> "These beings are strangely drawn to the character, and generally trust or obey
> her without thinking. She gets +3 on Communication and Presence rolls to
> affect them. **Characters who are Offensive to beings of this sort cannot take
> this Virtue, including those who have The Gift or Magical Air, though
> characters who are Inoffensive to them or have the Gentle Gift may.**"
> — ArMDE:3392

**Outside B15's span, reached by an inbound pointer.**
`virtue.alluring_to_beings` sits at ArMDE:3388-3395, which is **B01**'s range,
and its second paragraph names this batch's `flaw.magical_air`. No screen reading
only ArMDE:6382-6385 can see it. B01 did not report the restriction. It is rated
here, from the source.

**Current data.** `classification: "uncomputed_rule"`, no `effects`, **no
`prerequisites`**, **`incompatible_with` empty**.

**Why it is wrong.** The sentence is a three-way eligibility gate — excluded if
Offensive to that class of being, excluded if Gifted, excluded if you have
Magical Air, *unless* Inoffensive or Gentle-Gifted — and the engine writes
exactly this shape three times already for the neighbouring Beings entries
(`RULES.md:921-923`): `flaw.offensive_to_beings` carries
`Any([Nor([Has(virtue.the_gift)]), Has(virtue.gentle_gift)])`,
`flaw.unbearable_to_beings` and `virtue.inoffensive_to_beings` each carry
`Any([Has(virtue.the_gift), Has(flaw.magical_air)])`. This entry is the fourth
member of that family and is the only one with no gate at all, so a Gifted magus
or a character with Magical Air may take Alluring to (Beings) today and nothing
objects.

**Correct value.** A `prerequisites` tree on `virtue.alluring_to_beings`, in the
family's existing idiom — the natural reading is
`Any([Nor([Has(virtue.the_gift), Has(flaw.magical_air), Has(flaw.offensive_to_beings)]), Has(virtue.gentle_gift), Has(virtue.inoffensive_to_beings)])`,
subject to a reading of whether "Offensive to beings of **this sort**" can be
narrowed to the matching `being` value (it cannot: `Prereq::Has` is id-only and
carries no parameter test, so a flat `Has` over-excludes slightly). Note this is
a **prerequisite**, not an `incompatible_with`, so **no edge lands on
`flaw.magical_air`** and the correction is one-sided — which is why it belongs
to B01's entry even though only B15's reading found it.

**Severity: MEDIUM-HIGH.** An unenforced hard eligibility gate on a Virtue,
where three siblings in the same family already encode the identical sentence.

### F-476 — `virtue.blood_of_the_nephilim` names `flaw.lycanthrope` in a prohibition it does not encode, and drops an Arcane-Lore permission

> "You may learn **Dominion Lore during character creation without needing to
> take the Arcane Lore Minor Virtue**."
> — ArMDE:3511

> "**You may not take** The Gift or True Faith, Hermetic Virtues or Flaws,
> Methods or Powers …, Virtues such as Giant, Mythic, or Faerie Blood, **Flaws
> such as Age Quickly or Lycanthrope**, or Virtues or Flaws that affect your
> Size. … **Magi and Grogs may not take this Virtue.**"
> — ArMDE:3517

**Outside B15's span, reached by an inbound pointer.**
`virtue.blood_of_the_nephilim` sits at ArMDE:3504-3518, **B01**'s range
(ArMDE:3362-3562), and ArMDE:3517 names this batch's `flaw.lycanthrope`. B01 did
not report either clause below. Rated here from the source; the entry's remaining
clauses are B01's and are listed as leads rather than rated.

**Current data.** `classification: "creation_effect"`,
`effects: [{ "type": "size_delta", … }]` only, **`incompatible_with` empty**, no
`prerequisites`, no `entity_kinds` restriction beyond `character`.

**Why it is wrong, in two ways.**

1. **The Dominion Lore permission is a live false hard error — F-409's
   thirty-third instance.** `ability.dominion_lore` is `arcane`, `arcane` is in
   `categories_requiring_virtue`, and the entry carries no
   `ability_authorization` and no `restricted_ability_xp`. ArMDE:3517 says
   "Magi and Grogs may not take this Virtue", so the only holders are companions
   and mythic companions — neither `is_magus`, neither exempt. The sentence even
   names the Virtue it is dispensing with ("**without needing to take the Arcane
   Lore Minor Virtue**"), which is as explicit a statement of the permission
   mechanism as the book makes anywhere.
2. **The prohibition list names two Flaws and encodes neither.**
   `incompatible_with` is empty, so Blood of the Nephilim + Lycanthrope
   validates clean. The list's "such as" hedges the *class* it is illustrating,
   but the two ids it names — `flaw.age_quickly` and `flaw.lycanthrope` — are
   named, so at minimum those two belong in `incompatible_with` (with the
   symmetric edges, one of which lands on `flaw.lycanthrope` in this span). The
   same sentence's "The Gift or True Faith" and "Virtues such as Giant, Mythic,
   or Faerie Blood" name further ids, and "Virtues or Flaws that affect your
   Size" names a *class* the engine could compute (every `size_delta` carrier)
   but `incompatible_with` cannot express — that half is D3-shaped.

**Correct value.** Add
`{ "type": "ability_authorization", "abilities": ["ability.dominion_lore"] }`,
and populate `incompatible_with` with the ids ArMDE:3517 names, each with its
symmetric edge. The class-shaped clauses and the aging block belong in
`description`; `classification` stays `creation_effect`.

**Leads for B02, observed but not rated here**, because following one pointer
does not make this entry mine to audit: ArMDE:3513 states a **-5 to Aging
Rolls**, an aging roll only every ten years after 150, immunity to Longevity
Potions, and an `age ÷ 10` Advancement penalty floored at 1 — the entry carries
**no `aging_mod` at all**, only `size_delta`. ArMDE:3515 grants the Minor
Personality Flaw Greedy "(which counts as one of your normal Flaws)" — a
`grants_selection` shape that is *not* budget-exempt, which no effect expresses.
And ArMDE:3509's "increases by +1 for every century you are alive" is an
age-dependent Size the flat `size_delta` cannot carry. **B01's verdict on this
entry should be re-examined.**

**Severity: HIGH** for the permission half (a character the book describes and
the app refuses to build), **MEDIUM-HIGH** for the unencoded prohibitions.

### F-477 — `virtue.demonic_blood` drops the same Arcane-Lore permission, and it was reached through `flaw.manufactured_ignorance`

> "She may be unaware of her true heritage, in which case she probably has either
> the Delusion or **Manufactured Ignorance** Flaw to explain her remarkable
> capabilities. … **The character may learn Infernal Lore during character
> creation without needing to take the Arcane Lore Minor Virtue.**"
> — ArMDE:3655

**Outside B15's span, reached by an inbound pointer.** `virtue.demonic_blood`
sits at ArMDE:3649-3662, **B02**'s range, and ArMDE:3655 names this batch's
`flaw.manufactured_ignorance`. B02 did not report the permission.

**Current data.** `classification: "creation_effect"`,
`effects: [{ "type": "might_grant", … }, { "type": "power_levels", … }]`, no
`ability_authorization`, no `restricted_ability_xp`.

**Why it is wrong.** `ability.infernal_lore` is `arcane`, `arcane` is in
`categories_requiring_virtue`, and the entry authorizes nothing. Demonic Blood is
*Major, Supernatural*, a category permitted on all four profiles, so three
non-magus types can hold it and none is exempt. **F-409's thirty-fourth
instance**, and the third in this report to use the book's most explicit
formulation of the mechanism — "**without needing to take the Arcane Lore Minor
Virtue**".

**The Manufactured Ignorance mention itself adds nothing** and is rated and
discarded: "she **probably** has either the Delusion or Manufactured Ignorance
Flaw" is a suggestion, not a rule, and attributes nothing to
`flaw.manufactured_ignorance`, whose own `narrative` classification and empty
data stand ✓.

**Correct value.**
`{ "type": "ability_authorization", "abilities": ["ability.infernal_lore"] }`
added to `virtue.demonic_blood`. No classification change — it is already
`creation_effect`.

**The pattern this completes, stated no wider than checked.** Three entries in
this report (F-476, F-477, and the two in-span ones F-460/F-461) use one of two
book idioms — "may learn X during character creation **without needing to take
the Arcane Lore Minor Virtue**" and "**is allowed to have** a score of 1 … at
character creation" — and none of the four is encoded. `grep` over the English
core book for the first idiom alone would give a correction pass its whole work
list, and that is a cheaper route to F-409's remaining population than reading
655 entries again.

**Severity: HIGH.** A character the book describes and the app refuses to build.

### F-478 — a rule the book attributes to `flaw.lycanthrope` by name, from 3,500 lines away, reaches the user in neither locale

> "Those who have an innate supernatural power to change shape (represented by
> Virtues or Flaws such as Shapeshifter and Lycanthrope) only invoke a magical
> effect at the moment of transformation. … However, this type of shapechanger
> is still a human in an animal shape, and so Corpus and Mentem magics are
> effective, as are Animal spells. **An exception is the lycanthrope, who does
> not retain his human mind when transformed, and therefore cannot be affected
> by Mentem spells, although Corpus spells still work.**"
> — ArMDE:9906, in the blockquoted sidebar `#### Ringing the Changes` (ArMDE:9900)

**An inbound pointer, invisible to the entry's own cited range 6370-6377 and to
the `uncomputed_clauses.rs` screen, which reads only that range.** The sentence
names the Flaw explicitly and carves it out of a rule it has just stated for the
whole class — the same construction B14 confirmed at ArMDE:7397 for
`flaw.lesser_malediction` ("specifically, someone with the Lesser or Greater
Malediction Flaw is beyond the power of Curse-Throwing"), and **not** the
construction B14 *narrowed* F-453 on, where an inbound mention merely illustrated
a general rule. Here the Flaw is the exception, by name.

**Current data.** `flaw.lycanthrope` ships `uncomputed_rule` with its own
three-paragraph passage (ArMDE:6372-6376) as `description` in both locales ✓.
Neither locale contains anything from ArMDE:9906: no Mentem immunity, no Corpus
carve-out, and nothing from the sidebar's other Lycanthrope-relevant statement
(no Penetration needed for mundane attacks, no magic radiated, no Warping from
the transformation).

**Why it is wrong.** Per **D5**, any mechanical clause the engine does not
compute must be written into `description`, in both locales, whatever the
classification. Immunity to a whole Form of magic is as mechanical as a clause
gets, and the engine expresses nothing of the kind — Magic Resistance is
per-Form but is a *resistance total*, not an immunity, and no effect makes one
Form ineffective against the bearer. So the rule is inexpressible (D3-shaped) and
owed as text, and the text does not have it.

**One inbound restatement was checked and is *not* a finding**, which is what
keeps this one honest: ArMDE:18641 says "Lycanthropes are an exception to this —
when transforming back into human form, all wounds taken while an animal are
healed, although wounds suffered as a human remain." That restates ArMDE:6376's
own healing clause, which **is** in the shipped description in both locales ✓, so
it adds nothing and is rated and discarded.

**Correct value.** ArMDE:9906's Lycanthrope-specific clauses appended to
`flaw.lycanthrope`'s `description` in **both** locales, cited to ArMDE:9906 (DE
9906, the file being line-parallel). No classification change.

**Severity: MEDIUM-HIGH.** A lost rule in both locales, on a Major Supernatural
Flaw whose whole point is what happens in animal form. It is the fourth
consecutive batch in which a real defect was reachable only through a sentence
outside the entry's own range — see F-474's note on the pattern.

### F-479 — `virtue.mendicant_friar`'s **English** summary is cut after five words, at "St."

> "You are a follower of **St. Francis or St. Dominic** going among the rich and
> poor, spreading the word of God and giving comfort to the sick, homeless,
> hungry, or dying."
> — ArMDE:4490

**Current data.** `rules/i18n/en/virtues_flaws.json` → `virtue.mendicant_friar`:

```
"summary": "You are a follower of St."
```

Twenty-five characters, cut at the abbreviation, leaving a sentence that says the
character follows a saint without naming one. The **German** summary for the same
entry is the complete sentence, 208 characters — *"Du bist ein Anhänger des
heiligen Franziskus oder des heiligen Dominikus, unterwegs unter Reichen und
Armen, …"* — so the two locales are not merely different lengths; only the
English one is broken.

**Outside B15's span, and found by this batch's scan rather than by a pointer.**
`virtue.mendicant_friar`'s `source.lines` is `[4488, 4495]`, which is **B06**'s
range (ArMDE:4351-4597). B06 did not report it. It is carried here because it has
**one cause with F-471 and F-472** — a summary extractor that reads an
abbreviation's period as a sentence end — and the three must be corrected
together or the fix will be written twice.

**Why it matters more than its two German siblings.** F-471 and F-472 damage the
German locale, where a reader can at least tell that something is missing from a
sentence that stops at `(z.B.`. This one is **grammatical**: "You are a follower
of St." parses, so nothing about it looks broken, and it has therefore been
shipping invisibly. It also disproves the comfortable assumption that this defect
is a German-source problem: the extractor is locale-blind and the English book has
abbreviations too.

**And it is why the *right* scan is a ratio, not a vocabulary.** My own scan
looked for a summary ending in a known abbreviation or carrying an unclosed
parenthesis, and it found the two German rows because I had guessed `z.B.`. It
would have missed this one, because I did not think of `St.`. The scan that finds
all three without guessing is **EN/DE summary length ratio above 2×**, which
measures the locales against each other rather than against a word list; run
catalogue-wide it returns exactly the three truncations plus one benign row
(`virtue.tough`, legitimately "+3 to Soak."). That scan is recorded in this
batch's Method so the next batch inherits the method and not just the result.

**Correct value.** The complete first sentence, matching ArMDE:4490:

```
"You are a follower of St. Francis or St. Dominic going among the rich and poor, spreading the word of God and giving comfort to the sick, homeless, hungry, or dying."
```

**Severity: MEDIUM.** A localization defect in the *source* locale, on a Social
Status Virtue, hit by every English user on every visit to the entry — and
`CLAUDE.md` rates localization defects up. Not higher because no rule and no
number is lost.

### F-480 — the canonical glossary carries two rows for one English headword, and the second one states rules mechanics

> `| Monstrous Blood | Monströses Blut | |`
> — `rules/source/de/translation-tables/tugenden-fehler.md:331`

> `| Monstrous Blood | Monströses Blut | SdM:M; Klein; −1 Alterungswurf aber nachteilige körperliche Eigenschaft |`
> — `rules/source/de/translation-tables/tugenden-fehler.md:346`

**Two defects in one pair of rows, and neither depends on the source-project
question D6 raises.**

1. **A duplicate headword in a lookup table.** `CLAUDE.md` makes
   `rules/source/de/translation-tables/` the **canonical** EN→DE mapping, and the
   rule it states is a function: *"the German label for any term whose English
   form appears in a table MUST match the table's `Deutsch (DE)` value"*. Two
   rows for one key is a maintenance hazard even when they agree, because the
   next edit to one of them silently forks the mapping — and the two rows are
   already *not* identical, since `:346` carries an `Anmerkung` and `:331` does
   not. The German value happens to agree today (*Monströses Blut*, which matches
   DE 6454 and the shipped `rules/i18n/de/` name ✓), so nothing is wrong in the
   data **yet**. This is the faucet, not the puddle — which is exactly D6's
   stated reason for treating a table error as urgent.
2. **`:346`'s `Anmerkung` states rules mechanics in a terminology table.**
   *"Klein; −1 Alterungswurf aber nachteilige körperliche Eigenschaft"* is a
   magnitude claim and an effect claim. Per **D6** the tables are authoritative
   for terminology and are **not** authoritative for facts about the rules, and
   this is the error shape the audit has now met seven times. Both claims happen
   to be correct against ArMDE:6455-6456 — so, unlike the four provenance notes
   in **Q-121**, this one is not a *wrong* claim; it is a claim the table has no
   business making, which is why it is a finding where those remain a question.
   (It also spells its minus as **U+2212**, where the repo's convention is the
   ASCII hyphen — `CLAUDE.md`'s rule is scoped to *displayed* signs and a
   Markdown table is not displayed by the app, so that is noted rather than
   counted.)

**Why this is not a D6 "this repository's copy disagrees with the rulebook"
case.** No row here disagrees with anything: the German value is right, and the
mechanics `:346` states are right too. The defect is the table's *shape* —
duplicate key, out-of-scope column content — which is visible entirely within
this repository and needs no comparison against `arm-de-translation`. Whether the
duplicate also exists in the source project is still worth the orchestrator's
check, because D6 requires a table fix to land in both.

**Correct value.** One row for `Monstrous Blood`, with the `Anmerkung` reduced to
whatever is genuinely terminological (gender/number, disambiguation) and the
magnitude and effect claims removed. The provenance fragment *SdM:M* is the
subject of **Q-121** and should be settled with the other four rather than
changed here.

**Severity: MEDIUM.** No shipped string is wrong today. Rated at MEDIUM rather
than LOW because a canonical mapping with a duplicate key generates future wrong
data by construction, which is the argument D6 itself makes for fixing table
errors immediately instead of queueing them.

## Open questions

### Q-118 — is `flaw.monastic_vows_hermetic`'s "you cannot own vis" a mechanical rule, given that the engine models no vis at all?

> "You have taken vows of poverty, chastity, and obedience to a religious
> superior, which means that **you cannot own vis and must only possess
> functional magic devices**. You cannot marry, and **many magi might interpret
> that as prohibiting binding a familiar**. You must do what your master commands
> in service of your order, though in return you can expect aid and assistance
> when needed."
> — ArMDE:6452 / DE 6452

The entry is `narrative` and the whole first clause sits inside sentence one,
so it *does* ship, in both locales, as the summary. Nothing is lost to the
user. The question is only whether `narrative` — "the book states **nothing
mechanical**" — is true of this passage, and the three clauses answer
differently:

- **"cannot own vis"** — vis is a first-class game resource with its own rules,
  but `types.rs::Entity` has **no vis field of any kind**, so there is nothing
  for the prohibition to attach to and nothing it could ever be checked against.
  Under **D3** an engine that structurally cannot express a rule is grounds for
  `uncomputed_rule`, never for `narrative` — *if* the clause is a rule.
- **"must only possess functional magic devices"** — this one constrains a
  collection the engine **does** hold, `Entity::devices`, budgeted by
  `A19 ItemLevelBudget`. "Functional" is not a property the device model
  records, so it is inexpressible today, but it is a restriction on modelled
  data rather than on nothing.
- **"many magi might interpret that as prohibiting binding a familiar"** —
  hedged twice, and `Entity::familiar` is modelled, so this is the one clause
  that could be encoded and plainly should not be.

**What would settle it:** a ruling on whether a prohibition on acquiring a
resource the app does not model counts as "mechanical" for `classification`
purposes. The precedent matters well beyond this entry — the Flaws block
contains several vows, oaths and poverty clauses of the same shape — and it is
the kind of line the audit must not draw by judgement, which is why it is here
and not in the Findings. **Not marked checked.**

**The verification pass would resolve it toward `uncomputed_rule`** and its
argument is recorded so the ruling has it: `types.rs::Classification::Narrative`
says the passage must state *"no mechanical clause … **at all**"*, and D3's
precedent is that structural inexpressibility is grounds for `uncomputed_rule`
and **never** for `narrative`. "You cannot own vis" is an absolute about a
game resource. I have **not** adopted that resolution, because the premise it
turns on — whether a resource the app does not model can carry a "mechanical
clause" for classification purposes — is exactly the precedent at issue, and
this audit's standing rule is that a reading which cannot be settled from the
source is escalated rather than taken.

### Q-119 — is `flaw.necessary_condition`'s "you cannot cast spells at all" a mechanical absolute or a fiction condition?

> "In order for your magic to work, you must perform a specific action while
> casting any spell. This should be something simple, such as singing or
> spinning around three times. **If you cannot perform the action, you cannot
> cast spells at all.**"
> — ArMDE:6478 / DE 6478

`classification: "narrative"`, no `effects`, and the shipped summary is sentence
one in both locales — so the absolute reaches the user nowhere. The sentence is
as flat an absolute as ArMDE:6364's "never have any Confidence Points", which
B14 treated as a rule (F-459); but its antecedent ("if you cannot perform the
action") is a fact about a scene, not about a character sheet, so there is no
state the engine could ever evaluate it against and no number it could ever
change. Against that: the audit's own definition of `narrative` is a claim about
the **rulebook**, not about the engine, and the rulebook plainly states a
mechanical consequence.

**What would settle it:** a ruling on whether a mechanical consequence with a
purely narrative antecedent is "something mechanical" for `classification`.
Note the practical stake, which is not the class but the text: if the answer is
`uncomputed_rule`, D5 obliges the sentence into `description` in both locales,
and a player choosing this Major Hermetic Flaw is currently told only that he
must perform an action, never what happens if he cannot. **Not marked checked.**

**The verification pass would resolve this one toward `uncomputed_rule` too**,
and calls it the sharper of the pair because — unlike Q-118's clause — the
absolute is **not** in sentence one, so it reaches the user in neither locale.
Recorded, not adopted, for the same reason as Q-118.

**And the entry sits at one end of a three-way exclusion the catalogue does not
model, which belongs in this question's record.** ArMDE:7009
(`flaw.vulnerable_magic`, **B19**'s span) says: "This Flaw may be taken multiple
times, so long as a different condition is specified for each. **It may not be
combined with Restrictions or Necessary Conditions that have the same (or
equivalent) conditions.**" `flaw.vulnerable_magic` ships `narrative`, with **no
`parameters`, no `max_total` and an empty `incompatible_with`** — so it carries
neither the multiplicity permission the sentence grants (which needs a
`condition` parameter it does not have) nor the exclusion; and `flaw.restriction`
(ArMDE:6691-6694) is likewise `narrative` with an empty `incompatible_with`. The
exclusion is *condition*-scoped, and `flaw.necessary_condition` records its
condition nowhere at all — it has no parameter — so even a correction pass
willing to write `incompatible_with` could only over-exclude. **A B19 lead, and
the reason this question cannot be closed by simply adding a declaration.**

### Q-120 — CLOSED by the enum's own doc comment: an entry computing in both phases is `creation_effect`

**F-461** recommends adding an `ability_authorization` to `flaw.monstrous_blood`,
which already carries an `aging_mod`. The entry would then change a
character-creation state **and** move an in-play total.
`types.rs::Classification`'s own doc defines `in_play_effect` as "**does not
change a creation number**, but modifies an in-play/derived total", and
`creation_effect` as "changes a character-creation number or state" — so read
literally the pair is exclusive and `creation_effect` is forced, which is what
the verdict table records. But that answer silently discards the true statement
that the entry also computes in play, and `classification` is read by no
production code, so nothing depends on the choice except what a later reader
believes.

This is not hypothetical beyond this entry: `data_integrity.rs::every_vf_is_classified`
asserts that **every `in_play_effect` entry carries at least one effect** and
makes no converse assertion for `creation_effect` (Part C's **C7**), so moving
an entry out of `in_play_effect` also moves it out from under the only guard
that would notice if its effects were later emptied.

**Closed, on evidence rather than by ruling.** The verification pass read
`types.rs::Classification` and the doc is not ambiguous: `InPlayEffect` is
defined as *"**Does not change a creation number**, but modifies an
in-play/derived total the engine computes"*, and `CreationEffect` as *"Changes a
character-creation number or **state**"*. An `AbilityAuthorization` is
creation-time state and `validate_ability_authorization` runs at
`CreationPhase::Abilities`, so the moment the effect is added `in_play_effect`
becomes **false by its own definition** and the taxonomy admits exactly one
answer. **`flaw.monstrous_blood` → `creation_effect`**, as the verdict table
already records. No ruling is needed and none is requested.

**One consequence the correction pass must carry, which is why this is recorded
rather than deleted.** Part C's **C7**:
`data_integrity.rs::every_vf_is_classified` asserts that every `in_play_effect`
entry carries at least one effect and makes **no** converse assertion for
`creation_effect`. Moving this entry therefore also moves it out from under the
only guard that would notice if its `aging_mod` were later emptied — a real, if
small, loss of coverage, bought by a classification that is now correct.

### Q-121 — four terminology-table rows attribute core-book Flaws to supplements; at what count does "noted, not counted" become a finding?

| row | English | claim in `Anmerkung` | where the core book carries it |
|---|---|---|---|
| `tugenden-fehler.md:376` | Lycanthrope | *HdH:S* | ArMDE:6370 |
| `tugenden-fehler.md:377` | Meddler | *GotF* | ArMDE:6422 |
| `tugenden-fehler.md:346` | Monstrous Blood | *SdM:M; Klein; −1 Alterungswurf aber nachteilige körperliche Eigenschaft* | ArMDE:6454 |
| `tugenden-fehler.md:444` | Offensive to (Beings) | *SdM:M; Klein; magische Präsenz stört einen Wesentyp besonders* | ArMDE:6524 |

Per **D6** a terminology table is authoritative for *which German word renders
which English term* and is **not** authoritative for *facts about the rules*,
and a provenance attribution is a fact about the rules. B13 and B14 each found
this shape and each recorded it as "noted, not counted"; B14 found **two**
(`tugenden-fehler.md:687` Hermetic Patron → *HoH:TL*, `:330` Hunger for (Form)
Magic → *SdM:M*). This batch finds **four** in 35 entries, bringing the known
total to **six**, and two of the four go further than a book name: `:346` and
`:444` restate the Flaws' *mechanics* in a terminology table, which is the same
category error one step worse.

**The counter-argument, which is why this is a question and not a finding.** A
Virtue or Flaw can legitimately appear in a supplement *and* in the core book —
the Definitive Edition consolidates material — so an `Anmerkung` naming *SdM:M*
may be recording where the translator first met the term rather than claiming
the core book does not carry it. Three of the four supplements named are in this
repository (*RoP:M* = *SdM:M*, *HoH:S* = *HdH:S*); *GotF* is the Rhine Tribunal
book, which has a German source here and **no English one**, so a Flaw
attributed to it could not be checked against an English original at all. I
cannot settle whether any of the four claims is false without reading the named
supplements for these entries, which is a larger read than a name check.

**The verification pass sharpened the counter-argument and it now looks decisive
enough to be worth stating as the likely answer.** The notes never claim
*exclusivity*: a Flaw can be in a supplement **and** in the Definitive Edition,
which consolidated supplement material, so *"SdM:M"* is plausibly a record of
where the translator met the term rather than an assertion about where the rule
lives. On that reading the four rows are **incomplete rather than false**, and
"noted, not counted" remains right.

**What would settle it:** either a ruling that an `Anmerkung`'s book reference is
documentation rather than a claim (in which case all six known instances close
and no future batch reports them), or a pass that checks the named supplements
for these four Flaws. **No entry is marked unchecked on account of this** — every
shipped German *name* in this batch is correct and agrees with the rulebook
heading, so nothing the tables are authoritative for is in dispute.

**One row of the four is nevertheless a finding on other grounds and has been
split out.** `tugenden-fehler.md:346` is a **duplicate** of `:331` for the same
English headword, and its `Anmerkung` states a magnitude and an effect — see
**F-480**. That defect is visible entirely inside this repository, does not turn
on the provenance question, and does not need the source project to settle it.

### Q-122 — should `flaw.magical_fascination`'s authorization name both Lores, or record the player's choice?

> "he is allowed to have a score of 1 (but no more) in **either Magic or Faerie
> Lore** at character creation"
> — ArMDE:6394 / DE 6394 ("in Magie- **oder** Feenkunde")

**F-460**'s correction needs an `ability_authorization`, and the shape is not
obvious. `A14 AbilityAuthorization` takes flat `abilities` / `categories` lists
with no either/or:

- `{ "abilities": ["ability.faerie_lore", "ability.magic_lore"] }` is one line
  and is **more permissive than the passage** — it legalises a character with
  Magic Lore 1 *and* Faerie Lore 1, which "either … or" does not obviously
  grant.
- A `parameters` entry of domain `ability` with an `enumerated`-style restriction
  would record which one the player chose, but `ParameterDomain::Ability`
  resolves against the whole Ability catalogue and has no `values` list
  (load-time integrity rejects `values` on any domain but `enumerated`), so the
  closed two-way choice is not directly expressible either; and no effect reads
  a parameter to *authorize* an Ability — `A14`'s lists are stored directly, not
  parameter-relative. So the narrow reading would need a new effect shape.

The same question governs **F-461**'s fix only trivially (Monstrous Blood names
one Ability, so `{ "abilities": ["ability.magic_lore"] }` is exact), which is
what isolates the issue here.

**What would settle it:** a reading of whether "either A or B" in this idiom
means "one of the two, your choice" or merely "both are opened to you"; and, if
the former, a decision on whether it is worth a parameter-relative
authorization shape or whether the over-permissive two-id list is acceptable
given that the cap ("but no more") is inexpressible anyway and travels as text.
**`flaw.magical_fascination` is marked as a failure on F-460 regardless; this
question governs only the fix.**

### Q-123 — two entries state a *soft* restriction on magi, and the engine has only hard blocks

> "This Flaw normally makes seasonal Laboratory work impossible, and so is **not
> suitable for magi**."
> — ArMDE:6494 (`flaw.night_terrors`)

> "Magi are **forbidden** from taking Oaths of Fealty by the Hermetic Code.
> **Some don't let that stop them.**"
> — ArMDE:6514 (`flaw.oath_of_fealty`)

Both are restrictions aimed at one character type, and both are explicitly
softened — the first by "normally" and "not suitable", the second by a sentence
that permits breaking it outright. The engine's two mechanisms are both hard:
`entity_kinds` (a `wrong_entity_kind` **error**, and there is no `magus` entity
kind anyway — the split is by type *profile*, not by `EntityKind`) and the
profile's `forbidden_traits` (also an error). There is no "discouraged" level
between them, and `validate_house`'s ArMDE:2860 Hermetic-Flaw guideline is the
only soft, profile-scoped check in the codebase — a **warning** keyed on
categories, not on an id.

`flaw.night_terrors` carries its clause in `description` in both locales ✓;
`flaw.oath_of_fealty` does not — its Code sentence is sentence three, and the
shipped text is sentence one in both locales, so that half of the pair has a D5
exposure the other does not.

**What would settle it:** a ruling on whether a soft type restriction should be
modelled at all — as a warning-level `discouraged_traits` on the profile, as
text only, or not at all — and, if text only, whether `flaw.oath_of_fealty`'s
Code sentence is owed in `description` (which turns it into a plain D5 finding
against a `narrative` entry, and therefore also into a classification question:
is "magi are forbidden" mechanical when the same sentence says some do it
anyway?). **Both entries are recorded as escalated, not checked.**

## Sub-agent reconciliation

An independent verification sub-agent re-derived all 19 entries this file calls
clean, all of the findings, all twelve Method censuses, the three withdrawals and
the six open questions, from the source and the code rather than from my
reasoning. **It overturned no clean verdict and withdrew no finding.** It
narrowed one finding, corrected the proposed fix on another, strengthened a third
with evidence I did not have, caught one census gap, closed one open question,
and contributed two findings of its own — one of which is the batch's third
truncated summary and is in **English**, which falsified a claim in my Method.
Everything below is folded into the sections above; this section records the
delta so it is auditable.

### No verdict changed, and that is itself worth stating

All **19** clean entries were re-derived and all 19 hold. Three that the brief
singled out for attack held on evidence the pass produced independently:

| Entry | Why it was attacked | What the pass found |
|---|---|---|
| `flaw.mute` | my whole D3 argument for **F-464** rests on its -10 being genuinely inexpressible | **Confirmed, with the arithmetic.** `Effect::CastingTotalMod` feeds `InPlayMods::casting_mods` → `casting_mod_for(cast)` → the **base** Formulaic/Ritual/Spontaneous figures, while `SpecialCasting::QuietWords` feeds `InPlayMods::voice_reduction`, read by `residual_voice_penalty` **only**, which surfaces on `NonStandardCasting::silent`. So `casting_total_mod { -10, all }` would be un-offsettable by Quiet Magic **and** would double-count: the `silent` column would read base(-10) + residual(-10) = **-20**. The rejection stands and F-464 stands with it. |
| `flaw.necessary_realm_aura_for_ability` | I called `max_per_value: 1` "exactly" ArMDE:6482 | **Confirmed, by tracing both cases through `validate_per_value_cap`.** Same Ability + different realms → two distinct param maps, `counts[(ability, None)] = 2 > 1` → `too_many_for_param_value` ✓. Different Abilities + same realm → no error from either validator ✓. |
| `flaw.manifest_sin` | `tainted: false` against ArMDE:6406's "either the Divine or the Infernal" | **Confirmed, with the consequence.** ArMDE:3000 defines Tainted as Infernal-**associated**; the descriptor and the index (ArMDE:5549) both omit the tag; and `tainted: true` would feed `validate_tainted_cap`'s `2·tainted > total` ratio and raise a spurious `too_many_tainted_flaws` warning. Following the descriptor is right. |

The pass also added a fact that is **load-bearing for every D5 finding in this
batch** and which I had relied on without citing:
`uncomputed_clauses.rs` builds displayed text as `description` **if present, else
`summary`** — "exactly the precedence `VirtueFlawTab.svelte`'s tooltip applies".
That is what vindicates `flaw.missing_ear`'s missing description, and it means
every D5 finding must be judged against `description ?? summary`, not
`description` alone. Re-checked on that basis: F-461, F-463, F-464, F-467, F-468,
F-469 and F-470 each have the clause in **neither** field. All stand.

### The findings

All **CONFIRMED**. Four carry a change:

- **F-466 — the proposed fix was wrong and is corrected.** I wrote
  `incompatible_with` on both entries. `virtue.lone_redcap` carries
  `grants_selection: { items: ["virtue.well_traveled"] }` (ArMDE:4321), and
  `validate_incompatibilities` reads the bought-only `selected_ids` on **both**
  sides — so a Lone Redcap's *granted* Well-Traveled would escape the very
  declaration I asked for, on what is probably the commoner build. The remedy is
  `Prereq::Nor([Has(virtue.well_traveled)])`, whose `present_ids` is
  grants-inclusive. **A correction pass taking my first wording would have
  shipped a guard that misses half its cases** — the same class of error B14's
  pass caught on F-446's proposed parameterization. My Method's Part C bullet,
  which explicitly warned a correction pass *away* from `Prereq::Nor`, was right
  for F-463 and F-465 and wrong for F-466; it is now split.
- **F-469 — narrowed, and the narrowing is exact.** I wrote that two Magical
  (Being) Companions are "two tuples and legal". `validation/caps.rs::validate_caps`
  counts **selections**, not distinct items, so two copies do raise
  `too_many_story_flaws` — but that cap carries no `"hard": true`, so it is a
  **warning**. The precise claim is: no blocking check and nothing that names
  this item; only the generic Story-cap warning any two Story Flaws would raise.
- **F-462 — strengthened, with git evidence I did not gather.**
  `git log -S"missing_eye's ranged"` puts RULES.md's claim into `acc1657`, and
  `git show acc1657:rules/core/virtues_flaws.json` shows the entry already
  carried `{ -1, attack }` at that same commit. So the line was **false at
  birth**, not drifted — a sharper statement about the traceability map than the
  one I made. The pass also priced the remedy: seven `weapon`-scoped rows, with
  an eighth missile weapon later falling back silently to the melee -1, which
  collides with `CLAUDE.md`'s "catalogue size is data, never code" invariant.
  Both are now in the finding.
- **F-473 — widened with two independent confirmations.** DE 6530 is the **only**
  place in the entire German rulebook using *Magisches Auftreten* (`rg -c` → 1);
  every other DE reference — the heading DE 6382, the index DE 5411, the blatant
  variant DE 5715, the book's own index at DE 24317/25072 — uses *Magische
  Ausstrahlung*. So the defect originates in the German **source**, not in the
  extraction, and the shipped German file is additionally inconsistent with
  itself: `flaw.unbearable_to_beings` and `virtue.alluring_to_beings` both write
  *Ausstrahlung*.

Two independent arrivals are worth recording, because they are the strongest
evidence that either pass could have found them. **F-478** (ArMDE:9906's Mentem
immunity) was found by both of us separately, from the same line, with the same
D5 reading and the same B14 precedent for distinguishing it from an illustrative
mention. And **F-477**'s inbound pointer at ArMDE:7009 was rated by both, with
the same conclusion that the exclusion is condition-scoped and inexpressible.

### One census correction

Eleven of the twelve censuses matched exactly. The twelfth is the sign census
(**number 6**), and the history is worth recording because it failed twice in two
different ways:

1. My **first draft** had ArMDE:6462 and :6528 on the ASCII side and omitted
   :6474 — the ugrep-shim failure `CLAUDE.local.md` warns about and B14 walked
   into. I caught it myself with `rg` before the pass ran and corrected the table.
2. The **corrected** table was still incomplete, and the pass caught that: it had
   rows for "en dash + digit" and "sign + space + digit" but **none for an en
   dash not followed by a digit**, so it silently dropped **ArMDE:6390**
   (`10 – Size`). That line is in `flaw.magical_being_companion` — the very entry
   **F-469** is about, and one of two lines a correction pass must convert to
   ASCII when it writes the clause into `description` (the other is ArMDE:6462,
   in F-461's text). A fourth row now carries them.

Everything else re-derived and matched: the 22/9/4/0 classification census; 32
headings with byte-identical EN/DE line sequences (the pass re-checked with
`^\s*>?\s*#{3,6} `, which also rules out a blockquoted heading my `^#### ` scan
would have missed); 35/35 `source.lines`; 9 anchors coinciding exactly with the 9
`uncomputed_rule` entries; 10 descriptions in both locales; zero U+2212 and zero
en-dash-sign in the shipped files; 14 table rows and 18 without; 2 + 10 permission
carriers; 7 Story instances and 0 Social Status; 62 of 335 `narrative` entries
with an `incompatible_with`; and the three parameterized entries. One prose slip
was caught and fixed: "the three multi-paragraph ranges" introduced a list of
**five**.

### Two findings the pass contributed, and one question it closed

- **F-479** — `virtue.mendicant_friar`'s **English** summary is
  `"You are a follower of St."`, truncated at the abbreviation, while its German
  summary is complete. It falsifies my Method's claim that the truncation census
  is "exactly two rows and both are German": it is **three**, and the source
  locale is not immune. More usefully, it produced the *right scan*: an **EN/DE
  summary length ratio above 2×** finds all three without guessing which
  abbreviation is involved, where my abbreviation-and-parenthesis scan found only
  the two I had thought of. Four further scans returned zero rows each and are
  recorded in the Method so the next batch skips them.
- **F-480** — `tugenden-fehler.md:331` and `:346` are two rows for the same
  English headword `Monstrous Blood`, and `:346`'s `Anmerkung` states a magnitude
  and an effect. A duplicate key in the repo's canonical EN→DE mapping, plus a
  terminology table making a rules claim. I had read both lines and counted only
  the provenance fragment toward **Q-121**, missing the duplication entirely.
- **Q-120 is closed**, on `types.rs::Classification`'s own doc rather than on a
  ruling: `InPlayEffect` is defined as "does **not** change a creation number",
  so an entry that gains an `ability_authorization` is `creation_effect` by
  definition. I had escalated a question the code already answered.

### Two resolutions offered and deliberately not adopted

The pass would resolve **Q-118** and **Q-119** toward `uncomputed_rule`, arguing
from `Classification::Narrative`'s "no mechanical clause … **at all**" and from
D3's precedent. The argument is recorded inside both questions. **I have not
adopted it**, and the reason is the audit's own standing rule rather than doubt
about the reasoning: the premise both turn on — whether an absolute about a
resource or an event the app does not model is a "mechanical clause" for
classification purposes — *is* the precedent being set, and it would govern
several more vows, oaths and condition clauses in the remaining four batches.
That is a ruling, not a reading, and this audit escalates rather than takes them.
This is the mirror of B11's refused withdrawal: a verification pass's conclusion
is evidence, and evidence does not decide a question whose content is a policy.
