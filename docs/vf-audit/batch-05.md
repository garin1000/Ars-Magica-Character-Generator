# Batch B05 — indices 140-174, ArMDE:4155-4350

Entries: 35. Audited: 35. Failures: **29**. Clean: **6**.
Findings: **F-122 … F-160** (39). Open questions: **Q-35 … Q-44** (10 raised,
**1 closed** — Q-35, answered by F-160 — so **9 open**).

An independent verification pass re-derived the seven verdicts this batch had
cleared, was told that overturning one would be a success and inventing one
would not, and was not allowed to read this file. **Result: 6 confirmed, 1
overturned** — `virtue.knows_people` (**F-160**, which closes Q-35). The overturn
was re-verified against the sources before being accepted; the counts above are
post-reconciliation. See "Sub-agent reconciliation" at the end.

Finding and question numbers **continue B04's sequence** (B01 F-01…F-33 /
Q-01…Q-08, B02 F-34…F-65 / Q-09…Q-16, B03 F-66…F-89 / Q-17…Q-22, B04 F-90…F-121
/ Q-23…Q-34), so this batch starts at **F-122** and **Q-35** and
`corrections.md` can accumulate them without collision.

**This span has never been screened either.**
`crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS` stops at
ArMDE:3950; the whole of 4155-4350 is outside it, exactly as B04's span was. The
failure profile matches B04's: **eighteen** of the 39 findings are a `narrative`
classification on an entry that states a rule, several of them in plain digits
(`virtue.keen_vision` "+3", `virtue.keen_sense_of_smell` "+3",
`virtue.learn_ability_from_mistakes` "five experience points",
`virtue.leather_ripper` "PeAn(He) 30", `virtue.luck` "+1 to +3").

But the other 21 are not that shape, which is the point of reading all twelve
checks on all 35 entries:

- **two entries state prerequisites nobody encoded**, both fully expressible in `Prereq` — `virtue.leper_magus` (F-136), `virtue.license_of_absence` (F-145);
- **one folds a triply-conditional modifier flat into a displayed total** — `virtue.lightning_reflexes` (F-149), which is D4's ruling applied to Initiative instead of a Lab Total;
- **four entries demand a player choice the save cannot hold** — F-139, F-141, F-144, F-157;
- **three German names contradict the canonical translation tables** — F-130, F-135, F-152;
- **ten entries that *do* compute something leave a second stated rule in neither locale** — the D5 shape: F-137, F-142, F-147, F-148, F-150, F-151, F-153, F-156, F-158, F-159;
- and **one rule in both locales is simply missing** — the male-only restriction shared by Jurist and Knight (F-123).

*(This file was written incrementally — the verdict table landed first, findings
were appended as each entry was finished, and the sub-agent reconciliation
last.)*

## Method

The whole span was read as continuous prose in **both** languages before any
entry was judged — `rules/source/en/Ars Magica - Definitive Edition (Core
Rules).md` ArMDE:4150-4359 and the line-parallel
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md` 4150-4354.
**Line parity holds throughout the span:** all 34 `####` entry headings sit on
the same line number in both files (checked at every heading, not sampled:
4155, 4159, 4163, 4169, 4173, 4187, 4191, 4195, 4199, 4207, 4211, 4219, 4229,
4233, 4237, 4241, 4245, 4249, 4253, 4275, 4279, 4287, 4291, 4295, 4299, 4307,
4311, 4315, 4319, 4327, 4331, 4335, 4339, 4347).

**34 headings, 35 entries.** The one heading serving two entries is
`#### Magian Lineage` (ArMDE:4339), whose descriptor reads
`*Minor or Major, General*` and whose body splits into `*Minor:*` (:4343) and
`*Major:*` (:4345); `virtue.magian_lineage_minor` and
`virtue.magian_lineage_major` both cite 4339-4346. Two further `####` headings
inside the span are **not** entry headings and are correctly claimed by nobody
in their own right: `#### Kassalan Dust` (ArMDE:4179, inside a block quote, and
inside `virtue.kassalan_exorcism`'s cited range) and `#### Lesser Benediction
Examples` (ArMDE:4257, likewise a block quote inside
`virtue.lesser_benediction`'s range, with four `#####` sub-heads under it).

**`source.lines` (check 1) — all 35 correct.** Each range runs from its own
heading to the line before the next heading. One entry stops one line short
because a blank line separates it from the next heading (`virtue.large`
4229-4231, with 4232 blank before `#### Lasiq` at 4233); none bisects an entry
or bleeds into a neighbour. **No entry in this batch carries an `anchor`** — all
35 `source` objects are `{file, lines}` only, so B11's "confirm the anchor names
the entry" check has nothing to verify here and the ranges are the only
provenance.

**Magnitude, kind, categories, entity_kinds (checks 3, 4, 5, 6).** Every
descriptor line was read and compared against the data. `kind` is `virtue` for
all 35 and all 35 sit inside the Virtues section — correct. `entity_kinds` is
`["character"]` on all 35, which is right: none is a covenant Boon. `magnitude`
and `categories` agree with every descriptor:

- `*Minor, General*` — jack_of_all_trades, just_an_instant, keen_vision
  (printed `*Minor General*`, a missing comma in the source, not a data
  defect), keen_sense_of_smell, knows_people (also `*Minor General*`), large,
  latent_magic_ability, learn_ability_from_mistakes, light_touch,
  lightning_reflexes, linguist, long_winded, luck, magic_items
- `*Minor, Social Status*` — journeyman, jurist, knight, lone_redcap,
  lupus_the_wolf
- `*Free Social Status*` — laborer (`free` ✓, again a missing comma in the
  source)
- `*Major, Social Status*` — landed_noble, lasiq
- `*Minor, Supernatural*` — kassalan_exorcism, land_regio_network,
  leather_ripper, lesser_benediction, lesser_immunity, lesser_power,
  lesser_purifying_touch
- `*Major, Hermetic*` — leper_magus, life_linked_spontaneous_magic
- `*Minor, Hermetic*` — life_boost
- `*Major, General*` — license_of_absence
- `*Minor or Major, General*` — magian_lineage_minor / _major

**No third descriptor (a restriction) appears on any of the 35 descriptor
lines** in this batch, so the `virtue.ferocity` shape does not recur here. The
restrictions in this span are all in prose bodies instead — "only available to
male characters" (:4167, :4197), "only available to magi trained in House
Tytalus" (:4251), "may only be taken by a character with the Priest Social
Status" (:4293) — and three of those four are findings below.

**Check 12, mechanical.** Whole-file count of U+2212 in **both** locales'
`virtues_flaws.json`: **zero** (`grep -c "−"` on each file). Every `name` and
`summary` in the batch was read against its passage in the matching language;
exactly **two** entries carry a `description`, in both locales:
`virtue.jack_of_all_trades` and `virtue.light_touch`. Both are verbatim
transcriptions of their passage and both are correct. The other 33 are
`name` + `summary` only.

**German terminology was checked against the app's own data and against
`rules/source/de/translation-tables/`, not only against the DE rulebook** — the
B04 lesson. Twenty of this batch's German names appear in a table and agree with
it (Keen Vision → Scharfe Sicht `tugenden-fehler.md:204`, Lightning Reflexes →
Blitzreflexe `:206`, Long-Winded → Ausdauernd `:207`, Luck → Glück `:208`,
Knight → Ritter `:250`, Landed Noble → Landadliger `:251`, Leper Magus →
Lepra-Magus `:29`, Life-Linked Spontaneous Magic → Lebensgebundene Spontane
Magie `:30`, Life Boost → Lebensstärkung `:58`, Lesser Benediction → Kleine
Segnung `:129`, Lesser Immunity → Mindere Immunität `:130`, Lesser Power →
Mindere Macht `:131`, Lesser Purifying Touch → Mindere Reinigende Berührung
`:132`, Light Touch → Leichte Berührung `:133`). **Three diverge** — F-130,
F-135, F-152 — and one more is an open question (Q-40).

### Cross-references followed

The brief calls this the audit's most productive pattern and invisible to any
per-entry screen, so **every pointer in the span was followed and read**, not
only the ones on entries that turned out to fail. Twelve pointers — both the
`(see page NNN)` form and the named-Virtue form — and what each lands on:

| Entry | Pointer | Where it lands | Does it add a rule attributed to the Virtue? |
|---|---|---|---|
| `virtue.jack_of_all_trades` | "(see page 157)" / `#fertigkeiten-ohne-wert` | `## Abilities Without a Score` at **ArMDE:7126-7128** | **No.** It is the *baseline* the Virtue overrides: ":7128 If the Ability is not asterisked … she may use it as if it had a score of zero, but rolling three extra botch dice. If it is asterisked, she cannot use it at all." The Virtue's own `description` already carries both halves. Clean. |
| `virtue.large` | "(See page 404)" / `#wunden` | the wound rules | **No** — and the rule it names *is computed*: A23's wound bands, `u = max(1, size + 5)` = 6 at Size +1. |
| `virtue.large` | Giant Blood p.83, Small Frame p.145, Dwarf p.126 | the three incompatible entries | **No new rule** — and the encoding is complete and **symmetric**, verified rather than assumed: `flaw.dwarf`, `flaw.small_frame` and `virtue.giant_blood` each list `virtue.large` back. |
| `virtue.lesser_immunity` | "See Greater Immunity, page 83" | `#### Greater Immunity`, **ArMDE:4009-4015** | **Yes, two.** → F-140, Q-38. |
| `virtue.lesser_purifying_touch` | "See page 406 for rules on diseases" | the disease rules | **No** — they belong to the disease system, not the Virtue. |
| `virtue.lesser_purifying_touch` | "Art and Academe, page 45" | **outside `rules/source/en/`** | Nothing claimable: CLAUDE.md forbids implementing from a book not in the sources. |
| `virtue.lesser_power` | "See Greater Power … if you want …" (implicit, :4281 "cost less Fatigue if created using the Greater Power Virtue") | ArMDE:4017-4025 | **No new rule for this entry** — but it confirmed the two Fatigue ladders differ, which is what makes F-142's text entry-specific. |
| `virtue.lasiq` | "The Cradle and the Crescent, from page 162" | outside `rules/source/en/` | Nothing claimable. |
| `virtue.magian_lineage_major/_minor` | "The Cradle and the Crescent, Chapter 5" | outside `rules/source/en/` | Nothing claimable. |
| `virtue.kassalan_exorcism` | "Lands of the Nile, from page 88" (inside the Kassalan Dust insert) | outside `rules/source/en/` | Nothing claimable. |
| `virtue.life_linked_spontaneous_magic` | "(see page 169)" on "levels of Penetration" | `### Penetration Total` / `## Penetration` | **No** — the general Penetration rules, not attributed to this Virtue. The Virtue's own clauses are F-148's. |
| `virtue.lone_redcap` | "the Well-Traveled virtue" (a **named-Virtue** reference, no page number) | `#### Well-Traveled`, **ArMDE:5239-5242** | **Yes** — and the grant delivers none of it. See the note under F-151; the defect itself is B09's. |
| `virtue.landed_noble` | "an Oath of Fealty … balance this Virtue with that Flaw. You get the normal points for Oath of Fealty if you do." (named-Flaw) | `flaw.oath_of_fealty` | **No new rule** — "the normal points" is explicitly *no* change to the ordinary balance arithmetic B3 already performs. Correctly encoded by encoding nothing. |

So one pointer yields rules the entry drops (`lesser_immunity` → F-140), one
yields rules a *granted* entry drops (`lone_redcap` → Well-Traveled, flagged for
B09), one yields a computed baseline (`large`), one yields the baseline the entry
already restates (`jack_of_all_trades`), one yields an explicit "no change"
(`landed_noble`), and four lead to books this repo does not ship.

**The named-Virtue form is the one a page-number screen cannot see**, and it
produced the more interesting of the two live hits. Recorded as a pattern below.

**Part C systemic gaps are not re-reported per entry.** In particular:
`special_casting_mod`'s `life_boost` and `life_linked_spontaneous` kinds being
surfaced-only (C1) is **not** counted as a defect of `virtue.life_boost`,
`virtue.life_linked_spontaneous_magic` or `virtue.leper_magus`; the
`health_mod` `fatigue_roll` track being surfaced-only (C1) is **not** counted
against `virtue.long_winded`; and `grants_reputation`'s unenforced `score` (C2)
is **not** counted against `virtue.lone_redcap`. What *is* counted for each of
those is what D5 now obliges — the clause the effect does not implement, absent
from both locales.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D5 is the decisive one for this batch** and is the reason eleven entries that
carry effects still fail. Per D5 the description obligation follows the *rule*,
not the class, so for every entry with `effects` the question asked was not only
"does each effect row match the passage" but "what else does the passage state
that no effect implements". Eleven of the twelve effect-carrying entries in the
span answered that with something:
`virtue.leper_magus` (F-137), `virtue.lesser_power` (F-142),
`virtue.life_boost` (F-147), `virtue.life_linked_spontaneous_magic` (F-148),
`virtue.lightning_reflexes` (F-149), `virtue.linguist` (F-150),
`virtue.lone_redcap` (F-151), `virtue.long_winded` (F-153),
`virtue.magian_lineage_minor`/`_major` (F-156, F-158), `virtue.magic_items`
(F-159). One entry answered it with nothing and is clean on that count —
`virtue.large`, whose "wounds … increase in six point intervals rather than five
point intervals" **is** computed (A23: `derived/combat.rs::wound_ranges` uses
`u = max(1, size + 5)`, which is 6 at Size +1).

**D3's precedent decides most of the classification moves.** An engine that
structurally cannot express a rule is grounds for `uncomputed_rule` with the
rule written into both locales, and is never grounds for `narrative`. That is
what moves `virtue.just_an_instant`, `virtue.kassalan_exorcism`,
`virtue.keen_vision`, `virtue.keen_sense_of_smell`,
`virtue.land_regio_network`, `virtue.landed_noble`,
`virtue.learn_ability_from_mistakes`, `virtue.leather_ripper`,
`virtue.lesser_benediction`, `virtue.lesser_immunity`,
`virtue.lesser_purifying_touch`, `virtue.license_of_absence` and
`virtue.luck`. Where the engine *can* express the rule the move is to
`creation_effect` instead: `virtue.jurist`, `virtue.knight`, `virtue.lasiq`,
`virtue.lupus_the_wolf` (all four `ability_authorization`).

**D1 / D4 — not exercised.** None of this batch's 35 entries carries a
`lab_total_mod`, and none is named in either ruling's table.

**D2 — not exercised.** No entry in this batch carries a
`characteristic_score_delta_param` or an `ability_bonus`, so neither
bought-only validator is touched and no verdict here depends on D2.

