# Batch B14 — indices 455-489, ArMDE:6236-6369

Entries: 35. Audited: 35. Failures: **12**. Escalated, not checked: **6**
(`flaw.hobbled`, `flaw.horrifying_appearance_snake_legs`,
`flaw.independent_craftsman`, `flaw.inscribed_shadow`, `flaw.jinxed`,
`flaw.lame`). Clean: **17**. 12 + 6 + 17 = 35.
Findings **F-445 … F-459** — five reach outside the batch: **F-450** rests on
`virtue.famous` (**B03**'s span, ArMDE:3861-3864), **F-452** and **F-453** both
land on an *inbound* pointer at **ArMDE:7397** (`#### Curse-Throwing*`, the
Supernatural-Abilities chapter, ~1,050 lines past the Flaws block), **F-456**
rests on `flaw.crippled` (**B11**'s span, ArMDE:5877-5880), **F-457** carries
`flaw.vulnerable_to_folk_tradition` from **B19**'s span with it, and **F-459**
rests on `virtue.self_confident` (**B08**) and `virtue.ferocity` (**B03**).
Open questions **Q-111 … Q-117**.

**The verification sub-agent re-derived all 22 entries this file first called
clean and overturned three — `flaw.hobbled`, `flaw.lame` and
`flaw.low_self_esteem` — escalated two more, confirmed every one of the
fourteen findings (narrowing three, widening two, and correcting the proposed
fix on a third), and caught three counting errors in this file's own Method,
one of which had the headline census exactly backwards.** It also found the
batch's second live wrong number, on an entry I had passed on all twelve
checks. Of the five entries whose verdicts changed, one is now a finding
(**F-459**) and four are open questions (**Q-116**, **Q-117**). See the
reconciliation section.

**The batch's first shape is a miscalculation the audit has not seen before: an
effect that is right in sign and size and applied to rows the rule never
touches.** `flaw.low_tolerance`'s `health_mod { fatigue_penalty, -1 }` is folded
uniformly over **all five** Fatigue tiers by `derived/combat.rs::fatigue_levels`,
but ArMDE:17129 says "Each Fatigue level **above Winded** has a penalty
associated with it" — Fresh and Winded have none. A character who takes this
Minor Flaw is shown **"Fresh: -1"** and **"Winded: -1"** on his own sheet
(`DerivedVitalsSection.svelte:40` prints every tier's penalty), i.e. a permanent
-1 to every roll at full vigour, which the book never gives him (**F-449**).
This is the batch's only wrong number and its highest severity.

**The second shape is the one this batch's brief predicted: two `narrative`
entries describe characters the app then refuses to build.** Doing exactly what
`flaw.imagined_folk_tradition_vulnerability` permits — "purchase a score of 1
(but no more) in Faerie Lore" — raises a hard `ability_category_requires_virtue`
error for every non-magus, because `ability.faerie_lore` is `arcane` and the
entry carries no `ability_authorization` (**F-445**, F-409's thirtieth
instance). And doing exactly what `flaw.incompatible_arts` permits — "This Flaw
may be taken repeatedly with different combinations" — raises a hard
`duplicate_selection`, because the entry is **unparameterized** with no
`max_total`, so `max_per_target`'s default of 1 keyed on `(item_ref, {})`
forbids the second copy the book grants (**F-446**). Neither is visible to an
entry-versus-passage comparison; both fall straight out of asking what happens
when a player does what the entry says.

**The third shape is a stated restriction that has a Prereq variant waiting for
it and is encoded on no entry in the span.** `flaw.hermetic_patron` opens with
a whole sentence of restriction — "You must be a Redcap or magus to take this
Flaw" (ArMDE:6250) — and ships `narrative` with no `prerequisites`, although
`Prereq::IsMagus` and `Prereq::Has` both exist and `virtue.magic_items` already
encodes "Redcap only" with the latter (**F-448**). `flaw.judged_unfairly` states
a prohibition *and* an explicit incompatibility with "any Virtue that gives you
such a Reputation", against 15 catalogue Virtues that do, and carries an empty
`incompatible_with` (**F-447**).

**The fourth shape is the one this file missed on its own and the verification
pass found: an entry whose absolute a *Virtue* quietly defeats.**
`flaw.low_self_esteem`'s `confidence_bonus { score: -1, points: -3 }` lands
exactly on 0/0, which is why I passed it — but `effective/gift_confidence.rs`
sums additively and `incompatible_with` is absent on all **three** of the
catalogue's `confidence_bonus` carriers, so a companion holding Low Self-Esteem
*and* Self-Confident validates clean and ships **Confidence Score 1, Confidence
Points 2** (**F-459**) — a figure neither passage sanctions and a flat
contradiction of ArMDE:6364's "**never** have any Confidence Points". Nothing in
an entry-versus-passage comparison can see this; the question that finds it is
what happens when two legal entries meet.

*(Written incrementally: header, method and the 35-row verdict table first,
findings appended one at a time, sub-agent reconciliation last.)*

## Method

**Both languages were read as continuous prose before any entry was judged** —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` 6225-6384 and
the line-parallel `rules/source/de/Ars Magica Definitive Edition Basisregeln.md`
6225-6384, deliberately overrunning the span at both ends so the first and last
entries' boundaries could be seen against their neighbours
(`flaw.hallucinations` 6226 and `flaw.harmless_magic` 6230 before,
`flaw.lycanthrope` 6370, `flaw.magic_addiction` 6378 and `flaw.magical_air` 6382
after).

**Line parity holds throughout the span and through both overruns.**
`grep -n "^#### "` over each file, restricted to 6230-6378, returns
**byte-identical line-number sequences**: 6230, 6236, 6240, 6244, 6248, 6256,
6260, 6264, 6268, 6272, 6276, 6280, 6284, 6290, 6294, 6298, 6302, 6306, 6310,
6314, 6318, 6322, 6326, 6330, 6334, 6338, 6342, 6346, 6350, 6354, 6358, 6362,
6366, 6370, 6378. That is 32 headings inside 6236-6369 for 35 catalogue ids: the
three Major/Minor pairs `Hatred` 6236, `Higher Purpose` 6256 and `Lecherous`
6334 each ship as two ids citing one heading. No nested or blockquoted `####`
appears anywhere in the span, so unlike B13 there is no sibling-heading
ambiguity to resolve.

**The batch's own classification census matches the brief's.** Re-derived rather
than trusted: `jq` over the 35 ids returns **16 `narrative`, 8
`uncomputed_rule`, 4 `creation_effect`, 7 `in_play_effect`** (16+8+4+7 = 35).
The brief's figures are correct. **19 of 35 carry a non-`narrative` class**, as
the brief said — the highest proportion so far — and the consequence showed up
exactly where the brief predicted it would: this batch's single miscalculation
(F-449) is on an `in_play_effect` entry whose data matches its passage word for
word and whose *consumer* does something the passage does not say.

**Check 1 — `source.lines`: 34 of 35 correct, one failure.** Every range but one
runs from its own `####` heading to the line before the next heading.
`flaw.infamous` ships `[6310, 6312]` where the next heading is at 6314, so the
range stops one line short (**F-455**). It is the only entry in the span whose
range does *not* end on the blank line before the next heading — the convention
every other entry follows and the subject of B12's **Q-97**; no new instance of
that question is counted. Two ranges are longer than four lines and both are
correct: `flaw.hermetic_patron` 6248-6255 (three paragraphs) and
`flaw.impious_friend` 6284-6289 (two paragraphs), each ending on the line before
the next top-level heading ✓.

**Check 1, second half — `source.anchor`: 9 of 35 entries carry one, all nine
correct**, verified twice — derived from the `####` heading, and against the
book's own "List of Flaws" index links, which spell them out literally.

| id | anchor | heading | index link |
|---|---|---|---|
| `flaw.horrifying_appearance_snake_legs` | `horrifying-appearance--snake-legs` | ArMDE:6264 `#### Horrifying Appearance – Snake Legs` | ArMDE:5396 |
| `flaw.hunchback` | `hunchback` | ArMDE:6272 | ArMDE:5590 |
| `flaw.hunger_for_form_magic` | `hunger-for-form-magic` | ArMDE:6276 `#### Hunger for (Form) Magic` | ArMDE:5397 |
| `flaw.inconstant_magic` | `inconstant-magic` | ArMDE:6298 | ArMDE:5441 |
| `flaw.independent_craftsman` | `independent-craftsman` | ArMDE:6302 | ArMDE:5592 |
| `flaw.indiscreet` | `indiscreet` | ArMDE:6306 | ArMDE:5359 |
| `flaw.inscribed_shadow` | `inscribed-shadow` | ArMDE:6318 | ArMDE:5547 |
| `flaw.limited_magic_resistance` | `limited-magic-resistance` | ArMDE:6346 | ArMDE:5443 |
| `flaw.lingering_injury` | `lingering-injury` | ArMDE:6350 | ArMDE:5597 |

The two non-trivial ones are both right: the en dash in `Horrifying
Appearance – Snake Legs` is dropped and its two flanking spaces become the
double hyphen `--`, and the parentheses in `Hunger for (Form) Magic` are
stripped. **The one `uncomputed_rule` entry with no anchor is `flaw.jinxed`** —
B12's `flaw.evil_eye` and B13's `flaw.fish_out_of_water_terrain` observation for
the third batch running. Not a defect of the entry (B11's **Q-96** is the
catalogue-wide question), but worth the line: the anchor that *would* be correct
for it is `jinxed`, spelled out at ArMDE:5594. Two anchors sit on entries whose
class does not oblige one at all
(`flaw.horrifying_appearance_snake_legs` is `narrative`,
`flaw.limited_magic_resistance` is `in_play_effect`), so the anchor population
is not simply "the `uncomputed_rule` set".

**Checks 3, 4, 5, 6 (`kind` / `magnitude` / `entity_kinds` / `categories` +
`tainted`): 35 of 35 correct**, each read against its own descriptor line one by
one:

- `*Major or Minor, Personality*` — hatred_\* (6237), higher_purpose_\* (6257), lecherous_\* (6335)
- `*Minor, Hermetic*` — hedge_wizard (6241), incompatible_arts (6291), inconstant_magic (6299), infamous_master (6315), limited_magic_resistance (6347), loose_magic (6355)
- `*Minor, Story*` — heir (6245), hermetic_patron (6249)
- `*Minor, Story, Tainted*` — impious_friend (6285)
- `*Minor, General*` — hobbled (6261), hunchback (6273), incomprehensible (6295), independent_craftsman (6303), infamous (6311), jinxed (6323), judged_unfairly (6327), lame (6331), lingering_injury (6351), low_tolerance (6367)
- `*Major, General*` — leprosy (6339), low_self_esteem (6363)
- `*Major, Supernatural*` — horrifying_appearance_snake_legs (6265), hunger_for_form_magic (6277)
- `*Minor, Supernatural*` — inscribed_shadow (6319), lesser_malediction (6343)
- `*Minor, Personality*` — humble (6269), imagined_folk_tradition_vulnerability (6281), lost_love (6359)
- `*Major, Story*` — indiscreet (6307)

`kind` is `flaw` on all 35 ✓ — the whole span is inside the book's Flaws block
(ArMDE:5639-7113). `entity_kinds` is `["character"]` on all 35 ✓. **`tainted` is
`true` on exactly the one entry whose descriptor carries the tag**
(`flaw.impious_friend`, ArMDE:6285 `*Minor, Story, Tainted*`) and `false` on the
other 34 ✓. Two entries invite the mistake and neither gets it:
`flaw.lesser_malediction` ("cursed by some supernatural power" — a curse is not
an Infernal association) and `flaw.horrifying_appearance_snake_legs` (a
"horrific deformity" is an appearance, not a realm). `tainted` means
Infernal-realm associated (ArMDE:2998-3002), and following the descriptor rather
than the prose is correct. **No descriptor line in this span is malformed** —
every one of the 32 carries its comma, unlike B12's ArMDE:5973 and B13's
ArMDE:6069.

**The index confirms every filing, with no contradiction.** All 32 headings are
filed in the book's own "List of Flaws" (ArMDE:5283-5638) under a
`### <category>, <magnitude>` heading matching both the descriptor and the data:

| index lines | heading | entries |
|---|---|---|
| 5323-5325 | `### Personality, Major or Minor` (5310) | Hatred, Higher Purpose, Lecherous |
| 5359 | `### Story, Major` (5340) | Indiscreet |
| 5396-5397 | `### Supernatural, Major` (5387) | Horrifying Appearance – Snake Legs, Hunger for (Form) Magic |
| 5409-5410 | `### General, Major` (5401) | Leprosy, Low Self-Esteem |
| 5439-5444 | `### Hermetic, Minor` (5417) | Hedge Wizard, Incompatible Arts, Inconstant Magic, Infamous Master, Limited Magic Resistance, Loose Magic |
| 5483-5485 | `### Personality, Minor` (5467) | Humble, Imagined Folk Tradition Vulnerability, Lost Love |
| 5510-5512 | `### Story, Minor` (5501) | Heir, Hermetic Patron, Impious Friend |
| 5547-5548 | `### Supernatural, Minor` (5534) | Inscribed Shadow, Lesser Malediction |
| 5589-5598 | `### General, Minor` (5564) | Hobbled, Hunchback, Incomprehensible, Independent Craftsman, Infamous, Jinxed, Judged Unfairly, Lame, Lingering Injury, Low Tolerance |

No entry in this batch is dual-listed, so there is no `taken_as` case here, and
there is no index *gap* of B13's False Power kind. The index earns its keep on
one entry in particular: **`flaw.independent_craftsman` is filed under `###
General, Minor` at ArMDE:5592**, which corroborates the descriptor against the
entry's own trailing sentence telling you to "treat this is a Personality Flaw"
(ArMDE:6304) — see **Q-111**.

**Check 12, mechanically.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the 35 ids in each locale returns
`["name","summary"]` for **24** entries and `["description","name","summary"]`
for **11** — the same 11 in both locales, so the locales never disagree about
*whether* a description exists. The 11 are the batch's 8 `uncomputed_rule`
entries (`hunchback`, `hunger_for_form_magic`, `inconstant_magic`,
`independent_craftsman`, `indiscreet`, `inscribed_shadow`, `jinxed`,
`lingering_injury`), each carrying its full cited passage in both languages ✓,
**plus three `in_play_effect` entries that carry one although their class does
not oblige it** — `flaw.hobbled`, `flaw.lame` and
`flaw.limited_magic_resistance`, each with the entire passage including the
clauses the effects do not implement. That is B13's `flaw.flawed_parma_magica`
observation three times over in one batch, and it is what keeps three otherwise
D5-exposed entries clean. **No `narrative` and no `creation_effect` entry in
this batch carries a description in either locale** — which is where the
remaining D5 findings come from.

**ASCII hyphen (check 12, second half): clean, verified two ways.**
`grep -c "−"` (U+2212) over `rules/i18n/en/virtues_flaws.json` and
`rules/i18n/de/virtues_flaws.json` returns **0** for both, catalogue-wide. And
`grep -o "–[0-9]"` (U+2013 followed by a digit) returns **nothing** in either
file, so no en dash that does ship is used as a *sign*. The conversion works in
both directions in this batch: the DE source writes every negative with an en
dash (DE 6262 `–9`/`–6`, 6274 `–3`, 6278 `–5`, 6300 `–3`, 6304 `–3`, 6332
`–6`/`–3`/`–1`, 6340 `–2`/`–1`, 6352 `–1`/`–3`, 6368 `–3`) and every one that
ships does so as ASCII `-` (visible in the shipped DE descriptions of `hobbled`,
`hunchback`, `hunger_for_form_magic`, `inconstant_magic`,
`independent_craftsman`, `lame` and `lingering_injury`). The **English** source
is again inconsistent — **en dash at ArMDE:6262, 6332, 6352 and 6368; ASCII at
ArMDE:6274, 6278, 6300, 6304 and 6340** — yet the shipped EN descriptions carry
ASCII throughout. *(This sentence is the verification pass's second counting
correction: my first draft put 6278 in the en-dash list, 6352 and 6368 in the
ASCII list, and omitted 6340 from both. The cause is worth recording because
`CLAUDE.local.md` warns about it and I walked into it anyway — the `grep` shim's
ugrep dialect returned an incomplete match set for the en-dash scan, and `rg -n
"–[0-9]"` returned the right one. The shipped-data conclusion was independently
correct either way.)* One en dash *does* ship, in
`flaw.horrifying_appearance_snake_legs`'s name (`Schreckliches Äußeres –
Schlangenbeine` / `Horrifying Appearance – Snake Legs`), where it is the
heading's own punctuation and not a sign — correct, and deliberately not
counted.

**Truncation scan: this batch is clean, and so is the rest of the catalogue bar
B12's one entry.** `jq` over **both** locale files for any `name`, `summary` or
`description` containing `...` or `…` returns exactly **two** rows, and they are
the same id in each: `flaw.exciting_experimentation`, B12's F-415. **No entry in
this batch is truncated**, stated exactly as checked — a literal-ellipsis scan
over the two shipped locale files, which would not see a summary truncated
*without* an ellipsis.

**German names against the canonical tables.** Located by English key,
**nineteen** of the thirty-two distinct English names in this batch have a row
in `rules/source/de/translation-tables/`. **Sixteen agree** with the shipped
`rules/i18n/de/` name and with the DE rulebook heading; **two disagree with the
rulebook** (**F-456**, **F-457**); and one differs only in hyphen placement
inside a placeholder and is not a substantive disagreement. The remaining
**thirteen** English names have no table row and were checked directly against
the DE rulebook heading on the parallel line — **all thirteen match ✓**.

*(This paragraph's first draft had the two figures **swapped** — "thirteen with
a row, nineteen without" — and the verification pass caught it. The error was
visible inside this file: the table below already had seventeen rows, and two
further names with rows, `Judged Unfairly` and `Leprosy`, were quoted from
`reputationen.md:117-118` three paragraphs later while being listed among the
supposedly table-less names. 17 + 2 = 19. The two substantive disagreements are
unaffected.)*

| English | table row | DE rulebook heading | shipped DE name | verdict |
|---|---|---|---|---|
| Hatred | `tugenden-fehler.md:372` *Hass* | 6236 Hass | Hass (Groß/Klein) | ✓ |
| Hedge Wizard | `grundbegriffe.md:62` *Heckenzauberer* | 6240 Heckenzauberer | Heckenzauberer | ✓ |
| Hermetic Patron | `tugenden-fehler.md:687` *Hermetischer Patron* | 6248 Hermetischer Patron | Hermetischer Patron | ✓ (the row's note attributes the Flaw to *HoH:TL*, a provenance claim a terminology table has no business making and which the core book contradicts — noted, not counted) |
| Hobbled | `grundbegriffe.md:325` *Verkrüppelt* | 6260 **Humpelnd** | Humpelnd | data ✓; **table disagrees, and its value is already `flaw.crippled`'s name** — **F-456** |
| Humble | `tugenden-fehler.md:373` *Demütig* | 6268 Demütig | Demütig | ✓ |
| Hunger for (Form) Magic | `tugenden-fehler.md:330` *Hunger nach (Form-)Magie* | 6276 Hunger nach (Form)-Magie | Hunger nach {form}-Magie | ✓ in substance — the hyphen sits inside the parentheses in the table and outside them in book and data; only the data's placement survives interpolation (`Hunger nach Ignem-Magie`, which is exactly what DE 6278's body writes). Not counted. **But the row's note attributes the Flaw to *SdM:M* (RoP:M)**, which the core book contradicts at ArMDE:6276 — the second provenance claim by a terminology table in this batch, after `:687`'s *HoH:TL*, and a D6-rule-1 violation both times. Noted, not counted. |
| Imagined Folk Tradition Vulnerability | `tugenden-fehler.md:344` *Eingebildete Volksmagie-Verwundbarkeit* | 6280 **Eingebildete Volksüberlieferungsanfälligkeit** | Eingebildete Volksüberlieferungsanfälligkeit | data ✓; **table disagrees** — **F-457** |
| Incompatible Arts | `tugenden-fehler.md:298` *Unvereinbare Künste* | 6290 Unvereinbare Künste | Unvereinbare Künste | ✓ |
| Infamous | `tugenden-fehler.md:374`, `reputationen.md:100` *Berüchtigt* | 6310 Berüchtigt | Berüchtigt | ✓ — and `reputationen.md:100` independently records the Reputation as **`Frei wählbar | 4 (–)`**, i.e. a *freely chosen* type, which is the whole of **F-450** |
| Infamous Master | `reputationen.md:101` *Berüchtigter Meister* | 6314 Berüchtigter Meister | Berüchtigter Meister | ✓ — and the row independently records `Hermetisch | 3 (–)`, corroborating `grants_reputation {hermetic, 3}` |
| Lame | `tugenden-fehler.md:436` *Lahm* | 6330 Lahm | Lahm | ✓ |
| Lecherous | `tugenden-fehler.md:375` *Wollüstig* | 6334 Wollüstig | Wollüstig (Groß/Klein) | ✓ |
| Lesser Malediction | `tugenden-fehler.md:345` *Kleine Verfluchung* | 6342 Kleine Verfluchung | Kleine Verfluchung | ✓ |
| Limited Magic Resistance | `tugenden-fehler.md:300` *Begrenzte Magieresistenz* | 6346 Begrenzte Magieresistenz | Begrenzte Magieresistenz | ✓ |
| Lingering Injury | `tugenden-fehler.md:400` *Anhaltende Verletzung* | 6350 Anhaltende Verletzung | Anhaltende Verletzung | ✓ |
| Lost Love | `tugenden-fehler.md:401` *Verlorene Liebe* | 6358 Verlorene Liebe | Verlorene Liebe | ✓ |
| Low Self-Esteem | `tugenden-fehler.md:437` *Geringes Selbstwertgefühl* | 6362 Geringes Selbstwertgefühl | Geringes Selbstwertgefühl | ✓ |
| Judged Unfairly | `reputationen.md:117` *Ungerecht beurteilt* | 6326 Ungerecht beurteilt | Ungerecht beurteilt | ✓ — and the row independently records the prohibition the data drops (**F-447**) |
| Leprosy | `reputationen.md:118` *Lepra* | 6338 Lepra | Lepra | ✓ — and the row independently records the same prohibition (**F-453**) |

The **thirteen** table-less names that match their DE heading: Heir → *Erbe*,
Higher Purpose → *Höheres Ziel*, Horrifying Appearance – Snake Legs →
*Schreckliches Äußeres – Schlangenbeine*, Hunchback → *Bucklig*, Impious Friend
→ *Gottloser Freund*, Incomprehensible → *Unverständlich*, Inconstant Magic →
*Unbeständige Magie*, Independent Craftsman → *Unabhängiger Handwerker*,
Indiscreet → *Indiskret*, Inscribed Shadow → *Eingeschriebener Schatten*, Jinxed
→ *Verhext*, Loose Magic → *Lose Magie*, Low Tolerance → *Geringe Toleranz*.

**Two rows of `reputationen.md` independently record a rule the data drops, and
both are in this span.** `reputationen.md:117` `Judged Unfairly | Ungerecht
beurteilt | Alle | – | Kann keine positive Reputation erwerben` and
`reputationen.md:118` `Leprosy | Lepra | Alle | – | Kann keine positive
Reputation erwerben`. A terminology table is not authoritative for facts about
the rules (D6), so these do not *establish* anything — but they corroborate
**F-447** and **F-453** from a second source, and it is worth recording that a
hand-curated German glossary noticed a prohibition the catalogue data did not.

**One German-source grammatical error ships verbatim.** DE 6356 reads "**Dein**
Fortschrittssumme wird halbiert" — `die Summe` is feminine, so the possessive
must be *Deine*. The same error appears at DE 6296 ("müssen **ihren**
Fortschrittssumme halbieren"), but that sentence is not sentence one so it does
not reach the shipped text; DE 6356 *is* the whole entry, so
`flaw.loose_magic`'s shipped German summary carries it (**F-458**).

**`rules/core/abilities.json` and `rules/core/character_types.json` were
consulted** for the Abilities and profiles these passages name: `ability.faerie_lore`
(category `arcane`), `ability.finesse` (`arcane`), `ability.leadership`,
`ability.craft` and `ability.profession` (all `general`), and
`categories_requiring_virtue` = `["academic","arcane","martial"]`. The first of
those is the whole of F-445. `character_types.json`'s **top-level**
`confidence_score` / `confidence_points` profile fields (`:30-31`, `:100-101`,
`:147-148` — **not** under `budget`, which holds only `virtue_points`,
`flaw_points`, `max_major_*`, `max_minor_flaws` and the category caps; my first
draft put them there and the verification pass corrected it) are **1 / 3** on
companion, magus and mythic_companion and absent on grog, which is what makes
`flaw.low_self_esteem`'s `confidence_bonus { score: -1, points: -3 }` land
exactly on 0/0 rather than on a negative the clamp would have hidden — and which
is also why passing the entry on that arithmetic alone was not enough (**F-459**).

### Part C systemic gaps are not re-reported per entry

Six of Part C's rows touch this batch and none is counted against an entry:

- **`AdvancementMod` is surfaced-only** (A39, C-adjacent) — that is
  `flaw.incomprehensible` and `flaw.loose_magic`. Both carry
  `advancement_mod { amount: 0 }`, and they are the **only two of the fifteen
  `advancement_mod` rows in the catalogue with a zero amount**; the other
  thirteen carry a real ±N. The zero is not an authoring slip:
  `RULES.md:5159` records it deliberately — "`incomprehensible`/`loose_magic`
  halve advancement (amount 0 marker; surfaced)". I had this drafted as a
  finding and **withdrew it on that line**, which is B10's lesson applied. What
  survives is narrower and is counted: `DerivedSurfacedModifiersSection.svelte:27`
  guards the value with `{#if m.amount !== 0}`, so the marker renders as a bare
  source label with no number and no word "halved" — see **Q-113**, and
  **F-454** for the D5 half on `flaw.incomprehensible`, whose summary does not
  carry the halving either. `flaw.loose_magic` is **not** a D5 failure: its
  one-sentence passage *is* its summary, halving included, in both locales.
- **`HealthMod`'s three surfaced-only tracks** (C1) — no entry in this batch
  carries `fatigue_roll`, `casting_fatigue` or `recovery`. The two tracks this
  batch does carry (`fatigue_penalty`, `wound_penalty`, both on
  `flaw.low_tolerance`) are the two that **are** computed, which is precisely
  why F-449 is a miscalculation rather than a lost rule. Part C is therefore
  *not* the reason `flaw.low_tolerance` fails, and saying it was would have
  buried a live wrong number under a known gap.
- **The health-track sign convention (B12's Q-98)** — re-derived here rather
  than inherited, because `flaw.low_tolerance` is a carrier of both computed
  tracks. `fatigue_penalty` has exactly two shipped carriers,
  `flaw.low_tolerance` **-1** and `virtue.enduring_constitution` **+1**; so does
  `wound_penalty`, the same two. `A36`'s stated convention is "positive reduces
  the penalty magnitude", the code implements `(base + delta).min(0)`, and
  ArMDE:6368 says Low Tolerance *increases* the penalties — so a Flaw-negative
  value is right in both sign and magnitude ✓. Nothing in this batch is
  inverted, and the `casting_fatigue` doc-comment dissent Q-98 identified does
  not touch it.
- **`MagicResistanceMod`'s `param` is read at runtime but validated at load by
  nothing** (C2-e) — `flaw.limited_magic_resistance` is one of the two live
  carriers and is named by name in `RULES.md:4919`, `:4987` and `:5138`.
  Re-derived: the entry declares `parameters: [{ key: "form", domain: "form" }]`
  and its effect carries `"param": "form"` ✓, and
  `derived/casting.rs::magic_resistance` drops exactly that Form's bonus
  (`limited_magic_resistance_drops_the_form_bonus_of_its_own_form_only`). Not
  counted against the entry; the gap is the load gate's.
- **`grants_reputation`'s `score` is not enforced** (C2) — touches
  `flaw.hedge_wizard`, `flaw.infamous` and `flaw.infamous_master`. All three are
  correctly authored against an engine that counts kinds and slots; not counted
  against any of them. What **is** counted against one of them is a different
  field entirely — the `kind`, on `flaw.infamous` (**F-450**).
- **A Reputation's *negativity* is not representable at all** — B10's **F-380**,
  and the same three entries are carriers. `types.rs::Reputation`'s `score` is a
  `u8` with no sign and no polarity flag, while all three passages say the
  Reputation is a bad one: ArMDE:6242 "a **negative** Reputation within the
  Order of Hermes at level 3", ArMDE:6312 "a level 4 **bad** Reputation",
  ArMDE:6316 "a **bad** Reputation of the appropriate type at level 3". So all
  three ship as a *positive* Reputation of the right level to the right
  audience. Recorded once, as F-380 already is, and not counted against any of
  the three.
- **`creation_effect` is not required to carry effects** (C7) — all four
  `creation_effect` entries in this batch do carry effects, so this row is inert
  here. Recorded because one of my recommendations (F-445's added
  `ability_authorization`) turns a `narrative` entry into a `creation_effect`
  one, and C7 is why the guard will not notice if the accompanying effect is
  forgotten.

### Decisions applied

`docs/vf-audit/decisions.md` was read in full and is binding. It carried D1-D6
when this batch read it, **including D6's 2026-09-21 correction**, which is
dated the same day as this batch and which I read in its corrected two-rule
form.

- **D3** is the load-bearing one again and governs four entries. On each,
  `narrative` is forbidden and the rule must be written into both locales:
  `flaw.incompatible_arts` (no `Effect` variant disables a Technique×Form pair;
  `DeficientArt` halves an Art's own totals and has no pairwise axis, and
  `MagicTotalHalving`'s four `HalvableTotal` members are whole in-play totals,
  not combinations),
  `flaw.judged_unfairly` (nothing forbids *acquiring* a Reputation — `A25` is a
  grant-and-slot mechanism with no negative form, which is F-410/F-426's
  prohibition gap in a new place),
  `flaw.lesser_malediction` (no effect calibrates a Flaw's severity against a
  magnitude class, and none confers immunity to a named Supernatural Ability —
  the same two inexpressibilities B13 recorded for `flaw.greater_malediction`),
  and `flaw.imagined_folk_tradition_vulnerability`'s **cap** half (no effect
  caps a single Ability's score at a fixed number; `A13`'s
  `locality_ability_cap_fraction` is a fraction of a Characteristic scoped to
  the locality-dependent Abilities and cannot express "Faerie Lore, at most 1").
  Its **permission** half is a different matter and is fully expressible —
  `A14 AbilityAuthorization` exists for exactly it — which is why F-445
  recommends `creation_effect` with the effect added rather than
  `uncomputed_rule`.
- **D5** is the reason three entries that compute correctly still fail:
  `flaw.low_tolerance` (F-449's second half — the -3 resist-pain roll in neither
  locale), `flaw.leprosy` (F-453 — three uncomputed clauses, none anywhere), and
  `flaw.incomprehensible` (F-454 — the entire halving in neither locale). It is
  also the reason three *other* computed entries pass: `flaw.hobbled`,
  `flaw.lame` and `flaw.limited_magic_resistance` each ship the whole passage as
  `description` in both locales although nothing obliges them to.
- **D6** governs **F-456** and **F-457**. Per the brief, `arm-de-translation` is
  outside this repository and the read is denied, so each row is stated as
  *"this repository's copy disagrees with the rulebook"* and the
  error-vs-stale-copy determination is left to the orchestrator. F-456 carries
  an extra piece of evidence that does not depend on that determination: the
  value the table proposes for `Hobbled` (*Verkrüppelt*) is **already**
  `flaw.crippled`'s shipped German name and the DE rulebook's own heading at DE
  5877, so adopting the row would produce exactly the name collision D6's
  corrected text calls "a defect in the rendering". See **Q-114** for the
  precedence question the two rows raise against B13's F-438.
- **D1 and D4** touch nothing here: no entry in this batch carries
  `lab_total_mod`, and none of D1/D4's nine is in the span.
- **D2** touches nothing here: no entry is a granted Virtue and none carries
  `characteristic_score_delta_param` or `characteristic_score_delta`.

### Cross-references followed and rated

Every pointer in the span was followed and read, including on entries that
passed, and including the **four inbound** pointers a reader of the span alone
can never see.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the entry? |
|---|---|---|---|
| `flaw.imagined_folk_tradition_vulnerability` | "this Flaw allows the character to **purchase a score of 1 (but no more) in Faerie Lore** at character creation" (ArMDE:6282) / DE "erlaubt dieser Fehler dem Charakter, bei der Charaktererschaffung einen **Wert von 1 (aber nicht mehr) in Feenkunde** zu kaufen" | `rules/core/abilities.json` — `ability.faerie_lore` has `category: "arcane"`, and `arcane` is one of the three `categories_requiring_virtue` | **YES — this is F-445.** The gate is a permission gate and the entry carries neither `ability_authorization` nor `restricted_ability_xp`, so a companion or grog with this Flaw who buys Faerie Lore 1 raises a hard `ability_category_requires_virtue` error today (`validation/authorization.rs::validate_ability_authorization`, magi exempt). Thirtieth instance of F-409's population. The cap half is separately inexpressible (D3). |
| `flaw.incompatible_arts` | "may not be combined with a **Deficiency** (see page 125)" (ArMDE:6292) / DE "darf aber nicht mit einem **Defizit** kombiniert werden ([Seite 125](#auffällige-gabe))" | `flaw.deficient_form` **ArMDE:5909-5912** and `flaw.deficient_technique` **ArMDE:5913-5915** (B11's span) | **YES — a stated incompatibility, encoded on none of the three (F-446).** `jq` returns an empty `incompatible_with` on all three ids. Worth recording separately: the **German link target is wrong** — DE 6292 points "Seite 125" at `#auffällige-gabe` (Blatant Gift), which is the anchor DE 6384 legitimately uses for its own "Seite 120". A DE-source link defect, reproduced in no data, recorded so a correction pass adding anchors does not copy it. |
| `flaw.incompatible_arts` | "This Flaw **may be taken repeatedly with different combinations**" (ArMDE:6292) | `validation/selections.rs::validate_duplicate_selections`; ArMDE:2814 | **YES, and it is the second half of F-446.** The entry is **unparameterized** and carries no `max_total`, so `max_per_target`'s default of 1 keyed on `(item_ref, {})` blocks the second copy the sentence grants — a live false hard `duplicate_selection`. Contrast `flaw.deficient_form`/`flaw.deficient_technique`, which at least carry the `form`/`technique` parameter that makes two copies two tuples (their own gap is the opposite one, B11's F-400). |
| `flaw.hermetic_patron` | "You must be a **Redcap or magus** to take this Flaw." (ArMDE:6250) / DE "Du musst eine **Rotkappe oder ein Magus** sein, um diesen Fehler zu wählen." | `virtue.redcap` and `virtue.lone_redcap` (both `social_status`); `Prereq::IsMagus` | **YES — a stated prerequisite encoded nowhere (F-448).** The engine expresses this shape already: 19 catalogue entries carry `prerequisites`, `flaw.primogeniture_lineage` uses `All([IsMagus, House(house.verditius)])` and `virtue.magic_items` uses `Has(virtue.redcap)`. Whether `virtue.lone_redcap` also satisfies "a Redcap" is **Q-112**. |
| `flaw.judged_unfairly` | "this Flaw is **incompatible with any Virtue that gives you such a Reputation**" (ArMDE:6328) / DE "dieser Fehler ist **mit jeder Tugend unvereinbar, die dir eine solche Reputation verleiht**" | the 15 Virtues carrying `grants_reputation`: `baccalaureus`, `cathedral_school_master`, `doctor_in_faculty`, `famous`, `hermetic_prestige`, `lone_redcap`, `magister_in_artibus`, `magister_in_medicina`, `master_bard`, `physician_of_salerno`, `rard`, `rosh_beth_din`, `senior_bard`, `senior_clergy`, `templar_office_holder` | **YES — F-447.** `incompatible_with` is absent on `flaw.judged_unfairly` and none of the 15 names it, so Judged Unfairly + Famous (a level-4 *good* Reputation, ArMDE:3863) validates clean today. Fourteen of the fifteen grant a positive Reputation; the exception is `virtue.lone_redcap`, whose own passage and `reputationen.md:111` both make its hermetic 2 a **negative** one, so it is the one Virtue the sentence arguably does not reach. |
| `flaw.leprosy` | *(inbound)* ArMDE:7397, `#### Curse-Throwing*` — "note that curses laid directly by God (such as **leprosy**) are normally represented by permanent Flaws, and thus **exempt**" | `ability.curse_throwing`; the named Flaw is `flaw.leprosy` | **YES — an exemption from a modelled Supernatural Ability, attributed by name. Feeds F-453.** This is the *same sentence* B13's F-442 mined for `flaw.greater_malediction`, and B13 stopped at the Malediction clause; the Leprosy clause is nine words further on in the same line. An inbound pointer, invisible to the entry's own cited range 6338-6341 and to the `uncomputed_clauses.rs` screen, which reads only that range. |
| `flaw.lesser_malediction` | *(inbound)* ArMDE:7397 — "Curse-Throwing cannot affect Flaws; specifically, someone with the **Lesser** or Greater Malediction Flaw is beyond the power of Curse-Throwing, unless it is a Flaw imposed by a faerie or magician with a limited duration." | `ability.curse_throwing`; the named Flaws are this entry and `flaw.greater_malediction` (ArMDE:6210-6213, B13's span) | **YES — F-452's second half.** B13's F-442 named this entry from outside, calling it "B15's span"; it is in fact **B14's** (ArMDE:6342, index 483), so it is mine to rate and is rated here rather than inherited. Re-derived from the source, not from F-442: the sentence at ArMDE:7397 was read in full, and the carve-out ("unless it is a Flaw imposed by a faerie or magician with a limited duration") is part of the rule. |
| `flaw.lesser_malediction` | "The effects of the curse should be **about as bad as other Minor General Flaws**." (ArMDE:6344) / DE "Die Auswirkungen des Fluches sollten ungefähr **so schlimm sein wie andere Kleine Allgemeine Fehler**." | the magnitude taxonomy (`Magnitude::points`, `Ruleset.magnitude_points`) | **YES — F-452's first half.** A calibration instruction aimed at a specific magnitude class is a rule about how severe the chosen curse may be, not colour; no effect expresses it (D3), so it belongs in `description`. Identical in shape to `flaw.greater_malediction`'s own ArMDE:6212, which B13 also moved out of `narrative`. |
| `flaw.infamous` | "You have a **level 4 bad Reputation**, specifying the horrible deeds that earned you such ill will." (ArMDE:6312) | `virtue.famous` **ArMDE:3861-3864** (B03's span) — "You have a good Reputation of level 4. **Choose any reputation you like** (it need not be justified), **and one type**." | **YES, in the direction nobody looks — the comparison invalidates *this* entry's data (F-450).** The two are structural twins: both Minor General, both level 4, both letting the player pick the content. `virtue.famous` ships `grants_reputation { score: 4 }` with **no** `kind`, which A25 defines as the player-chosen wildcard; `flaw.infamous` ships `kind: "local"` although ArMDE:6312 names no audience at all. `validate_reputations` matches kinds exactly, so a player entering a hermetic or ecclesiastical Infamous Reputation gets `reputation_not_granted`. The translation table agrees with the book: `reputationen.md:100` records the Typ as *Frei wählbar*. |
| `flaw.leprosy` | "whenever she undergoes an **Aging Crisis (page 392)** the leper sustains a Heavy Wound in addition to any other result" (ArMDE:6340) | `aging.rs::carries_crisis_heavy_wound`; `RULES.md:7210-7213` | **No new rule — and the target confirms the encoding.** `RULES.md:7210` names this entry explicitly and its guard `leprosy_carries_its_crisis_wound_beside_its_living_conditions_penalty` pins it. `aging_mod { crisis_heavy_wound, 0 }` is right: A38 makes the kind a predicate and ignores `amount`, which ships as 0 by `data_integrity.rs` assertion ✓. |
| `flaw.leprosy` | "a permanent **-2 modifier to her Living Condition**" (ArMDE:6340) | `aging.rs::living_conditions_modifier` → `::aging_total` | **No new rule, and the sign was re-derived rather than assumed.** A38: the aging total **subtracts** the summed Living Conditions Modifier, so a stored **-2** *raises* the total by 2 — worse aging, which is what leprosy should do. Correct ✓, and it is the same polarity `flaw.poor_living_conditions` uses. |
| `flaw.hobbled` | "Any roll that requires moving quickly is penalized by -9 … you roll **double the normal botch dice** in combat situations" (ArMDE:6262) | `RULES.md:5023-5030`; `docs/open-todos.md` row 37 | **No unrecorded rule.** `RULES.md:5023` argues the encoding out in full — Dodge *is* a Defense (ArMDE:16959 gives it a Dfn column and no Atk), so the unscoped -6 already reaches it and no per-weapon scope is needed — and records the doubled botch dice as an open todo rather than dropping it. Both uncomputed clauses also ship in `description` in both locales ✓. Entry passes. |
| `flaw.lame` | "-6 penalty on rolls involving moving quickly or with agility, **-3 on Dodge**, and -1 on other combat scores" (ArMDE:6332) | `derived.rs::in_play_mods` (the scoped-delta fold) → `derived/combat.rs::combat_totals`; `RULES.md:5031-5047` | **No new rule, and the arithmetic was verified at the consumption site rather than taken from A35's prose.** `in_play_mods` runs a pre-pass folding the item's *own* unscoped `CombatMod`s into `unscoped_combat` **before** the main loop, so the scoped row stores `-3 - (-1) = -2` and `cm(Defense, weapon.dodge)` yields `-1 + -2 = -3`, not -4 ✓. Pinned by `weapon_scoped_combat_mod_replaces_general_on_that_weapon_only`. |
| `flaw.inscribed_shadow` | "Only characters with **stigmata** may have this Flaw, which **normally** means that they must be **magi of House Criamon**." (ArMDE:6320) | `house.criamon`; `Prereq::House`, used by `virtue.the_enigma` | **Undecided — escalated as Q-115.** The restriction is expressible and a House sibling already encodes it, but the book hedges it twice ("normally means"), and the actual gate is a thing the engine does not model at all (stigmata). Not counted against the entry; the whole clause ships in `description` in both locales ✓. |
| `flaw.independent_craftsman` | "If you are not using the rules in **City and Guild (page 73)**, treat this is a **Personality Flaw**." (ArMDE:6304) / DE "Wenn du nicht die Regeln in **Stadt und Gilde (Seite 73)** verwendest, behandle dies als **Persönlichkeits-Fehler**." | **Nowhere — *City and Guild* is not in `rules/source/en/`.** The recategorization target is `categories: ["personality"]` and the caps at `character_types.json` → `budget.flaw_category_caps`. | **Undecided — escalated as Q-111.** The pointer cannot be followed, which is exactly what makes the condition *satisfied*: this app uses no City and Guild rules, so the book's own instruction recategorizes the Flaw, which would subject it to the Personality caps (max 1 Major / 2 total). Both independent sources say General — the descriptor at ArMDE:6303 and the index at ArMDE:5592 — so I did not move it. The clause ships in `description` in both locales ✓. |
| `flaw.incomprehensible` | "must halve their Advancement Total (or **Lab Total**, if you are a magus …). If you are a magus teaching spells, halve all applicable Lab Totals, **both yours and the student's**." (ArMDE:6296) | `derived/lab.rs::lab_totals`; `A33 MagicTotalHalving`'s `HalvableTotal` members | **YES — a second mechanic beside the advancement one, and neither reaches the user (F-454).** `HalvableTotal` has four members (`spontaneous_casting`, `lab_enchanting`, `lab_longevity`, `penetration`) and none is a teaching Lab Total, so the clause is inexpressible (D3-shaped) — but D5 obliges it in `description` regardless of that, and the entry carries none in either locale. |
| `flaw.hunger_for_form_magic` | "must consume **1 pawn of vis each season**, corresponding to the Form …" (ArMDE:6278) | the vis rules; `RULES.md:254` | **No unrecorded rule — RULES.md:254 already quotes this sentence**, and the full passage ships in `description` in both locales ✓. What the pointer does surface is a multiplicity question the passage answers by silence: it names exactly one Form and permits no repeat, yet the entry is parameterized with no `max_total` — **F-451**. |
| `flaw.horrifying_appearance_snake_legs` | "your hips give rise to **two or more** snake-like tails" (ArMDE:6266) | `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`; `RULES.md:4834-4842` | **No — and the exemption's written reading is correct.** The brief asks that contradicting an exemption be reported as a finding against the exemption; I checked and it holds. "Two or more" counts *tails*, a feature of the body, not a die, a total or a cap; the rest of the passage ("Your movement is not hindered under most circumstances") states the *absence* of a penalty in prose. There is also no `Crippled`-composition here: "leaving you feigning being crippled as well" is an instruction about concealment, and `grep` over the Flaws block finds no clause making this entry confer another's effects. `narrative` survives ✓. |
| `flaw.leprosy` | *(inbound)* ArMDE:4251, `virtue.leper_magus` (*Major, Hermetic*) — "This Virtue **can only be bought if the character also has the Leprosy Flaw**, and is only available to magi trained in House Tytalus." | `virtue.leper_magus`; `Prereq::Has` / `Prereq::House` | **YES, and the defect is already held.** An inbound prerequisite of exactly F-448's shape, naming this Flaw. B05's **F-136** already records both of that Virtue's missing prerequisites, so nothing new goes on the correction list — but this file's first draft claimed a complete inbound census ("the two inbound pointers") and this one was absent from it. Found by the verification pass; recorded so the census is honest. |
| `flaw.low_tolerance` | *(no textual pointer — a mirror comparison)* "Increase the penalties for reduced Fatigue levels by one point … -3 penalty on rolls for the character to resist pain" (ArMDE:6368) | `virtue.enduring_constitution`, **ArMDE:3751-3754** (B02's span), and `derived/combat.rs::fatigue_levels` / `::wound_ranges` | **YES, in two directions.** (a) ArMDE:3754 carries the floor "**(but not below zero)**" that ArMDE:6368 does not, which is where `.min(0)` comes from and which is why the uniform fold over all five tiers is safe upward and unsafe downward — this is F-449's decisive evidence and it is **outside** the span. (b) ArMDE:3754's "+3 on rolls to resist pain" is already B02's **F-64** ("in neither the data nor either locale"); ArMDE:6368's "-3" is the same clause on the mirror entry and is missing for the same reason, so F-449's D5 half and F-64 are one correction. The target was read in full and rated: its shipped text carries `name` + `summary` only, exactly as F-64 says, and its `health_mod` signs are correct ✓. |
| `flaw.incompatible_arts` | *(inbound)* ArMDE:10163 / :10168, `### Beast Masters` (an Ex Miscellanea tradition) — "They are never taught how to turn into animal form, or how to harm animals, **which results in the Incompatible Arts Flaw (MuCo & PeAn)**" and "*Required Virtues and Flaws:* Animal Ken, Minor Magical Focus; **Incompatible Arts**." | the tradition's required-traits list; nothing in `rules/core/` grants it (`grep` over `rules/core/` outside `virtues_flaws.json` returns no hit for `incompatible_arts`) | **YES — it corroborates F-446's parameterization half from a second, independent site 3,870 lines away.** The book writes the Flaw's value out as a pair of Technique×Form combinations, **`(MuCo & PeAn)`**, exactly as `flaw.deficient_form` / `flaw.deficient_technique` carry a `form` / `technique` parameter. So the parameter F-446 asks for is not an audit invention: the rulebook already records instances of this Flaw in parameterized form. The tradition itself is not modelled (no rules file names the Flaw), so nothing breaks today — but the moment it is, an unparameterized entry cannot express `(MuCo & PeAn)` at all. |
| `flaw.limited_magic_resistance` | *(inbound)* ArMDE:10155, an Inner-Mystery Initiation script — "the Initiate receives the Minor Flaw Hubris if this her first Initiation into an Inner Mystery, or **Limited Magic Resistance with Vim** if it is not (Ordeal +3)" | `art.vim`; the entry's own `form` parameter | **No new rule, and a corroboration.** The Flaw is named with its Form filled in ("with Vim"), which is exactly what the `form` parameter carries, and being handed out repeatedly by Initiations is consistent with ArMDE:6348's explicit repeat permission and with the entry's absent `max_total` ✓. Nothing is attributed to the entry that its own passage does not already say. |
| `flaw.hedge_wizard` | *(inbound)* ArMDE:11598 — "She gains ten experience points in her **Impoverished Hedge Wizard Reputation**." | — | **No.** A worked example of the lab-Aesthetics rules that happens to name a Reputation of that flavour; it is not this Flaw's hermetic level-3 Reputation and states nothing about the Flaw. Followed and discarded. |
| `flaw.hallucinations` → `flaw.visions` | *(inbound, from B13's span)* ArMDE:6228 names the Visions Flaw | — | **Re-checked for completeness and adds nothing to B14**, since neither id is in this span. Recorded only because the line sits three lines above my range's start and was read in the overrun. |

### ArMDE:2814, :2816, :2818, :2820 — instances in this batch

ArMDE:2812-2820 was re-read in full for this batch rather than taken from B10,
B12 or B13, because F-427 exists precisely because an earlier batch carried one
of these rows across without re-deriving it.

- **ArMDE:2814** ("A Virtue or Flaw may be taken more than once only if the
  description explicitly allows it. Most Virtues and Flaws may only be taken
  once.") — **two** explicit permissions in the span, and the data gets one
  right and one wrong.
  `flaw.limited_magic_resistance` ArMDE:6348 ("You may take this Flaw multiple
  times, for multiple Forms") → a `form` parameter with no `max_total`, so
  `max_per_target`'s default of 1 keyed on `(item_ref, whole params map)` makes
  two copies naming different Forms two tuples and legal, two naming the same
  Form one tuple and blocked. Exactly what the sentence says ✓.
  `flaw.incompatible_arts` ArMDE:6292 ("may be taken repeatedly with different
  combinations") → **no parameter and no `max_total`**, so the repeat the book
  grants is refused (**F-446**).
  Against those, **one entry is parameterized where the book grants no repeat at
  all**: `flaw.hunger_for_form_magic` carries a `form` parameter and no
  `max_total`, so a character may hold Hunger for Ignem Magic *and* Hunger for
  Auram Magic — two Major Supernatural Flaws — which ArMDE:2814 forbids by
  silence (**F-451**). That is B10's **F-358** / B11's **F-400** shape, not a new
  one. The catalogue-wide population is stated no wider than checked: `jq`
  returns **42 of the 51** parameterized entries with no `max_total`; I read the
  passages of the three in my span only.
  **The other 32 entries are unparameterized with no `max_total`**, so
  `max_per_target`'s default of 1 keyed on `(item_ref, {})` already forbids a
  second copy, which is what the book requires — with the single exception of
  `flaw.incompatible_arts`, where that default is the bug.
- **ArMDE:2816** ("All characters must take one Social Status, and may only take
  more than one if the descriptions … explicitly note that they are
  compatible") — **zero instances**: no entry in this batch carries the
  `social_status` category. **F-427 was nevertheless re-derived independently
  and stands**: `jq` over `rules/core/character_types.json` returns
  `budget.flaw_category_caps` containing only `personality` (×2) and `story`
  rows on all four profiles, and `budget.virtue_category_caps` only a `hermetic`
  row on `magus`. There is no `social_status` cap of either sign anywhere, and
  `required_traits` is id-based so "one of category X" is inexpressible. Not
  counted against any entry here.
- **ArMDE:2818** ("A character should not have more than one Story Flaw") —
  **four** instances: `flaw.heir`, `flaw.hermetic_patron`, `flaw.impious_friend`,
  `flaw.indiscreet`. Enforced on all four profiles, re-derived: every profile
  carries `{"category":"story","max":1}` (grog `max: 0`) ✓. This is the densest
  Story span the audit has hit — four of the book's Story Flaws in 134 lines.
- **ArMDE:2820** ("A character may not have more than one Major Personality
  Flaw … should normally not have more than two Personality Flaws in total") —
  three Major/Minor pairs here (`hatred`, `higher_purpose`, `lecherous`), each
  with a mutual `incompatible_with` ✓, additionally load-enforced by
  `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`. Both halves
  are live and re-derived: every profile carries
  `{"category":"personality","max":1,"major_only":true,"hard":true}` and
  `{"category":"personality","max":2}` (grog 0 and 1) ✓. Three Minor singles
  join them (`humble`, `imagined_folk_tradition_vulnerability`, `lost_love`).
- **ArMDE:2960-2962** (Supernatural realm association) — **four** instances:
  `flaw.horrifying_appearance_snake_legs`, `flaw.hunger_for_form_magic`,
  `flaw.inscribed_shadow`, `flaw.lesser_malediction`. None carries a
  `realm`-domain parameter. Noted only; it is B11's standing observation, not a
  new one. `flaw.lesser_malediction` is the interesting case — "cursed by some
  **supernatural power**" leaves the realm open by design, which is one more
  reason its severity calibration is the only thing the entry can be pinned to.

## Verdicts

35 rows, one per entry. `class` / `data` / `text` report checks 2, 3-11 and 12.
"OK" means every check in that column passed; `?` means escalated, not resolved.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.hatred_major` | 6236-6239 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.hatred_minor` | 6236-6239 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.hedge_wizard` | 6240-6243 | creation_effect ✓ | `grants_reputation {hermetic, 3}` correct in kind and score ✓ | OK | **clean**; F-380 carrier (negativity), not counted |
| `flaw.heir` | 6244-6247 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.hermetic_patron` | 6248-6255 | narrative → uncomputed_rule / creation_effect | **the stated "Redcap or magus" restriction is encoded by no `prerequisites`** | the restriction is the *summary* in both locales, but nothing enforces it | **F-448** **Q-112**; ArMDE:2818 instance |
| `flaw.higher_purpose_major` | 6256-6259 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.higher_purpose_minor` | 6256-6259 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.hobbled` | 6260-6263 | in_play_effect ✓ | the arithmetic is right, but **ArMDE:16656 defines "five combat scores: Initiative, Attack, Defense, Damage, and Soak" and the data covers two of them** **?** | OK ✓ — full passage both locales, the -9 and the doubled botch dice included, ASCII hyphen ✓ | **Q-116** (overturned from clean by the verification pass); `RULES.md:5023-5030` |
| `flaw.horrifying_appearance_snake_legs` | 6264-6267 | narrative **?** | OK | OK ✓ — the en dash in the name is heading punctuation, not a sign; **but DE 6266 turns a concealment instruction into "als Gelähmter gelten"** | **Q-117**; `NO_RULE_DESPITE_TOKEN` reading verified and found to contradict `RULES.md:4684` |
| `flaw.humble` | 6268-6271 | narrative ✓ | OK | OK | **clean**; ArMDE:2820 instance |
| `flaw.hunchback` | 6272-6275 | uncomputed_rule ✓ | OK | OK ✓ — both -3s in both locales, ASCII ✓ | **clean** |
| `flaw.hunger_for_form_magic` | 6276-6279 | uncomputed_rule ✓ | **no `max_total`, so all ten Forms may be hungered for at once** (ArMDE:2814) | OK ✓ — full passage both locales, `-5` ASCII ✓ | **F-451**; ArMDE:2960 instance |
| `flaw.imagined_folk_tradition_vulnerability` | 6280-6283 | narrative → creation_effect | **no `ability_authorization` for `ability.faerie_lore` (`arcane`): buying the score the Flaw grants is a hard validator error today**; the "but no more" cap is inexpressible (D3) | the permission and the cap in neither locale | **F-445**; F-457 (table row); ArMDE:2820 instance |
| `flaw.impious_friend` | 6284-6289 | narrative ✓ | OK — `tainted: true` ✓ | OK | **clean**; ArMDE:2818 instance |
| `flaw.incompatible_arts` | 6290-6293 | narrative → uncomputed_rule | **unparameterized with no `max_total`, so the repeat ArMDE:6292 grants is a hard `duplicate_selection`**; **the stated Deficiency incompatibility is encoded on none of the three entries** | the Arts prohibition, the repeatability and the Deficiency exclusion in neither locale | **F-446** |
| `flaw.incomprehensible` | 6294-6297 | in_play_effect ✓ | `advancement_mod {teaching, 0}` is a deliberate marker (`RULES.md:5159`) ✓ — but the second clause, halving *both* Lab Totals, is encoded nowhere | **the entire halving reaches the user in neither locale**, and the marker renders as a bare label | **F-454** **Q-113** |
| `flaw.inconstant_magic` | 6298-6301 | uncomputed_rule ✓ | OK | OK ✓ — the -3 Finesse in both locales, ASCII ✓ | **clean** |
| `flaw.independent_craftsman` | 6302-6305 | uncomputed_rule ✓ | `categories: ["general"]` matches descriptor **and** index ✓ — but ArMDE:6304 recategorizes it when City and Guild is unused, which is this app's state **?** | OK ✓ — full passage both locales including the recategorization clause | **Q-111** |
| `flaw.indiscreet` | 6306-6309 | uncomputed_rule ✓ | OK | OK ✓ — the Ease Factor 9 and the botch clause in both locales | **clean**; ArMDE:2818 instance |
| `flaw.infamous` | 6310-**6312** | creation_effect ✓ | **`grants_reputation {kind: "local"}` where ArMDE:6312 names no audience; its twin `virtue.famous` correctly ships the wildcard**; **`source.lines` stops one line short of the next heading** | the summary is authored rather than the passage's first sentence, and states the rule — acceptable, noted | **F-450** **F-455** |
| `flaw.infamous_master` | 6314-6317 | creation_effect ✓ | `grants_reputation {hermetic, 3}` correct ✓, corroborated by `reputationen.md:101` | OK | **clean**; F-380 carrier, not counted |
| `flaw.inscribed_shadow` | 6318-6321 | uncomputed_rule ✓ | the Criamon/stigmata restriction is expressible (`Prereq::House`) but hedged twice in the source **?** | OK ✓ — full passage both locales, `+3` ASCII ✓ | **Q-115**; ArMDE:2960 instance |
| `flaw.jinxed` | 6322-6325 | uncomputed_rule **?** | OK | OK ✓ — full passage both locales | **Q-117** — the "states the *absence* of a rule" reading `RULES.md:4684` gives this entry is the opposite of the one `NO_RULE_DESPITE_TOKEN` gives `flaw.horrifying_appearance_snake_legs`; the one `uncomputed_rule` entry with no `anchor` |
| `flaw.judged_unfairly` | 6326-6329 | narrative → uncomputed_rule | **the stated incompatibility with any Reputation-granting Virtue is encoded on none of the 15**; nothing forbids acquiring a Reputation (D3) | the prohibition and the incompatibility in neither locale | **F-447** |
| `flaw.lame` | 6330-6333 | in_play_effect ✓ | the delta fold yields -3 on Dodge, not -4, verified at the consumption site ✓ — but **"-1 on other combat scores" against ArMDE:16656's five-score definition reaches Initiative, Damage and Soak, and the data covers Attack and Defense** **?** | OK ✓ — full passage both locales, ASCII ✓ | **Q-116** (overturned from clean by the verification pass); `RULES.md:5031-5047` |
| `flaw.lecherous_major` | 6334-6337 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.lecherous_minor` | 6334-6337 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean**; ArMDE:2820 instance |
| `flaw.leprosy` | 6338-6341 | in_play_effect ✓ | `aging_mod {living_conditions,-2}` correct in sign ✓ (the total subtracts it, so -2 raises the total) and `{crisis_heavy_wound,0}` correct as a predicate ✓ | **the leper-colony -1, the positive-Reputation prohibition and the ArMDE:7397 Curse-Throwing exemption in neither locale** | **F-453** |
| `flaw.lesser_malediction` | 6342-6345 | narrative → uncomputed_rule | OK — neither the magnitude calibration nor an immunity is expressible (D3) | the calibration and the ArMDE:7397 immunity in neither locale | **F-452**; ArMDE:2960 instance |
| `flaw.limited_magic_resistance` | 6346-6349 | in_play_effect ✓ | `magic_resistance_mod {no_form_bonus, param: form}` correct ✓; repeatability correct ✓ | OK ✓ — carries a description although its class does not oblige one | **clean**; C2-e carrier, not counted |
| `flaw.lingering_injury` | 6350-6353 | uncomputed_rule ✓ | OK | OK ✓ — the -1, the -3 and the Decrepitude multiplier in both locales, ASCII ✓ | **clean**; `RULES.md:4745` |
| `flaw.loose_magic` | 6354-6357 | in_play_effect ✓ | `advancement_mod {spell_mastery, 0}` is a deliberate marker (`RULES.md:5159`) ✓ | the rule is the summary in both locales ✓ — but **the DE summary reads "Dein Fortschrittssumme", wrong gender** | **F-458** **Q-113** |
| `flaw.lost_love` | 6358-6361 | narrative ✓ | OK | OK | **clean** |
| `flaw.low_self_esteem` | 6362-6365 | creation_effect ✓ | `confidence_bonus {score:-1, points:-3}` lands on 0/0 against the 1/3 profile bases ✓ — **but the fold is additive and `incompatible_with` is empty on all three `confidence_bonus` carriers, so Low Self-Esteem + Self-Confident ships Confidence 1 / 2 points, against ArMDE:6364's "never"** | the "never" absolute in neither locale | **F-459** (overturned from clean by the verification pass) |
| `flaw.low_tolerance` | 6366-6369 | in_play_effect ✓ | **the -1 is folded over all five Fatigue tiers, so Fresh and Winded — which ArMDE:17129 gives no penalty — are shown at -1**; the `wound_penalty` half is correct ✓ | the -3 resist-pain roll in neither locale | **F-449** |

## Findings

### F-445 — `flaw.imagined_folk_tradition_vulnerability` is `narrative` on a permission the app hard-errors on, and on a cap it expresses nowhere

> "Although this flaw closely resembles a Delusion, Imagined Folk Tradition
> Vulnerability **has other game mechanical effects as well**. … On the plus
> side, this Flaw **allows the character to purchase a score of 1 (but no more)
> in Faerie Lore at character creation**. This is because he needs a rudimentary
> knowledge of the fay and the wards against them in order to be able to act this
> way."
> — ArMDE:6282

> "Obwohl dieser Fehler einer Wahnvorstellung ähnelt, hat die Eingebildete
> Volksüberlieferungsanfälligkeit auch **andere spielmechanische Auswirkungen**.
> … Auf der positiven Seite **erlaubt dieser Fehler dem Charakter, bei der
> Charaktererschaffung einen Wert von 1 (aber nicht mehr) in Feenkunde zu
> kaufen**."
> — DE 6282

**Current data.** `classification: "narrative"`, no `effects`, no
`parameters`, no `prerequisites`. `rules/i18n/en|de/virtues_flaws.json` carry
`name` + `summary` only; the summary stops at the first sentence, so the only
thing that reaches the user is *"has other game mechanical effects as well"* —
an announcement of mechanics with the mechanics removed.

**Why it is wrong, in two independent ways.**

1. **The permission is a live false hard error.**
   `rules/core/abilities.json` gives `ability.faerie_lore` the category
   `arcane`, and `categories_requiring_virtue` is `["academic","arcane","martial"]`.
   `validation/authorization.rs::validate_ability_authorization` errors with
   `ability_category_requires_virtue` on any held Ability in a gated category
   unless the id or the category appears in the union of
   `AbilityAuthorization` and `RestrictedAbilityXp` lists — with a whole-character
   exemption only for a profile whose `is_magus` is true. This Flaw is *Minor,
   Personality*, so grogs, companions and mythic companions may all take it, and
   none of them is exempt. A player who takes the Flaw and does exactly what its
   sentence permits — buy Faerie Lore 1 — is told his character is illegal.
   This is F-409's mechanism, thirtieth stated instance against **2** encoded
   (`flaw.covenant_upbringing`, `virtue.student_of_realm`) plus 10 implicit via
   `restricted_ability_xp` on a gated category.
2. **The cap is a stated rule that reaches the user nowhere.** "(but no more)"
   is a hard ceiling of 1 on one named Ability. No `Effect` variant expresses
   it: `A9 AbilityScoreGrant` grants a score rather than capping one, and `A13
   LocalityAbilityCapFraction` is a fraction of a Characteristic scoped to the
   `locality_dependent` Abilities. That inexpressibility is grounds for the rule
   being written into `description` (D3/D5), never for `narrative`.

**Correct value.** `classification: "creation_effect"`, with
`effects: [{ "type": "ability_authorization", "abilities": ["ability.faerie_lore"] }]`
— the `abilities` form rather than `categories`, because the passage permits one
named Ability and not the whole `arcane` category. The cap sentence must be
written into `description` in **both** locales. Note C7: `creation_effect` is
not required to carry effects, so nothing will fail if the effect is forgotten
when the class is changed.

**The exact wording the phrase screen missed**, for the screen-fix list:

- EN: `allows the character to purchase a score of 1 (but no more) in Faerie Lore at character creation`
- DE: `erlaubt dieser Fehler dem Charakter, bei der Charaktererschaffung einen Wert von 1 (aber nicht mehr) in Feenkunde zu kaufen`

`states_a_mechanical_rule` is false here: `has_signed_number` sees no sign on
"1", there is no botch term, and `MECHANICAL_PHRASES` carries `"no more than"` /
`"nicht mehr als"` but **not** the bare `"no more"` / `"nicht mehr"` this
passage uses. The entry sits inside `SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` block,
swept 2026-09-15 and re-swept 2026-09-19, green both times. Two idiom families
would have caught it and neither is in the list: the **permission** idiom B12
named ("allows the character to purchase", "erlaubt … zu kaufen"), and a
**self-declaration** idiom this entry hands over free — `game mechanical
effects` / `spielmechanische Auswirkungen`, a phrase that is by construction
never flavour.

**Severity: HIGH.** A character the book describes and the app refuses to
build, plus a lost rule. Per `CLAUDE.md` this is product-integrity, not
cosmetic.

### F-446 — `flaw.incompatible_arts` is `narrative`, forbids the repeat the book grants, and drops a stated Deficiency exclusion

> "For some reason **you are completely unable to use two combinations of
> Techniques and Forms**. For example, you may be unable to use Intellego Herbam
> and Intellego Animal. **You may not use these Arts together even if one or
> both are requisites.** This Flaw **may be taken repeatedly with different
> combinations**, but **may not be combined with a Deficiency** (see page 125)."
> — ArMDE:6292

> "Aus irgendeinem Grund **bist du völlig außerstande, zwei Kombinationen von
> Techniken und Formen zu verwenden**. … **Du kannst diese Künste nicht zusammen
> einsetzen, selbst wenn eine oder beide als Requisite benötigt werden.** Dieser
> Fehler **kann mehrfach mit unterschiedlichen Kombinationen gewählt werden**,
> **darf aber nicht mit einem Defizit kombiniert werden**."
> — DE 6292

**Current data.** `classification: "narrative"`, no `effects`, no
`parameters`, no `max_total`, `incompatible_with` absent. The EN and DE
`summary` stop after the first sentence; there is no `description` in either
locale. One passage, four clauses, none of them anywhere in the data.

**Why it is wrong, in three ways.**

1. **`narrative` on an absolute prohibition.** The passage forbids a pair of
   Art combinations outright, including as requisites — the sharpest kind of
   mechanical statement there is. No `Effect` variant expresses it:
   `A32 DeficientArt` halves totals for one Art and has no pairwise axis, and
   `A33 MagicTotalHalving`'s four `HalvableTotal` members
   (`spontaneous_casting`, `lab_enchanting`, `lab_longevity`, `penetration`) are
   whole in-play totals, not Technique×Form pairs. Per **D3** that is grounds for
   `uncomputed_rule` with the rule written out, never for `narrative`.
2. **The repeat the book explicitly grants is a hard error today.** The entry is
   **unparameterized**, so `max_per_target`'s default of **1** is keyed on
   `(item_ref, {})` — the empty params map — and a second copy is the same tuple.
   `validation/selections.rs::validate_duplicate_selections` raises
   `duplicate_selection` (an **error**, per B10 §`max_per_target`). A player who
   does what ArMDE:6292 says and takes the Flaw twice for two different
   combinations is refused. This is the *inverse* of B11's F-400: there a
   parameterized entry lacked the `max_total` that would have limited it; here an
   unparameterized entry lacks the parameter that would have freed it.
3. **A stated pair rule encoded on neither side.** `jq` returns no
   `incompatible_with` key on `flaw.incompatible_arts`, on `flaw.deficient_form`
   (ArMDE:5909-5912) or on `flaw.deficient_technique` (ArMDE:5913-5915). A magus
   may hold Incompatible Arts and Deficient Technique together with no issue
   raised. That is the seventh stated pair rule the audit has found across
   B10/B11/B13 and this batch, against one encoded.

**Correct value.** `classification: "uncomputed_rule"`, with the full passage in
`description` in both locales; a mutual `incompatible_with` between
`flaw.incompatible_arts` and both Deficiency Flaws; and enough `parameters` that
two copies naming different combinations are two tuples. `max_total` stays at
its 255 sentinel, because the book states no ceiling on how many combinations
may be taken.

**How many parameters — corrected by the verification pass, and this matters.**
My first draft said "two `parameters` of domain `technique` and `form`". That
models **one** Technique×Form pair, and the passage says **two**: "you are
completely unable to use **two combinations** of Techniques and Forms" / DE
"**zwei Kombinationen** von Techniken und Formen", with ArMDE:6292's own worked
example naming both ("Intellego Herbam **and** Intellego Animal"). The book's
other instance is written the same way — ArMDE:10163's "the Incompatible Arts
Flaw **(MuCo & PeAn)**". So **one selection carries four refs** (two
technique/form pairs), not two. A correction pass taking my first wording would
have shipped a Flaw that forbids half of what the book forbids, and would then
have needed two *copies* to express one instance — which is exactly the
repeatability axis clause 2 is about, so the two mistakes would have masked each
other.

**The exact wording the phrase screen missed:**

- EN: `you are completely unable to use two combinations of Techniques and Forms` / `You may not use these Arts together even if one or both are requisites` / `may be taken repeatedly with different combinations, but may not be combined with a Deficiency`
- DE: `bist du völlig außerstande, zwei Kombinationen von Techniken und Formen zu verwenden` / `Du kannst diese Künste nicht zusammen einsetzen, selbst wenn eine oder beide als Requisite benötigt werden` / `kann mehrfach mit unterschiedlichen Kombinationen gewählt werden, darf aber nicht mit einem Defizit kombiniert werden`

No signed number, no botch term, and no `MECHANICAL_PHRASES` hit. This is the
**prohibition** idiom (F-410/F-426) plus an **incompatibility** idiom the screen
also has no phrase for — `may not be combined with` / `darf … nicht … kombiniert
werden`, which would be a cheap and precise addition.

**Independent corroboration for the parameter, from 3,870 lines away.** The
inbound sweep turned up ArMDE:10163, in the Ex Miscellanea traditions section:
"They are never taught how to turn into animal form, or how to harm animals,
**which results in the Incompatible Arts Flaw (MuCo & PeAn)**", with
ArMDE:10168 repeating it in the tradition's `*Required Virtues and Flaws:*`
line. The rulebook therefore writes an instance of this Flaw out **with its
value filled in** — a pair of Technique×Form combinations — which is precisely
the shape a parameter would carry and which an unparameterized entry cannot
express at all. `grep` over `rules/core/` outside `virtues_flaws.json` returns
no reference to `flaw.incompatible_arts`, so the Beast Masters tradition is not
modelled and nothing is broken by this today; it is evidence that the
parameterization is the book's own reading rather than an audit invention.

**One source defect recorded in passing, against no data.** DE 6292 renders the
English "(see page 125)" as `[Seite 125](#auffällige-gabe)` — a link to *Blatant
Gift*, which is the anchor DE 6384 legitimately uses for its own "Seite 120".
Page 125 is the Deficiency block. Reproduced nowhere in the data; noted so a
correction pass adding anchors reads the heading rather than the link.

**Severity: HIGH.** A character the book permits and the app refuses to build
(clause 2), plus a whole passage of lost rules.

### F-447 — `flaw.judged_unfairly` is `narrative` on a prohibition and on a stated incompatibility with fifteen Virtues

> "**You cannot gain a positive Reputation in any community**, and this Flaw is
> **incompatible with any Virtue that gives you such a Reputation**."
> — ArMDE:6328

> "**Du kannst in keiner Gemeinschaft eine positive Reputation erwerben**, und
> dieser Fehler ist **mit jeder Tugend unvereinbar, die dir eine solche
> Reputation verleiht**."
> — DE 6328

**Current data.** `classification: "narrative"`, no `effects`,
`incompatible_with` absent, no `description` in either locale. The shipped
summary is the passage's first sentence — "Somehow you come across the wrong way
to people, and they universally distrust and underestimate you" — so the whole
of ArMDE:6328's final sentence reaches the user nowhere.

**Why it is wrong.** The sentence states two rules and the data carries
neither.

- The **prohibition** ("cannot gain a positive Reputation in any community") is
  mechanical by any reading: Reputations are a modelled entity field
  (`types.rs::Reputation`, validated by `validation/scores.rs::validate_reputations`).
  No `Effect` variant forbids acquiring one — `A25 GrantsReputation` is a
  grant-and-slot mechanism with no negative form and no polarity axis at all
  (which is separately B10's F-380) — so per **D3** the entry is
  `uncomputed_rule` with the rule written out, not `narrative`.
- The **incompatibility** is not a general principle but a named, enumerable
  set. `jq` over `rules/core/virtues_flaws.json` returns **15 Virtues carrying
  `grants_reputation`**: `virtue.baccalaureus` (academic 1),
  `virtue.cathedral_school_master` (academic 2), `virtue.doctor_in_faculty`
  (academic 3), `virtue.famous` (wildcard 4), `virtue.hermetic_prestige`
  (hermetic 4), `virtue.lone_redcap` (hermetic 2),
  `virtue.magister_in_artibus` (academic 2), `virtue.magister_in_medicina`
  (academic 3), `virtue.master_bard` (local 3),
  `virtue.physician_of_salerno` (local 2), `virtue.rard` (local 1),
  `virtue.rosh_beth_din` (local 2), `virtue.senior_bard` (local 2),
  `virtue.senior_clergy` (ecclesiastical 4 **and** local 4),
  `virtue.templar_office_holder` (local 2). `flaw.judged_unfairly` names none of
  them, and none of them names it. A companion with Judged Unfairly and Famous —
  a level-4 *good* Reputation of any type he likes (ArMDE:3863) — validates
  clean today, which is precisely the combination the sentence forbids.

**Correct value.** `classification: "uncomputed_rule"` with the full sentence in
`description` in both locales, **plus** a mutual `incompatible_with` against the
Reputation-granting Virtues. Fourteen of the fifteen grant a positive
Reputation; `virtue.lone_redcap` is the one exception — its hermetic 2 is
negative by its own passage and by `reputationen.md:111` (`2 (–)`) — so the
correction list should record it as the one id to leave out, rather than letting
a later pass rediscover the question.

**Corroboration from a second source.** `rules/source/de/translation-tables/reputationen.md:117`
carries the row `Judged Unfairly | Ungerecht beurteilt | Alle | – | Kann keine
positive Reputation erwerben`. Per D6 a terminology table is not authoritative
for facts about the rules, so this establishes nothing on its own — but a
hand-curated glossary recorded the prohibition that the catalogue data dropped,
which is worth the line.

**The exact wording the phrase screen missed:**

- EN: `You cannot gain a positive Reputation in any community, and this Flaw is incompatible with any Virtue that gives you such a Reputation`
- DE: `Du kannst in keiner Gemeinschaft eine positive Reputation erwerben, und dieser Fehler ist mit jeder Tugend unvereinbar, die dir eine solche Reputation verleiht`

The prohibition idiom again, and the `MECHANICAL_PHRASES` absolute `"cannot
die"` shows why a *phrase* list cannot generalise: it carries one specific
"cannot", and the books write dozens of others. The German half additionally
demonstrates the discontinuous-negation limit the module's own doc comment
records — "kannst in keiner Gemeinschaft … erwerben" is invisible to any
contiguous substring.

**Severity: MEDIUM-HIGH.** A lost rule *and* an unenforced exclusion that lets a
legal-looking character hold two mutually exclusive traits.

### F-448 — `flaw.hermetic_patron`'s stated "Redcap or magus" restriction is enforced by nothing

> "**You must be a Redcap or magus to take this Flaw.**"
> — ArMDE:6250

> "**Du musst eine Rotkappe oder ein Magus sein, um diesen Fehler zu wählen.**"
> — DE 6250

**Current data.** `classification: "narrative"`, no `prerequisites`, no
`effects`. `categories: ["story"]`, which the profiles permit to companion,
magus and mythic_companion alike (grogs carry `{"category":"story","max":0}`).
The restriction *is* the shipped `summary` in both locales — it is the passage's
first sentence — so the player is told the rule and the app then ignores it.

**Why it is wrong.** The engine expresses this shape and already uses it. The
`Prereq` enum carries `Any`, `Has` and `IsMagus`; 19 catalogue entries carry
`prerequisites`; and two of them are this exact idiom:
`flaw.primogeniture_lineage` encodes "House Verditius only" as
`All([IsMagus, House(house.verditius)])` — a reading `RULES.md:4837-4839`
explicitly calls "already *computed* by the entry's `prerequisites`" — and
`virtue.magic_items` encodes a Redcap restriction as `Has(virtue.redcap)`. So
this is not an inexpressibility (not a D3 case) and not a screen blind spot in
the usual sense: it is a rule with a ready-made slot, left empty. A mythic
companion with no Redcap Virtue may take Hermetic Patron today and the
validator says nothing.

**Correct value.**
`prerequisites: { "kind": "any", "value": [ { "kind": "has", "value": "virtue.redcap" }, { "kind": "has", "value": "virtue.lone_redcap" }, { "kind": "is_magus" } ] }`,
and `classification: "creation_effect"` once it is enforced (per the README's
definition — the book states a rule the engine computes at character creation;
`prerequisites` is computed at creation by `validation/prereq.rs`). If the
correction pass declines to add the prereq, the entry must be `uncomputed_rule`
with the sentence in `description` in both locales; `narrative` is wrong either
way, because the book plainly states a rule. Whether `virtue.lone_redcap`
belongs in the disjunction is **Q-112**.

**The exact wording the phrase screen missed:**

- EN: `You must be a Redcap or magus to take this Flaw.`
- DE: `Du musst eine Rotkappe oder ein Magus sein, um diesen Fehler zu wählen.`

A third idiom family the screen has no phrase for, beside B12's permission and
prohibition idioms: the **eligibility** idiom, `You must be … to take this` /
`Du musst … sein, um diesen Fehler zu wählen`. It is short, formulaic and
almost certainly repeats across the catalogue, which makes it a cheap addition
with a high yield.

**Severity: MEDIUM.** An unenforced eligibility rule. It produces an illegal
character rather than a wrong number, and the summary does tell the player — so
it is a validation gap rather than a lost rule.

### F-449 — `flaw.low_tolerance` puts a -1 penalty on two Fatigue levels the rules give no penalty at all

> "The character cannot easily withstand pain and fatigue. **Increase the
> penalties for reduced Fatigue levels by one point**, and increase the total
> penalty from wounds the character has received by one point. You also suffer a
> **-3 penalty on rolls for the character to resist pain**."
> — ArMDE:6368

> "Der Charakter kann Schmerz und Erschöpfung nicht leicht ertragen. **Erhöhe die
> Malus-Werte für reduzierte Erschöpfungsstufen um einen Punkt**, und erhöhe den
> Gesamtmalus aus erhaltenen Wunden um einen Punkt. Du erleidest außerdem **-3
> auf Würfe des Charakters, um Schmerzen zu widerstehen**."
> — DE 6368

And the rule the engine contradicts:

> "**Each Fatigue level above Winded has a penalty associated with it** (except
> for Unconscious, which is its own penalty). … **The penalty for Weary is -1,
> for Tired -3, and for Dazed -5.**"
> — ArMDE:17129

**Current data.**
`effects: [{ "type": "health_mod", "track": "wound_penalty", "amount": -1 }, { "type": "health_mod", "track": "fatigue_penalty", "amount": -1 }]`,
`classification: "in_play_effect"`, no `description` in either locale.

**Why it is wrong.** The *data* is right and the *consumer* over-applies it.
`derived/combat.rs::fatigue_levels` builds the tier list as a fixed array and
adds the delta to every row uniformly:

```
[ (Fresh, 0), (Winded, 0), (Weary, -1), (Tired, -3), (Dazed, -5) ]
    .map(|(level, base)| FatigueLevel { level, penalty: (base + delta).min(0) })
```

With `delta = -1` that yields **Fresh -1, Winded -1, Weary -2, Tired -4, Dazed
-6**. The last three are exactly what ArMDE:6368 asks for. The first two are
invented: ArMDE:17129 states that only levels *above* Winded carry a penalty at
all, and Fresh is not a reduced Fatigue level in any reading — it is the state of
having lost none. `DerivedVitalsSection.svelte:40` prints every tier with its
penalty (`{store.t('derived-fatigue-' + f.level)}: {f.penalty}`), so a character
sheet for anyone holding this Minor Flaw literally reads **"Fresh: -1"** — a
standing -1 on every roll, forever, that the rulebook never grants.

**Why the `.min(0)` clamp hides it from the other carrier — and where the clamp
comes from.** `fatigue_penalty` has exactly two shipped carriers: this Flaw at
**-1** and `virtue.enduring_constitution` at **+1**. In the positive direction
the clamp does the work — `(0 + 1).min(0) = 0` — so Fresh and Winded stay at 0
and the Virtue looks correct. The defect is reachable only through a negative
delta, i.e. only through this entry, which is why no test noticed.

The clamp's provenance makes the point sharper. Enduring Constitution is Low
Tolerance's exact mirror and is the passage the clamp is taken from:

> "You can withstand pain and fatigue. **Decrease the penalties for reduced
> Fatigue levels by one point**, and reduce your total penalty from wounds by
> onespoint **(but not below zero)**. You also get +3 on rolls to resist pain."
> — ArMDE:3754 (`virtue.enduring_constitution`, *Minor, General*; the
> "onespoint" is a typo in the English source, correct in DE 3754)

The parenthesis "(but not below zero)" appears **only in the Virtue**. The
Flaw's own sentence (ArMDE:6368) contains no floor and no ceiling — because a
Flaw that worsens penalties needs neither. So the engine's uniform fold, which
the clamp exists to make safe in the *upward* direction, has nothing holding it
in the downward one, and the book gives it no authority to touch a row whose
printed penalty is 0.

**The `wound_penalty` half is correct** and was checked separately:
`derived/combat.rs::wound_ranges` applies the delta to Light (-1→-2), Medium
(-3→-4) and Heavy (-5→-6), while Incapacitating and Dead carry `penalty: None`
and are untouched. There is no zero-penalty wound band — `WoundBand::Light`
starts at `min: 1` — so the same over-application has nothing to bite on. ✓

**Correct value.** No data change. The fix is in
`derived/combat.rs::fatigue_levels`: the delta must apply only to tiers that
carry a base penalty, i.e. skip the rows whose printed penalty is 0
(`if base == 0 { 0 } else { (base + delta).min(0) }`), citing ArMDE:17129. The
sign convention and the magnitude are right as shipped and must not be touched.

**Second half, D5.** The passage's third sentence — the -3 on rolls to resist
pain — is computed by nothing and appears in neither locale. The entry carries
no `description` at all, so the only thing the player sees is the summary "The
character cannot easily withstand pain and fatigue", which states no number.
Both computed clauses do reach the sheet as numbers; this third one reaches the
player nowhere. Per D5 it must be written into `description` in both locales
whatever the classification.

**And it is the exact mirror of a finding the audit already holds.** B02's
**F-64** is "`virtue.enduring_constitution` — the +3 pain-resistance bonus is in
neither the data nor either locale", against ArMDE:3754's "+3 on rolls to resist
pain". ArMDE:6368's "-3 penalty on rolls for the character to resist pain" is
the same clause on the opposite entry, and it is missing for the same reason.
The two should be corrected together; fixing one and not the other would leave
the pain-resistance rule half-documented in the UI, which is worse than either
state alone.

**Widened by the verification pass: the yardstick itself states the wrong
behaviour.** Two places describe this code and both describe it as it is *not*.
`derived/combat.rs:354`'s own comment says "Fresh/Winded are penalty-free", and
— more seriously, because it is the document every batch of this audit measures
against — `docs/vf-audit/engine-semantics.md:1455` says:

> "`fatigue_penalty`: each tier's printed penalty becomes `(base + delta).min(0)`
> — Weary −1, Tired −3, Dazed −5; Fresh/Winded are 0. **Clamped at 0**: a Virtue
> can never turn a penalty into a bonus."

"Fresh/Winded are 0" is true only for `delta ≥ 0`. So Part A asserts the correct
behaviour as a fact about code that does not implement it, which is why reading
A36 and the data together — which is what the audit's check 10 asks for — could
never have found this: **the yardstick and the data agreed, and the code
disagreed with both.** The correction must touch `engine-semantics.md:1455` and
`derived/combat.rs:354` as well as the function, or the next reader will
"correct" the fix back. I did not report this; the verification pass did, and it
is the sharper half of the finding.

**Severity: HIGH — the batch's top finding.** Per `CLAUDE.md`, "wrong rules
output" is the top severity class: the app's whole purpose is computing correct
Ars Magica characters, and this prints a penalty on a line the rules leave
blank. The D5 half is MEDIUM on its own.

### F-450 — `flaw.infamous` hardcodes a Reputation audience the book leaves to the player, and its twin Virtue does not

> "People know you well and curse you in their prayers. **You have a level 4 bad
> Reputation, specifying the horrible deeds that earned you such ill will.**"
> — ArMDE:6312

> "Die Leute kennen dich gut und verfluchen dich in ihren Gebeten. **Du hast eine
> schlechte Reputation auf Stufe 4, die die schrecklichen Taten benennt, die dir
> solchen Unmut eingetragen haben.**"
> — DE 6312

The structural twin, thirty pages earlier:

> "**You have a good Reputation of level 4. Choose any reputation you like** (it
> need not be justified), **and one type.**"
> — ArMDE:3863 (`virtue.famous`, ArMDE:3861-3864, *Minor, General*)

**Current data.**
`flaw.infamous`: `effects: [{ "type": "grants_reputation", "kind": "local", "score": 4 }]`.
`virtue.famous`: `effects: [{ "type": "grants_reputation", "score": 4 }]` — **no
`kind`**.

**Why it is wrong.** `A25`: `kind` is `Option<ReputationType>` and `None` means
"player-chosen type (a wildcard, e.g. Famous)". `validation/scores.rs::validate_reputations`
builds `remaining: kind → count` from the concrete-kind grants plus a separate
wildcard counter; an `Entity::reputations` row consumes a matching concrete slot
first and falls back to a wildcard, and a row matching neither raises
`reputation_not_granted` — an **error**. So `kind: "local"` is not a default or
a hint: it is an exact-match gate. A player who takes Infamous and records his
level-4 ill repute among the clergy, or within the Order, or in the schools is
told his character is illegal.

ArMDE:6312 names no audience whatsoever. Infamous and Famous are the same entry
mirrored — same magnitude, same category (`*Minor, General*`, indexed at
ArMDE:5593 and under `### General, Minor` respectively), same level 4, both
leaving the *content* to the player — and Famous ships the wildcard correctly
while Infamous does not. A third source agrees with the book:
`rules/source/de/translation-tables/reputationen.md:100` records Infamous's Typ
as **`Frei wählbar`** (freely choosable) and its level as `4 (–)`, against
`:101`'s `Infamous Master | … | Hermetisch | 3 (–)` where the book *does* name
the audience ("among magi", ArMDE:6316) and the data correctly ships
`kind: "hermetic"`.

**Narrowed by the verification pass: the twin argument is not perfectly
symmetric.** Famous carries an explicit *"and one type"* instruction that
Infamous lacks, so Infamous is **silent** about the audience rather than
**explicitly open** about it. That weakens "the book says the player chooses"
to "the book says nothing". It does not rescue `local`, because `kind` is a
**hard gate** and not a default: asserting an audience the book never names
produces a `reputation_not_granted` error on a character the rules permit, which
is the one outcome that is wrong under either reading of the silence.

**Correct value.** Drop the `kind` field:
`{ "type": "grants_reputation", "score": 4 }`. The Reputation's negativity
remains unrepresentable (B10's F-380, not counted here).

**Severity: MEDIUM-HIGH.** A false hard error on a legal character, from
over-specified data — the same class of harm as F-445 and F-446, reached from
the opposite direction (too much data rather than too little).

### F-451 — `flaw.hunger_for_form_magic` has no `max_total`, so all ten Forms may be hungered for at once

> "The character has been repeatedly exposed to magic and has thus become
> dependent upon magic to survive. The character must consume 1 pawn of vis each
> season, **corresponding to the Form that it has been mostly exposed to**."
> — ArMDE:6278

And the governing rule:

> "A Virtue or Flaw **may be taken more than once only if the description
> explicitly allows it**. Most Virtues and Flaws may only be taken once."
> — ArMDE:2814

**Current data.** `parameters: [{ "key": "form", "type": "ref", "domain": "form" }]`,
no `max_total`, `classification: "uncomputed_rule"`, full passage in
`description` in both locales ✓.

**Why it is wrong.** `max_total` defaults to the **255** sentinel ("no stated
ceiling") and is keyed on `item_ref` alone, while `max_per_target`'s default of
1 is keyed on `(item_ref, whole params map)`. A parameterized entry therefore
admits one copy *per parameter tuple* — one per Form. ArMDE:6278 names
exactly one Form ("**the** Form that it has been mostly exposed to", singular,
definite) and the passage contains no permission to repeat, so ArMDE:2814's
default applies and a second copy is forbidden.

**Narrowed by the verification pass: the reachable over-permission is three
copies, not ten.** My first draft said "ten copies is 30 Flaw points". That is
not reachable. `validation/balance.rs` errors when `flaw_points` exceeds the
profile's flaw ceiling, which is **10** on companion, magus and
mythic_companion, and grogs are barred outright by `max_major_flaws: 0`. A Major
Flaw is 3 points, so the budget stops the fourth copy. The defect and the fix
are unchanged; the blast radius is **three copies (9 points)** where the book
grants one.

The contrast inside this very batch settles the reading rather than leaving it
to judgement: `flaw.limited_magic_resistance` (ArMDE:6348) says "You may take
this Flaw multiple times, for multiple Forms" and correctly carries **no**
`max_total`, and it is the only entry in the span that does say so.

**Correct value.** `"max_total": 1`, keeping the `form` parameter and
`max_per_target`'s default.

**This is not a new shape** — it is B10's **F-358** (three parameterized Flaws
with no `max_total`) and B11's **F-400** (`flaw.deficient_form` /
`flaw.deficient_technique`) found once more. Stated no wider than checked: `jq`
returns **42 of the catalogue's 51** parameterized entries with no `max_total`;
I read the passages of the three in my own span only, and one of those three
(`limited_magic_resistance`) is correct as shipped.

**Severity: MEDIUM.** A legality gate the book states and the app does not
enforce. It lets an illegal character through rather than blocking a legal one,
so the harm is the opposite of F-445's and lower.

### F-452 — `flaw.lesser_malediction` is `narrative` on a magnitude calibration and on an immunity another chapter attributes to it by name

> "You have been cursed by some supernatural power. **The effects of the curse
> should be about as bad as other Minor General Flaws.**"
> — ArMDE:6344

> "Du wurdest von einer übernatürlichen Macht verflucht. **Die Auswirkungen des
> Fluches sollten ungefähr so schlimm sein wie andere Kleine Allgemeine
> Fehler.**"
> — DE 6344

And, from 1,055 lines further on:

> "Curse-Throwing cannot affect Flaws; specifically, **someone with the Lesser or
> Greater Malediction Flaw is beyond the power of Curse-Throwing**, unless it is
> a Flaw imposed by a faerie or magician with a limited duration."
> — ArMDE:7397 (`#### Curse-Throwing*`)

**Current data.** `classification: "narrative"`, no `effects`, no
`description` in either locale. The shipped summary is the passage's first
sentence only, so the calibration reaches the user nowhere and the immunity
reaches him nowhere either.

**Why it is wrong.**

1. **The calibration is a rule.** "About as bad as other Minor General Flaws" is
   an instruction aimed at a named magnitude class — the class the engine models
   as `Magnitude` and surfaces as `Ruleset.magnitude_points`. It tells the player
   and storyguide how severe the invented curse may be, which is the only
   mechanical content this open-ended Flaw has. No effect calibrates severity
   against a magnitude class, so per **D3** the entry is `uncomputed_rule` with
   the rule written out — never `narrative`.
2. **The immunity is a rule the entry owns and cannot see.** ArMDE:7397 names
   this Flaw by name and makes its holder immune to a modelled Supernatural
   Ability (`ability.curse_throwing`), with its own carve-out for
   limited-duration faerie or magician curses. `grant.rs::GrantConstraint` has
   roughly the right shape and is unreachable from a V/F (B13's Q-107), so this
   too is a D3 case.

**Correct value.** `classification: "uncomputed_rule"`, with **both** clauses in
`description` in both locales — the calibration from ArMDE:6344 and the
Curse-Throwing immunity from ArMDE:7397, the latter cited to its own line so the
next reader can find it.

**Provenance note, and a correction to B13.** B13's **F-442** found the
ArMDE:7397 sentence while rating `flaw.greater_malediction` and carried
`flaw.lesser_malediction` along with it, describing it as "from B15's span".
That is wrong: `flaw.lesser_malediction` cites ArMDE:6342 and sorts to index
483, which is **B14**. The entry is mine, and it is rated here from the source
rather than inherited — ArMDE:7397 was read in full, in context, which is also
how F-453's Leprosy clause turned up in the same sentence.

**The exact wording the phrase screen missed:**

- EN: `The effects of the curse should be about as bad as other Minor General Flaws.`
- DE: `Die Auswirkungen des Fluches sollten ungefähr so schlimm sein wie andere Kleine Allgemeine Fehler.`

No sign, no botch term, no `MECHANICAL_PHRASES` hit. The inbound half is worse
than a missed phrase: the screen reads only an entry's own cited range
(6342-6345), so a rule stated in another chapter is **structurally** invisible
to it — the gap B13 identified on `flaw.greater_malediction` and which this
entry demonstrates a second time.

**Severity: MEDIUM.** Two lost rules, one of them an immunity a player would
have no way to discover from the entry.

### F-453 — `flaw.leprosy` computes two clauses and drops three, in both locales

> "The character has leprosy. A leper has a permanent -2 modifier to her Living
> Condition (**with an additional -1 if she lives in a leper colony**), and
> whenever she undergoes an Aging Crisis (page 392) the leper sustains a Heavy
> Wound in addition to any other result. **Lepers cannot gain a positive
> Reputation due to a pungent rotting smell that they emit.**"
> — ArMDE:6340

> "Der Charakter hat Lepra. Ein Leprakranker hat einen dauerhaften -2-Modifikator
> auf seine Lebensumstände (**mit einem zusätzlichen -1, wenn er in einer
> Leprakolonie lebt**), … **Leprakranke können aufgrund eines beißenden
> Verwesungsgeruchs, den sie ausströmen, keine positive Reputation erwerben.**"
> — DE 6340

And, inbound:

> "note that **curses laid directly by God (such as leprosy) are normally
> represented by permanent Flaws, and thus exempt**"
> — ArMDE:7397 (`#### Curse-Throwing*`)

**Current data.**
`effects: [{ "type": "aging_mod", "kind": "crisis_heavy_wound", "amount": 0 }, { "type": "aging_mod", "kind": "living_conditions", "amount": -2 }]`,
`classification: "in_play_effect"`, **no `description` in either locale**. The
shipped summary is "The character has leprosy." / "Der Charakter hat Lepra." —
four words, no mechanics.

**What is correct, re-derived.** Both effects are right and were verified at the
consumption site rather than by name. `aging_mod { living_conditions, -2 }`:
`aging.rs::living_conditions_modifier` sums it into `from_traits`, and
`aging.rs::aging_total` **subtracts** the summed Living Conditions Modifier, so
a stored -2 *raises* the aging total by 2 — worse aging, which is what leprosy
should do, and the same polarity `flaw.poor_living_conditions` uses ✓.
`aging_mod { crisis_heavy_wound, 0 }`: A38 makes this kind a **predicate**
consumed by `aging.rs::carries_crisis_heavy_wound`, with `amount` ignored and
asserted to be 0 by `data_integrity.rs`; `RULES.md:7210-7213` records the
mapping and its guard
`leprosy_carries_its_crisis_wound_beside_its_living_conditions_penalty` ✓.

**Why it fails.** Three mechanical clauses are computed by nothing and written
nowhere, which D5 forbids regardless of classification:

1. **The leper-colony -1.** A second, conditional Living Conditions modifier.
   `AgingEffect` has no conditional form and the engine models no dwelling, so
   it is inexpressible — and therefore owed to `description`.
2. **The positive-Reputation prohibition.** Identical in mechanism to
   `flaw.judged_unfairly`'s (F-447) and equally inexpressible: `A25` has no
   negative form. `reputationen.md:118` records it independently
   (`Leprosy | Lepra | Alle | – | Kann keine positive Reputation erwerben`).
3. **The Curse-Throwing exemption at ArMDE:7397** — **narrowed by the
   verification pass, and the narrowing is right.** The sentence's leprosy
   mention is *"note that curses laid directly by God (**such as leprosy**) are
   normally represented by permanent Flaws, and thus exempt"* — an
   **illustration** of the sentence's own general rule ("Curse-Throwing cannot
   affect Flaws", which covers all 338 Flaws), not a Leprosy-specific rule. That
   is materially weaker than F-452's Malediction clause, which names two Flaws
   *"specifically"* and gives them their own carve-out. Owing this to
   `flaw.leprosy`'s `description` alone would assert a special exemption the
   book states generally; it belongs on `ability.curse_throwing`. **Recorded,
   not counted as one of this entry's D5 failures.**

**Correct value.** `classification` stays `in_play_effect` (the encoded half
decides the class, per `RULES.md:4633-4639`), and clauses 1 and 2 go into
`description` in **both** locales. Clause 3 is a lead for whoever audits
`ability.curse_throwing`, not a correction to this entry.

**Severity: MEDIUM.** Two lost rules on an entry that otherwise computes
correctly — the D5 shape, not a miscalculation.

### F-454 — `flaw.incomprehensible`'s halving reaches the user in neither locale, and its Lab-Total clause is encoded nowhere

> "You are almost completely unable to convey the knowledge and understanding
> that you have. **Anyone trying to learn from you or from a book you have
> written must halve their Advancement Total** (or Lab Total, if you are a magus
> and have written Lab Texts on some spells or enchanted items). **If you are a
> magus teaching spells, halve all applicable Lab Totals, both yours and the
> student's.**"
> — ArMDE:6296

> "Du bist fast völlig außerstande, das Wissen und das Verständnis zu vermitteln,
> das du besitzt. **Alle, die versuchen, von dir oder aus einem von dir
> verfassten Buch zu lernen, müssen ihren Fortschrittssumme halbieren** (oder den
> Laborgesamtwert, …). **Wenn du ein Magus bist, der Zauber unterrichtet,
> halbiere alle anwendbaren Laborgesamtwerte, sowohl deinen eigenen als auch den
> des Schülers.**"
> — DE 6296

**Current data.**
`effects: [{ "type": "advancement_mod", "source": "teaching", "amount": 0 }]`,
`classification: "in_play_effect"`, no `description` in either locale. The
shipped summary is the passage's first sentence, which states nothing
mechanical.

**What is *not* wrong, and was withdrawn.** I had this drafted as a finding
against the `amount: 0`. `RULES.md:5159` records the zero as a deliberate
convention — "`incomprehensible`/`loose_magic` halve advancement (amount 0
marker; surfaced)" — and `flaw.incomprehensible` / `flaw.loose_magic` are the
only two of the catalogue's fifteen `advancement_mod` rows carrying it, the
other thirteen carrying a real ±N. Withdrawn on that line; B10's rule applied.

**Why it nevertheless fails, on two narrower points.**

1. **D5: the halving reaches the user nowhere.** The marker's whole payload is
   the `source` slug. `derived.rs::in_play_mods` pushes
   `SurfacedModifier { family: Advancement, detail: "teaching", amount: 0 }`, and
   `DerivedSurfacedModifiersSection.svelte:27` guards the value with
   `{#if m.amount !== 0}` — so the read-out prints the label "Teaching" and no
   number and no word "halved". The summary does not carry it either. Contrast
   `flaw.loose_magic`, whose one-sentence passage *is* its summary ("Your
   Advancement Total is halved whenever you try to Master spells"), so the rule
   does reach the player there and no D5 finding lies against it.
2. **A whole second mechanic is encoded nowhere at all.** ArMDE:6296's last
   sentence halves **all applicable Lab Totals, both the teacher's and the
   student's**, and the parenthesis halves the *learner's* Lab Total when
   learning from this character's Lab Texts. `A33 MagicTotalHalving`'s
   `HalvableTotal` has four members — `spontaneous_casting`, `lab_enchanting`,
   `lab_longevity`, `penetration` — and none is a teaching or text-learning Lab
   Total, so the clause is inexpressible. Inexpressible is grounds for writing
   it down (D3/D5), not for dropping it.

**Correct value.** `classification` stays `in_play_effect`; the **full passage**
goes into `description` in both locales, as `flaw.hobbled`, `flaw.lame` and
`flaw.limited_magic_resistance` already do in this same batch. The `amount: 0`
marker stays. Whether the read-out should render the marker as a word rather
than an empty value is **Q-113**.

**Severity: MEDIUM.** Two lost rules on a computed entry.

### F-455 — `flaw.infamous`'s `source.lines` stops one line short of its passage's end

**Current data.** `"lines": [6310, 6312]`.

**The source.** ArMDE:6310 `#### Infamous`, 6311 `*Minor, General*<br>`, 6312
the body, 6313 blank, **6314 `#### Infamous Master`**. Every other entry in this
span ends on the blank line before the next `####` heading; this one ends on
6312.

**Why it matters.** `source.lines` is consumed by no runtime code, but it is
consumed by `crates/arm-rules/tests/rules_source_provenance.rs`, by
`rules_md_citations.rs`, by `uncomputed_clauses.rs`'s block screen (which reads
**only** the cited range) and by this audit's own batch ordering. A range that
is short by a blank line loses nothing here — the body is on 6312 — but it is
the one entry in 35 that departs from the convention, and the convention is what
lets a reader tell a deliberate range from a truncated one.

**Correct value.** `"lines": [6310, 6313]`.

**Relationship to B12's Q-97.** Q-97 asks whether ending a range on a trailing
blank line is right at all. This finding does **not** re-open that: it reports
the inconsistency, in the direction the other 34 entries of this batch and the
31 of B13 establish as the house style. If Q-97 is ever settled the other way,
this correction inverts along with all of them.

**Severity: LOW.** A provenance inconsistency with no computed consequence.

### F-456 — `grundbegriffe.md:325` gives `Hobbled` a German name that is already `flaw.crippled`'s

**The table row.**

> `| Hobbled | Verkrüppelt | Kleiner Allgemeiner Fehler; beide Beine schwer geschädigt |`
> — `rules/source/de/translation-tables/grundbegriffe.md:325`

**The rulebook.**

> `#### Humpelnd` — DE 6260, `*Klein, Allgemein*`, "Beide Beine des Charakters
> sind stark beschädigt."

> `#### Verkrüppelt` — DE 5877, `*Groß, Allgemein*`, "Du hast entweder keine
> Beine, oder deine Beine sind vollständig unbrauchbar."
> (EN: `#### Crippled`, ArMDE:5877, `*Major, General*`.)

**Current data.** `flaw.hobbled` ships DE `Humpelnd`; `flaw.crippled` ships DE
`Verkrüppelt` (`rules/i18n/de/virtues_flaws.json`). Both follow their own
rulebook heading, and the DE book's back index corroborates Hobbled at DE 24857
(`| Humpelnd (Fehler) | [133](#humpelnd) |`).

**Why the row is wrong, and why this one does not need D6's precedence question
answered.** `Hobbled` (ArMDE:6260, *Minor, General*, both legs damaged, can walk
with crutches at half a mile per hour) and `Crippled` (ArMDE:5877, *Major,
General*, no legs or wholly useless legs, cannot walk at all) are two distinct
catalogue entries of different magnitudes. Adopting the table's value would give
them **the same German name**, which is exactly the collision D6's corrected
text calls "a defect in the rendering, not a rule the book states". The table
itself offers no row for `Crippled`, so it never had to reconcile the two. There
is no reading on which `Verkrüppelt` is right for `Hobbled` while `Crippled`
exists.

**Report form, per D6.** `arm-de-translation` is outside this repository and the
read is denied, so this is stated as **"this repository's copy of
`grundbegriffe.md:325` disagrees with the rulebook"**, and the
error-versus-stale-copy determination is left to the orchestrator, who can check
the source project. The collision evidence holds either way.

**Correct value.** `| Hobbled | Humpelnd | … |`, and a new row
`| Crippled | Verkrüppelt | … |` so the pair is reconciled in the table rather
than only in the data. The shipped data needs no change.

**Severity: MEDIUM.** A wrong terminology row is a *generator* of future wrong
data (D6's own argument), and this one would produce two indistinguishable
Flaws in the German UI.

### F-457 — `tugenden-fehler.md:344` renders `Folk Tradition` as `Volksmagie`

**The table row.**

> `| Imagined Folk Tradition Vulnerability | Eingebildete Volksmagie-Verwundbarkeit | |`
> — `rules/source/de/translation-tables/tugenden-fehler.md:344`

**The rulebook.**

> `#### Eingebildete Volksüberlieferungsanfälligkeit` — DE 6280
> (EN: `#### Imagined Folk Tradition Vulnerability`, ArMDE:6280.)

**Current data.** `flaw.imagined_folk_tradition_vulnerability` ships DE
`Eingebildete Volksüberlieferungsanfälligkeit`, following the heading.

**Why the row is wrong.** The English head noun is **Folk Tradition**, which is
*Volksüberlieferung*. *Volksmagie* is the German for **folk magic**, a different
term that appears in the passage's *body* — DE 6282, "gegenüber traditionellen
Schutzmitteln und **Volksmagie**", rendering ArMDE:6282's "traditional wards and
folk magic". So the table has taken a term from the body and used it to
translate the heading, which makes this a mistranslation rather than a competing
but defensible rendering. (The second half, *Verwundbarkeit* for *Vulnerability*
against the heading's *Anfälligkeit*, is a genuine synonym choice and would not
be worth a finding on its own.)

**Report form, per D6.** Stated as **"this repository's copy of
`tugenden-fehler.md:344` disagrees with the rulebook"**; the
error-versus-stale-copy determination is the orchestrator's. See **Q-114** for
the precedence question this row and F-456 raise together.

**Widened by the verification pass: the same head noun is mistranslated a second
time, in a second table, on a second entry.**

> `| Vulnerable to Folk Tradition | Anfällig für Volkszauber | Kleiner Hermetischer Fehler |`
> — `rules/source/de/translation-tables/grundbegriffe.md:410`

against DE 7011's heading `#### Anfällig für Volksüberlieferungen` and the
shipped DE name `Anfällig für Volksüberlieferungen`
(`flaw.vulnerable_to_folk_tradition`, ArMDE:7011-7014 — **B19's span**, carried
here because it is the same error and would otherwise be found twice). So one
English term, **three** German renderings across the repo: *Volksmagie*
(`tugenden-fehler.md:344`), *Volkszauber* (`grundbegriffe.md:410`) and
*Volksüberlieferung* (the rulebook and both shipped names). The first is
`tugenden-fehler.md:124`'s own rendering of a *different* English term (`Folk
Magic | Volksmagie`); the second is a third invention. This is no longer one
slip — it is an unreconciled term, which is exactly the job the tables exist to
do.

**Correct value.** `| Imagined Folk Tradition Vulnerability | Eingebildete
Volksüberlieferungsanfälligkeit | |` and `| Vulnerable to Folk Tradition |
Anfällig für Volksüberlieferungen | … |`. The shipped data needs no change in
either case.

**Severity: LOW-MEDIUM.** Two terminology-table errors with no shipped
consequence today, but a future German-text pass would reproduce them — and per
D6 a wrong table is a *generator* of future wrong data.

### F-458 — `flaw.loose_magic`'s German summary carries a gender error from the source

**Current data.** `rules/i18n/de/virtues_flaws.json`, `flaw.loose_magic`:

> `"summary": "Dein Fortschrittssumme wird halbiert, wann immer du versuchst, Zauber zu meistern."`

**The source.** DE 6356, verbatim: "**Dein** Fortschrittssumme wird halbiert,
wann immer du versuchst, Zauber zu meistern."

**Why it is wrong.** *die Summe* is feminine, so the possessive must be
**Deine** Fortschrittssumme. The shipped summary is a faithful copy of a
grammatical error in the German rulebook. The same error occurs a second time at
DE 6296 ("müssen **ihren** Fortschrittssumme halbieren" — should be *ihre*), but
that sentence is not sentence one of its entry, so it does not reach the shipped
text; `flaw.loose_magic`'s whole passage *is* one sentence, so DE 6356's error
ships.

**Correct value.** `"Deine Fortschrittssumme wird halbiert, wann immer du
versuchst, Zauber zu meistern."` This is a deviation from the source, which the
audit normally avoids — but the deviation is a *correction of a typo*, in the
same class as B13's note that `flaw.the_falling_evil` must take its name from
the heading rather than the body, and the alternative is shipping ungrammatical
German in the UI. If the correction pass prefers strict source fidelity, the
finding becomes an upstream report against the German rulebook instead.

**Severity: LOW.** A localization defect, visible to every German-speaking user
in the Virtue/Flaw list — which `CLAUDE.md` rates up, not down — but a single
word on a single entry.

### F-459 — `flaw.low_self_esteem`'s "never" is defeated by a Virtue, and no incompatibility stops the pair

*(Found by the verification pass, on an entry this file had passed on all twelve
checks.)*

> "You have a deflated opinion of your own self-worth. **You begin the game with
> no Confidence Score, and never have any Confidence Points.**"
> — ArMDE:6364

> "Du hast eine gedämpfte Meinung von deinem eigenen Selbstwert. **Du beginnst
> das Spiel ohne Selbstvertrauenswert und hast niemals Selbstvertrauenspunkte.**"
> — DE 6364

The two Virtues that undo it:

> "you have a Confidence Score of **two**. You also start with **five**
> Confidence Points, rather than the usual three."
> — ArMDE:4902 (`virtue.self_confident`, *Minor, General*)

> "take **3 Confidence points** and a **Confidence Score of 1**…"
> — ArMDE:3875 (`virtue.ferocity`)

**Current data.** The catalogue's only three `confidence_bonus` carriers are
`flaw.low_self_esteem { score: -1, points: -3 }`,
`virtue.ferocity { score: 1, points: 3 }` and
`virtue.self_confident { score: 1, points: 2 }`. **`incompatible_with` is absent
on all three.** `flaw.low_self_esteem` carries no `description` in either
locale; its summary is the passage's first sentence, which states nothing
mechanical.

**Why the arithmetic passes and the entry still fails.**
`effective/gift_confidence.rs::confidence` computes
`score = clamp(base_score + Σ score)` and `points = clamp(base_points + Σ points)`
with `clamp(n) = u8::try_from(n.max(0))` — a **sum**, floored at 0. Against the
profile bases of 1 / 3, Low Self-Esteem alone yields exactly 0 / 0 ✓, which is
what I checked and why I passed it. But the sum is the whole mechanism: a
companion holding **Low Self-Esteem and Self-Confident** — both *Minor/Major
General*, neither forbidden to that profile, nothing in
`validation/` objecting — ships

```
score  = clamp(1 + (-1) + 1) = 1
points = clamp(3 + (-3) + 2) = 2
```

**Confidence Score 1, Confidence Points 2.** No passage sanctions that figure:
Self-Confident's own sentence says 2 and 5, Low Self-Esteem's says 0 and
"never". The clamp never fires, because the Virtue lifts the running total back
above 0 before it is reached. The pairing with `virtue.ferocity` gives 1 / 3 —
the *default* a character with neither trait has, i.e. two traits that cancel to
nothing while costing and granting points.

**Why this is a rules defect and not a modelling opinion.** ArMDE:6364 is not a
modifier, it is an **absolute**: "**never** have any Confidence Points" / "hast
**niemals** Selbstvertrauenspunkte". An absolute cannot be expressed as a
summand, and the engine models it as one. `RULES.md:5203` acknowledges the model
("note: additive, so it zeroes only the default") but records no exclusion, no
guard and no open question, so the gap is documented as a property rather than
flagged as one.

**Correct value.** A mutual `incompatible_with` between `flaw.low_self_esteem`
and both `virtue.self_confident` and `virtue.ferocity` — the smallest fix, and
the one the audit already prescribes for every other stated-or-implied
exclusion. The "never" absolute must additionally be written into `description`
in both locales (D5), because even with the exclusion in place the permanence
("never", across play, not just at creation) is not something an additive
`confidence_bonus` expresses.

**Why no phrase screen could have caught it.** ArMDE:6364 contains no signed
number, no botch term and no `MECHANICAL_PHRASES` hit, but that is beside the
point: the entry is `creation_effect`, not `narrative`, so the screen never
looked at it. This defect is invisible to *every* guard the repo has, because it
is not a property of one entry at all — it is a property of a **pair**, and
nothing in the data or the tests ranges over pairs.

**Severity: MEDIUM.** Wrong rules output on a character the validator calls
legal. It is rated below F-449 only because it needs an unusual (though entirely
permitted) combination, where F-449 fires on the single Flaw alone.

## Open questions

### Q-111 — does `flaw.independent_craftsman`'s own recategorization clause apply, given that *City and Guild* is not in this repository?

ArMDE:6304 ends: "**If you are not using the rules in City and Guild (page 73),
treat this is a Personality Flaw.**" (DE 6304: "Wenn du nicht die Regeln in
**Stadt und Gilde (Seite 73)** verwendest, behandle dies als
**Persönlichkeits-Fehler**.") *City and Guild* is not among the nine files in
`rules/source/en/`, so this app is by construction "not using" its rules — which
means the book's condition is **satisfied** and the Flaw should be filed
`personality`, not `general`. That is not cosmetic: `personality` is capped at
`{"max":1,"major_only":true,"hard":true}` plus `{"max":2}` on every profile
(`character_types.json` → `budget.flaw_category_caps`), while `general` is
uncapped, so the category decides whether a character may hold this Flaw beside
two others.

Against that, **both** of the audit's independent sources say General: the
descriptor at ArMDE:6303 (`*Minor, General*`) and the book's own index at
ArMDE:5592, filed under `### General, Minor`. The data follows them, and the
clause ships in `description` in both locales, so nothing is lost.

**What would settle it.** A ruling on whether a conditional recategorization
keyed on an absent supplement is applied by this app, or whether the printed
descriptor governs and the clause stays as recorded text. The same ruling would
generalise: a grep for "treat this as a" over the Flaws block would show whether
this is the only instance.

### Q-112 — does `virtue.lone_redcap` satisfy `flaw.hermetic_patron`'s "a Redcap"?

F-448 recommends
`Any([Has(virtue.redcap), Has(virtue.lone_redcap), IsMagus])` for ArMDE:6250's
"You must be a Redcap or magus to take this Flaw." The catalogue holds two
Redcap Social Status Virtues — `virtue.redcap` (Major) and `virtue.lone_redcap`
(Minor) — and the one existing precedent, `virtue.magic_items`, names
**`virtue.redcap` alone**. Whether that precedent is a deliberate exclusion of
Lone Redcaps or simply the narrower case nobody revisited is not settled by
anything I read.

**What would settle it.** Reading `virtue.lone_redcap`'s own passage (it sorts
into B05's span) to see whether a Lone Redcap is a Redcap for rules purposes,
and then applying one answer to both entries rather than leaving two Redcap
prerequisites written differently.

### Q-113 — should an `advancement_mod` marker of `amount: 0` render as a word?

`RULES.md:5159` records the zero amount on `flaw.incomprehensible` and
`flaw.loose_magic` as a deliberate "halve advancement" marker, and they are the
only two of fifteen `advancement_mod` rows carrying it.
`DerivedSurfacedModifiersSection.svelte:27` guards the value with
`{#if m.amount !== 0}`, which is right for the variants whose zero means "no
magnitude carried" (`A37`'s four surfaced kinds, `A40`'s eight) and wrong here,
where the zero means "halved". The user sees a labelled row with nothing beside
it, indistinguishable from a modifier of no size.

**What would settle it.** A decision between three routes: (a) render a
localized "halved" token for the marker, which needs a way to tell this zero
from the others; (b) leave the read-out alone and discharge the rule entirely
through `description`, which is F-454's recommendation and needs no code; or (c)
give `AdvancementMod` an explicit halving discriminator. (b) is the smallest and
is what F-454 assumes; the question is whether a surfaced row that says nothing
is acceptable alongside it.

### Q-114 — under D6 as corrected, does the glossary or the shipped data win on F-456 and F-457?

D6's corrected text splits precedence by the kind of claim: a **factual** claim
about the rules → the rulebook wins; a **terminology** claim → the glossary wins,
"thematic table > `tugenden-fehler.md`", with the DE rulebook heading as "strong
evidence … but not decisive". Both of my table rows are terminology claims, so
read literally, rule 2 says the *glossary* wins and the shipped German names are
the things to change.

That cannot be right for **F-456**: adopting `Verkrüppelt` for `Hobbled` would
collide with `flaw.crippled`, which D6's own worked example (`Gefesselte Magie`)
treats as decisive evidence against the colliding name. So F-456 resolves
against the table on D6's own collision reasoning, independently of rule 2.
**F-457** has no collision to lean on, and there rule 2 read literally would
make `Eingebildete Volksmagie-Verwundbarkeit` the correct shipped name — which I
do not believe, because the row mistranslates the English head noun. B13's
**F-438** hit the same shape twice and resolved both *against* the table, under
the pre-correction wording.

**What would settle it.** The orchestrator's check of `arm-de-translation` (per
D6's "the copy drifts" paragraph), plus a one-line clarification in D6 that rule
2's "glossary wins" does not extend to a row that renders a *different* English
term than the one it is keyed to. Without that, a future batch facing a plain
terminology disagreement with no collision has no rule it can apply without
contradicting one of the two readings.

### Q-115 — is `flaw.inscribed_shadow`'s House Criamon restriction a prerequisite to encode?

ArMDE:6320: "**Only characters with stigmata may have this Flaw, which normally
means that they must be magi of House Criamon.**" (DE 6320: "Nur Charaktere mit
Stigmata dürfen diesen Fehler haben, was normalerweise bedeutet, dass sie Magi
des Hauses Criamon sein müssen.") The restriction is expressible —
`Prereq::House(house.criamon)` exists and `virtue.the_enigma` uses it — and the
entry ships `narrative`-free (`uncomputed_rule`, full passage in both locales),
so nothing is lost today.

Two things stop me calling it a finding. The actual gate is **stigmata**, which
the engine does not model at all, so a House prerequisite would encode the
book's *consequence* rather than its condition. And the book hedges that
consequence twice — "**normally** means" — so a hard `house` prerequisite would
forbid a character the book leaves room for, which is the failure mode F-445,
F-446 and F-450 all are.

**What would settle it.** A ruling on whether a hedged restriction ("normally
means") is encoded as a hard prerequisite, encoded as a warning, or left as
recorded text. The answer generalises: a grep for "normally means" / "which
usually means" over the V/F blocks would size the population.

### Q-116 — the book defines "combat scores", and the repo's reading of the term excludes three of the five

*(Raised by the verification pass, overturning this file's clean verdicts on
`flaw.hobbled` and `flaw.lame`.)*

`RULES.md:5010-5019` and the guard `data_integrity.rs` argue that "combat
rolls" / "combat scores" in a Flaw's passage means "the totals that take a
Combat Ability — Attack and Defense", citing ArMDE:16658/:16660/:16662/:16664/
:16666. Neither cites the line immediately above those formulas:

> "Characters have **five combat scores: Initiative, Attack, Defense, Damage,
> and Soak.**"
> — ArMDE:16656

That is a definition of the exact term in dispute, and both entries use the term
literally:

- **ArMDE:6332** (`flaw.lame`) — "…-3 on Dodge, and **-1 on other combat
  scores**." / DE 6332 "…-3 auf Ausweichen und **-1 auf andere Kampfwerte**."
  Under ArMDE:16656, "other combat scores" is Initiative, Damage and Soak. The
  data covers Attack and Defense.
- **ArMDE:6262** (`flaw.hobbled`) — "Her Dodge and **other combat rolls** are
  penalized by -6." Of the five scores, the three that end "+ Stress Die" are
  Initiative, Attack and Defense, so "combat rolls" reaches **Initiative**. The
  data covers Attack and Defense.

Dodge is not a counterexample: ArMDE:16959 gives it `Init 0`, `Atk n/a`,
`Dfn 0`, `Dam n/a` — it has an Initiative column. Nor is the catalogue's
consistency an argument: `jq` shows `initiative` is used only where a passage
names it outright (`flaw.slow_reflexes`, `virtue.fast_caster`,
`virtue.lightning_reflexes`), which is the *practice* under review rather than
evidence for it.

The arithmetic both entries were passed on is **not** in dispute and was
verified at the consumption site: `derived.rs::in_play_mods` stores Lame's
scoped Dodge row as `-3 - (-1) = -2`, so `cm(Defense, weapon.dodge)` yields -3
and not -4 ✓.

**What would settle it.** A ruling on whether ArMDE:16656's five-score
definition governs the Flaws that use its term, or whether `RULES.md:5010`'s
narrower reading stands with ArMDE:16656 cited and argued against. The answer
generalises immediately: the same argument reaches `flaw.missing_hand`,
`flaw.palsied_hands`, `flaw.missing_eye` and `flaw.poor_eyesight`, all pinned by
the same guard, so this is a six-entry decision and not a two-entry one. Until
it is taken, neither entry is marked correct and neither is marked defective.

### Q-117 — two in-repo authorities give opposite readings to "the passage states the *absence* of a rule", on two entries in this span

*(Raised by the verification pass, overturning this file's clean verdicts on
`flaw.jinxed` and `flaw.horrifying_appearance_snake_legs`.)*

> "`flaw.jinxed` is worth noting as the one that states the **absence** of a
> botch change ('need not roll any extra botch dice'). **That is still a rule** —
> a clarification a player needs — so it classifies the same way
> [`uncomputed_rule`]."
> — `crates/arm-rules/RULES.md:4684-4686`

> "the passage's one near-mechanical sentence, 'Your movement is not hindered
> under most circumstances', **explicitly declines to impose a penalty**. Nothing
> rolls, nothing is capped, nothing is modified. Pure body-horror description,
> which is `narrative`."
> — `crates/arm-rules/tests/uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`,
> row `flaw.horrifying_appearance_snake_legs`

The two passages, verbatim, in both locales:

- `flaw.jinxed`, **ArMDE:6324**: "He is not personally the cause of the bad
  luck, and so **need not roll any extra botch dice or suffer any penalty to die
  rolls.**" / **DE 6324**: "Er ist nicht persönlich die Ursache des Unglücks,
  und so **muss er weder zusätzliche Patzerwürfel würfeln noch Abzüge auf seine
  Würfe hinnehmen.**" → `uncomputed_rule`, full passage shipped in both locales.
- `flaw.horrifying_appearance_snake_legs`, **ArMDE:6266**: "**Your movement is
  not hindered under most circumstances.**" / **DE 6266**: "**Deine Bewegung
  wird unter den meisten Umständen nicht behindert.**" → `narrative`, **no
  `description` in either locale**, summary stopping at sentence one, and an
  explicit screen exemption.

I checked the exemption's written reading, as the brief requires, and confirmed
it against the passage — but I did not notice that the two readings cannot both
be right, and the verification pass did. A distinction does exist and is worth
recording so whoever rules has it: `flaw.jinxed` **also** carries a positive
rule in the same passage ("a jinxed character should be affected **half of the
time**, and his comrades between them the other half"), which independently
justifies `uncomputed_rule` — but `RULES.md:4684` does not rest on that
sentence, it rests on the absence-is-a-rule principle, and that principle is
what the exemption denies.

**What would settle it.** A ruling on whether "the book explicitly says this
imposes no penalty" is a rule a player needs written down. If it is,
`flaw.horrifying_appearance_snake_legs` moves to `uncomputed_rule` and the
`NO_RULE_DESPITE_TOKEN` row must be deleted rather than reworded (the module
asserts every row still fails the screen). If it is not, `RULES.md:4684`'s
sentence must be rewritten to rest on Jinxed's half-the-time clause instead.
Either way one of two authorities changes, which is why this is escalated rather
than decided here.

**One DE-source defect recorded against no data, found in the same pass.** DE
6266 renders "leaving you **feigning being crippled** as well" as "sodass du
auch **als Gelähmter gelten musst**". *"gelten als"* is the German rules idiom
for *counts as / is treated as*, so the German turns a concealment instruction
into an apparent mechanical equivalence the English does not state. Nothing
ships today, because the DE summary stops at sentence one — but if Q-117 is
settled such that the full passage goes into `description`, it would ship a
mechanical claim the source of truth does not make. English is the source of
truth (`CLAUDE.md`); recorded so the correction pass renders ArMDE:6266 rather
than DE 6266 at that clause.

## Sub-agent reconciliation

An independent verification sub-agent re-derived all 22 entries this file
originally called clean, all fourteen findings, and every numeric claim in the
Method. **It overturned three clean verdicts, escalated two more, confirmed all
fourteen findings — narrowing three, widening two, and correcting the proposed
fix on a third — and found three counting errors in this file's own Method, one
of which had a headline census exactly backwards.** Everything below is folded
into the sections above; this section records what changed and why, so the
delta is auditable rather than invisible.

### The five entries whose verdicts changed

| Entry | Was | Now | Why |
|---|---|---|---|
| `flaw.low_self_esteem` | clean | **F-459** | The 0/0 arithmetic is right and is not the whole question. `effective/gift_confidence.rs` sums additively and `incompatible_with` is empty on all three `confidence_bonus` carriers, so Low Self-Esteem + Self-Confident ships Confidence 1 / 2 points — against an absolute ("**never** have any Confidence Points", ArMDE:6364). **The batch's second live wrong number.** |
| `flaw.hobbled` | clean | **Q-116** | ArMDE:16656 defines five combat scores; the entry says "other combat rolls" and the data covers two. |
| `flaw.lame` | clean | **Q-116** | ArMDE:6332 says "other combat **scores**", the term ArMDE:16656 defines. The Dodge arithmetic (-3, not -4) is confirmed correct. |
| `flaw.jinxed` | clean | **Q-117** | Its `uncomputed_rule` rests on `RULES.md:4684`'s absence-is-a-rule principle, which the screen exemption on the next entry denies. |
| `flaw.horrifying_appearance_snake_legs` | clean | **Q-117** | The other half of the same contradiction, plus a DE-source deviation at DE 6266. |

The remaining **17** entries are confirmed clean, each re-derived rather than
accepted. Three confirmations added evidence this file did not have and which is
now folded in above: `flaw.impious_friend` (ArMDE:2998-3002's tainted
half-points cap **is** enforced, by `validation/caps.rs::validate_tainted_cap`
as a warning, so no gap hides behind `tainted: true`); `flaw.inconstant_magic`
(the -3 Finesse is genuinely inexpressible — `AbilityRollMod` is keyed on a
free-text *subject*, and the catalogue's only `ability_bonus` carrier is
`virtue.puissant_ability` at +2, with **zero** negative uses); and
`flaw.hedge_wizard` / `flaw.infamous_master` (both passages **do** name their
audience, at ArMDE:6242 "within the Order of Hermes" and ArMDE:6316 "among
magi", which is what makes `flaw.infamous`'s silence at ArMDE:6312 the anomaly
F-450 reports rather than a house style).

### The fourteen findings

All **CONFIRMED**. Six carry a change:

- **F-446** — the proposed fix was **wrong and is corrected**. I wrote "two
  `parameters` of domain `technique` and `form`", which models *one* pair;
  ArMDE:6292 says "**two combinations**" and ArMDE:10163 writes an instance out
  as "**(MuCo & PeAn)**". One selection needs **four** refs. A correction pass
  taking my first wording would have shipped a Flaw forbidding half of what the
  book forbids.
- **F-449** — **widened**, and the widening is the sharper half. The wrong
  behaviour is asserted as correct in two places: `derived/combat.rs:354`'s own
  comment and, more seriously, **`engine-semantics.md:1455`**, the yardstick
  every batch of this audit measures against. Reading A36 against the data —
  which is what check 10 asks for — could never have found this, because the
  yardstick and the data agreed and only the code dissented.
- **F-450** — **narrowed.** `virtue.famous` carries an explicit "and one type"
  that `flaw.infamous` lacks, so Infamous is *silent* rather than *explicitly
  open*. `kind: "local"` stays indefensible, because `kind` is a hard gate.
- **F-451** — **narrowed.** The reachable over-permission is **three** copies
  (9 points against a 10-point ceiling, `validation/balance.rs`), not ten.
- **F-453** — **narrowed.** ArMDE:7397's leprosy mention illustrates the
  sentence's general rule rather than stating a Leprosy-specific one, unlike the
  Malediction clause which names two Flaws "specifically" with its own carve-out.
  Clause 3 is now a lead for `ability.curse_throwing`, not a D5 failure of this
  entry. Clauses 1 and 2 stand.
- **F-457** — **widened.** `grundbegriffe.md:410` mistranslates the same head
  noun a second time (`Vulnerable to Folk Tradition → Anfällig für
  Volkszauber`), so one English term has **three** German renderings across the
  repo. The second row lands on `flaw.vulnerable_to_folk_tradition`, **B19's**
  span, and is carried here so it is not found twice.

The two verdicts most worth recording as *unchanged*: **F-454**'s withdrawal of
the `advancement_mod { amount: 0 }` finding on `RULES.md:5159` was confirmed
right, and so was keeping the D5 and Lab-Total halves; and **F-452**'s
provenance correction to B13 was confirmed — `batch-13.md` calls
`flaw.lesser_malediction` "B15's span" at four places and at `:1748` even
instructs "B15 must not re-litigate the Minor twin", when the entry sorts to
index **483**, which is B14.

### Three counting errors in this file, all corrected above

1. **The German-name census was inverted.** I wrote "thirteen … have a row …
   The remaining nineteen … have no table row". It is **19 with a row, 13
   without** — and the error was visible inside this file, whose comparison
   table already had seventeen rows and which then quoted
   `reputationen.md:117-118` for two names it had just listed as table-less.
   17 + 2 = 19. Corrected, and the two names added to the table. The two
   substantive disagreements (F-456, F-457) are unaffected.
2. **The English en-dash census was wrong in three places.** Actual: en dash at
   ArMDE:6262, 6332, 6352, 6368; ASCII at 6274, 6278, 6300, 6304, 6340. I had
   6278 in the wrong list, 6352 and 6368 in the wrong list, and omitted 6340.
   The cause is worth the line because `CLAUDE.local.md` warns about it
   specifically and I walked into it anyway: the `grep` shim's ugrep dialect
   returned an incomplete match set and `rg -n "–[0-9]"` returned the right one.
   Re-run with `rg` and corrected. The shipped-data conclusion was independently
   right.
3. **`budget.confidence_score` / `budget.confidence_points` is the wrong path.**
   They are **top-level** profile fields in `character_types.json`; `budget`
   holds only the point totals, the major/minor caps and the category caps. The
   values (1/3, absent on grog) were right.

The header, the verdict table's 35 rows, the classification census, the anchor
and description counts, the ArMDE:2814/:2816/:2818/:2820 instance counts, the
42-of-51 and 19-prerequisite and 15-`advancement_mod` figures were all
re-derived and **match**.

### Three things the verification pass added that are not findings

- **This file's inbound-pointer census was incomplete.** I wrote "the two
  inbound pointers a reader of the span alone can never see"; there are at least
  four. The two I missed are **ArMDE:4251** (`virtue.leper_magus`: "This Virtue
  **can only be bought if the character also has the Leprosy Flaw**, and is only
  available to magi trained in House Tytalus" — F-448's shape, naming
  `flaw.leprosy`; the defect itself is **not** new, B05's **F-136** already
  records both missing prerequisites) and **ArMDE:10163/:10168** (Beast Masters,
  which I had already found independently and which corrects F-446). Pointers
  followed and discarded, recorded so they are not re-chased: ArMDE:13708
  ("Curse of the Leprous Flesh" inflicts a wound, explicitly *not* the Flaw),
  ArMDE:21016 (`##### Horrifying Appearance`, a distinct "Corrupted Beasts only"
  Flaw), and the sample-character lines at :1716, :11598, :17896, :18293,
  :18332, :19784, :21002, :21094.
- **A second terminology table makes a provenance claim the core book
  contradicts.** I caught `tugenden-fehler.md:687` (Hermetic Patron attributed
  to *HoH:TL*) and called it noted-not-counted; the same shape occurs again in
  this batch at **`tugenden-fehler.md:330`**, attributing `Hunger for (Form)
  Magic` to *SdM:M* (RoP:M) when the core book carries it at ArMDE:6276. Two
  instances in 35 entries, both D6-rule-1 violations by a terminology table.
- **A truncation shape this file's scan cannot see, proved reachable one entry
  past the span.** `flaw.lycanthrope` (index 490, **B15**) ships a DE summary
  cut mid-sentence **with no ellipsis**: `"…in ein gefährliches Raubtier (z.B."`
  against DE 6372 "…(z.B. Wolf, Luchs oder Bär) zu verwandeln." My truncation
  scan was a literal-ellipsis scan and said so; this is exactly the case it
  warned it would miss. A lead for B15, not a B14 finding.

### Method notes the verification pass confirmed

`flaw.jinxed` is the one `uncomputed_rule` entry with no `anchor`, and it would
be `jinxed` (ArMDE:5594). ArMDE:16965's source typo "Braw1" did **not**
propagate — `equipment.json`'s `weapon.knife` ships `ability.brawl`. The DE link
defect at DE 6292 (`[Seite 125](#auffällige-gabe)`) is confirmed, as is DE
6384's legitimate use of the same anchor for its own "Seite 120".
