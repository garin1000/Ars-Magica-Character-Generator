# Batch B06 — indices 175-209, ArMDE:4351-4597

Entries: 35. Audited: 35. Failures: **31**. Clean: **4**.
Findings: **F-161 … F-208** (48). Open questions: **Q-45 … Q-54** (10).

An independent verification pass re-derived the four verdicts this batch had
cleared, was told that overturning one would be a success and inventing one would
not, and was not allowed to read this file. **Result: 4 confirmed, 0
overturned** — the first batch in the audit with no overturn. It contributed two
open questions (Q-53, Q-54), sharpened Q-45, added one cross-reference, and
independently corroborated eight of this batch's findings from the sources
without having seen them. See "Sub-agent reconciliation" at the end.

Finding and question numbers **continue B05's sequence** (B01 F-01…F-33 /
Q-01…Q-08, B02 F-34…F-65 / Q-09…Q-16, B03 F-66…F-89 / Q-17…Q-22, B04 F-90…F-121
/ Q-23…Q-34, B05 F-122…F-160 / Q-35…Q-44), so this batch starts at **F-161** and
**Q-45**.

**This span has never been screened.**
`crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS` stops at
ArMDE:3950; the whole of 4351-4597 is outside it, as B04's and B05's spans were.
The failure rate matches theirs (B04 28/35, B05 29/35, this batch 31/35), but
the *profile* is different and that is the useful part of the result:

