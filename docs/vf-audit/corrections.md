# Accumulated correction list — V/F audit

**Generated 2026-09-21** from `docs/vf-audit/batch-01.md` … `batch-16.md`, plus
`decisions.md` (D1–D7) and `README.md`.

**What this file is.** The single index of everything the Virtue/Flaw audit has
found, and the input to Phase 2 (the fixing pass). It is **authoritative for
*what* was found** — the finding number, the entry it lands on, the kind of
defect, its status. The **batch files remain authoritative for the evidence**:
the verbatim passage, the current data, the argument, and the correct value. A
row here is a pointer, never a substitute. Every row names its batch file so the
full reasoning is one read away.

**What it is not.** Not a transcript, and not a decision record. Rulings live in
`decisions.md`; open questions that still need Norbert are listed in § 6 of this
file but are *settled* there, not here.

**Coverage.** Findings **F-001 … F-501** (F-44 was never issued — see the note
below), open questions **Q-01 … Q-129**, batches **B01–B16** (entries 0–559 of
655, ArMDE:3362-6662). **B17–B19 (95 entries, ArMDE:6663-7119) have not run**,
so this file is complete for what has been checked and incomplete for the
catalogue.

**F-44 does not exist.** `batch-02.md:534` records the number as reserved for a
finding that was withdrawn before the batch was written. It is not a gap in this
index and needs no lookup.

**One gap in this file's own assurance, stated rather than papered over.** The
agent that built it was killed by a session rate limit after writing the last
section but **before running its verification pass**, so no second reader has
re-derived these 500 rows. The orchestrator independently confirmed the
load-bearing figures — 500 finding headings across the sixteen batch files by
three separate counts, 500 rows in § 1, the F-44 gap, and one duplicate
determination (F-479 ≡ F-193) spot-checked against both batch files — but the
per-row kind, severity and status assignments in § 1 carry **one** reader's
judgement, not two. § 4.2 is likewise honest that its nine duplicates are a
**floor, not a ceiling**. Treat a single row as a pointer to be confirmed at the
batch file, which is what § 0 says anyway; treat the counts and the groups as
sound.

---

## 0. How to read this file

### 0.1 Kind-of-defect vocabulary

Every finding carries one primary kind, and where it genuinely spans two, a
secondary after a `+`. Defined once here; used nowhere else in the audit, so it
is this file's own vocabulary and the batch files do not use these tokens.

| Token | Meaning |
|---|---|
| `class` | **Misclassification.** `classification` is the wrong one of the four. Almost always `narrative` on a passage that states mechanics. |
| `desc` | **Missing description (D5).** A mechanical clause the engine does not compute, reaching the user in neither locale. The single largest group. |
| `auth` | **Missing authorization.** A stated permission to buy a gated Ability or Ability category, carried by no `ability_authorization`. Frequently produces a *hard error on a legal character*. |
| `effect` | **Missing effect.** A stated mechanic the engine can already express, carried by no `effects` row at all. |
| `scope` | **Wrong scope.** The effect fires more widely or more narrowly than the passage says — a conditional folded in flat, a Form-specific bonus applied to all, a bonus scoped to one total landing on several. |
| `number` | **Wrong number / wrong arithmetic.** A shipped value contradicts the book, or a computation double-counts. |
| `param` | **Missing or wrong parameter.** No parameter records a choice the book demands; a parameter is unbounded free text where the book names a closed set; a missing `max_total` lets a once-only Virtue be taken repeatedly. |
| `incompat` | **Missing or wrong incompatibility.** A stated mutual exclusion absent from `incompatible_with`, or one present that the book does not state. |
| `prereq` | **Missing or wrong prerequisite.** A stated eligibility gate (score minimum, House, Social Status, entity kind, sex, age) enforced nowhere, or enforced wrongly. |
| `rep` | **Reputation defect.** A granted Reputation carried by no effect, at the wrong level, with a hardcoded audience the book leaves open, or of the wrong polarity. |
| `mag` | **Magnitude / category / kind error.** Wrong magnitude tier, wrong `kind`, a category filed in the wrong list, a missing magnitude variant. |
| `src` | **`source.lines` or anchor error.** The cited range bisects a passage, swallows a neighbour or a sidebar, or stops short of the rule it cites. |
| `text` | **Shipped text defect.** A `name`, `summary` or `description` in `rules/i18n/{en,de}/` that is truncated, mistranslated, uses a non-canonical term, or contradicts its own source. |
| `table` | **Translation-table defect.** A row in `rules/source/de/translation-tables/` that is wrong. Governed by **D6** — fixed in this repo *and* in `arm-de-translation`, immediately, not deferred to Phase 2. |
| `engine` | **Engine gap.** No mechanism exists to express the rule; the fix is Rust, not data. |
| `prov` | **Provenance defect.** `RULES.md`, a Rust doc comment, or another declared authority states something the source contradicts; or a catalogue entry the book defines is absent entirely. |
| `audit` | **Audit-record defect.** The audit's own records are wrong or incomplete. |

### 0.2 Severity normalization — read this before trusting a severity count

The sixteen batches used **two different severity vocabularies**, and neither is
a scale this file can count directly:

- **B01–B09** rate by **defect class**: `wrong rules output`, `lost-rule`,
  `lost-rule / provenance`, `data loss` / `lost player choice`, `localization
  defect` / `text`, `provenance`, `consistency`.
- **B10–B16** rate on a **three-point scale**: `high` / `medium` (or `moderate`)
  / `low`, with `medium-high` and `low-medium` used.

