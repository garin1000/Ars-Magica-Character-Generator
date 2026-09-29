# Phase 2 + open to-dos — combined implementation plan

**Approved by Norbert, 2026-09-26** (rev. 3, after two Fable plan-review rounds:
25 findings, then 7 minor — all applied).

**Status, 2026-09-26: first round DONE** (`6f34a70`…`0ef6dbd`): M0, Phase 1
(Q1–Q12, Q3b, Q4b, E2, U0), Phase 1S (S1–S4), A0–A2, U6, E1.
**Status, 2026-09-28:** **Groups C and B done** — C0…C5c (schema 19); B0 note
(rev. 3, D60), B1–B5; B-boundary e2e 10/10 (one retry: `wizard-flow.e2e.js`
"warns about unspent experience", a flake to watch). **Group D done** (D0 rev. 5,
D1–D3, B1c; D4 dropped by D62; D-boundary e2e 10/10). **Group F done** (F0 rev. 3,
F1 schema 20, F2; e2e 10/10, portable 1/1). **Phase 2 is complete.** **2026-09-29:**
X1 done (D43: general XP funds only explicitly authorized spends); U batch
implemented. X2a, X2b done. **Re-prioritised for a working app on 2026-10-03
(Norbert):** after X2c, X2 pauses and the wrong-rules-output slices run
first — X3, X7b-e, X7b-d, X4, X5, X7c, X6, X7a, X10 (a, d) — then a full
review. X2d–h, X2t, X8, X9, X10b/c follow afterwards. Every slice runs
under the red-checkpoint protocol. N1, N4–N7 answered (D65).
**Added 2026-09-28:** B1c (after D1) removes B1's unused
`RestrictsAbilityCategoryToAbilities`. D63 moved Feral's whitelist into its
D2 replacement pool, so the effect has no carrier.

## Context

The V/F audit's checking pass is complete (D1–D58, 0 open questions); what remains
is the fixing pass, entered via `phase-2-handover.md`. `docs/open-todos.md`
carries engine, data, UI and infra items besides. This is **one** plan for both,
**excluding open-todos row 54** (moved upstream). Rulings stay authoritative in
`decisions.md` and `corrections.md` § 3.0/§ 3.15b/§ 3.17; this plan only
sequences.

---

## 0. Facts re-checked against the tree (2026-09-25)

| Audit docs say | Today |
|---|---|
| F-540 before widening `SWEPT_BLOCKS` | **fixed** (`[7106, 7109]`); only the widening is owed |
| D19 regex conversion | not started (`MECHANICAL_PHRASES: &[&str]`) |
| D10 defaults | both still `u8::MAX` |
| D56 "~60 sites" | **52 files** incl. `ui/src` (33 in `validation/mod.rs`, 21 in `derived.rs`) |
| D44 "81 declarations" | **106** `incompatible_with` today — M0 re-measures |
| D1 | **unimplemented** — `effective/spell.rs::spell_level_cap` has no `lab_mod` term |
| `SCHEMA_VERSION` | 17 |

---

## 1. Rules for the whole round

- **Implementation is delegated to agents, at most ONE concurrent agent** (Norbert,
  2026-09-26), which may itself spawn **at most one** concurrent sub-agent. This
  holds for every kind of work — implementation, reads, reviews — so reviews run
  one after another, and the cap is written into every agent prompt. The
  orchestrator reads the agent's `tmp/` logs and `git status`, then commits.
- **Every slice:** test-first with verbatim RED/GREEN; full gate logged to
  `tmp/`; both locales; `RULES.md` row per mechanic (cite RULES.md by **heading**,
  not line); `README.md` when user-visible behaviour changes.
- **Every commit is green.** A guard that would go red on shipped data lands with
  a **pending work-list** in the `NO_RULE_DESPITE_TOKEN` shape: each listed entry
  must *still trip* the guard (so a fixed entry has to be removed — the list can
  only shrink), and every unlisted entry must pass. The consuming slice empties it.
