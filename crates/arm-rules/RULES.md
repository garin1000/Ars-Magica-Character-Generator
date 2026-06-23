# Rules implementation map

Traceability between the authoritative rulebook Markdown in `rules/source/en/`
and the engine that implements it. **English is the source of truth** for IDs
and rule text; line numbers below are 1-based, inclusive, and refer to the
English source files.

This file is the binding contract behind two project rules (see CLAUDE.md →
*Rules provenance*):

1. **Rules are backed by source, never memory.** A mechanic exists in code/data
   only if its passage exists in `rules/source/<lang>/`. Verify every line range
   against the actual file before recording it here.
2. **Cite by book.** Every entry names the source **file basename** (the book);
   bare line numbers are meaningless across books. JSON files cannot carry
   comments, so this file is the provenance home for rule *values* encoded as
   data (e.g. character-type budgets in `rules/core/character_types.json`).

Organization is **by book**, then by category. Only books with implemented
mechanics carry entries; the rest are stubbed at the end.

---

## Ars Magica - Definitive Edition (Core Rules).md

### Hardcoded engine values

#### Magnitude point weights — Free 0, Minor 1, Major 3
> "Virtues and Flaws are either Minor or Major. Virtues cost points, while Flaws
> grant points. Major Virtues cost three points, while Major Flaws grant three
> points. Minor Virtues and Flaws cost and grant (respectively) one point."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2774` (also stated at
  `:2209`). Free (0-point) virtues: `:2886-2896`.
- Implementation: `crates/arm-rules/src/types.rs` — `Magnitude` / `Magnitude::points()`.

