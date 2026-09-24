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
duplicate: **the work is described there, the release gate is here.** **Row 46 is largely discharged** — § 9 shows why: its premise (*"every sweep so
far has read `narrative` entries only"*) was written on 2026-09-18, **before** the
batches ran, and the audit's twelve-point check covered all 655 entries with
B19's check 11 doing this row's job verbatim. The residue is narrower than the
row reads: the computed entries in **B01–B11's** spans, since the sign-check
instruction entered the briefs only at **B12**. Tracked as § 8 row **12e**.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 38 | **The `description` fill is 109 entries into a 655-entry book. Both swept ranges — the Flaws block and the head of the Virtues block (`ArMDE:3360-3950`) — are clean *under the current screen*; `ArMDE:3951-5282` is what remains.** *Updated 2026-09-19.* **Read the previous sentence's caveat as the row's main lesson.** "The Flaws block is DONE" was written here on 2026-09-15 and was wrong: the token screen at the time knew only a signed number and a botch term, so every rule the books state in *words* — a cap, an Ease Factor, a formula, a rounding direction, an absolute — was invisible to it, and **13 Flaws in the "done" block plus 6 in the "done" Virtues head were still misclassified `narrative` four days later**. They were found on 2026-09-19 only because a third, phrase-based screen was added (`uncomputed_clauses.rs::MECHANICAL_PHRASES`) and both blocks were re-read under it; 19 entries moved to `uncomputed_rule`, 4 gained written `NO_RULE_DESPITE_TOKEN` readings. A block is clean against the screen that read it and no further, so **"swept" is dated, not permanent** — widening the screen re-opens every block behind it, and `SWEPT_BLOCKS` now says so in its comments. The Virtues sweep changed method on Norbert's instruction and the change is the point: the token screen is a **work-list generator, not the analysis**, so working only its output inherits its blind spot. Every `narrative` entry in the swept range is now read in full — 48 of them, of which only 16 were token-flagged. The other 32 are where the findings came from (rows 45 and 46 below both originate there). **Keep that method for the residual**: read every `narrative` passage in the range, not just the flagged ones, and record a verdict per entry so no successor pass re-reads it. The whole-block survey flagged **45** under the two-token screen; 24 are done, **21 remain** by that count, plus whatever the phrase screen adds in `ArMDE:3951-5282` (not yet surveyed) and every unflagged `narrative` entry there, whose count is not yet known because nobody has read them. Residual flagged ids are listed in the sweeps' commit messages. **Also owed from the swept ranges:** ~~heading anchors for the entries that stayed `narrative`~~ (discharged 2026-09-21 — all 655 entries carry one in both stores, so no sweep needs to record them any more) and 10 unnormalized `„…"` quote pairs elsewhere in `rules/i18n/de/virtues_flaws.json` that predate the normalize-on-the-way-out policy. *Prior state, 2026-09-15:* The Flaws sweep (`ArMDE:5639-7113`) read all 51 flagged `narrative` Flaws in both languages: **49 reclassified `narrative` → `uncomputed_rule` with `description` filled in both locales** (48 texts — `flaw.missing_ear`'s passage is a single sentence its `summary` already carries in full), and **2 confirmed correctly `narrative`** (`flaw.overconfident_major`/`_minor`, whose passage uses "botch" as a bare roleplaying verb and states no rule; recorded with their reading in `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`, kept honest by `exempted_entries_still_trip_the_screen`). The third assertion, *"a `narrative` entry whose cited passage carries a mechanical token is a dropped rule"*, is now `uncomputed_clauses.rs::no_narrative_entry_in_a_swept_block_drops_a_mechanical_clause` — **green**, scoped by `SWEPT_BLOCKS`, so re-dirtying the Flaws block is a failing build rather than a claim in a report. **What remains:** the **Virtues block (`ArMDE:3360-5282`)**, unsurveyed; running the same guard with a Virtues row added to `SWEPT_BLOCKS` is the cheapest way to produce its work list — that is how the Flaws block turned out to be 51 rather than the estimated 43. Same method: extract the full passage into `description` in English, take the German from the line-mirrored `rules/source/de/` file at the same line number, convert en dashes before digits to ASCII hyphens, strip Markdown link and emphasis syntax, and reclassify to `uncomputed_rule` where the engine computes nothing. (Recording the heading anchor while reading is no longer part of this job — row 43's backfill covered every Virtue and Flaw on 2026-09-21.) Beyond the rule-droppers sit roughly 550 entries whose rule is either already computed and displayed or is genuine flavour: worth doing for uniformity, not for correctness, and a separate decision. | an implementation pass (data, both locales) | 2026-09-14 |
| 39 | **DONE 2026-09-22 for the scanno (D26); the `-3penalty` half below is still open.** Fixed as this row predicted — source first, then the two i18n strings. A **second** occurrence turned up at `ArMDE:3502` (`virtue.berserk`, "give guarter" for the idiom *give quarter*), which is what proved the `q`→`g` OCR error systematic rather than a one-off; it is fixed too, and a todo is filed in `arm-de-translation` (`docs/todo.md`), whose `original-english/reviewed/` copy still carries both (at its own, different line numbers — that copy has drifted from ours). Original entry: **`ArMDE:7054` reads "guarter" where it means "quarter", and the typo is now shipped twice.** An OCR artefact in the source Markdown, faithfully transcribed into `flaw.waster_of_vis`'s `summary` and — by the same verbatim-from-source convention — into its new `description`. The convention is doing its job, so this is not a transcription bug, but it is user-visible in both locales' English text. **Fixing the source line is the correct move**, not the i18n: patching only `rules/i18n/` would break verbatim-ness and would be silently undone by any future re-extraction. Not done when found purely because `rules/source/**` was outside that slice's declared file list. One character, one line, then re-sync the two i18n strings. The German line (7054 of *Basisregeln*) reads "Viertel" correctly and needs nothing. **Two more source defects found since and belonging with it:** `ArMDE:6929` reads "-3penalty" with no space, and the German sources close a German opening quote `„` with an ASCII `"` (see "Recorded conventions" below — that one is corpus-wide house style and is **not** to be swept; it is listed here only so the two are not confused). | an implementation pass (source fix + i18n re-sync) | 2026-09-14 |
| 40 | **The rules-text search index reads `summary` only, so everything written into `description` is unsearchable.** `ui/src/lib/derive.ts` builds the V/F search index from each entry's `summary` alone. That was harmless while `description` was empty for all 655 V/F; it is now populated for 74 of them, and the populated half is exactly where the mechanical clauses live — so searching "botch" finds the two entries whose clause happened to land in sentence one rather than the fifty-odd that state it. The investigation called this a one-line change that should ship with the data. **Not done in the data slices**: their briefs scoped `ui/` to render-blocking changes only, and the index change alters search behaviour, so it wants its own test and a UI-side decision about whether a long paragraph should be indexed whole. The tooltip itself needed no change and got none — `VirtueFlawTab.svelte`'s `tip()` already reads `description ?? summary`. | an implementation pass (frontend, with a test) | 2026-09-14 |
| 41 | **The Markdown export lists Virtues and Flaws by name only, so no rules clause reaches an exported sheet.** `crates/arm-rules/src/export/sections.rs` emits neither `summary` nor `description` for a V/F. Under the old design that was defensible — the tooltip carried the text and the export carried the sheet. It stops being defensible now that "the displayed text is the only carrier" is the adopted principle for `uncomputed_rule` entries: an exported sheet has no tooltip to fall back on, so a player working from the export sees a Flaw's name and nothing about the botch die it costs them. Flagged by the investigation as a consequence to decide deliberately rather than discover later. Needs a product call on how much text an export should carry (name only, name + summary, or name + full description) before any code. | a decision, then an implementation pass | 2026-09-14 |
| 42 | **Four swept Flaws state rules the engine could compute but has no representation for — and three of them are *caps*, which nothing in the engine can express at all.** Surfaced by reading every cited passage in the Flaws block (2026-09-15); all four are now `uncomputed_rule` with their full text displayed, which is honest but is the weaker of the two possible answers. (a) `flaw.weak_personality` (`ArMDE:7078`) caps **every** Personality Trait to the range +1..-1 *and* ceilings the roll ("treat any roll above 6 as merely 6"). (b) `flaw.uninspirational` (`ArMDE:6921`) caps **two Characteristics**: "His Presence and Communication may not be greater than 0". **Re-costed 2026-09-15 by the row-32 fix:** this row previously said the engine "already has `characteristic_limit`, so this one is arguably encodable today". That effect is gone — it was never a cap in the first place, only Great/Poor (Characteristic) mis-modelled as one, and it is now `characteristic_score_delta_param`, a *grant*. Uninspirational, by contrast, **is** a genuine per-Characteristic buy cap, and the engine now has no way to express one: it would need a new effect variant lowering `base_max` for a named Characteristic, plus `characteristic_cap` becoming entity-relative again. Still the cheapest of the four, but engine work rather than a data edit. (c) `flaw.fickle_nature` (`ArMDE:6124`) is a creation-time **grant**, not a penalty: "Select a Personality Trait at +4, and its opposite at +4". (d) `flaw.lingering_injury` (`ArMDE:6352`) is a **formula**, not a constant: the penalty is multiplied by `1 + Decrepitude Score`. The common blocker for (a), (c) and partly (d) is that Personality Traits are free-form entries on `types.rs::Entity::personality_traits` with **no granting effect and no validation of any kind** — a user can type +9 on a Weak Personality character and nothing objects. Deliberately *not* fixed in the sweep: inventing an effect variant to fit is what `CLAUDE.md` → "Rules provenance" forbids, and each of these is a wrong-rules-output change wanting its own TDD slice and a reading decision. Rated as product-integrity rather than cosmetic, but low urgency: today the rule is *displayed* rather than *enforced*, which is the same standing as every other `uncomputed_rule` entry. **Nineteen more joined them on 2026-09-19**, when the phrase screen landed and both swept blocks were re-read: caps (`virtue.command_animals`, `ArMDE:3575-3578`, "up to 12 human-sized animals"), immunities (`virtue.death_prophecy`, `ArMDE:3639-3644`, "cannot die as a result of wounds or old age"), formulas (`virtue.extractor_of_form_vis`, `ArMDE:3779-3782`, a tenth of a Lab Total, round up; `virtue.folk_magic`, `ArMDE:3907-3920`, a Casting Total formula plus a Fatigue cost), selection prerequisites the engine cannot express (`flaw.university_dean`, `ArMDE:6923-6926`, a minimum age of 40; `flaw.flawed_powers`, `ArMDE:6146-6149`), and a budget exemption (`flaw.servant_of_the_land`, `ArMDE:6717-6720`, a granted Flaw that "does not count toward the character's total number of Virtues and Flaws"). All are now `uncomputed_rule` with their full passage displayed in both locales — the same honest-but-weaker answer as the four Flaws above, and the same open question: whether any of them is worth *computing*. The minimum-age prerequisite and the V/F-budget exemption are the two that look cheapest, because the engine already tracks age and already sums the budget. | an implementation pass each, (b) first | 2026-09-15 |
| 45 | **At least sixteen social-status Virtues say "you may take Academic Abilities during character generation" and carry no effect at all, so taking one and then buying the Ability it authorizes produces a spurious validation ERROR.** Found 2026-09-18 by reading the *unflagged* `narrative` entries in `ArMDE:3360-3950` — the token screen cannot see it, because the rule is stated without a signed number. **This is wrong rules output on a legal character, which `CLAUDE.md` rates up, and it is not a "rule the engine cannot express":** `types.rs::Effect::AbilityAuthorization` already carries a `categories: Vec<AbilityCategory>` arm, `abilities.json` gates `["academic", "arcane", "martial"]`, and `validation/authorization.rs::validate_ability_authorization` pushes `CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE` as an **error** for any non-magus holding a gated-category Ability without an authorizing Virtue. Only **two** entries in the whole catalogue use `ability_authorization` today (`virtue.student_of_realm`, `flaw.covenant_upbringing`). So a Clerk companion (`ArMDE:3573`, "Due to your training, you may take Academic Abilities during character generation") who buys Artes Liberales is told he may not. Confirmed in the data and the engine, not inferred from the passage alone. Same defect on Alim, Archieunuch, Beadle, Brother Chaplain, Brother Knight, Brother Sergeant, Bureaucrat, Eunuch, the three Failed Apprentice variants, Almogaten, Almogavar, Fida'i, and `virtue.familiarity_with_the_fae` (Faerie Lore specifically). `virtue.custos` needs a **parameter** — it authorizes a *choice* of one of Martial/Academic/Arcane. `virtue.covenfolk`/`virtue.custos` additionally state "may not take the Wealthy Virtue or Poor Flaw", which `incompatible_with` already expresses. **Survey the catalogue before writing any data** — this was found in one 590-line range and is certainly systemic beyond it. **⚠ SCOPE CORRECTED 2026-09-24: "social-status Virtues" is the wrong boundary, and three counterexamples now prove it.** `virtue.berserk` (`ArMDE:3500-3503`, *Minor, **General***, "You may learn Martial Abilities at character creation" — **F-18**, and row 48 shows it raising a spurious `ability_category_requires_virtue` **error** on the book's own Berserker grog), plus `virtue.knight` and `virtue.priest`, both `narrative` with no effects (row 50). **So the survey sweeps the whole catalogue for the authorization sentence in any category — not the social-status block.** Do not narrow it back: the original scope came from the range that was read, not from anything the rule says. `virtue.guild_master` (**F-102**, "You may select Academic Abilities at character generation") is a fourth instance found from the other direction, while settling **Q-26**. | an implementation pass (survey, then data + a parameter) | 2026-09-18 |
| 46 | **The 218 computed Virtues and Flaws have never been audited against what the engine actually does — and that is the class row 35 came from.** Norbert's call, 2026-09-18: do this after the row-38 narrative sweep, and be exhaustive. Every sweep so far has read `narrative` entries only, so `creation_effect` and `in_play_effect` entries have been checked for *referential integrity* and never for *correctness of the arithmetic*. That is not a theoretical gap: `flaw.flawed_parma_magica` and `flaw.limited_magic_resistance` were `in_play_effect` entries applying to all ten Forms instead of one, wrong on nine Forms each, and they surfaced by another route entirely (row 35) rather than by any guard. The scope is the whole computed catalogue and it is closed rather than partial — all 218 sit in the two core-rulebook blocks: **Virtues `ArMDE:3360-5282`, 96 `creation_effect` + 46 `in_play_effect` = 142; Flaws `ArMDE:5639-7113`, 29 + 47 = 76.** The method is the expensive one and the cheap version is worthless here: read the cited passage, then read what the engine computes, and compare. **A starting lead, measured 2026-09-18:** 213 entries carry an `effects` array but 218 are classified computed, so **five claim an effect they do not have** — `flaw.corrupted_arts`, `flaw.savantism`, `virtue.devil_child`, `virtue.nephilim`, `virtue.simple_student`. `devil_child`/`nephilim` are probably fine (mythic-type markers whose machinery is `mythic_type.*` grants in `mythic_companion.rs`), but `flaw.savantism` and `virtue.simple_student` are named nowhere in `crates/` or `ui/src` — both are XP-rate rules by name, so either misclassified or specified and never implemented. | an implementation pass, in sequential slices | 2026-09-18 |
| 43 | **DONE for Virtues/Flaws 2026-09-21 — all 655 entries now carry an anchor in both stores; the remaining catalogues (~350 `source` blocks) still owe theirs.** Norbert has confirmed the `rules/source/` Markdown will be updated upstream with **line numbers changing throughout**, and the German edition re-synced. Every `source: { file, lines }`, every acronym-plus-line citation, and every line reference in `crates/arm-rules/RULES.md` shifts at once — and `rulebook_citations.rs` cannot see it, because it checks a cited range lands on non-blank lines rather than that those lines say what the citation claims. The durable key already exists in the sources: every virtue and flaw has its own `####` heading in both languages, and the Markdown already generates anchor slugs in its own cross-links. The Flaws block and the head of the Virtues block now carry them for every entry either sweep touched (English on `types.rs::SourceRef::anchor`, German in the `rules/i18n/de/source_anchors.json` sidecar), with three guards including the first test of the German line-parity invariant. **The Virtues/Flaws rollout is now complete** (2026-09-21): the 561 missing English anchors and 561 missing German ones were derived from the `####` headings by `rules_source_provenance.rs::regenerate_source_anchors`, an `#[ignore]`d regenerator that rewrites both stores line by line. The slug rule reproduced all 94 hand-recorded anchors **exactly**, in both languages, before any were generated. Two new guards came with it: `every_anchored_catalogue_entry_records_the_heading_that_opens_its_range` (the anchor is the heading on `source.lines[0]`, and every entry of a swept catalogue has one in both stores) and `no_source_range_runs_past_the_heading_that_follows_it`, which found and fixed F-540. **What is still owed is the other catalogues** — abilities, arts, spells, houses, equipment, childhoods, aging rows — each added to `FULLY_ANCHORED_CATALOGUES` as it is swept. The 2026-09-19 re-sweep added 23 pairs and the parity guard passed on all of them first time; the 2026-09-21 backfill then passed it on all 655, which is strong evidence that the line-mirroring invariant actually holds. Full design and residual questions — notably whether prose code comments should move off line numbers at all — in `docs/rules-source-resync.md`. Related: P8, the re-sync itself. | rollout during later sweeps | 2026-09-15 |
| 47 | **`effective/xp.rs::charged_cost` overcharges an Affinity-bought score by one experience point, always against the player. Today it bites only Abilities under a 3/2 Affinity, at scores 1, 4, 7, 10, 13, 16 and 19 — but the formula is wrong generally, so any new ratio or table can widen it.** The two formulas differ exactly when `(table_xp × den) mod num` falls strictly between 0 and `den`; for 3/2 that is `T ≡ 2 (mod 3)`. Ability costs (`5·n(n+1)/2`) hit it at every third score; **Art costs are triangular numbers, whose residues mod 3 cycle 1, 0, 0 and are never 2, so no Art score trips it at 3/2**; `virtue.linguist` (5/4) cannot trip on Abilities because every Ability cost is a multiple of 5; and Flawless Magic's 2/1 requires `0 < r < 1`, an empty interval, so spell mastery is clean. That matches the V/F audit's F-547, which lists the same three as provably clean — the defect is real but narrow, and a fix must not be sold as sweeping. Its own doc comment states the rule correctly — "the points actually charged to reach a fixed table cost `T` are the smallest `c` with `ceil(c·num/den) ≥ T`" — and then implements `ceil(T·den/num)`, which is a different function. The smallest valid `c` is `floor(den·(T−1)/num) + 1`. Worked: Single Weapon 7 (`T = 140`) under Affinity 3/2 charges 94, but `c = 93` already satisfies `ceil(93·3/2) = 140 ≥ 140` while `c = 92` gives 138 — so the correct charge is 93. **Found by the book disagreeing with us, not by inspection:** the Specialist grog template (`ArMDE:1298-1332` `#### The Specialist`) balances to the point at 93 (`30+30+50+15+7+75+5+75+93 = 380`, against a grant of `75 + 45 + 15×14 + 50` for Warrior) and is one point short at 94 — which is also what makes its printed "Bows 1 (2)" (`ArMDE:1326`) come out at 2 banked points rather than 1. **The existing unit tests cannot catch this and one of them pins the bug:** `effective.rs::affinity_charged_cost_matches_perdo_example` uses the rulebook's own worked example (Perdo 10, `T = 55`, `ArMDE:2443`), where both formulas give 37 — which is precisely why it survived — and the sibling assertion `charged_cost(275, Some((3,2))) == 184` asserts the wrong value (correct: 183) and must change with the fix. Wrong rules output on every Affinity character, so `CLAUDE.md` rates it up. Recorded rather than fixed because the slice that found it was forbidden from touching production code. Detail and arithmetic: `docs/book-template-conformance.md` § S2. | an implementation pass (engine + the two existing assertions) | 2026-09-22 |
| 48 | **`virtue.berserk` applies its three combat modifiers unconditionally, so every Berserk character permanently fights as if raging — and it is missing the Martial-Ability authorization that is row 45's exact defect.** `ArMDE:3500-3503` `#### Berserk` says "**While berserk**, you get +2 to Attack and Soak scores, but suffer a -2 penalty to Defense"; the entry in `rules/core/virtues_flaws.json` carries the three as plain `combat_mod`/`soak_mod` effects under `classification: in_play_effect`, with nothing expressing the condition. The Berserker grog template (`ArMDE:1195-1227` `#### The Berserker`) is the proof: the engine prints Soak 11 and Pole Axe +15/+5 where the book prints Soak +9 and +13/+7 — every figure shifted by exactly Berserk's +2/−2/+2 — while Initiative, Damage, Encumbrance and the Size +1 wound bands, which Berserk does not touch, all reproduce exactly. **The engine has no way to express a conditional in-play modifier at all**, so this is engine work, not a data edit, and the same shape will recur on every "while X" Virtue. Separately, the same passage ends "You may learn Martial Abilities at character creation", and the entry carries no `ability_authorization` — so the Berserker's Great Weapons 5 and Single Weapon 1 raise a spurious `ability_category_requires_virtue` **error** on a legal character. That half is a data-only fix and belongs to **row 45's survey**, which should be widened beyond social-status Virtues to catch it. Detail: `docs/book-template-conformance.md` § B1, § B2. | an implementation pass each; the authorization half folds into row 45 | 2026-09-22 |
| 49 | **The engine cannot express a *conditional* modifier, so every Virtue whose bonus applies only in stated circumstances is applied ALL the time — a defect class the V/F audit has already rated HIGH twice, independently, as F-20 (`virtue.berserk`) and F-45 (`virtue.cyclic_magic_positive`, "both modifiers conditional in the book, unconditional in the data"). The instances below are the third and fourth, so this is one class with four witnesses rather than four separate bugs, and should be scoped and fixed as one.** **Read D4 before implementing**: it ruled that conditional modifiers resolve **statically** — each decided once from its own passage, with no runtime context — so the fix is a static reading per entry, *not* a condition engine. The Bjornaer's +22-against-+19 below is precisely what D4 exists to prevent. Row 48's Berserk is one instance of four now confirmed, three of them found by building the book's own magi. `Effect::CombatMod`, `SoakMod` and `CastingTotalMod` carry a `scope` but no condition, so `scope: all` is the only encodable reading and the character permanently enjoys a situational bonus. Confirmed instances: **`virtue.ways_of_the_land`** (`ArMDE:5231-5233` `#### Ways Of The (Land)`) grants +3 only on rolls "that directly involve that area and its inhabitants", but is encoded `casting_total_mod scope: all`, so the Bjornaer template's MuAn total reads +22 against the book's +19 (`ArMDE:1643`); **`virtue.cyclic_magic_positive`** / **`flaw.cyclic_magic_negative`** (day and night) and **`virtue.special_circumstances`** are all `scope: all` on the Mercere, where the three net to a permanent +3 and every printed Casting Total is 3 high (`ArMDE:1992-1996`). Note the two Cyclic entries are **not** symmetric — see the V/F audit's **D52**. The fix is an engine capability (a condition on the effect, plus a UI affordance for "is it active?"), not a data edit; until it exists these Virtues are *over*-applied, which flatters the character. **Wrong rules output on every character holding one**, which `CLAUDE.md` rates up. Detail: `docs/book-template-conformance.md` § MAG1, § B2. | an engine capability, then a data sweep of every conditional modifier | 2026-09-23 |
| 50 | **Six more Virtues carry none of the mechanics their rulebook text states — the same shape as row 45, found by building the book's own companions, and one of them is wrong rules output rather than a false rejection.** All confirmed against the source and the data, not inferred. (a) **`virtue.student_of_realm`** (`ArMDE:5052-5054` `#### Student of (Realm)`) is the serious one: it promises "a +2 bonus on all uses of the appropriate Lore" and the entry carries **only** an `ability_authorization`, so the Priest's printed `Dominion Lore 3+2`, the Witch's `Magic Lore 3+2` and the Merinita's `Faerie Lore 3+2` all compute as **3**. The same entry is also **over-permissive** — it authorizes all four realm Lores regardless of the `realm` parameter chosen, so a Student of Magic may buy Divine, Faerie and Infernal Lore — and its final clause, "You may not take Student of (Realm) and Puissant Ability for the same Lore", is unmodelled. **The over-permissive half is already `docs/vf-audit/decisions.md` D14**, which was written from exactly this entry and is what moved the parameter-constraint work ahead of D13; the **missing +2** is the half D14 does not cover and is the reason this row exists. Implement them together. (b) **`virtue.well_traveled`** (`ArMDE:5239-5241`) grants "fifty bonus experience points" on a named Ability list and is `classification: narrative` with **no effects at all**; the shape it needs already exists as `restricted_ability_xp` on `virtue.educated`. (c) **`virtue.educated`** (`ArMDE:3711-3713`) has two clauses and only the 50-XP earmark was extracted, so the Witch's Medicine 5 is refused. (d) **`virtue.clerk`, `virtue.knight`, `virtue.priest`** are all `classification: narrative` with no effects, each raising spurious `ability_category_requires_virtue` errors — **Clerk is named in row 45, Knight and Priest are not**, which is the second proof that row 45's "social-status Virtues" scope is too narrow (Berserk was the first). Detail: `docs/book-template-conformance.md` § F1, § K1, § P1, § P2, § P3, § P4, § W1. | a data pass for (b)-(d); (a) needs a parameter-aware authorization | 2026-09-23 |
| 51 | **Four smaller gaps the twelve magus templates exposed, none of them wrong arithmetic: three model/storage limits and one missing catalogue entry.** (a) **A granted Magical Focus cannot say what it is on.** `rules/core/houses.json` grants House Tremere `virtue.minor_magical_focus` with no `focus` parameter, though the book names it — "Minor Magical Focus (certamen)" (`ArMDE:2064`, `ArMDE:2281`). `validate_magical_focus` only counts foci, so nothing complains and the character carries a focus on nothing. (b) **An Art's leftover experience has nowhere to live.** `ArtScore` stores a whole score, so the Guernicus's printed "In 12+3 (5)" (`ArMDE:1881`) loses its 5 banked points and his fixture needs `xp_pool` 432 against the book's 435. The Ability side has the same limit (the Specialist's "Bows 1 (2)"). (c) **A Magical Focus is free text**, so the engine reports base and within-focus totals and cannot say which spells fall inside the focus; the book picks per spell (Flambeau, Ex Miscellanea, Mercere, Tremere). (d) **No grapple entry** in `rules/core/equipment.json`, so the Ex Miscellanea's Grappling row (`ArMDE:1774`) has no counterpart; likewise **`spell.piercing_the_magical_veil`** is absent — the book names the Magical, Divine and Infernal siblings at `ArMDE:15709` but prints none of them, so the Criamon's seventh spell cannot be recorded and he reads 100 of his 120 levels. Detail: `docs/book-template-conformance.md` § MAG10, § MAG12, § MAG8, § MAG5, § MAG2. | a decision each; (a) and (d) are the cheapest | 2026-09-23 |
| 52 | **Two engine-capability gaps on the Knight template that no V/F fixing round will catch, because neither is Virtue/Flaw data — and the second is an accepted compromise already committed into the test suite.** Both are recorded in `docs/book-template-conformance.md` but were missing from this list until 2026-09-24; the Knight's third finding (`virtue.knight` carrying no Ability authorization) *is* filed, as row 50 and as one of row 45's counterexamples, which is exactly why the other two were easy to overlook. (a) **K5 — `EquipmentSlot::equipped` is overloaded.** It gates **both** whether a weapon yields a Combat row **and** whether it contributes Load, and the book needs those separated: the Knight prints five Combat rows including two for a great sword (`ArMDE:1468-1472`) while printing an Encumbrance that counts only the wielded set (`ArMDE:1484`). One flag cannot do both. This became load-bearing when `derived/combat.rs::encumbrance` was changed to count equipped gear only (commit `f70c57f`, Norbert's call), so **the shipped fixture resolves it by marking the great sword unequipped and losing its two Combat rows** — a wrong number judged worse than a missing row. That trade is live in `tests/fixtures/book_templates/companion_knight.json` and is asserted by `the_knight_matches_the_book`, so it is not a hypothetical. The fix is a third state — carried and wieldable but not currently wielded — or two separate flags, plus a UI affordance for switching loadout. (b) **K3 — mounted combat is unmodelled.** "A mounted character adds his Ride score, to a maximum of +3, to his Attack and Defense Totals" (`ArMDE:16837-16839` `#### Mounted Combat`); at Ride 5 the Knight's two mounted rows are exactly his on-foot rows plus +3/+3, so the rule is trivial and the Ride score is already on the entity. What is missing is anywhere in the save format to say "mounted" and a `min(Ride, 3)` term on the two totals. Both are engine work; neither is expressible as a data edit, and **the V/F fixing round working from `docs/vf-audit/corrections.md` § 6.2 does not cover either**. Detail: `docs/book-template-conformance.md` § K5, § K3. | an engine capability each; (a) first, since its compromise is already shipped | 2026-09-24 |
| 53 | **The engine can grant an Ability authorization but cannot *restrict* one, so a Virtue offering "X **or** Y, not both" is modelled as offering both.** `virtue.wise_one` (`ArMDE:5257-5259` `#### Wise One`) lets the character "learn either Arcane or Academic Abilities, but not both"; `Effect::AbilityAuthorization` is purely additive, so whichever way it is encoded the character is permitted more than the passage allows. Found on the Witch companion template, which holds Wise One and buys from both groups without complaint. **This is the third member of the row-52 class** — an engine-capability gap sitting in a finding cluster whose Virtue/Flaw siblings *are* filed (row 50 covers the Witch's Educated defect), so the cluster looked covered. Not V/F data and therefore **outside the V/F fixing round's scope**, exactly like K3 and K5. **FOLDS INTO ROW 45** (agreed with the V/F audit session, 2026-09-24), the way row 48's authorization half does: Wise One needs "Arcane **or** Academic, not both" and `virtue.custos` needs a choice of one of Martial/Academic/Arcane, so both want the same **exclusive-choice parameter** on `Effect::AbilityAuthorization`. Row 45's survey has to build that parameter regardless, which makes this a second consumer rather than a second feature — do not design it twice. **The audit's F-349 carries the sharp end**: that finding recorded only "a two-category gated Ability permission" and *not* that the permission is exclusive, so the obvious remedy — authorize both categories — turns an under-permission into an **over-permission**. The Witch companion template is the proof. A fix that makes things worse, so read F-349 before touching either row. Detail: `docs/book-template-conformance.md` § W2. | folded into row 45's engine work | 2026-09-24 |
| 54 | **Nine conversion defects in the Core Rules Markdown, found by transcribing the 24 example characters — to be fixed UPSTREAM in `arm-de-translation`, not here.** *Venue corrected 2026-09-24 on Norbert's ruling: "The version in arm de translation is newer and with a different line count. Changing here does not make sense."* This row twice said the wrong thing before settling — first that the ninth item "waits for the re-sync" (it is forbidden, not deferred), then that the other eight were "safe in-place edits for this repo" (they are throwaway). **`rules/source/` is a copy**: the authoritative original for the **English** text as well as the German is `arm-de-translation/original-english/reviewed/…Core Rules.md`, which is newer and **80 lines longer** (25,803 against our 25,723, measured 2026-09-24 — our count independently confirmed). A hand repair here is replaced wholesale at the next sync and the edit goes with it. So all nine are **recorded here and filed there**, as a todo in `arm-de-translation/docs/todo.md`. The findings themselves are undiminished and survive a re-sync, because each is identified by what is *wrong* rather than by where it sits — and the `ArMDE:1733` case is provable rather than inferred (see (b)). All are Markdown-conversion artefacts, not rules content. **(a) A lost line break:** `ArMDE:1277` reads "Virtues and Flaws: Covenfolk, Warrior, Pessimistic Personality Traits: Brave +3, …" — the Hunter's "Personality Traits:" heading has been swallowed into the Virtues line, running the Flaw *Pessimistic* into the trait *Pessimistic*. **(b) A wrong Art abbreviation:** `ArMDE:1733` prints `Ag 0` in the Criamon's Arts line; there is no Art "Ag", the slot is Aquam, and the line carries 15 slots with only 14 valid abbreviations. **(c) Missing separators:** `ArMDE:1766` (`Giant Blood\* Major Magical Focus`, no comma), `ArMDE:1928` (a period for a comma, and `Music 4+2 (singing) Native Language`), `ArMDE:1954` and `ArMDE:2002` (`Per 0 Pre 0`), `ArMDE:1439` (a period for a comma). **(d) Stray characters:** `ArMDE:2548` (`Puissant Art Perdo)`, unmatched parenthesis), `ArMDE:1704`/`ArMDE:2153` (`Sta + 2`, space inside the value), `ArMDE:1480` (`Etiquette (noble) 3`, parameter before the score against the declared format at `ArMDE:1177`). **(a) IS FORBIDDEN TO FIX, NOT DEFERRED — and this row said "waits for the re-sync" until 2026-09-24, which was wrong.** Repairing it needs a line **inserted**, and `CLAUDE.md` now carries Norbert's invariant: *"We don't change number of lines in the source file… never."* An inserted or removed line renumbers everything below it and invalidates every citation into that file at once — `SourceRef` ranges in `rules/core/` and `rules/i18n/`, `// Source: ArMDE:NNNN` comments, `RULES.md`, and every dated record under `docs/` — and **nothing detects it**, because `rulebook_citations.rs` checks only that a cited range lands on non-blank lines, so after a shift every citation still **passes** while pointing at the wrong text. Silent, total and green: the worst failure shape available. So (a) is never repaired **in this repo** under any schedule — and as of the venue correction above it is not a local defect at all, but an upstream one like the other eight. **(b)-(d) keep their line count**, so they would have been mechanically safe here; they are still not to be done here, for the throwaway reason above. **Owner: the V/F audit session** — it is row 39's class, they have done that shape twice, and they own `arm-de-translation`'s todo file. **Blocked on Norbert confirming in *their* session**, since filing into a different repository raises the same authorization question and a relayed approval is not the same object as a direct one. Several entries also have German counterparts in the line-parallel `rules/source/de/`, whose upstream original needs the same check. Detail: `docs/book-template-conformance.md` § H1, § MAG3, § K4, § D3, and the per-section formatting notes. | (b)-(d) an implementation pass; (a) nothing, permanently | 2026-09-24 |
| 55 | **The book-template conformance suite added 208 `ArMDE:NNNN` citations to live test code and anchored only 13 of them, so a re-sync silently repoints 195 comments that explain *why* each expectation holds.** Self-reported debt from the 2026-09-22/24 pass. The citations are correct against our copy today and `rulebook_citations.rs` verifies them — which is precisely the failure shape D30 names: after an upstream sync every one still **passes**, because the guard checks only that a range lands on non-blank lines, while the text it names has moved. Row 54 measured the drift that makes this concrete rather than theoretical: the authoritative `arm-de-translation` original is already **80 lines longer** than our copy. **Scope is `crates/arm-rules/tests/book_templates.rs` only.** The sibling `docs/book-template-conformance.md` carries 359 citations with 63 anchored and is **deliberately left alone**: `source_citations.rs` puts `docs/` outside the guard on purpose, and anchoring a dated snapshot is tidying a record. **But "dated record versus live code" is the wrong axis for judging the risk, and the right one is recoverable versus not** (the V/F audit session's framing, 2026-09-24, from the same measurement on their side — ~4,180 citations in `docs/vf-audit/`, 470 in the three files a Phase 2 session works from). A citation to a **catalogue entry's own passage** survives a re-sync wherever it sits, because all 655 V/F entries carry a `source.anchor` in both stores — that is what D30 bought. A citation to a passage with **no entry** is unrecoverable, and that is the class to act on. This test file is dense with them, because it cites the *rules text* rather than the catalogue: the apprenticeship budget (`ArMDE:2435`), the template format convention (`ArMDE:1163`), the Characteristic point-buy (`ArMDE:2342`), the age caps (`ArMDE:2368-2374`), the Load table (`ArMDE:17107`) and the Initiative formula (`ArMDE:16658`) have no entry to anchor to. Nobody has counted the entry-less citations in either file. The fix is to carry the owning `####` heading beside each citation, as the conformance doc already does for its section headers — most of these point at statblock lines inside a template, so the durable form is the template's own heading (`ArMDE:1212` becomes `ArMDE:1212` `#### The Berserker`). Mechanical, and cheapest done in one pass rather than discovered piecemeal. Related: row 43 (the catalogue anchor rollout), D30. | an implementation pass (test comments only) | 2026-09-24 |

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
