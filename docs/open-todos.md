# Open to-dos

Items waiting on a decision, a visual check, or a follow-up pass. Kept here so
they survive a session ending. Findings that are merely *implemented* live in
their own findings document; this list is what is still owed.

**Surface this list when a release or a git tag is being prepared** — none of
these should be tagged over silently.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 16 | **Mismatched German quote glyphs are the German corpus's house style, and `rules/source/de/` stays as published.** Measured 2026-09-09 with `rg -a --count-matches` over `rules/source/de/`, after this pass's own edits: **1425** occurrences of `„…"` — opening with `„` (U+201E), closing with an ASCII `"` — against **6** correctly paired `„…“` and **0** genuine inverse pairs. Split: **1345** across all **7** German rulebook files (Basisregeln 309, Mysterienkulte 240, Heckenzauber 230, Societates 190, Wahre Linien 128, Rhein-Tribunal 125, Sphären der Macht — Magie 123) and **80** across 10 files under `translation-tables/` (grundbegriffe 30, tugenden-fehler 12, that directory's own README 12, sphären-mächte 9, orden-tribunale 5, konvent 5, reputationen 4, zauber-nach-form/tiere-kreaturen/magie-regeln 1 each). All **6** correct pairs sit in *Sphären der Macht — Magie*, on five lines: :4495, :4547, :4623, :4681 and :4885, which carries two. That doubled line is also why a naive `“…„` scan reports one "inverse" hit — it matches the gap *between* the two correct pairs, so the genuine inverse count is zero. Raw glyph totals corroborate: 1451 `„`, 6 `“`, 0 `”`, 1544 ASCII `"`. At 1425-to-6 this is the corpus's convention, not a slip, so the earlier reading of `Basisregeln.md:6232` as a one-off defect was wrong. **Decision: the source is not touched** — `rules/source/de/` reproduces the books as printed, and a 1425-site sweep would rewrite the CC-BY-SA rules text over a typographic preference. The policy instead: if an extraction ever carries quoted text past the first sentence, it normalizes the pairing **on the way out**, in the extractor, so `rules/i18n/de/` is clean without the source moving. Nothing shipped reaches such a quote today — every extracted summary stops at the first sentence. The row stays open as the recorded policy, not as work owed. | recorded policy — no source change | 2026-09-07 |
| 17 | **What an older save still reports on open, and why each one is correct.** `e443aaf` recovers what is mechanically recoverable from a character file written before this round of ruleset changes (`8801b03`, `bb305fd`, `2380322`, `b86889c`), and "such a save opens clean" is true of the **format** changes only. Three findings survive on purpose. Nothing is corrupted in any of them — saves store choices, not resolved values, and the engine only reports. (a) **`too_many_selections`** is a genuine rules violation, not a format problem: two bought Puissant Arts plus a House grant really do exceed the ceiling the descriptor states, the engine was simply blind to it before `8801b03`/`bb305fd`, so it survives migration by design and the character has to lose a copy. (b) **`missing_param`** on Folk Magic's `category` and on the three per-power Flaws' `power` (`flaw.slow_power`, `flaw.restricted_power`, `virtue.variable_power`): neither key was ever *stored*, so there is nothing to migrate from, and filling a placeholder would be inventing someone's rules choices — one pick each clears it permanently. (c) **`prereq_not_met`** where `7ea4f5b`'s new eligibility gates bite — a character holding The Gift plus Offensive to (Beings) without the Gentle Gift, which `ArMDE:6530` has always forbidden and nothing checked. That is not a hypothetical: it is the exact shape of `e443aaf`'s own migration fixture (`V0_2_X_MAGUS_SAVE` in `crates/arm-rules/tests/data_integrity.rs`), which is how it came to light. Recorded here so the three are explainable rather than mistaken for regressions; there is no fix owed for any of them. Since then the list of parameters an older save cannot answer has grown, on exactly the same standing policy: Folk Magic's `realm` (B7), Curse of Slander's `taken_as` (B4), Sufi's `taken_as` (B2) and both False Power entries' `virtue` (B9) all report `missing_param` until the player picks. | nothing owed — recorded so an older save's findings are explainable | 2026-09-09 |
| 19 | **The *and* / *or* / comma join in a dual-category descriptor is unmodelled, and *and* has no settled reading.** Half (a) of this row is closed: a Virtue **taken as** one of its categories now exists (`ParameterDomain::Category`, `PointItem::categories_for`) — see "Done since" below. What survives is the join itself. `taken_as` models ***or***, and only *or*: an *or* descriptor states a choice between two readings of one item, and the book says so outright for Sufi (`ArMDE:5083`). An *and* descriptor makes no such statement, and the rulebook never settles what it means — "either route" (the permissive reading the engine takes today, resolving the whole `categories` list, every membership site asking *any* and never *all*) or "both, hence both categories' restrictions apply at once" (the restrictive reading, which no mechanism expresses). Stretching `taken_as` over an *and* would be worse than leaving it open: it hands the player a choice the book does not offer, and under the restrictive reading it silently *drops* half of a restriction meant to bind. The three affected descriptors are `virtue.inoffensive_to_beings` (*General and Hermetic*, `ArMDE:4134`), `flaw.offensive_to_beings` (*Hermetic and General*, `ArMDE:6525`) and `flaw.primogeniture_lineage` (*Story and Hermetic*, `ArMDE:6635`); the reasoning is carried in `crates/arm-rules/RULES.md`. Resolving this needs a rules **decision**, not more code. | decision — a reading of *and* | 2026-09-09 |
| ~~22~~ | **DONE 2026-09-12 — see "Done since" below.** ~~A code comment can cite a Rust source file by a line number that rots silently, and nothing guards it.~~ The decision the row asked for was taken the way it proposed: **a guard**, `crates/arm-rules/tests/source_citations.rs`, plus the conversion of every in-scope site. `docs/` is deliberately outside it. | — | 2026-09-11 |
| ~~23~~ | **DONE 2026-09-11 — see "Done since" below.** ~~Focus Power's 25-point pool is modelled by nothing.~~ The decision the row asked for was taken: a **second budget kind**, with its own effect variant, its own read-out and its own over-spend code. | — | 2026-09-11 |
| ~~24~~ | **DONE 2026-09-12 — see "Done since" below.** ~~Four shipped items type their `realm` parameter as free text, now that a real Realm domain exists.~~ All four are `domain: "realm"`, and the migration question the row asked was answered the data-loss-averse way: **nothing is rewritten**, the unresolvable text is reported back to the player verbatim. | — | 2026-09-12 |
| 29 | **A parameter cap that binds one key, not the whole tuple, is unmodelled.** Necessary (Realm) Aura for (Ability) says "A character may take this Flaw once for any particular Ability" (`ArMDE:6482`), which caps repeats on the **`ability` key alone**. `PointItem::max_per_target` cannot express that: its duplicate key is `(item_ref, params)` — *all* parameters at once — so with E2's realm axis in place, two copies naming the same Ability in different Realms collide in no key and validate clean, which the sentence forbids. E2 found this while tightening the realm domain and deliberately **recorded rather than invented** it, per the standing "implement only what the source supports" rule; nothing regressed, because the second axis existed before E2 too (it was merely free text, so the same two copies differed in a typed word instead of a Realm). Expressing it needs a new `ParameterDef` field — a per-key uniqueness marker — plus a validator beside `validate_duplicate_selections`, and that is engine work with a data shape to settle first: whether the marker names one key ("unique on `ability`") or a subset, and whether an existing save holding two such copies is reported once or twice. The same entry's sibling restriction, "You may not take Student of (Realm) and Puissant Ability for the same Lore" (`ArMDE:5054`), is a cross-**item** constraint over a parameter value and is unmodelled for a different reason: no mechanism relates two different items' parameter values at all. Both are recorded in `crates/arm-rules/RULES.md`. | decision — the data shape for a per-key cap | 2026-09-12 |
| 25 | **`AppError::Ruleset` carries raw English integrity messages across IPC that nothing renders.** `crates/arm-app/src/error.rs:20-23` defines `Ruleset { ruleset_kind: String, errors: Vec<String> }`, and `From<RulesetError>` (`crates/arm-app/src/error.rs:69-81`) fills `errors` with the engine's own `IntegrityError` strings — hardcoded English built with `format!` in `crates/arm-rules/src/ruleset/integrity.rs`, over a hundred `errors.push(format!(...))` sites. The frontend declares the field (`ui/src/lib/types.ts:1765`) and then **reads it nowhere**: `ErrorBanner.svelte` maps `error-<kind>` and special-cases only `export`, and `StartScreen.svelte` maps `error-<kind>` alone, so a ruleset that fails to load shows the generic, fully localized `error-ruleset` sentence in both locales and the payload is dropped on the floor. Two things follow and both belong here. It is **not** the localization defect it looks like on a first read — but the only reason it is not is that the messages are invisible, so the moment anyone surfaces them (which is the obvious way to make a failed rules edit actionable) it becomes one, and that is the decision owed: surface them and localize them, or stop carrying a payload no consumer wants. And it is why the rulebook-citation guard's **string-literal exclusion** — the seven full-basename `.md:NNNN` spellings inside `integrity.rs`'s diagnostics — rests on a premise that is now *verified* rather than assumed: no user ever sees them. Pre-existing and consistent with `Export { missing }` (`crates/arm-app/src/error.rs:28-33`), which *is* rendered; introduced by no slice in this pack. | decision — surface and localize, or drop the payload | 2026-09-11 |
| 26 | **The macOS and Windows menus are unit-tested as data and have never been run.** C3a and C6 prove the menu *model* for all three platforms and, via `installed_menu`, that Tauri really installed it — on Linux. The macOS **Cmd+Q** path through `RunEvent::ExitRequested`, which the mandatory unsaved-changes guard depends on, has never executed on real hardware, and neither has the Windows menu bar. C7's own commit says it plainly: "macOS and Windows are unverified here, as every Phase C slice has said", with muda's `CmdOrCtrl` the only thing standing between the asserted model and a wrong modifier. Worth preserving rather than merely noting: C3a found that `PredefinedMenuItem::quit` on **Windows** would have bypassed the guard outright — muda implements it as `PostQuitMessage(0)`, which ends the message loop instead of raising a close request — so the Windows menu deliberately ships **no** Quit item and offers Window → Close Window (`WM_CLOSE`, guarded) as the way out. That reasoning is recorded in the doc comment on `menu_model` (`crates/arm-app/src/menu.rs`). What is owed is a run on real hardware of each, which a Linux box cannot supply. | a macOS machine and a Windows machine | 2026-09-11 |
| 27 | **The `e2e-testing` build emits a noticeably larger JS bundle than the plain build, from identical frontend sources.** Observed during Phase C: roughly **537 kB** against the plain build's **480 kB**. The plain figure was re-measured in this slice's gate build and is **486.67 kB** (`dist/assets/index-*.js`, 146.36 kB gzipped), so the plain half of the comparison is real; the 537 kB half has not been re-measured. The feature is Rust-side and `#[cfg]`-gated (`crates/arm-app/Cargo.toml` declares `e2e-testing = []`; `ui/e2e/wdio.conf.js` is its only caller, passing `--features e2e-testing` to the same `cargo tauri build --no-bundle`), so it runs the very same `beforeBuildCommand` over the very same `ui/src` and should not reach the frontend bundle at all. Nobody has looked. Most likely a build-configuration difference — a different Vite mode, sourcemap or minification setting on the path wdio takes — rather than real code, but "most likely" is not an answer. What is owed is the measurement and the explanation; if it turns out real code is being included, that is a shipped-binary concern rather than a curiosity. | an explanation | 2026-09-11 |
| 28 | **The logo has not been checked by eye on the light background.** C2 (`b04b473`) shipped light/dark/auto with the **same** asset in both themes — `ui/src/lib/assets/logo.png`, 242×96, one file, referenced once from `App.svelte`. The dark theme was verified in the running app; the light one was not. A logo authored against a dark chrome can lose its edges or halo on a light one, and no test can see it. Needs a human eye in the running app, in both themes, at the header's actual size. | a visual check | 2026-09-11 |
| ~~21~~ | **DONE 2026-09-10 — see "Done since" below.** ~~The e2e suite pays its startup cost 43 times.~~ Every spec file gets its own WebDriver session, so the app is launched and torn down once per file: 43 launches, strictly serial (`maxInstances: 1`, `ui/e2e/wdio.shared.conf.js:23`). Measured on the 2026-09-09 run from `tmp/e2e-logs/`, the blocking `POST /session` alone costs ~30s of a ~40s per-spec cycle (`app-quit-bridge-clean.e2e-0-3.log`: session posted at 21:13:12.020, first test command at 21:13:42.296, whole test body done by 21:13:44.067 — **30.3s of setup for 1.8s of testing**). Whole-suite: 41m43s. Consolidating the 43 spec files into ~10 larger ones removes ~33 startups at ~40s each, roughly **20 minutes**, with no infrastructure change and no loss of coverage — the specs already run serially against a fresh app each, so merging them only changes how many times that app is started. Worth doing independently of whether the 30s itself (see the finding on it) is ever fixed, since the two savings compound. Note `wdio.shared.conf.js:24` currently claims "Specs run serially against one shared app instance", which is wrong — every spec log carries its own session id and ends in `deleteSession()` — and should be corrected in the same pass. | decision on how to group the specs | 2026-09-09 |

## Done since this list was started

- A source-to-source cross-reference names the **symbol**, and a guard keeps it
  honest (old row 22, E3, 2026-09-12). The form is `` `<file>::<symbol>` `` —
  `` `types.rs::Entity::selections` ``,
  `` `effective/xp.rs::spell_mastery_flow_pool` `` — and it was not invented for
  this slice: thirteen comments across Rust and TypeScript already used it, and
  `rules_md_citations.rs`'s own `resolve_target` has read the same shape out of
  `crates/arm-rules/RULES.md` since B11. It cannot collide with a rulebook
  citation (`ArMDE:3899`), because it requires a source extension before its
  `::` and carries no digits at all, so the two guards can never see each
  other's citations. `crates/arm-rules/tests/source_citations.rs` is the guard:
  one sweep rejects a line-number citation of a source file anywhere in
  `crates/*/src`, `crates/*/tests` or `ui/src`, a second resolves every `::`
  citation to a real declaration in the named file (textually — no Rust or
  TypeScript parser, and no new dependency), and four fixture tests pin the
  detectors. Its walking-and-joining layer was extracted into
  `crates/arm-rules/tests/citation_support/mod.rs` and is now shared with
  `rulebook_citations.rs` rather than copied, so a fix to one scanner reaches
  both. **The sweep's value was immediate: every single one of the five in-repo
  Rust line citations in the tree was already stale**, the four in
  `effective/xp.rs` by 52-55 lines each and `derived.rs`'s `Entity::normalize`
  by 1045; on the frontend, `derive.ts` pointed 190 lines away from the comment
  it quoted, and both `App.svelte` citations and the `ParameterPicker.svelte`
  one had drifted 45-85 lines onto unrelated code. The symbol half found two
  more of a different kind — a citation whose symbol had been broken across a
  comment line wrap, leaving a truncated name that resolved to nothing
  (`WizardShell.svelte`, `WizardShell.test.ts`). Twenty-three sites were
  converted in all. Three things are deliberately outside the guard, each
  recorded in `scanned_roots`'s doc comment. **`docs/`**: it holds the bulk of
  the repository's source line citations (several hundred, against the couple of
  dozen in live source) and every one sits in a dated historical record — an
  implementation plan, a review, a findings sheet — where the line number is
  part of the snapshot, so converting them would falsify the record rather than
  repair it, the same call D1c made for `docs/audit-2026-08.md`. Worth naming,
  since it is the one genuinely *living* document in there: `docs/open-todos.md`
  itself carries a handful (`error.rs:20-23`, `types.ts:1765`,
  `wdio.shared.conf.js:23`), which are also snapshots of the day their row was
  raised and were left alone on the same reasoning. **`crates/arm-rules/RULES.md`**:
  not because it is dated — it is the most live document here — but because its
  own `` `validate_caps` (:22) `` convention is already *verified* by
  `rules_md_citations.rs`, and a blanket ban would contradict a working guard
  instead of adding one. Its single citation in a spelling that guard does not
  read (an e2e spec pinned by line) was converted by hand to name the spec's
  test title. **A vendored dependency at a pinned version**
  (`muda-0.19.3/src/accelerator.rs:539-541`, and nine more in `menu.rs`/
  `main.rs`) keeps its line number: those bytes are frozen, the version is
  written into the path so a bump makes the citation visibly rather than
  silently stale, and the file is not in this repository to resolve a symbol
  against. That exemption is checked rather than asserted — the guard parses
  `Cargo.lock` and fails if a cited version is not the one the workspace
  resolves. The rule is now written into `CLAUDE.md` beside the rulebook-citation
  rule it deliberately inverts.
