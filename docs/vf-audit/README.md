# Full Virtue/Flaw audit — 2026-09-19

Every one of the **655** Virtues and Flaws in `rules/core/virtues_flaws.json` is
checked, one entry at a time, against the passage in the rulebook that defines
it. Two questions per entry:

1. **Classification** — is `classification` the right one of the four?
2. **Implementation** — does the mechanical data (`effects`, `magnitude`,
   `kind`, `entity_kinds`, `prereqs`, `incompatible_with`, `parameters`) say
   what the passage says, and does the engine actually *do* what the data says?

Nothing is corrected during the checking pass. Every failure lands on the
correction list, and the corrections are made afterwards, test-first.

## Why this audit exists

Three prior slices each found the same shape of defect, and each found it by
accident while looking for something else:

- `e8707d6` — two Form-specific Flaws applied to all ten Forms.
- `7f5605a` — an aging floor that exists in no rulebook, invented to satisfy a
  guard, plus the warning that only existed to justify it.
- `b5fc9da` / `ecb5150` — 19 entries classified `narrative` while stating real
  mechanics; 13 of them inside a block that had been declared swept and clean
  four days earlier.

Each was found because somebody read a passage nobody had asked them to read.
The lesson recorded in `ecb5150` is that a token screen is a **work-list
generator, not the analysis**: a rule stated without a token the screen knows is
invisible to it, so working only a screen's output inherits its blind spot.
This audit therefore reads **every** entry, flagged or not.

## The four-way partition

`classification` is a claim about **the rulebook**, never about the engine's
current capability. "The engine does not compute this yet" is never a reason to
call something `narrative`.

| Value | Claim |
|---|---|
| `narrative` | The book states **nothing mechanical** for this entry. Pure story/colour. |
| `uncomputed_rule` | The book states a mechanical rule, and the engine does **not** compute it. The rule must be written out in `description` in every locale. |
| `creation_effect` | The book states a rule the engine computes at character creation. |
| `in_play_effect` | The book states a rule the engine computes during play (combat, casting, aging, lab). |

All sixteen transitions are live and none is privileged. `narrative` on an
entry that states a cap, a formula or an absolute is the error the *last* slice
found, which is exactly why it must not become the thing this audit aims at —
a `creation_effect` that is really pure colour, a rule computed in the wrong
phase, and an `uncomputed_rule` the engine turns out to compute are equally
real and have never been looked for.

## Batches

Entries are ordered by `source.lines[0]` then `id` and sliced into 19 batches.
The canonical ordering command — every agent uses exactly this, so batch
membership is reproducible:

```
jq -r 'sort_by(.source.lines[0], .id) | .[IDX0:IDX1] | .[] | .id' rules/core/virtues_flaws.json
```

| Batch | Index range | n | Source span | First … last |
|---|---|---|---|---|
| B01 | 0–34 | 35 | ArMDE:3362-3562 | `virtue.academic_concentration_subject` … `virtue.cautious_with_ability` |
| B02 | 35–69 | 35 | ArMDE:3563-3766 | `virtue.clan_ilfetu` … `virtue.enticer_of_multitudes` |
| B03 | 70–104 | 35 | ArMDE:3767-3966 | `virtue.entrancement` … `virtue.ghostly_warder` |
| B04 | 105–139 | 35 | ArMDE:3967-4154 | `virtue.the_gift` … `virtue.inventive_genius` |
| B05 | 140–174 | 35 | ArMDE:4155-4350 | `virtue.jack_of_all_trades` … `virtue.magic_items` |
| B06 | 175–209 | 35 | ArMDE:4351-4597 | `virtue.magic_sensitivity` … `virtue.nephilim` |
| B07 | 210–244 | 35 | ArMDE:4598-4883 | `virtue.notary` … `virtue.rosh_beth_din` |
| B08 | 245–279 | 35 | ArMDE:4884-5096 | `virtue.schooled_in_crime` … `virtue.supernatural_beauty` |
| B09 | 280–314 | 35 | ArMDE:5097-5256 | `virtue.tainted_treasure` … `virtue.wisdom_from_ignorance` |
| B10 | 315–349 | 35 | ArMDE:5257-5780 | `virtue.wise_one` … `flaw.castratus` |
| B11 | 350–384 | 35 | ArMDE:5781-5931 | `flaw.ceremonial_spontaneous_magic` … `flaw.demonic_familiar` |
| B12 | 385–419 | 35 | ArMDE:5932-6067 | `flaw.dependent` … `flaw.failed_master` |
| B13 | 420–454 | 35 | ArMDE:6068-6235 | `flaw.failed_monk` … `flaw.harmless_magic` |
| B14 | 455–489 | 35 | ArMDE:6236-6369 | `flaw.hatred_major` … `flaw.low_tolerance` |
| B15 | 490–524 | 35 | ArMDE:6370-6537 | `flaw.lycanthrope` … `flaw.optimistic_minor` |
| B16 | 525–559 | 35 | ArMDE:6538-6662 | `flaw.outcast` … `flaw.rebellious_minor` |
| B17 | 560–594 | 35 | ArMDE:6663-6802 | `flaw.reckless_major` … `flaw.supernatural_nuisance` |
| B18 | 595–629 | 35 | ArMDE:6803-6988 | `flaw.suppressed_gift` … `flaw.visions` |
| B19 | 630–654 | 25 | ArMDE:6989-7119 | `flaw.vow_major` … `flaw.wrathful_minor` |

