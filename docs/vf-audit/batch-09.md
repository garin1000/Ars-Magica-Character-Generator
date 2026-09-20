# Batch B09 — indices 280-314, ArMDE:5097-5256

Entries: 35. Audited: 35. Failures: **30**. Clean: **5**.
Findings: **F-309 … F-348** (40). Open questions: **Q-72 … Q-78** (7).

**F-348 was found by the verification sub-agent, not by the first pass**, and it
is the batch's cleanest single defect: `virtue.voice_of_the_land` carries a
free-text parameter and no `max_total`, so a Virtue ArMDE:2814 permits **once**
can be taken without limit, one copy per land. The first pass had marked that
entry `?` on classification and waved its multiplicity fields through as "OK" —
it asked what the fields *said* and never what `max_total`'s **absence** meant.
See the reconciliation section.

Finding and question numbers **continue B08's sequence** (B01 F-01…F-33 /
Q-01…Q-08, B02 F-34…F-65 / Q-09…Q-16, B03 F-66…F-89 / Q-17…Q-22, B04 F-90…F-121
/ Q-23…Q-34, B05 F-122…F-160 / Q-35…Q-44, B06 F-161…F-208 / Q-45…Q-54,
B07 F-209…F-257 / Q-55…Q-63, B08 F-258…F-305 / Q-64…Q-69). The brief directs
this batch to start at **F-309** and **Q-72**; the gaps F-306…F-308 and
Q-70/Q-71 are the orchestrator's reservation and are not this batch's to fill.

**This span has never been screened.**
`crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS` stops at
ArMDE:3950; the whole of 5097-5256 lies outside it, as B04's through B08's spans
did. The failure rate (29/35) sits with theirs (B04 28, B05 29, B06 31, B07 30,
B08 26). The profile of this span is dominated by three things:

- **Nineteen `narrative` reclassifications** — `templar_administrator`,
  `templar_confrere_or_consoeur`, `templar_prestige`, `templar_specialist`,
  `temporal_influence`, `tethered_magic`, `town_magistrate`, `troubadour`,
  `troupe_upbringing`, `true_love_pc`, `turb_trained`,
  `unaffected_by_the_gift`, `unbound_tongue`, `university_grammar_teacher`,
  `variable_power`, `venus_blessing`, `verditius_magic`, `well_traveled`,
  `wisdom_from_ignorance`. **Seven** go to `creation_effect` or
  `in_play_effect` (`templar_administrator`, `templar_prestige`,
  `temporal_influence`, `town_magistrate`, `troubadour`, `troupe_upbringing`,
  `turb_trained`, `university_grammar_teacher`, `verditius_magic` and
  `well_traveled` between them, once their missing data is added) and the rest
  to `uncomputed_rule`. Four are refuted by a plain digit alone:
  `troupe_upbringing` "+2", `true_love_pc` "+3", `venus_blessing` "+3", and
  `templar_prestige` "Reputation of level 4". **`virtue.verditius_magic` is the
  batch's rarest transition and the one the README says has never been looked
  for** — a `narrative` entry whose two rules the engine *already computes*, via
  a prerequisite and a `houses.json` grant rather than an `Effect` (F-337).
- **Seven more instances of the `ability_authorization` pattern**, now in nine
  consecutive spans — `templar_administrator`, `templar_specialist`,
  `town_magistrate`, `troubadour`, `turb_trained`,
  `university_grammar_teacher`, `venditor`. Every one is a mundane Virtue, so
  A14's `is_magus` exemption is empty and `validate_ability_authorization`
  raises a hard error on a legal character.
- **The batch's two strongest findings both came from following a page
  pointer.** `virtue.true_faith`'s "see page 419" lands on ArMDE:17603-17617,
  which states **"A character with a True Faith Score gains Magic Resistance
  equal to this score multiplied by ten"** — a number the engine computes
  nowhere (F-329); and `virtue.verditius_magic`'s "(see page 240)" lands on
  ArMDE:10076-10090, which is the entire Verditius Magic Outer Mystery: the
  Philosophiae Rune bonus, the Craft-Ability Lab Total addend, the reduced
  vis-to-open cost and the casting-tool requirement, none of which reaches the
  user in either locale (F-338).

Two further span-wide shapes worth naming up front:

- **A range that swallows a sidebar, and a rule that consequently belongs to
  nobody.** `virtue.tainted_treasure`'s `source.lines` runs to 5108 and so
  contains the block-quote sidebar `> #### The Knights Templar` (ArMDE:5105-5107),
  whose second sentence is a **rule** — women cannot use Paid Rights to take a
  male-only Templar role — governing the eight Templar entries that follow and
  reaching none of them (F-309).
- **`virtue.wealthy`'s `incompatible_with` is empty, and so is every entry's.**
  Eight core passages state a Wealthy/Poor exclusion (ArMDE:3611, :3631, :4494,
  :4634, :4850, :5181, :5751, :6540) and `jq` over the whole catalogue returns
  **no** entry naming `virtue.wealthy` or `flaw.poor` in `incompatible_with`
  (F-340). One of the eight is this batch's own `virtue.turb_trained` (F-325).

*(This file was written incrementally — the method notes and the verdict table
landed first, findings were appended as each entry was finished, and the
sub-agent reconciliation last.)*

## Method

The whole span was read as continuous prose in **both** languages before any
entry was judged — `rules/source/en/Ars Magica - Definitive Edition (Core
Rules).md` ArMDE:5089-5268 and the line-parallel
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md` 5089-5268 (the
read deliberately overran the span at both ends so the first and last entries'
boundaries could be seen).

**Line parity holds throughout the span**, checked at every heading rather than
sampled: 5097, 5109, 5113, 5117, 5121, 5125, 5129, 5133, 5137, 5141, 5145, 5149,
5153, 5157, 5165, 5169, 5173, 5179, 5183, 5187, 5191, 5195, 5199, 5207, 5211,
5215, 5219, 5223, 5227, 5231, 5235, 5239, 5243, 5247, 5251 — **35 `####`
headings for 35 entries**, one each, each on the same line number in both files.
The one non-entry heading inside the span is the block-quote
`> #### The Knights Templar` at 5105 (DE `> #### Die Tempelritter`, same line),
which is **not** a sub-heading of the entry whose range contains it — see F-309.

**`source.lines` (check 1) — 34 of 35 correct, 1 defective.** The dominant
convention is "own heading → the line before the next heading"; three entries
(`tough` 5145-5147, `verditius_magic` 5215-5217, `warrior` 5227-5229) stop one
line earlier, on their last body line rather than on the trailing blank, which
is tighter and equally correct. Every non-blank line in 5097-5256 is cited by
exactly one entry. The single defect is `virtue.tainted_treasure`'s upper bound
(F-309): mechanically it follows the dominant convention, but the convention
produces the wrong answer here because a **sidebar** sits between two entries.

**No entry in this batch carries an `anchor`** — the `jq` dump of all 35
`source` objects returns the key set `["file","lines"]` and nothing else, so
B11's "confirm the anchor names the entry" check has nothing to verify and the
line ranges are the only provenance.

**Magnitude, kind, categories, entity_kinds (checks 3, 4, 5, 6).** Every
descriptor line was read and compared against the data. `kind` is `virtue` for
all 35 and all 35 sit inside the Virtues section — correct. `entity_kinds` is
`["character"]` on all 35, which is right: none is a covenant Boon. `magnitude`
and `categories` agree with **every** descriptor:

- `*Minor, General*` — tainted_treasure (+`Tainted`), templar_prestige,
  temporal_influence, tough, trained_assassin, troupe_upbringing, true_love_pc,
  unaffected_by_the_gift, venus_blessing, well_traveled
- `*Major, General*` — true_faith, ways_of_the_land, wealthy
- `*Minor, Social Status*` — templar_administrator, templar_office_holder,
  templar_specialist, town_magistrate, troubadour, turb_trained,
  university_grammar_teacher
- `*Major, Social Status*` — templar_commander, venditor
- `*Free, Social Status*` — templar_confrere_or_consoeur, templar_servant,
  wanderer
- `*Minor, Hermetic*` — tethered_magic, verditius_magic
- `*Minor, Supernatural*` — unaging, unbound_tongue, variable_power,
  voice_of_the_land, wilderness_sense, wisdom_from_ignorance
- `*Major, Supernatural*` — whistle_up_the_wind
One descriptor is misprinted in the **English** source and is listed above under
its correct reading: ArMDE:5196 (`university_grammar_teacher`) prints
`*Minor. Social Status*` with a **period** for a comma. It parses unambiguously,
the data reads it correctly as `minor` + `social_status` ✓, and the DE at the
line-parallel 5196 prints a normal comma (`*Klein, Sozialer Status*`), so the
typo is EN-only. Same OCR shape B08 recorded at ArMDE:5074.

`virtue.tainted_treasure` is the **only** entry in the batch with
`tainted: true`, and ArMDE:5098 is the only descriptor line carrying the
`Tainted` type label ✓ (DE 5098 `Befleckt` ✓, which is the type-label spelling
CLAUDE.md fixes).

**Checks 7, 8, 9, 10 at a glance.** `prerequisites` is present on exactly one
entry (`virtue.verditius_magic`, `{ kind: house, value: house.verditius }` ✓,
documented at `RULES.md:3760-3775`); three entries state a prerequisite the
engine *can* express and carry none (F-322 Town Magistrate, F-324 Trained
Assassin) or cannot express (gender, age). `incompatible_with` is **empty on all
35**, which is a defect on two of them (F-325, F-340). `parameters` is present
on three (`variable_power` `power`/text/`require_power` ✓,
`voice_of_the_land` `land`/text, `ways_of_the_land` `land`/text) and missing on
one that needs it (F-328). Ten entries carry `effects`; the eleven effect rows
are checked individually below.

**Check 12, mechanical.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the batch's 35 entries in **each**
locale returns the key set `["name","summary"]` — except `virtue.variable_power`
(both locales, `name_unfilled` as well) and `virtue.ways_of_the_land` (both
locales, `description` as well). **`virtue.ways_of_the_land` is the only entry
in the batch carrying a `description`, in either locale**, which is why every
D5 finding below reads "reaches neither locale".

No negative sign appears in any `summary` or `description` in the batch, in
either locale — the only signed values in the span's prose are the `+3`/`+2`
bonuses, so the ASCII-hyphen rule has nothing to bite on here. Two negative
numbers a correction-pass `description` would have to transcribe do exist in
passages this batch's cross-references reach and must use the ASCII hyphen when
written out: ArMDE:9244-9245's `-5`/`-10`/`-2` Words-and-Gestures table (which
`virtue.unbound_tongue` waives, F-331) and ArMDE:9247's "–9".

### German names against the canonical tables

Checked against `rules/source/de/translation-tables/` **and**
`rules/i18n/de/abilities.json`, not only against the DE rulebook. Seventeen of
the batch's German names appear in a table; **fifteen agree with it and two do
not** — and in **both** disagreements the table contradicts the DE rulebook, so
per the audit's rule the finding is against the **table**, not the data.

| Entry | Table row | Table's DE | Data's DE | |
|---|---|---|---|---|
| Tainted Treasure | `sphären-mächte.md:281` | Verfluchter Schatz | Verfluchter Schatz | ✓ (the row even repeats the descriptor "Klein, Allgemein, Befleckt; in Basisregeln bestätigt") |
| Templar Commander | `reputationen.md:86` | Templerkommandeur | Templerkommandeur | ✓ — and the row independently records the Reputation the entry drops: "Lokal/Regional | 3 (+)". See F-312. |
| Tethered Magic | `tugenden-fehler.md:73` | Gefesselte Magie | Gefesselte Magie | ✓ |
| Tough | `tugenden-fehler.md:215` | Zäh | Zäh | ✓ (the *name* is right; the **summary's** term is not — F-320) |
| Troupe Upbringing | `tugenden-fehler.md:216` | Zirkuserziehung | Zirkuserziehung | ✓ |
| True Love | `tugenden-fehler.md:407` | Wahre Liebe | Wahre Liebe (SC) | ✓ — the "(PC)" → "(SC)" disambiguator is the DE rulebook's own at 5173 |
| Unaffected by The Gift | `tugenden-fehler.md:221` | **Unempfindlich gegenüber der Gabe** | **Unbeeindruckt von der Gabe** | ✗ table disagrees with the rulebook — **F-345** |
| Unaging | `tugenden-fehler.md:217` | **Nicht-Alternd** | **Nicht alternd** | ✗ table disagrees with the rulebook — **F-345** |
| Unbound Tongue | `tugenden-fehler.md:144` | Ungebundene Zunge | Ungebundene Zunge | ✓ |
| Variable Power | `tugenden-fehler.md:145` | Variable Kraft | Variable Kraft | ✓ |
| Venditor | `orden-tribunale.md:102` | Venditor (Venditores) | Venditor | ✓ — "Lat. Fachbegriff … im DE-Text unübersetzt belassen – korrekt", exactly the Latin-stays-untranslated convention |
| Venus's Blessing | `tugenden-fehler.md:218` | Venussegen | Venussegen | ✓ |
| Voice of the (Land) | `tugenden-fehler.md:146` | Stimme des/der (Land) | Stimme des {land} | ✓ on the noun; the table's "des/der" flags a gender problem the template cannot express — **Q-77** |
| Wanderer | `persoenlichkeitseigenschaften.md:154` | Wanderer | Wanderer | ✓ (that table is Personality Traits, a different item with the same rendering, so it does not bind; the DE rulebook heading at 5223 is the same word) |
| Warrior | `tugenden-fehler.md:219` | Krieger | Krieger | ✓ |
| Wealthy | `tugenden-fehler.md:222` | Wohlhabend | Wohlhabend | ✓ ("Konsistent im Text verwendet") |
| Well-Traveled | `tugenden-fehler.md:223` | Vielgereist | Vielgereist | ✓ |
| Whistle Up The Wind | `tugenden-fehler.md:108`; `fertigkeiten.md:94` | Den Wind herbeipfeifen | Den Wind herbeipfeifen | ✓ |
| Wilderness Sense | `tugenden-fehler.md:147`; `fertigkeiten.md:95` | Natursinn | Natursinn | ✓ |

> **`tugenden-fehler.md` is being edited by another workstream while this batch
> runs, and its line numbers moved under me.** A first pass of these citations
> returned rows two lines lower (Unaging at `:219`, Unaffected by The Gift at
> `:223`); a re-check minutes later returned `:217` and `:221`, and
> `git diff` shows two rows deleted from the file in between. Every
> `tugenden-fehler.md` citation above was **re-verified against the working
> tree after that change** and is current as of this writing — but a correction
> pass should locate these rows by their **English key**, not by line, because
> the line will move again. The other five tables cited in this batch
> (`reputationen.md`, `sphären-mächte.md`, `kampf.md`, `orden-tribunale.md`,
> `fertigkeiten.md`, `persoenlichkeitseigenschaften.md`, `tiere-kreaturen.md`)
> were re-checked in the same pass and are **unchanged**.

Two further table rows are **near-misses that are not findings**, recorded so
they are not re-raised:

- `reputationen.md:85` — `Hermetic Prestige (via Templar) | Templer-Ansehen |
  Templer | 4 (+)`. The English key is not a Virtue name in the catalogue, so
  the row does not bind `virtue.templar_prestige`'s German name — but it *does*
  independently record both halves of F-316 (a Reputation of **4**, scoped to
  **"Templer"**, which is none of the four `ReputationType` values — Q-74).
- `reputationen.md:87` — `Templar Officer | Templer-Offizier | Regional | 2 (+)`.
  The English key is "Templar Officer", not "Templar Office Holder", so it does
  not bind the data's `Templerstelleninhaber` (which matches the DE rulebook
  heading at 5121 ✓). The row corroborates the **score 2** the entry already
  carries.
- `tiere-kreaturen.md:98` — `Tough Hide | Zähes Fell | +2 Schutz; stapelt mit
  Zähe-Tugend …`. A creature Quality, not this Virtue; different item.

The remaining German names appear in no table (`templar_administrator`,
`templar_confrere_or_consoeur`, `templar_office_holder`, `templar_prestige`,
`templar_servant`, `templar_specialist`, `temporal_influence`,
`town_magistrate`, `trained_assassin`, `troubadour`, `true_faith`,
`turb_trained`, `university_grammar_teacher`, `verditius_magic`,
`ways_of_the_land`, `wisdom_from_ignorance`), so the table constraint does not
bind them; each was checked against its DE rulebook heading instead and all
match exactly.

**DE Ability names the passages depend on were confirmed against
`rules/i18n/de/abilities.json`**, not inferred: Whistle Up The Wind → **Den Wind
herbeipfeifen**, Wilderness Sense → **Natursinn**, Civil and Canon Law →
**Zivil- und Kanonisches Recht**, Common Law → **Gewohnheitsrecht**, Artes
Liberales → **Artes Liberales**, Teaching → **Unterrichten**, Living Language →
**Lebende Sprache**, Dead Language → **Tote Sprache**, Intrigue → **Intrige**,
Folk Ken → **Menschenkenntnis**. All match the DE rulebook body at
ArMDE:5151, :5197, :5209, :5245, :5249.

### Cross-references followed