- **e2e at phase boundaries** (after Phase 1, 1S, A, B, C, D/F, each X-block,
  U-batch — B because its new `Prereq` variants are mirrored in the UI) and inside
  a slice only where it changes the save format or IPC shape (C5a, F1, F2, X9b).
  Read `Spec Files:`, not the exit code. **Portable e2e** after slices touching
  `arm-app` load/path code (U0, A1's `arm-app` sub-slice, the U-batch).
- **Schema bumps — one criterion:** the saved shape or meaning moves → bump,
  with a test-first migration on untrusted `schema_version`, never two per slice;
  byte-compatible additive `serde(default)` → no bump. So: **CV** (catalogued
  Ability parameter values, 17→18), **C5a** (multi-valued parameter, 18→19),
  **F1** (`equipped` changes meaning), **X9b** (`virtue.rard` id rename) bump; **D42** concept realm and **F2** K3 mount record do not, unless
  F0 finds otherwise; **A1** only if A0 finds the entity stores the flag.
- **Design notes** (A0, B0, C0, D0, F0) are reviewed by the plan-reviewer, then
  the architect (one after the other), before their first implementation slice.
- **Coordination:** rows 47–53, 55 come from the book-template session. Before
  starting one, and before any commit to `open-todos.md`/`RULES.md`: `ListAgents`
  → `SendMessage`; if none alive, say so in the report.
- **Agent briefs:** live command-hygiene block from `full-review/SKILL.md`
  verbatim, the 1-agent / 1-sub-agent cap inside the prompt, never poll, ignore
  the auto-mode injection. Big reads are chained by line range (≤ ~60 entries per
  agent, verdict table persisted before the next starts), never thinned.

---

## 2. Decisions Norbert owes (only these; everything else is ruled)

| # | Question | Recommendation | Gates |
|---|---|---|---|
**All five answered 2026-09-28, see `decisions.md` D65.** N1: uncomputed entries get `description ?? summary`, and computed entries **their summary**. N4: whole. N5: all four parts in this round. N6: text. N7: nothing upstream.

| N1 | Row 41: V/F text in the Markdown export | name + `description ?? summary` for `uncomputed_rule`; name otherwise | X8c |
| N4 | Row 40: index a long `description` whole? | whole | X8b |
| N5 | Row 51 (a)–(d), one call each | (a), (d) now as data; (b), (c) later (bump) | X10 |
| N6 | D4's open item `virtue.aristotelian_training` | `uncomputed_rule` + text | X7a |
| N7 | Authority for the implementing session to file into `arm-de-translation` (D31 reverts, row 39's `-3penalty`) | — | X8d |

---

## 3. Slices

Sizes: **S** ≈ ½ session · **M** ≈ 1 session · **L** ≈ 3 sessions ("session" = one
implement-review-gate cycle).

### Phase 0 — measure

**M0 (S, docs only)** → `docs/vf-audit/measurements.md`, each figure with its
command: § 8 rows 2, 3, 6, 7, 9, 10, 11, 12, 12a, 12c (52-file `is_magus` list),
12d, 13; the **entry-less `ArMDE:` citations** in the three Phase-2 working files
and `book_templates.rs` (handover § 8); S1's before-offender set.
**Reads, not counts** (rows 1, 4, 5, 8, 12b, 12e, 14) run as the first step of
their consuming slice (X3, X6, X9c, X6, X2, X7c, X8d).

### Phase 1 — independent fixes

None touches a profile, XP shape or Hermetic gate (verified: e.g.
`effective/spell.rs` has no `is_magus`), so "A before everything" does not bind.

| Slice | Content | Size |
|---|---|---|
| Q1 | Row 47/F-547 `charged_cost`; fix the pinning sibling assertion. *Coord.* | S |
| Q2 | F-524: validator refusing a specialty under `flaw.unspecialized` | S |
| Q3 | D2: the two validators read effective selections; B1 comment at both sites | S |
| Q3b | D27: Rigid Magic + ritual spell → advisory (grant-aware) | S |
| Q4 | D10 defaults → 1, the ~42 repeaters declared in-slice, the no-parameters `max_per_target` sweep; D54 `independent_craftsman` → `personality` | M |
| Q5 | D11: `score` enforced, `max_score`, F-408/F-486/F-450, UI prefill | M |
| E2 | Q-32 authoring `AdvancementSource` (before Q6: `flaw.incomprehensible` needs it) | S |
| Q6 | D55 factor on `advancement_mod` | S |
| Q7 | D29 one Ability-maximum resolution point; F-194 age-cap waiver there | S |
| Q8 | F-550 `Prereq` warning severity | M |
| U0 | `EffectiveScores` DTO → `effective_dto.rs` with its pointer set (before Q9) | M |
| Q9 | D45 `SurfacedModifier.source`, five families, via Fluent | M |
| Q11 | D1: `lab_mod` term in `spell_level_cap` (before Q10, same function) | S |
| Q10 | D28 range-aware cap; re-keys caps + picker | M |
| Q12 | D37 / F-329: Magic Resistance = max(Form total, True Faith floor), keyed on the score | S |

### Phase 1S — the screen (tests only)

| Slice | Content | Size |
|---|---|---|
| S1 | D19 regex conversion, proved inert against M0's recorded offender set | S |
| S2 | § 3.1's 14 families + capability family (from D8's 48) + DE forms checked against the DE source + F-537 + U+2014 + leftward sign | M |
| S3 | D5: guard stops being class-keyed; D46: `every_vf_is_classified` tests *what* is computed. Reds → pending work-list | M |
| S4 | `SWEPT_BLOCKS` → whole catalogue; reds → pending work-list | S |

### Phase 2 — engine groups

**A — `is_magus` split (D56).** A0 (S) design + a verdict row for **every one of
the 52 files** · A1 (L) split in file-group sub-slices (validation → derived →
effective/xp → prereq/ruleset/integrity → arm-app → ui), `CLAUDE.md` Prereq
reference updated · A2 (M) conditional `creation_phases` `{phase, when}` via
`CategoryRule`. **E1** (S) D38 character-type `Prereq` incl. F-553.

**C — parameter model.** C0 (S) one design for D14/D33/D34/D35/D48/D9p3 +
W2/`custos` exclusive choice as D14 *binding*, F-349's trap; **co-designed with
B0** for D33's domain predicate · C1 (M) D14 literal + binding;
`covenant_upbringing` Latin; `student_of_realm` binding **and its +2 Lore
`ability_bonus`** (row 50a); RULES.md's "Documented approximation" under
*Access to Academic / Arcane / Martial Abilities* rewritten · C2 (S) D34
whitelist (D33's "minus trained-only" constraint is a predicate → B3) · C3 (M)
D35 `ParamType::Number` · C4 (M) D48 instances + union · C5a (M) D9p3 type +
migration (**bump**) · C5b (M) multi-select picker · C5c (S) D15's three
Corrupted entries; F-42/F-63/F-317 player-chosen Ability group.

**CV — catalogued parameter values (inserted 2026-09-27, after C4, before
C5a).** Found by Norbert: C1/C4's literal instances (`language = latin`,
`profession = marshal`, …) matched free text, so "Latin" or "Latein" silently
failed. Where a rule must recognise a value, the Ability parameter now draws
from a catalogue of ids with localized names and a free-text fallback. Existing
saves migrate by EN/DE name (bump 17→18). Spec:
`docs/vf-audit/design-cv-catalogued-values.md`. It runs under the **red
checkpoint** protocol: tests are written and verified red before any
implementation.

**B — ranging and predicates (after C1).** B0 (S) design · B1 (M) D21 category
prohibition (F-355/F-542/F-511) + category prereq (F-502, F-427); D40's feral
prohibitions · B2 (S) D41 minimum + second-status warning; the
second-guild-status *requirement* D41 leaves open · B3 (M) D23 predicates
(F-526); D33's domain predicate; F-334 parameter-comparing prereq · B4 (M) Q-51
parameter-gated effects · B5 (M) F-489: an `Effect` naming an Ability for a roll
modifier (42 entries' shape).

**D — four XP modes (after C1, C3).** D0 (S) design incl. D49 · D1 (M) D13
earmark (budget unchanged) · D2 (M) D40 replacement (feral, redcap, lone_redcap) ·
D3 (M) D56 truncated, **16 XP / 8 levels per year, fixed**, no
`minimum_abilities`, Parma advisory · D4 (S) D49 free seasons.

**F — book-template engine findings.** F0 (S) design: K5 flag model, K3 mount
record, and per D4/D58 the conditional-modifier shape per entry (extra row where
the book's template prints one, otherwise narrowed out of the base total — **no
saved toggle**) · F1 (M) K5, Knight regains two Combat rows (**bump**) · F2 (M)
K3 + Berserk, Ways of the Land, Cyclic (D52), Special Circumstances.

### Phase 3 — data and text

| Slice | Content | After | Size |
|---|---|---|---|
| X1 | Authorization family + D43 **one slice**: row 45 survey (any category), row 48 auth half, row 50 (b)–(d), § 3.2 incl. the 27 `class+auth` entries' reclassification, § 3.2a's four Educated entries + base Latin, W2/custos. **Plus (found in C1):** `house.jerbiton`'s free Minor Virtue Open grant can grant any of the four gated items (`wise_one`, `custos`, `templar_specialist`, `student_of_realm`), so a granted plus a bought copy with different choices could union to an over-permission (F-349 across selections). Check that `max_total` is grant-aware, or restrict the grant | C1, D1 | L |
| X2 | Reclassification + descriptions **excluding § 3.2's entries**: S3/S4 work-lists emptied, D8's 48, D20's 19 + 5 numbers, D50's 335 re-test, row 38 residual, `virtue.the_gift` (D46) | S1–S4 | L |
| X2t | D60.3: `virtue.true_friend_pc`, `flaw.true_friend_major`/`_minor` as twins of the True Love entries (data only, both locales, twin-parity test); lands with X2's `true_love_pc` description | X2 | S |
| X3 | D12's 122 intrinsic/trained, gated on *trained*; D24; **acceptance: no Gifted non-magus computes against a magus budget** (D12.6) | A1 | M |
| X4 | § 3.5: Wealthy/Poor closed set (F-340), incl. the gaps X1 found on almogavar, turb_trained, branded_criminal, mendicant_friar, priest; D44 over M0's figure, predicate cases incl. Q-138 | B3, X3 | M |
| X5 | § 3.6: F-518/F-532/F-533, F-502, D38 data, D51, D41's guild requirement data; F-270/F-283 (add if the book has the heading, else drop) | A1, E1, Q8, B1 | M |
| X6 | § 8 row 4 read first; D9p1 parameters (incl. turb_trained's open dead-language choice, found in X1), Q-134, D33/D34 data, D42 default realm | C2–C5, X3, B3 | L |
| X7a | § 3.8: D4 lab rows, D47, N6; Cyclic Lab row | Q7, X6 | M |
| X7b-e | **All of row 42**, engine and data together: a D58 compute/text verdict for each item (incl. the 2026-09-19 nineteen); per-Characteristic buy cap and Personality-Trait validation where the verdict says compute; the data for those entries. An entry both here and in § 3.9/§ 3.10 belongs here | F2 | M |
| X7b-d | § 3.9 wrong numbers + § 3.10 missing effects, **excluding row-42 entries** (X7b-e's) | F2 | M |
| X7c | Row 46 residue (§ 8 12e): consumer-tracing B01–B11 | X7b-e, X7b-d | M |
| X8a | § 3.12: truncation sub-slice, D53, D36, D57's ~36 apposition templates | — | M |
| X8b | Row 40 search index (N4) | X2 | S |
| X8c | Row 41 export (N1) | X2 | S |
| X8d | D31: re-derive the 21 verdicts **into `measurements.md`**, adopt 19 names, revert 7; nothing filed upstream (D65), the fixes are recorded here for Norbert | — | M |
| X9a | D30 anchors per catalogue — **spike first** for table-derived catalogues (aging, equipment, characteristics have no `####`); range normalisation; RULES.md/check-1 convention aligned; `anchor` non-optional **last** | M0 | L |
| X9b | F-16 `virtue.rard` id rename (**bump**) | — | S |
| X9c | D25's 630-entry descriptor sweep; § 3.14 `prov` findings (incl. `virtue.perfectus`'s `source.lines` running into the next entry, found in X1); F-500 integrity check | — | L |
| X9d | Row 55: heading beside each citation in `book_templates.rs`. *Coord.* | — | S |
| X10 | Row 51 (a) Tremere's focus names certamen, (d) Grapple row and Piercing the Magical Veil, as data (D65). *Coord.* | — | S |
| X10b | Row 51 (b): banked XP beside an Art or Ability score (**bump**; D65) | X10 | M |
| X10c | Row 51 (c): a per-spell "within the Magical Focus" marker (**bump**; D65) | X10b | M |

### Phase U — product and infra (batched, e2e once per batch)

U1 P1 Fullscreen handler + activation test · U2 P2 · U3 P3+P4 (guard-adjacent;
`state.svelte.test.ts`, `App.client.test.ts`) (M) · U4 P5+P6 · U5 P7 contrast in
`app.css.test.ts` · U6 e2e driver preflight + reaping · U7 `.icon-btn` e2e. All S
except U3.

---

## 4. Critical path and size

```
M0 → Phase 1 → 1S → A0 → A1 → A2 → E1 → C0…C5c → B0…B5 → D0…D4 → F0…F2
1S → X2 → X8b, X8c        C1+D1 → X1        A1 → X3 → X6 → X7a
B1/B3/E1/Q8 → X4, X5      F2 → {X7b-e, X7b-d} → X7c      X8a, X8d, X9*, U* interleave
```

Handover's hard constraints: A first (Phase 1 exempt, § 3) · D14 (C1) before D13
(D1) · C before D33/D35/D42/D48 data · D43 narrowing + authorizations in X1 ·
screen before reclassification · wrathful range already fixed.

**Size:** 33 S + 33 M + 6 L ≈ **68 sessions**, +20 % for review loops and
boundary e2e ≈ **80**. **First round:** M0 → Phase 1 → 1S → A → E1
(≈ 19 sessions) — it clears every standalone wrong-rules-output fix and the
gate every later group reads.

## 5. Not in this round

Row 54 (upstream) · P8 re-sync (D18/D31 forbid it; X9a + X9d make it safe for
**catalogue-anchored** citations only — the entry-less ones M0 counts stay debt) ·
row 39's `-3penalty` (upstream, N7) · non-goals: supplements (D22), animals (D58),
sex model (Q-05) · `corrections.md` § 5's cleared negatives.

---

## Verification

- **Per slice:** verbatim RED then GREEN; `cargo test --workspace`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo fmt --check`, UI
  `test:unit`/`lint`/`format:check`, `cargo tauri build --no-bundle`, logged to
  `tmp/`; the orchestrator reads the logs and `git status`.
- **Per phase:** `npm run test:e2e` (portable per § 1); `Spec Files:` line
  checked; book-template conformance suite (`tests/book_templates.rs`) stays green
  and gains the Knight/Berserker/Specialist corrections as their slices land.
