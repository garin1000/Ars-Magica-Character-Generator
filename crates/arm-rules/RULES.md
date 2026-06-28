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
  These weights are also surfaced to the UI verbatim as `Ruleset.magnitude_points`
  (derived from `Magnitude::points()` in `crates/arm-rules/src/ruleset.rs`); this
  introduces no new rule value — it is the same number, exposed so the frontend
  reads it from the engine instead of re-hardcoding it.

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
> "You may not take Major Virtues or Flaws" (grogs).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2824-2830` (grogs).
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_caps`
  (counts items with `magnitude == Major`; cap value is data via
  `max_major_virtues` / `max_major_flaws`).
- **Deferred to M4/4b (not yet implemented):** the magus rule "You may not have
  more than one Major Hermetic Virtue" (`:2857`) is a *category-restricted* cap
  (Hermetic Major Virtues only), not a plain Major-count cap. `PointBudget` has no
  virtue-category cap and no magus profile exists yet; M4/4b adds a data-driven
  `virtue_category_caps` (mirroring `flaw_category_caps`) and the magus profile.
  See the "Resolved: companion `max_major_virtues`" note below.

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
| magus | `virtue_points: 10`, `flaw_points: 10` | `:2303` ("Like companions, magi may take up to ten points of Flaws, and the same number of points of Virtues"), `:2855` ("up to 10 points of Flaws, and an equal number of points of Virtues") |
| magus | `max_minor_flaws: 5` | `:2856` ("may not have more than 5 Minor Flaws") |
| magus | `is_magus: true`, `gift_policy: required`, `required_traits: [virtue.hermetic_magus]` | `:2858` ("must take The Gift and the Hermetic Magus Social Status Virtue"), `:2293` (only magi may take the Hermetic Magus Status) |
| magus | `flaw_category_caps`: personality major_only/hard `max: 1`; personality `max: 2`; story `max: 1` | `:2862` ("should not take more than two Personality Flaws, and may not take more than one Major Personality Flaw"); `:2861` ("should not take more than one Story Flaw") |
| mythic_companion | `virtue_points: 20`, `flaw_points: 10`, `virtue_points_per_flaw_point: 2` | `:2638` ("up to ten points of Flaws, and each point of Flaws is worth two points of Virtues. This produces a maximum of 21 points of Virtues and 10 points of Flaws") |
| mythic_companion | `forbidden_categories: [hermetic]`, `gift_policy: forbidden` | `:2637` (Mythic Companion status Virtues "are incompatible … with The Gift"); generated as Companions `:2635` |

Deferred for `magus` (later M4 phases, not in the Phase-1 profile):
`max ≤1 Major Hermetic Virtue` needs `virtue_category_caps` (Phase 4b, `:2857`);
the free Minor House Virtue (`:2859`) and the "≥1 Hermetic Flaw" guideline
(`:2860`) land with Houses (Phase 4b). The Mythic Companion's free Minor status
Virtue (`:2638`, raising the balanced max from 20 to 21) is M8 catalogue data;
the `virtue_points: 20` ceiling here is the balanced maximum without it.

#### Virtue/Flaw funding rate — `virtue_points_per_flaw_point`
Each Flaw point funds one Virtue point by default; Mythic Companions fund two.
Modelled as the data-driven `PointBudget.virtue_points_per_flaw_point` (default
1), applied in `validation.rs::validate_balance` (the `unbalanced_virtues` check
compares virtue points against `flaw_points * virtue_points_per_flaw_point`).
Source: Core Rules.md:2638.

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
> to spend." Printed table: +3→6, +2→3, +1→1, 0→0, −1→Gain 1, −2→Gain 3, −3→Gain 6.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2340-2354` (printed
  table), `:4105` (the +3 base cap "unless you take … Great Characteristic").
- Data: `rules/core/characteristics.json` (`start_points: 7`, `costs`,
  `base_max: 3`, `base_min: -3`, `effective_max: 5`, `effective_min: -5`). The
  rulebook's "Gain N" rows are encoded as **negative** cost (`Gain 1` → `-1`,
  etc.) — an extraction sign convention. The printed table stops at ±3=±6; the
  ±4/±5 rows (+4→10, +5→15, −4→Gain 10, −5→Gain 15) **continue the table's own
  triangular progression** (marginal cost of level n is n) so the scores Great /
  Poor (Characteristic) unlock can be priced — the rulebook does not print them.
  The **base** limits (±3) are the no-virtue buy range; the **effective** limits
  (±5) are the absolute ceiling/floor those virtues/flaws open. The legal table
  range is now −5..+5.