Every pointer in the span was followed and read — the `(see page NNN)` form, the
chapter form and the page-number-free named-Virtue form alike — including on
entries that turned out to pass. **The two that paid are `true_faith` and
`verditius_magic`, both `(see page NNN)` pointers**, which is the first time in
the audit that the numbered form has out-produced the named form.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the Virtue? |
|---|---|---|---|
| sidebar ArMDE:5107 | "see Chapter 7 of *The Church*" | **outside `rules/source/en/`** | Nothing claimable. What the sidebar states *in its own voice* is the Paid Rights non-override → F-309. |
| `virtue.tainted_treasure` | "Tragic Life Flaw", "Plagued by Supernatural Entity Flaw" (named, no page) | `flaw.tragic_life` ArMDE:6855-6870 ✓, `flaw.plagued_by_supernatural_entity` ArMDE:6590-6593 ✓ | **No** — a story example of a swap the *Tainted* rules permit, not a rule of this Virtue. |
| `virtue.templar_administrator` | "the Brother-Knight, Brother-Sergeant, and **Brother-Priest** Status Virtues" (named, no page) | `virtue.brother_knight` ArMDE:3533 ✓, `virtue.brother_sergeant` ArMDE:3537 ✓, **Brother-Priest: no catalogue id and no `####` heading** | **Yes, a Social-Status replacement rule** → F-311; and the third name resolves to nothing → **Q-72**. |
| `virtue.templar_commander` | "the Temporal Influence Minor Virtue", "the Brother-Knight Virtue" (named, no page) | `virtue.temporal_influence` ArMDE:5137-5140 ✓, `virtue.brother_knight` ✓ | **Yes — and both are already encoded** as `grants_selection: [virtue.brother_knight, virtue.temporal_influence]` ✓ (`RULES.md:3024-3040`). The Reputation in the same paragraph is **not** → F-312. |
| `virtue.templar_confrere_or_consoeur` | "Clerk, Knight, Landed Noble, Peasant, or even Hermetic Magus" (named, no page) | all resolve in the catalogue | **Yes, an ArMDE:2816 compatibility override** ("he may possess other Social Status Virtues or Flaws") → F-314. |
| `virtue.templar_office_holder` | "any of the Templar Status Virtues", "the Temporal Influence Minor Virtue", "the Templar Commander Major Virtue" | this batch's own entries ✓ | **Yes, an ArMDE:2816 compatibility override, stated twice** → F-315. |
| `virtue.trained_assassin` | "one of the Social Status Virtues of the Nizaris" (named, no page) | `virtue.fidai` ArMDE:3877-3881 ✓ and `virtue.lasiq` ArMDE:4233-4236 ✓ — the passage names neither, so the referent had to be found by searching the source for "Nizari" | **Yes, a hard prerequisite**, and it is exactly expressible as `Any([Has, Has])` → F-324. |
| `virtue.troubadour` | "see *Faith and Flame*, page 12" | **outside `rules/source/en/`** | Nothing claimable. |
| `virtue.troubadour` | "Famous, Free Expression, Inspirational, Puissant Performance, Social Contacts, True Love, and Well-Traveled" | all resolve ✓ | **No** — an explicit suggestion list ("often possess"), not a grant. |
| `virtue.true_faith` | "For more about True Faith, see **page 419**." / DE `#wahrer-glaube-1` | `### True Faith` **ArMDE:17603-17617** | **YES — four rules, and the engine computes none of them.** ArMDE:17607 grants "a True Faith Score of 1, **and a single Faith Point**"; ArMDE:17611/:17613 give **Magic Resistance = Score × 10**; ArMDE:17609 the spend rule; ArMDE:17615 the dawn refresh → F-329, F-330. |
| `virtue.true_love_pc` | "Your True Love is another player character, who must also have this Virtue." | no target | **Yes, a reciprocity rule** the engine has no cross-character axis for → F-334. |
| `virtue.unaging` | "Decrepitude points", "crisis" (named, no page) | `aging.rs` / the aging chapter | **Yes, two clauses beyond the two `aging_mod` tags** → F-330b, reported as F-332. |
| `virtue.variable_power` | "(Greater, Lesser, Personal, or Ritual)" powers | the five Power Virtues ✓ | **No new rule** — the enumeration is the parameter's domain, and `RULES.md:2417-2448` already records why the `power` parameter is free text rather than an item ref. |
| `virtue.venditor` | "a Verditius maga" | House Verditius | **No** — colour. |
| `virtue.verditius_magic` | "the Outer Mystery of Verditius Magic (**see page 240**)" / DE `#verditius--verditius-magie` | `### Verditius — Verditius Magic` **ArMDE:10076-10090** | **YES — the entire Outer Mystery, imported wholesale**: the crafting season rule (:10078), the Philosophiae Rune bonus to Shape and Material (:10080), Finesse substituting for Craft (:10082), **the Craft Ability added to all enchanting Lab Totals (:10084)**, the vis-to-open reduction (:10086), the Rune reinforcement (:10088) and **the casting-tool requirement for Formulaic and Ritual spells (:10090)** → F-337. |
| `virtue.wealthy` | none in its own entry | — | The number it carries comes from **ArMDE:2214 / :2394**, not from ArMDE:5235-5238, and :2394 carries a restriction the data enforces only partly → F-339. |
| `virtue.well_traveled` | none outward; **two inward** | `virtue.lone_redcap` ArMDE:4321 ("receive the benefits of the Well-Traveled virtue") and `virtue.redcap` ArMDE:4848 ("you have the **Well-Traveled Virtue (page 116) at no cost**") | **Yes — the entry is the *target* of two grants and delivers nothing**, and one of the two granters does not even declare the grant → F-341, F-342. |
| `virtue.wilderness_sense` | "(page 172)" / DE `#natursinn-1` | the Wilderness Sense **Ability** | **No** — the Ability's own rules; the Virtue's mechanic (the score-1 grant) is encoded ✓. |

### Part C systemic gaps are not re-reported per entry

In particular: `later_life_xp_rate`'s unreachability on the flat XP-pool path
(C3b) is **not** counted against `virtue.wealthy`; "computes a number nothing
consumes" (C3c) is **not** counted against `virtue.true_faith` *as such* — what
**is** counted (F-329) is the different and specific fact that ArMDE:17611
states a Magic Resistance formula, which is a rule the passage gives and no
effect implements, i.e. a D5 obligation and a missing computation rather than a
scope boundary; `ability_roll_mod` being surfaced-only (C1) is **not** counted
against `virtue.troupe_upbringing`, only the absence of *any* carrier is
(F-327); the frontend `Effect` union's missing `later_life_xp_rate` tag (C6) is
**not** counted against `virtue.wealthy`; and `aging_mod`'s unread `amount` for
marker kinds (C2-d) is **not** counted against `virtue.unaging`.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**A note on which version of that file this batch read.** `decisions.md` gained
a **D6** (translation-table errors are fixed in both projects immediately;
tables are authoritative for terminology and **not** for facts about the rules;
check the source project before concluding a table row is wrong) **during** this
batch's run — the file was clean in the working tree when the batch started and
is modified now, by a concurrent workstream, not by this agent. D6 was read and
applied once it appeared: it governs **F-345** and it is the second reason
`reputationen.md`'s Reputation-level rows are treated here as corroboration
rather than as authority (F-312, F-316). Every other verdict below was reached
under D1-D5 and D6 does not disturb any of them.