- Every shipped `realm` parameter names one of the four Realms, and an older
  save's typed word is **handed back, not thrown away** (old row 24, E2,
  2026-09-12). `flaw.bound_to_realm` (`ArMDE:5733`),
  `flaw.necessary_realm_aura_for_ability` (`ArMDE:6482`, whose second
  `ability` parameter is untouched), `flaw.realm_stigmatic` (`ArMDE:6656`) and
  `virtue.student_of_realm` (`ArMDE:5054`) each moved from
  `"domain": "text"` to `"domain": "realm"` — a **data-only** change, since
  B7 (`d8abbb4`) had already built every piece it needs, which is exactly what
  "catalogue size is data, never code" is for. Each of the four states the
  closed list in its own entry ("Choose the realm (Divine, Faerie, Infernal, or
  Magic)", "Pick one of the four Realms of Power"), so none of this is inferred.
  The row's real question was what happens to saves holding free text there, and
  the answer is the third option it listed, sharpened: **no migration, and not
  `missing_param` either.** A blank slot is a choice not yet made and keeps
  reporting `missing_param`; a slot holding *"Faerie"* is a choice that no longer
  resolves, and it reports the established **`unknown_param_value`** — the same
  code an unresolvable `item` or `ability` ref has always raised, chosen because
  it is the only parameter finding whose args carry the offending **`value`**.
  That argument is the whole point: the player reads the words they typed, beside
  the picker the domain change gives them, and one pick clears it. The file is
  never touched — saves store choices, not resolved values, and guessing which
  Realm a word meant would be inventing someone's rules choice. `SCHEMA_VERSION`
  does not move and could not: `ParameterDomain` is ruleset shape, not save
  shape. Three things were deliberately **not** done. No `at_most_one_of` on any
  of the four: `ArMDE:3919`'s Divine/Infernal exclusion is stated inside Folk
  Magic's own entry as part of *its* repeat rule, and none of these four repeats
  it, so borrowing it would forbid a build the book permits. Student of (Realm)'s
  "multiple times, for a different realm each time" (`ArMDE:5054`) needed no
  change — the defaults already express it exactly. And Necessary (Realm) Aura's
  "once for any particular Ability" (`ArMDE:6482`) caps one key rather than the
  whole parameter tuple, which `max_per_target` cannot express; it is recorded as
  new row 29 instead of approximated. One fix travelled with the slice, because
  it is the message this change makes players read: `unknown_param_value`
  interpolated the `ParameterDomain` as its raw enum slug, so German said "hat
  unbekannten realm-Wert". The `param-domain-<id>` Fluent family (one key per
  variant, both locales, German from the glossary — `Realm` → `Sphäre`,
  `sphären-mächte.md:16`) now labels it, routed through
  `resolveIssueArgValue`'s `ENUM_ARG_FLUENT_PREFIX` exactly as `category` is, and
  both messages were reworded to put the domain in a trailing parenthesis rather
  than at the head of a compound, since a word arriving from data cannot be
  compounded or inflected reliably. `Record<ParameterDomain, true>` in
  `i18n.test.ts` makes a future unlabelled variant a type error. The **German
  item names** needed rewriting for the same underlying reason and are the most
  visible part of the slice: a typed word inflected however the player typed it,
  but a picked Realm arrives as a fixed label, and two of the four German labels
  are noun phrases with their own article ("Das Göttliche", "Das Infernale"), so
  every template rendered ungrammatically — "Gebunden an Das Göttliche",
  "Student der Das Göttliche", "Das Göttliche-Stigmatisierter", "Notwendige Das
  Göttliche-Aura für …". All four now take the label in apposition after a comma,
  uninflected, exactly as Folk Magic's does, with `name_unfilled` restoring the
  German rulebook's own heading on an unfilled row. English needed no change —
  its realm labels are bare adjectives. The full table is in
  `crates/arm-rules/RULES.md`.
