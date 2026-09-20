# Batch B11 — indices 350-384, ArMDE:5781-5931

Entries: 35. Audited: 35. Failures: **18**. Clean: **17**.
Findings **F-384 … F-407** (F-407 lies outside the batch, in B03's span).
Open questions **Q-88 … Q-96**.
The verification sub-agent re-derived all eighteen entries this file first called
clean and **overturned one** (`flaw.a_deal_with_the_devil`, F-406); it also
established four negatives, contributed one out-of-span finding and three open
questions, and made one withdrawal that this file does not accept — see the
reconciliation section.

**The whole span sits inside `SWEPT_BLOCKS`' ArMDE:5639-7113 Flaws block** — swept
2026-09-15, re-swept 2026-09-19, guarded green by
`uncomputed_clauses.rs::no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause`.
As B10 predicted, the green proves nothing: **seven `narrative` entries here state
a rule**, and every one states it in a word-form the 20-phrase screen does not
carry. The missed forms are tabulated in **F-403** and are the deliverable that
outlives this batch — and F-403 also shows the screen is a **blocker on the fix**,
not only a diagnosis: five of the seven reclassifications cannot be landed until
`MECHANICAL_PHRASES` grows, because the second, unscoped guard applies the same
blind detector to the new descriptions.

**The span's second shape is the one B10 handed over: `incompatible_with`.**
ArMDE:5783 states a two-way incompatibility over three catalogue entries; **none
of the three sides is encoded** (F-385). That makes six stated pair rules across
B10+B11 with one encoded.

**The span's third shape is new and is the largest single defect in the batch:
seven of the seven `in_play_effect` entries carry no `description` in either
locale**, and between them they drop roughly two dozen mechanical clauses. One of
them, `flaw.the_constant_expression`, is recorded in `crates/arm-rules/RULES.md`
as carrying those clauses in its rules text. It does not (F-392).

*(Written incrementally: header, method and the 35-row verdict table first,
findings appended one at a time, sub-agent reconciliation last.)*

## Method

**Both languages were read as continuous prose before any entry was judged** —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` 5773-5947 and
the line-parallel `rules/source/de/Ars Magica Definitive Edition Basisregeln.md`
5773-5937, the read deliberately overrunning the span at both ends so the first
and last entries' boundaries could be seen.

**Line parity holds throughout.** Every `####` heading in 5781-5931 sits on the
same line number in both files (33 headings for 35 catalogue entries — the three
Major/Minor pairs `Compassionate` 5809, `Compulsion` 5813 and `Compulsive Lying`
5817 each ship as two ids citing one heading).

**Check 1 — `source.lines`: 35 of 35 correct.** Every range runs from its own
`####` heading to the line before the next heading. **One** stops a line earlier,
on its last body line: `flaw.deficient_technique` 5913-5915 where the next
heading is 5917. That is the tighter variant B09 and B10 also saw and is equally
correct; it is recorded here only so the next reader does not count it as an
anomaly. No range in this batch swallows a neighbour and none is short.

**Check 1, second half — `source.anchor`: 4 of 35 entries carry one, all four
correct**, verified twice: derived from the `####` heading, and against the
book's own "List of Flaws" index links, which spell them out literally.

| id | anchor | heading | index link |
|---|---|---|---|
| `flaw.clumsy_magic` | `clumsy-magic` | ArMDE:5801 | ArMDE:5424 |
| `flaw.corrupted_abilities` | `corrupted-abilities` | ArMDE:5847 | ArMDE:5537 |
| `flaw.craving_for_travel` | `craving-for-travel` | ArMDE:5869 | ArMDE:5574 |
| `flaw.curse_of_slander` | `curse-of-slander` | ArMDE:5881 | ArMDE:5538, :5575 |

The other 31 carry the key set `["file","lines"]` only.

**Checks 3, 4, 5, 6 (`kind` / `magnitude` / `entity_kinds` / `categories`):
35 of 35 correct**, each read against its own descriptor line one by one:

- `*Minor, Hermetic*` — ceremonial_spontaneous_magic (5782), clumsy_magic (5802), consumed_casting_tools (5840), creative_block (5874), cyclic_magic_negative (5894), deficient_form (5910), deleterious_circumstances (5918)
- `*Major, Hermetic*` — chaotic_magic (5786), the_constant_expression (5822), deficient_technique (5914)
- `*Minor, Hermetic, Tainted*` — corrupted_arts (5854), corrupted_spells (5860)
- `*Minor, Supernatural, Tainted*` — corrupted_abilities (5848)
- `*Minor, Supernatural*` — cursed_guile (5890)
- `*Minor, General or Supernatural*` — curse_of_slander (5882)
- `*Minor, Personality*` — church_upbringing (5790), continence (5844), covenant_upbringing (5866), delusion (5922)
- `*Major or Minor, Personality*` — compassionate_\* (5810), compulsion_\* (5814), compulsive_lying_\* (5818)
- `*Minor, Story*` — close_family_ties (5794), demonic_familiar (5927)
- `*Major, Story*` — curse_of_venus (5886), dark_secret (5898), a_deal_with_the_devil (5906)
- `*Minor, Social Status, animals only*` — companion_animal (5806)
- `*Minor, General*` — clumsy (5798), craving_for_travel (5870)
- `*Major, General*` — crippled (5878), deaf (5902)

`kind` is `flaw` on all 35 ✓ (the whole span is inside the book's Flaws block).
`entity_kinds` is `["character"]` on all 35 ✓ — `types.rs::EntityKind` has exactly
two members, `Character` and `Covenant`, and none of these is a covenant Hook.
`tainted: true` is carried by exactly the three entries whose descriptor says
`Tainted` (corrupted_abilities, corrupted_arts, corrupted_spells) ✓ and by no
other — notably **not** by `flaw.cursed_guile` ("probably results from Infernal
influence", ArMDE:5891) or `flaw.a_deal_with_the_devil` or
`flaw.demonic_familiar`, none of whose descriptors carries the tag. Following the
descriptor rather than the prose is correct.

**The index confirms every filing, with no contradiction.** Unlike B10's Q-79,
every one of the 35 entries is filed in the book's own "List of Flaws"
(ArMDE:5283-5638) under a `### <category>, <magnitude>` heading that matches its
descriptor and the data:

| index lines | heading | entries |
|---|---|---|
| 5288-5290 | `### Hermetic, Major` (5285) | Chaotic Magic, The Constant Expression, Deficient Technique |
| 5314-5316 | `### Personality, Major or Minor` (5310) | Compassionate, Compulsion, Compulsive Lying |
| 5346-5348 | `### Story, Major` (5340) | Curse of Venus, Dark Secret, A Deal with the Devil |
| 5404-5405 | `### General, Major` (5401) | Crippled, Deaf |
| 5423-5431 | `### Hermetic, Minor` (5417) | Ceremonial Spontaneous Magic, Clumsy Magic, Consumed Casting Tools, Corrupted Arts, Corrupted Spells, Creative Block, Cyclic Magic (negative), Deficient Form, Deleterious Circumstances |
| 5471-5474 | `### Personality, Minor` (5467) | Church Upbringing, Continence, Covenant Upbringing, Delusion |
| 5506-5507 | `### Story, Minor` (5501) | Close Family Ties, Demonic Familiar |
| 5523 | `### Social Status, Minor` (5520) | Companion Animal |
| 5537-5539 | `### Supernatural, Minor` (5534) | Corrupted Abilities, Curse of Slander, Cursed Guile |
| 5573-5575 | `### General, Minor` (5564) | Clumsy, Craving for Travel, Curse of Slander |

`Curse of Slander` is the one dual listing (5538 Supernatural Minor **and** 5575
General Minor); the data carries **both** as real `categories` plus the
`taken_as` category-domain parameter, which `RULES.md:389` and `:512` already
record. Corroboration, not a finding.

**Check 12, mechanically.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the 35 ids in each locale returns the
key set `["name","summary"]` for **28** entries and
`["description","name","summary"]` for **7** — the same 7 in both locales, so the
locales never disagree about *whether* a description exists. The 7 are exactly
the batch's `uncomputed_rule` entries (clumsy, clumsy_magic, corrupted_abilities,
craving_for_travel, curse_of_slander, cursed_guile, deaf), each carrying the full
cited passage in both languages ✓. **No `narrative`, `creation_effect` or
`in_play_effect` entry in this batch carries a description in either locale** —
which is where every D5 finding below comes from.

**ASCII hyphen (check 12, second half): clean, and verified two ways.**
`grep -c "−"` (U+2212) over `rules/i18n/en/virtues_flaws.json` and
`rules/i18n/de/virtues_flaws.json` returns **0** for both — catalogue-wide, not
just this batch. And `grep -o "–[0-9]"` (U+2013 followed by a digit) returns
**nothing** in either file, so none of the 34 DE / 1 EN en dashes that do ship is
a *sign*; every one is punctuation. The conversion is happening correctly: the DE
source writes every negative as `–` (ArMDE:5799 `–3`, :5803 `–3`, :5849 `–3`,
:5871 `–1`/`–3`, :5875 `–3`, :5895 `–3`) and every one ships as ASCII `-`. A
correction pass adding the missing descriptions must keep doing this.

**German names against the canonical tables.** Twenty of the thirty-two distinct
English names in this batch have a row in
`rules/source/de/translation-tables/`, located by English key (the file is
modified in the working tree by another workstream, so no line number is relied
on). **All twenty agree with the shipped `rules/i18n/de/` name and with the DE
rulebook heading** — `tugenden-fehler.md` rows for Ceremonial Spontaneous
Magic → *Zeremonielle Spontane Magie*, Chaotic Magic → *Chaotische Magie*, Clumsy
→ *Ungeschickt*, Clumsy Magic → *Plumpe Magie*, Close Family Ties → *Enge
Familienbande*, Compassionate → *Mitfühlend*, Compulsion → *Zwang*, Compulsive
Lying → *Zwanghaftes Lügen*, Consumed Casting Tools → *Verbrauchende
Zauberwerkzeuge*, Continence → *Enthaltsamkeit*, Corrupted Arts → *Verderbte
Künste*, Corrupted Spells → *Verderbte Zauber*, Covenant Upbringing →
*Konventsprägung*, Craving for Travel → *Reiselust*, Cyclic Magic (negative) →
*Zyklische Magie (negativ)*, Deaf → *Taub*, Deficient Form → *Defizitäre Form*,
Deficient Technique → *Defizitäre Technik*; `sphären-mächte.md` for Corrupted
Abilities → *Verderbte Fertigkeiten*; `reputationen.md` for Curse of Slander →
*Verleumdungsfluch*. The twelve English names with **no** table row (Church
Upbringing, Companion Animal, The Constant Expression, Creative Block, Crippled,
Curse of Venus, Cursed Guile, Dark Secret, A Deal with the Devil, Deleterious
Circumstances, Delusion, Demonic Familiar) all match the DE rulebook heading on
the parallel line ✓. One **row** disagrees with the rulebook on a *fact* rather
than on terminology — F-403's D6 item.

**`rules/i18n/de/abilities.json` was consulted** for the Ability names this
batch's passages name (Artes Liberales, Latein/Dead Language, Musik,
Ordenskunde/Organization Lore, Theologie, Täuschung/Guile, Finesse,
Konzentration) — `ability.dead_language` is the catalogue's Latin, parameterized
by `language`; `ability.theology_christian` is the catalogue's Theology for a
Church upbringing. No name mismatch.

### Part C systemic gaps are not re-reported per entry

Four of Part C's rows touch this batch and none is counted against an entry:

- **`SpecialCasting::Circumstantial` is surfaced-only** (C1) — that is
  `flaw.the_constant_expression`, `flaw.corrupted_spells` and
  `flaw.deleterious_circumstances`, and `RULES.md:4869` records the choice
  deliberately. What *is* counted against all three is D5: the rule reaches the
  user through no text either (F-392, F-394, F-401).
- **`magical_focus`'s unread `major`** (C2-b) — no entry here carries one.
- **`creation_effect` is not required to carry effects** (C7) — this is
  `flaw.corrupted_arts`, one of the five entries `README.md` names. C7 explains
  why it is green; **F-393** is about whether `creation_effect` is the right
  class at all, which C7 does not answer.
- **`HalvableTotal` has four members and none is "all magic totals"** (A33) — so
  `flaw.deleterious_circumstances` genuinely cannot be a `magic_total_halving`.
  Not a data defect.

### Decisions applied

`docs/vf-audit/decisions.md` was read in full and is binding. It carried D1-D6
when this batch read it (the file is modified in the working tree by another
workstream; nothing in this batch touched it).

- **D3** is the load-bearing one here and governs five entries:
  `flaw.companion_animal` (no `Effect` variant grants a Personality Trait —
  confirmed against all 42 variants in `types.rs::Effect`; and there is no
  `animal` entity kind or type profile), `flaw.church_upbringing` (no effect
  shape *earmarks* existing budget), `flaw.corrupted_spells` (`Prereq` has no
  "holds ≥ 30 levels of formulaic spells" variant), `flaw.the_constant_expression`
  (no `HealthTrack` represents a permanently-lost long-term Fatigue level) and
  `flaw.crippled`. In every case D3 forbids `narrative` and requires the rule in
  the description.
- **D5** produces eleven of the twenty findings: 28 of 35 entries carry no
  description in either locale, and thirteen of those state a clause the effects
  do not implement.
- **D4** governs the two directed reads (`flaw.creative_block`,
  `flaw.cyclic_magic_negative`). Both conditions are transcribed verbatim in both
  languages below; neither entry's **data** is marked wrong on their account, per
  the brief. One sentence of ArMDE:5895 that D4's table does not mention is
  escalated as **Q-91**.
- **D1** touches `flaw.creative_block` and `flaw.cyclic_magic_negative` (both
  carry `lab_total_mod`); per D1 their conditions are deliberately ignored in
  `effective/spell.rs::spell_level_cap`, so neither is marked wrong there either.
- **D6** governs **F-403**'s last row. Per the brief the source project
  `arm-de-translation` is outside this repository, so it is stated as *"this
  repository's copy disagrees with the rulebook"* and the error-vs-stale-copy
  determination is left open.
- **D2** touches nothing here: no entry is a granted Virtue and none carries
  `characteristic_score_delta_param`.

### Cross-references followed and rated

Every pointer in the span was followed and read, including on entries that
passed.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the entry? |
|---|---|---|---|
| `flaw.ceremonial_spontaneous_magic` | "the rules for Ceremonial Casting (page 217)" / DE `#zeremonielles-zaubern` | `#### Ceremonial Casting` **ArMDE:9285-9295** | **YES — three clauses the Flaw makes compulsory**: fifteen minutes per magnitude; add Artes Liberales **and** Philosophiae to the Casting Score; and the spell's level is capped at one magnitude per fifteen minutes spent (ArMDE:9287, :9289, :9291). None reaches the user → folded into **F-384**. |
| `flaw.ceremonial_spontaneous_magic` | "not compatible with Difficult Spontaneous Magic or Weak Spontaneous Magic" | `flaw.difficult_spontaneous_magic` ArMDE:5966-5971 ✓, `flaw.weak_spontaneous_magic` ArMDE:7084-7089 ✓ | **YES — two stated incompatibilities, encoded on none of the three sides** → **F-385**. Reading the two targets also settles that they are compatible *with each other* — ArMDE:5970 and :7088 each explicitly permit the combination — so the data's empty `incompatible_with` is right for that pair and wrong only for the two edges to Ceremonial. |
| `flaw.the_constant_expression` | "a free Flaw providing a Safety penalty of –3 (**see page 288**)" / DE `#sicherheit` | the laboratory Safety rules; `RULES.md:4902` already quotes **ArMDE:11576** — "The Safety score subtracts its value from the number of botch dice on all lab activities" | **YES, and it is already adjudicated**: `RULES.md:4895-4915` records that Safety is a laboratory Characteristic, not a Lab-Total term, so the Flaw imposes botch dice and **no** Lab-Total penalty, and that `lab_total_mod −3` was removed from this entry in the round-4 audit. Corroboration ✓. What is **not** true is the same paragraph's claim that "its own rules text carries the −3 Safety plus the Warping-Score botch dice" → **F-392**. |
| `flaw.a_deal_with_the_devil` | "includes the effects of Plagued By Supernatural Entity" | `flaw.plagued_by_supernatural_entity` **ArMDE:6590-6593** | **YES — and this row's first version said "No", which the verification sub-agent overturned.** The target is indeed `narrative` with no effects, so nothing *numeric* is imported — that half stands. But the **idiom itself** is mechanical, and the catalogue encodes it four times out of four elsewhere (`grants_selection` ×3, `aging_mod` ×1). → **F-406**. The error was asking what the target *computes* instead of what the clause *is*. |
| `flaw.the_constant_expression` | "More details of Conciatta can be found in *Legends of Hermes*" | **outside `rules/source/en/`** | Nothing claimable. Flavour attribution only. |
| `flaw.corrupted_abilities` | "can be sensed by Divine Powers" | the Divine-detection rules | No number attributable to the Flaw; the description already carries the sentence in both locales ✓. |
| `flaw.covenant_upbringing` | "Order of Hermes Lore" | `ability.organization_lore` (category `general`, parameter `organization`) ✓ | **No** — a General Ability needs no authorization, and the passage says only "are likely to have". The *Latin* half is a real rule and **is** encoded; see the corroboration note under F-395. |
| `flaw.deficient_form` / `flaw.deficient_technique` | "Advancement Totals are not halved. Experience points required are based on the actual value of the Technique, before halving." | `effective/art.rs::deficient_arts` is consumed only by `derived/casting.rs`, `derived/lab.rs` and `effective/spell.rs::spell_level_cap` | **Already satisfied by construction** — `effective/xp.rs` never consults `deficient_arts`, so XP is costed off the unhalved score exactly as the passage requires, and no advancement total is computed at all. Corroboration ✓. The word "Technique" appearing in *Deficient Form*'s own sentence is **Q-89**. |

### ArMDE:2814, :2816, :2818, :2820 — instances in this batch

Noted, not re-derived (B10 established the enforcement state of all four).

- **ArMDE:2814** ("more than once only if the description explicitly allows it") —
  three entries in this batch are parameterized, and `max_per_target`'s default of
  1 groups by the *whole* params map, so a second copy with a different parameter
  value is a different tuple and is allowed. `flaw.curse_of_slander` carries
  `max_total: 1` ✓ (required anyway by its `category`-domain parameter);
  `flaw.deficient_form` and `flaw.deficient_technique` carry none → **F-400**.
  The three entries that say "only take this Flaw once" in words —
  `flaw.corrupted_abilities` (:5851), `flaw.corrupted_arts` (:5857),
  `flaw.corrupted_spells` (:5863) — carry no parameters at all, so
  `max_per_target`'s default already enforces it ✓.
- **ArMDE:2816** ("at most one Social Status") — one instance:
  **`flaw.companion_animal`** (:5805-5808). Its passage states no compatibility,
  so it may not be combined with another Social Status, and nothing says so.
- **ArMDE:2818** ("not more than one Story Flaw") — instances:
  `close_family_ties`, `curse_of_venus`, `dark_secret`, `a_deal_with_the_devil`,
  `demonic_familiar`. Enforced on all four profiles per B10 ✓. Worth one line:
  `flaw.a_deal_with_the_devil` is a Major Story Flaw that *declares itself to
  include* another Major Story Flaw, which is the one place the guideline and the
  book's own text pull against each other.
- **ArMDE:2960-2962** (Supernatural realm association) — instances:
  `flaw.corrupted_abilities`, `flaw.curse_of_slander`, `flaw.cursed_guile`. None
  carries a `realm`-domain parameter. Noted only.

## Verdicts

35 rows, one per entry. `class` / `data` / `text` report checks 2, 3-11 and 12.
"OK" means every check in that column passed; `?` means escalated, not resolved.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.ceremonial_spontaneous_magic` | 5781-5784 | narrative → uncomputed_rule | **two stated incompatibilities encoded on none of three sides** | the Ceremonial-only restriction and the three clauses behind "page 217" in neither locale | F-384 F-385 |
| `flaw.chaotic_magic` | 5785-5788 | narrative → uncomputed_rule | OK | the declare-a-level procedure and its ±1-level tolerance in neither locale | F-386 |
| `flaw.church_upbringing` | 5789-5792 | narrative → uncomputed_rule | **no `ability_authorization`: the Flaw's *mandatory* spend is a hard validator error today** | the 25-xp earmark and the Academic bar in neither locale; **the DE source names the wrong Ability at :5791** | F-387 F-388 F-405 **Q-92** |
| `flaw.close_family_ties` | 5793-5796 | narrative ✓ | OK | OK | **clean** |
| `flaw.clumsy` | 5797-5800 | uncomputed_rule ✓ | OK | OK ✓ | **clean** |
| `flaw.clumsy_magic` | 5801-5804 | uncomputed_rule ✓ | OK | OK ✓ | **clean** |
| `flaw.companion_animal` | 5805-5808 | narrative → uncomputed_rule | the granted Personality Trait and "animals only" are both inexpressible (D3) | both clauses in neither locale | F-389; ArMDE:2816 instance |
| `flaw.compassionate_major` | 5809-5812 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.compassionate_minor` | 5809-5812 | narrative ✓ | OK — pair incompatibility present ✓ | OK | **clean** |
| `flaw.compulsion_major` | 5813-5816 | narrative ✓ | OK | OK | **clean** |
| `flaw.compulsion_minor` | 5813-5816 | narrative ✓ | OK | OK | **clean** |
| `flaw.compulsive_lying_major` | 5817-5820 | narrative ✓ | OK | OK | **clean** |
| `flaw.compulsive_lying_minor` | 5817-5820 | narrative ✓ | OK | OK | **clean** |
| `flaw.the_constant_expression` | 5821-5838 | in_play_effect ✓ | `circumstantial` is a **recorded** choice (`RULES.md:4895-4915`) ✓; the lost Fatigue level is inexpressible (D3) | **six clauses in neither locale — and RULES.md asserts they are there** | **F-392** |
| `flaw.consumed_casting_tools` | 5839-5842 | narrative → uncomputed_rule | **`prerequisites: all(is_magus, house.verditius)` absent though exactly expressible** | the hour, the no-stockpiling rule, the erasure and the no-lab clause in neither locale | F-390 F-391 |
| `flaw.continence` | 5843-5846 | narrative ✓ | OK | OK | **clean** |
| `flaw.corrupted_abilities` | 5847-5852 | uncomputed_rule ✓ | OK — "only once" enforced by `max_per_target`'s default ✓ | OK ✓ | **clean**; ArMDE:2960 instance |
| `flaw.corrupted_arts` | 5853-5858 | creation_effect → uncomputed_rule | **no effects at all, while `RULES.md:5226-5229` records its mechanics as in-play** | the whole ±3 / ±5 xp swing in neither locale | **F-393** **Q-93** |
| `flaw.corrupted_spells` | 5859-5864 | in_play_effect ✓ | the 30-level threshold is inexpressible (D3) | the threshold, the ±3, the ±5 xp and the spell-forgetting rule in neither locale | F-394 |
| `flaw.covenant_upbringing` | 5865-5868 | creation_effect ✓ | `ability_authorization` correct; the "any dead language" width is a **recorded** approximation (`RULES.md:6360-6365`) ✓ | the native-language bar and the covenant-dialect allowance in neither locale | F-395 |
| `flaw.craving_for_travel` | 5869-5872 | uncomputed_rule ✓ | OK | OK ✓ | **clean** |
| `flaw.creative_block` | 5873-5876 | in_play_effect ✓ | `lab_total_mod -3` correct in sign and size ✓ (D1/D4 read) | the experimentation-dice clause and the Lab-Text condition in neither locale | F-396 |
| `flaw.crippled` | 5877-5880 | narrative → uncomputed_rule | OK | the absolute prohibition in neither locale | F-397 |
| `flaw.curse_of_slander` | 5881-5884 | uncomputed_rule ✓ | `max_total: 1` ✓, dual category ✓; the chosen section of society is recorded by no parameter **?** | OK ✓ | **Q-88**; ArMDE:2960 instance; F-403 (table row) |
| `flaw.curse_of_venus` | 5885-5888 | narrative ✓ | OK | OK | **clean** |
| `flaw.cursed_guile` | 5889-5892 | uncomputed_rule ✓ | OK | OK ✓ | **clean**; ArMDE:2960 instance |
| `flaw.cyclic_magic_negative` | 5893-5896 | in_play_effect ✓ | both `-3`s correct in sign and size ✓ (D1/D4 read); the condition is folded flat, which D4 already owns | the condition, the partial-season rule and the equal-length constraint in neither locale | F-398 **Q-91** |
| `flaw.dark_secret` | 5897-5900 | narrative ✓ | OK | OK | **clean** |
| `flaw.deaf` | 5901-5904 | uncomputed_rule ✓ | OK | OK ✓ | **clean** |
| `flaw.a_deal_with_the_devil` | 5905-5908 | narrative → uncomputed_rule | **the "includes the effects of" composition is encoded nowhere**, where all four other uses of the idiom are encoded | the composition clause in neither locale | **F-406** (sub-agent overturn) |
| `flaw.deficient_form` | 5909-5912 | in_play_effect ✓ | **no `max_total`, so all ten Forms may be taken at once** (ArMDE:2814) | OK — the summary states the halving verbatim in both locales ✓ | F-400 **Q-89** |
| `flaw.deficient_technique` | 5913-5915 | in_play_effect ✓ | **no `max_total`**; **the lone `prerequisites` among 122 hermetic entries**, and its Minor sibling has none | **the halving reaches the user in neither locale** — the summary is authored, not the passage's own sentence; and the name interpolates where the convention says it should not | F-399 F-400 F-402 F-404 **Q-90** |
| `flaw.deleterious_circumstances` | 5917-5920 | in_play_effect ✓ | `circumstantial` correct (A33: no `HalvableTotal` fits) ✓; the circumstance is recorded by no parameter | the halving **does** survive — it is sentence one, so the summary carries it ✓; the circumstance taxonomy does not | F-401 |
| `flaw.delusion` | 5921-5925 | narrative ✓ | OK | OK | **clean** |
| `flaw.demonic_familiar` | 5926-5931 | narrative ✓ **?** | OK | OK | **Q-88** |

**Fully clean: 17** — `close_family_ties`, `clumsy`, `clumsy_magic`,
`compassionate_major`, `compassionate_minor`, `compulsion_major`,
`compulsion_minor`, `compulsive_lying_major`, `compulsive_lying_minor`,
`continence`, `corrupted_abilities`, `craving_for_travel`, `curse_of_venus`,
`cursed_guile`, `dark_secret`, `deaf`, `delusion`.
**With at least one failed or escalated check: 18.**

*(`flaw.a_deal_with_the_devil` was in the clean list in this file's first pass and
was **overturned by the verification sub-agent** — see F-406 and the
reconciliation section. The other seventeen were re-derived independently and
none was overturned.)*

**The seven rows whose `class` column reads `narrative → …` are F-403's
evidence**: `ceremonial_spontaneous_magic`, `chaotic_magic`, `church_upbringing`,
`companion_animal`, `consumed_casting_tools`, `crippled`, and
`a_deal_with_the_devil`. All seven are inside the twice-swept Flaws block and all
seven are green under the live guard today.

## Findings

### F-384 — `flaw.ceremonial_spontaneous_magic` — `narrative` on a passage that states an absolute restriction and an incompatibility

> "You need time and effort to focus your Spontaneous magic. **You can only cast
> Spontaneous spells using the rules for Ceremonial Casting** (page 217). This
> Flaw is not compatible with Difficult Spontaneous Magic or Weak Spontaneous
> Magic." — ArMDE:5783
>
> DE 5783: "Du brauchst Zeit und Aufwand, um deine Spontane Magie zu fokussieren.
> **Du kannst Spontane Zauber nur nach den Regeln für Zeremonielles Zaubern
> wirken** ([Seite 217](#zeremonielles-zaubern)). Dieser Fehler ist nicht mit
> Schwieriger Spontaner Magie oder Schwacher Spontaner Magie vereinbar."

Two mechanical clauses, and `classification: "narrative"` asserts there are none.
The first is an absolute restriction on how one half of a magus's magic works;
following its page reference (ArMDE:9285-9295) shows what the restriction
actually costs, and it is substantial:

> "A maga may spend **fifteen minutes for every magnitude** of the spell
> performing rituals … she may **add her scores in Artes Liberales and
> Philosophiae to her Casting Score**. A maga may use ceremonial casting even if
> she has no experience points in one of the two Abilities, **but not if she has
> no experience points in either**." — ArMDE:9287
>
> "the level of the spell is limited by the time the maga spent in casting it,
> **to one magnitude per fifteen minutes**." — ArMDE:9289

So the Flaw makes every Spontaneous cast cost real time, gives it a compulsory
bonus from two Academic Abilities, caps its level by elapsed time, and — through
ArMDE:9287's last clause — makes a magus with *no* Artes Liberales and *no*
Philosophiae unable to cast Spontaneous magic at all. The entry's displayed text
is its summary alone ("You need time and effort to focus your Spontaneous
magic."), which is the passage's flavour sentence; none of the above reaches the
player in either locale.

**Verdict:** `narrative` → **`uncomputed_rule`**, with the restriction and the
incompatibility written into `description` in both locales. The three
ceremonial-casting clauses behind the page reference belong there too (D5), or at
minimum the pointer to them must survive.

**Why the swept-block guard is green on it:** the passage contains no signed
number, no botch term, and none of the twenty `MECHANICAL_PHRASES`. See F-403.

### F-385 — two stated incompatibilities, encoded on none of the three sides

ArMDE:5783 (quoted above) names two pairs. `jq` over
`rules/core/virtues_flaws.json` returns **no `incompatible_with` key at all** on
any of the three entries involved:

| entry | ArMDE | `incompatible_with` |
|---|---|---|
| `flaw.ceremonial_spontaneous_magic` | 5781-5784 | *(absent)* |
| `flaw.difficult_spontaneous_magic` | 5966-5971 | *(absent)* |
| `flaw.weak_spontaneous_magic` | 7084-7089 | *(absent)* |

`incompatible_with` is symmetry-enforced at load
(`ruleset/integrity.rs::validate_incompatibility_symmetry`), so the fix is four
edges across three entries: Ceremonial ↔ Difficult and Ceremonial ↔ Weak.

**Both targets were opened, and reading them sharpens the finding rather than
merely confirming it.** Difficult and Weak Spontaneous Magic are *explicitly
compatible with each other*, stated twice, once from each end:

> "This Flaw may be combined with Weak Spontaneous Magic (page 153) to create a
> magus who cannot use Spontaneous magic at all." — ArMDE:5970
>
> "This Flaw may be combined with Difficult Spontaneous Magic (page 126) to
> create a magus who cannot cast spontaneous magic at all." — ArMDE:7088

So their empty `incompatible_with` is **correct**, and only the two edges to
Ceremonial are missing. That also means this is not a case where one blanket
"these three don't mix" rule would do.

*Scope of the negative, stated exactly as checked:* I queried those three ids and
those three only. I did **not** claim anything about the catalogue at large —
`jq '[.[] | select(.incompatible_with != null)] | length'` reports **81**
populated entries, so "the data encodes no incompatibilities" would be false.

**Severity: high.** `validation/prereq.rs::validate_incompatibilities` is the
only consumer and it emits a hard `incompatible` error; with no edge, a character
sheet that the rulebook forbids validates clean in Enforced mode.

### F-386 — `flaw.chaotic_magic` — `narrative` on a spontaneous-casting procedure with a numeric tolerance

> "When you cast a spontaneous spell, **you must specify a desired level of
> effect**. If you fall short of or exceed that target **by more than one
> level**, the spell still works, but its effects are beyond your control—the
> storyguide decides the results. **The level of effect includes any levels you
> assign to Penetration.**" — ArMDE:5787
>
> DE 5787: "Wenn du einen Spontanzauber wirkst, musst du eine gewünschte
> Effektstufe angeben. Unterschreitest oder überschreitest du diese Zielstufe **um
> mehr als eine Stufe**, funktioniert der Zauber zwar, aber seine Wirkungen liegen
> außerhalb deiner Kontrolle – der Spielleiter entscheidet über die Ergebnisse.
> **Die Effektstufe schließt alle Stufen ein, die du der Penetration zuweist.**"

This is a procedure with a declared target, a ±1-level tolerance band, a
consequence on failure, and an explicit accounting rule for how Penetration
levels count toward the target. The last sentence in particular is a pure
bookkeeping rule of the kind nothing but a rulebook states. The engine computes
none of it — GM judgement decides the out-of-control result — which is exactly
`uncomputed_rule`'s definition, not `narrative`'s.

**Verdict:** `narrative` → **`uncomputed_rule`**, description in both locales.
Displayed text today is the summary "Your magic is very wild." alone.

### F-387 — `flaw.church_upbringing` — a mandatory 25-experience-point allocation classified as flavour

> "**The player must spend 25 experience points from the normal budget on Artes
> Liberales, Latin, Music, Organization Lore: Church, or Theology. Unless the
> character has a Virtue that permits it, no other experience points may be spent
> on Academic Abilities.**" — ArMDE:5791
>
> DE 5791: "**Der Spieler muss 25 Erfahrungspunkte aus dem normalen Budget für
> Artes Liberales, Latein, Musik, Ordenskunde: Kirche oder Theologie ausgeben.
> Sofern der Charakter keine Tugend besitzt, die es erlaubt, dürfen keine weiteren
> Erfahrungspunkte für Akademische Fertigkeiten ausgegeben werden.**"

Two hard rules — a compulsory allocation of a quarter of a grog's entire starting
experience, and a category bar — and the entry carries `classification:
"narrative"`, no `effects`, and no `description`. The displayed text is the
summary, which is the passage's *first* sentence ("This Flaw covers children
schooled by the Church…"); the rules are in sentence four. That is precisely the
sentence-order failure `uncomputed_clauses.rs`' module doc was written about.

**The encoding is genuinely hard, and the obvious fix is wrong.** The engine's
`RestrictedAbilityXp` (A7) *grants* a pool: "One `FlowPool` per effect
occurrence, capacity `amount`". Its 28 shipped users all cite passages that
**grant** experience — `virtue.privileged_upbringing` ArMDE:4808 "You have an
**additional** 50 experience points", `flaw.feral_upbringing` ArMDE:6112 "In your
first five years you **gain** 120 experience points". Church Upbringing says the
opposite: 25 points **from the normal budget**. Wiring
`restricted_ability_xp: 25` would hand the character 25 free points and turn a
Flaw into a benefit. No effect variant earmarks *existing* budget, so **D3
applies**: `uncomputed_rule`, with the rule written out in both locales.

**Verdict:** `narrative` → **`uncomputed_rule`**. See **Q-92** for the encoding
question and **F-388** for the half that *is* expressible and is a live bug.

### F-388 — `flaw.church_upbringing` — no `ability_authorization`, so doing what the Flaw *requires* is a hard validator error

`rules/core/abilities.json` sets
`"categories_requiring_virtue": ["academic", "arcane", "martial"]`. Three of the
five Abilities Church Upbringing compels a spend on are Academic:

| passage term | catalogue id | category |
|---|---|---|
| Artes Liberales | `ability.artes_liberales` | **academic** |
| Latin | `ability.dead_language` (parameter `language`) | **academic** |
| Theology | `ability.theology_christian` | **academic** |
| Music | `ability.music` | general |
| Organization Lore: Church | `ability.organization_lore` | general |

`validation/authorization.rs::validate_ability_authorization` errors
`ability_category_requires_virtue` on a held Ability in a gated category unless an
`AbilityAuthorization` or a `RestrictedAbilityXp` names its id or its category,
with a whole-character exemption only for `profile.is_magus`. Church Upbringing
is `*Minor, Personality*`, so grogs and companions may take it and the exemption
is empty.

**This is confirmed by the engine's own test, not inferred.**
`authorization.rs::an_authorizing_virtue_permits_one_ability_without_granting_xp`
builds a companion carrying an entry shaped exactly like
`flaw.covenant_upbringing` (`ability_authorization: ["ability.dead_language"]`),
buys `ability.artes_liberales`, and asserts
`gate_issues(&validate(&broader, &rs())).len() == 1`. So a companion with Church
Upbringing who spends the compelled 25 points on Artes Liberales gets one hard
error today, and there is no legal way to obey the Flaw.

ArMDE:2315, quoted in that function's own doc comment, anticipates exactly this
entry — "other Virtues (**and some Flaws**) also grant access to some of these
Abilities".

**And the fix must not over-permit**, which is what makes this more than a
data-entry omission. `Effect::AbilityAuthorization` is a static
`Vec<Id>`/`Vec<AbilityCategory>` with no amount and no `param`, so an
authorization naming those three Abilities permits **unlimited** Academic
spending on them, where the passage permits exactly 25 points' worth and then
bars the category. This is the `virtue.privileged_upbringing` mirror the audit
already has a name for, reappearing on a Flaw whose own text spells the bar out.
See **Q-92**.

**Severity: high** — it is a false hard error that blocks a legal character, and
its only two fixes each break a different rule.

### F-389 — `flaw.companion_animal` — a granted Personality Trait and a species restriction, classified as flavour

> "*Minor, Social Status, **animals only***" — ArMDE:5806
>
> "He is largely dependent on his master for food and shelter, and **has an
> additional Personality Trait of "Loyal to Master"** which represents this
> bond." — ArMDE:5807
>
> DE 5806: "*Klein, Sozialer Status, **nur Tiere***"; DE 5807: "…und **hat eine
> zusätzliche Persönlichkeitseigenschaft „Treu gegenüber dem Herrn"**, die diese
> Bindung widerspiegelt."

Two clauses, neither flavour. An *additional* Personality Trait is a grant with a
named value — the engine models Personality Traits as first-class entity state
(`Entity::personality_traits`, written by `export/magic.rs::write_personality_traits`
and read by `completeness.rs`) — and "animals only" is an eligibility rule on the
descriptor line itself, alongside the magnitude and category the data *does* read.

**Both are inexpressible, and that is D3's case, not `narrative`'s.** I checked
all 42 variants of `types.rs::Effect`: none grants a Personality Trait
(`grep -rn "GrantsPersonalityTrait\|personality_trait_grant" crates/arm-rules/src`
returns nothing, and the variant list at `types.rs` lines 852-1475 contains no
such member). And `types.rs::EntityKind` has exactly two members, `Character` and
`Covenant`, with no animal among the four profiles in
`rules/core/character_types.json` — so "animals only" has nothing to bind to
either.

**Verdict:** `narrative` → **`uncomputed_rule`**, both clauses in both locales.
Also an **ArMDE:2816** instance: it is a Social Status stating no compatibility.

### F-390 — `flaw.consumed_casting_tools` — a House restriction that is exactly expressible and absent

> "**This Flaw may only be taken by Verditius magi.**" — ArMDE:5841
>
> DE 5841: "**Dieser Fehler darf nur von Verditius-Magi genommen werden.**"

The entry carries `prerequisites: null`. `Prereq::House` exists, `house.verditius`
exists in `rules/core/houses.json`, and the exact composition this sentence needs
is already shipped on another entry: `flaw.primogeniture_lineage` carries
`{"kind":"all","value":[{"kind":"is_magus"},{"kind":"house","value":"house.verditius"}]}`.

**This is corroborated from an unusual direction, and that is what makes it
certain rather than arguable.** `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`
carries a written-down reading for `flaw.primogeniture_lineage` which says, of the
*identical sentence*:

> "The one genuinely mechanical clause, \"This Flaw can only be taken by magi of
> House Verditius\", is already *computed*: the entry carries
> `prerequisites: all(is_magus, house.verditius)`, so the rule is enforced rather
> than merely described"

So the repository has already ruled that this sentence is a computed rule and has
already written the expression for it. Consumed Casting Tools states the same
sentence, about the same House, and encodes nothing — while being classified
`narrative`, which asserts the sentence is not a rule at all.

**Severity: high.** `validate_prerequisites` emits a hard `prereq_not_met` error;
with no prerequisite, a Jerbiton magus — or a Gifted companion, since
`character_types.json` permits `hermetic` to a companion holding `virtue.the_gift`
— may take a Flaw the book reserves to one House, and nothing objects.

### F-391 — `flaw.consumed_casting_tools` — four further mechanical clauses reach neither locale

Beyond the House restriction, ArMDE:5841 states: casting tools are consumed on
use; **"It takes an hour to make a new casting tool"**; regular casting tools may
not be duplicated, so the magus **"may not stock up"**; making a tool for a spell
that already has one **erases the magical connection from the existing tool**;
and **"The magus does not need a laboratory to make casting tools."** The DE line
carries all five identically.

The entry's displayed text is its summary — the first sentence only. Everything
after it reaches the player nowhere, in either locale. D5 obliges all of it in
`description` once the entry is reclassified.

### F-392 — `flaw.the_constant_expression` — six mechanical clauses in neither locale, and `RULES.md` records that they are there

This is the batch's sharpest finding, because the divergence is between the data
and the repository's own traceability map.

`crates/arm-rules/RULES.md:4895-4915` explains why this entry carries
`special_casting_mod { circumstantial }` rather than `lab_total_mod −3` — Safety
is a laboratory Characteristic, not a Lab-Total term — and that reasoning is
correct and is corroboration, not a finding. What it then says is not true:

> "the item carries `special_casting_mod { circumstantial }` (surfaced) and **its
> own rules text carries the −3 Safety plus the Warping-Score botch dice on
> Ritual/Ceremonial casting** (`ArMDE:5829`)."

`jq` over `rules/i18n/en/virtues_flaws.json` and `rules/i18n/de/virtues_flaws.json`
returns the key set `["name","summary"]` for `flaw.the_constant_expression` in
**both** locales. There is no `description`, and the summary is the passage's
first sentence ("The maga forms the center of a magical maelstrom, perpetually
casting non-fatiguing Spontaneous magic effects."). The rules text carries
neither the Safety penalty nor the botch dice.

**What is actually lost** (ArMDE:5821-5838, eighteen lines — the longest passage
in the batch):

| ArMDE | clause | DE line |
|---|---|---|
| :5823 | "the maga is considered to have **lost one long-term Fatigue level which cannot be regained through sleep**" | 5823 "gilt als eine dauerhafte Erschöpfungsstufe verloren zu haben, die nicht durch Schlaf zurückgewonnen werden kann" |
| :5827 | suppression costs "an initial **Stamina + Concentration simple roll against Ease Factor 3 + (Warping Score)**"; on failure "a further roll … **every two minutes**" | 5827 "Ausdauer + Konzentration-Einfachwurf gegen Schwierigkeitsgrad 3 + (Verzerrungswert)" |
| :5829 | Ceremonial or Ritual casting needs that Concentration roll, "**One attempt … every 15 minutes**", and the maga "suffers a number of **additional botch dice equal to her Warping Score**" | 5829 "alle 15 Minuten"; "zusätzlicher Patzerwürfel in Höhe ihres Verzerrungswerts" |
| :5831 | "Any laboratory the maga works in is treated as having a free Flaw providing a **Safety penalty of –3**" | 5831 "Sicherheitsstrafe von –3" |
| :5833 | "**Where the penalty reduces the casting total to 0, no magic is expressed.**" | 5833 "Wo die Strafe den Zaubersumme auf 0 reduziert, wird keine Magie ausgedrückt." |
| :5825 | the effects may target anything the maga could target with a Spontaneous spell, herself included | 5825 |

The lost Fatigue level is itself inexpressible: `HealthTrack` has five members
(`fatigue_penalty`, `wound_penalty`, `fatigue_roll`, `casting_fatigue`,
`recovery`) and none represents a permanently-lost long-term level. D3 keeps the
entry's *class* as `in_play_effect` — it does carry an effect — and D5 obliges
every one of the six clauses into `description` in both locales.

**Two things to fix, not one:** the missing text, and `RULES.md`'s claim that it
is present. A traceability map that records a mitigation nobody landed is worse
than one that records the gap.

### F-393 — `flaw.corrupted_arts` — `creation_effect` with no effects, on a passage `RULES.md` itself calls in-play

The entry is `classification: "creation_effect"` with no `effects` key, no
`parameters`, and no `description` in either locale. It is one of the five
entries `README.md` names as effect-less `creation_effect`s, and C7 explains why
`data_integrity.rs::every_vf_is_classified` is green on it — the guard asserts
that an `in_play_effect` entry carries an effect, never the converse.

But `crates/arm-rules/RULES.md:5226-5229` already reached the opposite
conclusion about the rulebook:

> "**Savantism, Simple Student, Corrupted Arts (3 XP)** — … **Corrupted Arts
> (ArMDE:5853) has no creation XP figure (its ±3 casting swing / ±5 Art xp are
> in-play).**"

So the traceability map records that this entry's mechanics are in-play and that
there is no creation number, while `classification` claims it changes a
character-creation number or state. One of the two is wrong, and the passage
settles it — every clause is contingent on a use of the Art in play:

> "Use of a corrupted Art for a selfish or sinful action receives a **+3 bonus**
> to the character's Casting Total. Succeeding in a roll because of this bonus …
> means he immediately acquires **5 experience points** in that Art. However,
> uses of the Art that are neutral or selfless receive a **-3 penalty**, and if
> you fail the roll because of this penalty, he immediately **loses 5 experience
> points** in the Art." — ArMDE:5855

**And the three siblings disagree with each other**, which is the strongest
evidence that nobody has ever compared them. The book states the *same mechanic*
three times over, once for Abilities, once for Arts and once for Spells, in
near-identical wording (ArMDE:5849, :5855, :5861), and the catalogue treats each
differently:

| entry | ArMDE | classification | effects | description |
|---|---|---|---|---|
| `flaw.corrupted_abilities` | 5847-5852 | `uncomputed_rule` | none | **both locales ✓** |
| `flaw.corrupted_arts` | 5853-5858 | **`creation_effect`** | **none** | **neither** |
| `flaw.corrupted_spells` | 5859-5864 | **`in_play_effect`** | `special_casting_mod { circumstantial }` | **neither** |

**And `RULES.md` contradicts itself about this entry**, which is the third reason
it needs re-reading rather than trusting the map. The verification sub-agent
independently reached this same finding and then *withdrew* it on the strength of
`RULES.md:5411`; the withdrawal was correct procedure applied to a document that
disagrees with itself two hundred lines apart:

> `RULES.md:5411` — "`flaw.corrupted_arts` (ArMDE:5853-5858) — **grants XP swing
> at creation** + situational casting"
>
> `RULES.md:5226-5229` — "**Corrupted Arts (ArMDE:5853) has no creation XP figure
> (its ±3 casting swing / ±5 Art xp are in-play).**"

The first is an index line in the deferral list; the second is that same
deferral's stated rationale. They say opposite things about whether a creation
number exists. ArMDE:5855 settles it — every clause there fires on a *use* of the
Art — so the rationale is right and the index line is wrong. **Whichever way the
classification is corrected, one of those two `RULES.md` lines must change too.**

**Verdict:** `creation_effect` → **`uncomputed_rule`** (it carries no effect, so
it cannot be `in_play_effect`; and the book plainly states a rule, so it cannot
be `narrative`), with the passage written into `description` in both locales.
Whether the three should instead be unified the *other* way — all three
`in_play_effect` with a `circumstantial` marker — is **Q-93**.

### F-394 — `flaw.corrupted_spells` — the whole passage reaches neither locale

The entry's `special_casting_mod { circumstantial }` is surfaced-only by design
(C1, `RULES.md:5141`), so the displayed rules text is the rule's only carrier —
and it is the summary alone. Lost in both locales (ArMDE:5861, :5863; DE 5861,
5863):

- the **prerequisite**: "has learned **at least 30 levels** of formulaic spells
  from a source that has been corrupted" — `Prereq` has no "holds ≥ N levels of
  formulaic spells" variant, so D3 keeps it as text, but it must *be* text;
- **+3 to Casting Total *and* Penetration Total** for selfish use — note this is
  a wider bonus than Corrupted Arts' Casting-Total-only +3, which is exactly the
  kind of distinction a shared summary would erase;
- **5 experience points toward mastery of that spell** on a success owed to the
  bonus;
- **−3** for neutral or selfless use, and **5 experience points lost** on a
  failure owed to the penalty — *or on fatiguing himself*, a trigger the Arts and
  Abilities versions do not have;
- "**If this would result in negative experience, he forgets the spell
  completely.**" — an absolute consequence stated nowhere else in the catalogue;
- "You may only take this Flaw once, though it can affect as many of the
  character's spells as you wish."

D5 obliges all of it.

### F-395 — `flaw.covenant_upbringing` — the two clauses the authorization does not cover reach neither locale

The entry's `ability_authorization: ["ability.dead_language"]` is **correct**, and
its width is a **recorded** approximation, not a defect — `RULES.md:6360-6365`:

> "`flaw.covenant_upbringing` gains `ability_authorization: [ability.dead_language]`
> for 'You may take Latin at character creation' (`ArMDE:5867`). **Documented
> approximation:** authorization is by ability id, so this permits any dead
> language, not Latin alone".

*(I raised this as a suspected over-permission on first reading and withdraw it
on that evidence. It is also the precedent `RULES.md:5249-5263` cites for
`virtue.student_of_realm`, so it is load-bearing rather than incidental.)*

What is **not** covered is the rest of ArMDE:5867:

> "**While Latin cannot be your native language**, you may speak a language
> closely related to Latin that is spoken only at your home covenant."
>
> DE 5867: "**Während Latein nicht deine Muttersprache sein kann**, darfst du
> eine Sprache sprechen, die eng mit Latein verwandt ist und nur in deinem
> Heimatkonvent gesprochen wird."

Half of this is already satisfied by construction and half is not, and the
distinction is worth recording so a correction pass does not build machinery it
does not need. `rules/core/life_stages.json:21` sets
`"native_language_ability": "ability.living_language"`, and `ability.latin` does
not exist — Latin is an instance of `ability.dead_language`. So on the life-stage
path a native language is structurally a Living Language and Latin **cannot** be
one. On the flat `Entity::xp_pool` path there is no native language at all, so
there is nothing to violate either. The bar is therefore unenforceable-because-
unnecessary, and the *allowance* — a covenant-only Latin-adjacent language the
character may speak — is a real permission that the engine models nowhere and
that the player is never told about.

**Severity: low.** `creation_effect` is the right class (the authorization does
change creation legality), so this is a D5 text obligation only.

### F-396 — `flaw.creative_block` — the experimentation clause and the condition reach neither locale

**Directed read, D4. Transcribed verbatim, both languages:**

> **EN ArMDE:5875** — "You have problems creating new things in the lab. You take
> a –3 penalty to all Lab Totals **unless you are using a Laboratory Text or being
> taught**. If you experiment, roll twice as many dice on the experimentation
> table."
>
> **DE 5875** — "Du hast Probleme damit, neue Dinge im Labor zu erschaffen. Du
> erleidest eine –3-Strafe auf alle Laborsummen, **es sei denn, du verwendest
> einen Labortext oder wirst unterrichtet**. Wenn du experimentierst, würfle
> doppelt so viele Würfel auf der Experimentiertabelle."

**The data is correct and is not marked wrong.** `lab_total_mod { amount: -3 }`
matches the passage in sign and size; the sign is right against what a Lab Total
counts (higher is better, so a penalty is negative). D1 rules the condition
deliberately ignored in `effective/spell.rs::spell_level_cap`; D4's table rules
that the condition **holds** in a character-generation Lab Total ("not using a Lab
Text, not being taught" → yes, −3), and reading the passage confirms that reading
exactly: at creation the character is neither working from a Lab Text nor being
taught, so the penalty applies. D4 is right on this entry.

**What is a finding is the text.** The entry carries no `description` in either
locale and its summary is the flavour sentence, so two things reach the player
nowhere:

1. the **condition itself** — a player who has Creative Block is never told the
   −3 lifts when working from a Lab Text or being taught, which is the entire
   point of the Flaw;
2. **"If you experiment, roll twice as many dice on the experimentation table"** —
   a second, independent mechanic on a different channel, mirrored by
   `virtue.inventive_genius`' "+6 if you experiment" (ArMDE:4153).

D5 obliges both in both locales.

### F-397 — `flaw.crippled` — an absolute prohibition classified as flavour

> "You either have no legs, or your legs are completely useless. … **You cannot
> walk**, although you may drag yourself along the ground, or push a trolley
> around with a stick." — ArMDE:5879
>
> DE 5879: "**Du kannst nicht gehen**, obwohl du dich am Boden entlangziehen oder
> mit einem Stock einen Karren voranschieben kannst."

An absolute prohibition on an action, with two named exceptions. The
`uncomputed_rule` definition in `types.rs::Classification` names an "absolute"
explicitly as a rule shape, and `MECHANICAL_PHRASES`' own "Absolutes" section
exists for exactly this — it just happens to carry only `cannot die`.

**The parity is direct, and it is B10's, not mine.** B10's F-369 reclassified
`flaw.blind` (ArMDE:5719-5722) `narrative` → `uncomputed_rule` on "reading is
impossible", "Using missile weapons is futile" and "blind magi cannot aim spells
without magical aid" — the same shape, one entry-block away, in the same swept
block. Crippled is the mobility twin of that entry and states its prohibition
more flatly, not less.

**Verdict:** `narrative` → **`uncomputed_rule`**, description in both locales.

### F-398 — `flaw.cyclic_magic_negative` — three clauses reach neither locale

**Directed read, D4. Transcribed verbatim, both languages:**

> **EN ArMDE:5895** — "As with the Hermetic Virtue, your magic is attuned to some
> cycle of nature and is less potent at specific times. You have a –3 penalty to
> all Lab Totals and Casting Scores **during that time**. **The penalty applies to
> Lab Totals even if the negative period does not cover the whole of the season.**
> **The length of time during which you are at a disadvantage must be equal to the
> time when there is no penalty.**"
>
> **DE 5895** — "Wie bei der hermetischen Tugend ist deine Magie auf einen
> Naturzyklus abgestimmt und zu bestimmten Zeiten weniger potent. Du hast eine
> –3-Strafe auf alle Laborsummen und Zauberwerte **während dieser Zeit**. **Die
> Strafe gilt für Laborsummen, auch wenn die negative Phase nicht das gesamte
> Quartal umfasst.** **Der Zeitraum, in dem du im Nachteil bist, muss gleich lang
> sein wie der Zeitraum ohne Strafe.**"

**The data is correct and is not marked wrong.** `casting_total_mod { -3, all }`
and `lab_total_mod { -3 }` match the passage in sign, size and breadth ("all Lab
Totals and Casting Scores"), and mirror `virtue.cyclic_magic_positive`'s `+3`/`+3`
exactly. The unconditional folding is D4's subject, not this entry's defect.

**What is a finding is the text** — no `description` in either locale, summary is
the flavour sentence, so three clauses reach the player nowhere:

1. that the −3 applies **only during the attuned period** — without which the
   printed totals look like a flat permanent penalty;
2. the partial-season rule (third sentence), which is a specific carve-out for
   Lab Totals and for nothing else;
3. the equal-length constraint (fourth sentence), which is the only thing
   stopping a player from declaring a one-day negative period.

The third sentence also bears on D4's Phase 2 ruling — see **Q-91**.

### F-399 — `flaw.deficient_technique` — the halving reaches the user in neither locale, where its Minor sibling's does

The two Deficient Art entries state the same mechanic and their shipped text does
not. `flaw.deficient_form`'s summary is the passage's own first sentence:

> EN: "Almost all totals (including Casting Totals and Lab Totals, but excluding
> Magic Resistance) to which a particular Form is added are halved."
> DE: "Fast alle Gesamtwerte (einschließlich Zaubersummen und Laborsummen, jedoch
> ausgenommen Magieresistenz), zu denen eine bestimmte Form addiert wird, werden
> halbiert."

`flaw.deficient_technique`'s summary is **authored prose that appears nowhere in
the book**, in both locales:

> EN: "You are unable to use one Technique effectively."
> DE: "Du bist unfähig, eine Technik effektiv einzusetzen."

where ArMDE:5915 says "All totals, including Lab and Casting totals, including a
particular Technique **are halved**. Advancement Totals are **not** halved.
Experience points required are based on the actual value of the Technique, before
halving."

So the *Major*, more expensive, more consequential of the two Flaws tells the
player less than the Minor one: neither the halving, nor its exclusions, nor the
XP rule survives, in either language. The engine computes the halving correctly
(A32), so this is a text defect and not a wrong number — but the player reading
the sheet is told only that something is "ineffective".

**Fix:** replace the authored summary with the passage's own sentence (matching
its sibling's convention), and add the exclusions and the XP rule as a
`description` under D5 on both entries.

### F-400 — `flaw.deficient_form` and `flaw.deficient_technique` — no `max_total`, so every Form and every Technique may be deficient at once

> "A Virtue or Flaw may be taken more than once **only if the description
> explicitly allows it**. Most Virtues and Flaws may only be taken once." —
> ArMDE:2814

Neither passage says it may be taken more than once. ArMDE:5911 says "a
particular Form"; ArMDE:5915 says "a particular Technique". Both entries carry a
parameter (`form` / `technique`) and **no `max_total`**, and `max_per_target`
defaults to 1 grouped by `(item_ref, the whole params map)` — so two copies
naming two different Arts are two different tuples and both are legal. A
character may today take Deficient Form ten times (all ten Forms) for ten Flaw
points, and Deficient Technique five times (all five Techniques) for fifteen.

This is exactly the shape B10 found on `flaw.anchored_to_the_land`,
`flaw.bound_to_realm` and `flaw.bound_to_role_role` (its F-358); these are two
further instances, in a different part of the book, which makes it a pattern
rather than an accident. The catalogue does know the fix —
`flaw.curse_of_slander` in this very batch carries `max_total: 1`.

**Severity: moderate.** It is a missing hard cap in a budget-bearing validator
(`validate_total_selection_cap` → `too_many_selections`), so an illegal character
validates clean and the balance arithmetic accepts the Flaw points.

### F-401 — `flaw.deleterious_circumstances` — the circumstance taxonomy reaches neither locale, and no parameter records the chosen circumstance

> "**All your magic totals, excluding Magic Resistance, are halved** under certain
> uncommon circumstances. This can be your state, such as sitting or wet, the
> target of the magic, such as wild animals or iron, or the place where you are
> casting the magic, such as a city or high up a mountain." — ArMDE:5919
>
> DE 5919: "**Alle deine Magie-Gesamtwerte, ausgenommen Magieresistenz, werden
> unter bestimmten seltenen Umständen halbiert.** …"

**The effect is correct.** `special_casting_mod { circumstantial }` is
surfaced-only, and A33 confirms there is no alternative: `HalvableTotal` has four
members (`spontaneous_casting`, `lab_enchanting`, `lab_longevity`, `penetration`)
and none means "every magic total". `RULES.md:4869` records the choice. Not a
data defect.

**Two things are.** First, D5: the halving — the entire mechanic — reaches the
player only through the *summary*, which here happens to be the halving sentence,
so the rule does survive in one sentence. That is luck rather than design (the
sentence order fell the right way), and the *taxonomy* of circumstances, which is
what a player needs to pick one, does not survive. Second, the chosen
circumstance is recorded by **no parameter**, so two magi with this Flaw are
indistinguishable in the save file and the export prints no circumstance at all.
That is the same shape B10 raised as its F-362 on `flaw.baneful_circumstances`
and folded into its Q-86; it recurs here, and `flaw.curse_of_slander` in this
batch has a third instance of it ("centered on one specific section of mundane
society", ArMDE:5883) — see **Q-88**.

### F-402 — `flaw.deficient_technique` carries the only `has virtue.hermetic_magus` prerequisite in 122 Hermetic entries, and its sibling carries none

`jq` over `rules/core/virtues_flaws.json`:

- entries carrying the category `hermetic`: **122**
- of those, entries carrying any `prerequisites`: **8**
- of those 8, the complete list:

| id | prerequisites |
|---|---|
| `flaw.blatant_gift` | `has virtue.the_gift` |
| `flaw.suppressed_gift` | `has virtue.the_gift` |
| `virtue.gentle_gift` | `has virtue.the_gift` |
| `virtue.faerie_magic` | `house house.merinita` |
| `virtue.heartbeast` | `house house.bjornaer` |
| `virtue.the_enigma` | `house house.criamon` |
| `virtue.verditius_magic` | `house house.verditius` |
| **`flaw.deficient_technique`** | **`has virtue.hermetic_magus`** |

Seven of the eight gate on something the *passage* names — a Gift, a House.
`flaw.deficient_technique` is the only entry in the catalogue that gates on being
a Hermetic magus, and ArMDE:5915 does not say so. The other 121 Hermetic entries
rely on the category gate in `rules/core/character_types.json`, which is **not**
magus-only: `companion` permits `hermetic` conditionally
(`{"category":"hermetic","when":{"kind":"has","value":"virtue.the_gift"}}`), so a
Gifted companion may legally hold Hermetic V/F.

The consequence is a live asymmetry between two entries that state the same rule:
a Gifted companion may take **Deficient Form** (Minor) and may **not** take
**Deficient Technique** (Major), with nothing in either passage to justify the
split. One of the two shapes is wrong catalogue-wide, and which one is **Q-90**.

*Scope of this claim, stated exactly as checked:* I queried the `hermetic`
category and the `prerequisites` key over `rules/core/virtues_flaws.json` only. I
did not audit prerequisites on non-Hermetic entries, and I make no claim about
them.

### F-403 — the swept-block screen missed seven entries, and the exact word-forms it lacks

This is B10's F-383 recurring with a fresh set of word-forms, and it is the
deliverable most useful beyond this batch. All seven entries below are inside
`SWEPT_BLOCKS`' ArMDE:5639-7113, swept twice, and
`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` is green on every
one — because `states_a_mechanical_rule` is `has_signed_number ||
has_botch_term || has_mechanical_phrase`, and none of the six passages contains a
signed number, a botch term, or any of the twenty `MECHANICAL_PHRASES`.

| entry | ArMDE | the wording that states the rule | nearest phrase already on the list |
|---|---|---|---|
| `flaw.ceremonial_spontaneous_magic` | :5783 | "You **can only** cast Spontaneous spells **using the rules for** Ceremonial Casting" | — the "Absolutes" section holds only `cannot die` |
| `flaw.ceremonial_spontaneous_magic` | :5783 | "This Flaw **is not compatible with** Difficult Spontaneous Magic or Weak Spontaneous Magic" / DE "**ist nicht … vereinbar**" | none — there is no incompatibility idiom on the list at all, though the books state one repeatedly (B10 found four more) |
| `flaw.chaotic_magic` | :5787 | "**by more than one level**" / DE "**um mehr als eine Stufe**" | `one magnitude` / `eine magnitude` — **one word away**: the list counts steps in magnitudes and the book here counts them in *levels* |
| `flaw.chaotic_magic` | :5787 | "you **must specify a desired level of effect**" | — |
| `flaw.church_upbringing` | :5791 | "**must spend 25 experience points from the normal budget on**" / DE "**muss 25 Erfahrungspunkte aus dem normalen Budget für … ausgeben**" | none. The detector deliberately requires a *sign* ("1 pawn of vis each season" is its own documented non-rule), and a compulsory allocation carries no sign |
| `flaw.church_upbringing` | :5791 | "**no other experience points may be spent on** Academic Abilities" | `no more than` — **one word away**, and the list's `may not be greater than` shows the "may not" shape is already considered |
| `flaw.companion_animal` | :5806, :5807 | "***animals only***" (on the descriptor line) and "**has an additional Personality Trait of** 'Loyal to Master'" | — |
| `flaw.consumed_casting_tools` | :5841 | "This Flaw **may only be taken by** Verditius magi" / DE "**darf nur von** Verditius-Magi **genommen werden**" | none — and this is the form `NO_RULE_DESPITE_TOKEN`'s own `flaw.primogeniture_lineage` row already calls "genuinely mechanical" |
| `flaw.consumed_casting_tools` | :5841 | "**It takes an hour to** make a new casting tool"; "he **may not** stock up" | — |
| `flaw.crippled` | :5879 | "**You cannot walk**" / DE "**Du kannst nicht gehen**" | `cannot die` / `nicht sterben` — **one verb away**, and the German side is not even discontinuous here, so the substring screen could carry it |
| `flaw.a_deal_with_the_devil` | :5907 | "This Flaw **includes the effects of** Plagued By Supernatural Entity" / DE "**schließt die Auswirkungen von** … **ein**" | none — an item-transfer idiom, a third new family alongside the House restriction and the incompatibility. *Added by the verification sub-agent; see F-406.* |

**Four of the eleven rows are a single word-form from a phrase already on the
list** (`one level` vs `one magnitude`; `may be spent on` vs `may not be greater
than`; `cannot walk` vs `cannot die`; and `nicht gehen` vs `nicht sterben`, which
the module doc's own note about discontinuous German negation would not block).
The three genuinely new families the screen has no member of are **"may only be
taken by \<House\>"**, **"is not compatible with \<entry\>"** and **"includes the
effects of \<entry\>"** — and the second of those is now the sixth stated
incompatibility the audit has found unencoded, so a phrase for it would pay for
itself twice. The third is worth adding precisely because it is **cheap and
self-limiting**: the sub-agent checked, and `grep -n "includes the effects of"`
over the English core rulebook returns exactly **four** hits (ArMDE:4882, :5115,
:5743, :5907), of which three are already non-`narrative`. So the phrase produces
exactly one new red — the entry it should.

**And the screen is not only a diagnosis here — it is a blocker on the fix.**
This is the part I did not expect and it is worth stating plainly for whoever
runs the correction pass. `uncomputed_clauses.rs` has a *second* assertion,
`every_uncomputed_rule_entry_states_its_rule_in_every_locale`, which is unscoped
and applies the **same** `states_a_mechanical_rule` detector to an
`uncomputed_rule` entry's displayed text in every locale. So reclassifying these
seven out of `narrative` and writing their passages into `description` verbatim
turns the first guard green and the second one **red** — because the passages
contain no token, which is precisely why they were missed.

Running the detector by hand over each proposed description:

| entry | verbatim description would contain | second guard |
|---|---|---|
| `flaw.ceremonial_spontaneous_magic` | nothing from the list — **unless** the D5-complete text also carries the ceremonial-casting clauses, in which case "**one magnitude** per fifteen minutes" (ArMDE:9289) matches `one magnitude` | **green, but only by way of the cross-reference** |
| `flaw.chaotic_magic` | "by more than one level" — `or more` needs the literal two words, `one magnitude` needs "magnitude" | **red** |
| `flaw.church_upbringing` | "25 experience points" is unsigned (the detector's own documented non-rule shape), "no other experience points may be spent" is not `no more than` | **red** |
| `flaw.companion_animal` | "an additional Personality Trait of" | **red** |
| `flaw.consumed_casting_tools` | "may only be taken by", "It takes an hour to" | **red** |
| `flaw.crippled` | "You cannot walk" / "Du kannst nicht gehen" | **red** |
| `flaw.a_deal_with_the_devil` | "includes the effects of Plagued By Supernatural Entity" | **red** |

So **six of the seven reclassifications cannot be landed at all** until
`MECHANICAL_PHRASES` grows, and the seventh only squeaks through on a clause
borrowed from another passage. The module's own doc already anticipates this —
"when the books and the list disagree it is the list that is wrong" — so the
order of work is: extend the phrase list first (with the word-forms tabulated
above), then reclassify. A correction pass that tries it the other way round will
hit a red it cannot fix without rewording shipped rulebook text, which is exactly
the backwards move the `BOTCH_TERMS` history records.

**A D6 row, stated as the brief requires.** `rules/source/de/translation-tables/reputationen.md`
carries, in its `### Fehler / Flaws` table (located by the English key
`Curse of Slander`, not by line number, since the tables are modified in the
working tree):

> `| Curse of Slander | Verleumdungsfluch | Lokal | 1→↑ (–) | Wächst jede Saison; Gerüchte |`

The German *term* is right and matches the rulebook heading ✓. The row's factual
columns are not: ArMDE:5883 says the Reputation is "**usually** a Local
Reputation" (the table hardens it to a flat `Lokal`), and says that **at a
covenant** "The Reputation is gained and increases **in intervals of years rather
than seasons**" — which the note "Wächst jede Saison" contradicts outright. This
is D6's "a terminology table making a factual claim about the rules" shape, the
fourth-of-six category that decision names. **Per the brief, `arm-de-translation`
is outside this repository, so this is stated as: this repository's copy
disagrees with the rulebook.** Whether it is an error to fix in both projects or
a stale copy to re-sync is left to Norbert. **Severity: low** — the file is not
loaded at runtime; its harm is that it is the canonical input for the next agent
writing German rules text. The catalogue **data** is correct: `flaw.curse_of_slander`
carries no `grants_reputation`, which is right, because the Reputation is earned
after a season in play and is not a creation grant.

### F-404 — `flaw.deficient_technique` interpolates its parameter into its name where the catalogue's own convention says it should not, and its sibling does not

Found while verifying F-399 rather than while auditing the entry, and it is the
mirror image of what I first suspected — I began by suspecting
`flaw.deficient_form` of *missing* an interpolation and the evidence points the
other way.

**The catalogue has a clean, exceptionless convention, and it is derived from the
book: a name interpolates its parameter exactly when the `####` heading carries a
`(…)` placeholder.** Checked against every `form`-domain and `technique`-domain
entry in the catalogue — seven and one respectively, listed by
`jq '.[] | select(.parameters != null)'` — plus their headings:

| heading | placeholder? | shipped EN name | shipped DE name |
|---|---|---|---|
| ArMDE:3779 `#### Extractor of (Form) Vis` | yes | `Extractor of {form} Vis` ✓ | `Vis-Gewinner der {form}` ✓ |
| ArMDE:4085 `#### Imbued with the Spirit of (Form)` | yes | `Imbued with the Spirit of {form}` ✓ | `Durchdrungen vom Geist der {form}` ✓ |
| ArMDE:4463 `#### Master of (Form) Creatures` | yes | `Master of {form} Creatures` ✓ | `Meister der {form}-Kreaturen` ✓ |
| ArMDE:6162 `#### (Form) Monstrosity` | yes | `{form} Monstrosity` ✓ | `{form} Missgeburt` ✓ |
| ArMDE:6276 `#### Hunger for (Form) Magic` | yes | `Hunger for {form} Magic` ✓ | `Hunger nach {form}-Magie` ✓ |
| ArMDE:3645 `#### Deft Form` | no | `Deft Form` ✓ | `Gewandte Form` ✓ |
| ArMDE:6142 `#### Flawed Parma Magica` | no | `Flawed Parma Magica` ✓ | `Fehlerhafte Parma Magica` ✓ |
| ArMDE:6346 `#### Limited Magic Resistance` | no | `Limited Magic Resistance` ✓ | `Begrenzte Magieresistenz` ✓ |
| ArMDE:5909 `#### Deficient Form` | no | `Deficient Form` ✓ | `Defizitäre Form` ✓ |
| ArMDE:5913 `#### Deficient Technique` | **no** | **`Deficient {technique}`** ✗ | **`Defizitäre {technique}`** ✗ |

Nine of ten follow the rule; `flaw.deficient_technique` is the single exception,
in **both** locales. The book gives its heading exactly the shape it gives
Deficient Form's, so there is no source-side reason for the two siblings to
render differently — and today a character with three Deficient Forms shows three
rows reading "Deficient Form" while one Deficient Technique shows "Deficient
Creo".

**Which way to resolve it is not mine to pick** — interpolating is arguably the
more useful rendering and would mean changing five other entries plus Deficient
Form; not interpolating is the convention and means changing one. **What is not
defensible is the pair disagreeing**, since they are the same mechanic split by
Art class.

*One thing I checked and withdrew:* I suspected `flaw.deficient_technique` of
additionally lacking a `name_unfilled` fallback for an interpolating name. It
does lack one — and so do **39 of the 42** interpolating names in
`rules/i18n/en/virtues_flaws.json`, so that is the catalogue-wide norm and not a
defect of this entry. Recorded so the next reader does not raise it.

### F-405 — the German source at ArMDE:5791 names the wrong Ability, and F-387's fix would copy the error into the shipped data

Found while checking `rules/i18n/de/abilities.json` for the Ability names
`flaw.church_upbringing`'s passage lists. This is **not** a D6 translation-table
case — the error is in `rules/source/de/Ars Magica Definitive Edition
Basisregeln.md` itself, which CLAUDE.md makes the source of truth for German.

> **EN ArMDE:5791** — "…on Artes Liberales, Latin, Music, **Organization Lore:
> Church**, or Theology."
>
> **DE 5791** — "…für Artes Liberales, Latein, Musik, **Ordenskunde: Kirche**
> oder Theologie."

**"Ordenskunde" is the German book's term for *Order of Hermes Lore*, not for
*Organization Lore*.** Three independent sources agree, and the German book
disagrees with itself:

1. **The canonical table.** `rules/source/de/translation-tables/fertigkeiten.md:20`
   gives `(Organization) Lore\* | Allgemein | (Organisations-)Kunde`, with the
   note "auch Organisationskunde".
2. **The German book's own usage, six times.** `grep -n "Organisationskunde"`
   over the DE source returns it at 3615 ("Organisationskunde: **Zunft**" — Craft
   Guild Training's "Organization Lore: Guild"), 3717, 3729, 4177, 4183 and 4908.
   Every one renders *Organization Lore* as *Organisationskunde*.
3. **The German book's own usage of "Ordenskunde", six times, always meaning the
   Order.** `grep -n "Ordenskunde"` returns 4065 (Hermetic Experience, EN "Order
   of Hermes Lore"), 5867 (Covenant Upbringing, likewise), three statblocks that
   spell it out as "**Ordenskunde des Hermes**" (19862, 20411, 21373) — and
   **5791**, the single outlier.

So DE 5791 is the one line in the book where the two distinct Abilities are
conflated. The catalogue has them as one parameterized Ability,
`ability.organization_lore` (category `general`, parameter `organization`, DE name
`{organization}-Kunde`) — so the German text a reader is given would name
"Ordenskunde: Kirche" while the Ability picker beside it offers "Kirche-Kunde".
There is no `ability.order_lore` to confuse it with; `grep -c "ability.latin"`
and the abilities catalogue were both checked.

**Why this is worth a finding number rather than a note:** F-387 obliges a German
`description` for this entry, and the obvious way to write one is to copy DE 5791
verbatim — which is what every other description in this batch's i18n does, and
correctly so. Doing it here would ship the wrong Ability name into the
application. The German description must read **"Organisationskunde: Kirche"**.

**What to do about the source file is not mine to decide** and I have changed
nothing. Editing a rulebook source is a larger call than editing data, and the
memory note that the rules sources will be re-synced upstream bears on it. The
minimum is that the correction pass writing F-387's German text must not copy
line 5791's term. **Severity: moderate** — a wrong Ability name in shipped rules
text, on an entry whose whole mechanic is which Abilities the experience may buy.

### F-406 — `flaw.a_deal_with_the_devil` — an item-transfer clause the catalogue encodes everywhere else, encoded here nowhere

**Raised by the verification sub-agent, which overturned this file's first-pass
verdict of "clean". I accept the overturn; the evidence below I re-verified
myself.**

> "This Flaw **includes the effects of Plagued By Supernatural Entity** as Hell's
> agents goad and cajole the character…" — ArMDE:5907
>
> DE 5907: "Dieser Fehler **schließt die Auswirkungen von** Gepeinigt von einem
> übernatürlichen Wesen **ein**…"

My first pass followed this pointer, found `flaw.plagued_by_supernatural_entity`
(ArMDE:6590-6593) to be `narrative` with no effects, concluded "this entry
imports nothing", and passed it. **That reasoning was too narrow: it asked what
the target *computes* and not what the idiom *is*.**

**The catalogue answers that question four times out of four, and this is the
fifth.** `grep -n "includes the effects of"` over the English core rulebook
returns exactly four hits; adding the "grants the … Virtue" variant gives a fifth
passage. Every one of the others is encoded as a real mechanical composition:

| passage | clause | how the catalogue encodes it |
|---|---|---|
| ArMDE:4882 | "also includes the effects of the Social Contacts Virtue" | `virtue.rosh_beth_din` — `creation_effect`, `grants_selection` |
| ArMDE:5115 | "includes the effects of the Brother-Knight Virtue" | `virtue.templar_commander` — `creation_effect`, `grants_selection` |
| ArMDE:5743 | "also includes the effects of the Unaging Virtue" | `flaw.bound_to_role_role` — `in_play_effect`, `aging_mod` |
| ArMDE:5008 | "grants the Second Sight Virtue for free" | `virtue.spirit_votary` — `creation_effect`, `grants_selection` |
| **ArMDE:5907** | **"includes the effects of Plagued By Supernatural Entity"** | **`narrative`, no `effects`, no `description`** |

*(I re-ran both queries rather than taking them on trust: the four-hit grep, and
`jq` over the four ids, which returns `grants_selection` on three and `aging_mod`
on one.)*

**Which checks fail.** Check 2 — `Classification::Narrative`'s documented claim
is that the passage states no mechanical clause **at all**, and a clause that
imports another catalogue entry is not that; under D3 even an inexpressible
version would be `uncomputed_rule`, never `narrative`. Check 10 — a passage
clause with no effect, in the direction the audit keeps finding. Check 12 / D5 —
and this half is beyond argument: the shipped text is `summary` only, which stops
at sentence one ("The character has sold his soul to the devil…" / "Der Charakter
hat seine Seele dem Teufel verkauft…"), so **the composition clause reaches the
player in neither locale**.

**Nothing numeric is being dropped** — the target really does compute nothing, as
my first pass established — so the harm is the composition and its invisibility,
not a wrong figure. What shape the fix takes is **Q-94**, because the obvious
`grants_selection` has a side-effect the source cannot adjudicate.

**A German-source wrinkle for whoever writes the DE description.** DE:5907's body
calls the referenced Flaw *"Gepeinigt von einem übernatürlichen Wesen"*, but its
own DE heading at DE:6590, the DE index link at DE:5364, and the shipped
`rules/i18n/de/` name all read *"Von übernatürlichem Wesen geplagt"*. The data
follows the heading, which is right; the body text is the outlier. This is the
**second** German-source naming slip in this batch's span — see F-405 — and the
same caution applies: do not copy DE:5907 verbatim.

### F-407 — `virtue.familiarity_with_the_fae` (ArMDE:3857-3860, **outside this batch**) — a gated-Ability permission with no `ability_authorization`

Found by the verification sub-agent while following the item-transfer idiom out of
F-406, and re-verified here. It belongs to **B03**'s span (ArMDE:3767-3966);
recorded per the brief's instruction to rate a cross-reference in full wherever it
lands, because B03 can re-derive this far more cheaply than it can rediscover it.

> "You get a +2 to all rolls involving social interaction with faeries. You also
> gain the effects of the Common Sense Virtue, but only when the situation
> pertains to faeries. **You may purchase Faerie Lore at character generation,
> even if normally unable to take Arcane Abilities.**" — ArMDE:3859

`rules/core/abilities.json` gives `ability.faerie_lore` the category `arcane`,
and `arcane` is in `categories_requiring_virtue`. The entry is
`classification: "uncomputed_rule"` and carries **no `effects` key at all** (`jq`
returns `id, kind, magnitude, categories, classification, entity_kinds, source`).
So a grog or companion holding this Virtue *and* Faerie Lore — which is the exact
situation the sentence exists to permit — raises a hard
`ability_category_requires_virtue` error from
`validation/authorization.rs::validate_ability_authorization`.

**Unlike F-388, this one is not a hard design problem.** The sentence grants
unconditional permission for one named Ability, with no point budget attached, so
`{"type": "ability_authorization", "abilities": ["ability.faerie_lore"]}` states
exactly what the book states — the same shape `flaw.covenant_upbringing` already
ships four entries away from this batch's span. That also means `uncomputed_rule`
is the wrong class: the engine *can* express this, so D3 does not apply and the
entry should be `creation_effect` once wired. **Severity: high**, same as F-388 —
a false hard error blocking a legal character — but with an unambiguous fix.

*(Not rated here, and flagged for B03: the same passage's "+2 to all rolls
involving social interaction with faeries" and its conditional import of Common
Sense. I read them but did not audit their encoding.)*

## Open questions

**Q-88 — Should a player's free-text choice of *scope* be recorded by a
parameter?** Three entries in this batch let the player pick something the engine
stores nowhere: `flaw.deleterious_circumstances`' circumstance (ArMDE:5919 — a
state, a target, or a place), `flaw.curse_of_slander`' section of mundane society
(ArMDE:5883 — "the nobility, the church, guilds, peasants"), and
`flaw.demonic_familiar`'s kind of demon (ArMDE:5930 — "a warder, teacher, or
paramour"). B10 raised the same question as its Q-86 on `flaw.baneful_circumstances`
and `flaw.busybody`. A `text`-domain parameter would record the choice and make
two copies distinguishable; engine-semantics B8 notes that an unconsumed
parameter is "the normal and intended shape" for exactly this. But adding one
forces the player through a required picker, and nothing computes from it. This
needs one ruling for the whole catalogue, not five per-entry judgements.

**Q-89 — ArMDE:5911 says "Technique" inside *Deficient Form*'s own passage.** "Experience
points required are based on the actual value of the **Technique**, before
halving" appears verbatim in both the Deficient Form (:5911) and Deficient
Technique (:5915) entries, and the German mirrors it exactly (DE 5911 "richten
sich nach dem tatsächlichen Wert der **Technik**"). So either the book
copy-pasted the sentence and Deficient Form's XP rule is misprinted, or it means
something I am not seeing. It changes nothing today — `effective/xp.rs` never
consults `deficient_arts`, so *both* Arts are costed off their unhalved score —
but a correction pass writing this into a `description` must decide what to
write, and I will not pick a reading.

**Q-90 — Which of the two Deficient Art entries has the right shape?** F-402
shows `flaw.deficient_technique` carries `prerequisites: has virtue.hermetic_magus`
and `flaw.deficient_form` carries none, and that a Gifted companion may hold
Hermetic V/F. Either the prerequisite is a stray that should be deleted (and the
category gate is the only gate), or it is correct and 121 other Hermetic entries
are missing it. This is one ruling with 122 consequences, so it goes to Norbert
rather than into a per-entry verdict.

**Q-91 — Does ArMDE:5895's third sentence change D4's answer for Cyclic
Magic?** D4's table rules `virtue.cyclic_magic_positive` / `flaw.cyclic_magic_negative`
out of a character-generation Lab Total with the reason "no; creation fixes no
season". The passage's third sentence says "**The penalty applies to Lab Totals
even if the negative period does not cover the whole of the season**", and its
fourth makes the negative period exactly half the year ("must be equal to the
time when there is no penalty"). Read together, a character with this Flaw is
penalised in *any* season that overlaps the negative half — which is at least two
of four, whichever seasons creation is deemed to use. I am **not** overturning
D4; I am recording that its stated reason does not match this sentence, because
D4's own Phase 2 slice needs the condition precisely and a previous ruling here
had to be withdrawn for exactly this kind of unread passage.

**Q-92 — How should a *mandatory earmark of the normal budget* be encoded?**
`flaw.church_upbringing` (ArMDE:5791) compels 25 of the character's own
experience points onto a five-item list and then bars the Academic category.
`RestrictedAbilityXp` grants new points and would make the Flaw a benefit (F-387);
`AbilityAuthorization` is unbounded and would permit unlimited Academic spending
where the book permits 25 points' worth (F-388). Both available shapes are wrong
in opposite directions, and the character cannot legally exist today. Options I
can see: (a) `uncomputed_rule` + description and accept that a legal character
raises a false hard error; (b) add an authorization and accept the
over-permission, as `RULES.md:6362` already did once for Covenant Upbringing;
(c) a new effect variant that earmarks rather than grants. This is a design call.

**Q-93 — Which treatment is intended for the three Corrupted entries?**
ArMDE:5847-5864 states one mechanic three times (Abilities, Arts, Spells) and the
catalogue gives it three different classifications and three different effect sets
(table in F-393). At most one of the three can be right. Unifying them upward
(all `in_play_effect` with `special_casting_mod { circumstantial }`) loses the
Abilities version, whose ±3 is on an *Ability roll*, not a Casting Total.
Unifying them downward (all `uncomputed_rule` + description) is consistent, is
honest about what the engine computes, and matches what `RULES.md:5226-5229`
already says about Corrupted Arts — but it means deleting a shipped effect from
`flaw.corrupted_spells`. F-393 recommends the downward reading for Corrupted Arts
alone, which is the minimum change; the catalogue-wide question is Norbert's.

**Q-94 — Should `flaw.a_deal_with_the_devil`'s composition be a `grants_selection`,
and does a granted Story Flaw count toward the one-Story-Flaw guideline?** Raised
by the verification sub-agent and left open here. Two readings, and ArMDE:5907
chooses between neither. **(a)** `creation_effect` + `grants_selection:
[flaw.plagued_by_supernatural_entity]`, matching all four in-book precedents
(F-406's table) — but both entries are **Story** Flaws, so granting one would give
the character two, and `validation/caps.rs` carries
`{"category":"story","max":1}` on the companion, magus and mythic-companion
profiles and `max: 0` on grog. That may be exactly why it was never modelled.
**(b)** `uncomputed_rule` + the full passage as `description` in both locales,
which fixes the invisibility with no cap side-effect. What would settle (a) is a
product decision — whether `grants_selection`-derived rows count toward category
caps at all — not a rules reading. *(Neither the sub-agent nor I traced whether
`validate_caps` sees granted rows; engine-semantics B3 says `validate_caps`
iterates **bought** selections only, which would mean no spurious warning and
would favour (a) — but that is my inference from the yardstick document, not
something either of us verified against `caps.rs` in this run, so it stays open.)*

**Q-95 — Can a parameter express "one or more, your choice"?** The three Corrupted
entries each let the player pick an open-ended *set*: ArMDE:5851 "you can choose
to have it affect **multiple Abilities**", :5857 "it can affect **multiple
Arts**", :5863 "as many of the character's **spells** as you wish". Nothing
records any of the choices, though the per-item ±3 / ±5-XP consequences are
per-Ability, per-Art and per-spell. The sub-agent established the blocker
precisely: **all parameters in the catalogue are single-valued `"type": "ref"`**,
so one `ability`-domain parameter would misstate "one or more", and adding one
*without* `max_total: 1` would silently convert the book's explicit "only take
this Flaw once" into once-per-Ability. Settling it needs a decision on whether the
parameter model should gain a multi-valued variant. Neither of us found any note
in `RULES.md` either way. *(This is narrower and sharper than Q-88, which asks
about single-valued scope choices; they should be answered together.)*

**Q-96 — Is the partial `source.anchor` coverage a backfill in progress or an
oversight?** The sub-agent counted **94 of 655** catalogue entries carrying an
anchor, and only **4 of the 35** in this batch, though every one of the 35 has a
`####` heading that yields one. Engine-semantics B11 calls the anchor "the durable
half" of the provenance precisely because a line range shifts under any edit above
it and fails silently, while a renamed heading fails loudly. There is no guard
requiring an anchor. With the rules sources expected to be re-synced upstream —
which will shift every line number in the catalogue at once — this is the field
that would survive, and 561 entries do not have it. Not a defect of any entry in
this batch; a catalogue-wide question about whether the backfill should be
finished before the re-sync.

## Sub-agent reconciliation

One verification sub-agent was run, after this file's first pass was on disk, with
the brief of **independently re-deriving the verdict for every entry marked fully
clean** — eighteen at the time — reading the passages itself rather than this
file's notes. It confirmed it never opened `batch-11.md`. It spawned no agents,
wrote no file, and ran no git-mutating command.

**Result: 17 corroborated, 1 overturned.**

**The overturn is `flaw.a_deal_with_the_devil` → F-406**, and it is a good one,
because it catches a reasoning error rather than an oversight. My first pass *did*
follow the cross-reference and *did* open `flaw.plagued_by_supernatural_entity`
(ArMDE:6590-6593) — the rated-cross-references table records the read — but asked
the wrong question of it: whether the **target computes anything**, rather than
whether the **idiom is mechanical**. The sub-agent asked the second question,
grepped the idiom across the whole rulebook, and found the catalogue answering it
four times out of four. A negative established by reading one entry was wrapped in
a conclusion that needed four. I have re-verified both queries and accepted the
overturn in full.

**One withdrawal of the sub-agent's I do not accept, and the reason is a
contradiction inside `RULES.md`.** It raised `flaw.corrupted_arts` as
`creation_effect` with no effects — independently reaching this file's F-393 — and
then withdrew it on finding `RULES.md:5411`, which lists the entry among the
5a-wire deferrals as "grants XP swing **at creation** + situational casting". That
withdrawal was correct procedure (check `RULES.md` before calling a modelling
choice wrong) applied to a document that disagrees with itself:

> `RULES.md:5411` — "`flaw.corrupted_arts` (ArMDE:5853-5858) — **grants XP swing
> at creation** + situational casting"
>
> `RULES.md:5226-5229` — "**Corrupted Arts (ArMDE:5853) has no creation XP figure
> (its ±3 casting swing / ±5 Art xp are in-play).**"

The second passage is the deferral's own *rationale* and the first is the
index line above it; they say opposite things about whether there is a creation
number. The passage settles it — every clause at ArMDE:5855 is contingent on a use
of the Art in play — so **F-393 stands**, and the sub-agent's finding is
corroboration of it rather than a withdrawal. The `RULES.md` self-contradiction is
now recorded inside F-393 as a third reason the entry needs re-reading.

**Four negatives the sub-agent established that this file had assumed**, and which
I had not independently checked in the form it checked them:

1. **Repeat limits across the whole batch.** None of the eighteen has
   `parameters`, so `max_per_target`'s default of 1 — keyed on
   `(item_ref, {})` — already forbids a second copy. `flaw.corrupted_abilities`'
   "You may only take this Flaw once" is therefore enforced, if incidentally.
   My method section asserted this for the three "only once" entries; the
   sub-agent proved it for all eighteen.
2. **Gated Abilities across the whole batch.** It resolved both Abilities these
   passages name — `ability.guile` is `general` (ungated), `ability.finesse` is
   `arcane` but reachable only through a Hermetic-category Flaw — so **no**
   `ability_authorization` is owed anywhere in the eighteen.
3. **`ArMDE:2782`.** It read the general restriction on Blind and Mute for
   characters with easy access to Hermetic magic and confirmed it **does not name
   Deaf**, so `flaw.deaf` owes no prerequisite. I had not opened that passage.
4. **`ArMDE:2820` is live.** The Major/Minor split on Compassionate, Compulsion
   and Compulsive Lying feeds a real check — `validation/caps.rs` cites `:2820`
   and emits `too_many_major_personality_flaws` — so the split is load-bearing
   rather than cosmetic, and the mutual `incompatible_with` is additionally
   load-enforced by `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`.

**One finding outside the batch: F-407** (`virtue.familiarity_with_the_fae`,
ArMDE:3857-3860, B03's span), reached by following the F-406 idiom. Re-verified
here and written up in full.

**One German-source observation** — DE:5907's body naming
`flaw.plagued_by_supernatural_entity` differently from its own heading — folded
into F-406.

**Three open questions raised by the sub-agent** and adopted as **Q-94**, **Q-95**
and **Q-96**. None is settled; all three are stated self-contained.

**No disagreement between us remains unresolved.** The single divergence — the
`flaw.corrupted_arts` withdrawal — is resolved above on evidence (`RULES.md`
contradicting itself) rather than by preferring one of our readings, and the
resolution is recorded in F-393 so the next reader does not re-litigate it.
