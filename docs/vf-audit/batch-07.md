# Batch B07 — indices 210-244, ArMDE:4598-4883

Entries: 35. Audited: 35. Failures: **30**. Clean: **5**.
Findings: **F-209 … F-257** (49). Open questions: **Q-55 … Q-63** (9).

An independent verification pass re-derived the six verdicts this batch had
cleared, was told that overturning one would be a success and inventing one
would not, and was not allowed to read this file. **Result: 5 confirmed, 1
overturned** — `virtue.relic`, on a reading this batch did not reach because it
did not follow the entry's chapter cross-reference far enough. It also
independently corroborated eleven of this batch's findings from the sources
without having seen them, contributed one open question (Q-63) and one
catalogue-wide pattern, and found the overturn's corroboration inside
`crates/arm-rules/RULES.md` itself. See "Sub-agent reconciliation" at the end.

Finding and question numbers **continue B06's sequence** (B01 F-01…F-33 /
Q-01…Q-08, B02 F-34…F-65 / Q-09…Q-16, B03 F-66…F-89 / Q-17…Q-22, B04 F-90…F-121
/ Q-23…Q-34, B05 F-122…F-160 / Q-35…Q-44, B06 F-161…F-208 / Q-45…Q-54), so this
batch starts at **F-209** and **Q-55**.

**This span has never been screened.**
`crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS` stops at
ArMDE:3950; the whole of 4598-4883 is outside it, as B04's, B05's and B06's
spans were. The failure rate is in line with theirs (B04 28/35, B05 29/35,
B06 31/35, this batch 29/35), and the profile again shifts:

- **Seventeen `narrative` reclassifications** — `notary`, `paid_rights`,
  `partner`, `perfect_balance`, `perfect_eye_for_commodity`, `perfectus`,
  `performance_magic`, `personal_vis_source`, `piercing_gaze`,
  `prestigious_student`, `priest`, `protection`, `rabbi`,
  `rat_up_a_drainpipe`, `religious`, `reserves_of_strength`, `ripper`. That is
  **half the batch**, and the highest count the audit has seen (B06's sixteen
  was the previous high). Ten go to `uncomputed_rule` and seven to
  `creation_effect`; four of the seventeen are refuted by plain digits alone
  (`perfect_balance` "+6", `piercing_gaze` "+3", `protection` "level 3",
  `reserves_of_strength` "+3").
- **Eight instances of the `ability_authorization` pattern**, now found in seven
  consecutive spans — `notary`, `perfectus`, `prestigious_student`, `priest`,
  `redcap`, `religious` (six Virtues with **no effect at all** whose passage
  grants a gated category outright), plus `rosh_beth_din` (a
  `restricted_ability_xp` naming three ids against a passage that grants the
  whole Academic category) and the inverse case below.
- **The inverse of that pattern, and the single most consequential finding in
  the batch: `virtue.privileged_upbringing` (F-234).** The passage explicitly
  *forbids* buying Academic or Martial Abilities out of the normal XP pool, and
  the engine's own documented side-effect — a `restricted_ability_xp` pool is
  permission for what it funds (A7) — grants exactly that. This is the first
  time the audit has found the permission machinery **over**-permitting rather
  than under-permitting.
- **A second `mentored_by_demons`-shaped case, where the app refuses the
  character the passage describes: `virtue.ripper` (F-249).** Its two powers are
  a PeAn(He) 25 and a PeAn 45; `validate_powers` raises a hard
  `over_power_levels` **error** against a budget of 0.
- **`virtue.redcap` alone carries five findings** (F-242…F-246) — three missing
  incompatibilities, a missing `grants_selection`, 300 apprenticeship XP granted
  nowhere, and a missing three-category authorization.
- **A check-1 failure of the opposite shape to B06's** — `virtue.perfectus`'s
  range **over**-includes five lines that belong to `virtue.performance_magic`
  (F-215). B06's `minor_magical_focus` stopped short of its own rule; this one
  swallows its neighbour's.
- **And a defect shape the audit has never seen before, found by the
  verification pass: an effect whose *subject* is the wrong entity.**
  `virtue.relic` and `virtue.powerful_relic` (F-256) put the **relic's** True
  Faith Score onto the **character**, and ArMDE:17607 says in as many words that
  "Only by possessing the True Faith Major Virtue may a character have a True
  Faith score." `crates/arm-rules/RULES.md` already records the correct reading
  in one place and the wrong one in another.

*(This file was written incrementally — the verdict table landed first, findings
were appended as each entry was finished, and the sub-agent reconciliation
last.)*

## The directed read — Potent Magic

Both `virtue.potent_magic_major` and `virtue.potent_magic_minor` cite
**ArMDE:4740-4781**, a single `#### Potent Magic` heading whose descriptor line
reads `*Minor or Major, Hermetic*` (ArMDE:4741). That shared-heading shape is
correct and is B05's, not a defect.

### The two passages, verbatim

**English, ArMDE:4746** (the entire paragraph):

> Minor Potent Magic covers the same narrow fields as a Minor Magical Focus, and
> grants a +3 bonus to Lab Totals and Casting Score.

**English, ArMDE:4748** (the entire paragraph):

> Major Potent Magic covers the same wide fields as a Major Magical Focus, and
> grants a +6 bonus to Lab Totals and Casting Score.

**German, line-parallel, ArMDE:4746 and ArMDE:4748:**

> Kleine Potente Magie umfasst dieselben engen Bereiche wie ein Kleiner
> Magischer Fokus und gewährt einen Bonus von +3 auf Laborsummen und den
> Zauberwert.

> Große Potente Magie umfasst dieselben weiten Bereiche wie ein Großer Magischer
> Fokus und gewährt einen Bonus von +6 auf Laborsummen und den Zauberwert.

### What the focus condition is, exactly

The condition is **not** stated on the +3/+6 sentences themselves. It is stated
one paragraph earlier and in the same terms for both magnitudes.

**English, ArMDE:4742** (the entire paragraph, verbatim — note the OCR
duplication of "unlike a Magical Focus", which the German does not have):

> The maga's magic is particularly attuned to a narrow field, much as in a
> Magical Focus. The benefits of Potent Magic are compatible with a Magical
> Focus, unlike a Magical Focus, unlike a Magical Focus, a maga may have more
> than one area of Potent Magic, although only one Potent Magic Virtue applies
> to any single activity.

**English, ArMDE:4744:**

> Potent Magic provides the maga with a bonus **in her field of magic**, and
> permits her to devise Potent spells that gain a casting bonus from the
> sympathetic magic in shapes and materials. Potent Magic can be taught as an
> alternative to a Magical Focus.

**German, line-parallel, ArMDE:4742 and ArMDE:4744:**

> Die Magie der Maga ist in einem engen Bereich besonders ausgeprägt, ähnlich
> wie bei einem Magischen Fokus. Die Vorteile Potenter Magie sind mit einem
> Magischen Fokus vereinbar; anders als ein Magischer Fokus kann eine Maga jedoch
> mehr als einen Bereich Potenter Magie besitzen, wenngleich nur eine einzige
> Potente-Magie-Tugend auf eine bestimmte Tätigkeit angewendet werden kann.

> Potente Magie verschafft der Maga einen Bonus **in ihrem Magiebereich** und
> erlaubt ihr, Potente Zauber zu erfinden, die beim Zaubern durch sympathetische
> Magie in Formen und Materialien einen Bonus erhalten. Potente Magie kann
> anstelle eines Magischen Fokus unterrichtet werden.

**So the condition, stated for D4's Phase 2 slice as precisely as the source
permits:**

| | Value |
|---|---|
| **The condition** | The activity must lie **within the maga's field of Potent Magic** — "a bonus in her field of magic" (ArMDE:4744) / "einen Bonus in ihrem Magiebereich". |
| **Which totals it moves** | "Lab Totals **and** Casting Score" (ArMDE:4746, :4748) — **both**, in one sentence, with one condition governing both. |
| **The Minor field** | "the same narrow fields as a Minor Magical Focus" (ArMDE:4746) — which ArMDE:4538 defines as "slightly narrower than a single Technique and Form combination, although it may include restricted areas of several such combinations". |
| **The Major field** | "the same wide fields as a Major Magical Focus" (ArMDE:4748) — which ArMDE:4401 defines as "smaller than a single Art, but may be spread over several Arts". |
| **How the two magnitudes differ** | **Only in the breadth of the field and the size of the bonus.** Minor: narrow field, **+3**. Major: wide field, **+6**. The arithmetic is otherwise word-for-word identical, and neither sentence adds a clause the other lacks. |
| **Second condition, and it is separate** | "only one Potent Magic Virtue applies to any single activity" (ArMDE:4742) — a maga may hold several, but they never stack on one activity. |
| **Not a condition** | "The benefits of Potent Magic are compatible with a Magical Focus" (ArMDE:4742) — the two bonuses coexist. |

**Two things follow that are findings, not observations, and both are new.**

1. The field condition governs the **Casting Score** exactly as it governs the
   Lab Total — one sentence, one condition, two totals. D4's table decides only
   the Lab half ("not in `total`; belongs in `within_focus`"). The
   `casting_total_mod` half is authored `"scope": "all"` and is folded flat into
   every Casting Total by `derived.rs::InPlayMods::casting_mod_for`, and
   `CastingTotal::within_focus` / `PenetrationLine::within_focus` already exist
   on the casting side (`derived/casting.rs`). → **F-225.**
2. ArMDE:4742 states in both languages that **a maga may have more than one area
   of Potent Magic**. The data declares the two entries mutually
   `incompatible_with`, and `ruleset/integrity.rs::validate_magnitude_variant_exclusivity`
   *forces* that declaration because the ids follow the `<stem>_major` /
   `<stem>_minor` pattern. The book denies exactly what the guard compels.
   → **F-226**, with **Q-57** on how to lift it.

## Method

The whole span was read as continuous prose in **both** languages before any
entry was judged — `rules/source/en/Ars Magica - Definitive Edition (Core
Rules).md` ArMDE:4590-4894 and the line-parallel
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md` 4594-4888.

**Line parity holds throughout the span**, checked at every heading rather than
sampled: 4598, 4602, 4606, 4616, 4620, 4624, 4628, 4632, 4642, 4710, 4714, 4728,
4732, 4736, 4740, 4782, 4788, 4792, 4796, 4806, 4810, 4814, 4818, 4822, 4828,
4834, 4838, 4842, 4852, 4856, 4862, 4866, 4870, 4878 — 34 `####` headings for 35
entries, because `#### Potent Magic` (4740) carries **two** entries, one per
magnitude, exactly as its descriptor line `*Minor or Major, Hermetic*` says. The
five block-quote sub-headings inside `virtue.performance_magic`'s range
(`#### Example` 4666, `#### Performance Abilities` 4672, `#### Sorcerous Music
(Performance Magic)` 4686, `#### Recognizing Performance Magic` 4696, and the
nested `##### …` inside Potent Magic's box at 4752/4754/4772) are not entries and
are correctly claimed by nobody in their own right.

**`source.lines` (check 1) — 34 of 35 correct, one wrong, and the error is a
pair.** The convention every correct range follows is "own heading → the line
before the next heading"; four entries (`premonitions` 4788-4790,
`privileged_upbringing` 4806-4808, `puissant_ability` 4814-4816,
`puissant_art` 4818-4820) stop one line earlier, on their last body line rather
than on the trailing blank, which is tighter and equally correct. Every
**non-blank** line in 4598-4883 is cited by exactly one entry; the only
uncited lines are the blanks 4791, 4809, 4817 and 4821.

**Neither convention is enforced, and that is worth a decision before the
upstream re-sync.** `crates/arm-rules/tests/rules_source_provenance.rs::validate_source_ref`
was read in full: it rejects a range whose `end` exceeds the file length, and a
range in which **every** line is blank
(`bracketed.iter().all(|line| line.trim().is_empty())`) — and nothing else. So a
range carrying one trailing blank passes, a range one line tighter passes, and
the range in F-215 that swallows five lines of a *neighbouring* entry passes
too. The 31/4 split in this span is harmless today; under an upstream source
re-sync, in which every line number shifts, the two conventions will drift
differently. Recorded as a normalization decision owed, not as a finding against
any of the four. *(Contributed by the verification pass.)*

The exception is **`virtue.perfectus` 4632-4641**, which follows the convention
and thereby swallows **ArMDE:4636-4640**, a five-line `> #### Example` block
about Orlando, Sorcerous Music and Subtle Magic. Sorcerous Music is defined at
ArMDE:4686-4688 as "Performance Magic (Music)", so that block belongs to
`virtue.performance_magic` — whose own range, 4642-4709, therefore **excludes
its own first worked example**. The block is physically misordered in the
Markdown (it sits above the `#### Performance Magic` heading, not below it), so
this is a source-layout artefact the convention transcribed rather than an
invented range. It is still a check-1 failure on both entries. **F-215.**

**No entry in this batch carries an `anchor`** — all 35 `source` objects are
`{file, lines}` only, so B11's "confirm the anchor names the entry" check has
nothing to verify here and the line ranges are the only provenance.

**Magnitude, kind, categories, entity_kinds (checks 3, 4, 5, 6).** Every
descriptor line was read and compared against the data. `kind` is `virtue` for
all 35 and all 35 sit inside the Virtues section — correct. `entity_kinds` is
`["character"]` on all 35, which is right: none is a covenant Boon. `magnitude`
and `categories` agree with every descriptor:

- `*Minor, Social Status*` — notary, perfectus, priest, rabbi, religious
- `*Free, Social Status*` — nuntius, paid_rights, peasant
- `*Major, Social Status*` — partner, redcap, rosh_beth_din
- `*Minor, General*` — perfect_balance, perfect_eye_for_commodity,
  physician_of_salerno, piercing_gaze, privileged_upbringing, protection,
  puissant_ability, rapid_convalescence, rat_up_a_drainpipe, relic,
  reserves_of_strength
- `*General, Minor*` — prestigious_student (ArMDE:4793 prints the two words in
  the reverse order; it parses unambiguously and the data reads it correctly —
  recorded so a later reader does not take it for a data defect)
- `*Major, General*` — powerful_relic
- `*Minor, Hermetic*` — performance_magic, personal_vis_source, puissant_art,
  quiet_magic
- `*Minor, Supernatural*` — persona, personal_power, premonitions, ripper
- `*Major, Supernatural*` — ritual_power
- `*Minor or Major, Hermetic*` — potent_magic_minor (`minor`) and
  potent_magic_major (`major`), one descriptor covering both ✓

No entry in this batch sets `tainted`, and none of the 35 passages carries the
Tainted type label — checked against every descriptor line.

**Check 12, mechanical.** Every `name` and `summary` in the batch was read
against its passage in the matching language. Three whole-file mechanical checks
back the prose: `rg -ac "−"` (U+2212, the mathematical minus) over **both**
locales' `virtues_flaws.json` exits 1 — **zero occurrences**, matching B06's
result; `jq` over the batch's 35 `source` objects returns the key set
`["file", "lines"]` and nothing else, so **no entry carries an `anchor`**; and
`jq` over the batch's 35 entries in **each** locale returns the key set
`["name", "summary"]` and nothing else. **No entry in this batch carries a
`description` in either locale** — all 35 are `name` + `summary` only, which is
why every D5 finding below reads "reaches neither locale" rather than "is wrong
in one". The one place the source prints an en dash for a negative sign
(ArMDE:4824 "a –5 penalty", DE 4824 "einen Abzug von –5") is transcribed with
the **ASCII hyphen** in both locales' summaries ✓ — `virtue.quiet_magic` is the
only entry in the batch carrying a signed value in its text, and it is correct.
Note for the correction pass: ArMDE:4638, :4706 and :4872 also use en dashes
("–5", "–3", "Quickness – (twice power magnitude)"); any description
transcribing them must use the ASCII hyphen per CLAUDE.md.

**German terminology was checked against `rules/source/de/translation-tables/`
and against `rules/i18n/de/abilities.json`, not only against the DE rulebook.**
Fourteen of this batch's German names appear in a table and **all fourteen agree
with it** — this is the first batch in the audit with no German-name finding:

| Entry | Table row | German |
|---|---|---|
| Potent Magic | `tugenden-fehler.md:34` | Potente Magie ✓ (data adds the magnitude disambiguator "(Groß)" / "(Klein)", uninflected) |
| Puissant Art | `tugenden-fehler.md:65` | Begabung in (Kunst) ✓ |
| Puissant Ability | `tugenden-fehler.md:211` | Begabung in (Fertigkeit) ✓ |
| Quiet Magic | `tugenden-fehler.md:66` | Stille Magie ✓ |
| Ritual Power | `tugenden-fehler.md:100`, `magische-qualitaeten.md:39` | Ritualmacht ✓ |
| Personal Power | `tugenden-fehler.md:137`, `magische-qualitaeten.md:63` | Persönliche Kraft ✓ |
| Personal Vis Source | `tugenden-fehler.md:138` | Persönliche Vis-Quelle ✓ |
| Premonitions | `tugenden-fehler.md:139`; `fertigkeiten.md:76` | Vorahnungen ✓ |
| Protection | `tugenden-fehler.md:140`, `:210`; `reputationen.md:80` | Schutz ✓ |
| Privileged Upbringing | `tugenden-fehler.md:209` | Privilegierte Erziehung ✓ |
| Rapid Convalescence | `tugenden-fehler.md:212` | Schnelle Genesung ✓ |
| Redcap | `tugenden-fehler.md:256`; `grundbegriffe.md:98` | Rotkappe ✓ |
| Physician of Salerno | `reputationen.md:82`, `:131` | Arzt von Salerno ✓ |
| Paid Rights | `konvent.md:55` | Erkaufte Rechte ✓ — and the table records a corrected OCR error ("Erkämpfte Rechte" at DE lines 4603+4643); the data and DE:4606 both say **Erkaufte** ✓ |
| Persona (the Ability) | `fertigkeiten.md:74` | Persona ✓ |
| Rabbinic Law (the Ability) | `fertigkeiten.md:78` | Rabbinisches Recht ✓ |

Two table rows are **near-misses that are not findings** and are recorded so
they are not re-raised:

- `tugenden-fehler.md:752` — `Perfectus / Perfecti | Perfectus / Perfecti |
  SdM:G; Frei; Mythischer-Gefährten-Tugend`. That row describes the **RoP:D**
  Perfectus (Free, Mythic Companion). The core-book entry at ArMDE:4632-4634 is
  `*Minor, Social Status*`, which is what the data says. Different Virtue, same
  Latin name; no conflict.
- `grundbegriffe.md:710` — `Notary (Redcap) | Notar | Rotkappe mit
  Quaesitor-Vollmacht für bindende Verträge`. A House Mercere role, not the core
  Social Status Virtue at ArMDE:4598. The German name is the same word and the
  data's `Notar` matches the DE rulebook heading at 4598 ✓.

The remaining nineteen German names appear in no table, so the table constraint
does not bind them; each was checked against its DE rulebook heading instead and
all nineteen match exactly.

**Seven DE Ability names the passages depend on were confirmed against
`rules/i18n/de/abilities.json`**, not inferred: Persona → **Persona**,
Premonitions → **Vorahnungen**, Medicine → **Medizin**, Philosophiae →
**Philosophiae**, Rabbinic Law → **Rabbinisches Recht**, Theology: Judaism →
**Theologie: Judentum**, Dead Language → **Tote Sprache** (`name_unfilled`; the
filled form is `{language} (Tote Sprache)`). All seven match the DE rulebook
body at ArMDE:4712, :4790, :4734 and :4880.

### Cross-references followed

