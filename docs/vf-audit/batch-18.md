# Batch B18 — `flaw.suppressed_gift` … `flaw.visions`

Indices 595–629 of the canonical ordering, **ArMDE:6803-6988**, **35 entries, all
Flaws**. Findings **F-523 … F-538** (16). Open questions **Q-136 … Q-139** (4).
Shipped classifications: 18 `narrative`, **12 `uncomputed_rule`** (the highest of
any batch), 4 `in_play_effect`, 1 `creation_effect`.

**STATUS: primary read complete; independent verification pass RUN 2026-09-21.**
The primary agent was killed by a session rate limit mid-sentence and its own
verification sub-agent died with it having read nothing, so the pass recorded in
`## Sub-agent reconciliation` re-derived **every** census figure and **every**
verdict from primary sources rather than reviewing the primary read's reasoning.
Outcome: **18 clean / 16 failing / 1 escalated**; **eight census figures
corrected**; **no finding withdrawn**; **two findings materially scope-corrected**
(F-532, F-533 — a grog cannot take a Story Flaw at all), **four narrowed or
widened** (F-524, F-526, F-538, Q-136); **one new note added** (two further
faithful-copy German source errors). Rows and findings are written as each is
finished, never batched.

**The three shapes, stated as claims.**

1. **A binding decision has not reached the data.** `decisions.md` **D3** rules
   `flaw.susceptibility_to_faerie_power` and `flaw.susceptibility_to_infernal_power`
   `uncomputed_rule` **by name**, with the halving written into `description` in
   both locales. Both ship `in_play_effect`, with an effect
   (`magic_resistance_mod{susceptible_faerie|susceptible_infernal}`) that
   `engine-semantics.md` A37 records as **surfaced-only, pushed with `amount: 0`**,
   and with **no `description` in either locale**. This is not a judgement call the
   batch had to make; it is a ruling the data contradicts (F-523).
2. **One entry produces a number the rulebook forbids.** `flaw.unspecialized`
   (ArMDE:6945) says *"The character does not have any specialties for any of her
   Abilities."* The entity model stores a free-text `specialty` on every
   `AbilityScore`; nothing consults the Flaw; and
   `derived/combat.rs::specialization_bonus` ends `u8::from(has_specialty)`, so a
   character holding this Flaw and a weapon-Ability specialty is printed a **+1 on
   his combat totals that the Flaw exists to forbid** (F-524). This is
   `CLAUDE.md`'s top severity class — wrong rules output, not a lost rule.
3. **`uncomputed_rule` is not a free pass, and this span has the most of them.**
   Twelve entries carry it. Eleven are correctly so — the engine has no
   characteristic cap, no Twilight model, no culture model, no botch-dice model. But
   **`flaw.university_dean` states three eligibility clauses and encodes none**,
   two of which the engine can express today: `Prereq::Has("virtue.doctor_in_faculty")`
   (the Virtue exists) and an `incompatible_with` against `flaw.poor` **plus the
   sixteen catalogue Flaws that carry `grants_reputation`** (corrected by the
   verification pass from *seventeen*, which is the **effect-row** count, not the
   entry count — `flaw.failed_monk` carries two rows) — the book's phrase is
   *"or any other Flaw that grants a Bad Reputation"*, which is a set the data itself
   enumerates (F-526). That is B16's F-494 shape arriving from `uncomputed_rule`.

---

## Method

### What was read, and in what order

1. `docs/vf-audit/README.md`; `decisions.md` (**D1–D7 in full**);
   `corrections.md` § 0–§ 5 in full plus § 6 by index; `engine-semantics.md`
   Part A entries A25, A37, A39, A40 and Part B B1, B5, B6, B7, B8, B10, plus
   Part C headings; `batch-17.md` (header, Method in full, the cross-reference
   and inbound-sweep tables, the `NO_RULE_DESPITE_TOKEN` note).
