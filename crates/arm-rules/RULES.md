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

#### Parameterized Virtues/Flaws — `{param}` slots
The picker binds a value for items whose name carries a `{param}` placeholder
(resolved by `displayName`), via a `parameters` entry (`{ key, type: "ref",
domain }`). `ParameterPicker.svelte` renders a **dropdown** for registry-resolved
domains and a **free-text input** for `domain: "text"` (its `{:else}` branch).

- **Catalogue-ref domains** (`ability`, `art`, `characteristic`, `item`): the value
  must resolve; e.g. the `(Ability)` items (`flaw.careless_with_ability`, …) and the
  `(Form)` items (`(Form)`→`art`: `flaw.form_monstrosity`,
  `virtue.imbued_with_the_spirit_of_form`, …). Puissant/Affinity/Great/Poor already
  used this. (`art` for a `(Form)` is a Form-only subset of Arts; the dropdown offers
  all 15 Arts — the one remaining imprecision.)
- **Free-text domain** (`ParameterDomain::Text`, serde `"text"`): the parenthetical
  is a free choice with no registry — `(Realm)`, `(Land)`, `(Subject)`, `(Sin)`,
  `(Beings)`, `(Terrain)`, `(Commodity)`, `(Faculty)`, `(Role)`, plus the mixed
  `Necessary (Realm) Aura for (Ability)` (a `text` + an `ability`). `validate_parameters`
  accepts any value for a `text` param (no resolution); the UI text input already
  existed. Param hints come from Fluent `param-label-<key>`.
- **Name-qualifiers — not params, stay literal by design:** `(Dove)`, `(the Wolf)`,
  `(Muq-Ta')`, `(Hermetic)`, `(PC)`, and the `(positive)`/`(negative)` Cyclic Magic
  disambiguators.

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
Virtue (`:2638`, raising the balanced max from 20 to 21) is M5 catalogue data;
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
  range is now −5..+5. (The age → max-Ability-score bands live in
  `rules/core/abilities.json`, not here — see "Age → max Ability score" below.)
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
  (an error in Advisory/Enforced; the M6 wizard blocks the spend up front).
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

#### Ability catalogue (full Core set) — `rules/core/abilities.json`
> Ability list grouped by type (General, Academic, Arcane, Martial,
> Supernatural) at `:7177-7268`; alphabetical descriptions at `:7269-7789`.

> **Full-catalogue note.** `main` ships the **complete 78-ability Core Rules
> catalogue** (General 29, Academic 12, Arcane 12, Martial 4, Supernatural 21) —
> with all descriptions, specialties, and `requires_training` flags. (The former
> 23-ability *seed* set and the `full-abilities` staging branch are obsolete: the
> full catalogue landed on `main` before M5 — commit `185d064` — and is a
> data-only widening the engine, i18n schema, and UI already supported. No
> per-catalogue-total is asserted in tests; `tests/data_integrity.rs` checks only
> that known items and each category are present, per the catalogue-size-is-data
> invariant.)

- Source: each ability cites its description line range in `abilities.json`
  (e.g. Awareness `:7325-7328`, Magic Theory `:7646-7649`). The five categories
  are the book's Ability types (`:7177-7268`), taken from each entry's trailing
  `(Type)` label.
- Data: 78 abilities covering all five categories, including the full
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
  abilities. Notably Penetration and most combat/General abilities are *not*
  asterisked. The flag is set from the
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
- `art_bonuses` iterates the **full Art catalogue** (not just bought
  `art_scores`), emitting any nonzero effective-over-bought delta. A Puissant Art
  (or Elemental Magic form boost) applies at 0 bought points, but the UI drops an
  Art's row at score 0, so gating the list on `art_scores` would hide the badge
  until the first point is bought (Issue 13). Iterating the catalogue surfaces the
  bonus at bought-0 and naturally dedupes any duplicate bought rows.

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
- **Restricted pool spent before the general pool (two-phase fill).** The
  restricted XP is free-but-earmarked, so an eligible spend must drain it before
  the general pool: otherwise the general pool is over-consumed and unused
  restricted XP is spuriously wasted (a false `restricted_xp_unspent` warning).
  A single max-flow run would drain whichever pool BFS reaches first (the general
  node), so the solve is **two-phase** on the shared residual matrix: phase 1 runs
  max flow with the source→general edge closed (restricted-only), phase 2 opens
  that edge and continues Edmonds-Karp on the same residuals. The sum is the true
  max flow with restricted usage maximized = minimum general used; feasibility and
  `total_demand` are unchanged.
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

#### Linguist — group Affinity over all Languages (`group_affinity_cost`)
> "All Advancement Totals for any Language are increased by a quarter, rounded up
> … Both Living and Dead languages are augmented with this Virtue."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4315-4317`.
- Data: `virtue.linguist` — `group_affinity_cost: { abilities: [ability.dead_language,
  ability.living_language], counts_as_num: 5, counts_as_den: 4 }`.
- Implementation: `Effect::GroupAffinityCost { abilities, counts_as_num,
  counts_as_den }` — an Affinity auto-applied to a fixed set of ability ids (any
  instance), vs. `AffinityAbilityCost`'s single player-chosen target. Handled in
  `effective.rs::ability_affinity`, so it feeds the XP-cost reduction
  (`charged_cost`) and the +2 age-cap exemption exactly like a normal Affinity. The
  ability ids are validated to resolve in `ruleset.rs::validate_effect_refs`.

#### Elemental Magic — `Effect::ElementalMagic { forms }` (XP-space Art boost, slice 5c)
> "During character creation, assign all your experience points in Arts. Then
> assign half the experience points assigned to each of the elemental Forms
> [Aquam, Auram, Ignem, Terram] to each of the other elemental Forms." Worked
> example: 21 XP in Ignem → **11** bonus XP to each of the other three
> (`ceil(21/2) = 11`); 10 XP → 5 each. (`:3731-3737`.)

- Source: `Ars Magica - Definitive Edition (Core Rules).md:3731-3737`.
- Data value: `virtue.elemental_magic` in `rules/core/virtues_flaws.json` carries
  `{ "type": "elemental_magic", "forms": ["art.aquam","art.auram","art.ignem","art.terram"] }`
  — the four elemental Form ids are **data**, never hardcoded in the engine. The
  `forms` set is validated to resolve to known Arts in
  `ruleset.rs::validate_effect_refs` (only when the Arts catalogue is loaded, like
  `validate_spell_refs`).
- Implementation: `effective.rs::elemental_form_bonus` (folded into
  `effective_art_score`; surfaced via `art_bonuses`). For each listed Form `F`,
  reconstruct its table-XP from its bought score via
  `art_advancement.xp_for_score`, add `bonus_xp(F) = Σ_{G≠F} ceil(xp(G)/2)`, and
  invert the sum back to a score with `art_advancement.score_for_xp`; the
  effective-over-bought delta is the boost (then flat Puissant Art stacks on top).
- **Rounding is UP** (`u32::div_ceil(2)`), matching the worked example (21 → 11).
- **Scoping**: the boost applies **only** to the four elemental Forms — never a
  Technique or a non-elemental Form (Corpus etc.), enforced by the `forms.contains`
  guard.
- This is the one **XP-space** `ArtBonus` — nonlinear in the bought score, unlike
  every flat `Effect::ArtBonus`. It is a free derived bonus: it adds **no XP-pool
  demand** (`xp_allocation` prices only bought scores).
- **Documented limitation (whole-score storage):** the engine stores whole bought
  Art **scores** + a shared XP pool, not per-Art raw XP assignments. So the
  redistribution operates on the **table-XP of the bought score**, not the raw
  assigned XP; leftover XP sitting between two score thresholds is not represented.
  A by-hand assignment that left such leftover XP will not reproduce exactly.
  Verification therefore uses scores at **clean XP thresholds** only (tests in
  `effective.rs` and `data_integrity.rs`: three Forms at score 6 = 21 XP, one at
  score 4 = 10 XP → boosted 9/9/9 and 8 on the triangular Art curve).

#### Mastered Spells / Flawless Magic — Spell Mastery (`spell_mastery_xp`, `grants_spell_mastery`)
> Mastered Spells: "You have fifty experience points to spend on mastering spells
> that you know … You may take this Virtue multiple times." Flawless Magic: "All
> your spells start with a score of 1 in the corresponding Spell Mastery Ability
> … all your Advancement Totals for Spell Mastery Abilities are doubled."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4471-4474` (Mastered
  Spells), `:3887-3889` (Flawless Magic).
- Data: `virtue.mastered_spells` — `spell_mastery_xp: 50`; `virtue.flawless_magic`
  — `grants_spell_mastery: 1`. `SpellSelection` gains an optional `mastery` field
  (bought Spell Mastery score); `SCHEMA_VERSION` bumped 7 → 8 (backward-compatible
  — old saves default `mastery: None`).
- Implementation: `Effect::SpellMasteryXp { amount }` → `effective.rs::spell_mastery_xp`
  (restricted pool, summed). `Effect::GrantsSpellMastery { score }` →
  `spell_mastery_floor` (max grant); `effective_spell_mastery(sel)` =
  `max(bought, floor)`. Surfaced as `EffectiveScores.spell_mastery_{xp,floor}` and on
  the sheet (Fluent `spell-mastery-xp`/`spell-mastery-floor`; DE "Meisterschaft").
  **Deferred:** exact mastery-XP spend validation (5×new level per point) and the
  doubled-advancement rate are in-play/advancement mechanics, not gen-time budget;
  "may take multiple times" follows the existing `max_per_target: 1` convention
  (as Improved Characteristics).

