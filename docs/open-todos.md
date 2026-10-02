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

**Every row below is also folded into `docs/vf-audit/corrections.md` § 9**, which
maps it to the audit group, ruling or finding that owns the work (Norbert,
2026-09-24). **This file keeps its own job** — it is the list surfaced at a
release, which `corrections.md` is not — so the two are a pointer pair, not a
duplicate: **the work is described there, the release gate is here.** **Row 46 is now DONE** — its premise (*"every sweep so far has read `narrative`
entries only"*) was written on 2026-09-18, **before** the batches ran; the
audit's twelve-point check covered all 655 entries, and the one residue it left
(§ 8 row **12e**, the computed entries in **B01–B11's** spans, whose
consumer-tracing step entered the briefs only at **B12**) was closed by X7c,
commit `bd291a9`.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 38 | **The narrative sweep itself is DONE (S4, 2026-09-26, commits `b5fc9da`/`8fbe68b`): the whole V/F catalogue is now one contiguous swept block (`ArMDE:3360-7113`, `SWEPT_BLOCKS` in `uncomputed_clauses.rs`), closing the `ArMDE:3951-5282` residual this row used to describe — every `narrative` entry in that range has been read in full and reclassified to `uncomputed_rule` where it drops a rule (132 newly surfaced, triaged: 20 no-rule, 70 whole-passage, 59 dropped-clause; the rest pending D46's separate classification question, now also settled).** **Still open:** the "10 unnormalized `„…"` quote pairs" this row used to cite is stale and understated — `rules/i18n/de/virtues_flaws.json` now carries **31** such pairs (grown as the Virtues-block fill pulled in more verbatim text with embedded quotes), still predating the normalize-on-the-way-out policy (see "Recorded conventions" below) and still with no guard. | an implementation pass (quote-pair normalization only, both already-filled locale texts) | 2026-09-14 |
| 39 | **DONE 2026-09-22 for the scanno (D26); the `-3penalty` half below is still open.** Fixed as this row predicted — source first, then the two i18n strings. A **second** occurrence turned up at `ArMDE:3502` (`virtue.berserk`, "give guarter" for the idiom *give quarter*), which is what proved the `q`→`g` OCR error systematic rather than a one-off; it is fixed too, and a todo is filed in `arm-de-translation` (`docs/todo.md`), whose `original-english/reviewed/` copy still carries both (at its own, different line numbers — that copy has drifted from ours). Original entry: **`ArMDE:7054` reads "guarter" where it means "quarter", and the typo is now shipped twice.** An OCR artefact in the source Markdown, faithfully transcribed into `flaw.waster_of_vis`'s `summary` and — by the same verbatim-from-source convention — into its new `description`. The convention is doing its job, so this is not a transcription bug, but it is user-visible in both locales' English text. **Fixing the source line is the correct move**, not the i18n: patching only `rules/i18n/` would break verbatim-ness and would be silently undone by any future re-extraction. Not done when found purely because `rules/source/**` was outside that slice's declared file list. One character, one line, then re-sync the two i18n strings. The German line (7054 of *Basisregeln*) reads "Viertel" correctly and needs nothing. **A third, found 2026-09-26 in Phase 2 slice C3:** `ArMDE:4960` (`virtue.simple_student`) reads "who is has not yet taken a degree" — kept verbatim in the shipped summary; fix upstream, then re-sync. **A fourth, German, found 2026-09-27 in slice C5c:** *Basisregeln* at ArMDE:5855, :5861 (line-parallel) read "auf den Zaubersumme" (should be "die"), kept verbatim in the `flaw.corrupted_arts`/`_spells` descriptions. **Two more source defects found since and belonging with it:** `ArMDE:6929` reads "-3penalty" with no space, and the German sources close a German opening quote `„` with an ASCII `"` (see "Recorded conventions" below — that one is corpus-wide house style and is **not** to be swept; it is listed here only so the two are not confused). | an implementation pass (source fix + i18n re-sync) | 2026-09-14 |
| 42 | **DONE 2026-09-29 (X7b-e, D58/D69).** The four originally-swept Flaws plus the nineteen 2026-09-19 additions now compute per a per-entry D58 verdict: new effects `CharacteristicMax` (Uninspirational caps Pre/Com to 0), `PersonalityTraitRange` (Weak Personality tightens the range to ±1), `RequiresPersonalityTraitPair` (Fickle Nature requires a matched +4 pair), `DecrepitudeScaledRollMod` (Lingering Injury's penalty scales with Decrepitude); `Prereq::AgeMin`/`HasCategoryAtMagnitude` gate University Dean (age ≥ 40) and Flawed Powers (a Major Supernatural Virtue held); Servant of the (Land) grants `flaw.prohibition` for free via `GrantsSelection`, Raised from the Dead computes its Warping/Reputation grant from a `years_since_resurrection` parameter. Verified in `rules/core/virtues_flaws.json` and `crates/arm-rules/tests/x7be_row42.rs`. Commits `b49af2e`, `c1b4fcb`, `8c2a252`. Viaticarus's Ability list and Folk Magic's Casting Total stay text by ruling (D69.2-3, the latter needs an Aura term the app does not track — row 49), not oversight. | — | 2026-09-15 |
| 46 | **DONE 2026-09-30.** The arithmetic audit itself completed as the V/F audit's twelve-point check (all 655 entries, including the `creation_effect`/`in_play_effect` ones this row asked for) before this row was last updated; what remained was narrower, per § 9/12e: B01-B11's briefs checked an entry's effects against its passage but only started checking an effect's *sign against what the engine track actually counts* at B12, so B01-B11's spans needed a consumer-tracing re-pass. X7c closed it: Educated (Vernacular)'s Organization Lore is now bound to the entity's own company (a parameter, not a fixed Lore), Physician of Salerno's Reputation kind is player-chosen (Q-55), and six entries (Inventive Genius, Magian Lineage, Strong Faerie Blood, Age Quickly, Baneful Circumstances, Creative Block) had a dropped clause and are now `uncomputed_rule` with that clause stated verbatim (D46/D67). Commit `bd291a9`; tests `crates/arm-rules/tests/x7c_computed_entries.rs`. | — | 2026-09-18 |
| 43 | **DONE 2026-09-30 (D30).** `source.anchor` is mandatory catalogue-wide, not just for Virtues/Flaws: `types.rs::SourceRef::anchor` is a non-optional `String`, and every `rules/core/*.json` file that carries a per-entry `source` block (`virtues_flaws`, `spells`, `abilities`, `equipment`, `aging`, `houses`, `childhoods`, `arts`, `mythic_companion_types`, `spell_mastery_abilities`, `parameter_catalogues` — verified: anchor count equals `source` count in each, both English and `rules/i18n/de/source_anchors.json`) has one. Commits `14a0285`, `b5fc9da`, `8fbe68b`, `d587ffa`, `25f08e4`, `caf84d0`; the rollout is `FULLY_ANCHORED_CATALOGUES` in `crates/arm-rules/tests/rules_source_provenance.rs`. `characteristics.json`/`character_types.json`/`life_stages.json`/`ruleset.json` carry no per-entry `source` field at all, so none was in scope. P8 (the re-sync itself) is unaffected and still open, below. | — | 2026-09-15 |
| 49 | **DONE 2026-09-29 (Group F, D4/D52/D66, commit `a73e9fa`).** Rather than build a condition engine, the over-application was removed at the data layer: `virtue.berserk` and `virtue.ways_of_the_land` dropped their wrongly-`scope: all` effect and are `uncomputed_rule`/text for the conditional clause (Berserk keeps only its unconditional `ability_authorization`); `virtue.cyclic_magic_positive`/`flaw.cyclic_magic_negative` and `virtue.special_circumstances` keep only their unconditional half (Special Circumstances' +3 stays text entirely, confirmed in D70 — "since there is no toggle"). No character is over-applied any longer; the Bjornaer and Mercere book-template totals now match (`crates/arm-rules/tests/book_templates.rs`). | — | 2026-09-23 |
| 51 | **DONE 2026-09-30 (D65/N5, X10/X10b/X10c).** (a) House Tremere's granted `virtue.minor_magical_focus` now carries `"params": {"focus": "certamen"}` in `rules/core/houses.json` (commit `3ec4791`). (b) `ArtScore`/`AbilityScore` gained a banked-XP field, so the Guernicus's "In 12+3 (5)" and the Specialist's "Bows 1 (2)" are both recorded (commit `6918b0c`, X10b). (c) `SpellSelection::within_focus` marks a spell as falling inside a Magical Focus for its Casting Total, with UI in `ffbfb77`/`a2efbc5` (X10c). (d) `weapon.grapple` (`ArMDE:18561`) and `spell.piercing_the_magical_veil` (`ArMDE:1745`, InVi 20, Range/Duration/Target inferred from its named sibling Piercing the Faerie Veil per D68.6, recorded as an inference in `RULES.md`) both ship in `rules/core/equipment.json`/`spells.json` (commit `3ec4791`). | — | 2026-09-23 |
| 52 | **DONE 2026-09-29 (Group F, D66, commit `a73e9fa`).** (a) K5: `EquipmentSlot::equipped: bool` was replaced by `LoadoutState` (`Stowed`/`Carried`/`Wielded`, `types.rs::LoadoutState`) — `Carried` yields a Combat row without contributing Load, so the Knight's alternate great sword is now `"loadout": "carried"` with both its rows intact; the old lossy compromise in `tests/fixtures/book_templates/companion_knight.json` is gone. (b) K3: `Entity.mounted: bool` plus `derived/combat.rs` add a `min(Ride, 3)` mounted Attack/Defense twin per weapon line not flagged `body_attack` (Fist/Kick/Dodge get none, matching the Knight's own template). | — | 2026-09-24 |
| 53 | **DONE (folded into C1, commit `1b8100c`, D14/W2).** `Effect::AbilityAuthorization`'s category/ability entries now each carry a `ParamGate`, so `virtue.wise_one` (`"study"` ∈ {academic, arcane}) and `virtue.custos` (`"study"` ∈ {academic, arcane, martial}) gate every candidate category on the entity's own `study` parameter value — only the chosen value's gate is ever true, so the character is authorized exactly one category, never more than the passage grants. Verified in `rules/core/virtues_flaws.json`. | — | 2026-09-24 |
| 54 | **Nine conversion defects in the Core Rules Markdown, found by transcribing the 24 example characters — to be fixed UPSTREAM in `arm-de-translation`, not here.** *Venue corrected 2026-09-24 on Norbert's ruling: "The version in arm de translation is newer and with a different line count. Changing here does not make sense."* This row twice said the wrong thing before settling — first that the ninth item "waits for the re-sync" (it is forbidden, not deferred), then that the other eight were "safe in-place edits for this repo" (they are throwaway). **`rules/source/` is a copy**: the authoritative original for the **English** text as well as the German is `arm-de-translation/original-english/reviewed/…Core Rules.md`, which is newer and **80 lines longer** (25,803 against our 25,723, measured 2026-09-24 — our count independently confirmed). A hand repair here is replaced wholesale at the next sync and the edit goes with it. So all nine are **recorded here and filed there**, as a todo in `arm-de-translation/docs/todo.md`. The findings themselves are undiminished and survive a re-sync, because each is identified by what is *wrong* rather than by where it sits — and the `ArMDE:1733` case is provable rather than inferred (see (b)). All are Markdown-conversion artefacts, not rules content. **(a) A lost line break:** `ArMDE:1277` reads "Virtues and Flaws: Covenfolk, Warrior, Pessimistic Personality Traits: Brave +3, …" — the Hunter's "Personality Traits:" heading has been swallowed into the Virtues line, running the Flaw *Pessimistic* into the trait *Pessimistic*. **(b) A wrong Art abbreviation:** `ArMDE:1733` prints `Ag 0` in the Criamon's Arts line; there is no Art "Ag", the slot is Aquam, and the line carries 15 slots with only 14 valid abbreviations. **(c) Missing separators:** `ArMDE:1766` (`Giant Blood\* Major Magical Focus`, no comma), `ArMDE:1928` (a period for a comma, and `Music 4+2 (singing) Native Language`), `ArMDE:1954` and `ArMDE:2002` (`Per 0 Pre 0`), `ArMDE:1439` (a period for a comma). **(d) Stray characters:** `ArMDE:2548` (`Puissant Art Perdo)`, unmatched parenthesis), `ArMDE:1704`/`ArMDE:2153` (`Sta + 2`, space inside the value), `ArMDE:1480` (`Etiquette (noble) 3`, parameter before the score against the declared format at `ArMDE:1177`). **(a) IS FORBIDDEN TO FIX, NOT DEFERRED — and this row said "waits for the re-sync" until 2026-09-24, which was wrong.** Repairing it needs a line **inserted**, and `CLAUDE.md` now carries Norbert's invariant: *"We don't change number of lines in the source file… never."* An inserted or removed line renumbers everything below it and invalidates every citation into that file at once — `SourceRef` ranges in `rules/core/` and `rules/i18n/`, `// Source: ArMDE:NNNN` comments, `RULES.md`, and every dated record under `docs/` — and **nothing detects it**, because `rulebook_citations.rs` checks only that a cited range lands on non-blank lines, so after a shift every citation still **passes** while pointing at the wrong text. Silent, total and green: the worst failure shape available. So (a) is never repaired **in this repo** under any schedule — and as of the venue correction above it is not a local defect at all, but an upstream one like the other eight. **(b)-(d) keep their line count**, so they would have been mechanically safe here; they are still not to be done here, for the throwaway reason above. **Owner: the V/F audit session** — it is row 39's class, they have done that shape twice, and they own `arm-de-translation`'s todo file. **Blocked on Norbert confirming in *their* session**, since filing into a different repository raises the same authorization question and a relayed approval is not the same object as a direct one. Several entries also have German counterparts in the line-parallel `rules/source/de/`, whose upstream original needs the same check. Detail: `docs/book-template-conformance.md` § H1, § MAG3, § K4, § D3, and the per-section formatting notes. | (b)-(d) an implementation pass; (a) nothing, permanently | 2026-09-24 |
| 55 | **DONE 2026-10-02 — the per-grant realm override is wired.** `Effect::GrantsSelection`/`Grant::Fixed` now carry an optional `realm`, stamped onto the granted `Selection` by `effective/realm.rs::stamp_realm_override`; the three stated sites (Strong Faerie Blood, Faerie Doctor, Spirit Votary) are authored in `rules/core/*.json`. Detail and test list: `crates/arm-rules/RULES.md` § "D74.4/Row 55". **Still open:** the live Svelte V/F editor does not show a granted row's realm at all (bought rows do); export already does, for free. | a follow-up UI slice | 2026-10-02 |

## Product changes requested by Norbert, 2026-09-13

Raised in-session during the full-codebase audit. These are **product decisions
already taken**, not audit findings and not open questions — they are recorded
here because they arrived mid-audit and are owed as their own pass rather than
folded into a review fix. Numbered separately from the table above, which is for
items still waiting on a decision; nothing below is waiting on one.

**P1–P7 and the `.icon-btn` e2e-coverage regression are DONE** (Phase 2 U-batch,
U1–U5/U7, 2026-09-29): Window → Fullscreen is a real menu action on Windows and
Linux now (macOS keeps the native item); the header's Settings button, the
on-screen document-status chip and the licence logo's old header position are
gone; the window title carries the character's own name (falling back to the
file name, falling back to a new "Untitled" label) with the dirty asterisk in
every case, including a brand-new unnamed character; the logo and the
guided-creation entry both moved into `CharacterBanner`, beside the character
name; table/panel backgrounds are a beige `--panel` token, not white; and
`wizard-flow.e2e.js`'s `tab area at a short window height` describe again
measures a real `.icon-btn` against the WCAG 2.5.8 floor. What a finished item
settled lives in the U-batch's own commit(s); this file's job, per its own
policy above, is to say it is done and move on. Only P8 remains, below.

| # | Item | Kind |
|---|---|---|
| P8 | **Update the rules Markdown sources — there have been substantial changes.** Re-sync `rules/source/en/*.md` and `rules/source/de/*.md` from upstream. **Read the paragraph below before starting: this is the single most disruptive change in this file.** | data |

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
  `virtue`, the two Form-scoped Magic Resistance Flaws' `form`
  (`flaw.flawed_parma_magica`, `flaw.limited_magic_resistance`), and — slice Q4b
  (D9 part 1) — `flaw.deteriorating_power`'s `power`, `flaw.vulnerable_magic`'s
  `condition`, `virtue.greater_immunity`'s `hazard`, and
  `virtue.social_contacts`'s `social_group` joined it on the same standing
  policy. Q4 (D10) had briefly refused a second copy of any of these four
  outright, since none carried a target parameter yet to tell two copies
  apart; Q4b added the parameter, so an old save holding two copies with no
  value now reports `missing_param` on each instead. Phase 2 C5c (D9 part 3,
  D15) added a fourth: the three Corrupted entries' `targets` — a save written
  before this slice holds `flaw.corrupted_abilities`/`_arts`/`_spells` with no
  `targets` param at all (none was ever stored), and now reports
  `missing_param` until the player names which Abilities/Arts/spells the Flaw
  affects. Pinned by
  `c5c_corrupted_and_enchanting.rs::an_older_save_with_no_targets_param_reports_missing_param_once_declared`.
  The Form-scoped pair is worth spelling out, because the save
  looks unchanged while its *output* changes: before the Form existed, each Flaw
  was applied to **all ten Forms at once**, so an old save's Magic Resistance
  numbers were wrong on nine of them and a repeat purchase was indistinguishable
  from the first. The engine now applies each to the one Form its copy names,
  reports `missing_param` until the player names it, and the Flaw contributes
  **nothing** in the meantime — which is the honest state, not a silent revert to
  the old blanket behaviour. There is no correct Form to migrate to: the choice
  was never stored. Pinned by
  `data_integrity.rs::an_mr_flaw_selection_written_before_the_form_parameter_reports_missing_param`.
  Phase 2 B4 (Q-51) added a fifth: `virtue.magical_blood`'s new `bloodline`
  (which bloodline the character has, ArMDE:4359-4372) and `characteristic`
  (which Characteristic the Magic Human clause raises, ArMDE:4367) — a save
  written before B4 holds the Virtue with neither param, and now reports
  `missing_param` for `bloodline`, same standing policy, no special casing.
  `characteristic` is declared `required_if: bloodline=magic_human` (also B4,
  same slice): it stays silent until `bloodline` is filled, and only THEN, if
  the player named Magic Human specifically, does it report `missing_param`
  in turn — never for Magic Animal/Spirit/Thing, which the clause never
  reads. Pinned by
  `b4_parameter_gated_effects.rs::an_older_magical_blood_selection_with_no_params_reports_missing_param`,
  `::magic_animal_bloodline_does_not_require_the_characteristic_param`,
  `::magic_human_bloodline_still_requires_the_characteristic_param`.
  Phase 2 D3 (D56/D62/D64) added a sixth: `flaw.abandoned_apprentice`'s new
  `years_completed` (which year of apprenticeship the character was
  abandoned in, ArMDE:5641-5650) — a save written before D3 holds the Flaw
  with no such param (it did not exist to store), and now reports
  `missing_param` until the player fills it in. Meanwhile the character
  reads as untrained and gets the ordinary companion later-life total, never
  a silent zero (F1) — the engine's one evaluation path, not a special case
  for old saves.
- **`prereq_not_met`** where the eligibility gates bite — a character holding The
  Gift plus Offensive to (Beings) without the Gentle Gift, which `ArMDE:6530` has
  always forbidden and nothing checked. Not hypothetical: it is the exact shape of
  the migration fixture (`V0_2_X_MAGUS_SAVE` in
  `crates/arm-rules/tests/data_integrity.rs`), which is how it came to light.
- **`unknown_param_value`** on a `realm` a player typed by hand, since four items
  were tightened from free text onto `ParameterDomain::Realm`. The typed text is
  handed back verbatim in the finding rather than discarded.
