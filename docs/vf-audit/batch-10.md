# Batch B10 — indices 315-349, ArMDE:5257-5780

Entries: 35. Audited: 35. Failures: **24**. Clean: **11**.
Findings: **F-349 … F-383**, of which **F-373 is withdrawn by this batch itself**
— 34 live. Open questions: **Q-79 … Q-87**, of which **Q-85 is withdrawn** — 8
live. The verification sub-agent re-derived all eleven clean entries and
**overturned none**; it corrected one claim of this file's and raised **Q-87**.

**The batch's second shape is `incompatible_with`, and the span is a near-total
miss.** ArMDE:5257-5780 states **five** facts about which pairs of catalogue
entries may coexist, and the data gets **one** of them right:

| passage | rule | data |
|---|---|---|
| ArMDE:5269 (+ :7003) | Withstand Casting and Vulnerable Casting may not both be taken at creation | absent both sides — **F-352** |
| ArMDE:5697 | a troupe **may allow** both Beloved Rival versions | the data **forbids** it, and a load-time guard requires the data to — **F-360** |
| ArMDE:5717 | Blatant Gift and Blatant Magical Air may not both be held | absent both sides — **F-366** |
| ArMDE:5729 (+ :4057) | Bound Magic may not be taken with Harnessed Magic | absent both sides — **F-371** |
| ArMDE:5751 | Branded Criminal may not take Wealthy | absent both sides — **F-376** |
| ArMDE:5259 | Wise One is **not** excluded from Wealthy/Poor | correctly nothing ✓ (the one right answer, by omission) |

And ArMDE:5783, four lines past the span, states a sixth that is equally absent —
handed to B11 in the cross-reference table below.

**The batch's headline is F-383**: 33 of these 35 entries lie inside a
`SWEPT_BLOCKS` range that was swept on 2026-09-15, **re-swept on 2026-09-19**,
and is guarded by a live test — and nine of them are still `narrative` while
stating a rule, because all nine state it in words the 20-phrase screen does not
carry. Five are one word-form away from a phrase already on the list.

**F-373 and Q-85 are left in the file rather than deleted**, because the
withdrawal is the useful part: the DE name of `flaw.bound_to_realm` looks wrong
against both the DE rulebook heading and the canonical table, and
`RULES.md:2356-2373` explains why it must be. Four entries share that shape and
all four look wrong at first glance, so the next reader to notice deserves the
answer rather than a fresh finding.

**This is the only batch that crosses the Virtues→Flaws boundary.** Its span
holds the last two Virtues (`virtue.wise_one`, `virtue.withstand_casting`), the
whole of the book's **"List of Flaws" index** (ArMDE:5283-5637 — 356 lines cited
by no catalogue entry), the `## Flaws` heading at ArMDE:5639, and the first 33
Flaw headings. What the between-blocks material turned out to be worth is
written up in its own section below; the short version is that it is an index
and not a rules preamble, that it **independently confirms all eight `anchor`
values in this batch**, and that it contains a **book-internal contradiction**
about two of this batch's entries (Q-79).

Finding and question numbers continue the audit's sequence, starting where the
brief directs: **F-349** and **Q-79**.

*(Written incrementally — header, method and the 35-row verdict table landed
first; findings were appended one at a time; the sub-agent reconciliation last.)*

## Method

**Screening status — this batch is the first whose span is mostly *inside* a
swept block, and that changes what a finding here means.**
`crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS` covers
ArMDE:3360-3950 and **ArMDE:5639-7113**, so:

- `virtue.wise_one` (5257-5260) and `virtue.withstand_casting` (5261-5282) are in
  the **unswept** remainder ArMDE:3951-5282, like B04's through B09's spans;
- **all 33 Flaws** are inside the swept Flaws block, which `RULES.md:4688` records
  as having been read entry by entry on 2026-09-15 and which the file's own
  comment records as **re-swept on 2026-09-19**, with a live guard
  (`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause`) green on them.

So B10's failure rate is not comparable with B04-B09's, and every `narrative`
reclassification below is a claim that two sweeps and a passing test all missed
the same entry. That claim is made nine times, and **F-383** is the account of why
it is nonetheless true.

The span was read as continuous prose in **both** languages before any entry was
judged: `rules/source/en/Ars Magica - Definitive Edition (Core Rules).md`
ArMDE:5250-5790 and the line-parallel
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md` 5250-5790, the
read deliberately overrunning the span at both ends so the first and last
entries' boundaries could be seen.

**Line parity holds throughout.** Every `####` heading in 5257-5780 sits on the
same line number in both files, and so does every `###`/`##` heading of the
index block (checked at 5283, 5285, 5310, 5340, 5382, 5387, 5401, 5417, 5467,
5501, 5520, 5534, 5564, 5639). There are **33 `####` entry headings** in the span
for **35 catalogue entries** — the difference is the three Major/Minor pairs that
share one heading (`Ambitious` 5663, `Avaricious` 5683, `Beloved Rival` 5691),
each of which correctly ships as two ids citing the same range.

**`source.lines` (check 1) — 35 of 35 correct.** Every entry runs from its own
heading to the line before the next heading (the dominant convention) or stops
one line earlier on its last body line (`flaw.black_sheep` 5703-5705, the tighter
and equally correct variant B09 also saw). Every non-blank line in 5257-5282 and
5641-5780 is cited by exactly one entry, and the index block 5283-5638 plus the
`## Flaws` heading at 5639 are cited by none — which is right, an index is not an
entry. **No range in this batch swallows a neighbour and none is short**, so B09's
F-309 shape does not recur here.

**`source.anchor` (check 1, second half) — 8 of 35 entries carry one, and all
eight are correct**, verified twice: once by deriving the GitHub anchor from the
`####` heading, and once against the book's **own** index links, which spell the
same anchors out literally.

| id | anchor | heading | index link |
|---|---|---|---|
| `flaw.anchored_to_the_land` | `anchored-to-the-land` | ArMDE:5667 `#### Anchored to the (Land)` | ArMDE:5568 |
| `flaw.arthritis` | `arthritis` | ArMDE:5679 | ArMDE:5570 |
| `flaw.blatant_gift` | `blatant-gift` | ArMDE:5711 | ArMDE:5287 |
| `flaw.blatant_magical_air` | `blatant-magical-air` | ArMDE:5715 | ArMDE:5390 |
| `flaw.bound_to_realm` | `bound-to-realm` | ArMDE:5731 `#### Bound to (Realm)` | ArMDE:5391 |
| `flaw.broken_vessel` | `broken-vessel` | ArMDE:5753 | ArMDE:5393 |
| `flaw.brutal_artist` | `brutal-artist` | ArMDE:5757 | ArMDE:5421 |
| `flaw.castratus` | `castratus` | ArMDE:5777 | ArMDE:5572 |

The other 27 entries carry the key set `["file","lines"]` only, so for them the
line range is the whole of the provenance.

**Checks 3, 4, 5, 6 (kind / magnitude / entity_kinds / categories).** `kind` is
`virtue` for the two entries above ArMDE:5283 and `flaw` for the 33 below
ArMDE:5639 — matching the block each sits in ✓. `entity_kinds` is
`["character"]` on all 35, correct: none is a covenant Boon or Hook. `magnitude`
and `categories` agree with **every descriptor line**, read one by one:

- `*Minor, Social Status*` — wise_one (5258), branded_criminal (5750)
- `*Minor, Hermetic*` — withstand_casting (5262), bound_casting_tools (5724), bound_magic (5728), brutal_artist (5758), careless_sorcerer (5770)
- `*Major, Hermetic*` — blatant_gift (5712)
- `*Major, Story*` — abandoned_apprentice (5642), bigamist (5700), black_sheep (5704)
- `*Minor, Story*` — animal_companion (5672), blackmail (5708)
- `*Minor or Major, Story*` — beloved_rival_minor / _major (5692)
- `*Major or Minor, Personality*` — ambitious_* (5664), avaricious_* (5684)
- `*Minor, Personality*` — busybody (5762), carefree (5766)
- `*Major, Supernatural*` — age_quickly (5660), blatant_magical_air (5716), bound_to_realm (5732)
- `*Minor, Supernatural*` — baneful_circumstances (5688), bound_to_role_role (5736), broken_vessel (5754)
- `*Minor, General*` — ability_block (5652), afflicted_tongue (5656), anchored_to_the_land (5668), apostate (5676), arthritis (5680), careless_with_ability (5774), castratus (5778)
- `*Major, General*` — blind (5720)

No descriptor in this batch is misprinted (B08's and B09's period-for-comma OCR
shape does not recur), no entry carries `tainted`, and the `Tainted` type label
appears on no descriptor line in 5257-5780.

The **one** magnitude question in the batch is not a data defect but a
contradiction inside the book: the index files `Bound to (Role) Role` and
`Broken Vessel` under `### Supernatural, **Major**` while their descriptors say
`*Minor, Supernatural*`. The data follows the descriptor. See **Q-79**.

**Check 12, mechanically.** Every `name` and `summary` was read against its
passage in the matching language, and `jq` over the 35 ids in each locale returns
the key set `["name","summary"]` for 20 entries and
`["description","name","summary"]` for 14 — plus `flaw.bound_to_realm`, which
carries `name_unfilled` **in DE only** (`"Gebunden an (Sphäre)"`; EN has none, which
is legal — `types.rs` documents `name_unfilled` as opt-in). So **14 of 35 entries
carry a `description` in both locales and 21 carry none in either**; the locales
never disagree about *whether* a description exists, which is why every D5
finding below reads "reaches neither locale".

**ASCII hyphen (check 12, second half): clean, catalogue-wide.**
`grep -c "−"` (U+2212) over `rules/i18n/en/virtues_flaws.json` and
`rules/i18n/de/virtues_flaws.json` returns **0** for both files — not just for
this batch. The shipped descriptions convert correctly in both directions: the
EN source's en dash at ArMDE:5713 (`–6`) and ArMDE:5733 (`9 – Size`) ship as
ASCII `-6` and `(9 - Size)`, and the DE source's en dashes at 5657/5669/5681/5713/
5717/5733/5759/5779 (`–2`, `–3`, `–6`) all ship as ASCII. A correction pass adding
the missing descriptions must keep doing this — the DE rulebook writes every
negative as `–` (U+2013).

### The material between the two blocks

ArMDE:5283-5638 is the book's **"List of Flaws"** — thirteen `###` headings of
the form `<category>, <magnitude>`, each followed by anchor links, and **no prose
and no rule of its own**. Unlike B09's Knights Templar sidebar it states nothing
in its own voice, so there is no F-309-shaped orphaned rule here. It is still
load-bearing in three ways, and all three were checked:

1. **It is the authority behind `index_categories`.** `RULES.md:952` already
   cites this exact block (`ArMDE:5445`, `ArMDE:5455`) as the evidence that
   Offensive to (Beings) and Unbearable to (Beings) are Hermetic Flaws for
   `validate_house`'s ArMDE:2860 guideline. So the index is not decoration.
2. **Its dual listings were checked against the catalogue, and are clean.** Seven
   entries appear under two headings — `Suppressed Gift` (5301, 5369),
   `Raised from the Dead` (5365, 5399), `Primogeniture Lineage` (5447, 5516),
   `Visions` (5517, 5561), `Outsider` (5385, 5530), `Curse of Slander` (5538,
   5575), `Offensive to (Beings)` (5445, 5608), `Unbearable to (Beings)` (5455,
   5629). Only 4 entries in the whole catalogue carry `index_categories`, which
   looked like a gap — it is not. I read each dual-listed entry's **descriptor**
   and found that in every case but Primogeniture Lineage the descriptor itself
   names both categories (`*Minor, General or Supernatural*` ArMDE:5882,
   `*Major, Story, Supernatural*` ArMDE:6647, `*Minor, Story, Supernatural*`
   ArMDE:6986, `*Major, Hermetic, Story*` ArMDE:6804, `*Minor or Major, Social
   Status*` ArMDE:6551), and the data carries both as real `categories`.
   Primogeniture Lineage (`*Minor, Story and Hermetic*` ArMDE:6635) is the one
   that uses `index_categories: ["hermetic"]` instead — which `RULES.md:930-941`
   documents as deliberate, because adding `hermetic` as a membership category
   would make `effective::has_the_gift` treat the holder as Gifted. **No finding.**
   *(Scope of this claim: I checked exactly the eight dual listings above, by
   reading their descriptor lines; I did not re-walk the whole index.)*