#### Templar Commander — fixed nested free Virtue grant (`grants_selection`)
> "This Virtue also grants the Temporal Influence Minor Virtue … This Virtue
> includes the effects of the Brother-Knight Virtue."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:5113-5116`.
- Data: `virtue.templar_commander` — `grants_selection: [virtue.brother_knight,
  virtue.temporal_influence]`.
- Implementation: `Effect::GrantsSelection { items }` →
  `effective.rs::vf_granted_selections`, folded into `entity_grants` alongside House
  and Mythic grants (budget-exempt, one level of nesting — a granted item's own
  `grants_selection` is not re-applied). Each granted id is validated to resolve in
  `ruleset.rs::validate_effect_refs`. The granted rows now surface through
  `EffectiveScores.granted_selections` (switched from House-only to `entity_grants`)
  so the UI shows them read-only, and their own effects apply. This is the source's
  **fixed** nested grant; the Definitive Edition's Mythic Blood does not grant a
  player-chosen free Focus/Flaw (that is a different edition), so no choice-grant
  machinery is needed here.

#### Magic Items / Redcap — starting enchanted-device level budget (`item_level_budget`)
> Magic Items: "You begin with 25 more starting levels of magic items … you may
> take it more than once." Redcap: "enchanted devices with fifty levels of effect."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4347-4349` (Magic
  Items), `:4842-4846` (Redcap).
- Data: `virtue.magic_items` — `item_level_budget: 25` + `prerequisites: Has
  virtue.redcap` ("You must be a Redcap"); `virtue.redcap` — `item_level_budget: 50`.
- Implementation: `Effect::ItemLevelBudget { amount: u16 }` →
  `effective.rs::item_level_budget` (derived, base 0 + Σ). Surfaced as
  `EffectiveScores.item_level_budget` and shown on the sheet (Fluent
  `item-levels-label/readout`; DE "Zauberartefakte"). The device-crafting subsystem
  is out of scope; the granted budget is tracked (full scope of the *Virtue*).
- **M5/5e — starting enchanted devices stored & charged.** `Entity.devices:
  Vec<EnchantedDevice { name, level: u16 }>` records the player's chosen starting
  devices; the total `level` is charged against `item_level_budget()`.
  `effective.rs::item_level_used` sums the device levels (surfaced as
  `EffectiveScores.item_level_used`), and `validation.rs::validate_devices` emits
  `over_item_level` (Fluent `issue-over_item_level`) when `used > budget`. A device
  therefore requires a granting Virtue, exactly as a starting Reputation does.

