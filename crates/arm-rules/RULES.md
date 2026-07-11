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
- The magus rule "You may not have more than one Major Hermetic Virtue" (`:2857`)
  is a *category-restricted* cap (Hermetic Major Virtues only), not a plain
  Major-count cap. Implemented in M4/4b via the data-driven
  `virtue_category_caps` (mirroring `flaw_category_caps`) — see the **Houses**
  section below and the "Resolved: companion `max_major_virtues`" note.

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

#### Tainted Virtues/Flaws — the "Type" tag + half-of-taken cap
> "Tainted Virtues and Flaws are associated with the Infernal realm ... no more
> than half a character's Virtues should be tainted, and similarly for Flaws. ...
> Supernatural abilities granted by Tainted Virtues or Flaws are always Infernal
> powers." (`:3000`)

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2998-3002`.
- Data: the descriptor's optional "Type" token maps to `PointItem.tainted`
  (`bool`, default false) in `rules/core/virtues_flaws.json`.
- Implementation: `crates/arm-rules/src/validation.rs` — `validate_tainted_cap`.
  The book frames the limit as a "should", so it is a **non-blocking warning**,
  measured against the points **actually taken** (not the type budget): a side
  warns when `2·tainted_points > total_points` for that side (Virtue / Flaw).
  Free items contribute 0 points and never affect the ratio. Codes
  `too_many_tainted_virtues` / `too_many_tainted_flaws` (Fluent
  `issue-too_many_tainted_*`, args `$tainted`/`$total`). The tag itself renders
  via Fluent `vf-tag-tainted` (EN "Tainted" / DE "Befleckt", per the glossary's
  type-label rule). The parenthetical "(five points ... ten for a Mythic
  Companion ...)" is illustrative of a maxed build, not a separate flat cap.

#### Full core Virtue/Flaw catalogue — `rules/core/virtues_flaws.json`
> Virtues: `## Virtues` detailed entries `:3360-5282`; Flaws: `## Flaws`
> `:5639-7119`. Each entry is `#### Name` + an italic `*Magnitude, Category[,
> Type]*` descriptor + prose.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:3360-5282` (Virtues),
  `:5639-7119` (Flaws); each item's own `source` field carries its line range.
- Data: the full core catalogue (653 items) lives in
  `rules/core/virtues_flaws.json`, with EN/DE display text in
  `rules/i18n/{en,de}/virtues_flaws.json`. This widens the data only — the engine,
  i18n schema, and UI already support any catalogue size (catalogue size is data).
- Extraction conventions (structural fields; effects/prereqs authored per mechanic
  elsewhere): magnitude/category parsed from the descriptor; category slugged
  (`Social Status`→`social_status`, source typo `Subernatural`→`supernatural`,
  compound labels like `General and Hermetic` take the **earliest-listed**
  category). The optional `Type` token sets `tainted` (see the Tainted note above).
- **Dual-magnitude split.** A `*Major or Minor*` item becomes two entries,
  `<id>_minor` and `<id>_major`, marked mutually `incompatible_with` (so exactly
  one magnitude is chosen), disambiguated in i18n as "Name (Minor/Major)" /
  "Name (Klein/Groß)". 32 core items split this way.
- **German provenance.** DE names/summaries come from the line-mirrored German
  source (`Ars Magica Definitive Edition Basisregeln.md`, same line positions),
  cross-checked against `rules/source/de/translation-tables/tugenden-fehler.md`
  (glossary wins on any term mismatch). The DE descriptor uses `Kostenlos` as well
  as `Frei` for Free.
- Referential integrity + EN/DE i18n coverage are asserted structurally by
  `tests/data_integrity.rs` (`shipped_data_passes_integrity_check`,
  `english/german_i18n_covers_all_items`) — never an exact catalogue total.

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
| magus | `virtue_category_caps`: hermetic major_only/hard `max: 1` | `:2857` ("may not have more than one Major Hermetic Virtue") — see the Houses section |
| mythic_companion | `virtue_points: 20`, `flaw_points: 10`, `virtue_points_per_flaw_point: 2` | `:2638` ("up to ten points of Flaws, and each point of Flaws is worth two points of Virtues. This produces a maximum of 21 points of Virtues and 10 points of Flaws") |
| mythic_companion | `forbidden_categories: [hermetic]`, `gift_policy: forbidden` | `:2637` (Mythic Companion status Virtues "are incompatible … with The Gift"); generated as Companions `:2635` |

Landed for `magus` in M4/4b (Houses; see the Houses section): the `≤1 Major
Hermetic Virtue` cap (`:2857`) via `virtue_category_caps`, the free Minor House
Virtue (`:2859`) via the derived-grant model, and the "≥1 Hermetic Flaw"
guideline (`:2860`) via the `missing_hermetic_flaw` warning. The Mythic Companion's free Minor status
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

#### Affinity with (Ability) — creation XP counts for half again
> "All Advancement Totals for one Ability are increased by half, rounded up, as
> are any experience points you put in that Ability at character creation. … If
> you take this Virtue for an Ability, you may exceed the normal age-based cap
> during character generation … by two points for that Ability."

The "counts as 1½×" rule is modelled as a cost reduction: to reach a score whose
table cost is `T`, the XP *charged* is the smallest `c` with `ceil(c·3/2) ≥ T`,
i.e. `charged = ceil(T·2/3)`. The cap exemption is read off the effect's presence
(age cap itself is M4/4e).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:3372-3374`.
- Data: `rules/core/virtues_flaws.json` `virtue.affinity_ability` —
  `category: general`, `ability`-domain param, `effects: [{ affinity_ability_cost,
  param: "ability", counts_as_num: 3, counts_as_den: 2 }]`.
