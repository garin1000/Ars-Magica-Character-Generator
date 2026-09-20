# Batch B03 — indices 70-104, ArMDE:3767-3966

Entries: 35. Audited: 35. Failures: **24**. Clean: **11**.
Findings: **24** (F-66 … F-89).
Open questions: **6 new** (Q-17 … Q-22), plus five of B01's/B02's recurring here
(Q-05, Q-06, Q-10, Q-11, Q-14) and two the new ones must be answered together
with (Q-09, Q-13).

An independent verification pass re-derived my 14 clean verdicts from the
rulebook alone, was told that overturning one would be a success and inventing
one would not, and was not allowed to read this file. **Result: 11 confirmed, 3
overturned** (`virtue.faerie_magic` → **F-87**, `virtue.eye_of_hephaestus` →
**F-88**, `virtue.ferocity` → **F-89**), plus one open question I had not raised
(**Q-22**) and four incidentals outside the batch. All three overturns were
re-verified against the sources before being accepted; the counts above are
post-reconciliation. See "Sub-agent reconciliation" at the end.

Finding and question numbers **continue B02's sequence** (B01 used F-01…F-33 /
Q-01…Q-08, B02 used F-34…F-65 / Q-09…Q-16) so `corrections.md` can accumulate
them without collision.

Method note: the whole span was read as continuous prose in **both** languages —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` ArMDE:3760-3974
and the line-parallel German file — before any entry was judged. Line parity
holds throughout the span: every `####` heading sits on the same line number in
both files.

**The `####` walk.** Between ArMDE:3767 and :3966 there are exactly **35**
`####` headings and exactly 35 catalogue entries, and every entry's
`source.lines[0]` is a heading line. No heading is unclaimed and no entry claims
a non-heading. (This is a bracket check, not a census — the missing-entry census
is closed catalogue-wide per the brief.)

**`source.lines` (check 1) — all 35 correct.** Each range runs from its own
heading to the line before the next heading (some include the trailing blank,
some stop one short; neither bisects an entry nor bleeds into a neighbour).
Twelve entries carry an `anchor`, and they are exactly the twelve
`uncomputed_rule` entries: `exotic-casting`, `extractor-of-form-vis`,
`eye-of-hephaestus`, `fabric-ripper`, `falls-like-a-cat`,
`familiarity-with-the-fae`, `finding-hidden-loot`, `flexible-formulaic-magic`,
`folk-magic`, `forgettable-face`, `free-expression`, `frightful-presence`. Each
matches both its own heading and the index link that points at it.

**Magnitude and categories (checks 4 and 6) — verified twice, all 35 agree.**
Against each entry's own descriptor line, and independently against the book's
index groupings: `Hermetic, Major` ArMDE:3008 (Faerie-Raised Magic, Flawless
Magic, Flexible Formulaic Magic, Gentle Gift), `Supernatural, Major` :3023
(Entrancement, Focus Power), `General, Major` :3073 (Ghostly Warder),
`Hermetic, Minor` :3087 (Exotic Casting, Extractor of (Form) Vis, Faerie Magic,
Fast Caster, Free Study), `Supernatural, Minor` :3135 (Eye of Hephaestus, Fabric
Ripper, Faerie Blood, Familiarity with the Fae, Feather Messenger, Folk Magic,
Font of Knowledge, Frightful Presence, Gender Shift), `Social Status, Minor`
:3187 (Eunuch, Factor, Failed Apprentice, Falconer, Fida'i, Forge-Companion,
Gentleman/woman), `General, Minor` :3239 (Falls Like a Cat, Famous, Ferocity,
Finding Hidden Loot, Free Expression, Forgettable Face), `Mythic Companion,
Free` :3329 (Faerie Doctor). `kind` is `virtue` for all 35 and all 35 sit inside
the Virtues section — correct. `entity_kinds` is `["character"]` on all 35,
which is right: none of them is a covenant Boon.

**No U+2212 anywhere** in either locale's `virtues_flaws.json` (whole-file count,
both locales: zero). `virtue.forgettable_face`'s "-3" and
`virtue.frightful_presence`'s "-3" are ASCII hyphens in both locales even though
the English source writes them as en dashes at ArMDE:3931 — correct per the
project rule.

Part C systemic gaps are **not** re-reported per entry. In particular
`advancement_mod` being surfaced-only (C1) is not counted as a defect of
`virtue.free_study`, whose `{source: vis, amount: 3}` is a faithful encoding of
ArMDE:3939; and the surfaced-only `SpecialCasting` kind `faerie_raised` (C1) is
not counted as a defect of `virtue.faerie_raised_magic`. What *is* counted for
those entries is what the entry itself could have carried and does not.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D1 / D4.** This batch holds **none** of the nine `lab_total_mod` carriers, so
neither ruling is exercised. `virtue.extractor_of_form_vis` mentions a Lab Total
in its passage (ArMDE:3781) but carries no effect and claims none.

**D3's precedent is applied throughout** and decides four of this batch's five
classification moves. An engine that structurally cannot express a rule is
grounds for `uncomputed_rule` with the rule written out in both locales, never
for `narrative` — that is what moves `virtue.gentle_gift` (F-81) and
`virtue.ghostly_warder` (F-82). Where the engine *can* express the rule, the
move is to `creation_effect` instead: `virtue.eunuch` (F-66),
`virtue.failed_apprentice` (F-67), `virtue.fidai` (F-68),
`virtue.familiarity_with_the_fae` (F-71), `virtue.frightful_presence` (F-80).

**D2 — not exercised.** `virtue.great_characteristic` is not in this batch and
no entry here carries `characteristic_score_delta_param`, so neither validator's
behaviour is rated.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.entrancement` | 3767-3770 | OK | OK | conferred Ability and its score in neither locale | F-84 |
| `virtue.eunuch` | 3771-3774 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-66 Q-05 |
| `virtue.exotic_casting` | 3775-3778 | OK | OK | OK | |
| `virtue.extractor_of_form_vis` | 3779-3782 | OK | OK | OK | |
| `virtue.eye_of_hephaestus` | 3783-3788 | OK | OK | de renders the Ability *Awareness* as `Wahrnehmungs-Fertigkeit` | F-88 Q-10 |
| `virtue.fabric_ripper` | 3789-3792 | OK | OK | de name contradicts the canonical table | F-85 Q-10 |
| `virtue.factor` | 3793-3796 | OK | OK | OK | |
| `virtue.faerie_blood` | 3797-3820 | OK | effects missing (`ability_authorization`); no parameter for the blood type; five typed bonuses unmodelled | no description; seven types in neither locale | F-69 F-70 |
| `virtue.faerie_doctor` | 3821-3824 | OK | OK | OK | |
| `virtue.faerie_magic` | 3825-3827 | OK | Outer-Mystery cross-reference dropped (three rules) | same three rules in neither locale | F-87 Q-22 |
| `virtue.faerie_raised_magic` | 3829-3842 | OK | effects missing (`special_casting_mod spell_improvisation`) | description missing both locales | F-77 F-78 |
| `virtue.failed_apprentice` | 3843-3846 | narrative → creation_effect | effects missing (`ability_authorization` ×3 categories) | OK | F-67 |
| `virtue.falconer` | 3847-3852 | OK | pool instance scope ×2 | OK | F-73 |
| `virtue.falls_like_a_cat` | 3853-3856 | OK | OK | OK | |
| `virtue.familiarity_with_the_fae` | 3857-3860 | uncomputed_rule → creation_effect | effects missing (`ability_authorization`) | OK | F-71 |
| `virtue.famous` | 3861-3864 | OK | OK | OK | see "Directed read" |
| `virtue.fast_caster` | 3865-3868 | OK | effect unconditional; second modifier absent | second modifier in neither locale | F-75 F-76 |
| `virtue.feather_messenger` | 3869-3872 | `?` | OK | OK | Q-17 Q-05 |
| `virtue.ferocity` | 3873-3876 | OK | Confidence double-counts on every type that already has it | "animals only" in neither locale | F-89 Q-11 |
| `virtue.fidai` | 3877-3882 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-68 Q-20 |
| `virtue.finding_hidden_loot` | 3883-3886 | OK | OK | no `description` in either locale | F-86 Q-06 |
| `virtue.flawless_magic` | 3887-3890 | OK | OK | per-spell special-ability rule in neither locale | F-83 Q-14 |
| `virtue.flexible_formulaic_magic` | 3891-3894 | OK | OK | OK | |
| `virtue.focus_power` | 3895-3906 | OK | OK | OK | ArMDE:2960 instance |
| `virtue.folk_magic` | 3907-3920 | OK | effects missing (`ability_authorization`) | OK | F-72 |
| `virtue.font_of_knowledge` | 3921-3924 | OK | OK | conferred Ability and its score in neither locale | F-84 |
| `virtue.forge_companion` | 3925-3928 | OK | pool instance scope | OK | F-74 Q-21 |
| `virtue.forgettable_face` | 3929-3932 | OK | `incompatible_with` missing (two stated) | OK | F-79 |
| `virtue.free_expression` | 3933-3936 | OK | OK | OK | |
| `virtue.free_study` | 3937-3940 | OK | OK | OK | |
| `virtue.frightful_presence` | 3941-3950 | uncomputed_rule → creation_effect | effects missing (`grants_reputation`) | OK | F-80 |
| `virtue.gender_shift` | 3951-3954 | `?` | OK | OK | Q-17 |
| `virtue.gentle_gift` | 3955-3958 | narrative → uncomputed_rule | prereq possibly over-strict; incompatibility unsourced | description missing both locales | F-81 Q-18 Q-19 |
| `virtue.gentleman` | 3959-3962 | OK | OK | OK | |
| `virtue.ghostly_warder` | 3963-3966 | narrative → uncomputed_rule | OK | description missing both locales | F-82 |

## Findings

**One qualifier that applies to all six authorization findings (F-66, F-67,
F-68, F-69, F-71, F-72) and is stated once here rather than six times.** A14
records that `validation/authorization.rs::validate_ability_authorization`
exempts any character whose type profile has `is_magus`. So the refusal these
six describe bites **non-magi** — grogs and companions. That is the whole
population for the three Social Status entries (F-66, F-67, F-68), and it is
the majority case for the three Supernatural ones (F-69, F-71, F-72), whose
passages are about Faerie Lore and (Realm) Lore — Abilities an unGifted
companion is precisely the character who needs permission for. The defect is
real in every case; the exemption only bounds who meets it.

### F-66 — `virtue.eunuch` — `narrative` on a passage that grants Academic-Ability permission, and the permission is missing

**Passage** (ArMDE:3773, verbatim, English — the entry's whole body):
> You have been educated and trained to serve the Byzantine nobility as a courtier, steward, chamberlain, scribe, lawyer, or accountant. You are likely employed by a court, a lesser noble, or a covenant. It is also possible for you to pursue a career in the eastern Church, and some monasteries accept eunuchs. Due to your education, you may take Academic Abilities during character creation. This Virtue is only available to male characters, who must be sexually incapable, rather than simply inactive. This may be due to castration (see the Castratus Flaw on page 122), but need not be.

German, ArMDE:3773 (line-parallel):
> … Aufgrund deiner Ausbildung darfst du bei der Charaktererschaffung Akademische Fertigkeiten erwerben. …

**Current data:** `"classification": "narrative"`, no `effects`, no `prerequisites`.

**Why it is wrong:** `academic` is one of the three entries of
`rules/core/abilities.json::categories_requiring_virtue`
(`["academic","arcane","martial"]`), so the permission is **computed** —
`validation/authorization.rs::validate_ability_authorization` raises
`ability_category_requires_virtue` against any non-magus holding an Academic
Ability unless something authorizes it. A Eunuch who buys Latin, Artes
Liberales or Civil and Canon Law is refused by the app against a character the
book makes legal in so many words. This is B01's nine-Social-Status shape and
B02's F-38 / F-62 exactly.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`.

