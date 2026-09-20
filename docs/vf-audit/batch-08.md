# Batch B08 — indices 245-279, ArMDE:4884-5096

Entries: 35. Audited: 35. Failures: **26**. Clean: **9**.
Findings: **F-258 … F-305** (48 numbered, **47 live** — F-299 is **WITHDRAWN**,
see "Corrections to this batch's own first pass"). Open questions:
**Q-64 … Q-69** (6).

**Nine of the 48 turned out to be already recorded in
`crates/arm-rules/RULES.md` or in a source doc comment**, and were downgraded or
withdrawn after a deliberate self-check pass. They are kept in the list, with
the recording quoted, because a correction pass still needs them — but they are
**corroboration, not discovery**, and the batch says so rather than claiming
credit. See "Corrections to this batch's own first pass".

Finding and question numbers **continue B07's sequence** (B01 F-01…F-33 /
Q-01…Q-08, B02 F-34…F-65 / Q-09…Q-16, B03 F-66…F-89 / Q-17…Q-22, B04 F-90…F-121
/ Q-23…Q-34, B05 F-122…F-160 / Q-35…Q-44, B06 F-161…F-208 / Q-45…Q-54,
B07 F-209…F-257 / Q-55…Q-63), so this batch starts at **F-258** and **Q-64**.

**This span has never been screened.**
`crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS` stops at
ArMDE:3950; the whole of 4884-5096 lies outside it, as B04's through B07's spans
did. The failure rate is a little below theirs (B04 28/35, B05 29/35, B06 31/35,
B07 30/35, this batch 26/35), and the profile shifts again:

- **Thirteen `narrative` reclassifications** — `senior_master`, `shamash`,
  `sharp_ears`, `side_effect`, `skilled_smuggler`, `skinchanger`,
  `skinchanger_dove`, `social_contacts`, `sofer`, `spiritual_pact`,
  `strong_willed`, `sufi`, `supernatural_beauty`. **Eleven** go to
  `uncomputed_rule` and **two** to `creation_effect` (`senior_master` and
  `sufi`, which also gain an effect). Four are refuted by plain digits alone
  (`sharp_ears` "+3", `skilled_smuggler` "-9", `skinchanger` "+3 Soak",
  `strong_willed` "+3"), and `virtue.skilled_smuggler` is B05's
  internal-contradiction shape again — its own `summary` prints the number its
  class says does not exist, in **both** locales.
- **Six instances of the `ability_authorization` pattern**, now found in eight
  consecutive spans — `senior_bard` (a pool that authorizes two of the four
  Realm Lores the passage permits), `senior_clergy`, `senior_master`, `sufi`
  (three Virtues with no authorization at all against a passage that grants a
  gated category or gated ids outright), `simple_student` (no effects at all)
  and `strong_faerie_blood` ("you may learn Faerie Lore", an `arcane` Ability).
