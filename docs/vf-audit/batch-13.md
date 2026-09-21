# Batch B13 — indices 420-454, ArMDE:6068-6235

Entries: 35. Audited: 35. Failures: **15**. Clean: **20**.
Findings **F-428 … F-444** — several reach outside the batch: **F-432**'s second
half lands on `virtue.tethered_magic` (B09's span), **F-433** names three further
name collisions outside it, **F-434** rests on `virtue.wealthy` and `flaw.poor`,
**F-439** is wholly outside (it is F-428's shape found a second time, on
`virtue.lone_redcap` in **B05's** span, by the census F-428 obliged me to run),
**F-442** carries `flaw.lesser_malediction` from B15's span with it, and
**F-444** is a defect in `docs/vf-audit/decisions.md` itself.
Open questions **Q-103 … Q-110**.

**The verification sub-agent re-derived all 24 entries this file first called
clean and overturned two — `flaw.fickle_nature` and `flaw.gabai` — escalated
three more, and caught an arithmetic error in this file's own headline count.**
It confirmed all six high-stakes findings (widening F-428, narrowing F-433's
census), corrected two Method claims, contributed five observations this file had
missed, and **found the batch's second live false hard error** on an entry I had
passed on all twelve checks. Of the five entries whose verdicts changed, four are
now findings and one is an open question. See the reconciliation section.

**The batch's first shape is a Flaw that *replaces* a life-stage block being
encoded as one that *adds* to it.** `flaw.feral_upbringing`'s 120 experience
points are the same 120 the standard childhood block already grants
(ArMDE:2378 = 75 + 45), over the same five years, spent on a replacement
Ability list — and the engine stacks the Flaw's pool on top of the childhood
pools, so a life-stage character with this Flaw carries **240** first-five-years
XP instead of 120, including the 75 Living Language points the Flaw's own text
forbids (**F-428**). This is the batch's only miscalculation and its highest
severity: a wrong number on the sheet, not a lost rule.

**The second shape is `narrative` on a rule stated entirely in words, again,
and again inside the twice-swept block.** Six of this batch's nineteen
`narrative` entries state a mechanical clause and none of the six carries a
number the screen can see: `flaw.harmless_magic` disables a whole Technique,
`flaw.fettered_magic` makes every one of a magus's spells an Arcane Connection
to him, `flaw.false_power` (×2) moves a Virtue onto a second realm's
interaction column, `flaw.fluctuating_fortune` alternates two named catalogue
entries year by year, and `flaw.greater_malediction` carries a named immunity —
stated, as it turns out, in **another entry's** passage, which is a gap the
screen structurally cannot close. B10 found nine of this shape, B11 seven, B12
eleven; the screen is green on all six of mine.

**The fourth shape is the one this file missed on its own and the verification
pass found: an entry can be perfectly authored and still be unbuildable.**
`flaw.fickle_nature` is classified right, carries its rule in both locales, and
mandates two Personality Traits at +4 — which raises **two hard
`personality_trait_out_of_range` errors**, because the validator grants a ±4 slot
only per *Major* Personality Flaw and this one is Minor (**F-440**). Underneath
it the rulebook contradicts itself: ArMDE:2502 caps a Minor Personality Flaw's
trait at ±3. Nothing in an entry-versus-passage comparison can see this; the
question that finds it is *what happens when a player does what the entry says*.

**The third shape is B12's own two idiom families turning up again, one each.**
`flaw.failed_monk` — the entry B12 rated only as a comparison point for F-408 —
states the **permission** idiom ("You may take Academic Abilities during
character creation") and carries no `ability_authorization`, so it is a
twenty-ninth instance of F-409 and a live false hard error (**F-429**). And
`flaw.feral_upbringing` states the **prohibition** idiom ("you may not start
with a score in a Language"), which F-410/F-426 established the engine cannot
express at all.

*(Written incrementally: header, method and the 35-row verdict table first,
findings appended one at a time, sub-agent reconciliation last.)*

## Method

**Both languages were read as continuous prose before any entry was judged** —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` 6060-6249 and
the line-parallel `rules/source/de/Ars Magica Definitive Edition Basisregeln.md`
6060-6249, deliberately overrunning the span at both ends so the first and last
entries' boundaries could be seen against their neighbours
(`flaw.failed_journeyman` 6060 and `flaw.failed_master` 6064 before,
`flaw.hatred` 6236 / `flaw.hedge_wizard` 6240 / `flaw.heir` 6244 after).

**Line parity holds throughout the span, and through the overrun.** Every one of
the 31 `####` headings in 6068-6235 sits on the identical line number in both
files (6068, 6072, 6076, 6080, 6098, 6102, 6106, 6110, 6114, 6118, 6122, 6126,
6134, 6142, 6146, 6150, 6154, 6158, 6162, 6186, 6190, 6194, 6198, 6202, 6206,
6210, 6214, 6218, 6222, 6226, 6230 — plus the nested example-table heading at
6170, which is inside a blockquote and so is not a sibling), and so do the two
before the span (6060, 6064) and the four after (6236, 6240, 6244, 6248). The
two heading lists were compared mechanically as well as by eye: `grep -n "^#### "`
over each file, restricted to 6060-6298, returns **byte-identical** line-number
sequences. 31 headings for 35 catalogue ids: the three Major/Minor pairs
`Gender Nonconforming` 6202, `Generous` 6206 and `Greedy` 6214 each ship as two
ids citing one heading, and `False Power` 6080 ships as a Major + Minor pair
citing one heading for a *different* reason (ArMDE:6096 — see F-431 and
`RULES.md:1790-1822`).

**The batch's own classification census matches the brief's.** Re-derived rather
than trusted, per B12's warning: `jq` over the 35 ids returns **19 `narrative`,
7 `uncomputed_rule`, 6 `creation_effect`, 3 `in_play_effect`** (19+7+6+3 = 35).
The brief's figures are correct this time.

**Check 1 — `source.lines`: 35 of 35 correct.** Every range runs from its own
`####` heading to the line before the next heading. Two are worth naming because
they look wrong and are not: `flaw.false_power` / `flaw.false_power_minor`
6080-6097 spans eighteen lines because the passage has six paragraphs and three
indented examples before the next heading at 6098; `flaw.form_monstrosity`
6162-6185 spans twenty-four because it swallows the nested blockquote table
`> #### Monstrosity Examples` (6170-6184), whose `####` is inside a blockquote
and is therefore not a sibling heading. Both end on the line before the next
top-level heading ✓. As in B12, several ranges end on a blank line
(6071, 6075, 6079, …), which is this audit's check-1 convention and the subject
of B12's **Q-97**; no new instance is counted.

**Check 1, second half — `source.anchor`: 7 of 35 entries carry one, all seven
correct**, verified twice: derived from the `####` heading, and against the
book's own "List of Flaws" index links, which spell them out literally.

| id | anchor | heading | index link |
|---|---|---|---|
| `flaw.the_falling_evil` | `the-falling-evil` | ArMDE:6076 `#### The Falling Evil` | ArMDE:5408 |
| `flaw.fickle_nature` | `fickle-nature` | ArMDE:6122 | ArMDE:5479 |
| `flaw.flashbacks` | `flashbacks` | ArMDE:6134 | ArMDE:5584 |
| `flaw.flawed_parma_magica` | `flawed-parma-magica` | ArMDE:6142 | ArMDE:5437 |
| `flaw.flawed_powers` | `flawed-powers` | ArMDE:6146 | ArMDE:5543 |
| `flaw.fury` | `fury` | ArMDE:6194 | ArMDE:5358 |
| `flaw.gullible` | `gullible` | ArMDE:6222 | ArMDE:5587 |

Six of the seven are the batch's `uncomputed_rule` entries; the seventh is
`flaw.flawed_parma_magica` (`in_play_effect`). **The one `uncomputed_rule` entry
with no anchor is `flaw.fish_out_of_water_terrain`** — exactly B12's
`flaw.evil_eye` observation one batch on. Not a defect of the entry (B11's Q-96
is the catalogue-wide question), but worth the line: the anchor that *would* be
correct for it is `fish-out-of-water-terrain`, spelled out at ArMDE:5583.

**One index link in this span is broken in the book itself and is nobody's
data defect.** ArMDE:5526 reads `[Failed Monk/Nun](#failed-monknun)`, but the
heading at ArMDE:6068 is `#### Failed Monk`, whose anchor is `failed-monk`. The
entry carries no anchor, so nothing in the data reproduces the error; recorded
so a correction pass adding anchors does not copy the index instead of the
heading.

**Checks 3, 4, 5, 6 (`kind` / `magnitude` / `entity_kinds` / `categories` +
`tainted`): 35 of 35 correct**, each read against its own descriptor line one by
one:

- `*Minor Social Status*` (sic, no comma) — failed_monk (6069)
- `*Minor, Social Status*` — gabai (6199)
- `*Minor, General*` — failed_student (6073), feral_scent (6107), feral_upbringing (6111), fish_out_of_water_terrain (6127), flashbacks (6135), fragile_constitution (6187), frail (6191), gullible (6223), hallucinations (6227)
- `*Major, General*` — the_falling_evil (6077)
- `*Major, Supernatural, Tainted*` — false_power / false_power_minor (6081)
- `*Major, Supernatural*` — greater_malediction (6211)
- `*Minor, Supernatural*` — flawed_powers (6147), fluctuating_fortune (6151), form_monstrosity (6163)
- `*Major, Story*` — favors (6099), feud (6119), fury (6195)
- `*Minor, Personality*` — fear (6103), fickle_nature (6123), follower (6155), foreign_upbringing (6159), grudge (6219)
- `*Major or Minor, Personality*` — gender_nonconforming_\* (6203), generous_\* (6207), greedy_\* (6215)
- `*Minor, Hermetic*` — fettered_magic (6115), flawed_parma_magica (6143), harmless_magic (6231)

`kind` is `flaw` on all 35 ✓ — the whole span is inside the book's Flaws block
(ArMDE:5639-7113). `entity_kinds` is `["character"]` on all 35 ✓. **`tainted` is
`true` on exactly the two entries whose descriptor carries the tag**
(`flaw.false_power`, `flaw.false_power_minor`, ArMDE:6081 `*Major, Supernatural,
Tainted*`) and `false` on the other 33 ✓. Two entries invite the mistake and
neither should get it: `flaw.greater_malediction` ("cursed by some supernatural
power" — a curse is not an Infernal association) and `flaw.form_monstrosity`
(whose prose says people "invariably attribute" the mutation "to malign
influence", i.e. what people *think*, ArMDE:6166). `tainted` means
Infernal-realm associated (ArMDE:2998-3002), and following the descriptor rather
than the prose is correct.

**One descriptor line is malformed in the English source and correctly ignored.**
ArMDE:6069 reads `*Minor Social Status*` — no comma, where every other
descriptor in the book has one. DE 6069 has the comma (`*Klein, Sozialer
Status*`). This is the second instance of exactly the shape B12 found at
ArMDE:5973 (`*Minor. Hermetic*`), so the source has at least two such typos. The
data reads it correctly as Minor + Social Status, corroborated by the index
(ArMDE:5526 under `### Social Status, Minor`). A source typo, not a data defect;
noted so nobody "fixes" the data to match it.

**The index confirms every filing, with no contradiction.** All 31 headings are
filed in the book's own "List of Flaws" (ArMDE:5283-5638) under a
`### <category>, <magnitude>` heading matching both the descriptor and the data:

| index lines | heading | entries |
|---|---|---|
| 5320-5322 | `### Personality, Major or Minor` (5310) | Gender Nonconforming, Generous, Greedy |
| 5356-5358 | `### Story, Major` (5340) | Favors, Feud, Fury |
| 5394-5395 | `### Supernatural, Major` (5387) | False Power, Greater Malediction |
| 5408 | `### General, Major` (5401) | The Falling Evil |
| 5436-5438 | `### Hermetic, Minor` (5417) | Fettered Magic, Flawed Parma Magica, Harmless Magic |
| 5478-5482 | `### Personality, Minor` (5467) | Fear, Fickle Nature, Follower, Foreign Upbringing, Grudge |
| 5526-5527 | `### Social Status, Minor` (5520) | Failed Monk/Nun, Gabai |
| 5543, 5544, 5546 | `### Supernatural, Minor` (5534) | Flawed Powers, Fluctuating Fortune, (Form) Monstrosity — **not** a contiguous run: 5545 is `[Folk Magic](#folk-magic)`, which is not a V/F entry |
| 5580-5588 | `### General, Minor` (5564) | Failed Student, Feral Scent, Feral Upbringing, Fish Out of Water (Terrain), Flashbacks, Fragile Constitution, Frail, Gullible, Hallucinations |

No entry in this batch is dual-listed, so there is no `taken_as` case here. The
one index *gap* is the book's, not the data's: `False Power` is listed only
under `### Supernatural, Major` (5394), with no `### Supernatural, Minor` entry
for the subsequent-copy Minor form ArMDE:6096 creates.

**Check 12, mechanically.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the 35 ids in each locale returns the
key set `["name","summary"]` for **28** entries and
`["description","name","summary"]` for **7** — the same 7 in both locales, so the
locales never disagree about *whether* a description exists. The 7 are exactly
the batch's `uncomputed_rule` entries (`the_falling_evil`, `fickle_nature`,
`fish_out_of_water_terrain`, `flashbacks`, `flawed_powers`, `fury`, `gullible`),
each carrying its full cited passage in both languages ✓ — a fact
`RULES.md:4785-4795` independently records for two of them
(`flaw.the_falling_evil`, `flaw.flawed_powers`). **No `narrative`,
`creation_effect` or `in_play_effect` entry in this batch carries a description
in either locale** — which is where every D5 finding below comes from. One
`in_play_effect` entry is an exception worth naming in the other direction:
`flaw.flawed_parma_magica` carries a description in both locales although its
class does not oblige one, and that description is the entry's whole passage
including the repeatability sentence ✓.

**ASCII hyphen (check 12, second half): clean, verified two ways.**
`grep -c "−"` (U+2212) over `rules/i18n/en/virtues_flaws.json` and
`rules/i18n/de/virtues_flaws.json` returns **0** for both, catalogue-wide. And
`grep -o "–[0-9]"` (U+2013 followed by a digit) returns **nothing** in either
file, so no en dash that does ship is a *sign*. The conversion is working in this
batch in both directions: the DE source writes every negative as `–` (DE 6108
`–1`, 6128 `–1`, 6136 `–1 bis –5`, 6188 `–3`, 6192 `–3`, 6196 `–1`, 6200 `–2`)
and each one that ships does so as ASCII `-` (visible in the shipped DE
descriptions of `fish_out_of_water_terrain`, `flashbacks` and `fury`); the
**English** source is itself inconsistent, ArMDE:6128/6136/6188 using an en dash
while ArMDE:6108/6192/6196 use ASCII — yet the shipped EN descriptions carry
ASCII throughout.

**Truncation scan: this batch is clean, and so is the rest of the catalogue bar
B12's one entry.** `jq` over **both** locale files for any `name`, `summary` or
`description` containing `...` or `…` returns exactly **two** rows, and they are
the same id in each: `flaw.exciting_experimentation`, B12's F-415. **No entry in
this batch is truncated**, and the catalogue-wide population of this defect is
one entry — stated exactly as checked: a literal-ellipsis scan over the two
shipped locale files, which would not see a summary truncated *without* an
ellipsis.

**German names against the canonical tables.** Fifteen of the thirty-one
distinct English names in this batch have a row in
`rules/source/de/translation-tables/`, located by English key. **Twelve agree
with the shipped `rules/i18n/de/` name and with the DE rulebook heading**; **two
disagree with the rulebook**, in each case because the shipped data follows the
rulebook and the table's copy here does not (D6 — see **F-438**); and one is the
collision case where the rulebook contradicts *itself* and the table is the only
tie-breaker (**F-433**). The remaining sixteen English names have no table row
and were checked directly against the DE rulebook heading on the parallel line —
**all sixteen match ✓**.

| English | table row | DE rulebook heading | shipped DE name | verdict |
|---|---|---|---|---|
| False Power | `sphären-mächte.md:293` *Falsche Macht* | 6080 Falsche Macht | Falsche Macht (Groß/Klein) | ✓ |
| Favors | `tugenden-fehler.md:398` *Gefälligkeiten* | 6098 Gefälligkeiten | Gefälligkeiten | ✓ |
| Fear | `tugenden-fehler.md:369` *Angst* | 6102 Angst | Angst | ✓ |
| Feral Scent | `reputationen.md:110` *Wildgeruch* | 6106 **Wilder Geruch** | Wilder Geruch | data ✓; **table disagrees** — F-438 |
| Feral Upbringing | `tugenden-fehler.md:708` *Wilde Erziehung* | 6110 Wilde Erziehung | Wilde Erziehung | ✓ — and its note records the prohibition the data drops (F-428) |
| Fettered Magic | `tugenden-fehler.md:701` *Gekettete Magie* | 6114 **Gefesselte Magie** — the same heading the DE book gives *Tethered Magic* at 5141 | Gefesselte Magie | **✗ — F-433** |
| Flawed Parma | `tugenden-fehler.md:271`, `:297` *Fehlerhafte Parma* | 6142 Fehlerhafte Parma Magica | Fehlerhafte Parma Magica | ✓ (the table's key is the short English form; the German stem agrees). Note the row is duplicated verbatim at `:271` and `:297`. |
| Flawed Powers | `tugenden-fehler.md:296` *Fehlerhafte Kräfte* | 6146 Fehlerhafte Kräfte | Fehlerhafte Kräfte | ✓ |
| Form Monstrosity | `tugenden-fehler.md:343` *(Form-)Monstrosität* | 6162 **(Form) Missgeburt** | {form} Missgeburt | data ✓; **table disagrees** — F-438 |
| Fragile Constitution | `tugenden-fehler.md:435` *Gebrechliche Konstitution* | 6186 Gebrechliche Konstitution | Gebrechliche Konstitution | ✓ |
| Gabai | `reputationen.md:115` *Gabai* | 6198 Gabai | Gabai | ✓ — and `reputationen.md:115` independently records the Reputation as `2 (–)`, "Steuereintreiber", corroborating `grants_reputation {local, 2}` |
| Generous | `persoenlichkeitseigenschaften.md:79` *Großzügig* | 6206 Großzügig | Großzügig (Groß/Klein) | ✓ |
| Greater Malediction | `tugenden-fehler.md:329` *Große Verfluchung* | 6210 Große Verfluchung | Große Verfluchung | ✓ |
| Greedy | `tugenden-fehler.md:370` *Gierig* | 6214 Gierig | Gierig (Groß/Klein) | ✓ |
| Grudge | `tugenden-fehler.md:371` *Groll* | 6218 Groll | Groll | ✓ |

The sixteen table-less names that match their DE heading: Failed Monk →
*Gescheiterter Mönch*, Failed Student → *Gescheiterter Student*, The Falling Evil
→ *Fallsucht*, Feud → *Fehde*, Fickle Nature → *Launenhaftes Wesen*, Fish Out of
Water (Terrain) → *Fremd im (Gelände)*, Flashbacks → *Rückblenden*, Fluctuating
Fortune → *Schwankendes Glück*, Follower → *Mitläufer*, Foreign Upbringing →
*Fremde Erziehung*, Frail → *Hinfällig*, Fury → *Raserei*, Gender Nonconforming →
*Geschlechtsnichtkonform*, Gullible → *Leichtgläubig*, Hallucinations →
*Halluzinationen*, Harmless Magic → *Harmlose Magie*.

**One German-source inconsistency inside an entry.** `flaw.the_falling_evil`'s DE
heading at 6076 is *Fallsucht* but its own body at DE 6078 calls the affliction
*"Die Fallende Krankheit"*. The data follows the heading, which is right. This is
the third batch running to find this shape (B11's F-405/F-406, B12's
`deteriorating_power`), so it is now a pattern in the German source rather than
an accident. Recorded as the caution it is for whoever writes German text for
this entry: the shipped description already uses the body's wording at that one
spot, verbatim from the source, which is correct for a description quoting the
passage but must not become the entry's *name*.

**`rules/i18n/*/abilities.json` and `rules/core/abilities.json` were consulted**
for the Abilities these passages name: the nine wilderness Abilities
`flaw.feral_upbringing` lists (all `general`, all resolving, `ability.area_lore`
parameterized on `area`), the four Abilities carrying `locality_dependent`
(`area_lore`, `dead_language`, `living_language`, `organization_lore`), and
`categories_requiring_virtue` = `["academic","arcane","martial"]`. That last one
is the whole of F-429.

### Part C systemic gaps are not re-reported per entry

Six of Part C's rows touch this batch and none is counted against an entry:

- **`HealthTrack::Recovery` is surfaced-only** (C1) — that is
  `flaw.fragile_constitution`. The entry is *correctly authored against an engine
  that lists rather than computes it*, and the surfaced row does carry the
  magnitude (`SurfacedModifier { family: HealthRoll, detail, amount: -3 }`), so
  the −3 reaches the read-out. I checked what it actually renders as:
  `locales/en/main.ftl:1367` `derived-detail-recovery = Recovery` and
  `locales/de/main.ftl:1427` `derived-detail-recovery = Genesung`, so the player
  sees "Recovery -3" / "Genesung -3". Not counted, and **no D5 finding either**,
  because the number is displayed rather than dropped. The one nuance the label
  does not carry is the passage's scope — ArMDE:6188 says "all rolls to recover
  from wounds **and diseases**", and the label says only "Recovery" — which is
  the generic surfaced-only limitation Part C records once, not a defect of this
  entry.

- **B11's third shape — "`in_play_effect` entries carry no `description`" —
  partly holds here and has a counter-example.** This batch has three
  `in_play_effect` entries. Two carry no description (`fragile_constitution`,
  `frail`) and **neither needs one**: Frail's −3 Soak is fully computed and
  appears in the Soak breakdown, and Fragile Constitution's −3 is surfaced with
  its magnitude as above. The third, `flaw.flawed_parma_magica`, **does** carry a
  full description in both locales although nothing obliges it to. So the
  practice B11 found uniformly absent across its seven is present on one of my
  three, which is worth recording: it is not a policy of omission, it is
  inconsistent authoring.
- **The sign convention on the health tracks (B12's Q-98)** — re-derived here
  rather than inherited, because `flaw.fragile_constitution` is a carrier. The
  `recovery` track has exactly two shipped carriers,
  `flaw.fragile_constitution` **−3** and `virtue.rapid_convalescence` **+3**, and
  recovery rolls are rolls the character wants to *win*, so Flaw-negative /
  Virtue-positive is both internally consistent and semantically right. It
  matches the `fatigue_roll` track (Long-Winded +3, Obese −3, Short of Breath −3)
  and the `fatigue_penalty` / `wound_penalty` tracks (Enduring Constitution +1,
  Low Tolerance −1). **Four of the five tracks agree; `casting_fatigue` is the
  only one whose doc comment dissents**, which is exactly the conclusion Q-98
  reached from the other direction. Nothing in this batch is inverted.
- **`magic_resistance_mod`'s `param` is read at runtime but validated at load by
  nothing** (C2-e) — Part C names `flaw.flawed_parma_magica` by name as one of
  the two live carriers, and confirms it is "correct today". Re-derived: the entry
  declares `parameters: [{ key: "form", domain: "form" }]` and its effect carries
  `"param": "form"` ✓. Not counted against the entry; the gap is the load gate's.
- **`locality_ability_cap_fraction` rounds up and cannot narrow its own scope**
  (A13) — `flaw.foreign_upbringing` is the variant's *only* shipped carrier, and
  A13's own arithmetic cites ArMDE:6160, which is this entry's line. The
  "as well as some social Abilities" clause reaching nothing is a *deliberate*
  data decision recorded at `RULES.md:6408-6428` ("the passage's trailing … names
  no Abilities, so none are flagged"), so it is not counted; what **is** counted
  against the entry is a different clause entirely (F-437).
- **`grants_reputation`'s `score` is not enforced** (C2) — touches
  `flaw.failed_monk`, `flaw.failed_student`, `flaw.feral_scent` and
  `flaw.gabai`. All four are correctly authored against an engine that counts
  kinds and slots; not counted against any of them.
- **A Reputation's *negativity* is not representable at all** — B10's **F-380**,
  and the same four entries are carriers. `types.rs::Reputation`'s `score` is a
  `u8` with no sign and no polarity flag, while all four passages say the
  Reputation is a bad one: ArMDE:6070 "a **poor** Reputation at level 2",
  ArMDE:6074 "a **Bad** Academic Reputation of 2", ArMDE:6108 "a **negative**
  Reputation of Unclean at level 2", ArMDE:6200 "a **–2 negative** Reputation of
  'Tax Collector'". So all four ship as a *positive* Reputation of the right
  level to the right audience. Recorded once, as F-380 already is, and not
  counted against any of the four — but named here, because the first draft of
  this section listed C2 and omitted F-380 although every Reputation-granting
  entry in the batch carries it.
- **`creation_effect` is not required to carry effects** (C7) — no entry in this
  batch is one of the five, so this row is inert here. Recorded because two of my
  reclassification recommendations (F-429's added effect, and the correction
  shape F-428 needs) touch `creation_effect` entries, and C7 is why the guard
  will not notice if an accompanying effect is forgotten.

### Decisions applied

`docs/vf-audit/decisions.md` was read in full and is binding. It carried D1-D6
when this batch read it.

- **D3** is the load-bearing one again and governs seven entries. Six were
  identified in the first pass and the seventh —
  `flaw.greater_malediction` (no effect calibrates a Flaw's severity against a
  magnitude class, and none confers immunity to a named Supernatural Ability;
  `grant.rs::GrantConstraint` has the right shape and is unreachable from a V/F,
  see Q-107) — came from the verification pass, together with its Minor twin
  `flaw.lesser_malediction`. The six:
  `flaw.harmless_magic` (no effect variant can disable a Technique's permanence,
  and there is no Ritual/non-Ritual carve-out axis outside `CastingScope`, which
  is a *Casting Total* scope and not a spell-outcome one),
  `flaw.fettered_magic` (nothing represents a *character* being an Arcane
  Connection; `spell.rs::SpellRange::ArcaneConnection` is the spell-design Range
  and is the other end of the relationship — B09's F-319
  established this for the identical sentence in `virtue.tethered_magic`),
  `flaw.false_power` ×2 (no realm-interaction chart, and `MagicResistanceEffect`'s
  only realm-scoped members are the two surfaced-only susceptibilities),
  `flaw.fluctuating_fortune` (no effect alternates two other entries year by
  year; `later_life_xp_rate` **replaces** a rate and takes the `min` of several,
  so it structurally cannot express an oscillation), and
  `flaw.feral_upbringing`'s prohibition half (F-410/F-426: nothing forbids an
  Ability or a category). In every case D3 forbids `narrative` and requires the
  rule in the description.
- **D5** is the reason four entries that compute correctly still fail:
  `flaw.feral_scent` (F-436), `flaw.foreign_upbringing` (F-437),
  `flaw.failed_monk` (F-429's second half) and `flaw.gabai` (F-441) each carry a
  right effect and drop an uncomputed clause into neither locale. **Gabai was
  the verification pass's second overturn**, and the reason it was missed is
  worth stating in the Method rather than only in the finding: D5 is an
  obligation on the *text* and is independent of whether the engine's inability
  is already recorded as a finding of its own. "It is F-427" does not discharge
  it — as F-429's second half, three rows earlier in the same table, already
  demonstrated.
- **D6** governs **F-438**'s two rows and, differently, **F-433**. Per the brief
  `arm-de-translation` is outside this repository, so each F-438 row is stated as
  *"this repository's copy disagrees with the rulebook"* and the
  error-vs-stale-copy determination is left open. F-433 is **not** an F-438-shaped
  row: there the DE rulebook contradicts itself, so "follow the rulebook" names no
  answer and the table is the only tie-breaker available.
- **D1 and D4** touch nothing here: no entry in this batch carries
  `lab_total_mod`, and none of D1/D4's nine is in the span.
- **D2** touches nothing here: no entry is a granted Virtue and none carries
  `characteristic_score_delta_param` or `characteristic_score_delta`.

### Cross-references followed and rated

Every pointer in the span was followed and read, including on entries that
passed.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the entry? |
|---|---|---|---|
| `flaw.failed_monk` | "You may take **Academic Abilities** during character creation" (ArMDE:6070) / DE "Du darfst bei der Charaktererschaffung **Akademische Fertigkeiten** nehmen." | `rules/core/abilities.json` — `academic` is one of the three `categories_requiring_virtue` | **YES — this is F-429.** The gate is a permission gate and the entry carries neither `ability_authorization` nor `restricted_ability_xp`, so a companion or grog with Failed Monk who buys an Academic Ability raises a hard `ability_category_requires_virtue` error today. Twenty-ninth instance of F-409's population, and the first to be read entry-first rather than found by grep. |
| `flaw.failed_student` | "he may have a scholastic Social Status Virtue showing how far he got. **(Obviously, this cannot be Doctor in (Faculty).)**" (ArMDE:6074) / DE "(Offensichtlich kann dies kein Doktor in (Fachbereich) sein.)" | `virtue.doctor_in_faculty`, heading at **ArMDE:3683**, body at **:3685** (B02's span) | **YES — a stated exclusion, encoded on neither side (F-435).** `virtue.doctor_in_faculty` is the top of the scholastic ladder (`grants_reputation {academic,3}` + a 300-point Academic pool), and Failed Student is precisely the character who *did not finish*. `jq` returns no `incompatible_with` key on either entry. The target was read in full and rated: its **other** defects are already on the correction list as B02's **F-48** (its `source` range 3683-3698 swallows an unrelated sidebar), **F-56** (three stated minimum scores unencoded as prerequisites — to which :3685's "having already received his **magister in artibus** license" belongs), **F-57** and **F-58**, so the exclusion is the only thing this batch adds to it. |
| `flaw.the_falling_evil` | "usually caused by possession by demons (**Art & Academe, page 48**)" (ArMDE:6078) / DE "(Kunst & Gelehrsamkeit, Seite 48)" | **Nowhere — *Art & Academe* is not in `rules/source/en/`.** | **Cannot be followed, and nothing is lost.** The pointer attributes a *cause* to the affliction, not a rule of the Flaw, and the sentence immediately qualifies it ("but that form of the illness is usually temporary in nature. The character does not seem to have a disease"). The entry's own `description` carries the whole passage including the pointer in both locales ✓. Per CLAUDE.md a rule from a book with no English source here cannot be implemented anyway. |
| `flaw.false_power` / `_minor` | "as described in **Realms of Power: The Infernal, page 91**" (ArMDE:6092) / DE link `…Das Infernale.md#falsche-macht` | **RoP:I:4407-4429** (the same Flaw, near-verbatim) and **RoP:I:3920-3944** (`## False Powers`, the general rule) | **YES, and it sharpens F-431 rather than adding to it.** RoP:I:4407-4429 is the core entry word for word, plus one extra pointer ("see Chapter 12: Black Magic, The False Gift"), so it corroborates the descriptor `*Major, Supernatural, Tainted*` and the Major-then-Minor repeat rule. **RoP:I:3922 is what states the mechanic plainly**: "a False Shapeshifting Power might be **Faerie for the purposes of realm interaction bonuses and penalties**, and appear to be associated with the Faerie realm when subjected to magical or faerie investigation. However, the Power would read as unholy when investigated by a character with the Divine Power to Sense Holiness and Unholiness." A realm-interaction column is a table of numbers; `narrative` cannot survive it. |
| `flaw.feral_scent` | "if he has Initiated the **Sensory Magic Mystery** (described in **Houses of Hermes: Mystery Cults, page 27**)" (ArMDE:6108) / DE link `…Mysterienkulte.md#sinnesmagie-kleines-hausmysterium` | **HoH:MC:971-987**, `#### Sensory Magic (Minor House Mystery)` | **No added rule — and that is the point.** HoH:MC:971-987 defines the Scent/Sound/Spectacle Targets and lists seven restrictions on them, but says **nothing** about doubled area, so the doubling is stated *only* in the core Flaw's own sentence (ArMDE:6108). It is therefore a rule this entry owns outright, and it reaches the user in neither locale — feeds **F-436**. |
| `flaw.feral_scent` | "which **stacks with the penalties imposed by The Gift**, if he has it" (ArMDE:6108) | `virtue.the_gift` ArMDE:3967-3970 | **No rule added, and a negative worth recording.** ArMDE:3969 is three lines that defer entirely — "See earlier, page 63, for full details" — and `virtue.the_gift` ships `narrative` with **no effects at all**, as does `virtue.gentle_gift`; only `flaw.blatant_gift` is `uncomputed_rule`. So the engine has no social-interaction axis of any kind for the −1 to stack onto, which is what makes F-436's first clause a D3 case rather than a missing effect. |
| `flaw.feral_upbringing` | "In your **first five years** you gain **120 experience points**" (ArMDE:6112) | the standard Early Childhood block, **ArMDE:2378** (and its summary at **ArMDE:2213**) | **YES — and it is the batch's worst finding (F-428).** ArMDE:2378: "In the first five years of life, characters gain **75 experience points in their native language** … and **45 experience points** to divide between [eleven Abilities]". Same window, same total, a different and disjoint Ability list, plus an explicit contradiction of the native-language half. Following the pointer is the only way to see that the Flaw *replaces* the block rather than adding to it — and the data adds. |
| `flaw.fettered_magic` | "You cannot take this with the **Virtue Tethered Magic**, as the Virtue **already includes this effect**." (ArMDE:6116) / DE "Du kannst dies nicht zusammen mit der Tugend Gefesselte Magie nehmen, da die Tugend diesen Effekt bereits einschließt." | `virtue.tethered_magic` **ArMDE:5141-5144** (B09's span) | **YES, in three directions (F-432, F-433).** (a) A stated incompatibility, encoded on **neither** side — `jq` returns no `incompatible_with` key on either. (b) ArMDE:5143's last sentence is the Flaw's *entire* body verbatim ("all of your spells and the effects of any magic items you activate are Arcane Connections to you"), which B09 already rated mechanical in **F-319**; the identical sentence cannot be `narrative` on the Flaw while being `uncomputed_rule`-worthy on the Virtue. (c) The **German pointer names the Flaw's own heading**: DE 5141 and DE 6114 are both `#### Gefesselte Magie`, so DE 6116 reads "you cannot take this together with the Virtue *Gefesselte Magie*" while sitting under a heading of that name. |
| `flaw.fluctuating_fortune` | "considered to have the **Wealthy Virtue** one year, followed by the **Poor Flaw** the next" (ArMDE:6152) / DE "gilt ein Jahr lang als Tugend **Wohlhabend**, gefolgt von einem Jahr mit dem Fehler **Arm**" | `virtue.wealthy` **ArMDE:5235-5238**, `flaw.poor` **ArMDE:6594-6596** | **YES — the pointer is the rule (F-434).** ArMDE:5237 "You may spend **three seasons per year** on study or adventuring" and ArMDE:6596 "You must work **three seasons per year**" are exactly the alternation the Flaw's own second sentence spells out ("only work one season in one year, followed by a year in which he has to work three"). Both targets are `creation_effect` carrying `later_life_xp_rate` (20 and 10 against a base of 15, `rules/core/life_stages.json:39`) — so the two halves *average to the base rate*, which is a defensible reason the engine computes nothing and **no reason at all** for `narrative`. Both targets also say they are unavailable to magi; Fluctuating Fortune says nothing of the kind. |
| `flaw.foreign_upbringing` | "somewhat similar to **Covenant Upbringing**" (ArMDE:6160) / DE "ähnelt in gewissem Maße der **Konventsprägung**" | `flaw.covenant_upbringing` **ArMDE:5865-5868** | **No.** ArMDE:5867 states a *permission* ("You may take Latin at character creation") and a language note, and no cap of any kind. The similarity is thematic — both are "raised somewhere else" Minor Personality Flaws — so nothing transfers. Corroboration ✓, and it confirms that the half-cap is this entry's own rule. (Covenant Upbringing's own permission **is** encoded, `ability_authorization: ["ability.dead_language"]`, which is one of the two entries in F-409's encoded population.) |
| `flaw.flawed_powers` | "She suffers the effects of a **Major Hermetic Flaw** (commonly **Restriction** or **Necessary Condition**)" (ArMDE:6148) | the Major Hermetic Flaws block; the engine side is `crates/arm-rules/RULES.md:4785-4820` | **YES, and RULES.md already owns it.** `RULES.md:4818-4820` names this entry explicitly: "`flaw.university_dean` (ArMDE:6925) and `flaw.flawed_powers` (ArMDE:6148) are **selection prerequisites the engine does not express** — a minimum age of 40 and a Flaw exclusion in the first, 'at least one Major Supernatural Virtue' and a Hermetic-Flaw exclusion in the second." Both clauses are carried in `description` in both locales ✓. So the entry passes, and the only residue is a modelling asymmetry against `flaw.false_power` — **Q-104**, not a finding. |
| `flaw.greater_malediction` | *(inbound)* ArMDE:7397, `#### Curse-Throwing*` — "Curse-Throwing cannot affect Flaws; specifically, **someone with the Lesser or Greater Malediction Flaw is beyond the power of Curse-Throwing**, unless it is a Flaw imposed by a faerie or magician with a limited duration." | `ability.curse_throwing`; the named Flaw is this entry and its Minor twin `flaw.lesser_malediction` (ArMDE:6342-6345, B15's span) | **YES — an immunity to a modelled Supernatural Ability, attributed by name, with its own carve-out. This is F-442.** It is an *inbound* pointer, which is why every pass so far has missed it: the entry's own cited range 6210-6213 says nothing of the kind, and the `uncomputed_clauses.rs` screen reads only that range. Found by grepping the whole book for "Malediction" — **11 hits**, and exactly one of them is a rule: 5395 and 5548 are "List of Flaws" index links, 6210 and 6342 the two entry headings, 24788 and 25004 the back-of-book page index, 18655 / 20159 / 20794 / 21002 four sample-character Virtues-and-Flaws lines, and **7397** this immunity. |
| `flaw.fickle_nature` | "**Select a Personality Trait at +4, and its opposite at +4.**" (ArMDE:6124) | `validation/scores.rs::validate_personality_traits`, implementing **ArMDE:2500-2502** | **YES, and the two rulebook passages contradict each other — this is F-440 and Q-110.** ArMDE:2502 says a **Minor** Personality Flaw is represented by a trait of "+3 or -3" and grants a ±6 slot only for a **Major** one; the validator implements that and hard-errors on both of Fickle Nature's mandated +4s. Following the pointer is the only way to see it: nothing in ArMDE:6122-6125 hints that another passage forbids what it commands. |
| `flaw.fury` | *(no textual pointer — a shape comparison)* "**While enraged** you get +3 to Damage, but -1 on all other scores and rolls" (ArMDE:6196) | `virtue.berserk`, **ArMDE:3500-3503** (B01's span) | **YES, and it destabilises this entry's verdict — Q-109.** ArMDE:3502 states the identical condition ("**While berserk**, you get +2 to Attack and Soak scores, but suffer a -2 penalty to Defense") and ships `in_play_effect` with `combat_mod {attack,+2}`, `combat_mod {defense,-2}`, `soak_mod {+2}`, folded **flat**. B01's **F-20** already rates that a defect. One shape, two opposite encodings, and the audit has blessed both. |
| `flaw.hallucinations` | "If the character also has the **Visions Flaw**, then some of her visions are true" (ArMDE:6228) / DE "Hat der Charakter auch den Fehler **Visionen**" | `flaw.visions` (`narrative`, `["story","supernatural"]`, no effects) | **No.** The clause changes what the *other* Flaw delivers in fiction — some visions true, most nonsense — with no number, no roll and nothing granted or removed. Unlike B11's F-406 "includes the effects of" idiom, nothing is composed. `narrative` survives ✓ on both entries for this pointer. |
| `flaw.harmless_magic` | "a version of **The Wound that Weeps (PeCo15)**… **Fist of Shattering (PeTe10)**" (ArMDE:6232) | the spell catalogue | **No new rule — the two are worked examples of the rule the sentence before them states.** They are useful only as evidence that the clause is mechanical rather than atmospheric: the passage reaches for two specific spells at specific levels to show what changes. Feeds **F-430**. |
| `flaw.form_monstrosity` | "**1 pawn of Muto vis** may be extracted from the corpse of a monstrous character" (ArMDE:6166) / DE "kann aus dem Kadaver eines monsterhaften Charakters **1 Bauer Muto-Vis** gewonnen werden" | the vis rules | **Undecided — escalated as Q-105.** It is a concrete quantity of a modelled resource, which argues mechanical; it is hedged twice ("Often", "may be") and describes the character's *corpse* rather than the character, which argues colour. The passage also says the monstrous feature "generally gives the character **no advantage** over its non-mutated peers", i.e. the entry declares itself mechanically inert. I did not settle it. |
| `flaw.gabai` | "This Flaw is **compatible with any other Free Social Status Virtue**." (ArMDE:6200) | ArMDE:2816 | **An ArMDE:2816 exemption, and it is enforced nowhere** — re-derived, not inherited (see below). No `incompatible_with` is needed *because* the sentence permits rather than forbids, so the entry's empty set is right; what is missing is the rule the exemption is an exemption *from*. Not counted against the entry; it is F-427. |

### ArMDE:2814, :2816, :2818, :2820 — instances in this batch

ArMDE:2812-2820 was re-read in full for this batch rather than taken from B10 or
B12, because F-427 exists precisely because an earlier batch carried one of these
rows across without re-deriving it.

- **ArMDE:2814** ("A Virtue or Flaw may be taken more than once only if the
  description explicitly allows it. Most Virtues and Flaws may only be taken
  once.") — **three** instances, and the data handles all three correctly, which
  is the best score any batch has had on this row.
  `flaw.false_power` ArMDE:6096 permits repeats with a magnitude change → Major
  `max_total: 1`, Minor unbounded with `prerequisites: {has, flaw.false_power}`,
  a design `RULES.md:1772-1822` documents at length ✓.
  `flaw.flawed_parma_magica` ArMDE:6144 "You may purchase this Flaw more than
  once for **different Forms**" → no `max_total` (the 255 sentinel) with
  `max_per_target` at its default 1, whose key is `(item_ref, whole params map)`
  — so two copies naming different Forms are two tuples and legal, two naming the
  same Form are one tuple and blocked. Exactly what the sentence says ✓.
  And `flaw.fish_out_of_water_terrain` ArMDE:6132 says the **opposite** — "This
  Flaw may only be taken once, because taking it more than once makes it less
  serious, rather than more" — which needs `max_total: 1` *despite* the entry
  being parameterized, because `max_per_target` alone would let two terrains
  through. It carries `max_total: 1` ✓. That is the one shape in this row the
  default cannot cover, and it is right.
  **The other 32 entries are unparameterized with no `max_total`**, so
  `max_per_target`'s default of 1 keyed on `(item_ref, {})` already forbids a
  second copy, which is what the book requires.
- **ArMDE:2816** ("All characters must take one Social Status, and may only take
  more than one if the descriptions … explicitly note that they are compatible")
  — two instances, `flaw.failed_monk` and `flaw.gabai`, the latter carrying an
  explicit compatibility clause (ArMDE:6200). **F-427 re-derived independently
  here and stands**: `jq` over `rules/core/character_types.json` returns
  `flaw_category_caps` containing only `personality` (×2) and `story` rows on all
  four profiles, and `virtue_category_caps` only a `hermetic` row on `magus`.
  There is no `social_status` cap of either sign anywhere, and `required_traits`
  is id-based so "one of category X" is inexpressible. Neither half of ArMDE:2816
  is enforced. Not counted against either entry.
- **ArMDE:2818** ("A character should not have more than one Story Flaw") —
  three instances: `flaw.favors`, `flaw.feud`, `flaw.fury`. Enforced on all four
  profiles, re-derived: every profile carries
  `{"category":"story","max":1}` (grog `max: 0`) ✓.
- **ArMDE:2820** ("A character may not have more than one Major Personality
  Flaw … should normally not have more than two Personality Flaws in total") —
  three Major/Minor pairs here (`gender_nonconforming`, `generous`, `greedy`),
  each with a mutual `incompatible_with` ✓, additionally load-enforced by
  `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`. Both halves are
  live and re-derived: every profile carries
  `{"category":"personality","max":1,"major_only":true,"hard":true}` and
  `{"category":"personality","max":2}` (grog 0 and 1) ✓. This batch is the
  densest Personality span in the audit so far — three pairs plus five Minor
  singles (`fear`, `fickle_nature`, `follower`, `foreign_upbringing`, `grudge`).
- **ArMDE:2960-2962** (Supernatural realm association) — five instances:
  `flaw.false_power` ×2, `flaw.flawed_powers`, `flaw.fluctuating_fortune`,
  `flaw.form_monstrosity`, `flaw.greater_malediction`. None carries a
  `realm`-domain parameter. Noted only; it is B11's standing observation, not a
  new one. The False Power pair is the interesting case — it is the one entry in
  the span whose realm question the book *answers* (two realms at once,
  ArMDE:6082), and `tainted: true` records half of that answer.

## Verdicts

35 rows, one per entry. `class` / `data` / `text` report checks 2, 3-11 and 12.
"OK" means every check in that column passed; `?` means escalated, not resolved.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.failed_monk` | 6068-6071 | creation_effect ✓ | **no `ability_authorization` for the `academic` category: doing what the Flaw permits is a hard validator error today**; the two `grants_reputation` rows are correct in kind and score ✓ | the Academic permission, the vows release and the "Failed Nun" variant in neither locale | **F-429**; ArMDE:2816 instance |
| `flaw.failed_student` | 6072-6075 | creation_effect ✓ | `grants_reputation {academic, 2}` correct ✓; **the stated Doctor in (Faculty) exclusion is encoded on neither side** | the exclusion in neither locale | **F-435** |
| `flaw.the_falling_evil` | 6076-6079 | uncomputed_rule ✓ | OK | OK ✓ — full passage both locales | **clean**; DE heading/body term split noted |
| `flaw.false_power` | 6080-6097 | narrative → uncomputed_rule | `max_total: 1` + the item-domain `virtue` param correct ✓ (`RULES.md:1772-1860`); pair-incompatibility deliberately absent ✓ | the realm-alignment rule and the realm-interaction-chart option in neither locale | **F-431** **Q-103** |
| `flaw.false_power_minor` | 6080-6097 | narrative → uncomputed_rule | `prerequisites: {has, flaw.false_power}` correct ✓; unbounded `max_total` correct ✓ | as above | **F-431** **Q-103** |
| `flaw.favors` | 6098-6101 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.fear` | 6102-6105 | narrative ✓ | OK | OK | **clean** |
| `flaw.feral_scent` | 6106-6109 | creation_effect ✓ | `grants_reputation {local, 2}` correct ✓, corroborated by `reputationen.md:110` | **the -1 social penalty and the doubled Scent-Target area in neither locale** | **F-436**; F-438 (table row) |
| `flaw.feral_upbringing` | 6110-6113 | creation_effect ✓ | **the 120-point pool is stacked on top of the childhood block it replaces: 240 XP for the first five years, and 75 of them in the Language the Flaw forbids** | both prohibitions in neither locale | **F-428** **Q-106** |
| `flaw.fettered_magic` | 6114-6117 | narrative → uncomputed_rule | **the stated incompatibility with `virtue.tethered_magic` is encoded on neither side** | **the DE name is `Gefesselte Magie`, which is also `virtue.tethered_magic`'s DE name** | **F-432** **F-433** |
| `flaw.feud` | 6118-6121 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.fickle_nature` | 6122-6125 | uncomputed_rule ✓ | **entering the two +4 traits the Flaw mandates raises two hard `personality_trait_out_of_range` errors, because a *Minor* Personality Flaw buys no ±4 slot** | OK ✓ — description carries "+4 … +4" verbatim both locales | **F-440** **Q-110** |
| `flaw.fish_out_of_water_terrain` | 6126-6133 | uncomputed_rule ✓ | `max_total: 1` correct and load-bearing ✓ (see ArMDE:2814 above) | OK ✓ — the -1 and the extra botch die in both locales, ASCII hyphen ✓ | **clean**; the one `uncomputed_rule` entry with no `anchor` |
| `flaw.flashbacks` | 6134-6141 | uncomputed_rule ✓ | OK | OK ✓ — full passage both locales, `-1 bis -5` ASCII ✓ | **clean** |
| `flaw.flawed_parma_magica` | 6142-6145 | in_play_effect ✓ | `magic_resistance_mod {halved_parma, param: form}` correct ✓; repeatability correct ✓ | OK ✓ — carries a description although its class does not oblige one | **clean**; C2-e carrier, not counted |
| `flaw.flawed_powers` | 6146-6149 | uncomputed_rule ✓ | OK — both inexpressible clauses recorded at `RULES.md:4818-4820` | OK ✓ — full passage both locales | **clean**; **Q-104** (no target parameter, unlike False Power) |
| `flaw.fluctuating_fortune` | 6150-6153 | narrative → uncomputed_rule | OK — no effect can alternate two entries year by year (D3) | the Wealthy/Poor alternation and the one-vs-three-seasons rule in neither locale | **F-434** |
| `flaw.follower` | 6154-6157 | narrative ✓ | OK | OK | **clean** |
| `flaw.foreign_upbringing` | 6158-6161 | creation_effect ✓ | `locality_ability_cap_fraction {1,2}` correct, and the engine's ceiling division matches "half (round up)" ✓ | **the Native-Language requirement in neither locale** | **F-437** |
| `flaw.form_monstrosity` | 6162-6185 | narrative **?** | OK — the `form` param is unconsumed, which B8 names as the normal shape | OK | **Q-105**; F-438 (table row) |
| `flaw.fragile_constitution` | 6186-6189 | in_play_effect ✓ | `health_mod {recovery, -3}` correct in sign and size ✓ — re-derived against the neighbouring tracks, not against Q-98's disputed doc | OK — the surfaced row carries the magnitude, so nothing is dropped | **clean** |
| `flaw.frail` | 6190-6193 | in_play_effect ✓ | `soak_mod {-3}` correct ✓, computed in `derived/combat.rs::soak` — **but `types.rs::Effect::SoakMod`'s own doc says "Frail −1"** | OK | **clean** (the data is right); **F-443** is the doc comment |
| `flaw.fury` | 6194-6197 | uncomputed_rule **?** | the +3/-1 are conditional on being enraged and no effect gates a condition — **but `virtue.berserk` states the same condition and ships `in_play_effect` with flat `combat_mod`/`soak_mod`** | OK ✓ — full passage both locales, ASCII signs ✓ | **Q-109**; ArMDE:2818 instance |
| `flaw.gabai` | 6198-6201 | creation_effect ✓ | `grants_reputation {local, 2}` correct ✓, corroborated by `reputationen.md:115` | **the Free-Social-Status compatibility clause in neither locale** | **F-441**; ArMDE:2816 instance (F-427) |
| `flaw.gender_nonconforming_major` | 6202-6205 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.gender_nonconforming_minor` | 6202-6205 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.generous_major` | 6206-6209 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.generous_minor` | 6206-6209 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.greater_malediction` | 6210-6213 | narrative → uncomputed_rule | OK — a magnitude-calibration instruction the engine cannot express, and an immunity stated at ArMDE:7397 | the calibration and the Curse-Throwing immunity in neither locale | **F-442** **Q-107** |
| `flaw.greedy_major` | 6214-6217 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.greedy_minor` | 6214-6217 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.grudge` | 6218-6221 | narrative ✓ | OK | OK | **clean** |
| `flaw.gullible` | 6222-6225 | uncomputed_rule ✓ | OK | OK ✓ — full passage both locales | **clean** |
| `flaw.hallucinations` | 6226-6229 | narrative ✓ | OK | OK | **clean** |
| `flaw.harmless_magic` | 6230-6235 | narrative → uncomputed_rule | OK — no effect variant can disable a Technique's permanence (D3) | the Ritual-Perdo carve-out in neither locale; the main rule **does** survive, in the summary | **F-430** |

**Fully clean: 20** — `the_falling_evil`, `favors`, `fear`, `feud`,
`fish_out_of_water_terrain`, `flashbacks`, `flawed_parma_magica`,
`flawed_powers`, `follower`, `fragile_constitution`, `frail`,
`gender_nonconforming_major`, `gender_nonconforming_minor`, `generous_major`,
`generous_minor`, `greedy_major`, `greedy_minor`, `grudge`, `gullible`,
`hallucinations`. Two of those 20 pass all twelve checks while carrying an open
question about something *adjacent* to the entry — `flawed_powers` (**Q-104**, a
modelling asymmetry against False Power) and `frail` (**F-443**, a wrong doc
comment about its value, which is not a defect of the entry) — and both are
counted clean, because the question is not a failed check.

**With at least one failed or escalated check: 15** — `failed_monk`,
`failed_student`, `false_power`, `false_power_minor`, `feral_scent`,
`feral_upbringing`, `fettered_magic`, `fickle_nature`, `fluctuating_fortune`,
`foreign_upbringing`, `form_monstrosity` (escalated, not failed — **Q-105**),
`fury` (escalated — **Q-109**), `gabai`, `greater_malediction`,
`harmless_magic`.

*This count was wrong in the first draft of this file* — it read "Failures: 12.
Clean: 23", which is neither the 24 rows the table then marked clean nor the 11
it marked failed, and the parenthetical explaining the discrepancy was itself
self-contradictory. The verification pass caught the arithmetic; the substantive
changes above (four new failures, one new escalation) came from the same pass and
are recorded in the reconciliation section.

**The six rows whose `class` column reads `narrative → …` are the evidence for
the screen gap**: `false_power`, `false_power_minor`, `fettered_magic`,
`fluctuating_fortune`, `greater_malediction`, `harmless_magic`. All six are
inside `SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` block — swept 2026-09-15, re-swept
2026-09-19 — and all six are green under the live guard today
(`cargo test -p arm-rules --test uncomputed_clauses` → 4 passed, run 2026-09-20).
None of the 35 appears in `NO_RULE_DESPITE_TOKEN`, whose six ids
(`overconfident_major/_minor`, `horrifying_appearance_snake_legs`,
`primogeniture_lineage`, `true_love_major/_minor`) are all outside this span, so
no exemption is contradicted here.

**The exact wording the phrase screen missed, verbatim, in both locales.** This
is the list that outlives the batch and feeds the screen fix. `MECHANICAL_PHRASES`
is a ~20-phrase, left-word-boundary-matched list ("up to", "at least",
"ease factor", "equal to", "multiply", "round up/down", "simple die",
"stress die", "cannot die", "one magnitude", plus their German twins), and
`states_a_mechanical_rule = has_signed_number || has_botch_term ||
has_mechanical_phrase`.

| entry | EN wording the screen cannot see | DE wording the screen cannot see | why it slips through |
|---|---|---|---|
| `flaw.false_power` / `_minor` | ArMDE:6082 "is associated with the Infernal realm, **in addition to** the realm with which it would normally be associated, which causes it to **appear unholy when subjected to Divine or Infernal investigation**"; ArMDE:6092 "**using the Infernal Power column of the realm interaction chart** and thus giving him **a bonus in Infernal auras**" | DE 6082 "ist mit der Höllensphäre verbunden, **zusätzlich zu** der Sphäre, mit der sie normalerweise verbunden wäre … dass sie **unheilig erscheint**, wenn sie einer göttlichen oder infernalen Untersuchung unterzogen wird"; DE 6092 "dabei die **Infernale Macht-Spalte der Sphären-Interaktionstabelle** verwenden und so **einen Bonus in infernalen Auren** erhalten" | A named lookup table and an unsigned "a bonus". Zero digits in either sentence. |
| `flaw.fettered_magic` | ArMDE:6116 "**All of your spells and the effects of any magic items you activate are Arcane Connections to you.**" | DE 6116 "**Alle deine Zauber und die Wirkungen aller Magieartefakte, die du aktivierst, sind Arkane Verbindungen zu dir.**" | An absolute stated with a defined game term and **no number of any kind**. This is the same sentence B09's F-319 flagged on the Virtue. |
| `flaw.fluctuating_fortune` | ArMDE:6152 "He is **considered to have the Wealthy Virtue one year, followed by the Poor Flaw the next**. … the character will have to **only work one season in one year, followed by a year in which he has to work three**" | DE 6152 "Er gilt **ein Jahr lang als Tugend Wohlhabend, gefolgt von einem Jahr mit dem Fehler Arm**. … dass der Charakter **in einem Jahr nur eine Jahreszeit arbeiten muss, gefolgt von einem Jahr, in dem er drei arbeiten muss**" | The quantities are spelled as words ("one", "three" / "eine", "drei"), so `has_signed_number` sees nothing, and naming two catalogue entries is not a phrase the list knows. |
| `flaw.greater_malediction` | ArMDE:6212 "The effects of the curse **should be comparable to those of other Major Flaws**. Indeed, **almost any Flaw could be the result of a curse**."; and ArMDE:7397, outside the cited range, "someone with the Lesser or Greater Malediction Flaw is **beyond the power of Curse-Throwing**" | DE 6212 "Die Auswirkungen des Fluchs **sollten denen anderer Großer Fehler vergleichbar sein**. Tatsächlich **könnte fast jeder Fehler das Ergebnis eines Fluches sein**." | A magnitude-class comparison with no digits, and an absolute ("beyond the power of") that the screen could never see anyway, because it lives in **another entry's passage**. This one is a second, harder gap: the screen reads only the entry's own cited range. |
| `flaw.harmless_magic` | ArMDE:6232 "The character's Perdo spells **cannot permanently destroy anything**; they temporarily disrupt the target, like Perdo Imaginem magic, but **as soon as the duration has passed the target returns to its natural state** as if nothing had happened, like a Muto effect. … **The character's Ritual Perdo spells function normally, however.**" | DE 6232 "Die Perdo-Zauber des Charakters **können nichts dauerhaft zerstören** … sobald die Dauer verstrichen ist, **kehrt das Ziel in seinen natürlichen Zustand zurück** … **Die Ritualmagie-Perdo-Zauber des Charakters funktionieren jedoch normal.**" | The only digits in the passage are inside the two spell names (`PeCo15`, `PeTe10` / `PeCo 15`, `PeTe 10`) and carry no sign. "cannot permanently destroy" is one word away from the list's `cannot die` and matches nothing. |

**Two idiom families this batch adds evidence for, both named by B12.** Neither
is on the phrase list and neither entry below is `narrative`, so the screen would
not have looked at them anyway — recorded because the screen fix should cover the
families rather than the four rows above:

- **The permission idiom** (F-409). `flaw.failed_monk`, ArMDE:6070: "**You may
  take Academic Abilities during character creation.**" / DE 6070: "**Du darfst
  bei der Charaktererschaffung Akademische Fertigkeiten nehmen.**"
- **The prohibition idiom** (F-410 / F-426). `flaw.feral_upbringing`,
  ArMDE:6112: "**You may only choose beginning Abilities that you could have
  learned in the wilds. In particular, you may not start with a score in a
  Language.**" / DE 6112: "**Du darfst nur Ausgangsfertigkeiten wählen, die du in
  der Wildnis hättest erlernen können. Insbesondere darfst du nicht mit einem
  Wert in einer Sprache beginnen.**"

## Findings

### F-428 — `flaw.feral_upbringing` — a Flaw that *replaces* the childhood block is encoded as one that *adds* to it, so the first five years fund 240 XP instead of 120

> "You grew up in the wilderness, either raised by wild animals or surviving on
> your own. For much of your life you could not speak, and knew nothing of human
> ways. Now that you have joined human society (or the covenant), you have
> learned to understand some basic spoken phrases, but civilized life is still a
> mystery you want little part of. **You may only choose beginning Abilities that
> you could have learned in the wilds. In particular, you may not start with a
> score in a Language. In your first five years you gain 120 experience points,
> which must be split between (Area) Lore, Animal Handling, Athletics, Awareness,
> Brawl, Hunt, Stealth, Survival, and Swim.**" — ArMDE:6112
>
> DE 6112: "…**Du darfst nur Ausgangsfertigkeiten wählen, die du in der Wildnis
> hättest erlernen können. Insbesondere darfst du nicht mit einem Wert in einer
> Sprache beginnen. In deinen ersten fünf Jahren erhältst du 120
> Erfahrungspunkte, die auf (Gebiets-)Kunde, Tierumgang, Athletik, Wahrnehmung,
> Raufen, Jagen, Schleichen, Überleben und Schwimmen aufgeteilt werden müssen.**"

**And the block it is written against**, which is what makes the reading
inescapable:

> "**In the first five years of life**, characters gain **75 experience points in
> their native language** (see page 167 for the Language Ability), which normally
> gives them a score of 5, and **45 experience points** to divide between Area
> Lore (for the place or places the character is growing up), Athletics,
> Awareness, Brawl, Charm, Folk Ken, Guile, Living Language (other than the
> character's native language), Stealth, Survival, and Swim." — ArMDE:2378
> (summarised identically at ArMDE:2213, step 5, "Early Childhood")

**Current data.** `classification: "creation_effect"`, one effect:

```json
{"type": "restricted_ability_xp", "amount": 120,
 "abilities": ["ability.animal_handling","ability.area_lore","ability.athletics",
               "ability.awareness","ability.brawl","ability.hunt",
               "ability.stealth","ability.survival","ability.swim"]}
```

The nine ids all resolve and all carry category `general`; `ability.area_lore`
is parameterized on `area`, which `RestrictedAbilityXp` matches by id across every
instance (A7) ✓. The *list* is right. The *arithmetic* is not.

**Why this is a replacement and not a grant, in four independent ways.** The word
B11's F-387 keyed on is "gain", and on its own it does read as a grant. Four
facts in the two passages say otherwise, and no one of them is decisive alone:

1. **The window is identical.** "In your first five years" (ArMDE:6112) is
   "In the first five years of life" (ArMDE:2378), which is
   `rules/core/life_stages.json`'s `childhood.years: 5`.
2. **The total is identical.** 75 + 45 = **120**, the Flaw's figure exactly.
   `rules/core/life_stages.json`'s `childhood` block ships
   `native_language_xp: 75` and `spread_xp: 45`, read into
   `life_stage.rs::ChildhoodRules`.
3. **The Ability list is a substitution, not an extension.** The standard spread
   is eleven Abilities (Area Lore, Athletics, Awareness, Brawl, **Charm**, **Folk
   Ken**, **Guile**, **Living Language**, Stealth, Survival, Swim); the Flaw's is
   nine, dropping the four social/linguistic ones and adding **Animal Handling**
   and **Hunt**. "which **must** be split between" is a closed list, not an
   addition to an open one.
4. **The Flaw forbids what the standard block mandates.** ArMDE:6112's own
   preceding sentence is "you may not start with a score in a Language", and the
   standard block's larger half is *75 experience points in the native language,
   normally a score of 5*. A reading in which both apply has the Flaw handing the
   character the very thing it forbids.

**The engine really does add them, verified end to end.**
`effective/xp.rs::build_flow_pools` starts from
`restricted_ability_xp_pools(entity, ruleset)` — which is where the Flaw's 120
lands — and then, whenever a life-stage budget exists, **extends** that vector
with `childhood_native_language_pool(...)` and **pushes**
`childhood_spread_pool(...)`. There is no suppression path: the childhood pools
are built from `LifeStageBudget`'s `childhood_native_xp` and
`childhood_spread_xp`, which `life_stage.rs::LifeStageRules::budget` fills
straight from `self.childhood.native_language_xp` and `self.childhood.spread_xp`
with no consultation of the entity's selections at all. So the three pools
coexist and the two-phase max flow drains all three.

**What a player gets today.** A companion or grog with Feral Upbringing, entered
with a life-stage plan, has **75 restricted Living Language points + 45
restricted childhood-spread points + 120 restricted wilderness points = 240
experience points for the first five years**, against a rulebook figure of 120.
Because `RestrictedAbilityXp` also confers permission over what it funds
(`effective/xp.rs::ability_authorizations`, A7), nothing objects; and because the
childhood native-language pool is `PoolEligibility::Ability` scoped to
`ability.living_language`, the character is *pushed* toward a Language 5 — the
one thing ArMDE:6112 rules out. On the flat `AbilityFunding::Pool` path there is
no childhood block (`life_stage.rs::budget` returns `None`), so the 120 is a
clean grant there and only the life-stage path is wrong.

**And the app actively pushes the player into spending it.**
`validation/magus.rs` emits `restricted_xp_unspent` (and `general_xp_unspent`)
naming each pool that is not drained — its own comment at the helper that builds
the origin pair uses this exact scenario as its example, "a life-stage character
leaves childhood's two blocks (75 and 45) unspent". So a Feral Upbringing
character who *correctly* ignores the childhood pools is nagged about 120 unspent
points, while one who spends everything the app offers ends up 120 points over
the rulebook figure with no issue raised at all. The validator is pointed the
wrong way round.

**Nothing records the additive reading as a decision.** `crates/arm-rules/RULES.md`
mentions this entry twice — `:5200` lists it among the `restricted_ability_xp`
reuse examples ("Feral Upbringing → `{120, …}`") and `:5412` records
"`flaw.feral_upbringing` (ArMDE:6110-6113) — XP grant 120". Neither names
ArMDE:2378, the childhood block, or the overlap. B11's F-387 cited the entry in
passing as an example of a genuine grant, on the strength of the word "gain",
without reading ArMDE:2378 — which is exactly the inherited-verdict shape this
audit's rules forbid, and why I re-derived it rather than carrying it.

**Which checks fail.** Check 10, both directions — the effect does not correspond
to the clause (it adds where the clause replaces), and two clauses of the passage
reach nothing. Check 11, engine reality — the engine does something the rulebook
does not say. Check 2 survives: `creation_effect` is right, because the entry
*does* move a creation number; it moves it wrongly.

**What the correction is — and it is not obvious, which is Q-106.** The
`RestrictedAbilityXp` variant has no "replaces the childhood block" semantics and
`LifeStageBudget` has no suppression hook. Three shapes are available and they
differ in cost: a new `Effect` variant that zeroes the childhood pools; a
`childhood_package`-shaped data row (the field already exists on the plan,
`life_stage.rs::LifeStagePlan::childhood_package`) that the Flaw selects; or
leaving the pool at 120 and
subtracting the childhood total, which cannot be done with the current effect
set. The two prohibitions ("only Abilities learnable in the wilds", "no Language
score") are separately inexpressible — F-410/F-426 established that no effect
forbids an Ability or a category — and are a **D5** obligation in both locales
whichever shape is chosen.

**Severity: high, and it is the only miscalculation in this batch.** Per
CLAUDE.md, wrong rules output is the top severity class: this is a **doubled
starting budget** on a Minor Flaw, silently, on the app's own recommended entry
path, and it makes the character *stronger* for taking a Flaw the player paid a
point for. It also produces a character the rules forbid (a feral child with
Native Language 5). Both directions are wrong output, and neither is visible to
any existing guard: `data_integrity.rs::every_vf_is_classified` only asks whether
a `creation_effect` entry is classified, and C7 records that it does not even
require one to carry effects.

---

### F-429 — `flaw.failed_monk` — the permission idiom on the entry B12 used as its comparison point, and a live false hard error

> "You were once a member of a cloistered order, but were cast out for some great
> sin or gross incompetence, or perhaps you ran away and your abbot might allow
> you to come back after a suitable punishment. Because of this, you have a poor
> Reputation at level 2 in the local area and within the Church. You no longer
> need to observe your monastic vows of poverty, chastity, and obedience, though
> you may still practice them as they might be ingrained in your nature. **You may
> take Academic Abilities during character creation.** Female characters may take
> this Flaw as Failed Nun." — ArMDE:6070
>
> DE 6070: "…**Du darfst bei der Charaktererschaffung Akademische Fertigkeiten
> nehmen.** Weibliche Charaktere können diesen Fehler als Gescheiterte Nonne
> nehmen."

**Current data.** `classification: "creation_effect"` with two effects —
`grants_reputation {local, 2}` and `grants_reputation {ecclesiastical, 2}` —
no `parameters`, no `description` in either locale. The two Reputations are
**correct**: ArMDE:6070's "in the local area and within the Church" is exactly
two audiences at level 2, and `RULES.md:5376` records it
("`flaw.failed_monk` (ArMDE:6068-6071) — reputation grant poor score 2"). B12's
F-408 used this entry as the decisive comparison that proved
`flaw.excommunicate` is missing its grant, and that comparison stands.

**What B12 could not see from outside the span is that the same entry is itself
incomplete, in the other of B12's two families.** ArMDE:6070's fourth sentence is
the **permission idiom** — F-409's shape, and F-409's own census already lists
`flaw.failed_monk :6068` in its "carrying effects that do not cover the
permission" table. This batch reads the entry rather than grepping it, and
confirms the chain end to end:

1. `rules/core/abilities.json` sets
   `"categories_requiring_virtue": ["academic","arcane","martial"]`.
2. `effective/xp.rs::ability_authorizations` builds the permitted union from
   exactly two effect variants, `AbilityAuthorization` and `RestrictedAbilityXp`.
   `flaw.failed_monk` carries neither — its only effects are
   `grants_reputation`, which that function does not read.
3. `validation/authorization.rs::validate_ability_authorization` returns early
   only for `type_profile.is_some_and(|profile| profile.is_magus)`. Failed Monk is
   a **Minor Social Status** Flaw, so grogs and companions may take it and the
   magus exemption is empty for them.
4. Therefore a Failed Monk companion who buys any Academic Ability — Artes
   Liberales, Philosophiae, Medicine, Theology, Civil and Canon Law, a Dead
   Language — raises a hard `ability_category_requires_virtue` **error** in
   Enforced mode, which is the default for both guided and direct-validated entry.

**The Flaw exists in order to permit exactly that.** A man who was a cloistered
monk and is no longer one is the canonical character who can read Latin and has
no Social Status Virtue to prove it; the sentence is not incidental colour, it is
the mechanical compensation for the two negative Reputations.

**What the correction is.**
`{"type":"ability_authorization","categories":["academic"]}` — a **category**
authorization, not an id list, because the sentence names the category
("Academic Abilities") and not particular Abilities. The shape already ships:
`flaw.covenant_upbringing` carries `ability_authorization: {abilities:
["ability.dead_language"]}` for the narrower ArMDE:5867 clause, and
`virtue.student_of_realm` carries a four-Ability list. `classification` stays
`creation_effect`; the entry already is one. **Note C7**: the guard does not
assert that a `creation_effect` carries effects at all, so nothing would catch
the addition being forgotten.

**Two smaller clauses are D5 obligations in the same correction.** "You no longer
need to observe your monastic vows of poverty, chastity, and obedience" is
arguably colour and I do not press it; "**Female characters may take this Flaw as
Failed Nun**" is a naming variant that reaches the player nowhere — the shipped
EN name is `Failed Monk`, the DE `Gescheiterter Mönch`, and the book's own index
calls the entry `Failed Monk/Nun` (ArMDE:5526). A player building a female
character is not told the Flaw is hers to take.

**Which checks fail.** Check 10, both directions — a passage clause with no
effect. Check 11 — a validator rejecting a build the rulebook explicitly permits.
Check 12 / D5 — the permission, and the Failed Nun variant, reach the player in
neither locale. Check 2 survives.

**Severity: high**, on exactly the grounds B11 rated F-388 and F-407 and B12
rated F-409 high: a false hard error that blocks a character the rulebook
explicitly permits, firing in the default validation mode, with a fix that
over-permits nothing. It is not a miscalculation — no number is wrong — but a
validator that rejects a legal build is a product-integrity failure of the same
class.

---

### F-430 — `flaw.harmless_magic` — an entire Technique's permanence is switched off, classified as flavour

> "The character's Perdo spells **cannot permanently destroy anything**; they
> temporarily disrupt the target, like Perdo Imaginem magic, but **as soon as the
> duration has passed the target returns to its natural state as if nothing had
> happened, like a Muto effect**. This means that a version of The Wound that
> Weeps (PeCo15), for example, briefly causes a painful, bleeding wound that
> immediately closes again. Likewise, Fist of Shattering (PeTe10) causes an object
> of stone or weaker material to briefly break apart, but then fuse itself
> together again. **The character's Ritual Perdo spells function normally,
> however.**" — ArMDE:6232
>
> DE 6232: "Die Perdo-Zauber des Charakters **können nichts dauerhaft zerstören**;
> sie stören das Ziel vorübergehend, ähnlich wie Perdo-Imaginem-Magie, aber sobald
> die Dauer verstrichen ist, **kehrt das Ziel in seinen natürlichen Zustand
> zurück**, als wäre nichts geschehen – wie bei einem Muto-Effekt. … **Die
> Ritualmagie-Perdo-Zauber des Charakters funktionieren jedoch normal.**"

**Current data.** `classification: "narrative"`, no effects, no parameters, no
description in either locale.

**Why `narrative` is wrong.** `narrative` asserts the passage states nothing
mechanical. This passage changes what one of the five Techniques *does* — a Perdo
effect stops being a Perdo effect and becomes, in the book's own words, a Muto
one — for every non-Ritual spell the character will ever cast. It is one of the
most sweeping single statements in the whole Flaws chapter, and the book itself
treats it as mechanical: it reaches for two named spells at named levels
(PeCo15, PeTe10) to demonstrate the consequence, and it carves out an exception
by casting mode ("Ritual Perdo spells function normally") — an exception is a
thing only a rule can have.

The second paragraph (ArMDE:6234, the Concentration-Duration rock trick) confirms
the reading from the other side: it is a worked tactical exploit of the rule,
which presupposes that the rule is real.

**It is inexpressible, which is D3's case and not `narrative`'s.** I checked the
three axes that could plausibly carry it and none fits.
`HalvableTotal` has four members (`spontaneous_casting`, `lab_enchanting`,
`lab_longevity`, `penetration`) and this is not a halving of a total.
`CastingScope` has the right *shape* for the Ritual carve-out
(`all|formulaic|ritual|formulaic_ritual|spontaneous`) but it scopes a **Casting
Total modifier**, not a spell outcome — there is nothing to add or subtract here.
`SpecialCasting`'s eleven kinds are all caster-side quirks (voice, gesture, aura,
improvisation); none touches what a spell does to its target. So D3 applies:
`uncomputed_rule`, with the rule written out.

**Partial credit where it is due.** Unlike most of this shape, the **main rule
does reach the player**: the summary is the passage's first sentence in full, in
both locales, and that sentence carries "cannot permanently destroy anything …
returns to its natural state … like a Muto effect". What is lost is the **Ritual
carve-out**, which is the half that matters to a magus planning his spell list —
he is told his Perdo is useless and not told his Rituals still work.

**What the correction is.** `classification: "uncomputed_rule"` with the whole
passage (or at minimum the first sentence plus the Ritual carve-out) in
`description` in both locales. No effect variant is added; no number changes.

**Severity: moderate, as a lost rule — but it is the largest single rule this
batch loses.** No number is miscalculated, because the engine computes nothing
either way, and the harm is that a magus with a Minor Hermetic Flaw is left
believing his Perdo Rituals are as crippled as his Formulaics.

---

### F-431 — `flaw.false_power` and `flaw.false_power_minor` — a Virtue is moved onto a second realm's interaction column, classified as flavour

> "One of the character's Supernatural Virtues **is associated with the Infernal
> realm, in addition to the realm with which it would normally be associated**,
> which causes it to **appear unholy when subjected to Divine or Infernal
> investigation**." — ArMDE:6082
>
> "Once the character recognizes that the Power has an unholy aspect, he may
> choose to treat it as an Infernal Power at any time, **using the Infernal Power
> column of the realm interaction chart and thus giving him a bonus in Infernal
> auras**. This may also grant him other benefits: like other infernalists he can
> **boost his casting total with sacrifices**, and use an **Infernal Ceremony** to
> include others in the activation, as described in Realms of Power: The
> Infernal, page 91." — ArMDE:6092
>
> DE 6082: "Eine der Übernatürlichen Tugenden des Charakters ist mit der
> Höllensphäre verbunden, **zusätzlich zu** der Sphäre, mit der sie normalerweise
> verbunden wäre, was dazu führt, dass sie **unheilig erscheint**, wenn sie einer
> göttlichen oder infernalen Untersuchung unterzogen wird."
>
> DE 6092: "…kann er sie jederzeit als Infernale Macht behandeln, **dabei die
> Infernale Macht-Spalte der Sphären-Interaktionstabelle verwenden und so einen
> Bonus in infernalen Auren erhalten**. … kann er seine **Zaubersumme durch
> Opfergaben erhöhen** und eine **Infernale Zeremonie** nutzen…"

**Current data.** Both entries: `classification: "narrative"`, `tainted: true`,
no effects, no description in either locale. The Major carries `max_total: 1`;
the Minor carries `prerequisites: {kind: has, value: flaw.false_power}` and no
`max_total`. Both declare
`parameters: [{key: "virtue", domain: "item", require_categories: ["hermetic",
"special","supernatural"], require_possessed: true, forbid_tainted: true}]`.

**Everything about this pair except its classification is excellent, and that is
worth saying first**, because it is the most carefully modelled entry in the
batch and `crates/arm-rules/RULES.md` documents it at length (`:1604`, `:1772-1822`,
`:1824-1870`). The Major/Minor split encodes ArMDE:6096's "in each subsequent
instance as a Minor Flaw rather than a Major one" with the correct asymmetry —
and `RULES.md:781-784` explicitly records that the pair is **deliberately not** a
magnitude-variant pair and must *not* be mutually `incompatible_with`, because
the two entries have to be held together. That is the right answer to check 8's
"what does the absence mean". `forbid_tainted: true` encodes "cannot apply to
Supernatural Virtues that are affiliated to the Infernal realm in the first
place" ✓, `require_possessed: true` encodes "that the character possesses" ✓, and
`validation/selections.rs::validate_possessed_param_targets` keys its claim map on
`(parameter key, target)` rather than on the item — both entries use the key
`"virtue"`, so a Major and a Minor cannot taint the same Virtue, which is exactly
"once for each appropriate Supernatural Virtue" ✓.

**Why `narrative` is nonetheless wrong.** Two clauses state mechanics.

*The realm re-alignment.* ArMDE:6082 puts a Supernatural Virtue on a **second
realm** simultaneously. The realm a Power belongs to is not colour in this game —
it selects a column of the realm interaction chart, it decides which auras help
and which hinder, and it decides what a Divine investigation sees. **RoP:I:3922
states the mechanic in the plainest possible terms**, in a passage the core
entry's own page reference leads to: "a False Shapeshifting Power might be
**Faerie for the purposes of realm interaction bonuses and penalties**, and
appear to be associated with the Faerie realm when subjected to magical or faerie
investigation. However, the Power would read as unholy when investigated by a
character with the Divine Power to Sense Holiness and Unholiness."

*The opt-in Infernal treatment.* ArMDE:6092 offers a standing election with three
named consequences — the Infernal column of the interaction chart, a casting-total
boost through sacrifice, and Infernal Ceremony. Those are three mechanics, one of
which ("boost his casting total") moves a total the engine computes.

**`tainted: true` is not a defence.** It records that the *Flaw* is
Infernal-associated for `validation/caps.rs::validate_tainted_cap`'s
half-the-points warning; it says nothing about the realm of the *Virtue the Flaw
names*, which is the whole subject of the passage. The two are different objects:
the Flaw is Tainted, and the Flaw's effect is that a *different* item becomes
Infernal-aligned. Nothing in the data expresses the second.

**It is inexpressible, which is D3's case.** The engine has no realm interaction
chart at all (`grep` over `crates/arm-rules/src` for `realm_interaction` returns
nothing; the only realm-adjacent surfaced value is
`MagicResistanceEffect::AuraBonus`, which Part C records as pushed with
`amount: 0` and therefore incapable of even carrying a magnitude). There is no
per-item realm field a Flaw could re-write. D3 therefore forbids `narrative` and
requires `uncomputed_rule` with the rule written out.

**What the correction is.** `classification: "uncomputed_rule"` on **both**
entries, with ArMDE:6082's realm sentence and ArMDE:6092's election written into
`description` in both locales. No effect variant, no parameter change, no data
beyond the class and the text. **`RULES.md` records no rationale for `narrative`
anywhere in its three False Power sections** — every one of them is about the
multiplicity and the target parameter — so nothing is being contradicted.

**Severity: moderate, as a lost rule on two entries.** No number moves. But this
is a *Major* Flaw whose entire content is the rule, and a player who takes it is
currently told only the summary sentence — which, to be fair, does carry
ArMDE:6082's first clause. What is wholly lost is ArMDE:6092's election, the one
part of the passage the player must act on.

---

### F-432 — `flaw.fettered_magic` — the same sentence B09 rated mechanical on the Virtue is `narrative` on the Flaw, and the stated incompatibility between them is encoded on neither side

> "**All of your spells and the effects of any magic items you activate are
> Arcane Connections to you. You cannot take this with the Virtue Tethered Magic,
> as the Virtue already includes this effect.**" — ArMDE:6116 (the entry's whole
> body)
>
> DE 6116: "**Alle deine Zauber und die Wirkungen aller Magieartefakte, die du
> aktivierst, sind Arkane Verbindungen zu dir. Du kannst dies nicht zusammen mit
> der Tugend Gefesselte Magie nehmen, da die Tugend diesen Effekt bereits
> einschließt.**"

**And the Virtue it names:**

> "You can pass control of your non-Ritual spells to others, just as if they were
> the caster, 'tethering' the magic to them for the spell's duration. You may also
> tether a spell to an object… **However, a side effect of this sort of magic is
> that all of your spells and the effects of any magic items you activate are
> Arcane Connections to you.**" — ArMDE:5143

**Current data.** `flaw.fettered_magic`: `classification: "narrative"`, no
effects, no `incompatible_with`, no description. `virtue.tethered_magic`:
`classification: "narrative"`, no effects, no `incompatible_with`.

**Part (a) — the classification.** The Flaw's first sentence *is* the Virtue's
last sentence, word for word. B09 read that sentence on the Virtue and wrote
**F-319**: "the sharpest — **every one of the magus's spells becomes an Arcane
Connection to him**, which is a defined game term with Penetration and targeting
consequences. `narrative` asserts the passage states nothing mechanical; it
states a standing drawback that any hostile magus can use." That reasoning
transfers intact, and it transfers *more* forcefully here, because on the Flaw
the sentence is not a side effect of something else — it is the entire entry. One
sentence cannot be mechanical in one place and colour in another.

D3 applies: the engine has no way to say that a *character* is an Arcane
Connection to his own spells. Re-derived rather than taken from B09, and with one
near-miss worth naming so a reader does not think it was overlooked:
`spell.rs::SpellRange::ArcaneConnection` **does** exist — but that is the spell
*Range* an author picks when designing a spell, i.e. "this spell reaches its
target through an Arcane Connection", which is the opposite end of the
relationship from "you are one". Nothing represents the second: `Effect` has no
variant for it, `Entity` has no field for it, and
`derived/casting.rs::penetration` reads only the casting total, the spell level
and the Penetration Ability. So **`narrative` → `uncomputed_rule`**, with the
rule in both locales. `virtue.tethered_magic` needs the same, which is F-319 and
is already on the correction list.

**Part (b) — the incompatibility.** ArMDE:6116's second sentence is an explicit,
unambiguous pair rule: *you cannot take this with that*. `jq` over
`rules/core/virtues_flaws.json` returns **no `incompatible_with` key on either
entry**. Nothing prevents a magus from buying Tethered Magic (a Minor Hermetic
Virtue, +1 point) and Fettered Magic (a Minor Hermetic Flaw, −1 point) together,
which is a free point for a drawback he already has — the book's own stated
reason for the prohibition ("as the Virtue already includes this effect").

`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
one-sided declaration, so the correction is necessarily both-sided:
`flaw.fettered_magic.incompatible_with = ["virtue.tethered_magic"]` **and**
`virtue.tethered_magic.incompatible_with = ["flaw.fettered_magic"]`.

**This is the audit's running pair-rule tally, and it moves.** B10 and B11 found
six stated pair rules with one encoded; B12 found `flaw.dwarf`'s three, all
encoded on both sides, taking it to nine stated / four encoded. This batch adds
two more stated rules — this one and F-435's — and encodes neither, taking the
count to **eleven stated, four encoded**, *as measured across B10-B13 only*.

**Part (c) — see F-433**, which is the German half and is a different defect with
a different fix.

**Which checks fail.** Check 2 (classification), check 8 (`incompatible_with`,
and its absence means nothing here — there is a stated rule and it is simply
missing), check 10 both directions, check 12 / D5.

**Severity: moderate on the classification (a lost rule), and moderate on the
incompatibility.** The pair rule is not a miscalculation either — the engine
would compute both entries' effects correctly if they had any — but it lets a
player bank a free Virtue point for a Flaw that costs him nothing, which is a
balance defect the rulebook wrote one sentence specifically to prevent.

---

### F-433 — three German Virtue/Flaw names are each shared by two different entries; one of the three is in this span, and the translation table already resolves it

**The instance in this span.** `flaw.fettered_magic` and `virtue.tethered_magic`
both ship the German name **`Gefesselte Magie`** in
`rules/i18n/de/virtues_flaws.json`. They are different entries with
different mechanics, and in the German UI they are hard to tell apart:
`ui/src/lib/components/VirtueFlawTab.svelte` renders each row as
`displayName(...)` beside a single magnitude badge, and **both entries are
Minor**, so the badge does not separate them either. The one thing that does is
the picker's kind partition — Virtues and Flaws sit in different columns — so a
player reading carefully can tell which is which *in the picker*. Nothing carries
that context onward: the selected-items list, the character sheet and the
Markdown export render the name, and a German sheet listing *Gefesselte Magie*
does not say whether the character gained a Virtue or paid for a Flaw.
*(Stated exactly as checked: I read `VirtueFlawTab.svelte`'s two `item-name`
render sites and confirmed both resolve through the i18n `name`; I did not audit
every downstream surface.)*
The consequence is at its sharpest on exactly the sentence F-432 is about: DE 6116
reads "Du kannst dies nicht zusammen mit der **Tugend Gefesselte Magie** nehmen"
while sitting under the heading `#### Gefesselte Magie`.

**The German source is where the collision originates, and it collides with
itself.** DE 5141 is `#### Gefesselte Magie` (Tethered Magic) and DE 6114 is
`#### Gefesselte Magie` (Fettered Magic). Both headings are in the same file.
So "follow the rulebook" — D6's precedence rule for factual disagreements — names
no answer here: the rulebook gives one name to two entries.

**The canonical table already resolves it, and the data does not follow.**
`rules/source/de/translation-tables/tugenden-fehler.md` carries both terms and
distinguishes them:

| line | row |
|---|---|
| `:73` | `\| Tethered Magic \| Gefesselte Magie \| \|` |
| `:701` | `\| Fettered Magic \| Gekettete Magie \| HoH:TL; Alle Zauber/aktivierte Gegenstände sind Arkane Verbindungen zum Zauberer \|` |

The `:701` note is the core Flaw's mechanic restated in German, so the row is
about this entry and not a homonym. `grundbegriffe.md:676` adds a third, related
term kept separate — `Tethered Casting → Gefesseltes Zaubern` — which shows the
table's author was tracking this cluster deliberately.

**Which authority wins for a *name*, and why I think the two agree.**
`rules/source/de/translation-tables/README.md:17-27` scopes the tables
explicitly: they are authoritative **for terminology** ("der deutsche Begriff für
einen englischen Term ist der Wert in der Spalte `Deutsch (DE)`") and *not* for
**Sachaussagen** — factual claims about the rules — where "gilt ausschließlich das
Regelbuch". The precedence ladder that follows ("1. Das Regelwerk … gewinnt immer
bei **Sachaussagen**") is therefore scoped to factual disagreements, which is how
D6 states it too. CLAUDE.md says the same from the other end: the tables "are the
**canonical EN→DE terminology mapping**: … the German label for any term whose
English form appears in a table MUST match the table's `Deutsch (DE)` value."
And `crates/arm-rules/RULES.md:785-789` records the same practice for this very
data file — DE names come from the line-mirrored German source,
"cross-checked against `rules/source/de/translation-tables/tugenden-fehler.md`
(**glossary wins on any term mismatch**)". Three authorities, one answer: for a
*name*, the table wins. I looked for a contradiction between RULES.md and D6 here
and did not find one once the README's scoping is read — recorded because I
briefly thought I had, and a verification pass should challenge it.

**So the correction is `flaw.fettered_magic` → `Gekettete Magie`**, and it is a
data change only, in `rules/i18n/de/virtues_flaws.json`. Not a table change:
the table is right. This is the *opposite* polarity from F-438's two rows, and
the difference is that here the rulebook offers no usable answer.

**And the collision is not unique. Two censuses, because the scope matters and
the first draft of this finding got it wrong.**

*Scoped to the 655 V/F ids this audit covers* — a `group_by` over the `name`
field of every `virtue.`/`flaw.`-prefixed key — the German locale has **three**
collisions and the English locale has **none**:

| German name | entries | English names |
|---|---|---|
| **Gefesselte Magie** | `virtue.tethered_magic`, `flaw.fettered_magic` | Tethered Magic, Fettered Magic |
| **Klatschbase** | `virtue.gossip`, `flaw.busybody` | Gossip, Busybody |
| **Unscheinbares Gesicht** | `virtue.forgettable_face`, `virtue.indescribable_face` | Forgettable Face, Indescribable Face |

*Unscoped — over the whole of each locale file* — there is **one more, and it is
in both locales**. The two i18n files carry **665** keys against
`rules/core/virtues_flaws.json`'s **655**, and four of the extras are
`folk_magic.*`:

| name | entries |
|---|---|
| **Böser Blick** / **Evil Eye** | `flaw.evil_eye` + `folk_magic.evil_eye` |

So the honest figures are **four duplicate German names and one duplicate English
name across the shipped locale files**, of which three German ones are inside
this audit's scope. The first draft of this finding reported the scoped figures as
though they were the whole story, and dismissed the English `Evil Eye` case as
benign ("different item classes") **without checking whether it recurs in
German** — it does. B12 audited `flaw.evil_eye` and did not catch it either.
Whether a name shared between a V/F and a folk-magic power is a defect at all
depends on whether the two are ever offered in one list, which is a question
about the `folk_magic.*` catalogue and outside this audit's 655 entries; it is
recorded here rather than numbered.

The other two are worth a line each because they are *not* the same case.
**`Klatschbase` is a known collision the table records and does not resolve** —
`tugenden-fehler.md:196` reads
`| Gossip | Klatschbase | Als Tugend (Informationsnetzwerk); nicht mit dem Fehler Busybody = Klatschbase (Fehler) verwechseln |`,
i.e. the glossary explicitly warns the reader not to confuse the two and then
assigns them the same word anyway (`:359` gives Busybody → Klatschbase flatly,
and `reputationen.md:135` gives Gossip → Klatschbase). That is a table-level
decision to live with the collision, which is a different problem from this one
and needs a different answer.
**`Unscheinbares Gesicht` has one table row for one of the two**
(`tugenden-fehler.md:193`, Forgettable Face) and none for Indescribable Face, so
the table neither causes nor resolves it.
All three are outside B13's span and all three are rated here rather than flagged
for a later batch, per the standing cross-reference rule — but the *fix* for each
needs a German term neither the table nor the rulebook currently supplies, and
the direction of the fix depends on **F-444**, so all three are left to the
orchestrator.

**Which checks fail.** Check 12 — the German name is wrong against the canonical
table, and is ambiguous in the product.

**Severity: moderate, and it is a localization defect of exactly the class
CLAUDE.md rates up.** Localization defects hit real users every session on every
platform. The sharpest instance is not the picker (whose kind partition rescues
it) but the sentence the Flaw itself states: a German player reading DE 6116
under the heading `#### Gefesselte Magie` is told he may not take *Gefesselte
Magie*, which is unreadable — and the incompatibility F-432 asks for would, once
encoded, produce an error message naming one entry and appearing to mean the
other.

---

### F-434 — `flaw.fluctuating_fortune` — a Flaw that alternates two named catalogue entries year by year, classified as flavour

> "The character's finances rise and fall like the tide, regardless of how
> successful he is or what preventative measures he takes. **He is considered to
> have the Wealthy Virtue one year, followed by the Poor Flaw the next.** Besides
> monetary concerns, this means that **the character will have to only work one
> season in one year, followed by a year in which he has to work three**, all to
> maintain his livelihood. This cycle of feast and famine continues throughout the
> character's life, always in opposition to his financial desires." — ArMDE:6152
>
> DE 6152: "…**Er gilt ein Jahr lang als Tugend Wohlhabend, gefolgt von einem Jahr
> mit dem Fehler Arm.** Abgesehen von monetären Sorgen bedeutet dies, dass der
> Charakter **in einem Jahr nur eine Jahreszeit arbeiten muss, gefolgt von einem
> Jahr, in dem er drei arbeiten muss**, um seinen Lebensunterhalt zu sichern."

**Current data.** `classification: "narrative"`, no effects, no description in
either locale. The displayed text is the summary, which is the passage's *first*
sentence and stops exactly before the Wealthy/Poor clause.

**Why `narrative` is wrong.** This is B11's F-406 composition idiom in its purest
form: the passage does not describe an effect, it **names two other catalogue
entries and applies them alternately**. Both targets are real, both are
`creation_effect`, and both compute:

| entry | ArMDE | the rule the Flaw invokes | data |
|---|---|---|---|
| `virtue.wealthy` | 5235-5238 | ":5237 You may spend **three seasons per year** on study or adventuring" | `later_life_xp_rate {amount: 20}` |
| `flaw.poor` | 6594-6596 | ":6596 You must work **three seasons per year** … you have **one fewer season** available for any form of advancement" | `later_life_xp_rate {amount: 10}` |

The Flaw's second sentence is not a paraphrase of the first — it is the
*seasons* half of both targets restated precisely: "only work one season in one
year" is Wealthy's three free seasons, "a year in which he has to work three" is
Poor's three worked seasons. Two named entries plus two season counts is a
mechanical clause by any reading.

**The engine cannot express it, and the reason is specific rather than general.**
`LaterLifeXpRate` (A12) **replaces** the base rate rather than adding to it, and
when several selections name a rate the **lowest wins** (`min`, deliberately, so
the result does not depend on declaration order). An oscillation is therefore
structurally unrepresentable: authoring both rates on one entry would resolve to
Poor's 10 and make the Flaw strictly worse than Poor, and authoring neither is
what ships. There is no alternating or averaging axis anywhere in the effect set.
D3 → `uncomputed_rule`.

**The arithmetic that makes the silence *defensible* is not the same as a reason
to call it `narrative`, and this is the part worth being careful about.** The base
rate is `rules/core/life_stages.json:39`, `xp_per_year: 15`; Wealthy is 20 and
Poor is 10; alternating them across a career averages to **15**, which is exactly
the base. So computing nothing is very probably the *right numeric answer*, and I
am **not** recommending an effect. But "the net effect is zero" is a statement
about the engine's output, and `classification` is a claim about the rulebook.
The rulebook says something, at length, with numbers in it.

**One asymmetry the correction must not smuggle in.** Both targets exclude magi
outright — ArMDE:5237 "As all Hermetic magi are supported by their covenant, **no
magi may take this Virtue**" and ArMDE:6596 "**this Flaw is not available to
magi**" — while `flaw.fluctuating_fortune` is a Minor Supernatural Flaw stating
no such restriction and carrying no `entity_kinds` or profile restriction. Whether
"considered to have the Wealthy Virtue" drags Wealthy's magus exclusion along with
it is a rules question the passage does not answer; I did not settle it and I am
not asserting either way. It is not a finding, because the current data takes the
permissive reading and the book does not contradict it in so many words.

**What the correction is.** `classification: "uncomputed_rule"`, with the
Wealthy/Poor alternation and the one-vs-three-seasons rule written into
`description` in both locales. No effect.

**Severity: moderate, as a lost rule.** No number is wrong — the averaging above
is why — but a player who takes this Flaw is told only that his finances "rise and
fall like the tide", and is never told the two entries that define what that
means or the season counts he must plan around.

---

### F-435 — `flaw.failed_student` — the stated Doctor in (Faculty) exclusion is encoded on neither side

> "The character has studied for a specific university license and failed his
> final examination. If he passed earlier exams, he may have a scholastic Social
> Status Virtue showing how far he got. **(Obviously, this cannot be Doctor in
> (Faculty).)** The character has a Bad Academic Reputation of 2." — ArMDE:6074
>
> DE 6074: "…Wenn er frühere Prüfungen bestanden hat, besitzt er möglicherweise
> eine akademische Tugend des Sozialen Status, die zeigt, wie weit er gekommen
> ist. **(Offensichtlich kann dies kein Doktor in (Fachbereich) sein.)** Der
> Charakter hat eine Schlechte Akademische Reputation der Stufe 2."

**Current data.** `classification: "creation_effect"` with one effect,
`grants_reputation {academic, 2}` — **correct** against "a Bad Academic
Reputation of 2" and recorded at `RULES.md:5377`. No `incompatible_with`, no
description in either locale.

**Why the exclusion is a rule and not a parenthetical aside.** The bracket and
the word "Obviously" make it read like commentary, which is presumably why it was
passed over, but grammatically and mechanically it is a hard prohibition: *this*
refers to the scholastic Social Status Virtue the previous sentence permits, and
the sentence rules one specific entry out of that set. A character who failed his
final examination cannot be a Doctor in his Faculty; that is the entire premise of
the Flaw.

`virtue.doctor_in_faculty` (heading ArMDE:3683, body ArMDE:3685) is real, is the
top of the scholastic ladder, and is the most valuable entry in it: a **Major**
Social Status Virtue carrying
`grants_reputation {academic, 3}` *and* `restricted_ability_xp {amount: 300,
categories: ["academic"]}`. So the un-encoded pair permits a character who holds
both a 300-point Academic pool for the doctorate he earned and a Bad Academic
Reputation of 2 for the examination he failed — with the two Academic Reputations
(+3 and −2) sitting on the same sheet.

`jq` over `rules/core/virtues_flaws.json` returns **no `incompatible_with` key on
either entry**. The correction is necessarily both-sided, because
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
one-sided declaration:
`flaw.failed_student.incompatible_with = ["virtue.doctor_in_faculty"]` and
`virtue.doctor_in_faculty.incompatible_with = ["flaw.failed_student"]`.

**Which checks fail.** Check 8 — a stated pair rule with nothing encoded, and its
absence means nothing here except that it was missed. Check 10, both directions.
Check 12 / D5 — the exclusion reaches the player in neither locale.

**Severity: moderate.** It is not a miscalculation and not a false rejection; it
is a missing guard that lets an illegal combination through, on a
`ValidationMode::Enforced` path where every other stated pair rule the audit has
found encoded *is* caught. The player-facing harm is a buildable character the
rulebook forbids.

---

### F-436 — `flaw.feral_scent` — a −1 social penalty and a doubled spell area, in neither locale

> "Perhaps due to his beast blood or feral upbringing, the character has the
> strong natural smell of a wild animal. Humans tend to avoid being too close to
> him, and may be on edge in his vicinity without knowing why. He may well spook
> domesticated animals if he surprises them. **He suffers a -1 penalty to social
> interactions (which stacks with the penalties imposed by The Gift, if he has
> it), and develops a negative Reputation of Unclean at level 2.** However,
> **if he has Initiated the Sensory Magic Mystery (described in Houses of Hermes:
> Mystery Cults, page 27), any spells he casts with a Scent Target have twice
> their normal area of effect.**" — ArMDE:6108
>
> DE 6108: "…**Er erleidet –1 auf soziale Interaktionen (was sich mit den Abzügen
> durch Die Gabe stapelt, falls er sie besitzt) und erwirbt eine negative
> Reputation als Unrein der Stufe 2.** Hat er jedoch das Mysterium der Sinnesmagie
> eingeweiht …, **haben alle Zauber, die er mit einem Duft-Ziel wirkt, die
> doppelte normale Reichweite.**"

**Current data.** `classification: "creation_effect"` with one effect,
`grants_reputation {local, 2}`. No description in either locale.

**The Reputation is right and is triply corroborated.** ArMDE:6108 gives level 2
and names no audience beyond the local one implied by "Humans tend to avoid being
too close to him"; `types.rs::ReputationType` offers `Local`, `Ecclesiastical`,
`Hermetic`, `Academic`, of which only `Local` fits; and
`rules/source/de/translation-tables/reputationen.md:110` independently records
`| Feral Scent | Wildgeruch | Lokal | 2 (–) | Unrein |` — audience Lokal, level 2,
negative, label "Unrein". Three sources, one answer ✓. (`RULES.md:5378` records
it too.) The **name** on that same table row is a separate matter and is F-438.

**Two clauses reach the player nowhere, and both are mechanical.**

*The −1 to social interactions.* A signed numeric modifier, stated with an
explicit stacking rule against a named Virtue. I checked whether it is
expressible and it is not: there is no social-interaction axis anywhere in the
42-variant `Effect` enum. The closest, `AbilityRollMod` (A41), is surfaced-only,
carries a *free-text* subject with no Ability field at all, and would name no
Ability here because "social interactions" is not one Ability. The definitive
evidence is the cross-reference: `virtue.the_gift`, whose penalties this one
"stacks with", ships `narrative` with **no effects at all** (ArMDE:3967-3970 is
three lines that defer entirely to page 63), as does `virtue.gentle_gift`. The
engine does not model the Gift's social penalty, so there is nothing for this one
to stack onto. D3 → the rule can only be text, and it is not text either.

*The doubled Scent-Target area.* "twice their normal area of effect" is a
multiplier on a spell parameter, conditional on an Initiation the app does not
model. I followed the page reference to **HoH:MC:971-987** (`#### Sensory Magic
(Minor House Mystery)`) and read it in full: it defines the Scent/Sound/Spectacle
Targets and imposes seven restrictions on them, and it says **nothing** about
doubled area. So the doubling is stated *only* here, in the core Flaw's own
sentence — it is this entry's rule outright, and it is lost.

**Which checks fail.** Check 10, both directions — two passage clauses with no
effect. Check 12 / D5 — neither clause is in `description` in either locale, and
the entry has no `description` at all. Check 2 survives: `creation_effect` is
right, because the Reputation grant is a computed creation effect; D5 is
explicitly the ruling that a computed entry still owes its uncomputed leftovers.

**What the correction is.** `classification` unchanged. Both clauses into
`description` in both locales, with the −1 written as an **ASCII hyphen**
(the EN source uses ASCII at :6108, the DE source an en dash; the shipped text
must use ASCII either way).

**Severity: moderate, as a lost rule — with one aggravating detail.** The −1 is a
*number the player must apply by hand at the table*, and it is the Flaw's main
cost. The Reputation the engine does grant is the lesser half. So the sheet shows
the character's penalty as "a bad local Reputation" and omits the modifier he
actually rolls with.

---

### F-437 — `flaw.foreign_upbringing` — the Native-Language requirement reaches neither locale

> "…**His Native Language is one foreign to the saga, and he needs to learn
> another language in order to communicate with the rest of the turb.** The
> maximum scores at character creation for locality-dependent Abilities like
> Language, Area Lore, or Organization Lore, as well as some social Abilities,
> are half (round up) that which his age normally allows." — ArMDE:6160
>
> DE 6160: "…**Seine Muttersprache ist dem Rest der Saga fremd, und er muss eine
> andere Sprache erlernen, um mit dem Rest der Turba kommunizieren zu können.**
> Die Höchstwerte bei der Charaktererschaffung für ortsabhängige Fertigkeiten wie
> Sprache, Gebietskundige oder Organisationskunde sowie für einige soziale
> Fertigkeiten betragen die Hälfte (aufgerundet) dessen, was sein Alter
> normalerweise erlaubt."

**Current data.** `classification: "creation_effect"` with one effect,
`locality_ability_cap_fraction {num: 1, den: 2}`. No description in either
locale.

**The computed half is correct, and unusually well documented.** This entry is
the **only** shipped carrier of the variant. `effective/reputation_and_caps.rs::ability_age_cap`
computes `fractioned = ceil(base_cap · num / den)` as `(base·num + den − 1)/den`
— **rounded up**, which is exactly "half (round up)" ✓ — and applies it only to
Abilities the catalogue marks `locality_dependent`
(`ability.area_lore`, `ability.dead_language`, `ability.living_language`,
`ability.organization_lore`). `RULES.md:6408-6428` records the whole thing,
including that the passage's trailing "as well as some social Abilities" is
**deliberately** unimplemented because "the passage … names no Abilities, so none
are flagged — the engine enforces the flag it is given rather than deciding which
social Abilities a saga counts." That is a recorded decision and I do not contest
it; flagging more is a data edit with no code change, exactly as RULES.md says.

**What is not covered anywhere is the sentence before it.** "His Native Language
is one foreign to the saga, and he needs to learn another language in order to
communicate with the rest of the turb" is a creation-time requirement with a
concrete consequence: the character must buy a **second** Living Language, and
the one he buys is subject to the very half-cap the next sentence imposes
(`ability.living_language` is one of the four `locality_dependent` Abilities). So
the two sentences interact, and the player is shown only the second one — and
shown it only as a silently narrowed cap in the Ability tab, since the entry has
no `description` in either locale.

The engine expresses neither half of it: there is no effect that *requires* an
Ability be bought, and `AbilityScoreGrant` grants a floor rather than imposing an
obligation. D5 therefore applies whatever the classification.

**Which checks fail.** Check 10, both directions — a passage clause with no
effect and no text. Check 12 / D5. Check 2 survives; `creation_effect` is right.

**What the correction is.** `classification` unchanged, effect unchanged. The
Native-Language sentence into `description` in both locales. Whether the "some
social Abilities" clause should join it there is arguable — RULES.md justifies
the *engine's* silence, not the *description's* — and I include it in the
recommendation on D5's plain wording, flagging that it is the weaker half.

**Severity: low-to-moderate, as a lost rule.** Nothing is miscalculated and the
entry's headline mechanic works correctly. The harm is that a player is told his
caps are halved without being told why, and is not told he owes a second language.

---

### F-438 — two translation-table rows in this repository's copy disagree with the German rulebook

Per **D6** these are reported as *"this repository's copy disagrees with the
rulebook"*; whether each is a genuine error in the source project
`arm-de-translation` or a stale copy here cannot be determined from inside this
repository (the source project is outside the working directory and the read is
denied). B12 found five such rows and all five turned out to be genuine errors in
both projects, so the prior is that these are errors too — but the determination
is the orchestrator's.

Both rows are **terminology** rows, and per
`rules/source/de/translation-tables/README.md:17-22` the tables *are* the
authority for terminology, so the direction of the fix is not self-evident. In
both cases the shipped data follows the **DE rulebook heading on the
line-parallel line**, which is the German source of truth CLAUDE.md names for the
rulebook text itself; the table is the outlier. I state both and rate neither.

**Read this together with F-444, which is the reason I rate neither.** Under D6's
precedence as written (rulebook > thematic table > `tugenden-fehler.md`, no
scope) these are **table** errors and the shipped German is right. Under the
README's, RULES.md's and CLAUDE.md's scoping — the tables are authoritative for
*terminology*, the rulebook for *Sachaussagen* — these are **data** errors and
the shipped German should be *Wildgeruch* and *(Form-)Monstrosität*. The two
binding documents give opposite answers, and `reputationen.md` is additionally a
*thematic* table, which the README's ladder ranks above `tugenden-fehler.md`,
making row 1 the harder of the two to overrule either way.

| # | English | table row | DE rulebook heading | shipped `rules/i18n/de/` | note |
|---|---|---|---|---|---|
| 1 | Feral Scent | `reputationen.md:110` — `\| Feral Scent \| Wildgeruch \| Lokal \| 2 (–) \| Unrein \|` | **DE 6106 `#### Wilder Geruch`** | `Wilder Geruch` | The table's *factual* columns on this row are correct and corroborate the data (Lokal, 2, negative, "Unrein" — see F-436); only the name column disagrees. `tugenden-fehler.md` has no Feral Scent row at all, so `reputationen.md` is the sole table source and it is a reputation table making a naming claim. |
| 2 | Form Monstrosity | `tugenden-fehler.md:343` — `\| Form Monstrosity \| (Form-)Monstrosität \| Platzhalter (Form) ersetzen \|` | **DE 6162 `#### (Form) Missgeburt`** | `{form} Missgeburt` | The German rulebook uses *Missgeburt* consistently — the heading at 6162, the body at 6164/6166, and the nested example table's own heading at DE 6170 (`> #### Beispiele für Missgeburten`). *Monstrosität* appears nowhere in the DE passage. The table's `Anmerkung` ("Platzhalter (Form) ersetzen") is correct and matches the data's `{form}` placeholder ✓. |

**One row that is not a disagreement but is worth recording.**
`tugenden-fehler.md` carries the row `| Flawed Parma | Fehlerhafte Parma | |`
**twice**, verbatim, at `:271` and `:297`. Both agree with the data and with
DE 6142; the duplication is housekeeping, not an error of fact.

**Severity: low, as a data-provenance defect — but it is a generator, which is
why D6 exists.** Neither row changes a shipped string today (the data follows the
rulebook in both cases). The cost is forward-looking: CLAUDE.md makes the tables
the canonical EN→DE mapping, so the next agent generating German text for either
term reproduces the table's value, and the error outlives the audit.

### F-439 — `virtue.lone_redcap` — the same double-count as F-428, on an entry three batches back that nobody checked for it (B05's span)

This finding is outside B13's span and is rated here rather than flagged,
per the standing rule. It was reached not by a cross-reference but by the census
F-428 required: having established that a `restricted_ability_xp` grant scoped to
a life-stage window the engine already funds is double-counted, I checked whether
any of the other 27 carriers has the same shape. One does.

> "You are a Redcap who does not maintain ties to a Mercer House, and thus do not
> receive magic items or Longevity Rituals. **You still begin with 300 experience
> points for your fifteen years spent as an apprentice**, and receive the benefits
> of the Well-Traveled virtue, but you are estranged from the other Redcaps in
> your area, and have a poor Reputation at level 2 within your House."
> — ArMDE:4321

**Current data.** `virtue.lone_redcap`, `creation_effect`, carrying among others
`restricted_ability_xp {amount: 300, categories: [...]}` (the categories are
B05's **Q-41**, a separate question).

**The mechanism, verified.** Lone Redcap is a **Minor Social Status** Virtue, so
its holder is a companion or grog, not a magus — and
`life_stage.rs::apprenticeship_of` returns `None` for anyone whose profile is not
`is_magus`, with the doc comment saying so outright ("a grog or companion serves
no apprenticeship at all"). Consequently `apprenticeship_years` is **0** for this
character, and `life_stage.rs::later_life_years` computes
`stop_age − childhood.years − 0` — so the **fifteen apprentice years the passage
is paying for are already inside the later-life span**, funded at
`rules/core/life_stages.json:39`'s `xp_per_year: 15`. The Virtue's 300-point pool
is then added on top by `effective/xp.rs::build_flow_pools`, exactly as in F-428.

**The arithmetic, for a 30-year-old Lone Redcap.** Later life = 30 − 5 = 25
years × 15 = **375** general XP, plus the Virtue's **300** restricted =
**675**. On the passage's own accounting the fifteen apprentice years are worth
300 and the remaining ten are worth 150, for **450**. The engine over-funds by
**225**, and the over-funding scales with age.

**Why this is weaker evidence than F-428, stated plainly.** Feral Upbringing's
120 is numerically *identical* to the block it displaces (ArMDE:2378's 75 + 45),
over an identically-named five-year window, with a substituted Ability list and
an explicit contradiction of the displaced half — four independent confirmations.
Lone Redcap has only the one: the phrase "**for** your fifteen years spent as an
apprentice", which attributes the 300 to a span the engine is separately paying
for. There is no named block whose total the 300 matches, because the
apprenticeship block (240) is magus-only and this character never reaches it. So
the *mechanism* is verified and the *reading* is not, which is **Q-108**.

**B05 audited this entry and did not look for this.** Its verdict row
(`batch-05.md:246`) reads "OK … `supernatural` in the 300-point pool is unsourced
(Q-41)", and its findings on the entry are F-151 (D5 leftovers), F-152 (the
German name), Q-41/42/43. None concerns the life-stage overlap — which is not a
criticism of B05: the question only becomes visible once F-428 makes the shape
legible. `crates/arm-rules/RULES.md` mentions the 300 only as a reuse example
("Lone Redcap's 300 apprenticeship xp — use all five categories", `:5199-5200`)
and records no decision about the window.

**Severity: high if Q-108 resolves toward "replaces", nil if it resolves toward
"supplements"** — which is exactly why it is escalated rather than asserted. If
it replaces, this is a second silent over-funding on the recommended entry path,
on a Virtue rather than a Flaw, and it scales with the character's age. I am
**not** recommending a data change on my own reading.

### F-440 — `flaw.fickle_nature` — obeying the Flaw raises two hard errors, and the rulebook contradicts itself about why

**Found by the verification pass, on an entry this file had passed on all twelve
checks.** It is F-429's family — "doing exactly what the Flaw says is a validator
error" — and it is the batch's second live false hard error.

> "The character swings between two types of personality behaviors that are
> directly opposite. There is no middle ground, so the character is always either
> displaying traits of one behavior or the other. **Select a Personality Trait at
> +4, and its opposite at +4.** Typical Personality Traits are: Happy and Sad,
> Energetic and Lazy, Confident and Diffident, or Proud and Humble." — ArMDE:6124
>
> DE 6124: "…**Wähle eine Persönlichkeitseigenschaft mit +4 und ihr Gegenteil mit
> +4.**"

**Current data.** `classification: "uncomputed_rule"`, `magnitude: "minor"`,
`categories: ["personality"]`, no effects, full passage in `description` in both
locales. The classification and the text are **right** and are not what fails.

**What fails is the interaction with a validator the entry never mentions.**
`validation/scores.rs::validate_personality_traits` implements ArMDE:2500-2502
and ArMDE:2820: it counts the character's **Major** Personality Flaws, gives one
±4-to-±6 slot per Major Flaw, and errors on every trait beyond that budget.

```rust
let major_personality_flaws = entity.selections.iter().filter(|s| {
    … item.has_category(ENGINE_REQUIRED_CATEGORY_PERSONALITY)
        && item.magnitude == Magnitude::Major
}).count();
…
let mut over_three_budget = major_personality_flaws;
for trait_ in traits {
    let magnitude = trait_.value.unsigned_abs();
    if magnitude > 6 { issues.push(personality_out_of_range(trait_, 6)); }
    else if magnitude > 3 {
        if over_three_budget > 0 { over_three_budget -= 1; }
        else { issues.push(personality_out_of_range(trait_, 3)); }
    }
}
```

`flaw.fickle_nature` is **Minor**, so it contributes **0** to
`over_three_budget`. A character who enters the two +4 traits ArMDE:6124
*mandates* therefore receives **two `personality_trait_out_of_range` errors**,
and `personality_out_of_range` builds a `ValidationIssue::error`, not a warning,
filed on `CreationPhase::PersonalityReputations`. Even a character who also holds
a Major Personality Flaw buys off only one of the two — and he cannot hold two,
because ArMDE:2820 and every profile's
`{"category":"personality","max":1,"major_only":true,"hard":true}` cap him at
one.

**And underneath it is a genuine book-internal contradiction, which is why the
correction is not obvious.** ArMDE:2502 says: "Choose a few words to describe
your character's personality, and attach a value **between -3 and +3** to each.
… If you have a **Minor** Personality Flaw, you should represent that by a
Personality Trait with a score of **+3 or -3**, and a **Major** Personality Flaw
should have a Personality Trait of **+6 or -6**." ArMDE:6124 asks a **Minor**
Personality Flaw to produce **two +4** traits. Both sentences are in the same
book, and the engine implements the first as a hard error against the second.
That is **Q-110** and it must be settled before anything is changed: relaxing
the validator per-carrier, recording the conflict, and reading the +4 as a typo
are three different corrections with three different blast radii.

**Which checks fail.** Check 11 — the engine rejects a build the rulebook
mandates. Checks 2 and 12 survive intact, which is exactly why a
classification-and-text sweep passed the entry: nothing about the *entry's own
data* is wrong. The defect is only visible if you ask what happens when a player
does what the description tells him to.

**Severity: high**, on F-429's grounds — a false hard error in the default
validation mode, blocking a character the rulebook explicitly describes. It is
arguably worse than F-429, because here the app itself prints the instruction
(the rule is in `description` in both locales) and then refuses the result.

---

### F-441 — `flaw.gabai` — the Free-Social-Status compatibility clause reaches neither locale

**Overturns this file's own clean verdict**, caught by the verification pass, and
the reason is a split I applied correctly elsewhere in the same batch and then
failed to apply here.

> "Appointed by the community council, the gabai is the local tax collector. It
> is his responsibility to ensure that Jewish taxes, payable to the community on
> such things as wine and meat, are collected from all adult males. The gabai has
> a –2 negative Reputation of "Tax Collector" in his community. **This Flaw is
> compatible with any other Free Social Status Virtue.**" — ArMDE:6200
>
> DE 6200: "…**Dieser Fehler ist mit jeder anderen freien Sozialer-Status-Tugend
> vereinbar.**"

**Current data.** `classification: "creation_effect"`, one effect
`grants_reputation {local, 2}` — correct, and triply corroborated (ArMDE:6200,
`reputationen.md:115` "Gemeinde (jüdisch) | 2 (–) | Steuereintreiber",
`RULES.md`'s Reputation-grant list). Key set `["name","summary"]` in both
locales, and the summary is the passage's **first** sentence, which stops three
sentences before the clause.

**Why the original verdict was wrong.** This file's cross-reference table
dismissed the clause with "Not counted against the entry; it is F-427." That
conflates two different obligations. **F-427 is a finding about the *engine***:
no profile carries a `social_status` cap, so the one-Social-Status rule
ArMDE:2816 states is enforced nowhere. **D5 is an obligation on the *text***:
"any mechanical clause the engine does not compute must be written into
`description`, in both locales, **whatever the entry's classification**." An
exemption from an unenforced rule is still a mechanical clause about which
selections are legal, and it is computed by nothing. Both are true at once, and
the entry owes the text regardless of what the engine does.

The proof that this is a real inconsistency and not a technicality is two rows up
in the same verdict table: **F-429's second half charges `flaw.failed_monk` with
exactly this**, a D5 obligation on a clause whose engine-side gap is F-409's.
Gabai is the one `creation_effect` entry in the batch that escaped the same test.

**What the correction is.** `classification` unchanged, effect unchanged. The
compatibility clause into `description` in both locales. Whether the Reputation's
*negativity* should also be carried is a different matter and is **not** this
finding: B10's **F-380** established catalogue-wide that `types.rs::Reputation`'s
`score` is a `u8` with no sign, so no shipped Reputation can be marked "bad", and
that is recorded once rather than per entry.

**Severity: low-to-moderate, as a lost rule.** No number is wrong and nothing is
falsely rejected — the clause is a *permission*, and the rule it permits an
exception to is not enforced anyway, so nothing blocks the player today. The harm
is forward-looking: when F-427 is fixed and the one-Social-Status cap starts
firing, a Gabai character will be rejected for a combination this sentence
explicitly allows, and the sentence will still be nowhere in the app.

---

### F-442 — `flaw.greater_malediction` — a magnitude-calibration rule and a named immunity, classified as flavour; and `flaw.lesser_malediction` is the same

**Escalated by the verification pass and resolved into a finding here**, on
evidence neither pass had when the verdict table was written.

> "You have been cursed by some supernatural power, in a way that greatly hinders
> you. **The effects of the curse should be comparable to those of other Major
> Flaws. Indeed, almost any Flaw could be the result of a curse.**" — ArMDE:6212
>
> DE 6212: "…**Die Auswirkungen des Fluchs sollten denen anderer Großer Fehler
> vergleichbar sein. Tatsächlich könnte fast jeder Fehler das Ergebnis eines
> Fluches sein.**"

**Current data.** `classification: "narrative"`, no effects, no parameters, no
description in either locale. `flaw.lesser_malediction` (ArMDE:6342-6345, Minor
Supernatural, B15's span) is word-for-word the same shape — ArMDE:6344, "The
effects of the curse should be about as bad as other **Minor** General Flaws" —
and is also `narrative` with no effects.

**Two pieces of evidence, and the second is the one that settles it.**

*First, the in-batch counter-example.* `flaw.flawed_powers` (ArMDE:6148) says
"She **suffers the effects of a Major Hermetic Flaw** (commonly Restriction or
Necessary Condition)". That is the identical construction one category over — an
unnamed Flaw of a stated magnitude whose effects the character takes on — and it
is `uncomputed_rule`, with `RULES.md:4818-4820` recording it explicitly as a
"selection prerequisite the engine does not express". Two entries, one shape, two
classifications. One of them is wrong, and the one with a recorded rationale is
not the one to move.

*Second, and decisively: the book states a mechanical rule about this Flaw
somewhere else, and names it.* A grep of the whole rulebook for "Malediction"
returns **11 hits** — two "List of Flaws" index links, two entry headings, two
back-of-book page-index rows, four sample-character stat lines, and exactly one
rule. That one is **ArMDE:7397**, the Curse-Throwing Supernatural Ability:

> "Curse-Throwing cannot affect Flaws; specifically, **someone with the Lesser or
> Greater Malediction Flaw is beyond the power of Curse-Throwing**, unless it is
> a Flaw imposed by a faerie or magician with a limited duration."

That is an immunity to a modelled Supernatural Ability
(`ability.curse_throwing`), conditioned on holding this exact Flaw, with its own
carve-out. `narrative` asserts the book states nothing mechanical *for this
entry*; the book states an absolute about it, by name, and the same sentence
covers the Minor twin. **This cross-reference is not in the entry's own cited
range, which is why every pass so far has missed it** — it is reachable only from
the Abilities chapter, and I found it only because the verification pass grepped
the whole book for "Malediction" rather than reading 6210-6213.

**Both clauses are inexpressible.** No effect variant calibrates a Flaw's
severity against a magnitude class, and no variant confers immunity to a named
Supernatural Ability. D3 → `uncomputed_rule`, with the rule written out.

**What the correction is.** `classification: "uncomputed_rule"` on
`flaw.greater_malediction` **and** on `flaw.lesser_malediction`, with ArMDE:6212
/ ArMDE:6344's calibration and ArMDE:7397's immunity in `description` in both
locales. The two must move together; **B15 must not re-litigate the Minor twin**,
which is why it is rated here rather than flagged.

**Severity: moderate, as a lost rule on two entries.** The immunity is the part
that matters at the table: a player whose character is cursed has no way to learn
from the app that the ordinary remedy does not work on him.

---

### F-443 — `types.rs::Effect::SoakMod`'s doc comment states a shipped value wrongly, at exactly the place a reader would check it

This is a **source** defect rather than a data one, and it is recorded because
the audit's own method walked past it. `crates/arm-rules/src/types.rs`, the
`SoakMod` variant:

> `/// A flat modifier to Soak (Tough +3, Frail −1). Consumed by `derived.rs`
> `soak()` (5i).`

with `/// Source: ArMDE:5145-5147 (Tough),` and the Frail range beneath it.

**`flaw.frail` ships `soak_mod {amount: -3}`, and ArMDE:6192 says "He has a -3
penalty to his Soak score."** The doc comment says **−1**. Tough's +3 is right;
only the Frail half is wrong. (`virtue.tough`'s own ArMDE:5147 "+3 bonus on your
Soak score" ✓.)

The data is correct and no number is miscalculated, so no entry fails on it —
`flaw.frail` stays clean. What makes it worth a number is *where* it sits: this
doc comment is the one-line summary a reader consults to check a Soak sign, it is
the natural place to verify exactly the claim this batch had to verify, and it is
wrong. It is the same class of defect as the `AgingMod` doc comment
`engine-semantics.md`'s correction log records ("Three of the five kinds are
consumed" — there are eight, and seven are consumed), and the same argument
applies: fixing it is a code change outside a batch's remit, but leaving it
unrecorded means the next reader trusts it.

**Severity: low.** Documentation only, no runtime effect, caught by nothing
because no test reads a doc comment.

---

### F-444 — `decisions.md` D6 states the EN→DE precedence unscoped, contradicting the three documents it summarises — and this batch applied both readings at once

**This is a defect in the audit's own binding decision record**, found by the
verification pass while attacking F-433, and it is the reason F-433 and F-438
point in opposite directions.

**What the three primary sources say.** All three scope the rulebook's primacy to
*factual* claims and give the tables primacy over *terminology*:

- `rules/source/de/translation-tables/README.md:19-23` — "Sie sind für die
  **Terminologie** maßgeblich — der deutsche Begriff für einen englischen Term ist
  der Wert in der Spalte `Deutsch (DE)`. Sie sind **nicht** maßgeblich für
  **Sachaussagen** über die Regeln … Dafür gilt ausschließlich das Regelbuch."
  And `:25-26` — "Rangfolge bei Widerspruch: 1. Das Regelwerk … gewinnt immer
  **bei Sachaussagen**."
- `crates/arm-rules/RULES.md:785-789` — DE names come from the line-mirrored
  German source, "cross-checked against …`tugenden-fehler.md` (**glossary wins on
  any term mismatch**)".
- **CLAUDE.md** — the tables "are the **canonical EN→DE terminology mapping**: …
  the German label for any term whose English form appears in a table MUST match
  the table's `Deutsch (DE)` value."

**What D6 says.** `docs/vf-audit/decisions.md`, under "Precedence when sources
disagree (also in `rules/source/de/translation-tables/README.md`)":

> rulebook > thematic table > `tugenden-fehler.md`

with **no "bei Sachaussagen" qualifier** — presented as the general rule, and
introduced as a restatement of the README it does not in fact restate. D6 is
*binding* and is the document a batch agent is told to read before starting; the
README is not in the required-reading list.

**And this batch applied both readings, which is how the contradiction became
visible.** F-433 lets the **table** beat the rulebook (`flaw.fettered_magic` →
*Gekettete Magie*, because the rulebook collides with itself). F-438 lets the
**rulebook** beat the table on two rows that are equally pure terminology
(`reputationen.md:110` *Wildgeruch* vs shipped *Wilder Geruch*;
`tugenden-fehler.md:343` *(Form-)Monstrosität* vs shipped *{form} Missgeburt*).
Under the README/RULES.md/CLAUDE.md reading, **F-438's two rows are data defects,
not table defects**, and the shipped German is wrong — the opposite conclusion to
the one F-438 reaches. `reputationen.md` is additionally a *thematic* table,
which the README's ladder ranks **above** `tugenden-fehler.md`, making it the
harder of the two to overrule.

I do **not** resolve this, and I have deliberately left F-438 stated as a
disagreement rather than a verdict, exactly as D6's own "check the source project
before concluding the table is wrong" instruction requires. What I am reporting
is that the audit cannot currently be consistent about it: B12 found five rows of
this shape and rated them all as table errors; on the README's reading, some of
those may have been data errors.

**What the correction is.** Either add the `Sachaussagen` scope to D6 — one
clause — or, if Norbert's intent really is that the rulebook wins on terminology
too, say so in D6 *and* reconcile it with CLAUDE.md's "MUST match the table's
`Deutsch (DE)` value", which would then be wrong. It cannot stay as it is: two
binding documents currently give opposite answers to the commonest German finding
in this audit.

**Severity: moderate, and it is a *generator*, which is D6's own stated reason
for existing.** Every batch from here on will meet this shape, and each will
resolve it whichever way it happens to read first; the correction pass will then
encode a mixture.

## Open questions

### Q-103 — `flaw.false_power`'s `require_categories` admits any Hermetic or Special Virtue, where the book says "Supernatural Virtues"

Both False Power entries declare
`require_categories: ["hermetic","special","supernatural"]` on their `virtue`
parameter. The passage says "One of the character's **Supernatural Virtues**"
(ArMDE:6082) and then widens by example: "This Flaw can apply to Supernatural
Virtues that define the character's background, like **Faerie Blood, Diedne
Magic, or even The Gift**" (ArMDE:6082). Faerie Blood is `supernatural`, Diedne
Magic is `hermetic`, The Gift is `special` — so the three categories are
reverse-engineered from the three examples, which is a defensible reading and is
recorded as such at `RULES.md:1841-1846`.

**But `require_categories` is a membership test, not an example list.** Admitting
the whole `hermetic` category admits every Hermetic Virtue — Cautious Sorcerer,
Free Study, Method Caster — none of which is a "Supernatural Virtue" and none of
which the passage contemplates. The data is over-permissive by construction, and
the alternative (an explicit whitelist alongside `supernatural`) is uglier but
tighter. **What would settle it:** a ruling on whether the book's three examples
are meant to open two whole categories or to name three specific entries.
RoP:I:4415 words it identically and adds only a pointer ("see Chapter 12: Black
Magic, The False Gift"), so the supplement does not resolve it either.

### Q-104 — `flaw.flawed_powers` records no parameter for the Major Hermetic Flaw it imports, where `flaw.false_power` does for its target

> "She suffers the effects of a **Major Hermetic Flaw** (commonly Restriction or
> Necessary Condition), but it is applied to her Supernatural Virtues rather than
> to her Hermetic magic (if any)." — ArMDE:6148

The entry passes every check: `uncomputed_rule` is right, the whole passage is in
`description` in both locales, and `RULES.md:4818-4820` explicitly records its
two inexpressible clauses ("at least one Major Supernatural Virtue" and the
Hermetic-Flaw exclusion) as *selection prerequisites the engine does not express*.
I do not contest that; a RULES.md rationale I had missed is exactly what made B10
withdraw a finding, and I am not repeating that mistake.

**The residue is an asymmetry, not a defect.** `flaw.false_power` faces the same
shape — a Flaw that names another catalogue item — and solves it with an
`item`-domain parameter (`{key: "virtue", require_possessed: true,
forbid_tainted: true}`), so the save records *which* Virtue is tainted and the
sheet can print it. `flaw.flawed_powers` records nothing, so a character sheet
says "Flawed Powers" and never says *which* Major Hermetic Flaw's effects apply
or to *which* Supernatural Virtue. **What would settle it:** a decision on
whether that choice is worth an `item`-domain parameter
(`require_categories: ["hermetic"]`, plus a magnitude constraint the
`ParameterDef` shape cannot currently express) or whether text is sufficient. The
second half of that parenthesis is itself a possible reason to say no: nothing in
`ParameterDef` can require the named item be **Major**, so the parameter would be
half-enforced.

### Q-105 — is `flaw.form_monstrosity`'s "1 pawn of Muto vis" a mechanical clause?

> "Often, **1 pawn of Muto vis may be extracted from the corpse** of a monstrous
> character, found concentrated in the monstrous feature." — ArMDE:6166
>
> DE 6166: "Häufig kann aus dem Kadaver eines monsterhaften Charakters **1 Bauer
> Muto-Vis** gewonnen werden, der sich im monsterhaften Merkmal konzentriert
> findet."

The entry ships `narrative` with a `form` parameter and no effects, and I could
not settle whether that survives. **For mechanical:** it is a concrete quantity
of a resource the game models, in a named Art, and the audit has already ruled
that a number stated in words is still a number. **For colour:** it is hedged
twice ("Often", "may be"), it concerns the character's *corpse* rather than the
character, no player-facing total moves, and the same passage declares the entry
mechanically inert in so many words — "The feature may be beneficial in limited
circumstances, but **generally gives the character no advantage over its
non-mutated peers**" (ArMDE:6164).

**What would settle it:** a ruling on whether a post-mortem vis yield is a rule
of the Flaw. If it is, the class is `uncomputed_rule` and the sentence goes into
both locales; if it is not, `narrative` stands unchanged. I mark the entry
escalated rather than clean, per "doubt is escalated, never resolved quietly".

### Q-106 — what shape should F-428's correction take?

F-428 establishes that `flaw.feral_upbringing`'s 120 points **replace** the
standard childhood block rather than adding to it, and that the engine adds them.
It does not establish the fix, and the three candidates differ a great deal in
cost.

**First, a prior sub-question the verification pass raised and I accept: replace
*how much*?** Strict replacement of the whole block gives **120** for the first
five years. But ArMDE:6112 only *explicitly* forbids the Language half ("you may
not start with a score in a Language"), so a reading in which the Flaw replaces
the 75-point native-language pool and leaves the 45-point spread standing —
45 + 120 = **165** — is not absurd. The decisive evidence for full replacement is
the 120 = 75 + 45 coincidence over an identically-named window, which is
circumstantial rather than stated; it should be weighed, not assumed. Then the
shape:

1. **A new `Effect` variant** that suppresses the childhood pools (and, ideally,
   substitutes its own spread list). Honest, and it would also give the two
   prohibitions a home, but it is a 43rd variant and nine exhaustive matches to
   update.
2. **A `childhood_package`-shaped data row.** The field already exists —
   `life_stage.rs::LifeStagePlan::childhood_package`, an `Option<Id>` whose doc
   reads "`None` for a character who divided the childhood experience by hand" —
   and
   `ArMDE:2382` says packages "can be taken to speed up character generation".
   A Feral package would be data, not code. But a package is something the
   *player* picks, and this one is compelled by a Flaw, so something must still
   force the link.
3. **Leave the pool and subtract the childhood total**, which no current effect
   can express and which would in any case still hand the character the Living
   Language the Flaw forbids.

**What would settle it:** Norbert's ruling on which of the three, and on whether
the correction should also express "you may not start with a score in a Language"
(which F-410/F-426 establish nothing currently can). Until it is settled the
entry stays on the correction list with the defect described and the fix open —
and it should be settled together with **F-439**, which is the same shape on a
second entry.

*Scope of the census behind F-439, stated exactly as checked:* `jq` returned the
**28** entries carrying a `restricted_ability_xp` effect, and each of those 28
passages was grepped for life-stage-window language
(`first five years`, `apprenticeship`, `childhood`, `later life`, `per year`).
Three hits came back — `virtue.craft_guild_training` :3615,
`virtue.lone_redcap` :4321 and `flaw.feral_upbringing` :6112 — and all three were
then read in full. Craft Guild Training is clean ("Like Warrior and Educated,
this Virtue **gives a bonus of** 50 experience points" — explicitly additive, and
"during his apprenticeship" is narrative framing, not a budget window). The other
two are F-428 and F-439. I make **no** claim that a fourth spelling of the idiom
would not find more; the census is a floor over one effect variant.

### Q-107 — should the two Maledictions also carry an *open grant*, on top of F-442's reclassification?

> "You have been cursed by some supernatural power, in a way that greatly hinders
> you. **The effects of the curse should be comparable to those of other Major
> Flaws. Indeed, almost any Flaw could be the result of a curse.**" — ArMDE:6212
>
> DE 6212: "…**Die Auswirkungen des Fluchs sollten denen anderer Großer Fehler
> vergleichbar sein. Tatsächlich könnte fast jeder Fehler das Ergebnis eines
> Fluches sein.**"

**F-442 settles the classification** (`narrative` → `uncomputed_rule`, for both
Maledictions). What it does not settle is whether the pair should *also* record a
player choice, and that is a genuinely separate question with a genuinely
available mechanism.

This is not B11's F-406 idiom: F-406 is "**includes the effects of** [one named
entry]", encodable as `Effect::GrantsSelection` over a fixed set, and ArMDE:6212
names no entry and is doubly subjunctive. But the pair is arguably the
catalogue's one genuine **open** "pick another entry's effects" construct — a
Major Malediction whose effect *is* some other Major Flaw — and
`grant.rs::GrantConstraint` already expresses exactly that shape (`kind` +
`magnitude` + require/forbid categories) for House and Mythic-Companion slots.

**The obstacle is structural and is worth recording, because it is the D3 ground
for F-442 rather than an argument against this question.** `Grant::Open` is
reachable only from a House or a Mythic-Companion **type profile**, via a
`choice_key` on the entity. The only grant machinery a V/F entry can carry is
`Effect::GrantsSelection { items: BTreeSet<Id> }` — a **fixed** set, with no
constraint fields. So the engine has the shape and a V/F cannot reach it.

**What would settle it:** a ruling on whether "almost any Flaw could be the
result of a curse" is an instruction to *choose* one at creation — in which case
both Maledictions want a constrained open grant and `GrantsSelection` needs to
grow one — or a remark about what curses are like, in which case F-442's text-only
correction is the whole fix. I lean to the second; the entry is now a finding
either way, so nothing hangs on the lean.

### Q-108 — does `virtue.lone_redcap`'s 300 experience points *replace* the funding for its fifteen apprentice years, or supplement it?

F-439 verifies the mechanism — a non-magus serves no apprenticeship block, so his
apprentice years are funded as ordinary later life at 15/year, and the Virtue's
300-point pool is added on top. What it cannot settle from ArMDE:4321 alone is
whether "You still begin with 300 experience points **for** your fifteen years
spent as an apprentice" means *instead of* the ordinary later-life earnings for
those years or *in addition to* them.

Two readings, both arguable. **"Replaces":** the preposition "for" attributes the
300 to that span, and 15 years × 15 XP = 225 is close enough to 300 that the
Virtue reads as a modest upgrade of a Redcap's training rather than a doubling of
it. **"Supplements":** the word "still" frames the sentence as *preserving* a
benefit against the losses the rest of the passage lists (no magic items, no
Longevity Ritual), which is the grammar of a bonus; and `virtue.redcap` itself
(ArMDE:4842-4851) carries only an `item_level_budget`, so there is no sibling
grant to compare the 300 against.

**What would settle it:** a reading of the ordinary Redcap's training rules — the
Mercer House section this Virtue is the exception to — to see whether 300 is
stated anywhere as a Redcap's standard apprenticeship allowance, plus Norbert's
call on whether "for your fifteen years" is a budget window or a description. If
it replaces, F-439 is a live over-funding and needs the same correction shape as
Q-106; if it supplements, the data is correct as shipped and F-439 is withdrawn.

### Q-109 — `flaw.fury` and `virtue.berserk` state the same condition and the catalogue encodes it two contradictory ways

> `flaw.fury`, ArMDE:6196 — "**While enraged** you get **+3 to Damage**, but
> **-1 on all other scores and rolls**." → `uncomputed_rule`, no effects.
>
> `virtue.berserk`, ArMDE:3502 — "**While berserk**, you get **+2 to Attack and
> Soak scores**, but suffer a **-2 penalty to Defense**." → `in_play_effect`,
> carrying `combat_mod {attack, +2}`, `combat_mod {defense, -2}`,
> `soak_mod {+2}`, all folded **flat** by `derived.rs::in_play_mods`.

One condition ("while enraged" / "while berserk"), one engine (which gates
nothing on a condition), two opposite encodings. This file rated Fury
`uncomputed_rule` on the ground that no effect gates a condition — which is true
— without noticing that the catalogue's other conditional-combat-bonus entry does
the opposite. **B01's F-20 already rates Berserk's flat fold a defect** ("an
un-enraged Berserk character shows +2 Soak and +2 Attack he does not have"), so
the audit has a live finding on one side and a clean verdict on the other for the
same shape, and neither batch cross-references the other.

**What would settle it:** F-20's resolution. If Berserk's conditional bonuses are
to be *removed* and the rule written out, Fury's `uncomputed_rule` is right and
the two converge on Fury's side. If they are to be *kept* and gated by some new
mechanism, Fury becomes an `in_play_effect` carrying
`combat_mod {damage, +3}` and the two converge on Berserk's side. Until then
Fury's verdict is not safe, and I have marked it escalated rather than clean.
Note the asymmetry that makes Fury the harder of the two even under the second
answer: "-1 on **all other scores and rolls**" has no carrier at all, so Fury
could never be fully computed the way Berserk can.

### Q-110 — ArMDE:6124's two +4 traits contradict ArMDE:2502's ±3 for a Minor Personality Flaw

F-440 establishes that entering the traits `flaw.fickle_nature` mandates raises
two hard errors. It does not establish the fix, because the two rulebook
sentences cannot both be satisfied:

> ArMDE:2502 — "attach a value **between -3 and +3** to each … If you have a
> **Minor** Personality Flaw, you should represent that by a Personality Trait
> with a score of **+3 or -3**, and a **Major** Personality Flaw should have a
> Personality Trait of **+6 or -6**."
>
> ArMDE:6124 — "[Fickle Nature, ***Minor**, Personality*] **Select a Personality
> Trait at +4, and its opposite at +4.**"

Three candidate corrections, with very different blast radii. **(a) Per-entry
exemption:** `validate_personality_traits` gains a notion of a Flaw that grants
its own trait slots — honest, but it needs a data field, and nothing in
`PointItem` currently expresses "this Flaw authorises N traits at magnitude M".
**(b) Record the conflict:** leave the validator alone, treat ArMDE:6124 as a
book error, and note it — cheapest, but it leaves the app rejecting what its own
description tells the player to do. **(c) Read the +4 as a typo for ±3:** the
least invasive, and the most dangerous, because inventing a number the book does
not state is precisely the `7f5605a` aging-floor mistake this audit exists to
stop.

**What would settle it:** Norbert's ruling. Note that ArMDE:2502 hedges
("**should** represent") where ArMDE:6124 commands ("**Select**"), which argues
against (c) and mildly for (a) — but that is an inference, not a reading, and I
am not acting on it.

## Sub-agent reconciliation

One verification pass was run against this batch with the sole brief of
overturning it. It re-derived all 24 entries this file first called clean, all
six high-stakes findings, and every Method claim. **It overturned two clean
verdicts, escalated three more, corrected two Method claims, narrowed one
finding's census, confirmed all six findings it was asked to attack, and
contributed five observations this file had missed** — one of which is the
batch's second live false hard error.

Prior passes overturned 2 of 9, 1 of 5, 0 of 11, 1 of 18 and 0 of 20. This one
overturned **2 of 24**, which is the worst ratio since B11 — and the pattern is
worth naming, because both misses have the same cause.

### What it overturned, and why this file was wrong

| # | Entry | This file said | The pass found | Now |
|---|---|---|---|---|
| 1 | `flaw.fickle_nature` | **clean** — class, data and text all correct | Entering the two +4 traits the Flaw *mandates* raises **two hard `personality_trait_out_of_range` errors**, because `validation/scores.rs::validate_personality_traits` gives a ±4-to-±6 slot only per **Major** Personality Flaw and this one is Minor | **F-440**, high, plus **Q-110** |
| 2 | `flaw.gabai` | **clean** — the clause "is F-427, not counted against the entry" | F-427 is a finding about the **engine**; **D5** is an obligation on the **text**, and it is unconditional. The compatibility clause is in neither locale | **F-441** |

**Both misses have one cause, and it is a method error rather than a reading
error.** This file checked each entry's *own data* against its *own passage*
thoroughly — and twice failed to ask the next question: *what happens when a
player does what this entry says?* For Fickle Nature the answer lives in a
validator the entry never mentions and does not carry an effect for; for Gabai it
lives in the difference between an engine gap and a text obligation. I applied
exactly the second distinction correctly to `flaw.failed_monk` two rows earlier
in the same table (F-429's second half) and then did not apply it to Gabai. **The
lesson for the next batch: a `narrative`/`uncomputed_rule` entry with no effects
is not thereby insulated from the validators — an entry can be perfectly authored
and still be unbuildable**, and that is invisible to any check that reads only
the entry.

### What it escalated

| # | Entry | Why |
|---|---|---|
| 3 | `flaw.greater_malediction` | The pass declined my `narrative` on two grounds — the in-batch parallel with `flaw.flawed_powers` (same shape, `uncomputed_rule`, rationale recorded at `RULES.md:4818-4820`), and **ArMDE:7397**, which I had not read: Curse-Throwing "cannot affect Flaws; specifically, someone with the Lesser or Greater Malediction Flaw is **beyond the power of Curse-Throwing**". I verified ArMDE:7397 and accepted: it is a named immunity to a modelled Ability. Resolved into **F-442**, which also carries `flaw.lesser_malediction` (B15's span) so B15 does not re-litigate it. |
| 4 | `flaw.fury` | `virtue.berserk` states the identical "while enraged/berserk" condition and ships `in_play_effect` with flat `combat_mod`/`soak_mod`; **B01's F-20** already calls that a defect. Two encodings, one shape, and neither batch knew about the other. Verified from the data. Now **Q-109**. |
| 5 | `flaw.fragile_constitution` | Sign **confirmed** independently (Rapid Convalescence +3 / Fragile Constitution −3, and the roll is one the character wants to win). Escalated only on the D5 scope question — ArMDE:6188 says "wounds **and diseases**" and the surfaced label is the bare slug. I kept it **clean** and folded the nuance into the Part C bullet rather than raising a finding, because Part C's Phase-0 note says a non-computed `health_mod` track is *correctly authored against an engine that lists rather than computes it*. **The pass's sharper point stands and is not resolved**: B11 rated 7 of 7 of its `in_play_effect` entries as dropping clauses and called it the batch's largest defect, while this batch rates 3 of 3 as fine. The threshold moved between batches and nothing records why. That is a question for the orchestrator, not for either batch. |

### What it corrected in the Method

| claim | outcome |
|---|---|
| "**32** `####` headings in 6068-6235 … 32 headings for 35 ids" | **WRONG, and already fixed before the pass reported** — there are **31** top-level headings (plus the blockquoted `> #### Monstrosity Examples` at 6170, which is not a sibling). 31 + 3 Major/Minor pairs + the False Power pair = 35. Both of us caught the same error independently. |
| "Failures: 12. Clean: 23" | **WRONG.** 24 rows were marked clean and 11 failed; the parenthetical explaining the gap was self-contradictory. Corrected before the pass reported; the counts are now 15/20 after the overturns above. |
| Index row "5543-5546" for three Supernatural-Minor entries | **Imprecise** — ArMDE:5545 is `[Folk Magic](#folk-magic)`, so the three are **5543, 5544, 5546**. Verified and corrected below. |
| Line parity | **CONFIRMED mechanically** by a `diff` of the two files' heading line numbers over 6050-6260 — no output, 42 headings including the nested one. Stronger than my own check, which covered 6060-6298 by eye. |
| `source.lines` 35/35, `source.anchor` 7/7, checks 3-6 35/35, the census 19/7/6/3, the truncation scan, the U+2212 scan, the ArMDE:2814 three-instance claim, the F-427 re-derivation, ArMDE:2818/:2820 enforcement, `flaw.greater_malediction` not dual-listed | **all CONFIRMED**, each with the same file+line evidence. |

### What it confirmed among the findings

All six it was asked to attack survived. Three were **strengthened**:

- **F-428** — the pass added two facts I did not have, and both make it worse.
  `validation/life_stage.rs` raises `life_stage_native_language_unset` as a hard
  **error** for any life-stage character, so the engine *forces* a Feral
  Upbringing character to name the native language ArMDE:6112 forbids him a score
  in — on top of the `restricted_xp_unspent` warning nudging him to spend the
  surplus. It also confirmed there is **no** suppression path of any kind, that
  the flat `AbilityFunding::Pool` path is unaffected, that `RULES.md` records no
  additive decision, and that **B11's F-387 is not in conflict** — F-387
  contrasted "grant" against "earmark" and never asked whether the granted pool
  replaces a block, so it answered a different question. And it noted that a
  Minor Flaw granting 120 would be the largest XP grant in the catalogue bar
  apprenticeship and Master Bard, which is independent corroboration. Its caveat
  — that the passage supports 120 (full replacement) *or* 165 (replacing only the
  forbidden Language half) — is folded into **Q-106**.
- **F-431** — independently searched all 16 `false_power` hits in `RULES.md` and
  confirmed **none** records a rationale for `narrative`; also confirmed RoP:I:3922
  and RoP:I:4425 say what I quoted.
- **F-434** — added the argument I had not made: encoding both rates would resolve
  to **10**, not 15, because `LaterLifeXpRate` takes the `min`. So the naive fix
  is *worse* than the silence, which is a D3 case rather than a null one.

**F-429, F-430 and F-432(a)(b) were confirmed unchanged**, each re-derived end to
end from the code.

### The one place it overturned a claim of mine, and I accept it

**F-433's census was wrong as written.** I reported "exactly **three**" duplicate
German V/F names and "**zero**" in the English V/F namespace. Re-derived without
the `virtue.`/`flaw.` prefix filter, the two shipped locale files contain
**665** keys against `rules/core/virtues_flaws.json`'s **655** — the extra ten
include **four `folk_magic.*`** entries — and the true figures are **four
duplicates in DE and one in EN**:

| German | entries | English | entries |
|---|---|---|---|
| **Böser Blick** | `flaw.evil_eye` + `folk_magic.evil_eye` | **Evil Eye** | `flaw.evil_eye` + `folk_magic.evil_eye` |
| Gefesselte Magie | `virtue.tethered_magic` + `flaw.fettered_magic` | — | |
| Klatschbase | `virtue.gossip` + `flaw.busybody` | — | |
| Unscheinbares Gesicht | `virtue.forgettable_face` + `virtue.indescribable_face` | — | |

My figures are right **only** under the scope I stated (`virtue.*`/`flaw.*` keys),
and I did name the `Evil Eye` case in the EN half — but I asserted it was benign
("different item classes") without checking whether it recurs in German, and it
does. `Böser Blick` is an **unflagged collision in both locales**, and B12
audited `flaw.evil_eye` without catching it. F-433 is corrected below to state
both scopes.

### Observations it contributed that this file had missed

Five, beyond the two overturns and three escalations already listed. Three became
findings — **F-440** (Fickle Nature), **F-441** (Gabai), **F-443**
(`SoakMod`'s doc comment says "Frail −1" where the entry ships −3 and ArMDE:6192
says −3) — one became **F-444** (D6's unscoped precedence contradicting the
README, RULES.md and CLAUDE.md, with this batch applying both readings at once),
and two are recorded here without a number because neither is a defect of an
entry in this span:

- **B10's F-380 is not named in my "Part C systemic gaps" section although four
  of this batch's entries are carriers.** `types.rs::Reputation`'s `score` is a
  `u8` with no sign, so no shipped Reputation can be marked "bad" or "negative" —
  which is what ArMDE:6062, :6070, :6074, :6108 and :6200 all say. I named C2
  (`grants_reputation`'s `score` unenforced) for the four granters and not F-380.
  Corrected in that section below.
- **`flaw.the_falling_evil`'s shipped DE description silently normalises a source
  typo.** DE 6078 closes with a straight ASCII `"` (`„göttliche Krankheit"`) and
  the shipped text uses U+201D; DE 6200 has the same straight-quote typo. Benign,
  arguably an improvement, and not a hyphen-rule matter — but my Method says the
  seven `uncomputed_rule` entries carry "the full cited passage in both
  languages ✓", and that is not byte-verbatim. Recorded so a future byte-level
  comparison does not read it as corruption.

### Its own open questions, and what I did with them

Six. Three I have adopted as **Q-106**'s caveat (the 120-vs-165 reading),
**Q-109** (Fury vs Berserk) and **Q-110** (ArMDE:6124 vs ArMDE:2502). Two became
findings: the terminology-precedence contradiction is **F-444**, and the
Greater/Lesser Malediction classification is **F-442**. The sixth — *is a name
shared between a V/F and a folk-magic entry a defect at all, given the two are
picked from different lists?* — I leave open and unnumbered, because it is a
question about `folk_magic.*`, which is outside this audit's 655-entry scope
entirely; it belongs to whoever audits that catalogue.