I normalized both into `H` / `M` / `L` by the table below. **The H/M/L counts in
§ 7 are therefore derived by me, not quoted from the batches.** Where a batch
gave a compound severity ("wrong rules output plus data loss", "lost rule,
downgraded from wrong-rules-output"), I took the *highest* component.

| Batch term | Normalized |
|---|---|
| `wrong rules output`, `wrong-rules-output`, `data loss`, `lost player choice`, `high`, `HIGH`, `medium-high`, `MEDIUM-HIGH` | `H` |
| `lost-rule`, `lost rule`, `lost-rule / provenance`, `moderate`, `MEDIUM`, `medium`, `low-to-moderate`, `low-medium`, `LOW-MEDIUM`, `localization defect`, `text` | `M` |
| `provenance`, `consistency`, `low`, `LOW` | `L` |

`localization` / `text` is normalized to `M`, not `L`, because CLAUDE.md rates
localization defects *up* ("real users, every session, on every platform") and
several batches say so explicitly in the finding body. A pure documentation
defect with no runtime or user-facing consequence (`RULES.md` wrong, a doc
comment wrong, a `source.lines` range off by a line) is `L`.

### 0.3 Status vocabulary

| Status | Meaning |
|---|---|
| `live` | Stands. Phase 2 must fix it. |
| `withdrawn` | Retracted, with what superseded it. **Do not work these.** |
| `blocked` | The remedy depends on a decision that is not yet taken. Named in the row. |

A row may additionally carry `dup→F-NNN`, meaning **this index** judges it the
same defect as an earlier finding. Where the duplication is exact the status is
`withdrawn`; where it is partial the status stays `live` and the marker warns
that the two must be worked once. See § 4.

`n/s` in the severity column means **the batch stated no severity** for that
finding. It is not a judgement of mine; B11 and B12 frequently close a finding
with a `**Verdict:**` line and no severity.

---

## 1. Master index — F-001 … F-501

Sorted by finding number. `ArMDE` is the entry's `source.lines` range as the
batch's verdict table gives it; `—` where the finding is not about one entry.

### B01 — `batch-01.md` (ArMDE:3362-3562)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-01 | `virtue.academic_concentration_subject` | 3362-3367 | effect+engine | M | live | the -1 on the six non-concentrated subjects is implemented nowhere and stated in neither locale; `AbilityRollMod` cannot express the negated scope |
| F-02 | `virtue.academic_concentration_subject` | 3362-3367 | incompat | M | live | stated incompatibility with Puissant Artes Liberales encoded nowhere |
| F-03 | `virtue.academic_concentration_subject` | 3362-3367 | param | H | live | `subject` is unbounded free text where the passage states a closed set and an exclusion |
| F-04 | `virtue.adept_laboratory_student` | 3368-3371 | scope | H | live | a conditional +6 encoded as an unconditional `lab_total_mod` (contradicts D1's first ruling; input to D4) |
| F-05 | `virtue.affinity_ability` | 3372-3374 | text | M | live | both locales say "exempt from the age-based cap"; the book says "exceed … by two points" |
| F-06 | `virtue.alim` | 3380-3383 | class+auth | H | live | `narrative` on an Academic-Ability authorization, and the authorization is missing |
| F-07 | `virtue.alluring_to_beings` | 3388-3395 | param | H | live | `being` free text where the passage names exactly three classes |
| F-08 | `virtue.alluring_to_beings` | 3388-3395 | prereq | H | live | the Gifted / Magical Air prohibition is encoded nowhere *(re-found as F-475)* |
| F-09 | `virtue.almogaten` | 3396-3403 | class+auth | H | live | `narrative`, Martial-Ability authorization missing |
| F-10 | `virtue.almogavar` | 3404-3409 | class+auth | H | live | `narrative`, Martial-Ability authorization missing |
| F-11 | `virtue.almogavar` | 3404-3409 | incompat | H | live | "may not take the Poor Flaw or Wealthy Virtue" absent from `incompatible_with` *(F-482 re-found this site)* |
| F-12 | `virtue.amorphous_major` / `_minor` | 3410-3413 | class+desc | M | live | `narrative` on a passage whose point is the Major/Minor difference; the two rows are textually identical |
| F-13 | `virtue.arcane_lore` | 3430-3435 | desc+incompat | H | live | the Parma Magica exclusion and the Enemy-of-the-Order consequence are implemented nowhere and in neither locale |
| F-14 | `virtue.archieunuch` | 3436-3439 | class+auth | H | live | `narrative`, Academic-Ability authorization missing |
| F-15 | `virtue.aristotelian_training` | 3440-3443 | scope+effect | H | live | the Lab bonus is conditional in the book, unconditional in the data; two further bonuses unimplemented |
| F-16 | `virtue.rard` | 3476-3479 | text+prov | M | live | the English `name` is the scanno "Rard"; the book's index and the table both read "Bard" — the **id** is wrong too |
| F-17 | `virtue.beadle` | 3480-3483 | class+auth | H | live | `narrative`, Academic-Ability authorization missing |
| F-18 | `virtue.berserk` | 3500-3503 | auth | H | live | "You may learn Martial Abilities at character creation" unimplemented |
| F-19 | `virtue.berserk` | 3500-3503 | effect+desc | M | live | the automatic Personality Trait Angry +2 is unimplemented and untexted |
| F-20 | `virtue.berserk` | 3500-3503 | scope | H | live | combat and Soak modifiers conditional in the book, unconditional in the data |
| F-21 | `virtue.blood_of_the_nephilim` | 3504-3518 | effect | H | live | the -5 Aging Roll modifier is directly expressible and missing |
| F-22 | `virtue.blood_of_the_nephilim` | 3504-3518 | auth | H | live | the Dominion Lore permission is missing and Dominion Lore is a gated category *(re-found as F-476)* |
| F-23 | `virtue.blood_of_the_nephilim` | 3504-3518 | incompat | H | live | the whole prohibition list at ArMDE:3517 is encoded nowhere *(re-found as F-476)* |
| F-24 | `virtue.blood_of_the_nephilim` | 3504-3518 | prereq | H | live | "Magi and Grogs may not take this Virtue" unenforced |
| F-25 | `virtue.blood_of_the_nephilim` | 3504-3518 | desc | M | live | four further stated rules in neither the data nor either locale |
| F-26 | `virtue.brother_chaplain` | 3529-3532 | class+auth | H | live | `narrative`, Academic-Ability authorization missing |
| F-27 | `virtue.brother_knight` | 3533-3536 | class+auth | H | live | `narrative`, Academic **and** Martial authorization missing |
| F-28 | `virtue.brother_sergeant` | 3537-3540 | class+auth | H | live | `narrative`, Martial-Ability authorization missing |
| F-29 | `virtue.bureaucrat` | 3541-3544 | class+auth | H | live | `narrative`, Academic-Ability authorization missing |
| F-30 | `virtue.cathedral_school_master` | 3549-3554 | prereq | H | live | the stated minimum scores are not encoded as prerequisites |
| F-31 | `virtue.baccalaureus` | 3470-3475 | scope | H | live | the restricted pool funds any Dead Language, where the book names Latin |
| F-32 | `virtue.apprentice` | 3418-3421 | class | M | live | `narrative` on a passage stating a requirement the engine enforces (closes Q-02 via D3) |
| F-33 | `virtue.bee_king` | 3484-3499 | desc | M | live | five stated rules reach the user through no surface at all |

### B02 — `batch-02.md` (ArMDE:3563-3766)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-34 | — (4 `Educated` variants) | 3711-3730 | prov | M | live | four Virtues in this span exist in the rulebook and its index and are absent from the catalogue |
| F-35 | `virtue.clan_ilfetu` | 3563-3566 | prereq | H | live | "within House Bjornaer" not encoded, though the catalogue's own idiom is one line away |
| F-36 | `virtue.clan_ilfetu` | 3563-3566 | scope | H | live | pool funds any Dead Language and any Organization Lore; the book names Gothic and House Bjornaer Lore |
| F-37 | `virtue.clan_ilfetu` | 3563-3566 | desc | M | live | three of the passage's four rules reach the user nowhere |
| F-38 | `virtue.clerk` | 3571-3574 | class+auth | H | live | `narrative` on an Academic-Ability permission, and the permission is missing |
| F-39 | `virtue.commanding_aura` | 3579-3596 | effect | H | live | the Soak bonus is directly expressible and absent |
| F-40 | `virtue.commanding_aura` | 3579-3596 | param | M | live | the book states a four-way choice of rank; no parameter, so the whole mechanical payload is unrepresentable |
| F-41 | `virtue.covenfolk`, `virtue.custos` | 3609-3612, 3629-3634 | incompat | H | live | "may not take the Wealthy Virtue or Poor Flaw" encoded on neither, though both targets exist *(two of F-340's eight sites)* |
| F-42 | `virtue.custos` | 3629-3634 | param+engine | H | live | the three-way restricted-Ability-group choice is the entry's whole mechanical content and is modelled nowhere |
| F-43 | `virtue.craft_guild_training` | 3613-3616 | scope | H | live | the pool funds any Organization Lore where the book names the guild's |
| F-45 | `virtue.cyclic_magic_positive` | 3635-3638 | scope | H | live | both modifiers conditional in the book, unconditional in the data; the Lab one carries a second gate (directed read; input to D4) |
| F-46 | `virtue.cyclic_magic_positive` | 3635-3638 | text | M | live | the German summary is truncated mid-sentence at `(z.` *(re-found as F-472)* |
| F-47 | `virtue.diedne_magic` | 3675-3682 | desc | M | live | the compulsory Major Story Flaw and its unusual budget treatment reach the user nowhere |
| F-48 | `virtue.doctor_in_faculty` | 3683-3698 | src | L | live | the `source` range swallows an unrelated sidebar |
| F-49 | `virtue.demonic_blood` | 3649-3662 | effect | H | live | the aging immunity the passage states verbatim is the engine's own `no_aging`/`no_apparent_aging`, and neither is carried |
| F-50 | `virtue.demonic_blood` | 3649-3662 | auth | H | live | "may learn Infernal Lore … without needing Arcane Lore" is the exact rule the app computes the opposite of *(re-found as F-477)* |
| F-51 | `virtue.demonic_blood` | 3649-3662 | incompat | H | live | "you cannot take the Unaging Virtue or the Age Quickly Flaw" encoded nowhere; both ids exist |
| F-52 | `virtue.demonic_blood` | 3649-3662 | desc | M | live | a 14-line passage stating at least nine further rules ships with a one-line summary and no description |
| F-53 | `virtue.demonic_might` | 3663-3666 | desc | M | live | the vis-on-death formula is in neither locale |
| F-54 | Demonic family (3 entries) | 3649-3670 | text | M | live | the German text uses "infernalisch", which is neither the table's word nor the German rulebook's |
| F-55 | `virtue.devil_child` | 3671-3674 | effect+text | M | live | the free Minor Virtue is in neither locale; both locales instead state an incompatibility the passage does not |
| F-56 | `virtue.doctor_in_faculty` | 3683-3698 | prereq | H | live | three stated minimum scores are not encoded as prerequisites |
| F-57 | `virtue.doctor_in_faculty` | 3683-3698 | param | H | live | the `faculty` parameter is unbounded free text where the book names a closed set |
| F-58 | `virtue.doctor_in_faculty` | 3683-3698 | desc | M | live | four further stated rules in neither locale |
| F-59 | `virtue.educated` | 3711-3713 | auth | H | live | "You may purchase Academic Abilities" is the first mechanical sentence, both summaries promise it, the data does not deliver |
| F-60 | `virtue.educated` | 3711-3713 | scope | H | live | the 50-point pool funds any Dead Language, not Latin |
| F-61 | `virtue.elemental_magic` | 3731-3738 | desc | M | live | the requisite-Form rule is in neither locale |
| F-62 | `virtue.emir` | 3743-3746 | desc+effect | H | live | a rule stated by cross-reference to another Virtue, and the cross-reference is not followed |
| F-63 | `virtue.enchanting_ability` | 3747-3750 | param+engine | H | live | the entry declares a choice the book demands and nothing anywhere can act on it |
| F-64 | `virtue.enduring_constitution` | 3751-3754 | desc+effect | M | live | the +3 pain-resistance bonus is in neither the data nor either locale |
| F-65 | `virtue.curse_throwing`, `virtue.dowsing` | 3625-3628, 3703-3706 | text | M | live | German summaries render *Ability* as "Fähigkeit"; canonical German is "Fertigkeit" |

### B03 — `batch-03.md` (ArMDE:3767-3966)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-66 | `virtue.eunuch` | 3771-3774 | class+auth | H | live | `narrative` on an Academic-Ability permission, and the permission is missing |
| F-67 | `virtue.failed_apprentice` | 3843-3846 | auth | H | live | three gated categories granted in one sentence, none authorized |
| F-68 | `virtue.fidai` | 3877-3882 | auth | H | live | "Fida'i may take Martial Abilities at character creation" not encoded |
| F-69 | `virtue.faerie_blood` | 3797-3820 | auth | H | live | "can learn Faerie Lore at character generation" is the exact rule the app computes the opposite of |
| F-70 | `virtue.faerie_blood` | 3797-3820 | param+desc | M | live | seven blood types, five with numeric bonuses; no parameter, no effects, no text |
| F-71 | `virtue.familiarity_with_the_fae` | 3857-3860 | auth+class | H | live | the Faerie Lore permission is computable and absent, and the entry is `uncomputed_rule` because of it *(re-found as F-407)* |
| F-72 | `virtue.folk_magic` | 3907-3920 | auth | H | live | the (Realm) Lore the Virtue is built on is Arcane and the entry does not authorize it |
| F-73 | `virtue.falconer` | 3847-3852 | scope | H | live | the 50-point pool funds any Dead Language and any Profession; the book names Latin and Profession: Falconer |
| F-74 | `virtue.forge_companion` | 3925-3928 | scope | H | live | the 50-point pool funds any Craft; the book names the master's Crafts |
| F-75 | `virtue.fast_caster` | 3865-3868 | scope | H | live | +3 Initiative unconditional in the data, scoped to spellcasting in the book, so it lands on weapon Initiative |
| F-76 | `virtue.fast_caster` | 3865-3868 | desc+effect | M | live | the second +3 is in neither the data nor either locale |
| F-77 | `virtue.faerie_raised_magic` | 3829-3842 | effect | M | live | "also includes the Virtue Spell Improvisation" and the included Virtue's effect is not carried |
| F-78 | `virtue.faerie_raised_magic` | 3829-3842 | desc | M | live | a 14-line complete spell-invention costing system ships with a one-sentence summary and no description |
| F-79 | `virtue.forgettable_face` | 3929-3932 | incompat | H | live | two prohibitions stated by name, encoded on neither side, both targets exist |
| F-80 | `virtue.frightful_presence` | 3941-3950 | rep | H | live | the granted Reputation is encoded nowhere, so a character who enters it is refused |
| F-81 | `virtue.gentle_gift` | 3955-3958 | class | M | live | `narrative` on the Virtue whose entire content is the removal of a stated numeric penalty |
| F-82 | `virtue.ghostly_warder` | 3963-3966 | class | M | live | `narrative` on an XP total, an Ability freedom and a once-per-day limit |
| F-83 | `virtue.flawless_magic` | 3887-3890 | desc | M | live | the per-spell special-ability rule is in neither locale |
| F-84 | `virtue.entrancement`, `virtue.font_of_knowledge` | 3767-3770, 3921-3924 | text | M | live | the Ability each confers, and its score, appear in neither locale's displayed text |
| F-85 | `virtue.fabric_ripper` | 3789-3792 | text | M | **blocked** (Q-10) | the German name contradicts the canonical translation table |
| F-86 | `virtue.finding_hidden_loot` | 3883-3886 | desc | L | live | the only `uncomputed_rule` in the batch with no `description` in either locale |
| F-87 | `virtue.faerie_magic` | 3825-3827 | desc | M | live | the "(see page 236)" cross-reference carries three rules the entry drops, attributed to the Virtue by name |
| F-88 | `virtue.eye_of_hephaestus` | 3783-3788 | text | M | live | the German description names an Ability that does not exist in the German app |
| F-89 | `virtue.ferocity` | 3873-3876 | scope+number | H | live | the unmodelled "animals only" makes the Confidence grant double-count |

### B04 — `batch-04.md` (ArMDE:3967-4154)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-90 | `virtue.the_gift` | 3967-3970 | class+desc | M | live | `narrative` on a stub whose pointer leads to five rules, none of which reaches the entry's text |
| F-91 | `virtue.good_teacher` | 3971-3974 | scope | H | live | the `book` modifier points at the wrong person |
| F-92 | `virtue.gorgiastic` | 3979-3982 | class | M | live | `narrative` on a hard score cap |
| F-93 | `virtue.gossip` | 3983-3986 | class | M | live | `narrative` on a doubling and a target number |
| F-94 | `virtue.greater_immunity` | 4009-4016 | param | H | live | the book demands a choice, nothing is recorded, and the same choice may be taken 255 times |
| F-95 | `virtue.greater_immunity` | 4009-4016 | class | M | live | `narrative` on a complete immunity |
| F-96 | `virtue.greater_power` | 4017-4026 | desc | M | live | one `power_levels` row, and four rules stated around it reach nobody (D5) |
| F-97 | `virtue.greater_purifying_touch` | 4027-4030 | class | M | live | `narrative` on a Fatigue cost and a one-disease limit |
| F-98 | `virtue.greater_purifying_touch` | 4027-4030 | param | H | live | "You must choose the disease … when you take this Virtue" and nothing records it |
| F-99 | `virtue.guardian_angel` | 4031-4036 | class | M | live | `narrative` on "+5 bonus to Soak" and "a Magic Resistance of 15" |
| F-100 | `virtue.guild_apprentice` | 4041-4044 | class+incompat | M | live | `narrative` on a clause that switches Wealthy and Poor off *(F-482 re-found this site)* |
| F-101 | `virtue.guild_dean` | 4045-4048 | auth | H | live | "You may select Academic Abilities at character generation" not encoded |
| F-102 | `virtue.guild_master` | 4049-4052 | auth | H | live | the identical sentence, the identical omission |
| F-103 | `virtue.harnessed_magic` | 4053-4058 | class | M | live | `narrative` on a spell-cancellation rule with a roll, a range and a death clause |
| F-104 | `virtue.heartbeast` | 4059-4061 | desc | M | live | the "(see page 233)" pointer carries a prohibition and the entry drops it |
| F-105 | `virtue.hermetic_experience` | 4063-4066 | scope | H | live | the restricted pool funds any Dead Language and any Organization Lore; the book names two instances |
| F-106 | `virtue.hermetic_magus` | 4067-4070 | class+prereq | H | live | `narrative` on two rules, one of which the prereq encodes wrongly |
| F-107 | `virtue.homing_instinct` | 4079-4084 | class | M | live | `narrative` on a formula, an Ease Factor and a fixed Arcane Connection |
| F-108 | `virtue.homing_instinct` | 4079-4084 | text | M | live | the German name contradicts the canonical translation table |
| F-109 | `virtue.imbued_with_the_spirit_of_form` | 4085-4094 | class | M | live | `narrative` on a one-for-one vis substitution |
| F-110 | `virtue.immune_to_disease` | 4095-4098 | class | M | live | `narrative` on an immunity |
| F-111 | `virtue.immunity_to_cold` | 4099-4102 | class | M | live | `narrative` on an immunity, with the exception stated as sharply as the rule |
| F-112 | `virtue.indescribable_face` | 4107-4114 | param | H | live | the book requires the player to choose a form and nothing records it |
| F-113 | `virtue.independent_study` | 4115-4118 | text | M | live | the German name contradicts the canonical translation table |
| F-114 | `virtue.infernal_heirloom` | 4127-4132 | class | M | live | `narrative` on a level number and a use frequency |
| F-115 | `virtue.inoffensive_to_beings` | 4133-4142 | class | M | live | `narrative` on an exemption from The Gift's penalties |
| F-116 | `virtue.inoffensive_to_beings` | 4133-4142 | mag | L | live | `hermetic` is a descriptor membership category, filed as `index_categories` |
| F-117 | `virtue.inspirational` | 4143-4146 | class | M | live | `narrative` on a +3 |
| F-118 | `virtue.intuition` | 4147-4150 | class | M | live | `narrative` on a named die and a target number |
| F-119 | `virtue.inventive_genius` | 4151-4154 | desc | M | live | the +6 experimentation alternative reaches neither locale |
| F-120 | `virtue.hermetic_prestige` | 4071-4073 | text | M | live | the German summary renders *Reputation* as `Ansehen`, which the tables forbid |
| F-121 | `virtue.greater_benediction` | 3991-4008 | class+desc | M | live | `narrative` over a cited range containing five numbers |

### B05 — `batch-05.md` (ArMDE:4155-4350)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-122 | `virtue.jurist` | 4163-4168 | class+auth | H | live | `narrative` on a Virtue authorizing three gated Academic Abilities, with no `ability_authorization` |
| F-123 | `virtue.jurist`, `virtue.knight` | 4163-4168, 4195-4198 | desc+engine | M | live | "only available to male characters" reaches neither locale; the engine has no sex model |
| F-124 | `virtue.just_an_instant` | 4169-4172 | class | M | live | `narrative` on a rule that waives a roll outright |
| F-125 | `virtue.kassalan_exorcism` | 4173-4186 | class+desc | M | live | `narrative` on an entry stating a complete casting subsystem |
| F-126 | `virtue.keen_vision` | 4187-4190 | class | M | live | `narrative` on a plain "+3" |
| F-127 | `virtue.keen_sense_of_smell` | 4191-4194 | class | M | live | `narrative` on a plain "+3" |
| F-128 | `virtue.knight` | 4195-4198 | auth | H | live | "You may take Martial Abilities during character generation" with no `ability_authorization` |
| F-129 | `virtue.land_regio_network` | 4211-4218 | class | M | live | `narrative` on an Ease Factor and a fixed travel Duration |
| F-130 | `virtue.land_regio_network` | 4211-4218 | text | M | live | the German name drops a hyphen and contradicts the canonical table |
| F-131 | `virtue.landed_noble` | 4219-4228 | class+desc | M | live | `narrative` on two season obligations and two explicit Poor/Wealthy interactions |
| F-132 | `virtue.lasiq` | 4233-4236 | auth | H | live | "may take Martial Abilities at character creation" with no `ability_authorization` |
| F-133 | `virtue.learn_ability_from_mistakes` | 4241-4244 | class | M | live | `narrative` on "you gain five experience points" |
| F-134 | `virtue.leather_ripper` | 4245-4248 | class | M | live | `narrative` on a spell-level equivalent, a Penetration value, a penalty range and a Fatigue cost |
| F-135 | `virtue.leather_ripper` | 4245-4248 | text | M | live | the German name contradicts the canonical table |
| F-136 | `virtue.leper_magus` | 4249-4252 | prereq | H | live | both stated prerequisites are missing and both are expressible |
| F-137 | `virtue.leper_magus` | 4249-4252 | desc | M | live | the wound→vis table and the extra Aging roll reach neither locale (D5) |
| F-138 | `virtue.lesser_benediction` | 4253-4274 | class+desc | M | live | `narrative` over a cited range containing a "-3" and a yield rule |
| F-139 | `virtue.lesser_benediction` | 4253-4274 | param | H | live | no parameter records which benediction was taken |
| F-140 | `virtue.lesser_immunity` | 4275-4278 | class+desc | M | live | `narrative` on an absolute immunity; the cross-reference carries two further rules |
| F-141 | `virtue.lesser_immunity` | 4275-4278 | param | H | live | no parameter records which hazard |
| F-142 | `virtue.lesser_power` | 4279-4286 | desc | M | live | the Initiative formula, the Fatigue costs and the Penetration purchase reach neither locale (D5) |
| F-143 | `virtue.lesser_purifying_touch` | 4287-4290 | class | M | live | `narrative` on a Fatigue cost and a single-illness limit |
| F-144 | `virtue.lesser_purifying_touch` | 4287-4290 | param | H | live | no parameter records the chosen illness |
| F-145 | `virtue.license_of_absence` | 4291-4294 | prereq | H | live | both stated prerequisites are missing and both are expressible |
| F-146 | `virtue.license_of_absence` | 4291-4294 | class+desc | M | live | `narrative` on an extra free season and a hard cap of four |
| F-147 | `virtue.life_boost` | 4295-4298 | desc | M | live | the +5-per-Fatigue-level rule and its damage formula reach neither locale (D5) |
| F-148 | `virtue.life_linked_spontaneous_magic` | 4299-4306 | desc | M | live | the per-five-points Fatigue rule and the wound conversion reach neither locale (D5) |
| F-149 | `virtue.lightning_reflexes` | 4311-4314 | scope | H | live | a triply-conditional +9 folded flat into every Initiative Total |
| F-150 | `virtue.linguist` | 4315-4318 | desc | M | live | the in-play "All Advancement Totals" half reaches neither locale (D5) |
| F-151 | `virtue.lone_redcap` | 4319-4326 | desc+rep | M | live | the season obligations, two denials and the Reputation's polarity reach neither locale (D5) |
| F-152 | `virtue.lone_redcap` | 4319-4326 | text | M | live | the German name contradicts two canonical tables |
| F-153 | `virtue.long_winded` | 4327-4330 | desc | M | live | "This bonus does not apply to casting spells" reaches neither locale (D5) |
| F-154 | `virtue.luck` | 4331-4334 | class | M | live | `narrative` on "+1 to +3" |
| F-155 | `virtue.lupus_the_wolf` | 4335-4338 | auth | H | live | permission for two gated Academic Abilities with no `ability_authorization` |
| F-156 | `virtue.magian_lineage_minor` | 4339-4346 | desc | M | live | the "+3 bonus to resist the effects of disease" reaches neither locale (D5) |
| F-157 | `virtue.magian_lineage_major` | 4339-4346 | param | H | live | the three chosen Abilities have no parameter |
| F-158 | `virtue.magian_lineage_major` | 4339-4346 | desc | M | live | the XP-sharing rule and the disease bonus reach neither locale (D5) |
| F-159 | `virtue.magic_items` | 4347-4350 | desc | M | live | the improvement rate and the Level 30 per-effect cap reach neither locale (D5) |
| F-160 | `virtue.knows_people` | 4199-4206 | class+desc | M | live | `narrative` on a once-per-story player entitlement (closes Q-35) |

### B06 — `batch-06.md` (ArMDE:4351-4597)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-161 | `virtue.magic_sensitivity` | 4351-4354 | desc | M | live | the Magic-Resistance subtraction reaches neither locale (D5) |
| F-162 | `virtue.magical_memory` | 4355-4358 | class | M | live | `narrative` on a rule that waives a physical prerequisite |
| F-163 | `virtue.magical_blood` | 4359-4372 | auth | H | live | permission for a gated Arcane Ability with no `ability_authorization` |
| F-164 | `virtue.magical_blood` | 4359-4372 | desc | M | live | all four bloodline types' mechanics reach neither locale (D5) |
| F-165 | `virtue.magical_blood` | 4359-4372 | param | H | live | no parameter records which of the four types was taken |
| F-166 | `virtue.magical_mount` | 4373-4376 | class | M | live | `narrative` on a Might cap, a granted trait and a mandatory Major Story Flaw |
| F-167 | `virtue.magical_warder` | 4377-4384 | class | M | live | `narrative` on a detection rule and a stated range limit |
| F-168 | `virtue.magical_warder` | 4377-4384 | text | M | live | the German name contradicts the canonical translation table |
| F-169 | `virtue.magister_in_artibus` | 4385-4394 | prereq | H | live | both stated Ability prerequisites are missing and both are expressible |
| F-170 | `virtue.magister_in_artibus` | 4385-4394 | desc | M | live | "only available to male characters" reaches neither locale |
| F-171 | `virtue.magister_in_artibus` | 4385-4394 | desc | M | live | the age formula, the teaching obligation and two status interactions reach neither locale (D5) |
| F-172 | `virtue.magister_in_medicina` | 4395-4398 | prereq | H | live | the imported prerequisites are missing and three are expressible |
| F-173 | `virtue.magister_in_medicina` | 4395-4398 | desc | M | live | the imported Doctor-in-(Faculty) rules and the Salerno restriction reach neither locale (D5) |
| F-174 | `virtue.major_magical_focus` | 4399-4422 | desc | M | live | the requisite rule, the lab-activity restriction and the breadth rule reach neither locale (D5) |
| F-175 | `virtue.maker_of_textured_vessels` | 4423-4430 | class | M | live | `narrative` on a +3, a Fatigue cost and a shape formula |
| F-176 | `virtue.maker_of_water_vessels` | 4431-4438 | class | M | live | `narrative` on an Ability swap, a stated duration and a Warping rule |
| F-177 | `virtue.male_guild_sponsor` | 4439-4442 | class+desc | M | live | `narrative` on a *mandatory second* Social Status Virtue and a female-only restriction |
| F-178 | `virtue.mamluk` | 4443-4448 | auth | H | live | Martial permission and the Theology: Islam exception, with no `ability_authorization` |
| F-179 | `virtue.mamluk` | 4443-4448 | desc | M | live | "only available to male characters" reaches neither locale |
| F-180 | `virtue.marshal` | 4449-4456 | auth | H | live | "may take Martial Abilities freely" with no `ability_authorization` |
| F-181 | `virtue.marshal` | 4449-4456 | desc | M | live | the Profession-functions-as-Medicine rule reaches neither locale (D5) |
| F-182 | `virtue.master_bard` | 4457-4462 | auth | H | live | "Arcane Abilities" is authorized as two named ids |
| F-183 | `virtue.master_bard` | 4457-4462 | prereq | H | live | the stated Ability prerequisites are missing; the four-way one is fully expressible |
| F-184 | `virtue.master_bard` | 4457-4462 | desc | M | live | the two-season obligation and the Ireland-only restriction reach neither locale (D5) |
| F-185 | `virtue.master_of_form_creatures` | 4463-4466 | auth | H | live | permission for a gated Arcane Ability with no `ability_authorization` |
| F-186 | `virtue.master_of_kennels` | 4467-4470 | auth | H | live | "may take Martial Abilities freely" with no `ability_authorization` |
| F-187 | `virtue.masterpiece` | 4476-4479 | desc | M | live | the lesser-item restriction and the vis waiver reach neither locale (D5) |
| F-188 | `virtue.mazdean_priest` | 4480-4487 | auth | H | live | Academic permission with no `ability_authorization` |
| F-189 | `virtue.mazdean_priest` | 4480-4487 | desc | M | live | "only available to male characters" reaches neither locale |
| F-190 | `virtue.mendicant_friar` | 4488-4495 | auth | H | live | Academic permission with no `ability_authorization` |
| F-191 | `virtue.mendicant_friar` | 4488-4495 | incompat | H | live | "You may not take the Wealthy Virtue or Poor Flaw" is expressible and absent *(one of F-340's eight sites)* |
| F-192 | `virtue.mendicant_friar` | 4488-4495 | desc | M | live | "only available to male characters" reaches neither locale |
| F-193 | `virtue.mendicant_friar` | 4488-4495 | text | M | live | the **English** summary is truncated mid-abbreviation at "St." *(re-found as F-479)* |
| F-194 | `virtue.mentored_by_demons` | 4496-4499 | engine+prereq | H | live | the age-cap waiver is modelled nowhere and the app errors on the character the passage describes |
| F-195 | `virtue.mercenary_captain` | 4500-4505 | auth | H | live | Martial permission with no `ability_authorization` |
| F-196 | `virtue.mercurian_magic` | 4514-4523 | effect | H | live | the required companion Flaw is encoded nowhere |
| F-197 | `virtue.mercurian_magic` | 4514-4523 | desc | M | live | Wizard's Vigil, the Mastery addition and the half-vis rule reach neither locale (D5) |
| F-198 | `virtue.method_caster` | 4524-4527 | scope+desc | M | live | the "if you vary at all" gate is folded in flat and stated nowhere (D5) |
| F-199 | `virtue.minor_enchantments` | 4532-4535 | class+effect | H | live | a 25-level item allowance the engine can already express, authored as `narrative` with no effect |
| F-200 | `virtue.minor_magical_focus` | 4536-4538 | src | L | live | `source.lines` stops three lines before the rule it cites |
| F-201 | `virtue.minor_magical_focus` | 4536-4538 | desc | M | live | the inherited requisite rule and the lab-activity restriction reach neither locale (D5) |
| F-202 | `virtue.muqta_muq_ta` | 4559-4562 | class | M | live | `narrative` on "All rules for the Landed Noble Virtue apply" |
| F-203 | `virtue.muse` | 4563-4566 | class | M | live | `narrative` on granting and doubling another character's Virtue |
| F-204 | `virtue.mystical_choreography` | 4567-4572 | class | M | live | `narrative` on two numeric time reductions |
| F-205 | `virtue.mythic_blood` | 4573-4589 | effect | M | live | the free hereditary Minor Personality Flaw is encoded nowhere |
| F-206 | `virtue.mythic_blood` | 4573-4589 | desc | M | live | the two Fatigue waivers and the invocation table reach neither locale (D5) |
| F-207 | `virtue.natural_leader` | 4590-4593 | class | M | live | `narrative` on a plain "+3" |
| F-208 | `virtue.nephilim` | 4594-4597 | effect | H | live | the free Strong Angelic Heritage grant is missing; two sibling entries carry theirs |

### B07 — `batch-07.md` (ArMDE:4598-4883)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-209 | `virtue.notary` | 4598-4601 | class+auth | H | live | `narrative` on an entry granting a gated Ability category, no `ability_authorization` |
| F-210 | `virtue.notary` | 4598-4601 | desc | M | live | the clergy and secular-law restriction reaches neither locale (D5) |
| F-211 | `virtue.paid_rights` | 4606-4615 | class | M | live | `narrative` on the entry that overrides the one-Social-Status rule |
| F-212 | `virtue.partner` | 4616-4619 | class+desc | M | live | the Virtue-waiver rule reaches neither locale, and the entry is `narrative` |
| F-213 | `virtue.perfect_balance` | 4624-4627 | class | M | live | "+6" classified `narrative` |
| F-214 | `virtue.perfect_eye_for_commodity` | 4628-4631 | class | M | live | an absolute and a formula classified `narrative` |
| F-215 | `virtue.perfectus`, `virtue.performance_magic` | 4632-4709 | src | L | live | one `source.lines` range swallows the other's worked example |
| F-216 | `virtue.perfectus` | 4632-4641 | class+auth | H | live | `narrative` on an entry granting a gated Ability category, no `ability_authorization` |
| F-217 | `virtue.perfectus` | 4632-4641 | incompat | H | live | "You may not take the Wealthy Virtue" absent from `incompatible_with` *(F-340 family)* |
| F-218 | `virtue.perfectus` | 4632-4641 | desc | M | live | the Purity/Transcendence permission reaches neither locale (D5) |
| F-219 | `virtue.performance_magic` | 4642-4709 | class+desc | M | live | a 68-line rules subsystem classified `narrative` |
| F-220 | `virtue.performance_magic` | 4642-4709 | param | H | live | the Ability it names cannot be recorded, and it cannot be taken twice |
| F-221 | `virtue.personal_power` | 4714-4727 | desc | M | live | four stated clauses reach neither locale (D5) |
| F-222 | `virtue.personal_vis_source` | 4728-4731 | class | M | live | the yield rule classified `narrative` |
| F-223 | `virtue.physician_of_salerno` | 4732-4735 | prereq | H | live | a stated prerequisite is encoded nowhere |
| F-224 | `virtue.piercing_gaze` | 4736-4739 | class | M | live | "+3" and an absolute exemption classified `narrative` |
| F-225 | `virtue.potent_magic_major` / `_minor` | 4740-4781 | scope | H | live | the Casting-Score bonus is folded into *every* Casting Total though the passage scopes it to the field |
| F-226 | `virtue.potent_magic_major` / `_minor` | 4740-4781 | incompat | H | live | `incompatible_with` contradicts ArMDE:4742 and a load-time guard forces it |
| F-227 | `virtue.potent_magic_major` / `_minor` | 4740-4781 | param | H | live | the field cannot be recorded and a second area cannot be taken |
| F-228 | `virtue.potent_magic_major` / `_minor` | 4740-4781 | desc | M | live | the Potency subsystem and three further rules reach neither locale (D5) |
| F-229 | `virtue.powerful_relic` | 4782-4787 | desc | M | live | the relic's power, the impiety rule and the built-into-an-item clause reach neither locale (D5) |
| F-230 | `virtue.prestigious_student` | 4792-4795 | class+auth | H | live | `narrative` on an entry granting a gated Ability category, no `ability_authorization` |
| F-231 | `virtue.prestigious_student` | 4792-4795 | desc | M | live | the Social-Status constraint reaches neither locale (D5) |
| F-232 | `virtue.priest` | 4796-4805 | class+auth | H | live | `narrative` on an entry granting a gated Ability category, no `ability_authorization` |
| F-233 | `virtue.priest` | 4796-4805 | desc+incompat | M | live | a conditional Poor exclusion, a male-only restriction and a Social-Status compatibility reach neither locale (D5) *(F-340 family — the conditional site at :4800)* |
| F-234 | `virtue.privileged_upbringing` | 4806-4808 | scope | H | live | the engine grants exactly the permission the passage forbids |
| F-235 | `virtue.protection` | 4810-4813 | rep | H | live | a granted Reputation the engine can express, carried by no effect |
| F-236 | `virtue.protection` | 4810-4813 | desc | M | live | the "could be higher" clause and the good/bad choice reach neither locale (D5) |
| F-237 | `virtue.puissant_ability` | 4814-4816 | desc | M | live | the learning/writing/helping exclusion reaches neither locale (D5) |
| F-238 | `virtue.puissant_art` | 4818-4820 | desc | M | live | the requisite rule and the learning/teaching/writing exclusion reach neither locale (D5) |
| F-239 | `virtue.quiet_magic` | 4822-4827 | desc | M | live | the booming-voice and Voice-Range clauses reach neither locale (D5) |
| F-240 | `virtue.rabbi` | 4828-4833 | class+desc | M | live | a mandatory companion Virtue and a male-only restriction, on a `narrative` entry |
| F-241 | `virtue.rat_up_a_drainpipe` | 4838-4841 | class | M | live | an absolute and a named-Ability advantage classified `narrative` |
| F-242 | `virtue.redcap` | 4842-4851 | incompat | H | live | three stated exclusions absent from `incompatible_with` |
| F-243 | `virtue.redcap` | 4842-4851 | effect | H | live | the free Well-Traveled Virtue is granted nowhere |
| F-244 | `virtue.redcap` | 4842-4851 | effect+number | H | live | 300 apprenticeship experience points are granted nowhere |
| F-245 | `virtue.redcap` | 4842-4851 | auth | H | live | three gated Ability categories granted, with no `ability_authorization` |
| F-246 | `virtue.redcap` | 4842-4851 | desc | M | live | five further stated clauses reach neither locale (D5) |
| F-247 | `virtue.religious` | 4856-4861 | class+auth | H | live | `narrative` on an entry granting a gated Ability category, no `ability_authorization` |
| F-248 | `virtue.reserves_of_strength` | 4862-4865 | class | M | live | "+3", a use limit and a cost, classified `narrative` |
| F-249 | `virtue.ripper` | 4866-4869 | effect | H | live | 70 levels of power carried by no `power_levels`, and the app errors on a legal character |
| F-250 | `virtue.ripper` | 4866-4869 | desc | M | live | the activation cost, range and inflexibility reach neither locale (D5) |
| F-251 | `virtue.ritual_power` | 4870-4877 | desc | M | live | four stated clauses reach neither locale (D5) |
| F-252 | `virtue.rosh_beth_din` | 4878-4883 | prereq | H | live | three stated Ability prerequisites and an age requirement encoded nowhere |
| F-253 | `virtue.rosh_beth_din` | 4878-4883 | auth | H | live | the Academic *category* is authorized as three ids |
| F-254 | `virtue.rosh_beth_din` | 4878-4883 | rep | H | live | the granted Reputation is typed `local` though the passage says it applies across his country |
| F-255 | `virtue.rosh_beth_din` | 4878-4883 | desc | M | live | male-only, the earmark scope and the yeshivah clause reach neither locale (D5) |
| F-256 | `virtue.relic`, `virtue.powerful_relic` | 4852-4855, 4782-4787 | number+scope | H | live | `true_faith_grant` puts the *relic's* score on the *character*, which ArMDE:17607 forbids |
| F-257 | `virtue.relic` | 4852-4855 | desc | M | live | the relic's actual mechanics reach neither locale (D5) |

### B08 — `batch-08.md` (ArMDE:4884-5096)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-258 | `virtue.schooled_in_crime` | 4884-4887 | desc | M | live | the troupe-extension clause reaches neither locale (D5) |
| F-259 | `virtue.secondary_insight` | 4892-4895 | desc | M | live | three of the four clauses reach neither locale (D5) |
| F-260 | `virtue.senior_bard` | 4904-4909 | scope+auth | H | live | the passage permits **any** Realm Lore; the data authorizes two of the four |
| F-261 | `virtue.senior_bard` | 4904-4909 | prereq | H | live | the minimum age of 22 is carried nowhere (D3) |
| F-262 | `virtue.senior_bard` | 4904-4909 | desc | M | live | three further clauses reach neither locale (D5) |
| F-263 | `virtue.senior_bard` | 4904-4909 | text | M | live | the German name contradicts the canonical translation table |
| F-264 | `virtue.senior_clergy` | 4910-4921 | auth | H | live | "You may purchase Academic Abilities" with no `ability_authorization` |
| F-265 | `virtue.senior_clergy` | 4910-4921 | desc | M | live | four clauses reach neither locale (D5) |
| F-266 | `virtue.senior_master` | 4922-4925 | class+auth | H | live | `narrative` on an entry granting a gated Ability category, no `ability_authorization` |
| F-267 | `virtue.shadchan` | 4934-4939 | desc+prereq | M | live | the Social-Status compatibility override is stated and encoded nowhere |
| F-268 | `virtue.shadchan` | 4934-4939 | desc | M | live | the pool's scoping reaches neither locale (D5) |
| F-269 | `virtue.shamash` | 4940-4945 | class | M | live | `narrative` on an entry stating a hard requirement |
| F-270 | `virtue.shamash` | 4940-4945 | prov | H | **blocked** | the required Virtue does not exist in the catalogue |
| F-271 | `virtue.sharp_ears` | 4950-4953 | class | M | live | `narrative` on a plain "+3" |
| F-272 | `virtue.side_effect` | 4954-4957 | class | M | live | `narrative` on an entry stating a scaling rule and two worked bonuses |
| F-273 | `virtue.simple_student` | 4958-4963 | auth | H | live | the academic permission is missing (the XP grant's absence is a recorded deferral, not the finding) |
| F-274 | `virtue.simple_student` | 4958-4963 | desc | M | live | the female/Salerno restriction and the Paid Rights override reach neither locale (D5) |
| F-275 | `virtue.skilled_smuggler` | 4968-4971 | class | M | live | `narrative` on an entry whose own summary prints the number |
| F-276 | `virtue.skilled_smuggler` | 4968-4971 | text | M | live | the German name reproduces a spelling error in the DE source |
| F-277 | `virtue.skinchanger` | 4972-4975 | class | M | live | `narrative` on an entry stating a Soak bonus, a Size band and a transformation time |
| F-278 | `virtue.skinchanger_dove` | 4976-4987 | class | M | live | `narrative` on an entry stating a Soak bonus and a preparation time |
| F-279 | `virtue.skinchanger_dove` | 4976-4987 | desc | M | live | "the Minor Virtue above" imports a whole rule set and neither locale says so |
| F-280 | `virtue.social_contacts` | 4988-4991 | class | M | live | `narrative` on an entry stating a roll and an Ease Factor |
| F-281 | `virtue.social_contacts` | 4988-4991 | param | M | live | no parameter records the social circle the passage requires |
| F-282 | `virtue.sofer` | 4992-4997 | class | M | live | `narrative` on an entry stating a hard requirement |
| F-283 | `virtue.sofer` | 4992-4997 | prov | H | **blocked** | the required Virtue does not exist in the catalogue |
| F-284 | `virtue.special_circumstances` | 4998-5001 | scope | H | live | a conditional casting bonus folded in flat (`scope: "all"`) |
| F-285 | `virtue.special_circumstances` | 4998-5001 | number | H | live | two copies give +6 where the book caps the total at +3 |
| F-286 | `virtue.special_circumstances` | 4998-5001 | prov | L | live | `magic_resistance_mod`'s `aura_bonus` is a different Virtue's semantic; `RULES.md:5138` names this entry wrongly |
| F-287 | `virtue.special_circumstances` | 4998-5001 | param | H | live | no parameter records which circumstances |
| F-288 | `virtue.spell_improvisation` | 5002-5005 | desc | M | live | two clauses reach neither locale (D5) |
| F-289 | `virtue.spiritual_pact` | 5010-5021 | class+desc | M | live | `narrative` on the most mechanically dense entry in the batch |
| F-290 | `virtue.strong_angelic_heritage` | 5022-5031 | desc | M | live | six formulas and restrictions reach neither locale (D5) |
| F-291 | `virtue.strong_faerie_blood` | 5032-5047 | incompat | H | live | "You may not have both Faerie Blood and Strong Faerie Blood" declared on neither side |
| F-292 | `virtue.strong_faerie_blood` | 5032-5047 | auth | H | live | "you may learn Faerie Lore" with no `ability_authorization` |
| F-293 | `virtue.strong_faerie_blood` | 5032-5047 | desc | M | live | the aging onset age of fifty is carried nowhere (D3) |
| F-294 | `virtue.strong_faerie_blood` | 5032-5047 | desc+param | M | live | three further clauses reach neither locale (D5); the heritage choice is unrecordable |
| F-295 | `virtue.strong_willed` | 5048-5051 | class | M | live | `narrative` on a plain "+3" |
| F-296 | `virtue.student_of_realm` | 5052-5055 | effect | H | live | the +2 bonus is carried by no effect at all |
| F-297 | `virtue.student_of_realm` | 5052-5055 | scope | H | live | the authorization permits all four Lores where the passage permits one |
| F-298 | `virtue.student_of_realm` | 5052-5055 | incompat | M | live | the Puissant Ability exclusion is encoded nowhere (D3) |
| F-299 | `virtue.student_of_realm` | 5052-5055 | text | — | **withdrawn** | retracted by B08 itself — the German *filled* name is a deliberate, documented decision |
| F-300 | `virtue.study_bonus` | 5056-5072 | desc | M | live | the condition and the whole eight-row table reach neither locale (D5) |
| F-301 | `virtue.subtle_magic` | 5073-5076 | desc | M | live | the gesture clause reaches neither locale (D5) |
| F-302 | `virtue.sufi` | 5077-5084 | class+auth | H | live | `narrative` on an entry granting three gated Abilities, with no `ability_authorization` |
| F-303 | `virtue.sufi` | 5077-5084 | desc | H | live | the no-points Story Flaw rule reaches neither locale (D5) |
| F-304 | `virtue.supernatural_beauty` | 5089-5096 | class | M | live | `narrative` on an entry stating a once-per-story resource |
| F-305 | `virtue.supernatural_beauty` | 5089-5096 | prereq | H | live | the positive-Presence precondition is encoded nowhere (D3) |
| F-306 | `virtue.self_confident` | 4900-4902 | number | M | live | gives a grog a Confidence Score the book denies grogs, and a number the book never prints |
| F-307 | `virtue.sense_holiness_and_unholiness` | 4926-4929 | param+prov | M | live | the realm is fixed to Divine by the book, and that fact is in none of the five places it could be |
| F-308 | `tugenden-fehler.md` | — | table | L | live | three defective rows, one of which this batch's Q-69 reasoning relies on (D6) |

### B09 — `batch-09.md` (ArMDE:5097-5256)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-309 | `virtue.tainted_treasure` | 5097-5108 | src | M | live | `source.lines` swallows the Knights Templar sidebar, and the rule inside it belongs to nobody |
| F-310 | `virtue.templar_administrator` | 5109-5112 | class+auth | H | live | a gated-category permission with no `ability_authorization`, and `narrative` denying it |
| F-311 | `virtue.templar_administrator` | 5109-5112 | desc | M | live | the Status-replacement rule, the male-only restriction and the no-extra-time clause reach neither locale (D5) |
| F-312 | `virtue.templar_commander` | 5113-5116 | rep | H | live | the Reputation of level 3 is carried by no effect |
| F-313 | `virtue.templar_commander` | 5113-5116 | desc | M | live | four further clauses reach neither locale (D5) |
| F-314 | `virtue.templar_confrere_or_consoeur` | 5117-5120 | class+desc | M | live | a Social-Status compatibility override the class denies and nothing encodes |
| F-315 | `virtue.templar_office_holder` | 5121-5124 | desc | M | live | the compatibility override is stated twice and encoded nowhere (D5) |
| F-316 | `virtue.templar_prestige` | 5125-5128 | class+rep | H | live | a Reputation of level 4 stated by the book and carried by no effect, under `narrative` |
| F-317 | `virtue.templar_specialist` | 5133-5136 | class+engine | M | live | a player-chosen gated Ability group the engine cannot express, under `narrative` |
| F-318 | `virtue.temporal_influence` | 5137-5140 | prereq | H | live | "Grogs may not take this Virtue" enforced nowhere, although the mechanism exists and is used |
| F-319 | `virtue.tethered_magic` | 5141-5144 | class+desc | M | live | a Hermetic spell mechanic classified `narrative`, reaching neither locale *(F-432 re-rates the Flaw twin knowingly)* |
| F-320 | `virtue.tough` | 5145-5147 | text | M | live | the German summary calls Soak "Widerstandsfähigkeit" against the canonical table and the DE rulebook |
| F-321 | `virtue.town_magistrate` | 5149-5152 | class | M | live | a Social Status with three mechanical clauses classified `narrative` |
| F-322 | `virtue.town_magistrate` | 5149-5152 | prereq | H | live | a prerequisite the `Prereq` tree expresses exactly, and that is absent |
| F-323 | `virtue.town_magistrate`, `virtue.university_grammar_teacher` | 5149-5152, 5195-5198 | auth | H | live | Academic permissions with no `ability_authorization` |
| F-324 | `virtue.trained_assassin` | 5153-5156 | prereq | H | live | the Nizari Social-Status prerequisite is absent, though `Prereq::Any` expresses it exactly |
| F-325 | `virtue.turb_trained` | 5179-5182 | class+auth+incompat | H | live | a Martial permission, a single dead language, and a Wealthy/Poor exclusion, all under `narrative` *(F-340 family)* |
| F-326 | `virtue.troubadour` | 5157-5164 | class+auth | H | live | an Academic permission with no `ability_authorization`, under `narrative` |
| F-327 | `virtue.troupe_upbringing` | 5165-5168 | class | M | live | a "+2 modifier" under a `narrative` class |
| F-328 | `virtue.troupe_upbringing` | 5165-5168 | param | M | live | no parameter records which area was selected |
| F-329 | `virtue.true_faith` | 5169-5172 | effect+engine | H | live | the book gives a Magic Resistance formula and the engine computes nothing from the score |
| F-330 | `virtue.true_faith` | 5169-5172 | desc | M | live | the granted Faith Point, the spend rule and the refresh rule reach neither locale (D5, D3) |
| F-331 | `virtue.unbound_tongue` | 5191-5194 | class | M | live | the waiver of a -10 casting penalty, classified `narrative` |
| F-332 | `virtue.unaging` | 5187-5190 | desc | M | live | the crisis clause and the Decrepitude-4 waiver reach neither locale (D5, D3) |
| F-333 | `virtue.true_love_pc`, `virtue.venus_blessing` | 5173-5178, 5211-5214 | class | M | live | two "+3" Virtues classified `narrative` |
| F-334 | `virtue.true_love_pc` | 5173-5178 | prereq+engine | M | live | the reciprocity requirement is encoded nowhere and cannot be (D3) |
| F-335 | `virtue.unaffected_by_the_gift` | 5183-5186 | class | M | live | immunity to a numeric social penalty, classified `narrative` |
| F-336 | `virtue.variable_power` | 5199-5206 | class+desc | M | live | a scaling formula and four variables, classified `narrative` *(re-found as F-413)* |
| F-337 | `virtue.verditius_magic` | 5215-5217 | class | L | live | `narrative` on an entry whose two rules the engine already computes |
| F-338 | `virtue.verditius_magic` | 5215-5217 | desc | M | live | the entire Outer Mystery behind "(see page 240)" reaches neither locale (D5) |
| F-339 | `virtue.wealthy` | 5235-5238 | prereq | H | live | the companion-only rule is enforced for magi and mythic companions and not for grogs |
| F-340 | `virtue.wealthy`, `flaw.poor` + 8-11 sites | 5235-5238 | incompat | H | live | **umbrella:** ten flat Wealthy/Poor prohibition passages (plus one conditional) and `incompatible_with` empty catalogue-wide — see § 4 |
| F-341 | `virtue.well_traveled` | 5239-5242 | effect+number | H | live | fifty experience points stated by the book, carried by no effect, under `narrative` |
| F-342 | `virtue.well_traveled` | 5239-5242 | effect | H | live | the target of two grants, one handing out an empty box and the other not declaring the grant |
| F-343 | `virtue.wisdom_from_ignorance` | 5251-5256 | class | M | live | a study rule and an Ability substitution, classified `narrative` |
| F-344 | `virtue.ways_of_the_land` | 5231-5234 | scope+effect | H | live | the +3 reaches only Casting Totals, the book gives it to *all* rolls; the botch-die reduction is carried by nothing |
| F-345 | `reputationen.md`, `tugenden-fehler.md` | — | table | L | live | two table rows contradict the DE rulebook; the data is right and the **tables** are wrong (D6) |
| F-346 | `virtue.university_grammar_teacher` | 5195-5198 | class | M | live | `narrative` on an entry with a permission and a seasonal obligation |
| F-347 | `virtue.venditor` | 5207-5210 | auth | H | live | the Academic permission is carried by nothing, although the XP pool beside it is exactly right |
| F-348 | `virtue.voice_of_the_land` | 5219-5222 | param | H | live | a free-text parameter turns a once-only Virtue into an unlimited one (ArMDE:2814) |

### B10 — `batch-10.md` (ArMDE:5257-5780)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-349 | `virtue.wise_one` | 5257-5260 | class+auth | H | live | a two-category gated Ability permission with no `ability_authorization`, and `narrative` denying it |
| F-350 | `virtue.wise_one` | 5257-5260 | desc | M | live | three clauses reach neither locale (D5) |
| F-351 | `virtue.withstand_casting` | 5261-5282 | number | M | live | the `casting_fatigue` sign is the opposite of the engine's documented convention and of both Flaw carriers' |
| F-352 | `virtue.withstand_casting` ↔ `flaw.vulnerable_casting` | 5261-5282 | incompat | H | live | a creation-time incompatibility stated twice and encoded on neither side |
| F-353 | `virtue.withstand_casting` | 5261-5282 | desc | M | live | a 22-line passage reaches the user as one sentence (D5) |
| F-354 | `flaw.abandoned_apprentice` | 5641-5650 | class | M | live | a character-creation procedure and a hard consequence, classified `narrative` |
| F-355 | `flaw.ability_block` | 5651-5654 | class+param+engine | M | live | a whole-category learning block under `narrative`, with no parameter *(re-found as F-426)* |
| F-356 | `flaw.afflicted_tongue` | 5655-5658 | scope | M | live | a stated condition folded in flat, on a channel that prints |
| F-357 | `flaw.age_quickly` | 5659-5662 | desc | M | live | a schedule rule that reaches neither locale (D5) |
| F-358 | `flaw.anchored_to_the_land`, `flaw.bound_to_realm`, `flaw.bound_to_role_role` | 5667-5748 | param | M | live | three parameterized Flaws have no `max_total`, so rules ArMDE:2814 permits once are unlimited |
| F-359 | `flaw.brutal_artist` | 5757-5760 | prereq | M | live | a House restriction the `Prereq` tree expresses exactly, and that is absent |
| F-360 | `flaw.beloved_rival_major` / `_minor` | 5691-5698 | incompat | H | live | the data forbids the exact combination ArMDE:5697 permits, and the load-time guard forces it to |
| F-361 | `flaw.baneful_circumstances` | 5687-5690 | desc | M | live | an aging rule that overrides aging immunity, reaching neither locale (D5) |
| F-362 | `flaw.baneful_circumstances` | 5687-5690 | param | M | live | no parameter records the circumstance |
| F-363 | `flaw.apostate`, `flaw.black_sheep` | 5675-5678, 5703-5705 | rep+engine | M | live | `grants_reputation` cannot say a Reputation is **bad**, and both carriers in this batch are bad ones |
| F-364 | `flaw.bigamist` | 5699-5702 | class | M | live | two numbers under a `narrative` class |
| F-365 | `flaw.blackmail` | 5707-5710 | class | L | live | a quantified yearly benefit under a `narrative` class |
| F-366 | `flaw.blatant_gift` ↔ `flaw.blatant_magical_air` | 5711-5718 | incompat | H | live | a stated mutual exclusion missing from both sides *(re-found as F-474)* |
| F-367 | `flaw.blatant_magical_air` | 5715-5718 | prereq | H | live | an eligibility gate the `Prereq` tree expresses exactly, and that is absent *(re-found as F-474)* |
| F-368 | `flaw.blatant_gift` | 5711-5714 | desc | H | live | the rules behind "page 203" reach neither locale (D5) |
| F-369 | `flaw.blind` | 5719-5722 | class | M | live | three absolute prohibitions under a `narrative` class |
| F-370 | `flaw.bound_casting_tools` | 5723-5726 | class+src | M | live | an Arcane-Connection duration rule under `narrative`, and an EN source sentence that is truncated |
| F-371 | `flaw.bound_magic` ↔ `virtue.harnessed_magic` | 5727-5730 | incompat | H | live | a stated incompatibility, corroborated from both ends, encoded on neither |
| F-372 | `flaw.bound_magic` | 5727-5730 | desc | L | live | the death clause reaches neither locale (D5) |
| F-373 | `flaw.bound_to_realm` | 5731-5734 | text | — | **withdrawn** | retracted by B10 itself — the German name is a recorded and well-argued decision |
| F-374 | `flaw.bound_to_role_role` | 5735-5748 | prereq | H | live | "only grogs" enforced nowhere, though the mechanism exists and is in use |
| F-375 | `flaw.bound_to_role_role` | 5735-5748 | desc | M | live | four clauses reach neither locale (D5) |
| F-376 | `flaw.branded_criminal` | 5749-5752 | incompat | H | live | a Wealthy exclusion — one of F-340's passages, declared as such |
| F-377 | `flaw.branded_criminal` | 5749-5752 | class+auth | H | live | a Martial permission with no `ability_authorization`, and `narrative` denying it |
| F-378 | `tugenden-fehler.md:75` | — | table | M | live | names an English Virtue that does not exist, and the DE rulebook contradicts itself about the same entry (D6) |
| F-379 | `tugenden-fehler.md:429` | — | table | M | live | gives a German name the DE rulebook does not use (D6) |
| F-380 | `reputationen.md:102` | — | table | M | live | gives a German name the DE rulebook and the other table both contradict (D6) |
| F-381 | `grundbegriffe.md:579` + shipped name | — | table+text | L | live | both reverse the DE rulebook's word order for Wise One |
| F-382 | `flaw.busybody`, `virtue.gossip` | 5761-5764, 3983-3986 | text | M | live | both ship the identical German name, which the canonical table explicitly warns against |
| F-383 | `uncomputed_clauses.rs` | — | engine+audit | H | live | **screen gap:** nine `narrative` reclassifications sit inside a twice-swept block behind a live guard whose vocabulary cannot see any of them — see § 3.1 |

### B11 — `batch-11.md` (ArMDE:5781-5931)

B11 usually closes a finding with a `**Verdict:**` line and no severity; `n/s`
rows below are that, not an omission of mine.

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-384 | `flaw.ceremonial_spontaneous_magic` | 5781-5784 | class+desc | n/s | live | `narrative` on a passage stating an absolute restriction and an incompatibility |
| F-385 | `flaw.ceremonial_spontaneous_magic` + 2 | 5781-5784 | incompat | H | live | two stated incompatibilities, encoded on none of the three sides |
| F-386 | `flaw.chaotic_magic` | 5785-5788 | class | n/s | live | `narrative` on a spontaneous-casting procedure with a numeric tolerance |
| F-387 | `flaw.church_upbringing` | 5789-5792 | class+effect | n/s | live | a mandatory 25-experience-point allocation classified as flavour |
| F-388 | `flaw.church_upbringing` | 5789-5792 | auth | H | live | no `ability_authorization`, so doing what the Flaw *requires* is a hard validator error |
| F-389 | `flaw.companion_animal` | 5805-5808 | class | n/s | live | a granted Personality Trait and a species restriction, classified as flavour |
| F-390 | `flaw.consumed_casting_tools` | 5839-5842 | prereq | H | live | a House restriction that is exactly expressible and absent |
| F-391 | `flaw.consumed_casting_tools` | 5839-5842 | desc | n/s | live | four further mechanical clauses reach neither locale |
| F-392 | `flaw.the_constant_expression` | 5821-5838 | desc+prov | n/s | live | six mechanical clauses in neither locale, and `RULES.md` records that they are there |
| F-393 | `flaw.corrupted_arts` | 5853-5858 | class+prov | n/s | live | `creation_effect` with no effects, on a passage `RULES.md` itself calls in-play (and contradicts itself about) — **a sub-agent withdrawal B11 refused; the finding stands** |
| F-394 | `flaw.corrupted_spells` | 5859-5864 | desc | n/s | live | the whole passage reaches neither locale |
| F-395 | `flaw.covenant_upbringing` | 5865-5868 | desc | L | live | the two clauses the authorization does not cover reach neither locale |
| F-396 | `flaw.creative_block` | 5873-5876 | desc | n/s | live | the experimentation clause and the condition reach neither locale |
| F-397 | `flaw.crippled` | 5877-5880 | class | n/s | live | an absolute prohibition classified as flavour |
| F-398 | `flaw.cyclic_magic_negative` | 5893-5896 | desc | n/s | live | three clauses reach neither locale |
| F-399 | `flaw.deficient_technique` | 5913-5915 | desc | n/s | live | the halving reaches the user in neither locale, where its Minor sibling's does |
| F-400 | `flaw.deficient_form`, `flaw.deficient_technique` | 5909-5915 | param | M | live | no `max_total`, so every Form and every Technique may be deficient at once |
| F-401 | `flaw.deleterious_circumstances` | 5917-5920 | desc+param | n/s | live | the circumstance taxonomy reaches neither locale and no parameter records the choice |
| F-402 | `flaw.deficient_technique` | 5913-5915 | prereq | n/s | live | carries the only `has virtue.hermetic_magus` prerequisite in 122 Hermetic entries; its sibling carries none |
| F-403 | `uncomputed_clauses.rs`; `reputationen.md` | — | engine+audit (+table) | n/s (table row: L) | live | **screen gap:** the swept-block screen missed seven entries; the exact word-forms it lacks — see § 3.1 |
| F-404 | `flaw.deficient_technique` | 5913-5915 | text | n/s | live | interpolates its parameter into its name where the catalogue's convention says it should not; its sibling does not |
| F-405 | `flaw.church_upbringing` | 5789-5792 | text+prov | M | live | the German source at ArMDE:5791 names the wrong Ability, and F-387's fix would copy the error into shipped data |
| F-406 | `flaw.a_deal_with_the_devil` | 5905-5908 | effect | n/s | live | an item-transfer clause the catalogue encodes everywhere else, encoded here nowhere |
| F-407 | `virtue.familiarity_with_the_fae` | 3857-3860 | auth | H | **withdrawn** — dup→F-71 | same entry, same sentence, same fix as B03's F-71 (this index's determination; see § 4) |

### B12 — `batch-12.md` (ArMDE:5932-6067)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-408 | `flaw.excommunicate` | 6044-6047 | rep | H | live | a stated Reputation the catalogue grants twice elsewhere and not here |
| F-409 | `flaw.diabolic_past`, `flaw.faerie_friend`, `flaw.faerie_upbringing` (+≥29 sites) | 5958-6059 | auth | H | live | **umbrella:** three entries grant permission to buy a gated Ability and encode nothing, and the catalogue has the same hole in at least twenty-nine places — see § 3.2 |
| F-410 | `flaw.enfeebled` | 6008-6011 | class+engine | M | live | an Ability-category prohibition and a doubled Fatigue cost, classified as flavour; no effect can forbid a category |
| F-411 | `flaw.difficult_spontaneous_magic` | 5966-5971 | class+incompat | n/s | live | the removal of non-fatiguing spontaneous casting, classified as flavour; B11's incompatibility re-derived from this end |
| F-412 | `flaw.deteriorating_power` | 5944-5949 | class+param | M | live | a written-out formula classified as flavour, and no parameter where its twin has one |
| F-413 | `virtue.variable_power` | 5199-5206 | class+desc | n/s | **withdrawn** — dup→F-336 | same entry, same passage, same verdict as B09's F-336 (this index's determination; see § 4) |
| F-414 | `flaw.exciting_experimentation` | 6040-6043 | class | n/s | live | a dice-substitution rule classified as flavour |
| F-415 | `flaw.exciting_experimentation` | 6040-6043 | text | M | live | the summary is truncated mid-sentence at the book's ellipsis, in **both** locales |
| F-416 | `flaw.envied_beauty` | 6012-6015 | class+prereq | M | live | a Characteristic precondition classified as flavour; the passage names penalties it never states |
| F-417 | `flaw.difficult_underlings` | 5976-5979 | class | L | live | a taking-restriction, classified as flavour |
| F-418 | `flaw.disorientating_magic` | 5984-5987 | class | L | live | the entry's own summary states the rule the classification denies |
| F-419 | `flaw.dutybound` | 5992-5995 | text | M | live | the shipped German name is not the German rulebook's heading, and the German source disagrees with itself |
| F-420 | `flaw.difficult_longevity_ritual` | 5962-5965 | desc | L | live | the halving is computed; its scope and its carve-out reach neither locale |
| F-421 | `flaw.disjointed_magic` | 5972-5975 | desc | M | live | both of its clauses reach neither locale |
| F-422 | `flaw.dwarf` | 5996-5999 | desc | L | live | three effects correct, two clauses in neither locale |
| F-423 | `SurfacedModifier` (6 Flaws) | — | engine | M | live | a surfaced modifier carries no source item, so six different Flaws render as the same unattributed word — a gap Part C does not cover |
| F-424 | `uncomputed_clauses.rs` | — | engine+audit | n/s | live | **screen gap:** the screen missed eleven entries here, and the exact word-forms it lacks — see § 3.1 |
| F-425 | 5 translation-table rows | — | table | L | live | five rows disagree with the German rulebook; the shipped data is right on all five (D6) |
| F-426 | `flaw.ability_block` | 5651-5654 | class+engine | M | **withdrawn** — dup→F-355 | same entry, same passage, same verdict as B10's F-355 (this index's determination; see § 4) |
| F-427 | ArMDE:2816 | — | engine | M | live | the "one Social Status" rule is enforced nowhere, while the *guideline* one paragraph below it is enforced strictly |

### B13 — `batch-13.md` (ArMDE:6068-6235)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-428 | `flaw.feral_upbringing` | 6110-6113 | number | H | live | a Flaw that *replaces* the childhood block is encoded as one that *adds* to it, so the first five years fund 240 XP instead of 120 |
| F-429 | `flaw.failed_monk` | 6068-6071 | auth | H | live | the permission idiom on the entry B12 used as its comparison point, and a live false hard error |
| F-430 | `flaw.harmless_magic` | 6230-6235 | class | M | live | an entire Technique's permanence switched off, classified as flavour |
| F-431 | `flaw.false_power`, `flaw.false_power_minor` | 6080-6097 | class | M | live | a Virtue moved onto a second realm's interaction column, classified as flavour |
| F-432 | `flaw.fettered_magic` | 6114-6117 | class+incompat | M | live | the sentence B09 rated mechanical on the Virtue is `narrative` on the Flaw, and the stated incompatibility is encoded on neither side |
| F-433 | `flaw.fettered_magic` ↔ `virtue.tethered_magic` (+2 more pairs) | 6114-6117, 5141-5144 | text | M | live | three German names each shared by two different entries; the table already resolves one (drove D6's correction) |
| F-434 | `flaw.fluctuating_fortune` | 6150-6153 | class | M | live | a Flaw alternating two named catalogue entries year by year, classified as flavour |
| F-435 | `flaw.failed_student` | 6072-6075 | incompat | M | live | the stated Doctor in (Faculty) exclusion is encoded on neither side |
| F-436 | `flaw.feral_scent` | 6106-6109 | desc | M | live | a -1 social penalty and a doubled spell area, in neither locale |
| F-437 | `flaw.foreign_upbringing` | 6158-6161 | desc | M | live | the Native-Language requirement reaches neither locale |
| F-438 | 2 translation-table rows | — | table | L | live | two rows in this repository's copy disagree with the German rulebook (D6) |
| F-439 | `virtue.lone_redcap` | 4319-4326 | number | H/nil | **blocked** (Q-108) | the same XP double-count as F-428, three batches back; `high` if Q-108 resolves toward "replaces", nil if toward "supplements" |
| F-440 | `flaw.fickle_nature` | 6122-6125 | prereq+number | H | live | obeying the Flaw raises two hard errors, and the rulebook contradicts itself about why |
| F-441 | `flaw.gabai` | 6198-6201 | desc | M | live | the Free-Social-Status compatibility clause reaches neither locale |
| F-442 | `flaw.greater_malediction` (+`flaw.lesser_malediction`) | 6210-6213 | class+desc | M | live | a magnitude-calibration rule and a named immunity, classified as flavour *(the `lesser` half is re-rated as F-452, declared)* |
| F-443 | `types.rs::Effect::SoakMod` | — | prov | L | live | the doc comment states a shipped value wrongly, at exactly the place a reader would check it |
| F-444 | `docs/vf-audit/decisions.md` | — | audit | M | live | D6 stated the EN→DE precedence unscoped, contradicting the three documents it summarises — **already applied to `decisions.md`; verify before re-fixing** |

### B14 — `batch-14.md` (ArMDE:6236-6369)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-445 | `flaw.imagined_folk_tradition_vulnerability` | 6280-6283 | class+auth+param | H | live | `narrative` on a permission the app hard-errors on, and on a cap it expresses nowhere |
| F-446 | `flaw.incompatible_arts` | 6290-6293 | class+param+incompat | H | live | `narrative`, forbids the repeat the book grants, and drops a stated Deficiency exclusion |
| F-447 | `flaw.judged_unfairly` | 6326-6329 | class+incompat | H | live | `narrative` on a prohibition and on a stated incompatibility with fifteen Virtues |
| F-448 | `flaw.hermetic_patron` | 6248-6255 | prereq | M | live | the stated "Redcap or magus" restriction is enforced by nothing |
| F-449 | `flaw.low_tolerance` | 6366-6369 | number | H | live | puts a -1 penalty on two Fatigue levels the rules give no penalty at all |
| F-450 | `flaw.infamous` | 6310-6312 | rep | H | live | hardcodes a Reputation audience the book leaves to the player, and its twin Virtue does not |
| F-451 | `flaw.hunger_for_form_magic` | 6276-6279 | param | M | live | no `max_total`, so all ten Forms may be hungered for at once |
| F-452 | `flaw.lesser_malediction` | 6342-6345 | class+desc | M | live | `narrative` on a magnitude calibration and on an immunity another chapter attributes to it by name *(declared re-rating of F-442's `lesser` half)* |
| F-453 | `flaw.leprosy` | 6338-6341 | desc | M | live | computes two clauses and drops three, in both locales |
| F-454 | `flaw.incomprehensible` | 6294-6297 | desc | M | live | the halving reaches the user in neither locale, and its Lab-Total clause is encoded nowhere |
| F-455 | `flaw.infamous` | 6310-6312 | src | L | live | `source.lines` stops one line short of its passage's end |
| F-456 | `grundbegriffe.md:325` | — | table | M | live | gives `Hobbled` a German name that is already `flaw.crippled`'s (D6) |
| F-457 | `tugenden-fehler.md:344` | — | table | M | live | renders `Folk Tradition` as `Volksmagie` (D6) |
| F-458 | `flaw.loose_magic` | 6354-6357 | text | L | live | the German summary carries a gender error from the source |
| F-459 | `flaw.low_self_esteem` | 6362-6365 | incompat+number | M | live | its "never" is defeated by a Virtue, and no incompatibility stops the pair |

### B15 — `batch-15.md` (ArMDE:6370-6537)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-460 | `flaw.magical_fascination` | 6392-6395 | class+auth+param | H | live | `narrative` on a permission the app hard-errors on, and on a cap it expresses nowhere |
| F-461 | `flaw.monstrous_blood` | 6454-6467 | auth+desc | H | live | computes its aging bonus, hard-errors on its other benefit, and drops four mechanical blocks in both locales |
| F-462 | `flaw.missing_eye` | 6434-6437 | scope+number | H | live | applies the melee penalty to missile weapons and drops the missile penalty entirely |
| F-463 | `flaw.magical_air` | 6382-6385 | prereq+effect | H | live | every magus may take it, which ArMDE:6384 forbids outright, and the Flaw's own -3 reaches nobody |
| F-464 | `flaw.no_hands` | 6496-6499 | class | H | live | `narrative` on a -5 Casting Score penalty; the phrase screen missed it because of a space after the sign |
| F-465 | `flaw.night_terrors` ↔ `flaw.sleep_disorder` | 6488-6495 (+B17) | incompat | H | live | the exclusion is stated from both sides and encoded on neither |
| F-466 | `flaw.no_sense_of_direction` ↔ `virtue.well_traveled` | 6500-6503, 5239-5242 | incompat | M | live | the exclusion is stated and encoded on neither entry |
| F-467 | `flaw.master_of_none` | 6418-6421 | class+desc | H | live | `narrative` on an advancement rule that reaches the user in neither locale |
| F-468 | `flaw.motion_sickness` | 6468-6471 | class | H | live | `narrative` on a doubled Fatigue loss and a two-level floor |
| F-469 | `flaw.magical_being_companion` | 6386-6391 | class+param | H | live | `narrative` on a Might formula, and permits a repeat the book does not |
| F-470 | `flaw.obese` | 6516-6519 | desc | M | live | the movement penalty reaches the user in neither locale |
| F-471 | `flaw.lycanthrope` | 6370-6377 | text | M | live | the German summary is cut mid-sentence, with no ellipsis |
| F-472 | `virtue.cyclic_magic_positive` | 3635-3638 | text | M | **withdrawn** — dup→F-46 | same entry, same defect, same correct value as B02's F-46 (this index's determination; see § 4) |
| F-473 | `flaw.offensive_to_beings` | 6524-6533 | text | M | live | the shipped German description calls Magical Air by a name the app does not use |
| F-474 | `flaw.blatant_magical_air` | 5715-5718 | prereq+incompat | H | **withdrawn** — dup→F-366+F-367 | both halves are B10's F-366 and F-367 (this index's determination; see § 4) |
| F-475 | `virtue.alluring_to_beings` | 3388-3395 | prereq | H | **withdrawn** — dup→F-08 | same entry, same sentence as B01's F-08 (this index's determination; see § 4) |
| F-476 | `virtue.blood_of_the_nephilim` | 3504-3518 | auth+incompat | H | **withdrawn** — dup→F-22+F-23 | both halves are B01's F-22 and F-23 (this index's determination; see § 4) |
| F-477 | `virtue.demonic_blood` | 3649-3662 | auth | H | **withdrawn** — dup→F-50 | same entry, same sentence as B02's F-50 (this index's determination; see § 4) |
| F-478 | `flaw.lycanthrope` | 6370-6377 | desc | H | live | a rule the book attributes to the Flaw by name, from 3,500 lines away, reaches the user in neither locale |
| F-479 | `virtue.mendicant_friar` | 4488-4495 | text | M | **withdrawn** — dup→F-193 | same entry, same defect, same correct value as B06's F-193 (this index's determination; see § 4) |
| F-480 | `tugenden-fehler.md` | — | table | M | live | the canonical glossary carries two rows for one English headword, and the second states rules mechanics (D6) |