**Phase-0 open questions — two land in this span and are answered below.** OQ3
(`grants_reputation`'s `score`) is exercised by `virtue.lone_redcap`, whose
score 2 is corroborated by `rules/source/de/translation-tables/reputationen.md:111`;
what that row *adds* is a polarity the engine cannot store, which is **Q-43**.
OQ2 (Major vs Minor Magical Focus) belongs to B06, not here.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.jack_of_all_trades` | 4155-4158 | OK | OK | OK — full description, both locales | clean |
| `virtue.journeyman` | 4159-4162 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.jurist` | 4163-4168 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | male-only restriction in neither locale | F-122 F-123 ArMDE:2816 |
| `virtue.just_an_instant` | 4169-4172 | narrative → uncomputed_rule | OK | Awareness waiver in neither locale | F-124 |
| `virtue.kassalan_exorcism` | 4173-4186 | narrative → uncomputed_rule | OK | Casting-Total formula, Fatigue cost, Penetration rule, realm in neither locale | F-125 ArMDE:2960 |
| `virtue.keen_vision` | 4187-4190 | narrative → uncomputed_rule | OK | +3 in summary but not the missile-weapon exception | F-126 |
| `virtue.keen_sense_of_smell` | 4191-4194 | narrative → uncomputed_rule | OK | +3 in neither locale | F-127 |
| `virtue.knight` | 4195-4198 | narrative → creation_effect | effects missing (`ability_authorization`, martial) | male-only restriction in neither locale | F-128 F-123 ArMDE:2816 |
| `virtue.knows_people` | 4199-4206 | narrative → uncomputed_rule | OK | once-per-story bait entitlement + qualified veto in neither locale | F-160 (Q-35 closed) |
| `virtue.laborer` | 4207-4210 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.land_regio_network` | 4211-4218 | narrative → uncomputed_rule | OK | Ease Factor 9 + Diameter travel in neither locale; de name drops the hyphen and contradicts the table | F-129 F-130 ArMDE:2960 |
| `virtue.landed_noble` | 4219-4228 | narrative → uncomputed_rule | OK | Poor/Wealthy season obligations in neither locale | F-131 ArMDE:2816 Q-44 |
| `virtue.large` | 4229-4231 | OK | OK | OK | clean |
| `virtue.lasiq` | 4233-4236 | narrative → creation_effect | effects missing (`ability_authorization`, martial) | OK | F-132 Q-36 ArMDE:2816 |
| `virtue.latent_magic_ability` | 4237-4240 | OK | OK | OK | clean |
| `virtue.learn_ability_from_mistakes` | 4241-4244 | narrative → uncomputed_rule | OK | "five experience points" in neither locale | F-133 |
| `virtue.leather_ripper` | 4245-4248 | narrative → uncomputed_rule | OK | PeAn(He) 30 / +0 Pen / -9 / Fatigue in neither locale; de name contradicts the table | F-134 F-135 ArMDE:2960 |
| `virtue.leper_magus` | 4249-4252 | OK | both stated prerequisites missing | wound→vis table and extra Aging roll in neither locale | F-136 F-137 Q-37 |
| `virtue.lesser_benediction` | 4253-4274 | narrative → uncomputed_rule | no parameter for the chosen benediction | insert examples in neither locale | F-138 F-139 ArMDE:2960 |
| `virtue.lesser_immunity` | 4275-4278 | narrative → uncomputed_rule | no parameter for the chosen hazard | immunity + cross-referenced rules in neither locale | F-140 F-141 Q-38 ArMDE:2960 |
| `virtue.lesser_power` | 4279-4286 | OK | OK | Initiative formula, Fatigue costs, Penetration purchase in neither locale | F-142 ArMDE:2960 |
| `virtue.lesser_purifying_touch` | 4287-4290 | narrative → uncomputed_rule | no parameter for the chosen illness | Fatigue cost + illness-only limit in neither locale | F-143 F-144 ArMDE:2960 |
| `virtue.license_of_absence` | 4291-4294 | narrative → uncomputed_rule | both stated prerequisites missing | extra-free-season rule and cap of four in neither locale | F-145 F-146 Q-44 |
| `virtue.life_boost` | 4295-4298 | OK | OK | +5/level and the damage formula in neither locale | F-147 |
| `virtue.life_linked_spontaneous_magic` | 4299-4306 | OK | OK | per-five-points Fatigue rule in neither locale | F-148 |
| `virtue.light_touch` | 4307-4310 | OK | OK | OK — full description, both locales | clean |
| `virtue.lightning_reflexes` | 4311-4314 | OK | +9 Initiative folded in unconditionally | the 3+ gate in neither locale | F-149 |
| `virtue.linguist` | 4315-4318 | OK | OK | "All Advancement Totals" half in neither locale | F-150 Q-39 Q-40 |
| `virtue.lone_redcap` | 4319-4326 | OK | `supernatural` in the 300-point pool is unsourced (Q-41) | season obligations, two denials, Reputation polarity in neither locale; de name contradicts the table | F-151 F-152 Q-41 Q-42 Q-43 ArMDE:2816 |
| `virtue.long_winded` | 4327-4330 | OK | OK | "does not apply to casting spells" in neither locale | F-153 |
| `virtue.luck` | 4331-4334 | narrative → uncomputed_rule | OK | "+1 to +3" in neither locale | F-154 |
| `virtue.lupus_the_wolf` | 4335-4338 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | OK | F-155 ArMDE:2816 |
| `virtue.magian_lineage_major` | 4339-4346 | OK | no parameter for the three chosen Abilities | XP-sharing rule + "+3 to resist disease" in neither locale | F-157 F-158 |
| `virtue.magian_lineage_minor` | 4339-4346 | OK | OK | "+3 to resist disease" in neither locale | F-156 |
| `virtue.magic_items` | 4347-4350 | OK | OK | improvement rate + Level 30 per-effect cap in neither locale | F-159 |

**Totals (post-reconciliation): 29 entries carry at least one finding, 6 are
clean**: `virtue.jack_of_all_trades`, `virtue.journeyman`, `virtue.laborer`,
`virtue.large`, `virtue.latent_magic_ability`, `virtue.light_touch`.
29 + 6 = 35. No entry is left carrying only a question.

*(One entry left the question-only list during reconciliation.
`virtue.knows_people` was rated `?` in the first pass and the verification agent
overturned that to a defect — **F-160**, which closes **Q-35**. See "Sub-agent
reconciliation".)*

## Findings

**A qualifier that applies to all four `ability_authorization` findings (F-122,
F-128, F-132, F-155) and is stated once rather than four times.** A14 records
that `validation/authorization.rs::validate_ability_authorization` exempts any
character whose type profile has `is_magus`, and that the gated set is
`ruleset.categories_requiring_virtue()`. That set is
`rules/core/abilities.json` → `"categories_requiring_virtue": ["academic",
"arcane", "martial"]` — so **academic and martial really are gated**, and the
Abilities named in these four passages resolve to gated categories:
`ability.artes_liberales` → `academic`, `ability.civil_and_canon_law` →
`academic`, `ability.dead_language` → `academic` (and
`"scholarly_language": {"ability": "ability.dead_language", "exemplar":
"latin"}`, so the book's "Latin" is this id). All four Virtues are Social
Status Virtues for mundane characters — jurists, knights, assassins, university
beadles — so the `is_magus` exemption is empty in practice and the refusal is
exactly what bites. A7 also records that a `restricted_ability_xp` pool already
confers the same permission; **none of these four carries one**, so there is no
redundancy argument here (unlike B04's `virtue.hermetic_experience` and
`virtue.ineslemen`).

**A second qualifier, for every `narrative → uncomputed_rule` move below.** B1
records that `classification` is read by no production code, so none of these
moves changes a computed number. The cost is the one `ecb5150` names: a
`narrative` entry is not obliged to carry its rule in `description`, so the rule
leaves the application silently while the entry still looks complete. Every move
below therefore carries the same correction — reclassify **and** write the rule
into `description` in **both** locales, or
`uncomputed_clauses.rs::every_uncomputed_rule_entry_states_its_rule_in_every_locale`
goes red. Severity for all of them: **lost-rule / provenance**, not
miscalculation.

---

### F-122 — `virtue.jurist` — `narrative` on a Virtue that authorizes three gated Academic Abilities, with no `ability_authorization`

**Passage** (ArMDE:4165, verbatim, English — the relevant sentence):
> At character generation he may purchase the Abilities Latin, Artes Liberales, and Civil and Canon Law.

German, ArMDE:4165 (line-parallel):
> Bei der Charaktererschaffung darf er die Fertigkeiten Latein, Artes Liberales sowie Zivil- und Kanonisches Recht erwerben.

**Current data:** `"classification": "narrative"`, no `effects`, no
`prerequisites`, no `parameters`.

**Why it is wrong.** All three named Abilities are `academic`, and `academic` is
in `categories_requiring_virtue`. So a companion with the Jurist Virtue and a
score in Artes Liberales is told `ability_category_requires_virtue` by
`validate_ability_authorization` — the app refuses the exact character the
passage describes. This is the shape B04 found twice (F-101 Guild Dean, F-102
Guild Master) and it has now been found in four consecutive spans.

Note the wording is *stronger* than "may take": "may **purchase**", "darf …
**erwerben**". It is a character-generation permission, not an in-play one.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "abilities": ["ability.artes_liberales",
"ability.civil_and_canon_law", "ability.dead_language"]}`. Whether the whole
`academic` category or these three ids is the right scope is a judgement the
passage settles by naming three Abilities and not a category — so `abilities`,
not `categories`, unlike Knight and Lasiq below.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-123 — `virtue.jurist` and `virtue.knight` — "only available to male characters" is a rule that reaches neither locale

**Passages** (verbatim, English):
> This Virtue is compatible with the Baccalaureus, Magister in Artibus, and Doctor in (Faculty) Virtues, as a jurist may have a university education. It is also compatible with the Priest and Mendicant Friar Virtues. **It is only available to male characters.** (ArMDE:4167)

> This Virtue is only available to male characters, and is compatible with the Landed Noble Virtue. (ArMDE:4197)

German (line-parallel):
> Sie steht nur männlichen Charakteren zur Verfügung. (ArMDE:4167)

> Diese Tugend steht nur männlichen Charakteren zur Verfügung und ist kompatibel mit der Tugend Landadliger. (ArMDE:4197)

**Current data:** neither entry carries anything encoding it, and neither
`summary` mentions it in either locale.

**Why it is a finding and not merely an engine gap.** The engine genuinely
cannot express it: `types.rs`'s concept field declares
`pub gender: String` with the doc comment "The character's gender (free-text;
**no mechanical effect**)", so there is no enum to test and no `Prereq` variant
that could reach it. That is precisely D3's case — structurally inexpressible is
grounds for `uncomputed_rule` with the rule written out, and **never** grounds
for dropping it. Today it is dropped: a player building a female knight gets no
warning and no text.

**Correct value:** carry the sentence in `description` in both locales. For
Jurist that rides along with F-122's reclassification; for Knight with F-128's.
*(Recorded as one finding rather than two because it is one sentence, appearing
twice, with one correction.)*

**Severity:** lost-rule / provenance.

---

### F-124 — `virtue.just_an_instant` — `narrative` on a rule that waives a roll outright

**Passage** (ArMDE:4171, verbatim, English — the relevant sentence):
> A character with this Virtue, who has had a few seconds to look about a room, does not need to make Awareness checks to detect or remember anything of interest on the room's exposed surfaces.

German, ArMDE:4171 (line-parallel):
> Ein Charakter mit dieser Tugend, der einige Sekunden Zeit hatte, einen Raum zu überblicken, muss keine Wahrnehmungswürfe ablegen, um etwas Interessantes auf den zugänglichen Oberflächen des Raumes zu entdecken oder sich daran zu erinnern.

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** "Does not need to make Awareness checks" is an automatic
success against a named Ability — the same class of absolute that `ecb5150`
settled is "as mechanical as a +3", and the same shape as B04's three immunities
(F-95, F-110, F-111). The rest of the entry is colour; this sentence is not.

**Correct value:** `"classification": "uncomputed_rule"`, with the waiver in
`description` in both locales. The engine has no roll-waiver variant, so per D3
it stays text.

**Severity:** lost-rule / provenance.

---

### F-125 — `virtue.kassalan_exorcism` — `narrative` on an entry that states a complete casting subsystem

**Passage** (ArMDE:4177, verbatim, English — the second paragraph in full):
> To cast a spell you must expend a Fatigue level, whether the spell succeeds or fails. You must also spend at least 5 minutes preparing the spell, during which time you need unbroken concentration and expend one handful of Kassalan Dust. The spell is designed exactly like a Hermetic spell; if affecting a ghost you must be able to sense your target. Your Casting Total is equal to (Stamina + Organization Lore: Mortuary Society + Aura modifier + stress die) / 2. Penetration is calculated in the normal fashion: Casting Total – Spell Level + Penetration modifiers. Kassalan Exorcism is aligned to the Magic Realm.

and from the first paragraph (ArMDE:4175):
> Wards created with this Virtue need not be circular. but can instead block the threshold(s) of a room; this is still treated as a Duration: Ring, Target: Circle spell for level calculations.

German, ArMDE:4177 (line-parallel):
> … Deine Zaubersumme beträgt (Ausdauer + Organisationskunde: Bestattungsgesellschaft + Aura-Modifikator + Stresswürfel) / 2. Die Penetration wird auf normale Weise berechnet: Zaubersumme – Stufe des Zaubers + Penetrationsmodifikatoren. Kassalanischer Exorzismus ist der Magischen Sphäre zugeordnet.

**Current data:** `"classification": "narrative"`, no `effects`, no
`parameters`.

**Why it is wrong.** This is the densest `narrative` misclassification in the
batch. The passage states a **division-by-two formula with four named terms**, a
**Penetration formula**, a **Fatigue cost**, a **preparation time**, a
**material component**, a **Duration/Target substitution for level
calculation**, and an explicit **realm alignment** — seven mechanical clauses,
none of which is a judgement call. The whole-entry claim that "the book states
nothing mechanical" is false seven times over.

Note also that the cited range 4173-4186 *includes* the `> #### Kassalan Dust`
block quote (4179-4185), which carries "+3 to uses of the Corpse Magic
Supernatural Ability", "Multiple handfuls provide no additional effect" and "a
Shape & Material bonus of +3 against the dead". So even a token screen would
fire on this range the moment `SWEPT_BLOCKS` widens past :3950 — the same
mechanism B04's F-121 identified for `greater_benediction`.

**Correct value:** `"classification": "uncomputed_rule"`, with at minimum the
Casting Total formula, the Penetration formula, the Fatigue cost and the realm
alignment in `description` in both locales. The engine's casting machinery is
Hermetic (`derived/casting.rs::casting_totals`) and has no halved-non-Hermetic
path, so per D3 this stays text.

