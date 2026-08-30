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

(The aging-roll threshold used to sit here as `AGING_ROLLS_START_AGE`. Slice 6b6
introduced `rules/core/aging.json`, so it is now a data value — see
**Aging (M6/6b6)** below.)

### Enforcement logic

#### Virtue/Flaw balance — Virtues must be funded by Flaws
> "Players start with no points for buying Virtues and Flaws, and thus must take
> Flaws if they want Virtues. A central character may have up to ten points of
> Flaws ... and the same number of points of Virtues."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2774`, `:2297`
  (companions), `:2303` (magi).
- Implementation: `crates/arm-rules/src/validation/balance.rs` — `validate_balance`
  (:23), `compute_balance` (:168). Emits `unbalanced_virtues` (error) when spent virtue points
  exceed flaw points granted, plus the `over_budget_*` totals. (Per-type point
  totals are data; see below.)

#### Caps on Major virtues/flaws count
> "You may not take Major Virtues or Flaws" (grogs).

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2824-2830` (grogs).
- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_caps` (:22)
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
- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_caps` (:22), error
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
- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_caps` (:22). These
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
- **The same caps are also the guided wizard's V/F guidance**
  (guided-creation-review-2026-08 #7). The Virtues & Flaws step used to state only
  the point budget; it now appends one sentence per cap, generated from the very
  same `flaw_category_caps`, so no rules figure is frozen into a translated
  string. `ui/src/lib/derive.ts::flawCapNotes` maps each cap that is **not**
  `major_only` onto the Fluent key `wizard-guidance-<category>-flaw-cap`, passing
  `cap` (the maximum, as a **number**, so Fluent's `[0]`/`[1]`/`*[other]` variants
  can pick the grammatical form) and `rule` (`hard` | `soft`, straight off the
  cap's `hard` flag — which is precisely the book's own "may not" vs "should
  not"). The per-type wording it reproduces: grog `:2826-2827`, companion
  `:2837-2838`, Mythic Companion `:2850-2851`, magus `:2861-2862`, over the
  general statements at `:2818` (Story ceiling) and `:2820` (Personality
  ceiling + Major cap).
- `major_only` caps are deliberately **left out of the guidance**: they are hard
  caps the validator already reports as errors
  (`too_many_major_<category>_flaws`), and restating them in an advisory
  paragraph would state the same rule twice. `virtue_category_caps` is likewise
  not walked — every shipped entry is `major_only` and hard.
- **There is NO "at least one Story Flaw" rule**, and no guidance string may claim
  one. `:2818` and `:2837` give Story Flaws a recommended *ceiling* of one and no
  minimum whatsoever. The only "at least one" the rules state is the magus's
  **Hermetic** Flaw (`:2860`), which the guidance carries as its own separate
  clause (`wizard-guidance-hermetic-flaw`, worded exactly as the
  `missing_hermetic_flaw` warning) under the same condition the engine uses — a
  magus whose profile names at least one `gift_categories` entry.
  `ui/src/lib/i18n.test.ts` asserts the absence in both locales.

#### Tainted Virtues/Flaws — the "Type" tag + half-of-taken cap
> "Tainted Virtues and Flaws are associated with the Infernal realm ... no more
> than half a character's Virtues should be tainted, and similarly for Flaws. ...
> Supernatural abilities granted by Tainted Virtues or Flaws are always Infernal
> powers." (`:3000`)

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2998-3002`.
- Data: the descriptor's optional "Type" token maps to `PointItem.tainted`
  (`bool`, default false) in `rules/core/virtues_flaws.json`.
- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_tainted_cap` (:146).
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
domain }`). `ParameterPicker.svelte` renders a **dropdown** for every domain except
`text`, which alone gets a free-text input (its `{:else}` branch).

- **Catalogue-ref domains** (`ability`, `art`, `technique`, `form`, `characteristic`,
  `item`): the value must resolve; e.g. the `(Ability)` items
  (`flaw.careless_with_ability`, …) and the `(Form)` items. Puissant/Affinity/Great/Poor
  already used this.
- **`technique` / `form` are `art` narrowed to one Art class.** The engine validates
  such a value as an Art id **and** as that `ArtType`, raising `unknown_param_value`
  otherwise, so the two share one `<select>` branch with `art` and differ only in the
  option list — built by the shared `artsOfType(ruleset, artType)` in `derive.ts`, the
  same helper the Spells tab's Technique/Form filters and the meta-magic Vim spells'
  target Form read. One Art picker, not three.
- **Five items had the wrong domain (guided-creation review #5, fixed in Slice 7).**
  Their source restricts the parameter to a **Form**, but they declared the wider
  `art`, which the engine cannot catch because `art` accepts either class. Now
  `domain: "form"`, each verified against its own cited range:
  - `flaw.form_monstrosity` — "a monstrous feature, or mutation, which corresponds to
    a magical Form", plus an examples table headed `Form` listing Forms only
    (`Ars Magica - Definitive Edition (Core Rules).md:6162-6185`)
  - `flaw.hunger_for_form_magic` — "1 pawn of vis each season, corresponding to the
    Form that it has been mostly exposed to" (`:6276-6279`)
  - `virtue.extractor_of_form_vis` — "only if the features of the aura exemplify the
    Form … (once for each Form)" (`:3779-3782`)
  - `virtue.imbued_with_the_spirit_of_form` — "any being with a Magic Might associated
    with the Form of this Virtue" (`:4085-4094`)
  - `virtue.master_of_form_creatures` — "beings whose Magic Might is aligned with a
    particular Form … once for each Form" (`:4463-4466`)

  Already correct and deliberately untouched: `virtue.deft_form` (`:3645-3648`),
  `flaw.deficient_form` (`:5909-5912`), `flaw.deficient_technique` (`:5913-5915`).
  Correct by design as `art`: `virtue.affinity_art` and `virtue.puissant_art`, where
  **either** Art class is legal. Pinned by `tests/data_integrity.rs`
  (`form_restricted_virtues_flaws_declare_the_form_domain`,
  `virtue_deft_form_declares_the_form_domain`,
  `the_deficient_art_flaws_declare_their_own_art_class`,
  `items_legal_for_either_art_class_keep_the_art_domain`).
  A `domain` narrowing is a **ruleset** change, not an entity one: a save storing a
  Form under one of these five stays valid, while one storing a Technique now reports
  `unknown_param_value` — the correct, visible outcome rather than a silent
  re-interpretation.
- **`item`** is in the closed domain enum but declared by **no** shipped catalogue
  entry. The picker carries its branch anyway (the enum is exhaustive, and a
  point-item id must never be typed by hand); its menu is the whole point-item
  registry, because nothing in the data narrows it. No catalogue data was invented
  for it.
- **Free-text domain** (`ParameterDomain::Text`, serde `"text"`): the parenthetical
  is a free choice with no registry — `(Realm)`, `(Land)`, `(Subject)`, `(Sin)`,
  `(Beings)`, `(Terrain)`, `(Commodity)`, `(Faculty)`, `(Role)`, plus the mixed
  `Necessary (Realm) Aura for (Ability)` (a `text` + an `ability`). `validate_parameters`
  accepts any value for a `text` param (no resolution); the UI text input already
  existed. Param hints come from Fluent `param-label-<key>`.
- **Name-qualifiers — not params, stay literal by design:** `(Dove)`, `(the Wolf)`,
  `(Muq-Ta')`, `(Hermetic)`, `(PC)`, and the `(positive)`/`(negative)` Cyclic Magic
  disambiguators.
- **`name_unfilled` — the opt-out for a doubled hint (guided-creation review #13,
  Slice 7).** With no instance chosen, `displayName` fills a `{token}` with the
  localized hint `param-hint` = `({ $label })`, which is right for the overwhelming
  majority of templates: `Puissant (Ability)`, `Affinity with (Ability)`,
  `Great (Characteristic)`, `Ways Of The (Land)`. It **doubles** for the one shape that
  carries both a `{token}` and a parenthetical literal — `"{language} (Dead Language)"`
  plus `"(Language)"` read *"(Language) (Dead Language) 1 is not met"*. Grepping every
  i18n name template for that shape returns exactly **two** entries, both Abilities:
  `ability.dead_language` and `ability.living_language`. Each now declares an optional
  `name_unfilled` in `rules/i18n/<lang>/abilities.json` (`Dead Language` /
  `Tote Sprache`, `Living Language` / `Lebende Sprache` — lifted from the existing
  German templates, not re-translated), which `displayName` uses when **no** token in
  the template is filled; a filled instance still renders the full template
  (`Latin (Dead Language)`), which is why the parenthetical stays in `name` at all.
  Blanket suppression of the hint was **considered and rejected**: it would degrade the
  ~36 templates whose token is the head or sits mid-phrase to `Puissant`,
  `Affinity with`, `Ways Of The` — worse in German, where an inflected adjective would
  dangle with no noun to agree with. The guard test
  `an unfilled template without name_unfilled still renders the param hint`
  (`ui/src/lib/derive.test.ts`) exists solely to stop that "simplification" returning.
  The field is `Option<String>` on `I18nEntry`, `#[serde(default,
  skip_serializing_if = "Option::is_none")]`, so every other entry is untouched.
  **Both renderers honour it**: `displayName` in the UI and
  `export/resolve.rs::unfilled_name` (consulted by `parameterized_name` /
  `parameterized_value`) in the Markdown exporter, so a sheet and the screen word the
  Ability identically instead of the sheet alone printing the doubled form.

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
  "Name (Klein/Groß)". This mutual exclusion is enforced at load by
  `ruleset/integrity.rs` `validate_magnitude_variant_exclusivity` (see the Magical Focus
  section), which also covers the sole prefix-form pair
  `virtue.major_magical_focus` / `virtue.minor_magical_focus` (Ars Magica - Definitive Edition (Core Rules).md:4405).
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
- Implementation: `crates/arm-rules/src/validation/selections.rs` — `validate_gift_policy` (:459).
  The Gift policy is independent of the `is_magus` flag (an unGifted Redcap is a
  companion; a Gifted hedge wizard is not a magus).

#### Prerequisite evaluation (meta-mechanic)
- The tri-state `Prereq` evaluator (`crates/arm-rules/src/validation/prereq.rs` —
  `evaluate_prereq`, :137) is engine infrastructure, not a single rulebook passage. It
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
1), applied in `validation/balance.rs::validate_balance` (the `unbalanced_virtues` check
compares virtue points against `flaw_points * virtue_points_per_flaw_point`).
Source: Ars Magica - Definitive Edition (Core Rules).md:2638.

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
  `effective_min_score`); enforced in `validation/scores.rs` —
  `validate_characteristics` (:31) (off-table out-of-range error, above-cap /
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
  Native language is not a separate id: it is one *instance* of
  `ability.living_language`, told apart by the row's `parameter` (the language
  name), which is what `LifeStagePlan::native_language` names. Parameterized
  abilities carry a `parameter` key (`area` for (Area) Lore, `language` for the
  (Living/Dead Language) abilities).
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
  resolve against it) in `ruleset/integrity.rs`; `validate_abilities` in `validation/scores.rs` (:252).

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
  `ruleset/integrity.rs`; `validate_arts` in `validation/scores.rs` (:366).

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