- **Ten** findings are the B04/B05 shape in its pure form — `narrative` on an
  entry that states a rule and nothing else is wrong (F-162, F-166, F-167,
  F-175, F-176, F-177, F-202, F-203, F-204, F-207), four of them in plain digits
  (`natural_leader` "+3", `maker_of_textured_vessels` "+3",
  `mystical_choreography` "five minutes per magnitude", `magical_mount` "Magic
  Might Score of up to 25"). Six further entries are *also* reclassified out of
  `narrative`, but their primary defect is a missing effect and they are counted
  in the next two buckets instead (`mamluk`, `master_of_form_creatures`,
  `mazdean_priest`, `mendicant_friar`, `mercenary_captain`,
  `minor_enchantments`) — **sixteen `narrative` reclassifications in all.**
- **Nine** are the open `ability_authorization` pattern, now found in six
  consecutive spans. This batch is the densest yet: four Social Status Virtues
  that grant Martial or Academic permission with no effect at all
  (`mamluk`, `mazdean_priest`, `mendicant_friar`, `mercenary_captain`), two that
  carry a `restricted_ability_xp` naming Abilities where the passage grants a
  whole gated **category** (`marshal`, `master_of_kennels` — "may take Martial
  Abilities **freely**"), one where the pool authorizes two ids against a
  passage that says "Arcane Abilities" (`master_bard`), and two whose sole
  permission clause is a single gated Ability nobody encoded
  (`magical_blood`, `master_of_form_creatures` — both "may take Magic Lore",
  and `ability.magic_lore` is `arcane`).
- **Thirteen** are D5 leftovers — a rule stated by the passage that no effect
  implements and that appears in neither locale (F-161, F-164, F-171, F-173,
  F-174, F-181, F-184, F-187, F-194, F-197, F-198, F-201, F-206). The span holds
  **16** effect-carrying entries; **14** of them leave something (the other
  defect on `master_of_kennels` is a missing effect, not a missing description),
  and only **two** are clean throughout — `virtue.mastered_spells` and
  `virtue.mild_aging`.
- And **sixteen are none of those three shapes** — a third of the batch — which
  is why all twelve checks are run on all 35 (10 + 9 + 13 + 16 = 48):
  - **a `source.lines` range that stops three lines before the rule it cites** —
    `virtue.minor_magical_focus` (F-200), the **first check-1 failure the audit
    has found** in six batches;
  - **three missing grants of a named Virtue or Flaw**, all three expressible,
    one of them the directed read — `virtue.nephilim` (F-208),
    `virtue.mythic_blood` (F-205), and `virtue.mercurian_magic`'s required
    companion Flaw (F-196);
  - **an effect variant the engine already has, on an entry authored as
    `narrative` with no effect at all** — `virtue.minor_enchantments` (F-199)
    states a 25-level item allowance, and `item_level_budget` exists and is
    carried by its two neighbours (`virtue.magic_items` 25,
    `virtue.redcap` 50);
  - **a hard mutual exclusion stated in the book and absent from
    `incompatible_with`** — `virtue.mendicant_friar` (F-191), "You may not take
    the Wealthy Virtue or Poor Flaw";
  - **three entries whose stated Ability prerequisites are missing and
    expressible** — F-169, F-172, F-183 (the B05 `leper_magus` shape, now on
    three Social Status Virtues at once);
  - **four gender restrictions stated in the book and in neither locale** —
    F-170, F-179, F-189, F-192, plus the mirror case F-177
    (`male_guild_sponsor`, female-only), which is B05's F-123 recurring five
    times in one span;
  - **a player choice the save cannot hold** — `virtue.magical_blood` (F-165),
    which of the four bloodline types was taken;
  - **one German name that contradicts the canonical table** — F-168
    (Hüter vs. Wächter);
  - **one truncated English summary** — F-193 ("You are a follower of St.").

  The single most consequential finding is counted under D5 above but belongs
  here too: **`virtue.mentored_by_demons` (F-194)**, where the passage waives the
  age→Ability-score cap and `validate_ability_age_cap` raises a hard **error** —
  so the app refuses the character the passage describes, and explains nothing.

*(This file was written incrementally — the verdict table landed first, findings
were appended as each entry was finished, and the sub-agent reconciliation
last.)*

## The two directed reads — both answered

### 1. Major vs Minor Magical Focus — `major` is genuinely redundant at the computation layer

`Effect::MagicalFocus`'s `major` field is bound by no production site (C2-b).
Phase 0 could not settle whether that is a defect. **It is not.** The book gives
the two magnitudes the *same arithmetic*, in one sentence each, and the Minor
passage says so by explicit cross-reference.

**Major Magical Focus, ArMDE:4403** (verbatim, first sentence):

> When you cast a spell or generate a Lab Total within your focus, add the
> lowest applicable Art score twice.

**Minor Magical Focus, ArMDE:4540** (verbatim, the *entire* paragraph):

> When you cast a spell or generate a Lab Total within your focus, add the
> lowest applicable Art score twice, **as for a Major Magical Focus (page 94)**.

German, line-parallel, ArMDE:4403 and ArMDE:4540:

> Wenn du einen Zauber wirkst oder eine Laborsumme innerhalb deines Fokus
> berechnest, addiere den niedrigsten anwendbaren Kunstwert zweifach.

> Wenn du einen Zauber wirkst oder eine Laborsumme innerhalb deines Fokus
> berechnest, addiere den niedrigsten anwendbaren Kunstwert zweifach, **wie beim
> Großen Magischen Fokus ([Seite 94](#großer-magischer-fokus))**.

The Minor passage does not restate the arithmetic and then differ — it *points
at the Major one*. There is no second sentence, no halving, no cap, no
different multiplier.

**What does differ is breadth and cost, and nothing else:**

| | Major (ArMDE:4401) | Minor (ArMDE:4538) |
|---|---|---|
| Breadth | "This area should be smaller than a single Art, but may be spread over several Arts" | "the field should be slightly narrower than a single Technique and Form combination, although it may include restricted areas of several such combinations" |
| Cost | `*Major. Hermetic*` — 3 points | `*Minor, Hermetic*` — 1 point |
| Arithmetic | add the lowest applicable Art score twice | **the same**, by cross-reference |

Both passages also state the one-focus limit in identical terms — ArMDE:4405
"A character can have only one Magical Focus, either major or minor, regardless
of the source of the focus" / ArMDE:4542 "A magus may only have one Magical
Focus, whether major or minor, regardless of the source of the focus" — and the
engine already enforces it the right way: `validate_magical_focus` counts
`MagicalFocus` *effects* over the folded bought-plus-granted list, so it catches
two Minor foci and it catches Mythic Blood's granted focus alongside a bought
one, neither of which a pairwise `incompatible_with` would see.

**Conclusion, stated for the record so it is not re-opened:** `major` carries a
real distinction *in the rulebook* — the breadth of subject matter the player may
claim — but that distinction lives in free-text the engine deliberately never
parses (C2-b: `param` is validated at load and never consulted). **At the
computation layer it is redundant, and its being unread is correct, not a
defect.** Phase 0's open question 2 is closed.

The one thing that *is* wrong here is elsewhere in the same pair: the breadth
rule is the only mechanical difference between the two Virtues, and neither
entry states it in either locale (F-174, F-201), so a player picking between a
1-point and a 3-point Virtue is told nothing about what distinguishes them.

### 2. `virtue.nephilim` — the entry should carry a `grants_selection`, and `creation_effect` is right once it does

**Passage, ArMDE:4594-4597** (verbatim, the whole entry):

> #### Nephilim
> *Free, Mythic Companion*<br>
> You are one of the Nephilim, and a Mythic Companion (page 63). You receive the
> Strong Angelic Heritage Virtue free.

German, line-parallel:

> Du bist einer der Nephilim und ein Mythischer Gefährte ([Seite 63](#mythische-gefährten-2)).
> Du erhältst die Tugend Starkes Engelserbe kostenlos.

**The second sentence is a fixed, single-item free grant — exactly what
`Effect::GrantsSelection` expresses — and two of its three siblings carry
theirs.** This is the decisive evidence, and it is in the data, not in judgement:

| Mythic Companion Virtue | What its passage gives free | `effects` today |
|---|---|---|
| `virtue.faerie_doctor` (ArMDE:3821-3824) | one named Virtue | `grants_selection: ["virtue.dowsing"]` ✓ |
| `virtue.spirit_votary` (ArMDE:5006-5009) | one named Virtue | `grants_selection: ["virtue.second_sight"]` ✓ |
| `virtue.devil_child` (ArMDE:3673) | "the Demonic Might **or** Demonic Powers (**player's choice**) Minor Virtue free" | none — **correctly**, `grants_selection` names fixed ids and cannot express a choice |
| **`virtue.nephilim` (ArMDE:4596)** | **one named Virtue** | **none** |

So Nephilim is not in Devil Child's position. It is in Faerie Doctor's and
Spirit Votary's, and it is the only one of the three that is empty. → **F-208.**

**Is `creation_effect` the right class?** Yes — and it will be honest rather
than merely green once the effect is there. B1/C7 record that the guard does not
require a `creation_effect` entry to carry effects, which is why the empty entry
passes today. The clause the class asserts ("changes a character-creation number
or state") is real: a free Minor Virtue is a creation-time state change. What it
is *not* is `narrative` — the passage plainly states a mechanic.

**And the grant is reachable-as-missing, not merely theoretical.** The free
Virtue is delivered today only by `mythic_type.nephilim`'s
`{ "kind": "fixed", "item": "virtue.strong_angelic_heritage" }` grant. But
`rules/core/character_types.json`'s `mythic_companion` profile lists
`mythic_companion` among its `permitted_categories`, so a Mythic Companion can
select `virtue.nephilim` straight from the V/F picker **without** choosing the
mythic type — and then receives nothing. (Checked: `mythic_companion` is the
only character type permitting that category, so no other type is affected.)

**What the entry should *not* carry.** B04 was right that the two Great
Characteristics sit in `mythic_type.nephilim`'s `required_virtues`, which is
budgeted, and they belong there — ArMDE:2726-2727 lists them among the *required*
Virtues the character takes and pays for, not among the free ones. Only ArMDE:2730
"Strong Angelic Heritage (Minor, Supernatural — **free with Nephilim**)" is
marked free, and the parenthetical names the Virtue as the source. One grant, not
three.

## Method

The whole span was read as continuous prose in **both** languages before any
entry was judged — `rules/source/en/Ars Magica - Definitive Edition (Core
Rules).md` ArMDE:4345-4609 and the line-parallel
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md` 4345-4609.
**Line parity holds throughout the span:** all 35 entry `####` headings sit on
the same line number in both files, checked at every heading rather than sampled
— 4351, 4355, 4359, 4373, 4377, 4385, 4395, 4399, 4423, 4431, 4439, 4443, 4449,
4457, 4463, 4467, 4471, 4476, 4480, 4488, 4496, 4500, 4506, 4510, 4514, 4524,
4528, 4532, 4536, 4559, 4563, 4567, 4573, 4590, 4594 — as do the two block-quote
headings that are **not** entries and are correctly claimed by nobody in their
own right: `#### Sample Major Magical Foci` (ArMDE:4407, inside
`virtue.major_magical_focus`'s cited range) and `#### Sample Minor Magical Foci`
(ArMDE:4544, which is **outside** every cited range — see F-200).

**35 headings, 35 entries, one-to-one.** Unlike B05 there is no shared heading
in this span.

**`source.lines` (check 1) — 34 of 35 correct, one wrong.** The convention every
correct range follows is "own heading → the line before the next heading",
which usually means the range ends on a blank line
(`virtue.magic_sensitivity` 4351-4354, with 4354 blank before `#### Magical
Memory` at 4355). The exception is **`virtue.minor_magical_focus` 4536-4538**,
which stops inside its own entry: the arithmetic (:4540), the one-focus rule
(:4542) and the twelve-line sample box (4544-4557) all fall outside it, and
lines **4539-4558 are cited by no entry in the catalogue** — the next entry
starts at 4559. **F-200.**

**No entry in this batch carries an `anchor`** — all 35 `source` objects are
`{file, lines}` only, so B11's "confirm the anchor names the entry" check has
nothing to verify here and the line ranges are the only provenance. (Contrast
`virtue.free_expression`, followed as a cross-reference below, which does carry
`"anchor": "free-expression"`.)

**Magnitude, kind, categories, entity_kinds (checks 3, 4, 5, 6).** Every
descriptor line was read and compared against the data. `kind` is `virtue` for
all 35 and all 35 sit inside the Virtues section — correct. `entity_kinds` is
`["character"]` on all 35, which is right: none is a covenant Boon. `magnitude`
and `categories` agree with every descriptor:

- `*Minor, Supernatural*` — magic_sensitivity, magical_blood,
  maker_of_textured_vessels, maker_of_water_vessels, master_of_form_creatures,
  minor_enchantments, muse
- `*Minor, Hermetic*` — magical_memory, mastered_spells, method_caster,
  minor_magical_focus, mystical_choreography; plus **masterpiece**, printed
  `*Minor. Hermetic*` (a period for the comma, an OCR artefact in the source,
  not a data defect)
- `*Minor, General*` — magical_mount, mild_aging, natural_leader
- `*Minor, General, Tainted*` — mentored_by_demons, and `"tainted": true` is
  set ✓ (the only `tainted` entry in the batch; a whole-file check confirmed no
  other entry in the span sets it)
- `*Major, General*` — magical_warder
- `*Major, Hermetic*` — mercurian_magic, mythic_blood; plus **major_magical_focus**,
  printed `*Major. Hermetic*` (the same OCR artefact)
- `*Minor, Social Status*` — mamluk, marshal, master_of_kennels,
  mazdean_priest, mendicant_friar, mercenary_captain, merchant_adventurer
- `*Major, Social Status*` — magister_in_artibus, magister_in_medicina,
  master_bard, muqta_muq_ta
- `*Free, Social Status*` — male_guild_sponsor, merchant
- `*Free, Mythic Companion*` — nephilim

Two descriptor lines in the source carry a period where a comma belongs
(`*Major. Hermetic*` at ArMDE:4400, `*Minor. Hermetic*` at ArMDE:4477). Both are
OCR artefacts in the rulebook text, both parse unambiguously, and the data reads
them correctly — recorded so a later reader does not take them for data defects.

**Check 12, mechanical.** Whole-file count of U+2212 (the mathematical minus) in
**both** locales' `virtues_flaws.json`: **zero**. Every `name` and `summary` in
the batch was read against its passage in the matching language. **No entry in
this batch carries a `description` in either locale** — all 35 are `name` +
`summary` only, which is why every D5 finding below reads "reaches neither
locale" rather than "is wrong in one". One English summary is defective on its
own terms (F-193). Note for the correction pass: the source's own aging figure
is written with an en dash ("a –1 bonus", ArMDE:4363) in **both** languages; any
description transcribing it must use the ASCII hyphen per CLAUDE.md.

**German terminology was checked against `rules/source/de/translation-tables/`
and against `rules/i18n/de/abilities.json`, not only against the DE rulebook** —
the B04/B05 lesson. Ten of this batch's German names appear in a table and
**nine agree with it**:

| Entry | Table row | German |
|---|---|---|
| Magic Sensitivity | `tugenden-fehler.md:59`, `:134`; `fertigkeiten.md:68` | Magiegespür ✓ |
| Major Magical Focus | `tugenden-fehler.md:31`, `:60` | Großer Magischer Fokus ✓ |
| Minor Magical Focus | `tugenden-fehler.md:63` | Kleiner Magischer Fokus ✓ |
| Mastered Spells | `tugenden-fehler.md:61` | Gemeisterte Zauber ✓ |
| Masterpiece | `tugenden-fehler.md:62` | Meisterstück ✓ |
| Magical Blood | `tugenden-fehler.md:135` | Magisches Blut ✓ |
| Magical Mount | `tugenden-fehler.md:222` | Magisches Reittier ✓ |
| Mercurian Magic | `tugenden-fehler.md:32` | Merkurische Magie ✓ |
| Mythic Blood | `tugenden-fehler.md:33` | Mythisches Blut ✓ |
| Natural Leader | `tugenden-fehler.md:203` | Natürlicher Anführer ✓ |
| Magister in Artibus | `tugenden-fehler.md:252` | Magister in Artibus (Lat.), untranslated ✓ |
| Nephilim | `tugenden-fehler.md:744` | Nephilim ✓ |
| Method Caster | `grundbegriffe.md:367` | Methodischer Zauberer ✓ |
| Master Bard | `reputationen.md:89` | Meisterbarde ✓ (and its Local Reputation 3 ✓) |
| **Magical Warder** | **`tugenden-fehler.md:170`** | table says **Magischer Wächter**, data says **Magischer Hüter** → **F-168** |

`virtue.master_of_form_creatures` is a near-miss that is **not** a finding and is
recorded so it is not re-raised: `tugenden-fehler.md:136` gives
`Meister der (Form-)Kreaturen` while the data says `Meister der {form}-Kreaturen`
and the DE rulebook heading (ArMDE:4463) says `Meister der (Form)-Kreaturen`.
All three render the same string once a Form is substituted
("Meister der Animal-Kreaturen") — the parenthesis placement is a
placeholder-notation difference, not a terminology one.

The remaining twenty German names appear in no table, so the table constraint
does not bind them; each was checked against its DE rulebook heading instead and
all twenty match exactly.

Two DE ability names the passages depend on were confirmed against
`rules/i18n/de/abilities.json`: Magic Lore → **Magiekunde**, Theology: Islam →
**Theologie: Islam**. Both match the DE rulebook body (ArMDE:4363/4465, :4447).

### Cross-references followed

Every pointer in the span was followed and read — the `(see page NNN)` form and
the named-Virtue form alike — including on entries that turned out to pass.
Sixteen pointers:

| Entry | Pointer | Where it lands | Does it add a rule attributed to the Virtue? |
|---|---|---|---|
| `virtue.magic_sensitivity` | "(page 168)" / `#magiegespür-1` | `#### Magic Sensitivity\*`, **ArMDE:7635-7644** | **No new rule** — the Ability passage repeats the Virtue's two sentences verbatim and then adds the Ease Factor guidance, which belongs to the Ability. But it is *relevant*: the Ability's own i18n `description` already carries the Magic-Resistance subtraction in **both** locales ("the sensitivity subtracts from your Magic Resistance" / "doch das Gespür wird von deiner Magieresistenz abgezogen"), which is the mitigation noted under F-161. |
| `virtue.magical_memory` | "(see page 262)" / `#labortexte-1` | `## Laboratory Texts` → `### Using Laboratory Texts`, **ArMDE:10674-10684** | **No.** It is the *baseline* the Virtue waives the physical text for ("A magus who has a Laboratory Text for a particular effect may reproduce it in a single season if his Lab Total equals or exceeds the level"). Same shape as B05's `jack_of_all_trades`. The Virtue's own clause is F-162's. |
| `virtue.magical_warder` | "See page 457 for example statistics" / `#geisterwächter` | example statistics for a ghostly warder | **No** — a statistics block, not a rule attributed to the Virtue. |
| `virtue.magical_warder` | "the guidelines in Chapter 13, or in *Realms of Power: Magic*" | Chapter 13 (in-repo) and RoP:M (in-repo) | **No rule the Virtue states.** Designing a magic character is a whole subsystem; nothing here is attributed to the Virtue. |
| `virtue.magical_mount` | "Favors … Enemies" (named Flaws) | `flaw.favors`, `flaw.enemies` | **No new rule** — but the *requirement* to take a Major Story Flaw is one, and it is F-166's. |
| `virtue.magister_in_medicina` | "**This Virtue offers the same benefits as Doctor in (Faculty).**" (named-Virtue, no page) | `#### Doctor in (Faculty)`, **ArMDE:3683-3691** | **Yes, and it is the whole entry.** The 300 XP and the Academic Reputation 3 the data carries come from there and are **correct** (`:3687`). Four further rules come with them and reach neither locale → F-172, F-173. |
| `virtue.major_magical_focus` | (none) | — | — |
| `virtue.master_of_form_creatures` | "See page 384 for rules for training magical (and mundane) animals" / `#kreaturen-abrichten` | `## Training Creatures`, ArMDE:16290 | **No** — the general training rules, not attributed to the Virtue. |
| `virtue.mastered_spells` | "(See page 225 for rules on mastering spells.)" / `#zaubermeisterschaft` | `## Spell Mastery`, ArMDE:9514 | **No** — the general Mastery rules. The Virtue's own clause ("fifty experience points … spells that you know") **is** computed: `spell_mastery_xp` builds a `PoolEligibility::Mastery` pool that funds Mastery spends only (A16). |
| `virtue.masterpiece` | "the regular rules for construction of such a device" | the lesser-enchantment rules | **No new rule** — but "must be a lesser enchanted item" and "You ignore vis costs" are the Virtue's own and are F-187's. |
| `virtue.mazdean_priest` | "*The Cradle and the Crescent*, Chapter 5" | **outside `rules/source/en/`** | Nothing claimable: CLAUDE.md forbids implementing from a book not in the sources. |
| `virtue.mendicant_friar` | "Monastic Vows, see page 138" / `#klostergelübde` | `#### Monastic Vows`, **ArMDE:6446-6448** | **No.** The Flaw states no number and the Virtue's wording is a suggestion ("could together constitute", "would be a natural choice"), not a requirement. Correctly encoded by encoding nothing. |
| `virtue.merchant_adventurer` | "See *City and Guild*" | outside `rules/source/en/` | Nothing claimable. |
| `virtue.merchant_adventurer` | "**the Partner Virtue**" (named-Virtue, no page) | `#### Partner`, **ArMDE:4616-4618** | **A rule, but it belongs to Partner, not here.** ArMDE:4618: "a partner who is also a factor, **merchant adventurer**, local carrier, or urban merchant **need not purchase that Virtue if she has this one**." So holding both is *redundant*, not illegal — which is why Merchant Adventurer's own "should select the Partner Virtue instead of this one" is guidance and **not** an `incompatible_with`, and why the entry stays clean. `virtue.partner` is ArMDE:4616-4619, outside this span — **flagged for B07**, not rated here. (Found by the verification pass; confirmed against the source before being recorded.) |
| `virtue.mercurian_magic` | "Wizard's Vigil (page 370)" / `#wacht-des-zauberers`; "Wizard's Communion (page 369)"; "Mastery score (page 225)" | `##### Wizard's Vigil`, **ArMDE:15806-15808**; the Communion; `## Spell Mastery` | **No new rule** — but it confirms Wizard's Vigil is a **named spell**, and the engine has no variant that grants a named spell (`spell_levels` moves a budget, A10). That is what makes F-197's first clause structurally inexpressible rather than merely unencoded. |
| `virtue.minor_magical_focus` | "**as for a Major Magical Focus (page 94)**" / `#großer-magischer-fokus` | `#### Major Magical Focus`, **ArMDE:4399-4422** | **Yes — this is directed read 1**, and the answer is that the arithmetic is imported unchanged. See above. It also imports the requisite rule (:4403), which no effect implements → F-201. |
| `virtue.muqta_muq_ta` | "**All rules for the Landed Noble Virtue apply**" (named-Virtue, no page) | `#### Landed Noble`, **ArMDE:4219-4227** | **Yes, four of them** — the Poor "must spend every season managing it", the Wealthy "do not need to devote any time", "wealthier than most characters, but have no additional free time", and the law-enforcement limits. The entry is `narrative` and states none → F-202. B05 found the same four unstated on Landed Noble itself (F-131). |
| `virtue.muse` | "**Free Expression**" (named-Virtue, no page) | `#### Free Expression`, **ArMDE:3933-3935** | **Yes.** "You get a +3 bonus on all rolls to create a new work of art" — so Muse grants a +3 to a third party, or doubles it to +6. `virtue.free_expression` is itself classified `uncomputed_rule`, which is the class Muse states the same kind of rule in → F-203. |
| `virtue.mythic_blood` | "fast-casting a Mastered Formulaic spell (see page 213)" / `#formulaische-magie` | the fast-casting rules | **No** — it fixes the *timing* of the invocation, which is part of F-206's clause, not a separate rule. |
| `virtue.nephilim` | "**a Mythic Companion (page 63)**" / `#mythische-gefährten-2` | `## Mythic Companions`, **ArMDE:2633-2641** | **Yes, and it settles two things.** `:2637` "These Virtues are incompatible with each other, and with The Gift, and are not available to grogs" **sources the entry's four `incompatible_with` ids**, which the entry's own passage does not. `:2638` "You gain a free Minor Virtue, normally specified by the Mythic Companion Virtue" is the general rule the entry's own sentence specifies → F-208. A third thing it turns up lies outside this batch → Q-50. |

**The named-Virtue form again produced the live hits** — Magister in Medicina,
Muqta', Muse and Minor Magical Focus are all page-number-free pointers that a
`(see page NNN)` screen cannot see, and three of the four yield rules the entry
drops. That is now true in four consecutive batches and is restated as a pattern
at the end.

### Part C systemic gaps are not re-reported per entry

In particular: `special_casting_mod`'s `mercurian` kind being surfaced-only (C1)
is **not** counted as a defect of `virtue.mercurian_magic`; `magical_focus`'s
unread `major` field and never-consulted `param` (C2-b) are **not** counted
against `virtue.major_magical_focus`, `virtue.minor_magical_focus` or
`virtue.mythic_blood` — and the first of those is now affirmatively answered
above; and `grants_reputation`'s unenforced `score` (C2-c) is **not** counted
against `virtue.magister_in_artibus`, `virtue.magister_in_medicina` or
`virtue.master_bard`. What *is* counted for each is what D5 obliges — the clause
the effects do not implement, absent from both locales.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D5 is again the decisive one.** Sixteen entries in this span carry `effects`,
and **thirteen** of them leave a stated rule that no effect implements and that
appears in neither locale: `virtue.magic_sensitivity` (F-161),
`virtue.magical_blood` (F-164), `virtue.magister_in_artibus` (F-171),
`virtue.magister_in_medicina` (F-173), `virtue.major_magical_focus` (F-174),
`virtue.marshal` (F-181), `virtue.master_bard` (F-184), `virtue.masterpiece`
(F-187), `virtue.mentored_by_demons` (F-194), `virtue.mercurian_magic` (F-197),
`virtue.method_caster` (F-198), `virtue.minor_magical_focus` (F-201),
`virtue.mythic_blood` (F-206). A fourteenth, `virtue.master_of_kennels`, fails on
a missing *effect* rather than a missing description (F-186). The two
effect-carrying entries with nothing left over are **`virtue.mastered_spells`**
and **`virtue.mild_aging`**, and both are clean on all twelve checks.

**D5 was also applied in the negative** — the passages of the two `narrative`
entries that survived (`virtue.merchant` ArMDE:4508, `virtue.merchant_adventurer`
ArMDE:4512) were read for a mechanical clause and found to state none. Merchant's
only near-miss is "The Wealthy Major Virtue and Poor Major Flaw affect you
normally", which is an explicit statement that *nothing* changes — correctly
encoded by encoding nothing, exactly as B05 rated `virtue.landed_noble`'s "You
get the normal points for Oath of Fealty if you do". Merchant Adventurer's are
"may have substantial debts, which **may** be represented by the Favors Flaw" and
"a merchant adventurer who owns a share … **should** select the Partner Virtue
instead of this one" — both hedged guidance, neither a rule.

**D3** governs six findings where the engine structurally cannot express the
rule and the answer is therefore `uncomputed_rule` (or, for an effect-carrying
entry, a `description`) and **never** `narrative`: the Magic-Resistance
subtraction (F-161 — `MagicResistanceEffect` has no score-subtracting kind,
A37), the four male-only / female-only restrictions (F-170, F-179, F-189, F-192
— `types.rs`'s concept field declares `pub gender: String` with "no mechanical
effect", the same ground as B05's F-123), Mythic Blood's *open* Minor
Personality Flaw (F-205 — `grants_selection` names fixed ids only, A18), and
Mercurian Magic's named-spell grant (F-197).

**D1/D4** do not bite in this span: no entry here carries `lab_total_mod`.

**D2** does not bite either: no entry here is a granted Great Characteristic
carrier.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.magic_sensitivity` | 4351-4354 | OK | OK | MR subtraction in neither locale | F-161 ArMDE:2960 |
| `virtue.magical_memory` | 4355-4358 | narrative → uncomputed_rule | OK | Lab-Text waiver in neither locale | F-162 |
| `virtue.magical_blood` | 4359-4372 | OK | effects missing (`ability_authorization`, magic_lore); no parameter for the bloodline type | the four types' mechanics in neither locale | F-163 F-164 F-165 ArMDE:2960 Q-51 |
| `virtue.magical_mount` | 4373-4376 | narrative → uncomputed_rule | OK | Might ≤ 25, Loyal 0, mandatory Major Story Flaw in neither locale | F-166 |
| `virtue.magical_warder` | 4377-4384 | narrative → uncomputed_rule | OK | detection rule + half-day limit in neither locale; de name contradicts the table | F-167 F-168 |
| `virtue.magister_in_artibus` | 4385-4394 | OK | both stated Ability prerequisites missing | male-only, teaching obligation, Poor/Wealthy, age formula in neither locale | F-169 F-170 F-171 ArMDE:2816 Q-45 |
| `virtue.magister_in_medicina` | 4395-4398 | OK | three expressible prerequisites missing | imported Doctor-in-(Faculty) rules + Salerno restriction in neither locale | F-172 F-173 ArMDE:2816 Q-45 |
| `virtue.major_magical_focus` | 4399-4422 | OK | OK | requisite rule, lab-activity restriction, breadth rule in neither locale | F-174 |
| `virtue.maker_of_textured_vessels` | 4423-4430 | narrative → uncomputed_rule | OK | +3, Fatigue cost, shape formula, Arcane Connection in neither locale | F-175 ArMDE:2960 Q-46 |
| `virtue.maker_of_water_vessels` | 4431-4438 | narrative → uncomputed_rule | OK | Ability swap, duration, Warping rule in neither locale | F-176 ArMDE:2960 Q-46 |
| `virtue.male_guild_sponsor` | 4439-4442 | narrative → uncomputed_rule | OK | mandatory second Social Status + female-only in neither locale | F-177 ArMDE:2816 Q-45 |
| `virtue.mamluk` | 4443-4448 | narrative → creation_effect | effects missing (`ability_authorization`, martial + theology_islam) | male-only in neither locale | F-178 F-179 ArMDE:2816 Q-45 |
| `virtue.marshal` | 4449-4456 | OK | martial permission not authorized | Profession-as-Medicine rule in neither locale | F-180 F-181 ArMDE:2816 Q-47 |
| `virtue.master_bard` | 4457-4462 | OK | "Arcane Abilities" authorized as two ids; prerequisites missing | two-season obligation + Ireland-only in neither locale | F-182 F-183 F-184 ArMDE:2816 Q-47 |
| `virtue.master_of_form_creatures` | 4463-4466 | narrative → creation_effect | effects missing (`ability_authorization`, magic_lore) | OK | F-185 ArMDE:2960 |
| `virtue.master_of_kennels` | 4467-4470 | OK | martial permission not authorized | — | F-186 ArMDE:2816 Q-47 |
| `virtue.mastered_spells` | 4471-4475 | OK | OK | OK | clean |
| `virtue.masterpiece` | 4476-4479 | **?** — see Q-52 | OK | lesser-item restriction + vis waiver in neither locale | F-187 Q-52 |
| `virtue.mazdean_priest` | 4480-4487 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | male-only in neither locale | F-188 F-189 ArMDE:2816 |
| `virtue.mendicant_friar` | 4488-4495 | narrative → creation_effect | effects missing (`ability_authorization`, academic); Wealthy/Poor exclusion missing from `incompatible_with` | male-only in neither locale; **en summary truncated** | F-190 F-191 F-192 F-193 ArMDE:2816 Q-45 |
| `virtue.mentored_by_demons` | 4496-4499 | OK | OK | **age-cap waiver unmodelled — app errors on a legal character** | F-194 |
| `virtue.mercenary_captain` | 4500-4505 | narrative → creation_effect | effects missing (`ability_authorization`, martial) | OK | F-195 ArMDE:2816 |
| `virtue.merchant` | 4506-4509 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.merchant_adventurer` | 4510-4513 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.mercurian_magic` | 4514-4523 | OK | required companion Flaw encoded nowhere | Wizard's Vigil, Mastery addition, half-vis rule in neither locale | F-196 F-197 Q-48 |
| `virtue.method_caster` | 4524-4527 | OK | +3 folded in unconditionally | the "if you vary at all" gate in neither locale | F-198 |
| `virtue.mild_aging` | 4528-4531 | OK | OK | OK | clean |
| `virtue.minor_enchantments` | 4532-4535 | narrative → creation_effect | `item_level_budget: 25` missing though the variant exists | 25-level total + Level 30 per-power cap in neither locale | F-199 ArMDE:2960 |
| `virtue.minor_magical_focus` | 4536-**4538** | OK | **`source.lines` stops before the rule** | inherited requisite rule + lab-activity restriction in neither locale | F-200 F-201 |
| `virtue.muqta_muq_ta` | 4559-4562 | narrative → uncomputed_rule | OK | the four imported Landed Noble rules in neither locale | F-202 ArMDE:2816 Q-45 |
| `virtue.muse` | 4563-4566 | narrative → uncomputed_rule | OK | granting/doubling Free Expression in neither locale | F-203 ArMDE:2960 |
| `virtue.mystical_choreography` | 4567-4572 | narrative → uncomputed_rule | OK | five/one minutes per magnitude in neither locale | F-204 |
| `virtue.mythic_blood` | 4573-4589 | OK | free Minor Personality Flaw encoded nowhere | Fatigue waivers + invocation table in neither locale | F-205 F-206 Q-49 |
| `virtue.natural_leader` | 4590-4593 | narrative → uncomputed_rule | OK | "+3" in neither locale | F-207 |
| `virtue.nephilim` | 4594-4597 | OK | `grants_selection` missing (directed read 2) | OK | F-208 Q-50 |

**Totals: 31 entries carry at least one finding, 4 are clean** —
`virtue.mastered_spells`, `virtue.merchant`, `virtue.merchant_adventurer`,
`virtue.mild_aging`. 31 + 4 = 35. One entry (`virtue.masterpiece`) carries both
a finding and a `?` on its classification, so no entry is left carrying only a
question.

## Findings

**A qualifier that applies to all nine `ability_authorization` findings (F-163,
F-178, F-180, F-182, F-185, F-186, F-188, F-190, F-195) and is stated once
rather than nine times.** A14 records that
`validation/authorization.rs::validate_ability_authorization` errors
(`ability_category_requires_virtue`) on a held Ability whose category is in
`ruleset.categories_requiring_virtue()` unless its id or category is authorized,
with a whole-character exemption for a profile whose `is_magus` is true. That
gated set is `rules/core/abilities.json` →
`"categories_requiring_virtue": ["academic", "arcane", "martial"]`. The Abilities
these nine passages name resolve into it: `ability.magic_lore` → **arcane**,
`ability.faerie_lore` → **arcane**, `ability.theology_islam` → **academic**,
`ability.dead_language` → **academic** (and `"scholarly_language": {"ability":
"ability.dead_language", "exemplar": "latin"}`, so the book's "Latin" is that
id). Every one of the nine carriers is a mundane Virtue — a Mamluk, a marshal, a
friar, a mercenary captain — so the `is_magus` exemption is empty in practice and
the refusal is exactly what bites. A7 also records that a `restricted_ability_xp`
pool confers the same permission for what it funds, so the check is always
"does the pool's `abilities`/`categories` union cover what the passage permits" —
and for `magister_in_artibus`, `magister_in_medicina` and `mentored_by_demons` it
**does**, which is why those three draw no authorization finding.

**A second qualifier, for every `narrative → uncomputed_rule` move below.** B1
records that `classification` is read by no production code, so none of these
moves changes a computed number. The cost is the one `ecb5150` names: a
`narrative` entry is not obliged to carry its rule in `description`, so the rule
leaves the application silently while the entry still looks complete. Every move
therefore carries the same correction — reclassify **and** write the rule into
`description` in **both** locales, or
`uncomputed_clauses.rs::every_uncomputed_rule_entry_states_its_rule_in_every_locale`
goes red. Severity for all of them: **lost-rule / provenance**, not
miscalculation.

---

### F-161 — `virtue.magic_sensitivity` — the Magic-Resistance subtraction reaches neither locale (D5)

**Passage** (ArMDE:4353, verbatim, English):
> You are often able to identify a place or object as magical. However, your sensitivity makes you more susceptible to magical effects: **subtract your Magic Sensitivity score from your Magic Resistance.** Choosing this Virtue confers the Ability Magic Sensitivity 1 (page 168).

German, ArMDE:4353 (line-parallel):
> Du kannst einen Ort oder Gegenstand häufig als magisch identifizieren. Deine Empfindsamkeit macht dich jedoch anfälliger für magische Effekte: **Ziehe deinen Magiegespür-Wert von deiner Magieresistenz ab.** Das Nehmen dieser Tugend verleiht die Fertigkeit Magiegespür 1.

**Current data:** `"classification": "creation_effect"`, one effect
`{"type": "ability_score_grant", "ability": "ability.magic_sensitivity",
"amount": 1}`. Both locales carry only the *first* sentence as `summary`; no
`description` in either.

**What is right.** The third sentence is computed exactly: A9's grant is a bought-score
floor of 1, `ability.magic_sensitivity` resolves and is `requires_training: true`
and `supernatural`, and `validate_supernatural_abilities` treats a floored
Supernatural Ability as covered so it does not consume the Gift's free slot.

**What is wrong.** The second sentence — the Virtue's *cost*, and the reason it is
Minor rather than Free — is computed by nothing and stated in neither locale.
A37's `MagicResistanceEffect` has six kinds (`no_form_bonus`, `halved_parma`,
`aura_bonus`, `susceptible_faerie`, `susceptible_infernal`,
`conditional_penetration_waiver`) and **none of them subtracts a score** from the
resistance; `derived/casting.rs::magic_resistance` computes
`form_bonus + max(might, parma_for_form)` with no per-character penalty term. So
this is D3's case — structurally inexpressible — which obliges the rule in
`description`, in both locales, and never permits dropping it.

**The mitigation, stated so the severity is not overstated.** The rule does
reach the user by a side door: `rules/i18n/{en,de}/abilities.json` →
`ability.magic_sensitivity.description` says "Often able to identify a place or
object as magical, but the sensitivity subtracts from your Magic Resistance." /
"…doch das Gespür wird von deiner Magieresistenz abgezogen." So a player who
opens the *Ability* sees it. A player who reads the *Virtue* — which is where
the choice is made, and which is where the passage states it — does not.

**Correct value:** a `description` in both locales carrying the subtraction.

**Severity:** lost rule at the point of choice, mitigated by the Ability's text.
Low-moderate.

---

### F-162 — `virtue.magical_memory` — `narrative` on a rule that waives a physical prerequisite

**Passage** (ArMDE:4357, verbatim, English):
> Your memory has been developed to remember magical rather than mundane things. **You need not keep laboratory texts (see page 262) of your creations to get the benefit of a Lab Text when reproducing them. If you have created an effect by following another magus's Lab Text once, you may get the same benefit in future without needing to have the text available.**

German, ArMDE:4357:
> Dein Gedächtnis wurde entwickelt, um magische statt alltägliche Dinge zu behalten. **Du brauchst keine Labortexte (siehe Seite 262) deiner Erschaffungen aufzubewahren, um beim Reproduzieren den Vorteil eines Labtextes zu erhalten. Hast du einen Effekt einmal nach dem Labortext eines anderen Magus erschaffen, kannst du künftig denselben Vorteil erhalten, ohne den Text vorliegen zu haben.**

**Current data:** `"classification": "narrative"`, no effects, no parameters,
`summary` in both locales is the first sentence only.

**Why it is wrong.** Two of the passage's three sentences state a rule. The
cross-reference was followed: ArMDE:10684 gives the benefit being waived —
"A magus who has a Laboratory Text for a particular effect may reproduce it in a
single season if his Lab Total equals or exceeds the level of the effect" —
which more than doubles the speed of adding spells (:10680). The Virtue removes
the *possession* precondition for that benefit, permanently and for two distinct
cases (own creations; another magus's text used once). That is a mechanical
clause, whatever the engine does with it, so `narrative`'s claim that the
passage states nothing mechanical is false.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales carrying both sentences.

**Severity:** lost rule / provenance.

---

### F-163 — `virtue.magical_blood` — permission for a gated Arcane Ability with no `ability_authorization`

**Passage** (ArMDE:4363, verbatim, English — the relevant clause):
> **The character may learn Magic Lore during character creation**, and is resistant to aging, receiving a –1 bonus to all of her Aging rolls.

German, ArMDE:4363:
> **Der Charakter darf beim Erschaffen bereits Magiekunde erlernen** und ist widerstandsfähig gegen das Altern: Er erhält einen Bonus von –1 auf alle Alterungswürfe.

**Current data:** `"classification": "in_play_effect"`, one effect
`{"type": "aging_mod", "kind": "aging_roll", "amount": -1}`. No
`ability_authorization`, no `restricted_ability_xp`.

**Why it is wrong.** `ability.magic_lore` is `"category": "arcane"` and `arcane`
is in `categories_requiring_virtue`, so a companion with Magical Blood and a
score in Magic Lore is told `ability_category_requires_virtue` — the app refuses
the exact character the passage describes. This is the fifth consecutive span in
which the pattern appears, and the second *shape* of it: unlike Jurist or Knight,
the permission here is buried in the middle of a sentence whose other half **is**
computed, so the entry looks implemented.

**What is right, stated so the finding is not read as covering the whole entry.**
The second half of the same sentence is computed correctly. A38 records that
`aging_roll` modifiers are "**added** to the AGING TOTAL with their stored sign",
and a lower Aging Total is the better outcome, so the stored `-1` lowers the
total — which is what "a –1 bonus to all of her Aging rolls" means. Sign ✓,
kind ✓, magnitude ✓.

**Correct value:** `"classification"` stays a computing class (it already carries
an effect), plus
`{"type": "ability_authorization", "abilities": ["ability.magic_lore"]}`. The
passage names one Ability and not a category, so `abilities`, not `categories`.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-164 — `virtue.magical_blood` — all four bloodline types' mechanics reach neither locale (D5)

**Passage** (ArMDE:4363-4371, verbatim, English — the clause and the four types):
> In addition, she receives **a minor physical advantage appropriate to one of the four different types of magic beings** (magic animals, magic humans, magic spirits, and magic things), the one that is associated with the character's background.

> *Magic Animal:* The character has a physical feature normally associated with animals, such as wings, scales, gills, teeth, or claws. **These give as much as a +3 bonus to appropriate activities**, or may allow the character to perform actions that a normal character could not manage (such as fly, or breathe underwater). (:4365)

> *Magic Human:* **The character may increase one of his Characteristics by 1, but not above +3.** … **The character also has a positive Reputation at level 3 among others of his bloodline.** (:4367)

> *Magic Spirit:* The character **gains an appropriate Supernatural Ability (one associated with a Minor Virtue)** … **with an initial score of 1**, though whenever the character uses this Ability, her appearance becomes obviously supernatural. (:4369)

> *Magic Thing:* The character has **a Lesser or Personal Power associated with an object or thing**. (:4371)

German, line-parallel, ArMDE:4365 / :4367 / :4369 / :4371:
> Diese verleihen bis zu +3 auf geeignete Tätigkeiten …

> Der Charakter darf eine seiner Eigenschaften um 1 steigern, jedoch nicht über +3. … Außerdem besitzt der Charakter eine positive Reputation auf Stufe 3 bei anderen Angehörigen seiner Blutlinie.

> Der Charakter erlangt eine passende Übernatürliche Fertigkeit (eine, die mit einer Kleinen Tugend verbunden ist) … mit einem Anfangswert von 1 …

> Der Charakter besitzt eine Mindere oder Persönliche Kraft, die mit einem Objekt oder Ding verbunden ist.

**Current data:** one `aging_mod` effect. Nine of the passage's fourteen lines
state these four variants and **none** of them reaches either locale; both
summaries stop at the first sentence of :4361.

**Why it is a finding.** D5 is explicit: any mechanical clause the engine does
not compute must be written into `description`, in both locales, whatever the
classification. This is the shape D5's own text names —
"`virtue.faerie_blood` computes none of its seven blood types" — recurring on
the Magic-realm twin. Three of the four variants name an engine variant that
*exists* (`ability_bonus` for the +3, `characteristic_score_delta` capped at +3
plus `grants_reputation` score 3, `ability_score_grant` amount 1) and one does
not (a Lesser or Personal Power), but which of them fires depends on a choice the
save cannot record — see F-165 — so the honest answer today is text, not effects.
Whether some should become effects once a parameter exists is Q-51.

**Correct value:** a `description` in both locales carrying all four variants.

**Severity:** lost rule — four distinct mechanics, one of them a +3 in digits.

---

### F-165 — `virtue.magical_blood` — no parameter records which of the four types was taken

**Passage** (ArMDE:4363, verbatim):
> she receives a minor physical advantage appropriate to **one of the four different types of magic beings** (magic animals, magic humans, magic spirits, and magic things), **the one that is associated with the character's background**

**Current data:** `"parameters"` is absent.

**Why it is wrong.** The passage requires the player to pick exactly one of four
named alternatives, and that choice determines which mechanic applies. B8's
`enumerated` domain is precisely this shape: a closed `values` list of four ids,
validated at load, required and non-blank at validation, rendered by the picker
and by the Markdown export. Today the choice is unrecorded, so a saved Magical
Blood character does not say what kind of magic being is in the bloodline, and
neither the sheet nor the export can show it. This is the same shape as B05's
F-139/F-141/F-144/F-157 (four entries demanding a choice the save cannot hold),
and it is the fifth batch in a row to find it.

**Correct value:** a `parameters` entry with `"domain": "enumerated"` and a
four-value list, plus the matching i18n labels.

**Severity:** data loss on round-trip — the player's recorded choice is not
recorded at all.

---

### F-166 — `virtue.magical_mount` — `narrative` on a Might cap, a granted trait and a mandatory Major Story Flaw

**Passage** (ArMDE:4375, verbatim, English):
> The character has a mount, beast-of-burden, or "guard dog," a creature that has **Cunning instead of Intelligence and a Magic Might Score of up to 25**. It has **an extra Personality Trait (Loyal 0)** and will obey simple verbal commands. If it has a positive Cunning score, the creature is also able to perform limited acts on its own initiative. If the character is incapable of training the creature himself, it must have been a gift from another character (probably a powerful magus) and **the character must take a Major Story Flaw** to represent the consequences: Favors, to represent his debt to his patron, is the simplest, but Enemies, if the mount was lured away, is also a possibility. In this case, **only a companion or magus-level character can take this Virtue**.

German, ArMDE:4375:
> … eine Kreatur, die Scharfsinn statt Intelligenz besitzt und **einen Machtwert von bis zu 25**. Sie hat **eine zusätzliche Persönlichkeitseigenschaft (Loyal 0)** … und **der Charakter muss einen Großen Geschichte-Fehler wählen** … In diesem Fall kann **nur ein Gefährte oder ein Charakter auf Magus-Ebene** diese Tugend nehmen.

**Current data:** `"classification": "narrative"`, no effects, no prerequisites;
`summary` in both locales is the first sentence, which already contains "Magic
Might Score of up to 25" / "Machtwert von bis zu 25".

**Why it is wrong.** Four mechanical clauses: a numeric cap on the creature's
Magic Might (25), a fixed granted Personality Trait at a stated value (Loyal 0),
a *mandatory* Major Story Flaw under a stated condition, and a character-type
restriction. `narrative` asserts the passage states nothing mechanical; it
states four things, one of them in digits and already visible in the summary the
entry ships — which is the self-contradiction the `ecb5150` sweep was about.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales carrying all four. None is expressible today: the Might cap belongs
to a creature the app does not model, `flaw_category_caps` on the type profile
cannot *require* a Story Flaw conditionally, and the conditional type
restriction ("in this case") cannot be a flat `entity_kinds` or a profile
`forbidden_categories` row.

**Severity:** lost rule / provenance.

---

### F-167 — `virtue.magical_warder` — `narrative` on a detection rule and a stated range limit

**Passage** (ArMDE:4381, verbatim, English):
> A classic example is the ghost of someone close to the character. **The ghost is invisible and silent to anyone but you, unless they have Second Sight or some other supernatural means of detecting it.** It can see and hear what is going on around you, and **leave your presence for up to half a day**, so it makes a good spy.

German, ArMDE:4381:
> **Das Gespenst ist für jeden unsichtbar und unhörbar außer für dich, es sei denn, jemand besitzt das Zweite Gesicht oder ein anderes übernatürliches Mittel, es zu entdecken.** Es kann sehen und hören, was um dich herum vorgeht, und **deiner Gegenwart bis zu einem halben Tag fernbleiben** …

**Current data:** `"classification": "narrative"`, no effects; `summary` in both
locales is :4379's first sentence.

**Why it is wrong.** Two mechanical clauses: an absolute perception rule with a
named exception (`virtue.second_sight` defeats it), and a hard duration limit on
how long the warder may be away. This is the same shape B03 moved for
`virtue.ghostly_warder` (F-82) — that entry's "once per day" limit is this
entry's "up to half a day". The rest of the entry is genuinely colour (:4379's
"the more powerful the warder, the less willing"; :4383's design guidance), and
that is why this one needs saying explicitly: the entry is *mostly* narrative,
which is how the two clauses survived.

The three pointers were followed and add nothing attributable: "page 457"
(`#geisterwächter`) is an example statistics block, and "Chapter 13, or in
*Realms of Power: Magic*" is the magic-character design subsystem.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales carrying the invisibility rule (with the Second Sight exception) and
the half-day limit.

**Severity:** lost rule / provenance.

---

### F-168 — `virtue.magical_warder` — the German name contradicts the canonical translation table

**Canonical table** (`rules/source/de/translation-tables/tugenden-fehler.md:170`,
verbatim row):

| Englisch (EN) | Deutsch (DE) | Anmerkung |
|---|---|---|
| Magical Warder | **Magischer Wächter** | SdM:M; magisches Wesen als Beschützer; auch für Geisterhaften Wächter mit Machtwert |

**Current data:** `rules/i18n/de/virtues_flaws.json` →
`"virtue.magical_warder": { "name": "Magischer Hüter", … }`.

**Why it is wrong.** CLAUDE.md is explicit that the tables are "the **canonical
EN→DE terminology mapping**: when generating `rules/i18n/de/` text, the German
label for any term whose English form appears in a table MUST match the table's
`Deutsch (DE)` value." "Magical Warder" appears; the value is *Wächter*, not
*Hüter*.

**And the DE rulebook itself sides with the table**, which is what makes this a
clear call rather than a source conflict. The DE heading at ArMDE:4377 reads
`#### Magischer Hüter`, but the *body* of the same entry uses *Wächter* twice:

> Beispielwerte für einen solchen **Magischen Wächter** finden sich auf Seite 457. (ArMDE:4381)

> … ein geisterhafter **Wächter** ist ein gutes Beispiel für einen
> Gefährtencharakter … während die Unterstützung eines mächtigeren **Wächters**
> seltener … ist. (ArMDE:4383)

So the DE source is internally inconsistent, the heading is the outlier, and the
table resolves it. The consistency cost is real: the table row itself notes the
term must also serve "Geisterhaften Wächter" — `virtue.ghostly_warder`, whose
German name the audit should expect to be *Geisterhafter Wächter*; with *Hüter*
on the parent the two entries no longer read as a pair.

**Correct value:** `"name": "Magischer Wächter"`.

**Severity:** localization defect — a real user, every session.

---

### F-169 — `virtue.magister_in_artibus` — both stated Ability prerequisites are missing, and both are expressible

**Passage** (ArMDE:4389, verbatim, English — the relevant sentence):
> You are at least (25 – Int) years old, and **must have scores of at least 5 in Latin and Artes Liberales.**

German, ArMDE:4389:
> Du bist mindestens (25 – Int) Jahre alt und **musst mindestens 5 in Latein und Artes Liberales haben.**

**Current data:** `"prerequisites"` is absent.

**Why it is wrong.** "must have scores of at least 5" is a hard requirement, and
`Prereq::AbilityMin` exists for exactly it. Both Abilities resolve:
`ability.dead_language` (`rules/core/abilities.json` →
`"scholarly_language": {"ability": "ability.dead_language", "exemplar":
"latin"}`, so the book's "Latin" is that id) and `ability.artes_liberales`. B6
records that `ability_min` compares against the max **effective** score
(bought + Puissant) and is never `Unknown`, so the check is total. Today a
companion may take a Major Social Status Virtue whose own text forbids it and the
app says nothing.

This is the shape B05 found twice (F-136 Leper Magus, F-145 License of Absence);
it now appears three times in one batch (this, F-172, F-183).

**Correct value:**

```json
"prerequisites": {
  "kind": "all",
  "value": [
    { "kind": "ability_min", "value": { "ability": "ability.dead_language", "score": 5 } },
    { "kind": "ability_min", "value": { "ability": "ability.artes_liberales", "score": 5 } }
  ]
}
```

The age clause "(25 – Int)" is **not** expressible — `Prereq` has no age or
Characteristic-derived variant — and belongs in the `description` instead
(F-171).

**Severity:** wrong rules output — the app passes an illegal character silently.

---

### F-170 — `virtue.magister_in_artibus` — "only available to male characters" reaches neither locale

**Passage** (ArMDE:4393, verbatim):
> **This Virtue is only available to male characters**, and is compatible with the Hermetic Magus, Mendicant Friar, and Priest Virtues.

German, ArMDE:4393:
> **Diese Tugend ist nur männlichen Charakteren zugänglich** und ist kompatibel mit den Tugenden Hermetischer Magus, Bettelbruder und Priester.

**Current data:** nothing encodes it and neither `summary` mentions it.

**Why it is a finding and not merely an engine gap.** `types.rs`'s concept field
declares `pub gender: String` with the doc comment "The character's gender
(free-text; **no mechanical effect**)", so there is no enum to test and no
`Prereq` variant that could reach it. That is D3's case exactly — structurally
inexpressible is grounds for writing the rule out, **never** for dropping it.
Today it is dropped. This batch repeats B05's F-123 four times over (here,
F-179 Mamluk, F-189 Mazdean Priest, F-192 Mendicant Friar), and the span also
contains the mirror case, `virtue.male_guild_sponsor`'s female-only restriction
(F-177), plus two explicit *non*-restrictions correctly encoded by encoding
nothing (`virtue.magister_in_medicina` ArMDE:4397 "available to female
characters"; `virtue.mercenary_captain` ArMDE:4504 "available to male and female
characters").

**Correct value:** the restriction in `description`, both locales.

**Severity:** lost rule — the app cannot warn, so it must at least say.

---

### F-171 — `virtue.magister_in_artibus` — the age formula, the teaching obligation and the two status interactions reach neither locale (D5)

**Passage** (ArMDE:4387-4391, verbatim, English — what no effect implements):
> You are entitled to be addressed as Magister, **are subject only to canon law, and may teach anywhere in Europe.** (:4387)

> **You are at least (25 – Int) years old** … (:4389)

> **You must spend two seasons teaching to maintain yourself and your reputation as a dependable instructor. These two seasons are spread between September and June, so you are genuinely free in the summer. If you take the Poor Flaw, you are still genuinely free in the summer. If you take the Wealthy Virtue, you can maintain your reputation with a single season's teaching.** (:4391)

German, line-parallel ArMDE:4391:
> **Du musst zwei Quartale mit Unterrichten verbringen, um deinen Lebensunterhalt und deinen Ruf als verlässlicher Lehrer zu wahren. … Wenn du den Fehler Arm nimmst, bist du dennoch im Sommer frei. Wenn du die Tugend Wohlhabend nimmst, kannst du deinen Ruf mit einem einzigen Quartals-Unterricht aufrechterhalten.**

**Current data:** two effects, both correct —
`{"type": "grants_reputation", "kind": "academic", "score": 2}` matches ":4389 You
have an Academic Reputation of 2", and
`{"type": "restricted_ability_xp", "amount": 240, "abilities":
["ability.teaching"], "categories": ["academic"]}` matches ":4389 …a total of 240
additional experience points… must spend your additional experience points on
Academic Abilities or Teaching" (A7's eligibility is an **OR** of `abilities` and
`categories`, which is exactly what "Academic Abilities **or** Teaching" says).
The pool's `academic` category also confers the "may buy Academic Abilities"
permission of the same sentence (A7's permission side-effect), so this entry
correctly needs no `ability_authorization`. Neither locale carries a
`description`.

**Why it is a finding.** Per D5 the obligation follows the rule, not the class.
Four things the effects do not implement: the age formula, the two-season
teaching obligation with its September–June window, and the two *named*
interactions with `flaw.poor` and `virtue.wealthy` — the last of which is a real
rules change (one season instead of two) that a player with Wealthy will never
learn from this app. This is the same shape B05 found on `virtue.landed_noble`
(F-131) and it recurs here and on Muqta' (F-202).

**Correct value:** a `description` in both locales carrying all four.

**Severity:** lost rule — four clauses, one of them a numeric obligation.

---

### F-172 — `virtue.magister_in_medicina` — the imported prerequisites are missing, and three of them are expressible

**Passage** (ArMDE:4397, verbatim, English — the whole entry's mechanical content):
> The character has achieved a doctorate in medicine from one of the medical schools of Europe (Salerno, Cremona, Montpellier, or Bologna), and completed his two years' compulsory teaching. **This Virtue offers the same benefits as Doctor in (Faculty).** This Virtue is compatible with the Hermetic Magus, and Priest Virtues. **Note that this Virtue is available to female characters, although they must have graduated from Salerno**, male characters may have graduated from any of the medical schools.

**What the cross-reference lands on** (`#### Doctor in (Faculty)`, ArMDE:3687,
verbatim):
> A character starting the game with this Virtue **must be at least (27 Intelligence) years old. He must have a score of 5 in Latin. Artes Liberales, and the Ability that correlates to his faculty degree.** The character has spent ten years at a university and receives an additional 300 experience points, which must be spent on Latin and Academic Abilities. He also begins the game with an Academic Reputation of 3.

**Current data:** `"prerequisites"` is absent. The two effects **are** correct
and were verified against :3687 rather than assumed —
`grants_reputation academic 3` ✓ and `restricted_ability_xp 300 categories
["academic"]` ✓ (and the pool's category confers the Academic permission, so no
`ability_authorization` is needed here either).

**Why it is wrong.** "must have a score of 5 in Latin, Artes Liberales, and the
Ability that correlates to his faculty degree" is a hard requirement, and for
*this* Virtue the faculty is fixed — medicine — so all three resolve to concrete
ids: `ability.dead_language`, `ability.artes_liberales`, `ability.medicine`
(`"category": "academic"`, confirmed in `rules/core/abilities.json`). All three
are `Prereq::AbilityMin`. Nothing encodes any of them.

*(`virtue.doctor_in_faculty` itself carries no prerequisites either and has the
same defect. It is ArMDE:3683-3698 — **B02's span**, not this one — so it is
flagged here rather than rated. Its faculty is open, so only two of its three are
expressible.)*

**Correct value:** `Prereq::All` of the three `ability_min` clauses. The age
clause "(27 – Intelligence)" is not expressible and belongs in `description`
(F-173).

**Severity:** wrong rules output — the app passes an illegal character silently.

---

### F-173 — `virtue.magister_in_medicina` — the imported Doctor-in-(Faculty) rules and the Salerno restriction reach neither locale (D5)

**Passage.** The clauses of ArMDE:3687-3689 that "the same benefits as Doctor in
(Faculty)" imports and that no effect implements:
> A character starting the game with this Virtue **must be at least (27 Intelligence) years old.** (:3687)

> Like other working characters, **he must spend two seasons a year practicing his profession, either teaching or working in a secular or ecclesiastical court.** Both the Wealthy Virtue and the Poor Flaw are allowable, but players must decide what calamity befell such an erudite scholar if he is Poor, **for which he receives a Bad Reputation at a level of 2.** (:3689)

plus the entry's own, ArMDE:4397:
> Note that this Virtue is **available to female characters, although they must have graduated from Salerno**, male characters may have graduated from any of the medical schools.

German, ArMDE:4397:
> Beachte: Diese Tugend steht auch weiblichen Charakteren zur Verfügung, obwohl diese nur an der Schule von Salerno graduiert haben müssen …

**Current data:** two effects, both correct; no `description` in either locale;
the summaries are the first sentence only, and neither mentions the
cross-reference at all.

**Why it is a finding.** The entry's own text is four sentences, three of which
are pure setting; its entire mechanical content arrives through one named-Virtue
pointer, which the app never renders. So a player reading this entry in the app
sees a Major Social Status Virtue whose summary explains only that the character
went to medical school. The three imported clauses — the age formula, the
two-season obligation, and the Bad Reputation 2 for a Poor doctor — plus the
Salerno qualification on the (correctly unrestricted) female availability, reach
neither locale.

**Correct value:** a `description` in both locales carrying the four clauses.
This is the case for stating a cross-referenced rule *at the referring entry*:
the pointer is the entry.

**Severity:** lost rule — an entry whose mechanics are invisible in the app.

---

### F-174 — `virtue.major_magical_focus` — the requisite rule, the lab-activity restriction and the breadth rule reach neither locale (D5)

**Passage** (ArMDE:4401-4403, verbatim, English — what no effect implements):
> **This area should be smaller than a single Art, but may be spread over several Arts** — necromancy, for example, covers both Corpus and Mentem effects. **You cannot be focused on laboratory activities, although a focus does apply to laboratory activities.** (:4401)

> **If a spell has requisites, the lowest applicable score may be one of the requisites, rather than one of the primary Arts.** Thus, if a magus with a focus on birds was casting a spell to turn a bird into pure flame, MuAn (Ig), with Muto 14, An 18, and Ig 10, his final total would be 34 + other modifiers: 14 from Muto, and 20 from adding Ignem twice. (:4403)

German, line-parallel ArMDE:4403:
> **Besitzt ein Zauber Requisiten, kann der niedrigste anwendbare Wert einer der Requisiten sein anstatt einer der primären Künste.**

**Current data:** one effect
`{"type": "magical_focus", "param": "focus", "major": true}`, one `text`-domain
parameter, `incompatible_with: ["virtue.minor_magical_focus"]`. Both locales
carry `summary` only.

**What is right.** A29's arithmetic is `min(te, fo)` added once more within the
focus, surfaced as `within_focus` alongside the ordinary figure — which is
":4403 add the lowest applicable Art score twice" over the two primary Arts. The
one-focus limit (:4405) **is** enforced, by `validate_magical_focus` counting
effects over the folded list, which is stronger than the pairwise
`incompatible_with` and catches Mythic Blood's granted focus too. The
`incompatible_with` pair is symmetric (the Minor entry lists this one back) and
load-time `validate_magnitude_variant_exclusivity` would reject it otherwise.

**What is left over.** Three clauses:

1. **The requisite rule.** `min(te, fo)` is computed over the pair's two Arts
   only; the passage says the lowest applicable score "may be one of the
   requisites", and the engine has no requisite model in the focus. Its own
   worked example depends on it — Ignem 10, not Muto 14, is what doubles.
2. **"You cannot be focused on laboratory activities."** A restriction on the
   free-text `focus` value, and C2-b records the descriptor is never consulted,
   so nothing can enforce it — D3's case.
3. **The breadth rule** ("smaller than a single Art, but may be spread over
   several Arts"). This is the *only* mechanical difference between this Virtue
   and its 1-point Minor twin, per the directed read above, and it is stated in
   neither locale on either entry.

**Correct value:** a `description` in both locales carrying all three.

**Severity:** lost rule, and on (3) a genuine user-facing gap: the app offers a
3-point and a 1-point Virtue and explains the difference nowhere.

---

### F-175 — `virtue.maker_of_textured_vessels` — `narrative` on a +3, a Fatigue cost and a shape formula

**Passage** (ArMDE:4425-4427, verbatim, English):
> A character with this Virtue has a repertoire of shapes: **one shape per level in the Craft: Potter Ability.** Each shape corresponds to an Ability … **The character takes one season to learn each added shape** when her Ability increases, and requires the assistance of someone who already knows the new shape desired. (:4425)

> **Crafting a vessel costs one Long-Term Fatigue level. Each vessel grants a +3 bonus in a single Ability.** The materials used while attempting the roll must have been stored inside the vessel. **Storing materials for Abilities in pots does not cause Warping.** The first time a character uses a pot in this way, it accepts the user as its owner, and provides no bonus for anyone else. **A pot is an Arcane Connection to its master. Characters with Magic Resistance cannot be claimed by pots, and gain no benefit from them.** (:4427)

German, ArMDE:4427:
> **Das Herstellen eines Gefäßes kostet eine Langzeit-Erschöpfungsstufe. Jedes Gefäß verleiht +3 auf eine einzelne Fertigkeit.** … **Das Aufbewahren von Materialien für Fertigkeiten in Töpfen verursacht keine Verzerrung.** … **Ein Topf ist eine Arkane Verbindung zu seinem Meister. Charaktere mit Magieresistenz können nicht von Töpfen beansprucht werden und erhalten keinen Nutzen aus ihnen.**

**Current data:** `"classification": "narrative"`, no effects; `summary` in both
locales is :4425's first sentence (the definition of a textured vessel).

**Why it is wrong.** Six mechanical clauses, one of them a bare **+3** and one a
formula (`shapes = Craft: Potter score`). `narrative` asserts the passage states
nothing mechanical.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales. None of it is expressible: the +3 is a bonus from an *item* the
app does not model, not from the Virtue.

**Severity:** lost rule / provenance.

---

### F-176 — `virtue.maker_of_water_vessels` — `narrative` on an Ability swap, a stated duration and a Warping rule

**Passage** (ArMDE:4433-4437, verbatim, English):
> A character who drinks from a magical water vessel made by a character with this Virtue **may swap one Ability score for the Craft: Potter score of the crafter at the time the vessel was made. This effect lasts for one scene or three minutes, whichever is longer.** (:4433)

> **A starting character knows a number of shapes equal to her Craft: Potter score. The character takes one season to learn a new shape** when her Ability increases … **Making a vessel costs a Long-Term Fatigue level.** (:4435)

> **Characters with Magic Resistance cannot be claimed by vessels. The first drink from an unowned vessel, which establishes ownership, causes Warping, unless the drinker has Supernatural Virtues.** Similarly, **drinking from a vessel one does not own causes Warping, unless the drinker has Supernatural Virtues.** (:4437)

German, ArMDE:4433 / :4437:
> … darf einen Fertigkeitswert gegen den Handwerk: Töpfer-Wert des Herstellers … tauschen. **Dieser Effekt hält eine Szene oder drei Minuten an, je nachdem, was länger ist.**

> **Der erste Trunk aus einem unbesessenen Gefäß … verursacht Verzerrung, es sei denn, der Trinker besitzt Übernatürliche Tugenden.**

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** Six mechanical clauses including a score substitution, a
stated duration with a "whichever is longer" tie-break, a shape formula, a
Fatigue cost, and two Warping rules with a stated exception. `narrative` is
false.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales.

**Severity:** lost rule / provenance.

---

### F-177 — `virtue.male_guild_sponsor` — `narrative` on a *mandatory second* Social Status Virtue and a female-only restriction

**Passage** (ArMDE:4441, verbatim, English):
> The character's father or husband is a guild craftsman and she has been allowed entry into his field of work, which is otherwise restricted to men. The character may work at her trade, following the same procedures as the regular male workers. Every guild allows such members, so she may practice any craft she desires. **The character must select a separate guild Social Status Virtue as well as this free Virtue** to represent her status in the guild system. **This Virtue is only available to female characters**, and is compatible with all Social Status Virtues

German, ArMDE:4441:
> **Der Charakter muss zusätzlich zu dieser freien Tugend eine gesonderte Sozialer-Status-Tugend der Gilde wählen**, um seinen Stand im Gildensystem darzustellen. **Diese Tugend ist nur weiblichen Charakteren zugänglich** und ist mit allen Sozialer-Status-Tugenden kompatibel.

**Current data:** `"classification": "narrative"`, `magnitude: "free"`,
`categories: ["social_status"]`, no effects, no prerequisites.

**Why it is wrong.** Two hard rules. The first is the more interesting: this is
the only Virtue in the span that **requires** a second Social Status Virtue, and
it is therefore a stated exemption from ArMDE:2816 ("All characters must take one
Social Status, and may only take more than one if the descriptions of the Virtues
or Flaws explicitly note that they are compatible") in the *positive* direction —
one Social Status is not merely permitted here, it is illegal. Nothing encodes
it and neither summary states it, so a player can build a Male Guild Sponsor with
no guild at all. The second is the mirror of F-170's male-only case, D3's ground
again.

`Prereq` cannot express "has any item in category X" — there is no category
variant, only `Has(Id)` — so an enumeration of every guild Social Status Virtue
would be the only encoding, which is brittle and would rot as the catalogue
grows. D3 therefore gives `uncomputed_rule` with the rule written out.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales carrying both rules.

**Severity:** lost rule, and the more consequential half of Q-45.

---

### F-178 — `virtue.mamluk` — Martial permission and the Theology: Islam exception, with no `ability_authorization`

**Passage** (ArMDE:4447, verbatim, English):
> **You may take Martial Abilities at character creation, and as a special case you may also take the Ability Theology: Islam, even if you do not have the Minor General Virtue Educated.** This Virtue is only available to male characters, and (for Companion characters only) is compatible with both the Emir and Muqta' Virtues, as many Mamluks have climbed high in the Muslim hierarchy.

German, ArMDE:4447:
> **Du darfst beim Erschaffen Kampffertigkeiten nehmen, und als Sonderfall darfst du auch die Fertigkeit Theologie: Islam nehmen, selbst wenn du nicht die Kleine Allgemeine Tugend Gebildet besitzt.**

**Current data:** `"classification": "narrative"`, no effects, no parameters.

**Why it is wrong.** Both permissions are refused today. `martial` is in
`categories_requiring_virtue`, and `ability.theology_islam` is
`"category": "academic"`, likewise gated — the passage even *says* it is an
exception to the normal gate ("even if you do not have … Educated"), which is the
clearest possible statement that an authorization is what is being granted. A
Mamluk with a Bow score or a Theology: Islam score is told
`ability_category_requires_virtue`.

Note the scope difference between the two halves and why the encoding must
mirror it: the passage grants the **whole martial category** ("Martial
Abilities") but **one named Academic Ability** ("the Ability Theology: Islam"),
not the Academic category. A14's variant carries both lists, so both go in one
row.

**Correct value:** `"classification": "creation_effect"`, plus

```json
{ "type": "ability_authorization",
  "categories": ["martial"],
  "abilities": ["ability.theology_islam"] }
```

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-179 — `virtue.mamluk` — "only available to male characters" reaches neither locale

**Passage** (ArMDE:4447, verbatim):
> **This Virtue is only available to male characters**, and (for Companion characters only) is compatible with both the Emir and Muqta' Virtues …

German, ArMDE:4447:
> **Diese Tugend ist nur männlichen Charakteren zugänglich** und ist (ausschließlich für Gefährten) kompatibel sowohl mit der Emir- als auch der Muqta'-Tugend …

**Current data:** nothing encodes it; neither `summary` mentions it.

**Why it is a finding.** Identical ground to F-170 — `types.rs`'s `pub gender:
String` is documented as having no mechanical effect, so D3 obliges the text.
Note the second half is a *doubly* qualified compatibility ("for Companion
characters only", with two named Virtues), which is beyond what any
`virtue_category_caps` row could express — see Q-45.

**Correct value:** the restriction in `description`, both locales.

**Severity:** lost rule.

---

### F-180 — `virtue.marshal` — "may take Martial Abilities freely" with no `ability_authorization`

**Passage** (ArMDE:4453, verbatim, English):
> A marshal receives 50 extra experience points at character generation to spend on the abilities Animal Handling, Etiquette, Hunt, Latin, Profession: Marshal and Ride, and **may take Martial Abilities freely.**

German, ArMDE:4453:
> Ein Marschall erhält beim Erschaffen 50 zusätzliche Erfahrungspunkte, die er auf die Fertigkeiten Tierumgang, Etikette, Jagen, Latein, Beruf: Marschall und Reiten aufteilen darf, und **kann Kampffertigkeiten frei nehmen.**

**Current data:** `"classification": "creation_effect"`, one effect
`{"type": "restricted_ability_xp", "amount": 50, "abilities":
["ability.animal_handling", "ability.dead_language", "ability.etiquette",
"ability.hunt", "ability.profession", "ability.ride"]}`.

**What is right.** The 50 points and all six named Abilities are correct,
including `ability.dead_language` for "Latin" and `ability.profession` for
"Profession: Marshal".

**Why it is wrong.** A7's permission side-effect covers only what the pool
funds — those six ids — and **Martial Abilities are not among them**. The
passage grants the martial category *separately from* the XP, and it grants it
"freely", i.e. as a bare permission with no funding attached. That is precisely
A14's stated purpose ("this variant is only needed for a Virtue that permits
*without* funding"). Nothing encodes it, so a marshal with a Single Weapon score
is refused.

This is the second shape of the open pattern: an effect **is** present and
carries `abilities`, while the passage separately grants a whole `categories`
group.

**Correct value:** add
`{"type": "ability_authorization", "categories": ["martial"]}` alongside the
existing pool.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-181 — `virtue.marshal` — the Profession-functions-as-Medicine rule reaches neither locale (D5)

**Passage** (ArMDE:4453, verbatim):
> A marshal should take the Ability Profession: Marshal, which deals with understanding, purchasing, and caring for horses. **It functions as the Ability Medicine for the purpose of treating veterinary diseases, and for surgery involving these animals.**

German, ArMDE:4453:
> Sie funktioniert wie die Fertigkeit Medizin für die Behandlung tierärztlicher Krankheiten und für chirurgische Eingriffe bei diesen Tieren.

**Current data:** one `restricted_ability_xp` effect; no `description` in either
locale; both summaries are :4451's first sentence, which is pure setting.

**Why it is a finding.** Per D5 the obligation follows the rule. This is an
Ability-substitution rule — one Ability standing in for another in a named
context — and no `Effect` variant expresses it (`ability_bonus` adds a bonus,
`restricted_ability_xp` funds, neither substitutes). It is the entry's only
in-play mechanic and it reaches nobody.

**Correct value:** a `description` in both locales carrying it.

**Severity:** lost rule.

---

### F-182 — `virtue.master_bard` — "Arcane Abilities" is authorized as two named ids

**Passage** (ArMDE:4461, verbatim, English):
> **You can spend experience points on Arcane Abilities at character creation**, and have an extra 240 experience points to spend on Art of Memory, Profession: Storyteller, Profession: Poet, any Area Lore, any Organization Lore, Faerie Lore, or Magic Lore.

German, ArMDE:4461:
> **Du kannst beim Erschaffen Erfahrungspunkte für Arkane Fertigkeiten ausgeben** und hast 240 zusätzliche Erfahrungspunkte für Gedächtniskunst, Beruf: Geschichtenerzähler, Beruf: Dichter, beliebige Gebiets-Kunde, beliebige Organisations-Kunde, Feenkunde oder Magiekunde.

**Current data:** `{"type": "restricted_ability_xp", "amount": 240, "abilities":
["ability.area_lore", "ability.art_of_memory", "ability.faerie_lore",
"ability.magic_lore", "ability.organization_lore", "ability.profession"]}` plus
`{"type": "grants_reputation", "kind": "local", "score": 3}`.

**What is right.** The 240 points and all six listed ids are correct against the
passage's seven named targets (Profession covers both Storyteller and Poet), and
the Local Reputation 3 matches ":4459 either way this earns him a Local
Reputation at level 3" — confirmed independently against
`rules/source/de/translation-tables/reputationen.md:89`
("Senior Bard (Anruth) | Meisterbarde (Anruth) | Lokal | 3 (+)").

**Why it is wrong.** The permission and the funding are **two separate clauses**,
and only the funding is encoded. A7's side-effect authorizes exactly the six
funded ids, of which two are arcane (`ability.faerie_lore`,
`ability.magic_lore`). But the passage's first clause grants the whole `arcane`
category — which also contains `ability.code_of_hermes`,
`ability.dominion_lore`, `ability.enigmatic_wisdom`, `ability.finesse`,
`ability.infernal_lore`, `ability.magic_theory`, `ability.parma_magica`,
`ability.penetration` (the full list was read from
`rules/core/abilities.json`). A bard with an Infernal Lore or Dominion Lore
score is refused, though ":4461 You can spend experience points on Arcane
Abilities at character creation" permits it.

This is the same shape as F-180 on a different category, and it is the reason the
open pattern needs both of its shapes checked: the entry *has* an effect, and the
effect is *correct for what it funds*.

**Correct value:** add
`{"type": "ability_authorization", "categories": ["arcane"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-183 — `virtue.master_bard` — the stated Ability prerequisites are missing, and the four-way one is fully expressible

**Passage** (ArMDE:4461, verbatim):
> The character should be at least 25 years old, and **must have a Profession: Storyteller or Poet of at least 5, and also a 5 in at least one of Area Lore, Organization Lore, Faerie Lore, or Magic Lore.**

German, ArMDE:4461:
> Der Charakter sollte mindestens 25 Jahre alt sein und **muss mindestens 5 in Beruf: Geschichtenerzähler oder Dichter haben sowie 5 in mindestens einer der Fertigkeiten Gebiets-Kunde, Organisations-Kunde, Feenkunde oder Magiekunde.**

**Current data:** `"prerequisites"` is absent.

**Why it is wrong.** The second requirement is *fully* expressible and simply not
encoded:

```json
{ "kind": "any", "value": [
  { "kind": "ability_min", "value": { "ability": "ability.area_lore", "score": 5 } },
  { "kind": "ability_min", "value": { "ability": "ability.organization_lore", "score": 5 } },
  { "kind": "ability_min", "value": { "ability": "ability.faerie_lore", "score": 5 } },
  { "kind": "ability_min", "value": { "ability": "ability.magic_lore", "score": 5 } }
] }
```

B6 records that `any` short-circuits True on a True child and is never `Unknown`
for `ability_min`, so this is exact. The first requirement is *partly*
expressible: `ability.profession` is parameterized (`"parameter": "profession"`
in `rules/core/abilities.json`) and `Prereq::AbilityMin` names an Ability with no
instance, so `{"ability": "ability.profession", "score": 5}` would enforce
"a Profession at 5" but not "Storyteller or Poet" specifically — stricter than
nothing, looser than the book. Note "should be at least 25 years old" is a
*soft* recommendation ("should" / "sollte"), unlike Magister in Artibus's flat
"You are at least (25 – Int) years old", so it is correctly encoded by encoding
nothing.

**Correct value:** the `any`-of-four clause at minimum; the Profession clause is
a judgement call the correction pass should take with Q-47 in view.

**Severity:** wrong rules output — the app passes an illegal character silently.

---

### F-184 — `virtue.master_bard` — the two-season obligation and the Ireland-only restriction reach neither locale (D5)

**Passage** (ArMDE:4459, :4461, verbatim):
> **If resident at a school, he has an obligation to teach for at least two seasons a year; if he works for a lord, two seasons are instead spent composing praise poems and stories for his patron.** (:4459)

> **This Social Status is only available in Ireland.** (:4461)

German, ArMDE:4459 / :4461:
> **Wenn er an einer Schule tätig ist, ist er verpflichtet, mindestens zwei Quartale im Jahr zu unterrichten; wenn er für einen Herrn arbeitet, verbringt er stattdessen zwei Quartale damit, Lobgedichte und Geschichten für seinen Gönner zu verfassen.**

> **Dieser Soziale Status ist nur in Irland verfügbar.**

**Current data:** two effects; no `description` in either locale.

**Why it is a finding.** Per D5. The season obligation is a hard cost on the
character's year — the same clause B05 found unstated on Landed Noble (F-131) and
this batch finds unstated on Magister in Artibus (F-171) and Muqta' (F-202). The
Ireland restriction is a saga-setting gate with no engine expression.

**Correct value:** a `description` in both locales carrying both.

**Severity:** lost rule.

---

### F-185 — `virtue.master_of_form_creatures` — permission for a gated Arcane Ability with no `ability_authorization`

**Passage** (ArMDE:4465, verbatim, English):
> The character can tame animals and other unintelligent beings whose Magic Might is aligned with a particular Form. **During character creation, the character may take Magic Lore**, and this Virtue may be taken multiple times, once for each Form. See page 384 for rules for training magical (and mundane) animals.

German, ArMDE:4465:
> **Beim Erschaffen darf der Charakter Magiekunde nehmen**; diese Tugend darf mehrfach gewählt werden, einmal für jede Form.

**Current data:** `"classification": "narrative"`, no effects, one `form`-domain
parameter.

**Why it is wrong.** `ability.magic_lore` is `arcane` and gated, so the app
refuses a character built exactly as the passage describes. Same ground as
F-163, and the two together make the point that the shape is not confined to
Social Status Virtues.

**What is right, and is worth recording because it is easy to misread.** The
multiplicity clause is encoded **correctly by default**, and no change is needed:
`max_per_target` is absent, so B10's default of **1** applies to the
`(item_ref, whole params map)` tuple — and because the item declares a `form`
parameter, each Form is a distinct tuple. So ten Forms are legal and a second
copy naming the same Form is rejected, which is exactly ":4465 once for each
Form". `max_total` is likewise absent (sentinel 255, no ceiling), correct because
the book states none.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "abilities": ["ability.magic_lore"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-186 — `virtue.master_of_kennels` — "may take Martial Abilities freely" with no `ability_authorization`

**Passage** (ArMDE:4469, verbatim, English — the relevant sentence):
> A master of kennels receives 50 extra experience points at character generation to spend on the abilities Animal Handling, Etiquette, Hunt, Latin, Profession: Master of Kennels, and Ride, and **may take Martial Abilities freely.**

German, ArMDE:4469:
> Ein Meister der Hundezwinger erhält beim Erschaffen 50 zusätzliche Erfahrungspunkte … und **kann Kampffertigkeiten frei nehmen.**

**Current data:** one `restricted_ability_xp` effect, 50 points over the same six
ids as Marshal's — correct against the passage's six named Abilities.

**Why it is wrong.** Identical to F-180: the pool authorizes only what it funds,
and martial is granted separately and unfunded. The two entries are twins in the
book (the same sentence pattern, the same six Abilities, the same free martial
permission) and they are twins in this defect.

**Correct value:** add
`{"type": "ability_authorization", "categories": ["martial"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

**On the rest of the passage**, so it is clear D5 was checked and found nothing:
":4469 He should possess the Ability Profession: Master of Kennels, which governs
the care of the dogs…" and "The Ability Animal Handling is used for the training
of the animals" are guidance ("should") and description of what an Ability
covers, not rules the entry adds. Unlike Marshal, there is no
Ability-substitution clause here. So this entry draws one finding, not two.

---

### F-187 — `virtue.masterpiece` — the lesser-item restriction and the vis waiver reach neither locale (D5)

**Passage** (ArMDE:4478, verbatim, English):
> For some benevolent reason, the magus's parens has allowed him to keep the lesser enchanted item he made to prove himself a magus and pass his Gauntlet. **This masterpiece must be a lesser enchanted item.** You should design a lesser enchanted item that your character could make **based on his Lab Totals at character generation**, following the regular rules for construction of such a device. **You ignore vis costs, as the magus's parens provided those from her laboratory stores.**

German, ArMDE:4478:
> **Dieses Meisterstück muss ein Schlichtes Artefakt sein.** Entwirf ein Schlichtes Artefakt, das dein Charakter anhand seiner Laborsummen beim Erschaffen herstellen könnte … **Vis-Kosten entfallen, da der Parens des Magus diese aus seinen Laborvorräten bereitgestellt hat.**

**Current data:** one effect `{"type": "masterpiece_item"}`; no `description` in
either locale.

**What is right.** A20: the marker makes `derived/lab.rs::masterpiece_item_cap`
report `halve(best LabTotal::enchanting)` — the "based on his Lab Totals at
character generation" clause, computed.

**What is left over.** Two clauses. "must be a lesser enchanted item" is a
restriction on what the player may design that nothing checks (A20: "no device
is created, no budget spent"), and "You ignore vis costs" is an explicit waiver
of a cost the app does not model — so a player has no way to know the waiver
exists, and no way to know the item is exempt from whatever vis accounting a
troupe does by hand.

**Correct value:** a `description` in both locales carrying both.

**Severity:** lost rule.

---

### F-188 — `virtue.mazdean_priest` — Academic permission with no `ability_authorization`

**Passage** (ArMDE:4486, verbatim, English):
> **Your character may take Academic Abilities at character generation.** Most such characters take the Minor Personality Flaw Vow to reflect your constant state of ritual purity, and dedication to your community.

German, ArMDE:4486:
> **Dein Charakter darf beim Erschaffen Akademische Fertigkeiten nehmen.**

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** `academic` is in `categories_requiring_virtue`, so a Mazdean
priest with an Artes Liberales or Theology score is refused. Here the passage
names the **category**, not a list of Abilities, so the encoding takes
`categories`.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "categories": ["academic"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

**What is correctly *not* encoded**, recorded so the correction pass does not
over-reach: ":4484 If you are Wealthy, you have several ervads … If you are Poor,
it likely reflects the poor station of Mazdeans … and you may also possess the
Outsider Flaw" is soft throughout ("likely", "may"), and ":4486 Most such
characters take the Minor Personality Flaw Vow" is a suggestion. Neither is a
rule. The setting gate ("can only be your character's social status if he lives
in a Mazdean community") is likewise not an engine concept.

---

### F-189 — `virtue.mazdean_priest` — "only available to male characters" reaches neither locale

**Passage** (ArMDE:4486, verbatim):
> **This Virtue is only available to male characters.**

German, ArMDE:4486:
> **Diese Tugend ist nur männlichen Charakteren zugänglich.**

**Current data:** nothing encodes it; neither `summary` mentions it.

**Why it is a finding.** D3, as F-170.

**Correct value:** the restriction in `description`, both locales.

**Severity:** lost rule.

---

### F-190 — `virtue.mendicant_friar` — Academic permission with no `ability_authorization`

**Passage** (ArMDE:4492, verbatim, English):
> **Due to your training, you may take Academic Abilities during character generation.** If you wish, you may be an ordained priest and may officiate at marriages, baptisms, funerals, and the Mass …

German, ArMDE:4492:
> **Aufgrund deiner Ausbildung darfst du beim Erschaffen Akademische Fertigkeiten nehmen.**

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** Identical to F-188: `academic` is gated and a friar with a
Latin score is refused. Note the friar is explicitly a cleric subject to canon
law (:4490), i.e. exactly a literate character.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "categories": ["academic"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-191 — `virtue.mendicant_friar` — "You may not take the Wealthy Virtue or Poor Flaw" is expressible in `incompatible_with` and absent

**Passage** (ArMDE:4494, verbatim, English):
> You have sworn vows of poverty, chastity, and obedience, which could together constitute a Major Story Flaw (Monastic Vows, see page 138), and which would be a natural choice if you take this Virtue. **You may not take the Wealthy Virtue or Poor Flaw.** This Virtue is only available to male characters, and is compatible with the Magister in Artibus Major Virtue.

German, ArMDE:4494:
> **Du darfst weder die Tugend Wohlhabend noch den Fehler Arm nehmen.**

**Current data:** `"incompatible_with"` is absent.

**Why it is wrong.** This is a flat, unconditional prohibition on two named
items, and `incompatible_with` is the field for exactly it: B7 records
`validation/prereq.rs::validate_incompatibilities` as the runtime reader, raising
an `incompatible` **error** with the pair order normalised so a mutual
declaration reports once. Both targets exist in the catalogue. Today a friar may
take Wealthy and nothing objects.

Note the surrounding sentences are correctly *not* encoded and the contrast is
what makes the middle one unmistakable: the Monastic Vows clause is a suggestion
("could together constitute", "would be a natural choice"), and the Magister in
Artibus clause is a compatibility note. Between them sits a "may not" with no
hedge at all.

**Correct value:** `"incompatible_with": ["flaw.poor", "virtue.wealthy"]` on this
entry, **and the reciprocal entries on `flaw.poor` and `virtue.wealthy`** —
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the whole load on
a one-sided declaration, so the correction is three edits, not one.

**One scope caveat the correction pass must know.** B7 records that
`validate_incompatibilities` iterates **bought** selections only
(`selected_ids`), by a documented decision (review finding B1), so this would not
catch a *granted* Poor or Wealthy. That is the recorded engine scope, not a
defect of this fix.

**Severity:** wrong rules output — the app permits an illegal character.

---

### F-192 — `virtue.mendicant_friar` — "only available to male characters" reaches neither locale

**Passage** (ArMDE:4494, verbatim):
> **This Virtue is only available to male characters**, and is compatible with the Magister in Artibus Major Virtue.

German, ArMDE:4494:
> **Diese Tugend ist nur männlichen Charakteren zugänglich** und ist kompatibel mit der Großen Tugend Magister in Artibus.

**Current data:** nothing encodes it; neither `summary` mentions it.

**Why it is a finding.** D3, as F-170. This is the fourth instance in this batch
and the fifth in the audit; the pattern is now large enough to state once as a
catalogue-wide item.

**Correct value:** the restriction in `description`, both locales.

**Severity:** lost rule.

---

### F-193 — `virtue.mendicant_friar` — the English summary is truncated mid-abbreviation

**Current data:**

```json
"virtue.mendicant_friar": {
  "name": "Mendicant Friar",
  "summary": "You are a follower of St."
}
```

**Passage** (ArMDE:4490, verbatim — the sentence the summary is drawn from):
> You are a follower of **St. Francis or St. Dominic** going among the rich and poor, spreading the word of God and giving comfort to the sick, homeless, hungry, or dying.

**Why it is wrong.** The summary stops at the abbreviation's full stop, so the
app shows a sentence fragment that says nothing — "You are a follower of St."
This is a first-sentence extraction that split on `St.` as a sentence boundary.
It is **English-only**: the German summary is the complete sentence
("Du bist ein Anhänger des heiligen Franziskus oder des heiligen Dominikus,
unterwegs unter Reichen und Armen, …"), which is both the proof that the source
sentence is longer and the reason the defect is invisible to a
both-locales-differ check.

*(Every other `summary` in the batch was read in full against its passage; this
is the only truncation. Two others are short but complete sentences —
`virtue.merchant` "You live from the buying and selling of goods." and
`virtue.merchant_adventurer` "The character is in command of a ship, and a
crew." — and both match their source exactly.)*

**Correct value:** the full first sentence of ArMDE:4490.

**Severity:** localization / display defect, English locale, visible on every
view of the entry.

---

### F-194 — `virtue.mentored_by_demons` — the age-cap waiver is modelled nowhere, and the app errors on the character the passage describes

**Passage** (ArMDE:4498, verbatim, English):
> A character mentored by demons learns faster than is possible for those studying with human teachers, but demons only teach those Abilities that suit their plans for the character. **Characters trained by demons may exceed the maximum skill level for a given age provided by the character creation rules.** Students of demons may also have Abilities that are usually restricted to suitable backgrounds. … **Mentored characters have an additional 50 experience points to spend on any Ability. Characters may purchase this Virtue multiple times, and gain 50 further experience points each time.**

German, ArMDE:4498:
> **Von Dämonen ausgebildete Charaktere dürfen das in den Charaktererschaffungsregeln festgelegte maximale Fertigkeitsniveau für ein gegebenes Alter überschreiten.** … **Betreute Charaktere haben zusätzliche 50 Erfahrungspunkte für eine beliebige Fertigkeit. Charaktere können diese Tugend mehrfach kaufen und erhalten jedes Mal 50 weitere Erfahrungspunkte.**

**Current data:** `"classification": "creation_effect"`, `"tainted": true`,
`"max_per_target": 255`, one effect
`{"type": "restricted_ability_xp", "amount": 50, "categories": ["academic",
"arcane", "general", "martial", "supernatural"]}`.

**What is right, and was verified rather than assumed.** The five listed
categories are **all** of them —
`jq '[.abilities[].category] | unique'` over `rules/core/abilities.json` returns
exactly `["academic","arcane","general","martial","supernatural"]` — so
"any Ability" is faithfully encoded. A7's permission side-effect then covers
":4498 Students of demons may also have Abilities that are usually restricted to
suitable backgrounds", which is why this entry draws no authorization finding.
The pools stack (A7), matching "gain 50 further experience points each time", and
`max_per_target: 255` matches "may purchase this Virtue multiple times" under
ArMDE:2814's rule that repeats need explicit permission.

**Why it is wrong.** The second sentence waives the age→Ability-score cap, and
the engine enforces that cap as a hard **error**.
`validation/scores.rs::validate_ability_age_cap` was read in full: it takes the
ruleset's age-band cap, widens it by **+2 only** when
`effective::ability_affinity` finds an Affinity, and otherwise pushes
`ValidationIssue::error(CODE_ABILITY_ABOVE_AGE_CAP, …)`. There is no hook for a
waiver, and this was established by reading the cap's own producer rather than
by an absent-grep: `effective/reputation_and_caps.rs::ability_age_cap` takes the
ruleset's age-band figure and then folds **exactly one** effect variant over it,
`Effect::LocalityAbilityCapFraction`, combined as `cap = cap.min(fractioned)` —
so the only V/F influence on the cap can *narrow* it (Foreign Upbringing's
halving, ArMDE:6160) and none can widen it. `ability_authorizations`, the only
thing this entry's pool feeds, gates *ownership*, not *score*. So a
character built exactly as this passage describes — a young prodigy with an
above-band score — is rejected under `ValidationMode::Enforced`, which is both
guided and direct-validated mode.

And because the entry is `creation_effect` with an effect, nothing obliges the
waiver to appear in `description` today; it appears in neither locale, so the
player is not even told why the app is refusing.

**Correct value:** the waiver written into `description` in both locales at
minimum (D5/D3 — the engine structurally cannot express a per-character cap
lift). Whether `validate_ability_age_cap` should grow a waiver hook is an engine
change and a Phase 2 call, not a data fix.

**Severity:** **wrong rules output** — the app refuses a legal character and
explains nothing. The highest-severity finding in this batch.

---

### F-195 — `virtue.mercenary_captain` — Martial permission with no `ability_authorization`

**Passage** (ArMDE:4502, verbatim, English):
> You lead a small company of mercenaries (5 to 10), for hire to the highest bidder. You are much like a knight-errant, only without the prestige. During your travels you have gained great wealth — and squandered it — several times over. **You may take Martial Abilities during character generation.**

German, ArMDE:4502:
> **Du darfst beim Erschaffen Kampffertigkeiten nehmen.**

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** `martial` is gated; a mercenary captain with a weapon score
is refused. This is the most on-the-nose instance in the whole pattern — a
Virtue whose entire concept is commanding soldiers, unable to hold a martial
Ability.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "categories": ["martial"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

**On the rest of the passage.** ":4504 If you are Poor, you lead only a couple of
other mercenaries … If you are Wealthy you lead about twenty mercenaries, and can
delegate some of the work to sergeants" changes the size of a retinue the app
does not model, and ":4504 This Virtue is available to male and female
characters" is an explicit *non*-restriction correctly encoded by encoding
nothing. Neither adds a finding.

---

### F-196 — `virtue.mercurian_magic` — the required companion Flaw is encoded nowhere

**Passage** (ArMDE:4522, verbatim, English):
> **All known members of the Mercurian lineage also have the Minor Flaw Ceremonial Spontaneous Magic.**

German, ArMDE:4522:
> **Alle bekannten Mitglieder des merkurischen Stammbaums haben außerdem den Kleinen Fehler Zeremonielle Spontane Magie.**

**Current data:** no `prerequisites`, no `grants_selection`, no
`incompatible_with`. The named Flaw exists and matches:
`flaw.ceremonial_spontaneous_magic`, `"kind": "flaw"`, `"magnitude": "minor"`.

**Why it is wrong.** The sentence is unqualified — "**All** known members … also
have" — so a Mercurian magus without Ceremonial Spontaneous Magic is not a legal
character, and the app builds one silently. The relationship is expressible:
`"prerequisites": { "kind": "has", "value": "flaw.ceremonial_spontaneous_magic" }`
makes `validate_prerequisites` raise `prereq_not_met` (an error) when the Flaw is
absent, and B6 records that `has` reads the **grants-inclusive** `present_ids`
set, so a House- or type-granted copy would satisfy it too.

**Why this is not simply `grants_selection`, and why that is a question rather
than a verdict.** A18 records that granted rows are **budget-exempt**. But this
passage does not say "free" or "at no extra cost" — contrast ArMDE:4588, one
entry later, which says exactly that of Mythic Blood's inclusions. So granting
the Flaw would silently deny the player the Flaw point the book appears to give
them. `Prereq::Has` keeps the point and enforces the pairing; that is the reading
this finding proposes, and the residual doubt is **Q-48**.

**Correct value:** `"prerequisites": { "kind": "has", "value":
"flaw.ceremonial_spontaneous_magic" }`, subject to Q-48.

**Severity:** wrong rules output — the app permits an illegal character.

---

### F-197 — `virtue.mercurian_magic` — Wizard's Vigil, the Mastery addition and the half-vis rule reach neither locale (D5)

**Passage** (ArMDE:4516-4520, verbatim, English — what no effect implements):
> In addition to your standard spell allocation, **you also know Wizard's Vigil (page 370) at a level equal to the highest level of Ritual spell that you know, and should you invent or learn a Ritual spell of higher level, you automatically invent a Wizard's Vigil spell of the same level, without needing to spend extra time.** (:4516)

> When casting a spell using Wizard's Vigil … **you may add your Mastery score (page 225) in the spell being cast and your Mastery score in Wizard's Vigil to the effective level of the Wizard's Vigil.** (:4518)

> Finally, **any Ritual spells which you cast have only half the usual vis requirement. If cast as part of a Wizard's Vigil, all the participants need to have this Virtue to gain this benefit.** (:4520)

German, line-parallel ArMDE:4520:
> **Schließlich haben alle Ritualssprüche, die du wirkst, nur die Hälfte des üblichen Vis-Bedarfs. Falls sie als Teil eines Zaubervigils gewirkt werden, müssen alle Teilnehmer diese Tugend besitzen, um diesen Vorteil zu erlangen.**

**Current data:** one effect
`{"type": "special_casting_mod", "kind": "mercurian"}` — which C1 records as one
of the eight surfaced-only `SpecialCasting` kinds, pushed with `amount: 0`. That
is the Part C gap and is **not** re-reported here. No `description` in either
locale.

**Why it is a finding anyway.** D5 obliges the clauses the effects do not
implement, and here that is all three — which is to say the entire Virtue. The
first is structurally inexpressible: the cross-reference was followed to
`##### Wizard's Vigil` (ArMDE:15806-15808) and confirms it is a **named spell**,
while A10's `spell_levels` moves only a *budget* and no variant grants a named
spell at a computed level. The second and third are in-play arithmetic
(a Mastery-derived addition to an effective level; a halved vis cost) that the
app models nowhere. The half-vis rule in particular is one of the strongest
mechanical benefits in the core rulebook's Hermetic Virtues, and it currently
reaches the player in neither language.

**Correct value:** a `description` in both locales carrying all three clauses,
including the "all the participants need to have this Virtue" condition on the
third.

**Severity:** lost rule — a 3-point Major Virtue whose entire mechanical content
is invisible.

---

### F-198 — `virtue.method_caster` — the "if you vary at all" gate is folded in flat and stated nowhere (D5)

**Passage** (ArMDE:4526, verbatim, English):
> You are excellent at formulaic spells, as you have perfected a consistent and precise method for casting them. **You gain a +3 bonus to the Casting Total of any Formulaic or Ritual spell you cast. However, if you vary at all from your precise method (by altering your gestures or voicing), you do not get this bonus.**

German, ArMDE:4526:
> **Du erhältst +3 auf die Zaubersumme jedes formulaischen oder rituellen Zaubers, den du wirkst. Weichst du jedoch auch nur geringfügig von deiner präzisen Methode ab (indem du Gesten oder Aussprache veränderst), erhältst du diesen Bonus nicht.**

**Current data:** one effect
`{"type": "casting_total_mod", "amount": 3, "scope": "formulaic_ritual"}`; no
`description` in either locale.

**What is right.** Amount ✓, sign ✓, and the scope is exactly right: A30 records
that `formulaic_ritual` is accepted by both Formulaic and Ritual `CastType`s and
by neither Spontaneous, which is "any Formulaic or Ritual spell" precisely.

**Why it is a finding.** The second sentence is a condition under which the
bonus does not apply, and A30 records that the engine applies a
`casting_total_mod` **unconditionally** to every matching cell. So the +3 is
printed into every Formulaic and Ritual Casting Total with no note that altering
gestures or voicing removes it — and altering gestures or voicing is a routine
in-play choice (casting quietly, casting while restrained), not an exotic one.

**This is a milder case than B05's F-149 and the difference is stated so the
correction pass rates it right.** Lightning Reflexes folds a *triply*-conditional
+9 in flat, so the printed number is wrong most of the time. Here the condition
is an *exception* to a default — you get the bonus unless you deviate — so the
flat fold is the common case and the arithmetic as printed is usually correct.
What is missing is the exception, and D5 puts it in `description`, not in a
conditionality subsystem.

**Correct value:** a `description` in both locales carrying the second sentence.

**Severity:** lost rule; the printed total is right by default.

---

### F-199 — `virtue.minor_enchantments` — a 25-level item allowance the engine can already express, authored as `narrative` with no effect

**Passage** (ArMDE:4534, verbatim, English):
> The character has one or more items in his possession that have magical powers. These should be designed as Hermetic enchantments, and **the total levels of powers in all the items, after adjustment for uses per day and so forth, must be 25 or less.** The character may take this Virtue more than once: **add the total levels together, but no single power can be greater than 30th level.** If he loses the item or it is destroyed, then it is gone for good.

German, ArMDE:4534:
> … und **die Gesamtstufen der Kräfte aller Gegenstände, nach Anpassung für Nutzungen pro Tag und dergleichen, dürfen 25 nicht übersteigen.** Der Charakter darf diese Tugend mehr als einmal nehmen: **Addiere die Gesamtstufen, aber keine einzelne Kraft darf höher als Stufe 30 sein.**

**Current data:** `"classification": "narrative"`, **no effects**,
`"max_per_target": 255`.

**Why it is wrong, and why this one is not a judgement call.** The engine has a
variant for exactly this number and two neighbours already use it. A19:
`item_level_budget` is summed by
`effective/gift_confidence.rs::item_level_budget`, compared against
`item_level_used = Σ device.level` by `validation/might.rs::validate_devices`,
and surfaced as a budget bar — with **"Usage 2 — `virtue.magic_items`,
`virtue.redcap`"**. And `virtue.magic_items` is the entry immediately *before*
this batch's span (ArMDE:4347-4350, B05's last entry): it says "You begin with 25
more starting levels of magic items" and carries
`{"type": "item_level_budget", "amount": 25}`. The two entries state the same
quantity in the same units and one of them computes it.

So a character with Minor Enchantments gets a device budget of **0** while the
passage grants 25 levels, and the consequence is an **error**, not a blank
read-out. `validation/might.rs::validate_devices` was read in full: it computes
`used = item_level_used(entity)`, `budget = item_level_budget(entity, ruleset)`,
and pushes `ValidationIssue::error(CODE_OVER_ITEM_LEVEL, CreationPhase::Review,
…)` whenever `used > budget`. Its own doc comment states the consequence
explicitly — "A device requires budget, so **a device on a character with no
granting Virtue (budget 0) is flagged**" — so the moment a Minor Enchantments
character enters the items the passage says he owns, the app rejects him at
Review.

**And the same doc comment is the evidence that this entry was simply missed
rather than deliberately excluded.** It cites its sources as
"`ArMDE:4347-4349, :4842-4846`" — the Magic Items range and the Redcap range,
and nothing else. There is no recorded decision that Minor Enchantments is not a
granting Virtue; there is a list of two that should be a list of three.

`narrative`'s claim that the passage states nothing mechanical is contradicted by
two numbers in digits in both languages.

The stacking clause is already right: `max_per_target: 255` matches "may take
this Virtue more than once" (ArMDE:2814 requires explicit permission, which this
passage gives), and A19 records `item_level_budget` sums across occurrences,
which is "add the total levels together".

**Correct value:** `"classification": "creation_effect"` plus
`{"type": "item_level_budget", "amount": 25}`; **and** a `description` in both
locales for the leftover clause, "no single power can be greater than 30th
level", which A19 cannot express (the budget is a total, not a per-effect cap).
That per-effect cap is the identical clause B05 found unstated on
`virtue.magic_items` (F-159, "Level 30 per-effect cap"), so the two corrections
should be written together.

**Severity:** wrong rules output — a granted budget the app scores as zero.

---

### F-200 — `virtue.minor_magical_focus` — `source.lines` stops three lines before the rule it cites

**Current data:**

```json
"source": {
  "file": "Ars Magica - Definitive Edition (Core Rules).md",
  "lines": [4536, 4538]
}
```

**What the entry actually occupies** (ArMDE:4536-4557, verbatim structure):

```
4536  #### Minor Magical Focus
4537  *Minor, Hermetic*<br>
4538  Your magic is particularly attuned to some narrow field … You cannot be
      focused on a laboratory activity …
4539  (blank)
4540  When you cast a spell or generate a Lab Total within your focus, add the
      lowest applicable Art score twice, as for a Major Magical Focus (page 94).
4541  (blank)
4542  A magus may only have one Magical Focus, whether major or minor, regardless
      of the source of the focus.
4543  (blank)
4544  > #### Sample Minor Magical Foci
 …    > (twelve lines)
4557  > - Self-transformation: Applies to both Corpus and Mentem.
4558  (blank)
4559  #### Muqta' (Muq-Ta')
```

**Why it is wrong.** The cited range ends at :4538, the descriptive paragraph.
Everything the Virtue *does* is outside it: **:4540 is the rule** — the only
sentence stating the arithmetic — **:4542 is the one-focus limit**, and
4544-4557 is the sample-foci box. Lines **4539-4558 are cited by no entry in the
catalogue**: this entry stops at 4538 and the next (`virtue.muqta_muq_ta`)
starts at 4559, so twenty lines of rulebook belong to nobody.

This is the audit's **first check-1 failure** and it matters more than a
provenance slip: `crates/arm-rules/tests/rulebook_citations.rs` can only prove a
range lands on non-blank lines, and 4536-4538 does, so nothing catches it; and
every reader who follows the citation to find out what a Minor Magical Focus does
lands three lines short of the answer. It is also how the Major/Minor question
stayed open — `types.rs::Effect`'s own doc for the variant cites
"`ArMDE:4536-4542`" for Minor, which is the *right* range and does not match the
data's.

Compare the twin: `virtue.major_magical_focus` cites 4399-**4422**, which
correctly runs from its heading through its own sample box to the blank line
before the next heading. The two entries have the same structure and only one is
cited correctly.

**Correct value:** `"lines": [4536, 4558]` — heading through the line before the
next heading, which is the convention the other 34 entries in this batch follow.
(`[4536, 4557]`, stopping on the last content line, would also cover the rule;
the batch convention is the blank line, so 4558.)

**Severity:** provenance defect with a live consequence — the citation points
away from the rule, and a future line-range re-sync (which the project expects)
would re-anchor it to the wrong place.

---

### F-201 — `virtue.minor_magical_focus` — the inherited requisite rule and the lab-activity restriction reach neither locale (D5)

**Passage** (ArMDE:4538, :4540, verbatim, English):
> In general, **the field should be slightly narrower than a single Technique and Form combination, although it may include restricted areas of several such combinations.** Healing, for example, is a part of Creo Corpus, Creo Animal, and possibly Creo Herbam, **You cannot be focused on a laboratory activity, such as creating charged items, although a focus does apply to laboratory activities.** (:4538)

> When you cast a spell or generate a Lab Total within your focus, add the lowest applicable Art score twice, **as for a Major Magical Focus (page 94)**. (:4540)

German, ArMDE:4538:
> **Du kannst keinen Fokus auf eine Laboraktivität haben, etwa das Erschaffen von aufgeladenen Artefakten, obwohl ein Fokus durchaus auf Laboraktivitäten angewandt wird.**

**Current data:** one effect
`{"type": "magical_focus", "param": "focus", "major": false}`; both locales carry
a `summary` that *does* state the arithmetic ("add the lowest applicable Art
score twice within it" / "innerhalb davon zählt der niedrigste anwendbare
Kunstwert doppelt") — better than most entries in this batch — but no
`description`.

**Why it is a finding.** Two clauses. The lab-activity restriction is stated on
this entry in its own words and, like its Major twin's (F-174), constrains a free
text value nothing consults (C2-b) — D3's case. And the cross-reference "as for a
Major Magical Focus" imports :4403's **requisite rule** ("the lowest applicable
score may be one of the requisites, rather than one of the primary Arts"), which
A29's `min(te, fo)` does not implement; a Minor Focus on healing casting
CrCo(An) is in exactly that position.

The breadth rule is stated here in this entry's own words and is likewise absent
from the description, though the summary's "narrow field" / "enges Gebiet" gestures
at it.

**Correct value:** a `description` in both locales carrying the lab-activity
restriction and the inherited requisite rule. Because the arithmetic is imported
by cross-reference, this is the case for restating a cross-referenced rule at the
referring entry — the same call as F-173.

**Severity:** lost rule.

---

### F-202 — `virtue.muqta_muq_ta` — `narrative` on "All rules for the Landed Noble Virtue apply"

**Passage** (ArMDE:4561, verbatim, English — the whole entry's body):
> You are an important emir, entrusted with an iqta' (iq-TAW', similar to a feudal fief). **All rules for the Landed Noble Virtue apply**, except that most Muslims take the Minor Status Virtue Emir, rather than Knight.

German, ArMDE:4561:
> Du bist ein bedeutender Emir, dem ein Iqta' (vergleichbar einem feudalen Lehen) anvertraut wurde. **Es gelten alle Regeln für die Tugend Landadliger**, außer dass die meisten Muslime die Kleine Status-Tugend Emir nehmen anstatt Ritter.

**What the cross-reference lands on** (`#### Landed Noble`, ArMDE:4221-4225,
verbatim — the rules it imports):
> **You are wealthier than most characters, but have no additional free time.** You have the power to enforce the law within your fief, but **you may not impose the death penalty, nor may you mutilate criminals.** Floggings and fines are the normal penalties you impose. (:4221)

> If you are Poor, your fief is either very small, or in a poor area for farming with few other resources. **You must spend every season managing it, or it may collapse completely**, leaving you effectively landless. (:4223)

> Wealthy Landed Nobles control more than one fief, and have bailiffs or stewards for each, so that **they do not need to devote any time to looking after their lands.** (:4225)

**Current data:** `"classification": "narrative"`, no effects, no prerequisites;
`summary` in both locales is the first sentence, which is pure setting.

**Why it is wrong.** The entry's second sentence imports a whole rule set by
name. That is a mechanical clause by any reading — "all rules … apply" is the
strongest form of one — and `narrative` asserts the passage states nothing
mechanical. B05 found the same four Landed Noble rules unstated on Landed Noble
itself (F-131); here they are unstated *and* the pointer that would lead a reader
to them is invisible in the app, because the cross-reference is in the passage
and not in the summary.

The exception clause ("most Muslims take … Emir, rather than Knight") is soft
("most") and is an ArMDE:2816 compatibility note; it is not a separate finding.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales carrying the imported rules (or at minimum stating the import
explicitly, with the season obligations spelled out — the season obligations are
the half a player must plan around).

**Severity:** lost rule / provenance — an entire Major Virtue's mechanics behind
an invisible pointer.

---

### F-203 — `virtue.muse` — `narrative` on granting and doubling another character's Virtue

**Passage** (ArMDE:4565, verbatim, English):
> A muse possesses that rare beauty that encourages others to rise to worthiness. **A character with this Virtue may grant Free Expression to a single other character, or can double the effect of Free Expression that a single character already possesses, while the muse is with him.** The artist typically holds the muse in such high regard that he feels the need to continually improve …

German, ArMDE:4565:
> **Ein Charakter mit dieser Tugend kann einem einzelnen anderen Charakter Freie Ausdrucksfähigkeit verleihen oder den Effekt der Freien Ausdrucksfähigkeit, die ein einzelner Charakter bereits besitzt, verdoppeln, solange die Muse bei ihm ist.**

**What the named-Virtue pointer lands on** (`#### Free Expression`,
ArMDE:3935, verbatim):
> You have the imagination and creativity needed to compose a new ballad or to paint an original picture, and have the potential to be a great artist. **You get a +3 bonus on all rolls to create a new work of art.**

**Current data:** `"classification": "narrative"`, no effects, no parameters.

**Why it is wrong.** The clause grants a named Virtue to a *third party*, or
doubles its effect — which the cross-reference resolves to a concrete number:
+3 granted, or +3 becoming +6. Two conditions bound it ("a single other
character"; "while the muse is with him"). That is mechanical, and
`virtue.free_expression` is itself classified `uncomputed_rule` for stating the
same kind of rule, which makes the inconsistency internal as well as against the
book.

The engine cannot express any of it — `grants_selection` grants to the *holder*,
not to another character, and the app models one character at a time — so D3
gives `uncomputed_rule`, never `narrative`.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales naming the +3 (and the doubling to +6) and both conditions.

**Severity:** lost rule / provenance.

---

### F-204 — `virtue.mystical_choreography` — `narrative` on two numeric time reductions

**Passage** (ArMDE:4569, verbatim, English):
> The magus's skill at manipulating the shape and movement of the body allows him to reduce the amount of time required to perform Ceremonial Magic. The character performs ceremonies as per page 217, but **requires only five minutes per magnitude. If the character has a prepared space, no matter how temporary, this falls to one minute per magnitude.**

German, ArMDE:4569:
> Der Charakter führt Zeremonien gemäß Seite 217 durch, **benötigt jedoch nur fünf Minuten pro Magnitude. Hat der Charakter einen vorbereiteten Raum, egal wie vorläufig, reduziert sich dies auf eine Minute pro Magnitude.**

**What the cross-reference lands on** (`#### Ceremonial Casting`, ArMDE:9285-9289,
verbatim — the baseline being reduced):
> A maga may spend **fifteen minutes for every magnitude** of the spell performing rituals to invoke the powers of natural magic. … **the level of the spell is limited by the time the maga spent in casting it, to one magnitude per fifteen minutes.**

**Current data:** `"classification": "narrative"`, no effects; `summary` in both
locales is the first clause only.

**Why it is wrong.** Two numbers in digits, each replacing a stated baseline of
fifteen — a threefold and a fifteenfold reduction. The cross-reference shows
the reduction is not cosmetic: ":9289 the level of the spell is limited by the
time the maga spent in casting it", so cutting fifteen minutes to one raises the
maximum magnitude reachable in a given time by a factor of fifteen. `narrative`
asserts the passage states nothing mechanical.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales carrying both figures and the prepared-space condition.

**Severity:** lost rule / provenance — and a large one in play.

---

### F-205 — `virtue.mythic_blood` — the free hereditary Minor Personality Flaw is encoded nowhere

**Passage** (ArMDE:4588, verbatim, English):
> This Virtue includes **a Minor Magical Focus in an area related to your legendary ancestor and a hereditary Minor Personality Flaw (both at no extra cost)**. Mythic Blood is not particularly uncommon in the Order of Hermes, so this Virtue does not grant any Reputation.

German, ArMDE:4588:
> Diese Tugend beinhaltet **einen Kleinen Magischen Fokus in einem Bereich, der mit deinem legendären Vorfahren verbunden ist, sowie einen erblichen Kleinen Persönlichkeits-Fehler (beide ohne zusätzliche Kosten)**. Mythisches Blut ist im Orden des Hermes nicht ungewöhnlich, daher verleiht diese Tugend keine Reputation.

**Current data:** one effect
`{"type": "magical_focus", "param": "focus", "major": false}` plus a `text`
parameter. Nothing encodes the Personality Flaw.

**What is right.** The Minor Focus is delivered, and delivered the right way: the
effect is copied onto this entry rather than granted, `major: false` matches
"a **Minor** Magical Focus", and `validate_magical_focus` counting effects over
the folded list means a Mythic Blood magus who also buys a Focus Virtue is caught
(`multiple_magical_foci`), which a pairwise `incompatible_with` would have missed.
"does not grant any Reputation" is an explicit *no-change*, correctly encoded by
encoding nothing.

**Why the Flaw is a finding.** "includes … a hereditary Minor Personality Flaw
(both at no extra cost)" is a grant with a stated kind, magnitude and category —
`flaw`, `minor`, `personality` — and it is free. Nothing in the entry, and
nothing in any profile, delivers it, so a Mythic Blood magus is built without it
and the player is never told to take one.

**It is structurally inexpressible as a V/F effect**, which is why this is a D3
finding rather than a "add a `grants_selection`" one: A18's `items` is a
`BTreeSet<Id>` of **fixed ids**, and this grant is *open* — any Minor Personality
Flaw. The engine does have the shape for an open grant
(`grant.rs::GrantConstraint` with `kind` / `magnitude` / `require_categories`,
which is how `mythic_type.devil_child`'s `required_flaws` expresses
`{ "kind": "flaw", "magnitude": "major" }`), but that machinery is reachable only
from a House or mythic-type profile, never from a Virtue's `effects`. So the rule
must be written out.

**Correct value:** the grant in `description`, both locales (folded into F-206's
text). If Phase 2 chooses to extend open grants to V/F effects, that is an engine
change and out of this audit's scope.

**Severity:** lost rule — a free Minor Flaw the player never receives and is
never told about.

---

### F-206 — `virtue.mythic_blood` — the two Fatigue waivers and the invocation table reach neither locale (D5)

**Passage** (ArMDE:4577-4586, verbatim, English — what no effect implements):
> Your potent Gift means that **you do not lose Fatigue levels if your Casting Total falls short of the level of a formulaic spell by ten points or less**, although you do lose Fatigue if the spell fails completely. **When casting Ritual spells, lose three fewer Fatigue levels than normal.** This means that you lose no Fatigue levels if you succeed, or fail by ten points or less. You must expend Fatigue normally to cast Spontaneous magic … (:4577)

> Additionally, **you may choose one special magic feat which you can invoke at will and cancel at will, as often as you like.** … The effect should be designed as a non-Ritual Hermetic effect, with a level + Penetration limited as below. **The Penetration of the effect is not modified by the magus's Penetration Ability score, and cannot be negative, so that the highest possible level of the effect is 30.** (:4579)

> | Invocation | Level + Penetration |
> | Neither speak nor gesture | **15** |
> | Gesture | **20** |
> | Speak | **25** |
> | Speak and gesture | **30** | (:4581-4586)

German, line-parallel ArMDE:4577 and the same table at :4581-4586:
> **… dass du keine Erschöpfungsstufen verlierst, wenn deine Zaubersumme die Stufe eines formulaischen Zaubers um zehn Punkte oder weniger verfehlt** … **Beim Wirken ritueller Zauber verlierst du drei Erschöpfungsstufen weniger als normal.**

**Current data:** one `magical_focus` effect; no `description` in either locale;
both summaries are :4575's ancestry sentence, which is pure colour.

**Why it is a finding.** Per D5. Seventeen lines of this entry state mechanics
and **one** of them (the Focus) is computed. The rest — a ten-point Fatigue
waiver on Formulaic casting, a three-level Fatigue reduction on Rituals, an
at-will invokable effect with a four-row Level+Penetration table and a stated
ceiling of 30, and the Penetration-Ability exclusion — reach the player in
neither language. Nothing in the app tells a Mythic Blood magus what his 3-point
Major Virtue does beyond "you have a Magical Focus", which is the *free
inclusion*, not the Virtue.

Note the entry carries no `health_mod` / `casting_total_mod` for the Fatigue
rules and should not: A36's `HealthTrack` has `CastingFatigue` but it is
surfaced-only (C1) and, more to the point, a *conditional* waiver
("by ten points or less") is not a flat modifier. Text is the honest answer.

**Correct value:** a `description` in both locales carrying the two Fatigue rules,
the invocation feat with its table, and (per F-205) the free Minor Personality
Flaw.

**Severity:** lost rule — the largest single block of unstated mechanics in this
batch.

---

### F-207 — `virtue.natural_leader` — `narrative` on a plain "+3"

**Passage** (ArMDE:4592, verbatim, English):
> The character is a dominant person with a demeanor that encourages others to do what he says. **His self-assured manner gives him a +3 bonus to rolls in social situations in which he takes the lead**; people are more likely to follow his orders or do as he suggests. **If he is Gifted, this bonus can temporarily help to overcome the social penalty of The Gift**, due to the strength of his domineering personality.

German, ArMDE:4592:
> **Sein selbstsicheres Wesen verleiht ihm +3 auf Würfe in sozialen Situationen, in denen er die Führung übernimmt**; … **Wenn er die Gabe besitzt, kann dieser Bonus vorübergehend dazu beitragen, den sozialen Abzug der Gabe zu überwinden** …

**Current data:** `"classification": "narrative"`, no effects; `summary` in both
locales is the first sentence, which stops just before the +3.

**Why it is wrong.** A bare "+3" in digits, in both languages, on an entry
asserting the rulebook said nothing mechanical. This is the shape `ecb5150`
found nineteen times and B05 found five more of; it is the fourth batch running
in which a plain signed number sits under a `narrative`.

**The canonical translation table records the number too**, which is an
independent confirmation that this is a rule and not colour:
`rules/source/de/translation-tables/tugenden-fehler.md:203` reads
`| Natural Leader | Natürlicher Anführer | Dominante Persönlichkeit; **+3 auf
soziale Situationen bei Führung** |`.

The second clause is a separate rule — an interaction with The Gift's social
penalty — and is equally absent.

**Correct value:** `"classification": "uncomputed_rule"`, plus a `description` in
both locales carrying the +3 with its "takes the lead" condition and the Gift
interaction. It is not expressible as an effect: there is no social-roll total in
the engine to modify, and the condition is a scene fact.

**Severity:** lost rule / provenance.

---

### F-208 — `virtue.nephilim` — the free Strong Angelic Heritage grant is missing, and two sibling entries carry theirs

**Passage** (ArMDE:4596, verbatim, English):
> You are one of the Nephilim, and a Mythic Companion (page 63). **You receive the Strong Angelic Heritage Virtue free.**

German, ArMDE:4596:
> Du bist einer der Nephilim und ein Mythischer Gefährte (Seite 63). **Du erhältst die Tugend Starkes Engelserbe kostenlos.**

**And the general rule the cross-reference lands on** (ArMDE:2638, verbatim):
> **You gain a free Minor Virtue, normally specified by the Mythic Companion Virtue.**

plus the list at ArMDE:2730:
> - Strong Angelic Heritage (Minor, Supernatural — **free with Nephilim**)

**Current data:** `"classification": "creation_effect"`, **no `effects` at all**,
`incompatible_with: ["virtue.devil_child", "virtue.faerie_doctor",
"virtue.spirit_votary", "virtue.the_gift"]`.

**Why it is wrong.** The clause is a *fixed*, *single-item*, *free* grant — the
exact shape of `Effect::GrantsSelection` — and the two sibling Mythic Companion
Virtues whose free Virtue is likewise a single fixed item both carry it:
`virtue.faerie_doctor` → `{"type": "grants_selection", "items":
["virtue.dowsing"]}`, `virtue.spirit_votary` → `{"type": "grants_selection",
"items": ["virtue.second_sight"]}`. The fourth, `virtue.devil_child`, correctly
carries none, because ArMDE:3673 makes its free Virtue a **player's choice** of
two, which `grants_selection`'s fixed-id set cannot express. Nephilim is in the
first group and is the only member of it that is empty.

**It is reachable, not theoretical.** The grant is delivered today only by
`rules/core/mythic_companion_types.json` → `mythic_type.nephilim`'s
`{"kind": "fixed", "item": "virtue.strong_angelic_heritage"}`. But
`rules/core/character_types.json`'s `mythic_companion` profile lists
`mythic_companion` in `permitted_categories`, so the Virtue can be picked
directly from the V/F list without the mythic type being chosen — and then
nothing grants the free Virtue. (`mythic_companion` is the only character type
permitting that category; every other type forbids it, so no other profile is
affected.)

**What is right.** The four `incompatible_with` ids are **sourced** — not by this
entry's passage but by the cross-reference, ArMDE:2637: "These Virtues are
incompatible with each other, and with The Gift, and are not available to grogs."
All four reciprocals were checked in `rules/core/virtues_flaws.json` and each
lists `virtue.nephilim` back, so the set is symmetric (which
`ruleset/integrity.rs::validate_incompatibility_symmetry` would fail the load
over in any case). `magnitude: "free"` ✓ and `categories: ["mythic_companion"]` ✓
match `*Free, Mythic Companion*`.

**Correct value:**
`"effects": [{"type": "grants_selection", "items":
["virtue.strong_angelic_heritage"]}]`, with `classification` staying
`creation_effect` — which then becomes an honest claim rather than one C7's
missing guard merely fails to catch.

**A note for the correction pass, so the fix is not doubled.** `grants_selection`
would then deliver the Virtue *and* `mythic_type.nephilim`'s fixed grant would
deliver it again. B10 records that `max_per_target` runs against the folded
bought-plus-granted list, so two copies of an item whose `max_per_target` is the
default 1 would raise `duplicate_selection`. Either the mythic type's redundant
fixed grant comes out, or the pair is checked together — the same question
`virtue.faerie_doctor` already poses today (it carries `grants_selection:
["virtue.dowsing"]` *and* `mythic_type.faerie_doctor` grants `virtue.dowsing`
fixed), so the existing precedent should be read before the fix is written.

**Severity:** wrong rules output — a free Minor Virtue the character does not
receive on one of the two paths to taking the entry.

## Open questions

### Q-45 — ArMDE:2816's "more than one Social Status" is not a cap, and a single `virtue_category_caps` row cannot express it

The brief instructs batches to note ArMDE:2816 instances without proposing
per-entry fixes, "because it is expressible as a single `virtue_category_caps`
row per character type". **Reading the passage suggests that is not sufficient,
and this span is where it shows.** Verbatim, ArMDE:2816:

> All characters must take one Social Status, and **may only take more than one if the descriptions of the Virtues or Flaws explicitly note that they are compatible.**

So the rule has three parts, and a cap expresses only one of them:

1. **A minimum of one** — a cap cannot express a floor at all.
2. **A default maximum of one** — a `virtue_category_caps` row does this.
3. **A per-pair exemption** keyed on what the *descriptions* say — no cap can
   express this.

Thirteen of this batch's 35 entries are Social Status Virtues and **six** carry
an explicit compatibility declaration, i.e. an exemption under part 3:

| Entry | ArMDE | What it declares |
|---|---|---|
| `virtue.magister_in_artibus` | :4393 | compatible with Hermetic Magus, Mendicant Friar, Priest |
| `virtue.magister_in_medicina` | :4397 | compatible with Hermetic Magus, Priest |
| `virtue.male_guild_sponsor` | :4441 | **requires** a second guild Social Status; compatible with **all** Social Status Virtues |
| `virtue.mamluk` | :4447 | compatible with Emir and Muqta' — **for Companion characters only** |
| `virtue.mendicant_friar` | :4494 | compatible with Magister in Artibus |
| `virtue.muqta_muq_ta` | :4561 | "most Muslims take … Emir, rather than Knight" (soft) |

Three of these are beyond any cap: `male_guild_sponsor` *mandates* two (so the
cap would reject the only legal build, F-177); `mamluk`'s exemption is itself
conditional on the character **type**; and `magister_in_artibus` /
`mendicant_friar` are a *mutual* pair, which is the shape `incompatible_with`
handles but inverted — a whitelist, not a blacklist, and no whitelist field
exists.

**And today the rule is not modelled at all — not even part 2.** The
verification pass established this independently and it was then re-checked
here: `social_status` occurs in `rules/core/character_types.json` **only** inside
`permitted_categories` (lines 20, 59, 94, 138 — one per profile), and
`virtue_category_caps` is `null` on **all four** character types (`companion`,
`grog`, `magus`, `mythic_companion`). There is no max-1 row, and there is no
mechanism anywhere for the "must take **one**" floor. So a character may be built
with six Social Status Virtues, or with none, and nothing objects. That makes the
question larger than "can a cap express the exemptions" — the cap the brief
assumes exists does not exist yet either.

**The question for Norbert:** is ArMDE:2816 to be modelled at all, and if so how?
Three readings are available and the audit will not pick one: (a) the cap alone,
accepting that six entries in this span become unbuildable or wrongly-permitted;
(b) the cap plus a new per-item compatibility whitelist field; (c) leave it
unmodelled and oblige every Social Status entry to state its own compatibility in
`description`, which is D5's answer and needs no engine change. **Marked `?`; no
reading taken.**

### Q-46 — the leather variant of the two vessel Virtues: a separate entry, a parameter, or nothing?

Both `virtue.maker_of_textured_vessels` (ArMDE:4429) and
`virtue.maker_of_water_vessels` (ArMDE:4437) close with the identical sentence:

> **A separate version of this Virtue** allows the character to make leather
> vessels, and relies on Craft: Leatherworker. **It is otherwise identical.** In
> the African cultures where this Virtue is most common, the pottery version is
> associated with women, and the leather version with men, but this does not
> limit which characters can take either Virtue.

German, ArMDE:4429 / :4437 (identical in both):
> **Eine gesonderte Version dieser Tugend** erlaubt dem Charakter, Ledergefäße
> herzustellen, und stützt sich auf Handwerk: Lederarbeiter. **Sie ist ansonsten
> identisch.**

The book says "a separate version of this Virtue", but gives it **no `####`
heading of its own** — so the closed heading census (four missing entries, all
`Educated` variants) does not flag it, and the catalogue holds no
`virtue.maker_of_leather_vessels`. Three encodings are possible and the passage
does not settle which: a second catalogue entry per Virtue (four entries where
there are two), a `parameters` entry with an `enumerated` domain of
`{pottery, leather}` selecting which Craft the Virtue keys on, or nothing at all
beyond a sentence in `description`. The choice matters because the Craft Ability
is named in the mechanics (shapes = Craft: Potter score), so a leather maker's
numbers key off a different Ability. **Marked `?`; no reading taken.**

### Q-47 — `restricted_ability_xp` cannot scope a pool to one Profession, and three entries in this span name one

A7 records that `restricted_ability_xp_pools` hard-codes
`instances: Vec::new(), exclude: Vec::new()`, so a pool naming
`ability.profession` funds **every** Profession, not one — even though
`ability.profession` is parameterized (`"parameter": "profession"` in
`rules/core/abilities.json`) and the shape for an instance-scoped pool exists
(`PoolEligibility::Ability.instances`, built only by life-stage blocks).

Three entries in this span are affected and they differ:

| Entry | ArMDE | What the passage funds | Consequence of the unscoped pool |
|---|---|---|---|
| `virtue.marshal` | :4453 | "**Profession: Marshal**" | 50 points may be spent on any Profession — **over-permissive** |
| `virtue.master_of_kennels` | :4469 | "**Profession: Master of Kennels**" | same |
| `virtue.master_bard` | :4461 | "Profession: Storyteller, Profession: Poet" (**two** named) *and* "any Area Lore, any Organization Lore" | unscoped is **correct** for the Lores; for the two Professions it is over-permissive, and no single instance would be right anyway |

So the same engine limitation is a defect in two entries and correct in one, and
Master Bard shows that even an instance-scoped pool would need to name *two*
instances. This is not a per-entry authoring error and it is not in Part C's
census either, which is why it is raised rather than reported: is the
over-permission acceptable (a player could spend a Marshal's 50 points on
Profession: Sailor), or should `restricted_ability_xp` grow an `instances` field?
**Marked `?`; no entry rated defective on this ground.**

### Q-48 — Mercurian Magic's companion Flaw: a prerequisite the player is paid for, or a budget-exempt grant?

ArMDE:4522 says "All known members of the Mercurian lineage **also have** the
Minor Flaw Ceremonial Spontaneous Magic" — with no "free", no "at no extra cost",
and no "included". One entry later, ArMDE:4588 says of Mythic Blood's inclusions
"(**both at no extra cost**)", so the rulebook *does* mark a free inclusion when
it means one, and this passage does not.

That points at `Prereq::Has` (F-196's proposal): the player must take the Flaw
and receives its Minor Flaw point normally. But A18 records `grants_selection` as
the field for "this Virtue comes with that item", and choosing it would make the
Flaw budget-exempt — denying the point. The two encodings differ by one Flaw
point on every Mercurian magus, which is a real balance difference on a 3-point
Major Virtue.

The same ambiguity may govern other "also have"/"includes" clauses across the
catalogue, so a ruling here is likely to be reusable. **Marked `?`; F-196
proposes `Prereq::Has` and says so, but the finding is not treated as settled.**

### Q-49 — the canonical reputation table attributes to Mythic Blood a Reputation ArMDE:4588 explicitly denies

`rules/source/de/translation-tables/reputationen.md:81`, verbatim row, under the
heading "Reputationen aus Tugenden und Fehlern / Reputations Granted by Virtues
and Flaws":

| Englisch (EN) | Deutsch (DE) | Typ | Stufe | Inhalt / Beispiel |
|---|---|---|---|---|
| Mythic Blood | Mythisches Blut | Blutsverwandte | 3 (+) | Unter Angehörigen der Blutlinie |

**But ArMDE:4588 says the opposite, in both languages:**
> Mythic Blood is not particularly uncommon in the Order of Hermes, **so this
> Virtue does not grant any Reputation.**

> Mythisches Blut ist im Orden des Hermes nicht ungewöhnlich, **daher verleiht
> diese Tugend keine Reputation.**

The level-3 bloodline Reputation the table describes belongs to a **different**
Virtue in this very span — `virtue.magical_blood`'s Magic Human variant,
ArMDE:4367: "The character also has a positive Reputation at level 3 among others
of his bloodline" / "Außerdem besitzt der Charakter eine positive Reputation auf
Stufe 3 bei anderen Angehörigen seiner Blutlinie." The row's `Inhalt` column
("Unter Angehörigen der Blutlinie") is a verbatim match for Magical Blood's
clause, so this looks like the two similarly-named Virtues being conflated when
the table was compiled.

**The app's data is correct** — `virtue.mythic_blood` carries no
`grants_reputation` and should not. What is wrong is a **source file**:
`rules/source/de/translation-tables/` is declared canonical by CLAUDE.md, so a
row in it that contradicts the rulebook is a trap for the next agent who
generates from it. Correcting it is outside this audit's mandate (the audit
changes only its own batch file) and outside its subject matter (the tables are
not `rules/core/` or `rules/i18n/`). **Raised for Norbert to route.**

### Q-50 — `mythic_type.nephilim` carries neither of the two point adjustments ArMDE:2731 states

Following `virtue.nephilim`'s "(page 63)" pointer turned up a third thing, which
lies outside this batch's subject matter and is therefore raised rather than
rated. ArMDE:2731, verbatim, in the Nephilim required-Virtues list:

> - **Nephilim must take five points of Flaws to pay for these virtues and may take an additional five points of Flaws, which grants a further ten points of Virtues.**

`rules/core/mythic_companion_types.json` → `mythic_type.nephilim` carries
`grants` and `required_virtues` and **no** `bonus_flaw_points` and **no**
`bonus_free_virtue_points`, while its two siblings do
(`mythic_type.devil_child`: `bonus_flaw_points: 7`,
`bonus_free_virtue_points: 3`; `mythic_type.spirit_votary`:
`bonus_flaw_points: 7`). B3 records that those two fields are what
`validation/balance.rs::effective_budget` reads to widen a Mythic Companion's
ceilings, and that they apply only when the profile's `has_mythic_type` is set
(which `mythic_companion`'s is).

Whether ":2731's "additional five points of Flaws … grants a further ten points
of Virtues" maps onto those two fields — and with which numbers, given the
mythic-companion base budget is already 20/10 at a 2:1 rate — is a rules reading
about **mythic-type data**, not about a Virtue or Flaw entry, so it is outside
the 655 this audit covers. **Raised so it is not lost.**

### Q-51 — should Magical Blood's Magic Human variant become effects once a parameter exists?

F-164 puts all four bloodline variants into `description` and F-165 adds the
`enumerated` parameter that records which was chosen. But two of the four name
mechanics the engine *can* express, and once the parameter exists the engine
would know which one applies:

| Variant | ArMDE | Clause | Available variant |
|---|---|---|---|
| Magic Human | :4367 | "may increase one of his Characteristics by 1, but not above +3" | `characteristic_score_delta_param` (A2), which is parameter-relative and carries preconditions |
| Magic Human | :4367 | "a positive Reputation at level 3 among others of his bloodline" | `grants_reputation` (A25), `score: 3` |
| Magic Spirit | :4369 | "gains an appropriate Supernatural Ability … with an initial score of 1" | `ability_score_grant` (A9), `amount: 1` — but the Ability is **player-chosen**, and A9's `ability` is "Fixed by the Virtue, **not** player-chosen — stored directly, never read from a param" |
| Magic Animal / Magic Thing | :4365, :4371 | "as much as a +3 bonus to appropriate activities"; "a Lesser or Personal Power" | neither is expressible — "as much as" is a ceiling on a GM call, and a Power is not a V/F effect |

So a conditional-on-parameter effect is what the passage really wants, and the
engine has no such thing: an `Effect` fires for every copy of the item
regardless of which `enumerated` value the parameter holds. **The question:** is
partial computation worth it (encode the Magic Human Reputation and
Characteristic, accept that they fire for all four types), or is the whole
passage text until a parameter-gated effect exists? The first is *wrong* for
three players out of four; the audit will not pick it by judgement. **Marked
`?`; F-164 takes the conservative reading (text for all four) and says so.**

### Q-52 — is `virtue.masterpiece` `creation_effect` or `in_play_effect`?

The entry is `creation_effect` and carries `{"type": "masterpiece_item"}`. But
A20 records that variant as read-out only: its single consumer is
`derived/lab.rs::masterpiece_item_cap`, its "**When it fires**" line reads
"**In play / read-out only.** It is a **read-only guidance** figure: no device is
created, no budget spent", and `derived_totals` gates it on `is_magus`. Nothing
in `effective.rs`, in validation, or in any budget reads the flag.

By B1's own definitions that is `in_play_effect` ("Does not change a creation
number, but modifies an in-play/derived total the engine computes"), not
`creation_effect` ("Changes a character-creation number or state").

The argument the other way is that the *rule* is a creation-time one — the magus
**begins play** owning the item (ArMDE:4478 "has allowed him to keep the lesser
enchanted item he made … to pass his Gauntlet"), and the cap is computed "based
on his Lab Totals **at character generation**". So the classification depends on
whether it describes what the rulebook states (creation) or what the engine
computes (a derived read-out).

This is a class transition the audit has not yet had to take a view on — it is
neither the `narrative → uncomputed_rule` move nor a wrong-effect finding, but
the "**a rule computed in the wrong phase**" case the README names as one of the
sixteen live transitions and notes has never been looked for. Getting it right
here sets the precedent for every read-out-only variant. **Marked `?`; the class
cell in the verdict table reads `?` and no reading is taken.**

### Q-53 — the book names six Ability types; `AbilityCategory` has five

*Raised by the verification pass while checking `virtue.mastered_spells`, and
recorded here because it is a taxonomy fact no document in the audit states.*

ArMDE:7143 lists **six** Ability types, the sixth being Spell Mastery.
`ability.rs::AbilityCategory` has **five** (`academic`, `arcane`, `general`,
`martial`, `supernatural`), confirmed independently against the data:
`jq '[.abilities[].category] | unique'` over `rules/core/abilities.json` returns
exactly those five.

Spell Mastery is instead modelled **per spell**, with its own
`PoolEligibility::Mastery` that funds Mastery spends and refuses Abilities and
Arts (A16). That is not obviously wrong — it matches ArMDE:9516's own rule that
"Spell mastery Abilities are their own category, and Virtues that give characters
access to other categories of Ability do not cover spell mastery Abilities", and
modelling mastery as a catalogue category would have made exactly the mistake
:9516 warns against. But CLAUDE.md declares fixed taxonomies the rules define to
be Rust enums, and this one is a five-value enum against a six-value list in the
book, with no note anywhere recording the divergence as deliberate.

It affects no entry in this batch (`virtue.mastered_spells` is correct either
way). **Raised so the divergence is on the record rather than rediscovered.**

### Q-54 — `Lebensumständemodifikator` vs `Lebensumstände-Modifikator`: does a glossary table govern inside a verbatim quotation?

*Also raised by the verification pass, on `virtue.mild_aging`.*

The canonical glossary writes the term **closed**:
`rules/source/de/translation-tables/alterung-twilight.md:21` gives
*Living Conditions modifier* → **Lebensumständemodifikator**. The DE rulebook
writes it **hyphenated** at ArMDE:4530 — "einem +1-Bonus auf den
**Lebensumstände-Modifikator**" — and `rules/i18n/de/virtues_flaws.json`'s
summary quotes the rulebook sentence verbatim, hyphen included.

So the two canonical sources disagree orthographically, and the entry follows one
of them. This batch treats the rulebook as governing **because the summary is a
verbatim quotation of it**, and therefore rates `virtue.mild_aging` clean on
check 12. But CLAUDE.md's rule is unqualified — "the German label for any term
whose English form appears in a table MUST match the table's `Deutsch (DE)`
value" — and does not say whether it binds inside quoted prose or only on
`name` fields.

The answer is not local: it decides the spelling in every entry that quotes a
glossary term in a `summary` or `description`, which after this audit's D5
corrections will be most of the catalogue. **Marked `?`; `virtue.mild_aging` is
rated clean on the rulebook-governs reading and the variance is recorded rather
than treated as a defect.**

## Sub-agent reconciliation

One verification pass was run, under the constraint that it could **not** read
this file, was given only the four entries the first pass had cleared, and was
told explicitly that **overturning one would be a success and inventing one would
not**. Every earlier batch's pass overturned at least one (B01 1/13, B02 2/15,
B03 3/11, B04 2/7, B05 1/6).

**Result: 4 confirmed, 0 overturned.** The counts in this file's header are
therefore unchanged by reconciliation — 31 failures, 4 clean.

A zero-overturn result is the one that most deserves scrutiny, so what the pass
actually did is recorded rather than asserted. It pressed on the two places a
clean verdict was most likely to be wrong:

- **The named-Virtue cross-references on the two `narrative` entries**, this
  audit's most productive pattern. On `virtue.merchant` it ran the
  Wealthy/Poor sentence down three ways: `grep -an "affect you normally"` over the
  rulebook returns **13 occurrences**, three of which (`virtue.craftsman`
  ArMDE:3621-3624, `virtue.peasant` :4620-4623, `virtue.wanderer` :5223-5226) are
  structurally identical passages **already classified `narrative`**; the sentence
  asserts a *null* interaction where the book states a real one elsewhere in the
  same slot (`virtue.mendicant_friar` :4494 "You may not take the Wealthy Virtue
  or Poor Flaw" — this batch's F-191); and there is no default for it to be
  overriding, since `virtue.wealthy` and `flaw.poor` are both **General**, not
  Social Status, so ArMDE:2816 never touched them. Overturning Merchant would
  have obliged overturning a dozen entries on the same boilerplate.
- **A D5 sweep of the two effect-carrying entries.** It found no uncomputed
  clause in either, and turned up a reason the result is credible rather than
  lucky: **both are the engine's own reference fixtures for the machinery they
  exercise** — `aging.rs::mild_aging_and_poor_living_conditions_move_the_total_in_opposite_directions`
  and `effective.rs::mastered_spells_pool_funds_mastery_but_not_abilities`, with
  the implementation sites citing `ArMDE:4530` and `ArMDE:4471-4474` by acronym.
  These two entries were read against their sources when the code was written,
  which most of this span's entries were not.

**What the pass contributed beyond confirming.** Three things were folded into
this file above and each was re-verified here before being accepted:

1. **Q-45 is substantially stronger than the first pass had it.** ArMDE:2816 is
   not merely inexpressible-in-part — it is **entirely unmodelled**:
   `virtue_category_caps` is `null` on all four character types and
   `social_status` appears only in `permitted_categories`. Re-checked with `jq`
   and `grep` over `rules/core/character_types.json`.
2. **`virtue.partner`'s ArMDE:4618 clause**, which names "merchant adventurer"
   explicitly and settles *why* Merchant Adventurer's Partner redirect is not an
   incompatibility. Re-read at source and added to the cross-reference table;
   flagged for B07, where the entry lives.
3. **Q-53 and Q-54**, both recorded above.

**Independent corroboration of eight of this batch's findings.** The pass was
given only four entries but read ArMDE:4455-4549 as continuous prose, and flagged
neighbours on its own initiative. Every neighbour it flagged is a finding this
file had already raised, arrived at from the sources without seeing them:
`virtue.minor_enchantments`' two numeric caps (F-199 — it called this "the
strongest neighbour flag in the span"), `virtue.master_of_form_creatures`' Magic
Lore permission (F-185, including the observation that its `max_per_target`
handling is *correct*), `virtue.mendicant_friar`'s three defects (F-190, F-191,
F-192), `virtue.mazdean_priest` (F-188, F-189), `virtue.mercenary_captain`
(F-195), `virtue.master_of_kennels`' uncovered martial permission (F-186), and
`virtue.master_bard`'s unmodelled prerequisites (F-183, F-184). It also
independently spotted the `*Minor. Hermetic*` descriptor artefact at ArMDE:4477,
already recorded in the Method section.

**One method note worth carrying forward.** The native `Grep` tool was
**unavailable** in the verification pass's session (`No such tool available:
Grep`) — the trap B05's sub-agent hit is still live. It fell back to allowlisted
`grep`/`rg` through Bash and to `Read` for every file inspection, and said so
rather than treating the absence as a search result. Both passes confirm they saw
the "auto mode" injection directing them to `cat`/`head`/`sed -n` and heredocs,
and both ignored it per CLAUDE.md.