#### Virtue/Flaw polarity — Virtues cost, Flaws grant
> "Virtues cost points, while Flaws grant points."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2774`.
- Implementation: `crates/arm-rules/src/types.rs` — `ItemKind::is_positive()`.

### Enforcement logic

#### Virtue/Flaw balance — Virtues must be funded by Flaws
> "Players start with no points for buying Virtues and Flaws, and thus must take
> Flaws if they want Virtues. A central character may have up to ten points of
> Flaws ... and the same number of points of Virtues."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2774`, `:2297`
  (companions), `:2303` (magi).
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_balance`,
  `compute_balance`. Emits `unbalanced_virtues` (error) when spent virtue points
  exceed flaw points granted, plus the `over_budget_*` totals. (Per-type point
  totals are data; see below.)

#### Caps on Major virtues/flaws count
> "You may not have more than one Major Hermetic Virtue" (magi);
> "You may not take Major Virtues or Flaws" (grogs).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2857` (magi),
  `:2824-2830` (grogs).
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_caps`
  (counts items with `magnitude == Major`; cap value is data).

#### Cap on Minor Flaws count (hard)
> "A central character may have up to ten points of Flaws, but no more than five
> Minor Flaws."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2774`; companion
  `:2835`, magus `:2856`; grogs "no more than three Minor Flaws" `:1009`.
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_caps`, error
  `too_many_minor_flaws` (counts `magnitude == Minor` flaws; cap value is data,
  `max_minor_flaws`).

#### Per-category flaw caps (Personality hard + Personality/Story soft)
> "A character may not have more than one Major Personality Flaw." (`:2820`)
> "A character should normally not have more than two Personality Flaws in
> total" (`:2820`); "A character should not have more than one Story Flaw"
> (`:2818`); grogs "should not have Story Flaws" (`:1009`).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2820` (Major
  Personality, hard; restated per type at companion `:2838`, magus `:2851`),
  `:2976` (Personality total), `:2818`/`:2982` (Story), grogs `:1009`/`:2826`.
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_caps`. These
  caps are **fully data-driven**: each entry of the profile's
  `flaw_category_caps` (`PointBudget.flaw_category_caps`, type
  `FlawCategoryCap { category, max, major_only, hard }`) names the flaw
  **category as data**, so the engine hardcodes no category slug. A `hard` cap
  is a blocking error; otherwise a non-blocking warning (the book marks the
  Personality/Story guidelines troupe-overridable). The issue `code` is derived
  from the category as `too_many_<category>_flaws` (or
  `too_many_major_<category>_flaws` when `major_only`), so the shipped
  `personality`/`story` caps map onto the existing Fluent keys
  (`too_many_major_personality_flaws`, `too_many_personality_flaws`,
  `too_many_story_flaws`) by convention rather than a baked-in mapping.

#### The Gift policy — required / forbidden by type
> "all magi must have this Virtue" ... "Grogs can never have The Gift".

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2868-2877` (The Gift),
  `:2858` (magi must take The Gift + Hermetic Magus status), `:2293` and
  `:4067-4069` (only magi may take the Hermetic Magus Social Status).
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_gift_policy`.
  The Gift policy is independent of the `is_magus` flag (an unGifted Redcap is a
  companion; a Gifted hedge wizard is not a magus).

#### Prerequisite evaluation (meta-mechanic)
- The tri-state `Prereq` evaluator (`crates/arm-rules/src/validation.rs` —
  `evaluate_prereq`) is engine infrastructure, not a single rulebook passage. It
  enforces book requirements expressed as data, e.g. "all magi must take the
  Hermetic Magus Social Status" (`:2293`), encoded as a `Prereq` on the relevant
  items.

### Data-driven rule values — `rules/core/character_types.json`

The point budgets and caps per character type are data, not Rust. Each value's
source:

| Type | Value | Source |
|------|-------|--------|
| grog | `virtue_points: 3`, `flaw_points: 3` | `:2295`, `:2824-2830`, `:1009` |
| grog | `max_major_virtues: 0`, `max_major_flaws: 0` | `:2824-2830` ("may not take Major Virtues or Flaws"), `:1009` |
| grog | `max_minor_flaws: 3` | `:1009` ("no more than three Minor Flaws") |
| grog | `flaw_category_caps`: personality major_only/hard `max: 0`; personality `max: 1`; story `max: 0` | grogs take one Minor Personality Flaw, no Major Flaws, no Story Flaws `:1009`, `:2824-2830` |
| companion | `virtue_points: 10`, `flaw_points: 10` | `:2297`, `:2834-2840` |
| companion | `max_major_virtues: null`, `max_major_flaws: null` (no count cap) | no Major-count cap for companions in the book |
| companion | `max_minor_flaws: 5` | `:2774`, `:2835` |
| companion | `flaw_category_caps`: personality major_only/hard `max: 1`; personality `max: 2`; story `max: 1` | `:2820`, `:2838` (Major Personality hard); `:2820`/`:2976` (Personality total); `:2818`/`:2837` (Story) |

Magus and mythic-companion profiles are not yet in `character_types.json`. When
added, cite: magus budget/caps `:2303`, `:2855-2863`; mythic companion
`:2638`, `:2844-2851`.

#### Resolved: companion `max_major_virtues`
Earlier data set companion `max_major_virtues: 1`. The book's "no more than one
Major Hermetic Virtue" (`:2857`) is a **magus** rule, and companions may not
take Hermetic Virtues at all (`:2834-2840`); the book defines **no** cap on a
companion's count of Major Virtues. The value was therefore corrected to `null`
(no cap).

---

## Engine framework (book-agnostic, no rulebook source)

These checks are structural integrity, not Ars Magica rules, and intentionally
carry no source citation:

- Incompatibility symmetry (`ruleset.rs` — `validate_incompatibility_symmetry`)
- Category permit/forbid (`validation.rs` — `validate_permitted_categories`,
  `validate_forbidden_categories`)
- Required/forbidden traits (`validation.rs` — `validate_required_traits`,
  `validate_forbidden_traits`)
- Entity-kind applicability, parameter validation, duplicate-selection detection
  (`validation.rs`)

---

## Other available books — no implemented mechanics yet

The following English sources are present in `rules/source/en/` but no mechanics
from them are implemented yet. Add a section above (mirroring the Core Rules
layout) when mechanics from a book are implemented.

- Ars Magica 5e - Houses of Hermes - Mystery Cults.md
- Ars Magica 5e - Houses of Hermes - Societates.md
- Ars Magica 5e - Houses of Hermes - True Lineages.md
- Ars Magica 5e - Magic - Hedge Magic (Revised).md
- Ars Magica 5e - Realms of Power - Faerie.md
- Ars Magica 5e - Realms of Power - Magic.md
- Ars Magica 5e - Realms of Power - The Divine (Revised).md
- Ars Magica 5e - Realms of Power - The Infernal.md