Computed in `crates/arm-rules/src/effective/ability.rs` (`ability_bonus`,
`effective_ability_score`, `ability_bonuses`) and
`crates/arm-rules/src/effective/characteristic.rs` (`characteristic_cap`,
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
- Implementation: `effective/ability.rs::ability_bonus(.., parameter)` matches
  `(ability, parameter)` — a plain ability by id, a parameterized one only when
  the selection names the same instance; a selection missing the instance key
  matches nothing. `ability_bonuses` returns a per-instance `Vec<AbilityBonus>`
  (serialized directly to the frontend by `arm-app::ruleset_io`). `validation/scores.rs`
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
  `effective/art.rs::art_bonus`, `effective_art_score`, `art_bonuses` (serialized to
  the frontend by `arm-app::ruleset_io`). `validation/scores.rs` folds the bonus into the
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
- Implementation: `effective/characteristic.rs::characteristic_cap` =
  `min(base_max + Σ positive amounts, effective_max)`;
  `validation/scores.rs::validate_characteristics` (:31) flags a bought score above the cap
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
- Implementation: `effective/characteristic.rs::characteristic_floor` =
  `max(base_min + Σ negative amounts, effective_min)`;
  `validate_characteristics` flags a bought score below the floor
  (`characteristic_below_floor`); `validate_characteristic_limit_preconditions`
  flags a target base above the base floor (`characteristic_min_base_too_high`).

#### Selection multiplicity — `max_per_target`
The per-`(item, params)` selection cap. `validate_duplicate_selections`
(`validation/selections.rs`, :107) errors `duplicate_selection` when a target's count exceeds the
item's `max_per_target` (default 1; Great Characteristic 2). This generalizes the
former hardcoded "at most once" rule and enforces both "Puissant once per
Ability" (`:4816`) and "Great twice per Characteristic" (`:3989`). Effect
integrity (`ruleset/integrity.rs::validate_effect_refs`) rejects at load any effect whose
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
- Implementation: `effective/xp.rs::charged_cost` (the `ceil(T·den/num)` arithmetic,
  verified against the worked example below) + `ability_affinity`, folded into
  `effective/xp.rs::xp_allocation` and so into `validation/magus.rs::validate_xp_pool` (:559).

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
- Implementation: `effective/xp.rs::charged_cost` + `art_affinity`, via
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
- Implementation: `effective/xp.rs::xp_allocation` builds a bipartite **max-flow**
  feasibility graph (general pool + one node per restricted pool → eligible spends
  → sink). A greedy assignment is incorrect under overlapping eligibility
  (Educated's academic ids overlap Privileged's `academic` category), so flow is
  used. `validation/magus.rs::validate_xp_pool` (:559) reports `not_enough_xp` (with
  `shortfall`) and `restricted_xp_unspent` (warning, naming the granting item
  through `origin_kind`/`origin` — see the life-stage section for why the pool has to
  be named).
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
- **Unspent GENERAL experience warns too (Slice 11 / #30), and says nothing more.**
  Overspending the general pool has always been `not_enough_xp`; leaving it unspent
  produced **silence**, while a single unspent Characteristic point produced
  `characteristic_points_unspent`. `validate_xp_pool` now emits
  `general_xp_unspent` (warning, phase `abilities` — the step the XP bar lives on,
  the same as `not_enough_xp`; args `pool`/`used`/`unspent`) as the `else if` branch
  of the overspend test, so an infeasible allocation is never told both that it
  overspent and that it has points left over.
  - It counts `general_pool - general_used` **alone**. Each restricted pool already
    has its own `restricted_xp_unspent`, so adding them in would report one
    life-stage character's points twice — 300 general plus a 45 childhood spread
    reported as 345 unspent *and* 45 unspent.
  - **Purely factual wording, and the absence of a rule is the reason.** The pool's
    *size* is thoroughly sourced (`:2213` 75 + 45, `:2214` 15/yr, `:2215` 240 for
    apprenticeship, `:2216` 30/yr after the Gauntlet), but **no passage in
    `rules/source/en/` says unspent general experience is lost or wasted.** So the
    message is "N of M experience points are still unspent" and stops. The
    restricted sibling's "will be wasted" is backed by that pool being earmarked to
    a named list; the general pool is earmarked to nothing, so the same claim would
    be invented. Do not "improve" the copy into one — `ui/src/lib/i18n.test.ts`
    asserts the bare count in both locales.
  - A **warning**, so `canFinish` (errors only) is untouched: a character may be
    finished with experience in hand, which is exactly the state a player who has
    not decided yet is in.
- Permission unlock (Academic/Martial purchasable only with such a Virtue) is
  **deferred** — not enforced in this phase.

#### Improved Characteristics — +3 Characteristic-buy points
> "You have an additional three points to spend on buying Characteristics … You
> may take this Virtue multiple times."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:4103-4105`.
- Data: `rules/core/virtues_flaws.json` `virtue.improved_characteristics` —
  `effects: [{ characteristic_points, amount: 3 }]`. The `3` lives here.
- Implementation: `effective/characteristic.rs::characteristic_points_granted` sums the grants;
  `validation/scores.rs::validate_characteristics` (:31) budget = `start_points + granted`. The
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
  `effective/characteristic.rs::size` (base 0 + Σ, no cost/cap; surfaced as `EffectiveScores.size`).
  `Effect::CharacteristicScoreDelta { characteristic, amount }` →
  `characteristic_score_bonus` / `effective_characteristic_score` — a **free**
  effective-score bonus (separate from the bought score, no buy-budget cost) that
  may push the effective score past ±5 to ±6. Surfaced as
  `EffectiveScores.characteristic_bonuses` and shown in the sheet next to the bought
  score; Size shows via Fluent `characteristic-size`. The stored `characteristic`
  id is validated to resolve in `ruleset/integrity.rs::validate_effect_refs`.

#### Warped by Magic — Warping Score + Points (`warping_grant`)
> "He has five Warping Points and a Warping Score of 1, including a Minor Flaw
> (which is not balanced by a Virtue) … His encounters allow you to spend
> experience points on Magic Lore during character creation."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:7019-7021`.
- Data: `flaw.warped_by_magic` — `effects: [{ warping_grant, score: 1, points: 5 }]`.
- Implementation: `Effect::WarpingGrant { score, points }` → `effective/warping.rs::warping`
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
  `effective/xp.rs::ability_affinity`, so it feeds the XP-cost reduction
  (`charged_cost`) and the +2 age-cap exemption exactly like a normal Affinity. The
  ability ids are validated to resolve in `ruleset/integrity.rs::validate_effect_refs`.

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
  `ruleset/integrity.rs::validate_effect_refs` (only when the Arts catalogue is loaded, like
  `validate_spell_refs`).
- Implementation: `effective/art.rs::elemental_form_bonus` (folded into
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
> Spell Mastery is an Ability: "Spell mastery Abilities are their own category"
> (`:9516`), one of the six Ability types (`:7143`, `:7163-7165`), bought from the
> Ability advancement table "(Ability + 1) x 5" (`:15952`; To-Buy table 1=5, 2=15,
> 3=30, 4=50, 5=75 at `:15956-15979`). Mastered Spells: "You have fifty experience
> points to spend on mastering spells that you know … You may take this Virtue
> multiple times." Flawless Magic: "All your spells start with a score of 1 in the
> corresponding Spell Mastery Ability … all your Advancement Totals for Spell
> Mastery Abilities are doubled."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:9516`, `:7143`,
  `:7163-7165` (mastery is an Ability type); `:15952`, `:15956-15979` (Ability
  To-Buy table); `:4471-4474` (Mastered Spells); `:3887-3889` (Flawless Magic).
- Data: `virtue.mastered_spells` — `spell_mastery_xp: 50`; `virtue.flawless_magic`
  — `grants_spell_mastery: { score: 1, advancement_num: 2, advancement_den: 1 }`
  (the doubling stored as an Affinity "counts as 2/1", halving the charge).
  `SpellSelection.mastery` is the bought Spell Mastery score (serialized shape
  unchanged).
- Implementation: `effective/xp.rs::xp_allocation` now prices each `SpellSelection`'s
  bought `mastery` from `ruleset.advancement.xp_for_score` and adds it to the
  max-flow demand as a `SpendKind::Mastery`. The **free floor** (Flawless Magic
  auto-mastery 1) subtracts the floor's table cost before the charge — a bought
  score ≤ floor costs 0 — exactly as a granted Supernatural-Ability floor does; the
  **doubling** is applied through `charged_cost` with the Affinity from
  `spell_mastery_advancement_affinity` (Flawless → `(2,1)`), so `mastery 3` under
  Flawless costs `ceil((table(3) − table(1)) / 2) = ceil(25/2) = 13`. Eligibility:
  a `PoolEligibility::Mastery` pool (the summed `SpellMasteryXp`) funds only Mastery
  spends, the ability-restricted pools (`PoolEligibility::Ability`) never do, and
  neither funds the other's spends (`pool_covers`). Overspend surfaces through the
  existing `validate_xp_pool` → `CODE_NOT_ENOUGH_XP` (its `pool` arg is now
  `allocation.max_flow`, the total all pools can fund, so `spent − pool == shortfall`
  stays accurate). `Effect::GrantsSpellMastery { score, .. }` → `spell_mastery_floor`
  (max grant); `effective_spell_mastery(sel)` = `max(bought, floor)`. The UI mirrors
  the same floor + doubling charge in `derive.ts::spellMasteryXpSpent`, driven by
  `EffectiveScores.spell_mastery_{xp,floor}` and `spell_mastery_advancement_doubled`;
  the SpellPicker mastery spinner shows for every magus (buyable from the general
  pool). "May take multiple times" follows the `max_per_target: 1` convention.

#### Mastered Spell Special Abilities — choosable per-spell mastery options (`spell_mastery_abilities`)
> "For every level in the Mastery Ability, the maga may also choose one special
> ability, which applies only to that mastered spell. Thus, a maga with a Mastery
> Score of two for a spell has two special abilities for that spell." (`:9524-9526`)
> The catalogue of fourteen options is at `:9528-9592`: Adaptive, Ceremonial, Fast,
> Imperturbable, Magic Resistance, Multiple, Obfuscated, Penetration, Precise,
> Quick, Quiet, Rebuttal, Still Casting, and Unravelling. Most are once-per-spell;
> Precise (`:9570-9572`), Quick (`:9574-9576`), and Quiet Casting (`:9578-9580`)
> "may take this ability multiple times for the same spell".

- Source: `Ars Magica - Definitive Edition (Core Rules).md:9524-9526` (one special
  ability per Mastery level); `:9528-9592` (the catalogue); `:9572`, `:9576`,
  `:9580` (Precise/Quick/Quiet are repeatable). German source
  `Ars Magica Definitive Edition Basisregeln.md:9524-9592` mirrors it line-for-line.
- Data: catalogue in `rules/core/spell_mastery_abilities.json` — each entry an `id`
  (`spell_mastery_ability.<slug>`) + a `repeatable` bool (true only for Precise/
  Quick/Quiet Casting) + `source`. Names/descriptions in
  `rules/i18n/{en,de}/spell_mastery_abilities.json` (German names from the German
  source headings; terms matching the translation tables — e.g. Adaptives Zaubern,
  Zeremonielles Zaubern, Schnellzaubern, Magieresistenz, Penetration). Catalogue
  size is data (no counts in code).
- Model: `SpellSelection.mastery_abilities: Vec<Id>` (serde-default, may hold
  duplicates for repeatable picks; additive so `SCHEMA_VERSION` stays 13). Sorted
  stably by `Entity::normalize`.
- Engine plumbing: `SpellMasteryAbility` (`spell_mastery.rs`) threaded through
  `RulesetSources`/`Ruleset::from_sources` as a `BTreeMap<Id, SpellMasteryAbility>`
  (`spell_mastery_abilities`), loaded by `ruleset_io::load_ruleset_from_dir` and
  surfaced on the ruleset; the i18n file joins both languages via `read_i18n_sources`.
- Implementation: `validation/magus.rs::validate_spell_mastery_abilities` enforces
  (a) count ≤ `effective_spell_mastery(sel)` = `max(bought, floor)`, one per level
  (`CODE_TOO_MANY_MASTERY_ABILITIES`); (b) a non-repeatable ability chosen more than
  once for the same spell (`CODE_DUPLICATE_MASTERY_ABILITY`); (c) referential
  integrity — an unknown chosen id fails (`CODE_UNKNOWN_MASTERY_ABILITY`). Load-time
  integrity checks each catalogue entry's `source` range. UI: per-spell add/remove
  picker in `SpellPicker.svelte` (store `addMasteryAbilityAt`/`removeMasteryAbilityAt`),
  greying non-repeatable already-chosen options and hiding the add control once the
  count reaches the effective mastery.

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
  `ruleset/integrity.rs::validate_effect_refs`. The granted rows now surface through
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
  `effective/gift_confidence.rs::item_level_budget` (derived, base 0 + Σ). Surfaced as
  `EffectiveScores.item_level_budget` and shown on the sheet (Fluent
  `item-levels-label/readout`; DE "Zauberartefakte"). The device-crafting subsystem
  is out of scope; the granted budget is tracked (full scope of the *Virtue*).
- **M5/5e — starting enchanted devices stored & charged.** `Entity.devices:
  Vec<EnchantedDevice { name, level: u16 }>` records the player's chosen starting
  devices; the total `level` is charged against `item_level_budget()`.
  `effective/gift_confidence.rs::item_level_used` sums the device levels (surfaced as
  `EffectiveScores.item_level_used`), and `validation/might.rs::validate_devices` (:85) emits
  `over_item_level` (Fluent `issue-over_item_level`) when `used > budget`. A device
  therefore requires a granting Virtue, exactly as a starting Reputation does.

#### M5/5e — magic-possession Entity storage (aura, familiar, talisman, longevity)
Direct-entry storage for a magus's starting magic possessions. `SCHEMA_VERSION`
bumped 8 → 9 (single bump; all new fields `serde(default, skip_serializing_if)`,
so v≤8 saves load unchanged — **no migration function**, the same additive
precedent as `SpellSelection.mastery`'s 7 → 8 bump). Nothing is derived here
(that is 5i); these fields only *store* the choices.

- **Aura** — `Entity.aura: i32` (signed; a Divine aura can be a penalty).
  Persisted so 5i's Longevity hint / lab totals are reproducible. Casting Score
  adds the aura (Ars Magica - Definitive Edition (Core Rules).md:9089); every Lab Total takes it as a plain addend — "your
  basic Lab Total is: Technique + Form + Intelligence + Magic Theory + Aura
  Modifier" (Ars Magica - Definitive Edition (Core Rules).md:10276-10278) — with no gate on a nonzero aura (see M5.5a).
- **Familiar cords** — `Entity.familiar: Option<Familiar { name, cord_gold,
  cord_silver, cord_bronze: u8 }>`. Gold = −botch dice; Silver = +Personality /
  mental resistance; **Bronze = +Soak & aging-resistance** (feeds 5i Soak /
  longevity). Source: `Ars Magica - Definitive Edition (Core Rules).md:10840-10844`.
- **Talisman** — `Entity.talisman: Option<Talisman>` (see M5.5b below). Originally
  a flat `talisman_attunements: Vec<TalismanAttunement>`; schema 14 moved it under
  the item.
- **Longevity Ritual** — `Entity.longevity_ritual: Option<LongevityRitual { source:
  LongevitySource (SelfMade|External), bonus: Option<i8>, focus: String }>`. The
  bonus is **player-entered for both sources** and stored; `None` means "not entered
  yet", never a claimed 0 (see M5.5a for why it is not derived). `focus` is the
  ritual's culminating focus, free text. Source:
  `Ars Magica - Definitive Edition (Core Rules).md:10662` (formula), `:10656`
  (focus + permanent sterility).
- `EnchantedDevice`/`TalismanAttunement`/`TalismanEffect` derive `Ord` so
  `Entity::normalize()` sorts `devices` (by name) and — via `Talisman::normalize()`
  — the talisman's `attunements` (by description) and `effects` (by name) for
  zero-noise diffs. `LongevitySource` gets snake_case serde + `Display` (guarded by
  `display_matches_serde_scalar_for_every_enum`); the UI renders every enum through
  a Fluent key (`longevity-source-{self_made,external}`), never the raw slug.

#### M5.5b — Talisman as an item (storage + schema 14 migration)
The talisman is the magus's personal enchanted item, not a bare attunement list.
`Entity.talisman: Option<Talisman { description, attunements, effects }>` replaces
`talisman_attunements`, so `SCHEMA_VERSION` goes **13 → 14** — the first *field
move* (every earlier bump since 10 was additive). A magus may have only one
talisman, which is why it is an `Option`, not a `Vec`.

- Verbatim: "A talisman is a very personal item that contains magics and materials
  that tie it intimately to you and that can be used as a channel for your magical
  power." … "A magus can only have one talisman at once".
  Source: `Ars Magica - Definitive Edition (Core Rules).md:10603-10625`
  (`:10605` identity, `:10607` one-at-a-time).
- **Identity** — `Talisman.description: String`, free text: shape and material are
  open-ended (the Shape and Material Bonuses Table is not a closed catalogue in
  this model), so there is no id to reference.
- **Attunements** — `Talisman.attunements: Vec<TalismanAttunement { description,
  bonus: i8 }>`, unchanged in shape, only re-homed. "you may also open your
  talisman to one kind of magic attunement, based on the shape and material of the
  talisman, every time you prepare it for enchantment or instill an effect"
  (`:10623`); "only the highest bonus applies. They apply to Casting Scores for
  Ritual, Formulaic and Spontaneous magic, but they do not apply to Magic
  Resistance or any laboratory activities" (`:10625`). This corrects the M5/5e
  citation, which named no lines at all.
  **The bonus is stored but still unread** — nothing in `derived.rs` adds it to a
  Casting Total (`casting_totals` never looks at `talisman.attunements`;
  `talisman_capacity` is the only talisman consumer), in the same sense as
  `lab_enchanting` being *collected but unread* in the V/F effect table below. This
  is a decision, not an omission: the attunement's subject is free text
  (`TalismanAttunement.description`), so the engine cannot tell which cell of the
  casting grid an attunement enhances, "only the highest bonus applies" needs that
  same judgement to pick a winner, and the bonus applies only "when the magus is
  touching the talisman" (`:10625`) — a moment of play the model does not represent.
  So it is a situational modifier the player applies at the table, exactly like
  Soak's Form bonus (entered 0, see the *Soak* row). Computing it would require
  either a closed attunement catalogue keyed to spell Techniques/Forms or a
  per-total "touching my talisman" toggle; both are out of M5.5b's scope.
- **Instilled effects** — `Talisman.effects: Vec<TalismanEffect { name, level: u16 }>`.
  "When a magus instills effects into a talisman, he gets a +5 bonus to his Lab
  Total" (`:10621`). Deliberately **not** a reused `EnchantedDevice`: the two carry
  different budget contracts (see the non-goal under M5.5b in the derived-totals
  section) and different provenance — the same reason `SupernaturalPower` exists
  beside `EnchantedDevice`.
- **Migration** — `load_entity_migrating` gains a second legacy-key fold
  (`fold_legacy_talisman`) beside the `aging_reductions` one, dispatching on the
  legacy key's *presence*, never on the recorded version. The fold **invents
  nothing**: attunements are copied verbatim, and the identity/effects the old shape
  never stored stay empty. So unlike the aging fold — which *infers* a point total —
  it needs no `LoadedEntity` notice flag. Four ambiguous shapes are pinned
  by named tests: `"talisman_attunements": []` leaves `talisman: None` (no phantom
  item) but still bumps the version, since the key's presence proves the old shape
  (`legacy_empty_talisman_attunements_migrate_to_no_talisman`); a hand-edited save
  carrying **both** keys, where the new `talisman` already **carries attunements**,
  keeps the new one and drops the legacy list unmerged
  (`hand_edited_save_with_both_talisman_shapes_keeps_the_new_one` — whose
  legacy row is deliberately one that *cannot* deserialize, so the test also pins the
  order of operations: a new shape with attunements wins **before** the legacy value is
  parsed, hence its shape genuinely cannot matter); but a new shape with **no**
  attunements does **not** win, and the legacy list is folded into it — both when it is
  literally `"talisman": {}` (`legacy_attunements_fold_into_an_empty_new_talisman`) and
  when it carries data only in fields the fold cannot touch, e.g. a description
  (`legacy_attunements_fold_into_a_talisman_that_has_only_a_description`, which also
  asserts the description *survives* the fold). The gate is therefore
  `entity.talisman.as_ref().is_some_and(|t| !t.attunements.is_empty())` — neither
  `is_some()` nor `*t != Talisman::default()`. `attunements` is the sole field the
  legacy `talisman_attunements` key migrates into, so it is the only field whose
  contents can make merging *duplicate* attunements a player moved across by hand; a
  filled `description` or `effects` says nothing about attunements, and `{}` says
  nothing at all (every `Talisman` field is `skip_serializing_if`, so the *app itself*
  writes `{}` for an untouched talisman). Gating on anything wider would drop the
  legacy list unparsed while the caller still stamped `SCHEMA_VERSION` — the identical
  permanent loss the error path below exists to prevent, reached through the
  both-keys path instead. Because the list is folded **into** the existing talisman
  (only its empty `attunements` is filled), the item's own `description`/`effects`
  survive. `{}` beside an *empty* legacy list stays `{}`, since the
  fold invents nothing (`an_empty_talisman_beside_an_empty_legacy_list_gains_nothing`).
  App-written saves can never carry an empty list
  (`skip_serializing_if = "Vec::is_empty"`).
- **A legacy value that cannot deserialize fails the load** — both folds propagate
  the `serde_json::Error` (`from_value(legacy)?`), so a `bonus` outside `i8`, a bonus
  written as a JSON string, `null` in place of the list, or an out-of-`u8` /
  unknown-key `aging_reductions` map aborts the load exactly as a malformed current
  field does. Swallowing the error (the original `unwrap_or_default()`) was silent
  data loss: the fold yielded nothing while the caller still stamped
  `SCHEMA_VERSION`, so the load reported success and the next save rewrote the file
  without the legacy key — destroying the attunements. Failing the load leaves the
  file on disk untouched. Pinned by
  `legacy_talisman_attunements_that_cannot_deserialize_fail_the_load` and
  `legacy_aging_reductions_that_cannot_deserialize_fail_the_load`, which assert the
  **error text** (the out-of-range value plus `i8`/`u8`, the unknown Characteristic
  key) and pair every malformed fixture with a byte-identical **positive control**
  whose one offending value is corrected — so the failure is provably the legacy row's
  and not the surrounding document's. A bare `is_err()` would have stayed green if the
  fixtures had been invalidated some other way while the fold silently reverted to
  `unwrap_or_default()`.
- **Derived capacity** — see the *Talisman capacity* row in the derived-formulas
  table below, including the budget non-goal.
- **UI** — `TalismanPanel.svelte` (extracted from `MagicPossessions.svelte` in
  M5.5's prep commit) renders the identity input, the capacity read-out + its
  derivation note, the attunement list and the instilled-effects list. The capacity
  is read from `store.derived.talisman_capacity` and **never** recomputed in JS,
  the `itemBudget`/`itemUsed` precedent. `DerivedTotalsPanel.svelte` is
  deliberately untouched: the capacity renders where the talisman is edited, beside
  the effects it constrains. New Fluent keys in both locales; German terms come from
  `rules/source/de/translation-tables/` — `labor-fortschritt.md` (Talisman →
  Talisman, Talisman Attunement → Talismanabstimmung, Level → Stufe, Shape and
  Material Bonus → Form- und Materialbonus, hence "Form und Material"; "Instilling
  Effects → Effekte einbetten" gives the participle for the section title
  "Eingebettete Effekte") and `konvent.md` (Pawn → Bauer). *Vim* and *Vis* stay
  untranslated as Latin (`magie-regeln.md`, `grundbegriffe.md`). "Kapazität" is not
  in any table — it is the ordinary German rendering of *capacity*, which the
  glossary does not cover.
- **`Ruleset::art_ids_of(ArtType) -> Vec<Id>`** replaces the open-coded
  Technique/Form catalogue split that had accumulated in three places
  (`derived.rs::arts_of`, deleted, plus `effective/spell.rs::spell_level_caps`'s own
  pair of filters). It returns the ids **sorted**, which makes the canonical
  `(technique, form)` ordering `spell_level_caps` documents a property of the
  helper rather than an accident of how a ruleset's `arts.json` lists its entries
  — the shipped ruleset is canonically serialized, but a hand-built test fixture
  need not be.

#### M5.5c — Familiar as a creature statblock (storage, additive — no schema bump)
M5/5e stored a familiar as a name plus three cord scores, with none of the magical
animal behind it. M5.5c expands `Entity.familiar: Option<Familiar>` into a
**creature statblock**, whose field order follows the rulebook's own *Creature
Format* (`:17787-17827`) so a save reads like the printed creature entry:

> A familiar is a beast that a magus befriends and then magically bonds with,
> instilling the beast with magical powers in the process … Though a familiar is
> very close to the magus who creates it, it always has its own will, and is not
> under the control of the magus. — `:10770`

Source for the whole chapter: `Ars Magica - Definitive Edition (Core Rules).md:10766-10892`.

- **`animal: String`** (free text) — the kind of beast. "The first step in getting a
  familiar is finding an animal with inherent magic" (`:10774`). Deliberately **not**
  named `species`: the rules reserve *Species* for the Imaginem term (the sensory
  image a thing sheds), and the German glossary makes the same reservation
  (`translation-tables/grundbegriffe.md:112`), so "Spezies" is the wrong label too.
- **`might: Option<MightScore>`** — the familiar's own Magic Might + Realm. "the
  beast is likely to have a Magic Might score, which may be assigned based on the
  scores of comparable magical creatures" (`:10774`); the Creature Format prints it
  as the `(Realm) Might:` line (`:17791`). It is the **familiar's** Might: no Virtue
  grant stacks on top, which is why the UI labels it with its own Fluent key rather
  than reusing `might-score-label` ("Base Might Score").
- **`characteristics: BTreeMap<Characteristic, i8>`** — the creature's own
  Characteristics ("A list of the characteristics and values", `:17793`), signed and
  **never bought from the magus's Characteristic points**. A 0 score is **pruned by
  `Familiar::normalize()`**, and `skip_serializing_if` then omits the map once it is
  empty — so an explicit 0 and an absent entry write identical bytes, as canonical
  serialization requires. Pruning (rather than keeping a stored 0) is safe because
  these scores are display-only: no engine read-out or validation reads them, so a
  deliberately-entered 0 carries no mechanical meaning an absent entry lacks. Pinned
  by `entity_normalize_prunes_zero_familiar_characteristics`.
- **`size: i8`** — signed and commonly **negative**; a raven is Size -4
  (`:17829-17856`, the Size examples table). It lowers the bonding level: "If the
  familiar has negative Size, this reduces the level for the enchantment" (`:10824`).
  Every displayed sign is the ASCII hyphen-minus, per the project convention.
- **`personality_traits: Vec<PersonalityTrait>`** — `:17807`. Reuses the existing
  value struct; kept sorted by `Familiar::normalize()`.
- **`powers: Vec<SupernaturalPower>`** — the powers invested in the bond
  (`:10862-10884`), charged against **no** budget (`:10866`, see the derived row).
  Kept sorted by `Familiar::normalize()`.
- The three cords are unchanged, only re-ordered to sit where the statblock puts
  them (after Personality Traits, before Powers). Source: `:10840-10844`. A score
  above the +5 maximum (`:10836`) is **clamped by `Familiar::normalize()`**, the same
  self-healing repair as pruning a zero Characteristic above and for the same
  canonical-serialization reason — see the *Familiar cord cost* derived row for the
  full argument and for where `MAX_CORD_SCORE` lives.

**No `SCHEMA_VERSION` bump.** Every field beyond `name` is additive
`serde(default, skip_serializing_if)`, so a pre-5.5c familiar (name + cords) loads
unchanged *and* writes byte-identical JSON — the `mastery_abilities` /
`warping_choices` / `LongevityRitual::focus` precedent. Pinned by
`cords_only_familiar_omits_every_statblock_key` and
`save_with_cords_only_familiar_loads_statblock_as_defaults`.

**Two invariants that fall out of nesting**, each locked by a named regression test
in `validation/mod.rs`:

- `familiar_characteristics_never_enter_the_magus_point_buy` — the familiar's
  Characteristics live in `Entity.familiar`, never in `Entity.characteristics`, so
  `validate_characteristics` cannot see them; a familiar with eight maxed
  Characteristics changes no issue.
- `familiar_might_never_triggers_the_realm_mismatch_warning` — `validate_might`
  reads only `Entity.might`, and `validate_powers` only `Entity.powers`, so the
  familiar's (possibly other-Realm) Might raises no `might_realm_mismatch` and its
  invested powers raise no `over_power_levels`. The same test asserts
  `effective_might` stays `None`, so a magus with a Might-bearing familiar is never
  misdetected as a Might-being and offered the non-magus Might tab.

**No `Cunning` variant is added to `Characteristic`** (a deliberate, recorded
decision — do not re-litigate). The Creature Format notes that "Creatures with
animal intelligence have a Cunning (Cun) score rather than an Intelligence score"
(`:17793`), but a **bound** familiar is not such a creature: "If it did not
previously have human intelligence, it gains it, with a score of –3" (`:10854`) —
an ordinary Intelligence entry. The app only ever stores *bound* familiars, so
`Characteristic` stays the fixed eight the rules name (`:1023-1025`). Displaying an
unbound creature's score as "Cun" would be a **display-only** affordance, and it is
deferred until the app models unbound creatures at all.

**Deferred statblock lines** (out of M5.5c scope by decision): the familiar's
Abilities (`:17819`), Qualities, Virtues/Flaws (`:17805`) and the
Combat / Soak / Fatigue / Wound statlines (`:17811-17817`). These are the creature
lines the app does not yet enter for *any* creature, so adding them for the familiar
alone would be a lone special case.

**The bond's own grants are surfaced, never auto-applied.**

> The familiar binding gives both the magus and the familiar the Minor Virtue True
> Friend, relating to the other half of the partnership. Thus, they also gain
> Personality Traits of Loyal (partner) +3. — `:10852`

> If it did not previously have human intelligence, it gains it, with a score of
> –3. — `:10854`

`FamiliarPanel.svelte` renders these as a note (`familiar-bond-note`) and the player
enters them by hand. Auto-applying them is not possible today and would be wrong to
fake: `virtue.true_friend` is not in the seeded catalogue, and `grant.rs` is keyed to
House / mythic-companion **profiles**, not to the presence of a familiar bond — so
there is no hook a bond could grant through without inventing one.

**UI.** `FamiliarPanel.svelte` (extracted from `MagicPossessions.svelte` in M5.5's
prep commit) grows to the whole statblock: name + animal + signed Size, a Might block
adapted from `SupernaturalBeing.svelte` (reusing `might-realm-label` and `realm-*`
verbatim, but a **new** `familiar-might-score-label` — the existing
`might-score-label = Base Might Score` implies Virtue grants stack on top, which is
false for a familiar), a Characteristics grid over the engine's `CHARACTERISTICS`
order with plain signed inputs plus the not-from-the-magus's-points note, Personality
Trait rows adapted from `CharacterDetails.svelte`, the cord row unchanged, and an
invested-powers list adapted from `SupernaturalBeing.svelte` **with no budget bar**
(`:10866`). `DerivedTotalsPanel.svelte` gains a familiar-bond section modelled on the
Masterpiece block. Every read-out comes from `store.derived.familiar` and is **never**
recomputed in JS, the `itemBudget`/`itemUsed` precedent.

New Fluent keys in both locales. German terms come from
`rules/source/de/translation-tables/`: `tiere-kreaturen.md` (Animal → **Tier**, Size →
**Größe**, Power → **Kraft**), `grundbegriffe.md` (Familiar → **Vertrauter**, Might
Score → **Machtwert**, Intelligence → **Intelligenz**), `sphären-mächte.md`
(Magic Might → **Magische Macht**), `labor-fortschritt.md` (Familiar Bond →
**Vertrautenbindung**, Laboratory Total → **Laborsumme**, Level → **Stufe**),
`tugenden-fehler.md` (True Friend → **Wahrer Freund**, Magical Focus → **Magischer
Fokus**), `magische-qualitaeten.md` (Minor Virtue → **Kleine Tugend**),
`persoenlichkeitseigenschaften.md` (Personality Trait →
**Persönlichkeitseigenschaft**, Loyal → **Loyal**). Notably **not** "Spezies":
`grundbegriffe.md:112` reserves *Species* for the Imaginem term, which is the same
reason the Rust field is `animal`. Terms with no table headword, each derived from the
German rulebook passage rather than invented: **Kordelpunkte** (from `:10836`
"verteile die Punkte … auf die drei Kordeln … Kordelwerte"), **Bindungsstufe** (from
the `:10828` header "STUFE DER VERTRAUTENSBINDUNG"; the tables' `Vertrautenbindung`
spelling wins over the header's linking -s-), **Investierte Kräfte/Kraftstufen**
(from `:10864-10866` "Kräfte in die Vertrautenbindung investieren" — deliberately
*not* "Eingebettete Effekte", which `labor-fortschritt.md:23` assigns to the
talisman's *Instilling Effects*), and **Laborsumme für die Bindung** (phrase shape
from `:10818`). Every displayed negative sign is the ASCII hyphen-minus, including
the note's "Intelligenz -3", although both the tables and the German rulebook print
U+2212 there.

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
    model's Decrepitude under-count. A legacy map that cannot deserialize fails the
    load loudly rather than folding in nothing (see the M5.5b migration note).
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

  Source: `Ars Magica - Definitive Edition (Core Rules).md:1155`. **No longer a
  pure annotation.** M5/D2 shipped it as one because nothing computed aging rolls;
  since M6/6b6 `aging::resolve_year` seeds and advances it per `:16577` (see
  **Aging (M6/6b6)** below). It stays directly editable — a hand-entered figure is
  never overwritten — and it is never an *input* to a roll: the modifier "depends
  on the character's **actual, not apparent**, age" (`:16577`).
- **Warping effect** — `Entity.warping_effect: String` (skip-if-empty), a
  free-text flavor note for how the character's Warping manifests (Issue D).
  > "This Minor Flaw should reflect the predominant source of the Warping Points."

  Source: `Ars Magica - Definitive Edition (Core Rules).md:16547-16561` (### Effects
  of Warping). This is the flavor annotation only; the **mechanical** owed V/F it
  used to stand in for are now auto-granted (Issue E, below), and the textarea
  stays for additional narrative colour.

#### Effects of Warping — auto-granted owed Virtues/Flaws (Issue E, `warping_grant` curve)
> "Hermetic magi are made more prone to Wizard's Twilight by their Warping Score.
> This replaces the normal effects." (16551)
> "Mundane characters gain a Minor Flaw when they reach a Warping Score of one."
> (16553) "When the Warping Score reaches 3, the character gains a second Minor
> Flaw." (16557) "At a Warping Score of 5, the character gains a supernatural
> Minor Virtue…" (16559) "At a Warping Score of 6, and every point thereafter,
> the character gains a Major Flaw…" (16561)

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16551-16561`.
- Implementation: `effective/warping.rs::warping_owed(entity, ruleset) -> WarpingOwed`
  (`{ minor_flaws, minor_supernatural_virtues, major_flaws }`), driven by the pure
  threshold `WarpingOwed::from_score`: Minor Flaw at Score 1, a second at 3
  (`minor_flaws` cap 2); a supernatural Minor Virtue at 5; `major_flaws =
  score.saturating_sub(5)` (Score 6 → 1, 7 → 2, …). Magi (`profile.is_magus`) owe
  **zero** — Warping gives them Wizard's Twilight instead (16551), which this
  core-only slice does NOT model.
- Off-budget storage: the player's fills live in `Entity.warping_choices`
  (`BTreeMap<String, Selection>`, keyed by each owed slot's `choice_key`), resolved
  through the shared `grant.rs` `Grant::Open`/`GrantConstraint` machinery
  (`warping_owed_grants` builds one Open grant per owed slot; a Minor Flaw, a
  supernatural Minor Virtue, a Major Flaw). Like `house_choices`/`mythic_choices`
  the fills fold through `effective.rs::entity_grants` as real (budget- and
  cap-exempt) selections, so an owed warping Flaw never counts against the creation
  V/F budget. Additive `serde(default)`; `SCHEMA_VERSION` stays 13.
- **Recursion guard (the load-bearing design point).** `warping_owed` depends on
  the Warping Score, which folds `Effect::WarpingGrant` points; the natural fill
  `flaw.warped_by_magic` *is* a `WarpingGrant` +5 item, so folding fills back into
  the owed score would self-amplify (owe → pick warped_by_magic → +5 → owe more →
  …). Stratified two ways: (i) the owed count derives from
  `warping_score_for_owed`, computed over `entity.selections` + `entity_grants_base`
  (House/Mythic/`grants_selection` grants) **excluding** the owed warping fills; and
  (ii) any item carrying `Effect::WarpingGrant` is ineligible as a fill — dropped in
  `warping_granted_selections` and rejected in validation (`warping_fill_ineligible`).
  A regular, budget-bearing `warped_by_magic` selection still counts toward the
  score as before; only the owed-*fill* picks are excluded.
- Validation (`validation/warping.rs`): advisory `warping_owed_{minor_flaws,
  supernatural_virtues,major_flaws}` per unfilled kind; errors
  `warping_fill_constraint` (wrong kind/magnitude/category or unresolvable id),
  `warping_fill_ineligible` (WarpingGrant item), `warping_fill_excess` (a fill keyed
  to a slot not owed). Surfaced to the UI via `EffectiveScores.warping_owed` +
  `warping_owed_grants`; rendered in `CharacterDetails.svelte` (hidden for magi,
  whose owed-grant list is empty) with pickers filtered to eligible items, grouped
  per owed kind so each slot is labelled with what it expects. Labels via Fluent
  `warping-owed-*` / `warping-slot-*` / `issue-warping_*` (DE "Verzerrung", per the
  glossary).
- A fill of a **parameterized** item ("Enchanting (Ability)") must also name its
  parameter: each pick runs through
  `validation/selections.rs::validate_selection_parameters` — the same
  `missing_param` / `unexpected_param` / `unknown_param_value` checks bought
  selections get, shared with the House / Mythic-type `Grant::Open` picks. No new
  issue codes; the specific *item* a slot is filled with is storyguide judgement
  (16553-16561 names no list), so nothing beyond the `GrantConstraint` is filtered.
- **Interpretation notes.** (i) 16553 says "Mundane characters"; this core-only
  slice grants the owed V/F to **all non-magi**. Might-holders (`entity.might`) are
  absolutely immune to warping per 16483 — a flagged interpretation left to the
  troupe, **not** specially handled here (the guard keys only on `is_magus`).
  (ii) The 16559 clause that the supernatural Minor Virtue "stops any further gain
  of points from living in a strong aura of the same type" is a post-creation /
  in-play effect and is **NOT** modeled.
- **Decrepitude effect** — `Entity.decrepitude_effect: String` (skip-if-empty):
  free-text overall aging/decrepitude narrative. Pure annotation — Decrepitude
  itself is DERIVED from `aging_points` (see above). It is the **cumulative** account
  of what age has done to the character, distinct from the aging log's per-year
  one-liners below, so the UI renders it as a multi-line field like its analogue
  `Entity.warping_effect` (guided-creation-review-2026-08 #26). Source:
  `Ars Magica - Definitive Edition (Core Rules).md:16563-16577` (## Aging).
- **Aging log** — `Entity.aging_log: Vec<AgingLogEntry>` (skip-if-empty). `year`
  is the **first** field so the derived `Ord` sorts the log chronologically via
  `Entity::normalize()`. Shipped in D2 as `{ year: i32, effect: String }` — per-year
  free-text outcomes, a pure annotation. **M6/6b6 widened it** into the full record
  of a resolved aging year and made `year` optional (`SCHEMA_VERSION` 14 → 15); the
  free text stays authoritative for a hand-written entry. See **Aging (M6/6b6)**
  below. Source: `Ars Magica - Definitive Edition (Core Rules).md:16563-16577`
  (## Aging).

App/UI: `save_entity_to_path` (`arm-app/src/ruleset_io.rs`) now **stamps**
`arm_rules::SCHEMA_VERSION` onto the entity on write, so app-written saves never
drift from the engine version (the frontend `SCHEMA_VERSION` constant in
`ui/src/lib/state.svelte.ts` was likewise reconciled to 11). The four fields are
entered in `CharacterDetails.svelte`; every label is a Fluent key.

**Derived-score formulas** (in `effective/warping.rs`; slice 5i's `derived.rs` re-exports /
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
points by `effective/warping.rs::aging_drops(entity, char)`: once the points **exceed** the
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

**Validation (advisory, single path).** `validation/aging.rs::validate_aging` (:52) emits one
**warning** (never blocking), per Characteristic whose accrued points force a drop:
`excessive_aging_reduction` — when the derived drops would push the score below the
−5 floor (it is clamped regardless; args `characteristic`, `reduction`, `min`).
Fluent key `issue-excessive_aging_reduction` (en/de). (An earlier
`aging_points_force_drop` note announcing each auto-applied drop was removed as
validation noise — the drop is automatic and already reflected in the effective
score, so it is not an entry problem worth flagging.)

`validate_aging` also emits the entity-wide **warning**
`aging_rolls_pending` (arg `age`, phase `aging`) when the character has
reached `ruleset.aging()?.first_roll_age()` and its `aging_log` is empty — the rolls
the rules owe before play have not been made (`:2232`, `:16565`; the threshold is a
**data** value since M6/6b6, see **Aging (M6/6b6)** below). Emitted
for **every** character of that age, not only one built through its life stages,
which is why it lives here rather than in `validate_life_stage_plan` (that one
returns early without a plan). The **log**, never `aging_points`, settles it: a
roll can legitimately produce no points. Fluent key
`issue-aging_rolls_pending` (en/de) — the `life_stage_` prefix it shipped under in
6b5a was dropped in M6/6b6, because the finding fires for every character over the
threshold and the `life_stage_*` codes are all filed under `experience` (they were
`abilities` until the Slice 2 phase split below). Filed under
`review` only because
no aging phase exists yet — 6b6 adds `CreationPhase::Aging` and moves it there.

App/UI: `EffectiveScores` gains `decrepitude_score: u8` and widens `warping_points`
to `u32`; the Details tab (`CharacterDetails.svelte`) enters identity fields,
Warping Points and twilight scars, while the aging points per Characteristic (with
an `aging-points-note` explaining drops are auto-derived) sit on the Aging tab
(`AgingPanel.svelte` → `AgingRecordPanel.svelte`; they were on Details until the
Slice 3 tab split). Between them they show the engine-computed
Decrepitude / Warping **scores** and the aging-lowered Characteristics (never
recomputed in JS). Fluent keys en/de: identity + aging block (`identity-*`,
`aging-*` incl. `aging-points-note`, `warping-points-label`, `twilight-*`,
`decrepitude-{label,readout}`).

#### True Faith — special derived score (`true_faith_grant`)
> "You have a True Faith score of 1 and can gain more."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:5169-5171`.
- Data: `virtue.true_faith` — `effects: [{ true_faith_grant, score: 1 }]`.
- Implementation: `Effect::TrueFaithGrant { score }` → `effective/might.rs::true_faith`
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
- Implementation: `effective/ability.rs::granted_ability_floor`, folded into
  `effective_ability_score`; `ruleset/integrity.rs::validate_effect_refs` checks the
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
- **M6 (guided wizard):** *done* — the life-stage XP acquisition (early childhood
  75+45 xp `:2378`; later life 15/20/10 xp/yr `:2390-2394`; age→max-score cap
  `:2368-2374`) in **6b2**, the Sample Childhood packages (`:2380-2388` —
  catalogue, engine and picker, see **Sample Childhood packages** below) in
  **6b3**, the magus apprenticeship and post-apprenticeship Art-XP flow
  (`:2433-2471`) in **6b4**/**6b5**, the aging engine for characters over 35
  (`:16563-16617`, see **Aging (M6/6b6)** below) in **6b6**, and the Crisis of
  `:16619-16640` — which this bullet once excluded — in **6b7**. The
  age→max-score *cap* itself is enforced as direct-entry validation in M4/4e; only
  the XP *acquisition* and aging *rolls* are M6.

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
  valid) in `ruleset/integrity.rs`; `Prereq::House` evaluation and the `validate_house` pass
  in `validation/magus.rs` (:31). `Bonisagus`/`Mercere`/`Flambeau` reuse the generic
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
- Implementation: `validation/caps.rs::validate_caps` (:22) gains a virtue loop mirroring the
  flaw loop, counting `entity.selections` only (granted Hermetic Virtues exempt),
  emitting `too_many_major_hermetic_virtues` (code derived as
  `too_many_[major_]<category>_virtues`). Fluent key `issue-too_many_major_hermetic_virtues`
  in both locales; `dynamic_virtue_cap_codes()` extends the Fluent-coverage test.

#### ≥1 Hermetic Flaw (magus guideline)
> "You should take at least one Hermetic Flaw."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2860`.
- A "should", so a **soft warning** — `validation/magus.rs::validate_house` (:31) emits
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

| Item | Kind | Source (Ars Magica - Definitive Edition (Core Rules).md) |
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
`EffectiveBudget` (`validation/balance.rs`, :89): `flaw_ceiling = base + bonus_flaw`,
`virtue_ceiling = base + bonus_flaw·rate + bonus_free_virtue`,
`funded = flaw·rate + bonus_free_virtue` (rate = 2). Zero for a non-mythic type,
so the check reduces exactly to the base budget.

| Type | free status | free Minor | required Virtues (budgeted) | required Flaw (default) | bonus | Source |
|------|-------------|-----------|------------------------------|--------------------------|-------|--------|
| Devil Child | `virtue.devil_child` | Demonic Might **or** Powers | Demonic Blood, Puissant (Guile) | Tragic Life | +7 F, **+3 free V** | Ars Magica - Definitive Edition (Core Rules).md `:2643-2666` |
| Faerie Doctor | `virtue.faerie_doctor` | Dowsing | Wise One, Curse-Throwing | Faerie Friend, Dutybound | none | Ars Magica - Definitive Edition (Core Rules).md `:2668-2704` |
| Nephilim | `virtue.nephilim` | Strong Angelic Heritage | Blood of the Nephilim, Greater Immunity, Great Sta, Great Str, Improved Characteristics, Sense Holiness | — (5 F fund the +10 V) | none | Ars Magica - Definitive Edition (Core Rules).md `:2714-2739` |
| Spirit Votary | `virtue.spirit_votary` | Second Sight | Spiritual Pact (+ 1 Major/3 Minor Supernatural, **advisory**) | Pagan | +7 F | Ars Magica - Definitive Edition (Core Rules).md `:2741-2764` |

- Devil Child's **+3 free V / +7 F**: Ars Magica - Definitive Edition (Core Rules).md `:2664` ("three more points of Virtues
  at no cost… and an additional seven points of Flaws"). Verified maxed budget:
  flaw 17, virtue `20 + 14 + 3 = 37`.
- Spirit Votary's **+7 F**: *Ars Magica 5e - Realms of Power - Magic.md*`:5486` ("may take up to 7
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
| `flaw.tragic_life` | **Major, Story** (Tainted) | Ars Magica - Definitive Edition (Core Rules).md `:6855-6870` |
| `virtue.faerie_doctor` | Special/Free, Social Status | *Faerie*`:6394-6399` |
| `virtue.dowsing` (grants `ability.dowsing` 1) | Minor, Supernatural | Ars Magica - Definitive Edition (Core Rules).md `:3703-3706` |
| `virtue.curse_throwing` (grants `ability.curse_throwing` 1) | Major, Supernatural | *Faerie*`:6342-6347` |
| `virtue.wise_one` | Minor, Social Status | Ars Magica - Definitive Edition (Core Rules).md `:5257-5260` |
| `flaw.faerie_friend` | Minor, Story | Ars Magica - Definitive Edition (Core Rules).md `:6052-6055` |
| `flaw.dutybound` | Minor, Personality | Ars Magica - Definitive Edition (Core Rules).md `:5992-5995` |
| `virtue.nephilim` | Free, Social Status | *Divine*`:3485-3513` |
| `virtue.blood_of_the_nephilim` | Major, Supernatural | *Divine*`:1941-1954` |
| `virtue.strong_angelic_heritage` (req. Blood of the Nephilim) | Minor, Supernatural | *Divine*`:1969-1980` |
| `virtue.greater_immunity` (plain; "Disease" is an in-play target, no param) | Major, Supernatural | Ars Magica - Definitive Edition (Core Rules).md `:4009-4016` |
| `virtue.sense_holiness_and_unholiness` (grants `ability.sense_holiness_and_unholiness` 1) | Minor, Supernatural | Ars Magica - Definitive Edition (Core Rules).md `:4926-4929` |
| `virtue.spirit_votary` | Free, Supernatural (→ Social Status) | *Magic*`:5480-5486` |
| `virtue.spiritual_pact` | Major, Supernatural | *Magic*`:5488-5502` |
| `flaw.pagan` (Major or Minor; seeded Major) | Major, Personality | Ars Magica - Definitive Edition (Core Rules).md `:6570-6573` |
| `ability.curse_throwing` (requires_training) | Supernatural | Ars Magica - Definitive Edition (Core Rules).md `:7396-7430` |
| `ability.dowsing` (requires_training) | Supernatural | Ars Magica - Definitive Edition (Core Rules).md `:7439-7442` |
| `ability.sense_holiness_and_unholiness` (requires_training) | Supernatural | Ars Magica - Definitive Edition (Core Rules).md `:7720-7723` |

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
requisites, parameters }`, with `level: None` marking a **General** spell learned
at a per-character level). The chosen spells live on `Entity::spells`
(`SpellSelection { spell, level, mastery, parameter }`); `validate_spells`
(`validation/magus.rs`) enforces two sourced constraints, both magus-only (gated
on the profile `is_magus`).

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

`:2435`'s **other** number — the 240 experience points of the same sentence — lives in
`rules/core/life_stages.json` instead, on the apprenticeship block: see
**Apprenticeship — 240 experience points across Arts and Abilities (M6/6b4)** in the
*Life-stage experience* section below, which records why the two halves of one
sentence are deliberately split across two files. `ApprenticeshipRules` carries no
`spell_levels` field, so this profile value stays the single source.

The base budget is selected in one place, `effective::spell_levels_base` — the
per-character `Entity::spell_levels_override` (an optional stored *choice*, an
app affordance, not a rulebook mechanic) when set, otherwise the profile's
`spell_levels`. Both the effective payload (`arm-app/src/ruleset_io.rs`) and the
`over_spell_levels` validator (`validation/magus.rs`) call it, so the displayed
and validated budgets cannot diverge. `spell_levels_budget` then adds the
Skilled/Weak Parens `Effect::SpellLevels` modifiers on top of that base — and, for
a magus generated some years out of apprenticeship, the levels it took out of its
post-Gauntlet points (`:2471`): see **Life as a magus after the Gauntlet — 30 points
per year (M6/6b5)** below. So 120 is the budget *at the Gauntlet*, not a ceiling.

**Unspent levels warn, and say nothing more (Slice 11 / #30).** Overspending the
budget was an error while leaving 60 of 120 levels unlearned produced **silence** —
the same asymmetry the experience pool had. `validation/magus.rs::validate_spells`
now emits `spell_levels_unspent` (warning, phase `spells`, args `used`/`budget`/
`unspent`) as the `else if` branch of the very `used > budget` test that emits
`over_spell_levels`, so the pair reads one budget and can never disagree about it, and
never both fire.

**Why the message is purely factual, and must stay so.** The wording is "N of M
levels of spells are still unspent" and asserts nothing about consequence. This is
deliberate and was searched for while reviewing: `:2215` grants the levels ("Take 120
levels of spells") and **no passage anywhere in `rules/source/en/` states that
unlearned levels are lost, wasted or forfeit.** Its sibling
`restricted_xp_unspent` *may* say "will be wasted" because a restricted pool is
earmarked to a named list and buys nothing outside it — the reading recorded under
**Early childhood** below, off `:2378` — but that backing does not extend here.
The **absence** of a waste rule is the whole reason for the bare count — so a later
pass that "improves" the copy into "will be wasted" would be inventing a rule, which
`CLAUDE.md`'s provenance rule prohibits. `ui/src/lib/i18n.test.ts` pins it in both
locales.

**The base is offered for editing in direct entry only (Slice 8 / #19).** `:2215`'s
"Take 120 levels of spells" is a flat grant with no in-rules variation, and every
variation the rules *do* allow already reaches the budget elsewhere — Skilled/Weak
Parens as `Effect::SpellLevels`, and the post-Gauntlet split as `:2471`'s levels. So
`SpellBudgetBar` takes a `readonlyBase` prop: the guided wizard passes it (the step
shows the grant), the editor does not (direct entry exists to record a character the
rules-as-written did not build, and `spell_levels_override` is exactly that
affordance). The prop is declared at the mount, in `WizardStep.svelte`'s `STEPS`
table, so the divergence is visible where the two surfaces are wired rather than
inferred inside the component from which flow is running.

**The bar's displayed pair closes against base + post-Gauntlet levels, not the base
(Slice 8 / #18).** `spellLevelAllocation` (`ui/src/lib/derive.ts`) charges the
post-Gauntlet levels to the same unconditional side as the base, so the figure printed
beside the used total is `denominator = base + lifeStage`; `denominator - baseUsed`
is `available`, always. Printing the editable base there instead read "150 / 120,
Available: 0" for a magus the engine considers exactly balanced, so the two figures
are now rendered in separate slots.

**Per-spell cap — Technique + Form + Intelligence + Magic Theory + 3.**

> `:2465` "The highest level spell you can learn is equal to Technique + Form +
> Intelligence + Magic Theory +3 … If the spell has requisites … they apply to
> this total as well."

`spell_level_cap(entity, ruleset, technique, form)` (`effective/spell.rs`) computes it
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
`(spell, level, parameter)`; an unresolved General spell (no chosen level) warns
(`spell_level_unresolved`) and is excluded from the budget sum.

**Parametrized spells — one version per Hermetic Form.**

> `:15791-15794` "**Wizard's Boost (Form)** … There are ten versions of this
> spell, one for each Hermetic Form." (Also **Mirror of Opposition (form)**
> `:15776-15779`, **Wizard's Reach (Form)** `:15801-15804`, and **Unravelling the
> Fabric of (Form)** `:15843-15846`.)

These four meta-magic Vim spells (MuVi / PeVi, all General) bake the *target*
spell's Form into their name. The `(Form)` is modelled as a `ParameterDef`
(`{ key: "form", type: "ref", domain: "form" }`) on `Spell.parameters`
(`spell.rs`), chosen per selection via `SpellSelection.parameter` (an `art.*`
id). It is **display + identity only**: the spell's own `technique`/`form` stay
the catalogue Vim Arts (`art.muto`/`art.perdo` + `art.vim`), so casting totals,
penetration (`derived/casting.rs::penetration`), and the per-spell cap
(`spell_level_cap`) keep using Vim — the parameter never resolves an "effective
Te/Fo". `validate_spells` (`validation/magus.rs`) requires the parameter
(`missing_param` if absent) and resolves it to the Form domain
(`unknown_param_value` otherwise, via `selections::param_value_resolves`); the
same base spell may be taken **once per distinct Form** (the `parameter` is part
of the dedup key). The value flows onto `PenetrationLine.parameter` so two
instances of one spell id stay distinct. Data: `rules/core/spells.json` (the
`parameters` array; Te/Fo unchanged) + `rules/i18n/{en,de}/spells.json` (name
retokenized to `{form}`).

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

- **Load-time** (`Ruleset::validate_spell_refs`, `ruleset/integrity.rs`): for a fixed
  `level`, ritual ⇒ `level ≥ 20`, non-ritual ⇒ `level ≤ 50`; a non-ritual spell
  may not have `duration = Year` or `target = Boundary`, nor be a Momentary Creo
  spell with `creates_lasting`. Vision target is exempt from the Boundary rule.
- **Per-entity** (`validate_spells`, `validation/magus.rs`, :336): the *resolved* learned
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

**Both contributions are also surfaced on their own** (M6/6b8c for the experience
half; 6b5a already did the spell-levels half): `XpAllocation::general_bonus` beside
`general_pool`, exactly as `spell_levels_bonus` sits beside `spell_levels_budget`.
The bars need it because the *editable* figure is the base — the typed pool, or the
life-stage block — while the pool the solve funds from is base + bonus, and a bar
showing only the total cannot say why the two differ. Until 6b8c the XP bar charged
the spend against the typed base under flat funding, so a magus with Skilled Parens
spent the 60 the engine granted and reported an overspend nothing had raised; it now
reads the engine's pool in both funding modes and lists the bonus as its own entry,
spent before the base the way a restricted pool is (`generalXpAllocation` in
`ui/src/lib/derive.ts`, the twin of `spellLevelAllocation`).

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
weapon"). `validate_weapon_refs` (in `ruleset/integrity.rs`) requires each weapon's
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
Attack). `min_strength` for a weapon and a shield are met separately (`:16997`).

**Weapon + shield combine** — a weapon+shield combatant **adds both** rows'
modifiers (computed in 5i):

> `:16656` "If the character is using a weapon and a shield, add together the
> modifiers of the weapon and the shield to get the final modifier."

**Encumbrance** (the Load→Burden table; Encumbrance = `max(0, Burden − max(0,Str))`)
is defined at `:17103-17123` and **computed in 5i**, not here:

> `:17109-17123` "| Total Load | Burden | | 0 | 0 | | 1 | 1 | … | 55 | 10 |"

`validate_equipment` (in `validation/equipment.rs`, :16) emits `unknown_equipment` (error) for a
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
| Lab Total | `:10276-10278`, `:4151-4154` | Int + Magic Theory + Technique + Form + Aura + flat LabTotalMod (+ focus / halving as casting). "**YOUR BASIC LAB TOTAL IS: Technique + Form + Intelligence + Magic Theory + Aura Modifier**" (`:10276`); `:4151-4154` is Inventive Genius, the flat `LabTotalMod`. (Corrects the earlier `:4143-4154`, which is the Inspirational / Intuition / Inventive Genius Virtue block, not the formula.) |
| Penetration | `:9159-9161` | per known spell: Casting Total − Level + Penetration score |
| Weak Magic | `:7064-7067` | halves Penetration **after** subtracting level (not the casting total) |
| Magic Resistance | `:9390-9398` | per Form: Form + 5 × Parma Magica (Form-base rule `:9390`, Parma "five times" `:9398`); Limited MR drops the Form bonus, Flawed Parma halves |
| Longevity (stored) | `:10662`, `:10668`, `:10670` | the aging bonus is the **player-entered** `LongevityRitual.bonus`, passed through for **both** sources; `entered: false` marks an unfilled field so a placeholder 0 is never read as a claim. Bronze cord noted for aging-resistance (`:10840-10844`), via `cord_score` so it respects the +5 maximum (`:10836`) and matches the Soak and cord-cost figures |
| Longevity hint | `:10662`, `:10276-10278`, `:17658`, `:5909-5915`, `:5962-5964` | self-made only: `LongevityHint { lab_total, suggested_bonus, halved }` — Creo Corpus Lab Total (Int + Magic Theory + Creo + Corpus + Aura + flat LabTotalMod), halved by a Deficient Creo/Corpus and again by Difficult Longevity Ritual, then `suggested_bonus = ceil(lab_total / 5)` floored at 0. **Read-only guidance** — never written into the entity. `derived/lab.rs::suggested_longevity_bonus` / `creo_corpus_lab_total` |
| Masterpiece | `:4476-4479`, `:10410`, `:7060-7063` | magus with the Masterpiece Virtue (`Effect::MasterpieceItem` marker) surfaces a **read-only** lesser-enchanted-item cap = **best base `(Te,Fo)` Lab Total ÷ 2** (the lesser-enchantment rule caps single-season instillation at Lab Total ≥ 2×effect level, `:10410`; vis costs ignored per the Virtue). The best cell is picked by (and the cap built from) `LabTotal.enchanting`, not the plain `total`: designing the item is "creating" an enchanted item, so a **Weak Enchanter** magus's halved figure (`:7060-7063`) is "the regular rules for construction of such a device" (`:4476-4479`) for him too — `enchanting` equals `total` for everyone else, so the formula is unchanged for a magus without the Flaw (round 3, G1: the un-halved `total` used to leak through here, doubling the cap). No focus doubling. The engine does **not** create the device or spend an item-level budget — the player still enters the actual lesser enchanted item by hand under Magic Items; this is guidance only. `masterpiece_item_cap` / `DerivedTotals.masterpiece` |
| Talisman capacity | `:10619`, `:4347-4349`, `:4842-4850` | magus **with a talisman** surfaces a **read-only** enchantment capacity in pawns of Vim vis: "The maximum number of pawns of Vim vis that may be used to prepare a talisman is equal to the sum of the magus's highest Technique and highest Form" (`:10619`). Taken from the per-Art maxima of **effective** scores (`effective_art_score`, so Puissant Art folds in), **not** the best `lab_totals` pair — a Deficient Art halves *totals*, never the score, and a discriminating test pins that. Ties go to the alphabetically first Art (`Ruleset::art_ids_of` is sorted); a magus with no bought Arts still reads out, at 0 pawns. **Non-goal**: instilled `TalismanEffect` levels are charged against **no** budget — `item_level_budget` comes only from the Redcap-only Virtues (Magic Items "You must be a Redcap to take this Virtue", `:4347-4349`; Redcap's fifty starting levels `:4842-4850`, which also states "You may not take The Gift", `:4850`), so it can never fund a magus's talisman, and the talisman's real limit is this vis capacity, which the model cannot enforce (it holds no vis stock). `derived/familiar.rs::talisman_capacity` / `DerivedTotals.talisman_capacity` |
| Familiar bonding level | `:10824`, `:10828` | magus **with a familiar** surfaces a **read-only** bonding level = **Magic Might + 25 + 5 × Size**: "The level for the enchantment is equal to 25 plus the familiar's Magic Might plus 5 times its Size. If the familiar has negative Size, this reduces the level for the enchantment" (`:10824`), restated as "**FAMILIAR BONDING LEVEL: Familiar's Magic Might + 25 + (5 x Size)**" (`:10828`). Size is signed and commonly negative, so the level routinely drops *below* 25 — the book's own worked example (Size -2, Might 10 → level 25) is a named test. A familiar with **no entered Might** contributes 0 rather than suppressing the read-out; the panel says "no Magic Might entered" instead. `derived/familiar.rs::familiar_binding_level` |
| Familiar bonding Lab Total | `:10818`, `:10822`, `:10826`, `:10824` | the **ordinary Lab Total shape** — "any appropriate Technique + any appropriate Form + Int + Magic Theory + Aura Modifier" (`:10818`), restated as "**FAMILIAR BONDING LAB TOTAL**" (`:10826`) — so `lab_totals()` is **reused** and the best `(Te,Fo)` cell taken with `max_by_key`, the same max-over-grid reuse as Masterpiece. Which Arts are *appropriate* to a given beast is prose the engine cannot evaluate, and "Any magus should be able to find an animal that he can bind with his best Technique and Form" (`:10822`), so the best cell is the honest figure. **Unlike Masterpiece, a focus applies here**: "Puissant Arts and foci may apply to this" (`:10818`) — so `lab_total_within_focus` is surfaced as a **separate conditional** figure the UI labels as such (whether *this* familiar falls inside the focus's narrow field is a troupe judgment); Puissant Arts need no separate figure, `effective_art_score` folds them in. `lab_total_reaches_level` reports "A magus can only bind a familiar if his Lab Total equals or exceeds this level" (`:10824`). `derived/familiar.rs::familiar_readout` / `FamiliarBinding` |
| Familiar cord cost | `:10836` | cord scores 0…+5 cost **0 / 5 / 15 / 30 / 50 / 75** points: "a strength of +1 requires 5 points, a score of +2 requires 15 points, a score of +3 requires 30 points, a score of +4 requires 50 points, and a score of +5 (the maximum) requires 75 points" (`:10836`). The curve is a fixed 5-entry rule constant, so it is a Rust `const CORD_COST_TABLE` (precedent: `LOAD_TABLE` for Encumbrance) with this row as its provenance home. The same line also fixes the **+5 maximum** ("rated from 0 to +5 … a score of +5 (the maximum)"), which is stated in **one** place in the engine: `pub const MAX_CORD_SCORE: u8 = 5` in `types.rs`, beside the `Familiar` type whose fields it bounds. Two clamps read it and neither restates the number. (1) **On read** — `derived.rs::cord_score(raw)`, which **every** cord consumer routes through: `cord_points_spent` (the table index), `soak`'s `bronze_cord` addend, and `longevity_bonus`'s `bronze_cord` note. Cord fields are plain `u8`, so a hand-edited or legacy save can carry any value up to 255; unclamped, one entered number produced three contradictory figures (75 points spent / +255 Soak / +255 aging-resistance) and a raw table index would panic inside the `derived_totals` command and kill the read-out panel. (2) **On store** — `Familiar::normalize` clamps the three fields, so an out-of-range value **self-heals on the next save**, exactly as a zero Characteristic is pruned there and for the same canonical-serialization reason: since every consumer clamps, `cord_bronze: 255` and `cord_bronze: 5` are the same statement to the engine, yet they serialize differently and *display* differently (the panel renders the raw 255 in an input declaring `max="5"`, beside read-outs computed from 5) — a disagreement the user cannot resolve and that saving would otherwise perpetuate forever. UI input bounds are a separate, non-durable layer and do not replace either clamp. Tests: `cord_points_spent_clamps_a_score_above_the_curve`, `an_out_of_range_bronze_cord_reads_the_same_for_every_consumer`, `entity_normalize_clamps_out_of_range_familiar_cords`. `cord_points_within_lab_total` reports "The total cost of the cords you buy cannot exceed the magus's Lab Total" (`:10836`). `types.rs::MAX_CORD_SCORE` + `Familiar::normalize` / `derived.rs::cord_score` / `derived/familiar.rs::cord_points_spent` |
| Familiar invested powers | `:10866`, `:10862-10884` | the total level of the powers invested in the bond, summed **for information only**: "there is no limit to the number of powers which may be invested in a familiar" (`:10866`). So — unlike a being's own `Entity.powers`, which `power_levels_budget` bounds — there is **no budget bar** and no issue to raise, and the UI must not render one. Vis costs (`:10882`, one pawn per ten levels) are out of scope: the model holds no vis stock. `derived/familiar.rs::familiar_invested_power_levels` |
| Combat | `:16658-16670` | Init = Qik + WpnInit − Enc + CombatMod; Attack = Dex + Ability + WpnAtk + CombatMod; Defense = Qik + Ability + WpnDef + CombatMod; Damage = Str + WpnDam + CombatMod |
| Weapon+shield | `:16656`, `:7746`, `:1317`, `:1467-1472` | "If the character is using a weapon and a shield, add together the modifiers of the weapon and the shield to get the final modifier" (`:16656`). A shield is raised and dropped at will, so **each equipped one-handed weapon emits TWO `CombatLine`s** while any shield is equipped: the with-shield line first (every equipped shield's Init/Atk/Def mods added; all their ids listed in `CombatLine.shields`, since multiple shields stay **summed** into one pseudo-shield), then the bare line (`shields` empty, shield mods zeroed). The two differ **only** by the shield modifiers — Encumbrance and the specialization bonus are shield-independent, the latter explicitly so (`:7746`, see below). With-shield-first mirrors the book's own statblocks, which print the weapon-and-shield lines ahead of the rest (`:1467-1472` "- Long sword and heater shield (mounted): … - Great sword (mounted): …"). The renderers label the with-shield line `<weapon> <joiner> <shield>`; the joiner is the **translatable** `derived-combat-shield-joiner` Fluent key, not a hardcoded `&`, because the book alternates `&` (`:1317` "- Axe & Heater Shield: Init +1, Attack +17, Defense +15, Damage +8") with "and" (`:1468`) and the German rulebook writes `&` too. With no shield equipped the output is unchanged: one bare line per weapon. `derived/combat.rs::combat_totals` / `export/sections.rs::combat_line_name` / `derive.ts::combatRowLabel` |
| Two-handed weapon | `:7494`, `:17008-17013` | a two-handed weapon (`Weapon.two_handed`) cannot be paired with a shield, so it receives **no** shield Init/Atk/Def mods (the shield still counts toward Load, `:17107`) and therefore emits exactly **one** line — there is no second way to wield it. The 9 Great-Weapon melee weapons ("Fighting with a weapon which requires two hands to use", `:7494`) + both bows are flagged in `rules/core/equipment.json`. The bows come from the missile table's asterisked rows and its footnote (`:17008-17013`, i.e. `| Sling* …` through `:17013` "\* Requires two free hands to load and fire."), **not** from the Bows Ability entry at `:7333-7334` or the `Bow, Long:` flavor note at `:17099`, neither of which says anything about two hands. The **Sling** shares that asterisk but stays unflagged: it is a thrown weapon and keeps the status-quo shield handling. (`:17017` is the missile table's "Atk:" column legend — not a citation for this rule.) `derived/combat.rs::combat_totals` |
| Shield + two-handed advisory | `:7494`, `:17063`, `:16975` | advisory `shield_with_two_handed_weapon` warning when an equipped shield accompanies **only** two-handed weapon(s) — its dropped modifiers otherwise look like a bug (buckler prose `:17063`; shield table "Single" Ability column `:16975`). Non-blocking. `validation/equipment.rs` |
| Specialization +1 | `:7122`, `:7139`, `:7746` | when `EquipmentSlot.specialization_applies` is set AND the weapon's combat Ability carries a non-empty specialty, the Ability acts "as if your score were one level higher" (`:7122`, Single Weapon longsword example) for **Attack and Defense only** (Damage/Init do not use the Ability); "Add +1 when using an Ability's specialization" (`:7139`). The bonus is **shield-independent** and so applies identically to the with-shield and the bare line: a Single Weapon specialty is "any one weapon or shield, which covers using that weapon with any shield or none, and that shield with any weapon" (`:7746`). `derived/combat.rs::specialization_bonus` |
| Enc-exempt (conditional) | `:17105`, `:17107` | Attack/Defense are Encumbrance-penalized **only** when the Encumbrance is *not* largely weapons + armor; Init is **always** penalized (`:16658`). "Largely due to weapons and armor" is read as **combat-gear Load ≥ half of total Load** (majority) — an explicit interpretation assumption, since the rules give no numeric threshold. Combat gear = every carried weapon/shield/armor (equipped or not, `:17107` counts all Load). `derived/combat.rs::combat_encumbrance_applies` |
| Soak | `:16667` | Stamina + Armor Protection + SoakMod (Tough +3) + Bronze cord; Form bonus situational (entered 0). The Bronze-cord addend goes through `cord_score` (the +5 maximum, `:10836`) so it cannot disagree with the cord-cost or Longevity read-outs |
| Encumbrance | `:17103-17123` | Burden from Load table `[0,1,3,6,10,15,21,28,36,45,55]→[0..10]`; Enc = `max(0, Burden − max(0,Str))` |
| Fatigue | `:17127-17129` | Winded/Weary −1, Tired −3, Dazed −5, adjusted by HealthMod fatigue delta |
| Wounds | `:17167-17191` | Size unit `u = max(1, Size+5)`; Light 1..u, Medium u+1..2u, Heavy 2u+1..3u, Incap 3u+1..4u, Dead 4u+1.. ; penalties −1/−3/−5 adjusted by HealthMod wound delta |
| Decrepitude / Warping | `:16617`, `:16464-16475` | **reused** from `effective/warping.rs` (`decrepitude_score`, `warping_score`), not reimplemented |

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
effects are actually consumed. Worked-example tests: Longevity hint Lab Total 35 →
+7 (`:2573`, `:2488`); casting total with Encumbrance + Focus (base vs within-focus, Method
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

**Surfaced-only families** (study / conditional-casting /
wound-recovery / realm-conditional MR: `AdvancementMod`, the non-computed
`SpecialCastingMod` kinds, `AbilityRollMod`, the `HealthTrack::{FatigueRoll,
CastingFatigue, Recovery}` tracks, and the non-`no_form_bonus` `MagicResistanceMod`
kinds — `ModifierFamily::MagicResistance` — aura_bonus and the realm
susceptibilities) are **listed** as labelled `SurfacedModifier`s, not folded into a
simulated number, because the app does not simulate those subsystems.

**`AgingMod` left this list in M6/6b6.** `derived.rs` still lists it as a standing
modifier, but it is no longer surfaced-*only*: `aging.rs` consumes it — see
**Aging (M6/6b6)**, and the `AgingMod` row of the in-play effect table below.

#### M5.5a — Longevity Ritual: stored value + live hint

M5/5i **derived** the self-made aging bonus from the current Creo Corpus Lab Total.
That was wrong, and this slice replaces it with a stored player-entered value plus a
read-only suggestion.

**Why the bonus cannot be derived.** The formula is "+1 bonus for every five points
or fraction of Creo Corpus Lab Total" (`:10662`) — but the Lab Total in that sentence
is the one the *creating* magus had in the season the ritual was made. The rules
never restate this as a standalone sentence; it follows from two passages, so it is
recorded here as an **inference from quoted text**, not a quoted rule:

> If you reinvent the ritual to take advantage of increased Art scores, you can
> choose not to use extra vis. — `:10670`

> A Longevity Ritual's effect lasts until you suffer an aging crisis […] After this,
> the ritual loses its effectiveness and the focus must be repeated. — `:10668`

Reinvention is what captures raised Arts, and a failed ritual is repeated *unchanged*.
Deriving the bonus live therefore let a Creo increase or a move to a stronger aura
silently rewrite a past event. So `LongevityRitual.bonus: Option<i8>` is now entered
and stored for **both** sources (`None` = not entered yet, never a claimed 0), and
`LongevityBonus.entered` lets the UI distinguish an unfilled field from a deliberate
0. The derivation survives only as `LongevityHint` beside the input.

**`LongevityRitual.focus`** (free text, additive — no schema bump) records the
ritual's culminating focus:

> The ritual takes a season, and culminates in some sort of focus, which is
> appropriate to the magus in question. — `:10656`

The same line carries the consequence the UI surfaces as a note — "the magus becomes
permanently sterile" (`:10656`). The rules attach no mechanics to either, so both are
free text / a note; nothing is validated.

**The `aura != 0` gate is deleted.** 5i suppressed the whole bonus unless
`entity.aura != 0`, which has no source support: the Aura Modifier is a plain addend
in the Lab Total (`:10276-10278`), `lab_totals()` has never gated on it, and a
zero-aura location is explicitly unhindered —

> The mundane has no aura rating — in fact, it is the absence of aura, so powers used
> there function without hindrance. — `:17658`

A zero-aura magus now gets a hint; a negative aura simply lowers it.

**Two halvings now apply to the hint** (`creo_corpus_lab_total` returns
`(total, halved)`):

> Almost all totals (including Casting Totals and Lab Totals, but excluding Magic
> Resistance) to which a particular Form is added are halved. — Deficient Form,
> `:5909-5911`; Deficient Technique likewise, `:5913-5915`

> Anyone (including yourself) creating a Longevity Ritual for you must halve their
> Lab Total. — Difficult Longevity Ritual, `:5962-5964`

This is the **first read of `HalvableTotal::LabLongevity`**: before M5.5a
`flaw.difficult_longevity_ritual` was collected by `in_play_mods` and moved no
number.

**That the two halvings compound is an inference, not a quoted rule.** Each Flaw
says to halve the Lab Total and neither carves out the other, but no passage states
the interaction. Order is pinned base → Deficient → Difficult and is numerically
immaterial, since `halve()` truncates toward zero. `halved` is a single flag: the UI
marks the hint as halved without claiming which Flaw did it.

`suggested_bonus = ceil(lab_total / 5)`, floored at 0 — "every five points or
fraction" has no meaning below one point, and a negative Lab Total must not suggest a
negative bonus. Worked example on the shipped ruleset: Lab Total 35 → +7, the book's
own sheet line (`:2573`; the same magus's lab season at `:2488` shows the vis cost
too).

**Note on the Flaw's cited range.** `rules/core/virtues_flaws.json` records
`flaw.difficult_longevity_ritual` as `:5962-5965`; line 5965 is blank, so the
accurate inclusive range is **`:5962-5964`**, which is what this section and the
`derived.rs` comments cite. The JSON value is left for a data-only correction pass.

#### M5.5c — Familiar read-outs (guidance only)

Four numbers, all **read-only guidance** in the `masterpiece_item_cap` mould — see
the *Familiar bonding level* / *bonding Lab Total* / *cord cost* / *invested powers*
rows in the formula table above. They are aggregated by
`derived/familiar.rs::familiar_readout` into `FamiliarReadout { binding_level,
cord_points_spent, invested_power_levels, binding: FamiliarBinding }`, exposed as
`DerivedTotals.familiar` and **magus-gated** exactly like `talisman_capacity`.

**Nothing here can raise a `ValidationIssue`**, at any severity. That is a contract,
not an accident: whether a bonding season is legal turns on judgments the engine
cannot make (which Arts suit the beast, `:10818`; whether a Magical Focus covers it)
and on vis, which the model does not hold (`:10830`, `:10882`). So the engine reports
and the troupe decides. Pinned by
`validation::tests::fully_populated_familiar_raises_no_issues`, which drives a
familiar to every extreme the model allows — cords 5/5/5 (225 points, far past any
Lab Total), 450 levels of bond-invested powers, and a *Faerie* Might on a Hermetic
magus — and asserts the issue list is byte-identical to the same magus with no
familiar at all.

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
- **Hermetic Prestige `creation_effect`** — Ars Magica - Definitive Edition (Core Rules).md `:4071-4073`, a Hermetic
  Reputation at level **4** (the `:2518` Darius example's 3 is an errata slip; the
  Virtue text's 4 is authoritative). The stale "not in Core / Local-only" claim in
  the Reputations section above is corrected.
- **Famous `creation_effect`** — Ars Magica - Definitive Edition (Core Rules).md `:3861-3863`, a **player-chosen-type**
  Reputation at level 4 (5a-wire may add a player-selected `kind` param to
  `GrantsReputation`).

### In-play effect families (definitive input to slice 4 / 5b)

- **Magical Focus (major/minor)** — `virtue.major_magical_focus` (Ars Magica - Definitive Edition (Core Rules).md:4399-4422), `virtue.minor_magical_focus` (Ars Magica - Definitive Edition (Core Rules).md:4536-4538), `virtue.mythic_blood` (Ars Magica - Definitive Edition (Core Rules).md:4573-4589)
- **Flat casting-total bonus/penalty** — `virtue.method_caster` (Ars Magica - Definitive Edition (Core Rules).md:4524-4527), `flaw.poor_formulaic_magic` (Ars Magica - Definitive Edition (Core Rules).md:6610-6613), `flaw.afflicted_tongue` (Ars Magica - Definitive Edition (Core Rules).md:5655-5658), `virtue.life_boost` (Ars Magica - Definitive Edition (Core Rules).md:4295-4298), `virtue.leper_magus` (Ars Magica - Definitive Edition (Core Rules).md:4249-4252), `virtue.cyclic_magic_positive` (Ars Magica - Definitive Edition (Core Rules).md:3635-3638), `flaw.cyclic_magic_negative` (Ars Magica - Definitive Edition (Core Rules).md:5893-5896), `virtue.special_circumstances` (Ars Magica - Definitive Edition (Core Rules).md:4998-5001), `virtue.ways_of_the_land` (Ars Magica - Definitive Edition (Core Rules).md:5231-5234), `flaw.corrupted_spells` (Ars Magica - Definitive Edition (Core Rules).md:5859-5864), `flaw.susceptibility_to_divine_power` (Ars Magica - Definitive Edition (Core Rules).md:6815-6818)
- **Spontaneous-magic casting modifier** — `flaw.weak_spontaneous_magic` (Ars Magica - Definitive Edition (Core Rules).md:7084-7089), `virtue.diedne_magic` (Ars Magica - Definitive Edition (Core Rules).md:3675-3682), `virtue.faerie_raised_magic` (Ars Magica - Definitive Edition (Core Rules).md:3829-3842), `virtue.spell_improvisation` (Ars Magica - Definitive Edition (Core Rules).md:5002-5005), `virtue.life_linked_spontaneous_magic` (Ars Magica - Definitive Edition (Core Rules).md:4299-4306)
- **Art-halving (Technique / Form)** — `flaw.deficient_technique` (Ars Magica - Definitive Edition (Core Rules).md:5913-5915), `flaw.deficient_form` (Ars Magica - Definitive Edition (Core Rules).md:5909-5912)
- **Circumstantial casting/lab halving** — `flaw.deleterious_circumstances` (Ars Magica - Definitive Edition (Core Rules).md:5917-5920), `flaw.environmental_magic_condition` (Ars Magica - Definitive Edition (Core Rules).md:6020-6023), `flaw.short_ranged_magic` (Ars Magica - Definitive Edition (Core Rules).md:6737-6740)
- **Flat lab-total bonus/penalty** — `virtue.adept_laboratory_student` (Ars Magica - Definitive Edition (Core Rules).md:3368-3371), `virtue.aristotelian_training` (Ars Magica - Definitive Edition (Core Rules).md:3440-3443), `virtue.inventive_genius` (Ars Magica - Definitive Edition (Core Rules).md:4151-4154), `flaw.creative_block` (Ars Magica - Definitive Edition (Core Rules).md:5873-5876), `flaw.weak_scholar` (Ars Magica - Definitive Edition (Core Rules).md:7080-7083), `flaw.disjointed_magic` (Ars Magica - Definitive Edition (Core Rules).md:5972-5975), `flaw.the_constant_expression` (Ars Magica - Definitive Edition (Core Rules).md:5821-5838), `virtue.potent_magic_major` (Ars Magica - Definitive Edition (Core Rules).md:4740-4781), `virtue.potent_magic_minor` (Ars Magica - Definitive Edition (Core Rules).md:4740-4781)
- **Lab-total halving** — `flaw.weak_enchanter` (Ars Magica - Definitive Edition (Core Rules).md:7060-7063), `flaw.difficult_longevity_ritual` (Ars Magica - Definitive Edition (Core Rules).md:5962-5965)
- **Ritual effective-level bonus** — `virtue.mercurian_magic` (Ars Magica - Definitive Edition (Core Rules).md:4514-4523)
- **Penetration-total modifier** — `flaw.weak_magic` (Ars Magica - Definitive Edition (Core Rules).md:7064-7067)
- **Magic-resistance modifier** — `flaw.flawed_parma_magica` (Ars Magica - Definitive Edition (Core Rules).md:6142-6145), `flaw.limited_magic_resistance` (Ars Magica - Definitive Edition (Core Rules).md:6346-6349), `flaw.weak_magic_resistance` (Ars Magica - Definitive Edition (Core Rules).md:7068-7071), `flaw.susceptibility_to_faerie_power` (Ars Magica - Definitive Edition (Core Rules).md:6819-6822), `flaw.susceptibility_to_infernal_power` (Ars Magica - Definitive Edition (Core Rules).md:6823-6826), `virtue.commanding_aura` (Ars Magica - Definitive Edition (Core Rules).md:3579-3596)
- **Flat Soak bonus/penalty** — `virtue.tough` (Ars Magica - Definitive Edition (Core Rules).md:5145-5147), `flaw.frail` (Ars Magica - Definitive Edition (Core Rules).md:6190-6193)
- **Wound/fatigue penalty delta** — `virtue.enduring_constitution` (Ars Magica - Definitive Edition (Core Rules).md:3751-3754), `flaw.low_tolerance` (Ars Magica - Definitive Edition (Core Rules).md:6366-6369), `flaw.painful_magic` (Ars Magica - Definitive Edition (Core Rules).md:6574-6577), `flaw.vulnerable_casting` (Ars Magica - Definitive Edition (Core Rules).md:6993-7004), `virtue.withstand_casting` (Ars Magica - Definitive Edition (Core Rules).md:5261-5282), `flaw.obese` (Ars Magica - Definitive Edition (Core Rules).md:6516-6519), `flaw.short_of_breath` (Ars Magica - Definitive Edition (Core Rules).md:6733-6736), `virtue.long_winded` (Ars Magica - Definitive Edition (Core Rules).md:4327-4330)
- **Wound-recovery modifier** — `flaw.fragile_constitution` (Ars Magica - Definitive Edition (Core Rules).md:6186-6189), `virtue.rapid_convalescence` (Ars Magica - Definitive Edition (Core Rules).md:4834-4837)
- **Combat total modifier (atk/def/init/dam)** — `virtue.berserk` (Ars Magica - Definitive Edition (Core Rules).md:3500-3503), `flaw.hobbled` (Ars Magica - Definitive Edition (Core Rules).md:6260-6263), `flaw.lame` (Ars Magica - Definitive Edition (Core Rules).md:6330-6333), `flaw.missing_hand` (Ars Magica - Definitive Edition (Core Rules).md:6438-6441), `flaw.missing_eye` (Ars Magica - Definitive Edition (Core Rules).md:6434-6437), `flaw.poor_eyesight` (Ars Magica - Definitive Edition (Core Rules).md:6606-6609), `flaw.palsied_hands` (Ars Magica - Definitive Edition (Core Rules).md:6578-6581), `flaw.slow_reflexes` (Ars Magica - Definitive Edition (Core Rules).md:6763-6766), `virtue.lightning_reflexes` (Ars Magica - Definitive Edition (Core Rules).md:4311-4314), `virtue.fast_caster` (Ars Magica - Definitive Edition (Core Rules).md:3865-3868)
- **Study source-quality / advancement modifier** — `virtue.apt_student` (Ars Magica - Definitive Edition (Core Rules).md:3422-3425), `virtue.book_learner` (Ars Magica - Definitive Edition (Core Rules).md:3519-3522), `virtue.free_study` (Ars Magica - Definitive Edition (Core Rules).md:3937-3940), `virtue.good_teacher` (Ars Magica - Definitive Edition (Core Rules).md:3971-3974), `virtue.independent_study` (Ars Magica - Definitive Edition (Core Rules).md:4115-4118), `virtue.study_bonus` (Ars Magica - Definitive Edition (Core Rules).md:5056-5072), `flaw.unimaginative_learner` (Ars Magica - Definitive Edition (Core Rules).md:6915-6918), `flaw.poor_student` (Ars Magica - Definitive Edition (Core Rules).md:6626-6628), `flaw.incomprehensible` (Ars Magica - Definitive Edition (Core Rules).md:6294-6297), `virtue.secondary_insight` (Ars Magica - Definitive Edition (Core Rules).md:4892-4895), `flaw.loose_magic` (Ars Magica - Definitive Edition (Core Rules).md:6354-6357)
- **Aging / longevity modifier** — `flaw.age_quickly` (Ars Magica - Definitive Edition (Core Rules).md:5659-5662), `flaw.baneful_circumstances` (Ars Magica - Definitive Edition (Core Rules).md:5687-5690), `flaw.monstrous_blood` (Ars Magica - Definitive Edition (Core Rules).md:6454-6467), `virtue.bee_king` (Ars Magica - Definitive Edition (Core Rules).md:3484-3499), `virtue.faerie_blood` (Ars Magica - Definitive Edition (Core Rules).md:3797-3820), `virtue.magical_blood` (Ars Magica - Definitive Edition (Core Rules).md:4359-4372), `virtue.unaging` (Ars Magica - Definitive Edition (Core Rules).md:5187-5190), `flaw.bound_to_role_role` (Ars Magica - Definitive Edition (Core Rules).md:5735-5748), `flaw.leprosy` (Ars Magica - Definitive Edition (Core Rules).md:6338-6341), `flaw.poor_living_conditions` (Ars Magica - Definitive Edition (Core Rules).md:6618-6621), `virtue.mild_aging` (Ars Magica - Definitive Edition (Core Rules).md:4528-4531), `virtue.magian_lineage_major` (Ars Magica - Definitive Edition (Core Rules).md:4339-4346), `virtue.magian_lineage_minor` (Ars Magica - Definitive Edition (Core Rules).md:4339-4346)
- **Non-standard-casting penalty removal (Deft/Quiet/Subtle)** — **computed** into per-cell `NonStandardCasting` variants (`derived.rs`), not surfaced-only. Base Words/Gestures penalties `:9236-9245` (no voice −10, no gestures −5). `virtue.quiet_magic` (Ars Magica - Definitive Edition (Core Rules).md:4822-4826, +5 voice per casting, second casting eliminates), `virtue.subtle_magic` (Ars Magica - Definitive Edition (Core Rules).md:5073-5076, +5 gesture), `virtue.deft_form` (Ars Magica - Definitive Edition (Core Rules).md:3645-3648, Form-parameterized, waives both for that Form). Residuals clamp at 0.
- **Flat ability-total bonus (Concentration)** — `virtue.academic_concentration_subject` (Ars Magica - Definitive Edition (Core Rules).md:3362-3367)


### In-play effect variants (M5/5b) — implemented

Slice 5b adds 13 `Effect` variants (`types.rs`) representing all 18 in-play
families at full scope, and wires **all 93** `in_play_effect` V/F in
`rules/core/virtues_flaws.json` to them. Every variant is a **no-op** in
`effective.rs`/`validation/mod.rs`/`ruleset/integrity.rs` (asserted by
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
`ruleset::validate_effect_refs`. The **one-focus-per-magus** limit (Ars Magica - Definitive Edition (Core Rules).md:4542) is
a validation rule `validate_magical_focus` (issue code `multiple_magical_foci`,
Fluent `issue-multiple_magical_foci` in en/de) that **counts** the `MagicalFocus`
effect across selections + grants — so it also catches two Minor Foci with
distinct descriptors, which pairwise `incompatible_with` could not.

> `:4405` "A character can have only one Magical Focus, either major or minor,
> regardless of the source of the focus." (Restated at `:4542`.)

Because a magus may hold only one Magical Focus of **either** magnitude,
`virtue.major_magical_focus` and `virtue.minor_magical_focus` are marked mutually
`incompatible_with` in `rules/core/virtues_flaws.json` (Ars Magica - Definitive Edition (Core Rules).md:4405) — the same
dual-magnitude convention every `*_major`/`*_minor` V/F pair follows. That
convention is now enforced at load by `ruleset/integrity.rs`
`validate_magnitude_variant_exclusivity`: it detects variant pairs by shared stem
under both the `<stem>_major`/`<stem>_minor` **suffix** and the
`major_<stem>`/`minor_<stem>` **prefix** conventions (Magical Focus is the sole
prefix pair), and — only when **both** members exist, so a lone `*_major` or the
unrelated `virtue.minor_enchantments` is never flagged — fails loudly if the pair
is not mutually incompatible. This is the load-time complement to the selection-
time `validate_incompatibilities` (issue code `incompatible`); the effect-counting
`validate_magical_focus` above still additionally catches two Minor Foci with
distinct descriptors, which no `incompatible_with` pair can express.

In `enforced` mode the V/F picker also **prevents** the illegal combination up
front rather than only reporting it: `derive.ts` `incompatibleRefs` maps each
excluded id to the selected item responsible, and `VirtueFlawTab.svelte` greys
that Add row with the reason (`vf-blocked-incompatible`). It mirrors
`validate_incompatibilities` exactly — bought selections only, so a House grant
never blocks a pick — and stays inert in `advisory`/`silent`, where the pick
remains open and the `incompatible` issue does the reporting. It covers every
exclusion expressed through `incompatible_with` (magnitude pairs plus the
hand-authored cliques: Gentle vs Blatant Gift, Dwarf/Small Frame/Giant
Blood/Large, the four Mythic Companion status Virtues + The Gift). The
descriptor-blind two-Minor-Foci case is *not* blocked in the picker — it is not
an `incompatible_with` pair — and remains reported by `validate_magical_focus`.
E2E: `ui/e2e/specs/vf-incompatible.e2e.js`.

| Variant | Family / representative V/F | Source | 5i |
|---|---|---|---|
| `MagicalFocus { param(Text), major }` | Magical Focus — major/minor/mythic_blood | Ars Magica - Definitive Edition (Core Rules).md:4399-4422, 4536-4542, 4573-4589 | computed |
| `CastingTotalMod { amount, scope }` | Flat casting bonus/penalty — method_caster (+3 formulaic_ritual), poor_formulaic_magic (−5 formulaic), afflicted_tongue, cyclic_magic ±3, special_circumstances, ways_of_the_land, potent_magic | Ars Magica - Definitive Edition (Core Rules).md:4524-4527, 6610-6613, 5655-5658, 3635-3638, 5893-5896, 4998-5001, 5231-5234, 4740-4781 | computed (conditional ones toggled) |
| `LabTotalMod { amount }` | Flat lab bonus/penalty — adept_laboratory_student (+6), inventive_genius (+3), aristotelian_training (+1), creative_block (−3), weak_scholar (−6), the_constant_expression (−3), cyclic_magic, potent_magic | Ars Magica - Definitive Edition (Core Rules).md:3368-3371, 4151-4154, 3440-3443, 5873-5876, 7080-7083, 5821-5838, 4740-4781 | computed |
| `DeficientArt { param(Technique\|Form) }` | Art-halving — deficient_technique, deficient_form | Ars Magica - Definitive Edition (Core Rules).md:5913-5915, 5909-5912 | computed |
| `MagicTotalHalving { total }` | Halve spont casting / lab-enchant / lab-longevity / penetration / MR — weak_spontaneous_magic, weak_enchanter, difficult_longevity_ritual, weak_magic, flawed_parma_magica, weak_magic_resistance | Ars Magica - Definitive Edition (Core Rules).md:7084-7089, 7060-7063, 5962-5964, 7064-7067, 6142-6145, 7068-7071 | **computed** (round-2 audit finding GD3 closed the last gap): `spontaneous_casting`, `penetration`, `magic_resistance`, `lab_longevity` (since M5.5a), and now `lab_enchanting` too — folded into the new `LabTotal.enchanting` field in `derived/lab.rs::lab_totals` (Deficiency first, then this halving, per `:7060-7063`'s own stated order). Round 3 (G1) wired `enchanting` into `masterpiece_item_cap` too — the one remaining consumer of a Lab Total that used to read `total` instead — and into the frontend `LabTotal` type / `DerivedTotalsPanel` (G2) |
| `SoakMod { amount }` | Flat Soak — tough (+3), frail (−3), berserk (+2) | Ars Magica - Definitive Edition (Core Rules).md:5145-5147, 6190-6193, 3500-3503 | computed |
| `CombatMod { amount, target }` | Combat init/atk/def — berserk, hobbled, lame, missing_hand, missing_eye, poor_eyesight, palsied_hands, slow_reflexes, lightning_reflexes, fast_caster | Ars Magica - Definitive Edition (Core Rules).md:3500-3503, 6260-6263, 6330-6333, 6438-6441, 6434-6437, 6606-6609, 6578-6581, 6763-6766, 4311-4314, 3865-3868 | computed (conditional ones labelled) |
| `HealthMod { track, amount }` | Wound/fatigue penalty (enduring_constitution, low_tolerance), fatigue rolls (obese, short_of_breath, long_winded), casting-fatigue (painful_magic, vulnerable_casting, withstand_casting), recovery (fragile_constitution, rapid_convalescence) | Ars Magica - Definitive Edition (Core Rules).md:3751-3754, 6366-6369, 6516-6519, 6733-6736, 4327-4330, 6574-6577, 6993-7004, 5261-5282, 6186-6189, 4834-4837 | wound/fatigue computed; fatigue-roll/casting-fatigue/recovery surfaced |
| `MagicResistanceMod { kind }` | Non-halving MR — limited_magic_resistance (no_form_bonus), susceptibility faerie/infernal/divine, commanding_aura & special_circumstances (aura_bonus) | Ars Magica - Definitive Edition (Core Rules).md:6346-6349, 6819-6826, 6815-6818, 3579-3596 | **no_form_bonus computed** (folded into the flat per-Form MR number in `magic_resistance`); the four realm-conditional/situational kinds (aura_bonus, susceptible_divine/faerie/infernal) are surfaced as `ModifierFamily::MagicResistance` (amount 0) — they cannot be folded into the flat per-Form figure, so listing them keeps them from being silently dropped |
| `AgingMod { kind, amount }` | Aging/longevity — age_quickly, baneful_circumstances, monstrous_blood (−1), bee_king, faerie_blood (−1), magical_blood (−1), strong_faerie_blood (−3), unaging, bound_to_role, leprosy, poor_living_conditions, mild_aging, magian_lineage major/minor | Ars Magica - Definitive Edition (Core Rules).md:5659-5662, 5687-5690, 6454-6467, 3484-3499, 3797-3820, 4359-4372, 5032-5047, 5187-5190, 5735-5748, 6338-6341, 6618-6621, 4528-4531, 4339-4346 | **computed since M6/6b6**: `aging_roll` and `longevity_bonus` move the AGING TOTAL, `living_conditions` moves the modifier it subtracts, `no_apparent_aging` gates the apparent age and `no_aging` gates the Characteristic drop. Three items stay surfaced-only, each for a stated reason — age_quickly and baneful_circumstances (amount 0; schedule rules, not modifiers) and any `decrepitude` amount (no shipped item carries one). **The two immunities are separate tags**: bee_king carries `no_apparent_aging` alone, bound_to_role `no_aging` alone, unaging both — see **Aging (M6/6b6)** |
| `AdvancementMod { source, amount }` | Study/teaching — apt_student (+5 taught), book_learner (+3 book), free_study (+3 vis), good_teacher, independent_study, study_bonus, secondary_insight, unimaginative_learner, poor_student, incomprehensible, loose_magic | Ars Magica - Definitive Edition (Core Rules).md:3422-3425, 3519-3522, 3937-3940, 3971-3974, 4115-4118, 5056-5072, 4892-4895, 6915-6918, 6626-6628, 6294-6297, 6354-6357 | surfaced (app does not simulate advancement) |
| `SpecialCastingMod { kind, param? }` | Casting-style quirks — deft_form (Form-parameterized), quiet_magic, subtle_magic, diedne_magic, faerie_raised_magic, life_linked_spontaneous_magic, spell_improvisation, mercurian_magic, life_boost, leper_magus, and circumstantial halvings (deleterious_circumstances, environmental_magic_condition, short_ranged_magic, corrupted_spells, disjointed_magic) | Ars Magica - Definitive Edition (Core Rules).md:3645-3648, 4822-4826, 5073-5076, 9236-9245, 3675-3682, 3829-3842, 4299-4306, 5002-5005, 4514-4523, 4295-4298, 4249-4252, 5917-5920, 6020-6023, 6737-6740, 5859-5864, 5972-5975 | **deft_form/quiet_magic/subtle_magic computed** into per-cell `NonStandardCasting` (silent/still/silent_and_still); all other kinds surfaced (conditional penalties). `deft_form`'s `param` names the affected Form and is load-validated (`validate_effect_refs`, `ParameterDomain::Form` required) exactly as `DeficientArt`'s param, so a missing/wrong-domain key fails loudly instead of silently voiding the waiver in `in_play_mods` |
| `AbilityRollMod { param(Text), amount }` | Ability-roll bonus in a subject — academic_concentration_subject (+3) | Ars Magica - Definitive Edition (Core Rules).md:3362-3367 | surfaced |

**Modeling notes / accepted approximations** (each surfaced in 5i's labelled
read-out, so precision is not lost to the player): `weak_spontaneous_magic` maps
to `MagicTotalHalving { spontaneous_casting }`; the book rule is "you always
divide your Casting Score by five" (`:7084-7086`) — the Flaw removes the
fatiguing (exert-yourself, ÷2) option entirely rather than halving it a second
time. **Corrected in the round-2 audit (finding GD2)**: `derived/casting.rs`'s
`post()` previously applied a second `halve()` on top of the normal ÷2/÷5
split whenever this halving was present, silently producing ÷4/÷10 — neither
of which the rules state anywhere. It now reports the one rate the Flaw
actually leaves (÷5) at both the `spontaneous_fatiguing` and
`spontaneous_non_fatiguing` fields, since `CastingScores` has no "this option
does not exist" representation. Rank-dependent `commanding_aura` (MR
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

#### Supernatural Might & Magic Resistance (Realms of Power: Magic / The Infernal / The Divine)

A supernatural being has a **Might Score** aligned to one **Realm** (`Realm::{Magic,
Faerie, Divine, Infernal}`). The general rule:

> "Magic Might gives the character innate Magic Resistance equal to its Might
> Score, and this does not stack with other forms of resistance … they must use
> either their Parma or their Might for their base Magic Resistance."
> — *Ars Magica 5e - Realms of Power - Magic.md:1472* (general; also Ars Magica - Definitive Edition (Core Rules).md:2623-2631, :2627 "these
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
- `flaw.low_self_esteem` (Ars Magica - Definitive Edition (Core Rules).md:6362-6365) — Confidence (no score/points)
- `virtue.ferocity` (Ars Magica - Definitive Edition (Core Rules).md:3873-3876) — Confidence score 1 points 3

**Free nested V/F grant:**
- `virtue.devil_child` (Ars Magica 5e - Realms of Power - The Infernal.md:4144-4149) — grants free Minor Virtue
- `virtue.faerie_doctor` (Ars Magica 5e - Realms of Power - Faerie.md:6394-6399) — grants free Virtue (Dowsing)
- `virtue.nephilim` (Ars Magica 5e - Realms of Power - The Divine (Revised).md:3485-3513) — grants free Virtue

**Free starting Supernatural Ability score:**
- `virtue.animal_ken` (Ars Magica - Definitive Edition (Core Rules).md:3414-3417) — free starting Supernatural Ability score
- `virtue.corpse_magic` (Ars Magica - Definitive Edition (Core Rules).md:3605-3608) — free starting Supernatural Ability score
- `virtue.crafters_healing` (Ars Magica - Definitive Edition (Core Rules).md:3617-3620) — free starting Supernatural Ability score
- `virtue.embitterment` (Ars Magica - Definitive Edition (Core Rules).md:3739-3742) — free starting Supernatural Ability score
- `virtue.enchanting_ability` (Ars Magica - Definitive Edition (Core Rules).md:3747-3750) — free starting Supernatural Ability score
- `virtue.entrancement` (Ars Magica - Definitive Edition (Core Rules).md:3767-3770) — free starting Supernatural Ability score
- `virtue.font_of_knowledge` (Ars Magica - Definitive Edition (Core Rules).md:3921-3924) — free starting Supernatural Ability score
- `virtue.hex` (Ars Magica - Definitive Edition (Core Rules).md:4075-4078) — free starting Supernatural Ability score
- `virtue.induction` (Ars Magica - Definitive Edition (Core Rules).md:4119-4122) — free starting Supernatural Ability score
- `virtue.magic_sensitivity` (Ars Magica - Definitive Edition (Core Rules).md:4351-4354) — free starting Supernatural Ability score
- `virtue.persona` (Ars Magica - Definitive Edition (Core Rules).md:4710-4713) — free starting Supernatural Ability score
- `virtue.sense_passions` (Ars Magica - Definitive Edition (Core Rules).md:4930-4933) — free starting Supernatural Ability score
- `virtue.shapeshifter` (Ars Magica - Definitive Edition (Core Rules).md:4946-4949) — free starting Supernatural Ability score
- `virtue.spirit_votary` (Ars Magica 5e - Realms of Power - Magic.md:5480-5486) — grants free Virtue (Second Sight)
- `virtue.strong_faerie_blood` (Ars Magica - Definitive Edition (Core Rules).md:5032-5047) — free starting Second Sight ability
- `virtue.summon_animals` (Ars Magica - Definitive Edition (Core Rules).md:5085-5088) — free starting Supernatural Ability score
- `virtue.whistle_up_the_wind` (Ars Magica - Definitive Edition (Core Rules).md:5243-5246) — free starting Supernatural Ability score
- `virtue.wilderness_sense` (Ars Magica - Definitive Edition (Core Rules).md:5247-5250) — free starting Supernatural Ability score

**Item-level budget:**
- (Magic Items / Redcap wire `item_level_budget`; see slice 5e.)

**Might / power budget (wired, this slice):**
- `virtue.demonic_blood` (Ars Magica 5e - Realms of Power - The Infernal.md:4120, :4122) — `might_grant{infernal,5}` + `power_levels{30}`
- `virtue.demonic_might` (Ars Magica 5e - Realms of Power - The Infernal.md:4136) — `might_grant{infernal,2}` (adds +2)
- `virtue.demonic_powers` (Ars Magica 5e - Realms of Power - The Infernal.md:4142) — `power_levels{20}`
- `virtue.strong_angelic_heritage` (Ars Magica 5e - Realms of Power - The Divine (Revised).md:1975, :1977) — `might_grant{divine,0}` + `power_levels{30}` (Divine Might = age÷20 entered by hand; grant establishes Realm)

**Reputation grant:**
- `flaw.apostate` (Ars Magica - Definitive Edition (Core Rules).md:5675-5678) — reputation grant bad score 4
- `flaw.failed_journeyman` (Ars Magica - Definitive Edition (Core Rules).md:6060-6063) — reputation grant bad score 2
- `flaw.failed_master` (Ars Magica - Definitive Edition (Core Rules).md:6064-6067) — reputation grant bad score 4
- `flaw.failed_monk` (Ars Magica - Definitive Edition (Core Rules).md:6068-6071) — reputation grant poor score 2
- `flaw.failed_student` (Ars Magica - Definitive Edition (Core Rules).md:6072-6075) — reputation grant academic score 2
- `flaw.feral_scent` (Ars Magica - Definitive Edition (Core Rules).md:6106-6109) — reputation grant negative score 2
- `flaw.gabai` (Ars Magica - Definitive Edition (Core Rules).md:6198-6201) — reputation grant negative score 2
- `flaw.hedge_wizard` (Ars Magica - Definitive Edition (Core Rules).md:6240-6243) — reputation grant hermetic score 3
- `flaw.infamous_master` (Ars Magica - Definitive Edition (Core Rules).md:6314-6317) — reputation grant hermetic score 3
- `flaw.outlaw` (Ars Magica - Definitive Edition (Core Rules).md:6542-6545) — reputation grant score 2
- `flaw.outlaw_leader` (Ars Magica - Definitive Edition (Core Rules).md:6546-6549) — reputation grant score 3
- `flaw.outsider_major` (Ars Magica - Definitive Edition (Core Rules).md:6550-6561) — reputation grant bad score 1-3
- `flaw.outsider_minor` (Ars Magica - Definitive Edition (Core Rules).md:6550-6561) — reputation grant bad score 1-3
- `flaw.usurer` (Ars Magica - Definitive Edition (Core Rules).md:6951-6954) — reputation grant poor score 4
- `virtue.baccalaureus` (Ars Magica - Definitive Edition (Core Rules).md:3470-3475) — XP grant 90 + reputation grant academic score 1
- `virtue.cathedral_school_master` (Ars Magica - Definitive Edition (Core Rules).md:3549-3554) — XP grant 240 + reputation grant academic score 2
- `virtue.doctor_in_faculty` (Ars Magica - Definitive Edition (Core Rules).md:3683-3698) — XP grant 300 + reputation grant academic score 3
- `virtue.famous` (Ars Magica - Definitive Edition (Core Rules).md:3861-3864) — reputation grant player-chosen score 4
- `virtue.hermetic_prestige` (Ars Magica - Definitive Edition (Core Rules).md:4071-4073) — reputation grant hermetic score 4
- `virtue.lone_redcap` (Ars Magica - Definitive Edition (Core Rules).md:4319-4326) — XP grant 300 + grants Virtue + reputation grant poor score 2
- `virtue.magister_in_artibus` (Ars Magica - Definitive Edition (Core Rules).md:4385-4394) — XP grant 240 + reputation grant academic score 2
- `virtue.magister_in_medicina` (Ars Magica - Definitive Edition (Core Rules).md:4395-4398) — XP grant 300 + reputation grant academic score 3
- `virtue.master_bard` (Ars Magica - Definitive Edition (Core Rules).md:4457-4462) — XP grant 240 + reputation grant local score 3
- `virtue.physician_of_salerno` (Ars Magica - Definitive Edition (Core Rules).md:4732-4735) — XP grant 50 + reputation 2
- `virtue.rard` (Ars Magica - Definitive Edition (Core Rules).md:3476-3479) — reputation grant local score 1
- `virtue.rosh_beth_din` (Ars Magica - Definitive Edition (Core Rules).md:4878-4883) — XP grant 50 + reputation grant good score 2 + grants Virtue
- `virtue.senior_bard` (Ars Magica - Definitive Edition (Core Rules).md:4904-4909) — XP grant 90 + reputation grant local score 2
- `virtue.senior_clergy` (Ars Magica - Definitive Edition (Core Rules).md:4910-4921) — reputation grant score 4
- `virtue.templar_office_holder` (Ars Magica - Definitive Edition (Core Rules).md:5121-5124) — reputation grant score 2

**Size/characteristic delta:**
- `virtue.blood_of_the_nephilim` (Ars Magica 5e - Realms of Power - The Divine (Revised).md:1941-1954) — size delta + Dominion Lore

**True Faith score:**
- `virtue.powerful_relic` (Ars Magica - Definitive Edition (Core Rules).md:4782-4787) — True Faith score 3
- `virtue.relic` (Ars Magica - Definitive Edition (Core Rules).md:4852-4855) — True Faith score 1

**XP grant:**
- `flaw.corrupted_arts` (Ars Magica - Definitive Edition (Core Rules).md:5853-5858) — grants XP swing at creation + situational casting
- `flaw.feral_upbringing` (Ars Magica - Definitive Edition (Core Rules).md:6110-6113) — XP grant 120
- `flaw.savantism` (Ars Magica - Definitive Edition (Core Rules).md:6703-6708) — halves starting XP
- `virtue.arcane_lore` (Ars Magica - Definitive Edition (Core Rules).md:3430-3435) — XP grant 50
- `virtue.clan_ilfetu` (Ars Magica - Definitive Edition (Core Rules).md:3563-3566) — XP grant 50
- `virtue.craft_guild_training` (Ars Magica - Definitive Edition (Core Rules).md:3613-3616) — XP grant 50
- `virtue.elemental_magic` (Ars Magica - Definitive Edition (Core Rules).md:3731-3738) — Art XP distribution at creation (implemented, slice 5c: `Effect::ElementalMagic` XP-space Art boost — see section above)
- `virtue.falconer` (Ars Magica - Definitive Edition (Core Rules).md:3847-3852) — XP grant 50
- `virtue.forge_companion` (Ars Magica - Definitive Edition (Core Rules).md:3925-3928) — XP grant 50
- `virtue.hermetic_experience` (Ars Magica - Definitive Edition (Core Rules).md:4063-4066) — XP grant 50
- `virtue.ineslemen` (Ars Magica - Definitive Edition (Core Rules).md:4123-4126) — XP grant 50 + grants Minor Flaw
- `virtue.marshal` (Ars Magica - Definitive Edition (Core Rules).md:4449-4456) — XP grant 50
- `virtue.master_of_kennels` (Ars Magica - Definitive Edition (Core Rules).md:4467-4470) — XP grant 50
- `virtue.mentored_by_demons` (Ars Magica - Definitive Edition (Core Rules).md:4496-4499) — XP grant 50
- `virtue.schooled_in_crime` (Ars Magica - Definitive Edition (Core Rules).md:4884-4887) — XP grant 50
- `virtue.shadchan` (Ars Magica - Definitive Edition (Core Rules).md:4934-4939) — XP grant 50
- `virtue.simple_student` (Ars Magica - Definitive Edition (Core Rules).md:4958-4963) — XP grant 30/yr
- `virtue.trained_assassin` (Ars Magica - Definitive Edition (Core Rules).md:5153-5156) — XP grant 50
- `virtue.venditor` (Ars Magica - Definitive Edition (Core Rules).md:5207-5210) — XP grant 50


## Life-stage experience (M6/6b2) — `life_stage.rs`

Abilities are bought with experience earned in blocks, not from one bank:

> Abilities represent a character's learned abilities. For grogs and companions
> they are acquired in two blocks: early childhood, and later life. For magi, there
> are two more periods to consider: apprenticeship, and life as a magus after that.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2364`.
- Implementation: `crates/arm-rules/src/life_stage.rs`. **All four periods are
  modelled**: early childhood, later life, apprenticeship (**M6/6b4**), and life as a
  magus after the Gauntlet — `:2216`/`:2471`'s 30 points per year (**M6/6b5**). Each
  has a section below.
- **So a magus may be built through its life stages, at its Gauntlet or long past
  it.** Its later life runs only "until apprenticeship" (`:2214`), so
  `later_life_years` subtracts the apprenticeship span as well as childhood, and those
  pre-apprenticeship years become a restricted, Abilities-only pool; apprenticeship
  and the years after the Gauntlet both fund the **general** pool, since only that
  pool may buy an Art. See **Apprenticeship — 240 experience points across Arts and
  Abilities**, **Life as a magus after the Gauntlet — 30 points per year** and
  **Pre-apprenticeship experience buys Abilities only** below for all of it.
- The 6b2-era refusal `life_stage_magus_guided_unsupported` is **gone** — code, contract
  row and both Fluent messages. It said the engine modelled two of four periods, which
  is no longer true. What remains of the limit moved to where it belongs: a ruleset
  declaring magi must declare their apprenticeship, checked at **load**
  (`Ruleset::validate_apprenticeship_refs`), because a missing block is missing data,
  not a property of a character. The guided funding mode is therefore offered to magi
  too; `ui/src/lib/components/LifeStagePanel.svelte` no longer disables that radio.

#### Early childhood — 75 + 45, and the closed spread list

> In the first five years of life, characters gain 75 experience points in their
> native language (see page 167 for the Language Ability), which normally gives them
> a score of 5, and 45 experience points to divide between Area Lore (for the place
> or places the character is growing up), Athletics, Awareness, Brawl, Charm, Folk
> Ken, Guile, Living Language (other than the character's native language), Stealth,
> Survival, and Swim. You do not need to put points into all of these Abilities;
> choose the ones that best fit your conception of the character.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2378`.
- Data: `rules/core/life_stages.json` → `childhood`: `years` 5,
  `native_language_xp` 75, `spread_xp` 45, and `spread_abilities` (the eleven ids
  the passage names). `native_language_ability` names the ability the 75 buys
  (`ability.living_language`) as data rather than a hardcoded slug.
- Implementation: `life_stage.rs` — `ChildhoodRules`, and
  `LifeStageRules::budget`, which reports the two blocks separately because they
  fund different things. They become two **restricted pools** in the existing
  allocation solve (`effective/xp.rs` — `xp_allocation`): the native block funds one
  ability *instance* (the chosen language), the spread funds the eleven-ability list
  **excluding** that instance — the passage's "Living Language (other than the
  character's native language)". Unspent childhood experience is wasted, which the
  pre-existing `restricted_xp_unspent` warning reports — **naming the block**: it
  carries `origin_kind` (`item` | `life_stage`) and `origin` (the granting item's id,
  or the block slug from `LifeStageBlock`'s `Display`), read off
  `RestrictedXpPool::origin` by `validate_xp_pool` (`validation/magus.rs`). Without
  that pair a life-stage character gets two warnings differing only in their numbers
  — 75 unspent and 45 unspent — and neither says which block to go and spend. The
  engine emits machine names only, never a Fluent key or a label: the frontend
  resolves an `item` through the ruleset's i18n and a `life_stage` through its own
  `xp-pool-<block>` catalogue.
- **Childhood is granted unconditionally, so it does not wait for an age.** `:2378`
  gives the 75 + 45 "in the first five years of life" with no further condition;
  only later life is counted in years up to an age (`:2392`). `budget` therefore
  returns both childhood blocks with `later_life_years: 0` for a plan whose age is
  not yet set, and returns `None` only when there is no plan at all. The missing age
  is a finding in its own right — `life_stage_age_unset` (error, `experience`, no
  args) from `validate_life_stage_plan` — rather than the silent absence of a
  budget: with no budget the two childhood pools would stand at 0 and every
  childhood row would be reported as unfunded, which blames the player for rows the
  guided flow itself writes before an age is typed.
- Load-time referential integrity (not a sourced rule): every `spread_abilities` id
  and the `native_language_ability` must resolve, and the latter must be a
  *parameterized* ability — one language among many cannot be named otherwise.
  Checked by `Ruleset::validate_childhood_refs` (`ruleset/integrity.rs`), called from
  `validate_integrity` beside `validate_childhood_packages` — so it also runs on
  `from_serialized`, and a cached ruleset is trusted no further than a freshly
  parsed one.

#### Sample Childhood packages (M6/6b3a) — `childhood.rs`

> The following Ability packages can be taken to speed up character generation.
> Each represents a particular sort of childhood. Note that you can spend the 45
> experience points for yourself, as well.
>
> - Athletic Childhood: Athletics 2, Brawl 2, Native Language 5, Swim 2
> - Exploring Childhood: Area Lore 2, Athletics 1, Awareness 1, Native Language 5, Stealth 1, Survival 2
> - Mischievous Childhood: Brawl 2, Guile 2, Native Language 5, Stealth 2
> - Social Childhood: Charm 2, Folk Ken 2, Guile 2, Native Language 5
> - Traveling Childhood: Area A Lore 1, Area B Lore 1, Folk Ken 2, Living Language 1, Native Language 5, Survival 2

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2380-2388` (heading
  `:2380`, the "can be taken … for yourself, as well" sentence `:2382`, the five
  packages `:2384-2388`, one per line). Pricing off the Ability advancement table
  at `:2406-2427` (the shared "ABILITY To Buy" column, whose own provenance is the
  **Abilities** section above).
- Data: `rules/core/childhoods.json` — all five packages of `:2384-2388`, one
  object per rulebook line, each carrying that single line as its `source`
  (`[2384, 2384]` … `[2388, 2388]`). Packages id-sorted (`childhood.athletic`,
  `.exploring`, `.mischievous`, `.social`, `.traveling`), each package's entries
  sorted by `(ability, slot)`; an absent slot sorts first, which is what puts
  Traveling's native `Living Language 5` ahead of its slotted `Living Language 1`.
  Names in `rules/i18n/en/childhoods.json` and `rules/i18n/de/childhoods.json` —
  `name` only, no `description`: the entry list is mechanics, and a UI composes the
  human-readable spread from this file plus the Ability i18n, so no rules text is
  duplicated. The German names are read verbatim off the German mirror
  `Ars Magica Definitive Edition Basisregeln.md:2384-2388` — Athletische Kindheit,
  Forschende Kindheit, Mutwillige Kindheit, Soziale Kindheit, Reisende Kindheit.
  No translation table carries a "Childhood" entry, so the rulebook line **is** the
  authority here (the mirror is line-for-line, so `:2384` is the same package in
  both languages).
  The app reads the file in `load_ruleset_from_dir` (`crates/arm-app/src/ruleset_io.rs`)
  alongside the other `core/*.json`, with the same "an empty file means the ruleset
  ships none" idiom `life_stages.json` uses — so a ruleset may legitimately offer no
  packages, per `:2382`. `childhoods.json` is the ninth `i18n/<lang>/` file the
  localized ruleset merges, and it is required in every language's directory.
- Implementation: `crates/arm-rules/src/childhood.rs` — `ChildhoodPackage` /
  `ChildhoodEntry`, with `spread_xp` and `native_xp` pricing a package against
  `AdvancementTable::xp_for_score`, and `slots` naming the parameters a UI must
  ask for.
- **A package is a shortcut, never a restriction.** `:2382` says the packages
  *can* be taken and explicitly keeps hand-spending open ("you can spend the 45
  experience points for yourself, as well"), so no character type requires one and
  taking none is not a validation issue. What a package may buy is unchanged: the
  closed eleven-ability spread list of `:2378` above.
- **Every package prices to exactly 45 + 75.** Verified against the advancement
  table for all five: spreads 15+15+15 (Athletic), 15+5+5+5+15 (Exploring),
  15+15+15 (Mischievous), 15+15+15 (Social), 5+5+15+5+15 (Traveling) = **45**
  each, and every `Native Language 5` = **75**. That identity is why a package
  needs no budget of its own — it is one way of spending the two childhood blocks
  `:2378` already grants, so `spread_xp`/`native_xp` exist to *check* the shipped
  data against the blocks rather than to fund anything. Funding stays with the
  restricted 45/75 pools in `effective/xp.rs` (`xp_allocation`).
- **The 45/75 arithmetic is enforced at load, as the transcription trust gate.**
  `Ruleset::validate_childhood_packages` (`ruleset/integrity.rs`, called from
  `validate_integrity`, so it also runs on `from_serialized`) re-prices every
  shipped package off the advancement table and rejects the ruleset unless the
  spread equals `childhood.spread_xp` and the native entry equals
  `childhood.native_language_xp` — the numbers come from
  `rules/core/life_stages.json`, never from a constant in code. A score with no
  row in the table is reported as unpriceable rather than costed at 0. This is
  what keeps a hand-transcribed catalogue honest: a mistyped "Athletics 1" fails
  the load instead of shipping a package that quietly costs 35 rather than 45.
  The shipped catalogue is additionally gated from *outside* that arithmetic by
  `every_shipped_childhood_package_prices_to_45_and_75`
  (`crates/arm-rules/tests/data_integrity.rs`), which re-prices every package
  against the **literals** 45 and 75. The load-time check compares against
  `life_stages.json`, so a coordinated edit lowering the block and a package
  together would pass it in silence; the literals come from the rulebook line
  instead, and are the reason a transcription slip cannot ship. Two further data
  gates sit beside it: `known_childhood_packages_ship_with_their_entries_and_provenance`
  pins Athletic's four entries and Traveling's three slots (structure, never a
  package total — catalogue size is data), and
  `the_shipped_childhoods_file_is_canonically_ordered` checks the file's own
  `(ability, slot)` ordering, which nothing else can: entry order is deliberately
  preserved on load, so a mis-sorted file parses and validates happily.
  Twelve load-time rules guard this data in all: the two sum checks above, the
  unpriceable-score report above, and these nine — every entry's ability
  resolves; a parameterized ability carries a `slot` (unless it is the native
  language, chosen once per character) and a plain one does not; slots are unique
  within a package; there is exactly **one** `native` entry (which is what makes
  `native_entry`'s "at most one" a guarantee) and it names
  `childhood.native_language_ability`; every non-native entry is on the closed
  `:2378` spread list; packages shipped without life-stage rules are rejected,
  since nothing could price them; and `source` ranges are not inverted. All
  errors accumulate, so a broken file reports every problem at once.
- Engine reading where the text is terse: "Native Language 5" names no language,
  because the language is the character's own choice — so the entry carries a
  `native` flag and the chosen language lives in `LifeStagePlan::native_language`.
  Likewise "Area A Lore / Area B Lore" are two instances of one parameterized
  Ability, distinguished by an entry `slot` key (`area_a`, `area_b`) rather than by
  id; Traveling's plain "Living Language 1" is the spread's second language, the
  `:2378` "other than the character's native language", and is told apart from the
  native entry by the flag alone.
- Entry order is preserved on load rather than re-sorted (unlike the advancement
  table, whose order is unobservable): it is the order `slots()` asks the player
  for parameters in. Canonical `(ability, slot)` ordering is therefore a property
  of the shipped file, checked where the file is loaded.
- **The chosen package is stored, but only as a record.**
  `LifeStagePlan::childhood_package` (`life_stage.rs`) keeps the id the player took,
  so a save says which childhood the character had. Nothing is derived from it: the
  Abilities the package grants are ordinary bought `ability_scores` rows, funded by
  the same restricted 45/75 pools a hand-divided childhood uses. It is also
  deliberately **not** cross-checked against those rows — `:2382` ("Note that you can
  spend the 45 experience points for yourself, as well") leaves a taken package open
  to adjustment, so scores that no longer match the package are legal rather than an
  error. The parameterized slot values are not stored either: they persist as the
  rows' own `parameter` values, and a second copy could only diverge from them. The
  field is additive (`serde(default, skip_serializing_if)`), so `SCHEMA_VERSION`
  stays 14 and no migration is needed.
- **A stored package does not narrow the 45-point pool.** Taking one leaves the
  spread pool's eligibility exactly as `:2378` sets it — the closed eleven-ability
  list — rather than restricting it to the abilities the package names. No passage
  forbids the other eight once a package is taken, and `:2382` invites precisely
  that adjustment, so narrowing would be a rule the rulebook does not state.
  `effective/xp.rs::xp_allocation` therefore never reads
  `LifeStagePlan::childhood_package`; the pool it builds is identical whether a
  package was taken or the 45 points were divided by hand.
- **Applying a package is a monotone raise.** `childhood::apply_package`
  (`childhood.rs`) returns a **new** entity whose Ability rows are each brought to
  `max(existing, entry.score)`, keyed by `(ability, parameter)` — so a score bought
  from later life is never lowered by taking a package, a row the package does not
  name is never removed, and an existing row keeps its specialty. Two properties
  follow, both pinned by tests: the application is **idempotent** (the same package
  with the same slot values applied twice yields an identical entity, so a double
  click cannot charge twice), and a rejected application leaves the caller's
  character untouched, since nothing is mutated in place. The rows it writes are
  ordinary bought rows, indistinguishable from a hand-divided childhood, which is
  why nothing downstream needs to know a package was involved; the id is recorded in
  `LifeStagePlan::childhood_package` purely as the annotation described above.
  **Funding is deliberately not checked at application time** — the restricted 45/75
  pools in `effective/xp.rs` (`xp_allocation`) already price the rows against the
  blocks, so an overspend surfaces as `not_enough_xp` exactly as a hand-typed one
  would, and charging here as well would double-count.
- **A slot value is just the row's `parameter`.** The player's answer for an entry's
  `slot` (`area_a`, `area_b`, `language`) is written straight into the row's
  `parameter`, trimmed of surrounding whitespace, so Traveling's two Area Lore slots
  become two distinct rows and its spread `Living Language 1` sits beside the native
  `Living Language 5` — exactly the shape hand-buying two instances of one
  parameterized Ability produces. A slot that is missing, or blank once trimmed, is
  **unanswered** rather than answered with an empty string, since `Area Lore ()` is
  a row no one asked for.
- **The spread's second language may not be the native one.** `:2378` lists what the
  45 points buy as "Living Language (other than the character's **native
  language**)", so a language slot answered with the plan's own native language is
  rejected (`ChildhoodRejection::SlotIsNativeLanguage`). The check applies to the
  childhood's `native_language_ability` alone — an `Area Lore (German)` beside a
  native language of German is ordinary, not a violation.
- **Two slots may not share a value.** Rows merge by `(ability, parameter)`, so two
  Area Lore slots both answered "Bavaria" would collapse into **one** row at score 1:
  `duplicate_ability` (`validation/scores.rs`) would never fire, and 40 of the
  spread's 45 experience points would vanish into an anonymous
  `restricted_xp_unspent` warning. `ChildhoodRejection::DuplicateSlotValue` names
  both slots so the UI can point at the colliding field.
- **Rejections are collected, not short-circuited.** `apply_package` reports every
  bad field in one pass (`ChildhoodRejection`: unknown package, native language
  unset, slot unfilled, slot is the native language, duplicate slot value), so a
  player fixing a package fills in all of it at once. The rejections are plain
  data — no issue codes, no Fluent keys, no user-facing prose; mapping them onto
  localized `ValidationIssue`s happens in `validation/`, where the emit sites stay
  visible to the contract-table and phase scanners.
- **The slot codes are command-input findings, not `validate()` findings.**
  `validation::childhood_rejection_issues` (`validation/life_stage.rs`) turns the
  rejections into `childhood_slot_unfilled`, `childhood_slot_is_native_language`
  and `childhood_slot_duplicate_value` (all error / `experience`). They are emitted
  only there, never by `validate()`: applying a package is all-or-nothing, so a
  stored character cannot *hold* an unanswered or colliding slot — the rejection
  describes the form the player just submitted and is gone once it is corrected.
  Each carries the Ability plus the `key` its field label comes from — the
  Ability's own `parameter` key (`area`, `language`), looked up in the ruleset since
  the rejection carries only the id — while `slot`/`other_slot` travel for **field
  targeting only** and are never interpolated into a message, a slot key being a
  slug. `ChildhoodRejection::NativeLanguageUnset` reuses the plan's existing
  `life_stage_native_language_unset` rather than inventing a second code for the
  same fact.
- **A stored package id is a reference, so `validate()` checks it resolves.**
  `validate_life_stage_plan` reports `childhood_package_unknown` (error,
  `experience`, arg `package`) when `LifeStagePlan::childhood_package` names an id
  the loaded ruleset does not ship — which a save written against another ruleset
  can. This is the *only* thing checked about the recorded package: what it granted
  stays uncross-checked, per `:2382` above.
- **The application crosses IPC as a command.** `apply_childhood_package`
  (`crates/arm-app/src/commands.rs`) takes the entity, the package id and the slot
  values and returns `ChildhoodApplication`
  (`ruleset_io::apply_childhood_package_loaded`): either the character the entity
  becomes, or the rejections as localizable `ValidationIssue`s. A rejection is an
  ordinary `Ok` outcome, not an `AppError` — an unanswered slot is a finding about
  the form the player submitted, not a failed command — and no English prose crosses
  the boundary, since the frontend renders the issues through the `issue-<code>`
  Fluent path it already has.
- **The UI that offers the packages shipped in M6/6b3b:** the funding-mode toggle
  and life-stage panel (`ui/src/lib/components/LifeStagePanel.svelte`) and the
  package picker with one field per slot
  (`ChildhoodPackagePicker.svelte` → `store.applyChildhoodPackage`). It keeps the
  split above intact: the *drafted* package and its slot answers are UI state
  (`store.childhoodDraft`), never on the entity, while the package actually **taken**
  is the persisted `LifeStagePlan::childhood_package` record — so the picker's select
  starts unselected even for a character that has one recorded, since the answers
  live on the Ability rows and a recorded package is history rather than a form to
  re-open. Leaving the guided funding mode deletes the plan, which deletes the record
  and prunes the draft with it.

#### Later life — 15 experience points per year

> After early childhood, the character gains 15 experience points per year, which
> may be placed in any Abilities, as long as the character has a Virtue that permits
> her to learn those Abilities. Academic, Arcane, and Martial Abilities require a
> Virtue, as do Supernatural Abilities.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2392`.
- Data: `rules/core/life_stages.json` → `later_life.xp_per_year` 15.
- Implementation: `life_stage.rs` — `LaterLifeRules`, `later_life_years`
  (stop age − childhood years − apprenticeship years, where the stop age is the
  character's own age, or a magus's Gauntlet age) and `budget`. **For a grog or
  companion later life is the general pool**, since it funds anything the character
  may learn. For a magus it is neither the whole span nor the general pool — see
  **Pre-apprenticeship experience buys Abilities only** below.
- The Virtue requirement this passage restates for Academic/Arcane/Martial
  Abilities (`:2315` names Educated / Arcane Lore / Warrior) **landed in M6/6b2b** —
  see **Access to Academic / Arcane / Martial Abilities** below. Supernatural keeps
  its own stricter check (`supernatural_ability_requires_virtue`).

#### Apprenticeship — 240 experience points across Arts and Abilities (M6/6b4)

> The fifteen years of apprenticeship give the character 240 experience points, and
> 120 levels of spells. These experience points can be spent on Arts or Abilities,
> including Arcane, Academic, and Martial Abilities. Note that magi can only spend
> experience points on Arcane, Academic and Martial Abilities before apprenticeship
> if they have a Virtue which allows them to do so. A sensible division is to spend
> 120 experience points on Abilities and 120 on Arts.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2435` (heading
  `#### Magus Only — Apprenticeship` at `:2433`), the four periods at `:2364` ("For
  magi, there are two more periods to consider: apprenticeship, and life as a magus
  after that"), and the Darius example at `:2439-2449` — whose master "picks 10 as a
  nice, round number" for the start of apprenticeship (`:2402`) and who spends "his
  last 5 exp on Parma Magica 1" "just before Gauntlet" (`:2449`).
- Data: `rules/core/life_stages.json` → `apprenticeship`: `years` 15, `xp` 240, plus
  the two Ability lists (below). Canonically sorted, so `apprenticeship` leads the
  file.
- Implementation: `life_stage.rs` — `ApprenticeshipRules`,
  `LifeStageRules::apprenticeship_of` (the block for a magus, `None` for anyone else,
  read off the profile's `is_magus` flag) and `budget`, which reports
  `apprenticeship_years` / `apprenticeship_xp` as a fourth block. `total()` sums all
  four.
- **The 120 spell levels are deliberately NOT here.** They ship as
  `EntityTypeProfile.spell_levels: 120` on the magus profile
  (`rules/core/character_types.json`), whose single selector is
  `effective::spell_levels_base` — see **Spell-levels budget — 120 at creation** in
  the *Hermetic spells* section above, which cites this same `:2435`. The two numbers
  of one sentence live in two files on purpose: the spell budget belongs to the
  character *type* (a per-character `spell_levels_override` may replace it), while the
  240 belongs to the life-stage block. Duplicating either would create a second place
  to edit it.
- **Apprenticeship is the general pool.** Two independent facts force it. (a) `:2435`
  lets this experience buy "Arts or Abilities", and only the general pool may fund an
  Art — `pool_covers` returns false for every `(Ability pool, Art spend)` pair
  (`effective/xp.rs`). (b) `Effect::GeneralXp` **already means apprenticeship
  experience**: Skilled Parens grants "an additional 60 experience points … during
  apprenticeship" (`:4966`) and Weak Parens "60 fewer … from apprenticeship"
  (`:7074`), and `general_xp_bonus` is applied to the general pool alone. Assigning
  apprenticeship anywhere else would move that ±60 onto a child's money.
  `xp_allocation` therefore selects `base_general` in exactly one place:
  `apprenticeship_xp + post_gauntlet_xp` for a magus (the years after the Gauntlet buy
  Arts too, `:2471` — see **Life as a magus after the Gauntlet** below), later life for
  anyone else, the typed `xp_pool` without a plan.
- **No `general_xp` field on `LifeStageBudget`.** The pool the solve funds from is
  base + bonus (240 at the Gauntlet, 300 with Skilled Parens, more once the
  post-Gauntlet experience joins it); a budget field holding only the base
  would disagree with it and would have to be excluded from `total()`. The real pool
  is surfaced instead, as `EffectiveScores.xp_general_pool` (`arm-app/ruleset_io.rs`).
- **`apprenticeship_start_age` is not stored.** Apprenticeship is fifteen fixed years,
  so the span before it follows from the age the character was gauntleted at:
  `later_life_years = gauntlet_age − childhood.years − apprenticeship.years`. Verified
  against `:2402`, where a boy apprenticed at 10 has "75 experience points to spend from
  those five years" — exactly what a magus gauntleted at 25 earns here, whatever its age
  now. In M6/6b4 the Gauntlet age *was* the age, because a magus was generated standing
  at its Gauntlet; **M6/6b5** stores it as `LifeStagePlan::gauntlet_age` (absent = at the
  Gauntlet, so the numbers are unchanged) and counts the years after it separately — see
  the next section. `SCHEMA_VERSION` is unchanged (14) throughout and no save migrates.
- Load-time gate (an engine invariant, not a sourced rule): a ruleset that declares an
  `is_magus` profile **and** ships life-stage rules must declare an apprenticeship
  block — `Ruleset::validate_apprenticeship_refs`, gated exactly like
  `validate_engine_required_roles`. Without the block such a ruleset would cost a
  magus as a companion, counting every year to its age. **This replaces the 6b2
  runtime refusal** (`life_stage_magus_guided_unsupported`, now deleted): the limit was
  never a property of a character, it was missing data.

#### Life as a magus after the Gauntlet — 30 points per year (M6/6b5)

> 8. **Hermetic Magi Only (Optional):** Years after apprenticeship. Divide 30 points
> per year between experience points in Arts, experience points in Abilities, and
> levels of spells.

> For every year, the magus gets 30 points. Each point can be an experience point in an
> Art or Ability or one level of spell. The maximum spell level a magus may know is
> limited as before.

> For each season that your magus spends working on a lab project, the character loses
> 10 points from the yearly 30 experience points, to a minimum of 0 if three or four
> seasons are spent on lab work. Thus it is most cost effective to have the magus engage
> in a full year of lab work at a time.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2216` (the character-creation
  step), `:2471` (the rate) and `:2482` (lab work), under the heading
  `#### Magus Only — After Apprenticeship` at `:2467` — the fourth of the periods
  `:2364` names: "apprenticeship, and life as a magus after that".
- Data: `rules/core/life_stages.json` → `post_apprenticeship`: `points_per_year` 30
  (`:2471`), `lab_season_cost` 10 and `max_charged_lab_seasons_per_year` 3 (`:2482`).
  Canonically sorted, so the block closes the file after `later_life`.
- Implementation: `life_stage.rs` — `PostApprenticeshipRules`, carried by
  `LifeStageRules::post_apprenticeship` as an additive `Option` with
  `skip_serializing_if`, exactly like `apprenticeship`: `:2364` calls these "two
  **more** periods", so a ruleset with no Hermetic magi ships neither block and writes
  neither key. The player's three choices live on `LifeStagePlan` (`gauntlet_age`,
  `post_gauntlet_lab_seasons`, `post_gauntlet_spell_levels`), the derived figures on
  `LifeStageBudget` (`gauntlet_age`, `post_gauntlet_years`, `post_gauntlet_points`,
  `post_gauntlet_spell_levels`, `post_gauntlet_xp`).
- **Points, not experience points.** `:2471` makes each point fungible — "an experience
  point in an Art or Ability or one level of spell" — and the player decides which each
  one becomes. Hence `points_per_year` where apprenticeship says `xp`: `:2435` grants
  its 240 experience and 120 spell levels as two separate, non-interchangeable numbers,
  which is why those two live in two files (above) while this one number does not split.
- **Transcription trust gate:** `lab_season_cost × max_charged_lab_seasons_per_year`
  must equal `points_per_year`, and `lab_season_cost` may not be 0 —
  `Ruleset::validate_post_apprenticeship_rules`. The identity is not tidiness, it **is**
  `:2482`: the deduction runs "to a minimum of 0 if three or four seasons are spent", so
  three seasons at 10 have to cancel the yearly 30 exactly. A year that overshot would
  have lab work take points it never granted; one that fell short would still pay a
  magus who spent the whole year in the lab. Same idiom as re-pricing the
  apprenticeship's `recommended_xp` off the advancement table. The **fourth** season is
  free because `:2482` has already reached 0 by the third — hence `max_charged`, not a
  cap on seasons.
- Load-time gate (an engine invariant, not a sourced rule): a ruleset that declares an
  `is_magus` profile **and** ships life-stage rules must declare the
  `post_apprenticeship` block, exactly as it must declare `apprenticeship`. That block
  already ends a magus's later life at its Gauntlet age; without this one the years
  after the Gauntlet would grant nothing back, so a magus would simply lose them. Both
  checks run from `validate_integrity`, so a cached ruleset returning through
  `Ruleset::from_serialized` is trusted no further than a freshly parsed one.
- The arithmetic, in `LifeStageRules::budget` and `LifeStageRules::post_gauntlet_points`:

  ```text
  gauntlet_age        = min(plan.gauntlet_age ?? age, age)     # magi only
  later_life_years    = gauntlet_age − childhood.years − apprenticeship.years
  post_gauntlet_years = age − gauntlet_age
  charged_seasons     = min(plan.post_gauntlet_lab_seasons,
                            max_charged_lab_seasons_per_year × post_gauntlet_years)
  post_gauntlet_points      = post_gauntlet_years × points_per_year
                              − charged_seasons × lab_season_cost
  post_gauntlet_spell_levels = min(plan.post_gauntlet_spell_levels, post_gauntlet_points)
  post_gauntlet_xp           = post_gauntlet_points − post_gauntlet_spell_levels
  ```

  Every step saturates, so no stored value can underflow a figure; a stored total that
  the years cannot pay for is a validation finding, not a negative budget.
  `LifeStageBudget::total()` adds **`post_gauntlet_xp` only** — a level of spell is not
  experience (`:2471` has the player split the points), and folding it in would spend it
  twice, once here and once against the spell-levels budget.
- **`gauntlet_age` is the one number stored, and its absence means the magus stands at
  its Gauntlet.** `age = childhood + later life + apprenticeship + post-Gauntlet` is one
  equation in two unknowns, so exactly one of them has to be recorded and every other
  figure derived from it. Three consequences, all deliberate:
  - **Absent = at the Gauntlet.** The Gauntlet age is then `Entity::age` itself, which
    is exactly what M6/6b4 computed, so every save written before the field keeps its
    numbers, the field is purely additive and `SCHEMA_VERSION` stays 14.
  - **`post_gauntlet_years` was rejected as the stored number.** With the years stored
    instead, raising a magus's age would stretch the *childhood-to-apprenticeship* span
    — the years before it was taken as an apprentice — rather than its life as a magus,
    which is the opposite of what a player raising the age means. Hence
    `later_life_years(stop_age, apprenticeship_years)` is fed the **Gauntlet** age,
    which for anyone serving no apprenticeship is the age itself.
  - **Read for a magus only, and clamped to the age.** `:2216` is "**Hermetic Magi Only
    (Optional):** Years after apprenticeship", so the stored age is read for a character
    whose profile serves an apprenticeship and ignored on any other plan. Gating on the
    *points* being zero instead would let a hand-edited companion plan carrying
    `gauntlet_age: 25` at age 60 silently lose 35 later-life years (525 experience
    points). It is clamped to the age because Advisory and Silent validation do not
    block a Gauntlet later than the character's own.
- **`post_gauntlet_lab_seasons` is one total of *charged* seasons, not a per-year list.**
  The deduction stops at the third season of a year (`:2482`), so every legal per-year
  distribution totals at most `3 × years`, every total in that range is realizable, and
  all of them cost the same — one number is lossless. It must count *charged* seasons:
  sixteen seasons actually worked cost 0 points across four years but 30 across five, so
  a single total of worked seasons could not tell those apart.
- **The post-Gauntlet experience joins the *general* pool — it is not a block of its
  own.** `effective/xp.rs`'s `xp_allocation` selects `base_general` as
  `apprenticeship_xp + post_gauntlet_xp` for a magus (saturating), leaving the
  non-magus arm and the plan-less `xp_pool` arm untouched. Three sourced facts force
  the general pool rather than a restricted one:
  - `:2216` — "Divide 30 points per year between experience points in **Arts**,
    experience points in Abilities, and levels of spells" — and `:2471` — "Each point
    can be an experience point in an **Art** or Ability or one level of spell". Only
    the general pool may fund an Art (`pool_covers` returns false for every
    `(Ability pool, Art spend)` pair), so a restricted pool could not express this.
  - The Academic/Arcane/Martial gate does not narrow them: `:2435` restricts what a
    magus may spend "**before** apprenticeship", and `:7151` puts the years after it on
    the permitted side — "Magi without a specific Virtue may only buy Academic Abilities
    during or after apprenticeship". The whole-character waiver of `:7151` already
    applies to a magus, so nothing further is needed.
  - Consequently **no** `LifeStageBlock` variant, **no** `RestrictedXpPool`, and **no**
    `xp-pool-<slug>` Fluent key were added: the block that funds anything is the general
    pool, which is the one block with no slug. Its size is surfaced as
    `EffectiveScores.xp_general_pool`, exactly as apprenticeship's was.
  - Later life is untouched by this and stays the restricted, Abilities-only pool of
    `:2435`'s "before apprenticeship" clause however many years the magus has lived
    since (`post_gauntlet_years_leave_later_life_restricted`).
- **The spell levels are *additive* to the profile's 120 — not a second budget.**
  `effective/spell.rs`'s `life_stage_spell_levels(entity, ruleset)` (the budget's
  `post_gauntlet_spell_levels`; 0 without a plan or without the block) is folded into
  `spell_levels_budget`, which is `base + spell_levels_bonus +
  life_stage_spell_levels`, clamped as before. Apprenticeship's "120 levels of spells"
  (`:2435`) are the magus profile's `spell_levels` in
  `rules/core/character_types.json`, which is what `spell_levels_base` selects; the
  post-Gauntlet levels are the player's chosen slice of the fungible 30 points a year
  (`:2471`) — the same reason `post_gauntlet_xp + post_gauntlet_spell_levels ==
  post_gauntlet_points`. A magus 35 years out with 300 points banked as levels has a
  budget of 420.
  - Folded into **`spell_levels_budget`** on purpose: it is the single selector both
    `validate_spells` (the `over_spell_levels` finding) and the `EffectiveScores`
    payload call, so the finding and the bar can never disagree. `validate_spells`
    itself needed no change — a change there would have meant the term was in the
    wrong place.
  - `Entity::spell_levels_override` still replaces the **profile base** only; the
    post-Gauntlet levels stay additive on top. Deliberate: the override is the flat
    flow's escape hatch and was never made exclusive with a plan the way `xp_pool`
    once was — and since schema 16 `xp_pool` is not exclusive with one either (see
    *Plan vs. pool*).
  - **App payload:** `EffectiveScores` (`arm-app/ruleset_io.rs`) carries the three
    parts of the budget separately — `spell_levels_profile_base`,
    `spell_levels_bonus` and `spell_levels_life_stage`, whose identity is
    `base + bonus + life_stage == spell_levels_budget` — so the spell-levels bar can
    label each rather than showing an unexplained total, the way the XP bar lists its
    extra pools beside the general pool. Mirrored by hand in `ui/src/lib/types.ts`
    (`EffectiveScores`, `LifeStagePlan`'s three choices, and the new
    `PostApprenticeshipRules` on `LifeStageRules`), pinned by
    `every_life_stage_field_is_mirrored_in_the_frontend_types`. The views that render
    them are **M6/6b5b**.
- **Four findings, and the reason the clamps stay.** The first three are
  `validate_post_gauntlet_choices` in `validation/life_stage.rs`, all
  `error`/`experience`, all magus-gated because `:2216` is "**Hermetic Magi Only
  (Optional)**" and on any other plan the values are ignored outright:
  - `life_stage_gauntlet_age_after_age` (`gauntlet_age`, `age`) — a Gauntlet in the
    character's future.
  - `life_stage_lab_seasons_out_of_range` (`seasons`, `max`, `years`) — more charged
    seasons than `max_charged_lab_seasons_per_year × post_gauntlet_years` (`:2482`).
    The message spells out the *charged* reading, which is the one thing a player
    misreads: the deduction is exhausted by the third season of a year, so a fourth
    is free and never counted here.
  - `life_stage_spell_level_split_exceeds_points` (`levels`, `points`) — a spell-level
    share larger than the points the years granted (`:2471`).
  - The fourth is `aging_rolls_pending` (warning, `aging`), which the
    post-Gauntlet years make reachable at all: a magus generated years out of
    apprenticeship is routinely over 35, and "a character over the age of 35 must make
    aging rolls before the game begins" (`:2232`, `:16565`). It is emitted from
    `validation/aging.rs` for **every** character of that age, not only a magus with a
    plan — see *Age at which aging rolls become due* and the aging section above.
  - **Why the clamps stay.** `ValidationMode::Advisory` downgrades every issue to a
    warning and `Silent` drops it, so neither blocks a save; the budget therefore has
    to stay arithmetically sane whatever a file holds, which is what
    `min`/`saturating_sub` guarantee. The price is that a wrong number *vanishes*
    instead of failing — a Gauntlet at 40 on a magus of 25 silently loses the years
    between, a 200-season total silently costs the same as 105, a 5 000-level split
    silently becomes "all of them". Each finding names the value the clamp absorbed, so
    the two together are honest: the arithmetic never breaks and the player is still
    told what was ignored.
- **The Gauntlet-age floor moved onto the Gauntlet age** — an existing finding
  retargeted, not a new one. `minimum_gauntlet_age()` (childhood + apprenticeship,
  `:2435`) is compared against `LifeStageBudget::gauntlet_age` — the resolved value
  `budget()` funds the character from, read off the budget rather than re-derived, so
  `life_stage_age_before_gauntlet` and the arithmetic cannot drift. A magus of 60
  gauntleted at 12 never served its fifteen years either, and only the Gauntlet age
  sees that; with no `gauntlet_age` stored the two numbers are identical, which is what
  keeps every pre-6b5 plan reading as it did.
- **`life_stage_spell_level_split_exceeds_points` is filed under `experience`, not
  `spells`**, although it feeds `spell_levels_budget`. M6/6b1a's rule is that a finding
  belongs to the phase whose *input surface* owns the offending value: the number is
  typed into the life-stage panel, and the magus phase order is
  `… experience, abilities, arts, spells`, so filing it under `spells` would let the
  guided wizard walk past the only step that can correct it. (It was `abilities` until
  the Slice 2 phase split moved the panel — and with it every code it reports — onto
  the new `experience` step; the rule that decided it is unchanged.)

#### The life-stage blocks are an ordered sequence, and the XP bar reads as one (Slice 8 / #14)

> 5. **Early Childhood.** 75 experience points in Native Language …, and 45 experience
>    points spread between Area Lore …
> 6. **Later Life.** 15 experience points per year (until apprenticeship for magi) …
> 7. **Hermetic Magi Only: Apprenticeship.** Divide 240 experience points …
> 8. **Hermetic Magi Only (Optional):** Years after apprenticeship. Divide 30 points
>    per year …

> Abilities represent a character's learned abilities. For grogs and companions they
> are acquired in two blocks: early childhood, and later life. For magi, there are two
> more periods to consider: apprenticeship, and life as a magus after that.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2213-2216` (the numbered
  creation summary, steps 5-8) and `:2364` (the same four blocks named as periods, in
  the same order). Early childhood's two figures are **one** block, not two:
  `:2378` grants "75 experience points in their native language … and 45 experience
  points to divide between" the spread, in one sentence about "the first five years of
  life". The span later life covers is worked through by the Darius example at
  `:2402` — apprenticed at 10 after a childhood ending at 5, he "has 75 experience
  points to spend from those five years", i.e. ages 5-10 at 5 × 15.
- Data: no new value. Every figure is already on `LifeStageBudget`
  (`life_stage.rs`, `budget`), the span's start is
  `rules/core/life_stages.json` → `childhood.years` (5) and its length is the
  engine's own `later_life_years`.
- Implementation: `ui/src/lib/components/XpBar.svelte` renders **one chip per block**
  in this order — early childhood, later life, apprenticeship, after the Gauntlet —
  keyed `xp-pool-block-*` in both locales. Each chip merges the block's derivation
  with the spent/total of the restricted pool it forms
  (`XpPoolOrigin::LifeStage`, looked up by block, never by index), so no block's name
  can appear twice. The `xp-pool-<block-slug>` keys remain, for
  `resolveIssueArgValue`'s naming of an unspent-experience warning's `origin`.
- **Why the ORDER is a rules fact and not styling.** The blocks are the periods of one
  life, and `:2213-2216`/`:2364` state them in the order they are lived. Rendered with
  later life last, its label read as the years *after* the Gauntlet — a chronology
  asserted wrongly, which is a misstatement of `:2214`'s "until apprenticeship for
  magi" rather than an aesthetic complaint.
- **"After the Gauntlet", not "As a magus"** (`:2216`, "Years after apprenticeship").
  The block is driven by `LifeStagePlan::gauntlet_age`, so naming it for the Gauntlet
  ties the label to the field that moves it. Renamed in both bars —
  `xp-pool-block-after-gauntlet` and `spell-levels-post-gauntlet` — and **only** on
  those two labels: the six strings that say "years as a magus" are prose describing
  the span, and `:2216` calls the same period "life as a magus after that" (`:2364`),
  so they read correctly and keep it.
- German: `Frühe Kindheit`, `Späteres Leben`, `Lehrlingszeit` and
  `Nach der Lehrlingsprüfung`, from the mirrored German lines
  `Ars Magica Definitive Edition Basisregeln.md:2213-2216` (heading `:2376`, `:2390`)
  plus the glossary's `Gauntlet → Lehrlingsprüfung`
  (`rules/source/de/translation-tables/grundbegriffe.md:57`).

#### Pre-apprenticeship experience buys Abilities only — never Arts (M6/6b4)

> 6. **Later Life.** 15 experience points per year (until apprenticeship for magi),
>    spread between any Abilities the character can learn, based on the Virtues and
>    Flaws he has. …
> 7. **Hermetic Magi Only: Apprenticeship.** Divide 240 experience points between
>    Hermetic Arts and any nonSupernatural Abilities (or Supernatural Abilities, if
>    the magus has the relevant Virtue). …
> 8. **Hermetic Magi Only (Optional):** Years after apprenticeship. Divide 30 points
>    per year between experience points in Arts, experience points in Abilities, and
>    levels of spells.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2213-2216` — the numbered
  creation sequence, and the decisive statement of *which* period may buy Arts: step 6
  (`:2214`) says "any **Abilities**", step 7 (`:2215`) "between Hermetic **Arts** and
  … Abilities". Restated for the block itself at `:2392`.
- Implementation: `effective/xp.rs` — `xp_allocation` pushes later life as a
  **restricted** `PoolEligibility::Ability` pool for a magus (`is_magus &&
  budget.later_life_xp > 0`, the same shape of guard the mastery pool uses), so Arts
  fall out for free: an Ability pool never covers an Art spend. `LifeStageBlock` gains
  `LaterLife` (slug `later_life`, labelled `xp-pool-later_life` in both locales) so the
  bar can name the row.
- **The reason, not just the wording:** a magus's later life *ends where
  apprenticeship begins* (`:2214`, "until apprenticeship for magi"), so it is the span
  in which the character is a child not yet taken as an apprentice — and has no Arts at
  all. Funding an Art from it would be funding it before the Gift was ever opened.
- Where an experienced magus's Arts actually come from: `:2216`/`:2471`'s "For every
  year, the magus gets 30 points", the years *after* apprenticeship — modelled in
  **M6/6b5**, whose experience joins the **general** pool precisely because only that
  pool may fund an Art. So later life stays Abilities-only however many years the
  magus has lived since its Gauntlet
  (`post_gauntlet_years_leave_later_life_restricted`), and a magus is no longer built
  standing at its Gauntlet: the age past it is spent through
  `LifeStagePlan::gauntlet_age`. Nothing in the source bounds the apprenticeship start
  age — Darius's master "picks 10 as a nice, round number" (`:2402`) — so no such
  bound is enforced.

#### Pre-apprenticeship experience may not buy Arcane, Academic or Martial Abilities (M6/6b4)

> Note that magi can only spend experience points on Arcane, Academic and Martial
> Abilities before apprenticeship if they have a Virtue which allows them to do so.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2435` (second sentence),
  with `:2215`'s "any nonSupernatural Abilities (or Supernatural Abilities, if the
  magus has the relevant Virtue)" for apprenticeship itself. The Darius example
  reasons exactly this way about a pre-apprenticeship purchase: Order of Hermes Lore
  "It's a **general** Ability, so he can" (`:2402`).
- Implementation: the later-life pool's eligibility (`effective/xp.rs`,
  `xp_allocation`) — `categories` is `AbilityCategory::ALL` minus
  `ruleset.categories_requiring_virtue()` plus whatever the character's Virtues
  authorize, and `abilities` is the authorized ids. Both come from
  `effective::ability_authorizations`, moved here from `validation/authorization.rs` in
  this slice so the ownership check and the pool read one function. (Direction matters:
  `validation` already depends on `effective`, so calling the other way would invert
  the layering.)
- **No double-reporting with `validate_ability_authorization`.** That validator gates
  *owning* a gated Ability and exempts magi whole-character (`:7151`, "or if they are
  magi"); the pool decides *whose money* pays. A magus who overspends its
  apprenticeship gets `not_enough_xp`, never `ability_category_requires_virtue`. The
  two mechanisms are disjoint by construction.
- **Supernatural stays in the pool's category set and legalizes nothing.** Access to
  each Supernatural Ability is granted per Ability (`:2315`), which
  `validate_supernatural_abilities` enforces for magi as well — so an unauthorized one
  is already an error and funding it here changes nothing. Excluding it would only
  produce a second, differently-worded complaint about the same row.
- **Deliberate asymmetry: later life stays the general pool for a non-magus.** A grog
  or companion is gated by an error on the character
  (`ability_category_requires_virtue`), so its money needs no restriction; a magus's
  category gate is waived whole-character, so the pool is the only place the "before
  apprenticeship" half of `:2435` can live. This is also where `:7151`'s finer
  distinction now bites — see **Access to Academic / Arcane / Martial Abilities**
  below, whose "not modelled" note this slice narrows.

#### Hermetic minimum Abilities — Parma Magica 1, Magic Theory 1, Latin 1 (M6/6b4)

> Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory 1,
> Latin 1. Characters with lower scores would not be admitted to the Order.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2437`.
- Data: `rules/core/life_stages.json` → `apprenticeship.minimum_abilities` —
  `ability.dead_language 1`, `ability.magic_theory 1`, `ability.parma_magica 1`. The
  list is **data**: a ruleset demanding something else says so in its own file, and no
  Rust anywhere names these three.
- Implementation: `life_stage.rs` — `magus_minimum_abilities`, producing one
  `MagusMinimumAbility` row per requirement (`AbilityRequirementKind::Required` here,
  `Recommended` below). **One function, two consumers:**
  `validation/magus.rs::validate_magus_minimum_abilities` emits
  `magus_minimum_ability` (error, `abilities`, args `ability`/`min`/`score` plus an
  optional `exemplar`, context = the Ability), and `EffectiveScores.magus_minimum_abilities`
  (`arm-app/ruleset_io.rs`) hands the whole checklist to the frontend — so a finding
  and the checklist a UI shows cannot disagree.
- **An error, and unconditional.** "Would not be admitted to the Order" is a hard bar,
  and `:2437` says nothing about how the experience was earned, so a magus built from a
  flat `xp_pool` is held to it exactly as a guided one is. That is why the validator
  lives in `validation/magus.rs` and not in `validation/life_stage.rs`, which returns
  early without a life-stage plan.
- **Documented approximation: "Latin 1" is matched by Ability id.** Latin is one
  *value* of the parameterized `ability.dead_language` (`:7431-7434`, where Latin is
  only "the most important example"), and an instance value is free-text player input
  with no localization path — a German player types "Latein". Matching the instance
  would therefore fail for every non-English user, which is worse than
  under-enforcing. **The consequence, stated plainly: a magus whose only dead language
  is Greek 1 passes `:2437`.** The same id-level proxy `ability.scholarly_language`
  (`rules/core/abilities.json`) and `flaw.covenant_upbringing`'s
  `ability_authorization` already use. `AbilityRequirement::parameter` exists unused so
  a future language registry tightens this by filling one JSON field, with no code
  change; the test `latin_is_matched_by_ability_id_not_by_instance` pins the current
  behaviour so it can never become accidental.
- **The widening is PERMANENT, and a `language.*` catalogue is rejected — not
  deferred (guided-creation review #32, Slice 7).** The obvious way to make "Latin 1"
  enforceable is a language catalogue so the requirement can name Latin by id. **It
  cannot be built.** The rules publish no comprehensive list of languages, and whether
  a given language exists — and whether it is dead or living — is a **troupe's
  decision**. Authoring one would mean inventing rules data, which `CLAUDE.md`'s
  provenance rule prohibits outright. So free text is the **correct** model for the
  `language` parameter, not a shortcoming, and the paragraph above about
  `AbilityRequirement::parameter` awaiting "a future language registry" describes a
  registry that will never exist. It also follows that the check can never be narrowed
  to Latin mechanically: string-matching a user-typed value against a localized name
  would be locale-dependent and would break on `latin`, `Lateinisch`, or a troupe's own
  spelling. **Do not propose a language catalogue as an improvement.**
- **The honesty fix instead: the requirement carries the rules' own exemplar as a
  label.** `AbilityRequirement::exemplar` (and `ScholarlyLanguageRequirement::exemplar`)
  hold a **language-neutral slug** — `"exemplar": "latin"` — on the three sites where
  the rules name Latin: `rules/core/life_stages.json` `minimum_abilities`
  (`dead_language ≥ 1`, `:2437`) and `recommended_abilities` (`≥ 4`, `:2455`), plus
  `rules/core/abilities.json` `scholarly_language` (`≥ 3`, `:7151` — "For most
  characters, Latin 3 is required"). Its **translated text lives in
  `rules/i18n/<lang>/abilities.json`** under `exemplar.latin` (`Latin` / `Latein`, the
  latter as the German rulebook uses it at the mirrored `:2437`), so no translatable
  string enters the mechanics file. `MagusMinimumAbility` carries it through to the
  frontend and both validators emit it as an **optional** `exemplar` arg, so the
  minimums row and the `issue-magus_minimum_ability` /
  `issue-academic_ability_without_scholarly_language` messages read identically:
  *"Dead Language (e.g. Latin) 1 is not met"*. The one shared label path is
  `requirementAbilityLabel` in `ui/src/lib/derive.ts`. **The enforced check is
  unchanged** — still any Dead Language ≥ N. The exemplar is one *named example*, never
  an enumeration of languages.
- **The exemplar slug is a LABEL KEY, not a referential-integrity `ref`.** It resolves
  against no catalogue — there is none to resolve against — so the loader deliberately
  does not check it (documented at both check sites,
  `ruleset/integrity.rs::validate_apprenticeship_refs` and
  `ruleset/parse.rs::check_scholarly_language`) while the requirement's `ability` **is**
  a ref and still fails loudly. Pinned by
  `an_exemplar_slug_is_not_treated_as_a_referential_integrity_ref`, which also proves
  the test is not vacuous by showing a bogus `ability` still rejected. Its i18n
  coverage in both locales is pinned by `the_exemplar_slug_resolves_in_both_locales`,
  and its presence on the three sites by
  `the_magus_minimum_dead_language_requirement_names_its_exemplar`.
- **The score tested is the BOUGHT one**, not the effective one:
  `effective_ability_score` returns 2 for a magus with a Puissant Parma Magica and no
  Parma row at all, and `:2437`'s "scores" cannot mean a Virtue's +2 to *use*. The same
  `entry.score >= min_score` test `validate_academic_language` applies. General ruling:
  the age caps constrain the bought score, and the minimums test it. Pinned by
  `puissant_parma_magica_does_not_admit_a_magus_to_the_order`.
- **A minimum age of 20 follows** from the same block: childhood (5) plus
  apprenticeship (15), `LifeStageRules::minimum_gauntlet_age`. A younger magus with a
  life-stage plan gets `life_stage_age_before_gauntlet` (error, `experience`, args
  `age`/`min`) **instead of** `life_stage_age_before_childhood` — one wrong age, one
  finding, under the code that describes it truthfully.
- **The minimum set is deliberately not re-priced** the way the recommended one is:
  `:2437` states no total, so a pricing check could only compare the engine to itself.

#### Hermetic Magi Recommended Minimum Abilities — 90 experience points (M6/6b4)

> #### Hermetic Magi Recommended Minimum Abilities
>
> Artes Liberales 1
>
> Latin 4
>
> Magic Theory 3
>
> Parma Magica 1 (should be no higher if the magus is just out of apprenticeship)
>
> Total Cost: 90 experience points

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2451-2461` (heading
  `:2451`, the four Abilities `:2453-2459`, the total `:2461`), with the consequence
  half of `:2437`: "A character without a Latin score of least 4 and an Artes
  Liberales score of at least 1 is unable to read the books of the Order… A Magic
  Theory score of below 3 is weak, and, in particular, means that the magus cannot set
  up his own laboratory."
- Data: `rules/core/life_stages.json` → `apprenticeship.recommended_abilities` +
  `recommended_xp: 90`.
- Implementation: the same `magus_minimum_abilities` rows, tagged
  `AbilityRequirementKind::Recommended`, reported by
  `validate_magus_minimum_abilities` as `magus_recommended_ability` (**warning**,
  `abilities`, args `ability`/`min`/`score` plus an optional `exemplar` — the Latin 4
  row states one, `:2455`).
- **A warning, not an error**, because `:2451` calls the list *recommended* and the
  consequences `:2437` spells out describe a weak magus, not an illegal one — unlike
  `:2437`'s own three, which decide admission.
- **The 90 is a re-pricing trust gate.** `Ruleset::validate_apprenticeship_refs`
  prices the list off the Ability advancement table and refuses any other total:
  5 (`:2408`) + 50 (`:2411`) + 30 (`:2410`) + 5 = 90 exactly. A mistyped score fails
  the load instead of shipping a recommendation the rulebook never costed — the gate
  `validate_childhood_packages` applies to childhood's 45 and 75. The literals are also
  asserted from outside the data, in
  `tests/data_integrity.rs::shipped_apprenticeship_carries_the_2435_and_2437_numbers`,
  so an edit moving list and total together cannot pass in silence.
- Parma Magica 1 appears in **both** lists (`:2437` and `:2459`), so it legitimately
  produces two checklist rows — one required, one recommended. Not a duplicate.
- **Not modelled:** `:2459`'s "(should be no higher if the magus is just out of
  apprenticeship)". It is advice about what apprenticeship *teaches* — "this Ability is
  normally the last thing taught" (`:2437`) — and the engine has no per-stage
  attribution for a bought score to hang a maximum on, the same limit recorded for
  `:7151` below.

#### Wealthy / Poor — the rate, and who may take them

> Characters with the Wealthy Virtue get 20 experience points per year, while
> characters with the Poor Flaw get 10 experience points per year. Note that only
> companions can take this Virtue or Flaw.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2394`; the items
  themselves at `:5235-5238` (Wealthy) and `:6594-6596` (Poor, which repeats "In
  particular, this Flaw is not available to magi").
- Data: `rules/core/virtues_flaws.json` — `virtue.wealthy` and `flaw.poor` each
  carry `later_life_xp_rate` (20 / 10). **`flaw.poor` was missing from the
  catalogue entirely before this milestone** and is added here; German name "Arm"
  (`rules/source/de/translation-tables/tugenden-fehler.md:445`, corroborated by the
  German source at the mirrored `:6594-6596`).
- Implementation: `Effect::LaterLifeXpRate` (`types.rs`), read by
  `LifeStageRules::later_life_rate`. The effect **replaces** the base rate rather
  than adjusting it, because the passage states the whole rate. Engine reading where
  the text is silent: if several selections ever name a rate, the lowest applies —
  nothing ranks them, so this is the conservative and deterministic choice.
- Eligibility, as data: `rules/core/character_types.json` — the `magus` and
  `mythic_companion` profiles list both ids in `forbidden_traits`. The grog profile
  needs no entry, since `max_major_flaws: 0` already puts a Major Flaw out of reach.
  Scope reading: `:2394` says "only companions", while `:5237`/`:6596` say only that
  magi may not; a mythic companion *is* a companion in the rules' sense but is a
  distinct profile here, so it is forbidden too — the stricter reading of the line
  that names companions specifically.

#### Access to Academic / Arcane / Martial Abilities (M6/6b2b)

> There are two exceptions. One is that a character must have a Virtue to buy
> Academic, Arcane, Martial, or Supernatural Abilities at character creation.
> Educated, Arcane Lore, and Warrior, respectively, are the easiest options for the
> first three groups, although other Virtues (and some Flaws) also grant access to
> some of these Abilities.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:2315`, restated for the
  later-life block at `:2392` ("as long as the character has a Virtue that permits
  her to learn those Abilities").
- Data: `rules/core/abilities.json` → `categories_requiring_virtue`
  (`academic`, `arcane`, `martial`). Empty stands the rule down, so a ruleset gating
  a different set says so in its own file.
- Implementation: `crates/arm-rules/src/validation/authorization.rs` —
  `validate_ability_authorization`, emitting `ability_category_requires_virtue`.
- A Virtue grants access two ways, both counted: an explicit
  `Effect::AbilityAuthorization`, or **any** `Effect::RestrictedAbilityXp` pool —
  experience earmarked for a category is evidence the category is permitted, since
  the grant would otherwise be unspendable. That covers Educated, Warrior, Arcane
  Lore and Privileged Upbringing from their existing data.
- Wired Virtue/Flaw: `flaw.covenant_upbringing` gains
  `ability_authorization: [ability.dead_language]` for "You may take Latin at
  character creation" (`:5867`). **Documented approximation:** authorization is by
  ability id, so this permits any dead language, not Latin alone — the same
  id-level proxy the `ability_score_grant` effects already use. Other access-granting
  Virtues are a data addition, never a code change.
- **Supernatural is deliberately excluded** from this check: `:2315` says access "is
  granted by a separate Virtue" per Ability, which the stricter, pre-existing
  `validate_supernatural_abilities` / `supernatural_ability_requires_virtue` already
  enforces. Folding it in here would double-report.
- **Magi are exempt**: `:7151` ("Beginning characters may only purchase Academic
  Abilities if they are specifically permitted to through the purchase of a Virtue,
  **or if they are magi**") and `:2435`, where apprenticeship experience may go on
  "Arcane, Academic, and Martial Abilities". Read from the profile's `is_magus` flag,
  never a type id. `:7151`'s finer "Magi without a specific Virtue may only buy
  Academic Abilities **during or after** apprenticeship" is now modelled **for a
  guided magus** (M6/6b4): the restriction is not on the Ability but on the money, so
  a magus's pre-apprenticeship experience simply cannot fund those categories — see
  **Pre-apprenticeship experience may not buy Arcane, Academic or Martial Abilities**
  above. It remains unmodelled for a magus funded from a flat `xp_pool`, which carries
  no per-stage attribution at all; there the exemption stays whole-character. The
  ownership check here is unchanged either way, and the two never double-report: a
  shortfall is `not_enough_xp`, never `ability_category_requires_virtue`.

#### The scholarly-language expectation for Academic Abilities (M6/6b2b)

> Academic Abilities require formal training. … In addition, learning an Academic
> Knowledge normally requires a Latin, Greek, Hebrew, or Arabic score of at least 3,
> depending on the region of Europe you are from. For most characters, Latin 3 is
> required.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:7151`.
- Data: `rules/core/abilities.json` → `scholarly_language`
  (`{ ability: ability.dead_language, exemplar: "latin", min_score: 3 }`).
- Implementation: `validation/authorization.rs` — `validate_academic_language`,
  emitting `academic_ability_without_scholarly_language` (args `ability`/`min` plus an
  optional `exemplar`).
- A **warning**, not an error, because the passage hedges twice ("normally",
  "depending on the region of Europe"). Engine reading: the data names the
  parameterized dead-language ability and the minimum score rather than enumerating
  Latin/Greek/Hebrew/Arabic, and any instance at that score satisfies it — the engine
  cannot know a saga's region, and the four names are examples of one Ability.
- The `exemplar` slug surfaces the passage's own "For most characters, Latin 3 is
  required" as a label (`Latin` / `Latein` from `rules/i18n/<lang>/abilities.json`
  under `exemplar.latin`) without narrowing the check. See the widening note under
  **Hermetic minimum Abilities** for why it can never be narrowed, and why the slug is
  a label key rather than a `ref`.

#### Foreign Upbringing halves locality-dependent caps (M6/6b2c)

> The maximum scores at character creation for locality-dependent Abilities like
> Language, Area Lore, or Organization Lore, as well as some social Abilities, are
> half (round up) that which his age normally allows.

- Source: `Ars Magica - Definitive Edition (Core Rules).md:6160`.
- Data: `rules/core/virtues_flaws.json` — `flaw.foreign_upbringing` carries
  `locality_ability_cap_fraction { num: 1, den: 2 }`; `rules/core/abilities.json`
  flags `locality_dependent` on `ability.living_language`, `ability.dead_language`,
  `ability.area_lore` and `ability.organization_lore`.
- Implementation: `effective/reputation_and_caps.rs` — `ability_age_cap` (per-ability, ceiling division),
  consumed by `validate_abilities`'s `ability_above_age_cap` check.
- **Deliberately incomplete, per the source:** the passage's trailing "as well as
  some social Abilities" names no Abilities, so none are flagged — the engine
  enforces the flag it is given rather than deciding which social Abilities a saga
  counts. Flagging more is a data edit with no code change.
- The fraction caps the *score*, not the cost: such an Ability is bought at the usual
  price, just not as high. Composition rule (no shipped Flaw pairs with another): the
  smallest resulting cap applies.

#### Plan vs. pool — the funding mode is stored (schema 16), not inferred

**Retired invariant.** The engine used to make a life-stage plan and a typed
`xp_pool` **mutually exclusive** and report `life_stage_xp_pool_conflict` when a
character carried both. **That invariant no longer holds, and the finding no longer
exists** — code, contract table and both locales. It was removed deliberately in
Slice 4 of `docs/guided-creation-implementation-plan.md` (review issue #29); do not
reinstate it.

Why it was an invariant at all: the funding mode was not stored anywhere, so the
plan's *presence* **was** the mode. Recording "pool" therefore meant deleting the
plan, and recording "life stages" meant zeroing the pool — an unconfirmed,
unrecoverable loss of everything the player had typed on the side switched away
from. Two funding sources visible at once really did mean a character could spend
twice, so the finding was correct given the representation.

What replaces it: `Entity::ability_funding` (`AbilityFunding::Pool` /
`LifeStages`, `types.rs`), a closed enum stored on the entity. The inactive side is
now kept and simply **inert**, and nothing can double-count because
`LifeStageRules::budget` (`life_stage.rs`) returns `None` under `Pool` — the single
funnel every mode-sensitive path goes through (`effective::xp_allocation`'s
restricted and general pools, `life_stage_spell_levels`, and
`validate_life_stage_plan`, which returns early under `Pool` so an inert plan raises
no finding).

**Deliberate departure from the sparse-save principle.** Everywhere else a save
omits what it does not need, and every other defaulted `Entity` field is
`skip_serializing_if`. Here the save deliberately **keeps data the active mode
ignores** — a full life-stage plan beside a nonzero `xp_pool` — because discarding
it is precisely the destruction #29 fixed. Two consequences that must not be
"tidied" back:

- `ability_funding` is the **one** `Entity` field written even when it holds its
  default. `load_entity_migrating` dispatches on the key's *absence* to fold a
  pre-16 save (no key → infer `LifeStages` if `life_stages` is present, else
  `Pool` — the pre-16 rule, applied once at load), so omitting `Pool` would make a
  pool-funded character that keeps a plan silently reload as life-stage funded.
- A save may legitimately carry both a plan and a pool. Neither is redundant data
  to be pruned; each is the player's typed work on one of the two modes.

Sites that still read the plan's `Option` rather than the mode, deliberately:
`completeness.rs`'s `experience` row (it asks "has anything been recorded", and
`ability_funding` has a default every untouched character carries),
`childhood::apply_package` (it reads the native language off the plan and records the
package on it), and `native_language_instance` (`effective/xp.rs`, called only inside
the already mode-gated budget branch).

The remaining life-stage findings are unaffected: an unset age, an age below the
character's first possible one — childhood for a grog or companion
(`life_stage_age_before_childhood`), childhood plus apprenticeship for a magus
standing at its Gauntlet (`life_stage_age_before_gauntlet`, `:2435`) — an unchosen
native language, and a chosen one with no bought score (a warning — the points are
merely unspent). Only the two age bars are sourced.

#### `Entity::wizard_furthest_phase` — UI/document state, no rule

The furthest guided-wizard phase the player reached, added alongside
`ability_funding` in the same schema-16 bump (review issue #31). **Not a rule and not
rules data**: no rulebook passage is cited or citable for it, no validator reads it,
nothing derives from it, and no finding may ever be attached to it. It is UI state
that happens to live on an engine-defined entity because it has to survive a save.

Stored as a **raw slug (`Option<String>`), deliberately not `Option<CreationPhase>`**.
`CreationPhase` is a closed `Deserialize` enum, so an unrecognised slug would be a
serde error — and by this crate's migration rule a value that will not deserialize
fails the *whole* load, which would turn a save carrying a since-removed phase slug
(`"type"`, dropped in Slice 2) into an unopenable file, and would do the same for
every future phase rename. The slug is therefore kept verbatim and resolved
**leniently** by the caller against the loaded profile's `creation_phases`;
unresolvable is treated exactly like absent. `None` writes no key at all, so a
character built in the editor round-trips byte-identically.

**Who writes it, and what "leniently" means (Slice 5).** The frontend owns both ends;
the engine still never looks at it.

- *Written* by the wizard's `next()` only — `WizardNavigation.next` in
  `ui/src/lib/wizard-navigation.svelte.ts`, through the host callback
  `recordFurthestPhase` so the entity stays `AppStore`'s to mutate. `back()` and
  `goTo()` leave it alone, which is what keeps rail browsing off the unsaved-changes
  guard while a deliberate Next marks the document changed (even on a step left
  empty — a step can be legally empty, since the flow gates on errors only).
- *Read* by `WizardNavigation.restore`, called from `AppStore.enterWizard`. Two
  branches: a slug that names a step of this rail restores as both the current and
  the furthest step, so the run resumes clamped exactly as it was left; a slug that
  does not, or an absent one, opens the whole rail and sets `ungated`, which lifts
  the blocking clamp on forward jumps as well as the Next/Finish gates. **Both halves
  are required** — the ceiling alone leaves every step refused by `goTo`'s own
  `step > furthest` guard, and the flag alone leaves them all locked at step 0. The
  ungated branch is the editor-built (or migrated) character: it never passed these
  gates, so holding it to them would lock the very steps it must reach to be fixed.
  Findings are still displayed throughout; only enforcement is lifted.
- Resolved against the wizard's rail (`wizardPhases` = the profile's
  `creation_phases` plus the wizard's own terminal `review`) rather than
  `creation_phases` alone, so a run that reached Review round-trips too.

The unspent-block warning and the 75-point pool read `:2378` the same way, which is
a requirement rather than a coincidence: both key on
`childhood.native_language_ability` at the chosen instance, scoring above 0
(`validate_life_stage_plan` and `native_language_instance` in `effective/xp.rs`). A
looser test — any parameterized Ability whose parameter equals the language — would
let an `Area Lore (German)` declare the block spent while the pool, which funds one
instance of one id, paid for nothing of it.

## Aging (M6/6b6) — `aging.rs`

The yearly roll of `## Aging` (`:16563-16617`), from the threshold that owes it to
the write-back that applies it. `rules/core/aging.json` carries every number,
`crates/arm-rules/src/aging.rs` the arithmetic,
`crates/arm-rules/src/validation/aging.rs` the findings, and
`Ruleset::validate_aging_rules` (`ruleset/integrity.rs`) the load-time gates that make the
data checkable rather than merely transcribed.

**The engine never rolls.** `arm-rules` has no `rand` dependency and never will:
the stress die is thrown at the table and typed in, and everything here is the
arithmetic *around* that number. A generator that rolled would invent
rules-relevant state nobody at the table agreed to, and would make a character's
history unreproducible from its save file.

**The Crisis is resolved, never survived.** The engine totals it (`:16621`), reads
the row (`:16624-16632`), records it on the year (`resolve_year`'s Crisis leg) and
reports what surviving would take (`:16628-16638`). It never throws the Stamina die,
never resolves the doctor's Medicine roll (`:16634`) and never kills — `:16617`'s
death at Decrepitude 5 ships as a datum nothing acts on. See *Recorded gaps* at the
end of this section.

#### Age at which aging rolls become due — over 35
> "Characters begin aging in the Winter after they turn 35. Every year, a
> character must roll on the aging table."

> "The first thing to bear in mind is that a character over the age of 35 must
> make aging rolls (see page 392) before the game begins."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16565` (the threshold),
  `:2232` (the rolls are owed before play begins).
- **Data**: `rules/core/aging.json` → `start_age: 35`. It was a cited Rust
  constant (`AGING_ROLLS_START_AGE`) until this slice created the file; the
  constant is gone, and a ruleset shipping no aging block stands the subsystem
  down rather than letting the engine supply a fallback number.
- Implementation: `AgingRules::first_roll_age()` (`aging.rs`) does the one
  deliberate `+1` — "the Winter **after** they turn 35" falls in the 36th year, and
  `:2232`'s "over the age of 35" agrees — and
  `validation/aging.rs::report_pending_aging_rolls` emits
  `aging_rolls_pending` (warning) for a character who has reached it with
  an empty `aging_log`. `:2496`'s "each year from the age of 35" is advice inside a
  worked advancement example, disposed of in `first_roll_age()`'s doc comment rather
  than ignored.
- The schedule itself is `aging::aging_schedule(entity, ruleset) -> Vec<AgingYear
  { age, year, recorded }>` — `first_roll_age()..=age`, pairing each age with its
  calendar year (`birth_year + age`, `None` without a birth year) and with whether
  the log already records it. Empty at or under the threshold, with no age entered,
  and under a ruleset shipping no aging rules.
- **A Longevity Ritual holder under 35 is not scheduled** — a decision, not an
  oversight. `:16575`'s "should roll on the table no matter what his age" is
  unbounded downward, nothing records *when* the ritual was made, and those rolls
  are clamped so they can never cost a point. The obligation the app enforces is
  `:2232`'s, which is age-gated with no ritual clause. `aging_total` still computes
  a pre-35 roll correctly for a caller that asks for one.

#### The AGING TOTAL
> "**AGING TOTAL: Stress die (no botch) + age/10 (round up)**
> **\- Living Conditions modifier**
> **\- Longevity Ritual modifier**"

> "As a high roll generally indicates more serious effects of age, a high Longevity
> Ritual modifier and a high Living Conditions modifier both indicate longer life."

> "The modifier to rolls depends on the character's actual, not apparent, age."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16567-16569` (the
  formula), `:16571` (the sign convention), `:16577` (actual, not apparent, age).
- **Data**: `rules/core/aging.json` → `age_divisor: 10` — the 10 of "age/10 (round
  up)". Rounding **up** steps the term on the first year of each decade, not the
  last: 30 scores 3, 31 already scores 4 (`AgingRules::age_modifier`).
- Implementation: `aging::aging_total(entity, ruleset, age, die) ->
  Option<AgingTotal>`, which returns every term (`age`, `die`, `age_modifier`,
  `living_conditions`, `longevity_bonus`, `trait_modifier`, `uncapped_total`,
  `total`, `capped_by_longevity`) rather than a bare number, so a sheet can show the
  arithmetic without re-deriving it. `age` is a **parameter**, not read off the
  entity: `:2232`'s pre-play catch-up walks every owed year and each uses *that*
  year's age.
- **The signs, which are easy to get backwards.** `:16571` works only because the
  formula *subtracts* the two named modifiers, so both are stored with the book's
  own printed sign and negated exactly once, in `aging_total`:
  - `Effect::AgingMod { kind: living_conditions }` and the table rows are
    **SUBTRACTED**. Mild Aging's `+1` (`:4530`) therefore *lowers* the total, and
    Poor Living Conditions' `-1` (`:6620`) *raises* it.
  - The Longevity Ritual bonus is **SUBTRACTED** (`:10662`, `:10672`).
  - `Effect::AgingMod { kind: aging_roll }` is a **different quantity and is ADDED
    with its stored sign**: Faerie Blood's `-1` (`:3801`) lowers the total directly,
    Strong Faerie Blood's `-3` (`:5036`) by three. This is not the same convention
    as the two named modifiers, and reading it as one would invert the sign of the
    six shipped items that carry a non-zero `aging_roll` amount (five at `-1`, plus
    Strong Faerie Blood's `-3`); the guard is
    `mild_aging_and_poor_living_conditions_move_the_total_in_opposite_directions`
    plus `faerie_blood_lowers_the_aging_total_by_one` and
    `strong_faerie_blood_lowers_the_aging_total_by_three` in `data_integrity.rs`.
  - `Effect::AgingMod { kind: longevity_bonus }` moves the ritual term, and only for
    a character who actually holds a ritual — a modifier to a bonus that does not
    exist is meaningless.

App/UI: `EffectiveScores` (`arm-app/src/ruleset_io.rs`) gains
`aging: Option<AgingReadout>` — the **die-independent** half of all of the above
(`first_roll_age`, `begins_after_age`, the `schedule`, `rolls_owed`,
`rolls_recorded`, `age_modifier`, `living_conditions_modifier`,
`longevity_modifier`, `trait_modifier`, `longevity_clamp_active`, `fixed_total`), so
the surface that shows the schedule and explains the total never re-derives a rules
number in JS.
Every field is a pure function of `(entity, ruleset)`, which is why it rides on the
always-recomputed payload; the die stays out of the entity entirely. Its terms come
from one probe of `aging_total` at a die of **zero**, so the read-out and the roll
the player actually makes are arithmetically identical by construction — sign
conventions included — and `fixed_total` is that probe's `uncapped_total`, i.e. the
three-term formula above *plus* the `aging_roll` trait modifiers, which the book's
formula block does not name but which are just as die-independent. **`trait_modifier`
surfaces that fourth term on its own** (`terms.trait_modifier`, added by
guided-creation-review-2026-08 #22) precisely because `fixed_total` includes it: the
`aging-total-formula` read-out named only the book's three, so for a character
holding Faerie Blood (`:3801`) it printed "+4 (age) 0 (living conditions) 0
(Longevity Ritual) = stress die +3" — a rules readout that did not add up. Displayed
by `ui/src/lib/components/AgingSchedulePanel.svelte`, whose sibling
`aging-total-parts` in `AgingRollCalculator.svelte` has always carried the term (off
`AgingTotal::trait_modifier`); both strings now word it identically.
`longevity_clamp_active` is deliberately **not** `AgingTotal::capped_by_longevity`:
that one says a particular roll was cut down, this one that the `:16575` clamp
stands over the character at all.

#### The `:16575` Longevity Ritual clamp — it clamps the TOTAL, not the die
> "A character under the influence of a Longevity Ritual should roll on the table no
> matter what his age, but treats all rolls of 10 or more as rolls of 9 until he
> reaches the age of 35. His apparent age may be younger than his actual age, but he
> is at no risk of actually aging before any other characters."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16575`.
- **Data**: `rules/core/aging.json` → `longevity_clamp: { max_total: 9, until_age:
  35 }` — the 9, not the 10, because it is the value the roll *becomes*.
- Implementation: `aging::aging_total` applies `total =
  uncapped_total.min(max_total)` when the character holds a ritual **and** `age <
  until_age`. A ceiling, never a floor: an uncapped total of 2 stays 2.
  `capped_by_longevity` compares rather than assuming, so it never claims a cap on a
  total the clamp did not touch.
- **The ruling, recorded because a project note had it backwards.** "Rolls of 10 or
  more" is the **total**, not the die:
  1. The formula block those rolls feed is headed "AGING TOTAL" (`:16567`) and the
     table's index column "Aging Roll" (`:16597`) — the clamp's *rolls* and the
     table's *roll* are the same quantity.
  2. 10 is meaningful only in the table's index space: it is exactly where the first
     aging-point row begins (`:16601`). On a stress die 10 is nothing at all.
  3. Decisive: a 34-year-old average peasant with the weakest legal ritual (+1 — "+1
     bonus for every five points **or fraction** of Creo Corpus Lab Total",
     `:10662`) would under the die reading score `9 + ⌈34/10⌉ - 1 = 12` and take an
     Aging Point, while a ritual-less peer of the same age rolls nothing at all. The
     die reading makes a Longevity Ritual strictly *worse than nothing* in exactly
     the case the sentence calls safe. Under the total reading both halves of the
     sentence fall out: no points, and a good ritual drops the total below 3 so the
     apparent age does not advance either.
- **The one-year seam is the text's, not a bug.** `start_age` (35, `:16565`) and
  `longevity_clamp.until_age` (35, `:16575`) are two numbers from two different
  sentences that happen to coincide. "Until he reaches the age of 35" stops the clamp
  *at* 35 while rolls are owed only from 36, so at exactly 35 a ritual-holder rolls
  **unclamped** and a character without one does not roll at all. `validate_aging_rules`
  deliberately does **not** gate `start_age == until_age`: asserting equality would
  invent a relationship the rules never state.
- **The load-time trust gate**: `longevity_clamp.max_total < outcomes[0].min`.
  `:16575` states the clamp's *purpose* — "no risk of actually aging" — and that is
  true if and only if the ceiling sits strictly below the first row that costs Aging
  Points. The shipped `9 < 10` is therefore an identity derivable from two
  sentences, not a number to transcribe and hope for (the same idiom as re-pricing
  the apprenticeship's `recommended_xp`). A `max_total: 10` against a table starting
  at 10 fails the load, naming `:16575`.

#### The Living Conditions table
> "| Living Conditions | Modifier |"

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16581-16594` — the header
  at `:16581-16582`, the ten rows at `:16583-16592`, the footnote at `:16594`.
- **Data**: `rules/core/aging.json` → `living_conditions`, ten rows, each carrying
  its own one-line `source`. Ids are sorted canonically in the file, so the file
  order is *not* the book's; the `source` line is what pairs a row with its
  rulebook line:

  | id | modifier | `:line` | cumulative |
  |---|---|---|---|
  | `living_condition.wealthy_or_healthy_location` | +2 | `:16583` | |
  | `living_condition.typical_summer_or_autumn_covenant_magus` | +2 | `:16584` | |
  | `living_condition.typical_summer_or_autumn_covenant_mundane` | +1 | `:16585` | |
  | `living_condition.typical_spring_or_winter_covenant_magus` | +1 | `:16586` | |
  | `living_condition.average_peasant` | 0 | `:16587` | |
  | `living_condition.live_in_a_leper_colony` | -1 | `:16588` | yes |
  | `living_condition.work_in_a_bad_air_trade` | -1 | `:16589` | yes |
  | `living_condition.work_in_a_mine` | -1 | `:16590` | yes |
  | `living_condition.poor_or_unhealthy_location_typical_town` | -2 | `:16591` | yes |
  | `living_condition.leper` | -2 | `:16592` | yes |

  Names live in `rules/i18n/{en,de}/aging.json`, keyed by id. The German names come
  from the **rulebook body** at the mirrored `:16583-16592`, not from the glossary —
  see *Translation-table notes* below. Guarded by
  `shipped_aging_table_carries_the_16583_to_16611_rows` and
  `english_and_german_i18n_cover_all_living_conditions` (`data_integrity.rs`).
- **Stored as choices, never as a resolved integer**: `Entity.living_conditions:
  BTreeSet<Id>` (`serde(default, skip_serializing_if)`, additive — no schema bump of
  its own). A *set*, because the five asterisked rows stack; a `BTreeSet`, so it is
  canonical by construction and `Entity::normalize()` needs no line for it. An
  integer could not answer "which conditions?" for the sheet, and a covenant will
  hand over *ids* in a later milestone.
- **An empty set is the table's baseline, not an unfinished entry**: the table prints
  "Average peasant 0" (`:16587`), so a character naming no condition already has a
  modifier of exactly 0. Nothing prompts him to pick one, and
  `validate_aging_rules` deliberately does **not** gate "exactly one zero-modifier
  row" — the baseline row is a UI affordance, not a rule.
- Implementation: `aging::living_conditions_modifier(entity, ruleset) ->
  LivingConditionsModifier { rows, from_table, from_traits, total }`, split rather
  than a bare integer so the sheet can show how much of the figure is circumstance
  and how much is Virtues and Flaws. An id the table does not carry contributes
  nothing (the engine has **one evaluation path**, so a computation never refuses to
  produce a number) and is reported as a finding instead.

#### Living Conditions are alternatives unless marked cumulative
> "\* Modifiers marked with an asterisk are cumulative with each other."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16594`; the table itself
  at `:16581-16592`. The footnote is only worth writing because the *unmarked* rows
  are not cumulative: they name mutually exclusive situations (`:16583` "Wealthy, or
  healthy location" against `:16587` "Average peasant"; the four covenant rows are
  graded alternatives for one covenant), so at most one of them applies.
- **Data**: `rules/core/aging.json` → `living_conditions[].cumulative`, `true` on
  exactly the five asterisked rows (`:16588-16592`).
- Implementation: `validation/aging.rs::report_living_conditions` emits
  `living_conditions_conflict` (error, args `condition` / `other`) once when more
  than one non-cumulative row is chosen, naming the first two offenders in canonical
  id order — three exclusive rows are one mistake to fix, not three. The same
  function emits `unknown_living_condition` (error, arg `condition`) per chosen id
  the table does not carry, because `living_conditions_modifier` deliberately skips
  an unresolvable id, which would otherwise make the AGING TOTAL silently wrong.

#### The Aging Roll table
> "| Aging Roll | Result |"

> "If an Aging Point 'in any Characteristic' is gained, the player may choose the
> Characteristic."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16597-16611` — the header
  at `:16597-16598`, the two apparent-aging rows at `:16599-16600`, the eleven
  effect rows at `:16601-16611`; the player's choice at `:16615`.
- **Data**: `rules/core/aging.json` → `outcomes`, eleven rows, each with its own
  `source` line and a serde-tagged `effect` so an unknown kind is a **load**
  failure rather than a silently ignored row:

  | totals | effect | `:line` |
  |---|---|---|
  | 10-12 | `any_characteristic`, 1 point | `:16601` |
  | 13 | `next_decrepitude_level_and_crisis` | `:16602` |
  | 14 | `named_characteristics`, 1 point, `[qik]` | `:16603` |
  | 15 | `named_characteristics`, 1 point, `[sta]` | `:16604` |
  | 16 | `named_characteristics`, 1 point, `[per]` | `:16605` |
  | 17 | `named_characteristics`, 1 point, `[pre]` | `:16606` |
  | 18 | `named_characteristics`, 1 point, `[str, sta]` | `:16607` |
  | 19 | `named_characteristics`, 1 point, `[dex, qik]` | `:16608` |
  | 20 | `named_characteristics`, 1 point, `[com, pre]` | `:16609` |
  | 21 | `named_characteristics`, 1 point, `[int, per]` | `:16610` |
  | 22+ | `next_decrepitude_level_and_crisis` (open-ended) | `:16611` |

  The book's **"Prs"** (`:16606`, `:16609`) is `Characteristic::Pre`, slug `"pre"`.
  A named row gives **each** Characteristic it names a point — "1 Aging Point in Str
  and Sta" (`:16607`) is one point each, not one divided between them.
- **The "3 or more" row is a threshold, not a row.** `:16599-16600` ("2 or less — No
  apparent aging" / "3 or more — Apparent age increases by one year") **overlaps**
  every effect row: a 20 both ages the appearance and costs two Characteristics a
  point. Modelling it as two more table rows would make the rows non-exclusive and
  let the table disagree with itself, so it ships as a single number,
  `apparent_age_increase_min: 3`, asked of every total. `:16577` states it as a
  threshold in prose anyway ("Particularly low rolls … **Otherwise** …").
- Implementation: `aging::resolve_outcome(entity, ruleset, total) ->
  Option<AgingOutcome { total, apparent_age_increases, awards, crisis }>`, with
  `AgingPointAward { target, points }` over `AgingPointTarget::{Named, PlayerChoice,
  NextDecrepitudeLevel}` — three variants because the UI has three genuinely
  different questions to ask, and the exhaustive `match` makes a new kind of row a
  compile error until every reader has decided what to do with it. It reads and
  writes nothing. A total below the table's first row is `Some` with no awards: that
  is a result, not a missing one.
- **It needs the character** because rows 13 and 22+ ask for "sufficient Aging Points
  (in any Characteristic**s**) to reach the next level in Decrepitude" — a count the
  table never prints. `points_to_next_decrepitude_level` derives it: Decrepitude
  "increases as an Ability" (`:16617`), so the count is the advancement curve's price
  for the next score less the points already accrued, floored at 1 (sitting exactly
  on a boundary must still cost *something*). `points: None` **only** when the curve
  cannot price that score (`AdvancementTable` tops out) — reported as unpriceable,
  never silently costed at 0.
- **"In any Characteristic*s*" is plural** (`:16602`, `:16611`), so the points may be
  spread. Reaching Decrepitude 1 costs 5 points, and forcing them all into one
  Characteristic would force drops the player may legally avoid — which is why the
  writer takes a per-Characteristic **map**, not a single pick.
- **Load-time contiguity gate** (`Ruleset::validate_aging_rules`): the rows must tile
  the integers with no gap, no overlap, ascending `min`, and exactly one open-ended
  row which must be last; each row must award at least 1 point and name each
  Characteristic at most once; `apparent_age_increase_min <= outcomes[0].min`. A
  dropped row would otherwise degrade silently to "resolves to no effect". It is
  **contiguity**, deliberately not "must cover 10..=21" — hardcoding the shipped
  table's numbers in Rust would violate *catalogue size is data*. Row-by-row
  resolution against the shipped data is asserted by
  `the_shipped_aging_table_resolves_each_row_of_16599_to_16611`
  (`data_integrity.rs`).

#### Applying a year — `resolve_year`, and undoing it — `revert_year`
> "Aging points are accumulated in each Characteristic."

> "Every Aging Point also counts as an experience point towards Decrepitude, which
> increases as an Ability."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16567-16617`; `:16579`
  (the points accumulate), `:16577` (the apparent age), `:16615` (the player's
  choice), `:16617` (Decrepitude follows the points).
- Implementation: `aging::resolve_year(entity, ruleset, &AgingYearRequest { age, die,
  distribution }) -> Result<AgingYearResult { entity, total, outcome }, AgingError>`
  — the **single writer** in the subsystem, shaped on `apply_childhood_package`. It
  computes the year's `AgingTotal`, reads it with `resolve_outcome`, and returns a
  **new** entity: the awarded points added to `Entity.aging_points`, the apparent age
  advanced if the total cleared the threshold, and a structured `AgingLogEntry`
  appended. Nothing is mutated in place, so a refused year leaves the caller's
  character exactly as it was.
- **What it never does.** It never **rolls** (no `rand` dependency, ever — the die is
  the player's). It never **kills**: a row carrying a Crisis sets
  `AgingOutcome.crisis` and stops. And it never touches **Decrepitude**, which stays
  derived from `aging_points` through `decrepitude_score` (`:16617`) and is never
  written down beside them.
- **Every refusal is a refusal to write**, never a silent adjustment: `AgingError::{
  NoAgingRules, YearAlreadyRecorded, DistributionMismatch, DistributionNotOpen,
  AwardUnpriceable, YearNotRecorded }`. The distribution must sum to exactly the
  points the row left open, and a row that names its own Characteristics accepts no
  distribution at all. Plain data — no issue codes, no Fluent keys — like
  `ChildhoodRejection`.
- **Seeding the apparent age.** `Entity.apparent_age` is `None` on most characters.
  The first year that needs it seeds it at `start_age` (35 — nothing has happened to
  the appearance before the first owed roll, `:16577`) and every year after only
  increments. Seeding once and incrementing thereafter is what makes the result
  **order-independent**: a catch-up applied out of order lands on the same apparent
  age. A hand-entered apparent age is never re-seeded, only advanced.
- `aging::revert_year(entity, ruleset, age) -> Result<Entity, AgingError>` is the
  **exact** inverse: it subtracts precisely the points the entry recorded, steps the
  apparent age back iff that entry advanced it, and removes the entry. Exact rather
  than recomputed, because the conditions and ritual bonus that produced the roll may
  legitimately have changed since. A 25-roll pre-play catch-up with no undo is not
  shippable.
  - It addresses entries **by `age`**, since a character with no `birth_year` has no
    calendar year to be addressed by. A **legacy free-text entry carries no age** and
    is therefore out of its reach by construction; it is removed with the ordinary
    log editor, which is the right home for it — there is nothing mechanical to undo.
  - **Documented edge case**: undoing the seed clears `apparent_age` again when the
    decrement lands back on `start_age`, so a **hand-entered apparent age of exactly
    35 comes back unset**. The two states carry the same fact and the log entry
    cannot tell them apart. Under a ruleset with no aging rules there is no start age
    to compare, so the decremented figure simply stays.
  - Reverting an age no entry records is `Err`, never a silent no-op: a revert that
    quietly did nothing would tell the player the year had been undone.
- **The log records the total it computed**, which is a historical record of a roll
  and not a cached derivation — so it does not offend *saves store choices, not
  resolved values*. `AgingLogEntry.effect` is left empty by the writer: the
  structured fields *are* the record, and the prose is the player's to add.

App/UI: three thin commands wrap the above — `aging_preview(entity, age, die)`,
`aging_apply(entity, age, die, distribution)` and `aging_revert(entity, age)`
(`arm-app/src/commands.rs` → `ruleset_io::aging_{preview,apply,revert}_loaded`),
modelled on `apply_childhood_package`. The outcome resolution **cannot** live in
JS: the die is player input the entity must not store, and a stress die explodes,
so no bounded lookup table could stand in for the engine.
`ui/src/lib/components/AgingRollCalculator.svelte` is the surface over them (mounted
in `AgingPanel.svelte`, so the editor's Aging tab and the guided aging step share
it): the
die input carries `min="0"` and deliberately **no `max`** (`:16567`'s exploding
stress die), the total is shown broken into the terms `AgingTotal` already reports,
and the distributor renders one number input per Characteristic and refuses to
submit until it sums to exactly the points the row left open — the UI half of
`:16602`/`:16611`'s plural "in any Characteristic**s**". The year and die live in
`AppStore.agingDraft`, UI-only `$state` on the `childhoodDraft` precedent with its
own debounce and sequence guard, which is what keeps the calculator from dirtying
the document (`dirty` is a snapshot compare of the entity).

**Localizing `AgingError`** follows the `ChildhoodRejection` precedent exactly, and
that choice is deliberate. `AgingError` is plain data with no `Display`; the six
variants become ordinary `issue-*` findings through
`validation::aging_error_issue`, with contract-table rows and Fluent keys in both
locales — `aging_rules_missing`, `aging_year_already_recorded` (`age`),
`aging_distribution_mismatch` (`owed`, `distributed`), `aging_distribution_not_open`
(`count`), `aging_award_unpriceable`, `aging_year_not_recorded` (`age`), all errors,
all on the `aging` phase — like every other finding `validation/aging.rs` emits,
because the year, die and distribution they describe are typed on the aging step and
nowhere else. Several are errors, so the attribution is what lets that step block
Next on its own broken input. A refusal therefore crosses the IPC
edge as an ordinary `Ok` outcome (`status: "rejected"`) carrying issues, never as an
`AppError`: it describes the form the player just submitted, and the frontend
renders it through the `issue-<code>` path it already has, so no English prose
crosses the boundary. They are **command-input** findings — the engine writes
nothing when it refuses, so no saved character can hold one for `validate` to find —
which is why the emit site is a mapping function rather than a `validate` pass, and
why it still lives in `validation/` where the contract scanners can see it.
`aging_distribution_not_open` deliberately carries a **count**, not the
Characteristics: those would reach the message as slugs, and a slug is never shown
to a user.

#### Apparent age cannot outrun actual age
> "Otherwise, the character's apparent age increases by one year."

> "You may choose your apparent age freely, although if you are basically human it
> should be less than or equal to your actual age."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:16577` (one year per
  year), `:5189` (Unaging's aside).
- Implementation: `validation/aging.rs::report_apparent_age` emits
  `apparent_age_above_age` (**warning**, args `apparent_age` / `age`) when both are
  entered and the apparent age is higher. A warning, not an error: `:5189` states it
  as a *should* and explicitly excuses a character who is not basically human.

#### The three-way aging-immunity split — `no_aging` and `no_apparent_aging` are orthogonal
> "In game terms, your aging points do not decrease your Characteristics, only
> building up to give you Decrepitude points. … You may choose your apparent age
> freely, although if you are basically human it should be less than or equal to
> your actual age."

> "This Flaw also includes the effects of the Unaging Virtue, but the character's
> apparent age advances in line with their physical age."

> "Bee Kings do not appear to age after reaching maturity, but every Bee King not
> killed by circumstances dies of a rapid illness precisely a century after birth."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:5189` (Unaging), `:5743`
  (Bound to (Role)), `:3488` (Bee King).
- **The sources describe two independent facts**, and the shipped data used to
  collapse them into one `no_aging` tag — which made the Bee King entry simply
  wrong. Bound to (Role)'s explicit *but* at `:5743` is the sentence that proves
  they are separable at all:

  | item | Characteristics do not drop | apparent age does not advance | tags |
  |---|---|---|---|
  | `virtue.unaging` (`:5189`) | yes | yes | `no_aging` + `no_apparent_aging` |
  | `flaw.bound_to_role_role` (`:5743`) | yes | **no** | `no_aging` |
  | `virtue.bee_king` (`:3488`) | **no** | yes | `no_apparent_aging` |

- **Data**: `rules/core/virtues_flaws.json`. `AgingEffect::NoApparentAging` is the
  additive new variant (Fluent `derived-detail-no_apparent_aging`, en/de);
  `virtue.bee_king` was **re-tagged** from `no_aging` to `no_apparent_aging` alone,
  `virtue.unaging` gained the second tag, `flaw.bound_to_role_role` keeps `no_aging`
  only. Guarded by `the_three_aging_immunities_ship_their_two_facts_separately`
  (`data_integrity.rs`).
- Implementation, one reader each:
  - `effective/warping.rs::aging_drops` (and the surfaced `characteristic_aging_drops`
    in `effective/characteristic.rs`,
    which wraps it) return 0 for a `no_aging` carrier, so no Characteristic ever
    drops and `effective_characteristic_after_aging` leaves the bought score alone.
    **This is new in 6b6**: `aging_drops` ignored `:5189` until the engine started
    awarding the points, and both functions gained a `&Ruleset` parameter for it —
    the slice's only public-signature change.
  - `decrepitude_points_total` / `decrepitude_score` stay deliberately **untouched**:
    "only building up to give you Decrepitude points" is the other half of the same
    sentence, so a carrier reaches Decrepitude at everyone's rate.
  - `aging.rs::resolve_outcome` reads `no_apparent_aging`, so a carrier's
    `apparent_age_increases` is false at every total and `resolve_year` neither seeds
    nor advances his apparent age — one decision, read once rather than made twice.

#### Strong Faerie Blood's -3 to Aging Rolls
> "First, you have natural longevity. You start making aging rolls at the age of
> fifty, rather than the normal 35, and get –3 to Aging Rolls, cumulative with any
> other bonuses."

- Source: `Ars Magica - Definitive Edition (Core Rules).md:5036`.
- **Data**: `rules/core/virtues_flaws.json` → `virtue.strong_faerie_blood` gains
  `{"type":"aging_mod","kind":"aging_roll","amount":-3}` beside its existing
  `grants_selection`. The item carried **no** `aging_mod` at all before this slice;
  harmless while nothing consumed the modifiers, a wrong number in a shipped
  character the moment `aging_total` started reading them.
- Guarded by `strong_faerie_blood_lowers_the_aging_total_by_three`
  (`data_integrity.rs`). Its **start-at-fifty** half is *not* implemented — see
  *Recorded gaps*.

#### `SCHEMA_VERSION` 14 → 15
`AgingLogEntry` widened from `{ year: i32, effect: String }` into the full record of
a resolved aging year — `age`, `die`, `total`, `living_conditions`, `points`,
`apparent_age_increased`, `crisis` — and **`year` became `Option<i32>`**, because a
character with no `birth_year` has no calendar year to write and `age` is the key
the schedule matches on.

Every added field is `serde(default, skip_serializing_if)` and serde reads a bare
`year` number into `Some`, so a schema-14 save loads unchanged and **no migration
code exists**. What is *not* compatible is the **forward** direction: a schema-15
save may omit `year` entirely, which a schema-14 reader rejects — and a version
number is exactly how an older build learns not to try. That is why this one earns a
bump where `Entity.living_conditions` (purely additive) did not. The history line
lives on the `SCHEMA_VERSION` constant in `types.rs`; the frontend mirror in
`ui/src/lib/state.svelte.ts` is pinned equal to it by test.

Knock-on: `export.rs` now prints an **undated** log entry as a plain bullet rather
than an empty bold label (`an_undated_aging_log_entry_prints_its_effect_without_a_year_label`)
— a consequence of the optional `year`, not a formatting preference. The Markdown
sheet also carries the two new records: `Doc::write_living_conditions` opens the
annotation block with the chosen `Entity.living_conditions`, localized through
`rules/i18n/<lang>/aging.json` (never the slug), because they are a **stored choice**
and a standing term of the aging total (`:16567-16569`, table `:16581-16594`); and
`Doc::aging_log_entry` appends each logged year's `die` and `total` (`:16567`) after
its free text, so an exported sheet records what produced the year's outcome. The
resolved conditions *modifier* is deliberately not printed — it is derived from the
ids plus the character's Virtues and Flaws, and the sheet records choices.

#### Recorded gaps — deliberately not implemented in 6b6
Each is findable here so it is not rediscovered later as a bug. Several were closed by
6b7 and say so in place, rather than being deleted: a closed gap is worth more as a
record of *how* it closed than as a blank. As of the end of milestone 6 the ones still
genuinely open are **Strong Faerie Blood's start-at-fifty**, **Might-holders' immunity
to aging**, **Age Quickly / Baneful Circumstances' schedule rules**, **`:16575`'s
discretionary closing clause**, and **crisis survival itself** — the Stamina roll, the
doctor's Medicine roll and death, all out of scope by design.

- **Strong Faerie Blood's start-at-fifty** (`:5036`). "You start making aging rolls
  at the age of **fifty**, rather than the normal 35" — the **-3 ships**, the altered
  start age does not. A carrier is still scheduled from `first_roll_age()`. It would
  need a per-trait override of `AgingRules::start_age`, machinery no other shipped
  item asks for. Recorded on `aging_schedule`'s doc comment as well.
- **Might-holders' immunity to aging**, which `:5689` presupposes — "he must make an
  additional Aging roll even if he is normally immune to aging because of a Longevity
  Ritual **or Might Score**" — and which mythic companions can hold
  (`entity.might`). The clause states the immunity in passing rather than granting
  it, so no rule is implemented from it; the schedule takes no notice of `might`.
- **The Bronze Cord** (`:10844`, "and to rolls to resist aging") — *no longer a gap;
  delivered in 6b7.* It is still deliberately **not** folded into the aging total:
  an aging roll is not a roll one passes or fails, so the referent of "resist aging"
  is the *crisis survival* roll, and `:16636` ("Virtues that affect aging rolls do
  not affect crisis survival rolls") keeps the two roll families apart on purpose.
  The slice that revisited it against the crisis rules is 6b7, and it landed on that
  reading: the cord reaches `crisis_survival`'s `modifier_total` as a
  `CrisisModifierSource::BronzeCord` term (`aging.rs`), through the shared
  `derived::bronze_cord_bonus` accessor so the +5 maximum of `:10836` keeps its one
  home, and it moves no term of `aging_total`.
  `the_bronze_cord_reaches_crisis_survival_and_never_the_aging_total` (`aging.rs`)
  asserts both directions in one test, and
  `virtues_that_modify_aging_rolls_do_not_affect_crisis_survival_rolls`
  (`data_integrity.rs`) locks the converse against the shipped catalogue.
- **Age Quickly** (`:5661`) and **Baneful Circumstances** (`:5689`) both ship
  `aging_mod` amount **0**, deliberately. Their real mechanics are *schedule* rules —
  a doubled rate and effective age ("you make two aging rolls each year"), and a
  conditional extra roll — not modifiers to any one roll's total, and this slice
  implements neither schedule. Both stay visible in the surfaced-modifier read-out.
  `age_quickly_contributes_nothing_to_the_total_and_stays_surfaced`
  (`data_integrity.rs`) is a regression test whose doc comment exists so nobody
  "fixes" the 0 into a number.
- **`:16575`'s closing clause** — "At the player's and storyguide's discretion, this
  may also apply to characters with modifiers to the aging roll from other sources."
  Explicitly discretionary, so the clamp is **not** extended to non-ritual modifier
  holders unilaterally; `aging_total` gates it on `longevity_ritual.is_some()`.
- **The Crisis** (`:16619-16632`) — *no longer a gap; delivered in 6b7.* What 6b6 left
  as a flag `resolve_year` stopped at is now read, resolved and recorded. The serde
  shape (`CrisisRules` /
  `CrisisRow` / `CrisisOutcome` / `CrisisSeverity` / `CrisisDie` / `CrisisAttendant`
  in `aging.rs`, plus `AgingRules.crisis` and the two Decrepitude thresholds of
  `:16617` — all optional, so an aging block with no crisis key loads unchanged);
  the **shipped data** below and the load gates of `validate_crisis_rules`; the
  CRISIS TOTAL of `:16621` with `:16619`'s Decrepitude-first ordering; the survival
  read-out of `:16628-16638` with the doctor's allowance (`:16634`) and `:16636`'s
  wall; the table look-up (`resolve_crisis_row`); `crisis_preview`, which composes all
  four into one reading; and the **write-back** — `resolve_year`'s Crisis leg with
  `:16619`'s ordering, the four fields a resolved Crisis records on `AgingLogEntry`,
  and `:16573`'s spent Longevity Ritual as an `AgingNote`. The subsections at the end
  of this section carry the provenance for each.
  Survival itself stays out for good: the engine gives the total, the row, the Ease
  Factor and the Creo Corpus level, but never rolls the Stamina die, never resolves
  the attendant's Medicine roll, and never kills — so the `fatal_decrepitude_score`
  of `:16617` ships as a datum nothing in the engine acts on. Two consequences of the
  write-back reached the app edge in **6b7c**: `aging_apply` now carries the Simple
  Die through to `resolve_year`, and `aging_preview` answers with the Crisis the
  year *would* write. The preview does that by resolving the year in memory and
  keeping only the reading — never by calling `crisis_preview` on the character as
  it stands, which `:16619` makes one Decrepitude short (the year's own award is
  the increase that comes first), so a bare read would show 14 where the log then
  records 15. The Markdown sheet caught up in the same slice: `export.rs`'s
  `aging_log_crisis` prints the row (through the rules i18n), its severity (through
  `crisis-severity-<slug>`, now declared in `LABEL_KEYS`), the Simple Die and the
  CRISIS TOTAL — and says in words when a Crisis is owed and unrolled, which is a
  third state the sheet could not previously tell from no Crisis at all. The golden
  fixture carries one of each.
- **Leprosy's Heavy Wound at a Crisis** (`:6340`) — "whenever she undergoes an Aging
  Crisis (page 392) the leper sustains a Heavy Wound in addition to any other result".
  *No longer a gap; the note landed in 6b7c* as `AgingNote::HeavyWound`, emitted by
  `resolve_year`'s note leg when `carries_crisis_heavy_wound` finds the marker on any
  selection. Still **told, never written**: the engine records no wound, because the
  health track is the player's to mark and a marker with no number is not a quantity a
  writer could apply — and an invented wound would be one `revert_year` could not take
  back off, since the entry records no such thing. It is a **predicate**, not a sum:
  two carriers of the marker still cost one note. Like the spent ritual it follows the
  **Crisis**, not the Crisis roll, so an unrolled Crisis carries it too; "in addition
  to any other result" is why both notes stand on one year, which
  `a_crisis_costs_a_leper_a_heavy_wound_and_says_so_rather_than_writing_one`
  (`aging.rs`) pins alongside the byte-identical revert. Rendered through
  `aging-note-heavy_wound` in both locales.
- **Two `AgingEffect` kinds and the two shipped items that carry them (M6/6b7).**
  `crisis_survival` (a modifier to the crisis *survival* roll) and
  `crisis_heavy_wound` (a marker; `amount` ignored, shipped as **0**, because a Heavy
  Wound is a consequence and not a number) are two kinds rather than one, so the
  stored `amount` never means two different things depending on the carrier. Neither
  reaches the AGING TOTAL — `aging_total`'s no-op arm — and both are surfaced under
  `derived-detail-crisis_survival` / `derived-detail-crisis_heavy_wound`. The two data
  fixes, in `rules/core/virtues_flaws.json`:
  - **`virtue.mild_aging`** gains `crisis_survival +3` beside its existing
    `living_conditions +1`. "The character's aging rolls benefit from a +1 bonus to
    the Living Conditions Modifier … Furthermore, he receives a +3 bonus to rolls to
    survive an aging crisis." (`:4530`) — **one sentence, two mechanics, two
    destinations.** This is the proof case for `:16636`, "Virtues that affect aging
    rolls do not affect crisis survival rolls": that general prohibition governs the
    +1, which is an aging-roll modifier and so stays out of the crisis, while the +3
    is a *specific* grant to the survival roll and survives the general rule. Guards:
    `mild_aging_carries_both_halves_of_4530` and
    `a_crisis_survival_modifier_never_reaches_the_aging_total` (`data_integrity.rs`),
    the latter being `:16636`'s converse — a survival-roll grant is not an aging-roll
    modifier either.
  - **`flaw.leprosy`** gains `crisis_heavy_wound 0` beside its `living_conditions -2`:
    "whenever she undergoes an Aging Crisis (page 392) the leper sustains a Heavy
    Wound in addition to any other result" (`:6340`). Guard:
    `leprosy_carries_its_crisis_wound_beside_its_living_conditions_penalty`.
- **Covenant-derived Living Conditions.** The four covenant rows are chosen by hand
  today; a covenant will hand over ids in a later milestone.

#### The Crisis Table — shipped values (M6/6b7, data only)

> "| Crisis Roll | Result |" … "| 8 or less | Bedridden for a week |"
> — Ars Magica - Definitive Edition (Core Rules).md:16624-16632

This subsection records where the *numbers* in `rules/core/aging.json` come from,
because JSON carries no comments — plus the two items below, which are provenance the
load gates deliberately do **not** encode. The mechanics that read them — the CRISIS
TOTAL of `:16621`, the table look-up, the survival read-out and the refusals — are the
`#####` subsections that follow, all delivered in 6b7. (This paragraph promised them as
a placeholder until 6b8d noticed they had arrived.)

| Datum | Value | Source line |
|---|---|---|
| `frail_decrepitude_score` | 4 | `:16617` |
| `fatal_decrepitude_score` | 5 | `:16617` |
| `crisis.die` | 1–10 (Simple Die) | `:474` |
| `crisis.attendant` | `ability.medicine`, `int`, EF 6, botch `-3` | `:16634` |
| `crisis.bedridden_week` | ≤ 8, bedridden | `:16626` |
| `crisis.bedridden_month` | 9–14, bedridden | `:16627` |
| `crisis.minor_illness` | 15, EF 3, CrCo20 | `:16628` |
| `crisis.serious_illness` | 16, EF 6, CrCo25 | `:16629` |
| `crisis.major_illness` | 17, EF 9, CrCo30 | `:16630` |
| `crisis.critical_illness` | 18, EF 12, CrCo35 | `:16631` |
| `crisis.terminal_illness` | 19+, **no EF**, CrCo40 | `:16632` |

Three things a later sweep must not undo:

1. **`crisis.rows` ships in BAND order, not id order — a deliberate exception to the
   project's canonical-serialization rule.** The rows must tile the integers
   contiguously with the open-below row (`:16626`) first and the open-above row
   (`:16632`) last, which is what the load gates check and what the look-up walks. An
   id-alphabetical sort leaves every value intact and silently breaks all of it.
   Everything else in the file — `living_conditions` above all — stays id-sorted.
   `shipped_crisis_table_carries_the_16626_to_16632_rows` (`data_integrity.rs`)
   asserts the band order so the exception has a test, not just a paragraph.
2. **Terminal carries no `ease_factor` at all.** `:16632` offers no Stamina roll —
   "CrCo40 required to survive" — so the field is absent rather than set to an
   unbeatable number. `Option<i32>` says "no roll"; a 99 would say "roll and lose".
3. **`botch_penalty` ships signed (`-3`)**, matching "the character must subtract 3
   from the survival roll" (`:16634`) as the roll takes it: the survival read-out
   **adds** it. (The doc-vs-data disagreement recorded here while the field was
   unread — the doc comment then described a positive magnitude — was settled in
   favour of the signed datum when the survival read-out landed. The field's doc
   comment now states `-3` as stored-and-added, the one sign convention this file
   keeps.)

##### The `+5` between the table and the Creo Corpus guidelines

> "| 15 | … • Resolve a minor aging crisis |" … "| 35 | … • Resolve a terminal aging
> crisis … |"
> — Ars Magica - Definitive Edition (Core Rules).md:13372-13376

The Creo Corpus guidelines price a minor / serious / major / critical / terminal
aging crisis at **15 / 20 / 25 / 30 / 35**, exactly **5 below** the Crisis Table's
own **20 / 25 / 30 / 35 / 40** (`:16628-16632`) — the `+1` Touch magnitude of a
Ritual cast on someone other than the caster. Recorded here so nobody "fixes" one
table against the other: **both columns are transcribed correctly**, and the offset
is real.

It is provenance, **not a gate**. The guidelines are not loaded data, and the `+5`
is an inference rather than a sentence the rulebook states, so
`validate_crisis_rules` does not check it.

##### Gates deliberately not written on the Crisis Table

`validate_crisis_rules` (`ruleset/integrity.rs`) carries the full list in its doc comment;
the two that touch *rules numbers* are recorded here as well, because this file is
where a reader looks for why a shipped number is not enforced.

1. **The `+3` Ease-Factor and `+5` Ritual-level steps.** The shipped columns do step
   by exactly 3 and 5 down `:16628-16632`, but the rulebook never states that
   relation. Gating it would refuse a legitimate house table, and it is the same
   class of invention as `start_age == longevity_clamp.until_age`, which slice 6b6
   rejected for the aging block. What *is* gated is the direction the source does
   state — "The level of spell required depends on the severity of the crisis, as
   noted on the table" (`:16638`) — so severity, required Ritual level and Ease
   Factor must each climb strictly down the illness rows, by any step.
2. **`ritual_level == guideline + 5`.** See the offset above: an inference, not a
   stated rule, and one of the two tables is not loaded at all.

##### The CRISIS TOTAL adds the Decrepitude **that year** raised

> "**Crisis:** Increase the character's Decrepitude first, and then roll on the Crisis
> Table." … "**CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score**"
> — Ars Magica - Definitive Edition (Core Rules).md:16619, :16621

`crisis_total` (`aging.rs`) adds those three terms and no others. Its Decrepitude term
is **as of the crisis year**, not the character's score today: `:16619` puts the
year's own increase *first*, and the aging module is deliberately order-independent
(`resolve_year` refuses nothing but a year already recorded), so a player may roll 36,
carry on through 37-40, and resolve 36's Crisis afterwards. A live
`effective::decrepitude_score` read would charge that Crisis with four later years'
Aging Points. `decrepitude_points_as_of` therefore takes the lifetime total
(`:16617` — "Every Aging Point also counts as an experience point towards
Decrepitude") less the points of every `aging_log` entry whose `age` is **strictly**
greater, so the crisis year's own award stays in and an undated legacy entry
subtracts nothing. `the_crisis_total_reads_the_decrepitude_that_year_raised_not_todays`
is the regression test.

No trait modifier reaches this total: `:16621` names three terms, and `:16636` —
"Virtues that affect aging rolls do not affect crisis survival rolls" — refuses the
analogy with `aging_total` for the roll that follows. The Simple Die's 1–10 range
(`:474`) is an input affordance a UI applies; the engine totals whatever die it is
given rather than refusing one.

##### Which row a CRISIS TOTAL lands on — `resolve_crisis_row`

> "| Crisis Roll | Result |" … "| 8 or less | Bedridden for a week |" … "| 19+ |
> **Terminal illness**. CrCo40 required to survive. |"
> — Ars Magica - Definitive Edition (Core Rules).md:16624-16632

`resolve_crisis_row` (`aging.rs`) is the Aging Roll table's `resolve_outcome` twin,
and deliberately the simpler of the two. An aging row means nothing until it is read
against the character — "sufficient Aging Points … to reach the next level in
Decrepitude" (`:16602`) is a count the table does not print — whereas a crisis row
already says everything it does. So the look-up takes no `Entity` and returns the
`CrisisRow` itself: the caller needs the **id** as much as the `CrisisOutcome`,
because the row's display text ("Bedridden for a week") lives in
`rules/i18n/<lang>/aging.json` keyed by that id and never in the engine.

Band membership is `CrisisRow::covers`, inclusive at both ends, with an absent bound
meaning an open end — below for `:16626`, above for `:16632`. Because
`validate_crisis_rules` refuses at load any table that leaves a gap or an overlap
between those two open ends, a `None` from a ruleset that loaded means the ruleset
ships **no Crisis Table**, never that the total fell off the table.
`a_crisis_total_lands_on_the_row_whose_band_covers_it` (`aging.rs`) walks both open
ends and every band between them.

##### One Crisis read whole — `crisis_preview`

> "**Crisis:** Increase the character's Decrepitude first, and then roll on the Crisis
> Table." … "Creo Corpus magic can postpone a crisis, or resolve it if cast as a
> Momentary Ritual."
> — Ars Magica - Definitive Edition (Core Rules).md:16619, :16638

`crisis_preview` (`aging.rs`) composes `crisis_total`, `resolve_crisis_row` and
`crisis_survival` into the one value a caller needs from `(entity, ruleset, age,
die)`: the CRISIS TOTAL with its three terms, the **id** of the row it lands on, that
row's `CrisisOutcome`, and the survival read-out where one applies. Each of the three
stays the single home of its own rule; what the composition adds is the *pairing* —
the row is looked up against the total this call computed and the survival read-out
against the outcome that row carries, so no caller can pair a total with the wrong
row. The row travels as an id because its text ("Bedridden for a week") is i18n data,
never engine prose.

`survival` is `None` for `CrisisOutcome::Bedridden` (`:16626`, `:16627`): a week or a
month in bed is time, not a roll, and an empty read-out would read as "survivable on
a 0".

It **writes nothing**. `resolve_year` remains the aging subsystem's single writer;
`:16619`'s "increase the character's Decrepitude first" is honoured by *reading* the
score as of the crisis year (`decrepitude_points_as_of`), not by raising anything
here. `the_crisis_preview_composes_the_total_the_row_and_the_survival_roll`
(`aging.rs`) asserts the character is byte-identical after a preview, and
`a_crisis_preview_leaks_no_aging_roll_modifier_into_either_half` re-pins `:16636`
through the composed path — a read-out holding the total and the survival roll in one
value is a second place an aging-roll modifier could leak into either.
`the_shipped_crisis_table_answers_a_total_end_to_end` (`data_integrity.rs`) walks the
same path against the **shipped** `rules/core/aging.json` through the crate's public
surface, which is where the attendant of `:16634` actually ships.

##### The Crisis leg of the single writer — `resolve_year`

> "**Crisis:** Increase the character's Decrepitude first, and then roll on the Crisis
> Table."
> — Ars Magica - Definitive Edition (Core Rules).md:16619

`resolve_year` (`aging.rs`) gains the leg that makes a Crisis part of the character.
`AgingYearRequest` grows `crisis_die: Option<i32>` and `AgingYearResult` grows
`crisis: Option<CrisisPreview>`; everything the leg reads is `crisis_preview`'s, so no
crisis rule is implemented twice.

- **The order is the rule.** A row carrying a Crisis awards its Aging Points like any
  other row, and **that award IS `:16619`'s increase**. The Crisis is then read off the
  *applied* character rather than the one who walked into the year, so the CRISIS TOTAL
  adds the Decrepitude this very year raised. The writer does nothing but order it: it
  is `crisis_preview(&applied, …)` and not `crisis_preview(entity, …)`, and
  `a_resolved_crisis_year_records_the_roll_the_row_and_its_severity` witnesses the
  difference (five points take a fresh character to Decrepitude 1, and the total adds
  the 1, not the 0 he started the year with).
- **It still never rolls and never kills.** Both dice are typed in; the heaviest row the
  table has hands back a living character with the Ease Factor, the Creo Corpus level
  and nothing else decided (`a_terminal_crisis_is_recorded_and_kills_nobody`).
- **A Crisis nobody has rolled is a state, not a refusal.** With no `crisis_die`, or
  under a ruleset shipping no Crisis Table, the year is still written and the log
  records the Crisis as **owed and unrolled** (`crisis: true`, no `crisis_row`). The
  aging roll happened whether or not the second die has been thrown, and refusing to
  record it would lose the one thing that did. This is deliberately **not** an
  `AgingError`: every existing variant describes a write the engine will not make, and
  here it makes one.
- **A Crisis die on a year the table sent to no Crisis resolves nothing**, and is not an
  error either — whether a Crisis happened is `:16602`/`:16611`'s call, never the
  player's, and nothing is written from the unused die
  (`a_crisis_die_never_invents_a_crisis_the_table_did_not_call_for`).
- **`:16636` gets a third pin**, and this is the worst of the three places it could
  leak: one call now computes the AGING TOTAL, which *does* take the trait modifiers,
  and the CRISIS TOTAL, which takes none, off **one** character — the shape that
  invites someone to reuse a modifier between them. A character wearing Faerie Blood,
  Poor Living Conditions and Mild Aging is driven through the whole write-back in
  `a_resolved_crisis_year_leaks_no_aging_roll_modifier_into_the_crisis` (`aging.rs`):
  the aging half takes the `-1`, the CRISIS TOTAL is still the three terms of
  `:16621`, and only Mild Aging's `+3` reaches the survival read-out the year hands
  back.
- **`revert_year` needed nothing**, and that is a claim rather than an omission: the
  Crisis is recorded inside the year's own entry and the entry is what the revert
  removes, so a Crisis year still restores the save byte for byte — the Longevity
  Ritual included, which costs nothing precisely because the leg reports it spent
  instead of deleting it
  (`reverting_a_resolved_crisis_year_leaves_the_character_byte_identical`).
- **Walked against the shipped catalogue**, not only against fixtures:
  `a_shipped_crisis_year_is_written_into_the_character_and_reverts_exactly`
  (`data_integrity.rs`) drives a companion of 40 through a 9 (`13`, the `:16602` row),
  five Aging Points to Decrepitude 1, a Simple Die of 10 and so a CRISIS TOTAL of 15 —
  the shipped minor illness with its Ease Factor 3, CrCo20 and the `:16634` doctor,
  which only the real `rules/core/aging.json` ships — and then back off byte for byte.
- App/UI: **the die reaches the engine from the command edge.** This bullet
  described a stopgap — `ruleset_io::aging_apply_loaded` passing `crisis_die: None`
  — that 6b7c removed in the same slice; both `aging_apply_loaded` and
  `aging_preview_loaded` now take `crisis_die: Option<i32>` from the
  `aging_apply` / `aging_preview` commands and hand it to `resolve_year`. A die
  given for a year the table sent to no Crisis is simply unused: whether a Crisis
  happened is `:16602`/`:16611`'s call, never the player's.

##### The Longevity Ritual a Crisis spends — reported, never deleted

> "A Longevity Ritual is effective until the character suffers a crisis. When the
> crisis occurs, the ritual assures that the character survives, but its power is
> spent, and the focal ritual must be performed again (see page 261)."
> — Ars Magica - Definitive Edition (Core Rules).md:16573

`AgingYearResult` gains `notes: Vec<AgingNote>`, and `AgingNote::LongevityRitualSpent`
is the one variant (`aging.rs`). A tagged enum rather than a message, on the
`AgingError` / `ChildhoodRejection` precedent: the engine hardcodes no user-facing
string, so the caller maps the variant through Fluent, and an exhaustive `match` makes
a second note a compile error at every reader until it has been rendered.

**`Entity.longevity_ritual` is not touched.** The ritual is a *stored choice* holding a
player-entered bonus and the focus that "must be repeated" if the ritual is performed
again (`:10668`); an engine that cleared it would destroy both, and would make the year
unrevertible into the bargain. Performing the focal ritual again is a season's work the
player records, not an inference the sheet makes. (The sentence's other half — "the
ritual assures that the character survives" — is likewise **not** implemented as an
automatic survival: the engine resolves no survival roll at all, for anyone.)

The note follows the **Crisis**, not the Crisis *roll*: "when the crisis occurs" is the
aging row's doing (`:16602`, `:16611`) and the Simple Die only decides how bad it was,
so a Crisis owed and unrolled spends the ritual too.
`a_crisis_spends_the_longevity_ritual_and_never_deletes_it` (`aging.rs`) pins all four
cases — rolled, unrolled, no Crisis, no ritual.

##### What a resolved Crisis records — and why `SCHEMA_VERSION` stays 15

> "**CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score**" … "| Crisis
> Roll | Result |"
> — Ars Magica - Definitive Edition (Core Rules).md:16621, :16624-16632

`AgingLogEntry` (`types.rs`) widens by four fields, all
`serde(default, skip_serializing_if)`: `crisis_die` (the player's Simple Die),
`crisis_total` (the CRISIS TOTAL it made), `crisis_row` (the **id** of the row it
landed on) and `crisis_severity`. Together with the pre-existing `crisis` flag they
distinguish three states a year can be in, which one boolean could not: no Crisis
(`crisis: false`), a Crisis the table demanded and nobody has rolled yet
(`crisis: true`, no `crisis_row`), and a resolved one (all four present).

- **The roll is recorded for the same reason the aging roll is.** `die` and `total`
  are the historical record of what was thrown, not a cached derivation — the
  Decrepitude the CRISIS TOTAL was made against goes on climbing afterwards, so
  re-deriving it later would get a different number. Same argument, same fields.
- **The row travels as an id** — "Bedridden for a week" is `rules/i18n/<lang>/aging.json`
  data, never engine prose.
- **`crisis_severity` is recorded beside the id although the row carries it**, which is
  the one deliberate redundancy. `AgingRules::crisis` is **optional**: a ruleset may
  ship no Crisis Table at all, and a save can be loaded under one that does not, at
  which point the id resolves to nothing and the severity is all that is left to say
  what the character went through. A `None` severity beside a **present** `crisis_row`
  is a bedridden row (`:16626`, `:16627`) — time, not an illness. The writer takes both
  from the same resolved row, so they cannot disagree.
  `ease_factor` and `ritual_level` are **not** recorded: they are properties of the row
  the sheet re-reads, and the severity is the rank that outlives the table.
- **No `SCHEMA_VERSION` bump — it stays 15**, pinned by
  `a_resolved_crisis_round_trips_and_needs_no_schema_bump` (`types.rs`). The widening
  is additive in *both* directions: an older save omits the four keys and defaults
  them, and a newer save is still a document an older reader accepts, since
  `AgingLogEntry` declares no `deny_unknown_fields` and the keys are omitted whenever
  they are absent. That is exactly the case `Entity.living_conditions` made. The 14 → 15
  bump was earned by something different in kind — `AgingLogEntry::year` *became*
  `Option`, so a new save may omit a key an old reader **requires**.
- Mirrored by hand in `ui/src/lib/types.ts` (`AgingLogEntry` plus a `CrisisSeverity`
  union), which `every_aging_field_is_mirrored_in_the_frontend_types`
  (`arm-app/tests/commands.rs`) enforces by populating every optional field.

#### Translation-table notes — `alterung-twilight.md`
Two things about `rules/source/de/translation-tables/alterung-twilight.md` that a
future translator must not "fix" the core file from.

1. **Its modifier column carries a pre-subtracted sign.** The table declares it
   itself at `:43` — *"Negativer Modifikator: wird vom Alterungswurf subtrahiert."* —
   and its rows at `:47-56` accordingly print the negation of the book's own column
   (Wohlhabend −2 where `:16583` prints +2; Aussätziger +2 where `:16592` prints -2).
   That is a **convention difference, not a defect**, but it means the glossary must
   **never** source `rules/core/aging.json`'s numbers. The shipped modifiers come
   from the English `:16583-16592`, and the German *names* from the mirrored German
   rulebook body at the same line numbers — not from this table.
2. **Its `:67` narrows "3 or more" to "3–9", which _is_ a divergence** from `:16600`
   ("3 or more") and from the German rulebook's own mirrored line. Under the book's
   reading the apparent-age row overlaps every effect row, which is exactly why the
   engine models it as `apparent_age_increase_min` rather than as a table row; under
   the glossary's "3–9" a total of 15 would age the Characteristic but not the face.
   The core file is right and the glossary line is the outlier.
3. **The Crisis Table's German severities are a false friend — do not "correct"
   them.** German **_Schwere_ Erkrankung = _Major_ illness** (`:16630`) and **_Ernste_
   Erkrankung = _Serious_ illness** (`:16629`), which is the opposite of the English
   cognate pull ("schwer" reads as *severe/serious*). The German rulebook body at
   `Ars Magica Definitive Edition Basisregeln.md:16628-16632` and the glossary at
   `alterung-twilight.md:82-92` agree with each other, so both `rules/i18n/de` names
   are right as shipped. `english_and_german_i18n_cover_all_crisis_rows`
   (`data_integrity.rs`) pins the two of them as literals for exactly this reason.
   The ladder in full: Leichte / Ernste / Schwere / Kritische / Tödliche =
   Minor / Serious / Major / Critical / Terminal.

## Markdown character export (M5.6) — `export.rs`

`export.rs` implements **no new rules mechanic**, so it carries no rulebook
citation of its own. It is a pure formatter — `(&Entity, &LocalizedRuleset,
&labels) → String` — that *selects and lays out* numbers other modules already
compute: creation figures from `effective.rs` (effective Characteristic / Ability /
Art scores, the XP allocation, Confidence, Warping, Decrepitude, effective Might),
the point balance from `validation/balance.rs`, and play stats from `derived.rs`,
whose provenance is the **Derived play-stat totals (M5/5i)** section above. Every
formula it prints is documented there; nothing is recomputed here. A drift test
(`tests/export_golden.rs`) pins the whole document against a checked-in fixture, so
a change in any of those upstream numbers shows up as a fixture diff.

**Which `DerivedTotals` families the document includes** — the ones a character
sheet prints, i.e. a fixed statline the reader needs at the table:

| Included | From |
|---|---|
| Combat lines (weapon, Ability, Init/Atk/Def/Damage, Range) | `derived::combat_totals` |
| Soak, with its labelled addends | `derived::soak` |
| Encumbrance (Load, Burden, penalty) | `derived::encumbrance` |
| Fatigue levels and their penalties | `derived::fatigue_levels` |
| Wound bands, their damage spans and penalties | `derived::wound_ranges` |
| Warping Score + Points, Decrepitude Score | `effective::warping`, `effective::decrepitude_score` |

**Which are deliberately excluded, and why.** Each is a *per-line working figure*
recomputed as play proceeds, or a piece of read-only guidance the panel offers while
the character is being built — not sheet content. Printing them would bury the sheet
in a grid. A named test (`the_document_excludes_the_per_line_derived_readouts`)
holds the line, so this is a contract, not an oversight:

- `lab_totals`, `casting_totals` — one figure per `(Technique, Form)` cell, each with
  base and within-focus variants: a 5 × 10 grid the player derives on demand (the app
  itself surfaces them behind a Technique/Form picker).
- `penetration` — one line per known spell, and already a function of the Casting
  Total the export does not print.
- `magic_resistance` — one figure per Form, likewise a grid.
- `masterpiece` (lesser-item cap), `talisman_capacity` — read-only *guidance* for
  designing an item, explicitly not a stored value (see the 5i rows above).
- `familiar` (the `FamiliarReadout` bonding read-out) — guidance whose applicability
  is a troupe judgment. The familiar's **statblock** (animal, Might, Characteristics,
  Size, Personality Traits, cords, invested powers) *is* included: that is the
  creature's own recorded data, not a derived read-out.
- `longevity` (the `LongevityBonus` read-out and its `LongevityHint`) — the *stored*
  ritual (source, entered bonus, focus) is included; the hint is a suggestion for a
  ritual made today and must never be mistaken for the stored figure.
- `surfaced_modifiers` — by construction the families the app does **not** simulate
  (study, aging rolls, non-standard casting, wound recovery); they belong beside the
  subsystem that would use them.

**The sheet prints the whole taxonomy, not only the stored values.** A score of 0 is
stored as an *absent* key (the frontend deletes it), so "stored" and "entered" are not
the same thing: the Characteristics table emits every `Characteristic::ALL` row and the
Arts tables every Art in the catalogue (id order within each `ArtType`, which is also
the sheet's canonical Cr/In/Mu/Pe/Re + An/Aq/Au/Co/He/Ig/Im/Me/Te/Vi order), the
untouched ones as `0`. Which Arts exist is data, so the Arts section's size follows
`rules/core/arts.json` with no code change. Both sections keep an **emptiness** gate —
no Characteristic score or description at all, no scored Art the catalogue holds — so a
covenant's sheet still shows neither, without any `EntityKind` gate.

**The sheet shows what a reader can use, not only what was bought.** Three columns
read a *derived* value rather than the stored one, each from the function that owns it:

- The Characteristics table's Effective cell is
  `effective::effective_characteristic_after_aging` — the aging drops plus the free
  Virtue delta, with the effective-minimum clamp — so an aged character's sheet cannot
  show a score its own Soak, Combat and wound figures contradict (they all read the same
  function; see the M5/5i section above). Printed only where it differs from the bought
  score.
- The Abilities table appends a row for every `effective::ability_score_floors` entry
  the character has **not** also bought — an Ability held purely because a Virtue seeded
  it (Second Sight 1) is stored nowhere in `ability_scores`, so iterating the bought
  instances alone dropped it from the sheet. Bought column 0, effective column from
  `effective::effective_ability_score` (so a Puissant bonus on a granted Ability shows).
  The floor reaches only the parameter-less instance, so a bought row cancels it on the
  same instance match.
- The Aging block lists the accrued `entity.aging_points` per Characteristic
  (`aging-points-heading`, the non-zero entries). They are the recorded state both the
  Decrepitude Score and the Characteristic drops derive from
  (`effective::decrepitude_score`, `effective::effective_characteristic_after_aging`);
  without them a reader sees a dropped score but not how close the next drop is.

**A spell's Arts and level are one short code.** The spell list prints
`<Technique abbreviation><Form abbreviation><resolved level>` (`CrIg20`), the notation
the rulebook and the app's own spell list use, instead of three spelled-out columns.
The abbreviations are localized rules data (`LocalizedRuleset::abbreviation`, from
`rules/i18n/<lang>/arts.json` — see the Art catalogue rows above); nothing is composed
in code. An unresolved General level keeps the localized marker a space apart
(`MuVi General`), and a spell no catalogue holds leaves the cell empty rather than
printing a half-written code.

**Granted Virtues/Flaws are listed but off-budget.** Each Virtue/Flaw section prints
the point-bought rows from `entity.selections` and then, under a `export-granted`
sub-heading, the free rows `effective::entity_grants` resolves (House grant, mythic
type grant, `grants_selection`, owed Warping fill) — so a magus's free House Virtue
appears on the sheet, while the balance read-out keeps counting `compute_balance`,
which sees the bought selections alone (see *The free House Virtue is budget-free and
uncapped* above: grants are exempt from the budget and the count caps).

**Localization split** (an architecture rule, not a rules mechanic): item names come
from the rules i18n via `LocalizedRuleset::display_name`; document chrome is passed
in as a `key → text` map keyed by Fluent message name, so the engine hardcodes no
user-facing string and never renders a raw slug — including a Virtue/Flaw's **type**,
which is the catalogue item's `category` localized through the `category-<id>` key the
in-app badge also uses. `export::LABEL_KEYS` declares every chrome key except the three
families composed from catalogue *data* (`type-<profile>`, `param-label-<key>`,
`category-<category>`, all assembled by the frontend's `composedExportLabelKeys`), and
two tests keep it honest — one walks each fixed taxonomy
(`Characteristic`, `Magnitude`, `AbilityCategory`, `ArtType`, `ItemKind`, `Realm`,
`ReputationType`, `LongevitySource`, `CrisisSeverity`, `LifeStageBlock`, the Soak addend
labels, Fatigue tiers, wound bands) and asserts the composed key is declared; the other
renders a fully-populated magus with every declared key resolved and asserts no
key-shaped text survives.

**A restricted XP pool is labelled by its origin when it has one** (M6/6b8c).
`Doc::restricted_pool_label` follows the same rule as the in-app XP bar's
`restrictedPoolLabel`, so one budget reads the same on screen and on the sheet: a
`XpPoolOrigin::LifeStage` pool prints `xp-pool-<block>` (the block enum being a fixed
taxonomy, its three keys are declared in `LABEL_KEYS`), and a Virtue's grant keeps its
eligibility list, because the item's own name says nothing about what its points may buy
and that is exactly where Educated and Warrior differ. Eligibility could not name a
life-stage block at all: both childhood blocks list the same childhood Abilities, later
life lists every category, and the native-language block is restricted to one *instance*
— it carries no ability and no category, so before this it printed an **empty** label.

**No name reaches the reader as a raw template.** A localized item name may carry
`{placeholder}`s (`ability.dead_language` is "{language} (Dead Language)"), so *every*
name the document prints for a catalogue kind that can be parameterized — Virtues/Flaws,
Abilities, spells — goes through `Doc::parameterized_name`, which fills a placeholder
from the chosen value or else prints the localized `param-label-<key>` slot label. That
covers the two places the raw name used to leak: a restricted pool's eligibility list
(which names an Ability, not one instance of it, so it shows the slot label), and a
parameter *value* that is itself parameterized — Puissant Ability aimed at Provence Lore
stores `{ability: ability.area_lore, area: "Provence"}`, and `Doc::param_display_values`
renders the value as the whole instance name ("Puissant Provence Lore") instead of
leaking `{area}` and then repeating "(Provence)". The remaining raw-name sites print
kinds no shipped catalogue parameterizes and whose schema has no parameter at all
(Arts, Houses, weapons/shields/armor, spell-mastery abilities). Both locales are held to
naming every slot by `every_shipped_parameter_key_has_a_param_label_in_each_locale`
(arm-app), which recomputes the parameter-key set from `rules/core` — the family stays
catalogue data, never a list in code.

---

## Guided wizard step copy (M6/6b8b) — `locales/<lang>/main.ftl`

Each wizard step opens with two or three sentences saying what is decided there.
This is **UI copy, not a mechanic**: nothing reads it, nothing validates against
it, and no engine value is derived from it. It is filed here anyway, because
every factual claim it makes is a rules claim and therefore owes a source — the
one thing that separates guidance from invention.

It lives in Fluent (`wizard-guidance-<phase>`, one line per `CreationPhase`) and
not in `rules/i18n/`, because it is instructional chrome about this application's
flow, keyed by a phase of *our* wizard rather than by a catalogue item's ID.
`rules/i18n/` holds the text of items the ruleset names; no item is named here.

**No rules number is written into the copy.** Where a sentence needs one it is a
Fluent placeable filled from loaded data (`ui/src/lib/derive.ts` —
`wizardGuidance`), so the numbers below live in `rules/core/*.json` alone and a
translation can never freeze a stale one:

| Placeable | Filled from | Value today |
|---|---|---|
| `{ $points }` (characteristics) | `characteristic_rules.start_points` | 7 (`:2342`) |
| `{ $flaws }` / `{ $virtues }` (virtues_flaws) | the type profile's `budget.flaw_points` / `budget.virtue_points` | 3/3 grog, 10/10 companion and magus, 10/20 Mythic Companion (`:2210-2211`, `:2295`, `:2303`, `:2844`) |

Every other sentence is deliberately worded without a number — including the
spell-level cap, which names its terms (Technique, Form, Intelligence, Magic
Theory) but not the `+3` of `:2465`, since that constant is nowhere in the data.

Source per phase, all in `Ars Magica - Definitive Edition (Core Rules).md`:

| Phase | Claim made | Source |
|---|---|---|
| `concept` | creation starts from a concept; the examples (fire wizard / scholar / warrior or covenant staff) | `:2203` |
| `characteristics` | Characteristics are inborn and normal means never raise them; the point buy; a negative score gives points back | `:1025`, `:1027`, `:2342`, `:2346-2354` |
| `virtues_flaws` | Flaws fund Virtues up to the budget; the maximum need not be taken; every character takes a Social Status | `:2209-2211`, `:2295-2303`, `:2309`, `:2816`, `:2844` |
| `experience` | experience is acquired in blocks — the first five years of childhood, then later life a year at a time, plus apprenticeship and the years after it for a magus; a total may be entered instead, or the life stages earn it from the age | `:2364`, `:2378`, `:2392`, `:2213-2216` |
| `abilities` | Abilities are learned skills, bought with that experience; age caps the creation score | `:2364`, `:2366` |
| `arts` | every spell combines one Technique and one Form; apprenticeship's experience buys Arts and Abilities from the same total | `:8835`, `:2435` |
| `spells` | apprenticeship grants levels of spells; the highest level learnable is set by Technique, Form, Intelligence and Magic Theory | `:2215`, `:2435`, `:2465` |
| `house_specialisation` | a magus belongs to exactly one House, whose benefit at creation is a free Minor Virtue needing no Flaw to fund it | `:2264`, `:2859` |
| `mythic_type` | the type is a Free Virtue fixing the character's status; the types are incompatible with each other and with The Gift; one normally brings a free Minor Virtue | `:2637-2638`, `:2846-2847` |
| `personality_reputations` | a few words scored +3 to -3; Loyal for grogs, Brave for warriors; a Reputation only where a Virtue or Flaw grants one | `:2502`, `:2504`, `:2514` |
| `aging` | over 35 an aging roll per year before play; apparent age and Characteristic points are what it costs; Aging Points drop a Characteristic once they exceed it | `:2232`, `:16565`, `:16577`, `:16579` |
| `review` | **no rules claim** — the review step is this application's own, so its line describes the flow and nothing else | — |

#### A grog's Personality Traits (#33)

`rules/core/character_types.json` gave every profile the `personality_reputations`
phase **except `grog`**, so a grog built in the guided wizard could never record
Personality Traits. The editor always offered the tab, so the gap was in the wizard
alone — and it was exactly backwards, because the rules single out that type as the one
where the traits carry mechanical weight:

> "Personality Traits are a short description of important features of your character's
> personality. For major characters, such as magi and companions, they are normally
> nothing more than an aide memoire … **For grogs, they are more significant.** As grogs
> are often shared between players, or at least played rarely, the numbers attached to
> Personality Traits can be used as a concrete guide to playing the character. …
> 'Loyal' is a particularly important Trait, as it reflects the grog's attachment to the
> covenant, while 'Brave' is just as important for warrior grogs. A third Trait should
> be something distinctive about that grog."
>
> — Ars Magica - Definitive Edition (Core Rules).md:1073-1075

The character-sheet listing at `:1165` names Personality Traits unconditionally as
well. No passage restricts them by character type, so the phase belongs to every
profile. Pinned by
`every_shipped_profile_declares_the_personality_reputations_phase`
(`crates/arm-rules/tests/core_type_conformance.rs`), which asserts the phase is
declared per profile — structurally, never as a count.

#### The phase list after the guided-creation review (Slice 2 — #1, #11)

Two changes to the closed `CreationPhase` enum (`crates/arm-rules/src/types.rs`) and
to every profile's `creation_phases` in `rules/core/character_types.json`. **Neither
is a rules change** — the rules' own creation order is untouched; what changed is
which input surface owns which step:

- **`type` removed.** It asked for nothing: the character type is fixed when the
  character is created (`StartScreen` enters the wizard per type) and is immutable
  afterwards, so the step only read back what the profile already commits the
  character to. Its two facts that live nowhere else — the Virtue/Flaw budget numbers
  (`:2209-2211`, `:2295`, `:2303`, `:2844`) and the Gift policy line (`:2224`) — moved
  to the character banner shown above both the editor and the wizard
  (`ui/src/lib/components/CharacterBanner.svelte`), keyed `character-type-explainer`,
  `character-type-budget` and `character-type-gift-required|forbidden|optional`. Both
  numbers stay Fluent placeables filled from the loaded profile, so no rules value is
  written into a translated string.
- **`experience` added, immediately before `abilities`.** The blocks that fund a
  character — early childhood, later life, and for a magus apprenticeship and the
  years after it (`:2364`, `:2213-2216`) — are chosen and priced on their own step
  now, instead of as a ~620px preamble on the step that spends them. Every
  `life_stage_*` and `childhood_*` code, plus `restricted_xp_unspent`, is filed under
  `experience` for the M6/6b1a reason: a finding belongs to the phase whose *input
  surface* owns the offending value. `magus_minimum_ability`,
  `magus_recommended_ability`, `not_enough_xp`, `xp_solve_bound_exceeded`,
  `duplicate_ability` and the `ability_*` codes stay on `abilities`.

`CreationPhase::ALL` therefore still has twelve members; the magus flow is ten
declared phases plus the wizard's synthetic `review`.

---

## Engine framework (book-agnostic, no rulebook source)

These checks are structural integrity, not Ars Magica rules, and intentionally
carry no source citation:

- Incompatibility symmetry (`ruleset/integrity.rs` — `validate_incompatibility_symmetry`)
- Category permit/forbid (`validation/selections.rs` — `validate_permitted_categories` (:8),
  `validate_forbidden_categories` (:50))
- Required/forbidden traits (`validation/selections.rs` — `validate_required_traits` (:144),
  `validate_forbidden_traits` (:165))
- Entity-kind applicability, parameter validation, duplicate-selection detection
  (`validation/selections.rs` — `validate_entity_kind_applicability` (:83),
  `validate_parameters` (:216), `validate_duplicate_selections` (:107))

### The saga year (Slice 12, #25) — `validation/saga.rs`

**One rules value, wrapped in an editing aid.** The saga year — the calendar year
the troupe's saga stands in — is **not** a rule and **not** character state. It is a
saga fact shared by every character in one saga, so it lives in an app-settings file
owned by `arm-app` (`crates/arm-app/src/settings.rs`, commands `saga_year` /
`set_saga_year`), never on the entity and never in the ruleset. `arm-rules` gains no
filesystem dependency from it: the engine holds only the arithmetic and the one
advisory, and receives the year as a plain argument.

The **default** year, however, IS a rules value, and is the only cited thing here:

> `:597` "That domination persists until the present day, 1220."

Corroborated at `:364` ("much like the Europe of 1220, the middle ages") and `:440`
("Much like medieval Europe in 1220"). Encoded as
`arm_rules::DEFAULT_SAGA_YEAR = 1220` in `validation/saga.rs`, read by
`settings::read_saga_year` whenever there is no stored value — so the number appears
once, in the engine, and neither the app nor the frontend restates it.

**What it derives, and what it deliberately does not.** `age` and `birth_year` are
both already stored on the entity, so the saga year adds no state and forces **no
`SCHEMA_VERSION` bump**. It is the reference the two are linked against while the
user types: `age_in_saga_year(saga_year, birth_year)` and
`birth_year_in_saga_year(saga_year, age)`, exposed as the `derive_age` /
`derive_birth_year` commands so the frontend computes none of it itself. Editing
either half rewrites the other and dirties the document; **editing the saga year
rewrites nothing and does not dirty the document.** That is a deliberate refusal, not
an omission: advancing a character by N years requires an aging roll, Living
Conditions and any Longevity Ritual applied *per year* (see *Aging*, above), so a
silent recompute would fabricate ages that skipped their rolls. Saga progression is a
separate, explicit feature.

The derivation shares the calendar-year approximation the aging engine already makes
(`aging.rs`, calendar year = `birth_year + age`): birthdays within the year are
ignored, identically in both directions.

**The one finding.** `birth_year` is `i32` and `age` is `u32`, so a saga year *before*
the birth year would underflow. It clamps the derived age to 0 and emits
`saga_year_before_birth_year` — a **warning**, phase `concept`, args `saga_year` and
`birth_year`, localized as `issue-saga_year_before_birth_year` in both locales. Like
the three `childhood_slot_*` rows it is listed in the `ValidationIssue` contract table
but is **not** emitted by `validate`: the saga year never reaches the engine as entity
data, so only the derivation can raise it. It carries no rulebook citation — no
passage forbids an impossible date; the clamp exists because the type does.

### Creation-phase completeness (M6/6b8a) — `completeness.rs`

Which creation phases the player has not engaged with yet. **Product behaviour,
not a rule**: no passage says an untouched phase is incomplete, so no criterion
below is cited, and none may be invented later. It is deliberately not a
`ValidationIssue` of any severity — every wizard gate is phrased over issues, so an
issue-shaped verdict would be one severity change away from blocking. It rides on
`ValidationResult.completeness` and `apply_mode` passes it through all three modes
untouched (how hard the rules are enforced says nothing about what has been filled
in).

Each criterion reads a stored choice off the entity, scoped to the phases the
character's own type profile declares, in that declared order:

| Phase | Complete when | Rules source |
|---|---|---|
| `concept` | any identity field is set (name, description, concept, gender, birth year, sigil, covenant, parens) | — |
| `characteristics` | some Characteristic is non-zero (an all-zero spread is an untouched point-buy) | — |
| `virtues_flaws` | a selection the profile did not force (`required_traits`, plus The Gift where `gift_policy` requires it) | — |
| `experience` | a life-stage plan is stored **or** a nonzero `xp_pool` is typed — either funding mode counts. Deliberately reads the two stored *substances*, not schema 16's `ability_funding`, which has a default every untouched character carries and so cannot tell a visited step from a fresh one | — |
| `abilities` | an Ability score is bought | — |
| `arts` | an Art score is bought | — |
| `spells` | a spell is known | — |
| `house_specialisation` | the House is recorded | `:2859` "You receive one free Minor Virtue from your choice of House" |
| `mythic_type` | the Mythic Companion type is recorded | — |
| `personality_reputations` | a Personality Trait or a Reputation is recorded | — |
| `aging` | the age is recorded (every other reading on the step is taken against it). Since Slice 12 (#24) the age is *entered* on the `concept` step, so this phase reads as engaged before it is opened — which is correct: the choice it needs has been made | — |
| `review` | always — the closing look at the whole character holds no choices of its own | — |

The House row is the only one resting on a rule, and it is the same choice the
`house_unset` warning is about (see *`validate_house` — specialisation
resolution*); the report merely observes that the choice has not been made, and
still does not require it.

---

## Other available books

The following English sources are present in `rules/source/en/`. Add a section
above (mirroring the Core Rules layout) when mechanics from a book are implemented.

**Nothing implemented yet:**

- Ars Magica 5e - Houses of Hermes - Mystery Cults.md
- Ars Magica 5e - Houses of Hermes - Societates.md
- Ars Magica 5e - Houses of Hermes - True Lineages.md
- Ars Magica 5e - Magic - Hedge Magic (Revised).md

**Cited already, all four through the Mythic Companion types of M4/4d** — this
list used to name them as untouched, which stopped being true when those types
shipped. Each supplies one type's package (its free grants, its required Virtues
and Flaws, its Might grant and power levels) and, for two of them, the bonus
points that type adds to the Virtue/Flaw budget. See **Mythic Companion types
(M4/4d)** above for the per-item line ranges:

- Ars Magica 5e - Realms of Power - Faerie.md — Faerie Doctor
- Ars Magica 5e - Realms of Power - Magic.md — Spirit Votary (and its +7 Flaw
  points, `:5486`)
- Ars Magica 5e - Realms of Power - The Divine (Revised).md — Nephilim
- Ars Magica 5e - Realms of Power - The Infernal.md — Devil Child
