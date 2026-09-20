# Batch B01 — indices 0-34, ArMDE:3362-3562

Entries: 35. Audited: 35. Failures: 25. Findings: 33. Open questions: 7 live
(Q-01, Q-03, Q-04, Q-05, Q-06, Q-07, Q-08) + 1 closed by D3 (Q-02 → F-32).
An independent verification pass re-derived my 13 clean verdicts and overturned
one (`virtue.bee_king` → F-33); see "Verification pass" at the end.

**Read first: this batch contradicts `decisions.md` D1 on both of the entries D1
names in it.** See "Decisions applied" immediately below.

Method note: the whole span was read as continuous prose in **both** languages
(`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` ArMDE:3356-3569
and the line-parallel German file), before any entry was judged. Every
`source.lines` range in the batch was checked against the heading it claims and
the next heading below it; **all 35 bracket their passage correctly** (some
include the trailing blank line before the next `####`, some do not — neither
bisects an entry nor bleeds into a neighbour, so none is reported). The five
entries carrying an `anchor` (`alluring-to-beings`, `aptitude-for-sin`,
`atlantean-magic`, `boosted-magic`, `capo`) all name their own heading correctly.
No U+2212 occurs anywhere in either locale's `virtues_flaws.json`.

Part C systemic gaps are **not** re-reported per entry. In particular
`advancement_mod` and `ability_roll_mod` being surfaced-only (C1) is treated as
correct authoring, not as a defect of `virtue.apt_student`,
`virtue.book_learner` or `virtue.academic_concentration_subject`.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D1 — and B01 contradicts it on both of the entries it names here.** D1 rules
that the *unconditional* `lab_total_mod` entries belong in
`effective/spell.rs::spell_level_cap`, and lists
`virtue.adept_laboratory_student` and `virtue.aristotelian_training` among the
five "yes, unconditional" rows. **Both of those passages are conditional**, and
the coordinator asked to be told if they were. See **F-04** and **F-15** for the
verbatim sentences; the short form is:

| Entry | D1 says | ArMDE says |
|---|---|---|
| `virtue.adept_laboratory_student` | unconditional | ArMDE:3370 — "+6 bonus to Lab Totals **when working from the lab texts of others**, including when reinventing spells" |
| `virtue.aristotelian_training` | unconditional | ArMDE:3442 — "A **magus** with this Virtue may add +1 to his Lab Totals **if attempting to synthesize the New Aristotle with Magic Theory**" |

So two of D1's five "yes" rows rest on a reading the source does not support, and
both belong on the "no" side for the same reason Potent Magic does — they apply
only within a stated scope. The consequence runs the way D1 cares about: adding
Aristotelian Training's +1 to the creation-time spell-level cap would credit a
spell-invention total with a bonus the book grants only for reconciling Aristotle
with Magic Theory, which is not spell invention at all; and Adept Laboratory
Student's +6 would apply to inventing from scratch as readily as to working from
a text. I have **not** marked either entry's `effects` mis-implemented *on the
spell_level_cap account* — that defect is central and recorded once — but the
conditionality defect is a separate axis and is reported (F-04, F-15). It is also
a second instance of the very error D1's own closing note raises about
`derived/lab.rs::lab_totals`.

**D3's precedent is applied throughout.** An engine that structurally cannot
express a rule is grounds for `uncomputed_rule` with the rule written out in both
locales, never for `narrative`; `narrative` remains a claim that the book states
nothing mechanical. This settles F-12 (Amorphous) in the direction already
reached, and it closes **Q-02** — see **F-32**. It narrows but does not close
**Q-03**.

