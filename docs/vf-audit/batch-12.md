# Batch B12 — indices 385-419, ArMDE:5932-6067

Entries: 35. Audited: 35. Failures: **15**. Clean: **20**.
Findings **F-408 … F-427** — three lie outside the batch: **F-413**
(`virtue.variable_power`, B09's span), **F-426** (`flaw.ability_block`, B10's
span) and **F-427** (ArMDE:2816, no entry's defect at all).
Open questions **Q-97 … Q-102**.

The verification sub-agent re-derived all 20 entries this file called clean and
**overturned none**; it confirmed all four high-stakes findings, widening one and
narrowing the reasoning of two, contributed the two out-of-span findings above,
established six negatives this file had assumed — **and overturned a claim in this
file's own method section**, an inherited ArMDE:2816 verdict I carried across from
B10 without checking, which turned into F-427. See the reconciliation section.

**The batch's first shape is a class of rule the catalogue has no habit of
encoding at all: a Virtue/Flaw that grants permission to buy a *gated* Ability.**
Three entries here state it — `flaw.diabolic_past`, `flaw.faerie_friend`,
`flaw.faerie_upbringing` — and all three are `narrative` with no `effects`, so a
companion or grog who does exactly what the Flaw permits raises a hard
`ability_category_requires_virtue` error today. It is not three accidents: only
**two** entries in the whole 655-entry catalogue carry `ability_authorization`,
while the core rulebook states the permission idiom in at least **twenty-three**
passages (F-409). B11's F-407 found the twenty-fourth from the other direction.

**The second shape is a stated Reputation that is simply missing.**
`flaw.excommunicate` says "a bad reputation at level 3 within the Church" and
carries nothing, where `flaw.apostate` and `flaw.failed_monk` — the same clause,
the same audience, twenty lines and two thousand lines away — each carry a
`grants_reputation { ecclesiastical, N }` (F-408).

**The third shape is `narrative` on a stated rule, again, and again inside the
twice-swept block.** Eleven of this batch's twenty-five `narrative` entries state
a mechanical clause; B10 found nine, B11 found seven, and the screen is green on
all eleven of mine. F-424 tabulates the exact word-forms it lacks. Two of the ten
are formulas written out in full — `flaw.deteriorating_power`'s
`(age / 10)` / `(Might Score – Might Pool / 5)` and, in B09's span, its twin
`virtue.variable_power`, which sits outside every swept block and has therefore
never been screened at all (F-413).

*(Written incrementally: header, method and the 35-row verdict table first,
findings appended one at a time, sub-agent reconciliation last.)*

## Method

**Both languages were read as continuous prose before any entry was judged** —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` 5915-6084 and
the line-parallel `rules/source/de/Ars Magica Definitive Edition Basisregeln.md`
5915-6074, deliberately overrunning the span at both ends so the first and last
entries' boundaries could be seen against their neighbours (`flaw.demonic_familiar`
5926-5931 before, `flaw.failed_monk` 6068-6071 after).

**Line parity holds throughout the span.** Every one of the 32 `####` headings in
5932-6067 sits on the identical line number in both files (5932, 5936, 5940,
5944, 5950, 5954, 5958, 5962, 5966, 5972, 5976, 5980, 5984, 5988, 5992, 5996,
6000, 6004, 6008, 6012, 6016, 6020, 6024, 6028, 6036, 6040, 6044, 6048, 6052,
6056, 6060, 6064), and so does the next heading after the span (6068) and the two
before it (5917, 5921, 5926). 32 headings for 35 catalogue ids — the three
Major/Minor pairs `Depraved` 5936, `Driven` 5988 and `Envious` 6016 each ship as
two ids citing one heading.

**The batch's own classification census differs from the brief's.** The brief
stated 28 `narrative` / 3 `uncomputed_rule`; the data says **25 `narrative`, 4
`uncomputed_rule`, 3 `creation_effect`, 3 `in_play_effect`** (25+4+3+3 = 35). The
fourth `uncomputed_rule` is `flaw.evil_eye`. Stated so the next reader is not
hunting for a discrepancy.

**Check 1 — `source.lines`: 35 of 35 correct.** Every range runs from its own
`####` heading to the line before the next heading, with no tighter variant
anywhere in this batch. No range swallows a neighbour and none is short. One
range is nevertheless *declared wrong by `crates/arm-rules/RULES.md`* —
`flaw.difficult_longevity_ritual` 5962-5965, where RULES.md:4471-4474 says "line
5965 is blank, so the accurate inclusive range is **ArMDE:5962-5964** … The JSON
value is left for a data-only correction pass." That is a convention conflict
between the traceability map and this audit's check 1, not a defect of the entry;
it is **Q-97**.

**Check 1, second half — `source.anchor`: 3 of 35 entries carry one, all three
correct**, verified twice: derived from the `####` heading, and against the book's
own "List of Flaws" index links, which spell them out literally.

| id | anchor | heading | index link |
|---|---|---|---|
| `flaw.devoted_parent` | `devoted-parentchild` | ArMDE:5950 `#### Devoted Parent/Child` | ArMDE:5576 |
| `flaw.disfigured` | `disfigured` | ArMDE:5980 | ArMDE:5578 |
| `flaw.environmental_sensitivity` | `environmental-sensitivity` | ArMDE:6024 | ArMDE:5579 |

The other 32 carry the key set `["file","lines"]` only — including
`flaw.evil_eye`, the batch's fourth `uncomputed_rule`, whose three siblings all
have one. Not a defect of any entry (B11's Q-96 is the catalogue-wide question),
but worth the line: the anchor that *would* be correct for it is `evil-eye`,
spelled out at ArMDE:5541.

**Checks 3, 4, 5, 6 (`kind` / `magnitude` / `entity_kinds` / `categories` +
`tainted`): 35 of 35 correct**, each read against its own descriptor line one by
one:

- `*Major, Story*` — dependent (5933), diabolic_past (5959), difficult_underlings (5977), enemies (6005), envied_beauty (6013), evil_destiny (6029), excommunicate (6045)
- `*Minor, Story*` — employed_by_company (6001), faerie_friend (6053)
- `*Major or Minor, Personality*` — depraved_\* (5937), driven_\* (5989), envious_\* (6017)
- `*Minor, Personality*` — depressed (5941), dutybound (5993), faerie_upbringing (6057)
- `*Minor, Supernatural*` — deteriorating_power (5945), evil_eye (6037), exiled_atlantean (6049)
- `*Minor, General*` — devoted_parent (5951), dhimmi (5955), disfigured (5981), environmental_sensitivity (6025)
- `*Major, General*` — dwarf (5997), enfeebled (6009)
- `*Major, Hermetic*` — difficult_longevity_ritual (5963), environmental_magic_condition (6021)
- `*Minor, Hermetic*` — difficult_spontaneous_magic (5967), disjointed_magic (5973), disorientating_magic (5985), exciting_experimentation (6041)
- `*Minor, Social Status*` — failed_journeyman (6061), failed_master (6065)

`kind` is `flaw` on all 35 ✓ — the whole span is inside the book's Flaws block.
`entity_kinds` is `["character"]` on all 35 ✓ (`types.rs::EntityKind` has exactly
`Character` and `Covenant`; none of these is a covenant Hook). **`tainted` is
`false` on all 35 and that is right**: no descriptor line in this span carries the
`Tainted` tag. Three entries invite the mistake and none should get it —
`flaw.diabolic_past` (diabolists, Infernal Lore), `flaw.evil_destiny` ("The
Infernal realm has taken an interest in the character"), and
`flaw.exiled_atlantean`, whose own prose says "she is **tainted** by her contact
with the surface world" while its descriptor says only `*Minor, Supernatural*`.
Following the descriptor rather than the prose is correct (`tainted` means
*Infernal-realm associated*, ArMDE:2998-3002; the Atlantean's taint is the
surface world's).

**One descriptor line is malformed in the English source and correctly ignored.**
ArMDE:5973 reads `*Minor. Hermetic*` — a full stop where every other descriptor
in the book has a comma. DE 5973 has the comma (`*Klein, Hermetisch*`). The data
reads it correctly as Minor + Hermetic, corroborated by the index
(ArMDE:5433 under `### Hermetic, Minor`). A source typo, not a data defect; noted
so nobody "fixes" the data to match it.

**The index confirms every filing, with no contradiction.** All 32 headings are
filed in the book's own "List of Flaws" (ArMDE:5283-5638) under a
`### <category>, <magnitude>` heading matching both the descriptor and the data:

| index lines | heading | entries |
|---|---|---|
| 5291-5292 | `### Hermetic, Major` (5285) | Difficult Longevity Ritual, Environmental Magic Condition |
| 5317-5319 | `### Personality, Major or Minor` (5310) | Depraved, Driven, Envious |
| 5349-5355 | `### Story, Major` (5340) | Dependent, Diabolic Past, Difficult Underlings, Enemies, Envied Beauty, Evil Destiny, Excommunicate |
| 5406-5407 | `### General, Major` (5401) | Dwarf, Enfeebled |
| 5432-5435 | `### Hermetic, Minor` (5417) | Difficult Spontaneous Magic, Disjointed Magic, Disorientating Magic, Exciting Experimentation |
| 5475-5477 | `### Personality, Minor` (5467) | Depressed, Dutybound, Faerie Upbringing |
| 5508-5509 | `### Story, Minor` (5501) | Employed by Company, Faerie Friend |
| 5524-5525 | `### Social Status, Minor` (5520) | Failed Journeyman, Failed Master |
| 5540-5542 | `### Supernatural, Minor` (5534) | Deteriorating Power, Evil Eye, Exiled Atlantean |
| 5576-5579 | `### General, Minor` (5564) | Devoted Parent/Child, Dhimmi, Disfigured, Environmental Sensitivity |

No entry in this batch is dual-listed, so there is no `taken_as` case here.

**Check 12, mechanically.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the 35 ids in each locale returns the
key set `["name","summary"]` for **31** entries and
`["description","name","summary"]` for **4** — the same 4 in both locales, so the
locales never disagree about *whether* a description exists. The 4 are exactly the
batch's `uncomputed_rule` entries (`devoted_parent`, `disfigured`,
`environmental_sensitivity`, `evil_eye`), each carrying its full cited passage in
both languages ✓. **No `narrative`, `creation_effect` or `in_play_effect` entry in
this batch carries a description in either locale** — which is where every D5
finding below comes from.

**ASCII hyphen (check 12, second half): clean, verified two ways.**
`grep -c "−"` (U+2212) over `rules/i18n/en/virtues_flaws.json` and
`rules/i18n/de/virtues_flaws.json` returns **0** for both, catalogue-wide. And
`grep -o "–[0-9]"` (U+2013 followed by a digit) returns **nothing** in either
file, so no en dash that does ship is a *sign*. The conversion is working in both
directions: the DE source writes every negative as `–` (DE 5952 `–1`, 5982 `–3`,
5998 `–2`, 6026 `–3`) and every one ships as ASCII `-`; and the **English** source
is itself inconsistent — ArMDE:5952/5982/5998 use ASCII while ArMDE:6026 uses an
en dash (`a –3 penalty to his Stamina`) — yet the shipped EN description carries
ASCII `-3`. A correction pass adding the missing descriptions must keep doing
this.

**German names against the canonical tables.** Eighteen of the thirty-two
distinct English names in this batch have a row in
`rules/source/de/translation-tables/`, located by English key. **Eleven agree with
the shipped `rules/i18n/de/` name and with the DE rulebook heading**; **five
disagree with the rulebook**, in each case because the shipped data correctly
follows the rulebook and the table's copy here does not (D6 — see **F-425**); and
two are corroboration from a non-conflicting angle. The remaining fourteen
English names have no table row and were checked directly against the DE rulebook
heading on the parallel line — **thirteen match ✓, one does not**
(`flaw.dutybound`, **F-419**).

| English | table row | DE rulebook heading | shipped DE name | verdict |
|---|---|---|---|---|
| Dependent | `tugenden-fehler.md` *Schützling* | 5932 Schützling | Schützling | ✓ |
| Depraved | `tugenden-fehler.md:13` note *Verdorben* (and `README.md:76`) | 5936 Verdorben | Verdorben (Groß/Klein) | ✓ (CLAUDE.md's own named case) |
| Depressed | `tugenden-fehler.md:472` *Deprimiert* (tagged GotF) | 5940 Depressiv | Depressiv | data ✓; table row is a different book's term — F-425 |
| Deteriorating Power | `tugenden-fehler.md:340` *Schwindende Kraft* | 5944 Schwindende Macht | Schwindende Macht | data ✓; **table disagrees** — F-425 |
| Diabolic Past | `sphären-mächte.md:312` *Diabolische Vergangenheit* | 5958 Diabolische Vergangenheit | Diabolische Vergangenheit | ✓ |
| Difficult Longevity Ritual | `tugenden-fehler.md:293` *Schwieriges Langlebigkeitsritual* | 5962 idem | idem | ✓ |
| Difficult Spontaneous Magic | `tugenden-fehler.md:270` *Schwierige Spontane Magie* | 5966 idem | idem | ✓ |
| Disorientating Magic | `tugenden-fehler.md:294` *Desorientierungsmagie* | 5984 Desorientierende Magie | Desorientierende Magie | data ✓; **table disagrees** — F-425 |
| Driven | `tugenden-fehler.md:367`, `persoenlichkeitseigenschaften.md:68` *Getrieben* | 5988 Getrieben | Getrieben (Groß/Klein) | ✓ |
| Enemies | `tugenden-fehler.md:398` *Feinde* | 6004 Feinde | Feinde | ✓ |
| Enfeebled | `tugenden-fehler.md:368` *Entkräftet* | 6008 Geschwächt | Geschwächt | data ✓; **table disagrees** — F-425 |
| Envious | `tugenden-fehler.md:369`, `persoenlichkeitseigenschaften.md:71` *Neidisch* | 6016 Neidisch | Neidisch (Groß/Klein) | ✓ |
| Environmental Magic Condition | `tugenden-fehler.md:267`, `:341` *Magische Umgebungsbedingung* | 6020 Magische Umweltbedingung | Magische Umweltbedingung | data ✓; **table disagrees, twice** — F-425 |
| Environmental Sensitivity | `tugenden-fehler.md:342` *Umgebungsempfindlichkeit* | 6024 Umweltempfindlichkeit | Umweltempfindlichkeit | data ✓; **table disagrees** — F-425 |
| Evil Eye | `tugenden-fehler.md:802` *Böser Blick* | 6036 Böser Blick | Böser Blick | ✓ |
| Excommunicate | `tugenden-fehler.md:757` *Exkommuniziert*; `reputationen.md:103` *Excommunicated \| Exkommuniziert \| Kirchlich \| 3 (–)* | 6044 Exkommuniziert | Exkommuniziert | ✓ — **and `reputationen.md:103` independently records the Church Reputation at 3 that the catalogue data omits** (F-408) |
| Exiled Atlantean | `tugenden-fehler.md:343` *Verbannter Atlantier* | 6048 Verbannter Atlantier | Verbannter Atlantier | ✓ |
| Dutybound | *(no row)* | 5992 **Pflichtbewusst** | **Pflichtgebunden** | **✗ — F-419** |

The thirteen table-less names that match their DE heading: Devoted Parent/Child →
*Hingebungsvolles Elternteil/Kind*, Dhimmi → *Dhimmi*, Difficult Underlings →
*Schwierige Untergebene*, Disfigured → *Entstellt*, Disjointed Magic →
*Zusammenhanglose Magie*, Dwarf → *Zwerg*, Employed by Company → *Angestellt bei
einem Unternehmen*, Envied Beauty → *Beneidete Schönheit*, Evil Destiny → *Böses
Schicksal*, Exciting Experimentation → *Aufregendes Experimentieren*, Faerie
Friend → *Feenfreund*, Faerie Upbringing → *Feenaufwuchs*, Failed Journeyman →
*Gescheiterter Geselle*, Failed Master → *Gescheiterter Meister*.

**One German-source inconsistency inside an entry.** `flaw.deteriorating_power`'s
DE heading at 5944 is *Schwindende Macht* but its own body at DE 5946 calls it
*"Nachlassender Macht"*. The data follows the heading, which is right. Recorded
as the caution it is for whoever writes F-412's German description: do not copy DE
5946's term. (This is the same shape as B11's F-405 and F-406 German-source slips,
making it the third in two batches.)

**`rules/i18n/*/abilities.json` was consulted** for the two Abilities this
batch's passages name by title: `ability.infernal_lore` and `ability.faerie_lore`,
both category `arcane`. Both resolve; no name mismatch. Their category is the
whole of F-409.

### Part C systemic gaps are not re-reported per entry

Five of Part C's rows touch this batch and none is counted against an entry:

- **`SpecialCasting::Circumstantial` is surfaced-only** (C1) — that is
  `flaw.disjointed_magic` and `flaw.environmental_magic_condition`, and
  `RULES.md:4869` and `:4891-4894` record the choice deliberately for the first.
  What *is* counted is D5 (F-421) and, separately, a gap Part C does **not**
  cover: the surfaced row carries no source item, so the two are indistinguishable
  in the read-out (**F-423**).
- **`HalvableTotal` has four members and none is "all magic totals"** (A33) — so
  `flaw.environmental_magic_condition` genuinely cannot be a
  `magic_total_halving`, exactly as B11 established for
  `flaw.deleterious_circumstances`. Not a data defect.
- **`HealthTrack::CastingFatigue` is surfaced-only** (C1) — relevant to
  `flaw.enfeebled`'s doubled casting Fatigue (F-410). The gap is the engine's; what
  is counted against the entry is `narrative` and the missing text, not the
  engine's silence. The *sign* inversion on the track's three shipped carriers is a
  prior batch's finding and none of the three is in this span.
- **`grants_reputation`'s `score` is not enforced** (C2) — touches
  `flaw.failed_journeyman` and `flaw.failed_master`. Both are correctly authored
  against an engine that counts kinds and slots; not counted against either.
- **`creation_effect` is not required to carry effects** (C7) — no entry in this
  batch is one of the five, so this row is inert here. Recorded because two of my
  reclassification recommendations (F-408, F-409) move entries *into*
  `creation_effect`, and C7 is why the guard will not notice if the accompanying
  effect is forgotten.

### Decisions applied

`docs/vf-audit/decisions.md` was read in full and is binding. It carried D1-D6
when this batch read it.

- **D3** is the load-bearing one again and governs five entries:
  `flaw.difficult_spontaneous_magic` (`HalvableTotal` has no member for "removes
  the *non*-fatiguing option" — confirmed against `types.rs::HalvableTotal` and
  `derived/casting.rs`), `flaw.enfeebled` (no effect can forbid an Ability
  category, and `HealthTrack` is additive where the book says *double*),
  `flaw.envied_beauty` (`Prereq` has `AbilityMin` and `ArtMin` but **no**
  characteristic-minimum variant), `flaw.difficult_underlings` (no engine state
  for "has underlings"), and `flaw.deteriorating_power` /
  `virtue.variable_power` (no effect expresses a magnitude reduction scaling with
  age or Warping). In every case D3 forbids `narrative` and requires the rule in
  the description.
- **D5** produces six of the eighteen findings and is the reason three entries
  that compute correctly still fail: `flaw.difficult_longevity_ritual` (F-420),
  `flaw.disjointed_magic` (F-421) and `flaw.dwarf` (F-422) each carry the right
  effects and drop an uncomputed clause into neither locale.
- **D6** governs **F-425**'s five rows. Per the brief `arm-de-translation` is
  outside this repository, so each is stated as *"this repository's copy disagrees
  with the rulebook"* and the error-vs-stale-copy determination is left open.
- **D1 and D4** touch nothing here: no entry in this batch carries
  `lab_total_mod`. `flaw.difficult_longevity_ritual` carries a
  `magic_total_halving { lab_longevity }`, which is a different variant and is not
  one of D1/D4's nine.
- **D2** touches nothing here: no entry is a granted Virtue and none carries
  `characteristic_score_delta_param`. (`flaw.dwarf` carries the *unparameterized*
  `characteristic_score_delta`, which `validation/mod.rs::effect_target`
  classifies as `Other`, so the precondition validator D2 is about never sees it —
  correctly, since Dwarf does not require Str to be at −3 first.)

### Cross-references followed and rated

Every pointer in the span was followed and read, including on entries that
passed.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the entry? |
|---|---|---|---|
| `flaw.deteriorating_power` | "Like the Variable Powers Virtue" (ArMDE:5946) / DE "Ähnlich wie die Tugend Variable Mächte" | `virtue.variable_power`, **ArMDE:5199-5206** (the book's heading is singular, the pointer plural) | **YES, and the target is itself defective.** ArMDE:5203 gives the shared formula — "a single power can have its effect level increased by **five**, or include another effect of the same level … **for each level of variance**", worked as "**five magnitudes after he has lived for 50 years**" — and ArMDE:5205 adds "**but it only applies once to a single power**", which is the repeat rule Deteriorating Power states without the qualifier. The target is `narrative`, states a formula, and sits **outside every `SWEPT_BLOCKS` range** → **F-413**. The target also carries the `power` text parameter Deteriorating Power lacks → **F-412**. |
| `flaw.dhimmi` | "described in the sourcebooks for these areas … see *Realms of Power: The Divine Revised Edition*, *The Cradle and the Crescent*, *Between Sand and Sea*, *Lands of the Nile*" (ArMDE:5956) / DE link `…Das Göttliche (Überarbeitet).md#dhimmi` | `rules/source/en/Ars Magica 5e - Realms of Power - The Divine (Revised).md`, **RoP:D:5288** (the dhimmi social restrictions) and **RoP:D:5784** (the Flaw, worded identically to the core entry). Three of the four named books are not in `rules/source/en/`. | **No rule attributable to the entry's *mechanics*.** RoP:D:5288 lists the social restrictions in prose (no arms, no saddled mounts, restricted court testimony) with no roll, no modifier and no number; RoP:D:5784 is the same text as ArMDE:5956. **One clause does state a rule and it points the other way**: RoP:D:5671 — "Almost all non-Muslim characters living under Muslim rule **must also take** the Minor General Flaw *Dhimmi* … Hermetic magi are the most obvious examples of such exceptions." That is a rule about *when the Flaw is compulsory*, in a supplement, not about what it does. Escalated as **Q-99** rather than rated against `flaw.dhimmi`. |
| `flaw.exiled_atlantean` | "More details about the Atlanteans can be found in *Realms of Power: Magic*, page 90" (ArMDE:6050) / DE link `…Sphären der Macht - Magie.md#atlantier` | **RoP:M:2649-2653** (the same Flaw, word for word) and **RoP:M:5570-5618** (`### Atlanteans`, the creature write-up) | **No.** RoP:M:2649-2653 is verbatim identical to ArMDE:6049-6050 and adds nothing. RoP:M:5584's "**Inherited Flaws:** Exiled Atlantean, Personality Flaw associated with her exile (for example Depressed)" is NPC-construction guidance for an Atlantean creature, not a rule of the Flaw. Corroboration ✓ — and it confirms the core entry is complete, so `narrative` survives. |
| `flaw.environmental_sensitivity` | "debilitation checks (page 406)" (ArMDE:6026) / DE `[Seite 406](#krankheiten)` | `### Debilitation`, **ArMDE:17244-17248** | **No new rule attributable.** ArMDE:17248 defines the check the −3 lands on — "must make a **Stamina check against an Ease Factor** set by the cause … If it fails, the character takes a wound" — which is a general rule the Flaw modifies rather than one it states. The entry's own `description` already carries the −3 and the pointer in both locales ✓. Corroboration that `uncomputed_rule` is right: the engine models no debilitation check, so the −3 can only be text. |
| `flaw.dwarf` | "the severity of wounds you take increases in three point increments rather than five point increments (**see page 404**)" (ArMDE:5998) | the wound-band rules; the engine's implementation is `derived/combat.rs::wound_ranges` | **YES — and the engine already satisfies it exactly.** `wound_ranges` computes `u = (size + 5).max(1)`; with `size_delta −2` that is `u = 3`, so every band steps in threes. The book's sentence is a *consequence* of Size −2, not a second rule, and the data encodes the cause rather than the consequence — which is right. Corroboration ✓. |
| `flaw.dwarf` | "You cannot take this Flaw and Giant Blood (page 83), Large (page 89), or Small Frame (page 145)" (ArMDE:5998) | `virtue.giant_blood` ArMDE:3977, `virtue.large` ArMDE:4231, `flaw.small_frame` ArMDE:6769 | **YES — three stated incompatibilities, and all three are encoded**, on both sides. `flaw.dwarf` carries `incompatible_with: ["flaw.small_frame","virtue.giant_blood","virtue.large"]`, and each of the three targets names Dwarf back in its own passage (ArMDE:3977, :4231, :6769). Symmetry is load-enforced (`ruleset/integrity.rs::validate_incompatibility_symmetry`). **This is the first fully-encoded stated pair rule the audit has found since B10 opened the question** — six stated, one encoded across B10+B11; this makes it nine stated, four encoded. Corroboration ✓, and it is the counter-example that shows the omissions elsewhere are omissions rather than policy. |
| `flaw.difficult_spontaneous_magic` | "This Flaw may be combined with Weak Spontaneous Magic (**page 153**)" (ArMDE:5970) / DE `[Seite 153](#schwache-spontane-magie)` | `flaw.weak_spontaneous_magic` **ArMDE:7084-7089** | **YES, in two directions.** (a) It is an explicit *compatibility* statement, restated from the other end at ArMDE:7088, so both entries' empty `incompatible_with` is **correct for that pair** — B11's F-385 established this and it re-derives. (b) Reading the target is what settles F-411: ArMDE:7086 says Weak Spontaneous Magic means "**You may not exert yourself** when casting spontaneous magic, **so you always divide your Casting Score by five**", which pins "exerting yourself" to the ÷2 fatiguing option and makes Difficult Spontaneous Magic its exact mirror. |
| `flaw.difficult_spontaneous_magic` | *(inbound)* ArMDE:5783 "This Flaw is not compatible with Difficult Spontaneous Magic or Weak Spontaneous Magic" | `flaw.ceremonial_spontaneous_magic` ArMDE:5781-5784 (B11's span) | **YES — B11's F-385 re-derived from this end and still live.** `jq` returns no `incompatible_with` key on `flaw.difficult_spontaneous_magic`; the edge to Ceremonial is missing on this side too. Folded into **F-411** rather than given a new number, since F-385 owns it. |
| `flaw.exciting_experimentation` | "any of the experimentation tables" (ArMDE:6042) / DE "einer der Experimentiertabellen" | `### Extraordinary Results`, **ArMDE:11028-11040** | **YES — it is what makes the Flaw's sentence a rule.** ArMDE:11028: "you must also **roll a stress die on the 'Extraordinary Results Chart'** for each season that the project involves"; ArMDE:11038 adds the botch handling. So "roll two dice instead of the normal one" replaces a specified stress-die roll with two, and the storyguide picks — a real, numbered procedure. Feeds **F-414**. |
| `flaw.environmental_magic_condition` | "significantly more restrictive than the Hermetic Flaw Deleterious Circumstances" (ArMDE:6022) | `flaw.deleterious_circumstances` ArMDE:5917-5920 (B11's span) | **A guideline, not a computable rule** — it constrains the *player's choice* of condition, and the engine records no condition at all for either Flaw. Rated as no added mechanic, but it is the sentence that makes **F-423** matter: the two Flaws differ in magnitude (Major vs Minor) and in breadth ("common" vs "uncommon" conditions) and ship byte-identical effects that render as byte-identical read-out rows. |
| `flaw.evil_eye` | "including the effect of the Aegis of the Hearth" (ArMDE:6038) | the Aegis ritual | **No.** The clause names a source of Magic Resistance as an example of the ≥0 threshold it already states; the threshold itself is in the entry's `description` in both locales ✓. |
| `flaw.enfeebled` | "unable to learn **Martial Abilities**" (ArMDE:6010) | `rules/core/abilities.json` — `martial` is one of the three `categories_requiring_virtue` | **YES, and the direction is the trap.** The gate is a *permission* gate, so a Flaw that *forbids* the category has nothing to hang on: `Effect::AbilityAuthorization` only ever adds to the permitted set, and no variant subtracts from it. The type profiles' `forbidden_categories` is the only forbidding shape and it lives on the character *type*, not on a V/F. D3 → `uncomputed_rule`. Feeds **F-410**. |
| `flaw.evil_destiny` | "such as **Corrupted Abilities**. The character could also simply be **Plagued by Demons** or **Susceptible to the Infernal**" (ArMDE:6034) | `flaw.corrupted_abilities` ArMDE:5847-5852, and the two named Flaws | **No.** Unlike B11's F-406 idiom ("*includes the effects of*"), this passage says the character "**might have** other Flaws … but not (yet) know what they are" — it is an invitation to the storyguide, not a composition. Nothing is imported, nothing is granted, and no catalogue precedent treats "might have" as mechanical. `narrative` survives ✓. |
| `flaw.dependent` | "you should substitute another Story Flaw … taking the killers of the Dependent as **Enemies**, or taking the Dependent as a **True Friend**" (ArMDE:5934) | `flaw.enemies` ArMDE:6004-6007, `virtue.true_friend` | **No.** A "should" about what to do *after* the Flaw stops fitting, in play, with three offered alternatives — advice, not a rule, and it grants nothing at creation. `narrative` survives ✓. Also an **ArMDE:2818** instance (one Story Flaw). |
| `flaw.diabolic_past` / `flaw.faerie_friend` / `flaw.faerie_upbringing` | "Infernal Lore" / "Arcane Knowledge Faerie Lore" / "Faerie Lore" | `ability.infernal_lore` and `ability.faerie_lore`, both **category `arcane`**; `rules/core/abilities.json`'s `categories_requiring_virtue` is `["academic","arcane","martial"]` | **YES, on all three — this is F-409.** Each names a gated Ability and grants permission to buy it; none carries `ability_authorization` or `restricted_ability_xp`, so `validation/authorization.rs::validate_ability_authorization` raises a hard `ability_category_requires_virtue` on any non-magus who obeys the Flaw. |

### ArMDE:2814, :2816, :2818, :2820 — instances in this batch

Noted, not re-derived (B10 established the enforcement state of all four).

- **ArMDE:2814** ("more than once only if the description explicitly allows it") —
  exactly **one** entry in this batch says it may be taken more than once:
  `flaw.deteriorating_power` (ArMDE:5948, "This Flaw may be taken more than once,
  if the character has more than one Power"). It carries `max_per_target: 255`,
  which `RULES.md:1501-1530` records as the catalogue's "no stated ceiling"
  encoding and lists this entry in by name ✓. **No other entry in this batch is
  parameterized and none carries a `max_total`**, so `max_per_target`'s default of
  1 — keyed on `(item_ref, {})` — already forbids a second copy of each of the
  other 34, which is what the book requires. What the 255 does *not* enforce is
  the entry's own qualifier ("if the character has more than one Power"), and
  that is **F-412** / **Q-101**.
- **ArMDE:2816** ("All characters must take one Social Status, and may only take
  more than one if…") — two instances, `flaw.failed_journeyman` and
  `flaw.failed_master`. Neither passage states a compatibility, and the two are
  plainly mutually exclusive in fiction (one is expelled from a guild, the other
  has run his own workshop into the ground and must now work as a journeyman).
  **This row is not "noted, not re-derived" like its three neighbours**: my first
  pass wrote that the one-Social-Status rule is a profile-level cap B10 confirmed
  is enforced, the verification sub-agent challenged it, and **the sub-agent is
  right — neither half of ArMDE:2816 is enforced anywhere.** See **F-427**.
- **ArMDE:2818** ("not more than one Story Flaw") — nine instances, the most of
  any batch so far: `dependent`, `diabolic_past`, `difficult_underlings`,
  `employed_by_company`, `enemies`, `envied_beauty`, `evil_destiny`,
  `excommunicate`, `faerie_friend`. Enforced on all four profiles per B10 ✓.
- **ArMDE:2820** (the Major/Minor Personality split) — three pairs here
  (`depraved`, `driven`, `envious`), each with a mutual `incompatible_with` ✓,
  additionally load-enforced by
  `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`, and feeding the
  real `too_many_major_personality_flaws` cap B11 confirmed is live.
- **ArMDE:2960-2962** (Supernatural realm association) — three instances:
  `flaw.deteriorating_power`, `flaw.evil_eye`, `flaw.exiled_atlantean`. None
  carries a `realm`-domain parameter. Noted only; it is B11's standing observation,
  not a new one.

## Verdicts

35 rows, one per entry. `class` / `data` / `text` report checks 2, 3-11 and 12.
"OK" means every check in that column passed; `?` means escalated, not resolved.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.dependent` | 5932-5935 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.depraved_major` | 5936-5939 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.depraved_minor` | 5936-5939 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.depressed` | 5940-5943 | narrative ✓ | OK | OK | **clean**; F-425 (table row) |
| `flaw.deteriorating_power` | 5944-5949 | narrative → uncomputed_rule | **no `power` parameter, where its twin `virtue.variable_power` has one**; `max_per_target: 255` therefore permits unlimited copies on one Power | the whole formula — the three linkage options and the worked example — in neither locale | **F-412** **Q-101**; ArMDE:2814 + :2960 instance |
| `flaw.devoted_parent` | 5950-5953 | uncomputed_rule ✓ | OK | OK ✓ — description carries the −1 verbatim in both locales, ASCII hyphen ✓ | **clean** |
| `flaw.dhimmi` | 5954-5957 | narrative ✓ **?** | OK | OK | **clean**; **Q-99** (RoP:D:5671) |
| `flaw.diabolic_past` | 5958-5961 | narrative → creation_effect | **no `ability_authorization` for `ability.infernal_lore`: obeying the Flaw is a hard validator error today** | the permission clause in neither locale | **F-409**; ArMDE:2818 instance |
| `flaw.difficult_longevity_ritual` | 5962-5965 | in_play_effect ✓ | `magic_total_halving { lab_longevity }` correct ✓ | the "anyone creating one for you" scope and the "for others without penalty" carve-out in neither locale | F-420 **Q-97** |
| `flaw.difficult_spontaneous_magic` | 5966-5971 | narrative → uncomputed_rule | **the stated incompatibility with Ceremonial Spontaneous Magic is encoded on neither side** (B11 F-385, re-derived) | the removal of the non-fatiguing option in neither locale | **F-411** |
| `flaw.disjointed_magic` | 5972-5975 | in_play_effect ✓ | `circumstantial` is a **recorded** choice (`RULES.md:4891-4894`) ✓ | both clauses — the similar-spell bonus and the invested-Art bonus — in neither locale | F-421 F-423 |
| `flaw.difficult_underlings` | 5976-5979 | narrative → uncomputed_rule | the taking-restriction is inexpressible (D3) | the restriction **does** survive — it is sentence one, so the summary carries it ✓ | **F-417**; ArMDE:2818 instance |
| `flaw.disfigured` | 5980-5983 | uncomputed_rule ✓ | OK | OK ✓ | **clean** |
| `flaw.disorientating_magic` | 5984-5987 | narrative → uncomputed_rule | OK | OK ✓ — the summary **is** the rule sentence, in both locales | **F-418** (class only) |
| `flaw.driven_major` | 5988-5991 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.driven_minor` | 5988-5991 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.dutybound` | 5992-5995 | narrative ✓ | OK | **the DE name is `Pflichtgebunden`; the DE rulebook heading at 5992 is `Pflichtbewusst`** | **F-419** |
| `flaw.dwarf` | 5996-5999 | creation_effect ✓ | all three effects correct in sign and size ✓; **all three stated incompatibilities encoded on both sides** ✓ | the two-thirds walking speed and the −6 Characteristic floor in neither locale | F-422 |
| `flaw.employed_by_company` | 6000-6003 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.enemies` | 6004-6007 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.enfeebled` | 6008-6011 | narrative → uncomputed_rule | the Martial/physical bar and the doubled casting Fatigue are both inexpressible (D3) | both clauses in neither locale | **F-410** **Q-98** |
| `flaw.envied_beauty` | 6012-6015 | narrative → uncomputed_rule | **a Characteristic precondition `Prereq` cannot express** (D3); and the passage's "its penalties" names penalties it never states | the precondition in neither locale | **F-416** |
| `flaw.envious_major` | 6016-6019 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.envious_minor` | 6016-6019 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.environmental_magic_condition` | 6020-6023 | in_play_effect ✓ | `circumstantial` correct (A33: no `HalvableTotal` fits) ✓; the condition is recorded by no parameter **?** | the halving **does** survive — it is sentence one, so the summary carries it in both locales ✓ | F-423; Q-88 (B11) recurs |
| `flaw.environmental_sensitivity` | 6024-6027 | uncomputed_rule ✓ | OK | OK ✓ — EN source uses an en dash at :6026, the shipped text ASCII ✓ | **clean**; F-425 (table row) |
| `flaw.evil_destiny` | 6028-6035 | narrative ✓ | OK | OK | **clean**; ArMDE:2818 instance |
| `flaw.evil_eye` | 6036-6039 | uncomputed_rule ✓ | OK | OK ✓ | **clean**; ArMDE:2960 instance |
| `flaw.exciting_experimentation` | 6040-6043 | narrative → uncomputed_rule | OK | **the summary is truncated mid-sentence at the book's ellipsis, in both locales** | **F-414** **F-415** |
| `flaw.excommunicate` | 6044-6047 | narrative → creation_effect | **no `grants_reputation { ecclesiastical, 3 }`, where `flaw.apostate` and `flaw.failed_monk` both carry one** | the Reputation and the sacraments bar in neither locale | **F-408**; ArMDE:2818 instance |
| `flaw.exiled_atlantean` | 6048-6051 | narrative ✓ | OK | OK | **clean**; ArMDE:2960 instance |
| `flaw.faerie_friend` | 6052-6055 | narrative → creation_effect | **no `ability_authorization` for `ability.faerie_lore`** | the permission clause in neither locale | **F-409**; ArMDE:2818 instance |
| `flaw.faerie_upbringing` | 6056-6059 | narrative → creation_effect | **no `ability_authorization` for `ability.faerie_lore`** | the permission clause in neither locale | **F-409** |
| `flaw.failed_journeyman` | 6060-6063 | creation_effect ✓ | `grants_reputation { local, 2 }` correct in kind and score ✓ | OK | **clean**; ArMDE:2816 instance |
| `flaw.failed_master` | 6064-6067 | creation_effect ✓ | `grants_reputation { local, 4 }` correct in kind and score ✓ | OK | **clean**; ArMDE:2816 instance |

**Fully clean: 20** — `dependent`, `depraved_major`, `depraved_minor`,
`depressed`, `devoted_parent`, `dhimmi`, `disfigured`, `driven_major`,
`driven_minor`, `employed_by_company`, `enemies`, `environmental_magic_condition`,
`envious_major`, `envious_minor`, `environmental_sensitivity`, `evil_destiny`,
`evil_eye`, `exiled_atlantean`, `failed_journeyman`, `failed_master`.
**With at least one failed or escalated check: 15.**

**The ten rows whose `class` column reads `narrative → …` are F-424's evidence**:
`deteriorating_power`, `diabolic_past`, `difficult_spontaneous_magic`,
`difficult_underlings`, `disorientating_magic`, `enfeebled`, `envied_beauty`,
`exciting_experimentation`, `excommunicate`, `faerie_friend` — plus
`faerie_upbringing`, making eleven. All eleven are inside `SWEPT_BLOCKS`'
`(ArMDE, 5639, 7113)` block and all eleven are green under the live guard today
(`cargo test -p arm-rules --test uncomputed_clauses` → 4 passed, run 2026-09-20).

## Findings

### F-408 — `flaw.excommunicate` — a stated Reputation the catalogue grants twice elsewhere and not here

> "For your crimes against God and the Church, you have been cast out of your
> faith, undoing your baptism and driving you from your religious community. **You
> have a bad reputation at level 3 within the Church**, and cannot benefit from the
> sacraments." — ArMDE:6046
>
> DE 6046: "Für deine Vergehen gegen Gott und die Kirche wurdest du aus deinem
> Glauben ausgestoßen, deine Taufe wurde rückgängig gemacht und du wurdest aus
> deiner Glaubensgemeinschaft vertrieben. **Du hast eine schlechte Reputation der
> Stufe 3 innerhalb der Kirche** und kannst nicht von den Sakramenten profitieren."

The entry is `classification: "narrative"` with no `effects`, no `parameters` and
no `description` in either locale. Its displayed text is the summary, which is the
passage's *first* sentence and stops exactly before the Reputation.

**The catalogue already encodes this clause, in this exact audience, twice.**
`types.rs::ReputationType` has four members (`Local`, `Ecclesiastical`,
`Hermetic`, `Academic`), and `jq` over `rules/core/virtues_flaws.json` for the
`grants_reputation` carriers returns:

| entry | ArMDE | passage clause | data |
|---|---|---|---|
| `flaw.apostate` | 5675-5678 | "You have a **bad Reputation at 4** among members of your **previous faith**" | `creation_effect`, `grants_reputation { ecclesiastical, 4 }` |
| `flaw.failed_monk` | 6068-6071 | "a **poor Reputation at level 2** in the **local area and within the Church**" | `creation_effect`, `grants_reputation { local, 2 }` + `{ ecclesiastical, 2 }` |
| **`flaw.excommunicate`** | **6044-6047** | **"a bad reputation at level 3 within the Church"** | **`narrative`, no effects** |

`flaw.failed_monk` is the decisive comparison: it is the entry *immediately after*
this batch's last one, it states the same clause with the same two-word audience
("within the Church"), and it carries the grant. `RULES.md:5372-5401`'s
"**Reputation grant**" list names thirty entries, `flaw.apostate` and
`flaw.failed_monk` among them, and **does not name `flaw.excommunicate`** — so no
rationale for the omission is recorded anywhere.

**A third, independent source agrees.**
`rules/source/de/translation-tables/reputationen.md:103` carries the row
`| Excommunicated | Exkommuniziert | Kirchlich | 3 (–) | Exkommunikation |` —
Ecclesiastical, level 3, negative. The glossary table records the Reputation the
catalogue data drops.

**Which checks fail.** Check 2 — `narrative` asserts the passage states nothing
mechanical, and a granted Reputation at a named level to a named audience is the
most concretely mechanical thing a Social/Story Flaw can say. Check 10, both
directions — a passage clause with no effect. Check 12 / D5 — the Reputation and
the sacraments bar reach the player in neither locale.

**What the correction is.** `classification: "creation_effect"` plus
`{"type":"grants_reputation","kind":"ecclesiastical","score":3}`. Note C7: the
`data_integrity.rs` guard does **not** assert that a `creation_effect` carries an
effect, so reclassifying without adding the effect would go green — the two must
land together. The second clause, "cannot benefit from the sacraments", is
inexpressible (nothing models sacraments) and is a D5 description obligation.

**Severity: high — this is a wrong number on the character sheet, not a
provenance defect.** `validation/scores.rs::validate_reputations` builds its slot
table from `grants_reputation` effects and emits `reputation_not_granted` (a hard
**error**) for any `Entity::reputations` row with no matching slot. So today an
Excommunicate character who enters the Church Reputation the book gives him is
*rejected* in Enforced mode, and one who does not enter it is missing a starting
Reputation the rules award. Both directions are wrong output.

### F-409 — three entries grant permission to buy a gated Ability and encode nothing, and the catalogue has the same hole in at least twenty-nine places

This is the batch's largest finding and the only one whose scope is
catalogue-wide. B11's F-407 found the first instance from outside its own span;
this batch contains three more, and sizing the pattern shows it is the rule rather
than the exception.

**The three in this span.**

> `flaw.diabolic_past`, ArMDE:5960 — "…Your former associates still take an
> interest in your activities and whereabouts. Unfortunately. **You may purchase
> the Ability Infernal Lore, even if you are normally not permitted to buy Arcane
> Abilities.**"
>
> DE 5960 — "**Du darfst die Fertigkeit Infernalkunde erwerben, selbst wenn du
> normalerweise keine Arkanen Fertigkeiten kaufen darfst.**"

> `flaw.faerie_friend`, ArMDE:6054 — "…**Characters with this Flaw can purchase
> the Arcane Knowledge Faerie Lore, even if they are normally restricted from
> purchasing it.**"
>
> DE 6054 — "**Charaktere mit diesem Fehler dürfen das Arkane Wissen Feenkunde
> erwerben, selbst wenn sie es normalerweise nicht dürfen.**"

> `flaw.faerie_upbringing`, ArMDE:6058 — "…However, you find human society,
> including religion, bizarre. **You may learn Faerie Lore at character
> generation.**"
>
> DE 6058 — "**Du darfst Feenkunde bei der Charaktererschaffung erlernen.**"

All three are `classification: "narrative"` with **no `effects` key at all** —
`jq` returns `id, kind, magnitude, categories, classification, entity_kinds,
source` for each.

**The chain that makes this a live bug, verified end to end.**

1. `rules/core/abilities.json` gives both `ability.infernal_lore` and
   `ability.faerie_lore` the category **`arcane`**.
2. The same file sets `"categories_requiring_virtue": ["academic","arcane","martial"]`.
3. `validation/authorization.rs::validate_ability_authorization` returns early
   only for `type_profile.is_some_and(|profile| profile.is_magus)`; for everyone
   else it walks `entity.ability_scores`, and for each held Ability whose category
   is gated and which is named by neither `authorized_abilities` nor
   `authorized_categories` it pushes
   `ValidationIssue::error(CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE, …)`.
4. `effective/xp.rs::ability_authorizations` builds that union from exactly two
   effect variants: `AbilityAuthorization` and `RestrictedAbilityXp`. None of the
   three entries carries either.

All three Flaws are Minor and none is Hermetic, so grogs and companions may take
them and the magus exemption is empty. **A companion with Faerie Friend who buys
Faerie Lore — the one thing the Flaw exists to permit — gets a hard
`ability_category_requires_virtue` error today.** This is B11's F-388 mechanism
exactly, but without F-388's difficulty: these three grant permission with no
point budget attached, so the honest encoding is the unambiguous one.

**What the correction is**, per entry:

| entry | effect | class |
|---|---|---|
| `flaw.diabolic_past` | `{"type":"ability_authorization","abilities":["ability.infernal_lore"]}` | `narrative` → `creation_effect` |
| `flaw.faerie_friend` | `{"type":"ability_authorization","abilities":["ability.faerie_lore"]}` | `narrative` → `creation_effect` |
| `flaw.faerie_upbringing` | `{"type":"ability_authorization","abilities":["ability.faerie_lore"]}` | `narrative` → `creation_effect` |

`creation_effect` rather than `uncomputed_rule`, on the same reasoning B11 gave
for F-407: the engine *can* express this, so D3 does not apply. The shape already
ships — `flaw.covenant_upbringing` carries
`ability_authorization: ["ability.dead_language"]` and
`virtue.student_of_realm` carries a four-Ability list including
`ability.faerie_lore` and `ability.infernal_lore`, which is the exact pair these
three need.

**And the hole is catalogue-wide.** Two facts, each stated no wider than I
checked.

*Fact one.* `jq` over `rules/core/virtues_flaws.json` for entries carrying an
`ability_authorization` effect returns **exactly two**:
`flaw.covenant_upbringing` and `virtue.student_of_realm`. That is the whole
population, out of 655.

*Fact two.* One `grep` of the English core rulebook for thirteen spellings of the
permission idiom (`even if you are normally not permitted`, `even if normally
unable`, `even if they are normally restricted`, `normally restricted from
purchasing`, `may take Academic Abilities`, `may take Martial Abilities`,
`may purchase Arcane`, `even if she is normally`, `even if he is normally`,
`may learn Faerie Lore`, `may purchase the Ability`, `at character creation, even`,
`May take Martial`) returns 29 passages. Resolving each to its catalogue entry by
`source.lines[0]`, **29 entries state a permission their data does not
authorize** — 23 of them carrying no `effects` at all:

| carrying no effects at all (23) |
|---|
| `virtue.almogaten` :3396 · `virtue.almogavar` :3404 · `virtue.archieunuch` :3436 · `virtue.bureaucrat` :3541 · `virtue.clerk` :3571 · `virtue.eunuch` :3771 · `virtue.familiarity_with_the_fae` :3857 *(B11 F-407)* · `virtue.fidai` :3877 · `virtue.folk_magic` :3907 · `virtue.knight` :4195 · `virtue.lasiq` :4233 · `virtue.mamluk` :4443 · `virtue.mazdean_priest` :4480 · `virtue.mendicant_friar` :4488 · `virtue.mercenary_captain` :4500 · `virtue.notary` :4598 · `virtue.perfectus` :4632 · `virtue.religious` :4856 · `virtue.templar_administrator` :5109 · `flaw.branded_criminal` :5749 · **`flaw.diabolic_past` :5958** · **`flaw.faerie_friend` :6052** · **`flaw.faerie_upbringing` :6056** |

| carrying effects that do not cover the permission (6) |
|---|
| `virtue.marshal` :4449 — `restricted_ability_xp` naming six Abilities (Animal Handling, Dead Language, Etiquette, Hunt, Profession, Ride); ArMDE:4453's separate "**may take Martial Abilities freely**" is covered by none of them |
| `virtue.master_of_kennels` :4467 — the identical six-Ability pool; ArMDE:4469 states the same separate Martial clause |
| `virtue.strong_faerie_blood` :5032 — `grants_selection` + `aging_mod`; ArMDE:5040 "**Third, you may learn Faerie Lore during character generation**" is covered by neither |
| `flaw.failed_monk` :6068 — two `grants_reputation`; ArMDE:6070 "**You may take Academic Abilities during character creation**" is covered by neither |
| `flaw.outlaw` :6542 — one `grants_reputation`; ArMDE:6544 "**You may take Martial Abilities at character generation**" |
| `flaw.outlaw_leader` :6546 — one `grants_reputation`; ArMDE:6548, same clause |

*Scope of this negative, stated exactly as checked:* the 29 come from those
thirteen grep patterns over `rules/source/en/Ars Magica - Definitive Edition
(Core Rules).md` only, and from a `jq` join on `source.lines[0]`. A fourteenth
spelling of the idiom would find more, and I make **no** claim that 29 is the
complete population — only that it is a floor, and that the population of encoded
ones is exactly two.

**The verification sub-agent re-derived this by a different method and reached a
compatible, narrower, read-verified result** — recorded here because two
independent censuses agreeing matters more than either alone. It grepped the four
gated-category proper nouns (`Academic Abilit`, `Arcane Abilit`, `Martial Abilit`,
`Arcane Knowledge`) rather than the idiom, got 103 hits file-wide and 72 inside
the V/F chapter, then **narrowed to the Flaws block (ArMDE:5639-7113) and read
every hit**, plus ArMDE:6058 separately because that sentence names no category at
all. Its verdict: **seven Flaws grant gated-Ability access and not one encodes
it** — `flaw.branded_criminal` :5751, **`flaw.diabolic_past` :5960**,
**`flaw.faerie_friend` :6054**, **`flaw.faerie_upbringing` :6058**,
`flaw.failed_monk` :6070, `flaw.outlaw` :6544, `flaw.outlaw_leader` :6548. All
seven are in my 29, so the two lists do not conflict; the sub-agent's is the
**read-verified** subset for one block and mine is the **grep-derived** floor for
the whole book. It also independently counted `ability_authorization` carriers at
exactly 2 and added a figure I had not taken: of the 28 `restricted_ability_xp`
carriers, **10** name a gated category and so carry the permission implicitly
(`virtue.arcane_lore`, `cathedral_school_master`, `doctor_in_faculty`,
`lone_redcap`, `magister_in_artibus`, `magister_in_medicina`,
`mentored_by_demons`, `privileged_upbringing`, `trained_assassin`, `warrior`).
So the fully-covered population of this rule in the catalogue is **12**, not 2 —
and the uncovered one is still at least 29.

**Severity: high**, on the same grounds B11 rated F-388 and F-407 high: it is a
false hard error that blocks a character the rulebook explicitly permits, it fires
in Enforced mode which is the default for both guided and direct-validated entry,
and for these three entries (unlike Church Upbringing) the fix over-permits
nothing. It is *not* a miscalculation — no number is wrong — but a validator that
rejects a legal build is a product-integrity failure of the same class.

### F-410 — `flaw.enfeebled` — an Ability-category prohibition and a doubled Fatigue cost, classified as flavour

> "You cannot exert yourself for longer than a few seconds. Any need for rapid
> movement, such as combat or a chase, leaves you helpless. Long hikes are likewise
> beyond your capability. **You are unable to learn Martial Abilities or any other
> skills involving physical exertion**, since you cannot train in them. **If you are
> a magus, you lose double the normal number of Fatigue levels from casting
> spells**, but you may carry out Laboratory activities as normal." — ArMDE:6010
>
> DE 6010: "…**Du bist nicht in der Lage, Kampffertigkeiten oder andere
> Fertigkeiten zu erlernen, die körperliche Anstrengung erfordern**, da du nicht in
> ihnen üben kannst. **Bist du ein Magus, verlierst du beim Wirken von Zaubern
> doppelt so viele Erschöpfungsstufen wie normal**, kannst aber Laborarbeiten ohne
> Einschränkung durchführen."

Two hard mechanics, one of them a **doubling of a resource cost**, and the entry
carries `classification: "narrative"`, no `effects`, and no `description` in
either locale. The displayed text is the summary — the passage's first sentence,
"You cannot exert yourself for longer than a few seconds." — which is the one
sentence of the five that states nothing.

**Both clauses are inexpressible, and that is D3's case, not `narrative`'s.**

*The category bar.* The gate the engine has is a *permission* gate and it only
ever widens: `effective/xp.rs::ability_authorizations` unions two effect variants
into a permitted set, and `validation/authorization.rs::validate_ability_authorization`
errors on a gated Ability **absent** from that set. Nothing subtracts. I checked
the one forbidding shape in the codebase and it is a different taxonomy:
`validation/selections.rs::validate_forbidden_categories` reads
`EntityTypeProfile::forbidden_categories`, which lists **Virtue/Flaw** categories
(`hermetic`, `social_status`, …) on a character *type*, not Ability categories on
a V/F. So no effect can forbid `martial`, and "or any other skills involving
physical exertion" is an open-ended category the catalogue does not model at all.

*The doubled Fatigue.* `HealthTrack::CastingFatigue` exists and is the right
track — `types.rs`'s own doc for it reads "Fatigue levels lost per spell cast
(Vulnerable Casting +1, Withstand Casting −1, Painful Magic). Surfaced-only." But
`Effect::HealthMod` carries an **additive** `amount: i8`, and the book says
*double*. A doubling cannot be written as a constant: for a magus who normally
loses one level it is +1, for a Fatiguing Spontaneous cast that already costs
several (ArMDE:4303) it is not. Authoring `+1` would be inventing a number the
book does not state, which is the `7f5605a` aging-floor mistake.

So: **`narrative` → `uncomputed_rule`**, with both clauses written into
`description` in both locales.

**One thing that must not be got wrong if anyone ever does wire it**, recorded
because the track's sign is a live defect elsewhere. The doc quoted above fixes
the convention — the track counts *levels lost*, so **more is worse and the
penalty sign is positive**. The three shipped carriers are
`flaw.painful_magic` `−1`, `flaw.vulnerable_casting` `−1` and
`virtue.withstand_casting` `+1`: all three are inverted against the doc, on both
the Flaw and the Virtue side. None of the three is in this batch's span, so this
is not a B12 finding — it is the prior batch's inversion, restated here only
because Enfeebled is the natural fourth carrier and would otherwise be wired to
match the wrong three.

**And the verification pass narrowed which side is wrong, which the brief's own
framing did not.** The brief describes `casting_fatigue` as "ship[ping] exactly
inverted on both its carriers, caught only by reading `types.rs`'s own doc for the
track" — i.e. the data is wrong. The sub-agent read the three passages and the
sibling track and made the opposite case defensible:

| id | data | doc says | rulebook |
|---|---|---|---|
| `virtue.withstand_casting` | **+1** | −1 | ArMDE:5263 — "he loses **1 less** Fatigue level than normal, with a minimum of 1 Fatigue level" |
| `flaw.vulnerable_casting` | **−1** | +1 | ArMDE:6995 — "she loses **1 more** Fatigue level than normal" |
| `flaw.painful_magic` | **−1** | *(unsigned)* | ArMDE:6576 — "the equivalent of **one** Fatigue level in pain for each spell you cast" |

I re-read all three lines and they say exactly that. **The rulebook states
directions, not signs** ("1 less", "1 more"), so it cannot adjudicate. What can is
the sibling: `HealthTrack::FatigueRoll`'s doc reads "Fatigue / Stamina rolls to
avoid fatigue (**Long-Winded +3, Obese/Short of Breath −3**)" — the *benefit*
convention, Virtue positive — which is the convention the `casting_fatigue`
**data** follows and the `casting_fatigue` **doc** contradicts. On that evidence
the three amounts may well be right and the doc comment the single outlier, which
is the reverse of the received reading. This is why the fourth carrier cannot be
wired until it is settled. See **Q-98**.

**Severity: moderate, as a lost rule.** Nothing is miscalculated (the engine
computes nothing here either way) and nothing is falsely rejected — the harm is
that a Major Flaw's two actual costs reach the player in neither language, so the
sheet says only that the character tires easily.

### F-411 — `flaw.difficult_spontaneous_magic` — the removal of non-fatiguing spontaneous casting, classified as flavour; and the B11 incompatibility re-derived from this end

> "Spontaneous magic is always an effort for you. **You cannot cast Spontaneous
> spells without exerting yourself.** However, when you do exert yourself, you cast
> spells as any other magus.
>
> This Flaw may be combined with Weak Spontaneous Magic (page 153) to create a
> magus who cannot use Spontaneous magic at all." — ArMDE:5968, :5970
>
> DE 5968: "Spontane Magie erfordert für dich stets Anstrengung. **Du kannst keine
> spontanen Zauber wirken, ohne dich dabei anzustrengen.** Wenn du dich jedoch
> anstrengst, wirkst du Zauber wie jeder andere Magus."

The entry is `narrative`. The middle sentence is a rule, and following the page
reference is what proves it rather than asserting it.

**"Exerting yourself" is a defined mechanic, and the book defines it in the
spontaneous-casting rules.** ArMDE:9141, `### Spontaneous Magic`: "Magi may choose
whether or not to **exert themselves** when casting Spontaneous magic, but this
affects the casting total. **If a maga exerts herself, she loses a Fatigue
level** immediately after the spell is cast"; ArMDE:9143 "**FATIGUING SPONTANEOUS
MAGIC CASTING TOTAL: (Casting Score + Stress Die)/2**"; ArMDE:9145
"**NON-FATIGUING SPONTANEOUS MAGIC CASTING TOTAL: Casting Score/5**". The mirror
Flaw restates it from the other end — ArMDE:7086, *Weak Spontaneous Magic*: "You
may not exert yourself when casting spontaneous magic, so you always divide your
Casting Score by five."

So `flaw.difficult_spontaneous_magic` removes the ÷5 option and Weak Spontaneous
Magic removes the ÷2 one. They are the two halves of one switch, which is exactly
why ArMDE:5970 and :7088 both say that taking both leaves a magus with no
spontaneous magic at all.

*(The ArMDE:9141-9145 citations are the verification sub-agent's; my first pass
inferred the definition of "exerting yourself" from ArMDE:7086 alone, which is the
mirror Flaw's restatement rather than the rule. The inference was right and the
citation was second-hand — a rule read off another entry is weaker evidence than
the rule itself, so the better citations replace it here.)*

**The engine models one half and cannot model the other with today's
vocabulary.** `derived/casting.rs::casting_totals` computes both figures side by
side — `spontaneous_non_fatiguing = spont_base.div_euclid(5)` and
`spontaneous_fatiguing` — and a `weak_spont` flag makes the *fatiguing* field
report the ÷5 figure instead. That flag comes from
`magic_total_halving { spontaneous_casting }`, and `types.rs::HalvableTotal` has
exactly four members — `SpontaneousCasting`, `LabEnchanting`, `LabLongevity`,
`Penetration` — with no member meaning "the non-fatiguing option is unavailable".
**D3 therefore applies today: the rule cannot be expressed, and inexpressible is
grounds for `uncomputed_rule`, never for `narrative`.**

**One narrowing I accept from the verification pass, because my first wording
overstated it.** I wrote that the engine "structurally cannot" model this. That is
too strong. `derived/casting.rs` handles Weak Spontaneous Magic **not by halving
anything** but by having `spontaneous_fatiguing` echo the non-fatiguing figure,
with a comment in the source saying `CastingScores` "has no 'this option does not
exist' representation" so the dead slot mirrors the live one. Difficult
Spontaneous Magic is the identical trick with the two slots swapped. So the honest
framing is **"one new `HalvableTotal` member away from computable"**, not
"structurally inexpressible". The verdict is unchanged — D3 governs what the
engine can express *today*, and today it cannot — but the correction slice should
know the cheap fix exists, and should also know the sub-agent's second
observation: `magic_total_halving { spontaneous_casting }` is already a misnomer,
since ArMDE:7086 says *divide by five*, not *halve*, and `casting.rs` special-cases
around the variant's own name.

**Verdict:** `narrative` → **`uncomputed_rule`**, with the restriction written
into `description` in both locales. Displayed text today is the summary
"Spontaneous magic is always an effort for you." alone, which states the flavour
and not the cost.

**And the incompatibility, re-derived from this side.** B11's **F-385** found that
ArMDE:5783 names two pair rules and none of the three sides encodes them. Reading
this entry from the other end confirms it is still true and narrows it: `jq`
returns **no `incompatible_with` key** on `flaw.difficult_spontaneous_magic`,
while ArMDE:5783 says "This Flaw is not compatible with **Difficult Spontaneous
Magic** or Weak Spontaneous Magic". `incompatible_with` is symmetry-enforced at
load (`ruleset/integrity.rs::validate_incompatibility_symmetry`), so the fix must
add both halves of the Ceremonial ↔ Difficult edge. Conversely the *absence* of a
Difficult ↔ Weak edge is **correct**, stated explicitly from both ends
(ArMDE:5970, :7088). No new finding number — F-385 owns it; this is corroboration
that it survives a second, independent reading.

### F-412 — `flaw.deteriorating_power` — a written-out formula classified as flavour, and no parameter where its twin has one

> "Like the Variable Powers Virtue, this Flaw reduces the effectiveness of one of
> the character's powers (Greater, Lesser, or Ritual) over time. **This penalty is
> linked to either the character's (age / 10), (Might Score – Might Pool / 5), or
> Warping Score.** For example, a magic character with Deteriorating Powers linked
> to age and an age of 25 would have **one of his powers reduced by 3 magnitudes**.
>
> This Flaw may be taken more than once, if the character has more than one
> Power." — ArMDE:5946, :5948
>
> DE 5946: "…**Diese Einbuße ist entweder an (Alter / 10), (Machtwert – Machtpool
> / 5) oder den Verzerrungswert des Charakters geknüpft.** Beispielsweise hätte ein
> Magiecharakter mit Nachlassender Macht, die ans Alter gebunden ist, und einem
> Alter von 25 eine seiner Mächte **um 3 Magnituden reduziert**."

Three division formulas, a player choice between them, and a worked example with
a number. `classification: "narrative"` asserts there is nothing mechanical here.

**No effect variant can express it** — the reduction scales with age, with a
Might-Pool remainder, or with Warping Score, and none of the 42 variants takes a
scaling input. D3 therefore gives `uncomputed_rule`, with the formula written out
in both locales. Today the displayed text is the summary, which is the passage's
first sentence and stops one clause before the formulas.

**The second half of the finding is the parameter, and it is visible only by
opening the cross-reference.** ArMDE:5946's first three words point at
*Variable Powers*, and the target — `virtue.variable_power`, ArMDE:5199-5206 —
carries exactly the parameter this entry lacks:

| | `virtue.variable_power` | `flaw.deteriorating_power` |
|---|---|---|
| ArMDE | 5199-5206 | 5944-5949 |
| passage | "One of the character's magic powers (Greater, Lesser, Personal, or Ritual) becomes more powerful over time" | "reduces the effectiveness of **one of** the character's powers (Greater, Lesser, or Ritual)" |
| `parameters` | `[{"key":"power","type":"ref","domain":"text","require_power":true}]` | **none** |
| `max_per_target` | default 1 | **255** |
| repeat clause | ":5205 — more than once, if the character has more than one power, **but it only applies once to a single power**" | ":5948 — more than once, if the character has more than one Power" |

Both Flaw and Virtue name **one** of the character's powers, and only the Virtue
records which. The consequence is concrete: `require_power: true` makes
`validation/selections.rs::validate_power_targets` check the value names a real
`Entity::powers` entry, so a Variable Power pointing at nothing is caught, while a
Deteriorating Power points at nothing by construction — two copies are
indistinguishable in the save and the Markdown export prints no power at all.

`max_per_target: 255` is itself **correctly authored** and is not the defect:
`RULES.md:1501-1530` documents 255 as the "no stated ceiling" encoding and lists
this entry by name at `:1530`. The defect is that without a `power` parameter the
255 permits **five copies on the same Power**, where the book permits one copy per
Power — precisely the distinction `RULES.md:1482-1499` calls "shape 1" (repeats
with a different target, modelled by a parameter, `max_per_target` staying 1) and
which `flaw.flawed_parma_magica` and `flaw.limited_magic_resistance` were moved
*out* of the 255 table to obtain. How to land it is **Q-101**, because the two
entries must move together.

**Severity: moderate.** The classification is a lost-rule defect (the formula
reaches nobody); the missing parameter is a data-fidelity defect that also
loosens a stated ceiling.

### F-413 — `virtue.variable_power` (ArMDE:5199-5206, **outside this batch**, in B09's span) — the same formula, also `narrative`, and never screened by any guard

Found by following F-412's cross-reference and rated in full per the brief's
standing rule. It belongs to **B09**'s span (ArMDE:5097-5256); recorded here
because B09 can re-derive this far more cheaply than it can rediscover it.

> "One of the character's magic powers (Greater, Lesser, Personal, or Ritual)
> becomes more powerful over time, **based on either (age / 10), (Might Score / 5),
> Warping Score, or his score in an appropriate Ability**. (Age and Warping should
> not be allowed as variables if the character is immune to their effects.) …
>
> Generally speaking, **a single power can have its effect level increased by
> five, or include another effect of the same level that is similar the original,
> for each level of variance.** For a character with Variable Powers based on his
> age, for example, this would mean the effectiveness of one of his powers would be
> **increased by a total of five magnitudes after he has lived for 50 years**.
>
> **This Virtue may be taken more than once, if the character has more than one
> power, but it only applies once to a single power.**" — ArMDE:5201, :5203, :5205

`classification: "narrative"`, no `effects`. Four linkage options, a per-level
increment of five, a worked example, and an explicit per-target repeat ceiling.
This is not flavour by any reading.

**And the structural point, which is why it is worth its own number rather than a
line inside F-412.** `SWEPT_BLOCKS` in
`crates/arm-rules/tests/uncomputed_clauses.rs` covers `(ArMDE, 3360, 3950)` and
`(ArMDE, 5639, 7113)`. **ArMDE:5199-5206 lies between them**, so
`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` has never looked
at this entry at all — and it would not have caught it if it had: the passage has
no signed number (`(Might Score / 5)` has no sign; the en dash in the *Flaw*'s
version is followed by a space, not a digit), no botch term, and none of the
twenty `MECHANICAL_PHRASES`. So this entry is invisible twice over. It is the
clearest available evidence for why the unswept middle of the Virtues block
(ArMDE:3951-5282) needs the same treatment the two swept blocks got, and it
belongs in F-424's list.

**Verdict:** `narrative` → **`uncomputed_rule`**, formula in both locales.
Its `max_per_target` default of 1 combined with the `power` parameter already
encodes ":5205 — only applies once to a single power" correctly ✓, which is the
shape `flaw.deteriorating_power` should be given (F-412, Q-101).

*Not rated here, and flagged for B09:* the passage's parenthetical "(Age and
Warping should not be allowed as variables if the character is immune to their
effects.)", which is a conditional exclusion referencing the `no_aging` /
Unaging family. I read it but did not audit its encoding.

### F-414 — `flaw.exciting_experimentation` — a dice-substitution rule classified as flavour

> "Your character's experiments tend to have a flair for the ... dramatic. **When
> rolling on any of the experimentation tables, roll two dice instead of the normal
> one. The storyguide then chooses and applies the more amusing of your two
> results.**" — ArMDE:6042
>
> DE 6042: "Die Experimente deines Charakters neigen zu einer gewissen ...
> dramatischen Note. **Wenn du auf einer der Experimentiertabellen würfelst, wirfst
> du zwei Würfel statt des normalen einen. Der Spielleiter wählt dann das amüsantere
> deiner beiden Ergebnisse aus und wendet es an.**"

A die-count change with a named resolution procedure, and `narrative` asserts the
passage says nothing mechanical. **Following the cross-reference shows exactly
which roll it replaces**: ArMDE:11028, *Experimentation* — "you must also **roll a
stress die on the 'Extraordinary Results Chart'** for each season that the project
involves" — with ArMDE:11038 adding the botch handling. So the Flaw doubles a
specified stress-die roll and hands the choice of outcome to the storyguide.

**The catalogue's own precedent settles the classification.**
`flaw.creative_block` (ArMDE:5875) states the *same shape* of clause — "If you
experiment, roll twice as many dice on the experimentation table" — and is
`in_play_effect`, with B11's F-396 obliging that clause into its description. Its
mirror `virtue.inventive_genius` (ArMDE:4153) likewise. Exciting Experimentation
states the clause and nothing else, so it cannot be `in_play_effect` (it carries
no effect and the engine simulates no experimentation), which leaves
**`uncomputed_rule`** under D3.

**Verdict:** `narrative` → **`uncomputed_rule`**, the two sentences in
`description` in both locales.

### F-415 — `flaw.exciting_experimentation` — the summary is truncated mid-sentence at the book's ellipsis, in both locales

Separate from F-414 and independent of it: the entry's shipped `summary` is a
fragment that ends in the middle of a word-pair and says nothing at all.

| locale | shipped `summary` | the source sentence |
|---|---|---|
| EN | "Your character's experiments tend to have a flair for the ..." | ArMDE:6042 — "Your character's experiments tend to have a flair for the ... **dramatic.**" |
| DE | "Die Experimente deines Charakters neigen zu einer gewissen ..." | DE 6042 — "Die Experimente deines Charakters neigen zu einer gewissen ... **dramatischen Note.**" |

The book's rhetorical ellipsis has been read as a sentence terminator by whatever
produced the summaries, so the payload word — *dramatic* / *dramatischen Note* —
is cut in **both** locales, identically. The entry's name is "Exciting
Experimentation", so the picker currently offers a Flaw whose whole description is
"…experiments tend to have a flair for the …".

I checked whether this recurs: `grep` for a summary ending in ` ..."` across both
`rules/i18n/*/virtues_flaws.json` finds it on this entry. *Scope stated as
checked:* I searched the two virtues-and-flaws i18n files, not the other i18n
domains.

**Severity: moderate.** It is not a wrong rule but it is a user-facing string
that fails to say anything, on every platform, in every session, in both
languages — the accessibility/localization class `CLAUDE.md` rates up rather than
down. The fix is trivial (extend the summary through "dramatic." /
"dramatischen Note.") and must land with F-414's description rather than instead
of it.

### F-416 — `flaw.envied_beauty` — a Characteristic precondition, classified as flavour; and the passage names penalties it never states

> "The character's beauty draws revulsion and jealousy. This envy does not strike
> everyone, but vain persons of the character's gender are particularly susceptible
> to it. Characters with this Flaw may avoid **its penalties** by refusing to reveal
> their beauty to the world, which creates its own complications. **A character
> lacking a positive Presence score may not have this Flaw.**" — ArMDE:6014
>
> DE 6014: "…Charaktere mit diesem Fehler können seinen **Nachteilen** ausweichen,
> indem sie ihre Schönheit vor der Welt verbergen, was wiederum eigene
> Komplikationen mit sich bringt. **Ein Charakter ohne positiven Präsenz-Wert darf
> diesen Fehler nicht nehmen.**"

The last sentence is a hard eligibility rule keyed on a Characteristic score, and
`classification: "narrative"` asserts the passage states nothing mechanical. The
entry carries `prerequisites: null`.

**It is exactly expressible in shape and not in substance, which is the D3 case.**
`types.rs::Prereq` has eight variants — `All`, `Any`, `Nor`, `Has`, `House`,
`AbilityMin`, `ArtMin`, `IsMagus` — and there is **no** characteristic-minimum
variant. A Presence floor has nothing to bind to. Compare
`Effect::CharacteristicScoreDeltaParam`, whose "must already be at ±3" gate is
enforced not by `Prereq` but by a hard-coded validator
(`validation/scores.rs::validate_characteristic_delta_preconditions`) derived from
the sign of the effect's own `amount` — a bespoke check for one variant, not a
reusable prerequisite. So the engine cannot express "Presence > 0" today, and D3
gives **`uncomputed_rule`** with the rule written out, never `narrative`.

**A second, smaller thing the passage does, which the description must handle
honestly.** "Characters with this Flaw may avoid **its penalties**" refers to
penalties the entry never states — there is no number anywhere in ArMDE:6012-6015.
So the avoidance clause is a rule about a mechanic the book leaves to the
storyguide. Whoever writes the description should transcribe it as the book has it
rather than inventing the penalty it points at; that is the same discipline the
invented aging floor (`7f5605a`) failed.

**Severity: moderate.** Nothing is miscalculated, but a Major Flaw's one hard
eligibility rule reaches the player in neither locale, and the catalogue records
it nowhere a validator could ever read.

### F-417 — `flaw.difficult_underlings` — a taking-restriction, classified as flavour

> "**You may only take this Story Flaw if your character has, and will keep,
> underlings of some sort or another.** No matter how many people you fire, or how
> carefully you vet new candidates, your underlings always cause problems for
> you…" — ArMDE:5978
>
> DE 5978: "**Du darfst diesen Geschichte-Fehler nur nehmen, wenn dein Charakter
> Untergebene hat und auch weiterhin haben wird.** …"

Sentence one is a restriction on *who may take the Flaw*, which is a
character-creation legality rule, and `narrative` asserts the passage contains
none.

**The precedent is B11's and it is direct.** B11's **F-390** reclassified
`flaw.consumed_casting_tools` on the identical grammatical shape — "This Flaw may
only be taken by Verditius magi" (ArMDE:5841) — and rated it high; B11's **F-389**
did the same for `flaw.companion_animal`'s "*animals only*" (ArMDE:5806), whose
condition is as unmodellable as this one. **D3 is explicit that inexpressibility
is never grounds for `narrative`**, so the fact that the engine has no concept of
"has underlings" settles the encoding question and not the classification one.

I record honestly where this differs from F-390 and where it does not. It differs
in that Verditius is a `Prereq::House` the engine ships and this is not: there is
no entity state for underlings, no `Prereq` variant, and no candidate — so unlike
F-390 there is no live validator bug here and nothing is falsely accepted that the
data *could* have caught. It does **not** differ in the thing the check is about:
the sentence is a rule, `narrative` says the passage has none.

**One thing is already right and is the reason this is the mildest of the
reclassifications.** The entry's summary **is** sentence one, verbatim, in both
locales — so unlike every other `narrative → uncomputed_rule` row in this batch,
the rule does reach the player today. The D5 obligation is already met by
accident; only the class is wrong.

**Verdict:** `narrative` → **`uncomputed_rule`**. **Severity: low** — a
provenance/lost-rule defect whose lost-rule half is already covered.

### F-418 — `flaw.disorientating_magic` — the entry's own summary states the rule the classification denies

> "After casting a spell, **you must spend a round doing nothing but recovering
> your mental faculties**." — ArMDE:5986
>
> DE 5986: "Nach dem Wirken eines Zaubers **musst du eine Runde lang nichts tun
> außer deine geistigen Kräfte zu sammeln**."

The passage is one sentence long and that sentence is a rule: a compulsory loss of
one combat round's action after every cast. The entry is `narrative`.

This is the batch's cleanest single-check failure, and it is worth a number
precisely because it is so unambiguous. The shipped `summary` in both locales is
that sentence, verbatim and complete — so the entry's own displayed text states
the mechanic that its own `classification` field says does not exist. There is no
ambiguity about what the passage says, no second reading, and no cross-reference
to weigh: a round of lost action is a rule in any tabletop game and this one has a
combat round as a defined unit.

**No effect can express it** — nothing in the 42 variants models an
action-economy cost, and the engine simulates no combat rounds — so D3 gives
**`uncomputed_rule`**.

**Verdict:** `narrative` → **`uncomputed_rule`**, **class only**. Checks 3-11
pass and check 12 passes: the D5 obligation is already satisfied by the summary,
in both locales. **Severity: low**, for the same reason as F-417 — the rule
reaches the player; the data's claim about the rulebook is false.

### F-419 — `flaw.dutybound` — the shipped German name is not the German rulebook's heading, and the German source disagrees with itself

`rules/i18n/de/virtues_flaws.json` gives `flaw.dutybound` the name
**`Pflichtgebunden`**. The German rulebook's heading at DE 5992 is
**`#### Pflichtbewusst`**. There is no row for *Dutybound* anywhere in
`rules/source/de/translation-tables/`, so the rulebook is the only authority and
the two disagree.

**But the German source is not self-consistent, and both terms are its own**, in a
pattern worth spelling out because it decides the fix:

| DE line | context | term |
|---|---|---|
| 5992 | the entry's own `####` heading | **Pflichtbewusst** |
| 5476 | the "List of Flaws" index link, `[Pflichtbewusst](#pflichtbewusst)` | **Pflichtbewusst** |
| 24583 | the back-of-book index, `\| Pflichtbewusst (Fehler) \| [126](#pflichtbewusst) \|` | **Pflichtbewusst** |
| 2692 | the Faerie Doctor character-concept list | Pflichtgebunden |
| 3879 | *Fida'i*'s body text, "den Treueeid- oder Pflichtgebunden-Fehler" | Pflichtgebunden |
| 18948 | a spell's body text, "den Fehler Pflichtgebunden besitzen" | Pflichtgebunden |

Every **structural** reference — heading, index link, back-of-book index, and the
anchor all three share — says *Pflichtbewusst*. Every **body-prose** reference says
*Pflichtgebunden*. The shipped data followed the body prose.

**By the precedent B11 set this is backwards.** B11's F-406 met the mirror image —
DE:5907's body calling a Flaw *"Gepeinigt von einem übernatürlichen Wesen"* where
its own heading, index link and shipped name all read *"Von übernatürlichem Wesen
geplagt"* — and ruled: "The data follows the heading, which is **right**; the body
text is the outlier." Applying the same rule here makes the shipped name wrong.

**What I am not doing is picking the better German word.** *Pflichtgebunden* is
arguably the closer rendering of *Dutybound* and the data's choice may well be the
better one — in which case the correction is to the **heading**, which is a
rulebook-source edit and a larger call than a data edit, with the upstream re-sync
bearing on it exactly as B11 said of F-405. What is not defensible is the current
state, where the picker offers *Pflichtgebunden* while the rulebook a player has
open beside it is headed *Pflichtbewusst* and its index lists it under P-f-l-i-c-h-t-b-e-w.

**This is the third German-source naming inconsistency found in two adjacent
batches** (B11's F-405 *Ordenskunde*/*Organisationskunde* at DE:5791, B11's F-406
*Gepeinigt*/*geplagt* at DE:5907, and this one). Three in ~270 lines of source is
a rate, not a coincidence, and it argues for a mechanical check that every shipped
DE name equals the DE heading on the parallel line — which is a guard the
repository does not have and could cheaply gain.

**Severity: moderate** — a user-facing name in one of the two shipped locales that
does not match the book it is a translation of, on an entry two other passages of
the same book also name.

### F-420 — `flaw.difficult_longevity_ritual` — the halving is computed; its scope and its carve-out reach neither locale

> "Something in your magical nature makes it difficult to create an effective
> Longevity Ritual for you. **Anyone (including yourself) creating a Longevity
> Ritual for you must halve their Lab Total. You may create Longevity Rituals for
> others without penalty.**" — ArMDE:5964
>
> DE 5964: "…**Jeder (einschließlich dir selbst), der ein Langlebigkeitsritual für
> dich erschafft, muss seine Laborsumme halbieren. Du kannst ohne Einschränkung
> Langlebigkeitsrituale für andere erschaffen.**"

**The data is correct and is not marked wrong.**
`magic_total_halving { total: "lab_longevity" }` is the right variant and the
right member: `derived/lab.rs::creo_corpus_lab_total` reads it, halves with the
floor `div_euclid(2)`, and `RULES.md:4443-4463` records the whole derivation
including that this Flaw was the first reader of `HalvableTotal::LabLongevity`.
`in_play_effect` is right — the halving moves a real number in the derived
read-out. Nothing here is a miscalculation.

**What is a finding is the text, under D5.** The entry carries no `description` in
either locale and its summary is the flavour sentence, so two clauses reach the
player nowhere:

1. **the scope** — "Anyone (**including yourself**) creating a Longevity Ritual
   **for you**". The engine halves the character's *own* Creo Corpus hint, which is
   the "including yourself" half; the "anyone" half — that a *different* magus
   building the ritual must halve *his* Lab Total — is a rule about another
   character's sheet and is computed nowhere. Without the sentence, a troupe
   reading the sheet cannot know the Flaw travels.
2. **the carve-out** — "**You may create Longevity Rituals for others without
   penalty.**" This is the sentence that stops the halving being read as a
   property of the character's lab, and it is computed nowhere either (the engine
   models one longevity figure, the character's own).

**Severity: low.** The class is right, the number is right, and the halved figure
is flagged as halved in the UI. What is missing is the two sentences that say
whose ritual is affected.

### F-421 — `flaw.disjointed_magic` — both of its clauses reach neither locale

> "You cannot use previous knowledge to help you with magic. **You gain no benefit
> from knowing a spell that is similar to one you are learning or inventing, and
> you gain no enchantment bonuses from Techniques and Forms already invested in an
> item.**" — ArMDE:5974
>
> DE 5974: "…**Du erhältst keinen Nutzen daraus, einen Zauber zu kennen, der einem
> ähnelt, den du gerade erlernst oder erfindest, und du erhältst keine
> Verzauberungsboni von Techniken und Formen, die bereits in einen Gegenstand
> eingebettet wurden.**"

**The effect is not marked wrong, and RULES.md is why.** `RULES.md:4891-4894`
records the decision explicitly: "`flaw.disjointed_magic` carries
`special_casting_mod { circumstantial }`, never a `lab_total_mod`, and
`ArMDE:5974` states no Lab-Total number at all — it is filed under
`SpecialCastingMod` below, which is its only correct home." That is a written
rationale and I do not contradict it.

**One observation I record without overturning it**, because it bears on F-423
and on whoever revisits the family. Both of the passage's clauses are *laboratory*
mechanics — the similar-spell bonus applies when learning or inventing a spell,
and the invested-Art bonus applies when enchanting — while `SpecialCastingMod` is
by name and by consumer a **casting**-side family
(`derived.rs::InPlayMods::residual_voice_penalty` and friends; the surfaced row is
rendered under the Fluent key `derived-surfaced-special_casting`). The choice is
still defensible on RULES.md's argument (no number is stated, so no `lab_total_mod`
can be authored), but the read-out consequence is that a lab-only Flaw is listed
to the player under a casting heading. Not counted as a data defect.

**What is a finding is the text, under D5.** No `description` in either locale;
the summary is sentence one ("You cannot use previous knowledge to help you with
magic."), which states the theme and neither mechanic. Both clauses — the
similar-spell bonus and the invested-Technique/Form enchantment bonus — are real,
named bonuses the rules grant elsewhere and this Flaw removes, and neither reaches
the player in either language. **Severity: moderate** — two removed bonuses is the
Flaw's entire cost, and a player cannot price a Minor Flaw whose cost is invisible.

### F-422 — `flaw.dwarf` — three effects correct, two clauses in neither locale

> "You are the size of a child. **Your comfortable walking speed is twothirds that
> of a normal person.** Your Size is -2, so the severity of wounds you take
> increases in three point increments rather than five point increments (see page
> 404). You take a -1 penalty to each of Strength and Stamina, **which may reduce
> each Characteristic as low as -6**. You cannot take this Flaw and Giant Blood
> (page 83), Large (page 89), or Small Frame (page 145)." — ArMDE:5998
>
> DE 5998: "Du hast die Größe eines Kindes. **Deine angenehme Gehgeschwindigkeit
> beträgt zwei Drittel der einer normalen Person.** Deine Größe ist –2 … Du erhältst
> –1 auf je Stärke und Ausdauer, **was jede Eigenschaft bis auf –6 senken kann**. …"

**The data is correct in every mechanical respect and is not marked wrong**, which
is worth stating in full because this entry is the batch's best-encoded:

| clause | data | engine |
|---|---|---|
| "Your Size is -2" | `size_delta { amount: -2 }` | `effective/characteristic.rs::size` ✓ |
| "wounds … in three point increments rather than five" | *(no separate effect)* | **a consequence, computed**: `derived/combat.rs::wound_ranges` sets `u = (size + 5).max(1)` = 3 ✓ — the data encodes the cause, which is right |
| "a -1 penalty to each of Strength and Stamina" | `characteristic_score_delta { characteristic.str, -1 }` + `{ characteristic.sta, -1 }` | `effective/characteristic.rs::characteristic_score_bonus` ✓, two effects for two targets as A24 requires |
| "may reduce each Characteristic as low as -6" | *(no clamp)* | **satisfied by construction**: the fold is explicitly unclamped (A2/A24), mirroring Giant Blood's +6 at ArMDE:3977 ✓ |
| "cannot take this Flaw and Giant Blood, Large, or Small Frame" | `incompatible_with: ["flaw.small_frame","virtue.giant_blood","virtue.large"]` | all three encoded, **and all three targets name Dwarf back** (ArMDE:3977, :4231, :6769); symmetry load-enforced ✓ |

The sign of every effect is right against what the track counts (Size and the two
Characteristics all count *higher is bigger/stronger*, so a Dwarf's are negative),
and `creation_effect` is the right class. This is also the audit's **first fully
encoded multi-way stated incompatibility** — set against the six stated / one
encoded that B10 and B11 found, it shows the omissions elsewhere are omissions.

**What is a finding is the text, under D5.** No `description` in either locale;
the summary is "You are the size of a child." / "Du hast die Größe eines Kindes.",
so two clauses reach the player nowhere:

1. **"Your comfortable walking speed is two-thirds that of a normal person"** — a
   ratio on a movement rate the engine models nowhere, and the only clause of the
   five that is neither computed nor a consequence of something computed;
2. **"which may reduce each Characteristic as low as -6"** — computed by absence,
   which is precisely why it needs saying. A player looking at Strength −4 has no
   way to know the engine is permitting it deliberately rather than failing to
   clamp, and the mirror sentence at ArMDE:3977 (Giant Blood's +6) has the same
   problem from the other end.

**Severity: low.** Everything that changes a number is right; what is missing is
two sentences, one of which explains why a number the engine prints is legal.

### F-423 — a surfaced modifier carries no source item, so six different Flaws render as the same unattributed word — a gap Part C does not cover

Two of this batch's three `in_play_effect` entries carry
`special_casting_mod { circumstantial }`, and following the read-out to the screen
turns up something `engine-semantics.md` Part C records nowhere.

`derived.rs::SurfacedModifier` has exactly three fields:

```rust
pub struct SurfacedModifier {
    pub family: ModifierFamily,
    pub detail: String,
    pub amount: i32,
}
```

There is **no source-item field**. For a surfaced-only kind, `in_play_mods` pushes
`detail: kind.to_string()` and `amount: 0`. And
`ui/src/lib/components/DerivedSurfacedModifiersSection.svelte` renders
`{store.t('derived-surfaced-' + m.family)}: {store.t('derived-detail-' + m.detail)}`
and suppresses the value entirely when `m.amount === 0`. So the whole payload that
reaches the player for a `circumstantial` carrier is the two words
**"Special casting: Circumstantial"**, with nothing naming which Flaw produced it.

`jq` over `rules/core/virtues_flaws.json` returns **six** carriers of
`special_casting_mod { kind: "circumstantial" }`: `flaw.corrupted_spells`,
`flaw.deleterious_circumstances`, **`flaw.disjointed_magic`**,
**`flaw.environmental_magic_condition`**, `flaw.short_ranged_magic`,
`flaw.the_constant_expression`. They produce byte-identical rows. A magus with two
of them sees the same line twice — `surfaced_modifiers` is a `Vec`, so it is not
even deduplicated — and cannot tell which is which.

**Why this is a finding rather than a Part C restatement.** Part C's C1 records
that eight of eleven `SpecialCasting` kinds are surfaced-only (the *magnitude* is
lost), and C2-d records that `param` is unread for every kind but `deft_form`.
Neither records that the **identity of the carrying item** is lost too — and that
is the one that matters for check 10, because it is what decides whether "the rule
reaches the user somehow". For these six it does not: the row does not say what is
halved, under which circumstances, or on whose account.

**This is what makes the two cross-references in my span bite.** ArMDE:6022 tells
the troupe that Environmental Magic Condition "**should be significantly more
restrictive than** the Hermetic Flaw Deleterious Circumstances" — the book's own
distinction between a Major and a Minor Flaw — and the engine renders both as the
same two words. And ArMDE:5974's Disjointed Magic, whose two clauses are
laboratory bonuses (F-421), is listed under the same casting heading as the other
five.

**Not counted against `flaw.disjointed_magic` or
`flaw.environmental_magic_condition`** — their data is what the engine offers, and
`RULES.md:4869`/`:4891-4894` record the choice. The defect is
`SurfacedModifier`'s shape. **Severity: moderate** — it is the read-out of six
shipped Flaws, and it compounds with D5: for the four of the six that carry no
`description`, the anonymous row is the *only* thing the player gets.

### F-424 — the swept-block screen missed eleven entries here, and the exact word-forms it lacks

This is B10's F-383 and B11's F-403 recurring with a third fresh set of
word-forms, and it is again the deliverable most useful beyond the batch. My whole
span (ArMDE:5932-6067) lies inside `SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` block,
swept 2026-09-15 and re-swept 2026-09-19.
`cargo test -p arm-rules --test uncomputed_clauses` is **green on all four
assertions** (run 2026-09-20), and eleven `narrative` entries in my span state a
rule.

The screen is `states_a_mechanical_rule = has_signed_number || has_botch_term ||
has_mechanical_phrase`. None of the eleven passages contains a botch term, a
signed number (`has_signed_number` requires a sign char *immediately* followed by
a digit), or any of the twenty `MECHANICAL_PHRASES`.

| entry | ArMDE | the wording that states the rule | nearest phrase already on the list |
|---|---|---|---|
| `flaw.deteriorating_power` | :5946 | "linked to either the character's **(age / 10)**, **(Might Score – Might Pool / 5)**, or Warping Score" / DE "an **(Alter / 10)**, **(Machtwert – Machtpool / 5)**" | none. The en dash here is followed by a **space**, so `has_signed_number` reads no sign; the "Formulas" section holds only `equal to` / `multiply` and a division written with `/` matches nothing |
| `flaw.deteriorating_power` | :5946 | "reduced by **3 magnitudes**" / DE "um **3 Magnituden** reduziert" | `one magnitude` / `eine magnitude` — **one word away**, and the German is one *inflection* away (`Magnituden` vs `magnitude`, which the left-boundary substring match would catch if the phrase were stemmed to `magnitude`) |
| `flaw.diabolic_past` | :5960 | "**You may purchase the Ability** Infernal Lore, **even if you are normally not permitted to buy** Arcane Abilities" / DE "**Du darfst die Fertigkeit** … **erwerben, selbst wenn du normalerweise** … **nicht** …" | none — a fourth new family, the **Ability-permission idiom**, and by F-409's census the most numerous unencoded rule in the book |
| `flaw.faerie_friend` | :6054 | "**can purchase** the Arcane Knowledge Faerie Lore, **even if they are normally restricted from purchasing it**" | same family |
| `flaw.faerie_upbringing` | :6058 | "**You may learn** Faerie Lore **at character generation**" | same family, and the *shortest* form of it — nine words with no "even if" at all, which is why a phrase for the family must catch `may learn … at character generation` and not only the `even if` tail |
| `flaw.excommunicate` | :6046 | "a bad reputation **at level 3** within the Church" / DE "eine schlechte Reputation **der Stufe 3**" | none. The detector deliberately requires a *sign*, and a Reputation level carries none — the same blind spot B11 recorded for Church Upbringing's unsigned "25 experience points" |
| `flaw.enfeebled` | :6010 | "**unable to learn** Martial Abilities" / DE "**nicht in der Lage**, Kampffertigkeiten **zu erlernen**" | a fifth new family: the **prohibition** idiom. `cannot die` is the only Absolute on the list |
| `flaw.enfeebled` | :6010 | "you lose **double the normal number of** Fatigue levels" / DE "**doppelt so viele** Erschöpfungsstufen **wie normal**" | `multiply`/`multiplied`/`multiplizier` is the Formulas entry for exactly this idea and does not reach `double` / `doppelt` — **one synonym away** |
| `flaw.difficult_spontaneous_magic` | :5968 | "**You cannot cast** Spontaneous spells **without exerting yourself**" / DE "**Du kannst keine** spontanen Zauber **wirken, ohne** dich dabei anzustrengen" | `cannot die` / `nicht sterben` — the same "cannot \<verb\>" Absolute B11 flagged for `cannot walk`; the German here is again continuous (`kannst keine … wirken` is discontinuous, but `Du kannst keine` is not) |
| `flaw.difficult_underlings` | :5978 | "You **may only take this** Story Flaw **if** your character has, and will keep, underlings" / DE "**Du darfst** diesen Geschichte-Fehler **nur nehmen, wenn**" | the `may only be taken by \<House\>` family B11's F-403 already proposed — **this is its second member and a different grammatical shape**, so the phrase must be `may only take this` / `darf … nur … nehmen` as well as `may only be taken by` |
| `flaw.disorientating_magic` | :5986 | "you **must spend a round** doing nothing" / DE "musst du **eine Runde lang** nichts tun" | none — and `round up` / `round down` are on the list, so a naive `round` would collide with them; the form needed is `spend a round` / `eine runde lang` |
| `flaw.envied_beauty` | :6014 | "A character **lacking a positive Presence score may not have this Flaw**" / DE "Ein Charakter **ohne positiven Präsenz-Wert darf diesen Fehler nicht nehmen**" | `may not be greater than` — **the "may not" shape is already on the list**, one continuation away from `may not have this Flaw`; the German `darf … nicht nehmen` is discontinuous and would need its own form |
| `flaw.exciting_experimentation` | :6042 | "roll **two dice instead of the normal one**" / DE "wirfst du **zwei Würfel statt des normalen einen**" | `simple die` / `stress die` / `einfachen würfel` / `stresswürfel` — **the Dice section exists and names dice by kind, not by count**; this passage names a count and no kind |

**Four of the thirteen rows are a single word or inflection from something already
on the list** (`3 magnitudes` vs `one magnitude`; `double`/`doppelt` vs
`multiply`/`multiplizier`; `may not have this Flaw` vs `may not be greater than`;
`cannot cast` vs `cannot die`). **Two new families** the screen has no member of
at all:

1. **The Ability-permission idiom** — "you may purchase/learn/take \<Ability or
   category\>, even if normally …". This is the highest-value phrase the list
   could gain, because F-409 shows it marks **at least twenty-nine** passages of
   which exactly **two** are encoded, so it is simultaneously a classification
   screen and a live-bug detector.
2. **The prohibition idiom** — "unable to learn \<category\>", "may not
   have/take \<this Flaw\>", joining B11's proposed Absolutes extension.

**The screen remains a blocker on the fix, exactly as B11 found**, and the
arithmetic is worse here. `every_uncomputed_rule_entry_states_its_rule_in_every_locale`
is unscoped and applies the same detector to an `uncomputed_rule` entry's
displayed text in every locale, so reclassifying these eleven and writing their
passages into `description` verbatim turns the first guard green and the second
one red. Running the detector by hand over each proposed verbatim description:

| entry | would the description trip the detector? |
|---|---|
| `flaw.deteriorating_power` | **red** — "(age / 10)" has no sign, "3 magnitudes" is not `one magnitude` |
| `flaw.diabolic_past` | **red** |
| `flaw.difficult_spontaneous_magic` | **red** |
| `flaw.difficult_underlings` | **red** |
| `flaw.disorientating_magic` | **red** |
| `flaw.enfeebled` | **red** |
| `flaw.envied_beauty` | **red** |
| `flaw.exciting_experimentation` | **red** |
| `flaw.excommunicate` | *(moot — it becomes `creation_effect`, not `uncomputed_rule`)* |
| `flaw.faerie_friend` / `flaw.faerie_upbringing` | *(moot — both become `creation_effect`)* |

So **eight of the eleven cannot be landed** until `MECHANICAL_PHRASES` grows, and
the three that escape do so only because F-408 and F-409 move them into
`creation_effect` instead. The order of work is unchanged from B11's conclusion:
extend the phrase list first, with the word-forms tabulated above, then
reclassify.

**And one thing B11 could not see from inside a swept block.** `virtue.variable_power`
(F-413) lies at ArMDE:5199-5206, **between** the two `SWEPT_BLOCKS` ranges
(3360-3950 and 5639-7113), so the screen has never examined it — and it would not
have flagged it if it had. The unswept middle of the Virtues block,
**ArMDE:3951-5282**, is 1,331 lines and the screen has never run over any of it.
That is the single largest unscreened region in the book and, on this batch's hit
rate, it is where the next dozen of these live.

### F-425 — five translation-table rows disagree with the German rulebook, and the shipped data is right on all five

Stated as **D6** requires: `arm-de-translation` is outside this repository, so
each of these is reported as *"this repository's copy of the table disagrees with
the rulebook"* and the error-versus-stale-copy determination is left to the
orchestrator. In every case the **shipped `rules/i18n/de/` name follows the
rulebook heading and is correct**; the table's copy here is the outlier. Precedence
is rulebook > thematic table (`decisions.md` D6, and
`rules/source/de/translation-tables/README.md`).

| table row | table's German | DE rulebook heading | shipped data |
|---|---|---|---|
| `tugenden-fehler.md:340` — `\| Deteriorating Power \| Schwindende Kraft \| SdM:M; eine Kraft wird schwächer über Zeit \|` | *Schwindende **Kraft*** | DE 5944 `#### Schwindende **Macht**` | Schwindende Macht ✓ |
| `tugenden-fehler.md:294` — `\| Disorientating Magic \| Desorientierungsmagie \| \|` | *Desorientierungsmagie* | DE 5984 `#### Desorientier**ende Magie**` | Desorientierende Magie ✓ |
| `tugenden-fehler.md:368` — `\| Enfeebled \| Entkräftet \| \|` | *Entkräftet* | DE 6008 `#### **Geschwächt**` | Geschwächt ✓ |
| `tugenden-fehler.md:267` **and** `:341` — `\| Environmental Magic Condition \| Magische Umgebungsbedingung \| … \|` | *Magische **Umgebungs**bedingung* | DE 6020 `#### Magische **Umwelt**bedingung` | Magische Umweltbedingung ✓ |
| `tugenden-fehler.md:342` — `\| Environmental Sensitivity \| Umgebungsempfindlichkeit \| \|` | *Umgebungs­empfindlichkeit* | DE 6024 `#### **Umwelt**empfindlichkeit` | Umweltempfindlichkeit ✓ |

Note the internal pattern in the last two: the table consistently renders
*Environmental* as **Umgebungs-** and the rulebook consistently as **Umwelt-**, on
two neighbouring entries. That is a systematic divergence in one prefix rather
than two independent slips, which is exactly the shape a **stale copy** takes —
and D6 names the stale copy as one of the two possible causes.

**Two further rows are not errors and are recorded so nobody "fixes" them.**

- `tugenden-fehler.md:472` — `| Depressed | Deprimiert | **GotF** – Persönlichkeitsfehler; …` is tagged *Guardians of the Forest* (the Rhine Tribunal book), which has no English source in `rules/source/en/` and therefore cannot supply an ID. The core-rules entry is `Depressiv` (DE 5940) and the shipped name is `Depressiv` ✓. The row is about a different book's Flaw of the same English name. Worth a line only because a future agent looking up *Depressed* will hit it first.
- `tugenden-fehler.md:341` additionally claims a "**Klein**" (Minor) Environmental Magic Condition and cross-references a "Groß/Hermetisch-Version". The core book has **one** Environmental Magic Condition and it is Major (ArMDE:6021, index ArMDE:5292 under `### Hermetic, Major`). This is D6's "a terminology table making a factual claim about the rules" shape — the fifth-of-seven instance the audit has now found — and it is **also** a claim this repository cannot verify, since the row is tagged `SdM:M` (*Sphären der Macht: Magie* = RoP:M) and RoP:M ships no such Flaw at that magnitude in `rules/source/en/`. Folded into the Environmental Magic Condition row above rather than numbered separately.

**Severity: low** on all five. None of these files is loaded at runtime and the
shipped data is correct in every case; the harm is that the tables are the
canonical input CLAUDE.md points the *next* German-writing agent at, so an
uncorrected row regenerates the error. That is precisely D6's "the faucet is still
running" argument for fixing the table rather than only the data.

### F-426 — `flaw.ability_block` (ArMDE:5651-5654, **outside this batch**, in B10's span) — the archetype of F-410's missing mechanism, also `narrative`

Raised by the verification sub-agent while establishing that no effect can forbid
an Ability category (F-410), and re-verified here from the source. It belongs to
**B10**'s span (ArMDE:5257-5780); recorded per the standing rule to rate a
cross-reference wherever it lands.

> "**You are completely unable to learn a certain class of Abilities**, for some
> reason. This may be Martial Abilities, or a more limited set of the others. A
> profound inability to master logic would rule out Artes Liberales, Philosophiae,
> any Law, Medicine, and Theology. Alternatively, you might be unable to learn any
> languages other than your native tongue. **It must be possible for your character
> to learn the abilities in question in the absence of this Flaw**, but she need
> have no intention of doing so. **You may only take this Flaw once.**" —
> ArMDE:5653
>
> DE 5653: "**Du bist aus irgendeinem Grund völlig unfähig, eine bestimmte Klasse
> von Fertigkeiten zu erlernen.** … **Du darfst diesen Fehler nur einmal nehmen.**"

`classification: "narrative"`, no `effects`, no `parameters`, no `description` in
either locale.

**Why it earns a number rather than a mention.** F-410 rests on the claim that the
engine has a *grant* for an Ability category and no *forbid*. This entry is the
proof: it is the one Flaw in the catalogue whose **entire content** is that
prohibition, and it is unencoded for exactly the same reason — so the gap is a
missing mechanism, not an oversight on one Flaw. Two Flaws in two adjacent batches
state the same inexpressible rule, which is what distinguishes a gap from an
accident. D3 gives **`uncomputed_rule`** on both.

**What is already right.** The summary is sentence one verbatim in both locales,
so the prohibition itself does reach the player (as with F-417 and F-418, the D5
half is met by luck of sentence order). And "You may only take this Flaw once" is
enforced by construction: the entry carries no `parameters`, so
`max_per_target`'s default of 1 keys on `(item_ref, {})` and a second copy is a
`duplicate_selection` error ✓.

**What is not.** The class, and — a **Q-88** instance — the *chosen class* of
Abilities is recorded by no parameter, so two characters with Ability Block are
indistinguishable in the save and the export prints no class at all. Given the
passage's own examples span a whole category (Martial), an ad-hoc list (Artes
Liberales, Philosophiae, Law, Medicine, Theology) and a linguistic rule, a
single-valued `text` parameter is the only shape that could hold it — which is
Q-95's territory, not a thing this batch settles.

**Severity: moderate** as a lost-rule defect, and **it is the entry that should
drive the decision**: whether to add a forbidding effect variant is a question
about two entries now, not one.

### F-427 — ArMDE:2816's "one Social Status" rule is enforced nowhere, while the *guideline* one paragraph below it is enforced strictly

**Raised by the verification sub-agent against a claim this file's first pass made
and got wrong.** My method section asserted that the one-Social-Status rule is a
profile-level cap B10 confirmed is enforced. The sub-agent challenged it; I
re-derived it myself and **the sub-agent is right**.

> "**All characters must take one Social Status, and may only take more than one
> if the descriptions of the Virtues or Flaws explicitly note that they are
> compatible.**" — ArMDE:2816
>
> "A character **should** not have more than one Story Flaw. This is a guideline,
> and may be violated with the whole troupe's agreement." — ArMDE:2818

Two rules, four lines apart, and the engine enforces the weaker one.

`rules/core/character_types.json` gives every profile a `budget.flaw_category_caps`
list, and `jq` over all four returns:

| profile | `flaw_category_caps` | `virtue_category_caps` |
|---|---|---|
| `grog` | `personality max 0 (major_only, hard)`, `personality max 1`, **`story max 0`** | — |
| `companion` | `personality max 1 (major_only, hard)`, `personality max 2`, **`story max 1`** | — |
| `magus` | same three | `hermetic max 1 (major_only, hard)` |
| `mythic_companion` | same three | — |

**`social_status` appears in no cap list on any profile, in either direction.** In
the whole file the slug occurs only inside `permitted_categories` (lines 20, 59,
94, 138) — which permits the category, and permits any number of it. And the two
halves of ArMDE:2816 fail for two different reasons:

- *"may only take more than one"* — `validation/caps.rs::validate_caps` reads only
  the two `*_category_caps` lists, and neither names `social_status`, so no cap
  exists to compare against. A character may hold five Social Statuses today.
- *"All characters **must** take one"* — the only "must hold" machinery is
  `EntityTypeProfile::required_traits`, which is a list of **ids**
  (`["virtue.hermetic_magus"]` on magus; empty on the other three), not a list of
  categories. There is no shape that says "at least one item carrying category X",
  so a character with **no** Social Status at all validates clean.

*Scope stated as checked:* I queried `rules/core/character_types.json` for
`flaw_category_caps` / `virtue_category_caps` / `required_traits` on all four
profiles, and grepped `crates/arm-rules/src` and `crates/arm-app/src` for
`social_status` — every hit in `src` is a `#[cfg(test)]` JSON fixture or a doc
comment. I did not audit ArMDE:2816's enforcement in the UI layer.

**Two instances sit in this batch and neither is flagged today.**
`flaw.failed_journeyman` and `flaw.failed_master` are both `*Minor, Social
Status*`, neither passage notes a compatibility, and they are mutually exclusive
in fiction — a man expelled from his guild and a man who ran his own workshop into
the ground and must now work as a journeyman. Nothing stops a character taking
both, and per B10's finding that pairwise `incompatible_with` edges are the wrong
shape for a *categorical* rule, adding an edge between these two would not fix it
either: the rule spans the whole category and every future Social Status entry.

**Severity: moderate**, and it is the ordinary product-integrity class rather than
the provenance one — `validate_caps` is a gate that silently passes a character
the rulebook forbids, and the mandatory half means the app will happily produce a
character with no Social Status at all, which is not a legal Ars Magica character.
It is **not** a B12 entry defect: no entry's data is wrong. It is a missing
profile-level rule, and it is `RULES.md`-silent — `grep "2816"` over
`crates/arm-rules/src`, `crates/arm-rules/RULES.md` and
`rules/core/character_types.json` returns **nothing**, where ArMDE:2814, :2818 and
:2820 are all cited and all enforced. **Q-102** asks whether the omission is a
deliberate YAGNI deferral.

## Open questions

**Q-97 — `crates/arm-rules/RULES.md` declares a `source.lines` range wrong that
this audit's check 1 declares right, and the disagreement is about the convention,
not the entry.** `RULES.md:4471-4474`:

> "**Note on the Flaw's cited range.** `rules/core/virtues_flaws.json` records
> `flaw.difficult_longevity_ritual` as `ArMDE:5962-5965`; line 5965 is blank, so
> the accurate inclusive range is **`ArMDE:5962-5964`**, which is what this section
> and the `derived.rs` comments cite. The JSON value is left for a data-only
> correction pass."

Check 1's stated convention is "the entry's own `####` heading to the line before
the next heading", which for this entry is 5962-5965 exactly. **And the blank last
line is not peculiar to this entry: it is the shape of 34 of this batch's 35
ranges**, because every entry in the book is separated from the next heading by one
blank line. So RULES.md is not reporting a defect in one entry's data; it is
proposing a *different convention* (stop at the last non-blank body line) and has
recorded the one entry it happened to look at as a pending correction. B09, B10 and
B11 each met the tighter variant on one or two entries and called both forms
correct. Either convention is defensible and mixing them is not, and a pending
"data-only correction pass" that would rewrite 34 ranges in this batch alone
deserves a decision before it runs. **What would settle it:** a ruling on which
form is canonical, plus deleting or rewriting `RULES.md:4471-4474` so the
traceability map does not carry a standing instruction to change data the audit
has just certified.

**Q-98 — how should `flaw.enfeebled`'s "double the normal number of Fatigue
levels" be encoded, and which way does `HealthTrack::CastingFatigue`'s sign run?**
Two halves, and the second is the sharper.

*(a) The multiplier.* `Effect::HealthMod` carries an additive `amount: i8`. The
book says *double* (ArMDE:6010). For a magus who normally loses one Fatigue level
per fatiguing cast, `+1` is right; for a Fatiguing Spontaneous cast that already
costs several (ArMDE:4303 — "one additional Fatigue level per five points … by
which you missed the target level"), it is not. Options: (i) `uncomputed_rule` +
description, accepting that the track computes nothing anyway (C1 lists
`CastingFatigue` as surfaced-only, so even a correct number would move nothing);
(ii) author `+1` and record the approximation in RULES.md, the shape
`RULES.md:6360-6365` already used for Covenant Upbringing's authorization width;
(iii) a multiplier field on `HealthMod`. F-410 recommends (i) as the only one that
states no number the book does not.

*(b) The sign, which must be settled before (ii) or (iii) is possible.*
`types.rs::HealthTrack::CastingFatigue`'s doc reads "Fatigue levels lost per spell
cast (**Vulnerable Casting +1, Withstand Casting −1**, Painful Magic)". The three
shipped carriers are `flaw.painful_magic` **−1**, `flaw.vulnerable_casting` **−1**,
`virtue.withstand_casting` **+1** — all three the opposite of the doc. Either the
doc is backwards or the data is, and nothing in the repository resolves it: the
track is surfaced-only, so no test and no computed number can catch the
disagreement, and the UI prints `formatSigned(amount)` beside an untranslated
family label. None of the three carriers is in this batch's span so I raise no
finding against them; what I cannot do is wire a fourth carrier without knowing
which side is right. **What would settle it:** reading `virtue.withstand_casting`'s
and `flaw.vulnerable_casting`'s passages (ArMDE:5261-5266 and :6993-6998) and
ruling on whether the track counts *levels lost* (doc) or *benefit to the
character* (data).

**The rules read that would have settled it has been done, and it does not
settle it.** ArMDE:5263 says Withstand Casting means "he loses **1 less** Fatigue
level than normal, with a minimum of 1"; ArMDE:6995 says Vulnerable Casting means
"she loses **1 more** Fatigue level than normal"; ArMDE:6576 says Painful Magic
costs "the equivalent of **one** Fatigue level in pain for each spell you cast".
Those are **directions, not signs** — the book never writes `+1` or `−1` for any
of the three, so no reading of the rulebook can decide which sign the engine
should store. The question is therefore purely a **convention** question and it
belongs to the codebase, not to the source.

**The one piece of internal evidence points at the doc, not the data.** The
sibling track's doc, `HealthTrack::FatigueRoll`, reads "(Long-Winded +3,
Obese/Short of Breath −3)" — Virtue positive, the *benefit* convention, which is
what all three `casting_fatigue` amounts follow and what the `casting_fatigue` doc
alone contradicts. So the cheapest correct fix may be to rewrite one doc comment
rather than flip three shipped numbers. That is an inference from one sibling, not
a source fact, so it is recorded here and not acted on. **What would settle it:**
a ruling on what the `derived-detail-casting_fatigue` read-out is a signed
quantity *of*, followed by aligning the doc, the three amounts and the EN/DE
Fluent label to whichever is chosen — and only then can F-410's Enfeebled be
wired. *(A small side note from the same read: `types.rs`'s doc comment for this
track itself writes `−` U+2212, the character the project bans in output. A Rust
doc comment is not displayed text so it is outside the hyphen rule, but it is the
same glyph.)*

**Q-99 — does RoP:D:5671 oblige anything of `flaw.dhimmi`, and does a supplement's
rule about *when a core Flaw is required* belong in the core entry?** ArMDE:5956
states no game mechanic and I passed the entry as `narrative`. But following its
own pointer reaches `rules/source/en/Ars Magica 5e - Realms of Power - The Divine
(Revised).md`:5671 — "**Almost all non-Muslim characters living under Muslim rule
must also take** the Minor General Flaw *Dhimmi* … Hermetic magi are the most
obvious examples of such exceptions." That is a rule, it is in a book the
repository has in English, and it is about this exact entry. It is not a mechanic
*of* the Flaw, and no `Prereq` or effect expresses "is a non-Muslim in Muslim
lands" — but the audit has no settled position on whether a supplement's
*take-this-Flaw* requirement counts toward the core entry's classification, and
getting it wrong in either direction sets a precedent across every entry with a
supplement pointer. **What would settle it:** a ruling on whether a rule stated in
a supplement, about a core entry, and attributable to no clause of the core
passage, can make that core entry non-`narrative`. (Related but distinct from D3,
which is about engine capability, not about which book the rule is in.)

**Q-100 — should `SurfacedModifier` carry the id of the item that produced it?**
F-423 establishes that six shipped Flaws render as the identical unattributed
string "Special casting: Circumstantial", with `amount: 0` suppressed, and that a
character holding two of them sees the line twice. Adding a `source: Id` field is
a small, purely additive change — `derived.rs::in_play_mods` already iterates
`(selection, effect)` pairs and has the item in hand at every push site — and the
UI could then render the Flaw's localized name beside the family. Against it:
`surfaced_modifiers` is a display list with no consumer that needs identity today,
the id would have to be mapped through `rules/i18n/` to be renderable, and it
widens a DTO that crosses the IPC boundary. This is a product decision about the
read-out, not a rules reading, and it affects all five `ModifierFamily` values and
not only `SpecialCasting`. **What would settle it:** a decision on whether the
surfaced list is a *diagnostic* (identity matters) or a *summary* (it does not) —
noting that for the four `circumstantial` carriers with no `description`, the
anonymous row is currently the only thing the player is given.

**Q-101 — how should "once per Power" be encoded, on the twin entries that
disagree about it today?** `virtue.variable_power` (ArMDE:5205 — "more than once,
if the character has more than one power, **but it only applies once to a single
power**") carries a `power` text parameter with `require_power: true` and the
default `max_per_target` of 1, which encodes the rule exactly.
`flaw.deteriorating_power` (ArMDE:5948 — "more than once, if the character has
more than one Power") carries **no parameter** and `max_per_target: 255`, which
permits unlimited copies on one Power. Three readings and I will not pick one:
**(a)** the two passages state the same rule with the Flaw's qualifier elided, so
the Flaw should be given the Virtue's shape (a `power` parameter,
`max_per_target` back to 1) and removed from `RULES.md:1509`'s 255 table — the
same move `RULES.md:1534-1539` already made for `flaw.flawed_parma_magica` and
`flaw.limited_magic_resistance`; **(b)** the Flaw genuinely omits the qualifier, so
255 is a faithful reading of *its* sentence and only the missing parameter is a
defect, which would leave a repeat ceiling the book does not state; **(c)** both
stay as they are and the divergence is accepted. (a) changes the enforced ceiling
on a shipped entry and (b) leaves a known-looser one, so this needs a ruling
rather than a per-entry judgement. **What would settle it:** a decision on whether
a cross-referencing entry ("Like the Variable Powers Virtue") inherits the
qualifiers of the entry it points at.

**Q-102 — is ArMDE:2816's total absence from the engine a deliberate deferral or a
gap?** F-427 establishes that neither half of "All characters must take one Social
Status, and may only take more than one if the descriptions … explicitly note that
they are compatible" is enforced anywhere, while the *guideline* two paragraphs
below it (ArMDE:2818, one Story Flaw) is enforced strictly on all four profiles.
The mandatory half additionally needs machinery the engine does not have:
`required_traits` is id-based, and "at least one item carrying category X" has no
representation. YAGNI is a real possibility — a direct-entry app whose user has the
book open may not need to be told to pick a Social Status — but nothing records
the choice: `grep "2816"` over `crates/arm-rules/src`,
`crates/arm-rules/RULES.md` and `rules/core/character_types.json` returns nothing,
where :2814, :2818 and :2820 are each cited *and* enforced. **What would settle
it:** an owner's ruling, plus either a `social_status` cap on the four profiles
and a category-level required-trait shape, or a `RULES.md` decision record saying
why not. Raised by the verification sub-agent as its open question O2 and adopted.

## Sub-agent reconciliation

One verification sub-agent was run, after this file's header, method and
35-row verdict table were on disk, with a two-part brief: **independently
re-derive the verdict for every entry marked fully clean** (20 at the time),
reading the passages itself rather than this file's notes, and **independently
re-derive the four highest-stakes findings** without being given my reasoning. It
confirmed it never opened `batch-12.md`. It spawned no agents, wrote no file, and
ran no git-mutating command. It reported receiving the auto-mode injection and
ignoring it.

**Result: 20 of 20 corroborated, 0 overturned. Of the four high-stakes claims, 2
confirmed outright, 1 confirmed and widened, 1 confirmed with the reasoning
narrowed. It also overturned one claim in this file's *method* section, raised two
out-of-span findings, and established six negatives I had assumed.**

### The one thing it overturned, and it is mine

**ArMDE:2816 → F-427.** My method section's ArMDE-guideline roll-up said, of the
one-Social-Status rule, "the one-Social-Status rule is the profile-level cap B10
confirmed is enforced." That was an **inherited verdict I did not check** — B10
established the enforcement state of ArMDE:2818 (Story Flaws) and I carried the
sentence across to :2816 without querying the profiles. The sub-agent queried them
and found no `social_status` cap anywhere. I re-derived it myself before accepting:
`jq` over all four profiles in `rules/core/character_types.json` returns
`flaw_category_caps` containing only `personality` and `story`, no
`virtue_category_caps` outside magus's `hermetic`, `required_traits` id-based and
empty on three of four profiles, and every `social_status` occurrence in
`crates/*/src` a test fixture or doc comment. **The overturn is accepted in full**
and the method section now carries the correction rather than the claim. This is
precisely the failure mode `README.md`'s "**No inherited verdicts**" rule names,
committed inside a batch whose own brief quotes it — recorded plainly so the next
agent treats a neighbouring batch's roll-up as a lead and not a warrant.

### What it widened

**F-409, the Ability-permission gap.** My census was a grep of thirteen idiom
spellings across the whole book, joined to entries by `source.lines[0]` — 29
entries, grep-derived. Its census was a grep of the four gated-category proper
nouns, narrowed to the Flaws block and then **read hit by hit** — 7 Flaws,
read-verified, all 7 inside my 29. Neither supersedes the other and the finding now
carries both with their methods named. It also contributed the figure that makes
the denominator honest: **10 of the 28 `restricted_ability_xp` carriers name a
gated category** and therefore carry the permission implicitly (A7), so the
catalogue's covered population is 12 rather than the 2 my `ability_authorization`
query alone suggested. That does not weaken the finding — the uncovered population
is still ≥29 — but stating 2 without stating 12 would have been the exact
"true narrow claim wrapped in a false wide one" the brief warns about.

### What it narrowed, on two findings

1. **F-411's "structurally cannot".** I wrote that the engine structurally cannot
   express "the non-fatiguing spontaneous option is unavailable". The sub-agent
   read `derived/casting.rs` and found the mirror Flaw is handled **not by
   halving** but by having the dead slot echo the live one, with a source comment
   saying `CastingScores` "has no 'this option does not exist' representation". So
   Difficult Spontaneous Magic is one new `HalvableTotal` member away from
   computable, not structurally out of reach. **The verdict is unchanged** — D3
   turns on what the engine can express *today* — but the wording was overstated
   and is corrected. It also supplied the primary citations (ArMDE:9141, :9143,
   :9145) for a definition I had taken second-hand off the mirror Flaw's text at
   ArMDE:7086; I verified all three and replaced the citation.
2. **F-410 / Q-98's `casting_fatigue` sign.** The brief hands this down as "the
   data is inverted, caught by reading the doc". The sub-agent read the three
   passages (ArMDE:5263 "1 less", :6995 "1 more", :6576 "one Fatigue level in
   pain" — I re-read all three) and the sibling track's doc
   (`FatigueRoll`: "Long-Winded +3, Obese/Short of Breath −3", the *benefit*
   convention the data follows) and made the case that **the doc comment, not the
   data, may be the outlier**. The rulebook states directions, never signs, so no
   rules read can settle it — which converts Q-98 from a rules question into a
   convention question, and materially changes what a correction pass should do
   (rewrite one doc comment rather than flip three shipped numbers).

### Two out-of-span findings it contributed

- **F-426 — `flaw.ability_block` (ArMDE:5651-5654, B10's span)**, found while
  establishing that no effect can forbid an Ability category. `narrative`, no
  effects, on a passage whose whole content is that prohibition. Re-verified here
  from the source and written up in full; it is what turns F-410's missing
  mechanism from one entry's problem into a gap.
- **F-427 — ArMDE:2816**, above.

It also flagged `flaw.savantism` as a `creation_effect` with no effects; that is
**C7**, already recorded in `engine-semantics.md` and in `README.md`'s list of
five, so it is not a new finding. And it noted that
`validation/scores.rs::validate_reputations` ignores the granted `score`; that is
**C2** and is likewise already recorded.

### Six negatives it established that this file had assumed

Listed because an assumed negative that happens to be true is still a hole in the
record.

1. **`tainted` on all 20.** It located the rule (`RULES.md:156-159` — the flag
   comes from the descriptor's optional third token, never the name or the prose),
   confirmed all 20 descriptors are bare two-token lines, and checked the trap case
   from a second source: `flaw.exiled_atlantean`'s prose says "she is **tainted**"
   but **RoP:M:2651**'s independent copy of the descriptor is also plain
   `*Minor, Supernatural*`. I had reasoned to the same conclusion from the
   descriptor alone.
2. **No missing `prerequisites` among the 20.** It read all 20 for a gating
   sentence and found none, and independently surfaced the contrast case inside my
   span (`flaw.envied_beauty`, ArMDE:6014) — reaching F-416 by its own route.
3. **`flaw.dhimmi`'s "a dinar per year" is not a dropped money rule.** It grepped
   the whole core rulebook and found ArMDE:5956 is the **only** occurrence of
   "dinar" (every other hit is `-ordinar-` inside "extraordinary"). With no
   dinar-denominated economy anywhere in the book, the sentence cannot be a sheet
   number. I had judged it colour; this proves it.
4. **No missing `max_total` / `max_per_target` / `parameters` among the 20.**
5. **The absent `incompatible_with` between `failed_journeyman` and
   `failed_master` is the *correct shape*** — ArMDE:2816 is categorical, so a
   pairwise edge would be the wrong instrument — **which is what led it to
   discover that the categorical rule is enforced nowhere** (F-427). A negative
   that turned into the batch's second systemic finding.
6. **The ASCII-hyphen property is mechanically guarded**, not merely true today:
   `crates/arm-rules/tests/rules_i18n_ascii_hyphen.rs`. My method section had
   verified the property with two greps and not named the guard.

### One cross-reference it caught that I did not

`flaw.evil_eye`: ArMDE:3917 also contains an "*Evil Eye*" — but it is the Folk
Magic **effect type** (`folk_magic.evil_eye`, "Uses the Perdo Corpus, Perdo
Animal, or Perdo Herbam guidelines. Divine Lore cannot be used to produce this
type of effect"), a different mechanic from the Supernatural Flaw at ArMDE:6036.
It confirmed no contamination between the two in the data. Worth the line because
the shared name is exactly the sort of thing a grep-driven correction pass would
conflate.

### Disagreements remaining

**None.** The single divergence — my ArMDE:2816 claim — is resolved on evidence I
re-derived myself rather than by preferring either reading, and the resolution is
recorded in F-427 and in the corrected method roll-up. Both narrowings are
accepted; the widening is folded into F-409 with both methods and both scopes
stated. No finding in this file rests on the sub-agent's word alone: every claim
it raised was re-verified here against the named file and line before being
written up.