**D1 does not bite.** No entry in this span carries a `lab_total_mod` (`jq` over
the batch's 35 entries returns none), so the spell-level-cap ruling has no
carrier here.

**D4's *shape* recurs once, on the casting side.** `virtue.ways_of_the_land`'s
+3 is conditional on the terrain ("that directly involve that area and its
inhabitants") and is folded flat as `scope: "all"` — the same complaint B07
raised as F-225 and B08 as F-284. Unlike those two, this one is **on record**:
`RULES.md:5131` lists `ways_of_the_land` among the `CastingTotalMod` carriers
and says "conditional ones folded **unconditionally** (no toggle exists)". It is
reported inside F-344 with that recording quoted, as corroboration rather than
discovery — F-344's *live* half is a different and unrecorded complaint, that
the bonus is not a casting bonus at all.

**D5 is applied to all ten effect-carrying entries in the span** —
`templar_commander`, `templar_office_holder`, `tough`, `trained_assassin`,
`true_faith`, `unaging`, `venditor`, `warrior`, `ways_of_the_land`, `wealthy`,
`whistle_up_the_wind`, `wilderness_sense` (twelve, counting the two
`ability_score_grant` carriers). **Five** leave a stated rule that no effect
implements and that appears in neither locale — F-313, F-315, F-330, F-332, and
`virtue.wealthy`'s three-free-seasons clause, which is reported inside F-339
rather than as a finding of its own. The six effect-carrying entries with
nothing left over are
**`virtue.tough`** (on the D5 axis only — it carries F-320, a German
terminology defect), **`virtue.trained_assassin`** (on the D5 axis only — it
carries F-324), **`virtue.warrior`**, **`virtue.ways_of_the_land`** (whose
`description` carries the whole passage verbatim in both locales — the only
entry in the batch that satisfies D5 by having actually been written out),
**`virtue.whistle_up_the_wind`** and **`virtue.wilderness_sense`**.

**D5 was also applied in the negative** — the passages of the two `narrative`
entries that survive were read for a mechanical clause. `virtue.templar_servant`
(ArMDE:5131) and `virtue.wanderer` (ArMDE:5225) state none. `virtue.wanderer`'s
"The Wealthy Major Virtue and Poor Major Flaw affect you normally" is a
*clarification that no exclusion applies*, which is the opposite of a rule, and
is recorded here so it is not mistaken for one. `virtue.voice_of_the_land`
(ArMDE:5221) could **not** be settled and is marked `?` with **Q-76**.

**D3 governs seven findings** where the engine structurally cannot express the
rule and the answer is therefore a `description` in both locales (or
`uncomputed_rule` for an effect-less entry), and **never** `narrative`:

| Finding | The rule | Why the engine cannot express it |
|---|---|---|
| F-311, F-313 | "only available to male characters" (ArMDE:5111, :5115) | `types.rs`'s `gender` field is declared "free-text; **no mechanical effect**", and no `Prereq` variant reads it (B6's eight variants). |
| F-317 | `virtue.templar_specialist`'s "you may take **one restricted group** of Abilities … such as Academic or Martial" | `AbilityAuthorization` (A14) carries a fixed `categories` list and has **no `param` field** — it cannot be made player-chosen. The `category` domain is no help either: B8 records it resolves against *the declaring item's own* `categories`, which are `["social_status"]` here. |
| F-323 | `virtue.university_grammar_teacher`'s "the Academic Abilities: **Latin** and Artes Liberales" | Latin is an *instance* of `ability.dead_language`. A7/A14 name the Ability, never one language — B08's F-273 on Simple Student, from the same shape. |
| F-325 | `virtue.turb_trained`'s "whichever **single** dead language the magi speak" | Same reason, and additionally instance-**count**-scoped. |
| F-330 | `virtue.true_faith`'s "and a single Faith Point" (ArMDE:17607) | `TrueFaithGrant` (A21) has exactly one field, `score`. There is no Faith-Point field and no `Entity` column for one. |
| F-332 | `virtue.unaging`'s "You are not enfeebled when you reach four Decrepitude points" and "If a crisis is not potentially fatal, you suffer no ill-effects" | `AgingEffect`'s eight kinds carry no Decrepitude-threshold waiver; `Decrepitude` itself is C1's one dead-and-unused value, and `crisis_survival` is a roll modifier, not a blanket "no ill-effects". |
| F-334 | `virtue.true_love_pc`'s "another player character, who must also have this Virtue" | The engine models one entity at a time; there is no cross-character reference of any kind. |

**D2 does not bite:** no entry here is a granted Great Characteristic carrier.
(`virtue.templar_commander` *is* a `grants_selection` carrier, and A18's
"a granted `Selection` carries no parameters" applies — but neither
`virtue.brother_knight` nor `virtue.temporal_influence` declares a parameter, so
the granted rows are not inert.)

**ArMDE:2816 ("All characters must take one Social Status, and may only take
more than one if the descriptions … explicitly note that they are
compatible").** **Twelve** entries in the span are Social Status Virtues —
`templar_administrator`, `templar_commander`, `templar_confrere_or_consoeur`,
`templar_office_holder`, `templar_servant`, `templar_specialist`,
`town_magistrate`, `troubadour`, `turb_trained`, `university_grammar_teacher`,
`venditor`, `wanderer` — the largest concentration the audit has met. Instances
are noted only, per the brief. **Three of them state the compatibility override
ArMDE:2816 anticipates**, and each is a rule of its own: `templar_administrator`
("can replace the Brother-Knight, Brother-Sergeant, and Brother-Priest Status
Virtues", F-311), `templar_confrere_or_consoeur` ("he may possess other Social
Status Virtues or Flaws", F-314) and `templar_office_holder` ("You may take this
Virtue with any of the Templar Status Virtues", F-315). For the record and
without proposing a fix: `jq` over `rules/core/character_types.json` shows **no**
`virtue_category_caps` entry for `social_status` on any of the four profiles, so
the base rule is enforced nowhere today and all three overrides are currently
inert — the same standing state B08 recorded for `virtue.shadchan`.

**ArMDE:2960-2962 (realm association of every Supernatural Virtue).** Seven
entries in the span are Supernatural — `unaging`, `unbound_tongue`,
`variable_power`, `voice_of_the_land`, `whistle_up_the_wind`,
`wilderness_sense`, `wisdom_from_ignorance` — and **none carries a `realm`
parameter**. Instances noted only. None is one of ArMDE:2960's named exceptions.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.tainted_treasure` | 5097-5108 | OK | OK | OK | **`source.lines` swallows the Knights Templar sidebar and its Paid Rights rule** — F-309 |
| `virtue.templar_administrator` | 5109-5112 | narrative → creation_effect | effects missing (`ability_authorization`, academic); the Status-replacement rule encoded nowhere | the replacement, the male-only restriction and the no-extra-time clause in neither locale | F-310 F-311 ArMDE:2816 **Q-72** |
| `virtue.templar_commander` | 5113-5116 | OK | **the Reputation of level 3 carried by no effect** (the `grants_selection` half is ✓ and recorded) | the male-only restriction, the tax/judge rights, the crusade obligation and the grand-master vote in neither locale | F-312 F-313 ArMDE:2816 |
| `virtue.templar_confrere_or_consoeur` | 5117-5120 | narrative → uncomputed_rule | the Social-Status compatibility override encoded nowhere | the override and the fewer-rights clause in neither locale | F-314 ArMDE:2816 |
| `virtue.templar_office_holder` | 5121-5124 | OK | the compatibility override (twice stated) encoded nowhere | the override, the Temporal Influence compatibility and the exalted-rank clause in neither locale | F-315 ArMDE:2816 |
| `virtue.templar_prestige` | 5125-5128 | narrative → creation_effect | **effects missing (`grants_reputation`, score 4)** | OK | F-316 **Q-73** |
| `virtue.templar_servant` | 5129-5132 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.templar_specialist` | 5133-5136 | narrative → uncomputed_rule | the player-chosen gated Ability group encoded nowhere (D3) | the group choice and the papal-bull exemption in neither locale | F-317 ArMDE:2816 |
| `virtue.temporal_influence` | 5137-5140 | narrative → creation_effect | **"Grogs may not take this Virtue" enforced nowhere**, although `forbidden_traits` is the mechanism the magus profile already uses | the grog exclusion in neither locale | F-318 |
| `virtue.tethered_magic` | 5141-5144 | narrative → uncomputed_rule | OK | the spell-control transfer, the object tether and the Arcane-Connection side effect in neither locale | F-319 |
| `virtue.tough` | 5145-5147 | OK | OK | **the DE summary calls Soak "Widerstandsfähigkeit" against both the canonical table and the DE rulebook** | F-320 |
| `virtue.town_magistrate` | 5149-5152 | narrative → creation_effect | **the Ability-3 prerequisite absent though exactly expressible**; `ability_authorization` (academic) missing | the prerequisite, the citizenship rule, the wage and the two-season obligation in neither locale | F-321 F-322 F-323 ArMDE:2816 |
| `virtue.trained_assassin` | 5153-5156 | OK | **the Nizari Social-Status prerequisite absent though exactly expressible** | OK | F-324 |
| `virtue.troubadour` | 5157-5164 | narrative → creation_effect | `ability_authorization` (academic) missing | the Academic permission and the reputation expectation in neither locale | F-326 ArMDE:2816 |
| `virtue.troupe_upbringing` | 5165-5168 | narrative → in_play_effect (or uncomputed_rule) | no parameter for the selected area | "+2" in neither locale | F-327 F-328 |
| `virtue.true_faith` | 5169-5172 | OK | **Magic Resistance = Score × 10 computed nowhere**; the granted Faith Point not expressible | the MR formula, the Faith Point, the spend rule and the dawn refresh in neither locale | F-329 F-330 **Q-74** |
| `virtue.true_love_pc` | 5173-5178 | narrative → uncomputed_rule | the reciprocity rule encoded nowhere (D3) | "+3", the "never in excess of +3" cap, the melancholy penalty, the reciprocity and the True Friend rename in neither locale | F-333 F-334 |
| `virtue.turb_trained` | 5179-5182 | narrative → creation_effect | `ability_authorization` (martial) missing; the dead-language permission not instance-expressible; **`incompatible_with` Wealthy/Poor missing on all sides** | the martial permission, the one dead language and the Wealthy/Poor exclusion in neither locale | F-325 F-340 ArMDE:2816 |
| `virtue.unaffected_by_the_gift` | 5183-5186 | narrative → uncomputed_rule | OK | the Gift/Magical Air immunity in neither locale | F-335 F-345 |
| `virtue.unaging` | 5187-5190 | OK | OK — both `aging_mod` tags correct and recorded (`RULES.md:7033`) | the non-fatal-crisis clause and the Decrepitude-4 enfeeblement waiver in neither locale | F-332 F-345 **Q-78** |
| `virtue.unbound_tongue` | 5191-5194 | narrative → uncomputed_rule | OK | the waiver of the Words-and-Gestures penalty in neither locale | F-331 |
| `virtue.university_grammar_teacher` | 5195-5198 | narrative → creation_effect | `ability_authorization` (Artes Liberales + the Latin instance) missing | the two named Abilities, the Teaching expectation and the two-season obligation in neither locale | F-323 F-346 ArMDE:2816 |
| `virtue.variable_power` | 5199-5206 | narrative → uncomputed_rule | OK — the per-power multiplicity is correct and recorded (`RULES.md:2417-2448`) | the four variables, the five-levels-per-variance scaling and the worked 50-year example in neither locale | F-336 |
| `virtue.venditor` | 5207-5210 | OK | **`ability_authorization` (academic) missing** | the Academic permission in neither locale | F-347 ArMDE:2816 |
| `virtue.venus_blessing` | 5211-5214 | narrative → uncomputed_rule | OK | "+3 on Communication and Presence rolls" in neither locale | F-333 |
| `virtue.verditius_magic` | 5215-5217 | narrative → creation_effect | OK — the House prereq and the House grant are both encoded ✓ | the whole Verditius Magic Outer Mystery behind "(see page 240)" in neither locale | F-337 F-338 |
| `virtue.voice_of_the_land` | 5219-5222 | OK **?** | **`max_total` absent, so a once-only Virtue is unlimited, one copy per land (ArMDE:2814)** | OK **?** | **F-348** (sub-agent); **Q-76 Q-77**; ArMDE:2960 instance |
| `virtue.wanderer` | 5223-5226 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.warrior` | 5227-5229 | OK | OK | OK | clean — recorded at `RULES.md:2784-2788` |
| `virtue.ways_of_the_land` | 5231-5234 | OK | **the +3 to combat and to "all rolls" carried by no effect**; conditional +3 folded flat (recorded at `RULES.md:5131`); the botch-die reduction carried by nothing (D3) | OK — the only entry in the batch whose `description` carries its whole passage, in both locales | F-344 |
| `virtue.wealthy` | 5235-5238 | OK | **the companion-only rule is enforced for magus and mythic companion but not for grog**; `incompatible_with` empty against eight stated exclusions | the three free seasons and the not-supported-by-the-covenant clause in neither locale | F-339 F-340 **Q-75** |
| `virtue.well_traveled` | 5239-5242 | narrative → creation_effect | **the fifty experience points carried by no effect**, although all eight targets *are* expressible | the fifty points and the eight targets in neither locale | F-341 F-342 |
| `virtue.whistle_up_the_wind` | 5243-5246 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.wilderness_sense` | 5247-5250 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.wisdom_from_ignorance` | 5251-5256 | narrative → uncomputed_rule | OK | the unreadable-book study rule and the Ability substitution in neither locale | F-343 |

**Totals: 30 entries carry at least one finding, 5 do not** —
`virtue.templar_servant`, `virtue.wanderer`, `virtue.warrior`,
`virtue.whistle_up_the_wind`, `virtue.wilderness_sense`. 30 + 5 = 35 ✓.

All five of those were **independently re-derived by the verification sub-agent
and corroborated** (see the reconciliation section); none is clean merely
because the first pass said so.

`virtue.voice_of_the_land` is counted among the 30: it carries **F-348**, found
by the sub-agent. It is also still **marked `?`** on the classification axis and
carries Q-76 and Q-77, so per the audit's rules it is **not marked checked** —
its classification could not be settled from the source and is escalated rather
than decided, independently of the multiplicity defect.

## Findings

**A qualifier that applies to all six `ability_authorization` findings — seven
entries, since F-323 covers two (F-310, F-317, F-323, F-325, F-326, F-347) —
stated once rather than seven times.** A14 records that
`validation/authorization.rs::validate_ability_authorization` errors
(`ability_category_requires_virtue`) on a held Ability whose category is in
`ruleset.categories_requiring_virtue()` unless its id or its category is
authorized, with a whole-character exemption for a profile whose `is_magus` is
true. That gated set is `rules/core/abilities.json` →
`"categories_requiring_virtue": ["academic", "arcane", "martial"]` (read
directly, this run). Every carrier below is a **mundane** Virtue — a Templar
administrator, a Templar specialist, a town magistrate, a troubadour, a turb
grog, a grammar teacher, a selling-agent — so the `is_magus` exemption is empty
in practice and the refusal is exactly what bites. A7 also records that a
`restricted_ability_xp` pool confers the same permission for what it funds,
which is how `virtue.warrior` and `virtue.trained_assassin` escape an
authorization finding (their pools name `categories: ["martial"]`, so the
permission arrives with the funding) while `virtue.venditor` does not (its six
pool ids are all `general`, an ungated category, so the pool authorizes nothing
the character did not already have and the passage's separate Academic
permission is carried by nothing).

**`crates/arm-rules/RULES.md` confirms these are genuine gaps and not settled
decisions, and it names the fix path itself.** Its authorization section
(`RULES.md:6350-6365`) lists exactly one explicitly-wired Virtue —
`flaw.covenant_upbringing` — plus the four covered incidentally by pools, and
then says, verbatim: **"Other access-granting Virtues are a data addition, never
a code change."** None of this batch's seven carriers appears in that section
(each id was searched in RULES.md individually, this run).

**A second qualifier, for every `narrative → …` move below.** B1 records that
`classification` is read by no production code, so none of these moves changes a
computed number. The cost is the one `ecb5150` names: a `narrative` entry is not
obliged to carry its rule in `description`, so the rule leaves the application
silently while the entry still looks complete. Every move therefore carries the
same correction — reclassify **and** write the rule into `description` in
**both** locales, or
`uncomputed_clauses.rs::every_uncomputed_rule_entry_states_its_rule_in_every_locale`
goes red. Severity for a bare reclassification: **lost-rule / provenance**, not
miscalculation.

---

### F-309 — `virtue.tainted_treasure` — `source.lines` swallows the Knights Templar sidebar, and the rule inside it belongs to nobody

**Passage** (ArMDE:5105-5107, verbatim — the whole sidebar):
> \> #### The Knights Templar
> \>
> \> The Poor Knights of the Temple of Solomon, the Knights Templar, are an extremely important and influential order in Mythic Europe. **Women cannot pay a fine, with the Paid Rights Virtue, to take a role in the order that is restricted to men.** For full details, see Chapter 7 of The Church.

German, ArMDE:5105-5107 (line-parallel):
> \> #### Die Tempelritter
> \>
> \> Die Armen Ritter Christi und des Tempels zu Salomon, die Tempelritter, sind ein äußerst wichtiger und einflussreicher Orden im Mythischen Europa. **Frauen können sich nicht durch die Tugend Erkaufte Rechte von der Zahlung einer Buße freikaufen, um eine Rolle im Orden zu übernehmen, die Männern vorbehalten ist.** Ausführliche Einzelheiten finden sich in Kapitel 7 von *Die Kirche*.

**Current data.** `virtue.tainted_treasure` — `source.lines: [5097, 5108]`. The
entry's own text ends at ArMDE:5103; 5104 is blank; **5105-5107 is the sidebar**;
5108 is blank; the next `####` heading is 5109.

**Why wrong.** The range mechanically follows the batch's dominant convention
("own heading → the line before the next heading"), and that convention is
correct everywhere else in the span — but here it produces a range containing
three lines of a **different topic**. The consequence is not cosmetic. The
sidebar's second sentence is a **rule**: Paid Rights, a Virtue that elsewhere
lets a woman buy into a male-only role, is explicitly *disabled* for the Templar
order. That rule governs the eight Templar entries at ArMDE:5109-5136, and the
provenance now attributes it to a cursed-treasure Virtue eight entries earlier.
Nothing in the catalogue points at it: `jq` over the eight Templar entries shows
no `source` range reaching 5105-5107, and none of their `summary` fields in
either locale mentions Paid Rights.

This is the same class of rule B08 found at ArMDE:4912 for
`virtue.senior_clergy` ("The Paid Rights Virtue does **not** enable women to take
any of the other positions", its F-265) — there stated *inside* the entry, here
stated in a sidebar no entry claims.

**The contrast that makes this a defect rather than a convention quibble.** The
span immediately before this one contains the *other* kind of block-quote
sidebar: `> #### Study Bonus Examples` at ArMDE:5060, which sits **inside**
`virtue.study_bonus`'s own range (5056-5072) and genuinely belongs to that
entry — B08 recorded it as "correctly claimed by nobody in its own right". The
Knights Templar sidebar is the opposite case: it sits **between** two entries and
its content belongs to the eight that *follow* it, not to the one whose range
swallows it. A `grep -an "^#### \|^> #### "` over the source shows both sidebars
and no third in the neighbourhood, so the distinction is exactly this pair.

**Correct value.** Narrow `virtue.tainted_treasure`'s range to `[5097, 5104]`
(or `[5097, 5103]`, the tighter convention three other entries in this batch
use). The sidebar itself is not an entry and gets no `source` of its own; its
rule must instead be written into `description` — in both locales — on the
Templar Status entries it constrains, at minimum
`virtue.templar_administrator` and `virtue.templar_commander`, the two that
carry an explicit male-only restriction (F-311, F-313).

**Severity.** Lost rule plus a provenance defect. The Paid Rights interaction
reaches no user at all, and the range misattributes three lines.

---

### F-310 — `virtue.templar_administrator` — a gated-category permission with no `ability_authorization`, and `narrative` denying it

**Passage** (ArMDE:5111, verbatim, the operative sentences):
> He may have considerable influence and access to enormous resources, but no additional time. **You may take Academic Abilities during character creation.** This Virtue can replace the Brother-Knight, Brother-Sergeant, and Brother-Priest Status Virtues. This Virtue is only available to male characters.

German, ArMDE:5111 (line-parallel):
> Er kann erheblichen Einfluss haben und Zugang zu enormen Ressourcen besitzen, hat jedoch keine zusätzliche freie Zeit. **Du kannst bei der Charaktererschaffung Akademische Fertigkeiten wählen.** Diese Tugend kann die Sozialer-Status-Tugenden Bruder-Ritter, Bruder-Sergeant und Bruder-Priester ersetzen. Diese Tugend steht nur männlichen Charakteren zur Verfügung.

**Current data.** `classification: "narrative"`, **no `effects`**, no
`prerequisites`.

**Why wrong, twice over.** `academic` is in
`rules/core/abilities.json`'s `categories_requiring_virtue`, so a Templar
Administrator who buys Artes Liberales gets
`ability_category_requires_virtue` — a hard **error** — from
`validation/authorization.rs::validate_ability_authorization`. The passage
grants that permission in as many words. And `narrative` asserts the book states
nothing mechanical for this entry, which the same sentence refutes.

**Correct value.** `classification: "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}` — the bare
permission case A14 exists for, since the passage permits without funding.

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-311 — `virtue.templar_administrator` — the Status-replacement rule, the male-only restriction and the no-extra-time clause reach neither locale (D5)

**Passage.** As quoted in F-310, plus ArMDE:5111's opening: "While he has sworn
the Templar oath and vows, and lives a monastic life, his many duties mean he is
likely to never see active combat."

**Current data.** Both `summary` fields stop at the first sentence
("The character is technically a brother-knight or brothersergeant …" /
"Der Charakter ist technisch gesehen ein Bruder-Ritter oder Bruder-Sergeant …").
Neither locale carries a `description`.

**What is left over — three clauses.**

1. **"This Virtue can replace the Brother-Knight, Brother-Sergeant, and
   Brother-Priest Status Virtues."** An ArMDE:2816 interaction: it is the
   *replacement* form of the compatibility override rather than the
   *co-existence* form `virtue.templar_office_holder` states (F-315). Nothing in
   the data expresses "may stand in for"; `incompatible_with` would say the
   opposite of what is meant, and there is no "supersedes" axis. And one of the
   three named targets resolves to nothing — **Q-72**.
2. **"This Virtue is only available to male characters."** D3: `types.rs`'s
   `gender` field is declared "free-text; **no mechanical effect**" and no
   `Prereq` variant reads it.
3. **"but no additional time"** — the seasons-per-year economy, which the app
   does not model at all.

**Correct value.** A `description` in both locales carrying all three, alongside
F-310's reclassification. The sidebar rule of F-309 belongs here too.

**Severity.** Lost rule.

---

### F-312 — `virtue.templar_commander` — the Reputation of level 3 is carried by no effect

**Passage** (ArMDE:5115, verbatim, the operative sentence):
> Because of his high position, he is a wellknown figure and **has a Reputation of level 3 in his area.**

German, ArMDE:5115 (line-parallel):
> Aufgrund seiner hohen Stellung ist er eine allgemein bekannte Person und **besitzt eine Reputation der Stufe 3 in seinem Gebiet.**

**Current data.** `creation_effect`, one effect:
`{"type": "grants_selection", "items": ["virtue.brother_knight", "virtue.temporal_influence"]}`.
That effect is **correct and recorded** — `RULES.md:3024-3040` quotes the same
passage's two grant clauses and names this exact data. There is **no
`grants_reputation`**.

**Why wrong.** ArMDE:5115 states a Reputation in the same paragraph and in the
same shape as `virtue.templar_office_holder`'s "has a Reputation of level 2 in
his region" — which *is* encoded, as
`{"type": "grants_reputation", "kind": "local", "score": 2}`. The two entries sit
eight lines apart and were evidently read by the same pass; one got its grant
and the other did not. Without the grant, `validation/scores.rs::validate_reputations`
raises `reputation_not_granted` — a hard **error** — the moment the player
enters the Reputation the book says he has.

**Independent corroboration from the translation tables.**
`rules/source/de/translation-tables/reputationen.md:86` reads
`| Templar Commander | Templerkommandeur | Lokal/Regional | 3 (+) | Bekannte
Führungspersönlichkeit |` — a hand-curated table that recorded both the type and
the score the data omits.

**Correct value.** Add
`{"type": "grants_reputation", "kind": "local", "score": 3}` alongside the
existing `grants_selection`. ("in his area" is the same scope wording
ArMDE:5123 uses for the `local` grant on Office Holder.)

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-313 — `virtue.templar_commander` — four further clauses reach neither locale (D5)

**Passage** (ArMDE:5115, verbatim, the clauses not covered by F-312):
> He may levy taxes and tithes over the lands he controls as if he were a landed noble or bishop, and charge service fees on monies he lends, and he may even act as a judge for minor lay crimes committed on his lands. … He is expected to support the order's crusading efforts if he is in the West, and if he lives in the East he is expected to participate directly. Should the grand master die and the character lives in the East, he has the right to participate in choosing a new grand master. This Virtue includes the effects of the Brother-Knight Virtue, and **likewise can only be taken by male characters.**

German, ArMDE:5115 (line-parallel):
> Er kann Steuern und Zehnten über die Ländereien erheben, die er kontrolliert, als wäre er ein Landadliger oder Bischof, Servicegebühren für Geldleihen erheben und sogar als Richter für geringfügige Laiendelikte auf seinen Ländereien fungieren. … Diese Tugend beinhaltet die Wirkungen der Tugend Bruder-Ritter und **kann ebenfalls nur von männlichen Charakteren gewählt werden.**

**Current data.** Both `summary` fields stop at the first sentence. Neither
locale carries a `description`.

**What is left over.** The male-only restriction (D3, as F-311), the fiscal and
judicial rights, the crusade obligation and the grand-master election right. Of
these only the first is a hard rule; the rest are story-level entitlements that
nonetheless define what the Major Virtue buys, and the player sees none of them.

**Correct value.** A `description` in both locales.

**Severity.** Lost rule.

---

### F-314 — `virtue.templar_confrere_or_consoeur` — a Social-Status compatibility override the class denies and nothing encodes

**Passage** (ArMDE:5119, verbatim, the operative sentence):
> His membership is generally temporary (although it need not be), and **he may possess other Social Status Virtues or Flaws to reflect his true station, such as Clerk, Knight, Landed Noble, Peasant, or even Hermetic Magus.**

German, ArMDE:5119 (line-parallel):
> Seine Mitgliedschaft ist im Allgemeinen vorübergehend (muss es aber nicht sein), und **er kann andere Sozialer-Status-Tugenden oder -Fehler besitzen, die seinen wahren Stand widerspiegeln, etwa Kleriker, Ritter, Landadliger, Bauer oder sogar Hermetischer Magus.**

**Current data.** `classification: "narrative"`, no effects, no parameters.

**Why wrong.** ArMDE:2816 is the base rule — "All characters must take one
Social Status, and may only take more than one if the descriptions … explicitly
note that they are compatible" — and this sentence is exactly the explicit note
ArMDE:2816 anticipates. A rule that unlocks a restriction is a mechanical
clause, so `narrative` is refuted; it is the same shape B08 rated for
`virtue.shadchan` (its F-267). The clause is also unusually wide: it permits
even `virtue.hermetic_magus`, which is the one Social Status the magus profile
*requires*.

**What the data can and cannot do.** As B08 recorded, no profile carries a
`virtue_category_caps` entry for `social_status` (`jq` over
`rules/core/character_types.json`, re-run this batch: the only cap present is
the magus profile's `hermetic` major cap), so the base rule is unenforced and
the override is inert either way. That makes this a text finding today and a
data finding the moment ArMDE:2816 is enforced.

**Correct value.** `classification: "uncomputed_rule"` with the override written
into `description` in both locales, together with the "fewer rights" and
separate-housing clauses.

**Severity.** Lost rule.

---

### F-315 — `virtue.templar_office_holder` — the compatibility override is stated twice and encoded nowhere, and reaches neither locale (D5)

**Passage** (ArMDE:5123, verbatim, the operative sentences):
> **You may take this Virtue with any of the Templar Status Virtues**, as your character may be a senior brothersergeant or senior Templar chaplain. **This Virtue is compatible with the Temporal Influence Minor Virtue.** If you take this virtue with the Templar Commander Major Virtue, then your character holds one of the few exalted ranks within the Templars, such as grand commander or grand marshal …

German, ArMDE:5123 (line-parallel):
> **Du kannst diese Tugend zusammen mit einer der anderen Tempelritter-Sozialer-Status-Tugenden wählen**, da dein Charakter ein erfahrener Bruder-Sergeant oder erfahrener Tempelritter-Kaplan sein kann. **Diese Tugend ist mit der Kleinen Tugend Zeitlicher Einfluss vereinbar.** Wenn du diese Tugend zusammen mit der Großen Tugend Templerkommandeur wählst, bekleidet dein Charakter einen der wenigen erhabenen Ränge …

**Current data.** `creation_effect` ✓ with
`{"type": "grants_reputation", "kind": "local", "score": 2}` ✓ — correct against
"has a Reputation of level 2 in his region" and recorded at `RULES.md:5401`.
Both `summary` fields stop at the first sentence; neither locale carries a
`description`.

**What is left over — three clauses.** The Templar-Status co-existence override
(ArMDE:2816's shape again, and a *broader* one than F-314's because it names a
whole family), the explicit Temporal Influence compatibility — which is
interesting in its own right, because `virtue.templar_commander` **grants**
Temporal Influence and this entry merely permits it, so a Commander who is also
an Office Holder receives it twice, once free and once bought — and the
exalted-rank consequence.

**Correct value.** Keep the class and the effect; add a `description` in both
locales carrying all three.

**Severity.** Lost rule.

---

### F-316 — `virtue.templar_prestige` — a Reputation of level 4 stated by the book and carried by no effect, under a `narrative` class

**Passage** (ArMDE:5127, verbatim — the entry's whole body):
> A member of the Poor Knights of the Temple of Solomon, the character enjoys great respect and admiration among his fellow brothers. This may be because of a great act of heroism or piety, or because of the rank or station he possessed before he joined the order. **He starts with a Reputation of level 4 within the Templars.**

German, ArMDE:5127 (line-parallel):
> Als Mitglied der Armen Ritter Christi und des Tempels zu Salomon genießt der Charakter große Wertschätzung und Bewunderung unter seinen Ordensbrüdern. … **Er beginnt mit einer Reputation der Stufe 4 innerhalb der Tempelritter.**

**Current data.** `classification: "narrative"`, **no effects**.

**Why wrong.** "Starts with a Reputation of level 4" is a plain digit and a
character-creation grant — the identical sentence shape that *is* encoded on
`virtue.templar_office_holder` four entries earlier and on fourteen other
Virtues across the catalogue. `narrative` asserts the book states nothing
mechanical, which one clause refutes outright, and without a
`grants_reputation` the player who enters the Reputation gets
`reputation_not_granted`.

Note also the magnitude/category shape: this is the batch's one Templar entry
that is `*Minor, General*` rather than a Social Status, which is correct against
ArMDE:5126 ✓ — the Reputation is a *prestige* on top of a Status the character
takes separately.

**Independent corroboration.** `reputationen.md:85` reads
`| Hermetic Prestige (via Templar) | Templer-Ansehen | Templer | 4 (+) | Hohes
Ansehen unter Templern |` — score 4 ✓, and the scope column "Templer", which is
**none** of the engine's four `ReputationType` values. That is **Q-73**.

**Correct value.** `classification: "creation_effect"` plus a
`grants_reputation` of score 4. Which `kind` — `local`, or the `None` wildcard
A25 provides for a player-chosen audience — is **Q-73** and is not decided here.

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-317 — `virtue.templar_specialist` — a player-chosen gated Ability group the engine cannot express, under a `narrative` class

**Passage** (ArMDE:5135, verbatim, the operative sentence):
> **You may take one restricted group of Abilities during character creation, such as Academic or Martial Abilities.**

German, ArMDE:5135 (line-parallel):
> **Du kannst bei der Charaktererschaffung eine eingeschränkte Gruppe von Fertigkeiten wählen, etwa Akademische oder Kampffertigkeiten.**

**Current data.** `classification: "narrative"`, no effects, **no parameters**.

**Why wrong.** The sentence is a creation-time permission for a gated category —
the same rule `virtue.warrior` and `virtue.templar_administrator` state, except
that *which* category is the **player's choice**. `narrative` is refuted.

**Why this is D3 and not a plain `ability_authorization`.** A14's
`AbilityAuthorization` carries a fixed `categories: Vec<AbilityCategory>` and
has **no `param` field** — B8's table of the eleven parameter-relative effect
variants does not include it, so the permission cannot be made
selection-dependent. The `category` parameter domain is no help either: B8
records that it resolves against *the declaring item's own* `categories`, which
here are `["social_status"]`. Listing both `academic` and `martial` in one
`ability_authorization` would be the **over-permission** shape B07 found on
`virtue.privileged_upbringing` and B08 on `virtue.student_of_realm` — it grants
what the passage explicitly rations to one — and A14 records that a permission
which is too wide raises no error, so nothing would catch it.

Note the passage's "**such as**": the two named categories are examples, so even
an enumerated choice would be narrower than the book. The nearest already-shipped
comparison is ArMDE:3631 (`virtue.covenant_specialist`), whose "either Martial,
Academic, or Arcane" is closed — worth checking when that entry's batch runs.

**Correct value.** `classification: "uncomputed_rule"`, with the group choice
written into `description` in both locales, plus the papal-bull exemption and
the Muslim-character clause.

**Severity.** Lost rule, and a permission the app will refuse — but the refusal
cannot be fixed by data alone, which is why it is D3 rather than F-310's shape.

---

### F-318 — `virtue.temporal_influence` — "Grogs may not take this Virtue" is enforced nowhere, although the mechanism exists and is used

**Passage** (ArMDE:5139, verbatim — the entry's whole body):
> Through blood or a position of trust, you enjoy some political weight in society. You have the ear of a leader and may yourself lead common folk at times, if they respect your position. The more influence you have, the more responsibility, and the harder it is to work unopposed with magi. **Grogs may not take this Virtue.**

German, ArMDE:5139 (line-parallel):
> Durch Geburt oder eine Vertrauensstellung genießt du einiges politisches Gewicht in der Gesellschaft. … **Grogs können diese Tugend nicht wählen.**

**Current data.** `classification: "narrative"`, no effects, `categories: ["general"]`,
`entity_kinds: ["character"]`.

**Why wrong.** Two things at once.

1. **`narrative` is refuted** — a hard "may not take" is a mechanical
   restriction, the same shape B08 rated for `virtue.supernatural_beauty`'s
   Presence precondition (its F-305).
2. **The restriction is expressible and is not expressed.** `entity_kinds`
   distinguishes *character* from *covenant* (B4), not grog from companion, so
   it does not carry this. The axis that does is the type profile's
   `forbidden_traits`, and the magus and mythic-companion profiles already use
   it for exactly this purpose (`["flaw.poor", "virtue.wealthy"]`). The grog
   profile's `forbidden_traits` is `["virtue.the_gift"]` and does not name this
   Virtue; its `permitted_categories` includes `general`, so a grog may take
   Temporal Influence today with no issue raised at all.

This also makes `virtue.templar_commander`'s grant reachable by a grog by a
second route (`grants_selection` is budget-exempt and, per B4, not even
entity-kind-checked) — though a grog taking a Major Social Status is already
excluded on points.

**Correct value.** Add `"virtue.temporal_influence"` to the grog profile's
`forbidden_traits` in `rules/core/character_types.json`, and reclassify to
`creation_effect` — C7's "modelled by something other than an effect", exactly
as Devil Child's budget bonus lives on a type profile. If the correction pass
prefers not to touch the profile, the honest class is `uncomputed_rule` with the
exclusion written into `description` in both locales; what is **not** defensible
is leaving it `narrative`.

**Severity.** Wrong rules output — the app permits an illegal character.

---

### F-319 — `virtue.tethered_magic` — a Hermetic spell mechanic classified `narrative`, reaching neither locale

**Passage** (ArMDE:5143, verbatim — the entry's whole body):
> You can pass control of your non-Ritual spells to others, just as if they were the caster, "tethering" the magic to them for the spell's duration. You may also tether a spell to an object, which can then transfer the spell to an appropriate target when it comes into range. This can even be done whenever you activate an effect in a magic item. **However, a side effect of this sort of magic is that all of your spells and the effects of any magic items you activate are Arcane Connections to you.**

German, ArMDE:5143 (line-parallel):
> Du kannst die Kontrolle über deine Nicht-Ritual-Zauber an andere weitergeben … **Ein Nebeneffekt dieser Art von Magie ist jedoch, dass alle deine Zauber und die Effekte aller magischen Artefakte, die du aktivierst, Arkane Verbindungen zu dir sind.**

**Current data.** `classification: "narrative"`, no effects.

**Why wrong.** Three mechanical clauses, none of them colour: control of a
spell transfers to another character (which changes who the spell's caster is
for every subsequent ruling), spells may be tethered to objects with a
range-triggered transfer, and — the sharpest — **every one of the magus's spells
becomes an Arcane Connection to him**, which is a defined game term with
Penetration and targeting consequences. `narrative` asserts the passage states
nothing mechanical; it states a standing drawback that any hostile magus can use.

The scope limiter "non-Ritual" is itself a mechanical restriction on a
`CastingScope`-shaped axis the engine already has (`derived.rs::CastType`), so
this is not even a rule foreign to the model — just one with no carrier.

**Correct value.** `classification: "uncomputed_rule"` with all three clauses in
`description` in both locales. No effect variant fits: there is no
transfer-of-control axis and no Arcane-Connection axis.

**Severity.** Lost rule — and a lost *Flaw-shaped* half of a Virtue, which is
worse than average: the player buys a Minor Hermetic Virtue and is never told it
makes him permanently targetable.

---

### F-320 — `virtue.tough` — the German summary calls Soak "Widerstandsfähigkeit" against both the canonical table and the DE rulebook

**Passage** (ArMDE:5147, verbatim — the entry's whole body):
> You can take physical punishment better than most people. **You get a +3 bonus on your Soak score.**

German, ArMDE:5147 (line-parallel):
> Du kannst körperliche Strapazen besser ertragen als die meisten Menschen. **Du erhältst einen Bonus von +3 auf deinen Absorptionswert.**

**Current data.** `in_play_effect` ✓ with `{"type": "soak_mod", "amount": 3}` ✓ —
correct in sign, size and target, and recorded at `RULES.md:4994`. The English
`summary` is "+3 to Soak." ✓. The German `summary` is
**"+3 auf Widerstandsfähigkeit."**

**Why wrong.** Two independent authorities give a different German term and they
agree with each other:

- `rules/source/de/translation-tables/kampf.md:27` — `| Soak | **Absorption** |
  Rüstungsschutz + Ausdauer |`, with `kampf.md:28` and `magie-regeln.md:51`
  giving `Soak Total | Absorptionssumme`.
- The **DE rulebook itself**, at the line-parallel ArMDE:5147, writes
  "**Absorptionswert**".

So this is not the table-versus-rulebook conflict the audit's rules carve out —
both point the same way, and the data agrees with neither. "Widerstandsfähigkeit"
is a plausible-sounding but non-canonical rendering (it more naturally reads
"resilience"/"resistance", and "Magieresistenz" is already the established German
for Magic Resistance, so the word actively invites confusion with a different
game statistic).

The entry's *name* is fine: `tugenden-fehler.md:217` gives `Tough | Zäh` and the
data says `Zäh` ✓.

**Correct value.** German `summary`: `"+3 auf Absorption."` (or
`"+3 auf den Absorptionswert."`, matching the DE rulebook's own wording). Worth
checking in the same correction whether `flaw.frail` — the other `soak_mod`
carrier, `RULES.md:4994` — and the German `.ftl` Soak label use the canonical
term; that check is outside this batch's span and is not claimed here.

**Severity.** Localization defect — a real user, every session, reading a German
label for a statistic that is named differently everywhere else in the app and
the book.

---

### F-321 — `virtue.town_magistrate` — a Social Status with three mechanical clauses classified `narrative`

**Passage** (ArMDE:5151, verbatim — the entry's whole body):
> The character has a position of judicial responsibility in a town, with a small staff of minor officials (up to five individuals). **The character must be a citizen of the town, have a score of at least 3 in the Civil and Canon Law (or Common Law) Ability**, and is paid a wage or gains special privileges in return for his services. **Being a magistrate occupies the character for two seasons each year**, but he is free for the remaining two seasons. **Academic Abilities may be bought for the character, during character generation.**

German, ArMDE:5151 (line-parallel):
> Der Charakter hat eine richterliche Verantwortungsposition in einer Stadt mit einem kleinen Stab von untergeordneten Beamten (bis zu fünf Personen). **Der Charakter muss Bürger der Stadt sein, einen Wert von mindestens 3 in der Fertigkeit Zivil- und Kanonisches Recht (oder Gewohnheitsrecht) besitzen** … **Das Amt des Richters beansprucht den Charakter für zwei Quartale pro Jahr** … **Akademische Fertigkeiten können bei der Charaktererschaffung für den Charakter erworben werden.**

**Current data.** `classification: "narrative"`, no effects, no prerequisites.
Both `summary` fields stop at the first sentence; neither locale carries a
`description`.

**Why wrong.** An Ability minimum with a number in it is the least ambiguous
mechanical clause there is, and the entry carries two more (the seasons
obligation and the Academic permission). `narrative` is refuted three times over.

**Correct value.** `classification: "creation_effect"` once F-322's prerequisite
and F-323's authorization are added, with the citizenship and two-season clauses
written into `description` in both locales.

**Severity.** Lost rule.

---

### F-322 — `virtue.town_magistrate` — a prerequisite the `Prereq` tree expresses exactly, and that is absent

**Passage** (ArMDE:5151): "The character must … **have a score of at least 3 in
the Civil and Canon Law (or Common Law) Ability**".

**Current data.** `prerequisites` absent.

**Why wrong, and why this one is not D3.** Unlike the age, gender and
cross-character preconditions elsewhere in this batch, this rule maps onto the
`Prereq` enum without any loss at all. B6 lists `ability_min` and `any`; both
target Abilities exist and are unparameterized:
`ability.civil_and_canon_law` (`academic`) and `ability.common_law`
(`academic`), read from `rules/core/abilities.json` this run. B6 also records
that `ability_min` compares against the **effective** score (bought + Puissant),
which is the right reading for "have a score of at least 3".

**Correct value.**

```json
"prerequisites": { "kind": "any", "value": [
  { "kind": "ability_min", "value": { "ability": "ability.civil_and_canon_law", "score": 3 } },
  { "kind": "ability_min", "value": { "ability": "ability.common_law", "score": 3 } }
]}
```

**Severity.** Wrong rules output — the app permits an illegal character, and
this is one of the few cases in the batch where the fix is pure data with no
approximation.

---

### F-323 — `virtue.town_magistrate` and `virtue.university_grammar_teacher` — Academic permissions with no `ability_authorization`

**Passages.** ArMDE:5151 (Town Magistrate): "Academic Abilities may be bought
for the character, during character generation." ArMDE:5197 (University Grammar
Teacher), verbatim:
> The character can be of any age or gender. **They may purchase the Academic Abilities: Latin and Artes Liberales at character generation**, and should have a score in Teaching. **They must teach two seasons out of the year.**

German, ArMDE:5197 (line-parallel):
> Der Charakter kann jeden Alters und Geschlechts sein. **Er kann bei der Charaktererschaffung die Akademischen Fertigkeiten Latein und Artes Liberales erwerben** und sollte einen Wert in der Fertigkeit Unterrichten besitzen. **Er muss zwei Quartale im Jahr unterrichten.**

**Current data.** Both entries: `narrative`, no effects, no parameters.

**Why wrong.** The standard pattern — see the qualifier above. The two differ in
one respect worth stating, because it changes the fix:

- **Town Magistrate** permits the **whole** `academic` category, so
  `{"type": "ability_authorization", "categories": ["academic"]}` is exact.
- **University Grammar Teacher** permits **two named Abilities**, so the
  category form would be over-permission (B07's `privileged_upbringing` shape).
  The id form is right — but only **half** of it is expressible:
  `ability.artes_liberales` is a plain id ✓, while **Latin is an *instance* of
  `ability.dead_language`**, and A7/A14 name an Ability, never one of its
  instances. `RULES.md:2779-2782` records the existing approximation for exactly
  this word: "Latin is modelled as the parameterized `ability.dead_language`, so
  Educated lists `ability.dead_language` + `ability.artes_liberales` (any Dead
  Language qualifies — a deliberate seed approximation of 'Latin')."

**Correct value.** Town Magistrate: the category authorization above.
University Grammar Teacher:
`{"type": "ability_authorization", "abilities": ["ability.artes_liberales", "ability.dead_language"]}`,
following the recorded Educated approximation, with the "Latin specifically"
narrowing written into `description` in both locales along with the Teaching
expectation and the two-season obligation.

**Severity.** Wrong rules output — the app refuses a legal character, twice.

---

### F-324 — `virtue.trained_assassin` — the Nizari Social-Status prerequisite is absent, though `Prereq::Any` expresses it exactly

**Passage** (ArMDE:5155, verbatim — the entry's whole body):
> **This Virtue is only available to characters with one of the Social Status Virtues of the Nizaris.** He has completed several missions on behalf of his masters, giving him **50 additional experience points that may be spent on any Martial Abilities as well as Athletics, Guile, or Stealth.**

German, ArMDE:5155 (line-parallel):
> **Diese Tugend steht nur Charakteren mit einer der Sozialer-Status-Tugenden der Nizaris zur Verfügung.** Er hat mehrere Missionen im Auftrag seiner Meister absolviert und erhält dadurch **50 zusätzliche Erfahrungspunkte, die für beliebige Kampffertigkeiten sowie für Athletik, Täuschung oder Schleichen ausgegeben werden können.**

**Current data.** `creation_effect` ✓, one effect:
`{"type": "restricted_ability_xp", "amount": 50, "abilities": ["ability.athletics", "ability.guile", "ability.stealth"], "categories": ["martial"]}`.
The XP effect is **exactly right**: A7's eligibility is an **OR** of `abilities`
and `categories`, which is precisely "any Martial as well as Athletics, Guile,
or Stealth"; the three named ids are all `general` and the category covers the
martial half; and A7's permission side-effect authorizes `martial`, so no
separate `ability_authorization` is owed. `prerequisites` is **absent**.

**Why the prerequisite is absent-and-expressible.** The passage names no
Virtue, so the referent had to be found by searching the source: `grep -an
"Nizari"` over the English file returns ArMDE:3879 (`virtue.fidai`, "an assassin
of the Nizari Isma'ilis … Fida'i may take Martial Abilities at character
creation") and ArMDE:4235 (`virtue.lasiq`, "an experienced assassin of the
Nizari İsma'ilis … Lasiq may take Martial Abilities at character creation").
Both ids exist in the catalogue (`virtue.fidai` 3877-3881, `virtue.lasiq`
4233-4236) and both are `social_status`. B6's `has` leaf reads the
**grants-inclusive** present set, so a granted Fida'i would satisfy it too,
which is the right behaviour here.

This is the same shape as B08's F-270/F-283 (Shamash and Sofer requiring
Educated (Hebrew)) except for the decisive difference that **here the target ids
exist**, so it is a plain data omission rather than a D3 gap.

**Correct value.**

```json
"prerequisites": { "kind": "any", "value": [
  { "kind": "has", "value": "virtue.fidai" },
  { "kind": "has", "value": "virtue.lasiq" }
]}
```

A correction pass should confirm that ArMDE:3877-3881 and ArMDE:4233-4236 are
the *only* Nizari Social Statuses in the core catalogue before committing the
closed list; this batch's search found no third.

**Severity.** Wrong rules output — the app permits an illegal character.

---

### F-325 — `virtue.turb_trained` — a Martial permission, a single dead language, and a Wealthy/Poor exclusion, all under `narrative`

**Passage** (ArMDE:5181, verbatim — the entry's whole body):
> This character has grown up at the covenant, been trained to fight in the turb, and picked up a few things from the magi's trusted servants. **At character creation, he is allowed to learn Martial Abilities. He may also learn whichever single dead language the magi speak in addition to his local tongue.** His fortunes are tied to the turb, and **he is thus prohibited from being Wealthy or Poor.**

German, ArMDE:5181 (line-parallel):
> Dieser Charakter ist im Konvent aufgewachsen … **Bei der Charaktererschaffung darf er Kampffertigkeiten erlernen. Er darf außerdem die einzige tote Sprache erlernen, die die Magi sprechen, zusätzlich zu seiner Muttersprache.** Sein Schicksal ist mit der Turba verbunden, weshalb **es ihm verboten ist, Wohlhabend oder Arm zu sein.**

**Current data.** `classification: "narrative"`, no effects, no parameters,
`incompatible_with: []`.

**Three defects in one entry.**

1. **`narrative` is refuted** by any of the three clauses.
2. **Two gated permissions carried by nothing.** `martial` is gated, so the
   Martial permission needs `ability_authorization` (the standard pattern).
   The dead language is gated too — `ability.dead_language` is `academic` — and
   is doubly inexpressible: A7/A14 cannot name one *instance*, and the passage
   additionally caps it at **one** language, a per-instance count no
   authorization or pool field carries (A7 hard-codes `instances: Vec::new()`).
   That half is D3.
3. **The Wealthy/Poor exclusion is in no `incompatible_with`, on any side.**
   See F-340 for the catalogue-wide form of this; `virtue.turb_trained` is one
   of the eight passages that states it.

**Correct value.** `classification: "creation_effect"` (or `uncomputed_rule` if
the correction pass declines the partial modelling), plus
`{"type": "ability_authorization", "categories": ["martial"], "abilities": ["ability.dead_language"]}`,
plus `incompatible_with: ["virtue.wealthy", "flaw.poor"]` with the symmetric
entries on those two, plus a `description` in both locales carrying the
one-language cap.

**Severity.** Wrong rules output in both directions at once — the app refuses a
legal Martial purchase *and* permits an illegal Wealthy Turb-Trained grog.

---

### F-326 — `virtue.troubadour` — an Academic permission with no `ability_authorization`, under `narrative`

**Passage** (ArMDE:5161, verbatim, the operative sentence):
> You are not tied to any community and survive by performing, entertaining, or doing other casual work. **You may take Academic skills during character creation** and should have a reputation regarding the sort of material you produce and your adherence to the tenets of courtly love.

German, ArMDE:5161 (line-parallel):
> Du bist an keine Gemeinschaft gebunden und lebst vom Auftreten, Unterhalten oder sonstigen Gelegenheitsarbeiten. **Du kannst bei der Charaktererschaffung Akademische Fertigkeiten wählen** und solltest eine Reputation hinsichtlich der Art des Materials, das du produzierst, sowie deiner Treue zu den Grundsätzen der Courtoisie besitzen.

**Current data.** `classification: "narrative"`, no effects. Both `summary`
fields stop at the first sentence of the *first* paragraph ("Troubadours are
wandering minstrels and poets …"), so neither the Academic permission nor the
reputation expectation reaches the user.

**Why wrong.** The standard pattern — `academic` is gated, the permission is
carried by nothing, and `narrative` denies that the passage states it.

**A note on the third paragraph, which is *not* a finding.** ArMDE:5163 lists
seven Virtues Troubadour characters "often possess" (Famous, Free Expression,
Inspirational, Puissant Performance, Social Contacts, True Love, Well-Traveled).
All seven resolve in the catalogue, and the sentence is an explicit *suggestion*
("often possess"), not a grant — unlike ArMDE:5115's "also grants", which is
encoded. No `grants_selection` is owed.

**Correct value.** `classification: "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}`, with the
reputation expectation and the Wealthy-supporter clause in `description` in both
locales.

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-327 — `virtue.troupe_upbringing` — a "+2 modifier" under a `narrative` class

**Passage** (ArMDE:5167, verbatim — the entry's whole body):
> You were raised among a group of entertainers, spending much of your childhood traveling from town to town. **Rolls involving Abilities in a selected area (which you should have approved by the storyguide) receive a +2 modifier.** Examples might include tumbling and acrobatics, knife throwing and juggling, or storytelling and acting.

German, ArMDE:5167 (line-parallel):
> Du wurdest in einer Gruppe von Unterhaltungskünstlern aufgezogen … **Würfe auf Fertigkeiten in einem ausgewählten Bereich (der vom Spielleiter genehmigt werden sollte) erhalten einen Bonus von +2.** Beispiele wären Akrobatik und Bodenturnen, Messerwurf und Jonglieren, oder Geschichtenerzählen und Schauspielerei.

**Current data.** `classification: "narrative"`, no effects.

**Why wrong.** A signed integer in the passage body. B1 is explicit that "an
entry whose text states a signed modifier … is **not** narrative even when the
engine computes nothing", and neither `summary` carries the number, so the
player is told the Virtue exists and never told what it does.

**The shaped variant exists.** A41's `AbilityRollMod` is the exact carrier for
"a modifier on rolls using certain Abilities", and it is parameter-relative with
a `text` domain (B8's table) — which fits "a selected area … approved by the
storyguide" better than any enumerated list would. C1 records that
`ability_roll_mod` is **surfaced-only by design** (1 shipped use), and that gap
is *not* re-reported here: surfacing "+2" beside a named area is exactly what
the player needs and is a strict improvement on surfacing nothing.

**Correct value.** `classification: "in_play_effect"` with
`{"type": "ability_roll_mod", "param": "area", "amount": 2}` and the parameter
of F-328 — or, if the correction pass prefers not to add an effect,
`uncomputed_rule` with the +2 written into `description` in both locales. What
is not defensible is `narrative`.

**Severity.** Lost rule.

---

### F-328 — `virtue.troupe_upbringing` — no parameter records which area was selected

**Passage.** ArMDE:5167, as above: "Abilities in **a selected area** (which you
should have approved by the storyguide)".

**Current data.** `parameters` absent; `max_per_target` at its default of 1.

**Why wrong.** The Virtue's entire mechanical content is scoped to a
player-chosen, storyguide-approved subject that the save records nowhere. The
character sheet therefore cannot say *what* the +2 applies to, and a Markdown
export prints the Virtue with no subject. This is the same complaint B08 raised
as F-287 against `virtue.special_circumstances`, and the same shape B08 recorded
as a deferral for `virtue.social_contacts` (`RULES.md:2509-2517`, a
save-compatibility cost) — so a correction pass should check whether that
deferral is meant to cover this entry too before adding the parameter.

**Correct value.** `"parameters": [{ "key": "area", "type": "ref", "domain": "text" }]`
— the free-text domain, as `virtue.ways_of_the_land` and
`virtue.voice_of_the_land` already use for the same "player names the subject"
shape two entries later in this very batch.

**Severity.** Lost rule / data-fidelity: the choice the Virtue is *about* is not
stored.

---

### F-329 — `virtue.true_faith` — the book gives a Magic Resistance formula and the engine computes nothing from the score

**Passage** (ArMDE:5171, the entry's whole body, verbatim):
> Through piety and holy devotion you have faith that can move mountains. **You have a True Faith score of 1 and can gain more.** For more about True Faith, see page 419.

**Behind the pointer** (ArMDE:17611 and :17613, verbatim):
> A character with Faith Points but no True Faith Score does not benefit from Magic Resistance. **A character with a True Faith Score gains Magic Resistance equal to this score multiplied by ten.**
>
> **TRUE FAITH MAGIC RESISTANCE: True Faith Score X 10**

German, ArMDE:5171 (line-parallel):
> Durch Frömmigkeit und heilige Hingabe hast du einen Glauben, der Berge versetzt. **Du hast einen Wert von 1 im Wahren Glauben und kannst weitere erlangen.** Mehr über den Wahren Glauben findest du auf [Seite 419](#wahrer-glaube-1).

**Current data.** `creation_effect` ✓, one effect:
`{"type": "true_faith_grant", "score": 1}` ✓ — the score is right, and it is
recorded at `RULES.md:3582-3590`.

**What no effect implements.** The resistance. A21 records that `true_faith` is
read by `effective/might.rs::true_faith` and surfaced as
`EffectiveScores::true_faith_score`, and that "no total anywhere adds it". I
confirmed that independently this run rather than inheriting it.
`grep -rn "true_faith" crates/arm-rules/src crates/arm-app/src ui/src` returns
**17 lines in 6 files**. Five of them are test code (`effective.rs:1009`,
`:1012`, `:1779`, `:1783`, `:1785` and
`MythicCompanionTypeSelector.test.ts:11`). The remaining non-test lines are,
exhaustively: a comment and a re-export in `lib.rs` (`:70`, `:88`); the
computation itself, `effective/might.rs::true_faith` (`:46`); five lines in
`crates/arm-app/src/ruleset_io.rs` (`:29` import, `:216` and `:620` DTO fields,
`:628` the call, `:742` a legacy-totals copy); two type declarations in
`ui/src/lib/types.ts` (`:125`, `:660`); and one read-only display binding,
`ui/src/lib/components/CharacterDetails.svelte:28`. **Not one of them is a
total.** And
`derived/casting.rs::magic_resistance` was read in full: its `addends` are
exactly `[form_bonus, max(might, parma_for_form)]`. **There is no True Faith
term**, so a True Faith 1 character's sheet prints a Magic Resistance that
ignores a flat 10 the book grants him in every Form.

**Independent corroboration from the translation tables.**
`rules/source/de/translation-tables/sphären-mächte.md:358` reads
`| True Faith Score | Wahrer-Glauben-Wert | Wert des Wahren Glaubens; bestimmt
max. Glaubenspunkte pro Einsatz und **Magieresistenz (× 10)** |` — the
hand-curated glossary records the multiplier the engine drops.

**Why this is not C3c.** C3c says True Faith "produces a number nothing
consumes" and calls that "a scope boundary, not a defect". That framing was made
without the ArMDE:17611 passage in hand: it is not that the rules leave True
Faith unmodelled, it is that the rules give a one-line formula onto a grid the
engine already computes per Form, next to a `might` term that works the same way
(A26: "Might and Parma do **not** stack; the higher wins"). Whether True Faith
joins that `max` or is additive is **Q-74**.

**Correct value.** Not decided here — see Q-74. At minimum, D5 obliges the
formula to be written into `description` in both locales (F-330). The
computation is a code change in `derived/casting.rs::magic_resistance`, not a
data change.

**Severity.** Wrong rules output — a Major Virtue's headline benefit is missing
from a printed total. The highest-severity class CLAUDE.md names.

---

### F-330 — `virtue.true_faith` — the granted Faith Point, the spend rule and the refresh rule reach neither locale (D5, D3)

**Passage** (ArMDE:17607, :17609, :17615, verbatim):
> True Faith, like Confidence, has a Score and Points. **Taking the True Faith Major Virtue grants you a True Faith Score of 1, and a single Faith Point.** Only by possessing the True Faith Major Virtue may a character have a True Faith score. …
>
> You may spend Faith Points like Confidence Points (**and may spend as many Faith Points at once as your True Faith Score**), as long as you are acting in accordance with God's will. …
>
> **Each dawn, you regain a number of Faith Points up to your True Faith Score**, although if you already have more Faith Points than your True Faith Score, you do not gain additional Points.

**Current data.** Both `summary` fields are the first sentence of ArMDE:5171
only. Neither locale carries a `description`.

**What is left over — four clauses.**

1. **The single Faith Point.** D3: `TrueFaithGrant` (A21) has exactly one field,
   `score`. There is no Faith-Point field on the effect and no Faith-Point
   column on `Entity` — unlike Confidence, which A15 carries as a
   `(score, points)` pair. The asymmetry is worth naming: the engine models
   Confidence Points and not Faith Points, although ArMDE:17607 introduces the
   latter by analogy with the former.
2. **"and can gain more"** (ArMDE:5171) — the score is not fixed at 1.
3. **The spend rule** (as many Points at once as the Score).
4. **The dawn refresh.**

**A cross-reference verdict that lands outside this batch's span, recorded here
rather than deferred**, per the brief. ArMDE:17607's second sentence is
**"Only by possessing the True Faith Major Virtue may a character have a True
Faith score."** Three catalogue entries carry `true_faith_grant`
(A21: `virtue.true_faith`, `virtue.relic`, `virtue.powerful_relic`), and
`effective/might.rs::true_faith` sums all three onto the **character**. So a
character with a Relic and without this Virtue is given a character-level True
Faith score that ArMDE:17607 forbids outright — which is the same defect B07
raised against `virtue.relic` from ArMDE:17623 ("All relics **contain** a True
Faith score"), reached here by a different route and from a different sentence.
This batch did not re-derive `virtue.relic`'s own data, so it claims only the
corroboration: **ArMDE:17607 is a second, independent citation for that finding,
and it is the stronger one**, because it states the prohibition directly rather
than requiring an inference about whose score it is.

**Correct value.** A `description` in both locales carrying all four clauses.
The Faith Point itself cannot be modelled without a new field and is not
proposed here.

**Severity.** Lost rule.

---

### F-331 — `virtue.unbound_tongue` — the waiver of a -10 casting penalty, classified `narrative`

**Passage** (ArMDE:5193, verbatim — the entry's whole body):
> Whenever the character is transformed into a non-human form (whether by spell, magic item, curse, or a heartbeast) he may speak any human languages he knows with no impediment. **If he is a magus, he may use his voice as normal to cast spells.**

German, ArMDE:5193 (line-parallel):
> Wann immer der Charakter in eine nicht-menschliche Form verwandelt wird (ob durch Zauber, magisches Artefakt, Fluch oder ein Herztier), kann er alle menschlichen Sprachen, die er kennt, ohne Einschränkung sprechen. **Ist er ein Magus, kann er seine Stimme wie gewohnt zum Zaubern einsetzen.**

**Current data.** `classification: "narrative"`, no effects.

**Why wrong, with the number named.** "He may use his voice as normal to cast
spells" is a waiver of a penalty the core rules give in a table.
ArMDE:9240-9245:

| Words | Modifier |
|---|---|
| Loud | +1 |
| Firm | 0 |
| Quiet | -5 |
| **None** | **-10** |

and ArMDE:9249: "unless the caster can speak in a loud voice and make exaggerated
gestures, **they cannot cast a Ritual spell**". A magus in animal form who cannot
speak is at Words: None, i.e. **-10** to his Casting Score and barred from
Rituals; this Virtue removes both. That is as mechanical as a clause gets, so
`narrative` — which asserts the passage states nothing mechanical — is refuted.
Neither `summary` carries the consequence either: both stop after the
languages sentence in English and carry only the first sentence in German.

The Virtue is also squarely aimed at `virtue.heartbeast` (the passage names the
heartbeast explicitly), which makes it a Bjornaer-facing Hermetic mechanic
wearing a Supernatural label.

**Correct value.** `classification: "uncomputed_rule"`, with the waiver written
into `description` in both locales, naming what it waives. A
`special_casting_mod` would be the wrong carrier: C1 records that eight of the
eleven `SpecialCasting` kinds are surfaced with `amount: 0`, and none of them
means "the Words penalty does not apply".

**Severity.** Lost rule.

---

### F-332 — `virtue.unaging` — the crisis clause and the Decrepitude-4 waiver reach neither locale (D5, D3)

**Passage** (ArMDE:5189, verbatim — the entry's whole body):
> You do not suffer the effects of age. In game terms, your aging points do not decrease your Characteristics, only building up to give you Decrepitude points. **If a crisis is not potentially fatal, you suffer no ill-effects.** You may die from terminal and potentially fatal crises, according to the normal rules. **You are not enfeebled when you reach four Decrepitude points, but you die as normal when you reach five.** You may choose your apparent age freely, although if you are basically human it should be less than or equal to your actual age.

German, ArMDE:5189 (line-parallel):
> Du leidest nicht unter den Auswirkungen des Alterns. … **Wenn eine Krise nicht potenziell tödlich ist, erleidest du keine nachteiligen Folgen.** … **Du wirst nicht hinfällig, wenn du vier Gebrechlichkeitspunkte erreichst, stirbst aber wie üblich, wenn du fünf erreichst.** …

**Current data.** `in_play_effect` ✓ with **two** effects,
`aging_mod {kind: "no_aging", amount: 0}` and
`aging_mod {kind: "no_apparent_aging", amount: 0}`. Both are **correct and
well-guarded**: `RULES.md:7033` tabulates this entry as the one carrier of both
tags, and `aging.rs`'s own test
`unaging_accrues_decrepitude_without_dropping_a_characteristic` pins exactly the
two halves of ArMDE:5189's second sentence — Characteristics spared, Decrepitude
still accruing. Neither locale carries a `description`; both `summary` fields
are the first sentence only.

**What no effect implements — two clauses, both about the *back half* of aging
that the engine now does model.**

1. **"If a crisis is not potentially fatal, you suffer no ill-effects."**
   The crisis engine is live (M6/6b7 — `aging.rs::crisis_total`,
   `::crisis_survival`, `::crisis_preview`), and `AgingEffect::CrisisSurvival`
   exists as a *modifier to the survival roll*. There is no kind meaning
   "non-fatal outcomes are voided", so the Virtue's crisis half is inert. This
   is Q-78.
2. **"You are not enfeebled when you reach four Decrepitude points."** Searched
   with `grep -rani "enfeebl"` (the `-a` form, so a NUL-bearing file could not be
   silently skipped) over `crates/arm-rules/src`, `crates/arm-app/src` and
   `ui/src`: **no hit at all**. The catalogue does carry `flaw.enfeebled`
   (ArMDE:6008-6011), so the Flaw the threshold confers exists as an id while the
   threshold itself is modelled nowhere — and this Virtue's waiver of it
   therefore has nothing to waive.

**Correct value.** Keep the class and both effects; add a `description` in both
locales carrying both clauses and the "die as normal at five" rider.

**Severity.** Lost rule. Note the asymmetry the player sees: the sheet does
silently apply the two halves the engine models, so the entry *looks* complete,
which is exactly the failure mode `ecb5150` describes.

---

### F-333 — `virtue.true_love_pc` and `virtue.venus_blessing` — two "+3" Virtues classified `narrative`

**Passage** (ArMDE:5175, `virtue.true_love_pc`, verbatim, the operative
sentences):
> Thus, **you may add +3 to appropriate Personality Trait rolls**, and add additional bonuses as allowed by the storyguide (**never in excess of +3**) to activities that will return you to your love, or save her life. … If you do not, **you may suffer penalties to most actions requiring spirit due to melancholy.**

**Passage** (ArMDE:5213, `virtue.venus_blessing`, verbatim — the entry's whole
body):
> People are often attracted to you. **You get +3 on Communication and Presence rolls with sexually compatible characters in appropriate situations.** At times you can put this to good use. At other times it's an annoyance.

German, ArMDE:5175 / :5213 (line-parallel):
> Du kannst daher **+3 auf passende Persönlichkeitseigenschaft-Würfe** addieren und zusätzliche Boni, die der Spielleiter erlaubt (**niemals mehr als +3 insgesamt**) …
>
> Menschen fühlen sich oft zu dir hingezogen. **Du erhältst +3 auf Kommunikations- und Präsenz-Würfe mit Charakteren, die sexuell kompatibel sind, in passenden Situationen.**

**Current data.** Both `narrative`, no effects. All four `summary` fields
(two entries × two locales) are the first sentence only, so **neither locale
carries either number**.

**Why wrong.** Two plain signed integers, and in True Love's case a stated
**cap** on top ("never in excess of +3"), which B1 lists alongside the signed
modifier as the exact thing `narrative` may not be used for. Both bonuses are
circumstantial (a Personality Trait roll; a roll against a sexually compatible
character in an appropriate situation), which is what makes them
`uncomputed_rule` rather than an effect — the condition is a scene fact, the
category `uncomputed_rule` exists for.

**Correct value.** `classification: "uncomputed_rule"` on both, with the numbers
and their conditions written into `description` in both locales. For True Love
the melancholy penalty belongs there too.

**Severity.** Lost rule, twice.

---

### F-334 — `virtue.true_love_pc` — the reciprocity requirement is encoded nowhere and cannot be (D3)

**Passage** (ArMDE:5175, verbatim, the last two sentences):
> **Your True Love is another player character, who must also have this Virtue. True Love is never one-sided.**

German, ArMDE:5175 (line-parallel):
> **Deine Wahre Liebe ist ein anderer Spielercharakter, der ebenfalls diese Tugend besitzen muss. Wahre Liebe ist niemals einseitig.**

**Current data.** No prerequisites, no parameters, no effects.

**Why it is D3.** The requirement is not about this character's own sheet at
all: it constrains a *second* entity. B6's eight `Prereq` variants all evaluate
against the entity under validation (`present_ids`, its Abilities, its Arts, its
House, its profile) and none can name another character; the save format holds
one entity, and nothing in `types.rs::Entity` references another. There is no
approximation available either — a self-referential `Has` would be trivially
satisfied.

**A second clause in the same entry that is also unmodelled but *is* worth
recording:** ArMDE:5177, "This Virtue may be renamed 'True Friend' to cover
equally close attachments which are not romantic." That is a rename rule, and
the catalogue has the machinery for it — `name_unfilled` plus a free-text
parameter — but neither locale mentions it, so the player never learns the
Virtue is dual-purpose. Note also the id's `_pc` suffix and the existence of
`flaw.true_love_major` / `flaw.true_love_minor` (`RULES.md:4839-4840`), which
makes this a three-entry family where only this member states the reciprocity.

**Correct value.** `description` in both locales carrying the reciprocity
requirement and the True Friend rename, alongside F-333's reclassification.

**Severity.** Lost rule.

---

### F-335 — `virtue.unaffected_by_the_gift` — immunity to a numeric social penalty, classified `narrative`

**Passage** (ArMDE:5185, verbatim — the entry's whole body):
> **The character is not affected by the negative effects of The Gift or Magical Air in others.** Even a Blatant Gift does not especially bother the character.

German, ArMDE:5185 (line-parallel):
> **Der Charakter ist von den negativen Auswirkungen der Gabe oder des Magischen Flairs anderer nicht betroffen.** Selbst eine Auffällige Gabe stört den Charakter nicht sonderlich.

**Current data.** `classification: "narrative"`, no effects.

**Why wrong, with the numbers named.** "The negative effects of The Gift" is not
a mood; it is a defined penalty the core rules quantify. ArMDE:2870: "The
character suffers **all the penalties of The Gift**, just as magi do (see page
203)." ArMDE:5713 (`flaw.blatant_gift`): "You suffer a **–6 penalty on all
interaction rolls** with normal people and animals." ArMDE:6528: beings react
"as if he had The Gift, which … gives him a **–3 to all social interaction
rolls**". This Virtue switches those off for its bearer, so the passage plainly
states a mechanical clause and `narrative` is refuted.

The direction is what makes it unmodellable rather than merely uncomputed: the
penalty belongs to the *other* character's roll, and the engine holds one entity
(the same structural reason as F-334). So `uncomputed_rule` is the honest class
and D3 is the reason.

**Correct value.** `classification: "uncomputed_rule"` with the immunity — and
the Blatant Gift rider — written into `description` in both locales.

**Severity.** Lost rule.

---

### F-336 — `virtue.variable_power` — a scaling formula and four variables, classified `narrative`

**Passage** (ArMDE:5201 and :5203, verbatim):
> One of the character's magic powers (Greater, Lesser, Personal, or Ritual) becomes more powerful over time, **based on either (age / 10), (Might Score / 5), Warping Score, or his score in an appropriate Ability.** (Age and Warping should not be allowed as variables if the character is immune to their effects.) …
>
> Generally speaking, **a single power can have its effect level increased by five, or include another effect of the same level that is similar the original, for each level of variance.** For a character with Variable Powers based on his age, for example, this would mean the effectiveness of one of his powers would be **increased by a total of five magnitudes after he has lived for 50 years.**

German, ArMDE:5201, :5203 (line-parallel):
> Eine der magischen Kräfte des Charakters (Groß, Gering, Persönlich oder Ritual) wird im Laufe der Zeit stärker, basierend auf entweder **(Alter / 10), (Machtwert / 5), dem Verzerrungswert** oder seinem Wert in einer passenden Fertigkeit. …
>
> Im Allgemeinen kann eine einzelne Kraft ihre Effektstufe **um fünf erhöhen** … **für jede Variationsstufe.**

**Current data.** `classification: "narrative"`, no effects, one parameter
(`power`, `text`, `require_power: true`). Both `summary` fields are ArMDE:5201's
first sentence.

**Why the *data* is right and the *class* is wrong.** The multiplicity half is
correct and fully recorded: `RULES.md:2417-2448` quotes ArMDE:5205 ("may be
taken more than once, if the character has more than one power, but it only
applies once to a single power"), explains why `max_per_target: 1` plus a
free-text `power` parameter is the right encoding and why `domain: "item"` would
be wrong, and names the guard tests. No finding there.

But the passage also states three arithmetic rules — two explicit divisors
(`age / 10`, `Might / 5`), a five-levels-or-one-more-effect step per level of
variance, and a worked example yielding five magnitudes at fifty years. B1's
test is whether *the rulebook* said something mechanical, and it said three
things.

**Correct value.** `classification: "uncomputed_rule"`, with the four variables,
the immunity caveat and the per-variance scaling written into `description` in
both locales. No effect variant fits: the scaling target is a power's *level*,
and `power_levels` (A27) is a creation budget, not a per-power multiplier.

**Severity.** Lost rule.

---

### F-337 — `virtue.verditius_magic` — `narrative` on an entry whose two rules the engine already computes

**Passage** (ArMDE:5217, verbatim — the entry's whole body):
> **You have been initiated into the Outer Mystery of Verditius Magic (see page 240), and thus are a member of House Verditius.** Note that **all Verditius magi gain this Virtue for free at character creation.**

German, ArMDE:5217 (line-parallel):
> **Du wurdest in das Äußere Mysterium der Verditius-Magie eingeweiht (siehe [Seite 240](#verditius--verditius-magie)) und bist damit Mitglied des Hauses Verditius.** Beachte, dass **alle Verditius-Magi diese Tugend bei der Charaktererschaffung kostenlos erhalten.**

**Current data.** `classification: "narrative"`, no effects,
`prerequisites: { "kind": "house", "value": "house.verditius" }`.

**Why wrong.** Both sentences state mechanics, and **both are already
computed**:

- The House-membership clause is the prerequisite, and `RULES.md:3760-3775`
  identifies this entry as one of exactly four V/F descriptors in the core rules
  carrying the "and thus are a member of House X" clause, all four of which
  carry the bare `house` leaf.
- The free-grant clause is encoded in `rules/core/houses.json`:
  `house.verditius` carries
  `"grants": [{ "kind": "fixed", "item": "virtue.verditius_magic" }]` (read
  directly this run).

So `narrative` — "the cited passage states **no mechanical clause at all**" — is
false twice, and the entry is out of step with its own three siblings:
`virtue.the_enigma`, `virtue.faerie_magic` and `virtue.heartbeast` are all
`creation_effect`. The only thing distinguishing this one is that its Outer
Mystery seeds no Ability, which `RULES.md:3751` records as "(no creation-number
effect)" — a statement about *effects*, not about whether the book said
something.

This is C7's shape exactly: a `creation_effect` whose mechanic is modelled by
something other than an `Effect` (here, a prerequisite and a House grant), which
`data_integrity.rs::every_vf_is_classified` cannot detect because it asserts the
converse only for `in_play_effect`.

**Correct value.** `classification: "creation_effect"`. No data change to the
entry is needed for the class itself.

**Severity.** Provenance / lost-rule. Low mechanical impact — both rules already
fire — but it is the audit's cleanest example of a wrong `narrative` on an entry
the engine *does* compute, which is a transition the README warns has never been
looked for.

---

### F-338 — `virtue.verditius_magic` — the entire Outer Mystery behind "(see page 240)" reaches neither locale (D5)

**Passage behind the pointer** (ArMDE:10076-10090, the section
`### Verditius — Verditius Magic`; the four load-bearing clauses verbatim):
> As part of this process, the magus may add details that enhance the Shape and Material bonus of the item. **These details give an additional bonus to all the item's existing Shape and Material bonuses equal to the creating magus's Philosophiae score, for the purposes of enchantment.** … The total bonus from Shape and Material and Verditius Runes is still limited by the magus's Magic Theory score. (`:10080`)
>
> … a magus who does so may **add his score in the relevant Craft Ability to all Lab Totals for enchanting that item**, both in the first season and in the future. … Note that only one Craft Ability can be added to the Lab Total in a given season … (`:10084`)
>
> If the magus creates the enchanted item in this way, **the number of pawns of vis needed to open the item for enchantment is reduced by the magus's Craft score, to a minimum of one pawn.** (`:10086`)
>
> **Verditius magi need casting tools to cast Formulaic or Ritual spells.** … However, if they lose or cannot reach their tools, **they cannot cast their Formulaic spells.** Spontaneous spells do not require casting tools … (`:10090`)

**Current data.** Both `summary` fields paraphrase the initiation
("Initiation into the Outer Mystery of House Verditius, enabling the casting of
enchantments through craft." / "Einweihung in das Äußere Mysterium des Hauses
Verditius, das das Wirken von Verzauberungen durch Handwerk ermöglicht."), which
gestures at the mystery without stating one rule of it. Neither locale carries a
`description`.

**What is left over.** All of it. Four of the seven clauses are hard mechanics
with named statistics, and one of them — the Craft-Ability addend at :10084 — is
a **Lab Total modifier**, i.e. the shape `lab_total_mod` (A31) exists for and
D1/D4 are entirely about. It is conditional in exactly D4's manner ("for
enchanting **that item**", one Craft Ability per season), so it belongs in
`derived/lab.rs::LabTotal`'s conditional half rather than the flat one — but no
carrier exists at all today, which is a step before the D4 question. The
casting-tool requirement at :10090 is the mirror image: a *drawback* on a Minor
Virtue, and one that can stop a magus casting Formulaic spells entirely.

I checked whether any of this is already recorded rather than assuming: `grep -an
"Verditius Rune\|casting tool\|Hubris\|Philosophiae"` over
`crates/arm-rules/RULES.md` returns **one** hit, `RULES.md:4321`, and that is the
unrelated Ritual casting-total formula. None of the four clauses appears in the
traceability map.

**Correct value.** A `description` in both locales carrying at minimum the
Philosophiae Rune bonus, the Craft addend, the vis reduction and the casting-tool
requirement, each as the book states it. Whether any of them should become an
effect is a design question this batch does not settle.

**Severity.** Lost rule, and the largest single block of it in the batch — the
whole content of a House's Outer Mystery, for the one House whose entire
identity is enchantment.

---

### F-339 — `virtue.wealthy` — the companion-only rule is enforced for magi and mythic companions and not for grogs

**Passage** (ArMDE:2394, verbatim — where the entry's number actually lives):
> **Characters with the Wealthy Virtue get 20 experience points per year, while characters with the Poor Flaw get 10 experience points per year. Note that only companions can take this Virtue or Flaw.**

and ArMDE:2214, the summary step:
> **Later Life.** 15 experience points per year (until apprenticeship for magi) … **Characters with the Wealthy Major Virtue get 20 experience points per year, those with the Poor Major Flaw get 10.**

The entry's own passage (ArMDE:5237) states only the magus half:
> Wealthy characters may live at a covenant, but they are not supported by the covenant. **As all Hermetic magi are supported by their covenant, no magi may take this Virtue.**

**Current data.** `creation_effect` ✓, one effect:
`{"type": "later_life_xp_rate", "amount": 20}` ✓ — the 20 is exactly ArMDE:2214
/ :2394, and A12 records the semantics (replaces the base rate, lowest wins).
`entity_kinds: ["character"]`; no prerequisites.

**Where the restriction is, and where it is not.** A12 states that "the rule
that both Wealthy and Poor are companion-only is **not** in this effect — it is
enforced by the type profiles". I checked all four profiles rather than taking
that on trust (`jq` over `rules/core/character_types.json`, this run):

| Profile | `forbidden_traits` |
|---|---|
| companion | `[]` — correct, this is the one type that may take it |
| **grog** | `["virtue.the_gift"]` — **does not name `virtue.wealthy`** |
| magus | `["flaw.poor", "virtue.wealthy"]` ✓ |
| mythic_companion | `["flaw.poor", "virtue.wealthy"]` ✓ |

So two of the three excluded types are covered and the third is not. A grog may
take the Major Virtue Wealthy today with no issue raised — and ArMDE:2394 says
"only companions", in as many words, so the grog is exactly as excluded as the
magus.

**Why the omission is understandable and still wrong.** The entry's *own*
descriptor (ArMDE:5237) names only magi, so anyone authoring from that passage
alone would produce precisely this data. The companion-only rule lives 2,800
lines earlier, in the life-stage chapter, beside the number. That is the same
split-provenance shape this batch met on `virtue.true_faith` — the entry's own
lines are not where its rule is.

**Correct value.** Add `"virtue.wealthy"` (and, when its batch reaches it,
`"flaw.poor"`) to the grog profile's `forbidden_traits`. Whether ArMDE:2394's
"only companions" is meant to bind grogs, given ArMDE:5237 names only magi, is
**Q-75**; the data should not be changed until that is answered, but it cannot be
left in its current state either, since it is inconsistent with its own two
neighbours.

**Severity.** Wrong rules output — the app permits an illegal character. C3b
(the flat XP-pool path making `later_life_xp_rate` inert) is a separate,
recorded gap and is **not** counted here.

---

### F-340 — `virtue.wealthy` — eight passages state a Wealthy/Poor exclusion and `incompatible_with` is empty catalogue-wide

**Passages** (verbatim, the operative clause of each):

| ArMDE | Entry | Clause |
|---|---|---|
| :3611 | covenant folk | "**You may not take the Wealthy Major Virtue or the Poor Major Flaw.**" |
| :3631 | covenant specialist | "you may not take the Wealthy Virtue or Poor Flaw" |
| :4494 | (monastic, vows of poverty) | "**You may not take the Wealthy Virtue or Poor Flaw.**" |
| :4634 | (Cathar perfectus) | "**You may not take the Wealthy Virtue**, as you are supported by the tithes …" |
| :4850 | `virtue.redcap` | "You are supported by your covenant, so **you cannot take the Wealthy Virtue or Poor Flaw.**" |
| :5181 | `virtue.turb_trained` | "he is thus **prohibited from being Wealthy or Poor**" |
| :5751 | (branded criminal) | "**You may not take the Wealthy Virtue**, but you may take Martial Abilities …" |
| :6540 | `flaw.outcast` | "**You may not take the Wealthy Virtue.**" |

**Current data.** `virtue.wealthy` — `incompatible_with` absent (i.e. `{}`).
And the claim generalises, precisely: `jq` over the whole of
`rules/core/virtues_flaws.json` selecting every entry whose `incompatible_with`
indexes `virtue.wealthy` **or** `flaw.poor` returns **the empty list**. That is
not a sampling result — B7 records that
`ruleset/integrity.rs::validate_incompatibility_symmetry` **fails the load** on a
one-sided declaration, and the ruleset loads, so an empty set on `virtue.wealthy`
is by itself proof that no entry anywhere names it.

**Why wrong.** Eight core passages state the exclusion and none of it is
enforced. B7 records that `validate_incompatibilities` is the only runtime
reader and emits a hard `incompatible` error — so the mechanism exists, is
correct, and is simply unused for the most-excluded Virtue in the book.

**One scope caveat, stated so the fix is not over-sold.** B7 also records that
`selected_ids` is built from **bought** selections only, deliberately (review
finding B1, documented at `validation/prereq.rs::PrereqCtx::build`). So an
`incompatible_with` pair would catch a player who buys both and would **not**
catch a *granted* Redcap beside a bought Wealthy. That is the recorded behaviour,
not a new defect.

**Correct value.** `"incompatible_with"` on `virtue.wealthy` naming all eight
partners (seven for `flaw.poor` — ArMDE:4634, :5751 and :6540 exclude Wealthy
only), with the symmetric entry on each partner, since the load enforces
symmetry. Each partner id must be confirmed against its own entry by the batch
that owns it; this batch confirmed only the two in its own span
(`virtue.turb_trained`, F-325) and `virtue.redcap` (ArMDE:4850, read in full
while following F-342's cross-reference).

**Severity.** Wrong rules output — the app permits an illegal character, on
eight different Virtues at once. This is the batch's widest single defect.

---

### F-341 — `virtue.well_traveled` — fifty experience points stated by the book, carried by no effect, under `narrative`

**Passage** (ArMDE:5241, verbatim — the entry's whole body):
> You have journeyed extensively in this part of the world and find it easy to get along with people throughout the area. **You have fifty bonus experience points to spend on living languages, Area Lores, and Bargain, Carouse, Charm, Etiquette, Folk Ken, or Guile.**

German, ArMDE:5241 (line-parallel):
> Du hast diesen Teil der Welt ausgiebig bereist und kommst gut mit Menschen in der gesamten Region aus. **Du erhältst fünfzig Bonus-Erfahrungspunkte, die du für Lebende Sprachen, Gebietskunde und Feilschen, Zechen, Charme, Etikette, Menschenkenntnis oder Täuschung ausgeben kannst.**

**Current data.** `classification: "narrative"`, **no effects**. Both `summary`
fields are the first sentence only, so neither the fifty points nor the eight
targets reaches the user in either locale.

**Why wrong.** A creation-time XP grant with a closed target list is the single
most-encoded shape in the catalogue — A7 counts 28 `restricted_ability_xp` uses,
and `virtue.warrior`, `virtue.trained_assassin` and `virtue.venditor` in this
very batch all carry one for the identical sentence pattern. `narrative` asserts
the passage states nothing mechanical while it states a number and a list.

**And it is fully expressible — the brief's framing of this entry needs
correcting.** The claim this batch was handed is that "living languages" and
"Area Lores" are instance-scoped families `RestrictedAbilityXp` cannot name. I
checked that against the code rather than accepting it, and **it is wrong**.
`effective/xp.rs::pool_covers` was read in full: for a `PoolEligibility::Ability`
pool it tests `exclude` first, then

```rust
if !instances.is_empty() {
    return instances.iter().any(|i| i.matches(ability, parameter));
}
abilities.contains(ability) || categories.contains(category)
```

— so with the empty `instances` that `restricted_ability_xp_pools` hard-codes,
naming a **parameterized Ability's id** matches **every** instance of it. That is
precisely what "living languages" (plural, unrestricted) and "Area Lores"
(plural, unrestricted) mean. The shipped precedent is one entry away:
`virtue.venditor`'s pool names `ability.living_language` for "**any** Living
Language" ✓.

The limitation is real only for a **single, qualified** instance — B08's
`virtue.shadchan` ("an Area Lore appropriate for their community") and
`virtue.simple_student` ("Latin"), where the passage picks *one*. Well-Traveled
picks none, so nothing is lost.

**Correct value.** `classification: "creation_effect"` plus

```json
{ "type": "restricted_ability_xp", "amount": 50,
  "abilities": ["ability.area_lore", "ability.bargain", "ability.carouse",
                "ability.charm", "ability.etiquette", "ability.folk_ken",
                "ability.guile", "ability.living_language"] }
```

All eight are `general` in `rules/core/abilities.json` (verified this run), so
A7's permission side-effect grants nothing gated and **no**
`ability_authorization` is owed — this is one of the few XP-granting Virtues in
the audit that needs nothing but the pool.

**Severity.** Wrong rules output — a Minor Virtue that should fund fifty
experience points funds zero, so a legal character is under-built by 50 XP and
the validator will report `not_enough_xp` against spends the Virtue was supposed
to pay for.

---

### F-342 — `virtue.well_traveled` — it is the target of two grants, one of which hands out an empty box and the other of which does not declare the grant at all

**Passages** (verbatim):

`virtue.lone_redcap`, ArMDE:4321:
> You still begin with 300 experience points for your fifteen years spent as an apprentice, and **receive the benefits of the Well-Traveled virtue**, but you are estranged from the other Redcaps in your area …

`virtue.redcap`, ArMDE:4848:
> … you have spent fifteen years as an apprentice, and gained a total of 300 experience points in those fifteen years. … **In addition, you have the Well-Traveled Virtue (page 116) at no cost.**

**Current data.**

- `virtue.lone_redcap` — `creation_effect`, three effects, one of which is
  `{"type": "grants_selection", "items": ["virtue.well_traveled"]}` ✓ — the
  grant is declared.
- `virtue.redcap` — `creation_effect`, **one** effect,
  `{"type": "item_level_budget", "amount": 50}`. **No `grants_selection`.**

**The two halves of the defect.**

1. **The declared grant delivers nothing.** A18 records that a granted id
   becomes a bare `Selection` whose own effects are then folded exactly like a
   bought one's. `virtue.well_traveled` has no effects (F-341), so Lone Redcap's
   grant expands to a row that computes zero. The player is told he "receives
   the benefits of the Well-Traveled virtue" and receives none. **F-341 fixes
   this one automatically** — the moment the pool exists, the grant starts
   working, which is the strongest practical argument for fixing F-341 first.
2. **The undeclared grant.** ArMDE:4848 says a plain Redcap "has the
   Well-Traveled Virtue … at no cost" — the same grant, stated at least as
   plainly, and `virtue.redcap` carries no `grants_selection` at all. So the
   two Redcap Virtues disagree about a rule both their passages state.

`virtue.redcap` is B07's entry, not this batch's, and the brief directs that a
cross-reference be **rated**, not deferred: rated, it is a missing
`grants_selection: ["virtue.well_traveled"]`. While reading ArMDE:4842-4850 for
that verdict I also saw two further clauses `virtue.redcap` does not carry —
"you may take Academic. Arcane, and Martial Abilities during character
generation" (three gated categories, no authorization visible in its effect
list) and the 300 apprenticeship experience points — and its ArMDE:4850 Wealthy
exclusion is one of F-340's eight. Those are stated as observations for the
correction pass, not as verdicts: this batch did not read `virtue.redcap`'s full
data, only its `effects` array.

**Correct value.** Add `{"type": "grants_selection", "items": ["virtue.well_traveled"]}`
to `virtue.redcap`. A18's "one level of nesting only" is not a problem here:
Well-Traveled's own effect would be a `restricted_ability_xp`, not a further
`grants_selection`.

**Severity.** Wrong rules output on both entries — 50 experience points missing
from every Redcap and every Lone Redcap in the app.

---

### F-343 — `virtue.wisdom_from_ignorance` — a study rule and an Ability substitution, classified `narrative`

**Passage** (ArMDE:5253 and :5255, verbatim, the operative clauses):
> … pondering texts which have been structured as meaningful, but which the viewer cannot read, can be used as a form of meditation which brings enlightenment. **They are able to use books as sources of training provided they are unable to read the language they are written in.**
>
> The sufis use this ability to gain understanding of spiritual matters, so **players should create Ability books normally, and when sufis use this Virtue, switch the Ability learned from the book to Theology Organisation Lore: Sufis or similarly uplifting subjects.** A sufi using this Virtue may occasionally gain experience in materially practical Abilities, but this only occurs in the context of overcoming specific challenges on spiritual journeys.

German, ArMDE:5253, :5255 (line-parallel):
> … **Sie können Bücher als Quellen für das Üben nutzen, vorausgesetzt, sie können die Sprache, in der sie geschrieben sind, nicht lesen.**
>
> … daher sollten Spieler Fähigkeitsbücher ganz normal erstellen, und wenn Sufis diese Tugend nutzen, **die aus dem Buch erlernte Fertigkeit auf Theologie Organisationskunde: Sufis oder ähnlich erhebende Themen umstellen.**

**Current data.** `classification: "narrative"`, no effects. Both `summary`
fields are the first sentence of ArMDE:5253 (the calligraphy sentence), i.e. the
colour and none of the rule.

**Why wrong.** Two mechanical clauses. The first **inverts a precondition of the
book-study rules** — normally a book can only be studied in a language the
reader knows, and this Virtue requires the opposite; "training" is a defined
advancement source (the `AdvancementSource` taxonomy C1 discusses has a
`taught`/`book` axis). The second is a **substitution rule on what the book
teaches**. `narrative` claims the passage states nothing mechanical; it states
an advancement-source rule and a target-swap rule.

The named target "Theology Organisation Lore: Sufis" is itself worth a note:
`rules/core/abilities.json` has `ability.theology_islam` and
`ability.organization_lore` (parameterized by `organization`) as separate ids,
and the book's phrase runs them together, so a correction pass writing the
description should not invent a single Ability for it.

**Correct value.** `classification: "uncomputed_rule"` with both clauses written
into `description` in both locales. No effect variant fits: C1 records
`advancement_mod` as surfaced-only and it carries an amount, not a rewritten
eligibility condition.

**Severity.** Lost rule.

---

### F-344 — `virtue.ways_of_the_land` — the +3 reaches only Casting Totals, the book gives it to *all* rolls, and the botch-die reduction is carried by nothing

**Passage** (ArMDE:5233, verbatim, the mechanical sentences):
> **You get a +3 bonus to all rolls, including combat and Casting Scores, that directly involve that area and its inhabitants; mundane, magical, or faerie.** In addition, **you roll one fewer botch die than normal (which may mean you roll no botch dice) in rolls that pertain to your area of understanding.** … (For Ways of the Town, this still applies to town animals … rather than people. **The +3 bonus does apply to people.**) You may choose this Virtue multiple times, for different types of terrain.

German, ArMDE:5233 (line-parallel):
> **Du erhältst einen Bonus von +3 auf alle Würfe, einschließlich Kampf und Zauberwerte, die direkt dieses Gebiet und seine Bewohner betreffen; ob weltlich, magisch oder feenhaft.** Außerdem **würfelst du bei Würfen, die deinen Verständnisbereich betreffen, einen Patzerwürfel weniger als üblich (was bedeuten kann, dass du gar keinen Patzerwürfel würfelst).**

**Current data.** `in_play_effect` ✓, one parameter (`land`, `text`) ✓ —
documented as the repeat mechanism at `RULES.md:1485-1495`, which cites
`ArMDE:5233` for exactly this entry — and one effect:
`{"type": "casting_total_mod", "amount": 3, "scope": "all"}`. **This is the one
entry in the batch whose `description` carries its whole passage, verbatim, in
both locales**, so D5 is satisfied and no text finding is owed.

**Three separate complaints, of descending strength.**

1. **The +3 is not a casting bonus; it is a bonus to "all rolls".** The passage
   names combat and Casting Scores as *examples* ("including"), and adds an
   explicit rider that it applies to people for Ways of the Town. A single
   `casting_total_mod` therefore implements a strict subset: `combat_mod` (A35)
   and `ability_roll_mod` (A41) exist and carry none of it, so a Ways of the
   Forest character gets his +3 on spells cast in the forest and nothing on his
   Awareness roll, his Brawl attack or his Survival roll there. This is the
   **under**-implementation half and it is not on record anywhere I could find
   (`RULES.md:5131` lists the entry only as a `CastingTotalMod` carrier).
2. **The condition is folded flat** — `scope: "all"` means every Casting Total
   in every situation, while the book scopes it to rolls "that directly involve
   that area and its inhabitants". This is D4's shape on a casting total rather
   than a Lab Total, the same complaint B07 raised as F-225 and B08 as F-284 —
   but unlike those two, **this one is recorded**: `RULES.md:5131` says of
   `CastingTotalMod`, verbatim, "computed; **conditional ones folded
   unconditionally (no toggle exists)**, surfaced per scope as the
   `casting_mod_formulaic`/`_ritual`/`_spontaneous` addends", and lists
   `ways_of_the_land` among the carriers. Reported as corroboration, not
   discovery.
3. **The botch-die reduction is carried by nothing.** No `Effect` variant in
   the 42 touches botch dice (checked against Part A's full list), so this is
   D3 — and it is already in the `description` in both locales ✓, which is the
   correct outcome under D5.

**Correct value.** Not fully settled here. Complaint 2 is a recorded decision.
Complaint 3 is correctly handled already. **Complaint 1 is the live one**: the
entry needs, at minimum, a `combat_mod` companion (the passage names combat
explicitly), and arguably an `ability_roll_mod` for the general case. Two
practical notes for whoever takes it: A35's `CombatMod` carries a **`target`**
(`initiative` / `attack` / `defense` / `damage`) and "an item may carry several
`CombatMod` effects, one per affected `target`", so "+3 to combat" has to be
decided per stat rather than written once — and the book says "+3 bonus to all
**rolls**", which is an argument for `attack`/`defense`/`initiative` and against
`damage`. Whether the app should fold several unconditional +3s for a
conditional bonus, or carry one and describe the rest, is a judgement the
correction pass must make — but the present state silently picks the
**narrowest** of the alternatives, which is the one choice the passage does not
support.

**Severity.** Wrong rules output — a **Major** Virtue delivering roughly a third
of what it says, with the two-thirds it drops being the parts a non-magus
character (the typical taker) would actually use.

---

### F-345 — two translation-table rows contradict the DE rulebook; the data is right and the **tables** are wrong

The audit's rule is explicit: "A translation table contradicting the rulebook is
a finding against the TABLE, not against the data", and
`rules/source/de/translation-tables/README.md` records the precedence
rulebook > thematic table > `tugenden-fehler.md`. Two rows in this batch's span
fall on the wrong side of it.

| # | English | Table row | Table's DE | DE rulebook | Data's DE |
|---|---|---|---|---|---|
| 1 | Unaging | `tugenden-fehler.md:219` | **Nicht-Alternd** | **Nicht alternd** (heading, ArMDE:5187) | **Nicht alternd** |
| 2 | Unaffected by The Gift | `tugenden-fehler.md:223` | **Unempfindlich gegenüber der Gabe** | **Unbeeindruckt von der Gabe** (heading, ArMDE:5183) | **Unbeeindruckt von der Gabe** |

**Row 1** is a hyphenation difference only (`Nicht-Alternd` vs `Nicht alternd`),
but it is the kind that silently breaks a lookup: a correction pass grepping the
table for the German name would not find the shipped one.

**Row 2 is the substantive one**, and it is instructive about *why* the table is
wrong. Its `Anmerkung` column reads "SdM:M; nicht betroffen von negativen
Effekten der Gabe/Magischen Ausstrahlung" — i.e. the row was distilled from
**Sphären der Macht: Magie** (RoP:M), not from the core rulebook, and carries
that book's rendering of a Virtue the Definitive Edition core also prints under
a different German heading. The core rulebook is the source for this entry
(`source.file` is the core file), so its heading wins. The same `Anmerkung`
appears on `tugenden-fehler.md:147` (Variable Power) and `:148` (Voice of the
(Land)), where the table and the rulebook happen to agree, so the SdM:M
provenance is not by itself a defect — it is only visible as one where the two
books diverge.

**Correct value, and what D6 obliges.** `decisions.md`'s **D6** — added to the
file *during* this batch's run, by a concurrent workstream — governs this
finding and changes what it asks for. Three of its clauses apply directly:

1. **"Any error found in `rules/source/de/translation-tables/` is corrected here
   and in the source project `arm-de-translation` in the same step."** So these
   two rows are not Phase 2 items; they are fixed now, in both places.
2. **"The tables are authoritative for terminology … They are **not**
   authoritative for **facts about the rules**."** That is a second reason this
   batch treated `reputationen.md:85` and `:86` as **corroboration only** for
   F-312 and F-316 rather than as authority: those rows assert Reputation
   levels and scopes, which D6 says a terminology table has no business
   asserting. The corroboration is still worth having — two independent readers
   reached the same number — but it is not evidence.
3. **"Check the source project before concluding the table is wrong"** — a row
   disagreeing with the rulebook may be a real error (fix both) or merely a
   **stale copy** of a row `arm-de-translation` already has right, as the
   `Covenfolk` case turned out to be. **I could not perform this check.**
   `arm-de-translation` is outside this repository, and a background agent's
   file arguments are containment-checked against the working directory, so any
   read of it is auto-denied. **So F-345 is reported as "the copy in this repo
   disagrees with the rulebook", which is certain, and *not* as "the source
   project is wrong", which I have no evidence for.** Whoever fixes it must
   resolve that fork first.

Mechanically: correct both rows in
`rules/source/de/translation-tables/tugenden-fehler.md` to the DE rulebook's own
headings (`Nicht alternd`, `Unbeeindruckt von der Gabe`), keeping the RoP:M form
in the `Anmerkung` column if it is worth preserving, and mirror the fix in
`arm-de-translation` per D6. **No change to
`rules/i18n/de/virtues_flaws.json`** — the data is already right, and "fixing"
it toward the table would be the regression this finding exists to prevent.

**Severity.** Provenance / localization-hygiene. No user impact today (the app
ships the correct names), but D6 states the mechanism by which it becomes one:
a wrong table is "a *generator* of future wrong data", because the next agent
producing German text reproduces it.

---

### F-346 — `virtue.university_grammar_teacher` — `narrative` on an entry with a permission and a seasonal obligation

**Passage.** ArMDE:5197, quoted in full under F-323.

**Current data.** `classification: "narrative"`, no effects. Both `summary`
fields are the first sentence only.

**Why wrong.** Separate from F-323's missing authorization: the entry's class
asserts that ArMDE:5197 states nothing mechanical, while it states a
creation-time Ability permission ("may purchase the Academic Abilities: Latin
and Artes Liberales at character generation") and a seasonal obligation
("They must teach two seasons out of the year") — the same two-season economy
`virtue.town_magistrate` and `virtue.redcap` state and the app does not model.

A descriptor note worth recording so it is not re-raised: the **English**
ArMDE:5196 prints `*Minor. Social Status*` with a **period** where the other 34
descriptors in the span use a comma, while the line-parallel German prints
`*Klein, Sozialer Status*` correctly. It parses unambiguously and the data reads
it as `magnitude: "minor"`, `categories: ["social_status"]` ✓ — the same OCR
shape B08 recorded at ArMDE:5074.

**Correct value.** `classification: "creation_effect"` once F-323's
authorization is added, with the Latin narrowing, the Teaching expectation and
the two-season obligation in `description` in both locales.

**Severity.** Lost rule.

---

### F-347 — `virtue.venditor` — the Academic permission is carried by nothing, although the XP pool beside it is exactly right

**Passage** (ArMDE:5209, verbatim, the operative sentences):
> He is most likely a minor noble, ousted merchant-guild master, or defrocked cleric. **He may select Academic Abilities at character generation.** You also receive **50 additional experience points that you can put in Bargain, Charm, Folk Ken, Guile, Intrigue, or any Living Language.**

German, ArMDE:5209 (line-parallel):
> Er ist höchstwahrscheinlich ein niederer Adliger, gestürzter Kaufmannsgildenmeister oder enthobener Kleriker. **Er kann bei der Charaktererschaffung Akademische Fertigkeiten wählen.** Du erhältst außerdem **50 zusätzliche Erfahrungspunkte, die du in Feilschen, Charme, Menschenkenntnis, Täuschung, Intrige oder eine beliebige Lebende Sprache investieren kannst.**

**Current data.** `creation_effect` ✓, one effect:
`{"type": "restricted_ability_xp", "amount": 50, "abilities": ["ability.bargain", "ability.charm", "ability.folk_ken", "ability.guile", "ability.intrigue", "ability.living_language"]}`
— **six ids for the passage's six targets, in the passage's own order**,
recorded at `RULES.md:5429`. `ability.living_language` for "any Living Language"
is correct and is the shipped precedent F-341 leans on: with an empty
`instances`, `pool_covers` matches every instance of a parameterized Ability.
The 50 and the list are right, and all six are `general`.

**What is not carried.** Precisely because all six are `general`, A7's
permission side-effect authorizes **nothing gated** — and the passage's separate
sentence grants the whole `academic` category, which *is* gated. So a Venditor
who buys Artes Liberales or Civil and Canon Law gets
`ability_category_requires_virtue`, a hard **error**, on a permission the book
grants him in its own sentence. This is the cleanest instance of the pattern in
the batch: the funding rule and the permission rule are two different sentences,
the first is encoded perfectly, and the second is not encoded at all.

**Correct value.** Add
`{"type": "ability_authorization", "categories": ["academic"]}` alongside the
existing pool — A14's bare-permission case, since the passage permits without
funding.

**Severity.** Wrong rules output — the app refuses a legal character.

### F-348 — `virtue.voice_of_the_land` — a free-text parameter turns a once-only Virtue into an unlimited one (ArMDE:2814)

**Found by the verification sub-agent, not by the first pass, and verified
independently before being accepted.** The first pass rated this entry
`?`/no-finding on the classification axis and marked checks 8-10 "OK" without
asking what `max_total`'s **absence** meant. That was the miss.

**Passage** (ArMDE:2814, verbatim — the general rule):
> **A Virtue or Flaw may be taken more than once only if the description explicitly allows it. Most Virtues and Flaws may only be taken once.**

**`virtue.voice_of_the_land`'s entire body** (ArMDE:5221, verbatim) contains no
such permission:
> The character can speak with any creature whose natural habitat is a particular environment associated with this Virtue (including animals and magic beings), and the character is not normally perceived as either a threat or a prey object by these creatures.

**The controlled comparison is twelve lines later, in this same batch.**
`virtue.ways_of_the_land` — the other "(Land)"-shaped, free-text-parameterized
entry in the span — ends (ArMDE:5233, verbatim):
> **You may choose this Virtue multiple times, for different types of terrain.**

The book grants the repeat to one and withholds it from the other, in adjacent
entries of identical shape. Under ArMDE:2814 that silence is decisive, and the
pair is as clean a control as the audit has had.

**Current data.** `"parameters": [{ "key": "land", "type": "ref", "domain": "text" }]`,
**no `max_total`** (→ the **255** "no stated ceiling" sentinel), no
`max_per_target` (→ 1).

**Why that permits what the book forbids.** B10's three multiplicity axes have
different grouping keys, and this is the authoring error B10 opens by warning
about. `max_per_target`'s key is `(item_ref, **whole** params map)`, so two
selections naming "Forest" and "Mountain" are *different targets* and collide on
nothing — the default of 1 blocks only a byte-identical repeat. `max_total`'s key
is `item_ref` **alone**, and it is at the sentinel. So
`validate_duplicate_selections` and `validate_total_selection_cap` both pass a
build holding five copies of a Minor Virtue the book permits once. Exactly the
OVER-permission shape: **no error is raised, so no test catches it.**

`RULES.md:1485-1495` documents the design this falls out of — "Repeats with a
different target each time — modelled by a *parameter*. Two selections with
different `params` are different duplicate keys, so they never collide;
`max_per_target` stays 1" — and lists `virtue.ways_of_the_land (ArMDE:5233)`
among the entries it covers. That list is of entries the book *does* let you
repeat. Voice of the (Land) has the parameter without the permission, and
RULES.md has no row for it at all.

**Correct value.** `"max_total": 1`. That is the knob keyed on `item_ref` alone,
so it caps the entry however each copy fills its parameter, while leaving the
parameter free to record the one land chosen. **The catalogue already ships this
exact shape**, which is what makes the fix a data change and not a design
question: `flaw.fish_out_of_water_terrain` carries a `text`-domain `terrain`
parameter **and** `"max_total": 1`, and `virtue.inoffensive_to_beings` does the
same with an `enumerated` parameter (both read directly this run).

**Verified safe against grants.** B10 records that `max_total` counts the
**folded** bought-plus-granted list, so a cap could in principle collide with a
House or `grants_selection` grant. It cannot here:
`grep -ran "voice_of_the_land"` over `rules/core`, `crates/arm-rules/src`,
`crates/arm-rules/RULES.md`, `examples` and `ui/src` returns **exactly one
line** — `rules/core/virtues_flaws.json:6396`, the entry's own `id`. No grant,
no example save, no traceability row.

**Severity.** Wrong rules output at character creation — CLAUDE.md's
highest-rated class. Two honest mitigations: it fails **permissive** (it accepts
an illegal build rather than refusing a legal one), and it only bites under
`ValidationMode::Enforced` or `Advisory`.

**The wider pattern, verified and routed rather than claimed.** `jq` over the
catalogue returns **18** entries carrying a `text`-domain parameter with no
`max_total`. Four are accounted for: `virtue.major_magical_focus`,
`virtue.minor_magical_focus` and `virtue.mythic_blood` are capped elsewhere
(`validate_magical_focus` → `multiple_magical_foci`), and
`virtue.ways_of_the_land` is legitimately repeatable per ArMDE:5233. **A fifth
is accounted for by this batch's own reading**: `virtue.variable_power`
(ArMDE:5205, "This Virtue may be taken more than once, if the character has more
than one power") is correctly at the sentinel, and `RULES.md:2417-2448` records
it. So of the three text-parameterized entries in B09's span, two are right and
one is not. The remaining **13** — `flaw.anchored_to_the_land`,
`flaw.bound_to_role_role`, `flaw.magical_being_companion`,
`flaw.restricted_power`, `flaw.servant_of_the_land`, `flaw.slow_power`,
`virtue.academic_concentration_subject`, `virtue.alluring_to_beings`,
`virtue.aptitude_for_sin`, `virtue.doctor_in_faculty`,
`virtue.land_regio_network`, `virtue.perfect_eye_for_commodity` — are listed
**as a work item for whichever batch owns each**, with **no claim made about
any of them**: their passages were not read here, and `flaw.restricted_power` /
`flaw.slow_power` in particular are likely legitimate repeats
(`RULES.md:2417-2448` quotes ArMDE:6689 and :6761 permitting exactly that).

---

## Sub-agent reconciliation

One verification sub-agent independently re-derived the six entries the first
pass rated as carrying no finding. It did not read this file. It **overturned
one and corroborated five**, and its overturn is F-348 above — accepted after I
re-verified every load-bearing claim myself rather than taking its word:
ArMDE:2814 read in place; the `max_total` sentinel and the `max_per_target`
grouping key re-read in B10; `flaw.fish_out_of_water_terrain` and
`virtue.inoffensive_to_beings` confirmed by `jq` to carry `max_total: 1` beside
a parameter; the 18-entry `jq` sweep re-run and its list reproduced exactly; and
the single-occurrence `grep -ran` for `voice_of_the_land` re-run.

**The five corroborations, and what each rests on** — these are not bare
agreements, and two of them settled a question the first pass had left implicit:

- **`virtue.templar_servant`** — clean. Its argument is better than the first
  pass's: the entry's `narrative` is proved by *contrast inside its own block*,
  since its four siblings state numbers (ArMDE:5115 "level 3", :5123 "level 2",
  :5127 "level 4") or a permission (:5135) and it states neither.
- **`virtue.wanderer`** — clean, and it closed the one clause I had flagged as
  "a clarification, not a rule" by showing it is **boilerplate**:
  `grep -n "affect you normally"` returns 13 hits across the Social Status
  Virtues (ArMDE:3573, :3623, :3845, :3961, :4197, :4508, :4600, :4622, :4802,
  :5161, :5225, :5259). A sentence that appears twelve times verbatim is house
  style, not a rule of any one entry. That is a stronger basis than mine.
- **`virtue.warrior`** — clean, and it checked the thing I asserted:
  `effective/xp.rs::ability_authorizations` matches `RestrictedAbilityXp` and
  `AbilityAuthorization` in a **shared arm**, with the source comment
  "Experience earmarked for a category or Ability is itself permission to learn
  it". So the pool really does discharge the permission, and the entry is not an
  instance of the batch's dominant pattern.
- **`virtue.whistle_up_the_wind`** and **`virtue.wilderness_sense`** — clean,
  **and the absolute-vs-delta question the brief raises is now settled from the
  book rather than from A9's code reading.** ArMDE:2639, verbatim (I re-read the
  line to confirm the wording):

  > "(The set will often include Supernatural Abilities in which **the character
  > gets an initial score of 1 from the Virtue granting it, and you will not
  > need to spend experience points for the first point of those Abilities**.)"

  That is A9's floor semantics stated by the rulebook — `max(bought, floor)`
  with the floor's table cost subtracted — and not a delta. A character who buys
  the Ability to 3 ends at 3 and pays for points 2-3. **Both grants are right,
  and the citation for why is now on record.**

**Where I did not simply adopt its reasoning.** It argued that
`virtue.voice_of_the_land` confers **no** Supernatural Ability, so the absence
of an `ability_score_grant` is correct rather than a gap, and proved it three
ways (the Supernatural Ability index at ArMDE:7250-7267 lists Whistle Up The
Wind and Wilderness Sense and no Voice entry; the alphabetical Ability List runs
Thrown Weapon → Whistle Up The Wind → Wilderness Sense with no Voice heading;
the back index gives an `(Ability)` **and** a `(Virtue)` row for both of those
at ArMDE:25700-25703 but only a `(Virtue)` row for Voice at :25666). I accept
that and record it, because it closes a gap the first pass had not thought to
open — but it is a **confirmation of correct data**, not a finding, and it is
listed here rather than as one.

**Its reframing of Q-76 is better than mine and Q-76 is rewritten to match.**
I had asked the question about one entry. It showed the question governs **48**:
of the 115 `supernatural`-category entries, `narrative` accounts for 48
(`creation_effect` 33, `in_play_effect` 9, `uncomputed_rule` 25), and the cohort
is full of the same roll-free-capability shape — `virtue.see_in_darkness`
(already B08's Q-64), `virtue.immunity_to_cold`, `virtue.homing_instinct`,
`virtue.gender_shift`, and this batch's own `virtue.unbound_tongue`. **Moving
Voice alone would make the catalogue less consistent, not more.** It also raised
a standing argument that would flip all 48 at once and correctly declined to
apply it: by ArMDE:2960-2962 every Supernatural Virtue carries a realm
association with aura and Warping consequences, so arguably no Supernatural
Virtue can state *nothing* mechanical. Both are folded into Q-76 below.

**Two routings it produced that belong to the Abilities catalogue, not to this
file**, recorded so they are not lost:

1. **`ability.whistle_up_the_wind`'s entire mechanic reaches no user.** The
   Ability entry (ArMDE:7783-7784) points at "the Hermetic Magic chapter, page
   246" = **ArMDE:10242-10254**, which gives a Stamina + Ability roll, a
   five-row Ease Factor table (6/9/12/15/18 → breeze … hurricane), a
   Communication + Music roll to change a wind, and a per-sunset decay rule.
   `rules/i18n/{en,de}/abilities.json` say only "You can create wind by
   whistling (detailed in the Hermetic Magic chapter)". Its `source.lines`
   `[7783,7785]` brackets the **pointer**, not the rules — unlike its
   neighbour `ability.wilderness_sense` `[7786,7789]`, whose range does contain
   its rule.
2. **`ability.wilderness_sense` drops two numbers in both locales.** ArMDE:7787
   (re-read here to confirm): "**A Perception + Wilderness Sense roll against an
   Ease Factor of 9** lets you determine the direction of north, the upcoming
   weather, or the presence of natural hazards or resources. One roll will only
   reveal one piece of information." The i18n text carries the one-fact-per-roll
   rule in both locales and **neither the attribute nor the Ease Factor**.

Neither is a V/F defect and neither is counted in this batch's totals.

**Heading anchors captured during the pass**, since none of the 35 entries
carries one and the upstream source re-sync will invalidate every line number in
this file. Verified against the book's own link targets:
`templar-servant`, `voice-of-the-land`, `wanderer`, `warrior`,
`whistle-up-the-wind`, `wilderness-sense`. **One trap for whoever writes them
in:** the two Supernatural *Abilities* of the same name take a `-1` suffix
(`#whistle-up-the-wind-1`, `#wilderness-sense-1`), so the **Virtue** anchors are
the un-suffixed forms and the two must not be swapped.

**Process.** The sub-agent reports that it saw the "auto mode" injection
instructing it to use `cat`/`sed`/heredocs and ignored it, that it spawned no
agents, changed no file and ran no state-mutating git command, and that no
command was denied. It re-ran its one load-bearing negative with `grep -ran`
after a plain `grep -rn`, which is the discipline the brief asks for.

## Open questions

Seven readings this batch could not settle from the source. Per the audit's
rules each is escalated rather than decided, and the entries carrying one are
marked `?` and are **not** marked checked.

---

**Q-72 — What is the "Brother-Priest Status Virtue" that `virtue.templar_administrator` names?**

ArMDE:5111 says the Virtue "can replace the **Brother-Knight, Brother-Sergeant,
and Brother-Priest** Status Virtues" (DE 5111: "die Sozialer-Status-Tugenden
Bruder-Ritter, Bruder-Sergeant und **Bruder-Priester** ersetzen"). The first two
resolve cleanly — `virtue.brother_knight` (ArMDE:3533-3536) and
`virtue.brother_sergeant` (ArMDE:3537-3540). The third resolves to **nothing**:
`grep -an "^#### Brother"` over the English source returns exactly three
headings — Brother Chaplain (3529), Brother Knight (3533), Brother Sergeant
(3537) — and `jq` over the catalogue returns no id matching "priest" beyond
`virtue.priest` and `virtue.mazdean_priest`, neither of which is a Templar rank.
So either (a) the book means **Brother Chaplain** and ArMDE:5111 uses a
different word for it (the DE translation follows the English, so it is no help),
or (b) Brother-Priest is a fourth rank the core file does not print, which would
make it a **fifth** missing entry alongside the four `Educated` variants the
README's census closed. This batch does **not** pick a reading. If (a), the
replacement rule names three existing ids; if (b), the census is not closed.

---

**Q-73 — Which `ReputationType` carries a Reputation scoped to an *organization*, as `virtue.templar_prestige` requires?**

ArMDE:5127: "He starts with a Reputation of level 4 **within the Templars**."
`types.rs::ReputationType` has exactly four values — `Local`,
`Ecclesiastical`, `Hermetic`, `Academic` — and none is "within a lay military
order". `Hermetic` is the closest *shape* (a reputation inside one organization)
and the plainly wrong *organization*. A25 offers `kind: None` — the
player-chosen wildcard `virtue.famous` uses — which fits but discards the
information that the audience is fixed by the passage. The translation table
confirms the problem rather than solving it: `reputationen.md:85` gives the
scope as `Templer`, a fifth value. **Whether to widen `ReputationType`, use the
wildcard, or approximate with `Local` is a rules-and-model decision, not a
reading this batch can take from the source.** The same question will recur on
`flaw.gabai`, `virtue.rosh_beth_din` and every other organization-scoped
reputation in the catalogue, so it is worth settling once.

---

**Q-74 — How does True Faith's Magic Resistance join the per-Form grid: does it replace, stack with, or compete against Parma and Might?**

ArMDE:17611: "A character with a True Faith Score gains Magic Resistance equal
to this score multiplied by ten." The engine's grid
(`derived/casting.rs::magic_resistance`) has two terms per Form:
`form_bonus` and `max(might, parma_for_form)`, and A26 records the rule behind
that `max` — "Might and Parma do **not** stack; the higher wins (RoP:M:1472,
ArMDE:2627)". ArMDE:17611 says nothing about interaction. Three readings are
each defensible from the text alone: True Faith joins the same `max` (a third
blanket resistance), True Faith is additive on top (the sentence says "gains",
not "has"), or True Faith is a separate resistance the character may elect to
use. ArMDE:17623's relic rule ("A relic also grants Magic Resistance equal to
ten times its True Faith score to its bearer", plus "A person can only benefit
from **one relic at a time**") hints at a non-stacking model but is about
relics, not about the character's own Faith. **F-329 does not depend on this** —
the number is missing under every reading — but the fix does.

---

**Q-75 — Does ArMDE:2394's "only companions can take this Virtue or Flaw" bind grogs, given that the Virtue's own descriptor (ArMDE:5237) excludes only magi?**

`virtue.wealthy`'s number and its restriction live in different chapters and the
two do not say the same thing. ArMDE:2394 (the life-stage chapter, beside the
20 xp/year): "Note that **only companions** can take this Virtue or Flaw."
ArMDE:5237 (the Virtue's own entry): "As all Hermetic magi are supported by
their covenant, **no magi** may take this Virtue." The data enforces the second
reading twice (magus and mythic-companion profiles) and the first reading not at
all (the grog profile permits it). A grog is neither a companion nor a magus, so
the two passages disagree about him. **This batch does not pick a reading**, but
records that the present data is consistent with neither: it is the
magi-only-excluded reading applied to three of four profiles. See F-339.

---

**Q-76 — Is a roll-free supernatural *capability* with a stated scope `narrative` or `uncomputed_rule`? This governs 48 entries, not one, and should be answered once as a decision.**

The entry that raises it is `virtue.voice_of_the_land` (ArMDE:5221): "The
character can speak with any creature whose natural habitat is a particular
environment associated with this Virtue (including animals and magic beings),
and the character is not normally perceived as either a threat or a prey object
by these creatures." There is no digit, no roll, no Ability grant and no
reference to a rules subsystem — and yet the Virtue plainly *does* something the
rules elsewhere would require magic for. The README's worked examples of
*not*-narrative are all number-bearing (a signed modifier, a botch-dice change,
a cap) and this bears none; `uncomputed_rule`'s own definition ("GM judgement,
open-ended magnitudes") fits it at least as well. **Nothing in `rules/source/`
decides it — it is a project taxonomy question, not a rules question.**

**The scale, which the verification sub-agent established and I had missed.**
`jq` over the catalogue: of the **115** `supernatural`-category entries,
`narrative` accounts for **48** (`creation_effect` 33, `in_play_effect` 9,
`uncomputed_rule` 25), and the cohort is full of the same shape —
`virtue.see_in_darkness` (already escalated by B08 as its **Q-64**),
`virtue.immunity_to_cold`, `virtue.homing_instinct`, `virtue.gender_shift`, and
this batch's own `virtue.unbound_tongue`. **Moving one entry would make the
catalogue less consistent, not more**, which is why this is escalated as a
decision covering all 48 rather than as 48 judgement calls. Answering it also
closes B08's Q-64.

**A standing argument that would flip all 48, flagged and deliberately not
applied.** By ArMDE:2960-2962 every Supernatural Virtue carries a realm
association that determines how it interacts with supernatural auras and confers
Warping immunity in a same-realm aura. Read literally, that means **no**
Supernatural Virtue can state *nothing* mechanical, and the whole `narrative`
cohort is misclassified. The audit's standing instruction is to note
ArMDE:2960-2962 instances without proposing fixes, so this is recorded as input
to the decision and not acted on.

**The distinction the answer must draw**, and why this batch could apply it in
one direction but not the other: `virtue.unbound_tongue` looks like the same
shape and is **not** escalated — it is a finding (F-331) — because its passage
names a consequence the rules *quantify elsewhere* (casting with no voice,
ArMDE:9245's `-10`). Voice of the (Land) names no such consequence. Whether
"removes a barrier the rules impose" is enough without a number is precisely
what is open.

---

**Q-77 — How should a free-text `{land}` placeholder render in German, where the genitive article inflects for gender?**

`virtue.voice_of_the_land`'s German `name` is `Stimme des {land}` and
`virtue.ways_of_the_land`'s is `Wege des {land}`. That is correct for masculine
and neuter terrain nouns ("Stimme des Waldes") and **wrong for feminine ones**
("Stimme des Steppe" for *der Steppe*). The canonical table flags the problem
explicitly — `tugenden-fehler.md:148` gives the German as
**`Stimme des/der (Land)`**, with the slash — and the DE rulebook headings at
5219/5231 dodge it by naming a neuter placeholder ("Stimme des (Landes)",
"Wege des (Landes)"). `RULES.md:2356-2391` records the repo's chosen fix for the
same problem on the four `realm` items — **apposition after a comma**, e.g.
`Student einer Sphäre, {realm}` — but that argument was made for a *picked*
label with a fixed article, and its own text says "a typed word inflects
however the player typed it", which reads as a reason the free-text case is
different. So the precedent points both ways. Neither entry carries a
`name_unfilled`, which is **correct** and not part of this question:
`derive.ts::displayName`'s doc records that the ~36 mid-phrase templates keep
the "(Land)" hint deliberately.

---

**Q-78 — Does `virtue.unaging`'s "If a crisis is not potentially fatal, you suffer no ill-effects" have a home in the crisis engine M6/6b7 built?**

ArMDE:5189 states it and `AgingEffect`'s eight kinds carry nothing like it.
`crisis_survival` is the nearest, and it is a **modifier to the survival roll**
(`aging.rs::crisis_survival` pushes an `AgingEffect::CrisisSurvival` amount into
`CrisisSurvival::modifiers`), not a blanket void of non-fatal outcomes. So the
rule is uncomputed today, which F-332 reports as a text obligation. What this
batch cannot settle is whether it *should* stay that way: the crisis engine
resolves outcomes from a table and knows each row's severity
(`CrisisSeverity` ascends in declaration order, per `aging.rs`'s own test), so
"skip every non-fatal row for this carrier" may be a small, well-scoped change
rather than a new subsystem — or it may cut across the table's design in a way
only its author can judge. **Not decided here.**