**Severity:** wrong-rules-output

*(The same passage's "only available to male characters, who must be sexually
incapable" is B01's Q-05 eligibility question; it is noted, not re-litigated.)*

---

### F-67 — `virtue.failed_apprentice` — three gated categories granted in one sentence, and none of them is authorized

**Passage** (ArMDE:3845, verbatim, English — the relevant sentences):
> You may learn Academic, Arcane, and Martial Abilities during character creation, and you are familiar with the lives of magi. You may not have The Gift, but if your Gift was not completely destroyed, you may have some Supernatural Abilities. The Wealthy Virtue and Poor Flaw affect you normally.

German, ArMDE:3845 (line-parallel):
> Du darfst bei der Charaktererschaffung Akademische, Arkane und Kampf-Fertigkeiten erwerben und bist mit dem Leben der Magi vertraut. Du darfst die Gabe nicht besitzen, …

**Current data:** `"classification": "narrative"`,
`incompatible_with: ["virtue.the_gift"]`, no `effects`.

**Why it is wrong:** the sentence names **all three** gated categories at once —
the widest instance of the pattern the audit has found. The Gift clause is
correctly encoded (and symmetric: `virtue.the_gift` lists
`virtue.failed_apprentice` back). "The Wealthy Virtue and Poor Flaw affect you
normally" is B02's cleared shape — it states that *no* special rule applies.
Only the authorization is missing, and it is the passage's one computable rule.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic","arcane","martial"]}`.

**Severity:** wrong-rules-output

---

### F-68 — `virtue.fidai` — "Fida'i may take Martial Abilities at character creation" is not encoded

**Passage** (ArMDE:3879, verbatim, English — the relevant sentences):
> He has been trained in precise placement of daggers and blades, as well as disguise. Fida'i may take Martial Abilities at character creation. He can expect to become a lasiq if he proves his loyalty to the sect.

German, ArMDE:3879 (line-parallel):
> Er wurde im präzisen Einsetzen von Dolchen und Klingen sowie in der Verkleidung ausgebildet. Fida'i dürfen bei der Charaktererschaffung Kampffertigkeiten erwerben.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong:** `martial` is gated, so a Fida'i who buys Single Weapon —
the Virtue's entire point — draws `ability_category_requires_virtue`. Same
mechanism as F-66.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["martial"]}`.

**Severity:** wrong-rules-output

---

### F-69 — `virtue.faerie_blood` — "can learn Faerie Lore at character generation" is the exact rule the app computes the opposite of

**Passage** (ArMDE:3803, verbatim, English — a paragraph of its own):
> Characters with Faerie Blood can learn Faerie Lore at character generation.

German, ArMDE:3803 (line-parallel):
> Charaktere mit Feenblut können Feenkunde bereits bei der Charaktererschaffung erlernen.

**Current data:** one effect,
`{"type":"aging_mod","kind":"aging_roll","amount":-1}`, and nothing else.

**Why it is wrong:** `ability.faerie_lore` is category `arcane`
(`rules/core/abilities.json`), and `arcane` is in
`categories_requiring_virtue`. The sentence exists precisely to lift that gate,
and the entry lifts nothing — so a character with Faerie Blood who buys Faerie
Lore is refused. The `aging_mod` row itself is **correct**: A38 adds
`aging_roll`'s amount to the Aging Total with its stored sign, and ArMDE:3801
says "get -1 to all aging rolls", so -1 is right in both sign and size.

**Correct value:** add
`{"type":"ability_authorization","abilities":["ability.faerie_lore"]}`.
`classification` stays `in_play_effect` (the entry already carries an in-play
effect, and B1 records that carrying an `Effect` is what decides that line).

**Severity:** wrong-rules-output

---

### F-70 — `virtue.faerie_blood` — the book demands a choice of seven blood types, five of which state numeric bonuses; the entry has no parameter, no effects for them, and no text

**Passage** (ArMDE:3805-3819, verbatim, English — the choice line and the two
types whose mechanics the engine can already express, plus the three it cannot):
> Type of Faerie Blood (pick one, or create a similar one):
>
> *Dwarf Blood:* You are descended from the master craftsmen of the fay, and get a +1 bonus to any total including a Craft Ability.
>
> *Goblin Blood:* Your ancestors were the sneaky inhabitants of the shadows underground, and you get a +1 bonus on all totals involving stealth.
>
> *Satyr Blood:* The satyrs are notoriously lecherous. You get a +1 bonus to Communication and Presence totals when dealing with sexually compatible characters.
>
> *Sidhe Blood:* You are descended from one of the noble fay who rule the lands of Summer and Sunlight. Because of the striking and unusual qualities of your nature add +1 to your Presence, but not to more than +3. Many mortals may consider you fascinating or alluring.
>
> *Undine Blood:* The undines are the faeries of the water, and you get a +2 bonus to any action taken underwater, which will partially offset any penalty applied.

German, ArMDE:3805 and :3815 (line-parallel):
> Art des Feenbluts (eines auswählen oder ein ähnliches erschaffen):
>
> *Sidhe-Blut:* … Aufgrund der auffälligen und ungewöhnlichen Qualitäten deiner Natur erhältst du +1 auf deine Präsenz, jedoch nicht über +3 hinaus. …

(The remaining two types, *Blood of the Bee King* ArMDE:3807 and *Stinnen
Blood* :3817, state powers rather than modifiers.)

**Current data:** no `parameters`, one `aging_mod` effect, no `description` in
either locale. The summary in both locales is the passage's opening flavour
sentence only, so **not one word of ArMDE:3803-3819 reaches the user**.

**Why it is wrong, on three checks at once:**

- **Check 9.** "pick one" is the book demanding a choice, and the option set is
  closed and named (seven types). The catalogue's own idiom for this is an
  `enumerated` parameter — `virtue.folk_magic`, four lines further down the same
  span, does exactly that for its own four-way choice. `virtue.faerie_blood` has
  no parameter at all, so the character sheet cannot record which faerie the
  ancestor was.
- **Check 10.** *Sidhe Blood*'s "+1 to your Presence" is a
  `characteristic_score_delta` on `characteristic.pre` — the variant exists, A24
  computes it, and `virtue.giant_blood` and `flaw.dwarf` already carry two each.
  Its "but not to more than +3" is the *one* place in the catalogue where a
  ceiling on a `characteristic_score_delta` is stated, and A24 records that the
  fold has **no clamp of any kind** deliberately (Giant Blood must reach +6), so
  the cap would need somewhere to live. *Dwarf Blood*'s "+1 bonus to any total
  including a Craft Ability" is an `ability_roll_mod` candidate (B01's Q-06
  shape). Nothing of any of this is carried.
- **Check 12.** Whatever is not computed must at least be readable, and none of
  it is, in either locale.

**Correct value:** an `enumerated` parameter over the seven named types, and —
at minimum — the whole of ArMDE:3803-3819 in `description` in both locales. Whether
Sidhe Blood's +1 Presence should become a real `characteristic_score_delta`
depends on how the parameter gates it, which the engine has no mechanism for
today (an effect cannot be conditioned on a parameter's *value*); on that
reading it is D3 territory and belongs in the description with the rest.

**Severity:** lost-rule (major — a whole choice and five bonuses), plus
wrong-rules-output for the Presence bonus a Sidhe-blooded character never receives

---

### F-71 — `virtue.familiarity_with_the_fae` — the Faerie Lore permission is computable and absent, and the entry is `uncomputed_rule` because of it

**Passage** (ArMDE:3859, verbatim, English — the entry's whole body):
> You have a natural understanding of faerie ways, perhaps due to spending time among them. You get a +2 to all rolls involving social interaction with faeries. You also gain the effects of the Common Sense Virtue, but only when the situation pertains to faeries. You may purchase Faerie Lore at character generation, even if normally unable to take Arcane Abilities.

German, ArMDE:3859 (line-parallel):
> … Du darfst Feenkunde bei der Charaktererschaffung erwerben, selbst wenn du normalerweise keine Arkanen Fertigkeiten nehmen kannst.

**Current data:** `"classification": "uncomputed_rule"`, no `effects`, a
`description` in both locales carrying the passage in full.

**Why it is wrong:** the description is good and the +2 and the Common Sense
clause are genuinely uncomputable — but the last sentence names the gate
explicitly ("even if normally unable to take Arcane Abilities") and the engine
computes exactly that gate. B1 is unambiguous that an entry the engine computes
*something* for is not `uncomputed_rule`: "carrying an `Effect` is what decides
it". So this entry is one `ability_authorization` short, and the classification
follows the effect rather than the other way round.

Note the contrast with `virtue.faerie_blood` (F-69) one column over: there the
permission is worded as a bare grant, here as an explicit exemption. Both are
the same computed rule and neither entry carries it.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","abilities":["ability.faerie_lore"]}`. The
`description` stays, carrying the two clauses that remain uncomputed — which is
the nine-of-93 practice Q-14 is about, and here it already exists.

**Severity:** wrong-rules-output

---

### F-72 — `virtue.folk_magic` — the (Realm) Lore the Virtue is built on is an Arcane Ability, and the entry does not authorize it

**Passage** (ArMDE:3909, verbatim, English — the opening):
> The character is capable of performing very minor acts of magic through his knowledge of scraps of occult lore. Choose one (Realm) Lore that is the key Ability for this magic, he may learn this Ability at Character Creation even if he is normally unable to take Arcane Abilities.

German, ArMDE:3909 (line-parallel):
> … Wähle eine (Sphären-)Kunde als Schlüsselfertigkeit für diese Magie; er darf diese Fertigkeit bereits bei der Charaktererschaffung erlernen, auch wenn er normalerweise keine Arkanen Fertigkeiten erwerben kann. …

**Current data:** `"classification": "uncomputed_rule"`; two parameters
(`category`, `enumerated` over the four named options; `realm`, domain `realm`,
with `at_most_one_of: [["realm.divine","realm.infernal"]]`); no `effects`; a
`description` in both locales carrying the whole fourteen-line passage.

**What is right, and should not be touched.** The four `enumerated` values match
the book's four options one-for-one (ArMDE:3911-3917), both locales carry labels
for them, and `at_most_one_of` is the exact encoding of ArMDE:3919's "a character
cannot have access to both the Divine and Infernal Realms" — B8 confirms the
constraint spans **all copies** of the item, which is what that sentence needs.
"You may pick this Virtue more than once" is satisfied by the default
`max_per_target: 1` keyed on the whole params map plus the default `max_total`.

**Why it is nevertheless wrong:** all four (Realm) Lores in
`rules/core/abilities.json` — `ability.magic_lore`, `ability.faerie_lore`,
`ability.infernal_lore`, `ability.dominion_lore` — are category `arcane`, and
`arcane` is gated. So a Folk Magician who buys the key Ability the Virtue is
defined around is refused.