- Implementation: `crates/arm-rules/src/characteristics.rs` —
  `CharacteristicRules` (`cost_for`, `total_cost`, `min_score`, `max_score`,
  `base_max_score`, `base_min_score`, `effective_max_score`,
  `effective_min_score`); enforced in `validation.rs` —
  `validate_characteristics` (off-table out-of-range error, above-cap /
  below-floor errors against the per-characteristic buy range, overspent error,
  points-unspent warning). See the Great/Poor (Characteristic) layer below.

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
  This pool is **shared with Arts** (see the Arts section): the combined Ability +
  Art cost may not exceed it — `validate_xp_pool` emits `not_enough_xp` otherwise
  (an error in Advisory/Enforced; the M4 wizard blocks the spend up front).
  Leftover pool is the character's banked XP.
- Engine integrity check (not a sourced rule): a non-zero ability score with no
  row in the advancement table (`xp_for_score` → `None`) is off-table and
  flagged `ability_score_out_of_range` by `validate_abilities` rather than
  silently priced at 0 XP. This mirrors the characteristic out-of-range check
  (the rulebook gives no explicit ability-score ceiling; the table's highest
  priced score — `max_score` — is used as the upper bound, lower bound 0). The
  check is skipped when the ruleset ships no advancement table.
- Load-time data-integrity invariant (not a sourced rule): the advancement table
  must have unique scores and a non-decreasing `total_xp` column (the "To Buy"
  totals only ever grow). This is the precondition that makes `xp_to_raise`'s
  step subtraction (`to - from`) sound. `AdvancementTable::validation_errors`
  enforces it from `Ruleset::validate_integrity`, so a malformed `advancement`
  array is rejected at load with a clear message naming the offending scores
  rather than wrapping/panicking on first use. `xp_to_raise` additionally uses
  `checked_sub` (returns `None`) as defense-in-depth for tables built outside the
  load gate.

#### Ability catalogue (seed set) — `rules/core/abilities.json`
> Ability list grouped by type (General, Academic, Arcane, Martial,
> Supernatural) at `:7177-7268`; alphabetical descriptions at `:7269-7789`.

> **Partial-catalogue note.** This branch ships the **23-ability seed set**, not
> the complete Core Rules catalogue. The full 78-ability catalogue (General 29,
> Academic 12, Arcane 12, Martial 4, Supernatural 21) — with all descriptions,
> specialties, and `requires_training` flags — already exists on the
> `full-abilities` branch and is **pulled over later in the plan** (it widens the
> data only; the engine, i18n schema, and UI here already support it). When you
> pull it in, restore the `78`/`21`/`50` counts in
> `tests/data_integrity.rs` and `commands.rs`. See also the project memory note
> on the deferred ability catalogue.

- Source: each ability cites its description line range in `abilities.json`
  (e.g. Awareness `:7325-7328`, Magic Theory `:7646-7649`). The five categories
  are the book's Ability types (`:7177-7268`), taken from each entry's trailing
  `(Type)` label.
- Data: 23 seed abilities covering all five categories and the full
  early-childhood restricted list (`:2378`: Area Lore, Athletics, Awareness,
  Brawl, Charm, Folk Ken, Guile, Living Language, Stealth, Survival, Swim).
  Native language is a *specialty* of `ability.living_language`, not a separate
  id. Parameterized abilities carry a `parameter` key (`area` for (Area) Lore,
  `language` for the (Living/Dead Language) abilities).
- The `*` marker (`requires_training` flag): an *asterisked* Ability cannot be
  used without at least one experience point in it — there is no untrained roll.
  > "Characters without this Virtue cannot even attempt rolls on an asterisked
  > Ability without at least one experience point in it." — Jack of All Trades,
  > `Ars Magica - Definitive Edition (Core Rules).md:4157`.

  This is an **independent per-ability property, not derived from `category`**:
  it spans General (e.g. (Area) Lore), Academic, Arcane, and all Supernatural
  abilities — 8 of the 23 seed abilities are asterisked. Notably Penetration and
  most combat/General abilities are *not* asterisked. The flag is set from the
  `*` on each ability's `####` heading (`:7273-7786`); `Ability.requires_training`
  (`ability.rs`) stores it and the UI renders the trailing `*` via the
  `ability-requires-training-marker` Fluent key. (The German translation table's
  `*` legend carries the same meaning; its `Typ` column is informational — the
  English `(Type)` label remains the source of truth for `category`.)