- Implementation: `effective.rs::charged_cost` (the `ceil(T·den/num)` arithmetic,
  verified against the worked example below) + `ability_affinity`, folded into
  `effective.rs::xp_allocation` and so into `validation.rs::validate_xp_pool`.

#### Affinity with (Art) — creation XP counts for half again
> "Your Advancement Totals for one Hermetic Art are increased by one half, rounded
> up. At character creation, any experience points you put into that Art are also
> increased by one half (rounded up) … You may take this Virtue twice, for two
> different Arts."

Worked example (`:2443`): "He spends 37 points on Perdo, which his affinity turns
into 56 points, so that he has Perdo 10 (1)" — Perdo 10 needs 55 on the Art table,
and `charged_cost(55, 3/2) = ceil(55·2/3) = 37`. (Perdo is a Technique/Art; the
identical rule applies to Abilities but against the 5×-larger Ability table.)

- Source: `Ars Magica - Definitive Edition (Core Rules).md:3376-3378`, example
  `:2443`.
- Data: `rules/core/virtues_flaws.json` `virtue.affinity_art` — `category:
  hermetic`, `art`-domain param, `effects: [{ affinity_art_cost, param: "art",
  counts_as_num: 3, counts_as_den: 2 }]`.
- Implementation: `effective.rs::charged_cost` + `art_affinity`, via
  `xp_allocation`.

#### Educated / Warrior / Privileged Upbringing — restricted XP pools
> Educated: "you get an additional 50 experience points, which must be spent on
> Latin and Artes Liberales." Warrior: "gain an additional 50 experience points
> which must be spent on Martial Abilities." Privileged Upbringing: "an additional
> 50 experience points, which may be spent on General, Academic, or Martial
> Abilities."

Each grants a pool spendable only on its eligible Abilities (never Arts); the
general `xp_pool` covers anything; unused restricted XP is wasted (a non-blocking
warning). Eligibility is by ability id **or** category. Latin is modelled as the
parameterized `ability.dead_language`, so Educated lists `ability.dead_language` +
`ability.artes_liberales` (any Dead Language qualifies — a deliberate seed
approximation of "Latin").

- Source: `:3711-3713` (Educated), `:5227-5229` (Warrior), `:4806-4808`
  (Privileged Upbringing).
- Data: `rules/core/virtues_flaws.json` `virtue.educated` /`virtue.warrior` /
  `virtue.privileged_upbringing` — `effects: [{ restricted_ability_xp, amount: 50,
  abilities | categories }]`. The `50` lives here.
- Implementation: `effective.rs::xp_allocation` builds a bipartite **max-flow**
  feasibility graph (general pool + one node per restricted pool → eligible spends
  → sink). A greedy assignment is incorrect under overlapping eligibility
  (Educated's academic ids overlap Privileged's `academic` category), so flow is
  used. `validation.rs::validate_xp_pool` reports `not_enough_xp` (with
  `shortfall`) and `restricted_xp_unspent` (warning).
- Permission unlock (Academic/Martial purchasable only with such a Virtue) is
  **deferred** — not enforced in this phase.

#### Improved Characteristics — +3 Characteristic-buy points
> "You have an additional three points to spend on buying Characteristics … You
> may take this Virtue multiple times."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4103-4105`.
- Data: `rules/core/virtues_flaws.json` `virtue.improved_characteristics` —
  `effects: [{ characteristic_points, amount: 3 }]`. The `3` lives here.
- Implementation: `effective.rs::characteristic_points_granted` sums the grants;
  `validation.rs::validate_characteristics` budget = `start_points + granted`. The
  per-characteristic +3 *cap* is unchanged (only Great Characteristic widens it).

#### Weak Characteristics — −3 Characteristic-buy points (`characteristic_points`, signed)
> "You have three fewer points to spend buying Characteristics … You may take
> this Flaw twice, leaving you with only one point to spend."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:7056-7058`.
- Data: `flaw.weak_characteristics` — `effects: [{ characteristic_points, amount:
  -3 }]`, `max_per_target: 2`. `Effect::CharacteristicPoints.amount` is `i8`
  (signed) and `characteristic_points_granted` returns `i32`, so Weak nets against
  Improved (+3 and −3 cancel). The budget clamps naturally via the cost check.