- Focus Power has the point pool the rulebook gives it, as a **second power
  currency** (old row 23, E1, 2026-09-11, `9d1d75f`, the single commit *"Focus
  Power's 25 points are a currency the engine can count"* — the hash was added
  by the next slice, since the row is closed **in** that commit and no hash of a
  commit can exist inside it). The row offered two answers and the
  first was taken: a budget kind of its own, not "player arithmetic plus an
  Initiative". `virtue.focus_power` now carries
  `effects: [{ "type": "focus_points", "amount": 25 }]` (`ArMDE:3899`) and, with
  it, the `creation_effect` classification; copies combine, because "This Virtue
  may be taken more than once, and the points gained may be combined"
  (`ArMDE:3903`). `effective::focus_points_budget` sums the grants and
  `effective::focus_points_used` charges **2 × max_level + penetration** — both of
  the book's own splits of 25 (max level 10 with Penetration 5; max level 5 with
  Penetration 15) come out at exactly 25 — with
  `validation/might.rs::validate_focus_powers` raising `over_focus_points` on
  Review. **B10's exclusion stands and its control test is untouched**:
  `focus_power_funds_no_power_levels_because_its_pool_is_points` still pins that
  Focus Power grants zero *power levels*; what changed is that the currency it
  does grant now exists. **The recommended design was not followed on one point,
  deliberately.** A funding discriminant on `SupernaturalPower` was rejected
  because the two lists hold different quantities: a `SupernaturalPower.level` is
  a level that was *spent* out of the level budget, while Focus Power's is a
  **ceiling** — "the maximum level of effect", and "the character may create any
  effect within the scope of the power, up to the level of the effect"
  (`ArMDE:3899`). So the entity gained a separate `focus_powers:
  Vec<FocusPower { name, max_level, penetration }>`, which names the quantity
  honestly and makes it *structurally* impossible for a focus power to be charged
  against `powers_used` — a stronger guarantee than a conditional, and one that
  leaves the existing power path literally unchanged (a character with no Focus
  Power sees no difference in any number, any issue, or any exported byte; that is
  asserted, not assumed). Both cheap derivations made it in rather than being
  deferred: **Initiative** = Quickness − the maximum magnitude, where the
  magnitude is the book's own general rule, "equal to the level divided by five,
  rounded up" (`ArMDE:9097`) — the repo had no existing helper for it, so one was
  written against that line rather than guessed — and the **Fatigue bands**,
  1 / 2 / 3 levels for ≤25 / 26–50 / 51–75 (`ArMDE:3901`). Above 75 the rulebook
  states nothing, so the read-out says "unstated" (`focus-power-fatigue-unstated`
  → `n/a` / `n/v`) instead of inventing a fourth band. Both are display-only, in
  `derived/focus_power.rs`, surfaced as `DerivedTotals.focus_powers`, shown per
  row in the Supernatural panel and printed in the export's own Focus Powers
  table. **Deliberately not done:** `ArMDE:3905`'s cross-book guidance — basing
  the power on another book's power system, with its different level scales and a
  points-per-level cost the troupe "may want to change" — is recorded in
  `crates/arm-rules/RULES.md` and implemented by nothing; the engine charges the
  core 2-points-per-level rate only. The same line's "may be associated with any
  supernatural realm" is likewise unmodelled: a focus power carries no Realm.
  Additive serde defaults throughout, so **SCHEMA_VERSION stays 17** and no
  migration exists. Both locales shipped in the same commit, with the German terms
  taken from the German rulebook's own mirrored lines (Basisregeln.md:3895-3903 —
  *Fokussierte Macht*, *maximale Effektstufe*, *Erschöpfungsstufe*) and the
  glossary (`magische-qualitaeten.md:29`, `kampf.md:19-20`, `grundbegriffe.md:51`).

