# Manual-testing findings — 2026-09-03

Findings from a manual test session through guided creation, the
Arts/Abilities/Spells surfaces, the life-stage panel and the Aging tab. Each
entry records the symptom as observed, the root cause located in the code, the
rules citation where the finding is a rules question, and the decision taken.

Rules citations are `file basename:line` into `rules/source/en/`; the mechanics
behind them live in `crates/arm-rules/RULES.md`, which is the traceability map
and is not restated here.

Status values: **open**, **done**, **closed** (deliberately no change).

Findings are grouped by the work package that implements them. Two numbers are
absent by decision: 13 was withdrawn, 14 was closed after the rules check
contradicted its premise.

The last section, *Third pass — the four carried decisions*, is numbered by
`docs/open-todos.md` **row** rather than by finding: those four were decisions
carried out of earlier findings, not new observations from a test session.

---

## Progress log

Appended as work lands, so a session that dies mid-flight can be resumed from
here. The plan this executes is `/home/norbert/.claude/plans/done-doocument-issues-in-jolly-quiche.md`
(machine-local, not in the repo). Implementation order: WP1 → WP2 → WP3 → WP4 →
WP5 → WP6 → WP7, one finding at a time.

| Date | Work package | What landed | Gate |
|---|---|---|---|
| 2026-09-03 | — | This findings document written; 25 findings recorded, 13 withdrawn, 10 and 14 closed with reasons | n/a (docs only) |
| 2026-09-03 | WP1 | Finding 8 done: eleven `source` refs re-pointed at the core rules, locked by `core_rules_virtues_cite_the_core_rules_file` in `tests/data_integrity.rs`; RULES.md re-cited throughout. Finding 28 opened as a by-product. | `cargo test --workspace`, `clippy --all-targets -D warnings`, `fmt --check` — all green |
| 2026-09-03 | WP1 | Findings 9 and 10 done: `tainted: true` added to `virtue.demonic_blood` **and** `flaw.tragic_life` (found by auditing all 21 tagged descriptors), locked by a new test; finding 10 closed with the Mythic-Companion-is-a-marker rationale recorded in RULES.md. **WP1 complete.** | `cargo test --workspace` (16 suites ok, 0 failed), `clippy`, `fmt --check` — all green |
| 2026-09-03 | WP2 | Findings 6 and 7 done: `categories: Vec<String>` end to end — engine, rules data, export, UI grouping/filtering/badges; legacy `category:` fails loudly; `$category` no longer renders a raw slug. **WP2 complete.** | Full gate green including `cargo tauri build --no-bundle`; e2e not yet run |
| 2026-09-03 | WP3 | Finding 4 done, engine untouched: `issuesForStep` + `phaseSelectedItemIds` let the docked panel show a foreign-phase finding whose `context` was chosen on this step, marked `data-elsewhere` with a dashed bar and the `issue-other-step` sentence naming the owning step (4a/4b); a `data-pending` rail marker with its own glyph and `.sr-only` label carries an open warning forward, gated to steps already reached after measuring 7 warnings over 4 of 11 unopened steps on a fresh magus (4c). | Full gate green including `cargo tauri build --no-bundle`; e2e not run |
| 2026-09-04 | WP3 | Finding 5 done: the ability parameter picker offers the whole catalogue plus an instance field for parameterized abilities, and `ability_bonus_dangling_target` moved to the `abilities` phase, so the wizard runs forward and the rail keeps flagging what is owed. Finding 12 done: the exemplar now leads the requirement ("below Latin 1 (any Dead Language)"), fixed alongside the recommended-ability warning, the scholarly-language finding and the checklist. **WP3 complete.** | Full gate green including `cargo tauri build --no-bundle`; e2e run at package end |
| 2026-09-04 | WP5 | Finding 16 done: `AppStore.#effectiveBasis` + `readSettled()` pin a badge's bought score to the generation its modifiers were computed for, so the Art/Ability/Characteristic badges make one transition per committed edit instead of rendering `newScore + oldBonus` for the length of the debounce. Three new `client` tests. | `npm run test:unit` (1259 tests), `check`, `lint`, `format:check`, `cargo test --workspace` — all green; release build and e2e **not** run, an e2e suite held `target/release` |
| 2026-09-04 | WP7 | Findings 21, 2 and 3 done: the project-wide prose sweep. 43 Fluent keys deleted from both locales (rendering site, EN, DE and the tests that pinned them), one sentence trimmed, one `familiar-powers-note` swept in beyond the brief; every `aria-describedby` that named a deleted node removed with it; `wizardGuidance` and the whole guidance machinery deleted from `derive.ts`; RULES.md re-written where it cited the removed keys. | `npm run test:unit` (1239 tests), `check`, `lint`, `format:check`, `cargo test --workspace`, `clippy --all-targets -D warnings`, `cargo fmt --check` — all green; release build and e2e **not** run, an e2e suite held `target/release` |
| 2026-09-04 | WP4/WP6/WP7 | Findings 11, 15, 19, 20, 22–27, 1 done and committed; statuses corrected across the document. **All 25 findings from the session are resolved** — 22 fixed, 13 withdrawn, 10 and 14 closed. (10 was reopened the same day and fixed; see the row below.) Only finding 28, raised here as a by-product, is still open and needs a decision. | Full gate green per package; e2e green after WP3 (43/43), WP4, WP5, WP6 |
| 2026-09-04 | WP1 correction | **Finding 10 reopened and done.** The 2026-09-03 closure was wrong: `Mythic Companion` is one of the `### <Category>, <Magnitude>` headings the `## List of Virtues` index groups by (:3329), not a `Tainted`-style marker. `mythic_companion` is now a real category on the four Virtues, permitted to the `mythic_companion` profile alone — which stops a grog taking Devil Child, as :2637 requires. `category-mythic_companion` added to both locales; RULES.md's marker note replaced by the corrected one. | `cargo test --workspace`, `clippy --all-targets -D warnings`, `fmt --check`, full UI gate, `cargo tauri build --no-bundle` — all green |
| 2026-09-04 | follow-up | Two real e2e failures from WP6/WP7 fixed: the Living Conditions checklist overflowed its column by 15px (rows inherit `nowrap`, widest label 358px against a 325px track — floor raised to 24rem, labels may wrap), and the new German tab-strip test read `''` for every tab because this WebKitGTK driver's `getText()` returns the empty string for any `overflow: hidden` element, clipped or not. Both locked with unit-level assertions. | `aging`, `aging-crisis`, `tab-area`, `i18n-german` specs green individually; full suite re-run after |
| 2026-09-04 | second pass | Findings 32, 31 done: the four Outer-Mystery Virtues (Heartbeast, The Enigma, Faerie Magic, Verditius Magic) now require their House via `Prereq::House`, including the open House-grant menus that let a Jerbiton or Ex Miscellanea pick around it; the not-bought Ability row's dropped remove button was redistributing its width onto the name column, now reserved in an empty `.remove-slot`. Findings 29, 30 done: granted Reputations are listed rather than added by a button, each row naming its granting Virtue/Flaw; Famous's wildcard stopped flattening into four rows. Finding 34 done: the whole V/F chapter swept for repeatability, 25 items given a data-declared ceiling; finding 35 opened as the mirror defect (a total-copy cap the model has no field for) and recorded in `docs/open-todos.md`. | Not re-run for this row: written when the statuses were reconciled on 2026-09-06, so the gate is assumed from the project's pre-commit rule rather than observed |
| 2026-09-05 | second pass | Finding 33 done: the Aging tab's three columns restored as explicit `.aging-column` wrappers (schedule + living conditions, roll calculator alone, log + accumulated read-outs + longevity ritual) rather than the CSS multi-column approach findings 22/20 had already reverted; default window raised to 1400x900 so the three ~431px tracks fit. `docs/open-todos.md` refreshed (two items closed, three raised); the German tab-strip labels (11.25px) checked in the running app and left as they are. | Not re-run for this row: written during the 2026-09-06 reconciliation, so the gate is assumed from the project's pre-commit rule rather than observed |
| 2026-09-06 | — | Finding 28 closed: Spirit Votary's +7 Flaw points confirmed as the standard Mythic Companion arithmetic (`:2638` ten Flaw points at 2:1, minus the 3 Pagan funds toward the 6 budgeted points its required Virtues cost) — Core does state the number, just not as a digit; only the RULES.md provenance note was wrong. No data change. | n/a (docs only) |
| 2026-09-06 | max_total | Finding 35 done, finding 36 done, finding 37 opened deliberately: `PointItem::max_total` (total copies across all targets, default 255 = no stated ceiling) added alongside `max_per_target`, with a new `too_many_selections` validation error; `validate_duplicate_selections` made grant-aware in the same slice, closing a second hole where a House-granted Puissant Ignem stacked invisibly with a bought one; the four once-only items get `max_total: 1`, Affinity/Puissant Art get `max_total: 2`; the Available list, open grant menus and `ParameterPicker` all consult the new ceiling; `flaw.restricted_power` unblocked (`max_per_target: 255`, matching `flaw.slow_power`); `virtue.folk_magic` left capped, pending an enumerated parameter domain. | `cargo test --workspace`, `clippy --all-targets -D warnings`, `cargo fmt --check`, full UI gate (`test:unit`, `check`, `lint`, `format:check`) and `cargo tauri build --no-bundle` all green per slice; the full 43-spec e2e suite ran 42 passing, 1 failure — a test defect in the new `repeat-virtues.e2e.js` spec (a hardcoded selections index invalid for a magus, whose mandatory traits occupy the first slots) — fixed and re-verified for that spec |
| 2026-09-06 | third pass | To-do 5 done: `PointItem.max_share_of_kind` (a `Share { numerator, denominator }`, rejected at load if the denominator is zero or the numerator exceeds it) plus `validate_share_of_kind_cap`, a **warning** keyed `too_large_share` with `issue-too_large_share` in both locales; Demonic Might and Demonic Powers carry `1/2`. Reads the folded (bought + granted) selection list, unlike `validate_tainted_cap`, because Devil Child grants a free copy (`042acba`). | Not observed in this row's session: the plan's per-slice gate was the orchestrator's to run between slices, and the bookkeeping pass that wrote this row ran only `cargo test --workspace` and the UI `format:check` |
| 2026-09-06 | third pass | To-do 7 done: a free-text `power` parameter on `flaw.slow_power`, `flaw.restricted_power` and `virtue.variable_power`, `max_per_target: 255` removed so the default ceiling of 1 applies per named power; `param-label-power` in both locales, the three `name` templates take `{power}`, and `flaw.restricted_power`'s German name corrected to "Eingeschränkte Kraft". Data only. Save impact: a paramless copy now reports `missing_param` (`2380322`). | As above — not observed in this row's session |
| 2026-09-06 | third pass | To-do 4 done: `flaw.false_power` keeps its id, stays Major and gains `max_total: 1`; new `flaw.false_power_minor` (Minor, `tainted`, prerequisite `flaw.false_power`, repeats freely), so Major + two Minors costs 3 + 1 + 1. EN/DE labels gain the "(Major)"/"(Minor)" suffix; the ids stay asymmetric on purpose, since `validate_magnitude_variant_exclusivity` would force a `_major`/`_minor` pair to be mutually incompatible, which these two must not be (`7b95ca0`). | As above — not observed in this row's session |
| 2026-09-06 | third pass | To-do 6 done, finding 37 closed with it: `ParameterDomain::Enumerated` carrying its `values` on the `ParameterDef`, with load-time integrity on both directions (an enumerated param needs a non-empty duplicate-free list; every other domain must carry none); a value outside the list raises the existing `unknown_param_value`, so open grants and Warping fills are covered free. Folk Magic gains a `category` parameter and repeats once per category; the three (Beings) items become dropdowns; nine value ids ship with EN + DE text and a new coverage test; `ParameterPicker.svelte` gains an `enumerated` branch. Save impact accepted, not migrated (`b86889c`). | As above — not observed in this row's session |
| 2026-09-06 | bookkeeping | The third-pass section above written, the progress log extended, and findings 34, 36 and 37 re-pointed at it; `docs/open-todos.md` rows 4–7 closed and replaced by seven narrowed rows (9–15). Two stale docs defects fixed in `crates/arm-rules/RULES.md`: the "Proportional per-item caps" deferral, which `042acba` had implemented, and two moved source citations (`types.rs:443-446` → `:455-458`, `selections.rs:270` → `:277`). `README.md` checked and left alone — it describes stack, commands and features, none of which these four changes touch. | `cargo test --workspace` green (exit 0) and `cd ui && npm run format:check` green, both observed. Nothing else re-run: this pass changed Markdown only. Note the edited docs sit **outside** prettier's scope — it runs from `ui/`, and `README.md`, `CLAUDE.md`, `PLAN.md` and `crates/arm-rules/RULES.md` are all equally unformatted by it |
| 2026-09-09 | backlog slice 0 | **v0.2.x saves open without a wall of errors** (`e443aaf`). The three preceding rounds each accepted a save impact of their own; together they meant a v0.2.0 player opened their own character into four findings. `fold_legacy_being_params` in `load_entity_migrating` (`crates/arm-rules/src/types.rs`) maps the fifteen `being` labels a v0.2.x player could have typed — six classes × both shipped locales, plus the three German dative forms `:4135` prints — onto the `being.*` ids, case- and whitespace-insensitively; the fold covers the House/Mythic/Warping pick maps as well as bought selections. No `SCHEMA_VERSION` bump and none possible: these were *ruleset* changes, so the fold is value-driven and idempotent, pinned byte-stable across save→load→save. `virtue.alluring_to_beings` and `flaw.magical_being_companion` are excluded because their `being` is still free text. Folk Magic's `category` and the three power items' `power` stay un-migrated on purpose — never stored, nothing to migrate from — and the three items gained `name_unfilled` so the message reads properly instead of "Slow Power ((Power))". A genuine `too_many_selections` deliberately survives. | Not observed in this row's session: this row was written by the 2026-09-09 bookkeeping pass, which ran `cargo test --workspace` and the UI `format:check` only. The slice's own gate is recorded in its commit, not here |
| 2026-09-09 | backlog slice 1 | **To-do 13, three of five items: two Flaws the book indexes under General stop being magus-only** (`7ea4f5b`). `flaw.offensive_to_beings` and `flaw.unbearable_to_beings` shipped `["hermetic"]`, which grog, companion and mythic companion all forbid, so only a magus could take Flaws the book also lists under General (`:5445`/`:5608`, `:5455`/`:5629`); both are now `["general"]`. Adding `hermetic` beside it was the trap — `validate_forbidden_categories` ruled an item out if *any* category was forbidden, and `hermetic` doubles as the engine's Gift-detection category. So forbidding became the mirror of permitting and fires only when **every** category is forbidden: strictly loosening, and it lets a grog be a mundane Sufi (`:5079`) and a companion carry Suppressed Gift (`:6809`). The eligibility the category enforced by accident is now explicit — Unbearable needs The Gift or Magical Air and refuses the Blatant Gift (`:6895`), Offensive needs the Gentle Gift if Gifted and refuses Magical Air (`:6530`), and `virtue.inoffensive_to_beings` gained the gate it never had (`:4139`). One warning is accepted rather than avoided (`missing_hermetic_flaw`, now to-do row 18); the *and*/*or*/comma and "taken as" gaps went to row 19, the unsourced grog `supernatural` and `:2840`'s conditional to row 20. | As above — not observed in this row's session |
| 2026-09-09 | backlog slice 2 | **To-do 10 closed: a typed parameter is the value, not the whitespace around it** (`101e2bb`). `ParameterDomain::Text` documented "any non-empty value is legal" while `param_value_resolves` returned `true` for everything, blank included, so "Wolf Shape " was a different power from "Wolf Shape". Decision: **trim, do not case-fold** — trimming makes the documentation true and costs nothing a player meant; folding is a judgement the rulebook never asks for. Values are normalized at the boundaries (on **load**, in `load_entity_migrating`, plus the three store setters that were not trimming) rather than at the four byte-for-byte comparison sites, and not in `Entity::normalize`, which runs on save and would have reordered a file that was merely opened. A blank value reports `missing_param`, not `unknown_param_value`: there is no value to name, and it makes blank and absent read identically — which is what an old save with an unnamed power wants. Spell parameters come along for the same reason. | As above — not observed in this row's session |
| 2026-09-09 | backlog slice 5 | **Documentation bookkeeping for the three slices above, plus to-dos 14 and 16 — Markdown only, no source, data, i18n or test change.** Row 14 **closed**: the Inoffensive to (Beings) rulebook-vs-glossary conflict is recorded as a prose subsection in `rules/source/de/translation-tables/README.md`, beside the "Tainted" note, with both citations and the per-row `SdM:M` provenance; the two paragraphs in this document that left it "now a to-do row" carry supersession notes. Row 16 **restated and left open** as a recorded policy: re-measured with `rg -a --count-matches` over `rules/source/de/` — **1425** mismatched `„…"` (1345 across all seven rulebooks, 80 across ten files under `translation-tables/`) against **6** correct `„…“` on five lines of *Sphären der Macht — Magie*, and **0** genuine inverse pairs; so the mismatch is the corpus's house style, the source is not touched, and normalization belongs in any future extractor. That README's index was over-counting by five files that are **not on disk and never were** in git history (`kreaturenkraefte`, `goettliche-kraefte`, `infernale-kraefte`, `islamische-begriffe`, `juedische-begriffe`): corrected to the 17 rows that exist — 16 thematic glossary tables plus `uebersetzungsregeln.md`, which is a rules document, not a glossary — and `CLAUDE.md:135`'s "18 thematic tables" corrected to **16** to match. Row 13 narrowed to its two remaining items, with the newly-noticed gap that `flaw.primogeniture_lineage`'s House Verditius restriction (`:6636`) is unenforced; the dangling "rows 9 and **10**" reference fixed; rows 18–20 added for what the three slices recorded as not-fixed; and row 17 records, as behaviour rather than as notes, the three findings an older save still legitimately reports. `README.md`'s "653-entry catalogue" corrected — the catalogue is 655 entries today, and CLAUDE.md's own "catalogue size is data" invariant makes a hardcoded total a drift trap, so it now reads "the Core Rules Virtue/Flaw catalogue" (all 655 entries cite that one book). | `cargo test --workspace` — **observed green**, 16 suites, 1357 passed, 0 failed. `cd ui && npm run format:check` — **observed green** ("All matched files use Prettier code style!"). Nothing else run: this pass changed Markdown only. Note prettier runs from `ui/`, so `docs/`, `rules/`, `README.md`, `CLAUDE.md` and `crates/arm-rules/RULES.md` are **outside its scope** and were not format-checked by it — the wrapping in each was matched by hand |

