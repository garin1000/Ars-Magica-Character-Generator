# Batch B02 — indices 35-69, ArMDE:3563-3766

Entries: 35. Audited: 35. Failures: **22**. Clean: **13**.
Findings: **31** (F-34 … F-65; there is no F-44, see the note at F-45).
Open questions: **8 new** (Q-09 … Q-16), plus three of B01's recurring here
(Q-06, Q-07, Q-08).

An independent verification pass re-derived my 15 clean verdicts from the rulebook
alone, was told that overturning one would be a success and inventing one would
not, and was not allowed to read this file. **Result: 13 confirmed, 2 overturned**
(`virtue.curse_throwing` and `virtue.dowsing` → **F-65**), plus three open
questions I had not raised (**Q-14, Q-15, Q-16**) and one measurement that
**weakens six of my own findings** — see "Sub-agent reconciliation" at the end.
The counts above are post-reconciliation.

Finding and question numbers **continue B01's sequence** (B01 used F-01…F-33 and
Q-01…Q-08) so `corrections.md` can accumulate them without collision.

Method note: the whole span was read as continuous prose in **both** languages —
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` ArMDE:3555-3773
and the line-parallel German file — before any entry was judged. Reading it as
prose is what turned up **F-34** (four whole Virtues missing from the catalogue),
which no per-entry lookup could have found.

Every entry's **magnitude and categories were verified twice**: against the
descriptor line under its own heading, and independently against the book's own
index groupings (`### Hermetic, Major` ArMDE:3008, `Supernatural, Major` :3023,
`Social Status, Major` :3052, `General, Major` :3073, `Hermetic, Minor` :3087,
`Supernatural, Minor` :3135, `Social Status, Minor` :3187, `General, Minor` :3239,
`Mythic Companion, Free` :3329, `Social Status, Free` :3336, `Supernatural, Free`
:3356). **All 35 agree on both axes.** `kind` is `virtue` for all 35 and all 35
sit inside the Virtues section — correct.

Every `source.lines` range was checked against the heading it claims and the next
heading below it. **Thirty-four bracket their passage correctly** (some include
the trailing blank line before the next `####`, some do not — neither bisects an
entry nor bleeds into a neighbour, so none of those is reported). The one
exception is `virtue.doctor_in_faculty`, whose range swallows an unrelated
sidebar — **F-48**. The seven entries carrying an `anchor` (`clear-thinker`,
`command-animals`, `convoluted-mind`, `death-prophecy`, `dust-devil`,
`enduring-magic`, `enticer-of-multitudes`) all name their own heading correctly.
**No U+2212 occurs anywhere in either locale's `virtues_flaws.json`** (checked
whole-file, both locales: zero occurrences).

Part C systemic gaps are **not** re-reported per entry. In particular:
`MagicResistanceEffect::AuraBonus` being surfaced-only with `amount: 0` (C1) is
not counted as a defect of `virtue.commanding_aura`, and the eight surfaced-only
`SpecialCasting` kinds (C1) are not counted as defects of `virtue.diedne_magic`.
What *is* counted for those entries is what the entry itself could have carried
and does not.

## Decisions applied

`docs/vf-audit/decisions.md` is binding and was applied to every verdict below.

