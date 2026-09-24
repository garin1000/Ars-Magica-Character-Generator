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

**Coverage — THE CHECKING PASS IS COMPLETE.** Findings **F-001 … F-552** (551 of
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

**Updated 2026-09-22.** § 6's 31 evidence-settleable questions are **answered**
(`docs/vf-audit/q-resolutions.md`), so their rows carry a verdict instead of
"open"; § 1 gains **seven** findings those answers turned up (**F-546 … F-552**,
one of them already **fixed** by D32 and committed); § 7 is recounted from
scratch. **Nothing was renumbered.** The seven new rows are in their own block at
the end of § 1, because they belong to no batch.

**§ 8 is new and Phase 2 must not start without it.** The rulings had accumulated
**33** separate instructions to sweep, re-derive or measure something, spread
across 48 decisions with nothing collecting them — the same shape of gap that
once made B16 duplicate B09's work. § 8 lists all of them. **Every figure a
ruling quotes is a snapshot of one span, not a catalogue census**; the row says
what to measure and why the stated number is not enough.

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
| `fixed` | **Already corrected and committed**, with the ruling and the commits named in the row. Recorded so the defect is findable and so nobody re-files or re-works it; it is **not** Phase 2 work. Added 2026-09-22 for F-546 (D32). |

A row may additionally carry `dup→F-NNN`, meaning **this index** judges it the
same defect as an earlier finding. Where the duplication is exact the status is
`withdrawn`; where it is partial the status stays `live` and the marker warns
that the two must be worked once. See § 4.

`n/s` in the severity column means **the batch stated no severity** for that
finding. It is not a judgement of mine; B11 and B12 frequently close a finding
with a `**Verdict:**` line and no severity.

---

## 1. Master index — F-001 … F-552

Sorted by finding number, 551 rows — 544 from the nineteen batches, plus the
seven in the `q-resolutions.md` block at the end. `ArMDE` is the entry's `source.lines` range as
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
| F-65 | `virtue.curse_throwing`, `virtue.dowsing` | 3625-3628, 3703-3706 | text | M | live | German summaries render *Ability* as "Fähigkeit"; canonical German is "Fertigkeit". **D36**: the term binds here because the sentence names a game element the app shows |

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
| F-74 | `virtue.forge_companion` | 3925-3928 | scope | H | live | the 50-point pool funds any Craft; the book names the master's Crafts. **This is a D48 instance, and the fourth known** — B06 found three in its own span and did not have this one, so § 8 row 12's sweep starts from four. Remedy: the `instances` field D48 adds, with the union semantics. **Q-21** settles this entry's other two rules |
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
| F-91 | `virtue.good_teacher` | 3971-3974 | scope | H | live | the `book` modifier points at the wrong person. **Remedy changed by Q-32**: replace it with a new **authoring** `AdvancementSource`, do **not** drop it for text — `flaw.incomprehensible` (ArMDE:6296, *"or from a book you have written"*) is a second caller, so the variant is warranted |
| F-92 | `virtue.gorgiastic` | 3979-3982 | class | M | live | `narrative` on a hard score cap |
| F-93 | `virtue.gossip` | 3983-3986 | class | M | live | `narrative` on a doubling and a target number |
| F-94 | `virtue.greater_immunity` | 4009-4016 | param | H | live | the book demands a choice, nothing is recorded, and the same choice may be taken 255 times |
| F-95 | `virtue.greater_immunity` | 4009-4016 | class | M | live | `narrative` on a complete immunity |
| F-96 | `virtue.greater_power` | 4017-4026 | desc | M | live | one `power_levels` row, and four rules stated around it reach nobody (D5) |
| F-97 | `virtue.greater_purifying_touch` | 4027-4030 | class | M | live | `narrative` on a Fatigue cost and a one-disease limit |
| F-98 | `virtue.greater_purifying_touch` | 4027-4030 | param | H | live | "You must choose the disease … when you take this Virtue" and nothing records it |
| F-99 | `virtue.guardian_angel` | 4031-4036 | class | M | live | `narrative` on "+5 bonus to Soak" and "a Magic Resistance of 15". **Remedy fixed by Q-28: `uncomputed_rule` + text in both locales, and NO `soak_mod` or resistance effect** — ArMDE:4035 gates both on *"acting in accordance with God's will"*, and encoding either flat would repeat F-20's HIGH defect |
| F-100 | `virtue.guild_apprentice` | 4041-4044 | class+incompat | M | live | `narrative` on a clause that switches Wealthy and Poor off *(F-482 re-found this site)*. **Remedy fixed by D47 (revised)**: **encode the suppression**, keyed on holding this Virtue — `virtue.journeyman` exists, so with D41's one-Social-Status rule the book's *"until journeyman"* is simply *"while he holds this"*. Plus description in both locales, and **no `incompatible_with` on any of the three entries** — the `incompat` half is answered *against* encoding |
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
| F-116 | `virtue.inoffensive_to_beings` | 4133-4142 | mag | L | **withdrawn (Q-30)** | `hermetic` is a descriptor membership category, filed as `index_categories` — **and that is correct, deliberate and documented**. `RULES.md` records Norbert's 2026-09-13 ruling (an *and*-joined descriptor means *either route*) and the reason `hermetic` must stay out of `categories`: `gift_categories: ["hermetic"]` would make `effective::has_the_gift` read the Magical Air route as possession of The Gift. Not a defect |
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
| F-218 | `virtue.perfectus` | 4632-4641 | desc | M | live | the Purity/Transcendence permission reaches neither locale (D5). **This finding is also D22's visibility record for the cross-book debt (Q-56)**: the two Abilities are **not** added to the catalogue — all 78 are core-book and D22 keeps the implementation core-book-only — so the **text is the whole remedy**, and it must name the source book as the passage does |
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
| F-234 | `virtue.privileged_upbringing` | 4806-4808 | scope | H | live | the engine grants exactly the permission the passage forbids. **Unblocked by D43** (via Q-59, which is Q-27 under another entry): the pool permits spending itself and nothing more. **Work it with F-102, F-18 and the other 28 pool carriers in one slice** — narrowing alone removes legitimate access |
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
| F-320 | `virtue.tough` | 5145-5147 | text | M | live | the German summary calls Soak "Widerstandsfähigkeit" against the canonical table and the DE rulebook. **D36** applies (it names a displayed score); which term is canonical is **D31**'s question |
| F-321 | `virtue.town_magistrate` | 5149-5152 | class | M | live | a Social Status with three mechanical clauses classified `narrative` |
| F-322 | `virtue.town_magistrate` | 5149-5152 | prereq | H | live | a prerequisite the `Prereq` tree expresses exactly, and that is absent |
| F-323 | `virtue.town_magistrate`, `virtue.university_grammar_teacher` | 5149-5152, 5195-5198 | auth | H | live | Academic permissions with no `ability_authorization` |
| F-324 | `virtue.trained_assassin` | 5153-5156 | prereq | H | live | the Nizari Social-Status prerequisite is absent, though `Prereq::Any` expresses it exactly |
| F-325 | `virtue.turb_trained` | 5179-5182 | class+auth+incompat | H | live | a Martial permission, a single dead language, and a Wealthy/Poor exclusion, all under `narrative` *(F-340 family)* |
| F-326 | `virtue.troubadour` | 5157-5164 | class+auth | H | live | an Academic permission with no `ability_authorization`, under `narrative` |
| F-327 | `virtue.troupe_upbringing` | 5165-5168 | class | M | live | a "+2 modifier" under a `narrative` class |
| F-328 | `virtue.troupe_upbringing` | 5165-5168 | param | M | live | no parameter records which area was selected |
| F-329 | `virtue.true_faith` | 5169-5172 | effect+engine | H | live | the book gives a Magic Resistance formula and the engine computes nothing from the score. **Unblocked by D37**: fold `effective/might.rs::true_faith × 10` into `derived/casting.rs::magic_resistance` as a **floor across every Form**, never an addend. Consumer change only — the score is already modelled |
| F-330 | `virtue.true_faith` | 5169-5172 | desc | M | live | the granted Faith Point, the spend rule and the refresh rule reach neither locale (D5, D3) |
| F-331 | `virtue.unbound_tongue` | 5191-5194 | class | M | live | the waiver of a -10 casting penalty, classified `narrative` |
| F-332 | `virtue.unaging` | 5187-5190 | desc | M | live | the crisis clause and the Decrepitude-4 waiver reach neither locale (D5, D3) |
| F-333 | `virtue.true_love_pc`, `virtue.venus_blessing` | 5173-5178, 5211-5214 | class | M | live | two "+3" Virtues classified `narrative` |
| F-334 | `virtue.true_love_pc` | 5173-5178 | prereq+engine | M | live | the reciprocity requirement is encoded nowhere and cannot be (D3) |
| F-335 | `virtue.unaffected_by_the_gift` | 5183-5186 | class | M | live | immunity to a numeric social penalty, classified `narrative` |
| F-336 | `virtue.variable_power` | 5199-5206 | class+desc | M | live | a scaling formula and four variables, classified `narrative` *(re-found as F-413)* |
| F-337 | `virtue.verditius_magic` | 5215-5217 | class | L | live | `narrative` on an entry whose two rules the engine already computes |
| F-338 | `virtue.verditius_magic` | 5215-5217 | desc | M | live | the entire Outer Mystery behind "(see page 240)" reaches neither locale (D5) |
| F-339 | `virtue.wealthy` | 5235-5238 | prereq | H | live | the companion-only rule is enforced for magi and mythic companions and not for grogs. **D38**: fix it on the **entry** (a character-type `Prereq`), not by adding a third `forbidden_traits` list; the two existing lists then come out. Applies identically to `flaw.poor` |
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
| F-349 | `virtue.wise_one` | 5257-5260 | class+auth | H | live | a two-category gated Ability permission with no `ability_authorization`, and `narrative` denying it. **⚠ The permission is EXCLUSIVE, and the obvious fix makes it worse (added 2026-09-24).** ArMDE:5259 reads *"either Arcane or Academic Abilities, **but not both**"*, while `Effect::AbilityAuthorization` is purely **additive** — it can grant a permission and cannot restrict one. So authorizing both categories converts an under-permission into an **over-permission**, which the concurrent session confirmed on the book's own Witch, who holds Wise One and buys from both groups uncomplainingly. Needs an **exclusive-choice** authorization — the same machinery `virtue.custos` requires under `open-todos.md` row 45, so treat them as one job. The engine half is row 53; **this entry still owes its text either way** (D3/D50) |
| F-350 | `virtue.wise_one` | 5257-5260 | desc | M | live | three clauses reach neither locale (D5) |
| F-351 | `virtue.withstand_casting` | 5261-5282 | number | M | live | the `casting_fatigue` sign is the opposite of the engine's documented convention and of both Flaw carriers'. **Q-81 answers it and REVERSES the remedy** (`q-resolutions.md` § Q-81): a 12-row `health_mod` census and `derived/combat.rs::fatigue_levels` prove *positive = better for the character* is deliberate and catalogue-wide, so **the three shipped amounts stay**; the defects are the doc comment on `types.rs::HealthTrack::CastingFatigue` and the Fluent label, which names a *cost* and so prints *"Casting fatigue: +1"* for a **Virtue**. Rename the label's value in **both** locales. Kind and severity left as the batch wrote them |
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
| F-370 | `flaw.bound_casting_tools` | 5723-5726 | class+src | M | live (half fixed) | an Arcane-Connection duration rule under `narrative`, and an EN source sentence that is truncated. **The source half is FIXED — D39** (ArMDE:5725 reconstructed from the English fragment). The `narrative` half stands: the durations are a rule and owe `uncomputed_rule` + text (D20) |
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
| F-423 | `SurfacedModifier` (6 Flaws) | — | engine | M | live | a surfaced modifier carries no source item, so six different Flaws render as the same unattributed word — a gap Part C does not cover. **Unblocked by D45**: add `source: Id`, render via the label map, and apply it to all five `ModifierFamily` values. Widens an **IPC-crossing DTO**, so the frontend mirror needs checking |
| F-424 | `uncomputed_clauses.rs` | — | engine+audit | n/s | live | **screen gap:** the screen missed eleven entries here, and the exact word-forms it lacks — see § 3.1 |
| F-425 | 5 translation-table rows | — | table | L | live | five rows disagree with the German rulebook; the shipped data is right on all five (D6) |
| F-426 | `flaw.ability_block` | 5651-5654 | class+engine | M | **withdrawn** — dup→F-355 | same entry, same passage, same verdict as B10's F-355 (this index's determination; see § 4) |
| F-427 | ArMDE:2816 | — | engine | M | live | the "one Social Status" rule is enforced nowhere, while the *guideline* one paragraph below it is enforced strictly |

### B13 — `batch-13.md` (ArMDE:6068-6235)

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-428 | `flaw.feral_upbringing` | 6110-6113 | number | H | live | a Flaw that *replaces* the childhood block is encoded as one that *adds* to it, so the first five years fund 240 XP instead of 120. **Unblocked by D40**: `replaces_life_stage_xp { stage: childhood, … }`, designed together with **F-439**'s apprenticeship case. Lowers a funded total, so saves legal today report and block per D10 |
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
| F-442 | `flaw.greater_malediction` (+`flaw.lesser_malediction`) | 6210-6213 | class+desc | M | live | a magnitude-calibration rule and a named immunity, classified as flavour *(the `lesser` half is re-rated as F-452, declared)*. **⚠ The "named immunity" could not be found (Q-107, 2026-09-23).** Both passages are two sentences and contain only the calibration: ArMDE:6212 *"comparable to those of other Major Flaws … almost any Flaw could be the result of a curse"*, ArMDE:6344 *"about as bad as other Minor General Flaws"*. **Verify the claim before working the finding**; the calibration half stands and **Q-107 rules out an open grant**, so reclassification + text is the whole remedy. Not re-rated here, per § 0.2 |
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
| F-473 | `flaw.offensive_to_beings` | 6524-6533 | text | M | live | the shipped German description calls Magical Air by a name the app does not use. **The case that decided D36** — a broken cross-reference inside our own product: the reader looks for that element in the UI and it is not there |
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
| F-520 | `flaw.stuck_in_your_ways` | 6791-6794 | class+prov | M | **live** (Q-131 settled) | `narrative` on a `min()` formula naming "Covenant Lore". **Q-131 resolves it to `ability.area_lore` for a covenant and `ability.organization_lore` for the other targets**, so the description can now be written in both locales and the entry reclassified `uncomputed_rule` (D20) |
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

### Q — `q-resolutions.md` (2026-09-22), found while answering questions, not while reading a span

These seven belong to **no batch**. They were turned up by the pass that answered
§ 6's 31 evidence-settleable questions, and each is stated in `q-resolutions.md`
— six of them in its closing *"Things nobody asked about, found on the way"*
section (its items 1, 2, 6, 7, 8 and 11), and **F-552** in § Q-116's own verdict,
which ends those two entries' unrated status. They are numbered from the next
free number and **nothing is renumbered**.

**The other five items of that closing section are deliberately NOT filed as new
rows, and here is why for each**, so the next reader does not "complete" the
block. **Item 3** — the `casting_fatigue` read-out — lands on **F-351**, whose
summary now carries the reversed remedy. **Item 4** — the free Well-Traveled
Virtue and the Wealthy/Poor/Gift prohibitions on `virtue.redcap` — is already
**F-243** and **F-242**; q-resolutions says no *question* raises them, which is
true, but findings do. **Item 5** — `virtue.berserk`'s mandated *Angry +2* — is
already **F-19**. **Item 9** — the `tainted`↔descriptor census, 21 ↔ 23 and
exact — records that the data is **right** and merely suggests a cheap guard.
**Item 10** — that `forbid_tainted` would make False Power (Sense Passions)
unselectable — is evidence *for* Q-70's verdict, not a defect.