Every pointer in the span was followed and read — the `(see page NNN)` form and
the named-Virtue form alike — including on entries that turned out to pass.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the Virtue? |
|---|---|---|---|
| `virtue.perfectus` | "Purity and Transcendence … *Realms of Power: The Divine Revised Edition* (page 53)" | **RoP:D is in `rules/source/en/`**, so unlike B06's *Cradle and the Crescent* this is claimable in principle — but `ability.purity` and `ability.transcendence` are **not** in `rules/core/abilities.json` (searched the whole `abilities` array). | **Yes** — a conditional permission ("if your character has the True Faith Virtue"). → F-218, Q-56. |
| `virtue.perfectus` | "*Faith and Flame*, from page 13" | **outside `rules/source/en/`** | Nothing claimable. |
| `virtue.perfect_eye_for_commodity` | "the trading rules in City and Guild" | **outside `rules/source/en/`** | The Labor Point formula is stated in ArMDE's own text, so it is a rule the *entry* states; the system it plugs into is not in repo. → F-214. |
| `virtue.performance_magic` | "Fast Casting to oppose a spell … (page 217)" / `#schnellzaubern` | the fast-casting rules | **No** — the baseline the box's Ease-Factor table adjusts. The table itself is the Virtue's own → F-219. |
| `virtue.personal_power` | "see the Hermetic Magic chapter for further details on the functioning of Penetration" | the Penetration rules | **No** — general rules, not attributed to the Virtue. |
| `virtue.personal_power` / `virtue.ritual_power` | "*Realms of Power: The Divine Revised Edition*, *… The Infernal*, *… Faerie*, *Hedge Magic Revised Edition*" | all four are in `rules/source/en/` | **No new rule**, but the sentence they hang off *is* one: "The power must be associated with the same supernatural realm as the system on which it is based." `types.rs::SupernaturalPower` stores `name`, `level` and `penetration` and **no realm**, so it is structurally inexpressible (D3). → F-221, F-251. |
| `virtue.physician_of_salerno` | "the rules in Art and Academe, Chapter Five" | **outside `rules/source/en/`** | Nothing claimable (the "medical formulae" third XP target). |
| `virtue.potent_magic_minor` / `_major` | "the Shape and Material Bonus table (page 282)" | the S&M table, in repo | **No new rule** — the table is the input to Potency, which is the Virtue's own subsystem → F-228. |
| `virtue.potent_magic_minor` | "**the same narrow fields as a Minor Magical Focus**" (named-Virtue, no page) | `#### Minor Magical Focus`, **ArMDE:4536-4542** | **Yes — the breadth definition**, imported wholesale. ArMDE:4538: "the field should be slightly narrower than a single Technique and Form combination". B06's F-201 found that same sentence unstated on Minor Magical Focus itself; it is unstated here too → F-228. |
| `virtue.potent_magic_major` | "**the same wide fields as a Major Magical Focus**" (named-Virtue, no page) | `#### Major Magical Focus`, **ArMDE:4399-4422** | **Yes** — ArMDE:4401: "This area should be smaller than a single Art, but may be spread over several Arts" → F-228. |
| `virtue.powerful_relic` | "see Relics, page 419" / `#reliquien` | the Relics rules | **No** — general rules. |
| `virtue.powerful_relic` | "**As with the Minor General Virtue Relic**" (named-Virtue, no page) | `#### Relic`, **ArMDE:4852-4855** | **Yes, one clause** — "may be built into any other item that you possess", which Relic states too and which neither entry carries in either locale → F-229. |
| `virtue.premonitions` | "(page 170)" / `#vorahnungen-1` | the Premonitions Ability | **No** — the Ability's own rules. Unlike B06's `magic_sensitivity`, the Virtue states nothing the Ability passage has to carry back. |
| `virtue.priest` | "Vow (see page 150)" / `#gelübde` | `#### Vow`, the Minor Personality Flaw | **No** — and the Virtue's own wording is "would normally take", hedged guidance, correctly encoded by encoding nothing. |
| `virtue.privileged_upbringing` | "the Wealthy Virtue (page 115)" / `#wohlhabend` | `#### Wealthy` | **No** — "if you are wealthy now, you **should** take the Wealthy Virtue" is guidance, not a requirement, and not an incompatibility. |
| `virtue.rabbi` | "**the Educated (Hebrew) Virtue**" (named-Virtue, no page) | `#### Educated (Hebrew)`, **ArMDE:3723** | **Yes, and it is a hard requirement** ("must take"). The target entry is one of the **four known-missing `Educated` variants** (README's closed census), so the prerequisite is not expressible in the catalogue today → F-240. |
| `virtue.redcap` | "House Mercere (see page 15)" / `#haus-mercere` | the House description | **No.** |
| `virtue.redcap` | "the Hermetic Magus Social Status (page 85)" / `#hermetischer-magus` | `virtue.hermetic_magus` | **No new rule** — it says Gifted Mercere take *that* Virtue instead, which is a consequence of "You may not take The Gift" (F-242), not a separate rule. |
| `virtue.redcap` | "the Laboratory chapter, page 256" / `#tabelle-zur-verwendungshäufigkeit` | the uses-per-day table | **No** — the general enchantment rules. |
| `virtue.redcap` | "**the Well-Traveled Virtue (page 116) at no cost**" / `#vielgereist` | `#### Well-Traveled`, **ArMDE:5239-5242**, id `virtue.well_traveled` ✓ exists | **Yes — a fixed, single-item free grant**, exactly what `Effect::GrantsSelection` expresses (A18), and encoded nowhere → F-243. The same shape as B06's F-208 on Nephilim. |
| `virtue.redcap` | "Detailed Character Creation, page 43" / `#detaillierte-charaktererschaffung` | the life-stage rules | **No new rule** — but the sentence pointing at it states one: "you have … gained a total of 300 experience points in those fifteen years" → F-244. |
| `virtue.redcap` | "a Longevity Ritual (see page 261)" / `#langlebigkeitsrituale` | the ritual rules | **No** — general rules; the Virtue's own clause (free, from a magus with Lab Total ≥ 50) is F-246's. |
| `virtue.relic` | "See Chapter 12: Realms for rules for relics and True Faith" | `### True Faith` **ArMDE:17603-17617** and `### Relics` **ArMDE:17619-17627** | **YES — four rules, and one of them invalidates the entry's own effect.** This row read "No — general rules, not attributed to the Virtue" in the first pass, which is **wrong** and is the overturn: ArMDE:17607 restricts a *character's* True Faith Score to the True Faith Major Virtue, and ArMDE:17623 gives the *relic* its score, its Faith Points, its bearer's MR ×10 and the one-relic-at-a-time limit. → **F-256, F-257.** |
| `virtue.religious` | "**the Mendicant Friar Virtue**", "**the Priest Social Status**", "**the Senior Clergy Social Status**" (three named-Virtue pointers, no page) | `virtue.mendicant_friar`, `virtue.priest`, `virtue.senior_clergy` — all three exist ✓ | **No rules.** All three are "should take … instead", hedged redirects, not exclusions. Correctly encoded by encoding nothing — the same reading B06 applied to `virtue.merchant_adventurer`'s "should select the Partner Virtue instead". |
| `virtue.rosh_beth_din` | "**the Social Contacts Virtue**" (named-Virtue, no page) | `#### Social Contacts`, **ArMDE:4988-4991**, id `virtue.social_contacts` ✓ | **Yes, and it is already encoded** — `grants_selection: ["virtue.social_contacts"]` ✓. The extra half-sentence ("you are able to find contacts within any Jewish community that supports a yeshivah") is the Virtue's own scoping and reaches neither locale → F-255. |
| `virtue.partner` | "factor, merchant adventurer, local carrier, or urban merchant" (four named Virtues) | `virtue.merchant_adventurer` ArMDE:4510-4513 and its siblings | **Yes — a Virtue-waiver rule.** B06 flagged this passage for B07 from the *other* side (Merchant Adventurer's entry), and confirmed there that holding both is redundant rather than illegal. Read from this side it is a rule this entry states and does not carry → F-212. |

**The named-Virtue form again produced the live hits** — Redcap's Well-Traveled,
Rabbi's Educated (Hebrew), Potent Magic's two Magical Focus breadth definitions,
Powerful Relic's Relic clause and Partner's four roles are all page-number-free
pointers a `(see page NNN)` screen cannot see, and every one of them yields a
rule the entry drops. That is now true in five consecutive batches.

### Part C systemic gaps are not re-reported per entry

In particular: `true_faith_grant` producing a number nothing consumes (C3c) is
**not** counted as a defect of `virtue.relic` or `virtue.powerful_relic` (F-256
is a different complaint entirely — that the effect names the wrong *subject*,
not that its result goes unconsumed);
`health_mod`'s `recovery` track being surfaced-only (C1) is **not** counted
against `virtue.rapid_convalescence`; `grants_reputation`'s unenforced `score`
(C2-c) is **not** counted against `virtue.physician_of_salerno` or
`virtue.rosh_beth_din`; and the frontend `Effect` union's missing
`power_levels` and `ability_authorization` tags (C6) are **not** counted against
`virtue.personal_power` or `virtue.ritual_power`. What *is* counted for each is
what D5 obliges — the clause the effects do not implement, absent from both
locales.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D4 is the decisive one here, and it is the directed read.** Two entries in
this span carry `lab_total_mod` — `virtue.potent_magic_major` and
`virtue.potent_magic_minor`, which are exactly the two rows D4's table resolves
as "not in `total`; belongs in `within_focus`". That half is **already ruled and
is not re-reported as a finding**. What *is* reported is the casting half
(F-225), which D4's table does not cover and which has the identical condition
from the identical sentence.

**D1** does not bite beyond the same two entries: D1 governs
`effective/spell.rs::spell_level_cap` alone and rules that all nine
`lab_total_mod` carriers apply there flat, conditions ignored. Nothing in this
batch contradicts that and no finding rests on it.

**D5 is applied to all sixteen effect-carrying entries in the span** —
`persona`, `personal_power`, `physician_of_salerno`, `potent_magic_major`,
`potent_magic_minor`, `powerful_relic`, `premonitions`, `privileged_upbringing`,
`puissant_ability`, `puissant_art`, `quiet_magic`, `rapid_convalescence`,
`redcap`, `relic`, `ritual_power`, `rosh_beth_din` (the two Potent Magic rows
counted separately). **Twelve** leave a stated rule that no effect
implements and that appears in neither locale (F-221, F-223, F-228, F-229,
F-234, F-237, F-238, F-239, F-246, F-251, F-255, **F-257**). The four
effect-carrying entries with nothing left over are **`virtue.persona`**,
**`virtue.premonitions`**, **`virtue.rapid_convalescence`** and — on the D5 axis
only — `virtue.privileged_upbringing`, whose finding is a *wrong* computation
rather than a missing description.

F-257 is the verification pass's, not this batch's: the first pass read
`virtue.relic`'s summary ("The relic does not possess any additional powers")
as closing the entry and did not follow "**See Chapter 12: Realms for rules for
relics and True Faith**" into ArMDE:17619-17624, where four mechanics wait. That
is the named-cross-reference pattern this file has already recorded as the most
productive in five batches, applied to a `(see Chapter N)` form rather than a
`(see page NNN)` one — and the first pass missed it on exactly the entry it had
declared clean.

**D5 was also applied in the negative** — the passages of the two `narrative`
entries that survived (`virtue.nuntius` ArMDE:4604, `virtue.peasant`
ArMDE:4622) were read for a mechanical clause and found to state none. Both
contain only the "explicit statement that *nothing* changes" shape —
"He is affected by the Wealthy Virtue and Poor Flaw as normal" / "The Wealthy
Major Virtue and Poor Major Flaw affect you normally" — which B05 and B06 both
rated as correctly encoded by encoding nothing (`virtue.landed_noble`,
`virtue.merchant`). Nuntius's "The character can be either male or female" is
the mirror of B05's and B06's five gender restrictions: an explicit *absence* of
a restriction, which likewise encodes to nothing.

**D3** governs six findings where the engine structurally cannot express the
rule and the answer is therefore a `description` in both locales (or
`uncomputed_rule` for an effect-less entry), and **never** `narrative`: the
per-power realm association (F-221, F-251 — `SupernaturalPower` has no realm
field), Rosh Beth Din's national Reputation scope (F-254 — `ReputationType` has
no national tier), Priest's *conditional* Poor exclusion (F-233 — the save
cannot hold "am I a parish priest?"), Physician of Salerno's "must be able to
take Academic Abilities" prerequisite (F-223 — no `Prereq` variant tests an
authorization), and the four gender restrictions (F-211, F-233, F-240, F-255 —
`types.rs`'s concept field declares `pub gender: String` with "no mechanical
effect", the same ground as B05's F-123 and B06's F-170/F-179/F-189/F-192).

**D2** does not bite: no entry here is a granted Great Characteristic carrier.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.notary` | 4598-4601 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | clergy/secular-law restriction in neither locale | F-209 F-210 ArMDE:2816 |
| `virtue.nuntius` | 4602-4605 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.paid_rights` | 4606-4615 | narrative → uncomputed_rule | OK | female-only, the Social-Status compatibility override and the three un-buyable prohibitions in neither locale | F-211 ArMDE:2816 |
| `virtue.partner` | 4616-4619 | narrative → uncomputed_rule | OK | the Virtue-waiver rule in neither locale | F-212 ArMDE:2816 |
| `virtue.peasant` | 4620-4623 | OK | OK | OK | clean; ArMDE:2816 instance |
| `virtue.perfect_balance` | 4624-4627 | narrative → uncomputed_rule | OK | "+6" in neither locale | F-213 |
| `virtue.perfect_eye_for_commodity` | 4628-4631 | narrative → uncomputed_rule | OK | "never fails" + the Labor Point formula in neither locale | F-214 |
| `virtue.perfectus` | 4632-**4641** | narrative → creation_effect | **`source.lines` swallows the next entry's example**; effects missing (`ability_authorization`, academic); Wealthy exclusion missing from `incompatible_with` | Purity/Transcendence permission in neither locale | F-215 F-216 F-217 F-218 ArMDE:2816 Q-56 |
| `virtue.performance_magic` | **4642**-4709 | narrative → uncomputed_rule | **`source.lines` excludes its own first example**; no parameter for the named Ability, and it cannot be taken twice | the Duration, the Ease Factor 3 roll, the Ability restriction, the combat botch rule and the Ease-Factor table in neither locale | F-215 F-219 F-220 |
| `virtue.persona` | 4710-4713 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.personal_power` | 4714-4727 | OK | OK | Initiative, Fatigue cost, Personal/constant requirement and realm association in neither locale | F-221 ArMDE:2960 |
| `virtue.personal_vis_source` | 4728-4731 | narrative → uncomputed_rule | OK | the one-tenth yield rule in neither locale | F-222 Q-60 |
| `virtue.physician_of_salerno` | 4732-4735 | OK | OK | the "must be able to take Academic Abilities" prerequisite in neither locale | F-223 Q-55 |
| `virtue.piercing_gaze` | 4736-4739 | narrative → uncomputed_rule | OK | "+3", the forced rolls and the faerie/demon exemption in neither locale | F-224 |
| `virtue.potent_magic_major` | 4740-4781 | OK | **casting bonus folded in flat**; `incompatible_with` contradicts the book; no parameter for the field, cannot be held twice | the Potency subsystem, the one-Virtue-per-activity rule and the breadth definition in neither locale | F-225 F-226 F-227 F-228 Q-57 |
| `virtue.potent_magic_minor` | 4740-4781 | OK | **casting bonus folded in flat**; `incompatible_with` contradicts the book; no parameter for the field, cannot be held twice | the Potency subsystem, the one-Virtue-per-activity rule and the breadth definition in neither locale | F-225 F-226 F-227 F-228 Q-57 |
| `virtue.powerful_relic` | 4782-4787 | OK | **`true_faith_grant` names the wrong subject** | the relic's one power, the impiety rule and the built-into-an-item clause in neither locale | F-229 F-256 Q-62 |
| `virtue.premonitions` | 4788-4790 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.prestigious_student` | 4792-4795 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | the Social-Status constraint in neither locale | F-230 F-231 ArMDE:2816 |
| `virtue.priest` | 4796-4805 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | conditional Poor exclusion, male-only and the Magister compatibility in neither locale | F-232 F-233 ArMDE:2816 |
| `virtue.privileged_upbringing` | 4806-4808 | OK | **the engine grants exactly the permission the passage forbids** | the pool restriction in neither locale | F-234 Q-59 |
| `virtue.protection` | 4810-4813 | narrative → creation_effect | `grants_reputation` (wildcard, score 3) missing though the variant exists | "could be higher" + the good/bad choice in neither locale | F-235 F-236 |
| `virtue.puissant_ability` | 4814-4816 | OK | OK | the learning/writing/helping exclusion in neither locale | F-237 |
| `virtue.puissant_art` | 4818-4820 | OK | OK | the requisite rule + the learning/teaching/writing exclusion in neither locale | F-238 |
| `virtue.quiet_magic` | 4822-4827 | OK | OK | the booming-voice and Voice-Range clauses in neither locale | F-239 |
| `virtue.rabbi` | 4828-4833 | narrative → uncomputed_rule | required Educated (Hebrew) encoded nowhere (target entry missing from the catalogue) | the requirement + male-only in neither locale | F-240 ArMDE:2816 |
| `virtue.rapid_convalescence` | 4834-4837 | OK | OK | OK | clean |
| `virtue.rat_up_a_drainpipe` | 4838-4841 | narrative → uncomputed_rule | OK | the Athletics advantage in neither locale | F-241 Q-61 |
| `virtue.redcap` | 4842-4851 | OK | three `incompatible_with` missing; `grants_selection` missing; 300 XP granted nowhere; `ability_authorization` missing (academic, arcane, martial) | the two-season obligation, the item restriction, the +2 levels/year and the free Longevity Ritual in neither locale | F-242 F-243 F-244 F-245 F-246 ArMDE:2816 |
| `virtue.relic` | 4852-4855 | creation_effect → uncomputed_rule | **`true_faith_grant` names the wrong subject** | the relic's Faith Points, MR ×10, Divine Might and the one-relic limit in neither locale | F-256 F-257 (overturn) |
| `virtue.religious` | 4856-4861 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | OK | F-247 ArMDE:2816 |
| `virtue.reserves_of_strength` | 4862-4865 | narrative → uncomputed_rule | OK | "+3 Strength", once per day, two Fatigue rolls in neither locale | F-248 |
| `virtue.ripper` | 4866-4869 | narrative → creation_effect | **70 levels of power carried by no `power_levels` — the app errors on a legal character** | the Fatigue cost, Sight range, no words/gestures, Penetration +0 in neither locale | F-249 F-250 ArMDE:2960 Q-58 |
| `virtue.ritual_power` | 4870-4877 | OK | OK | Initiative, the Fatigue tiers, the Confidence cost and the realm association in neither locale | F-251 ArMDE:2960 |
| `virtue.rosh_beth_din` | 4878-4883 | OK | three Ability prerequisites missing; academic category authorized as three ids | the national Reputation scope, male-only and the yeshivah clause in neither locale | F-252 F-253 F-254 F-255 ArMDE:2816 |

**Totals: 30 entries carry at least one finding, 5 are clean** —
`virtue.nuntius`, `virtue.peasant`, `virtue.persona`, `virtue.premonitions`,
`virtue.rapid_convalescence`. 30 + 5 = 35. No entry is left carrying only a
question: every entry marked `?` also carries a finding.

`virtue.relic` was cleared by this batch's first pass and **overturned by the
verification pass**; its row above and F-256/F-257 below are the corrected
verdict, and the reconciliation section at the end records how the first pass
got it wrong.

## Findings

**A qualifier that applies to all eight `ability_authorization` findings
(F-209, F-216, F-230, F-232, F-245, F-247, F-253, and the inverse case F-234),
stated once rather than eight times.** A14 records that
`validation/authorization.rs::validate_ability_authorization` errors
(`ability_category_requires_virtue`) on a held Ability whose category is in
`ruleset.categories_requiring_virtue()` unless its id or category is authorized,
with a whole-character exemption for a profile whose `is_magus` is true. That
gated set is `rules/core/abilities.json` →
`"categories_requiring_virtue": ["academic", "arcane", "martial"]` (read
directly). Every one of these seven carriers is a **mundane** Virtue — a notary,
a Cathar perfectus, a student, a priest, a Redcap, a monk, the head of a
rabbinic court — so the `is_magus` exemption is empty in practice and the
refusal is exactly what bites. A7 also records that a `restricted_ability_xp`
pool confers the same permission for what it funds, so the check is always "does
the pool's `abilities`/`categories` union cover what the passage permits" — and
for `virtue.physician_of_salerno` it does (the pool names `ability.medicine` and
`ability.philosophiae`, both `academic`, which is exactly the two the passage
funds), which is why that entry draws no authorization finding.

**A second qualifier, for every `narrative → uncomputed_rule` and
`narrative → creation_effect` move below.** B1 records that `classification` is
read by no production code, so none of these moves changes a computed number.
The cost is the one `ecb5150` names: a `narrative` entry is not obliged to carry
its rule in `description`, so the rule leaves the application silently while the
entry still looks complete. Every move therefore carries the same correction —
reclassify **and** write the rule into `description` in **both** locales, or
`uncomputed_clauses.rs::every_uncomputed_rule_entry_states_its_rule_in_every_locale`
goes red. Severity for a bare reclassification: **lost-rule / provenance**, not
miscalculation.

---

### F-209 — `virtue.notary` — `narrative` on an entry that grants a gated Ability category, with no `ability_authorization`

**Passage** (ArMDE:4600, verbatim, the final sentence of the entry):
> The Wealthy Virtue and Poor Flaw affect you normally. **Due to your training, you may take Academic Abilities during character creation;** however, notaries may not be members of the clergy and are subject to secular law.

German, ArMDE:4600 (line-parallel):
> Die Tugend Wohlhabend und der Fehler Arm wirken sich normal auf dich aus. **Aufgrund deiner Ausbildung darfst du beim Erschaffen Akademische Fertigkeiten nehmen;** Notare dürfen jedoch keine Mitglieder des Klerus sein und unterliegen dem weltlichen Recht.

**Current data:** `"classification": "narrative"`, no `effects`, no `prerequisites`.

**Why it is wrong.** `academic` is in
`rules/core/abilities.json`'s `categories_requiring_virtue`, so a notary who
buys Artes Liberales gets `ability_category_requires_virtue` — a hard **error** —
from `validation/authorization.rs::validate_ability_authorization`. The passage
grants that permission in as many words. And the class asserts the book says
nothing mechanical, which is false twice over.

**Correct value.** `"classification": "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}`. Note this is
the *bare permission* case — the passage grants no experience points, so
`ability_authorization` rather than `restricted_ability_xp` is the right variant
(A14's "only needed for a Virtue that permits *without* funding").

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-210 — `virtue.notary` — the clergy and secular-law restriction reaches neither locale (D5)

**Passage** (ArMDE:4600): "**notaries may not be members of the clergy and are
subject to secular law**". German: "Notare dürfen jedoch keine Mitglieder des
Klerus sein und unterliegen dem weltlichen Recht."

**Current data:** both locales carry only the first sentence as `summary`
("Notaries are legal officials employed by lords or towns." / "Notare sind
Rechtsbeamte im Dienst von Herrschaften oder Städten."); no `description` in
either.

**Why it is wrong.** "may not be members of the clergy" is a Social-Status
exclusion — it bars the character from Priest, Religious, Senior Clergy and
Mendicant Friar. It is not a hedge ("may not", not "should not"). It is not
expressible as an `incompatible_with` set without enumerating every clerical
Social Status in the catalogue, so under D3 it belongs in `description`.
"subject to secular law" is the negation of Priest's "you cannot be prosecuted
by secular authorities" (ArMDE:4798) and is likewise a rule.

**Correct value.** Both clauses written into `description` in **both** locales.

**Severity.** Lost rule.

*(The first sentence, "The Wealthy Virtue and Poor Flaw affect you normally", is
the explicit-no-change shape and is correctly encoded by encoding nothing.)*

**ArMDE:2816 instance** — Notary is a Social Status and its exclusion of
clerical statuses is a per-pair rule of exactly the kind ArMDE:2816's
"may only take more than one if the descriptions … explicitly note that they are
compatible" turns on. Noted only; no fix proposed.

---

### F-211 — `virtue.paid_rights` — `narrative` on the entry that overrides the one-Social-Status rule

**Passage** (ArMDE:4614, verbatim, the entire final paragraph — and the entry's
whole mechanical payload):
> This Virtue is only available to female characters, and is compatible with any Social Status that is normally restricted to men, and any Social Status compatible with the Social Status acquired through the paid rights.

German, ArMDE:4614 (line-parallel):
> Diese Tugend ist nur weiblichen Charakteren zugänglich und ist mit jedem Sozialen Status kompatibel, der normalerweise auf Männer beschränkt ist, sowie mit jedem Sozialen Status, der mit dem durch die erkauften Rechte erlangten Status kompatibel ist.

**A second mechanical paragraph**, ArMDE:4612 (verbatim, the operative
sentences):
> There are, however, a few prohibitions that a woman cannot pay a fine to ignore. She cannot pay a fine to do anything that only men are permitted to do in the administrative structure of the Church, which includes universities and cathedral schools. **She could pay a fine to be allowed to study at a university, but not to graduate and gain the accompanying social status.** Similarly, **a woman cannot pay a fine to be ordained into Minor or Holy Orders, for example as a priest.**

German, ArMDE:4612:
> Es gibt jedoch einige wenige Verbote, von denen eine Frau sich nicht durch Zahlung freikaufen kann. Sie kann keine Zahlung leisten, um irgendetwas zu tun, was nur Männern in der Verwaltungsstruktur der Kirche erlaubt ist, was Universitäten und Kathedralschulen einschließt. **Sie könnte eine Zahlung leisten, um an einer Universität studieren zu dürfen, aber nicht um zu graduieren und den damit verbundenen sozialen Status zu erlangen.** Ebenso **kann eine Frau keine Zahlung leisten, um in die Kleinen oder Heiligen Weihen ordiniert zu werden, etwa als Priesterin.**

**Current data:** `"classification": "narrative"`, no `effects`, no
`incompatible_with`, `magnitude: "free"`, `categories: ["social_status"]`. Both
locales carry the opening sociological sentence as `summary` and nothing else.

**Why it is wrong.** ArMDE:4614 is the general *exemption clause* for
ArMDE:2816's "All characters must take one Social Status, and may only take more
than one if the descriptions … explicitly note that they are compatible" — it is
the one entry in the catalogue that grants compatibility with a whole *class* of
Social Statuses. ArMDE:4612 then carves three exceptions out of it, one of which
names a specific Virtue in this very batch (`virtue.priest`, ArMDE:4804 "only
available to male characters"). None of that is colour; all of it decides which
Virtue combinations are legal. `narrative` asserts the passage states nothing
mechanical.

**Correct value.** `"classification": "uncomputed_rule"`, with the female-only
restriction, the compatibility override and the three un-buyable prohibitions
written into `description` in **both** locales. The compatibility override is
not expressible today: `incompatible_with` is a flat symmetric id set with no
"compatible with any member of category X" form, and the ArMDE:2816 rule itself
has no engine representation (the brief records `virtue_category_caps` is absent
on companion, grog and mythic_companion). D3 therefore governs.

**Severity.** Lost rule, and the highest-leverage one in the span — it is the
rule that makes several *other* Social Status Virtues legal for a female
character.

**ArMDE:2816 instance.** Noted only; no fix proposed.

---

### F-212 — `virtue.partner` — the Virtue-waiver rule reaches neither locale, and the entry is `narrative`

**Passage** (ArMDE:4618, verbatim, the final two sentences):
> A partner may act in any of the roles of the house without taking the Virtue that corresponds to that role, save the role of capo, with the permission of the troupe. That is, **a partner who is also a factor, merchant adventurer, local carrier, or urban merchant need not purchase that Virtue if she has this one.**

German, ArMDE:4618 (line-parallel):
> Ein Partner darf ohne Erwerb der entsprechenden Tugend in jeder der Rollen des Hauses agieren – außer in der Rolle des Capo –, mit Genehmigung der Spieltruppe. Das heißt, **ein Partner, der zugleich Faktor, Handelsabenteurer, lokaler Frachtführer oder städtischer Kaufmann ist, muss diese Tugend nicht kaufen, wenn er diese eine besitzt.**

**Current data:** `"classification": "narrative"`, no `effects`. Both locales
carry only "The character has a large financial stake in a wealthy trading
company." / "Der Charakter hat einen großen finanziellen Anteil an einem
wohlhabenden Handelsunternehmen."

**Why it is wrong.** B06 followed this passage from the *other* side
(`virtue.merchant_adventurer`, whose "should select the Partner Virtue instead
of this one" it correctly rated hedged guidance) and recorded that the rule
"belongs to Partner, not here", flagging ArMDE:4616-4619 for this batch. Read
from this side it is unambiguously a rule: it exempts the holder from *buying*
four named Social Status Virtues, which is a statement about what the character
legally is without paying the points. "need not purchase" is not a hedge. The
"save the role of capo" carve-out and the troupe-permission gate are part of it.

**Correct value.** `"classification": "uncomputed_rule"` with both sentences
written into `description` in **both** locales. The waiver is not expressible as
data — it is the *absence* of a requirement, and no `Effect` variant grants
"counts as holding X without holding X".

**Severity.** Lost rule.

**ArMDE:2816 instance.** Noted only; no fix proposed.

---

### F-213 — `virtue.perfect_balance` — "+6" classified `narrative`

**Passage** (ArMDE:4626, verbatim, the entire entry body):
> You are skilled at keeping your balance, especially on narrow ledges or tightropes. **Add +6 to any roll to avoid falling or tripping.**

German, ArMDE:4626 (line-parallel):
> Du beherrschst es, dein Gleichgewicht zu halten, besonders auf schmalen Vorsprüngen oder Seilen. **Addiere +6 auf jeden Wurf, um ein Fallen oder Stolpern zu vermeiden.**

**Current data:** `"classification": "narrative"`, no `effects`. Both locales
carry only the first sentence as `summary`; no `description` in either.

**Why it is wrong.** A signed modifier in plain digits. The
`Classification` doc is explicit that "an entry whose text states a signed
modifier … is **not** narrative even when the engine computes nothing". The
engine genuinely cannot compute it — `ability_roll_mod` (A41) is the nearest
variant and is surfaced-only, and the modifier is not scoped to a named Ability
anyway ("any roll to avoid falling or tripping") — so D3 applies.

**Correct value.** `"classification": "uncomputed_rule"` with "+6 to any roll to
avoid falling or tripping" written into `description` in **both** locales,
ASCII sign.

**Severity.** Lost rule.

---

### F-214 — `virtue.perfect_eye_for_commodity` — an absolute and a formula classified `narrative`

**Passage** (ArMDE:4630, verbatim, first and last sentences):
> For one commodity, and products manufactured from it, **the character never fails to make an accurate assessment of value.** … If you are using the trading rules in City and Guild, they gain an extra **(3 x Wealth Multiplier) Labor Points per year.**

German, ArMDE:4630 (line-parallel):
> Für eine bestimmte Ware und daraus hergestellte Produkte **kann der Charakter niemals eine ungenaue Werteinschätzung vornehmen.** … Wenn du die Handelsregeln aus *Stadt & Gilde* verwendest, erhalten sie zusätzliche **(3 × Wohlstandsmultiplikator) Arbeitspunkte pro Jahr.**

**Current data:** `"classification": "narrative"`, no `effects`, one `text`-domain
parameter `commodity`. Both locales carry the first sentence as `summary`; no
`description` in either.

**Why it is wrong.** "never fails" is an **absolute** — the `Classification` doc
names a cap and an absolute alongside a signed modifier as things that make
`narrative` false. The entry also states a per-year formula, which the summary
drops. The formula plugs into *City and Guild*, which is not in
`rules/source/en/`, so it cannot be implemented (CLAUDE.md) — but the *rule* is
stated in ArMDE's own text, so the entry owes it as text.

**Correct value.** `"classification": "uncomputed_rule"` with the absolute and
the Labor Point formula written into `description` in **both** locales.

**Severity.** Lost rule.

*(The `commodity` parameter is consumed by no effect. Per B8 that is the normal
and intended shape for a parameter recording a player choice the engine does not
compute from, and is **not** a finding here: the entry's own mechanic is
per-commodity and nothing else could consume it.)*

---

### F-215 — `virtue.perfectus` and `virtue.performance_magic` — one `source.lines` range swallows the other's worked example (check 1)

**The five lines in dispute**, ArMDE:4636-4640 (verbatim):
> \> #### Example
> \>
> \> Orlando is a musician and magus, and knows Sorcerous Music and Subtle Magic. He may cast spells while he sings (with no penalty for no gestures), plays his lute, or sings and plays. If he sings or plays quietly, he suffers the normal –5 penalty. He can cast spells silently, but only by not using Sorcerous Music.
> \>
> \> His friend Furioso, another Sorcerous Musician, lacks Subtle Magic. If he has no instrument with him, then, unlike Orlando, he must use normal Hermetic gestures or suffer the penalty for no gestures.

German, line-parallel at ArMDE:4636-4640, same block ("Orlando ist Musiker und
Magus und kennt Zauberhaften Gesang und Subtile Magie. …").

**Whose it is.** ArMDE:4686-4688, inside `virtue.performance_magic`'s own box:
"**#### Sorcerous Music (Performance Magic)** … **Performance Magic (Music) is
often known as Sorcerous Music.**" The example is about Sorcerous Music, Subtle
Magic and replacing gestures — three Performance Magic concepts. It has nothing
to do with Cathar heretics.

**Current data:** `virtue.perfectus` `"lines": [4632, 4641]`;
`virtue.performance_magic` `"lines": [4642, 4709]`.

**Why it is wrong.** Perfectus's own passage is ArMDE:4632-4634 (heading,
descriptor, one body paragraph). The range extends to 4641 because the batch's
convention is "own heading → the line before the next heading", and the example
block is **physically misordered in the Markdown** — it sits above the
`#### Performance Magic` heading instead of below it. The convention transcribed
the misordering. The consequence is a provenance error on **both** entries at
once: Perfectus cites five lines that are not its rule, and Performance Magic's
range excludes its own first worked example, which is the only place the source
states that a Sorcerous Musician "can cast spells silently, but only by not
using Sorcerous Music".

This is the mirror image of B06's F-200 (`virtue.minor_magical_focus`, a range
that stopped three lines *before* its own rule) and the audit's **second**
check-1 failure in seven batches.

**Correct value.** `virtue.perfectus` → `[4632, 4635]` (or `[4632, 4634]`,
matching the four tighter ranges elsewhere in this span);
`virtue.performance_magic` → the example block claimed, which cannot be done
with one contiguous `LineRange` while the block sits above the heading. Either
the source's block order is corrected (it is an OCR/layout artefact, and
`rules/source/` is redistributable under the Ars Magica Open License) or the
example is recorded as belonging to Performance Magic by some other means. No
open question is raised for this, because the choice is an editorial one for the
correction pass rather than a rules reading — but it is flagged here as needing
a decision, not a mechanical edit.

**Severity.** Provenance. No number moves (B11: `source` has no runtime
consumer), but a reader following Perfectus's citation lands on another Virtue's
rules, which is exactly the failure mode the `anchor` field exists to make loud.

---

### F-216 — `virtue.perfectus` — `narrative` on an entry that grants a gated Ability category, with no `ability_authorization`

**Passage** (ArMDE:4634, verbatim, the relevant clause):
> **Due to your training, you may take Academic Abilities during character creation.**

German, ArMDE:4634: "**Aufgrund deiner Ausbildung darfst du beim Erschaffen
Akademische Fertigkeiten nehmen.**"

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Identical to F-209's mechanism: `academic` is gated, and
`validate_ability_authorization` errors without permission.

**Correct value.** `"classification": "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}`.

**Severity.** Wrong rules output.

---

### F-217 — `virtue.perfectus` — "You may not take the Wealthy Virtue" is absent from `incompatible_with`

**Passage** (ArMDE:4634, verbatim):
> **You may not take the Wealthy Virtue,** as you are supported by the tithes and contributions of your congregations.

German, ArMDE:4634: "**Du darfst die Tugend Wohlhabend nicht nehmen,** da du von
den Zehnten und Beiträgen deiner Gemeinden unterstützt wirst."

**Current data:** `virtue.perfectus` has no `incompatible_with` key at all.
`virtue.wealthy` (ArMDE:5235-5238) likewise carries none.

**Why it is wrong.** "may not take" is an absolute exclusion, and
`incompatible_with` is exactly the field for it (B7). `validate_incompatibilities`
would raise `incompatible` — an **error** — which is the right outcome.

**Correct value.** `virtue.perfectus.incompatible_with` gains
`"virtue.wealthy"` **and** `virtue.wealthy.incompatible_with` gains
`"virtue.perfectus"` — the declaration is load-enforced symmetric
(`ruleset/integrity.rs::validate_incompatibility_symmetry`), so the correction
touches **both** entries or the ruleset fails to load.

**Severity.** Wrong rules output — the app permits an illegal combination.

*(The neighbouring sentence "You **should** normally take the Flaw: Vow" is
hedged guidance and is correctly encoded by encoding nothing — the same reading
B06 gave `virtue.mendicant_friar`'s Monastic Vows suggestion.)*

---

### F-218 — `virtue.perfectus` — the Purity/Transcendence permission reaches neither locale (D5)

**Passage** (ArMDE:4634, verbatim):
> **You may take the Purity and Transcendence Supernatural Abilities from Realms of Power: The Divine Revised Edition (page 53) if your character has the True Faith Virtue, but these are not free Virtues.** You may, however, take them as Virtues, as normal.

German, ArMDE:4634:
> **Du kannst die Übernatürlichen Fertigkeiten Reinheit und Transzendenz aus *Sphären der Macht: Das Göttliche (Überarbeitete Ausgabe)* (Seite 53) nehmen, wenn dein Charakter die Tugend Echter Glaube besitzt; dies sind jedoch keine freien Tugenden.** Du darfst sie jedoch als Tugenden nehmen, wie normal.

**Current data:** no `description` in either locale; no effect.

**Why it is wrong.** This is a conditional permission with an explicit
cost caveat — two mechanical clauses. Unlike B06's *Cradle and the Crescent*
pointer, **RoP:D is in `rules/source/en/`**, so the rule is properly sourced and
not barred by CLAUDE.md's provenance rule. What blocks encoding it is that
`ability.purity` and `ability.transcendence` are not in the catalogue — searched
the whole `abilities` array of `rules/core/abilities.json` for `purity` and
`transcendence`, no match. → **Q-56.**

**Correct value.** Both clauses written into `description` in **both** locales.
Once the two Abilities exist, the permission is an
`{"type": "ability_authorization", "abilities": [...]}` gated by a
`Prereq::Has("virtue.true_faith")`, which is expressible — but `prerequisites`
gate *this* Virtue, not the permission it confers, so the conditionality would
still need text.

**Severity.** Lost rule.

---

### F-219 — `virtue.performance_magic` — a 68-line rules subsystem classified `narrative`

**Passage** — the entry runs ArMDE:4642-4709 and states, among others, these
five rules verbatim:

ArMDE:4646:
> When the Virtue is acquired, specify the Ability to which it applies. You may choose any Craft, Profession, or other Ability with a clear verbal or physical practice. **You may not choose any Language, Supernatural, Academic, or Arcane Ability.**

ArMDE:4658, :4660 (a new spell Duration and its roll):
> *Performance (Duration):* The spell lasts as long as the caster performs the Performance Ability. Performance is equivalent to Concentration Duration.
> To cast a Performance Duration spell, the magus must succeed in a roll (simple or stress die, according to circumstances) of **Characteristic (varies with Ability) + Ability against an Ease Factor of 3.**

ArMDE:4684:
> If the magus is in combat, then he may only use Performance Magic (Brawl or Martial Abilities) if the player rolls a **stress die with three extra botch dice, and treats all nonbotches as die rolls of zero.**

ArMDE:4702-4708 (a full Ease-Factor table):
> **DETERMINING THE FORM OF A MAGICAL EFFECT: Stress Die + Perception + Awareness vs. 15 – effect magnitude**
> -3 if Hermetic words are spoken (and can be heard)
> +0 if Hermetic gestures are made (and seen)
> +3 if both words and gestures are mundane

German, line-parallel at ArMDE:4646, :4658, :4660, :4684, :4702-4708:
> … **Du darfst keine Sprach-, Übernatürliche, Akademische oder Arkane Fertigkeit wählen.**
> *Aufführung (Dauer):* Der Zauber hält an, solange der Wirker die Aufführungsfertigkeit ausführt. Aufführung entspricht der Dauer Konzentration.
> Um einen Zauber mit Aufführungs-Dauer zu wirken, muss dem Magus ein Wurf gelingen (einfacher oder Stresswürfel, je nach Umständen) auf **Eigenschaft (variiert je nach Fertigkeit) + Fertigkeit gegen einen Schwierigkeitsgrad von 3.**
> … nur nutzen, wenn der Spieler einen **Stresswürfel mit drei zusätzlichen Patzerwürfeln** wirft und alle Nicht-Patzer als Würfelergebnis null wertet.
> **DIE FORM EINES MAGISCHEN EFFEKTS BESTIMMEN: Stresswürfel + Wahrnehmung + Wahrnehmung vs. 15 – Effektmagnitude**
> –3 wenn Hermetische Worte gesprochen werden (und gehört werden können)
> +0 wenn Hermetische Gesten ausgeführt werden (und gesehen werden können)
> +3 wenn sowohl Worte als auch Gesten gewöhnlich sind

**Current data:** `"classification": "narrative"`, no `effects`, no
`parameters`. Both locales carry the single scene-setting sentence at
ArMDE:4644 as `summary`; no `description` in either.

**Why it is wrong.** This is the largest single rules body in the batch — 68
lines, a new spell Duration, a die roll with a stated Ease Factor, an explicit
botch-dice rule and a three-row modifier table — and the entry asserts the book
states nothing mechanical. It is also the one entry in the span that touches
machinery the engine *has*: `special_casting_mod`'s `quiet_words` /
`subtle_gestures` kinds and `derived.rs::residual_voice_penalty` /
`::residual_gesture_penalty` model exactly the no-words / no-gestures penalties
that Performance Magic replaces (ArMDE:4650: "An Ability that is verbal … replaces
the words of spellcasting, similarly, an Ability that is physical … replaces
gestures"), and ArMDE:4654 names Quiet Magic by name as still applying to the
components Performance Magic does *not* replace.

**Correct value.** `"classification": "uncomputed_rule"` with the five clauses
above written into `description` in **both** locales. Whether the
words/gestures replacement should become a computed `special_casting_mod` kind
is a design question and is not proposed here — the replacement is
per-performance and scene-contingent (ArMDE:4652: "The magus must actually
perform the Ability to count as using this Virtue"), which is the shape A40's
surfaced-only kinds already occupy.

**Severity.** Lost rule — the largest by volume in the batch.

---

### F-220 — `virtue.performance_magic` — the Ability it names cannot be recorded, and it cannot be taken twice

**Passage** (ArMDE:4646, :4648, verbatim):
> **When the Virtue is acquired, specify the Ability to which it applies.** …
> **There are distinct Virtues for each possible Ability,** as each allows different methods to work its actions into magic: knowing one method of Performance Magic does not let the magus use any other Ability in magic.

German, ArMDE:4646, :4648:
> **Wenn die Tugend erworben wird, gib die Fertigkeit an, auf die sie sich anwendet.** …
> **Es gibt unterschiedliche Tugenden für jede mögliche Fertigkeit,** da jede andere Methoden erlaubt, ihre Handlungen in die Magie einzubinden: Das Kennen einer Methode der Aufführungsmagie erlaubt dem Magus nicht, irgendeine andere Fertigkeit für die Magie zu nutzen.

**Current data:** no `parameters`; `max_per_target` absent (defaults to **1**);
`max_total` absent (defaults to 255); both locales' `name` is the bare
"Performance Magic" / "Aufführungsmagie" with no `{…}` placeholder.

**Why it is wrong, on two counts.**

1. **The choice the passage requires the player to make is not storable.** B10
   records that `max_per_target`'s grouping key is `(item_ref, whole params
   map)`; with no params, every copy is the same target. The book's own naming
   convention throughout the entry is parenthesised — "Performance Magic (Hunt)"
   (ArMDE:4668), "Performance Magic (Music)" (ArMDE:4688), "Performance Magic
   (Brawl or Martial Abilities)" (ArMDE:4684) — and four other entries in this
   span do carry the placeholder (`Perfect Eye for {commodity}`,
   `Puissant {ability}`, `Puissant {art}`). This is B06's `magical_blood` shape
   (F-165): a player choice the save cannot hold.
2. **"There are distinct Virtues for each possible Ability" means several may be
   held**, and `max_per_target: 1` over an empty params map forbids a second
   copy. ArMDE:2814 ("A Virtue or Flaw may be taken more than once only if the
   description explicitly allows it") is satisfied here — the description does.

**Correct value.** A `{"key": "ability", "type": "ref", "domain": "ability"}`
parameter, `name` gaining the `{ability}` placeholder in both locales, and
`max_per_target` left at 1 (one copy *per Ability*, which is what the parameter
then makes it mean). The restriction "You may not choose any Language,
Supernatural, Academic, or Arcane Ability" is **not** expressible on a parameter
— `ParameterDef` has `require_categories` for the `item` domain only (B8), with
no `ability`-domain analogue — so it stays a `description` clause under F-219.

**Severity.** Data loss (the choice is unrecordable) plus wrong rules output
(a legal second copy is refused).

---

### F-221 — `virtue.personal_power` — four stated clauses reach neither locale (D5)

**Passage** (ArMDE:4716, :4720-4722, :4726, verbatim):
> The power has an **Initiative equal to the character's Quickness – (power magnitude/2)**. **It costs one Fatigue level to activate if its level is less than or equal to 50, and two Fatigue levels if its level is 51 to 100.** You may also spend levels one-for-one to give the power Penetration; otherwise, it has a Penetration of zero.
> The effect must meet at least one of the following criteria:
> **The effect must be Range: Personal,** OR **The effect must be constant**
> … **The power must be associated with the same supernatural realm as the system on which it is based.**

German, line-parallel at ArMDE:4716, :4720-4722, :4726:
> Die **Initiative der Kraft entspricht der Schnelligkeit des Charakters – (Magnitude der Kraft/2)**. **Sie kostet eine Erschöpfungsstufe zur Aktivierung, wenn ihre Stufe 50 oder weniger beträgt, und zwei Erschöpfungsstufen, wenn ihre Stufe zwischen 51 und 100 liegt.** …
> **Der Effekt muss Reichweite: Persönlich haben,** ODER **Der Effekt muss dauerhaft sein**
> … **Die Kraft muss derselben übernatürlichen Sphäre zugeordnet sein wie das System, auf dem sie basiert.**

**Current data:** one effect, `{"type": "power_levels", "amount": 25}`;
`max_per_target: 255`; no `description` in either locale.

**What the effect *does* get right, so the finding is precise.** The 25 levels
are correct (ArMDE:4716 "a level of 25 or lower"). `max_per_target: 255` is
correct — ArMDE:4724 "This Virtue may be taken more than once, and the levels
added together". And the Penetration clause is **already computed**: A27 records
`powers_used = Σ (power.level + power.penetration)`, and the book's own worked
example at ArMDE:4724 ("a power with a level of 30 and a Penetration of 0, and a
second power with a level and Penetration of 10 each") sums to exactly the 50
levels two copies grant. No finding there.

**Why the four remaining clauses are a finding.** None is implemented and none
is stated. The realm association is structurally inexpressible: `types.rs`'s
`SupernaturalPower` declares `name`, `level` and `penetration` and **no realm
field** — read in full — so D3 governs and `uncomputed_rule` is not available
either, because the entry legitimately carries an effect.

**Correct value.** All four written into `description` in **both** locales.

**Severity.** Lost rule. The Range: Personal / constant requirement is the one
that bites hardest — it is the only thing distinguishing this Minor Virtue from
`virtue.ritual_power`'s Major one at the same 25 levels.

**ArMDE:2960 instance** — a Supernatural Virtue whose realm association the
engine does not model. Per the brief, noted only.

---

### F-222 — `virtue.personal_vis_source` — the yield rule classified `narrative`

**Passage** (ArMDE:4730, verbatim, the entire entry body):
> You have exclusive access to a supply of raw vis. Determine the amount and type with the help of your troupe, **the yield should be about one tenth as much as the player covenant expects to gather per year at the beginning of the saga.** **The yield of your source does not normally change over the course of time,** even if the covenant uncovers new sources.

German, ArMDE:4730 (line-parallel):
> Du hast exklusiven Zugang zu einer Versorgung mit rohem Vis. Lege Menge und Art in Absprache mit Deiner Spieltruppe fest; **der Ertrag sollte etwa ein Zehntel dessen betragen, was der Spielerkonvent zu Beginn der Saga jährlich zu gewinnen erwartet.** **Der Ertrag Deiner Quelle verändert sich im Laufe der Zeit normalerweise nicht,** selbst wenn der Konvent neue Quellen erschließt.

**Current data:** `"classification": "narrative"`, no `effects`. Both locales
carry only the first sentence as `summary`.

**Why it is wrong.** The passage states a **quantity** — one tenth of the
covenant's annual yield — and a persistence rule. That is a rule stated with a
number, arrived at by troupe agreement, which is precisely the
`uncomputed_rule` definition ("GM judgement, open-ended magnitudes"), not the
`narrative` one.

**The hedge is real and is stated here rather than suppressed.** "should be
about" and "does not *normally* change" are both softened, and B06 rated
similarly hedged sentences (`virtue.merchant_adventurer`'s "may be represented
by the Favors Flaw") as guidance rather than rules. The distinction drawn here
is that those hedged sentences carried **no number**, and this one does. →
**Q-60** escalates the reading rather than resolving it.

**Correct value.** `"classification": "uncomputed_rule"` with the one-tenth rule
and the no-change rule written into `description` in **both** locales.

**Severity.** Lost rule.

---

### F-223 — `virtue.physician_of_salerno` — a stated prerequisite is encoded nowhere (D5)

**Passage** (ArMDE:4734, verbatim, the final sentence):
> **To take this Virtue, you must be able to take Academic Abilities.**

German, ArMDE:4734: "**Um diese Tugend zu wählen, musst Du Akademische
Fertigkeiten erlernen dürfen.**"

**Current data:** no `prerequisites` key; two effects
(`grants_reputation` local 2, and a `restricted_ability_xp` of 50 over
`ability.medicine` + `ability.philosophiae`); no `description` in either locale.

**Why it is wrong.** This is a *gate on taking the Virtue*, not a permission the
Virtue confers — and the direction matters, because the entry's own
`restricted_ability_xp` pool **grants** exactly the permission the sentence
presupposes (A7: a pool is permission for what it funds). So today the Virtue
bootstraps its own prerequisite: a grog with no Educated, no Warrior and no
Social Status may take Physician of Salerno and legally hold Medicine and
Philosophiae, which the passage forbids.

It is not expressible as a `Prereq`: B6 lists `has` / `house` / `ability_min` /
`art_min` / `is_magus` and their boolean combinators, and none tests whether a
*category* is authorized. Enumerating every academic-granting Virtue in an `any`
would be an incomplete list by construction (the catalogue grows as data).
D3 therefore governs.

**Correct value.** The sentence written into `description` in **both** locales.
If the engine later grows a `Prereq::AbilityCategoryAuthorized`, this is its
first customer.

**Severity.** Wrong rules output — the app permits a character the passage
forbids — plus a lost rule.

**Q-55** is raised separately on the Reputation's type.

---

### F-224 — `virtue.piercing_gaze` — "+3" and an absolute exemption classified `narrative`

**Passage** (ArMDE:4738, verbatim, the entire entry body):
> By staring intently at people you make them feel uneasy, as if you are peering into their souls. **Those with ulterior motives, uneasy consciences, or lying tongues must make rolls against an appropriate Personality Trait, Guile, or whatever the storyguide deems appropriate, to remain calm.** Furthermore, **you gain a +3 to rolls involving intimidation. Faeries and demons are unfazed by your power.**

German, ArMDE:4738 (line-parallel):
> Wenn Du Menschen eindringlich anschaust, vermittelst Du ihnen das unbehagliche Gefühl, als würdest Du in ihre Seelen blicken. **Personen mit Hintergedanken, schlechtem Gewissen oder einer verlogenen Zunge müssen Würfe gegen eine angemessene Persönlichkeitseigenschaft, Täuschung oder was der Spielleiter für geeignet hält ablegen, um ruhig zu bleiben.** Außerdem **erhältst Du einen Bonus von +3 auf Würfe, die Einschüchterung beinhalten. Feen und Dämonen lässt Deine Kraft unbeeindruckt.**

**Current data:** `"classification": "narrative"`, no `effects`. Both locales
carry only the first sentence as `summary`.

**Why it is wrong.** Three mechanical clauses: a compelled roll, a signed +3,
and an absolute immunity for two creature types. The summary keeps only the
flavour sentence, so all three leave the application.

**Correct value.** `"classification": "uncomputed_rule"` with all three written
into `description` in **both** locales.

**Severity.** Lost rule.

---

### F-225 — `virtue.potent_magic_major` / `_minor` — the Casting-Score bonus is folded into *every* Casting Total, though the passage scopes it to the field

**Passage** — see the directed read above. The operative words are ArMDE:4744
("Potent Magic provides the maga with a bonus **in her field of magic**" /
"Potente Magie verschafft der Maga einen Bonus **in ihrem Magiebereich**") and
ArMDE:4746/:4748 ("grants a +3 / +6 bonus to Lab Totals **and Casting Score**").

**Current data (both entries):**
```
{"type": "casting_total_mod", "amount": 3|6, "scope": "all"}
{"type": "lab_total_mod",     "amount": 3|6}
```

**Why it is wrong.** A30 records that `derived.rs::InPlayMods::casting_mod_for`
sums every `casting_total_mod` whose scope matches the cast type, and `all`
matches Formulaic, Ritual and Spontaneous alike; the sum then enters
`derived/casting.rs::casting_totals` as an unconditional addend on **every**
Technique×Form cell and flows on into `::formulaic_casting_score` and
`::penetration`. A magus with Major Potent Magic in, say, "the destruction of
stone" therefore reads +6 on his Creo Corpus healing spells.

**This is D4's shape on the total D4's table does not cover.** D4 rules the
`lab_total_mod` half explicitly — "not in `total`; belongs in `within_focus`" —
and that half is *not* re-reported here. The casting half comes from the **same
sentence** with the **same condition**, and the engine already has the same
receptacle: `derived/casting.rs` declares `CastingTotal::within_focus` and
`PenetrationLine::within_focus` beside the ordinary figures, exactly as
`derived/lab.rs` declares `LabTotal::within_focus`. Both are gated on
`mods.has_focus`, which is the one wrinkle — a Potent Magic maga need not have a
Magical Focus at all (ArMDE:4742: "The benefits of Potent Magic are compatible
with a Magical Focus", i.e. they are separate things), so the within-focus
figures would have to become reachable without one.

**Correct value.** The +3/+6 belongs on the within-field figure, not on `total`
— on **both** the casting and the lab side. The condition, recorded for D4's
Phase 2 slice, is in the directed-read table above.

**Severity.** Wrong rules output. Every Casting Total, every Penetration line
and (per D4) every Lab-Total-derived figure a Potent Magic maga sees is
currently 3 or 6 too high outside her field.

---

### F-226 — `virtue.potent_magic_major` / `_minor` — `incompatible_with` contradicts ArMDE:4742, and a load-time guard forces it

**Passage** (ArMDE:4742, verbatim):
> … **a maga may have more than one area of Potent Magic,** although only one Potent Magic Virtue applies to any single activity.

German, ArMDE:4742: "… **anders als ein Magischer Fokus kann eine Maga jedoch
mehr als einen Bereich Potenter Magie besitzen,** wenngleich nur eine einzige
Potente-Magie-Tugend auf eine bestimmte Tätigkeit angewendet werden kann."

**Current data:** `virtue.potent_magic_major.incompatible_with =
["virtue.potent_magic_minor"]` and the mirror.

**Why it is wrong.** The sentence is an explicit *permission* to hold more than
one, drawn as a contrast with Magical Focus, which the very next clause confirms
by speaking of "only one Potent Magic Virtue applies to any single activity" —
a stacking rule, which presupposes several. `validate_incompatibilities` raises
`incompatible`, a hard **error**, on a maga holding one of each.

**And the declaration is not a free choice.**
`ruleset/integrity.rs::validate_magnitude_variant_exclusivity` pairs
`<stem>_major` with `<stem>_minor` and **fails the whole ruleset load** unless
each lists the other. Read in full: it iterates `self.point_items`, resolves
`minor_variant_sibling(major_id)`, and pushes
`"magnitude variants '…' and '…' must be mutually incompatible_with each other"`
when either side omits the other. `ruleset.rs::minor_variant_sibling` was read in
full and the match is exact: it splits `virtue.potent_magic_major` at the `.`,
strips the `_major` suffix from the name, and returns
`virtue.potent_magic_minor` — which exists, so the pair is checked. So deleting
the two ids is not enough; the ruleset would stop loading. → **Q-57.**

Note the contrast the guard was built for and which holds everywhere else:
ArMDE:4405 and :4542 state that a magus may have **only one** Magical Focus
"either major or minor". Potent Magic is the explicit counter-example, in one
sentence, and the guard has no exemption for it.

**Correct value.** The two entries must be mutually compatible. Achieving that
requires either an exemption in
`validate_magnitude_variant_exclusivity` or ids outside the `_major`/`_minor`
stem pattern.

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-227 — `virtue.potent_magic_major` / `_minor` — the field cannot be recorded, and a second area cannot be taken

**Passage** — ArMDE:4742 as quoted in F-226, plus ArMDE:4744 ("a bonus **in her
field of magic**") and ArMDE:4750 ("A maga with Potent Magic may also invent new
Potent spells **within the field of her Potent Magic**").

**Current data (both entries):** no `parameters`; `max_per_target` absent
(defaults to **1**); `max_total` absent (defaults to 255); both locales' names
are "Potent Magic (Major)" / "(Minor)" with no field placeholder.

**Why it is wrong, on two counts.**

1. **The field is a player choice the save cannot hold.** Every rule the entry
   states is scoped to it ("in her field of magic", "within the field of her
   Potent Magic", "only one Potent Magic Virtue applies to any single
   activity"), and there is nowhere to write it down. `virtue.major_magical_focus`
   and `virtue.minor_magical_focus` — which ArMDE:4746/:4748 name as the
   *definition* of the two field breadths — both carry a `text`-domain parameter
   for exactly this, consumed by `Effect::MagicalFocus`'s `param` (A29). Potent
   Magic carries none.
2. **A second area cannot be taken.** With no params, B10's
   `(item_ref, whole params map)` key makes every copy the same target and
   `max_per_target: 1` refuses the second — while ArMDE:4742 says in both
   languages that a maga may have more than one area. ArMDE:2814's "only if the
   description explicitly allows it" is satisfied: it does.

**Correct value.** A `{"key": "field", "type": "ref", "domain": "text"}`
parameter on both entries, the name gaining the placeholder in both locales, and
`max_per_target` left at 1 (which the parameter then makes mean "one copy per
field"). Note this **also** fixes half of F-226: two Minor Potent Magics with
different fields become distinct targets, and only the cross-magnitude
`incompatible_with` would remain to be lifted.

**Severity.** Data loss (the field is unrecordable) plus wrong rules output
(a legal second area is refused).

---

### F-228 — `virtue.potent_magic_major` / `_minor` — the Potency subsystem and three further rules reach neither locale (D5)

**Passage** — ArMDE:4744-4780. The clauses no effect implements, verbatim:

ArMDE:4758-4760 (the arithmetic, as printed):
> **POTENCY SCORE:** Sum of Bonus from each Casting Item
> **POTENCY BONUS: Add Potency to Casting Score**

ArMDE:4762:
> … **a magus may not invent a Potent spell with a Potency higher than his Magic Theory score,** even with the aid of a Lab Text or teacher.

ArMDE:4764:
> **Any magus can learn a Potent spell from a Lab Text or teacher, and apply Potency to his Casting Score; the Potent Magic Virtue is required to design a Potent spell from scratch, but not to reproduce one from a Lab Text.**

ArMDE:4766:
> To cast a Potent spell, **the caster must touch the specified Casting Items.** … **If the caster does not have the Casting Items available, the spell cannot be cast at all.**

ArMDE:4770:
> **Potency applies only to Formulaic spells** — Spontaneous Magic allows Ceremonial Casting to produce similar results, and enchanted devices use Shape and Material Bonuses during enchantment.

ArMDE:4780:
> … **If the only difference is a change in Casting Items, he may invent the variant as if in possession of a Lab Text. If, for any reason, his current Lab Total is lower than the level of the spell, he may still invent the variant in a single season.**

ArMDE:4742 (the stacking rule): "**only one Potent Magic Virtue applies to any
single activity**" — and the compatibility statement "**The benefits of Potent
Magic are compatible with a Magical Focus**".

ArMDE:4746 / :4748 (the breadth definitions, imported by cross-reference from
ArMDE:4538 and ArMDE:4401 respectively): "the same **narrow** fields as a Minor
Magical Focus" / "the same **wide** fields as a Major Magical Focus".

German, line-parallel throughout — e.g. ArMDE:4760 "**POTENZBONUS: Addiere
Potenz zum Zauberwert**"; ArMDE:4770 "**Potenz gilt nur für Formulaische
Zauber**"; ArMDE:4766 "**Stehen dem Wirkenden die Zauberobjekte nicht zur
Verfügung, kann der Zauber überhaupt nicht gewirkt werden.**"

**Current data:** two effects per entry (the +3/+6 pair); no `description` in
either locale; both summaries carry only ArMDE:4742's opening clause.

**Why it is wrong.** None of the above is implemented — the engine has no
Casting Item, no Potency and no spell-design step — and none of it is written
down. Two of them are the *only* things that distinguish the two entries from
each other beyond the number (the breadth definitions), which is B06's F-174 /
F-201 complaint about Major vs Minor Magical Focus recurring on their Potent
twins: a player choosing between a 1-point and a 3-point Virtue is told nothing
about what he is buying.

**Correct value.** All of the above written into `description` in **both**
locales.

**Severity.** Lost rule, and a buying-decision defect for the breadth clause.

---

### F-229 — `virtue.powerful_relic` — the relic's power, the impiety rule and the built-into-an-item clause reach neither locale (D5)

**Passage** (ArMDE:4784, :4786, verbatim):
> You own an unusually powerful relic with a True Faith score of 3. **The relic also has one power, which should be agreed upon with the storyguide (see Relics, page 419). As with the Minor General Virtue Relic, the item may be built into any other item that you possess,** like a sword or a pendant.
> Owning such a powerful relic is a great responsibility, and your character should behave in an appropriate manner. **If you ever behave impiously (as judged by the storyguide) your relic will cease to function until suitable penance is made.**

German, ArMDE:4784, :4786 (line-parallel):
> Du besitzt ein ungewöhnlich mächtiges Relikt mit einem Wahrer-Glaube-Wert von 3. **Das Relikt besitzt außerdem eine Kraft, die in Absprache mit dem Spielleiter festgelegt werden sollte (siehe Relikte, Seite 419). Wie bei der Kleinen Allgemeinen Tugend Relikt kann der Gegenstand in jeden anderen Gegenstand eingebaut werden, den Du besitzt,** etwa in ein Schwert oder ein Amulett.
> … **Wenn Du Dich jemals gottlos verhältst (nach Beurteilung des Spielleiters), hört das Relikt auf zu funktionieren, bis angemessene Buße geleistet wurde.**

**Current data:** one effect, `{"type": "true_faith_grant", "score": 3}`. No
`description` in either locale; both summaries stop after the first clause.

**The score is right and the *subject* is wrong** — this finding covers only the
missing text; the effect itself is **F-256**, which the verification pass found
and which applies to this entry and `virtue.relic` alike. The first pass rated
this effect "correct" on the strength of ArMDE:4784's "a True Faith score of 3"
matching `score: 3`, without following the entry's own Relics cross-reference.

**Why it is wrong.** Three clauses no effect implements. "The relic also has one
power" is the sharpest: the character gains a supernatural power with no level
budget behind it, and `effective::powers_used` charges every `Entity::powers`
row against `power_levels_budget`, which for this Virtue is 0. Whether a
*relic's* power is the bearer's power in that sense is genuinely unclear from
the source → **Q-62**. The conditional shutdown ("will cease to function until
suitable penance is made") is a real rule with a storyguide trigger — the
`uncomputed_rule` shape exactly — and the built-into-an-item clause is the one
imported by cross-reference from `virtue.relic` (ArMDE:4854 states it too, and
that entry does not carry it either — see F-257, which owes it there).

**Correct value.** All three written into `description` in **both** locales.

**Severity.** Lost rule.

---

### F-230 — `virtue.prestigious_student` — `narrative` on an entry that grants a gated Ability category, with no `ability_authorization`

**Passage** (ArMDE:4794, verbatim, the relevant clause):
> He was trained in Artes Liberales and Latin by a private tutor, and **may purchase Academic Abilities at character generation.**

German, ArMDE:4794: "Er wurde von einem Privatlehrer in Artes Liberales und
Latein unterrichtet und **kann bei der Charaktererschaffung Akademische
Fertigkeiten erwerben.**"

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Same mechanism as F-209 and F-216.

**Correct value.** `"classification": "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}`.

**Note on what is *not* claimed.** "He was trained in Artes Liberales and Latin
by a private tutor" grants no experience points and no scores — the passage
states no number, unlike `virtue.educated` (ArMDE:3711-3713) which grants 50 XP
over `ability.artes_liberales` + `ability.dead_language`. So the correct variant
here is the bare `ability_authorization`, not a `restricted_ability_xp`.

**Severity.** Wrong rules output.

---

### F-231 — `virtue.prestigious_student` — the Social-Status constraint reaches neither locale (D5)

**Passage** (ArMDE:4794, verbatim, the final sentence):
> **The character must take a Social Status Virtue to reflect where he is in the educational process.**

German, ArMDE:4794: "**Der Charakter muss eine Tugend des Sozialen Status
wählen, um seinen Stand im Bildungsweg widerzuspiegeln.**"

**Current data:** no `description` in either locale.

**Why it is wrong.** ArMDE:2816 already requires *every* character to take one
Social Status, so the bare requirement adds nothing — but the qualification does:
the Social Status must be one that reflects a stage of the **educational
process**, which narrows the choice to the scholastic ladder (Student,
Baccalaureus, Magister in Artibus, …). That narrowing is a rule, it is stated
with "must", and it is not derivable from ArMDE:2816.

**Correct value.** The sentence written into `description` in **both** locales.

**Severity.** Lost rule, low.

**ArMDE:2816 instance.** Noted only; no fix proposed.

---

### F-232 — `virtue.priest` — `narrative` on an entry that grants a gated Ability category, with no `ability_authorization`

**Passage** (ArMDE:4804, verbatim, the opening clause):
> **You may purchase Academic Abilities during character generation.**

German, ArMDE:4804: "**Du kannst bei der Charaktererschaffung Akademische
Fertigkeiten erwerben.**"

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Same mechanism as F-209.

**Correct value.** `"classification": "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}`.

**Severity.** Wrong rules output.

---

### F-233 — `virtue.priest` — a conditional Poor exclusion, a male-only restriction and a Social-Status compatibility reach neither locale (D5)

**Passage** (ArMDE:4800, :4804, verbatim):
> You may be a parish priest. **If you are, you cannot take the Poor Flaw.** …
> … **This Virtue is only available to male characters, and is compatible with the Magister in Artibus Major Virtue,** and with some other Virtues, as noted in their descriptions.

German, ArMDE:4800, :4804 (line-parallel):
> Du kannst ein Pfarrpriester sein. **Wenn Du es bist, kannst Du nicht den Fehler Arm wählen.** …
> … **Diese Tugend steht nur männlichen Charakteren zur Verfügung und ist mit der Großen Tugend Magister in Artibus vereinbar,** sowie mit einigen anderen Tugenden, wie in deren Beschreibungen vermerkt.

**Current data:** no `incompatible_with`, no `description` in either locale.

**Why it is wrong, on three counts.**

1. **The Poor exclusion is conditional on a choice the save cannot hold.**
   "Parish priest or not" is a fork the passage offers (ArMDE:4800 vs :4802,
   "If you are not a parish priest, the Wealthy Virtue and Poor Flaw affect you
   normally") and the entry declares no parameter for it. So the exclusion
   cannot become an `incompatible_with` — that field is unconditional (B7) — and
   an unconditional declaration would be *wrong*, forbidding Poor to the
   non-parish priest the passage expressly permits it to. D3 governs: text.
2. **Male-only** — `types.rs`'s concept field declares `pub gender: String` with
   "no mechanical effect", the same ground as B05's F-123 and B06's four gender
   findings. This is the batch's second of four.
3. **The Magister in Artibus compatibility** is an explicit ArMDE:2816 exemption
   — it is precisely the "descriptions … explicitly note that they are
   compatible" clause — and it names `virtue.magister_in_artibus`, which exists
   ✓. B06 flagged the same entry from the other side (F-169) for missing
   prerequisites; neither side records the pairing.

**Correct value.** All three written into `description` in **both** locales.

**Severity.** Lost rule; the Poor exclusion additionally permits an illegal
combination for the parish-priest case.

**ArMDE:2816 instance.** Noted only; no fix proposed.

---

### F-234 — `virtue.privileged_upbringing` — the engine grants exactly the permission the passage forbids

**Passage** (ArMDE:4808, verbatim, the first two sentences):
> You have an additional 50 experience points, which may be spent on General, Academic, or Martial Abilities. **You may not, however, buy Academic or Martial Abilities with your normal pool of experience points unless you have another Virtue or Flaw permitting that.**

German, ArMDE:4808 (line-parallel):
> Du erhältst zusätzliche 50 Erfahrungspunkte, die für Allgemeine, Akademische oder Kampf-Fertigkeiten ausgegeben werden können. **Du kannst jedoch Akademische oder Kampf-Fertigkeiten nicht mit Deinem normalen Erfahrungspunkte-Vorrat erwerben, sofern Du keine andere Tugend oder keinen anderen Fehler hast, der dies erlaubt.**

**Current data:**
```
{"type": "restricted_ability_xp", "amount": 50,
 "categories": ["general", "academic", "martial"]}
```
No `description` in either locale.

**Why it is wrong.** The first sentence is modelled exactly right — A7's
`FlowPool` of capacity 50 over those three categories, drained in phase 1 before
the general pool opens. The second sentence is **inverted** by the same effect.
`effective/xp.rs::ability_authorizations` — read in full — folds
`Effect::RestrictedAbilityXp`'s `categories` into the character-wide
authorization set in the same match arm as `Effect::AbilityAuthorization`, with
the comment "Experience earmarked for a category or Ability is itself permission
to learn it — otherwise the grant could never be spent." That reasoning is sound
for Warrior and Educated, and **wrong for this one entry**, because this entry's
passage draws the distinction the engine's model does not have: permission is
scoped to *this pool*, not to the character.

The consequence is concrete and unbounded. `validate_ability_authorization` now
sees `academic` and `martial` authorized, so the character may hold them; and
`xp.rs::two_phase_max_flow`'s phase 2 opens the general pool to *any* Ability
spend. A companion with Privileged Upbringing and 250 general XP may put all of
it into Latin and Single Weapon. The passage forbids precisely that, and names
the only escape ("unless you have another Virtue or Flaw permitting that").

**Correct value.** Not proposable as a data edit: A7's permission side-effect is
hard-coded in `ability_authorizations` and there is no pool-scoped permission
shape in the engine. → **Q-59**. What is proposable today is the second sentence
written into `description` in **both** locales under D5, so the player is at
least told the rule the app is not enforcing.

**Severity.** **Wrong rules output, and the most permissive one in the batch** —
it silently unlocks two gated Ability categories for the character's entire XP
budget. Note the direction: every other authorization finding in seven batches
has been the engine refusing something legal; this is the engine permitting
something illegal, which no test would catch because nothing errors.

---

### F-235 — `virtue.protection` — a granted Reputation the engine can express, carried by no effect

**Passage** (ArMDE:4812, verbatim, the final sentence):
> **You have a Reputation (good or bad, your choice) of level 3,** which could be higher if your protector is particularly great or well-known.

German, ArMDE:4812 (line-parallel):
> **Du hast eine Reputation (gut oder schlecht, nach Deiner Wahl) von Stufe 3,** die höher sein kann, wenn Dein Beschützer besonders bedeutend oder bekannt ist.

**Current data:** `"classification": "narrative"`, **no `effects` at all**.

**Why it is wrong.** `Effect::GrantsReputation` exists, is the most-used variant
in the catalogue (33 uses, A25), and its `kind` field is
`Option<ReputationType>` where `None` means "player-chosen type" — the exact
shape for a grant whose audience the passage leaves open. Without the effect,
`validation/scores.rs::validate_reputations` emits `reputation_not_granted`, a
hard **error**, for the Reputation the passage says the character *has*. The
function was read in full to confirm it: it builds `remaining` and `wildcard`
counters from `effective::reputation_grants`, decrements a matching concrete
slot then a wildcard slot for each `entity.reputations` row, and in the `else`
branch pushes `ValidationIssue::error(CODE_REPUTATION_NOT_GRANTED, …)`. With no
grants at all both counters are 0, so the `else` branch is the only reachable
one.

**The repo's own translation table corroborates it independently.**
`rules/source/de/translation-tables/reputationen.md:80`:
`| Protection | Schutz | Frei wählbar | 3 (+/–) | Abhängig vom Schirmherrn |` —
"freely choosable", level 3. A hand-curated table already records what the
effect should say.

**Correct value.** `"classification": "creation_effect"` plus
`{"type": "grants_reputation", "score": 3}` with `kind` omitted (the wildcard).

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-236 — `virtue.protection` — the "could be higher" clause and the good/bad choice reach neither locale (D5)

**Passage** — ArMDE:4812 as quoted in F-235: "(good or bad, your choice)" and
"**which could be higher if your protector is particularly great or
well-known**".

**Current data:** no `description` in either locale.

**Why it is wrong, and why it matters beyond this entry.** `types.rs::Reputation`
declares `kind`, `score` and a free-text `content` and **no valence field** —
read in full — so "good or bad" is not modelled at all and can only travel as
text. And "could be higher" is direct evidence on **Phase 0's open question 3**
(is a granted Reputation's `score` a ceiling, a fixed value, or a suggestion?),
which `README.md` records as unanswered and assigns to whichever batch reads
`ArMDE:2512-2514`. That passage was read in this batch and does **not** answer it
— ArMDE:2514 says only "Characters only start with a Reputation if they choose a
Virtue or Flaw that grants one". **ArMDE:4812 does**, at least for this entry:
the granted 3 is a **baseline that may be exceeded**, not a ceiling and not a
fixed value. C2-c records that `validate_reputations` compares kinds and counts
only and never checks `Reputation.score` against `ReputationGrant.score` — which,
on this entry's evidence, is the **correct** behaviour rather than the gap Part C
implies. Recorded here as evidence toward Q3 rather than as a separate finding,
because one entry does not settle a catalogue-wide question.

**Correct value.** Both clauses written into `description` in **both** locales.

**Severity.** Lost rule, low — but the Q3 evidence is the valuable part.

---

### F-237 — `virtue.puissant_ability` — the learning/writing/helping exclusion reaches neither locale (D5)

**Passage** (ArMDE:4816, verbatim, the entire entry body):
> You are particularly adept with one Ability, and add 2 to its value whenever you use it. **Note that you do not, in general, use an Ability when learning it, writing about it, or helping someone else to improve it.** You may only take this Virtue once for a given Ability, but may take it more than once for different Abilities.

German, ArMDE:4816 (line-parallel):
> Du bist in einer bestimmten Fertigkeit besonders geschickt und addierst 2 zu ihrem Wert, wann immer Du sie einsetzt. **Beachte, dass Du eine Fertigkeit im Allgemeinen nicht einsetzt, wenn Du sie lernst, über sie schreibst oder jemandem hilfst, sie zu verbessern.** Du kannst diese Tugend für eine bestimmte Fertigkeit nur einmal wählen, aber für verschiedene Fertigkeiten mehrfach.

**Current data:** `{"type": "ability_bonus", "param": "ability", "amount": 2}`,
one `ability`-domain parameter, `max_per_target` and `max_total` both at their
defaults. Both locales' `summary` is the bare "+2 to all rolls with one Ability."
/ "+2 auf alle Würfe mit einer Fertigkeit."; no `description` in either.

**What is right, so the finding is precise.** The +2 ✓ (ArMDE:4816). The
repeat rules are modelled exactly: B10's `max_per_target` groups on
`(item_ref, whole params map)`, so the default 1 forbids two copies naming the
same Ability ("only take this Virtue once for a given Ability") while the
default `max_total` of 255 permits copies naming different ones ("may take it
more than once for different Abilities"). A1 also records that the XP charge
prices the **bought** score, so the bonus is already free at learning time,
which is half of the excluded clause honoured by accident of design.

**Why the clause is still a finding.** The other two thirds are not honoured and
not stated: the bonus reaches every derived total through
`effective_ability_score`, including any writing or teaching total a later slice
computes, and — more immediately — it satisfies `Prereq::AbilityMin`, which B6
records is "compared against the max **effective** score (bought + Puissant), so
a boosted Ability satisfies it". Whether a prerequisite counts as "using" the
Ability is exactly the kind of thing this sentence exists to decide, and the
player is not told the sentence exists.

**Correct value.** The sentence written into `description` in **both** locales.

**Severity.** Lost rule.

---

### F-238 — `virtue.puissant_art` — the requisite rule and the learning/teaching/writing exclusion reach neither locale (D5)

**Passage** (ArMDE:4820, verbatim, the entire entry body):
> You add 3 to the value of one Art whenever you use it. This means all totals in which the score of the Art is part of the total. **It does not apply when learning, teaching, or writing about the Art.** You may take this Virtue twice, for two different Arts. **If a spell has requisites, include the bonus from Puissant Art with that Art when calculating which Art is higher. If the Puissant Art is higher, the bonus does not apply to the requisite.**

German, ArMDE:4820 (line-parallel):
> Du addierst 3 zum Wert einer Kunst, wann immer Du sie einsetzt. Das bedeutet alle Summen, in denen der Wert der Kunst Teil der Summe ist. **Es gilt nicht beim Lernen, Unterrichten oder Schreiben über die Kunst.** Du kannst diese Tugend zweimal wählen, für zwei verschiedene Künste. **Wenn ein Zauber Requisiten hat, beziehe den Bonus aus Begabung in (Kunst) mit dieser Kunst ein, wenn Du berechnest, welche Kunst höher ist. Wenn die Kunst mit Begabung höher ist, gilt der Bonus nicht für das Requisit.**

**Current data:** `{"type": "art_bonus", "param": "art", "amount": 3}`, one
`art`-domain parameter, `"max_total": 2`. Both locales' `summary` carries "+3 to
all totals using one Art. May be taken twice, for two Arts."; no `description`
in either.

**What is right.** The +3 ✓. `max_total: 2` ✓ ("You may take this Virtue twice")
combined with the default `max_per_target: 1` ✓ ("for two **different** Arts") —
this is the pattern the brief asks about under "repeat limits and instance
scope", and here it is **correct on both axes**, which is worth recording
because the same pair is wrong on `virtue.performance_magic` (F-220) and on
Potent Magic (F-227) in this same batch.

**Why the two clauses are a finding.** The requisite rule is a genuine piece of
arithmetic — it decides *which* Art a requisite-bearing spell scores from, and
it explicitly withholds the bonus in one branch. RULES.md records
requisite-Art reduction as an acknowledged approximation of
`effective/spell.rs::spell_level_cap`, so the engine does not compute requisites
at all; the rule is therefore inexpressible (D3) and owes its text.

**Correct value.** Both clauses written into `description` in **both** locales.

**Severity.** Lost rule.

---

### F-239 — `virtue.quiet_magic` — the booming-voice and Voice-Range clauses reach neither locale (D5)

**Passage** (ArMDE:4824, verbatim):
> You can cast spells using only a soft voice at no penalty, and at only a –5 penalty if you do not speak at all. **You gain no benefits from using your voice normally but gain the normal benefit for using a booming voice. The range of Voice Range spells is determined normally, based on how loud your voice is.**

German, ArMDE:4824 (line-parallel):
> Du kannst Zauber ohne Abzug mit nur leiser Stimme wirken und erhältst nur einen Abzug von –5, wenn Du überhaupt nicht sprichst. **Du erhältst keinen Vorteil davon, Deine Stimme normal einzusetzen, aber Du erhältst den normalen Bonus für den Einsatz einer schallenden Stimme. Die Reichweite von Zaubern mit Reichweite Stimme wird normal bestimmt, abhängig davon, wie laut Deine Stimme ist.**

**Current data:** `{"type": "special_casting_mod", "kind": "quiet_words"}`,
`"max_per_target": 2`. Both locales' `summary` carries the **first** sentence,
with the ASCII hyphen ✓.

**What is right, and it is exact.** A40 gives
`residual_voice_penalty = min(-10 + Σ quiet_words·5, 0)`. One copy →
−10 + 5 = **−5** ✓ matching "only a –5 penalty if you do not speak at all". Two
copies → −10 + 10 = 0, clamped, ✓ matching ArMDE:4826 "You may take this Virtue
twice, and eliminate the penalty altogether". `max_per_target: 2` ✓ and
ArMDE:2814 is satisfied by that same sentence. This is one of the most precisely
implemented entries the audit has seen.

**Why the two clauses are a finding.** Neither is implemented — the engine has no
loud-voice bonus and no Voice Range model — and neither is stated. The second is
not a restatement of the general rule: it is an *exception* to the exception,
telling the player that a Virtue about casting quietly does **not** shrink his
Voice Range, which is exactly the inference a player would otherwise draw.

**Correct value.** Both clauses written into `description` in **both** locales,
ASCII hyphen for any sign.

**Severity.** Lost rule.

---

### F-240 — `virtue.rabbi` — a mandatory companion Virtue and a male-only restriction, on a `narrative` entry

**Passage** (ArMDE:4832, verbatim, the entire second paragraph):
> **The rabbi must take the Educated (Hebrew) Virtue to provide the required Academic Abilities. This Virtue is only available to male characters.**

German, ArMDE:4832 (line-parallel):
> **Der Rabbi muss die Tugend Gebildet (Hebräisch) wählen, um die erforderlichen Akademischen Fertigkeiten zu erhalten. Diese Tugend steht nur männlichen Charakteren zur Verfügung.**

**Current data:** `"classification": "narrative"`, no `effects`, no
`prerequisites`. Both locales carry only the first paragraph's opening sentence
as `summary`.

**Why it is wrong.** "must take" is a hard requirement naming a specific Virtue —
`Prereq::Has` is exactly that shape — and "only available to male characters" is
the batch's third gender restriction. `narrative` asserts neither exists.

**Why the prerequisite cannot be encoded today.** `#### Educated (Hebrew)` is
ArMDE:3723 in the source, but the catalogue holds a single `virtue.educated`
(cited 3711-3713, granting 50 XP over `ability.artes_liberales` +
`ability.dead_language`) and **no `_hebrew` / `_bardic` / `_islamic` /
`_vernacular` variants** — searched `rules/core/virtues_flaws.json` for
`educated`, one hit. These are the **four known-missing entries** README records
as the closed `####`-census result, so this is a *consequence* of that gap and
not a new one. Recorded here because it makes Rabbi's requirement
un-encodable until those four exist, which the correction pass needs to know.

Note also that the requirement *interacts* with the ArMDE:2816 authorization
pattern in an instructive way: Rabbi is one of the very few Social Status
Virtues in this span that does **not** grant Academic access itself — it
delegates ("to provide the required Academic Abilities"). So the absence of an
`ability_authorization` on Rabbi is **correct**, and Rabbi is not counted among
this batch's eight authorization findings.

**Correct value.** `"classification": "uncomputed_rule"` with both sentences
written into `description` in **both** locales; and, once
`virtue.educated_hebrew` exists, `prerequisites: {"kind": "has", "value":
"virtue.educated_hebrew"}`.

**Severity.** Lost rule; the missing prerequisite additionally permits an
illegal character.

**ArMDE:2816 instance.** ArMDE:2956 lists Rabbi under "**Minor Virtues**" in the
Jewish Social Statuses table. Noted only; no fix proposed.

---

### F-241 — `virtue.rat_up_a_drainpipe` — an absolute and a named-Ability advantage classified `narrative`

**Passage** (ArMDE:4840, verbatim, the entire entry body):
> Using a mixture of acrobatics, climbing and dodging, **this character can run through urban areas as if unobstructed. This gives a substantial advantage in opposed Athletics rolls which represent being chased.**

German, ArMDE:4840 (line-parallel):
> Durch eine Mischung aus Akrobatik, Klettern und Ausweichen **kann dieser Charakter durch städtische Gebiete laufen, als ob er ungehindert wäre. Dies verschafft einen erheblichen Vorteil bei gegnerischen Athletik-Würfen, die eine Verfolgungsjagd darstellen.**

**Current data:** `"classification": "narrative"`, no `effects`. Both locales'
`summary` carries the first sentence only.

**Why it is wrong.** "as if unobstructed" is an absolute, and the second
sentence names a specific roll type (**opposed Athletics**) and a specific
circumstance (being chased) in which the character has an advantage the
storyguide must apply. That is a mechanical clause with no number, which is the
`uncomputed_rule` case, not the `narrative` one — `narrative` claims the book
states *nothing* mechanical.

**The absence of a number is real and is escalated rather than resolved.** B06
drew its line at entries stating "+3" or "Might ≤ 25"; this one states neither.
→ **Q-61.**

**Correct value.** `"classification": "uncomputed_rule"` with both clauses
written into `description` in **both** locales.

**Severity.** Lost rule, low.

---

### F-242 — `virtue.redcap` — three stated exclusions absent from `incompatible_with`

**Passage** (ArMDE:4850, verbatim, the opening two sentences):
> You are supported by your covenant, so **you cannot take the Wealthy Virtue or Poor Flaw. You may not take The Gift.**

German, ArMDE:4850 (line-parallel):
> Du wirst von Deinem Konvent versorgt, daher **kannst Du weder die Tugend Wohlhabend noch den Fehler Arm wählen. Du darfst nicht die Gabe besitzen.**

**Current data:** `virtue.redcap` has **no `incompatible_with` key at all**.
`virtue.the_gift` carries `["virtue.devil_child", "virtue.faerie_doctor",
"virtue.failed_apprentice", "virtue.nephilim", "virtue.spirit_votary"]` —
Redcap is not among them. `virtue.wealthy` and `flaw.poor` each carry none
relevant.

**Why it is wrong.** Three absolute exclusions ("cannot", "may not"), all three
naming ids that exist, all three exactly `incompatible_with`'s shape (B7). The
Gift one is the most consequential: ArMDE:4844 opens the entry with "Although
you do not have The Gift and cannot work Hermetic magic", and ArMDE:4844
continues "There are Gifted members of House Mercere, but they do not take this
Virtue, taking the Hermetic Magus Social Status (page 85) instead" — so a Gifted
Redcap is not merely discouraged, it is the wrong Virtue by construction. The
engine permits the combination silently.

**Correct value.** `virtue.redcap.incompatible_with` gains `"virtue.the_gift"`,
`"virtue.wealthy"` and `"flaw.poor"`, **and each of those three gains
`"virtue.redcap"`** — `validate_incompatibility_symmetry` fails the load
otherwise, so this correction touches four entries.

**Severity.** Wrong rules output — the app permits three illegal combinations.

---

### F-243 — `virtue.redcap` — the free Well-Traveled Virtue is granted nowhere

**Passage** (ArMDE:4848, verbatim):
> **In addition, you have the Well-Traveled Virtue (page 116) at no cost.**

German, ArMDE:4848: "**Darüber hinaus erhältst Du die Tugend Vielgereist (Seite
116) ohne Kosten.**"

**Current data:** one effect, `{"type": "item_level_budget", "amount": 50}`. No
`grants_selection`.

**Why it is wrong.** This is a **fixed, single-item, budget-exempt free grant** —
`Effect::GrantsSelection` (A18) is the variant for it, and the target
`virtue.well_traveled` exists ✓ (ArMDE:5239-5242, `minor`, `general`,
`narrative`). B06's F-208 found the identical shape on `virtue.nephilim` and
established the comparison: `virtue.faerie_doctor` and `virtue.spirit_votary`
both carry theirs, while `virtue.devil_child` correctly carries none because its
passage offers a *choice* and `grants_selection` names fixed ids only. Redcap's
passage names exactly one Virtue, so it is in Faerie Doctor's position.

The concrete cost: a Redcap must today either buy Well-Traveled with a Virtue
point the book says it does not cost, or go without it. And "without it" is not
cosmetic — ArMDE:5241 was read and Well-Traveled grants "**fifty bonus
experience points to spend on living languages, Area Lores, and Bargain,
Carouse, Charm, Etiquette, Folk Ken, or Guile**", so the missing grant is 50
experience points on top of F-244's 300.

**A flag for B09, found by following this pointer.**
`virtue.well_traveled` (ArMDE:5239-5242, index range 280-314) is itself
`"classification": "narrative"` with **no `effects`** — its data was read in the
same pass — while its passage states fifty experience points over an explicit
Ability list, which is `restricted_ability_xp`'s exact shape and is what
`virtue.educated` (ArMDE:3711-3713) carries for the identical sentence pattern.
That entry belongs to **B09** and is **not rated here**; it is recorded because
this batch had to read it to rate F-243, and because a correction pass fixing
Redcap's grant will be pointing it at an empty entry.

**Correct value.**
`{"type": "grants_selection", "items": ["virtue.well_traveled"]}` added to
`virtue.redcap.effects`.

**Severity.** Wrong rules output — a free Virtue is charged, or lost.

---

### F-244 — `virtue.redcap` — 300 apprenticeship experience points are granted nowhere

**Passage** (ArMDE:4848, verbatim):
> **You have spent fifteen years as an apprentice, and gained a total of 300 experience points in those fifteen years.** (See Detailed Character Creation, page 43.)

German, ArMDE:4848: "**Du hast fünfzehn Jahre als Lehrling verbracht und in
diesen fünfzehn Jahren insgesamt 300 Erfahrungspunkte gesammelt.** (Siehe
Detaillierte Charaktererschaffung, Seite 43.)"

**Current data:** the entry's only effect is `item_level_budget: 50`. No
`general_xp`, no `restricted_ability_xp`, no life-stage hook.

**The negative is stated exactly.** `grep -arn "redcap" rules/core/
crates/arm-rules/src/ crates/arm-app/src/ ui/src/` — run with **`-a`**, so a
NUL-bearing file could not be silently skipped — returns **six** lines and no
more: `virtue.lone_redcap`'s id,
`virtue.magic_items`'s `prerequisites: {"kind": "has", "value":
"virtue.redcap"}`, `virtue.redcap`'s own id, and three unrelated test-fixture
lines in `crates/arm-rules/src/validation/mod.rs` naming a type profile
`"ungifted_redcap"`. So **nothing in the rules data or in either crate's
production source** grants a Redcap experience points, and no character-type
profile is keyed to the Virtue.

**Why it is wrong.** 300 XP is the largest single number in the batch and one of
the largest in the catalogue — for comparison, `virtue.educated` grants 50 and
B06's `virtue.magister_in_medicina` 300 via a `restricted_ability_xp`. A Redcap
built in this app is 300 experience points poorer than the book's Redcap.

**Correct value.** Not obvious, and deliberately not asserted here as a single
answer: `{"type": "general_xp", "amount": 300}` would model it on the flat
XP-pool path, but A11 records that a magus's or companion's `base_general`
already comes from a life-stage plan when one exists, and the passage frames the
300 as a **fifteen-year apprenticeship** — which is the life-stage machinery's
own concept, not a flat bonus. Both readings give 300 points; they differ in
whether the points are earmarked. The correction pass must pick one against
`life_stage.rs`, which is outside this audit's remit.

**Severity.** **Wrong rules output, high** — a 300-point shortfall in the
character's entire experience budget.

---

### F-245 — `virtue.redcap` — three gated Ability categories granted, with no `ability_authorization`

**Passage** (ArMDE:4848, verbatim):
> You are trained in a similar manner to magi, and **may take Academic. Arcane, and Martial Abilities during character generation.**

German, ArMDE:4848: "Du wirst ähnlich wie Magi ausgebildet und **kannst bei der
Charaktererschaffung Akademische, Arkane und Kampf-Fertigkeiten erwerben.**"

*(The English prints a period where a comma belongs, "Academic. Arcane" — an OCR
artefact; the German has the comma and parses identically. Recorded so a later
reader does not take it for a data defect.)*

**Current data:** the entry's only effect is `item_level_budget: 50`.

**Why it is wrong.** This is the **whole gated set** —
`rules/core/abilities.json`'s `categories_requiring_virtue` is exactly
`["academic", "arcane", "martial"]`, and the passage grants all three. A Redcap
who buys Latin, Magic Lore or Single Weapon gets
`ability_category_requires_virtue`, a hard **error**, on every one of them. It is
the widest single authorization grant in the catalogue and it is absent.

**Correct value.**
`{"type": "ability_authorization", "categories": ["academic", "arcane",
"martial"]}`.

**Severity.** Wrong rules output — the app refuses a legal character, three
categories wide.

---

### F-246 — `virtue.redcap` — five further stated clauses reach neither locale (D5)

**Passage** (ArMDE:4846, :4848, :4850, verbatim):
> A newly-Gauntleted Redcap has enchanted devices with fifty levels of effect, **including modifications to the level due to factors such as the number of uses per day** (see the Laboratory chapter, page 256, for details).
> **These levels are invariably split between two or more effects useful for delivering messages. New Redcaps are never given items capable of killing, wounding, or ensorcelling large numbers of mundanes;** the risk of abuse bringing trouble on the Order is too great. These will be upgraded and replaced in return for good service, **on average an extra two levels per year.**
> … **When you start to age, a magus with a Lab Total of at least fifty will devise a Longevity Ritual (see page 261) for you free of charge, if you wish.**
> … **You must spend two seasons per year delivering messages for the Order.** Your other two seasons are, however, genuinely free … **This Virtue is available to male and female characters.**

German, line-parallel at ArMDE:4846, :4848, :4850 — e.g. "**Diese Stufen sind
grundsätzlich auf zwei oder mehr Effekte aufgeteilt** …"; "**im Durchschnitt um
zwei zusätzliche Stufen pro Jahr**"; "**wird ein Magus mit einer Laborsumme von
mindestens fünfzig für Dich auf Wunsch kostenlos ein Langlebigkeitsritual …
entwickeln**"; "**Du musst zwei Quartale pro Jahr damit verbringen, Nachrichten
für den Orden zu überbringen.**"

**Current data:** `{"type": "item_level_budget", "amount": 50}` — which is
**correct** (ArMDE:4846 "fifty levels of effect") and matches its neighbour
`virtue.magic_items`' 25, whose own entry is prerequisite-gated on this one. No
`description` in either locale.

**Why it is wrong.** Five clauses no effect implements: the split-across-two-or-
more-effects requirement and the never-lethal restriction both constrain how the
50 levels may be spent (`validate_devices` checks only the total, A19); the
+2 levels/year and the free Longevity Ritual are advancement rules; and the
two-seasons-per-year obligation is a standing seasonal cost. The male-and-female
sentence is the explicit-non-restriction shape and would normally encode to
nothing — but in a batch where three sibling Social Statuses are male-only
(F-233, F-240, F-255), the explicit statement that this one is not is worth the
line.

**Correct value.** All of them written into `description` in **both** locales.

**Severity.** Lost rule.

**ArMDE:2816 instance.** Noted only; no fix proposed.

---

### F-247 — `virtue.religious` — `narrative` on an entry that grants a gated Ability category, with no `ability_authorization`

**Passage** (ArMDE:4858, verbatim):
> **You may take Academic Abilities for the character during character generation.** The Wealthy Virtue and Poor Flaw are unlikely to be appropriate.

German, ArMDE:4858: "**Du kannst bei der Charaktererschaffung Akademische
Fertigkeiten für den Charakter erwerben.** Die Tugend Wohlhabend und der Fehler
Arm sind wahrscheinlich nicht angemessen."

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Same mechanism as F-209, F-216, F-230, F-232.

**Correct value.** `"classification": "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}`.

**Why this entry draws only one finding.** Everything else it says is hedged and
is correctly encoded by encoding nothing: "are **unlikely** to be appropriate"
(Wealthy/Poor), and the three redirects at ArMDE:4860 ("you **should** take the
Mendicant Friar Virtue instead", "**should** instead take the Priest Social
Status", "**should** instead have the Senior Clergy Social Status") — all three
targets exist in the catalogue, and all three are guidance, the same reading B06
gave `virtue.merchant_adventurer`'s Partner redirect.

**Severity.** Wrong rules output.

**ArMDE:2816 instance.** Noted only; no fix proposed.

---

### F-248 — `virtue.reserves_of_strength` — "+3", a use limit and a cost, classified `narrative`

**Passage** (ArMDE:4864, verbatim, the entire entry body):
> **Once per day,** when in need, you can perform an incredible feat of strength. For the duration of the action, **add +3 to your effective Strength score. Afterwards, though, you must make two Fatigue rolls.**

German, ArMDE:4864 (line-parallel):
> **Einmal pro Tag** kannst Du in der Not eine außergewöhnliche Kraftleistung vollbringen. Für die Dauer der Handlung **addierst Du +3 zu Deinem effektiven Stärkewert. Danach musst Du jedoch zwei Erschöpfungswürfe ablegen.**

**Current data:** `"classification": "narrative"`, no `effects`. Both locales'
`summary` carries the first clause only.

**Why it is wrong.** Three mechanical clauses in one sentence pair: a signed +3
to a named Characteristic, a per-day use limit, and a cost paid afterwards. It
is emphatically not colour.

**Why it must not become a `characteristic_score_delta`.** A24's fold is
unconditional and permanent — it moves the effective Strength on the character
sheet for every total, every roll, always. This bonus lasts "for the duration of
the action", once a day, and is paid for with two Fatigue rolls. Encoding it as
a flat +3 would be a *worse* error than leaving it uncomputed, and is exactly the
"conditional modifier folded in flat" shape the brief warns about. D3 governs.

**Correct value.** `"classification": "uncomputed_rule"` with all three clauses
written into `description` in **both** locales.

**Severity.** Lost rule.

---

### F-249 — `virtue.ripper` — 70 levels of power carried by no `power_levels`, and the app errors on a legal character

**Passage** (ArMDE:4868, verbatim, the entire entry body):
> A ripper has two powers. **He can destroy a single piece of cloth, or disembowel a single animal, by spending a Fatigue level, at Sight range, without words or gestures. These are a PeAn(He) 25 and a PeAn 45 effect, each with +0 Penetration.** Note that this description is deliberately limiting: a ripper can't break a horse's leg or send it blind. A ripper cannot sever a rope. A ripper cannot cleanly slice things with his mind. **The ripper has two, entirely inflexible, effects.**

German, ArMDE:4868 (line-parallel):
> Ein Aufschlitzer besitzt zwei Kräfte. **Er kann auf Sichtweite, ohne Worte oder Gesten, durch Aufwenden einer Erschöpfungsstufe ein einzelnes Stück Stoff zerstören oder ein einzelnes Tier ausweiden. Es handelt sich um einen PeAn(He)-25-Effekt und einen PeAn-45-Effekt, jeder mit Penetration +0.** … **Der Aufschlitzer hat zwei völlig unveränderliche Effekte.**

**Current data:** `"classification": "narrative"`, **no `effects` at all**,
`categories: ["supernatural"]`, `magnitude: "minor"`.

**Why it is wrong, and why this is the batch's second `mentored_by_demons`
case.** The passage states two powers of level 25 and 45 — **70 levels**, with
Penetration 0 so nothing is added for that. A player who records them in
`Entity::powers`, which is where a supernatural power goes, is measured by
`validation/might.rs::validate_powers` — read in full — as
`used = powers_used(entity)` against `budget = power_levels_budget(entity,
ruleset)`, and `used > budget` pushes `ValidationIssue::error` with
`CODE_OVER_POWER_LEVELS`. Ripper grants no `power_levels`, so the budget is
**0** and the error fires at 70 over. The app refuses the character the passage
describes, and explains nothing — exactly B06's F-194 shape, on a different
validator.

**And it really is reachable for a Might-less character.** `validation/mod.rs`
calls `validate_powers` inside the single `if entity.entity_kind ==
EntityKind::Character` block, alongside `validate_devices` and
`validate_focus_powers` — the gate is the **entity kind**, not the presence of
Might. (`types.rs::SupernaturalPower`'s own doc comment opens "A supernatural
power a **Might-being** holds", which is narrower than what the validator
actually enforces; a Minor Supernatural Virtue like Ripper grants no Might. That
mismatch is a doc-comment defect, recorded here rather than raised as a separate
finding because it is source prose, outside this audit's data scope.)

Note the catalogue's own comparison, which makes the omission visible without
any judgement: `virtue.personal_power` (Minor, Supernatural) and
`virtue.ritual_power` (Major, Supernatural) each carry
`{"type": "power_levels", "amount": 25}` for powers the *player* designs, and
Ripper — Minor, Supernatural, two powers **specified by the book** — carries
nothing.

**Correct value.** `"classification": "creation_effect"` plus a `power_levels`
grant. Whether the amount is 70 (the two powers' levels) or whether the two
fixed powers should instead be pre-entered as `Entity::powers` rows — which no
`Effect` variant can express — is a modelling choice. → **Q-58.**

**Severity.** **Wrong rules output** — the app raises a hard error on a legal
character.

**ArMDE:2960 instance** — a Supernatural Virtue whose realm association the
engine does not model. Noted only.

---

### F-250 — `virtue.ripper` — the activation cost, range and inflexibility reach neither locale (D5)

**Passage** — ArMDE:4868 as quoted in F-249. The clauses: "**by spending a
Fatigue level**", "**at Sight range**", "**without words or gestures**",
"**each with +0 Penetration**", and the four-sentence limiting paragraph ending
"**The ripper has two, entirely inflexible, effects.**"

**Current data:** both locales' `summary` is the four-word "A ripper has two
powers." / "Ein Aufschlitzer besitzt zwei Kräfte." — which states the count and
nothing else. No `description` in either locale.

**Why it is wrong.** This is the shortest summary in the batch against one of
the most mechanically specific passages in it. Everything that makes the Virtue
usable — what the two powers *are*, what they cost, how far they reach, and the
explicit statement that they cannot be varied — leaves the application.

**Correct value.** All of it written into `description` in **both** locales.

**Severity.** Lost rule.

---

### F-251 — `virtue.ritual_power` — four stated clauses reach neither locale (D5)

**Passage** (ArMDE:4872, :4876, verbatim):
> The power has an **Initiative equal to the character's Quickness – (twice power magnitude)**. **It costs one Fatigue level to activate if its level is less than or equal to 25, two Fatigue levels if its level is 26 to 50, and so on. In addition, you must spend one Confidence Point for every magnitude of the effect.** You may also spend levels onefor-one to give the power Penetration, otherwise, it has a Penetration of zero.
> … **The power must be associated with the same supernatural realm as the system on which it is based.**

German, ArMDE:4872, :4876 (line-parallel):
> Die **Initiative der Kraft entspricht der Schnelligkeit des Charakters – (zweifache Magnitude der Kraft)**. **Sie kostet eine Erschöpfungsstufe zur Aktivierung, wenn ihre Stufe 25 oder weniger beträgt, zwei Erschöpfungsstufen bei Stufe 26 bis 50 und so weiter. Außerdem musst Du für jede Magnitude des Effekts einen Selbstvertrauenspunkt ausgeben.** …
> … **Die Kraft muss derselben übernatürlichen Sphäre zugeordnet sein wie das System, auf dem sie basiert.**

**Current data:** `{"type": "power_levels", "amount": 25}`,
`max_per_target: 255`; no `description` in either locale.

**What is right.** The 25 levels ✓ (ArMDE:4872 "a level of 25 or lower"),
`max_per_target: 255` ✓ (ArMDE:4874 "may be taken more than once, and the levels
added together"), and the one-for-one Penetration spend ✓ via A27's
`powers_used = Σ (level + penetration)`. Note that the English prints
"onefor-one" — an OCR join; the German "im Verhältnis eins zu eins" confirms the
reading.

**Why the four clauses are a finding, and why the third is the interesting
one.** The **Confidence Point cost is a second currency** — one point per
magnitude, every activation — and the engine *does* compute a Confidence score
(`effective/gift_confidence.rs::confidence`, A15) that **nothing spends** (C3c
records it as display-only). So this is not a case of an unmodelled concept but
of a modelled number with no consumer, which makes writing the rule out the only
way it reaches the player. The realm association is inexpressible for the same
reason as F-221 (`SupernaturalPower` has no realm field).

**And the comparison against `virtue.personal_power` is the finding's point.**
Both Virtues grant 25 levels; one is Minor and one Major. What justifies the
3-point price is entirely in the clauses neither entry states — Ritual vs
Formulaic equivalence, a steeper Fatigue schedule, the Confidence cost, a
harsher Initiative formula — against Personal Power's Range: Personal / constant
restriction. A player comparing the two in the picker sees two identical
summaries and two different prices.

**Correct value.** All four written into `description` in **both** locales.

**Severity.** Lost rule, and a buying-decision defect.

**ArMDE:2960 instance.** Noted only.

---

### F-252 — `virtue.rosh_beth_din` — three stated Ability prerequisites and an age requirement are encoded nowhere

**Passage** (ArMDE:4880, verbatim, the second sentence):
> **Your character must be (30 – Int) years old to take this Virtue and have scores of at least 5 in Hebrew, Rabbinic Law, and Theology: Judaism.**

German, ArMDE:4880 (line-parallel):
> **Dein Charakter muss mindestens (30 – Int) Jahre alt sein, um diese Tugend wählen zu können, und muss mindestens einen Wert von 5 in Hebräisch, Rabbinisches Recht und Theologie: Judentum haben.**

**Current data:** no `prerequisites` key.

**Why it is wrong.** Two of the three Ability minima are **directly
expressible**: `ability.rabbinic_law` and `ability.theology_judaism` both exist
in `rules/core/abilities.json` (both `academic`, neither parameterized), so
`{"kind": "all", "value": [{"kind": "ability_min", "value": {"ability":
"ability.rabbinic_law", "score": 5}}, …]}` is exactly the shape. B6 records that
`ability_min` compares against the **effective** score, which is the right
reading of "have scores of at least 5".

Hebrew is the awkward one and is stated precisely rather than glossed: the
catalogue has no `ability.hebrew`; Hebrew is an instance of
`ability.dead_language`, which carries `"parameter": "language"`. `Prereq::AbilityMin`
declares `{ability: Id, score: u8}` and **no instance field** (B6), so the most
that can be encoded is "some Dead Language at 5", which a rabbi with Latin 5 and
no Hebrew would satisfy. The age requirement, `(30 – Int)` years, is not
expressible at all — no `Prereq` variant reads age or a Characteristic.

**This is B05's `leper_magus` / B06's three-Social-Status shape**, recurring on a
fourth entry, and it is the strongest instance yet because two of the three
minima are exactly expressible and simply absent.

**Correct value.** The two expressible minima encoded as `prerequisites`; the
Hebrew instance and the age formula written into `description` in **both**
locales.

**Severity.** Wrong rules output — the app permits a character the passage
forbids.

---

### F-253 — `virtue.rosh_beth_din` — the Academic *category* is authorized as three ids

**Passage** (ArMDE:4880, verbatim, the third sentence):
> **Your character may purchase Academic Abilities at character generation** and gains 50 extra experience points to be spent on the required Abilities.

German, ArMDE:4880: "**Dein Charakter kann bei der Charaktererschaffung
Akademische Fertigkeiten erwerben** und erhält 50 zusätzliche Erfahrungspunkte,
die für die erforderlichen Fertigkeiten ausgegeben werden müssen."

**Current data:**
```
{"type": "restricted_ability_xp", "amount": 50,
 "abilities": ["ability.dead_language", "ability.rabbinic_law",
               "ability.theology_judaism"]}
```

**Why it is wrong.** The 50 points and their three targets are right — ArMDE:4880
earmarks them for "the required Abilities", which are exactly the three the
previous sentence names ✓. But A7 records that the pool *also* confers
permission, and it confers it only over those three ids. The passage grants
**"Academic Abilities"** unqualified — the whole gated category. So a Rosh Beth
Din who buys Artes Liberales or Medicine, both `academic`, is refused with
`ability_category_requires_virtue` although the passage lets him.

This is B06's `marshal` / `master_of_kennels` shape (a pool naming ids where the
passage grants a category), and B06's qualifier applies unchanged: the check is
always "does the pool's `abilities`/`categories` union cover what the passage
permits", and here it does not.

**Correct value.** A separate
`{"type": "ability_authorization", "categories": ["academic"]}` alongside the
existing pool — *not* widening the pool's own `categories`, because the 50
points really are earmarked for the three named Abilities and widening them
would let the pool fund any Academic Ability.

**Severity.** Wrong rules output — the app refuses a legal purchase.

---

### F-254 — `virtue.rosh_beth_din` — the granted Reputation is typed `local` though the passage says it applies across his country

**Passage** (ArMDE:4882, verbatim, the first sentence):
> The Rosh Beth Din gains the **+2 good Reputation Rosh Beth Din, which applies across his country.**

German, ArMDE:4882 (line-parallel):
> Der Rosh Beth Din erhält die **gute Reputation Rosh Beth Din +2, die in seinem gesamten Land gilt.**

**Current data:** `{"type": "grants_reputation", "kind": "local", "score": 2}`.

**Why it is wrong.** `types.rs::ReputationType` — read in full — declares four
values: `Local` ("Known to those who live near the character (the default)"),
`Ecclesiastical`, `Hermetic`, `Academic`. A country-wide Reputation is none of
them, and `Local`'s own doc comment ("those who live **near**") is the direct
contradiction.

**The repo's own tables agree, twice.**
`rules/source/de/translation-tables/reputationen.md:88`:
`| Rosh Beth Din | Rosh Beth Din | Lokal/National | 2 (+) | Rosh Beth Din |`; and
`:138`: `| Rosh Beth Din | Rosh Beth Din | National (jüdisch) | Tugend Rosh Beth
Din |`. Both hand-curated rows record a **national** scope the enum cannot hold.

**Correct value.** `local` is the least-wrong of four, and there is no fifth to
move to. Under D3 the right answer is to keep the grant (so
`validate_reputations` does not refuse the Reputation) and write "applies across
his country" into `description` in **both** locales, so the scope reaches the
player as text. Adding a national tier to `ReputationType` is a design change
and is not proposed here.

**Severity.** Wrong rules output, low — the audience shown is narrower than the
book's, and nothing else depends on the type beyond slot matching.

---

### F-255 — `virtue.rosh_beth_din` — male-only, the earmark scope and the yeshivah clause reach neither locale (D5)

**Passage** (ArMDE:4880, :4882, verbatim):
> … gains **50 extra experience points to be spent on the required Abilities.**
> This Virtue also includes the effects of the Social Contacts Virtue and **you are able to find contacts within any Jewish community that supports a yeshivah. This Virtue is only available to male characters.**

German, ArMDE:4880, :4882 (line-parallel):
> … erhält **50 zusätzliche Erfahrungspunkte, die für die erforderlichen Fertigkeiten ausgegeben werden müssen.**
> Diese Tugend beinhaltet auch die Effekte der Tugend Soziale Kontakte, und **Du kannst innerhalb jeder jüdischen Gemeinschaft, die eine Jeschiwa unterhält, Kontakte finden. Diese Tugend steht nur männlichen Charakteren zur Verfügung.**

**Current data:** three effects — the `grants_reputation`, the
`restricted_ability_xp`, and `{"type": "grants_selection", "items":
["virtue.social_contacts"]}` ✓ which is correct and is the *right* use of A18
(a fixed, single-item free grant, target exists). No `description` in either
locale.

**Why the three clauses are a finding.**

1. **Male-only** — the batch's fourth gender restriction, unmodelled
   (`gender: String`, "no mechanical effect") and unstated.
2. **The earmark is narrower than the pool.** "to be spent on the **required**
   Abilities" means Hebrew, Rabbinic Law and Theology: Judaism — and Hebrew is
   an instance of `ability.dead_language`, which the pool names **without an
   instance**. A7 records `PoolEligibility::Ability` is built with
   `instances: Vec::new()`, and that an Ability spend qualifies on
   `abilities.contains(id)` alone — so the pool funds *every* Dead Language,
   Latin and Greek included. The engine cannot scope a restricted pool to one
   instance (A7: "It cannot express an instance-scoped pool"), so D3 governs and
   the earmark owes its text.
3. **The yeshivah scoping** is the Virtue's own extension of Social Contacts and
   is not in Social Contacts' entry either (`virtue.social_contacts`,
   ArMDE:4988-4991, is `narrative` with no effects and is outside this batch —
   **flagged for B08**, not rated here).

**Two forward-looking notes for B08, from following the Social Contacts
pointer.** ArMDE:4990 was read in full and states "you can contact someone on a
**simple Presence roll against an Ease Factor of 6**" and "this Virtue …
(**specified when this Virtue is purchased**)" and "You may purchase this Virtue
more than once, **each time specifying a different social group**". The entry is
`narrative` with no `parameters` and `max_per_target: 255`. That is B08's call,
not this batch's — but it bears directly on Rosh Beth Din: A18 records that
`grants_selection` appends a bare `Selection::new(id)` **with no params**, so the
day `virtue.social_contacts` gains the social-circle parameter its passage
requires, **this entry's grant becomes a selection with an unfilled parameter**,
which A18 says resolves to nothing and which nothing warns about. The two
corrections must land together.

**Correct value.** All three written into `description` in **both** locales.

**Severity.** Lost rule; the earmark clause is additionally a live
over-permission (Latin funded from a Hebrew earmark).

**ArMDE:2816 instance.** Noted only; no fix proposed.

---

### F-256 — `virtue.relic` and `virtue.powerful_relic` — `true_faith_grant` puts the *relic's* score on the *character*, which ArMDE:17607 forbids

*(Found by the verification pass, which was not allowed to see this file. The
first pass cleared `virtue.relic` outright and rated `virtue.powerful_relic`'s
effect correct. Both verdicts are withdrawn. Every claim below was re-derived
from the source before being recorded here.)*

**The entries' own passages** (ArMDE:4854 and ArMDE:4784, verbatim):
> You own a holy relic, such as the finger bone of a saint, **with a True Faith Score of one.** … **See Chapter 12: Realms for rules for relics and True Faith.**
> You own an unusually powerful relic **with a True Faith score of 3.**

German, line-parallel:
> Du besitzt ein heiliges Relikt, etwa den Fingerknöchelknochen eines Heiligen, **mit einem Wahrer-Glaube-Wert von 1.** … **Regeln für Relikte und Wahren Glauben findest Du in Kapitel 12: Sphären.**
> Du besitzt ein ungewöhnlich mächtiges Relikt **mit einem Wahrer-Glaube-Wert von 3.**

In both sentences the score belongs to **the relic** — "a relic *with* a True
Faith Score", not "you have a True Faith Score".

**Where the cross-reference lands** (ArMDE:17607, verbatim — the whole point):
> True Faith, like Confidence, has a Score and Points. Taking the True Faith Major Virtue grants you a True Faith Score of 1, and a single Faith Point. **Only by possessing the True Faith Major Virtue may a character have a True Faith score.** Any character may possess any number of Faith Points … that once spent, do not return.

German, ArMDE:17607 (line-parallel):
> Die Große Tugend Wahrer Glaube verleiht einen Wahrer-Glaube-Wert von 1 und einen einzelnen Glaubenspunkt. **Nur wer die Große Tugend Wahrer Glaube besitzt, kann einen Wahrer-Glaube-Wert haben.** …

And ArMDE:17623, which gives the score to the object instead:
> 1. FAITH: **All relics contain a True Faith score**, giving it Faith Points that may be used by its bearer as Confidence. A relic also grants **Magic Resistance equal to ten times its True Faith score to its bearer.** **A person can only benefit from one relic at a time**, any attempt to do otherwise is a sin, and means that they lose the benefits of all relics.

ArMDE:17609 and :17611 confirm the two states are mechanically distinct rather
than two names for one thing: a character **without** a True Faith Score "may
only spend one Faith Point at a time" and "does not benefit from Magic
Resistance"; a character **with** one spends up to his Score and gains MR ×10.

**Current data.** `virtue.relic` → `{"type": "true_faith_grant", "score": 1}`;
`virtue.powerful_relic` → `{"type": "true_faith_grant", "score": 3}`. Neither
entry is the True Faith Major Virtue — Relic is `minor`/`general`, Powerful
Relic `major`/`general`, and `virtue.true_faith` exists separately ✓.

**What the engine does with it.** `effective/might.rs::true_faith` sums every
`TrueFaithGrant` into **the character's** derived True Faith Score;
`arm-app/src/ruleset_io.rs` surfaces it as
`EffectiveScores::true_faith_score`, and the sheet renders it through the Fluent
`true-faith-label` / `true-faith-readout` keys. So a companion whose only
relevant Virtue is Relic is shown **"True Faith — Score 1" / "Wahrer Glaube —
Wert 1"**, a state ArMDE:17607 says he cannot be in.

Two consequences follow from the same wiring. Because A21's fold is a **Σ**,
Relic + Powerful Relic displays **4**, against ArMDE:17623's "a person can only
benefit from one relic at a time". And
`crates/arm-rules/tests/data_integrity.rs` pins the wrong reading in place —
read directly, line 2536's comment is "Relic → True Faith 1 (ArMDE:4854);
Powerful Relic → 3 (ArMDE:4783)" followed by
`assert_eq!(true_faith(&relic, &rs), 1);` and
`assert_eq!(true_faith(&prelic, &rs), 3);`.

**The repo already records the correct reading, and contradicts itself.**
`crates/arm-rules/RULES.md` — both passages read directly:

- Line 3592-3593, in the True Faith entry: "`flaw`/`virtue.relic` grants a
  *possessed* holy item with True Faith 1 — **the character's own score stays
  0** — so Relic is item-possession, left structural."
- Lines 5406-5408, in the per-variant inventory, under the heading
  "**True Faith score:**": "`virtue.powerful_relic` (ArMDE:4782-4787) — True
  Faith score 3" and "`virtue.relic` (ArMDE:4852-4855) — True Faith score 1".

The traceability map asserts both that Relic leaves the character at 0 and that
Relic is a True Faith score granter. The data implements the second; ArMDE:17607
supports the first. This is in-repo corroboration reached *after* the source
reading, not the basis for it.

**Correct value.** Drop `true_faith_grant` from both entries, leaving
`virtue.true_faith` (ArMDE:5169-5171) as its only legitimate author —
which is what RULES.md:3592-3593 already says the design is. `virtue.relic` then
carries no effects and its class moves `creation_effect` → `uncomputed_rule`;
`virtue.powerful_relic` likewise, since its remaining clauses (F-229) are all
uncomputed. The relic's own score and everything it confers become
`description` text under **F-257**. The two `data_integrity.rs` assertions and
RULES.md:5406-5408 must be corrected in the same change.

**Severity.** **Wrong rules output.** It is not merely a missing computation: the
app *displays a number* asserting a rules state the book explicitly forbids, and
it compounds across two relics against a stated one-at-a-time limit. Note that
A21 records nothing consumes `true_faith_score` (C3c) — that is **not** what
this finding is about and is not re-reported; the defect is that the effect
names the wrong subject and the surfaced figure is therefore a false claim.

---

### F-257 — `virtue.relic` — the relic's actual mechanics reach neither locale (D5)

**Passage** — ArMDE:4854 ends "**See Chapter 12: Realms for rules for relics and
True Faith.**" / "**Regeln für Relikte und Wahren Glauben findest Du in Kapitel
12: Sphären.**" Following it to ArMDE:17619-17624 yields four mechanics, none
computed and none stated:

1. **Faith Points usable as Confidence** — ArMDE:17623 "All relics contain a True
   Faith score, **giving it Faith Points that may be used by its bearer as
   Confidence**."
2. **Magic Resistance ×10 to the bearer** — ArMDE:17623 "A relic also grants
   **Magic Resistance equal to ten times its True Faith score to its bearer**."
   For `virtue.relic` that is **MR 10**, from a *Minor* Virtue.
3. **Divine Might = Faith × 10** — ArMDE:17624 "A relic's Divine Might is equal
   to the relic's Faith score … multiplied by 10, and the Might Pool is
   refreshed with every sunrise."
4. **One relic at a time** — ArMDE:17623 "**A person can only benefit from one
   relic at a time**, any attempt to do otherwise is a sin, and means that they
   lose the benefits of all relics."

German, line-parallel at ArMDE:17623: "…Glaubenspunkte, die ihr Träger wie
Selbstvertrauen einsetzen darf. Eine Reliquie verleiht ihrem Träger zudem
Magieresistenz in Höhe des Zehnfachen ihres Wahrer-Glaube-Wertes. **Eine Person
kann immer nur von einer einzigen Reliquie profitieren**…"

**Current data:** no `description` in either locale — confirmed by `jq` over the
batch's 35 entries in both files, which returns the key set
`["name", "summary"]` and nothing else. Both summaries stop at the first clause.

**Why it is wrong.** This is D5's own worked example transplanted:
`virtue.faerie_magic` "computes its main effect while three mechanics behind its
page-reference reach the user nowhere". Here the entry computes a number that is
itself wrong (F-256) while four mechanics behind its chapter-reference reach the
user nowhere. **A Minor Virtue silently conferring Magic Resistance 10 is not a
rounding error** — it is larger than a starting magus's Parma Magica contributes
at Parma 1 or 2.

The one-relic-at-a-time limit additionally has no data form: `incompatible_with`
between `virtue.relic` and `virtue.powerful_relic` would be wrong, because
ArMDE:17623 permits *owning* several ("a person can humbly carry other relics,
while only relying on one for the benefits") and restricts only *benefiting*.
D3 governs.

**Correct value.** All four written into `description` in **both** locales. The
same four belong on `virtue.powerful_relic`, where the score is 3 and the
bearer's Magic Resistance is therefore **30**.

**Severity.** Lost rule, and the largest single unstated number in the batch.

---

## Open questions

Nine, each self-contained. Per the audit's rules these are **escalated, not
resolved**: every entry carrying one also carries a finding, so no entry is left
unrated on a question alone. Q-63 is the verification pass's.

### Q-55 — `virtue.physician_of_salerno`: is the granted Reputation Local or Academic?

ArMDE:4734 says only "**granting a Reputation of Physician of Salerno 2**" — it
names the Reputation and its score and **not its audience**. The data asserts
`"kind": "local"`. The repo's own hand-curated table gives two answers in one
cell: `rules/source/de/translation-tables/reputationen.md:82` and `:131` both
read `Arzt von Salerno | **Lokal / Akademisch** | 2 (+)`. `ReputationType`
offers `Local`, `Ecclesiastical`, `Hermetic` and `Academic`, and `Academic`'s
own doc comment in `types.rs` says it is "the 'Academic Reputation' the
scholastic Social-Status Virtues confer — Baccalaureus, Magister in Artibus,
Doctor in (Faculty), …", which is the company a Salerno physician keeps. The
choice is not settleable from ArMDE:4732-4735. **Which type is correct — and if
the answer is "either, player's choice", should the grant become the wildcard
`kind: null` instead?**

### Q-56 — `virtue.perfectus`: should Purity and Transcendence be added to the Ability catalogue?

ArMDE:4634 permits a True-Faith-bearing perfectus to take "**the Purity and
Transcendence Supernatural Abilities from Realms of Power: The Divine Revised
Edition (page 53)**". RoP:D **is** in `rules/source/en/`, so CLAUDE.md's
provenance rule does not bar implementing them — unlike B06's *Cradle and the
Crescent* pointer. But `rules/core/abilities.json` contains neither: searched
the whole `abilities` array for `purity` and `transcendence`, no match. **Should
the two Abilities be added from RoP:D (which would make the permission an
`ability_authorization` gated by `Prereq::Has("virtue.true_faith")`), or does
adding non-core Abilities to the catalogue exceed what this project models
today?** Note the same question will recur: this is the first time a core-book
entry has pointed at a *supplement* Ability whose source is in repo.

### Q-57 — Potent Magic: how should the forced magnitude-variant incompatibility be lifted?

ArMDE:4742 states in both languages that "**a maga may have more than one area of
Potent Magic**", so `virtue.potent_magic_major` and `virtue.potent_magic_minor`
must be compatible (F-226). But
`ruleset/integrity.rs::validate_magnitude_variant_exclusivity` **fails the whole
ruleset load** unless every `<stem>_major` / `<stem>_minor` pair declares each
other `incompatible_with`, and these two ids match that pattern. The guard is
right for every other pair in the catalogue — Magical Focus's own passages
(ArMDE:4405, :4542) say a magus may have only one "either major or minor" — and
Potent Magic is the book's explicit counter-example. **Two ways out: exempt this
pair in the guard (a named exception, which the guard has no mechanism for
today), or rename the ids out of the stem pattern (e.g.
`virtue.potent_magic_wide` / `_narrow`), which is a breaking change to every
saved character holding one.** Which?

### Q-58 — `virtue.ripper`: are the two fixed powers a `power_levels` grant or pre-entered powers?

ArMDE:4868 specifies both powers outright — "**a PeAn(He) 25 and a PeAn 45
effect, each with +0 Penetration**" — and insists "**The ripper has two,
entirely inflexible, effects.**" The entry carries no `power_levels`, so a
player who records them hits a hard `over_power_levels` error against a budget of
0 (F-249). `{"type": "power_levels", "amount": 70}` clears the error, but it
models the wrong thing: a budget implies the player *designs* the powers, and
this passage's whole point is that he may not. The engine has no `Effect` variant
that grants a **named** `Entity::powers` row — which is the same structural gap
B06 found on `virtue.mercurian_magic`'s Wizard's Vigil (a named spell the engine
cannot grant). **Is the right answer the budget (accepting that it over-permits),
a new grant variant, or `uncomputed_rule` with the two powers written out and the
budget left at 0 — which leaves the error firing?**

### Q-59 — `virtue.privileged_upbringing`: can a pool-scoped permission be expressed at all?

ArMDE:4808 says "**You may not, however, buy Academic or Martial Abilities with
your normal pool of experience points unless you have another Virtue or Flaw
permitting that.**" The engine's permission model is character-wide:
`effective/xp.rs::ability_authorizations` folds a `restricted_ability_xp` pool's
`categories` into the authorization set with the comment "Experience earmarked
for a category or Ability is itself permission to learn it — otherwise the grant
could never be spent" (A7). That reasoning is correct for Warrior and Educated
and inverted for this entry (F-234). **Is the right fix (a) a flag on
`RestrictedAbilityXp` meaning "funds but does not authorize", (b) separating the
permission axis from the funding axis generally, or (c) accepting the
over-permission and carrying the rule as text?** Note that (a) would change the
authorization semantics of 28 shipped `restricted_ability_xp` uses, so the
answer needs a survey this batch cannot do.

### Q-60 — `virtue.personal_vis_source`: is "about one tenth" a rule or hedged guidance?

ArMDE:4730: "Determine the amount and type with the help of your troupe, **the
yield should be about one tenth as much as the player covenant expects to gather
per year at the beginning of the saga.**" It states a **quantity**, which the
`uncomputed_rule` definition covers ("GM judgement, open-ended magnitudes"), but
it states it with two hedges ("should be", "about") of the kind B06 read as
guidance on `virtue.merchant_adventurer`. The distinction drawn in F-222 is that
B06's hedged sentences carried no number and this one does. **Is that the right
line, or does a hedge outrank a number?** The answer generalises: the same shape
will recur wherever the book says "should be roughly N".

### Q-61 — `virtue.rat_up_a_drainpipe`: is "a substantial advantage" mechanical?

ArMDE:4840: "this character can run through urban areas **as if unobstructed**.
This gives **a substantial advantage in opposed Athletics rolls** which represent
being chased." It names a roll type and a circumstance and asserts an advantage,
with **no number**. Every `narrative → uncomputed_rule` move the audit has made
so far has rested on a number, a cap or an absolute; this has the absolute ("as
if unobstructed") but its operative clause has none. **Is an un-numbered,
storyguide-sized advantage in a named roll a mechanical clause (→
`uncomputed_rule`) or colour (→ `narrative`)?** F-241 takes the first reading.
The answer sets a threshold several later batches will need.

### Q-62 — `virtue.powerful_relic`: is the relic's one power charged against the power-levels budget?

ArMDE:4784: "**The relic also has one power, which should be agreed upon with the
storyguide.**" `effective::powers_used` charges every `Entity::powers` row
against `power_levels_budget`, which this Virtue leaves at 0 — so if a relic's
power is recorded there, the character gets `over_power_levels`, the same error
F-249 describes for Ripper. But a relic is an **item** the character owns, not a
power the character has, and the engine already keeps a separate item-level
budget (A19) and a separate familiar-power pool that is charged against nothing
(`types.rs`: "there is no limit to the number of powers which may be invested in
a familiar"). **Where does a relic's power live — `Entity::powers` (and if so,
does Powerful Relic owe a `power_levels` grant?), a device, or nowhere the engine
models?** ArMDE:4784's cross-reference to the Relics rules at page 419 was
followed and states no level for a relic power, so the source does not settle it.

### Q-63 — does a translation table's term bind *inside a sentence*, or only on a label?

*(Raised by the verification pass; every citation below was re-checked here.)*

`rules/source/de/translation-tables/sphären-mächte.md:358` gives
`| True Faith Score | **Wahrer-Glauben-Wert** | …`, while the DE rulebook uses
**`Wahrer-Glaube-Wert`** at every occurrence (ArMDE:4854, :4784, :17607, :17609,
:17611, :17613, :17615, :17623), and `rules/i18n/de/virtues_flaws.json` follows
the rulebook for both `virtue.relic` ("mit einem Wahrer-Glaube-Wert von 1") and
`virtue.powerful_relic` ("von 3").

`rules/source/de/translation-tables/README.md` sets a precedent that runs the
other way: for *Inoffensive to (Beings)* it rules "**Kanonisch ist die Tabelle**"
and records that `b86889c` changed the German **name** to match the table
against the printed book. But that ruling is worded for a **label**, and this
term appears inside a **summary sentence**. The standalone label is a third
string again — `locales/de/main.ftl:498` is `true-faith-label = Wahrer Glaube`,
with no hyphenated compound at all.

**So: does CLAUDE.md's "the German label for any term whose English form appears
in a table MUST match the table's `Deutsch (DE)` value" reach prose inside a
`summary`/`description`, or only the `name` field?** The answer decides whether
F-257's new German description text must read `Wahrer-Glauben-Wert` (table) or
`Wahrer-Glaube-Wert` (rulebook and existing data), and it generalises to every
description this audit's correction pass will write. Not settleable from the
sources; escalated.

---

## Sub-agent reconciliation

One verification agent was run. Its brief: re-derive, **from the sources**, the
six verdicts this batch's first pass had cleared — `virtue.nuntius`,
`virtue.peasant`, `virtue.persona`, `virtue.premonitions`,
`virtue.rapid_convalescence`, `virtue.relic`. It was told explicitly that
overturning one would be a success and inventing one would not, and it was
**forbidden to read this file** or any other `batch-NN.md`. It confirmed it did
not.

**Result: 5 confirmed, 1 overturned.** Prior passes ran 1/13, 2/15, 3/11, 2/7,
1/6 and 0/4.

### The overturn — `virtue.relic`

**Accepted in full, and extended to a second entry.** Every claim in it was
re-derived here before being written up, because the audit's rule is source or
nothing and a sub-agent's report is not a source:

- ArMDE:17603-17617 and ArMDE:17619-17627 were read directly. ArMDE:17607's
  "Only by possessing the True Faith Major Virtue may a character have a True
  Faith score" and ArMDE:17623's "All relics contain a True Faith score …
  grants Magic Resistance equal to ten times its True Faith score to its
  bearer … A person can only benefit from one relic at a time" are verbatim as
  reported.
- `crates/arm-rules/RULES.md` lines 3592-3593 and 5406-5408 were read directly
  and do contradict each other exactly as reported.
- `crates/arm-rules/tests/data_integrity.rs` lines 2536-2543 were read directly
  and do pin `true_faith(relic) == 1` and `== 3`.

**Why the first pass missed it, stated plainly.** It read `virtue.relic`'s
passage, matched `score: 1` against "a True Faith Score of one", noted the
summary's "The relic does not possess any additional powers" as an explicit
*nothing*, and rated the entry clean. It classified the trailing "See Chapter
12: Realms for rules for relics and True Faith" as a general-rules pointer in
the cross-reference table — the same call it correctly made for
`virtue.premonitions`' "(page 170)" and `virtue.redcap`'s "page 256". **The call
was right four times and wrong once, and the difference is that this pointer
leads to a rule about *who may hold the score the entry grants*, not to the
baseline the entry sits on.** The first pass's own summary of the
cross-reference pattern — "the named-Virtue form again produced the live hits" —
is a reminder that the audit had been treating the *page-numbered* form as the
low-yield one; here a **chapter**-reference carried the highest-severity finding
in the batch. That is the lesson, and it is recorded as pattern 1 below.

The overturn became **F-256** (covering `virtue.powerful_relic` as well, which
the first pass had rated data-OK on the identical mistake) and **F-257**.
`virtue.relic` moves from clean to a two-finding entry, the totals move from
29/6 to **30/5**, and the D5 count from eleven to twelve.

### The five confirmations

Each was re-derived from the passage, not merely re-asserted, and each named
what it checked:

| Entry | What the pass verified independently |
|---|---|
| `virtue.nuntius` | Every clause either declines to alter a mechanic ("affected by the Wealthy Virtue and Poor Flaw **as normal**") or explicitly withholds one ("**not necessarily educated**"). It checked the entry specifically against the batch's densest pattern and found it is the **inverse** — no Academic permission is granted, so no `ability_authorization` is owed. It followed both named Virtues to `virtue.wealthy` (ArMDE:5235-5238) and `flaw.poor` (ArMDE:6594-6596). |
| `virtue.peasant` | It verified the one detail worth verifying: the passage names the Wealthy **Major** Virtue and Poor **Major** Flaw, and both entries are `major` in the catalogue. It then searched the catalogue for `serf\|villein\|peasant` to confirm "free, rather than a serf" names no entry and is therefore colour, not an incompatibility. |
| `virtue.persona` | Effect, target, amount and direction all match ArMDE:4712. It confirmed `ability.persona` is `supernatural`, which is **not** in `categories_requiring_virtue`, so no authorization is owed — the check this batch's eight authorization findings turn on. D5 leftover: none, because the disguise mechanics belong to the **Ability** (ArMDE:7692-7694) and **do** reach both locales via `rules/i18n/{en,de}/abilities.json`. |
| `virtue.premonitions` | It followed "(page 170)" to ArMDE:7700-7702 and confirmed that range is `ability.premonitions`' own cited source, so the Perception + Premonitions roll belongs to the Ability and leaves no D5 residue on the Virtue. |
| `virtue.rapid_convalescence` | +3, `recovery` track, correct sign, and — the useful part — it checked the passage for a condition the flat effect would lose and found **none**, which is the "conditional modifier folded in flat" screen coming back negative. It also confirmed the surfaced-only track still reaches the user, `derived-detail-recovery` being present in both locales. |

### Findings it corroborated without having seen them

Reading ArMDE:4598-4883 as continuous prose, it independently reached **eleven**
of this batch's findings — each from the passage plus the entry's data, with no
sight of the verdict table:

- the six `ability_authorization` gaps on `notary` (F-209), `perfectus` (F-216),
  `prestigious_student` (F-230), `priest` (F-232), `redcap` (F-245) and
  `rosh_beth_din` (F-253), plus Redcap's missing 300 XP (F-244) and free
  Well-Traveled (F-243) from the same sentence;
- the four numeric-modifier misclassifications on `perfect_balance` (F-213),
  `piercing_gaze` (F-224), `reserves_of_strength` (F-248) and
  `performance_magic` (F-219) — for the last it cited the same three numbers
  this file does (Ease Factor 3, three extra botch dice, the −3/+0/+3 table);
- the missing `grants_reputation` on `protection` (F-235) — and it reached it by
  the same comparison, that `virtue.physician_of_salerno` four entries earlier
  carries one for the identical sentence shape;
- the missing `power_levels` on `ripper` (F-249), likewise by comparison with
  `virtue.personal_power`;
- the missing `ability_min` prerequisites on `rosh_beth_din` (F-252);
- `paid_rights` as an explicit waiver of ArMDE:2816 carried by no data (F-211).

It contradicted nothing in the other 29 verdicts.

### What it contributed beyond the six

- **Q-63**, on whether a translation table's term binds inside a sentence or
  only on a `name` label — a question that governs every German `description`
  the correction pass will write, not just this batch's.
- **A scale figure for the ArMDE:2960 note**: 115 entries carry the
  `supernatural` category and only five carry a `realm` parameter at all
  (`flaw.bound_to_realm`, `flaw.necessary_realm_aura_for_ability`,
  `flaw.realm_stigmatic`, `virtue.folk_magic`, `virtue.student_of_realm`). The
  audit has been recording ArMDE:2960 as a per-entry "instance, noted only" for
  six batches; that ratio is the argument for deciding it once, centrally,
  rather than noting it 115 times.
- **The `source.lines` end-boundary observation** folded into the Method section
  above: 31 of this batch's 35 ranges end on the trailing blank line and 4 end
  on the last content line, and `rules_source_provenance.rs` rejects only a
  range that is *entirely* blank, so nothing flags either form. Not a per-entry
  defect and raised as one against nobody — but it is a normalization decision
  owed before the upstream source re-sync, under which the two conventions will
  drift differently.

### New catalogue-wide patterns for later batches

1. **An effect whose *subject* is an item, not the character.** F-256 is the
   audit's first instance: the passage gives a score to a possessed object and
   the effect gives it to the owner. The `Effect` enum has no item-scoped
   variant, so every "you own an item with score/level N" Virtue is exposed to
   the same conflation. Worth a targeted screen on `true_faith_grant`,
   `might_grant`, `masterpiece_item` and `item_level_budget`. Note this is a
   *new shape*, not a new instance of a known one: eleven Part-C gaps and six
   batches of findings had all been about a rule the data fails to state or
   states with the wrong number — never about the right number attached to the
   wrong entity.
2. **A chapter-level cross-reference can carry a higher-severity rule than a
   page-level one.** Five batches have recorded the page-number-free
   named-Virtue form as the high-yield pointer. F-256/F-257 came from
   "**See Chapter 12: Realms**", the vaguest form in the book, and the
   first pass dismissed it as a general-rules pointer. **Follow the vague ones
   too**, especially where the entry grants a score whose rules live in the
   chapter being pointed at.
3. **RULES.md can contradict itself about one entry**, once in its narrative
   section and once in its per-variant inventory (here :3592-3593 vs
   :5406-5408). An agent consulting only one inherits whichever it landed on.
   Checking both is cheap and caught this.