2. **The English source as continuous prose, ArMDE:6780-6999** — overrunning the
   span by 23 lines above (back through `#### Casting Tools for Spontaneous
   Magic`, B17's territory) and 11 below (forward through `#### Vulnerable
   Casting`, B19's).
3. **The German source over the matching range, DE 6795-6994.**
4. The shipped mechanics for all 35 entries (`jq` over
   `rules/core/virtues_flaws.json`) and the shipped text for all 35 in **both**
   locales (`rules/i18n/{en,de}/virtues_flaws.json`).
5. The consumer functions themselves, not the yardstick's summary of them,
   wherever a verdict rests on engine behaviour:
   `crates/arm-rules/src/derived/combat.rs::specialization_bonus` (read in full),
   `crates/arm-rules/src/validation/prereq.rs::PrereqCtx::build` (`:90-172`),
   `crates/arm-rules/tests/uncomputed_clauses.rs`'s `MECHANICAL_PHRASES` and
   `has_mechanical_phrase`, `rules/core/character_types.json`.
6. `crates/arm-rules/RULES.md` — searched for every entry name before any
   modelling choice was called wrong.

### Line parity — confirmed, and here is the check

The German file is line-parallel with the English across the whole span and past
it at both ends. Every `####` heading in 6803-6988 sits on the same line number
in both files, and each descriptor line follows on the same line:

| ArMDE | EN heading | DE heading |
|---|---|---|
| 6799 | `#### Supernatural Nuisance` | `#### Übernatürliche Plage` |
| **6803** | `#### Suppressed Gift` | `#### Unterdrückte Gabe` |
| 6855 | `#### Tragic Life` | `#### Tragisches Leben` |
| 6891 | `#### Unbearable to (Beings)` | `#### Unerträglich für (Wesen)` |
| 6951 | `#### Usurer` | `#### Wucherer` |
| **6985** | `#### Visions` | `#### Visionen` |
| 6989 | `#### Vow` | `#### Gelübde` |

There are **34** `####` headings between 6803 and 6988 and **35** ids, because
`flaw.true_love_major` and `flaw.true_love_minor` correctly share the single
`#### True Love` heading's range (6871-6878). Parity holds.

### Per-check roll-ups

**Check 1 — `source.lines` and `source.anchor`.** All 35 ranges run from the
heading to the line before the next heading, including the four multi-paragraph
entries (`suppressed_gift` 6803-6810, `tragic_life` 6855-6870, `true_love`
6871-6878, `vengeful_powers` 6959-6976, `viaticarus` 6977-6984). Pass, 35/35.

`source.anchor` is present on **11** entries (corrected by the verification pass
from **10**, which contradicted the very list it introduced): `surgical_empiricus`,
`susceptibility_to_sunlight`, `true_love_major`, `true_love_minor`,
`unbearable_to_beings`, `uncertain_faith`, `uninspirational`, `university_dean`,
`unlucky`, `vengeful_powers`, `viaticarus` — eleven ids, carrying the ten distinct
values `surgical-empiricus`, `susceptibility-to-sunlight`, `true-love` (×2),
`unbearable-to-beings`, `uncertain-faith`, `uninspirational`, `university-dean`,
`unlucky`, `vengeful-powers`, `viaticarus`. Every one matches its heading's
GitHub-style slug and the alphabetical index's own link (ArMDE:25533-25640) —
verified value by value, not sampled. Three `uncomputed_rule` entries carry
**no** anchor (`twilight_prone`, `uncontrollable_strength`,
`unpredictable_magic`) and two `narrative` entries carry one (the True Love
pair). B17 established that the field is carried by 84 of 102 `uncomputed_rule`
entries catalogue-wide and by 10 others, so the span's pattern matches the
catalogue's; **no anchor finding is raised**, on B17's precedent.

**One source-side oddity, recorded and deliberately not filed against the data.**
The "List of Flaws" index link at ArMDE:5373 reads
`[True Love (NPC)](#true-love-npc)`, but the heading at :6871 is `#### True Love`
and the alphabetical index at :25606 reads `True Love (Flaw) | [147](#true-love)`.
The shipped `anchor: "true-love"` follows the heading and the alphabetical index —
two sources against one — and is **correct**. The `#true-love-npc` link is a
defect in the rulebook markdown, not in the catalogue.

**Check 2 — `classification`.** Re-derived from the data, not inherited: 18
`narrative`, 12 `uncomputed_rule`, 4 `in_play_effect`, 1 `creation_effect`.
**Seven `narrative` entries are wrong** (F-524, F-525, F-528, F-529, F-530,
F-531, F-535), **two `in_play_effect` entries are wrong** (F-523, both named by
D3), and **one `in_play_effect` entry is escalated rather than judged** (Q-136,
`flaw.susceptibility_to_divine_power`). All nine reclassifications go to
`uncomputed_rule`; none to a computed class — in every case the engine
structurally cannot express the rule, which is **D3** exactly.

18 − 7 = **eleven** `narrative` entries are correct: `tainted_offspring`,
`temperate`, `true_love_major`, `true_love_minor`, `tzadik_nistar`, `unbaptized`,
`unhappily_married`, `unruly_air`, `visions`, `tormenting_master` and
`vendetta` — the last two *fail on their prereqs* (F-532, F-533) while their
classification stands, because an eligibility gate the data omits is a
`prerequisites` defect, not a claim that the book states a computable mechanic.

**The twelve `uncomputed_rule` entries were each asked whether the engine can in
fact compute the rule** (the F-494 transition). **Eleven cannot** and are
correctly classified; **one can, in part** — `flaw.university_dean`, F-526.

**Check 3 — `kind`.** All 35 are `flaw`; all 35 descriptor lines are Flaw
descriptors; all 35 appear under `## List of Flaws` (ArMDE:5283). Pass.

**Check 4 — `magnitude`.** All 35 match their descriptor line. **The F-487
dual-magnitude check, run over the span:** exactly one descriptor in 6803-6988
reads `*Major or Minor*` — `#### True Love` at ArMDE:6872 (`*Groß oder Klein*`,
DE 6872) — and both `flaw.true_love_major` and `flaw.true_love_minor` exist, with
a symmetric `incompatible_with` pair. The book's own index confirms the split
independently: `[True Love (NPC)]` appears at ArMDE:5373 under
`### Story, Major` (:5340) **and** the alphabetical index carries one
`True Love (Flaw)` row (:25606). Pass; **no new instance of F-487.**

**Check 5 — `entity_kinds`.** All 35 are `["character"]`. One inbound passage
bears on it and confirms rather than contradicts: ArMDE:2662 makes
`flaw.tragic_life` a compulsory Flaw of the **Devil Child** Mythic Companion, an
`EntityKind::Character` sub-type. Pass.

**Check 6 — `categories` + `tainted`.** Verified against the descriptor line
**and** the book's index list, two independent sources:

| Index list (ArMDE) | Span entries found there |
|---|---|
| `### Hermetic, Major` (:5285) | Suppressed Gift :5301, Twilight Prone :5302, Unnatural Magic :5303, Unstructured Caster :5304 |
| `### Story, Major` (:5340) | Suppressed Gift :5369, Tainted Offspring :5370, Tormenting Master :5371, Tragic Life :5372, True Love (NPC) :5373, Tzadik Nistar :5374, Unbaptized :5375, Unhappily Married :5376, University Dean :5377, Vendetta :5378, Vengeful Powers :5379 |
| `### Hermetic, Minor` (:5417) | Susceptibility to Divine/Faerie/Infernal Power :5452-5454, Unbearable to (Beings) :5455, Unimaginative Learner :5456, Unpredictable Magic :5457 |
| `### Story, Minor` (:5498) | Visions :5517 |
| `### Social Status, Minor` (:5520) | Surgical Empiricus :5531, Usurer :5532 |
| `### Supernatural, Minor` (:5534) | Susceptibility to Sunlight :5557, Susceptibility to Warping :5558, Unruly Air :5559, Viaticarus :5560, Visions :5561 |
| `### General, Minor` (:5564) | Tainted with Evil :5628, Unbearable to (Beings) :5629, Uncertain Faith :5630, Uncontrollable Strength :5631, Uninspirational :5632, Unlucky :5633, Unspecialized :5634 |

`tainted: true` appears **three** times, on `tainted_offspring` (:6840
`*Major, Story, Tainted*` / DE `*Groß, Geschichte, Befleckt*`), `tragic_life`
(:6856) and `vengeful_powers` (:6960) — the `Tainted → Befleckt` mapping
`CLAUDE.md` names, correct in all three.

**Two multi-list entries and what the data does with them, both checked rather
than assumed.** `flaw.unbearable_to_beings` is indexed under *both* Hermetic
Minor (:5455) and General Minor (:5629), matching its descriptor
`*Minor, Hermetic or General*`; the data ships `categories: ["general"]` with
`index_categories: ["hermetic"]`. That is the **consistent convention of the
whole four-entry "beings" family** — `flaw.offensive_to_beings`,
`virtue.inoffensive_to_beings` and `flaw.unbearable_to_beings` all carry exactly
this pair, and they are three of the only four `index_categories` users in the
catalogue. Per `engine-semantics.md` B5 the field is provenance with a single
consumer (`validate_house`'s ArMDE:2860 "at least one Hermetic Flaw" guideline),
which is the right consumer here. **Cleared, not re-opened.**
`flaw.visions` is indexed under both Story Minor (:5517) and Supernatural Minor
(:5561), matching `*Minor, Story, Supernatural*`, and ships
`categories: ["story","supernatural"]` — both real, which B5 says is what a
two-category descriptor means. Pass.

**A rulebook self-contradiction, recorded so the next reader does not "fix" the
data toward it.** ArMDE:2662 lists the Devil Child's compulsory Flaw as
**"Tragic Life (Major, Supernatural)"**, while the Flaw's own descriptor at
:6856 reads `*Major, Story, Tainted*` and the index files it under Story Major
(:5372). Two sources against one; the shipped `categories: ["story"]` is
**correct** and :2662 is the book's slip. No finding.

**Check 7 — `prereqs`.** **Two** entries carry one and both are right.
`flaw.suppressed_gift` carries `{"kind":"has","value":"virtue.the_gift"}`, which
ArMDE:6805's opening clause states outright. `flaw.unbearable_to_beings` carries
`Any[Has(virtue.the_gift), Has(flaw.magical_air)]`, which is ArMDE:6895's
*"Only characters with The Gift or Magical Air may take this Flaw"* exactly.
**Both are satisfiable in practice, and that was traced rather than assumed:**
`PrereqCtx::build` (`validation/prereq.rs:153-163`) resolves `Prereq::Has`
against bought **and** granted rows, and `rules/core/character_types.json` gives
**no** profile a `granted_selections` list, so a magus buys `virtue.the_gift`
(free) and `virtue.hermetic_magus` himself. Neither entry describes a character
the app refuses to build.

**Three passages state an eligibility gate and none is encoded** — F-526
(`university_dean`, ArMDE:6925: a required Virtue, a minimum age, a forbidden
Flaw class), F-532 (`tormenting_master`, :6853: *"only applicable to magi"*) and
F-533 (`vendetta`, :6957: *"generally restricted to magi of House Verditius"*).
The remaining 30 absences are correct.

**Check 8 — `incompatible_with`, and what its absence means.** Present on
**three** entries. `flaw.true_love_major` ↔ `flaw.true_love_minor` is the
universal dual-magnitude convention. `flaw.unbearable_to_beings` →
`flaw.blatant_gift` is ArMDE:6895's *"it cannot be combined with the Blatant
Gift"*, symmetric (a one-sided declaration fails the load in
`ruleset/integrity.rs::validate_incompatibility_symmetry`, and the ruleset
loads). **Two absences are meaningful and both are new:**

- `flaw.uncertain_faith` ↔ `virtue.true_faith` — ArMDE:6905 states it in the
  entry's own shipped `description` in **both** locales and the partner Virtue
  exists. **F-527.**
- `flaw.university_dean` ↔ `flaw.poor` and the bad-Reputation Flaw class —
  ArMDE:6925. **F-526.**

The other 30 absences were each interrogated and are correct.

**Check 9 — `parameters`.** **One** entry carries any:
`flaw.unbearable_to_beings`, `{"key":"being","type":"ref","domain":"enumerated",
"values":["being.demons","being.divine","being.mundane_humans"]}` with
**`max_total: 1`**. Both halves are exactly the passage: ArMDE:6893 names three
classes and no more, and ArMDE:6897 says *"You may not take this Flaw more than
once"*. **This is the one entry in the span that could have been an F-522/F-495
instance and is not** — it is the counter-example, a parameterized entry that
carries the `max_total` its passage demands.

Two entries make a choice the save cannot record, and both were checked rather
than assumed:

- `flaw.surgical_empiricus` — *"Pick a type of surgery as your specialization in
  that Ability"*. **No parameter is owed.** The choice is an Ability *specialty*,
  and `types.rs::AbilityScore::specialty` already stores it as free text
  (`ability.rs:68` records it as "free text and not part of the id"). The save
  round-trips it on the Chirurgy row. Cleared.
- `flaw.vengeful_powers` — *"one or more non-Infernal Powers"*. The Powers are
  the character's own Supernatural Virtues; no new parameter is owed either.

**Check 10 — `effects` vs the passage, in both directions.** **Five** entries
carry effects (corrected by the verification pass from **six**; check 11's own
table has always listed exactly five rows). The outbound direction (an effect the passage does not support) is
clean 35/35. The inbound direction fails on the two D3 entries (F-523) and on
`flaw.usurer` (F-534).

**Check 11 — engine reality, signs included.** Every effect-bearing entry, with
the consumer read rather than the variant name:

| Entry | Effect | Consumer read | Verdict |
|---|---|---|---|
| `flaw.susceptibility_to_divine_power` | `special_casting_mod{doubled_aura_penalty}` | A40 — `derived.rs::in_play_mods` | **Computes nothing.** `doubled_aura_penalty` is one of A40's eight surfaced-only kinds, pushed with `amount: 0`, and the variant carries **no amount field at all** — the kind slug is the whole payload. The classification question is **Q-136**. |
| `flaw.susceptibility_to_faerie_power` | `magic_resistance_mod{susceptible_faerie}` | A37 — `derived.rs::surfaced_modifiers` | **Computes nothing**, `amount: 0`. D3 governs. **F-523.** |
| `flaw.susceptibility_to_infernal_power` | `magic_resistance_mod{susceptible_infernal}` | A37 — same | **Computes nothing**, `amount: 0`. **F-523.** |
| `flaw.unimaginative_learner` | `advancement_mod{source: "vis", amount: -3}` | A39 — `derived.rs::in_play_mods` → `surfaced_modifiers` | **Correct, sign included.** ArMDE:6917 says *"Subtract 3 from rolls when you study from raw vis"*; a Source Quality is a total one adds to, so **−3** lowers it. Both fields reach the read-out intact, so the number **does** reach the player. Part C's advancement gap is the engine's and is not re-reported. **Clean.** |
| `flaw.usurer` | `grants_reputation{kind: "local", score: 4}` | A25 — `effective/reputation_and_caps.rs::reputation_grants`, `validation/scores.rs::validate_reputations` | **Score and audience correct**; ArMDE:6953 says *"a poor reputation (Usurer) at level 4 within your community and the local region"*. The **polarity is lost** — A25 has no bad/good field, the known **Q-43** gap. The other clause (ten pounds of silver a year) reaches neither locale: **F-534.** |

**Check 12 — text.** 35 EN names, 35 DE names, 35 summaries per locale, 12
descriptions per locale, each against the passage in the **matching** language.

- **All 35 EN names** match their EN heading. `Unbearable to {being}` uses the
  parameter placeholder, which is the family convention.
- **All 35 DE names** match their DE heading at the matching line — verified
  heading by heading, not sampled. **D7's default (the DE rulebook heading wins)
  is satisfied everywhere, so no name is in dispute and no glossary row is
  consulted against a heading.**
- **Negative signs.** Every signed value in the shipped text — `-2`
  (`susceptibility_to_sunlight`), `-3` (`unbearable_to_beings` ×2,
  `uncertain_faith`, `uninspirational`), `-1` (`susceptibility_to_infernal_power`),
  `-1 to -3` (`unlucky`), `+1` (`surgical_empiricus`, `uncertain_faith`) — is
  **ASCII hyphen-minus U+002D in both locales**, while the German source writes
  U+2013. Correct per `CLAUDE.md`. 
- **Three faithful-copy cases were checked and left alone**, on the `scheitest`
  precedent (`corrections.md` § 5). DE `flaw.vengeful_powers` reproduces DE 6971's
  `vergekten` (for *vergelten*); DE `flaw.temperate` reproduces DE 6849's
  `fröhnst` (for *frönst*); EN `flaw.unpredictable_magic`'s source descriptor at
  ArMDE:6936 reads `*Minor. Hermetic*` with a period, which the data correctly
  ignores (it is a descriptor, not shipped text).
- **One case runs the other way and is also left alone.** ArMDE:6929 writes
  `-1 to -3penalty` with a missing space; the shipped EN description writes
  `-1 to -3 penalty`. This is a typographic repair, not a rules change, and it
  makes the string readable. Recorded so nobody "restores" the typo.
- **Two summaries paraphrase rather than quote, and both are accurate.**
  `flaw.tragic_life` EN *"Demons have manipulated your life to your own
  detriment; you are Tainted."* / DE *"Dämonen haben dein Leben zu deinem
  Nachteil manipuliert; du bist befleckt."* condenses ArMDE:6857's long first
  sentence. `flaw.unhappily_married`'s summary is the source's literal first
  sentence (*"Young merchants often marry for financial reasons."*), which says
  nothing about the character — faithful but weak. Neither is filed.

### Part C systemic gaps are not re-reported per entry

Four entries carry an effect whose variant is a known Part C gap and **none is
reported as a per-entry defect on that ground**:
`flaw.unimaginative_learner`'s `advancement_mod` (surfaced-only, both fields
intact); `flaw.susceptibility_to_divine_power`'s `special_casting_mod` kind (one
of A40's eight); the two realm susceptibilities' `magic_resistance_mod` kinds
(two of A37's four surfaced-only kinds). **F-523 is about D3's ruling on the
*classification*, which D3 itself says is the honest answer precisely *because*
the engine cannot compute it — it is not a re-raising of the engine gap.**

### What `crates/arm-rules/RULES.md` says about this span — searched before any modelling choice was called wrong

`RULES.md` is fallible (`corrections.md` records `:5161` as false at the commit
that wrote it), so it is evidence, not authority. But it carries **eleven** rows
touching this span, and reading them **confirmed four of my verdicts, supplied the
narrowing in F-523, and supplied the core of Q-136**.

| `RULES.md` | What it records | Effect on this batch |
|---|---|---|
| `:4740-4752` | *"`flaw.uninspirational` (ArMDE:6921) is likewise a **cap on two Characteristics** … All four are `uncomputed_rule` rather than `creation_effect`/`in_play_effect` because the engine has no representation for any of them: Personality Traits are free-form user entries … with no granting effect and no cap validation"* | **Confirms my clean verdict.** The entry is deliberately `uncomputed_rule`, with the reason written down. It also names the four best candidates if a later slice wants to compute something, and points at `docs/open-todos.md`. |
| `:4870-4888` | The full rationale for `flaw.susceptibility_to_divine_power`'s move off `magic_resistance_mod` onto `special_casting_mod{doubled_aura_penalty}`, ending *"surfaced, not simulated"* | **Supplies Q-136.** The effect variant is right; the sentence describes an `uncomputed_rule` under an `in_play_effect` label. |
| `:4919`, `:5138` | Both susceptibility siblings listed under "Magic-resistance modifier", surfaced with `amount: 0`, *"listing them keeps them from being silently dropped and from being applied where the book does not apply them"* | **Confirms F-523's recommendation to keep the effect** and move only the class. |
| `:4728-4731` | The no-byte-identical-`description` policy, naming `flaw.susceptibility_to_divine_power` as its precedent | **Narrowed F-523**, which originally proposed adding a `description` to both entries. See the finding. |
| `:4722-4725` | `surgical_empiricus`, `susceptibility_to_sunlight`, `unbearable_to_beings`, `uncertain_faith`, `uninspirational`, `unlucky`, `vengeful_powers` listed among the 2026-09-15 reclassifications that gained a `description` | Confirms the 12 descriptions are deliberate, not incidental. |
| `:4792` | `flaw.university_dean` and `flaw.viaticarus` listed among the 2026-09-19 phrase-screen re-sweep's 19 reclassifications | Their `uncomputed_rule` label is deliberate; **F-526 does not dispute it**, it disputes the *absent prereq and incompatibility*. |
| `:159-166` | *"`flaw.tainted_with_evil` is named 'Tainted With Evil' but reads `*Minor, General*` (`ArMDE:6844`) and is therefore **not** flagged, while … `flaw.tragic_life` (`*Major, Story, Tainted*`, `ArMDE:6856`) [is]"* | **Confirms check 6** for both entries, and records a `data_integrity.rs` test pinning it. |
| `:368`, `:392`, `:921`, `:990`, `:1600` | `flaw.suppressed_gift`'s dual index filing; `flaw.unbearable_to_beings`'s `["general"]` + dual index, its prereq tree, its `max_total` from ArMDE:6897 | **Confirms checks 6, 7, 8 and 9** for both entries. |
| `:3840` | The Devil Child package table: `virtue.devil_child` → required Flaw **Tragic Life**, `+7 F, +3 free V`, `ArMDE:2643-2666` | The inbound Devil Child rule **is** recorded, on the Virtue's row. **My cross-reference note is a confirmation, not a new gap.** |
| `:5386`, `:5390` | *"`flaw.usurer` … reputation grant **poor** score 4"*; *"`virtue.famous` … reputation grant **player-chosen** score 4"* | **Confirms F-534** (the polarity exists in the provenance record and cannot be expressed in the data — Q-43) **and F-525** (Famous is a wildcard grant). |

**No `RULES.md` self-contradiction was found in this span.** Every row above
agrees with the shipped data and with the rulebook. Where `RULES.md` and
`decisions.md` D3 pull apart — over whether a `description` must duplicate a
`summary` — that is recorded inside F-523 rather than filed as a contradiction,
because the two are answering different questions.

### The live catalogue-wide patterns, each checked against this span

| Pattern | Result in ArMDE:6803-6988 |
|---|---|
| **F-409 — the authorization family** | **No instance.** Two passages name an Ability and neither is a permission: ArMDE:6813 names *Chirurgy* (`ability.chirurgy` is `category: "general"`, so it is not in `abilities.json::categories_requiring_virtue` = `["academic","arcane","martial"]` and needs no authorization), and ArMDE:6921 names *Leadership, Charm, Intrigue, Etiquette* inside a **penalty**, which presupposes rather than grants. No `ability_authorization` is owed anywhere in the span, and adding one would be a bug. |
| **F-355 / F-511 — nothing FORBIDS an Ability** | **No instance of either sub-shape.** No passage in the span blocks an Ability category (F-355) or a named Ability (F-511). The nearest, `flaw.uninspirational`, caps two **Characteristics**, which is a different and equally absent mechanism — see the `narrative`/`uncomputed_rule` note under F-526. |
| **F-428 / F-439 / F-510 — a budget that should be halved or replaced** | **No instance.** No entry in the span carries `restricted_ability_xp`, `general_xp`, `later_life_xp_rate` or any other budget variant; the only creation-time effect in the span is `flaw.usurer`'s `grants_reputation`. `flaw.usurer`'s *"ten pounds of silver each year"* is an **income**, which the engine models nowhere at all, so it is a D5 text defect (F-534) and not a budget-stacking one. |
| **F-522 / F-495 — a parameterized entry with no `max_total`** | **No instance, and one counter-example.** The span's single parameterized entry, `flaw.unbearable_to_beings`, carries `max_total: 1` exactly as ArMDE:6897 requires. See check 9. |
| **F-450 / F-486 — a hardcoded or invented `grants_reputation` value** | **No instance.** The span's one `grants_reputation` (`flaw.usurer`, `local`/4) is the value ArMDE:6953 states, in the audience it states. The **polarity** loss is Q-43's, catalogue-wide, and is not re-derived. |
| **F-449 / F-462 / F-510 — an effect folded over more cases than the rule covers** | **No instance in the outbound direction.** But the *mirror* of F-449 is this batch's headline: `derived/combat.rs::specialization_bonus`'s `u8::from(has_specialty)` fires for a character whose Flaw says he has no specialties — the engine folds a bonus over a case the rule **excludes**. **F-524.** |
| **F-459 — an absolute broken by a legal PAIR** | **One instance: F-525.** `flaw.tainted_with_evil` (:6845) says *"Gaining a positive Reputation is impossible"*; `virtue.famous` grants a **wildcard** Reputation at score 4 and `virtue.hermetic_prestige` a Hermetic one at 4; neither entry names the other; the pair is legal and `validate_reputations` authorizes the row. Every other absolute in the span was asked the same question and **none is defeated**, because none is computed: `flaw.unnatural_magic`'s *"none … have a permanent effect"*, `flaw.unstructured_caster`'s *"may not learn Ritual spells at all"*, `flaw.uninspirational`'s *"may not be greater than 0"*, `flaw.unbearable_to_beings`'s *"not more than once"* (which **is** computed, by `max_total`, and holds), `flaw.suppressed_gift`'s *"cannot … improve his Arts"*, `flaw.unspecialized`'s *"does not have any specialties"* (F-524 — computed in the **wrong direction**). |
| **F-463 / F-465 — a stated prohibition carried by no `incompatible_with`** | **Two instances, both new: F-527** (`uncertain_faith` ↔ `virtue.true_faith`, ArMDE:6905) and **F-526**'s second half (`university_dean` ↔ `flaw.poor` + the bad-Reputation class, :6925). B15's F-466 applies to both: `validate_incompatibilities` reads **bought** selections on both sides, and neither partner is reachable as a *grant* in this catalogue (no profile carries `granted_selections`; no House grants `virtue.true_faith` or `flaw.poor`), so a plain symmetric `incompatible_with` suffices and `Prereq::Nor` is **not** needed here. |
| **F-487 — a dual-magnitude descriptor shipped in one magnitude** | **No instance.** The span's single `*Major or Minor*` descriptor is True Love and both ids exist, symmetric. See check 4. |
| **F-427 — ArMDE:2816's one-Social-Status rule** | **Two sites** (`flaw.surgical_empiricus`, `flaw.usurer`), both `categories: ["social_status"]`. Noted; the catalogue-wide claim is F-427's and is **not** re-derived. |
| **Truncated shipped text** | **Not re-scanned** — CLOSED, bounded at four entries and five strings. |

### Decisions applied

| Decision | Where it bit in this batch |
|---|---|
| **D1** | Not reached — no entry in the span carries `lab_total_mod`. |
| **D2** | Not reached — no entry in the span is grantable by a House or type profile (confirmed against `character_types.json`, where no profile carries `granted_selections`), and none carries a precondition of the `great_characteristic` shape. |
| **D3** | **The load-bearing ruling, twice over.** It *names two of this span's entries* and the data contradicts it (F-523). And it is the reason all nine other reclassifications go to `uncomputed_rule` rather than to a computed class, and the reason eleven of the twelve shipped `uncomputed_rule` entries are **correct** despite stating real numbers: no characteristic cap (`uninspirational`), no Twilight model (`twilight_prone`), no botch-dice model (`unpredictable_magic`, `uncontrollable_strength`), no Decrepitude-scaled social penalty (`viaticarus`), no culture model (`surgical_empiricus`), no `Prereq` variant comparing a Characteristic (`uncontrollable_strength`'s *"may not be taken if … Strength is below 0"*). |
| **D4** | Not reached (no `lab_total_mod`). |
| **D5** | Applied to every finding. **Two findings are D5-only** — F-534 (`usurer`'s income clause) and the second half of F-523. All **12** shipped `uncomputed_rule` entries do carry a `description` in **both** locales, so the brief's hypothesised "an `uncomputed_rule` whose description does not state the rule in both locales" **has no instance here** — see the guard subsection below, where each was run against `states_a_mechanical_rule` by hand. |
| **D6** | **Not reached.** No shipped German name in the span disagrees with its DE rulebook heading, so no glossary row is in dispute and no stale-copy determination is needed. |
| **D7** | Applied as the default test for all 35 DE names; all 35 follow the heading and all 35 pass. No escalation. |

### Cross-references followed and rated

Every outbound pointer in the span's 34 passages, plus every inbound reference
found by sweeping all 34 entry names across the whole English rulebook.

| Entry | Pointer | Where it lands | Adds a rule? | Already in `corrections.md` / `batch-17.md`? |
|---|---|---|---|---|
| `flaw.true_love_major/_minor` | *"see the True Love Virtue on page 113"* (:6873) | `virtue.true_love_pc` (ArMDE:5173-5178) | **Yes, a partitioning rule** — *"Your True Love must be a non-player character"* here, *"Your True Love is another player character, who must also have this Virtue. True Love is never one-sided"* there (:5175). The Flaw and the Virtue are disjoint by construction; the book states no incompatibility between them, and the reciprocity that **is** stated sits on the Virtue. | **Yes — F-333 and F-334 (B09)**, which already rate `virtue.true_love_pc`'s classification and the inexpressible reciprocity. **Not re-reported.** The Flaw side is clean: nothing is owed on `flaw.true_love_*`. |
| `flaw.uncertain_faith` | *"Realms of Power: The Divine Revised Edition, pages 38-41, 87, 103"* (:6901) and *"Realms of Power: The Infernal, page 31"* (:6903) | `RoP:D` and `RoP:I`, both **in `rules/source/en/`** | **No rule this entry needs.** The pointers name *where the rolls being penalised are defined*, not a rule about the Flaw. The −3 and the +1 Personality Trait are both stated in ArMDE itself and both reach the shipped `description`. Unlike B17's `flaw.seeker`, nothing in the pointed-to book restates the Flaw's own eligibility. | No — and no escalation is needed, which is the distinction from Q-135. |
| `flaw.uncertain_faith` | *"not compatible with the True Faith Virtue"* (:6905) | `virtue.true_faith` (exists, Major, General, `creation_effect`) | **Yes.** Encoded on neither side. | No. **F-527.** |
| `flaw.university_dean` | *"the Virtue Doctor in (faculty)"* (:6925) | `virtue.doctor_in_faculty` (exists, Major, Social Status, `creation_effect`, `grants_reputation{academic, 3}`) | **Yes, twice** — a required Virtue and a forbidden Flaw class. | No. **F-526.** |
| `flaw.viaticarus` | *"he should take the Outcast Social Status Flaw"* (:6983) | `flaw.outcast` (exists) | **Yes, conditionally** — but the condition (*"If the character's status is known in the community"*) is a saga fact the app stores nowhere, and the verb is *should*. It also collides with ArMDE:2816's one-Social-Status rule, since Viaticarus is Supernatural and Outcast is Social Status, so no conflict arises. **No defect.** | `flaw.outcast` carries **F-340 row 8** and the withdrawn **F-481**; neither is touched. |
| `flaw.tragic_life` | *"usually gains the Plagued by Supernatural Entity Flaw instead"* (:6869) | `flaw.plagued_by_supernatural_entity` | **No creation-time rule** — an in-play swap the app models nowhere. | No |
| `flaw.unbearable_to_beings` | *"should take the Blatant Gift instead"* (:6897) | `flaw.blatant_gift` | **Yes, and it is encoded** — as the `incompatible_with` and the `max_total: 1`, together. | n/a — clean |
| **inbound** → `flaw.tragic_life` | *"All Devil Children must take the following Flaw … • Tragic Life (Major, Supernatural)"* (ArMDE:2658-2664) | the **Devil Child** Mythic Companion package | **Yes — a compulsory Flaw plus a budget rule** (*"three more points of Virtues at no cost … an additional seven points of Flaws, each point granting two Virtue points"*). The obligation sits on the **Devil Child** side, not on `flaw.tragic_life`, so **nothing is owed by this entry**. It also gives the Flaw a *third*, contradictory category label — see check 6. | `virtue.devil_child` carries **F-55 (B02)**. The compulsory-Flaw link and the budget rule are **not** in F-55, whose subject is the free Minor Virtue and a wrongly-stated incompatibility. Recorded here as an **extension to F-55**, not a new finding, because the defect is on `virtue.devil_child`'s row. |
| **inbound** → `flaw.surgical_empiricus` | *"#### Western Christendom … **Minor Flaws**: Failed Journeyman, Failed Master, Failed Monk/Nun, Surgical Empiricus"* (ArMDE:2909, under :2890 *"the available options vary depending on the society they live in"*) | the Social-Statuses-by-Culture tables | **Yes — a culture eligibility restriction**, and it is the *restrictive* case: Surgical Empiricus is listed **only** under Western Christendom. | **B16 read the same table family** (`batch-16.md:538`) and rated it *"no new rule"* — correctly, because its five entries all sit in the **All Cultures** list (:2899), which restricts nothing. This is the first span to hit a *culture-specific* row. **F-536.** |
| **inbound** → `flaw.usurer` | *"#### All Cultures … **Minor Flaws**: Branded Criminal, Companion Animal, Outcast, Outlaw Leader, Outsider, **Usurer**"* (ArMDE:2899) | the same tables | **No new rule.** All Cultures restricts nothing, and the magnitude and category match. | **Yes — `batch-16.md:538` rated this exact row.** Confirmed, not re-reported. |
| **inbound** → `flaw.unstructured_caster` | *"Any Flaw that is only appropriate to Hermetic Magic (for example, Deficient Technique or **Unstructured Caster**) cannot be taken with this Flaw"* (ArMDE:6148) | `flaw.limited_magic_powers` / the entry at ArMDE:6145-6148, **B13's span (6068-6235)** | **Yes — a prohibition.** It is stated *from the other entry's side* and names a **class** (Hermetic-only Flaws), not a fixed list, so the fix belongs on the ArMDE:6148 entry rather than on `flaw.unstructured_caster`. | **Checked in `corrections.md` § 1 B13 and in `batch-17.md`: no finding exists for it.** Recorded as **Q-138** rather than numbered, because it is another batch's entry and the fix shape (a category-valued incompatibility, which `incompatible_with` cannot express) is a decision, not a data edit. |
| **inbound** → `flaw.visions` | *"If the character also has the Visions Flaw, then some of her visions are true, but most of them are meaningless jumbles"* (ArMDE:6228) | `flaw.delusions`-family entry at :6225-6228, **B13's span** | **No mechanical rule** — a colour clause about how two Flaws read together. No number, no roll, no prohibition. | No |
| **inbound** → `flaw.unbearable_to_beings` | *"Specific Flaws (such as Offensive to Divine Beings and **Unbearable to Divine Beings**) … may cause creatures with Divine Might to react negatively"* (ArMDE:21130) | the Divine Might chapter | **No new rule** — it confirms that `being.divine` is a real parameter value and that the Flaw's effect is a Gift-like social reaction. Supports the check-9 verdict. | No |
| **inbound** → `flaw.viaticarus` | *"you are dead to the world in some sense, and must enter a monastery if you recover (see the Viaticarus Flaw, page 150)"* (ArMDE:21514) | the Extreme Unction sacrament section | **No.** *"many people believe"* frames it as in-world opinion, not a rule; the sentence's only hard claim (*"Extreme Unction can only be administered by a priest"*) is about the sacrament, not the Flaw. | No |
| **inbound** → `flaw.tragic_life` | ArMDE:3655 (`virtue.demonic_blood` suggests offsetting with Tragic Life), :5099 (`virtue.tainted_treasure` describes swapping it out) | two other catalogue entries | **No.** Both are suggestions, and both partners already carry findings of their own (F-50/F-477 for Demonic Blood). | `virtue.demonic_blood` → F-50, F-477 (withdrawn dup). Not touched. |
| **inbound** → `flaw.tormenting_master`, `flaw.twilight_prone`, `flaw.susceptibility_to_divine_power`, `flaw.susceptibility_to_infernal_power`, `flaw.true_love_*`, `flaw.visions` | ArMDE:1486, :1617, :1716, :1913, :2064, :2117, :5163, :20253 | sample characters and templates | **No.** Example characters; no rule. | No |
| **inbound** → `flaw.visions` | *"Grant Visions, 5 points, Init –12, Vim"* (:19572-19573), Maria the volcano seer (:19546) | a creature power and a story example | **No.** In-play acquisition, which the app models nowhere. Same shape as B17's `flaw.simple_minded` rows. | No |
| **inbound** → `flaw.tainted_with_evil` | *"the creature acquires the Tainted with Evil Flaw which affects the reactions of both humans and other animals towards it"* (:21008, Corrupted Beasts) | the Infernal-corruption rules | **No new creation rule**, but it **widens the audience**: the Flaw's own passage says only that others feel ill at ease, and :21008 extends it to animals. No number either way. | No |

### Inbound sweep

All 34 entry names were grepped across the full English core rulebook
(`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md`) with `rg -a`.
Excluding the 34 headings, the "List of Flaws" index links (ArMDE:5301-5634) and
the alphabetical-index rows (:25533-25640), the sweep returned **26 sites outside
those three**, enumerated rather than sampled:

| Group | Lines | n | Verdict |
|---|---|---|---|
| Sample characters and templates | :1486, :1617, :1716, :1913, :2064, :2117, :5163, :20253 | 8 | no rule |
| Real content, rated in the table above | :2662, :2899, :2909, :3655, :5099, :6148, :6228, :21008, :21130, :21514 | 10 | see table |
| `virtue.true_love_pc`'s own passage and its index links | :3323, :5173, :5175 | 3 | the pointer's **target**; rated in the table |
| `flaw.visions` in a creature power and a story example | :19546, :19572, :19573 | 3 | in-play acquisition, not modelled |
| Substring false hits on the spell *Visions of the Infernal Terrors* | :15176, :24022 | 2 | not a Flaw |

8 + 10 + 3 + 3 + 2 = **26**.

**This number was corrected after the first draft, which said "21 sites … rated
above: sixteen" — both figures were wrong**, produced by eyeballing a grep block
rather than enumerating it. The corrected count is above and every line is listed,
so it can be checked rather than trusted.

**Two of the twenty-six exposed a defect** (the Western Christendom culture row →
F-536; the ArMDE:6148 Hermetic-only-Flaw prohibition → Q-138), **three were
confirmations** (:2899, :21008, :21130), and the rest carried no rule.

**Every out-of-span entry the sweep reached — `virtue.true_love_pc`,
`virtue.devil_child`, `virtue.demonic_blood`, `virtue.tainted_treasure`,
`flaw.plagued_by_supernatural_entity`, `flaw.outcast`, `flaw.blatant_gift`,
`flaw.magical_air`, `virtue.true_faith`, `virtue.doctor_in_faculty`,
`flaw.poor`, `virtue.famous`, `virtue.hermetic_prestige` — was looked up in
`corrections.md` § 1 and in `batch-17.md` before anything was numbered.** Five
already carry findings (F-333/F-334, F-55, F-50/F-477, F-340/F-481) and **none
is re-reported.**

### The body-level duplicate check

`corrections.md` § 4.2 admits its nine duplicates are *"a floor, not a ceiling"*
because the search keyed on the entry id in a finding's **heading**. B17 ran the
body-level scan for its span; this batch ran the same one for 6803-6988.

A grep of the whole of `corrections.md` **and** `batch-17.md` for all 35 entry
ids returns exactly **three** hits, none of them a finding on a B18 entry:
`corrections.md:498` and `:499` (F-333/F-334, on `virtue.true_love_pc`, a
different id) and `batch-17.md:559` (the `NO_RULE_DESPITE_TOKEN` list, which
names `flaw.true_love_major` and `flaw.true_love_minor` as exemptions and
explicitly declines to rate them, being out of B17's span).

**So no prior finding lands on any entry in 6803-6988, and no body-only duplicate
exists for this span.** Findings F-523…F-537 are all new.

### `NO_RULE_DESPITE_TOKEN` — two of its rows are in this span, and both readings hold

`batch-17.md:556-560` flagged that the exemption list names
`flaw.true_love_major` and `flaw.true_love_minor`, both **outside** B17's span
and therefore unrated. They are in mine, so the written reading was checked.

The exemption exists because the True Love passage contains
`equal to or better than the player character` (ArMDE:6877) — the
`MECHANICAL_PHRASES` needle `equal to` — in a sentence that is **not** a
mechanical rule but the book's guidance on *which magnitude to pick*. That
reading is **correct**: ArMDE:6877 decides Major versus Minor from the NPC's
competence, and the catalogue already models the choice as two ids with a
symmetric incompatibility. No number, no roll, no cap follows from it. **Both
exemptions stand; no finding is raised against either.**

### What the phrase screen missed — the exact wording, in both locales

The span sits inside `SWEPT_BLOCKS`' `(ArMDE, 5639, 7113)` entry, swept
2026-09-15 and re-swept 2026-09-19, green both times, and it saw none of the
seven `narrative` misclassifications below. Every string is verbatim from the
cited line. This table extends `corrections.md` § 3.1 and `batch-17.md`'s.

| Entry | ArMDE | EN wording the screen missed | DE wording the screen missed |
|---|---|---|---|
| `flaw.suppressed_gift` | :6805 / DE :6805 | `the character cannot perform Hermetic magic, improve his Arts, or perform the Parma Magica. His Arts do provide him with Magic Resistance and he continues to suffer the negative social penalties of The Gift.` | `kann der Charakter keine hermetische Magie wirken, seine Künste verbessern oder die Parma Magica praktizieren. Seine Künste verleihen ihm jedoch weiterhin Magieresistenz, und er leidet weiterhin unter den negativen sozialen Nachteilen der Gabe.` |
| `flaw.susceptibility_to_warping` | :6835 / DE :6835 | `the character gains one additional Warping Point associated with that same Realm. This means that if he gains a single Warping Point from each Realm in a year, he gains four additional Warping Points, one from each Realm, at the end of that year.` | `erhält der Charakter einen zusätzlichen Verzerrungspunkt, der mit derselben Sphäre verbunden ist. Das bedeutet: Erhält er in einem Jahr einen einzelnen Verzerrungspunkt aus jeder Sphäre, erhält er am Ende dieses Jahres vier zusätzliche Verzerrungspunkte, je einen aus jeder Sphäre.` |
| `flaw.tainted_with_evil` | :6845 / DE :6845 | `Gaining a positive Reputation is impossible.` | `Es ist unmöglich, eine positive Reputation zu erlangen.` |
| `flaw.tragic_life` | :6859 / DE :6859 | `The predisposition toward sin at the character's pivotal moment should be represented with a sinful Personality Trait.` | `Die Neigung zur Sünde am Wendepunkt des Charakters sollte durch einen sündhaften Persönlichkeitszug dargestellt werden.` |
| `flaw.unnatural_magic` | :6933 / DE :6933 | `none of the character's Creo rituals have a permanent effect` … `He also cannot extract vis from an aura using Creo` | `haben keine seiner Creo-Rituale einen dauerhaften Effekt` … `Er kann auch keine Vis aus einer Aura mit Creo gewinnen` |
| `flaw.unspecialized` | :6945 / DE :6945 | `The character does not have any specialties for any of her Abilities.` | `Der Charakter hat keine Spezialisierungen in irgendeiner seiner Fertigkeiten.` |
| `flaw.unstructured_caster` | :6949 / DE :6949 | `You cast all Formulaic spells as though they were Ritual spells (including the need for vis), and you may not learn Ritual spells at all.` | `Du wirkst alle Formulaischen Zauber so, als wären sie Ritualzauber (einschließlich des Vis-Bedarfs), und du kannst überhaupt keine Ritualzauber erlernen.` |

**Which § 3.1 families these fall into, and which are new.** Family 1
(Prohibition) covers `cannot perform`, `cannot extract`, `may not take` — and its
English side would catch `flaw.suppressed_gift`, `flaw.unnatural_magic` and
`flaw.unstructured_caster`. Family 1's `is impossible` / `unmöglich` catches
`flaw.tainted_with_evil` in **both** locales, which is the only one of the seven
that the *proposed* list already handles end to end. Family 14 (Named rulebook
terms) lists `an additional Personality Trait of`, which does **not** match
`flaw.tragic_life`'s unnumbered *"a sinful Personality Trait"*. **Three forms are
new and are in neither § 3.1 nor `batch-17.md`'s four:**

| New form | EN | DE | From |
|---|---|---|---|
| **An additive counter with no sign character** | `gains one additional Warping Point`, `four additional Warping Points` | `erhält … einen zusätzlichen Verzerrungspunkt`, `vier zusätzliche Verzerrungspunkte` | `flaw.susceptibility_to_warping` — the rule is a whole worked example and contains no sign, no `Ease Factor` and no cap idiom |
| **A bare "does not have any X"** — an absolute stated as a *state*, not a prohibition | `does not have any specialties` | `hat keine Spezialisierungen` | `flaw.unspecialized` |
| **An unnumbered mandated Personality Trait** | `should be represented with a sinful Personality Trait` | `sollte durch einen sündhaften Persönlichkeitszug dargestellt werden` | `flaw.tragic_life` |

### This span's German prohibitions, in `batch-17.md`'s terms

B17 established that four of its five German prohibitions put words between the
two halves of the negation, which `has_mechanical_phrase`'s contiguous substring
match (`uncomputed_clauses.rs:234-241`) cannot span. **Reported here in the same
terms, so the two spans can be added rather than re-derived:**

| Entry | DE source sentence (DE line) | Proposed § 3.1 token | Matches? |
|---|---|---|---|
| `flaw.suppressed_gift` | `kann der Charakter keine hermetische Magie wirken` (6805) | `kannst keine` / `kann … nicht` | **No** — `kann` is followed by `der Charakter` before `keine`, three words apart |
| `flaw.unnatural_magic` | `Er kann auch keine Vis aus einer Aura mit Creo gewinnen` (6933) | `kann keine` | **No** — `auch` sits between `kann` and `keine` |
| `flaw.unstructured_caster` | `du kannst überhaupt keine Ritualzauber erlernen` (6949) | `kannst keine` | **No** — `überhaupt` sits between |
| `flaw.unspecialized` | `Der Charakter hat keine Spezialisierungen` (6945) | — | **No token proposed at all**; `hat keine` is in no family |
| `flaw.tainted_with_evil` | `Es ist unmöglich, eine positive Reputation zu erlangen` (6845) | `unmöglich` | **Yes** — § 3.1 family 1 lists this exact contiguous form |

**Four of five again, and the pattern is now measured across two adjacent spans:
nine of eleven German prohibitions in ArMDE:6663-6988 defeat a contiguous
substring match.** Two of this batch's four misses are a *new* shape B17 did not
see — a single adverb (`auch`, `überhaupt`) wedged into an otherwise contiguous
`kann … keine`, which a two-token proximity match with a small window would
catch and a longer needle never will.

### All 12 `uncomputed_rule` entries against the guard's own detector, by hand

The brief flagged a hypothesised shape: *an `uncomputed_rule` whose `description`
does not in fact state the rule in both locales*, which
`every_uncomputed_rule_entry_states_its_rule_in_every_locale`
(`uncomputed_clauses.rs:316-350`) is supposed to guarantee. **There is no instance
in this span**, and that was established by running each entry's shipped text
through `states_a_mechanical_rule` = `has_signed_number || has_botch_term ||
has_mechanical_phrase` (`:244-246`) rather than by assuming the green guard.
All 12 carry a `description` in **both** locales, and each satisfies the detector:

| Entry | EN satisfies via | DE satisfies via |
|---|---|---|
| `surgical_empiricus` | `+1` (signed) | `+1-Bonus` (signed) |
| `susceptibility_to_sunlight` | `-2` | `-2-Abzug` |
| `twilight_prone` | `botch` | `Patzer` (`BOTCH_TERMS` carries `patzer`) |
| `unbearable_to_beings` | `-3` | `-3-Abzug` |
| `uncertain_faith` | `-3`, `+1` | `-3-Abzug`, `+1` |
| `uncontrollable_strength` | `ease factor`, `botch die` | `Schwierigkeitsgrad`, `Patzerwürfel` |
| `uninspirational` | `may not be greater than`, `-3` | **`-3-Abzug` only** — the German cap idiom's needle is dead; see F-537 |
| `university_dean` | `at least` (40 years old) | `mindestens` |
| `unlucky` | `-1`, `-3` | `-1 bis -3-Abzug` |
| `unpredictable_magic` | `stress die`, `botch` | `Stresswürfel`, `Patzer` |
| `vengeful_powers` | `botch dice` | `Patzerwürfel` |
| `viaticarus` | `or more`, `equal to` | `oder mehr` |

`BOTCH_TERMS` is `["botch", "patzer", "patzen", "patzt", "gepatzt"]` and
`has_signed_number` accepts `-`, `+`, U+2013 and U+2212 followed **immediately**
by an ASCII digit (`uncomputed_clauses.rs`, both read). Every cell above was
checked against those definitions, not against the guard's exit code.

**The one near-miss is worth recording**: `flaw.uninspirational`'s German passes
only on the unrelated `-3-Abzug` in its *third* sentence. Its actual rule —
*"Sein Präsenz- und Kommunikationswert darf nicht größer als 0 sein"* — is
invisible to the German half of the detector, which is F-537.

### A defect in a shipped `MECHANICAL_PHRASES` needle — F-537

Separate from the missing forms above: one German needle **already in the list**
matches nothing in the German core book, because its word order is not the
book's. See F-537.

### Counting note

Figures in this Method that were counted rather than estimated, and how: the
**34** headings and **35** ids (`grep -n "^#### "` filtered to 6803-6988, and
`jq '… | .[595:630] | length'` — **but note the slice only works after sorting**:
`virtues_flaws.json` is stored **id-sorted**, where indices 595-629 are Virtues.
The audit's canonical ordering is by `(source.file, source.lines[0])`, so the
reproducible form is
`jq '[.[] | select(.source.lines[0] >= 6803 and .source.lines[0] <= 6988)] | length'`.
The verification pass re-derived the 595-629 window that way and it does land on
`flaw.suppressed_gift` … `flaw.visions`); the **16** Flaws carrying `grants_reputation`
(`jq` over the whole catalogue, enumerated in F-526 rather than sampled — **17**
is the *effect-row* count, and the verification pass corrected this line's unit);
the **4** `index_categories` users (`jq '[.[] | select(.index_categories)] | length'`);
the **2** `ParameterDomain::Category` users (`jq` over `parameters[].domain`);
the **26** inbound sites (`rg -a`, then the three index families subtracted by
line range and the remainder listed individually — this line said **21**, the
withdrawn first-draft figure, which the inbound-sweep section itself already
retracted; corrected by the verification pass). **Where a count mattered it
was taken with `jq` over the parsed JSON rather than with a line-counting
`grep -c`** — B17's recorded unit trap.

---

## Verdicts

`data` covers checks 1–11; `text` covers check 12.

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `flaw.suppressed_gift` | 6803-6810 | **fail** | pass | pass | F-531 — `narrative` on four absolutes; prereq, magnitude, categories all correct |
| `flaw.surgical_empiricus` | 6811-6814 | pass | **fail** | pass | F-536 — the Western Christendom culture restriction reaches neither locale |
| `flaw.susceptibility_to_divine_power` | 6815-6818 | **escalated** | pass | pass | **Q-136** — `in_play_effect` on a surfaced-only kind; D3's reasoning vs B17's F-515 precedent |
| `flaw.susceptibility_to_faerie_power` | 6819-6822 | **fail** | **fail** | pass | F-523 — contradicts D3 by name |
| `flaw.susceptibility_to_infernal_power` | 6823-6826 | **fail** | **fail** | pass | F-523 — contradicts D3 by name |
| `flaw.susceptibility_to_sunlight` | 6827-6830 | pass | pass | pass | clean; `-2` ASCII in both locales |
| `flaw.susceptibility_to_warping` | 6831-6838 | **fail** | pass | pass | F-528 — a fully quantified Warping rule shipped `narrative` |
| `flaw.tainted_offspring` | 6839-6842 | pass | pass | pass | clean; `tainted: true` correct |
| `flaw.tainted_with_evil` | 6843-6846 | **fail** | **fail** | pass | F-525 — an absolute broken by a legal pair |
| `flaw.temperate` | 6847-6850 | pass | pass | pass | clean; the Personality-Trait obligation is ArMDE:2502's general rule, not this entry's |
| `flaw.tormenting_master` | 6851-6854 | pass | **fail** | pass | F-532 — *"only applicable to magi"*, `prerequisites: null` |
| `flaw.tragic_life` | 6855-6870 | **fail** | pass | pass | F-535 — a mandated sinful Personality Trait (marginal, and rated as such) |
| `flaw.true_love_major` | 6871-6878 | pass | pass | pass | clean; F-487 satisfied; `NO_RULE_DESPITE_TOKEN` reading upheld |
| `flaw.true_love_minor` | 6871-6878 | pass | pass | pass | clean; same |
| `flaw.twilight_prone` | 6879-6882 | pass | pass | pass | clean; no Twilight model exists (D3) |
| `flaw.tzadik_nistar` | 6883-6886 | pass | pass | pass | clean |
| `flaw.unbaptized` | 6887-6890 | pass | pass | pass | clean |
| `flaw.unbearable_to_beings` | 6891-6898 | pass | pass | pass | **clean, and the span's model entry** — prereq, `incompatible_with`, `parameters`, `max_total: 1`, `index_categories` all match the passage |
| `flaw.uncertain_faith` | 6899-6906 | pass | **fail** | pass | F-527 — the stated True Faith incompatibility is encoded on neither side |
| `flaw.uncontrollable_strength` | 6907-6910 | pass | pass | pass | clean; `Prereq` has no Characteristic-comparing variant (D3) |
| `flaw.unhappily_married` | 6911-6914 | pass | pass | pass | clean |
| `flaw.unimaginative_learner` | 6915-6918 | pass | pass | pass | clean; `-3`/`vis` sign correct and the amount reaches the read-out |
| `flaw.uninspirational` | 6919-6922 | pass | pass | pass | clean; no characteristic-cap mechanism exists (D3) |
| `flaw.university_dean` | 6923-6926 | pass | **fail** | pass | F-526 — three eligibility clauses, none encoded; two are expressible today |
| `flaw.unlucky` | 6927-6930 | pass | pass | pass | clean |
| `flaw.unnatural_magic` | 6931-6934 | **fail** | pass | pass | F-530 — two absolutes shipped `narrative` |
| `flaw.unpredictable_magic` | 6935-6938 | pass | pass | pass | clean |
| `flaw.unruly_air` | 6939-6942 | pass | pass | pass | clean |
| `flaw.unspecialized` | 6943-6946 | **fail** | **fail** | pass | **F-524 — the engine prints a +1 the Flaw forbids** |
| `flaw.unstructured_caster` | 6947-6950 | **fail** | pass | pass | F-529 — a Ritual-spell prohibition shipped `narrative` |
| `flaw.usurer` | 6951-6954 | pass | **fail** | pass | F-534 — the income clause reaches neither locale (D5) |
| `flaw.vendetta` | 6955-6958 | pass | **fail** | pass | F-533 — House/magus restriction, `prerequisites: null` |
| `flaw.vengeful_powers` | 6959-6976 | pass | **fail** | pass | F-538 — *"may be taken as a Hermetic Flaw"* is unrepresentable, though `taken_as` exists |
| `flaw.viaticarus` | 6977-6984 | pass | pass | pass | clean; the Outcast pointer is conditional and advisory |
| `flaw.visions` | 6985-6988 | pass | pass | pass | clean; both index categories carried |

**Arithmetic — recounted row by row by the verification pass, which corrected
both figures.** **18 entries pass every check; 16 entries fail at least one; 1
entry (`flaw.susceptibility_to_divine_power`) is ESCALATED and is NOT marked
checked.** 18 + 16 + 1 = 35. (The Method first read **19 / 15**; counting the
table's `pass pass pass` rows gives eighteen, and the sixteen failing ids are
`suppressed_gift`, `surgical_empiricus`, `susceptibility_to_faerie_power`,
`susceptibility_to_infernal_power`, `susceptibility_to_warping`,
`tainted_with_evil`, `tormenting_master`, `tragic_life`, `uncertain_faith`,
`university_dean`, `unnatural_magic`, `unspecialized`, `unstructured_caster`,
`usurer`, `vendetta`, `vengeful_powers`.) The 16 findings land on **16 entries
plus one test file** (F-537, `uncomputed_clauses.rs`) — also corrected, from
*fifteen entries*: fifteen findings land on entries and one of them, F-523,
covers **two** (`flaw.susceptibility_to_faerie_power` and
`flaw.susceptibility_to_infernal_power`), so 14 + 2 = 16.

---

## Findings

### F-523 — D3 names two entries by id and the data still contradicts it

**Entries.** `flaw.susceptibility_to_faerie_power` (ArMDE:6819-6822),
`flaw.susceptibility_to_infernal_power` (ArMDE:6823-6826).

**The passage, verbatim.**

> ArMDE:6821 — "In addition, your Magic Resistance score, including Parma Magica,
> against faerie effects is halved. If someone else uses their Parma Magica to
> protect you, their resistance is not affected and you benefit normally."
>
> ArMDE:6825 — "You get only half your normal Magic Resistance score against
> infernal effects, though if someone else's Parma Magica is protecting you, it
> counts normally."

German, DE 6821 / DE 6825:

> "Darüber hinaus wird dein Magieresistenzwert, einschließlich der Parma Magica,
> gegen Feeneffekte halbiert."
>
> "Du erhältst nur die Hälfte deines normalen Magieresistenzwerts gegen Höllische
> Effekte."

**The ruling.** `decisions.md` **D3** ("realm-scoped Magic Resistance") names
both entries and rules:

> "**Both entries are `uncomputed_rule`**, with the halving written out in
> `description` in **both** locales — the rule reaches the player as text rather
> than as a number, which is exactly what `uncomputed_rule` asserts."

**Current data.** Both ship `"classification": "in_play_effect"` with
`{"type":"magic_resistance_mod","kind":"susceptible_faerie"}` /
`{"kind":"susceptible_infernal"}`, and **no `description` in either locale**.

**Why it is wrong.** Two independent grounds, and the first is not a judgement:

1. **D3 is binding and the data does not match it.** The README calls
   `decisions.md` binding and tells every batch to apply rather than re-litigate
   it. The ruling was taken; the data was never changed.
2. **`in_play_effect` is false on the engine's own terms.**
   `engine-semantics.md` A37 (verified against `derived.rs::surfaced_modifiers`)
   records `susceptible_faerie` and `susceptible_infernal` as two of the four
   **surfaced-only** kinds, "pushed with `amount: 0` — so even a magnitude is not
   carried for them", and states outright that "Both realm susceptibilities —
   which the book says *halve* resistance against that realm — move no number;
   the per-Form grid has no realm axis". `in_play_effect` claims the engine
   computes the rule during play. It does not.

**What the player actually gets today.** A magus with either Flaw sees a row in
the in-play read-out carrying the slug and the number **0**. His Magic Resistance
grid is unchanged in every Form. The halving reaches him only because the shipped
`summary` happens to be the entire passage — the `description` slot D3 names is
empty.

**Correct value.** `"classification": "uncomputed_rule"` on both. Both will also
need a `source.anchor` (`susceptibility-to-faerie-power`,
`susceptibility-to-infernal-power`, per ArMDE:5453-5454), which is the convention
for `uncomputed_rule` entries.

**The `description` half of D3 is narrowed here, on `RULES.md` evidence I found
after drafting this finding, and the narrowing is recorded rather than quietly
applied.** `RULES.md:4728-4731` states a deliberate, general policy:

> "`flaw.missing_ear` is the one that gained no `description`: its passage is a
> single sentence that its `summary` already carries in full, so a `description`
> would be a byte-identical duplicate. **Same reading as
> `flaw.susceptibility_to_divine_power` in the previous pass.**"

Both of this finding's entries are in exactly that position: their shipped
`summary` in **both** locales is the **entire** passage, ArMDE:6821 and :6825
word for word (verified by comparison, not by length). So D3's *"written out in
`description`"* is already satisfied in substance, by the slot the guard itself
falls back to (`uncomputed_clauses.rs:298-302` reads `description`, else
`summary`). **The live defect is the classification alone.** Writing a
byte-identical `description` would contradict a recorded policy to gain nothing;
if Phase 2 wants the `description` slot filled for consistency, that is a
decision about the policy, not about these two entries.

**Whether the `effects` row should survive the move.** It should, and `RULES.md`
argues the case better than this finding could. `RULES.md:5138` records that the
four conditional kinds are surfaced with `amount: 0` because "each carries a scope
the flat per-Form figure has no axis for (a realm, an aura, a scene condition plus
the incoming spell's level), so listing them keeps them from being silently
dropped *and* from being applied where the book does not apply them."
`RULES.md:4919` lists both entries under "Magic-resistance modifier". Removing the
row would drop the Flaw from the in-play read-out. **Keep the effect; move the
classification.** Same call B17 made for `flaw.short_ranged_magic`.

**`RULES.md` was searched before this finding was written and it does not defend
`in_play_effect`** — it defends the *effect variant* and the *surfacing*, which is
a different claim, and one this finding agrees with.

**Severity — MEDIUM.** A lost-rule / provenance defect, not a miscalculation:
`classification` is read by no production code (`engine-semantics.md` B1), and the
halving reaches the player through `summary` today. It is rated **above** an
ordinary misclassification because a *binding decision* is what the data
contradicts, and because a batch reading only the data would keep re-deriving the
same question D3 already closed.

**Kind of defect.** `class` + `desc`.

---

### F-524 — the sheet prints a +1 on a character whose Flaw forbids specialties

**Entry.** `flaw.unspecialized` (ArMDE:6943-6946), shipped `narrative`.

**The passage, verbatim and complete — it is one sentence.**

> ArMDE:6945 — "The character does not have any specialties for any of her
> Abilities."
>
> DE 6945 — "Der Charakter hat keine Spezialisierungen in irgendeiner seiner
> Fertigkeiten."

**Current data.** `{"id":"flaw.unspecialized","kind":"flaw","magnitude":"minor",
"categories":["general"],"classification":"narrative","entity_kinds":["character"]}`
— no effects, no prereqs, no description in either locale.

**What happens when a player does what the entry says.** He takes Unspecialized,
then opens the Abilities list and types a specialty into any Ability row. Nothing
objects. Then:

1. `types.rs::AbilityScore::specialty` is `Option<String>` free text
   (`ability.rs:68`), stores it, and the save round-trips it.
2. `export.rs` prints it under the `ability-specialty-label` column on the
   character sheet.
3. **`derived/combat.rs::specialization_bonus` gives him +1.** Read in full:

   ```rust
   fn specialization_bonus(entity, _ruleset, slot, weapon) -> u8 {
       if !slot.specialization_applies { return 0; }
       let has_specialty = entity.ability_scores.iter().any(|a| {
           a.ability == weapon.ability
               && a.specialty.as_deref().is_some_and(|s| !s.trim().is_empty())
       });
       u8::from(has_specialty)
   }
   ```

   It consults the weapon's Ability and the specialty string. **It consults no
   Virtue and no Flaw.** `combat.rs:190` folds the result straight into the
   combat total (`let spec_bonus = i32::from(specialization_bonus(...))`).

So a grog with Unspecialized and a Single Weapon specialty of "long sword" is
printed an **Attack and Defense total one point higher than the rules permit** —
and the Flaw he paid for is the one that forbids the specialty generating it.

**Why it is wrong, in two layers.**

- **The computation.** This is the mirror of F-449: an effect folded over a case
  the rule excludes. `u8::from(has_specialty)` can only return 0 or 1, and
  nothing can make it return 0 for this Flaw, because no `Effect` variant and no
  validator forbids a specialty.
- **The classification.** `narrative` asserts the book states nothing mechanical.
  The entry's whole text is a mechanical absolute about a field the engine
  stores and reads. The phrase screen missed it because "does not have any
  specialties" contains no sign, no botch term and none of the **33**
  `MECHANICAL_PHRASES` (the figure was **20**; the verification pass counted the
  list at `uncomputed_clauses.rs:179-219` item by item and it holds 33 needles —
  12 cap/floor/threshold, 2 target-number, 5 formula, 6 rounding, 4 dice, 2
  absolute, 2 magnitude); the German `hat keine Spezialisierungen` is not covered by
  any form § 3.1 proposes either.

**The engine half is confirmed verbatim by the verification pass**, which read
`crates/arm-rules/src/derived/combat.rs:244-257` rather than this finding's quote
of it: the body is exactly as transcribed, `combat.rs:190` folds it in as
`let spec_bonus = i32::from(specialization_bonus(entity, ruleset, slot, weapon));`,
and a grep of `crates/` and `ui/src/` for `unspecialized` returns **no hit at
all** — no validator, no effect, no UI branch consults the Flaw. The path is
reachable from the UI: `slot.specialization_applies` is a checkbox
(`ui/src/lib/components/EquipmentTab.svelte:160`, set via
`state.svelte.ts::setEquipmentSpecialization`) and the specialty is free text.

**Correct value — and the `description` half is narrowed here.** The shipped
`summary` **already carries the whole rule, verbatim and in both locales**
(EN *"The character does not have any specialties for any of her Abilities."*,
DE *"Der Charakter hat keine Spezialisierungen in irgendeiner seiner
Fertigkeiten."*), because ArMDE:6945 is a single sentence. Under
`RULES.md:4728-4731`'s recorded policy — the one F-523 invokes — writing that
sentence into `description` would be a **byte-identical duplicate**. So the
correct edit is `"classification": "uncomputed_rule"` **and nothing added to the
text**, plus enforcement. This narrowing changes nothing about the severity: the
rule already reaches the player as text and the engine contradicts it anyway,
which is the whole point. Two enforcement routes, and the choice is a decision
rather than an obvious edit:

- A validator: `validation/scores.rs` refuses a non-empty `specialty` on any
  Ability row when `flaw.unspecialized` is selected (an error under `Enforced`,
  a warning under `Advisory`). This is the cheap one and it is the honest one —
  the book forbids the *state*, not the bonus.
- Or `specialization_bonus` learns to return 0 when the Flaw is held. This fixes
  the number but leaves the sheet printing specialties the character cannot have,
  and leaves every other specialty consumer wrong.

**The first is correct and the second is a patch**, because ArMDE:6945 is a
statement about what the character *has*, not about what he *rolls*.

**Severity — HIGH.** `CLAUDE.md`'s top class: wrong rules output. The app's
purpose is computing correct Ars Magica characters, and this one computes a
combat total the rulebook forbids, silently, on a legal selection.

**Kind of defect.** `engine` + `class`.

**Interaction worth recording for Phase 2.** `flaw.surgical_empiricus`
(ArMDE:6813) *instructs* the player to "Pick a type of surgery as your
specialization in that Ability" and promises "the usual +1 bonus". Unspecialized
+ Surgical Empiricus is therefore a **directly contradictory legal pair** that
the book does not name. It is **Q-19**'s question ("may the catalogue assert an
incompatibility no passage states, when the pair is logically contradictory?"),
not a new one, and it is raised here as the concrete instance Q-19 lacked.

---

### F-525 — "Gaining a positive Reputation is impossible" is broken by a legal pair

**Entry.** `flaw.tainted_with_evil` (ArMDE:6843-6846), shipped `narrative`.

**The passage, verbatim.**

> ArMDE:6845 — "An air of corruption surrounds you as a result of something you,
> your parents, or your ancestors did. Others naturally feel very ill at ease
> around you, and can easily grow to hate you. **Gaining a positive Reputation is
> impossible.** Magi do not react as strongly to this attribute as normal people."
>
> DE 6845 — "**Es ist unmöglich, eine positive Reputation zu erlangen.**"

**Current data.** `classification: "narrative"`, no effects, no
`incompatible_with`, no description.

**What happens when a player does what the entry says.** Nothing stops him from
gaining a positive Reputation, and a second legal selection hands him one:

| Partner | Effect | Legal alongside Tainted With Evil? |
|---|---|---|
| `virtue.famous` | `grants_reputation{score: 4}` with **no `kind`** — A25's wildcard, "player-chosen type" | yes; neither entry names the other |
| `virtue.hermetic_prestige` | `grants_reputation{kind: "hermetic", score: 4}` | yes |
| `virtue.doctor_in_faculty` | `grants_reputation{kind: "academic", score: 3}` | yes |

`validation/scores.rs::validate_reputations` authorizes a `Reputation` row
against the grant's **kind and count only** (A25: "it never checks
`Reputation.score` against `ReputationGrant.score`"), so the player enters
"Famous, Local 4" and the character validates clean. He holds a Flaw whose whole
mechanical content is that this cannot happen.

**Why it is wrong, in two layers.**

- **F-459's shape.** Nothing in the validation layer ranges over pairs, and this
  absolute is defeated by a second legal selection. Unlike B17's five absolutes,
  which are "not defeated because none is computed at all", this one *is*
  reachable: reputations are validated, so there is a place where the prohibition
  could bite and does not.
- **The classification.** `narrative` claims the book states nothing mechanical.
  "Gaining a positive Reputation is impossible" is an absolute about a validated
  field.

**Correct value.** `"classification": "uncomputed_rule"` with the sentence written
into `description` in both locales — and the enforcement is **blocked on Q-43**,
which this finding is the sharpest instance of. `GrantsReputation` carries no
polarity (`corrections.md` § 3.15: "`grants_reputation` cannot say a Reputation
is **bad**"), so the engine cannot tell a positive Reputation from a negative one
and therefore cannot enforce "no positive Reputation" even in principle. **Settle
Q-43 first**; the classification and the description are correct to write
immediately and do not wait on it.

**Severity — MEDIUM.** A lost rule plus an unenforceable absolute. It is not
rated HIGH because no *number* is computed wrongly — the app does not invent a
Reputation, it merely fails to forbid one — but it is rated above a plain
misclassification because the rule is reachable and is silently not applied.

**Kind of defect.** `class` + `incompat`/`engine` (blocked on Q-43).

---

### F-526 — University Dean states three eligibility clauses and encodes none; two are expressible today

**Entry.** `flaw.university_dean` (ArMDE:6923-6926), shipped `uncomputed_rule`.

**The passage, verbatim.**

> ArMDE:6925 — "The character must have the Virtue Doctor in (Faculty), be at
> least 40 years old, and can not have the Poor Flaw or any other Flaw that
> grants a Bad Reputation."
>
> DE 6925 — "Der Charakter muss die Tugend Doktor in (Fachbereich) besitzen,
> mindestens 40 Jahre alt sein und darf nicht den Fehler Arm oder einen anderen
> Fehler haben, der eine schlechte Reputation verleiht."

**Current data.** `classification: "uncomputed_rule"` (with the sentence
correctly present in the `description` in both locales), **`prerequisites`
absent**, **`incompatible_with` absent**, no effects.

**What happens when a player does what the entry says.** He takes University Dean
on a 25-year-old with no Doctor in (Faculty) Virtue and the Poor Flaw, and the
app builds it without a murmur. The description tells him it is illegal; nothing
else does.

**Reachability, traced by the verification pass rather than assumed — and it
narrows the `flaw.poor` half specifically.** `flaw.university_dean` is `story`,
which the **grog** profile does not permit, so a grog is already refused it by
`validate_permitted_categories`. And `flaw.poor` sits in `forbidden_traits` for
**both** the `magus` and the `mythic_companion` profiles
(`rules/core/character_types.json`), so `validate_forbidden_traits` already
blocks the Dean + Poor pair for those two. **The University Dean + Poor
combination is therefore reachable on the `companion` profile alone.** The other
two clauses — the missing `virtue.doctor_in_faculty` requirement and the sixteen
bad-Reputation Flaws — are reachable on **companion and mythic companion and
magus** alike and are unaffected by this narrowing.

**Why it is wrong.** This is B16's **F-494 shape arriving from `uncomputed_rule`**
— the classification is a claim that the engine cannot compute the rule, and for
**two of the three clauses that claim is false**:

1. **The required Virtue is expressible and the target exists.**
   `virtue.doctor_in_faculty` is in the catalogue (Major, Social Status,
   `creation_effect`, carrying `grants_reputation{academic, 3}`), so
   `"prerequisites": {"kind":"has","value":"virtue.doctor_in_faculty"}` resolves.
   `PrereqCtx::build` (`validation/prereq.rs:153-163`) evaluates `Has` against
   bought and granted rows alike, and `validate_prerequisites` raises the error.
   This is the **identical** fix B17 proposed for `flaw.spontaneous_casting_tools`
   (F-518) and B01 for `virtue.alluring_to_beings` (F-08).
2. **The forbidden Flaw class is expressible, and the data enumerates it.**
   `flaw.poor` exists. And "any other Flaw that grants a Bad Reputation" is not a
   vague class in this catalogue — it is exactly the set of `kind: "flaw"` entries
   carrying a `grants_reputation` effect. **Counted carefully, with the unit
   stated, because the two plausible units differ:**

   | Unit | Value |
   |---|---|
   | distinct **Flaw ids** carrying at least one `grants_reputation` | **16** |
   | `grants_reputation` **effect rows** on those ids | **17** — `flaw.failed_monk` carries two (`local` 2 and `ecclesiastical` 2) |

   The sixteen ids, enumerated rather than counted: `apostate`, `black_sheep`,
   `failed_journeyman`, `failed_master`, `failed_monk`, `failed_student`,
   `feral_scent`, `gabai`, `hedge_wizard`, `infamous`, `infamous_master`,
   `outlaw`, `outlaw_leader`, `outsider_major`, `outsider_minor`, `usurer`.
   **`flaw.poor` is not among them** — it grants no Reputation, which is why
   ArMDE:6925 names it separately. So the forbidden set is **16 + 1 = 17
   partners**, and `flaw.usurer` — an entry in this very span — is one of them.
3. **The age floor is not expressible *as a `Prereq`* — but the engine does know
   the character's age, and the verification pass widens this clause.** `Prereq`
   has no age variant (`All/Any/Nor/Has/House/AbilityMin/ArtMin/IsMagus`,
   confirmed by reading the enum), and the entity's age is not a `Prereq` input.
   **But `Entity::age` exists** — `types.rs:3717`, `pub age: Option<u32>`, doc
   comment *"The character's age in years. Drives the age → max-Ability-score cap
   (ArMDE:2366-2376)"* — and it is already read by a validator of exactly the
   shape this clause needs:
   `validation/life_stage.rs::validate_life_stage_age_meets_minimum` reads
   `entity.age` and raises an error when it is below a floor.

   So the honest statement is **not** that the engine structurally cannot express
   an age floor; it is that **no data mechanism carries one** — there is no
   `min_age` field on a V/F entry and no `Prereq::AgeMin` variant. That is a
   missing-mechanism gap, not D3's structural-impossibility case. **All three
   clauses are expressible with modest machinery, not two**, and the
   `uncomputed_rule` label is therefore weaker still than this finding first
   said. It should nonetheless **stay** `uncomputed_rule` until that mechanism
   exists, since D3's precedent forbids `narrative` and no computed class is
   honest while two of three gates are unenforced.

**Correct value.** Keep `"classification": "uncomputed_rule"` (clause 3 is real),
keep the description, and **add**:

- `"prerequisites": {"kind":"has","value":"virtue.doctor_in_faculty"}`
- `"incompatible_with"` listing `flaw.poor` plus the sixteen bad-Reputation Flaws
  — **on both sides**, because
  `ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
  one-sided declaration.

**A caution about the second half, stated rather than glossed.** A seventeen-way
incompatibility list is large, brittle and will go stale the moment a
Reputation-granting Flaw is added — which collides with `CLAUDE.md`'s
catalogue-size-is-data invariant. The book states a **predicate over the
catalogue**, not a list, and `incompatible_with` can only hold ids. The two
honest options are (a) ship the seventeen ids and add a test asserting the list
equals the derived set, or (b) add a predicate-valued exclusion mechanism. That
choice is **Q-137**, not a data edit this finding can prescribe. The
`virtue.doctor_in_faculty` prerequisite is unblocked and should land regardless.

**Severity — MEDIUM.** Not a wrong number, but the app builds a character the
book explicitly forbids, on three counts, with a Virtue requirement that costs
three Virtue points the player never pays. Rated below F-524 because nothing is
mis-computed, and above a plain misclassification because two of the three gates
exist in the engine today.

**Kind of defect.** `prereq` + `incompat`, with a `class`-adjacent note (the
`uncomputed_rule` label is *partly* false).

---

### F-527 — the True Faith incompatibility is in the shipped text and in neither entry's data

**Entry.** `flaw.uncertain_faith` (ArMDE:6899-6906), shipped `uncomputed_rule`.

**The passage, verbatim.**

> ArMDE:6905 — "Supernatural Abilities originating from the Divine, including
> Methods and Powers, are not penalized. **This Flaw is not compatible with the
> True Faith Virtue.**"
>
> DE 6905 — "**Dieser Fehler ist nicht kompatibel mit der Tugend Wahrer Glaube.**"

**Current data.** `classification: "uncomputed_rule"` (correct — the −3 on holy
rolls and the +1 Personality Trait are both uncomputable), description present and
faithful in **both** locales **including this very sentence**, and
**`incompatible_with` absent**. `virtue.true_faith` likewise carries no
`incompatible_with` naming this Flaw.

**What happens when a player does what the entry says.** He takes Uncertain Faith
and True Faith together. `validation/incompat.rs::validate_incompatibilities` has
nothing to read, so the pair validates clean — and the app then prints a character
holding both a Virtue granting Divine Magic Resistance and a Flaw stating that
"some element of doubt stands between him and the Divine". The user is told the
combination is illegal in the Flaw's own description text and is allowed to build
it anyway, which is the worst of the three possible behaviours.

**Why it is wrong.** F-463/F-465's shape exactly: a prohibition the book states
plainly, carried by no `incompatible_with`. It is the **most clear-cut instance
the audit has found**, because the prohibition is not merely in the source — it is
already in the *shipped* `description` in both locales, so the data contradicts
itself.

**Correct value.** `"incompatible_with": ["virtue.true_faith"]` on
`flaw.uncertain_faith`, and `"flaw.uncertain_faith"` added to
`virtue.true_faith`'s list — **both sides**, per
`validate_incompatibility_symmetry`.

**Whether `Prereq::Nor` is needed instead.** It is **not**, and this was checked
rather than assumed. B15's F-466 established that `validate_incompatibilities`
reads **bought** selections on both sides, so a prohibition that must also catch a
*granted* Virtue needs `Prereq::Nor`. `virtue.true_faith` is grantable by nothing
here: no profile in `rules/core/character_types.json` carries
`granted_selections`, and no House grants it. A plain symmetric
`incompatible_with` is sufficient and is the right fix.

**Severity — MEDIUM.** The app builds a character the book forbids, and its own
shipped text says so.

**Kind of defect.** `incompat`.

---

### F-528 — a fully quantified Warping rule shipped `narrative`

**Entry.** `flaw.susceptibility_to_warping` (ArMDE:6831-6838), shipped
`narrative`.

**The passage, verbatim.**

> ArMDE:6835 — "In any year in which the character gains a Warping Point from any
> Realm or any source (including Longevity Rituals, living within an aura, and
> powerful supernatural effects), the character gains **one additional Warping
> Point** associated with that same Realm. This means that if he gains a single
> Warping Point from each Realm in a year, he gains **four additional Warping
> Points**, one from each Realm, at the end of that year."
>
> ArMDE:6837 — "These additional points do **not** contribute to Wizard's Twilight
> or to other events caused by the acquisition of Warping Points as they accrete
> slowly over the course of the year."
>
> DE 6835 / DE 6837 — "erhält der Charakter **einen zusätzlichen
> Verzerrungspunkt** … erhält er am Ende dieses Jahres **vier zusätzliche
> Verzerrungspunkte**" … "Diese zusätzlichen Punkte tragen **nicht** zum Zwielicht
> … bei".

**Current data.** `classification: "narrative"`, no effects, **no `description` in
either locale**, and a `summary` that carries only the first, purely colour
paragraph ("Your character's constant exposure to magic has made him more
susceptible to supernatural Warping than he might otherwise be").

**Why it is wrong.** `narrative` asserts the book states nothing mechanical. The
book states a multiplier (one extra point per point, per Realm, per year), a
worked arithmetic example (1 + 1 + 1 + 1 → four extra), a timing rule (end of
year) and an explicit exclusion (they do not count toward Twilight). **The
player reads none of it**: the second and third paragraphs reach neither locale,
so the rule that makes this Flaw worth a point is invisible in the app.

**Why the screen missed it.** No signed number, no botch term, and none of the
**33** `MECHANICAL_PHRASES` (corrected from **20** by the verification pass, which
counted the list) — "gains one additional Warping Point" has no sign character,
and § 3.1's proposed families do not cover an additive counter written in words.
The German is the same shape. See the new-form table in the Method.

**Correct value.** `"classification": "uncomputed_rule"`, with ArMDE:6835 and
:6837 written into `description` in both locales, plus a
`source.anchor: "susceptibility-to-warping"` (ArMDE:5558).

**Whether the engine could compute it.** It could not, and that was checked
against Part A rather than assumed: `WarpingGrant` (A22) grants a *starting*
Warping score at creation, and `corrections.md` § 3.15 records F-500 —
"`WarpingGrant.score` is dead data with no integrity check". There is no
per-year Warping accrual model at all, so D3 applies and `uncomputed_rule` is the
honest class.

**Severity — MEDIUM.** A lost rule, and a total loss: this entry's only mechanic
reaches the user nowhere.

**Kind of defect.** `class` + `desc`.

---

### F-529 — a Ritual-spell prohibition shipped `narrative`

**Entry.** `flaw.unstructured_caster` (ArMDE:6947-6950), shipped `narrative`.

**The passage, verbatim.**

> ArMDE:6949 — "You cast **all** Formulaic spells as though they were Ritual
> spells (including the need for vis), and you **may not learn Ritual spells at
> all**. You cast Spontaneous spells normally."
>
> DE 6949 — "Du wirkst alle Formulaischen Zauber so, als wären sie Ritualzauber
> (einschließlich des Vis-Bedarfs), und du kannst **überhaupt keine Ritualzauber
> erlernen**. Spontane Zauber wirkst du normal."

**Current data.** `classification: "narrative"`, no effects, no description.

**What happens when a player does what the entry says.** He takes Unstructured
Caster, then opens the spell picker and buys a Ritual — `The Bountiful Feast`,
`Aegis of the Hearth`, any of them. `spell.rs::SpellDef::ritual` exists and is
read (B17 confirmed it while rating `flaw.rigid_magic`), so the engine **knows
which spells are Rituals**; nothing consults this Flaw. The sheet prints a magus
holding Rituals the Flaw forbids him to have learned.

**Why it is wrong.** `narrative` on a passage containing a blanket casting
conversion and an absolute prohibition. And the prohibition is a **creation-time**
one — "may not *learn*" — so it belongs to the phase the app does model, unlike
the casting conversion, which is in-play.

**Correct value.** `"classification": "uncomputed_rule"` with both clauses written
into `description` in both locales, plus the anchor `unstructured-caster`
(ArMDE:5304).

**Whether the engine could compute the prohibition.** Partly — and this is worth
stating precisely rather than waving at. `SpellDef::ritual` is a readable flag, so
a validator refusing a Ritual spell selection when this Flaw is held is
constructible. What is **not** constructible is the Formulaic→Ritual casting
conversion, which needs a vis-cost model the engine does not have. So this is a
**mixed** case: `uncomputed_rule` is right for the entry, but the Ritual-learning
half is an F-494-shaped opportunity and should be recorded as such for Phase 2
rather than lost inside a classification change. There is no `Effect` variant that
forbids a spell, so it would be a new validator, not a data row.

**Severity — MEDIUM.** A lost rule plus a buildable-but-illegal character. Rated
below F-524 because no number is computed wrongly.

**Kind of defect.** `class` + `desc`, with an `engine` note.

---

### F-530 — two absolutes about Creo shipped `narrative`

**Entry.** `flaw.unnatural_magic` (ArMDE:6931-6934), shipped `narrative`.

**The passage, verbatim.**

> ArMDE:6933 — "Because of the unreal and illusory nature of your magic, **none of
> the character's Creo rituals have a permanent effect**. Wounds magically closed
> with The Chirurgeon's Healing Touch (CrCo20), for example, reopen again as soon
> as he finishes casting the spell, as the magic truly lasts only a moment. **He
> also cannot extract vis from an aura using Creo**, since his version of that Art
> is too unstable for the vis to remain in a lasting physical form."
>
> DE 6933 — "haben **keine** seiner Creo-Rituale einen dauerhaften Effekt" …
> "Er **kann auch keine** Vis aus einer Aura mit Creo gewinnen".

**Current data.** `classification: "narrative"`, no effects, no description. The
`summary` carries the first clause only; the vis-extraction prohibition reaches
neither locale.

**Why it is wrong.** Two absolutes about a named Art, one of which (vis
extraction) is a distinct lab activity. `narrative` claims the book states nothing
mechanical; it states two prohibitions and a worked example naming a specific
spell and its level.

**Correct value.** `"classification": "uncomputed_rule"` with both clauses in
`description` in both locales, plus the anchor `unnatural-magic` (ArMDE:5303).

**Whether the engine could compute either.** **Neither.** There is no ritual
permanence model and no vis-extraction model — `derived/lab.rs` computes Lab
Totals, not lab *activities*. `DeficientArt` (A32) is the nearest variant and is
the wrong one: it halves totals for an Art, which is not what this says. D3's case
exactly.

**Severity — MEDIUM.** A lost rule, half of it reaching the user nowhere at all.

**Kind of defect.** `class` + `desc`.

---

### F-531 — Suppressed Gift's four mechanical clauses shipped `narrative`

**Entry.** `flaw.suppressed_gift` (ArMDE:6803-6810), shipped `narrative`.

**The passage, verbatim.**

> ArMDE:6805 — "While the Gift is suppressed, **the character cannot perform
> Hermetic magic, improve his Arts, or perform the Parma Magica. His Arts do
> provide him with Magic Resistance and he continues to suffer the negative social
> penalties of The Gift.** Such a character may still use Supernatural Virtues and
> Abilities. He may be a member of the Order of Hermes, depending on when his Gift
> was suppressed."
>
> DE 6805 — "kann der Charakter **keine** hermetische Magie wirken, seine Künste
> verbessern oder die Parma Magica praktizieren. Seine Künste verleihen ihm jedoch
> **weiterhin Magieresistenz**, und er leidet **weiterhin** unter den negativen
> sozialen Nachteilen der Gabe."

**Current data.** `classification: "narrative"`, `prerequisites:
{"kind":"has","value":"virtue.the_gift"}` (**correct**, and one of only two
correct prereqs in the span), `categories: ["hermetic","story"]` (**correct**,
matching both index lists at :5301 and :5369), no effects, no description.

**Why it is wrong.** Four separate mechanical clauses, two of them prohibitions on
things the app models and two of them explicit *retentions*:

| Clause | What it touches | Engine |
|---|---|---|
| cannot perform Hermetic magic | casting | in-play, not modelled per-character |
| **cannot improve his Arts** | **Art XP at creation and in play** | the app sells Art scores from the shared `xp_pool` and nothing stops it |
| cannot perform the Parma Magica | the Parma Ability | `derived/casting.rs::magic_resistance` reads Parma directly |
| his Arts **do** provide Magic Resistance | the resistance grid | computed, and correctly — the Flaw does not change it |
| still suffers the Gift's social penalties | the Gift's −3 | computed, and correctly |

The second clause is the one with teeth at character creation: a magus with
Suppressed Gift may still spend his whole `xp_pool` on Arts, which is exactly
what the Flaw forbids.

**Correct value.** `"classification": "uncomputed_rule"` with ArMDE:6805 written
into `description` in both locales, plus the anchor `suppressed-gift`
(ArMDE:5301/:5369).

**Why not a computed class.** Because none of the four is expressible. There is no
"cannot spend XP on Arts" variant (the nearest, `RestrictedAbilityXp` (A7), is
Ability-scoped, not Art-scoped), no per-character casting suppression, and no way
to disable Parma while keeping the Form bonus. D3 applies.

**Severity — MEDIUM.** A lost rule, and the Art-XP half is a real
creation-time hole.

**Kind of defect.** `class` + `desc`.

---

### F-532 — "only applicable to magi" with `prerequisites: null`

**Entry.** `flaw.tormenting_master` (ArMDE:6851-6854), shipped `narrative`.

**The passage, verbatim.**

> ArMDE:6853 — "**This Flaw is only applicable to magi**, although other
> characters could take an analogous Story Flaw."
>
> DE 6853 — "**Dieser Fehler ist nur für Magi anwendbar**, obwohl andere Charaktere
> einen analogen Geschichte-Fehler nehmen könnten."

**Current data.** `categories: ["story"]`, `classification: "narrative"`,
**`prerequisites` absent**.

**What happens when a player does what the entry says — SCOPE CORRECTED by the
verification pass.** A **companion or a mythic companion** takes Tormenting Master
and the app builds it. The primary read said *"a grog, a companion or a mythic
companion … Every entity kind can take this Flaw today"* and that is **wrong**:
the category gate *does* half of the job here. `rules/core/character_types.json`
gives the **grog** profile
`permitted_categories: ["general","personality","social_status","supernatural"]`
— **`story` is not in it** — and
`validation/selections.rs::validate_permitted_categories` (read in full, `:56-129`)
raises a hard `ValidationIssue::error` with `CODE_CATEGORY_NOT_PERMITTED` for any
selection none of whose in-force categories is permitted. So a grog is already
refused this Flaw, and refused it as an **error**, not a warning.

**The defect is real but covers two of the four profiles, not four.** `companion`
and `mythic_companion` both list `story` among their permitted categories and
both have `is_magus: false`, so each can hold a Flaw the book confines to magi.
That is still the app building a character the book forbids — the finding stands,
at the same severity — but the "every entity kind" claim does not, and the same
correction applies to **F-533** (`flaw.vendetta`, also `story`) and to the
reachability sentence in **F-526** (`flaw.university_dean`, also `story`).

**Why it is wrong.** This is F-518's shape with a weaker verb but the same
content: an eligibility gate the `Prereq` enum expresses natively.

**Correct value.** `"prerequisites": {"kind":"is_magus"}`. `Prereq::IsMagus`
resolves against `EntityTypeProfile::is_magus`, which
`PrereqCtx::build` (`validation/prereq.rs:107`) reads directly from the profile,
so the gate is exact and needs no new machinery.

**The classification stays `narrative`** and that is deliberate: the rest of the
passage ("He periodically troubles you with political moves and indirect attacks")
is pure story, and an eligibility gate is a `prerequisites` defect rather than a
claim that the book states a computable mechanic. This is the distinction B17 drew
for F-518, whose classification moved only because that entry's *other* sentence
(the 15 casting tools) was mechanical. Here there is no other sentence.

**Severity — MEDIUM.** The app builds a character the book forbids.

**Kind of defect.** `prereq`.

---

### F-533 — Vendetta's House restriction, and the word "generally"

**Entry.** `flaw.vendetta` (ArMDE:6955-6958), shipped `narrative`.

**The passage, verbatim.**

> ArMDE:6957 — "The **magus** is engaged in one of House Verditius' vendettas,
> mostly likely carrying on his parens's vendetta against another Verditius magus.
> … **This Flaw is generally restricted to magi of House Verditius**, as the
> custom of vendetta is limited to that House. Other characters should take Enemy
> or Feud to represent a similar situation."
>
> DE 6957 — "**Dieser Fehler ist im Allgemeinen auf Magi des Hauses Verditius
> beschränkt**, da der Brauch der Vendetta auf dieses Haus beschränkt ist. Andere
> Charaktere sollten Feinde oder Fehde nehmen, um eine ähnliche Situation
> darzustellen."

**Current data.** `categories: ["story"]`, `classification: "narrative"`,
**`prerequisites` absent**.

**What happens when a player does what the entry says — SCOPE CORRECTED by the
verification pass, exactly as in F-532.** The entry is `story`, and `story` is
**not** among the grog profile's `permitted_categories`, so
`validate_permitted_categories` already refuses a grog this Flaw with a hard
error. The original sentence — *"any grog of any covenant may take a House
Verditius vendetta"* — is **wrong**. What is true is that a **companion or a
mythic companion**, both `is_magus: false`, may take a House Verditius vendetta,
and the Flaw's own first words are "The magus … his parens". The finding stands at
the same severity over a smaller, still-real population.

**Why it is wrong, and where the doubt is.** The restriction is stated twice —
once implicitly (every sentence presupposes a magus and a parens) and once
explicitly. `Prereq` expresses it exactly:
`All[IsMagus, House("house.verditius")]`. But the book hedges with **"generally"**,
and hedged restrictions are precisely what this audit must not quietly harden:
B17's F-518 had "*can only be taken by*", an unhedged absolute, and was rated as
one.

**Correct value — two candidates, and the choice is Q-139.**

- **(a)** `"prerequisites": {"kind":"all","value":[{"kind":"is_magus"},
  {"kind":"house","value":"house.verditius"}]}` — treating "generally restricted"
  as a rule with a storyguide override, which is what `ValidationMode::Advisory`
  exists for.
- **(b)** `"prerequisites": {"kind":"is_magus"}` only, treating the House half as
  the soft part and the magus half as hard (the passage's "The magus … his parens"
  is not hedged at all).

**Option (b) is the minimum that is certainly right**, and I recommend it land
regardless of how Q-139 resolves: nothing in the passage permits a non-magus, and
"Other characters should take Enemy or Feud" says so directly.

**Severity — MEDIUM** for the magus half (the app builds a character the book
excludes), **escalated** for the House half.

**Kind of defect.** `prereq`.

---

### F-534 — Usurer's income reaches neither locale

**Entry.** `flaw.usurer` (ArMDE:6951-6954), shipped `creation_effect`.

**The passage, verbatim.**

> ArMDE:6953 — "**You receive the equivalent of approximately ten pounds of silver
> each year from interest payments**, though you may occasionally need to chase
> down debtors, and you have a **poor** reputation (Usurer) at level 4 within your
> community and the local region."
>
> DE 6953 — "Du erhältst jährlich das Äquivalent von ungefähr **zehn Pfund Silber**
> aus Zinszahlungen … und du hast eine **schlechte** Reputation (Wucherer) auf
> **Stufe 4** innerhalb deiner Gemeinschaft und der lokalen Region."

**Current data.** `classification: "creation_effect"` with
`{"type":"grants_reputation","kind":"local","score":4}`, and **no `description` in
either locale**. The `summary` is the passage's first sentence only ("You often
lend silver or other valuables to people at interest rates that many consider
abusive"), which contains neither the income nor the Reputation.

**What the player actually gets today.** A Reputation *slot* of kind `local` he
may fill at any level he likes (A25: `score` is not enforced), pre-filled at 4 by
the UI. He is told nothing about the ten pounds of silver, and nothing about the
Reputation being a **bad** one.

**Why it is wrong — D5, in its plainest form.** D5 obliges every mechanical
clause the engine does not compute to be written into `description` in both
locales, *whatever the classification*. The income clause is a hard number
("approximately ten pounds of silver each year") that the engine models nowhere —
there is no income or wealth variant among the 42 `Effect`s — and it reaches the
user in neither locale. This is the B17 F-515 shape: a computed entry that
expresses one clause and silently drops the other.

**Correct value.** Add `description` in `en` and `de` carrying ArMDE:6953 in full.
`classification` stays `creation_effect` — D5's closing paragraph: "the obligation
moves; the taxonomy does not" — because the Reputation grant genuinely is a
creation-time computation.

**The polarity is a separate, known gap and is not re-raised.** "a **poor**
reputation" cannot be expressed: A25's `GrantsReputation` has `kind` and `score`
and nothing else, which is **Q-43** and `corrections.md` § 3.15's F-363 row. This
finding does not re-derive it; the description text is what carries "poor" to the
player until Q-43 is settled.

**Severity — LOW-MEDIUM.** A dropped clause, and the one that is dropped is a
saga-facing number rather than a character-sheet one.

**Kind of defect.** `desc`.

---

### F-535 — Tragic Life mandates a Personality Trait (marginal, and rated as such)

**Entry.** `flaw.tragic_life` (ArMDE:6855-6870), shipped `narrative`.

**The passage, verbatim.**

> ArMDE:6859 — "**The predisposition toward sin at the character's pivotal moment
> should be represented with a sinful Personality Trait.**"
>
> DE 6859 — "**Die Neigung zur Sünde am Wendepunkt des Charakters sollte durch
> einen sündhaften Persönlichkeitszug dargestellt werden.**"

**Current data.** `classification: "narrative"`, `tainted: true`, no effects, no
description.

**Why this may be a defect.** A mandated Personality Trait is a numbered row on
the character sheet, and the catalogue already treats that as mechanical: the
neighbouring `flaw.uncertain_faith` states *"has an 'Uncertain Faith' Personality
Trait at +1"* and is `uncomputed_rule`, and `corrections.md` § 3.1 family 14 lists
`an additional Personality Trait of` as a missing mechanical word-form sourced
from F-467 and F-445. Tragic Life is a **Story** Flaw, so the trait it mandates is
*not* the one ArMDE:2502 already requires of Personality Flaws — it is an extra
one this entry alone imposes.

**Why it is marginal, stated rather than hidden.** Two reasons, both real:

1. The verb is **"should be represented"**, not "has" or "must take". The
   `uncertain_faith` twin says "has", and gives a number.
2. **No number is given.** ArMDE:2502's general rule ("+3 or -3" for a Minor
   Personality Flaw, "+6 or -6" for a Major) is keyed to *Personality* Flaws and
   does not obviously reach a Story Flaw's extra trait, so the score is left open.

**Correct value, if it stands.** `"classification": "uncomputed_rule"` with
ArMDE:6859 in `description` in both locales, plus an anchor `tragic-life`
(ArMDE:5372). No effect is owed: Personality Traits are free-text entity rows with
no `Effect` variant granting one.

**Severity — LOW.** A lost sentence in a long, otherwise purely narrative
passage. **This is the weakest finding in the batch and is flagged for the
verification pass specifically.** If it is withdrawn, the entry is clean.

**Kind of defect.** `class`.

---

### F-536 — Surgical Empiricus is a Western Christendom Flaw and nothing says so

**Entry.** `flaw.surgical_empiricus` (ArMDE:6811-6814), shipped `uncomputed_rule`.

**The passage, verbatim — and it is not the entry's own.**

> ArMDE:2890 — "Because Social Status Virtues and Flaws refer to a character's
> place in society, **the available options vary depending on the society they
> live in.**"
>
> ArMDE:2902-2909 — "#### Western Christendom … | **Minor Flaws** | Failed
> Journeyman, Failed Master, Failed Monk/Nun, **Surgical Empiricus** |"

Surgical Empiricus appears in the Western Christendom table and in **no other
culture table** — not in "All Cultures" (:2892-2900), not Eastern Christendom
(:2911-), and not the later ones.

**Current data.** `classification: "uncomputed_rule"`, `categories:
["social_status"]`, description present and faithful in both locales, carrying the
entry's own rules (the +1 specialization bonus and the halved Chirurgy score) —
**and nothing about culture**.

**Why it is wrong.** It is an eligibility restriction the book states, reaching
the player in neither locale and enforced nowhere. Per D5 every mechanical clause
the engine does not compute must be written into `description` whatever the
classification, and per D3 an engine that structurally cannot express a rule keeps
`uncomputed_rule` **with the rule written out** — it never gets to stay silent.
The app models no culture at all (there is no culture field on `Entity` and no
culture data file), so writing it out is the whole of the available fix.

**Why this is not already reported, checked rather than asserted.** B16 read the
same table family — `batch-16.md:538` rates ArMDE:2899-2900 for its five Social
Status Flaws and concludes "**No new rule**". That conclusion is **correct for
those five**, because every one of them sits in the **All Cultures** list, which
restricts nothing. This span is the first to reach an entry in a *culture-specific*
row. `corrections.md` carries no finding citing :2890, :2896, :2899 or :2909
(grepped), and `batch-17.md` carries none either.

**Correct value.** Extend the `description` in both locales with the restriction,
sourced to ArMDE:2890/:2909 — e.g. EN "This Social Status is available only in
Western Christendom", DE "Dieser Soziale Status ist nur im westlichen Christentum
verfügbar" (the DE source's own heading at DE 2902 supplies the term).

**Scope, stated no wider than checked.** I checked the culture tables for **my
two** Social Status entries only: `flaw.usurer` is in All Cultures (no
restriction, confirmed) and `flaw.surgical_empiricus` is Western-Christendom-only.
**The culture tables at ArMDE:2888-2930 cover the whole Social Status catalogue**,
so there are very likely many more instances, and the right Phase 2 unit is one
sweep of those tables against every `categories: ["social_status"]` entry — not 60
per-entry findings. **That sweep is a lead recorded here, not a claim this batch
has made.** It is also entangled with **F-427 / Q-07**, the one-Social-Status rule
that is enforced nowhere, and the two should be worked together.

**Severity — LOW-MEDIUM.** A missing eligibility rule with no computation behind
it, on an entry whose other rules are all correctly written out.

**Kind of defect.** `desc`, with a `prov` note (the restriction's source line is
outside the entry's own `source.lines`, which is why the phrase screen — which
reads only the cited range — is structurally blind to it; `corrections.md`
§ 3.1's blind spot #2).

---

### F-537 — a German needle already in `MECHANICAL_PHRASES` matches nothing the German book writes

**File.** `crates/arm-rules/tests/uncomputed_clauses.rs`, `MECHANICAL_PHRASES`.

**The needle.** The list carries the pair

```
"may not be greater than",
"darf nicht größer sein als",
```

**What the German core book actually writes.** The idiom's German word order puts
the copula last. `flaw.uninspirational`'s own text, in the source and in the
shipped `description`:

> DE 6921 — "Sein Präsenz- und Kommunikationswert **darf nicht größer als 0
> sein**."

`has_mechanical_phrase` matches a **contiguous substring** with a left word
boundary (`uncomputed_clauses.rs:234-241`). `darf nicht größer sein als` is not a
substring of `darf nicht größer als 0 sein` — the needle has `sein als` where the
text has `als 0 sein`. **The needle does not match, and the entry is screened
green only by the unrelated `-3-Abzug` further down the same paragraph.**

**Why this is a defect and not a curiosity.** It is the same class as B17's
`halbiert`-versus-`halbier` finding: a proposed or shipped German form that does
not match the German the books write, so the German half of the screen silently
does less work than the English half. Here it is worse, because the form is
**already shipped** rather than proposed — the list has carried a dead needle
since it was written, and its English twin (`may not be greater than`) does match
ArMDE:6921's English.

**Counted, not asserted — and the unit is occurrences, not lines.** The form was
found by reading DE 6921; it was then counted with `rg -a -o … | wc -l`, which
counts **occurrences**, over the German core book and over the shipped German
i18n:

| Needle | German core book | `rules/i18n/de/*.json` |
|---|---|---|
| `darf nicht größer sein als` (the shipped needle) | **0** | **0** |
| `darf nicht größer als` (what the book writes) | **2** (DE 4970, DE 6921) | **1** (`flaw.uninspirational`'s description) |
| `nicht größer` (the widest form) | **3** occurrences on **3** lines | — |
| EN twin `may not be greater than` | **1** occurrence — ArMDE:6921, **this entry** | 1 |

So the German needle matches **nothing anywhere**: not one line of the German
rulebook and not one shipped German string. Its English twin has exactly one hit
in the whole book, on this same entry. The verification sub-agent was asked to
re-take both counts with a third method.

**Correct value.** Replace the needle with a form the book writes. The safe
minimum is `darf nicht größer` (a left-anchored prefix that matches both word
orders and over-matches only into the same idiom); the thorough fix is the
two-token proximity match the German discontinuous-negation problem needs anyway
(see the Method's German-prohibition subsection).

**Severity — LOW.** A test-only defect on a guard that is already known to be
partial, and no shipped entry currently depends on it. Rated as a finding rather
than a note because `corrections.md` § 3.1 is about to *grow* this list, and
adding forms beside a dead one repeats the mistake.

**Kind of defect.** `test`.

---

### F-538 — "may be taken as a Hermetic Flaw" is unrepresentable, though the mechanism exists

**Entry.** `flaw.vengeful_powers` (ArMDE:6959-6976), shipped `uncomputed_rule`.

**The passage, verbatim.**

> ArMDE:6975 — "**Vengeful Powers may be taken as a Hermetic Flaw**, as applicable
> to Hermetic magic. In that case, the effect triggers whenever the character
> performs magic that is without sin. It is more commonly associated with
> Supernatural Abilities, however."
>
> DE 6975 — "**Rachsüchtige Mächte können als Hermetischer Fehler genommen
> werden**, soweit sie auf Hermetische Magie anwendbar sind."

**Current data.** `categories: ["story"]`, `tainted: true`, `classification:
"uncomputed_rule"`, description present in both locales **and carrying this very
sentence**, no `index_categories`, no `parameters`.

**Why it matters, concretely.** `validation/magus.rs::validate_house` enforces
ArMDE:2860's "at least one Hermetic Flaw" guideline, and per
`engine-semantics.md` B5 it is the **only** consumer of `index_categories`. A
magus who takes Vengeful Powers *as a Hermetic Flaw* — which the book expressly
permits — has satisfied that requirement, and the app does not know it. He is told
he still owes a Hermetic Flaw.

**Why the obvious fix is wrong.** Adding `index_categories: ["hermetic"]` would be
a misuse of the field: B5 defines it as "headings the book's own *index* files the
entry under", and the book's index files Vengeful Powers under **Story Major**
only (ArMDE:5379). It is not in the Hermetic Major list at :5285-5304. The
permission is in the prose, not the index.

**The mechanism the catalogue already has.** `ParameterDomain::Category` — a
`taken_as` parameter recording which single reading the player chose — is used by
exactly two entries, `virtue.sufi` (Social Status *or* Supernatural, ArMDE:5083)
and `flaw.curse_of_slander` (General *or* Supernatural). Per B5, five membership
sites resolve a multi-category item through
`types.rs::PointItem::categories_for`, which consults that parameter. That is
exactly this shape: one entry, two mutually exclusive category readings, the
player picks one.

**Correct value.** `"categories": ["story","hermetic"]` plus
`"parameters": [{"key":"taken_as","type":"ref","domain":"category",
"values":["story","hermetic"]}]`, following `virtue.sufi` character for
character. `classification` stays `uncomputed_rule`.

**A reachability caveat the verification pass adds, from reading `validate_house`
itself rather than B5's summary of it.** The remedy *does* fire — but it fires
**unconditionally**, which is not what the passage says.
`validation/magus.rs::validate_house`'s `counts_as_hermetic` closure tests
`item.any_category_in(&profile.hermetic_flaw_categories)` **or**
`item.index_categories`, and its own comment states the choice deliberately:
*"Membership, not the taken-as-narrowed `categories_for`: an index heading is
provenance … so the player's presentation choice must not remove a Flaw from the
book's own Hermetic list."* So adding `hermetic` to `categories` would credit a
magus with a Hermetic Flaw **even when he took Vengeful Powers as Story** — the
book's own stated default (*"more commonly associated with Supernatural
Abilities"*). That is an error in the opposite direction from the one this
finding reports, and it means the fix is **not** the pure `virtue.sufi` copy the
finding describes: either `validate_house` must learn to respect `taken_as` for
this case, or the remedy must be something else. (The warning it raises is
`CODE_MISSING_HERMETIC_FLAW`, a `ValidationIssue::warning`, not an error — so
today's cost to the player is an incorrect nag, not a blocked build.) **This
does not withdraw the finding; it shows the proposed remedy is under-specified.**

**The doubt, stated rather than buried.** `virtue.sufi`'s and
`flaw.curse_of_slander`'s passages offer the two readings in the **descriptor
line** itself; this one offers them in the body, and the descriptor says only
`*Major, Story, Tainted*`. So the two are not perfectly analogous, and a reviewer
could reasonably hold that the descriptor is decisive and the prose sentence is
advice to the storyguide. **I report it as a finding rather than a question
because the consequence is concrete and one-directional** — a magus is denied
credit for a Hermetic Flaw the book says he may take — but the verification pass
was asked to challenge it specifically.

**Severity — LOW-MEDIUM.** Not a wrong number; a legal build the app refuses to
recognise.

**Kind of defect.** `class`-adjacent / `param`.

---

## Open questions

### Q-136 — does D3's ruling extend to `flaw.susceptibility_to_divine_power`?

**Who settles it.** N (a policy call about how far a named ruling generalises).

**The facts.** `flaw.susceptibility_to_divine_power` (ArMDE:6815-6818) ships
`in_play_effect` with `{"type":"special_casting_mod","kind":"doubled_aura_penalty"}`.
Per `engine-semantics.md` A40, `doubled_aura_penalty` is one of **eight
surfaced-only kinds**, "pushed with `amount: 0`", and the variant "carries no
amount at all, so even a surfaced row has no magnitude — the *kind* slug is the
whole payload". The entry therefore computes exactly as much as its two Faerie and
Infernal siblings: nothing.

**Why it is not simply F-523's third entry.** D3 names Faerie and Infernal only,
and for a *specific* reason — the per-Form resistance grid has no realm axis. The
Divine entry's rule is different in kind ("suffer twice the normal penalties …
such as spellcasting modifiers and botch dice"), so D3 does not literally reach
it.

**Why it is not simply clean either.** B1 makes `classification` a claim about the
rulebook, and `in_play_effect` means "the book states a rule the engine computes
during play". The engine computes nothing — and **`RULES.md` says so in its own
words**, at `:4882-4888`, defending the modelling choice at length:

> "it is **surfaced, not simulated**, because the engine models neither an aura of
> a foreign realm nor botch dice, and the rule carries no number of its own — it
> doubles whatever the scene's aura rating happens to be. … **The value the engine
> cannot compute lives in the Flaw's own rules text in `rules/i18n/<lang>/`.**"

That is a description of an `uncomputed_rule`, written under an `in_play_effect`
label. `RULES.md:4870` also records that the entry was deliberately moved **off**
`magic_resistance_mod` in the round-5 audit because ArMDE:6817 never mentions
Magic Resistance — so the *effect variant* is right and well-reasoned, and only
the class is in question. `RULES.md:4728-4731` separately records why it carries
no `description`: its `summary` already is the whole passage.

**Why it is escalated rather than decided.** There is a **precedent pointing the
other way and it is one batch old**: B17 rated `flaw.short_ranged_magic`, which
carries the surfaced-only `special_casting_mod{circumstantial}`, and **left it
`in_play_effect`**, filing F-515 as a D5-description finding only. Deciding the
Divine entry either way silently overturns or silently entrenches that precedent
for eight `special_casting_mod` kinds and four `magic_resistance_mod` kinds across
the catalogue. **The entry is NOT marked checked.**

**What settles it.** One ruling: *does a surfaced-only effect kind — one that
reaches the read-out as a slug with `amount: 0` and moves no number — satisfy
`in_play_effect`, or is `uncomputed_rule` the honest class?* The answer governs
**19 catalogue entries** and should be taken once.

**The blast radius was understated, and the verification pass corrects it.** This
question first read *"at least twelve catalogue entries"* — but twelve is the
count of **effect kinds** (eight surfaced-only `SpecialCasting` variants + four
surfaced-only `MagicResistanceEffect` variants), not of entries. Both arm lists
were re-read straight out of `derived.rs::in_play_mods`'s `match` rather than out
of A37/A40, and the entries carrying them were counted with `jq`: **19** —
`flaw.corrupted_spells`, `flaw.deleterious_circumstances`,
`flaw.disjointed_magic`, `flaw.environmental_magic_condition`,
`flaw.short_ranged_magic`, `flaw.susceptibility_to_divine_power`,
`flaw.susceptibility_to_faerie_power`, `flaw.susceptibility_to_infernal_power`,
`flaw.the_constant_expression`, `flaw.weak_magic_resistance`,
`virtue.commanding_aura`, `virtue.diedne_magic`, `virtue.faerie_raised_magic`,
`virtue.leper_magus`, `virtue.life_boost`,
`virtue.life_linked_spontaneous_magic`, `virtue.mercurian_magic`,
`virtue.special_circumstances`, `virtue.spell_improvisation`. The ruling is
therefore **more** consequential than the question claimed, not less.

---

### Q-137 — `incompatible_with` holds ids; ArMDE:6925 states a predicate

**Who settles it.** N (a modelling call) — see F-526.

The book forbids University Dean to a character with "**any other Flaw that grants
a Bad Reputation**". That is a predicate over the catalogue, and the catalogue
answers it: sixteen Flaws carry `grants_reputation`. But `incompatible_with` holds
a literal id list, so encoding the rule means freezing a derived set into data,
which goes stale the moment a seventeenth Reputation-granting Flaw is added — and
`CLAUDE.md`'s catalogue-size-is-data invariant says the executable and the rules
must stay separable.

**The options.** (a) Ship the sixteen ids plus `flaw.poor`, and add a test
asserting the list equals the derived set — cheap, and the staleness becomes a red
rather than a silent wrong. (b) Add a predicate-valued exclusion (an
`incompatible_with_any_granting` shape, or a `Prereq::Nor` over a derived group) —
correct, and new machinery. **Note that (a) and (b) are not equivalent under
`CLAUDE.md`**: (a) encodes a *number* of entries into data, which the invariant
tolerates only because the test regenerates it.

`virtue.doctor_in_faculty` as a prerequisite is **not** blocked on this and should
land either way.

---

### Q-138 — ArMDE:6148 forbids a *class* of Flaws, and it is on another batch's entry

**Who settles it.** R then N.

ArMDE:6148 (inside B13's span, 6068-6235) reads: "Any Flaw that is only
appropriate to Hermetic Magic (for example, Deficient Technique or **Unstructured
Caster**) cannot be taken with this Flaw." I found it in my inbound sweep for
`flaw.unstructured_caster`; **I checked `corrections.md` § 1's B13 block and
`batch-17.md` and neither carries a finding for it.**

It is not numbered here for two reasons. First, the defect belongs to the
ArMDE:6145-6148 entry, not to mine — the prohibition is stated from that side, and
naming it on `flaw.unstructured_caster` would be the cross-span duplicate shape
§ 4.2 warns about. Second, the fix is undecided in the same way as Q-137: the
prohibition names a **class** ("any Flaw that is only appropriate to Hermetic
Magic"), which `incompatible_with` cannot express and which — unlike Q-137's
Reputation predicate — the data does not cleanly enumerate, because
`categories: ["hermetic"]` includes Flaws that are not *only* Hermetic.

**Recorded so B13's entry is not left silently unrated.**

---

### Q-139 — does "generally restricted" warrant a hard prerequisite?

**Who settles it.** N. See F-533.

ArMDE:6957 says Vendetta "is **generally** restricted to magi of House Verditius".
B17's F-518 encoded an *unhedged* "can only be taken by Verditius magi" as a hard
`Prereq`. Encoding a hedged one the same way turns storyguide latitude into a hard
error under `ValidationMode::Enforced`; not encoding it leaves any grog able to
inherit a Verditius parens's vendetta.

**The narrower question, which may make this moot:** the magus half is **not**
hedged ("The magus is engaged…", "his parens", "Other characters should take Enemy
or Feud"). `Prereq::IsMagus` alone is defensible with no ruling. Q-139 is only
about whether `House("house.verditius")` joins it.

**Q-13 is adjacent and unsettled** ("does the book distinguish 'may only **take**'
from 'may only **have**'?"), and a ruling on hedged-versus-absolute eligibility
language would settle both.

---

## Sub-agent reconciliation

**Run 2026-09-21, by the verification pass.** The primary agent was killed by a
session rate limit mid-sentence, its last words being *"My own recount corrects
two of my numbers. Updating the file."* — it never said which, and its own
verification sub-agent read nothing before dying with it. So **no independent
check of this batch existed** and every census figure and every verdict was
re-derived here from scratch, from primary sources, with a different tool or
command shape wherever the Method named one.

**Method of this pass.** Verdicts were re-derived, not reviewed: for each entry I
read ArMDE:6795-6994 as continuous prose, pulled the shipped mechanics with `jq`
over `rules/core/virtues_flaws.json` (selecting on `source.lines`, not on an
index, so the "canonical ordering" claim was itself re-derived), pulled both
locales with `jq` over the `rules/i18n/{en,de}/virtues_flaws.json` **objects**,
and read every consumer function in the engine rather than `engine-semantics.md`'s
summary of it. Then I asked the brief's question of every entry — *what happens
when a player actually does what this says?* — before comparing with the file.

### Census figures — all re-derived, four corrected

Because the primary agent did not say which two of its numbers were wrong, every
figure in `## Method` was re-taken. Where the Method names a tool, a different one
was used.

| Figure | Method said | Re-derived | Tool used here | Verdict |
|---|---|---|---|---|
| entries in span | 35 | **35** | `jq '[.[] \| select(.source.lines[0]>=6803 and <=6988)] \| length'` | ✅ |
| canonical indices 595–629 | 595–629 | **595–629** | `jq 'sort_by(.source.file,.source.lines[0]) \| to_entries'` — the ordering is by *source file + start line*, not file order (id-sorted) and not a bare index | ✅ confirmed, and the ordering key is now recorded |
| `####` headings in 6803-6988 | 34 | **34** | enumerated by hand from a single Read of the source, not `grep -n` | ✅ |
| `source.anchor` present | **10** | **11** | `jq '[.[]\|select(.source.anchor)]\|length'` | ❌ **CORRECTED** — and the Method's own sentence already contradicted itself, listing eleven ids then calling them ten. Nine `uncomputed_rule` + the two True Love `narrative` entries = 11. |
| entries carrying `effects` | **6** | **5** | `jq '[.[]\|select(.effects)]\|length'` → `divine`, `faerie`, `infernal`, `unimaginative_learner`, `usurer` | ❌ **CORRECTED** — check 11's table always listed exactly five rows, so the prose figure was the outlier. |
| classifications | 18 / 12 / 4 / 1 | **18 narrative, 12 uncomputed_rule, 4 in_play_effect, 1 creation_effect** | `jq 'group_by(.classification)'` | ✅ |
| `tainted: true` | 3 | **3** | `jq '[.[]\|select(.tainted==true)]\|length'` | ✅ |
| entries with `prerequisites` | 2 | **2** | `jq` | ✅ |
| entries with `incompatible_with` | 3 | **3** | `jq` | ✅ |
| entries with `parameters` | 1 | **1**, and `max_total: 1` is a **top-level** field on the entry, not a key inside the parameter object | `jq` full-record dump | ✅ (location clarified) |
| Flaws carrying `grants_reputation` | **17** (Method counting note) and "**seventeen** catalogue Flaws" (header claim 3) | **16 distinct Flaw ids**; **17 effect rows** (`flaw.failed_monk` carries two) | `jq '[.[]\|select(.kind=="flaw")\|select(.effects//[]\|map(.type=="grants_reputation")\|any)]\|length'` vs. a second `jq` flattening `.effects[]` | ❌ **CORRECTED, and it is exactly the unit trap the brief warned of** — 17 is the row count, 16 the entry count. F-526's body and Q-137 already said 16 correctly; only the header and the counting note carried the wrong unit. |
| inbound sites | **21** (counting note) vs **26** (inbound-sweep section) | **26** | the sweep section enumerates all 26 by line and its arithmetic (8+10+3+3+2) checks out; the counting note's 21 is the *withdrawn first draft's* figure, which that same section explicitly says was wrong | ❌ **CORRECTED** — a stale leftover the primary agent fixed in one place and not the other. |
| `index_categories` users catalogue-wide | 4 | **4** | `jq '[.[]\|select(.index_categories)]\|length'` | ✅ |
| `ParameterDomain::Category` users | 2 | **2** (`virtue.sufi`, `flaw.curse_of_slander`) | `jq` over `parameters[].domain` | ✅ |
| verdict arithmetic | **19 pass / 15 fail** / 1 escalated | **18 pass / 16 fail / 1 escalated** = 35 | counted row by row off the Verdicts table's `pass pass pass` cells, then cross-checked against the finding→entry map | ❌ **CORRECTED** |
| entries carrying a finding | **15 entries** + 1 test file | **16 entries** + 1 test file | finding→entry map: 15 findings land on entries, and F-523 covers two | ❌ **CORRECTED** |
| `MECHANICAL_PHRASES` size | **20** (stated twice, in F-524 and F-528) | **33** | counted item by item from `uncomputed_clauses.rs:179-219` | ❌ **CORRECTED** |
| entries governed by Q-136's ruling | **12** | **19** | `jq` over the eight `SpecialCasting` + four `MagicResistanceEffect` surfaced-only kinds read off `derived.rs::in_play_mods`'s match arms | ❌ **CORRECTED** — twelve is the *kind* count, not the entry count |
| `Effect` variants | 42 | **42** | counted the `pub enum Effect` variant heads | ✅ |
| `BOTCH_TERMS` | 5 (`botch`, `patzer`, `patzen`, `patzt`, `gepatzt`) | **5**, verbatim at `uncomputed_clauses.rs:84` | Read | ✅ |
| F-537's four needle counts | 0 / 2 / 3 / 1 | **0 / 2 / 3 / 1**, and the two DE hits are at DE **4970** and DE **6921** exactly as claimed | `rg --count-matches` (**occurrences**, not lines) instead of the Method's `rg -o \| wc -l`, plus a line-printing pass to check the line numbers | ✅ **all four confirmed by a different command shape** |
| findings / questions | F-523…F-538 (16), Q-136…Q-139 (4) | **16 / 4**, and the numbering is continuous: `corrections.md` tops out at **F-501** and **Q-129**, `batch-17.md` at **F-522** and **Q-135** | `rg -o` over both files, sorted | ✅ |

**Eight census figures were wrong, not two.** The primary agent's dying note said
"two"; it had most likely found the anchor count and the effects count, the two
that its own neighbouring prose already contradicts. The `grants_reputation` unit
error, the stale inbound `21`, the pass/fail split, the entries-carrying-a-finding
count, the `MECHANICAL_PHRASES` size and Q-136's entry count were **not** among
them and are found here. **Three of the eight are unit errors** — rows counted as
entries (`grants_reputation`), kinds counted as entries (Q-136) — which is the
trap the brief named, and every one of them survived because the recount would
have been taken the same way.

### Line parity — re-derived independently, and it holds

`rg -an "^#### "` over the **German** core book returns **34** headings in
6803-6988, at exactly the line numbers of the 34 English ones — 6803, 6811, 6815,
6819, 6823, 6827, 6831, 6839, 6843, 6847, 6851, 6855, 6871, 6879, 6883, 6887,
6891, 6899, 6907, 6911, 6915, 6919, 6923, 6927, 6931, 6935, 6939, 6943, 6947,
6951, 6955, 6959, 6977, 6985. Parity confirmed, not sampled. The 34-heading /
35-id gap is the shared `#### True Love` range, as stated.

### Verdicts — re-derived, not reviewed

**Nothing was overturned in the clean column, and that is a result, not an
omission.** All 18 clean verdicts were re-derived by asking what the engine does
when a player takes the entry, and the four with a live mechanical claim were
settled against the consumer function rather than the yardstick:

| Clean entry | The claim that had to hold | How it was settled here | Verdict |
|---|---|---|---|
| `flaw.uninspirational` | no characteristic cap exists, so "Presence and Communication may not be greater than 0" is uncomputable | `effective/characteristic.rs::characteristic_cap` takes `_characteristic` — **unused** — and returns the ruleset's single global `base_max_score`. **No `Effect` variant lowers it**, so nothing a V/F carries can reach it. `RULES.md:4740-4752` says the same in its own words (*"no cap validation"*) and the citation checks out. | **clean, confirmed** |
| `flaw.uncontrollable_strength` | `Prereq` has no Characteristic-comparing variant | the enum is exactly `All/Any/Nor/Has/House/AbilityMin/ArtMin/IsMagus` (read) | **clean, confirmed** |
| `flaw.viaticarus` | a penalty *equal to Decrepitude* is inexpressible | the nearest variant, `Effect::AbilityRollMod`, carries a **fixed `amount: i8`** and one free-text subject, so it cannot hold a value derived from another score. `RULES.md:4740-4752` independently records *"no `Effect` variant multiplies by Decrepitude"* for `flaw.lingering_injury`, the same shape. | **clean, confirmed** |
| `flaw.unbearable_to_beings` | the span's model entry — every field right, and the incompatibility symmetric | `flaw.blatant_gift` lists `flaw.unbearable_to_beings` back (`jq`), so symmetry holds and the ruleset loads. `max_total: 1` is a **top-level** field on the entry, not a key inside `parameters` — worth stating, because the Method's phrasing reads as though it lived inside the parameter object. | **clean, confirmed** |
| `flaw.unimaginative_learner` | the `-3` reaches the read-out with its sign | `Effect::AdvancementMod` pushes a `SurfacedModifier` with `amount: i32::from(*amount)` — the real `-3`, unlike the susceptibilities' hard-coded `0`. That difference is exactly what distinguishes this entry from Q-136's, and it makes the clean verdict defensible rather than lucky. | **clean, confirmed** |

The remaining thirteen clean entries state no mechanic the engine could reach
(`tainted_offspring`, `temperate`, `true_love_major/_minor`, `tzadik_nistar`,
`unbaptized`, `unhappily_married`, `unruly_air`, `visions`) or state one for a
subsystem that does not exist at all (`susceptibility_to_sunlight` — no light or
Fatigue-over-time model; `twilight_prone`, `unpredictable_magic`, `unlucky` — no
Twilight, botch-die or luck model). Each was checked against the shipped text in
**both** locales and against the passage; none moved.

### Findings — every one re-derived from primary sources

**Confirmed outright, on evidence taken independently (7):**

- **F-523.** Every leg holds. `decisions.md` D3 names both ids and rules them
  `uncomputed_rule` with the halving in `description` (read in full). Both ship
  `"classification": "in_play_effect"` with
  `magic_resistance_mod{susceptible_faerie|susceptible_infernal}` (`jq`). **Neither
  carries a `description` in `en` or `de`** (`jq` over both i18n objects — which
  are JSON **objects** keyed by id, not arrays, a shape worth recording). And the
  consumer was read rather than trusted: `derived.rs::in_play_mods:299-320` matches
  `Effect::MagicResistanceMod`, routes only `NoFormBonus` and `HalvedParma` into
  the per-Form grid sets, and sends `SusceptibleFaerie` / `SusceptibleInfernal`
  (with `AuraBonus` and `ConditionalPenetrationWaiver`) to
  `m.surfaced.push(SurfacedModifier { … amount: 0 })`. A37 is accurate here; its
  attribution to `surfaced_modifiers` is loose — that function (`derived.rs:627`)
  merely exposes `m.surfaced`, while the push happens in `in_play_mods`
  (`derived.rs:206`). The narrowing also holds: both shipped `summary` strings are
  ArMDE:6821 and :6825 **word for word in both locales**, so `RULES.md:4728-4731`'s
  byte-identical-duplicate policy applies and only the classification is live.
- **F-524.** The highest-stakes claim, and it is exact. See the confirmation block
  added inside the finding. The one change is a **narrowing of the remedy**: the
  `summary` already carries the whole one-sentence rule in both locales, so no
  `description` is owed. The HIGH severity stands untouched — the sheet still
  prints a +1 the Flaw forbids, and nothing in `crates/` or `ui/src/` mentions
  `flaw.unspecialized`.
- **F-525.** `validation/scores.rs::validate_reputations:585-617` read in full: it
  builds a per-kind grant count plus a `wildcard` bucket for grants with
  `reputation_type: None`, decrements one per `entity.reputations` row, and errors
  only when both are exhausted. **It never looks at score and has no polarity
  field.** `virtue.famous` ships `grants_reputation{score: 4}` with **no `kind`** —
  the wildcard — and `virtue.hermetic_prestige` ships `{hermetic, 4}`; both are
  `incompatible_with: null`. `flaw.tainted_with_evil` is `general`, so the pair is
  legal on **every** profile including a grog. Confirmed.
- **F-527.** `virtue.true_faith` exists (Major, general, `creation_effect`,
  `true_faith_grant{score: 1}`) and carries `incompatible_with: null`; the Flaw
  carries none either; and the prohibition **is** in the shipped `description` in
  both locales (EN *"This Flaw is not compatible with the True Faith Virtue."*, DE
  *"Dieser Fehler ist nicht kompatibel mit der Tugend Wahrer Glaube."*). The
  `Prereq::Nor`-versus-`incompatible_with` call was re-checked against the actual
  reachability rather than the field name the Method used: `granted_selections`
  appears **nowhere in `rules/`** — the real grant mechanism is
  `Effect::GrantsSelection`, carried by exactly **7** entries
  (`faerie_doctor`, `ineslemen`, `lone_redcap`, `rosh_beth_din`, `spirit_votary`,
  `strong_faerie_blood`, `templar_commander`), **none** of which grants
  `virtue.true_faith`, and `rules/core/houses.json` contains no `true_faith` at
  all. `validation/prereq.rs::validate_incompatibilities:299-335` iterates
  `entity.selections` and tests against the bought `selected_ids`, confirming the
  bought-only reading. **A plain symmetric `incompatible_with` is sufficient and
  the finding's remedy is reachable.**
- **F-536.** ArMDE:2890 and the tables at :2892-2930 read directly. Surgical
  Empiricus occurs at exactly **four** sites in the whole English book (`rg -an`):
  the Western Christendom Minor Flaws row (:2909), the index link (:5531), its own
  heading (:6811) and the alphabetical index (:25534). It is in **no** other
  culture table, and `flaw.usurer` is in the All Cultures row (:2899) as claimed.
  Confirmed. *Incidental lead for the Phase 2 sweep this finding proposes:*
  **`Doctor in (Faculty)` is itself Western-Christendom-only** (ArMDE:2908), so
  `flaw.university_dean` inherits the restriction transitively.
- **F-537.** All four counts re-taken with `rg --count-matches` (occurrences) and
  a line-printing pass: the shipped needle `darf nicht größer sein als` matches
  **0** in the German core book and **0** in `rules/i18n/de/`; `darf nicht größer
  als` matches **2** (DE 4970, DE 6921); `nicht größer` matches **3** (4970, 6921,
  15400); the English twin `may not be greater than` matches **1** (ArMDE:6921).
  The mechanism claim was checked against the function itself —
  `has_mechanical_phrase` lowercases and does a `match_indices` contiguous
  substring test with a left word boundary only — so the needle genuinely cannot
  span `als 0 sein`. Confirmed in full.
- **F-529's engine claim.** `SpellDef::ritual: bool` exists at `spell.rs:187` and
  is read (`integrity.rs:1646,1651`; `derived/casting.rs:293,306`), so a validator
  refusing a Ritual selection under this Flaw is genuinely constructible, exactly
  as the finding says.

**Scope-corrected, and these are the pass's substantive catches (2):**

- **F-532 and F-533 both asserted that a grog can take the Flaw** — F-532 in the
  strongest possible form (*"Every entity kind can take this Flaw today"*). **Both
  are wrong.** The `grog` profile's `permitted_categories` is
  `["general","personality","social_status","supernatural"]`; **`story` is absent**,
  and `validation/selections.rs::validate_permitted_categories:56-129` raises a hard
  `ValidationIssue::error` (`CODE_CATEGORY_NOT_PERMITTED`) when no in-force category
  is permitted. Both findings survive at unchanged severity over `companion` and
  `mythic_companion` — both `is_magus: false`, both permitting `story` — but the
  stated population was double the real one. The same correction reaches **F-526**,
  whose entry is also `story`. This is the "a remedy that cannot fire is not a
  remedy" check applied to a *defect scope*, and it is the reason the pass exists.

**Narrowed or widened without overturning (4):**

- **F-524** — remedy narrowed: no `description` is owed (see above). Severity
  unchanged.
- **F-526** — clause 3 **widened**. The finding said the age floor "is not
  expressible" and rested the whole `uncomputed_rule` label on it. But
  `Entity::age` exists (`types.rs:3717`) and
  `validation/life_stage.rs::validate_life_stage_age_meets_minimum:141-166` already
  reads it against a floor. The gap is a **missing data mechanism**, not D3's
  structural impossibility, so **all three** clauses are expressible with modest
  machinery. Separately, the `flaw.poor` half of the incompatibility is reachable
  on the `companion` profile **only**: `flaw.poor` is in `forbidden_traits` for
  both `magus` and `mythic_companion`, and a grog cannot take a `story` Flaw.
  The count is corrected from seventeen Flaws to **sixteen**.
- **F-538** — the reported defect stands (a magus who takes Vengeful Powers as a
  Hermetic Flaw gets no credit), but the **proposed remedy is under-specified**.
  `validation/magus.rs::validate_house`'s `counts_as_hermetic` tests *membership*
  categories and `index_categories`, **deliberately not** the `taken_as`-narrowed
  `categories_for` — its own comment says so. So the `virtue.sufi` copy would
  credit the Hermetic Flaw unconditionally, including for a player who took the
  Story reading the book calls the common one. Recorded inside the finding.
- **Q-136** — **widened from 12 to 19 entries**, with the entries enumerated. The
  ruling matters more than the question claimed.

**Refused withdrawal (1):**

- **F-535 was flagged by the primary agent "for the verification pass
  specifically" as the weakest in the batch, and it is NOT withdrawn — but the
  tension that argues for withdrawal is real and is recorded here rather than
  buried.** The tension is *internal to this batch*: the cross-reference table
  rates `flaw.viaticarus`'s *"he **should** take the Outcast Social Status Flaw"*
  (ArMDE:6983) as **"No defect"** partly because *"the verb is *should*"*, while
  F-535 files a finding against `flaw.tragic_life`'s *"**should** be represented
  with a sinful Personality Trait"* (ArMDE:6859). The same hedge, two verdicts.
  **What distinguishes them, and why the finding survives:** the Viaticarus
  sentence is additionally **conditional** on a saga fact the app stores nowhere
  (*"If the character's status is known in the community"*), so it is unenforceable
  twice over; Tragic Life's is unconditional. And D5's own point 3 forecloses the
  easy exit — *"A `narrative` entry still carries no description … If it does have
  something to write, it was misclassified"* — so the question cannot be dodged by
  calling the clause soft. It stays at **LOW**, and a reviewer who withdrew it on
  the grounds that an *unscored* Personality Trait on a **Story** Flaw (ArMDE:2502's
  ±3/±6 is keyed to *Personality* Flaws and does not reach it) is characterisation
  rather than mechanics would not be obviously wrong. **Left as a finding; flagged
  as the batch's single genuinely arguable call.**

### Added by this pass

**Two further faithful-copy German source errors, neither a data defect.** The
Method lists three (`vergekten`, `fröhnst`, and the EN `*Minor. Hermetic*`
descriptor typo). Reading the DE source against the shipped DE text turns up two
more of the same family, both reproduced **exactly** from the source and both
therefore correct to leave alone on the `scheitest` precedent
(`corrections.md` § 5):

| Entry | Shipped DE | DE source | The error |
|---|---|---|---|
| `flaw.suppressed_gift` | *"durch **einen** Missgeschick oder ein anderes Unglück"* | DE 6805, identical | *Missgeschick* is neuter — `ein`, not `einen` |
| `flaw.usurer` | *"**Du verleiht** oft Silber oder andere Wertsachen"* | DE 6953, identical | 2nd person needs *verleihst* |

Recorded so a later pass does not "fix" the data away from its source, and so the
list of known DE-source errata in this span is complete at **five**.

**The ASCII-hyphen check was re-run stronger than per-value.** Rather than
inspecting each signed string, `rg -c '[–−][0-9]'` was run over **both** shipped
i18n files: **zero** occurrences of U+2013 or U+2212 immediately before a digit in
either locale, catalogue-wide — while the German *source* does use U+2013 (DE 6921
writes `–3-Abzug`, the shipped text `-3-Abzug`). `CLAUDE.md`'s hyphen rule holds,
and the claim is now proven for the whole file rather than for this span's values.

### Duplicate check — re-run and confirmed

All 35 in-span ids were grepped across `corrections.md` **and** `batch-17.md`
(`rg -an`, one alternation). **Exactly one hit**: `batch-17.md:559`, the
`NO_RULE_DESPITE_TOKEN` exemption note naming `flaw.true_love_major` and
`flaw.true_love_minor` as out of B17's span and explicitly declining to rate them.
The Method reported *three* hits; the other two are on `virtue.true_love_pc`, a
**different id**, which the Method itself says. Scoped to the 35 ids that are
actually B18's, the answer is one hit and it is not a finding. **No prior finding
lands on any entry in ArMDE:6803-6988; F-523…F-538 are all new.** Numbering is
continuous behind them (`corrections.md` → F-501/Q-129, `batch-17.md` →
F-522/Q-135).

### What this pass could not check

`rules/source/de/translation-tables/` was not in dispute anywhere in this span —
all 35 shipped German names match their DE rulebook heading at the matching line,
so **D7's default decides every one and no glossary row is consulted against a
heading**. The upstream project `arm-de-translation` is outside the working
directory and unreadable here (argument-path containment), but **no D6 stale-copy
determination arises**, so nothing is owed to the orchestrator on that front.