- The whole repository's rulebook citations are one mechanically-checked
  spelling, and a guard keeps them there (Phase D, four commits: `1003453`,
  `a551aa3`, `054a5f1`, `de0e868`, 2026-09-11). This is the largest single piece
  of work in the phase and nothing recorded it. Every citation in
  `crates/*/src`, `crates/*/tests`, `crates/arm-rules/RULES.md`, `ui/src` and
  `docs/` now uses the nine-book acronym form (`ArMDE`, `HoH:TL`, `HoH:MC`,
  `HoH:S`, `HM:RE`, `RoP:M`, `RoP:F`, `RoP:D`, `RoP:I`) that `CLAUDE.md`
  defines; D1a alone reported ~1326 sites across 49 files for its two roots, and
  measured after the sweep the five roots carry **3595** acronym-bearing
  citations, of which `RULES.md` holds 1365. `crates/arm-rules/tests/rulebook_citations.rs`
  is what makes the shorthand safe: it resolves every acronym to a real file in
  `rules/source/en/` (and rejects `ArM5`), bounds-checks every cited line against
  that file for "inside the file and non-blank", forbids the bare `:NNNN` form,
  and forbids any full-basename `.md:NNNN` spelling — including a mangled one,
  which was D1a2's whole point after three sites cited "The Divine (Revised).md"
  and "Definitive Edition Core Rules.md", neither a real filename and exactly the
  `docs/audit-2026-08.md` failure mode the guard was meant to make impossible.
  **It earns its keep: the sweep found genuinely wrong citations that were
  invisible until someone opened the rulebook.** The Soak formula cited the core
  book's line 16667, which is blank, instead of `ArMDE:16666` where the SOAK
  TOTAL formula actually is — in `derived.rs`, in `derived/combat.rs` and again
  in `RULES.md`. The Confidence rule cited line 2521, the blank line after the
  `### Confidence` heading, instead of `ArMDE:2524` where the quoted text is.
  (Both wrong numbers are written out in prose here rather than in the acronym
  form, exactly as `docs/audit-2026-08.md` does for its own historical example:
  quoting a deliberately-wrong citation in the canonical shape would make the
  bounds check flag this document.)
  One more was found and corrected in `docs/guided-creation-review-2026-08.md`,
  with a bracketed note preserving the historical record. Four `RULES.md`
  implementation-site line pins shifted by 1–2 lines and were corrected in the
  same pass. **Four deliberate, tested exclusions**, each documented in the
  guard's own module comment: the seven full-basename spellings inside
  `crates/arm-rules/src/ruleset/integrity.rs`'s `errors.push(format!(...))`
  diagnostics (string literals, not comments — see row 25 above, which now
  verifies the premise they rest on); the German source basenames, discovered
  from the directory rather than listed, since German provenance is a different
  and valid convention; `RULES.md`'s own paren-bare implementation-site citations
  (`` `validate_caps` (:22) ``, owned by `rules_md_citations.rs`), recognised by
  their adjacent backticked `.rs` token so the sweep could not corrupt them into
  fake rulebook citations; and the two guard test files themselves, whose doc
  comments illustrate citation shapes with realistic-looking examples.
  **The trap the sweep found and every future pass must know about:** dozens of
  bare `:NNNN` refs in `docs/` were not rulebook citations at all but stale
  *source-code* cross-references — the "`xp.rs` trap", where three bare numbers
  in `effective/xp.rs` were pointing at `types.rs` fields whose lines had moved.
  Blind conversion would have fabricated provenance, attaching a book and a page
  to a number that never referred to one. They were made non-bare instead, by
  naming their file explicitly; the residual risk is row 22 above.

- **A 79-site prose corruption the sweep left behind is repaired (this slice,
  2026-09-11).** D1b's regex treated *every* colon immediately following a
  closing backtick as the start of a bare citation, so a prose colon after a code
  span became `` `ArMDE: `` — "`max_total: 1`ArMDE: `ArMDE:5083` offers…",
  "`PointItem::normalize`ArMDE: an index has no authored order.". Sixty-seven
  sites in `crates/arm-rules/RULES.md`, eleven in
  `crates/arm-rules/tests/data_integrity.rs` and one in
  `crates/arm-app/tests/commands.rs`, all in prose and comments, none affecting
  behaviour. Four variants had no trailing space and were mangled differently:
  `` `::forbid_tainted` `` and three `` `::test_name` `` continuations became
  `` `ArMDE::…` ``, the two `` `:line` `` table-column headers became
  `` `ArMDE:line` ``, and the sentence describing `RULES.md`'s own
  implementation-site convention had its `` `symbol` (:NNN) `` example rewritten
  into `(ArMDE:NNN)`, which describes the wrong thing entirely. All restored
  against the pre-sweep text rather than guessed at.
  **The repair itself missed ten sites on the first pass, and that is the more
  useful half of the lesson:** the verification scan keyed on a trailing space
  after `ArMDE:`, so it shared the original regex's blind spot and reproduced
  the same class of miss — the ten survivors were exactly the ones where the
  corrupted colon was *line-terminal*, its continuation on the next line and no
  space behind it. A pattern written to find the damage of a bad pattern must not
  inherit that pattern's assumptions; the scan that finally came back clean looks
  for an acronym-colon followed by a non-digit **or end of line**, across every
  file type and with `grep -a` so nothing is skipped for looking binary.
  The guard never saw any of it: none of these shapes carries digits after the
  colon, so neither the bare-form nor the bounds check had anything to match —
  a reminder that the guard checks citations, not the prose a citation sweep
  walks through.

- Row 19, half (a) is closed: a Virtue can be **taken as** one of its categories
  (`dee4b50`, with the presentation follow-up `1811edb`, 2026-09-10).
  `ArMDE:5083` makes Sufi's dual descriptor an explicit player choice — "either
  as a Minor Social Status Virtue **or** a Minor Supernatural Virtue" — and the
  engine had no way to say which, so a grog who was a mundane Sufi still counted
  as holding a Supernatural Virtue for every `has_category` rule. The choice is
  stored as a `params` entry under a new `ParameterDomain::Category`, **not** a
  new `Selection` field: `params` already joins every byte-identity site,
  including the duplicate key, which is literally `(item_ref, params)`, so a new
  field would have sat outside it and split identity from duplicate-identity.
  `Category` rather than plain `Enumerated` earns two things — load-time
  integrity requires every declared value to be one of the declaring item's own
  categories, so the parameter cannot drift from the descriptor; and the picker
  labels options through the existing `category-<id>` Fluent keys, where
  `Enumerated` would have found no rules-i18n entry and rendered the raw slug,
  which `CLAUDE.md` forbids outright. **Membership is taken-as aware; browsing,
  catalogue queries and provenance are deliberately not, and forever.** One
  helper, `PointItem::categories_for`, decides, and exactly five membership sites
  call it: the permitted and forbidden category gates
  (`validation/selections.rs`), the category caps (`validation/caps.rs`), Gift
  detection (`effective/gift_confidence.rs`) and open-grant constraints
  (`grant.rs`). `Ruleset::items_by_category`, the `derive.ts` browsing helpers and
  the Markdown export's Type cell keep reading the **whole** list, each with a
  comment saying why: the book indexes a dual-category item under both headings,
  so hiding it from the second hides it from the very category that may make it
  legal — and the export's Type cell is the *descriptor*, i.e. provenance, while
  the choice goes in the name ("Sufi (Social Status)"). B3 then made every
  surface report what the engine judged: the two category issues take their
  argument from `first_in_force` rather than the first-listed tie-break;
  `derive.ts::selectionCategories` is the single frontend mirror of
  `categories_for`, so the grouping and the badge cannot re-derive it
  differently; and the export's `taxonomy_label` routes a `Category` value
  through `category-<id>` instead of falling through to raw text, which had been
  an outright raw-ID-as-label breach. Sufi ships the parameter and, mandatorily,
  `max_total: 1`, enforced for any future taken-as item by
  `ruleset::integrity::validate_taken_as_max_total`. Half (b) — the *and*-joined
  descriptors — stays open as the narrowed row 19 above.

- `flaw.curse_of_slander` ships both its categories, and the player says which
  (old row 13). `2a78d81`, 2026-09-10. `ArMDE:5882` reads *Minor, General or
  Supernatural*, and the book indexes the Flaw under both headings — `ArMDE:5538`
  in `### Supernatural, Minor` and `ArMDE:5575` in `### General, Minor` — so the
  second reading did not exist and the descriptor could not be honoured at all.
  **A correction the pack itself made belongs here: the first cut of this change
  shipped `["general"]` alone and was corrected to both categories.** The shipped
  state in `rules/core/virtues_flaws.json` is
  `categories: ["general", "supernatural"]` plus a `taken_as` parameter offering
  exactly those two values, plus — as a mandatory companion change, not an
  optional one — `max_total: 1`, because `taken_as` sits inside the duplicate key
  and `ArMDE:5083` offers a choice between two **readings of one** Flaw, not two
  Flaws. Adding `supernatural` *flatly* was precisely what old row 13 objected
  to, and `taken_as` is what removes the objection: taken as General, the bearer
  is not a Supernatural Flaw holder for any `has_category` rule. Nothing legal
  became illegal — both categories are on every profile's permitted list and
  neither is on any forbidden list — and no i18n was owed, since both names,
  both summaries and the `category-general` / `category-supernatural` labels
  already existed in both locales. The **and**-joined descriptors are explicitly
  *not* closed by this; they are row 19, half (b).

