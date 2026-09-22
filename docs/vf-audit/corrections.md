# Accumulated correction list — V/F audit

**Generated 2026-09-21** from `docs/vf-audit/batch-01.md` … `batch-16.md`, plus
`decisions.md` (D1–D7) and `README.md`. **Extended the same day** to
`batch-17.md`, `batch-18.md` and `batch-19.md` and revised throughout against
`decisions.md` **D8–D18**.

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

**Coverage — THE CHECKING PASS IS COMPLETE.** Findings **F-001 … F-545** (544 of
them; F-44 was never issued — see the note below), open questions
**Q-01 … Q-142**, batches **B01–B19**, **all 655 catalogue entries**
(ArMDE:3362-7119). Every Virtue and Flaw in `rules/core/virtues_flaws.json` has
now been through a batch, confirmed two ways by B19's closure check:
`jq 'length'` returns **655**, and `grep -c '"entity_kinds"'` over the same file
independently returns **655**, with `entity_kinds` appearing exactly once per
entry.

**"Complete" means every entry was *read*, not that every entry is settled.**
**Five entries are explicitly NOT marked checked**, each held open by an
escalation rather than by an oversight: `flaw.seeker` (Q-135),
`flaw.susceptibility_to_divine_power` (Q-136), `flaw.weak_parens` (Q-140),
`flaw.weak_personality` (Q-141) and `flaw.waster_of_vis` (Q-142, on its text
only). They are listed here so "655 of 655" is not read as "655 clean".

**F-44 does not exist.** `batch-02.md:534` records the number as reserved for a
finding that was withdrawn before the batch was written. It is not a gap in this
index and needs no lookup.

**One gap in this file's own assurance, stated rather than papered over.** The
agent that built the first sixteen batches' rows was killed by a session rate
limit after writing the last section but **before running its verification
pass**, so no second reader has re-derived rows F-001…F-501. The orchestrator
independently confirmed the load-bearing figures — 500 finding headings across
the sixteen batch files by three separate counts, 500 rows in § 1, the F-44 gap,
and one duplicate determination (F-479 ≡ F-193) spot-checked against both batch
files — but the per-row kind, severity and status assignments in that range carry
**one** reader's judgement, not two. § 4.2 is likewise honest that its nine
duplicates are a **floor, not a ceiling** *for that range*. Treat a single row as
a pointer to be confirmed at the batch file, which is what § 0 says anyway; treat
the counts and the groups as sound.

**What the 2026-09-21 extension did and did not verify.** Rows F-502…F-545 were
written from the three batch files directly, and every figure in § 7 was
recounted from scratch and **cross-checked with a second, different command** —
which is how the `n/s` compound row and the status-filter trap in § 7.1 were
caught. The duplicate check was **extended with a body-level pass in both
directions** across the B17-B19 boundary (§ 4.2, passes 5-7) and found **zero**
duplicates. What it did **not** do is re-run that body-level pass over
F-001…F-501 against each other; the original floor disclaimer still stands there.

**Ten rulings landed after this file was first written, and several of them
change work recorded here rather than merely adding to it.** § 2.2 (D2 is ruled),
§ 3.7 (D9 unblocks, **D10 reshapes**), § 3.11 (D11 unblocks and narrows), § 3.2
(**D14 lands first**) and the new § 3.17 (the three XP modes) are the sections a
Phase 2 slice would otherwise work from a stale reading. **If you read an earlier
copy of this file, re-read those five.**

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

The nineteen batches used **two different severity vocabularies**, and neither is
a scale this file can count directly:

- **B01–B09** rate by **defect class**: `wrong rules output`, `lost-rule`,
  `lost-rule / provenance`, `data loss` / `lost player choice`, `localization
  defect` / `text`, `provenance`, `consistency`.
- **B10–B19** rate on a **three-point scale**: `high` / `medium` (or `moderate`)
  / `low`, with `medium-high` and `low-medium` used. B17–B19 state a severity on
  **every** finding, so none of their 44 rows is `n/s`.

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

## 1. Master index — F-001 … F-545

Sorted by finding number, 544 rows. `ArMDE` is the entry's `source.lines` range as
the batch's verdict table gives it; `—` where the finding is not about one entry.
**Note that F-540's range is the *wrong* one the data ships** (`7106-7119`),
because that is what the verdict table gives — the finding is that it should be
`7106-7109`.

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
| F-16 | `virtue.rard` | 3476-3479 | text+prov | M | live | the English `name` is the scanno "Rard"; the book's index and the table both read "Bard" — the **id** is wrong too. **Do NOT change `source.anchor`**: it stays `rard` because the *heading* stays `#### Rard`, and Guard A compares the anchor against the heading. Name and id track the truth; the anchor tracks the book. Independently rediscovered by the 2026-09-21 anchor backfill, which also found the entry cross-references `Educated (Bardic)` — one of § 3.2a's four missing Virtues |
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
| F-439 | `virtue.lone_redcap` | 4319-4326 | number | H | live | the same XP double-count as F-428, three batches back. **Unblocked by D17 (2026-09-21): the 300 points REPLACE**, so a 25-year-old Lone Redcap is over-funded by ~225 XP. Remedy is § 3.17's third mode, not a patch here |
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

### B17 — `batch-17.md` (ArMDE:6663-6802)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-502 | `flaw.rector` | 6671-6674 | class+prereq | M | live | `narrative` on a stated Social Status requirement; no `Prereq` variant ranges over a `categories` value (**Q-132**), so the gate reaches the player in neither locale |
| F-503 | `flaw.regular` | 6675-6678 | class+desc | M | live | `narrative` on a compulsory seasonal expenditure; the Poor-Regular consequence reaches the player nowhere |
| F-504 | `flaw.restricted_learning` | 6683-6686 | class+param | H | live | `narrative`, no parameter for the five chosen Abilities (**data loss**), no description; the app lets the whole XP pool go anywhere |
| F-505 | `flaw.restricted_power` | 6687-6690 | class+desc | M | live | `narrative` on a stated restriction on when a power functions; the `power` parameter is correct and the absent `max_total` is correct |
| F-506 | `flaw.restricted_power` | 6687-6690 | text | M | live | the German name follows a `SdM:M`-tagged glossary row against the core book's own DE heading — **D7 rule 3**; needs no source-project determination |
| F-507 | `flaw.restriction` | 6691-6694 | class+desc | M | live | `narrative` on an absolute about casting plus an enchanted-item extension |
| F-508 | `flaw.rigid_magic` | 6695-6698 | class | H | live | "cannot cast Ritual magic" as `narrative`, on an engine that already knows which spells are Rituals (**Q-130**) |
| F-509 | `flaw.rigid_magic` | 6695-6698 | text | L | live | the German summary alters a word of its own source line |
| F-510 | `flaw.savantism` | 6703-6708 | class+number | H | live | `creation_effect` with **no effects**: a Savant is funded at full XP instead of half and may start an Ability at the age cap. **Point 4 routes to § 3.9, points 1-3 to § 3.3/3.4** |
| F-511 | `flaw.sheltered_upbringing` | 6721-6724 | class+engine | H | live | seven named Abilities forbidden at creation and the app sells all seven; an **id-list** negative authorization, a distinct sub-shape from F-355/F-410's category one |
| F-512 | `flaw.short_lived_magic` | 6729-6732 | class+desc | M | live | `narrative` on an explicit Duration downgrade chain |
| F-513 | `flaw.short_of_breath` | 6733-6736 | desc | M | live | the Concentration clause reaches the user in neither locale (D5) |
| F-514 | `uncomputed_clauses.rs` | — | engine+audit | M | live | **screen gap:** `SIGN_CHARS` cannot see U+2014, so a signed number in this span is invisible to the detector |
| F-515 | `flaw.short_ranged_magic` | 6737-6740 | desc | H | live | two halvings, neither computed, described in neither locale; the Lab-Total half is **Q-133** |
| F-516 | `flaw.slow_caster` | 6755-6758 | class+desc | M | live | `narrative` on explicit casting-round counts |
| F-517 | `flaw.slow_power` | 6759-6762 | class+desc | M | live | `narrative` on an extra preparation round; the repeat rule is correctly carried by the default `max_per_target` |
| F-518 | `flaw.spontaneous_casting_tools` | 6779-6782 | prereq+class | H | live | "can only be taken by Verditius magi" gates nothing, so every magus of the other twelve Houses and every Gifted companion may take it and bank the Flaw point |
| F-519 | `flaw.stockade_parma_magica` | 6787-6790 | class+prereq | M | live | `narrative` on a Magic Resistance rule; the `IsMagus` presupposition is raised as an observation, not asserted |
| F-520 | `flaw.stuck_in_your_ways` | 6791-6794 | class+prov | M | **blocked** (Q-131) | `narrative` on a `min()` formula naming "Covenant Lore", which is in no catalogue id; the description cannot be written until Q-131 resolves |
| F-521 | `flaw.study_requirement` | 6795-6798 | class+desc | M | live | `narrative` on a study prohibition; the Study Bonus compatibility is correctly *not* encoded |
| F-522 | `flaw.servant_of_the_land` | 6717-6720 | param | H | live | parameterized with no `max_total`, so the same **Major** Flaw banks without limit. Found by B17's verification pass; **D10's motivating case** |

### B18 — `batch-18.md` (ArMDE:6803-6988)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-523 | `flaw.susceptibility_to_faerie_power`, `flaw.susceptibility_to_infernal_power` | 6819-6826 | class+desc | M | live | **D3 names these two entries by id** and the data still ships a class D3 rules out |
| F-524 | `flaw.unspecialized` | 6943-6946 | engine+class | H | live | the sheet prints a **+1 specialty bonus** on a character whose Flaw forbids specialties; no validator, no effect and no UI branch consults the Flaw |
| F-525 | `flaw.tainted_with_evil` | 6843-6846 | class+incompat | M | live | "Gaining a positive Reputation is impossible" is `narrative` and enforced nowhere. Enforcement was blocked on Q-43; **D11 answers Q-43 no**, so the class+description half is the whole remedy |
| F-526 | `flaw.university_dean` | 6923-6926 | prereq+incompat | M | live | three eligibility clauses, none encoded; the `virtue.doctor_in_faculty` prerequisite is unblocked, the seventeen-way Reputation exclusion is **Q-137** |
| F-527 | `flaw.uncertain_faith` ↔ `virtue.true_faith` | 6899-6906 | incompat | M | live | the True Faith exclusion is in the shipped text and in neither entry's `incompatible_with` |
| F-528 | `flaw.susceptibility_to_warping` | 6831-6838 | class+desc | M | live | a fully quantified Warping rule shipped `narrative` |
| F-529 | `flaw.unstructured_caster` | 6947-6950 | class+desc | M | live | a Ritual-spell prohibition shipped `narrative`; **Q-138** records the reciprocal rule at ArMDE:6148, which is B13's entry and unrated |
| F-530 | `flaw.unnatural_magic` | 6931-6934 | class+desc | M | live | two absolutes about Creo shipped `narrative` |
| F-531 | `flaw.suppressed_gift` | 6803-6810 | class+desc | M | live | four mechanical clauses shipped `narrative`; prereq, magnitude and categories are all correct |
| F-532 | `flaw.tormenting_master` | 6851-6854 | prereq | M | live | "only applicable to magi" with `prerequisites: null`; `Prereq::IsMagus` expresses it exactly |
| F-533 | `flaw.vendetta` | 6955-6958 | prereq | M | live | the magus half is unhedged and encodable now; whether `House("house.verditius")` joins it is **Q-139** — and **D16** now supplies the general test |
| F-534 | `flaw.usurer` | 6951-6954 | desc | M | live | the income clause reaches neither locale (D5) |
| F-535 | `flaw.tragic_life` | 6855-6870 | class | L | live | a mandated sinful Personality Trait; marginal, and rated as such by its own batch |
| F-536 | `flaw.surgical_empiricus` | 6811-6814 | desc+prov | M | live | the Western Christendom culture restriction reaches neither locale |
| F-537 | `uncomputed_clauses.rs` | — | engine+audit | L | live | **screen gap:** a German needle already in `MECHANICAL_PHRASES` matches nothing the German book writes, so it screens nothing |
| F-538 | `flaw.vengeful_powers` | 6959-6976 | param+mag | M | live | "may be taken as a Hermetic Flaw" is unrepresentable though `taken_as` exists; the proposed `virtue.sufi` copy is **under-specified** — `validate_house` would credit it unconditionally |

### B19 — `batch-19.md` (ArMDE:6989-7119)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-539 | `flaw.warped_by_magic` | 7019-7022 | auth | H | live | the Magic Lore XP permission is encoded nowhere and `ability.magic_lore` is `arcane`, so the app refuses to build the character. An instance of § 3.2 / F-409 |
| F-540 | `flaw.wrathful_major` / `_minor` | 7106-7119 | src | M | live | `source.lines` ends 11 lines past the Flaw, inside **Chapter 5**; correct range `[7106, 7109]`. The only two entries in all 655 citing past `SWEPT_BLOCKS`' 7113 — see § 2.1a |
| F-541 | `flaw.vulnerable_magic` | 7005-7010 | class+param | H | live | `narrative` on a duration-termination, a repeat and an incompatibility rule, **and** `max_per_target: 255` with no `parameters`, which licenses 255 *identical* copies — the exact inverse of ArMDE:7009 |
| F-542 | `flaw.weak_personality` | 7076-7079 | incompat+engine | H | live | "may have no other Personality Flaws" is enforced nowhere and four legal pairs sit in this same span; the **Virtue/Flaw-axis twin of F-355** |
| F-543 | `flaw.blind`, `flaw.deaf`, `virtue.keen_vision`, `virtue.sharp_ears` | 5719-5722, 5901-5904, 4187-4190, 4950-4953 | desc | L | live | Warped Senses' **8 hard** prohibitions (2 further pairs are soft advice and must not be encoded) are legible from `flaw.warped_senses` alone; its four partners carry no hint |
| F-544 | 7 entries: `flaw.vulnerable_casting`, `flaw.vulnerable_magic`, `flaw.warped_by_magic`, `flaw.weak_enchanter`, `flaw.weak_magic`, `flaw.weak_scholar`, `flaw.weak_spontaneous_magic` | 6993-7089 | desc | M | live | one D5 omission shape applied seven times; all seven ship **no `description` in either locale**. Two of the clauses are *ordering* rules, where order changes the arithmetic |
| F-545 | `flaw.wanderlust` | 7015-7018 | class | M | live | a countable seasonal constraint (one place per season; two, nonconsecutive, per year) as flavour. Verdict **overturned by B19's verification pass** — "the app models no seasons" is not a ground D3 admits |

---

## 2. Ordering constraints

These are hard. A slice that ignores them produces a red it cannot fix without
rewriting rulebook text — or builds a new mechanism twice in two incompatible
spellings. **§ 2.1, § 2.1a and § 2.1b are one chain and must be worked in that
order**; § 2.5, § 2.6 and § 2.7 were created by the 2026-09-21 rulings and did
not exist when this file was first written.

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

### 2.1a Fix `flaw.wrathful_*`'s `source.lines` BEFORE widening `SWEPT_BLOCKS` — B19

Added 2026-09-21. `SWEPT_BLOCKS`' Flaws entry is `(ArMDE, 5639, 7113)` and its
comment claims it runs "to the end of `#### Wrathful`". It does not, and
`is_swept` (`uncomputed_clauses.rs:476-480`) requires the entry's range to lie
**wholly** inside the block:

```rust
start >= *lo && end <= *hi
```

`flaw.wrathful_major` and `flaw.wrathful_minor` both carry
`source.lines: [7106, 7119]`, and `7119 > 7113`, so **both are silently excluded
from the narrative screen today** — a guard hole, not untidiness. They are the
only two entries in all 655 that cite past 7113 (`jq '[.[]|.source.lines[1]]|max'`
= 7119, carried by exactly those two).