**D2 — nothing in ArMDE:3362-3562 bears on it.** No passage in this span says
anything about whether a House- or type-granted Virtue must satisfy the same
preconditions as a bought one. The nearest thing, offered as adjacent evidence
only and explicitly *not* as the passage D2 needs, is ArMDE:3515 inside
`virtue.blood_of_the_nephilim`: "…and have the Minor Personality Flaw Greedy
(**which counts as one of your normal Flaws**)." That is a Virtue mandating a
Flaw and saying the Flaw is charged to the normal allowance — a statement about
**budget** scope, not about precondition scope, and it cuts against the engine's
blanket budget-exemption for grants (B3) rather than for or against D2. Recorded
in case the budget question is ever asked; it does not answer D2.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.academic_concentration_subject` | 3362-3367 | OK | effects incomplete; `incompatible_with` missing; param unbounded | OK | F-01 F-02 F-03 Q-01 |
| `virtue.adept_laboratory_student` | 3368-3371 | OK | effects scope | OK | F-04 |
| `virtue.affinity_ability` | 3372-3374 | OK | OK | en+de summary misstates the age-cap rule | F-05 |
| `virtue.affinity_art` | 3376-3378 | OK | OK | OK | |
| `virtue.alim` | 3380-3383 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-06 |
| `virtue.all_according_to_plan` | 3384-3387 | OK | OK | OK | |
| `virtue.alluring_to_beings` | 3388-3395 | OK | param domain; prereqs + `incompatible_with` missing | de name `?` | F-07 F-08 Q-04 |
| `virtue.almogaten` | 3396-3403 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-09 |
| `virtue.almogavar` | 3404-3409 | narrative → creation_effect | effects missing; `incompatible_with` missing | OK | F-10 F-11 |
| `virtue.amorphous_major` | 3410-3413 | narrative → uncomputed_rule | OK | description missing both locales; Major/Minor indistinguishable | F-12 |
| `virtue.amorphous_minor` | 3410-3413 | narrative → uncomputed_rule | OK | description missing both locales; Major/Minor indistinguishable | F-12 |
| `virtue.animal_ken` | 3414-3417 | OK | OK | OK | |
| `virtue.apprentice` | 3418-3421 | `?` | OK | OK | Q-02 |
| `virtue.apt_student` | 3422-3425 | OK | OK | OK | |
| `virtue.aptitude_for_sin` | 3426-3429 | OK | OK | OK | |
| `virtue.arcane_lore` | 3430-3435 | OK | OK | two rules carried in neither locale | F-13 Q-03 |
| `virtue.archieunuch` | 3436-3439 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-14 |
| `virtue.aristotelian_training` | 3440-3443 | OK | effects scope; effects incomplete | OK | F-15 |
| `virtue.atlantean_magic` | 3444-3469 | OK | OK | OK | |
| `virtue.baccalaureus` | 3470-3475 | OK | pool instance scope | OK | F-31 |
| `virtue.rard` | 3476-3479 | OK | OK | en `name` is the scanno "Rard" | F-16 |
| `virtue.beadle` | 3480-3483 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-17 |
| `virtue.bee_king` | 3484-3499 | OK | OK | description missing both locales | F-33 Q-03 |
| `virtue.berserk` | 3500-3503 | OK | effects scope; effects missing | OK | F-18 F-19 F-20 |
| `virtue.blood_of_the_nephilim` | 3504-3518 | OK | effects missing; `incompatible_with` missing; type restriction unenforced | OK | F-21 F-22 F-23 F-24 F-25 |
| `virtue.book_learner` | 3519-3522 | OK | OK | OK | |
| `virtue.boosted_magic` | 3523-3528 | OK | OK | OK | |
| `virtue.brother_chaplain` | 3529-3532 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-26 |
| `virtue.brother_knight` | 3533-3536 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-27 |
| `virtue.brother_sergeant` | 3537-3540 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-28 |
| `virtue.bureaucrat` | 3541-3544 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-29 |
| `virtue.capo` | 3545-3548 | OK | OK | OK | |
| `virtue.cathedral_school_master` | 3549-3554 | OK | prereqs missing | OK | F-30 Q-03 |
| `virtue.cautious_sorcerer` | 3555-3558 | OK | OK | OK | |
| `virtue.cautious_with_ability` | 3559-3562 | OK | OK | OK | |

## Findings

### F-01 — `virtue.academic_concentration_subject` — the -1 penalty on the six non-concentrated subjects is implemented nowhere and stated in neither locale

**Passage** (ArMDE:3364, verbatim, English):
> He may add +3 to his Artes Liberales score for that subject. However, he must subtract 1 from Artes Liberales rolls and totals for the subjects he did not concentrate in. Include the bonus and penalty modifier in every total that Artes Liberales is used for, including writing books.

**Current data:** one effect, `{"type":"ability_roll_mod","param":"subject","amount":3}`.
No second row. The i18n `summary` in both locales stops at "…in preference to
the other six" and there is no `description` (classification is
`in_play_effect`, which carries no description obligation).

**Why it is wrong:** the passage states a *pair* of modifiers and explicitly says
both are to be included in every total. Only the bonus reaches the data, so the
read-out tells the user about the +3 and nothing at all about the -1. This is
check 10's "nothing in the passage left unimplemented" direction.

**Correct value:** the -1 must be represented. Note that a second
`ability_roll_mod` keyed on the same `subject` param would read as "-1 *in* the
chosen subject", which is the opposite of the rule — the modifier's scope is "the
six subjects that are *not* the param's value", which `AbilityRollMod` (A41,
free-text subject, no negation) cannot express. So the fix is either a new scope
on the variant or, at minimum, the rule written into `description` in both
locales.

**Severity:** lost-rule

---

### F-02 — `virtue.academic_concentration_subject` — the stated incompatibility with Puissant Artes Liberales is encoded nowhere

**Passage** (ArMDE:3364, verbatim, English):
> This Virtue is incompatible with the Virtue Puissant Artes Liberales. Puissant Artes Liberales means a character is particularly adept with all seven liberal arts, while Academic Concentration means the character focused on one liberal art in preference to the others.

**Current data:** no `incompatible_with` key at all.

**Why it is wrong:** the passage states an incompatibility and check 8 requires
every stated one to be present. Nothing in the app stops a character taking both.

**Correct value:** the target is `virtue.puissant_ability` **with its `ability`
param set to `ability.artes_liberales`** — there is no `virtue.puissant_artes_liberales`
id (confirmed: the catalogue holds `virtue.puissant_ability` and
`virtue.puissant_art` only). `incompatible_with` is a `BTreeSet<Id>` (B7), so it
cannot express a parameter-scoped exclusion, and listing bare
`virtue.puissant_ability` would wrongly forbid Puissant *anything*. The honest
minimum is to carry the clause in the i18n text; the enforceable fix needs a
parameter-aware incompatibility, which is a schema question.

**Severity:** lost-rule

---

### F-03 — `virtue.academic_concentration_subject` — the `subject` parameter is unbounded free text where the passage states a closed set and an explicit exclusion

**Passage** (ArMDE:3362, 3364, 3366, verbatim, English):
> #### Academic Concentration (Subject)
> The character has concentrated in one of the seven subjects of Artes Liberales, in preference to the other six.
> With troupe approval, this Virtue could be extended to Philosophiae. It is not allowed for any other Ability besides Artes Liberales and Philosophiae.

**Current data:** `{"key":"subject","type":"ref","domain":"text"}` — per B8, any
non-whitespace string is legal.

**Why it is wrong:** the book names a closed option set ("one of the seven
subjects") and then bans everything outside it ("not allowed for any other
Ability besides…"). A `text` domain accepts anything. The repo already uses the
`enumerated` domain for exactly this shape — `virtue.folk_magic`,
`flaw.offensive_to_beings`, `flaw.unbearable_to_beings` and
`virtue.inoffensive_to_beings` all carry closed `values` lists.

**Correct value:** `domain: "enumerated"` with the seven Artes Liberales subjects
plus Philosophiae. The seven are **not named inside this passage** — ArMDE:3442
names three of them in passing ("grammar, logic, and rhetoric") — so the exact
list must be sourced from the Artes Liberales Ability description before the fix
is written. See **Q-01**.

**Severity:** wrong-rules-output (an unbounded picker accepts an illegal subject)

---

### F-04 — `virtue.adept_laboratory_student` — a conditional +6 is encoded as an unconditional Lab Total modifier

**Passage** (ArMDE:3370, verbatim, English):
> You digest the instruction of others quite easily. You get a +6 bonus to Lab Totals when working from the lab texts of others, including when reinventing spells.

**Current data:** `{"type":"lab_total_mod","amount":6}`.

**Why it is wrong:** per `engine-semantics.md` A31, `lab_total_mod` is "a single
summed scalar, added into the base Lab Total … Summed, unconditional". The book
scopes the +6 to one activity (working from another's lab text). As authored, the
engine adds +6 to **every** Lab Total the character ever shows — inventing from
scratch, enchanting, longevity rituals, familiar bonding. This is a 6-point
inflation of the displayed Lab Total in the majority of cases, and the same
number also seeds `derived/lab.rs::creo_corpus_lab_total` (the Longevity hint) and
`masterpiece_item_cap`.

**Correct value:** the +6 must be gated on the activity. `LabTotalMod` carries no
scope field, so this is not expressible today — the defect is real either way and
is not one of the Part C entries.

**Against D1.** `decisions.md` D1 lists this entry as one of the five
*unconditional* `lab_total_mod` rows and rules it into
`effective/spell.rs::spell_level_cap`. The sentence quoted above does not support
that: "when working from the lab texts of others" is a scope, of exactly the kind
that put `virtue.potent_magic_*` on D1's "no" side. Reported here rather than
acted on — D1 is binding until amended, and the coordinator asked to be told if a
passage contradicted it. Nothing about this entry's *data* is marked wrong on the
`spell_level_cap` account; the finding above is about the missing condition, which
is a separate axis and the same error D1's own closing note raises against
`derived/lab.rs::lab_totals`.

**Severity:** wrong-rules-output

---

### F-05 — `virtue.affinity_ability` — both locales' summary says "exempt from the age-based cap"; the book says "exceed … by two points"

**Passage** (ArMDE:3374, verbatim, English):
> If you take this Virtue for an Ability, you may exceed the normal age-based cap during character generation (see page 48) by two points for that Ability.

German, ArMDE:3374 (line-parallel):
> Wenn du diese Tugend für eine Fertigkeit nimmst, darfst du während der Charaktererschaffung die normale altersbedingte Obergrenze (siehe Seite 48) für diese Fertigkeit um zwei Punkte überschreiten.

**Current data:** the mechanical data is **correct** —
`affinity_ability_cost 3/2` is right (pay `ceil(table·2/3)`, A4), and
`validation/scores.rs::validate_ability_age_cap` grants exactly the flat **+2**
the book states. The defect is in `rules/i18n/*/virtues_flaws.json`:

- en `summary`: "Experience put into one Ability counts for half again, and that Ability **is exempt from the age-based cap**."
- de `summary`: "…und diese Fertigkeit **ist von der altersbedingten Obergrenze ausgenommen**."

**Why it is wrong:** "exempt" claims the cap does not apply at all. The book
raises it by two. The text therefore contradicts both the rulebook and the
engine's own behaviour, and a user following the summary will believe an
uncapped score is legal and then be blocked by a validation error they were told
would not fire.

**Correct value:** en — "…and that Ability may exceed the age-based cap by two
points."; de — "…und diese Fertigkeit darf die altersbedingte Obergrenze um zwei
Punkte überschreiten." (`Obergrenze` and `altersbedingt` follow the German
rulebook's own wording at ArMDE:3374.)

**Severity:** text (misleading in both locales, on a rule the engine does enforce)

---

### F-06 — `virtue.alim` — `narrative` on a passage granting an Academic-Ability authorization, and the authorization is missing

**Passage** (ArMDE:3382, verbatim, English):
> You may purchase Academic Abilities during character generation. This Virtue is only available to male characters.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong:** two distinct defects.

1. **The classification.** `academic` is one of
   `rules/core/abilities.json::categories_requiring_virtue`
   (`["academic","arcane","martial"]`), so permission to buy Academic Abilities
   is a rule the engine *computes*:
   `validation/authorization.rs::validate_ability_authorization` errors with
   `ability_category_requires_virtue` on a held Academic Ability unless some
   selected item authorizes it (A14). This is therefore a `creation_effect`, not
   `narrative`.
2. **The missing effect.** With no `ability_authorization`, a legally-built
   'Alim who buys an Academic Ability is told by the app that he may not have it.
   That is a false error on a legal character, not merely an unrecorded rule.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}` — the exact shape
`flaw.covenant_upbringing` and `virtue.student_of_realm` already use (the only
two entries in the catalogue that do). The "only available to male characters"
clause stays uncomputed (see Q-05).

**Severity:** wrong-rules-output

---

### F-07 — `virtue.alluring_to_beings` — `being` is free text where the passage names exactly three classes and three sibling entries already enumerate them

**Passage** (ArMDE:3390, verbatim, English):
> This Virtue is associated with one of three classes of beings: mundane animals, faeries, or magical beings.

**Current data:** `{"key":"being","type":"ref","domain":"text"}`.

**Why it is wrong:** the set is closed and the book names its three members. The
catalogue's three siblings on the same axis all use the `enumerated` domain over
a `being.*` value list: `flaw.offensive_to_beings`
(`being.animals`, `being.demons`, `being.divine`, `being.faeries`,
`being.magical_creatures`, `being.mundane_humans`), `virtue.inoffensive_to_beings`
(the same minus `being.mundane_humans`), and `flaw.unbearable_to_beings`. So the
free-text domain here is not a modelling limit, it is an inconsistency: the
player can type anything, and `max_per_value` / `at_most_one_of` cannot be applied
to a domain with no values.

**Correct value:** `domain: "enumerated"`, `values: ["being.animals",
"being.faeries", "being.magical_creatures"]` — three, matching the passage; the
sibling entries' larger lists come from their own passages and must not be copied
in.

**Severity:** wrong-rules-output (an unbounded picker accepts an illegal class)

---

### F-08 — `virtue.alluring_to_beings` — the stated prohibition on Gifted / Magical Air characters is encoded nowhere

**Passage** (ArMDE:3392, verbatim, English):
> Characters who are Offensive to beings of this sort cannot take this Virtue, including those who have The Gift or Magical Air, though characters who are Inoffensive to them or have the Gentle Gift may.

**Current data:** no `prerequisites`, no `incompatible_with`.

**Why it is wrong:** the passage states a hard bar ("cannot take this Virtue")
with a named exemption. Nothing is encoded, so the app will let a Gifted
character take Alluring to (Beings) without a word.

**Correct value:** the catalogue already ships the exact idiom on the mirror
entry. `flaw.offensive_to_beings` carries

```
"incompatible_with": ["flaw.magical_air"],
"prerequisites": { "kind": "any", "value": [
    { "kind": "none", "value": [ { "kind": "has", "value": "virtue.the_gift" } ] },
    { "kind": "has", "value": "virtue.gentle_gift" } ] }
```

which reads "no Gift, or else the Gentle Gift" and bars Magical Air outright —
precisely what ArMDE:3392 states for this Virtue too. The "Inoffensive to them"
exemption is parameter-scoped (it must match the *chosen* class against
`virtue.inoffensive_to_beings`'s chosen class) and `Prereq` has no
parameter-comparing variant, so that half is not expressible; the Gift / Magical
Air / Gentle Gift half is, and is missing.

**Severity:** wrong-rules-output (an illegal combination validates clean)

---

### F-09 — `virtue.almogaten` — `narrative`, and the Martial-Ability authorization is missing

**Passage** (ArMDE:3398, verbatim, English):
> He commands a dozen men, and he is responsible for their welfare and prosperity. He has Standard Armaments and may take Martial Abilities.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong:** `martial` is a gated category
(`abilities.json::categories_requiring_virtue`), so this is a computed
creation-time permission, exactly as in F-06. Without it, an Almogaten who buys
a Martial Ability draws `ability_category_requires_virtue`.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["martial"]}`. The Wealthy/Poor
choice at ArMDE:3400 and "Standard Armaments" remain uncomputed.

**Severity:** wrong-rules-output

---

### F-10 — `virtue.almogavar` — `narrative`, and the Martial-Ability authorization is missing

**Passage** (ArMDE:3406, verbatim, English):
> He has Standard Armaments, and may take Martial Abilities.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong / correct value:** identical to F-09 —
`"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["martial"]}`.

**Severity:** wrong-rules-output

---

### F-11 — `virtue.almogavar` — "may not take the Poor Flaw or Wealthy Virtue" is absent from `incompatible_with`

**Passage** (ArMDE:3406, verbatim, English):
> He is supported by his unit, and may not take the Poor Flaw or Wealthy Virtue.

**Current data:** no `incompatible_with` key.

**Why it is wrong:** both target ids exist (`flaw.poor`, `virtue.wealthy`) and
the exclusion is absolute, not conditional. The pattern is already used in the
repo: two character-type profiles in `rules/core/character_types.json` list
exactly `["flaw.poor", "virtue.wealthy"]` as `forbidden_traits`. At item level
`incompatible_with` is the mechanism and it is empty.

**Correct value:** `"incompatible_with": ["flaw.poor", "virtue.wealthy"]`, with
the reciprocal entries added to `flaw.poor` and `virtue.wealthy` —
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
one-sided declaration (B7), so the correction is necessarily a three-entry edit.

**Severity:** wrong-rules-output

---

### F-12 — `virtue.amorphous_major` / `virtue.amorphous_minor` — `narrative` on a passage whose whole point is the mechanical difference between the two magnitudes, and the two rows are textually identical

**Passage** (ArMDE:3412, verbatim, English):
> The character develops this ability because his body has adopted the almost-gaseous property of demonic flesh, so any apparent changes in size or fitness are cosmetic, and do not affect the character's statistics. Some amorphous people can select a single shape at midnight, which is the Major form of the Virtue. Its Minor form allows the character to change shape only after having performed a significant act linked to the Obsession of the sponsoring demon. The character may only change to his birth form by selecting it once the conditions of transformation have been met, or involuntarily changing shape when dying or entering holy ground.

**Current data:** both rows `"classification": "narrative"`, no `description` in
either locale, and an **identical** `summary` in both locales for both rows
("The character is able to take on any human form, so long as it does not
identifiably belong to a particular human being." / "Der Charakter kann jede
menschliche Gestalt annehmen…").

**Why it is wrong:** the passage states rules, not colour — a shape-change
trigger condition that *differs by magnitude* (Major: select a shape at midnight;
Minor: only after an act linked to the demon's Obsession), a return-to-birth-form
condition, an involuntary-change trigger, and an explicit "do not affect the
character's statistics" clause. `narrative` asserts the book said nothing
mechanical, which suppresses the description obligation — and the consequence is
visible in the data: the app presents a 3-point Major Virtue and a 1-point Minor
Virtue whose displayed text is character-for-character the same, so a user
cannot learn what the extra two points buy.

**Correct value:** `"classification": "uncomputed_rule"` on both rows, with a
`description` in **both** locales carrying at minimum the magnitude-specific
trigger (and differing between the two rows).

**Severity:** lost-rule

---

### F-13 — `virtue.arcane_lore` — the Parma Magica exclusion and the Enemy-of-the-Order consequence are implemented nowhere and carried in neither locale

**Passage** (ArMDE:3432, 3434, verbatim, English):
> You may take Arcane Abilities during character generation. Unless you have The Gift, you cannot learn Parma Magica. You get an additional 50 experience points, which must be spent on Arcane Abilities.
>
> A Gifted character who is not a Hermetic magus and knows Parma Magica must take the Major Story Flaw Enemy: Entire Order of Hermes, as magi are bound by their Oath to slav the character on sight, unless he immediately joins the Order.

**Current data:** one effect,
`{"type":"restricted_ability_xp","amount":50,"categories":["arcane"]}` — correct
for the 50 XP and, per A7, it also confers the Arcane-category permission, so
the first and third sentences are handled. `summary` in both locales is the
first sentence only; no `description` (classification `creation_effect`).

**Why it is wrong:** `ability.parma_magica` is category `arcane`, so the
restricted pool's permission side-effect legalises Parma Magica for **every**
Arcane Lore holder, Gifted or not — which is exactly what ArMDE:3432 forbids.
`RestrictedAbilityXp` has no `exclude` field (A7 notes
`restricted_ability_xp_pools` hard-codes `exclude: Vec::new()`), so the exclusion
is not expressible as data today. The :3434 consequence is likewise absent from
data and from both locales' text.

**Correct value:** the exclusion needs either an `exclude` list on the variant or
a validator rule; at minimum both clauses belong in `description` in both
locales.

**Severity:** wrong-rules-output (an un-Gifted character may buy Parma Magica and
nothing objects)

---

### F-14 — `virtue.archieunuch` — `narrative`, and the Academic-Ability authorization is missing

**Passage** (ArMDE:3438, verbatim, English):
> Due to your education, you may take Academic Abilities during character creation.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong / correct value:** as F-06 —
`"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`. The
"male characters, who must also be eunuchs" clause stays uncomputed (Q-05).

**Severity:** wrong-rules-output

---

### F-15 — `virtue.aristotelian_training` — the Lab bonus is conditional in the book but unconditional in the data, and two further bonuses are unimplemented

**Passage** (ArMDE:3442, verbatim, English):
> The character gains a +1 bonus on Artes Liberales rolls for grammar, logic, and rhetoric, as well as a +1 bonus when calculating Disputatio Totals (see Art and Academe, page 103, for details). A magus with this Virtue may add +1 to his Lab Totals if attempting to synthesize the New Aristotle with Magic Theory (as described on page 11 of Art and Academe). This Virtue is compatible with Puissant Artes Liberales.

**Current data:** one effect, `{"type":"lab_total_mod","amount":1}`.

**Why it is wrong:** three separate problems in one entry.

1. The `lab_total_mod` is unconditional (A31) while the book gates it on a very
   narrow activity ("if attempting to synthesize the New Aristotle with Magic
   Theory") **and** on the character being a magus. Every Lab Total the
   character shows is inflated by 1.
2. The +1 Artes Liberales roll bonus for grammar / logic / rhetoric is not
   represented at all. `ability_roll_mod` (A41) is the variant for exactly this
   shape and is unused here.
3. The +1 Disputatio Total bonus is not represented and is in neither locale's
   text.

The final sentence is a *compatibility* statement, so the absent
`incompatible_with` is correct — check 7/8's "no requirement it does not state"
direction passes.

**Correct value:** the Lab +1 must be scoped (or dropped in favour of text); the
two roll bonuses must be represented or written into the text.

**Against D1, and more sharply than F-04.** `decisions.md` D1 lists this entry as
unconditional and rules its +1 into `effective/spell.rs::spell_level_cap`. The
passage carries **two** gates, not one: the holder must be "A magus", and the
bonus applies only "if attempting to synthesize the New Aristotle with Magic
Theory". Synthesising Aristotle with Magic Theory is not inventing a spell, so on
the source's own words this +1 has no business in the spell-level cap at all —
this is the row where following D1 would newly *introduce* a wrong number into a
creation-time total that is currently right. Reported, not acted on.

**Severity:** wrong-rules-output (1) and lost-rule (2, 3)

---

### F-16 — `virtue.rard` — the English `name` is the scanno "Rard"; the book's own index and the translation table both read "Bard"

**Passage** (ArMDE:3476, verbatim, English):
> #### Rard

against the same file's own index, ArMDE:3339, verbatim:
> [Bard](#bard)<br>

and the German line-parallel heading, ArMDE:3476:
> #### Barde

and `rules/source/de/translation-tables/tugenden-fehler.md:241`:
> | Bard | Barde | Sozialer Status, Frei; irischer Berufsstand nach Bardenschule |

**Current data:** `rules/i18n/en/virtues_flaws.json` → `"name": "Rard"`. German is
already correct (`"Barde"`). The id is `virtue.rard`.

**Why it is wrong:** the English source heading carries an OCR corruption
(`B` → `R`). The evidence that it is a corruption and not the book's word is
entirely in-repo and needs no outside knowledge: the file's own index at
ArMDE:3339 links `[Bard](#bard)` — a link that does not resolve against a
`#### Rard` heading — the line-parallel German heading is `Barde`, and the
canonical EN↔DE table lists the English term as `Bard`. So the shipped English
name is a scanno the extraction carried through verbatim.

**Correct value:** en `"name": "Bard"`. The **id** is a separate call: `virtue.rard`
is the join key for saves and i18n, so renaming it is a migration, not a text fix.

A second, smaller clause in the same passage is unrecorded: ArMDE:3478 "He should
be no younger than 20." — a stated (soft) age constraint that appears neither in
the data nor in either locale's summary.

**Severity:** text (the name), provenance (the id)

---

### F-17 — `virtue.beadle` — `narrative`, and the Academic-Ability authorization is missing

**Passage** (ArMDE:3482, verbatim, English):
> The character may purchase Academic Abilities at character generation. This Virtue is only available to male characters.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong / correct value:** as F-06 —
`"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`.

**Severity:** wrong-rules-output

---

### F-18 — `virtue.berserk` — "You may learn Martial Abilities at character creation" is unimplemented

**Passage** (ArMDE:3502, verbatim, English):
> You may learn Martial Abilities at character creation.

**Current data:** three effects (`combat_mod` ×2, `soak_mod`), none of them an
`ability_authorization`.

**Why it is wrong:** `martial` is a gated category, so a Berserk grog who buys a
Single Weapon score draws `ability_category_requires_virtue` from
`validation/authorization.rs` against a character the book makes legal. Unlike
the Social Status entries, the classification here is already correct
(`in_play_effect`, carried by the combat effects) — only the effect is missing.

**Correct value:** add `{"type":"ability_authorization","categories":["martial"]}`.

**Severity:** wrong-rules-output

---

### F-19 — `virtue.berserk` — the automatic Personality Trait Angry +2 is unimplemented and untexted

**Passage** (ArMDE:3502, verbatim, English):
> You automatically gain the Personality Trait Angry +2 (or more, at your option).

**Current data:** nothing. The summary in both locales stops at the first
sentence and there is no `description`.

**Why it is wrong:** this is an automatic, numbered grant at character creation
("automatically gain … +2"), and it feeds the entry's own later arithmetic —
the berserk trigger roll is "a stress die + your Angry score" and the
calm-down roll is "stress die + Perception – Angry". None of the 42 `Effect`
variants grants a Personality Trait, so it is not expressible as an effect; but
it is also in neither locale's text, so the rule reaches the user nowhere.

**Correct value:** at minimum the grant written into `description` in both
locales.

**Severity:** lost-rule

---

### F-20 — `virtue.berserk` — the combat and Soak modifiers are conditional in the book and unconditional in the data

**Passage** (ArMDE:3502, verbatim, English):
> While berserk, you get +2 to Attack and Soak scores, but suffer a -2 penalty to Defense.

**Current data:** `combat_mod +2 attack`, `combat_mod -2 defense`,
`soak_mod +2` — the three numbers, signs and targets are each individually
correct against the sentence.

**Why it is wrong:** all three apply unconditionally. `derived/combat.rs::soak`
and `::combat_totals` have no state for "is berserk" (A34: "the Berserk bonus is
applied unconditionally rather than only while berserk; the engine has no
condition to gate it on"; A35: "Berserk's combat bonuses apply always"). So an
un-enraged Berserk character shows +2 Soak and +2 Attack he does not have, and a
-2 Defence penalty he does not suffer. Recorded here because it is an
entry-level mismatch between passage and data, not one of the eleven Part C
systemic gaps — though `engine-semantics.md` A34/A35 already name this entry, so
the corrections pass should treat F-20 as confirmation rather than a new
discovery.

**Correct value:** either a conditional scope on the three effects or their
removal in favour of `description` text.

**Severity:** wrong-rules-output

---

### F-21 — `virtue.blood_of_the_nephilim` — the -5 Aging Roll modifier is directly expressible and missing

**Passage** (ArMDE:3513, verbatim, English):
> You age incredibly slowly, and may live for hundreds of years. You need make an aging roll only once every ten years after the age of 150, and receive a –5 to Aging Rolls.

**Current data:** one effect, `{"type":"size_delta","amount":1}`. No `aging_mod`.

**Why it is wrong:** `aging_mod` with `kind: "aging_roll"` is precisely this
mechanic — per A38, `aging.rs::aging_total` adds `amount` to the AGING TOTAL with
its stored sign, and the catalogue already ships eight `aging_roll` rows. A
flat -5 on the aging roll is the single largest number in this Virtue's passage
and the engine computes that shape today. It is simply absent.

**Correct value:** add `{"type":"aging_mod","kind":"aging_roll","amount":-5}`.
(The stored sign is negative — the book's "–5" is an en dash in the source; the
JSON value is the integer `-5`.) The "once every ten years after 150" interval
remains uncomputed.

**Severity:** wrong-rules-output

---

### F-22 — `virtue.blood_of_the_nephilim` — the Dominion Lore permission is missing, and Dominion Lore is a gated category

**Passage** (ArMDE:3511, verbatim, English):
> You may learn Dominion Lore during character creation without needing to take the Arcane Lore Minor Virtue.

**Current data:** no `ability_authorization`.

**Why it is wrong:** `ability.dominion_lore` has category `arcane`, and `arcane`
is in `abilities.json::categories_requiring_virtue`. So a Nephilim-blooded
companion who buys Dominion Lore — which this sentence explicitly permits
*without* Arcane Lore — is told by
`validation/authorization.rs::validate_ability_authorization` that he needs a
Virtue he was just told he does not need. The sentence is a computed rule and
the app computes the opposite of it.

**Correct value:**
`{"type":"ability_authorization","abilities":["ability.dominion_lore"]}` — the
`abilities` list, not `categories`, because the permission is for that one
Ability and not for Arcane generally.

**Severity:** wrong-rules-output

---

### F-23 — `virtue.blood_of_the_nephilim` — the whole prohibition list at ArMDE:3517 is encoded nowhere

**Passage** (ArMDE:3517, verbatim, English):
> You may not take The Gift or True Faith, Hermetic Virtues or Flaws, Methods or Powers (see Realms of Power: The Divine, Revised Edition, pages 46-56), Virtues such as Giant, Mythic, or Faerie Blood, Flaws such as Age Quickly or Lycanthrope, or Virtues or Flaws that affect your Size.

**Current data:** no `incompatible_with`, no `prerequisites`.

**Why it is wrong:** every named target exists in the catalogue —
`virtue.the_gift`, `virtue.true_faith`, `virtue.giant_blood`,
`virtue.mythic_blood`, `virtue.faerie_blood` (and `virtue.strong_faerie_blood`),
`flaw.age_quickly`, `flaw.lycanthrope` — and `incompatible_with` is the
mechanism for exactly this (B7). Nothing is declared, so all of these combine
silently. The Gift one matters most: the same entry's own `size_delta` says Size
becomes +1, and taking Giant Blood alongside would stack two Size effects the
book forbids.

**Correct value:** `incompatible_with` naming at least the seven ids above, with
the reciprocal entries added to each (symmetry is enforced at load). The two
*category-level* bans — "Hermetic Virtues or Flaws" and "Virtues or Flaws that
affect your Size" — cannot be expressed by an id set; the first has a home in the
character-type profile's `forbidden_categories`, the second has none.

**Severity:** wrong-rules-output

---

### F-24 — `virtue.blood_of_the_nephilim` — "Magi and Grogs may not take this Virtue" is unenforced

**Passage** (ArMDE:3517, verbatim, English):
> Magi and Grogs may not take this Virtue.

**Current data:** `"entity_kinds": ["character"]`.

**Why it is wrong:** `EntityKind` has exactly two values, `Character` and
`Covenant` (`types.rs::EntityKind`), so `entity_kinds` cannot express a
*character-type* restriction — grog / companion / mythic companion / magus are
type profiles, not entity kinds. The mechanism that can is
`rules/core/character_types.json`'s `forbidden_traits`, which is already used for
this shape (the magus and grog profiles list
`["flaw.poor", "virtue.wealthy"]`). `virtue.blood_of_the_nephilim` appears
**nowhere** in `character_types.json` (confirmed: zero matches for "nephilim" in
that file), so a grog or a magus may take it with no objection.

**Correct value:** add `virtue.blood_of_the_nephilim` to the `forbidden_traits`
of the grog and magus profiles. `entity_kinds: ["character"]` is itself correct
and stays.

**Severity:** wrong-rules-output

---

### F-25 — `virtue.blood_of_the_nephilim` — four further stated rules are in neither the data nor either locale's text

**Passage** (ArMDE:3513, 3515, verbatim, English):
> You gain no benefit from Longevity Potions or any magic or supernatural power that slows or relieves Aging or Decrepitude. Once you gain your first Decrepitude Point, it becomes increasingly difficult for you to learn new things: subtract your age ÷ 10 from all Advancement Totals, although the Advancement Total for a season cannot drop below 1.
>
> Due to your great size, you must eat vast amounts of food (equal to what three normal people would eat in a day), and have the Minor Personality Flaw Greedy (which counts as one of your normal Flaws). … You will starve to death in (2 + your Divine Might) days unless you are fed your own body weight in food. After awakening, you suffer a number of lost Long Term Fatigue Levels equal to the number of days you went without food.

**Current data:** nothing for any of these. Both locales' `summary` is the
one-line "You descend from the Nephilim; your Size grows and you age very
slowly." / "Du stammst von den Nephilim ab; deine Größe wächst und du alterst
sehr langsam." There is no `description` (classification `creation_effect`).

**Why it is wrong:** four numbered rules — the Longevity-immunity, the
`age ÷ 10` Advancement penalty with its floor of 1, the granted Minor Personality
Flaw **Greedy** counting against the normal Flaw allowance
(`flaw.greedy_minor` exists in the catalogue), and the starvation /
Long-Term-Fatigue clock — reach the user through no surface at all. Note the
Greedy grant is specifically *not* a `grants_selection` candidate: grants are
budget-exempt (B3), whereas the book says it "counts as one of your normal
Flaws".

**Correct value:** these belong in `description` in both locales. Whether the
Greedy requirement should additionally become a profile-level `required_trait`
is a design call.

**Severity:** lost-rule

---

### F-26 — `virtue.brother_chaplain` — `narrative`, and the Academic-Ability authorization is missing

**Passage** (ArMDE:3531, verbatim, English):
> You may purchase Academic Abilities during character generation.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong / correct value:** as F-06 —
`"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`.

**Severity:** wrong-rules-output

---

### F-27 — `virtue.brother_knight` — `narrative`, and an Academic **and** Martial authorization is missing

**Passage** (ArMDE:3535, verbatim, English):
> You may take Academic and Martial Abilities during character generation.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong:** as F-06, but **two** gated categories, so the entry needs
both. The equipment clause ("Unless you are Poor, you may have high-quality
weapons and armor, and two horses") stays uncomputed.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic","martial"]}`.

**Severity:** wrong-rules-output

---

### F-28 — `virtue.brother_sergeant` — `narrative`, and the Martial-Ability authorization is missing

**Passage** (ArMDE:3539, verbatim, English):
> You may also take Martial Abilities during character generation.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong:** as F-06. Note the asymmetry against F-27 is the book's own
and must be preserved: Brother Knight gets Academic *and* Martial, Brother
Sergeant gets Martial only.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["martial"]}`.

**Severity:** wrong-rules-output

---

### F-29 — `virtue.bureaucrat` — `narrative`, and the Academic-Ability authorization is missing

**Passage** (ArMDE:3543, verbatim, English):
> You may take Academic Abilities during character creation. This Virtue is only available to male characters.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong / correct value:** as F-06 —
`"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`.

**Severity:** wrong-rules-output

---

### F-30 — `virtue.cathedral_school_master` — the stated minimum scores are not encoded as prerequisites

**Passage** (ArMDE:3551, verbatim, English):
> He is at least (30 – Intelligence) vears old and must have scores of 5 in Latin and Artes Liberales, and a Teaching score of at least 3.

**Current data:** no `prerequisites` key at all. The two effects
(`grants_reputation academic 2`; `restricted_ability_xp 240` over
`ability.teaching` + category `academic`) are both correct.

**Why it is wrong:** "must have scores of 5 … and a Teaching score of at least 3"
is an absolute requirement, and `Prereq::AbilityMin { ability, score }` is the
variant for it (B6 — compared against the **effective** score, so a Puissant
boost counts). Two of the three are directly expressible and neither is present,
so a character with Artes Liberales 0 and Teaching 0 can take a Major Social
Status Virtue the book gates behind three trained scores.

**Correct value:**

```
"prerequisites": { "kind": "all", "value": [
  { "kind": "ability_min", "value": { "ability": "ability.artes_liberales", "score": 5 } },
  { "kind": "ability_min", "value": { "ability": "ability.teaching",        "score": 3 } } ] }
```

Latin is the third and is only partly expressible: there is no `ability.latin` —
Latin is an *instance* of the parameterized `ability.dead_language` (see
`abilities.json::scholarly_language`, which names `ability.dead_language` with
exemplar `latin`). `AbilityMin` carries no instance field, so
`ability_min ability.dead_language 5` would be satisfied by any Dead Language at
5, not specifically Latin. Whether to add the looser check or leave Latin out is
a judgement the correction pass must make.

The `(30 – Intelligence)` age floor and the male-only clause stay uncomputed
(Q-05).

**Severity:** wrong-rules-output

---

### F-31 — `virtue.baccalaureus` — the restricted pool funds any Dead Language, not Latin

**Passage** (ArMDE:3472, verbatim, English):
> He is typically between 16 and 19 years old, and has 90 experience points that he may spend on Latin and Artes Liberales — 30 experience points per finished year of studies.

**Current data:**
`{"type":"restricted_ability_xp","amount":90,"abilities":["ability.artes_liberales","ability.dead_language"]}`.

**Why it is wrong:** `ability.dead_language` is parameterized by `language`, and
per A7 `restricted_ability_xp_pools` hard-codes `instances: Vec::new()`, so the
pool's eligibility test matches the Ability id with **no instance test**. The 90
points will therefore fund Greek, Hebrew or Gothic just as happily as Latin,
while the book names Latin only. The amount (90) and the second target
(`ability.artes_liberales`) are correct, as is the accompanying
`grants_reputation academic 1` against "The character has an Academic Reputation
of 1."

**Correct value:** an instance-scoped pool. `PoolEligibility::Ability` has an
`instances` field that would express it, but `RestrictedAbilityXp` has no field
to fill it from, so the fix is a variant change, not a data edit. Recorded as the
weakest finding in the batch: `ability.dead_language` is the closest available
target and the over-permission is narrow.

**Severity:** wrong-rules-output (minor)

### F-32 — `virtue.apprentice` — `narrative` on a passage that states a requirement the engine enforces (closes Q-02 via D3)

**Passage** (ArMDE:3420, verbatim, English):
> This Virtue may be taken by a child character who has the Gift and who has been accepted by an experienced Hermetic magus, with the troupe's approval.

**Current data:** `"classification": "narrative"`, together with
`"prerequisites": {"kind":"has","value":"virtue.the_gift"}`.

**Why it is wrong:** the entry simultaneously claims the book said nothing
mechanical and carries a machine-checked requirement derived from that same
sentence — `validation/prereq.rs::validate_prerequisites` raises `prereq_not_met`
(an **error**) against an Apprentice without `virtue.the_gift`. D3 restates the
class boundary in terms that settle this without needing a convention call:
"`narrative` remains a claim that the book states nothing mechanical, and nothing
about engine capability can make that claim true." ArMDE:3420 states a mechanical
eligibility requirement, so `narrative` is false here regardless of how the field
is normally used. Between the two remaining classes, `uncomputed_rule` asserts the
engine does **not** compute the rule, and here it plainly does — at creation, with
a blocking error.

**Correct value:** `"classification": "creation_effect"`.

**Residual, for the coordinator rather than for this entry.** D3 removes the
`narrative` reading but does not state a general convention for *prereq-only*
entries, and the catalogue holds many. If `creation_effect` is the intended class
for them, it is worth saying so in `decisions.md`, because
`data_integrity.rs::every_vf_is_classified` places no constraint on a
`creation_effect` carrying no `effects` (C7) — so the whole family will load and
pass green under either label, exactly as it does today.

The same passage's other unmodelled clauses — "a child character" (an age floor)
and "with the troupe's approval" — remain uncomputed and are unaffected.

**Severity:** lost-rule / provenance

### F-33 — `virtue.bee_king` — five stated rules reach the user through no surface at all, and the repo's own convention says they should

Found by the verification pass, against my own OK verdict. I had recorded this
entry as clean on the ground that its one effect is correct and that B1 forces
`in_play_effect` once any effect is carried — both of which remain true. What I
missed is that the repo already has a settled convention for exactly this case,
which makes it a finding rather than an instance of Q-03.

**Passages** (ArMDE:3488, 3490, 3494, 3498, verbatim, English):
> Bee Kings do not appear to age after reaching maturity, but every Bee King not killed by circumstances dies of a rapid illness precisely a century after birth.
>
> Any Bee King may command any group of bees to perform any action of which they are physically capable, even if it will lead to their deaths. Usual instructions … continue to be performed until the instruction is countermanded (Penetration 50).
>
> Bee Kings are never stung by any variety of biting insect. An attacking bee swarm is treated as an environmental effect, rather than an opposing mêlee group. The swarm does +10 damage automatically each round. This damage may be Soaked normally.
>
> This is a Supernatural Virtue, and you cannot lose it when being trained as a magus (see page 269). If your master cannot preserve the ability, you cannot be trained.

**Current data:** one effect,
`{"type":"aging_mod","kind":"no_apparent_aging","amount":0}` — correct, and
correctly the `no_apparent_aging` half of the split rather than `no_aging`, since
a Bee King really does age and dies at 100 (A38 uses this very entry as its
example). `summary` in both locales is the single colour sentence "Bee Kingship
is the result of descent from particular faeries…". **No `description` in either
locale.** So one sentence of a fifteen-line passage is modelled and none of the
rest is stated anywhere the user can see.

**Why it is wrong:** not because the classification is wrong — it is not — but
because `description` on an effect-bearing entry is an established practice in
this catalogue, not a novelty. **Nine of the 93 `in_play_effect` entries carry
one**, and they are precisely the entries whose passage exceeds what their effect
can express: `flaw.lame`, `flaw.hobbled`, `flaw.flawed_parma_magica`,
`flaw.limited_magic_resistance`, `virtue.ways_of_the_land` among them. Bee King is
the same shape and does not follow the practice. No new `Effect` variant is
wanted: the catalogue models no immunity as an effect anywhere
(`virtue.lesser_immunity`, `virtue.greater_immunity`, `virtue.immunity_to_cold`,
`virtue.immune_to_disease` all carry none), so prose is the intended home.

**Correct value:** a `description` in **both** locales carrying the century
lifespan, the Penetration 50 command, the biting-insect immunity, the +10/round
soakable swarm, and the apprenticeship-preservation clause.

**Scope note.** The same reasoning reaches four other entries in this batch that
I had already written up on the "belongs in the text" ground without knowing the
precedent existed — F-13 (`virtue.arcane_lore`), F-19 (`virtue.berserk`), F-25
(`virtue.blood_of_the_nephilim`) and F-30's age and sex clauses
(`virtue.cathedral_school_master`). The precedent strengthens all four.

**Severity:** lost-rule

## Open questions

### Q-01 — `virtue.academic_concentration_subject` — what is the closed value set the `subject` parameter should enumerate?

**Passage:** ArMDE:3364 "The character has concentrated in one of the seven
subjects of Artes Liberales" and ArMDE:3366 "It is not allowed for any other
Ability besides Artes Liberales and Philosophiae."

**Reading A:** the seven values are the seven liberal arts, and the parameter
should be `enumerated` over them plus a Philosophiae value (nine-ish options,
since Philosophiae itself subdivides).

**Reading B:** the parameter is the *Ability*, not the subject — two values,
Artes Liberales and Philosophiae — and the sub-subject stays free text.

**What I could not settle it with:** the passage never names the seven subjects.
The only three named anywhere in this span are "grammar, logic, and rhetoric"
(ArMDE:3442, in a different entry). The list must come from the Artes Liberales
Ability description elsewhere in the book, which is outside this batch's span,
and I will not supply it from memory. **F-03's defect stands regardless** — the
parameter is unbounded where the book bounds it; only the correct `values` list
is in doubt.

---

### Q-02 — CLOSED by D3. Superseded by F-32.

Asked whether a prereq-only entry is `narrative` or `creation_effect`. D3's
restatement — "`narrative` remains a claim that the book states nothing
mechanical" — makes the `narrative` reading untenable, and since the engine does
compute the prerequisite at creation, `creation_effect` follows. Recorded as
**F-32**. The original text is kept below for the record.

<details>
<summary>Original Q-02 (superseded)</summary>

### Q-02 — `virtue.apprentice` — does an entry whose only engine-computed mechanic is a `prerequisites` tree count as `creation_effect`, or as `narrative`?

**Passage:** ArMDE:3420 "This Virtue may be taken by a child character who has
the Gift and who has been accepted by an experienced Hermetic magus, with the
troupe's approval."

**Current data:** `"classification": "narrative"` with
`"prerequisites": {"kind":"has","value":"virtue.the_gift"}` — i.e. the entry
*does* carry a rule the engine computes (`validation/prereq.rs::validate_prerequisites`
raises `prereq_not_met`), and simultaneously claims the book said nothing
mechanical.

**Reading A:** `narrative`. The four-way partition is defined against `effects`,
not against every field; `engine-semantics.md` B1 describes the
`uncomputed_rule` / `in_play_effect` boundary as "carrying an `Effect` is what
decides it", and a prereq is not an `Effect`. On this reading every
prereq-only entry in the catalogue is correctly `narrative`.

**Reading B:** `creation_effect`. The stated definition is "the book states a
rule the engine computes at character creation", and a prerequisite is exactly
that — computed, at creation, with a blocking error. On this reading `narrative`
is false here because the Gift requirement *is* a mechanical clause.

**What I could not settle it with:** the definitions in
`docs/vf-audit/README.md` and `engine-semantics.md` B1 are written in terms of
effects and say nothing about prerequisites; `data_integrity.rs::every_vf_is_classified`
constrains only the effect-bearing cases, so both readings load and pass today.
**This is a batch-wide convention, not one entry** — it governs every
prereq-only entry in all 19 batches, so it should be settled once, by the user,
before the corrections pass. The same passage's *other* unmodelled clause
("a child character") is untouched by the answer.

</details>

---

### Q-03 — Does an entry classified `creation_effect` / `in_play_effect` on the strength of one small effect owe a `description` for the substantial uncomputed rules in the same passage?

**Passage (the clearest instance):** `virtue.bee_king`, ArMDE:3484-3499 — a Major
Supernatural Virtue whose passage states a Penetration 50 command power, "The
swarm does +10 damage automatically each round. This damage may be Soaked
normally", immunity to being stung, death "precisely a century after birth", and
"you cannot lose it when being trained as a magus". Its data carries exactly one
effect, `aging_mod no_apparent_aging`, which correctly models one sentence out
of fifteen lines.

**Reading A:** the current rule is right and there is nothing to fix.
`engine-semantics.md` B1 states it explicitly — "an entry the engine computes
*something* for is `in_play_effect` even if its passage also contains an
uncomputable clause" — and only `uncomputed_rule` carries a description
obligation (enforced by `uncomputed_clauses.rs`).

**Reading B:** that rule creates a blind spot that is the exact twin of the
`narrative` defect this audit exists for. One computed sentence launders fourteen
uncomputed ones out of the application: the user sees a summary line and nothing
else. `virtue.arcane_lore` (F-13), `virtue.berserk` (F-19),
`virtue.blood_of_the_nephilim` (F-25) and `virtue.cathedral_school_master`
(F-30's age and sex clauses) are the same shape in this batch alone.

**What I could not settle it with:** this is a policy about the schema, not a
reading of the rulebook — no ArMDE passage bears on it. Raised once, here, rather
than repeated per entry. It materially changes the size of the correction list
for all 19 batches, so it wants an answer early.

**Narrowed by D3, not closed.** D3 rules that where the engine structurally
cannot express a rule, the rule is written out in `description` in both locales —
"the rule reaches the player as text rather than as a number". Every clause listed
under Reading B is exactly that case. What D3 does not resolve is the *mechanical*
obstacle: D3 reaches its outcome by classifying the entry `uncomputed_rule`, and
an entry carrying a live effect cannot take that class (B1, and
`data_integrity.rs::every_vf_is_classified` forbids effects on `uncomputed_rule`).
So an `in_play_effect` entry has no class that triggers the description
obligation, and `uncomputed_clauses.rs` — the guard that keeps the obligation
honest — only ever inspects `uncomputed_rule` entries. D3's *rationale* therefore
points squarely at Reading B while D3's *mechanism* cannot deliver it.

**Narrowed much further by a precedent the verification pass found.** Reading B is
not a proposal — **the repo already does it**. Nine of the 93 `in_play_effect`
entries carry a `description` (`flaw.lame`, `flaw.hobbled`,
`flaw.flawed_parma_magica`, `flaw.limited_magic_resistance`,
`virtue.ways_of_the_land`, …), and they are exactly the entries whose passage
exceeds what their effect expresses. So the schema permits it, the practice
exists, and the classification is not the obstacle after all. That converts
`virtue.bee_king` from an instance of this question into a straightforward
deviation from convention (**F-33**).

**What actually remains open** is therefore much smaller than as first posed: not
*may* an effect-bearing entry carry the residue as text — it may, and nine do —
but **should the obligation be enforced**, i.e. should a guard exist that catches
the other 84, as `uncomputed_clauses.rs` catches `uncomputed_rule` entries today.
Without one, the nine are a convention nobody is held to, and which of the 84 are
legitimately complete is not knowable from the data. That is a test-strategy
decision for the correction phase, and it no longer blocks any classification
verdict in this batch.

---

### Q-04 — `virtue.alluring_to_beings` — German name: the translation table says "Anziehend **für** (Wesen)", the German rulebook heading says "Anziehend **auf** (Wesen)". Which governs?

**Passage:** German rulebook ArMDE:3388 (line-parallel with the English
"#### Alluring to (Beings)"):
> #### Anziehend auf (Wesen)

against `rules/source/de/translation-tables/grundbegriffe.md:343`:
> | Alluring to (Beings) | Anziehend für (Wesen) | Kleine Allgemeine Tugend |

**Current data:** `rules/i18n/de/virtues_flaws.json` →
`"name": "Anziehend auf {being}"`, i.e. it follows the rulebook heading, not the
table. The body text at ArMDE:3392 also uses "auf" throughout ("Anziehend auf
Magische Wesen").

**Reading A:** the table wins. CLAUDE.md is categorical — "the German label for
any term whose English form appears in a table **MUST** match the table's
`Deutsch (DE)` value" — so the current name violates a stated project rule and
should become "Anziehend für {being}".

**Reading B:** the rulebook wins. CLAUDE.md also names the German source files as
"the **source of truth for German translations** — both the full rulebook text
and the term mappings", and the rulebook uses "auf" in the heading *and*
consistently in the prose, so changing only the name would leave the entry's own
summary and description saying "auf" under a heading saying "für".

**What I could not settle it with:** the two canonical sources genuinely
disagree and CLAUDE.md designates both as canonical without a tie-break for this
case. `rules/source/de/translation-tables/README.md` records resolved naming
conflicts, but this pair is not among them. This is a one-word decision that
should be recorded there once resolved, because the same "auf/für" split may
recur on the sibling entries (`flaw.offensive_to_beings`,
`virtue.inoffensive_to_beings`, `flaw.unbearable_to_beings`), which are outside
this batch.

---

### Q-05 — Six entries in this batch state "only available to male characters" and one states a female-only exception; the engine has no sex or gender model. Where should these rules live?

**Passages:** ArMDE:3382 ('Alim), :3438 (Archieunuch, "male characters, who must
also be eunuchs"), :3482 (Beadle), :3531 (Brother Chaplain), :3535 (Brother
Knight), :3539 (Brother Sergeant), :3543 (Bureaucrat), :3553 (Cathedral School
Master) — all "This Virtue is only available to male characters." And ArMDE:3474
(Baccalaureus): "can be taken by a female character, but only if she is (or was)
studying to be a physician at Salerno."

**Current data:** none of the nine carries anything for this clause, in data or
in either locale's text.

**Reading A:** these are setting colour and the troupe's business, not rules the
generator should carry — leave them out.

**Reading B:** they are absolute eligibility restrictions phrased exactly like
the ones the audit treats as rules elsewhere ("may not take the Poor Flaw",
F-11), so they belong at least in the entry's text, and the classification of the
six `narrative` ones would become `uncomputed_rule` on that ground alone —
independently of the `ability_authorization` reclassification in F-06/F-14/F-17/F-26
through F-29.

**What I could not settle it with:** whether `Entity` should gain a sex field is
a product decision, not a rulebook reading, and no ArMDE passage in this span
settles whether a character generator is expected to enforce it. It affects far
more than this batch (the Social Status block runs on well past ArMDE:3562), so
it wants one answer rather than 19.

---

### Q-06 — `virtue.aptitude_for_sin` — should the +3 be an `ability_roll_mod` row rather than prose only?

**Passage** (ArMDE:3428, verbatim, English):
> Each Aptitude for a particular sin adds +3 to all rolls in a very limited circumstance linked to a demon's Obsession.

**Current data:** `uncomputed_rule`, no effects, with a `text`-domain parameter
`sin` that no effect consumes, and the sentence carried verbatim in `description`
in both locales.

**Reading A — it should carry the effect.** `Effect::AbilityRollMod` is a
structural match and the closest thing the enum has to a purpose-built home: a
`text`-domain param naming the subject, a signed `amount`, and — unusually —
**no** field naming which Ability it applies to (A41). Its one existing user,
`virtue.academic_concentration_subject`, encodes the identical shape (+3 in a
named free-text subject). On this reading the unconsumed `sin` param is the tell.

**Reading B — prose is right.** The README's own definition of `uncomputed_rule`
names "scene-/activity-contingent modifiers" as the class, and "a very limited
circumstance linked to a demon's Obsession" is as scene-contingent as the book
gets. Adding the effect would force the class to `in_play_effect` (B1) for a
modifier that computes nothing anyway (`ability_roll_mod` is surfaced-only, C1) —
i.e. it would buy a read-out row at the cost of a truthful classification.

**What I could not settle it with:** the two shipped entries with this exact
shape are classified *differently* from each other today, and nothing in the
source or the guards says which is the model. The number is not lost either way —
it is in `description` in both locales verbatim — so this is a consistency
question, not a lost-rule one, and it recurs wherever a "+N in a narrow
circumstance" Virtue appears.

---

### Q-07 — ArMDE:2816's "one Social Status" rule is modelled nowhere; does that make the empty `incompatible_with` on this batch's Social Status entries correct?

**Passage** (ArMDE:2816, verbatim, English):
> All characters must take one Social Status, and may only take more than one if the descriptions of the Virtues or Flaws explicitly note that they are compatible.

**Why it matters here:** fifteen of this batch's 35 entries are
`social_status`, and the rule's carve-outs are stated *in* them — ArMDE:3474
("This Virtue is compatible with the Hermetic Magus, Mendicant Friar, and Priest
Virtues"), ArMDE:3553 ("compatible with the Baccalaureus and Priest Virtues"),
ArMDE:3547 (Capo and the Partner Virtue). I read those compatibility statements
as requiring *no* `incompatible_with`, which is right only if the general
exclusion is enforced somewhere else.

**It is not.** No character-type profile in `rules/core/character_types.json`
carries a `social_status` category cap, and only three of the ~80 social-status
entries in the whole catalogue declare any `incompatible_with` at all
(`flaw.outsider_major`, `flaw.outsider_minor`, `virtue.failed_apprentice`). So a
character may hold Bureaucrat, Beadle, Brother Knight and Capo simultaneously and
nothing objects.

**Reading A:** the entries are correct and the defect is a single missing
profile-level cap — `max_per_category` style, one Social Status per character,
which is where a rule stated once for all characters belongs.

**Reading B:** the defect is per-entry and ~80 entries need pairwise
`incompatible_with`, which is an O(n²) declaration the symmetry guard would then
police.

**What I could not settle it with:** ArMDE:2816 states the rule but not its
shape, and the compatibility carve-outs are phrased as exceptions to a general
bar rather than as pairwise permissions. Reading A is plainly the cheaper and
more faithful model, but a category cap with per-entry exceptions is not a shape
the current schema has, so this is a design call. **It is catalogue-wide, not
B01's** — recorded here because this batch is where the Social Status block
starts.

---

### Q-08 — the realm association ArMDE:2960-2962 requires of every Supernatural Virtue is modelled on 4 of ~115 entries

**Passage** (ArMDE:2960-2962, verbatim, English):
> All Supernatural Virtues and Flaws are associated with one of the four realms … a character with a Supernatural Virtue or Flaw is immune to Warping caused by living in a high aura associated with the same realm.

**Current data:** only `flaw.bound_to_realm`,
`flaw.necessary_realm_aura_for_ability`, `flaw.realm_stigmatic` and
`virtue.folk_magic` carry a `realm`-domain parameter. This batch's four
Supernatural entries — `virtue.amorphous_major`, `virtue.amorphous_minor`,
`virtue.animal_ken`, `virtue.bee_king`, `virtue.blood_of_the_nephilim` — carry
none, though two of them state their realm in prose (Amorphous is `tainted`, i.e.
Infernal, and Bee King is "a form of faerie blood", ArMDE:3486).

**Why it is not counted as a per-entry failure:** the rule attaches to *every*
Supernatural Virtue and Flaw, so if it is a defect it is one defect affecting
~111 entries, not 111 defects — exactly the Part C shape, and Part C does not
list it. The consequence is concrete: the warping immunity the sentence grants is
computable in principle (`effective/warping.rs` exists) and is computed for
nobody.

**What I could not settle it with:** whether the association belongs as a
per-entry `realm` parameter, as a derived fact (Tainted ⇒ Infernal, and so on),
or as a deliberate scope exclusion is a design question with no answer in the
source. Raised because this batch is the first to hold Supernatural entries and
the question will recur in every later one.

## Verification pass — reconciliation

One independent agent re-derived, from the rulebook alone, the 13 entries I had
marked completely clean. It was told that overturning one would be a success and
that inventing one would not, and it was not allowed to read this file.

**Result: 12 confirmed, 1 overturned.**

| Outcome | Entries |
|---|---|
| Confirmed OK | `affinity_art`, `all_according_to_plan`, `animal_ken`, `apprentice` (non-class checks), `apt_student`, `aptitude_for_sin`, `atlantean_magic`, `book_learner`, `boosted_magic`, `capo`, `cautious_sorcerer`, `cautious_with_ability` |
| **Overturned** | `bee_king` → **F-33** |

**What the overturn was worth.** It did not merely add an entry to the list — it
dissolved most of Q-03. I had reasoned that an `in_play_effect` entry has no class
that triggers a description obligation and had therefore parked its uncomputed
clauses as a policy question. The verification pass went and counted: nine
`in_play_effect` entries already carry a `description`, and they are exactly the
ones whose passage outruns their effect. The practice existed; I had filed a
question against something the repo had already answered. Q-03 is now a narrow
test-strategy question and four other findings in this batch (F-13, F-19, F-25,
F-30) gained a precedent they were missing.

**Negatives it cleared**, recorded so the correction phase does not re-open them:

- `virtue.affinity_art` — "you may exceed the normal recommended limits" waives nothing, because `validation/scores.rs::validate_arts` enforces no Art cap at all (only catalogue resolution, duplicates and advancement-table range). The book agrees: ArMDE:3374 states "by two points" for the Ability version and ArMDE:3378 omits it for the Art version.
- `virtue.affinity_art` — "two *different* Arts" is enforced: `max_total: 2` plus the default `max_per_target: 1` keyed on the whole params map.
- `virtue.animal_ken` — nothing missing. `supernatural` is **not** in `categories_requiring_virtue`, and `ability_score_grant` doubles as the supernatural authorization (A9).
- `virtue.apt_student` — "taught **or trained**" is not a half-implemented row: `AdvancementSource` has no `Trained` variant and its `Taught` doc comment names Apt Student explicitly.
- `virtue.atlantean_magic` — correctly `uncomputed_rule`: `rules/core/spells.json` is a fixed catalogue with no spell-design surface, so no `Effect` can add a Range/Duration/Target option. The `[3444,3469]` range correctly includes the "Possible Abuses of Storms" sidebar.
- All four botch-dice entries — botch dice are modelled nowhere in `crates/arm-rules/src` and are the README's own named example of `uncomputed_rule`; all four carry `description` in both locales.
- Magnitudes and categories for all 13 were verified a second way, against the book's own index groupings (Supernatural Major ArMDE:3023-3050, Social Status Major :3052-3071, Hermetic Minor :3087-3134, Supernatural Minor :3135-3185, General Minor :3239-3328, Social Status Free :3336-3354) — all agree.

**One thing it found that is not a defect in our layer:** the DE `description` for
`cautious_sorcerer` and `cautious_with_ability` reads "scheitest" where German
wants "scheiterst". The typo is in the German *source* at ArMDE:3557 and :3561, so
the i18n is a faithful copy and correct as-is under the "German source is the
source of truth for German" rule. A source fix someday, not an audit finding — and
recorded here so a later pass does not "correct" the i18n away from its source.