**D1 — applied, not contradicted.** This batch holds one of D1's nine
`lab_total_mod` carriers, `virtue.cyclic_magic_positive`, and D1 already places it
on the conditional side. Per the re-taken ruling its condition is deliberately
ignored in `effective/spell.rs::spell_level_cap`, so **its data is not marked
mis-implemented on that account**. The directed read asked for its exact wording;
it is quoted verbatim in **F-45**, which is filed against the *in-play* totals
(D4's territory), not against the cap.

**D3's precedent is applied throughout.** An engine that structurally cannot
express a rule is grounds for `uncomputed_rule` with the rule written out in both
locales, never for `narrative`. It decides **F-42** (`virtue.custos`, whose
three-way Ability-group choice `AbilityAuthorization` structurally cannot carry)
and underwrites the four `narrative → creation_effect` moves in this batch.

**D2 — this batch contains the most on-point passages found so far, and they are
quoted in full in the "D2 — directed read" section below.** They do not close it,
but ArMDE:3669 words a precondition as a condition on *having* the Virtue rather
than on *taking* it, which is exactly the distinction D2 turns on.

## Verdicts

| id | ArMDE | class | data | text | note |
|---|---|---|---|---|---|
| `virtue.clan_ilfetu` | 3563-3566 | OK | prereqs missing; pool instance scope | three rules in neither locale | F-35 F-36 F-37 |
| `virtue.clear_thinker` | 3567-3570 | OK | OK | OK | |
| `virtue.clerk` | 3571-3574 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-38 |
| `virtue.command_animals` | 3575-3578 | OK | OK | OK | Q-08 |
| `virtue.commanding_aura` | 3579-3596 | OK | effects missing (`soak_mod`); rank parameter missing | description missing both locales | F-39 F-40 |
| `virtue.common_sense` | 3597-3600 | `?` | OK | OK | Q-09 |
| `virtue.convoluted_mind` | 3601-3604 | OK | OK | OK | Q-06 |
| `virtue.corpse_magic` | 3605-3608 | OK | OK | OK | |
| `virtue.covenfolk` | 3609-3612 | narrative → creation_effect | `incompatible_with` missing | OK | F-41 Q-10 |
| `virtue.craft_guild_training` | 3613-3616 | OK | pool instance scope | OK | F-43 |
| `virtue.crafters_healing` | 3617-3620 | OK | OK | OK | |
| `virtue.craftsman` | 3621-3624 | OK | OK | OK | Q-15 |
| `virtue.curse_throwing` | 3625-3628 | OK | OK | de says `Fähigkeit` where the term is `Fertigkeit`; granted score dropped, both locales | F-65 |
| `virtue.custos` | 3629-3634 | narrative → creation_effect | effects missing; `incompatible_with` missing | description missing both locales | F-42 F-41 |
| `virtue.cyclic_magic_positive` | 3635-3638 | OK | effects unconditional (both rows) | de summary truncated mid-sentence | F-45 F-46 |
| `virtue.death_prophecy` | 3639-3644 | OK | OK | OK | |
| `virtue.deft_form` | 3645-3648 | OK | OK | OK | Q-14 |
| `virtue.demonic_blood` | 3649-3662 | OK | effects missing ×3; `incompatible_with` missing | description missing both locales; de terminology | F-49 F-50 F-51 F-52 F-54 |
| `virtue.demonic_might` | 3663-3666 | OK | OK | vis-on-death rule in neither locale; de terminology | F-53 F-54 |
| `virtue.demonic_powers` | 3667-3670 | OK | OK | de terminology | F-54 |
| `virtue.devil_child` | 3671-3674 | OK | OK | free Minor Virtue in neither locale | F-55 |
| `virtue.diedne_magic` | 3675-3682 | OK | OK | compulsory Major Story Flaw in neither locale | F-47 |
| `virtue.doctor_in_faculty` | 3683-3698 | OK | `source` bleeds into a sidebar; prereqs missing; param unbounded | four rules in neither locale | F-48 F-56 F-57 F-58 |
| `virtue.domestic_animal` | 3699-3702 | `?` | OK | OK | Q-11 |
| `virtue.dowsing` | 3703-3706 | OK | OK | de says `Fähigkeit` where the term is `Fertigkeit`; granted score dropped, both locales | F-65 |
| `virtue.dust_devil` | 3707-3710 | OK | OK | OK | Q-16 |
| `virtue.educated` | 3711-3713 | OK | effects missing (`ability_authorization`); pool instance scope | OK | F-59 F-60 |
| `virtue.elemental_magic` | 3731-3738 | OK | OK | requisite rule in neither locale | F-61 |
| `virtue.embitterment` | 3739-3742 | OK | OK | OK | |
| `virtue.emir` | 3743-3746 | narrative → creation_effect | effects missing (`ability_authorization`) | OK | F-62 |
| `virtue.enchanting_ability` | 3747-3750 | OK | param consumed by nothing / target not parameterized | OK | F-63 Q-12 |
| `virtue.enduring_constitution` | 3751-3754 | OK | OK | +3 pain rule in neither locale | F-64 |
| `virtue.enduring_magic` | 3755-3758 | OK | OK | OK | |
| `virtue.the_enigma` | 3759-3761 | OK | OK | OK | see "Negatives cleared" |
| `virtue.enticer_of_multitudes` | 3763-3766 | OK | OK | OK | |

Batch-level (not attributable to any one row): **F-34** — four Virtues in this
span are absent from the catalogue entirely.

## Findings

### F-34 — four Virtues in this span exist in the rulebook and in its index, and are absent from the catalogue

**Passages** (ArMDE:3715, :3719, :3723, :3727, verbatim, English — headings only,
each followed by a `*Minor, General*` descriptor and a full rules paragraph):

> #### Educated (Bardic)
>
> #### Educated (Islamic)
>
> #### Educated (Hebrew)
>
> #### Educated (Vernacular)

Their bodies state real mechanics. ArMDE:3717:

> You receive 50 extra experience points to spend on Art of Memory, Profession: Storyteller, Profession: Poet, any Area Lore, or any Organization Lore.

ArMDE:3721:

> You have been educated in a mosque or Qur'an school, and may purchase Academic Abilities during character generation. You get an extra 50 experience points at character creation, which must be spent on some or all of Arabic, Persian, Greek, Latin, Theology: Islam, Islamic Law, and Artes Liberales.

ArMDE:3725:

> Your character has been educated in a beit ha-midrash or yeshivah and may purchase Academic Abilities at character generation. Your character gains 50 extra experience points to be spent on some or all of the following: Hebrew, Aramaic, Theology: Judaism, and Judaic Lore. Characters from Iberia or the East may also spend some of these points on Arabic.

ArMDE:3729:

> The character may purchase Academic Abilities during character creation. She also gains 50 additional experience points, which must be spent on Academic Abilities, Bargain, the Organization Lore of the character's company, Profession Merchant, or the language of trade in the company's region (usually Latin, Greek, or Arabic).

**Current data:** nothing. The catalogue's entries jump from
`virtue.educated` `[3711, 3713]` straight to `virtue.elemental_magic`
`[3731, 3738]`. A `jq` scan of every entry whose `source.lines[0]` falls in
3700-3770 returns twelve rows and none of them covers 3715-3729. A search of
both i18n files for "Bardic", "Islamic", "Hebrew" and "Vernacular" returns zero
hits. A search of the core catalogue for any id matching
`educated|bardic|islamic|hebrew|vernacular` returns exactly one: `virtue.educated`.

**Why it is wrong:** these are four separate Virtues, not variants of one. The
book's own index lists them as four distinct links under `### General, Minor`
(ArMDE:3257-3260):

> [Educated (Bardic)](#educated-bardic)<br>
> [Educated (Hebrew)](#educated-hebrew)<br>
> [Educated (Islamic)](#educated-islamic)<br>
> [Educated (Vernacular)](#educated-vernacular)<br>

and they are load-bearing elsewhere in the book: ArMDE:4996, inside another
Virtue's own passage, reads

> As a sofer, the character must have the Educated (Hebrew) Virtue.

— a prerequisite that **cannot be encoded at all** while the target id does not
exist, because `ruleset/integrity.rs::validate_prereq_refs` requires every
referenced id to resolve.

Three of the four also grant the Academic-category permission in so many words
("may purchase Academic Abilities"), which is the computed rule F-59 and B01's
nine Social Status findings turn on.

**Correct value:** four new entries — `virtue.educated_bardic`,
`virtue.educated_islamic`, `virtue.educated_hebrew`, `virtue.educated_vernacular`
— each Minor / General, each with a `restricted_ability_xp` of 50 over its own
list, and each of the latter three additionally carrying
`{"type":"ability_authorization","categories":["academic"]}`. The German source
already has all four (ArMDE:3715-3729, line-parallel: *Gebildet (Bardisch)*,
*Gebildet (Islamisch)*, *Gebildet (Hebräisch)*, *Gebildet (Weltlich)*), and
`rules/source/de/translation-tables/tugenden-fehler.md:792` already carries
`Educated (Hebrew) | Gebildet (Hebräisch)`, so the i18n has a source in both
locales.

**Scope note for later batches.** This is the first *missing-entry* defect the
audit has found, and the method that found it was reading the span as prose
rather than looking each catalogued entry up. **Every remaining batch should walk
its span's `####` headings against the ids it was given** — a catalogued entry
that is wrong is visible to a per-entry audit; one that was never extracted is
invisible to it. (The rest of this batch's span was walked heading by heading:
every other `####` between ArMDE:3563 and :3766 has exactly one catalogue entry,
and the only non-entry heading is the `> #### Animal Characters` sidebar at
:3693, which is F-48's subject.)

**Severity:** lost-rule (four whole Virtues unavailable to the user), and
wrong-rules-output downstream (ArMDE:4996's prerequisite is unencodable)

---

### F-35 — `virtue.clan_ilfetu` — "a member of Clan Ilfetu **within House Bjornaer**" is not encoded as a prerequisite, though the catalogue's own idiom is one line away

**Passage** (ArMDE:3565, verbatim, English):
> The character is a member of Clan Ilfetu within House Bjornaer, and has received extra training in the rituals of the House.

**Current data:** no `prerequisites` key.

**Why it is wrong:** House membership is directly expressible — `Prereq::House`
(B6) — and the catalogue uses it one entry later in this very batch:
`virtue.the_enigma` carries `{"kind":"house","value":"house.criamon"}` for the
identical shape ("thus are a member of House Criamon"). Clan Ilfetu is a Hermetic
Minor Virtue whose whole content is House Bjornaer training; as authored, a
Flambeau or Tytalus magus may take it and receive its 50 experience points with
no objection.

**Correct value:** `"prerequisites": {"kind":"house","value":"house.bjornaer"}`.
Note the tri-state (B6): `house` returns `Unknown` when the entity has no house
at all, so this cannot misfire on a character whose House is not yet chosen.

**Severity:** wrong-rules-output

---

### F-36 — `virtue.clan_ilfetu` — the pool funds any Dead Language and any Organization Lore, where the book names Gothic and House Bjornaer Lore

**Passage** (ArMDE:3565, verbatim, English):
> You receive 50 extra experience points that may be spent on House Bjornaer Lore, Magic Lore (with a specialty in the Great Beasts), and Gothic, the dead language that the House uses for all of its rituals.

**Current data:**
```
{"type":"restricted_ability_xp","amount":50,
 "abilities":["ability.dead_language","ability.magic_lore","ability.organization_lore"]}
```

**Why it is wrong:** `ability.dead_language` is parameterized by `language` and
`ability.organization_lore` by `organization` (both confirmed in
`rules/core/abilities.json`). Per A7, `restricted_ability_xp_pools` hard-codes
`instances: Vec::new()`, so the pool's eligibility test matches the Ability id
with **no instance test at all**. The 50 points will therefore fund Hebrew,
Aramaic or Latin as readily as Gothic, and Order-of-Hermes Lore or a guild's Lore
as readily as House Bjornaer Lore. `ability.magic_lore` is not parameterized, so
that third target is exact.

This is the same engine limit B01 recorded as **F-31** (`virtue.baccalaureus`),
and B02 adds two more instances (this one and **F-43**). Three instances in 70
entries makes it a recurring authoring shape rather than a one-off; recorded here
so the corrections pass treats it as one variant change, not three data edits.

**Correct value:** an instance-scoped pool. `PoolEligibility::Ability` already has
an `instances` field that would express it; `RestrictedAbilityXp` has no field to
fill it from, so the fix is a variant change.

**Severity:** wrong-rules-output (minor)

---

### F-37 — `virtue.clan_ilfetu` — three of the passage's four rules reach the user through no surface at all

**Passage** (ArMDE:3565, verbatim, English):
> You receive 50 extra experience points … It is possible that the magus has also been Initiated into the Esoteric Mystery of Divination and Augury (see The Mysteries Revised Edition, page 58); if so, that Virtue must be purchased with the normal allowance of ten points of Virtues and Flaws. Clan Ilfetu teaches the Divination method of haemagmomancy, which is divination by observing the blood splatters of wounded animals; this gives the following bonuses to Divination: +5 family, +3 Corpus.

**Current data:** the 50 XP is modelled. `summary` in both locales is the opening
colour sentence only ("The character is a member of Clan Ilfetu within House
Bjornaer, and has received extra training in the rituals of the House." /
"Der Charakter ist Mitglied des Clans Ilfetu innerhalb des Hauses Bjornaer und hat
eine zusätzliche Ausbildung in den Ritualen des Hauses erhalten."). No
`description` in either locale.

**Why it is wrong:** not the classification — `creation_effect` is right, the 50
XP is a creation number the engine computes. The defect is the same one B01
settled as **F-33**: the repo already carries a `description` on effect-bearing
entries whose passage outruns their effect, and this entry does not follow the
practice. Three numbered rules are lost: the +5 family / +3 Corpus Divination
bonuses (a named method with two named modifiers), the fact that the 50 points do
**not** pay for the Divination and Augury Mystery Virtue, and that the Mystery
Virtue is charged to the normal ten-point allowance. The last of those is a
*budget* statement of exactly the kind B01 flagged at ArMDE:3515, and it cuts the
same way against the engine's blanket budget-exemption for grants (B3).

**Correct value:** a `description` in **both** locales carrying the three clauses.

**Severity:** lost-rule

---

### F-38 — `virtue.clerk` — `narrative` on a passage granting Academic-Ability permission, and the permission is missing

**Passage** (ArMDE:3573, verbatim, English):
> Due to your training, you may take Academic Abilities during character generation.

German, ArMDE:3573 (line-parallel):
> Aufgrund deiner Ausbildung kannst du während der Charaktererschaffung Akademische Fertigkeiten erwerben.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong:** exactly B01's F-06 shape. `academic` is one of
`rules/core/abilities.json::categories_requiring_virtue`
(confirmed: `["academic","arcane","martial"]`), so the permission is a rule the
engine **computes** —
`validation/authorization.rs::validate_ability_authorization` errors with
`ability_category_requires_virtue` on a held Academic Ability unless some selected
item authorizes it (A14). A legally-built Clerk who buys Artes Liberales is told
by the app that he may not have it. So the classification is false and the effect
is missing.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["academic"]}`.

The passage's remaining clauses — the minor/holy-orders and canon-law material,
"The Wealthy Virtue and Poor Flaw affect you normally" (correctly encoded as *no*
incompatibility), and "available to male and female characters, but female
characters may not be in minor or holy orders" — are setting and Q-05 territory
respectively, and are unaffected.

**Severity:** wrong-rules-output

---

### F-39 — `virtue.commanding_aura` — the Soak bonus is directly expressible and absent

**Passage** (ArMDE:3583, :3585-3591, verbatim, English):
> The character also has a Magic Resistance and a Soak bonus that depend upon his rank in the Church. If the character carries a relic, this Magic Resistance is added to that of the relic.
>
> *Pope:* Magic Resistance 25, Soak bonus +5.
>
> *Cardinal, or legatus a latere:* Magic Resistance 20, Soak Bonus +4.
>
> *Legatus missus:* Magic Resistance 15, Soak Bonus +3.
>
> *Archbishop:* Magic Resistance 10, Soak Bonus +2.

**Current data:** one effect,
`{"type":"magic_resistance_mod","kind":"aura_bonus"}`. No `soak_mod`.

**Why it is wrong:** `Effect::SoakMod` (A34) is a computed variant —
`derived/combat.rs::soak` adds it into `stamina + armor + soak_mod + bronze_cord +
form_bonus` — and the catalogue already ships three users. The book states a Soak
bonus of a definite size for each of four ranks and none of it is carried. This is
**not** the Part C `aura_bonus` gap: that gap explains why the *Magic Resistance*
number cannot be carried, and is correctly not counted against this entry. The
Soak half has no such excuse.

**Correct value:** a `soak_mod` whose amount is the chosen rank's — which is
blocked on F-40, because without a rank parameter there is no amount to write.

**Severity:** wrong-rules-output

---

### F-40 — `virtue.commanding_aura` — the book states a four-way choice of rank and the entry has no parameter, so the entire mechanical payload is unrepresentable

**Passage:** ArMDE:3585-3591 as quoted in F-39 — four ranks, each with its own
Magic Resistance and Soak figures.

**Current data:** no `parameters` key; one amount-less
`magic_resistance_mod aura_bonus` effect; no `description` in either locale.
`summary` in both locales is the opening colour sentence only ("This supernatural
power is granted to characters by either the pope, or the Divine directly." /
"Diese übernatürliche Kraft wird Charakteren entweder vom Papst oder direkt vom
Göttlichen verliehen.").

**Why it is wrong:** check 9. The book demands a choice — which of four ranks the
character holds — from a closed, bounded, four-member set, and every number in the
entry depends on it. The catalogue's idiom for this is an `enumerated`-domain
parameter with a `values` list (B8; `virtue.folk_magic` and the
`*_to_beings` family already use it). With no parameter there is nowhere to hang
either figure, so the entry ships a marker effect that carries no magnitude and
prose that mentions no number at all. A Pope and an Archbishop are the same
character in this application.

**And the chosen `kind` is arguably the wrong one, independently of Part C.**
`MagicResistanceEffect::AuraBonus` is the kind the catalogue's other user applies
to a *bonus added to existing resistance* — `virtue.special_circumstances`,
ArMDE:5000: "gaining a +3 bonus to your Casting Scores and Magic Resistance.
(A character only gains a bonus to Magic Resistance if she has Magic Resistance
from another source.)". Commanding Aura's is not a bonus to anything: it is a
**flat Magic Resistance score** the character has outright ("The character also
has a Magic Resistance … *Pope:* Magic Resistance 25"), which is then *added to* a
relic's. So the two shipped users of `aura_bonus` are modelling two different
mechanics under one kind. I record this rather than rule on it — `aura_bonus` is
surfaced-only with `amount: 0` either way, so no number changes today, and which
kind a future computing implementation should use is a design call.

The passage also states three further rules that reach the user nowhere: the
Voice-Range, cost-free, Penetration-free *Aura of Rightful Authority* power
(:3583); "If the character carries a relic, this Magic Resistance is added to that
of the relic" (:3583); and the *legatus missus* suspension/loss conditions
(:3593). The entry is `in_play_effect` and carries one effect, so per B1 the class
is right — but per B01's **F-33** precedent an `in_play_effect` entry whose
passage outruns its effect should carry a `description`, and nine already do.

**Correct value:** an `enumerated` parameter over the four ranks the passage names
— Pope, Cardinal (or *legatus a latere*), *legatus missus*, Archbishop; that is
the whole option set and it is closed, so the `values` list can be written from
this passage alone with nothing to source elsewhere (the id slugs themselves are a
naming decision for the corrections pass, not something this audit should
invent) — plus a
`soak_mod` driven by it, and a `description` in both locales carrying the Magic
Resistance table, the relic-addition rule, the granted power and the loss
conditions. Note that neither `SoakMod` nor `MagicResistanceMod` can currently
read a parameter at all (A34 has no `param` field; A37's is Form-scoped), so the
per-rank numbers need either four sibling entries or a variant change — which is
itself the argument for at minimum carrying the table as text.

**Severity:** lost-rule (the entry's entire numeric content)

---

### F-41 — `virtue.covenfolk` and `virtue.custos` — "may not take the Wealthy Virtue or Poor Flaw" is encoded on neither, though both targets exist

**Passages** (verbatim, English):

ArMDE:3611 (Covenfolk):
> You are supported by the covenant, and so your standard of living is determined by the covenant's resources rather than your own. You may not take the Wealthy Major Virtue or the Poor Major Flaw.

ArMDE:3631 (Custos):
> As a covenant employee, your wealth is determined by the covenant's prosperity, and you may not take the Wealthy Virtue or Poor Flaw.

German, line-parallel, ArMDE:3611 and :3631:
> Du kannst nicht die Große Tugend Wohlhabend oder den Großen Fehler Arm nehmen.
>
> … und du kannst nicht die Tugend Wohlhabend oder den Fehler Arm nehmen.

**Current data:** neither entry carries an `incompatible_with` key, and both are
`"classification": "narrative"`.

**Why it is wrong:** both targets exist and both are Major, matching the book's
own wording — `virtue.wealthy` is `major`/`general` and `flaw.poor` is
`major`/`general` (confirmed in the catalogue). `incompatible_with` is the
mechanism (B7) and is empty on both. This is B01's **F-11** exactly, and the same
precedent applies: two character-type profiles in
`rules/core/character_types.json` already list `["flaw.poor", "virtue.wealthy"]`
as `forbidden_traits`, so the pair is a known idiom in this repo. As authored, a
Covenfolk grog may take Poor and a Custos may take Wealthy with no objection.

`narrative` is also false on both: a prohibition the engine can compute is a
mechanical clause, and D3 is explicit that nothing about engine capability can
make the `narrative` claim true. Since `incompatible_with` is validated at
creation (`validation/prereq.rs::validate_incompatibilities`, an **error**),
the class is `creation_effect`.

**Correct value:** `"incompatible_with": ["flaw.poor", "virtue.wealthy"]` on both,
with the reciprocal ids added to `flaw.poor` and `virtue.wealthy` —
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
one-sided declaration (B7), so this is necessarily a four-entry edit. Plus
`"classification": "creation_effect"` on both.

**Note the asymmetry the book itself draws, and preserve it:**
`virtue.craftsman` (ArMDE:3623) and `virtue.clerk` (:3573) both say the Wealthy
Virtue and Poor Flaw "affect you normally", and `virtue.doctor_in_faculty`
(:3689) says "Both the Wealthy Virtue and the Poor Flaw are allowable". Those
three correctly carry **no** incompatibility, and the corrections pass must not
propagate the pair to them.

**Severity:** wrong-rules-output

---

### F-42 — `virtue.custos` — the three-way restricted-Ability-group choice is the entry's whole mechanical content and is modelled nowhere

**Passage** (ArMDE:3631, verbatim, English):
> You may take one group of restricted Abilities during character generation, either Martial, Academic, or Arcane Abilities. If you choose Martial or Arcane Abilities, you may still learn to speak Latin, although you cannot read or write it.

German, ArMDE:3631 (line-parallel):
> Du kannst während der Charaktererschaffung eine Gruppe eingeschränkter Fertigkeiten erwerben: entweder Kampf-, Akademische oder Arkane Fertigkeiten.

**Current data:** `"classification": "narrative"`, no `effects`, no `parameters`,
no `description` in either locale.

**Why it is wrong:** the three named groups are **exactly**
`abilities.json::categories_requiring_virtue` — `["academic","arcane","martial"]`
— so this sentence is the general statement of the very rule
`validation/authorization.rs` enforces, and a Custos who buys Single Weapon,
Artes Liberales or Magic Lore is refused by the app in all three cases. The
"one group" restriction is also the *interesting* half and is equally absent: as
authored, nothing stops a Custos taking all three.

`AbilityAuthorization` (A14) carries fixed `abilities`/`categories` lists and has
**no `param` field**, so a player-chosen category is structurally inexpressible as
one effect. D3 governs: an engine that structurally cannot express a rule gets
`uncomputed_rule` with the rule written out in both locales — never `narrative`.
Here, though, the *same entry* also carries a prohibition the engine can compute
(F-41), and an entry carrying any computed creation rule is `creation_effect`. So
the class is `creation_effect` and the group choice travels as `description` text,
which B01's F-33 precedent establishes as permitted practice on effect-bearing
entries.

**Correct value:** `"classification": "creation_effect"`, the F-41
incompatibility, and a `description` in **both** locales carrying the one-group
choice, its three options, and the Latin-speech carve-out. An alternative the
corrections pass may prefer — three sibling entries, or an `enumerated`
`category` parameter feeding a param-aware authorization — is a schema decision,
not a data edit.

**Severity:** wrong-rules-output (a legal Custos is refused every Ability his
Virtue exists to permit)

---

### F-43 — `virtue.craft_guild_training` — the pool funds any Organization Lore where the book names the guild's

**Passage** (ArMDE:3615, verbatim, English):
> Like Warrior and Educated, this Virtue gives a bonus of 50 experience points. These must be spent on any Craft or Profession Abilities, Bargain, or Organization Lore: Guild.

**Current data:**
```
{"type":"restricted_ability_xp","amount":50,
 "abilities":["ability.bargain","ability.craft","ability.organization_lore","ability.profession"]}
```

**Why it is wrong:** the amount (50) and three of the four targets are exactly
right — and note that `ability.craft` and `ability.profession` being matched
any-instance is **correct** here, because the book says "any Craft or Profession
Abilities". Only `ability.organization_lore` is over-broad: the book scopes it to
the guild's Lore and A7's instance-blind matching funds every Organization Lore
the character holds. Same engine limit as F-36 and B01's F-31.

Also correct and worth recording so it is not re-opened: all four targets are
category `general`, so this Virtue needs **no** `ability_authorization` — unlike
`virtue.educated`, which the same sentence names as its sibling (F-59).

**Correct value:** an instance-scoped pool (a variant change, not a data edit).

**Severity:** wrong-rules-output (minor)

---

### F-45 — `virtue.cyclic_magic_positive` — the directed read: both modifiers are conditional in the book and unconditional in the data, and the Lab one carries a second gate

*(Numbering note: there is no F-44 — the number was reserved for a finding that
merged into F-42 and is deliberately left unused rather than renumbering the
table.)*

**Passage** (ArMDE:3637, verbatim, English — the whole entry body, since D4 turns
on how the condition is worded):
> Your magic is attuned to some cycle of nature (solar, lunar, or seasonal, for example) and as such, is more potent at specific times. At those times, you receive a +3 bonus to all Casting Scores. The bonus also applies to Lab Totals if the positive part of the cycle covers the whole season. The cycle of your magic must be regular and approved by the storyguide. Furthermore, the length of time when the bonus applies must be equal to the amount of time when it does not.

German, ArMDE:3637 (line-parallel):
> Deine Magie ist auf einen Naturzyklus abgestimmt (z. B. solar, lunar oder saisonal) und ist daher zu bestimmten Zeiten besonders potent. Zu diesen Zeiten erhältst du einen +3-Bonus auf alle Zauberwerte. Der Bonus gilt auch für Laborsummen, wenn der positive Teil des Zyklus die gesamte Saison abdeckt. Der Zyklus deiner Magie muss regelmäßig und vom Spielleiter genehmigt sein. Außerdem muss der Zeitraum, in dem der Bonus gilt, gleich lang sein wie der Zeitraum, in dem er nicht gilt.

**Current data:**
```
{"type":"casting_total_mod","amount":3,"scope":"all"}
{"type":"lab_total_mod","amount":3}
```

**What the passage actually says, clause by clause** — this is the part D4 needs:

| Clause | Scope it imposes |
|---|---|
| "At those times, you receive a +3 bonus to all Casting Scores" | the bonus exists only during the cycle's positive phase |
| "**Furthermore, the length of time when the bonus applies must be equal to the amount of time when it does not**" | the positive phase is exactly **half** the time — the book fixes the duty cycle, it is not left open |
| "The bonus also applies to Lab Totals **if the positive part of the cycle covers the whole season**" | the Lab bonus is gated **a second time**, on the positive phase spanning an entire season — which for a lunar or solar cycle it never does |

**Why it matters:** the `casting_total_mod` row's number, sign and `scope: "all"`
are each correct against "a +3 bonus to all Casting Scores" (A30's `all` matches
Formulaic, Ritual and Spontaneous, which is what "all Casting Scores" means), and
A30 already records that the engine applies such modifiers unconditionally. But
this entry is the sharpest case in the catalogue for the in-play question, because
the book does not merely gate the bonus on a circumstance — **it states the duty
cycle numerically**. A character with Cyclic Magic (positive) is displayed a
Casting Score that is +3 too high exactly half the time. The Lab row is worse: its
condition ("the positive part of the cycle covers the whole season") is one the
book's own examples mostly fail, and it is applied to every cell of
`derived/lab.rs::lab_totals` unconditionally.

**Per D1, none of this is marked against `effective/spell.rs::spell_level_cap`.**
The re-taken ruling applies all nine `lab_total_mod` carriers flat to the cap with
conditions deliberately ignored, and this entry is explicitly one of them. This
finding is filed against the **in-play** totals, which D1's closing note assigns
to D4.

**Correct value:** a conditional scope on both rows, or their removal in favour of
`description` text. Neither `CastingTotalMod` nor `LabTotalMod` carries a
condition field, so this is not a data edit.

**Severity:** wrong-rules-output (in play); input to D4

---

### F-46 — `virtue.cyclic_magic_positive` — the German summary is truncated mid-sentence

**Current data**, `rules/i18n/de/virtues_flaws.json`:
> "summary": "Deine Magie ist auf einen Naturzyklus abgestimmt (z."

against the English summary, which is the book's full first sentence:
> "Your magic is attuned to some cycle of nature (solar, lunar, or seasonal, for example) and as such, is more potent at specific times."

and the German source it should have come from, ArMDE:3637:
> Deine Magie ist auf einen Naturzyklus abgestimmt (z. B. solar, lunar oder saisonal) und ist daher zu bestimmten Zeiten besonders potent.

**Why it is wrong:** the extraction's sentence splitter broke on the period inside
the German abbreviation "z. B." and kept only the fragment before it. The shipped
German summary ends in the middle of a parenthesis, on a bare "z." — it names
none of the three cycle types, never reaches "is more potent at specific times",
and is not a grammatical German sentence. The entry carries no `description`, so
this fragment is the **only** German text a user sees for a Minor Hermetic Virtue.

**Correct value:** the German source sentence in full, as quoted above.

**Scope note for later batches.** This is a mechanical extraction failure, not a
translation judgement, and its cause — a sentence splitter cutting at `z. B.`,
`d. h.`, `Nr.`, `ca.`, `bzw.`, `usw.` — is not specific to this entry. **Every
later batch should check its German summaries for a trailing single-letter or
abbreviation fragment.** I checked this batch's other 34 German summaries by
eye against their sources; this is the only one affected here.

**Severity:** text (the entry's only German text is a fragment)

---

### F-47 — `virtue.diedne_magic` — the compulsory Major Story Flaw, and its unusual budget treatment, reach the user nowhere

**Passage** (ArMDE:3681, verbatim, English):
> You must keep your lineage hidden from the Order, giving you a Major Story Flaw — Dark Secret is an obvious choice, but you may choose a different one with troupe approval. This is in addition to your normal allowance of Flaws, and does not grant you any points with which to buy Virtues.

German, ArMDE:3681 (line-parallel):
> Du musst deine Abstammung vor dem Orden verborgen halten, was dir einen Großen Geschichte-Fehler einbringt – Dunkles Geheimnis ist eine naheliegende Wahl, aber du kannst mit Zustimmung der Spieltruppe einen anderen wählen. Dies ist zusätzlich zu deiner normalen Zahl an Fehlern und gewährt dir keine Punkte, mit denen du Tugenden kaufen kannst.

**Current data:** one effect,
`{"type":"special_casting_mod","kind":"diedne"}` — correct as far as it goes and
correctly surfaced-only (C1; not counted against the entry). `summary` in both
locales is the opening colour sentence. No `description` in either locale.

**Why it is wrong:** two rules are lost. The compulsory Major Story Flaw is an
absolute requirement stated with a named default ("Dark Secret"), and the
`story`-category cap in every type profile is 1 (`character_types.json`), so
taking it consumes the character's only Story Flaw slot — a real planning
consequence the user is never told about. And the budget clause is the **exact
inverse** of the one B01 recorded at ArMDE:3515: there a mandated Flaw "counts as
one of your normal Flaws"; here a mandated Flaw is "in addition to your normal
allowance … and does not grant you any points". Two Virtues in the same book
specify opposite budget treatments for a mandated Flaw, and the engine models
neither — `validation/balance.rs::compute_balance` iterates bought selections and
knows only one rule (B3).

The ÷5-or-÷2 spontaneous choice and the fatiguing-spontaneous Art-doubling
(ArMDE:3677, :3679) are the `diedne` kind's own surfaced-only payload and are
Part C, not this entry's defect — but they are also in neither locale's text,
which is the same F-33 shape.

**Correct value:** a `description` in **both** locales carrying the compulsory
Major Story Flaw, its "in addition, grants no points" treatment, and the two
spontaneous-casting rules.

**Severity:** lost-rule; and a second data point for whether the engine should
model per-item budget treatment of mandated Flaws at all

---

### F-48 — `virtue.doctor_in_faculty` — the `source` range swallows an unrelated sidebar

**Current data:** `"lines": [3683, 3698]`.

**What is actually there:** the entry's own text runs ArMDE:3683 (the heading) to
:3691 (its last paragraph). ArMDE:3692 is blank, and :3693-3697 is a blockquote
sidebar belonging to no Virtue at all:

> \> #### Animal Characters
> \>
> \> An animal can have personality and add greatly to stories, just as a human can. …

:3698 is blank and :3699 is the next entry's heading (`#### Domestic Animal`).

**Why it is wrong:** check 1. The range bleeds five lines of unrelated editorial
content into the entry's provenance. It is not a harmless over-reach either: the
sidebar's subject is animal characters, which is the subject of the **next** entry
(`virtue.domestic_animal`, "*Free, Social Status, animals only*"), so a reader
resolving this citation lands on text that looks like it belongs to a different
Virtue. The provenance guards cannot catch it — `rules_source_provenance.rs` can
only prove the range lands on non-blank lines (B11).

Contrast B01's accepted case: `virtue.atlantean_magic`'s `[3444, 3469]` also
includes a sidebar, but that sidebar ("Possible Abuses of Storms") is *about* the
Virtue. This one is not.

**Correct value:** `"lines": [3683, 3691]` (or `[3683, 3692]` to keep the trailing
blank, matching the batch's prevailing style). An `anchor` of `doctor-in-faculty`
would additionally make the citation survive a re-sync; this entry carries none.

**Severity:** provenance

---

### F-49 — `virtue.demonic_blood` — the aging immunity the passage states verbatim is the engine's own `no_aging` and `no_apparent_aging`, and neither is carried

**Passage** (ArMDE:3659, verbatim, English):
> However, she does not show the effects of aging; any Aging Points acquired do not get applied to her Characteristics, although they do still count as experience points towards Decrepitude.

German, ArMDE:3659 (line-parallel):
> Sie zeigt jedoch keine Anzeichen von Alterung; erworbene Alterungspunkte werden nicht auf ihre Eigenschaften angewendet, zählen aber weiterhin als Erfahrungspunkte für die Gebrechlichkeit.

**Current data:** two effects, `might_grant infernal 5` and `power_levels 30`. No
`aging_mod` of any kind.

**Why it is wrong:** this sentence is, clause for clause, the engine's two aging
immunity markers as `engine-semantics.md` A38 defines them:

| Book clause | Engine kind | A38's own words |
|---|---|---|
| "she does not show the effects of aging" | `no_apparent_aging` | "`AgingOutcome::apparent_age_increases` is false at every total" |
| "any Aging Points acquired do not get applied to her Characteristics, although they do still count as experience points towards Decrepitude" | `no_aging` | "Returns 0 drops: Aging Points still accrue and still build Decrepitude, but never lower a Characteristic" |

Both are computed, not surfaced-only, and A38 states they are "orthogonal" —
deliberately two tags so an entry can carry either or both. `virtue.unaging`
already carries **exactly this pair**
(`[{aging_mod no_aging 0}, {aging_mod no_apparent_aging 0}]`), so the idiom is one
lookup away. As authored, a Demonic Blooded character's Characteristics drop with
age, which the book flatly denies.

The book's own text confirms the reading from the other side: ArMDE:3661 forbids
taking the Unaging Virtue alongside Demonic Blood (F-51) — which only makes sense
because Demonic Blood already grants Unaging's effect.

**Correct value:** add
`{"type":"aging_mod","kind":"no_aging","amount":0}` and
`{"type":"aging_mod","kind":"no_apparent_aging","amount":0}`
(amount 0 for both — A38 records that `amount` is ignored for these kinds and
`data_integrity.rs` pins it).

The same paragraph's other two rules — effective age advancing two years per year,
and two aging rolls per year from effective age 35 — have no `AgingEffect` kind
and stay uncomputed; they belong in the F-52 description.

**Severity:** wrong-rules-output (a computed total is wrong, and the variant to
fix it already exists and is already used by a sibling entry)

---

### F-50 — `virtue.demonic_blood` — "may learn Infernal Lore … without needing to take the Arcane Lore Minor Virtue" is the exact rule the app computes the opposite of

**Passage** (ArMDE:3655, verbatim, English):
> The character may learn Infernal Lore during character creation without needing to take the Arcane Lore Minor Virtue.

German, ArMDE:3655 (line-parallel):
> Der Charakter kann während der Charaktererschaffung Infernalkunde erlernen, ohne die Kleine Tugend Arkanes Wissen nehmen zu müssen.

**Current data:** no `ability_authorization`.

**Why it is wrong:** `ability.infernal_lore` is category `arcane` (confirmed in
`rules/core/abilities.json`), and `arcane` is in
`abilities.json::categories_requiring_virtue`. So a Demonic Blooded character who
buys Infernal Lore — which this sentence explicitly permits *without* Arcane Lore
— is told by `validation/authorization.rs::validate_ability_authorization` that he
needs the very Virtue he was just told he does not need. This is B01's **F-22**
(`virtue.blood_of_the_nephilim` / Dominion Lore) repeated word for word with a
different Lore; the two passages are near-identical in construction, which makes
it a deliberate authorial pattern in the book rather than a coincidence.

**Correct value:**
`{"type":"ability_authorization","abilities":["ability.infernal_lore"]}` — the
`abilities` list, not `categories`, because the permission is for that one Ability
and not for Arcane generally.

**Severity:** wrong-rules-output

---

### F-51 — `virtue.demonic_blood` — "you cannot take the Unaging Virtue or the Age Quickly Flaw" is encoded nowhere, and both ids exist

**Passage** (ArMDE:3661, verbatim, English):
> You cannot take the Infernal Blessings Virtue described in *Realms of Power: The Infernal*; the Demonic Powers Virtue replaces that Virtue for the demon-blooded. You may not take any Virtue that affiliates her with a realm other than the Infernal. Also, you cannot take the Unaging Virtue or the Age Quickly Flaw. She may not have children, and so cannot have Dependents of this sort.

**Current data:** no `incompatible_with` key.

**Why it is wrong:** two of the four prohibitions name ids the catalogue holds —
`virtue.unaging` (Minor / Supernatural) and `flaw.age_quickly` (Major /
Supernatural), both confirmed present — and `incompatible_with` is the mechanism
for exactly this (B7). Nothing is declared, so both combine silently. The Unaging
pair is the one that bites: `virtue.unaging` carries the very two `aging_mod`
markers F-49 says Demonic Blood is missing, so today a player can *only* get the
book's stated behaviour by taking a combination the book forbids.

The other two prohibitions are not expressible as an id set and must not be
faked: "the Infernal Blessings Virtue" is from *Realms of Power: The Infernal*,
which has no English source in `rules/source/en/`, so per CLAUDE.md's provenance
rule it cannot be implemented at all; and "any Virtue that affiliates her with a
realm other than the Infernal" is a category-level ban with no home in the schema
— it is the same shape as B01's F-23 residue and is adjacent to Q-08.

**Correct value:** `"incompatible_with": ["flaw.age_quickly", "virtue.unaging"]`,
with the reciprocal ids added to both targets (symmetry is enforced at load).

**Severity:** wrong-rules-output

---

### F-52 — `virtue.demonic_blood` — a fourteen-line passage stating at least nine further rules ships with a one-line summary and no description in either locale

**Passages** (ArMDE:3651, :3653, :3659, verbatim, English — the clauses not
covered by F-49/F-50/F-51):
> Her body also contains one pawn of Corpus vis, which can only be extracted once she is dead. She is immune to Warping of any kind, need not eat or drink, and cannot produce a child; however, she suffers the natural urges associated with these activities, and may be required to make Personality rolls to overcome her desires.
>
> The Might cost for each Power is equal to its magnitude divided by two, rounded down (but always at least one point). These may be constant effects designed in the usual fashion (Sun duration, two uses per day, and an Environmental Trigger, see page 257), her Might Pool is always reduced by the cost of such a Power, and she cannot turn these Powers off. The Initiative total of each Power is the character's Quickness.
>
> A Demonic Blooded character's life span is short — roughly half that of a pure-blooded human. Her effective age (which applies as if it were her real age when creating a Longevity Ritual and when making rolls on the Aging Table) increases two years for every year that passes, and you must make two aging rolls each year once her effective age reaches 35.

**Current data:** `summary` in both locales is one line ("A demon is your parent;
you have an Infernal Might and Infernal Powers." / "Ein Elternteil ist ein Dämon;
du besitzt einen infernalischen Machtwert und infernalische Kräfte."). No
`description` in either locale.

**Why it is wrong:** B01's **F-33** precedent. Nine numbered or absolute rules —
the one pawn of Corpus vis, blanket Warping immunity, no food or drink, sterility
with Personality rolls for the urges, the magnitude÷2 (min 1) Might cost per
Power, the constant-effect Might Pool reduction and inability to switch a Power
off, Initiative = Quickness, the doubled effective-age rate, and the two aging
rolls per year from effective age 35 — reach the user through no surface at all.
Of these, the Warping immunity is the most costly: `effective/warping.rs` does
compute a Warping Score and owed Virtues/Flaws, so the app will present a Demonic
Blooded character with warping consequences the book says she is immune to, and
nothing in the entry says otherwise. (No `Effect` variant expresses a Warping
immunity; the two aging immunities in F-49 do not cover it.)

Also worth recording for check 10's "nothing implemented the passage never says"
direction, and **cleared**: `might_grant infernal 5` and `power_levels 30` are
both right — ArMDE:3651 "an Infernal Might (Corpus) score of 5" and :3653 "up to
30 levels of Infernal Powers". The `(Corpus)` alignment of the Might is not
expressible (`MightGrant` carries `realm` only, A26) and is one more clause for
the description.

**Correct value:** a `description` in **both** locales carrying the nine clauses.

**Severity:** lost-rule

---

### F-53 — `virtue.demonic_might` — the vis-on-death formula is in neither locale

**Passage** (ArMDE:3665, verbatim, English):
> Her Infernal Might increases by 2 points. Upon her death, her body contains a number of pawns of Corpus vis equal to her (Infernal Might / 5), rounding up. You may take this Virtue more than once, though it can account for no more than half of the character's total Virtues.

**Current data:** `might_grant infernal 2` — correct; `max_per_target: 255` —
correct for "may take this Virtue more than once" (B10: `max_per_target` is the
axis that keys on the whole params map, and with no params every copy is one
target, so 255 is the right knob and `max_total` correctly stays at its default);
`max_share_of_kind: {1, 2}` — correct for "no more than half of the character's
total Virtues" (B10 compares `points · denominator > total · numerator`, i.e.
warns above half). `prerequisites: has virtue.demonic_blood` — correct against
ArMDE:3665's first sentence. **The data is clean.**

`summary` in both locales is "Increases your Infernal Might. Requires Demonic
Blood." / "Erhöht deinen infernalischen Machtwert. Erfordert Dämonisches Blut."
There is no `description`.

**Why it is wrong:** the `(Infernal Might / 5)` rounded-up vis yield is a stated
formula and appears nowhere. No vis model exists in the engine, so it cannot be
computed — which is precisely the case D3 and F-33 say is carried as text.

**Correct value:** a `description` in both locales carrying the formula.

**Severity:** lost-rule (minor)

---

### F-54 — both locales' German text for the Demonic family uses "infernalisch", which is neither the translation table's word nor the German rulebook's

**Current data**, `rules/i18n/de/virtues_flaws.json`:

- `virtue.demonic_blood` summary: "…du besitzt einen **infernalischen** Machtwert und **infernalische** Kräfte."
- `virtue.demonic_might` summary: "Erhöht deinen **infernalischen** Machtwert."
- `virtue.demonic_powers` summary: "Verleiht zusätzliche Stufen **infernalischer** Kräfte."

against the canonical table,
`rules/source/de/translation-tables/sphären-mächte.md:37`:
> | Infernal Might | Infernale Macht | (K) |

and the German rulebook itself, ArMDE:3651 and :3665 (line-parallel):
> …besitzt einen **Infernalen Machtwert** (Corpus) von 5.
>
> Ihr **Infernaler Machtwert** steigt um 2 Punkte.

and ArMDE:3653, :3669 for the Powers:
> Sie hat bis zu 30 Stufen **Höllischer Kräfte** …
>
> Er erhält 20 zusätzliche Stufen **Höllischer Kräfte** …

**Why it is wrong:** CLAUDE.md is categorical — "the German label for any term
whose English form appears in a table MUST match the table's `Deutsch (DE)`
value" — and `sphären-mächte.md` uses the adjective **infernal-** throughout
("Infernale Macht", "Infernaler Segen", "Infernale Reputation", "Infernales
Vis"), never *infernalisch-*. The German rulebook agrees. The i18n's
*infernalisch-* is a third form belonging to neither source. For the Powers the
rulebook additionally prefers *Höllische Kräfte*.

Note this is not the same class of problem as B01's closing note about
"scheitest": that was a faithful copy of a typo in the German *source*. Here the
i18n deviates *from* the source.

**Correct value:** "Infernaler Machtwert" (following the rulebook, which is the
source of truth for German and which is also the `Might Score` → `Machtwert`
mapping at `sphären-mächte.md:31`), and "Höllische Kräfte" for the Powers.

**Severity:** text (three entries, one locale, a term CLAUDE.md declares canonical)

---

### F-55 — `virtue.devil_child` — the only rule in the passage is the free Minor Virtue, and neither locale's text mentions it while both mention an incompatibility the passage does not state

**Passage** (ArMDE:3673, verbatim, English — the entry's whole body):
> This Virtue has no cost, and can only be taken for a Mythic Companion who has been born with demonic parents. The character gets the Demonic Might or Demonic Powers (player's choice) Minor Virtue free.

German, ArMDE:3673 (line-parallel):
> Diese Tugend hat keine Kosten und kann nur für einen Mythischen Gefährten genommen werden, der von dämonischen Eltern geboren wurde. Der Charakter erhält nach Wahl des Spielers die Kleine Tugend Dämonische Macht oder Dämonische Kräfte kostenlos.

**Current data:** no `effects`; `incompatible_with` naming
`virtue.faerie_doctor`, `virtue.nephilim`, `virtue.spirit_votary`,
`virtue.the_gift`. `summary` en: "Free status Virtue for a Mythic Companion born
of demonic parents; incompatible with The Gift."; de: "Freie Status-Tugend für
einen mythischen Gefährten dämonischer Abstammung; unvereinbar mit der Gabe."

**Two things checked and cleared first, so the corrections pass does not undo
them:**

1. **The `incompatible_with` set is correctly sourced** — not from this passage,
   but from the general Mythic Companion rule, ArMDE:2637, verbatim:
   > All Mythic Companions take a Free Virtue which specifies their status. These Virtues are incompatible with each other, and with The Gift, and are not available to grogs.

   That covers all four ids exactly, and the three sibling entries declare the
   mirror sets, so symmetry holds. Check 8 passes.
2. **The missing `effects` is correct too.** The free Minor Virtue is a
   *player choice* between two items, which `GrantsSelection` (A18, a flat set)
   structurally cannot express — and the repo models it in the right place
   instead: `rules/core/mythic_companion_types.json` gives
   `mythic_type.devil_child` a `{"kind":"choice","choice_key":
   "devil_child_free_minor","options":[…demonic_might…demonic_powers]}`. So the
   rule *is* computed, which is why `creation_effect` on an effect-less entry is
   right here and C7's blanket suspicion does not apply.

   Worth flagging for a later batch: the two siblings that grant a *single* item
   do it **twice** — `virtue.faerie_doctor` carries
   `grants_selection ["virtue.dowsing"]` while `mythic_type.faerie_doctor`
   independently grants the same id, and `virtue.spirit_votary` /
   `mythic_type.spirit_votary` duplicate `virtue.second_sight` the same way.
   Devil Child's single-site modelling is the correct pattern of the three.

**Why it is nonetheless a finding:** the entry's only stated mechanic — "gets the
Demonic Might or Demonic Powers (player's choice) Minor Virtue free" — is in
neither locale's text, while both locales' text foregrounds an incompatibility
that comes from a different page. A user reading the app learns that Devil Child
clashes with The Gift and does not learn that it hands them a free Minor Virtue
worth one point, nor that there is a choice to make.

**Correct value:** the free-Minor-Virtue choice added to the `summary` (or a
`description`) in **both** locales. "Mythischer Gefährte" should also be
capitalised in the German summary — it is the type name, spelled
`Mythischer Gefährte` in the German descriptor line at ArMDE:3672.

**Severity:** lost-rule (minor)

---

### F-56 — `virtue.doctor_in_faculty` — three stated minimum scores are not encoded as prerequisites

**Passage** (ArMDE:3687, verbatim, English — the OCR's missing comma after "Latin"
is the source's, not mine):
> A character starting the game with this Virtue must be at least (27 Intelligence) years old. He must have a score of 5 in Latin. Artes Liberales, and the Ability that correlates to his faculty degree.

German, ArMDE:3687 (line-parallel, and it resolves the OCR):
> Ein Charakter, der das Spiel mit dieser Tugend beginnt, muss mindestens (27 – Intelligenz) Jahre alt sein. Er muss einen Wert von 5 in Latein, Artes Liberales und der Fertigkeit haben, die seiner Fakultät entspricht.

**Current data:** no `prerequisites` key. The two effects are both correct —
`grants_reputation academic 3` against ":3687 He also begins the game with an
Academic Reputation of 3", and `restricted_ability_xp 300 categories:["academic"]`
against ":3687 receives an additional 300 experience points, which must be spent
on Latin and Academic Abilities" (`ability.dead_language` is itself category
`academic`, so Latin is inside the pool and inside the permission it confers).

**Why it is wrong:** "must have a score of 5" is absolute, and
`Prereq::AbilityMin { ability, score }` is the variant for it (B6 — compared
against the **effective** score, so a Puissant boost counts). This is B01's
**F-30** (`virtue.cathedral_school_master`) repeated on a second scholastic
Virtue, and the two passages are the same construction, which makes it a
book-wide shape rather than a one-off. As authored, a character with Artes
Liberales 0 may take a Major Social Status Virtue the book gates behind three
trained 5s.

**Correct value:** at minimum
```
"prerequisites": { "kind": "all", "value": [
  { "kind": "ability_min", "value": { "ability": "ability.artes_liberales", "score": 5 } } ] }
```
The other two are only partly expressible, for the reasons B01 recorded: there is
no `ability.latin` — Latin is an *instance* of the parameterized
`ability.dead_language` (parameter `language`), and `AbilityMin` carries no
instance field, so `ability_min ability.dead_language 5` would be satisfied by any
Dead Language at 5. And "the Ability that correlates to his faculty degree"
depends on the `faculty` parameter's value, which no `Prereq` variant can read.
The German text confirms the reading the English OCR obscures: three Abilities at
5, not "Latin at 5" plus two unquantified.

The `(27 – Intelligence)` age floor is a second stated requirement with no engine
model; it goes to F-58.

**Severity:** wrong-rules-output

---

### F-57 — `virtue.doctor_in_faculty` — the `faculty` parameter is unbounded free text where the book names a closed set

**Passage** (ArMDE:3685, verbatim, English):
> The character has graduated from one of the higher faculties of a university, in medicine, civil or canon law, or theology, having already received his magister in artibus license, and may instruct fellow students.

**Current data:** `{"key":"faculty","type":"ref","domain":"text"}` — per B8, any
non-whitespace string is legal.

**Why it is wrong:** "one of the higher faculties" followed by an exhaustive list
is a closed option set: medicine, civil law, canon law, theology. A `text` domain
accepts anything, and the repo already uses `enumerated` with a `values` list for
exactly this shape (`virtue.folk_magic`, `flaw.offensive_to_beings` and siblings).
This is B01's **F-03** / **F-07** repeated, and here it has a second consequence
the earlier two did not: F-56's third prerequisite ("the Ability that correlates
to his faculty degree") is only ever encodable if the faculty is drawn from a
known set.

The parameter is consumed by no effect, which per B8 is normal and not itself a
defect — its job is to fill the `{faculty}` slot in the name, which both locales
do correctly ("Doctor in {faculty}" / "Doktor der {faculty}", the latter matching
`rules/source/de/translation-tables/reputationen.md:93`).

**Correct value:** `domain: "enumerated"` with four values for medicine, civil
law, canon law and theology. All four are named inside this passage, so unlike
B01's Q-01 there is nothing to source elsewhere.

**Severity:** wrong-rules-output (an unbounded picker accepts an illegal faculty)

---

### F-58 — `virtue.doctor_in_faculty` — four further stated rules are in neither locale

**Passages** (ArMDE:3687, :3689, :3691, verbatim, English):
> A character starting the game with this Virtue must be at least (27 Intelligence) years old.
>
> Like other working characters, he must spend two seasons a year practicing his profession, either teaching or working in a secular or ecclesiastical court. Both the Wealthy Virtue and the Poor Flaw are allowable, but players must decide what calamity befell such an erudite scholar if he is Poor, for which he receives a Bad Reputation at a level of 2.
>
> This Virtue is compatible with the Hermetic Magus, Mendicant Friar, and Priest Virtues. It is only available to male characters, with the exception of Doctors in Medicine who graduated from Salerno. That university does train female physicians.

**Current data:** `summary` in both locales is the opening sentence only. No
`description`.

**Why it is wrong:** four rules with real consequences reach no surface. The
`(27 – Intelligence)` age floor is a numeric creation constraint. The two seasons
a year is an ongoing obligation. The conditional Bad Reputation 2 is a *granted
Reputation* the engine would model if it could — but `ReputationType` has exactly
four values (`Local`, `Ecclesiastical`, `Hermetic`, `Academic`; confirmed in
`types.rs::ReputationType`) with no good/bad axis, and the grant is conditional on
holding `flaw.poor`, which `GrantsReputation` (A25) has no field to express — so
this one is doubly inexpressible and text is the only home. The compatibility
statement is correctly encoded as *no* `incompatible_with` (see also Q-07). The
male-only clause with its Salerno exception is Q-05's family.

**Correct value:** a `description` in **both** locales carrying the age floor, the
two-seasons obligation, the Poor/Bad-Reputation-2 rule and the Salerno exception.

**Severity:** lost-rule

---

### F-59 — `virtue.educated` — "You may purchase Academic Abilities" is the entry's first mechanical sentence, both locales' summary promises it, and the data does not deliver it

**Passage** (ArMDE:3713, verbatim, English — the entry's whole body):
> You have been educated in a grammar school, and may have attended a university or cathedral school. You may purchase Academic Abilities during character generation. During character generation you get an additional 50 experience points, which must be spent on Latin and Artes Liberales.

German, ArMDE:3713 (line-parallel):
> Du darfst während der Charaktererschaffung Akademische Fertigkeiten erwerben. Während der Charaktererschaffung erhältst du zusätzliche 50 Erfahrungspunkte, die du auf Latein und Artes Liberales verwenden musst.

**Current data:**
```
{"type":"restricted_ability_xp","amount":50,
 "abilities":["ability.artes_liberales","ability.dead_language"]}
```
and both locales' summary already states the permission — en: "+50 experience
points, spent only on Latin and Artes Liberales. **You may buy Academic
Abilities.**"; de: "…**Du darfst Akademische Fertigkeiten erwerben.**"

**Why it is wrong, and why A7's permission side-effect does not cover it.** A7
records that a `RestrictedAbilityXp` grant also confers permission to buy what it
funds. True — but it confers permission over the **lists the effect carries**, and
this effect carries an `abilities` list and **no** `categories` list. I read
`effective/xp.rs::ability_authorizations` to be sure: it extends `abilities` from
the effect's `ids` and `categories` from its `cats`, with nothing deriving one
from the other. So Educated authorizes `ability.artes_liberales` and
`ability.dead_language` — and nothing else. A grog or companion with Educated who
buys Philosophiae, Medicine, Theology or Church Lore draws
`ability_category_requires_virtue` from
`validation/authorization.rs::validate_ability_authorization`, against a character
whose Virtue says in its first mechanical sentence that he may have them.

This is worse than B01's nine `narrative` Social Status findings in one respect:
there the app's text was silent; here the app's own summary **tells the user in
both locales** that the permission exists and the validator then refuses it. It is
also the pattern's first appearance on a **General** (not Social Status) Virtue,
so the correction sweep must not scope itself to `social_status`.

Note the contrast with `virtue.craft_guild_training` (F-43), which ArMDE:3615
names as Educated's sibling: its four targets are all category `general`, so it
correctly needs no authorization. The book distinguishes them and so must the data.

**Correct value:** add
`{"type":"ability_authorization","categories":["academic"]}` alongside the
existing pool. `classification` stays `creation_effect`.

**Severity:** wrong-rules-output

---

### F-60 — `virtue.educated` — the 50-point pool funds any Dead Language, not Latin

**Passage:** ArMDE:3713 as quoted in F-59 — "must be spent on Latin and Artes
Liberales".

**Current data:** the `abilities` list above, naming `ability.dead_language`.

**Why it is wrong:** `ability.dead_language` is parameterized by `language`, and
per A7 the pool matches the Ability id with no instance test, so the 50 points
fund Greek, Hebrew, Aramaic or Gothic as readily as Latin. Identical to B01's
**F-31** and to F-36 and F-43 above — the fourth instance in 70 entries.

**Correct value:** an instance-scoped pool (a variant change, not a data edit).

**Severity:** wrong-rules-output (minor)

---

### F-61 — `virtue.elemental_magic` — the requisite-Form rule is in neither locale

**Passage** (ArMDE:3737, verbatim, English):
> In addition, if a spell with one of these Forms as its primary Form has another element as a requisite, you use the primary Form to calculate totals, even if the requisite is lower.

German, ArMDE:3737 (line-parallel):
> Darüber hinaus gilt: Wenn ein Zauber, der eine dieser Formen als primäre Form hat, ein anderes Element als Requisit benötigt, verwendest du die primäre Form zur Berechnung der Gesamtwerte, selbst wenn das Requisit niedriger ist.

**Current data:** one effect,
`{"type":"elemental_magic","forms":["art.aquam","art.auram","art.ignem","art.terram"]}`
— **correct**, and the arithmetic behind it (A42) is a faithful model of
ArMDE:3733/:3735 including the rounding the book's own worked example fixes
("21 experience points … 11 from Ignem" = `ceil(21/2)`). `summary` in both locales
is the opening sentence. No `description`.

**Why it is wrong:** the third paragraph is a separate rule about *requisites*,
not about XP redistribution, and it is the one clause of the passage that survives
into play. `engine-semantics.md`'s note on `spell_level_cap` names
"requisite-Art reduction" as an acknowledged engine approximation, so there is no
prospect of this being computed — which makes it exactly the D3 / F-33 case for
text.

**Correct value:** a `description` in **both** locales carrying the requisite rule.

**Severity:** lost-rule (minor)

---

### F-62 — `virtue.emir` — a rule stated by cross-reference to another Virtue, and the cross-reference is not followed

**Passage** (ArMDE:3745, verbatim, English — the entry's whole body):
> This is the same as the Knight Virtue, though due to the rather different upbringing of Muslim emirs, you are likely to be as skilled with hunting, religious teachings, and culture as you are with martial pursuits.

German, ArMDE:3745 (line-parallel):
> Dies entspricht der Ritter-Tugend, doch aufgrund der recht anderen Erziehung muslimischer Emire bist du wahrscheinlich ebenso versiert in Jagd, religiösen Lehren und Kultur wie in kriegerischen Belangen.

**And what it points at**, ArMDE:4197 (`#### Knight`, verbatim):
> You are a knight, a member of the noble classes and one of the elite warriors of Europe. Unless you are Poor, you may have high quality weapons and armor, and a horse. Typical armaments for a mid-13th century knight are lance, sword, heater shield, a complete mail suit, and a warhorse. You may take Martial Abilities during character generation. The Wealthy Virtue and Poor Flaw affect you normally. This Virtue is only available to male characters, and is compatible with the Landed Noble Virtue.

**Current data:** `"classification": "narrative"`, no `effects`.

**Why it is wrong:** "This is the same as the Knight Virtue" incorporates Knight's
mechanics by reference, and the computed one among them is the Martial-category
permission. `martial` is a gated category, so an Emir who buys Single Weapon draws
`ability_category_requires_virtue` against a character the book makes legal —
B01's F-09/F-10 shape, reached by one extra hop.

**Correct value:** `"classification": "creation_effect"` plus
`{"type":"ability_authorization","categories":["martial"]}`.

**A catalogue-wide pattern this opens, which later batches must check.**
`virtue.knight` itself (ArMDE:4195-4198) is *also* `narrative` with no effects, so
the permission is missing at **both** ends of the reference — the referring entry
and the referenced one. More generally: **a Virtue whose passage says "this is the
same as X" carries X's mechanics, and a screen looking for "may take … Abilities"
inside the entry's own cited line range cannot see them.** Every later batch
should treat "same as", "a variant of", "as the … Virtue" as a pointer to follow.
In this batch the same construction also appears at ArMDE:3709
(`virtue.dust_devil`, "This variant of the Skinchanger Virtue") — followed, and
cleared: `virtue.skinchanger` at ArMDE:4973 confers no Ability and no
gated-category permission, so Dust Devil inherits nothing computable and its
`uncomputed_rule` + description is correct. It also appears at ArMDE:3765
(`virtue.enticer_of_multitudes`, "a version of the Inspirational Virtue") —
followed, and cleared: `virtue.inspirational` (ArMDE:4143-4146) is itself
`narrative` with no effects, so there is nothing to inherit.

**Severity:** wrong-rules-output

---

### F-63 — `virtue.enchanting_ability` — the entry declares a choice the book demands, and nothing anywhere can act on it

**Passage** (ArMDE:3749, verbatim, English — the entry's whole body):
> When you set your mind to it, you can magically induce emotions and beliefs in others with a particular kind of artistic expression: music, dance, drawing, storytelling, even craftwork. Choosing this Virtue confers the Ability Enchanting (Ability) 1 (page 164).

and the Ability it confers, ArMDE:7446 (heading) and :7447:
> #### Enchanting (Ability)\*
> When you set your mind to it, you can influence others with a particular performance ability. …

**Current data:** `parameters: [{"key":"ability","type":"ref","domain":"ability"}]`
and `effects: [{"type":"ability_score_grant","ability":"ability.enchanting","amount":1}]`.

**Why it is wrong:** three linked problems.

1. **The grant ignores the choice.** `AbilityScoreGrant` (A9) stores its target
   directly and never reads a param, and A9 further records that the grant
   "applies only to the **parameter-less** instance". So whichever expression the
   player picks, the free score lands on one undifferentiated
   `ability.enchanting`.
2. **The target Ability is not parameterized, though both the Virtue and the
   Ability are titled `(Ability)`.** `rules/core/abilities.json` gives
   `ability.enchanting` no `parameter` key — unlike `ability.craft`,
   `ability.profession`, `ability.area_lore`, `ability.dead_language` and
   `ability.organization_lore`, which all have one. A character with Enchanting
   Music and Enchanting Storytelling therefore has **one** score, not two; and
   taking the Virtue twice with two different params yields two distinct
   `max_per_target` targets (B10 keys on the whole params map) whose grants then
   collapse to a single `max` (A9: grants do not stack).
3. **The `ability` domain cannot name the book's own examples.** Of "music, dance,
   drawing, storytelling, even craftwork", `rules/core/abilities.json` holds
   `ability.music` and `ability.craft`; there is no `ability.dance` and no
   `ability.drawing` (searched). Three of the five examples are unrepresentable in
   the declared domain.

**Correct value:** not settleable from this passage alone — see **Q-12**. The
minimum honest statement is that the parameter records a choice the engine acts on
in no way, which B8 calls normal *unless the passage makes the choice mechanical*
— and here the chosen expression is part of the Ability's own identity.

**Severity:** wrong-rules-output (two different Enchanting expressions are one
score) — but read Q-12 before acting

---

### F-64 — `virtue.enduring_constitution` — the +3 pain-resistance bonus is in neither the data nor either locale

**Passage** (ArMDE:3753, verbatim, English — the entry's whole body; "onespoint"
is the source's OCR):
> You can withstand pain and fatigue. Decrease the penalties for reduced Fatigue levels by one point, and reduce your total penalty from wounds by onespoint (but not below zero). You also get +3 on rolls to resist pain.

German, ArMDE:3753 (line-parallel, and it resolves the OCR):
> Verringere die Abzüge für verringerte Erschöpfungsstufen um einen Punkt und reduziere deinen Gesamtabzug aus Wunden um einen Punkt (jedoch nicht unter null). Du erhältst außerdem +3 auf Würfe, um Schmerz zu widerstehen.

**Current data:**
```
{"type":"health_mod","track":"wound_penalty","amount":1}
{"type":"health_mod","track":"fatigue_penalty","amount":1}
```
**Both are correct**, including the sign and the clamp: A36 states that a positive
`amount` *reduces* the penalty magnitude and that both computed tracks apply
`(base + delta).min(0)` — which is the book's own "(but not below zero)". Two of
the three sentences are modelled exactly. `summary` in both locales is the opening
sentence only; no `description`.

**Why it is wrong:** the third sentence — a flat, unconditional **+3** on a named
class of rolls — reaches the user nowhere. It is not expressible as an effect:
`AbilityRollMod` (A41) is scoped to a free-text *subject* of one Ability and is
surfaced-only, and "rolls to resist pain" names no Ability at all. So it is the
D3 / F-33 case for text, on an `in_play_effect` entry where nine siblings already
carry a `description`.

**Correct value:** a `description` in **both** locales carrying all three
sentences.

**Severity:** lost-rule

---

### F-65 — `virtue.curse_throwing` and `virtue.dowsing` — the German summaries render the game term *Ability* as "Fähigkeit"; canonical German is "Fertigkeit"

*Found by the verification pass, against two of my own OK verdicts. I had checked
both entries' `name` against the translation tables and both `effects` against the
passage, all four of which are correct, and did not re-read the body of the
summary sentence for a term substitution.*

**Current data**, `rules/i18n/de/virtues_flaws.json`:
> "Heile Krankheiten und entferne Flüche, indem du sie auf eine andere Person überträgst. Verleiht die **Fähigkeit** Fluchschleudern."
>
> "Finde nahe Dinge mit einer Wünschelrute. Verleiht die **Fähigkeit** Wünschelrutengehen."

**Why it is wrong — four witnesses, each checked:**

1. **The German rulebook says *Fertigkeit* in the very sentence being
   summarised.** ArMDE:3627 and :3705 (line-parallel):
   > Die Wahl dieser Tugend verleiht die Übernatürliche **Fertigkeit** Fluchschleudern 1.
   >
   > Die Wahl dieser Tugend verleiht dir die **Fertigkeit** Wünschelrutengehen 1 ([Seite 164]…).
2. **The canonical table fixes Ability = Fertigkeit.**
   `rules/source/de/translation-tables/fertigkeiten.md` is titled
   "Fertigkeiten / Abilities", and its rows give
   `Curse-Throwing* | Übernatürlich | Fluchschleudern` (:40) and
   `Dowsing* | Übernatürlich | Wünschelrutengehen` (:43).
3. **"Fähigkeit" is reserved for the *non*-Ability sense in the same corpus.**
   `grundbegriffe.md:93` warns explicitly not to confuse a faerie's *Fähigkeiten*
   with the *Fertigkeit* Guile; `tiere-kreaturen.md:144` uses "Übernatürliche
   Fähigkeit" for a **Power**. Using it for a named Ability collapses a
   distinction the tables draw on purpose.
4. **Four siblings on the same template get it right**, and so does one of my own
   fifteen:
   ```
   virtue.faerie_magic  … Verleiht die Fertigkeit Feenmagie auf 1.
   virtue.heartbeast    … Verleiht die Fertigkeit Herztier auf 1.
   virtue.premonitions  … Verleiht die Fertigkeit Vorahnungen auf 1.
   virtue.second_sight  … Verleiht die Fertigkeit Zweites Gesicht auf 1.
   virtue.corpse_magic  … die Übernatürliche Fertigkeit Leichenmagie mit einem Wert von 1.
   ```
   Corpse Magic is the tell: it mirrors its source sentence and is correct; the
   two defective ones are the two that were hand-condensed away from the source.

**I re-ran the measurement myself rather than take it on trust.** "Fähigkeit"
occurs 26 times in `rules/i18n/de/virtues_flaws.json`; exactly **three** of those
are the game term in the conferral formula — these two plus
`virtue.sense_holiness_and_unholiness` ("Verleiht die gleichnamige Fähigkeit",
outside this batch). The other 23 are the ordinary German word, including
`virtue.command_animals`' "hat die Fähigkeit, … zu befehligen", which copies
ArMDE:3577 verbatim and is correct. So the fix is three entries, not a sweep.

**Second defect in the same two summaries, this one in *both* locales.** The book
states the granted score — "Curse-Throwing **1**" (ArMDE:3627), "Dowsing **1**"
(ArMDE:3705) — and both summaries drop it:
> en: "…Confers the Curse-Throwing Ability." / "…Confers the Dowsing Ability."

while the four siblings above all carry it ("Confers the Second Sight Ability
**at 1**"). Verified across the English file: of the six entries using the
"Confers the … Ability" formula, four carry "at 1" and the two exceptions are
these — plus `virtue.sense_holiness_and_unholiness` again. The `effects` row has
`amount: 1`, so nothing computes wrong; this is displayed-text completeness.

**Correct value:**
> de, `virtue.curse_throwing`: "Heile Krankheiten und entferne Flüche, indem du sie auf eine andere Person überträgst. Verleiht die Übernatürliche Fertigkeit Fluchschleudern auf 1."
>
> de, `virtue.dowsing`: "Finde nahe Dinge mit einer Wünschelrute. Verleiht die Fertigkeit Wünschelrutengehen auf 1."
>
> en: "…Confers the Curse-Throwing Ability at 1." / "…Confers the Dowsing Ability at 1."

**Severity:** text — localization, which CLAUDE.md rates up ("real users, every
session, on every platform"), on a term the project declares canonical

## D2 — directed read

D2 asks what the book requires of a **granted** Virtue's preconditions. B01 found
nothing bearing on it. This span contains the closest thing found so far, and it
turns on a single verb.

**The grant.** ArMDE:3673 (`virtue.devil_child`, verbatim):
> The character gets the Demonic Might or Demonic Powers (player's choice) Minor Virtue free.

and the general Mythic Companion rule it instantiates, ArMDE:2638, verbatim:
> You gain a free Minor Virtue, normally specified by the Mythic Companion Virtue. In addition, you may take up to ten points of Flaws, and each point of Flaws is worth two points of Virtues. This produces a maximum of 21 points of Virtues and 10 points of Flaws. **Most Mythic Companion Virtues require you to take some particular Virtues and Flaws, these count against your maximum of 20 points of Virtues and 10 points of Flaws.**

**The two grantable items' own preconditions**, and note that they are worded
**differently from each other**:

ArMDE:3665 (`virtue.demonic_might`, verbatim):
> You may only **take** this Virtue if your character has the Demonic Blood Virtue.

ArMDE:3669 (`virtue.demonic_powers`, verbatim):
> Only a character with the Demonic Blood Virtue may **have** Demonic Powers.

**Why the difference matters for D2.** D2's question is whether a precondition is
a condition on *purchase* or a condition on *possession*. The book uses both verbs
here, one sentence apart, for the same precondition on two sibling Virtues. If the
distinction is deliberate, ArMDE:3669's "may **have**" is a possession condition
that a free grant must satisfy exactly as a purchase must, and the engine's
bought-only validators would be wrong at least for that shape. If it is
stylistic variation, the passage says nothing. **I cannot settle which it is from
the source and I am not picking one** — recorded as **Q-13**.

**What this batch can settle, and it is narrow.** The specific case does not
misfire today: Demonic Blood is a *required* Virtue for a Devil Child
(ArMDE:2654, verbatim — "All Devil Children must take the following Virtues: …
Demonic Blood (Major, Supernatural)"), and
`rules/core/mythic_companion_types.json` encodes exactly that as
`required_virtues`. So the granted Demonic Might/Powers always does in fact have
its precondition met, by a different mechanism. The engine's skip is unobservable
here.

**Budget evidence, offered as adjacent and explicitly not as D2's answer.** Three
passages in and around this span state *budget* treatment for a mandated Virtue or
Flaw, and they do not agree with each other:

| Passage | What it says | Against the engine's blanket grant-exemption (B3) |
|---|---|---|
| ArMDE:2638 | Mythic Companion required Virtues/Flaws "count against your maximum" | **cuts against** — they are charged |
| ArMDE:2664 | "Devil Children may take three more points of Virtues at no cost" | a *bonus*, which `bonus_free_virtue_points: 3` already models |
| ArMDE:3681 (Diedne Magic, F-47) | the compulsory Major Story Flaw "is in addition to your normal allowance of Flaws, and does not grant you any points" | a **third** treatment: off-budget on the Flaw side *and* yielding no Virtue points |
| ArMDE:3565 (Clan Ilfetu, F-37) | an associated Mystery Virtue "must be purchased with the normal allowance of ten points" | **cuts against** — charged |

So the book specifies at least three distinct budget treatments for mandated
items and the engine implements one. This is the same observation B01 recorded
from ArMDE:3515 and it is now four passages strong. It is a **budget** question,
not a **precondition** question, and it does not answer D2 — but if D2 is being
decided from evidence about how the book treats grants generally, this is the
evidence.

## Negatives cleared

Recorded so the corrections pass and later batches do not re-open them.

- **`virtue.the_enigma`'s `house.criamon` prerequisite.** ArMDE:3761 words
  membership as a *consequence* — "You have been initiated into the Outer Mystery
  of The Enigma …, **and thus are** a member of House Criamon" — while the data
  encodes it as a requirement. I considered this a possible check-7 violation
  ("no requirement the passage does not state") and cleared it: the same sentence
  closes "Note that all Criamon magi get this Virtue free at character creation",
  `rules/core/houses.json` grants `virtue.the_enigma` from `house.criamon`
  accordingly, and B6's tri-state means `house` yields `Unknown` (no issue) rather
  than an error for a character with no House. The set of characters the two
  readings admit is the same, so the prereq is at worst a restatement. The
  `ability_score_grant ability.enigmatic_wisdom 1` is correct against "You have a
  score of 1 in Enigmatic Wisdom", and since `ability.enigmatic_wisdom` is
  category `arcane` (gated), the grant's A9 permission side-effect is what makes
  the Ability legal — it is load-bearing, not decorative.
- **The six Supernatural Virtues that confer an Ability at 1** —
  `corpse_magic`, `crafters_healing`, `curse_throwing`, `dowsing`, `embitterment`,
  `enchanting_ability`. Each passage says "confers the Ability X 1" (or "at a
  score of 1") and each carries exactly `ability_score_grant X 1`. Verified that
  every target id exists in `rules/core/abilities.json` with category
  `supernatural`; `supernatural` is **not** in `categories_requiring_virtue`, so
  no separate authorization is needed, and A9's grant doubles as the supernatural
  permission anyway. **The `effects` on all six stand.** Two of the six carry a
  further problem that is not in their effects: `enchanting_ability` (F-63) and,
  found by the verification pass after I had cleared them, `curse_throwing` and
  `dowsing` in their *text* (F-65).
- **`virtue.craftsman` is correctly `narrative`.** ArMDE:3623's whole body is
  "You live by making and selling goods. You are probably a free resident of a
  town, but you may be from a rural area. The Wealthy Major Virtue and Poor Major
  Flaw affect you normally." The last sentence states that **no** special rule
  applies, which is not a mechanical clause, and its correct encoding is the
  absent `incompatible_with` the entry already has. Contrast F-41.
- **`virtue.demonic_might` / `virtue.demonic_powers`' multiplicity fields.**
  `max_per_target: 255` + `max_share_of_kind: {1, 2}` is the right pair of knobs
  for "may take this Virtue more than once, though it can account for no more than
  half of the character's total Virtues" — B10 confirms `max_per_target` is the
  axis keyed on the whole params map (and with no params every copy is one
  target), and that `max_share_of_kind` compares
  `points · denominator > total · numerator`, i.e. warns strictly above half.
  `max_total` correctly stays at its 255 default. Both prerequisites
  (`has virtue.demonic_blood`) are correct against ArMDE:3665 and :3669.
- **`virtue.deft_form`.** ArMDE:3647 is fully covered by
  `special_casting_mod deft_form param:"form"` over a `form`-domain parameter:
  A40 computes it (one of the three non-surfaced kinds), and
  `ruleset/integrity.rs::validate_effect_refs` *requires* the `form`-domain param
  for this kind, so it cannot silently dangle. The trailing caveat "Voice Range
  spells still have a Range based on how loudly you are speaking" restates the
  ordinary Range rule rather than adding one.
- **`virtue.cyclic_magic_positive`'s `scope: "all"`.** Checked against A30's
  `CastType::matches` table: `all` is accepted by Formulaic, Ritual and
  Spontaneous, which is what "all Casting Scores" means. Number, sign and scope
  are each right; only the condition is missing (F-45).
- **All seven `uncomputed_rule` entries in the batch are clean** —
  `clear_thinker`, `command_animals`, `convoluted_mind`, `death_prophecy`,
  `dust_devil`, `enduring_magic`, `enticer_of_multitudes`. Each carries a
  `description` in
  **both** locales reproducing the passage in full, each states a rule the engine
  has no variant for (a +3 on resistance rolls; a count of commandable animals; a
  +3 on Infernal Lore rolls; a no-death-until-condition; a shapechange with a
  focus object; a duration multiplier rolled secretly by the storyguide; a
  +5-or-more Personality Trait
  roll), and none carries `effects`. `tainted` is set on exactly the three whose
  descriptor line says *Tainted* (`command_animals`, `convoluted_mind`,
  `enticer_of_multitudes`) and on `demonic_blood`, matching ArMDE:3576, :3602,
  :3764 and :3650 respectively.
- **`virtue.convoluted_mind`'s +3 Infernal Lore bonus** is B01's **Q-06** shape
  (an `AbilityRollMod` candidate carried as prose instead), not a new finding. The
  number is in `description` verbatim in both locales, so nothing is lost; whether
  it should be an effect row is the consistency question Q-06 already asks.

## Open questions

### Q-09 — `virtue.common_sense` — is a guaranteed storyguide intervention a mechanical clause, or is it colour?

**Passage** (ArMDE:3599, verbatim, English — the entry's whole body):
> Whenever you are about to do something contrary to what is sensible in the game setting, common sense (the storyguide) alerts you to the error. This is an excellent Virtue for a beginning player, as it legitimizes any help the storyguide may give.

German, ArMDE:3599 (line-parallel):
> Wann immer du dabei bist, etwas zu tun, das dem gesunden Menschenverstand im Spielumfeld widerspricht, macht dich der gesunde Menschenverstand (der Spielleiter) auf den Fehler aufmerksam.

**Current data:** `"classification": "narrative"`, no `effects`, no
`description`. Both locales' summary is the first sentence.

**Reading A — `narrative` is right.** There is no number, no roll, no modifier and
no target. README defines `narrative` as "the book states nothing mechanical"
and this is as close to pure procedure-and-colour as a Minor Virtue gets.

**Reading B — `uncomputed_rule`.** "Whenever … the storyguide alerts you" is an
*obligation on play*, stated with "whenever", and README's own gloss of
`uncomputed_rule` explicitly names "GM judgement" as a member of the class. On
this reading Common Sense buys the player a guaranteed entitlement for one Virtue
point, and calling that nothing-mechanical suppresses the only thing the Virtue
does.

**What I could not settle it with:** the README's two definitions overlap exactly
here — one says "nothing mechanical", the other says "GM judgement" *is* the
uncomputed class — and no ArMDE passage distinguishes them. It is a convention
question, not a rules reading, and **it is catalogue-wide**: the Virtue is used as
the reference point by a second entry in this very batch (ArMDE:3603, Convoluted
Mind: "has a feeling, similar to the Common Sense Virtue, that prevents him from
doing stupid things"), and every "the storyguide tells you" entry in the remaining
17 batches turns on the same answer. Worth settling once.

---

### Q-10 — `virtue.covenfolk` — the two canonical German sources give two different names

**Current data:** `rules/i18n/de/virtues_flaws.json` → `"name": "Konventsbewohner"`.

**Source 1 — the German rulebook**, ArMDE:3609 (line-parallel with
`#### Covenfolk`):
> #### Konventsbewohner

**Source 2 — the Virtue/Flaw translation table**,
`rules/source/de/translation-tables/tugenden-fehler.md:244`:
> | Covenfolk | Konventsmitglied | |

**Source 3 — the covenant translation table**,
`rules/source/de/translation-tables/konvent.md:38`:
> | Covenfolk | Konventsbewohner | Alle Bewohner des Konvents außer Magi und Gefährten |

**Reading A — `Konventsbewohner` (current).** Two of three sources agree, and one
of them is the rulebook, which CLAUDE.md names "the **source of truth for German
translations**". The German body text at :3611 uses "Konventspersonal" /
"Konventsangestellter" in the same register.

**Reading B — `Konventsmitglied`.** CLAUDE.md is categorical that a term whose
English form appears in a table MUST match the table's `Deutsch (DE)` value, and
`tugenden-fehler.md` is *the* table for Virtue and Flaw names — the most specific
authority for this particular string. Note that the same file at :254 renders
`Magical Covenfolk` as `Magisches Konventsmitglied`, so the table is internally
consistent and the conflict is genuinely across files, not a typo.

**What I could not settle it with:** the two tables disagree with each other and
CLAUDE.md designates both as canonical without a tie-break.
`rules/source/de/translation-tables/README.md` records resolved naming conflicts
and this pair is not among them. **This is B01's Q-04 (`auf`/`für`) recurring with
different words**, which suggests the underlying need is a stated precedence rule
— rulebook vs. table, and thematic table vs. `tugenden-fehler.md` — rather than
two one-word decisions. Whichever way it goes it should be written into that
README, because `Magical Covenfolk` (outside this batch) must follow the same
answer.

---

### Q-11 — `virtue.domestic_animal` — "animals only" is an eligibility restriction with no engine model; does it make the entry non-`narrative`?

**Passage** (ArMDE:3700-3701, verbatim, English — descriptor line and body):
> *Free, Social Status, animals only*<br>
> The character is an animal who is the property of a covenant or character, and is supplied with food and shelter. He is expected to serve his master in return for this provender, as a mount, beast of burden, hunter, or so forth.

German, ArMDE:3700 (line-parallel):
> *Frei, Sozialer Status, nur Tiere*<br>

and the sidebar the book places immediately above it, ArMDE:3695:
> An animal can have personality and add greatly to stories, just as a human can. Despite their obvious limitations, it can be a great deal of fun to play one as a grog, probably while running human grogs as well.

**Current data:** `"classification": "narrative"`, `categories: ["social_status"]`,
`entity_kinds: ["character"]`, no effects, no prereqs. The body prose is pure
colour and is faithfully summarised in both locales.

**Reading A — `narrative` is right.** The body states nothing mechanical, and
"animals only" is a *descriptor tag*, on the same line as the magnitude and the
category rather than in the rules text. `EntityKind` has exactly two values
(`Character`, `Covenant`), so there is no axis for it and no rule is being lost
that any other entry's data would carry.

**Reading B — `uncomputed_rule`.** "animals only" is an absolute eligibility bar
of exactly the kind this audit treats as a rule elsewhere ("Magi and Grogs may not
take this Virtue", B01 F-24). D3 says an engine that structurally cannot express a
rule gets `uncomputed_rule` with the rule written out in both locales, and this is
structurally inexpressible today. On that reading the entry needs a `description`
saying it is for animal characters only.

**What I could not settle it with:** this is **B01's Q-05 with a different axis**
— there it was "only available to male characters", here it is "animals only", and
in both cases the question is whether an eligibility restriction the engine has no
model for is a rule the catalogue must carry. B01 parked Q-05 without letting it
move a classification; I have done the same and marked the class `?`. It should be
answered together with Q-05, because the answer is the same answer. I flag one
difference worth weighing: unlike the male-only clauses, "animals only" is this
entry's **only** candidate mechanical content, so it alone decides the
classification rather than riding along with other defects.

---

### Q-12 — `virtue.enchanting_ability` — should `ability.enchanting` be a parameterized Ability, and is `domain: "ability"` the right domain for the Virtue's parameter?

**Passages** (verbatim, English):

ArMDE:3749 (the Virtue):
> …a particular kind of artistic expression: music, dance, drawing, storytelling, even craftwork. Choosing this Virtue confers the Ability Enchanting (Ability) 1 (page 164).

ArMDE:7446-7447 (the Ability):
> #### Enchanting (Ability)\*
> When you set your mind to it, you can influence others with a particular performance ability. … When you use Enchanting Ability, roll a die (stress or simple, depending on the situation) and add Communication and Enchanting Ability.

**Current data:** Virtue parameter `{"key":"ability","domain":"ability"}`,
consumed by no effect; Ability `ability.enchanting` with **no** `parameter` key.

**Reading A — the Ability should be parameterized** (`"parameter": "ability"`, as
`ability.craft` has `"parameter": "craft"`), the Virtue's grant should be able to
target an instance, and the two would then line up. This matches the book's own
title for both, and it is the only reading under which Enchanting Music and
Enchanting Dance are two different scores. Cost: `AbilityScoreGrant` (A9)
"returns 0 immediately when `parameter.is_some()`", so the grant would stop
working entirely until the variant learns to read a param — this is not a
data-only change.

**Reading B — the current shape is deliberate.** The `(Ability)` in the title
names the *performance* Ability used alongside it (Communication + Enchanting
Ability is the roll, per :7447), not a sub-instance of Enchanting itself — so one
Enchanting score applied through different performance Abilities is a coherent
reading, and the Virtue's `ability`-domain parameter records which performance
Ability that is.

**What I could not settle it with:** the two passages support the two readings
respectively and neither is decisive. Reading B is also embarrassed by the data:
of the book's five examples only `ability.music` and `ability.craft` exist as
Ability ids — there is no `ability.dance`, no `ability.drawing`, and
"storytelling" would have to be `ability.profession` with an instance the
`ability` domain cannot express — so under Reading B the parameter's domain is
wrong for three of five stated options. **Whichever way it goes, the fix touches
`rules/core/abilities.json`, which is outside this audit's V/F remit**, so it
needs the user rather than the corrections pass. The defect in F-63 stands under
both readings; only its correct shape is in doubt.

---

### Q-13 — does the book distinguish "may only **take**" from "may only **have**" when stating a Virtue's precondition?

**Passages** (verbatim, English, one entry apart):

ArMDE:3665 (`virtue.demonic_might`):
> You may only **take** this Virtue if your character has the Demonic Blood Virtue.

ArMDE:3669 (`virtue.demonic_powers`):
> Only a character with the Demonic Blood Virtue may **have** Demonic Powers.

**Why it matters:** this is D2's question in the book's own words. D2 asks whether
a *granted* Virtue must satisfy the same preconditions as a bought one; the engine
says no (`validation/prereq.rs` iterates bought `entity.selections`, B6). A
"may take" condition is a condition on purchase and is consistent with the
engine's behaviour. A "may have" condition is a condition on possession and is
not — a free grant confers possession.

**Reading A — the distinction is deliberate.** The two sentences sit one entry
apart, written by the same hand about the same precondition, and the verbs differ.
Under this reading Demonic Powers' precondition survives a grant and the engine is
wrong for that shape.

**Reading B — it is stylistic variation.** The two Virtues are obvious twins
(same magnitude, same category, same precondition, same "more than once … no more
than half" clause), and nothing else in either passage suggests the author meant
two different rules.

**What I could not settle it with:** no third passage in this span uses either
verb about a precondition, and the one case where it could bite does not
(ArMDE:2654 makes Demonic Blood a *required* Virtue of a Devil Child, so the
granted Might/Powers always has its precondition met by another route). I did not
search outside the batch's span. **This is offered to D2 as evidence, not as an
answer**, and per the batch rules I have not marked either validator's behaviour
correct or defective.

---

### Q-14 — `virtue.deft_form` — where is the threshold at which an effect-bearing entry owes a `description`?

*Raised by the verification pass, which declined to overturn on it.*

**Passage** (ArMDE:3647, third sentence, verbatim):
> Voice Range spells still have a Range based on how loudly you are speaking.

**Current data:** the `deft_form` effect is correct and genuinely computed (see
"Negatives cleared"). Displayed text is the summary alone — "You are particularly
skilled with one Form." / "Du bist mit einer Form besonders geschickt." — which
conveys neither the waiver the effect implements nor this caveat on it.

**Reading A — it is a defect.** B01's F-33 test applies: the passage outruns the
effect and the residue is a real limitation on the benefit, so a `description` is
owed in both locales.

**Reading B — it is within tolerance.** The practice is a **minority** one, and
the calibration cases are all much heavier. Nine of 93 `in_play_effect` entries
carry a description (`flaw.afflicted_tongue`, `flaw.lame`, `flaw.hobbled`,
`flaw.missing_eye`, `flaw.missing_hand`, `flaw.palsied_hands`,
`flaw.flawed_parma_magica`, `flaw.limited_magic_resistance`,
`virtue.ways_of_the_land`), and each carries *substantial* unmodelled mechanics —
Afflicted Tongue adds an extra botch die plus a penalty to all voice rolls on top
of its `casting_total_mod`. Deft Form's residue is one clarifying sentence and is
thinner than every calibration case.

**What could not settle it:** nothing in `docs/vf-audit/` or
`crates/arm-rules/RULES.md` states a threshold. **This is B01's Q-03 sharpened
from "may an effect-bearing entry carry the residue as text" (answered: yes, nine
do) to "how much residue triggers the obligation", and it now needs a number or a
rule, because this batch proposes nine new descriptions and one of them is this
thin.**

---

### Q-15 — `virtue.craftsman` — is "The Wealthy Major Virtue and Poor Major Flaw affect you normally" a mechanical clause?

*Raised by the verification pass. Both of us independently reached the same
verdict — `narrative` — for the same reason, and it flagged the reasoning as
unsafe rather than the verdict as wrong.*

**Passage** (ArMDE:3623, verbatim, the entry's whole body):
> You live by making and selling goods. You are probably a free resident of a town, but you may be from a rural area. The Wealthy Major Virtue and Poor Major Flaw affect you normally.

**Reading A — `narrative` (current, and what both passes leaned to).** The
sentence asserts that the **default** applies; it states no restriction, no
number, no cap. It is the *absence* of a rule said out loud, because the siblings
in the same block do restrict — ArMDE:3611 (Covenfolk) and :3631 (Custos), which
are F-41. All sixteen other Free Social Status entries in the catalogue are
`narrative`.

**Reading B — `uncomputed_rule`.** Against ArMDE:2816 — "All characters must take
one Social Status, and may only take more than one if the descriptions of the
Virtues or Flaws **explicitly note that they are compatible**" — the book treats a
compatibility statement in a Social Status descriptor as operative text. On that
reading "affect you normally" is an explicit compatibility ruling, hence
mechanical, and must be written into `description` in both locales.

**What could not settle it:** the README's partition turns on whether the book
"states nothing mechanical", and a sentence that mechanically states *nothing is
restricted* sits exactly on the line. This governs a class, not an entry, and it
is adjacent to **Q-07** (ArMDE:2816's one-Social-Status rule is enforced nowhere)
— they should probably be answered together.

---

### Q-16 — `virtue.dust_devil` — how much of Skinchanger does "this variant" inherit?

*Raised by the verification pass while following the cross-reference F-62 asks
later batches to follow. It did not overturn, judging the current state safe.*

**Passages** (verbatim, English):

ArMDE:3709 (Dust Devil) gives three facts of its own — Size +1 form, "can assume
the form **at any time**", a glass amulet as the focus object — and is silent on
the rest.

ArMDE:4974 (Skinchanger) supplies what it is silent about:
> While in physical contact with it, you may transform into the form of the animal represented by the item. The transformation takes one full round … If the item is stolen, the new owner has an Arcane Connection to you … The character has the normal physical characteristics of the animal, except that **+3 is added to the character's Soak score** (in animal form only).

**Reading A:** Dust Devil is Skinchanger with three overrides, so the +3 Soak and
the item rules carry over — and the +3 Soak is a `soak_mod` this entry is missing.

**Reading B:** "can assume the form at any time" already contradicts Skinchanger's
one-round and physical-contact clauses, so the variant restates what it keeps, and
a Soak bonus expressed as "the normal physical characteristics of the animal,
except…" does not travel to a whirlwind.

**What could not settle it:** the book states no general rule for what a "variant
of" entry inherits. One data point cuts for Reading B — ArMDE:4978, Skinchanger
(Dove), is "a more powerful variant of the Minor Virtue above" and **restates the
Soak clause explicitly** at :4984, so a variant that means to keep it says so —
but one data point is not a rule. The current data (`uncomputed_rule`, own passage
quoted in both locales, no effects) is safe under either reading, which is why
this is a question and not F-66; under Reading A it becomes a lost rule.

## Sub-agent reconciliation

One independent agent re-derived, from the rulebook alone, the 15 entries I had
marked completely clean. It was told that overturning one would be a success and
that inventing one would not, and it was not allowed to read this file.

**Result: 13 confirmed, 2 overturned.**

| Outcome | Entries |
|---|---|
| Confirmed OK | `clear_thinker`, `command_animals`, `convoluted_mind`, `corpse_magic`, `crafters_healing`, `craftsman`, `death_prophecy`, `deft_form`, `dust_devil`, `embitterment`, `enduring_magic`, `the_enigma`, `enticer_of_multitudes` |
| **Overturned** | `curse_throwing`, `dowsing` → **F-65** |

**What the overturn was worth.** It found a class of defect neither my pass nor
B01's looked for: not a missing rule or a wrong number, but a **term
substitution inside otherwise-correct German prose**. I had checked every German
*name* against the translation tables and every effect against the passage, and
both of those were clean on both entries — the error was in the body of a summary
sentence, in a word the tables govern and I had not thought to check there. It
generalises immediately (a third instance,
`virtue.sense_holiness_and_unholiness`, is outside this batch) and it gives every
later batch a concrete instruction: **check German summary and description prose
for table-governed terms, not just the `name` field.**

**I verified both overturns and the counts behind them myself** rather than
accepting the report: the three game-term uses of "Fähigkeit", the four correct
siblings, the two dropped "at 1"s, and the source sentences at ArMDE:3627/:3705.
All confirmed.

### The measurement that weakens six of my own findings

The pass measured the description precedent to calibrate Q-14, and the number it
produced cuts against me. **I re-ran it:**

| Classification | Entries | Carrying an English `description` |
|---|---|---|
| `in_play_effect` | 93 | **9** |
| `creation_effect` | 125 | **0** |

B01's F-33 established the precedent as "nine `in_play_effect` entries already do
this". That is true — **and it is entirely confined to `in_play_effect`.** Six of
this batch's findings propose a `description` on a **`creation_effect`** entry,
where the count is zero: **F-37** (`clan_ilfetu`), **F-52** (`demonic_blood`),
**F-53** (`demonic_might`), **F-55** (`devil_child`), **F-58**
(`doctor_in_faculty`), **F-61** (`elemental_magic`) — and **F-42** (`custos`)
makes it seven, since it proposes `creation_effect` *plus* a description.

This does **not** retract those findings: the rules really are lost, the passages
really do outrun the data, and D3's rationale reaches them regardless of
classification. But it changes what they are. They are **not** deviations from an
established practice, as F-33's framing implied — they are a **new** practice on
the `creation_effect` side, with no precedent at all, proposed seven times in one
batch. That is a decision the corrections pass should make deliberately rather
than inherit, and it belongs with **Q-14** and B01's **Q-03**.

Three of this batch's description findings are on the safer ground: **F-40**
(`commanding_aura`), **F-47** (`diedne_magic`) and **F-64**
(`enduring_constitution`) are all `in_play_effect`, where the nine-entry precedent
does apply.

### Negatives the pass cleared, recorded so the corrections phase does not reopen them

- **`virtue.the_enigma` needs no `ability_authorization`** even though
  `ability.enigmatic_wisdom` is category `arcane` (a gated category, confirmed at
  ArMDE:7455 "Enigmatic Wisdom* … (Arcane)"), for two independent reasons:
  `effective/xp.rs::ability_authorizations` folds `AbilityScoreGrant` straight
  into the permitted set, and `validate_ability_authorization` exempts any
  `is_magus` profile, which a Criamon necessarily is. This independently confirms
  the reading in my own "Negatives cleared".
- **`virtue.convoluted_mind` needs no Infernal Lore authorization**, despite its
  +3 on Infernal Lore rolls and `ability.infernal_lore` being `arcane`. The
  book's own contrast settles it: ArMDE:3655 (Demonic Blood, F-50) spells the
  permission out where it is meant, and ArMDE:3603 says nothing of the kind.
  Source or nothing.
- **`virtue.command_animals` confers no Ability.** There is no "Command Animals"
  Ability: the Abilities chapter runs `#### Code of Hermes*` (ArMDE:7357)
  straight to `#### Common Law*` (:7361), `rules/core/abilities.json` has no such
  id, and ArMDE:3577's lower-case "the ability to mentally command" is not the
  book's conferral formula.
- **`virtue.death_prophecy` is correctly `uncomputed_rule`, not an `aging_mod`.**
  "cannot die as a result of wounds or old age" (ArMDE:3641) matches none of the
  eight `AgingEffect` kinds — `crisis_survival` pushes a *numeric* modifier,
  `no_aging` stops only Characteristic drops, `no_apparent_aging` freezes only
  appearance. D3's case exactly, and the full text ships in both locales.
- **`virtue.dust_devil` correctly carries no `SizeDelta`.** ArMDE:3709's "Size +1"
  is the Size of the *assumed form*; `SizeDelta` would change the character's.
- **`virtue.enticer_of_multitudes` inherits nothing gated from Inspirational**
  (ArMDE:4143-4145 grants no Ability and no category permission) — and note the
  numbers legitimately differ, +3 there against "+5 or more" at ArMDE:3765, which
  the description correctly carries as +5.
- **No entry is missing a `text`-domain parameter.** All 19 `text`-domain
  parameters in the catalogue sit on entries whose *name* carries a parenthesised
  slot; Death Prophecy's death condition and Command Animals' animal species are
  narrative specifics on unslotted names.
- **`virtue.the_enigma`'s `[3759, 3761]` is not short by a line.** A
  catalogue-wide contiguity check found 62 non-adjacent pairs, many one-line gaps
  of exactly this shape, so there is no uniform convention being violated.

### Incidental, outside this batch — recorded so they are not lost

- **`virtue.sense_holiness_and_unholiness`** carries F-65's "Fähigkeit" defect and
  the dropped granted score. B08's range.
- **`virtue.inspirational`** (ArMDE:4143-4145, "+3 bonus to rolls for appropriate
  Personality Traits") is `narrative`, while the structurally identical
  `virtue.enticer_of_multitudes` in this batch is correctly `uncomputed_rule`.
- **`virtue.skinchanger`** (ArMDE:4972-4974, "+3 is added to the character's Soak
  score", "between Size -10 … and Size +2") is `narrative` with no effects —
  relevant to both F-62's cross-reference pattern and Q-16.
- **`virtue.sharp_ears`** (ArMDE:4952, "+3 bonus to all rolls involving
  hearing"), `virtue.keen_vision` and `virtue.strong_willed` are all `narrative`
  while the structurally identical `virtue.clear_thinker` in this batch is
  `uncomputed_rule`. A consistency question for whichever batch reaches them.
- The pass independently re-derived **B01's F-16** (`virtue.rard`) from the index
  mismatch, without having read B01 — which is a useful cross-check that the
  scanno finding was real.

### Process notes

- The verification agent reports that it **also received the "auto mode" Bash
  injection** telling it to read with `cat`/`head`/`sed -n` and write with
  `sed`/heredocs, and that it ignored it per CLAUDE.md. It changed no file and ran
  no git command.
- It spawned no sub-agents, as instructed.


