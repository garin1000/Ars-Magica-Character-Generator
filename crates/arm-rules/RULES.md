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

#### Virtue/Flaw balance — Virtues balanced by an equal value of Flaws
> "A central character may have up to ten points of Flaws ... and the same
> number of points of Virtues."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2774`, `:2297`
  (companions), `:2303` (magi).
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_balance`,
  `compute_balance`. (Per-type point totals are data; see below.)

#### Caps on Major virtues/flaws count
> "You may not have more than one Major Hermetic Virtue" (magi);
> "You may not take Major Virtues or Flaws" (grogs).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2857` (magi),
  `:2824-2830` (grogs).
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_caps`
  (counts items with `magnitude == Major`; cap value is data).
- **Not yet enforced:** the "no more than five Minor Flaws" limit (`:2774`,
  `:2835`, `:2856`) and the Personality/Story Flaw guidelines (`:2818-2820`).

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
| companion | `virtue_points: 10`, `flaw_points: 10` | `:2297`, `:2834-2840` |
| companion | `max_major_flaws: null` (no cap) | no Major-count cap for companions in the book |

Magus and mythic-companion profiles are not yet in `character_types.json`. When
added, cite: magus budget/caps `:2303`, `:2855-2863`; mythic companion
`:2638`, `:2844-2851`.

#### ⚠ Discrepancy to confirm
`character_types.json` sets companion `max_major_virtues: 1`. The book's
"no more than one Major Hermetic Virtue" (`:2857`) is a **magus** rule, and
companions may not take Hermetic Virtues at all (`:2834-2840`). The book defines
**no** cap on a companion's count of Major Virtues. This value appears
unsourced — confirm with the troupe whether it should be `null` (no cap) before
relying on it.

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
