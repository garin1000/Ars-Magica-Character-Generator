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

### Characteristics

#### Eight Characteristics
> "There are eight Characteristics in Ars Magica, each representing one of a
> given character's inborn attributes."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:1023-1025`.
- Implementation: `crates/arm-rules/src/characteristics.rs` — `Characteristic`
  enum (Int, Per, Str, Sta, Pre, Com, Dex, Qik).

#### Point-buy cost table + seven starting points — `rules/core/characteristics.json`
> "Characteristics are bought on the following table. You start with seven points
> to spend." Table: +3→6, +2→3, +1→1, 0→0, −1→Gain 1, −2→Gain 3, −3→Gain 6.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2340-2354`.
- Data: `rules/core/characteristics.json` (`start_points: 7`, `costs`). The
  rulebook's "Gain N" rows are encoded as **negative** cost (`Gain 1` → `-1`,
  etc.) — an extraction sign convention. The legal score range (−3..+3) is
  derived from the table rows, not hardcoded.
- Implementation: `crates/arm-rules/src/characteristics.rs` —
  `CharacteristicRules` (`cost_for`, `total_cost`, `min_score`, `max_score`,
  `effective_max_score`); enforced in `validation.rs` —
  `validate_characteristics` (base out-of-range error, effective-ceiling error,
  overspent error, points-unspent warning). `effective_max` (=5, the Great
  Characteristic ceiling) is data in `characteristics.json`; see the
  effective-score layer below.

### Abilities

#### Ability XP advancement table ("ABILITY To Buy") — `rules/core/abilities.json`
> Advancement Table, "ABILITY To Buy" column: total XP to reach a score from
> zero — 1→5, 2→15, 3→30, … (triangular 5·n·(n+1)/2), through 20→1050.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2406-2427` (header
  `:2406`, data rows `:2408-2427`).
- Data: `rules/core/abilities.json` `advancement` array (scores 1-20).
- Implementation: `crates/arm-rules/src/ability.rs` — `AdvancementTable`
  (`xp_for_score`, `xp_to_raise`, `max_score`). Used to price whole-point steps;
  a character stores whole bought scores plus an `xp_pool` total (`types.rs`).
  The XP spent (Σ `xp_for_score`) may not exceed the pool — `validate_abilities`
  emits `not_enough_xp` otherwise (an error in Advisory/Enforced; the M4 wizard
  blocks the spend up front). Leftover pool is the character's banked XP.
- Engine integrity check (not a sourced rule): a non-zero ability score with no
  row in the advancement table (`xp_for_score` → `None`) is off-table and
  flagged `ability_score_out_of_range` by `validate_abilities` rather than
  silently priced at 0 XP. This mirrors the characteristic out-of-range check
  (the rulebook gives no explicit ability-score ceiling; the table's highest
  priced score — `max_score` — is used as the upper bound, lower bound 0). The
  check is skipped when the ruleset ships no advancement table.

#### Seed Ability catalogue — `rules/core/abilities.json`
> Ability list grouped by type (General, Academic, Arcane, Martial,
> Supernatural) at `:7177-7268`; alphabetical descriptions at `:7269-7786`.

- Source: each ability cites its description line range in `abilities.json`
  (e.g. Awareness `:7325-7328`, Magic Theory `:7646-7649`). The five categories
  are the book's Ability types (`:7177-7268`).
- Data: 23 seed abilities covering all five categories and the full early-childhood
  restricted list (`:2378`: Area Lore, Athletics, Awareness, Brawl, Charm, Folk
  Ken, Guile, Living Language, Stealth, Survival, Swim). Native language is a
  *specialty* of `ability.living_language`, not a separate id (`:2378`).
- Implementation: `crates/arm-rules/src/ability.rs` — `Ability`,
  `AbilityCategory`; registry + integrity (`AbilityMin`, `ability`-domain params
  resolve against it) in `ruleset.rs`; `validate_abilities` in `validation.rs`.

### Effective-score layer (score-boosting Virtues)

A character's *effective* score is the bought score plus the bonuses granted by
score-boosting Virtues. Effective scores are always computed, never stored, and
are used for `AbilityMin` prerequisites, the characteristic ceiling, and display.
The mechanic is **data-driven**: a `PointItem` declares `effects` (an `Effect`
list) and the target ability/characteristic is named by the selection's
parameter value — the engine hardcodes no Virtue IDs. Computed in
`crates/arm-rules/src/effective.rs` (`ability_bonus`, `characteristic_bonus`,
`effective_ability_score`, `effective_characteristic`, plus `*_bonuses` maps for
the UI).

#### Puissant (Ability) — +2 to one Ability
> "You are particularly adept with one Ability, and add 2 to its value whenever
> you use it … You may only take this Virtue once for a given Ability, but may
> take it more than once for different Abilities."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4814-4816`.
- Data: `rules/core/virtues_flaws.json` `virtue.puissant_ability` —
  `effects: [{ ability_bonus, param: "ability", amount: 2 }]`; `max_per_target`
  defaults to 1 ("only once for a given Ability").
- Implementation: `effective.rs::ability_bonus` sums it; `validation.rs`
  folds it into the score map so `AbilityMin` is met by the effective score.

#### Great (Characteristic) — +1, base ≥ +3, up to +5
> "You may raise any Characteristic that already has a score of at least +3 by
> one point, to no more than +5 … You may take this Virtue twice for the same
> Characteristic, and for more than one Characteristic."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:3987-3989`.
- Data: `rules/core/virtues_flaws.json` `virtue.great_characteristic` —
  `characteristic`-domain param; `effects: [{ characteristic_bonus, param:
  "characteristic", amount: 1, min_base: 3 }]`; `max_per_target: 2`. The +5
  ceiling is `effective_max` in `rules/core/characteristics.json`.
- Implementation: `effective.rs::characteristic_bonus` /
  `effective_characteristic`; `validation.rs::validate_characteristics` flags
  effective > 5 (`characteristic_effective_out_of_range`);
  `validate_characteristic_bonuses` flags a target base < `min_base`
  (`characteristic_bonus_base_too_low`) — the "≥ +3" precondition is
  parameter-relative, so it lives on the effect, not in the static `Prereq`.

#### Selection multiplicity — `max_per_target`
The per-`(item, params)` selection cap. `validate_duplicate_selections`
(`validation.rs`) errors `duplicate_selection` when a target's count exceeds the
item's `max_per_target` (default 1; Great Characteristic 2). This generalizes the
former hardcoded "at most once" rule and enforces both "Puissant once per
Ability" (`:4816`) and "Great twice per Characteristic" (`:3989`). Effect
integrity (`ruleset.rs::validate_effect_refs`) rejects at load any effect whose
`param` is undeclared or whose domain mismatches the effect kind.

#### Deferred to M4/M5
The life-stage XP acquisition (early childhood 75+45 xp `:2378`; later life
15/20/10 xp/yr `:2390-2394`; age→max-score cap `:2368-2374`) and the Sample
Childhood packages (`:2380-2388`) are deferred to M4. *Improved Characteristics*
(a +3 point-buy pool, not an effective-score bonus) is a separate
characteristic-budget follow-up. Puissant Art (+3) / `House` / `ArtMin`
evaluation and the Art registry are deferred to M5 (no Art registry yet).

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