#### Giant Blood / Large / Dwarf / Small Frame — Size + free Characteristic bonus
> Giant Blood: "Your Size is +2 … gain +1 to both Strength and Stamina. This
> bonus may raise your scores … as high as +6." Dwarf: "Your Size is −2 … −1 to
> each of Strength and Stamina … as low as −6." Large: Size +1. Small Frame: Size −1.
> Each lists the other three as mutually exclusive.

- Source: `:3975-3978` (Giant Blood), `:4229-4231` (Large), `:5996-5998` (Dwarf),
  `:6767-6769` (Small Frame).
- Data: `virtue.giant_blood` / `virtue.large` / `flaw.dwarf` / `flaw.small_frame`,
  each with a `size_delta` effect (and Giant Blood/Dwarf a `characteristic_score_delta`
  for Str and Sta), plus a symmetric `incompatible_with` clique across all four.
- Implementation: two new effects. `Effect::SizeDelta { amount }` →
  `effective.rs::size` (base 0 + Σ, no cost/cap; surfaced as `EffectiveScores.size`).
  `Effect::CharacteristicScoreDelta { characteristic, amount }` →
  `characteristic_score_bonus` / `effective_characteristic_score` — a **free**
  effective-score bonus (separate from the bought score, no buy-budget cost) that
  may push the effective score past ±5 to ±6. Surfaced as
  `EffectiveScores.characteristic_bonuses` and shown in the sheet next to the bought
  score; Size shows via Fluent `characteristic-size`. The stored `characteristic`
  id is validated to resolve in `ruleset.rs::validate_effect_refs`.

#### Warped by Magic — Warping Score + Points (`warping_grant`)
> "He has five Warping Points and a Warping Score of 1, including a Minor Flaw
> (which is not balanced by a Virtue) … His encounters allow you to spend
> experience points on Magic Lore during character creation."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:7019-7021`.
- Data: `flaw.warped_by_magic` — `effects: [{ warping_grant, score: 1, points: 5 }]`.
- Implementation: `Effect::WarpingGrant { score, points }` → `effective.rs::warping`
  (derived `(score, points)`, base 0 each, summed across grants — never stored, like
  Confidence). Surfaced as `EffectiveScores.warping_{score,points}` and shown on the
  sheet via Fluent `warping-label`/`warping-readout` (DE "Verzerrung", per the
  glossary). **Deferred (separate machinery):** the free unbalanced Minor Flaw
  (nested-grant, B6) and the "spend XP on Magic Lore" purchase permission are not
  part of this numeric effect.