**Caution for the next session:** `npm run test:e2e` exited **0** with
`41 passed, 2 retries, 2 failed, 43 total`. The exit status is not a verdict —
read the `Spec Files:` summary line and grep for `✖`.

---

## WP1 — Rules data corrections

### 8 — Eleven core-rules Virtues cite sourcebooks they were not taken from

**Status:** done (2026-09-03)

Each of these has a real entry in *Ars Magica - Definitive Edition (Core
Rules).md*, but `rules/core/virtues_flaws.json` cites a Realms of Power book:

| ID | Correct core-rules line | Currently cited |
|---|---|---|
| `virtue.blood_of_the_nephilim` | :3504 | Divine |
| `virtue.curse_throwing` | :3625 | Faerie |
| `virtue.demonic_blood` | :3649 | Infernal |
| `virtue.demonic_might` | :3663 | Infernal |
| `virtue.demonic_powers` | :3667 | Infernal |
| `virtue.devil_child` | :3671 | Infernal |
| `virtue.faerie_doctor` | :3821 | Faerie |
| `virtue.nephilim` | :4594 | Divine |
| `virtue.spirit_votary` | :5006 | RoP: Magic |
| `virtue.spiritual_pact` | :5010 | RoP: Magic |
| `virtue.strong_angelic_heritage` | :5022 | Divine |

Magnitudes and categories otherwise match the core tags. Every replacement range
must be verified against the file before it is written:
`crates/arm-rules/tests/rules_source_provenance.rs:165` checks that a citation
brackets real content in the file it names.

**Resolution.** All eleven now cite the core rules, with end lines derived from
the file (heading through the line before the next `####`, the convention the
neighbouring entries already use). Locked by
`core_rules_virtues_cite_the_core_rules_file` in
`crates/arm-rules/tests/data_integrity.rs`. Worth recording: the sourcebook
citations were not fabricated — these Virtues really are reprinted in the Realms
of Power books — so this was a source-of-truth violation (English core wins), not
a dangling reference, which is why the existing provenance test never caught it.
Curse-Throwing is the Virtue at :3625, not the Supernatural Ability at :7396.
After the fix no file under `rules/core/` cites a Realms of Power book at all.

### 28 — `virtue.spirit_votary`'s +7 Flaw points is cited to a core passage that does not state it

**Status:** closed (2026-09-06) — the value is sound and Core-derived; only its
provenance note was wrong

`rules/core/mythic_companion_types.json` gives Spirit Votary a +7 Flaw-point
budget and cites core `:2741-2764`. Those lines are silent on the number; only
*Realms of Power: Magic* `:5486` states it. `crates/arm-rules/RULES.md:1667`
already notes the discrepancy. Since English core is the source of truth for
values as well as ids, either the citation is wrong or the value is unsourced —
it needs a decision, not a silent fix.

**Resolution.** Neither: the 7 is the standard Mythic Companion arithmetic, so
Core does state it, just not as a number. `:2638` gives every Mythic Companion
ten points of Flaws at two Virtue points each. Spirit Votary's required Virtues
cost 6 budgeted points — Spiritual Pact (Major, 3) plus "one more Major
Supernatural Virtue or three Minor Supernatural Virtues" (3), :2750-2751 — and
its required Flaw, Pagan (Major, 3) at :2756, funds exactly those 6 at 2:1. What
remains of the ten-point allowance is 7, the same way every other type's bonus
falls out. *RoP: Magic* :5486 states the same number independently, so there was
never a discrepancy — only an incomplete note, now rewritten in RULES.md with the
derivation and the second citation. No data change.