**The obvious fix is the wrong order.** Widening the block to 7119 first would
pull both entries in — and the screen would then read lines 7106-7119 as
Wrathful's text. But the Flaw ends at ArMDE:7108; `---` follows at 7110, a pull
quote at 7112, and **`# Chapter 5: Abilities` at 7114**, whose :7118 reads
"meeting or exceeding an **Ease Factor**". `"ease factor"` is literal #7 in
`MECHANICAL_PHRASES` (`:194`). So the screen would flag two `narrative` entries
as dropping a mechanical clause **that belongs to a different chapter** — a false
offender, and exactly the kind of guard-driven confusion the audit exists to end.

**Order: fix F-540's range to `[7106, 7109]` first, then widen the block.** Doing
so brings the entire 655-entry catalogue inside the swept block for the first
time.

### 2.1b The screen needs a **capability** family before any of D8's 48 entries can land

Added 2026-09-21, from **D8**'s own closing obligation. D8 reclassifies the **48**
`supernatural` + `narrative` entries to `uncomputed_rule`. Those entries carry
**no signed number and no botch term**, so they depend entirely on
`MECHANICAL_PHRASES` to satisfy
`every_uncomputed_rule_entry_states_its_rule_in_every_locale` — and the list
today has **no capability idiom in either language**. D8 states this as a third
requirement on the § 2.1 sequence, not as a separate task:

> "the screen needs a capability family before any of the 48 can land."