#### True Faith — special derived score (`true_faith_grant`)
> "You have a True Faith score of 1 and can gain more."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:5169-5171`.
- Data: `virtue.true_faith` — `effects: [{ true_faith_grant, score: 1 }]`.
- Implementation: `Effect::TrueFaithGrant { score }` → `effective.rs::true_faith`
  (derived, base 0 + Σ, like Warping). True Faith is a special score with its own
  rules (Core p.419), **not** a Supernatural Ability, so it is not modelled via
  `ability_score_grant`. Surfaced as `EffectiveScores.true_faith_score` and shown on
  the sheet (Fluent `true-faith-label/readout`, DE "Wahrer Glaube" per glossary).
  `flaw`/`virtue.relic` grants a *possessed* holy item with True Faith 1 — the
  character's own score stays 0 — so Relic is item-possession, left structural.

#### Second Sight / Premonitions — free starting Ability score (`ability_score_grant`)
> Second Sight: "Choosing this Virtue confers the Ability Second Sight 1."
> Premonitions: "Choosing this Virtue confers the Ability Premonitions 1."

A free bought-score *floor*, costing no XP: effective score = `max(bought, grant)
+ bonuses`. The target Ability is fixed by the Virtue (not player-chosen), so the
effect stores the ability id directly (`ability`), not a selection parameter.

- Source: `:4888-4890` (Second Sight), `:4788-4790` (Premonitions).
- Data: `rules/core/virtues_flaws.json` `virtue.second_sight` /
  `virtue.premonitions` — `category: supernatural`, `effects: [{ ability_score_grant,
  ability: "ability.second_sight" | "ability.premonitions", amount: 1 }]`.
- Implementation: `effective.rs::granted_ability_floor`, folded into
  `effective_ability_score`; `ruleset.rs::validate_effect_refs` checks the
  ability id resolves against the catalogue.

#### Deferred — milestone assignments
The plan was reordered so all input for all character types lands in M4 (direct
entry) before the guided wizard in M5. Accordingly:

- **M4/4a (Arts):** the Art registry, `ArtMin` evaluation, and registry-
  backed `ParameterDomain::Art` resolution (replacing the `true` stub).
  (`Prereq::House` evaluation lands in M4/4b — see below.) Puissant
  Art (+3) lands here too (M4/4f), now that the Art registry exists — it was only
  ever blocked on the registry, not on the wizard.
- **M4/4b (Houses):** *done* — the twelve Hermetic Houses, the derived
  free-House-Virtue grant model, `Prereq::House` evaluation, `validate_house`,
  and the *category-restricted* magus cap "≤1 Major Hermetic Virtue" (`:2857`)
  via the data-driven `virtue_category_caps`. See the **Houses** section above.
- **M4/4f (V/F effect families):** *done* — Affinity with Ability/Art (XP-cost
  modifier), restricted XP-grant pools (Educated/Warrior/Privileged Upbringing),
  Improved Characteristics (+3 point-buy pool), and `ability_score_grant`
  starting-score effects. See the effect-layer subsections above.
- **M5 (guided wizard):** the life-stage XP acquisition (early childhood 75+45 xp
  `:2378`; later life 15/20/10 xp/yr `:2390-2394`; age→max-score cap
  `:2368-2374`), the Sample Childhood packages (`:2380-2388`), the magus
  apprenticeship/post-apprenticeship Art-XP flow (`:2433-2471`), and the aging
  engine for characters over 35 (`:16563-16640`). The age→max-score *cap* itself
  is enforced as direct-entry validation in M4/4e; only the XP *acquisition* and
  aging *rolls* are M5.

### Houses (magus-only)

Every magus belongs to one of the twelve Hermetic Houses. A House confers a free
benefit — a Virtue that "you need not balance with a Flaw" (`:2859`). The save
stores only `Entity::house` + the specialisation `house_choices` (choices, not
resolved values); the free Virtue is **derived** at eval by
`house.rs::granted_selections`, never persisted.

#### House benefit table
> The twelve Houses and their benefits — Bjornaer → Heartbeast (score 1);
> Bonisagus → Puissant Magic Theory *or* Intrigue; Criamon → The Enigma
> (Enigmatic Wisdom 1); Ex Miscellanea → a free Minor Hermetic Virtue, a free
> Major non-Hermetic Virtue, and a compulsory Major Hermetic Flaw "in addition
> to the normal allowance"; Flambeau → Puissant Perdo *or* Ignem; Guernicus →
> Hermetic Prestige; Jerbiton → a Minor Virtue; Mercere → Puissant Creo *or*
> Muto; Merinita → Faerie Magic (score 1); Tremere → Minor Magical Focus
> (certamen); Tytalus → Self-Confident; Verditius → Verditius Magic.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2270-2283` (the House
  table). Every house in `rules/core/houses.json` cites this range.
- Data: `rules/core/houses.json` — each `House { id, lineage_type, grants }`.
  A grant is `fixed` (a named Virtue), `choice` (pick one of listed options), or
  `open` (pick any item satisfying a declarative `GrantConstraint`). Ex Miscellanea
  is three `open` grants (Minor Hermetic Virtue / Major non-Hermetic Virtue /
  Major Hermetic Flaw); Jerbiton is one broad `open` (Minor Virtue, troupe-judged
  per the rule's own wording). `lineage_type` (True Lineage / Mystery Cult /
  Societas) is the book's grouping, flavor only — no mechanics hang off it.
- Implementation: `crates/arm-rules/src/house.rs` — `House`, `LineageType`,
  `HouseGrant`, `GrantConstraint`, `HousesFile`, and `granted_selections`.
  Registry `Ruleset::houses` + integrity `validate_house_refs` (every `Fixed.item`
  / `Choice.options[].ref` resolves against `point_items`; `House.source` range is
  valid) in `ruleset.rs`; `Prereq::House` evaluation and the `validate_house` pass
  in `validation.rs`. `Bonisagus`/`Mercere`/`Flambeau` reuse the generic
  `virtue.puissant_ability` / `virtue.puissant_art` (target via param) — no new
  Puissant items.

#### The free House Virtue is budget-free and uncapped
> "You receive one free Minor Virtue from your choice of House, which you need
> not balance with a Flaw."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2859`.
- Implementation: the derived grant model. `effective.rs::selections_for_effects`
  returns `entity.selections ++ granted_selections` so a granted Virtue's effects
  (e.g. a Mystery Ability floor) participate, but `compute_balance` and
  `validate_caps` stay on `entity.selections` (bought only) — commented as the
  line that makes grants free and uncapped. So a granted Virtue never costs points
  and never trips a virtue-count cap, and Ex Miscellanea's three grants are "in
  addition to the normal allowance" (`:2273`) purely by construction.