- **A second instance of B07's OVER-permission shape — but, unlike
  `privileged_upbringing`'s, a *documented* one.** `virtue.student_of_realm`
  (F-297) permits all four Realm Lores where the passage permits one ("You may
  take **that** Lore"), and nothing errors on a permission that is too wide.
  This batch first wrote that it was "only findable by reading the passage
  against the data" and then **found its own claim false**:
  `RULES.md:5249-5263` records it as a deliberate approximation with a named
  precedent. The pattern is real; this instance is not new.
- **The same entry also loses its only number**, and that too is on record.
  `virtue.student_of_realm`'s "+2 bonus on all uses of the appropriate Lore" is
  carried by **no effect at all** (F-296), although `ability_bonus` is exactly
  the shape `virtue.puissant_ability` uses for the same idea two hundred lines
  earlier — and `RULES.md:5264-5274` flags precisely that, with the same two
  candidate fixes this batch derived independently.
- **`virtue.special_circumstances` alone carries four findings** (F-284…F-287),
  and one of them is the batch's strongest: **a non-stacking cap the engine
  breaks.** ArMDE:5000 says "you may take this Virtue more than once, **but you
  only gain a +3 bonus even if more than one set of circumstances applies**";
  `max_per_target: 255` permits the repeat and `casting_mod_for` **sums**, so
  two copies give **+6** and three give +9. `RULES.md:1527` quotes that exact
  sentence in its repeat-rules table and acts on only its first half. The other
  three — a conditional bonus folded in flat, an `aura_bonus` kind documented as
  a *different* Virtue's semantic, and no parameter for the circumstances — are
  partly on record and are downgraded accordingly.
- **`virtue.strong_faerie_blood` carries four more** (F-291…F-294), including a
  flat "You may not have both Faerie Blood and Strong Faerie Blood"
  (ArMDE:5044) that appears in **neither** entry's `incompatible_with`, and a
  natural-longevity onset age of fifty that no `AgingEffect` kind can express —
  the latter a **known** engine gap `aging.rs::aging_schedule` documents by
  name, so only its text half is reported (F-293).
- **The most productive cross-reference in the batch is the shortest.**
  `virtue.skinchanger_dove`'s entire first paragraph is "This is a more powerful
  variant of the Minor Virtue above" (ArMDE:4978) — a page-number-free
  named-Virtue pointer that imports the whole of `virtue.skinchanger`'s rule
  set, so that everything the Major entry then says is a *delta* on rules stated
  only at ArMDE:4974. → F-279.
- **And one entry passed every check including its cross-reference**:
  `virtue.spirit_votary`, whose "(page 63)" pointer lands on ArMDE:2633-2639 and
  ArMDE:2741-2764 and whose four rules turn out to be encoded in three different
  files — the V/F entry, `character_types.json` and
  `mythic_companion_types.json`. It is the audit's cleanest worked example of
  C7's "modelled by something other than an effect".

*(This file was written incrementally — the method notes and the verdict table
landed first, findings were appended as each entry was finished, and the
sub-agent reconciliation last.)*

## Method

The whole span was read as continuous prose in **both** languages before any
entry was judged — `rules/source/en/Ars Magica - Definitive Edition (Core
Rules).md` ArMDE:4876-5100 and the line-parallel
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md` 4876-5100.

**Line parity holds throughout the span**, checked at every heading rather than
sampled: 4884, 4888, 4892, 4896, 4900, 4904, 4910, 4922, 4926, 4930, 4934, 4940,
4946, 4950, 4954, 4958, 4964, 4968, 4972, 4976, 4988, 4992, 4998, 5002, 5006,
5010, 5022, 5032, 5048, 5052, 5056, 5073, 5077, 5085, 5089 — **35 `####`
headings for 35 entries**, one each, and each falls on the same line number in
both files. The only non-entry heading inside the span is the block-quote
sub-heading `> #### Study Bonus Examples` at 5060, inside
`virtue.study_bonus`'s own range, which is correctly claimed by nobody in its
own right.

**`source.lines` (check 1) — 35 of 35 correct.** The dominant convention is
"own heading → the line before the next heading"; three entries
(`second_sight` 4888-4890, `self_confident` 4900-4902, `skilled_parens`
4964-4966) stop one line earlier, on their last body line rather than on the
trailing blank, which is tighter and equally correct. Every **non-blank** line
in 4884-5096 is cited by exactly one entry; the only uncited lines are the
blanks 4891, 4903 and 4967. No range stops short of its own rule and no range
swallows a neighbour's — the two shapes B06 and B07 each found once. This batch
draws **no check-1 finding**.

The 32/3 convention split is the same one B07 recorded as a normalization
decision owed before the upstream source re-sync; it is not re-raised here as a
finding.

**No entry in this batch carries an `anchor`** — `jq` over the batch's 35
`source` objects returns the key set `["file","lines"]` and nothing else, so
B11's "confirm the anchor names the entry" check has nothing to verify and the
line ranges are the only provenance.

**Magnitude, kind, categories, entity_kinds (checks 3, 4, 5, 6).** Every
descriptor line was read and compared against the data. `kind` is `virtue` for
all 35 and all 35 sit inside the Virtues section — correct. `entity_kinds` is
`["character"]` on all 35, which is right: none is a covenant Boon. `magnitude`
and `categories` agree with **every** descriptor:

- `*Minor, General*` — schooled_in_crime, self_confident, sharp_ears,
  skilled_smuggler, social_contacts, strong_willed, student_of_realm
- `*Minor, Supernatural*` — second_sight, see_in_darkness,
  sense_holiness_and_unholiness, skinchanger, strong_angelic_heritage
- `*Major, Supernatural*` — sense_passions, shapeshifter, skinchanger_dove,
  spiritual_pact (printed `*Major, Subernatural*` — an OCR typo at ArMDE:5011;
  the data reads it correctly as `supernatural`), strong_faerie_blood,
  summon_animals, supernatural_beauty
- `*Minor, Hermetic*` — secondary_insight, side_effect, skilled_parens,
  special_circumstances, spell_improvisation, study_bonus, subtle_magic
  (printed `*Minor. Hermetic*` at ArMDE:5074 — a period for a comma; parses
  unambiguously and the data reads it correctly)
- `*Minor, Social Status*` — senior_bard, shadchan, simple_student
- `*Major, Social Status*` — senior_clergy, senior_master
- `*Free, Social Status*` — shamash, sofer
- `*Free, Mythic Companion*` — spirit_votary
- `*Minor, Social Status, Supernatural*` — sufi, dual-category ✓ with the
  `taken_as` `category`-domain parameter and the `max_total: 1` that domain
  requires, which is exactly the shape `engine-semantics.md` § B5 cites
  ArMDE:5083 for

No entry in this batch sets `tainted`, and none of the 35 descriptor lines
carries the Tainted type label — the first one that does is `Tainted Treasure`
at ArMDE:5098, one line past the span.

**Check 12, mechanical.** Every `name` and `summary` was read against its
passage in the matching language. `jq` over the batch's 35 entries in **each**
locale returns the key set `["name","summary"]` — except `virtue.student_of_realm`
in **de**, which also carries `name_unfilled`. **No entry in this batch carries
a `description` in either locale**, which is why every D5 finding below reads
"reaches neither locale" rather than "is wrong in one".

The one place the span prints a negative sign inside a summary is ArMDE:4970
("a –9 penalty", DE 4970 "einen Abzug von –9"), an en dash in the source; both
locales' summaries transcribe it with the **ASCII hyphen** `-9` ✓, as CLAUDE.md
requires. Two further en dashes in the span that any correction-pass
`description` must likewise transcribe as ASCII hyphens: ArMDE:4974 ("Size –10
(robin)", DE "Größe –10 (Rotkehlchen)") and ArMDE:5036 ("–3 to Aging Rolls",
DE "–3 auf Alterungswürfe"). `virtue.strong_faerie_blood`'s -3 lives in the data
as `"amount": -3`, a JSON number, so no glyph question arises there.

### German names against the canonical tables

Checked against `rules/source/de/translation-tables/` **and**
`rules/i18n/de/abilities.json`, not only against the DE rulebook. Twenty-three
of the batch's German names appear in a table; **twenty-two agree with it and
one does not** (`virtue.senior_bard`, F-263).

| Entry | Table row | Table's DE | Data's DE | |
|---|---|---|---|---|
| Second Sight | `tugenden-fehler.md:141`; `fertigkeiten.md:80` | Zweites Gesicht | Zweites Gesicht | ✓ |
| Secondary Insight | `tugenden-fehler.md:67` | Nebenerkenntnis | Nebenerkenntnis | ✓ |
| Self-Confident | `tugenden-fehler.md:213` | Selbstbewusst | Selbstbewusst | ✓ |
| Senior Bard (Cano/Cli) | `reputationen.md:90` | **Hochrangiger Barde** | **Älterer Barde** | ✗ **F-263** |
| Senior Clergy | `tugenden-fehler.md:753` | Höherer Klerus | Höherer Klerus | ✓ (the row also records a corrected earlier error, "nicht *Hoher Klerus*") |
| Sense Holiness and Unholiness | `fertigkeiten.md:81` | Gespür für Heiliges und Unheiliges | Gespür für Heiliges und Unheiliges | ✓ — but `tugenden-fehler.md:142` gives a **different** form for the same term; see **Q-65** |
| Sense Passions | `tugenden-fehler.md:101`, `:143`; `fertigkeiten.md:82` | Gespür für Leidenschaft | Gespür für Leidenschaft | ✓ |
| Shapeshifter | `tugenden-fehler.md:102`; `fertigkeiten.md:83` | Gestaltwandler | Gestaltwandler | ✓ |
| Side Effect | `tugenden-fehler.md:68` | Nebeneffekt | Nebeneffekt | ✓ |
| Skilled Parens | `tugenden-fehler.md:69` | Erfahrener Parens | Erfahrener Parens | ✓ |
| Skinchanger | `tugenden-fehler.md:144` | Tierwandler | Tierwandler | ✓ (and the derived `Tierwandler (Taube)` matches DE 4976 ✓) |
| Social Contacts | `tugenden-fehler.md:214`; `reputationen.md:84` | Soziale Kontakte | Soziale Kontakte | ✓ — but `reputationen.md:84`'s *Anmerkung* attributes a rule ArMDE:4990 does not state; see **Q-69** |
| Special Circumstances | `tugenden-fehler.md:70` | Besondere Umstände | Besondere Umstände | ✓ |
| Spirit Votary | `konvent.md:57`; `tugenden-fehler.md:257`; `sphären-mächte.md:206` | Geistverehrer | Geistverehrer | ✓ — and the **DE rulebook heading at 5006 says "Geistesdiener"**. The data correctly followed the table over the rulebook, which is what CLAUDE.md requires. Recorded so a later reader does not "fix" it back. |
| Spiritual Pact | `tugenden-fehler.md:103` | Geistiger Pakt | Geistiger Pakt | ✓ |
| Strong Angelic Heritage | `tugenden-fehler.md:745` | Starkes Engelserbe | Starkes Engelserbe | ✓ |
| Strong Faerie Blood | `tugenden-fehler.md:104` | Starkes Feenblut | Starkes Feenblut | ✓ |
| Strong-Willed | `tugenden-fehler.md:215` | Willensstark | Willensstark | ✓ |
| Student of (Realm) | `tugenden-fehler.md:216`; `grundbegriffe.md:399` | Student der (Sphäre) / Student der Magiesphäre | `name_unfilled` = "Student der (Sphäre)" ✓, `name` = "Student einer Sphäre, {realm}" | ✓ — the table's value is honoured in `name_unfilled`; the comma-apposition filled form is the **documented fix** for ungrammatical German (`RULES.md:2355-2391`). F-299 raised this and is **WITHDRAWN**. |
| Study Bonus | `tugenden-fehler.md:71` | Studierbonus | Studierbonus | ✓ |
| Subtle Magic | `tugenden-fehler.md:72` | Subtile Magie | Subtile Magie | ✓ |
| Summon Animals | `tugenden-fehler.md:105`, `:145`; `fertigkeiten.md:86` | Tiere rufen | Tiere rufen | ✓ |
| Supernatural Beauty | `tugenden-fehler.md:106` | Übernatürliche Schönheit | Übernatürliche Schönheit | ✓ |

Two table rows are **near-misses that are not findings**, recorded so they are
not re-raised:

- `reputationen.md:89` — `Senior Bard (Anruth) | Meisterbarde (Anruth) | Lokal |
  3 (+)`. A **different** Senior Bard rank: ArMDE:4906 knows only "Cano or Cli"
  with a Local Reputation of **2**, which is exactly `reputationen.md:90`'s row.
  The Anruth row (Reputation 3) has no counterpart in the core catalogue.
- `tiere-kreaturen.md:87` — `Sharp Ears | Spitze Ohren | +1 Wahrnehmung; +3 auf
  Hörwürfe`. That is the **animal/creature Quality** of the same name, which
  carries a +1 Perception the core Virtue at ArMDE:4952 does not have. Different
  item, same English name — B07's `Perfectus` / `Notary` shape. The data's
  `Scharfe Ohren` matches the DE rulebook heading at 4950 ✓.

The remaining twelve German names appear in no table (`schooled_in_crime`,
`see_in_darkness`, `senior_master`, `shadchan`, `shamash`, `simple_student`,
`skilled_smuggler`, `sofer`, `sufi`, `spell_improvisation`,
`skinchanger_dove` as a derived form, `strong_angelic_heritage`'s core reading),
so the table constraint does not bind them; each was checked against its DE
rulebook heading instead and all match exactly — including
`virtue.skilled_smuggler`, whose `Erfahrener Schmugler` faithfully reproduces a
**spelling error in the DE source itself** (`Schmugler`, one `g`; correct German
is `Schmuggler`). That is F-276.

**Eleven DE Ability names the passages depend on were confirmed against
`rules/i18n/de/abilities.json`**, not inferred: Second Sight → **Zweites
Gesicht**, Sense Holiness and Unholiness → **Gespür für Heiliges und
Unheiliges**, Sense Passions → **Gespür für Leidenschaft**, Shapeshifter →
**Gestaltwandler**, Summon Animals → **Tiere rufen**, Faerie Lore →
**Feenkunde**, Magic Lore → **Magiekunde**, Dominion Lore → **Dominiumkunde**,
Art of Memory → **Gedächtniskunst**, Theology: Islam → **Theologie: Islam**,
Islamic Law → **Islamisches Recht**. All eleven match the DE rulebook body at
ArMDE:4890, :4908, :4928, :4932, :4948, :5040, :5081 and :5087. Latin is not an
Ability id at all — the catalogue has `ability.dead_language` with
`"parameter": "language"` (DE `{language} (Tote Sprache)`, `name_unfilled`
"Tote Sprache"), which is what makes `virtue.simple_student`'s "Latin or Artes
Liberales" only half-expressible (F-273).

### Cross-references followed

Every pointer in the span was followed and read — the `(see page NNN)` form, the
chapter form and the page-number-free named-Virtue form alike — including on
entries that turned out to pass.

| Entry | Pointer | Where it lands | Does it add a rule attributed to the Virtue? |
|---|---|---|---|
| `virtue.second_sight` | "(see page 421)" / `#regiones-betreten-und-verlassen` | the regio rules | **No** — general rules; the Virtue states only that its Ability perceives regio boundaries. |
| `virtue.second_sight` | "(page 170)" / `#zweites-gesicht-1` | the Second Sight **Ability** | **No** — the Ability's own rules, and the Virtue's mechanic (the score-1 grant) is encoded ✓. |
| `virtue.self_confident` | "(See page 52 for Confidence rules.)" | the Confidence rules | **No** — general rules. *(The DE anchor at 4902 reads `#reputationen-1`, a **source-side anchor defect**: the DE anchor points at Reputations, not Confidence. Recorded, not rated — it is a rulebook-file defect, not a catalogue one.)* |
| `virtue.senior_clergy` | "The Paid Rights Virtue" (named, no page) | `virtue.paid_rights`, ArMDE:4606-4615 ✓ exists | **Yes, one clause** — "The Paid Rights Virtue does **not** enable women to take any of the other positions", an explicit *non*-override of a Virtue that overrides elsewhere. In neither locale → F-265. |
| `virtue.sense_holiness_and_unholiness` | "(page 170)" | the Ability | **No.** |
| `virtue.sense_passions` | "(page 170)" | the Ability | **No.** |
| `virtue.shamash` | "**the Educated (Hebrew) Virtue**" (named, no page) | `#### Educated (Hebrew)`, **ArMDE:3723** | **Yes, a hard requirement** ("must have"). The target is one of the **four known-missing `Educated` variants** (README's closed census), so the prerequisite is not expressible in the catalogue today → F-270. Same shape as B07's F-240 on `virtue.rabbi`. |
| `virtue.shapeshifter` | "(page 171)" | the Ability | **No.** |
| `virtue.simple_student` | "the Paid Rights Virtue" (named, no page) | `virtue.paid_rights` ✓ | **Yes** — a *conditional* override of the female-only restriction ("would allow them to take this Virtue elsewhere. However, at other universities they would not be able to graduate") → F-274. |
| `virtue.skinchanger_dove` | "**the Minor Virtue above**" (named, no page) | `#### Skinchanger`, **ArMDE:4972-4975** | **YES — the whole Minor Virtue's rule set, imported wholesale.** ArMDE:4978 is the entry's entire first paragraph; everything the Major Virtue then says is a *delta* on rules stated only at 4974 → F-279. The most productive instance of the named-cross-reference pattern in this batch. |
| `virtue.skinchanger_dove` | "*Between Sand & Sea*, page 107" | **outside `rules/source/en/`** | Nothing claimable (the Daughters of Four Fathers tradition). |
| `virtue.sofer` | "**the Educated (Hebrew) Virtue**" (named, no page) | `#### Educated (Hebrew)`, ArMDE:3723 | **Yes, a hard requirement**, identical to Shamash's → F-283. |
| `virtue.spell_improvisation` | "(see Similar Spells, page 260)" / `#ähnliche-zauber` | the Similar Spells rules | **No** — the baseline the Virtue's bonus attaches to. |
| `virtue.spirit_votary` | "a kind of Mythic Companion (**page 63**)" / `#mythische-gefährten-2` | `## Mythic Companions` **ArMDE:2633-2639** and `### Spirit Votary` **ArMDE:2741-2764** | **Yes — four rules, and every one of them is already encoded.** ArMDE:2637 ("These Virtues are incompatible with each other, and with The Gift, and are not available to grogs") is exactly the entry's four-way `incompatible_with` ✓, plus the grog exclusion, which `rules/core/character_types.json`'s grog profile carries by omitting `mythic_companion` from `permitted_categories` ✓. ArMDE:2638's two-for-one rate is on the `mythic_companion` profile as `virtue_points_per_flaw_point: 2` ✓ (C7's shape, and the entry's own ArMDE:5008 restates it). ArMDE:2748-2764's required Virtues/Flaws and minimum Ability scores are in `rules/core/mythic_companion_types.json` under `mythic_type.spirit_votary` ✓. **No finding.** |
| `virtue.strong_angelic_heritage` | "the guidelines for Holy Powers in *Realms of Power: The Divine Revised Edition*, from page 46" | **RoP:D is in `rules/source/en/`** | **No new rule attributed to the Virtue** — an alternative design system for its powers, offered as a choice ("or"). The Virtue's own power rules (Might cost = magnitude ÷ 2 rounded down, minimum 1; Initiative = Quickness) are stated in ArMDE:5028 itself → F-290. |
| `virtue.strong_faerie_blood` | "the Virtue Second Sight (**see page 106**)" / `#zweites-gesicht` | `#### Second Sight`, **ArMDE:4888-4890**, in this very span | **Yes, and it is already encoded** — `grants_selection: ["virtue.second_sight"]` ✓, the A18 shape. |
| `virtue.strong_faerie_blood` | "the Virtue Faerie Blood (**page 79**)" / `#feenblut` | `#### Faerie Blood`, **ArMDE:3797-3819** | **YES — seven blood-type benefit blocks, imported wholesale**, each with its own mechanics (Dwarf Blood +1 to Craft totals, Goblin Blood +1 to stealth totals, Satyr Blood +1 Com/Pre, Sidhe Blood +1 Presence to no more than +3, Undine Blood +2 underwater, Bee King Penetration 25, Spinnen body-weight conversion). The entry carries none of them and has no parameter recording which was chosen → F-294. |
| `virtue.strong_faerie_blood` | "you cannot lose it when being trained as a magus (**see page 64**)" / `#übernatürlich` | the Supernatural-Virtue-and-apprenticeship rule | **Yes, one clause** — and it is stated in ArMDE:5046 itself, not behind the pointer: "If your master cannot preserve the ability, you cannot be trained" → F-294. |
| `virtue.sufi` | "*Realms of Power: The Divine Revised Edition*, **page 116**, for details of the supernatural powers wielded by some Sufis" | **RoP:D is in `rules/source/en/`** | **No rule attributed to *this* entry** — the powers are RoP:D's own subsystem, and ArMDE:5079 offers "an entirely mundane Sufi" as the alternative. What ArMDE:5079-5081 states in its own voice is the Story-Flaw rule (F-303) and the three-Ability permission (F-302). |

**The named-Virtue form again produced the live hits** — Skinchanger (Dove)'s
"the Minor Virtue above", Strong Faerie Blood's Faerie Blood and Second Sight,
Shamash's and Sofer's Educated (Hebrew), Simple Student's and Senior Clergy's
Paid Rights are all page-number-free pointers a `(see page NNN)` screen cannot
see, and five of the six yield a rule the entry drops. That is now true in six
consecutive batches. The one `(page NNN)` pointer that *did* pay was
`virtue.spirit_votary`'s — and it paid by **confirming** four encodings rather
than exposing a gap, which is the first time in the audit that following a
pointer has cleared an entry rather than condemned one.

### Part C systemic gaps are not re-reported per entry

In particular: `advancement_mod` being surfaced-only (C1) is **not** counted
against `virtue.secondary_insight` or `virtue.study_bonus`;
`special_casting_mod`'s `spell_improvisation` kind being surfaced-only (C1) is
**not** counted against `virtue.spell_improvisation`;
`grants_reputation`'s unenforced `score` (C2-c) is **not** counted against
`virtue.senior_bard` or `virtue.senior_clergy`; "computes a number nothing
consumes" (C3c) is **not** counted against `virtue.self_confident`; and the
frontend `Effect` union's missing `power_levels`, `might_grant` and
`ability_authorization` tags (C6) are **not** counted against
`virtue.strong_angelic_heritage` or `virtue.student_of_realm`. What *is* counted
for each is what D5 obliges — the clause the effects do not implement, absent
from both locales.

**A gap outside Part C that is nonetheless already recorded, and is treated the
same way.** `aging.rs::aging_schedule` carries a doc comment headed "Known gap —
a trait may move the start age, and none does here", naming
`virtue.strong_faerie_blood` and ArMDE:5036 explicitly. Part C did not census
it, but it is a recorded engine decision all the same, so F-293 reports only the
half Part C's exclusion does **not** cover: that the rule appears in neither
locale. Found by reading `aging.rs` rather than by reading the entry, and worth
a general note — **an engine gap can be documented at its own call site rather
than in `engine-semantics.md`, so "not in Part C" does not mean "undiscovered".**

**One entry is a deliberate exception and needs saying plainly.**
`virtue.special_circumstances`'s `magic_resistance_mod` with
`kind: "aura_bonus"` is **not** reported as an instance of C1's "`AuraBonus` is
surfaced with `amount: 0`" gap. The complaint (F-286) is a different one:
`types.rs::MagicResistanceEffect`'s own doc comment defines `AuraBonus` as "A
bonus to Magic Resistance while in a matching aura (**Commanding Aura**)", and
this entry's passage describes no aura at all. That is a wrong *semantic*, in
the shape B07 found on `virtue.relic` — an effect that is not about the thing
the passage is about — not the engine's inability to carry the amount.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D1 and D4 do not bite.** No entry in this span carries a `lab_total_mod`
(`jq` over the batch's 35 entries returns none), so neither the spell-level-cap
ruling nor the conditional-Lab-Total ruling has a carrier here. D4's *shape*
recurs on the **casting** side instead — `virtue.special_circumstances`'s
conditional +3 folded flat as `scope: "all"` (F-284), which is the same
complaint B07 raised as F-225 for Potent Magic's casting half and which D4's
table does not cover.

**D5 is applied to all twenty effect-carrying entries in the span** —
`schooled_in_crime`, `second_sight`, `secondary_insight`, `self_confident`,
`senior_bard`, `senior_clergy`, `sense_holiness_and_unholiness`,
`sense_passions`, `shadchan`, `shapeshifter`, `skilled_parens`,
`special_circumstances`, `spell_improvisation`, `spirit_votary`,
`strong_angelic_heritage`, `strong_faerie_blood`, `student_of_realm`,
`study_bonus`, `subtle_magic`, `summon_animals`. **Eleven** leave a stated rule
that no effect implements and that appears in neither locale (F-258, F-259,
F-262, F-265, F-268, F-285, F-288, F-290, F-294, F-300, F-301). The nine
effect-carrying entries with nothing left over are **`virtue.second_sight`**,
**`virtue.self_confident`**, **`virtue.sense_holiness_and_unholiness`** (on the
D5 axis only — it carries Q-66), **`virtue.sense_passions`**,
**`virtue.shapeshifter`**, **`virtue.skilled_parens`**,
**`virtue.spirit_votary`**, **`virtue.summon_animals`** and — on the D5 axis
only — `virtue.student_of_realm`, whose findings are a missing computation and
an over-wide permission rather than a missing description.

`virtue.simple_student` is `creation_effect` with **no** effects at all and is
handled separately below as the directed read.

**D5 was also applied in the negative** — the passage of the one `narrative`
entry that survives (`virtue.see_in_darkness`, ArMDE:4898) was read for a
mechanical clause and found to state none that the core rulebook gives a number
or a roll for. That verdict is marked `?` and carries **Q-64**, because the
reasoning rests on an absence I searched for rather than on a positive reading.

**D3 governs six findings** where the engine structurally cannot express the
rule and the answer is therefore a `description` in both locales (or
`uncomputed_rule` for an effect-less entry), and **never** `narrative`:

| Finding | The rule | Why the engine cannot express it |
|---|---|---|
| F-261 | `virtue.senior_bard`'s "minimum age of 22" | No `Prereq` variant reads age (B6's eight variants). `validation/life_stage.rs` has one *global* minimum, from `LifeStageRules`, not a per-Virtue one. Exactly what B07 found for Rosh Beth Din's `(30 – Int)`. |
| F-270, F-283 | Shamash's and Sofer's "must have the Educated (Hebrew) Virtue" | The target id does not exist in the catalogue (README's closed four-missing-`Educated` census), so `Prereq::Has` has nothing to name. |
| F-273 | `virtue.simple_student`'s "Latin or Artes Liberales" | Latin is an *instance* of `ability.dead_language`. A7 records that `restricted_ability_xp_pools` hard-codes `instances: Vec::new()`, so a pool can name the Ability but never one language. |
| F-268 | `virtue.shadchan`'s "an Area Lore appropriate for their community" | Same reason — the pool covers every Area Lore instance, and A7 cannot scope one. |
| F-293 | `virtue.strong_faerie_blood`'s "You start making aging rolls at the age of fifty, rather than the normal 35" | `AgingEffect`'s eight kinds are `aging_roll`, `living_conditions`, `no_aging`, `no_apparent_aging`, `crisis_heavy_wound`, `crisis_survival`, `decrepitude`, `longevity_bonus`. **None moves the onset age**, which is the ruleset-wide `aging.json` → `start_age: 35`; `no_aging` suppresses rolls entirely, a different rule. `aging.rs::aging_schedule` documents this gap by name, so only the *text* half is reported. |
| F-298 | `virtue.student_of_realm`'s "You may not take Student of (Realm) and Puissant Ability for the same Lore" | `incompatible_with` is a set of ids and is all-or-nothing (B7); it cannot be scoped to a shared **parameter value**, and both entries are parameterized. |
| F-305 | `virtue.supernatural_beauty`'s "A character lacking a positive Presence score may not have this Virtue" | `Prereq` has no characteristic-minimum variant at all (B6). Flagged for this batch by B05 and confirmed here from the source. |

**D2 does not bite:** no entry here is a granted Great Characteristic carrier.

**ArMDE:2816 ("All characters must take one Social Status, and may only take
more than one if the descriptions … explicitly note that they are
compatible").** Eight entries in the span are Social Status Virtues —
`senior_bard`, `senior_clergy`, `senior_master`, `shadchan`, `shamash`,
`simple_student`, `sofer`, and `sufi` (dual). Instances are noted only, per the
brief. One of them, `virtue.shadchan`, states the **compatibility override**
ArMDE:2816 anticipates ("this Virtue is compatible with any other Minor or Free
Social Status Virtue", ArMDE:4936), and that is a rule of its own → F-267. For
the record and without proposing a fix: `jq` over
`rules/core/character_types.json` shows **no** `virtue_category_caps` entry for
`social_status` on any of the four profiles (only magus's `hermetic` major cap
exists), so the base rule is enforced nowhere today and the override is
currently inert.

**ArMDE:2960-2962 (realm association of every Supernatural Virtue).** Thirteen
entries in the span are Supernatural — `second_sight`, `see_in_darkness`,
`sense_holiness_and_unholiness`, `sense_passions`, `shapeshifter`,
`skinchanger`, `skinchanger_dove`, `spiritual_pact`, `strong_angelic_heritage`,
`strong_faerie_blood`, `sufi` (dual), `summon_animals`,
`supernatural_beauty` — and **none carries a `realm` parameter**. Instances
noted only. Two are the passage's own named exceptions and are therefore *not*
free choices: ArMDE:2960 names "**Faerie Blood and Strong Faerie Blood**" as
"always associated with Faerie", and ArMDE:5026 fixes Strong Angelic Heritage's
powers to the Divine — the latter *is* carried, by the entry's
`might_grant { realm: "divine", score: 0 }` ✓, which A26 documents as the
establish-the-realm-without-points idiom.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.schooled_in_crime` | 4884-4887 | OK | OK | the open-ended troupe extension in neither locale | F-258 |
| `virtue.second_sight` | 4888-4890 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.secondary_insight` | 4892-4895 | OK | OK | the 4-Forms/2-Techniques split, the one-point-per-Art cap and the no-Affinity rule in neither locale | F-259 |
| `virtue.see_in_darkness` | 4896-4899 | OK **?** | OK | OK | clean; ArMDE:2960 instance; **Q-64** |
| `virtue.self_confident` | 4900-4902 | OK | OK | OK | clean |
| `virtue.senior_bard` | 4904-4909 | OK | **"any Realm Lore" permission narrower than the passage — Dominion and Infernal Lore unauthorized**; the minimum age carried nowhere | the age, the Hibernia restriction, the score-5 guideline and the Profession over-scope in neither locale; DE name contradicts the canonical table | F-260 F-261 F-262 F-263 ArMDE:2816 |
| `virtue.senior_clergy` | 4910-4921 | OK | **effects missing (`ability_authorization`, academic)** | the good/bad Reputation choice, celibacy/tonsure, the Abbess-only restriction and the Paid Rights non-override in neither locale | F-264 F-265 ArMDE:2816 |
| `virtue.senior_master` | 4922-4925 | narrative → creation_effect | effects missing (`ability_authorization`, academic) | OK | F-266 ArMDE:2816 |
| `virtue.sense_holiness_and_unholiness` | 4926-4929 | OK **?** | OK | OK **?** | clean; **Q-65 Q-66**; ArMDE:2960 instance |
| `virtue.sense_passions` | 4930-4933 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.shadchan` | 4934-4939 | OK | **the Social-Status compatibility override encoded nowhere** | the override, "social Abilities" and the community-Area-Lore scoping in neither locale | F-267 F-268 ArMDE:2816 |
| `virtue.shamash` | 4940-4945 | narrative → uncomputed_rule | required Educated (Hebrew) encoded nowhere (target entry missing from the catalogue) | the requirement in neither locale | F-269 F-270 ArMDE:2816 |
| `virtue.shapeshifter` | 4946-4949 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.sharp_ears` | 4950-4953 | narrative → uncomputed_rule | OK | "+3" in neither locale | F-271 |
| `virtue.side_effect` | 4954-4957 | narrative → uncomputed_rule | OK | the level-scaling rule and the two worked bonuses in neither locale | F-272 |
| `virtue.simple_student` | 4958-4963 | OK — `creation_effect` with no effects is a **recorded deferral** (`RULES.md:5225-5228`) | **`ability_authorization` missing (two academic ids)** — not covered by that deferral | the per-year rate, the Latin instance scoping, the female/Salerno restriction and the Paid Rights override in neither locale | F-273 F-274 ArMDE:2816 **Q-67** |
| `virtue.skilled_parens` | 4964-4966 | OK | OK | OK | clean |
| `virtue.skilled_smuggler` | 4968-4971 | narrative → uncomputed_rule | OK | the -9 penalty is **in both summaries** while the class denies it; the dagger-size and swap clauses in neither locale | F-275 F-276 |
| `virtue.skinchanger` | 4972-4975 | narrative → uncomputed_rule | OK | +3 Soak, the Size -10…+2 band, the one-round change, the Arcane-Connection-if-stolen rule and the one-season remake in neither locale | F-277 |
| `virtue.skinchanger_dove` | 4976-4987 | narrative → uncomputed_rule | OK | +3 Soak, the dove statistics, the one-week preparation — **and the whole of the Minor Virtue it imports** — in neither locale | F-278 F-279 |
| `virtue.social_contacts` | 4988-4991 | narrative → uncomputed_rule | no parameter for the social circle — a **recorded deferral** (`RULES.md:2509-2517`, save-compatibility cost) | the Presence roll against Ease Factor 6 **and** the distinctness rule in neither locale | F-280 F-281 **Q-69** |
| `virtue.sofer` | 4992-4997 | narrative → uncomputed_rule | required Educated (Hebrew) encoded nowhere (target entry missing from the catalogue) | the requirement and the read/write-Hebrew clause in neither locale | F-282 F-283 ArMDE:2816 |
| `virtue.special_circumstances` | 4998-5001 | OK | **conditional casting bonus folded in flat**; **two copies give +6 where the book caps at +3**; **`aura_bonus` is the wrong variant semantic**; no parameter for the circumstances | the "only if she has Magic Resistance from another source" condition and the non-stacking cap in neither locale | F-284 F-285 F-286 F-287 |
| `virtue.spell_improvisation` | 5002-5005 | OK | OK | the fast-cast/Mastery interaction and the double non-stacking rule in neither locale | F-288 |
| `virtue.spirit_votary` | 5006-5009 | OK | OK | OK | clean — the only entry whose cross-reference yielded nothing owed |
| `virtue.spiritual_pact` | 5010-5021 | narrative → uncomputed_rule | OK — the "single pact" limit is already enforced by `max_per_target`'s default of 1 | the Confidence cost, the Pre + Magic Lore roll, the botch consequence, the no-Magic-Resistance and no-vis clauses and the pool-replacement rule in neither locale | F-289 |
| `virtue.strong_angelic_heritage` | 5022-5031 | OK | OK | the age÷20 Might formula, the vis formula, the Warping immunity, the Divine-only power restriction, the Might cost formula and the Initiative rule in neither locale | F-290 ArMDE:2960 |
| `virtue.strong_faerie_blood` | 5032-5047 | OK | **`incompatible_with: ["virtue.faerie_blood"]` missing on both sides**; `ability_authorization` (Faerie Lore, arcane) missing | the onset age (a **documented** engine gap, text half only), the darkness sight, the seven imported blood types and the apprenticeship clause in neither locale | F-291 F-292 F-293 F-294 ArMDE:2960 |
| `virtue.strong_willed` | 5048-5051 | narrative → uncomputed_rule | OK | "+3" in neither locale | F-295 |
| `virtue.student_of_realm` | 5052-5055 | OK | the +2 carried by no effect; `ability_authorization` names all four Lores where the passage permits one; the Puissant exclusion encoded nowhere — **all three recorded in `RULES.md`** | the +2, the over-permission's cost and the exclusion in neither locale | F-296 F-297 F-298 (F-299 **withdrawn**) |
| `virtue.study_bonus` | 5056-5072 | OK | OK | the presence-of-the-Art condition and the whole eight-row magnitude table in neither locale | F-300 |
| `virtue.subtle_magic` | 5073-5076 | OK | OK | the normal/exaggerated gesture clause in neither locale **?** | F-301 **Q-68** |
| `virtue.sufi` | 5077-5084 | narrative → creation_effect | effects missing (`ability_authorization`: two academic ids + one arcane id) | the no-points Story Flaw rule in neither locale | F-302 F-303 ArMDE:2816 ArMDE:2960 |
| `virtue.summon_animals` | 5085-5088 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.supernatural_beauty` | 5089-5096 | narrative → uncomputed_rule | **the positive-Presence precondition encoded nowhere** | the once-per-story use, the troupe veto and the precondition in neither locale | F-304 F-305 |

**Totals: 26 entries carry at least one finding, 9 are clean** —
`virtue.second_sight`, `virtue.see_in_darkness`,
`virtue.self_confident`, `virtue.sense_holiness_and_unholiness`,
`virtue.sense_passions`, `virtue.shapeshifter`, `virtue.skilled_parens`,
`virtue.spirit_votary`, `virtue.summon_animals`. 26 + 9 = 35 ✓.

Two of the nine clean rows are **marked `?` and carry a question with no
finding** — `virtue.see_in_darkness` (Q-64) and
`virtue.sense_holiness_and_unholiness` (Q-65, Q-66). Per the audit's rules those
two are **not marked checked**: their reading could not be settled from the
source and is escalated rather than decided.

## Findings

**A qualifier that applies to all six `ability_authorization` findings
(F-260, F-264, F-266, F-273, F-292, F-302), stated once rather than six
times.** A14 records that
`validation/authorization.rs::validate_ability_authorization` errors
(`ability_category_requires_virtue`) on a held Ability whose category is in
`ruleset.categories_requiring_virtue()` unless its id or its category is
authorized, with a whole-character exemption for a profile whose `is_magus` is
true. That gated set is `rules/core/abilities.json` →
`"categories_requiring_virtue": ["academic", "arcane", "martial"]` (read
directly). Every carrier below is a **mundane** Virtue — a bard, a bishop, a
guild master, a university student, a Sufi mystic, a half-faerie — so the
`is_magus` exemption is empty in practice and the refusal is exactly what bites.
A7 also records that a `restricted_ability_xp` pool confers the same permission
for what it funds, so the check is always "does the pool's `abilities` /
`categories` union cover what the passage permits" — which is how
`virtue.schooled_in_crime` and `virtue.shadchan` escape an authorization finding
(every id in their pools is `general`, an ungated category) while
`virtue.senior_bard` does not.

**`crates/arm-rules/RULES.md` confirms that these six are genuine gaps and not
settled decisions, and it names the fix path itself.** Its authorization section
(`RULES.md:6350-6365`) lists exactly **one** explicitly-wired Virtue —
`flaw.covenant_upbringing` — plus the four covered incidentally by pools
(Educated, Warrior, Arcane Lore, Privileged Upbringing), and then says, verbatim:

> **Other access-granting Virtues are a data addition, never a code change.**

None of `virtue.senior_bard`, `virtue.senior_clergy`, `virtue.senior_master`,
`virtue.simple_student`, `virtue.strong_faerie_blood` or `virtue.sufi` appears
anywhere in that section (each id was searched in RULES.md individually). So the
catalogue's authorization coverage is acknowledged to be partial and extendable
by data alone — which is what makes 53 instances across eight batches a
systematic data gap rather than 53 independent oversights.

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

### F-258 — `virtue.schooled_in_crime` — the troupe-extension clause reaches neither locale (D5)

**Passage** (ArMDE:4886, verbatim — the entry's whole body):
> The character has been trained extensively in the criminal arts, and has 50 additional experience to spend on Abilities related to thievery. Area Lore, Athletics, Awareness, Bargain, Brawl, Charm, Guile, Legerdemain and Stealth are all suitable, **and other Abilities may be added to this list with troupe approval**.

German, ArMDE:4886 (line-parallel):
> Der Charakter wurde ausgiebig in kriminellen Künsten ausgebildet und erhält 50 zusätzliche Erfahrungspunkte für Fertigkeiten, die mit Diebstahl in Verbindung stehen. Gebietskundearten, Athletik, Wahrnehmung, Feilschen, Raufen, Charme, Täuschung, Fingerfertigkeit und Schleichen sind alle geeignet; **andere Fertigkeiten können mit Zustimmung der Spieltruppe zu dieser Liste hinzugefügt werden**.

**Current data.** `creation_effect` ✓, one effect:
`restricted_ability_xp` with `amount: 50` and the nine ids
`ability.area_lore`, `ability.athletics`, `ability.awareness`,
`ability.bargain`, `ability.brawl`, `ability.charm`, `ability.guile`,
`ability.legerdemain`, `ability.stealth` — **exactly the nine the passage
names, in the passage's own order**, and all nine are `general` in
`rules/core/abilities.json`, so A7's permission side-effect grants nothing the
character did not already have. The 50 and the list are right.

**What is left over.** The final clause. The pool is a closed id list (A7); a
troupe-added tenth Ability is not expressible, and nothing tells the player the
list is open. Both `summary` fields stop at "…related to thievery." and drop the
whole second sentence, so neither the nine names **nor** the openness reaches
the user.

**Correct value.** Keep the class and the effect; add a `description` in both
locales carrying the nine suitable Abilities and the troupe-approval extension.

**Severity.** Lost rule — a GM-adjudicated widening the player cannot discover
from the app.

---

### F-259 — `virtue.secondary_insight` — three of the four clauses reach neither locale (D5)

**Passage** (ArMDE:4894, verbatim — the entry's whole body):
> Your method of magical study is especially versatile. When you spend a season studying one of the magical Techniques from a book, a teacher, or raw vis, **you also gain a single experience point in any 4 Forms of your choosing**. When studying one of the magical Forms, **you also gain a single experience point in any 2 separate Techniques of your choosing**. **You may not put more than one bonus experience point into a single Art**, and may choose different Arts to receive the bonus experience points in each season, even when continuing to study the same Art from the same source. **These bonus experience points are not increased by Affinities or any other factors.**

German, ArMDE:4894 (line-parallel):
> Deine Methode des magischen Studiums ist besonders vielseitig. Wenn Du ein Quartal damit verbringst, eine der magischen Techniken aus einem Buch, bei einem Lehrer oder aus rohem Vis zu studieren, erhältst Du außerdem **einen einzelnen Erfahrungspunkt in beliebigen 4 Formen Deiner Wahl**. Wenn Du eine der magischen Formen studierst, erhältst Du außerdem **einen einzelnen Erfahrungspunkt in beliebigen 2 separaten Techniken Deiner Wahl**. **Du kannst nicht mehr als einen Bonus-Erfahrungspunkt in eine einzelne Kunst einbringen** […] **Diese Bonus-Erfahrungspunkte werden durch Affinitäten oder andere Faktoren nicht erhöht.**

**Current data.** `in_play_effect` ✓ (it carries an effect, which B1 records as
what decides the line against `uncomputed_rule`), one effect:
`{"type": "advancement_mod", "source": "insight", "amount": 1}`.

**The `source` value is right, and that is worth saying because it looks
wrong.** `types.rs::AdvancementSource::Insight`'s doc comment reads "Insight
into a magical topic (**Secondary Insight**)" — the variant was created for this
entry. So naming `insight` rather than `book`/`vis`/`taught` is deliberate, and
the `amount: 1` correctly encodes "a **single** experience point". No finding
there.

**What is left over — three clauses, none of which any effect implements and
none of which is in either `summary`** (both stop at the first sentence,
"Your method of magical study is especially versatile." / "Deine Methode des
magischen Studiums ist besonders vielseitig."):

1. **The 4-vs-2 asymmetry.** Studying a *Technique* yields four bonus points
   (one each into four Forms); studying a *Form* yields two (one each into two
   Techniques). One `amount: 1` row carries neither count nor the direction.
2. **The one-point-per-Art cap**, plus the freedom to re-choose each season.
3. **"not increased by Affinities or any other factors"** — an explicit
   *exclusion* from `affinity_art_cost` (A5), which the engine otherwise applies
   to every Art spend.

**Correct value.** Keep the class and the effect; write all three clauses into
`description` in both locales.

**Severity.** Lost rule. `advancement_mod` is surfaced-only by design (C1, not
re-reported), so the entry reaches the user as the bare label "insight +1" and
the player has no way to learn that it is four points, or two, or capped.

---

### F-260 — `virtue.senior_bard` — the passage permits **any** Realm Lore; the data authorizes two of the four

**Passage** (ArMDE:4908, verbatim, the first two sentences):
> The character has a minimum age of 22, and **you can spend experience points on any Realm Lore at character creation even if otherwise unable to take Arcane Abilities.** The character should have at least one Area Lore, Realm Lore, or Organization Lore at a score of 5, and has an extra **90 experience points to spend on Art of Memory, Profession: Storyteller, Profession: Poet, any Area Lore, any Organization Lore, Faerie Lore, or Magic Lore.**

German, ArMDE:4908 (line-parallel):
> Der Charakter hat ein Mindestalter von 22 Jahren, und **Du kannst bei der Charaktererschaffung Erfahrungspunkte für jede Sphärenkunde ausgeben, auch wenn Du andernfalls keine Arkanen Fertigkeiten erwerben könntest.** […] und erhält **90 zusätzliche Erfahrungspunkte für Gedächtniskunst, Beruf: Geschichtenerzähler, Beruf: Dichter, jede Gebietskunde, jede Organisationskunde, Feenkunde oder Magiekunde.**

**Current data.** `creation_effect` ✓, two effects:
`grants_reputation {kind: "local", score: 2}` ✓ (ArMDE:4906, "a Local Reputation
of 2") and `restricted_ability_xp` with `amount: 90` naming
`ability.area_lore`, `ability.art_of_memory`, `ability.faerie_lore`,
`ability.magic_lore`, `ability.organization_lore`, `ability.profession`.

**Why the pool's id list is right and the permission is not.** The 90-point pool
is a *funding* rule and its six ids match the passage's spend list ✓. But
ArMDE:4908 states a **second, wider** rule in its own sentence: the character
may spend on **any** Realm Lore — which `rules/core/abilities.json` shows to be
four Abilities, all `arcane`:

| Ability | category | in the pool? |
|---|---|---|
| `ability.faerie_lore` | `arcane` | ✓ |
| `ability.magic_lore` | `arcane` | ✓ |
| `ability.dominion_lore` | `arcane` | **✗** |
| `ability.infernal_lore` | `arcane` | **✗** |

A7's documented side-effect authorizes exactly what a pool funds, so a Senior
Bard may own Faerie Lore and Magic Lore but **not** Dominion Lore or Infernal
Lore: `validate_ability_authorization` raises
`ability_category_requires_virtue` — a hard **error** — on either. The passage
says "any Realm Lore", in as many words, and the DE says "jede Sphärenkunde".

**Correct value.** Add
`{"type": "ability_authorization", "abilities": ["ability.dominion_lore", "ability.infernal_lore"]}`
alongside the existing pool — the *bare permission* case A14 describes, because
the passage permits those two without funding them. (Listing all four is
equivalent; the pool already covers two.)

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-261 — `virtue.senior_bard` — the minimum age of 22 is carried nowhere (D3)

**Passage** (ArMDE:4908, verbatim): "**The character has a minimum age of 22**"
/ DE: "**Der Charakter hat ein Mindestalter von 22 Jahren**".

**Current data.** No `prerequisites`, no effect, and nothing in either
`summary`.

**Why it cannot be encoded, stated precisely.** B6 lists the eight `Prereq`
variants — `all`, `any`, `none`, `has`, `house`, `ability_min`, `art_min`,
`is_magus` — and **none reads `Entity::age`**. The only age check in the engine
is `validation/life_stage.rs::validate_life_stage_age_meets_minimum`, which
compares against a *ruleset-wide* floor (`LifeStageRules`'s childhood years, or
the minimum Gauntlet age for a magus) and takes no per-item input. There is no
per-Virtue age gate to attach 22 to.

This is the same structural gap B07 recorded for `virtue.rosh_beth_din`'s
"(30 – Int) years old", one entry earlier in the book, and D3 gives it the same
answer.

**Correct value.** The minimum age written into `description` in both locales.
No data change and no new `Prereq` variant is proposed here.

**Severity.** Wrong rules output — the app permits a 15-year-old Senior Bard —
mitigated to lost-rule by the fact that nothing could have caught it.

---

### F-262 — `virtue.senior_bard` — three further clauses reach neither locale (D5)

Beyond F-260 and F-261, ArMDE:4906-4908 states three things no effect carries
and neither `summary` mentions (both stop after the first sentence, "…is ranked
a Cano or Cli respectively." / "…als Cano oder Cli eingestuft."):

1. **"This Social Status is only available in Hibernia."** (ArMDE:4908) / "Dieser
   Sozialer Status ist nur in Hibernia verfügbar." A setting restriction with no
   field to hold it — the entity has no region.
2. **"The character should have at least one Area Lore, Realm Lore, or
   Organization Lore at a score of 5"** — hedged ("should"), so correctly
   encoded by encoding nothing, but it is still a stated guideline the player
   never sees.
3. **The Profession over-scope.** The passage funds two *named instances*,
   "Profession: Storyteller" and "Profession: Poet" (DE "Beruf:
   Geschichtenerzähler, Beruf: Dichter"). `ability.profession` carries
   `"parameter": "profession"`, and A7 records that
   `restricted_ability_xp_pools` hard-codes `instances: Vec::new()` — so the
   pool funds **every** Profession, a Profession: Blacksmith included. The
   engine cannot narrow it (D3), and nothing tells the player it should be
   narrowed.

The same applies, more mildly, to "any Area Lore" and "any Organization Lore",
where the wide reading is the *correct* one ✓.

**Correct value.** All three written into `description` in both locales.

**Severity.** Lost rule, plus a real over-permission on Profession that no test
can see (B07's over-permission pattern, in its structural rather than its
authored form).

---

### F-263 — `virtue.senior_bard` — the German name contradicts the canonical translation table

**Table row** (`rules/source/de/translation-tables/reputationen.md:90`,
verbatim):
> `| Senior Bard (Cano/Cli) | Hochrangiger Barde | Lokal | 2 (+) | Bekannter Geschichten­erzähler |`

**The row is unambiguously about this entry**: it names the Cano/Cli rank, the
**Local** audience and the score **2** — which is exactly ArMDE:4906 ("is ranked
a Cano or Cli respectively. The character has a Local Reputation of 2") and
exactly the entry's `grants_reputation {kind: "local", score: 2}`.

**Current data.** `rules/i18n/de/virtues_flaws.json` →
`"name": "Älterer Barde"`, which is the DE rulebook heading at 4904.

**Why it is a finding.** CLAUDE.md states the rule flatly: "when generating
`rules/i18n/de/` text, the German label for any term whose English form appears
in a table **MUST** match the table's `Deutsch (DE)` value." The tables are the
canonical EN→DE terminology mapping and they outrank the DE rulebook text — that
is precisely the precedence `virtue.spirit_votary` in this same batch follows
correctly (table "Geistverehrer" over rulebook "Geistesdiener"). Senior Bard
resolves the conflict the other way.

The neighbouring row `reputationen.md:89` (`Senior Bard (Anruth) | Meisterbarde
(Anruth) | Lokal | 3 (+)`) is a **different** rank with a different Reputation
score and is not this entry, so the disambiguation the table itself provides
does not rescue "Älterer Barde".

**Correct value.** `"name": "Hochrangiger Barde"` — or a recorded decision in
`decisions.md` that the DE rulebook wins here, which would then also have to
re-decide `virtue.spirit_votary` the other way round.

**Severity.** Localization / terminology consistency — one of the classes
CLAUDE.md rates **up**, since it is user-facing on every session.

---

### F-264 — `virtue.senior_clergy` — "You may purchase Academic Abilities" with no `ability_authorization`

**Passage** (ArMDE:4918, verbatim — the whole of its own paragraph):
> You may purchase Academic Abilities for the character during character generation.

German, ArMDE:4918 (line-parallel):
> Du kannst bei der Charaktererschaffung Akademische Fertigkeiten für den Charakter erwerben.

**Current data.** `creation_effect`, two effects, both `grants_reputation`:
`{kind: "local", score: 4}` and `{kind: "ecclesiastical", score: 4}` — which is
exactly ArMDE:4916 ("a Reputation of level 4 … in both the local community and
the Church"), and `ReputationType::Ecclesiastical` exists ✓. No
`ability_authorization` of any kind, and no `restricted_ability_xp` whose
side-effect could stand in for one.

**Why it is wrong.** `academic` is in `rules/core/abilities.json`'s
`categories_requiring_virtue`, so a bishop who buys Artes Liberales or Theology:
Christian gets `ability_category_requires_virtue` — a hard **error** — from
`validation/authorization.rs::validate_ability_authorization`. The passage
grants that permission in a sentence of its own, unhedged.

The `creation_effect` class is already correct (the two Reputations are computed
mechanics), so this is a pure missing-effect finding, not a reclassification.

**Correct value.** Add
`{"type": "ability_authorization", "categories": ["academic"]}`. This is A14's
bare-permission case: the passage grants no experience points, so
`ability_authorization` rather than `restricted_ability_xp` is the right
variant.

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-265 — `virtue.senior_clergy` — four clauses reach neither locale (D5)

Both `summary` fields stop at the first sentence ("The character is an
archbishop, bishop, abbot, abbess, or other senior member of the Church." /
"Der Charakter ist ein Erzbischof, Bischof, Abt, eine Äbtissin oder ein anderes
höheres Mitglied der Kirche."). Four stated clauses survive uncarried:

1. **The good/bad Reputation choice.** ArMDE:4916: "a Reputation of level 4,
   **either good or bad**, in both the local community and the Church" / DE
   "entweder gut oder schlecht". `types.rs::Reputation` declares `kind`, `score`
   and a free-text `content` and has **no good/bad axis**, so the choice is
   structurally unrecorded. This is the same clause B07 reported as F-236 for
   `virtue.protection`, so the correction pass should decide the two together.
2. **Celibacy and tonsure.** ArMDE:4920: "Senior clergy are subject to canon
   law, and must be celibate and tonsured (if male)."
3. **The Abbess-only restriction.** ArMDE:4920: "Abbess is the only common
   position available to women with this Virtue, and even such women are not
   ordained." `types.rs`'s concept field declares `pub gender: String` with "no
   mechanical effect", the same ground as B05's F-123 and B07's four gender
   findings (D3).
4. **The Paid Rights non-override.** ArMDE:4920: "The Paid Rights Virtue does
   **not** enable women to take any of the other positions." This one was found
   by following the named-Virtue pointer; it is a rule *about* another Virtue in
   the catalogue (`virtue.paid_rights` ✓ exists, ArMDE:4606-4615), and it is the
   **negative** of the override shape B07 recorded there.

Not a finding, and recorded so it is not re-raised: ArMDE:4914's "You may make
take either the Wealthy or Poor Virtue/Flaw for the character" is the
"explicit statement that *nothing* changes" shape B05, B06 and B07 all rated as
correctly encoded by encoding nothing — `virtue.wealthy` and `flaw.poor` are
both `general` with `entity_kinds: ["character"]` and are permitted on all four
type profiles already, so the sentence grants no permission the character
lacked.

**Correct value.** All four written into `description` in both locales.

**Severity.** Lost rule.

---

### F-266 — `virtue.senior_master` — `narrative` on an entry that grants a gated Ability category, with no `ability_authorization`

**Passage** (ArMDE:4924, verbatim — the entry's whole body; the operative
sentence is the last):
> The character has been a prosperous guild master for a number of years and has risen to a position of authority in his guild. He has knowledge of guild affairs and participates in the self-governing of the guild. He may own multiple workshops and employ a large number of workers. **You may select Academic Abilities at character generation.**

German, ArMDE:4924 (line-parallel):
> […] **Du kannst bei der Charaktererschaffung Akademische Fertigkeiten erwerben.**

**Current data.** `"classification": "narrative"`, no `effects`, no
`prerequisites`, no `parameters`.

**Why it is wrong, twice over.** `narrative` asserts the passage states nothing
mechanical. It states a permission, and that permission is load-bearing:
`academic` is in `categories_requiring_virtue`, so a guild master who buys
Artes Liberales gets a hard `ability_category_requires_virtue` error today. The
entry is `virtue.notary`'s shape exactly (B07's F-209), one book-section apart,
and `virtue.senior_clergy`'s two lines above.

**Correct value.** `"classification": "creation_effect"` plus
`{"type": "ability_authorization", "categories": ["academic"]}` — A14's
bare-permission case again, since no experience points are granted.

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-267 — `virtue.shadchan` — the Social-Status compatibility override is stated and encoded nowhere

**Passage** (ArMDE:4936, verbatim, the final sentence):
> The role of shadchan is open to both men and women alike and **this Virtue is compatible with any other Minor or Free Social Status Virtue.**

German, ArMDE:4936 (line-parallel):
> Die Rolle des Schadchens steht sowohl Männern als auch Frauen offen, **und diese Tugend ist mit jeder anderen Kleinen oder Freien Sozialen Status-Tugend vereinbar.**

**Why this is a rule and not colour.** ArMDE:2816 is the rule it overrides, and
it names the override form explicitly: "All characters must take one Social
Status, and **may only take more than one if the descriptions of the Virtues or
Flaws explicitly note that they are compatible**." Shadchan's sentence is one of
the descriptions ArMDE:2816 is talking about, and it is *scoped* — Minor or Free
only, so a second **Major** Social Status is still forbidden.

**Current data.** `creation_effect` with one `restricted_ability_xp` (see
F-268). Nothing encodes the compatibility, and neither `summary` mentions it
(both stop at "…for their sons and daughters." / "…für ihre Söhne und Töchter zu
finden.").

**What could and could not be encoded.** Nothing, today — and the reason is
worth stating because it changes the severity. `jq` over
`rules/core/character_types.json` shows **no** `virtue_category_caps` row for
`social_status` on any of the four profiles (the only virtue-side cap in the
file is magus's `hermetic` major cap), so `validation/caps.rs::validate_caps`
enforces no one-Social-Status rule at all. The base rule being unenforced makes
the override currently inert — but it does not make it unstated.

Not a finding: "open to both men and women alike" is the explicit *absence* of a
restriction, the mirror of B05's, B06's and B07's gender cases, and encodes to
nothing ✓.

**Correct value.** The compatibility clause written into `description` in both
locales. (Whether ArMDE:2816 itself should be enforced is outside this batch's
brief, which says to note instances only.)

**Severity.** Lost rule.

---

### F-268 — `virtue.shadchan` — the pool's scoping reaches neither locale (D5)

**Passage** (ArMDE:4938, verbatim — its own paragraph):
> The shadchan receives an extra 50 experience points to be spent on **social Abilities and an Area Lore appropriate for their community.**

German, ArMDE:4938 (line-parallel):
> Der Schadchen erhält 50 zusätzliche Erfahrungspunkte für **soziale Fertigkeiten und eine für seine Gemeinschaft geeignete Gebietskunde.**

**Current data.** `restricted_ability_xp` with `amount: 50` ✓ naming
`ability.area_lore`, `ability.bargain`, `ability.charm`, `ability.etiquette`,
`ability.folk_ken`, `ability.guile`, `ability.intrigue`, `ability.leadership` —
all eight `general` in `rules/core/abilities.json`, so no authorization question
arises ✓.

**Two things the data cannot say, and neither locale says either.**

1. **"social Abilities" is not a catalogue category.** `rules/core/abilities.json`
   knows `general`, `academic`, `arcane`, `martial` and `supernatural`; there is
   no `social` bucket. The seven non-Lore ids are therefore an *interpretation*
   of the phrase, not a transcription of a list the book gives — unlike
   `virtue.schooled_in_crime`, which names its nine. The interpretation looks
   sound, but nothing records that it is one.
2. **"an Area Lore appropriate for their community"** is singular and
   community-scoped. `ability.area_lore` carries `"parameter": "area"`, and A7
   records that `restricted_ability_xp_pools` hard-codes `instances:
   Vec::new()`, so the pool funds **every** Area Lore instance (D3).

**Correct value.** Keep the effect; write the two scoping clauses into
`description` in both locales, and record in `RULES.md` which seven Abilities
the "social Abilities" reading resolved to.

**Severity.** Lost rule, with a provenance gap underneath it — the seven ids are
an editorial reading with nothing pointing at it.

---

### F-269 — `virtue.shamash` — `narrative` on an entry that states a hard requirement

**Passage** (ArMDE:4944, verbatim — its own paragraph, the whole of the entry's
second half):
> As a shamash, the character **must have the Educated (Hebrew) Virtue**.

German, ArMDE:4944 (line-parallel):
> Als Schamasch **muss der Charakter die Tugend Gebildet (Hebräisch) besitzen**.

**Current data.** `"classification": "narrative"`, no `effects`, no
`prerequisites`.

**Why it is wrong.** `narrative` claims the passage states nothing mechanical.
"Must have Virtue X" is a prerequisite — the paradigm content of
`Prereq::Has` — and it is the one sentence in the entry that is not description
of a synagogue servant's duties. Neither `summary` carries it (both stop at
"…responsible for its well-being." / "…für ihr Wohlergehen verantwortlich.").

**Correct value.** `"classification": "uncomputed_rule"`, with the requirement
written into `description` in both locales. It is `uncomputed_rule` rather than
`creation_effect` because the prerequisite cannot be encoded at all — see F-270.

**Severity.** Lost rule / provenance.

---

### F-270 — `virtue.shamash` — the required Virtue does not exist in the catalogue

**What I searched, exactly.** `jq` over `rules/core/virtues_flaws.json` for any
id matching `educated`; and the README's own closed census, which records
"**four missing entries, the `Educated` variants**" as a settled fact and tells
batches not to re-walk the `####` headings for it. The English heading is
`#### Educated (Hebrew)` at **ArMDE:3723**; no catalogue id corresponds to it.

**Consequence.** `Prereq::Has` needs an `Id` that resolves —
`ruleset/integrity.rs::validate_prereq_refs` fails the **load** on one that does
not (B6) — so the requirement cannot be added as data until the target entry
exists. This is exactly the position B07 reported for `virtue.rabbi` (F-240),
and `virtue.sofer` in this same batch is the third instance (F-283).

**Correct value.** No data change is possible today. The requirement goes into
`description` in both locales (F-269's correction), and the entry joins
`virtue.rabbi` and `virtue.sofer` on the list of prerequisites that become
encodable the moment the four `Educated` variants are added.

**Severity.** Wrong rules output, blocked — the app permits a shamash with no
Hebrew education, and nothing can currently stop it.

---

### F-271 — `virtue.sharp_ears` — `narrative` on a plain "+3"

**Passage** (ArMDE:4952, verbatim — the entry's whole body):
> You hear better than most. **You get a +3 bonus to all rolls involving hearing.**

German, ArMDE:4952 (line-parallel):
> Du hörst besser als die meisten Menschen. **Du erhältst einen Bonus von +3 auf alle Würfe, die das Hören beinhalten.**

**Current data.** `"classification": "narrative"`, no effects. Both summaries
carry only the first sentence ("You hear better than most." / "Du hörst besser
als die meisten Menschen."), so the number reaches the user nowhere.

**Why it is wrong.** A signed numeric modifier is the paradigm case of a
mechanical clause, and `types.rs::Classification`'s own doc is explicit that an
entry whose text states a signed modifier is **not** narrative even when the
engine computes nothing.

**This is the third sibling of a family B05 already decided.** B05's F-126
(`virtue.keen_eyesight`, "+3 to all rolls involving sight", ArMDE:4191) and
F-127 (`virtue.keen_sense_of_smell`, "+3 to all rolls involving his sense of
smell", ArMDE:4193) are the same sentence with a different sense, and B05 ruled
both `uncomputed_rule` — explicitly ruling **out** `ability_roll_mod` on the
ground that the bonus is to a *sense*, not to a named Ability, and that A41
records the variant as surfaced-only anyway. Sharp Ears is that argument
verbatim, so this batch reaches the same verdict rather than opening it again.

**Correct value.** `"classification": "uncomputed_rule"`, with the +3 written
into `description` in both locales.

**Severity.** Lost rule / provenance.

---

### F-272 — `virtue.side_effect` — `narrative` on an entry that states a scaling rule and two worked bonuses

**Passage** (ArMDE:4956, verbatim — the entry's whole body):
> Your magic has some incidental feature that is generally useful, though occasionally annoying. **The intensity of the side effect increases with the level of the spell.** Examples include a commanding presence when casting that translates into a temporary **+1 Presence** bonus for a short time after casting, or a calm state of mind derived from casting which allows a bonus on **Concentration rolls** for a short time after casting.

German, ArMDE:4956 (line-parallel):
> […] **Die Intensität des Nebeneffekts steigt mit der Stufe des Zaubers.** Beispiele sind eine gebietende Präsenz beim Zaubern, die sich in einen vorübergehenden **Präsenzbonus von +1** für kurze Zeit nach dem Zaubern übersetzt, oder ein ruhiger Geisteszustand nach dem Zaubern, der für kurze Zeit nach dem Wirken einen Bonus auf **Konzentrationswürfe** erlaubt.

**Current data.** `"classification": "narrative"`, no effects. Both summaries
carry only the first sentence.

**Why it is wrong, and why the answer is `uncomputed_rule` rather than an
effect.** The second sentence states a rule — the effect's size scales with
spell level — and the third prints a number (+1 Presence) and names a roll
(Concentration). The rule is genuinely uncomputable: the feature itself is
player-invented, the scaling has no table, and the duration is "for a short
time". That is word-for-word the `uncomputed_rule` definition in
`types.rs::Classification` ("GM judgement, open-ended magnitudes"), and it is
the class B01 confirmed for `virtue.all_according_to_plan`'s analogous shape.

The worked bonuses are *examples* rather than the rule, which is why they alone
would not settle it — but a player choosing Side Effect needs the scaling
statement and at least one example to know what the Virtue does, and today gets
neither.

**Correct value.** `"classification": "uncomputed_rule"`, with the scaling rule
and both examples written into `description` in both locales.

**Severity.** Lost rule / provenance.

---

## The directed read — `virtue.simple_student`

One of the five `creation_effect` entries that carry **no `effects` at all**
(README, and C7's explanation of why `data_integrity.rs::every_vf_is_classified`
lets them through). The brief asks two things: what it should carry, and whether
`creation_effect` is even the right class. **The data answers both, and I did
not have to judge** — B06 resolved `virtue.nephilim` by comparing it with its
three siblings, and Simple Student has a sibling that is closer still.

### The passage, verbatim

**English, ArMDE:4960** (the entry's first paragraph):
> The character is a university student who is has not yet taken a degree. He is typically between 14 and 16 years old and somewhere along his university program. **He receives 30 experience points per finished year that he can apply to Latin or Artes Liberales.** If he has finished his second year of studies, he is in the liminal position of either applying for work or continuing his education. **Female characters can only take this Virtue if they are studying to be physicians at Salerno, although the Paid Rights Virtue would allow them to take this Virtue elsewhere. However, at other universities they would not be able to graduate.**

**English, ArMDE:4962** (the whole second paragraph):
> More than half of all university students are Simple Students.

**German, line-parallel, ArMDE:4960 and 4962:**
> Der Charakter ist ein Universitätsstudent, der noch keinen Abschluss erworben hat. Er ist typischerweise zwischen 14 und 16 Jahre alt und irgendwo in seinem Universitätsprogramm. **Er erhält 30 Erfahrungspunkte pro abgeschlossenem Jahr, die er für Latein oder Artes Liberales aufwenden kann.** […] **Weibliche Charaktere können diese Tugend nur wählen, wenn sie in Salerno Medizin studieren, obwohl die Tugend Erkaufte Rechte ihnen erlauben würde, diese Tugend auch anderswo zu wählen. An anderen Universitäten könnten sie jedoch keinen Abschluss machen.**

> Mehr als die Hälfte aller Universitätsstudenten sind Einfache Studenten.

### The sibling that decides it

**`virtue.baccalaureus`, ArMDE:3472**, is the same Virtue one degree later and
states the identical rule in the identical words:

> The character has completed a three-year program at a university to receive a baccalaureus artium […] and has **90 experience points that he may spend on Latin and Artes Liberales — 30 experience points per finished year of studies.**

And its **data** is:

```json
"effects": [
  { "type": "grants_reputation", "kind": "academic", "score": 1 },
  { "type": "restricted_ability_xp", "amount": 90,
    "abilities": ["ability.artes_liberales", "ability.dead_language"] }
]
```

Two further siblings agree on the shape: `virtue.educated` (ArMDE:3711-3713)
carries `restricted_ability_xp {amount: 50, abilities: [artes_liberales,
dead_language]}`, and `virtue.magister_in_artibus` (ArMDE:4385-4394) carries
`{amount: 240, abilities: [ability.teaching], categories: ["academic"]}`. Latin
is `ability.dead_language` with `"parameter": "language"` — there is no
`ability.latin` in the catalogue — so `[artes_liberales, dead_language]` is the
established encoding of "Latin and Artes Liberales", used by three entries.

Baccalaureus's 90 is its three-year program resolved to a flat number
(3 × 30 = 90). Simple Student has no stated total, because "somewhere along his
university program" leaves the number of finished years open — the passage only
fixes the ceiling implicitly, at the point ("finished his second year") where
the character stops being a Simple Student.

---

### F-273 — `virtue.simple_student` — the academic permission is missing (the XP grant's absence is a *recorded deferral*, and is not the finding)

**Answer to the brief's second question first: `creation_effect` is the right
class, and it was not lost by accident.** `crates/arm-rules/RULES.md:5225-5228`
lists this entry under "**Deferred (3), source-not-present or no clean creation
number**", verbatim:

> **Savantism, Simple Student, Corrupted Arts (3 XP)** — Savantism (ArMDE:6703)
> *halves* starting XP (multiplicative; no variant); **Simple Student
> (ArMDE:4958) is 30 xp *per finished year* (age/life-stage, M6)**; Corrupted
> Arts (ArMDE:5853) has no creation XP figure […]

So the effect-less state is a **recorded decision with a stated reason**, and
that reason is exactly the one Q-67 asks about — the per-year rate has no clean
creation number. This is C7's *other* case: not "a `creation_effect` that lost
its effect", but "a `creation_effect` whose mechanic was deliberately deferred".
**The missing XP grant is therefore NOT reported as a finding.** (Answer to the
brief's first question, for the record: when it is wired, the shape is
`{"type": "restricted_ability_xp", "amount": <N>, "abilities": ["ability.artes_liberales", "ability.dead_language"]}`,
which is what all three siblings use; `<N>` remains open, Q-67.)

**What IS the finding, and it is not covered by that deferral.**
`ability.artes_liberales` and `ability.dead_language` are both **`academic`**,
which is in `rules/core/abilities.json`'s `categories_requiring_virtue`. With no
effect at all, the entry grants no **permission** either — and permission is a
separate mechanic from funding, with its own `Effect` variant (A14) that needs
no XP figure at all. A Simple Student who buys Artes Liberales today gets
`ability_category_requires_virtue`, a hard **error** from
`validation/authorization.rs::validate_ability_authorization`.

Nothing in RULES.md defers *that*. The deferral note is explicitly about the
"3 XP" wiring and cites "no clean creation number" as its reason, which is an
argument about the pool's `amount` and has no bearing on a bare permission. A
character who "receives 30 experience points … that he can apply to Latin or
Artes Liberales" is plainly permitted to own Artes Liberales, whatever the
number turns out to be.

**Correct value.**
`{"type": "ability_authorization", "abilities": ["ability.artes_liberales", "ability.dead_language"]}`
— id-scoped, not category-scoped (the passage names two Abilities and does not
open the `academic` category). When the XP deferral is later closed, the
`restricted_ability_xp` pool's A7 side-effect will subsume this row and it can
be dropped.

**Two things the effect still cannot say (D3), which belong in `description`:**

- **"per finished year"** — the entity stores no count of finished university
  years, which is precisely RULES.md's stated reason for the deferral.
- **"Latin or Artes Liberales"** — Latin is one *instance* of
  `ability.dead_language`, and A7 records that `restricted_ability_xp_pools`
  hard-codes `instances: Vec::new()`, so a future pool would fund **every** Dead
  Language. The same over-scope is live on `virtue.educated` and
  `virtue.baccalaureus` today, so this is a family-wide note rather than a
  defect of this entry alone.

**Two things the effect still cannot say (D3), which belong in `description`:**

- **"per finished year"** — the entity stores no count of finished university
  years, so the grant can only be a flat number.
- **"Latin or Artes Liberales"** — Latin is one *instance* of
  `ability.dead_language`, and A7 records that `restricted_ability_xp_pools`
  hard-codes `instances: Vec::new()`, so the pool funds **every** Dead Language.
  A Simple Student could spend the grant on Classical Greek. The same
  over-scope is live on `virtue.educated` and `virtue.baccalaureus` today, so
  this is a family-wide note rather than a defect of this entry alone.

**Severity.** Wrong rules output — the app refuses a legal character. (The lost
XP is real too, but it is a recorded deferral and is not charged to this batch.)

---

### F-274 — `virtue.simple_student` — the female/Salerno restriction and the Paid Rights override reach neither locale (D5)

**Passage** (ArMDE:4960, verbatim, the final two sentences — quoted in full in
the directed read above): female characters may take the Virtue only if
studying to be physicians at Salerno; the Paid Rights Virtue lifts the
restriction elsewhere; but at other universities they still cannot graduate.

**Current data.** Both summaries stop at "…has not yet taken a degree." / "…der
noch keinen Abschluss erworben hat." Neither the restriction, nor the override,
nor the graduation exception reaches the user.

**Why it cannot be encoded (D3).** Three independent reasons, each already
recorded: `types.rs`'s concept field declares `pub gender: String` with "no
mechanical effect" (B05's F-123, B07's four gender findings); the override is
*conditional* on another Virtue and on a place ("elsewhere"), and no `Prereq`
variant reads a place; and "would not be able to graduate" is a consequence in
the fiction, not a number.

The Paid Rights pointer resolves ✓ — `virtue.paid_rights` exists at
ArMDE:4606-4615 — so this is the **positive** of the override shape whose
negative `virtue.senior_clergy` states eleven lines earlier (F-265). The two are
the same rule read from two sides and should be corrected together.

`virtue.baccalaureus` carries the identical clause at ArMDE:3474 with the same
gap, so the correction pass should expect a third instance there.

**Correct value.** All three clauses written into `description` in both locales.

**Severity.** Lost rule.

---

### F-275 — `virtue.skilled_smuggler` — `narrative` on an entry whose own summary prints the number

**Passage** (ArMDE:4970, verbatim — the entry's whole body):
> The character can hide one item on his person, and **attempts to find it suffer a –9 penalty on Awareness rolls.** The item cannot be larger than a dagger, and the character may swap which items are hidden with a few minutes of work. Wealthy characters often procure collapsible tools to use in conjunction with this Virtue.

German, ArMDE:4970 (line-parallel):
> Der Charakter kann einen Gegenstand an seiner Person verbergen, und **Versuche, ihn zu finden, erleiden einen Abzug von –9 auf Wahrnehmungswürfe.** Der Gegenstand darf nicht größer als ein Dolch sein, und der Charakter kann mit wenigen Minuten Arbeit wechseln, welche Gegenstände verborgen sind. […]

**Current data.** `"classification": "narrative"`, no effects — **and both
`summary` fields already carry the number:**

> EN: "The character can hide one item on his person, and attempts to find it suffer a **-9** penalty on Awareness rolls."
> DE: "Der Charakter kann einen Gegenstand an seiner Person verbergen, und Versuche, ihn zu finden, erleiden einen Abzug von **-9** auf Wahrnehmungswürfe."

**This is B05's internal-contradiction shape in both locales at once** — the
entry's own displayed text states the mechanic its `classification` asserts does
not exist. That is the whole finding in one line, and it is the cleanest
instance the audit has found, because it needs no reading of the source at all
to see.

*(The hyphen is the correct ASCII `-` in both summaries ✓, where the source
prints an en dash `–`. Recorded as a pass on the CLAUDE.md glyph rule.)*

**Correct value.** `"classification": "uncomputed_rule"`, with the -9, the
one-item limit, the dagger size limit and the swap rule in `description` in both
locales. The residual clauses — the size limit and the swap — are in neither
summary today, so they are lost even though the number is not.

**Severity.** Lost rule / provenance. The -9 itself survives in the summary, so
the *user* impact is smaller than usual; the defect is that the class is the
thing guarding the obligation, and here it is demonstrably false.

---

### F-276 — `virtue.skilled_smuggler` — the German name reproduces a spelling error in the DE source

**Current data.** `rules/i18n/de/virtues_flaws.json` →
`"name": "Erfahrener Schmugler"`.

**Where it comes from.** The DE rulebook heading at ArMDE:4968 reads
`#### Erfahrener Schmugler` — one `g`. The correct German spelling is
**Schmuggler** (double `g`), and the DE body text at 4970 avoids the word
entirely, so the heading is the only occurrence and nothing in the source
corrects it.

**Why it is a finding rather than a faithful transcription.** The German name is
a **user-facing label rendered every session**, not a quotation of the source.
No translation table covers Skilled Smuggler (searched all sixteen files under
`rules/source/de/translation-tables/` for `Skilled Smuggler`, `Schmugler` and
`Schmuggler` — no hits, with `grep -a` to defeat the injected `grep`'s
hard-coded `-I`), so no canonical value binds it and the DE rulebook is the only
source — which is exactly the case where an obvious orthographic error in the
source should be corrected rather than propagated. CLAUDE.md rates localization
defects **up**.

**Correct value.** `"name": "Erfahrener Schmuggler"`.

**Severity.** Localization — low impact, high visibility.

---

### F-277 — `virtue.skinchanger` — `narrative` on an entry stating a Soak bonus, a Size band and a transformation time

**Passage** (ArMDE:4974, verbatim — the entry's whole body, mechanical clauses
bolded):
> You have a magical cloak, animal skin or similar item made from an animal. While in physical contact with it, you may transform into the form of the animal represented by the item. **The transformation takes one full round**, and you retain both intelligence and sentience while in animal form. Clothing and possessions (save the animal item) do not transform, and you may be seen as a transformed human with InAn or InCo, or similar spells. **If the item is stolen, the new owner has an Arcane Connection to you, and you may not transform until the item is retrieved. If the item is destroyed, you can make a new one over the course of a season.** although the method varies depending on what the item is. **Skinchangers may transform into any non-magical animal between Size -10 (robin) and Size +2 (bear).** The character has the normal physical characteristics of the animal, **except that +3 is added to the character's Soak score (in animal form only).**

German, ArMDE:4974 (line-parallel):
> […] **Die Verwandlung dauert eine volle Runde** […] **Wenn der Gegenstand gestohlen wird, hat der neue Besitzer eine Arkane Verbindung zu Dir, und Du kannst Dich nicht verwandeln, bis der Gegenstand zurückgewonnen ist. Wenn der Gegenstand zerstört wird, kannst Du im Laufe eines Quartals einen neuen anfertigen** […] **Tierwandler können sich in jedes nicht-magische Tier zwischen Größe –10 (Rotkehlchen) und Größe +2 (Bär) verwandeln.** […] **außer dass seiner Absorption (nur in Tiergestalt) +3 hinzugefügt wird.**

**Current data.** `"classification": "narrative"`, no effects. Both summaries
carry only the first sentence ("You have a magical cloak, animal skin or similar
item made from an animal." / "Du besitzt einen magischen Umhang, eine Tierhaut
oder einen ähnlichen Gegenstand aus einem Tier.").

**Why it is wrong.** Five mechanical clauses, three of them numeric: a **+3
Soak**, a **Size band of -10 to +2**, a **one-round** transformation time, a
**one-season** item remake, and an Arcane Connection consequence with a hard
"you may not transform until the item is retrieved". `narrative` asserts the
book says nothing mechanical; it says five things.

**Why `uncomputed_rule` and not an effect.** `soak_mod` (A34) exists, but the +3
is explicitly conditional — "**in animal form only**" — and A34 offers no
condition axis, so authoring it would fold a form-scoped bonus into the
character's standing Soak for every roll. That is D4's shape and the mistake
this audit already found on `virtue.special_circumstances` in the same batch
(F-284). `size_delta` (A23) is likewise wrong: the Size band is the range of
animals available, not a change to the character's own Size.

**Correct value.** `"classification": "uncomputed_rule"`, with all five clauses
in `description` in both locales. **No effect.**

**Severity.** Lost rule / provenance, with a real risk of a *wrong* correction —
this is the entry where a correction pass is most likely to author the wrong
effect, so the "no effect" half of the verdict matters as much as the class.

---

### F-278 — `virtue.skinchanger_dove` — `narrative` on an entry stating a Soak bonus and a preparation time

**Passage** (ArMDE:4980 and :4984, verbatim, the mechanical clauses):
> She can, **over the course of a week**, prepare any artistic representation of a feather, or a single real feather, to act instead of a bundle of feathers. She may have several feathers prepared at one time, and **may use feathers prepared by any other character with this Virtue.**

> The character has **the statistics of a common dove** when in animal form, save that **her Soak is +3 higher than usual.**

German, ArMDE:4980 and :4984 (line-parallel):
> Sie kann **im Laufe einer Woche** jede künstlerische Darstellung einer Feder oder eine einzelne echte Feder vorbereiten, die stattdessen als Federbündel fungiert. Sie kann mehrere vorbereitete Federn gleichzeitig besitzen und **von jedem anderen Charakter mit dieser Tugend vorbereitete Federn verwenden.**

> Der Charakter besitzt in Tiergestalt **die Werte einer gewöhnlichen Taube**, außer dass **ihre Absorption um +3 höher ist als üblich.**

**Current data.** `"classification": "narrative"`, no effects. Both summaries
carry only "This is a more powerful variant of the Minor Virtue above." / "Dies
ist eine mächtigere Variante der Kleinen Tugend oben." — which, note, is a
summary that describes *nothing at all* about the Virtue without the entry it
points at.

**Why it is wrong.** A +3 Soak, a one-week preparation time, and a fixed animal
form (common dove) whose statistics the character assumes. Plus ArMDE:4982's
clause that garments and equipment **do** transform, which is the Major
Virtue's principal delta on the Minor's "Clothing and possessions … do not
transform".

**Correct value.** `"classification": "uncomputed_rule"`, with those clauses in
`description` in both locales. As with F-277, **no `soak_mod`**: the +3 is
animal-form-only.

**Severity.** Lost rule / provenance.

---

### F-279 — `virtue.skinchanger_dove` — "the Minor Virtue above" imports a whole rule set, and neither locale says so

**Passage** (ArMDE:4978, verbatim — the entry's entire first paragraph):
> This is a more powerful variant of the Minor Virtue above.

German, ArMDE:4978 (line-parallel):
> Dies ist eine mächtigere Variante der Kleinen Tugend oben.

**What the pointer lands on.** `#### Skinchanger`, **ArMDE:4972-4975**, id
`virtue.skinchanger` ✓ exists, the entry immediately above in both the book and
this batch.

**Why this is the batch's most productive cross-reference.** It is a
page-number-free named-Virtue pointer — invisible to any `(see page NNN)` screen
— and it is not a courtesy reference. It is the *only* statement of what the
Virtue does at all. Everything ArMDE:4980-4984 then says is a **delta**:

| Minor Virtue states (ArMDE:4974) | Major Virtue changes it to (ArMDE:4980-4984) |
|---|---|
| a cloak / skin / similar item, held in physical contact | a prepared feather, pin, brooch or swallowed engraved stone; several at once; usable across characters |
| transformation takes one full round | *unchanged* — stated only at 4974 |
| clothing and possessions do **not** transform | garments and equipment **do**, becoming feathers |
| any non-magical animal, Size -10 to +2 | **only** a common dove |
| +3 Soak in animal form | +3 Soak (restated) |
| item stolen → Arcane Connection, no transforming | *unchanged* — stated only at 4974 |
| item destroyed → one season to remake | replaced by the one-week feather preparation |

Four of those seven rows are inherited unchanged and exist **only** at
ArMDE:4974. A player who selects Skinchanger (Dove) and reads its entry learns
none of them — not the round-long transformation, not the Arcane Connection
consequence, not that intelligence is retained, not that InAn/InCo reveal him.
Neither entry carries a `description`, so the inheritance is invisible in both
locales.

**This is the shape B07 named as the audit's most productive pattern**
("the same as the X Virtue"), and it is the strongest instance yet: the pointer
is not *part* of the entry, it **is** the entry's first paragraph.

**Correct value.** A `description` in both locales that states the inherited
rules explicitly rather than pointing at them — the app has no cross-reference
mechanism, so a pointer that reads "see the Virtue above" resolves to nothing in
the UI. Alternatively, and better, both entries get full descriptions and this
one says which clauses it overrides.

**Severity.** Lost rule — and unusually complete, since without the pointer
resolved the entry's displayed text ("This is a more powerful variant of the
Minor Virtue above.") conveys **no** rule whatsoever.

---

### F-280 — `virtue.social_contacts` — `narrative` on an entry that states a roll and an Ease Factor

**Passage** (ArMDE:4990, verbatim, the operative sentences):
> Whenever you are somewhere new, **you can contact someone on a simple Presence roll against an Ease Factor of 6.** The storyguide may modify this target number upward for very small areas or areas where it is extremely unlikely that you would know someone. **You may purchase this Virtue more than once, each time specifying a different social group.**

German, ArMDE:4990 (line-parallel):
> Wann immer du dich an einem neuen Ort befindest, **kannst du mit einem einfachen Präsenz-Wurf gegen einen Schwierigkeitsgrad von 6 jemanden ausfindig machen.** Der Spielleiter kann diesen Zielwert nach oben anpassen […] **Du kannst diese Tugend mehrfach erwerben und dabei jedes Mal einen anderen gesellschaftlichen Kreis festlegen.**

**Current data.** `"classification": "narrative"`, no effects,
`"max_per_target": 255`, no `parameters`.

**Why it is wrong.** A named roll (simple Presence) against a named Ease Factor
(6) is a mechanic with two numbers in it, and it is the entry's whole point.
Both summaries stop at "…accumulated over years of travel and socializing." /
"…aufgebaut hast.", so the roll reaches the user nowhere. The storyguide
modifier is a second clause, and the correct reading of it — that it is
open-ended GM judgement — is exactly why the class is `uncomputed_rule` rather
than an effect.

**Why this entry matters more than most.** `virtue.social_contacts` is the
target of B07's `virtue.rosh_beth_din` `grants_selection` ✓, so a Rosh Beth Din
silently acquires it; whatever this entry fails to state, that character also
fails to learn.

**Correct value.** `"classification": "uncomputed_rule"`, with the Presence /
Ease Factor 6 roll and the storyguide modifier in `description` in both locales.

**Severity.** Lost rule / provenance.

---

### F-281 — `virtue.social_contacts` — no parameter records the social circle the passage requires

**Passage** (ArMDE:4990, verbatim, twice):
> You have a broad range of acquaintances in **a specific social circle (specified when this Virtue is purchased)** […] You may purchase this Virtue more than once, **each time specifying a different social group.**

German, ArMDE:4990: "in einem bestimmten gesellschaftlichen Umfeld (**das beim
Erwerb dieser Tugend festgelegt wird**)" … "und dabei **jedes Mal einen anderen
gesellschaftlichen Kreis** festlegen".

**Current data.** No `parameters` at all, and `"max_per_target": 255`.

**Two defects in one, and the second follows from the first.**

1. **The choice is not recorded.** The passage says the circle is "specified
   when this Virtue is purchased" — a mandatory player choice. B8 describes
   exactly the shape for it: a `text`-domain `ParameterDef`, whose value is then
   rendered by the picker and the Markdown export even though no effect consumes
   it, which engine-semantics explicitly calls "the normal and intended shape"
   for a parameter that exists to record a choice the engine does not compute
   from. `virtue.academic_concentration_subject` and
   `virtue.perfect_eye_for_commodity` already use it. Without it, a character
   sheet with two Social Contacts rows shows two identical entries and the
   player has nowhere to write "the Cathedral chapter" and "the Venetian wool
   trade".
2. **"a different social group" each time is therefore unenforceable.** B10
   records that `max_per_target`'s grouping key is
   `(item_ref, **whole** params map)`. With no parameters every copy has the
   same empty map, so `max_per_target: 255` is what makes the second copy legal
   at all — and it makes an *identical* second copy legal too. With a `text`
   parameter, the default `max_per_target: 1` would enforce "different each
   time" for free, exactly as it does for `virtue.student_of_realm`'s `realm`
   key four entries later.

**Qualifier — this is a recorded deferral, and the finding is downgraded
accordingly.** `crates/arm-rules/RULES.md:2509-2517` carries it by name:

> **"A different X each time" where X is free text.** Greater Immunity's
> immunity, **Social Contacts' social group** and Vulnerable Magic's condition
> are not recorded, so distinctness is not enforced. These items record no
> target at all today; adding a `Text` parameter would enforce it **but would
> invalidate existing saves whose selections carry no such parameter.**

So both halves above are known, and the reason for the deferral — save
compatibility — is a real cost this batch is not in a position to weigh. The
same note records that the three per-power caps *were* closed and "did carry
exactly the save cost described here, which is the price of closing the gap
rather than an argument against it", so the deferral is explicitly not final.

**What is therefore reported.** The gap stands as a defect in kind and is
recorded here for completeness with its RULES.md citation, **not** as a
newly-discovered one. What is genuinely unrecorded, and is this finding's live
half, is that neither locale carries the "different social group each time"
rule either — so a player is not even *told* the distinctness requirement the
engine cannot enforce.

**Correct value.** The distinctness rule in `description` in both locales today.
If the save-compatibility cost is accepted later: a `text`-domain `parameters`
entry plus dropping `"max_per_target": 255` back to the default of 1 — both
together, since dropping the 255 alone would forbid the second purchase the
passage allows.

**Severity.** Lost rule, downgraded from wrong-rules-output on the strength of
the RULES.md deferral.

---

### F-282 — `virtue.sofer` — `narrative` on an entry that states a hard requirement

**Passage** (ArMDE:4994 and :4996, verbatim — the second sentence of the first
paragraph and the whole of the second):
> The sofer **must be educated and able to read and write Hebrew** as the act of scribing the Sefer Torah requires knowledge and understanding of the text.

> As a sofer, the character **must have the Educated (Hebrew) Virtue.**

German, ArMDE:4994 and :4996 (line-parallel):
> Der Sofer **muss gebildet sein und Hebräisch lesen und schreiben können**, da das Schreiben der Sefer Torah Kenntnis und Verständnis des Textes voraussetzt.

> Als Sofer **muss der Charakter die Tugend Gebildet (Hebräisch) besitzen.**

**Current data.** `"classification": "narrative"`, no `effects`, no
`prerequisites`. Both summaries stop at the first sentence.

**Why it is wrong.** Identical to F-269's reasoning for `virtue.shamash` — "must
have Virtue X" is the paradigm content of `Prereq::Has`, and it is stated twice
here, once in prose ("must be educated and able to read and write Hebrew") and
once by name. This is the **third** entry in the book stating this same
requirement (`virtue.rabbi` ArMDE:4830, B07's F-240; `virtue.shamash`
ArMDE:4944; this one), and all three are currently unencoded.

**Correct value.** `"classification": "uncomputed_rule"`, with both sentences in
`description` in both locales.

**Severity.** Lost rule / provenance.

---

### F-283 — `virtue.sofer` — the required Virtue does not exist in the catalogue

Identical in substance to F-270. `#### Educated (Hebrew)` is **ArMDE:3723** and
has no catalogue id — one of the four missing `Educated` variants the README's
closed census records. `ruleset/integrity.rs::validate_prereq_refs` fails the
**load** on a `Prereq::Has` naming an unresolvable id (B6), so the requirement
cannot be added as data until the target entry exists.

The three affected entries — `virtue.rabbi`, `virtue.shamash`, `virtue.sofer` —
should be corrected in one change once the `Educated` variants are added, and
all three are Jewish-community Social Statuses from the same block of the book,
which is why the gap clusters.

**Severity.** Wrong rules output, blocked.

---

### F-284 — `virtue.special_circumstances` — a conditional casting bonus folded in flat (`scope: "all"`)

**Passage** (ArMDE:5000, verbatim — the entry's whole body, the condition
bolded):
> You are able to perform magic better **in certain uncommon situations (such as during a storm or while touching the target)**, gaining a +3 bonus to your Casting Scores and Magic Resistance. (A character only gains a bonus to Magic Resistance if she has Magic Resistance from another source.) You may take this Virtue more than once, but you only gain a +3 bonus even if more than one set of circumstances applies.

German, ArMDE:5000 (line-parallel):
> Du kannst **in bestimmten ungewöhnlichen Situationen** besser zaubern (etwa während eines Sturms oder während du das Ziel berührst) und erhältst einen Bonus von +3 auf deinen Zauberwert und deine Magieresistenz. […]

**Current data.**
`{"type": "casting_total_mod", "amount": 3, "scope": "all"}`.

**Why it is wrong.** `CastingScope::All` means every cast type — Formulaic,
Ritual and Spontaneous (`derived.rs::CastType::matches`) — and
`derived.rs::InPlayMods::casting_mod_for` folds it into **every** Casting Total
unconditionally. The passage conditions it on "certain uncommon situations",
which are by construction *not* every situation; the whole Virtue is that the
bonus applies rarely. A magus with Special Circumstances currently reads +3 on
his sheet for a calm afternoon in his laboratory.

**This is D4's shape on the casting side**, which D4's table does not cover, and
it is the **second** instance of it — B07 raised F-225 for Potent Magic's
casting half, whose condition ("in her field of magic") is likewise dropped by a
`scope: "all"` row. The engine already has the place to put a conditional
casting figure: `derived/casting.rs`'s `CastingTotal::within_focus` and
`PenetrationLine::within_focus` exist side by side with the unconditional total,
which is exactly the `LabTotal::total` / `within_focus` split D4 points at on
the lab side.

`virtue.ways_of_the_land` (ArMDE:5231-5234) and `virtue.cyclic_magic_positive`
(ArMDE:3635-3638) carry the same `scope: "all"` +3 / +3 shape and are in B09's
and B01's ranges respectively; the correction pass should decide the family
together.

**Qualifier — recorded in RULES.md, so this is corroboration.**
`crates/arm-rules/RULES.md:5131`'s `CastingTotalMod` row names this entry among
its carriers and states the behaviour flatly:

> computed; **conditional ones folded unconditionally (no toggle exists)**,
> surfaced per scope as the `casting_mod_formulaic`/`_ritual`/`_spontaneous`
> addends

So the flat fold is a known consequence of there being no toggle, not an
unnoticed bug. **Downgraded accordingly** — but note that B07 raised the same
shape as F-225 for Potent Magic and D4 has already ruled the *lab* half of that
family a defect rather than an acceptable approximation, so "no toggle exists"
is a statement of the current engine, not a ruling that it should stay that way.

**What remains live.** The condition reaches neither locale. Both summaries
carry "in certain uncommon situations (such as during a storm or while touching
the target)" ✓ — so the *condition* is actually stated, unusually for this
batch. What is not stated is that the app's displayed +3 ignores it. That is a
weaker D5 case than most here and is why F-284's severity drops furthest.

**Correct value.** Either a conditional home for the bonus (the `within_focus`
pattern, generalised), or a decision recorded in `decisions.md` that the
condition is deliberately ignored — which is what D1 ruled for the *spell-level
cap* and which D4 explicitly refused to extend to in-play totals. The family
decision (Potent Magic, Ways of the Land, Cyclic Magic, Special Circumstances)
should be taken once.

**Severity.** Wrong rules output in effect — a displayed Casting Total +3 too
high for every ordinary cast — but **recorded**, and the family already has an
open ruling (D4) covering its lab half. The live increment this batch adds is
small.

---

### F-285 — `virtue.special_circumstances` — two copies give +6 where the book caps the total at +3

**Passage** (ArMDE:5000, verbatim, the final sentence):
> **You may take this Virtue more than once, but you only gain a +3 bonus even if more than one set of circumstances applies.**

German, ArMDE:5000 (line-parallel):
> **Du kannst diese Tugend mehrfach wählen, erhältst jedoch nur einmal +3, selbst wenn mehrere der festgelegten Umstände gleichzeitig zutreffen.**

**Current data.** `"max_per_target": 255` (correctly permitting the repeat) plus
`{"type": "casting_total_mod", "amount": 3, "scope": "all"}`.

**Why it is wrong, mechanically.** Every effect fold in the engine iterates
`effective.rs::selections_for_effects` and **sums**;
`derived.rs::InPlayMods::casting_mod_for` is no exception. Two Special
Circumstances selections therefore contribute `3 + 3 = 6`. Nothing caps it:
`max_per_target: 255` is what *allows* the second row, and there is no
per-effect ceiling anywhere in the enum. Three copies give +9.

The book states the opposite in the same breath as the permission — the
permission and the cap are one sentence, and the data took the first half and
dropped the second.

**Note the interaction with F-284.** The two findings compound: the bonus that
should apply only in a named circumstance applies always, **and** it doubles.
With two copies a magus reads +6 on every Casting Total in the grid.

**Correct value.** Not expressible with the current enum — `casting_total_mod`
has no non-stacking flag and no `max` field. Either a new capping mechanism, or
the non-stacking rule written into `description` in both locales with the
stacking accepted as a known approximation and recorded in `RULES.md`. This is a
decision, not a data edit, and is raised as such.

**Severity.** Wrong rules output, and of the worst kind — a silently doubled
number on a total the app prints as a result. The app's entire purpose is
computing correct Ars Magica characters.

---

### F-286 — `virtue.special_circumstances` — `magic_resistance_mod`'s `aura_bonus` is a different Virtue's semantic

**Passage** (ArMDE:5000, verbatim, the Magic Resistance half):
> gaining a +3 bonus to your Casting Scores and **Magic Resistance. (A character only gains a bonus to Magic Resistance if she has Magic Resistance from another source.)**

German, ArMDE:5000: "…und deine Magieresistenz. **(Ein Charakter erhält nur dann
einen Bonus auf die Magieresistenz, wenn er bereits aus einer anderen Quelle
über Magieresistenz verfügt.)**"

**Current data.** `{"type": "magic_resistance_mod", "kind": "aura_bonus"}` — a
kind with **no amount field at all**.

**Why the variant is wrong, not merely inert.**
`types.rs::MagicResistanceEffect`'s own doc comment reads, verbatim:

> `/// A bonus to Magic Resistance while in a matching aura (Commanding Aura).`

The kind is defined *by* the Virtue it was written for, and that Virtue is
`virtue.commanding_aura` — which is the only other carrier in the catalogue
(`jq` over `rules/core/virtues_flaws.json` returns exactly two:
`virtue.commanding_aura` and this entry). Special Circumstances' passage
describes no aura; it describes "a storm" and "touching the target". Borrowing
`aura_bonus` here asserts a condition the book does not state and drops the one
it does.

**This is the defect shape B07's verification pass found on `virtue.relic`** —
an effect that is not about the thing the passage is about — rather than C1's
"the variant computes nothing", which is why it is reported here despite the
Part C exclusion. Two independent consequences follow from the variant choice,
and neither is C1's:

1. **The +3 is not representable.** `AuraBonus` is a unit-like kind carrying no
   amount, so the number the passage states cannot be written down at all — not
   even for the read-out, which surfaces it with `amount: 0`.
2. **The precondition is lost.** "only … if she has Magic Resistance from
   another source" is a real, checkable condition — `derived/casting.rs`'s
   `magic_resistance` already computes `max(might, parma_for_form)` per Form and
   knows perfectly well whether that is zero — and no field on this variant can
   hold it.

**Qualifier — half of this is recorded, and half is not.**
`crates/arm-rules/RULES.md:5138`'s `MagicResistanceMod` row **does** name this
entry as an `aura_bonus` carrier ("commanding_aura & **special_circumstances**
(aura_bonus)") and **does** explain why the four situational kinds are
surfaced-only: "each carries a scope the flat per-Form figure has no axis for (a
realm, **an aura**, a scene condition plus the incoming spell's level)". So the
*surfaced-only* treatment is a recorded decision and is not re-reported.

**What that row does not address, and what this finding is.** It lists Special
Circumstances under a kind whose parenthetical scope is "an aura" — and Special
Circumstances' condition is **not** an aura. The row's own justification
therefore does not fit the entry it applies it to: a storm and touching the
target are scene conditions, which is the *third* scope in that same list,
already represented by `conditional_penetration_waiver`. Nothing in RULES.md
explains why the aura kind was chosen for a non-aura Virtue, and
`types.rs::MagicResistanceEffect`'s doc comment attributes `AuraBonus` to
Commanding Aura alone. The mis-attribution survives both records.

**Correct value.** A rules-and-engine decision, not a data edit. Either a
`MagicResistanceEffect` kind that carries an amount and a "requires existing
resistance" condition, or a re-tag onto a kind whose documented scope matches,
or `uncomputed_rule`-style prose. What is **not** acceptable is leaving
`aura_bonus` unexplained, because it makes the entry look modelled while
asserting a condition the book does not state.

**Severity.** Wrong provenance — a rule attributed to a mechanism the book does
not use. The surfaced-only consequence is recorded; the mis-attribution is not.

---

### F-287 — `virtue.special_circumstances` — no parameter records which circumstances

**Passage** (ArMDE:5000): "in **certain uncommon situations** (such as during a
storm or while touching the target)" — the situations are the player's choice,
which is why the Virtue may be taken more than once "even if more than one
**set of circumstances** applies".

**Current data.** No `parameters`.

**Why it is a defect.** The same argument as F-281's, and the same remedy: a
`text`-domain `ParameterDef` is the established shape for recording a
player-chosen scope the engine does not compute from (B8), and
`virtue.academic_concentration_subject` already uses it with the neighbouring
`ability_roll_mod`. Without it, two Special Circumstances rows on a sheet are
indistinguishable, the choice does not survive a save/load round trip, and the
Markdown export prints the Virtue twice with nothing to tell them apart.

It also makes F-285's cap undiagnosable from the data: with a parameter, "two
different sets of circumstances" is visible; without one, the two rows look like
a duplicate-entry bug.

**Correct value.** Add a `text`-domain `parameters` entry for the circumstances.

**Severity.** Data loss — a player choice the rules require with nowhere to be
stored.

---

### F-288 — `virtue.spell_improvisation` — two clauses reach neither locale (D5)

**Passage** (ArMDE:5004, verbatim — the entry's whole body):
> The magus may add the magnitude of a Formulaic spell he knows as a bonus to his Casting Total when Spontaneously casting a spell that is similar to it (see Similar Spells, page 260). **This includes fast-casting a spell that is the same as or very similar to one of his Formulaic spells, though he does not get this bonus if he has the Fast Cast Ability for a mastered spell, since in that case you add his Mastery Ability instead. This bonus does not stack with other bonuses to his Casting Total, nor does it stack with itself if the magus happens to know several similar spells.**

German, ArMDE:5004 (line-parallel):
> […] **Dies gilt auch für das Schnellzaubern eines Zaubers, der einem seiner formulaischen Zauber gleich oder sehr ähnlich ist – allerdings erhält er diesen Bonus nicht, wenn er die Schnellzauber-Fertigkeit für einen gemeisterten Zauber besitzt, da in diesem Fall stattdessen sein Fertigkeitswert in der Zaubermeisterschaft addiert wird. Dieser Bonus lässt sich nicht mit anderen Boni auf die Zaubersumme stapeln und lässt sich auch nicht mit sich selbst stapeln, falls der Magus zufällig mehrere ähnliche Zauber kennt.**

**Current data.** `in_play_effect` ✓, one effect:
`{"type": "special_casting_mod", "kind": "spell_improvisation"}`.

**The effect is correctly authored and the class is correct.** The bonus is the
magnitude of *whichever* similar Formulaic spell the magus happens to know, on
*whichever* spontaneous spell he happens to cast — a pair of scene facts, so no
number on the sheet can carry it. `SpecialCasting::SpellImprovisation` being
surfaced-only is C1's gap and is not re-reported.

**What is left over.** Both summaries carry the first sentence and stop. Two
stated rules survive:

1. **The fast-cast interaction** — the bonus extends to fast-casting, *except*
   where the Fast Cast Ability for a mastered spell applies, in which case the
   Mastery score replaces it. That is a precedence rule between two named
   mechanics the app models (`spell_mastery_abilities.json` carries Fast Cast).
2. **The double non-stacking rule** — it stacks neither with other Casting Total
   bonuses **nor with itself**. Note that this is the same clause
   `virtue.special_circumstances` states three entries earlier (F-285), and here
   it is even stronger: it does not stack with *other* bonuses either.

**Correct value.** Keep the class and the effect; write both clauses into
`description` in both locales.

**Severity.** Lost rule.

---

### F-289 — `virtue.spiritual_pact` — `narrative` on the most mechanically dense entry in the batch

**Passage** (ArMDE:5014 and :5016, verbatim, the operative paragraphs — twelve
lines of pure mechanics):
> The character can channel the power of the spirit **by spending a Confidence Point. Make a Presence + Magic Lore + stress die roll: this is the amount of Magic Might Pool that the character acquires** from his spiritual master. **The Might points acquired are always less than the current Might points of the spirit, regardless of the roll's result. On a botch, the character loses all current Confidence points.** This action is equivalent to spellcasting with regard to the concentration it requires. The character can spend these Might points on any of the spirit's powers. **Penetration is calculated in the usual way for magical creatures using the initial Might Pool in place of Magic Might, and including the character's Penetration Ability, if any.**

> The character using this power **does not have a Might score, just a Might pool. He does not gain Magic Resistance from the use of this power, nor does he leave behind vis if he is slain. He cannot be affected by Vim spells (or similar magics) that target the Might score of supernatural creatures. Without a Might score, the pool does not replenish; once the character has spent all of his Might points this power ends.** All powers used have their duration lapse when the character uses his last point of Might Pool. **If this power is evoked again while the character still possesses Might Pool, then the new points gained replace the points left over; the two pools do not add.**

German, ArMDE:5014 and :5016 (line-parallel): "…indem er einen
Selbstvertrauenspunkt ausgibt. Würfle **Präsenz + Magiekunde + Stresswürfel**…
Bei einem Patzer verliert der Charakter alle aktuellen Selbstvertrauenspunkte."
… "Der Charakter …**hat keinen Machtwert, sondern nur einen
Machtwert-Vorrat. Er erlangt durch den Einsatz dieser Kraft keine
Magieresistenz** und hinterlässt auch keine Vis…"

**Current data.** `"classification": "narrative"`, no effects. Both summaries
are one line ("A pact with a powerful Magical spirit that grants access to its
power." / "Ein Pakt mit einem mächtigen magischen Geist, der Zugang zu dessen
Macht gewährt.").

**Why it is wrong.** At least eight mechanical clauses: a Confidence Point cost,
a named dice roll with a named formula, a cap on the result, a botch
consequence, a Penetration calculation, an explicit *denial* of Magic
Resistance, an explicit denial of vis and of Vim targeting, and a
pool-replacement rule. `narrative` claims the book states nothing mechanical
here; it states a subsystem.

**Why `uncomputed_rule` and no effect — including the one that looks obvious.**
A correction pass will be tempted by `might_grant` (A26). It must not: ArMDE:5016
says in as many words that the character "**does not have a Might score, just a
Might pool**" and "**does not gain Magic Resistance** from the use of this
power", while A26 records that `might_grant` does exactly the two things the
passage forbids — it produces a Might Score and `derived/casting.rs::magic_resistance`
reads it as a blanket resistance. A `might_grant` here would be the wrong
*subject* in B07's `virtue.relic` sense. The pact's Might Pool is a
scene-duration resource the engine has no field for, and the roll's result is a
die roll. Surfaced-as-text is the honest answer (D3).

**One thing that looks missing and is not.** "A character can only have a single
pact" (ArMDE:5020) is **already enforced**: the entry declares no `parameters`,
so B10's `max_per_target` grouping key `(item_ref, whole params map)` collapses
every copy into one group, and the field's default is **1** — a second selection
raises `duplicate_selection`. No `max_total: 1` is needed and none is missing.
Recorded because it is the shape a reviewer would flag.

**Correct value.** `"classification": "uncomputed_rule"`, with the eight clauses
in `description` in both locales, and **no effect**.

**Severity.** Lost rule / provenance, and the largest single loss of rules text
in the batch — twelve lines of mechanics reaching the user as one sentence of
flavour.

---

### F-290 — `virtue.strong_angelic_heritage` — six formulas and restrictions reach neither locale (D5)

**Passage** (ArMDE:5026 and :5028, verbatim — the two mechanical paragraphs):
> You are a divine being and possess **a Divine Might (Corpus) score equal to your age divided by 20**, which increases as you grow older. This grants you a Magic Resistance score. You contain **a number of pawns of Corpus vis equal to your Divine Might divided by 10 (rounded down, but always at least one pawn)** that can only be extracted if you are dead. **You are immune to Warping of any sort, and may not have any supernatural powers that derive from a realm other than the Divine.**

> You have up to thirty levels of holy powers that may be invoked by spending Might points. Design the powers' effects using the Hermetic spell guidelines (or the guidelines for Holy Powers in Realms of Power: The Divine Revised Edition, from page 46). **The Might cost for each power is equal to its magnitude divided by two (rounded down, but always at least one point). The Initiative score of the power is your Quickness.**

German, ArMDE:5026 and :5028 (line-parallel): "…einen **Göttlichen Machtwert
(Corpus) gleich deinem Alter geteilt durch 20**… **gleich deinem Göttlichen
Machtwert geteilt durch 10 (abgerundet, mindestens jedoch ein Bauer)**… **Du
bist gegen jede Art von Verzerrung immun und darfst keine übernatürlichen
Kräfte besitzen, die aus einer anderen Sphäre als dem Göttlichen stammen.**" …
"**Die Machtkosten für jede Kraft entsprechen ihrer Magnitude geteilt durch zwei
(abgerundet, mindestens jedoch ein Punkt). Der Initiativewert der Kraft
entspricht deiner Schnelligkeit.**"

**Current data.** `creation_effect`, `prerequisites: {kind: "has", value:
"virtue.blood_of_the_nephilim"}` ✓ (ArMDE:5024, "may only be taken if you have
the Major Virtue Blood of the Nephilim" — and `virtue.blood_of_the_nephilim`
exists at ArMDE:3504-3518 ✓), `"max_per_target": 255` ✓ (ArMDE:5030, "You may
take this Virtue multiple times"), and two effects:
`{"type": "might_grant", "realm": "divine", "score": 0}` and
`{"type": "power_levels", "amount": 30}`.

**Both effects are correctly authored**, and the `score: 0` is the deliberate
idiom A26 documents ("**0 establishes the Realm without adding points**") — the
score itself is age-dependent and cannot be a constant. `power_levels: 30`
matches "up to thirty levels" ✓ and stacks correctly on a second copy, which is
exactly ArMDE:5030's "increases by thirty … but has no other effect; in
particular, it does not increase your Divine Might" — the stacking
`power_levels` and the non-stacking (because zero) `might_grant` together model
that sentence precisely. That is a genuinely well-built entry.

**What is left over — six clauses, none computed, none in either locale** (both
summaries are one line):

1. **Divine Might = age ÷ 20**, and it rises with age. The `score: 0` is
   honest but silent; nothing tells the player what the number should be.
2. **Corpus vis = Divine Might ÷ 10**, rounded down, minimum 1, extractable only
   from a corpse.
3. **Immunity to Warping of any sort.** No effect variant expresses a warping
   immunity (the enum's warping variant, A22, only *grants* points), so this is
   structurally inexpressible.
4. **"may not have any supernatural powers that derive from a realm other than
   the Divine."** `types.rs::SupernaturalPower` stores `name`, `level` and
   `penetration` and **no realm** — the same structural gap B07 recorded as
   F-221 / F-251 for `virtue.personal_power` and `virtue.ritual_power` (D3).
5. **Might cost per power = magnitude ÷ 2**, rounded down, minimum 1.
6. **Initiative = Quickness.**

Clauses 5 and 6 are the same two B07 found unstated on `virtue.personal_power`
and `virtue.ritual_power`, so this is the **third** power-granting Virtue in two
batches whose power-usage rules reach the user nowhere. The correction pass
should write one shared block of text for all three rather than three
paraphrases.

**Correct value.** Keep the class, the prerequisite and both effects; write all
six clauses into `description` in both locales.

**Severity.** Lost rule — the player is given a 30-level power budget and told
nothing about what a power costs to use, when it acts, or what realm it must
belong to.

---

### F-291 — `virtue.strong_faerie_blood` — "You may not have both Faerie Blood and Strong Faerie Blood" is declared on neither side

**Passage** (ArMDE:5044, verbatim — its own one-sentence paragraph):
> You may not have both Faerie Blood and Strong Faerie Blood.

German, ArMDE:5044 (line-parallel):
> Du kannst nicht sowohl Feenblut als auch Starkes Feenblut besitzen.

**Current data, both entries, read directly.**

| id | ArMDE | `incompatible_with` |
|---|---|---|
| `virtue.strong_faerie_blood` | 5032-5047 | **absent** |
| `virtue.faerie_blood` | 3797-3819 | **absent** |

Neither entry declares the key at all. `validation/prereq.rs::validate_incompatibilities`
therefore raises nothing, and a character may hold both.

**Why nothing caught it.** B7 records that
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
**one-sided** declaration — it checks that a declared incompatibility is
mutual, not that a required one exists. Two absent declarations are perfectly
symmetric. And `validate_magnitude_variant_exclusivity`, the one place the
engine *compels* a pair to be mutually incompatible, keys on the
`<stem>_major`/`<stem>_minor` id shape; these two ids are `faerie_blood` and
`strong_faerie_blood`, which do not match that pattern, so nothing forces them
either. The rule is stated flatly in one sentence in the book and is invisible
to every guard.

Note the contrast with B07's F-226, which found the *opposite* failure:
`validate_magnitude_variant_exclusivity` **compelling** an exclusivity
ArMDE:4742 denies for Potent Magic. The same guard is both too strong and too
weak, in two entries eight book-pages apart.

**Correct value.** `"incompatible_with": ["virtue.faerie_blood"]` on this entry
and `"incompatible_with": ["virtue.strong_faerie_blood"]` on `virtue.faerie_blood`
— both, because the symmetry check will fail the load on one alone.

**Severity.** Wrong rules output — the app permits a character the passage
forbids in as many words, and the two Virtues' aging bonuses (-1 and -3) would
stack, since `aging_mod` sums across selections.

---

### F-292 — `virtue.strong_faerie_blood` — "you may learn Faerie Lore" with no `ability_authorization`

**Passage** (ArMDE:5040, verbatim — its own one-sentence paragraph):
> Third, **you may learn Faerie Lore during character generation.**

German, ArMDE:5040 (line-parallel):
> Drittens darfst du **Feenkunde während der Charaktererschaffung erlernen.**

**Current data.** Two effects, `grants_selection` and `aging_mod`; no
`ability_authorization`, and no `restricted_ability_xp` whose side-effect could
stand in for one.

**Why it is wrong.** `ability.faerie_lore` is **`arcane`** in
`rules/core/abilities.json`, and `arcane` is in that file's
`categories_requiring_virtue`. So a character with Strong Faerie Blood who buys
Faerie Lore gets `ability_category_requires_virtue` — a hard **error** — from
`validation/authorization.rs::validate_ability_authorization`. The passage
grants that exact permission, as one of the four numbered things the Virtue
does.

**The Minor version has the same gap and the same sentence.**
`virtue.faerie_blood` at ArMDE:3803 reads "Characters with Faerie Blood can
learn Faerie Lore at character generation", and its data carries only the
`aging_mod`. Both should be corrected together.

**Correct value.** Add
`{"type": "ability_authorization", "abilities": ["ability.faerie_lore"]}` to
both entries — A14's bare-permission case, id-scoped rather than
category-scoped, because the passage permits one Ability and not the whole
`arcane` category. (Category-scoping it would be B07's `privileged_upbringing`
over-permission mistake.)

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-293 — `virtue.strong_faerie_blood` — the aging onset age of fifty is carried nowhere (D3)

**Passage** (ArMDE:5036, verbatim — its own paragraph):
> First, you have natural longevity. **You start making aging rolls at the age of fifty, rather than the normal 35**, and get –3 to Aging Rolls, cumulative with any other bonuses.

German, ArMDE:5036 (line-parallel):
> Erstens hast du natürliche Langlebigkeit. **Du beginnst erst im Alter von fünfzig Jahren Alterungswürfe zu machen, anstatt der üblichen 35**, und erhältst –3 auf Alterungswürfe, kumulativ mit allen anderen Boni.

**Current data.** `{"type": "aging_mod", "kind": "aging_roll", "amount": -3}` —
the *second* half of the sentence ✓, correctly signed, and "cumulative with any
other bonuses" is exactly what `aging.rs`'s summing fold does ✓. The **first**
half is encoded nowhere.

**Why it cannot be encoded, stated precisely.** `AgingEffect` has eight kinds —
`aging_roll`, `living_conditions`, `no_aging`, `no_apparent_aging`,
`crisis_heavy_wound`, `crisis_survival`, `decrepitude`, `longevity_bonus`
(read from `types.rs`, and confirmed against C1's shipped-usage census). **None
moves the age at which aging rolls begin.** The onset age is a *ruleset-wide*
constant, `rules/core/aging.json` → `"start_age": 35`, consumed through
`aging.rs::AgingRules::first_roll_age`, and no effect variant overrides it per
character. `no_aging` is the nearest kind and is wrong: it suppresses aging
entirely rather than deferring its onset by fifteen years. So this is D3.

**Important qualifier — the engine-side half of this gap is already known and
documented, and this finding must not claim otherwise.** `aging.rs::aging_schedule`
carries a doc comment headed "**Known gap — a trait may move the start age, and
none does here**", which names Strong Faerie Blood, quotes ArMDE:5036 verbatim,
states that "the -3 is implemented … the start-at-fifty half is **not**", and
records what a fix would need ("a per-trait override of
`AgingRules::start_age`, which no other shipped item asks for"). So the
*computation* gap is a recorded decision, not an oversight, and is **not**
reported here as a defect — that would be re-reporting a known engine gap, which
the audit forbids.

**What remains a defect, and it is entirely D5's.** The rule reaches the user
**nowhere**. The engine's doc comment is in a Rust file no player opens; the
entry carries no `description` in either locale, and neither `summary` mentions
the onset age. A player who takes Strong Faerie Blood is shown an aging schedule
starting at 36 and is told nothing about why the book says 51. The documented
engine gap makes the number's absence *defensible*; it does not discharge the
obligation to state the rule.

**Correct value.** The onset age written into `description` in both locales. No
data change and no ninth `AgingEffect` kind is proposed.

**Severity.** Lost rule — downgraded from "wrong rules output" on the strength
of the `aging.rs::aging_schedule` comment, which shows the approximation was
made knowingly. The *consequence* for the player is still a computed aging
schedule fifteen years too long, with nothing in the app to warn him.

---

### F-294 — `virtue.strong_faerie_blood` — three further clauses reach neither locale (D5)

Both summaries are one line ("The blood of the fay is strong in you." / "Das
Blut der Feen ist stark in dir."). Beyond F-291 to F-293, three stated things
survive uncarried:

1. **"you can see normally in total darkness or semi-darkness"** (ArMDE:5038) /
   "du kannst bei völliger Dunkelheit oder Halbdunkel normal sehen". The same
   capability `virtue.see_in_darkness` is built entirely out of (Q-64), here as
   one of four numbered benefits.
2. **The seven faerie-blood heritages, imported wholesale.** ArMDE:5042:
   "Finally, you get the benefits of a particular type of fay heritage, **as
   given in the Virtue Faerie Blood (page 79)**." That pointer resolves to
   ArMDE:3805-3819, which lists seven blocks, each with its own mechanic —
   Bee King (Penetration 25), Dwarf Blood (**+1 to any total including a Craft
   Ability**), Goblin Blood (**+1 on all totals involving stealth**), Satyr
   Blood (**+1 to Communication and Presence totals** when dealing with
   sexually compatible characters), Sidhe Blood (**+1 to your Presence, but not
   to more than +3**), Spinnen Blood (body-weight cloth conversion per day),
   Undine Blood (**+2 to any action taken underwater**). Five of the seven print
   a number. **None of them reaches this entry in either locale, and there is no
   parameter recording which was chosen** — so the player cannot even write down
   his heritage, let alone learn what it does. `decisions.md`'s D5 rationale
   already names `virtue.faerie_blood` as the case that "computes none of its
   seven blood types"; this entry inherits the whole gap and adds a missing
   parameter on top.
3. **The apprenticeship clause.** ArMDE:5046: "This is a Supernatural Virtue,
   and you cannot lose it when being trained as a magus (see page 64). **If your
   master cannot preserve the ability, you cannot be trained.**" A hard
   consequence, and the entry says nothing about it.

Also uncarried but correctly so: "Choose one physical quirk, such as small
horns" (ArMDE:5042) is pure description ✓.

**Correct value.** All three written into `description` in both locales, plus a
`text`- or `enumerated`-domain parameter for the heritage — `enumerated` with
the seven names would be the stronger choice, since the list is closed in the
book, and it would let the picker offer them.

**Severity.** Lost rule, with a data-loss component (the heritage choice has
nowhere to live).

---

### F-295 — `virtue.strong_willed` — `narrative` on a plain "+3"

**Passage** (ArMDE:5050, verbatim — the entry's whole body):
> You cannot easily be coerced into activities, beliefs, or feelings. **You get +3 on any roll which may require strength of will.**

German, ArMDE:5050 (line-parallel):
> Du lässt dich nicht leicht zu Handlungen, Überzeugungen oder Gefühlen zwingen. **Du erhältst +3 auf jeden Wurf, der Willenskraft erfordern könnte.**

**Current data.** `"classification": "narrative"`, no effects. Both summaries
carry only the first sentence, so the +3 reaches the user nowhere.

**Why it is wrong.** The same reasoning as F-271's, and the same family: a
signed numeric modifier is the paradigm mechanical clause, and
`types.rs::Classification`'s doc says so explicitly.

**Why no effect.** `ability_roll_mod` (A41) requires a `text` param naming a
subject, and there is no player choice here — the subject is fixed, and it is
not an Ability at all: "any roll which may require strength of will" names no
catalogue Ability and no combat stat. B05 ruled out `ability_roll_mod` on
exactly this ground for `virtue.keen_eyesight`, and A41 records the variant as
surfaced-only in any case.

**Correct value.** `"classification": "uncomputed_rule"`, with the +3 in
`description` in both locales.

**Severity.** Lost rule / provenance.

---

### F-296 — `virtue.student_of_realm` — the +2 bonus is carried by no effect at all

**Passage** (ArMDE:5054, verbatim — the entry's whole body, the two operative
clauses bolded):
> You have been trained in the mystical aspects of one of the four realms of power (Divine, Faerie, Infernal, or Magic), and **you have a +2 bonus on all uses of the appropriate Lore.** **You may take that Lore at character generation even if you cannot learn other Arcane Abilities.** You may take this Virtue multiple times, for a different realm each time. **You may not take Student of (Realm) and Puissant Ability for the same Lore.**

German, ArMDE:5054 (line-parallel):
> Du wurdest in den mystischen Aspekten einer der vier Machtsphären (Göttlich, Feen, Infernal oder Magie) unterwiesen und **erhältst einen Bonus von +2 auf alle Verwendungen der entsprechenden Kundefertigkeiten.** **Du kannst diese Kundefertigkeiten bei der Charaktererschaffung erwerben, selbst wenn du normalerweise keine anderen Arkanen Fertigkeiten erlernen kannst.** Du kannst diese Tugend mehrfach wählen, jedes Mal für eine andere Sphäre. **Du kannst Student der (Sphäre) und Begabung in (Fertigkeit) nicht für dieselbe Kundefertigkeiten wählen.**

**Current data.** One parameter (`realm`, domain `realm`) and **one** effect:
`{"type": "ability_authorization", "abilities": [...]}`. There is **no
`ability_bonus`**, no `ability_roll_mod`, and nothing else. The +2 does not
exist anywhere in the catalogue for this entry.

**Why the engine could carry it, which is what makes this a miscalculation
rather than a D3 case.** `virtue.puissant_ability`, **two hundred lines earlier
in the same book** (ArMDE:4814-4816), encodes the identical idea:

```json
"parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
"effects":    [{ "type": "ability_bonus", "param": "ability", "amount": 2 }]
```

Same magnitude (Minor), same category (General), same number (+2). And the
passage's own last sentence — "You may not take Student of (Realm) **and
Puissant Ability** for the same Lore" — is the book telling us they are the same
kind of bonus, since that is the only reason two bonuses would need to be
mutually exclusive on one target.

**The one real obstacle, stated rather than glossed.** `Effect::AbilityBonus`
names a `param` whose domain must be **`ability`** (B8's table; enforced at load
by `ruleset/integrity.rs::validate_effect_refs`), and this entry's only
parameter has domain **`realm`**. So the fix is not a one-line effect addition:
either the entry gains a second, `ability`-domain parameter (with nothing to
restrict its value to the four Lores — `values` is `enumerated`-only, B8), or
the realm→Lore mapping is resolved somewhere.

**Qualifier — this is already flagged in RULES.md, and the finding is downgraded
to "recorded, still open".** `crates/arm-rules/RULES.md:5264-5274` says so in a
bolded heading of its own:

> **The `+2` bonus on the chosen Lore ("all uses of the appropriate Lore") is
> NOT implemented here.** It genuinely needs the parameter-relative form
> (`AbilityBonus`), which requires either a second `ability`-domain parameter
> paired to `realm` (with a validator enforcing the pairing, since nothing today
> stops selecting realm=Divine with ability=Faerie Lore) or a new realm→Lore
> resolution in the engine. […] flagged here as a follow-up, **not silently
> dropped**.

That is the identical diagnosis, reached independently here, including the same
two candidate fixes. So this batch **corroborates** an open item rather than
discovering one, and the "no test pins it" note at RULES.md:5275-5282 is
likewise already recorded.

**What is nonetheless live and unrecorded.** The +2 reaches the user **nowhere**
— there is no `description` in either locale, and neither `summary` states it.
RULES.md is a developer artefact; a player choosing this Minor Virtue is shown
no bonus and told of none. D5 obliges the text whatever the engine does.

**Correct value.** The +2 in `description` in both locales today; the effect
encoded whenever the RULES.md follow-up is taken.

**Severity.** Wrong rules output in effect (the character loses a +2 the book
grants) — but **recorded, not newly found**. Reported so the correction pass has
it in one list with the rest, and so the text obligation is not overlooked while
the engine half waits.

---

### F-297 — `virtue.student_of_realm` — the authorization permits all four Lores where the passage permits one

**Passage** (ArMDE:5054, verbatim): "**You may take *that* Lore** at character
generation even if you cannot learn other Arcane Abilities." — singular, and
anaphoric on "one of the four realms of power" chosen at the start of the
sentence. German, line-parallel: "Du kannst **diese** Kundefertigkeiten bei der
Charaktererschaffung erwerben".

**Current data**, read directly:

```json
{ "type": "ability_authorization",
  "abilities": ["ability.dominion_lore", "ability.faerie_lore",
                "ability.infernal_lore", "ability.magic_lore"] }
```

All four, unconditionally, regardless of which realm the `realm` parameter
names.

**Why it is wrong.** A14: `effective/xp.rs::ability_authorizations` unions every
carrier's lists into one `(abilities, categories)` pair with no reference to the
selection's params, so a character taking Student of (Magic) is thereby licensed
to buy **Infernal Lore** — an `arcane` Ability he could not otherwise have and
which the passage does not permit him. Taking the Virtue once unlocks the entire
`arcane` Realm-Lore set.

**And nothing errors on it — that is the point.** This is B07's
`virtue.privileged_upbringing` pattern (F-234) in its second instance, and it is
cleaner: there the over-permission was a *side-effect* of a restricted XP pool;
here it is authored directly, four ids where the passage grants one. A
permission that is too **narrow** produces a visible
`ability_category_requires_virtue` error, which is how six findings in this
batch were found. A permission that is too **wide** produces nothing at all —
no error, no warning, no test.

**Qualifier — and it corrects something this batch first wrote.** My first pass
said this was "only findable by reading the passage against the data". That is
**wrong**: `crates/arm-rules/RULES.md:5249-5263` documents it as a deliberate,
reasoned approximation, under the heading "**Documented approximation, same
shape as `flaw.covenant_upbringing`'s Latin proxy**":

> `Effect::AbilityAuthorization`, unlike `AbilityBonus`, is **not**
> parameter-relative — it is a static `Vec<Id>` — so authorizing all four Lore
> Abilities unconditionally is expressible without an engine change, **at the
> cost of over-authorizing: taking Student of (Divine) also nominally
> authorizes buying Faerie/Infernal/Magic Lore at creation.** This mirrors the
> already-accepted "any dead language, not Latin alone" approximation.

So this is a **known trade**, taken knowingly, with a named precedent — not an
unnoticed over-permission. B07's `privileged_upbringing` was genuinely
unnoticed; this one is not, and conflating them would overstate the pattern.

**What remains worth reporting.** Two things. First, the approximation's cost is
recorded in RULES.md and **in neither locale** — the player is never told that
his Student of (Magic) does not license Infernal Lore, so the app silently
offers him a build the book forbids with nothing to warn him. Second, and this
is a genuine difference from the "any dead language" precedent it claims to
mirror: that approximation is *over-funding within one gated Ability*, whereas
this one **crosses into three Abilities the character has no claim to at all**.
Whether the two are really the same shape is worth re-examining when the
RULES.md follow-up (F-296) is taken, since a fix for the +2 would give the
realm→Lore mapping this authorization also needs.

**Severity.** Wrong rules output in effect — the app permits three Abilities the
passage forbids — but **documented and deliberate**, so the live defect is the
missing warning text, not the data value.

---

### F-298 — `virtue.student_of_realm` — the Puissant Ability exclusion is encoded nowhere (D3)

**Passage** (ArMDE:5054, verbatim, the final sentence): "**You may not take
Student of (Realm) and Puissant Ability for the same Lore.**" / DE: "**Du kannst
Student der (Sphäre) und Begabung in (Fertigkeit) nicht für dieselbe
Kundefertigkeiten wählen.**"

**Current data.** No `incompatible_with` on this entry, and none on
`virtue.puissant_ability` (read directly — the entry carries only `parameters`
and `effects`). Neither summary mentions it.

**Why it cannot be encoded, stated precisely.** B7: `incompatible_with` is a
`BTreeSet<Id>` and `validation/prereq.rs::validate_incompatibilities` tests set
membership of **ids**. The exclusion here is not between two items — holding
both Virtues is perfectly legal, for *different* Lores, and the passage says so
by saying "for the same Lore". It is between two items **whose parameters
coincide**: Student of (Magic) and Puissant Magic Lore. No field expresses a
parameter-scoped incompatibility, and declaring `incompatible_with` would forbid
the legal combination as well as the illegal one — a *worse* error than the
current silence.

Note that this is the one place the two Virtues' shared "+2" reading is
load-bearing: the exclusion exists because the two bonuses are the same thing,
which is the evidence F-296 rests on.

**Qualifier — recorded in RULES.md, so this too is corroboration rather than
discovery.** `crates/arm-rules/RULES.md:2413-2415`:

> The same entry's "You may not take Student of (Realm) and Puissant Ability for
> the same Lore" (`ArMDE:5054`) is a cross-*item* constraint over a parameter
> value and is **likewise unmodelled**.

Identical diagnosis, independently reached. The neighbouring bullet also
confirms the *other* half of this entry's multiplicity handling is correct and
deliberate: "You may take this Virtue multiple times, for a different realm each
time" is "already expressed exactly by the defaults: `max_per_target` 1 over the
duplicate key `(item_ref, params)`, whose only parameter is the realm, plus no
`max_total`. Nothing to add." ✓ — which matches this batch's own reading.

**Correct value.** The exclusion written into `description` in both locales, on
this entry (and, ideally, on `virtue.puissant_ability` too, which states nothing
about it). **No `incompatible_with`** — RULES.md:2393-2398 warns in the same
section that inventing a restriction here "would forbid a build the book
permits".

**Severity.** Lost rule — the engine gap is recorded; what is not recorded is
that the rule reaches no locale, so a player is never told the two Virtues do
not combine.

---

### F-299 — **WITHDRAWN** — `virtue.student_of_realm`'s German *filled* name is a deliberate, documented decision

**This number is retired rather than renumbered**, so the batch's finding
sequence stays stable and the retraction stays visible. It is the same shape
`decisions.md` uses for D1's withdrawn first ruling. **Do not act on it.**

**What the first pass claimed.** That `"name": "Student einer Sphäre, {realm}"`
matches no canonical translation table and should become something of the form
"Student der …".

**Why that was wrong.** `crates/arm-rules/RULES.md:2355-2391` documents the
comma-apposition pattern as a **deliberate fix to a real bug**, applied
consistently to four sibling entries at once. Verbatim:

> Every German template put the token somewhere that demands agreement, so the
> moment the domain changed they rendered ungrammatical German: *"Gebunden an
> Das Göttliche"*, ***"Student der Das Göttliche"***, *"Das
> Göttliche-Stigmatisierter"*, *"Notwendige Das Göttliche-Aura für …"*. The fix
> is Folk Magic's own pattern from B7 — the label stands in **apposition after a
> comma**, uninflected, and no noun follows it

— with the four-row table that follows naming `virtue.student_of_realm` →
`Student einer Sphäre, {realm}` / `Student der (Sphäre)` explicitly, and a
recorded rejection of the parenthesis alternative ("the unfilled hint is itself
'(Sphäre)', so a `… ({realm})` template renders the nested '((Sphäre))'").

**So the first pass had it backwards twice.** "Student der {realm}" is the form
that was *removed* because it produced "Student der Das Göttliche"; and the
canonical table's value **is** honoured — in `name_unfilled`, "Student der
(Sphäre)", which is exactly `tugenden-fehler.md:216`. The table constrains the
*term*, and "Sphäre" is the term; it does not prescribe a filled-template
grammar the German language will not support.

**What I got right and should keep.** The observation that `name_unfilled`
exists only in `de` and not in `en` is correct and is also deliberate:
RULES.md:2387-2391 records that "the English realm labels are bare adjectives
(Magic, Faerie, Divine, Infernal), so … 'Student of Divine' … read as the book
heads them", so English needs no escape hatch. **No finding.**

**Severity.** None. Withdrawn.

*(The retracted text follows, for the record.)*

**Table rows**, both canonical:

> `rules/source/de/translation-tables/tugenden-fehler.md:216`
> `| Student of (Realm) | Student der (Sphäre) | |`

> `rules/source/de/translation-tables/grundbegriffe.md:399`
> `| Student of Magic Realm | Student der Magiesphäre | Mysterientugend; Verweis auf DM:ÜA |`

**Current data** (`rules/i18n/de/virtues_flaws.json`):

```json
"name": "Student einer Sphäre, {realm}",
"name_unfilled": "Student der (Sphäre)"
```

**The unfilled form is right and the filled form is not.** `name_unfilled`
reproduces `tugenden-fehler.md:216` exactly ✓ — and
`ui/src/lib/derive.ts::displayName` returns it only when no token is filled,
which is the picker's browsing state. The moment the player chooses a realm the
template takes over and the label becomes "Student einer Sphäre, Magie" — a
comma-spliced apposition that appears in no table, reads as a list rather than a
name, and does not match `grundbegriffe.md:399`'s "Student der Magiesphäre"
pattern.

**Why the data is like this, and why that is not an excuse.** German requires
inflection the template cannot supply: "Student der **Magie**sphäre", "der
**Feen**sphäre", "der **Göttlichen** Sphäre", "der **Infernalen** Sphäre" —
four different forms, and `displayName` does a plain `replace` of `{realm}` with
the resolved label. The comma form is a workaround for that. But the repo has
already faced this exact problem and solved it properly: `feedback_german_adjective_inflection`
is a recorded convention, and `rules/i18n/de/abilities.json` handles the same
shape with `"{language} (Tote Sprache)"` — putting the uninflected value in a
position where no inflection is needed.

**Correct value.** ~~A template that needs no inflection and matches the table's
pattern — e.g. `"Student der Sphäre: {realm}"` or `"Student der {realm}-Sphäre"`
if the four realm labels compound cleanly.~~ **Retracted — see the withdrawal
note above. `"Student der {realm}-Sphäre"` is precisely the shape RULES.md
records as having produced ungrammatical German, and the current value is the
documented fix.**

**Severity.** ~~Localization.~~ **None — withdrawn.**

---

### F-300 — `virtue.study_bonus` — the condition and the whole eight-row table reach neither locale (D5)

**Passage** (ArMDE:5058, verbatim, plus the block-quote table at 5060-5071):
> When given the opportunity to study an Art from books or raw vis **in the presence of the Form or Technique**, your surroundings give you new insights into your studies. Add two to die rolls to study from vis, or two to the Quality of any text you study from. **Your current Art score determines the magnitude of the surroundings you require to get the bonus. See the table for some guidelines.**

> | Art Score | Minimum Presence of Art |
> | 0 | A candle flame (Ignem), a magic aura (Vim), a dying insect (Perdo) |
> | 5 | A pond (Aquam), a painting (Imaginem), a caterpillar and butterfly (Muto) |
> | … | … |
> | 35 | Flying in the middle of a hurricane (Auram), while dissecting several magical animals (Animal), … |

German, ArMDE:5058 and 5060-5071 (line-parallel, the table translated row for
row): "…**in Anwesenheit der betreffenden Form oder Technik** zu studieren…
**Dein aktueller Kunstwert bestimmt die Magnitude der Umgebung, die du
benötigst, um den Bonus zu erhalten.**"

**Current data.** `in_play_effect` ✓, two effects:
`{"type": "advancement_mod", "source": "vis", "amount": 2}` and
`{"type": "advancement_mod", "source": "book", "amount": 2}`.

**The two effects are exactly right**, and worth confirming because one thing
looks wrong and is not: `types.rs::AdvancementSource::All`'s doc comment reads
"Every advancement source (**Study Bonus**; Unimaginative Learner)", which
would suggest `source: "all"`. The passage names two sources only — "books or
raw vis" — so the data's `vis` + `book` pair is the correct reading and the doc
comment's parenthetical is stale. (`jq` over the catalogue confirms **zero**
entries author `source: "all"`, so nothing else depends on it.) Recorded as a
source-comment defect, not rated as a finding against this entry.

**What is left over.** Both summaries carry only the first clause and drop
everything after "…new insights into your studies." / "…neue Einsichten in dein
Studium." Three things go with it:

1. **The condition** — the bonus applies only "in the presence of the Form or
   Technique", which is not a property of the character sheet at all. Two flat
   `advancement_mod` rows carry no condition (and, being surfaced-only per C1,
   carry no number into any total either — that half is not re-reported).
2. **The scaling requirement** — "Your current Art score determines the
   magnitude of the surroundings you require", which means the *same*
   surroundings stop working as the Art rises.
3. **The eight-row table itself**, 5060-5071, which is inside the entry's own
   `source.lines` range (5056-5072 ✓) and is the only thing that makes clauses 1
   and 2 usable at play. It exists in both languages in the source and in
   neither locale in the app.

**Correct value.** Keep the class and both effects; write the condition, the
scaling rule and the table into `description` in both locales.

**Severity.** Lost rule — the largest single table in the batch, fully
translated in the source and reaching the user nowhere.

---

### F-301 — `virtue.subtle_magic` — the gesture clause reaches neither locale (D5)

**Passage** (ArMDE:5075, verbatim — the entry's whole body):
> You may cast spells without using gestures at no penalty. **You gain no benefits from using normal gestures but gain the normal benefit for exaggerated gestures.**

German, ArMDE:5075 (line-parallel):
> Du kannst Zauber ohne Gesten wirken, ohne einen Malus zu erleiden. **Du profitierst nicht mehr von normalen Gesten, erhältst aber weiterhin den normalen Bonus für übertriebene Gesten.**

**Current data.** `in_play_effect` ✓, one effect:
`{"type": "special_casting_mod", "kind": "subtle_gestures"}`.

**The first sentence is genuinely computed** — one of only three live
`SpecialCasting` kinds. `derived.rs::in_play_mods` adds
`SUBTLE_MAGIC_GESTURE_REDUCTION` (`= 5`) to `gesture_reduction`, and
`residual_gesture_penalty` returns `(NO_GESTURE_PENALTY + gesture_reduction).min(0)`
= `(-5 + 5).min(0)` = **0** ✓. That is precisely "without using gestures at no
penalty", and the constant carries its own `ArMDE:5073-5076` citation. No
finding on the first sentence.

**What is left over.** The second sentence, in neither locale (both summaries
stop after the first). Against the Words/Gestures table at **ArMDE:9240-9245**
— Exaggerated +1, Bold 0, Subtle -2, None -5 — it says two things: normal
(Bold) gestures give the Subtle-Magic caster nothing, and exaggerated gestures
still give +1.

**Why this finding is qualified and carries Q-68.** The Bold row is already 0 in
the table, so "you gain no benefits from using normal gestures" may be a
restatement of the baseline rather than a rule of its own — in which case only
the exaggerated-gestures half is new information. The engine models neither row:
`residual_gesture_penalty` computes only the "None" case, and `InPlayMods`
carries no Bold/Exaggerated axis, so both halves are uncomputed either way. The
finding stands on the exaggerated clause; the normal-gestures clause is
escalated.

**Correct value.** Keep the class and the effect; write the second sentence into
`description` in both locales.

**Severity.** Lost rule, low impact.

---

### F-302 — `virtue.sufi` — `narrative` on an entry that grants three gated Abilities, with no `ability_authorization`

**Passage** (ArMDE:5081, verbatim — its own paragraph):
> Most Muslims treat you with respect for your pious lifestyle. **You may purchase the Abilities Theology: Islam, Islamic Law, and Dominion Lore at character creation.**

German, ArMDE:5081 (line-parallel):
> Die meisten Muslime begegnen dir mit Respekt für deinen frommen Lebensstil. **Du kannst die Fertigkeiten Theologie: Islam, Islamisches Recht und Dominiumkunde bei der Charaktererschaffung erwerben.**

**Current data.** `"classification": "narrative"`, no `effects`, one
`taken_as` parameter (`category` domain, values `["social_status",
"supernatural"]`) and `"max_total": 1`.

**The parameter half of the entry is exemplary and is not the finding.**
`engine-semantics.md` § B5 cites **ArMDE:5083** for the `categories_for`
mechanism by name ("Sufi as Social Status *or* Supernatural"), and ArMDE:5083
says exactly that: "either as a Minor Social Status Virtue or a Minor
Supernatural Virtue". The `category`-domain parameter, its two values, and the
`max_total: 1` that domain requires are all correct ✓, and ArMDE:5079's "It is
also possible to be an entirely mundane Sufi, in which case you should take this
Virtue as a Social Status Virtue" is the same rule stated a second time ✓.

**Why the classification is wrong.** `narrative` asserts the passage states
nothing mechanical. ArMDE:5081 grants three named Abilities, and all three are
in gated categories — confirmed against `rules/core/abilities.json`:

| Ability | id | category |
|---|---|---|
| Theology: Islam | `ability.theology_islam` | **academic** |
| Islamic Law | `ability.islamic_law` | **academic** |
| Dominion Lore | `ability.dominion_lore` | **arcane** |

`academic` and `arcane` are both in `categories_requiring_virtue`, so a Sufi who
buys any of the three gets `ability_category_requires_virtue` — a hard
**error** — today.

**Correct value.** `"classification": "creation_effect"` plus

```json
{ "type": "ability_authorization",
  "abilities": ["ability.dominion_lore", "ability.islamic_law",
                "ability.theology_islam"] }
```

**Id-scoped, not category-scoped** — the passage names three Abilities and does
not open the `academic` or `arcane` categories. Category-scoping it would be
F-297's over-permission mistake on the very next entry.

**Severity.** Wrong rules output — the app refuses a legal character.

---

### F-303 — `virtue.sufi` — the no-points Story Flaw rule reaches neither locale (D5)

**Passage** (ArMDE:5079, verbatim, the operative clause):
> your shaykh expects equal commitment from you to him or her and the other members: **you should choose an appropriate Minor Story Flaw, such as Mentor, which does not yield any points for buying Virtues.**

German, ArMDE:5079 (line-parallel):
> **Wähle einen passenden Kleinen Geschichte-Fehler, etwa Mentor, der keine Punkte für den Erwerb von Tugenden einbringt.**

**Why this is a mechanical clause and an unusual one.** It is a **budget** rule:
a Flaw that is taken but whose points do **not** fund Virtues.
`validation/balance.rs::compute_balance` sums `Magnitude::points()` over every
bought selection whose `kind` is negative, with no per-selection exemption — B3
records the whole arithmetic, and there is no field anywhere on `PointItem` or
`Selection` that marks one Flaw's points as non-funding. So a Sufi who takes
Mentor as the passage directs currently gets a free point of Virtues the book
denies him.

It is also the **inverse** of the `free`/`grants_selection` machinery: those
give an item at no cost; this takes an item at no *benefit*. The engine has the
first and not the second.

**Current data.** Nothing. Both summaries stop at "…asceticism and rituals." /
"…Vereinigung mit Gott anstrebt."

**Correct value.** The clause written into `description` in both locales (D3 —
the budget exemption is structurally inexpressible). Whether a non-funding flag
should exist is a design question raised, not decided.

Not a finding, recorded so it is not re-raised: ArMDE:5083's "This Virtue may be
taken by both male and female characters" is the explicit *absence* of a
restriction, the mirror of B05's, B06's and B07's gender cases ✓.

**Severity.** Wrong rules output — a one-point budget inflation the engine
cannot see — mitigated by the passage's hedged "should".

---

### F-304 — `virtue.supernatural_beauty` — `narrative` on an entry that states a once-per-story resource

**Passage** (ArMDE:5091, verbatim, the operative sentences):
> A player may use this Virtue, **once per story**, to ask a storyguide to insert a fortunate coincidence, of the storyguide's choice, into a scene. **The troupe may veto the use of the Virtue in any situation where supernatural aid seems profoundly unlikely.**

German, ArMDE:5091 (line-parallel):
> Ein Spieler kann diese Tugend **einmal pro Geschichte** einsetzen, um einen Spielleiter zu bitten, einen glücklichen Zufall nach Wahl des Spielleiters in eine Szene einzufügen. **Die Spieltruppe kann den Einsatz der Tugend in jeder Situation ablehnen, in der übernatürliche Hilfe ausgesprochen unwahrscheinlich erscheint.**

**Current data.** `"classification": "narrative"`, no effects. Both summaries
carry the sentence up to "…heroism to its defense." and drop the rule.

**B05 flagged this entry for B08 from outside, and I re-derived it rather than
inheriting it.** B05's F-160 established the family by a whole-book search:
`"once per story" | "once per session"` returns **exactly three** hits in the
English core rulebook — ArMDE:3386 `virtue.all_according_to_plan` (already
`uncomputed_rule`), ArMDE:4203 `virtue.knows_people` (B05's F-160,
`narrative` → `uncomputed_rule`), and ArMDE:5091, this entry. Reading ArMDE:5091
directly confirms the structural identity B05 predicted, **including the
qualified veto**: Knows People's and Supernatural Beauty's rules are the same
rule with a different flavour, and one of the three siblings is already
classified the way the other two should be. The two open cases must move
together.

**Why `uncomputed_rule` and not an effect.** A per-story budget of one
storyguide-adjudicated coincidence is the enum doc's own example of an
uncomputable rule — "GM judgement, open-ended magnitudes" — and there is no
per-story resource anywhere in the model.

**Correct value.** `"classification": "uncomputed_rule"`, with the once-per-story
use, the troupe veto and F-305's precondition in `description` in both locales.

**Severity.** Lost rule / provenance. The Virtue is **Major** — three points —
and a player reading its entry in the app learns nothing he can actually do.

---

### F-305 — `virtue.supernatural_beauty` — the positive-Presence precondition is encoded nowhere (D3)

**Passage** (ArMDE:5095, verbatim — its own one-sentence paragraph):
> A character lacking a positive Presence score may not have this Virtue.

German, ArMDE:5095 (line-parallel):
> Ein Charakter ohne einen positiven Präsenzwert kann diese Tugend nicht besitzen.

**Current data.** No `prerequisites`, and nothing in either summary.

**Why it cannot be encoded, stated precisely.** B6 lists the eight `Prereq`
variants — `all`, `any`, `none`, `has`, `house`, `ability_min`, `art_min`,
`is_magus`. There is **no `characteristic_min`**. `ability_min` and `art_min`
are the two numeric-threshold variants and neither reads a Characteristic. So a
character with Presence -3 may take Supernatural Beauty today and nothing
objects.

**Two details worth recording for the correction pass.**

- **The verb is "may not *have*", not "may not *take*".** B02 collected that
  distinction as the non-purchase form, and it matters here: the restriction
  binds a *granted* copy too, which C4a records that
  `validate_characteristic_delta_preconditions`-style bought-only validators
  would miss even if the variant existed. (D2 is pending on the neighbouring
  question and is not pre-empted.)
- **The threshold is "positive", i.e. ≥ +1**, not ≥ 0. A Presence of 0 is not
  positive.

B05 found this clause while confirming F-160's citation, "recorded, not rated";
this batch rates it, from the source.

**Correct value.** The precondition written into `description` in both locales,
as part of F-304's correction. Whether a `characteristic_min` `Prereq` variant
should exist is a design question raised, not decided — but note that it would
also give `virtue.rosh_beth_din`'s `(30 – Int)` age formula half of what it
needs (B07, F-252).

**Severity.** Wrong rules output — the app permits a character the passage
forbids — mitigated to lost-rule by the fact that no available `Prereq` variant
could have prevented it.

---

## Corrections to this batch's own first pass

After the first pass was written to disk, a self-check searched
`crates/arm-rules/RULES.md` and the Rust sources for each entry id and for each
mechanic the findings named — something the first pass had **not** done, because
it checked the data and the engine's behaviour but not the project's own
traceability map. **Nine of the 48 findings were already on record in whole or
in part**, and one of them turned out to be no finding at all. All corrections
are folded into the findings above; they are listed here so the batch's error
rate is visible rather than quietly edited away.

| Finding | What the first pass claimed | What the record actually says | Outcome |
|---|---|---|---|
| **F-299** | `virtue.student_of_realm`'s German filled name matches no canonical table and should read "Student der …" | `RULES.md:2355-2391` records that "Student der Das Göttliche" is the **ungrammatical output that was fixed**, that the comma apposition is the deliberate replacement applied to four entries at once, that parentheses were considered and rejected, and that the table's term survives in `name_unfilled` ✓ | **WITHDRAWN** — the claim was backwards |
| **F-297** | The four-Lore over-authorization is "only findable by reading the passage against the data" | `RULES.md:5249-5263` documents it as a deliberate approximation, names its cost in the same words, and cites `flaw.covenant_upbringing`'s Latin proxy as precedent | **Downgraded** — corroboration, not discovery; the live half is the missing warning text |
| **F-296** | The missing +2 is an undiscovered miscalculation | `RULES.md:5264-5274` flags it under its own bolded heading as "**NOT implemented here** … flagged here as a follow-up, not silently dropped", with the same two candidate fixes | **Downgraded** — corroboration; the live half is D5's text |
| **F-298** | The Puissant-Ability exclusion is unencoded | `RULES.md:2413-2415` records it as "a cross-*item* constraint over a parameter value and is likewise unmodelled" | **Downgraded** — corroboration; the live half is D5's text |
| **F-281** | `virtue.social_contacts` needs a `text` parameter and a `max_per_target` of 1 | `RULES.md:2509-2517` names the entry, states the gap, and gives the reason for deferring it (adding the parameter "would invalidate existing saves") | **Downgraded** — recorded deferral; the live half is that the distinctness rule reaches no locale |
| **F-273** | `virtue.simple_student` lost its `restricted_ability_xp` | `RULES.md:5225-5228` lists it under "**Deferred (3)**" with the reason "30 xp *per finished year* (age/life-stage, M6)" — which is Q-67's question, answered | **Split** — the XP half is a recorded deferral and is no longer a finding; the **academic authorization** half is untouched by it and stands |
| **F-293** | The aging-onset age is carried nowhere | `aging.rs::aging_schedule` carries a doc comment headed "**Known gap — a trait may move the start age, and none does here**", naming Strong Faerie Blood and ArMDE:5036 | **Downgraded** — engine half on record; the text half stands |
| **F-284** | `virtue.special_circumstances`'s conditional +3 folded flat is an unnoticed D4-shaped defect | `RULES.md:5131` states it: "computed; **conditional ones folded unconditionally (no toggle exists)**" | **Downgraded** — recorded; and both summaries do state the condition, so even the D5 half is weak |
| **F-286** | `aura_bonus` on a non-aura Virtue is an unnoticed mis-attribution | `RULES.md:5138` **names** this entry as an `aura_bonus` carrier and justifies the surfaced-only treatment — but justifies it with "an aura", which is not this Virtue's condition | **Half downgraded** — the surfaced-only half is recorded and withdrawn; the mis-attribution is not addressed anywhere and stands |

**The methodological lesson, which later batches should take rather than
repeat.** `crates/arm-rules/RULES.md` is **7,400+ lines** and is the repo's
declared provenance home for exactly this class of decision — CLAUDE.md says so
("JSON files carry no comments, so RULES.md is the provenance home for rule
values encoded as data"). A batch that checks the rulebook, the JSON and the
Rust *behaviour* but not RULES.md will reliably re-report settled trade-offs as
new defects, which inflates the finding count and, worse, invites a correction
pass to "fix" a documented decision. **Grep RULES.md for every entry id in the
span, and for every mechanic a finding names, before writing the finding.** The
second place to check is a doc comment at the consuming call site (F-293's case)
— an engine gap can be recorded there and never reach `engine-semantics.md`'s
Part C, so "not in Part C" does **not** mean "undiscovered".

**What survives the check and is genuinely new** — the other 39 findings,
including every one of the six `ability_authorization` gaps (none of which
RULES.md records; see the qualifier at the head of the Findings section, where
RULES.md's own "Other access-granting Virtues are a data addition" sentence
confirms the coverage is acknowledged to be partial),
`virtue.strong_faerie_blood`'s missing mutual `incompatible_with` (F-291) and
missing Faerie Lore permission (F-292), **`virtue.special_circumstances`'s
+3-that-becomes-+6 (F-285)** — checked specifically against RULES.md's
`max_per_target` section, which quotes the very sentence at `:1527` while
recording no ceiling for it — and all eleven `narrative` reclassifications. Each
of those ids was searched in RULES.md and returned either nothing or an
unrelated row.

**F-285 is the batch's strongest finding after this check**, precisely because
it is the one that survived it: RULES.md:1526-1527 lists Social Contacts and
Special Circumstances side by side in its repeat-rules table and quotes Special
Circumstances' cap verbatim — "more than once, **but you only gain a +3 bonus
even if more than one set of circumstances applies**" — and then nothing in the
file, the data or the engine acts on the second half of that quotation. The
clause was transcribed and dropped in the same table.

---

## Open questions

### Q-64 — Is a supernatural *capability* with no number, no roll and no waived penalty a mechanical clause?

**The entry.** `virtue.see_in_darkness`, ArMDE:4896-4899, `narrative`, no
effects, marked `?` in the verdict table and **not** marked checked.

**The passage** (ArMDE:4898, verbatim — the entry's whole body):
> You can see in complete darkness. Other than that, your eyesight is not more acute than ordinary people's, and you do not see farther than normal people would see in daylight.

German, line-parallel: "Du kannst in vollständiger Dunkelheit sehen. Ansonsten
ist Dein Sehvermögen nicht schärfer als das normaler Menschen…"

**Why `narrative` is defensible.** The second sentence is the "explicit
statement that *nothing* changes" shape B05, B06 and B07 all rated as correctly
encoded by encoding nothing. And the first states no number, names no roll and
waives no stated penalty — **which I checked rather than assumed**: `grep -n
"darkness"` over the whole English core rulebook returns **14 hits**, and none
of them is a general rule imposing a penalty for acting in darkness. (The hits
are: this entry; `virtue.strong_faerie_blood` 5038; `flaw.*` 6829's -2 in
*bright* light, the inverse case; four Shape-and-Material bonus rows;
`Eyes of the Cat` 13552 and two other spell descriptions; and two aura/scene
passages.) So there is no penalty for the Virtue to waive and nothing for it to
modify.

**Why `narrative` is nonetheless doubtful.** It is a **Supernatural** Virtue
costing a point, and its whole content is a capability the mundane world does
not have. If "you can see in complete darkness" states nothing mechanical, the
class is asserting that a Minor Supernatural Virtue does nothing — which reads
oddly next to `virtue.second_sight` four lines earlier, whose equally
capability-shaped text ("see through illusionary concealment") is `creation_effect`
because the book happens to attach an Ability to it. The difference between the
two is not what the *rule* says; it is whether the book bolted an Ability onto
it.

**The question, stated so it can be answered once for a family.** Does a bare
supernatural capability with no number, no named roll and no penalty to waive
count as a "mechanical clause" for `classification` purposes? The answer governs
at least three entries beyond this one: `virtue.strong_faerie_blood`'s identical
darkness clause (ArMDE:5038, where it sits among four other benefits and is
counted in F-294), and any comparable capability the remaining eleven batches
turn up. **I have not picked a reading.**

---

### Q-65 — Two canonical translation tables give `Sense Holiness and Unholiness` different German names

**The conflict**, both rows read directly:

> `rules/source/de/translation-tables/fertigkeiten.md:81`
> `| Sense Holiness and Unholiness\* | Übernatürlich | Gespür für Heiliges und Unheiliges | |`

> `rules/source/de/translation-tables/tugenden-fehler.md:142`
> `| Sense Holiness/Unholiness | Heiligkeit/Unheiligkeit spüren | |`

**What the data says.** `rules/i18n/de/virtues_flaws.json` →
`"Gespür für Heiliges und Unheiliges"`, and `rules/i18n/de/abilities.json` →
the same string for `ability.sense_holiness_and_unholiness`. Both match
`fertigkeiten.md:81` and both match the DE rulebook heading at ArMDE:4926 and
its body at ArMDE:4928.

**Why I did not rate it a finding.** Three independent canonical sources agree
with the data (the abilities table, the abilities i18n file, and the DE
rulebook) against one row in the Virtues/Flaws table. Overturning three on the
strength of one would be picking a reading, which the audit forbids.

**Why it is nonetheless a question and not a shrug.** CLAUDE.md says the tables
are *the* canonical EN→DE mapping and that
`rules/source/de/translation-tables/README.md` "records resolved naming
conflicts" — so an unresolved conflict *between* two tables is precisely the
thing that file exists to settle, and it is not settled there. Note also that
`tugenden-fehler.md:142`'s English side is spelled differently ("Sense
Holiness/Unholiness" with a slash), which may mean the row is about a
differently-named item rather than this one.

**The question.** Which row governs, and should `README.md`'s conflict list
record it? The Virtue and the Ability must end up with the same German name
either way, since ArMDE:4928 names the Ability by the Virtue's own words.

---

### Q-66 — Does `virtue.sense_holiness_and_unholiness`'s "may overwhelm you" state a rule D5 obliges?

**The entry.** ArMDE:4926-4929, `creation_effect` with
`{"type": "ability_score_grant", "ability": "ability.sense_holiness_and_unholiness", "amount": 1}`
✓ — which is exactly ArMDE:4928's "Choosing this Virtue confers the Ability
Sense Holiness and Unholiness 1", correctly encoded. Marked `?` and **not**
marked checked.

**The clause in question** (ArMDE:4928, verbatim, the middle sentence):
> In auras of particularly strong divine or infernal influence, **your sensitivity may overwhelm you.**

German, line-parallel: "In Auren von besonders starkem göttlichem oder
infernalen Einfluss **kann Deine Sensibilität Dich überwältigen.**"

**Why it might be a D5 clause.** It states an in-play *consequence*, scoped to a
named condition (a strong Divine or Infernal aura), that the engine computes
nothing for and that appears in neither locale (both summaries stop after
"Choosing this Virtue confers…"/"Verleiht die gleichnamige Fähigkeit."). D5 is
worded broadly — "any mechanical clause the engine does not compute".

**Why it might not be.** It carries no number, no roll, no duration and no
defined consequence — "may overwhelm you" is not a mechanic but a licence for
the storyguide. If that is not a mechanical clause, then the entry is fully
clean and D5 obliges nothing. The `virtue.side_effect` case in this same batch
(F-272) resolved a similar vagueness *toward* the rule, but only because it also
carried a stated scaling rule and a printed +1.

**The question.** Is a named-condition consequence with no defined effect a
clause D5 obliges into `description`? Answering it sets the floor for the
remaining eleven batches, which will meet the shape repeatedly. **I have not
picked a reading.**

---

### Q-67 — What amount should `virtue.simple_student`'s restricted XP pool carry?

**Settled by the data** (see the directed read): the effect's *shape* is
`{"type": "restricted_ability_xp", "amount": <N>, "abilities": ["ability.artes_liberales", "ability.dead_language"]}`,
the encoding three sibling entries already use, and the `creation_effect` class
is correct. **Not settled:** `<N>`.

**What the book gives.** ArMDE:4960: "He receives **30 experience points per
finished year**", with no total, because the character is "somewhere along his
university program". Two bounds are stated indirectly: he is "typically between
14 and 16 years old", and "If he has finished his **second** year of studies, he
is in the liminal position of either applying for work or continuing his
education" — after which, per ArMDE:3472, three finished years make him a
Baccalaureus instead.

**The candidates.**

| `<N>` | Argument for |
|---|---|
| **60** | Two finished years, the maximum a character can have and still be a Simple Student rather than a Baccalaureus. Mirrors Baccalaureus's 90 = 3 × 30 exactly. |
| **30** | One finished year, the modal case for a 14-16-year-old "somewhere along" the program. |
| **0**, with the grant as a per-year note in `description` | Refuses to invent a number the book does not print. |
| A player-chosen 0/30/60 | Honest, but there is no `Effect` variant with a variable amount and no field on the entity for finished years. |

**Why I will not choose.** Baccalaureus's 90 is derived from a **stated**
three-year program; Simple Student has no stated year count, so any `<N>` is an
invention, and inventing a number to satisfy a shape is precisely what produced
the aging floor `7f5605a` deleted. The value belongs in `RULES.md` with a
recorded justification, so it needs a decision rather than a guess.

---

### Q-68 — Does Subtle Magic's "no benefits from using normal gestures" state anything the Words/Gestures table does not already price at zero?

**The clause.** ArMDE:5075: "You gain no benefits from using normal gestures but
gain the normal benefit for exaggerated gestures." / DE: "Du profitierst nicht
mehr von normalen Gesten, erhältst aber weiterhin den normalen Bonus für
übertriebene Gesten."

**The table it sits against**, ArMDE:9240-9245, read directly:

| Gestures | Modifier |
|---|---|
| Exaggerated | +1 |
| Bold | 0 |
| Subtle | -2 |
| None | -5 |

**The difficulty.** If "normal gestures" means the **Bold** row, the clause
restates a modifier that is already 0 and adds nothing — in which case only the
exaggerated half is informative and F-301 shrinks. If "normal gestures" means
**Subtle** (the row named after the Virtue), the clause says something else
entirely: that a Subtle Magic caster does not get a *reduced* -2 but simply
loses the benefit. The German ("nicht **mehr**", "no longer") hints at the
second reading; the English does not.

**Why it matters beyond one sentence.** `virtue.quiet_magic` (ArMDE:4822-4827,
B07's range) is the Words-side twin, with the same `voice_reduction` constant
and, per B07's F-239, its own uncarried clauses. Whatever this resolves to
applies there too.

**The question.** Which gesture row does "normal gestures" name, and does the
clause add a rule? **I have not picked a reading**, and F-301 is written to
stand on the exaggerated-gestures half alone.

---

### Q-69 — A canonical table attributes a Reputation rule to `Social Contacts` that ArMDE:4990 does not state

**The row** (`rules/source/de/translation-tables/reputationen.md:84`, verbatim):
> `| Social Contacts | Soziale Kontakte | Lokal | – | Verdoppelt Lokale Reputationen; kann Reputationen verleihen |`

— "**Doubles Local Reputations; can confer Reputations**".

**What ArMDE:4988-4991 actually states.** The full passage was read in both
languages (quoted at F-280). It states a Presence roll against Ease Factor 6, a
storyguide modifier, and repeat purchase with a different social group. **It
says nothing about Reputations at all** — the word does not appear in either
language's text for this entry.

**Why this is a question rather than a finding.** The German name is correct
(F-280's table check ✓), so no data value is wrong. But the `Anmerkung` column
is asserting a mechanic, in a table CLAUDE.md declares canonical, that the
English source of truth does not carry. Three readings, and I cannot choose
between them from the repo:

1. The row describes a **different** Social Contacts — a supplement version, as
   `tugenden-fehler.md:257`'s "SdM:M" Spirit Votary row demonstrably does.
2. The row's `Anmerkung` is an error.
3. The rule exists somewhere in ArMDE outside this entry's range, in which case
   it is a cross-reference nothing in the entry points at.

**Why it matters.** `virtue.social_contacts` is `grants_selection`'d by
`virtue.rosh_beth_din` (B07), so whatever it does propagates. And if reading 3
is right, the entry is missing a `grants_reputation` effect, which would turn
F-280's reclassification into a data finding as well.

---

## Sub-agent reconciliation

An independent pass re-derived the verdict for the **nine entries this batch
marked clean**, from the passages, without reading this file's reasoning for
those nine beforehand. Findings continue at **F-306**, questions at **Q-70**.

**Result: 9 re-derived, 2 overturned** — `virtue.self_confident` and
`virtue.sense_holiness_and_unholiness`. Seven stand clean. Three new findings
(F-306, F-307, F-308) and two new questions (Q-70, Q-71).

### Per-entry verdicts

| Entry | Independent verdict | Agrees with batch? |
|---|---|---|
| `virtue.second_sight` | clean | **yes** |
| `virtue.see_in_darkness` | clean, Q-64 stands | **yes** |
| `virtue.self_confident` | **NOT clean — F-306** | **no** |
| `virtue.sense_holiness_and_unholiness` | **NOT clean — F-307** (in addition to Q-65, Q-66) | **no** |
| `virtue.sense_passions` | clean; new **Q-70** | yes, with a question added |
| `virtue.shapeshifter` | clean | **yes** |
| `virtue.skilled_parens` | clean | **yes** |
| `virtue.spirit_votary` | clean; new **Q-71** on its German name | yes, with a question added |
| `virtue.summon_animals` | clean | **yes** |

### What was corroborated independently

- **All nine check-1 ranges.** Re-derived from the file: six of the nine end on
  the trailing blank (4899, 4929, 4933, 4949, 5009, 5088), three end on the last
  body line (4890, 4902, 4966). No range bisects an entry and none reaches a
  neighbour's `####`. B04/B05's ruling that the two-convention split is INFO and
  not a defect is followed, as this batch did.
- **All nine descriptor lines** (ArMDE:4889, :4897, :4901, :4927, :4931, :4947,
  :4965, :5007, :5086) against `magnitude`, `kind` and `categories` — nine of
  nine agree, including the two Major Supernaturals (`sense_passions` :4931,
  `summon_animals` :5086) and `spirit_votary`'s `*Free, Mythic Companion*`.
- **`prereqs`, `incompatible_with`, `parameters` (checks 7, 8, 9).** No passage
  among the nine states a prerequisite or demands a parameter choice. The four
  `mythic_companion` Virtues carry one identical symmetric four-way
  `incompatible_with`, which is ArMDE:2637 verbatim ("These Virtues are
  incompatible with each other, and with The Gift").
- **`virtue.spirit_votary`'s "(page 63)" cross-reference, re-walked to the same
  conclusion.** ArMDE:2637's grog exclusion is carried by the grog profile
  omitting `mythic_companion` from `permitted_categories`; ArMDE:2638's and
  :5008's two-for-one rate is `virtue_points_per_flaw_point: 2` on the
  `mythic_companion` profile; ArMDE:2748-2764's required Virtues/Flaws are in
  `mythic_companion_types.json`. Reached blind, same four encodings.
- **Q-65's two-table conflict, found blind.** `fertigkeiten.md:81` vs
  `tugenden-fehler.md:142`, with the same observation that three canonical
  sources back the shipped `"Gespür für Heiliges und Unheiliges"` (including the
  DE rulebook at ArMDE:4926/:4928) against the one dissenting row.
- **The German names of all nine against the line-parallel DE rulebook**:
  Zweites Gesicht (:4888), Im Dunkeln sehen (:4896), Selbstbewusst (:4900),
  Gespür für Heiliges und Unheiliges (:4926), Gespür für Leidenschaft (:4930),
  Gestaltwandler (:4946), Erfahrener Parens (:4964), Tiere rufen (:5085) — eight
  of eight match. The ninth is Q-71.
- **No U+2212** anywhere in the nine entries' EN or DE strings; none of them
  renders a signed value at all.

### Anchors for the nine (check 12 / B11 aid)

None of the nine carries an `anchor`, as this batch recorded. The EN anchors,
taken from the book's own internal links rather than derived:
`second-sight` (ArMDE:3174), `see-in-darkness` (:3175), `self-confident`
(:3311), `sense-holiness-and-unholiness` (:3176), `sense-passions` (:3043),
`shapeshifter` (:3044), `skilled-parens` (:3126), `summon-animals` (:3048).

**One trap worth recording before a correction pass adds them.**
`virtue.spirit_votary`'s anchor is **`spirit-votary-1`**, not `spirit-votary`:
ArMDE:3334 links the Virtue as `#spirit-votary-1` and ArMDE:25485-25486 shows
why — bare `spirit-votary` is the *Mythic Companion chapter* heading
`### Spirit Votary` at ArMDE:2741. The five Supernatural entries have the mirror
of the same problem: their `-1` suffixes belong to the **Abilities** chapter
(ArMDE:7261-7265 links `#second-sight-1`, `#sense-holiness-and-unholiness-1`,
`#sense-passions-1`, `#shapeshifter-1`, `#summon-animals-1`), so the *unsuffixed*
form is the Virtue's for those five and the *suffixed* form is the Virtue's for
Spirit Votary. A uniform rule would get one of the two groups wrong.

---

### F-306 — `virtue.self_confident` gives a grog a Confidence Score the book denies grogs, and a number the book never prints

**Severity: MEDIUM.** Wrong rules output, reachable in the shipped app with no
warning, and displayed.

**The passage** (ArMDE:4900-4902, verbatim):

> #### Self-Confident
> *Minor, General*<br>
> You have firm confidence in your own abilities, and have a Confidence Score of
> two. You also start with five Confidence Points, rather than the usual three.
> (See page 52 for Confidence rules.)

German, line-parallel (ArMDE:4902): "…besitzt einen Selbstvertrauenswert von 2.
Außerdem beginnst Du mit fünf Selbstvertrauenspunkten statt der üblichen drei."

**What the data says.** `{"type": "confidence_bonus", "score": 1, "points": 2}`
— a **delta**, not the two absolutes the passage states.
`effective/gift_confidence.rs::confidence` adds it to the type profile's base:
`score += s; points += p`.

**Why the delta is right for three of the four character types, and wrong for
the fourth.** The "(See page 52)" cross-reference lands on ArMDE:1115, which
licenses the delta reading:

> Important characters have a Confidence Score and Confidence Points. This
> includes both central player characters (magi and companions)… These
> characters start with three Confidence Points. Most such characters start with
> a Confidence Score of one, **but this can be modified by Virtues and Flaws.**

So 1 + 1 = 2 and 3 + 2 = 5 for companion, magus and mythic companion, whose
profiles carry `confidence_score: 1, confidence_points: 3`. ArMDE:2115 prints a
sample character at "Confidence Score: 2 (5)", which corroborates it.

**The grog case.** The grog profile in `rules/core/character_types.json` omits
both fields, so `types.rs::EntityTypeProfile`'s `#[serde(default)]` makes them
**0/0** — deliberately, and the doc comment on `confidence_score` says so
("grogs have no Confidence (0/omitted)"). But the same profile lists `general`
in `permitted_categories`, `virtue.self_confident` is `["general"]`, `minor`,
with no `prerequisites` and no `entity_kinds` restriction beyond `character`,
and the grog budget is 3 virtue points. So a grog may take it, and the result is
**Confidence Score 1, Confidence Points 2** — 0 + 1 and 0 + 2.

**What the book says about that state.** Three passages, all outside the entry's
range and none reachable from it except through the "(page 52)" pointer:

> ArMDE:1161 — "**Grogs don't have Confidence**, so this line is omitted."
>
> ArMDE:2221 — "10. **Companions and Magi Only:** Confidence. Your character
> starts with a Confidence Score of 1 and 3 Confidence Points, unless he has a
> Virtue or Flaw modifying this."
>
> ArMDE:2522 — "**Grogs do not have Confidence Points.** Like Story Flaws,
> Confidence Points indicate a central character."

So 1 (2) is a state the book forbids *and* a pair of numbers the book never
states for this Virtue, which says two and five.

**It reaches the user.** `ruleset_io.rs::confidence_fields` computes it for every
profile; `ui/src/lib/components/CharacterDetails.svelte` gates the read-out on
`confScore > 0 || confPoints > 0`, which 1 (2) satisfies, so the Confidence line
**appears** on a grog sheet. `export/magic.rs::write_confidence` returns early
only when both are zero, so the Markdown export prints a Confidence section for
that grog too — the doc comment there ("a grog has neither, so the section
disappears for one") is true only until this Virtue is taken.

**Nothing objects.** Searched `crates/arm-rules/src` and `crates/arm-app/src`
for `confidence_score`/`confidence_points` and for callers of `confidence(`: the
only non-test hits are `types.rs` (the field declarations),
`export/magic.rs::write_confidence` and `ruleset_io.rs::confidence_fields`. No
validator reads either field, so there is no `wrong_entity_kind`-style issue and
no warning on any path.

**Why the batch missed it.** The batch considered `confidence_bonus` against
C3c ("produces a number nothing consumes") and correctly declined to count that
systemic gap against the entry — but C3c is about the number having no
*downstream consumer*, which is a different question from the number being
wrong. This is B03's `virtue.ferocity` shape exactly: an unmodelled restriction
assumed inert, colliding with what a type profile already does (or, here,
deliberately does *not*) grant.

**Two candidate fixes, not chosen here.** Either the entry needs a restriction
the engine can express (a prereq or an entity/type gate keeping it off grogs,
which the book does not state in so many words), or `confidence_bonus` needs to
be able to say "set to 2/5" rather than "+1/+2". Both are rules calls; the
finding is that the current pair is wrong for a reachable character type.

---

### F-307 — `virtue.sense_holiness_and_unholiness`'s realm is fixed to Divine by the book, and that fact is in none of the five places it could be

**Severity: MEDIUM.** A lost rule with mechanical consequence, invisible in both
locales, and it is the B07 `virtue.relic` shape — a "(page 170)" pointer this
batch's cross-reference table (row: "the Ability | **No.**") read as generic.

**The rule.** ArMDE:2960 establishes that the realm is normally a *choice* and
names the exception class:

> All Supernatural Virtues and Flaws are associated with one of the four realms,
> Magic, Faerie, Infernal, and Divine… Some Virtues are always associated with
> other realms, such as Faerie Blood and Strong Faerie Blood, which are always
> associated with Faerie. **A Virtue's description notes if it is limited in this
> way.**

The entry's own description (ArMDE:4926-4929) notes no such limitation. Its
"(page 170)" pointer does — twice:

> ArMDE:7171 — "Most of these Supernatural Abilities are granted by the Magic or
> Faerie realms, and use the relevant column on the realm interaction table (page
> 410)… **Sense Holiness and Unholiness is the main exception, as a Divine
> power.**"
>
> ArMDE:7721 — "**This Ability is granted by the Divine realm, not the Magic or
> Faerie realms, and thus uses the Divine column of the realm interaction chart
> (page 410).**"

**Why it is mechanical rather than colour.** ArMDE:2962 states the consequence:

> The realm of a Virtue or Flaw determines how it interacts with supernatural
> auras (see page 410), and provides important background color. In addition, a
> character with a Supernatural Virtue or Flaw is **immune to Warping caused by
> living in a high aura associated with the same realm** (see page 389).

So the realm decides which column of the interaction table applies and which
aura the character is warping-immune in. For this entry the book removes the
choice; the shipped app neither records the fix nor offers the choice.

**Where it is absent — stated precisely.** Five places were checked by reading
each file, not by inference:

1. `rules/core/virtues_flaws.json` → `virtue.sense_holiness_and_unholiness` has
   no `parameters` at all (`jq` over the entry: keys are `id`, `kind`,
   `magnitude`, `categories`, `classification`, `entity_kinds`, `effects`,
   `source`).
2. `rules/i18n/en/virtues_flaws.json` → `summary` only: "Feel the presence of
   good and evil. Confers the Sense Holiness and Unholiness Ability." No
   `description`, no realm.
3. `rules/i18n/de/virtues_flaws.json` → same shape, same absence.
4. `rules/i18n/en/abilities.json` → `ability.sense_holiness_and_unholiness`
   `description` is "Feel the presence of good and evil in an area, person, or
   object." No realm.
5. `rules/i18n/de/abilities.json` → "Spüre die Gegenwart von Gut und Böse in
   einem Gebiet, einer Person oder einem Gegenstand." No realm.

So the word *Divine/göttlich* attaches to this power nowhere in the shipped data,
in either language.

**Why this is distinct from the systemic ArMDE:2960-2962 gap and must not be
folded into it.** That gap is "the realm **choice** is modelled on 5 of 115
Supernatural entries" — a missing `realm`-domain parameter. This entry needs the
opposite: not a picker, but a **statement**, because the book has already made
the choice. A correction that adds a realm parameter to the 110 would get this
one *wrong*, by offering a choice the book forbids. D5 covers it directly — a
mechanical clause the engine does not compute, which must be written into
`description` in both locales whatever the classification.

**The sibling instances, so a corrector knows it is a class and not a one-off.**
ArMDE:2960 names Faerie Blood and Strong Faerie Blood as always-Faerie;
`virtue.faerie_blood` and `virtue.strong_faerie_blood` likewise carry no
`parameters` and no realm word in either locale's `summary`. Both are outside
B08's range (B04 and this batch respectively) and neither is re-rated here — but
they show the shape repeats, and `virtue.sense_passions` in this very batch is
the third instance (Q-70).

**Relationship to Q-66.** Q-66 asks whether ArMDE:4928's "may overwhelm you" is
a D5 clause. This finding is independent of that answer: the Divine fix is not
vague, carries a named consequence (which column of ArMDE:410 applies), and is
stated as a flat absolute twice.

---

### F-308 — Three defective rows in `tugenden-fehler.md`, one of which this batch's Q-69 reasoning relies on

**Severity: LOW.** No shipped value is wrong today. It is a regeneration hazard
in a file CLAUDE.md declares the canonical EN→DE mapping, and it is the class
commit `1f8558d` opened ("say plainly that the tables can be wrong").

**Rows 1 and 2 — two Virtues are listed in the Major *and* the Minor
Supernatural table, with contradictory magnitudes.**

| Row | Section | Says |
|---|---|---|
| `tugenden-fehler.md:101` | `### Übernatürliche Tugenden, Groß` (starts :77) | `\| Sense Passions \| Gespür für Leidenschaft \| \|` |
| `tugenden-fehler.md:143` | `### Übernatürliche Tugenden, Klein` (starts :110) | `\| Sense Passions \| Gespür für Leidenschaft \| \|` |
| `tugenden-fehler.md:105` | `### Übernatürliche Tugenden, Groß` | `\| Summon Animals \| Tiere rufen \| \|` |
| `tugenden-fehler.md:145` | `### Übernatürliche Tugenden, Klein` | `\| Summon Animals \| Tiere rufen \| \|` |

The book is unambiguous: ArMDE:4931 `*Major, Supernatural*` for Sense Passions
and ArMDE:5086 `*Major, Supernatural*` for Summon Animals. The catalogue has both
as `major` — **the data is right and the Minor rows are the error.** The German
names are identical in both copies, so nothing in `rules/i18n/de/` is affected;
what is affected is anyone using the table's *section* to decide a magnitude.

**Row 3 — `tugenden-fehler.md:257` describes `Spirit Votary` as a Minor Social
Status Virtue. It is neither, in either book.**

> `| Spirit Votary | Geistverehrer | SdM:M; Klein; Charakter pflegt rituelle Beziehung zu einem Geist |`

It sits in `### Sozialer Status Tugenden / Social Status Virtues (Auswahl)`
(`tugenden-fehler.md:236`) and its `Anmerkung` says **Klein**. But:

> ArMDE:5007 — `*Free, Mythic Companion*`
>
> RoP:M:5482 — `*Free, Supernatural*`

Free in both, and Social Status in neither.

**The knock-on for Q-69.** Q-69 argues that a `tugenden-fehler.md` row may
describe a *different, supplement-only* item, and offers this row as the worked
example: "as `tugenden-fehler.md:257`'s 'SdM:M' Spirit Votary row demonstrably
does". **That premise does not hold.** RoP:M:5484 and ArMDE:5008 are the same
Virtue in the same words —

> RoP:M:5484 — "This Virtue grants the Second Sight Virtue for free, and allows
> the character to have two points of Virtues for every point of Flaw"
>
> ArMDE:5008 — "This Virtue grants the Second Sight Virtue for free, and allows
> the character to have two points of Virtues for every point of Flaw."

— identical clause, differing only in which book's Mythic Companion rules they
point at. So the `SdM:M` tag marks *where the glossary phrasing was distilled
from*, not a second item, which is exactly what `translation-tables/README.md:86`
concluded for the `Inoffensive to (Beings)` row. Q-69's reading 1 keeps whatever
independent merit it has, but it loses this example and should not be settled on
it.

---

### Q-70 — Should `virtue.sense_passions` carry `tainted: true`?

**Not settled from the source; deliberately not decided here.**

**The entry.** ArMDE:4930-4933, `major`, `["supernatural"]`, **no `tainted`
flag**, `creation_effect` with `ability_score_grant ability.sense_passions 1` —
which is ArMDE:4932 verbatim and correctly encoded. The descriptor line
(ArMDE:4931) reads `*Major, Supernatural*` with **no** `Tainted` tag, and
`engine-semantics.md` § B9 defines the field as exactly that descriptor tag.

**What the "(page 170)" cross-reference lands on** (ArMDE:7737, verbatim):

> **Sense Passions is either a false power (see the False Power Flaw), or is
> associated with the Infernal.** This means that it always appears infernal to
> divine or infernal detection. The presence of Infernal taint allows the Sense
> Holiness and Unholiness aspect to work.

**What pulls toward `tainted: true`.** ArMDE:3000's box defines the tag as
"Tainted Virtues and Flaws are associated with the Infernal realm", and the
Ability this Virtue confers is stated to be Infernal-associated (or a false
power) with no third option. `validate_tainted_cap` is the only consumer that
would move, and it is a warning.

**What pulls against.** The tag is a *descriptor* label and this descriptor does
not carry it, in either language (DE:4931 `*Groß, Übernatürlich*`). ArMDE:3000
states the implication in one direction only — Tainted ⇒ Infernal — and the
converse is nowhere stated. And the "false power" alternative means the
association is not unconditional.

**Note this is not the same gap as F-307.** Unlike Sense Holiness, the realm
fact for Sense Passions **does** reach the user: both locales' `abilities.json`
descriptions carry it ("associated with the Infernal or a false power" /
"mit dem Infernalen oder einer falschen Macht verbunden"). Only the `tainted`
flag is open.

---

### Q-71 — The German core rulebook gives `virtue.spirit_votary` two different names, and the tables give a third

**The four spellings, each read in place:**

| Source | German name |
|---|---|
| `Ars Magica Definitive Edition Basisregeln.md:5006` (the entry's own cited passage), plus its list at :3334 and its index at :25486 | **Geistesdiener** |
| `Ars Magica Definitive Edition Basisregeln.md:2741`, :2743, :2746, :2748-2749 (the Mythic Companion chapter, same Virtue) | **Geist-Anhänger** |
| `Ars Magica 5e - Sphären der Macht - Magie.md:5476`, :5478, :5480 | **Geistverehrer** |
| `tugenden-fehler.md:257`, `konvent.md:57`, `sphären-mächte.md:206` | **Geistverehrer** |

**What ships.** `rules/i18n/de/virtues_flaws.json` and
`rules/i18n/de/mythic_companion_types.json` both say **Geistverehrer** — the
tables' value, consistently.

**Why the data is defensible as it stands.** `translation-tables/README.md:83-88`
records the `Inoffensive to (Beings)` precedent on exactly this shape: rulebook
and table disagree, **the table is canonical**, and `rules/source/de/` is *not*
adjusted because it reproduces the books as printed. Applying that precedent
mechanically yields Geistverehrer. So this is **not** rated a finding.

**Why it is nonetheless a question.** Three things make it a worse fit for that
precedent than the Inoffensive case was:

1. The German **core** book — the entry's own cited source — is *internally*
   inconsistent, calling the same Virtue Geistesdiener at :5006 and
   Geist-Anhänger at :2741. No decision about the tables fixes that, and a reader
   comparing the app to their printed core book will find neither word.
2. The README's resolved-conflict list does not record this case at all, though
   CLAUDE.md says that file is where resolved naming conflicts live.
3. The deciding row, `tugenden-fehler.md:257`, is demonstrably unreliable on its
   other two columns — it calls the Virtue Minor and files it under Social
   Status, and it is neither (F-308).

**The question.** Which German name should `virtue.spirit_votary` and
`mythic_type.spirit_votary` carry, and should `README.md` record the resolution?
(The two must agree whichever way it goes.)

---