| F | entry | ArMDE | kind | sev | status | summary |
|---|---|---|---|---|---|---|
| F-546 | `mythic_type.devil_child`, `mythic_type.spirit_votary` | 2638-2664 | number | H | **fixed** | `bonus_flaw_points: 7` read ArMDE:2664's *"an **additional** seven points of Flaws"* as a bonus on top of ArMDE:2638's ten, when the compulsory Major Flaw's 3 + 7 **is** the ten; `validation/balance.rs::effective_budget` folded it into the ceilings, so a Devil Child was allowed 17 Flaw / **37** Virtue points. (`q-resolutions.md` says 34, which omits the entry's own `bonus_free_virtue_points: 3`; **37** is the figure D32 quotes from the withdrawn `RULES.md:3840-3857` and the one `c3714b2` states, and it is the right one.) **Superseded and settled by D32**, which is broader — *every* Mythic Companion has 10 Flaw / 20 budgeted Virtue points, so `bonus_free_virtue_points: 3` was wrong too. **Already fixed and committed** (`212d726`, `c3714b2`): both values removed from `rules/core/mythic_companion_types.json` **and both fields removed from the model**, pinned by a ruleset-level test that iterates whatever types ship. Filed for the record only — **do not work it** |
| F-547 | `virtue.affinity_ability` | 3372-3374 | number | H | live | `effective/xp.rs::charged_cost` computes `ceil(T·den/num)`, which **floors** the effective points where ArMDE:3374 **ceils** them; the correct inverse is `(T − 1)·den/num + 1`. Overcharges 1 XP at Ability scores **1, 4, 7, 10, 13, 16, 19** (costs ≡ 2 mod 3) — Ability 4 costs 50, the engine charges 34, 33 suffices. **The divergence condition, contributed by a concurrent session and verified here:** the two formulas differ **iff `(T × den) mod num` lies strictly in `(0, den)`**. From it, `virtue.linguist` (5/4) and every Art are **provably clean**, and so is Flawless Magic's 2/1 — *proofs, not spot-checks*. **Arts**: Art costs are the triangular numbers and those cycle **1, 0, 0 mod 3**, never 2, so no Art score can trip a 3/2 Affinity *at any score*. **Linguist**: every Ability cost is a multiple of 5, so `T mod 5 = 0` and the interval is never entered. **Flawless Magic**: `0 < r < 1` is empty over the integers, which also clears **spell mastery**. **Fix `charged_cost` itself, not a special case for Abilities** — the formula is wrong generally, and a new ratio or advancement table would widen it silently. Q-39, revised: the defect is Linguist's *sibling*, not Linguist. Raises caps, so it invalidates no save. **ArMDE:2443 is NOT a regression guard — corrected 2026-09-22.** The book's worked example (Perdo 10, T = 55) yields **37 under both** the correct and the incorrect formula, which is exactly why `effective.rs::affinity_charged_cost_matches_perdo_example` has been green over this bug since it was written. Worse, its sibling line asserts `charged_cost(275, Some((3,2))) == 184` with the comment `ceil(275·2/3)` — **that pins the defect** (correct: **183**) and must change with the fix. A concurrent session found this from the Specialist grog template, which balances at 93 and is one short at 94; inspection never would have |
| F-548 | `virtue.simple_student` | 4958-4963 | audit | L | live | **D16's sex-restriction count is short by one.** D16 re-counted the family as *"20 entries … plus **one** female-only exception (`virtue.baccalaureus`, ArMDE:3474)"*; ArMDE:4960 is a **second** — *"Female characters can only take this Virtue if they are studying to be physicians at Salerno, although the Paid Rights Virtue would allow them to take this Virtue elsewhere."* Found answering Q-67. The remedy is already delivered by Q-67's reclassification (`uncomputed_rule` + `description` in both locales, which is D16's own disposal); what is owed here is the corrected count, so D16's scope is not quoted short |
| F-549 | `virtue.potent_magic_major` / `_minor` | 4740-4781 | text+prov | L | live | **a doubled clause in the English source**: ArMDE:4742 reads *"compatible with a Magical Focus, **unlike a Magical Focus, unlike a Magical Focus,** a maga may have more than one area"*, where DE:4742 has it once. Two independent renderings disagree, so it is an **extraction artefact, not the book** — D26's decision shape (correct it in `rules/source/en/`), but **outside D26's closed scope**, which named only ArMDE:7054 and ArMDE:3502. Rated `L` on what was verified, the source file; **if the doubled clause also reached `rules/i18n/en/` it is a user-facing `text` defect and rates `M` — not checked** |
| F-550 | `types.rs::Prereq` | — | engine | M | live | **`Prereq` has no warning severity**, so **D16's hedged→warning rule has no carrier at the entry level** — `prereq_not_met` is an error, and the profile level is the only place D16 has been applied. Q-115 makes this **live rather than latent**: ArMDE:6320 owes exactly such a warning (holds `flaw.inscribed_shadow`, not House Criamon), and the sweep finds a population of **two** with ArMDE:6957. `IssueSeverity::Warning` already exists; what is missing is a way for an *entry's* eligibility rule to reach it |
| F-551 | `flaw.deficient_form` | 5909-5912 | text | M | live, dup→F-404 | the `name` is the flat `"Deficient Form"` in **both** locales (DE `"Defizitäre Form"`) though the entry carries a `form` parameter, so **two Deficient Form selections render identically** and the player cannot tell which Form each is. Its twin interpolates (`"Deficient {technique}"` / `"Defizitäre {technique}"`). **F-404 raises the same asymmetry from the opposite side** — it reads the catalogue's convention as *against* interpolating and calls `flaw.deficient_technique` the defect. **The two must be worked once and in one direction**; this row records the consequence that decides it, and takes no position on which way |
| F-552 | `flaw.lame`, `flaw.hobbled` | 6330-6333, 6260-6263 | scope+prov | H | live | **both entries are under-modelled, and both stop being "checked and clean"** — B14 passed them and then escalated them itself as **Q-116**. ArMDE:16656 defines *"five combat scores: Initiative, Attack, Defense, Damage, and Soak"*, and ArMDE:22809 separates *scores* from *rolls* — only Initiative, Attack and Defense end in a stress die. So `flaw.lame` (*"other combat **scores**"*) owes `combat_mod -1` on `initiative` and `damage` **and** `soak_mod -1`, while `flaw.hobbled` (*"other combat **rolls**"*) owes `combat_mod -6` on `initiative` only. The existing Dodge-scoped `-3 defense` row on `flaw.lame` is correct and stays. **No engine change.** `RULES.md`'s *"combat rolls / combat scores"* block is **rewritten, not amended** — its conclusion *and* its warrant fail — and row 37 of `docs/open-todos.md` reopens. Scope is these **two** entries; the four ArMDE:6436, :6608, :6580, :6440 use neither phrase |
| F-553 | `virtue.magical_mount` | 4373-4376 | prereq | M | live | **an unencoded character-type restriction, and D38's second caller.** ArMDE:4375 ends *"only a companion or magus-level character can take this Virtue"*; the entry carries no prerequisite and no profile forbids it, so a **grog** may take it. Distinct from **F-166**, which rates the same entry's *classification* (`narrative` over a Might cap, a granted trait and a mandatory Major Story Flaw) and says nothing about who may take it. Found while settling **Q-75** by sweeping the core book for sibling "only … can take this" sentences — there are exactly **two**, this and ArMDE:2394. **Design D38's `Prereq` against both**: this one names *two* audiences, so a single-type variant must compose under `Prereq::Any`. *"magus-level character"* is **not** defined in the passage and needs a reading before the data is written — it plausibly covers `mythic_companion` as well as `magus` |
| F-555 | `flaw.cyclic_magic_negative` | 5893-5896 | scope | H | live | **the Lab Total penalty is not season-gated the way the Virtue's bonus is, and one D4 row stood for both.** ArMDE:5895: *"The penalty applies to Lab Totals **even if** the negative period does not cover the whole of the season"* — against the Virtue's *"**if** the positive part of the cycle covers the whole season"* (ArMDE:3637). Both fix the halves as equal (ArMDE:3637, :5896), so on a **solar or lunar** cycle — the book's own examples — every season contains negative time and the −3 **always** applies to Lab Totals, while the bonus never can. Only a *seasonal* cycle makes them behave alike, and **the cycle type is recorded nowhere** (a **D9** instance, which must land first). Distinct from **F-45** (the Virtue's scope) and **F-398** (this entry's missing text); see **D52** |
| F-556 | `virtue.domestic_animal` | 3699-3702 | prereq | H | live | **a human character can satisfy his mandatory Social Status by declaring himself an animal, free of charge.** ArMDE:3701 opens *"The character **is an animal** who is the property of a covenant or character"*, and the entry ships `magnitude: "free"`, `categories: ["social_status"]` with **no prerequisite** — so under **D41** (every character takes exactly one Social Status) a grog or companion satisfies that requirement at **no cost** with this, and validation accepts it. Found while settling **Q-11**, which asked the *other* question (whether animals are modellable — they are not: no **Cunning** characteristic, no profile). **This half does not depend on that**: the gate is needed whether or not an animal type ever exists, and it wants the character-type `Prereq` **D38** introduces |
| F-554 | `flaw.dhimmi` | 5954-5957 | prov | L | live | **a cross-book rule recorded so the debt is visible, per D22 — not implementable and not a data defect.** `RoP:D:5671` requires this Flaw of *"almost all non-Muslim characters **living under Muslim rule**"* and exempts Hermetic magi. Three reasons nothing changes: the condition is a **saga setting** the engine does not model; the wording is hedged twice (*"almost all"*, *"tend to be"*), so **D16** bars an error; and D22 keeps the implementation core-book-only. The entry's `source` correctly cites ArMDE and must **not** gain a second `SourceRef` (D22 part 2). Filed from **Q-99** |

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
| F-520 | **Q-131** | ~~the identity of "Covenant Lore"~~ — **settled 2026-09-22**: `ability.area_lore` (covenant) / `ability.organization_lore` (other targets), ArMDE:7282 vs :7674. **No longer blocking.** |
| F-85 | **Q-10** | the German name for `virtue.fabric_ripper`; Q-10 is the Covenfolk two-sources question. |
| F-270, F-283 | a catalogue decision | the required Virtue does not exist; the fix is either to add the entry (B02's F-34 shape) or to drop the requirement. |
| F-496 | **settled by D7** | D7 (2026-09-21) rules the DE rulebook heading wins. The name change is **cancelled**; the table-annotation obligation remains. |
| F-444 | already applied | `decisions.md` D6 now carries the corrected two-rule precedence. Verify rather than re-fix. |
| F-525 | ~~Q-43~~ → **D11** | D11 answers Q-43 **no**: a Reputation has three components and polarity is not one. The "no positive Reputation" absolute is therefore **not enforceable even in principle**, and the whole remedy is the reclassification + `description`. |
| F-526, F-538 | **Q-137**, remedy under-specified | F-526's `virtue.doctor_in_faculty` prerequisite lands regardless; only the seventeen-way exclusion waits. F-538's `virtue.sufi` copy is *not* sufficient — `validate_house` ignores `taken_as` by design. |
| the **seven** remedies `q-resolutions.md` settles but cannot land | **machinery another ruling owns** | **NEW 2026-09-22.** Each question is answered; only the fix waits. **Q-117** → § 2.1b's *"declines to impose a penalty"* capability family, the same blocker as D8's 48 — it is stated in § Q-117's body and **missing from q-resolutions' own six-row table**, which is why the count here is seven. **Q-42** → Q-132 / F-427's category mechanism, which must also express a *scoped* exception. **Q-67** → a **numeric** parameter type + a parameter-scaled `RestrictedAbilityXp` (the interim `uncomputed_rule` + text is **not** blocked and should land). **Q-138** → D12's classification pass, then D23 **with a magnitude qualifier added to its scope**. **Q-29 / Q-38** → **F-141**'s parameter, then D10's explicit `max_total`. **Q-110** → a "mandates N traits at value M" field, designed with D21's Effect-side twin; **F-440**'s own fix (the validator) is *not* blocked. **Q-115** → a warning severity for `Prereq`, which is **F-550**. |
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

> **This is the single list of engine work. Added 2026-09-24 on Norbert asking
> where engine changes are logged — they were in three places and none was
> complete.** The table below stops at D18, because it was written then; the
> second table carries D19–D58 and the concurrent session's findings. **§ 3.0
> routes; this enumerates.** Under **D58** every row is project work: a missing
> mechanic is a task, not a boundary.

#### 3.15b Machinery the rulings since D19 require

| Ruling / finding | Machinery |
|---|---|
| **D21** (+ F-427, F-502, F-355, F-542, Q-07, Q-102) | range over a **category** — one mechanism, with its Effect-side twin |
| **D23** (+ Q-137) | **predicate**-valued exclusions — different quantifier from D21, designed together |
| **Q-51**, **D33** | an effect **gated on a parameter value** — *"applies only when the parameter holds Y"*. Distinct from parameter-**valued** (`characteristic_score_delta_param`), which exists |
| **D28** | `spell_level_cap` **range-aware**; re-keys `spell_level_caps` and the UI picker |
| **D29** | **one** resolution point for an Ability's maximum score (age band folded with V/F overrides) |
| **D34** | an **id whitelist** on `ParameterDef`, additive to `require_categories` |
| **D35** | `ParamType::Number { min, max }` + a parameter-scaled `RestrictedAbilityXp` |
| **D38** (+ **F-553**) | a `Prereq` naming a **character type**; must compose under `Any` |
| **D40** | `replaces_life_stage_xp` — substitutes a whole block |
| **D56** | a **truncated** life-stage block (the fourth XP mode), **conditional `creation_phases`** (`{ phase, when }`), and the **`is_magus` split** across ~60 sites |
| **D41** | a category **minimum** in the profile budget; a warning on a second Social Status |
| **D43** (+ F-234, F-102, F-18) | scope a pool's implied permission to **itself**; general XP needs explicit authorization |
| **D45** (+ F-423) | `SurfacedModifier.source`, all five families — widens an **IPC-crossing DTO** |
| **D48** (+ F-74) | pool `instances` + **union** eligibility |
| **D50** | the phrase screen must catch clauses with no signed number — § 3.1, and it may not be mechanisable |
| **D55** | a **factor** on `advancement_mod`; retire `amount: 0` as a marker |
| **D42** | a **default realm** on the concept, seeding every Supernatural entry |
| **D57** | German **apposition** templates — data, but ~36 need the sweep |
| **W2** (`open-todos.md` row 53) | `AbilityAuthorization` cannot **restrict** — an exclusive-choice authorization. **Folds into row 45 with `virtue.custos`** |
| **K5** (`open-todos.md` row 52) | `EquipmentSlot::equipped` is **overloaded** — gates Combat rows *and* Load; needs a third state or two flags |
| **K3** (`open-todos.md` row 52) | mounted combat: nowhere records *"mounted"*. **In scope under D58** — the sheet shows it, so it is computed |

**Two of these are not in this file's § 1 and must not be added** — K3 and K5 are
engine findings from the book-template session and live in `open-todos.md`. That
is a filing rule, not a scope one (§ 4a, D58).

#### 3.15a The original list (D1–D18 era)

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

### 3.0 Routing table for D19–D56 — **read this before working § 3** (NEW, 2026-09-24)

**Why this exists.** § 3 was written when the audit had eighteen rulings. It
references **D1–D18 and nothing later**. Thirty-eight further decisions have
landed since, each recorded in `decisions.md` and in its question's row — neither
of which a correction pass reads. A settled question also *drops out of § 6.2*,
which is the list that says "this needs Norbert", so **closing a question can make
its work disappear**. That is the § 4a failure mode aimed at this file's own
structure, and Norbert found it by asking whether D56's truncated apprenticeship
was queued. It was not.

**This table routes, it does not restate.** The ruling is in `decisions.md`; this
says where the work belongs and what it joins.

| Ruling | Work | Joins |
|---|---|---|
| **D19** | the screen matches by **regex**; convert first and prove the conversion inert | § 3.1 |
| **D20** | 19 entries → `uncomputed_rule` + text; **5 swallowed numbers** | § 3.3, § 3.4, § 3.9 |
| **D21** | one **category** mechanism + its Effect-side twin | § 3.15 |
| **D22** | cross-book rules filed as findings; implementation core-book-only | § 3.14, § 3.16 |
| **D23** | **predicate**-valued exclusions — design with D21 | § 3.5, § 3.15 |
| **D24** | `IsMagus` on both parens entries; profile condition **unchanged** | § 3.6 |
| **D25** | errata note in `RULES.md`; sweep 630 entries against the index | § 3.16, § 8 |
| **D26** | *done* (`7cd8a26`) — but see `CLAUDE.md`: repair upstream, not here | — |
| **D27** | spell list models **known**; warn, never block | § 3.15 |
| **D28** | `spell_level_cap` becomes **range-aware**; re-keys the picker | § 3.8, § 3.15 |
| **D29** | **one** resolution point for an Ability's maximum score | § 3.15 |
| **D30** | **563** anchors; every range to the non-blank form | § 3.14, § 8 |
| **D31** | table wins on a **name**; **7 corrections reverted** in both projects | § 3.13 |
| **D32** | *done* (`c3714b2`) | — |
| **D33** | `flaw.flawed_powers` gains the **import parameter** — with D34 | § 3.7 |
| **D34** | id **whitelist** on `ParameterDef` — with D14 | § 3.7 |
| **D35** | `ParamType::Number`; cap **2 years** — with D9, D14 | § 3.7, **§ 3.17** |
| **D36** | German term binds in prose; **no guard**, review-enforced | § 3.12 |
| **D37** | True Faith is a **floor** across every Form, not an addend | § 3.10 |
| **D38** | character-type `Prereq`; **two** callers (**F-553**) | § 3.6 |
| **D39** | *done* (`cc068e2`) — same upstream caveat as D26 | — |
| **D40** | `replaces_life_stage_xp` — **two** carriers | **§ 3.17** |
| **D41** | category **minimum**; a second Social Status **warns** | § 3.15 |
| **D42** | concept gains a **default realm**; ~111 entries record one | § 3.7 |
| **D43** | pool permits **itself**; narrowing + authorizing are **one slice** | § 3.2 |
| **D44** | mark **entailed** pairs; measure the other 80 | § 3.5, § 8 |
| **D45** | `SurfacedModifier.source`, all five families; **IPC DTO** | § 3.15 |
| **D46** | classification by **what** is computed; fix `every_vf_is_classified` | § 3.3, § 3.16 |
| **D47** | encode Guild Apprentice's suppression, keyed on the holder | § 3.10 |
| **D48** | pool `instances` + **union** eligibility; **four** carriers | § 3.7, § 3.15 |
| **D49** | free seasons **bind the creation plan**; F-131/F-146/F-151 change remedy | **§ 3.17** |
| **D50** | re-test **all 335** `narrative` entries against the new line | § 3.3, § 8 |
| **D51** | prerequisite / grant / **effects-only**, per the book's wording | § 3.5, § 3.6 |
| **D52** | D4's Cyclic row **splits**; **F-555** | § 3.8 |
| **D53** | glossary governs in quotations; `virtue.mild_aging` not clean | § 3.12 |
| **D54** | `flaw.independent_craftsman` → `personality`; **§ 8a** | § 3.16 |
| **D55** | `advancement_mod` gains a **factor**; `amount: 0` stops overloading | § 3.15 |
| **D56** | **`is_magus` splits** — ~60 sites; `Prereq::Has(the_gift)` *done* (`7ca56ad`); **truncated apprenticeship**; conditional `creation_phases` | § 3.15, **§ 3.17**, § 8 |

**Three of these change § 3.17 and are marked in bold above** — D35, D40, D49 and
D56 all touch the XP model, and D56 adds a **fourth mode** the section does not
have (see its note below).

---

### 3.17 The three XP modes — **design all three together** (NEW, 2026-09-21)

> **⚠ There are FOUR modes, not three (added 2026-09-24).** **D56** requires a
> **truncated** life-stage block — apprenticeship stopped at a chosen age, then
> later-life XP — for `flaw.abandoned_apprentice` (ArMDE:5647: *"Decide at what
> age the character was abandoned. Create the character as a regular apprentice
> up until that age"*). That is **not** replacement: replacement substitutes a
> whole block, this one shortens it.
>
> **The rate is SETTLED (Norbert, 2026-09-24): 16 XP and 8 spell levels per
> year**, from ArMDE:2435's *"240 experience points, and 120 levels of spells"*
> over fifteen years. **Both** quantities divide, not only the XP. The rate is
> **ours** — the book prices apprenticeship as a lump — and saying so is the
> point: **this also unblocks guided-creation issue #15**, which wanted an
> editable apprenticeship duration and was *BLOCKED ON SOURCE* on exactly this.
> Spendable per ArMDE:2435 on *"Arts or Abilities, including Arcane, Academic,
> and Martial"*; **Parma Magica allowed with a warning**, not excluded
> (ArMDE:5648 threatens, ArMDE:5650 assumes he may join); and the truncated block
> **must not enforce `apprenticeship.minimum_abilities`**, which describe a
> *completed* apprenticeship. **D35** (the numeric age parameter) and **D49**
> (free seasons binding `post_gauntlet_lab_seasons`) belong in the same pass.

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

**Ten duplicate relationships across all 551 findings, and all ten are in
F-001…F-501.** (**Eleven if F-551 is counted**: it carries `dup→F-404`, the only
marker the 2026-09-22 block adds, and it is a *partial* overlap of the kind § 4.4
collects rather than a re-rating — the two rows read the same asymmetry in
opposite directions and must be worked once, in one direction.) One (F-481) was already found and withdrawn by B16. **Nine were
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

## 4a. What this ROUND does not cover — which is not the same as out of scope

> **⚠ REFRAMED BY D58 (2026-09-24).** This section listed K3, K5 and W2 as things
> the V/F round does not cover. That was true of the **pass** and was then read as
> true of the **project**. Norbert's goal settles it: *"an application which can
> build all characters legal under core rules … **if mechanics are missing, the
> mechanics need to be implemented**."* So **everything below is in scope for the
> project** — it is queued elsewhere, not declined. The test is **does its absence
> prevent building, or misreport, a character legal under the core rules?**
>
> **Two deliberate non-goals** sit outside the test and are listed together so
> nobody mistakes them for oversights: **supplement content** (D22 — "core rules"
> excludes it by construction) and **animal characters** (D58 — no `Cunning`, no
> animal profile; `F-556`'s gate is the permanent answer, not a stopgap). Q-05's
> refusal of a **sex model** is the third.
>
> The filing failure this section was written for stands unchanged, and is still
> the reason to read it.

**The failure mode, stated first because it is the point.** A finding cluster in
which **one member fits this round's scope** can make the **whole cluster look
covered**. The member that fits gets filed here, someone reads "those findings
are filed", and the rest — which were never V/F data — are never queued anywhere.
Identified 2026-09-24 by the concurrent book-template session, from a case where
it had already happened.

**The worked example.** The Knight template produced three findings. **K1** is a
missing effect on `virtue.knight` — V/F data, this round's business, and part of
`open-todos.md` row 45's class. **K3** and **K5** are **engine capability**:

- **K5** — `EquipmentSlot::equipped` is **overloaded**, gating both whether a
  weapon yields a Combat row and whether it adds Load. The Knight needs those
  separated (five Combat rows including two for a great sword, ArMDE:1468-1472,
  against an Encumbrance counting only the wielded set, ArMDE:1484). **Already
  shipped**: since `f70c57f` made `equipped` load-bearing for Load, the fixture
  resolves the conflict by marking the great sword unequipped and **losing its
  two Combat rows**, on the judgement that a wrong number beats a missing row.
- **K3** — mounted combat is unmodelled (ArMDE:16837-16839, +min(Ride, 3) to
  Attack and Defense). The rule is trivial and `Ride` is already on the entity;
  what is missing is anywhere to record *"mounted"*.

A perfectly executed V/F round working from § 6.2 would touch **neither**, and
K1 being correctly filed here is precisely what made them easy to overlook.

**What this obliges.**

- Both are in `docs/open-todos.md` **row 52**, which is the list surfaced at
  release. They are **not** findings in § 1 and must not be added — they are not
  V/F defects. **That is a filing rule, not a scope one** (D58): they are the
  project's work, tracked where engine work is tracked.
- **When a finding arrives from outside this audit, ask which of its siblings are
  not V/F data**, and confirm those have a home before treating the cluster as
  filed.
- The same shape may already exist inside this file: an entry whose V/F half is
  filed while its engine half sits only in a batch record. § 8's sweeps are the
  place to catch that.

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

**REVISED AGAIN 2026-09-22**, against `decisions.md` D24-D32 and against
`docs/vf-audit/q-resolutions.md`, which answers the **31** questions that were
settleable from the evidence. **A row that reads "open" below is a question
nobody has answered — it is not a row nobody has looked at.** The gap this
closes has cost the audit real rework once already, when one batch re-derived
another's findings from a stale index.

**Eighty-two of the 142 are settled or closed**, in four tranches (60 open:
**57** `N`, **3** `R` — Q-01, Q-15, Q-72):

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

- **By D24-D31 and one source read (seventeen).** Q-140 (**D24**), Q-141
  (**D25**), Q-142 (**D26**, and fixed), Q-130 (**D27**), Q-133 (**D28** + **D29**
  for its sibling), Q-96 / Q-97 / Q-124 (**D30**), Q-10 / Q-40 / Q-49 / Q-65 /
  Q-69 / Q-71 / Q-114 / Q-121 (**D31**; Q-65 is *partly* settled — it is now
  table-vs-table and falls to D7 rule 3), and Q-131 settled from ArMDE:7282.
  These rows keep their original settler code and carry a bold **SETTLED** in the
  status column rather than the earlier tranches' **—**. **One exception, and it
  breaks any filter written on the word:** Q-65 reads `**PARTLY settled — D31**`,
  lowercase and qualified, so a `SETTLED` grep returns 47 where this tranche plus
  the next make 48.

- **By `q-resolutions.md`, 2026-09-22 (thirty-one).** Q-16, Q-18, Q-20, Q-25,
  Q-29, Q-33, Q-38, Q-39, Q-41, Q-42, Q-50, Q-53, Q-55, Q-57, Q-62, Q-67, Q-68,
  Q-70, Q-78, Q-79, Q-81, Q-89, Q-103, Q-109, Q-110, Q-112, Q-115, Q-116, Q-117,
  Q-125, Q-138 — every question marked **R**, **C** or a mix, re-derived from the
  sources and then **independently re-derived a second time**. The two
  derivations agreed on 24 of 31, and **seven diverged** (Q-25, Q-29/Q-38, Q-39,
  Q-50, Q-81, Q-103, Q-115). **Six of the seven overturned the first answer** and
  those six rows say so, because a question subtle enough to be answered twice
  differently is worth flagging to a later reader; the seventh, **Q-50**, was
  *confirmed and extended* rather than revised — the second pass agreed on
  Nephilim and found a new defect on its siblings, so its row claims no reversal.
  **All 31 are settled on the question asked**, but **three carry a narrow
  follow-on that still needs Norbert** — Q-103's remedy, Q-138's reading and
  Q-67's interim — and **seven have a remedy blocked on machinery another ruling
  owns**: the six `q-resolutions.md`'s own table lists (Q-29/Q-38, Q-42, Q-67,
  Q-110, Q-115, Q-138) **plus Q-117**, whose § Q-117 obligation 4 states a blocker
  — § 2.1b's capability family — that the table omits. Count seven.

**A ruling that closes a question does not always unblock the work.** D8's 48
reclassifications are still blocked behind § 2.1b, and D13's remedy is still
blocked behind D14.

| Q | Topic | Settled by | Status |
|---|---|---|---|
| Q-01 | the closed value set for `virtue.academic_concentration_subject`'s `subject` | R | open |
| Q-02 | is a prereq-only entry `creation_effect` or `narrative`? | — | **closed — D3, superseded by F-32** |
| Q-03 | does a computed entry owe a `description` for its uncomputed clauses? | — | **closed in substance — D5** |
| Q-04 | `virtue.alluring_to_beings` German name: table vs DE rulebook heading | N | **SETTLED — D31: the table wins.** This is a **name**, and D31 reversed D7.1 for exactly this case — `rules/source/de/` is an **older copy** than the tables, so convicting a table row with a stale heading is circular. Adopt the table's rendering into `rules/i18n/de/`. The three-way test D18 proposed is no longer needed; D31 replaced it. Original — **D7 supplies the test, D18 adds a third arm to it** |
| Q-05 | "only available to male characters" — the engine has no sex model | — | **closed — D16: `uncomputed_rule` + `description` in both locales. `Entity` does NOT gain a sex field and nothing enforces it.** Scope re-counted catalogue-wide: **20** entries, not 6, plus one female-only exception (`virtue.baccalaureus`, ArMDE:3474) and ArMDE:3438's "who must also be eunuchs". **A deliberate non-goal — do not re-open** |
| Q-06 | `virtue.aptitude_for_sin`: should the +3 be an `ability_roll_mod`? | N | **SETTLED: no — the variant requires an Ability the passage does not name.** `Effect::AbilityRollMod` is *"a flat modifier to rolls of **a specific Ability** in the free-text subject"* (Academic Concentration = Artes Liberales + a field, ArMDE:3362-3367). ArMDE:3428 instead gives *"+3 to **all rolls in a very limited circumstance**"*, and its own examples span Abilities — *picking pockets* is Legerdemain, *committing adultery skillfully* is Charm, *poisoning people* is something else again. **Forcing an Ability in would invent a scope the book does not give**, which is F-45's defect class run backwards. The entry is already correct: `uncomputed_rule` with a `sin` parameter at `domain: "text"`, so the +3 and the chosen circumstance reach the player as text (D20, D50). **No new Effect variant** — a circumstance-scoped surfaced modifier would be machinery for **one** caller (D21, D38); `virtue.special_circumstances` is *not* a second, since it names specific **totals** and belongs to the F-45 conditional-scope family |
| Q-07 | ArMDE:2816's one-Social-Status rule is modelled nowhere | N | **SETTLED — D21**, which names Q-07 and Q-102 as "the same ArMDE:2816 machinery" and closes them with the category mechanism. It is a **gap**, not a deferral, and the answer is one mechanism shared with Q-132, F-427, F-542 and F-355. Build work, not a decision. **Q-45 is the residue** and is *not* closed by this: ArMDE:2816 is a global rule, not a `virtue_category_caps` row. Original: **rated a defect by F-427** |
| Q-08 | the ArMDE:2960-2962 realm association is on 4 of ~115 Supernatural entries | N | **SETTLED — D42**: the **concept gains one optional default realm** that seeds every Supernatural entry's realm, and every such entry records one — overridable per entry, **fixed** where the book fixes it. It is a **default source, not a character attribute**: the book never says a character has a realm, and a character may hold a Faerie Blood and a Magic-realm Second Sight at once. Nothing is computed (the engine models no auras); the sheet must state it because D9 and D20 both bite. **Read which entries the book fixes** — ArMDE:2961 says *"a Virtue's description notes if it is limited in this way"* |
| Q-09 | `virtue.common_sense`: is a guaranteed storyguide intervention mechanical? | N | **SETTLED — D50**, which answers the whole Q-83 family at once: "mechanical" is the wrong test. ArMDE:3599's *"common sense (the storyguide) **alerts you** to the error"* has no number and no roll, and a storyguide who does not know it will not do it → **`uncomputed_rule`** + text in both locales |
| Q-10 | `virtue.covenfolk`: two canonical German sources, two names | X / N | **SETTLED — D31**: a *name*, so the **table** wins and its value is adopted into `rules/i18n/de/`. **F-85 unblocked.** (This is the one the source project was already ahead on — `Konventsbewohner` — which is D31's own evidence) |
| Q-11 | `virtue.domestic_animal`: "animals only" with no engine model | N | **SETTLED — and D58 makes it permanent: animal characters are a deliberate non-goal** (humans and covenants only), so **F-556's gate is the answer rather than a stopgap**, and no `Cunning` characteristic or animal profile is added. Original reading follows. **SETTLED: the animal character stays unbuildable, but the entry must stop being selectable by a human.** ArMDE:3701 opens *"The character **is an animal** who is the property of a covenant or character"*, and the engine has no animal model at all — `Characteristic` has no **Cunning** (ArMDE names it as what an animal has instead of Intelligence, e.g. at :4375), and no profile exists. So building one is out of scope, as creature rules were in **Q-34**. **But unlike Q-34 this entry ships in the catalogue and is reachable**: it is `magnitude: "free"`, `categories: ["social_status"]`, so under **D41** a human grog or companion can satisfy his *mandatory* Social Status by declaring himself a domestic animal, at **no cost**, and the app accepts it. That is wrong output today and does not depend on any animal machinery. **Remedy**: keep the entry (it is RAW), give it the text (D50), and gate it so a human cannot take it — which needs the same **character-type `Prereq`** D38 introduces, with an animal type as its value, or an explicit exclusion until one exists. **Do not add Cunning or an animal profile for this** |
| Q-12 | `ability.enchanting` parameterized? is `domain: "ability"` right? | N | **SETTLED: yes, parameterised — and `domain: "ability"` is the right *kind* but cannot express the domain on its own.** ArMDE:7446's heading is `#### Enchanting **(Ability)**` and its body says *"you can influence others with **a particular performance ability**"*; the book's parenthesis is its parameter marker, exactly as in `#### (Area) Lore` — and `ability.area_lore` ships `"parameter": "area"` while `ability.enchanting` ships **none**. Same convention, parameter omitted. **The value must be a *performance* Ability, and no `category` expresses that** — the five are academic, arcane, general, martial, supernatural, and Music is plain `general`. So it needs **D34's id whitelist** on top of `domain: "ability"`. And **`virtue.enchanting_ability` grants `ability_score_grant { ability: ability.enchanting, amount: 1 }` with no instance named** — the `virtue.student_of_realm` shape that **D14** was written from, so **D14 lands first** |
| Q-13 | does the book distinguish "may only **take**" from "may only **have**"? | — | **closed — D2's rules read: NO.** ArMDE:3665 (*Demonic Might*) says "may only **take**" and :3669 (*Demonic Powers*), the very next entry, states the *same* restriction as "may **have**". Two adjacent entries, one restriction, two verbs. **No argument may rest on that pair** |
| Q-14 | where is the threshold at which an effect-bearing entry owes a `description`? | — | **closed in substance — D5** |
| Q-15 | `virtue.craftsman`: is "Wealthy/Poor affect you normally" a clause? | R | open — settled in practice (§ 5) |
| Q-16 | `virtue.dust_devil`: how much of Skinchanger does "this variant" inherit? | R | **SETTLED — `q-resolutions.md` § Q-16 (2026-09-22).** Reading B: the **+3 Soak does not transfer** — ArMDE:4974's clause is quantified over *animals* and a dust devil (ArMDE:3709) is none — while the **item rules do**, because the entry swaps the focus object rather than dropping the slot. The Dove variant confirms the drafting habit: ArMDE:4984 overwrites the same sentence and **restates** the Soak; Dust Devil does not. Nothing owed, nothing blocked: the entry stays `uncomputed_rule` with no effects, and F-62's cross-reference instruction is satisfied for it |
| Q-17 | a supernatural power described only in prose, with constraints but no number | — | **closed — D8** (with Q-61, Q-64, Q-76, Q-126): a capability is a rule → `uncomputed_rule` |
| Q-18 | `virtue.gentle_gift`: does the book state the `has virtue.the_gift` prereq? | R | **SETTLED — `q-resolutions.md` § Q-18 (2026-09-22).** **Yes**, not in the entry's body but in ArMDE:2880 (*"**Only** characters with The Gift can take these Virtues and Flaws"*) and ArMDE:2840, both absolute in D16's sense — so the shipped prerequisite is correct and the over-strict reading is refuted (ArMDE:4139's unGifted clause restricts *Inoffensive*, and :4141 redirects to a Virtue that does not exist). Nothing owed, nothing blocked. Two side effects: this is **D12's worked *intrinsic* case**, so D12's pass must **not** hand it an `IsMagus` gate; and ArMDE:2880 is the general warrant for every `hermetic` entry's Gift requirement — cite it once rather than re-derive it 122 times. F-81 unaffected |
| Q-19 | may the catalogue assert an incompatibility no passage states, when the pair is logically contradictory? | N | **SETTLED — D44**: yes, as a **hard block**, but **only where each entry's own text contradicts the other's** — entailment, never theme or plausibility. `virtue.gentle_gift` ↔ `flaw.blatant_gift` stays. **Two obligations**: mark entailed pairs with the two contradicting passages (an unmarked unsourced constraint is indistinguishable from an invented one), and **measure the other 80** — B03 left that count to this ruling, and each unsourced pair must be shown entailed or removed. **Does not generalise to prerequisites**, which stay "source or nothing" |
| Q-20 | `virtue.fidai`: does the pretended Social Status permit a second one? | R / N | **SETTLED — `q-resolutions.md` § Q-20 (2026-09-22).** **No.** ArMDE:2816 admits a second status only where a description *"explicitly note[s] that they are compatible"*, and the book's idiom for meeting that test is the word **compatible** (ArMDE:4325 is the one entry that uses it); ArMDE:3881 says *"**pretending** to have"*, which is the negation of having — DE:3881's *vorgeben* agrees. But the sentence **is** a stated choice, so under **D9** it owes a `ref` parameter over the `social_status` category — a **new D9 instance**, absent from § 3.7's list, for D9's catalogue-wide re-derivation. Nothing on `incompatible_with` and nothing on ArMDE:2816. F-68 is independent and stands |
| Q-21 | `virtue.forge_companion`: is "unGifted craftsman" an encodable incompatibility? | N | **SETTLED: encodable, but not as an incompatibility — use `Prereq::Nor([Has(virtue.the_gift)])`.** ArMDE:3927's *"an **unGifted** craftsman"* is a property of this character, so it belongs on this entry alone. `incompatible_with` is wrong twice over: it is **symmetric**, so `virtue.the_gift` would have to name Forge Companion back — polluting the most heavily-used entry in the catalogue with one Virtue's rule, the same objection **D47** raised for Guild Apprentice — and `validate_incompatibilities` reads **bought** selections on both sides (**F-466**), so a *granted* Gift would slip through, while a `Prereq` is evaluated against the full `PrereqCtx` (**D2**). **Two further rules in the same passage are not covered here**: *"attached to a Verditius magus"* is a relationship to another character and is text; and the 50 XP is scoped to *"the particular Crafts **her master** practices"*, which `abilities: ["ability.craft"]` funds unscoped — a **D48** instance needing a Craft instance parameter |
| Q-22 | where does an *uncomputable* rule go on an entry that already carries an `Effect`? | — | **closed in substance — D5** |
| Q-23 | does modelling a rule on the *type profile* make the Virtue a `creation_effect`? | N | **SETTLED — D46**: classification follows **what** is computed, never **where**. All stated rules computed (entry or profile) → `creation_effect`; any stated rule computed nowhere → `uncomputed_rule` + text. So `virtue.the_gift` → **`uncomputed_rule`** (ArMDE:2870-2876's penalties reach the player nowhere) and `virtue.hermetic_magus` stays `creation_effect` (computed via the magus profile's `required_traits`). **B04's precedent evaporated**: it rested on `virtue.devil_child`'s profile-held budget bonus, which **D32 deleted**. Obliges a re-examination of the five effect-less `creation_effect` entries and a fix to `every_vf_is_classified`, whose asymmetry lets them pass |
| Q-24 | `virtue.greater_benediction`: reclassify, or renarrow `source.lines`? | N | **SETTLED: reclassify — D30 forecloses renarrowing.** The next heading is ArMDE:4009, so the insert through :4008 lies inside this entry's span and `[3991, 4008]` is **correct** under D30 (a range ends on the last non-blank body line before the next heading). That option was still live when B04 asked; it is not now. The entry's **own** rule is ArMDE:3993's calibration — *"The effects of the benediction should be comparable to other Major Virtues"* — the same shape as the Maledictions in **Q-107**, so `uncomputed_rule` + text under **D50**, and **no open grant**. The insert's five numbers are what the book itself calls *"examples"*. **Consequence for the screen**: it reads the whole cited range, so it will keep flagging the insert's *"+3 bonus to all social rolls"* — the entry is **not** thereby a rule-dropper, and § 2.1b's queue must not treat it as one. Original — narrowed by F-121 |
| Q-25 | is "politically Criamon, created by another House's rules" a rule? | R / N | **SETTLED (revised) — `q-resolutions.md` § Q-25 (2026-09-22).** **`narrative` → `uncomputed_rule`**, plus the passage in `description` in both locales. **The first answer — `narrative` stands, on D22's `flaw.seeker` precedent — was overturned by the verification pass**, which found ArMDE:2264: *"Membership in a House grants a particular benefit at character creation … A magus can only be a member of one House"*, which ArMDE:4039 then **decouples**; `Entity` has one `house` field and structurally cannot hold that, so **D3** governs and `narrative` is not available. The two prerequisites are unaffected and complementary: `Prereq::IsMagus` (trained under D12, so inside D12's pass, not a separate slice) and `Prereq::Nor([House("house.criamon")])` for *"any **other** House"* — a `Prereq`, **not** an `incompatible_with` row, since a House is assigned rather than bought (the D2 / F-466 trap). Not blocked |
| Q-26 | classification and encoding for a rule that *nullifies* another Virtue's effect | N | **SETTLED — D47 (revised)**: **encode the suppression**, keyed on holding `virtue.guild_apprentice`, plus the text. D47's first version said *text only*, reasoning that the stage was unmodelled — **wrong**: `virtue.journeyman` exists (ArMDE:4159, *Minor, Social Status*), so with **D41**'s one-Social-Status rule, *"until he moves to the journeyman stage"* means exactly *"while he holds Guild Apprentice"*. **Reachable**: Wealthy is companion-only (D38), so a companion with both is shown 20 XP/year the book denies. Sibling sweep found no second caller and no new defect (Guild Master's authorization is already **F-102**). `incompatible_with` stays rejected. Second half — `virtue.indescribable_face` stays **`narrative`** (D46), choice recorded as a parameter (F-112 stands) |
| Q-27 | `restricted_ability_xp` authorizes permanently; ArMDE:4065 says it should not | N | **SETTLED — D43**: the pool permits spending **itself** and nothing more; general XP needs an explicit `AbilityAuthorization`. The existing warrant (*"the grant would otherwise be unspendable"*) justifies the pool and nothing wider. **28 entries carry a pool** and each must be re-read and given an authorization where the book grants general access (Educated's *"you may buy Academic Abilities"*) — **narrowing and authorizing are ONE slice**, since shipping the narrowing alone removes legitimate access. Rewrite the `AbilityAuthorization` doc comment, which records the rejected reasoning as fact. Note ArMDE:4065 bars **Magic Lore and Latin** but not **Order of Hermes Lore**, the third Ability its own pool funds — do not tidy that into symmetry |
| Q-28 | `virtue.guardian_angel`: encode the +5 Soak despite its condition? | N | **SETTLED from the source: no — carry both bonuses as text.** ArMDE:4035 gates them with *"The angel only grants you these bonuses **if you are acting in accordance with God's will**"*, a fiction condition the engine cannot evaluate, and it gates the Magic Resistance 15 as well as the Soak. Encoding either flat would reproduce **F-20** exactly — *"combat and Soak modifiers conditional in the book, unconditional in the data"*, rated **H** against `virtue.berserk` — and **D4** already refuses conditional modifiers whose condition is not a creation-time fact. So **F-99**'s remedy is `uncomputed_rule` + text in both locales (D50), **not** a `soak_mod`. Note **D37** cites this passage as one of the two *additive* Magic Resistance exceptions: that describes the **rule**, not something the engine will compute |
| Q-29 | `greater_immunity` and `lesser_immunity` disagree about repeatability | R | **SETTLED (revised, and the audit's one genuinely *arguable* reading) — `q-resolutions.md` § Q-29 / Q-38 (2026-09-22).** The two **passages never disagreed**; the *data* did — `virtue.greater_immunity` ships `max_per_target: 255` with **no parameter**, licensing 255 *identical* copies where ArMDE:4015 says *"a different immunity each time"*, and `virtue.lesser_immunity` ships neither. **F-94 confirmed.** On whether the repeat clause travels through ArMDE:4277's bare pointer the two derivations reached **opposite** answers and neither has decisive evidence, so **the first answer (it transfers, both repeat) was overturned by the verification pass** — ArMDE:2814 allows a repeat *"only if the description **explicitly** allows it"*, and ArMDE:4283 has `virtue.lesser_power` state its own repeatability rather than inherit Greater Power's — **and D10 breaks the tie toward *once***. Record the reasoning in `RULES.md` so the pointer is not re-litigated. **Remedy BLOCKED** on F-141's parameter, then D10's explicit `max_total`. Original follows. — **D10 supplies the mechanism** (absent now means *once*), but which of the two passages is right is still a rules read |
| Q-30 | how to model a "General **and** Hermetic" descriptor without blocking the unGifted case | N | **SETTLED — and it was already settled before the audit asked.** `RULES.md` records *"Decided 2026-09-13 (Norbert): an **and**-joined descriptor means **either route**"*, ratifying existing behaviour. **B04's premise was false**: `validate_permitted_categories` permits an item when **any** of its categories is permitted (`flaw.suppressed_gift` is the worked precedent), so dual membership never blocked a grog. `virtue.inoffensive_to_beings` is **correct as shipped** — ArMDE:4139 is already `Any([Has(virtue.the_gift), Has(flaw.magical_air)])`, and `hermetic` is deliberately in `index_categories` rather than `categories` because `gift_categories: ["hermetic"]` would otherwise make `effective::has_the_gift` read the Magical Air route as possession of The Gift. **No change.** *Lesson: check `RULES.md` for an existing ruling before escalating* |
| Q-31 | three Supernatural Virtues require player-defined content and none records it | — | **closed — D9**: record every stated choice |
| Q-32 | `AdvancementSource` has `Teaching` but nothing for books-written | N | **SETTLED: add the variant — there are TWO callers, and B04's hypothesis was right.** B04 said *"if Good Teacher is alone, text is the cheaper answer"* and guessed `flaw.incomprehensible` might be the counterpart. It is: ArMDE:6296 — *"Anyone trying to learn from you **or from a book you have written** must halve their Advancement Total (or Lab Total, if you are a magus and have written Lab Texts…)"* — against ArMDE:3973's *"Add three to the Quality of any books **that you write**"*. **A sweep for *"Quality"* across the V/F block misses it**, because this entry says *Advancement Total* instead; every other Quality modifier in the block is inward-facing (:3424 taught, :3521 books, :3939 self-study, :4117 practice, :5058 vis). **`flaw.incomprehensible` needs two rows, not one** — `Teaching` *and* the new authoring source — and its halving is **D55**'s factor, not `amount: 0`. **F-91**'s remedy changes: the mis-directed `book` row is replaced by an authoring row rather than dropped for text |
| Q-33 | the book gives Hermetic Prestige two different levels and the data picks one | R | **SETTLED — `q-resolutions.md` § Q-33 (2026-09-22).** **The data is right: `score: 4` follows the entry's own descriptor** (ArMDE:4073, *"a Reputation of level 4 within the Order"*); the level-3 figure is inside a block-quoted worked example (ArMDE:2518, Darius of Flambeau), and **D25**'s precedent gives the substantive statement the win over the secondary listing. `kind: "hermetic"` is right on both passages. No data change, nothing blocked. Two obligations: one more row in **D25's `RULES.md` errata note**, worded to hold descriptor-vs-**worked-example** as well as descriptor-vs-index; and the note must say explicitly that **`max_score` is not the answer** (D11 surveyed all 31 granting passages and found exactly one range), because D11's exact enforcement now makes the book's *own printed example character* invalid in the app |
| Q-34 | ArMDE:18432's Size-scaled Improved Characteristics is modelled nowhere and unreachable | N | **SETTLED — nothing is owed, and no `latent` status is needed.** ArMDE:18432 is explicitly **creature**-scoped (*"large **creatures** require more characteristic points… a **creature** of Size…"*), and `EntityKind` is `Character \| Covenant`. **The human side is already carried by the entries that state it**, verified not assumed: `virtue.giant_blood` ships `size_delta: 2` + Str/Sta +1, which is ArMDE:3977 exactly, and the book grants it **no** extra characteristic points — nor do Dwarf, Large or Small Frame. For a human, Size changes wound increments, not the point budget, so the flat `characteristic_points: 3` is correct for every entity this app can build. The entry's clean verdict stands |
| Q-35 | `virtue.knows_people`: is a once-per-story entitlement mechanical? | — | **closed — F-160** |
| Q-36 | `virtue.lasiq`: does "some other social status, which you should choose" demand a parameter? | — | **closed — D9: yes** |
| Q-37 | `virtue.leper_magus`: `grants_selection` or a copied effect? | N | **SETTLED — D51**, and this entry is the ruling's proof. ArMDE:4251 carries **both** patterns in one sentence: *"can only be bought **if the character also has** the Leprosy Flaw"* → **prerequisite** (`Prereq::Has(flaw.leprosy)`), and *"**granting** the Life Boost Minor Virtue"* → **grant** (`grants_selection`, budget-exempt). The absence of *"at no extra cost"* does **not** make the grant a purchase |
| Q-38 | does Greater Immunity's repeatability transfer through Lesser Immunity's cross-reference? | R | **SETTLED (revised, **arguable**) — `q-resolutions.md` § Q-29 / Q-38 (2026-09-22).** **No — adopt *once* for `virtue.lesser_immunity`.** Q-29 and Q-38 are one question and were answered together; see Q-29's row for the full reconciliation. **The first answer — the bare pointer at ArMDE:4277 carries the whole entry including ArMDE:4015's repeat clause — was overturned by the verification pass** (ArMDE:2814's *"only if the description **explicitly** allows it"*, and ArMDE:4283's Lesser Power stating its own repeatability), with **D10** breaking the tie: D10 exists precisely to stop a repeat being *inferred*. Flagged as the audit's one genuinely arguable reading, to be written up in `RULES.md`. **Remedy BLOCKED** on **F-141**'s parameter (without it "a different immunity each time" is unrepresentable and `max_per_target` has no target to count), then D10's explicit `max_total` — both entries are among D10's 42. **F-140, F-141** stand |
| Q-39 | `virtue.linguist`: the book rounds the XP **up**, the engine rounds the **cost** up — do they agree? | C | **SETTLED (revised) — `q-resolutions.md` § Q-39 (2026-09-22).** On the entry Q-39 names, **yes, they agree: `virtue.linguist` is clean at every reachable value** — every Ability cost in `abilities.json` is a multiple of 5, and at `T = 5k` both roundings give `4k`. **The first answer's impact table was overturned by the verification pass**: it was keyed by the player's *spend* `S`, where the engine is exercised by the *table cost* `T`, and the two identify different carriers. The formula survives — `effective/xp.rs::charged_cost` floors where the book ceils (ArMDE:3374 and ArMDE:4317 are word-for-word the same *gain* construction, so B05's special-case premise is false), and the correct charge is `(T − 1)·den/num + 1`. The live defect is on Linguist's **sibling**: `virtue.affinity_ability` is overcharged 1 XP at Ability scores 1, 4, 7, 10, 13, 16, 19 (costs ≡ 2 mod 3). Arts are provably clean (triangular costs are never ≡ 2 mod 3) and Flawless Magic's 2/1 is clean (the formulas coincide at `den = 1`). ArMDE:2443 endorses neither rounding and is the regression guard. Filed as **F-547**; raises caps, so no save is invalidated. Not blocked |
| Q-40 | `virtue.linguist`: the table's only row is in a supplement section and gives "Linguist" | X | **SETTLED — D31**: a *name*, so the table wins. Its being in a supplement section is a **placement** defect in the table, not grounds to reject the rendering; note the placement separately |
| Q-41 | `virtue.lone_redcap`: is `supernatural` in the 300-point pool sourced? | R | **SETTLED (challenged and CONFIRMED) — `q-resolutions.md` § Q-41 (2026-09-22).** The word is in neither passage, but the shipped list is **correct — do not remove `supernatural`.** ArMDE:4848's two sentences are separate: a *permission* over three gated categories, then a *quantity* with no category restriction, and the shipped `["academic","arcane","general","martial","supernatural"]` is all five of `AbilityCategory::ALL` — the **absence** of a funding restriction, not a restriction. The verification pass recommended dropping it on the premise that Supernatural Abilities are ungated; **that premise is false and the challenge does not survive**: the gate is per-Ability, not per-category — `validation/scores.rs::validate_supernatural_abilities` raises `CODE_SUPERNATURAL_ABILITY_REQUIRES_VIRTUE` and is called from `validation/mod.rs`. Nothing owed, nothing blocked. The same reasoning covers `virtue.mentored_by_demons`; it does **not** cover `virtue.privileged_upbringing`, whose narrower list is a genuine funding restriction (**Q-59**, untouched) |
| Q-42 | `virtue.lone_redcap` / `virtue.redcap`: nothing encodes that they are alternatives | R / N | **SETTLED on the rules question; remedy BLOCKED — `q-resolutions.md` § Q-42 (2026-09-22).** The book **does** make them mutually exclusive, but through **ArMDE:2816**, which this repo models nowhere — not by any sentence in either entry. ArMDE:4325's compatibility note is scoped to *mundane* statuses and ArMDE:4844 makes Redcap non-mundane, so :2816's default (only one) governs. So **no pair-specific `incompatible_with` row**: singling out the pair would encode a specific rule derived from a general one nobody has modelled, which is what `7f5605a` was reverted for. File it as a worked instance against **Q-132 / Q-07 / Q-102 / F-427**, carrying a design constraint that ruling does not currently record: the mechanism must express a **scoped** exception (*"compatible with any other **mundane** Social Status"*), not a boolean "may stack". The pair's incoherence (ArMDE:4321 against :4844, :4848) is entailment, not a stated exclusion, so it turns on **Q-19** and cannot carry the verdict |
| Q-43 | a granted Reputation has no **polarity**, so "a *poor* Reputation at level 2" is unrepresentable | — | **closed — D11: NO polarity field.** ArMDE:1093 gives a Reputation three components (score, content, type) and explicitly refuses to make good-versus-bad mechanical. "Bad" is **content**. Consequences: **F-363** stops being an engine gap, and **F-525**'s absolute becomes unenforceable by design |
| Q-44 | no `Effect` expresses free seasons per year, though three entries modify them | N | **SETTLED — D49**: **model it as a constraint on the creation plan**, not as text and not as a new sheet axis. B05 treated free seasons as in-play and therefore out of scope; that is wrong — `post_gauntlet_lab_seasons` is a **save-persisted creation-plan field** and `post_gauntlet_points` subtracts `lab_season_cost` per charged season from the **creation** budget (ArMDE:2471). So `virtue.landed_noble`, `virtue.license_of_absence` (cap **4**, ArMDE:4293) and `virtue.lone_redcap` change what the player may claim at creation. **F-131, F-146, F-151 change remedy** — text is now the lesser half. Mind that the field counts **charged** seasons, not available ones |
| Q-45 | ArMDE:2816 is not a cap, and one `virtue_category_caps` row cannot express it | N | **SETTLED — D41**: *"must take one"* is a **hard error** (a category **minimum**, which the budget model lacks); a **second** Social Status is a **warning, never an error**, and there is **no per-entry flag**. All three stated exceptions are predicates, not id lists — ArMDE:4325 is hedged (*"reasonably"*, so D16 forbids an error), ArMDE:4614 needs a sex model **Q-05/D16 ruled out as a deliberate non-goal**, and ArMDE:4441 *requires* a second status (a separate obligation, not discharged). Uses **D21**'s machinery; D41 says what it must *say* |
| Q-46 | the leather variant of the two vessel Virtues: entry, parameter, or nothing? | N | **SETTLED: a parameter — not a new entry, and not text alone.** **A new entry is ruled out by CLAUDE.md's own convention**: *"IDs are derived from the English source"*, and the leather version has **no `####` heading**, so `virtue.maker_of_leather_vessels` would be an id with nothing to derive it from. Text alone is not enough either, because the book says the variant *"**relies on Craft: Leatherworker**"* while the mechanics key on the Craft score — a different Ability, so a different number. And the shape already exists: `ability.craft` ships `"parameter": "craft"`, so the variant is simply **which Craft instance the Virtue keys on**. Encode it as a parameter on each of the two Virtues, its value a Craft instance constrained to {potter, leatherworker} — **D34**'s whitelist over **D48**'s instances, the same combination **Q-12** needs. The gendering (*"pottery … associated with women, leather with men"*) is explicitly **not** a restriction: *"this does not limit which characters can take either Virtue"* |
| Q-47 | `restricted_ability_xp` cannot scope a pool to one Profession, and three entries name one | N | **SETTLED — D48**: add `instances` to a V/F grant and make eligibility the **union** of `abilities`, `categories` and `instances`, replacing *"when non-empty it is the ONLY test"*. The union is forced by `virtue.master_bard`, whose **single** 50-point pool funds two named Professions **and** two whole categories — instances-wins cannot express it, and splitting the pool would invent a division of the 50. **Verified behaviour-preserving**: the only existing `instances` user (childhood native language) has both other lists empty. Narrowing, so affected saves report and block per D10. **Sweep the catalogue** — B06 measured its own span only |
| Q-48 | Mercurian Magic's companion Flaw: a prerequisite the player is paid for, or a budget-exempt grant? | N | **SETTLED — D51: a prerequisite.** ArMDE:4522's *"All known members of the Mercurian lineage **also have** the Minor Flaw Ceremonial Spontaneous Magic"* is a fact about the character, not something the Virtue hands over, so the player takes the Flaw and **receives its Minor Flaw point normally**. This is **F-196's proposal**, now grounded in the book's own wording rather than in a preference. ArMDE:4588's *"at no extra cost"* confirms the contrast |
| Q-49 | the reputation table attributes to Mythic Blood a Reputation ArMDE:4588 denies | X | **SETTLED — D6 rule 1**, and **D31 confirms it** (a rule fact, not a name), so this row stays a proven table error. Only the *fix* is outstanding, in the source project. — **D6 rule 1 decides it** (a terminology table making a factual claim), and **D18 does not rescue this one**: a newer edition changes *terminology*, not which Virtue grants which Reputation. The fix still needs the source project |
| Q-50 | `mythic_type.nephilim` carries neither of ArMDE:2731's two point adjustments | R / N | **SETTLED — `q-resolutions.md` § Q-50 (2026-09-22) — and the defect it turned up is already FIXED.** **It should not carry them:** ArMDE:2731 states no adjustment at all. The required list (ArMDE:2723-2730) prices to **10 bought Virtue points = 5 Flaw points** at the 2:1 rate, and the second clause's five more Flaw points for ten more Virtue points closes the standard **10 Flaw / 20 Virtue** budget exactly (ArMDE:2844). So `bonus_flaw_points: null` and `bonus_free_virtue_points: null` are **correct**, and either field would over-fund the type. Record the arithmetic in `RULES.md` so the absence reads as verified rather than unexamined (D25's closing precedent). The pass then rated `bonus_flaw_points: 7` on its two siblings a new `high` defect — **superseded by D32**, which is broader and already implemented: *every* Mythic Companion has 10 Flaw / 20 Virtue points, `bonus_free_virtue_points: 3` is **not** a sound transcription either, and both fields are now **deleted from the model** (commits `212d726`, `c3714b2`). Registered as **F-546, status `fixed`** |
| Q-51 | should Magical Blood's Magic Human variant become effects once a parameter exists? | N | **SETTLED: no — text for all four, until a *parameter-gated* effect exists.** F-165's `enumerated` parameter records **which** bloodline was chosen, but an `Effect` fires for **every** copy of the item regardless of the value held, so encoding Magic Human's *"positive Reputation at level 3"* and its Characteristic clause (ArMDE:4367) would apply them to Magic Animal, Magic Spirit and Magic Thing as well — **wrong for three players out of four**. That is the **F-45 / F-20 shape**, a conditional applied unconditionally, which this audit rates **HIGH** on two entries already; partial computation would buy one correct case by manufacturing three wrong ones. **F-164's conservative reading stands.** The missing machinery is *"this effect applies only when parameter X holds value Y"* — **parameter-*gated*, not parameter-*valued***; `characteristic_score_delta_param` is the latter and does not help. **A second caller exists**: D33's `flaw.flawed_powers`, whose imported Flaw's effects apply through a parameter — so design the gate with D33, D21 and D23 rather than alone |
| Q-52 | is `virtue.masterpiece` `creation_effect` or `in_play_effect`? | — | **answered in substance by Q-120's resolution** |
| Q-53 | the book names six Ability types; `AbilityCategory` has five | R / N | **SETTLED — `q-resolutions.md` § Q-53 (2026-09-22).** **Five is correct, and a sixth member would break a rule the book states.** ArMDE:9516: *"Spell mastery Abilities are their own category, and Virtues that give characters access to other categories of Ability **do not cover** spell mastery Abilities"* — a sixth member would sweep into `effective/xp.rs::magus_later_life_pool`, funding Spell Mastery from later-life XP, which is exactly what `PoolEligibility::Mastery` exists to prevent; and ArMDE:9518's *"for every possible Hermetic spell, there is a corresponding Ability"* is an open-ended set no catalogue can enumerate, so it is modelled per spell (`SpellSelection.mastery`). **No data change, no behaviour change, nothing blocked.** Q-53's premise that the divergence is undocumented is wrong — `RULES.md` already cites ArMDE:7143 and :9516; what is owed is a **doc comment on `ability.rs::AbilityCategory`**, where a reader counting members will notice |
| Q-54 | does a glossary table govern *inside a verbatim quotation*? | N | **SETTLED — D53: yes.** It turns on **D31**: "verbatim" means faithful to `rules/source/de/`, which is an **older copy than the tables**, so the fidelity is to the wrong thing. Glossary spelling wins inside quoted prose as well as on labels. **`virtue.mild_aging` is therefore not clean on check 12** — B06 rated it clean on the rulebook-governs reading. **Not a licence to rewrite quoted rules text**: D39 and the `scheitest` precedent stand; substitute the glossary's spelling of a glossary term and change nothing else. The German rulebook is **not** edited to match (D18). Sweep travels with **D36**'s German-prose pass. Original — **Q-63 family** |
| Q-55 | `virtue.physician_of_salerno`: is the granted Reputation Local or Academic? | R | **SETTLED — `q-resolutions.md` § Q-55 (2026-09-22).** **Neither is stated, and `local` is positively wrong** — ArMDE:4734 says he *"carries the reputation of the school **with him**"*, which is the one type a place-scoped Reputation excludes. Under **D11**'s own worked remedy the pin is dropped and the grant ships the **player-chosen wildcard** (`kind: None`); `academic` is the better guess but is still a guess dressed as data, and the repo's own table hedges (`reputationen.md` reads *Lokal / Akademisch*, which is an accurate report of an open question and **not** a D6 error). **This is F-450's defect on a second entry** — fix the two in one pass. `score: 2` stays exact. Nothing blocked. Original follows. — **narrowed by D11**: both are valid `ReputationType` members and the type set does **not** grow, so this is now a plain rules read, no longer "Q-73 family" |
| Q-56 | `virtue.perfectus`: should Purity and Transcendence enter the Ability catalogue? | N | **SETTLED — D22: no, not now.** ArMDE:4634 is a **core entry pointing at a supplement's content** — *"the Purity and Transcendence Supernatural Abilities **from Realms of Power: The Divine Revised Edition** (page 53)"* — and D22 holds that cross-book rules bind in principle while the **implementation stays core-book-only**. **All 78 catalogue Abilities are core-book today**; these two would be the first supplement entries, and D22 refuses the half-done state where some supplement content is in and most is not. Provenance is *not* the obstacle — RoP:D **is** in `rules/source/en/` — the **scope boundary** is. D22's first obligation applies: the debt is **visible**, and it already is, in **F-218** (the permission reaches neither locale). Note the clause is conditional on **True Faith** and says *"these are **not free** Virtues"*, so whenever they do land they are purchased normally |
| Q-57 | Potent Magic: how should the forced magnitude-variant incompatibility be lifted? | C / N | **SETTLED — `q-resolutions.md` § Q-57 (2026-09-22).** **Neither of B07's two ways out.** `ruleset/integrity.rs::validate_magnitude_variant_exclusivity` derives a *rules fact* from an **id spelling** and pushes a load error unless both entries declare an incompatibility ArMDE:4742 denies, so the catalogue ships a wrong value because the code compels it. Exempting the pair by name puts a catalogue id in Rust — the same separability violation in a smaller font — and renaming the ids breaks every save holding one (`Selection.item_ref` is the id) and discards the book's own *"Minor or Major"* descriptor. **The fix is D10's shape:** keep the default, add one optional `skip_serializing_if` data opt-out (as D11 did for `max_score` and D13 for `from_normal_budget`), then delete the two rows — which closes **F-226**. D10 has a second claim: *"more than one area"* makes both entries legitimately repeatable, so each needs an explicit `max_total` and a D9 **area** parameter, without which the repeats are indistinguishable. Not blocked. The doubled clause found in the English source at ArMDE:4742 is **F-549** |
| Q-58 | `virtue.ripper`: are the two fixed powers a `power_levels` grant or pre-entered powers? | N | **SETTLED from the source: pre-entered powers.** ArMDE:4868 specifies both completely — *"a **PeAn(He) 25** and a **PeAn 45** effect, each with **+0 Penetration**"*, Sight range, one Fatigue level, no words or gestures — and closes *"The ripper has two, **entirely inflexible**, effects."* A `power_levels` grant is a **budget the player spends designing powers**, which is exactly what "entirely inflexible" forbids; it would let a ripper build something the passage spends three sentences ruling out. The entry is `narrative` with no effects today, so **D50** also applies: these are rules a player must act on |
| Q-59 | `virtue.privileged_upbringing`: can a pool-scoped permission be expressed at all? | N | **SETTLED — D43, which is this question under another entry.** ArMDE:4808 (*"may not … buy Academic or Martial Abilities with your **normal pool** … unless you have another Virtue or Flaw permitting that"*) is the identical shape to ArMDE:4065, and D43 chose exactly B07's option **(b)**: separate the permission axis from the funding axis — the pool permits spending **itself**, general XP needs an explicit `AbilityAuthorization`. D43 also carries the 28-carrier survey B07 said it could not do, and the rule that narrowing and authorizing are **one slice**. **F-234 unblocked** |
| Q-60 | `virtue.personal_vis_source`: is "about one tenth" a rule or hedged guidance? | N | **SETTLED — D50**: both. It is hedged *and* a quantity the troupe must set (ArMDE:4730), so it is **`uncomputed_rule`** + text — while **D16** keeps it unenforceable. D50 governs whether a rule is carried, not how strictly it is checked. Original — **Q-83 family** |
| Q-61 | `virtue.rat_up_a_drainpipe`: is "a substantial advantage" mechanical? | — | **closed — D8: yes** |
| Q-62 | `virtue.powerful_relic`: is the relic's one power charged against the power-levels budget? | R | **SETTLED — `q-resolutions.md` § Q-62 (2026-09-22).** **No — and it must not be recorded in `Entity::powers` at all.** ArMDE:4784's subject throughout is an **item the character owns**, and the power's content is *"agreed upon with the storyguide"* with no level stated, so no budget can charge it; `effective::powers_used` charges every `Entity::powers` row against a budget this Virtue leaves at 0, so writing the relic's power down yields a false `over_power_levels`. A `power_levels` grant would invent a number (`7f5605a`), and `item_level_budget` counts Hermetic enchantment levels, which a Divine relic's power is not. **No new effect**; `true_faith_grant: 3` is correct and complete. The power and ArMDE:4786's impiety clause reach the player as text (D5/D20). Worth stating once beside `powers_used` that an owned item's power does not enter `Entity::powers` — it settles `virtue.relic` and `virtue.infernal_heirloom` too. **F-249 (`virtue.ripper`) is expressly NOT settled by this**: there the powers are the character's and their levels *are* stated — that is **Q-58**. Nothing blocked |
| Q-63 | does a translation table's term bind *inside a sentence*, or only on a label? | N | **SETTLED — D36**: it binds **wherever the word names a game element the app shows** (label, summary, description, export); ordinary-language use is untouched. Test: could the reader want to *find* the thing named? **F-65, F-320 and F-473 are unblocked** — F-473 is the decisive case, a description naming an element by a word that appears nowhere in the UI. **Enforced at review, deliberately no guard** (a term-scanner would fire on every ordinary use). Which term is canonical remains **D31**'s question, not this one |
| Q-64 | is a supernatural *capability* with no number, no roll and no waived penalty mechanical? | — | **closed — D8: yes.** Three independent supports, incl. ArMDE:2960-2962's realm association + same-realm-aura Warping immunity on **every** Supernatural Virtue, which falsifies the `narrative` claim for this cohort on grounds independent of the capability argument |
| Q-65 | two canonical tables give `Sense Holiness and Unholiness` different German names | X | **PARTLY settled — D31**: the rulebook heading no longer breaks the tie, so this is now **table vs table** and falls to D7 rule 3 (thematic table before `tugenden-fehler.md`). Decide on that alone; do **not** reach for the stale heading |
| Q-66 | does `virtue.sense_holiness_and_unholiness`'s "may overwhelm you" state a D5 rule? | N | **SETTLED — D50**: yes. It states a consequence a player would get wrong by not knowing it → **`uncomputed_rule`** + text; **D16** keeps the hedge unenforceable |
| Q-67 | what amount should `virtue.simple_student`'s restricted XP pool carry? | R / N | **SETTLED in full — rules read in `q-resolutions.md` § Q-67, model and cap in **D35** (2026-09-22): `ParamType` gains a **number** variant, `restricted_ability_xp` scales by it, and the cap is **2 finished years / 60 XP** because the third year is the Baccalaureus Artium (ArMDE:3472). No interim; the XP must reach the player.** **No single amount exists.** ArMDE:4960 states a **rate** — *"30 experience points per **finished year**"* — and fixes neither the year count (*"somewhere along his university program"*) nor a maximum, so there is no constant for `RestrictedAbilityXp.amount` and choosing one would invent a number (`7f5605a`). Meanwhile the entry ships `creation_effect` with **no `effects` array at all**, so a Simple Student currently receives **nothing** and neither locale says why. **Interim, not blocked, should land:** reclassify to `uncomputed_rule` and write the rule into `description` in both locales (D3, and D20's fidelity test, which the entry fails outright). **Exact model blocked** on a **numeric** parameter type plus a parameter-scaled `RestrictedAbilityXp` — a *third* parameter kind, not D9 part 3's multi-valued one, but worth designing alongside it. **For Norbert:** ship no pool + text (the recommendation, and arguably already forced by D3) or ship 60 (the bound before Baccalaureus). The same read found a **second female-only exception**, so D16/Q-05's count is short by one — **F-548** |
| Q-68 | does Subtle Magic's "no benefits from normal gestures" add anything to the Words/Gestures table? | R | **SETTLED — `q-resolutions.md` § Q-68 (2026-09-22).** **No: "normal gestures" means Bold, which the table prices at 0**, so the clause restates a zero. ArMDE:9236 defines *normal* for the table (*"normally cast with a firm voice and bold gestures"*) and the twin confirms it independently — `virtue.quiet_magic`'s *"using your voice normally"* is :9236's **Firm**, also 0, and cannot point at its own name's row because there is none. DE:5075's *"nicht mehr"* is a translator's intensifier, not evidence (English is the source of truth). **Nothing computed changes and nothing is blocked**; `RULES.md`'s residual **clamp at 0** already implements this reading, so only its justification needs re-founding on ArMDE:5075 and :4824. **F-301 shrinks** to the exaggerated-gestures half plus the waived *None* penalty, and **F-239 takes the same resolution** — write the two in one pass so the parallel wording stays parallel. Worth putting in the description: the clause blocks the inference that the whole column shifts |
| Q-69 | a table attributes a Reputation rule to `Social Contacts` that ArMDE:4990 does not state | X | **SETTLED — D6 rule 1**, and **D31 confirms it** (a rule fact, not a name). Only the fix is outstanding. — **D6 rule 1 decides it**; like Q-49, **D18 does not rescue it** (a factual claim, not terminology) |
| Q-70 | should `virtue.sense_passions` carry `tainted: true`? | R | **SETTLED — `q-resolutions.md` § Q-70 (2026-09-22).** **No.** `tainted` is exactly the descriptor tag, and that is **measured, not assumed**: 21 descriptor lines carrying *Tainted* against 23 ids carrying the flag, the gap fully explained by two *"Major **or** Minor"* descriptors that define two ids each (ArMDE:3411, ArMDE:6081). ArMDE:4931 carries no tag. The page-170 cross-reference does not override it: ArMDE:7737 is **disjunctive** (*"either a false power … or is associated with the Infernal"*), and ArMDE:3000 states Tainted ⇒ Infernal, never the converse. Decisive corroboration from the verification pass: `flaw.false_power`'s parameter carries `forbid_tainted: true`, so tagging the entry would make **False Power (Sense Passions) unselectable** — the very arm :7737 names first. **Nothing owed on the entry, nothing blocked.** Record the census in `RULES.md`; both sides are mechanically derivable, so it is also a cheap data-integrity guard |
| Q-71 | the German core rulebook gives `virtue.spirit_votary` two names and the tables give a third | X / N | **SETTLED — D31**: the third name is the **table's**, and on a *name* the table wins outright — the two rulebook renderings are both from the stale copy and no longer compete. (The entry's *magnitude/category* claim is a rule fact and stays with the rulebook) |
| Q-72 | what is the "Brother-Priest Status Virtue" that ArMDE:5111 names? | R | open — **if it is a fourth rank, F-34's census is not closed** |
| Q-73 | which `ReputationType` carries an **organization**-scoped Reputation? | — | **closed — D11: none, and the enum does not grow.** ArMDE:1093 names Local, Ecclesiastical and Hermetic, with Academic alongside, and the ease-factor table has exactly those columns. "Among Templars", "among the Jewish community" are **content** |
| Q-74 | how does True Faith's Magic Resistance join the per-Form grid — replace, stack, or compete? | N | **SETTLED — D37, and from the source rather than by choice**: ArMDE:2627 says Magic Resistance from more than one source *"does not stack … you simply use the higher total"*, so it **competes** — per Form, `max(existing, true_faith × 10)`. The book's two additive cases (ArMDE:4035, ArMDE:3583) say "add" **explicitly**, which confirms the default. **F-329's fix needs no new data** — `Effect::TrueFaithGrant` and `effective/might.rs::true_faith` already carry the score; only the consumer changes. Faith Points without a *Score* grant nothing (ArMDE:17611) |
| Q-75 | does ArMDE:2394's "only companions can take this" bind grogs? | N | **SETTLED — D38**: yes, and it is stated **on the entry** per D24, not in a third `forbidden_traits` list. Needs a `Prereq` naming a **character type** (only `IsMagus` exists, and it reads the profile flag rather than an id). Grogs are refused today **by accident** — both entries are Major and the grog has `max_major_*: 0` — so the rule would silently stop being enforced if either magnitude or cap moved, with **no test failing**. **F-339 unblocked** |
| Q-76 | is a roll-free supernatural *capability* `narrative` or `uncomputed_rule`? | — | **closed — D8: `uncomputed_rule`.** **48** of the 115 `supernatural` entries move (`jq`-verified). Enumerate them **from the data**. Largely a relabelling, not 96 new texts — a `description` is owed only where the `summary` does not already carry the rule. **Blocked behind § 2.1b**: the screen needs a capability idiom first |
| Q-77 | how should a free-text `{land}` placeholder render in German, where the article inflects? | N | **SETTLED — D57: apposition after a comma, uninflected**, the pattern `RULES.md:2356-2368` already uses for the four `realm` items and for Folk Magic. `Stimme des {land}` is right for *des Waldes* and wrong for *der Steppe*, and **no fixed article can be right for a word the player types**. B09 read *"a typed word inflects however the player typed it"* as a reason free text differs; it is a reason to keep the word **out of** a slot demanding agreement. The table's `Stimme des/der (Land)` is an **editorial flag, not a label** — D31 makes the table authoritative on a *term*, not on a notation. **Sweep the other ~36 mid-phrase German templates**; only the realm four are fixed. `name_unfilled` is correct on both and is **not** part of this |
| Q-78 | does `virtue.unaging`'s non-fatal-crisis clause have a home in the M6/6b7 crisis engine? | C | **SETTLED — `q-resolutions.md` § Q-78 (2026-09-22).** **Yes, and it is small and well-scoped.** B09 looked one level too low: the split ArMDE:5189 draws is the table's **outcome type**, not `CrisisSeverity`, and `aging.rs::CrisisOutcome` has exactly the two variants the rule needs — *not potentially fatal* **is** `Bedridden` (no roll, so no death possible) and *potentially fatal* **is** every `Illness` row. So: a carrier of `virtue.unaging` who rolls a `Bedridden` row suffers no ill-effect; every `Illness` row resolves unchanged. **No new `AgingEffect` kind, no new table column, no change to `CrisisSeverity`** — and it must **not** be routed through `AgingEffect::CrisisSurvival`, because ArMDE:16636 says Virtues affecting aging rolls do not affect crisis-survival rolls. Scope it to the outcome only: Decrepitude still accrues (ArMDE:16619, and :5189's own *"only building up to give you Decrepitude points"* / *"die as normal when you reach five"*). **F-332's text obligation stands alongside** and is not replaced. Not blocked |
| Q-79 | the book files two Flaws under a magnitude their own descriptor contradicts | R | **SETTLED (by D25) — `q-resolutions.md` § Q-79 (2026-09-22).** **The descriptor wins and `magnitude: "minor"` stands on both entries.** Verified line by line: the index heading `### Supernatural, Major` is ArMDE:5387 and both entries' index lines (ArMDE:5392 and :5393) sit under it, while their own descriptors (ArMDE:5736, :5754) read *Minor, Supernatural* — and both index lines are literally `[Name](#anchor)<br>` link lines, which D25's obligation 3 already calls unreliable (134 dead link targets). The content test agrees: ArMDE:5755's 5-XP loss and one-level botch loss is Minor-scale. **No data change, nothing blocked.** Two more rows in **D25's `RULES.md` errata note**, which must be worded to hold the **magnitude** axis as well as the category one. This answers D25's obligation 2 for B10's span — **the sweep will return a list, not a single row** (B19 one, B10 two, B16 one via Q-125), and should be budgeted as such |
| Q-80 | is `flaw.black_sheep`'s "a bad Reputation **of your choice** at level 2" a wildcard? | — | **closed — D11: already solved.** `GrantsReputation.kind: Option<..>` with `None` *means* player-chosen, and `virtue.famous` (ArMDE:3861) already uses it. `flaw.infamous` pinning `kind: "local"` is the defect (F-450) |
| Q-81 | which `casting_fatigue` sign convention is intended? | C / N | **SETTLED (revised) — `q-resolutions.md` § Q-81 (2026-09-22).** **The data convention is right; the read-out is the defect.** **The first answer — flip all three shipped amounts — was overturned by the verification pass and is withdrawn**: `derived/combat.rs::fatigue_levels` proves the convention arithmetically (*"A positive delta reduces magnitude; never flip a penalty positive"*, `penalty: (base + delta).min(0)`), and a **12-row `health_mod` census across ten entries** finds *positive = better for the character* obeyed everywhere, including two computed penalty tracks. **What both passes found independently, and what matters, is that the user reads a backwards number today**: the Fluent label `Casting fatigue` / `Zauber-Erschöpfung` names a *cost*, so a Withstand Casting magus — a **Virtue** — is shown *"Casting fatigue: +1"*. **Revised remedy for F-351:** fix the doc comment on `types.rs::HealthTrack::CastingFatigue` and rename the label's value in **both** locales; **leave the three amounts alone** — cheaper, touches no shipped data, preserves the family consistency. **F-352 is confirmed outright** (ArMDE:7003 and :5269 state the creation-time exclusion verbatim on both sides), and both entries join **D10's 42**, since both passages explicitly permit repeats (*"Vulnerable Casting (2)"*, *"Withstand Casting (3)"*) and need an explicit `max_total`. `flaw.painful_magic` is mis-**tracked** rather than mis-signed (ArMDE:6576 counts pain levels for *every* spell cast, recovered separately) — flag it for the D20 text pass, do not invent a fourth track. Original follows. — **F-351 depends on it** |
| Q-82 | may the German source be used to repair a truncated **English** sentence? | N | **SETTLED — D39, and FIXED.** Not by back-translation: the repair is an **English-side reconstruction from the surviving fragment**, with the German as corroboration only. ArMDE:5725 now reads *"…for a few weeks, **but** his last **for** years."* No i18n change — the shipped `summary` carries only the first sentence. **Test for the next one:** if the surviving English does not determine the missing words, do not reconstruct — write the rule into `description` (D20) as honestly our prose. **F-370's source half is discharged**; its `narrative` half stands |
| Q-83 | is a hedged monetary value a mechanical clause for classification purposes? | N | **SETTLED — D50, the ruling this family is named for.** The question dissolves: "mechanical" is not the test. A hedged monetary value is a number a player must act on → **`uncomputed_rule`** + text, unenforced per **D16** |
| Q-84 | what entity or type should hold `flaw.abandoned_apprentice`? | N | **SETTLED — D56: a companion, and `is_magus` splits.** He is an ordinary companion at **1:1** (not Mythic — the 2:1 rule does not apply) holding a Major Story Flaw worth **3 points**. `is_magus` separates into *Hermetically trained* (Arts phase, Arcane authorization, XP shape, **D12**'s gate) and *member of the Order* (Houses); the Flaw confers **training**, while the **Gift is selected** (`gift_policy: "allowed"`, Free) and required via `Prereq::Has` per **D51**. **XP is truncated apprenticeship + later life**, not post-gauntlet — he never had a Gauntlet (ArMDE:5647). UI: conditional `creation_phases` reusing `CategoryRule`'s `{ …, when: <Prereq> }` shape. **The rate is settled**: **16 XP and 8 spell levels per year** (ArMDE:2435's 240 and 120 over fifteen years — **both** divide). The rate is ours, the book pricing apprenticeship as a lump, and **this unblocks guided-creation #15**. Parma allowed **with a warning**, not excluded (ArMDE:5648/:5650); `apprenticeship.minimum_abilities` **not** enforced on a truncated block |
| Q-85 | (the `flaw.bound_to_realm` German-name question) | — | **withdrawn — answered from the repository; see withdrawn F-373** |
| Q-86 | should every "choose the specifics" Flaw carry a parameter? | — | **closed — D9: YES, record every stated choice**, `enumerated` where the book lists the options and `text` only where the choice is genuinely open. **Catalogue-wide** — § 3.7's 26 findings are *not* the full set; the slice re-derives it. **Multiplicity is D10's, not D9's** |
| Q-87 | should `flaw.broken_vessel` carry an enumerated `prerequisites` tree? | N | **SETTLED: no — enumerating it would freeze catalogue size into data.** ArMDE:5755 requires *"at least one **Supernatural Ability or Art** normally improved through experience points"*. Every `Prereq` variant names a **specific** thing (`Has`, `AbilityMin`, `ArtMin`, `House`), so a tree means an `Any` over **all fifteen Arts** plus one clause per Supernatural Ability — and CLAUDE.md's *"catalogue size is data, never code"* forbids exactly that, as **D23** argued when refusing an id list for the same reason. It is **D21**'s work: a prerequisite that ranges over the `supernatural` **Ability category**, plus an *any-Art* predicate. **One qualifier the catalogue cannot yet express**: *"normally improved through experience points"* excludes Might-driven Powers, and no Ability property records that — check before encoding, and do not silently drop it |
| Q-88 | should a player's free-text choice of *scope* be recorded by a parameter? | — | **closed — D9: yes** |
| Q-89 | ArMDE:5911 says "Technique" inside *Deficient Form*'s own passage — misprint or not? | R | **SETTLED — `q-resolutions.md` § Q-89 (2026-09-22).** **A misprint in the book, not in this repo.** ArMDE:5911's final sentence is byte-identical to ArMDE:5915's, where it is correct — a copy-paste from the neighbouring entry — and **the German carries it too** (DE:5911 reads *"der **Technik**"*), so it is neither an OCR slip (D26's carve-out needs the error to be the OCR's, and `Technique`/`Form` is not an OCR confusion) nor a translation slip. Two independent renderings locate it in the printed book. **No change to `rules/source/en/`, `rules/source/de/` or the shipped strings** — the `scheitest` precedent governs and D26 does not reach it. One more row in **D25's `RULES.md` errata note**. Nothing computes off it either way: `effective/xp.rs` never consults `effective/art.rs::deficient_arts`, so XP is already costed off the unhalved score for **both** Arts. **F-400 is independent and stands.** Nothing blocked |
| Q-90 | which of the two Deficient Art entries has the right shape? | — | **closed — D12: neither, and `flaw.deficient_form` is the one missing its gate.** Hermetic V/F split **intrinsic** (operates on The Gift itself — exactly the three entries already gating on `virtue.the_gift`) from **trained** (Techniques, Forms, spells, Casting/Lab Totals, Parma, certámen, Twilight → **magus only**). Obliges classifying all **122** `hermetic` entries — measured: 41 mechanically obvious, 56 carry no effects at all, ~25 need checking — **derived from the batch files, not a re-read**. Use `Prereq::IsMagus`, and normalise `flaw.deficient_technique`'s `Has` spelling to it |
| Q-91 | does ArMDE:5895's third sentence change **D4**'s answer for Cyclic Magic? | N | **SETTLED — D52: yes, and D4's row splits.** The book is deliberately asymmetric — the Virtue's Lab bonus applies *"**if** the positive part of the cycle covers the whole season"* (ArMDE:3637), the Flaw's penalty *"**even if** the negative period does not cover the whole of the season"* (ArMDE:5895). Both fix the halves as **equal in length**, so for a solar or lunar cycle every season contains negative time: the penalty always applies and the bonus never can. The Virtue's *"no"* stands; the Flaw's is *"yes, unless the cycle is seasonal"*, and the cycle type is an **unrecorded choice** (D9), so D9 lands first. Filed as **F-555**. Original — **D4's own slice needs it** |
| Q-92 | how should a *mandatory earmark of the normal budget* be encoded? | — | **closed — D13: add `from_normal_budget` to `RestrictedAbilityXp` and model both clauses exactly.** An engine change for **one entry in 655** (the phrase occurs once in the rulebook) and the cheaper approximation was overruled. **§ 3.17**, and **D14 first** |
| Q-93 | which treatment is intended for the three Corrupted entries? | — | **closed — D15: all three `uncomputed_rule`**, full rule in `description` in both locales, and **`flaw.corrupted_spells`' `special_casting_mod` deleted** — which needs its own test, because a Casting Total silently changes. Unifying *upward* was rejected: Corrupted Abilities' ±3 is on an **Ability roll**, not a Casting Total. Orthogonal to D9, which still owes all three the multi-valued parameter |
| Q-94 | `grants_selection` for `flaw.a_deal_with_the_devil`, and does a granted Story Flaw count toward the cap? | N | **SETTLED — D51, both halves.** ArMDE:5907 says the Flaw *"includes the **effects of** Plagued By Supernatural Entity"* — **effects only, the item is never held**. So it is not `grants_selection`, and the cap question **does not arise**: there is no held Flaw to count. **Check the engine can copy an effect without granting the item**; if it cannot, that is a finding, not a reason to fall back on `grants_selection` |
| Q-95 | can a parameter express "one or more, your choice"? | — | **closed — D9 part 3: not today, so the model gains a multi-valued parameter type.** The only part of D9 needing a **`SCHEMA_VERSION` bump**; pulls in `ParameterDef`, a migration, a multi-select UI picker and a canonical form for `max_per_target` grouping |
| Q-96 | is the partial `source.anchor` coverage a backfill in progress or an inconsistency? | C / N | **SETTLED — D30.** Mandatory **catalogue-wide**. V/F is done (655/655 EN + DE); **563 refs in the other 11 core files have none** (360 of them spells). Backfill, then make the field non-optional — type change last. Original follows (its 14.4 % figure is **stale**). — **RE-RATED UP by D18**: **94 of 655 (14.4 %)**, and B19 shows the 94 are a by-product of the description-extraction pass, not a principled subset. Since the anchor survives a re-pagination and a line range does not, **backfilling to 100 % is the prerequisite for ever accepting a newer rulebook**. Decide first whether `anchor` is mandatory or dropped; D18 points at mandatory |
| Q-97 | `RULES.md` declares a `source.lines` range wrong that the file does not have | C | **SETTLED — D30.** `RULES.md`'s form wins: a range ends on the last non-blank body line. `RULES.md` and check 1 must be brought into agreement — their disagreement is what let both forms ship |
| Q-98 | how should `flaw.enfeebled`'s "double the normal number of Fatigue levels" be encoded? | N | **SETTLED: as text, not as a number.** `casting_fatigue` takes a **flat signed amount** and, per **D45**, is *surfaced and never computed* — the engine does not know what a spell costs, so it cannot double it. **D3** makes that grounds for `uncomputed_rule` + text, never for `narrative`, and **D50** obliges the text anyway. **Do not approximate it as `-1`**: a doubling is not a flat delta, and a wrong number is worse than a stated rule. F-410's **other** half — *"unable to learn Martial Abilities or any other skills involving physical exertion"* — is the Ability-category prohibition axis and belongs to **D21**, not here |
| Q-99 | does RoP:D:5671 oblige anything of `flaw.dhimmi`, and does a supplement bind a core entry? | N | **SETTLED.** Policy half: **D22** — a supplement **does** bind in principle, the implementation stays core-book-only, and every known cross-book rule is **filed as a finding** so the debt is visible. Entry half, read from the source: RoP:D:5671 obliges **nothing implementable** — it requires Dhimmi of *"almost all non-Muslim characters **living under Muslim rule**"*, a saga setting the engine does not model, and it is hedged twice (*"almost all"*, *"tend to be"*), so **D16** bars an error either way; it also exempts Hermetic magi. Filed as **F-554** per D22's first obligation |
| Q-100 | should `SurfacedModifier` carry the id of the item that produced it? | N | **SETTLED — D45**: yes, the list is a **diagnostic**. `source: Id` on the DTO, populated in `derived.rs::in_play_mods` (which already holds the item), rendered **through the label map, never as the id**. Applies to **all five `ModifierFamily` values**. Note B12's strongest argument — that the anonymous row was the player's only information — was **removed by D20**, so the ruling rests on the duplicate-row defect instead. De-duplicating the list is a **separate** UI call, not folded in. **F-423 unblocked** |
| Q-101 | how should "once per Power" be encoded, on the twin entries that state it? | N | **SETTLED from the source — B12's reading (a): give the Flaw the Virtue's shape.** Both passages carry the **same conditional** — *"may be taken more than once, **if the character has more than one Power**"* (ArMDE:5205, :5948) — and that clause is meaningless as a gate unless each instance attaches to a **distinct** Power. The Virtue spells the consequence out (*"but it only applies once to a single power"*) and the Flaw elides it; the condition still does the work, so the omission is not a different rule. `flaw.deteriorating_power` therefore gains a `power` parameter (`domain: text`, `require_power: true`) and `max_per_target` returns to **1** from 255; drop it from `RULES.md:1509`'s 255 table. **Precedent already set in-repo**: `RULES.md:1534-1539` made exactly this move for `flaw.flawed_parma_magica` and `flaw.limited_magic_resistance`. **D10** agrees — both entries explicitly allow repetition, so the *total* is not capped; only the per-target count is |
| Q-102 | is ArMDE:2816's total absence from the engine a deliberate deferral or a gap? | N | **SETTLED — D21**: a **gap**. D21 names Q-07 and Q-102 together as "the same ArMDE:2816 machinery" and closes both with the category mechanism, so nothing was ever deliberately deferred here. Original: **F-427** |
| Q-103 | `flaw.false_power`'s `require_categories` admits any Hermetic or Special Virtue where the book says *Supernatural* | R / C | **FULLY SETTLED — rules question in `q-resolutions.md` § Q-103, remedy in D34 (2026-09-22): whitelist `virtue.diedne_magic` + `virtue.the_gift` alongside the `supernatural` category, via a one-id field on `ParameterDef` designed with D14.** Neither of B13's options is what the book says: ArMDE:6082 states a **predicate** whose head noun is *Supernatural Virtues* and marks its own open-endedness with *"**like** Faerie Blood, Diedne Magic, or even The Gift"* — so *"like"* defeats a whitelist, and the shipped three-category list is reverse-engineered from the three examples' categories (one each), which is the signature of a set inferred from three points. **The first answer — the whole shape needs D23's predicate plus a new data property — was overstated, and the verification pass measured it away**: `special` contains **exactly one** Virtue (`virtue.the_gift`) and `supernatural` **is** the stated class, so **two of the three arms are already exact** and the over-permission is one arm, **55 entries**. ArMDE:6096 is the warrant for `forbid_tainted: true`. **For Norbert:** a one-id whitelist on the `hermetic` arm naming `virtue.diedne_magic` (exact for today's catalogue, closed where the book is open, ~1 optional `ParameterDef` field, lands beside D14), route it through **D23**'s predicate (open, but needs a property the catalogue does not carry — `jq` finds **zero** entries with a `realm` key), or keep 56. **Both passes agree the status quo is wrong**; until it moves, `RULES.md`'s record must say the list is an approximation with a known over-permission, as D14 required of the Covenant Upbringing record |
| Q-104 | `flaw.flawed_powers` records no parameter for the Flaw it imports, where `flaw.false_power` does | N | **SETTLED — D33**, which obliges exactly this: the entry *gains a parameter naming the imported Major Hermetic Flaw*, with a constraint excluding the only-Hermetic ones. D33 reached it from the other direction (is ArMDE:6148 an import restriction or an incompatibility?) and noted the missing parameter as **a D9 instance** in passing — this row is the same defect seen head-on. **Design with D34**, which adds the sibling whitelist to `ParameterDef` for `flaw.false_power` |
| Q-105 | is `flaw.form_monstrosity`'s "1 pawn of Muto vis" a mechanical clause? | N | **SETTLED — D50**: a stated quantity a player must act on → **`uncomputed_rule`** + text, whether or not the engine models vis |
| Q-106 | what shape should F-428's correction take? | N | **SETTLED — D40**: one effect naming a life stage, `replaces_life_stage_xp { stage, amount, abilities }`. **Two carriers, designed together** — F-428 (`childhood`, 120) and **F-439** (the Redcap's 300, `apprenticeship`, D17) — so a variant shaped around childhood alone would not fit. Rejected: a suppress-flag-plus-additive-grant (splits one rule across two effects, and the suppression could ship without the grant), and engine special-casing (rules in code). **Not covered and still owed:** ArMDE:6113's Language prohibition and wilderness-Abilities restriction, which are authorization rules, not XP. **F-428 unblocked** |
| Q-107 | should the two Maledictions also carry an *open grant*, on top of F-442's reclassification? | N | **SETTLED from the source: no open grant.** Both passages are pure **calibration guidelines** — ArMDE:6212 *"The effects of the curse **should be comparable to** those of other Major Flaws"*, ArMDE:6344 *"**about as bad as** other Minor General Flaws"* — and neither says the curse **is** another Flaw. Under **D51** nothing is granted, because the book names no giver and no gift; an open grant would make the player pick a Flaw id and import its mechanics, which no sentence asks for. Under **D50** the calibration is still a rule a player would get wrong without it → `uncomputed_rule` + text, and under **D16** the double hedge (*"should be"*, *"almost any"*, *"about as bad as"*) keeps it unenforceable. **So F-442's reclassification is the whole remedy.** ⚠ **F-442's summary claims "a named immunity"; neither passage contains one** — both are two sentences long and quoted in full here. Verify before working it |
| Q-108 | does `virtue.lone_redcap`'s 300 XP **replace** the funding for its fifteen apprentice years, or supplement it? | — | **closed — D17: it REPLACES.** Resolved *from the source*, not by choice: ArMDE:4848 and :4321 describe the same 300 for the same fifteen years, and `life_stages.json` already models a 15-year, 240-XP apprenticeship block. **F-439 unblocked and confirmed `high`** (~225 XP over-funded). A second defect fell out: `virtue.redcap` encodes its 300 **not at all**, so the pair is wrong in opposite directions. Remedy is **§ 3.17**'s third mode |
| Q-109 | `flaw.fury` and `virtue.berserk` state the same condition, encoded two contradictory ways | R / C | **SETTLED — `q-resolutions.md` § Q-109 (2026-09-22), and both derivations reached it independently.** **They converge on `flaw.fury`'s side.** Neither condition is a character-generation fact (ArMDE:3502 needs a 9+ stress die after a wound; ArMDE:6196 a failed 9+ roll on a provoking event), and **D4** — not D1, whose scope paragraph confines it to `spell_level_cap` — rules that a condition creation cannot resolve must not be folded flat into a number printed as a result. A combat total is exactly that. So `virtue.berserk`'s `combat_mod {attack +2}`, `combat_mod {defense -2}` and `soak_mod +2` are the defect (**F-20 confirmed**, and `derived.rs::in_play_mods` sums them unconditionally today), and **`flaw.fury` is clean and needs no change** — *"-1 on all other scores and rolls"* is broader than any effect vocabulary in the engine anyway. Delete the three effects, move to `uncomputed_rule` with the conditional bonuses in `description` in both locales (D5/D20), and give it **its own test** per D15's `flaw.corrupted_spells` shape: a Soak total and two combat totals change silently for every Berserk character. Berserk's unencoded *"automatically gain the Personality Trait Angry +2"* is **F-19** and should be designed with **Q-110**'s family. Not blocked |
| Q-110 | ArMDE:6124's two +4 traits contradict ArMDE:2502's ±3 for a Minor Personality Flaw | R | **SETTLED — `q-resolutions.md` § Q-110 (2026-09-22).** **There is no contradiction.** ArMDE:1075 hedges three times in two sentences (*"normally range between +3 and -3, **although there are exceptions**"*, *"would normally"*, *"would justify"*) and ArMDE:2502 exceeds its own range one clause after stating it, while ArMDE:6124 **commands** (*"**Select** a Personality Trait at +4, and its opposite at +4"*). **D16's table therefore inverts today's behaviour:** `validation/scores.rs::validate_personality_traits` enforces the hedged guideline as a hard error and raises **two** of them on a Minor Flaw whose own shipped description instructs the player to build it. B13's option (a) is right and (b) and (c) are excluded — (b) leaves the app rejecting a legal character, (c) invents a number (`7f5605a`). So: the ±3/±6 check becomes a **warning** and the ±6 hard cap goes with it, and **F-440 is unblocked** with its fix in the validator, not the data. **Remedy BLOCKED** on a "mandates N traits at value M" field, which nothing in `PointItem` expresses — a family of **at least five** (ArMDE:3502, :4375, :6124, :6903, :7078) that must also express **narrowing** (ArMDE:7078 constrains *all* traits to +1…-1, which is **F-542**'s observation from the other side), so design it with D21's Effect-side twin. The trait names are a **D9** `enumerated` choice |
| Q-111 | does `flaw.independent_craftsman`'s recategorization clause apply, given *City and Guild* is not in the repo? | N | **SETTLED — D54: yes, ship `personality` now.** The clause is in the **core book** (ArMDE:6304), so this is not a cross-book provenance question at all — it is a core conditional referencing another book, and the app satisfies the condition today. A later supplement-selection feature will flip it back to `general` when City and Guild is in play; the entry is therefore listed in **§ 8a**, because it is invisible from that feature. **Measured: one instance** — *"treat this as a"* returns two hits and the other is a creature rule. Narrows buildability (`personality` caps at 2), so saves report per D10 |
| Q-112 | does `virtue.lone_redcap` satisfy `flaw.hermetic_patron`'s "a Redcap"? | R / N | **SETTLED — `q-resolutions.md` § Q-112 (2026-09-22).** **Yes** — ArMDE:4321's first six words are *"You are a Redcap"*, and nothing in ArMDE:6248-6255 narrows the word (ArMDE:6252 uses it unqualified again). **F-448 is unblocked** and its `Any([Has(virtue.redcap), Has(virtue.lone_redcap), IsMagus])` is right, with `Prereq::IsMagus` for the magus arm per D12's normalisation. **But Q-112's stated aim is the wrong aim:** the source says the two Redcap prerequisites **must** differ. `virtue.magic_items` naming `virtue.redcap` alone is **correct**, because ArMDE:4321 itself says a Lone Redcap *"do[es] not receive magic items"* — a character who receives none cannot begin with 25 more levels of them. **Do not widen it;** note ArMDE:4321 at the entry or in `RULES.md` so the next reader does not "harmonise" the two. A stronger `incompatible_with` there is arguably owed and is **flagged, not asserted** (nothing is reachable today — `prereq_not_met` already fires). D17 untouched. Nothing blocked. Original follows. — **F-448's fix depends on it** |
| Q-113 | should an `advancement_mod` marker of `amount: 0` render as a word? | N | **SETTLED — D55, and the question was aimed at the symptom.** The defect is that **`amount: 0` means two different things** — *"no magnitude carried"* on the surfaced kinds, and *"halve advancement"* on `flaw.incomprehensible` and `flaw.loose_magic` (`RULES.md:5159`) — so the UI guard `{#if m.amount !== 0}` is right for one reading and wrong for the other. Ruling: **give `advancement_mod` a factor** and retire 0 as a marker, leaving it one meaning. Unlike **Q-98**'s doubling (which sits on the surfaced-only `casting_fatigue` and cannot be computed at all), `advancement_mod` **is** computed and carries real values, so the halving can be a number. Rendering then needs no separate ruling — **D45** names the source beside it |
| Q-114 | under D6 as corrected, does the glossary or the shipped data win on F-456 and F-457? | N | **SETTLED — D31**: the **glossary** wins; both are names. The shipped values were generated from the stale rulebook, so "shipped disagrees with the glossary" is now evidence *against* the shipped value |
| Q-115 | is `flaw.inscribed_shadow`'s House Criamon restriction a prerequisite to encode? | R / N | **SETTLED (revised) — `q-resolutions.md` § Q-115 (2026-09-22).** **No hard prerequisite** — ArMDE:6320's absolute is *stigmata* (*"**Only** characters with stigmata may have this Flaw"*), which the engine models nowhere and the entry already ships as text under D3; the House clause is a hedged gloss, and a `Prereq::House` would forbid the non-Criamon stigmatic the sentence's own *"normally"* leaves room for. **But the first answer's "and no warning either" half was overturned by the verification pass**: **D16 closed Q-139 on identical wording** (ArMDE:6957's *"is **generally** restricted to magi of House Verditius"*) with the standing instruction to *"apply this wherever the book hedges, rather than escalating each instance"*. So **one warning is owed**: holds `flaw.inscribed_shadow` and is not House Criamon → **warning, never an error**, and per D2 the check must see *granted* Flaws. The widened sweep (*"generally/normally restricted"*) returns **5** hits of which **2** restrict an entry — ArMDE:6320 and ArMDE:6957 — so the population is **two**, not one. **Remedy BLOCKED: `Prereq` has no warning severity at the entry level** (`prereq_not_met` is an error), so D16's hedged→warning rule has no carrier here — registered as **F-550**, and Q-115 is what makes that gap live rather than latent |
| Q-116 | the book defines "combat scores"; the repo's reading excludes three of the five | R / C | **SETTLED — `q-resolutions.md` § Q-116 (2026-09-22), reached independently by both derivations, reduction included.** **ArMDE:16656's defined term governs** — *"Characters have **five combat scores: Initiative, Attack, Defense, Damage, and Soak**"* — and the book distinguishes *scores* from *rolls*, which `RULES.md` does not: ArMDE:22809 says *"All combat rolls use stress dice"*, and of the five formulas at ArMDE:16658-16666 exactly three end in a stress die. So `flaw.lame`'s *"other combat **scores**"* (ArMDE:6332) and `flaw.hobbled`'s *"other combat **rolls**"* (ArMDE:6262) are **not** saying the same thing, and any reading that collapses them cannot produce the difference. `flaw.lame` gains `combat_mod -1` on `initiative` and `damage` plus `soak_mod -1` (the existing `-3 defense weapon:weapon.dodge` row is correct and stays); `flaw.hobbled` gains `combat_mod -6` on `initiative` only. **No engine change** — `CombatStat` has all four and Soak rides `soak_mod`. **Scope is two entries, not six**: the other four B14 predicted use neither phrase (ArMDE:6436, :6608, :6580, :6440) and `RULES.md`'s reading is untouched for them. `RULES.md`'s *"combat rolls / combat scores"* block is **rewritten, not amended** (both its conclusion and its warrant stop being true), row 37 of `docs/open-todos.md` **reopens**, and `data_integrity.rs::the_combat_roll_flaws_penalize_the_combat_ability_totals` is the test that must go red first. **Both entries stop being unrated: defective at `high`** — wrong numbers on a printed sheet. Not blocked. Original follows. — **overturned the clean verdicts on `flaw.hobbled` and `flaw.lame`** |
| Q-117 | two in-repo authorities give opposite readings to "the passage states the *absence* of a rule" | C / N | **SETTLED (by D20 + D3, applied) — `q-resolutions.md` § Q-117 (2026-09-22).** **`RULES.md`'s principle survives; the `NO_RULE_DESPITE_TOKEN` exemption does not**, and D20 post-dates both authorities. ArMDE:6324 and ArMDE:6266 are the same shape — a sentence written to forestall a penalty the rest of the entry invites (that a legless character moves like `flaw.lame` or `flaw.hobbled`) — and a sentence written to stop a player applying a penalty is a rule the player needs. `flaw.horrifying_appearance_snake_legs` ships `narrative` with **no `description` in either locale** and a `summary` stopping at sentence one, so neither of D20's two routes happens: that is D20's silent drop verbatim. D3 independently forbids grounding `narrative` in what the engine computes. So: **`narrative` → `uncomputed_rule`** with the passage in both locales (DE:6266 is already complete and line-parallel), and the exemption row is **deleted, not reworded**. `jq` confirms the entry is `supernatural` + `narrative`, i.e. already one of **D8's 48** by D8's own enumeration rule — check that list before filing it as a separate move. **Remedy BLOCKED behind § 2.1b / D19**: the new text carries no signed number and no botch term, so the screen needs a *"declines to impose a penalty"* family first (*"nicht behindert"*, *"muss … weder … noch"* — both **discontinuous**, which is why D19 ruled for regex). Original follows. — `NO_RULE_DESPITE_TOKEN` versus `RULES.md:4684` |
| Q-118 | is `flaw.monastic_vows_hermetic`'s "you cannot own vis" mechanical, given the engine models no vis? | N | **SETTLED — D50**: the engine's inability is irrelevant to the question (D3 said as much). An absolute prohibition a player must act on → **`uncomputed_rule`** + text; **D16** makes it text rather than a hard error, since nothing can check it |
| Q-119 | is `flaw.necessary_condition`'s "you cannot cast spells at all" a mechanical absolute or a fiction condition? | N | **SETTLED — D50**: the distinction does not survive the test. It is an absolute a player must act on → **`uncomputed_rule`** + text |
| Q-120 | is an entry computing in both phases `creation_effect` or `in_play_effect`? | — | **closed — the enum's own doc comment: `creation_effect`** |
| Q-121 | four terminology-table rows attribute core-book Flaws to supplements — at what count does "noted" become a finding? | N | **SETTLED — the count is irrelevant.** "Which book renders this entry" is a **factual claim about the rules**, so D6 rule 1 decides each row on its own and **all four are findings**; D31 does not rescue them, because a translation revision changes wording, not which book an entry is in. **F-501 raises the same rows**; D7 rule 3 stays the test for *which heading a row governs* (F-506 is the worked case) |
| Q-122 | should `flaw.magical_fascination`'s authorization name both Lores, or record the player's choice? | — | **closed — D9: record the choice.** **D14 supplies the missing half**: an ability reference in an effect gains a **binding** to the selecting entry's own parameter, which is exactly this shape (`virtue.student_of_realm` is its sibling defect) |
| Q-123 | two entries state a *soft* restriction on magi, and the engine has only hard blocks | — | **closed — D16: emit a WARNING, never an error.** No new machinery — `IssueSeverity::Warning` / `ValidationIssue::warning` exist (`validation/mod.rs:73`, `:844`). A `forbidden_traits` row would say something the book (*"normally"*, *"not suitable"*) does not, which is the class of error `7f5605a` was reverted for. **Apply the hedged-versus-absolute rule wherever the book hedges, rather than escalating each instance** |
| Q-124 | this span carries both `source.lines` conventions | C / N | **SETTLED — D30** (with Q-97): a range ends on the **last non-blank body line**. That is the *minority* form (B16: 32 of 35 use the other), so most rows change; normalise mechanically from the anchors, and fix `RULES.md` and check 1 to agree |
| Q-125 | the rulebook contradicts itself about `flaw.prohibition`'s category | R | **SETTLED (by D25) — `q-resolutions.md` § Q-125 (2026-09-22).** **The shipped `["supernatural"]` is right, by two independent margins.** Verified line by line: the descriptor (ArMDE:6639, *Minor, Supernatural*) and the index **agree** here — ArMDE:5552's link sits under the `### Supernatural, Minor` heading at :5534 — unlike Q-79 and unlike D25's own case; and the dissent (ArMDE:6719) is a passing cross-reference inside a *different* entry, weaker even than an index line. **The mechanical test agrees too, which D25's own case did not have:** a `personality` reading would consume a Personality-Flaw slot (ArMDE:2820) and feed `validate_personality_traits`' Major-Flaw budget, while :6719's own words are that the grant *"does not count toward the character's total number of Virtues and Flaws"*. **No data change, nothing blocked**; one more row in **D25's `RULES.md` errata note** carrying all three citations and that argument, since the dissent cannot simply be adopted. **F-498 is unaffected and stands** — it names the right entry whichever category it carries |
| Q-126 | is an entry whose only mechanical content is a *selection restriction* mechanical? | — | **closed — D8: yes** |
| Q-127 | a granted Reputation's `score` cannot hold the range ArMDE:6554 gives | — | **closed — D11: add `max_score: Option<u8>`.** Absent = exact, present = the score must lie in `[score, max_score]`. **31 entries grant a Reputation; exactly one states a range** (`flaw.outsider_*`, ArMDE:6554), so `skip_serializing_if` leaves 30 entries' JSON unchanged. Both magnitudes get `score: 1, max_score: 3` |
| Q-128 | should `flaw.plagued_by_supernatural_entity` carry an `entity` parameter? | — | **closed — D9: yes** |
| Q-129 | which German name is canonical for Primogeniture Lineage and Palsied Hands? | — | **closed — D7 (2026-09-21): the DE rulebook heading wins; all three shipped names are correct** |
| Q-130 | should a Rigid Magic magus be blocked from *selecting* a Ritual spell at creation? | N | **SETTLED — D27.** No. The list models spells **known**; selection stays legal and a Rigid Magic character holding a Ritual raises an **advisory warning**. Text on the entry per D20; the check must see *granted* Flaws too (D2). Original follows. — B17. `SpellDef::ritual: bool` exists, so the engine *could*; but ArMDE:6697 forbids **casting**, not knowing, and a magus may have learned a Ritual before acquiring the Flaw. Turns on whether spell selection models "spells known" or "spells usable" |
| Q-131 | which catalogue Ability is "Covenant Lore"? | R | **SETTLED from the source, 2026-09-22.** It is **`ability.area_lore` parameterised by the covenant**: ArMDE:7282 defines `(Area) Lore` as "Knowledge of one particular region, **covenant**, or even a village", and DE:7282 matches ("einen Konvent"). `(Organization) Lore` (ArMDE:7674) names the Church and a craft guild, never a covenant. The Flaw's closing sentence — "can also be applied to organizations other than covenants, such as abbeys, universities, guilds, or churches" — makes the reference **target-dependent**: `area_lore` for a covenant, `organization_lore` for the rest. That is why no `ability.covenant_lore` exists. **F-520 unblocked.** Original follows. — B17. The phrase occurs **exactly once** in the English core book (ArMDE:6793) and there is no `ability.covenant_lore`; DE's *Konventskunde* is equally absent. Candidates: `ability.area_lore`, `ability.organization_lore`. **F-520 is blocked on it**; an agent can settle it by reading the Abilities chapter |
| Q-132 | should `Prereq` gain a variant that ranges over a `categories` value? | N | **SETTLED — D21**, which is written as this question's ruling and closes it together with Q-07, Q-102, F-427, F-542 and F-355 — **one mechanism, not four**, with an Effect-side twin. What remains is design-and-build, not a decision; D23's *predicate*-valued exclusions are related machinery with a different quantifier and are to be designed alongside it. Original follows. — B17. `flaw.rector` requires "a Social Status Virtue" — **99** entries carry it, so an enumerated `Prereq::Any` violates CLAUDE.md's catalogue-size invariant, and description-only enforces nothing. **Decide with F-427 / Q-07 / Q-102**, which need the same machinery for ArMDE:2816, and with **F-542 / F-355**, which need its Effect-side twin |
| Q-133 | does `flaw.short_ranged_magic`'s Lab Total halving belong in `spell_level_cap`? | N | **SETTLED — D28.** Yes: the cap becomes **range-aware** (halved for `eye`/`voice`/`sight`/`arcane_connection`, by name not by ordering). Re-keys `spell_level_cap(s)` and the picker; lowers caps, so saves legal today report and block per D10. The `flaw.savantism` sibling is **settled — D29** (one resolution point folds age band + V/F overrides; a separate validator cannot express the favored Ability's 6 above an age cap of 5). Original follows. — B17. **D1 does not reach it**: the Flaw carries no `lab_total_mod` but a *halving*, `HalvableTotal` has no member for it, and D1's scope paragraph is explicit that it governs `spell_level_cap` "and nothing else". Note D1's *generous* reasoning does **not** carry over — applying it here would **lower** the cap. Sibling: should `flaw.savantism`'s "no Ability above 3" run through `AgeAbilityCaps` / `validate_ability_age_cap`? |
| Q-134 | the unrecorded-choice family, six named instances | — | **closed — D9** (it is the Q-86/Q-88/Q-95 family and B17 filed it as instances, not a new question). The six: `flaw.restriction`, `flaw.supernatural_nuisance`, `flaw.repellent`, `flaw.rector`, `flaw.savantism` (B17) and `flaw.vulnerable_magic` (B19). **`flaw.repellent` is materially different and must be weighed separately when the parameter shapes are chosen** — its "minor advantage" can be a **+3 Soak**, a number belonging on the sheet, where the other five are narrative labels |
| Q-135 | is `flaw.seeker` magus-only, and **does a rule in another book bind an entry that cites ArMDE?** | N | **SETTLED — D22**: a cross-book rule **binds in principle**, but the implementation stays **core-book-only**. `flaw.seeker` gains `Prereq::IsMagus` on **core-book** evidence alone ("organization of *competitive magi*", "your *House*"), so no second `SourceRef` is needed and the data-model change the "yes" answer seemed to force is **not** triggered. Original follows. — B17; **`flaw.seeker` is ESCALATED and NOT marked checked.** The entry's own pointer resolves to HoH:TL:503, *"A magus from any House may be a Seeker"* — which both *removes* a House restriction and presupposes a magus, while ArMDE:6713-6716 never says "only magi". **Question 2 is the policy call and has consequences far beyond this entry** (§ 3.1's second blind spot already records F-442/F-452/F-453/F-478 of the shape "a rule the book states elsewhere about a named entry is structurally invisible"; this is the first where "elsewhere" is another **book**). If cross-book rules bind, a **second citation** must be recorded — which `source`, a single `SourceRef`, cannot hold |
| Q-136 | does a **surfaced-only** effect kind satisfy `in_play_effect`, or is `uncomputed_rule` the honest class? | N | **SETTLED — D20**, which is written as this question's ruling (*"Question (Q-136)"*). **All 19 become `uncomputed_rule` and all 19 owe their rule in `description` in both locales**; five additionally owe a number they swallow today, enumerated in `tmp/q136-number-check.md`. B17's F-515 reading is **superseded**; B18's escalation was right for a reason it did not name. Original follows. — B18; **`flaw.susceptibility_to_divine_power` is ESCALATED and NOT marked checked.** It ships `in_play_effect` with `doubled_aura_penalty`, one of eight surfaced-only kinds pushed with `amount: 0`, and `RULES.md:4882-4888` defends it in words that describe an `uncomputed_rule` (*"surfaced, not simulated"*). **B17's F-515 set the opposite precedent one batch earlier** by leaving `flaw.short_ranged_magic` `in_play_effect`. **The ruling governs 19 catalogue entries** — re-counted by B18's verification pass from `derived.rs::in_play_mods`' own `match`, correcting an earlier "at least twelve", which was the count of *effect kinds*, not entries |
| Q-137 | `incompatible_with` holds ids; ArMDE:6925 states a **predicate** | N | **SETTLED — D23**: exclusions become **predicate-valued**, not an id list — the list would have worked here but freezes a *count* of entries into data (against CLAUDE.md's "catalogue size is data, never code") and does nothing for Q-138. Two obligations carried: re-derive the count at implementation (B18 found **16 ids across 17 rows**, `flaw.failed_monk` carrying two), and remember that `validate_incompatibilities` reads **bought** selections on both sides, so catching a *granted* Virtue needs `Prereq::Nor` (F-466). Original follows. — B18, see F-526. "Any other Flaw that grants a Bad Reputation" = sixteen Flaws today. **(a)** ship the ids plus a test asserting the list equals the derived set, or **(b)** add a predicate-valued exclusion. **Not equivalent under CLAUDE.md**: (a) freezes a *number* of entries into data and is tolerable only because the test regenerates it. `virtue.doctor_in_faculty` as a prerequisite is **not** blocked on this |
| Q-138 | ArMDE:6148 forbids a **class** of Flaws, and it sits on another batch's entry | R then N | **SETTLED on the rules question and on the entry's status; remedy BLOCKED; and the *reading* is now a narrow question LEFT FOR NORBERT — `q-resolutions.md` § Q-138 (2026-09-22).** **D23 settles the mechanism and D12 supplies the predicate**: ArMDE:6148's two worked examples, `flaw.deficient_technique` and `flaw.unstructured_caster`, are both squarely **trained** under D12's criterion, which is exactly the gap B18 called harder than Q-137. **The entry is NOT unrated** — B13 checked and passed `flaw.flawed_powers` on `RULES.md`'s record that both restrictions are inexpressible, so the residue is precise: **when D23 lands, that record stops being true and must be rewritten**, as D14 required of the Covenant Upbringing record. **New design constraint neither D21 nor D23 currently carries:** the predicate must be able to name a **magnitude** and a **minimum count**, not only a category — ArMDE:6148's *"at least one **Major** Supernatural Virtue"*, the same sentence's *"a **Major** Hermetic Flaw"* (Q-104), and ArMDE:6082 (Q-103) are three sites wanting it, so fold it into D21 + D23's shared design rather than building a fourth mechanism. **Remedy BLOCKED** on D12's classification pass over the 122 `hermetic` entries, then D23. **For Norbert:** is the clause a constraint on the Major Hermetic Flaw that Flawed Powers **imports** (the verification pass's reading — the two named examples are exactly the Hermetic Flaws that cannot be re-*targeted* at a Supernatural Virtue, and the incompatibility reading would forbid a magus with Flawed Powers from holding Deficient Technique as an ordinary, unrelated Flaw, which :6148's own *"rather than to her Hermetic magic **(if any)**"* contemplates), or a plain incompatibility (**D23 assumed this**, and the grammar *"taken **with** this Flaw"* favours it)? **It changes what D23 builds, not only where it is pointed.** No data change now and no new finding against the entry; Q-104 stays separately open. Original follows. — B18. *"Any Flaw that is only appropriate to Hermetic Magic (for example, Deficient Technique or Unstructured Caster) cannot be taken with this Flaw."* Inside **B13**'s span and carried by no finding in B13, B17 or this index. Harder than Q-137: `categories: ["hermetic"]` includes Flaws that are not *only* Hermetic, so the data does not cleanly enumerate it. **Recorded so B13's entry is not left silently unrated** |
| Q-139 | does "generally restricted" warrant a hard prerequisite? | — | **closed — D16.** ArMDE:6957's *"is **generally** restricted to magi of House Verditius"* is hedged, and D16's table maps hedged → **warning, never an error**, with the standing instruction to *"apply this wherever the book hedges, rather than escalating each instance"*. **F-533's magus half was never hedged** and needs no ruling. Q-13, which B18 called adjacent, is separately closed by D2 |
| Q-140 | Weak Parens / Skilled Parens have no `IsMagus` prereq, and a **Gifted companion** can reach them | N | **SETTLED — D24.** Both gain `Prereq::IsMagus` (they are *trained* under D12, since :7074's 180/90 are apprenticeship figures), and the companion profile's Gift-conditional `hermetic` permission **stays**: The Gift is necessary, not sufficient (ArMDE:2880 states both halves; ArMDE:2840 forbids dropping the first). **Blocked on D12**, whose pass covers the other 120 `hermetic` entries. Original escalation follows. — B19; **both entries NOT marked checked.** `categories: ["hermetic"]` is not a magus gate — the companion profile permits it conditionally on `virtue.the_gift`. What follows is not an error but a **silent budget change**: `general_pool_and_bonus` builds the base from apprenticeship + post-gauntlet **only when `is_magus`**, otherwise from `later_life_xp`, and the −60 is folded in unconditionally. Three readings are defensible; reading 2 (add `is_magus` to both) has the strongest textual support, since :7074's 180/90 are *apprenticeship* figures. **Whatever the answer, it applies identically to `virtue.skilled_parens`**, which is out of span and appears in no batch record |
| Q-141 | the book indexes Weak Personality under *Story, Minor*; its own descriptor says *Personality* | N | **SETTLED — D25.** `["personality"]` stands; the disagreement is recorded as a **known source erratum** in `crates/arm-rules/RULES.md` (both citations, side taken) — **no new register**. Two obligations: sweep the other 630 entries against the book's index lists once and state the result, and treat that sweep as a human-read comparison, never a guard (link hygiene is unreliable — 134 dead link targets). Original escalation follows. — B19. Descriptor (ArMDE:7077) and DE :7077 agree on *Personality* and the data follows them, so on the narrow question the data is right and **the book is internally inconsistent**. The consequence is user-facing and CLAUDE.md rates it up: a reader working from the book's Story list filters the app by Story and the entry is not there. Options: leave it; ship `["personality","story"]` (asserts a category the descriptor denies); or **record it as a known source erratum**, which B19 recommends — but **the audit has no errata register today**, and creating one is part of the decision. **The only descriptor/index disagreement in B19's 25** |
| Q-142 | does the `scheitest` precedent cover a string containing **both** `guarter` and `quarter`? | N | **SETTLED — D26, and FIXED.** No: it is a scanno. Corrected in `rules/source/en/` at ArMDE:7054 **and** ArMDE:3502, and in `rules/i18n/en/virtues_flaws.json`. DE unaffected. Todo filed in `arm-de-translation`. Original escalation follows. — B19; **`flaw.waster_of_vis` NOT marked checked on text.** The EN `summary`/`description` reproduce ArMDE:7054's `guarter` faithfully — and the same `q`→`g` OCR error recurs at ArMDE:3502 (`virtue.berserk`), confirming it is systematic in the source, not a transcription slip. German is unaffected (DE :7054 reads *Viertel*). **What is new is that one shipped string spells the word both ways six clauses apart**, giving the reader no way to know which is meant — a comprehension defect a faithful-copy rule was probably not written to protect. Either the precedent holds and the typo stays, or self-contradicting strings are carved out — in which case **both shipped strings and `rules/source/en/…:7054` and `:3502` are corrected together**, since the JSON is generated from the Markdown |

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

**Amended 2026-09-22.** `q-resolutions.md` settled **31** questions, none of
which was on this list — it carries what needs Norbert, and those 31 were the
ones evidence could settle. Two consequences here: **item 4 has shrunk**, because
Q-138's *rules* question is now answered and only a narrow reading is left; and
**item 10 is new**, holding the three narrow follow-on calls those resolutions
threw up. Everything else below is untouched.

1. ~~**Q-136** — does a **surfaced-only** effect kind satisfy `in_play_effect`?~~
   **SETTLED — D20**, which is written as this question's ruling and opens
   *"Question (Q-136)"*. This list was stale: it still called Q-136 "the largest
   single open question" after D20 had answered it. **What is left is Phase 2
   work, not a decision** — 19 reclassifications to `uncomputed_rule`, 19
   descriptions in both locales, and five swallowed numbers, all enumerated in
   `tmp/q136-number-check.md`.
2. ~~**Q-132 (+ Q-07 / Q-102 / F-427, and F-542 / F-355 on the Effect side)**~~
   **Settled — D21**, which decided exactly the "decide once" this item asked
   for: one category mechanism with an Effect-side twin, covering all four axes.
   Build work, not a decision — and to be designed **with D23's predicates**.
3. ~~**Q-135** — **does a rule in another book bind an entry that cites ArMDE?**~~
   **Settled — D22**: it binds in principle, the implementation stays
   core-book-only, and `flaw.seeker` takes `Prereq::IsMagus` on core-book
   evidence — so the feared `SourceRef` data-model change is **not** triggered.
4. ~~**Q-137** — `incompatible_with` holds ids where the book states a
   predicate.~~ **Settled — D23**: exclusions become predicate-valued.
   ~~Q-138 is the harder of the two and sits unrated on B13's entry.~~ **Q-138 is
   settled** (`q-resolutions.md` § Q-138): D23 supplies the mechanism, D12's
   *trained* flag supplies the predicate, and the entry was **never unrated** —
   B13 checked and passed it. What is left of Q-138 is the narrow *reading* in
   item 10 below, plus one design constraint that belongs to **Q-132**'s decision
   as much as to this one: the predicate must be able to name a **magnitude** and
   a **minimum count**, not only a category.
5. ~~**Q-140 / Q-141 / Q-142** — B19's three escalations.~~ **All settled:
   D24, D25, D26.** Q-142 is also already *fixed*; the other two are Phase 2 work.
6. ~~**Q-130 / Q-133** — how far an existing mechanism reaches.~~ **Settled:
   D27** (the list models spells *known* — warn, never block) and **D28** (the
   cap becomes range-aware, which **lowers** it), plus **D29** for the
   `flaw.savantism` sibling named in Q-133's row.
7. ~~**Q-96 (+ Q-124, Q-97)** — is `source.anchor` mandatory or dropped?~~
   **Settled: D30.** Mandatory catalogue-wide (**563 refs still to backfill**),
   and a range ends on the last non-blank body line.
8. ~~**Q-10 / Q-40 / Q-49 / Q-65 / Q-69 / Q-71 / Q-114 / Q-121**~~ — **all
   settled by D31**, which removed the need for the edition check: on a *name*
   the table wins, on a *rule fact* the rulebook does. Seven previously-applied
   "corrections" are **reverted**. Q-65 alone is now table-vs-table and falls to
   D7 rule 3. `tmp/table-sync-check.md` carries a withdrawal banner — its
   findings stand, its verdicts do not.
9. ~~**Q-131** — the identity of "Covenant Lore".~~ **Settled 2026-09-22** from
   ArMDE:7282 (`(Area) Lore` covers "one particular region, **covenant**, or even
   a village") against :7674. Target-dependent: `area_lore` for a covenant,
   `organization_lore` for abbeys, universities, guilds and churches.
   **F-520 unblocked.**
10. **Three narrow follow-on calls from `q-resolutions.md` (NEW, 2026-09-22).**
    Each is one sentence, each sits *behind* an answered question, and none is a
    reading the source can settle — they are cost or policy calls. Listed last
    because they are narrow, not because they are optional.
    - ~~**Q-103's remedy**~~ — **SETTLED, D34 (2026-09-22): whitelist the named
      ids.** Domain becomes the `supernatural` category **plus
      `virtue.diedne_magic` and `virtue.the_gift`**; `virtue.faerie_blood` needs
      no entry, being `supernatural` already. A one-id whitelist field on
      `ParameterDef`, **designed with D14**. Not routed through D23 — its
      predicate would need a "defines the character's background" property the
      catalogue does not carry. The cost is accepted explicitly: the book's
      *"like"* is open and this closes it, but a missing id is **visible** where
      55 wrongly-admitted ones are not. **Re-derive the 56 at implementation and
      assert behaviour, never the count.**
    - ~~**Q-138's reading**~~ — **SETTLED, D33 (2026-09-22): it constrains the
      import.** `flaw.flawed_powers` gains a **parameter** naming the imported
      Major Hermetic Flaw, with a constraint excluding the only-Hermetic ones —
      D14's shape, and a **D9** instance too, since the entry records no choice
      today. **D23 assumed an incompatibility and was wrong on this entry**; its
      mechanism still stands for Q-137. Decisive word: *"(if any)"*, which
      concedes the character may be a magus and would otherwise bar him from
      Deficient Technique for no stated reason. **No `incompatible_with`, no
      `Prereq::Nor` here.**
    - ~~**Q-67's interim**~~ — **SETTLED, D35 (2026-09-22), and the interim was
      refused**: the XP must reach the player, so `ParamType` gains a **number**
      variant and `restricted_ability_xp` learns to scale by it. **Cap: 2
      finished years (60 XP)**, derived from the catalogue — the third finished
      year *is* the Baccalaureus Artium (ArMDE:3472, 3 years, 90 XP), a different
      Virtue. An age formula was considered and rejected: it breaks on the
      perpetual student. Design the number type **with D9 and D14**, which extend
      the same struct.

---

## 7. Counts

### 7.1 Method — read this before quoting a number

Three earlier agents in this audit shipped wrong census figures, one exactly
backwards; **B18's verification pass then found eight wrong figures where its own
batch had predicted two**. So the method is stated per figure rather than assumed,
and **every figure below was verified with a different command from the one that
produced it.**

Every number is derived **mechanically from § 1 of this file**, not recounted by
eye, using the section's own line range and its column positions:

```
sed -n '155,838p' corrections.md | grep -a "^| F-" | cut -d'|' -f5 | ...
```

That restriction matters: **three** later sections contain tables whose rows also
begin `| F-` — § 2.3, § 3.15 and § 3.16, **25** rows between them — so counting
the whole file returns **576** instead of 551, and 576 − 551 = 25 checks out.
(**Recounted 2026-09-22**: the earlier figure of "eight sections" was wrong, and
nothing depended on it. Several further sections — § 3.5, § 4.3, § 4.4 — open
their rows `| **F-`, which the plain pattern misses, so widen the filter before
concluding a section has no finding rows.)

**The line range moves whenever this file is edited** — it was `118,767` when
§ 7 was first written and is `155,838` after the 2026-09-22 update. Re-derive it
from `grep -n "^## 1\.\|^## 2\."` before quoting any figure; a stale range is a
silently wrong count, which is the exact failure this section exists to prevent.

**Every figure in § 7 was recounted from scratch on 2026-09-22**, not adjusted
arithmetically from the previous numbers. Two of them moved for reasons that have
nothing to do with the seven new findings, and both would have been missed by
arithmetic: **F-520 left `blocked` for `live`** when Q-131 was settled, and
**`fixed` is a new status** (F-546).

**Column positions**, so a later reader can re-derive any figure: `f2` = finding
number, `f3` = entry, `f4` = ArMDE, `f5` = kind, `f6` = severity, `f7` = status,
`f8` = summary. **Filter on `f7`, never on the whole line** — a summary containing
the word "unblocked" or "blocked" silently corrupts a status filter, which it did
on the first attempt here.

**Four cross-checks agree on the headline figure.**

1. `grep -c "^### F-"` over all nineteen batch files returns **544** finding
   headings. Per batch: 33, 31, 24, 32, 39, 48, 49, 51, 40, 35, 24, 20, 17, 15,
   21, 21, **21, 16, 7** — which sums to 544. **The seven `q-resolutions.md`
   findings have no batch heading**, because they belong to no batch, so
   544 + 7 = **551**.
2. `grep -o "^### F-[0-9]*" | sort -u | wc -l` over the same files also returns
   **544**, so no heading re-uses a number.
3. § 1 of this file has exactly **551** rows, and **551 distinct** finding numbers
   (`cut`ing the number column and `sort -u`-ing it gives the same figure, so
   there are no accidental duplicate rows).
4. Finding numbers run F-001 … F-552 with **one gap, F-44**, which
   `batch-02.md:4` and `:534` both record as never issued. 552 − 1 = **551**.

**Every kind and severity tally sums to 551**, and each is shown summing below.
Where a figure is approximate, it says so.

### 7.2 Findings by status

Derived from `cut -d'|' -f7` over § 1's rows; cross-checked by subtracting the
non-`live` rows from 551.

| Status | Count |
|---|---|
| `live` | **533** (includes F-482, which is live but partly duplicates F-11 and F-100; **F-439, unblocked by D17**; **F-520, unblocked by Q-131's resolution**; and **F-551, `dup→F-404`** — the two are one job in one direction) |
| `withdrawn` | **13** — 3 by their own batch (F-299, F-373, F-481), 9 as duplicates found by **this index** (§ 4.3), 1 settled by D7 (F-496's name question). **B17-B19 and the `q-resolutions.md` block add none** |
| `blocked` | **3** — F-85 (Q-10), F-270 and F-283 (a catalogue decision). **F-439 left this list** when D17 answered Q-108, and **F-520 left it** when Q-131 was settled from ArMDE:7282 |
| closed by this file | **1** — F-499 |
| `fixed` | **1** — **F-546**, settled by D32 and already committed (`212d726`, `c3714b2`) |
| **Total** | **551** |

**The blocked count fell from 4 to 3** — F-520 was unblocked, and no new finding
is blocked: **six of the seven new rows are `live`** and the seventh is `fixed`.
**Seven** of § 6's newly-settled questions still have a *remedy* blocked on
machinery (Q-29/Q-38, Q-42, Q-67, Q-110, Q-115, Q-117, Q-138), but those blocks
sit on **questions and rulings**, not on a finding's status — they are in each
question's own row in § 6 and summarised as one row in § 2.3. **Seven, not the
six `q-resolutions.md`'s own table lists:** its § Q-117 states a blocker in the
body that the table omits.

**D2 blocks no individual finding**, but it blocks part of § 3.6 and is the
audit's one genuinely pending ruling.

### 7.3 Findings by kind of defect — **primary kind**

Sums to 551. Derived by `cut -d'|' -f5 | cut -d'+' -f1` over § 1's rows, so a
compound `class+desc` counts here only under `class`. **Cross-checked** by
summing the column (551) and by the per-block delta: B17-B19 contributed exactly
44 (`class` +23, `desc` +6, `prereq` +4, `engine` +3, `text` +2, `param` +2,
`incompat` +2, `auth` +1, `src` +1 — itself summing to 44), and the
`q-resolutions.md` block contributes exactly 7 (`number` +2, `text` +2,
`engine` +1, `scope` +1, `audit` +1).

| Kind | Count | vs B01-B16 | vs the 544 |
|---|---|---|---|
| `class` — misclassification | **166** | +23 | ±0 |
| `desc` — missing description (D5) | **116** | +6 | ±0 |
| `auth` — missing authorization | **39** | +1 | ±0 |
| `text` — shipped text / localization | **38** | +2 | **+2** |
| `prereq` — missing prerequisite | **34** | +4 | ±0 |
| `param` — missing/wrong parameter | **29** | +2 | ±0 |
| `incompat` — missing incompatibility | **27** | +2 | ±0 |
| `scope` — wrong scope | **25** | ±0 | **+1** |
| `effect` — missing effect | **19** | ±0 | ±0 |
| `table` — translation-table defect | **13** | ±0 | ±0 |
| `engine` — engine gap | **11** | +3 | **+1** |
| `rep` — Reputation defect | **9** | ±0 | ±0 |
| `number` — wrong number/arithmetic | **9** | ±0 | **+2** |
| `src` — `source.lines` / anchor | **6** | +1 | ±0 |
| `prov` — provenance | **5** | ±0 | ±0 |
| `audit` — audit-record defect | **3** | ±0 | **+1** |
| `mag` — magnitude/category/kind | **2** | ±0 | ±0 |
| **Total** | **551** | +44 | **+7** |

**`number` was the audit's second-smallest kind and has just grown by 29 %** on
seven findings. That is not a coincidence of sample size: **both** new `number`
rows are *wrong arithmetic reaching a printed sheet* (`charged_cost`'s rounding,
the Mythic Companion ceilings), and both were found by **answering a question**
rather than by reading a span — which is what a `number` defect needs, since it
is invisible in the entry's own JSON.

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
551. It is the better guide to slice size: § 3.2 is a 67-finding slice, not a
39-finding one. Derived by `cut -d'|' -f5 | tr '+' '\n'` over § 1's rows;
cross-checked against § 7.3 — every row here must be ≥ its primary count, and
the column total (**703**) minus 551 gives **152 secondary kinds**, which is
exactly the number of `+` characters in the kind column (146 rows carry a `+`;
six of them carry two). **One row needs hand-correction and always has:**
F-403's kind is `engine+audit (+table)`, so a naive `tr '+' '\n'` yields the
fragments `audit (` and `table)` — they are counted below under `audit` and
`table`.

| Kind | Count | vs primary |
|---|---|---|
| `class` | 169 | +3 |
| `desc` | **154** | +38 |
| `auth` | **67** | +28 |
| `text` | 40 | +2 |
| `prereq` | 40 | +6 |
| `param` | 39 | +10 |
| `incompat` | 39 | +12 |
| `effect` | 28 | +9 |
| `scope` | 27 | +2 |
| `engine` | **25** | +14 |
| `number` | 17 | +8 |
| `prov` | 15 | +10 |
| `table` | 14 | +1 |
| `rep` | 11 | +2 |
| `audit` | 8 | +5 |
| `src` | 7 | +1 |
| `mag` | 3 | +1 |

**The headline, and it has sharpened.** Three kinds are **speaking for 58 % of
the audit by primary kind** — `class` (166), `desc` (116) and `auth` (39) make
**321 of 551**. Under D5 the first two are one job per entry, and § 2.1 says
neither can start until `MECHANICAL_PHRASES` grows. **The screen fix is the
critical path for well over half the correction list**, and § 2.1b adds a
requirement to it that nobody has enumerated yet.

**`engine` more than doubled between the two columns (11 → 25)**, which is the
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

| Severity | All 551 | Live only (533) | B17-B19's 44 | the 7 new |
|---|---|---|---|---|
| `H` | **205** | **197** | **11** | **3** |
| `M` | **290** | **284** | **29** | **2** |
| `L` | **32** | **32** | **4** | **2** |
| `n/s` — the batch stated none | **21** | **20** | **0** | **0** |
| — (withdrawn, no severity) | **3** | 0 | 0 | 0 |
| **Total** | **551** | **533** | **44** | **7** |

Both totals verified two ways: the column sums, and `551 − 13 withdrawn − 3
blocked − 1 closed − 1 fixed = 533`. The `n/s` figure is **20 + 1** — F-403
carries the compound `n/s (table row: L)` and is counted once, in `n/s`.

**H went from 190 to 202 (+12, not +11)** when B17-B19 landed, because **F-439**
moved out of the `H/nil` bucket when D17 resolved Q-108 toward "replaces". That
was the one severity in this file changed by a ruling rather than by a new
finding.

**H is now 205 — but one of the three new `H` rows is not Phase 2 work.**
**F-546** is `fixed`, so it counts in the `All` column and not in `Live only`;
the other two, **F-547** and **F-552**, are both *wrong numbers on a printed
sheet* and both are live. **Plan a slice from the live column, never the total.**

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
| Findings | **551** (F-001…F-552, no F-44) | four agreeing cross-checks — § 7.1 |
| — from the nineteen batches | **544** | `grep -c "^### F-"` over `batch-*.md` |
| — filed by this index from `q-resolutions.md` | **7** (F-546…F-552) | the block at the end of § 1; they belong to no batch |
| Open questions | **142** (Q-01…Q-142) | `grep -c "^| Q-"` over § 6 = 142; `sort -u` on the id column also = 142 |
| — settled / closed / withdrawn | **82** | the two encodings no longer agree and **must not be conflated**: **34** rows carry `—` in the settler column (the pre-D24 tranches), and **48** keep their original code and carry a settled marker in the status column. **A grep for `SETTLED` returns 47, not 48** — Q-65 reads `**PARTLY settled — D31**`, lowercase. Count the complement instead (see the next row), which is the only filter that does not need a special case |
| — open | **60** | 142 − 82, and independently: every remaining row's status field begins with the literal `open`, which is the **only** reliable filter now |
| — needing **Norbert** (`N`) | **57** of the 60 | every open row is plain `N` bar three; the **compound** codes (`R/N`, `C/N`, `X/N`, `R then N`) are now **all settled**. Three answered questions nevertheless carry a *narrow follow-on* for him — Q-103, Q-138, Q-67 — listed as § 6.2 item 10 and **not** counted here, because the question itself is closed |
| — settleable by an agent (`R`, `C`, `R/C`) | **3** | Q-01, Q-15 (settled in practice — § 5), Q-72, all `R`. **`C` and `R/C` are exhausted** — `q-resolutions.md` cleared every one |
| — needing `arm-de-translation` (`X`, `X/N`) | **0** open, 6 settled | all six fell to **D31** |
| Rulings in `decisions.md` | **32** (D1…D32), **all taken** | `grep -c "^## D"` = 32, `sort -u` also 32 |
| — taken after this file was first written | **25** — D8…D32, since tranche 1 of § 6 is *"before D8"*. D8-D18 close **25** questions; D19-D23 close none; **D24-D31** close **16**; D32 closes none but **fixes** F-546 | § 6's four-tranche list. The 17th row in that tranche is **Q-131**, settled from ArMDE:7282 rather than by a ruling |
| Duplicate relationships found | **10** (9 by this index, 1 already withdrawn by B16), **or 11 counting F-551** | **F-502…F-545 add zero**, by the seven-pass method in § 4.2. The 2026-09-22 block adds **one `dup→` marker**, F-551 → F-404 — a *partial* overlap (the two rows read one asymmetry in opposite directions), so § 4.1 counts it separately rather than folding it into the ten |
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
  more than doubles between primary and secondary, and four of the largest pieces
  of work (D9 part 3, D10, D13/D17, D14) carry **no finding number at all**.
- **No count of the entries the rulings reach.** D8's **48**, D12's **122**,
  D10's **~42** and Q-136's **19** are listed in § 7.6 as *figures the rulings
  state*, and every one of them says to re-derive the set from the data rather
  than from a list. **None of those entries has a finding row in § 1**, so § 1's
  551 is not a measure of how much data Phase 2 touches.
- **No re-rating.** Where a batch's severity looks wrong to me I left it and said
  so in § 0.2 rather than adjusting it silently. The one exception is **F-439**,
  whose severity was written as a conditional (`H/nil`) pending Q-108 and is now
  `H` because **D17 answered the condition** — that is a resolution, not a
  re-rating. The 2026-09-22 update kept to that rule: **F-351's kind and severity
  are untouched** even though Q-81 reverses its remedy, because reversing a
  remedy is not re-rating a defect.
- **No verdict on the five escalated entries.** They are not marked checked, and
  this file records the question rather than guessing the answer.

---

## 8. Measurements owed — every sweep and re-derivation a ruling requires

**Why this section exists.** By D48 the decisions carried **33** separate
instructions to sweep, re-derive or measure something, scattered across 48
rulings. Each is load-bearing — a figure quoted in a decision is a *snapshot of
one span*, not a catalogue census — and nothing collected them. That is the exact
shape of the gap that made B16 duplicate B09's work, so it is collected here.

**Read every row as "measure it, then act" — never "trust the number".** The
audit has already been bitten by unit errors (a `halbier` count wrong by 5, eight
wrong census figures in one batch), and by figures that were true of a span and
quoted as true of the catalogue.

| # | Ruling | What must be measured | Why the stated figure is not enough |
|---|---|---|---|
| 1 | **D12** | classify all **122** `hermetic` entries as intrinsic or trained | the split drives every `IsMagus` gate, and D24 adds two entries outside every batch span |
| 2 | **D10** | the **~42** entries riding the unlimited multiplicity default | the figure is a data census that moves with the catalogue |
| 3 | **D20** | the **19** surfaced-only entries | enumerated in `tmp/q136-number-check.md`, which is scratch, not data |
| 4 | **D9** | which entries state a choice nobody records | the slice must re-derive the set |
| 5 | **D25** | the other **630** entries, descriptor against the book's index lists | B19 checked 25 and found one disagreement |
| 6 | **D30** | **563** source refs needing anchors; every `source.lines` range needing the non-blank form | B16 measured 35 ranges; nobody has measured the other 620 |
| 7 | **D34** | the **56** Hermetic Virtues `flaw.false_power` admits | a category census; assert behaviour, never the count |
| 8 | **D42** | which Supernatural entries the book *fixes* a realm for | ArMDE:2961 says each description notes it, so it must be read per entry |
| 9 | **D43** | the **28** `restricted_ability_xp` carriers, each against its passage | the narrowing and the authorizations must land together or access is lost |
| 10 | **D44** | the **81** `incompatible_with` declarations, for which cite a passage | each unsourced pair must be shown entailed or removed |
| 11 | **D46** | the **five** effect-less `creation_effect` entries | `README.md` calls them "a lead, not a conclusion" |
| 12 | **D48** | the catalogue, for pools that name an Ability *instance* | B06 measured its own span and found three — but **F-74** (`virtue.forge_companion`, B02, **H**) is a **fourth, outside that span**, so three was never the count. Start the sweep from those four |
| 12a | **D49** | the catalogue, for entries that change **free seasons per year** | B05 found three in a 35-entry window and treated them as unrelated omissions |
| 12d | **D57** | the **~36 mid-phrase German templates**, for tokens sitting where German demands agreement | `RULES.md` names the count; only the **four** `realm` items have been fixed, and `{land}` was found by reading one entry |
| 12c | **D56** | **every `is_magus` site**, re-read for which of the two meanings it wanted | ~33 in `validation/mod.rs`, ~19 in `derived.rs`, plus `types.rs`, `validation/magus.rs`, `prereq.rs`, `ruleset.rs`, `effective/xp.rs`, `integrity.rs`, `life_stage.rs`, `authorization.rs`. **This is the cost of D56 and it will be underestimated** — the compiler cannot tell you which half a site meant |
| 12b | **D50** | **all 335 `narrative` entries**, re-tested against "would a player get it wrong without it?" | the batches classified against *"is it mechanical?"*, which D50 replaces. **The phrase screen cannot find these** — *"the storyguide alerts you"* carries no signed number, botch term or set phrase — so this is a manual read unless the screen grows first (§ 2) |
| 13 | **D23 / Q-137** | the Bad-Reputation id list | B18 got **16 ids across 17 rows**, `flaw.failed_monk` carrying two |
| 14 | **D18 / D31** | the **21** table disagreements, re-derived under D31 | every verdict in `tmp/table-sync-check.md` is withdrawn |

### 8a. Supplement-conditional entries — revisit when supplement selection lands

A planned feature will let the troupe choose which supplements are supported in
character generation. Entries whose **data depends on that choice** are listed
here, because they are **invisible from the feature that will need them**. Flip
them by reading this list, never from memory.

| Entry | Now | When the supplement is selected | Source |
|---|---|---|---|
| `flaw.independent_craftsman` | `categories: ["personality"]` (D54) | back to `["general"]` when **City and Guild** is in play | ArMDE:6304 |

**One instance, measured** — *"treat this as a"* over the core book returns two
hits and the second is a creature rule (ArMDE:18519). Add a row here rather than
inventing a mechanism if a second ever appears.

**Two of these are already discharged and are kept for the record**: D38's "find
the other *only X can take this* sentences" (exactly two, the second filed as
**F-553**), and D47's sibling sweep of the guild entries (no second caller, no
new defect).