**ArMDE:2960 instance, and the strongest kind.** The passage does not merely
default a realm, it **fixes** one ("aligned to the Magic Realm" / "ist der
Magischen Sphäre zugeordnet"), yet the entry carries no `realm` parameter. This
is a harder case than B04's `virtue.hex` ("most often associated with the
Infernal realm"), which was correctly left alone.

**Severity:** lost-rule / provenance, on seven clauses.

---

### F-126 — `virtue.keen_vision` — `narrative` on a plain "+3"

**Passage** (ArMDE:4189, verbatim, English — the entry's whole body):
> You can see farther and more clearly than most. You get a +3 bonus to all rolls involving sight, not including attacks with missile weapons.

German, ArMDE:4189 (line-parallel):
> Du kannst weiter und klarer sehen als die meisten. Du erhältst einen +3-Bonus auf alle Würfe, die das Sehen beinhalten, ausgenommen Angriffe mit Fernkampfwaffen.

**Current data:** `"classification": "narrative"`, no effects. The `summary`
reads "+3 to sight-based rolls." / "+3 auf sichtbasierte Würfe." — **the entry's
own summary states the number the classification claims does not exist.** That
internal contradiction is the whole finding in one line.

**Why it is wrong.** A signed numeric modifier is the paradigm case of a
mechanical clause. The exception clause ("not including attacks with missile
weapons") is a second one, and it is in neither `summary`.

**Correct value:** `"classification": "uncomputed_rule"`, with the full sentence
including the exception in `description` in both locales. A `combat_mod`/
`ability_roll_mod` would not do: the bonus is to "all rolls involving sight",
not to a named Ability or combat stat, and A41 records `ability_roll_mod` as
surfaced-only anyway.

**Severity:** lost-rule / provenance, with the exception clause genuinely
unreachable today.

---

### F-127 — `virtue.keen_sense_of_smell` — `narrative` on a plain "+3"

**Passage** (ArMDE:4193, verbatim, English — the entry's whole body):
> The character's nose is more sensitive than normal for a human. He gets a +3 bonus to all rolls involving his sense of smell, including following tracks with the Hunt Ability.

German, ArMDE:4193 (line-parallel):
> Die Nase des Charakters ist empfindlicher als bei einem normalen Menschen. Er erhält einen +3-Bonus auf alle Würfe, die seinen Geruchssinn beinhalten, einschließlich des Spurenverfolgens mit der Fertigkeit Jagen.

**Current data:** `"classification": "narrative"`, no effects. Unlike its
neighbour F-126, **neither locale's `summary` carries the number** — it stops at
"The character's nose is more sensitive than normal for a human." / "Die Nase
des Charakters ist empfindlicher als bei einem normalen Menschen." So the +3 and
the named Hunt application reach the user nowhere at all.

**Correct value:** `"classification": "uncomputed_rule"`, with the whole
sentence in `description` in both locales.

**Severity:** lost-rule / provenance.

---

### F-128 — `virtue.knight` — "You may take Martial Abilities during character generation" with no `ability_authorization`

**Passage** (ArMDE:4197, verbatim, English — the relevant sentences):
> Unless you are Poor, you may have high quality weapons and armor, and a horse. … You may take Martial Abilities during character generation. The Wealthy Virtue and Poor Flaw affect you normally.

German, ArMDE:4197 (line-parallel):
> Sofern du nicht Arm bist, darfst du hochwertige Waffen und Rüstungen sowie ein Pferd besitzen. … Du kannst bei der Charaktererschaffung Kampffertigkeiten nehmen. Die Tugend Wohlhabend und der Fehler Arm betreffen dich normal.

**Current data:** `"classification": "narrative"`, **no effects at all**.

**Why it is wrong.** `martial` is in `categories_requiring_virtue`, so a knight
who buys Single Weapon is refused by `validate_ability_authorization`. This is
the "no effect at all" shape of the open pattern, not the "wrong field" shape:
the entry is empty, so nothing can be repaired in place. Compare
`virtue.warrior`, which is the same permission plus funding and carries
`{"type":"restricted_ability_xp","amount":50,"categories":["martial"]}` — A7's
permission side-effect is why *it* works. Knight funds nothing, so it needs the
authorization row in its own right.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "categories": ["martial"]}`. The passage
grants the **whole category** ("Martial Abilities" / "Kampffertigkeiten"), so
`categories`, not `abilities`.

**Severity:** wrong rules output — the app blocks a legal character, and a
knight who cannot buy a weapon Ability is the central case of this Virtue.

---

### F-129 — `virtue.land_regio_network` — `narrative` on an Ease Factor and a fixed travel Duration

**Passage** (ArMDE:4215, verbatim, English — the second paragraph in full):
> Once the character is in such a regio he may travel to any other regio in the network; travel time is one Diameter regardless of geographic distance. To determine whether the character knows the location of a networked regio in a particular locale an Intelligence + (Area) Lore roll is made against an Ease Factor of 9. The character knows how to enter these regiones and can, if he chooses, guide other characters into the regiones — although other characters must be able to survive in the environment of the regio.

German, ArMDE:4215 (line-parallel):
> Sobald der Charakter sich in einer solchen Regio befindet, kann er zu jeder anderen Regio im Netz reisen; die Reisezeit beträgt einen Durchmesser, unabhängig von der geografischen Entfernung. Um festzustellen, ob der Charakter den Standort einer vernetzten Regio in einem bestimmten Gebiet kennt, wird ein Intelligenz + Gebietskunde-Wurf gegen einen Schwierigkeitsgrad von 9 vorgenommen. …

**Current data:** `"classification": "narrative"`, no effects, one parameter
(`{"key":"land","type":"ref","domain":"text"}`).

**Why it is wrong.** A named roll (Intelligence + Area Lore) against a stated
**Ease Factor of 9**, plus a fixed Duration for travel. The canonical
translation table records the same two facts as the entry's defining mechanics —
`tugenden-fehler.md:127` "SdM:M; Zugang zu Netzwerk von Regiones; **Reisezeit 1
Durchmesser**" — which is independent confirmation that these are read as rules
and not as colour.

**The parameter itself is correct.** "The character is associated with one
particular type of regio network" (:4213) demands a choice; the `land` text
parameter records it and is rendered in both names. B8 records that a parameter
no effect consumes is the normal shape for a recorded player choice, so this is
not a defect.

**Correct value:** `"classification": "uncomputed_rule"`, with the roll, the
Ease Factor and the travel Duration in `description` in both locales.

**Severity:** lost-rule / provenance.

---

### F-130 — `virtue.land_regio_network` — the German name drops a hyphen and contradicts the canonical table

**Current data:**

| Locale | `name` |
|---|---|
| en | `{land} Regio Network` |
| de | `{land}Regio-Netz` |

**Why it is wrong, twice over.**

1. **It is not renderable German.** The DE rulebook heading at ArMDE:4211 is
   `#### (Land-)Regio-Netz` — the hyphen after the placeholder is part of the
   compound. With the placeholder filled the shipped template produces
   *"SeeRegio-Netz"*, not *"See-Regio-Netz"*. The English template has the space
   it needs; the German one has lost the hyphen that does the same job.
2. **It contradicts the canonical table.**
   `rules/source/de/translation-tables/tugenden-fehler.md:127` reads
   `| (Land) Regio Network | (Land-)Regionetzwerk | SdM:M; … |`, in the
   `### Übernatürliche Tugenden, Klein / Supernatural, Minor` block — the same
   block, four lines above, from which B04 took F-107/F-108's ruling on
   `virtue.homing_instinct` (`:126`, "Ortsgespür"). CLAUDE.md states the rule
   flatly: "the German label for any term whose English form appears in a table
   MUST match the table's `Deutsch (DE)` value."

**Correct value:** `Regionetzwerk` per the table, with the hyphen the compound
needs — i.e. `{land}-Regionetzwerk`. *(The table's `(Land-)Regionetzwerk` and the
rulebook heading's `(Land-)Regio-Netz` disagree on `Regionetzwerk` vs
`Regio-Netz`; the table wins by CLAUDE.md's rule, exactly as it did for
`homing_instinct`, where the rulebook and the table also disagreed and the table
was taken.)*

**Severity:** localization defect — user-visible in every session, and this repo
rates those up.

---

### F-131 — `virtue.landed_noble` — `narrative` on two season obligations and two explicit Poor/Wealthy interactions

**Passage** (ArMDE:4221-4225, verbatim, English — the mechanical sentences):
> You are wealthier than most characters, but have no additional free time. (ArMDE:4221)

> If you are Poor, your fief is either very small, or in a poor area for farming with few other resources. **You must spend every season managing it**, or it may collapse completely, leaving you effectively landless. You are no wealthier than most average characters, and you have only a couple of servants. (ArMDE:4223)

> Wealthy Landed Nobles control more than one fief, and have bailiffs or stewards for each, **so that they do not need to devote any time to looking after their lands**. (ArMDE:4225)

German, ArMDE:4223 (line-parallel):
> Bist du Arm, ist dein Lehen entweder sehr klein oder liegt in einer landwirtschaftlich schlechten Gegend mit wenigen anderen Ressourcen. Du musst jedes Quartal damit verbringen, es zu verwalten, sonst könnte es völlig zusammenbrechen und dich effektiv landlos zurücklassen. …

**Current data:** `"classification": "narrative"`, no effects, no
`incompatible_with`.

**Why it is wrong.** Three mechanical clauses, all of them about the year's four
seasons — the same currency `virtue.license_of_absence` (:4293) and
`virtue.lone_redcap` (:4323) trade in, and the same currency the app's
post-apprenticeship advancement config already counts
(`max_charged_lab_seasons_per_year`). "You must spend every season managing it"
is an absolute: it leaves zero free seasons. And the passage explicitly
*modifies* the Wealthy Virtue and Poor Flaw, which are themselves
`creation_effect` entries carrying `later_life_xp_rate` 20 and 10 — so this is
not a decorative aside but an interaction between three modelled entries.

**Correct value:** `"classification": "uncomputed_rule"`, with the season
obligations and the Poor/Wealthy variations in `description` in both locales.
No Effect variant expresses free seasons per character (**Q-44**), so per D3
this stays text.

**Severity:** lost-rule / provenance.

---

### F-132 — `virtue.lasiq` — "may take Martial Abilities at character creation" with no `ability_authorization`

**Passage** (ArMDE:4235, verbatim, English — the relevant sentence):
> Lasiq may take Martial Abilities at character creation.

German, ArMDE:4235 (line-parallel):
> Lasiqs dürfen bei der Charaktererschaffung Kampffertigkeiten nehmen.

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** Identical to F-128: `martial` is gated, the entry grants
the whole category, and nothing in the data expresses it. An assassin who cannot
buy Single Weapon is the Virtue's central case failing.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "categories": ["martial"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-133 — `virtue.learn_ability_from_mistakes` — `narrative` on "you gain five experience points"

**Passage** (ArMDE:4243, verbatim, English — the entry's whole body):
> You are able to improve a particular Ability through the expedient of repeated failure. The first time in a given game session that you botch a roll or fail by exactly one point, you gain five experience points in the Ability. The roll must have come up naturally in the course of the story. You may take this Virtue several times, once for each Ability chosen.

German, ArMDE:4243 (line-parallel):
> Du bist in der Lage, eine bestimmte Fertigkeit durch wiederholtes Scheitern zu verbessern. Beim ersten Mal in einer gegebenen Spielsitzung, dass du einen Wurf verpatzst oder um genau einen Punkt verfehlst, erhältst du fünf Erfahrungspunkte in der Fertigkeit. Der Wurf muss sich natürlich im Verlauf der Geschichte ergeben haben. Du kannst diese Tugend mehrmals nehmen, je einmal für jede gewählte Fertigkeit.

**Current data:** `"classification": "narrative"`, no effects, one parameter
(`{"key":"ability","type":"ref","domain":"ability"}`).

**Why it is wrong.** "Five experience points" is a number, and its trigger
("botch, or fail by exactly one point") is a precise, non-judgement condition.
The book even fixes the frequency ("the first time in a given game session").
This is three mechanical clauses under a `narrative` tag.

**The multiplicity data is correct and was checked rather than assumed.** "You
may take this Virtue several times, once for each Ability chosen" needs (a)
unlimited copies and (b) at most one per Ability. `max_total` is absent (= 255,
"no stated ceiling") and `max_per_target` is absent (= 1, keyed on the **whole**
params map per B10). Since `ability` is the item's only parameter, the
`(item_ref, params)` key *is* the Ability, so `max_per_target: 1` already
enforces "once for each Ability". This is the opposite of B04's
`greater_immunity` defect, where `max_per_target: 255` with **no** parameter
permitted 255 indistinguishable copies.

**Correct value:** `"classification": "uncomputed_rule"`, with the trigger, the
5 XP and the once-per-session limit in `description` in both locales.

**Severity:** lost-rule / provenance.

---

### F-134 — `virtue.leather_ripper` — `narrative` on a spell-level equivalent, a Penetration value, a penalty range and a Fatigue cost

**Passage** (ArMDE:4247, verbatim, English — the entry's whole body):
> This character has the supernatural ability to destroy a group of leather objects. This causes metal armor to fall away, as the strips holding it together disintegrate. It destroys the tack of a horseman, forcing a Ride roll, with penalties of up to -9 depending on how dangerous his current maneuvers are. It can destroy the scabbard, belt and boots of a foe, allowing the ripper to flee or attack. This is a PeAn(He) 30 effect with +0 Penetration. The character must concentrate for a moment and spend a Fatigue level to activate the power, but does not need to speak or gesture.

German, ArMDE:4247 (line-parallel):
> … Es zerstört das Zaumzeug eines Reiters und erzwingt einen Reitenwurf mit Abzügen von bis zu -9, je nach der Gefährlichkeit seiner aktuellen Manöver. … Dies ist ein PeAn(He) 30-Effekt mit +0 Penetration. Der Charakter muss sich einen Moment konzentrieren und eine Erschöpfungsstufe aufwenden, um die Kraft zu aktivieren, muss aber nicht sprechen oder gestikulieren.

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** Four mechanical clauses: a named roll with a penalty up to
**-9**, a **level-30 PeAn(He)** effect, a **+0 Penetration**, and a **Fatigue
cost**. Note that a Virtue of the neighbouring Lesser Power family, given a
level, gets a `power_levels` effect (`virtue.lesser_power`, 25) — so the
catalogue already has a variant that would express the level here, and this
entry not only lacks it but is classed as stating nothing at all.

**Correct value:** `"classification": "uncomputed_rule"` at minimum, with all
four clauses in `description` in both locales. Whether the 30 levels should
instead become a `power_levels` effect is a modelling question the passage does
not settle — the level describes the power's *strength*, not a budget the player
spends — so the conservative correction is text.

**Severity:** lost-rule / provenance, on four clauses including two plain
numbers.

---

### F-135 — `virtue.leather_ripper` — the German name contradicts the canonical table

**Current data:** de `name` = `Lederreißer`.

**The table:** `rules/source/de/translation-tables/tugenden-fehler.md:128`
reads `| Leather Ripper | Lederzerreißer | |`, in the
`### Übernatürliche Tugenden, Klein / Supernatural, Minor` block — one line
below the `(Land) Regio Network` row of F-130 and two below the
`Homing Instinct` row B04 used for F-107/F-108.

**Corroboration from the same table.** The parallel entry `Fabric Ripper` is
mapped `Stoffzerreißer` at `:118`, in the same block. So the table is internally
consistent about the `-zerreißer` stem for the "Ripper" family, and
`Lederreißer` is the outlier.

**Why the DE rulebook is not the answer here.** The DE rulebook heading at
ArMDE:4245 does read `#### Lederreißer`, matching the shipped i18n. That is the
inverse of B04's cases, where the rulebook agreed with the table. CLAUDE.md
resolves it: the tables are "the **canonical** EN→DE terminology mapping" and
the i18n label "MUST match the table's `Deutsch (DE)` value". B04 applied that
rule in the same direction for `virtue.homing_instinct` and
`virtue.independent_study`.

**Correct value:** `Lederzerreißer`.

**Severity:** localization defect.

---

### F-136 — `virtue.leper_magus` — both stated prerequisites are missing, and both are expressible

**Passage** (ArMDE:4251, verbatim, English — the relevant sentence):
> This Virtue can only be bought if the character also has the Leprosy Flaw, and is only available to magi trained in House Tytalus.

German, ArMDE:4251 (line-parallel):
> Diese Tugend kann nur genommen werden, wenn der Charakter auch den Fehler Lepra hat, und steht nur Magi zur Verfügung, die im Haus Tytalus ausgebildet wurden.

**Current data:** `"prerequisites": null`. No `incompatible_with`.

**Why it is wrong, and why this one is not a D3 case.** Every part of the
sentence is directly expressible in `types.rs::Prereq`:
`{"kind":"all","value":[{"kind":"has","value":"flaw.leprosy"},
{"kind":"house","value":"house.tytalus"}]}`. Both targets exist in the shipped
data — `flaw.leprosy` is in `rules/core/virtues_flaws.json` (ArMDE:6338-6341,
carrying two `aging_mod` rows) and `house.tytalus` is one of the twelve ids in
`rules/core/houses.json`. So this is not "the engine cannot"; it is a
requirement that was simply never encoded, and B6 records that
`validate_prerequisites` would raise `prereq_not_met` as an **error** the moment
it is.

Note the verb, per the D2 vocabulary B02 and B04 collected: "can only be
**bought**" / "kann nur **genommen** werden" — an unambiguous purchase
condition, so no D2 ambiguity arises and `validate_prerequisites`' bought-only
scope (B6) is exactly right for it.

**Correct value:** the `all`/`has`/`house` tree above.

**Severity:** wrong rules output — the app permits an illegal character
(a non-Tytalus, non-leprous magus with Leper Magus) with no diagnostic.

---

### F-137 — `virtue.leper_magus` — the wound→vis table and the extra Aging roll reach neither locale (D5)

**Passage** (ArMDE:4251, verbatim, English — the mechanical clauses):
> By accepting a Light Wound, the magus can infuse a single magical working with three pawns of vis, of any Technique or Form. A Medium Wound supplies six pawns, a Heavy Wound nine pawns, an Incapacitating Wound 12 pawns, and a Deadly Wound (killing the magus) 15 pawns. … All Lab Totals suffer the wound penalty as normal, as do the Casting Totals of spells that take more than one round to cast (such as Ritual spells). … A wound taken in this fashion must heal completely before the power may be used again, and any character using this power more than three times a year must make an extra Aging roll in winter.

German, ArMDE:4251 (line-parallel):
> Indem er eine Leichte Wunde akzeptiert, kann der Magus ein einzelnes magisches Wirken mit drei Bauern Vis beliebiger Technik oder Form versorgen. Eine Mittelschwere Wunde liefert sechs Bauern, eine Schwere Wunde neun Bauern, eine Lähmende Wunde zwölf Bauern und eine Tödliche Wunde (die den Magus tötet) fünfzehn Bauern. … jeder Charakter, der diese Kraft mehr als dreimal im Jahr einsetzt, muss im Winter einen zusätzlichen Alterungswurf ablegen.

**Current data:** one effect,
`{"type":"special_casting_mod","kind":"life_boost"}`; `summary` in both locales
stops at the first sentence ("This Virtue describes the mystic legacy passed on
from Tytalus …"); **no `description` in either locale**.

**Why it is a finding under D5.** The single effect implements the *Life Boost
grant* and nothing else (and per C1 even that is surfaced-only, which is not
counted here). Five wound tiers mapped to five vis amounts, a
three-uses-per-year threshold, an extra Aging roll, and the Lab-Total/Ritual
wound-penalty rule are all stated and all uncomputed. D5: "any mechanical clause
the engine does not compute must be written into `description`, in both locales,
whatever the entry's classification."

**Correct value:** `description` in both locales carrying the wound→vis table
(3/6/9/12/15), the healing precondition, the three-times-a-year Aging roll and
the wound-penalty rule. `classification` stays `in_play_effect` — B1 records
that carrying an `Effect` is what decides that line.

**Severity:** lost-rule / provenance, on the entry's entire distinctive
mechanic.

---

### F-138 — `virtue.lesser_benediction` — `narrative` over a cited range that contains a "-3" and a yield rule

**Passage** (ArMDE:4255, verbatim, English — the entry's own body):
> You have been blessed by some supernatural power. The effects of the benediction should be comparable to other Minor Virtues. (See insert for examples.)

and, inside the **same cited range** (4253-4274), the insert:
> You are a very convincing speaker, anyone attempting to detect untruth in your words receives a -3 penalty to their rolls. (ArMDE:4261, "Gift of the Gab")

> Plants always prosper under your care. Your crops never suffer from natural diseases or pests so long as you personally tend to them, and you can therefore get half as much again in terms of yield as others. (ArMDE:4265, "Green Fingers")

> Every sexual encounter with a partner of the opposite sex results in conception. (ArMDE:4273, "Unusually Fecund")

German, ArMDE:4261 (line-parallel):
> Du bist ein sehr überzeugender Redner; jeder, der versucht, Unwahrheiten in deinen Worten aufzuspüren, erhält einen -3-Abzug auf seine Würfe.

**Current data:** `"classification": "narrative"`, `"source": {"lines": [4253,
4274]}`.

**Why it is wrong — and this is exactly B04's F-121, one magnitude down.**
Whatever one concludes about whether the insert's numbers belong to the *Virtue*
or to its *examples* (that reading is B04's Q-24 and is not re-litigated here),
`classification: "narrative"` and `source.lines: [4253, 4274]` make
**incompatible claims about the same bytes**: the cited range contains "-3"
(:4261), "half as much again" (:4265) and an absolute ("always", "never",
"Every … results in conception"). `uncomputed_clauses.rs::bracketed_passage`
slices exactly the cited range, and there is no `NO_RULE_DESPITE_TOKEN` row for
this id — so the repo's own guard goes red on this entry the moment
`SWEPT_BLOCKS` widens past :3950, precisely as it does for
`virtue.greater_benediction`.

*(Evidence for the exemption claim, stated precisely rather than as a bare
negative: `grep -n "\"virtue\." crates/arm-rules/tests/uncomputed_clauses.rs`
returns **nothing at all** — every `NO_RULE_DESPITE_TOKEN` row in the file names
a `flaw.` id, so no Virtue anywhere in the catalogue is exempted, this one
included. And `SWEPT_BLOCKS` is confirmed to be exactly two ranges,
ArMDE:5639-7113 and ArMDE:3360-3950, with its own comment stating "The rest of
the block (ArMDE:3951-5282) is still unswept" — so the whole of this batch's
4155-4350 lies outside it.)*

That the *same* defect occurs on the Major and Minor members of one pair is
itself the evidence that it is structural rather than a one-off transcription
slip.

**Correct value:** reclassify (to `uncomputed_rule`, with the four example
benedictions written into `description` in both locales) **or** renarrow
`source.lines` to 4253-4255 so the insert is not claimed. B04 left the same
choice open as its Q-24; this batch records the second instance rather than
deciding it.

**Severity:** lost-rule / provenance, plus a latent test-guard failure.

---

### F-139 — `virtue.lesser_benediction` — no parameter records which benediction was taken

**Passage** (ArMDE:4255, verbatim):
> You have been blessed by some supernatural power. The effects of the benediction should be comparable to other Minor Virtues. (See insert for examples.)

**Current data:** `"parameters": null`, `max_per_target` absent (= 1).

**Why it is wrong.** The Virtue is meaningless until the player names the
blessing — the insert exists solely to give four candidates, and two characters
holding this Virtue hold two different things. Nothing in the saved entity
records which. B8 shows the right shape and the catalogue already uses it: a
`text`-domain parameter, exactly as `virtue.land_regio_network` uses for its
`land`. This is the same gap B04 raised as F-121's data half for
`virtue.greater_benediction`.

**Correct value:** a `text`-domain parameter (e.g. `{"key": "benediction",
"type": "ref", "domain": "text"}`), rendered in the name.

**Severity:** data loss — a player choice the save cannot hold, so a
round-tripped character loses which benediction it has.

---

### F-140 — `virtue.lesser_immunity` — `narrative` on an absolute immunity, and the cross-reference carries two further rules

**Passage** (ArMDE:4277, verbatim, English — the entry's whole body):
> You are immune to some hazard which is either rare, or not deadly, or both. See Greater Immunity, page 83.

German, ArMDE:4277 (line-parallel):
> Du bist immun gegen eine Gefahr, die entweder selten oder nicht tödlich ist – oder beides. Siehe Große Immunität, [Seite 83](#große-immunität).

**Where the pointer leads.** Page 83 / `#große-immunität` is
`#### Greater Immunity` at ArMDE:4009, which states, verbatim:
> You may not take immunity to aging — see the Unaging Minor Virtue (page 114) instead. **This immunity applies to mundane and magical versions of the thing.** If you are immune to fire, you are also immune to magically created fire. (ArMDE:4011)

> You may take this Virtue more than once, with a different immunity each time. (ArMDE:4015)

**Current data:** `"classification": "narrative"`, no effects, no parameters,
`max_per_target` absent (= 1), `max_total` absent (= 255).

**Why it is wrong.** "You are immune to X" is an absolute — the same clause B04
rated a finding three times over (F-95 `greater_immunity`, F-110
`immune_to_disease`, F-111 `immunity_to_cold`), and B04 moved all three to
`uncomputed_rule`. The Minor member of the same family is still `narrative`,
which means the catalogue is internally inconsistent about the identical
sentence *after* B04 read it. Beyond that, following the "See Greater Immunity"
pointer — this batch's brief's second open pattern, and the audit's most
productive one — yields two more rules that are attributed to this Virtue by its
own text and appear nowhere in its data: the aging-immunity prohibition
(redirecting to `virtue.unaging`, which exists and carries `no_aging` +
`no_apparent_aging`) and the mundane-and-magical scope.

**Correct value:** `"classification": "uncomputed_rule"`, with the immunity, the
mundane-and-magical scope and the aging prohibition in `description` in both
locales.

**Severity:** lost-rule / provenance, on an entry whose entire content is a
pointer.

---

### F-141 — `virtue.lesser_immunity` — no parameter records which hazard

**Passage** (ArMDE:4277, verbatim):
> You are immune to some hazard which is either rare, or not deadly, or both.

**Current data:** `"parameters": null`.

**Why it is wrong.** Identical in shape to F-139 and to B04's F-94/F-97: the
choice is the entry's whole content and the save cannot hold it. Two characters
both holding `virtue.lesser_immunity` are indistinguishable in the data, and the
Markdown export prints the same line for an immunity to seasickness and an
immunity to bee stings.

**Correct value:** a `text`-domain parameter recording the hazard.

**Severity:** data loss.

---

### F-142 — `virtue.lesser_power` — the Initiative formula, the Fatigue costs and the Penetration purchase reach neither locale (D5)

**Passage** (ArMDE:4281, verbatim, English — the first paragraph in full):
> The character has a supernatural power that he can activate at will. This is one or more powers. equivalent to Formulaic Hermetic spells with total levels of 25 or lower. Each power has an Initiative equal to the character's Quickness - (twice power magnitude). It costs one Fatigue level to activate if its level is less than or equal to 25. two Fatigue levels if its level is 26 to 50, and so on. Note that powers with a level of 26 or higher cost less Fatigue if created using the Greater Power Virtue. You may also spend levels one-for-one to give the power Penetration; otherwise, it has a Penetration of zero.

German, ArMDE:4281 (line-parallel):
> Der Charakter hat eine übernatürliche Kraft, die er nach Belieben aktivieren kann. Dies sind eine oder mehrere Kräfte, die Hermetischen Formulaischen Zaubern mit einer Gesamtstufe von 25 oder weniger entsprechen. Jede Kraft hat eine Initiative, die der Schnelligkeit des Charakters minus dem Doppelten der Kraft-Magnitude entspricht. Das Aktivieren kostet eine Erschöpfungsstufe, wenn ihre Stufe 25 oder darunter liegt, zwei Erschöpfungsstufen bei Stufe 26 bis 50 und so weiter. …

**Current data:** one effect, `{"type":"power_levels","amount":25}`;
`max_per_target: 255`; no `description` in either locale.

**What is right, checked rather than assumed.**

- `amount: 25` matches "total levels of 25 or lower", and is independently
  corroborated by `rules/source/de/translation-tables/magische-qualitaeten.md:61`
  — "Lesser Power | Mindere Macht | **25 Zauberstufen**; formulaischer Effekt".
- `max_per_target: 255` is **correct here**, and is the case that proves B04's
  `greater_immunity` finding was about the *passage* and not about the number:
  ArMDE:4283 says "This Virtue may be taken more than once, and the levels added
  together to create several powers", so two indistinguishable copies are exactly
  what the book intends. Greater Immunity's "with a **different** immunity each
  time" is the opposite instruction, which is why the same 255 was a defect
  there and is right here.
- A27 records `powers_used = Σ (power.level + power.penetration)`, so "spend
  levels one-for-one to give the power Penetration" **is** computed.

**What is missing under D5.** The **Initiative formula**
(Quickness − 2 × magnitude) and the **Fatigue cost ladder** (1 level at ≤ 25, 2
at 26-50, "and so on") are stated, are pure arithmetic, and are computed
nowhere. Neither locale's `summary` gets past "The character has a supernatural
power that he can activate at will."

This is the identical gap B04 recorded as F-96 for `virtue.greater_power`, whose
own formula differs (Quickness − magnitude/2, rounded down, at ArMDE:4023) — so
the two entries need **different** text, and copying one to the other would
introduce a wrong number.

**And the engine already computes exactly this shape for a third Power Virtue,
which makes the absence sharper than "not modelled".**
`derived/focus_power.rs::focus_power_lines` produces, per focus power, a
`FocusPowerLine` carrying `magnitude` (`magnitude_of(level) = level.div_ceil(5)`,
cited `ArMDE:9097`), `initiative` (`Quickness − magnitude`, cited `ArMDE:3899`)
and `fatigue_levels` (`fatigue_levels_for`: 1 at 0-25, 2 at 26-50, 3 at 51-75,
and deliberately `None` above 75 because "the book says nothing about higher
effects, so the honest answer is 'unstated', never a fourth band invented by
continuing the pattern", cited `ArMDE:3901`). So the machinery — the
level→magnitude rule, the Quickness-minus-magnitude Initiative, the banded
Fatigue ladder — is written, sourced and shipped.

It is wired to `Entity::focus_powers` only. `Entity::powers`
(`types.rs::SupernaturalPower`, `{name, level, penetration}`) is read by
**`effective/gift_confidence.rs::powers_used` alone**, which sums
`level + penetration` for the budget bar and derives nothing —
`grep -rn "\.powers\b" crates/arm-rules/src/derived.rs crates/arm-rules/src/derived/ crates/arm-rules/src/effective/gift_confidence.rs`
returns three hits, two of which are the *familiar's* powers
(`derived/familiar.rs`, `derived.rs`) and the third is `powers_used`. There is no
`power_lines` analogue.

The result is that of the three Power Virtues the book gives three different
formulas for, the app computes the read-out for exactly one:

| Virtue | Initiative | Fatigue | Computed? |
|---|---|---|---|
| `virtue.focus_power` (ArMDE:3899-3901) | Qik − magnitude | 1/2/3 by 25-level band | **yes**, `focus_power_lines` |
| `virtue.lesser_power` (ArMDE:4281) | Qik − 2 × magnitude | 1 at ≤25, 2 at 26-50, "and so on" | no |
| `virtue.greater_power` (ArMDE:4023) | Qik − magnitude/2, rounded down | 1 at ≤50, 2 at 51-100 | no |

Whether the right correction is text (this batch's recommendation, and the
conservative one) or a `power_lines` read-out mirroring `focus_power_lines` is a
scope call, not a rules reading — but the text is owed under D5 either way.

**Correct value:** `description` in both locales carrying the Initiative
formula, the Fatigue ladder and the Penetration purchase.
`classification` stays `creation_effect`.

**ArMDE:2960 instance.** ArMDE:4285 states "This Virtue may be associated with
any supernatural realm. … The power must be associated with the same
supernatural realm as the system on which it is based." No `realm` parameter.

**Severity:** lost-rule / provenance.

---

### F-143 — `virtue.lesser_purifying_touch` — `narrative` on a Fatigue cost and a single-illness limit

**Passage** (ArMDE:4289, verbatim, English — the entry's whole body):
> You can, with a touch and the expenditure of a Fatigue level, heal a specific illness. This illness should be one that people often recover from on their own, or one that is not particularly serious. You can only choose an illness, not an injury or other misfortune. See page 406 for rules on diseases, and Art and Academe, page 45 for more detail.

German, ArMDE:4289 (line-parallel):
> Du kannst durch eine Berührung und den Aufwand einer Erschöpfungsstufe eine bestimmte Krankheit heilen. Diese Krankheit sollte eine sein, von der sich Menschen oft von selbst erholen, oder eine, die nicht besonders schwerwiegend ist. Du kannst nur eine Krankheit wählen, keine Verletzung oder sonstige Heimsuchung. Siehe [Seite 406](#krankheiten) für Regeln zu Krankheiten und Art und Academe, Seite 45 für weitere Details.

**Current data:** `"classification": "narrative"`, no effects, no parameters.

**Why it is wrong.** A stated resource cost (one Fatigue level) plus a hard
restriction on the choice ("You can **only** choose an illness, not an injury or
other misfortune"). B04 made exactly this move on the Major member of the pair,
`virtue.greater_purifying_touch` (F-97), so leaving the Minor one `narrative`
is the same internal inconsistency F-140 describes.

**Correct value:** `"classification": "uncomputed_rule"`, with the Fatigue cost
and the illness-only restriction in `description` in both locales.

*(The "Art and Academe, page 45" pointer leads outside `rules/source/en/`, so
under CLAUDE.md's provenance rule nothing behind it can be implemented and none
is claimed here. The "page 406" pointer leads to the disease rules, which belong
to the disease system rather than to this Virtue.)*

**Severity:** lost-rule / provenance.

---

### F-144 — `virtue.lesser_purifying_touch` — no parameter records the chosen illness

**Passage** (ArMDE:4289, verbatim):
> You can only choose an illness, not an injury or other misfortune.

**Current data:** `"parameters": null`.

**Why it is wrong.** The book requires a choice ("a **specific** illness" / "eine
**bestimmte** Krankheit") and the data cannot hold it. Identical in shape to
F-139, F-141 and to B04's F-98 on the Major member.

**Correct value:** a `text`-domain parameter recording the illness.

**Severity:** data loss.

---

### F-145 — `virtue.license_of_absence` — both stated prerequisites are missing, and both are expressible

**Passage** (ArMDE:4293, verbatim, English — the relevant sentences):
> A license of absence may only be taken by a character with the Priest Social Status. It may not be taken by Senior Clergy.

German, ArMDE:4293 (line-parallel):
> Eine Beurlaubungslizenz kann nur von einem Charakter mit dem Sozialen Status Priester genommen werden. Sie kann nicht von Hochrangigen Klerikern genommen werden.

**Current data:** `"prerequisites": null`, `"incompatible_with": null`.

**Why it is wrong.** Both targets exist in the shipped catalogue —
`virtue.priest` (ArMDE:4796-4805, `narrative`, `social_status`) and
`virtue.senior_clergy` (ArMDE:4910-4921, `creation_effect`, `social_status`,
carrying two `grants_reputation` rows). The requirement is directly expressible:
`{"kind":"all","value":[{"kind":"has","value":"virtue.priest"},
{"kind":"none","value":[{"kind":"has","value":"virtue.senior_clergy"}]}]}`.
B6 records that `Prereq::Has` reads the **grants-inclusive** set, and that a
`False` result raises `prereq_not_met` as an error, so the check would bite.

The exclusion could alternatively be an `incompatible_with` pair
(`virtue.license_of_absence` ↔ `virtue.senior_clergy`), which B7 requires to be
**symmetric** — so that spelling would touch both entries. Either is defensible;
the `Nor` spelling keeps the change on one entry and matches the passage's own
phrasing ("may not be **taken**").

Note the verb again: "may only be **taken**" / "kann nur … **genommen** werden",
twice — a purchase condition, so `validate_prerequisites`' bought-only scope is
the right scope and no D2 ambiguity arises.

**Correct value:** the `all`/`has`/`none` tree above.

**Severity:** wrong rules output — the app permits a Major Virtue on a character
the book forbids it to, with no diagnostic.

---

### F-146 — `virtue.license_of_absence` — `narrative` on an extra free season and a hard cap of four

**Passage** (ArMDE:4293, verbatim, English — the relevant sentences):
> The character has an extra free season each year, but sometimes it is expected that the extra season is used for study. This Virtue is compatible with the Wealthy Virtue and Poor Flaw. A Wealthy priest with a license of absence thus has the whole year free; a character can never have more than four free seasons in a year.

German, ArMDE:4293 (line-parallel):
> Der Charakter hat jedes Jahr ein zusätzliches freies Quartal, doch es wird manchmal erwartet, dass das zusätzliche Quartal für das Studium genutzt wird. Diese Tugend ist kompatibel mit der Tugend Wohlhabend und dem Fehler Arm. Ein wohlhabender Priester mit einer Beurlaubungslizenz hat damit das gesamte Jahr frei; ein Charakter kann jedoch niemals mehr als vier freie Quartale pro Jahr haben.

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** "+1 free season per year" is a signed integer on a counted
resource, and "never more than four free seasons in a year" is a **hard cap** —
the exact clause type `ecb5150` settled must not be called `narrative`. The
passage also states an interaction with `virtue.wealthy` and `flaw.poor`, both
of which are modelled `creation_effect` entries.

**Correct value:** `"classification": "uncomputed_rule"`, with the +1 season, the
cap of four and the Wealthy/Poor interaction in `description` in both locales.
No Effect variant expresses free seasons per character (**Q-44**), so per D3
this stays text.

**Severity:** lost-rule / provenance, on a Major Virtue whose entire mechanical
content is this sentence.

---

### F-147 — `virtue.life_boost` — the +5-per-Fatigue-level rule and its damage formula reach neither locale (D5)

**Passage** (ArMDE:4297, verbatim, English — the entry's whole body):
> You may boost your Formulaic or Ritual spell casting totals by expending additional Fatigue levels. Each Fatigue level gives you an additional bonus of +5 on the roll, which can yield very impressive Penetration totals. You may burn more Fatigue levels than you possess. If you do, you must Soak damage, without the help of armor. The Damage total is 5 for every additional Fatigue level spent, plus a stress die. Thus, if you spend three additional levels, you must Soak a damage of 15 + stress die, with your Soak (no armor) + stress die. Fatigue levels spent in this way are spent regardless of the success or failure of the casting roll, and any wounds taken are similarly taken even if you fail to cast the spell. You can kill yourself doing this. The total number of Fatigue levels to be used must be committed before the casting roll is made.

German, ArMDE:4297 (line-parallel):
> Du kannst deine Zaubersummen für Formulaische oder Ritualzauber steigern, indem du zusätzliche Erschöpfungsstufen aufwendest. Jede Erschöpfungsstufe gibt dir einen zusätzlichen Bonus von +5 auf den Wurf … Der Schadenssumme beträgt 5 für jede zusätzlich aufgewendete Erschöpfungsstufe, plus ein Stresswürfel. …

**Current data:** one effect,
`{"type":"special_casting_mod","kind":"life_boost"}`; both locales' `summary`
stops at the first sentence; **no `description` in either locale**.

**Why it is a finding under D5, and not a re-report of C1.** C1 records that
`life_boost` is one of eight `SpecialCasting` kinds surfaced with `amount: 0` —
that is the *engine's* gap and is not counted here. What is counted is that the
variant "carries no amount at all, so even a surfaced row has no magnitude —
the *kind* slug is the whole payload" (A40). So the **+5 per Fatigue level**, the
**damage total of 5 per level plus a stress die**, the **no-armor Soak**, the
**commit-before-the-roll rule** and "You can kill yourself doing this" are
stated in the book and appear to the user nowhere at all — not as a number, not
as text. D5's whole point is that the obligation follows the rule.

**Correct value:** `description` in both locales carrying the +5 per level, the
damage formula, the no-armor Soak and the commitment rule.

**Severity:** lost-rule / provenance, and the clause at risk includes a
self-kill condition.

---

### F-148 — `virtue.life_linked_spontaneous_magic` — the per-five-points Fatigue rule and the wound conversion reach neither locale (D5)

**Passage** (ArMDE:4303, verbatim, English — the second paragraph in full):
> Roll to cast a Fatiguing Spontaneous spell. If your result after division is higher than the level you declared, you spend only one Fatigue level as usual. If your result after division is less than the level you declared, you must expend one additional Fatigue level per five points (or fraction thereof) by which you missed the target level. If you run out of Fatigue levels, you take a wound. The number of levels still needed for the spell is treated as the amount by which a Damage total exceeds your Soak, and you take the corresponding wound. You can kill yourself this way.

German, ArMDE:4303 (line-parallel):
> Würfle, um einen Ermüdenden Spontanzauber zu wirken. … Ist dein Ergebnis nach der Division niedriger als die von dir angegebene Stufe, musst du für je fünf Punkte (oder einen Teil davon), um die du die Zielstufe verfehlt hast, eine zusätzliche Erschöpfungsstufe aufwenden. Gehen dir die Erschöpfungsstufen aus, erleidest du eine Wunde. …

**Current data:** one effect,
`{"type":"special_casting_mod","kind":"life_linked_spontaneous"}`; both locales'
`summary` stops at the first sentence; **no `description` in either locale**.

**Why it is a finding under D5.** Same shape as F-147: the surfaced-only kind
carries no amount, so the **one extra Fatigue level per five points (rounding
any fraction up)**, the **wound conversion** ("the number of levels still needed
… is treated as the amount by which a Damage total exceeds your Soak") and the
declare-before-rolling rule reach the user in neither number nor text. ArMDE:4305
adds a fourth clause that is also absent: "A maga with this Virtue may still cast
Fatiguing Spontaneous spells normally" — i.e. the Virtue is optional per casting,
not compulsory, which is exactly the kind of clause a player needs and the app
does not say.

**Correct value:** `description` in both locales carrying the declaration step,
the per-five-points rule with its round-up, the wound conversion and the
"may still cast normally" clause.

**Severity:** lost-rule / provenance.

---

### F-149 — `virtue.lightning_reflexes` — a triply-conditional +9 is folded flat into every Initiative Total

**Passage** (ArMDE:4313, verbatim, English — the relevant sentences):
> Whenever you are surprised or startled, roll a stress die + Quickness. If you get a 3 or better, you respond reflexively. You must tell the storyguide on each occasion what one type of action (attacking, blocking, running, etc.) you would like to respond with. **If attacking in response, you gain +9 to your Initiative Total.** … You only react to threats that you are not fully aware of, so you don't get a bonus against an assassin you watch sneak up on you.

German, ArMDE:4313 (line-parallel):
> Wann immer du überrascht oder erschreckt wirst, würfle einen Stresswürfel + Schnelligkeit. Bei einem Ergebnis von 3 oder besser reagierst du reflexartig. … Wenn du als Reaktion angreifst, erhältst du +9 auf deinen Initiativewert. … Du reagierst nur auf Bedrohungen, derer du dir nicht vollständig bewusst bist …

**Current data:** `{"type":"combat_mod","amount":9,"target":"initiative"}`.

**Why it is wrong.** The book gates the +9 on **three** conditions, all stated in
one paragraph: you must be surprised *and* roll 3+ on a stress die + Quickness
*and* have chosen to respond by attacking. The passage then names the exclusion
explicitly — "you don't get a bonus against an assassin you watch sneak up on
you". `derived/combat.rs::combat_totals` adds `combat_mods[initiative]`
unconditionally, so **every** Initiative figure this character's sheet displays
is +9 too high, including the ordinary, unsurprised case which is the one a
player reads most often.

This is D4's ruling applied to a different total. D4 settled that "a Lab Total
does not change unconditionally" and resolved each condition **statically**: a
modifier whose condition is not satisfied by the situation the sheet shows is
simply absent. The static resolution here is unambiguous — a displayed Initiative
Total is not a surprise reaction — so the +9 does not belong in it. Compare
`virtue.adept_laboratory_student` and `flaw.weak_scholar`, which D4's table marks
"no" for exactly this reason.

**A35 records circumstance-blindness as this variant's known behaviour** ("Silently
ignores. Circumstance. Berserk's combat bonuses apply always."), which is why the
entry looks correct in isolation. But that note is a description of the fold, not
a licence: it is not one of Part C's seven census items, no decision records it,
and the consequence here is a wrong number on a printed sheet. `virtue.berserk`
is named in the same note and is outside this batch — see "Recurring patterns".

**Correct value:** remove the `combat_mod` row and carry the whole rule — the
surprise trigger, the stress die + Quickness 3+ gate, the declared response type,
the +9 on attack and the "not fully aware" exclusion — as `description` in both
locales. `classification` then becomes `uncomputed_rule`, per D3. If instead the
row is kept, the passage's three conditions must still be written out under D5;
either way both locales gain text they do not have today.

**Severity:** wrong rules output — a displayed combat total the book does not
grant, on every character holding this Virtue.

---

### F-150 — `virtue.linguist` — the in-play "All Advancement Totals" half reaches neither locale (D5)

**Passage** (ArMDE:4317, verbatim, English — the entry's whole body):
> You are extremely proficient learning new languages. All Advancement Totals for any Language are increased by a quarter, rounded up, as are any experience points you put into any language at character generation. Both Living and Dead languages are augmented with this Virtue.

German, ArMDE:4317 (line-parallel):
> Du bist außerordentlich begabt darin, neue Sprachen zu erlernen. Alle Fortschrittssummen für jede Sprache werden um ein Viertel erhöht (aufgerundet), ebenso wie alle Erfahrungspunkte, die du bei der Charaktererschaffung in eine Sprache investierst. Sowohl Lebende als auch Tote Sprachen werden mit dieser Tugend gefördert.

**Current data:**
```json
"effects": [
  { "type": "group_affinity_cost",
    "abilities": ["ability.dead_language", "ability.living_language"],
    "counts_as_num": 5, "counts_as_den": 4 }
]
```
No `description` in either locale.

**What is right, checked rather than assumed.** The `abilities` list is complete
and correct: `rules/core/abilities.json` contains exactly two language Abilities,
`ability.dead_language` (category `academic`) and `ability.living_language`
(category `general`), which is precisely "Both Living and Dead languages". A6
records that the match is `abilities.contains(ability)` with **no instance
test**, so it covers *every* Living Language and *every* Dead Language — which is
what "any Language" requires. The 5/4 ratio is the right direction: A4's
`charged_cost = ceil(table_xp · den / num)` charges 4/5 of the price, which is
the cost-side expression of "increased by a quarter".

**What is missing under D5.** The effect implements the **second** half of the
sentence — "any experience points you put into any language **at character
generation**" (A6: "When it fires. Character creation only"). The **first** half
— "All **Advancement Totals** for any Language are increased by a quarter" — is
an in-play advancement rule the engine does not model at all (C1 records
`advancement_mod` as surfaced-only, and this entry does not even carry one). It
reaches neither locale's `summary`, both of which stop at "You are extremely
proficient learning new languages."

**Correct value:** `description` in both locales stating that Advancement Totals
for Languages are likewise increased by a quarter, rounded up.
`classification` stays `creation_effect`.

*(The rounding direction is a separate and unsettled matter — **Q-39**.)*

**Severity:** lost-rule / provenance.

---

### F-151 — `virtue.lone_redcap` — the season obligations, two denials and the Reputation's polarity reach neither locale (D5)

**Passage** (ArMDE:4321-4325, verbatim, English):
> You are a Redcap who does not maintain ties to a Mercer House, and thus **do not receive magic items or Longevity Rituals**. You still begin with 300 experience points for your fifteen years spent as an apprentice, and receive the benefits of the Well-Traveled virtue, but you are estranged from the other Redcaps in your area, and have a **poor** Reputation at level 2 within your House. (ArMDE:4321)

> You must still devote two seasons each year carrying messages and performing other services for the Order, for if you do not there is the possibility you will be declared Orbus and thrown out of your House. This work pays enough for you to live on if you do not belong to a covenant, unless you take the Poor flaw and must work a third season as well. If you take the Wealthy virtue, you can maintain your position with only a single season of effort each year. (ArMDE:4323)

German, ArMDE:4323 (line-parallel):
> Du musst weiterhin jedes Jahr zwei Quartale damit verbringen, Nachrichten zu überbringen und andere Dienste für den Orden zu leisten … Diese Arbeit reicht aus, um davon zu leben, wenn du keinem Konvent angehörst, sofern du nicht den Fehler Arm nimmst und ein drittes Quartal arbeiten musst. Nimmst du die Tugend Wohlhabend, kannst du deine Stellung mit nur einem Quartal Einsatz pro Jahr aufrechterhalten.

**Current data:**
```json
"effects": [
  { "type": "grants_reputation", "kind": "hermetic", "score": 2 },
  { "type": "restricted_ability_xp", "amount": 300,
    "categories": ["academic","arcane","general","martial","supernatural"] },
  { "type": "grants_selection", "items": ["virtue.well_traveled"] }
]
```
No `description` in either locale.

**What is right, checked rather than assumed.** All three effects match a stated
clause: 300 XP (:4321), the Well-Traveled grant (:4321, and A18 confirms a
`grants_selection` id becomes a bare budget-exempt `Selection`), and the
Hermetic Reputation at 2 (:4321). The Reputation is independently corroborated by
`rules/source/de/translation-tables/reputationen.md:111` —
"Lone Redcap | Einzelgänger-Rotkappe | **Hermetisch (Haus)** | **2** (–) |
Entfremdet von Haus Mercere". The **absence** of an `item_level_budget` is also
correct and deliberate-looking: `virtue.redcap` carries
`{"type":"item_level_budget","amount":50}` and this entry carries none, which is
exactly "do not receive magic items".

**What is missing under D5.** Four clauses, none computed and none written:

1. **Two seasons a year** of service, rising to **three** with `flaw.poor` and
   falling to **one** with `virtue.wealthy` — both of which are modelled
   `creation_effect` entries, so this is a stated interaction between three
   catalogue entries.
2. **No magic items** — expressed only as the absence of an effect, which no
   reader can see. A player looking at the sheet learns nothing.
3. **No Longevity Rituals** — and `aging.rs::aging_total` has a
   `longevity_bonus` term that a Lone Redcap must never receive.
4. **The Reputation is "poor"**, which `types.rs::Reputation` cannot store: it
   has `kind`, `score` and a free-text `content`, and no polarity field
   (**Q-43**). The translation table marks it "(–)" precisely because the
   polarity matters.

**Correct value:** `description` in both locales carrying all four.
`classification` stays `creation_effect`.

*(The pool's `supernatural` category is a separate and unsettled matter —
**Q-41**; the missing Redcap/Lone Redcap exclusivity is **Q-42**.)*

**Severity:** lost-rule / provenance, across four clauses including two
absolute denials.

#### A fifth clause that fails for a different reason, found by following the named-Virtue cross-reference — and it points at an entry in B09, not this one

ArMDE:4321 says the Lone Redcap "receive[s] the benefits of the Well-Traveled
virtue", and the data honours that literally:
`{"type":"grants_selection","items":["virtue.well_traveled"]}`. A18 confirms the
grant becomes a real `Selection` whose own effects are then folded by every
consumer.

**But `virtue.well_traveled` has no effects to fold.** Its shipped record, in
full, is `{id, kind: virtue, magnitude: minor, categories: ["general"],
classification: "narrative", entity_kinds: ["character"], source:
[5239, 5242]}` — **no `effects` key at all** — while its passage reads,
verbatim (ArMDE:5241):

> You have journeyed extensively in this part of the world and find it easy to get along with people throughout the area. You have **fifty bonus experience points** to spend on living languages, Area Lores, and Bargain, Carouse, Charm, Etiquette, Folk Ken, or Guile.

That is a textbook `restricted_ability_xp` of 50 over a named `abilities` list —
the exact shape twenty-eight other entries in the catalogue use (`jq` over all
`restricted_ability_xp` occurrences), including `virtue.schooled_in_crime` and
`virtue.venditor`, whose passages are worded almost identically. Its EN `summary`
stops at "You have journeyed extensively …" and there is no `description`, so the
fifty points appear nowhere in the app either.

**The consequence lands on this entry.** A Lone Redcap is told by the book that
he gets Well-Traveled's benefits, and the app gives him a grant that computes
nothing and says nothing. So "receive the benefits of the Well-Traveled virtue"
is, today, a no-op.

**This is deliberately not given a finding number here.** `virtue.well_traveled`
sits at ArMDE:5239-5242, inside **B09**'s span (5097-5256), and this batch does
not rate entries outside its own range. It is recorded because it was found from
inside this batch, by following a named-Virtue cross-reference, and because a
per-entry screen run over B09 would find the `narrative`-on-a-number half but
**not** the part that matters here — that a grant in another batch depends on it.
**B09 must check `virtue.well_traveled` against ArMDE:5241 and is hereby told
what this pass found.**

---

### F-152 — `virtue.lone_redcap` — the German name contradicts the canonical tables, in two of them

**Current data:** de `name` = `Einsame Rotkappe`.

**The tables, both of them:**

- `rules/source/de/translation-tables/reputationen.md:111` —
  `| Lone Redcap | Einzelgänger-Rotkappe | Hermetisch (Haus) | 2 (–) | Entfremdet von Haus Mercere |`
- `rules/source/de/translation-tables/tugenden-fehler.md:650` —
  `| Lone Redcap | Einzelgänger-Rotkappe | HoH:TL; Rotkappe ohne Hausanbindung |`

**Why the `reputationen.md` row settles it.** The `tugenden-fehler.md` row sits
under `### Ergänzungen aus Houses of Hermes: True Lineages (HoH:TL)`, so on its
own it could be argued to describe a supplement entry rather than this core one
— that is the argument that makes Q-40 a question rather than a finding. The
`reputationen.md` row is **not** in a supplement section, and its content columns
transcribe *this* passage exactly: Hermetic (House), level 2, negative,
"estranged from House Mercere" ↔ ArMDE:4321 "estranged from the other Redcaps in
your area, and have a poor Reputation at level 2 within your House". So the
canonical mapping for the term this entry uses is on record, unambiguously, and
the shipped label is not it.

The DE rulebook heading at ArMDE:4319 reads `#### Einsame Rotkappe`, agreeing
with the shipped i18n — the same rulebook-vs-table conflict as F-135, resolved
the same way by CLAUDE.md and by B04's precedent.

**Correct value:** `Einzelgänger-Rotkappe`.

**Severity:** localization defect.

---

### F-153 — `virtue.long_winded` — "This bonus does not apply to casting spells" reaches neither locale (D5)

**Passage** (ArMDE:4329, verbatim, English — the entry's whole body):
> You can last longer when exerting yourself than most, and gain +3 on all your Fatigue rolls. This bonus does not apply to casting spells.

German, ArMDE:4329 (line-parallel):
> Du kannst dich länger anstrengen als die meisten und erhältst +3 auf alle deine Erschöpfungswürfe. Dieser Bonus gilt nicht für das Wirken von Zaubern.

**Current data:** `{"type":"health_mod","track":"fatigue_roll","amount":3}`;
both locales' `summary` carries the +3 but **stops before the exception**
("… und erhältst +3 auf alle deine Erschöpfungswürfe." — the second sentence is
simply absent).

**Why it is a finding under D5, and not a re-report of C1.** C1 records that the
`fatigue_roll` track is surfaced-only; that is not counted here, and the +3 is
correctly authored against an engine that lists rather than computes it. What is
counted is the **exception**. The engine has a separate `casting_fatigue` track
(A36) — so the taxonomy distinguishes exactly the two things this sentence
distinguishes — yet the exclusion is expressible in neither direction: there is
no negative scope on a `health_mod`, and adding a `casting_fatigue` row of −3 to
cancel it would be inventing a rule the book does not state. Per D3 the honest
answer is text, and per D5 the text is owed regardless of the entry staying
`in_play_effect`.

**Correct value:** `description` in both locales, or at minimum a `summary`
extended with the second sentence.

**Severity:** lost-rule / provenance. Note the user-facing shape: the read-out
shows "+3 Fatigue rolls" with nothing saying it is off during spellcasting,
which is the precise circumstance a magus cares about.

---

### F-154 — `virtue.luck` — `narrative` on "+1 to +3"

**Passage** (ArMDE:4333, verbatim, English — the entry's whole body):
> You perform well in situations where luck is more of a factor than skill or talent. You get +1 to +3 (storyguide's discretion) on rolls in such situations, depending upon how much luck is involved. You do well at games of chance, but may be labeled a cheater if you play them too often.

German, ArMDE:4333 (line-parallel):
> Du erzielst gute Ergebnisse in Situationen, in denen Glück mehr eine Rolle spielt als Können oder Talent. Du erhältst +1 bis +3 (nach Ermessen des Spielleiters) auf Würfe in solchen Situationen, je nachdem, wie viel Glück im Spiel ist. Du spielst gut bei Glücksspielen, könntest aber als Betrüger abgestempelt werden, wenn du sie zu oft spielst.

**Current data:** `"classification": "narrative"`, no effects; both locales'
`summary` stops before the number.

**Why it is wrong.** "+1 to +3" is a signed numeric range with a stated upper
bound. Storyguide discretion is what makes it **uncomputable**, not what makes it
non-mechanical — B1 spells the distinction out: `uncomputed_rule` covers "real
rules that are genuinely uncomputable … GM judgement, open-ended magnitudes",
and `narrative` is reserved for an entry that states **no mechanical clause at
all**. Luck states one and bounds it.

**Correct value:** `"classification": "uncomputed_rule"`, with the +1-to-+3 range
and the discretion clause in `description` in both locales.

**Severity:** lost-rule / provenance. This is the cleanest single example in the
batch of the `uncomputed_rule`/`narrative` line being drawn in the wrong place.

---

### F-155 — `virtue.lupus_the_wolf` — permission for two gated Academic Abilities with no `ability_authorization`

**Passage** (ArMDE:4337, verbatim, English — the relevant sentence):
> He may begin play with scores in Latin or Artes Liberales, although a score of more than 1 in Artes Liberales would be rare.

German, ArMDE:4337 (line-parallel):
> Er kann zu Beginn Punkte in Latein oder Artes Liberales haben, obwohl ein Wert von mehr als 1 in Artes Liberales selten wäre.

**Current data:** `"classification": "narrative"`, no effects.

**Why it is wrong.** Both named Abilities are `academic`, which is gated, so a
Lupus with Latin 2 is refused by `validate_ability_authorization`. The sentence
is a character-generation permission — "may **begin play** with scores in" — and
nothing in the data carries it. Fourth instance of the open pattern in this
batch alone.

The trailing clause ("a score of more than 1 in Artes Liberales would be rare")
is a soft guideline, not a cap — "would be rare" / "wäre selten", not "may not
exceed" — so it is *not* claimed as a second rule here, unlike B04's
`virtue.gorgiastic`, whose "cannot be raised above 4" was an absolute.

**Correct value:** `"classification": "creation_effect"`, plus
`{"type": "ability_authorization", "abilities": ["ability.artes_liberales",
"ability.dead_language"]}`.

**Severity:** wrong rules output — the app blocks a legal character.

---

### F-156 — `virtue.magian_lineage_minor` — the "+3 bonus to resist the effects of disease" reaches neither locale (D5)

**Passage** (ArMDE:4343, verbatim, English — the Minor half in full):
> *Minor:* Your character's lineage is weak, although he or she is still ethnically Magian, and gains a strong constitution due to his purity. The character gains a -1 bonus to Aging rolls and a +3 bonus to resist the effects of disease.

German, ArMDE:4343 (line-parallel):
> *Klein:* Die Abstammungslinie deines Charakters ist schwach, obwohl er oder sie ethnisch gesehen ein Magier ist und aufgrund seiner Reinheit eine robuste Konstitution erlangt. Der Charakter erhält einen -1-Bonus auf Alterungswürfe und einen +3-Bonus, um den Auswirkungen von Krankheiten zu widerstehen.

**Current data:** one effect,
`{"type":"aging_mod","kind":"aging_roll","amount":-1}`; both locales' `summary`
is the *shared* first sentence of the joint heading ("This Virtue makes the
character a true blood descendent of the Median tribe of Magians …") and
mentions neither number.

**What is right, checked rather than assumed.** The sign is correct. A38 records
that an `aging_roll` modifier is "**added** to the AGING TOTAL with its stored
sign", and the book calls −1 a *bonus* because a lower aging total is better —
so `amount: -1` lowers the total, which is what the passage grants. The same
mapping is confirmed for the sibling entry `virtue.magical_blood` by
`rules/source/de/translation-tables/tugenden-fehler.md:135`
("SdM:M; −1 auf Alterungswürfe; …").

**What is missing under D5.** The **+3 to resist the effects of disease** is the
other half of one sentence, is computed by nothing, and appears in neither
locale. The engine has no disease-resistance total at all, so per D3 it stays
text — and per D5 the text is owed even though the entry is `in_play_effect`.

**Correct value:** `description` in both locales carrying both numbers, so the
pair of bonuses the book grants in one sentence reaches the player together.

**Severity:** lost-rule / provenance.

---

### F-157 — `virtue.magian_lineage_major` — the three chosen Abilities have no parameter

**Passage** (ArMDE:4345, verbatim, English — the relevant sentences):
> The player **must choose three** Arcane or Supernatural Abilities which the character need not start play with, and which cannot be True Names. These Abilities are considered connected, so that whenever your character gains experience from a source dedicated to one of these Abilities, he gains half the Source Quality (round up) in experience points in each of the other two Abilities.

German, ArMDE:4345 (line-parallel):
> Der Spieler muss drei Arkane oder Übernatürliche Fertigkeiten wählen, mit denen der Charakter nicht notwendigerweise zu Beginn des Spiels vertraut sein muss und die keine Wahren Namen sein dürfen. …

**Current data:** `"parameters": null`; one effect,
`{"type":"aging_mod","kind":"aging_roll","amount":-1}` (inherited from the Minor
half, correctly — ":4345 In addition to the benefits of the Minor Virtue").

**Why it is wrong.** "The player **must** choose three" is the strongest form of
the choice requirement in this batch — stronger than Lesser Benediction's or
Lesser Immunity's — and the data records none of the three. Two characters with
this Major Virtue are indistinguishable in the save. The right shape exists and
the catalogue uses it: three `ability`-domain parameters (or one with a
`max_per_value`), exactly as `virtue.learn_ability_from_mistakes` uses one
three lines up in this same batch. The book also states two restrictions on the
choice — "Arcane or Supernatural" and "cannot be True Names" — which a
parameter's `require_categories` cannot express for Abilities (B8 scopes that
field to the `item` domain), so those restrictions ride along as text.

**Correct value:** three `ability`-domain parameters, plus the two restrictions
in `description`.

**Severity:** data loss — a mandatory player choice the save cannot hold, on a
Major Virtue whose whole distinctive effect depends on it.

---

### F-158 — `virtue.magian_lineage_major` — the XP-sharing rule and the disease bonus reach neither locale (D5)

**Passage** (ArMDE:4345, verbatim, English — the mechanical clauses):
> These Abilities are considered connected, so that whenever your character gains experience from a source dedicated to one of these Abilities, he gains half the Source Quality (round up) in experience points in each of the other two Abilities. A character must have access to a Supernatural Ability in one of the normal ways, such as taking the Virtue at character creation or Initiating it later, in order to put any experience points into it. If you choose a Supernatural Ability that your character does not yet have access to, any experience points generated for it by this Virtue before the character gains access to the Ability are lost.

German, ArMDE:4345 (line-parallel):
> Diese Fertigkeiten gelten als miteinander verbunden, sodass dein Charakter, wann immer er Erfahrung aus einer Quelle erhält, die einer dieser Fertigkeiten gewidmet ist, die Hälfte der Quellenqualität (aufgerundet) als Erfahrungspunkte in jede der beiden anderen Fertigkeiten erhält. Ein Charakter muss auf eine der üblichen Weisen Zugang zu einer Übernatürlichen Fertigkeit haben … Wählst du eine Übernatürliche Fertigkeit, zu der dein Charakter noch keinen Zugang hat, gehen alle Erfahrungspunkte, die durch diese Tugend für sie generiert werden, bevor der Charakter Zugang zu der Fertigkeit erhält, verloren.

**Current data:** one effect (the inherited aging −1); no `description` in
either locale; the `summary` is shared verbatim with the Minor entry and says
nothing about any of this.

**Why it is a finding under D5.** Three uncomputed clauses plus the Minor half's
+3 disease bonus (F-156 applies here too, since the Major explicitly includes
"the benefits of the Minor Virtue"). The XP-sharing rule states a **formula with
a rounding direction** — "half the Source Quality (round up)" — which is exactly
the class of clause `ecb5150` settled must not be dropped.

**A deliberate near-miss worth recording, so it is not counted twice later.**
This entry names Arcane and Supernatural Abilities but grants **no**
authorization to own them: ":4345 A character must have access to a Supernatural
Ability in one of the normal ways … in order to put any experience points into
it." So it is *not* an instance of the `ability_authorization` open pattern —
the passage goes out of its way to say the opposite — and no authorization row is
owed. This is the same distinction B04 drew for `virtue.gorgiastic`.

**Correct value:** `description` in both locales carrying the connected-Abilities
XP rule with its round-up, the access requirement, the forfeiture rule and the
inherited +3 disease bonus.

**Severity:** lost-rule / provenance, on the Major Virtue's entire distinctive
mechanic.

---

### F-159 — `virtue.magic_items` — the improvement rate and the Level 30 per-effect cap reach neither locale (D5)

**Passage** (ArMDE:4349, verbatim, English — the entry's whole body):
> You begin with 25 more starting levels of magic items than you would otherwise, and the rate at which your items are improved is increased by one level per year. This is probably because of your exceptional devotion to the House, or because you have inherited a number of items from other Redcaps. You must be a Redcap to take this Virtue, and you may take it more than once, though no single effect in any of your items can be greater than Level 30.

German, ArMDE:4349 (line-parallel):
> Du beginnst mit 25 zusätzlichen Startstufen an Zauberartefakten im Vergleich zu dem, was du sonst hättest, und die Rate, mit der deine Artefakte verbessert werden, erhöht sich um eine Stufe pro Jahr. … Du musst eine Rotkappe sein, um diese Tugend zu nehmen, und du kannst sie mehr als einmal nehmen, obwohl kein einzelner Effekt in einem deiner Artefakte größer als Stufe 30 sein darf.

**Current data:** `{"type":"item_level_budget","amount":25}`,
`"prerequisites": {"kind":"has","value":"virtue.redcap"}`,
`"max_per_target": 255`; no `description` in either locale, though **both
locales' `summary` does carry the 25 and the improvement rate** (en: "You begin
with 25 more starting levels of magic items than you would otherwise, and the
rate at which your items are improved is increased by one level per year.").

**What is right, checked rather than assumed.** Three clauses are correctly
encoded: the 25 levels (A19: summed into `item_level_budget`, spent side
`Σ device.level`, over-spend an error); "You must be a Redcap"
(`Prereq::Has("virtue.redcap")`, and B6 confirms `has` reads the
grants-inclusive set, so a type- or House-granted Redcap would satisfy it); and
"you may take it more than once" (`max_per_target: 255`, correct because
identical copies genuinely stack here — the budgets add — exactly as for
`virtue.lesser_power` and unlike B04's `greater_immunity`).

**What is missing under D5.** The **Level 30 per-effect cap** is a hard ceiling,
is stated in the same sentence as the multiplicity rule the data *does* encode,
and is in neither locale — the `summary` stops before it. It is also structurally
inexpressible today: `types.rs::EnchantedDevice` is `{name, level}` where `level`
is documented as the device's **total** effect level, with no per-effect
breakdown, so nothing could compare a single effect against 30. That is D3's
case, and D5 makes the text mandatory.

*(The "one level per year" improvement rate is in the `summary`, so it is
carried — just not in a `description`. It is uncomputed, and this batch does not
treat a `summary` that states the rule as a defect: B1 records that
`uncomputed_clauses.rs` itself accepts `description`, **else** `summary`.)*

**Severity:** lost-rule / provenance, on a hard cap.

---

### F-160 — `virtue.knows_people` — `narrative` on a once-per-story player entitlement

*(Raised by the verification pass, which overturned this batch's first-pass `?`.
Every citation below was re-opened in the source before the overturn was
accepted.)*

**Passage** (ArMDE:4203, verbatim, English — the second paragraph in full):
> Once per story or session, a character with this Virtue may ask for a bait for a non-player character. A bait is the beginning of a scene or short, secondary story, outside the main story being told, which if completed allows the character to gain aid from the nominated target. For example, if the player characters are unable to gain the assistance of a nobleman, a player may demand a bait. In the game, this means the character uses his social skills to determine the needs of the nobleman, and to hint that he may know a third person with a solution. If the troupe then plays out a brief scene in which the nobleman's problem is sorted out, he becomes more biddable.

and ArMDE:4205:
> Troupes may veto any use of these connections which spoils the tension and pace of the game.

German, ArMDE:4203 (line-parallel):
> Einmal pro Geschichte oder Spielsitzung darf ein Charakter mit dieser Tugend einen Köder für einen Nicht-Spieler-Charakter verlangen. … Sind die Spielercharaktere beispielsweise nicht in der Lage, die Unterstützung eines Adeligen zu gewinnen, kann ein Spieler einen Köder verlangen. …

German, ArMDE:4205:
> Die Spieltruppe kann jeden Einsatz dieser Verbindungen ablehnen, der die Spannung und das Tempo des Spiels beeinträchtigt.

**Current data:** `"classification": "narrative"`, no effects, no `description`
in either locale; both `summary` fields carry **only** the colour paragraph
(:4201, "Your social contacts are not important for what they give you
directly …").

**Why the first pass got this wrong.** It read ArMDE:4205's veto as unconditional
and concluded that an entitlement the troupe can revoke at will is not a rule.
That reading does not survive the sentence: the veto is **qualified** — "any use
of these connections **which spoils the tension and pace of the game**" /
"der die Spannung und das Tempo des Spiels beeinträchtigt". The entitlement
stands by default and the veto is an exception to it.

**What settles it, and it is not a judgement call.** The catalogue has already
decided this exact shape, inside a block the guard has swept:

| Entry | Passage | ArMDE | Classification | In `SWEPT_BLOCKS`? |
|---|---|---|---|---|
| `virtue.all_according_to_plan` | "**Once per session**, a player whose character has this Virtue can reroll a botch die." | 3386 | **`uncomputed_rule`**, 0 effects | **yes** (3360-3950) |
| `virtue.knows_people` | "**Once per story or session**, a character with this Virtue may ask for a bait …" | 4203 | `narrative`, 0 effects | no |
| `virtue.supernatural_beauty` | "A player may use this Virtue, **once per story**, to ask a storyguide to insert a fortunate coincidence …" | 5091 | `narrative`, 0 effects | no |

Those three are the **entire** family: `grep -n "[Oo]nce per st\|[Oo]nce per session"`
over the English core rulebook returns exactly `:3386`, `:4203` and `:5091` and
nothing else. The one member inside a swept block is already `uncomputed_rule`;
the two outside it are still `narrative`. That is the README's thesis about
`SWEPT_BLOCKS` demonstrated on a three-member set — and note *why* the screen
caught the swept one and would have missed these two even inside the block:
`all_according_to_plan` says "botch die", a token the screen knows, while "bait"
and "fortunate coincidence" are tokens nothing knows. So this is not a hole the
screen could close; only reading it closes it.

**Three further reasons from the text itself**, independent of the precedent:

1. **"Once per story or session" is a frequency cap** with a stated period. `ecb5150`'s standard, which this audit applies verbatim: a cap is as mechanical as a +3.
2. **The entitlement is addressed to the player, not to the fiction.** ":4203 a player may **demand** a bait" / "kann ein Spieler einen Köder **verlangen**" — a rule about what a player may compel at the table, not a description of the character's world.
3. **It states a conditional state change:** "which if completed allows the character to gain aid from the nominated target" — a defined conversion of an NPC's refusal into aid, with a precondition.

**And discretion has never made a rule into colour in this book.** ArMDE:2818
calls the one-Story-Flaw limit "a guideline, and may be violated with the whole
troupe's agreement", and the catalogue encodes it anyway. B1's definition is the
governing one: `uncomputed_rule` explicitly covers "GM judgement, open-ended
magnitudes"; `narrative` requires the passage to state **no mechanical clause at
all**, and :4203 states three.

**What it costs today.** Because the entry is `narrative` it owes no
`description` and has none, so the once-per-story bait entitlement — the only
thing this Minor Virtue actually gives a player — reaches the application
**nowhere, in either locale**. What ships is the colour paragraph.

**Correct value:** `"classification": "uncomputed_rule"`, with :4203's
entitlement and :4205's qualified veto in `description` in both locales (the
German is available verbatim and its terminology is already canonical —
`Tugend`, `Spieltruppe`, `Nicht-Spieler-Charakter`). That also switches
`uncomputed_clauses.rs` on for the entry.

**Severity:** lost-rule / provenance — a paid-for rule that reaches no user. Per
B1 it cannot produce a wrong number, since `classification` is read by no
production code.

---

## Open questions

Per the audit's rules these are **escalated, not resolved**. Each names what
could not be settled from the source, and what was tried.

### Q-35 — **ANSWERED AND CLOSED, see F-160** — `virtue.knows_people`: is "once per story or session … may ask for a bait" a mechanical clause?

**Answer: yes.** The first pass raised this as a question; the verification pass
settled it from the source and the overturn was accepted after re-checking every
citation. **F-160** carries the verdict and the evidence. The number is kept
rather than reused so `corrections.md` can accumulate without collision, and the
original reasoning is left below because it records what the first pass actually
thought — and because the specific thing it got wrong (reading ArMDE:4205's veto
as unconditional) is worth keeping visible.

*The question as it was originally posed:*

**Passage** (ArMDE:4203, verbatim, English):
> Once per story or session, a character with this Virtue may ask for a bait for a non-player character. A bait is the beginning of a scene or short, secondary story, outside the main story being told, which if completed allows the character to gain aid from the nominated target.

and ArMDE:4205:
> Troupes may veto any use of these connections which spoils the tension and pace of the game.

**Current data:** `"classification": "narrative"`, no effects.

**Why it is a question and not a finding.** It has the *shape* of a rule — a
stated frequency limit on a stated entitlement — and this audit's standard
(`ecb5150`) counts a cap as mechanical. But there is no die, no number, no
target, and the very next paragraph hands the troupe an unconditional veto, which
is unusual even among GM-discretion clauses: the entitlement is not merely
adjudicated, it is revocable. `virtue.luck` (F-154) is discretionary *within a
stated numeric range* and I rated that a finding; this one has no range at all,
so the two are not the same case and I did not want to decide by analogy.

**What I could not settle it with.** The book gives no general statement about
whether a per-session narrative entitlement counts as a rule. `README.md`'s
table says `narrative` means "states **nothing mechanical**", and a frequency
limit is arguably something; but `uncomputed_rule`'s own definition names "GM
judgement" as its content, which fits too. Either reading is defensible from the
text and I decline to pick one. The entry is marked `?` and is **not** counted
as checked-clean.

---

### Q-36 — `virtue.lasiq`: does "some other social status, which you should choose" demand a parameter?

**Passage** (ArMDE:4235, verbatim, English):
> As for a fida'i, a lasig may be far from home on a mission, either alone or with some fida'i. In such as case, he is pretending to have some other social status, which you should choose.

German, ArMDE:4235 (line-parallel):
> Wie ein Fida'i kann sich ein Lasiq weit weg von seiner Heimat auf einem Auftrag befinden, entweder allein oder mit einigen Fida'is. In einem solchen Fall gibt er vor, einen anderen sozialen Status zu haben, den du festlegen solltest.

**Current data:** `"parameters": null`.

**Why it is a question.** The verb is "**should** choose" / "festlegen
**solltest**", and it is conditioned on a situation ("In such a case") rather
than on taking the Virtue. Contrast the three parameter findings above, whose
verbs are unconditional: "You can only **choose** an illness" (F-144), "The
player **must choose** three" (F-157). A conditional "should" may be advice
rather than a required choice.

**What it interacts with.** If it *is* a required choice, it also touches
ArMDE:2816 — a second Social Status the character pretends to have is not a
second Social Status he possesses, but the data has no way to say either.

**What I could not settle it with.** Nothing in ArMDE:2816 or in the Social
Status block addresses a *pretended* status.

---

### Q-37 — `virtue.leper_magus`: should the Life Boost grant be a `grants_selection` rather than a copied effect?

**Passage** (ArMDE:4251, verbatim):
> It allows him to draw upon the strength of his body to increase the power of his magic, **granting the Life Boost Minor Virtue**.

German, ArMDE:4251:
> Sie ermöglicht es ihm, die Stärke seines Körpers zu nutzen, um die Kraft seiner Magie zu steigern, indem er die Kleine Tugend Lebensstärkung erhält.

**Current data:** `virtue.leper_magus` carries
`{"type":"special_casting_mod","kind":"life_boost"}` — the *same effect row*
`virtue.life_boost` carries — rather than
`{"type":"grants_selection","items":["virtue.life_boost"]}`.

**Why it is a question.** The book's verb is "granting … the Virtue", which is
literally what A18's `grants_selection` models, and the catalogue uses that
variant seven times (including on `virtue.lone_redcap` in this same batch, for
"receive the benefits of the Well-Traveled virtue"). Copying the effect instead
is observationally equivalent **today**, because `life_boost` is surfaced-only
(C1) and the surfaced row carries no amount. It stops being equivalent the
moment `life_boost` computes anything, and it is already visibly different in
one place: a magus with both Leper Magus and a bought Life Boost gets **two**
identical surfaced rows, where a `grants_selection` would be deduplicated by
`max_per_target` (B10 counts grants).

**What I could not settle it with.** Whether the catalogue has a convention for
"grants Virtue X" where X's own effect is a single surfaced marker. Both shapes
ship. A ruling would also decide whether `virtue.life_boost`'s description
(F-147) should be *reachable* from Leper Magus, which a grant would give for
free and a copied effect would not.

---

### Q-38 — `virtue.lesser_immunity`: does Greater Immunity's "more than once, with a different immunity each time" transfer through the cross-reference?

**Passage** (ArMDE:4277, the entry's whole body):
> You are immune to some hazard which is either rare, or not deadly, or both. See Greater Immunity, page 83.

**What the pointer lands on** (ArMDE:4015):
> You may take this Virtue more than once, with a different immunity each time.

**Current data:** `max_per_target` absent (= **1**), `max_total` absent (= 255).
So `virtue.lesser_immunity` may be taken exactly **once** today. Its Major
counterpart carries `max_per_target: 255`, which B04 found defective for the
opposite reason (F-94: 255 with no parameter permits 255 *identical* copies
where the book says "a different one each time").

**Why it is a question.** "See Greater Immunity" is a bare pointer with no scope
marker — it does not say "see Greater Immunity for the rules on repetition" or
"except as noted". If the whole of :4009-4015 transfers, Lesser Immunity is
repeatable and `max_per_target: 1` is wrong. If only the *description of what an
immunity is* transfers, the current 1 is right. The two entries therefore cannot
both be correct as authored, but which one moves depends on the reading.

**What I could not settle it with.** The book states no general rule about how
far a "See X" cross-reference reaches. Note that whichever way it goes, F-141's
missing parameter must be fixed first: without a parameter, "a different one each
time" is unrepresentable either way.

---

### Q-39 — `virtue.linguist`: the book rounds the XP **up**; the engine rounds the **cost** up. Do they agree?

**Passage** (ArMDE:4317, verbatim):
> All Advancement Totals for any Language are increased by a quarter, **rounded up**, as are any experience points you put into any language at character generation.

**Current data:** `group_affinity_cost` 5/4, which A4/A6 apply as
`charged_cost(table_xp) = ceil(table_xp · den / num)` = `ceil(table_xp · 4 / 5)`.

**Why it is a question.** The two roundings favour different sides and diverge at
non-multiples of 5:

| Spend | Book: `ceil(S · 5/4)` effective XP | Engine: largest `T` with `ceil(T·4/5) ≤ S` |
|---|---|---|
| 10 | **13** | **12** |
| 12 | 15 | 15 |
| 16 | 20 | 20 |
| 18 | **23** | **22** |

So at S = 10 the book gives a Linguist 13 experience points and the app gives 12.
The drift is one point, always against the player, and only at non-multiples.

**Why it is not simply a finding.** The cost-side `ceil` is the engine's *general*
Affinity arithmetic, and the book's own worked example endorses it there — A4
cites `ArMDE:2443`: table 55 with a 3/2 Affinity → `ceil(55·2/3) = 37`. Affinity
is worded as a cost reduction, so `ceil` on the cost is right for it. Linguist is
worded the other way round — as an **XP gain** — and the identical machinery
cannot honour both roundings at once. Whether the book intends them to be the
same mechanism with one rounding, or two mechanisms, is a rules reading.

**What I could not settle it with.** Nothing in ArMDE:2443's neighbourhood or in
:4317 reconciles the two wordings. The same question would apply to any future
XP-gain-worded Virtue routed through `group_affinity_cost`.

---

### Q-40 — `virtue.linguist`: the canonical table's only `Linguist` row is in a supplement section and gives "Linguist", not "Sprachbegabt"

**The table:** `rules/source/de/translation-tables/tugenden-fehler.md:617` —
`| Linguist | Linguist | HoH:TL; Erleichtertes Sprachenlernen |`, under
`### Ergänzungen aus Houses of Hermes: True Lineages (HoH:TL)` →
`#### Allgemeine Tugenden, Klein / General, Minor`.

**Current data:** de `name` = `Sprachbegabt`, matching the DE rulebook heading at
ArMDE:4315 (`#### Sprachbegabt`).

**Why it is a question and not a finding.** CLAUDE.md's rule is unconditional —
"the German label for any term whose English form appears in a table MUST match
the table's `Deutsch (DE)` value" — and "Linguist" appears in a table. But this
is the **only** `Linguist` row in all sixteen tables (searched with
`grep -rn "Linguist" rules/source/de/translation-tables/`, one hit), it sits in a
section explicitly headed as HoH:TL additions, and its Anmerkung tags it `HoH:TL`.
Our entry is core (ArMDE:4315). So either the table is describing a *different*
Virtue that happens to share an English name, or the core Virtue's row is
missing and :617 is the closest thing to it.

**How this differs from F-130, F-135 and F-152, which I did rate.** Those three
rows sit in **core** sections of the tables (`### Übernatürliche Tugenden, Klein`
at `:127`/`:128`) or, for Lone Redcap, in a second non-supplement table
(`reputationen.md:111`) whose content columns transcribe the core passage. B04's
F-107/F-108 precedent is for a core-section row too. No such corroboration exists
for Linguist.

**What I could not settle it with.** Whether the tables' supplement sections are
binding on core entries of the same English name. A ruling here also settles
Lone Redcap's second row (`tugenden-fehler.md:650`), though F-152 does not depend
on it.

---

### Q-41 — `virtue.lone_redcap`: is `supernatural` in the 300-point pool sourced?

**Current data:**
```json
{ "type": "restricted_ability_xp", "amount": 300,
  "categories": ["academic","arcane","general","martial","supernatural"] }
```

**What the source says.** `virtue.lone_redcap`'s own passage (ArMDE:4319-4326)
names **no** Ability categories at all — only "You still begin with 300
experience points for your fifteen years spent as an apprentice" (:4321). The
nearest statement is in `virtue.redcap`'s passage, which the Lone Redcap
inherits by "You **still** begin with": ArMDE:4848 —
> You are trained in a similar manner to magi, and may take Academic. Arcane, and Martial Abilities during character generation.

That is **three** categories. `general` is uncontroversial (it is ungated, and
apprenticeship XP must be spendable on ordinary Abilities). `supernatural` is in
neither passage.

**Why it matters.** A7 records that a restricted pool's category list is both a
funding eligibility *and* a permission. `supernatural` is not in
`categories_requiring_virtue`, so the permission half is inert — but the funding
half is not: 300 apprenticeship points become spendable on Supernatural
Abilities, which ArMDE:2870/:2874 tie to holding a specific Virtue per Ability.

**Why it is a question and not a finding.** A Lone Redcap who legitimately holds
a Supernatural Virtue arguably *should* be able to spend apprenticeship XP on the
Ability it grants, so the extra category is defensible even though it is not in
the text. And the shape is not a one-off: exactly one other entry in the whole
catalogue uses the same five-category list — `virtue.mentored_by_demons` — while
`virtue.privileged_upbringing` uses `["general","academic","martial"]`, so there
is no house convention either way (`jq` over
`rules/core/virtues_flaws.json`, all 28 `restricted_ability_xp` occurrences
listed and read).

**What I could not settle it with.** The passage is silent, so "source or
nothing" argues for removing `supernatural`, while the rules-as-played argument
argues for keeping it. A ruling covers both entries.

---

### Q-42 — `virtue.lone_redcap` and `virtue.redcap`: nothing encodes that they are alternatives

**Passages, verbatim:**
> You are a Redcap who does not maintain ties to a Mercer House … This social status is compatible with any other **mundane** Social Status Virtue that would reasonably allow you to do your job as a Redcap, such as Merchant or Mendicant Friar. (ArMDE:4319, :4325)

> Although you do not have The Gift and cannot work Hermetic magic, you are a full member of the Order of Hermes and of House Mercere … (ArMDE:4844)

**Current data:** neither entry lists the other in `incompatible_with`; neither
carries a `prerequisites` referring to the other.

**Why it is a question.** ArMDE:2816's general rule — "All characters must take
one Social Status, and may only take more than one if the descriptions …
explicitly note that they are compatible" — would make the two mutually exclusive
by default, and Lone Redcap's own compatibility note is scoped to **mundane**
statuses, which Redcap is not. But that inference runs through ArMDE:2816, which
this repo encodes nowhere at all (the catalogue-wide pattern below), so singling
out this one pair would be inventing a specific rule from a general one nobody
has decided how to model.

**What I could not settle it with.** No sentence in either passage says "you may
not also take Redcap". The reading is an inference, and the audit's rule is that
an inference is escalated.

---

### Q-43 — a granted Reputation has no polarity, so "a **poor** Reputation at level 2" is unrepresentable

**Passage** (ArMDE:4321, verbatim):
> … but you are estranged from the other Redcaps in your area, and have a **poor** Reputation at level 2 within your House.

**Current data:** `{"type":"grants_reputation","kind":"hermetic","score":2}`, and
`types.rs::Reputation` is `{ kind: ReputationType, score: u8, content: String }`
— no sign, no polarity, no good/ill flag. A25 confirms the effect carries only
`kind` and `score`.

**Why it is recorded.** This is not `virtue.lone_redcap`'s fault and is not
counted as its finding (F-151 folds the wording into the description instead).
It is catalogue-wide: `rules/source/de/translation-tables/reputationen.md` has a
whole `### Fehler / Flaws` block of Reputations marked "(–)" — Infamous,
Apostate, Excommunicated, Black Sheep, Outlaw, Usurer, Lone Redcap and a dozen
more — every one of which lands in the same undifferentiated
`{kind, score, content}` today. The only place the difference survives is the
free-text `content` field, which nothing validates and nothing reads.

**Why it is a question rather than a Part C addition.** Phase 0's open question 3
already asks whether a granted Reputation's `score` should be enforced; this is
the adjacent question about its **sign**, and both bear on the same struct. It
did not seem right to answer one while Phase 0 has the other open. Note that if
OQ3 is answered "enforce the score", the sign question becomes urgent rather than
cosmetic: enforcing a level while ignoring whether it helps or harms is worse
than enforcing neither.

---

### Q-44 — no Effect variant expresses free seasons per year, though three entries in this batch modify them

**The three passages:**
> You must spend every season managing it, or it may collapse completely … (`virtue.landed_noble`, ArMDE:4223)

> The character has an extra free season each year … a character can never have more than four free seasons in a year. (`virtue.license_of_absence`, ArMDE:4293)

> You must still devote two seasons each year … unless you take the Poor flaw and must work a third season as well. If you take the Wealthy virtue, you can maintain your position with only a single season of effort each year. (`virtue.lone_redcap`, ArMDE:4323)

**Current state.** None of the 42 `Effect` variants (Part A, read in full)
touches a per-year season count. The engine *does* count seasons, but only in the
ruleset-level post-apprenticeship configuration —
`max_charged_lab_seasons_per_year` and `lab_season_cost`, which are ruleset
constants validated in `ruleset/integrity.rs`, not per-character values a V/F
could move.

**Why it is a question.** F-131, F-146 and F-151 each resolve to "write it as
text", which is correct under D3 and is what this batch recommends. But three
entries in a 35-entry window modifying the same unmodelled resource — with a
stated hard cap of four, and with explicit interactions with `virtue.wealthy` and
`flaw.poor`, both of which *are* modelled — suggests this may be a missing axis
rather than three unrelated omissions. Whether the existing
post-apprenticeship season config is the right home, or whether free seasons are
simply out of scope for a character generator, is a design call.

**What I could not settle it with.** It is not a rules question — the book is
perfectly clear — so "source or nothing" gives no answer. It is a scope decision.

---

## Recurring patterns noted, not re-argued

Per the brief, these are catalogue-wide rules already recorded elsewhere. This
batch's instances are listed so a later pass can count them.

### ArMDE:2816 — the one-Social-Status rule

> All characters must take one Social Status, and may only take more than one if the descriptions … explicitly note that they are compatible.

**Eight instances in this batch**, none of which carries any encoding of the
rule: `virtue.journeyman`, `virtue.jurist`, `virtue.knight`, `virtue.laborer`,
`virtue.landed_noble`, `virtue.lasiq`, `virtue.lone_redcap`,
`virtue.lupus_the_wolf`. That is the running total's largest single-batch
contribution so far.

**This batch is unusually rich in the *exceptions* the rule anticipates**, which
makes it the best evidence yet that the rule is doing real work and that not
encoding it loses information in both directions. Four of the eight carry an
explicit compatibility note, and **none of the four is encoded either**:

- `virtue.jurist` (ArMDE:4167): "This Virtue is compatible with the Baccalaureus, Magister in Artibus, and Doctor in (Faculty) Virtues … It is also compatible with the Priest and Mendicant Friar Virtues." — five named partners.
- `virtue.knight` (ArMDE:4197): "… is compatible with the Landed Noble Virtue."
- `virtue.landed_noble` (ArMDE:4227): "This Status Virtue is compatible with the Knight Minor Status Virtue, but unlike that Virtue it is available to male and female characters." — the reciprocal, stated on both entries.
- `virtue.lone_redcap` (ArMDE:4325): "This social status is compatible with any other **mundane** Social Status Virtue that would reasonably allow you to do your job as a Redcap, such as Merchant or Mendicant Friar." — a *scoped* permission, which is the hardest shape: it is neither "compatible with everything" nor a closed list.

So the modelling question ArMDE:2816 poses is not "are Social Statuses mutually
exclusive" but "how is a per-entry exception list expressed". `incompatible_with`
is the wrong polarity for it (B7: it is a *forbid* list requiring symmetry, and
encoding :2816 that way would mean every Social Status listing every other one
except its exceptions — quadratic, and the symmetry guard would have to hold
across all of it).

**The "at most one" half, though, is a profile field that already exists — and
no profile uses it.** *(Contributed by the verification pass; re-checked in the
data before being written here.)* `rules/core/character_types.json` declares
`virtue_category_caps`, and the magus profile uses it for a different category:
`{"category": "hermetic", "max": 1, "major_only": true, "hard": true}` at
`character_types.json:90-92`. B5 records that the cap machinery is **derived from
the slug**, so no category is hardcoded in Rust — a
`{"category": "social_status", "max": 1}` row would be a pure data change with
no code at all. `grep -n "social_status" rules/core/character_types.json
rules/core/mythic_companion_types.json` returns four hits and **every one is a
`permitted_categories` membership**, never a cap.

That changes the shape of the problem usefully: the *default* half of :2816 is a
one-line data addition per profile that the engine already enforces, and only the
*exception* half needs a mechanism that does not exist. Recording it here so the
eight entries above are not "fixed" one at a time. **`virtue.journeyman` and
`virtue.laborer` remain clean** — the gap is at the profile level, not theirs.

### ArMDE:2960-2962 — realm association for Supernatural Virtues

> All Supernatural Virtues and Flaws are associated with one of the four realms, Magic, Faerie, Infernal, and Divine. … A Virtue's description notes if it is limited in this way.

**Seven instances** — every Supernatural entry in this batch — and **none**
carries a `realm` parameter: `virtue.kassalan_exorcism`,
`virtue.land_regio_network`, `virtue.leather_ripper`,
`virtue.lesser_benediction`, `virtue.lesser_immunity`, `virtue.lesser_power`,
`virtue.lesser_purifying_touch`.

Two of the seven restate the rule in their own passage, and they are opposite
cases, which is worth recording because B04 found the same split:

- `virtue.kassalan_exorcism` (ArMDE:4177) **fixes** a realm: "Kassalan Exorcism is aligned to the Magic Realm." / "ist der Magischen Sphäre zugeordnet." A fixed association is the strongest possible case for the missing parameter — there is not even a choice to model, just a constant nobody stored.
- `virtue.lesser_power` (ArMDE:4285) leaves it **open**: "This Virtue may be associated with any supernatural realm. … The power must be associated with the same supernatural realm as the system on which it is based." That is the player-choice case, and it is the exact wording B04 quoted for `virtue.greater_power`.

`tainted` is correctly `false`/absent on all seven: none of the seven descriptor
lines carries a `Tainted` tag (checked against each `*…*` line, ArMDE:4174,
4212, 4246, 4254, 4276, 4280, 4288).

`virtue.student_of_realm` remains the shape that would work — a `realm`-domain
parameter alongside the effect.

### The `ability_authorization` pattern — four new instances, all in one batch

F-122 (`virtue.jurist`, `abilities` shape), F-128 (`virtue.knight`,
`categories` shape), F-132 (`virtue.lasiq`, `categories` shape), F-155
(`virtue.lupus_the_wolf`, `abilities` shape). All four are the **"no effect at
all"** variant: none of the four entries carries *any* effect, so there is
nothing to repair in place.

**Two near-misses that are NOT instances**, recorded so they are not
double-counted later:

- `virtue.lone_redcap` carries a `restricted_ability_xp` over
  `academic`/`arcane`/`martial` (among others), whose A7 permission side-effect
  already authorizes exactly the categories `virtue.redcap`'s passage names. No
  `ability_authorization` row is owed, and adding one would be redundant — the
  same conclusion B04 reached for `virtue.hermetic_experience` and
  `virtue.ineslemen`.
- `virtue.magian_lineage_major` names Arcane and Supernatural Abilities but
  **explicitly withholds** permission: ArMDE:4345 "A character must have access to
  a Supernatural Ability in one of the normal ways, such as taking the Virtue at
  character creation or Initiating it later, in order to put any experience points
  into it." The passage says the opposite of an authorization, so none is owed.

## Five new patterns this batch raises for later batches

### New pattern 1 — conditional `combat_mod` folded flat

F-149 (`virtue.lightning_reflexes`) folds a triply-conditional +9 flat into
every displayed Initiative Total. A35's own note names a second instance in the
same breath — "**Berserk's combat bonuses apply always**" — and `combat_mod` has
**17 uses** in the catalogue, none of which this audit has yet examined for
conditionality. D4 settled the identical question for `lab_total_mod` and settled
it *against* the flat fold, resolving each condition statically. **Every batch
from here should check each `combat_mod` it meets against its passage for a
condition, exactly as B04's brief had it check `lab_total_mod` carriers.** The
same test applies to `soak_mod` and `health_mod`.

### New pattern 2 — the **named-Virtue** cross-reference, which no page-number screen can see

B03 and B04 established the `(see page NNN)` pointer as the audit's most
productive pattern. This batch adds its harder sibling: a pointer that names
another Virtue or Flaw **with no page number at all**.

- `virtue.lone_redcap` (ArMDE:4321): "receive the benefits of **the Well-Traveled virtue**" — and the granted entry computes nothing, so the benefits are nil. Flagged for B09 under F-151.
- `virtue.leper_magus` (ArMDE:4251): "granting **the Life Boost Minor Virtue**" — Q-37.
- `virtue.landed_noble` (ArMDE:4221): "balance this Virtue with **that Flaw** … the normal points for **Oath of Fealty**" — benign; `flaw.oath_of_fealty` exists and "the normal points" means no arithmetic change.
- `virtue.large` (ArMDE:4231): "Giant Blood (page 83), Small Frame (page 145), or Dwarf (page 126)" — the hybrid form, with pages; correctly encoded and symmetric.

**The dangerous shape is a named Virtue reached through `grants_selection`.**
A grant is only as good as the granted entry's own effects (A18), so an entry can
be perfectly authored and still deliver nothing. Nothing in the repo checks that a
`grants_selection` target actually carries effects — `validate_effect_refs` only
checks the id resolves. Later batches should, for every `grants_selection` they
meet, open the granted entry and ask whether it implements what the granting
passage promises. **Seven entries in the catalogue carry `grants_selection`; this
batch checked the one in its own span and found it hollow.**

### New pattern 3 — one Power Virtue gets a derived read-out and two do not

Recorded under F-142 in full. In short: `derived/focus_power.rs` computes
magnitude, Initiative and activation Fatigue for `Entity::focus_powers`, while
`Entity::powers` — the `power_levels` currency that `virtue.lesser_power` and
`virtue.greater_power` feed — is read only by
`effective/gift_confidence.rs::powers_used` for the budget bar. All three
Virtues state a formula, the three formulas differ from each other, and only one
is computed. **A later batch meeting any further `power_levels` or
`focus_points` carrier should check it against this table rather than assuming
the family is uniformly uncomputed.** It also means B04's F-96
(`virtue.greater_power`) is stronger than B04 could see: the engine has the
shape, one file over.

### New pattern 4 — "may purchase / may begin play with" is a permission, not colour

Three of this batch's four authorization findings use a verb other than "may
take": "may **purchase** the Abilities" (:4165), "may **begin play with** scores
in" (:4337), "may **have** high quality weapons and armor" (:4197). A screen
keyed on "may take … Abilities" would have missed two of the four. The pattern
to look for is a **named Ability or Ability category** plus any permission verb,
not a fixed phrase.

### New pattern 5 — "once per story / session" entitlements are a closed family of three

Established by F-160 and listed here so a later batch does not have to rediscover
it. `grep -n "[Oo]nce per st\|[Oo]nce per session"` over the English core
rulebook returns **exactly three** hits, and they are the whole family:

| ArMDE | Entry | Classification today | In `SWEPT_BLOCKS`? |
|---|---|---|---|
| 3386 | `virtue.all_according_to_plan` | `uncomputed_rule` | yes |
| 4203 | `virtue.knows_people` | `narrative` → **F-160** | no |
| 5091 | `virtue.supernatural_beauty` | `narrative` — **B08's** | no |

**B08 is flagged for `virtue.supernatural_beauty`** (ArMDE:5089-5096), which this
batch did not audit and does not rate. Its :5091 is the structural twin of
Knows People's :4203 — "A player may use this Virtue, **once per story**, to ask
a storyguide to insert a fortunate coincidence, of the storyguide's choice, into
a scene. The troupe may veto the use of the Virtue in any situation where
supernatural aid seems profoundly unlikely." — including the same qualified
veto. If F-160 stands, B08 must reach the same verdict there, and the two should
be decided together rather than diverging.

**While confirming that citation I read the rest of the entry and noticed a
second thing B08 should check, which neither this pass nor the verification pass
was looking for:** ArMDE:5095 states "A character lacking a positive Presence
score may not have this Virtue" — a stated precondition, and the same "may not
**have**" verb B02 collected as the *non*-purchase form. That is D2's vocabulary,
and `Prereq` has no characteristic-minimum variant at all (B6: the eight variants
are `all`/`any`/`none`/`has`/`house`/`ability_min`/`art_min`/`is_magus` — there
is no `characteristic_min`). Recorded, not rated.

---

## Sub-agent reconciliation

**One** verification sub-agent was run, after the first pass over all 35 entries
was complete and on disk. It was given the seven entries this batch had cleared
(six fully clean, one carrying only an open question), told to **re-derive each
verdict from the rulebook and the code alone**, told explicitly that overturning
one would be a success and inventing one would not, and **forbidden to read
`batch-05.md`**. It was allowed `README.md`, `decisions.md`,
`engine-semantics.md` and `batch-01.md`…`batch-04.md`, and allowed no sub-agents
of its own. It confirmed it changed no file, ran no git command **of any kind**,
and saw and ignored the "auto mode" Bash injection.

**Result: 6 confirmed, 1 overturned.** The overturn was re-verified against the
sources before being accepted; every count in this file is post-reconciliation.

| Entry | Verification verdict | Outcome |
|---|---|---|
| `virtue.jack_of_all_trades` | confirmed clean | stands |
| `virtue.journeyman` | confirmed clean | stands |
| `virtue.laborer` | confirmed clean | stands |
| `virtue.large` | confirmed clean | stands |
| `virtue.latent_magic_ability` | confirmed clean | stands |
| `virtue.light_touch` | confirmed clean | stands |
| `virtue.knows_people` | **OVERTURNED** — `?` was wrong; it is a defect | accepted as **F-160**; **Q-35 closed** |

### What I re-checked before accepting the overturn

The audit's rule is that a verdict is re-verified against the source, not
accepted on a sub-agent's say-so. Three claims carried the overturn and all three
were re-opened here:

1. **`grep -n "[Oo]nce per st\|[Oo]nce per session"` over the English core rulebook returns exactly three hits** — ArMDE:3386, :4203, :5091. Confirmed, and the family is therefore closed rather than sampled.
2. **`virtue.all_according_to_plan` (ArMDE:3384-3387) is `uncomputed_rule` with zero effects, and its range lies inside `SWEPT_BLOCKS`' 3360-3950.** Confirmed by `jq` over `rules/core/virtues_flaws.json` and by reading `SWEPT_BLOCKS`. This is the load-bearing one: it makes the overturn a *consistency* argument against an existing catalogue decision rather than a fresh judgement call.
3. **ArMDE:4205's veto is qualified, not unconditional.** Confirmed by re-reading the sentence in both languages. This is exactly what the first pass got wrong, and it is why it withheld a verdict it should have taken.

I also checked the one way the overturn could have been wrong: whether
`all_according_to_plan` is `uncomputed_rule` for a *different* reason that does
not generalise. It is — its passage says "reroll a **botch die**", which is a
token the screen recognises — but that cuts the other way. The token is why the
sweep caught it; the *classification* it received is a decision about the "once
per session, a player may …" shape, and `knows_people` and `supernatural_beauty`
differ from it only in carrying no token any screen knows. That is the README's
own thesis, demonstrated on a three-member family.

### The corroborations the verification pass added to entries that stayed clean

Worth recording because they harden verdicts rather than change them, and because
each is something the first pass asserted and did not prove:

- **`virtue.large`** — the first pass established from A23 that the wound bands scale with Size. The verification pass went further and matched the computed bands against the book's own table: `u = max(1, size + 5)` = 6 at Size +1 gives 1-6 / 7-12 / 13-18 / 19-24 / 25+, which is the `+1` row of the Damage Table at **ArMDE:17176**, with :17180 stating the general rule. So "wounds … increase in six point intervals rather than five" is not merely *modelled* but demonstrably *correct*. It also verified the incompatibility symmetry in the book as well as in the data (ArMDE:5998 and :6769 each name Large), which this pass had checked only in the data.
- **`virtue.jack_of_all_trades`** — it independently reached the same conclusion about the "(see page 157)" pointer (ArMDE:7128 is the baseline the Virtue overrides, and the Virtue restates both halves), and added the sharper point that this entry is a **negative** instance of the `ability_authorization` pattern: ":4157 he must have a Virtue permitting him to take Arcane Abilities" *requires* a separate authorizing Virtue, so adding an `ability_authorization` row here would be a rules error, not a fix. Recorded so a later sweep for that pattern does not "correct" it.
- **`virtue.light_touch`** — it stress-tested the one clean verdict that carries a number (+1) and reached the same place by three routes: the botch-die half is uncomputable regardless, so no effect could carry the whole rule; `ability_roll_mod` requires a `text` param naming a player-chosen subject and there is no choice here; and per C1 that variant is surfaced-only anyway, so authoring it would trade a complete description for an inert row. All three clauses are in `description` in both locales, so D5 is satisfied.
- **`virtue.latent_magic_ability`** — it tested the one clause that could have broken `narrative`, "This is not The Gift", and correctly ruled that it *disclaims* a mechanic rather than asserting one: it neither forbids holding both nor grants anything. It also noted that "Latente Magische Fähigkeit" is **not** a `Fähigkeit`/`Fertigkeit` violation, because "Ability" here is the English word for a capacity and not the game term — confirmed by the DE body text's "magische Eigenschaft".

### Contributions accepted

- **The `virtue_category_caps` observation** — folded into the ArMDE:2816 pattern above and re-checked in `rules/core/character_types.json` before being written. It materially changes that pattern: the "at most one Social Status" half is a one-line data addition to an existing, slug-derived cap mechanism, not a missing subsystem.
- **The `virtue.supernatural_beauty` twin** — folded into new pattern 5 as a flag for **B08**, together with a second thing found while confirming its citation (ArMDE:5095's "A character lacking a positive Presence score may not have this Virtue", which `Prereq` has no variant for).

### Contributions examined and NOT adopted

- **"`source.lines` uses two conventions for the trailing blank line" (~26 entries).** True, and the verification pass quantified it more precisely than anyone has — but **B04 examined the identical claim and declined it**, on the grounds that `rules_source_provenance.rs` accepts both, no range bisects an entry or bleeds into a neighbour, and B03 had already recorded the same variation without rating it. That precedent is followed here. `virtue.large`'s `[4229,4231]` is not a defect. INFO, catalogue-wide, not this audit's object.
- **"`anchor` is absent on all seven, and could be filled."** The offer is genuine and the anchors it verified against the book's own link targets look right (`jack-of-all-trades` :3279/:24942, `journeyman` :3211/:24951, `knows-people` :3283/:24969, `laborer` :3344/:24978, `large` :3284, `latent-magic-ability` :24985, `light-touch` :3287). But B11 states plainly that `anchor` is "**Present only for entries a sweep has already read**" — so its absence is the *expected* state for an unswept span and is not a defect. Filling them is a correction-pass decision, not a batch verdict. Recorded here so the work is not lost.
- **Its observation that five neighbouring entries "plainly state mechanics while sitting at `narrative`".** All five are findings in this batch already (F-126, F-124, F-125, F-122, and B04's F-118 for `virtue.intuition`), reached independently and before the sub-agent ran. Noted as convergence, not as a new contribution.

### One environment note the verification pass reported

The native **Grep tool errored out in its session**, so its content searches fell
back to allowlisted `grep`/`jq` through Bash. That is the sanctioned fallback and
it used Read for every file inspection, so no rule was bent — but it is worth
knowing for the next batch's sub-agent brief, because a Grep-tool outage makes
the `-I`/`grep -a` binary-file trap live again (the native tool and the injected
`grep` skip binary files by different mechanisms, and only `grep -a` overrides
the injected one).

---