#### M5/5e — magic-possession Entity storage (aura, familiar, talisman, longevity)
Direct-entry storage for a magus's starting magic possessions. `SCHEMA_VERSION`
bumped 8 → 9 (single bump; all new fields `serde(default, skip_serializing_if)`,
so v≤8 saves load unchanged — **no migration function**, the same additive
precedent as `SpellSelection.mastery`'s 7 → 8 bump). Nothing is derived here
(that is 5i); these fields only *store* the choices.

- **Aura** — `Entity.aura: i32` (signed; a Divine aura can be a penalty).
  Persisted so 5i's self-made Longevity / lab totals are reproducible. Casting
  Score adds the aura (Core:9089); Longevity Lab Total is aura-gated (below).
- **Familiar cords** — `Entity.familiar: Option<Familiar { name, cord_gold,
  cord_silver, cord_bronze: u8 }>`. Gold = −botch dice; Silver = +Personality /
  mental resistance; **Bronze = +Soak & aging-resistance** (feeds 5i Soak /
  longevity). Source: `Ars Magica - Definitive Edition (Core Rules).md:10840-10844`.
- **Talisman attunements** — `Entity.talisman_attunements: Vec<TalismanAttunement {
  description, bonus: i8 }>`; free-text descriptor + its shape bonus (5i decides
  where each bonus applies).
- **Longevity Ritual** — `Entity.longevity_ritual: Option<LongevityRitual { source:
  LongevitySource (SelfMade|External), bonus: Option<i8> }>`. `SelfMade` leaves
  `bonus` `None` (5i computes +1 aging bonus per 5 points, round up, of the
  Creo+Corpus Lab Total); `External` carries the player-entered bonus. Source:
  `Ars Magica - Definitive Edition (Core Rules).md:10662-10672`.
- `EnchantedDevice`/`TalismanAttunement` derive `Ord` so `Entity::normalize()`
  sorts `devices` (by name) and `talisman_attunements` (by description) for
  zero-noise diffs. `LongevitySource` gets snake_case serde + `Display` (guarded by
  `display_matches_serde_scalar_for_every_enum`); the UI renders every enum through
  a Fluent key (`longevity-source-{self_made,external}`), never the raw slug.

#### M5/5g — aged / warped state, effects & identity Entity storage
Direct-entry storage (plus the derived scores computed from points) for an
already-aged / already-warped character, and free-text identity/flavor fields. The
M5 `Entity` fields were additive `serde(default, skip_serializing_if)`; the later
aging rework (A1) **removed** `aging_reductions` and bumped `SCHEMA_VERSION` 9 → 10,
with `load_entity_migrating` upgrading older saves (see the Aging points note
below). A v8 (pre-5e) and a v9 (5e) save both still load. Aging *rolls* stay in M6;
M5 only makes the raw state + effects enterable and computes the scores from points.

- **Aging points** — `Entity.aging_points: BTreeMap<Characteristic, u8>`: the
  *lifetime* accrued aging points *per Characteristic* (the sheet prints these).
  The Characteristic **drops** they force are DERIVED, never stored (see the
  "Aging lowers derived" note below). Source:
  `Ars Magica - Definitive Edition (Core Rules).md:16579`.
  - The former manual `aging_reductions` map was **removed** (schema 9 → 10):
    modelling aging fully from `aging_points` per the rule made a separate
    stored-drops field redundant and a divergence risk. `load_entity_migrating`
    (`types.rs`) migrates old saves by folding any legacy `aging_reductions[c] = R`
    into `aging_points[c]` as the *minimal* point total that reproduces `R` drops
    under the derived rule (`minimal_aging_points_for_drops`), reporting which
    Characteristics were migrated. Because every aging point counts toward
    Decrepitude — including those "lost" to a drop — the fold also corrects the old
    model's Decrepitude under-count.
- **Warping points** — `Entity.warping_points: u32`: accrued Warping Points.
- **Twilight scars** — `Entity.twilight_scars: Vec<TwilightScar { description }>`
  (free-text; `TwilightScar` derives `Ord`, so `Entity::normalize()` sorts them for
  zero-noise diffs). Source: `:9731`, `:9743`.
- **Identity/flavor** — `name`, `gender`, `sigil`, `covenant_name`, `parens`
  (`String`, skip-if-empty) and `birth_year: Option<i32>`. No mechanical effect.

##### Aging & warping annotation fields (D2 — additive, schema 10 → 11)
Four **character-only annotation** fields carrying **NO game mechanic** — the
engine computes nothing from them (covenants omit them; no validator is added).
Additive `serde(default, skip_serializing_if)`, so old saves load unchanged (no
migration code; `load_entity_migrating` is untouched). Bumped `SCHEMA_VERSION`
10 → 11. Fields in `types.rs`; `Entity::normalize()` sorts `aging_log`.

- **Apparent age** — `Entity.apparent_age: Option<u32>` (mirrors `age`).
  > "**Age:** The character's actual age, with the apparent age in parentheses."

  Source: `Ars Magica - Definitive Edition (Core Rules).md:1155`. This is a
  **pure annotation**: apparent age is the resolved *outcome* of aging rolls the
  app deliberately does **not** simulate — consistent with the `Effect::AgingMod`
  "surfaced-only" doc (`types.rs`, "the app does not simulate aging rolls").
- **Warping effect** — `Entity.warping_effect: String` (skip-if-empty), the
  source-reflecting Flaw a character gains from Warping.
  > "This Minor Flaw should reflect the predominant source of the Warping Points."

  Source: `Ars Magica - Definitive Edition (Core Rules).md:16547-16561` (### Effects
  of Warping). Deliberately **free text, NOT a Flaw `selection`**: a post-creation
  warping Flaw must not count against the creation Virtue/Flaw budget.
- **Decrepitude effect** — `Entity.decrepitude_effect: String` (skip-if-empty):
  free-text overall aging/decrepitude narrative. Pure annotation — Decrepitude
  itself is DERIVED from `aging_points` (see above). Source:
  `Ars Magica - Definitive Edition (Core Rules).md:16563-16577` (## Aging).
- **Aging log** — `Entity.aging_log: Vec<AgingLogEntry { year, effect }>`
  (skip-if-empty). `year` is the **first** field so the derived `Ord` sorts the
  log chronologically via `Entity::normalize()`. Per-year free-text outcomes;
  pure annotation (the app does not simulate the aging rolls). Source:
  `Ars Magica - Definitive Edition (Core Rules).md:16563-16577` (## Aging).

App/UI: `save_entity_to_path` (`arm-app/src/ruleset_io.rs`) now **stamps**
`arm_rules::SCHEMA_VERSION` onto the entity on write, so app-written saves never
drift from the engine version (the frontend `SCHEMA_VERSION` constant in
`ui/src/lib/state.svelte.ts` was likewise reconciled to 11). The four fields are
entered in `CharacterDetails.svelte`; every label is a Fluent key.

**Derived-score formulas** (in `effective.rs`; slice 5i's `derived.rs` re-exports /
consumes them). Both invert the **Ability** advancement table via the new
`AdvancementTable::score_for_xp(xp)` (the highest score whose cumulative `total_xp
≤ xp`) — the ×5 curve is never hand-rolled:

- **Decrepitude** — `decrepitude_score(entity, ruleset) = advancement.score_for_xp(
  Σ aging_points)`. Every aging point is 1 XP toward Decrepitude, which rises like
  an Ability (5×new score): 17 aging points → 15 ≤ 17 < 30 → **Decrepitude 2**.
  Source: `:16617`.
- **Warping (unified)** — `warping_points_total(entity, ruleset) =
  entity.warping_points + Σ WarpingGrant.points`, then `warping_score =
  advancement.score_for_xp(points_total)` (cumulative 5/15/30/50/75: 15 points →
  **Warping Score 2**). The two warping sources are routed through **one** function:
  `warping()` now returns `(warping_score, warping_points_total)`, and the
  `WarpingGrant.score` field is **ignored** for the derived score (it is asserted
  consistent — Warped by Magic's declared Score 1 equals `score_for_xp(5)`). Source:
  `:16464-16475`; grant at `:7019-7021`.

**Aging lowers derived, not creation.** The drops are DERIVED from the accrued
points by `effective.rs::aging_drops(entity, char)`: once the points **exceed** the
absolute value of the (already aged-down) score the Characteristic drops by one and
the points reset, so the simulation consumes `|score| + 1` points per drop over the
lifetime total. Worked examples encoded as tests (`:16613`): a Communication of +2
drops on its **3rd** aging point; a Stamina of −3 on its **4th**.
`effective_characteristic_after_aging(entity, ruleset, char) = (bought − drops)`
floored at the rules effective minimum (−5), **plus** any free
`CharacteristicScoreDelta` bonus (Giant Blood +1 Str/Sta, Dwarf −1) added on top —
so an aged Giant-Blood score can still reach ±6. *Decision:* the drop lowers the
*bought* score (its threshold is the bought score, per the rule text); the free
delta is a separate additive layer. This is what DERIVED / play stats consume (5i);
creation-legality validators keep reading the **un-aged bought score** from
`entity.characteristics`, so entering an aged-down character can never
retroactively make its point-buy illegal. Source: `:16579`, `:16613`.

**Validation (advisory, single path).** `validation.rs::validate_aging` emits one
**warning** (never blocking), per Characteristic whose accrued points force a drop:
`excessive_aging_reduction` — when the derived drops would push the score below the
−5 floor (it is clamped regardless; args `characteristic`, `reduction`, `min`).
Fluent key `issue-excessive_aging_reduction` (en/de). (An earlier
`aging_points_force_drop` note announcing each auto-applied drop was removed as
validation noise — the drop is automatic and already reflected in the effective
score, so it is not an entry problem worth flagging.)

App/UI: `EffectiveScores` gains `decrepitude_score: u8` and widens `warping_points`
to `u32`; the Details tab (`CharacterDetails.svelte`) enters identity fields, aging
points per Characteristic (with an `aging-points-note` explaining drops are
auto-derived), Warping Points, and twilight scars, and shows the engine-computed
Decrepitude / Warping **scores** and the aging-lowered Characteristics (never
recomputed in JS). Fluent keys en/de: identity + aging block (`identity-*`,
`aging-*` incl. `aging-points-note`, `warping-points-label`, `twilight-*`,
`decrepitude-{label,readout}`).

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
- **First granted point is free (XP charge).** A granted Supernatural Ability's
  first point costs no experience: "the character gets an initial score of 1 from
  the Virtue granting it, and you will not need to spend experience points for the
  first point of those Abilities." So `xp_allocation` charges only the score
  **above** the granted floor — `charged_cost(xp_for_score(score) −
  xp_for_score(floor), affinity)` — leaving the first point free and pricing score
  2 at the normal 1→2 step (e.g. 15 − 5 = 10 on the ×5 table). This is a
  charge-only fix: `effective_ability_score` (= `max(bought, floor) + bonuses`) and
  the stored full sheet score are unchanged, and the age-cap check (which reads the
  bought score) is unaffected. Source: `Ars Magica - Definitive Edition (Core
  Rules).md:2639` (a parenthetical inside the Mythic Companions bullet list).

#### Deferred — milestone assignments
The plan was reordered so all input for all character types lands in M4 (direct
entry) before the guided wizard in M6. Accordingly:

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
- **M6 (guided wizard):** the life-stage XP acquisition (early childhood 75+45 xp
  `:2378`; later life 15/20/10 xp/yr `:2390-2394`; age→max-score cap
  `:2368-2374`), the Sample Childhood packages (`:2380-2388`), the magus
  apprenticeship/post-apprenticeship Art-XP flow (`:2433-2471`), and the aging
  engine for characters over 35 (`:16563-16640`). The age→max-score *cap* itself
  is enforced as direct-entry validation in M4/4e; only the XP *acquisition* and
  aging *rolls* are M6.

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
  points" rule (M5).

#### New V/F & Ability definitions (structural; core effects M5, supplement effects M9)
Per CLAUDE.md each is sourced from the authoritative Markdown by book. The
supernatural **Might** effects (Infernal/Divine Might + power levels) for Demonic
Blood/Might/Powers and Strong Angelic Heritage are now **wired** (see Supernatural
Might & Magic Resistance below); the core in-play details (e.g. the Greater
Immunity target) are M5/5b.

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
(M5/5b), and the Nephilim requirement matches it by ref alone.

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

`spell_level_cap(entity, ruleset, technique, form)` (`effective.rs`) computes it
from the effective Art scores, the Intelligence characteristic, and effective
Magic Theory → a spell above it emits `spell_level_exceeds_cap` (validation, in
`validation/magus.rs`). The same function is surfaced per-Te/Fo combination as
`spell_level_caps` → `EffectiveScores.spell_level_caps` (`ruleset_io.rs`), so the
spell picker greys a spell above the cap from the one engine-authoritative value
rather than recomputing it in JS. **Approximation:** requisite-Art reduction is a
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

**Ritual legality — Range/Duration/Target + `ritual`/`creates_lasting` (5d).**
A spell carries a `ritual: bool` plus optional RDT (`SpellRange`/`SpellDuration`/
`SpellTarget`, snake_case scalars) and a `creates_lasting` marker (`spell.rs`).

> `:12283` "Formulaic and Spontaneous spells may not have Year duration"
> `:12284` "Formulaic and Spontaneous spells may not have Boundary target. They
> may have Vision target, if they are magical sense spells."
> `:12285` "Formulaic and Spontaneous spells may not have a level greater than 50.
> (Note that they may have a level of 50, but not 51 or higher.)"
> `:12290` "If the spell is a Momentary Creo spell creating a lasting thing, it
> must be a Ritual."
> `:12293` "Ritual spells are always at least level 20, even if the level
> calculation would make them lower."
> `:12055` "**Year:** … A spell with this duration must be ritual."
> `:12077` "**Boundary:** … A spell with this target must be a Ritual."
> `:12099` "**Vision**: … unlike Boundary, it does not require Ritual magic."
> `:12039`/`:12115` A Momentary Creo spell creating a lasting thing must be a
> Ritual (the Creo/Momentary interaction).

Two-level enforcement:

- **Load-time** (`Ruleset::validate_spell_refs`, `ruleset.rs`): for a fixed
  `level`, ritual ⇒ `level ≥ 20`, non-ritual ⇒ `level ≤ 50`; a non-ritual spell
  may not have `duration = Year` or `target = Boundary`, nor be a Momentary Creo
  spell with `creates_lasting`. Vision target is exempt from the Boundary rule.
- **Per-entity** (`validate_spells`, `validation.rs`): the *resolved* learned
  level (General chosen level or fixed) must obey the same ≥20 / ≤50 bounds — a
  violation emits `spell_ritual_legality` (`CODE_SPELL_RITUAL_LEGALITY`). This
  bites for General spells whose chosen level is illegal; fixed-level spells are
  already caught at load.

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
(`validate_effect_refs`). The two Parens follow `tugenden-fehler.md`
(Skilled → *Erfahrener Parens*; Weak → *Schwacher Parens*, the Latin *Parens*
kept per the Latin-term convention).

**Full core spell catalogue (5d-data).** The catalogue is the complete Core Rules
spell list, extracted from the **Spells chapter** (`:12385-15941`, from
`## Animal Spells` to the line before `# Chapter 10: Long Term Events`) by the
dev/build tool `scripts/extract_spells.py` (never loaded at runtime; see
`scripts/README.md`). Each spell is organised by `### <Technique> <Form>` section
(the source is inconsistent — one `## Creo Mentem Spells` uses two hashes, and
`### Rego Corpus Svells` is a typo — so the extractor keys a section on
Technique+Form regardless of the trailing word). The parser reads level from the
`#### LEVEL n` / `#### GENERAL` header and the mechanics from the
`R: … D: … T: …[, Ritual]` line: **ritual** = the word `Ritual` on that line (it
co-occurs with every Year/Boundary spell, and forced by them at load anyway);
`creates_lasting` = a Momentary Creo ritual. Abbreviations map to the enum
scalars (Per→personal, Arc→arcane_connection, Mom→momentary, Diam→diameter,
Ind→individual, Bound→boundary, Str→structure, …); `Special`/`Spec`
duration/target have no enum scalar and become `None`. Levels are usually
multiples of 5 but the source has individual levels 2/3/4 (e.g. Moonbeam = 3);
the load-time check does **not** require multiple-of-5. The extractor's in-loop
checks mirror the load-time gate (Technique/Form resolve to the right Art class,
requisites known, ritual⇒≥20, non-ritual⇒≤50, Year/Boundary⇒ritual, unique ids,
source ranges in-bounds) and it reports per-Technique/Form counts + ritual count.
Output is canonically sorted by id with stable formatting (deterministic
re-runs; zero-noise diffs). **The engine's load-time referential-integrity +
ritual-legality check over the whole catalogue is the trust gate.** The 14
original seed ids are all present (same spells), so `examples/`, `types.rs`, and
UI tests keep resolving. **Residual risk (recorded):** only ~10 % of entries
were hand-checked against the Markdown (36/360, spread across every
Technique/Form — all correct); transcription errors integrity cannot catch may
remain. German spell names come from
`rules/source/de/translation-tables/zauber-nach-form.md` (360/360 matched, 0 EN
fallbacks; two book-vs-table spelling variants — "Thread"/"Tread",
"Unravelling"/"Unraveling" — are bridged by an explicit alias in the extractor).

---

## Equipment: weapons / shields / armor (M5/5h)

The equipment catalogue (`equipment.rs` + `rules/core/equipment.json`) defines
weapons, shields, and armor as language-neutral records with per-row provenance.
This slice stores the character's choices (`Entity::equipment`, a `Vec<EquipmentSlot
{ item, equipped }>`) and validates references; it does **not** compute combat
totals, Soak, or Encumbrance — that lands in the derived-totals slice (5i), which
consumes these rows. The data shape is deliberately rich enough for 5i.

**Armor Table** (each material split into partial / full rows; full is `n/a` for
Quilted/Fur and Heavy Leather):

> `:16944-16949` "| Quilted/Fur | 1 | 2 | n/a | n/a … | Chain Mail | 6 | 4 | 9 | 6 |"

`Armor { protection, load }` — Prot is the Soak bonus, Load feeds Encumbrance.
10 armor rows in `equipment.json`.

**Melee Weapon Statistics** (Ability, Init, Atk, Dfn, Dam, Str, Load; shields are
rows in this table but modeled as their own `Shield` type):

> `:16959-16986` "| Dodge | Brawl | 0 | n/a | 0 | n/a | n/a | 0 … | Warhammer |
> Great | 0 | +6 | 0 | +12 | +2 | 3 |"

`:16988` names the "Ability" column ("The Weapon Ability needed to use this
weapon"). `validate_weapon_refs` (in `ruleset.rs`) requires each weapon's
`ability` to resolve to a Martial Ability, or an Ability flagged
`combat_ability` in data — Brawl (the combat Ability for unarmed/improvised
weapons, which the rules class as General). 25 melee weapons + 3 shields.

**`Ability.combat_ability` data flag** (`rules/core/abilities.json`, set on
`ability.brawl`; consumed by `validate_weapon_refs`): Brawl is the non-Martial
combat Ability, so the engine drives the weapon-Ability exception from this flag
rather than a hardcoded `ability.brawl` slug (a ruleset that slugs unarmed combat
differently just sets the flag).

> `:7337-7340` "**Brawl** Fighting hand-to-hand without weapons, or with the
> sorts of improvised weapons you just pick up, including knives. Brawl is also
> the Ability used to dodge attacks if you have no Martial Abilities."

**Missile Weapon Statistics** (adds a Range column; Thrown-Ability rows are
`WeaponKind::Thrown`, Bow-Ability rows `Missile`):

> `:17005-17011` "| Axe, Throwing | Thrown | 0 | +2 | 0 | +6 | 5 | 0 | 1 … | Bow,
> Short | Bow | –1 | +3 | 0 | +6 | 15 | –1 | 2 |"

7 missile-table weapons (32 weapons total).

**`n/a` cells** are `Option::None`: Dodge has no Attack/Damage; the body attacks
(Dodge/Fist/Kick) have no minimum-Strength — distinct from a real `0` (Fist's `+0`
Attack). `min_strength` for a weapon and a shield are met separately (`:16993`).

**Weapon + shield combine** — a weapon+shield combatant **adds both** rows'
modifiers (computed in 5i):

> `:16656` "If wielding a shield as well as a weapon, add the modifiers from the
> shield to those from the weapon."

**Encumbrance** (the Load→Burden table; Encumbrance = `max(0, Burden − max(0,Str))`)
is defined at `:17103-17123` and **computed in 5i**, not here:

> `:17109-17123` "| Total Load | Burden | | 0 | 0 | | 1 | 1 | … | 55 | 10 |"

`validate_equipment` (in `validation.rs`) emits `unknown_equipment` (error) for a
slot whose id resolves to no catalogue row, and `equipment_min_strength` (advisory
warning) when an equipped weapon/shield's min-Strength exceeds the character's
(aged) Strength — never blocking, since wielding an over-heavy weapon is a
storyguide call. German equipment names follow
`rules/source/de/translation-tables/kampf.md`.

---

## Derived play-stat totals (M5/5i) — `derived.rs`

`derived.rs` is the **read-only, pure** play-stat layer: `(&Entity, &Ruleset) →
owned structs`. It computes the in-play totals a character sheet prints (casting,
lab, penetration, magic resistance, combat, soak, encumbrance, fatigue, wounds,
longevity) and reuses `effective.rs` for effective Art/Ability scores, the
aging-adjusted Characteristic, Decrepitude, and Warping. It mutates nothing and
recomputes no creation-legality; a purity test asserts two calls are byte-equal
and leave the entity untouched. Result structs carry **labelled addend
breakdowns** (stable slug ids, mapped through Fluent `derived-*` keys on the UI
side) — the engine never emits English. **The UI computes no mechanics: it renders
these numbers.** The `derived_totals` Tauri command mirrors `effective_scores`.

**Formulas** (all `Ars Magica - Definitive Edition (Core Rules).md`):

| Total | Source | Formula |
|---|---|---|
| Casting Score | `:9089` | Technique + Form + Stamina − Encumbrance + Aura |
| Cast types | `:9103-9145` | Formulaic = score; Ritual = score + Artes Liberales + Philosophiae; Spont fatiguing = ÷2; non-fatiguing = ÷5 |
| Method Caster | `:4524-4527` | +3 flat, Formulaic/Ritual scope only |
| Non-standard casting | `:9236-9245` | Words/Gestures penalties (Formulaic/Spont, not Ritual): no voice −10, no gestures −5. Per cell, `NonStandardCasting` exposes `silent` (Formulaic − residual voice penalty), `still` (− residual gesture penalty), `silent_and_still`; residuals clamp at 0 |
| Quiet Magic | `:4822-4826` | reduces the no-voice penalty +5 per casting (soft voice → 0, no voice → −5; a second casting eliminates it) |
| Subtle Magic | `:5073-5076` | reduces the no-gesture penalty +5 (no gestures → 0) |
| Deft Form | `:3645-3648` | casting in the named Form suffers **no** non-standard voice/gesture penalty (both residuals 0 for that Form's cells) |
| Magical Focus | `:4399-4422` | within focus, add the **lower** applicable Art again (per-`(Te,Fo)` `within_focus` value; applicability is user-judged, never auto-detected) |
| Deficient Art | `:5909-5915` | totals adding that Technique/Form **halved** (Form excludes Magic Resistance) |
| Lab Total | `:4143-4154` | Int + Magic Theory + Technique + Form + Aura + flat LabTotalMod (+ focus / halving as casting) |
| Penetration | `:9159-9161` | per known spell: Casting Total − Level + Penetration score |
| Weak Magic | `:7064-7067` | halves Penetration **after** subtracting level (not the casting total) |
| Magic Resistance | `:9390-9398` | per Form: Form + 5 × Parma Magica (Form-base rule `:9390`, Parma "five times" `:9398`); Limited MR drops the Form bonus, Flawed Parma halves |
| Longevity | `:10662-10672` | self-made: +1 per 5 points (round **up**) of Creo+Corpus Lab Total (aura-gated); external: entered bonus passthrough. Bronze cord noted for aging-resistance (`:10840-10844`) |
| Masterpiece | `:4476-4479`, `:10410` | magus with the Masterpiece Virtue (`Effect::MasterpieceItem` marker) surfaces a **read-only** lesser-enchanted-item cap = **best base `(Te,Fo)` Lab Total ÷ 2** (the lesser-enchantment rule caps single-season instillation at Lab Total ≥ 2×effect level, `:10410`; vis costs ignored per the Virtue). The best Lab Total is used (magus picks the Te/Fo), no focus doubling. The engine does **not** create the device or spend an item-level budget — the player still enters the actual lesser enchanted item by hand under Magic Items; this is guidance only. `masterpiece_item_cap` / `DerivedTotals.masterpiece` |
| Combat | `:16658-16670` | Init = Qik + WpnInit − Enc + CombatMod; Attack = Dex + Ability + WpnAtk + CombatMod; Defense = Qik + Ability + WpnDef + CombatMod; Damage = Str + WpnDam + CombatMod |
| Weapon+shield | `:16656` | one `CombatLine` per equipped weapon, combining every equipped shield's Init/Atk/Def mods |
| Enc-exempt | `:17105` | Attack/Defense are **not** Encumbrance-penalized; Init **is** |
| Soak | `:16667` | Stamina + Armor Protection + SoakMod (Tough +3) + Bronze cord; Form bonus situational (entered 0) |
| Encumbrance | `:17103-17123` | Burden from Load table `[0,1,3,6,10,15,21,28,36,45,55]→[0..10]`; Enc = `max(0, Burden − max(0,Str))` |
| Fatigue | `:17127-17129` | Winded/Weary −1, Tired −3, Dazed −5, adjusted by HealthMod fatigue delta |
| Wounds | `:17167-17191` | Size unit `u = max(1, Size+5)`; Light 1..u, Medium u+1..2u, Heavy 2u+1..3u, Incap 3u+1..4u, Dead 4u+1.. ; penalties −1/−3/−5 adjusted by HealthMod wound delta |
| Decrepitude / Warping | `:16617`, `:16464-16475` | **reused** from `effective.rs` (`decrepitude_score`, `warping_score`), not reimplemented |

**Order of operations** (pinned + unit-tested): base casting/lab score → + flat
CastingTotalMod/LabTotalMod → within-focus adds the lower Art → **halve** (Deficient
Art / halving flaws) → for penetration, **− spell level** then Weak-Magic halve.
Because a Magical Focus is a free-text descriptor the engine cannot auto-detect
applicability, each `(Technique, Form)` cell carries **both** a base and a
`within_focus` value; the reader picks whichever applies (no stored per-total
toggle).

**Exhaustiveness ≠ consumption.** A single `in_play_mods` fold is an exhaustive
`match` over every `Effect` variant (so a new variant is a compile error in
`derived.rs`), but exhaustiveness alone does not prove an effect moves a number.
The per-area functions *read* each folded modifier and the unit tests assert the
number changes — those reducers **and tests** are what guarantee the 5b in-play
effects are actually consumed. Worked-example tests: Longevity Lab Total 35 → +7
(`:2573`); casting total with Encumbrance + Focus (base vs within-focus, Method
Caster +3); Deficient Technique halving; per-Form Magic Resistance = Form + 5×Parma;
Flawed Parma halving; per-spell penetration + Weak Magic; weapon+shield combat line;
Soak with Tough + Bronze cord; Encumbrance from Load; wound ranges Size 0 / +1;
Enduring Constitution penalty reduction; Decrepitude 17→2 & Warping 15→2 via the
reused functions; purity.

**Non-standard casting is computed** (no longer surfaced-only). The three penalty
relievers — Quiet Magic (`quiet_words`), Subtle Magic (`subtle_gestures`), Deft
Form (`deft_form`, now Form-parameterized) — are folded by `in_play_mods` into a
voice reduction, a gesture reduction, and a Deft-Form set, and each casting cell
carries a `NonStandardCasting` variant (`silent` / `still` / `silent_and_still`,
residuals clamped at 0). Every **other** `SpecialCastingMod` kind (spontaneous-magic
variants, circumstantial halvings) stays surfaced-only.

**Surfaced-only families** (study / aging-roll / conditional-casting /
wound-recovery: `AdvancementMod`, `AgingMod`, the non-computed `SpecialCastingMod`
kinds, `AbilityRollMod`, and the `HealthTrack::{FatigueRoll, CastingFatigue,
Recovery}` tracks) are **listed** as labelled `SurfacedModifier`s, not folded into a
simulated number, because the app does not simulate those subsystems.

---

## Per-character fields (M4/4d-rest, 4e)

The final M4 phase adds age, Confidence, Personality Traits, Reputations, and the
Gift/Supernatural gate — the remaining Core character-generation surfaces.

**Age → max Ability score.** The band table is **data**, not code: because it caps
*Ability* scores by age (not any Characteristic), it lives in
`rules/core/abilities.json` as `age_ability_caps` (parsed into
`AgeAbilityCaps` in `ability.rs`; each band is `{ max_age?, max_score }`, the
open-ended 46+ band omitting `max_age`). `AgeAbilityCaps::max_ability_score` reads
it; `effective::age_max_ability_score(ruleset, age)` and
`age_ability_cap(entity, ruleset)` surface it via `EffectiveScores.age_ability_cap`
so the UI never re-hardcodes it. A ruleset that ships no bands cannot enforce the
cap (returns `None`).

> `:2366-2374` "Your character's age determines the maximum score … | under 30 |
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
never. The grog-Loyal / warrior-Brave "should" is guided (M6), not enforced here.
The `personality` category is an **engine invariant**
(`ENGINE_REQUIRED_CATEGORY_PERSONALITY` in `ruleset.rs`): because the rule keys
off this exact slug, `Ruleset::validate_integrity` fails loudly at load if a
ruleset that ships a V/F catalogue (non-empty `point_items`) declares no item in
the category — mirroring the `ENGINE_REQUIRED_ABILITIES`/`ARTS` role checks, so a
renamed/dropped category cannot silently make the engine count zero.

**Reputations** (`Entity.reputations`, `ReputationType` = Local / Ecclesiastical /
Hermetic / **Academic**, `:1091-1101`; `:1097` names Local as "the most basic"
type with the others alongside):

> `:2514` "Characters only start with a Reputation if they choose a Virtue or Flaw
> that grants one, but all characters can develop them in play."

`Effect::GrantsReputation {kind: Option<ReputationType>, score}` authorizes one
starting Reputation; `validate_reputations` errors on any reputation beyond the
grants of its kind (`reputation_not_granted`). A grant with `kind == None` is a
**player-chosen-type** wildcard (Famous, `:3861-3863` — "Choose … one type"): it
authorizes one Reputation of *any* type; concrete-kind grants authorize only that
type (a Reputation consumes a matching concrete slot first, then a wildcard). Seed
granters: **Infamous** (`:6310-6312`, `{local, 4}`) and **Black Sheep**
(`:5703-5705`, `{local, 2}`). **Correction (M5/5a):** the earlier "only Local
granters exist in Core" claim was wrong. **Hermetic Prestige** *is* in Core at
`:4071-4073` (Hermetic Reputation level **4** — the Darius `:2518` example's 3 is
an errata slip; the Virtue text's 4 is authoritative). The full set of
reputation-granting `creation_effect` V/F is wired in slice 5a-wire (see below).
`ReputationType::Academic` was added because six scholastic Social-Status Virtues
(Baccalaureus, Cathedral School Master, Doctor in (Faculty), Magister in
Artibus/Medicina, Failed Student) confer an "Academic Reputation" — a named type
in the source. Fluent `reputation-type-academic` (en `Academic`, de `Akademisch`).

---

## Virtue/Flaw classification (M5/5a)

Every entry in `rules/core/virtues_flaws.json` carries a required `classification`
field (engine enum `Classification`, serde `snake_case`; no serde default — an
unclassified entry fails to load, and `tests/data_integrity.rs::every_vf_is_classified`
guards the acceptance criterion). This slice adds **no mechanical effects**; it is a
data-tagging + provenance pass whose output finalizes the slice-4/5b `Effect`
variant set and the slice-5a-wire creation-number wiring. The three disjoint
classes:

- **`narrative`** — no mechanical creation number and no in-play/derived-total
  effect. Personality, Story, and most Social-Status V/F are narrative by design;
  they are **never** given an invented effect. No source citation is required for a
  narrative item.
- **`creation_effect`** — changes a character-creation number/state (starting
  scores, XP grants, Confidence, Size/characteristic deltas, reputation grants,
  spell-levels, item-level budget, True Faith / Might scores, a free starting
  Supernatural Ability score, …). Includes every entry that already carries
  `effects`.
- **`in_play_effect`** — no creation-number change, but modifies an in-play/derived
  total the engine computes in slice 5i (casting / lab / penetration / magic
  resistance / combat / soak / study / aging-longevity).

Counts over the shipped catalogue (structural, not asserted as exact totals in
tests): **narrative 445**, **creation_effect 115**
(36 wired via `effects` before 5a; 68 more wired in 5a-wire; **11 deferred** —
see the 5a-wire section for the itemized deferrals),
**in_play_effect 93** (all wired in 5b). Total 653.

### Roadmap corrections applied here

- **Wealthy / Poor are `narrative`.** Core defines only advancement-*season*
  effects for them (`:5235-5237`, `:6594-6596`); there is no creation XP-per-year
  figure, so no invented creation effect is assigned (the season effect is an M6
  life-stage concern).
- **Hermetic Prestige `creation_effect`** — Core `:4071-4073`, a Hermetic
  Reputation at level **4** (the `:2518` Darius example's 3 is an errata slip; the
  Virtue text's 4 is authoritative). The stale "not in Core / Local-only" claim in
  the Reputations section above is corrected.
- **Famous `creation_effect`** — Core `:3861-3863`, a **player-chosen-type**
  Reputation at level 4 (5a-wire may add a player-selected `kind` param to
  `GrantsReputation`).

### In-play effect families (definitive input to slice 4 / 5b)

- **Magical Focus (major/minor)** — `virtue.major_magical_focus` (Core:4399-4422), `virtue.minor_magical_focus` (Core:4536-4538), `virtue.mythic_blood` (Core:4573-4589)
- **Flat casting-total bonus/penalty** — `virtue.method_caster` (Core:4524-4527), `flaw.poor_formulaic_magic` (Core:6610-6613), `flaw.afflicted_tongue` (Core:5655-5658), `virtue.life_boost` (Core:4295-4298), `virtue.leper_magus` (Core:4249-4252), `virtue.cyclic_magic_positive` (Core:3635-3638), `flaw.cyclic_magic_negative` (Core:5893-5896), `virtue.special_circumstances` (Core:4998-5001), `virtue.ways_of_the_land` (Core:5231-5234), `flaw.corrupted_spells` (Core:5859-5864), `flaw.susceptibility_to_divine_power` (Core:6815-6818)
- **Spontaneous-magic casting modifier** — `flaw.weak_spontaneous_magic` (Core:7084-7089), `virtue.diedne_magic` (Core:3675-3682), `virtue.faerie_raised_magic` (Core:3829-3842), `virtue.spell_improvisation` (Core:5002-5005), `virtue.life_linked_spontaneous_magic` (Core:4299-4306)
- **Art-halving (Technique / Form)** — `flaw.deficient_technique` (Core:5913-5915), `flaw.deficient_form` (Core:5909-5912)
- **Circumstantial casting/lab halving** — `flaw.deleterious_circumstances` (Core:5917-5920), `flaw.environmental_magic_condition` (Core:6020-6023), `flaw.short_ranged_magic` (Core:6737-6740)
- **Flat lab-total bonus/penalty** — `virtue.adept_laboratory_student` (Core:3368-3371), `virtue.aristotelian_training` (Core:3440-3443), `virtue.inventive_genius` (Core:4151-4154), `flaw.creative_block` (Core:5873-5876), `flaw.weak_scholar` (Core:7080-7083), `flaw.disjointed_magic` (Core:5972-5975), `flaw.the_constant_expression` (Core:5821-5838), `virtue.potent_magic_major` (Core:4740-4781), `virtue.potent_magic_minor` (Core:4740-4781)
- **Lab-total halving** — `flaw.weak_enchanter` (Core:7060-7063), `flaw.difficult_longevity_ritual` (Core:5962-5965)
- **Ritual effective-level bonus** — `virtue.mercurian_magic` (Core:4514-4523)
- **Penetration-total modifier** — `flaw.weak_magic` (Core:7064-7067)
- **Magic-resistance modifier** — `flaw.flawed_parma_magica` (Core:6142-6145), `flaw.limited_magic_resistance` (Core:6346-6349), `flaw.weak_magic_resistance` (Core:7068-7071), `flaw.susceptibility_to_faerie_power` (Core:6819-6822), `flaw.susceptibility_to_infernal_power` (Core:6823-6826), `virtue.commanding_aura` (Core:3579-3596)
- **Flat Soak bonus/penalty** — `virtue.tough` (Core:5145-5147), `flaw.frail` (Core:6190-6193)
- **Wound/fatigue penalty delta** — `virtue.enduring_constitution` (Core:3751-3754), `flaw.low_tolerance` (Core:6366-6369), `flaw.painful_magic` (Core:6574-6577), `flaw.vulnerable_casting` (Core:6993-7004), `virtue.withstand_casting` (Core:5261-5282), `flaw.obese` (Core:6516-6519), `flaw.short_of_breath` (Core:6733-6736), `virtue.long_winded` (Core:4327-4330)
- **Wound-recovery modifier** — `flaw.fragile_constitution` (Core:6186-6189), `virtue.rapid_convalescence` (Core:4834-4837)
- **Combat total modifier (atk/def/init/dam)** — `virtue.berserk` (Core:3500-3503), `flaw.hobbled` (Core:6260-6263), `flaw.lame` (Core:6330-6333), `flaw.missing_hand` (Core:6438-6441), `flaw.missing_eye` (Core:6434-6437), `flaw.poor_eyesight` (Core:6606-6609), `flaw.palsied_hands` (Core:6578-6581), `flaw.slow_reflexes` (Core:6763-6766), `virtue.lightning_reflexes` (Core:4311-4314), `virtue.fast_caster` (Core:3865-3868)
- **Study source-quality / advancement modifier** — `virtue.apt_student` (Core:3422-3425), `virtue.book_learner` (Core:3519-3522), `virtue.free_study` (Core:3937-3940), `virtue.good_teacher` (Core:3971-3974), `virtue.independent_study` (Core:4115-4118), `virtue.study_bonus` (Core:5056-5072), `flaw.unimaginative_learner` (Core:6915-6918), `flaw.poor_student` (Core:6626-6628), `flaw.incomprehensible` (Core:6294-6297), `virtue.secondary_insight` (Core:4892-4895), `flaw.loose_magic` (Core:6354-6357)
- **Aging / longevity modifier** — `flaw.age_quickly` (Core:5659-5662), `flaw.baneful_circumstances` (Core:5687-5690), `flaw.monstrous_blood` (Core:6454-6467), `virtue.bee_king` (Core:3484-3499), `virtue.faerie_blood` (Core:3797-3820), `virtue.magical_blood` (Core:4359-4372), `virtue.unaging` (Core:5187-5190), `flaw.bound_to_role_role` (Core:5735-5748), `flaw.leprosy` (Core:6338-6341), `flaw.poor_living_conditions` (Core:6618-6621), `virtue.mild_aging` (Core:4528-4531), `virtue.magian_lineage_major` (Core:4339-4346), `virtue.magian_lineage_minor` (Core:4339-4346)
- **Non-standard-casting penalty removal (Deft/Quiet/Subtle)** — **computed** into per-cell `NonStandardCasting` variants (`derived.rs`), not surfaced-only. Base Words/Gestures penalties `:9236-9245` (no voice −10, no gestures −5). `virtue.quiet_magic` (Core:4822-4826, +5 voice per casting, second casting eliminates), `virtue.subtle_magic` (Core:5073-5076, +5 gesture), `virtue.deft_form` (Core:3645-3648, Form-parameterized, waives both for that Form). Residuals clamp at 0.
- **Flat ability-total bonus (Concentration)** — `virtue.academic_concentration_subject` (Core:3362-3367)


### In-play effect variants (M5/5b) — implemented

Slice 5b adds 13 `Effect` variants (`types.rs`) representing all 18 in-play
families at full scope, and wires **all 93** `in_play_effect` V/F in
`rules/core/virtues_flaws.json` to them. Every variant is a **no-op** in
`effective.rs`/`validation.rs`/`ruleset.rs` (asserted by
`in_play_effects_do_not_perturb_creation_totals`); the exhaustive `match` (no
`_`) forces slice 5i (`derived.rs`) to consume each. "Computed by 5i" = folded
into a simulated derived total; "surfaced-only" = the app does not simulate that
subsystem, so 5i presents the data labelled in the read-out.

New scalar enums (all `Display` == snake_case serde, guarded by
`display_matches_serde_scalar_for_every_enum`): `CastingScope`, `HalvableTotal`,
`CombatStat`, `HealthTrack`, `MagicResistanceEffect`, `AgingEffect`,
`AdvancementSource`, `SpecialCasting`. Two new `ParameterDomain` values
(`Technique`, `Form`) resolve against the Art catalogue *and* enforce `ArtType`,
so Deficient Technique cannot target a Form (and vice-versa) — the art-class
restriction, checked in `validation::validate_parameters` and, statically, in
`ruleset::validate_effect_refs`. The **one-focus-per-magus** limit (Core:4542) is
a validation rule `validate_magical_focus` (issue code `multiple_magical_foci`,
Fluent `issue-multiple_magical_foci` in en/de) that **counts** the `MagicalFocus`
effect across selections + grants — so it also catches two Minor Foci with
distinct descriptors, which pairwise `incompatible_with` could not.

| Variant | Family / representative V/F | Source | 5i |
|---|---|---|---|
| `MagicalFocus { param(Text), major }` | Magical Focus — major/minor/mythic_blood | Core:4399-4422, 4536-4542, 4573-4589 | computed |
| `CastingTotalMod { amount, scope }` | Flat casting bonus/penalty — method_caster (+3 formulaic_ritual), poor_formulaic_magic (−5 formulaic), afflicted_tongue, cyclic_magic ±3, special_circumstances, ways_of_the_land, potent_magic | Core:4524-4527, 6610-6613, 5655-5658, 3635-3638, 5893-5896, 4998-5001, 5231-5234, 4740-4781 | computed (conditional ones toggled) |
| `LabTotalMod { amount }` | Flat lab bonus/penalty — adept_laboratory_student (+6), inventive_genius (+3), aristotelian_training (+1), creative_block (−3), weak_scholar (−6), the_constant_expression (−3), cyclic_magic, potent_magic | Core:3368-3371, 4151-4154, 3440-3443, 5873-5876, 7080-7083, 5821-5838, 4740-4781 | computed |
| `DeficientArt { param(Technique\|Form) }` | Art-halving — deficient_technique, deficient_form | Core:5913-5915, 5909-5912 | computed |
| `MagicTotalHalving { total }` | Halve spont casting / lab-enchant / lab-longevity / penetration / MR — weak_spontaneous_magic, weak_enchanter, difficult_longevity_ritual, weak_magic, flawed_parma_magica, weak_magic_resistance | Core:7084-7089, 7060-7063, 5962-5965, 7064-7067, 6142-6145, 7068-7071 | computed |
| `SoakMod { amount }` | Flat Soak — tough (+3), frail (−3), berserk (+2) | Core:5145-5147, 6190-6193, 3500-3503 | computed |
| `CombatMod { amount, target }` | Combat init/atk/def — berserk, hobbled, lame, missing_hand, missing_eye, poor_eyesight, palsied_hands, slow_reflexes, lightning_reflexes, fast_caster | Core:3500-3503, 6260-6263, 6330-6333, 6438-6441, 6434-6437, 6606-6609, 6578-6581, 6763-6766, 4311-4314, 3865-3868 | computed (conditional ones labelled) |
| `HealthMod { track, amount }` | Wound/fatigue penalty (enduring_constitution, low_tolerance), fatigue rolls (obese, short_of_breath, long_winded), casting-fatigue (painful_magic, vulnerable_casting, withstand_casting), recovery (fragile_constitution, rapid_convalescence) | Core:3751-3754, 6366-6369, 6516-6519, 6733-6736, 4327-4330, 6574-6577, 6993-7004, 5261-5282, 6186-6189, 4834-4837 | wound/fatigue computed; fatigue-roll/casting-fatigue/recovery surfaced |
| `MagicResistanceMod { kind }` | Non-halving MR — limited_magic_resistance (no_form_bonus), susceptibility faerie/infernal/divine, commanding_aura & special_circumstances (aura_bonus) | Core:6346-6349, 6819-6826, 6815-6818, 3579-3596 | computed/surfaced |
| `AgingMod { kind, amount }` | Aging/longevity — age_quickly, baneful_circumstances, monstrous_blood (−1), bee_king, faerie_blood (−1), magical_blood (−1), unaging, bound_to_role, leprosy, poor_living_conditions, mild_aging, magian_lineage major/minor | Core:5659-5662, 5687-5690, 6454-6467, 3484-3499, 3797-3820, 4359-4372, 5187-5190, 5735-5748, 6338-6341, 6618-6621, 4528-4531, 4339-4346 | surfaced (app does not simulate aging rolls) |
| `AdvancementMod { source, amount }` | Study/teaching — apt_student (+5 taught), book_learner (+3 book), free_study (+3 vis), good_teacher, independent_study, study_bonus, secondary_insight, unimaginative_learner, poor_student, incomprehensible, loose_magic | Core:3422-3425, 3519-3522, 3937-3940, 3971-3974, 4115-4118, 5056-5072, 4892-4895, 6915-6918, 6626-6628, 6294-6297, 6354-6357 | surfaced (app does not simulate advancement) |
| `SpecialCastingMod { kind, param? }` | Casting-style quirks — deft_form (Form-parameterized), quiet_magic, subtle_magic, diedne_magic, faerie_raised_magic, life_linked_spontaneous_magic, spell_improvisation, mercurian_magic, life_boost, leper_magus, and circumstantial halvings (deleterious_circumstances, environmental_magic_condition, short_ranged_magic, corrupted_spells, disjointed_magic) | Core:3645-3648, 4822-4826, 5073-5076, 9236-9245, 3675-3682, 3829-3842, 4299-4306, 5002-5005, 4514-4523, 4295-4298, 4249-4252, 5917-5920, 6020-6023, 6737-6740, 5859-5864, 5972-5975 | **deft_form/quiet_magic/subtle_magic computed** into per-cell `NonStandardCasting` (silent/still/silent_and_still); all other kinds surfaced (conditional penalties). `deft_form`'s `param` names the affected Form and is load-validated (`validate_effect_refs`, `ParameterDomain::Form` required) exactly as `DeficientArt`'s param, so a missing/wrong-domain key fails loudly instead of silently voiding the waiver in `in_play_mods` |
| `AbilityRollMod { param(Text), amount }` | Ability-roll bonus in a subject — academic_concentration_subject (+3) | Core:3362-3367 | surfaced |

**Modeling notes / accepted approximations** (each surfaced in 5i's labelled
read-out, so precision is not lost to the player): `weak_spontaneous_magic` maps
to `MagicTotalHalving { spontaneous_casting }` though the book rule is "always
÷5" (5i implements the exact spont rate). Rank-dependent `commanding_aura` (MR
25/soak +5 … MR 10/soak +2) is wired as `MagicResistanceMod { aura_bonus }` only;
the rank-specific numbers are surfaced. `mythic_blood`'s bundled formulaic/ritual
fatigue benefits beyond its Minor Focus are surfaced in the item text.
`incomprehensible`/`loose_magic` halve advancement (amount 0 marker; surfaced).
Conditional `CastingTotalMod`/`CombatMod`/`LabTotalMod` addends (cyclic magic,
potent magic, special circumstances, missing_eye's ranged −3, etc.) are shown as
toggleable/labelled addends in 5i rather than always-on numbers.


### `creation_effect` V/F wiring (M5/5a-wire) — implemented

Slice 5a-wire wired **68 of the 79** previously-unwired `creation_effect` V/F to
existing `Effect` variants (plus `ReputationType::Academic` and an optional
`kind` on `GrantsReputation`; **no new `Effect` variant**). Green under the full
gate; tested in `data_integrity.rs` (`shipped_reputation_granters_authorize_their_kind`,
`shipped_famous_authorizes_any_reputation_kind`, `shipped_supernatural_virtues_grant_starting_score`,
`shipped_xp_granters_add_restricted_pool`, `shipped_confidence_true_faith_and_size_granters`).

**Variants reused:** Supernatural starting scores → `ability_score_grant {ability, 1}`
(the Second Sight pattern; each id verified in `abilities.json`, e.g.
Enchanting (Ability) → `ability.enchanting`). Reputation granters →
`grants_reputation {kind?, score}` with source-verified audience+level (Famous →
`{score:4}` wildcard; Hermetic Prestige → `{hermetic,4}`; the six scholastic
Virtues → `{academic,N}`; two-audience items Failed Monk and Senior Clergy carry
*two* effects, one per audience). XP grants → `restricted_ability_xp {amount,
abilities?, categories?}` scoped to the source's eligible list (Arcane Lore →
`{50, [arcane]}`; Feral Upbringing → `{120, …}`; "any Ability" items — Mentored
by Demons, Lone Redcap's 300 apprenticeship xp — use all five categories).
Confidence → `confidence_bonus` (Ferocity `{+1,+3}`; Low Self-Esteem `{-1,-3}`,
which cancels the standard 1/3 default; note: additive, so it zeroes only the
default). True Faith → `true_faith_grant` (Relic 1, Powerful Relic 3). Size →
`size_delta` (Blood of the Nephilim +1). Nested V/F grants → `grants_selection`
(Faerie Doctor → Dowsing; Strong Faerie Blood & Spirit Votary → Second Sight;
Ineslemen → Noncombatant Flaw; Lone Redcap → Well-Traveled; Rosh Beth Din →
Social Contacts).

**Famous player-chosen kind** is modeled by making `GrantsReputation.kind` an
`Option<ReputationType>` (`None` = any type), rather than a new param domain:
`validate_reputations` treats a `None` grant as a single any-type slot, and
`arm-app`'s `EffectiveScores` expands it to one `ReputationGrant` per type for the
UI's add-controls (so the app/UI boundary keeps a concrete `kind`).

**Now wired — supernatural Might (4):** Demonic Blood/Might/Powers (*Infernal*),
Strong Angelic Heritage (*Divine*) are wired via the `might_grant` / `power_levels`
`Effect` variants added in this slice — see **Supernatural Might & Magic
Resistance** below. Devil Child and Nephilim are modelled as Mythic-Companion type
profiles (`mythic_companion_types.json`, M4 Phase 4/5); with the 4 V/F now carrying
real Might/power effects, a Devil Child resolves end-to-end to a nonzero effective
Infernal Might + power-levels budget (tested in `arm-app`'s
`devil_child_resolves_infernal_might_and_power_budget_end_to_end`).

**Deferred (3), source-not-present or no clean creation number:**
- **Savantism, Simple Student, Corrupted Arts (3 XP)** — Savantism
  (:6703) *halves* starting XP (multiplicative; no variant); Simple Student (:4958)
  is 30 xp *per finished year* (age/life-stage, M6); Corrupted Arts (:5853) has no
  creation XP figure (its ±3 casting swing / ±5 Art xp are in-play). (Elemental
  Magic, :3731, is now implemented in slice 5c — see its section above.)

#### Supernatural Might & Magic Resistance (RoP: Magic / Infernal / Divine)

A supernatural being has a **Might Score** aligned to one **Realm** (`Realm::{Magic,
Faerie, Divine, Infernal}`). The general rule:

> "Magic Might gives the character innate Magic Resistance equal to its Might
> Score, and this does not stack with other forms of resistance … they must use
> either their Parma or their Might for their base Magic Resistance."
> — *Realms of Power: Magic.md:1472* (general; also Core:2623-2631, :2627 "these
> totals do not stack … use the higher total").

**MR-from-Might formula** (`derived::magic_resistance`): per Form, `total = Form
bonus + max(5 × Parma, Might Score)`. Might and Parma do **not** stack — the higher
is the base (labelled `might` or `parma` addend); the Form bonus is compatible with
either. A pure being (no Parma, Art 0) therefore gets a flat blanket MR = Might
Score on every Form. `derived_totals` now surfaces Magic Resistance for a
Might-being, not only a magus.

**Data model.** `Entity.might: Option<MightScore{realm, score}>` is the base the
player enters (may be 0); `Entity.powers: Vec<SupernaturalPower{name, level}>` are
free-text powers charged against a power-levels budget (mirroring `devices` vs
`item_level_budget` — the engine is not a power *designer*). Two additive `Effect`
variants (SCHEMA_VERSION unchanged at 9 — both `#[serde(default)]`, old saves load):
- `Effect::MightGrant{realm, score}` — summed (same Realm) on top of the entered
  base by `effective::effective_might`; a `score` 0 grant establishes the Realm
  without adding points.
- `Effect::PowerLevels{amount}` — summed by `effective::power_levels_budget`.

**The four V/F grants (verified against source):**

| Virtue | Grant | Source (file:line) |
|--------|-------|--------------------|
| `virtue.demonic_blood` (Major) | Infernal Might **5** + **30** power levels | *Infernal*:4120, :4122 |
| `virtue.demonic_might` (Minor, req. Demonic Blood) | Infernal Might **+2** | *Infernal*:4136 |
| `virtue.demonic_powers` (Minor, req. Demonic Blood) | **+20** power levels | *Infernal*:4142 |
| `virtue.strong_angelic_heritage` (Minor, req. Blood of the Nephilim) | Divine Might = **age ÷ 20** (entered by hand) + **30** power levels | *Divine*:1975, :1977 |

Effective Might = entered base (may be 0/None) + Σ same-Realm `MightGrant` scores.
So Demonic Blood alone → Infernal Might 5; with Demonic Might → 7. Strong Angelic
Heritage's Divine Might is age-derived (no fixed constant), so it grants
`might_grant{divine,0}` (establishing the Realm + MR) plus its 30 power levels; the
player enters the age÷20 base. Validation: `validate_powers` (used ≤ budget →
`over_power_levels` error, mirroring devices); `validate_might` (base Realm vs
granted Realm → `might_realm_mismatch` warning).

The per-item audit that fed this wiring follows (source line-ranges retained).

**Confidence:**
- `flaw.low_self_esteem` (Core:6362-6365) — Confidence (no score/points)
- `virtue.ferocity` (Core:3873-3876) — Confidence score 1 points 3

**Free nested V/F grant:**
- `virtue.devil_child` (Realms of Power - The Infernal:4144-4149) — grants free Minor Virtue
- `virtue.faerie_doctor` (Realms of Power - Faerie:6394-6399) — grants free Virtue (Dowsing)
- `virtue.nephilim` (Realms of Power - The Divine (Revised):3485-3513) — grants free Virtue

**Free starting Supernatural Ability score:**
- `virtue.animal_ken` (Core:3414-3417) — free starting Supernatural Ability score
- `virtue.corpse_magic` (Core:3605-3608) — free starting Supernatural Ability score
- `virtue.crafters_healing` (Core:3617-3620) — free starting Supernatural Ability score
- `virtue.embitterment` (Core:3739-3742) — free starting Supernatural Ability score
- `virtue.enchanting_ability` (Core:3747-3750) — free starting Supernatural Ability score
- `virtue.entrancement` (Core:3767-3770) — free starting Supernatural Ability score
- `virtue.font_of_knowledge` (Core:3921-3924) — free starting Supernatural Ability score
- `virtue.hex` (Core:4075-4078) — free starting Supernatural Ability score
- `virtue.induction` (Core:4119-4122) — free starting Supernatural Ability score
- `virtue.magic_sensitivity` (Core:4351-4354) — free starting Supernatural Ability score
- `virtue.persona` (Core:4710-4713) — free starting Supernatural Ability score
- `virtue.sense_passions` (Core:4930-4933) — free starting Supernatural Ability score
- `virtue.shapeshifter` (Core:4946-4949) — free starting Supernatural Ability score
- `virtue.spirit_votary` (Realms of Power - Magic:5480-5486) — grants free Virtue (Second Sight)
- `virtue.strong_faerie_blood` (Core:5032-5047) — free starting Second Sight ability
- `virtue.summon_animals` (Core:5085-5088) — free starting Supernatural Ability score
- `virtue.whistle_up_the_wind` (Core:5243-5246) — free starting Supernatural Ability score
- `virtue.wilderness_sense` (Core:5247-5250) — free starting Supernatural Ability score

**Item-level budget:**
- (Magic Items / Redcap wire `item_level_budget`; see slice 5e.)

**Might / power budget (wired, this slice):**
- `virtue.demonic_blood` (Realms of Power - The Infernal:4120, :4122) — `might_grant{infernal,5}` + `power_levels{30}`
- `virtue.demonic_might` (Realms of Power - The Infernal:4136) — `might_grant{infernal,2}` (adds +2)
- `virtue.demonic_powers` (Realms of Power - The Infernal:4142) — `power_levels{20}`
- `virtue.strong_angelic_heritage` (Realms of Power - The Divine (Revised):1975, :1977) — `might_grant{divine,0}` + `power_levels{30}` (Divine Might = age÷20 entered by hand; grant establishes Realm)

**Reputation grant:**
- `flaw.apostate` (Core:5675-5678) — reputation grant bad score 4
- `flaw.failed_journeyman` (Core:6060-6063) — reputation grant bad score 2
- `flaw.failed_master` (Core:6064-6067) — reputation grant bad score 4
- `flaw.failed_monk` (Core:6068-6071) — reputation grant poor score 2
- `flaw.failed_student` (Core:6072-6075) — reputation grant academic score 2
- `flaw.feral_scent` (Core:6106-6109) — reputation grant negative score 2
- `flaw.gabai` (Core:6198-6201) — reputation grant negative score 2
- `flaw.hedge_wizard` (Core:6240-6243) — reputation grant hermetic score 3
- `flaw.infamous_master` (Core:6314-6317) — reputation grant hermetic score 3
- `flaw.outlaw` (Core:6542-6545) — reputation grant score 2
- `flaw.outlaw_leader` (Core:6546-6549) — reputation grant score 3
- `flaw.outsider_major` (Core:6550-6561) — reputation grant bad score 1-3
- `flaw.outsider_minor` (Core:6550-6561) — reputation grant bad score 1-3
- `flaw.usurer` (Core:6951-6954) — reputation grant poor score 4
- `virtue.baccalaureus` (Core:3470-3475) — XP grant 90 + reputation grant academic score 1
- `virtue.cathedral_school_master` (Core:3549-3554) — XP grant 240 + reputation grant academic score 2
- `virtue.doctor_in_faculty` (Core:3683-3698) — XP grant 300 + reputation grant academic score 3
- `virtue.famous` (Core:3861-3864) — reputation grant player-chosen score 4
- `virtue.hermetic_prestige` (Core:4071-4073) — reputation grant hermetic score 4
- `virtue.lone_redcap` (Core:4319-4326) — XP grant 300 + grants Virtue + reputation grant poor score 2
- `virtue.magister_in_artibus` (Core:4385-4394) — XP grant 240 + reputation grant academic score 2
- `virtue.magister_in_medicina` (Core:4395-4398) — XP grant 300 + reputation grant academic score 3
- `virtue.master_bard` (Core:4457-4462) — XP grant 240 + reputation grant local score 3
- `virtue.physician_of_salerno` (Core:4732-4735) — XP grant 50 + reputation 2
- `virtue.rard` (Core:3476-3479) — reputation grant local score 1
- `virtue.rosh_beth_din` (Core:4878-4883) — XP grant 50 + reputation grant good score 2 + grants Virtue
- `virtue.senior_bard` (Core:4904-4909) — XP grant 90 + reputation grant local score 2
- `virtue.senior_clergy` (Core:4910-4921) — reputation grant score 4
- `virtue.templar_office_holder` (Core:5121-5124) — reputation grant score 2

**Size/characteristic delta:**
- `virtue.blood_of_the_nephilim` (Realms of Power - The Divine (Revised):1941-1954) — size delta + Dominion Lore

**True Faith score:**
- `virtue.powerful_relic` (Core:4782-4787) — True Faith score 3
- `virtue.relic` (Core:4852-4855) — True Faith score 1

**XP grant:**
- `flaw.corrupted_arts` (Core:5853-5858) — grants XP swing at creation + situational casting
- `flaw.feral_upbringing` (Core:6110-6113) — XP grant 120
- `flaw.savantism` (Core:6703-6708) — halves starting XP
- `virtue.arcane_lore` (Core:3430-3435) — XP grant 50
- `virtue.clan_ilfetu` (Core:3563-3566) — XP grant 50
- `virtue.craft_guild_training` (Core:3613-3616) — XP grant 50
- `virtue.elemental_magic` (Core:3731-3738) — Art XP distribution at creation (implemented, slice 5c: `Effect::ElementalMagic` XP-space Art boost — see section above)
- `virtue.falconer` (Core:3847-3852) — XP grant 50
- `virtue.forge_companion` (Core:3925-3928) — XP grant 50
- `virtue.hermetic_experience` (Core:4063-4066) — XP grant 50
- `virtue.ineslemen` (Core:4123-4126) — XP grant 50 + grants Minor Flaw
- `virtue.marshal` (Core:4449-4456) — XP grant 50
- `virtue.master_of_kennels` (Core:4467-4470) — XP grant 50
- `virtue.mentored_by_demons` (Core:4496-4499) — XP grant 50
- `virtue.schooled_in_crime` (Core:4884-4887) — XP grant 50
- `virtue.shadchan` (Core:4934-4939) — XP grant 50
- `virtue.simple_student` (Core:4958-4963) — XP grant 30/yr
- `virtue.trained_assassin` (Core:5153-5156) — XP grant 50
- `virtue.venditor` (Core:5207-5210) — XP grant 50


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