#### ≤1 Major Hermetic Virtue (magus cap) — `virtue_category_caps`
> "You may not have more than one Major Hermetic Virtue."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2857`.
- This is a *category-restricted* virtue-count cap (Hermetic Major Virtues only),
  with no prior analogue — the flaw side had `flaw_category_caps`, virtues had
  none. Modelled by extracting the shared `CategoryCap { category, max, major_only,
  hard }` (byte-identical to the former `FlawCategoryCap`) and adding
  `PointBudget.virtue_category_caps`.
- Data: magus profile in `rules/core/character_types.json` —
  `virtue_category_caps: [{ "category": "hermetic", "max": 1, "major_only": true,
  "hard": true }]`. The cap *value* lives here.
- Implementation: `validation.rs::validate_caps` gains a virtue loop mirroring the
  flaw loop, counting `entity.selections` only (granted Hermetic Virtues exempt),
  emitting `too_many_major_hermetic_virtues` (code derived as
  `too_many_[major_]<category>_virtues`). Fluent key `issue-too_many_major_hermetic_virtues`
  in both locales; `dynamic_virtue_cap_codes()` extends the Fluent-coverage test.

#### ≥1 Hermetic Flaw (magus guideline)
> "You should take at least one Hermetic Flaw."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2860`.
- A "should", so a **soft warning** — `validation.rs::validate_house` emits
  `missing_hermetic_flaw` when a magus has no selected Flaw whose category is in
  the profile's Gift categories (data-driven, not a hardcoded `"hermetic"`).

#### `validate_house` — specialisation resolution
- `house_choice_unresolved` (error): a `Choice` pick missing or not among its
  `options`, or an `Open` pick missing.
- `house_grant_constraint` (error): an `Open` pick violating its `GrantConstraint`
  (kind / magnitude / require- / forbid-categories, all read from data).
- `house_unset` (warning): a magus with no House chosen.

#### House-granted Virtue/Ability definitions
The named Virtues a House grants, and the Mystery Abilities their
`ability_score_grant` seeds at score 1, are all defined in the Core Rules
(this edition folds the House virtues into the core V/F list). Each new
`rules/core/virtues_flaws.json` item and `rules/core/abilities.json` entry cites:

| Item | Kind | Source (Core Rules.md) |
|------|------|------------------------|
| `virtue.the_enigma` (grants `ability.enigmatic_wisdom` 1) | Minor, Hermetic | `:3759-3761` |
| `virtue.faerie_magic` (grants `ability.faerie_magic` 1) | Minor, Hermetic | `:3825-3827` |
| `virtue.heartbeast` (grants `ability.heartbeast` 1) | Minor, Hermetic | `:4059-4061` |
| `virtue.verditius_magic` (no creation-number effect) | Minor, Hermetic | `:5215-5217` |
| `virtue.hermetic_prestige` (Reputation → M5, no creation effect) | Minor, Hermetic | `:4071-4073` |
| `virtue.minor_magical_focus` (in-play casting, no creation effect) | Minor, Hermetic | `:4536-4542` |
| `virtue.self_confident` (Confidence → M5, no creation effect) | Minor, General | `:4900-4902` |
| `ability.intrigue` | General | `:7590-7592` |
| `ability.enigmatic_wisdom` (requires_training) | Arcane | `:7454-7455` |
| `ability.faerie_magic` (requires_training) | Arcane | `:7478-7479` |
| `ability.heartbeast` (requires_training) | Arcane | `:7501-7502` |

All four Mystery Virtues (The Enigma, Faerie Magic, Heartbeast, Verditius Magic)
are **Minor** — the free House Virtue is Minor per `:2859`, so none of them can
trip the ≤1-Major-Hermetic-Virtue cap even when granted. Magical Focus's field is
freeform prose (doesn't fit the ref-only param model), so Tremere grants a plain
`fixed virtue.minor_magical_focus` with no param. Ex Miscellanea's Minor Hermetic
Virtue is player-chosen (`open`), not a fixed item, so there is no invented
`virtue.ex_misc_minor_hermetic`.

---

## Mythic Companion types (M4/4d)

A Mythic Companion picks a **type** (Devil Child, Faerie Doctor, Nephilim, Spirit
Votary) that grants a free "status" Virtue **plus** a free Minor Virtue (both
point-free grants, via the shared `grant.rs` model), imposes a required V/F
package that counts against the budget normally, and may raise the budget
ceilings with per-type bonus points. Data: `rules/core/mythic_companion_types.json`
+ `rules/i18n/<lang>/mythic_companion_types.json`; engine `mythic_companion.rs`
(`MythicCompanionType`, `RequiredFlaw`); validation `validate_mythic_type` +
the effective-budget fold in `validate_balance`. The general rules are Core
Rules.md `:2635-2639`; the V/F guidelines `:2842-2851`.

#### General mechanism & per-type budgets
> `:2637` "All Mythic Companions take a Free Virtue which specifies their status.
> These Virtues are incompatible with each other, and with The Gift, and are not
> available to grogs." `:2638` "You gain a free Minor Virtue… you may take up to
> ten points of Flaws, and each point of Flaws is worth two points of Virtues…
> Most Mythic Companion Virtues require you to take some particular Virtues and
> Flaws, these count against your maximum."

The four status Virtues carry a symmetric `incompatible_with` web (each other +
`virtue.the_gift`). The free status Virtue's category is `social_status` (a
"status" Virtue); the free Minor and required Virtues keep their own book
categories. Per-type bonus points fold into the balance ceilings via
`EffectiveBudget` (`validation.rs`): `flaw_ceiling = base + bonus_flaw`,
`virtue_ceiling = base + bonus_flaw·rate + bonus_free_virtue`,
`funded = flaw·rate + bonus_free_virtue` (rate = 2). Zero for a non-mythic type,
so the check reduces exactly to the base budget.

