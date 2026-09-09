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
| 10 | **Text-param values are compared byte-for-byte.** The duplicate key is exact `(item_ref, params)` equality (`crates/arm-rules/src/validation/selections.rs:135-140`), so "Wolf Shape", "wolf shape" and "Wolf Shape " read as three distinct targets, and an empty string is accepted — the per-power cap stops an honest mistake, not a determined evasion. Whether to trim/case-fold text parameter values is an engine-wide decision affecting every existing text param (`power`, `terrain`, `land`, `realm`, `role`, …). Related and verified today: `crates/arm-rules/src/types.rs:455-458` documents text values as "any non-empty value is legal", while `crates/arm-rules/src/validation/selections.rs:277` (`ParameterDomain::Text => true`) enforces no such thing. | decision | 2026-09-06 |
| 11 | **False Power's copies do not name which Supernatural Virtue they taint.** ":6096" says the Flaw is taken "once for each appropriate Supernatural Virtue that the character possesses", so the copies must name *different* Virtues; none names any. Expressing it needs a parameter domain meaning "an item of category X that this character possesses" — `ParameterDomain::Item` has no category narrowing and no is-possessed predicate, so the picker would list every item in the catalogue. Narrowed from the old row 4. | decision | 2026-09-06 |
| 12 | **Folk Magic's realm alignment is unmodelled.** The category axis is now a dropdown, but each copy *also* aligns to a `(Realm) Lore`, re-choosable per copy, "although a character cannot have access to both the Divine and Infernal Realms" (:3919, the tail of the same line that grants the repeat). Neither the second axis nor its exclusion is expressed. Narrowed from the old row 6. | decision | 2026-09-06 |
| 13 | **Five Virtues/Flaws carry a dual-category descriptor joined by *and*/*or* but ship with one category.** :4134 Inoffensive to (Beings) "General and Hermetic" (ships `["general"]`), :5882 Curse of Slander "General or Supernatural" (`["general"]`), :6525 Offensive to (Beings) "Hermetic and General" (`["hermetic"]`), :6635 Primogeniture Lineage "Story and Hermetic" (`["story"]`), :6892 Unbearable to (Beings) "Hermetic or General" (`["hermetic"]`). The dual-category sweep (`5729e6d`) covered only the four **comma**-joined descriptors (Sufi, Visions, Raised from the Dead, Suppressed Gift), so these were never in its scope rather than missed. The *or* cases raise a separate question: "General or Supernatural" reads as an either/or origin, not dual membership. | decision | 2026-09-06 |
| 14 | **The German rulebook and the curated glossary disagree on Inoffensive to (Beings).** The rulebook heads the entry "Für (Wesen) ungefährlich" (`rules/source/de/Ars Magica Definitive Edition Basisregeln.md:4133`); `rules/source/de/translation-tables/tugenden-fehler.md:180` gives "Unauffällig für (Wesen)". `b86889c` followed the table, since CLAUDE.md makes the tables canonical for i18n labels, but the two German sources still disagree and someone may want to reconcile them at source. | decision | 2026-09-06 |
| 15 | **Three spell descriptions now end on a colon that introduces a table they cannot carry.** The leaked Markdown table syntax is fixed: `scripts/extract_spells.py` no longer folds table rows into a description, the three affected entries were re-extracted (German prose repaired by hand), and `no_spell_description_carries_markdown_table` in `data_integrity.rs` guards it. What is left is a wording question, not a defect: `spell.the_shadow_of_life_renewed` (both locales) now ends "… to determine the success of the attempt:" and `spell.mists_of_change` has the same dangling colon mid-description, because the roll table those sentences announce lives only in `rules/source/<lang>/`. Options: leave it (faithful to the source), or teach the UI to render a spell's table from the source. Note two other descriptions legitimately end without punctuation because the rulebook does (`spell.notes_of_a_delightful_sound` :14676, `spell.scent_of_peaceful_slumber` :15171) — those must not be "corrected". | decision | 2026-09-07 |
| 16 | **German quote glyphs are mismatched in the German source itself.** `rules/source/de/Ars Magica Definitive Edition Basisregeln.md:6232` writes `„Die Wunde, die blutet"` — opening with `„` (U+201E) and closing with an ASCII `"`. It sits in an entry's second sentence, so no shipped summary reached it, but any future extraction that goes past the first sentence would carry the mismatch into the data, and the pattern may recur elsewhere in the file. | decision | 2026-09-07 |

## Done since this list was started

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
  typed name still does not guarantee is rows 9 and 10 above.
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