- i18n: `rules/i18n/{en,de}/abilities.json` carry per-ability `name`,
  a 1–2 sentence `description` (an authored condensation of the whole rulebook
  entry), and example `specialties`. German names follow the canonical
  translation table; German text is drawn from the same line range in
  `Ars Magica Definitive Edition Basisregeln.md` (which mirrors the English file
  line-by-line). `I18nEntry.specialties` (`types.rs`) holds the list;
  `LocalizedRuleset::specialties` exposes it.
- Implementation: `crates/arm-rules/src/ability.rs` — `Ability`,
  `AbilityCategory`; registry + integrity (`AbilityMin`, `ability`-domain params
  resolve against it) in `ruleset.rs`; `validate_abilities` in `validation.rs`.

### Arts

#### Art XP advancement table ("ART To Buy") — `rules/core/arts.json`
> Advancement Table, "ART To Buy" column: total XP to reach a score from zero —
> 1→1, 2→3, 3→6, 4→10, 5→15, … (triangular n·(n+1)/2), through 20→210. Cheaper
> than the Ability column (which is 5× these values).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2406-2427` (header
  `:2406`, data rows `:2408-2427` — the "ART To Buy" / "To Raise" columns).
- Data: `rules/core/arts.json` `advancement` array (scores 1-20).
- Implementation: reuses `crates/arm-rules/src/ability.rs` — `AdvancementTable`
  (the same generic table type as Abilities). Surfaced on `Ruleset` as
  `art_advancement`. A character stores whole bought Art scores
  (`Entity::art_scores`) priced from this table.
- **Shared XP pool.** Abilities and Arts are bought from **one** bank
  (`Entity::xp_pool`): the rules give apprenticeship experience as a single pool
  the magus splits freely between Arts and Abilities ("These experience points
  can be spent on Arts or Abilities", `:2429-2433`). So there is no Art-specific
  pool — `validate_xp_pool` sums the Ability cost (Ability table) and the Art cost
  (Art table) and emits `not_enough_xp` if the total exceeds `xp_pool`. The
  per-domain validators (`validate_abilities` / `validate_arts`) own only the
  structural checks (unknown/duplicate/off-table); `xp_spent` does the pricing.
- Engine integrity checks (not sourced rules): an off-table Art score is flagged
  `art_score_out_of_range`; the table's unique-score + non-decreasing-`total_xp`
  invariant is enforced from `Ruleset::validate_integrity` (same as the Ability
  table).

#### Art catalogue (15 Arts) — `rules/core/arts.json`
> The Hermetic Arts: 5 Techniques (Creo, Intellego, Muto, Perdo, Rego) and 10
> Forms (Animal, Aquam, Auram, Corpus, Herbam, Ignem, Imaginem, Mentem, Terram,
> Vim).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:8833-8982` — chapter
  intro `:8833`, Techniques `:8845-8897`, Forms `:8899-8982`. Each Art entry cites
  its own description line range (e.g. Creo `:8847-8863`, Ignem `:8941-8945`).
- Data: 15 Arts, each `{ id, art_type }` where `art_type` is `technique` or
  `form`. Arts are **not** parameterized and carry no specialty. The complete Core
  Art set, so unlike the ability seed this is the full catalogue.
- i18n: `rules/i18n/{en,de}/arts.json` carry per-Art `name`, two-letter
  `abbreviation` (Cr, In, … Vi), and a condensed `description`. Art **names stay
  Latin** in both languages (untranslated per `uebersetzungsregeln.md`); German
  descriptions are drawn from the same line range in `Ars Magica Definitive
  Edition Basisregeln.md`. EN↔DE Art glossary: `translation-tables/magie-regeln.md`.
- Implementation: `crates/arm-rules/src/art.rs` — `Art`, `ArtType` (fixed enum;
  `ArtType::ALL` surfaces `art_type_order` on `Ruleset`), `ArtsFile` loader.
  Registry + integrity (`ArtMin`, `art`-domain params resolve against it) in
  `ruleset.rs`; `validate_arts` in `validation.rs`.