| Type | free status | free Minor | required Virtues (budgeted) | required Flaw (default) | bonus | Source |
|------|-------------|-----------|------------------------------|--------------------------|-------|--------|
| Devil Child | `virtue.devil_child` | Demonic Might **or** Powers | Demonic Blood, Puissant (Guile) | Tragic Life | +7 F, **+3 free V** | Core `:2643-2666` |
| Faerie Doctor | `virtue.faerie_doctor` | Dowsing | Wise One, Curse-Throwing | Faerie Friend, Dutybound | none | Core `:2668-2704` |
| Nephilim | `virtue.nephilim` | Strong Angelic Heritage | Blood of the Nephilim, Greater Immunity, Great Sta, Great Str, Improved Characteristics, Sense Holiness | — (5 F fund the +10 V) | none | Core `:2714-2739` |
| Spirit Votary | `virtue.spirit_votary` | Second Sight | Spiritual Pact (+ 1 Major/3 Minor Supernatural, **advisory**) | Pagan | +7 F | Core `:2741-2764` |

- Devil Child's **+3 free V / +7 F**: Core `:2664` ("three more points of Virtues
  at no cost… and an additional seven points of Flaws"). Verified maxed budget:
  flaw 17, virtue `20 + 14 + 3 = 37`.
- Spirit Votary's **+7 F**: *Realms of Power — Magic.md*`:5486` ("may take up to 7
  more points of Flaws, each point granting 2 points"). The Core mythic section
  is silent on this bonus — RoP Magic is the authority; the discrepancy is noted
  here.
- **Nephilim** has no bonus: its "5 points of Flaws to pay for these virtues"
  (`:2731`) is a consequence of funding 10 pts of required Virtues at 2:1 within
  the base 10 F / 20 V, not a budget change.
- The `20`-vs-`21` inconsistency (`:2638` "21" vs `:2844` "20"): the ceiling is
  `20`; the free Minor status Virtue is a point-free grant that never consumes
  budget (same treatment as the free House Virtue).
- **Required-package enforcement is advisory** (`mythic_required_trait_missing`
  warning): the rules permit "a suitable substitute agreed with the troupe" for
  the required Flaws (`:2660`, `:2689`, `:2754`), so a missing/substituted slot
  never hard-blocks. Required Virtues are matched by full `Selection` (ref +
  params), so parameterized/duplicated requirements are exact — Nephilim's two
  distinct Great Characteristics (`great_characteristic` with
  `characteristic.sta` / `.str`) and Puissant Guile (`puissant_ability` +
  `ability.guile`) — an approximation matching only the seeded default target.
- Spirit Votary's open "1 Major OR 3 Minor Supernatural" requirement is left
  **advisory** (unenforced) for M4 — a candidate for a future "minimum category
  points" rule (M8).

#### New V/F & Ability definitions (structural; full in-play effects M8)
Per CLAUDE.md each is sourced from the authoritative Markdown by book. Only the
*structural* item (kind/magnitude/category) is seeded now; the full supernatural
effects (Infernal Might, Divine Might, immunity target, etc.) are M8.

