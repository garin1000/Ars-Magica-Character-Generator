# Batch B04 — indices 105-139, ArMDE:3967-4154

Entries: 35. Audited: 35. Failures: **29**. Clean: **6**.
Findings: **32** (F-90 … F-121). Open questions: **12 new** (Q-23 … Q-34), plus
two of B03's recurring here (Q-14, Q-22 — the "`description` on a
`creation_effect`" practice, which three of this batch's findings depend on).

An independent verification pass re-derived the nine verdicts this batch had
cleared, was told that overturning one would be a success and inventing one
would not, and was not allowed to read this file. **Result: 7 confirmed, 2
overturned** — `virtue.hermetic_prestige` (**F-120**, which this pass had
already found independently by a separate route) and `virtue.greater_benediction`
(**F-121**, a genuine correction: rated `?` here, and it is a defect). It also
contributed two open questions (**Q-33**, **Q-34**) and corrected this file's own
D2 section. Both overturns were re-verified against the sources before being
accepted; the counts above are post-reconciliation. See "Sub-agent
reconciliation" at the end.

**Twenty of the thirty-two findings are a `narrative` classification on an entry
whose passage states a rule.** That is a far higher rate than any previous
batch, and it has one cause, established before the first verdict:
`uncomputed_clauses.rs::SWEPT_BLOCKS` covers ArMDE:3360-3950, and **this batch's
entire span (3967-4154) lies outside it** — the guard that keeps `narrative`
honest has never read a single line of it. `virtue.guardian_angel` is the proof:
its passage says "+5 bonus to Soak" in plain digits, which even the narrow
pre-`ecb5150` screen would have caught on sight.

Finding and question numbers **continue B03's sequence** (B01 F-01…F-33 /
Q-01…Q-08, B02 F-34…F-65 / Q-09…Q-16, B03 F-66…F-89 / Q-17…Q-22), so this batch
starts at **F-90** and **Q-23** and `corrections.md` can accumulate them without
collision.

*(This file is written incrementally — the verdict table lands first, findings
are appended as each entry is finished, and the sub-agent reconciliation last.)*

## Method

The whole span was read as continuous prose in **both** languages before any
entry was judged — `rules/source/en/Ars Magica - Definitive Edition (Core
Rules).md` ArMDE:3960-4159 and the line-parallel German file
`rules/source/de/Ars Magica Definitive Edition Basisregeln.md`. **Line parity
holds throughout the span:** every `####` heading sits on the same line number
in both files (spot-checked at :3967, :3987, :4009, :4059, :4103, :4133, :4151).

**The `####` walk (bracket check, not a census).** Between ArMDE:3967 and :4154
there are exactly 35 `####` entry headings and exactly 35 catalogue entries, and
every entry's `source.lines[0]` is one of those heading lines. Two `####`
headings inside the span are **not** entry headings and are correctly not
claimed by anything: `#### Greater Benediction Examples` (ArMDE:3995, inside a
block quote) and the `#####` sub-heads under it.

**`source.lines` (check 1) — all 35 correct.** Each range runs from its own
heading to the line before the next heading. Three entries stop one line short
of the next heading because a blank line separates them (`great_characteristic`
3987-3989, `heartbeast` 4059-4061, `hermetic_prestige` 4071-4073,
`improved_characteristics` 4103-4105); none bisects an entry or bleeds into a
neighbour. Exactly **one** entry carries an `anchor` — `great-characteristic` on
`virtue.great_characteristic` — and it matches both its own heading
(`#### Great (Characteristic)`) and the index link at ArMDE:3271.

**Magnitude and categories (checks 4 and 6) — verified twice.** Against each
entry's own descriptor line, and independently against the book's index
groupings, whose headings were re-read for this batch: `Supernatural, Major`
ArMDE:3023 (Greater Benediction, Greater Immunity, Greater Power, Greater
Purifying Touch, Hex, Immune to Disease, Induction), `Social Status, Major`
:3052 (Guild Dean), `General, Major` :3073 (Giant Blood, Guardian Angel),
`Hermetic, Minor` :3087 (Gorgiastic, Guest of House Criamon, Harnessed Magic,
Heartbeast, Hermetic Prestige, Imbued with the Spirit of (Form), **Inoffensive
to (Beings)**, Inventive Genius), `Supernatural, Minor` :3135 (Homing Instinct,
Immunity to Cold, Infernal Heirloom), `Social Status, Minor` :3187 (Guild
Apprentice, Guild Master, Ineslemen), `General, Minor` :3239 (Good Teacher,
Gossip, Great (Characteristic), Hermetic Experience, Improved Characteristics,
Independent Study, Indescribable Face, **Inoffensive to (Beings)**,
Inspirational, Intuition), `Social Status, Free` :3336 (Hermetic Magus). `The
Gift` sits alone at ArMDE:3006, between `## List of Virtues` (:3004) and the
first group heading — which is what `categories: ["special"]`, `magnitude:
"free"` encodes, and it matches the descriptor `*Free, Special*`.

All 35 agree with the data **except `virtue.inoffensive_to_beings`**, which the
index lists **twice** (:3110 Hermetic Minor and :3276 General Minor) and whose
descriptor reads `*Minor, General and Hermetic*` — see **F-116**.

`kind` is `virtue` for all 35 and all 35 sit inside the Virtues section —
correct. `entity_kinds` is `["character"]` on all 35, which is right: none is a
covenant Boon.

**Check 12, mechanical.** Whole-file count of U+2212 in both locales'
`virtues_flaws.json`: **zero**. No entry in this batch carries a `description`
in either locale — all 35 are `name` + `summary` only. Every `name` and
`summary` was read against its passage in the matching language.

**This span has never been screened.** `crates/arm-rules/tests/uncomputed_clauses.rs::SWEPT_BLOCKS`
covers ArMDE:3360-3950 and its own comment says "the block ArMDE:3951-5282 is
still unswept". So
`no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` — the guard that
keeps `narrative` honest — has never looked at a single entry in this batch.
That is the direct explanation for this batch's classification failure rate,
which is far higher than B03's: B03's span was mostly inside the swept block.
The standard applied here is `ecb5150`'s, verbatim: a cap, a formula, a target
number, a rounding direction, an absolute/immunity and a step counted in
magnitudes are each "as mechanical as a +3".

Part C systemic gaps are **not** re-reported per entry. In particular
`advancement_mod` being surfaced-only (C1) is not counted as a defect of
`virtue.good_teacher` or `virtue.independent_study` — what *is* counted for Good
Teacher is that its own `source` value points the wrong way (F-91).

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D1 / D4 — exercised once.** `virtue.inventive_genius` is one of the nine
`lab_total_mod` carriers and is named in both rulings' tables. Per D1 its
condition is deliberately ignored in `effective/spell.rs::spell_level_cap`; per
D4 its +3 **does** apply in a character-generation Lab Total. Its
`{"type":"lab_total_mod","amount":3}` is therefore **not** marked wrong on
either account. Its exact wording is transcribed verbatim under "Directed reads"
for D4's Phase 2 slice.

**D2 — the directed read lands here.** `virtue.great_characteristic` is index
110 in this batch. The evidence is quoted in full under "Directed reads". **No
verdict is taken**: per D2 no batch marks
`validate_characteristic_delta_preconditions` either correct or defective until
Norbert rules, and this batch does not.

**D3's precedent is applied throughout** and decides most of this batch's
classification moves: an engine that structurally *cannot* express a rule is
grounds for `uncomputed_rule` with the rule written into both locales, and never
for `narrative`. That is what moves `virtue.guardian_angel` (F-99, a flat Magic
Resistance of 15 that the per-Form grid cannot express), `virtue.gossip` (F-93,
a Reputation doubling with no doubling mechanism), `virtue.homing_instinct`
(F-107), `virtue.imbued_with_the_spirit_of_form` (F-109),
`virtue.greater_benediction` (F-121, a player-defined open-ended blessing) and
the three immunities (F-95, F-110, F-111). Where the engine *can* express the
rule, the
move is to `creation_effect` instead: `virtue.guild_dean` (F-101),
`virtue.guild_master` (F-102).

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.the_gift` | 3967-3970 | narrative → ? | OK | page-63 mechanics in neither locale | F-90 Q-23 |
| `virtue.good_teacher` | 3971-3974 | OK | `{book,+3}` points the wrong way | OK | F-91 |
| `virtue.giant_blood` | 3975-3978 | OK | OK | OK | clean |
| `virtue.gorgiastic` | 3979-3982 | narrative → uncomputed_rule | OK | cap of 4 in neither locale | F-92 |
| `virtue.gossip` | 3983-3986 | narrative → uncomputed_rule | OK | doubling + 6+ roll in neither locale | F-93 |
| `virtue.great_characteristic` | 3987-3989 | OK | OK | OK | D2 evidence |
| `virtue.greater_benediction` | 3991-4008 | narrative → uncomputed_rule | no parameter for the chosen benediction | insert examples in neither locale | F-121 Q-24 Q-31 |
| `virtue.greater_immunity` | 4009-4016 | narrative → uncomputed_rule | no parameter; `max_per_target: 255` permits identical copies | immunity rule in neither locale | F-94 F-95 Q-29 Q-31 |
| `virtue.greater_power` | 4017-4026 | OK | OK | Initiative formula, Fatigue cost, Penetration rule in neither locale | F-96 Q-14 Q-22 Q-31 |
| `virtue.greater_purifying_touch` | 4027-4030 | narrative → uncomputed_rule | no parameter for the chosen disease | Fatigue cost + single-disease limit in neither locale | F-97 F-98 Q-31 |
| `virtue.guardian_angel` | 4031-4036 | narrative → uncomputed_rule | +5 Soak and MR 15 unmodelled | both numbers in neither locale | F-99 Q-28 |
| `virtue.guest_of_house_criamon` | 4037-4040 | `?` | OK | OK | Q-25 |
| `virtue.guild_apprentice` | 4041-4044 | narrative → uncomputed_rule | OK | Poor/Wealthy nullification in neither locale | F-100 Q-26 ArMDE:2816 |
| `virtue.guild_dean` | 4045-4048 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-101 ArMDE:2816 |
| `virtue.guild_master` | 4049-4052 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-102 ArMDE:2816 |
| `virtue.harnessed_magic` | 4053-4058 | narrative → uncomputed_rule | OK | cancellation rule in neither locale | F-103 |
| `virtue.heartbeast` | 4059-4061 | OK | page-233 cross-reference drops three rules | same three rules in neither locale | F-104 Q-14 Q-22 |
| `virtue.hermetic_experience` | 4063-4066 | OK | pool instance scope ×2; no-further-XP clause unmodelled | second sentence in neither locale | F-105 Q-27 |
| `virtue.hermetic_magus` | 4067-4070 | narrative → ? | prereq is "has The Gift" where the book says "only magi" | OK | F-106 ArMDE:2816 |
| `virtue.hermetic_prestige` | 4071-4073 | OK | OK | de renders the game term *Reputation* as `Ansehen` | F-120 (Phase-0 OQ3) |
| `virtue.hex` | 4075-4078 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.homing_instinct` | 4079-4084 | narrative → uncomputed_rule | OK | formula + Ease Factor in neither locale; de name contradicts the canonical table | F-107 F-108 ArMDE:2960 |
| `virtue.imbued_with_the_spirit_of_form` | 4085-4094 | narrative → uncomputed_rule | OK | vis substitution in neither locale | F-109 |
| `virtue.immune_to_disease` | 4095-4098 | narrative → uncomputed_rule | OK | immunity in neither locale | F-110 ArMDE:2960 |
| `virtue.immunity_to_cold` | 4099-4102 | narrative → uncomputed_rule | OK | immunity in neither locale | F-111 ArMDE:2960 |
| `virtue.improved_characteristics` | 4103-4105 | OK | OK | OK | clean |
| `virtue.indescribable_face` | 4107-4114 | `?` | the required choice of form has no parameter | OK | F-112 Q-26 |
| `virtue.independent_study` | 4115-4118 | OK | OK | de name contradicts the canonical table | F-113 |
| `virtue.induction` | 4119-4122 | OK | OK | OK | clean; ArMDE:2960 instance |
| `virtue.ineslemen` | 4123-4126 | OK | OK | OK | clean; Q-27 ArMDE:2816 |
| `virtue.infernal_heirloom` | 4127-4132 | narrative → uncomputed_rule | OK | level-25 / once-per-day in neither locale | F-114 ArMDE:2960 |
| `virtue.inoffensive_to_beings` | 4133-4142 | narrative → uncomputed_rule | `hermetic` is a descriptor category, filed as `index_categories` | Gift exemption in neither locale | F-115 F-116 Q-30 |
| `virtue.inspirational` | 4143-4146 | narrative → uncomputed_rule | OK | +3 in neither locale | F-117 |
| `virtue.intuition` | 4147-4150 | narrative → uncomputed_rule | OK | simple-die 6+ in neither locale | F-118 |
| `virtue.inventive_genius` | 4151-4154 | OK | OK (D1/D4) | the +6 experiment clause in neither locale | F-119 Q-14 Q-22 |

**Totals (post-reconciliation): 28 entries carry at least one finding, 1 more
carries only an open question** (`virtue.guest_of_house_criamon`, **Q-25**),
**6 are clean**: `virtue.giant_blood`, `virtue.great_characteristic`,
`virtue.hex`, `virtue.improved_characteristics`, `virtue.induction`,
`virtue.ineslemen`. 28 + 1 + 6 = 35.

*(Two entries left the clean/question-only list during reconciliation.
`virtue.hermetic_prestige` was clean in the first pass and was moved out by a
later German-terminology sweep — **F-120**, independently confirmed by the
verification agent. `virtue.greater_benediction` was rated `?` in the first pass
and the verification agent overturned that to a defect — **F-121**. See
"Sub-agent reconciliation".)*

## Directed reads

### D2 — `virtue.great_characteristic`, ArMDE:3987-3989