**Correct value:** an `ability_authorization` naming the four (Realm) Lores, and
`"classification": "creation_effect"`. **Caveat, recorded rather than hidden:**
`AbilityAuthorization` carries a fixed list and cannot be scoped to the `realm`
the player chose, so a four-Lore authorization is *over*-permissive — it would
legalise Infernal Lore for a Magic-realm Folk Magician. The alternative is to
leave it uncomputed, which is what ships today. I record the over-permissive fix
as the correct value because the present state refuses a legal character
(an error) whereas the fix permits an illegal one (no error raised) — but the
choice between the two is a judgement the corrections pass should make
deliberately, not inherit from this line.

**Severity:** wrong-rules-output

---

### F-73 — `virtue.falconer` — the 50-point pool funds any Dead Language and any Profession, where the book names Latin and Profession: Falconer

**Passage** (ArMDE:3851, verbatim, English):
> A falconer receives 50 extra experience points at character generation to spend on the Abilities: Animal Handling, Area Lore, Etiquette, Hunt, Latin, Profession: Falconer, and Ride. Many falconers are also Educated.

German, ArMDE:3851 (line-parallel):
> Ein Falkner erhält bei der Charaktererschaffung 50 zusätzliche Erfahrungspunkte, die auf folgende Fertigkeiten verwendet werden können: Tierumgang, Gebietskunde, Etikette, Jagen, Latein, Beruf: Falkner und Reiten.

**Current data:**
```
{"type":"restricted_ability_xp","amount":50,
 "abilities":["ability.animal_handling","ability.area_lore","ability.dead_language",
              "ability.etiquette","ability.hunt","ability.profession","ability.ride"]}
```

**Why it is wrong:** `ability.dead_language` is parameterized by `language` and
`ability.profession` by `profession`, and A7 records that a restricted pool
matches the Ability **id with no instance test at all**. So the 50 points fund
Greek, Hebrew or Aramaic as readily as Latin, and Profession: Scribe as readily
as Profession: Falconer. This is B01's **F-31** and B02's **F-36 / F-43 / F-60**
— the fifth and sixth instances in 105 entries, and the first entry to carry
*two* of them. (`ability.area_lore` is also parameterized, but the book says
plain "Area Lore", so that one is correct as written.)

The permission side-effect is fine: `ability.dead_language` is category
`academic` and gated, and A7 records that a pool confers permission over the
lists it carries, so Latin is legal for a Falconer without a separate
authorization.

**Correct value:** an instance-scoped pool (a variant change, not a data edit —
`PoolEligibility::Ability.instances` exists but `restricted_ability_xp_pools`
hard-codes it empty).

**Severity:** wrong-rules-output (minor)

---

### F-74 — `virtue.forge_companion` — the 50-point pool funds any Craft, where the book names the master's Crafts