| Item | Kind | Source |
|------|------|--------|
| `virtue.devil_child` | Special/Free, Social Status | *Infernal*`:4144-4149` |
| `virtue.demonic_blood` | Major, Supernatural (Tainted) | *Infernal*`:4116-4131` |
| `virtue.demonic_might` (req. Demonic Blood) | Minor, Supernatural | *Infernal*`:4132-4137` |
| `virtue.demonic_powers` (req. Demonic Blood) | Minor, Supernatural | *Infernal*`:4138-4143` |
| `flaw.tragic_life` | **Major, Story** (Tainted) | Core `:6855-6870` |
| `virtue.faerie_doctor` | Special/Free, Social Status | *Faerie*`:6394-6399` |
| `virtue.dowsing` (grants `ability.dowsing` 1) | Minor, Supernatural | Core `:3703-3706` |
| `virtue.curse_throwing` (grants `ability.curse_throwing` 1) | Major, Supernatural | *Faerie*`:6342-6347` |
| `virtue.wise_one` | Minor, Social Status | Core `:5257-5260` |
| `flaw.faerie_friend` | Minor, Story | Core `:6052-6055` |
| `flaw.dutybound` | Minor, Personality | Core `:5992-5995` |
| `virtue.nephilim` | Free, Social Status | *Divine*`:3485-3513` |
| `virtue.blood_of_the_nephilim` | Major, Supernatural | *Divine*`:1941-1954` |
| `virtue.strong_angelic_heritage` (req. Blood of the Nephilim) | Minor, Supernatural | *Divine*`:1969-1980` |
| `virtue.greater_immunity` (plain; "Disease" is an in-play target, no param) | Major, Supernatural | Core `:4009-4016` |
| `virtue.sense_holiness_and_unholiness` (grants `ability.sense_holiness_and_unholiness` 1) | Minor, Supernatural | Core `:4926-4929` |
| `virtue.spirit_votary` | Free, Supernatural (→ Social Status) | *Magic*`:5480-5486` |
| `virtue.spiritual_pact` | Major, Supernatural | *Magic*`:5488-5502` |
| `flaw.pagan` (Major or Minor; seeded Major) | Major, Personality | Core `:6570-6573` |
| `ability.curse_throwing` (requires_training) | Supernatural | Core `:7396-7430` |
| `ability.dowsing` (requires_training) | Supernatural | Core `:7439-7442` |
| `ability.sense_holiness_and_unholiness` (requires_training) | Supernatural | Core `:7720-7723` |

Reused existing items: `virtue.great_characteristic` (Great Sta/Str),
`virtue.improved_characteristics`, `virtue.second_sight`,
`virtue.puissant_ability` (+ `ability.guile`). **Greater Immunity** is seeded
without a parameter: the `ParamType` model is ref-domain-only and there is no
"hazard" registry, so the specific "Disease" target is an in-play/sheet detail
(M8), and the Nephilim requirement matches it by ref alone.

---

## Spells (magus-only, M4/4c)

A starting magus knows a list of spells, each drawn from the catalogue in
`rules/core/spells.json` (`spell.rs`: `Spell { technique, form, level,
requisites }`, with `level: None` marking a **General** spell learned at a
per-character level). The chosen spells live on `Entity::spells`
(`SpellSelection { spell, level }`); `validate_spells` (`validation.rs`) enforces
two sourced constraints, both magus-only (gated on the profile `is_magus`).

**Spell-levels budget — 120 at creation.**

> `:2215-2216` "**Hermetic Magi Only: Apprenticeship.** … Take 120 levels of
> spells, of no higher level than Technique + Form + Intelligence + Magic Theory
> +3."
> `:2435` "The fifteen years of apprenticeship give the character 240 experience
> points, and 120 levels of spells."

Encoded as `EntityTypeProfile.spell_levels: 120` on the magus profile
(`rules/core/character_types.json`; JSON has no comments, so the value's
provenance lives here). The sum of the chosen spells' levels must not exceed the
effective budget → `over_spell_levels`. Value lives in data; the engine never
hardcodes 120.

**Per-spell cap — Technique + Form + Intelligence + Magic Theory + 3.**

> `:2465` "The highest level spell you can learn is equal to Technique + Form +
> Intelligence + Magic Theory +3 … If the spell has requisites … they apply to
> this total as well."

`spell_level_cap` (`validation.rs`) computes it from the effective Art scores,
the Intelligence characteristic, and effective Magic Theory → a spell above it
emits `spell_level_exceeds_cap`. **Approximation:** requisite-Art reduction is a
lab-total nuance out of M4 scope — requisites are stored on the spell for display
but not folded into the cap.

**General spells.**

> `:12349-12353` "Some spells are General spells (abbreviated to Gen), which
> means that they may be learned at any level … different levels of a General
> level spell are still different spells."

A General spell's catalogue `level` is `None`; the learned level is the
per-character `SpellSelection.level`. Identity (and the dedup key) is
`(spell, level)`; an unresolved General spell (no chosen level) warns
(`spell_level_unresolved`) and is excluded from the budget sum.

**Budget-modifier Virtues/Flaws (Skilled/Weak Parens).** Two Hermetic V/F modify
the apprenticeship grant, each via *two* `Effect`s
(`spell_levels` ± and `general_xp` ±, both signed, summed and clamped at 0):

| Item | magnitude/category | effects | source |
|------|--------------------|---------|--------|
| `virtue.skilled_parens` | Minor, Hermetic | `spell_levels +30`, `general_xp +60` | `:4964-4966` |
| `flaw.weak_parens` | Minor, Hermetic | `spell_levels -30`, `general_xp -60` | `:7072-7074` |

> `:4966` (Skilled Parens) "You gain an additional 60 experience points and 30
> spell levels during apprenticeship."
> `:7074` (Weak Parens) "You gain 60 fewer experience points and 30 fewer spell
> levels from apprenticeship, for a total of 180 experience points and 90 levels
> of spells."