### B16 — `batch-16.md` (ArMDE:6538-6662)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-481 | `flaw.outcast` | 6538-6541 | incompat | — | **withdrawn** | retracted by B16 itself — duplicate of B09's F-340, row eight |
| F-482 | `virtue.almogavar`, `virtue.guild_apprentice`, `virtue.priest` | 3404-3409, 4041-4044, 4796-4805 | incompat | H | live — partly dup | an **extension of F-340**: the passage list is a closed set of ten flat sites plus one conditional, and the "granted Redcap" caveat retires. Two of its three named sites are already F-11 and F-100 |
| F-483 | `flaw.outlaw`, `flaw.outlaw_leader` | 6542-6549 | auth | H | live | both permit Martial Abilities and the app refuses to build either character |
| F-484 | `flaw.outlaw` | 6542-6545 | rep | L | live | the granted Reputation hardcodes an audience the passage does not name |
| F-485 | `flaw.outlaw_leader` | 6546-6549 | prereq | M | live | "Grogs may not take this Flaw" is in no profile's `forbidden_traits` |
| F-486 | `flaw.outsider_major` / `_minor` | 6550-6561 | rep+number | M | live | both ship a fixed Reputation level where the book gives one range to both |
| F-487 | `flaw.pagan` | 6570-6573 | mag | H | live | the book offers Pagan in two magnitudes, the catalogue ships one, and the missing one is the grog's |
| F-488 | `flaw.pagan` | 6570-6573 | class+auth | H | live | `narrative` on a permission the app hard-errors on |
| F-489 | `flaw.poor_hearing` | 6614-6617 | class+engine | M | live | `narrative` on a stated -3; no `Effect` variant can name an Ability for a roll modifier |
| F-490 | `flaw.poor_characteristic` (+`virtue.great_characteristic`) | 6598-6600, 3987-3989 | text | M | live | the shipped summary narrows the rule and adds a claim the passage does not make; the mirror narrowing is on the Virtue |
| F-491 | `flaw.poor_eyesight` | 6606-6609 | desc+scope | M | live | computes the combat third of its -3 and drops the rest in both locales |
| F-492 | `flaw.poor_student` | 6626-6628 | desc | M | live | the floor of one reaches the user nowhere |
| F-493 | `flaw.painful_magic` | 6574-6577 | desc | M | live | drops the two clauses that say what a "pain level" is |
| F-494 | `flaw.raised_from_the_dead` | 6646-6649 | class+effect | H | live | `uncomputed_rule` with no effects, although the engine can express two of its clauses |
| F-495 | `flaw.realm_stigmatic` | 6654-6658 | param | M | live | parameterized with no `max_total`, so four copies are legal |
| F-496 | `flaw.palsied_hands`, `flaw.primogeniture_lineage`, `flaw.raised_from_the_dead` | 6578-6581, 6634-6637, 6646-6649 | text | M | **withdrawn** (name question) — see D7 | three shipped German names disagree with the tables; **D7 (2026-09-21) rules the DE rulebook heading wins and all three shipped names are correct**. What survives is the obligation to annotate the three table rows in both projects |
| F-497 | `reputationen.md:112-113` | — | table | L | live | states Outlaw's magnitude and Reputation level wrongly in both rows; there is **no** row for Outlaw Leader at all (D6 rule 1) |
| F-498 | `flaw.servant_of_the_land` | 6717-6720 (B17) | effect | M | live | grants a budget-exempt Prohibition and encodes neither half |
| F-499 | `docs/vf-audit/corrections.md` | — | audit | M | **this file closes it** | the file declared at `README.md:93` did not exist; sixteen batches were unindexed |
| F-500 | `WarpingGrant.score` (`flaw.warped_by_magic`, B18) | — | engine+prov | L | live | dead data with no integrity check; F-494 would be its second carrier |
| F-501 | 5 `Anmerkung` cells | — | table | L | live | five make provenance claims the core book contradicts, and the audit has been applying two standards to them (D6 rule 1) |