So the chain is **§ 2.1 (grow the list, including the capability family) → § 2.1a
(fix F-540's range, then widen `SWEPT_BLOCKS`) → the reclassifications**, D8's 48
included. The word-forms this family needs are **not yet enumerated anywhere** —
§ 3.1's fourteen families were derived from B10-B19's *findings*, none of which
is one of D8's 48. Deriving them is the first task of the § 3.1 slice.

### 2.2 D2 is **ruled** — both validators become grant-aware

**Superseded 2026-09-21.** `decisions.md` D2 was pending when this file was
written; it has since been taken. The reading behind it is recorded there in
full: ArMDE:3989's "*already has a score of at least +3*" describes the
Characteristic's **state**, not the act of purchase, and the take-versus-have
split that prompted the question is a red herring (ArMDE:3665 and :3669 state one
restriction with two verbs).

**The ruling.** `validate_ability_bonus_targets` and
`validate_characteristic_delta_preconditions` must read **effective** selections
— bought *and* granted — rather than bought only.

**The trap, which is the part a slice will get wrong.** The change must **not**
re-import what review finding B1 deliberately excluded.
`validation/prereq.rs::PrereqCtx::build` records that grants "must never reach"
the incompatibility and trait checks, and that exemption is *correct*: those ask
*may this character hold this Virtue at all*, and a House grant is precisely what
makes him eligible. **These two validators ask a different question** — *is the
thing this Virtue modifies in a legal state*. D2 requires the distinction be
written as a comment at both sites, because the next reader will otherwise see
two validators disagreeing with B1 and "fix" one of them.

**Reachability, so the slice is sized honestly.** The gap is **latent, not
live**: exactly three entries carry the governed effects
(`virtue.great_characteristic`, `virtue.puissant_ability`,
`flaw.poor_characteristic`) and **nothing in `rules/core/` grants any of them**.
It matters because CLAUDE.md makes the catalogue a data-only extension point, so
a future House that grants Puissant Ability is a JSON edit and nothing would
fail.

### 2.3 Other blocked or ruling-dependent rows

| Finding | Blocked on | Note |
|---|---|---|
| F-439 | ~~Q-108~~ → **D17** | **Unblocked and confirmed at `high`.** D17 reads ArMDE:4848 against :4321 and rules the 300 points **replace** the apprenticeship block. A 25-year-old Lone Redcap is over-funded by ~225 XP. The remedy is **D13's third mode** and waits for it. |
| F-520 | **Q-131** | the identity of "Covenant Lore"; the `description` cannot be written until it resolves. **R** — an agent can settle it. |
| F-85 | **Q-10** | the German name for `virtue.fabric_ripper`; Q-10 is the Covenfolk two-sources question. |
| F-270, F-283 | a catalogue decision | the required Virtue does not exist; the fix is either to add the entry (B02's F-34 shape) or to drop the requirement. |
| F-496 | **settled by D7** | D7 (2026-09-21) rules the DE rulebook heading wins. The name change is **cancelled**; the table-annotation obligation remains. |
| F-444 | already applied | `decisions.md` D6 now carries the corrected two-rule precedence. Verify rather than re-fix. |
| F-525 | ~~Q-43~~ → **D11** | D11 answers Q-43 **no**: a Reputation has three components and polarity is not one. The "no positive Reputation" absolute is therefore **not enforceable even in principle**, and the whole remedy is the reclassification + `description`. |
| F-526, F-538 | **Q-137**, remedy under-specified | F-526's `virtue.doctor_in_faculty` prerequisite lands regardless; only the seventeen-way exclusion waits. F-538's `virtue.sufi` copy is *not* sufficient — `validate_house` ignores `taken_as` by design. |
| all `table` rows | **D6 + D18** | still not Phase 2 work — but **D18 changes the test**. A table row disagreeing with the shipped heading now has *three* possible causes, not two: a table error, a stale copy, **or a correction from a newer German edition the repo has not received**. 20 of the 21 known disagreements must **not** be auto-reverted to the heading. |

### 2.4 Within Phase 2, work order that avoids rework

1. `MECHANICAL_PHRASES` (§ 3.1) — unblocks § 3.3, and per § 2.1b must include a
   **capability** family before D8's 48 can land.
2. The authorization family (§ 3.2) — it is the largest group, it is mostly
   mechanical, and several classification findings resolve to `creation_effect`
   *because* the authorization exists, so doing it first removes them from § 3.3.
   **D14 lands first within this group** — see § 2.5.
3. Classifications + descriptions (§ 3.3, § 3.4) together — D5 makes them one job
   per entry, and doing them separately means opening the same passage twice.
4. Everything else.

### 2.5 **D14 before D13** — an ability reference must be able to name its parameter

`decisions.md` D14 carries this as a *"Hard sequencing constraint"* in its own
words, and it reaches further than D13.

**The rule.** An ability reference in an effect gains an optional parameter
constraint — a **literal** (`ability.dead_language` + `language = latin`) and a
**binding** to the selecting entry's own parameter (Student of (Magic) authorizes
Magic Lore alone) — **before** § 3.2 writes its ~30 new authorizations.

**Two reasons, and the second is the one that forces the order.**

1. **Volume, which argues the other way and is overruled.** Exactly *one* gated
   parameterized Ability exists today (`ability.dead_language`), so the defect
   has one carrier per shape. **The reason to do it now is timing, not volume**:
   writing ~30 authorizations against a shape known to over-permit and
   retrofitting later is the expensive order.
2. **D13's earmark names Abilities, so it inherits the defect on a brand-new
   effect.** Two of Church Upbringing's five (ArMDE:5791) are parameterized —
   `ability.dead_language` (the book means **Latin**) and
   `ability.organization_lore` (the book means **Church**). Written against the
   id-only shape, the earmark would let the 25 points go to any dead language and
   any organization's Lore, so *the ruling that exists to model a passage exactly
   would ship an approximation on day one*.

**Also part of D14:** `RULES.md:6362` is **rewritten, not amended** — it
currently blesses the over-permission as a documented approximation and cites
`ability_score_grant` as precedent for the same looseness. Both halves stop being
true.

### 2.6 **D10 before D9's data work** — invert the default, then declare the repeats

D9 originally required every *added* parameter to carry `max_total: 1`. **D10
supersedes that clause and must land first**, because the two do not compose in
the other order: D9's version is author discipline, and forgetting it once is
exactly how F-522 (`flaw.servant_of_the_land`, a **Major** Flaw banked without
limit) and F-541 (`flaw.vulnerable_magic`, 255 identical copies) happened.

**Order.**

1. **Invert `default_max_total` and `default_max_per_value` to 1.** Test-first,
   and the reds are the point: every entry that legitimately repeats goes red
   until it declares so.
2. **Declare the ~42 entries that legitimately repeat** (`jq`: `parameters !=
   null and max_total == null`), each read against its own passage.
3. **Then** D9 part 1's data work — adding the parameter every "choose the
   specifics" passage demands — with the default already safe.

**Do not add `max_total: 1` entry by entry under D9.** D10 says so explicitly,
and it reaches the 42 entries already riding the unlimited default, which D9's
clause did not.

**Expect a large mechanical diff**: `is_default_max_total`'s serialization skip
flips meaning, so canonical JSON changes for every entry that gains an explicit
value. Check it is exactly the intended set. **No `SCHEMA_VERSION` bump** — only
D9 part 3's multi-valued type needs one, and the two must not be bundled.

### 2.7 The three XP modes are designed **together** — D13 + D17 + F-428

**This is the ordering constraint only; the work itself is § 3.17, which is
authoritative for the shapes.** D13 names three modes and the engine has had only
one. Designing them separately invites a third incompatible spelling, and D13's
closing note and D17's remedy both say so in terms.

| Mode | Carriers | Status |
|---|---|---|
| **Additive grant** — new points on top of the budget | Educated, Warrior, Privileged (12 existing `RestrictedAbilityXp` carriers) | modelled |
| **Earmark** — N of the *existing* budget, constrained | `flaw.church_upbringing`, ArMDE:5791 — the phrase "from the normal budget" occurs **exactly once in the rulebook** | **D13**, unbuilt |
| **Replacement** — substitutes a life-stage block | **childhood:** `flaw.feral_upbringing` (F-428). **apprenticeship:** `virtue.lone_redcap` (F-439) and the unencoded `virtue.redcap` (D17) | unmodelled |

**The replacement mode has two carriers pointing in opposite directions**, which
is D17's second defect and is easy to miss: the **Major** `virtue.redcap` encodes
its 300 apprenticeship points **not at all** (it carries only
`item_level_budget: 50`) and is therefore *under*-funded by a whole block, while
the **Minor** `virtue.lone_redcap` encodes them additively and is *over*-funded
by one. The fix is neither deleting Lone Redcap's pool nor copying it onto
Redcap — both need the replacement shape.

**Shape notes already settled**, so the design slice does not rediscover them:

- The earmark is **not additional supply**. `effective/xp.rs::build_flow_pools`
  treats every restricted pool as a *source*; an earmark draws from the general
  pool instead — `general → earmark → eligible spends`. **The total budget must
  not rise.**
- **Authorization comes free.** `types.rs:1034-1035` already records that a
  `RestrictedAbilityXp` pool implies permission for what it funds, so an earmark
  naming five Abilities authorizes exactly those five and leaves every other
  Academic Ability gated — which is ArMDE:5791's second clause precisely, with no
  second effect.
- `from_normal_budget: bool` is `#[serde(default, skip_serializing_if)]`, so the
  twelve existing carriers' JSON is unchanged.
- **And D14 lands first** (§ 2.5).

---

## 3. Grouped correction plan

Each group names the change, the findings, the files, whether a test moves, and
the constraint. **Finding lists are by primary kind**; a finding whose secondary
kind belongs to another group is worked under its primary and mentioned there.

### 3.1 Extend `MECHANICAL_PHRASES` — the consolidated missing word-form list

**Change.** Add the word-forms below to `MECHANICAL_PHRASES` in
`crates/arm-rules/tests/uncomputed_clauses.rs`, in both locales, and fix the two
*structural* blind spots the batches identified alongside them.

**Findings.** F-383 (B10), F-403 (B11), F-424 (B12), **F-514 (B17)**, **F-537
(B18)**, plus the per-finding tables in B13 (six entries), B14 (five), B15
(nine), B16 (three), **B17 (fourteen)** and **B19 (one, `flaw.vulnerable_magic`,
demonstrated live)**. **This is the single list; it was scattered across seven
documents before this file.**

**Files.** `crates/arm-rules/tests/uncomputed_clauses.rs` only. **This is a test
change, and it is the only one that must land before data changes.**

**The baseline, corrected 2026-09-21.** Batches B10, B11 and B13 describe the
screen as "the **20-phrase** list", and the orchestrator repeated that figure in
every brief from B12 on. **It is wrong.** `uncomputed_clauses.rs:179-220` holds
**33 string literals in 15 bilingual groups** — so the list covers **15
concepts in two languages**, not twenty independent idioms. (An earlier version
of this note said "~17 concepts"; B19 counted the groups and it is 15.) The
figure came from `ecb5150`'s commit message and was never checked against the
array. **The list contains no prohibition idiom in either language** — which is
why `flaw.vulnerable_magic` sits inside a swept block, ships `narrative`, passes
the screen, and drops a real rule. The batch files keep the old number because they are dated
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

**B18 found the mirror defect: a needle already in the list that matches nothing
the book writes** (F-537). So the German half is not only *incomplete*, it is
partly *inert* — a form can sit in `MECHANICAL_PHRASES` looking like coverage and
screen zero passages. Any slice growing the list must verify each added DE form
against the German source, not against a plausible rendering of the English one.

#### Three measurements this group must carry

Stated together because each was taken by a different batch and each independently
sizes the slice.

1. **B17: fourteen of fifteen reclassifications go red immediately.** B17's span
   ships 24 `narrative` entries; fourteen state a rule in words the token screen
   cannot see, and **every one of them sits inside `SWEPT_BLOCKS`'
   `(ArMDE, 5639, 7113)`**, swept 2026-09-15 and re-swept 2026-09-19, green both
   times. This is the same ratio B11 (six of seven) and B12 (eight of eleven)
   measured, now confirmed a third time on a larger sample.
2. **B18/B19: the prohibition family is the single largest hole, and B19 proved
   it live.** `flaw.vulnerable_magic` lies inside the swept block, ships
   `narrative`, passes
   `no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause`, and drops a
   real rule — because the list's only "Absolutes" pair is `cannot die` /
   `nicht sterben` and ArMDE:7009's "*It may **not** be combined with…*" is
   invisible. That is a reproducible miss on an entry the sweep has twice
   declared clean, not a hypothetical.
3. **D8: the 48 reclassified Supernatural entries need an idiom family that does
   not exist.** They carry **no signed number and no botch term**, so they depend
   entirely on `MECHANICAL_PHRASES` — and there is **no capability idiom in
   either language**. D8 makes this a third requirement on § 2.1's sequence
   (§ 2.1b). The forms are **not in the table below**: the fourteen families were
   derived from B10-B19's findings, and none of D8's 48 is one. **Deriving the
   capability family is the first task of this slice**, and the sample to derive
   it from is the 48 entries themselves — D8 says to enumerate them from the
   data, not from any list.

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
   - **And `SIGN_CHARS` does not contain U+2014 at all** (F-514, B17), so an em
     dash used as a minus is invisible to the detector before the
     leftward/rightward question even arises. The fix is one character added to
     `SIGN_CHARS`; B17 weighed the false-positive risk and found it acceptable
     because an em dash in running prose is not adjacent to a digit.
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
exactly two encoded; this index finds **64 findings** on it.

**D14 lands first, inside this group. Added 2026-09-21.** Before the ~30 new
authorizations are written, `Effect::AbilityAuthorization` — and every other
effect that names an Ability — must gain an **optional parameter constraint**.
Today it carries `abilities: Vec<Id>`, *an id and nothing else*, and an Ability id
is not always the whole reference. Two entries are already wrong in two different
ways:

| Shape | Entry | What ships | What the book says |
|---|---|---|---|
| **A parameterized Ability, over-authorized** | `flaw.covenant_upbringing` | authorizes `ability.dead_language` — `academic`, `parameter: "language"` — so **every** dead language, Ancient Greek and Hebrew included | ArMDE:5867 grants **Latin** |
| **The entry's own parameter ignored** | `virtue.student_of_realm` | lists **all four** realm Lores unconditionally, so a magus who takes Student of the *Magic* Realm is authorized for **Infernal Lore** | the `realm` parameter already records which one he picked |

The second is the worse of the two: it is not an id-level proxy limitation, it is
a **missing binding** between the authorization and a choice the entry already
holds. So D14 asks for both forms — a **literal** value and a **binding** to the
selecting entry's parameter — and its syntax should be designed **once**,
together with **D9 part 3**'s multi-valued parameter type, rather than twice.
`RULES.md:6362`, which currently documents the over-permission as acceptable, is
rewritten rather than amended.

**Scope, stated because it sounds larger than it is.** Exactly one gated
parameterized Ability exists in the catalogue (`ability.dead_language`), so each
shape has one carrier today. **The reason to do it now is timing, not volume** —
see § 2.5.

**Findings (primary `auth`).**
F-18, F-22, F-50, F-59, F-67, F-68, F-69, F-71, F-72, F-101, F-102, F-128, F-132,
F-155, F-163, F-178, F-180, F-182, F-185, F-186, F-188, F-190, F-195, F-245,
F-253, F-260, F-264, F-273, F-292, F-323, F-347, F-388, F-409, F-429, F-461,
F-483, **F-539**.
**Plus the `class+auth` rows, which are the same fix with a classification change
riding on it:** F-06, F-09, F-10, F-14, F-17, F-26, F-27, F-28, F-29, F-38, F-66,
F-122, F-209, F-216, F-230, F-232, F-247, F-266, F-302, F-310, F-325, F-326,
F-349, F-377, F-445, F-460, F-488.

**Files.** `rules/core/virtues_flaws.json`. Where the classification moves, the
entry also needs `description` in `rules/i18n/{en,de}/virtues_flaws.json` per D5.

#### 3.2a The `Educated` family — work all five together, after D14

**Added 2026-09-21.** Three findings in this index are the same job and are
currently scattered across three *kinds*, so nobody would work them together:
**F-34** (`prov`), **F-59** (`auth`) and **F-60** (`scope`). They are one slice.

**The catalogue has one of the book's five `#### Educated` headings.**

| Heading | ArMDE | Catalogue |
|---|---|---|
| `Educated` | 3711 | ✅ `virtue.educated` |
| `Educated (Bardic)` | 3715 | ❌ |
| `Educated (Islamic)` | 3719 | ❌ |
| `Educated (Hebrew)` | 3723 | ❌ |
| `Educated (Vernacular)` | 3727 | ❌ |

These are **not parameter variants**: each carries its own heading, its own
`*Minor, General*` descriptor and its own rules paragraph, so they are five
distinct Virtues. They are also **the whole of F-34** — the census counted 626
headings against 622 entry starts, and every other heading maps, so closing this
closes the catalogue's only coverage gap.

**All five belong to this group.** Each grants 50 extra experience points
restricted to a named list, and all but Bardic state the permission idiom
outright — *"may purchase Academic Abilities during character generation"*
(ArMDE:3721, :3725, :3729). Without an entry, the character the passage describes
cannot be built at all: the 50 points land on `academic` Abilities, which is a
gated category.

**The Ability catalogue is already complete for them** — checked entry by entry.
`ability.islamic_law`, `ability.judaic_lore`, `ability.theology_islam`,
`ability.theology_judaism`, `ability.rabbinic_law`, `ability.art_of_memory` and
the parameterized `ability.profession` all exist. **So this is data-only**: four
new entries plus their `name`/`summary` in both locales. No Ability additions and
no engine work beyond D14 itself.

**But it cannot be done before D14, and F-60 is why.** F-60 records that base
`virtue.educated`'s 50-point pool names `ability.dead_language` where ArMDE:3713
says **Latin** — which is exactly the defect D14 was ruled on, *found in batch 2
and unfixable until now*. The four new entries make it sharper still: **Islamic**
names Arabic, Persian, Greek and Latin across both `ability.dead_language` and
`ability.living_language` (each `parameter: "language"`), and **Hebrew** names
Hebrew and Aramaic, plus Arabic conditionally for characters "from Iberia or the
East". Written against today's id-only shape, every one of them would grant *any*
language where the book names three or four. **Fix base Educated's Latin
reference in the same pass**, so the family is read once rather than twice.

**Terminology is already in place.** The 2026-09-21 table re-sync brought
`islamische-begriffe.md` and `juedische-begriffe.md` into
`rules/source/de/translation-tables/` — the glossaries these two Virtues need for
their German text arrived before the entries did.

**Out of scope for the anchor backfill (§ 3.14):** an entry that does not exist
cannot carry a `source.anchor`. These four are the reason that slice reports
651 of 655 rather than full coverage, and they should be re-run for anchors once
the entries land.
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

**B17-B19 add 21 more**, and they are the densest run in the audit: F-502, F-503,
F-504, F-505, F-507, F-508, F-510, F-511, F-512, F-516, F-517, F-519, F-520,
F-521, F-523, F-525, F-528, F-529, F-530, F-531, F-535, F-541, F-545. Two of them
are the group's sharpest cases: **F-523** reclassifies two entries **D3 names by
id**, so the data contradicts a written ruling; **F-545** was *passed* by its
batch's primary read on the reasoning that "the app models no seasons" and
**overturned by the verification pass**, because D3 does not admit engine
incapability as a ground for `narrative`.

**And D8 adds 48 that no finding numbers.** D8 reclassifies every
`supernatural` + `narrative` entry to `uncomputed_rule` — **48 of the 115
`supernatural` entries**, verified by `jq`. They are to be enumerated **from the
data**, not from any list in this file, and they are the reason § 2.1b exists: all
48 carry no signed number and no botch term, so none of them can satisfy the
`uncomputed_rule` guard until the screen grows a capability family. D8 also notes
that a separate `description` is owed **only where the `summary` does not already
carry the rule** (`RULES.md:4728-4731`, the `flaw.missing_ear` precedent) — most
of the 48 are one-sentence capabilities, so this is **largely a relabelling, not
96 new pieces of text**. Check each; do not assume either way.

**Files.** `rules/core/virtues_flaws.json` + both `rules/i18n/*/virtues_flaws.json`.

**Tests.** `uncomputed_clauses.rs` — and **this group is blocked on § 3.1**, now
including § 2.1b's capability family.

### 3.4 Missing descriptions (D5) — a computed entry that drops a clause

**Change.** Write the uncomputed clause into `description` in both locales. The
entry's `classification` does **not** move (D5's closing paragraph: "the
obligation moves; the taxonomy does not").

**Findings (primary `desc`).** The single largest kind in the index. B17-B19 add
**F-513, F-515, F-534, F-536, F-543** and **F-544**.

**F-544 is one finding covering seven entries and should be worked as one edit**
— `flaw.vulnerable_casting`, `flaw.vulnerable_magic`, `flaw.warped_by_magic`,
`flaw.weak_enchanter`, `flaw.weak_magic`, `flaw.weak_scholar`,
`flaw.weak_spontaneous_magic`, all shipping **no `description` in either
locale**. Two of its clauses are **ordering** rules between two effects (*"apply
the Deficiency first and then halve the remaining total"*, ArMDE:7062; *"halve
the Penetration Total **after** subtracting the spell level"*, :7066) where the
order changes the arithmetic — these are not decorative sentences.

**F-543 is the inverse shape and is easy to get wrong.** The entry that is
*correct* (`flaw.warped_senses`) is the only place four other entries' hard
prohibitions are legible. The description goes on the **four partners**
(`flaw.blind`, `flaw.deaf`, `virtue.keen_vision`, `virtue.sharp_ears`), stated
from each one's own side. **8 pairs are hard and 2 are soft** — ArMDE:7031 puts
both strengths in one sentence (*"It is **inadvisable** to combine…"* versus
*"and these **are incompatible** with…"*), and the two soft pairs must **not** be
encoded as prohibitions. Under **D16** a hedged restriction is a warning at most,
never an error.

**The German text must come from the line-parallel DE source, not from
translating the English.**

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
removal), F-366, F-371, F-385, F-435, F-459, F-465, F-466, **F-527**, **F-542**,
plus the umbrella **F-340 / F-376 / F-482**.

**Three of the new ones state a predicate, not a list, and that is a different
job.** `incompatible_with` holds bare ids, and the book increasingly does not:

| Finding | The predicate | Status |
|---|---|---|
| **F-526** (`flaw.university_dean`) | "any other Flaw that grants a Bad Reputation" — sixteen Flaws carry `grants_reputation` today | **Q-137**: ship the list plus a test that regenerates it, or add a predicate-valued exclusion. The two are **not** equivalent under CLAUDE.md's catalogue-size invariant |
| **F-542** (`flaw.weak_personality`) | "may have no other Personality Flaws" | a **category**-scoped prohibition; see § 3.15 and B19's closure note — one `Effect` variant closes this **and** F-355 |
| **Q-138** (ArMDE:6148, B13's entry) | "any Flaw that is only appropriate to Hermetic Magic" | worse than the other two: `categories: ["hermetic"]` includes Flaws that are not *only* Hermetic, so the data does not cleanly enumerate it. **Recorded so B13's entry is not left silently unrated** |

**F-527 is a plain two-sided id pair** and needs none of that.

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
F-390, F-402, F-416, F-440, F-448, F-463, F-485, **F-502**, **F-518**, **F-519**,
**F-526**, **F-532**, **F-533**.

**Files.** `rules/core/virtues_flaws.json`; `rules/core/character_types.json` for
the entity-kind ones (F-24, F-318, F-374, F-485); `RULES.md`.

**Constraint.** ~~D2 is pending~~ — **D2 is now ruled** (§ 2.2): both validators
become grant-aware, reading effective rather than bought-only selections, while
`PrereqCtx::build`'s exclusion of grants from the *incompatibility and trait*
checks stands. The distinction must be written as a comment at both sites.

**Three of the new gates are unhedged and encodable today**, and should not wait
on anything: F-518 (`{"kind":"all",[is_magus, house.verditius]}`), F-532
(`Prereq::IsMagus`) and F-533's magus half. **F-533's House half is the only
hedged one** — ArMDE:6957 says Vendetta "is **generally** restricted to magi of
House Verditius" — and **D16 now settles the shape**: a hedged restriction is a
**warning**, never an error. `IssueSeverity::Warning` and
`ValidationIssue::warning` already exist (`validation/mod.rs:73`, `:844`); no new
machinery is needed. Q-139 asked only whether `House("house.verditius")` joins
`IsMagus`, and D16 answers it by changing the *strength* rather than the
membership.

**Two are NOT encodable and must not be forced.** F-502 (`flaw.rector`) needs a
`Prereq` variant ranging over a `categories` value — 99 entries carry
`social_status`, and an enumerated `Prereq::Any` would couple the entry to the
catalogue's size, which CLAUDE.md's first architecture invariant forbids. That is
**Q-132**, and **F-427 / Q-07 / Q-102 need the same machinery for ArMDE:2816's
one-Social-Status rule — decide them together, not separately.** F-519's
`IsMagus` is *inferential* (the passage presupposes a Parma rather than stating a
gate) and B17 deliberately raised it as an observation; a fixer decides it, the
audit does not assert it.

### 3.7 Parameters, `max_total` and unrecordable choices

**UNBLOCKED and RESHAPED, 2026-09-21. Read this whole section before starting —
the change is not what it was.** D9 answers the blocking question *yes*, and D10
then inverts the mechanism, so the naïve reading of the old text ("add
`max_total: 1` to each entry") is now **explicitly forbidden**.

**Change, in three parts, in this order.**

**(a) Land D10 first — invert the default.** `PointItem::max_total` defaults to
`u8::MAX` (`types.rs:2209-2212`, `default_max_total`) and
`ParameterDef::max_per_value` likewise, while ArMDE:2814 is explicit in both
directions: *"A Virtue or Flaw may be taken more than once **only if the
description explicitly allows it**. **Most Virtues and Flaws may only be taken
once.**"* The book's default is once; the model's is unlimited. **Invert both
default functions to 1.** This is test-first and the reds are the point.

**(b) Declare the ~42 entries that legitimately repeat** — `jq`: `parameters !=
null and max_total == null`. Each is read against its own passage. The classic
repeaters (`virtue.puissant_ability`, `virtue.affinity_ability`,
`virtue.great_characteristic`, `virtue.minor_magical_focus`,
`virtue.major_magical_focus`, `virtue.deft_form`) need one; `virtue.mythic_blood`
and `flaw.servant_of_the_land` do not.

**(c) Then D9 part 1 — every entry whose passage tells the player to choose gets
a parameter.** Use `enumerated` where the book lists the options ("a warder,
teacher, or paramour"); use `text` only where the choice is genuinely open.
**The ruling is catalogue-wide and the finding list below is not the full set** —
D9 says so in terms: the slice must re-derive which entries state a choice.

> **Do not add `max_total: 1` entry by entry.** D9's original clause 2 said to,
> and D10 replaced it: *"if only once, it should be only once."* Author
> discipline is exactly what failed — **F-522** let a **Major** Flaw bank without
> limit and **F-541** licensed 255 identical copies — and D10 additionally
> reaches the 42 entries already riding the unlimited default, which D9's clause
> never touched.

**Findings.** F-03, F-07, F-40, F-57, F-70, F-94, F-98, F-112, F-139, F-141,
F-144, F-157, F-165, F-220, F-227, F-281, F-287, F-294, F-328, F-348, F-358,
F-362, F-400, F-412, F-451, F-495, **F-504**, **F-522**, **F-538**, **F-541**.

**Starting state, verified by `jq` for D9.** 51 of 655 entries carry parameters,
across ten domains (`text` 19, `form` 9, `ability` 7, `realm` 5, `enumerated` 4,
`art`/`category`/`characteristic`/`item` 2 each, `technique` 1). **Every
parameter in the catalogue is single-valued `"type": "ref"`.**

**The multi-valued type (D9 part 3) is a separate, test-first engine slice**, and
it is the only part of this group needing a **`SCHEMA_VERSION` bump** — because a
saved selection's parameter value changes shape. It is motivated by the three
Corrupted entries, where the book lets the player pick an open-ended *set*
(ArMDE:5851 "multiple Abilities", :5857 "multiple Arts", :5863 "as many … spells
as you wish"). It pulls in `types.rs::ParameterDef` (a second `type` with an
exhaustive `match`), a migration, a multi-select UI picker, and a **canonical
form for `max_per_target` grouping** — otherwise `{A, B}` and `{B, A}` read as
two different selections and the once-only rule fails again. CLAUDE.md's
canonical-serialization convention supplies the answer; apply it explicitly.
**Land it before the three Corrupted entries are touched** (D15), so they are
worked once. **And design its syntax together with D14's parameter binding**
(§ 2.5) rather than twice.

**A catalogue sweep this group owes, from B19's closure notes.**
`max_per_target` with **no `parameters`** is a *silent inversion*: with no
parameters every copy shares one empty target, so a high value licenses
*identical* repeats. That is the **right** encoding for a level-stack
(`flaw.vulnerable_casting`, `virtue.withstand_casting` — "Vulnerable Casting (2)
… loses 2 extra Fatigue levels") and the **exact inverse** for a vary-the-target
repeat (`flaw.vulnerable_magic` — "so long as a different condition is specified
for each"). **The two are indistinguishable in the data and sit three entries
apart**, which is the smallest sample that can show the ambiguity and the
strongest reason to think there are more. **Sweep every entry with a non-default
`max_per_target` and no `parameters` and ask which of the two rules it means.**

**Files.** `rules/core/virtues_flaws.json`; `crates/arm-rules/src/types.rs` for
(a) and for the multi-valued type.

**Note.** Several of these are **data loss** — the save cannot round-trip a choice
the rules require — which CLAUDE.md rates at the top, and that is the reason D9
overruled YAGNI: the alternative was shipping a model that cannot state what the
rulebook says, leaving two copies of a Flaw indistinguishable and the Markdown
export unable to print the choice at all.

**Q-134 is this family's sixth-through-eleventh instances, not a new question.**
B17 records five (`flaw.restriction`, `flaw.supernatural_nuisance`,
`flaw.repellent`, `flaw.rector`, `flaw.savantism`) and B19 adds
`flaw.vulnerable_magic`. **One of them is materially different and must be
weighed separately**: `flaw.repellent`'s "minor advantage" can be a **+3 Soak** —
a number that belongs on the character sheet — where the other five are narrative
labels. A blanket "no parameter for free-text choices" answer would leave a real
mechanical bonus unrepresentable.

**And F-504 needs a shape decision this group should take once.** Restricted
Learning's five chosen Abilities have **no precedent**: of 655 entries exactly
two carry more than one parameter at all
(`flaw.necessary_realm_aura_for_ability`, `virtue.folk_magic`), and in both the
two domains *differ*. The two routes are **five keys** (`ability_1` …
`ability_5`, all domain `ability`) — expressible today, no code change, and it
hardcodes the number five into data, which is acceptable because **five *is* the
rule**, unlike a catalogue size — or a **list-valued parameter**, which is D9
part 3's machinery. B17 recommends the first.

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
F-486, **F-510 (point 4 only)**, **F-524**, and **F-439 — now unblocked and
confirmed `high` by D17**.

**F-510 is split deliberately and the split matters.** Points 1-3 of the finding
are `class` + `desc` and belong to § 3.3 / § 3.4; **point 4 is a wrong number**
and belongs here. A red for point 4 is writable immediately and independently:
build an entity with `flaw.savantism`, a weapon Ability carrying a specialty and
a slot flagged `specialization_applies`, and assert the combat total. It is one
finding because it is one entry and one passage.

**F-524 is the audit's clearest wrong-rules-output case and the cheap fix is the
wrong one.** `derived/combat.rs` folds `u8::from(has_specialty)` into the combat
total, and **nothing** — no validator, no effect, no UI branch — consults
`flaw.unspecialized`; a grep of `crates/` and `ui/src/` for `unspecialized`
returns no hit at all. Two routes exist: a validator in `validation/scores.rs`
refusing a non-empty `specialty` while the Flaw is held, or teaching
`specialization_bonus` to return 0. **The first is correct and the second is a
patch**, because ArMDE:6945 is a statement about what the character *has*, not
about what he *rolls* — the second leaves the sheet printing specialties the
character cannot have and every other specialty consumer wrong. Note also that
the `description` half is **narrowed away**: the shipped `summary` already
carries the whole sentence verbatim in both locales, so writing it into
`description` would be a byte-identical duplicate (`RULES.md:4728-4731`).

**F-439 and F-428 are the same defect twice and are now § 3.17's, not this
group's.** Both are life-stage **replacement** encoded as addition. Do not patch
either number here; see § 3.17.

**Files.** `rules/core/virtues_flaws.json`; `crates/arm-rules/src/` for F-256,
F-428, F-449, F-524; `RULES.md` for every changed number.

**Tests.** Each needs a red that asserts the wrong number today.

### 3.10 Missing effects the engine can already express

**Change.** Add the `effects` row. No new machinery.

**Findings.** F-01, F-19, F-21, F-39, F-49, F-55, F-77, F-196, F-205, F-208,
F-243, F-249, F-296, F-329, F-341, F-342, F-406, F-463, F-494, F-498.

**Files.** `rules/core/virtues_flaws.json`, `RULES.md`.

### 3.11 Reputations

**UNBLOCKED by D11, 2026-09-21, and narrower than it looked.** All four blocking
questions and the Phase-0 one are answered, and **four of the five are answered
`no`** — the model does not grow. **This group is now almost entirely data**, plus
one small engine addition.

**What the ruling refuses, and why the refusals are not evasions.** ArMDE:1093
reverses the premise the questions rested on: *"Reputations have **a score, a
content, and a type**. … They don't determine how people react to characters they
have heard of, **as that depends on what they think of what they've heard**."*
The book gives a Reputation three components and `types.rs::Reputation` carries
exactly those three.

| Question | Answer | Why |
|---|---|---|
| **Q-43** — add a polarity field? | **No** | "A bad Reputation at level 3" describes the **content** — *Unclean*, *Usurer*. The book explicitly refuses to make good-versus-bad mechanical. A polarity field would invent a mechanic the rulebook does not have — the error `7f5605a` was reverted for |
| **Q-73** — grow `ReputationType` for organization scope? | **No** | ArMDE:1093 names Local, Ecclesiastical and Hermetic, with Academic alongside, and the ease-factor table has exactly those columns. "Among Templars", "among the Jewish community" are **content**, not types; a new member would be an audience the engine has no distance rules for |
| **Q-80** — wildcards? | **Already solved** | `GrantsReputation.kind: Option<..>` with `None` *means* player-chosen, and `virtue.famous` (ArMDE:3861) already uses it |
| **Q-127** + Phase-0 `score` | **`score` becomes enforced: exact by default, bounded where the book states a range** | see below |

**Change, as it now stands.**

1. **`validate_reputations` gains a score check.** Today it validates **kind and
   count only** — B18's F-525 confirmed this against `selections.rs:585-617` — so
   a grant's `score` is enforced nowhere, which is *why* Outsider's invented 3 and
   1 shipped with nothing objecting (F-486).
2. **Exact is the default.** The entity's Reputation score must equal the grant's.
3. **One optional upper bound carries the single exception.** Add
   `max_score: Option<u8>` to `Effect::GrantsReputation`, `skip_serializing_if` so
   **30 of the 31** granting entries' JSON is unchanged. Absent = exact; present =
   the score must lie in `[score, max_score]`. **31 entries grant a Reputation and
   exactly one states a range** — `flaw.outsider_major` / `_minor`, ArMDE:6554,
   *"a bad Reputation of **level 1 to 3**"* — which becomes `score: 1,
   max_score: 3` on **both** magnitudes, because :6556's Minor version says *"You
   **still** have the bad Reputation"*.
4. **Everything else is a plain data fix:** add the `grants_reputation` a passage
   states and the entry lacks (**F-408**, `flaw.excommunicate`), correct invented
   levels (**F-486**), and stop hardcoding an audience the passage leaves open
   (**F-450**, `flaw.infamous` pins `kind: "local"` where ArMDE:6312 names none,
   while its twin `virtue.famous` correctly ships the wildcard).

**Findings.** F-80, F-235, F-254, F-312, F-316, F-363, F-408, F-450, F-484, F-486.

**F-363 changes shape.** § 3.15 lists it as an engine gap — *"`grants_reputation`
cannot say a Reputation is **bad**"* — and **D11 rules that it never will**. The
polarity belongs in the Reputation's *content*, so F-363's remedy is a content
value, not a field. The same applies to **F-525**, whose "Gaining a positive
Reputation is impossible" is now **unenforceable by design**: its whole remedy is
the reclassification plus a `description`.

**Expect rejections, and do not migrate.** Enforcing a field that was never
checked **will reject characters that load cleanly today** — a save whose Infamous
Reputation was typed as 5 becomes invalid. Same treatment as D10: validation
reports it, `Enforced` blocks, `Advisory` warns, `Silent` suppresses, and **no
migration mutates the save**. **No `SCHEMA_VERSION` bump** is needed for the
entity side; `max_score` is additive on the ruleset side only.

**Files.** `rules/core/virtues_flaws.json`;
`crates/arm-rules/src/types.rs` (`max_score` only — *not* a polarity field and
*not* a wider `ReputationType`); `crates/arm-rules/src/validation/selections.rs`
(`validate_reputations`); `ui/src/lib/components/Reputations.svelte` pre-fills the
level (F-486).

### 3.12 Shipped text and localization

**Change.** Repair the shipped string in `rules/i18n/{en,de}/virtues_flaws.json`.

**Findings.** F-05, F-16, F-46, F-54, F-65, F-84, F-85 *(blocked on Q-10)*, F-88,
F-108, F-113, F-120, F-130, F-135, F-152, F-168, F-193, F-263, F-276, F-320,
F-381, F-382, F-404, F-405, F-415, F-419, F-433, F-458, F-471, F-473, F-490,
**F-506**, **F-509**.

**F-506 is the one German-name finding in the audit that needs no source-project
determination**, and it is the worked example of **D7 rule 3**. The shipped
*Eingeschränkte Kraft* follows `tugenden-fehler.md:346`, a row tagged **`SdM:M`**
(*Sphären der Macht: Magie*) — a different book from the one the entry cites — so
the row is *correct for its own book* and simply does not govern here. The core
book's own DE heading at :6687 is `#### Eingeschränkte Macht`, it collides with no
other entry (*Große Macht*, *Mindere Macht*, *Persönliche Kraft*), and the sibling
`flaw.slow_power` correctly ships *Langsame Kraft* from **its** heading. The
German core book genuinely uses *Macht* for one and *Kraft* for the other; the
two shipped names should differ and today they wrongly agree. **Per D6 the table
row is annotated in both projects, not corrected.**

**But apply D18's three-way test before the edit.** D18 (2026-09-21) establishes
that the shipped German rulebook is an **older edition** than the tables, so a
table–heading disagreement now has three possible causes rather than two: a table
error, a stale copy, **or a correction from a newer edition**. F-506's reasoning
is independent of currency — it turns on the row's *book tag*, not on which is
newer — so it survives the test, but the test must be run rather than assumed.

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

**D18 makes that blocker structural, not incidental, and it is the single most
important addition to this group. Added 2026-09-21.**

1. **The tables are NEWER than the rulebook copy.**
   `rules/source/de/Ars Magica Definitive Edition Basisregeln.md` is an older
   edition; the tables in `arm-de-translation` are maintained against a newer
   German rulebook. **So a table row disagreeing with a heading may be a
   correction the shipped book has not caught up with.** Two cases established
   it, both ruled *table wins, data changed*: `virtue.relic` / `virtue.powerful_relic`
   (*Relikt* → **Reliquie**) and `spell.curse_of_circe` (*Fluch der Kirke* →
   **Fluch der Circe**). The Relic row even carried its reasoning (*"**Nicht**
   ‚Relikt'"*), which under D7 alone read as a terminology table overreaching.
2. **A disagreement is now a three-way question**: table error, stale copy, or
   newer-edition correction. **Only the first is fixed in the table.**
3. **DO NOT auto-fix the remaining disagreements.** A sweep after the 2026-09-21
   re-sync found **21** table-versus-shipped-data disagreements, and the obvious
   action — bringing 20 rows into line with the heading — would have **reverted
   genuine corrections**. Each must be checked individually against the current
   edition, which only Norbert can consult. The list is in
   `tmp/table-sync-check.md`.
4. **D7's rule 2 stands, narrowed.** The heading still wins for a name *at equal
   currency*. It does not win against a later edition.

**Standing instruction attached to the same ruling: DO NOT re-sync the rulebook
sources.** Pulling in the newer German (and English) books would shift **every
line number** — 655 `source.lines` ranges, every `ArMDE:NNNN` citation in code,
`RULES.md`, all nineteen batch files, `SWEPT_BLOCKS`, and
`rulebook_citations.rs`'s range guard, which checks only that a range lands on
non-blank lines and would therefore go **green on citations now pointing at the
wrong text**. See § 3.14 for the consequence this has for `source.anchor`.

### 3.14 `source.lines`, anchors and provenance

**Findings.** `src`: F-48, F-200, F-215, F-309, F-370, F-455, **F-540**. `prov`:
F-16 (the **id** `virtue.rard` is wrong, not only the name), F-34 (four rulebook
Virtues absent from the catalogue), F-270, F-283 (a required Virtue that does not
exist), F-286, F-307, F-392, F-393, F-443, F-500, **F-520** (secondary),
**F-536** (secondary).

**F-540 is first in this group and is a § 2.1a prerequisite, not a tidy-up.**
`flaw.wrathful_major` / `_minor` declare `[7106, 7119]`; the Flaw occupies
**7106-7108**, and 7110 is `---`, 7112 a chapter-closing pull quote and **7114
`# Chapter 5: Abilities`**. Correct value `[7106, 7109]`. B19 confirmed
catalogue-wide that these are the **only** two entries citing past 7113
(`jq '[.[] | .source.lines[1]] | max'` = 7119, carried by exactly those two;
`grep -c "7119"` independently returns 2), so **fixing them lowers the
catalogue's maximum cited line to 7109 and brings all 655 entries inside
`SWEPT_BLOCKS` for the first time** — which is the whole point of § 2.1a's
ordering.

#### Two catalogue-wide obligations B19's closure pass added

Neither is a per-entry finding, and neither should be filed as 561 of them.

**1. `source.anchor` covers 94 of 655 (14.4 %), and the 94 are not a principled
subset.** In B19's own span exactly three entries carry one
(`flaw.vulnerable_to_folk_tradition`, `flaw.warped_senses`,
`flaw.weak_personality`) — and those are precisely the three whose `description`
is a verbatim transcription of the passage. **The field looks like a by-product
of the description-extraction pass rather than something the catalogue populates
on purpose.** All three present anchors are correct and the 22 absences break
nothing today, but 14.4 % is the worst kind of partial: present often enough to
look reliable, absent often enough not to be.

> **D18 promotes this from cleanup to prerequisite.** The anchor is derived from
> the `####` heading, so it survives a re-pagination; a line range does not, and
> **fails silently**. **Backfilling anchors to 100 % is the prerequisite for ever
> accepting a newer rulebook.** Treat it as such rather than as the low-priority
> tidy-up **Q-96** filed it as. The fixing pass must first decide whether `anchor`
> is **mandatory or dropped**; D18 points at mandatory.

**2. Line-citation drift is now the catalogue's only unguarded provenance axis,
and one assertion closes it.** Three guards exist and none covers this shape:
`rulebook_citations.rs` guards *comment* citations; `rules_source_provenance.rs`
guards out-of-bounds `source.lines`; `uncomputed_clauses.rs` guards
misclassification inside swept blocks. **Nothing guards a range that is in
bounds, lands on non-blank lines, and is simply the wrong extent** — which is
exactly F-540, and the reason it survived nineteen batches at the most visible
position in the file. **A cheap guard exists: assert that a range ends before the
next `####`/`###`/`##`/`#` heading in the cited file.** That one assertion would
have caught F-540 mechanically.

**Files.** `rules/core/virtues_flaws.json`, `crates/arm-rules/RULES.md`,
`crates/arm-rules/src/types.rs` (doc comments),
`crates/arm-rules/tests/rules_source_provenance.rs` (the extent assertion).

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
| F-355, F-410, F-426, **F-511**, **F-542** | no *negative* authorization: nothing can forbid an Ability category — **and see the consolidation note below** |
| F-363 | ~~`grants_reputation` cannot say a Reputation is **bad**~~ — **D11 rules it never will**; polarity lives in the Reputation's *content*, so this is a data fix in § 3.11, not an engine gap |
| F-383, F-403, F-424, **F-514**, **F-537** | the phrase screen — § 3.1 |
| F-423 | `SurfacedModifier` carries no source item, so six Flaws render as one unattributed word |
| F-427, **F-502** | nothing ranges over a `categories` value: ArMDE:2816's "one Social Status" rule is enforced nowhere, and `flaw.rector`'s "must have a Social Status Virtue" cannot be written (**Q-132**) |
| F-489 | no `Effect` variant can name an Ability for a *roll* modifier (all 42 enumerated) |
| F-500 | `WarpingGrant.score` is dead data with no integrity check |
| **F-524** | nothing consults `flaw.unspecialized`, so `derived/combat.rs` prints a +1 the Flaw forbids — the fix is a validator, not an effect (§ 3.9) |
| **D14** | an ability reference in an effect carries an id and nothing else, so it cannot name a parameter — § 2.5, § 3.2 |
| **D9 part 3** | no multi-valued parameter type, so "as many spells as you wish" is unstatable — § 3.7 |
| **D13 / D17 / F-428 / F-439** | two of the rules' three XP modes have no shape — **§ 3.17** |

**Consolidation, from B19's closure pass: F-355 and F-542 are ONE missing
mechanism, not two.** F-355 is the missing category block on the **Ability** axis
("a class of Abilities", ArMDE:5653); F-542 is its twin on the **Virtue/Flaw**
axis ("no other Personality Flaws", ArMDE:7078). The engine has the *vocabulary*
on neither — `profile.forbidden_categories` even supports a conditional `when`
form, as `character_types.json`'s companion profile shows (`hermetic` forbidden
unless `virtue.the_gift`) — but there is **no route from a V/F to it**. **One
`Effect` variant carrying a category-scoped prohibition closes both. Deciding
them separately risks two mechanisms for one idea.**

**F-511 is a third sub-shape and the cheapest test case for the family.**
`flaw.sheltered_upbringing` names **seven specific ids**, not a category — so it
wants an *id-list* negative authorization where F-355 and F-410 want a
*category*-ranging one. All seven ids resolve and all seven are `general`, so no
category gate confounds the test.

**F-542's remedy is deliberately the expensive one, and B19 defends the choice
against D3.** An `incompatible_with` enumerating the Personality Flaw ids is
expressible **today** — plain data, no new variant — so D3's
"structurally-inexpressible" exemption does **not** apply and the finding stands.
The reason to prefer the category variant is maintainability: an id list would
silently *weaken* the absolute every time a Personality Flaw is added, and cannot
catch the "grants a Personality Trait" half at all. `Prereq::Nor` is worse still
— same decay, plus `validate_incompatibilities` is bought-only on both sides, so
a *granted* Personality Flaw would slip through.

### 3.16 The audit's own records

| Finding | What |
|---|---|
| F-444 | `decisions.md` D6's unscoped precedence rule — **already corrected in `decisions.md`; verify** |
| F-499 | `corrections.md` did not exist — **this file** |
| F-514, F-537 | two defects in `uncomputed_clauses.rs` itself: `SIGN_CHARS` lacks U+2014, and one German needle matches nothing the German book writes. Filed `engine+audit` and worked in § 3.1 |
| — | B15's F-472/F-474/F-475/F-476/F-477/F-479 each assert that an earlier batch "did not report" a defect that the earlier batch did report. Six false statements of record; see § 4. |

### 3.17 The three XP modes — **design all three together** (NEW, 2026-09-21)

**Change.** Give the XP model the two shapes it lacks. This group did not exist
when this file was written; **D13** created it, **D17** filled in its third mode,
and both say in terms that the three must be designed in one pass.

**The modes.**

| Mode | Shape | Carriers | Status |
|---|---|---|---|
| **Additive grant** | new points **on top of** the budget | the twelve existing `RestrictedAbilityXp` carriers — Educated, Warrior, Privileged, … | **modelled** |
| **Earmark** | N **of the existing** budget, constrained to named Abilities | `flaw.church_upbringing`, ArMDE:5791 | **D13 — unbuilt** |
| **Replacement** | **substitutes** a life-stage block | childhood: `flaw.feral_upbringing` (**F-428**). apprenticeship: `virtue.lone_redcap` (**F-439**) and the unencoded `virtue.redcap` (**D17**) | **unmodelled** |

**Findings.** **F-428**, **F-439** (unblocked by D17 and confirmed `high`), plus
the unnumbered `virtue.redcap` defect D17 records.

#### The earmark (D13)

ArMDE:5791: *"The player **must spend 25 experience points from the normal
budget** on Artes Liberales, Latin, Music, Organization Lore: Church, or Theology.
**Unless the character has a Virtue that permits it, no other experience points
may be spent on Academic Abilities.**"*

Two clauses, neither expressible today. `RestrictedAbilityXp` **grants** new
points, which would turn a Flaw into a benefit; `AbilityAuthorization` is
unbounded, which would permit exactly the spending clause 2 forbids. The entry
ships `narrative` with no effects, so **a player who does what the Flaw mandates
gets a hard `ability_category_requires_virtue`** — a legal character the app
refuses to build.

1. **`from_normal_budget: bool`** on `RestrictedAbilityXp`, `#[serde(default,
   skip_serializing_if)]` so the twelve existing carriers' JSON is unchanged.
2. **One new edge shape in the flow graph.** `effective/xp.rs::build_flow_pools`
   treats every restricted pool as a *source*. An earmark is not supply — it is a
   constraint on supply the character already has: `general → earmark → eligible
   spends`, not `source → pool → eligible spends`. **The total budget must not
   rise.**
3. **Authorization comes free**, and this is what makes the design tidy.
   `types.rs:1034-1035` already records that a restricted pool implies permission
   for what it funds, "since the grant would otherwise be unspendable" — so an
   earmark naming five Abilities authorizes exactly those five and leaves every
   other Academic Ability gated by `categories_requiring_virtue`. **That is clause
   2 precisely, with no second effect.**
4. **`restricted_xp_unspent` applies unchanged** (`validation/life_stage.rs:745`);
   here the existing warning says something the rulebook actually requires.
5. **`flaw.church_upbringing` becomes `creation_effect`**, with the full passage in
   `description` in both locales per D5 — the "unless the character has a Virtue
   that permits it" escape is **not** modelled and must reach the player as text.
6. **Test-first.** The red that matters: a character with this Flaw who spends none
   of the 25 must fail, **and the total budget must be unchanged by taking the
   Flaw.**

**Sizing, recorded because it argues the other way and was overruled.** "From the
normal budget" appears **exactly once in the rulebook**, so this is an engine
change for one entry in 655, and the cheaper approximation had a documented
precedent at `RULES.md:6362`. Norbert chose the exact model.

#### The replacement (D17, F-428)

**D17 resolved Q-108 from the source rather than by choice, and it resolved it
toward "replaces".** ArMDE:4848 (`virtue.redcap`): *"You **have spent fifteen
years as an apprentice**, and gained a **total of 300 experience points in those
fifteen years**."* ArMDE:4321 (`virtue.lone_redcap`): *"You **still** begin with
300 experience points **for your fifteen years spent as an apprentice**."* Both
describe the **same** 300 for the **same** fifteen years; "still" means the Lone
Redcap keeps what a Redcap has despite losing his Mercer House ties.

**The structural proof, from `rules/core/life_stages.json`:** a magus's
`apprenticeship` is already **a fixed 15-year, 240-XP block that replaces
per-year later-life XP**. The Redcap's is the identical structure with a
different figure. Reading it as additive would make the Redcap the only character
in the game whose apprenticeship is funded twice.

**The arithmetic, which is F-439.** A companion has no `apprenticeship` creation
phase, so a 25-year-old Lone Redcap is funded childhood (120) + later life for
years 5-25 (20 × 15 = 300) — and the Virtue then adds **another 300**. The book
gives him roughly five years of ordinary later life plus the 300 block.
**Over-funded by around 225 XP; F-439 is confirmed `high`.**

**And the two Redcap entries are wrong in OPPOSITE directions**, which no question
anticipated. `virtue.redcap` does not encode its 300 **at all** — it carries only
`item_level_budget: 50` — so the **Major** Redcap is *under*-funded by a whole
apprenticeship block while the **Minor** Lone Redcap is *over*-funded by one.

**The fix is neither deleting Lone Redcap's pool nor adding a matching one to
Redcap.** Both need the replacement shape, and `flaw.feral_upbringing` (F-428)
needs it at the childhood block. A `from_normal_budget` flag and a
`replaces_life_stage` flag are the same kind of answer to the same kind of
question; **building one in ignorance of the other invites a third incompatible
spelling.**

**Constraint. D14 lands first** — see § 2.5. Two of Church Upbringing's five named
Abilities are parameterized (`ability.dead_language` = Latin,
`ability.organization_lore` = Church), so written against today's id-only shape
the earmark would ship an approximation on day one, which is the opposite of
D13's whole purpose.

---

## 4. Cross-batch duplicate check

This is the check whose absence produced **F-499**. Nobody had run it before.

### 4.1 Headline

**Ten duplicate relationships across all 544 findings, and all ten are in
F-001…F-501.** One (F-481) was already found and withdrawn by B16. **Nine were
not**, and every one of them is a *cross-span* finding — a batch following a
pointer or a scan out of its own range and rating an entry another batch had
already rated. Three further findings overlap an umbrella at the *site* level.

In six of the nine, the later finding states in its own body that the earlier
batch "did not report" the defect. **That claim is false in all six cases**, and
it is the mechanism: the later batch asserted a negative about a document it had
not read, because nothing indexed the findings.

**F-502…F-545 add ZERO duplicates.** The extended check (§ 4.2, passes 5-7) found
no pair in B17-B19 that re-rates a defect an earlier batch already rated, and none
inside B17-B19 either. It did confirm **seven declared overlaps** — sites where a
new finding touches an entry or a line an earlier finding owns, *and says so* —
which are recorded in § 4.4 rather than as duplicates.

**Why the rate drops to zero, stated so the number is not mistaken for luck.**
The nine duplicates all arose the same way: a batch scanning outside its own span
asserted a negative about an unindexed record. B17, B18 and B19 ran with this
file in existence and each explicitly checked it — B19's F-543 opens with a
"**Prior-record check**" naming `corrections.md:271`, `:431` and `:539` by line —
so the mechanism that produced the nine was closed before these three ran. **That
is a claim about these three batches' method, not a guarantee about the
catalogue.**

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

#### The extension to F-502…F-545, run 2026-09-21 — three further passes

Passes 1-4 above were re-run over all nineteen batch files. **The floor is
repaired for this span**: passes 5-7 are the body-level scan § 4.2 admits it
lacked, run in both directions across the B17-B19 boundary. The result is
**zero** duplicates, and this is how that was established.

5. **Heading-level, extended.** `grep -a "^### F-"` across `batch-01.md` …
   `batch-16.md`, filtered by an alternation of **every entry id named in a
   B17-B19 finding heading** (44 findings, 49 distinct ids including F-543's four
   partners and F-544's seven entries). Twenty-five headings matched the
   alternation; **all but four matched on a shared *word*, not a shared entry**
   (`restriction`, `blind` and so on inside an unrelated heading). The four real
   entry matches are F-352 (`flaw.vulnerable_casting`), F-369 (`flaw.blind`),
   F-498 (`flaw.servant_of_the_land`) and F-126/F-271 (`virtue.keen_vision`,
   `virtue.sharp_ears`) — **all four are declared overlaps, cited by name in the
   B17/B19 finding that touches them**, and all four are a *different defect* on
   the shared entry. See § 4.4.
6. **Body-level, B17-B19 entries against B01-B16 bodies.** `grep -ao` over the
   whole text of `batch-01.md` … `batch-16.md` for each of the 44 entry ids —
   headings *and* bodies, which is exactly what pass 1 could not do. Seventeen
   ids appear at all, in 39 places. Every one was opened and read. **Two are the
   near-misses this pass existed to find, and neither is a finding:**
   - `batch-12.md:1732` — B12's verification sub-agent *"flagged
     `flaw.savantism` as a `creation_effect` with no effects"*, which is F-510's
     defect exactly — but B12 declined to number it, routing it to **C7** in
     `engine-semantics.md` and `README.md`'s list of five. An observation, not a
     finding, so F-510 is the first numbering of it.
   - `batch-15.md:2182-2195` — B15 describes `flaw.vulnerable_magic`'s
     `narrative` class, absent parameters and empty `incompatible_with` in
     detail, and labels it **"A B19 lead"** in its own words. Again unnumbered.
     F-541 is the first numbering.

   The other fifteen are citations of context (`flaw.weak_scholar` in D4's table,
   `flaw.deaf` in a cleared negative, `flaw.tragic_life` in a pointer census) and
   two explicit *non*-reads: `batch-09.md:2140-2146` records that
   `flaw.restricted_power` and `flaw.slow_power` "**their passages were not read
   here**".
7. **Body-level, the reverse direction.** Every `virtue.*` / `flaw.*` id occurring
   anywhere in B17's, B18's and B19's **Findings** sections was extracted (36 + 32
   + 29 distinct ids) and the out-of-span ones checked against B01-B16's finding
   headings. All are cited as precedent, contrast or context — F-518 says of
   F-463 *"the same **shape** … on a different entry and a different gate"*, F-511
   distinguishes itself from F-355/F-410 as a distinct sub-shape, F-542 names
   F-355 as its twin axis. The one genuinely new pairing checked from scratch was
   **F-527**'s `virtue.true_faith` incompatibility: B09 rates True Faith twice
   (F-329 the Magic Resistance formula, F-330 the Faith Point), **neither on the
   Uncertain Faith exclusion**, so F-527 is new.

**What this extension still does not cover.** It does **not** re-run the
body-level scan over F-001…F-501 against each other — the original floor
disclaimer stands for that range, and B17's and B19's own spans were scanned only
by their own batches. **"Nine" remains a floor for the first sixteen batches;
"zero" is a measured result for the last three.**

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

**B17-B19's seven, all declared by the batch that raised them.** None is a
duplicate; each is a *second, different* defect on an entry or a line an earlier
finding owns, and each must be worked in the same edit as its partner so the
entry is opened once.

| Later | Shares its site with | What is actually different |
|---|---|---|
| **F-522** (B17) | **F-498** (B16), `flaw.servant_of_the_land` | F-498 is the ungranted, budget-exempt Prohibition; F-522 is the missing `max_total` that lets the Major Flaw bank without limit. B17 confirms F-498 live and extends it with the fact that `flaw.prohibition` exists as a grantable id. |
| **F-539** (B19) | **F-500** (B16), `flaw.warped_by_magic` | F-500 is `WarpingGrant.score` as dead data; F-539 is the missing Magic Lore authorization. B19 explicitly declines to re-report F-500. |
| **F-544** row 1 (B19) | **F-352** (B10), `flaw.vulnerable_casting` at ArMDE:7003 | **Same line, two rules.** F-352 owns the *prohibition* (the missing two-sided `incompatible_with` with `virtue.withstand_casting`); F-544 owns the *order-of-application* rule in the same sentence, which no `description` carries. |
| **F-544** rows 2-3 (B19) | **F-541**, **F-539** (B19, same batch) | Declared in-batch: the D5 description obligation on two entries that also carry their own class/auth finding. One edit per entry. |
| **F-543** (B19) | **F-369** (B10, `flaw.blind`), **F-126** (B05, `virtue.keen_vision`), **F-271** (B08, `virtue.sharp_ears`) | B19 ran the **prior-record check by line** and found all three rated on their *classification* and none on the Warped Senses prohibition. `flaw.deaf` and `flaw.warped_senses` carry no prior finding at all. |
| **F-518** (B17) | **F-463** (B14), `flaw.magical_air` | Same *shape* — an unhedged eligibility absolute gating nothing — on a different entry, a different passage and a different `Prereq` tree. Filed separately, and B17 says why. |
| **F-542** (B19) | **F-355** (B10), `flaw.ability_block` | Different entries, **one missing mechanism**: the Virtue/Flaw axis and the Ability axis of a category-scoped prohibition. B19's closure note asks for one decision, not two. |

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

## 6. Open-questions index — Q-01 … Q-142

**Who settles it** uses four codes. **N** = Norbert's decision (a design, model or
policy call the source cannot settle). **R** = a rules read (the answer is in
`rules/source/en/`; an agent can settle it). **C** = a codebase read. **X** =
needs `arm-de-translation`, which is outside this repository. A question that has
been settled carries **—** in that column and names the ruling.

**REVISED 2026-09-21 against `decisions.md` D8-D18.** Ten rulings landed after
this section was first written, and they close **25 further questions**, so a row
that still reads "open" here has been re-checked against the ruling list rather
than left standing by default.

**Thirty-four of the 142 are closed**, in three tranches:

- **Before D8 (nine).** Q-02 (D3), Q-35 (F-160), Q-52 (via Q-120's resolution),
  Q-85 (withdrawn, answered from the repo), Q-120 (the enum's own doc comment),
  Q-129 (**D7**), and Q-03/Q-14/Q-22 as a family (**D5** — see the note under the
  table).
- **By the 2026-09-21 rulings (twenty-five).**

  | Ruling | Closes |
  |---|---|
  | **D8** — a roll-free supernatural capability is a rule | **Q-76**, and with it **Q-64, Q-17, Q-61, Q-126** — 48 entries |
  | **D9** — record every stated choice; add a multi-valued parameter type | **Q-86, Q-88, Q-31, Q-36, Q-95, Q-122, Q-128**, and **Q-134** as its six named instances |
  | **D11** — the Reputation model does not grow; `score` gets enforced | **Q-43, Q-73, Q-80, Q-127**, and the Phase-0 `score` question (§ 6.1) |
  | **D12** — Hermetic V/F are intrinsic or trained | **Q-90** |
  | **D13** — an earmark of the normal budget is a third XP mode | **Q-92** |
  | **D15** — the three Corrupted entries get one treatment | **Q-93** |
  | **D16** — an unmodelled absolute is text; a *soft* restriction is a warning | **Q-05, Q-123**, and **Q-139** by its own generalisation clause |
  | **D17** — the Redcap's 300 points replace | **Q-108** — which **unblocks F-439 and confirms it at `high`** |
  | **D2** — granted Virtues and the bought-only validators | **Q-13**, as a by-product of its rules read |

- **Two further rulings close no question but change many answers.** **D10**
  supersedes D9's multiplicity clause and reaches the 42 entries already on the
  unlimited default (§ 2.6). **D18** amends D7 and bears on **every**
  translation-table question — Q-04, Q-10, Q-40, Q-49, Q-54, Q-63, Q-65, Q-69,
  Q-71, Q-114, Q-121 — by adding a third possible cause for a table–heading
  disagreement: *a correction from a newer German edition*. It also re-rates
  **Q-96** from cleanup to prerequisite. **D14** closes none either, but it
  supplies the mechanism Q-122 needs and is a hard predecessor of D13 (§ 2.5).

**A ruling that closes a question does not always unblock the work.** D8's 48
reclassifications are still blocked behind § 2.1b, and D13's remedy is still
blocked behind D14.

| Q | Topic | Settled by | Status |
|---|---|---|---|
| Q-01 | the closed value set for `virtue.academic_concentration_subject`'s `subject` | R | open |
| Q-02 | is a prereq-only entry `creation_effect` or `narrative`? | — | **closed — D3, superseded by F-32** |
| Q-03 | does a computed entry owe a `description` for its uncomputed clauses? | — | **closed in substance — D5** |
| Q-04 | `virtue.alluring_to_beings` German name: table vs DE rulebook heading | N | open — **D7 supplies the test, D18 adds a third arm to it** (table error / stale copy / newer-edition correction) |
| Q-05 | "only available to male characters" — the engine has no sex model | — | **closed — D16: `uncomputed_rule` + `description` in both locales. `Entity` does NOT gain a sex field and nothing enforces it.** Scope re-counted catalogue-wide: **20** entries, not 6, plus one female-only exception (`virtue.baccalaureus`, ArMDE:3474) and ArMDE:3438's "who must also be eunuchs". **A deliberate non-goal — do not re-open** |
| Q-06 | `virtue.aptitude_for_sin`: should the +3 be an `ability_roll_mod`? | N | open |
| Q-07 | ArMDE:2816's one-Social-Status rule is modelled nowhere | N | open — **rated a defect by F-427** |
| Q-08 | the ArMDE:2960-2962 realm association is on 4 of ~115 Supernatural entries | N | open |
| Q-09 | `virtue.common_sense`: is a guaranteed storyguide intervention mechanical? | N | open |
| Q-10 | `virtue.covenfolk`: two canonical German sources, two names | X / N | open — **F-85 is blocked on it**; **D18** adds the newer-edition arm |
| Q-11 | `virtue.domestic_animal`: "animals only" with no engine model | N | open |
| Q-12 | `ability.enchanting` parameterized? is `domain: "ability"` right? | N | open |
| Q-13 | does the book distinguish "may only **take**" from "may only **have**"? | — | **closed — D2's rules read: NO.** ArMDE:3665 (*Demonic Might*) says "may only **take**" and :3669 (*Demonic Powers*), the very next entry, states the *same* restriction as "may **have**". Two adjacent entries, one restriction, two verbs. **No argument may rest on that pair** |
| Q-14 | where is the threshold at which an effect-bearing entry owes a `description`? | — | **closed in substance — D5** |
| Q-15 | `virtue.craftsman`: is "Wealthy/Poor affect you normally" a clause? | R | open — settled in practice (§ 5) |
| Q-16 | `virtue.dust_devil`: how much of Skinchanger does "this variant" inherit? | R | open |
| Q-17 | a supernatural power described only in prose, with constraints but no number | — | **closed — D8** (with Q-61, Q-64, Q-76, Q-126): a capability is a rule → `uncomputed_rule` |
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
| Q-29 | `greater_immunity` and `lesser_immunity` disagree about repeatability | R | open — **D10 supplies the mechanism** (absent now means *once*), but which of the two passages is right is still a rules read |
| Q-30 | how to model a "General **and** Hermetic" descriptor without blocking the unGifted case | N | open |
| Q-31 | three Supernatural Virtues require player-defined content and none records it | — | **closed — D9**: record every stated choice |
| Q-32 | `AdvancementSource` has `Teaching` but nothing for books-written | N | open |
| Q-33 | the book gives Hermetic Prestige two different levels and the data picks one | R | open |
| Q-34 | ArMDE:18432's Size-scaled Improved Characteristics is modelled nowhere and unreachable | N | open |
| Q-35 | `virtue.knows_people`: is a once-per-story entitlement mechanical? | — | **closed — F-160** |
| Q-36 | `virtue.lasiq`: does "some other social status, which you should choose" demand a parameter? | — | **closed — D9: yes** |
| Q-37 | `virtue.leper_magus`: `grants_selection` or a copied effect? | N | open |
| Q-38 | does Greater Immunity's repeatability transfer through Lesser Immunity's cross-reference? | R | open |
| Q-39 | `virtue.linguist`: the book rounds the XP **up**, the engine rounds the **cost** up — do they agree? | C | open |
| Q-40 | `virtue.linguist`: the table's only row is in a supplement section and gives "Linguist" | X | open — **D18**: check the newer-edition arm before calling the row wrong |
| Q-41 | `virtue.lone_redcap`: is `supernatural` in the 300-point pool sourced? | R | open |
| Q-42 | `virtue.lone_redcap` / `virtue.redcap`: nothing encodes that they are alternatives | R / N | open |
| Q-43 | a granted Reputation has no **polarity**, so "a *poor* Reputation at level 2" is unrepresentable | — | **closed — D11: NO polarity field.** ArMDE:1093 gives a Reputation three components (score, content, type) and explicitly refuses to make good-versus-bad mechanical. "Bad" is **content**. Consequences: **F-363** stops being an engine gap, and **F-525**'s absolute becomes unenforceable by design |
| Q-44 | no `Effect` expresses free seasons per year, though three entries modify them | N | open |
| Q-45 | ArMDE:2816 is not a cap, and one `virtue_category_caps` row cannot express it | N | open |
| Q-46 | the leather variant of the two vessel Virtues: entry, parameter, or nothing? | N | open |
| Q-47 | `restricted_ability_xp` cannot scope a pool to one Profession, and three entries name one | N | open |
| Q-48 | Mercurian Magic's companion Flaw: a prerequisite the player is paid for, or a budget-exempt grant? | N | open |
| Q-49 | the reputation table attributes to Mythic Blood a Reputation ArMDE:4588 denies | X | open — **D6 rule 1 decides it** (a terminology table making a factual claim), and **D18 does not rescue this one**: a newer edition changes *terminology*, not which Virtue grants which Reputation. The fix still needs the source project |
| Q-50 | `mythic_type.nephilim` carries neither of ArMDE:2731's two point adjustments | R / N | open |
| Q-51 | should Magical Blood's Magic Human variant become effects once a parameter exists? | N | open |
| Q-52 | is `virtue.masterpiece` `creation_effect` or `in_play_effect`? | — | **answered in substance by Q-120's resolution** |
| Q-53 | the book names six Ability types; `AbilityCategory` has five | R / N | open |
| Q-54 | does a glossary table govern *inside a verbatim quotation*? | N | open — **Q-63 family**; **D18** bears on it (the table may be the more current wording) |
| Q-55 | `virtue.physician_of_salerno`: is the granted Reputation Local or Academic? | R | open — **narrowed by D11**: both are valid `ReputationType` members and the type set does **not** grow, so this is now a plain rules read, no longer "Q-73 family" |
| Q-56 | `virtue.perfectus`: should Purity and Transcendence enter the Ability catalogue? | N | open |
| Q-57 | Potent Magic: how should the forced magnitude-variant incompatibility be lifted? | C / N | open |
| Q-58 | `virtue.ripper`: are the two fixed powers a `power_levels` grant or pre-entered powers? | N | open |
| Q-59 | `virtue.privileged_upbringing`: can a pool-scoped permission be expressed at all? | N | open |
| Q-60 | `virtue.personal_vis_source`: is "about one tenth" a rule or hedged guidance? | N | open — **Q-83 family** |
| Q-61 | `virtue.rat_up_a_drainpipe`: is "a substantial advantage" mechanical? | — | **closed — D8: yes** |
| Q-62 | `virtue.powerful_relic`: is the relic's one power charged against the power-levels budget? | R | open |
| Q-63 | does a translation table's term bind *inside a sentence*, or only on a label? | N | open — **F-65, F-320, F-473 depend on it**; **D18** does not answer it but changes what a disagreement means |
| Q-64 | is a supernatural *capability* with no number, no roll and no waived penalty mechanical? | — | **closed — D8: yes.** Three independent supports, incl. ArMDE:2960-2962's realm association + same-realm-aura Warping immunity on **every** Supernatural Virtue, which falsifies the `narrative` claim for this cohort on grounds independent of the capability argument |
| Q-65 | two canonical tables give `Sense Holiness and Unholiness` different German names | X | open — **D7's three-rule test applies, now with D18's third arm** |
| Q-66 | does `virtue.sense_holiness_and_unholiness`'s "may overwhelm you" state a D5 rule? | N | open |
| Q-67 | what amount should `virtue.simple_student`'s restricted XP pool carry? | R / N | open |
| Q-68 | does Subtle Magic's "no benefits from normal gestures" add anything to the Words/Gestures table? | R | open |
| Q-69 | a table attributes a Reputation rule to `Social Contacts` that ArMDE:4990 does not state | X | open — **D6 rule 1 decides it**; like Q-49, **D18 does not rescue it** (a factual claim, not terminology) |
| Q-70 | should `virtue.sense_passions` carry `tainted: true`? | R | open |
| Q-71 | the German core rulebook gives `virtue.spirit_votary` two names and the tables give a third | X / N | open — **D7 rule 2: a colliding/absent heading is where the glossary wins**, and **D18** makes the third name a possible newer-edition correction rather than a third error |
| Q-72 | what is the "Brother-Priest Status Virtue" that ArMDE:5111 names? | R | open — **if it is a fourth rank, F-34's census is not closed** |
| Q-73 | which `ReputationType` carries an **organization**-scoped Reputation? | — | **closed — D11: none, and the enum does not grow.** ArMDE:1093 names Local, Ecclesiastical and Hermetic, with Academic alongside, and the ease-factor table has exactly those columns. "Among Templars", "among the Jewish community" are **content** |
| Q-74 | how does True Faith's Magic Resistance join the per-Form grid — replace, stack, or compete? | N | open — F-329's *fix* depends on it |
| Q-75 | does ArMDE:2394's "only companions can take this" bind grogs? | N | open — **F-339 depends on it** |
| Q-76 | is a roll-free supernatural *capability* `narrative` or `uncomputed_rule`? | — | **closed — D8: `uncomputed_rule`.** **48** of the 115 `supernatural` entries move (`jq`-verified). Enumerate them **from the data**. Largely a relabelling, not 96 new texts — a `description` is owed only where the `summary` does not already carry the rule. **Blocked behind § 2.1b**: the screen needs a capability idiom first |
| Q-77 | how should a free-text `{land}` placeholder render in German, where the article inflects? | N | open |
| Q-78 | does `virtue.unaging`'s non-fatal-crisis clause have a home in the M6/6b7 crisis engine? | C | open |
| Q-79 | the book files two Flaws under a magnitude their own descriptor contradicts | R | open |
| Q-80 | is `flaw.black_sheep`'s "a bad Reputation **of your choice** at level 2" a wildcard? | — | **closed — D11: already solved.** `GrantsReputation.kind: Option<..>` with `None` *means* player-chosen, and `virtue.famous` (ArMDE:3861) already uses it. `flaw.infamous` pinning `kind: "local"` is the defect (F-450) |
| Q-81 | which `casting_fatigue` sign convention is intended? | C / N | open — **F-351 depends on it** |
| Q-82 | may the German source be used to repair a truncated **English** sentence? | N | open — **F-370 depends on it** |
| Q-83 | is a hedged monetary value a mechanical clause for classification purposes? | N | open |
| Q-84 | what entity or type should hold `flaw.abandoned_apprentice`? | N | open |
| Q-85 | (the `flaw.bound_to_realm` German-name question) | — | **withdrawn — answered from the repository; see withdrawn F-373** |
| Q-86 | should every "choose the specifics" Flaw carry a parameter? | — | **closed — D9: YES, record every stated choice**, `enumerated` where the book lists the options and `text` only where the choice is genuinely open. **Catalogue-wide** — § 3.7's 26 findings are *not* the full set; the slice re-derives it. **Multiplicity is D10's, not D9's** |
| Q-87 | should `flaw.broken_vessel` carry an enumerated `prerequisites` tree? | N | open |
| Q-88 | should a player's free-text choice of *scope* be recorded by a parameter? | — | **closed — D9: yes** |
| Q-89 | ArMDE:5911 says "Technique" inside *Deficient Form*'s own passage — misprint or not? | R | open |
| Q-90 | which of the two Deficient Art entries has the right shape? | — | **closed — D12: neither, and `flaw.deficient_form` is the one missing its gate.** Hermetic V/F split **intrinsic** (operates on The Gift itself — exactly the three entries already gating on `virtue.the_gift`) from **trained** (Techniques, Forms, spells, Casting/Lab Totals, Parma, certámen, Twilight → **magus only**). Obliges classifying all **122** `hermetic` entries — measured: 41 mechanically obvious, 56 carry no effects at all, ~25 need checking — **derived from the batch files, not a re-read**. Use `Prereq::IsMagus`, and normalise `flaw.deficient_technique`'s `Has` spelling to it |
| Q-91 | does ArMDE:5895's third sentence change **D4**'s answer for Cyclic Magic? | N | open — **D4's own slice needs it** |
| Q-92 | how should a *mandatory earmark of the normal budget* be encoded? | — | **closed — D13: add `from_normal_budget` to `RestrictedAbilityXp` and model both clauses exactly.** An engine change for **one entry in 655** (the phrase occurs once in the rulebook) and the cheaper approximation was overruled. **§ 3.17**, and **D14 first** |
| Q-93 | which treatment is intended for the three Corrupted entries? | — | **closed — D15: all three `uncomputed_rule`**, full rule in `description` in both locales, and **`flaw.corrupted_spells`' `special_casting_mod` deleted** — which needs its own test, because a Casting Total silently changes. Unifying *upward* was rejected: Corrupted Abilities' ±3 is on an **Ability roll**, not a Casting Total. Orthogonal to D9, which still owes all three the multi-valued parameter |
| Q-94 | `grants_selection` for `flaw.a_deal_with_the_devil`, and does a granted Story Flaw count toward the cap? | N | open |
| Q-95 | can a parameter express "one or more, your choice"? | — | **closed — D9 part 3: not today, so the model gains a multi-valued parameter type.** The only part of D9 needing a **`SCHEMA_VERSION` bump**; pulls in `ParameterDef`, a migration, a multi-select UI picker and a canonical form for `max_per_target` grouping |
| Q-96 | is the partial `source.anchor` coverage a backfill in progress or an inconsistency? | C / N | open, and **RE-RATED UP by D18**: **94 of 655 (14.4 %)**, and B19 shows the 94 are a by-product of the description-extraction pass, not a principled subset. Since the anchor survives a re-pagination and a line range does not, **backfilling to 100 % is the prerequisite for ever accepting a newer rulebook**. Decide first whether `anchor` is mandatory or dropped; D18 points at mandatory |
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
| Q-108 | does `virtue.lone_redcap`'s 300 XP **replace** the funding for its fifteen apprentice years, or supplement it? | — | **closed — D17: it REPLACES.** Resolved *from the source*, not by choice: ArMDE:4848 and :4321 describe the same 300 for the same fifteen years, and `life_stages.json` already models a 15-year, 240-XP apprenticeship block. **F-439 unblocked and confirmed `high`** (~225 XP over-funded). A second defect fell out: `virtue.redcap` encodes its 300 **not at all**, so the pair is wrong in opposite directions. Remedy is **§ 3.17**'s third mode |
| Q-109 | `flaw.fury` and `virtue.berserk` state the same condition, encoded two contradictory ways | R / C | open |
| Q-110 | ArMDE:6124's two +4 traits contradict ArMDE:2502's ±3 for a Minor Personality Flaw | R | open |
| Q-111 | does `flaw.independent_craftsman`'s recategorization clause apply, given *City and Guild* is not in the repo? | N | open — **provenance: a rule from a book with no source cannot be implemented** |
| Q-112 | does `virtue.lone_redcap` satisfy `flaw.hermetic_patron`'s "a Redcap"? | R / N | open — **F-448's fix depends on it** |
| Q-113 | should an `advancement_mod` marker of `amount: 0` render as a word? | N | open — UI |
| Q-114 | under D6 as corrected, does the glossary or the shipped data win on F-456 and F-457? | N | open — **D7's three-rule test now answers it; confirm** — but run **D18**'s newer-edition arm before concluding the glossary is wrong |
| Q-115 | is `flaw.inscribed_shadow`'s House Criamon restriction a prerequisite to encode? | R / N | open |
| Q-116 | the book defines "combat scores"; the repo's reading excludes three of the five | R / C | open — **overturned the clean verdicts on `flaw.hobbled` and `flaw.lame`** |
| Q-117 | two in-repo authorities give opposite readings to "the passage states the *absence* of a rule" | C / N | open — `NO_RULE_DESPITE_TOKEN` versus `RULES.md:4684` |
| Q-118 | is `flaw.monastic_vows_hermetic`'s "you cannot own vis" mechanical, given the engine models no vis? | N | open |
| Q-119 | is `flaw.necessary_condition`'s "you cannot cast spells at all" a mechanical absolute or a fiction condition? | N | open |
| Q-120 | is an entry computing in both phases `creation_effect` or `in_play_effect`? | — | **closed — the enum's own doc comment: `creation_effect`** |
| Q-121 | four terminology-table rows attribute core-book Flaws to supplements — at what count does "noted" become a finding? | N | open — **F-501 raises the same rows; D6 rule 1 names "from which book"**; **D7 rule 3** makes a book tag decisive for *which heading a row governs* (F-506 is the worked case) |
| Q-122 | should `flaw.magical_fascination`'s authorization name both Lores, or record the player's choice? | — | **closed — D9: record the choice.** **D14 supplies the missing half**: an ability reference in an effect gains a **binding** to the selecting entry's own parameter, which is exactly this shape (`virtue.student_of_realm` is its sibling defect) |
| Q-123 | two entries state a *soft* restriction on magi, and the engine has only hard blocks | — | **closed — D16: emit a WARNING, never an error.** No new machinery — `IssueSeverity::Warning` / `ValidationIssue::warning` exist (`validation/mod.rs:73`, `:844`). A `forbidden_traits` row would say something the book (*"normally"*, *"not suitable"*) does not, which is the class of error `7f5605a` was reverted for. **Apply the hedged-versus-absolute rule wherever the book hedges, rather than escalating each instance** |
| Q-124 | this span carries both `source.lines` conventions | C / N | open — **Q-96 family** |
| Q-125 | the rulebook contradicts itself about `flaw.prohibition`'s category | R | open |
| Q-126 | is an entry whose only mechanical content is a *selection restriction* mechanical? | — | **closed — D8: yes** |
| Q-127 | a granted Reputation's `score` cannot hold the range ArMDE:6554 gives | — | **closed — D11: add `max_score: Option<u8>`.** Absent = exact, present = the score must lie in `[score, max_score]`. **31 entries grant a Reputation; exactly one states a range** (`flaw.outsider_*`, ArMDE:6554), so `skip_serializing_if` leaves 30 entries' JSON unchanged. Both magnitudes get `score: 1, max_score: 3` |
| Q-128 | should `flaw.plagued_by_supernatural_entity` carry an `entity` parameter? | — | **closed — D9: yes** |
| Q-129 | which German name is canonical for Primogeniture Lineage and Palsied Hands? | — | **closed — D7 (2026-09-21): the DE rulebook heading wins; all three shipped names are correct** |
| Q-130 | should a Rigid Magic magus be blocked from *selecting* a Ritual spell at creation? | N | open — B17. `SpellDef::ritual: bool` exists, so the engine *could*; but ArMDE:6697 forbids **casting**, not knowing, and a magus may have learned a Ritual before acquiring the Flaw. Turns on whether spell selection models "spells known" or "spells usable" |
| Q-131 | which catalogue Ability is "Covenant Lore"? | R | open — B17. The phrase occurs **exactly once** in the English core book (ArMDE:6793) and there is no `ability.covenant_lore`; DE's *Konventskunde* is equally absent. Candidates: `ability.area_lore`, `ability.organization_lore`. **F-520 is blocked on it**; an agent can settle it by reading the Abilities chapter |
| Q-132 | should `Prereq` gain a variant that ranges over a `categories` value? | N | open — B17. `flaw.rector` requires "a Social Status Virtue" — **99** entries carry it, so an enumerated `Prereq::Any` violates CLAUDE.md's catalogue-size invariant, and description-only enforces nothing. **Decide with F-427 / Q-07 / Q-102**, which need the same machinery for ArMDE:2816, and with **F-542 / F-355**, which need its Effect-side twin |
| Q-133 | does `flaw.short_ranged_magic`'s Lab Total halving belong in `spell_level_cap`? | N | open — B17. **D1 does not reach it**: the Flaw carries no `lab_total_mod` but a *halving*, `HalvableTotal` has no member for it, and D1's scope paragraph is explicit that it governs `spell_level_cap` "and nothing else". Note D1's *generous* reasoning does **not** carry over — applying it here would **lower** the cap. Sibling: should `flaw.savantism`'s "no Ability above 3" run through `AgeAbilityCaps` / `validate_ability_age_cap`? |
| Q-134 | the unrecorded-choice family, six named instances | — | **closed — D9** (it is the Q-86/Q-88/Q-95 family and B17 filed it as instances, not a new question). The six: `flaw.restriction`, `flaw.supernatural_nuisance`, `flaw.repellent`, `flaw.rector`, `flaw.savantism` (B17) and `flaw.vulnerable_magic` (B19). **`flaw.repellent` is materially different and must be weighed separately when the parameter shapes are chosen** — its "minor advantage" can be a **+3 Soak**, a number belonging on the sheet, where the other five are narrative labels |
| Q-135 | is `flaw.seeker` magus-only, and **does a rule in another book bind an entry that cites ArMDE?** | N | open — B17; **`flaw.seeker` is ESCALATED and NOT marked checked.** The entry's own pointer resolves to HoH:TL:503, *"A magus from any House may be a Seeker"* — which both *removes* a House restriction and presupposes a magus, while ArMDE:6713-6716 never says "only magi". **Question 2 is the policy call and has consequences far beyond this entry** (§ 3.1's second blind spot already records F-442/F-452/F-453/F-478 of the shape "a rule the book states elsewhere about a named entry is structurally invisible"; this is the first where "elsewhere" is another **book**). If cross-book rules bind, a **second citation** must be recorded — which `source`, a single `SourceRef`, cannot hold |
| Q-136 | does a **surfaced-only** effect kind satisfy `in_play_effect`, or is `uncomputed_rule` the honest class? | N | open — B18; **`flaw.susceptibility_to_divine_power` is ESCALATED and NOT marked checked.** It ships `in_play_effect` with `doubled_aura_penalty`, one of eight surfaced-only kinds pushed with `amount: 0`, and `RULES.md:4882-4888` defends it in words that describe an `uncomputed_rule` (*"surfaced, not simulated"*). **B17's F-515 set the opposite precedent one batch earlier** by leaving `flaw.short_ranged_magic` `in_play_effect`. **The ruling governs 19 catalogue entries** — re-counted by B18's verification pass from `derived.rs::in_play_mods`' own `match`, correcting an earlier "at least twelve", which was the count of *effect kinds*, not entries |
| Q-137 | `incompatible_with` holds ids; ArMDE:6925 states a **predicate** | N | open — B18, see F-526. "Any other Flaw that grants a Bad Reputation" = sixteen Flaws today. **(a)** ship the ids plus a test asserting the list equals the derived set, or **(b)** add a predicate-valued exclusion. **Not equivalent under CLAUDE.md**: (a) freezes a *number* of entries into data and is tolerable only because the test regenerates it. `virtue.doctor_in_faculty` as a prerequisite is **not** blocked on this |
| Q-138 | ArMDE:6148 forbids a **class** of Flaws, and it sits on another batch's entry | R then N | open — B18. *"Any Flaw that is only appropriate to Hermetic Magic (for example, Deficient Technique or Unstructured Caster) cannot be taken with this Flaw."* Inside **B13**'s span and carried by no finding in B13, B17 or this index. Harder than Q-137: `categories: ["hermetic"]` includes Flaws that are not *only* Hermetic, so the data does not cleanly enumerate it. **Recorded so B13's entry is not left silently unrated** |
| Q-139 | does "generally restricted" warrant a hard prerequisite? | — | **closed — D16.** ArMDE:6957's *"is **generally** restricted to magi of House Verditius"* is hedged, and D16's table maps hedged → **warning, never an error**, with the standing instruction to *"apply this wherever the book hedges, rather than escalating each instance"*. **F-533's magus half was never hedged** and needs no ruling. Q-13, which B18 called adjacent, is separately closed by D2 |
| Q-140 | Weak Parens / Skilled Parens have no `IsMagus` prereq, and a **Gifted companion** can reach them | N | **SETTLED — D24.** Both gain `Prereq::IsMagus` (they are *trained* under D12, since :7074's 180/90 are apprenticeship figures), and the companion profile's Gift-conditional `hermetic` permission **stays**: The Gift is necessary, not sufficient (ArMDE:2880 states both halves; ArMDE:2840 forbids dropping the first). **Blocked on D12**, whose pass covers the other 120 `hermetic` entries. Original escalation follows. — B19; **both entries NOT marked checked.** `categories: ["hermetic"]` is not a magus gate — the companion profile permits it conditionally on `virtue.the_gift`. What follows is not an error but a **silent budget change**: `general_pool_and_bonus` builds the base from apprenticeship + post-gauntlet **only when `is_magus`**, otherwise from `later_life_xp`, and the −60 is folded in unconditionally. Three readings are defensible; reading 2 (add `is_magus` to both) has the strongest textual support, since :7074's 180/90 are *apprenticeship* figures. **Whatever the answer, it applies identically to `virtue.skilled_parens`**, which is out of span and appears in no batch record |
| Q-141 | the book indexes Weak Personality under *Story, Minor*; its own descriptor says *Personality* | N | **SETTLED — D25.** `["personality"]` stands; the disagreement is recorded as a **known source erratum** in `crates/arm-rules/RULES.md` (both citations, side taken) — **no new register**. Two obligations: sweep the other 630 entries against the book's index lists once and state the result, and treat that sweep as a human-read comparison, never a guard (link hygiene is unreliable — 134 dead link targets). Original escalation follows. — B19. Descriptor (ArMDE:7077) and DE :7077 agree on *Personality* and the data follows them, so on the narrow question the data is right and **the book is internally inconsistent**. The consequence is user-facing and CLAUDE.md rates it up: a reader working from the book's Story list filters the app by Story and the entry is not there. Options: leave it; ship `["personality","story"]` (asserts a category the descriptor denies); or **record it as a known source erratum**, which B19 recommends — but **the audit has no errata register today**, and creating one is part of the decision. **The only descriptor/index disagreement in B19's 25** |
| Q-142 | does the `scheitest` precedent cover a string containing **both** `guarter` and `quarter`? | N | open — B19; **`flaw.waster_of_vis` NOT marked checked on text.** The EN `summary`/`description` reproduce ArMDE:7054's `guarter` faithfully — and the same `q`→`g` OCR error recurs at ArMDE:3502 (`virtue.berserk`), confirming it is systematic in the source, not a transcription slip. German is unaffected (DE :7054 reads *Viertel*). **What is new is that one shipped string spells the word both ways six clauses apart**, giving the reader no way to know which is meant — a comprehension defect a faithful-copy rule was probably not written to protect. Either the precedent holds and the typo stays, or self-contradicting strings are carved out — in which case **both shipped strings and `rules/source/en/…:7054` and `:3502` are corrected together**, since the JSON is generated from the Markdown |

### 6.1 The Phase-0 questions — all three now closed

`README.md:138-147` lists three Phase-0 questions as unread. **All three are
answered and the README has not been updated.**

1. **Major vs Minor Magical Focus arithmetic** — read and answered inside B06
   (*"The two directed reads — both answered"*).
2. **Is a granted Reputation's `score` a ceiling, a fixed value or a
   suggestion?** — read inside B03 (*"Directed read — README's open question
   3"*), and now **ruled by D11**: **exact by default, with an optional
   `max_score` upper bound for the one entry whose passage states a range.**
   ArMDE:2514 settles only *entitlement* ("characters only start with a
   Reputation if they choose a Virtue or Flaw that grants one") and says nothing
   about the number, so the semantics are per-entry from each granting passage —
   and those passages were surveyed rather than assumed: **31 grant a Reputation,
   exactly one states a range.**
3. **D2** — ~~genuinely pending~~ **ruled 2026-09-21**; see § 2.2. Both
   validators become grant-aware.

### 6.2 The ones that still need Norbert, ordered by how much they unblock

He works from this list, so it is ordered by consequence rather than by number.
**Rewritten 2026-09-21:** the previous list's items 1-8 have all been ruled on,
so what follows is what is *left*, not what was there before.

1. **Q-136** — does a **surfaced-only** effect kind satisfy `in_play_effect`?
   **19 catalogue entries**, and two batches have already set opposite
   precedents (B17's F-515 versus B18's escalation), so deciding it silently
   overturns or entrenches one of them. **The largest single open question in the
   audit, and the only one two batches have disagreed about in practice.**
2. **Q-132 (+ Q-07 / Q-102 / F-427, and F-542 / F-355 on the Effect side)** —
   should something range over a `categories` value? It appears on four
   independent axes — a prerequisite (`flaw.rector`), a global rule
   (ArMDE:2816's one Social Status), an Ability-category prohibition (F-355,
   F-410) and a V/F-category prohibition (F-542). **Decide once; the alternative
   is four mechanisms for one idea.**
3. **Q-135** — **does a rule in another book bind an entry that cites ArMDE?** A
   provenance policy call with consequences far beyond `flaw.seeker`, and one
   whose "yes" answer needs a data-model change (`source` is a single
   `SourceRef` and cannot hold a second citation).
4. **Q-137 / Q-138** — `incompatible_with` holds ids where the book states a
   predicate. Q-138 is the harder of the two and sits unrated on B13's entry.
5. **Q-142** — the last of B19's three escalations, holding
   `flaw.waster_of_vis` at **not marked checked** on text: whether the
   `scheitest` precedent covers a string that spells one word two ways.
   (**Q-140 is settled — D24; Q-141 — D25.**)
6. **Q-130 / Q-133** — two questions about how far an existing mechanism reaches:
   whether spell *selection* models knowing or using, and whether D1's generous
   flat reading extends from `lab_total_mod` to a *halving*. **Note that D1's own
   reasoning does not carry over to Q-133** — applying it there would lower the
   cap, not raise it.
7. **Q-96 (+ Q-124)** — is `source.anchor` mandatory or dropped? **Re-rated up by
   D18**: at 14.4 % coverage it is the prerequisite for ever accepting a newer
   rulebook.
8. **Q-10 / Q-40 / Q-49 / Q-65 / Q-69 / Q-71 / Q-114 / Q-121** — all need
   `arm-de-translation`, which no agent in this repository can open, **and all
   now need D18's three-way test**, which only Norbert can run because it
   requires the current German edition. The 21 disagreements are listed in
   `tmp/table-sync-check.md`. **Do not auto-revert them to the heading.**
9. **Q-131** — the identity of "Covenant Lore". **Not Norbert's**: it is an
   **R**, an agent can settle it from the Abilities chapter, and **F-520 is
   blocked until someone does.** The cheapest open item in the audit.

---

## 7. Counts

### 7.1 Method — read this before quoting a number

Three earlier agents in this audit shipped wrong census figures, one exactly
backwards; **B18's verification pass then found eight wrong figures where its own
batch had predicted two**. So the method is stated per figure rather than assumed,
and **every figure below was verified with a different command from the one that
produced it.**

Every number is derived **mechanically from § 1 of this file**, not recounted by
eye, using the section's own line range (118-767) and its column positions:

```
sed -n '118,767p' corrections.md | grep -a "^| F-" | cut -d'|' -f5 | ...
```

That restriction matters: eight later sections now contain tables whose rows also
begin `| F-`, and counting the whole file returns **569** instead of 544.

**Column positions**, so a later reader can re-derive any figure: `f2` = finding
number, `f3` = entry, `f4` = ArMDE, `f5` = kind, `f6` = severity, `f7` = status,
`f8` = summary. **Filter on `f7`, never on the whole line** — a summary containing
the word "unblocked" or "blocked" silently corrupts a status filter, which it did
on the first attempt here.

**Four cross-checks agree on the headline figure.**

1. `grep -c "^### F-"` over all nineteen batch files returns **544** finding
   headings. Per batch: 33, 31, 24, 32, 39, 48, 49, 51, 40, 35, 24, 20, 17, 15,
   21, 21, **21, 16, 7** — which sums to 544.
2. `grep -o "^### F-[0-9]*" | sort -u | wc -l` over the same files also returns
   **544**, so no heading re-uses a number.
3. § 1 of this file has exactly **544** rows, and **544 distinct** finding numbers
   (`cut`ing the number column and `sort -u`-ing it gives the same figure, so
   there are no accidental duplicate rows).
4. Finding numbers run F-001 … F-545 with **one gap, F-44**, which
   `batch-02.md:4` and `:534` both record as never issued. 545 − 1 = 544.

**Every kind and severity tally sums to 544**, and each is shown summing below.
Where a figure is approximate, it says so.

### 7.2 Findings by status

Derived from `cut -d'|' -f7` over § 1's rows; cross-checked by subtracting the
non-`live` rows from 544.

| Status | Count |
|---|---|
| `live` | **526** (includes F-482, which is live but partly duplicates F-11 and F-100, and **F-439, unblocked by D17**) |
| `withdrawn` | **13** — 3 by their own batch (F-299, F-373, F-481), 9 as duplicates found by **this index** (§ 4.3), 1 settled by D7 (F-496's name question). **B17-B19 add none** |
| `blocked` | **4** — F-85 (Q-10), F-270 and F-283 (a catalogue decision), **F-520 (Q-131, new)**. **F-439 left this list** when D17 answered Q-108 |
| closed by this file | **1** — F-499 |
| **Total** | **544** |

**The blocked count is unchanged at 4 by coincidence, not by cancellation** — one
row left and a different one arrived. Do not read "still 4" as "nothing moved".

**D2 blocks no individual finding**, but it blocks part of § 3.6 and is the
audit's one genuinely pending ruling.

### 7.3 Findings by kind of defect — **primary kind**

Sums to 544. Derived by `cut -d'|' -f5 | cut -d'+' -f1` over § 1's rows, so a
compound `class+desc` counts here only under `class`. **Cross-checked** by
summing the column (544) and by the per-batch delta: B17-B19 contribute exactly
44, distributed `class` +23, `desc` +6, `prereq` +4, `engine` +3, `text` +2,
`param` +2, `incompat` +2, `auth` +1, `src` +1 — which itself sums to 44.

| Kind | Count | vs B01-B16 |
|---|---|---|
| `class` — misclassification | **166** | +23 |
| `desc` — missing description (D5) | **116** | +6 |
| `auth` — missing authorization | **39** | +1 |
| `text` — shipped text / localization | **36** | +2 |
| `prereq` — missing prerequisite | **34** | +4 |
| `param` — missing/wrong parameter | **29** | +2 |
| `incompat` — missing incompatibility | **27** | +2 |
| `scope` — wrong scope | **24** | ±0 |
| `effect` — missing effect | **19** | ±0 |
| `table` — translation-table defect | **13** | ±0 |
| `engine` — engine gap | **10** | +3 |
| `rep` — Reputation defect | **9** | ±0 |
| `number` — wrong number/arithmetic | **7** | ±0 |
| `src` — `source.lines` / anchor | **6** | +1 |
| `prov` — provenance | **5** | ±0 |
| `mag` — magnitude/category/kind | **2** | ±0 |
| `audit` — audit-record defect | **2** | ±0 |
| **Total** | **544** | +44 |

**The last three batches are the most `class`-heavy in the audit**, and that is a
fact about the span rather than about the reviewers: **more than half** of
B17-B19's findings are misclassifications, against 29 % over B01-B16. The span is
the tail of the Flaws block, where `narrative` was the authoring default and the
swept-block screen has been green over it twice.

**Note the two kinds that did NOT move.** `scope` and `number` gained nothing in
95 entries. Scope errors cluster on *computed* entries, and this span computes
almost nothing — which is the same fact as the `class` spike seen from the other
side.

### 7.4 Findings by kind — **primary *or* secondary**

A finding spanning two kinds is counted under both, so this does **not** sum to
544. It is the better guide to slice size: § 3.2 is a 67-finding slice, not a
39-finding one. Derived by `cut -d'|' -f5 | tr '+' '\n'` over § 1's rows;
cross-checked against § 7.3 — every row here must be ≥ its primary count, and
the column total (**694**) minus 544 gives **150 secondary kinds**, which is
exactly the number of `+` characters in the kind column (144 rows carry a `+`;
six of them carry two).

| Kind | Count | vs primary |
|---|---|---|
| `class` | 169 | +3 |
| `desc` | **154** | +38 |
| `auth` | **67** | +28 |
| `prereq` | 40 | +6 |
| `param` | 39 | +10 |
| `incompat` | 39 | +12 |
| `text` | 38 | +2 |
| `effect` | 28 | +9 |
| `scope` | 26 | +2 |
| `engine` | **24** | +14 |
| `number` | 15 | +8 |
| `table` | 14 | +1 |
| `prov` | 13 | +8 |
| `rep` | 11 | +2 |
| `src` | 7 | +1 |
| `audit` | 7 | +5 |
| `mag` | 3 | +1 |

**The headline, and it has sharpened.** Three kinds are **speaking for 59 % of
the audit by primary kind** — `class` (166), `desc` (116) and `auth` (39) make
**321 of 544**. Under D5 the first two are one job per entry, and § 2.1 says
neither can start until `MECHANICAL_PHRASES` grows. **The screen fix is the
critical path for well over half the correction list**, and § 2.1b adds a
requirement to it that nobody has enumerated yet.

**`engine` more than trebled between the two columns (10 → 24)**, which is the
clearest signal in this table: engine gaps are almost never the *primary*
complaint, because a batch reports the entry it found, not the machinery it
needs. § 3.15 is therefore a much larger body of work than a primary-kind count
suggests — and D9 part 3, D13, D14 and F-542's category variant are all *rulings*
that appear in no finding's kind column at all.

### 7.5 Findings by severity

Normalized per § 0.2; for a compound severity I took the **first-named**
component, which is what the batches lead with. **These are derived by me from two
different batch vocabularies and are not quoted figures.**

B17-B19 used the three-point scale, plus `MEDIUM-HIGH` (→ `H`) and `LOW-MEDIUM`
(→ `M`), so § 0.2's table covers them unchanged. **They stated a severity on every
one of their 44 findings**, which is why the `n/s` column does not move.

| Severity | All 544 | Live only (526) | B17-B19's 44 |
|---|---|---|---|
| `H` | **202** | **195** | **11** |
| `M` | **288** | **281** | **29** |
| `L` | **30** | **30** | **4** |
| `n/s` — the batch stated none | **21** | **20** | **0** |
| — (withdrawn, no severity) | **3** | 0 | 0 |
| **Total** | **544** | **526** | **44** |

Both totals verified two ways: the column sums, and `544 − 13 withdrawn − 4
blocked − 1 closed = 526`. The `n/s` figure is **20 + 1** — F-403 carries the
compound `n/s (table row: L)` and is counted once, in `n/s`.

**H went from 190 to 202 (+12, not +11)** because **F-439** moved out of the
`H/nil` bucket when D17 resolved Q-108 toward "replaces". That is the one
severity in this file changed by a ruling rather than by a new finding.

`n/s` is concentrated in **B11 (16 of 24)** and **B12 (3 of 20)**, which close a
finding with a `**Verdict:**` line instead. It is not a gap in this index.

**The `H` rate is stable across the audit** — 190/500 = 38 % over B01-B16,
11/44 = 25 % over B17-B19. The last three batches skew `M` because their dominant
defect is misclassification, which § 0.2 normalizes to `M`, not because they found
less.

### 7.6 Other counts

| Thing | Count | How derived |
|---|---|---|
| Batches indexed | **19** of 19 — **the checking pass is complete** | `ls batch-*.md` = 19; § 1 carries a block per batch |
| Catalogue entries checked | **655** of 655 | B19's closure check (a): `jq 'length'` over `rules/core/virtues_flaws.json` = **655**, and `grep -c '"entity_kinds"'` over the same file independently = **655**. 630 + B19's 25 = 655 |
| — *not* marked checked | **5** entries | escalations: `flaw.seeker` (Q-135), `flaw.susceptibility_to_divine_power` (Q-136), `flaw.weak_parens` (Q-140), `flaw.weak_personality` (Q-141), `flaw.waster_of_vis` (Q-142, text only). **Checked ≠ clean and checked ≠ settled** |
| Findings | **544** (F-001…F-545, no F-44) | four agreeing cross-checks — § 7.1 |
| Open questions | **142** (Q-01…Q-142) | `grep -c "^| Q-"` over § 6 = 142; `sort -u` on the id column also = 142 |
| — closed / withdrawn | **34** | `cut -f5 \| grep -c "closed\|withdrawn"` = 34, which equals the count of `—` in the settler column, so the two encodings agree |
| — open | **108** | 142 − 34; and the open settler codes sum to 108 independently |
| — needing **Norbert** (`N`, `R/N`, `C/N`, `X/N`, `R then N`) | **80** | 64 + 8 + 5 + 2 + 1 |
| — settleable by an agent (`R`, `C`, `R/C`) | **24** | 18 + 3 + 3 |
| — needing `arm-de-translation` (`X`, `X/N`) | **6** rows | 4 + 2; more when the `N`-tagged DE ones are included, and **all of them now need D18's current-edition arm too** |
| Rulings in `decisions.md` | **18** (D1…D18), **all taken** | `grep -c "^## D"` = 18, `sort -u` also 18. **D2 is no longer pending** |
| — taken after this file was first written | **10** (D8-D18, less D7) | they close 25 questions — § 6 |
| Duplicate relationships found | **10** (9 by this index, 1 already withdrawn by B16) | **unchanged: F-502…F-545 add zero**, by the seven-pass method in § 4.2 |
| Site-level overlaps inside an umbrella | **12 clusters** (§ 4.4) | 5 from B01-B16, **7 declared by B17-B19** |
| `MECHANICAL_PHRASES` additions | **14 idiom families, ~94 word-forms** across both locales, **plus 3 structural fixes** and **one family not yet enumerated** | the families are counted from § 3.1's table; the third structural fix is F-514's `SIGN_CHARS` gap |
| Entries D8 reclassifies with no finding number | **48** | `jq` over `rules/core/`: 48 of 115 `supernatural` entries are `narrative` |
| Entries D12 obliges classifying | **122** | all `hermetic`-category entries; measured as 41 mechanically obvious + 56 with no effects + ~25 needing a check |
| Entries D10 obliges declaring | **~42** | `jq`: `parameters != null and max_total == null` |
| Entries Q-136 governs | **19** | re-counted from `derived.rs::in_play_mods`' own `match`, correcting an earlier "twelve" that was a count of *effect kinds* |
| `source.anchor` coverage | **94 of 655 (14.4 %)** | B19's closure note (d)(1); **D18 makes 100 % a prerequisite** |

**The `MECHANICAL_PHRASES` figure is approximate on the word-form count** — the
families are the actionable unit and some rows give a stem rather than a literal.
14 families and 3 structural fixes are exact. **The capability family D8 needs
(§ 2.1b) is NOT in the 14** and has no word-form count at all yet; that is a
stated gap, not an omission.

**Two figures deliberately have no number here.** The count of *catalogue entries*
affected (§ 7.7) and the size of the D9/D10 data pass, which both rulings say must
be re-derived catalogue-wide rather than taken from a finding list.

### 7.7 What this file does *not* count

- **No count of affected catalogue entries.** Several findings cover two or more
  entries (F-543 covers four, **F-544 covers seven**), several entries carry five
  findings, and the umbrella findings (F-340, F-409) name sites that are not
  enumerated anywhere. Deriving an entry count from this table would be a guess.
- **No estimate of Phase 2 effort.** The groups in § 3 are sized by finding count,
  which is not the same thing — and § 7.4 shows why that gap widened: `engine`
  more than trebles between primary and secondary, and four of the largest pieces
  of work (D9 part 3, D10, D13/D17, D14) carry **no finding number at all**.
- **No count of the entries the rulings reach.** D8's **48**, D12's **122**,
  D10's **~42** and Q-136's **19** are listed in § 7.6 as *figures the rulings
  state*, and every one of them says to re-derive the set from the data rather
  than from a list. **None of those entries has a finding row in § 1**, so § 1's
  544 is not a measure of how much data Phase 2 touches.
- **No re-rating.** Where a batch's severity looks wrong to me I left it and said
  so in § 0.2 rather than adjusting it silently. The one exception is **F-439**,
  whose severity was written as a conditional (`H/nil`) pending Q-108 and is now
  `H` because **D17 answered the condition** — that is a resolution, not a
  re-rating.
- **No verdict on the five escalated entries.** They are not marked checked, and
  this file records the question rather than guessing the answer.
