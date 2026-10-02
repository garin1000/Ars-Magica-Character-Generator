# P8 design: re-syncing `rules/source/` from upstream

**DESIGN ONLY. Nothing in this document is authorization to execute.**
`docs/vf-audit/decisions.md` D18 ("DO NOT re-sync the rulebook sources — the
line numbers will break") and the Phase 2 plan's own exclusion list
(`docs/vf-audit/phase-2-plan.md` § 5: *"P8 re-sync (D18/D31 forbid it; X9a +
X9d make it safe for catalogue-anchored citations only...)"*) are standing
prohibitions, not stale notes. This document exists to make the re-sync
*safe to authorize*, not to authorize it. Phase P8-2 below is the first
state-changing step, and it needs an explicit go-ahead separate from this
design being accepted — see Q1.

## Why now, and why this isn't reckless

Since D18 (2026-09-21), the project has built exactly the machinery D18 itself
called the prerequisite: `crates/arm-rules/tests/rules_source_provenance.rs`
now anchors every catalogue that carries a `source` block, in both languages
(`FULLY_ANCHORED_CATALOGUES`, confirmed complete by
`only_fully_anchored_catalogues_carry_a_source_block`), with guards that bind
an anchor to the heading it names, to the line range beside it, to the German
line-parity invariant, and to the mechanic the cited passage actually states.
That is the durable key D18 said a re-sync needs. What is still missing, and
what this design is for, is (1) the *tooling* to walk that key backwards after
a sync (today's tools only derive an anchor *from* a line; nothing yet derives
a line *from* an anchor) and (2) a plan for the citations that carry **no**
anchor at all — the `ArMDE:NNNN` comments in Rust/Svelte source and the
line-pinned rows in `RULES.md` and `book_templates.rs`.

## Measurements (read-only pass, 2026-10-02)

Performed against `../arm-de-translation`
(`original-english/reviewed/`, `german-reviewed/`). Working files are in
`tmp/resync-measure/`.

### Line counts

| File | Ours | Upstream | Δ |
|---|---|---|---|
| EN Core Rules | 25723 | 25803 | +80 |
| EN HoH:MC | 5057 | 5076 | +19 |
| EN HoH:S | 5825 | 5836 | +11 |
| EN HoH:TL | 5011 | 5057 | +46 |
| EN HM:RE | 7511 | 7510 | -1 |
| EN RoP:F | 7960 | 7922 | -38 |
| EN RoP:M | 8820 | 8735 | -85 |
| EN RoP:D | 7696 | 7730 | +34 |
| EN RoP:I | 6671 | 6727 | +56 |
| DE Basisregeln | 25723 | 25803 | +80 |
| DE Mysterienkulte | 5057 | 5076 | +19 |
| DE Societates | 5825 | 5836 | +11 |
| DE Wahre Linien | 5011 | 5057 | +46 |
| DE Heckenzauber | 7511 | 7510 | -1 |
| DE Sphären-Magie | 8820 | 8735 | -85 |
| DE Rhein-Tribunal (no EN counterpart) | 4724 | 4732 | +8 |

The 80-line Core Rules delta `open-todos.md` row 54 recorded on 2026-09-24 is
still current — upstream has not moved again on that book since.

**The German line-parity invariant still holds upstream, exactly.** For every
book we carry in both languages, upstream's EN delta equals its DE delta,
line for line (e.g. Core/Basisregeln both +80, RoP:M/Sphären-Magie both -85).
This is the load-bearing fact for the whole plan: it means a re-sync does not
have to *re-establish* German line-parity, only *carry forward* an invariant
that already survived upstream's own edits.

**Upstream now also carries three German books we don't have yet**
(RoP:Faerie/Divine/Infernal, each line-count-matched to its English upstream
counterpart) and an English Rhine Tribunal file with no counterpart here.
CLAUDE.md's "Not yet available in German" / "no English source" notes are
therefore stale upstream, though still accurate for *this repo*. See Q2 —
onboarding these is a different shape of work and is deliberately **not**
folded into P8.

### Heading-sequence diff (the anchor-relocation risk surface)

Two distinct risk groups, not a uniform one:

**Clean group** — Core Rules, all three Houses of Hermes books, RoP:Infernal
(and their German mirrors): heading sequences are near-identical to ours.
Upstream adds a handful of headings and rewords roughly 1-2% of them, almost
all OCR-class (`Rard`→`Bard`, `Subernatural`→`Supernatural`,
`Oreatures`→`Creatures`). A heading anchor built from our current wording will
fail to resolve for these reworded cases and **only** these — which is the
designed failure mode (D18: "fails loudly... rather than silently pointing at
the wrong text"), not a defect in the approach.

A few of the ~15 reword examples found are **not** OCR noise and are
semantically real:
- `5. Combat Statistics` → `6. Combat Statistics` in HoH:MC — a section
  **renumbered**, which can shift which items a row-key anchor resolves
  under if any equipment/childhood/aging row cites that book (Phase P8-0
  must confirm none currently do — see Risks).
- `Determining the Principles` → `Determining the Principals` (HoH:TL) and
  `Expanded Form & Material Bonuses` → `Expanded Shape & Material Bonuses`
  (HoH:MC) — genuine word-choice revisions.
- DE: `Mächtiges Relikt`/`Relikt` → `Mächtige Reliquie`/`Reliquie`, and
  `Infernale Kontamination` → `Infernale Befleckung` — see P8-6, these are
  exactly D18's own worked examples and D31-shaped terminology questions.

**Structurally heavy group** — Hedge Magic (Revised), all three Realms of
Power EN books, and their German mirrors (Heckenzauber, Sphären der Macht —
Magie) plus the no-EN-counterpart Rhein-Tribunal: heading **count** jumps
1.3×-1.6× (e.g. HM:RE 515→680, RoP:M 427→680). This is upstream promoting
sidebar/"Story Seed" titles to real Markdown headings — text our copies carry
today only as TOC bullets, never marked up as a heading at all. This is a
**structural** change, not a wording change: no existing anchor breaks (the
old headings are still there, unreworded, in a strict-superset sense), but the
heading *hierarchy* deepens around them, which is relevant only if a future
row-key anchor needs to resolve inside one of these books — today's shipped
row-key catalogues (`equipment.json`, `childhoods.json`, `aging.json`) are, as
far as `FULLY_ANCHORED_CATALOGUES`'s comments indicate, sourced from Core
Rules tables only. **Unverified, not assumed** — Phase P8-0 confirms this by
reading which `source.file` every row-key entry actually cites.

### Body-text sampling (EN Core Rules)

- 85% of the Core Rules' +80 lines is **one section**,
  `### Identified Issues From Source PDF Release`, a hand-curated upstream
  errata list that grew from 40 to 108 lines since our last sync — not new
  rules content.
- One rules-content gap, unrelated to re-sync mechanics: upstream's Core
  Rules carries a spell, "Frosty Breath of the Spoken Lie", that our copy is
  missing entirely. Out of scope for P8 (new content needs its own
  from-source implementation pass per CLAUDE.md's "Rules provenance") — filed
  to Phase P8-7 as a todo, not implemented here.
- The German quote-glyph house style (`„…"`, ASCII-closed) is unchanged
  upstream (spot-checked 3 files, counts match within normal drift) — a
  re-sync will **not** trigger a corpus-wide quote-style churn.

### Our known local same-line fixes: **none are present upstream**

All four of our in-place, line-count-preserving fixes are **still the
original defect** in upstream's current "reviewed" copy — confirmed by direct
read, not inferred from the errata changelog (which also doesn't mention any
of them):

| Fix (ours) | Commit | Upstream state |
|---|---|---|
| "give guarter" → "give quarter" (`virtue.berserk`, ArMDE:3502) | `a7affd0` | still "guarter" |
| "guarter" → "quarter" (`flaw.waster_of_vis`, ArMDE:7054) | `7cd8a26` | still "guarter" |
| "In such as case" → "In such a case" (ArMDE:4235) | `0b101ab` | still "In such as case" |
| reconstructed truncated sentence (Bound Casting Tools) | `cc068e2` | still truncated |
| "-3penalty" (ArMDE:6929, `open-todos.md` row 39, already known unfixed) | — | confirmed still "-3penalty" |

**A straight copy-in regresses all four of our own fixes on day one.** This is
not a reason not to sync — upstream is the authoritative source and our
fixes are documented as throwaway precisely because of this (CLAUDE.md:
"a hand repair here is throwaway work... the edit is lost along with it") —
but it means Phase P8-2 cannot be "copy the files in" alone; it needs an
immediate, explicit re-application pass. See Q3.

## What moves, and how each is re-derived

### (a) `source.lines` in `rules/core/*.json`

Already anchor-mandatory, catalogue-wide, both languages (D30.1,
`only_fully_anchored_catalogues_carry_a_source_block`). Nothing today derives
a line *from* an anchor, only the reverse
(`regenerate_source_anchors` walks `lines[0]` → finds the heading at that line
→ records its anchor). P8-1/T1 builds the missing direction: given the
recorded `anchor` and the catalogue's heading level, locate that anchor's
heading in the **new** file and take its line as the new `lines[0]`;
re-derive `lines[1]` by reusing the existing boundary logic
(`section_boundary_after`, `OWN_BLOCKQUOTED_SIDEBARS`, `SUBDIVIDED_ITEMS`) to
find the next section boundary, then stepping back to the last non-blank line
per D30.2. An anchor whose heading was reworded or removed resolves to
nothing — T1 must emit that as a **named manual-review item**, never guess
(D18's whole point).

Row-key anchors (`heading/row-key`, `heading/table-key/row-key`) relocate the
same way for their heading segment; the row-key segment itself re-resolves
against whatever text now sits in that (possibly deepened) section, which can
fail silently different from a heading rename — see Risks.

### (b) DE sidecar `rules/i18n/de/source_anchors.json`

Relocated in lock-step with (a), via the German line-parity invariant
(confirmed still holding upstream, above) — the existing
`regenerate_source_anchors` already derives both stores together; T1 extends
it symmetrically in the reverse direction.

### (c) `ArMDE:NNNN` comments in `crates/*/src`, `crates/*/tests`, `ui/src`

**Carry no anchor at all.** These are exactly what `phase-2-plan.md` § 5 calls
"the entry-less ones [that] stay debt." P8-1/T2 is new tooling for this class:
for each cited line/range, read a few lines of context from the file **as it
was immediately before the sync** (trivially available: `git show
<pre-sync-sha>:<path>`, since Phase P8-2 commits the raw copy-in as its own
commit before anything else touches these files), then locate that exact text
in the post-sync file. An exact match relocates directly and is
**self-verifying by construction** — it only matched because the text is
identical, which is the same "compare the quoted text" idea the brief asks
for, without needing a quoted excerpt to already exist in the comment. No
match (the line sits inside a changed hunk) is flagged for a mandatory human
read rather than a best-effort guess.

See Q5 for the trade-off against building a full diff/alignment table instead
of a search; the design recommends the search approach as a first cut, because
it is cheaper and because it reuses, line-citation by line-citation, the exact
"relocation self-verifies because it's the same text" property that the
brief's "compare the quoted text or the first N chars" is reaching for.

### (d) `RULES.md` citations

Same tool as (c), with a **third**, independent check available for free:
`RULES.md` already stores a verbatim excerpt per rule
("rule → verbatim excerpt → source file:line → implementing function"). After
relocation, search for that stored excerpt in the new file near the relocated
line as a cross-check — two independent methods (text-search relocation,
excerpt-text confirmation) agreeing is strong evidence; disagreement escalates
to a human rather than silently picking one.

### (e) `docs/` dated records

**Not rewritten — this is already settled, not an open question.** D1c
decided line citations inside dated historical records (implementation plans,
reviews, findings sheets) stay as the snapshot they were written against;
rewriting them to a post-sync line would falsify the record. The only P8
action here is a documentation courtesy, not a requirement: a single dated
note in `docs/rules-source-resync.md` recording the sync's commit SHA and
date, so a reader of an old `docs/` citation knows which `rules/source/`
revision it was accurate against.

### (f) Verbatim-description guards (the 655-entry sweep, `MECHANICAL_PHRASES`, etc.)

"Descriptions are verbatim" (project memory) means matching the *current*
canonical source, not a frozen snapshot of a past OCR error — so where
upstream's fix changes the actual passage text (not mere paragraph reflow),
the stored `description`/`summary` in both locales is now stale against the
new canonical source and should be refreshed, test-first, exactly like the
original sweep (D5 / `uncomputed_clauses.rs` machinery), in the
pending-work-list shape ("every commit is green"). Scope is bounded by (a)'s
relocation: once an entry's range is relocated, diff its old bracketed passage
against its new one; byte-identical or reflow-only needs nothing, a real text
difference goes on the refresh list. Expected small for the clean group
(the one concrete sample found, `Subernatural`→`Supernatural`, is a strict
improvement); size for the structurally-heavy group is unmeasured — Phase
P8-0 must determine how many anchored entries in those books exist before
this can be sized beyond "unknown, possibly large."

This phase is also where the four known local fixes get **re-applied** as a
distinct, reviewable, line-count-neutral patch (Q3), since a same-line
character substitution is a special case of "description now differs from
source" with the opposite direction of travel (ours is right, upstream is
still wrong).

### (g) `book_templates.rs` and other line-pinning tests

Same tool as (c). Phase P8-0 must first enumerate every test file that pins
a bare `ArMDE:`-shaped citation (`book_templates.rs` is the known one; there
may be others in `crates/*/tests`) before this can be sized.

## Phases

Sizes per the project's convention: **S** ≈ ½ session, **M** ≈ 1 session,
**L** ≈ 3 sessions.

### P8-0 — Measure, remaining pieces (S)

**Done 2026-10-02: see `p8-0-census.md`.** It confirms the Core-only assumption for
every anchored entry, re-estimates P8-4 at M (L without continuation parsing) and
P8-5 at S, and finds 58 hard line pins in 7 test files that must move in P8-3's commit.

This design's own measurement pass (above) covers the heading/body/line-count
questions the brief asked for. Two things are still unmeasured and gate
sizing for later phases:
1. **The entry-less citation census** — every bare `ArMDE:`/other-acronym
   citation outside `rules/core/*.json`, by file and acronym
   (`crates/*/src`, `crates/*/tests`, `ui/src`, `RULES.md`,
   `book_templates.rs`). `phase-2-plan.md`'s own M0 may already have produced
   a version of this ("the entry-less `ArMDE:` citations in the three Phase-2
   working files and `book_templates.rs`") — check currency before
   re-deriving.
2. **Which row-key catalogues cite which books** — confirm
   `equipment.json`/`childhoods.json`/`aging.json` cite Core Rules only (the
   working assumption above), so the structurally-heavy group's deepened
   heading hierarchy is confirmed irrelevant to today's shipped row-key
   anchors rather than merely assumed so.

### P8-1 — Build the relocation tooling, test-first (L)

- **T1 (M)**: anchor→line relocation for `rules/core/*.json` + the DE
  sidecar, per (a)/(b) above. Red-then-green against fixtures (a tiny
  synthetic old/new Markdown pair with a reworded heading, mirroring
  `rules_source_provenance.rs`'s own test style), `#[ignore]`d like its
  siblings since it writes into `rules/`.
- **T2 (L)**: text-search relocation for bare line citations, per (c)/(d)/(g).
  Takes a pre-sync and post-sync file pair plus a list of
  `(file, acronym, line_or_range)` citations; emits
  `(old, new, confidence)` triples, confidence being `exact` / `fuzzy` /
  `unresolved`. Unit-tested against synthetic fixtures covering: an unchanged
  region (trivial offset), a region inside a changed hunk (must report
  `unresolved`, never guess), and a region containing the OCR-class
  single-character fixes actually found in measurement (confirms search
  tolerance is neither too strict — missing the real match because of the
  single fixed character — nor too loose — matching a wrong, coincidentally
  similar, line elsewhere).
- **T3 (S)**: strengthen `rulebook_citations.rs` per `open-todos.md`'s own
  recommendation ("checks the cited range against an *expected excerpt*
  rather than merely non-blankness"). Snapshot each current bare citation's
  first-line text as its expected excerpt, stored as a new guard input;
  the guard then fails if a future edit (sync or otherwise) moves a citation
  onto text that no longer matches. **Land this against today's pre-sync
  corpus, as its own gate-strengthening slice, before P8-2** — see Q7: it is
  valuable independent of this resync and should be proven on known-good data
  first, not bootstrapped mid-migration.

### P8-2 — Stage the sync (M)

One commit, touching only `rules/source/en/*.md` and `rules/source/de/*.md`
(the 9+7 files already carried; not the 3 new German RoP books or the English
Rhine Tribunal file — see Q2), copied verbatim from upstream. Nothing else
changes in this commit — it is deliberately "broken" (every citation now
stale) so the diff is reviewable in isolation and every later phase's diff is
reviewable against a known-clean baseline. The pre-sync commit SHA is the
rollback point and needs no separate tag; git history already holds it.

Immediately follows, as its **own** commit: re-apply the four known
local same-line fixes (Q3), each diffed individually and verified
line-count-neutral (`wc -l` before/after per file, matching CLAUDE.md's
absolute invariant). Kept separate from the raw copy-in specifically so it
can be reverted/redone independently if one of the four needs a different
re-application (e.g. if upstream's surrounding wording shifted enough that
the same string substitution no longer applies cleanly).

### P8-3 — Relocate anchored data citations, (a)+(b) (M)

Run T1 across every `FULLY_ANCHORED_CATALOGUES` entry, one pass, both stores
together. Diff-review before committing — `CLAUDE.md`'s canonical
serialization means this should be a narrow, mechanical diff (anchors and
`lines` values only), and anything wider is itself a signal something went
wrong. Triage the manual-review list T1 emits (expected: the ~12-20 genuine
reword cases found in measurement, including `virtue.rard`'s `Rard`→`Bard`
heading — note this coincides with the already-planned X9b id rename, worth
sequencing together rather than twice — and HoH:MC's "5."→"6." Combat
Statistics renumbering).

**Gate**: every test in `rules_source_provenance.rs` must pass —
`every_core_source_citation_brackets_real_content_in_its_named_file`,
`every_cited_source_file_exists_under_rules_source_en`,
`every_recorded_source_anchor_resolves_to_its_own_heading`,
`every_localized_source_anchor_resolves_to_a_heading`,
`german_anchors_sit_on_the_same_line_as_their_english_counterparts`,
`every_anchored_catalogue_entry_records_the_heading_that_opens_its_range`,
`no_source_range_runs_past_the_heading_that_follows_it`,
`every_subdivided_item_really_is_subdivided`,
`no_source_range_ends_on_a_blank_line`,
`every_guarded_effect_cites_a_passage_that_names_its_own_mechanic`,
`known_misencodings_still_fail_the_guard`,
`only_fully_anchored_catalogues_carry_a_source_block` — this whole
sub-system exists to stay green through exactly this event.

### P8-4 — Relocate bare citations, (c)+(d)+(g) (size set by P8-0's census — provisionally L)

Run T2 across the census from P8-0. Apply exact-match relocations
mechanically, reviewed as a diff; hand the fuzzy/unresolved bucket to a human
pass, grouped by acronym and file. `RULES.md` additionally gets the
excerpt-text cross-check (d). `crates/arm-app/tests/menu.rs`'s vendored
`muda-0.19.3` citation is explicitly **not** touched — it pins a dependency
version, not a rulebook, and is out of scope.

**Gate**: `rulebook_citations.rs`'s four existing tests, plus T3's new
excerpt check from P8-1.

### P8-5 — Verbatim description refresh, (f) (size set by P8-0/P8-3 — provisionally M, possibly L for the structurally-heavy group)

For every relocated entry whose old-vs-new bracketed passage differs by more
than reflow, refresh `description`/`summary` in both locales, test-first,
pending-work-list shape. Re-verify the four local fixes landed in P8-2 are
still reflected correctly in the i18n text (they were already fixed there
once; confirm the refresh pass doesn't silently revert them by re-copying the
still-broken upstream wording over the already-fixed local text — this is a
real failure mode of an automated refresh and needs an explicit exclusion
list naming the four fixed entries).

### P8-6 — D31/D18 retirement (M)

D31: *"When a current German rulebook does land, D7.1 returns to full force
and this decision retires."* **This sync is that landing.** Re-derive, per
`tmp/table-sync-check.md`'s method:
- The two D18 worked examples (`Relikt`→`Reliquie`, `Fluch der Kirke`→`Fluch
  der Circe`) — measurement confirms the DE heading has indeed changed to
  match the table upstream, so this should be a confirmation, not new work:
  check our `rules/i18n/de/` already says "Reliquie" (it should, D18 ruled
  "table wins; data changed"), and that the new anchor slug
  (`reliquie`/`mächtige-reliquie`) is what P8-3 actually recorded.
- The 7 reverted-name entries (Deteriorating Power, Disorientating Magic,
  Enfeebled, Environmental Magic Condition, Environmental Sensitivity,
  Vulnerable Magic, Vulnerable to Folk Tradition) and the 19 live
  name-disagreements adopted under D31 — re-check each against the **new**
  heading text. Where upstream's heading now agrees with the table, D7.1's
  revived default and D31's table-preference agree too and nothing moves.
  Where it still disagrees, D7.1 (not D31) now governs: the rulebook heading
  wins unless defective (collision/absent), per D7 rule 2.
- One case measurement surfaced that the audit never flagged as a
  table-vs-heading dispute, because it wasn't one before: DE HoH:S's
  `Infernale Kontamination`→`Infernale Befleckung` is a genuine upstream
  terminology change with no prior table dispute attached. Treat it as a
  fresh D7/D6 case, not folded into D31's existing 21.

### P8-7 — New-content triage, docs only, nothing implemented (S)

Log to `docs/open-todos.md`: the "Frosty Breath of the Spoken Lie" spell gap,
and (separately) the newly-available German RoP books and English Rhine
Tribunal file as a future sourcebook-onboarding decision (Q2). Neither is
implemented as part of P8 — CLAUDE.md's "new content is not free" applies in
full: a rule may only be implemented from source text, by its own test-first
slice, never bundled into a citation-relocation pass.

### P8-8 — Full verification gate + doc updates (M)

Full round gate per `CLAUDE.md`'s "Required gate", plus `npm run test:e2e`
(phase boundary, not per-slice, per the project's e2e policy) since this
touches `arm-app` resource loading indirectly through every consumer of
`rules/`.

Doc updates:
- `CLAUDE.md`'s "rules/source/ is a COPY" section states concrete line counts
  ("25,803 lines... 25,723... 80 lines longer") as a **living fact**, not a
  dated record — unlike `docs/` citations (D1c), this must be corrected (or
  the specific-numbers framing removed) once the counts no longer hold.
- `docs/rules-source-resync.md`'s "Still owed" section, closing out the
  "re-sync itself" line.
- `docs/open-todos.md`'s P8 row — closed per the project's convention
  (commit message is the archive), new rows added for P8-7's deferred items.
- `crates/arm-rules/RULES.md` — citations relocated in P8-4 are already
  covered; no separate doc-update action.

## Risks and rollback

- **Every phase is its own commit**, so a bad phase can be reverted on its
  own as long as later phases haven't touched the same lines — which is why
  P8-2 splits the raw copy-in from the four-fix re-application, and why
  P8-3/P8-4/P8-5 touch disjoint file sets (`rules/core/`+sidecar,
  code/docs comments, i18n text respectively). Catastrophic rollback is
  always available via the P8-2 pre-sync commit SHA; nothing is destructive
  to git history.
- **Biggest unmeasured risk: P8-4's size.** Depends entirely on P8-0's
  census, which this design pass did not run (it requires enumerating, not
  sampling). Treat the "L" above as provisional and re-estimate once P8-0
  reports a real count.
- **Row-key anchors in the structurally-heavy group.** The working
  assumption — today's row-key catalogues (`equipment`, `childhoods`,
  `aging`) cite Core Rules only, so the heavy group's deepened heading
  hierarchy is irrelevant to anything shipped today — is unverified. P8-0
  confirms it before P8-3 relies on it.
- **An automated refresh (P8-5) silently reverting an already-fixed local
  description back to upstream's still-wrong wording** is a real failure
  mode for exactly the four known fixes, not a hypothetical one — P8-5 states
  the required exclusion list explicitly for this reason.
- **The anchor-rewrite tooling (T1) must follow the existing
  `regenerate_source_anchors` discipline** — rewrite the `source` line
  in place, never round-trip the whole JSON document through `serde_json`
  (which would reflow every entry and bury the real diff in formatting
  churn, against CLAUDE.md's "Canonical serialization").

## QUESTIONS for Norbert

**Q1 — Does this design authorize execution, or is a separate go/no-go
needed before P8-2?** D18's "DO NOT re-sync" and the Phase 2 plan's exclusion
are standing prohibitions this document was explicitly asked to prepare for,
not override. *Recommendation:* treat this document as the prerequisite
artifact only; require an explicit separate go-ahead before P8-2 (the first
state-changing phase) begins, consistent with "wait for the go-ahead."

**Q2 — Fold in the three newly-available German RoP books
(Faerie/Divine/Infernal) and/or the English Rhine Tribunal file, or strictly
limit P8 to the 9+7 books already carried?** *Recommendation:* strictly
limit. Onboarding a new sourcebook is a different shape of work — new IDs,
a full extraction pass, English-source-of-truth questions for Rhine Tribunal
— and bundling it into a citation-relocation exercise turns an already large,
already risky change into an unbounded one. File as its own future decision
(P8-7).

**Q3 — The four known local same-line fixes are confirmed still unfixed
upstream; a straight sync regresses all four.** *Recommendation:* re-apply
all four as an explicit, line-count-neutral patch commit immediately after
the raw copy-in (P8-2), rather than waiting on upstream to catch up — the
alternative ships a known regression. Confirm, or say whether you'd rather
push on `arm-de-translation` first so the fix is genuinely merged before
syncing (avoiding permanent duplicate-fix bookkeeping going forward).

**Q4 — Should P8-6 (D31/D18 retirement) run as part of this plan, or be
deferred to its own later slice once the rest of P8 is green?**
*Recommendation:* run it as part of this plan (P8-6) — the sync is
specifically what unblocks it, and leaving the name verdicts unreconciled
again re-creates the exact "circular, stale copy convicting a current one"
failure D31 already had to correct once.

**Q5 — Text-search relocation (T2) vs. a full diff/alignment table for bare
code-comment citations.** *Recommendation:* text-search first — cheaper to
build, and it reuses the same "the relocation is only valid because the text
matched" verification property a full alignment table would also provide.
Confirm, or require the more robust (and more expensive) alignment-table
approach regardless, given upstream is visibly still active and there will
likely be a next re-sync this investment should also serve.

**Q6 — When upstream's OCR fix measurably improves a passage already
transcribed verbatim (e.g. `Subernatural`→`Supernatural`), refresh the local
`description`/`summary`, or leave it since the intended meaning was already
captured?** *Recommendation:* refresh. "Descriptions are verbatim" means
matching the *current* canonical source, not a frozen snapshot of a past OCR
error.

**Q7 — Land T3 (the excerpt-based `rulebook_citations.rs` strengthening)
before the sync, proven against today's known-good corpus, or build it as
part of the same pass?** *Recommendation:* land it first, as its own small
gate-strengthening slice — it has value independent of this resync, and
proving it against stable data first de-risks trusting it during the actual
migration.

## Size summary

P8-0 (S) → P8-1 (L: T1 M, T2 L, T3 S) → P8-2 (M) → P8-3 (M) → P8-4
(provisional L) → P8-5 (provisional M-L) → P8-6 (M) → P8-7 (S) → P8-8 (M).
**Provisional total: roughly 10-13 sessions**, with P8-4's and P8-5's real
sizes unknown until P8-0's census runs — this is explicitly not a final
estimate, per the risks above.

```
P8-0 → P8-1 (T1, T2, T3) → P8-2 → P8-3 → P8-4 → P8-5 → P8-6 → P8-7, P8-8
                                    ^ gate: rules_source_provenance.rs, full
                                      ^ gate: rulebook_citations.rs, full
```