**The passage, verbatim, English (the entry's whole body, ArMDE:3989):**

> You may raise any Characteristic that already has a score of at least +3 by one point, to no more than +5. Make sure you describe what it is about you that causes that increase (such as sheer bulk, a lean build, or extreme charisma). You may take this Virtue twice for the same Characteristic, and for more than one Characteristic.

**German, ArMDE:3989 (line-parallel):**

> Du kannst jede Eigenschaft, die bereits einen Wert von mindestens +3 hat, um einen Punkt anheben, auf höchstens +5. Beschreibe genau, was an dir diese Steigerung verursacht (wie zum Beispiel schiere körperliche Masse, eine schlanke Figur oder extremes Charisma). Du kannst diese Tugend zweimal für dieselbe Eigenschaft und für mehr als eine Eigenschaft nehmen.

**The verb is the evidence, and it is not `take`.** The precondition sentence
says "You may **raise** any Characteristic that already has a score of at least
+3" — German "Du kannst jede Eigenschaft, die bereits … hat, … **anheben**". The
condition is attached to the **act of raising**, i.e. to the effect, not to the
purchase. The very next sentence in the same paragraph *does* use the
purchase verb for the multiplicity rule — "You may **take** this Virtue twice"
/ "Du kannst diese Tugend zweimal … **nehmen**" — so the book distinguishes the
two verbs inside one entry, deliberately.

This is the third verb-pair the audit has collected, and all three point the
same way. B02 found ArMDE:3665 "You may only **take** this Virtue if…" against
ArMDE:3669 "Only a character with … may **have** Demonic Powers". Here the
contrast is *inside a single paragraph*: a `raise`-conditioned clause and a
`take`-conditioned clause, one sentence apart.

**What the rest of the batch says about granted Virtues.** The span is unusually
rich in purchase-conditioned clauses, and every one of them uses `take`:

- ArMDE:3977 (Giant Blood): "You cannot **take** this Virtue and Large (page 89), Small Frame (page 145), or Dwarf (page 126)." — DE :3977 "Du kannst diese Tugend nicht zusammen mit Großgewachsen … nehmen."
- ArMDE:4011 (Greater Immunity): "You may not **take** immunity to aging — see the Unaging Minor Virtue (page 114) instead."
- ArMDE:4015 (Greater Immunity): "You may **take** this Virtue more than once, with a different immunity each time."
- ArMDE:4021 (Greater Power): "This Virtue may be **taken** more than once, and the levels added together to create several powers."
- ArMDE:4029 (Greater Purifying Touch): "You must choose the disease that you can cure when you **take** this Virtue."
- ArMDE:4069 (Hermetic Magus): "All magi must **take** this as their Social Status, and only magi may **take** it."
- ArMDE:4105 (Improved Characteristics): "You may **take** this Virtue multiple times."
- ArMDE:4139 (Inoffensive to (Beings)): "You may not **take** this Virtue more than once; … UnGifted characters may **take** this Virtue only if they have the Flaw Magical Air."

Against that eight-strong `take` block, Great Characteristic's precondition is
the one clause in the span phrased as a condition on the **operation**.

**And the span states the only thing the book says here about a grant.**
ArMDE:4061 (Heartbeast): "Note that all Bjornaer magi gain this Virtue **for
free** at character creation." — DE :4061 "Beachte, dass alle Bjornaer-Magi diese
Tugend bei der Charaktererschaffung kostenlos erhalten." The general statement
sits outside the span at ArMDE:2264: "Membership in a House **grants** a
particular benefit at character creation, which is listed in the table", and the
table row at :2272 reads "Heartbeast (page 85), beginning score of 1 in
Heartbeast Ability." Both sentences say only that the Virtue arrives free.
**Neither says anything about the Virtue's own conditions being waived**, and
nowhere in the span — or in :2264-2284 — is there a sentence exempting a granted
Virtue from what its description requires.

**What that adds up to, stated as evidence and not as a ruling.** Two
independent strands both bear on D2:

1. Great Characteristic's precondition is not a purchase condition at all. It
   conditions the *raise*. A granted copy still performs a raise, so on the
   plain reading the condition still has to be met — which is what B02's
   `have`/`take` split already suggested, restated here in a much sharper form
   because both verbs appear in one paragraph.
2. Nothing in the book waives a granted Virtue's own conditions. The grant
   language is uniformly about **cost** ("for free", "in addition to the normal
   allowance" at ArMDE:2275), never about **eligibility**.

No batch verdict is taken on
`validation/scores.rs::validate_characteristic_delta_preconditions` or
`validate_ability_bonus_targets`. D2 is Norbert's.

**How reachable D2's worry-case actually is, in the shipped data.** This bounds
how much the ruling would change and is recorded so the decision is not taken in
the abstract. *(The second paragraph corrects this batch's first pass, which had
missed the Mythic Companion path entirely; it was contributed by the independent
verification pass and re-checked against both the source and the code.)*

No **House** grant confers `virtue.great_characteristic`. The grants in
`rules/core/houses.json` are `virtue.heartbeast`, `virtue.the_enigma`,
`virtue.faerie_magic`, `virtue.verditius_magic`, Puissant Arts/Abilities,
`virtue.hermetic_prestige`, `virtue.minor_magical_focus` and
`virtue.self_confident`, plus Ex Miscellanea's and Jerbiton's **open** slots. An
open slot *could* be filled with it — `grant.rs::GrantConstraint` matches on
`kind`, `magnitude` and categories, and Great Characteristic is a Minor General
Virtue — so Jerbiton's "A Minor Virtue relating to scholarship arts, or mundane
interaction" is the one path by which a *granted* Great Characteristic could
arise today.

**The Mythic Companion path exists but is budgeted, not granted, and the book
says so.** `rules/core/mythic_companion_types.json:61-62` names
`virtue.great_characteristic` twice — Stamina and Strength, for Nephilim — but
in **`required_virtues`**, not in `grants`. That distinction is load-bearing:
`mythic_companion.rs::MythicCompanionType::required_virtues` is documented
"Fixed required Virtues (**budgeted**)", stored as `Selection`s and auto-seeded
by the UI, so they land in `entity.selections` and **are** inside
`validate_characteristic_delta_preconditions`' bought-only scope already. And
the book agrees: ArMDE:2723-2731 lists Nephilim's required Virtues — "Great
Stamina (Minor, General) · Great Strength (Minor, General)" — and marks exactly
one of them free ("Strong Angelic Heritage (Minor, Supernatural — **free with
Nephilim**)"), closing with ":2731 Nephilim must take five points of Flaws **to
pay for these virtues**". The app's encoding matches the book's: these two are
paid for, so the "already at +3" gate applies to them under any reading of D2.

So the honest summary is: **D2's gap is real in the code but currently
unreachable in shipped data except through Jerbiton's open slot.** That does not
make the ruling less necessary — an open slot is a live path and the next House
or supplement could add a fixed one — but it does mean no shipped character is
mis-validated today.

### D4 Phase 2 — `virtue.inventive_genius`, ArMDE:4151-4154

**Verbatim, English (ArMDE:4153, the entry's whole body):**

> Invention comes naturally to you. You get +3 to your Lab Total if you are not using a Laboratory Text or being taught. If you experiment, you get +6.

**Verbatim, German (ArMDE:4153, line-parallel):**

> Erfinden liegt dir im Blut. Du erhältst +3 auf deine Laborsumme, wenn du keinen Labortext verwendest und auch nicht unterrichtet wirst. Wenn du experimentierst, erhältst du +6.

**Transcription notes for D4's Phase 2 slice.** Three things the wording fixes
that a paraphrase would lose:

- The condition is a **conjunction of two negatives**: *not* using a Lab Text
  **and** *not* being taught. German makes the conjunction explicit
  ("wenn du keinen Labortext verwendest **und auch nicht** unterrichtet wirst");
  the English leaves it to the shared scope of "if you are not". D4's table row
  ("not using a Lab Text, not being taught") is faithful.
- The +6 is **not** a third state stacked on the +3. "If you experiment, you get
  +6" replaces the +3 — the two are alternatives, not addends. Nothing in either
  language adds them.
- The +6 has **no** stated Lab-Text/teaching condition of its own. It is
  conditioned only on experimenting.

Per D1 the condition is deliberately ignored in
`effective/spell.rs::spell_level_cap`; per D4 the +3 **does** apply in a
character-generation Lab Total. The shipped
`{"type":"lab_total_mod","amount":3}` is therefore correct on both rulings and
is **not** marked wrong. What *is* marked is that the +6 alternative reaches
neither locale — **F-119**.

## Findings

**One qualifier that applies to both authorization findings (F-101, F-102) and
is stated once rather than twice.** A14 records that
`validation/authorization.rs::validate_ability_authorization` exempts any
character whose type profile has `is_magus`. Guild Dean and Guild Master are
both Social Status Virtues for townsfolk, so the exempted population is empty in
practice: the characters who take them are companions and grogs, which is
exactly who the refusal bites.

**A second qualifier, for every `narrative → uncomputed_rule` move below.**
B1 records that `classification` is read by no production code, so none of these
moves changes a computed number. The cost is the one `ecb5150` names: a
`narrative` entry is not obliged to carry its rule in `description`, so the rule
leaves the application silently while the entry still looks complete. Every move
below therefore carries the same correction — reclassify **and** write the rule
into `description` in **both** locales, or
`uncomputed_clauses.rs::every_uncomputed_rule_entry_states_its_rule_in_every_locale`
goes red. Severity for all of them: **lost-rule / provenance**, not
miscalculation.

### F-90 — `virtue.the_gift` — `narrative` on a stub whose pointer leads to five rules, none of which reaches the entry's text

**Passage** (ArMDE:3969, verbatim, English — the entry's whole body):
> You have the ability to work magic. See earlier, page 63, for full details.

German, ArMDE:3969 (line-parallel):
> Du besitzt die Fähigkeit, Magie zu wirken. Vollständige Einzelheiten finden sich weiter oben auf [Seite 63](#die-gabe).

**Where the pointer leads.** `#die-gabe` / page 63 is `### The Gift` at
ArMDE:2868, and ArMDE:2870-2876 is the Virtue's real content. Verbatim, the
rules it states:

> The character suffers all the penalties of The Gift, just as magi do (see page 203), but can be taught Supernatural Abilities without having to take the corresponding Virtues (see page 383 for rules). Most importantly, the character can be taught Hermetic Magic, so all magi must have this Virtue. A character with The Gift, even if he is not a magus, may take Hermetic Virtues and Flaws which relate to intrinsic ability rather than background or training. (ArMDE:2870)

> Characters who have The Gift may start play with a single Supernatural Ability, without having to take any other Virtue, but if they wish to learn others they must find opportunities to do so in the course of the saga. (ArMDE:2874)

> Grogs can never have The Gift, as a character with The Gift is too important to be a grog. (ArMDE:2876)

**Current data:** `"classification": "narrative"`, no `effects`, no
`prerequisites`, `incompatible_with` of five ids, `categories: ["special"]`.

**Why it is wrong.** This is B03's F-87 shape — a `(see page NNN)` pointer at
real mechanics — and here the pointed-at mechanics are not marginal: four of
them are **computed by this engine today**, and every one of them is computed
somewhere *other* than this entry:

| Rule | Where the engine does it |
|---|---|
| "Grogs can never have The Gift" (:2876) | `rules/core/character_types.json` — grog `forbidden_traits: ["virtue.the_gift"]`, `gift_policy: "forbidden"` |
| "all magi must have this Virtue" (:2870) | magus `gift_policy: "required"` |
| "may take Hermetic Virtues and Flaws" (:2870) | companion `permitted_categories` — `hermetic` conditional on `{"kind":"has","value":"virtue.the_gift"}` |
| "may start play with a single Supernatural Ability" (:2874) | `effective/reputation_and_caps.rs::supernatural_free_slots` |
| "suffers all the penalties of The Gift" (:2870) | nowhere — the social penalty is not modelled |

So `narrative` — the claim that the book states nothing mechanical for this
entry — is contradicted five times over. It is also the single most load-bearing
entry in the catalogue: `gift_id` on all four type profiles names it, and it is
the pivot of three `permitted_categories` rules.

**`incompatible_with` (check 8) is complete and correctly sourced** — verified
rather than assumed, because a five-item list on a `narrative` entry is exactly
the shape B03's Q-19 flagged as possibly unsourced. It is not. ArMDE:2637 states
the rule for four of the five in one sentence:

> All Mythic Companions take a Free Virtue which specifies their status. **These Virtues are incompatible with each other, and with The Gift**, and are not available to grogs.

and the `### Mythic Companion, Free` index block at ArMDE:3329-3334 lists
exactly four such Virtues — Devil Child, Faerie Doctor, Nephilim, Spirit Votary
— all four of which are on the list. The fifth,
`virtue.failed_apprentice`, is sourced separately at ArMDE:3845 ("You may not
have The Gift"). Nothing is missing and nothing is invented.

**Correct value:** not `narrative`. Which of the other three it becomes is
**Q-23**: the profile-side modelling is the same shape C7 records for
`virtue.devil_child` (a `creation_effect` carrying no effects, because the
mechanic lives on the Mythic Companion profile), which argues `creation_effect`;
the unmodelled social penalty argues `uncomputed_rule`. Either way the
:2870-2876 rules must be written into `description` in **both** locales.

**Severity:** lost-rule / provenance, on the catalogue's most-referenced entry.

---

### F-91 — `virtue.good_teacher` — the `book` modifier points at the wrong person

**Passage** (ArMDE:3973, verbatim, English — the entry's whole body):
> You can explain new concepts and skills with great facility. Add three to the Quality of any books that you write, and five to the Source Quality for anyone who studies with you.

German, ArMDE:3973 (line-parallel):
> Du kannst neue Konzepte und Fähigkeiten mit großer Leichtigkeit vermitteln. Addiere drei zur Qualität aller Bücher, die du schreibst, und fünf zur Quellenqualität für jeden, der bei dir studiert.

**Current data:**
```json
"effects": [
  { "type": "advancement_mod", "source": "teaching", "amount": 5 },
  { "type": "advancement_mod", "source": "book",     "amount": 3 }
]
```

**Why it is wrong.** The two rows are not symmetric, and only one of them is
right.

- `{"source":"teaching","amount":5}` is **correct**. `types.rs::AdvancementSource::Teaching`
  is documented as "The character **teaching others** (Good Teacher,
  Incomprehensible)" — it is the outward-facing source, and +5 to "the Source
  Quality for anyone who studies with you" is exactly it.
- `{"source":"book","amount":3}` is **wrong in direction**.
  `AdvancementSource::Book` is documented as "Learning from a book (Book
  Learner)", and the catalogue confirms the reading: `virtue.book_learner`
  carries the identical `{"source":"book","amount":3}` for ArMDE:3521 "When
  studying from books, treat them as if they were three Quality levels higher
  than they actually are." So `book` unambiguously means *this character
  reading*. Good Teacher's +3 is to "the Quality of any books that **you
  write**" — a bonus to every *other* reader, and never to this character's own
  book study.

Because `advancement_mod` is surfaced-only (C1), the consequence is a **wrong
displayed modifier**: a Good Teacher's read-out lists a +3 on his own
book-learning that the rulebook does not grant him, side by side with Book
Learner's identical row. The taxonomy has no outward-facing book source — there
is a `Teaching` for taught-others but no `Authoring` for written-books — so this
is the `Teaching`/`Taught` split half-done.

**Correct value:** remove the `{"source":"book","amount":3}` row and carry "add
three to the Quality of any books that you write" as text. `classification`
stays `in_play_effect` (the entry still carries the `teaching` effect, and B1
records that carrying an `Effect` is what decides that line). Adding an
`AdvancementSource::Authoring` variant is the alternative and is out of this
audit's scope — **Q-32**.

**Severity:** wrong rules output (a modifier displayed on a total the book does
not modify)

---

### F-92 — `virtue.gorgiastic` — `narrative` on a hard score cap

**Passage** (ArMDE:3981, verbatim, English — the relevant sentences):
> These magi may have Enigmatic Wisdom and House Criamon Lore scores after character creation. The character's Enigmatic Wisdom Score cannot exceed 4 without the assistance of Criamon magi, or a magical breakthrough.

German, ArMDE:3981 (line-parallel):
> Diese Magi dürfen nach der Charaktererschaffung Werte in Enigmatischer Weisheit und Hauskundige: Criamon besitzen. Der Wert der Enigmatischen Weisheit des Charakters kann ohne die Hilfe von Criamon-Magi oder einen magischen Durchbruch 4 nicht überschreiten.

**Current data:** `"classification": "narrative"`, no `effects`, no
`prerequisites`, `categories: ["hermetic"]`, `magnitude: "minor"`.

**Why it is wrong.** "cannot exceed 4" is a cap on an Ability score — precisely
the shape `ecb5150` reclassified nineteen entries for ("a cap … is as mechanical
as a +3"). It escapes the mechanical-token screen only because the span is
unswept *and* because the German idiom is the discontinuous negation
("kann … 4 nicht überschreiten") that `MECHANICAL_PHRASES` structurally cannot
see; the English "cannot exceed" is not in the list either.

Two things that are **not** defects and are recorded so a later pass does not
re-flag them. First, the permission clause is explicitly scoped "**after**
character creation", so no `ability_authorization` is owed: the gate
`validate_ability_authorization` enforces is a creation-time gate, and magi are
whole-character exempt from it anyway. Second, `ability.enigmatic_wisdom` is
`arcane` in `rules/core/abilities.json`, which matches the book — ArMDE:7227
lists `[Enigmatic Wisdom\*]` under `#### Arcane Abilities`, and the `*` marks an
Ability that cannot be attempted unskilled (ArMDE:4157), not a Supernatural one.

**Correct value:** `"classification": "uncomputed_rule"`, with the Enigmatic
Wisdom ceiling of 4 (and the condition that lifts it) written into `description`
in both locales.

**Severity:** lost-rule / provenance

---

### F-93 — `virtue.gossip` — `narrative` on a doubling and a target number

**Passage** (ArMDE:3985, verbatim, English — the entry's whole body):
> You have regular social contacts in the area that provide you with all kinds of information about local social and political goingson. On a simple roll of 6+, you hear interesting news before almost everyone else. You treat all local Reputations as twice their actual level. With some well-placed words, you may be able to bestow new Reputations (whether deserved or not). You quite likely have a Reputation too — as a gossip.

German, ArMDE:3985 (line-parallel):
> … Bei einem einfachen Wurf von 6+, erfährst du interessante Neuigkeiten fast vor allen anderen. Du behandelst alle lokalen Reputationen als doppelt so hoch wie ihren tatsächlichen Wert. …

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Two mechanical clauses, either of which is disqualifying:

- **"On a simple roll of 6+"** — a named die kind plus a target number.
  `MECHANICAL_PHRASES` carries "simple die"/"einfachen würfel" precisely for
  this, and misses it here only because the book writes "simple **roll**" in
  English (the German "einfachen Wurf" likewise misses "einfachen würfel" by one
  word).
- **"You treat all local Reputations as twice their actual level"** — a ×2 on a
  score the engine stores. `Entity::reputations` rows carry a `score`, and
  `validation/scores.rs::validate_reputations` reads them; nothing doubles
  anything, and no `Effect` variant expresses a multiplier on a Reputation. Per
  **D3** that inability is grounds for `uncomputed_rule` with the rule written
  out, never for `narrative`.

The third clause — "you may be able to bestow new Reputations" — is GM
judgement and correctly computes nothing.

**Correct value:** `"classification": "uncomputed_rule"`, with the 6+ simple
roll and the local-Reputation doubling in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-94 — `virtue.greater_immunity` — the book demands a choice, records nothing, and then lets the same choice be taken 255 times

**Passage** (ArMDE:4011-4015, verbatim, English):
> You are completely immune to one hazard which is both common and potentially deadly. For example, you might be immune to fire or to iron (and only iron) weapons. You may not take immunity to aging — see the Unaging Minor Virtue (page 114) instead. This immunity applies to mundane and magical versions of the thing. If you are immune to fire, you are also immune to magically created fire. (:4011)

> You may take this Virtue more than once, with a different immunity each time. (:4015)

German, ArMDE:4011 / :4015 (line-parallel):
> Du bist vollständig immun gegen eine Gefahr, die sowohl häufig als auch potenziell tödlich ist. … Du darfst keine Immunität gegen Altern nehmen – siehe stattdessen die kleine Tugend Nicht alternd ([Seite 114](#nicht-alternd)). …

> Du kannst diese Tugend mehr als einmal nehmen, jedes Mal mit einer anderen Immunität.

**Current data:** `"classification": "narrative"`, **no `parameters`**,
`"max_per_target": 255`, no `effects`.

**Why it is wrong — two joined defects, and the second is caused by the first.**

1. **No parameter.** "You are completely immune to **one** hazard" and "with a
   **different** immunity each time" both require the chosen hazard to be
   recorded. It is not. A character sheet with two Greater Immunities says
   nothing about what they are, and the Markdown export prints two identical
   rows. This is check 9's "does the book demand a choice?" answered yes and
   modelled no. The right domain is `text` (the hazard is free-form: fire, iron
   weapons, deprivation), which is the same shape
   `types.rs::ParameterDomain::Text` already serves elsewhere.

2. **`max_per_target: 255` is the wrong multiplicity axis.** B10: the grouping
   key is `(item_ref, **whole** params map)`. With **no** params, every copy is
   the same target, so 255 means *255 identical copies of the same unnamed
   immunity are legal*. The book's permission is conditional — "more than once,
   **with a different immunity each time**" — and ArMDE:2814 states the default
   the entry is deviating from ("A Virtue or Flaw may be taken more than once
   only if the description explicitly allows it"). The description allows
   *distinct* repeats, not identical ones.

The two fix together: add the `text`-domain parameter and set `max_per_target`
back to **1**. Then one copy per named hazard is legal and a second copy of the
same hazard draws `duplicate_selection`, which is exactly what the sentence
says. `max_total` stays at the 255 sentinel — the book states no ceiling on how
many different immunities you may buy.

**Contrast that proves the 255 is not simply the house style for repeatable
Virtues.** `virtue.greater_power` three entries away carries the same
`max_per_target: 255` and is **correct**, because ArMDE:4021 says "This Virtue
may be taken more than once, and the levels added together" — identical copies
are the *point* there. The two passages differ and the data does not.

**Severity:** wrong rules output (an illegal character validates clean) plus a
lost player choice

---

### F-95 — `virtue.greater_immunity` — `narrative` on a complete immunity

**Passage:** as F-94, ArMDE:4011-4015. The clauses at issue:
> You are **completely immune** to one hazard which is both common and potentially deadly. … This immunity applies to mundane and magical versions of the thing. If you are immune to fire, you are also immune to magically created fire. (:4011)

> One important possibility is immunity to deprivation, which means that you suffer **no loss of Fatigue or wounds** from going without air, food or drink. However, you cannot regain long-term Fatigue without rest and sustenance, and if you are injured deprivation could cause your wounds to worsen. (:4013)

German, ArMDE:4013 (line-parallel):
> Eine wichtige Möglichkeit ist die Immunität gegen Entbehrung, was bedeutet, dass du keinen Erschöpfungs- oder Wundenverlust durch den Mangel an Luft, Nahrung oder Wasser erleidest. …

**Current data:** `"classification": "narrative"`.

**Why it is wrong.** `ecb5150`'s title names this exact category — "Stop calling
a cap, a formula and **an immunity** 'flavour'" — and its body lists "an
absolute ('cannot die as a result of wounds or old age')" among the six shapes
that are "as mechanical as a +3". "Completely immune", "applies to mundane and
magical versions", and the deprivation clause's "no loss of Fatigue or wounds"
are three absolutes in one entry. The engine has no immunity axis, so per **D3**
the answer is `uncomputed_rule` with the rule written out.

Two neighbouring entries carry the identical defect and are reported separately
because they are separate entries: `virtue.immune_to_disease` (F-110) and
`virtue.immunity_to_cold` (F-111). All three are `narrative` today. A fourth,
`virtue.lesser_immunity` at ArMDE:4275-4278, is **also** `narrative` and sits in
B05 — flagged there, not here.

**Correct value:** `"classification": "uncomputed_rule"`, with the immunity, the
mundane-and-magical clause, the aging exclusion and the deprivation rider in
`description` in both locales.

**Severity:** lost-rule / provenance

---

### F-96 — `virtue.greater_power` — one `power_levels` row, and four rules the passage states around it reach nobody

**Passage** (ArMDE:4019-4025, verbatim, English — the clauses the effect does
not carry):
> If you take the Virtue once, this is a single power, equivalent to a Formulaic Hermetic spell with a level of 50 or lower. You may also spend levels one-for-one to give the power Penetration; otherwise, it has a Penetration of zero. (:4019)

> The power has an Initiative equal to the character's Quickness - (power magnitude/2) (rounded down). It costs one Fatigue level to activate if its level is less than or equal to 50, or two Fatigue levels if its level is 51 to 100. Higher level effects are unlikely to make sense. (:4023)

> This Virtue may be associated with any supernatural realm. … The power must be associated with the same supernatural realm as the system on which it is based. (:4025)

German, ArMDE:4023 (line-parallel):
> Die Macht hat eine Initiative gleich der Schnelligkeit des Charakters – (Magnitude der Macht / 2) (abgerundet). Das Aktivieren kostet eine Erschöpfungsstufe, wenn ihre Stufe 50 oder niedriger ist, oder zwei Erschöpfungsstufen, wenn ihre Stufe 51 bis 100 beträgt. …

**Current data:** `"classification": "creation_effect"`,
`"effects": [{"type":"power_levels","amount":50}]`, `"max_per_target": 255`, no
`description` in either locale; both summaries are the passage's first sentence.

**What is correct, so the finding is not read wider than it is.** The 50 is
right (":4019 a level of 50 or lower"). `max_per_target: 255` is right
(":4021 may be taken more than once, and the levels added together"). And the
Penetration rule *is* computed — A27 records `powers_used = Σ (power.level +
power.penetration)`, and `types.rs::SupernaturalPower::penetration`'s doc
comment cites `ArMDE:4019` and reproduces the book's own 60/0 + 20/20 = 100
worked example. So three of the four are fine.

**Why it is still a finding.** The Initiative formula
(`Quickness − magnitude/2`, rounded down) and the Fatigue activation cost
(one level at ≤50, two at 51-100) are stated in numbers, are computed by
nothing, and appear in **neither** locale's text — the summary stops at "The
character has a supernatural power that he can activate at will." A player
reading this Virtue in the app learns the level budget and nothing else about
using the power.

**Correct value:** a `description` in both locales carrying the Initiative
formula and the Fatigue costs. **This is a `description` proposed on a
`creation_effect`, which is a new practice** — 0 of 125 `creation_effect`
entries carry one today — so it is recorded as depending on **Q-14 / Q-22** and
is not a plain defect.

The realm clause is an instance of the catalogue-wide ArMDE:2960-2962 rule and
is listed under "Recurring patterns" rather than re-argued here.

**Severity:** lost-rule, pending Q-14/Q-22

---

### F-97 — `virtue.greater_purifying_touch` — `narrative` on a Fatigue cost and a one-disease limit

**Passage** (ArMDE:4029, verbatim, English — the entry's whole body):
> You can, with a touch and the expenditure of a Fatigue level, cure a single serious disease. This disease should be either life-threatening or seriously disabling, and should be one from which people do not normally recover by themselves. You must choose the disease that you can cure when you take this Virtue, and you can only cure that disease. You can only choose a disease, not other types of injury or misfortune. See page 406 for more information on diseases, and Art and Academe, page 45 for more detail.

German, ArMDE:4029 (line-parallel):
> Du kannst mit einer Berührung und dem Aufwand einer Erschöpfungsstufe eine einzelne schwere Krankheit heilen. … Du musst die Krankheit, die du heilen kannst, beim Nehmen dieser Tugend festlegen, und du kannst nur diese Krankheit heilen. …

**Current data:** `"classification": "narrative"`, no `parameters`, no `effects`.

**Why it is wrong.** "the expenditure of a Fatigue level" is a resource cost in
a tracked game resource — the same currency `Entity`'s Fatigue track and
`derived/combat.rs` use — and "you can only cure that disease" is an absolute
restriction on the power. Neither is fiction. The cross-reference to *Art and
Academe* p.45 is to a book not in `rules/source/en/`, so nothing follows from it
(and citing it is not a provenance violation, because the rule this entry states
is stated *here*, at :4029).

**Correct value:** `"classification": "uncomputed_rule"`, with the Fatigue cost
and the single-disease restriction in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-98 — `virtue.greater_purifying_touch` — "You must choose the disease … when you take this Virtue" and nothing records it

**Passage:** ArMDE:4029, the sentence quoted in F-97:
> You must choose the disease that you can cure when you take this Virtue, and you can only cure that disease.

German, ArMDE:4029:
> Du musst die Krankheit, die du heilen kannst, beim Nehmen dieser Tugend festlegen, und du kannst nur diese Krankheit heilen.

**Current data:** no `parameters`.

**Why it is wrong.** This is the book demanding a choice in so many words —
"**must** choose … when you take this Virtue" — and the entry records none. The
chosen disease is the entire content of the Virtue: two characters with Greater
Purifying Touch who cure different diseases are mechanically different, and the
save file cannot tell them apart. B8 is explicit that a parameter no effect
consumes is legitimate and normal ("the parameter exists to record a player
choice the engine does not compute from"), which is exactly this case — there is
no engine obstacle, only a missing row.

**Correct value:** add
`{"key":"disease","type":"ref","domain":"text"}`. `max_per_target` stays 1 (the
book gives no permission to repeat this Virtue, and ArMDE:2814 makes once the
default).

**Severity:** lost player choice (silent data loss — the save cannot round-trip
the character's actual Virtue)

---

### F-99 — `virtue.guardian_angel` — `narrative` on "+5 bonus to Soak" and "a Magic Resistance of 15"

**Passage** (ArMDE:4035, verbatim, English — the second paragraph in full):
> Your guardian angel can also help in two practical ways. First, he can grant you a +5 bonus to Soak. Second, he can grant you a Magic Resistance of 15. This Magic Resistance is not compatible with a magus's Parma Magica, or Magic Resistance from most other sources, but it does add to the Magic Resistance resulting from Faith Points (see page 419). The angel only grants you these bonuses if you are acting in accordance with God's will.

German, ArMDE:4035 (line-parallel):
> Dein Schutzengel kann auch auf zwei praktische Weisen helfen. Erstens kann er dir einen Bonus von +5 auf die Absorption gewähren. Zweitens kann er dir eine Magieresistenz von 15 verleihen. Diese Magieresistenz ist nicht mit der Parma Magica eines Magus oder der Magieresistenz aus den meisten anderen Quellen vereinbar, addiert sich jedoch zur Magieresistenz, die aus Glaubenspunkten resultiert (siehe [Seite 419](#wahrer-glaube-1)). Der Engel gewährt dir diese Boni nur, wenn du im Einklang mit Gottes Willen handelst.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** This is the loudest misclassification in the batch: **two
explicit signed numbers**, +5 and 15, in one paragraph, on an entry asserting
that the book states nothing mechanical. It is not even a case the token screen
would have needed widening for — `has_signed_number` would have caught "+5"
on sight. It survives only because ArMDE:3951-5282 is unswept.

The two halves need different answers:

- **+5 Soak is directly expressible.** `soak_mod` (A34) is the variant, and
  `derived/combat.rs::soak` sums it unconditionally. The obstacle is the last
  sentence: "only … if you are acting in accordance with God's will". A34 already
  records the precedent — `virtue.berserk` carries a `soak_mod` that "is applied
  unconditionally rather than only while berserk; the engine has no condition to
  gate it on". Whether to follow that precedent here is **Q-28**; a standing +5
  Soak on a character whose angel has left him is a wrong number, and unlike
  Berserk the condition here is a *moral* one the player cannot even declare.
- **Magic Resistance 15 is not expressible.** A37's six `MagicResistanceEffect`
  kinds are all modifiers to the per-Form grid (`no_form_bonus`,
  `halved_parma`, `aura_bonus`, two susceptibilities, a penetration waiver);
  none grants a flat resistance. `derived/casting.rs::magic_resistance` computes
  `base = might if might > parma_for_form else parma_for_form`, so a flat 15
  would have to arrive as Might, which it is not. Per **D3** that is
  `uncomputed_rule` with the rule written out — and the non-stacking clause
  ("not compatible with a magus's Parma Magica … but it does add to … Faith
  Points") has to travel with it, because it is the rule that says *how* the 15
  interacts with the number the engine does compute.

**Correct value:** `"classification": "uncomputed_rule"` at minimum, with both
numbers, the God's-will condition and the Parma/Faith-Points stacking rule
written into `description` in both locales. Whether a `soak_mod` is added
alongside (which would make it `in_play_effect`) is **Q-28**.

**Severity:** lost-rule / provenance, on two stated numbers

---

### F-100 — `virtue.guild_apprentice` — `narrative` on a clause that switches two other Virtues off

**Passage** (ArMDE:4043, verbatim, English — the relevant sentence):
> The character is not able to benefit from either the Poor Flaw or the Wealthy Virtue, since he is essentially the property of his master, until he moves to the journeyman rank.

German, ArMDE:4043 (line-parallel):
> Der Charakter kann weder vom Fehler Arm noch von der Tugend Wohlhabend profitieren, da er im Wesentlichen das Eigentum seines Meisters ist, bis er den Rang eines Gesellen erreicht.

**Current data:** `"classification": "narrative"`, no `effects`, no
`incompatible_with`.

**Why it is wrong, and why the neighbours prove it.** B02 cleared the shape "The
Wealthy Virtue and Poor Flaw affect you normally" as *stating that no special
rule applies*, and the immediately preceding entry in the source —
`virtue.gentleman` at ArMDE:3961 — ends with exactly that sentence. Guild
Apprentice says the **opposite** of it, in the same words, negated. If the
positive form is "no rule", the negated form is a rule.

What the rule does: `virtue.wealthy` and `flaw.poor` carry
`later_life_xp_rate` effects, and the magus profile already lists them in
`forbidden_traits`. So the engine has both the effect and the machinery to
suppress it; what it does not have is this entry's suppression.

**How to encode it is genuinely open (Q-26).** "not able to benefit from" is
weaker than "may not take": the book leaves the character holding the Flaw's
points while denying its effect, which the engine cannot express —
`incompatible_with` would forbid the combination outright, changing the point
balance the book leaves intact. Per **D3** the honest answer is
`uncomputed_rule` with the rule written out, and a decision on
`incompatible_with` deferred.

**Correct value:** `"classification": "uncomputed_rule"`, with the
Poor/Wealthy nullification in `description` in both locales. `incompatible_with`
left alone pending Q-26.

**Severity:** lost-rule / provenance

---

### F-101 — `virtue.guild_dean` — "You may select Academic Abilities at character generation" is not encoded

**Passage** (ArMDE:4047, verbatim, English — the final sentence):
> You may select Academic Abilities at character generation.

German, ArMDE:4047 (line-parallel):
> Du kannst Akademische Fertigkeiten bei der Charaktererschaffung wählen.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** `academic` is one of the three entries of
`rules/core/abilities.json::categories_requiring_virtue`
(`["academic","arcane","martial"]`), so the permission is **computed**:
`validation/authorization.rs::validate_ability_authorization` raises
`ability_category_requires_virtue` against any non-magus holding an Academic
Ability unless something authorizes it. A Guild Dean who buys Artes Liberales,
Latin or Civil and Canon Law is refused by the app against a character the book
makes legal in one sentence. This is B01's nine-Social-Status shape, B02's F-38
/ F-62 and B03's F-66 / F-67 / F-68 exactly — the running total the brief tracks.

Note the shape: the passage grants a whole **category**, so the effect must
carry `categories`, not `abilities`. That is the `virtue.educated` half of the
pattern the brief names.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`.

**Severity:** wrong rules output

---

### F-102 — `virtue.guild_master` — the identical sentence, the identical omission

**Passage** (ArMDE:4051, verbatim, English — the final sentence):
> You may select Academic Abilities at character generation.

German, ArMDE:4051 (line-parallel):
> Du kannst Akademische Fertigkeiten bei der Charaktererschaffung wählen.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Same mechanism as F-101, on the adjacent entry. The two
sentences are byte-identical in both languages, which is worth recording: this
is not a one-off transcription slip but the extraction pass not looking for the
clause at all.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`.

**Severity:** wrong rules output

---

### F-103 — `virtue.harnessed_magic` — `narrative` on a spell-cancellation rule with a roll, a range and a death clause

**Passage** (ArMDE:4055-4057, verbatim, English):
> You have great control over your spells. You are able to cancel any of your spells simply by concentrating. You can even cancel the magic in magic items which you created. The act of canceling your magic should be treated as if you were casting a spell for timing and concentration purposes. If you are distracted and fail a Concentration roll, another attempt may be made in a later round. Spells and magic items can be canceled out over any distance, but once they have been canceled, you must recast a spell or reinvest a power in a magic item to start the effect again. (:4055)

> The drawback is that when you die, all of your spells and magic items sputter out. (:4057)

German, ArMDE:4055-4057 (line-parallel):
> … Das Aufheben deiner Magie sollte für Timing- und Konzentrationszwecke so behandelt werden, als würdest du einen Zauber wirken. Wirst du abgelenkt und misslingt dir ein Konzentrationswurf, kann in einer späteren Runde ein weiterer Versuch unternommen werden. Zauber und Zauberartefakte können über jede Entfernung hinweg aufgehoben werden, …

> Der Nachteil ist, dass beim Tod alle deine Zauber und Zauberartefakte erlöschen.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Three mechanical clauses, none of them fiction:

- a **procedure**: cancellation "treated as if you were casting a spell for
  timing and concentration purposes" — it takes a round, it uses the casting
  timing rules;
- a **roll and a retry rule**: "If you are distracted and fail a Concentration
  roll, another attempt may be made in a later round";
- an **absolute on range**: "over **any** distance" — the same shape as the
  "cannot die" absolute `ecb5150` names, stated positively.

Plus the drawback clause, which is a hard consequence, not colour.

**Correct value:** `"classification": "uncomputed_rule"`, with the cancellation
procedure, the Concentration-roll retry, the unlimited range and the
death clause in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-104 — `virtue.heartbeast` — the "(see page 233)" pointer carries a prohibition, and the entry drops it

**Passage** (ArMDE:4061, verbatim, English — the entry's whole body):
> You have been initiated into the Outer Mystery of the Heartbeast (see page 233), and thus are a member of House Bjornaer. You start with the Ability Heartbeast 1. Note that all Bjornaer magi gain this Virtue for free at character creation.

German, ArMDE:4061 (line-parallel):
> Du wurdest in das Äußere Mysterium des Herztieres eingeweiht (siehe [Seite 233](#bjornaer--das-herztier)) und bist damit ein Mitglied des Hauses Bjornaer. Du beginnst mit der Fertigkeit Herztier 1. Beachte, dass alle Bjornaer-Magi diese Tugend bei der Charaktererschaffung kostenlos erhalten.

**Where the pointer leads.** `#bjornaer--das-herztier` / page 233 is
`### Bjornaer — The Heartbeast` at ArMDE:9886. This is the *same pointer shape*
B03 followed on `virtue.faerie_magic` (F-87), and the brief names Heartbeast as
one of the two remaining carriers. Three rules there are the Virtue's own and
are stated nowhere else:

> Initiates of the Bjornaer mystery gain the Ability Heartbeast. **This Ability cannot be gained by any character who has not been Initiated into the mystery.** For Initiates of the Outer Mystery, the Heartbeast Ability is only used when something tries to stop the magus changing forms. In that case, a roll of Stamina + Heartbeast against an Ease Factor set by the storyguide allows the character to change anyway. (ArMDE:9888)

> Bjornaer magi can cast spells while in the form of their heartbeast, but they cannot speak or make the appropriate gestures, and thus normally take a **–15 penalty**. (ArMDE:9892)

> Finally, **Bjornaer magi cannot bind familiars.** The reasons for this are debated, but the fact is uncontroversial. (ArMDE:9898)

**Current data:** `"classification": "creation_effect"`,
`"prerequisites": {"kind":"house","value":"house.bjornaer"}`,
`"effects": [{"type":"ability_score_grant","ability":"ability.heartbeast","amount":1}]`,
no `description` in either locale.

**What is correct.** The `ability_score_grant` of 1 matches ":4061 You start
with the Ability Heartbeast 1" and matches the House table row at ArMDE:2272
("beginning score of 1 in Heartbeast Ability"). `house.bjornaer` grants this
Virtue (`rules/core/houses.json`), so the free-at-creation clause is modelled.
The `house` prerequisite inverts the passage's causality — the book says the
Virtue *makes* you Bjornaer, not that being Bjornaer lets you take it — but the
two are equivalent in extension and the prereq is not marked wrong.
`ability.heartbeast` is `arcane` in `rules/core/abilities.json`, which matches
ArMDE:7231 listing it under `#### Arcane Abilities`, and A9 records that the
grant is itself permission to own the Ability, so the gate is satisfied.

**Why it is a finding.** **"Bjornaer magi cannot bind familiars" is a hard
prohibition on a subsystem this engine models.** `derived/familiar.rs` computes
a familiar read-out, `Entity` carries a familiar, and
`grep -rn "bjornaer" crates/arm-rules/src rules/core ui/src` returns **no
production hit at all** — only `migration.rs` (an unrelated legacy-id rewrite)
and `types.rs`'s `#[cfg(test)]` prereq tests. So a Bjornaer magus can be given a
familiar in this app with nothing objecting, and the entry's text — in either
locale — never tells the player he may not. The –15 in-form casting penalty and
the Initiation restriction on the Ability are lost the same way.

**Correct value:** a `description` in both locales carrying at least the
no-familiars prohibition and the –15 in-form casting penalty (ASCII hyphen, per
the project rule — the English source writes an en dash at :9892 and the German
at :4023 uses one too). **This is a `description` on a `creation_effect`**, so
it depends on **Q-14 / Q-22**. Whether the no-familiars rule should additionally
become a validator is a separate, larger question and is not proposed here.

**Severity:** lost-rule / provenance, pending Q-14/Q-22 — with the familiar
prohibition being the sharpest instance the audit has found of a cross-reference
carrying a *prohibition* rather than a bonus

---

### F-105 — `virtue.hermetic_experience` — the restricted pool funds any Dead Language and any Organization Lore, where the book names two specific instances

**Passage** (ArMDE:4065, verbatim, English — the relevant sentences):
> In any case, you have an additional 50 experience points to spend on Order of Hermes Lore, Magic Lore, or Latin. You cannot spend other experience points on Magic Lore or Latin unless the character has another Virtue or Flaw permitting this.

German, ArMDE:4065 (line-parallel):
> In jedem Fall hast du zusätzliche 50 Erfahrungspunkte, die du für Ordenskunde, Magiekunde oder Latein ausgeben kannst. Du kannst keine anderen Erfahrungspunkte für Magiekunde oder Latein ausgeben, es sei denn, der Charakter hat eine weitere Tugend oder einen Fehler, der dies erlaubt.

**Current data:**
```json
"effects": [{
  "type": "restricted_ability_xp",
  "amount": 50,
  "abilities": ["ability.dead_language", "ability.magic_lore", "ability.organization_lore"]
}]
```

**Why it is wrong — two defects in one row.**

1. **Instance scope is lost, twice.** The book names **Latin** and **Order of
   Hermes Lore**, two specific instances. `ability.dead_language` is
   parameterized on `language` and `ability.organization_lore` is parameterized
   on `organization` (`rules/core/abilities.json`), and A7 is explicit that
   `restricted_ability_xp` "cannot express an instance-scoped pool" —
   `restricted_ability_xp_pools` hard-codes `instances: Vec::new()`. So the 50
   points will happily fund Dead Language (**Greek**) and Organization Lore
   (**the Cistercian Order**), neither of which the Virtue grants. This is
   B03's F-73 / F-74 shape (`virtue.falconer`, `virtue.forge_companion`) on two
   Abilities at once, and it is the more expensive variant: Hermetic Experience
   is 50 points, not a score.

2. **The second sentence is a restriction the entry cannot state and does not.**
   "You cannot spend **other** experience points on Magic Lore or Latin" caps
   these two Abilities at whatever the 50-point pool buys. But A7 records the
   pool's **permission side-effect**: `ability_authorizations` treats a
   restricted pool as evidence the character may *own* what it funds, and once
   `magic_lore` (arcane) and `dead_language` (academic) are authorized, the
   general `Entity::xp_pool` funds them without limit —
   `effective/xp.rs::build_flow_pools` opens the general pool in phase 2 with no
   category filter. So the engine permits precisely what :4065 forbids. That is
   a catalogue-wide consequence of A7's design, not of this entry's data, and it
   is raised as **Q-27**.

**Correct value:** for (1), the pool needs instance scoping — the shape exists
(`PoolEligibility::Ability.instances`) but `restricted_ability_xp` has no field
to carry it, so this is an engine change, and until then the two instance names
must at least reach the text. For (2), see Q-27. The second sentence of :4065
reaches neither locale today; carrying it in a `description` is the minimum, and
is again a `description` on a `creation_effect` (**Q-14 / Q-22**).

**Severity:** wrong rules output (the pool funds Abilities the Virtue does not
grant) — an over-permissive failure, which is the direction that silently
produces illegal characters

---

### F-106 — `virtue.hermetic_magus` — `narrative` on two rules, one of which the prereq encodes wrongly

**Passage** (ArMDE:4069, verbatim, English — the entry's whole body):
> You are a member of the Order of Hermes. All magi must take this as their Social Status, and only magi may take it.

German, ArMDE:4069 (line-parallel):
> Du bist Mitglied des Ordens des Hermes. Alle Magi müssen dies als ihren Sozialen Status nehmen, und nur Magi dürfen es nehmen.

**Current data:** `"classification": "narrative"`,
`"prerequisites": {"kind":"has","value":"virtue.the_gift"}`, no `effects`,
`magnitude: "free"`, `categories: ["social_status"]`.

**Why it is wrong — two separate problems.**

1. **The prereq encodes the wrong condition.** The book says "**only magi** may
   take it". The data says "has The Gift". Those are not the same set: the
   companion profile has `gift_policy: "allowed"`, so a Gifted companion — the
   book's own recommended case at ArMDE:2876 ("companions should only have The
   Gift if they are intended to become magi") — satisfies `has
   virtue.the_gift` and can take Hermetic Magus with nothing objecting, while
   being `is_magus: false`. The engine has the exact variant for this:
   `Prereq::IsMagus`, which reads `profile.is_magus` and returns False for every
   non-magus profile. The correct prereq is
   `{"kind":"is_magus"}`; `has the_gift` is at best a proxy and at worst admits
   the population the sentence excludes.

2. **`narrative` is wrong.** The passage states two requirements and no fiction
   beyond its first six words. The forward half ("All magi must take this") *is*
   computed — the magus profile carries `required_traits:
   ["virtue.hermetic_magus"]` — and the reverse half is the prereq above. An
   entry whose whole body is two enforceable rules is not "pure story/colour".

**Correct value:** `"prerequisites": {"kind":"is_magus"}`. The classification is
**Q-23**'s question in miniature and is left `?`: the rules are enforced, but on
the profile and the prereq rather than by an `Effect`, which is the same
`creation_effect`-with-no-effects shape C7 records for `virtue.devil_child`.

Note also: this is one of five Social Status entries in the batch and so one of
five instances of the ArMDE:2816 "one Social Status" rule — listed under
"Recurring patterns", not re-argued here.

**Severity:** wrong rules output (an ineligible character validates clean)

---

### F-107 — `virtue.homing_instinct` — `narrative` on a formula, an Ease Factor and a fixed Arcane Connection

**Passage** (ArMDE:4081-4083, verbatim, English):
> The character always knows precisely how to get from where she is to a number of locations **equal to** her Intelligence Score (a minimum of 1). To add a location to the character's repertoire she must be at that location, the location must be open to the air (so it cannot be inside, for example), and the player must make an Intelligence + Concentration Roll against an **Ease Factor of 6**. If this would increase the number of known locations beyond the character's limit, then another location must be "forgotten." (:4081)

> In addition, the character has a fixed Arcane Connection to locations that she knows, which may be exploited in the usual ways (page 219). This Arcane Connection is an artifact of the mind, and so may not be given to another character. (:4083)

German, ArMDE:4081 (line-parallel):
> Der Charakter weiß stets genau, wie er von seinem aktuellen Standort zu einer Anzahl von Orten gelangt, die seinem Intelligenzwert entspricht (mindestens jedoch 1). … der Spieler muss einen Intelligenz + Konzentrations-Wurf gegen einen Schwierigkeitsgrad von 6 schaffen. …

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** This entry trips **four** of `MECHANICAL_PHRASES` outright
and would have been caught the moment the screen reached line 4079: "equal to"
(:4081), "entspricht" (:4081 DE), "at least"/"mindestens" (:4081, "a minimum of
1" / "mindestens jedoch 1"), and "Ease Factor"/"Schwierigkeitsgrad" (:4081). A
capacity formula keyed on a Characteristic, a target number of 6, and a standing
Arcane Connection are three rules, not colour.

**Correct value:** `"classification": "uncomputed_rule"`, with the
Intelligence-keyed location count (minimum 1), the Int + Concentration roll
against Ease Factor 6, the open-to-the-air condition, the forget-one rule and
the non-transferable Arcane Connection in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-108 — `virtue.homing_instinct` — the German name contradicts the canonical translation table

**Current data:** `rules/i18n/de/virtues_flaws.json` →
`"virtue.homing_instinct": { "name": "Heimfindungsinstinkt", … }`.

**The table** (`rules/source/de/translation-tables/tugenden-fehler.md:126`):
> `| Homing Instinct | Ortsgespür | SdM:M; kennt Weg zu Int-Wert vielen Orten; feste Arkane Verbindung |`

**Why it is wrong.** CLAUDE.md states the rule without exception: "when
generating `rules/i18n/de/` text, the German label for any term whose English
form appears in a table MUST match the table's `Deutsch (DE)` value." The
German rulebook writes "Heimfindungsinstinkt" at ArMDE:4079, and the shipped
label follows the rulebook rather than the table. This is exactly the conflict
`rules/source/de/translation-tables/README.md:53-58` already adjudicated for
`Inoffensive to (Beings)` — same shape, same `SdM:M` marker on the table row,
same resolution: "**Kanonisch ist die Tabelle**", and the German source files
are deliberately **not** adjusted because they reproduce the books as printed.

This is B03's F-85 (`virtue.fabric_ripper`) recurring, and the README's
adjudication makes it a settled rule rather than an open reading — which is why
it is a finding and not a question.

**Correct value:** `"name": "Ortsgespür"`.

**Severity:** localization defect

---

### F-109 — `virtue.imbued_with_the_spirit_of_form` — `narrative` on a one-for-one vis substitution

**Passage** (ArMDE:4089-4091, verbatim, English):
> Whenever casting a spell or performing a Laboratory Activity that requires vis of the chosen Form, the magus may substitute Long-Term Fatigue Levels for the vis. **Each Long-Term Fatigue Level lost reduces the vis requirement of the spell or effect by 1.** When casting a spell the Fatigue Levels are lost after the spell is cast. For Laboratory Activities the Fatigue loss lasts for the duration of the Laboratory process (that is, **at least** a season), and Fatigue Levels substituted for vis do not count towards the magi's limit of vis expenditure in a season. **The magus takes the Fatigue Penalty to his Lab Total.** (:4089)

> Note that studying vis is not a Laboratory activity, so the magus may not substitute Fatigue Levels for vis when studying. (:4091)

German, ArMDE:4089 (line-parallel):
> … Jede verlorene langfristige Erschöpfungsstufe verringert den Vis-Bedarf des Zaubers oder Effekts um 1. … (also mindestens ein Quartal) … Der Magus erleidet den Erschöpfungsabzug auf seine Laborsumme.

**Current data:** `"classification": "narrative"`,
`"parameters": [{"key":"form","type":"ref","domain":"form"}]`, no `effects`.

**Why it is wrong.** A one-for-one exchange rate ("reduces the vis requirement …
by 1" per Fatigue Level) is a formula; "at least a season" trips
`MECHANICAL_PHRASES` in both languages ("at least" / "mindestens"); "the magus
takes the Fatigue Penalty to his Lab Total" names a specific modifier applied to
a total the engine computes (`derived/lab.rs::lab_totals`); and the :4093
worked example spells the arithmetic out (6 pawns, 3 Fatigue Levels, 3 pawns
remaining). This is `virtue.folk_magic` all over again — the entry
`ecb5150` calls "the loudest case: `narrative` while carrying … parameters and
one of the most mechanical passages in the Virtues block". The parameter is not
the problem; the classification is.

The `form` parameter itself is **correct** and correctly unconsumed: B8 records
that a parameter no effect reads is the normal shape for a recorded player
choice, and this one is required ("vis of the **chosen** Form").

**Correct value:** `"classification": "uncomputed_rule"`, with the
Fatigue-for-vis exchange rate, the season-long duration, the
vis-expenditure-limit exemption, the Lab Total Fatigue Penalty and the
vis-study exclusion in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-110 — `virtue.immune_to_disease` — `narrative` on an immunity

**Passage** (ArMDE:4097, verbatim, English — the entry's whole body):
> The character is marked as the property of a very powerful demon, and the lesser demons that cause most diseases refuse to harm him. Even characters who turn against their masters retain this protection, because the mark is indelible, and disease demons are not usually very bright. A few diseases, for example, those sent as scourges by God, affect the character normally.

German, ArMDE:4097 (line-parallel):
> Der Charakter ist als Eigentum eines sehr mächtigen Dämons gekennzeichnet, und die niederen Dämonen, die die meisten Krankheiten verursachen, weigern sich, ihm Schaden zuzufügen. … Einige Krankheiten – zum Beispiel solche, die Gott als Geißel schickt – betreffen den Charakter normal.

**Current data:** `"classification": "narrative"`, `"tainted": true`, no
`effects`.

**Why it is wrong.** The Virtue's entire content is an immunity — to most
disease — with a stated exception. The fiction ("marked as the property of a
very powerful demon", "disease demons are not usually very bright") is the
*explanation* of the rule, not a substitute for it: strip the fiction and "the
character does not contract most diseases; divinely-sent ones affect him
normally" remains, which is a rule. `ecb5150` names the immunity case in its own
title.

`"tainted": true` is **correct** — the descriptor at ArMDE:4096 reads `*Major,
Supernatural, Tainted*`, German :4096 `*Groß, Übernatürlich, Befleckt*`, and
`Befleckt` is the table-mandated rendering of the `Tainted` type label
(`translation-tables/README.md:44-46`).

**Correct value:** `"classification": "uncomputed_rule"`, with the disease
immunity and its divine exception in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-111 — `virtue.immunity_to_cold` — `narrative` on an immunity, with the exception stated as sharply as the rule

**Passage** (ArMDE:4101, verbatim, English — the entry's whole body):
> Normal cold does not harm you, nor does it make you feel uncomfortable. You do not need warm environments to remain healthy. Extreme, magically created cold, such as the effect of Wizard's Icy Grip (page 346), still affects you normally.

German, ArMDE:4101 (line-parallel):
> Normale Kälte schadet dir nicht, noch bereitet sie dir Unbehagen. Du benötigst keine warme Umgebung, um gesund zu bleiben. Extreme, magisch erzeugte Kälte, wie etwa der Effekt von Eisiger Griff des Zauberers ([Seite 346](#eisiger-griff-des-zauberers)), betrifft dich jedoch weiterhin normal.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** Same reasoning as F-110, and this one is if anything
cleaner: the passage is three sentences, all three of which are rules
statements (an immunity, a consequence of the immunity, and a named exception
with a spell cited by page). Nothing here is characterisation.

Note the contrast with `virtue.greater_immunity` (F-95), which says the
opposite about magical versions — ":4011 This immunity applies to mundane **and
magical** versions of the thing." The two entries state deliberately different
scopes, which is precisely why both need their text carried rather than
paraphrased into a generic "you are immune".

**Correct value:** `"classification": "uncomputed_rule"`, with the normal-cold
immunity and the magical-cold exception in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-112 — `virtue.indescribable_face` — the book requires the player to choose a form, and nothing records it

**Passage** (ArMDE:4109, verbatim, English — the final sentence of the first
paragraph):
> A player who selects this Virtue for his character needs to select which form of the Virtue his character has.

German, ArMDE:4109 (line-parallel):
> Ein Spieler, der diese Tugend für seinen Charakter wählt, muss festlegen, welche Form der Tugend sein Charakter hat.

The two forms are then given their own paragraphs, ArMDE:4111 ("A character who
is simply average is always forgettable. He **can't turn the ability off** …")
and ArMDE:4113 ("A character who distracts with a prop is memorable **when she
wishes to be** … this protection is **less perfect** than a truly forgettable
face").

**Current data:** no `parameters`, `"classification": "narrative"`.

**Why it is wrong.** "needs to select" / "muss festlegen" is the book demanding
a choice in the imperative, and the two branches are not cosmetic: one cannot be
switched off and is the more reliable, the other can and is the less reliable.
A save file cannot distinguish the two characters. This is check 9 failed on the
clearest possible wording.

The shape to use is already in the catalogue: an `enumerated` domain with the
two values, exactly as `virtue.inoffensive_to_beings` in the same batch uses one
for its five classes of beings and `virtue.folk_magic` for its four spell
categories. `ParameterPicker.svelte` maps each id through the rules i18n layer,
so the two options get proper localized labels rather than raw slugs.

**Correct value:** add
`{"key":"form","type":"ref","domain":"enumerated","values":["indescribable_face.average","indescribable_face.prop"]}`
(ids illustrative), with `name` entries for both in both locales'
`virtues_flaws.json`. `classification` is **Q-26** — the branches differ in
reliability but the book attaches no number to either.

**Severity:** lost player choice (silent data loss)

---

### F-113 — `virtue.independent_study` — the German name contradicts the canonical translation table

**Current data:** `rules/i18n/de/virtues_flaws.json` →
`"virtue.independent_study": { "name": "Unabhängiges Studium", … }`.

**The table** (`rules/source/de/translation-tables/grundbegriffe.md:342`):
> `| Independent Study | Eigenständiges Studium | Kleine Allgemeine Tugend |`

**Why it is wrong.** Same rule and same adjudicated precedent as F-108: the
table is the canonical EN→DE mapping for `rules/i18n/de/` labels, the German
rulebook's own wording at ArMDE:4115 ("Unabhängiges Studium") does not override
it, and `translation-tables/README.md:53-58` has already settled that direction
of conflict once. Note the table row here also confirms the entry's
classification data independently — it annotates the term "Kleine Allgemeine
Tugend", matching `magnitude: "minor"`, `categories: ["general"]`.

The rest of the entry is **correct** and is recorded so the finding is not read
wider: `{"source":"practice","amount":2}` and
`{"source":"adventure","amount":3}` match ArMDE:4117 ("When he is studying
through Practice, add two to the Source Quality, and add three to the Source
Quality of his Adventure experience") in source, sign and size, and both point
at the character himself — the direction Good Teacher gets wrong (F-91).

**Correct value:** `"name": "Eigenständiges Studium"`.

**Severity:** localization defect

---

### F-114 — `virtue.infernal_heirloom` — `narrative` on a level number and a use frequency

**Passage** (ArMDE:4131, verbatim, English):
> Most Infernal heirlooms have only a single effect, and they are usually triggered by a minor act of, or intention to, sin. As a guideline, each heirloom may create an effect **once per day** that is equivalent to a Hermetic spell of **level 25**. Items that produce less powerful effects may be used more frequently at the troupe's discretion.

German, ArMDE:4131 (line-parallel):
> Die meisten infernalen Erbstücke haben nur einen einzigen Effekt, und sie werden in der Regel durch eine geringfügige Sünde oder die Absicht zur Sünde ausgelöst. Als Richtlinie gilt: Jedes Erbstück kann einmal pro Tag einen Effekt erzeugen, der einem Hermetischen Zauber der Stufe 25 entspricht. Gegenstände mit weniger mächtigen Effekten dürfen nach Ermessen der Spieltruppe häufiger eingesetzt werden.

**Current data:** `"classification": "narrative"`, `"tainted": true`, no
`effects`.

**Why it is wrong.** A level and a frequency. The German even trips
`MECHANICAL_PHRASES` outright — "entspricht" (:4131) is on the formula list. The
hedge "As a guideline" / "Als Richtlinie gilt" does not make it fiction: the
book hedges plenty of numbers it still expects to be used, and the sentence
gives a *specific* number to hedge. `narrative` asserts there is no number at
all.

**Correct value:** `"classification": "uncomputed_rule"`, with the level-25
once-per-day guideline and the sin trigger in `description` in both locales.

Consider also `{"type":"item_level_budget", …}` — A19 exists and 25 levels is
exactly the currency it carries. Whether an heirloom is the character's item
budget or an NPC-owned object outside it is a reading this batch does not take;
the text fix is the one proposed.

**Severity:** lost-rule / provenance

---

### F-115 — `virtue.inoffensive_to_beings` — `narrative` on an exemption from The Gift's penalties

**Passage** (ArMDE:4135-4137, verbatim, English):
> This Virtue is associated with one of five classes of beings: animals, divine beings, faeries, demons, or magical creatures. These last four include characters associated with the Divine, Faerie, Infernal, or Magic realm, respectively, through Supernatural Virtues or Flaws, as well as beings with Might. (:4135)

> The character's Gift does not bother beings of this sort, although it still has the normal effects on others. For example, Inoffensive to Animals makes it easier for her to get along with mundane beasts. Animals who react positively to The Gift still react positively to her, since she does have The Gift, but those that do not are not disturbed by her presence. (:4137)

German, ArMDE:4137 (line-parallel):
> Die Gabe des Charakters stört Wesen dieser Art nicht, obwohl sie bei anderen weiterhin die normale Wirkung hat. …

**Current data:** `"classification": "narrative"`,
`"prerequisites": {"kind":"any","value":[{"kind":"has","value":"virtue.the_gift"},{"kind":"has","value":"flaw.magical_air"}]}`,
one `enumerated` parameter with five values, `"max_total": 1`, no `effects`.

**Why it is wrong.** The Gift's penalties are a numbered mechanic (ArMDE:2870
"The character suffers all the penalties of The Gift, just as magi do (see page
203)"). A Virtue that switches those penalties off against a named class of
beings is a rule about a rule — the same category as B03's `virtue.gentle_gift`
(F-81), which this very passage names as the wider-scope alternative
(":4139 characters who are Inoffensive to more than one type of being should
take Gentle Gift instead"). B03 moved Gentle Gift `narrative → uncomputed_rule`
under D3; the narrower Virtue cannot be narrative while the broader one is a
rule.

**What is correct, and it is most of the entry** — recorded so the finding is
read narrowly:

- The `any(has the_gift, has magical_air)` prereq is an exact encoding of
  ":4139 UnGifted characters may take this Virtue only if they have the Flaw
  Magical Air" — Gifted satisfies the first arm, unGifted-with-Magical-Air the
  second. Tri-state evaluation makes `any` short-circuit True on either.
- `max_total: 1` is an exact encoding of ":4139 You may not take this Virtue
  more than once."
- The five `enumerated` values match ":4135 animals, divine beings, faeries,
  demons, or magical creatures" one for one — and, notably, the catalogue's
  sixth being id `being.mundane_humans` (which `flaw.unbearable_to_beings` does
  use) is **correctly omitted**, matching ":4141 Note that Inoffensive to
  Mundane Humans is not available as a Minor Virtue; take Gentle Gift instead."
  That omission is a positive check, and it passes.

**Correct value:** `"classification": "uncomputed_rule"`, with the Gift-exemption
rule (and the ":4135 these last four include characters associated with the
realm through Supernatural Virtues or Flaws, as well as beings with Might"
scoping clause) in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-116 — `virtue.inoffensive_to_beings` — `hermetic` is a descriptor membership category, filed as `index_categories`

**The descriptor** (ArMDE:4134, verbatim):
> *Minor, General and Hermetic*<br>

German, ArMDE:4134 (line-parallel):
> *Klein, Allgemein und Hermetisch*<br>

**And the book's index lists it twice**: ArMDE:3110 under `### Hermetic, Minor`
(:3087) and ArMDE:3276 under `### General, Minor` (:3239). It is the only entry
in this batch that appears in two groups.

**Current data:** `"categories": ["general"]`, `"index_categories": ["hermetic"]`.

**Why it is wrong.** B5 defines `index_categories` as "**Provenance, not
membership.** It records headings the book's own *index* files the entry under,
**beyond its descriptor's membership categories**." Here `hermetic` is *in* the
descriptor — "General **and** Hermetic" — so it is a membership category by that
definition and `index_categories` is the wrong field for it. The consequence is
concrete: B5 records that every membership surface and `categories_for` are
"deliberately blind to" `index_categories`, whose **only** consumer is
`validation/magus.rs::validate_house`'s ArMDE:2860 Hermetic-Flaw guideline —
which does not apply to a Virtue at all. So the `hermetic` half of this
entry's type currently affects nothing whatsoever, including the UI picker,
which offers it under "General" only.

**But the obvious fix breaks a rule the same passage states, which is why this
carries a question.** Setting `categories: ["general","hermetic"]` would make
the entry unavailable to a grog (grog `forbidden_categories: ["hermetic"]`) and
to an unGifted companion (companion forbids `hermetic` unless
`has virtue.the_gift`) — yet ArMDE:4139 explicitly permits "UnGifted characters
… if they have the Flaw Magical Air". A naive dual membership therefore
contradicts :4139.

The catalogue already has the shape for this: `virtue.sufi` carries
`categories: ["social_status","supernatural"]` plus a `category`-domain
`taken_as` parameter (ArMDE:5083), and B5 records that five membership sites
resolve a multi-category item through `types.rs::PointItem::categories_for`,
which reads that parameter. Whether that is the right model here — the book does
not phrase :4134 as an either/or the way :5083 does — is **Q-30**.

**Correct value:** not `index_categories`. Either
`categories: ["general","hermetic"]` with a `taken_as` parameter (the Sufi
shape), or a documented decision that the book's dual listing is not dual
membership. Pending Q-30.

**Severity:** data/provenance — a declared category that nothing can read

---

### F-117 — `virtue.inspirational` — `narrative` on a +3

**Passage** (ArMDE:4145, verbatim, English — the entry's whole body):
> You are a stirring speaker or a heroic figure, and can urge people to great efforts. You give targets a **+3 bonus** to rolls for appropriate Personality Traits.

German, ArMDE:4145 (line-parallel):
> Du bist ein mitreißender Redner oder eine Heldenfigur und kannst Menschen zu großen Leistungen anspornen. Du gewährst Zielen einen +3-Bonus auf Würfe für angemessene Persönlichkeitseigenschaften.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** An explicit signed modifier, in both languages, on an entry
asserting the book states nothing mechanical. `has_signed_number` — the older
and narrower of the two screens, the one that existed *before* `ecb5150` widened
anything — would catch this on sight. It survives purely because ArMDE:3951-5282
has never been screened.

The bonus goes to **other** characters ("You give **targets** a +3"), so no
effect on this character's own sheet is owed — this is the same
outward-facing direction that makes Good Teacher's `teaching` row right and its
`book` row wrong (F-91), and the engine has no variant for a modifier conferred
on a third party. Per **D3** that is `uncomputed_rule`, not `narrative`.

**Correct value:** `"classification": "uncomputed_rule"`, with the +3 to
targets' Personality Trait rolls in `description` in both locales. ASCII
characters only; the value is positive so no hyphen question arises.

**Severity:** lost-rule / provenance

---

### F-118 — `virtue.intuition` — `narrative` on a named die and a target number

**Passage** (ArMDE:4149, verbatim, English — the entry's whole body):
> You have a natural sensitivity that allows you to make the right decisions more often than luck can account for. Whenever you are given a choice in which luck plays a major role (such as deciding which of three unexplored paths to follow), you have a good chance of choosing correctly. The storyguide should secretly roll a **simple die**. On a **6+**, your intuition kicks in and you make whatever might be considered the "right" decision. Otherwise, you fail to get any flash of insight and must make the decision without aid.

German, ArMDE:4149 (line-parallel):
> … Der Spielleiter würfelt heimlich einen einfachen Würfel. Bei einem Ergebnis von 6 oder mehr setzt deine Intuition ein und du triffst die Entscheidung, die als „richtig" gelten kann. …

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong.** This is the one entry in the batch that trips
`MECHANICAL_PHRASES` on its *exact* listed spelling, in both languages at once:
"simple die" (:4149 EN) and "einfachen Würfel" (:4149 DE) are both in the list,
and the German additionally carries "oder mehr" ("or more"). `ecb5150` widened
the screen specifically so that "a target number ('against an Ease Factor of
15')" would stop reading as flavour; a secret simple die against 6+ is the same
thing.

The procedure is genuinely uncomputable — the storyguide rolls, secretly, about
a fictional choice — which is the textbook `uncomputed_rule`, not a reason to
call it narrative.

**Correct value:** `"classification": "uncomputed_rule"`, with the secret simple
die and the 6+ threshold in `description` in both locales.

**Severity:** lost-rule / provenance

---

### F-119 — `virtue.inventive_genius` — the +6 experimentation alternative reaches neither locale

**Passage** (ArMDE:4153, verbatim, English — the entry's whole body; the full
transcription and its three reading notes are under "Directed reads"):
> Invention comes naturally to you. You get +3 to your Lab Total if you are not using a Laboratory Text or being taught. **If you experiment, you get +6.**

German, ArMDE:4153 (line-parallel):
> Erfinden liegt dir im Blut. Du erhältst +3 auf deine Laborsumme, wenn du keinen Labortext verwendest und auch nicht unterrichtet wirst. Wenn du experimentierst, erhältst du +6.

**Current data:** `"classification": "in_play_effect"`,
`"effects": [{"type":"lab_total_mod","amount":3}]`, no `description`; summaries
are "Invention comes naturally to you." / "Erfinden liegt dir im Blut." — the
first sentence, which carries no number at all.

**What is explicitly NOT a finding.** Per **D1** the +3's condition is
deliberately ignored in `effective/spell.rs::spell_level_cap`; per **D4** the +3
does apply in a character-generation Lab Total, and this entry is the first row
of D4's table. The shipped `lab_total_mod` of 3 is correct on both rulings and
is not marked wrong. Nor is `classification`: B1 records that carrying an
`Effect` is what puts an entry in `in_play_effect`.

**Why it is a finding.** The passage has **two** numbers and the data and the
text between them carry **one**. The +6 is not derivable from the +3 (it
replaces it, it is not additive — see the Directed-reads note), it is not
conditioned on the Lab-Text/teaching clause, and a player reading this Virtue in
the app sees neither number: the summary stops before the first one. So the app
shows a magus a +3 in his Lab Total read-out and tells him nothing about the
doubled bonus for experimenting.

**Correct value:** a `description` in both locales carrying both conditions and
both numbers. The +6 cannot become a second `lab_total_mod` — that would stack
to +9 — and `Effect` has no alternative-value shape, so text is the only route.
This is a `description` on an `in_play_effect` **whose passage outruns its
effects**, which is the *established* practice (9 of 93 `in_play_effect` entries
carry one), not the new one — so unlike F-96, F-104 and F-105 this does **not**
depend on Q-14/Q-22.

**Severity:** lost-rule / provenance

---

### F-120 — `virtue.hermetic_prestige` — the German summary renders the game term *Reputation* as `Ansehen`, which the tables forbid

**Passage** (ArMDE:4073, verbatim, English — the final sentence):
> You gain a **Reputation** of level 4 within the Order.

German, ArMDE:4073 (line-parallel):
> Du erhältst eine **Reputation** der Stufe 4 innerhalb des Ordens.

**Current data:** `rules/i18n/de/virtues_flaws.json` →
```json
"virtue.hermetic_prestige": {
  "name": "Hermetisches Ansehen",
  "summary": "Andere Magi blicken zu dir auf. Du erhältst ein Ansehen der Stufe 4 innerhalb des Ordens."
}
```

**Why it is wrong.** The Virtue's **name** is correctly "Hermetisches Ansehen" —
`translation-tables/tugenden-fehler.md:55` and `reputationen.md:79` both give
it. But the **game term** in the second sentence is a different thing, and the
tables pin it flatly, with an explicit exclusion, and with the word *stets*:

- `rules/source/de/translation-tables/reputationen.md:16` — "**Wichtig:** Der korrekte deutsche Begriff ist **stets** **Reputation** (Pl.: Reputationen) – **nicht** „Ruf"."
- `rules/source/de/translation-tables/reputationen.md:20` — `| Reputation | Reputation | Pl.: Reputationen; **nicht** „Ruf" |`
- `rules/source/de/translation-tables/grundbegriffe.md:102` — the same row, repeated in the master glossary.

**The structural twin settles it.** `virtue.famous` carries the *same* effect
(`grants_reputation`, `score: 4`) and its German summary reads "Du hast eine
gute **Reputation** der Stufe 4." — the only other summary in the file using the
phrase "Reputation der Stufe", and it uses the canonical term. So the catalogue
already has a house style for this exact sentence, and Hermetic Prestige is the
single entry that departs from it.

The German rulebook writes "eine **Reputation** der Stufe 4" at the very line
this summary paraphrases, and the whole German UI agrees — `locales/de/main.ftl`
uses `Reputationen`, `reputations-label = Reputationen`,
`reputation-granted-by = Stufe { $score } von { $source }`. So the app will show
the player "Du erhältst **ein Ansehen** der Stufe 4" next to a panel headed
"Reputationen", for a grant the engine records as a `ReputationType::Hermetic`
slot. The name's rendering has bled into the term.

**This is the only instance.** All three other occurrences of "Ansehen" in
`rules/i18n/de/virtues_flaws.json` are ordinary prose, not the game term
("in schlechtem Ansehen", "von einigem Ansehen", "von gutem Ansehen"), and are
correct.

**Correct value:** `"summary": "Andere Magi blicken zu dir auf. Du erhältst eine
Reputation der Stufe 4 innerhalb des Ordens."` Note the article changes with the
noun's gender — *Reputation* is feminine ("eine"), *Ansehen* is neuter ("ein") —
so the existing "ein" cannot simply be left in place.

**What is otherwise correct on this entry**, since this finding is narrow:
`{"type":"grants_reputation","kind":"hermetic","score":4}` matches ArMDE:4073 in
type and level; `ReputationType::Hermetic` exists and is documented as "Known
within the Order of Hermes"; `magnitude: "minor"` / `categories: ["hermetic"]`
match the descriptor at :4072 and the index at :3108; the entry is also House
Guernicus's granted benefit (ArMDE:2277, `rules/core/houses.json`), which the
`creation_effect` classification and the grant machinery handle. That the
grant's `score` is not enforced is Phase 0's open question 3 and is not
re-reported here.

**Severity:** localization defect

---

### F-121 — `virtue.greater_benediction` — `narrative` on a cited range that contains five numbers

*(Raised by the independent verification pass, which overturned this batch's
first-pass verdict of "open question only". Accepted after re-checking every
claim against the sources — see "Sub-agent reconciliation".)*

**Passage.** The entry's own body is one line, ArMDE:3993, verbatim:
> You have been blessed by some supernatural power. The effects of the benediction should be comparable to other **Major** Virtues. (See insert for examples.)

The cited range then continues through a block quote, ArMDE:3995-4007:
> ##### Flight … Every time you take to the air, you lose a Long-Term Fatigue Level, and can remain airborne for a maximum of an hour, traveling **up to** fifty miles in this hour. (:3999)

> ##### True Sight … This power has a **Penetration of 20**. (:4003)

> ##### Universally Liked … You receive a **+3 bonus** to all social rolls with people who have known you more than one month. Anyone who tries to act against you by swaying the emotions or opinions of others has **+3** added to all **Ease Factors**. This effect has a **Penetration of 0**. (:4007)

German, ArMDE:3999 / :4003 / :4007 (line-parallel): "eine Langzeit-Erschöpfungsstufe",
"maximal eine Stunde", "**bis zu** fünfzig Meilen"; "Penetration von 20";
"+3 Bonus auf alle sozialen Würfe", "+3 auf alle **Schwierigkeitsgrade**",
"Penetration von 0".

**Current data:** `"classification": "narrative"`, `"source": {"lines": [3991, 4008]}`,
no `effects`, no `parameters`, no `description` in either locale.

**Why the first pass was wrong to leave this as a question, and the verification
pass right to call it a defect.** I had treated "does a boxed insert inside the
cited range count as the entry's own passage?" as unsettled (Q-24) and therefore
taken no verdict. That was the wrong shape of answer: **whichever way that
question is decided, the entry as authored is internally contradictory today**,
because `classification` and `source.lines` make incompatible claims about the
same bytes. The decisive evidence is mechanical, not interpretive:

- The guard reads exactly the bracketed range —
  `uncomputed_clauses.rs::bracketed_passage(file, start, end)` slices
  `lines[start-1..end]` and hands that string to `states_a_mechanical_rule`.
- That string contains **"up to"** (:3999), which is in `MECHANICAL_PHRASES`,
  and **"+3"** (:4007), which `has_signed_number` catches on its own.
- There is **no** `NO_RULE_DESPITE_TOKEN` row for this id.
- So `no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` goes **red**
  on this entry the moment `SWEPT_BLOCKS` widens past ArMDE:3950 — which is the
  stated intention of the comment in that very array.

And the range is not obviously wrong: the repo's convention is
heading-to-before-next-heading, the neighbouring `virtue.greater_immunity`
(`[4009,4016]`) and `virtue.greater_power` (`[4017,4026]`) both swallow all
their sub-paragraphs, and the body's own final words — "(See insert for
examples.)" — point *into* the insert, making it the entry's rules text by the
book's own cross-reference. Even narrowed to `[3991,3993]`, ":3993 should be
comparable to other **Major** Virtues" is a magnitude-calibration instruction,
not colour.

**Correct value:** `"classification": "uncomputed_rule"`, with the three worked
benedictions' rules written into `description` in both locales. The engine
cannot express a player-defined open-ended blessing, which is exactly what
`uncomputed_rule` asserts (**D3**). What remains genuinely open is only the
narrower question of whether the *right* fix is to reclassify or to renarrow
`source.lines` — that is **Q-24**, restated below.

**Twin, in B05:** `virtue.lesser_benediction` (`[4253,4274]`, also `narrative`,
also no effects) has the identical shape — its insert states "anyone attempting
to detect untruth in your words receives a **-3 penalty** to their rolls"
(ArMDE:4261) and "you can therefore get **half as much again** in terms of
yield" (ArMDE:4265). Verified in the source for this batch so B05 does not have
to rediscover it.

**Severity:** lost-rule / provenance

---

**How F-120 was found, recorded because it bears on how this batch should be
trusted.** `virtue.hermetic_prestige` was marked fully clean in this batch's
first pass — its English text is faithful, its effect is right, its magnitude
and category check out twice. The defect surfaced only on a second, deliberate
sweep of every German *game term* in the batch against
`rules/source/de/translation-tables/`, which is B03's F-88 lesson
(`eye_of_hephaestus` rendering *Awareness* as "Wahrnehmungs-Fertigkeit" where
`rules/i18n/de/abilities.json` names it "Aufmerksamkeit") applied as a
systematic check rather than an incidental one. Together with F-108 and F-113,
**three of this batch's thirty findings are German terminology contradicting the
canonical tables**, and none of them is visible from the English side or from
the German rulebook alone — the rulebook agrees with the table in all three
cases; it is the shipped i18n that diverges.

---

## Open questions

### Q-23 — Does modelling a rule on the *type profile* rather than on the Virtue make the Virtue a `creation_effect`?

**Passages.** `virtue.the_gift` ArMDE:3969 ("See earlier, page 63, for full
details") pointing at ArMDE:2870-2876; `virtue.hermetic_magus` ArMDE:4069
("All magi must take this as their Social Status, and only magi may take it").

**The two competing readings.**

- **`creation_effect`.** C7 records that `virtue.devil_child` is a shipped
  `creation_effect` carrying **no** `effects`, precisely because "Devil Child's
  budget bonus lives on the Mythic Companion type profile, not on the Virtue".
  The Gift is the same: grog exclusion, magus requirement, the conditional
  `hermetic` permission and the free Supernatural Ability slot are all real
  creation-time computations keyed on this id, just held elsewhere. By the
  Devil Child precedent, the engine *does* compute this at creation.
- **`uncomputed_rule`.** B1's table says `creation_effect` means "Changes a
  character-creation number or state", and the entry itself changes nothing —
  the profile does. Meanwhile ArMDE:2870's "suffers all the penalties of The
  Gift" is genuinely uncomputed, and `uncomputed_rule` would oblige the
  :2870-2876 rules into `description`, which is the outcome the audit actually
  wants.

**What I could not settle it with.** B1 states the `uncomputed_rule` /
`in_play_effect` line explicitly ("carrying an `Effect` is what decides it") but
states **no** rule for `creation_effect` versus the other two on an entry with
no effects, and C7 flags the same asymmetry in the guard
(`every_vf_is_classified` requires effects on `in_play_effect` and not on
`creation_effect`). The existing five effect-less `creation_effect` entries are
listed in `README.md` as "a lead, not a conclusion", so they are not a
precedent I can lean on without ruling on them too.

**Scope.** This decides `virtue.the_gift` and `virtue.hermetic_magus` in this
batch, and it will recur for every Virtue whose mechanic lives on a profile.

---

### Q-24 — For `virtue.greater_benediction`, is the fix to reclassify or to renarrow `source.lines`?

**Narrowed by the verification pass.** The original form of this question asked
whether a boxed insert counts as the entry's passage at all, and left the entry
unrated. That was too weak: **F-121** establishes that the entry is
self-contradictory as authored whichever way the general question goes, so a
defect is recorded. What is still open is only *which field* to change.

**Passage.** `virtue.greater_benediction`, ArMDE:3991-4008. The entry's own body
is one line:

> You have been blessed by some supernatural power. The effects of the benediction should be comparable to other Major Virtues. (See insert for examples.) (:3993)

The cited range then runs through a block quote, `> #### Greater Benediction
Examples` (:3995), containing three worked examples with hard numbers:

> ##### Flight … Every time you take to the air, you lose a Long-Term Fatigue Level, and can remain airborne for a maximum of an hour, traveling **up to** fifty miles in this hour. (:3999)

> ##### True Sight … This power has a **Penetration of 20**. (:4003)

> ##### Universally Liked … You receive a **+3 bonus** to all social rolls with people who have known you more than one month. Anyone who tries to act against you by swaying the emotions or opinions of others has **+3** added to all **Ease Factors**. This effect has a Penetration of 0. (:4007)

German, ArMDE:4007 (line-parallel): "Du erhältst +3 Bonus auf alle sozialen
Würfe … erhält +3 auf alle Schwierigkeitsgrade. Dieser Effekt hat eine
Penetration von 0."

**The two competing readings.**

- **`narrative` is right.** The Virtue itself states no mechanic — it says the
  effect "should be comparable to other Major Virtues", which is a magnitude
  guideline and troupe judgement. The numbers belong to three *example*
  benedictions, none of which this Virtue grants; a character with Greater
  Benediction may have none of them.
- **`narrative` is wrong.** `source.lines` brackets :3991-4008, so the insert
  **is** the cited passage, and the guard that checks a `narrative` entry's
  passage for a mechanical token reads exactly those lines. "up to",
  "Ease Factor" and "+3" are all in range. On the audit's own rule —
  classification is a claim about *the rulebook passage* — the passage has
  numbers.

**What I could not settle it with.** The audit's rules say classification is a
claim about the bracketed passage, and give no carve-out for a worked example.
But applying that literally would reclassify every entry whose citation happens
to swallow an example box, which is a policy question, not a reading of Ars
Magica. Note the same question will recur on `virtue.lesser_benediction`
(ArMDE:4253-4274, also `narrative`, also an example insert, and its insert
verified in this batch to state "-3 penalty" at :4261 and "half as much again"
at :4265) in **B05**, so it is worth settling before that batch runs.

**The recommendation, if a default is wanted:** reclassify rather than renarrow.
Renarrowing would remove the insert's rules from the audit's reach entirely,
which is the direction that loses rules silently — and the boxed content is
where the only usable description of a Greater Benediction lives, so a player
who never sees it has nothing to build the Virtue from.

**Related, and reported as data regardless of the answer.** Greater Benediction
carries **no parameter** for the chosen benediction, while ":3993 (See insert
for examples.)" and the Virtue's whole premise require one to be defined. That
is the same gap as F-98 and is folded into **Q-31** rather than raised as a
separate finding, because unlike Greater Purifying Touch the book does not
say "you must choose" in so many words here.

---

### Q-25 — Is "politically Criamon, created by another House's rules" a mechanical rule?

**Passage.** `virtue.guest_of_house_criamon`, ArMDE:4039, verbatim:

> Magi with this Virtue are, politically, members of House Criamon, but may be created using the rules for any other House. Guests are offered membership, which Criamon see as a political formality, for many reasons. The troupe and player should determine why the character found it necessary to find sanctuary with this House.

German, ArMDE:4039 (line-parallel):

> Magi mit dieser Tugend sind politisch gesehen Mitglieder des Hauses Criamon, können jedoch nach den Regeln für jedes andere Haus erschaffen werden. …

**The two competing readings.**

- **Narrative.** The sentence does not *change* anything the engine does: the
  magus picks a House normally, gets that House's benefit normally, and the
  Criamon membership is fiction with political consequences the app has no model
  of. Nothing is added, capped or forbidden.
- **A rule.** `Entity` has exactly **one** house field, and this Virtue asserts
  a character is a member of one House while using another's creation rules —
  a state the data model structurally cannot represent. Per **D3**, an engine
  that cannot express a rule is grounds for `uncomputed_rule`, never for
  `narrative`; and the "may be created using the rules for any other House"
  clause is an explicit permission about the character-creation procedure,
  which is the same category as the Academic-Abilities permissions that are
  findings elsewhere in this batch.

**What I could not settle it with.** Whether "may be created using the rules for
any other House" *grants* anything depends on whether the default is otherwise
restrictive, and I found no passage saying a Criamon-affiliated magus would
normally be forced into Criamon's creation rules — ArMDE:2264 says only "A magus
can only be a member of one House." Without that, the clause may be stating a
non-restriction, which B02 already cleared as narrative on "The Wealthy Virtue
and Poor Flaw affect you normally". I could not find the restriction it is
lifting, and I will not guess.

---

### Q-26 — What classification, and what encoding, for a rule that nullifies another Virtue's effect without forbidding it?

**Passages.** Two entries in this batch, different shapes of the same problem.

`virtue.guild_apprentice`, ArMDE:4043:
> The character is not able to benefit from either the Poor Flaw or the Wealthy Virtue, since he is essentially the property of his master, until he moves to the journeyman rank.

`virtue.indescribable_face`, ArMDE:4109-4113: two named forms that differ only
in reliability, with no number attached to either.

**The competing readings, for Guild Apprentice.** `incompatible_with:
["virtue.wealthy","flaw.poor"]` would forbid the combination — but the book does
not forbid it, it *neutralises* it, and forbidding changes the V/F point balance
(a Guild Apprentice could still legitimately be Poor and simply get nothing for
it, keeping the 1 point). `Effect` has no suppression variant, and B7 records
that `incompatible_with` must be symmetric, so encoding it would also force
`virtue.wealthy` and `flaw.poor` to name Guild Apprentice back — polluting two
heavily-used entries with a rule that belongs to one. The alternative is text
only.

**The competing readings, for Indescribable Face.** If a required choice with no
number is enough to make an entry non-narrative, then this is `uncomputed_rule`;
if the test is whether the *book* states a mechanic, the two branches state only
"more perfect" and "less perfect", which is prose. The parameter gap is a
finding either way (**F-112**); only the classification is open.

**What I could not settle it with.** Both hinge on the same unstated policy: does
"the book states a mechanical clause" include a rule that only *removes* or
*qualifies* a mechanic without naming a number? Nothing in `README.md`,
`decisions.md` or B1 answers it, and `ecb5150`'s six shapes (cap, target number,
formula, rounding, absolute, magnitude step) do not obviously cover either case.
Guild Apprentice is reported as a finding (F-100) because the nullified thing
*is* a number — Wealthy's and Poor's `later_life_xp_rate`; Indescribable Face is
left `?`.

---

### Q-27 — A `restricted_ability_xp` pool authorizes the Ability permanently; ArMDE:4065 says it should not

**Passage.** `virtue.hermetic_experience`, ArMDE:4065, second sentence, verbatim:

> You cannot spend other experience points on Magic Lore or Latin unless the character has another Virtue or Flaw permitting this.

German, ArMDE:4065 (line-parallel):

> Du kannst keine anderen Erfahrungspunkte für Magiekunde oder Latein ausgeben, es sei denn, der Charakter hat eine weitere Tugend oder einen Fehler, der dies erlaubt.

**The mechanism.** A7 records that `restricted_ability_xp` has a **permission
side-effect**: `effective/xp.rs::ability_authorizations` folds the pool's
`abilities` list into the authorization set, and A7 calls this "load-bearing:
adding `ability_authorization` alongside it would be redundant". The flow solve
then runs two phases (`two_phase_max_flow`) — restricted pools drain first, then
the **general** `Entity::xp_pool` opens on the same residuals with no category
filter. So once Hermetic Experience authorizes `ability.magic_lore` (arcane) and
`ability.dead_language` (academic), a grog may pour his whole general pool into
Magic Lore and nothing objects, which is exactly what :4065 forbids.

**The competing readings.**

- **The engine is right and :4065 is unenforceable.** There is no "authorized up
  to N points" shape, and building one is a real subsystem. Surfaced-as-text is
  the honest answer (D3), and the entry gets a `description`.
- **The engine is wrong and it is over-permissive.** The whole purpose of
  `categories_requiring_virtue` is to stop a grog buying Arcane Abilities; a
  Virtue that funds 50 points of Magic Lore should not silently also unlock
  unlimited Magic Lore. This fails in the permissive direction, which is the
  direction that produces illegal characters that validate clean.

**Scope, and why this is not a per-entry finding.** It is catalogue-wide:
`restricted_ability_xp` has **28** uses. `virtue.ineslemen` in this same batch
has the identical shape (ArMDE:4125 "He may purchase Theology: Islam, Islamic
Law and (Realm) Lore during character generation **and** gains an additional 50
XP") — but crucially Ineslemen's sentence *does* grant free-standing permission
("may purchase"), so for Ineslemen the side-effect is correct and no
`ability_authorization` row is owed. Hermetic Experience is the case where the
book grants the pool and then explicitly withholds the general permission. That
distinction — "funds and permits" versus "funds but does not permit" — is not
expressible today.

**What I could not settle it with.** Whether A7's side-effect is a deliberate
simplification with a recorded rationale, or an unexamined convenience. A7 calls
it load-bearing but cites no decision, and `crates/arm-rules/RULES.md` was not
consulted on this point in this batch.

---

### Q-28 — Should `virtue.guardian_angel`'s +5 Soak be encoded despite its condition?

**Passage.** ArMDE:4035, quoted in full under F-99. The operative pair:

> First, he can grant you a +5 bonus to Soak. … The angel only grants you these bonuses **if you are acting in accordance with God's will**.

**The competing readings.**

- **Encode it.** A34 records the precedent in the same engine:
  `virtue.berserk` carries a `soak_mod` "applied unconditionally rather than only
  while berserk; the engine has no condition to gate it on". Consistency says
  do the same here, and a number the player can see and mentally gate beats a
  number nobody sees.
- **Do not.** Berserk's condition is a combat state the player declares; "acting
  in accordance with God's will" is a standing moral judgement the storyguide
  makes, so a permanent +5 on the character sheet is wrong far more of the time
  than Berserk's is. And unlike Berserk, the *other half* of this Virtue (Magic
  Resistance 15) is inexpressible anyway, so encoding only the Soak produces a
  half-modelled Virtue whose read-out is more misleading than no read-out.

**What I could not settle it with.** Whether Berserk's `soak_mod` is a
considered precedent or an unexamined one. A34 reports it as a fact without
citing a decision, and I did not find a RULES.md entry arguing it either way.
Either answer needs Norbert, and the D3-minimum (`uncomputed_rule` with both
numbers in the description) is correct under **both** readings, so F-99 stands
regardless.

---

### Q-29 — `greater_immunity` and `lesser_immunity` disagree about repeatability, and one of them is wrong

**Passages.** `virtue.greater_immunity` ArMDE:4015:
> You may take this Virtue more than once, with a different immunity each time.

`virtue.lesser_immunity` is at ArMDE:4275-4278, **in B05, not this batch**, and
its data carries the **default** `max_per_target: 1` and no parameter, while
Greater Immunity carries `max_per_target: 255` and no parameter.

**Why it is raised here.** F-94 argues Greater Immunity's 255 is wrong on its
own terms (no parameter, so 255 means 255 *identical* copies). But the divergence
between the two entries is itself evidence: either both passages permit distinct
repeats and Lesser Immunity's 1 is wrong, or Greater Immunity's 255 is an
unexamined copy of `virtue.greater_power`'s (which is correct for a different
reason). I did not read ArMDE:4275-4278, because it is outside my span and the
audit's rule is source-or-nothing.

**What B05 must do:** read ArMDE:4275-4278 verbatim and reconcile the pair. If
Lesser Immunity also says "with a different immunity each time", the two entries
need the *same* fix — a `text` parameter and `max_per_target: 1` — and the
divergence is two independent mistakes rather than one.

---

### Q-30 — How should a "General **and** Hermetic" descriptor be modelled without blocking the unGifted case?

**Passage.** `virtue.inoffensive_to_beings`, descriptor ArMDE:4134 `*Minor,
General and Hermetic*`; indexed twice, at :3110 (Hermetic, Minor) and :3276
(General, Minor); and ArMDE:4139: "UnGifted characters may take this Virtue only
if they have the Flaw Magical Air."

**The competing readings.** Laid out in full under F-116. In short: plain dual
membership (`categories: ["general","hermetic"]`) is what the descriptor says
and what B5's definition of `index_categories` demands, but it makes the entry
unavailable to a grog and to an unGifted companion, contradicting :4139. The
Sufi `taken_as` shape (`ParameterDomain::Category`) sidesteps that, but Sufi's
passage at ArMDE:5083 phrases its dual nature as an explicit either/or and
:4134 does not.

**What I could not settle it with.** Whether "General and Hermetic" in a
descriptor means "both at once" or "either, as the character's nature dictates".
No passage in the span defines the conjunction, and the only other dual-category
descriptor I could reach from here is Sufi's, which is worded differently.

---

### Q-31 — Three Supernatural Virtues require the player to define their own content and none records it

**Passages, all in this batch.**

- `virtue.greater_benediction` ArMDE:3993: "The effects of the benediction should be comparable to other Major Virtues. (See insert for examples.)"
- `virtue.greater_immunity` ArMDE:4011: "You are completely immune to **one hazard**" + :4015 "with a **different** immunity each time"
- `virtue.greater_power` ArMDE:4019: "this is a **single power**, equivalent to a Formulaic Hermetic spell with a level of 50 or lower"

**Current data:** none of the three carries a parameter. Greater Power is the
partial exception — `Entity::powers` stores the power by free-text name and
level, so the choice *is* recorded, just on the entity rather than on the
selection. Greater Immunity and Greater Benediction record nothing anywhere.

**The question.** Greater Purifying Touch (F-98) is a clean finding because
ArMDE:4029 says "You **must choose** … when you take this Virtue". These three
imply a choice without commanding one. Is an implied choice enough to require a
parameter? A `text` parameter on Greater Immunity is separately justified by the
`max_per_target` argument in F-94 and is proposed there; Greater Benediction's
is not, which is why it sits here rather than as a finding.

**What I could not settle it with.** The audit's check 9 asks "does the book
demand a choice?" — and these three describe one without demanding it in words.
Drawing the line is a policy call. It recurs immediately: `virtue.lesser_power`,
`virtue.lesser_immunity` and `virtue.lesser_benediction` are the same three
shapes at Minor magnitude, all in **B05**.

---

### Q-32 — `AdvancementSource` has `Teaching` for taught-others but nothing for books-written

**Passage.** `virtue.good_teacher`, ArMDE:3973: "Add three to the Quality of any
books that **you write**, and five to the Source Quality for anyone who studies
with you."

**The gap.** `types.rs::AdvancementSource` has nine values. `Teaching` is
documented as "The character teaching others", so the outward-facing direction
*is* represented — for teaching. `Book` is documented as "Learning from a book",
i.e. inward-facing. There is no `Authoring`. F-91's proposed fix is to drop the
mis-directed row and carry the rule as text; adding a tenth variant is the
alternative.

**What I could not settle it with.** Whether any other catalogue entry needs an
authoring source. A grep of the catalogue's `advancement_mod` rows was not run
across all 15 uses in this batch's scope, and `flaw.incomprehensible` (the
`Teaching` twin, `amount: 0`) suggests the book-writing side may have its own
counterpart Flaw elsewhere. If it does, the variant is worth adding; if Good
Teacher is alone, text is the cheaper answer. **B05-B19 should flag any
book-writing modifier they meet.**

---

### Q-33 — The book gives Hermetic Prestige two different levels, and the data picks one

*(Raised by the verification pass; both citations re-checked in the source.)*

**The two passages.** ArMDE:4073, the Virtue's own entry:
> You gain a Reputation of **level 4** within the Order.

ArMDE:2518, the worked example in the Reputations section:
> Darius does have a Reputation, thanks to his Hermetic Prestige Virtue. It's a reputation with Hermetic magi, and it has a **level of 3**. Niall picks 'Dedicated Hoplite' as the content.

**Current data:** `{"type":"grants_reputation","kind":"hermetic","score":4}` —
it follows the Virtue's own entry.

**Why this is recorded rather than fixed.** The data is almost certainly right:
a Virtue's own descriptor outranks a worked example elsewhere in the book, and
the German translation table independently records `4 (+)`
(`translation-tables/reputationen.md:79`). But this is a **book
inconsistency**, not a data one, and it is exactly the shape that invites a
later well-meaning "correction" to 3 by somebody who happens to read :2518
first. Recording it here is the point.

**What I could not settle it with.** Nothing in the source reconciles the two;
Ars Magica errata are not in `rules/source/en/`, so under the audit's
source-or-nothing rule there is nothing further to consult. Note also that the
grant's `score` is not enforced at all (A25, and Phase 0's open question 3), so
today the difference changes no behaviour — it would start to matter the moment
OQ3 is answered "enforce it".

---

### Q-34 — ArMDE:18432 states a Size-scaled Improved Characteristics that nothing models, and nothing can reach

*(Raised by the verification pass; citation re-checked in the source.)*

**Passage** (ArMDE:18432, verbatim, English — a footnote to the creature
Virtues/Flaws list):
> *\* Improved Characteristics: large creatures require more characteristic points, because high value characteristics are increasingly more expensive. A creature of Size +1 or smaller gets 3 characteristic points from this Virtue as normal, a creature of Size +2 gets 6 characteristic points, a creature of size +3 gets 9 characteristic points, and so on.*

**Current data:** `virtue.improved_characteristics` carries a flat
`{"type":"characteristic_points","amount":3}` with no Size scaling, and A8
records that the variant "has no target — it is a whole-character budget
adjustment" with no Size input.

**Why this is a question and not a finding.** The formula is real, sourced, and
unmodelled — but it is scoped to *creatures*, and `EntityKind` is only
`Character | Covenant`. The app cannot build a beast, so the rule is
**unreachable**: no entity this engine can construct has a Size derived from a
creature template rather than from `size_delta`. Marking it a defect would flag
a rule that can never fire; ignoring it silently is how a rule gets lost when
`EntityKind` eventually grows. So it is recorded.

**What I could not settle it with.** Whether the audit's scope covers rules that
are correct-by-unreachability. `README.md` says classification is a claim about
the rulebook and never about engine capability, which argues the rule should at
least be *carried*; but `virtue.improved_characteristics` is a
`creation_effect`, so carrying it means a `description` on a `creation_effect`
— **Q-14 / Q-22** again. Hence: question, not finding. The entry's verdict
stays clean.

---

## Recurring patterns noted, not re-argued

Per the brief, these are catalogue-wide rules already recorded elsewhere. The
instances in this batch are listed so a later pass can count them.

**ArMDE:2816 — "All characters must take one Social Status, and may only take
more than one if the descriptions … explicitly note that they are compatible."**
Five instances in this batch, none of which carries any encoding of the
one-Social-Status rule: `virtue.guild_apprentice`, `virtue.guild_dean`,
`virtue.guild_master`, `virtue.hermetic_magus`, `virtue.ineslemen`. None of the
five descriptions notes compatibility with another Social Status, so on
ArMDE:2816 all five are mutually exclusive and nothing says so.

**ArMDE:2960-2962 — realm association for Supernatural Virtues.** "All
Supernatural Virtues and Flaws are associated with one of the four realms,
Magic, Faerie, Infernal, and Divine. … A Virtue's description notes if it is
limited in this way." **Ten instances** in this batch — every Supernatural entry
in it — and **none** carries a `realm` parameter:
`virtue.greater_benediction`, `virtue.greater_immunity`, `virtue.greater_power`,
`virtue.greater_purifying_touch`, `virtue.hex`, `virtue.homing_instinct`,
`virtue.immune_to_disease`, `virtue.immunity_to_cold`, `virtue.induction`,
`virtue.infernal_heirloom`.

Two of the ten restate the rule in their own passage, which is worth recording
because it makes them the strongest cases:
- `virtue.greater_power` ArMDE:4025: "This Virtue may be associated with any supernatural realm. … The power must be associated with the same supernatural realm as the system on which it is based."
- `virtue.hex` ArMDE:4077: "This Virtue is most often associated with the Infernal realm." — note "most often", so this is the default-suggestion case of :2960, not a fixed association. `tainted: false` is therefore **correct**: the descriptor at :4076 is `*Major, Supernatural*` with no `Tainted` tag, unlike :4096 and :4128 in the same batch which do carry it and whose entries do set the flag.

`virtue.student_of_realm` shows the shape that would work — a `realm`-domain
parameter alongside its effect.

**The `ability_authorization` pattern.** Two new instances this batch (F-101,
F-102), both of the `categories` shape rather than the `abilities` shape. Two
near-misses that are **not** instances and are recorded so they are not
double-counted later:
- `virtue.hermetic_experience` and `virtue.ineslemen` both grant gated-category Abilities, but both carry a `restricted_ability_xp` whose permission side-effect (A7) already authorizes them. Adding an `ability_authorization` row would be redundant, as A7 states explicitly.
- `virtue.gorgiastic` grants Enigmatic Wisdom (arcane) but scopes it "**after** character creation", so no creation-time authorization is owed — and it is a Hermetic Virtue, so its holders are `is_magus` and whole-character exempt anyway.

---

## Sub-agent reconciliation

**One** verification sub-agent was run, after the first pass was complete. It was
given the nine entries this batch had cleared (seven fully clean, two carrying
only an open question), told to **re-derive each verdict from the rulebook and
the code alone**, told explicitly that overturning one would be a success and
inventing one would not, and **forbidden to read `batch-04.md`**. It was allowed
`README.md`, `decisions.md` and `engine-semantics.md`, and allowed no sub-agents
of its own. It confirmed it changed no file, ran no git-mutating command, and
saw and ignored the "auto mode" Bash injection.

**Result: 7 confirmed, 2 overturned.** Both overturns were re-verified against
the sources before being accepted; every count in this file is
post-reconciliation.

| Entry | Verification verdict | Outcome |
|---|---|---|
| `virtue.giant_blood` | confirmed clean | stands |
| `virtue.great_characteristic` | confirmed clean | stands |
| `virtue.hex` | confirmed clean | stands |
| `virtue.improved_characteristics` | confirmed clean | stands |
| `virtue.induction` | confirmed clean | stands |
| `virtue.ineslemen` | confirmed clean | stands |
| `virtue.guest_of_house_criamon` | confirmed — question only | stands as **Q-25** |
| `virtue.hermetic_prestige` | **OVERTURNED** — German summary renders *Reputation* as "Ansehen" | **independently converged** with F-120 |
| `virtue.greater_benediction` | **OVERTURNED** — a defect, not merely a question | accepted as **F-121**; Q-24 narrowed |

### The convergence on `virtue.hermetic_prestige` is the useful part

This batch's first pass had already moved `virtue.hermetic_prestige` out of the
clean list as **F-120**, found by a deliberate second sweep of every German game
term in the batch against `rules/source/de/translation-tables/`. The
verification agent — which could not see that file — found the same defect
independently, from the same table rows, and **added two corroborations the
first pass had not used**: the emphatic wording at `reputationen.md:16` ("Der
korrekte deutsche Begriff ist **stets** Reputation … **nicht** „Ruf"") and the
structural twin `virtue.famous`, whose German summary uses the canonical term in
the identical sentence shape ("Du hast eine gute **Reputation** der Stufe 4").
Both are now folded into F-120. Two independent derivations landing on the same
correction is the strongest evidence this batch produced for any single finding.

### The `virtue.greater_benediction` overturn is a real correction to this pass

The first pass rated it `?` and raised Q-24, on the ground that the numbers
belong to *example* benedictions rather than to the Virtue. That reasoning is
not wrong, but the verdict was: whichever way Q-24 is decided, `classification:
"narrative"` and `source.lines: [3991, 4008]` make **incompatible claims about
the same bytes**, so a defect exists now. The verification pass made that
concrete by pointing at the guard's own mechanism — `bracketed_passage` slices
exactly the cited range, the slice contains "up to" (:3999) and "+3" (:4007),
and there is no `NO_RULE_DESPITE_TOKEN` row — so the repo's own test goes red on
this entry the moment `SWEPT_BLOCKS` widens past :3950. Recorded as **F-121**,
with Q-24 narrowed to "reclassify or renarrow?".

The lesson, worth writing down because it will recur: **an unsettled question is
not a reason to withhold a verdict when the two fields under dispute already
contradict each other.** The audit's rule is that doubt is escalated, not
resolved quietly — but escalating the *reading* does not excuse failing to
record that the *data* is inconsistent today.

### What the verification pass re-checked and confirmed, independently

Worth recording because it hardens verdicts rather than changing them: it
re-derived `virtue.ineslemen`'s six-id `restricted_ability_xp` expansion of
"(Realm) Lore" and reached the same conclusion this pass did, including that
A7's permission side-effect means no `ability_authorization` is owed; it
confirmed `virtue.hex`'s `tainted: false` from the descriptor and the "most
often" wording at :4077; it followed `virtue.induction`'s "(page 166)" pointer to
ArMDE:7547-7585 and agreed the rules there belong to `ability.induction`, not to
the Virtue; and it independently reached seven of this batch's findings from the
other twenty-six entries while reading the span as prose — `greater_immunity`'s
multiplicity defect (F-94/F-95), both Guild authorizations (F-101/F-102),
`inoffensive_to_beings`'s missing `hermetic` category (F-116),
`hermetic_experience`'s instance scope **and** its dropped prohibition
(F-105/Q-27), `gorgiastic`'s cap (F-92), `guardian_angel`'s +5/15 (F-99), and
`inspirational`'s outward-facing +3 (F-117). None of those was in its brief.

### Two contributions accepted as new open questions

- **Q-33** — ArMDE:2518's worked example gives Hermetic Prestige "a level of 3" where ArMDE:4073 gives level 4. The data follows :4073 and is right; recorded so it is not "corrected" later.
- **Q-34** — ArMDE:18432 states a Size-scaled Improved Characteristics for creatures. Real, sourced, unmodelled — and unreachable, because `EntityKind` has no beast. Recorded rather than rated.

### One correction the verification pass made to this file's own D2 section

The first pass wrote that no shipped grant confers `virtue.great_characteristic`
and that Jerbiton's open slot is the only reachable path. That was incomplete:
`rules/core/mythic_companion_types.json:61-62` names it twice for Nephilim — but
in `required_virtues`, which
`mythic_companion.rs::MythicCompanionType::required_virtues` documents as
"budgeted" and which therefore land in `entity.selections` and *are* inside the
validator's scope. ArMDE:2723-2731 confirms the book intends them to be paid
for. The "Directed reads" section has been corrected accordingly; the
conclusion — that D2 still needs ruling, but that no shipped character is
mis-validated today — is sharper than what the first pass had.

### One verification claim examined and NOT adopted

The verification pass noted a "range-convention drift": some entries end
`source.lines` on the blank line before the next `####` and some on the last
text line. That is true and is already recorded in this file's Method section as
an observation. It is **not** a finding: `rules_source_provenance.rs` accepts
both, no range bisects an entry or bleeds into a neighbour, and B03 recorded the
same variation in its own span without rating it. Cosmetic, catalogue-wide, and
not this audit's object.