All 655 entries cite the core rulebook; no supplement V/F are in the catalogue.

## Files

| File | Contents |
|---|---|
| `engine-semantics.md` | The yardstick for "implementation correct": what every field of a V/F entry actually *means at runtime* — all 42 `Effect` variants, plus `classification`, `magnitude`, `kind`, `entity_kinds`, `categories`, `prereqs`, `incompatible_with`, `parameters`. For each: where the engine consumes it, the arithmetic it performs, and what it silently ignores. Written once, before any batch. |
| `batch-NN.md` | One verdict row per entry, plus the full reasoning for every entry that failed. |
| `corrections.md` | The accumulated correction list — everything the batches found, which is the input to the fixing pass. |
| `decisions.md` | Norbert's rulings on questions the audit could not settle from the source. **Binding.** Read it before starting a batch; do not re-litigate a ruling and do not guess where one is pending. |

## Starting counts (pre-audit)

| Classification | Virtues | Flaws | Total | With `effects` |
|---|---|---|---|---|
| `narrative` | 146 | 189 | 335 | 0 |
| `uncomputed_rule` | 29 | 73 | 102 | 0 |
| `creation_effect` | 96 | 29 | 125 | 120 |
| `in_play_effect` | 46 | 47 | 93 | 93 |
| **Total** | **317** | **338** | **655** | **213** |

The five `creation_effect` entries carrying no `effects` at all are already
known and are a lead, not a conclusion: `flaw.corrupted_arts`,
`flaw.savantism`, `virtue.devil_child`, `virtue.nephilim`,
`virtue.simple_student`.

## Progress

| Batch | Checked | Failures | Open questions | Status |
|---|---|---|---|---|
| Phase 0 (`engine-semantics.md`) | — | 11 systemic, in 7 areas (Part C) | 4 | **done** |
| B01–B19 | 0 / 655 | — | — | pending |

Phase 0 notes, which every batch agent must know before starting:

- **`classification` is read by no production code.** It gates nothing and
  changes no number; its only runtime effect is that serde requires it. Its real
  consumers are two test-side guards. Rate a misclassification as a lost-rule /
  provenance defect, never as a miscalculation — the mechanism by which a wrong
  `narrative` hurts is the *missing description text*, not a missing computation.
  See `engine-semantics.md` § B1.
- **Eleven systemic gaps are already known** (§ Part C). Do **not** re-report them
  per entry: an entry carrying `advancement_mod`, `ability_roll_mod`, a
  surfaced-only `special_casting_mod` kind, a realm susceptibility, or a
  non-computed `health_mod` track is *correctly authored against an engine that
  lists rather than computes it*. The gap is the engine's, and it is recorded
  once.
- **Three of Phase 0's open questions are now ruled on — see `decisions.md`
  (D1, D2, D3), which is binding.** D3 in particular sets a precedent every
  batch needs: an engine that *structurally cannot express* a rule is grounds
  for `uncomputed_rule` with the rule written out in both locales, and is never
  grounds for `narrative`.
- **Two Phase 0 questions are still open and are for a batch agent to settle
  from the book, not to guess:** whether a Major Magical Focus has different
  arithmetic from a Minor one (`ArMDE:4399-4422` — the batch reaching
  `virtue.major_magical_focus` reads it and reports), and whether a granted
  Reputation's `score` is a ceiling, a fixed value or a suggestion
  (`ArMDE:2512-2514`). Neither has been read yet.
- **D2 is pending on a rules read.** The batch reaching
  `virtue.great_characteristic` must read its passage and report what the book
  requires of a *granted* Virtue's preconditions. Until then, no batch rates
  the bought-only validators either correct or defective.

## Rules the audit runs under

- **No privileged failure mode.** All checks carry equal weight and all sixteen
  classification transitions are live. The last slice's bug — mechanics stated
  in words rather than numbers — is not the thing being hunted; it is one of
  many shapes, and aiming at it is how the next shape gets missed.
- **No inherited verdicts.** No entry is trusted because an earlier sweep
  declared its block clean. `SWEPT_BLOCKS` is a record of what a screen saw,
  not a warrant.
- **Doubt is escalated, never resolved quietly.** An entry whose reading cannot
  be settled from the source is recorded as an open question and is **not**
  marked checked. Picking a reading by judgement is what put the invented aging
  floor (`7f5605a`) and seven knowingly-wrong classifications (`ecb5150`) into
  the data.
- **Source or nothing.** Every verdict cites ArMDE line numbers. A rule
  remembered rather than read is not a rule.
