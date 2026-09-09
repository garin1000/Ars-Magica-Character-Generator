# Open to-dos

Items waiting on a decision, a visual check, or a follow-up pass. Kept here so
they survive a session ending. Findings that are merely *implemented* live in
their own findings document; this list is what is still owed.

**Surface this list when a release or a git tag is being prepared** — none of
these should be tagged over silently.

| # | Item | Waiting on | Raised |
|---|---|---|---|
| 8 | **GitHub issue #3 is fixed but not yet answered or closed on GitHub.** Finding 34 fixed the repeatable-Virtues bug it reported (Improved Characteristics and 24 others); the issue itself still needs a reply and closing. | reply + close on GitHub | 2026-09-06 |
| 9 | **Per-power targets are unverified strings.** `flaw.slow_power`, `flaw.restricted_power` and `virtue.variable_power` now name the power they limit, but nothing checks the typed name against a power the character actually holds — Greater, Lesser, Personal, Ritual and Focus Power are themselves unparameterized and repeatable, so their rows are anonymous and there is nothing to point at. Giving those five a free-text `name` parameter would make them addressable and let the picker offer a dropdown; larger change, own save impact. Narrowed from the old row 7. | decision | 2026-09-06 |
| 11 | **False Power's copies do not name which Supernatural Virtue they taint.** ":6096" says the Flaw is taken "once for each appropriate Supernatural Virtue that the character possesses", so the copies must name *different* Virtues; none names any. Expressing it needs a parameter domain meaning "an item of category X that this character possesses" — `ParameterDomain::Item` has no category narrowing and no is-possessed predicate, so the picker would list every item in the catalogue. Narrowed from the old row 4. | decision | 2026-09-06 |
| 12 | **Folk Magic's realm alignment is unmodelled.** The category axis is now a dropdown, but each copy *also* aligns to a `(Realm) Lore`, re-choosable per copy, "although a character cannot have access to both the Divine and Infernal Realms" (:3919, the tail of the same line that grants the repeat). Neither the second axis nor its exclusion is expressed. Narrowed from the old row 6. | decision | 2026-09-06 |
| 13 | **One Virtue/Flaw still carries a dual-category descriptor joined by *and*/*or* while shipping one category.** `7ea4f5b` resolved three of the original five — Offensive to (Beings) and Unbearable to (Beings) became `["general"]` with real eligibility gates, and Inoffensive to (Beings) gained the gate it was missing. Primogeniture Lineage is now closed too, from the other end: its real restriction was never a category, and it is enforced as `prerequisites: All([IsMagus, House(house.verditius)])` — see "Done since" below. What remains: :5882 `flaw.curse_of_slander` "General or Supernatural" (ships `["general"]`). It is not a wrong-output bug the way the two Flaws were — `general` is on every profile's permitted list and no profile's forbidden list, so Curse of Slander is blocked for nobody, and adding `supernatural` beside it would change no outcome. What is still owed is the *reading*: "General or Supernatural" looks like an either/or origin rather than dual membership, and the engine has no notion of a Virtue **taken as** one of its categories (`:5083` makes exactly that an explicit player choice for Sufi), so a second category would make the bearer count as holding a Supernatural Flaw for every `has_category` rule — caps, Gift categories, grant constraints. That flatness is row 19; whether *or* means one route or both is the decision owed here. | decision | 2026-09-06 |
| 15 | **Three spell descriptions now end on a colon that introduces a table they cannot carry.** The leaked Markdown table syntax is fixed: `scripts/extract_spells.py` no longer folds table rows into a description, the three affected entries were re-extracted (German prose repaired by hand), and `no_spell_description_carries_markdown_table` in `data_integrity.rs` guards it. What is left is a wording question, not a defect: `spell.the_shadow_of_life_renewed` (both locales) now ends "… to determine the success of the attempt:" and `spell.mists_of_change` has the same dangling colon mid-description, because the roll table those sentences announce lives only in `rules/source/<lang>/`. Options: leave it (faithful to the source), or teach the UI to render a spell's table from the source. Note two other descriptions legitimately end without punctuation because the rulebook does (`spell.notes_of_a_delightful_sound` :14676, `spell.scent_of_peaceful_slumber` :15171) — those must not be "corrected". | decision | 2026-09-07 |
| 16 | **Mismatched German quote glyphs are the German corpus's house style, and `rules/source/de/` stays as published.** Measured 2026-09-09 with `rg -a --count-matches` over `rules/source/de/`, after this pass's own edits: **1425** occurrences of `„…"` — opening with `„` (U+201E), closing with an ASCII `"` — against **6** correctly paired `„…“` and **0** genuine inverse pairs. Split: **1345** across all **7** German rulebook files (Basisregeln 309, Mysterienkulte 240, Heckenzauber 230, Societates 190, Wahre Linien 128, Rhein-Tribunal 125, Sphären der Macht — Magie 123) and **80** across 10 files under `translation-tables/` (grundbegriffe 30, tugenden-fehler 12, that directory's own README 12, sphären-mächte 9, orden-tribunale 5, konvent 5, reputationen 4, zauber-nach-form/tiere-kreaturen/magie-regeln 1 each). All **6** correct pairs sit in *Sphären der Macht — Magie*, on five lines: :4495, :4547, :4623, :4681 and :4885, which carries two. That doubled line is also why a naive `“…„` scan reports one "inverse" hit — it matches the gap *between* the two correct pairs, so the genuine inverse count is zero. Raw glyph totals corroborate: 1451 `„`, 6 `“`, 0 `”`, 1544 ASCII `"`. At 1425-to-6 this is the corpus's convention, not a slip, so the earlier reading of `Basisregeln.md:6232` as a one-off defect was wrong. **Decision: the source is not touched** — `rules/source/de/` reproduces the books as printed, and a 1425-site sweep would rewrite the CC-BY-SA rules text over a typographic preference. The policy instead: if an extraction ever carries quoted text past the first sentence, it normalizes the pairing **on the way out**, in the extractor, so `rules/i18n/de/` is clean without the source moving. Nothing shipped reaches such a quote today — every extracted summary stops at the first sentence. The row stays open as the recorded policy, not as work owed. | recorded policy — no source change | 2026-09-07 |
| 17 | **What an older save still reports on open, and why each one is correct.** `e443aaf` recovers what is mechanically recoverable from a character file written before this round of ruleset changes (`8801b03`, `bb305fd`, `2380322`, `b86889c`), and "such a save opens clean" is true of the **format** changes only. Three findings survive on purpose. Nothing is corrupted in any of them — saves store choices, not resolved values, and the engine only reports. (a) **`too_many_selections`** is a genuine rules violation, not a format problem: two bought Puissant Arts plus a House grant really do exceed the ceiling the descriptor states, the engine was simply blind to it before `8801b03`/`bb305fd`, so it survives migration by design and the character has to lose a copy. (b) **`missing_param`** on Folk Magic's `category` and on the three per-power Flaws' `power` (`flaw.slow_power`, `flaw.restricted_power`, `virtue.variable_power`): neither key was ever *stored*, so there is nothing to migrate from, and filling a placeholder would be inventing someone's rules choices — one pick each clears it permanently. (c) **`prereq_not_met`** where `7ea4f5b`'s new eligibility gates bite — a character holding The Gift plus Offensive to (Beings) without the Gentle Gift, which ":6530" has always forbidden and nothing checked. That is not a hypothetical: it is the exact shape of `e443aaf`'s own migration fixture (`V0_2_X_MAGUS_SAVE` in `crates/arm-rules/tests/data_integrity.rs`), which is how it came to light. Recorded here so the three are explainable rather than mistaken for regressions; there is no fix owed for any of them. | nothing owed — recorded so an older save's findings are explainable | 2026-09-09 |
| 18 | **`gift_categories` serves three masters, and "Hermetic Flaw" is not one of them.** `effective::has_the_gift` reads `gift_categories` (`["hermetic"]` on every shipped profile) for Gift detection, `supernatural_free_slots` for the Gift's free Supernatural-Ability slot, and `validate_house` for the ":2860" guideline that a magus should take at least one Hermetic Flaw. Only the first two forced `7ea4f5b` to drop `hermetic` from Offensive and Unbearable to (Beings); the third took collateral damage, so a magus whose only Hermetic Flaw is one of those two is now warned he has none, though the book indexes both under Hermetic (":5445", ":5455"). The guideline wants its own notion of "Hermetic Flaw", independent of Gift detection. Related and unresolved: `Prereq::Has(virtue.the_gift)` is **id**-based while `has_the_gift` is **category**-based, so a Suppressed-Gift companion is Gifted by the engine's reckoning (and by ":6805") yet satisfies Offensive's `Nor([Has(the_gift)])` arm without the Gentle Gift and fails Unbearable's Gift arm — both readings diverge from a literal ":6530" / ":6895". Recorded by `7ea4f5b` in `crates/arm-rules/RULES.md`, not fixed. | decision | 2026-09-09 |
| 19 | **`categories: Vec<String>` is flatter than the descriptors it stores.** Two distinctions the book draws and the model cannot: (a) a Virtue **taken as** one of its categories — ":5083" makes it an explicit player choice ("either as a Minor Social Status Virtue **or** a Minor Supernatural Virtue"), and since `7ea4f5b` a grog may be a mundane Sufi (":5079"), but the engine has no "taken as", so that grog still counts as holding a Supernatural Virtue for every `has_category` rule: caps, `gift_categories`, grant constraints; (b) the *and* / *or* / comma join in a dual-category descriptor — a `Vec` cannot express it, and whether *and* means "either route" (what the engine now assumes) or "both, hence both categories' restrictions" is a genuine interpretive question the book does not settle. Row 13's two remaining items sit on (b). | decision | 2026-09-09 |
| 20 | **A type profile's category lists are unconditional.** ":2840" bars a companion from Hermetic Virtues and Flaws "**unless you have The Gift** (this would be highly unusual)"; the companion profile forbids `hermetic` flatly, so a Gifted companion is refused the exception the book grants. Nothing in the model expresses a conditional category — `permitted_categories`/`forbidden_categories` are plain slug lists, with no place to hang a `Prereq` on a category the way an item can hang one on itself. (The row's second half, the unsourced grog `supernatural` restriction, is resolved — see "Done since" below.) | decision | 2026-09-09 |

## Done since this list was started

- Primogeniture Lineage's House restriction is enforced (part of old row 13).
  `:6636` reads "This Flaw can only be taken by magi of House Verditius, as a
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
  (part (b) of old row 20). `:2822-2830` is the grog guidelines in full — up to 3
  points of Flaws and an equal number of Virtues, must take one Social Status,
  should not take Story Flaws, not more than one Personality Flaw, may not take
  Major Virtues or Flaws, may not take Hermetic Virtues and Flaws, may not take
  The Gift — and Supernatural appears nowhere in it; `:1009` likewise; the
  `### Supernatural` prose at `:2958-2962` explains realm association and Warping
  immunity and sets no character-type restriction. **Both halves were removed**,
  because the restriction was encoded twice: `supernatural` left
  `forbidden_categories` *and* joined `permitted_categories`. Permitting is an
  ANY test, so dropping only the forbid would have left every single-category
  Supernatural item refused with `category_not_permitted` — a change that looks
  like a fix and does nothing. `hermetic` stays forbidden; `:2829` sources it.
  **Blast radius: 113 shipped items newly clear a grog's category gate** — the
  111 carrying `["supernatural"]` alone (49 Minor Virtues, 26 Major Virtues, 1
  Free Virtue, 27 Minor Flaws, 8 Major Flaws) plus the two *Story, Supernatural*
  Flaws. Of those, 34 stay blocked by `:2828`'s Major cap and the two Story ones
  by `:2826`'s Story cap, and 4 more are unreachable anyway: `virtue.demonic_might`,
  `virtue.demonic_powers`, `virtue.strong_angelic_heritage` and
  `flaw.false_power_minor` are all Minor but each is `Has(...)` on a **Major**
  item (`virtue.demonic_blood`, `virtue.blood_of_the_nephilim`,
  `flaw.false_power`) that `:2828` denies a grog. So what genuinely opened is 72
  Minor items plus one Free Virtue, `virtue.commanding_aura`
  (`:3579-3583`) — the only zero-cost entrant, an "inherent benefit of Church
  office" the book puts no type restriction on. `:2824`'s budget, `:2828`'s Major
  cap and `:2830`'s Gift policy are all confirmed still working by tests in
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
  the line-mirrored `rules/source/en/…Core Rules.md:4133`). `rules/source/de/`
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
  power, no limit across different powers — `2380322`, 2026-09-06. What the
  typed name still does not guarantee is row 9 above — that it names a power the
  character actually holds. It is at least trimmed and non-blank now; see the
  old row 10 entry at the head of this list.
- False Power repeats, and every copy after the first is Minor (old row 4).
  `flaw.false_power` keeps its id and stays Major with `max_total: 1`;
  `flaw.false_power_minor` requires it and repeats, so Major plus two Minors
  costs 3 + 1 + 1 — `7b95ca0`, 2026-09-06. Which Virtue each copy taints is
  row 11 above.
- Closed lists in the rulebook are closed lists in the picker (old row 6).
  `ParameterDomain::Enumerated` carries its values on the `ParameterDef`, so the
  three (Beings) items became dropdowns and Folk Magic repeats once per
  category with no ceiling written anywhere — `b86889c`, 2026-09-06. Folk
  Magic's second axis is row 12 above.
- Twenty truncated Virtue/Flaw summaries got their endings back (old row 15).
  The 240-character cap an earlier session's extraction applied had cut sixteen
  entries mid-word — twelve in German only, four in both locales. Each is the
  full first sentence again, re-extracted from the `source` range the core entry
  already records, with no replacement limit. A guard in `data_integrity.rs` now
  fails on any summary that does not end on sentence punctuation, naming every
  offender and its locale in one run — `6f06679`, 2026-09-07. The same class of
  defect in `spells.json` is row 15 above.
- The visual pass on the audit-tier work came back green (2026-09-09). Checked
  in the running app: the at-cap Virtue/Flaw row dims with its reason tooltip at
  the current type scale, and the new enumerated dropdowns — Folk Magic's four
  categories and the three (Beings) lists — render and fit in both locales. No
  layout or label changes owed, so `max_total`'s UI mirror and
  `ParameterDomain::Enumerated` are closed end to end.