`Effect::SpellLevels` folds into the spell budget (`effective::spell_levels_budget`);
`Effect::GeneralXp` folds into the general apprenticeship pool
(`effective::xp_allocation`'s `general_pool`). Both are ref-free effects
(`validate_effect_refs`). Seeded spells (14, spread across Creo/Rego × several
Forms, incl. one requisite spell and two General spells) each cite their Core
Rules line range in `spells.json`; the full catalogue stays M8. German spell
names follow `rules/source/de/translation-tables/zauber-nach-form.md`; the two
Parens follow `tugenden-fehler.md` (Skilled → *Erfahrener Parens*; Weak →
*Schwacher Parens*, the Latin *Parens* kept per the Latin-term convention).

---

## Per-character fields (M4/4d-rest, 4e)

The final M4 phase adds age, Confidence, Personality Traits, Reputations, and the
Gift/Supernatural gate — the remaining Core character-generation surfaces.

**Age → max Ability score.** `effective::age_max_ability_score` encodes the table
(a fixed taxonomy, like `Magnitude::points`; surfaced via
`EffectiveScores.age_ability_cap` so the UI never re-hardcodes it):

> `:2366-2376` "Your character's age determines the maximum score … | under 30 |
> 5 | | 30-35 | 6 | | 36-40 | 7 | | 41-45 | 8 | | 46+ | 9 |"

`validate_abilities` flags a bought Ability above this cap (`ability_above_age_cap`).
An Ability carrying an Affinity may exceed it **by +2** — not without limit:

> `:3374` (Affinity with (Ability)) "you may exceed the normal age-based cap
> during character generation … by two points for that Ability."

Virtues that *raise* the cap generally are deferred (none seeded); only the
Affinity +2 is modelled.

**The Gift → one free Supernatural Ability.** A Supernatural Ability normally
requires a granting Virtue (an `ability_score_grant` effect seeds it — Second
Sight, etc.). The Gift lets a Gifted **non-magus** take one such Ability with no
Virtue; a **magus** gets none (his free supernatural ability is Hermetic magic):

> `:2874` "Characters who have The Gift may start play with a single Supernatural
> Ability, without having to take any other Virtue … The ability to cast Hermetic
> magic is the single supernatural ability possessed by Hermetic magi in virtue
> of The Gift".

`effective::supernatural_free_slots` = `(has_the_gift && !is_magus ? 1 : 0, used)`;
`validate_supernatural_abilities` errors on uncovered Supernatural abilities beyond
the free allowance (`supernatural_ability_requires_virtue`). **Companion
`gift_policy` is `allowed`** (was `forbidden`) so a Gifted companion is legal
(`:2872` "companions should only have The Gift if they are intended to become
magi, or … other magical traditions"); grog/mythic stay `forbidden`.
**Approximation:** "covered" = "has an `ability_score_grant` floor", a proxy for
"has a granting Virtue" — exact for the current seed; `ability.animal_ken` has no
granting Virtue, so it is takeable only via the free slot.

**Confidence** (derived, never stored: type-profile default + `ConfidenceBonus`
effects, via `effective::confidence`):

> `:2521` "Companions and Magi start with a Confidence Score of 1 and 3 Confidence
> Points … Grogs do not have Confidence Points."

`character_types.json` sets `confidence_score:1`/`confidence_points:3` on
companion/magus/mythic; grog omits (0/0). **Self-Confident** (`:4900-4902`, Minor
General) → `confidence_bonus {score:1, points:2}` (raising the default to 2/5).

**Personality Traits** (`Entity.personality_traits`, `validate_personality_traits`):

> `:2500-2503` "attach a value between -3 and +3 … a Major Personality Flaw
> should have a Personality Trait of +6 or -6."

Each trait `|value| ≤ 3`; up to one trait per selected Major Personality Flaw
(`category == personality`, `magnitude == major`) may reach ±6; `|value| > 6`
never. The grog-Loyal / warrior-Brave "should" is guided (M5), not enforced here.

**Reputations** (`Entity.reputations`, `ReputationType` = Local / Ecclesiastical /
Hermetic, `:1091-1101`):

> `:2514` "Characters only start with a Reputation if they choose a Virtue or Flaw
> that grants one, but all characters can develop them in play."

`Effect::GrantsReputation {kind, score}` authorizes one starting Reputation;
`validate_reputations` errors on any reputation beyond the grants of its kind
(`reputation_not_granted`). Seeded granters: **Infamous** (`:6310-6312`, Minor
General, `grants_reputation {local, 4}`) and **Black Sheep** (`:5703-5705`, Major
Story, `{local, 2}`). Only Local granters exist in Core; Hermetic/Ecclesiastical
granters (e.g. Hermetic Prestige, not in the Core source) are M8.

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