### 9 — `virtue.demonic_blood` is missing `tainted: true`

**Status:** done (2026-09-03) — widened by one

The core rules tag it *Major, Supernatural, **Tainted*** (:3649-3650). Without
the flag the Tainted half-of-points cap (`validation/caps.rs`,
`validate_tainted_cap`) under-counts it, so a character can carry more Tainted
Virtue points than the rules allow — a wrong-rules-output defect, not cosmetic.

**Resolution.** An audit of all 21 core-rules descriptor lines carrying the
`Tainted` tag against the catalogue found a **second** entry missing it:
`flaw.tragic_life` (:6855-6856, *Major, Story, Tainted*), which under-counted the
Flaw side of the same cap. Both fixed. `flaw.tainted_with_evil` correctly has no
flag — its descriptor (:6844) is *Minor, General*, so the name is a false friend.
Locked by `core_rules_tainted_virtues_carry_the_tainted_flag` in
`crates/arm-rules/tests/data_integrity.rs`, which samples tagged entries and
keeps `flaw.tainted_with_evil` as a control.

### 10 — Four "Free, Mythic Companion" virtues are stored as `social_status`

**Status:** done (2026-09-04) — reopened after being closed on a wrong reading,
then fixed

Devil Child (:3671), Faerie Doctor (:3821), Nephilim (:4594) and Spirit Votary
(:5006) are tagged *Free, Mythic Companion* in the source.

**The first ruling was wrong.** It read "Mythic Companion" as a marker like
`Tainted`, on the grounds that the chapter's prose defines six categories
(Hermetic :2878, Social Status :2884, Supernatural :2958, Personality :2964,
Story :2980, General :2994) and this is not among them — so the four stayed
`social_status`. That argument mistook a set of *explanatory sections* for the
taxonomy. The taxonomy is the index: `## List of Virtues` (:3004) groups every
entry under a `### <Category>, <Magnitude>` heading, and one of those headings is
**`### Mythic Companion, Free`** (:3329), listing exactly these four
(:3331-3334). None of them appears under `### Social Status, Free`
(:3336-3354), whose seventeen entries are Apprentice, Bard, Covenfolk and the
rest. The descriptors settle it: `*Free, Mythic Companion*` (:3672, :3822,
:4595, :5007) is magnitude-then-category, the same shape as `*Minor, Social
Status*` — while `Tainted` never occupies that slot, appearing only as a third
token *after* a real category (`*Major, Supernatural, Tainted*`, :3650). That
asymmetry is what the earlier argument missed.

**Resolution.** `mythic_companion` is a real category. The four carry it alone —
each is listed under one heading only, so none of them is dual-category — with
`category-mythic_companion` in both locales (DE *Mythischer Gefährte*, the German
index heading at `Basisregeln.md:3329` and `translation-tables/grundbegriffe.md:83`,
the same string `type-mythic_companion` already used).

The move is not cosmetic: it changes who may take them, correctly. Only the
`mythic_companion` profile lists the category in `permitted_categories`, because
":2637 All Mythic Companions take a Free Virtue which specifies their status.
These Virtues are incompatible with each other, and with The Gift, and are **not
available to grogs**", and each descriptor says the Virtue *makes* its bearer a
Mythic Companion (:3673 "can only be taken for a Mythic Companion", :3823, :4596,
:5008). Under the old mapping a **grog** could legally take Devil Child, since
grogs may take Social Status Virtues — a wrong-rules-output defect the mis-ruling
carried with it. A companion could too, acquiring a Mythic Companion's status
without its type; both are now `category_not_permitted`. A magus was already
blocked by the `incompatible_with: virtue.the_gift` its profile requires, and now
fails the category check as well.

No other consumer moved: the shipped category caps name only `personality`,
`story` and `hermetic`; `gift_categories` is `["hermetic"]`; the House grant
constraints in `houses.json` name only `hermetic` (and match on Minor/Major, never
Free); and the mythic-type constraints in `mythic_companion_types.json` name
`supernatural`, `story` and `personality`. The UI needed no change — its category
headings, badges, filter dropdown and the export's Type cell are all built from
the catalogue's own categories through `category-<id>`.

Locked by `the_mythic_companion_virtues_carry_the_mythic_companion_category`,
`only_the_mythic_companion_profile_permits_the_mythic_companion_category` and
`a_grog_may_not_take_a_mythic_companion_virtue` in
`crates/arm-rules/tests/data_integrity.rs`, plus the locale check in
`ui/src/lib/i18n.test.ts`. `crates/arm-rules/RULES.md` records the corrected
reasoning where it previously recorded the wrong one.

---

## WP2 — Multi-category Virtues and Flaws

### 7 — The engine models one category per V/F; the rules give some two

**Status:** done (2026-09-03)

`PointItem.category` is a single `String` (`crates/arm-rules/src/types.rs:1541`),
and every consumer assumes exactly one: category caps, type-profile
permit/forbid lists, grant constraints, gift-category checks,
`items_by_category`, the export Type cell, and the UI's grouping and filtering.
The extraction convention recorded at `crates/arm-rules/RULES.md:252-256` —
"compound labels … take the earliest-listed category" — is where the second
category was dropped.

Four items in the English corpus carry two categories. There are none with
three:

| Name | Source line | Tag | Stored today |
|---|---|---|---|
| Sufi | :5077 | Minor, Social Status, Supernatural | `social_status` |
| Raised from the Dead | :6646 | Major, Story, Supernatural | `story` |
| Suppressed Gift | :6803 | Major, Hermetic, Story | `hermetic` |
| Visions | :6985 | Minor, Story, Supernatural | `story` |

`Tainted` is **not** affected: it is already a separate `tainted: bool`, which is
correct. It is a cross-cutting property — every Tainted item also carries a real
category, it has its own independent cap ("no more than half a character's
Virtues should be tainted", :2998-3002), and it carries a rider unrelated to
categorisation (a Supernatural Ability it grants is an Infernal power). Modelling
it as a category would make it compete for a category slot and give the Tainted
cap and the category caps a shared mechanism the rules do not share.

Consequences of the fix, which are the point of it: Suppressed Gift starts
counting against the Story flaw cap, and Sufi becomes findable as a Supernatural
virtue.

**Resolution.** `PointItem.category: String` became `categories: Vec<String>`
with the descriptor's earliest-listed category first. Membership tests (permit,
forbid, caps, grant constraints, gift categories, `items_by_category`) use the
whole set; display and grouping use the primary, so a bought row still appears
under exactly one heading and the index-addressed remove buttons stay valid. A
legacy singular `category:` key now fails to load with an error naming the
offending item rather than being silently coerced — an old `rules/` directory
beside a new binary is the portable layout's real hazard. No save migration and
no `SCHEMA_VERSION` bump: saves store `Selection { ref, params }`, never a
category.

Two notes on the finding as written above:

- **Suppressed Gift *does* become legal for a companion** — superseded 2026-09-09
  by `7ea4f5b` (the old open-to-dos row 13, since narrowed to its two remaining
  items). This entry previously recorded the opposite ("the
  companion profile also forbids `hermetic`, so `forbidden_category` still fires
  — correctly, it being a Hermetic flaw"). That was wrong twice over. The
  forbidden check fired on *any* forbidden category while the permitted check
  passed on *any* permitted one, so the two contradicted each other: one granted
  the Story route and the other took it straight back. `validate_forbidden_categories`
  is now the true mirror and fires only when **every** category is forbidden. And
  the outcome is what the book says: `:2840` bars a companion from Hermetic
  Virtues and Flaws "unless you have The Gift", and a Suppressed-Gift character
  *does* have The Gift (`:6805` — it does not function, but the social penalties
  remain), which is why `has_the_gift` flags them; `:6809` then describes the Flaw
  as a companion's, "If he replaces a companion, he will become much more
  powerful when the Story Flaw is resolved". A mythic companion is still refused
  it, now by `gift_forbidden` (`:2637`) rather than by category — the same
  outcome, the honest reason. See `crates/arm-rules/RULES.md`, *Two Flaws the book
  indexes under General were magus-only*.
- The Markdown export's Type cell now lists every category, joined through the
  localized separator, each still resolved via `category-<id>` — no raw slug.

A pre-existing raw-slug bug was fixed alongside: `issue-forbidden_category`
rendered its `$category` argument as the bare slug ("belongs to a forbidden
category (supernatural)") because `ENUM_ARG_FLUENT_PREFIX` had no `category`
entry.

### 6 — Visions is categorised Story only

**Status:** done (2026-09-03) — subsumed by 7

Reported from *Definitive Edition* p. 150 (`:6985-6986`, *Minor, Story,
Supernatural*). A symptom of 7, not a standalone data typo; corrected as part of
it.

---

## WP3 — Wizard flow and validation messages

### 5 — Puissant Ability deadlocks the wizard's Next button

**Status:** done (2026-09-03)

Taking Puissant Ability on the Virtues/Flaws step blocks Next until its ability
parameter is filled, but the parameter picker offers only abilities the
character already has (`ui/src/lib/components/ParameterPicker.svelte:223-242`) —
and abilities are bought on a *later* step. There are two blocking errors, not
one: `missing_param`, and `ability_bonus_dangling_target`
(`crates/arm-rules/src/validation/selections.rs:355-367`), an error filed on the
`virtues_flaws` phase reading "targets an Ability the character does not have;
add it first".

**Decision.** The parameter picker offers the whole ability catalogue, as the
`art` domain already does (`ParameterPicker.svelte:105-113`) — Puissant (Ability)
is "choose one Ability" with no requirement that a score already exists.
`ability_bonus_dangling_target` stays an error but is re-filed to the `abilities`
phase, which follows `virtues_flaws` in every profile: the wizard then flows
forward, and the block lands on the step where the work actually is. A pending
item is carried forward on the wizard rail so it is not silently forgotten.

**Implemented as decided.** The `ability` domain now lists the whole catalogue,
keeping the character's own instances of a parameterized ability alongside the
generic entry — and, because the engine expects the instance discriminator
whenever the target is parameterized, the picker grew a text input for it, so the
generic entry is not a `missing_param` dead end of its own.
`ability_bonus_dangling_target` is filed on `CreationPhase::Abilities`
(`validation/selections.rs`), pinned by
`every_shipped_profile_buys_abilities_after_virtues_flaws` in
`crates/arm-rules/tests/core_type_conformance.rs`, and its message now names the
work ("Add … to the character's Abilities") instead of the Virtue. The rail's
carry-forward needed no new mechanism: the existing `data-blocked` marker plus its
`.sr-only` hint already marks any phase holding an error, and a regression test in
`WizardShell.test.ts` now locks that for this case.

### 4 — Characteristic-affecting Virtues give no feedback where the user is

**Status:** done (2026-09-03)

Three cases, one cause. Great Characteristic and Poor Characteristic already
produce precondition errors, and Improved Characteristics' three extra points
already produce a `characteristic_points_unspent` warning — but all of them are
filed under `CreationPhase::Characteristics`
(`crates/arm-rules/src/validation/scores.rs:156-163, 213-241`), while the user is
standing on the Virtues/Flaws step. The docked validation panel filters to the
current phase, so the step where the virtue was taken says nothing.

- **4a** Great Characteristic: no hint that the chosen characteristic must be
  raised on the Characteristics step.
- **4b** Poor Characteristic: mirrored.
- **4c** Improved Characteristics grants 3 points with no warning that they are
  unspent.

**Decision.** Do not move the engine's phase attribution — the offending *value*
really is the characteristic score. Instead the docked panel also admits issues
whose `context` is an item selected on the current step, rendered with a distinct
marker naming the step that owns the fix. `characteristic_points_unspent` has no
`context`, so 4c is served by a "still to do" marker on the wizard rail, which is
also what finding 5 needs.

**Implemented as decided.** The engine is untouched.

- **4a/4b.** `issuesForStep` (`ui/src/lib/derive.ts`) replaces `issuesForPhase` in
  the docked panel: it keeps the step's own findings and admits a finding filed on
  another phase when its `context` names an item that step holds
  (`phaseSelectedItemIds`, which is the entity's `selections` and only on
  `virtues_flaws`). An admitted row carries `data-elsewhere="<phase>"`, a **dashed**
  severity bar instead of a solid one (a shape change, so it survives greyscale) and
  the visible sentence `issue-other-step` — "Resolve on the Characteristics step",
  built from the existing `phase-<slug>` label. That sentence is the part that
  matters: `canAdvance` still keys strictly on the owning phase, so the row reads as
  an error while Next stays enabled, and without it the gate would look broken.
- **4c.** A second, lower-weight rail marker: `data-pending="true"` plus an
  `.sr-only` `wizard-step-pending-label` ("has open warnings"), glyph `*` in the
  warning hue against the blocked marker's bold `!`. It is **gated to steps already
  reached** (`index <= wizardFurthest`, the same set the rail lets you click) and
  suppressed on a blocked step, so a step says one thing and the strongest one.

  The gate is not a guess. `a_fresh_wizard_magus_already_carries_warnings_on_several_unreached_phases`
  (`crates/arm-rules/tests/core_type_conformance.rs`) measures the alternative: a
  magus the wizard has only just created already emits 7 warnings —
  `missing_hermetic_flaw`, `house_unset`, `spell_levels_unspent` and four
  `magus_recommended_ability` — across 4 of its 11 steps, every one of them a step
  never opened. An ungated marker would light over a third of the rail on step one.

One side effect, judged an improvement: `ability_bonus_dangling_target` (finding 5)
also carries an item `context` and is filed on `abilities`, so the V/F step now
shows it too, marked "Resolve on the Abilities step". Next stays enabled exactly as
finding 5 requires; the step simply stops being silent about it.

### 12 — "below Dead Language (e.g. Latin) 1" reads wrong

**Status:** done (2026-09-04) — reads "below Latin 1 (any Dead Language)"; the
recommended-ability warning, the scholarly-language finding and the
minimum-abilities checklist carried the same shape and were fixed with it

`issue-magus_minimum_ability` composes `requirement-exemplar`
(`locales/en/main.ftl:930`) into its `$ability` argument and then appends the
score, so the parenthetical lands between ability and score and the actual
requirement — Latin 1 — is buried. Wording only, in EN and DE. The check itself
stays deliberately wider than the rules' letter (any Dead Language instance
counts), because language instances are free-text player input with no
localization path (`crates/arm-rules/src/life_stage.rs:112-118`).

### 14 — Order-minimum severity

**Status:** closed — no change

Originally logged as "demote the hard ERROR to a warning, since the book calls
these *Recommended* Minimum Abilities". That was a mis-attribution. Two distinct
sets exist and the app already models both correctly:

- **Hard minimum, error:** Parma Magica 1, Magic Theory 1, Latin 1 — ":2437
  Magi must have the following minimum Abilities … Characters with lower scores
  would not be admitted to the Order." An explicit bar.
- **Recommended, warning:** Artes Liberales 1, Latin 4, Magic Theory 3 (:2451,
  "Total Cost: 90 experience points") — already emitted as a warning
  (`crates/arm-rules/src/validation/magus.rs:244-249`).

Only finding 12's wording fix remains.

### 13 — (withdrawn)

Proposed adding "Puissant does not count toward the minimum" to the Order-minimum
message. Cut by the user: do not overcomplicate messages. The underlying
behaviour is correct and unchanged — the minimum is judged on the *bought* score,
because Puissant adds 2 "whenever you use it" (:4816) and meeting an
apprenticeship threshold is not a use, and because the book's own "Total Cost: 90
experience points" (:2461) only adds up for purchased scores.

---

## WP4 — Life stage: Gauntlet age and lab seasons

### 15 — Gauntlet age defaults to the character's age

**Status:** done (2026-09-04) — `apprenticeship.default_gauntlet_age: 25` in
`rules/core/life_stages.json`, clamped to the age, nothing written to the save

Leaving the Gauntlet-age field blank makes the engine read the character's own
age as the Gauntlet age (`crates/arm-rules/src/life_stage.rs:505-509`), i.e. zero
years as a magus. The book's baseline is a magus "25 years old and just out of
apprenticeship" (:1601), and the Darius example runs post-Gauntlet years from 26
(:2486).

**Decision.** The default is data, not a frontend prefill:
`apprenticeship.default_gauntlet_age: 25` in `rules/core/life_stages.json`, with
the absent-value fallback reading `min(default, age)`. A magus younger than 25
still stands at its Gauntlet, so no new error becomes reachable, nothing is
written into the save, and the unsaved-changes flag is untouched. A frontend
write was considered and rejected: `setGauntletAge` no-ops without a life-stage
plan (`ui/src/lib/state.svelte.ts:847-852`), the age is usually unset when the
type is chosen, and a stored 25 against a later-typed age of 22 would raise
`life_stage_gauntlet_age_after_age` — an error the blank field never produced.

### 11 — The lab-seasons warning is incomprehensible at zero post-Gauntlet years

**Status:** done (2026-09-04) — own code and message for the zero-year case, the
general one shortened, and both per-year fields read-only while the span is empty

Observed: "5 lab seasons are charged against 0 year(s) as a magus, more than the
0 those years can be charged for: only three seasons a year cost anything,
because the third already takes the whole 30 points that year — a fourth is
free." Technically correct and useless. At `max == 0` the cause is that the magus
has no years as a magus at all, which the message never says, while the
three-seasons-a-year clause it does say has no bearing on that case.

**Decision.** A distinct code and message for the `max == 0` case naming the real
cause, a plainer wording for the general case, and the lab-seasons input disabled
while post-Gauntlet years are 0. Finding 15 makes this state rare rather than the
default.

---

## WP5 — Effective-score surfaces

### 17 — Puissant Magic Theory does not appear in the Abilities section

**Status:** done (2026-09-04) — bonuses walk the catalogue and the bought
instances as a deduped union, and the tab renders a read-only "not bought" row;
granted floors such as Bjornaer's Heartbeast surface the same way

`ability_bonuses` iterates `entity.ability_scores`
(`crates/arm-rules/src/effective/ability.rs:133-146`), so a Puissant target with
no bought score produces no bonus entry and no row to show it on. Arts fixed
exactly this and recorded why (`effective/art.rs:117-122`, "a Puissant Art
applies even at 0 bought points"); Abilities never got the same treatment, and
the current behaviour is pinned by a test named
`ability_bonuses_still_omit_zero_entries`.

The fix must iterate the union of catalogue entries and bought instances:
`ability_bonus` matches `(ability, parameter)` exactly, so iterating catalogue
definitions alone would silently zero the bonus on a parameterized ability like
Puissant "(Area) Lore: Brandenburg" that *is* bought.

### 16 — Effective badges flicker while editing

**Status:** done

Changing an elemental Form makes the other three Forms' effective badges cycle
through intermediate values before settling. The score is written synchronously
(`ui/src/lib/state.svelte.ts:988-1000`) while the bonus arrives from a 150 ms
debounced IPC round-trip (`:62, 2206-2263`), and the badge renders
`currentScore + staleBonus`. Most visible with Elemental Magic because it is the
only nonlinear cross-Art recompute, but the same shape affects the ability and
characteristic badges.

**Decision.** Hold the badge's previous value while a revalidation is in flight,
so there is one transition per committed edit.

**Landed.** `AppStore` now retains `#effectiveBasis` — the very
`$state.snapshot(entity)` that produced the currently-published
`effective`/`derived`, assigned inside `revalidate`'s existing `#seq` guard so
basis and payload can never drift apart, out-of-order responses included. The
badges read their bought score through `store.readSettled(read)` instead of off
the live entity, which pins both halves of the `bought + modifier` pair to one
generation; the spinners keep reading the live entity, so direct feedback is
unchanged. Applied in `ArtGrid.svelte`, `AbilityTab.svelte` (matched by
ability + parameter, so the pairing survives a row removed above it) and
`CharacteristicPicker.svelte` (where the stale pair also flashed a phantom
"→ +0" badge, since its gate compares effective against bought). Covered by
`ArtGrid.client.test.ts`, `AbilityTab.client.test.ts` and
`CharacteristicPicker.client.test.ts` — `client` tests because the defect is a
mid-flight frame the SSR renderer can never show.

### 18 — Non-takable rows are effectively invisible

**Status:** done (2026-09-04) — the row's contents dim (brightness, not hue),
the recolour is demoted to a redundant cue, the focus ring is deliberately left
unfaded, and the dead `.pick-row:disabled` rule is gone. The blocked reason was
already announced through `use:tooltip`'s `aria-describedby`, not a `title`

Rows that cannot be taken are no longer greyed; the only cue is the "+" turning
from gold (`--accent`) to `--muted`, which is close to the normal ink colour.
Affects Abilities, Spells and Virtues/Flaws — all three use `SourcePicker`
(`ui/src/lib/components/SourcePicker.svelte:109-111`). The rows carry
`aria-disabled` rather than the native `disabled` attribute (deliberately, to
keep the reason tooltip reachable), so the generic `button:disabled { opacity:
.45 }` never applies, and the `.pick-row:disabled` rule in `app.css:969-971` is
dead code. Nothing tests this today.

---

## WP6 — Aging tab

### 23 — The log's × and the take-back button are not the same function

**Status:** done (2026-09-04) — data-corruption defect

Reported as a duplicate control. It is worse than that: the log row's ×
(`ui/src/lib/components/AgingRecordPanel.svelte:174`) calls
`removeAgingLogEntryAt(i)` (`ui/src/lib/state.svelte.ts:1739-1742`), a plain
array filter that deletes the record while leaving the aging points and the
apparent-age increase applied to the character. The take-back button calls
`revertAgingRoll(age)` → `aging::revert_year`
(`crates/arm-rules/src/aging.rs:1560-1583`), which subtracts that entry's points,
steps apparent age back, and then removes the entry.

**Decision.** Route the × through the engine revert for engine-recorded rows,
keeping plain deletion only for legacy free-text rows that carry no age (which
`revert_year` documents as out of its reach). Only then remove the duplicate
control (finding 19).

**Fixed as decided.** `AppStore.removeAgingLogEntryAt` now reads the row at the
clicked index and dispatches on it: a row carrying an `age` goes to
`revertAgingRoll(entry.age)` — the calculator's own path — and a row without one
is dropped as before. The index is the UI's address and the age is the engine's,
so reading the entry is what translates between them; `age` is exactly the key
`revert_year` addresses and `retain`s on, which is why nothing narrower would be
correct. Refusals behave as the calculator's do: a rejected revert leaves both the
character and the row untouched and lands in `agingRejections`, a failed IPC call
in the error banner — the row is never dropped on a revert that did not happen.
The ×'s accessible name follows the row: `aging-revert` ("Take back age N") for a
recorded row, `remove-item` for a hand-written one. Covered by five cases in
`state.svelte.test.ts` (undo, age-not-index addressing across a hand-written row,
the hand-written row itself, rejection, IPC failure) and one in
`AgingRecordPanel.client.test.ts` that presses the real button.

### 19 — One take-back button per recorded year

**Status:** done (2026-09-04)

Fine at three recorded years, unusable at thirty or fifty: the button list
(`AgingRollCalculator.svelte:285-302`) renders one per recorded year, wrapping
across the panel with no ordering cue. Removing arbitrary years is legitimate —
`revert_year` addresses entries by age and is exact rather than recomputed — so
the capability is kept, in the log where the years are already listed.

**Fixed as decided.** The list and its `recorded` derived value are gone from
`AgingRollCalculator.svelte`. Nothing is lost: finding 23 had already re-pointed
the log row's × at `revert_year` through `AppStore.removeAgingLogEntryAt`, under
the same `aging-revert` accessible name, and `revert_year` addresses any recorded
year rather than only the latest — so per-year undo survives with one control per
year instead of two. The e2e specs that clicked `aging-revert-36`
(`aging.e2e.js`, `aging-crisis.e2e.js`) now click `aging-log-remove-0`, which is
that same year's row and the same engine call.

### 22, 20 — Layout is awkward and wastes a screen-third

**Status:** done (2026-09-04)

The panel is an `auto-fit` grid (`ui/src/app.css:1308-1313`) with a full-width
row below it, which reads as a header band plus an orphan row. The large vertical
gap above the "Aging" heading is not an empty element: `align-items: start` means
a short block leaves the rest of its row's height bare, and the tall roll
calculator sets that height.

**Fixed at the cause.** A grid row is as tall as its tallest item, so *any*
pairing of a short block with the roll calculator reproduces the gap somewhere;
capping the calculator or stretching the short block would only move it. So the
aging surface no longer pairs at all: every one of its blocks takes a full-width
row (`.character-details .aging-panel > *`, `.character-details .aging-record > *`
in `app.css`), which is the one arrangement in which the gap cannot arise at any
width or content length. The width is reclaimed *inside* the stages instead of
between them — the Living Conditions checklist flows into as many readable columns
as fit (`.living-conditions-list`), and the four accumulated read-outs are one
stage laid out across its own width (`.aging-state`) rather than four items
scattered around the calculator, which was the other half of the "band of
columns". The surface is therefore shorter than the columns it replaces, not
taller. Reading order is now literally DOM order — one column, nothing moved by
CSS — so eye, caret and screen reader agree. The rest of `.character-details`
(the Details and Personality tabs) keeps the auto-fit columns unchanged; their
blocks are of comparable height and never had this problem.

### 24, 25 — The aging log is below the fold; longevity is below that

**Status:** done (2026-09-04)

Even on a large monitor the log scrolls out of view, so its existence is not
apparent, and the longevity-potion section sits past it. Reading order becomes
schedule → living conditions → roll → log → longevity.

**Fixed as decided.** The log now leads `AgingRecordPanel` instead of trailing it,
so the surface reads schedule → living conditions → roll → log → accumulated
totals → ritual, and the three stages above the log are all shorter than they were
(no per-year revert list, a multi-column conditions checklist). Guarded by
`AgingPanel.test.ts`'s reading-order test, which asserts the order over the real
composition rather than over any one child.

### 26 — The aging-effects list grows one row per year, unbounded

**Status:** done (2026-09-04)

A magus aged forty years renders forty stacked rows, which is what pushes
everything else off screen. A bounded scrollport already exists
(`app.css:1359-1377`); it contains the growth once the block is not last.

**Fixed as decided.** The block is no longer last — the log leads the record (see
24/25) — so its bounded scrollport is what absorbs the growth: a fortieth year
scrolls in place and moves nothing. The bound itself was also unqualified: it read
`.character-details .aging-log-scroll`, which made a correctness guarantee
conditional on a layout class, while `AgingPanel`'s own comment contemplates a
mount without that wrapper. It is now the log's own property.

### 27 — The log is not prefilled from the stress roll

**Status:** done (2026-09-04) — a recorded row reads back total, die, points per
Characteristic and apparent age, localized at render from the stored fields;
nothing generated is written to the save, and the free text stays as a note

`resolve_year` writes `effect: String::new()` deliberately
(`crates/arm-rules/src/aging.rs:1471-1488`, doc `:1396-1402`: "the structured
fields *are* the record, and the prose is the player's to add later") — while
recording `die`, `total`, the resolved per-Characteristic `points` and
`apparent_age_increased`, none of which the log row displays.

**Decision.** Render a localized read-only summary from those structured fields,
exactly as the crisis read-out already does, and leave `effect` as an optional
player note. Generated prose is deliberately *not* written into the save: saves
store choices, not rendered values, and stored prose would freeze one language
into the file.

---

## WP7 — Chrome: tabs and explanatory prose

### 1 — Edit-mode tab titles ellipsize at the default window size

**Status:** done (2026-09-04) — `.tab` gets its own 0.75rem type, tighter side
padding, and a narrower strip gap, sized so the widest set (a magus in German,
146 characters over thirteen tabs) fits the default window. The `title` fallback
stays: at the 900px minimum width German still truncates

The tabs eat too much horizontal space and their titles ellipsize; German is
worse, its labels being longer. `.tab` sets no `font-size` (it inherits 15px) and
pads `0.6rem 0.75rem` (`ui/src/app.css:252-273`). Any change must keep the strip
one line tall with every tab hit-testable (`ui/e2e/specs/tab-area.e2e.js:125-137`)
and must be checked against the German locale, not English.

### 21 — Explanatory prose, project-wide

**Status:** done (2026-09-04), with findings 2 and 3

The Aging panel spends a whole column on explanation instead of content, and the
same pattern recurs across guided creation. The player has the rulebook open; the
app does not need to teach the rules, and the prose pushes real content off
screen.

**Decision.** Remove it — not shrink, collapse, or move to tooltips — in EN and
DE together, along with the tests that pin the sentences. Keep field labels,
validation messages, and short warnings. Findings 2 and 3 are instances:

- **2** — the "Nothing has been recorded on this step yet. It does not block: you
  can continue and come back to it." notice
  (`ui/src/lib/components/WizardShell.svelte:120-127`), which is always mounted
  and reserves a line above every step body.
- **3** — the character-type explanation banner (fixed type / Flaw budget / The
  Gift prose, `ui/src/lib/components/CharacterBanner.svelte:35-60`).

The sweep also covers the per-step `wizard-guidance-*` family, the in-step hints
on the concept, experience and abilities steps, and the Aging/longevity notes.
Two hazards: deleting a hint must also remove the `aria-describedby` that points
at it, and `crates/arm-rules/RULES.md` cites two of the removed keys.

**What was removed.** 43 Fluent keys in both locales, plus one sentence trimmed:

- Wizard chrome: `wizard-step-incomplete-hint`; the whole `wizard-guidance-*`
  family (twelve per-phase lines, the two `-flaw-cap` selectors and
  `wizard-guidance-hermetic-flaw`); `character-type-explainer`,
  `character-type-budget`, `character-type-gift-{required,forbidden,optional}`.
- In-step hints: `saga-year-hint`, `ability-funding-{pool,life_stages}-hint`,
  `life-stage-gauntlet-note`, `life-stage-{gauntlet-age,lab-seasons,spell-levels}-hint`,
  `childhood-apply-hint`, `magus-recommended-hint`, `age-cap-note`,
  `warping-owed-hint`, `familiar-powers-note`.
- Aging/longevity: `living-conditions-hint`, `living-conditions-cumulative-note`,
  `aging-longevity-clamp`, `aging-die-hint`, `aging-outcome-crisis-note`,
  `aging-calculator-note`, `aging-points-note`, `longevity-sterility-note`,
  `crisis-note`, `crisis-die-hint`.
- Trimmed: `life-stage-post-gauntlet-no-years-note` lost its second sentence; the
  first stays because it explains a read-only state and is the `aria-describedby`
  target of both read-only fields.

**Kept deliberately.** The `start-*-hint` copy on the start screen, the
`derived-*-note` "guidance only / vis costs ignored" disclosures,
`familiar-{characteristics,bond}-note`, `crisis-bedridden`,
`crisis-die-unrolled`, `aging-die-capped` and `aging-note-*` — each states what
*this application* does or does not do, or a read-only state a control cannot
state for itself, rather than teaching a rule.

**Code that went with the copy.** `wizardGuidance`, `GUIDANCE_ARGS`,
`guidanceNotes`, `flawCapNotes`, `GuidanceContext`, `GuidanceNote` and
`WizardGuidance` in `ui/src/lib/derive.ts`; `WizardNavigation.phaseIncomplete`
and `AppStore.wizardPhaseIncomplete`; the `.wizard-incomplete-hint`,
`.wizard-guidance`, `.age-cap`, `.gauntlet-note`, `.magus-recommended-hint` and
`.childhood-hint` rules in `app.css`. The `.hidden-reserved` utility is kept
though nothing uses it now: it encodes cross-cutting theme 1's house rule and is
pinned by `app.css.test.ts`.

---

## Second pass — 2026-09-05

Findings from a further session against the Windows build. Originally
collected only, with nothing implemented and no approach decided; see each
entry's own **Status** for where it stands now. Numbering continues the first
pass.

### 29 — Granted Reputations are added by a button instead of being listed

**Status:** done (2026-09-04)

A character whose Virtue or Flaw grants a Reputation gets a button reading
"Add Ecclesiastical Reputation (level 4)". Pressing it twice produces two
identical rows, and the duplicate is only reported afterwards as a validation
error.

The user's objection is the flow, not the error: the grant is already known, so
the Reputations list should simply *contain* the granted rows with their
description empty and waiting to be filled, rather than making the player press
a button to conjure something the rules already gave them. That would also make
the duplicate unreachable rather than merely invalid.

To settle at fix time: whether any V/F can grant the same Reputation kind twice
legitimately (two separate reputations of one kind), and what a save holds for a
granted-but-undescribed reputation.

**Resolution.** The Reputations panel now lists a row per grant directly —
description empty and waiting, no add button, nothing to duplicate (`66d46c6`).
Both open questions are answered: yes, a character can legitimately hold two
Reputations of the same kind and score (Apostate and Senior Clergy both grant
Ecclesiastical 4) — pinned by
`two_reputations_of_one_kind_and_score_roundtrip_as_distinct_rows`
(`crates/arm-rules/src/types.rs`), where only `content` (the free-text
description) tells the two rows apart; and a granted-but-undescribed
Reputation writes nothing to the save at all — the rows are derived from the
character's Virtues/Flaws each time the file is opened, so their presence
never marks the save dirty. Famous's wildcard grant (type left to the player)
was fixed alongside: it used to flatten into one row per Reputation type
(four), where `validate_reputations` had always counted it as the single slot
it is; it now renders as one row with a type `<select>`.

### 30 — A granted Reputation never says which Virtue or Flaw granted it

**Status:** done (2026-09-04)

Asked as "Why Ecclesiastical at all??" — the app offers an Ecclesiastical
Reputation with no indication of where it came from. Three shipped items grant
that kind: `flaw.apostate` (Ecclesiastical 4), `virtue.senior_master`
(Ecclesiastical 4) and `flaw.failed_monk` (Ecclesiastical 2). The level 4 in the
screenshot narrows it to the first two, but the character's full Virtue/Flaw list
is needed to say which — and that is precisely the finding: the player should not
have to work it out. `virtue.hermetic_magus` grants no reputation, so the
character's Social Status is not the source.

Related to 29: if the list were prefilled from the grants, the row could name its
source directly.

**Resolution.** `ReputationGrant` gained a `source: Id` field naming the
granting Virtue/Flaw (`crates/arm-rules/src/effective/reputation_and_caps.rs`),
populated from the selection `for_each_effect!` hands to `reputation_grants`
instead of being dropped on the floor; the app layer
(`crates/arm-app/src/ruleset_io.rs`) passes it through unchanged and each row
now names its source (`66d46c6`). Landed together with 29, since both needed
the same prefilled-row mechanism.

### 31 — Alignment breaks on the not-bought Ability rows

**Status:** done (2026-09-04)

The read-only "not bought" rows added for finding 17 (an Ability carrying a
Puissant bonus or a granted floor but no bought score) do not line up with the
bought rows around them. Reported after the type rebase, so check it against the
current scale: the marker occupies the specialty column
(`.ability-selection .ability-unbought`) and the row omits the specialty input,
the parameter field and the remove button, any of which may be what shifts the
columns.

**Resolution.** The remove button, not the specialty input or parameter
field, was the cause: `.ability-selection li` is a flex line with two
growing items (`.item-name` and the trailing `.specialty`/`.ability-unbought`),
so omitting the button redistributed its width (1.9rem plus a 0.4rem gap)
between them — about 15px that pushed the spinner, badge and marker right on
exactly the not-bought rows. Fixed by reserving the button's width in an
empty, `aria-hidden` `.remove-slot` box, the same trade the effective-score
slot already makes for an absent badge, so both row kinds spend identical
width (`37a3a36`; `ui/src/app.css`, `ui/src/lib/components/AbilityTab.svelte`).
The marker's type size also moved from `--font-small` to `--font-chrome`, the
scale step row markers actually use. Pinned by geometry assertions in
`ui/src/app.css.test.ts` and `AbilityTab.test.ts` — neither existed before.

### 34 — A Virtue the rules let you take repeatedly can only be taken once

**Status:** done (2026-09-05) — reported externally, GitHub issue #3 (Windows,
.exe, v0.2.0); swept catalogue-wide, with two exceptions recorded below and a
mirror defect opened as finding 35

> "I tried to build a character using the 'Improved Characteristics' virtue. This
> is a virtue that can be taken multiple times. However, the system only allows
> it to be taken once."

Correct, and the rules are explicit: *"You have an additional three points to
spend on buying Characteristics… **You may take this Virtue multiple times**."*
(`Ars Magica - Definitive Edition (Core Rules).md:4105`).

**Cause.** Repeatability is not modelled; it is *inferred*. The engine keys
duplicates on `(item_ref, params)` and allows `max_per_target` copies, defaulting
to 1 (`crates/arm-rules/src/validation/selections.rs:121-144`,
`crates/arm-rules/src/types.rs:1679-1700`). The UI mirrors that inference —
`repeatable(item)` is "has a target parameter, or `max_per_target > 1`"
(`ui/src/lib/components/VirtueFlawTab.svelte:79-82`). So an item that repeats
*with a different target each time* works (Immunity, Student of (Realm)), while
one that simply repeats does not. `virtue.improved_characteristics`
(`rules/core/virtues_flaws.json:4511-4519`) has neither a parameter nor a raised
`max_per_target`.

**Scope — nine entries in the core rules say so, and they split two ways.**
Parameterized, working today: Immunity :4015, Student of (Realm) :5054, Weak
Magic Resistance :6348. Unparameterized and therefore blocked: Improved
Characteristics :4105, Demonic Might :3665 ("no more than half the character's
total Virtues"), Mastered Spells :4474, Magic Item :4534 ("add the total levels
together"), Special Circumstances :5000 ("you only gain a +3 bonus even if more
than one set of circumstances applies"), Holy Powers :5030.

Note three of those carry a *limit* in the same sentence, so a blanket "repeat
freely" flag would be wrong for them — the fix needs to say what each item's
ceiling is, and whether stacking multiplies the effect (Improved Characteristics:
yes, +3 each; Special Circumstances: explicitly no).

**Resolution.** A sweep of the whole chapter — three grep passes over every
phrasing, each hit mapped to its id through the item's own `source` range —
found **25** items in this position, not the six first estimated. All now carry
their ceiling as data: the rulebook's stated number where it gives one
(`virtue.quiet_magic` "twice"), otherwise a bound no legal build can reach, since
every copy costs at least one point against a 21-point budget. `Greater Immunity`
(:4015) turned out to be affected too — it repeats "with a different immunity
each time" but records no target, so its copies collided. Stacking was verified
rather than assumed: two Improved Characteristics really do grant 6 points, two
Demonic Powers 40 levels.

**Two things deliberately not encoded:**

- *"No more than half of the character's total Virtues"* (Demonic Might :3665,
  Demonic Powers :3669) is a whole-build ratio, not an absolute cap; the engine
  has only absolute caps, and it is a troupe judgement.
- `flaw.false_power` (:6096) repeats "in each subsequent instance as a Minor Flaw
  rather than a Major one". Magnitude belongs to the catalogue entry, not the
  selection, so every copy would be charged 3 points instead of 3-then-1. Left
  non-repeatable: blocking a legal build beats silently wrong point totals. The
  catalogue already has the clean fix as a precedent — the
  `virtue.amorphous_major` / `virtue.amorphous_minor` split — so a second entry
  plus its EN/DE text would resolve it as data.

**Both were carried to `docs/open-todos.md` (rows 5 and 4) and are now encoded**
— the half-of-Virtues ratio by `042acba`, False Power's Major/Minor split by
`7b95ca0`. See *Third pass — the four carried decisions* below.

### 35 — The mirror defect: items the rules forbid repeating can be repeated

**Status:** done (2026-09-06) — found while fixing 34

`max_per_target` is per *target* by construction, and the duplicate check keys on
`(item, params)` (`crates/arm-rules/src/validation/selections.rs:121-144`). So an
item that carries a target parameter can always be taken once per target, however
firmly the rules forbid repetition. Four entries say so in as many words and are
repeatable anyway:

- Inoffensive to (Beings) :4139 — "You may not take this Virtue more than once"
- Fish out of Water :6132 — "may only be taken once, because taking it more than
  once makes it less serious, rather than more"
- Offensive to (Beings) :6530 — "You may not take this Flaw more than once"
- Unbearable to (Beings) :6897 — "You may not take this Flaw more than once"

Two more are over-permissive by a different amount: `virtue.affinity_art` (:3378)
and `virtue.puissant_art` (:4820) say "twice, for two different Arts", and the
app allows one per Art — up to fifteen.

All six need a cap on **total copies across all targets**, which the model has no
field for. That is the whole finding: a `max_selections` beside `max_per_target`,
or an equivalent, plus the data.

**Resolution.** `PointItem` gained `max_total` — total copies across every
distinct parameter target, default 255 = no stated ceiling — alongside the
existing `max_per_target`, and a new `too_many_selections` validation error
fires when the count exceeds it (`8801b03`). The four once-only items
(Inoffensive to (Beings) `:4139`, Fish out of Water `:6132`, Offensive to
(Beings) `:6530`, Unbearable to (Beings) `:6897`) now carry `max_total: 1`;
`virtue.affinity_art` (`:3378`) and `virtue.puissant_art` (`:4820`) carry
`max_total: 2`, keeping their `max_per_target: 1` too, so "twice, for two
different Arts" enforces both halves of the sentence (`bb305fd`). The
Available list greys an at-cap row with a reason, open grant menus drop an
at-cap item (excluding the menu's own current occupant, so a pick never
vanishes from its own dropdown), and `ParameterPicker` counts granted copies
alongside bought ones (`a2dc130`).

The fix turned out larger than the finding described. `validate_duplicate_selections`
read only bought selections, so a House-granted copy was invisible to the
*per-target* check as well as the new total one: a Flambeau magus granted a
free Puissant Ignem who also bought Puissant Ignem validated clean while
quietly stacking +6 onto Ignem — two copies of the same Virtue for the same
Art, against `:4820`'s "twice, for **two different** Arts" taken literally.
`validate()` now folds bought-plus-granted selections once per run, and both
the per-target and the total check read that folded list (`8801b03`).

One leg is deliberately report-only: a mandatory House choice (e.g.
Flambeau's Puissant Perdo/Ignem) can itself push a character over
`max_total`, since that grant is unconditional — gating it would deadlock
the build with no way forward. The validator reports `too_many_selections`;
the player resolves it by removing a bought copy (`a2dc130`).

Accepted consequence: an existing save holding two bought Puissant Arts plus
a House-granted one now opens showing an error it did not show before. No
data loss — saves store choices (`Selection { ref, params }`), never
resolved legality, and the engine only reports.

### 33 — The Aging tab lost its columns; it should keep three, done properly

**Status:** done (2026-09-05) — corrects findings 22/20/24/25 as delivered

Findings 22 and 20 asked for the awkward layout and its screen-third of white
space to be fixed. What landed instead **removed the multi-column layout**: every
aging block now spans the full width (`.character-details .aging-panel > *` /
`.aging-record > *` at `grid-column: 1 / -1`), one stage per row. That was chosen
because a grid row is as tall as its tallest item, so any pairing of a one-line
read-out with the tall roll calculator leaves the leftover as blank space — one
item per row is the only arrangement where the gap *cannot* arise.

That reasoning was sound but answered the wrong question. The three-column layout
is wanted; the gap is what was not.

Approaches to weigh at fix time — not yet decided:

- **CSS multi-column** (`column-count: 3`, `break-inside: avoid`) for the stages
  above the log, with the log kept full-width below. Blocks flow down one column
  and into the next, so short and tall neighbours pack without leftover rows and
  no block's height is set by another's. Reading order stays DOM order. The cost
  is that column *balance* is the browser's decision, not ours.
- **Explicit grid placement** — assign each block a column deliberately, grouping
  by height (e.g. schedule and living conditions in one column, the roll
  calculator alone in another, the read-outs in a third). Fully deterministic and
  keeps the log's full-width row, but a content change can reintroduce a gap, so
  it needs a geometry test that fails when one appears.
- Whatever is chosen must keep what the last pass bought: the log visible without
  scrolling, its growth bounded, the panel degrading to one column on a narrow
  window, DOM order equal to visual order, and no horizontal clipping (the
  Living Conditions row overflowed by 15px and is now held by a 360px column
  floor plus wrapping).

**Fixed by the second approach — explicit grouping, as COLUMN WRAPPERS.** CSS
multi-column was not weighed again: it is the arrangement finding 20 already
reverted, because content *flows* between columns, so any height change moves the
break (ticking one Living Condition relaid the whole panel and split the record
block). `app.css.test.ts` bans `columns:` outright as a result.

`AgingPanel` now groups its blocks into three `.aging-column` divs, and only those
three — plus the tab heading, which spans them — are items of the
`.character-details` grid:

| Column | Blocks |
|---|---|
| 1 | aging schedule, living conditions |
| 2 | roll calculator (the tall one, alone) |
| 3 | aging log, accumulated read-outs, longevity ritual |

Each wrapper is a flex column with its own `gap` (the grid's gap stops at the
wrapper, and `.character-details` deliberately zeroes the `.detail-section`
margins), so a wrapper's height is its own content's and never a neighbour's. The
gap of findings 22/20 needed *two blocks in one row*; no two aging blocks are
siblings in the grid any more, so it cannot arise. `.aging-record` stays
`display: contents` — both of its blocks belong in column 3.

Five supporting changes, each of which the layout needs:

1. **The default window is 1400x900** (`crates/arm-app/tauri.conf.json`, was
   1100x800). With the measured 360px content floor a 1100px window fits only two
   tracks, so the third wrapper would have wrapped below the taller of the other
   two — reintroducing 24/25. `minWidth` stays 900 and the two- and one-column
   degradation is unchanged. The floor was **not** lowered to force three columns
   at 1100px: it is a content measurement (`app.css:1416-1435`) and
   `app.css.test.ts` asserts it.
2. **Each wrapper renders only when it has content.** All three wrapped blocks are
   `{#if}`-gated on engine read-outs; a starting character too young to owe a roll
   would otherwise get an empty middle track. Covered by `AgingPanel.test.ts`.
3. **The wrappers carry their own vertical rhythm** (`gap: 1rem`, matching the
   grid's row gap).
4. **The log's effect input has a real width floor.** It was `flex: 1;
   min-width: 0`, which is not a floor at all — a flex item that may shrink to
   nothing never triggers a wrap — so in a 431px column the placeholder truncated
   again. It is now `min-width: min(22rem, 100%)`, 22rem being the widest shipped
   placeholder plus the input's chrome as measured in the shipped WebKitGTK
   (German 257.75px + 14.75px = 272.5px). `aging.e2e.js` no longer asserts a bare
   `> 300`: it measures the live placeholder in the input's own font and requires
   the input to be at least that wide, so the assertion states the actual guarantee
   and holds in either language.
5. **The `grid-column: 1 / -1` span is gone.** The `.field { max-width: 26rem }`
   cap stays, re-derived: it is barely binding inside a 431px track but is exactly
   right once the panel degrades to one ~840px track at the 900px minimum window.

Measured in the shipped binary at the new default window: three tracks of 431px
each; the columns are 422px, 136px and 612px tall; growing the log to sixteen rows
takes the record column to 773px and leaves the other two **byte-identical** in
position, width and height. The log block runs 326-404px down a 900px viewport,
well clear of the fold.

### 32 — Heartbeast is offered to a magus of any House

**Status:** done (2026-09-04)

A Bonisagus magus can select `virtue.heartbeast`. The entry carries no
`prerequisites` (`rules/core/virtues_flaws.json:4421-4429`), while the source
says: "You have been initiated into the Outer Mystery of the Heartbeast … and
thus are a member of House Bjornaer. You start with the Ability Heartbeast 1.
Note that all Bjornaer magi gain this Virtue for free at character creation."
(`Ars Magica - Definitive Edition (Core Rules).md:4059-4061`).

So the Virtue is not merely *associated* with Bjornaer — taking it makes the
character Bjornaer, which contradicts a House already chosen as Bonisagus. The
engine has a `Prereq::House` variant, so the mechanism exists; what needs
deciding is whether the correct model is a House prerequisite, a
mutual-exclusion with the other Houses' free Virtues, or something that changes
the House. Worth checking the other Houses' Outer Mystery Virtues for the same
gap while fixing it.

**Resolution.** Fixed as a House prerequisite: `Prereq::House` already
existed in the engine, unused by any shipped data. The other Houses' Outer
Mystery Virtues did carry the same gap, as flagged: The Enigma (`:3759-3761`,
Criamon), Faerie Magic (`:3825-3827`, Merinita) and Verditius Magic
(`:5215-5217`, Verditius) are, with Heartbeast, the only four core-rules V/F
descriptors carrying the "and thus are a member of House X" clause. All four
now carry `"prerequisites": { "kind": "house", "value": "house.<x>" }`
(`rules/core/virtues_flaws.json`). On a bought row, a magus of another House
gets `prereq_not_met`; a magus with no House chosen yet gets the
`prereq_unevaluated` warning rather than a block, since an absent House is
genuinely undecided, not a failure. The House's own granted row is never
prerequisite-checked, so Bjornaer's free Heartbeast stays legal. The open
House-grant menus were the real hole: Jerbiton's `jerbiton_minor_virtue` and
Ex Miscellanea's `ex_misc_minor_virtue` both offer any Minor Hermetic Virtue,
and a grant pick is never prerequisite-checked, so a Jerbiton could have
acquired Heartbeast with nothing said at all — `grant.rs::open_pick_satisfies`
and `ui/src/lib/derive.ts::eligibleForConstraint`'s `houseOnlyValue` now both
filter House-prerequisite items out of those menus (`8c1ee42`).

### 36 — `flaw.restricted_power` was capped at one, not one per power

**Status:** done (2026-09-06)

An escapee from finding 34's sweep, not part of 35. `:6689` "This Flaw may be
taken once for each power the character possesses" — but the entry carried
neither a parameter nor a raised `max_per_target`, so the app allowed exactly
one copy.

**Resolution.** `max_per_target: 255` added
(`rules/core/virtues_flaws.json:2399`), the same "no stated ceiling" treatment
`flaw.slow_power` already had (`98a2a0d`). Residual gap, written down rather
than papered over: the rule caps *per power*, and no selection records which
power a copy names, so nothing stops two copies naming the same one — the
same recorded limitation as `flaw.slow_power` (`:6761`) and
`virtue.variable_power` (`:5205`); see the "Repeat rules the data model cannot
express" section of `crates/arm-rules/RULES.md`.

**That residual gap is closed** — carried as `docs/open-todos.md` row 7 and
fixed by `2380322`, which gave all three items a free-text `power` parameter.
See *Third pass — the four carried decisions* below for what the parameter does
and does not guarantee.

### 37 — `virtue.folk_magic` still caps at one copy

**Status:** done (2026-09-06) — opened deliberately, closed by the enumerated
parameter domain it asked for

Also an escapee from finding 34's sweep. `:3919` "You may pick this Virtue
more than once, to acquire expertise in a different category of spells" — the
entry carries no parameter, so it stays at the default `max_per_target: 1`.

`:3909` restricts the category to "one of the following four options",
enumerated at `:3911-3917` (Abjuration, Divination, Healing, Evil Eye). A
`max_per_target: 4` ceiling was considered and rejected (`98a2a0d`): it
hardcodes a count the list already implies, and would still allow two copies
naming the same category.

**Decision.** The right fix is a data-declared enumerated parameter domain: a
category parameter whose legal values are exactly the four listed spell
categories, after which `max_per_target: 1` on that parameter gives
distinctness per category, the total-of-four falls out of the list's length
instead of being stated separately, and a future supplement adding a fifth
category needs no engine change. `ParameterDomain`
(`crates/arm-rules/src/types.rs:424-447`) is a fixed enum over existing
registries (Ability, Art, Technique, Form, Characteristic, Item, Text) with no
such variant today; building one is out of scope for this fix. See the
deferral recorded in `crates/arm-rules/RULES.md` ("Enumerated parameter
domain, not free text").

**Resolution.** Built, as `docs/open-todos.md` row 6 and commit `b86889c` —
`ParameterDomain::Enumerated` with the list on the `ParameterDef` itself. Folk
Magic now repeats once per category with no number written anywhere. The full
entry is *To-do 6* in the third pass below.

---

## Third pass — the four carried decisions, 2026-09-06

These four are **not** findings from a manual-testing session, and they are
numbered by their `docs/open-todos.md` **row** rather than as new findings,
because nothing new was observed. Each was a deferral already written down
inside an earlier finding, parked as `waiting on: decision`, and then authorised:
finding 34 left the Demonic half-of-Virtues ratio (row 5) and False Power's
per-copy magnitude change (row 4) deliberately unencoded, finding 36 recorded the
missing per-power target (row 7), and finding 37 asked for the enumerated
parameter domain by name (row 6). All four rows are now closed; what each one
leaves behind is listed here and carried back into `docs/open-todos.md` in a
narrowed form.

Rules citations are into `Ars Magica - Definitive Edition (Core Rules).md` and
were re-read in the file while this section was written, not recalled.

### To-do 5 — Demonic Might and Demonic Powers cap at half the character's Virtues

**Status:** done (2026-09-06) — `042acba`; deferred by finding 34

> "You may take this Virtue more than once, though it can account for no more
> than half of the character's total Virtues." (`:3665`, Demonic Might)

> "You may also take this Virtue more than once, though it can account for no
> more than half of the character's total Virtues." (`:3669`, Demonic Powers)

Both shipped with `max_per_target: 255` and repeated freely, so neither ceiling
existed. Finding 34 left it out as "a whole-build ratio, not an absolute cap".

**Resolution.** The ratio is data, not code: `PointItem.max_share_of_kind`
carries a `Share { numerator, denominator }`, rejected at load if the
denominator is zero or the numerator exceeds it, and both Demonic entries carry
`1/2` in `rules/core/virtues_flaws.json`. `validate_share_of_kind_cap`
(`crates/arm-rules/src/validation/caps.rs`) follows `validate_tainted_cap`, the
same sentence shape already implemented: points rather than a headcount, the
integer form `part · denominator > total · numerator` so no rounding rule is
needed, and measured against the item's own kind. New code `too_large_share`
with `issue-too_large_share` in both locales.

Two things are deliberate rather than incidental, and both are written into
`crates/arm-rules/RULES.md`. It is a **warning, not an error**: each sentence
says "this Virtue", so the two ceilings are independent, and reading "total
Virtues" as Virtue *points* is an interpretation — the Tainted rule has the
book's own gloss at `:3000` to settle points-vs-headcount, these two sentences
have none. And it reads the **folded** selection list, not raw
`entity.selections`, because Devil Child grants a free Demonic Might or Powers
(`:3673`) and a granted copy is still a copy; `validate_tainted_cap` counts only
bought ones, so two identically-worded "half" rules now disagree about grants on
purpose.

### To-do 7 — Slow, Restricted and Variable Power name the power they limit

**Status:** done (2026-09-06) — `2380322`; residual gap recorded by finding 36

> "This Flaw may be taken once for each power the character possesses."
> (`:6689`, Restricted Power)

> "This Flaw may be taken more than once, if the character has multiple powers,
> but not more than once for a single power." (`:6761`, Slow Power)

> "This Virtue may be taken more than once, if the character has more than one
> power, but it only applies once to a single power." (`:5205`, Variable Power)

All three shipped unparameterized with `max_per_target: 255`, so every copy
shared one duplicate key and nothing distinguished two copies aimed at one power
from two aimed at different ones — the cap was unenforced in the permissive
direction.

**Resolution.** A free-text `power` parameter on each of the three, with
`max_per_target` removed so it falls back to its default of 1: one copy per named
power, no ceiling across different powers. Data only — the `text` domain and the
`(item_ref, params)` duplicate key already did the work. A power cannot be a
`ref`, because Focus, Greater, Lesser, Personal and Ritual Power are themselves
repeatable and unparameterized, so a magus with three Greater Powers holds three
indistinguishable rows with nothing to point at. `param-label-power` added to
both locales, and the three `name` templates take the `{power}` placeholder;
`flaw.restricted_power`'s German name was corrected from "Eingeschränkte
**Macht**" to "Eingeschränkte **Kraft**" per `tugenden-fehler.md:350` in the same
edit.

**Save impact, accepted:** an existing save holding a paramless copy raises
`missing_param` on open. No data is lost — saves store choices and the engine
only reports — and the player clears it by naming the power.

**What this does not claim.** Nothing checks the typed string against a power the
character actually holds, and the duplicate key is byte-for-byte, so "Wolf
Shape", "wolf shape" and "Wolf Shape " are three targets and an empty name is
accepted. It stops an honest mistake, not a determined evasion. Both halves go
back into `docs/open-todos.md` as narrowed rows.

### To-do 4 — False Power repeats, and every copy after the first is Minor

**Status:** done (2026-09-06) — `7b95ca0`; deferred by finding 34

> "This Flaw may be taken multiple times, once for each appropriate Supernatural
> Virtue that the character possesses, but in each subsequent instance as a Minor
> Flaw rather than a Major one." (`:6096`)

Magnitude belongs to the catalogue entry, never to a selection, so a second copy
of one entry would be charged 3 points instead of 1. Finding 34 blocked the
repeat outright rather than ship wrong point totals.

**Resolution.** Two entries, the way `virtue.amorphous_major` /
`virtue.amorphous_minor` already does it: `flaw.false_power` keeps its id, stays
Major and gains `max_total: 1`; the new `flaw.false_power_minor` is Minor,
requires `flaw.false_power` and repeats freely. Major plus two Minors costs
3 + 1 + 1, which is the whole point of the change and is what the new test
asserts. Labels changed on an existing entry — "False Power" became "False Power
(Major)" in English and "Falsche Macht (Groß)" in German — because an unsuffixed
name sitting beside "(Minor)" reads worse.

**The asymmetric id pair is deliberate**, and `RULES.md` says so twice over.
Renaming to `_major` would break every save holding the Flaw for cosmetic
symmetry, and it would fail the load outright:
`validate_magnitude_variant_exclusivity` requires a `_major`/`_minor` pair to be
mutually incompatible, which these two must **not** be, since the Minor copies
only exist once the Major one does.

**Still not expressed:** "once for each appropriate Supernatural Virtue" also
means the copies must name *different* Supernatural Virtues, and no copy names
any. That needs a parameter domain meaning "an item of category X that this
character possesses"; recorded, not built.

### To-do 6 — closed lists in the rulebook become closed lists in the picker

**Status:** done (2026-09-06) — `b86889c`; asked for by name in finding 37

> "He can only create spells in one narrow area, which must be one of the
> following four options" (`:3909`, enumerated at `:3911-3917` as *Abjuration*,
> *Divination*, *Healing*, *Evil Eye*), and "You may pick this Virtue more than
> once, to acquire expertise in a different category of spells." (`:3919`)

> "associated with one of five classes of beings: animals, divine beings,
> faeries, demons, or magical creatures" (`:4135`, Inoffensive to (Beings));
> "one of six classes of beings: animals, mundane humans, divine beings,
> faeries, demons, or magical creatures" (`:6526`, Offensive to (Beings));
> "one of three classes of beings: mundane humans, demons, or divine beings"
> (`:6893`, Unbearable to (Beings))

All four stored free text, so any string was legal, and Folk Magic — carrying no
parameter at all — could not repeat.

**Resolution.** `ParameterDomain::Enumerated`, with the list on the
`ParameterDef` itself (`values`) rather than in a global registry, because the
three being lists are different subsets and no single enumeration could serve
them. Load-time integrity rejects an `Enumerated` parameter with an empty or
duplicated list and any other domain carrying one, naming the item and key. A
value outside the list simply fails to resolve in its domain, so the existing
`unknown_param_value` covers it and no new issue code or Fluent key was needed —
which also means open grants and Warping fills come along free, since they share
`validate_selection_parameters`. `ParameterPicker.svelte` gained an
`enumerated` branch rendering a `<select>` whose options run through
`displayName`, reusing the existing grant-aware already-taken greying.

Folk Magic now repeats "to acquire expertise in a different category" with **no
ceiling written anywhere**: one copy per category, four categories, so a fifth
copy must repeat one and is rejected as a duplicate. A supplement adding a fifth
category would raise the ceiling on its own — which is exactly why this was the
recorded fix rather than `max_per_target: 4`. `flaw.fish_out_of_water_terrain`
stays free text; its list ends "…, etc." (`:6130`), which is what the book means.

Nine value ids ship with EN and DE text, locked by a new coverage test:
`being.animals`, `being.demons`, `being.divine`, `being.faeries`,
`being.magical_creatures`, `being.mundane_humans`, `folk_magic.abjuration`,
`folk_magic.divination`, `folk_magic.evil_eye`, `folk_magic.healing`. The German
labels came from the German rulebook rather than being invented, since the
curated glossary tables do not cover them. `param-label-category` was added to
both locales; `virtue.inoffensive_to_beings`'s German name was corrected from
"Für {being} ungefährlich" to "Unauffällig für {being}" per
`tugenden-fehler.md:180`.

**Save impact, accepted and not migrated:** a save holding a typed `being` value
now reports `unknown_param_value`, and one holding `virtue.folk_magic` reports
`missing_param`. The old values were unconstrained free text, so no mapping would
be honest; `SCHEMA_VERSION` does not apply, since the save bytes are untouched
and this is a ruleset change. Nothing is lost — the player re-picks from a
dropdown.

> **Superseded for the `being` half (Slice 0, before 0.3).** Three rounds each
> accepted a save impact of their own, and together they meant a v0.2.0 player
> upgraded into a wall of errors on their own character. The typed `being` labels
> **are** migrated now: `fold_legacy_being_params`
> (`crates/arm-rules/src/types.rs`) maps the fifteen labels a v0.2.x player could
> have typed — six classes × both shipped languages, plus the three German dative
> forms the Inoffensive entry prints at `:4135` — onto the `being.*` ids, case- and
> whitespace-insensitively, which answers the "no mapping would be honest"
> objection by carrying German as well as English. `SCHEMA_VERSION` still does not
> apply, exactly as reasoned above, so the fold is value-driven and idempotent.
> Folk Magic's `category` and the per-power `power` remain un-migrated on purpose
> — they were never *stored*, so there is nothing to migrate from — and a genuine
> `too_many_selections` is a rules finding that survives untouched. See
> `crates/arm-rules/RULES.md`, "Save impact, and the migration that now absorbs
> it".

**Adjudicated, not silently fixed.** The three (Beings) items carry
dual-category descriptors joined by *and*/*or* ("General and Hermetic" `:4134`,
"Hermetic and General" `:6525`, "Hermetic or General" `:6892`) yet ship with one
category each. The dual-category sweep (`5729e6d`) covered only the four items
whose descriptors use a **comma** — Sufi, Visions, Raised from the Dead,
Suppressed Gift — so *and*/*or* descriptors were never in its scope and nothing
was missed. Two further items are in the same position (`:5882` Curse of
Slander, `:6635` Primogeniture Lineage), and the *or* cases raise a separate
semantic question, so all five go to `docs/open-todos.md` as one row rather than
being changed here.

> **Superseded 2026-09-09 for three of the five.** "Adjudicated, not silently
> fixed" no longer holds for Offensive and Unbearable to (Beings): `7ea4f5b`
> found the single `hermetic` category *was* a wrong-output bug, since the three
> non-magus profiles forbid it, and set both to `["general"]` with the
> eligibility gates the category had been enforcing by accident. Inoffensive to
> (Beings) kept `["general"]` but gained the gate `:4139` requires. Only
> `flaw.curse_of_slander` and `flaw.primogeniture_lineage` remain on the row.

**Also found, not changed:** the German rulebook heads Inoffensive to (Beings)
"Für (Wesen) ungefährlich" (`rules/source/de/…Basisregeln.md:4133`) while
`tugenden-fehler.md:180` gives "Unauffällig für (Wesen)". The tables are
canonical for i18n labels, so the table won — but the two German sources
disagree, and that is now a to-do row.

> **Closed 2026-09-09, not merely noted.** The disagreement is explained: the
> glossary row carries a per-row `SdM:M` tag (*Sphären der Macht: Magie*, EN
> RoP:M per `grundbegriffe.md:215`), so its wording was distilled from another
> book's printing of the same Virtue — the row sits in `### Allgemeine Tugenden,
> Klein` (`tugenden-fehler.md:173`), not a supplement block, and the Core Rules
> carry the Virtue too, at the line-mirrored `…Core Rules.md:4133`.
> `rules/source/de/` is left as published; the decision and both citations now
> live as a prose subsection in
> `rules/source/de/translation-tables/README.md`, beside the "Tainted" note.