---

## 2. Ordering constraints

Three of these are hard. A slice that ignores them produces a red it cannot fix
without rewriting rulebook text.

### 2.1 `MECHANICAL_PHRASES` must grow **before** any `narrative` → `uncomputed_rule` reclassification lands

`crates/arm-rules/tests/uncomputed_clauses.rs` carries two assertions that pull in
opposite directions:

- `no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` screens a
  `narrative` entry's **English source passage** and is green on every entry this
  audit reclassifies — that is why the entries were missed.
- `every_uncomputed_rule_entry_states_its_rule_in_every_locale` is **unscoped** and
  applies the *same* `states_a_mechanical_rule` detector to an `uncomputed_rule`
  entry's shipped `description` in **every locale**.

So writing a missed passage verbatim into `description` turns the first assertion
green and the second one **red**, because the passage contains no token the
detector knows. B11 measured six of its seven reclassifications blocked this way;
B12 measured eight of eleven. **Extend the phrase list first, then reclassify.**
The consolidated word-form list is § 3.1.

### 2.2 D2 is pending — one finding is blocked on it, and two validators must not be touched

`decisions.md` D2 ("granted Virtues and the bought-only validators") is **not
ruled**. Until it is, no slice may change
`validate_ability_bonus_targets` or `validate_characteristic_delta_preconditions`,
and no slice may declare their current behaviour correct. B04 read
`virtue.great_characteristic` (ArMDE:3987-3989) and recorded it as "D2 evidence";
the ruling itself has not been taken.