### Effect layer (score-boosting Virtues, limit-shifting Virtues/Flaws)

Two `Effect` kinds, both **data-driven** (a `PointItem` declares `effects` and the
target ability/characteristic is named by the selection's parameter value — the
engine hardcodes no Virtue/Flaw IDs):

- `ability_bonus` — adds to an ability's *effective* score (bought + bonus,
  always computed, never stored), used for `AbilityMin` prerequisites and display.
- `characteristic_limit` — shifts a characteristic's *buy limit*. It grants no
  points: the score must still be bought against the cost table. A positive
  amount raises the cap (Great Characteristic), a negative one lowers the floor
  (Poor Characteristic).

Computed in `crates/arm-rules/src/effective.rs` (`ability_bonus`,
`effective_ability_score`, `ability_bonuses`; `characteristic_cap`,
`characteristic_floor`, and the all-eight `characteristic_caps` /
`characteristic_floors` maps for the UI). Ability bonuses are **per instance**
`(ability, parameter)`, not per id, so a Puissant on one (Area) Lore does not
bleed onto the character's other areas.

#### Puissant (Ability) — +2 to one Ability
> "You are particularly adept with one Ability, and add 2 to its value whenever
> you use it … You may only take this Virtue once for a given Ability, but may
> take it more than once for different Abilities."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4814-4816`.
- Data: `rules/core/virtues_flaws.json` `virtue.puissant_ability` —
  `effects: [{ ability_bonus, param: "ability", amount: 2 }]`; `max_per_target`
  defaults to 1 ("only once for a given Ability").
- **Targets one ability instance.** The selection stores the ability id under
  `ability`; for a *parameterized* ability ((Area) Lore) it also stores the
  instance value (the area/language) under the ability's own param key, e.g.
  `params: { ability: "ability.area_lore", area: "Brandenburg" }`. Each (Area)
  Lore is a distinct Ability (`:4816`), so "Puissant Brandenburg Lore" boosts
  that row alone, not "Berlin Lore".
- Implementation: `effective.rs::ability_bonus(.., parameter)` matches
  `(ability, parameter)` — a plain ability by id, a parameterized one only when
  the selection names the same instance; a selection missing the instance key
  matches nothing. `ability_bonuses` returns a per-instance `Vec<AbilityBonus>`
  (serialized directly to the frontend by `arm-app::ruleset_io`). `validation.rs`
  folds the per-instance bonus into the score map so `AbilityMin` is met by the
  strongest instance.
- Validation: `validate_parameters` makes the expected param-key set
  target-aware — a parameterized ability target also expects its instance key
  (else `missing_param`; a stray instance key on a plain target is
  `unexpected_param`). `validate_ability_bonus_targets` flags
  `ability_bonus_dangling_target` when the targeted `(ability, parameter)` is not
  among the character's bought abilities (e.g. the ability was later removed).

#### Puissant (Art) — +3 to one Art
> "You add 3 to the value of one Art whenever you use it. This means all totals in
> which the score of the Art is part of the total. … You may take this Virtue
> twice, for two different Arts."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4818-4820`.
- Data: `rules/core/virtues_flaws.json` `virtue.puissant_art` —
  `category: hermetic`, `art`-domain param, `effects: [{ art_bonus, param: "art",
  amount: 3 }]`; `max_per_target` defaults to 1 ("once for a given Art"). Taking
  it for two Arts is two selections with different targets (repeatable by the
  parameterized-item rule). Being Hermetic, only magus profiles permit it.
- The `art_bonus` effect adds to an Art's *effective* score (Arts are not
  parameterized, so the target is matched by id alone). Implementation:
  `effective.rs::art_bonus`, `effective_art_score`, `art_bonuses` (serialized to
  the frontend by `arm-app::ruleset_io`). `validation.rs` folds the bonus into the
  Art score map so `ArtMin` is met by the boosted score.

#### Great (Characteristic) — raise the buy cap to +4/+5
> "You may raise any Characteristic that already has a score of at least +3 by
> one point, to no more than +5 … You may take this Virtue twice for the same
> Characteristic, and for more than one Characteristic."