3. **It confirms all eight anchors in this batch**, as tabulated above.

Two observations from the index that are **not** findings of this batch but are
recorded so they are not re-discovered: the index's own category/magnitude filing
contradicts two descriptors (Q-79), and the DE index at the line-parallel 5387-5399
repeats the *same* misfiling, so it is inherited from the original book rather
than introduced in translation.

### Cross-references followed

Every pointer in the span was followed and read — the `(see page NNN)` form, the
named-Virtue/Flaw form and the other-supplement form alike — including on entries
that passed.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the entry? |
|---|---|---|---|
| `virtue.wise_one` | "The Wealthy Virtue and Poor Flaw affect you normally" | `virtue.wealthy` ArMDE:5235-5238 ✓, `flaw.poor` ArMDE:6594-6596 ✓ | **No new rule** — it is an explicit *denial* of an exclusion, notable only because so many neighbours state one. Recorded under F-350 as a D5 clause. |
| `virtue.withstand_casting` | "Vulnerable Casting"; "Life Boost and Life-Linked Spontaneous Magic" | `flaw.vulnerable_casting` ArMDE:6993-7004 ✓, `virtue.life_boost` ✓, `virtue.life_linked_spontaneous_magic` ✓ | **YES — a creation-time incompatibility**, restated from the other side at ArMDE:7003, encoded on neither side → **F-352**; and a forfeiture rule → F-353. |
| `flaw.blatant_gift` | "should see **page 203** for further discussion of this Flaw's effects" / DE `#die-gabe-2` | `## The Gift` **ArMDE:8743-8769** | **YES — five clauses, none of which reaches the user** → **F-368**. |
| `flaw.blatant_magical_air` | "a Magical Air or The Gift"; "the same as the Blatant Gift" | `flaw.magical_air` ArMDE:6382-6385 ✓, `virtue.the_gift` ArMDE:3967-3970 ✓, `flaw.blatant_gift` ✓ | **YES, twice** — an exactly-expressible prerequisite (**F-367**) and a stated mutual exclusion (**F-366**). |
| `flaw.bound_magic` | "Harnessed Magic (**page 85**)" | `virtue.harnessed_magic` **ArMDE:4053-4058**, whose own ArMDE:4057 states the same drawback | **YES — a stated incompatibility, corroborated from both ends**, encoded on neither → **F-371**. |
| `flaw.bound_to_role_role` | "the Unaging Virtue"; "*Covenants*, page 11" | `virtue.unaging` ArMDE:5187-5190 ✓; *Covenants* is **outside `rules/source/en/`** | **Yes for Unaging** — and the data gets it *right*: Unaging ships `no_aging` **and** `no_apparent_aging`, this Flaw ships `no_aging` alone, exactly as ArMDE:5743's carve-out requires (`RULES.md:7034`). Corroboration, not a finding. Nothing claimable from *Covenants*. |
| `flaw.brutal_artist` | "*Houses of Hermes: Societates*, **page 53**" / DE `#ligamitgliedschaft` | `### League Membership` **HoH:S:2260-2284** | **No rule attributable to the Flaw.** The only number behind the pointer is the leagues' own "All Reputations act as if they were 3 points higher, between colleagues" (HoH:S:2278) — a league *benefit*, not a Flaw rule. The Flaw's own mechanical clauses (the −3 and the Reputation bar) are already in its description ✓. |
| `flaw.bigamist` | "the rules in *City and Guild*" | **outside `rules/source/en/`** | **The numbers are stated in ArMDE:5701 itself** (6 × / 3 × Wealth Multiplier), so the passage is not narrative even though the rule cannot be implemented → **F-364**. |
| `flaw.blind` | "Blind magi cannot aim spells without magical aid" | the targeting rules | **Yes, an absolute prohibition** the class denies → **F-369**. |
| `flaw.broken_vessel` | "Supernatural Abilities, Powers, or … casting spells" | the Supernatural-Ability category ✓ | **Yes, a prerequisite** the `Prereq` tree cannot express (no "holds any Ability of category X" variant) → D3 keeps it `uncomputed_rule`, and the description **does** carry it in both locales ✓. No finding. |
| `flaw.ceremonial_spontaneous_magic` (ArMDE:5781-5784, the **first entry of B11**) | "not compatible with Difficult Spontaneous Magic or Weak Spontaneous Magic" | its two named neighbours | **Not this batch's entry, but checked and handed over:** `jq` returns `incompatible_with: null` on it, and `classification: "narrative"`. So B11's very first entry is the same missing-incompatibility shape as F-352, F-366 and F-371 here — four instances in one span plus its immediate neighbour. |

### ArMDE:2814, :2816, :2818, :2820 — which of the four general rules bite here

The brief asks for ArMDE:2816 instances to be noted, not re-reported. Reading the
whole `## Virtues and Flaws Rules and Guidelines` block (ArMDE:2812-2820) while
settling F-358 turned up that the four rules are in **different** states, which
is worth one paragraph so no later batch re-derives it:

| rule | ArMDE | enforced? |
|---|---|---|
| "may be taken more than once **only if the description explicitly allows it**" | :2814 | **Partly.** `max_per_target`'s default of 1 enforces it for unparameterized entries; parameterized entries need an explicit `max_total` and three in this batch lack one → **F-358**. |
| "must take one Social Status, and may only take more than one if the descriptions explicitly note that they are compatible" | :2816 | **No** — neither half. |
| "should not have more than one Story Flaw" | :2818 | **Yes, on all four profiles** — `flaw_category_caps` carries `{ "category": "story", "max": 1 }` on companion, magus and mythic companion, and `{ "category": "story", "max": 0 }` on **grog**, which additionally encodes the per-type list at ArMDE:2826 ("You should not take Story Flaws"). *(First written here as "on the companion profile"; the verification sub-agent corrected it and the correction is verified — `rules/core/character_types.json` lines 13, 56, 88, 131.)* |
| "may not have more than one Major Personality Flaw … should normally not have more than two Personality Flaws in total" | :2820 | **Yes** — `{ "category": "personality", "max": 1, "major_only": true, "hard": true }` and `{ "category": "personality", "max": 2 }` ✓. |

**Scope of the ArMDE:2816 negative, stated exactly as checked:** I grepped
`rules/core/character_types.json` for `social_status` — four hits, all inside
`permitted_categories` arrays, none a cap or a requirement — and grepped
`crates/arm-rules/src/validation/` for `social`, whose every hit is inside a
`#[cfg(test)]` JSON fixture rather than a rule. I did not audit the whole
validation module for some differently-named check.

This batch's ArMDE:2816 instances are **`virtue.wise_one`** (ArMDE:5257-5260) and
**`flaw.branded_criminal`** (:5749-5752); neither states a compatibility, so
neither may be combined with another Social Status, and nothing says so.
ArMDE:2818 instances: `abandoned_apprentice`, `animal_companion`,
`beloved_rival_*`, `bigamist`, `black_sheep`, `blackmail`. ArMDE:2820 instances:
`ambitious_*`, `avaricious_*`, `busybody`, `carefree`. ArMDE:2960-2962 (realm
association) instance: **`flaw.bound_to_realm`**, which is the only entry in the
batch with a `realm`-domain parameter and correctly carries one.

### Part C systemic gaps are not re-reported per entry

In particular: `HealthTrack::CastingFatigue` being **surfaced-only** (C1) is not
counted against `virtue.withstand_casting` — what *is* counted (F-351) is its
**sign**, which is a data claim, not an engine one. `grants_reputation`'s
unenforced `score` (C2-c) is not counted against `flaw.apostate` or
`flaw.black_sheep`; the *polarity* gap (F-363) is a different and, as far as I
can tell, unrecorded one. The frontend `Effect` union's omissions (C6) touch
nothing in this batch. And `aging_mod`'s inert `amount` for marker kinds (C2-d)
is not counted against `flaw.bound_to_role_role`.

## Decisions applied

`docs/vf-audit/decisions.md` was read in full and is binding. It carried D1-D6
when this batch read it (the file is modified in the working tree by another
workstream; nothing in this batch touched it).

- **D3** governs `flaw.broken_vessel` (category-scoped prerequisite),
  `flaw.castratus` (a sex-of-birth restriction), `virtue.wise_one`'s either/or
  half and `flaw.beloved_rival_*`: an engine that structurally cannot express a
  rule means `uncomputed_rule` with the rule written out, **never** `narrative`.
- **D5** is the single largest source of findings here: 21 of 35 entries carry no
  `description` in either locale, and 13 of those state a mechanical clause the
  effects do not implement.