### 2.3 Other blocked or ruling-dependent rows

| Finding | Blocked on | Note |
|---|---|---|
| F-439 | **Q-108** | `high` if Q-108 resolves toward "replaces", **nil** if toward "supplements". Do not work it either way first. |
| F-85 | **Q-10** | the German name for `virtue.fabric_ripper`; Q-10 is the Covenfolk two-sources question. |
| F-270, F-283 | a catalogue decision | the required Virtue does not exist; the fix is either to add the entry (B02's F-34 shape) or to drop the requirement. |
| F-496 | **settled by D7** | D7 (2026-09-21) rules the DE rulebook heading wins. The name change is **cancelled**; the table-annotation obligation remains. |
| F-444 | already applied | `decisions.md` D6 now carries the corrected two-rule precedence. Verify rather than re-fix. |
| all `table` rows | **D6** | these are **not** Phase 2 work. D6 obliges them fixed in `arm-char-gen` *and* `arm-de-translation` immediately, and first checked against the source project for staleness. |

### 2.4 Within Phase 2, work order that avoids rework

1. `MECHANICAL_PHRASES` (§ 3.1) — unblocks § 3.3.
2. The authorization family (§ 3.2) — it is the largest group, it is mostly
   mechanical, and several classification findings resolve to `creation_effect`
   *because* the authorization exists, so doing it first removes them from § 3.3.
3. Classifications + descriptions (§ 3.3, § 3.4) together — D5 makes them one job
   per entry, and doing them separately means opening the same passage twice.
4. Everything else.

---

## 3. Grouped correction plan

Each group names the change, the findings, the files, whether a test moves, and
the constraint. **Finding lists are by primary kind**; a finding whose secondary
kind belongs to another group is worked under its primary and mentioned there.

### 3.1 Extend `MECHANICAL_PHRASES` — the consolidated missing word-form list

**Change.** Add the word-forms below to `MECHANICAL_PHRASES` in
`crates/arm-rules/tests/uncomputed_clauses.rs`, in both locales, and fix the two
*structural* blind spots the batches identified alongside them.

**Findings.** F-383 (B10), F-403 (B11), F-424 (B12), plus the per-finding tables in
B13 (six entries), B14 (five), B15 (nine) and B16 (three). **This is the single
list; it was scattered across seven documents before this file.**

**Files.** `crates/arm-rules/tests/uncomputed_clauses.rs` only. **This is a test
change, and it is the only one that must land before data changes.**

**The baseline, corrected 2026-09-21.** Batches B10, B11 and B13 describe the
screen as "the **20-phrase** list", and the orchestrator repeated that figure in
every brief from B12 on. **It is wrong.** `uncomputed_clauses.rs:179-219` holds
**33** entries, counted item by item — and they are **bilingual pairs**, so the
list covers roughly **17 concepts in two languages**, not twenty independent
idioms. The figure came from `ecb5150`'s commit message and was never checked
against the array. The batch files keep the old number because they are dated
records, but any slice working this group must start from **33 / ~17 concepts**.

**And the German half is where the real problem is** — B17 measured it. A
contiguous, left-word-boundary substring match **structurally cannot express
German discontinuous negation**: four of the five German prohibitions in B17's
span put one or more words between the two halves of the negation (`kann … nicht
anwenden`, `darf … nicht nehmen`). Family 1 below is therefore **not
implementable as written in German**, and adding its DE forms as literal
substrings would leave the guard red on exactly the entries the reclassification
is meant to fix. **This must be decided before any data lands** — see § 2.1 —
because the alternative pressure is to reword shipped rulebook text to satisfy a
detector, which is what `ecb5150` was written to stop.

#### The list — 14 idiom families, ~60 word-forms

Recorded as families, because every batch that found a miss found it to be a
*family* rather than a single string, and the module's own doc comment already
argues for breadth bounded by `NO_RULE_DESPITE_TOKEN` rows.

| # | Family | EN forms | DE forms | Sourced from |
|---|---|---|---|---|
| 1 | **Prohibition** | `may not take`, `cannot take`, `may not have`, `may only take this`, `is impossible`, `unable to learn`, `can't apply`, `cannot walk`, `cannot cast`, `cannot permanently destroy`, `cannot gain` | `darfst nicht`, `dürfen nicht`, `darf … nicht nehmen`, `kann nicht genommen werden`, `nicht in der Lage`, `unmöglich`, `kannst nicht`, `kann … nicht anwenden`, `nicht gehen`, `können nichts` | F-383, F-403, F-424, F-447, F-467, F-475, F-481/F-482 |
| 2 | **Permission** | `may take`, `may learn`, `may purchase`, `may begin with`, `can purchase`, `is allowed to have`, `allows the character to purchase`, `even if normally`, `at character generation`, `at character creation` | `darfst … wählen`, `darf … erwerben`, `darf … erlernen`, `darf … haben`, `erlaubt … zu kaufen`, `selbst wenn du normalerweise`, `bei der Charaktererschaffung` | F-409, F-424, F-429, F-445, F-460, F-483, F-488 |
| 3 | **Absolutes beyond `cannot die`** — stem the existing entry | `cannot <verb>` generally, not the single literal | `nicht sterben` → the discontinuous-negation limit is real; add `kannst keine`, `Du kannst nicht` | F-383, F-403, F-424 |
| 4 | **Eligibility** | `You must be … to take this`, `may only be taken by`, `Only characters with` | `Du musst … sein, um diesen Fehler zu wählen`, `darf nur von … genommen werden` | F-448, F-403, F-367 |
| 5 | **Incompatibility / exclusion** | `is not compatible with`, `is incompatible with`, `may not also take`, `may not be combined with`, `may not have both` | `ist nicht … vereinbar`, `ist unvereinbar mit`, `darf nicht zusätzlich`, `darf … nicht … kombiniert werden`, `kann nicht beide` | F-403, F-446, F-465, F-466 |
| 6 | **Item transfer / composition** | `includes the effects of` | `schließt die Auswirkungen von … ein` | F-403/F-406 — **exactly 4 hits in the whole English core book, 3 already non-`narrative`, so it produces exactly one new red** |
| 7 | **Multiplier stem** | `multiplier` | `multiplikator` | F-383 — the narrow form the doc comment did not consider (it rejected the over-broad `multipl`) |
| 8 | **Multiplier in words** | `double`, `twice`, `halve` | `doppelt`, `halbiert` | F-424, F-468 |
| 9 | **`more` without the `or`** | relax `or more`'s left anchor to catch `possibly more` | — | F-383 |
| 10 | **Bare `no more`** | `no more` (the list has only `no more than`) | `nicht mehr` (the list has only `nicht mehr als`) | F-445, F-460 |
| 11 | **Floor** | `minimum` | `Mindest` | F-468 |
| 12 | **Magnitude/level stem** | `magnitudes` (the list has only `one magnitude`), `one level`, `by more than one level` | `Magnituden`, `um mehr als eine Stufe` | F-403, F-412/F-424 |
| 13 | **Bare imperative modifier** | `Subtract N`, `Add N` with no sign character | `Ziehe N … ab`, `Addiere N` | F-489 — *"the sharpest miss in the batch"*: the rule is the entry's whole first sentence |
| 14 | **Named rulebook terms** (the list already knows `ease factor`) | `Advancement Total`, `Wealth Multiplier`, `Reputation … at level N`, `experience points … must be spent on`, `an additional Personality Trait of`, `spend a round`, `game mechanical effects`, `two dice instead of` | `Fortschrittssumme`, `Reputation der Stufe N`, `Erfahrungspunkte … ausgeben`, `eine Runde lang`, `spielmechanische Auswirkungen`, `zwei Würfel statt` | F-403, F-424, F-467, F-445 |

#### Two blind spots that are **not** vocabulary, and need their own fix

1. **`has_signed_number` only looks rightward from the sign, and only at an
   immediately adjacent ASCII digit.** Two shipped passages defeat it:
   - `flaw.no_hands` (ArMDE:6498) writes `– 5` — U+2013, **a space**, then `5`.
     Verified byte-by-byte with `od -c`; it is the **only** sign-space-digit
     occurrence in the whole Flaws block (F-464).
   - `flaw.magical_being_companion` (ArMDE:6388) writes `10 – Size` — the only
     digit is on the **left** of the operator (F-469).
2. **The screen reads only the entry's own cited range, and only `narrative`
   entries.** A rule the book states in another chapter about a named entry is
   structurally invisible (F-442's ArMDE:7397, F-452, F-453, F-478), and an
   `in_play_effect` or `creation_effect` entry that computes one clause and drops
   five is examined by no assertion at all (F-461, F-465, F-470). **This is D5's
   first obligation** — the guard must stop being class-keyed — and it is the
   larger of the two fixes.

**Ordering.** Nothing else in this document may land before this, per § 2.1.

### 3.2 The authorization family — `ability_authorization` where the book grants a gated Ability

**Change.** Add `{"type":"ability_authorization", "abilities":[…]}` or the
category form to every entry whose passage grants permission to buy a gated
Ability or Ability category, and reclassify `narrative` entries in the family to
`creation_effect`.

**Why it is first after the screen.** It is the largest single group, the fix is
formulaic, and it is the audit's highest-yield **live-bug** family: the shipped
app raises a hard `ability_category_requires_virtue` error on a character the book
describes. B12's F-409 puts the hole at **at least twenty-nine** passages with
exactly two encoded; this index finds **63 findings** on it.

**Findings (primary `auth`).**
F-18, F-22, F-50, F-59, F-67, F-68, F-69, F-71, F-72, F-101, F-102, F-128, F-132,
F-155, F-163, F-178, F-180, F-182, F-185, F-186, F-188, F-190, F-195, F-245,
F-253, F-260, F-264, F-273, F-292, F-323, F-347, F-388, F-409, F-429, F-461,
F-483.
**Plus the `class+auth` rows, which are the same fix with a classification change
riding on it:** F-06, F-09, F-10, F-14, F-17, F-26, F-27, F-28, F-29, F-38, F-66,
F-122, F-209, F-216, F-230, F-232, F-247, F-266, F-302, F-310, F-325, F-326,
F-349, F-377, F-445, F-460, F-488.

**Files.** `rules/core/virtues_flaws.json`. Where the classification moves, the
entry also needs `description` in `rules/i18n/{en,de}/virtues_flaws.json` per D5.
`crates/arm-rules/RULES.md` gains a row per entry.

**Tests.** No new machinery — `validation/authorization.rs::validate_ability_authorization`
and `effective/xp.rs::ability_authorizations` already do the work. Add per-entry
assertions; do **not** assert a catalogue total (CLAUDE.md's catalogue-size
invariant).

**Constraint.** Several of these entries are also in § 3.3; do the authorization
and the reclassification in one edit per entry.

### 3.3 Reclassification — `narrative` on a passage that states mechanics

**Change.** `classification` moves, and per D5 every clause the engine does not
compute is written into `description` in **both** locales.

**Findings (primary `class`).** The largest group after § 3.2. Includes the whole
`narrative`-on-a-plain-number family (F-126, F-127, F-207, F-213, F-224, F-271,
F-295, F-489), the immunities (F-95, F-110, F-111, F-140), and the entries that
carry a whole subsystem (F-125, F-219, F-289, F-336).

**Files.** `rules/core/virtues_flaws.json` + both `rules/i18n/*/virtues_flaws.json`.

**Tests.** `uncomputed_clauses.rs` — and **this group is blocked on § 3.1**.

### 3.4 Missing descriptions (D5) — a computed entry that drops a clause

**Change.** Write the uncomputed clause into `description` in both locales. The
entry's `classification` does **not** move (D5's closing paragraph: "the
obligation moves; the taxonomy does not").

**Findings (primary `desc`).** The single largest kind in the index.

**Files.** `rules/i18n/{en,de}/virtues_flaws.json` only, in most cases.

**Tests.** D5 obliges `uncomputed_clauses.rs` to stop being class-keyed — see
§ 3.1's second blind spot. Until that lands, these entries are unguarded; they are
still correct to write.

### 3.5 Incompatibilities

**Change.** Populate `incompatible_with` **on both sides** —
`ruleset/integrity.rs::validate_incompatibility_symmetry` fails the load on a
one-sided declaration.

**Findings.** F-02, F-11, F-23, F-41, F-51, F-79, F-191, F-217, F-226 (a *removal*:
the shipped pair contradicts ArMDE:4742), F-242, F-291, F-298, F-352, F-360 (also a
removal), F-366, F-371, F-385, F-435, F-459, F-465, F-466, plus the umbrella
**F-340 / F-376 / F-482**.

**The Wealthy/Poor sub-slice is a closed set and must be worked once.** B16's
verification pass made it mechanical: one `rg` over the prohibition's six spellings
returns **eleven** passages in the whole book and nothing else — **ten flat plus one
conditional**. Known sites already carrying a finding: `virtue.almogavar` (F-11),
`virtue.covenfolk` + `virtue.custos` (F-41), `virtue.guild_apprentice` (F-100),
`virtue.mendicant_friar` (F-191), `virtue.perfectus` (F-217), `virtue.priest`
(F-233, the conditional one), `virtue.turb_trained` (F-325), `flaw.branded_criminal`
(F-376), `flaw.outcast` (F-340 row eight / withdrawn F-481). **Work F-340 as the
single slice and close the per-entry findings against it.**

**Files.** `rules/core/virtues_flaws.json`, `RULES.md`.

**Tests.** The load-time symmetry guard already exists; add per-pair assertions.

### 3.6 Prerequisites and eligibility gates

**Change.** Add the `prerequisites` tree, or the profile-side `forbidden_traits`
row, that the passage states.

**Findings.** F-08, F-24, F-30, F-35, F-56, F-136, F-145, F-169, F-172, F-183,
F-223, F-252, F-261, F-305, F-318, F-322, F-324, F-339, F-359, F-367, F-374,
F-390, F-402, F-416, F-440, F-448, F-463, F-485.

**Files.** `rules/core/virtues_flaws.json`; `rules/core/character_types.json` for
the entity-kind ones (F-24, F-318, F-374, F-485); `RULES.md`.

**Constraint.** **D2 (§ 2.2).** Several of these gates are also reachable through
a *grant*, and D2 governs whether a granted Virtue must satisfy them.

### 3.7 Parameters, `max_total` and unrecordable choices

**Change.** Add the parameter the book's choice requires; bound a free-text
parameter to its closed set; add `max_total` where ArMDE:2814 makes the Virtue
once-only.

**Findings.** F-03, F-07, F-40, F-57, F-70, F-94, F-98, F-112, F-139, F-141,
F-144, F-157, F-165, F-220, F-227, F-281, F-287, F-294, F-328, F-348, F-358,
F-362, F-400, F-412, F-451, F-495.

**Files.** `rules/core/virtues_flaws.json`.

**Note.** Several of these are **data loss** — the save cannot round-trip a choice
the rules require — which CLAUDE.md rates at the top. **Q-86 / Q-88 / Q-95 ask
whether every "choose the specifics" entry should carry a parameter; that is one
ruling with many consequences and is not settled.**

### 3.8 Scope — a conditional folded in flat, or a bonus on the wrong total

**Change.** Narrow the effect to the passage's stated scope, or move it to the
field that already exists (`LabTotal::within_focus`).

**Findings.** F-04, F-15, F-20, F-31, F-36, F-43, F-45, F-60, F-73, F-74, F-75,
F-89, F-91, F-105, F-149, F-198, F-225, F-234, F-260, F-284, F-297, F-344, F-356,
F-462, F-491.

**Files.** `rules/core/virtues_flaws.json`; `crates/arm-rules/src/derived/lab.rs`
and `effective/spell.rs` for the D1/D4 pair; `RULES.md`.

**Constraint — read D1 and D4 together, and do not generalise either.**
D1 governs `effective/spell.rs::spell_level_cap` **and nothing else**: all nine
`lab_total_mod` entries apply flat there, conditions deliberately ignored. D4
governs everything else computed from the lab grid and goes the *other* way: each
of the nine is decided statically from its passage, and `virtue.potent_magic_*`
belongs in `within_focus`, not in `total`. `virtue.aristotelian_training` is
explicitly **flagged and not yet ruled on** by D4.

### 3.9 Wrong numbers and wrong arithmetic

**Change.** Correct the value or the computation. **This is CLAUDE.md's top
severity class** — the app's purpose is computing correct characters.

**Findings.** F-89, F-244, F-256, F-285, F-306, F-351, F-428, F-440, F-449, F-462,
F-486, and **F-439 (blocked on Q-108)**.

**Files.** `rules/core/virtues_flaws.json`; `crates/arm-rules/src/` for F-256,
F-428, F-449; `RULES.md` for every changed number.

**Tests.** Each needs a red that asserts the wrong number today.

### 3.10 Missing effects the engine can already express

**Change.** Add the `effects` row. No new machinery.

**Findings.** F-01, F-19, F-21, F-39, F-49, F-55, F-77, F-196, F-205, F-208,
F-243, F-249, F-296, F-329, F-341, F-342, F-406, F-463, F-494, F-498.

**Files.** `rules/core/virtues_flaws.json`, `RULES.md`.

### 3.11 Reputations

**Change.** Add the `grants_reputation` the passage states; correct the level; stop
hardcoding an audience the book leaves open.

**Findings.** F-80, F-235, F-254, F-312, F-316, F-363, F-408, F-450, F-484, F-486.

**Blocked on questions that recur across the whole catalogue:** **Q-43** (a granted
Reputation has no polarity, so "a *poor* Reputation at level 2" is
unrepresentable), **Q-73** (no `ReputationType` is organization-scoped), **Q-80**
(is "of your choice" a wildcard?), **Q-127** (a `score` cannot hold a range), and
the Phase-0 question of whether `score` is a ceiling, a fixed value or a
suggestion. **Settle those before the slice, not during it.**

**Files.** `rules/core/virtues_flaws.json`; `crates/arm-rules/src/types.rs` if the
polarity or the type widens; `ui/src/lib/components/Reputations.svelte` pre-fills
the level (F-486).

### 3.12 Shipped text and localization

**Change.** Repair the shipped string in `rules/i18n/{en,de}/virtues_flaws.json`.

**Findings.** F-05, F-16, F-46, F-54, F-65, F-84, F-85 *(blocked on Q-10)*, F-88,
F-108, F-113, F-120, F-130, F-135, F-152, F-168, F-193, F-263, F-276, F-320,
F-381, F-382, F-404, F-405, F-415, F-419, F-433, F-458, F-471, F-473, F-490.

**The truncation sub-slice has one cause and must be fixed once.** A summary
extractor reading an abbreviation's full stop as a sentence end produced
**F-46** (`virtue.cyclic_magic_positive`, DE, cut at `(z.`), **F-193**
(`virtue.mendicant_friar`, **EN**, cut at `St.`), **F-471** (`flaw.lycanthrope`,
DE) and **F-415** (`flaw.exciting_experimentation`, **both** locales, cut at the
book's ellipsis). B15 records the scan that finds them without guessing:
**EN/DE summary length ratio above 2×**, catalogue-wide, which returns exactly the
three plus one benign row (`virtue.tough`).

**Constraint.** **D7** now governs a German name that disagrees with a glossary
table: the DE rulebook heading wins by default; the glossary wins only where the
heading is defective (a collision, or a term the rulebook never renders); a
glossary row tagged to another book is not in the dispute. That cancels F-496's
name change and settles Q-129.

### 3.13 Translation tables — **D6, not Phase 2**

**Change.** Correct the row in `rules/source/de/translation-tables/` **and in
`arm-de-translation`, in the same step** — but first check the source project,
because D6's closing paragraph records that the copy here can simply be **stale**
(the `Covenfolk` case was).

**Findings.** F-308, F-345, F-378, F-379, F-380, F-425, F-438, F-456, F-457,
F-480, F-497, F-501, plus the table row embedded in F-403.

**Blocker, confirmed rather than skipped.** `arm-de-translation` is outside the
repository and denied by argument-path containment, so a background agent cannot
make the stale-copy-versus-real-error determination. **F-496, F-497, F-501 and
Q-129 all stop at "this repository's copy disagrees with the rulebook."**

### 3.14 `source.lines`, anchors and provenance

**Findings.** `src`: F-48, F-200, F-215, F-309, F-370, F-455. `prov`: F-16 (the
**id** `virtue.rard` is wrong, not only the name), F-34 (four rulebook Virtues
absent from the catalogue), F-270, F-283 (a required Virtue that does not exist),
F-286, F-307, F-392, F-393, F-443, F-500.

**Files.** `rules/core/virtues_flaws.json`, `crates/arm-rules/RULES.md`,
`crates/arm-rules/src/types.rs` (doc comments).

**Note.** F-393 is the one finding in the audit that a verification sub-agent tried
to withdraw and **B11 refused**, on the ground that `RULES.md:5411` and
`RULES.md:5226-5229` say opposite things about whether Corrupted Arts has a
creation number. **The finding stands, and the `RULES.md` self-contradiction is
part of the fix.**

### 3.15 Engine gaps — new machinery, not data

**Change.** These cannot be fixed in `rules/core/`. Each needs a decision first.

| Finding | Gap |
|---|---|
| F-01 | `AbilityRollMod` cannot express "the six subjects that are *not* the param's value" |
| F-42, F-63 | a player-chosen restricted Ability *group* is unrepresentable |
| F-123, F-170, F-179, F-189, F-192 | no sex/gender model, so "only available to male characters" can only be text |
| F-194 | no age-cap waiver, and the app errors on the character the passage describes |
| F-317 | a player-chosen gated Ability group |
| F-329 | True Faith → Magic Resistance is computed from nothing (and **Q-74** asks how it joins the grid) |
| F-334 | `Prereq` has no parameter-comparing variant, so reciprocity is inexpressible |
| F-355, F-410, F-426 | no *negative* authorization: nothing can forbid an Ability category |
| F-363 | `grants_reputation` cannot say a Reputation is **bad** |
| F-383, F-403, F-424 | the phrase screen — § 3.1 |
| F-423 | `SurfacedModifier` carries no source item, so six Flaws render as one unattributed word |
| F-427 | ArMDE:2816's "one Social Status" rule is enforced nowhere |
| F-489 | no `Effect` variant can name an Ability for a *roll* modifier (all 42 enumerated) |
| F-500 | `WarpingGrant.score` is dead data with no integrity check |

### 3.16 The audit's own records

| Finding | What |
|---|---|
| F-444 | `decisions.md` D6's unscoped precedence rule — **already corrected in `decisions.md`; verify** |
| F-499 | `corrections.md` did not exist — **this file** |
| — | B15's F-472/F-474/F-475/F-476/F-477/F-479 each assert that an earlier batch "did not report" a defect that the earlier batch did report. Six false statements of record; see § 4. |

---

## 4. Cross-batch duplicate check

This is the check whose absence produced **F-499**. Nobody had run it before.

### 4.1 Headline

**Ten duplicate relationships.** One (F-481) was already found and withdrawn by
B16. **Nine were not**, and every one of them is a *cross-span* finding — a batch
following a pointer or a scan out of its own range and rating an entry another
batch had already rated. Three further findings overlap an umbrella at the
*site* level.

In six of the nine, the later finding states in its own body that the earlier
batch "did not report" the defect. **That claim is false in all six cases**, and
it is the mechanism: the later batch asserted a negative about a document it had
not read, because nothing indexed the findings.

### 4.2 How I searched — so the number can be trusted

Four passes, in this order:

1. **Every finding heading, mechanically.** `grep "^### F-"` over all sixteen
   files gives 500 headings (F-001…F-501 less F-44). Extracted every
   `virtue.*` / `flaw.*` id from them and counted ids appearing in more than one
   heading. That produced ~90 multi-finding entries.
2. **Filtered to cross-batch.** For each of those ids, `grep -n "^### F-.*<id>"`
   across files and kept only ids whose findings sit in **different** batch files
   — most multi-finding entries are one batch rating several aspects of one entry
   and are not duplicates.
3. **Read both bodies of every cross-batch pair** and compared the passage cited,
   the current-data claim, and the proposed correct value. A pair counts as a
   duplicate only when all three match.
4. **Checked whether the later batch knew.** For B11, B12, B14, B15 and B16 I
   extracted every `F-NNN` reference in the file (`grep -o "F-[0-9]*"`, sorted
   unique) and checked whether the earlier finding's number appears. B14 and B16
   *do* cite the earlier finding where they overlap, and are therefore declared
   overlaps rather than blind duplicates. B11, B12 and B15 do not.

**What this method would miss, stated honestly.** It keys on the **entry id in the
heading**. Two findings describing the same defect on the *same* entry from within
one batch are excluded by design (they are usually different aspects). A duplicate
where the two findings name the entry only in their bodies and not in their
headings would not be caught — I did not read all 500 bodies, only the ~25 that
step 2 selected plus the sections § 5 draws on. **So "nine" is a floor, not a
ceiling.** The umbrella findings (F-340, F-409) are the likeliest place for more:
F-409's "at least twenty-nine" sites are not enumerated anywhere, and each one may
already carry a per-entry finding.

### 4.3 The duplicates

| # | Later | ≡ Earlier | Entry | Evidence |
|---|---|---|---|---|
| 1 | **F-407** (B11) | **F-71** (B03) | `virtue.familiarity_with_the_fae` | Same sentence (ArMDE:3859), same missing `ability_authorization`, same proposed `uncomputed_rule` → `creation_effect`. B11: *"B03 can re-derive this far more cheaply than it can rediscover it."* |
| 2 | **F-413** (B12) | **F-336** (B09) | `virtue.variable_power` | Same passage (ArMDE:5201/:5203), same `narrative`, same verdict. B12: *"recorded here because B09 can re-derive this…"* |
| 3 | **F-426** (B12) | **F-355** (B10) | `flaw.ability_block` | Same passage (ArMDE:5653), same `narrative`, same missing negative-authorization mechanism. |
| 4 | **F-472** (B15) | **F-46** (B02) | `virtue.cyclic_magic_positive` | Same German summary cut at `(z.`, same correct value. B15: *"B02 did not report it."* — **false.** |
| 5 | **F-474** (B15) | **F-366 + F-367** (B10) | `flaw.blatant_magical_air` | Both halves: the Blatant Gift mutual exclusion **and** the `Only characters with a Magical Air or The Gift` prerequisite. B15: *"B10 did not report either restriction."* — **false on both.** |
| 6 | **F-475** (B15) | **F-08** (B01) | `virtue.alluring_to_beings` | Same sentence (ArMDE:3392), same missing prereq tree, same `flaw.offensive_to_beings` idiom as the model. B15: *"B01 did not report the restriction."* — **false.** |
| 7 | **F-476** (B15) | **F-22 + F-23** (B01) | `virtue.blood_of_the_nephilim` | Both halves: the Dominion Lore permission (F-22) and the ArMDE:3517 prohibition list (F-23). B15: *"B01 did not report either clause."* — **false on both.** |
| 8 | **F-477** (B15) | **F-50** (B02) | `virtue.demonic_blood` | Same sentence (ArMDE:3655, Infernal Lore without Arcane Lore). B15: *"B02 did not report the permission."* — **false.** |
| 9 | **F-479** (B15) | **F-193** (B06) | `virtue.mendicant_friar` | Same English summary cut at `St.`, same correct value, character-for-character. B15: *"B06 did not report it."* — **false.** |
| 10 | **F-481** (B16) | **F-340** row 8 (B09) | `flaw.outcast` | **Already withdrawn by B16.** Its verification pass found it and filed F-499. |

**Status applied.** Rows 1-9 are marked `withdrawn — dup→F-NNN` in § 1. **That is
this index's determination, not the batch owners'** — the batch files still
present those nine as live, and nothing in them has been edited. Row 10 was
withdrawn by its own batch.

**Do not simply delete the withdrawn nine's evidence.** Each adds something its
predecessor lacks and a Phase 2 slice should read it alongside the surviving
finding: F-474 adds the `RULES.md:921-923` precedents; F-475 adds the third
sibling entry; F-476 and F-477 add the F-409 census position; F-479 adds the
EN/DE length-ratio scan that finds all three truncations without guessing.

### 4.4 Site-level overlaps inside an umbrella finding

Not duplicate findings, but the same defect counted twice. **Work once.**

| Umbrella | Overlaps | Note |
|---|---|---|
| **F-340** (Wealthy/Poor, B09) | **F-11** (B01, `virtue.almogavar`), **F-41** (B02, `virtue.covenfolk` + `virtue.custos`), **F-100** (B04, `virtue.guild_apprentice`), **F-191** (B06, `virtue.mendicant_friar`), **F-217** (B07, `virtue.perfectus`), **F-233** (B07, `virtue.priest`, the conditional site), **F-325** (B09, `virtue.turb_trained`), **F-376** (B10, declared), **F-481** (withdrawn) | F-340 is *later* than four of these and did not cite them. B16's pass closed the set at ten flat sites plus one conditional. |
| **F-482** (B16, F-340's extension) | **F-11**, **F-100** | Two of the three sites F-482 calls "new" already carry a finding. Only `virtue.priest` (:4800) was genuinely unrated — and B07's F-233 mentions it as a conditional Poor exclusion. |
| **F-409** (permission idiom, B12) | the entire § 3.2 list | F-409 is a census of "at least twenty-nine" sites; ~63 per-entry findings are instances of it. B16's pass dropped F-409's "instances 35/36/37" ordinals as unreconstructable. |
| **F-442** (B13) / **F-452** (B14) | `flaw.lesser_malediction` | **Declared.** B14 explicitly corrects B13's span attribution and claims the entry. |
| **F-319** (B09) / **F-432** (B13) | `virtue.tethered_magic` ↔ `flaw.fettered_magic` | **Declared.** B13 cites B09 and rates the Flaw twin plus the incompatibility, which is new. |

---

## 5. What Phase 2 must not re-open

The batches recorded cleared negatives so a correction pass does not "fix" a thing
that is right. The full lists are in each batch's *Negatives cleared* /
*Sub-agent reconciliation* section; these are the ones most likely to be
re-litigated.

- **`virtue.affinity_art`** — "you may exceed the normal recommended limits" waives
  nothing: `validation/scores.rs::validate_arts` enforces no Art cap. The book
  states "by two points" for the Ability version (ArMDE:3374) and omits it for the
  Art version (:3378). *(B01)*
- **`virtue.the_enigma`** and **`virtue.convoluted_mind`** need **no**
  `ability_authorization` despite naming `arcane` Abilities — the first because
  `AbilityScoreGrant` folds into the permitted set and the `is_magus` exemption
  covers a Criamon, the second because ArMDE:3603 simply does not state the
  permission that ArMDE:3655 does. *(B02)*
- **`virtue.death_prophecy`** is correctly `uncomputed_rule`, not an `aging_mod` —
  "cannot die as a result of wounds or old age" matches none of the eight
  `AgingEffect` kinds. D3's case exactly. *(B02)*
- **`virtue.dust_devil`** correctly carries no `SizeDelta`: ArMDE:3709's "Size +1"
  is the *assumed form's* Size. *(B02)*
- **The four botch-dice entries** are correctly `uncomputed_rule` with
  `description` in both locales — botch dice are modelled nowhere in
  `crates/arm-rules/src`. *(B01)*
- **`flaw.dwarf`'s** three-point wound bands are a *consequence* of Size -2, not a
  second rule; `derived/combat.rs::wound_ranges` already computes it exactly. The
  data encodes the cause, which is right. *(B12)*
- **Repeat limits without `parameters`** are already enforced: `max_per_target`'s
  default of 1, keyed on `(item_ref, {})`, forbids a second copy. An
  "only once" clause on an unparameterized entry needs no `max_total`. *(B11,
  proved for all eighteen of its entries)*
- **`virtue.student_of_realm`'s** German *filled* name and **`flaw.bound_to_realm`'s**
  German name are recorded decisions in `RULES.md`; both were raised as findings and
  both were withdrawn on that evidence (F-299, F-373). `flaw.realm_stigmatic`'s
  DE-only `name_unfilled` key is the same shape (`RULES.md:2368-2374`) and stopped
  a third attempt. *(B08, B10, B16)*
- **The `scheitest` typo** in the DE `description` of `virtue.cautious_sorcerer`
  and `virtue.cautious_with_ability` is in the German **source**, so the i18n is a
  faithful copy and correct as-is. Do not "correct" it away from its source. *(B01)*
- **`flaw.poor`** is clean, and it is clean *because* of Part C's **C3b** —
  `LaterLifeXpRate` is unreachable without a life-stage plan. *(B16, via B09)*
- **`virtue.merchant` / `virtue.merchant_adventurer` / `virtue.gentleman`** end with
  "The Wealthy Virtue and Poor Flaw affect you normally", which states that **no**
  special rule applies. `virtue.guild_apprentice` (F-100) says the negation of that
  sentence, which is why the negation *is* a rule. *(B02, B04)*

---

## 6. Open-questions index — Q-01 … Q-129

**Who settles it** uses four codes. **N** = Norbert's decision (a design, model or
policy call the source cannot settle). **R** = a rules read (the answer is in
`rules/source/en/`; an agent can settle it). **C** = a codebase read. **X** =
needs `arm-de-translation`, which is outside this repository.

**Six are closed.** Q-02 (D3), Q-35 (F-160), Q-85 (withdrawn, answered from the
repo), Q-120 (the enum's own doc comment), Q-129 (**D7**), and Q-03/Q-14/Q-22 as a
family (**D5** — see the note under the table).

| Q | Topic | Settled by | Status |
|---|---|---|---|
| Q-01 | the closed value set for `virtue.academic_concentration_subject`'s `subject` | R | open |
| Q-02 | is a prereq-only entry `creation_effect` or `narrative`? | — | **closed — D3, superseded by F-32** |
| Q-03 | does a computed entry owe a `description` for its uncomputed clauses? | — | **closed in substance — D5** |
| Q-04 | `virtue.alluring_to_beings` German name: table vs DE rulebook heading | N | open — **D7 supplies the test; apply it** |
| Q-05 | "only available to male characters" — the engine has no sex model (6+ entries) | N | open |
| Q-06 | `virtue.aptitude_for_sin`: should the +3 be an `ability_roll_mod`? | N | open |
| Q-07 | ArMDE:2816's one-Social-Status rule is modelled nowhere | N | open — **rated a defect by F-427** |
| Q-08 | the ArMDE:2960-2962 realm association is on 4 of ~115 Supernatural entries | N | open |
| Q-09 | `virtue.common_sense`: is a guaranteed storyguide intervention mechanical? | N | open |
| Q-10 | `virtue.covenfolk`: two canonical German sources, two names | X / N | open — **F-85 is blocked on it** |
| Q-11 | `virtue.domestic_animal`: "animals only" with no engine model | N | open |
| Q-12 | `ability.enchanting` parameterized? is `domain: "ability"` right? | N | open |
| Q-13 | does the book distinguish "may only **take**" from "may only **have**"? | R | open |
| Q-14 | where is the threshold at which an effect-bearing entry owes a `description`? | — | **closed in substance — D5** |
| Q-15 | `virtue.craftsman`: is "Wealthy/Poor affect you normally" a clause? | R | open — settled in practice (§ 5) |
| Q-16 | `virtue.dust_devil`: how much of Skinchanger does "this variant" inherit? | R | open |
| Q-17 | a supernatural power described only in prose, with constraints but no number | N | open — **same family as Q-64 / Q-76, which governs 48 entries** |
| Q-18 | `virtue.gentle_gift`: does the book state the `has virtue.the_gift` prereq? | R | open |
| Q-19 | may the catalogue assert an incompatibility no passage states, when the pair is logically contradictory? | N | open |
| Q-20 | `virtue.fidai`: does the pretended Social Status permit a second one? | R / N | open |
| Q-21 | `virtue.forge_companion`: is "unGifted craftsman" an encodable incompatibility? | N | open |
| Q-22 | where does an *uncomputable* rule go on an entry that already carries an `Effect`? | — | **closed in substance — D5** |
| Q-23 | does modelling a rule on the *type profile* make the Virtue a `creation_effect`? | N | open |
| Q-24 | `virtue.greater_benediction`: reclassify, or renarrow `source.lines`? | N | open — narrowed by F-121 |
| Q-25 | is "politically Criamon, created by another House's rules" a rule? | R / N | open |
| Q-26 | classification and encoding for a rule that *nullifies* another Virtue's effect | N | open |
| Q-27 | `restricted_ability_xp` authorizes permanently; ArMDE:4065 says it should not | N | open |
| Q-28 | `virtue.guardian_angel`: encode the +5 Soak despite its condition? | N | open |
| Q-29 | `greater_immunity` and `lesser_immunity` disagree about repeatability | R | open |
| Q-30 | how to model a "General **and** Hermetic" descriptor without blocking the unGifted case | N | open |
| Q-31 | three Supernatural Virtues require player-defined content and none records it | N | open — **Q-86 family** |
| Q-32 | `AdvancementSource` has `Teaching` but nothing for books-written | N | open |
| Q-33 | the book gives Hermetic Prestige two different levels and the data picks one | R | open |
| Q-34 | ArMDE:18432's Size-scaled Improved Characteristics is modelled nowhere and unreachable | N | open |
| Q-35 | `virtue.knows_people`: is a once-per-story entitlement mechanical? | — | **closed — F-160** |
| Q-36 | `virtue.lasiq`: does "some other social status, which you should choose" demand a parameter? | N | open — **Q-86 family** |
| Q-37 | `virtue.leper_magus`: `grants_selection` or a copied effect? | N | open |
| Q-38 | does Greater Immunity's repeatability transfer through Lesser Immunity's cross-reference? | R | open |
| Q-39 | `virtue.linguist`: the book rounds the XP **up**, the engine rounds the **cost** up — do they agree? | C | open |
| Q-40 | `virtue.linguist`: the table's only row is in a supplement section and gives "Linguist" | X | open |
| Q-41 | `virtue.lone_redcap`: is `supernatural` in the 300-point pool sourced? | R | open |
| Q-42 | `virtue.lone_redcap` / `virtue.redcap`: nothing encodes that they are alternatives | R / N | open |
| Q-43 | a granted Reputation has no **polarity**, so "a *poor* Reputation at level 2" is unrepresentable | N | open — **§ 3.11 is blocked on it** |
| Q-44 | no `Effect` expresses free seasons per year, though three entries modify them | N | open |
| Q-45 | ArMDE:2816 is not a cap, and one `virtue_category_caps` row cannot express it | N | open |
| Q-46 | the leather variant of the two vessel Virtues: entry, parameter, or nothing? | N | open |
| Q-47 | `restricted_ability_xp` cannot scope a pool to one Profession, and three entries name one | N | open |
| Q-48 | Mercurian Magic's companion Flaw: a prerequisite the player is paid for, or a budget-exempt grant? | N | open |
| Q-49 | the reputation table attributes to Mythic Blood a Reputation ArMDE:4588 denies | X | open — **D6 rule 1 decides it; the fix needs the source project** |
| Q-50 | `mythic_type.nephilim` carries neither of ArMDE:2731's two point adjustments | R / N | open |
| Q-51 | should Magical Blood's Magic Human variant become effects once a parameter exists? | N | open |
| Q-52 | is `virtue.masterpiece` `creation_effect` or `in_play_effect`? | — | **answered in substance by Q-120's resolution** |
| Q-53 | the book names six Ability types; `AbilityCategory` has five | R / N | open |
| Q-54 | does a glossary table govern *inside a verbatim quotation*? | N | open — **Q-63 family** |
| Q-55 | `virtue.physician_of_salerno`: is the granted Reputation Local or Academic? | R / N | open — **Q-73 family** |
| Q-56 | `virtue.perfectus`: should Purity and Transcendence enter the Ability catalogue? | N | open |
| Q-57 | Potent Magic: how should the forced magnitude-variant incompatibility be lifted? | C / N | open |
| Q-58 | `virtue.ripper`: are the two fixed powers a `power_levels` grant or pre-entered powers? | N | open |
| Q-59 | `virtue.privileged_upbringing`: can a pool-scoped permission be expressed at all? | N | open |
| Q-60 | `virtue.personal_vis_source`: is "about one tenth" a rule or hedged guidance? | N | open — **Q-83 family** |
| Q-61 | `virtue.rat_up_a_drainpipe`: is "a substantial advantage" mechanical? | N | open — **Q-64 family** |
| Q-62 | `virtue.powerful_relic`: is the relic's one power charged against the power-levels budget? | R | open |
| Q-63 | does a translation table's term bind *inside a sentence*, or only on a label? | N | open — **F-65, F-320, F-473 depend on it** |
| Q-64 | is a supernatural *capability* with no number, no roll and no waived penalty mechanical? | N | open — **Q-76 puts this family at 48 entries** |
| Q-65 | two canonical tables give `Sense Holiness and Unholiness` different German names | X | open — **D7's three-rule test applies** |
| Q-66 | does `virtue.sense_holiness_and_unholiness`'s "may overwhelm you" state a D5 rule? | N | open |
| Q-67 | what amount should `virtue.simple_student`'s restricted XP pool carry? | R / N | open |
| Q-68 | does Subtle Magic's "no benefits from normal gestures" add anything to the Words/Gestures table? | R | open |
| Q-69 | a table attributes a Reputation rule to `Social Contacts` that ArMDE:4990 does not state | X | open — **D6 rule 1 decides it** |
| Q-70 | should `virtue.sense_passions` carry `tainted: true`? | R | open |
| Q-71 | the German core rulebook gives `virtue.spirit_votary` two names and the tables give a third | X / N | open — **D7 rule 2: a colliding/absent heading is where the glossary wins** |
| Q-72 | what is the "Brother-Priest Status Virtue" that ArMDE:5111 names? | R | open — **if it is a fourth rank, F-34's census is not closed** |
| Q-73 | which `ReputationType` carries an **organization**-scoped Reputation? | N | open — **recurs on ≥4 entries; § 3.11 is blocked on it** |
| Q-74 | how does True Faith's Magic Resistance join the per-Form grid — replace, stack, or compete? | N | open — F-329's *fix* depends on it |
| Q-75 | does ArMDE:2394's "only companions can take this" bind grogs? | N | open — **F-339 depends on it** |
| Q-76 | is a roll-free supernatural *capability* `narrative` or `uncomputed_rule`? | N | open — **governs 48 entries; answer once** |
| Q-77 | how should a free-text `{land}` placeholder render in German, where the article inflects? | N | open |
| Q-78 | does `virtue.unaging`'s non-fatal-crisis clause have a home in the M6/6b7 crisis engine? | C | open |
| Q-79 | the book files two Flaws under a magnitude their own descriptor contradicts | R | open |
| Q-80 | is `flaw.black_sheep`'s "a bad Reputation **of your choice** at level 2" a wildcard? | N | open — **Q-73/Q-43 family** |
| Q-81 | which `casting_fatigue` sign convention is intended? | C / N | open — **F-351 depends on it** |
| Q-82 | may the German source be used to repair a truncated **English** sentence? | N | open — **F-370 depends on it** |
| Q-83 | is a hedged monetary value a mechanical clause for classification purposes? | N | open |
| Q-84 | what entity or type should hold `flaw.abandoned_apprentice`? | N | open |
| Q-85 | (the `flaw.bound_to_realm` German-name question) | — | **withdrawn — answered from the repository; see withdrawn F-373** |
| Q-86 | should every "choose the specifics" Flaw carry a parameter? | N | open — **one ruling, catalogue-wide; § 3.7 is blocked on it** |
| Q-87 | should `flaw.broken_vessel` carry an enumerated `prerequisites` tree? | N | open |
| Q-88 | should a player's free-text choice of *scope* be recorded by a parameter? | N | open — **the same ruling as Q-86; five entries named** |
| Q-89 | ArMDE:5911 says "Technique" inside *Deficient Form*'s own passage — misprint or not? | R | open |
| Q-90 | which of the two Deficient Art entries has the right shape? | N | open — **"one ruling with 122 consequences"** |
| Q-91 | does ArMDE:5895's third sentence change **D4**'s answer for Cyclic Magic? | N | open — **D4's own slice needs it** |
| Q-92 | how should a *mandatory earmark of the normal budget* be encoded? | N | open — both available shapes are wrong in opposite directions; the character cannot legally exist today |
| Q-93 | which treatment is intended for the three Corrupted entries? | N | open — **catalogue-wide; F-393 recommends the minimum change only** |
| Q-94 | `grants_selection` for `flaw.a_deal_with_the_devil`, and does a granted Story Flaw count toward the cap? | N | open |
| Q-95 | can a parameter express "one or more, your choice"? | N | open — **Q-86 family** |
| Q-96 | is the partial `source.anchor` coverage a backfill in progress or an inconsistency? | C / N | open |
| Q-97 | `RULES.md` declares a `source.lines` range wrong that the file does not have | C | open |
| Q-98 | how should `flaw.enfeebled`'s "double the normal number of Fatigue levels" be encoded? | N | open |
| Q-99 | does RoP:D:5671 oblige anything of `flaw.dhimmi`, and does a supplement bind a core entry? | N | open |
| Q-100 | should `SurfacedModifier` carry the id of the item that produced it? | N | open — **F-423** |
| Q-101 | how should "once per Power" be encoded, on the twin entries that state it? | N | open |
| Q-102 | is ArMDE:2816's total absence from the engine a deliberate deferral or a gap? | N | open — **F-427** |
| Q-103 | `flaw.false_power`'s `require_categories` admits any Hermetic or Special Virtue where the book says *Supernatural* | R / C | open |
| Q-104 | `flaw.flawed_powers` records no parameter for the Flaw it imports, where `flaw.false_power` does | N | open |
| Q-105 | is `flaw.form_monstrosity`'s "1 pawn of Muto vis" a mechanical clause? | N | open |
| Q-106 | what shape should F-428's correction take? | N | open — **F-428's fix depends on it** |
| Q-107 | should the two Maledictions also carry an *open grant*, on top of F-442's reclassification? | N | open |
| Q-108 | does `virtue.lone_redcap`'s 300 XP **replace** the funding for its fifteen apprentice years, or supplement it? | R | open — **F-439 is blocked on it; the answer decides `high` versus nil** |
| Q-109 | `flaw.fury` and `virtue.berserk` state the same condition, encoded two contradictory ways | R / C | open |
| Q-110 | ArMDE:6124's two +4 traits contradict ArMDE:2502's ±3 for a Minor Personality Flaw | R | open |
| Q-111 | does `flaw.independent_craftsman`'s recategorization clause apply, given *City and Guild* is not in the repo? | N | open — **provenance: a rule from a book with no source cannot be implemented** |
| Q-112 | does `virtue.lone_redcap` satisfy `flaw.hermetic_patron`'s "a Redcap"? | R / N | open — **F-448's fix depends on it** |
| Q-113 | should an `advancement_mod` marker of `amount: 0` render as a word? | N | open — UI |
| Q-114 | under D6 as corrected, does the glossary or the shipped data win on F-456 and F-457? | N | open — **D7's three-rule test now answers it; confirm** |
| Q-115 | is `flaw.inscribed_shadow`'s House Criamon restriction a prerequisite to encode? | R / N | open |
| Q-116 | the book defines "combat scores"; the repo's reading excludes three of the five | R / C | open — **overturned the clean verdicts on `flaw.hobbled` and `flaw.lame`** |
| Q-117 | two in-repo authorities give opposite readings to "the passage states the *absence* of a rule" | C / N | open — `NO_RULE_DESPITE_TOKEN` versus `RULES.md:4684` |
| Q-118 | is `flaw.monastic_vows_hermetic`'s "you cannot own vis" mechanical, given the engine models no vis? | N | open |
| Q-119 | is `flaw.necessary_condition`'s "you cannot cast spells at all" a mechanical absolute or a fiction condition? | N | open |
| Q-120 | is an entry computing in both phases `creation_effect` or `in_play_effect`? | — | **closed — the enum's own doc comment: `creation_effect`** |
| Q-121 | four terminology-table rows attribute core-book Flaws to supplements — at what count does "noted" become a finding? | N | open — **F-501 raises the same rows; D6 rule 1 names "from which book"** |
| Q-122 | should `flaw.magical_fascination`'s authorization name both Lores, or record the player's choice? | N | open — **Q-86 family** |
| Q-123 | two entries state a *soft* restriction on magi, and the engine has only hard blocks | N | open |
| Q-124 | this span carries both `source.lines` conventions | C / N | open — **Q-96 family** |
| Q-125 | the rulebook contradicts itself about `flaw.prohibition`'s category | R | open |
| Q-126 | is an entry whose only mechanical content is a *selection restriction* mechanical? | N | open — **Q-64 family** |
| Q-127 | a granted Reputation's `score` cannot hold the range ArMDE:6554 gives | N | open — **§ 3.11; Q-43/Q-73 family** |
| Q-128 | should `flaw.plagued_by_supernatural_entity` carry an `entity` parameter? | N | open — **Q-86 family** |
| Q-129 | which German name is canonical for Primogeniture Lineage and Palsied Hands? | — | **closed — D7 (2026-09-21): the DE rulebook heading wins; all three shipped names are correct** |

### 6.1 The Phase-0 questions

`README.md:138-147` lists three Phase-0 questions as unread. Two were read and
answered inside batches and the README has not been updated: **Major vs Minor
Magical Focus arithmetic** (B06, *"The two directed reads — both answered"*) and
**whether a granted Reputation's `score` is a ceiling, a fixed value or a
suggestion** (B03, *"Directed read — README's open question 3"*). **D2 remains
genuinely pending** — see § 2.2.

### 6.2 The ones that need Norbert, ordered by how much they unblock

He works from this list, so it is ordered by consequence rather than by number.

1. **Q-76 / Q-64 / Q-17 / Q-61 / Q-126** — is a roll-free supernatural *capability*
   mechanical? **48 entries.** One answer settles the largest classification
   family in the audit.
2. **Q-86 / Q-88 / Q-31 / Q-36 / Q-95 / Q-122 / Q-128** — must every "choose the
   specifics" entry carry a parameter? Catalogue-wide, and § 3.7 cannot start
   without it. Several of these are **data loss** under CLAUDE.md.
3. **D2** (not a Q, but the same queue) — granted Virtues and the bought-only
   validators. Blocks § 3.6.
4. **Q-43 / Q-73 / Q-80 / Q-127** — Reputation polarity, organization scope,
   wildcards and ranges. § 3.11 cannot start without them.
5. **Q-90** — one ruling with **122 consequences** (the Hermetic-category prereq).
6. **Q-92 / Q-93** — the mandatory-earmark encoding and the three Corrupted
   entries. Q-92's character *cannot legally exist today*.
7. **Q-05 / Q-123** — sex restrictions and soft restrictions: does the engine grow
   a model, or do they stay text forever?
8. **Q-108** — the only question a *finding* is outright blocked on (F-439), and
   it is answerable from the source (**R**), so it is cheap.
9. **Q-10 / Q-40 / Q-49 / Q-65 / Q-69 / Q-71 / Q-114 / Q-121** — all need
   `arm-de-translation`, which no agent in this repository can open.

---

## 7. Counts

### 7.1 Method — read this before quoting a number

Three earlier agents in this audit shipped wrong census figures, one exactly
backwards, so the method is stated rather than assumed.

Every number below is derived **mechanically from § 1 of this file**, not
recounted by eye, using the section's own line range (105-694) and its column
positions:

```
sed -n '105,694p' corrections.md | grep "^| F-" | cut -d'|' -f5 | ...
```

That restriction matters: five later sections contain tables whose rows also begin
`| F-`, and counting the whole file returns **521** instead of 500.

**Three cross-checks agree.**

1. `grep -c "^### F-"` over `batch-01.md` … `batch-16.md` returns **500** finding
   headings. Per batch: 33, 31, 24, 32, 39, 48, 49, 51, 40, 35, 24, 20, 17, 15,
   21, 21 — which sums to 500.
2. § 1 of this file has exactly **500** rows.
3. Finding numbers run F-001 … F-501 with **one gap, F-44**, which
   `batch-02.md:4` and `:534` both record as never issued. 501 − 1 = 500.

**Every kind and severity tally sums to 500**, and each is shown summing below.
Where a figure is approximate, it says so.

### 7.2 Findings by status

| Status | Count |
|---|---|
| `live` | **482** (includes F-482, which is live but partly duplicates F-11 and F-100) |
| `withdrawn` | **13** — 3 by their own batch (F-299, F-373, F-481), 9 as duplicates found by **this index** (§ 4.3), 1 settled by D7 (F-496's name question) |
| `blocked` | **4** — F-85 (Q-10), F-270 and F-283 (a catalogue decision), F-439 (Q-108) |
| closed by this file | **1** — F-499 |
| **Total** | **500** |

**D2 blocks no individual finding**, but it blocks part of § 3.6 and is the
audit's one genuinely pending ruling.

### 7.3 Findings by kind of defect — **primary kind**

Sums to 500.

| Kind | Count |
|---|---|
| `class` — misclassification | **143** |
| `desc` — missing description (D5) | **110** |
| `auth` — missing authorization | **38** |
| `text` — shipped text / localization | **34** |
| `prereq` — missing prerequisite | **30** |
| `param` — missing/wrong parameter | **27** |
| `incompat` — missing incompatibility | **25** |
| `scope` — wrong scope | **24** |
| `effect` — missing effect | **19** |
| `table` — translation-table defect | **13** |
| `rep` — Reputation defect | **9** |
| `number` — wrong number/arithmetic | **7** |
| `engine` — engine gap | **7** |
| `src` — `source.lines` / anchor | **5** |
| `prov` — provenance | **5** |
| `mag` — magnitude/category/kind | **2** |
| `audit` — audit-record defect | **2** |
| **Total** | **500** |

### 7.4 Findings by kind — **primary *or* secondary**

A finding spanning two kinds is counted under both, so this does **not** sum to
500. It is the better guide to slice size: § 3.2 is a 66-entry slice, not a
38-entry one.

| Kind | Count | vs primary |
|---|---|---|
| `class` | 144 | +1 |
| `desc` | 136 | +26 |
| `auth` | **66** | +28 |
| `text` | 36 | +2 |
| `param` | 35 | +8 |
| `incompat` | 35 | +10 |
| `prereq` | 34 | +4 |
| `effect` | 28 | +9 |
| `scope` | 26 | +2 |
| `engine` | 19 | +12 |
| `number` | 14 | +7 |
| `table` | 14 | +1 |
| `rep` | 11 | +2 |
| `prov` | 11 | +6 |
| `src` | 6 | +1 |
| `audit` | 4 | +2 |
| `mag` | 2 | ±0 |

**The headline.** Three kinds are **77 %** of the audit by primary kind —
`class` (143), `desc` (110) and `auth` (38) make 291 of 500. Under D5 the first
two are one job per entry, and § 2.1 says neither can start until
`MECHANICAL_PHRASES` grows. **The screen fix is the critical path for more than
half the correction list.**

### 7.5 Findings by severity

Normalized per § 0.2; for a compound severity I took the **first-named**
component, which is what the batches lead with. **These are derived by me from two
different batch vocabularies and are not quoted figures.**

| Severity | All 500 | Live only (482) |
|---|---|---|
| `H` | **190** | **183** |
| `M` | **259** | **253** |
| `L` | **26** | **26** |
| `n/s` — the batch stated none | **21** | **20** |
| — (withdrawn, no severity) | **3** | 0 |
| **Total** | **500** | **482** |

`n/s` is concentrated in **B11 (16 of 24)** and **B12 (3 of 20)**, which close a
finding with a `**Verdict:**` line instead. It is not a gap in this index.

### 7.6 Other counts

| Thing | Count |
|---|---|
| Batches indexed | **16** of 19 (B17-B19, 95 entries, ArMDE:6663-7119, not run) |
| Catalogue entries checked | **560** of 655 |
| Findings | **500** (F-001…F-501, no F-44) |
| Open questions | **129** (Q-01…Q-129) |
| — closed / withdrawn | **9** (Q-02, Q-03, Q-14, Q-22, Q-35, Q-52, Q-85, Q-120, Q-129) |
| — open | **120** |
| — needing **Norbert** (`N`, `R/N`, `C/N`, `X/N`) | **92** |
| — settleable by an agent (`R`, `C`, `R/C`) | **24** |
| — needing `arm-de-translation` (`X`, `X/N`) | **6** rows, 8 questions when the `N`-tagged DE ones are included |
| Rulings in `decisions.md` | **7** (D1, D3, D4, D5, D6, D7 taken; **D2 pending**) |
| Duplicate relationships found | **10** (9 new here, 1 already withdrawn by B16) |
| Site-level overlaps inside an umbrella | **5 clusters** (§ 4.4) |
| `MECHANICAL_PHRASES` additions | **14 idiom families, ~94 word-forms** across both locales, **plus 2 structural fixes** that are not vocabulary |

**The `MECHANICAL_PHRASES` figure is approximate on the word-form count** — the
families are the actionable unit and some rows give a stem rather than a literal.
14 families and 2 structural fixes are exact.

### 7.7 What this file does *not* count

- **No count of affected catalogue entries.** Several findings cover two or more
  entries, several entries carry five findings, and the umbrella findings (F-340,
  F-409) name sites that are not enumerated anywhere. Deriving an entry count from
  this table would be a guess.
- **No estimate of Phase 2 effort.** The groups in § 3 are sized by finding count,
  which is not the same thing.
- **No re-rating.** Where a batch's severity looks wrong to me I left it and said
  so in § 0.2 rather than adjusting it silently.