Great Characteristic grants **no free point**: it raises the buy *cap* (+3 → +4
→ +5; line 4105 confirms +3 is the cap "unless you take … Great Characteristic"),
and the score is still bought against the cost table.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:3987-3989` (and
  `:4105`).
- Data: `rules/core/virtues_flaws.json` `virtue.great_characteristic` —
  `characteristic`-domain param; `effects: [{ characteristic_limit, param:
  "characteristic", amount: 1 }]`; `max_per_target: 2`. The base cap (+3) and the
  +5 ceiling are `base_max` / `effective_max` in `rules/core/characteristics.json`.
- Implementation: `effective.rs::characteristic_cap` =
  `min(base_max + Σ positive amounts, effective_max)`;
  `validation.rs::validate_characteristics` flags a bought score above the cap
  (`characteristic_above_cap`); `validate_characteristic_limit_preconditions`
  flags a target base below the base cap (`characteristic_max_base_too_low`) —
  the "≥ +3" precondition is parameter-relative, derived from `base_max` by the
  amount's sign, so it lives on the effect, not in the static `Prereq`.

#### Poor (Characteristic) — lower the buy floor to −4/−5
> "lower one which is already −3 or lower by one point … You may take this Flaw
> twice for a single Characteristic, lowering it to −5, and multiple times for
> different Characteristics."

The exact sign-mirror of Great: a `flaw`, `amount: -1`, lowering the buy *floor*
(−3 → −4 → −5) without granting/removing points beyond the score's own cost.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:6598-6600`.
- Data: `rules/core/virtues_flaws.json` `flaw.poor_characteristic` —
  `characteristic`-domain param; `effects: [{ characteristic_limit, param:
  "characteristic", amount: -1 }]`; `max_per_target: 2`. The base floor (−3) and
  the −5 floor are `base_min` / `effective_min` in `characteristics.json`.
- Implementation: `effective.rs::characteristic_floor` =
  `max(base_min + Σ negative amounts, effective_min)`;
  `validate_characteristics` flags a bought score below the floor
  (`characteristic_below_floor`); `validate_characteristic_limit_preconditions`
  flags a target base above the base floor (`characteristic_min_base_too_high`).

#### Selection multiplicity — `max_per_target`
The per-`(item, params)` selection cap. `validate_duplicate_selections`
(`validation.rs`) errors `duplicate_selection` when a target's count exceeds the
item's `max_per_target` (default 1; Great Characteristic 2). This generalizes the
former hardcoded "at most once" rule and enforces both "Puissant once per
Ability" (`:4816`) and "Great twice per Characteristic" (`:3989`). Effect
integrity (`ruleset.rs::validate_effect_refs`) rejects at load any effect whose
`param` is undeclared or whose domain mismatches the effect kind.

#### Deferred — milestone assignments
The plan was reordered so all input for all character types lands in M4 (direct
entry) before the guided wizard in M5. Accordingly:

- **M4/4a (Arts):** the Art registry, `House`/`ArtMin` evaluation, and registry-
  backed `ParameterDomain::Art` resolution (replacing the `true` stub). Puissant
  Art (+3) lands here too (M4/4f), now that the Art registry exists — it was only
  ever blocked on the registry, not on the wizard.
- **M4/4b (Houses):** the *category-restricted* magus cap "≤1 Major Hermetic
  Virtue" (`:2857`) via a new data-driven `virtue_category_caps` (see the cap
  note above).
- **M4/4f (V/F effect families):** *Improved Characteristics* (a +3 point-buy
  pool, not an effective-score bonus), Affinity with Ability/Art (XP-cost
  modifier `:3372-3378`), restricted XP-grant pools (Educated/Warrior/Privileged
  Upbringing), and `ability_score_grant` starting-score effects.
- **M5 (guided wizard):** the life-stage XP acquisition (early childhood 75+45 xp
  `:2378`; later life 15/20/10 xp/yr `:2390-2394`; age→max-score cap
  `:2368-2374`), the Sample Childhood packages (`:2380-2388`), the magus
  apprenticeship/post-apprenticeship Art-XP flow (`:2433-2471`), and the aging
  engine for characters over 35 (`:16563-16640`). The age→max-score *cap* itself
  is enforced as direct-entry validation in M4/4e; only the XP *acquisition* and
  aging *rolls* are M5.

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