- **D4** governs **F-356** (`flaw.afflicted_tongue`'s condition folded flat) by
  analogy: D4 rules only on `lab_total_mod`, so F-356 is reported as a
  conditional-folded-flat instance on a *different* channel, not as an
  application of D4.
- **D6** governs **F-378, F-379, F-380** and **F-381**. Per the brief, the source
  project `arm-de-translation` is outside this repository, so each is stated as
  *"this repository's copy disagrees with the rulebook"* and the
  error-vs-stale-copy determination is left open.
- **D1** and **D2** touch nothing in this batch: no entry carries
  `lab_total_mod`, and no entry is a granted Virtue or a
  `characteristic_score_delta_param` carrier.

## Verdicts

35 rows, one per entry. `class` / `data` / `text` report checks 2, 3-11 and 12.
"OK" means every check in that column passed; `?` means escalated, not resolved.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.wise_one` | 5257-5260 | narrative → creation_effect | **`ability_authorization` missing for two gated categories**; the either/or is not expressible (D3) | the either/or, the Wealthy/Poor non-exclusion and the availability clause in neither locale | F-349 F-350 F-381 ArMDE:2816 |
| `virtue.withstand_casting` | 5261-5282 | in_play_effect ✓ | **`casting_fatigue` sign is +1 where both Flaw carriers ship −1**; the Vulnerable Casting incompatibility encoded on neither side | the whole mechanic — the minimum, the per-level scaling, the ordering and the Life-Linked forfeiture — in neither locale | F-351 F-352 F-353 **Q-81**; DE name split F-378 |
| `flaw.abandoned_apprentice` | 5641-5650 | narrative → uncomputed_rule | OK | the creation procedure and the Parma consequence in neither locale | F-354 **Q-84** |
| `flaw.ability_block` | 5651-5654 | narrative → uncomputed_rule | no parameter records the blocked class | the block and the "must be learnable" proviso in neither locale | F-355 |
| `flaw.afflicted_tongue` | 5655-5658 | in_play_effect ✓ | **`scope: all` folds in the book's "using words" condition**; `CastingScope` has no words axis though `SpecialCasting::QuietWords` exists | OK — description carries the whole passage in both locales ✓ | F-356 |
| `flaw.age_quickly` | 5659-5662 | in_play_effect ✓ | inert `amount: 0` is a **recorded** decision (`RULES.md:5139`) ✓ | the doubled effective age and the **two** aging rolls a year in neither locale | F-357 |
| `flaw.ambitious_major` | 5663-5666 | narrative ✓ | OK | OK | **clean** |
| `flaw.ambitious_minor` | 5663-5666 | narrative ✓ | OK | OK | **clean** |
| `flaw.anchored_to_the_land` | 5667-5670 | uncomputed_rule ✓ | **no `max_total`, so a once-only Flaw is unlimited, one copy per land (ArMDE:2814)** | OK ✓ | F-358 F-379 |
| `flaw.animal_companion` | 5671-5674 | narrative ✓ | OK | OK | **clean** |
| `flaw.apostate` | 5675-5678 | creation_effect ✓ | kind + score correct and doubly corroborated; **"bad" is not representable** | the bad Reputation's level reaches the user only as a grant row | F-363 F-380 |
| `flaw.arthritis` | 5679-5682 | uncomputed_rule ✓ | OK | OK ✓ | **clean** |
| `flaw.avaricious_major` | 5683-5686 | narrative ✓ | OK | OK | **clean** |
| `flaw.avaricious_minor` | 5683-5686 | narrative ✓ | OK | OK | **clean** |
| `flaw.baneful_circumstances` | 5687-5690 | in_play_effect ✓ | inert `amount: 0` recorded ✓; no parameter records the circumstance | the extra aging roll and its **override of aging immunity** in neither locale | F-361 F-362 **Q-86** |
| `flaw.beloved_rival_major` | 5691-5698 | narrative → uncomputed_rule | **the data forbids what ArMDE:5697 permits** — both versions together | the mutual-rivalry permission in neither locale | **F-360** |
| `flaw.beloved_rival_minor` | 5691-5698 | narrative → uncomputed_rule | same, other side | same | **F-360** |
| `flaw.bigamist` | 5699-5702 | narrative → uncomputed_rule | OK — the numbers belong to a book outside `rules/source/en/` | the two Labor-Point figures in neither locale | F-364 |
| `flaw.black_sheep` | 5703-5705 | creation_effect ✓ | `kind: local` fixed where the book says "of your choice" **?**; "bad" not representable | OK — the summary states the level ✓ | F-363 **Q-80** |
| `flaw.blackmail` | 5707-5710 | narrative → uncomputed_rule **?** | OK | the 50-silver-penny yearly value in neither locale | F-365 **Q-83** |
| `flaw.blatant_gift` | 5711-5714 | uncomputed_rule ✓ | the Blatant Magical Air exclusion missing; the two edges it *does* carry are correct and recorded (`RULES.md:921`, `:5122`) ✓ | **five clauses behind "page 203" in neither locale** | F-366 F-368 |
| `flaw.blatant_magical_air` | 5715-5718 | uncomputed_rule ✓ | **prerequisite absent though exactly expressible**; the Blatant Gift exclusion missing | OK — the description carries both clauses ✓ | F-366 F-367 |
| `flaw.blind` | 5719-5722 | narrative → uncomputed_rule | OK | the reading, missile and spell-aiming prohibitions in neither locale | F-369 |
| `flaw.bound_casting_tools` | 5723-5726 | narrative → uncomputed_rule | OK | the Arcane-Connection durations in neither locale | F-370 **Q-82** (EN source sentence truncated) |
| `flaw.bound_magic` | 5727-5730 | narrative → uncomputed_rule | **the Harnessed Magic exclusion encoded on neither side** | the death clause and the exclusion in neither locale | F-371 F-372 |
| `flaw.bound_to_realm` | 5731-5734 | uncomputed_rule ✓ | **no `max_total`, so all four Realms may be taken at once** (ArMDE:2814) | OK ✓ — the DE name's divergence from the rulebook heading is a recorded decision (F-373 withdrawn) | F-358; ArMDE:2960 instance |
| `flaw.bound_to_role_role` | 5735-5748 | in_play_effect ✓ | **"only grogs" enforced nowhere**; **no `max_total`**; the `no_aging`-only tag is correct ✓ | the deprivation rule, the Unaging carve-out and the grog restriction in neither locale | F-358 F-374 F-375 **Q-79** |
| `flaw.branded_criminal` | 5749-5752 | narrative → creation_effect | **`incompatible_with virtue.wealthy` missing on both sides**; **`ability_authorization` (martial) missing** | both clauses in neither locale | F-376 F-377 ArMDE:2816 |
| `flaw.broken_vessel` | 5753-5756 | uncomputed_rule ✓ | OK — the inexpressible prerequisite is correctly left as text (D3) | OK ✓ | **clean**, except the index/descriptor magnitude clash — **Q-79** |
| `flaw.brutal_artist` | 5757-5760 | uncomputed_rule ✓ | **`Prereq::House(house.jerbiton)` absent though exactly expressible** | OK ✓ | F-359 |
| `flaw.busybody` | 5761-5764 | narrative ✓ **?** | the creation-time scoping choice recorded by no parameter | OK | F-382 (**DE name collides with `virtue.gossip`**) |
| `flaw.carefree` | 5765-5768 | narrative ✓ | OK | OK | **clean** |
| `flaw.careless_sorcerer` | 5769-5772 | uncomputed_rule ✓ | OK — botch dice are the enum doc's own canonical `uncomputed_rule` example | OK ✓ | **clean** |
| `flaw.careless_with_ability` | 5773-5776 | uncomputed_rule ✓ | OK — `max_per_target: 1` over an `ability` param gives exactly "more than once, each time a different Ability" | OK ✓ | **clean** |
| `flaw.castratus` | 5777-5780 | uncomputed_rule ✓ | OK — the sex-of-birth restriction is inexpressible (D3) and is in the description | OK ✓ | **clean** |

**The nine rows above whose `class` column reads `narrative → …` are collectively
F-383's evidence**: `abandoned_apprentice`, `ability_block`,
`beloved_rival_major`, `beloved_rival_minor`, `bigamist`, `blackmail`, `blind`,
`bound_casting_tools`, `bound_magic`, `branded_criminal` — ten rows for nine
passages, since the Beloved Rival pair shares one. Every one of them is inside the
twice-swept Flaws block.

**Fully clean: 10** — `flaw.ambitious_major`, `flaw.ambitious_minor`,
`flaw.animal_companion`, `flaw.arthritis`, `flaw.avaricious_major`,
`flaw.avaricious_minor`, `flaw.carefree`, `flaw.careless_sorcerer`,
`flaw.careless_with_ability`, `flaw.castratus`. **With at least one failed
check: 24.** `flaw.broken_vessel` is the eleventh entry that fails none of the
twelve checks — its data and text are correct and its inexpressible prerequisite
is handled exactly as D3 requires — and it is counted with the failures only
because the book contradicts itself about its magnitude (Q-79), which is not a
defect of the entry. All eleven went to the verification sub-agent.

`flaw.busybody`'s missing parameter (ArMDE:5763) is folded into **Q-86** rather
than carrying a finding number of its own, because whether it is a defect at all
is the question that section asks.

## Findings

### F-349 — `virtue.wise_one` — a two-category gated Ability permission with no `ability_authorization`, and `narrative` denying it

> "You may take either Arcane or Academic Abilities, but not both, at character
> creation." — ArMDE:5259
>
> DE 5259: "Du kannst bei der Charaktererschaffung entweder Arkane oder
> Akademische Fertigkeiten wählen, aber nicht beides."

`rules/core/abilities.json:31` sets
`"categories_requiring_virtue": ["academic", "arcane", "martial"]`, so **both**
categories this Virtue permits are gated. `validation/authorization.rs::validate_ability_authorization`
errors `ability_category_requires_virtue` on a held Ability in a gated category
unless a `RestrictedAbilityXp` or an `AbilityAuthorization` names its id or its
category, with a whole-character exemption only for a profile whose `is_magus` is
true. Wise One is `*Minor, Social Status*`, taken by grogs and companions, so the
exemption is empty: **a legal Wise One with an Arcane Ability raises a hard
validator error today.** This is the ninth consecutive batch to find this shape.

The entry carries **no `effects` at all** (`jq` over its catalogue object returns
the key set `id, kind, magnitude, categories, classification, entity_kinds,
source`), and `classification: "narrative"` asserts that the passage states
nothing mechanical — which this sentence refutes on its own.

**The second half is the trap, and it is the `virtue.privileged_upbringing`
mirror the audit already has a name for.** `Effect::AbilityAuthorization` has two
fields, `abilities` and `categories`, and **no `param`** — it is a static list.
"Either … but not both" is a *player choice between two categories*, and the
engine has no way to say that: authorizing `["arcane", "academic"]` grants exactly
what the passage forbids, and over-permission raises no error, so no test would
catch it. Per **D3** the honest shape is `ability_authorization` for neither
category *or* the correction slice inventing a parameter-relative form; the
choice is not this batch's to make. What *is* this batch's verdict: `narrative`
is wrong, and the rule reaches the user nowhere.

**Severity: high** — a hard error on a legal character, plus a lost rule.
**Discovery** (new instance of a known pattern).

### F-350 — `virtue.wise_one` — three clauses reach neither locale (D5)

The shipped text is `name` + a one-sentence `summary` ("A mystic, seer, or healer
of good standing in the community." / "Ein Mystiker, Seher oder Heiler von gutem
Ansehen in der Gemeinschaft."), and **no `description` in either locale**. Note
that the summary is not even a compression of the passage — it is a *paraphrase*
of ArMDE:5259's first two sentences that silently drops the rest. Three
clauses of ArMDE:5259 therefore reach no user:

1. The either/or Ability permission (F-349).
2. **"The Wealthy Virtue and Poor Flaw affect you normally."** This is worth
   carrying precisely because it is a *denial* of an exclusion: eight core
   passages state a Wealthy/Poor exclusion, one of them four lines from this
   batch's own `flaw.branded_criminal` (F-376), so a player who has learned the
   pattern will assume one here. The passage says there is none.
3. **"This Virtue is available to male and female characters."** The engine has
   no sex axis, so this can only ever be text.

**Severity: medium.** **Discovery.**

### F-351 — `virtue.withstand_casting` — the `casting_fatigue` sign is the opposite of the one the engine documents, and of the one both Flaw carriers ship

The data:

| id | kind | effect | book |
|---|---|---|---|
| `virtue.withstand_casting` | virtue | `health_mod { track: casting_fatigue, amount: **1** }` | "he loses **1 less** Fatigue level than normal" ArMDE:5263 |
| `flaw.vulnerable_casting` | flaw | `health_mod { track: casting_fatigue, amount: **-1** }` | "she loses **1 more** Fatigue level than normal" ArMDE:6995 |
| `flaw.painful_magic` | flaw | `health_mod { track: casting_fatigue, amount: **-1** }` | "suffer the equivalent of **one** Fatigue level in pain for each spell" ArMDE:6576 |

The shipped data is internally consistent under the convention *positive = better
for the character*. `types.rs::HealthTrack::CastingFatigue`'s own doc comment
states the **opposite** convention, naming these very entries:

> "Fatigue levels lost per spell cast (Vulnerable Casting **+1**, Withstand
> Casting **−1**, Painful Magic). Surfaced-only."

So every one of the three shipped values contradicts the comment that defines the
track. Only one of the two can be right, and the user-facing label does not
settle it: `locales/en/main.ftl:1366` is
`derived-detail-casting_fatigue = Casting fatigue` and
`locales/de/main.ftl:1426` is `= Zauber-Erschöpfung`. Under a label that names
*the fatigue*, a reader parses the number as an amount of fatigue — which makes
the shipped `+1` on a Virtue that **reduces** fatigue read backwards, and the
shipped `-1` on Vulnerable Casting read backwards too.

The track is surfaced-only (C1), so nothing miscomputes — but the number is
printed, and printed with a sign that the engine's own definition of the track
calls wrong. **This is not a Part C re-report**: C1 records that the track is not
computed; it says nothing about which sign the data should carry.

**Severity: medium** (a displayed value whose sign is ambiguous in both locales,
on three entries). **Discovery.** See **Q-81** — the fix could equally be the
comment, the label, or all three amounts, and the audit does not pick.

### F-352 — `virtue.withstand_casting` ↔ `flaw.vulnerable_casting` — a creation-time incompatibility stated twice and encoded on neither side

> "You may not start your character with both Vulnerable Casting and Withstand
> Casting. However, she may acquire both in the course of play …" — ArMDE:5269
>
> "You may not start your character with both Vulnerable Casting and Withstand
> Casting, but the maga may acquire either in the course of play …" — ArMDE:7003

The book states it from both ends. `jq` over both catalogue objects: neither
carries an `incompatible_with` key at all.

The one subtlety that makes this *not* a trivial fix, and which the correction
slice must not miss: the prohibition is **at character creation only**, and
`validation/prereq.rs::validate_incompatibilities` is a creation-time validator
over `entity.selections` — so an `incompatible_with` pair models exactly the rule
the book states, and nothing else. The "may acquire both in play" half needs no
machinery, because the app builds starting characters. It does, however, belong in
the description (F-353).

**Severity: high** (a stated rule, exactly expressible, absent — and its absence
lets an illegal starting magus validate clean). **Discovery.**

### F-353 — `virtue.withstand_casting` — a 22-line passage reaches the user as one sentence (D5)

The entry has **no `description` in either locale**; the `summary` is ArMDE:5263's
first sentence, verbatim in both. Everything else in ArMDE:5261-5282 reaches no
user:

- **the minimum** — "with a minimum of 1 Fatigue level", and "if the spell would
  have been cast without Fatigue loss, then he still loses no Fatigue" (:5263);
- **the per-level scaling** — "withstand 1 Fatigue level for each level of the
  Virtue … Withstand Casting (3) … up to 3 Fatigue levels lost" (:5265), the one
  clause `RULES.md:1529` already quotes, and the justification for the entry's
  `max_per_target: 255`;
- **the severity rule** — short-term vs long-term Fatigue, the unconsciousness
  hours, the wound reduction on Rituals (:5267);
- **the ordering rule** — "apply the extra loss from Vulnerability first, then
  withstand the increased loss" (:5269, restated :7003);
- **the forfeiture** — "Some Virtues, such as Life Boost and Life-Linked
  Spontaneous Magic, actually use the magus's life-force … if he does then he
  loses the benefits from withstood levels" (:5271).

The last is the one with teeth: it is a rule under which using this Virtue
*costs* the magus something, and it names two other catalogue entries.

**Severity: medium.** **Discovery.**

### F-354 — `flaw.abandoned_apprentice` — a character-creation procedure and a hard consequence, classified `narrative`

> "Decide at what age the character was abandoned. **Create the character as a
> regular apprentice up until that age, and then give him experience points based
> on his age and other Virtues for his life past being abandoned.** If the
> character knows the Parma Magica, **he must join the Order or be slain**." —
> ArMDE:5647

The first is a character-generation instruction that changes how the whole
advancement is laid out — precisely the kind of thing `life_stage.rs` exists for.
The second is an absolute. Neither is colour, so `classification: "narrative"` is
false: the correct value is `uncomputed_rule` (the engine has no
"apprenticeship interrupted at age N" life stage), and under D5 both clauses must
be written into `description` in both locales. Today the entry carries `name` +
`summary` only, in both.

Two further clauses of the same passage are equally absent: "He is not a member of
the Order of Hermes" (:5643) — a status that matters, because the magus profile's
`required_traits` is `["virtue.hermetic_magus"]` — and the Tribunal consequences
at :5645.

**Severity: medium.** **Discovery.** See **Q-84**.

### F-355 — `flaw.ability_block` — a whole-category learning block under `narrative`, with no parameter recording which category

> "You are completely unable to learn a certain class of Abilities … This may be
> Martial Abilities, or a more limited set of the others … **It must be possible
> for your character to learn the abilities in question in the absence of this
> Flaw** … You may only take this Flaw once." — ArMDE:5653

Three things:

1. **`narrative` is wrong.** A complete inability to learn a class of Abilities is
   the mirror image of `ability_authorization` and is unambiguously mechanical.
   The engine has no *negative* authorization variant, so per D3 the class is
   `uncomputed_rule` and the rule goes in the description — which the entry does
   not have, in either locale.
2. **No parameter records the blocked class.** Compare `flaw.careless_with_ability`
   four entries later, which carries `{ key: "ability", domain: "ability" }`, and
   `flaw.anchored_to_the_land`, which carries a free-`text` `land`. A `text`
   parameter is the shape that fits here, because the book's own examples
   ("Martial Abilities", "Artes Liberales, Philosophiae, any Law, Medicine, and
   Theology", "any languages other than your native tongue") are not all
   categories. Without it, two Ability Blocks are indistinguishable and the choice
   is not saved.
3. **The proviso** — the blocked Abilities must be ones the character *could*
   otherwise learn — is a constraint on the choice, and reaches no user.

**"You may only take this Flaw once" is explicitly NOT a finding**, and the
reasoning matters because it is the opposite of F-358's: the entry declares no
parameters, so `max_per_target`'s **default of 1** already groups every copy under
the same `(item_ref, {})` tuple and `validate_duplicate_selections` rejects the
second. The absent `max_total` is a sentinel meaning "no stated ceiling" where the
book states one, but it changes no behaviour here.

**Severity: medium.** **Discovery.**

### F-356 — `flaw.afflicted_tongue` — a stated condition folded in flat, on a channel that prints

```json
{ "type": "casting_total_mod", "amount": -2, "scope": "all" }
```

> "You suffer a -2 penalty to all rolls involving the voice, and all Casting
> Scores **if you cast the spell using words**." — ArMDE:5657

`types.rs::CastingScope` has exactly five values — `All`, `Formulaic`, `Ritual`,
`FormulaicRitual`, `Spontaneous` — none of which is a words/gestures axis, so
`scope: "all"` is the widest available and applies the −2 to every Casting Total
the read-out shows, including spells cast without words.

What makes this more than a shrug is that **the engine already knows the axis
exists**: `types.rs::SpecialCasting` carries `QuietWords` and `SubtleGestures` as
first-class kinds. So the condition the data drops is one the engine models
elsewhere, on the same subsystem.

This is the D4 shape on a different channel. **D4 itself does not reach here** —
it rules on `lab_total_mod` and explicitly scopes itself — so this is reported as
its own instance, and it is *worse* than the lab cases in one respect: `RULES.md:5131`
records that conditional `casting_total_mod` rows are folded "**unconditionally**
(no toggle exists)" and the result is "surfaced per scope as the
`casting_mod_formulaic`/`_ritual`/`_spontaneous` addends" — i.e. the wrong number
is *printed*, not merely used as a ceiling the way D1's cap is.

The entry's text half is clean: the `description` in **both** locales carries the
whole passage including the condition and the extra botch die ✓.

**Severity: medium** (wrong printed number in a narrow case; the condition is
recorded, so the fix is a scope decision, not a rules read). **Discovery of the
instance; corroboration of the folding, which `RULES.md:5131` states.**

### F-357 — `flaw.age_quickly` — a schedule rule that reaches neither locale (D5)

> "Your effective age (which applies as if it were your actual age when creating a
> Longevity Ritual, and when making rolls on the aging table) **increases two
> years for every year that passes**, and **you make two aging rolls each
> year**." — ArMDE:5661

The entry ships `aging_mod { kind: "aging_roll", amount: 0 }`. The amount is
genuinely inert — `AgingEffect::AgingRoll`'s doc says it is "added to the AGING
TOTAL with its stored sign", and 0 adds nothing — **and that is a recorded
decision, not an oversight**: `RULES.md:5139` states that age_quickly and
baneful_circumstances "stay surfaced-only … (amount 0; **schedule rules, not
modifiers**)". So that half is **corroboration** and no finding lies there.

The finding is the other half. The entry carries `name` + `summary` only, in both
locales, and the summary is the passage's *first* clause ("Probably due to a curse
or a magical disaster, you age twice as fast as normal people."). **Neither
number reaches any user**: not the doubled effective age, not its application to
the Longevity Ritual, and not the two rolls a year. Under D5 a mechanical clause
the engine does not compute must be written into `description` in both locales
whatever the classification — and `in_play_effect` carries no such obligation
today, which is exactly the hole D5 was taken to close.

**Severity: medium.** **Discovery** (the D5 half); the inert amount is
corroboration.

### F-358 — three parameterized Flaws have no `max_total`, so rules ArMDE:2814 permits once are unlimited

> "A Virtue or Flaw may be taken more than once **only if the description
> explicitly allows it**. Most Virtues and Flaws may only be taken once." —
> ArMDE:2814

`max_total` defaults to **255** ("no stated ceiling") and is keyed on `item_ref`
alone, while `max_per_target` defaults to 1 and is keyed on `(item_ref, whole
params map)`. For an entry with **no** parameters the default already enforces
"once". For a **parameterized** entry it does not: two copies naming different
values are two different targets, both legal.

Three entries in this batch are parameterized, and **none of their descriptions
allows a repeat**:

| id | parameter | what the absence permits | what the passage says |
|---|---|---|---|
| `flaw.anchored_to_the_land` | `land` (`text`) | one copy per land, unbounded | ArMDE:5669 describes "a particular type of environment", singular; no repeat clause |
| `flaw.bound_to_realm` | `realm` (`realm`) | up to four copies, one per realm | ArMDE:5733 "**Choose the realm** … to which the character is bound when you take the Flaw" — singular, no repeat clause |
| `flaw.bound_to_role_role` | `role` (`text`) | one copy per role, unbounded | ArMDE:5735-5748 describes one bonding; no repeat clause |

The fourth parameterized entry in the batch, `flaw.careless_with_ability`, is
**correct** and is the control that proves the shape: ArMDE:5775 says "This Flaw
may be taken more than once; each time, it applies to a different Ability", and
its data ships exactly that — `max_per_target` at its default of 1 over an
`ability` param, so a repeat on the *same* Ability is rejected and a repeat on a
different one is not. It needed no `max_total`; the other three do.

This is B09's F-348 shape, found by asking what the **absence** of `max_total`
means rather than what the present fields say. `flaw.bound_to_realm` is the
sharpest of the three because its parameter domain is closed at four values, so
the defect is precisely "a character may be Bound to all four Realms at once".

**`RULES.md` has already ruled on this exact class, twice, and in a way that
settles it.** Its ArMDE:2814 section (`RULES.md:1466-1553`) separates
"repeatable" into two shapes and **enumerates by id** the items of shape 1
(repeats with a different target each time, modelled by a parameter):
`virtue.affinity_ability`, `virtue.affinity_art`, `virtue.puissant_ability`,
`virtue.puissant_art`, `virtue.extractor_of_form_vis`,
`virtue.master_of_form_creatures`, `virtue.student_of_realm`,
`virtue.ways_of_the_land`, **`flaw.careless_with_ability`**,
`flaw.flawed_parma_magica`, `flaw.limited_magic_resistance` — each cited to the
line of its descriptor that grants the repeat. That list corroborates this
batch's clean verdict on `flaw.careless_with_ability` (ArMDE:5775 is cited at
`RULES.md:1493`) and it does **not** contain any of the three entries above.

And `RULES.md:1534-1542` records the repo *fixing* the identical over-permission
on two entries: `flaw.flawed_parma_magica` and `flaw.limited_magic_resistance`
"used to sit in that table and no longer do … The 255 they shipped with predated
the parameter and **was over-permissive** — it also allowed a second, identical
copy naming the *same* Form, which neither descriptor grants." The three entries
here are the same mistake in its other half: not an over-permissive
`max_per_target`, but an absent `max_total`, on entries whose descriptors grant
no repeat at all. The two pins that came out of that fix
(`data_integrity.rs::the_form_scoped_magic_resistance_flaws_name_their_form` and
`::a_form_scoped_mr_flaw_repeats_across_forms_but_never_within_one`) are
Form-specific, so nothing catches these three.

**Severity: medium** (a rules-legality gap on three entries; no wrong number).
**Discovery**, corroborating B09's pattern and extending a fix `RULES.md` already
records.

### F-359 — `flaw.brutal_artist` — a House restriction the `Prereq` tree expresses exactly, and that is absent

> "This Flaw is only available to magi of **House Jerbiton**." — ArMDE:5759
>
> DE 5759: "Dieser Fehler steht nur Magi des Hauses Jerbiton zur Verfügung."

`Prereq::House(Id)` exists for precisely this, it is the first sentence of the
passage, and `jq` returns no `prerequisites` key on the entry. The catalogue
already ships the identical shape one book-page away —
`virtue.verditius_magic`'s `{ kind: "house", value: "house.verditius" }`,
recorded at `RULES.md:3760-3775` — so this is not a missing mechanism but a
missing row.

`house.jerbiton` resolves: it is one of the twelve Houses, and
`ruleset/integrity.rs::validate_prereq_refs` would fail the load if it did not.
Evaluation is tri-state, and `Prereq::House` on a character with **no** house
yields `Unknown` → `prereq_unevaluated` (a warning), not an error — so adding the
row does not break grogs and companions, who simply get no verdict.

The restriction does reach the user as text: the `description` opens with it in
both locales ✓. So this is a validation gap, not a lost rule.

**Severity: medium.** **Discovery.**

### F-360 — `flaw.beloved_rival_major` / `_minor` — the data forbids the exact combination ArMDE:5697 permits, and the load-time guard forces it to

> "Such relationships are a tradition of House Tytalus, and may be mutual. **When
> the rivalry is mutual, the troupe may allow the character to take both the Minor
> and Major versions of this Flaw.** This will mean that the rivalry will be a very
> important feature of the whole saga …" — ArMDE:5697
>
> DE 5697: "Ist die Rivalität gegenseitig, **kann die Spieltruppe dem Charakter
> erlauben, sowohl die Kleine als auch die Große Version dieses Fehlers zu
> nehmen.**"

The data says the opposite:

```json
"flaw.beloved_rival_major": { "incompatible_with": ["flaw.beloved_rival_minor"] }
"flaw.beloved_rival_minor": { "incompatible_with": ["flaw.beloved_rival_major"] }
```

`validation/prereq.rs::validate_incompatibilities` raises `incompatible` — an
**error**, not a warning — on a character holding both. The book says a troupe
may allow it.

**This is the batch's structural finding, because the data cannot simply be
corrected.** `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`
**requires** `<stem>_major`/`<stem>_minor` pairs to list each other, checked at
load and failing the whole ruleset otherwise. Deleting the two rows would make
the ruleset unloadable. So expressing ArMDE:5697 needs either an opt-out on that
guard or a different id shape — a decision, not an edit.

`classification: "narrative"` is wrong on both sides for the same reason: ArMDE:5697
states a permission about how many copies of a catalogue item a character may
hold, which is mechanical by any reading, and the permission reaches no user
(neither entry carries a `description`, in either locale). Note also that both
`summary` fields, in both locales, begin "**For a Minor Story Flaw**, the
character has a rival…" — correct for `_minor`, wrong for `_major`, whose own rule
is the *next* paragraph (ArMDE:5695, "If the character takes on the role of the
beloved rival for another character, then this is a Major Flaw").

**Severity: high** — a hard validator error on a build the rulebook explicitly
sanctions, plus a wrong summary on one of the two. **Discovery.**

### F-361 — `flaw.baneful_circumstances` — an aging rule that overrides aging immunity, reaching neither locale (D5)

> "At these times, the character cannot recover Fatigue, heal wounds, or recover
> Might, and if at the end of the year the character has spent more than half of
> his time subject to these conditions, he must make **an additional Aging roll
> even if he is normally immune to aging because of a Longevity Ritual or Might
> Score**." — ArMDE:5689

As with F-357, the entry's `aging_mod { kind: "aging_roll", amount: 0 }` is inert
by **recorded** decision (`RULES.md:5139` names this entry by id), so that is
corroboration. The finding is that the entry carries `name` + `summary` only in
both locales, and the summary stops at "Something about the character's
supernatural nature weakens him in relatively common circumstances…" — so three
clauses reach no user: the no-recovery rule, the additional aging roll, and the
**immunity override**.

The override is the one that deserves weight. `AgingEffect::NoAging` is a real
computed marker — `aging.rs` gates the Characteristic drop on it — and this
passage says an aging immunity is *beaten* under a stated condition. A player
holding both `virtue.unaging` (or a Longevity Ritual) and this Flaw sees an
unqualified "no aging" in the read-out and nothing anywhere telling him it can
lapse.

**Severity: medium.** **Discovery.**

### F-362 — `flaw.baneful_circumstances` — no parameter records the circumstance

The passage requires a choice — "Something about the character's supernatural
nature weakens him in **relatively common circumstances, such as when touching
the ground or when in the presence of women**" (ArMDE:5689) — and the entry
declares no `parameters`. The Flaw is therefore indistinguishable from any other
copy and the choice is not saved.

The batch's own `flaw.anchored_to_the_land` is the template: a `text`-domain
parameter whose value the engine does not compute from but which the picker makes
the player fill and the Markdown export renders (engine-semantics § B8, "What
happens to a parameter no effect consumes"). The English name has no `{…}` slot
today, so adding one is a text change as well as a data change.

**Severity: low-medium.** **Discovery.** See **Q-86** — whether every
"choose the specifics" Flaw should carry such a parameter is a catalogue-wide
convention question, not this entry's alone.

### F-363 — `grants_reputation` cannot say a Reputation is **bad**, and both carriers in this batch are bad ones

> "You have a **bad** Reputation at 4 among members of your previous faith" —
> ArMDE:5677 (`flaw.apostate`)
>
> "You begin the game with a **bad** Reputation of your choice at level 2, among
> those who respect your family." — ArMDE:5705 (`flaw.black_sheep`)

`Effect::GrantsReputation` has exactly two fields, `kind: Option<ReputationType>`
and `score: u8`, and `types.rs::Reputation` has three — `kind`, `score`,
`content: String`. **There is no polarity field anywhere.** A granted Reputation
of 4 from a Flaw and a granted Reputation of 4 from a Virtue are the same object.

`RULES.md:5373` writes the word out — "`flaw.apostate` (ArMDE:5675-5678) —
reputation grant **bad** score 4" — so the traceability map records a property the
type cannot hold. The German translation table records it too, in its own
notation: `reputationen.md:102` gives `Apostate … Kirchlich | 4 (–)` and
`reputationen.md:105` gives `Black Sheep … 2 (–)`, where `(–)` is that table's
marker for a negative Reputation. Three independent sources say "bad"; the data
model says nothing.

The user can still type the polarity into `content` by hand, which is why this is
not higher: the *number* is right and the free-text field can carry the sense. But
nothing requires it, nothing displays it, and `validation/scores.rs::validate_reputations`
counts kinds and slots only, so a Flaw's Reputation and a Virtue's are
interchangeable to every check.

**Scope of this claim, stated narrowly:** I read `types.rs::Effect::GrantsReputation`
and `types.rs::Reputation` in full and neither declares a polarity, positivity,
`good`/`bad` or sign field; I did not grep the whole crate for some other
representation. Part C does not list this — C2-c records the ignored `score`, not
the absent polarity — so as far as this batch can tell it is unrecorded.

**Severity: medium** (a stated property of two shipped grants, and of others
outside this span, that the model cannot hold). **Discovery.**

### F-364 — `flaw.bigamist` — two numbers under a `narrative` class

> "If you are using the rules in *City and Guild*, the merchant's annual cost of
> maintaining his business rises by **(6 x Wealth Multiplier)** Labor Points. Some
> bigamists mitigate this expense by pretending to be of lower status in their
> alternate life, which reduces the additional Labor Point cost by half (to **3 x
> Wealth Multiplier**)." — ArMDE:5701

*City and Guild* is not in `rules/source/en/`, so the rule **cannot be
implemented** — CLAUDE.md's provenance rule is explicit about that, and this
finding does not ask for it to be. But the classification question is a different
one: `narrative` claims the *passage* states nothing mechanical, and ArMDE:5701
states two formulas. The correct value is `uncomputed_rule`, and under D5 the
clause belongs in `description` in both locales, which today are `name` +
`summary` only.

This is the cleanest example in the batch of the distinction the README draws:
"the engine does not compute this" is never a reason to call something
`narrative`, and neither is "the supplement is missing".

**Severity: low-medium** (a lost rule, not a wrong number). **Discovery.**

### F-365 — `flaw.blackmail` — a quantified yearly benefit under a `narrative` class

> "This benefit has a yearly value of about **50 silver pennies**, possibly more if
> you keep the pressure on. You should detail and record the specifics of this
> arrangement." — ArMDE:5709

`types.rs::Classification`'s own doc says an entry whose text states "a signed
modifier, a botch-dice change or a cap" is not narrative even when the engine
computes nothing. 50 silver pennies a year is a quantified benefit hedged by
"about", which is not quite any of those three — which is why this is filed with a
`?` on the class rather than as a flat reclassification. The second sentence is
also a creation-time instruction ("detail and record the specifics"), and nothing
records it: the entry declares no parameter.

**Severity: low.** **Discovery.** See **Q-83**.

### F-366 — `flaw.blatant_gift` ↔ `flaw.blatant_magical_air` — a stated mutual exclusion missing from both sides

> "This effect is the same as the Blatant Gift, and **a character may not have
> both Flaws**." — ArMDE:5717
>
> DE 5717: "Dieser Effekt entspricht der Auffälligen Gabe; **ein Charakter kann
> nicht beide Fehler besitzen.**"

`flaw.blatant_gift` ships `incompatible_with: ["flaw.unbearable_to_beings",
"virtue.gentle_gift"]` — **and both of those are right**: the first is the
mandatory reverse edge of ArMDE:6895, documented at `RULES.md:921`, and the second
is the Gentle/Blatant clique at `RULES.md:5122`. `flaw.blatant_magical_air` ships
**no `incompatible_with` key at all**. So the one exclusion the Blatant Gift
passage's own neighbour states in plain words is the one edge missing, on a Flaw
whose list is otherwise carefully maintained.

Both sides are needed: `ruleset/integrity.rs::validate_incompatibility_symmetry`
fails the load on a one-sided declaration.

**Severity: high** (an explicitly forbidden pair validates clean; the mechanism is
in use two rows away in the same entry). **Discovery.**

### F-367 — `flaw.blatant_magical_air` — an eligibility gate the `Prereq` tree expresses exactly, and that is absent

> "**Only characters with a Magical Air or The Gift may take this Flaw.**" —
> ArMDE:5717

This is, word for word, the shape the catalogue already ships on
`flaw.unbearable_to_beings`:

| entry | prerequisite | source |
|---|---|---|
| `flaw.unbearable_to_beings` (shipped) | `Any([Has(virtue.the_gift), Has(flaw.magical_air)])` | ArMDE:6895, `RULES.md:921` |
| `virtue.inoffensive_to_beings` (shipped) | `Any([Has(virtue.the_gift), Has(flaw.magical_air)])` | ArMDE:4139, `RULES.md:923` |
| `flaw.blatant_magical_air` (**missing**) | — | ArMDE:5717 |

Both referents resolve: `virtue.the_gift` at ArMDE:3967-3970 and `flaw.magical_air`
at ArMDE:6382-6385 are in the catalogue. `Prereq::Has` reads the
**grants-inclusive** `present_ids` set, so a House- or type-granted Gift satisfies
it correctly, and an unsatisfied `Any` of two `Has` is never `Unknown` — it
resolves `False` and raises `prereq_not_met`, an error, which is what the passage
asks for.

`RULES.md:917` describes exactly this defect class on a neighbouring trio — "A
third item from the same block of the same page had the eligibility gate missing
outright" — and fixed three entries. This is a fourth, in a different block, that
the same pass did not reach.

**Severity: high** (an ineligible character validates clean; the fix is one
existing expression). **Discovery.**

### F-368 — `flaw.blatant_gift` — the rules behind "page 203" reach neither locale (D5)

The entry's `description`, in both locales, ends with the pointer itself: "…and
should see **page 203** for further discussion of this Flaw's effects." / "…weitere
Ausführungen zu den Auswirkungen dieses Fehlers findest du auf Seite 203."

The DE source hyperlinks it — `[Seite 203](#die-gabe-2)` at DE:5713 — and that
anchor is the third `Die Gabe` heading in the file, i.e. **ArMDE:8743 `## The
Gift`**. What the pointer lands on is a full chapter section, and five of its
clauses are rules about the Blatant Gift specifically:

1. **The −6 is not only a social-roll penalty.** ArMDE:8753: "If a maga with
   Blatant Gift interacts with someone, she suffers the -6 penalty **to all social
   rolls and totals**, as for the normal Gift" — and the normal Gift's version it
   points back to, ArMDE:8751, spells out what "totals" covers: "**including
   training, whether the maga is the trainer or the trainee**". So the −6 reaches
   the advancement subsystem, not just conversation.
2. ArMDE:8753: "An unGifted individual negotiating on her behalf suffers no
   penalty" — a stated way to avoid it entirely.
3. ArMDE:8757: the precautions that reduce a normal Gift's effect "**make this a
   lot harder**" for the Blatant Gift, and at best "reduce people's reactions to
   those inspired by the normal Gift" — i.e. the −6 degrades to −3, never to zero.
4. ArMDE:8765: "The Parma Magica blocks these effects of The Gift entirely" — for
   the perceiver, which is why magi are not affected by each other.
5. ArMDE:8767: "Gifted characters **cannot ride horses** without magical aid …
   Similarly, they can **never train dogs** to recognize them as friends", "with
   much more hostility if the magus has the Blatant Gift" — two absolutes.

None of this reaches either locale, and the page number the user is sent to is a
printed-book page the app cannot open.

This is the same shape as B09's two largest findings (True Faith's "page 419",
Verditius Magic's "page 240") and it is now three batches in a row in which the
numbered-page form out-produced everything else.

**Severity: medium-high** (a −6 with a documented scope that includes teaching, and
four further clauses, reaching no user in either locale). **Discovery.**

### F-369 — `flaw.blind` — three absolute prohibitions under a `narrative` class

> "Using missile weapons is futile, **reading is impossible**, and navigation in
> unknown territory is difficult to say the least. Blind magi can detect targets by
> other senses, and thus are less limited than people trying to use missile
> weapons. However, **blind magi cannot aim spells without magical aid**." —
> ArMDE:5721

"Reading is impossible" is not colour: reading is how a character studies from
books, which is a whole advancement path. "Cannot aim spells without magical aid"
is a hard constraint on Hermetic targeting. Both are mechanical absolutes, so
`narrative` is false and `uncomputed_rule` is the value, with the clauses written
into `description` in both locales — which today are `name` + a four-word
`summary` ("You have little or no sight." / "Du hast wenig oder gar keine
Sehkraft.").

Note the contrast with `flaw.castratus` four entries later, which carries an
equally inexpressible restriction (sex of birth) and is correctly
`uncomputed_rule` with the whole passage in its description in both locales. Same
shape, opposite treatment.

**Severity: medium.** **Discovery.**

### F-370 — `flaw.bound_casting_tools` — an Arcane-Connection duration rule under a `narrative` class, and an EN source sentence that is truncated

> "The magus's casting tools, as used by House Verditius, are so personal that they
> become **lasting Arcane Connections** to him. Regular casting tools remain as
> Arcane Connections for **a few weeks**, his last years." — ArMDE:5725

Arcane Connection duration is a mechanical property with its own table in the
magic chapter, and this Flaw changes it from "a few weeks" to "lasting". That is a
rule, so `narrative` is wrong → `uncomputed_rule`, and the entry carries no
`description` in either locale, so the durations reach no user.

**The EN sentence is also corrupt**, and the DE proves it. The English "…for a few
weeks, his last years." has lost a verb; the line-parallel DE:5725 reads
"Reguläre Zauberwerkzeuge bleiben wenige Wochen als Arkane Verbindungen erhalten,
**die zuletzt verwendeten bleiben es jahrelang.**" — "regular casting tools remain
Arcane Connections for a few weeks, **the last-used ones remain so for years**".
So the intended English is "…for a few weeks, **his last for** years", and the EN
source has dropped two words. Since English is the source of truth for this
repository, the rule as stated in EN is unreadable. See **Q-82**.

**Severity: medium** (lost rule) plus a **source-integrity** note.
**Discovery.**

### F-371 — `flaw.bound_magic` ↔ `virtue.harnessed_magic` — a stated incompatibility, corroborated from both ends, encoded on neither

> "You cannot take this Flaw with **Harnessed Magic** (page 85), as that Virtue
> already includes this effect." — ArMDE:5729
>
> And from the other side: "The drawback is that **when you die, all of your spells
> and magic items sputter out**." — ArMDE:4057, the last line of
> `virtue.harnessed_magic` (ArMDE:4053-4058)

So the book states the exclusion in one entry and states *why* in the other. `jq`
over both catalogue objects: `flaw.bound_magic` has no `incompatible_with` key,
and neither does `virtue.harnessed_magic`. Both edges are needed —
`validate_incompatibility_symmetry` fails the load on a one-sided declaration.

The pair is also unusually clean to encode: it is not magnitude-variant, not
conditional, and not a guideline ("cannot", not "should not").

**Severity: high** (an explicitly forbidden pair validates clean, and the
duplication it creates — paying points for a drawback already included — is
exactly the kind of silent double-charge the balance validator exists to prevent).
**Discovery.**

### F-372 — `flaw.bound_magic` — the death clause reaches neither locale (D5)

> "When you die, all of your spells end abruptly and any magic items that you
> created cease to function." — ArMDE:5729

The `summary` in both locales is that sentence, so this one clause *does* reach
the user. What does not is the second sentence — the Harnessed Magic exclusion
(F-371) — and there is no `description` in either locale to hold it. Reported
separately from F-371 because the two need different fixes: one is data, one is
text.

**Severity: low.** **Discovery.**

### F-373 — WITHDRAWN. `flaw.bound_to_realm`'s German name is a recorded decision, and a well-argued one

**This was raised as a finding and is withdrawn by the same pass that raised it.
It is recorded rather than deleted, because the reasoning is the kind a later
reader will re-derive and the withdrawal is the useful half.**

The observation that prompted it stands on its facts:

| source | form |
|---|---|
| DE rulebook heading, DE:5731 | `#### Gebunden an (Sphäre)` |
| `rules/source/de/translation-tables/tugenden-fehler.md:339` | `Bound to (Realm) \| Gebunden an (Sphäre)` |
| `rules/i18n/de/virtues_flaws.json` → `name_unfilled` | `Gebunden an (Sphäre)` ✓ |
| `rules/i18n/de/virtues_flaws.json` → `name` | `Gebunden an eine Sphäre, {realm}` |
| `rules/i18n/en/virtues_flaws.json` → `name` | `Bound to {realm}` |

With `realm.magic` chosen, the German renders "Gebunden an eine Sphäre, Magie"
where the English renders "Bound to Magic", and the guess was that this is an
unprincipled dodge of German case government.

**It is not. `RULES.md:2356-2373` gives the rationale in full, and it is
correct.** Two of the four German Realm labels are noun phrases carrying their
own article — *Das Göttliche* and *Das Infernale*
(`rules/source/de/translation-tables/sphären-mächte.md:18-21`) — so the bare
template the rulebook heading suggests would render **"Gebunden an Das
Göttliche"**. RULES.md names that exact string as one of four ungrammatical
renderings it was fixing, and applies one consistent remedy across all four
affected entries (`flaw.bound_to_realm`, `flaw.necessary_realm_aura_for_ability`,
`flaw.realm_stigmatic`, `virtue.student_of_realm`): the label stands **in
apposition after a comma, uninflected, with no noun following it**, which is
"Folk Magic's own pattern from B7". And `name_unfilled` exists precisely so the
DE rulebook's own heading is still what the user sees before he picks —
RULES.md:2375 cites `Basisregeln.md:5731` for it by line.

So the divergence from the rulebook heading is deliberate, documented,
consistent across four entries, and solves a real problem that the "obvious fix"
would reintroduce. **No finding.** The lesson for the rest of this audit: before
calling a German template ungrammatical, check `RULES.md` for the Realm/Ability
apposition pattern — four entries use it and all four look wrong at first glance.

**Q-85 is answered by the same passage and is withdrawn with this finding.** What
it asked — "is this a deliberate dodge of German case government?" — has the
answer *yes, and here is the argument*. The residue is not a question but a fact
worth carrying forward: B09's Q-77 (`virtue.voice_of_the_land`'s "des/der") is a
case the apposition pattern does **not** cover, because there the placeholder is
free text rather than a fixed label, so it cannot be moved into apposition.

**Corroboration**, not discovery.

### F-374 — `flaw.bound_to_role_role` — "only grogs" is enforced nowhere, though the mechanism exists and is in use

> "This Flaw **may only be taken by grogs** and is suitable for the Warping to a
> Pattern Minor Site Hook presented in *Covenants*, page 11." — ArMDE:5747
>
> DE 5747: "Dieser Fehler **darf nur von Grogs genommen werden**…"

The entry's `entity_kinds` is `["character"]`, which distinguishes a character
from a covenant and says nothing about character *type*. The mechanism for type
restriction is the profile's `forbidden_traits`, and
`rules/core/character_types.json` already uses it: the grog profile forbids
`virtue.the_gift` (line 62), and the magus and mythic-companion profiles each
forbid `flaw.poor` and `virtue.wealthy` (lines 97, 144). Expressing ArMDE:5747
means adding `flaw.bound_to_role_role` to the `forbidden_traits` of **companion,
magus and mythic_companion** — three existing arrays, no new machinery.

Nothing enforces it today: `supernatural` is in every profile's
`permitted_categories`, so a magus may take this Flaw and validate clean.

This is the exact mirror of B09's F-318 (`virtue.temporal_influence`, "Grogs may
not take this Virtue"). Two instances in consecutive spans, opposite directions,
same unused mechanism.

**Severity: medium-high.** **Discovery.**

### F-375 — `flaw.bound_to_role_role` — four clauses reach neither locale (D5)

The entry carries `name` + `summary` only, in both locales, and the summary is the
first sentence. ArMDE:5735-5748 is fourteen lines; what reaches no user:

- **the deprivation rule** — "If separated from the bonded device or place in any
  way, the character must make deprivation checks **as though deprived of food
  (i.e., a check every three days)**" (:5741), and the cancellation on rebonding;
- **the Unaging inclusion and its carve-out** — "This Flaw also includes the
  effects of the Unaging Virtue, **but the character's apparent age advances in
  line with their physical age**" (:5743) — which is the *justification* for the
  data's single `no_aging` tag and therefore the one clause a player needs in
  order to read the aging read-out correctly;
- **the retained needs** — sleep, breath, and the other physical needs of
  place-bound characters (:5743);
- **the grog restriction** (:5747, F-374).

The data half of the aging question is **correct and recorded**:
`virtue.unaging` ships both `no_aging` and `no_apparent_aging`, this Flaw ships
`no_aging` alone, and `RULES.md:7034` tabulates exactly that split, citing
`ArMDE:5743`. `types.rs::AgingEffect`'s own doc names this entry as the reason the
two markers are separate tags ("Bound to (Role) has the first without the
second"). So the effect is right; only the text is missing.

**Severity: medium.** **Discovery** (the D5 half); the tag split is
**corroboration**.

### F-376 — `flaw.branded_criminal` — a Wealthy exclusion, one of the eight passages behind the confirmed catalogue-wide hole

> "**You may not take the Wealthy Virtue**, but you may take Martial Abilities at
> character creation." — ArMDE:5751
>
> DE 5751: "**Du kannst die Wohlhabend-Tugend nicht nehmen**, darfst jedoch bei
> der Charaktererschaffung Kampffertigkeiten wählen."

`flaw.branded_criminal` carries no `incompatible_with` key, and neither does
`virtue.wealthy`. Both sides are required by
`validate_incompatibility_symmetry`.

**The narrow claim, stated exactly as I checked it.** I ran

```
jq -r '[.[] | select((.incompatible_with // [])
        | any(. == "virtue.wealthy" or . == "flaw.poor"))] | length'
       rules/core/virtues_flaws.json
```

over the whole catalogue and it returned **0**; and `jq` over `virtue.wealthy`
and `flaw.poor` themselves returns no `incompatible_with` key on either. That is
the entire claim. I did **not** check — and it is **not true** — that every
entry's `incompatible_with` is empty: it is populated on many entries, seven of
them in this batch alone (`flaw.ambitious_major`, `flaw.ambitious_minor`,
`flaw.avaricious_major`, `flaw.avaricious_minor`, `flaw.beloved_rival_major`,
`flaw.beloved_rival_minor`, `flaw.blatant_gift`). B09 reported the wide version
by mistake; this is the narrow one.

ArMDE:5751 is one of the eight passages B09 enumerated (:3611, :3631, :4494,
:4634, :4850, :5181, :5751, :6540), so this instance was predicted; what this
batch adds is the verification that the entry at the other end of it is equally
bare, and one *non*-instance worth recording: `virtue.wise_one` at ArMDE:5259
states the **opposite** ("The Wealthy Virtue and Poor Flaw affect you normally"),
so the two Social Statuses four hundred lines apart in this same span go opposite
ways and neither is encoded.

**Severity: high.** **Corroboration** of the known hole; **discovery** of this
instance and of the paired non-instance.

### F-377 — `flaw.branded_criminal` — a Martial permission with no `ability_authorization`, and `narrative` denying it

> "…but **you may take Martial Abilities at character creation**. You may choose
> not to take such abilities, if your crime was not violent." — ArMDE:5751

`martial` is one of the three gated categories
(`rules/core/abilities.json:31`), so a Branded Criminal who takes the Martial
Abilities the book grants him raises `ability_category_requires_virtue`. Unlike
F-349's either/or, this one is **exactly expressible** —
`ability_authorization { categories: ["martial"] }`, a single row.

**A census worth recording here, because it sharpens every instance of this
pattern across nine batches.** `jq` over `rules/core/virtues_flaws.json` returns
**2** entries carrying an `ability_authorization` effect at all
(`flaw.covenant_upbringing`, `virtue.student_of_realm`, matching
engine-semantics § A14's "Usage 2"), and **0** entries whose
`ability_authorization` uses the **`categories`** field — both shipped users name
individual `abilities` (`ability.dead_language`; the four Realm Lores). Ten
entries do use `restricted_ability_xp`'s parallel `categories` list, which feeds
the same union in `effective/xp.rs::ability_authorizations`, so the *union* is
exercised — but `AbilityAuthorization::categories` itself is authored by no
shipped entry. Every one of the 50-plus missing-permission findings this audit has
raised would therefore be its **first** author, and a correction slice should
expect to be exercising an untested field rather than copying a pattern.
*(Scope: I ran the two `jq` counts above over that one file; I did not check
whether any test fixture authors the field.)*

`classification: "narrative"` asserts the passage states nothing mechanical while
it states two rules in one sentence (this and F-376), so the class is wrong; with
the authorization added the value is `creation_effect`. Both clauses, and the
"you may choose not to" rider, reach neither locale — the entry has no
`description` in either.

**Severity: high** (hard error on a legal character). **Discovery** (new instance
of the 50+ pattern).

> **Line numbers in `rules/source/de/translation-tables/` moved under this batch
> and will move again.** `git status` shows `tugenden-fehler.md` modified in the
> working tree by another workstream throughout this run. Every table citation in
> F-378 … F-381 was resolved by searching for its **English key** and is current
> as of this writing; a correction pass must do the same rather than trusting the
> line number. The tables cited are `tugenden-fehler.md`, `reputationen.md`,
> `grundbegriffe.md` and `persoenlichkeitseigenschaften.md`; only the first is
> modified in the working tree.

### F-378 — `tugenden-fehler.md:75` names an English Virtue that does not exist, and the DE rulebook contradicts itself about the same entry

Two intertwined problems, reported together because neither is intelligible
alone.

**(a) The table row.** `rules/source/de/translation-tables/tugenden-fehler.md:75`
reads:

```
| Withstand Magic | Magie widerstehen | |
```

There is no Virtue called **Withstand Magic**. The catalogue has
`virtue.withstand_casting`, the EN heading at ArMDE:5261 is
`#### Withstand Casting`, and the book's Virtue index uses the same words. Since
the tables are keyed on the English term, a row whose English key matches nothing
cannot bind any German name — so `virtue.withstand_casting`'s German name is, as
far as the tables go, unconstrained.

**(b) The DE rulebook disagrees with itself.** The German heading at DE:5261 is
**"Zaubern aushalten"**, and the body of that same entry calls the Virtue
**"Magie widerstehen"** six times — DE:5265 ("Hat er **Magie widerstehen** (3)…"),
:5269, :5271, :5275, :5277, :5279. The shipped data uses the heading form,
`"name": "Zaubern aushalten"`, which is defensible; but a German-speaking user who
looks the Virtue up in the rulebook finds the body calling it something else, and
the table backs the body.

Per D6's precedence (rulebook > thematic table > `tugenden-fehler.md`) the
rulebook wins — except that here the rulebook is the thing that is inconsistent.
Per the brief, the source project `arm-de-translation` is outside this repository,
so this is stated as: **this repository's copy of `tugenden-fehler.md` carries an
English key that resolves to nothing in the rulebook, and the DE rulebook's own
heading and body disagree.** Whether the table row is an error or a stale copy of
one is not determinable from here.

**Severity: medium** (the table is a generator of future wrong data, per D6's
rationale). **Discovery.**

### F-379 — `tugenden-fehler.md:429` gives a German name the DE rulebook does not use

```
| Anchored to the (Land) | Verankert im/am (Land) | SdM:M; Klein; −3 auf alle Würfe außerhalb des Heimatlebensraums |
```

The DE rulebook heading at DE:5667 is **"Verwurzelt im (Land)"** and the body at
DE:5669 uses "verwurzelt" throughout ("Ein Charakter, der zum Beispiel am Meer
**verwurzelt** ist…"). The shipped data uses `"Verwurzelt im {land}"` — i.e. the
data follows the **rulebook**, and the table is the outlier. Per D6's precedence
the rulebook wins, so the finding is against the table.

Two riders on the same row, recorded because D6 makes them relevant:

- The `Anmerkung` column attributes the Flaw to **`SdM:M`** (*Sphären der Macht:
  Magie* = RoP:M), while the catalogue sources it to **ArMDE:5667-5670** and the
  passage is plainly in the core book. A terminology table is not authoritative
  for provenance, and this one is asserting some.
- The same column states the rule ("−3 auf alle Würfe außerhalb des
  Heimatlebensraums"), which is D6's "facts about the rules" over-reach — and it
  writes the minus as **U+2212**, which is the glyph CLAUDE.md bans for displayed
  values. The translation tables are not a display surface, so that is a note, not
  a finding; but a correction pass copying the number out of the table must
  convert it.

**Severity: medium.** **Discovery.**

### F-380 — `reputationen.md:102` gives a German name the DE rulebook and the other table both contradict

```
| Apostate | Abtrünniger | Kirchlich | 4 (–) | Glaubensabfall |
```

The DE rulebook heading at DE:5675 is **"Apostat"**, and
`tugenden-fehler.md:771` independently gives **"Apostat"** as well. The shipped
data uses "Apostat" ✓. So `reputationen.md`'s **Abtrünniger** disagrees with the
rulebook *and* with the other table in this same repository.

**The same row is otherwise excellent evidence and is cited approvingly
elsewhere in this batch**: `Kirchlich | 4 (–)` independently corroborates the
entry's `grants_reputation { kind: "ecclesiastical", score: 4 }` ✓ and supplies the
third witness for the missing polarity (F-363). So the row is right about the
mechanics and wrong about the name — which is the inverse of the usual D6 failure
and worth noting as such.

**Severity: low-medium.** **Discovery.**

### F-381 — `grundbegriffe.md:579` and the shipped name reverse the DE rulebook's word order for Wise One

| source | form |
|---|---|
| DE rulebook heading, DE:5257 | `#### Weiser Mann / Weise Frau` |
| `grundbegriffe.md:579` | `Wise One \| Weise Frau / Weiser Mann` |
| `rules/i18n/de/virtues_flaws.json` | `Weise Frau / Weiser Mann` |

The data follows the table; the table reverses the rulebook. Per the precedence
recorded in `rules/source/de/translation-tables/README.md` and restated in D6 —
rulebook > thematic table > `tugenden-fehler.md` — the rulebook's order should
win. Nothing hinges on it beyond the order of two words, which is why this is the
lowest-severity finding in the batch; it is recorded only because D6 obliges a
table row that contradicts the rulebook to be resolved rather than shrugged at.

**Severity: low.** **Discovery.**

### F-382 — `flaw.busybody` and `virtue.gossip` ship the identical German name, which the canonical table explicitly warns against

`rules/i18n/de/virtues_flaws.json` gives both:

- `flaw.busybody` → **"Klatschbase"**
- `virtue.gossip` → **"Klatschbase"**

And `tugenden-fehler.md:196` names the collision in its own `Anmerkung` column:

```
| Gossip | Klatschbase | Als Tugend (Informationsnetzwerk); nicht mit dem Fehler Busybody = Klatschbase (Fehler) verwechseln |
```

— "as a Virtue (information network); **not to be confused with** the Flaw
Busybody = Klatschbase (Flaw)". The table anticipated the confusion and the data
walked into it: in German the two catalogue entries are indistinguishable by name.

The mitigation is partial. The V/F picker partitions by `kind`, so a Virtue and a
Flaw never share a list there — but the Markdown export renders selected Virtues
and Flaws together, and every other surface that prints a name (validation issue
text, a saved character's rendered sheet) has nothing to disambiguate them. The
English pair is unambiguous, so this is a DE-only defect.

Note what this is **not**: a table error. `tugenden-fehler.md:196` is right, and
is the reason the collision is provable rather than a matter of taste. The fix
belongs in `rules/i18n/de/virtues_flaws.json` — a disambiguated form for one of
the two, in the style the data already uses elsewhere (`Wahre Liebe (SC)`,
`Ehrgeizig (Groß)`).

**Severity: medium** (localization, every session, on a surface with no other
disambiguator). **Discovery.**

### F-383 — nine `narrative` reclassifications in this batch sit inside a **twice-swept block behind a live guard**, and the guard's vocabulary cannot see any of them

This is the batch's most consequential finding, and it is about the guard rather
than about any one entry. It is placed last because it only becomes visible once
the nine entry-level findings above are on the table.

**The span is not unswept.** `crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS`
carries `("Ars Magica - Definitive Edition (Core Rules).md", 5639, 7113)` — "The
Flaws block … **Swept 2026-09-15; re-swept 2026-09-19** under the widened screen"
— so **33 of this batch's 35 entries lie inside it**, and
`::no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` is green on them
today. (The other two, `virtue.wise_one` and `virtue.withstand_casting`, are at
ArMDE:5257-5261, inside the explicitly unswept remainder `ArMDE:3951-5282`.)
`RULES.md:4688-4712` records what the sweep did: 49 Flaws moved `narrative` →
`uncomputed_rule`, seven of them in this span, each gaining the full passage as
`description` in both locales.

**So nine entries this batch reclassifies survived two human sweeps and a live
test.** None of the nine is in `NO_RULE_DESPITE_TOKEN`, the file's list of
"somebody read this and found no rule" exemptions — so no written reading
contradicts this batch; the entries were simply never surfaced.

**Why they were never surfaced.** `states_a_mechanical_rule` fires on exactly
three things: a **signed number** (`has_signed_number` — a sign character
immediately followed by a digit), a **botch term** (`BOTCH_TERMS`, 5 entries), or
one of the **20 `MECHANICAL_PHRASES`**, matched with a word boundary on the left.
I checked each of the nine passages against all three:

| entry | the rule it states | why the screen misses it |
|---|---|---|
| `flaw.abandoned_apprentice` :5647 | "give him experience points based on his age" — a creation procedure | no sign, no botch, no phrase |
| `flaw.ability_block` :5653 | a whole class of Abilities unlearnable | no sign, no botch, no phrase |
| `flaw.beloved_rival_*` :5697 | "the troupe **may allow** the character to **take both**" | no sign, no botch, no phrase |
| `flaw.bigamist` :5701 | "(6 x Wealth **Multiplier**)", "(3 x Wealth Multiplier)" | **near-miss**: `MECHANICAL_PHRASES` carries `multiply`, `multiplied`, `multiplizier`; the book writes *Multiplier* / *Multiplikator*, and the numbers are unsigned |
| `flaw.blackmail` :5709 | "a yearly value of about 50 silver pennies, **possibly more**" | **near-miss**: the list carries `or more`, matched at a left word boundary, so "possibly more" does not match; and `50` is unsigned |
| `flaw.blind` :5721 | "reading **is impossible**", "**cannot** aim spells without magical aid" | **near-miss**: the list's only absolutes are `cannot die` / `nicht sterben` |
| `flaw.bound_casting_tools` :5725 | Arcane-Connection durations, "a few weeks" vs "years" | durations in words; no token |
| `flaw.bound_magic` :5729 | "You **cannot take** this Flaw with Harnessed Magic" | **near-miss**: same absolutes gap |
| `flaw.branded_criminal` :5751 | "You **may not take** the Wealthy Virtue, but you **may take** Martial Abilities" | **near-miss**: no permission/prohibition idiom in the list |

Five of the nine are *near-misses* — the passage states its rule with an idiom
one word-form away from a phrase the list already carries. That is the actionable
part, because it is a bounded change rather than an open-ended one. Three
families would have caught five of the nine, and the module's own doc comment
already argues for exactly this trade ("Given the choice, be noisy", with the
noise bounded by `NO_RULE_DESPITE_TOKEN` rows rather than by hope):

1. **Prohibitions and permissions** — `may not take`, `cannot take`, `may take`,
   `is impossible`, and their German twins (`darfst du nicht`, `kann nicht
   genommen werden`, `darfst … wählen`, `unmöglich`). Catches `blind`,
   `bound_magic`, `branded_criminal`, `beloved_rival_*`, `ability_block`, and
   `virtue.wise_one` when the Virtues block is swept past ArMDE:3950.
2. **The multiplier stem** — `multiplier`, `multiplikator`. Catches `bigamist`.
   Note that the doc comment at `uncomputed_clauses.rs` already reasons carefully
   about this exact stem, rejecting the over-broad `multipl` because it swallows
   "multiple times"; `multiplier`/`multiplikator` is the narrow form it did not
   consider.
3. **`more` without the `or`** — the current `or more` is anchored on the left,
   which is right for avoiding "f|or more| details" but also excludes "possibly
   more". Catches `blackmail`.

Two facts about the guard's mechanics, checked rather than assumed, that a
correction slice needs:

- **The screen reads the English source passage only.** The assertion resolves
  `source.file` (always the English basename) and `source.lines` through
  `bracketed_passage`, and never opens the German file. So the German entries in
  `MECHANICAL_PHRASES` do nothing in *this* assertion — they earn their keep in
  the **first** assertion, which screens shipped `description`/`summary` text in
  every locale. Adding a German-only idiom therefore fixes nothing here.
- **`NO_RULE_DESPITE_TOKEN` cannot hide any of the nine**, because none of them is
  listed and because `exempted_entries_still_trip_the_screen` requires every row
  to keep tripping — an entry that does not trip cannot be silenced by a row.

**Severity: high** — not because any single number is wrong, but because this is
the mechanism by which the audit's founding defect recurs. The block was declared
swept and clean twice; nine rules stated in words are still classified as flavour
inside it; and the only thing standing between that and the next sweep is a
20-phrase list that this batch has now shown misses five idioms in one 140-line
span. **Discovery**, and it corroborates the README's thesis ("a token screen is a
work-list generator, not the analysis") with a worked example against a live
guard rather than a historical one.

## Open questions

**Q-79 — The book files two of this batch's Flaws under a magnitude their own
descriptors contradict.** `Bound to (Role) Role` and `Broken Vessel` appear in the
"List of Flaws" index under `### Supernatural, **Major**` (ArMDE:5392, :5393) and
appear in the `### Supernatural, Minor` list (ArMDE:5534-5562) not at all — while
their descriptor lines read `*Minor, Supernatural*` (ArMDE:5736, :5754). The DE
index repeats the same filing at the line-parallel DE:5392-5393, so it is
inherited from the original, not a translation slip. The data follows the
descriptor (`magnitude: "minor"` on both), which this batch judged the right call
because the descriptor is the entry's own statement — but the choice is a rules
reading, and a Major/Minor error is worth 2 points of budget either way.

**Q-80 — Does `flaw.black_sheep`'s "a bad Reputation **of your choice** at level
2" (ArMDE:5705) choose the Reputation's *type* or its *content*?** The engine has
an exact representation for the first: `Effect::GrantsReputation`'s
`kind: Option<ReputationType>`, where `None` is documented as the "player-chosen
type" wildcard and `virtue.famous` (ArMDE:3861-3863) ships it. The data instead
fixes `kind: "local"`, and `RULES.md:4583` records `{local, 2}` as a seed granter
without addressing the phrase. The competing reading is that "of your choice" picks
the `Reputation::content` free text and "among those who respect your family"
fixes the audience as Local — which is defensible and is what the data assumes.
`reputationen.md:105` calls the audience "Sozial (Familienkreis)", which is not one
of the four `ReputationType` values and so settles nothing.

**Q-81 — Which `casting_fatigue` sign convention is intended?** The shipped data
uses *positive = better for the character* (`virtue.withstand_casting` +1,
`flaw.vulnerable_casting` −1, `flaw.painful_magic` −1). `types.rs::HealthTrack::CastingFatigue`'s
doc comment states the reverse and names the same entries ("Vulnerable Casting
+1, Withstand Casting −1"). The Fluent label is `Casting fatigue` /
`Zauber-Erschöpfung` in the two locales, which disambiguates neither. Three
different fixes are available — change three amounts, change one comment, or
change two labels — and they are not equivalent: the track is surfaced-only, so
whichever is chosen, a **user** reads the sign next to that label. (F-351.)

**Q-82 — May the German source be used to repair a truncated English sentence?**
ArMDE:5725 reads "Regular casting tools remain as Arcane Connections for a few
weeks, **his last years.**" — a clause with no verb. The line-parallel DE:5725 is
complete and gives the sense ("die zuletzt verwendeten bleiben es **jahrelang**"),
so the intended English is evidently "…his last **for** years." CLAUDE.md makes
English the source of truth and forbids implementing a rule not present in the
English source. Does a demonstrably-truncated English sentence, whose meaning the
line-parallel German fixes unambiguously, count as present? (F-370.)

**Q-83 — Is a hedged monetary value a mechanical clause for classification
purposes?** `flaw.blackmail` states "a yearly value of **about** 50 silver
pennies, possibly more if you keep the pressure on" (ArMDE:5709). The app models
no wealth, and `types.rs::Classification`'s narrative test names "a signed
modifier, a botch-dice change or a cap", none of which this quite is. The same
question governs every Virtue and Flaw in the catalogue that quantifies an income
or a cost, so a ruling here is worth more than one entry. (F-365.)

**Q-84 — What entity or type should hold `flaw.abandoned_apprentice`?** The
passage says the character "knows Hermetic magic and can cast spells and enchant
items like other magi" but "**is not a member of the Order of Hermes**"
(ArMDE:5643), and adds that if he knows the Parma Magica "he must join the Order
or be slain" (:5647). The magus profile's `required_traits` is
`["virtue.hermetic_magus"]`, so the character the passage describes — Hermetic but
not of the Order — has no profile that fits: as a magus he is required to be a
member, as a companion he may not take Hermetic V/F unless Gifted. The entry
carries no prerequisite and no `entity_kinds` restriction today. (F-354.)

**Q-85 — WITHDRAWN, answered from the repository itself.** It asked whether the
DE `flaw.bound_to_realm` name is a deliberate dodge of German case government.
It is: `RULES.md:2356-2373` states the problem (*Das Göttliche* and *Das
Infernale* carry their own article, so "Gebunden an {realm}" renders "Gebunden an
Das Göttliche"), names the remedy (apposition after a comma, uninflected) and
applies it to four entries. See the withdrawn F-373.

**Q-86 — Should every "choose the specifics" Flaw carry a parameter?** Three
entries in this batch require the player to choose something and record it
nowhere: `flaw.ability_block` (which class of Abilities, ArMDE:5653),
`flaw.baneful_circumstances` (which circumstances, :5689) and `flaw.busybody`
(whether the Flaw is scoped to the covenant's lower-class members, :5763) — while
`flaw.anchored_to_the_land`, `flaw.bound_to_realm`, `flaw.bound_to_role_role` and
`flaw.careless_with_ability` all do carry one. engine-semantics § B8 records that
an unconsumed parameter is "the normal and intended shape" for recording a player
choice, so the machinery is not the obstacle; the question is whether the audit
should treat a missing one as a defect by default or only where the name template
already has a slot. (F-355, F-362.)

**Q-87 — Should `flaw.broken_vessel` carry an enumerated `prerequisites` tree, or
is D3's "inexpressible" the end of it?** *(Raised by the verification sub-agent.)*
ArMDE:5755 states "Characters may only take this Flaw if they have **at least one
Supernatural Ability or Art** normally improved through experience points."
`Prereq` has no "holds any Ability of category X" variant, so D3's letter is
satisfied and the batch's clean verdict stands — but an
`Any([ability_min(each supernatural Ability, 1), art_min(…)])` tree **would** bind
and the engine would enforce it. Three facts bear on the ruling, all checked:
**no** catalogue entry has ever used `ability_min` or `art_min` (all 19 entries
with `prerequisites` use `has`/`house`/`is_magus`/`any`/`none`/`all`);
`supernatural` is deliberately **not** in `categories_requiring_virtue`, so there
is no authorization hook to lean on instead; and an enumeration would silently
fail to cover a supernatural Ability added later as data, which collides with
CLAUDE.md's "catalogue size is data, never code" invariant in spirit if not in
letter. The live consequence is small but real: `supernatural` is in the **grog**
profile's `permitted_categories`, so a grog with no Supernatural Ability and no
Arts may take Broken Vessel today and nothing objects. This is the same shape D3
ruled on and probably wants the same answer, but the audit does not pick.

## Sub-agent reconciliation

One verification sub-agent was run, briefed to **re-derive every entry this batch
marked clean** from the source rather than from these notes, and told that
overturning one is a success. It re-derived all eleven
(`flaw.ambitious_major`, `flaw.ambitious_minor`, `flaw.animal_companion`,
`flaw.arthritis`, `flaw.avaricious_major`, `flaw.avaricious_minor`,
`flaw.broken_vessel`, `flaw.carefree`, `flaw.careless_sorcerer`,
`flaw.careless_with_ability`, `flaw.castratus`).

**Result: 11 corroborated, 0 overturned — but the pass was not empty.** It
corrected one claim of this file's, established four negatives this file had
assumed rather than checked, and raised one open question (Q-87). It changed no
file and ran no git command, and it confirmed it saw and ignored the "auto mode"
injection.

### What it corrected

**The ArMDE:2818 Story-Flaw cap is on all four profiles, not just companion.**
The § "ArMDE:2814, :2816, :2818, :2820" table first read "on the companion
profile", which was true and too narrow. `rules/core/character_types.json` carries
`{ "category": "story", "max": 1 }` at lines 13 (companion), 88 (magus) and 131
(mythic companion), and `{ "category": "story", "max": 0 }` at line 56 (**grog**)
— the last additionally encoding ArMDE:2826's per-type list. I re-ran the check
and the correction is right; the table above is amended and marked.

That matters beyond tidiness: it means `flaw.animal_companion`'s
`categories: ["story"]` is load-bearing on every character type, which is part of
why its clean verdict holds.

### Negatives it established that this file had assumed

Each of these is a claim the first pass made or relied on **without opening the
source**, and which the sub-agent checked:

1. **The engine models no botch dice at all.** `types.rs::SpecialCasting` has
   eleven kinds and none is botch-related, and the doc on its
   `DoubledAuraPenalty` variant says so in as many words — "the engine models
   neither an aura of a foreign realm nor botch dice". This is what makes
   `flaw.careless_sorcerer`'s and `flaw.careless_with_ability`'s
   `uncomputed_rule` correct rather than merely plausible.
2. **There is no sex or gender axis a `Prereq` could read.** `types.rs` declares
   `pub gender: String` with the doc "(free-text; **no mechanical effect**)", and
   no `Prereq` variant reads it. That is the evidence for `flaw.castratus`'s D3
   verdict, which this file asserted from the shape of the restriction alone.
3. **No catalogue entry has ever used `ability_min` or `art_min`.** The
   sub-agent dumped all 19 entries carrying `prerequisites` and found every one is
   `has` / `house` / `is_magus` / `any` / `none` / `all`. This is what turns
   `flaw.broken_vessel`'s precondition from "inexpressible" into "expressible only
   by a construction with no precedent in the data" — see Q-87.
4. **`supernatural` is not in `categories_requiring_virtue`**
   (`rules/core/abilities.json` lists `academic`, `arcane`, `martial` only), so
   there is no authorization hook that could stand in for Broken Vessel's
   precondition either.

### Where its method went further than the first pass's

- On the **ASCII-hyphen check** it did not stop at `grep`: it scanned all eleven
  entries' `name`/`summary`/`description` in both locales for U+2212, U+2013,
  U+2014, U+2010 and U+2011 (0 hits) and then **byte-checked** the DE Arthritis
  description with `od -c`, confirming `0x2D`. This file's catalogue-wide
  `grep -c "−"` covered only U+2212.
- On `flaw.careless_with_ability` it read
  `validation/selections.rs::validate_duplicate_selections` and confirmed the
  duplicate key is `(&selection.item_ref, &selection.params)` — the **whole**
  params map — rather than inferring it from engine-semantics § B10, and then
  found the contrast case that proves the reading:
  `flaw.necessary_realm_aura_for_ability` carries `max_per_value: 1` on its
  ability key **because it has two parameters**, so `max_per_target` alone would
  not catch two copies sharing an Ability but differing in Realm. A
  single-parameter entry needs no such field. That is a cleaner proof of F-358's
  control case than the one written above.
- It independently re-verified **Q-79** by counting every occurrence of "Broken
  Vessel" in the English file — exactly three: the index at ArMDE:5393, the
  descriptor at :5753, and the page index at :24348 — establishing that the entry
  appears in the Supernatural **Major** index list and in the Minor list not at
  all. It reached the same verdict (follow the descriptor, escalate the clash).

### One observation it declined to make a finding, and was right to

On `flaw.avaricious_*` it noted that the shipped `summary` in both locales is
"You want money, lots of money." / "Du willst Geld, viel Geld." — unusually thin
even by the first-sentence convention — and explicitly declined to raise it,
because nothing mechanical is lost and D5 is therefore not engaged. Recorded here
so a later reader does not mistake the thinness for a missed D5 case.

### Net effect on this batch's counts

None. 11 clean stands, and no finding above is withdrawn or added by the
verification pass. Q-87 is added.
