# Open to-dos

Items waiting on a decision, a visual check, or a follow-up pass. Kept here so
they survive a session ending. **The table below is what is still owed.**

**Completed items are not kept here.** They used to be, and by 2026-09-15 the
closed-item history had grown to roughly 900 lines — seven times the live
content, so the file no longer answered the one question it exists to answer.
What a finished item settled lives in its commit message, which is where a
reader looking for *why* will be anyway; `git log` is the archive.

**Surface this list when a release or a git tag is being prepared** — none of
these should be tagged over silently.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 38 | **The `description` fill is 74 entries into a 655-entry book. The Flaws block is DONE and the third uncomputed-clause assertion has LANDED GREEN, scoped to it; the Virtues block is next.** *Updated 2026-09-15.* The Flaws sweep (`ArMDE:5639-7113`) read all 51 flagged `narrative` Flaws in both languages: **49 reclassified `narrative` → `uncomputed_rule` with `description` filled in both locales** (48 texts — `flaw.missing_ear`'s passage is a single sentence its `summary` already carries in full), and **2 confirmed correctly `narrative`** (`flaw.overconfident_major`/`_minor`, whose passage uses "botch" as a bare roleplaying verb and states no rule; recorded with their reading in `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`, kept honest by `exempted_entries_still_trip_the_screen`). The third assertion, *"a `narrative` entry whose cited passage carries a mechanical token is a dropped rule"*, is now `uncomputed_clauses.rs::no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` — **green**, scoped by `SWEPT_BLOCKS`, so re-dirtying the Flaws block is a failing build rather than a claim in a report. **What remains:** the **Virtues block (`ArMDE:3360-5282`)**, unsurveyed; running the same guard with a Virtues row added to `SWEPT_BLOCKS` is the cheapest way to produce its work list — that is how the Flaws block turned out to be 51 rather than the estimated 43. Same method: extract the full passage into `description` in English, take the German from the line-mirrored `rules/source/de/` file at the same line number, convert en dashes before digits to ASCII hyphens, strip Markdown link and emphasis syntax, and reclassify to `uncomputed_rule` where the engine computes nothing. **Also record the heading anchor while reading** (row 43) — it costs nothing during a read that is happening anyway and is a second full pass over both books if deferred. Beyond the rule-droppers sit roughly 550 entries whose rule is either already computed and displayed or is genuine flavour: worth doing for uniformity, not for correctness, and a separate decision. | an implementation pass (data, both locales) | 2026-09-14 |
| 39 | **`ArMDE:7054` reads "guarter" where it means "quarter", and the typo is now shipped twice.** An OCR artefact in the source Markdown, faithfully transcribed into `flaw.waster_of_vis`'s `summary` and — by the same verbatim-from-source convention — into its new `description`. The convention is doing its job, so this is not a transcription bug, but it is user-visible in both locales' English text. **Fixing the source line is the correct move**, not the i18n: patching only `rules/i18n/` would break verbatim-ness and would be silently undone by any future re-extraction. Not done when found purely because `rules/source/**` was outside that slice's declared file list. One character, one line, then re-sync the two i18n strings. The German line (7054 of *Basisregeln*) reads "Viertel" correctly and needs nothing. **Two more source defects found since and belonging with it:** `ArMDE:6929` reads "-3penalty" with no space, and the German sources close a German opening quote `„` with an ASCII `"` (see "Recorded conventions" below — that one is corpus-wide house style and is **not** to be swept; it is listed here only so the two are not confused). | an implementation pass (source fix + i18n re-sync) | 2026-09-14 |
| 40 | **The rules-text search index reads `summary` only, so everything written into `description` is unsearchable.** `ui/src/lib/derive.ts` builds the V/F search index from each entry's `summary` alone. That was harmless while `description` was empty for all 655 V/F; it is now populated for 74 of them, and the populated half is exactly where the mechanical clauses live — so searching "botch" finds the two entries whose clause happened to land in sentence one rather than the fifty-odd that state it. The investigation called this a one-line change that should ship with the data. **Not done in the data slices**: their briefs scoped `ui/` to render-blocking changes only, and the index change alters search behaviour, so it wants its own test and a UI-side decision about whether a long paragraph should be indexed whole. The tooltip itself needed no change and got none — `VirtueFlawTab.svelte`'s `tip()` already reads `description ?? summary`. | an implementation pass (frontend, with a test) | 2026-09-14 |
| 41 | **The Markdown export lists Virtues and Flaws by name only, so no rules clause reaches an exported sheet.** `crates/arm-rules/src/export/sections.rs` emits neither `summary` nor `description` for a V/F. Under the old design that was defensible — the tooltip carried the text and the export carried the sheet. It stops being defensible now that "the displayed text is the only carrier" is the adopted principle for `uncomputed_rule` entries: an exported sheet has no tooltip to fall back on, so a player working from the export sees a Flaw's name and nothing about the botch die it costs them. Flagged by the investigation as a consequence to decide deliberately rather than discover later. Needs a product call on how much text an export should carry (name only, name + summary, or name + full description) before any code. | a decision, then an implementation pass | 2026-09-14 |
| 42 | **Four swept Flaws state rules the engine could compute but has no representation for — and three of them are *caps*, which nothing in the engine can express at all.** Surfaced by reading every cited passage in the Flaws block (2026-09-15); all four are now `uncomputed_rule` with their full text displayed, which is honest but is the weaker of the two possible answers. (a) `flaw.weak_personality` (`ArMDE:7078`) caps **every** Personality Trait to the range +1..-1 *and* ceilings the roll ("treat any roll above 6 as merely 6"). (b) `flaw.uninspirational` (`ArMDE:6921`) caps **two Characteristics**: "His Presence and Communication may not be greater than 0". **Re-costed 2026-09-15 by the row-32 fix:** this row previously said the engine "already has `characteristic_limit`, so this one is arguably encodable today". That effect is gone — it was never a cap in the first place, only Great/Poor (Characteristic) mis-modelled as one, and it is now `characteristic_score_delta_param`, a *grant*. Uninspirational, by contrast, **is** a genuine per-Characteristic buy cap, and the engine now has no way to express one: it would need a new effect variant lowering `base_max` for a named Characteristic, plus `characteristic_cap` becoming entity-relative again. Still the cheapest of the four, but engine work rather than a data edit. (c) `flaw.fickle_nature` (`ArMDE:6124`) is a creation-time **grant**, not a penalty: "Select a Personality Trait at +4, and its opposite at +4". (d) `flaw.lingering_injury` (`ArMDE:6352`) is a **formula**, not a constant: the penalty is multiplied by `1 + Decrepitude Score`. The common blocker for (a), (c) and partly (d) is that Personality Traits are free-form entries on `types.rs::Entity::personality_traits` with **no granting effect and no validation of any kind** — a user can type +9 on a Weak Personality character and nothing objects. Deliberately *not* fixed in the sweep: inventing an effect variant to fit is what `CLAUDE.md` → "Rules provenance" forbids, and each of these is a wrong-rules-output change wanting its own TDD slice and a reading decision. Rated as product-integrity rather than cosmetic, but low urgency: today the rule is *displayed* rather than *enforced*, which is the same standing as every other `uncomputed_rule` entry. | an implementation pass each, (b) first | 2026-09-15 |
| 43 | **Heading anchors are recorded for 51 of ~1000 `source` blocks, and the rulebook sources are about to move.** Norbert has confirmed the `rules/source/` Markdown will be updated upstream with **line numbers changing throughout**, and the German edition re-synced. Every `source: { file, lines }`, every acronym-plus-line citation, and every line reference in `crates/arm-rules/RULES.md` shifts at once — and `rulebook_citations.rs` cannot see it, because it checks a cited range lands on non-blank lines rather than that those lines say what the citation claims. The durable key already exists in the sources: every virtue and flaw has its own `####` heading in both languages, and the Markdown already generates anchor slugs in its own cross-links. The Flaws block now carries them (English on `types.rs::SourceRef::anchor`, German in the `rules/i18n/de/source_anchors.json` sidecar), with three guards including the first test of the German line-parity invariant. **What is owed is the rollout**: every later sweep records anchors as it reads, per the cheap-now-expensive-later argument. Full design and residual questions — notably whether prose code comments should move off line numbers at all — in `docs/rules-source-resync.md`. Related: P8, the re-sync itself. | rollout during later sweeps | 2026-09-15 |

## The `EffectiveScores` DTO extraction is deferred

**This is the audit's one consciously accepted architectural deferral, and this
entry is its only record in the repository.** It was declined twice,
independently, in round 2; both fixers wrote the reasoning into
`tmp/review/`, which is gitignored scratch that nothing preserves, and a commit
message then asserted it was recorded here when it was not. The gap was filed in
round 3 and is closed by this entry — the decision itself stands and is **not**
being re-litigated.

(Note for anyone editing this entry, the same one the P8 section carries: do
**not** write a source location as a file plus a line number here. `docs/` is
inside the `rulebook_citations.rs` sweep, which parses anything shaped like an
acronym followed by a colon and digits as a real rulebook citation. Name the
**symbol** instead — the form `` `file.rs::Symbol` `` — which is also the
convention `CLAUDE.md` requires for a cross-reference to a source file, and
which cannot rot when the lines below it move.)

**What is owed.** `crates/arm-app/src/ruleset_io.rs` carries the whole IPC
read-out DTO layer — `ruleset_io.rs::EffectiveScores`, the private per-domain
field structs it is assembled from (`ruleset_io.rs::XpFields`,
`ruleset_io.rs::SpellFields`, `ruleset_io.rs::WarpingFields`,
`ruleset_io.rs::CharacteristicFields`, `ruleset_io.rs::ConfidenceFields`,
`ruleset_io.rs::GrantBudgetFields`, `ruleset_io.rs::DecrepitudeFaithItemFields`,
`ruleset_io.rs::SpellMasteryFields`, `ruleset_io.rs::MightPowerFields` and their
siblings), their assemblers, and `ruleset_io.rs::effective_scores_loaded` —
inside a module that is also the ruleset loader, the path helpers and the aging
commands. Roughly 730 lines of one concern under a name that promises another.
The destination is a module of its own, `crates/arm-app/src/effective_dto.rs`.

**Why it is deferred rather than done.** Moving it is not the hard part; the
**pointers into it** are. Its cross-references span crates and documents — four
in `crates/arm-rules/RULES.md`, one in `ui/src/lib/derive.ts`, and two in
`docs/` — so an extraction performed inside any single slice's file list
strands the rest, manufacturing exactly the stale-pointer defect round 2 spent a
slice repairing. That is the whole argument, and it is the reason the move needs
one coordinated slice rather than a spare afternoon.

**The file list that slice must own**, so it is actionable rather than a wish:

- `crates/arm-app/src/ruleset_io.rs` (the source), `crates/arm-app/src/lib.rs`
  (the module declaration) and `crates/arm-app/src/commands.rs` (the callers).
- `crates/arm-app/tests/commands.rs`.
- `crates/arm-rules/RULES.md`, `ui/src/lib/derive.ts`, and the two `docs/`
  references — the pointer set, which is what makes this a coordinated slice.

The single DTO test inside `ruleset_io.rs`'s own `#[cfg(test)]` module moves
with the block; leaving it behind would put a test for one module in another,
which is the same drift in miniature.

## The e2e harness leaks its driver processes

Observed directly, twice, and confirmed as the cause of a false failure — so this
is a measurement, not a suspicion.

`npm run test:e2e` and `npm run test:e2e:portable` both leave
`tauri-driver --port 4444 --native-port 4445` and
`/usr/bin/WebKitWebDriver --port=4445` **running after the suite exits 0**. They
are not cleaned up on a normal, fully-passing run.

Why it matters, and why it cost a real debugging detour: a later run does not
start its own driver on an occupied port — it connects to the **stale** one, and
`POST /session` then hangs for its full 120 s timeout and fails. The symptom is
maximally misleading:

- It reports as `Failed to create a session: WebDriverError: timeout`, which
  reads like infrastructure flakiness.
- It is **deterministic**, not flaky, so "run it again" reproduces it exactly and
  appears to confirm a genuine regression.
- It strikes whichever spec happens to be scheduled first (here
  `app-shell.e2e.js`), so it looks like that spec is broken.
- The other nine specs pass, which makes it look like a defect localized to one
  area rather than an environment problem.

During the round-2 gate this produced a 9-passed/1-failed result that survived a
full re-run and a single-spec isolation run, and was investigated as a suspected
startup deadlock in the round-2 close-guard rework before `ps` showed two
day-old driver processes still holding the ports. Killing them made the same
commit pass 10/10 with no code change.

Worth fixing rather than remembering: an `onComplete` hook in
`ui/e2e/wdio.shared.conf.js` that reaps the driver it spawned, or a preflight
check that refuses to start when 4444 is already held by a process this run did
not create. The preflight is the more valuable half — it converts a silent
120 s hang into an immediate, accurate error message.

Until then: if e2e fails at session creation, check `ps aux | grep tauri-driver`
**before** reading anything into the failure.

## `.icon-btn` lost its e2e coverage

Not one of the audit findings; turned up while repairing dangling e2e spec
citations, and recorded because it is a **coverage regression nobody noticed**,
not a comment defect.

`ui/src/app.css` claimed `.icon-btn` — every `×` remove button and every
`+`/`-` stepper in the app — was measured in a real browser engine by an e2e
spec. No spec measures it: `grep -rn "icon-btn" ui/e2e/` is empty. The pointer
has been dangling since the 43→10 spec consolidation, so the check was dropped
silently rather than deliberately.

Why it matters more than a typical stale comment: this is a **WCAG 2.5.8
pointer-target floor with roughly 0.2px of margin**. The only surviving guard
parses the CSS text, which can prove the declared size and nothing else — it
cannot see the button being squeezed by its container, which is precisely the
failure a real layout would catch and the reason the e2e check existed.

The repair pass wrote an honest "no e2e spec measures this" comment rather than
inventing a replacement pointer, so the gap is now visible in the source instead
of disguised. Restoring it is small: a one-`it` addition to `wizard-flow.e2e.js`'s
tab-area describe, which already has the harness set up.

## Product changes requested by Norbert, 2026-09-13

Raised in-session during the full-codebase audit. These are **product decisions
already taken**, not audit findings and not open questions — they are recorded
here because they arrived mid-audit and are owed as their own pass rather than
folded into a review fix. Numbered separately from the table above, which is for
items still waiting on a decision; nothing below is waiting on one.

| # | Item | Kind |
|---|---|---|
| P1 | **Menu → Window → Fullscreen does nothing. Confirmed on Windows hardware (2026-09-15) as well as Linux**, so this is the menu model being right and the handler being absent, not a platform quirk. The item exists in the native menu and is inert. The menu model is unit-tested *as data*, so an item that is correctly declared and does nothing when activated is exactly the class those tests cannot catch — `crates/arm-app/tests/menu.rs` proves the accelerator parses and the item installs, never that the handler fires. Whatever fixes this should also close that gap for the item it fixes; it is the only defect the Windows hardware run turned up. | defect |
| P2 | **Remove the Settings button from the web UI.** No longer needed — settings are reachable from the native menu. Check `App.svelte`'s `inert` predicate and `store.settingsOpen` on the way out; the settings dialog itself stays, only the in-page button goes. | removal |
| P3 | **Put the character name in the window title**, as is conventional for desktop applications, with a leading **asterisk** marking unsaved state. Touches the existing window-title `$effect` in `App.svelte` (covered by `App.client.test.ts`) and reads `AppStore.dirty` — the same flag the mandatory unsaved-changes guard uses, so the dirty source is already there and must not be duplicated. | change |
| P4 | **Remove the "Unsaved document" text.** Superseded by P3's asterisk — the two would say the same thing twice. Do P3 and P4 together, or the app briefly has neither indicator. | removal |
| P5 | **Move the logo into the character-type / character-name section**, right-bound, sized to the combined height of the character-type and character-name lines. Norbert confirmed the logo reads correctly on the light background **at any size**, so moving and resizing it owes no new visual check. | layout |
| P6 | **Move "continue in guided creation" to below the logo**, on the same line as the character one-liner description (e.g. "Knight of the Teutonic Order…"). | layout |
| P7 | **Table and panel backgrounds should be a lighter beige, not white.** A palette change in `ui/src/app.css`. Note `app.css.test.ts` machine-checks contrast ratios in both palettes — any new background must clear 4.5:1 against the text on it, and the test is the place to prove it rather than the eye. | visual |
| P8 | **Update the rules Markdown sources — there have been substantial changes.** Re-sync `rules/source/en/*.md` and `rules/source/de/*.md` from upstream. **Read the paragraph below before starting: this is the single most disruptive change in this file.** | data |

**Not yet scoped or estimated.** P3+P4 and P5+P6 each pair naturally; P1 is a
defect and is independent of the rest.

### P8 is a line-number earthquake, not a file copy

Every rule in this codebase is cited by **acronym + line range into these exact
files** — the nine acronyms in `CLAUDE.md`, each followed by a line number, as
in the rounding default at `ArMDE:547` — and that convention is only safe
because a published rulebook never moves under you. Re-syncing the sources
breaks that assumption for every citation at once. What is affected:

(Note for anyone editing this entry: do **not** write illustrative citations
with invented line numbers here. `docs/` is inside the
`rulebook_citations.rs` sweep, which parses anything of that shape as a real
citation and fails when the range lands on blank lines — this paragraph did
exactly that when first written, and turned the suite red. It then happened a
second time in `docs/rules-source-resync.md`, so the warning is evidently easy
to read past: use a range you have verified, or name no line at all.)

- **Hundreds of citations** across `crates/*/src`, `crates/*/tests`,
  `crates/arm-rules/RULES.md`, `ui/src` and `docs/`. Any line inserted near the
  top of a source file shifts every citation below it in that file.
- **`crates/arm-rules/tests/rulebook_citations.rs`** will keep passing while
  being wrong. It checks a cited range lands on **non-blank lines** — not that
  those lines say what the comment claims. So a wholesale shift produces a green
  suite and hundreds of citations silently pointing at the wrong rule. This is
  precisely the failure `docs/audit-2026-08.md` records: four wrong ranges hid
  behind unverifiable shorthand and were each found only by opening the file.
- **The German line-parity invariant** (`CLAUDE.md`): German sources mirror the
  English **line-by-line throughout**, deliberately un-re-sorted, so a German
  line number identifies the same item as the English one. A re-sync that
  updates one language and not the other, or that changes line counts
  differently between them, destroys that property. It is now tested for the
  **swept** entries only (see row 43), so the guard's reach grows with the sweep
  rather than covering the book.
- **`rules/core/*.json` and `rules/i18n/<lang>/*.json`** carry `source`
  (`SourceRef { file, lines: [start, end] }`) per item, pointing at the English
  file. Those ranges shift with everything else — which is what row 43's heading
  anchors exist to survive.
- **New content is not free.** If the re-sync adds rules, they may only be
  *implemented* from the source text — never from recollection — and English
  remains the source of truth for IDs.

**Therefore P8 is not "copy the files in".** The work is: sync, then
mechanically re-derive every citation against the new text, then re-verify by
opening files rather than by trusting a green suite. Worth considering as part
of the same pass: strengthening `rulebook_citations.rs` so it checks the cited
range against an *expected excerpt* rather than merely non-blankness — which
would make the next re-sync a caught failure instead of a silent one.

There is a gitignored helper, `rules/source/sync-sources.sh`, that refreshes the
German sources from `arm-de-translation`; check whether it is still current
before relying on it.

## Recorded conventions — nothing owed

Neither of these is work. One is a settled policy and the other a reference
list; they live here so the table above means what it says.

### The native menus: Windows is verified, macOS is deliberately not

This was row 26 until 2026-09-15, asking for a run on real Windows and macOS
hardware. It is closed, in two different ways, and the reasoning is worth keeping
because the row would otherwise be re-raised.

**Windows: done.** Norbert ran it on Windows hardware. The menu bar works. The
one defect it turned up is the inert Window → Fullscreen item, which is P1 above
and was already known from Linux — so the hardware run confirmed the model is
right and a handler is missing, rather than finding a platform quirk.

Worth preserving from the original row, because it is the reason the Windows menu
looks asymmetric: `PredefinedMenuItem::quit` on Windows would have **bypassed the
unsaved-changes guard outright** — muda implements it as `PostQuitMessage(0)`,
which ends the message loop instead of raising a close request — so the Windows
menu deliberately ships **no** Quit item and offers Window → Close Window
(`WM_CLOSE`, guarded) as the way out. That reasoning lives in the doc comment on
`menu_model` (`crates/arm-app/src/menu.rs`); do not "fix" the missing Quit item.

**macOS: not owed, and not a gap in the testing.** There is no macOS hardware
here and **no prebuilt macOS download is released** — `README.md` states this
outright; the app builds and runs on macOS from source, which is a different
promise from a shipped binary. So the untested path is the **Cmd+Q route through
`RunEvent::ExitRequested`**, which the mandatory unsaved-changes guard depends on
and which no Linux or Windows run exercises. That is accepted risk on an
unreleased target, not an outstanding task.

Two consequences to keep in view rather than rediscover. `CLAUDE.md`'s stack
table still lists macOS among the targets, which is true of the source build and
overstates what is released — if macOS ever *is* released, this becomes a real
pre-release item again and the Cmd+Q path is the first thing to exercise. And
`crates/arm-app/tests/menu.rs` still proves the macOS menu *model*, which remains
worth having: it is what makes a future macOS run a verification rather than a
first draft.

### German quote glyphs are the corpus's house style, and `rules/source/de/` stays as published

Measured 2026-09-09 with `rg -a --count-matches` over `rules/source/de/`:
**1425** occurrences of `„…"` — opening with `„` (U+201E), closing with an ASCII
`"` — against **6** correctly paired `„…“` and **0** genuine inverse pairs. Split:
**1345** across all **7** German rulebook files (Basisregeln 309, Mysterienkulte
240, Heckenzauber 230, Societates 190, Wahre Linien 128, Rhein-Tribunal 125,
Sphären der Macht — Magie 123) and **80** across 10 files under
`translation-tables/` (grundbegriffe 30, tugenden-fehler 12, that directory's own
README 12, sphären-mächte 9, orden-tribunale 5, konvent 5, reputationen 4,
zauber-nach-form/tiere-kreaturen/magie-regeln 1 each). All **6** correct pairs sit
in *Sphären der Macht — Magie*, on five lines: :4495, :4547, :4623, :4681 and
:4885, which carries two. That doubled line is also why a naive `“…„` scan reports
one "inverse" hit — it matches the gap *between* the two correct pairs, so the
genuine inverse count is zero. Raw glyph totals corroborate: 1451 `„`, 6 `“`,
0 `”`, 1544 ASCII `"`. At 1425-to-6 this is the corpus's convention, not a slip,
so the earlier reading of a single line as a one-off defect was wrong.

**The source is not touched** — `rules/source/de/` reproduces the books as
printed, and a 1425-site sweep would rewrite the CC-BY-SA rules text over a
typographic preference. The policy instead: if an extraction ever carries quoted
text past the first sentence, it normalizes the pairing **on the way out**, in the
extractor, so `rules/i18n/de/` is clean without the source moving.

### What an older save still reports on open, and why each one is correct

**Read this before opening an old save to check it** — every finding below is
correct behaviour, and without the list a correct result reads as a regression.

The migration recovers what is mechanically recoverable from a character file
written before that round of ruleset changes, and "such a save opens clean" is
true of the **format** changes only. Nothing is corrupted in any of these — saves
store choices, not resolved values, and the engine only reports.

- **`too_many_selections`** is a genuine rules violation, not a format problem:
  two bought Puissant Arts plus a House grant really do exceed the ceiling the
  descriptor states. The engine was blind to it before the selection-cap work, so
  it survives migration by design and the character has to lose a copy.
- **`missing_param`**, on a growing list of keys that were never *stored*, so
  there is nothing to migrate from and filling a placeholder would invent someone's
  rules choices. One pick each clears it permanently. Folk Magic's `category` and
  the three per-power Flaws' `power` (`flaw.slow_power`, `flaw.restricted_power`,
  `virtue.variable_power`) were the original set; since then Folk Magic's `realm`,
  Curse of Slander's `taken_as`, Sufi's `taken_as`, both False Power entries'
  `virtue`, and the two Form-scoped Magic Resistance Flaws' `form`
  (`flaw.flawed_parma_magica`, `flaw.limited_magic_resistance`) joined it on the
  same standing policy. The last pair is worth spelling out, because the save
  looks unchanged while its *output* changes: before the Form existed, each Flaw
  was applied to **all ten Forms at once**, so an old save's Magic Resistance
  numbers were wrong on nine of them and a repeat purchase was indistinguishable
  from the first. The engine now applies each to the one Form its copy names,
  reports `missing_param` until the player names it, and the Flaw contributes
  **nothing** in the meantime — which is the honest state, not a silent revert to
  the old blanket behaviour. There is no correct Form to migrate to: the choice
  was never stored. Pinned by
  `data_integrity.rs::an_mr_flaw_selection_written_before_the_form_parameter_reports_missing_param`.
- **`prereq_not_met`** where the eligibility gates bite — a character holding The
  Gift plus Offensive to (Beings) without the Gentle Gift, which `ArMDE:6530` has
  always forbidden and nothing checked. Not hypothetical: it is the exact shape of
  the migration fixture (`V0_2_X_MAGUS_SAVE` in
  `crates/arm-rules/tests/data_integrity.rs`), which is how it came to light.
- **`unknown_param_value`** on a `realm` a player typed by hand, since four items
  were tightened from free text onto `ParameterDomain::Realm`. The typed text is
  handed back verbatim in the finding rather than discarded.