- False Power's copies name the Supernatural Virtue they taint (old row 11).
  `011ba15`, 2026-09-10. **The row's premise was wrong twice over**, and the
  corrections are the interesting part. It assumed a parameter already existed
  and only needed narrowing — neither `flaw.false_power` nor
  `flaw.false_power_minor` carried one at all. And it assumed the legal targets
  were Supernatural Virtues alone: `ArMDE:6082` names Faerie Blood, Diedne Magic
  and The Gift, which ship `["supernatural"]`, `["hermetic"]` and `["special"]`
  respectively, so a bare `supernatural` would have refused two Virtues the
  source permits outright. Both entries now declare the same `virtue` parameter —
  `domain: "item"`, `require_categories: ["hermetic", "special", "supernatural"]`,
  `require_possessed: true`, `forbid_tainted: true` — making False Power the
  first shipped user of B8's `ParameterDef::require_categories`, which had been
  latent. Two codes, because `ArMDE:6096` states two things.
  `param_target_not_possessed` is raised by
  `validation/selections.rs::validate_possessed_param_targets`, which reads
  `PrereqCtx::present_ids` — bought **plus** granted, the very set `Prereq::Has`
  consults, so a House-granted or warping-filled Supernatural Virtue counts as
  held and the engine keeps one notion of possession rather than two that can
  drift. `param_target_already_claimed` catches a second copy naming a Virtue an
  earlier one already claims ("once for **each** appropriate Supernatural
  Virtue"), which `max_per_target` cannot express: its duplicate key is
  `(item_ref, params)`, and the Major and Minor entries are different ids, so one
  of each naming Second Sight collides in no key. `max_per_target` is now 1 on
  the Minor entry too (it shipped 255; the Major already had the default 1
  alongside `max_total: 1`). `forbid_tainted` is entity-free and so narrows the
  domain in `param_value_resolves` beside `require_categories`, raising the
  existing `unknown_param_value` rather than a code of its own — the descriptor's
  *Tainted* tag **is** Infernal affiliation (`ArMDE:2998-3002`), and inventing a
  code for a value simply not in the domain would split one idea across two
  messages. A `taken_as` reading is deliberately not consulted: possession is by
  id, so a Sufi taken as Social Status stays a legal target. **Deliberately not
  implemented, and recorded in RULES.md rather than approximated:** the Divine
  clause of `ArMDE:6096` — "the troupe **may not allow** it to apply to Virtues
  derived from the Divine" — is explicit troupe discretion, and no
  Divine-affiliation flag exists to decide it on.

- Folk Magic records and displays the Realm its magic is aligned to (old row 12).
  `d8abbb4` recorded it, `ba561d6` put it on the screen, both 2026-09-10.
  `ArMDE:3909` says the choice of (Realm) Lore "also determines which
  supernatural realm his magic is aligned to for the purposes of aura
  modifiers". What is stored is the **Realm**, not the (Realm) Lore Ability: the
  book prints no closed list of Lores, while the four Realms are closed, already
  a Rust enum, and already labelled by the `realm-<id>` Fluent family. So
  `ParameterDomain::Realm` resolves through `Realm::from_id`, the exact mirror of
  `Characteristic::from_id`, and needs no catalogue and no declared `values` —
  a `values` list on it is rejected by the same branch that rejects one on text.
  The exclusion `ArMDE:3919` states in the same breath — align it to the same
  Realm as before or pick a different one, "although a character cannot have
  access to both the Divine and Infernal Realms" — is **data, not a pair of ids
  in Rust**: `ParameterDef::at_most_one_of` is a list of value groups of which at
  most one member may be named across an item's copies,
  `virtue.folk_magic` declares `[["realm.divine", "realm.infernal"]]`, and
  `validation/selections.rs::validate_exclusive_param_values` raises the new
  `exclusive_param_values`. Load-time integrity resolves every group member
  through the very `param_value_resolves` that validation uses, so the two
  notions of "resolves" cannot drift, and rejects a group of fewer than two,
  which would exclude nothing. B7b then closed the surface gap the first commit
  left: a parameter reaches a display name only where the item's own i18n
  template mentions it, and the template said "Folk Magic {category}" alone, so
  the Realm was stored, validated, exported and **invisible on the one surface a
  player reads**. Both locales now name both axes in apposition ("Folk Magic
  {category}, {realm}" / "Volksmagie {category}, {realm}"), and the row
  resolution moved out of `VirtueFlawTab` into `derive.ts::selectionDisplayName`
  — which is why the gap could exist unnoticed, since the local resolver no test
  could reach would have printed `realm.divine` raw. **Out of scope, and
  deliberately not stretched onto `at_most_one_of`:** `ArMDE:3915` (Infernal Lore
  cannot produce Healing) and `ArMDE:3917` (Divine Lore cannot produce the Evil
  Eye) are cross-*parameter* constraints **within** one copy, a different shape
  from `at_most_one_of`, which excludes values *across* copies. RULES.md says so
  rather than approximating them.

- The `gift_categories` overload is split into the two questions it was serving
  (old row 18). `5e02658`, 2026-09-10. `PointItem::index_categories` records the
  headings the book's own index files an entry under *beyond* its membership
  `categories` — provenance, never membership — and
  `EntityTypeProfile::hermetic_flaw_categories` says which categories the
  `ArMDE:2860` guideline counts (`["hermetic"]` on the magus profile and on no
  other). `validation/magus.rs::validate_house` is the **only** reader of
  `index_categories`; Gift detection, the caps, the permitted/forbidden gates,
  grants, `items_by_category` and the Markdown export all stay blind to it, and
  that is guarded rather than merely intended by
  `gift_detection_ignores_index_categories` and
  `index_categories_are_invisible_to_every_membership_surface`. **The row
  miscounted, twice.** It predicted the field would land on the two Beings Flaws;
  **four** shipped items carry it, frozen in `INDEX_CATEGORY_ITEMS` with the
  index line each was verified against — `flaw.offensive_to_beings`
  (`ArMDE:5445`), `flaw.primogeniture_lineage` (`ArMDE:5447`),
  `flaw.unbearable_to_beings` (`ArMDE:5455`) and `virtue.inoffensive_to_beings`
  (`ArMDE:3110`). Inoffensive is a Virtue and can never satisfy a Flaw guideline;
  it carries the heading anyway, because provenance populated only where a
  consumer needs it is not provenance. And the row's list of `gift_categories`
  read sites omitted `validate_gift_policy`, which is part of why the id-versus-
  category cleanup it gestured at is larger than it looked. `hermetic` remains
  **not** a membership category on either Beings Flaw. On that id-versus-category
  split the row raised, B6 recorded an answer rather than a shrug: the two ought
  to agree, and `ArMDE:2858` and `ArMDE:2870` make The Gift a **row** rather than
  a category, while `ArMDE:2880` states a *requirement on* Hermetic items rather
  than an implication from holding one — so Giftedness should be `gift_id` alone,
  with `ArMDE:2880` expressed as `Has(virtue.the_gift)` prerequisites, which B5
  went on to do for four items. Not done there, deliberately: it moves Gift
  detection, the Gift's free Supernatural-Ability slot and `validate_gift_policy`
  for every profile, and needs an audit across every `hermetic` item.
  `flaw.unbearable_to_beings` still gets no `taken_as`, and now structurally:
  integrity requires a `taken_as` value to be a subset of `categories`, and
  `hermetic` is an `index_categories` entry, so the reading is not expressible.

- A type profile's category lists can carry a condition, and a Gifted companion
  may take Hermetic Virtues and Flaws (old row 20). `23c985f`, 2026-09-10.
  `ArMDE:2840` grants the exception the profile could not express — a companion
  "may not take Hermetic Virtues and Flaws, **unless you have The Gift** (this
  would be highly unusual)" — and `ArMDE:2880` says the same from the category
  side. A profile's `permitted_categories` / `forbidden_categories` entry is now a
  `CategoryRule`, `#[serde(untagged)]` over either a bare slug or
  `{ category, when }`, so every existing rules file loads and re-emits
  byte-identically. An entry is in force iff `when` is absent or evaluates to
  `Tri::True`; `False` and `Unknown` both leave it out, `Unknown` resolving in the
  player's favour like the existing `prereq_unevaluated` model. One resolution
  point, `validation/selections.rs::categories_in_force`, feeds both gates and
  evaluates against the `PrereqCtx` `validate()` already builds — no second
  evaluation path. **Both halves of the companion profile carry the condition**,
  and that is not redundancy: permitting is ANY and forbidding is EVERY, so
  relaxing only the forbid would have left every single-category Hermetic item
  refused with `category_not_permitted` — the exact lesson the grog
  `supernatural` removal recorded below. **The row's own recorded fix was
  circular, and the id leaf is the resolution.** The leaf is
  `Has(virtue.the_gift)`, never a category test, because `effective::has_the_gift`
  is category-based on `gift_categories: ["hermetic"]`, so a category condition
  would license itself: you may take a Hermetic item because you are Gifted, and
  you are Gifted because you hold one. `virtue.the_gift` is
  `categories: ["special"]` and Free, in no Gift category and always permitted,
  so satisfying the condition can never require what it licenses;
  `a_companion_does_not_gift_himself_with_a_hermetic_virtue` asserts both the
  structure and that a companion holding only Blatant Gift is still refused.
  **Four sourced prerequisite corrections ship with it**, because a conditional
  category is half a fix if the Gift-bearing items state the wrong requirement:
  `virtue.gentle_gift` moves from `Has(virtue.hermetic_magus)` to
  `Has(virtue.the_gift)` — `ArMDE:3956` is *Major, Hermetic* with no prerequisite
  line, matching its twin Blatant Gift at `ArMDE:5712`, and the magus reading came
  from the comparative at `ArMDE:3957`, which compares rather than restricts;
  `virtue.failed_apprentice` becomes `incompatible_with: ["virtue.the_gift"]` per
  `ArMDE:3845`, an incompatibility rather than a `Nor` because that is how the
  data already states a flat may-not; `virtue.apprentice` gains
  `Has(virtue.the_gift)` per `ArMDE:3420`; and `flaw.suppressed_gift` gains it per
  `ArMDE:6805`. The `when` clauses sit on the hostile-input surface, so
  `validate_category_rule_conditions` walks each through `validate_prereq_refs`:
  a dangling ref fails the load naming the profile, and `PREREQ_MAX_DEPTH` bounds
  the recursion.

- Power Virtues fund the budget their powers are charged against, which is
  **not** what old row 9 prescribed (`6317923`, 2026-09-10). The row asked for a
  per-copy free-text `name` on the five Power Virtues so that a power could be
  pointed at. **The source contradicts that outright**: `ArMDE:4021` (Greater),
  `ArMDE:4283` (Lesser) and `ArMDE:4874` (Ritual) all add copies' levels
  **together** to make several powers *or* one stronger one, and `ArMDE:3903`
  combines Focus Power's points — so a per-copy `name` would encode a 1:1
  correspondence the book denies. **And the pack's own fallback claim, that
  Personal Power at least is one-copy-one-power, is false too**: `ArMDE:4716` does
  call it "a single power", but `ArMDE:4724` then gives it the same
  taken-more-than-once, levels-added-together clause as the rest. The shipped code
  takes that reading — **no** Power Virtue is one-copy-one-power — and uses
  `Entity::powers`, already the named-power registry and already budgeted, as the
  place a power is named. What was missing was the connection: Greater Power 50,
  Lesser Power 25, Personal Power 25 and Ritual Power 25 now carry
  `Effect::PowerLevels`, so a character who buys one has a budget instead of
  reporting `over_power_levels` for every power he enters. Focus Power is
  deliberately excluded and is now row 23 above. `SupernaturalPower` gained a
  `penetration` field, which fixed a real under-count: `ArMDE:4019` spends levels
  one-for-one on Penetration and `ArMDE:4021` does the arithmetic out loud — two
  Greater Powers give 100 levels, spent as a level-60 power with Penetration 0 and
  a level-20 power with Penetration 20, i.e. 60 + 0 + 20 + 20 — while
  `effective::powers_used` counted levels alone, **reported 80 against a real
  100**, and let the player spend 20 levels the book had already spent. Additive
  with a serde default, so SCHEMA_VERSION stayed 16 and no migration exists. The
  three per-power items keep their free-text `power` parameter and add
  `require_power`, validated against `entity.powers[].name` by
  `validation/selections.rs::validate_power_targets` → `power_dangling_target`,
  filed on **Review** — the phase that owns Might and powers, and therefore the
  step that can offer the fix — rather than on the V/F step, which would deadlock
  the wizard; for the same reason the picker keeps a text input rather than a
  select over held powers. **The count of powers stays deliberately
  unconstrained:** `ArMDE:4021` says there should be one power per Virtue by
  default but lets the troupe allow more, which is discretion, not a rule.

- Old row 15 is closed as **no change, and the row was wrong about the count**.
  Measured in `rules/i18n/en/spells.json` and `rules/i18n/de/spells.json`:
  exactly **one** description ends on a dangling colon,
  `spell.the_shadow_of_life_renewed`, in both locales. `spell.mists_of_change`
  does carry a colon mid-description, but prose **resumes immediately after it**
  and the description ends on a full stop, so nothing dangles;
  `spell.visions_of_the_infernal_terrors` has no dangling colon at all. The one
  real case is faithful to the source — the roll table the sentence announces
  lives only in `rules/source/<lang>/` — the only surface a spell description
  ever reaches is the source-row tooltip in
  `ui/src/lib/components/SpellTab.svelte`, and the frontend ships **no Markdown
  or table renderer of any kind**: no `marked`, `markdown-it`, `remark` or
  `micromark` dependency, and not one `{@html}` anywhere in `ui/src`. So there is
  nothing to render a table into and nothing to fix. As the row already said,
  `spell.notes_of_a_delightful_sound` (`ArMDE:14676`) and
  `spell.scent_of_peaceful_slumber` (`ArMDE:15171`) end without punctuation
  because the rulebook does, and must still not be "corrected".

- Keyboard shortcuts are discoverable, and the menu is the only thing that owns
  them (`956eea8`, 2026-09-11). Two long-standing Phase C findings —
  "the shortcuts exist only in the page, where nothing announces them" and
  "accelerators are deliberately omitted, because a muda accelerator and a
  webview `keydown` handler both see the chord and one press fires twice" — were
  the same finding, and declaring the chord on the menu item settles both: one
  owner, and the OS draws "Ctrl+N" beside New because it is the thing that bound
  it. Verified on the running release build before anything was deleted:
  `gtk_accel_groups_activate` answers true (and still does after a menu rebuild),
  the items' `GtkAccelLabel`s report accelerator widths of 37–76px, and
  `gtk_accelerator_get_label` renders the chord in GTK's own language, which is
  why no `.ftl` key was added. `accelerator_for` (`crates/arm-app/src/menu.rs`)
  spells each chord once in muda's cross-platform `CmdOrCtrl` notation:
  Ctrl/Cmd+N/O/S, Shift+S for Save As, Shift+E for Export, and Ctrl/Cmd+, for
  Settings. Export keeps its Shift for a stronger reason than before — a bare
  Ctrl+E on the menu would not merely shadow GTK's readline end-of-line binding
  in a text entry, it would take it away, since the accel group is matched before
  the focused widget sees the key. The rule this established is now in
  `CLAUDE.md`: **a menu action's shortcut is the menu item's accelerator, and is
  declared nowhere else.**

- "A native menu can never be driven or observed by WebDriver" is no longer an
  open item; it is a recorded fact with a seam beside it (`cfe40d6`, 2026-09-11).
  C3a proved the menu was the right *value* and stopped, because a native menu is
  drawn outside the webview. C6 added two `#[cfg(feature = "e2e-testing")]`
  commands, inert in the build users install: `activate_menu_item`, which is the
  **real** dispatch path — `on_menu_event`'s inline body moved into
  `menu::forward_menu_action`, and both the OS handler and the seam call that one
  function, with a test asserting `main.rs` no longer names `MENU_ACTION_EVENT`
  at all — and `installed_menu`, which reads `AppHandle::menu()`, the live object,
  so a menu rebuilt in German after a runtime language switch can be told from one
  never rebuilt. C7 then confirmed the negative half by measurement:
  WebKitWebDriver feeds a synthesized key into WebKit's input pipeline rather than
  delivering it as a window event, so it never reaches the accel group GTK matches
  — every chord is inert under `browser.keys`, and F10 does not open the menubar
  either. Both suites moved off chords accordingly, and the portable run, which
  builds **without** the feature, stopped needing a document action at all and
  reloads instead. What the API can report is less than one would like and is
  stated rather than faked: tauri 2.11.3 exposes only `id()` and `text()` on a
  `PredefinedMenuItem`, so predefined items read back by text and position alone.

- **There is no Gifted-Arts-trained-non-member type gap**, contrary to an earlier
  reading; both shapes are buildable today and were verified against the shipped
  character-type profiles (2026-09-11). `ArMDE:3843-3845`'s **Failed Apprentice**
  is the book's own answer for a Gift lost during apprenticeship — *Minor, Social
  Status*, "You may not have The Gift, but if your Gift was not completely
  destroyed, you may have some Supernatural Abilities", and explicitly no Arts —
  and it ships as `categories: ["social_status"]` with
  `incompatible_with: ["virtue.the_gift"]`, a category both the grog and the
  companion profiles permit. A Gift suppressed **after** the Gauntlet keeps Order
  membership: `ArMDE:6805` says the character "may be a member of the Order of
  Hermes, depending on when his Gift was suppressed", and `flaw.suppressed_gift`
  ships `categories: ["hermetic", "story"]` with `Has(virtue.the_gift)`, both
  categories permitted by the magus profile (and, since B5, reachable by a Gifted
  companion too). Neither needs a new character type.

- The e2e suite runs in **3m26s instead of 41m43s**, about 12× (old row 21).
  Two commits. `c694ea3` made the run parallel: four workers, each with its own
  tauri-driver port pair, its own save/export fixture path (derived from
  `WDIO_WORKER_ID`, which the runner sets before the worker's modules load) and
  its own `XDG_CONFIG_HOME`, which also ended `saga-year.e2e.js` writing the
  developer's real settings file. The hardcoded `setTimeout(resolve, 2000)`
  standing in for driver readiness became a TCP poll that fails loudly naming
  the port. `755a110` then merged the 43 spec files into **10**, removing 33 app
  launches; all 148 tests survive verbatim. The wrong comment at
  `wdio.shared.conf.js:24` and stale serial-execution prose in `e2e/README.md`
  are corrected. `c52b9f0` additionally pinned Mesa to llvmpipe so the run stops
  emitting `libEGL warning: DRI3` per worker.
  **Not fixed, and recorded rather than hidden:** the ~30s `POST /session`
  itself, which lives inside the app's WebKitGTK startup — wdio, tauri-driver,
  WebKitWebDriver and the ruleset load are all excluded by measurement. It is
  now paid four at a time and ten times instead of forty-three, not eliminated.
  `docs/test-performance-2026-09.md` holds the numbers, the disproven theories
  and the one discriminator never run, so the search is not repeated.
  **Lesson worth keeping:** the app has at least three pieces of session-scoped
  state that survive `startCharacter()` — UI language, window size, and
  **validation mode**. Only the third was unpredicted, and only a real run found
  it: five merged tests silently ran under `Silent` and failed.

- GitHub issues #1–#3 are answered and closed, alongside the public **v0.3.0**
  release (old row 8). #3 reported the repeatable-Virtues bug that finding 34
  had already fixed (Improved Characteristics and 24 others); the reply and the
  close were made on GitHub (2026-09-10).

- Primogeniture Lineage's House restriction is enforced (part of old row 13).
  `ArMDE:6636` reads "This Flaw can only be taken by magi of House Verditius, as a
  maga who has left the House is no longer a candidate for Primus", and nothing
  enforced it: the Flaw ships `categories: ["story"]`, which the companion,
  mythic-companion and magus profiles all permit, so all three could take it and
  only the grog was refused — for the unrelated reason that `story` is not on its
  permitted list. It now carries
  `prerequisites: All([IsMagus, House(house.verditius)])` in
  `rules/core/virtues_flaws.json`; no engine code changed. **The `IsMagus`
  conjunct is not redundant**, contrary to what old row 13 assumed:
  `Prereq::House` is tri-state and an *absent* house evaluates to `Unknown`,
  which is the non-blocking `prereq_unevaluated` warning, so a bare House leaf
  would merely have warned a companion — and since `validate_house` returns early
  for a non-magus profile, a hand-edited save could put `house: house.verditius`
  on a companion and be waved through entirely. Six tests in
  `tests/data_integrity.rs` pin the shape and the Verditius / other-House /
  companion / mythic-companion / house-carrying-companion / no-House-yet cases.
  Not modelled, and not claimed to be: the passage's own reasoning about a maga
  who has *left* the House (the engine knows only current membership), and "at
  least three places removed from the Primus". RULES.md carries the citation and
  the reasoning (2026-09-09).
- The grog profile's Supernatural restriction is gone, because it had no source
  (part (b) of old row 20). `ArMDE:2822-2830` is the grog guidelines in full — up to 3
  points of Flaws and an equal number of Virtues, must take one Social Status,
  should not take Story Flaws, not more than one Personality Flaw, may not take
  Major Virtues or Flaws, may not take Hermetic Virtues and Flaws, may not take
  The Gift — and Supernatural appears nowhere in it; `ArMDE:1009` likewise; the
  `### Supernatural` prose at `ArMDE:2958-2962` explains realm association and Warping
  immunity and sets no character-type restriction. **Both halves were removed**,
  because the restriction was encoded twice: `supernatural` left
  `forbidden_categories` *and* joined `permitted_categories`. Permitting is an
  ANY test, so dropping only the forbid would have left every single-category
  Supernatural item refused with `category_not_permitted` — a change that looks
  like a fix and does nothing. `hermetic` stays forbidden; `ArMDE:2829` sources it.
  **Blast radius: 113 shipped items newly clear a grog's category gate** — the
  111 carrying `["supernatural"]` alone (49 Minor Virtues, 26 Major Virtues, 1
  Free Virtue, 27 Minor Flaws, 8 Major Flaws) plus the two *Story, Supernatural*
  Flaws. Of those, 34 stay blocked by `ArMDE:2828`'s Major cap and the two Story ones
  by `ArMDE:2826`'s Story cap, and 4 more are unreachable anyway: `virtue.demonic_might`,
  `virtue.demonic_powers`, `virtue.strong_angelic_heritage` and
  `flaw.false_power_minor` are all Minor but each is `Has(...)` on a **Major**
  item (`virtue.demonic_blood`, `virtue.blood_of_the_nephilim`,
  `flaw.false_power`) that `ArMDE:2828` denies a grog. So what genuinely opened is 72
  Minor items plus one Free Virtue, `virtue.commanding_aura`
  (`ArMDE:3579-3583`) — the only zero-cost entrant, an "inherent benefit of Church
  office" the book puts no type restriction on. `ArMDE:2824`'s budget, `ArMDE:2828`'s Major
  cap and `ArMDE:2830`'s Gift policy are all confirmed still working by tests in
  `tests/data_integrity.rs`; the `forbidding_fires_only_when_every_category_is_forbidden`
  fixture moved from grog/Visions to grog/Suppressed Gift, the only shipped
  pairing that still exercises the ANY/EVERY conjunction for a grog
  (2026-09-09).
- The German rulebook/glossary disagreement over Inoffensive to (Beings) is
  settled, and the glossary keeps it (old row 14). The rulebook heads the entry
  "Für (Wesen) ungefährlich"
  (`rules/source/de/Ars Magica Definitive Edition Basisregeln.md:4133`) while
  `rules/source/de/translation-tables/tugenden-fehler.md:180` gives "Unauffällig
  für (Wesen)"; `b86889c` followed the glossary, which CLAUDE.md makes canonical
  for i18n labels, and that stands. The disagreement is explained rather than
  merely noted: the glossary row carries a per-row `SdM:M` tag — *Sphären der
  Macht: Magie*, EN RoP:M per `grundbegriffe.md:215` — so its wording was
  distilled from a different book's printing of the same Virtue. It is not a
  supplement-only entry (the row sits inside `### Allgemeine Tugenden, Klein`,
  `tugenden-fehler.md:173`) and not a second Virtue (the Core Rules carry it at
  the line-mirrored `ArMDE:4133`). `rules/source/de/`
  is **not** changed — it reproduces the books as printed. Recorded as a prose
  subsection in `rules/source/de/translation-tables/README.md`, beside the
  existing "Tainted" note (2026-09-09).
- Text-param values are trimmed, and a blank one is rejected (old row 10). The
  decision taken was **trim, do not case-fold**: trimming makes
  `ParameterDomain::Text`'s own "any non-empty value is legal" true and costs
  nothing a player meant, so "Wolf Shape " is the same power as "Wolf Shape",
  while capitalisation stays theirs to distinguish — "wolf shape" is still a
  separate target, deliberately, since the rulebook asks for no folding.
  Normalization happens on **load** (`load_entity_migrating`), not in
  `Entity::normalize`, so an opened file is not silently reordered at save time;
  an empty or whitespace-only value now raises `missing_param` naming the box to
  fill. RULES.md carries the reasoning (2026-09-09).
- Dual-category Virtues/Flaws now appear under every category heading in the
  Available list (the rulebook indexes each of them twice); "primary" survives
  only as a tie-break where a surface structurally holds one value.
- `Mythic Companion` became a real category, which stopped a grog taking Devil
  Child.
- The 11.25px tab labels were checked in the running app and read well
  (2026-09-05), so the German labels stay as they are.
- Spirit Votary's +7 Flaw points turned out to be the standard Mythic Companion
  arithmetic — ten points of Flaws at 2:1, minus the 3 that Pagan spends funding
  the 6 points of required Virtues. Core does state it, just not as a number;
  RULES.md carries the derivation now (2026-09-06).
- The total-copies cap the duplicate check was missing is now data (`max_total`
  beside `max_per_target`), closing Inoffensive/Offensive/Unbearable to
  (Beings), Fish out of Water, and Affinity/Puissant Art at the ceilings their
  own descriptors state — finding 35, fixed 2026-09-06.
- Demonic Might and Demonic Powers stop at half your Virtues (old row 5). The
  ratio is data (`max_share_of_kind`), not two hardcoded ids;
  `validate_share_of_kind_cap` warns rather than blocks, and counts granted
  copies because Devil Child grants one — `042acba`, 2026-09-06.
- Slow, Restricted and Variable Power name the power they limit (old row 7). A
  free-text `power` parameter and the default ceiling of 1: one copy per named
  power, no limit across different powers — `2380322`, 2026-09-06. That the typed
  name refers to a power the character actually holds was old row 9, and is
  answered above: `require_power` and `validate_power_targets` shipped in
  `6317923`.
- False Power repeats, and every copy after the first is Minor (old row 4).
  `flaw.false_power` keeps its id and stays Major with `max_total: 1`;
  `flaw.false_power_minor` requires it and repeats, so Major plus two Minors
  costs 3 + 1 + 1 — `7b95ca0`, 2026-09-06. Which Virtue each copy taints was
  old row 11, closed above by `011ba15`.
- Closed lists in the rulebook are closed lists in the picker (old row 6).
  `ParameterDomain::Enumerated` carries its values on the `ParameterDef`, so the
  three (Beings) items became dropdowns and Folk Magic repeats once per
  category with no ceiling written anywhere — `b86889c`, 2026-09-06. Folk
  Magic's second axis was old row 12, closed above by `d8abbb4` / `ba561d6`.
- Twenty truncated Virtue/Flaw summaries got their endings back (old row 15).
  The 240-character cap an earlier session's extraction applied had cut sixteen
  entries mid-word — twelve in German only, four in both locales. Each is the
  full first sentence again, re-extracted from the `source` range the core entry
  already records, with no replacement limit. A guard in `data_integrity.rs` now
  fails on any summary that does not end on sentence punctuation, naming every
  offender and its locale in one run — `6f06679`, 2026-09-07. The same class of
  defect in `spells.json` was the later row 15, closed above as no change.
- The visual pass on the audit-tier work came back green (2026-09-09). Checked
  in the running app: the at-cap Virtue/Flaw row dims with its reason tooltip at
  the current type scale, and the new enumerated dropdowns — Folk Magic's four
  categories and the three (Beings) lists — render and fit in both locales. No
  layout or label changes owed, so `max_total`'s UI mirror and
  `ParameterDomain::Enumerated` are closed end to end.