**Passage** (ArMDE:3927, verbatim, English — the entry's whole body):
> The character is an unGifted craftsman attached to a Verditius magus and working for him in his lab. You receive 50 additional experience points, which you can use to raise the particular Crafts her master practices. As a member of his household, she receives protection and support, but she is not protected by any of the legal codes of the Order of Hermes.

German, ArMDE:3927 (line-parallel):
> … Du erhältst 50 zusätzliche Erfahrungspunkte, die du verwenden kannst, um die spezifischen Handwerke zu steigern, die sein Meister ausübt. …

**Current data:**
`{"type":"restricted_ability_xp","amount":50,"abilities":["ability.craft"]}`

**Why it is wrong:** `ability.craft` is parameterized by `craft`, so per A7 the
pool funds every Craft the character owns, where the book restricts it to "the
particular Crafts her master practices". Same mechanism as F-73; listed
separately because the two entries need different scoping data (a language
instance vs. a set the player must name).

**Correct value:** an instance-scoped pool. Unlike F-73 the scoping value is not
fixed by the book — it is "her master's Crafts" — so this one additionally needs
a parameter recording which Crafts those are.

**Severity:** wrong-rules-output (minor)

---

### F-75 — `virtue.fast_caster` — the +3 Initiative is unconditional in the data and scoped to spellcasting in the book, so it lands on weapon Initiative too

**Passage** (ArMDE:3867, verbatim, English — the entry's whole body):
> Your magic takes less time to perform than that of other magi. You gain +3 to Initiative to cast spells in combat and +3 to rolls to determine fast casting speed.

German, ArMDE:3867 (line-parallel):
> Deine Magie erfordert weniger Zeit als die anderer Magi. Du erhältst +3 auf Initiative zum Zaubern im Kampf und +3 auf Würfe zur Bestimmung der Schnellzauber-Geschwindigkeit.

**Current data:** `{"type":"combat_mod","amount":3,"target":"initiative"}`.

**Why it is wrong:** the book's bonus is "+3 to Initiative **to cast spells**".
A35 records that `CombatMod` "silently ignores **circumstance** — Berserk's
combat bonuses apply always", and that unscoped figures sum per stat across
`derived/combat.rs::combat_totals`' whole grid. So a Fast Caster swinging a
sword gets +3 Initiative he has not earned, on every weapon line the sheet
prints. The variant's `weapon` field scopes to a weapon, not to an activity, so
there is no data spelling that fixes this.

This is B02's **F-45** shape (`virtue.cyclic_magic_positive`: an unconditional
effect where the book states a condition) on a different variant, and it is the
first time it has appeared in `combat_mod`.

**Correct value:** either a scope axis on `CombatMod` (an engine change), or —
if the number is to stay flat — a recorded decision saying so, as D1 did for the
spell-level cap. What must not happen is that the condition disappears without
either.

**Severity:** wrong-rules-output

---

### F-76 — `virtue.fast_caster` — the second +3 is in neither the data nor either locale

**Passage:** ArMDE:3867 as quoted in F-75 — "and +3 to rolls to determine fast
casting speed".

**Current data:** one effect, covering the first +3. `summary` in both locales
is the opening flavour sentence ("Your magic takes less time to perform than
that of other magi." / "Deine Magie erfordert weniger Zeit als die anderer
Magi."). No `description`.

**Why it is wrong:** half the entry's mechanical content — a second, separate
+3 on a different roll — reaches the user through no surface at all. Fast
casting speed has no engine model, so this is D3 / B01-F-33 territory: the rule
must be carried as text.

**Correct value:** a `description` in **both** locales carrying the full
sentence. **Q-14 applies**: this is an `in_play_effect` entry, which is the class
where the practice already exists (nine of 93 carry a description), but the
residue here is one clause and Q-14 asks exactly where the threshold is. I record
it as a defect and flag the dependency rather than pretending the threshold is
settled.

**Severity:** lost-rule

---

### F-77 — `virtue.faerie_raised_magic` — "This Virtue also includes the Virtue Spell Improvisation", and the included Virtue's own effect is not carried

**Passage** (ArMDE:3839, verbatim, English — a paragraph of its own):
> This Virtue also includes the Virtue Spell Improvisation. That is, you may add the magnitude of a known Formulaic spell as a bonus to your character's Casting Total when spontaneously casting a spell that is similar to it.

German, ArMDE:3839 (line-parallel):
> Diese Tugend schließt auch die Tugend Zauberimprovisation ein. Das bedeutet, du kannst die Magnitude eines bekannten formulaischen Zaubers als Bonus zu deiner Zaubersumme hinzufügen, wenn du spontan einen ähnlichen Zauber wirkst.

**And what it points at**, ArMDE:5004 (`#### Spell Improvisation`, verbatim):
> The magus may add the magnitude of a Formulaic spell he knows as a bonus to his Casting Total when Spontaneously casting a spell that is similar to it (see Similar Spells, page 260). This includes fast-casting a spell that is the same as or very similar to one of his Formulaic spells, though he does not get this bonus if he has the Fast Cast Ability for a mastered spell, since in that case you add his Mastery Ability instead. This bonus does not stack with other bonuses to his Casting Total, nor does it stack with itself if the magus happens to know several similar spells.

`virtue.spell_improvisation` exists in the catalogue (`virtue`/`minor`,
`hermetic`, `in_play_effect`) and carries `special_casting_mod` with kind
`spell_improvisation`. ArMDE:3839's second sentence restates :5004's first
almost word for word, which removes any doubt that the cross-reference is
literal.

**Current data:** one effect,
`{"type":"special_casting_mod","kind":"faerie_raised"}`.

**Why it is wrong:** this is B02's **F-62** pattern — a rule stated by
cross-reference to another entry, invisible to any screen that reads only the
referring entry's own line range. Faerie-Raised Magic's data claims one of the
two kinds it is entitled to. Both kinds are surfaced-only (C1), so no number
changes either way; what changes is that the read-out lists one modifier where
the book gives two, and a later slice that makes `spell_improvisation` compute
will silently skip every Faerie-Raised magus.

The alternative encoding is
`{"type":"grants_selection","items":["virtue.spell_improvisation"]}`, which A18
would expand into the folded list and which states the book's "includes the
Virtue" literally. Either is defensible; carrying neither is not.

**Correct value:** add the second `special_casting_mod` (kind
`spell_improvisation`), or a `grants_selection` naming
`virtue.spell_improvisation`.

**Severity:** lost-rule

---

### F-78 — `virtue.faerie_raised_magic` — a fourteen-line passage containing a complete spell-invention costing system ships with a one-sentence summary and no description

**Passage** (ArMDE:3833-3837, verbatim, English — the two rules paragraphs):
> Because of this, he can teach himself spells outside of the laboratory. You may spend experience points from Exposure, Adventure, and Practice on spells you could normally invent that mimic faerie powers or other supernatural effects your character has observed that season.
>
> To invent a spell in this way, the magus's Technique + Form + Intelligence + Magic Theory must at least equal (the spell's level – 10), and you must spend a number of experience points equal to (the spell magnitude + 4). Spells of level 5 or less cost their level in experience points (but always at least 1). For example, a level 2 spell costs 2 experience points, and a Level 15 spell costs 7 experience points. The magus cannot invent Ritual spells in this way.
>
> If the magus already knows a spell that is similar to one he wishes to invent (see Similar Spells, page 260), any experience points you spend towards learning that spell are increased by one-half, rounded up. This means that a level 25 spell would cost 6 experience points instead of 9, for example.

German, ArMDE:3835 (line-parallel):
> Um einen Zauber auf diese Weise zu erfinden, muss die Summe aus Technik + Form + Intelligenz + Magietheorie des Magus mindestens (Zauberstufe – 10) betragen, und du musst eine Anzahl von Erfahrungspunkten aufwenden, die (der Magnitude des Zaubers + 4) entspricht. …

**Current data:** `summary` in both locales is the opening flavour sentence
only; no `description`.

**Why it is wrong:** a threshold formula, a cost formula, a floor ("but always
at least 1"), two worked examples, an exclusion ("cannot invent Ritual spells")
and a discount rule ("increased by one-half, rounded up") all leave the
application. This is far above every calibration case Q-14 lists — Afflicted
Tongue's residue is an extra botch die and a penalty; this is a subsystem. The
engine does not model out-of-lab spell invention, so text is the only surface
available.

**Correct value:** a `description` in **both** locales carrying ArMDE:3833-3837.
The trailing advice paragraph (ArMDE:3841, "you should normally take the Faerie
Upbringing Flaw") is advice and need not be encoded as a prerequisite.

**Severity:** lost-rule (major)

---

### F-79 — `virtue.forgettable_face` — two prohibitions stated by name, encoded on neither side, and both targets exist

**Passage** (ArMDE:3931, verbatim, English — the closing sentences):
> On the downside, this trait is incompatible with great beauty, charisma, or commanding presence. Virtues like Venus' Blessing or Inspirational are prohibited, although Curse of Venus could work.

German, ArMDE:3931 (line-parallel):
> Auf der negativen Seite ist diese Eigenschaft unvereinbar mit großer Schönheit, Charisma oder befehlsgewaltiger Präsenz. Tugenden wie Venussegen oder Inspirierend sind ausgeschlossen, obwohl Venusfluch möglich wäre.

**Current data:** no `incompatible_with` on `virtue.forgettable_face`, and none
on `virtue.venus_blessing` or `virtue.inspirational` either (both exist in the
catalogue, both `virtue`/`minor`/`general`, both `narrative`).

**Why it is wrong:** the book uses the word "incompatible" and then names two
Virtues. `incompatible_with` is the field for exactly that, and
`ruleset/integrity.rs::validate_incompatibility_symmetry` would require the
declaration on both sides — so the fix is three entries, not one. This is B02's
**F-41** shape (`virtue.covenfolk` / `virtue.custos`, "may not take the Wealthy
Virtue or Poor Flaw", encoded on neither, both targets existing).

The hedge "Virtues **like** Venus' Blessing or Inspirational" does open a
judgement question about *unnamed* Virtues of great beauty or charisma, which
the data cannot express. That residue belongs in the description — which the
entry already has, carrying the sentence verbatim in both locales. Only the two
**named** ones are encodable, and neither is encoded.

**Correct value:** `incompatible_with: ["virtue.inspirational","virtue.venus_blessing"]`
on `virtue.forgettable_face`, and `virtue.forgettable_face` added to each of the
other two.

**Severity:** wrong-rules-output (an illegal combination is accepted silently)

---

### F-80 — `virtue.frightful_presence` — the Reputation the passage grants is encoded nowhere, so a character who enters it is refused

**Passage** (ArMDE:3947, verbatim, English — the relevant clause):
> Once a person has been affected by the Frightful Presence and recovered from its effects, he cannot be affected again; although you will acquire an appropriate Reputation (such as Fearsome or Awesome) at a score of 2 among those you have affected, which will color your dealings with them.

German, ArMDE:3947 (line-parallel):
> … allerdings wirst du eine entsprechende Reputation (wie Furchteinflößend oder Ehrfurchtgebietend) mit einem Wert von 2 bei jenen erwerben, die du betroffen hast, …

**Current data:** `"classification": "uncomputed_rule"`, no `effects`, a
`description` in both locales carrying the whole four-paragraph passage.

**Why it is wrong:** `validation/scores.rs::validate_reputations` raises
`reputation_not_granted` — an **error** — against any `Entity::reputations` row
with no matching `GrantsReputation` slot. A25's own summary of the rule is that
"a grant does not *create* the Reputation — it authorizes one". So the app
refuses a Frightful Presence character who writes down the Reputation his Virtue
gives him. `grants_reputation` is the most-used variant in the catalogue (33
uses) and `ReputationType::Local` exists, so the shape is available and
familiar.

Independent corroboration that this is read as a granted Reputation and not as
colour: the project's own curated table
`rules/source/de/translation-tables/reputationen.md:83`, whose whole section is
titled "Reputationen aus Tugenden und Fehlern / Reputations Granted by Virtues
and Flaws", lists
`Frightful Presence | Furchteinflößende Erscheinung | Lokal | 2 (+) | Furchteinflößend oder Ehrfurchtgebietend`
— type, score and example content all matching ArMDE:3947.

**Correct value:** `{"type":"grants_reputation","kind":"local","score":2}` and
`"classification": "creation_effect"` (an entry carrying an effect cannot stay
`uncomputed_rule`; `data_integrity.rs::every_vf_is_classified` enforces that).
The `description` stays for the Brave rolls and the Penetration 0.

**Caveat recorded, not hidden.** The Reputation is acquired *in play*, "among
those you have affected", not at character creation — so one could argue the slot
should not exist at generation time. I do not think that survives contact with
the engine: A25's slot is an authorization, not a fact, and the alternative is
that the app errors on a legal sheet. But the timing is the one thing about this
finding a reader might reasonably dispute, so it is stated rather than buried.

**Severity:** wrong-rules-output

---

### F-81 — `virtue.gentle_gift` — `narrative` on the Virtue whose entire content is the removal of a stated numeric penalty

**Passage** (ArMDE:3957, verbatim, English — the entry's whole body):
> Unlike other magi, whose Magical nature disturbs normal people and animals, your Gift is subtle and quiet. You do not suffer the usual penalties when interacting with people and animals.

German, ArMDE:3957 (line-parallel):
> Im Gegensatz zu anderen Magi, deren magische Natur normale Menschen und Tiere beunruhigt, ist deine Gabe subtil und still. Du leidest nicht unter den üblichen Abzügen im Umgang mit Menschen und Tieren.

**And what "the usual penalties" are**, ArMDE:8751 (verbatim):
> If the maga tries to overcome this reaction through negotiation, she suffers a -3 penalty to any die rolls she must make. Someone without The Gift negotiating on her behalf does not suffer the penalty, but must deal with the mistrust inspired by The Gift. If the maga manages to convince or coerce someone into interacting with her, she suffers the -3 penalty to all rolls and totals based on social interaction, including training, whether the maga is the trainer or the trainee.

and ArMDE:8749, which names this Virtue as the exemption:
> Some Hermetic magi have the Gentle Gift (a Virtue, see page 82), which does not affect people in this way …

**Current data:** `"classification": "narrative"`, no `effects`, no
`description`; `summary` is "Your Gift does not disturb others." / "Deine Gabe
verstört andere nicht."

**Why it is wrong:** `narrative` asserts the passage states nothing mechanical.
ArMDE:3957 states the removal of a penalty the book quantifies at -3 two
sentences of cross-reference away, affecting "all rolls and totals based on
social interaction, including training". That is a mechanical clause by any
reading, and B1's own wording of the `narrative` claim — "an entry whose text
states a signed modifier … is **not** narrative even when the engine computes
nothing" — covers it directly. The engine models neither the Gift's penalty
(`virtue.the_gift` itself is `narrative` with no effects) nor its removal, so
D3 applies: `uncomputed_rule` with the rule written out, never `narrative`.

Note the asymmetry this exposes: `flaw.blatant_gift`, the same mechanism in the
opposite direction and a -6 stated in its own body (ArMDE:5713), **is** already
`uncomputed_rule`. The two ends of one rule are classified differently today.

**Correct value:** `"classification": "uncomputed_rule"` plus a `description` in
**both** locales carrying ArMDE:3957 and naming the -3 it waives (with an ASCII
hyphen). `crates/arm-rules/tests/uncomputed_clauses.rs` requires a mechanical
token in the displayed text of every `uncomputed_rule` entry in every locale, so
the description is not optional once the class moves.

**Severity:** lost-rule — and, because it is the flagship Major Hermetic Virtue
of House Jerbiton (ArMDE:707), a conspicuous one

---

### F-82 — `virtue.ghostly_warder` — `narrative` on a passage that states an experience-point total, an Ability freedom and a once-per-day limit

**Passage** (ArMDE:3965, verbatim, English — the entry's whole body):
> A ghost watches over you. It might be a grandparent, a childhood friend, or anyone else who cares for you enough to stay around after death. The ghost is invisible and silent to all but you and those with Second Sight (see page 170). It can see and hear what is going on around you and makes an excellent spy, since it can leave your presence once per day for up to half an hour. However, death does not leave people in their normal state of mind, so the ghost probably has some quirks that make it less than dependable — it might even encourage you to join it on the other side. The ghost has 300 experience points in various Abilities that it can use to advise you, and ghosts may take any Abilities. See page 457 for an example of a ghostly warder. This is a more specific version of Magical Warder (see later), and is an example of that Virtue.

German, ArMDE:3965 (line-parallel):
> … Der Geist verfügt über 300 Erfahrungspunkte in verschiedenen Fertigkeiten, die er verwenden kann, um dich zu beraten, und Geister dürfen jede Fertigkeit besitzen. …

**Current data:** `"classification": "narrative"`, no `effects`, no
`description`; `summary` is "A ghost watches over you." / "Ein Geist wacht über
dich."

**Why it is wrong:** three mechanical clauses, one of them a bare number:

1. **"The ghost has 300 experience points in various Abilities"** — a build
   budget for the warder, as concrete as any of this batch's 50-point pools.
2. **"ghosts may take any Abilities"** — an explicit lifting of the Ability
   category restrictions, for the warder.
3. **"it can leave your presence once per day for up to half an hour"** — a
   frequency and duration limit on the Virtue's usable benefit.

The engine builds no companion NPC, so none of the three is computable and D3
puts the entry at `uncomputed_rule` with the rules written out — the same place
B02 put `virtue.command_animals` for "a count of commandable animals", which is
a strictly weaker case than a 300-point budget.

**The cross-reference was followed** (B02's F-62 discipline): "This is a more
specific version of Magical Warder (see later)". `virtue.magical_warder`
(ArMDE:4377-4381) states no number at all — its closest approach is "leave your
presence for up to half a day" — so Ghostly Warder inherits nothing computable
and nothing is owed on that account. It is, however, itself `narrative` with no
effects and carries the same half-a-day limit; **B06, whose span covers
ArMDE:4351-4597, should judge it on its own terms rather than inherit this
line.**

**Correct value:** `"classification": "uncomputed_rule"` plus a `description` in
**both** locales carrying ArMDE:3965.

**Severity:** lost-rule

---

### F-83 — `virtue.flawless_magic` — the per-spell special-ability rule is in neither locale

**Passage** (ArMDE:3889, verbatim, English — the entry's whole body):
> You automatically master every spell that you learn. All your spells start with a score of 1 in the corresponding Spell Mastery Ability. You may choose a different special ability for every spell you have. Further, all your Advancement Totals for Spell Mastery Abilities are doubled.

German, ArMDE:3889 (line-parallel):
> Du meisterst automatisch jeden Zauber, den du lernst. Alle deine Zauber beginnen mit einem Wert von 1 in der entsprechenden Zaubermeisterschafts-Fertigkeit. Du kannst für jeden deiner Zauber eine andere Spezialisierung wählen. Darüber hinaus werden alle deine Fortschrittssummen für Zaubermeisterschafts-Fertigkeiten verdoppelt.

**Current data:**
`{"type":"grants_spell_mastery","score":1,"advancement_num":2,"advancement_den":1}`
— **both halves correct**. A17: `score` is a floor applied to every known spell,
which is exactly "all your spells start with a score of 1"; and the `2/1` ratio
is collected only when `num > den` and applied as
`charged_cost = ceil(payable · den / num)`, i.e. half price, which is what
doubling an Advancement Total buys. `summary` in both locales is the first
sentence. No `description`.

**Why it is wrong:** the third sentence is a separate rule — normally a magus's
Spell Mastery special abilities are constrained; this Virtue lifts that per
spell — and it reaches the user nowhere. The engine ships
`rules/core/spell_mastery_abilities.json`, so the subject exists in the app even
though this freedom is not modelled.

**Correct value:** a `description` in **both** locales carrying the third
sentence (or the whole passage).

**This proposal depends on Q-14, and on the harder half of it.** This is a
`creation_effect` entry, and **0 of the 125 `creation_effect` entries in the
catalogue carry a `description`** — so unlike F-76 and F-78 this is not
restoring an existing practice but proposing a new one. Recorded as a defect and
explicitly parked on that dependency.

**Severity:** lost-rule (minor)

---

### F-84 — `virtue.entrancement` and `virtue.font_of_knowledge` — the Ability each confers, and its score, appear in neither locale's displayed text

**Passages** (verbatim, English — each entry's whole body):
> ArMDE:3769 — You have the power to control another's will by staring into their eyes and giving them a verbal command. Choosing this Virtue confers the Ability Entrancement 1 (page 164).
>
> ArMDE:3923 — You have supernatural access to information you have never learned. This Virtue bestows the Ability Font of Knowledge 1.

German, ArMDE:3769 and :3923 (line-parallel):
> Die Wahl dieser Tugend verleiht dir die Fertigkeit Betörung 1 ([Seite 164]…).
>
> Diese Tugend verleiht dir die Fertigkeit Quell des Wissens 1.

**Current data:** both carry the right effect —
`ability_score_grant ability.entrancement 1` and
`ability_score_grant ability.font_of_knowledge 1`; both target ids exist in
`rules/core/abilities.json` with category `supernatural` (not gated, so A9's
permission side-effect is sufficient and no authorization is needed). Both
locales' `summary` is the passage's **first** sentence only, and neither entry
has a `description`. So the conferral sentence — the entry's only mechanical
content — is displayed nowhere.

**Why it is wrong:** this is the second half of B02's **F-65**, one step further.
There, two summaries used the conferral formula and dropped the score; here two
summaries omit the conferral entirely. The four siblings B02 measured
(`faerie_magic`, `heartbeast`, `premonitions`, `second_sight`) all carry it,
`virtue.faerie_magic` in this very batch reading "Verleiht die Fertigkeit
Feenmagie auf 1." — so the convention exists and these two depart from it.

Nothing computes wrong: the score reaches the sheet through the grant. This is
displayed-text completeness, on the same footing F-65 gave it.

**Correct value:** append the conferral with its score to both summaries in both
locales, on the `virtue.faerie_magic` template ("Confers the Entrancement
Ability at 1." / "Verleiht die Fertigkeit Betörung auf 1.").

**Severity:** text completeness (minor)

---

### F-85 — `virtue.fabric_ripper` — the German name contradicts the canonical translation table

**The two canonical German sources disagree.**

`rules/source/de/translation-tables/tugenden-fehler.md:118`, verbatim:
> | Fabric Ripper | Stoffzerreißer | |

The German rulebook, ArMDE:3789 (line-parallel with the English heading):
> #### Stoffreißer

**Current data:** `rules/i18n/de/virtues_flaws.json` has
`"name": "Stoffreißer"` — the rulebook's spelling, not the table's.

**Why it is a finding:** CLAUDE.md states that the tables are "the **canonical**
EN→DE terminology mapping" and that "the German label for any term whose English
form appears in a table MUST match the table's `Deutsch (DE)` value" — while the
same file also makes the German rulebook "the source of truth for German
translations". Here the two collide on a single word. This is exactly B02's
**Q-10** (`virtue.covenfolk`, "the two canonical German sources give two
different names"), and it is the second instance, which makes it a recurring
class rather than an accident.

A near-miss of the same kind sits two entries away and is recorded here rather
than filed separately, because it is an `Anmerkung` and not a `Deutsch (DE)`
cell: `tugenden-fehler.md:170`'s note on Magical Warder reads "auch für
Geisterhaften **Wächter**", while the rulebook heading at ArMDE:3963 is
"Geisterhafter **Hüter**" and the i18n follows the rulebook.

**Correct value:** whichever Q-10 resolves to. I do **not** pick one.

**Severity:** text — localization, rated up per CLAUDE.md, but blocked on Q-10

---

### F-86 — `virtue.finding_hidden_loot` — the only `uncomputed_rule` in the batch with no `description` in either locale

**Passage** (ArMDE:3885, verbatim, English — the entry's whole body):
> A character with this Virtue gains a +9 bonus to Awareness rolls when searching a confined space for hidden items, provided they can move the contents of the space about undisturbed.

German, ArMDE:3885 (line-parallel):
> Ein Charakter mit dieser Tugend erhält +9 Bonus auf Wahrnehmungswürfe beim Durchsuchen eines abgegrenzten Raumes nach versteckten Gegenständen, sofern er den Inhalt des Raumes ungestört umherräumen kann.

**Current data:** `"classification": "uncomputed_rule"`, no `effects`, **no
`description`** — but the `summary` in both locales is the whole sentence
verbatim, +9 included.

**Why it is a finding, and why it is a small one.** `README.md`'s partition
table is explicit: `uncomputed_rule` means "the rule must be written out in
`description` in every locale". Eleven of the twelve `uncomputed_rule` entries
in this batch do exactly that; this is the twelfth. Nothing is lost to the user
today — the rule is on screen, in the summary — and the guard agrees:
`uncomputed_clauses.rs` tests `description`, **else** `summary`, so it stays
green. So this is a consistency defect against the class's stated contract,
not a lost rule.

The +9-on-Awareness-rolls shape itself is **B01's Q-06** (an `AbilityRollMod`
candidate carried as prose), not a new finding — `ability_roll_mod` is
surfaced-only per C1, so nothing would be computed by making it an effect.

**Correct value:** either a `description` carrying the sentence in both locales,
or an explicit ruling that a summary carrying the full rule satisfies
`uncomputed_rule`. The second would be cheaper and would need recording in
`README.md`.

**Severity:** consistency (minor)

---

*The three findings below were produced by the verification pass, against three
of my own clean verdicts. Each was re-derived against the sources before being
accepted here; the re-derivation is recorded under "Sub-agent reconciliation".*

### F-87 — `virtue.faerie_magic` — the "(see page 236)" cross-reference carries three rules the entry drops, and the book attributes them to the Virtue by name

**Passage** (ArMDE:3827, verbatim, English — the entry's whole body):
> You have been initiated into the Outer Mystery of Faerie Magic (see page 236), and thus are a member of House Merinita. You have the Ability Faerie Magic 1. Note that all Merinita magi gain this Virtue for free at character generation.

**And what the pointer leads to**, ArMDE:9967 (inside `### Merinita — Faerie
Magic`, verbatim):
> Characters initiated into Faerie Magic are attuned to both Magical and Faerie auras, and so gain Warping Points from neither. Further, they gain full benefit from both kinds of aura, and do not gain additional botch dice from either. Magic cast by these magi counts as a fay power, so anyone who gains a Warping Point from one of their spells can be initiated into the Mystery.

**The clincher — the book names the Virtue, not the Mystery chapter**,
ArMDE:9626 (verbatim):
> Note that there are a number of Virtues and Flaws that can modify both the number of botch dice, and what they mean. For example, the Faerie Magic Virtue attunes Merinita magi to Faerie auras, so that they do not gain additional botch dice in them.

and ArMDE:9969, scoped to this Virtue's own tier:
> Initiates of the Outer Mystery gain access to special Ranges, Durations, and Targets. They may use these with Spontaneous, Ritual, and Formulaic magic, although some of them require Ritual magic. Spells created using these parameters can only be learned by characters with Faerie Magic.

**Current data:** `classification: creation_effect`; one effect,
`ability_score_grant ability.faerie_magic 1`; `prerequisites: {house:
house.merinita}`; **no `description` in either locale**, and both summaries stop
at the Ability grant ("Initiation into the Outer Mystery of House Merinita.
Confers the Faerie Magic Ability at 1." / "Einweihung in das Äußere Mysterium
des Hauses Merinita. Verleiht die Fertigkeit Feenmagie auf 1.").

**Why it is wrong, and why I missed it.** The Ability grant and the House
prerequisite are both correct — I cleared the entry on B02's `virtue.the_enigma`
precedent, which settles the *prerequisite-as-consequence* question and nothing
else. What that precedent does not cover, and what I did not do, is **follow the
page reference**. Three separate mechanics live behind it: no Warping from
Magic or Faerie auras, full aura benefit from both, and no extra botch dice in
either. ArMDE:9626 removes any argument that these belong to the Mystery rather
than to the Virtue — it says "the Faerie Magic Virtue attunes Merinita magi".

None of the three reaches the user by any channel. The engine models no aura,
no aura-sourced Warping and no botch dice, so nothing computes them; and a
`creation_effect` entry carries no `description`, so nothing prints them.

**Correct value:** the rules of ArMDE:9967 must be carried as text in **both**
locales. *Where* is Q-22's subject and I do not settle it here: a `description`
on a `creation_effect` entry would be a new practice (0 of 125 today), and the
alternative — extending the summary — has its own cost.

**The same pointer sits on three sibling entries, all outside this batch.**
`crates/arm-rules/RULES.md` treats the four Outer-Mystery Virtues uniformly for
House membership and records nothing about their other mechanics, so
`virtue.the_enigma` (ArMDE:3759-3761, **which B02 cleared**),
`virtue.heartbeast` (ArMDE:4059-4061) and `virtue.verditius_magic`
(ArMDE:5215-5217) each need the same pointer followed. **B02's clearance of
`virtue.the_enigma` should be treated as covering the prerequisite only.**

**Severity:** lost-rule

---

### F-88 — `virtue.eye_of_hephaestus` — the German description names an Ability that does not exist in the German app

**Passage** (ArMDE:3785, the clause at issue, verbatim):
> For supernatural items made by a craftsman, make a Perception + Awareness + stress die roll against an Ease Factor of 9. … For Hermetic enchanted items, or those made by other sorcerers, make a Perception + Awareness + stress die roll against an Ease Factor of 12.

German, ArMDE:3785 (line-parallel), reproduced verbatim in
`rules/i18n/de/virtues_flaws.json`:
> … wird ein **Wahrnehmung + Wahrnehmungs-Fertigkeit** + Stresswürfelwurf gegen einen Schwierigkeitsgrad von 9 vorgenommen … (and again for the 12 case)

**Why it is wrong — and the app itself is the witness, not only the table.**

1. `rules/source/de/translation-tables/fertigkeiten.md:26` gives
   `| Awareness | Allgemein | Aufmerksamkeit | |`.
2. **The app already follows it**: `rules/i18n/de/abilities.json`'s
   `ability.awareness` is named **"Aufmerksamkeit"**.
3. `locales/de/main.ftl` gives `characteristic-per = Wahrnehmung`.

So the description renders the *Characteristic* Perception and the *Ability*
Awareness with the same root word, and tells the German player to roll
"Wahrnehmung + Wahrnehmungs-Fertigkeit" — a pairing that names nothing on his
own character sheet, where the Ability is called Aufmerksamkeit. This is worse
here than it would be elsewhere because the entry is `uncomputed_rule`: the
`description` is the **only** channel the rule has.

Measured, not assumed: "Wahrnehmungs-Fertigkeit" occurs in
`rules/i18n/de/virtues_flaws.json` exactly **once**, in this entry. It is an
isolated defect, not a sweep.

**A smaller sibling family, recorded here rather than filed separately.** Three
entries render "Awareness rolls" as "Wahrnehmungswürfe" instead of
"Aufmerksamkeitswürfe" — `virtue.finding_hidden_loot` (**in this batch**, see
F-86), `virtue.just_an_instant` and `virtue.skilled_smuggler`. Same table
violation, one degree less confusing because no "Wahrnehmung +" stands beside
it.

**The Q-10 tension applies and is not resolved here.** In every case the German
i18n faithfully reproduces the German *rulebook*, which CLAUDE.md makes the
source of truth for German text, while contradicting the *tables*, which the
same file makes canonical for terminology. This is the third instance of Q-10 in
two batches (after `virtue.covenfolk` and F-85). It differs from those two in
one respect that may well decide it: here the table is corroborated by the app's
own shipped `abilities.json`, so following the rulebook makes the application
internally inconsistent rather than merely unidiomatic.

**Correct value:** both occurrences → "ein Wahrnehmung + Aufmerksamkeit +
Stresswürfelwurf", subject to Q-10.

**Severity:** text — localization, rated up per CLAUDE.md; for the German locale
alone it is arguably lost-rule, since the roll cannot be constructed from the
text as written

---

### F-89 — `virtue.ferocity` — the unmodelled "animals only" is not merely a lost restriction; it makes the Confidence grant double-count

**Passage** (ArMDE:3873-3875, verbatim, English — descriptor line and body):
> #### Ferocity
> *Minor, General, animals only*<br>
> Like companion and magus characters, this character has Confidence points. However, these Confidence points may only be used in situations where its natural animal ferocity is triggered, such as when defending its den or fighting a natural enemy. Describe a situation which activates the Confidence for its species, and take 3 Confidence points and a Confidence Score of 1 to use when those circumstances are met.

German, ArMDE:3873-3875 (line-parallel):
> *Klein, Allgemein, nur Tiere*<br>
> Wie Gefährten- und Magus-Charaktere besitzt dieser Charakter Selbstvertrauenspunkte. … nimm 3 Selbstvertrauenspunkte und einen Selbstvertrauenswert von 1 …

**Current data:** `categories: ["general"]`, `entity_kinds: ["character"]`,
`classification: creation_effect`, one effect
`{"type":"confidence_bonus","score":1,"points":3}`. Nothing expresses "animals
only", and neither locale's summary mentions animals.

**Why it is wrong.** I cleared this entry because the two numbers match the
passage exactly, and they do. What I did not do is ask **who can select it**.

`rules/core/character_types.json`, read at the source: **companion, magus and
mythic_companion each carry `confidence_score: 1` and `confidence_points: 3`**;
grog carries neither key. A15's arithmetic is
`score = clamp(base_score + Σ score)` over the type profile's base. And
`general` is in the `permitted_categories` of all four types. So a **companion**
who takes Ferocity in the app today is computed at **Confidence Score 2 with 6
points**.

The passage's own first sentence is the proof that this is wrong rather than
merely generous: "**Like companion and magus characters**, this character has
Confidence points" — the Virtue exists to give an animal what a companion
already has. On a companion it is a pure double-count, and the sheet prints a
number the rules do not give.

This is what separates Ferocity from B02's Q-11 case. On
`virtue.domestic_animal` the unmodelled "animals only" loses a restriction and
computes nothing; here it produces a **wrong number** on a legal, reachable
character. The descriptor line carries the tag in exactly the position where
`Tainted` sits on other entries — and `tainted` *is* a modelled field
(`PointItem::tainted`), which is the sharpest available argument that this tag
is data and not editorial aside. The book does build animals with Virtues
(ArMDE:18378, ArMDE:18389: "A creature may have the Minor Virtue Increased
Characteristics … just like a human character"), so the restriction bounds a
real character class.

**Correct value:** depends on Q-11's ruling, and the two branches differ in
cost. If "animals only" gets a real restriction axis, Ferocity becomes
unselectable by the three types that already have Confidence and the
double-count disappears with it. If it does not, the narrower fix is to stop
this effect stacking on a base the profile already supplies. Either way the
phrase should reach both locales' summaries so the player is told. **I do not
pick**; the arithmetic defect is the finding, the remedy is Q-11's.

**The "animals only" tag appears on exactly three catalogue entries** —
`virtue.domestic_animal` (ArMDE:3700), `virtue.ferocity` (:3874) and
`flaw.companion_animal` (:5806) — and all three model it with nothing. Ferocity
is the only one of the three carrying an effect that collides with a type-profile
default, which is why it is the only one that is a finding rather than a
question.

**Severity:** wrong-rules-output

---

## Directed read — `README.md`'s open question 3: is a granted Reputation's `score` a ceiling, a fixed value or a suggestion?

`README.md` parks this as a Phase-0 open question for "the batch that reaches
it", citing `ArMDE:2512-2514` as unread. `virtue.famous` is this batch's
`grants_reputation` carrier, so the passage was read. It is short:

**ArMDE:2512-2514, verbatim:**
> ### Reputations
>
> Characters only start with a Reputation if they choose a Virtue or Flaw that grants one, but all characters can develop them in play. See page 28 for rules on Reputations.

**ArMDE:2518, the worked example immediately below it, verbatim:**
> Darius does have a Reputation, thanks to his Hermetic Prestige Virtue. It's a reputation with Hermetic magi, and it has a level of 3. Niall picks 'Dedicated Hoplite' as the content.

**ArMDE:3863, `virtue.famous`'s whole body, verbatim:**
> You have a good Reputation of level 4. Choose any reputation you like (it need not be justified), and one type.

German, ArMDE:3863 (line-parallel):
> Du hast eine gute Reputation der Stufe 4. Wähle eine beliebige Reputation (sie muss nicht gerechtfertigt sein) und einen Typ.

**What this settles, and what it does not.** :2514 answers a *different*
question from the one asked — it says a Reputation must be granted to exist at
creation (which is the rule `validate_reputations` already enforces) and says
nothing about the level. The level question is answered only by the individual
grants' own wording, and both samples here are **declarative, not permissive**:
"You *have* a good Reputation of level 4", and the example character "has a
level of 3" matching his Virtue exactly. Neither says "up to" or "no more than".
So for these two the score is a **fixed value**, and C2's observation that
`validate_reputations` compares kinds and counts only — so a player may enter
Local 10 backed by a grant of 4 — is a real permissiveness, not a deliberate
ceiling reading.

I stop there deliberately. Two samples do not settle 33 uses; a grant elsewhere
in the catalogue may word its level as a maximum, and the batch that meets one
should say so. What is now established is that **nothing at ArMDE:2512-2514
makes the score advisory**, which was the only reading that would have justified
ignoring it.

## Negatives cleared

Recorded so the corrections pass and later batches do not re-open them.

- **`virtue.focus_power` is fully implemented, including the parts that look
  uncomputed.** Every number in its twelve-line passage is modelled:
  `focus_points 25` against A28's `focus_points_used = Σ(2 × max_level +
  penetration)` is ArMDE:3899's "2 points to raise the maximum level of effect by
  1, and 1 point to raise the Penetration by 1"; `derived/focus_power.rs`
  computes the Initiative as Quickness − the maximum magnitude (:3899) and the
  1/2/3-Fatigue bands (:3901); and `max_per_target: 255` is :3903's "may be taken
  more than once, and the points gained may be combined" — correct precisely
  because the item has no parameters, so B10's `(item_ref, whole params map)` key
  makes every copy one target. The power's narrow scope is recorded as
  `FocusPower::name` on the entity, so the choice ArMDE:3897 demands is captured
  — it simply is not a Virtue parameter. **The one thing not carried is the realm
  association** (ArMDE:3905, "This Virtue may be associated with any supernatural
  realm … The power must be associated with the same supernatural realm as the
  system on which it is based"): `FocusPower` has no realm field. That is an
  instance of the known ArMDE:2960-2962 pattern, noted per the brief and not
  re-argued.
- **`virtue.faerie_doctor`'s four incompatibilities are sourced, and its
  apparently-duplicated grant is safe.** ArMDE:2637, verbatim: "All Mythic
  Companions take a Free Virtue which specifies their status. These Virtues are
  incompatible with each other, and with The Gift, and are not available to
  grogs." The four ids listed (`devil_child`, `nephilim`, `spirit_votary`,
  `the_gift`) are exactly that sentence, and all four declare it back
  symmetrically. The `grants_selection: ["virtue.dowsing"]` is ArMDE:3823's "You
  get the Dowsing Virtue for free" and ArMDE:2638's "You gain a free Minor Virtue,
  normally specified by the Mythic Companion Virtue". `mythic_companion_types.json`
  *also* grants both `virtue.faerie_doctor` and `virtue.dowsing`, which looks like
  a double grant and is not: A18 records that `vf_granted_selections` scans bought
  selections only, so a **granted** Faerie Doctor's own `grants_selection` is not
  expanded. The required Virtues and Flaws of ArMDE:2681-2692 (Wise One,
  Curse-Throwing, Faerie Friend, Dutybound) live on the type profile, which is
  where C7 says that kind of rule belongs. Its summary's "who mediates between
  humans and the fae" is **not** invented: ArMDE:2670 says the faerie doctor "may
  act as an intermediary between the humans and the fae" and "attempts to
  mediate". Only the free Dowsing is absent from the summary, and it is computed
  and displayed as a granted selection, so nothing is lost.
- **`virtue.faerie_magic`'s prerequisite and grant are correct on the
  `virtue.the_enigma` precedent — but this bullet covered only half the entry,
  and the other half was overturned (F-87).** What follows stands; what it does
  not reach is the "(see page 236)" cross-reference.
  ArMDE:3827 words House membership as a consequence ("initiated into the Outer
  Mystery of Faerie Magic … **and thus are** a member of House Merinita"), the
  data encodes it as `prerequisites: {house: house.merinita}`, and B02 already
  cleared exactly this construction: `rules/core/houses.json` grants
  `virtue.faerie_magic` from `house.merinita`, and B6's tri-state makes `house`
  yield `Unknown` (no issue) for a character with no House, so the two readings
  admit the same set of characters. `ability.faerie_magic` is category `arcane`
  and therefore gated, which makes the `ability_score_grant`'s A9 permission
  side-effect load-bearing rather than decorative.
- **`virtue.extractor_of_form_vis`'s multiplicity is right.** ArMDE:3781, "This
  Virtue may be taken multiple times (once for each Form)", against ArMDE:2814's
  default ("A Virtue or Flaw may be taken more than once only if the description
  explicitly allows it"). The default `max_per_target: 1` is keyed on the whole
  params map, so two copies naming different Forms are two targets and legal,
  while two naming the same Form are one target and blocked — which is "once for
  each Form" exactly. The `form`-domain parameter is consumed by no effect, which
  B8 records as the normal shape for a parameter that records a player choice.
- **`virtue.free_study`'s data is a faithful encoding.** ArMDE:3939, "Add +3 to
  the Source Quality when studying from raw vis", against
  `{"type":"advancement_mod","source":"vis","amount":3}`. `advancement_mod` being
  surfaced-only is C1's systemic gap, not this entry's defect.
- **`virtue.ferocity`'s numbers are exact — and that was the wrong question.**
  ArMDE:3875's "take 3 Confidence points and a Confidence Score of 1" does match
  `{"type":"confidence_bonus","score":1,"points":3}` exactly. But "A15 adds each
  to the type profile's base" is the clause I wrote down and did not follow up:
  three of the four profiles supply that base already. **Overturned — F-89.**
- **`virtue.famous`'s wildcard is right.** ArMDE:3863 says "Choose any reputation
  you like … and one type", and A25 records that an omitted `kind` is precisely
  the player-chosen-type wildcard. `score: 4` matches "level 4"; that the score is
  unenforced is C2, not this entry's defect (see the directed read above).
- **The five `uncomputed_rule` entries with full descriptions and nothing
  computable** — `exotic_casting`, `eye_of_hephaestus`, `fabric_ripper`,
  `falls_like_a_cat`, `flexible_formulaic_magic`. The *classification and
  completeness* half of this bullet stands for all five; `eye_of_hephaestus`'s
  German **terminology** was overturned inside it (F-88), which is precisely the
  "check the German term by term" step this bullet claims to have done and did
  not do thoroughly enough. Each carries its whole passage
  in `description` in both locales, each states rules the engine has no variant
  for (an Ease Factor exception for Form identification; two Perception+Awareness
  Ease Factors; a PeAn(He) 25 effect at Penetration +0; a fall-damage table; a
  magnitude-for-parameter trade on Formulaic casting), and none carries `effects`.
  German terminology checked term by term against the tables in each: the one that
  looked like B02's F-65 shape is `fabric_ripper`'s "die übernatürliche
  **Fähigkeit**, jedes gefertigte Ding zu zerreißen", and it is **correct** — the
  English is lowercase "the supernatural ability to tear", the ordinary word, not
  the game term, and the German rulebook at :3791 writes it the same way.
  (Its *name* is a separate matter — F-85.)
- **`virtue.factor` and `virtue.gentleman` are correctly `narrative`.**
  ArMDE:3795's whole body is a social position plus a pointer to *City and Guild*
  plus "Many factors are junior partners in their companies, and they choose the
  Partner Virtue, not this one" — advice about which Virtue to pick, not an
  incompatibility, and `virtue.partner` exists so the distinction matters.
  ArMDE:3961's body is entirely about family, address and expectations, and closes
  with "The Wealthy Virtue and Poor Flaw affect you normally", which B02 settled
  as a statement that *no* special rule applies. Neither grants an Ability
  category, which is what separates them from F-66/F-67/F-68 three rows away.
- **`virtue.free_expression` is correctly `uncomputed_rule` with its number in
  the text.** ArMDE:3935's "+3 bonus on all rolls to create a new work of art"
  names no Ability, so it is not even an `ability_roll_mod` candidate; the
  description carries it verbatim in both locales.
- **All twelve anchors and all 35 `source` ranges.** Checked individually against
  the heading each claims and the heading below it; listed in the header rather
  than repeated here.

## Open questions

### Q-17 — `virtue.gender_shift` and `virtue.feather_messenger` — is a supernatural power described only in prose, with constraints but no number, `narrative`?

**Passages** (verbatim, English — each entry's whole body):

ArMDE:3953:
> Each midnight, the character may choose to change genders. The character's male and female forms are consistent across transformations, and usually appear to be blood kin of each other. The character's Personality Traits may vary slightly between forms. Pregnant characters may not use this ability.

ArMDE:3871:
> This Virtue is only available to a character who can take the form of a bird (which may be her natural form). She can painlessly separate a feather from her body, and use it to write as a quill, controlling its movements telepathically while it remains within sight. The quill doesn't need ink; it provides it magically. After she finishes writing, the character can reattach the feather if in bird form. The quality of the character as a scribe does not differ between human and bird forms. The character can also pull off feathers in bird form, turn into a human, and still mentally control the dropped feathers. This Virtue is particularly associated with a north African magical tradition called the Daughters of Four Fathers (see *Between Sand & Sea*, page 107).

German, ArMDE:3953 and :3871 are line-parallel and say the same.

**Current data:** both `"classification": "narrative"`, no effects, no
description, summary = the passage's first sentence in both locales.

**Reading A — `narrative` is right.** Neither passage contains a number, a
signed modifier, a roll, an Ease Factor or a cap. What they contain is a
description of what the power *does*, which is colour by any ordinary reading,
plus a few limits that are limits on the fiction rather than on any total.

**Reading B — `uncomputed_rule`.** Each states constraints on *using* a
supernatural power that a storyguide would have to adjudicate: Gender Shift is
usable **only at midnight**, **once** per occasion, and **not while pregnant**;
Feather Messenger works **only while the quill is within sight** and the feather
can be reattached **only in bird form**. B02 put `virtue.dust_devil` at
`uncomputed_rule` for "a shapechange with a focus object" and
`virtue.command_animals` for "a count of commandable animals", neither of which
is a signed modifier either. On that calibration these two belong there too, and
each would owe a description in both locales.

**What I could not settle it with:** nothing in `README.md`, `decisions.md` or
`engine-semantics.md` § B1 draws the line between "a constraint on using a
power" and "colour". B1's own examples of the `narrative` trap are all numeric
(a signed modifier, a botch-dice change, a cap), which is no help for a passage
that has no numbers at all. **This is the third member of a family with Q-09**
(is a guaranteed storyguide intervention mechanical?) **and it should be
answered with Q-09, because the answer is the same answer.** Both entries are
marked `?` and neither classification is moved.

Feather Messenger additionally carries B01's **Q-05** eligibility shape, and in
its strongest form yet: "only available to a character who can take the form of a
bird" is stated in the **rules text**, not on the descriptor line as the
male-only and animals-only cases were. Q-05 governs it; no new question.

---

### Q-18 — `virtue.gentle_gift` — is the `has virtue.the_gift` prerequisite a requirement the book actually states?

**Passage** (ArMDE:3957, verbatim — the entry's whole body):
> Unlike other magi, whose Magical nature disturbs normal people and animals, your Gift is subtle and quiet. You do not suffer the usual penalties when interacting with people and animals.

**And the passage that complicates it**, ArMDE:4139 and :4141, inside
`#### Inoffensive to (Beings)`, verbatim:
> You may not take this Virtue more than once; characters who are Inoffensive to more than one type of being should take Gentle Gift instead. UnGifted characters may take this Virtue only if they have the Flaw Magical Air.
>
> Note that Inoffensive to Mundane Humans is not available as a Minor Virtue; take Gentle Gift instead.

**Current data:** `prerequisites: {"kind":"has","value":"virtue.the_gift"}`.

**Reading A — the prereq is right.** ArMDE:3957 is addressed to a magus
throughout ("Unlike other magi … your Gift"), and ArMDE:8749 names Gentle Gift as
a thing "some Hermetic magi have". Nothing in the Virtue's own passage offers it
to the unGifted.

**Reading B — the prereq is over-strict.** Read ArMDE:4139 and :4141 together:
an **unGifted** character with Magical Air may take Inoffensive to (Beings); a
character inoffensive to more than one type "should take Gentle Gift instead";
and Inoffensive to Mundane Humans is unavailable, "take Gentle Gift instead". The
book therefore routes at least some unGifted characters to Gentle Gift, which a
`has virtue.the_gift` prerequisite makes impossible — `prereq_not_met` is an
**error**, not a warning. Check 7 forbids a requirement the passage does not
state, and the Virtue's own passage states this one only by implication.

**What I could not settle it with:** the routing at :4139/:4141 is an inference
from two sentences about a *different* Virtue, and it may simply be loose
writing that assumes the reader has The Gift. I could find no sentence anywhere
saying either "Gentle Gift requires The Gift" or "an unGifted character may take
Gentle Gift". The classification move in F-81 does not depend on this and stands
either way.

---

### Q-19 — may the catalogue assert an incompatibility that no passage states, when the pair is logically contradictory?

**The instance.** `virtue.gentle_gift` declares
`incompatible_with: ["flaw.blatant_gift"]` and `flaw.blatant_gift` declares it
back (symmetric, so the load passes). **Neither passage states it.**

ArMDE:3957 (Gentle Gift, whole body) is quoted in Q-18 and F-81 and names no
Virtue or Flaw. ArMDE:5713 (Blatant Gift, whole body), verbatim:
> People immediately realize that there is something strange about you, even if they do not know you are a magus. Animals are extremely disturbed, frightened, and possibly enraged by your presence. You suffer a -6 penalty on all interaction rolls with normal people and animals, and should see page 203 for further discussion of this Flaw's effects.

ArMDE:8749 and :8753 discuss the two together but never as an exclusion. The
book *does* state exclusions of this shape when it means them — ArMDE:5717,
"This effect is the same as the Blatant Gift, and a character may not have both
Flaws"; ArMDE:6895, "it cannot be combined with the Blatant Gift" — which makes
the silence at :3957 and :5713 meaningful rather than accidental.

**Reading A — it is a defect (an unstated requirement, check 8's mirror of
check 7).** "Source or nothing" applies to a negative constraint as much as to a
positive one. The audit already treats an unsourced clause in *text* as a defect
(B02's F-55 flagged a summary "mentioning an incompatibility the passage does not
state"); this is the same thing in the data.

**Reading B — it is correct and needs no passage.** The two are definitionally
contradictory — a Gift cannot be both "subtle and quiet" and "-6 on all
interaction rolls" — so the pair is entailed by the two descriptions even though
neither names the other, and removing it would let a save hold both.

**What I could not settle it with:** the audit has no stated policy on entailed
constraints, and the answer generalises well beyond this pair — it decides how
every "obviously contradictory" pair in the catalogue is to be judged for the
rest of the sweep, which is why it is raised as a general question rather than
as a finding against one entry. I did not count how many such pairs exist; that
measurement belongs with the ruling.

---

### Q-20 — `virtue.fidai` — does "choose the Social Status he is pretending to have" permit a second Social Status?

**Passage** (ArMDE:3881, verbatim, English — the entry's second paragraph):
> This Social Status Virtue may be taken by a character who is living far from the home of the Nizaris, as the assassins are sent on missions, and may be sent far away. Such a character should have a Story Flaw representing his mission, and choose the Social Status he is pretending to have. See *The Cradle and the Crescent*, from page 162, for more detail on the Nizaris.

German, ArMDE:3881 (line-parallel):
> … Ein solcher Charakter sollte einen Geschichte-Fehler haben, der seine Mission darstellt, und den Sozialen Status wählen, den er vorgibt zu haben. …

**And the rule it runs into**, ArMDE:2816, verbatim:
> All characters must take one Social Status, and may only take more than one if the descriptions of the Virtues or Flaws explicitly note that they are compatible.

**Reading A — no.** The pretended status is *fiction*: the character claims to be
a merchant, he does not hold the Merchant Virtue. Nothing is taken and :2816 is
untouched.

**Reading B — yes, and this is the "explicitly note" escape hatch.** :2816 says a
second Social Status is legal when a description says so, and this description
tells the player to choose one. On that reading `virtue.fidai` is one of the rare
entries that licenses a second Social Status and the data would need to say so —
except that the engine has no field for "this Social Status is compatible with
another", which would make it D3 text instead.

**What I could not settle it with:** the sentence says "choose", which is the
book's ordinary verb for taking a Virtue, but it also says "pretending", which is
the book's ordinary verb for fiction. This is the first instance of the
ArMDE:2816 pattern in this batch and the brief asks for instances to be noted
rather than the rule re-argued — but the *ambiguity* is new, so it is raised.

---

### Q-21 — `virtue.forge_companion` — is "The character is an unGifted craftsman" an encodable incompatibility with The Gift?

**Passage** (ArMDE:3927, first sentence, verbatim):
> The character is an unGifted craftsman attached to a Verditius magus and working for him in his lab.

German, ArMDE:3927 (line-parallel):
> Der Charakter ist ein unBegabter Handwerker, der einem Verditius-Magus angegliedert ist und für ihn in seinem Labor arbeitet.

**Current data:** no `incompatible_with`, no `prerequisites`.

**Reading A — it should carry `incompatible_with: ["virtue.the_gift"]`.**
`virtue.failed_apprentice`, four entries earlier in the same span, carries
exactly that for ArMDE:3845's "You may not have The Gift" — and a Forge-Companion
*is* unGifted by definition, so the set of legal characters is identical. The
data should not depend on whether the book phrased the same restriction as a
prohibition or as a definition.

**Reading B — it should not.** "You may not have The Gift" is a rule addressed to
the player; "is an unGifted craftsman" is a description of the archetype, in the
same grammatical register as "attached to a Verditius magus" — which nobody would
encode. Encoding it would be reading a constraint into flavour, which is the
failure mode `7f5605a` was reverted for.

**What I could not settle it with:** the two readings turn on a distinction
between prescriptive and descriptive phrasing that the audit has not ruled on,
and it is the same axis as **B02's Q-13** ("does the book distinguish 'may only
**take**' from 'may only **have**'?"). It should be answered with Q-13.

---

### Q-22 — where does an *uncomputable* rule go on an entry that already carries an `Effect`?

*Raised by the verification pass, which hit it while trying to state F-87's
correct value and found there was nowhere to put the rule.*

**The structural problem.** `engine-semantics.md` § B1 records that between
`uncomputed_rule` and `creation_effect`/`in_play_effect`, "carrying an `Effect`
is what decides it". The obligation to write a rule out in `description` in
every locale is attached to the **class** `uncomputed_rule`, and
`crates/arm-rules/tests/uncomputed_clauses.rs` enforces it only for that class.

So an entry that computes *one* of its mechanics and cannot compute another
falls into a hole: it is not `uncomputed_rule`, therefore nothing obliges the
second mechanic to be carried, and no guard ever looks at it. The rule leaves
the application while the entry's data looks complete — which is the exact
failure mode this audit was created to find, generalised from one entry to a
whole class of entries.

**Three readings, none of which I can pick from the source:**

- **(a)** The summary absorbs the uncomputed clause. Cheapest, needs no new
  field or practice; but a summary is one sentence and F-87's residue is three
  rules.
- **(b)** `creation_effect` entries may carry a `description`, as nine of 93
  `in_play_effect` entries already do. This is the same question as **Q-14**
  approached from the other side: Q-14 asks *how much* residue triggers the
  obligation, Q-22 asks *whether the class permits carrying it at all*. Today
  **0 of 125** `creation_effect` entries have one, so this is a new practice.
- **(c)** The partition needs a fifth state, or `uncomputed_clauses.rs` needs to
  check displayed text on any entry whose passage outruns its effects — which is
  not mechanically detectable, and is why the guard is class-keyed today.

**Instances in this batch alone:** F-76 and F-78 (`in_play_effect`, where
reading (b) is already established practice), F-83 and F-87
(`creation_effect`, where it is not), and F-70 (`in_play_effect`, the largest
residue of the five). **This should be ruled on together with Q-14** — they are
one decision with two halves, and five of this batch's findings are parked on
it.

## Sub-agent reconciliation

One verification pass was run against the 14 verdicts I had marked clean on all
three axes. It was given the twelve checks, the open patterns, `decisions.md`
and `engine-semantics.md`, and was **not** allowed to read this file; it was
told that overturning a verdict would be a success and inventing one would not,
and that it could spawn no agents of its own and change no file.

**Result: 11 confirmed, 3 overturned**, plus one open question I had not raised
(Q-22) and four incidentals. It confirmed changing no file, running no
git-mutating command, spawning no sub-agent, and not reading `batch-03.md`; and
it reported receiving the "auto mode" Bash injection and overriding it per
CLAUDE.md, as I did.

### The three overturns, and what I re-checked before accepting them

I did not take any of the three on trust. Each claim was re-derived against the
files before it was written up above:

| Overturn | What I re-verified, and where |
|---|---|
| **F-87** `virtue.faerie_magic` | Opened ArMDE:9961-9971 and ArMDE:9626. Both quotations are accurate, and :9626's "**the Faerie Magic Virtue** attunes Merinita magi" is the sentence that makes this the Virtue's rule rather than the Mystery chapter's — which is what turns a plausible reading into a finding. |
| **F-88** `virtue.eye_of_hephaestus` | `fertigkeiten.md:26` gives `Awareness → Aufmerksamkeit`; `rules/i18n/de/abilities.json`'s `ability.awareness` is `"Aufmerksamkeit"`; and `grep -c "Wahrnehmungs-Fertigkeit" rules/i18n/de/virtues_flaws.json` returns **1**, confirming both the violation and its isolation. |
| **F-89** `virtue.ferocity` | Read `rules/core/character_types.json` directly: companion, magus and mythic_companion each carry `confidence_score: 1` / `confidence_points: 3`, grog carries neither, and `general` is in all four `permitted_categories` lists. The double-count is arithmetic, not inference. |

### Why I missed each one — recorded so the next batch does not repeat it

- **F-87.** I cleared `virtue.faerie_magic` by pattern-matching it to B02's
  cleared `virtue.the_enigma`, which settles the prerequisite-as-consequence
  question — and then did not apply my own brief's "follow every cross-reference"
  rule to the "(see page 236)" in the very sentence I was reading. The pattern
  match substituted for the check. Worse, I *did* follow the cross-references in
  `virtue.faerie_raised_magic` (F-77) and `virtue.ghostly_warder` (F-82) in the
  same pass, so this was not ignorance of the rule but a lapse in applying it to
  an entry I had already decided was clean. **The lesson generalises: a page
  reference inside a passage is a pointer to follow even when — especially when —
  the entry otherwise looks finished.**
- **F-88.** My negatives-cleared bullet claims German terms were "checked term by
  term against the tables", and for `fabric_ripper` they were (that check
  produced F-85). For `eye_of_hephaestus` I read the German description, noticed
  "Wahrnehmung + Wahrnehmungs-Fertigkeit" was odd, confirmed it matched the
  German rulebook at :3785 verbatim, and stopped there — treating "matches the
  source of truth" as the end of the check when the tables are the canonical
  authority for terminology and the app's own `abilities.json` was one lookup
  away.
- **F-89.** I checked the effect's two numbers against the passage and they
  matched, then wrote "A15 adds each to the type profile's base" without opening
  the type profiles. The descriptor tag "animals only" I logged as a Q-11
  instance — a note, not a defect — and that framing is what stopped me asking
  the next question, which is *who can select this*. **An unmodelled restriction
  is not automatically inert: where the entry carries an effect, ask what the
  effect does on the characters the restriction was supposed to exclude.**

### Negatives the pass cleared independently, recorded so the corrections phase does not reopen them

Its confirmations were not bare. Among the checks it ran that I had not:

- **`virtue.factor`** — it followed the "they choose the Partner Virtue, not this
  one" pointer to ArMDE:4616-4618 and found Partner says a partner "**need not**
  purchase that Virtue", never "may not" — so the *absence* of an
  `incompatible_with` is positively correct, not merely unexamined.
- **`virtue.gentleman`** — it checked "The Wealthy Virtue and Poor Flaw affect
  you normally" against the whole family before accepting B02's Q-15 reading,
  and found the book **deviates** from the stock sentence where it means
  something (ArMDE:3689 adds a Bad Reputation 2 for a poor scholar; :4858 says
  the two "are unlikely to be appropriate"; :4293 and :6677 state explicit
  compatibilities). Gentleman's is the null form, so it obliges no data — which
  is a much stronger clearance than "B02 said so".
- **`virtue.exotic_casting`** — it enumerated all eleven `SpecialCasting` kinds
  from `types.rs` to confirm none fits the Ease-Factor-15 rule, rather than
  assuming.
- **`virtue.faerie_doctor`** — it read all four Mythic-Companion Virtues' lists
  to verify the incompatibility symmetry by hand, and confirmed the
  `grants_selection` is not caught by A18's param-less-grant trap because
  `virtue.dowsing` declares no parameters.
- **`virtue.free_study`** — it pulled all eleven `advancement_mod` carriers and
  found every one uses a number-free first-sentence summary with no
  `description`, so the omission is uniform practice rather than entry-level rot.
- **`virtue.focus_power`** — it independently reached the same conclusion I did,
  including that `derived/focus_power.rs` computes the Initiative and the
  Fatigue bands, and that the missing realm association is the ArMDE:2960
  pattern.
- **Catalogue-wide:** it re-ran the math-minus check itself
  (`grep -oa "−"` over both locales' `virtues_flaws.json` → 0 hits).

### Incidental, outside this batch — recorded so they are not lost

1. **The Merinita starting Warping Point is modelled nowhere, and is
   expressible.** ArMDE:9965: "Merinita characters without a faerie Virtue or
   Flaw start with one Warping Point, caused by their parens to qualify them for
   the mystery." `rules/core/houses.json`'s `house.merinita` carries only the
   fixed grant of `virtue.faerie_magic`, and `crates/arm-rules/RULES.md` has no
   entry for it. `Entity::warping_points` exists and `Effect::WarpingGrant`
   carries points, so this is a plain omission at the **House** layer, not an
   engine limit. It belongs to no V/F batch and would otherwise fall between
   them.
2. **`virtue.wise_one` (ArMDE:5259, B10's range)** — "You may take either Arcane
   or Academic Abilities, but not both" is `narrative` with no effects, so both
   the permission *and* the exclusive choice are unrecorded. B10 should judge it
   on its own terms; flagged only so the exclusive-choice half is not overlooked
   when the permission half is fixed.
3. **The three Outer-Mystery siblings** — `virtue.the_enigma`,
   `virtue.heartbeast`, `virtue.verditius_magic` — carry F-87's cross-reference
   shape. Recorded in F-87 itself; repeated here because one of them was cleared
   by an earlier batch.
4. **All four Mythic-Companion status Virtues' German summaries** write
   "mythischen Gefährten" in lower case where `grundbegriffe.md:83` gives
   "Mythischer Gefährte" and ArMDE:3823 capitalises it. Consistent across all
   four, so a style choice rather than drift — not filed.

### The pass's own open questions, and where each landed

- Its **Q1** (where an uncomputable rule goes on an effect-bearing entry) is new
  and is filed above as **Q-22**.
- Its **Q2** ("is 'animals only' in scope at all?") is **Q-11**, sharpened: it
  supplies the second reading — fix the stacking without inventing an axis — that
  Q-11 did not have, and F-89 records both branches.
- Its **Q3** (should the German *source* be corrected alongside the i18n?) is
  **Q-10**'s territory; folded into F-88 rather than filed separately, with the
  one new argument it contributes — that here the table is corroborated by the
  app's own shipped `abilities.json`.

