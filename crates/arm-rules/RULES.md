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

## Ars Magica - Definitive Edition (Core Rules).md — ArMDE

### Hardcoded engine values

#### Magnitude point weights — Free 0, Minor 1, Major 3
> "Virtues and Flaws are either Minor or Major. Virtues cost points, while Flaws
> grant points. Major Virtues cost three points, while Major Flaws grant three
> points. Minor Virtues and Flaws cost and grant (respectively) one point."

- Source: `ArMDE:2774` (also stated at
  `ArMDE:2209`). Free (0-point) virtues: `ArMDE:2886-2896`.
- Implementation: `crates/arm-rules/src/types.rs` — `Magnitude` / `Magnitude::points()`.
  These weights are also surfaced to the UI verbatim as `Ruleset.magnitude_points`
  (derived from `Magnitude::points()` in `crates/arm-rules/src/ruleset.rs`); this
  introduces no new rule value — it is the same number, exposed so the frontend
  reads it from the engine instead of re-hardcoding it.

#### Virtue/Flaw polarity — Virtues cost, Flaws grant
> "Virtues cost points, while Flaws grant points."

- Source: `ArMDE:2774`.
- Implementation: `crates/arm-rules/src/types.rs` — `ItemKind::is_positive()`.

#### Rounding default — round **down** when the rule names no direction
> "The rules for Ars Magica sometimes involve division. In most cases, a rule
> specifies whether you should round up or down, but if it does not, round down."

- Source: `ArMDE:547`.
- Implementation: `crates/arm-rules/src/derived.rs` — `halve`, which is
  `i32::div_euclid(2)` (exactly floor division for a positive divisor), shared by
  every undirected halving in the engine: the Deficient Technique/Form halving of
  Casting and Lab Totals, fatiguing Spontaneous ÷2, Weak Magic's Penetration
  halving, Flawed Parma's halving of the Parma contribution to Magic Resistance,
  the Weak Enchanter and Difficult
  Longevity Ritual lab halvings, and the Masterpiece lesser-item cap. The
  non-fatiguing Spontaneous ÷5 follows the same default in
  `crates/arm-rules/src/derived/casting.rs` — `casting_totals`.
- **Why this is not plain `/`.** Rust's `/` truncates toward zero, which agrees
  with flooring only for non-negative operands. A negative total — a Dominion
  aura, a newly gauntleted magus, a halved Penetration — would be reported one
  point *in the character's favour*. Where a rule *does* name a direction the
  engine says so at that site instead: `suggested_longevity_bonus` rounds **up**
  ("every five points or fraction", `ArMDE:10662`) and the aging age modifier
  rounds the decade up (see **Aging**).

(The aging-roll threshold used to sit here as `AGING_ROLLS_START_AGE`. Slice 6b6
introduced `rules/core/aging.json`, so it is now a data value — see
**Aging (M6/6b6)** below.)

### Enforcement logic

#### Virtue/Flaw balance — Virtues must be funded by Flaws
> "Players start with no points for buying Virtues and Flaws, and thus must take
> Flaws if they want Virtues. A central character may have up to ten points of
> Flaws ... and the same number of points of Virtues."

- Source: `ArMDE:2774`, `ArMDE:2297`
  (companions), `ArMDE:2303` (magi).
- Implementation: `crates/arm-rules/src/validation/balance.rs` — `validate_balance`
  (:23), `compute_balance` (:141). Emits `unbalanced_virtues` (error) when spent virtue points
  exceed flaw points granted, plus the `over_budget_*` totals. (Per-type point
  totals are data; see below.)

#### Caps on Major virtues/flaws count
> "You may not take Major Virtues or Flaws" (grogs).

- Source: `ArMDE:2824-2830` (grogs).
- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_caps` (:22)
  (counts items with `magnitude == Major`; cap value is data via
  `max_major_virtues` / `max_major_flaws`).
- The magus rule "You may not have more than one Major Hermetic Virtue" (`ArMDE:2857`)
  is a *category-restricted* cap (Hermetic Major Virtues only), not a plain
  Major-count cap. Implemented in M4/4b via the data-driven
  `virtue_category_caps` (mirroring `flaw_category_caps`) — see the **Houses**
  section below and the "Resolved: companion `max_major_virtues`" note.

#### Cap on Minor Flaws count (hard)
> "A central character may have up to ten points of Flaws, but no more than five
> Minor Flaws."

- Source: `ArMDE:2774`; companion
  `ArMDE:2835`, magus `ArMDE:2856`; grogs "no more than three Minor Flaws" `ArMDE:1009`.
- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_caps` (:22), error
  `too_many_minor_flaws` (counts `magnitude == Minor` flaws; cap value is data,
  `max_minor_flaws`).

#### Per-category flaw caps (Personality hard + Personality/Story soft)
> "A character may not have more than one Major Personality Flaw." (`ArMDE:2820`)
> "A character should normally not have more than two Personality Flaws in
> total" (`ArMDE:2820`); "A character should not have more than one Story Flaw"
> (`ArMDE:2818`); grogs "should not have Story Flaws" (`ArMDE:1009`).

- Source: `ArMDE:2820` (Major
  Personality, hard; restated per type at companion `ArMDE:2838`, magus `ArMDE:2851`),
  `ArMDE:2976` (Personality total), `ArMDE:2818`/`ArMDE:2982` (Story), grogs `ArMDE:1009`/`ArMDE:2826`.
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
- **The caps are reported, never explained** (manual-testing-findings #21). The
  guided wizard briefly restated each cap as a sentence on the Virtues & Flaws
  step (`wizard-guidance-<category>-flaw-cap`, generated from these very caps);
  that whole guidance family was removed along with the rest of the app's
  explanatory prose — the player has the rulebook open. The caps therefore reach
  the screen through the **validator's own findings** alone:
  `too_many_<category>_flaws` and `too_many_major_<category>_flaws`, which is
  where the book's "may not" (`hard`) vs "should not" distinction is carried, as
  an error and a warning respectively. The per-type figures the data encodes:
  grog `ArMDE:2826-2827`, companion `ArMDE:2837-2838`, Mythic Companion `ArMDE:2850-2851`,
  magus `ArMDE:2861-2862`, over the general statements at `ArMDE:2818` (Story ceiling) and
  `ArMDE:2820` (Personality ceiling + Major cap).
- **There is NO "at least one Story Flaw" rule.** `ArMDE:2818` and `ArMDE:2837` give Story
  Flaws a recommended *ceiling* of one and no minimum whatsoever. The only "at
  least one" the rules state is the magus's **Hermetic** Flaw (`ArMDE:2860`), carried
  by the `missing_hermetic_flaw` warning under the condition that the profile
  names at least one `hermetic_flaw_categories` entry.

#### Tainted Virtues/Flaws — the "Type" tag + half-of-taken cap
> "Tainted Virtues and Flaws are associated with the Infernal realm ... no more
> than half a character's Virtues should be tainted, and similarly for Flaws. ...
> Supernatural abilities granted by Tainted Virtues or Flaws are always Infernal
> powers." (`ArMDE:3000`)

- Source: `ArMDE:2998-3002`.
- Data: the descriptor's optional "Type" token maps to `PointItem.tainted`
  (`bool`, default false) in `rules/core/virtues_flaws.json`. Every core-rules
  entry whose descriptor line lists `Tainted` must carry the flag — the tag is
  the *descriptor's*, never the name's: `flaw.tainted_with_evil` is named
  "Tainted With Evil" but reads `*Minor, General*` (`ArMDE:6844`) and is therefore
  **not** flagged, while `virtue.demonic_blood` (`*Major, Supernatural,
  Tainted*`, `ArMDE:3650`) and `flaw.tragic_life` (`*Major, Story, Tainted*`,
  `ArMDE:6856`) are. Both of the latter shipped without the flag, which made the cap
  below under-count their points; `core_rules_tainted_virtues_carry_the_tainted_flag`
  in `crates/arm-rules/tests/data_integrity.rs` now pins a sample of tagged
  entries plus that control.
- Implementation: `crates/arm-rules/src/validation/caps.rs` — `validate_tainted_cap` (:257).
  The book frames the limit as a "should", so it is a **non-blocking warning**,
  measured against the points **actually taken** (not the type budget): a side
  warns when `2·tainted_points > total_points` for that side (Virtue / Flaw).
  Free items contribute 0 points and never affect the ratio. Codes
  `too_many_tainted_virtues` / `too_many_tainted_flaws` (Fluent
  `issue-too_many_tainted_*`, args `$tainted`/`$total`). The tag itself renders
  via Fluent `vf-tag-tainted` (EN "Tainted" / DE "Befleckt", per the glossary's
  type-label rule). The parenthetical "(five points ... ten for a Mythic
  Companion ...)" is illustrative of a maxed build, not a separate flat cap.

#### Demonic Might / Demonic Powers — the half-of-Virtues ratio
> "You may take this Virtue more than once, though it can account for no more
> than half of the character's total Virtues." (`ArMDE:3665`, Demonic Might)

> "You may also take this Virtue more than once, though it can account for no
> more than half of the character's total Virtues." (`ArMDE:3669`, Demonic Powers)

- Source: `ArMDE:3663-3666` (Demonic
  Might) and `ArMDE:3667-3670` (Demonic Powers).
- **Each sentence says "this Virtue", so the two ceilings are independent**, not
  a shared Demonic pool. A character may hold half his Virtue points in Demonic
  Might *and* half in Demonic Powers as far as these two sentences go.
- Data: `"max_share_of_kind": { "numerator": 1, "denominator": 2 }` on
  `virtue.demonic_might` and `virtue.demonic_powers` in
  `rules/core/virtues_flaws.json`. The ratio is a *value the rulebook states*, so
  it lives in the JSON rather than as two hardcoded ids in engine code — the same
  reason the Tainted cap is driven by the `tainted` flag.
- Implementation: `crates/arm-rules/src/validation/caps.rs` —
  `validate_share_of_kind_cap`, registered in `validate()` right after
  `validate_tainted_cap`. Code `too_large_share` (Fluent `issue-too_large_share`,
  args `$item`/`$points`/`$total`).
- **Measured in points, and that is an interpretation.** The engine reads "total
  Virtues" as Virtue *points*, weighed against the points actually taken and
  split by kind (a Virtue's copies against Virtue points, a Flaw's against Flaw
  points), matching `validate_tainted_cap`. The Tainted rule can lean on the
  book's own gloss — "That is, no more than five points…" (`ArMDE:3000`) — which
  settles points-vs-headcount for it. **These two sentences carry no such
  gloss**, so a headcount reading is at least as natural and this choice is a
  judgement call, not a deduction.
- **Warning, not error**, for two reasons: the ratio flickers while a build is
  mid-edit, and the points reading above is debatable — blocking a build on a
  debatable reading is worse than flagging it.
- The comparison is the integer form `part · denominator > total · numerator`,
  so "no more than half" permits exactly half with no rounding choice; with 1/2
  it is literally `2·part > total`, the form `validate_tainted_cap` uses.
- **Granted copies count here, and that is a deliberate divergence from the
  Tainted precedent.** `validate_share_of_kind_cap` takes the **folded**
  selection list (bought ++ granted), because **Devil Child grants a free
  Demonic Might or Demonic Powers** (`ArMDE:3673`) and a granted copy is still a copy
  of the Virtue. `validate_tainted_cap` (`validation/caps.rs`, :257) reads raw
  `entity.selections` and so counts only bought ones — arguably right for Tainted,
  which the book frames as a character-generation guideline. Two identically
  worded "half" rules therefore disagree about grants **on purpose**; do not
  "fix" one to match the other without deciding the question again.
- One transient state warns by design and is pinned by
  `a_devil_child_without_demonic_blood_yet_warns_on_the_ratio` in
  `crates/arm-app/tests/commands.rs`: a Devil Child whose free-Minor choice is
  Demonic Might but who has not yet bought Demonic Blood holds 1 granted point
  of a 1-point Virtue total (Devil Child itself is Free and lifts no
  denominator), so `2·1 > 1` fires. A finished build is clean — both Demonic
  entries require Demonic Blood (Major, 3 points), so one Minor copy sits at 1
  against a total of at least 4.

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
    (`ArMDE:6162-6185`)
  - `flaw.hunger_for_form_magic` — "1 pawn of vis each season, corresponding to the
    Form that it has been mostly exposed to" (`ArMDE:6276-6279`)
  - `virtue.extractor_of_form_vis` — "only if the features of the aura exemplify the
    Form … (once for each Form)" (`ArMDE:3779-3782`)
  - `virtue.imbued_with_the_spirit_of_form` — "any being with a Magic Might associated
    with the Form of this Virtue" (`ArMDE:4085-4094`)
  - `virtue.master_of_form_creatures` — "beings whose Magic Might is aligned with a
    particular Form … once for each Form" (`ArMDE:4463-4466`)

  Already correct and deliberately untouched: `virtue.deft_form` (`ArMDE:3645-3648`),
  `flaw.deficient_form` (`ArMDE:5909-5912`), `flaw.deficient_technique` (`ArMDE:5913-5915`).
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
- **Enumerated domain** (`ParameterDomain::Enumerated`, serde `"enumerated"`): the
  parenthetical is a **closed list the rulebook prints in full**, declared on the
  parameter itself as `values` — `(Beings)` on the three Inoffensive/Offensive/
  Unbearable items, and Folk Magic's spell `category`. The domain is its list, so
  the picker shows a dropdown and a value outside it raises `unknown_param_value`.
  See *Enumerated parameter domain — a closed list the rulebook prints* below for
  the rule, the four value tables and the save impact.
- **Free-text domain** (`ParameterDomain::Text`, serde `"text"`): the parenthetical
  is a free choice with no registry — `(Realm)`, `(Land)`, `(Subject)`, `(Sin)`,
  `(Terrain)`, `(Commodity)`, `(Faculty)`, `(Role)`, `(Power)`, plus the mixed
  `Necessary (Realm) Aura for (Ability)` (a `text` + an `ability`). `validate_parameters`
  resolves a `text` param against no registry, so any value with non-whitespace
  content is legal; an empty or whitespace-only one is **not** — it reads as a
  choice not yet made and raises `missing_param` (see *Parameter-value identity:
  trimmed, never case-folded*). The UI text input already existed. Param hints come from Fluent `param-label-<key>`. `(Terrain)` is here
  rather than under `enumerated` on purpose — its list ends "…, etc." (`ArMDE:6130`).
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
> Virtues: `## Virtues` detailed entries `ArMDE:3360-5282`; Flaws: `## Flaws`
> `ArMDE:5639-7119`. Each entry is `#### Name` + an italic `*Magnitude, Category[,
> Type]*` descriptor + prose.

- Source: `ArMDE:3360-5282` (Virtues),
  `ArMDE:5639-7119` (Flaws); each item's own `source` field carries its line range.
- Data: the full core catalogue (653 items) lives in
  `rules/core/virtues_flaws.json`, with EN/DE display text in
  `rules/i18n/{en,de}/virtues_flaws.json`. This widens the data only — the engine,
  i18n schema, and UI already support any catalogue size (catalogue size is data).
- Extraction conventions (structural fields; effects/prereqs authored per mechanic
  elsewhere): magnitude/categories parsed from the descriptor; each category slugged
  (`Social Status`→`social_status`, source typo `Subernatural`→`supernatural`).
  The optional `Type` token sets `tainted` (see the Tainted note above).
- **A descriptor may name more than one category, and all of them are kept —
  none of them is "primary".** `PointItem.categories` is a `Vec<String>` holding
  **every** category the descriptor lists, in the descriptor's own order. The
  extraction used to keep only the earliest-listed one and drop the rest, which
  silently made a Flaw invisible to the other category's cap and to the other
  category's permit/forbid list. Four core items carry two categories in the data
  — **not** the complete set of multi-category descriptors, which is nine; see
  *Nine descriptors name two categories, and only four were extracted as such*
  below. These four are the ones whose descriptor separates the categories with a
  **comma**, which is all the extraction recognised (`Tainted` is not a category —
  see the Tainted note):

  | id | descriptor | `categories` | source |
  |---|---|---|---|
  | `virtue.sufi` | *Minor, Social Status, Supernatural* | `["social_status", "supernatural"]` | `ArMDE:5077-5078` |
  | `flaw.raised_from_the_dead` | *Major, Story, Supernatural* | `["story", "supernatural"]` | `ArMDE:6646-6647` |
  | `flaw.suppressed_gift` | *Major, Hermetic, Story* | `["hermetic", "story"]` | `ArMDE:6803-6804` |
  | `flaw.visions` | *Minor, Story, Supernatural* | `["story", "supernatural"]` | `ArMDE:6985-6986` |

  **The book itself treats the two categories as equals.** Under `## List of
  Virtues` `ArMDE:3004` and `## List of Flaws` `ArMDE:5283` the indexes are cut into
  `### <Category>, <Magnitude>` sections, and each of the four items above is
  listed in **both** of its sections — verified line by line:

  | item | listed at | and at |
  |---|---|---|
  | Sufi | `ArMDE:3179` (`### Supernatural, Minor` `ArMDE:3135`) | `ArMDE:3230` (`### Social Status, Minor` `ArMDE:3187`) |
  | Suppressed Gift | `ArMDE:5301` (`### Hermetic, Major` `ArMDE:5285`) | `ArMDE:5369` (`### Story, Major` `ArMDE:5340`) |
  | Raised from the Dead | `ArMDE:5365` (`### Story, Major` `ArMDE:5340`) | `ArMDE:5399` (`### Supernatural, Major` `ArMDE:5387`) |
  | Visions | `ArMDE:5517` (`### Story, Minor` `ArMDE:5501`) | `ArMDE:5561` (`### Supernatural, Minor` `ArMDE:5534`) |

  So `categories[0]` is **not** a rule and must never be documented as one. It is
  only the descriptor's own first-listed slug, used as a deterministic tie-break by
  the three surfaces that have room for exactly one (below).

  No core descriptor names three real categories. `Tainted` is **not** a category
  (it is `PointItem.tainted: bool`), so descriptors such as `*Minor, Story,
  Tainted*` stay single-category. `Mythic Companion` **is** one (see its own note
  below).

- **Nine descriptors name two categories, and only four were extracted as such.**
  The sweep's extraction recognised **comma**-separated category lists only, so
  the five descriptors that join their two categories with *and* or *or* were
  extracted single-category:

  | id | descriptor | source | `categories` | indexed at |
  |---|---|---|---|---|
  | `virtue.inoffensive_to_beings` | *Minor, General and Hermetic* | `ArMDE:4134` | `["general"]` | `ArMDE:3110` (`### Hermetic, Minor` `ArMDE:3087`) and `ArMDE:3276` (`### General, Minor` `ArMDE:3239`) |
  | `flaw.curse_of_slander` | *Minor, General or Supernatural* | `ArMDE:5882` | `["general", "supernatural"]` | `ArMDE:5538` (`### Supernatural, Minor` `ArMDE:5534`) and `ArMDE:5575` (`### General, Minor` `ArMDE:5564`) |
  | `flaw.offensive_to_beings` | *Minor, Hermetic and General* | `ArMDE:6525` | `["general"]` | `ArMDE:5445` (`### Hermetic, Minor` `ArMDE:5417`) and `ArMDE:5608` (`### General, Minor` `ArMDE:5564`) |
  | `flaw.primogeniture_lineage` | *Minor, Story and Hermetic* | `ArMDE:6635` | `["story"]` | `ArMDE:5447` (`### Hermetic, Minor` `ArMDE:5417`) and `ArMDE:5516` (`### Story, Minor` `ArMDE:5501`) |
  | `flaw.unbearable_to_beings` | *Minor, Hermetic or General* | `ArMDE:6892` | `["general"]` | `ArMDE:5455` (`### Hermetic, Minor` `ArMDE:5417`) and `ArMDE:5629` (`### General, Minor` `ArMDE:5564`) |

  All five are dual-indexed in the book's own lists, exactly like the four above,
  so the book treats both of their categories as equals here too.

  **Two of the five were a wrong-output bug, and are fixed** (open-to-dos row 13):
  `flaw.offensive_to_beings` and `flaw.unbearable_to_beings` shipped `["hermetic"]`
  alone, and `hermetic` is forbidden for grog, companion and mythic companion — so
  only a magus could take two Flaws the book also indexes under General. See *Two
  Flaws the book indexes under General were magus-only* below.

  **One of the two *or*-joined descriptors is now modelled as a choice (row 13,
  B4).**
  `flaw.curse_of_slander` — "*Minor, General or Supernatural*" `ArMDE:5882` — ships
  both categories plus a `taken_as` parameter offering exactly those two, and
  `max_total: 1`, the same shape `virtue.sufi` carries (see *Row 19, half (a)*
  below). Its "or" is the same either/or `ArMDE:5083` states in prose, so it gets the
  same mechanism rather than flat dual membership: taken as General it must not
  count as a Supernatural Flaw for any `has_category` rule, which is precisely
  the objection open-to-dos row 13 raised against simply adding the second
  category. Both readings were open to every profile before and still are —
  `general` and `supernatural` are both on every profile's permitted list and
  neither is on any forbidden list — so nothing that was legal became illegal;
  what changed is that the bearer's category is now *recorded* rather than
  guessed.

  **The remaining two keep one MEMBERSHIP category deliberately, and the reason
  differs per item.** `virtue.inoffensive_to_beings` already carries the
  *permissive* category (`general` is on every profile's permitted list and no
  profile's forbidden list), so nothing is blocked and adding the second
  category would change no outcome. `flaw.primogeniture_lineage` keeps `story`
  for the opposite reason: adding `hermetic` would corrupt Gift detection,
  because `gift_categories` is `["hermetic"]` on every shipped profile — see
  the overloading note below. Its magi-only restriction is a House matter, not
  a category one, and is now enforced as a prerequisite — see *Primogeniture
  Lineage is for magi of House Verditius only* below.

  **Both nevertheless record the book's index heading (row 18).** Keeping
  `hermetic` out of `categories` is a membership decision; it must not also
  erase what the book's index does. Both — and both Beings Flaws — carry
  `index_categories: ["hermetic"]`, which exactly one consumer reads (the
  `ArMDE:2860` Hermetic-Flaw guideline). See *Resolved (row 18)* below.

  **Decided 2026-09-13 (Norbert): an *and*-joined descriptor means *either
  route*.** That is the permissive reading, and it is what the engine already
  does — so this decision ratifies the existing behaviour rather than changing
  it, and no code or data moved. The three affected items keep their whole
  `categories` list, every membership site keeps asking *any* rather than *all*,
  and none of them gains a `taken_as`. The reasoning the decision rests on is
  below, kept because the alternative was genuinely arguable and a later reader
  should see why it was not taken. Two supporting facts: the book indexes the
  Beings items under Hermetic without their being Hermetic in play, which is what
  `index_categories` exists to record (`ArMDE:2860`'s guideline reads it, nothing
  else does); and `flaw.primogeniture_lineage`'s real restriction was never a
  category at all — it is `All([OrderMember, House(house.verditius)])`, sourced to
  `ArMDE:6636`. So no restriction is lost under the permissive reading.

  **The reasoning, recorded (was row 19, half b): the *and*-joined
  descriptors.** `taken_as` models ***or***, and only *or*. An *or*
  descriptor states a choice between two readings of one item — the book says so
  outright for Sufi (`ArMDE:5083`, "either as a Minor Social Status Virtue or a Minor
  Supernatural Virtue") — so the parameter records which reading the player
  picked and exactly one category is in force. An *and* descriptor makes no such
  statement, and the book never settles what it means: "either route" (the
  permissive reading, which is what the engine does today by resolving the whole
  `categories` list) or "both, hence both categories' restrictions apply at
  once" (the restrictive reading, which no current mechanism expresses —
  `categories_for` returns a list, but every membership site asks *any*, never
  *all*). Stretching `taken_as` over them would be worse than leaving them open:
  it would hand the player a choice the book does not offer, and under the
  restrictive reading it would silently *drop* half of a restriction meant to
  bind. So the three *and*-joined descriptors stay as they are, now by decision
  rather than by deferral: `virtue.inoffensive_to_beings` (*General and
  Hermetic*, `ArMDE:4134`), `flaw.offensive_to_beings` (*Hermetic and General*,
  `ArMDE:6525`) and `flaw.primogeniture_lineage` (*Story and Hermetic*,
  `ArMDE:6635`).

  **`flaw.unbearable_to_beings` is *or*-joined and still stays out, for a
  separate reason.** Its descriptor is *Minor, Hermetic or General* (`ArMDE:6892`), so
  the *and*/*or* question above does not apply to it — but its second category is
  `hermetic`, and `hermetic` is overloaded: `gift_categories` is `["hermetic"]`
  on every shipped profile, so `effective::has_the_gift` reads any hermetic
  category as "has The Gift" (see the overloading note below). Offering
  `hermetic` as a selectable reading would therefore make the *choice itself*
  decide whether the character is Gifted, which is not what `ArMDE:6892` says. The
  eligibility that category was standing in for is already modelled explicitly
  and correctly, as `prerequisites: Any([Has(virtue.the_gift),
  Has(flaw.magical_air)])` — see *Two Flaws the book indexes under General were
  magus-only* below — so `taken_as` would add nothing here and would re-introduce
  the Gift-detection corruption that fix removed. `virtue.inoffensive_to_beings`
  and `flaw.offensive_to_beings` sit behind the same `hermetic` overload on top
  of the *and* question.

  **Row 18's split does not reopen this, and makes the reason structural.**
  `index_categories` is provenance, not a membership category, and load-time
  integrity requires a `taken_as` parameter's declared values to be a subset of
  the item's own `categories` (`validate_parameter_defs`). So `hermetic` is not
  an expressible `taken_as` value for any of these items, and the answer no
  longer rests on remembering why.

  **Row 19, half (a), resolved: "taken as" is now modelled.** `ArMDE:5083` makes
  Sufi's dual category an explicit player CHOICE ("either as a Minor Social
  Status Virtue or a Minor Supernatural Virtue"), not membership in both at
  once. Storage: a `params` entry under a new [`ParameterDomain::Category`]
  (`taken_as`) — not a new `Selection` field, because `params` already joins
  every byte-identity site (the duplicate key at `validation/selections.rs`'s
  `validate_duplicate_selections`) that a new field would sit outside of, and
  needs no `Ord`/`normalize`/canonical-output change. `virtue.sufi` additionally
  carries `max_total: 1`: `ArMDE:5083` offers a choice between two READINGS of one
  item, not two items, and load-time integrity
  (`ruleset::integrity::validate_taken_as_max_total`) now rejects any
  `taken_as`-declaring item that omits this cap.

  **Shipped `taken_as` items.** Two, both in `rules/core/virtues_flaws.json`,
  both carrying `max_total: 1` and a parameter whose values are exactly the two
  categories the descriptor names. The frozen `TAKEN_AS_ITEMS` table in
  `crates/arm-rules/tests/data_integrity.rs` pins the set:

  | item | descriptor | source | `taken_as` values |
  |---|---|---|---|
  | `flaw.curse_of_slander` | *Minor, General or Supernatural* | `ArMDE:5882` | `["general", "supernatural"]` |
  | `virtue.sufi` | *Minor, Social Status, Supernatural* (choice stated at `ArMDE:5083`) | `ArMDE:5077-5078` | `["social_status", "supernatural"]` |

  A save written before an item gained its parameter reports the standing
  non-blocking `missing_param` until the player states the choice, and is
  deliberately **not** migrated: fabricating a placeholder would invent a
  player's rules choice. `SCHEMA_VERSION` is unaffected — this is catalogue
  data, not a save-shape change.

  The resolution is centralized in one place, `PointItem::categories_for`
  (`crates/arm-rules/src/types.rs`): given a selection's `params`, it returns
  the ONE category the selection recorded (`taken_as`) if present, otherwise
  the whole `categories` list exactly as before this mechanism existed. Every
  membership test that reads "is this item of category X for THIS entity" now
  calls it, so a grog Sufi taken as Social Status no longer counts as holding a
  Supernatural Virtue:

  | site | file:function |
  |---|---|
  | permitted categories | `validation/selections.rs::validate_permitted_categories` |
  | forbidden categories | `validation/selections.rs::validate_forbidden_categories` |
  | category caps | `validation/caps.rs::validate_caps` |
  | Gift detection | `effective/gift_confidence.rs::has_the_gift` |
  | grant-constraint filtering | `grant.rs::open_pick_satisfies` (reads the *pick's own* `params` — set by the player exactly like any other parameter on that pick, whether the pick came from a `Fixed`/`Choice` grant's own data or a player's `Open` choice) |

  Deliberately whole-list still, and commented as such at each site: a
  catalogue query with no entity (`Ruleset::items_by_category`) and every UI
  *browsing* surface (`derive.ts`'s `groupByCategory`/`filterItems`,
  `VirtueFlawTab.categoriesFor`) and the Markdown export's Type cell
  (`export/sections.rs`) — there is no selection to narrow against for the
  first, and the book itself indexes a dual-category item under both headings
  for the rest, so hiding it from one heading would hide it from the very
  category that may be the one making it legal.

  **Row 19, half (a), presentation (B3): every user-facing surface now SAYS the
  chosen category.** B2 made the engine decide by it while every surface still
  reported the `first_listed_category` tie-break, which meant the app could
  reject a Virtue and then name a category the player had not chosen. Four
  surfaces changed, each reading the same single resolution:

  | surface | file:function | now |
  |---|---|---|
  | `category_not_permitted` / `forbidden_category` argument | `validation/selections.rs::first_in_force` | names the first category *in force*, i.e. the chosen reading where one was recorded |
  | Selected V/F grouping | `derive.ts::groupSelectionsByCategory` | files the row under the chosen reading, not `categories[0]` |
  | Selected V/F category badge | `VirtueFlawTab.svelte` `nameWrap` snippet | badges the chosen reading alone, so badge and heading always agree |
  | exported Virtue/Flaw **name** | `export/resolve.rs::taxonomy_label` | "Sufi (Social Status)" — the value is labelled through the `category-<id>` Fluent family instead of falling through to the raw slug |

  `derive.ts`'s `selectionCategories` is the frontend's single mirror of
  `PointItem::categories_for`, exported so the grouping and the badge cannot
  re-derive the answer differently; it reads the parameter's **domain** off the
  ruleset item rather than matching the key name `taken_as`. The export fix is
  the sharp edge of the four: a `Category` value is an id like `social_status`
  with no rules-i18n entry of its own, so the tolerant parameter-value path
  printed the bare slug — a direct breach of the "never render a raw ID as a
  user-facing label" rule. `taxonomy_label`'s `match` on `ParameterDomain` is
  exhaustive so a future taxonomy domain (a `Realm`, labelled through the
  existing `realm-<id>` family) must be answered explicitly. No new Fluent key
  was needed: `category-*` already covers every shipped category in both
  `locales/en/main.ftl` and `locales/de/main.ftl`, and the export label map is
  composed from the catalogue's own categories
  (`file-operations.svelte.ts::composedExportLabelKeys`), which load-time
  integrity guarantees is a superset of any `Category` parameter's values.

  The exported **Type** cell is deliberately unchanged and still lists both
  categories: it is the descriptor, i.e. provenance, not the player's choice.

  No shipped House or mythic-companion-type `Grant::Choice` offers
  `virtue.sufi` (verified by grep across `rules/core/`): a `taken_as` item in a
  `Choice` options list would need one option per value, since `Grant::Choice`
  matches the player's pick by full `Selection` equality (`grant.rs`) — this is
  a real hazard for a *future* `taken_as` item that also wants a `Choice` menu,
  not one this catalogue hits today.

- **Primogeniture Lineage is for magi of House Verditius only, and that is a
  prerequisite, not a category.**

  > "This Flaw can only be taken by magi of House Verditius, as a maga who has
  > left the House is no longer a candidate for Primus. In her case, it would be
  > no more than an interesting feature of her background."
  > — ArMDE:6636 (entry `ArMDE:6634-6637`)

  Data: `rules/core/virtues_flaws.json` — `flaw.primogeniture_lineage` carries
  `prerequisites: All([OrderMember, House(house.verditius)])`. Evaluated by
  `validation/prereq.rs::evaluate_prereq`. Migrated from `Prereq::IsMagus` in
  D56/A0 sub-slice 4 (the enum split): `OrderMember`, not
  `HermeticallyTrained`, because `ArMDE:6636` itself draws the disqualifying
  line at the House — "a maga **who has left the House** is no longer a
  candidate for Primus" — not at training, and House membership is
  structurally an Order concern (`validate_house` gates on `order_member`).

  Before this the restriction was enforced by **nothing**. The Flaw ships
  `categories: ["story"]`, which the companion, mythic-companion and magus
  profiles all permit, so a companion, a mythic companion and a magus of any
  House could take it; only the grog was refused, and merely because `story` is
  not on its permitted list — the wrong reason for the right outcome.

  **`Prereq::House` alone would not have done it, and the `OrderMember` conjunct
  is not redundant.** The House leaf is tri-state (`types.rs`, `Prereq::House`): a
  matching house is True, a differing one False, and **no house at all is
  `Unknown`** — which surfaces as the non-blocking `prereq_unevaluated` warning,
  not an error. A companion has no house, so a bare House leaf would merely have
  warned him. Nor does the engine forbid the value: `validate_house` returns
  early for a profile whose `order_member` is false, so a hand-edited save can put
  `house: house.verditius` on a companion, and a bare House leaf would then have
  evaluated True and waved it through. `All([OrderMember, House(...)])`
  short-circuits to False on any non-Order-member while still leaving a magus who
  has not reached the House step on the warning — the engine's error-that-resolves
  model, and exactly how the four Outer-Mystery Virtues behave. This is also why
  the four existing House-prereq items (`virtue.faerie_magic`,
  `virtue.heartbeast`, `virtue.the_enigma`, `virtue.verditius_magic`) can use the
  bare leaf and this one cannot: all four are `categories: ["hermetic"]`, which
  every non-magus profile forbids, so Order membership was already enforced
  beside them.

  `Prereq::conflicts_with_house` handles the conjunction correctly for open grant
  menus (the owed-Warping Minor-Flaw menu can offer this Flaw): `OrderMember` is
  undecided there by design, and a House leaf naming a different House sinks the
  `All`, so the item is excluded from a non-Verditius magus's menu. The UI mirror
  `houseOnlyValue` in `ui/src/lib/derive.ts` folds `all` the same way.

  **What the prerequisite cannot express**, and is not claimed to: the passage's
  own reasoning about a maga who has *left* the House (the engine models only
  current membership and has no notion of a former House), and the entry's "She
  is at least three places removed from the Primus", which is narrative
  positioning with no mechanical hook. Locked by
  `primogeniture_lineage_requires_an_order_member_of_house_verditius` and the
  five behavioural tests beside it in `tests/data_integrity.rs`.
- **Vendetta's House restriction HEDGES, unlike Primogeniture Lineage's, and a
  hedge is a warning, never an error (F-533/F-550, D16, Q-115, Q-139).**

  > "This Flaw is generally restricted to magi of House Verditius, as the
  > custom of vendetta is limited to that House."
  > — ArMDE:6957 (entry ArMDE:6955-6958)

  Where Primogeniture Lineage's "can only be taken by" is absolute, this
  passage says "**generally** restricted" — D16's rule is that the engine's
  push must match the book's, so a hedge gets a warning, not a hard block.
  `Prereq` had no warning severity to carry that (F-550): `prereq_not_met` is
  unconditionally an error. The fix is a sibling field, not a wrapper `Prereq`
  variant — `PointItem::advisory_prerequisites` (`types.rs`), a second,
  independent tree evaluated by the same tri-state
  `validation/prereq.rs::evaluate_prereq`, whose `Tri::False` reports the new
  `advisory_prereq_not_met` (warning) code instead of `prereq_not_met` (error).
  A sibling field rather than a `Prereq::Advisory(Box<Prereq>)` wrapper because
  the hard and advisory trees never need to compose under `All`/`Any`/`Nor` —
  what a "soft" child inside a hard boolean expression would even mean is
  exactly the ambiguity a second, wholly independent tree avoids — and because
  it needs no UI parity guard the way a new `Prereq` variant would
  (`ui/src/lib/prereq-parity.test.ts` diffs the `Prereq` union itself, which is
  unchanged). `Tri::Unknown` stays silent on the advisory tree (no
  `..._unevaluated` twin): D16 hedges a STATED violation into a warning, it
  does not ask the engine to nag about data it cannot yet see.

  Data: `rules/core/virtues_flaws.json` — `flaw.vendetta` carries
  `advisory_prerequisites: House(house.verditius)`. The Flaw's own magus half
  (Q-139: unhedged, "The magus is engaged in…") is deliberately NOT encoded
  here — a hard `prerequisites: OrderMember` (D56/A0's split; House custom is
  the same Order concern as Primogeniture Lineage's, pending X5's own
  confirmation) is slice X5's job, which this
  machinery unblocks. Tests: `vendetta_ships_an_advisory_house_verditius_restriction`
  and the three behavioural tests beside it in `tests/data_integrity.rs`.

  **Other hedges this same machinery is now owed to, not yet paid**: Q-115
  names `flaw.inscribed_shadow` (ArMDE:6320, "generally restricted" to House
  Criamon) as the population's other member — same shape, unencoded, no slice
  assigned yet. D44 (Q-19) considered a warning for `virtue.gentle_gift` /
  `flaw.blatant_gift`'s unstated `incompatible_with` but ruled the OPPOSITE way
  — a hard block stays, because the pair is *entailed* contradiction, not a
  hedge — so it is not a live carrier. D16's own Q-05 (20 sex-restricted
  Virtues) and Q-123 (`flaw.night_terrors`, `flaw.oath_of_fealty`) are hedges
  too, but of a different shape — text-only-by-design, and a profile-level
  `forbidden_traits` softening, respectively — neither is a `Prereq`, so
  neither is a carrier for this mechanism either.
- **Membership uses the whole list, unless a `taken_as` selection narrows it —
  and browsing always uses the whole list.** Every rule that asks "is this item
  of category X" is a membership test over `PointItem::categories_for` (row 19,
  above), which returns all of `categories` (`PointItem::has_category` /
  `any_category_in`) UNLESS the selection recorded which one it was taken as:
  permitted and forbidden categories (`validation/selections.rs`), the
  data-driven per-category caps (`validation/caps.rs`), Open-grant
  `require_categories`/`forbid_categories` (`grant.rs`), and the Gift categories
  (`effective/gift_confidence.rs`). `validation/magus.rs`'s Hermetic-Flaw-guideline
  read of `gift_categories`, the engine-required `personality` category
  (`validation/scores.rs`, `ruleset/integrity.rs`), and `Ruleset::items_by_category`
  stay whole-list unconditionally — none of the shipped catalogue's `taken_as`
  items carries `hermetic`/`personality`, and `items_by_category` is a catalogue
  query with no entity to narrow against in the first place. The UI's
  **Available (source) V/F picker** mirrors the book's indexes and ALSO stays
  whole-list, deliberately: `derive.ts` `groupByCategory` emits the item once
  per category it carries, so Sufi is offered under *both* Social Status and
  Supernatural regardless of any `taken_as` a later selection might record —
  there is no selection yet to narrow against while browsing. That grouping is
  also the sole source of the category-filter dropdown's options
  (`VirtueFlawTab.svelte` `categoriesFor`), so a category carried only in second
  position is still filterable. The Markdown export's "Type" cell renders them
  all too, joined with the shared localized list separator, each through its
  own `category-<id>` Fluent key (`export/sections.rs`) — the exported
  descriptor is provenance, not the player's choice; the chosen reading appears
  in the exported *name* instead ("Sufi (Social Status)", via
  `export/resolve.rs::taxonomy_label`). Consequences worth naming: Suppressed Gift
  counts against the **Story** Flaw cap as well as being Hermetic, and Sufi
  (absent a `taken_as`, or browsed rather than selected) is returned by
  `items_by_category("supernatural")` as well as by `"social_status"`.
- **Three surfaces have room for exactly one category, and all take the FIRST
  category in force** — the chosen reading where a selection recorded one
  (row 19, above), otherwise the descriptor's own first, which is a tie-break
  and not a rule (hence `PointItem::first_listed_category`, renamed from
  `primary_category`, which asserted a rule the book does not have). They are
  the `category_not_permitted` issue's `category` argument
  (`validation/selections.rs::first_in_force` — only reached when *every*
  category in force failed, so any of them would do), the `forbidden_category`
  issue's argument (same helper: it used to name "the category that actually
  offended, whichever position it holds", but under the conjunction below there
  is no distinguished offender — either every category in force is forbidden or
  the issue is not raised at all), and the UI's **Selected** V/F list
  (`derive.ts` `groupSelectionsByCategory`, via `selectionCategories`). The
  Selected list must not duplicate a row: bought rows are removed by their
  `entity.selections` index, so the same row under two headings would delete each
  other and make one selection read as two against the point budget. That
  heading always agrees with the row's first category badge, which renders the
  same resolution and which `ui/e2e/specs/magus-editor.e2e.js`'s "lists a
  granted Virtue under its own category heading, inline with the bought rows"
  asserts.
- **Permitting is ANY, and forbidding is EVERY — the two are mirrors**
  (`validation/selections.rs`: `validate_permitted_categories`,
  `validate_forbidden_categories`). An item is permitted when **any** of its
  categories is on the profile's permitted list, and ruled out only when **every**
  one of them is on its forbidden list. Both readings follow from the same fact:
  a descriptor naming two categories offers two routes to the same Virtue/Flaw, and
  one blocked route leaves the other open. Forbidding used to fire on *any*
  forbidden category, which contradicted the permitted side — it granted a route
  and then took it straight back. For a single-category item "every" is identical
  to "any", so the change loosened these cells and nothing else:

  | item | type | before | after | source |
  |---|---|---|---|---|
  | `virtue.sufi` (*Minor, Social Status, Supernatural*) | grog | blocked | **allowed** | `ArMDE:5079` "It is also possible to be an entirely mundane Sufi, in which case you should take this Virtue as a Social Status Virtue"; `ArMDE:5083` "either as a Minor Social Status Virtue **or** a Minor Supernatural Virtue" |
  | `flaw.suppressed_gift` (*Major, Hermetic, Story*) | companion | blocked | **allowed** | `ArMDE:2840` bars a companion from Hermetic V/F "unless you have The Gift", and a Suppressed-Gift character *does* have it (`ArMDE:6805` — it does not function, but the social penalties remain); `ArMDE:6809` "If he replaces a companion, he will become much more powerful when the Story Flaw is resolved". Since row 20 the "does have it" half is *modelled*: the Flaw carries `Has(virtue.the_gift)`, so the companion has to hold The Gift rather than merely be assumed to |
  | `flaw.suppressed_gift` | mythic companion | blocked by category | **blocked by `gift_forbidden`** | `ArMDE:2637` — the status Virtues "are incompatible … with The Gift". Same outcome, honest issue code |

  Three further cells (a grog taking Raised from the Dead, Visions, or Suppressed
  Gift) stop emitting a redundant *second* issue while staying blocked by the
  permitted check — the honest reason, since neither of their categories is on a
  grog's permitted list.

  **Superseded in part by the grog profile's Supernatural removal** (see the
  profile table's grog rows): a grog no longer forbids `supernatural` and now
  permits it, so the first table row's *mechanism* is obsolete even though its
  outcome stands — Sufi is open to a grog through **either** of its categories
  now, not rescued by `social_status` against a forbidden `supernatural`. Of the
  three further cells, Raised from the Dead and Visions (both *Story,
  Supernatural*) are no longer blocked by the category checks at all; they are
  refused by the sourced Story-Flaw cap (`ArMDE:2826`, `flaw_category_caps` story
  `max: 0`) instead. Suppressed Gift (*Hermetic, Story*) is unaffected and is now
  the only shipped pairing that still exercises the conjunction for a grog, which
  is why `forbidding_fires_only_when_every_category_is_forbidden` uses it as its
  fixture rather than Visions. **Qualified again by row 20** (below): Suppressed
  Gift now requires The Gift (`ArMDE:6805`) and a grog may never hold one (`ArMDE:2830`),
  so the pairing carries a `prereq_not_met` on top of everything else. The
  conjunction it demonstrates is untouched — the two category validators run
  independently of prerequisite evaluation — but the pairing is no longer a build
  anyone could complete, and the test says so and asserts the extra issue.
  **No replacement exists and the test proves it**: a grog forbids only
  `hermetic`, and `flaw.suppressed_gift` is the sole multi-category item in the
  catalogue carrying that category.
- **`categories` is order-significant and therefore exempt from canonical sorting** —
  the same deliberate exception the crisis table's `crisis.rows` takes (see the
  aging section's "Three things a later sweep must not undo"), and the reason it is
  a `Vec` rather than a `BTreeSet`. `PointItem::normalize` sorts `parameters` and
  leaves `categories` alone. Because a `Vec` can express what a set cannot,
  load-time integrity (`ruleset/integrity.rs::validate_item_categories`) rejects an
  empty list and a repeated slug, naming the offending item.
- **The removed singular `category` key fails the load loudly.** `PointItem` is
  `#[serde(try_from = "PointItemRepr")]` purely so that an old `rules/` directory
  beside a newer binary — the portable layout, where the rules live next to the exe
  — stops with `point item '<id>' uses the removed singular 'category' key …`
  instead of being silently coerced into a one-element list. There is deliberately
  **no serde alias**. No save migration and no `SCHEMA_VERSION` bump came with this:
  saves store `Selection { ref, params }` (`types.rs`) and never an item's category,
  so no save file mentions one.
- **`Mythic Companion` is a category.** Four Free Virtues carry it — Devil Child
  (`ArMDE:3671-3672`), Faerie Doctor (`ArMDE:3821-3822`), Nephilim (`ArMDE:4594-4595`), Spirit
  Votary (`ArMDE:5006-5007`), stored as `categories: ["mythic_companion"]` in
  `rules/core/virtues_flaws.json`. The `## List of Virtues` index (`ArMDE:3004`) groups
  its entries under `### <Category>, <Magnitude>` headings, and
  `### Mythic Companion, Free` (`ArMDE:3329`) is one of them, listing exactly these four
  (`ArMDE:3331-3334`). None of them appears under `### Social Status, Free`
  (`ArMDE:3336-3354`). Their descriptors read `*Free, Mythic Companion*` — magnitude then
  category, the same shape as `*Minor, Social Status*` — whereas `Tainted` never
  occupies that slot, only ever appearing as a **third** token after a real
  category (`*Major, Supernatural, Tainted*`, `ArMDE:3650`). That asymmetry is what
  separates the two: `Tainted` is a cross-cutting flag, `Mythic Companion` is a
  heading of the index itself. The chapter's prose sections (`ArMDE:2878` Hermetic,
  `ArMDE:2884` Social Status, `ArMDE:2958` Supernatural, `ArMDE:2964` Personality, `ArMDE:2980` Story,
  `ArMDE:2994` General) explain six categories; they are not an exhaustive list of the
  index's headings, and reading them as one is what produced the earlier, wrong
  `social_status` mapping (manual-testing-findings-2026-09-03 #10).

  Only the `mythic_companion` profile lists the category in `permitted_categories`
  (`rules/core/character_types.json`), so the other three types get
  `category_not_permitted`. That is the rules' own restriction, not a convenience:
  `ArMDE:2637` — "All Mythic Companions take a Free Virtue which specifies their status.
  These Virtues are incompatible with each other, and with The Gift, and are **not
  available to grogs**" — and each descriptor says the Virtue *makes* its bearer a
  Mythic Companion (`ArMDE:3673` "can only be taken for a Mythic Companion", `ArMDE:3823`,
  `ArMDE:4596`, `ArMDE:5008`). A magus is doubly excluded, its profile requiring The Gift the
  four are `incompatible_with`. Locked by
  `the_mythic_companion_virtues_carry_the_mythic_companion_category`,
  `only_the_mythic_companion_profile_permits_the_mythic_companion_category` and
  `a_grog_may_not_take_a_mythic_companion_virtue` in `tests/data_integrity.rs`;
  labelled by `category-mythic_companion` in both locales (DE *Mythischer
  Gefährte*, the German index heading at `Basisregeln.md:3329` and
  `translation-tables/grundbegriffe.md:83`).
- **Dual-magnitude split.** A `*Major or Minor*` item becomes two entries,
  `<id>_minor` and `<id>_major`, marked mutually `incompatible_with` (so exactly
  one magnitude is chosen), disambiguated in i18n as "Name (Minor/Major)" /
  "Name (Klein/Groß)". This mutual exclusion is enforced at load by
  `ruleset/integrity.rs` `validate_magnitude_variant_exclusivity` (see the Magical Focus
  section), which also covers the sole prefix-form pair
  `virtue.major_magical_focus` / `virtue.minor_magical_focus` (ArMDE:4405).
  **`flaw.false_power` / `flaw.false_power_minor` is deliberately NOT such a
  pair** — its two entries must be held together, and its asymmetric ids are what
  keep it out of that check. Do not rename them; see *Selection multiplicity —
  False Power's subsequent copies are Minor*.
- **German provenance.** DE names/summaries come from the line-mirrored German
  source (`Ars Magica Definitive Edition Basisregeln.md`, same line positions),
  cross-checked against `rules/source/de/translation-tables/tugenden-fehler.md`
  (glossary wins on any term mismatch). The DE descriptor uses `Kostenlos` as well
  as `Frei` for Free.
- Referential integrity + EN/DE i18n coverage are asserted structurally by
  `tests/data_integrity.rs` (`shipped_data_passes_integrity_check`,
  `english/german_i18n_covers_all_items`) — never an exact catalogue total.

#### Known source errata (D25, D78)

The book's own catalogue index (the `### <Category>, <Magnitude>` link lists,
Virtues `ArMDE:3006-3358`, Flaws `ArMDE:5285-5637`) is provenance, never a data
source and never a guard (D25) — where it disagrees with an entry's own
descriptor, **the descriptor always wins**, and the disagreement is recorded
here rather than left implicit. D25 found the first row (`flaw.weak_personality`)
in a 25-entry span; the X9c descriptor sweep (`tmp/x9c-plan.md`,
`tmp/x9c-verdicts*.md`) then compared descriptor, data, and index for the
**entire** catalogue — all 662 entries — and found **zero data mismatches**
(every shipped `kind`/`magnitude`/`categories` already agrees with its own
descriptor) and exactly these further 7 index errata, for 8 rows total:

| Entry | Descriptor cites | Index cites | Kind | Side taken |
|---|---|---|---|---|
| `virtue.lupus_the_wolf` (Lupus (the Wolf)) | `ArMDE:4336` *Minor, Social Status* | absent from Social Status, Minor (`ArMDE:3187-3237`) | omission | descriptor stands — the entry is real, just unindexed |
| `virtue.minor_enchantments` (Minor Enchantments) | `ArMDE:4533` *Minor, Supernatural* | absent from Supernatural, Minor (`ArMDE:3135-3185`) | omission | descriptor stands |
| `virtue.turb_trained` (Turb Trained) | `ArMDE:5180` *Minor, Social Status* | absent from Social Status, Minor (`ArMDE:3187-3237`) | omission | descriptor stands |
| `flaw.true_love_minor` (True Love, Minor) | `ArMDE:6872` and `ArMDE:6877` *Major or Minor, Story* | absent from Story, Minor (`ArMDE:5501-5518`); only "True Love (NPC)" appears under Story, Major (`ArMDE:5373`) | omission | descriptor stands — both magnitudes are real, per the Dual-magnitude split convention above |
| `flaw.bound_to_role_role` (Bound to (Role) Role) | `ArMDE:5736` *Minor*, Supernatural | `ArMDE:5392`, listed under Supernatural, **Major** | disagreement (magnitude) | descriptor (Minor) stands — shipped data already agrees |
| `flaw.broken_vessel` (Broken Vessel) | `ArMDE:5754` *Minor*, Supernatural | `ArMDE:5393`, listed under Supernatural, **Major** | disagreement (magnitude) | descriptor (Minor) stands — shipped data already agrees |
| `flaw.weak_personality` (Weak Personality) | `ArMDE:7077` *Minor*, **Personality** | `ArMDE:5518`, listed under Story, Minor | disagreement (category) | descriptor (Personality) stands — shipped data already agrees (D25's original finding) |
| Folk Magic | Virtue heading `ArMDE:3907`, correctly listed under Supernatural, Minor in the Virtue index (`ArMDE:3150`) | ALSO stray-listed under Supernatural, Minor in the **Flaw** index (`ArMDE:5545`) | stray index entry | Folk Magic is a Virtue only (`virtue.folk_magic`); the Flaw-index row is spurious and ignored |

**Sweep result, stated explicitly per D25 obligation 2: all 662 catalogue
entries checked, 0 data mismatches, 8 index errata (the above), none still
open.**

#### A category rule may carry a condition (open-to-dos row 20)
> "You may not take Hermetic Virtues and Flaws, unless you have The Gift (this
> would be highly unusual)" — `ArMDE:2840`

> "Only characters with The Gift can take these Virtues and Flaws, and some are
> only applicable to Hermetic magi who have already completed their training."
> — `ArMDE:2880`

The companion profile encoded only the unconditional half of `ArMDE:2840`, so the
exception the book grants a Gifted companion was unreachable. A profile's
`permitted_categories` / `forbidden_categories` entry is now a
`CategoryRule` (`types.rs`) — `#[serde(untagged)]` over either a bare slug or
`{ category, when: <Prereq> }`, so every existing rules file loads and re-emits
byte-identically.

**Semantics, stated and implemented once.** An entry is in force **iff** `when`
is absent or evaluates to `Tri::True`. `Tri::False` and `Tri::Unknown` both
leave it out of force; Unknown resolving that way matches the engine's existing
non-blocking `prereq_unevaluated` model. The single resolution point is
`categories_in_force` (`validation/selections.rs`), which both gates call, so
they cannot disagree — the same discipline `PointItem::categories_for` applies
to an item's own categories. The `PrereqCtx` it evaluates against is the one
`validation::validate` already builds for `validate_prerequisites`
(hoisted for this purpose in `e2e35f3`), so there is no second evaluation path.

**The leaf is `Has(virtue.the_gift)`, and it has to be.** A category-based Gift
test would be circular: `effective::has_the_gift` reads
`gift_categories: ["hermetic"]`, so "permit `hermetic` when Gifted" plus "Gifted
because you hold a `hermetic` item" is a rule that licenses itself. The id leaf
is non-circular because `virtue.the_gift` is `categories: ["special"]` and Free —
it is in no Gift category and always permitted, so satisfying the condition can
never require the category it licenses. Pinned by
`a_companion_does_not_gift_himself_with_a_hermetic_virtue`
(`tests/data_integrity.rs`), which asserts both the structural fact and that a
companion holding *only* `flaw.blatant_gift` — exactly the character
`has_the_gift` calls Gifted — is still refused.

**Both halves of the profile carry the condition** (`rules/core/character_types.json`,
the companion profile): `hermetic` is permitted when `Has(virtue.the_gift)` and
forbidden when `Nor([Has(virtue.the_gift)])`. Permitting is ANY and forbidding
is EVERY, so relaxing only the forbid leaves every single-category Hermetic item
refused with `category_not_permitted` — the lesson the grog `supernatural` row
records in the profile table above. The two conditions are complements, so an
unGifted companion sees exactly the behaviour he saw before (both issues), and
only the Gifted case moves.

**Load gate.** A profile-borne `when` is authored in the `rules/` directory
beside the binary — the declared hostile-input surface — so
`ruleset/integrity.rs::validate_category_rule_conditions` walks each one through
the same `validate_prereq_refs` an item's prerequisite passes: a dangling ref
fails the load, and `PREREQ_MAX_DEPTH` bounds the recursion. `validate_prereq_refs`
takes a free-text `context` rather than an `Id` for this reason. And because the
two fields are `Vec`s now rather than self-canonicalising `BTreeSet`s,
`EntityTypeProfile::normalize` sorts them explicitly and
`validate_category_rules` rejects a category named twice — a case the set made
unrepresentable. Three tests in `ruleset.rs`:
`a_conditional_category_referencing_an_unknown_item_fails_the_load`,
`a_pathologically_deep_conditional_category_is_rejected_cleanly`,
`a_profile_naming_one_category_twice_fails_the_load`.

**Not done, deliberately: a Gifted companion is not a magus.** The magus profile
mandates Hermetic Magus status, a House and the Order's minimum Abilities *as
errors*. The unschooled Gifted person `ArMDE:2840` contemplates is none of those, and
the point of the rule is that a **companion** may be Gifted.

**Four prerequisite corrections shipped in the same change**, all in
`rules/core/virtues_flaws.json`, because the conditional category is only half a
fix if the Gift-bearing items around it state the wrong requirement:

| item | before | after | source |
|---|---|---|---|
| `virtue.gentle_gift` | `Has(virtue.hermetic_magus)` | `Has(virtue.the_gift)` | `ArMDE:3956` "*Major, Hermetic*" with no prerequisite line, matching its twin `flaw.blatant_gift` at `ArMDE:5712`; `ArMDE:2880` gates the whole category on The Gift, not on the Order. The magus requirement was inferred from the comparative at `ArMDE:3957` ("Unlike other magi, whose Magical nature disturbs normal people and animals"), which compares rather than restricts, and the penalty it cancels attaches to The Gift — `ArMDE:6805` has a possible non-member who "continues to suffer the negative social penalties of The Gift". Without this, B5 half-works: a Gifted companion could take Blatant Gift but not its twin |
| `virtue.failed_apprentice` | — | `incompatible_with: [virtue.the_gift]` (symmetric) | `ArMDE:3845` "You may not have The Gift, but if your Gift was not completely destroyed, you may have some Supernatural Abilities" |
| `virtue.apprentice` | — | `Has(virtue.the_gift)` | `ArMDE:3420` "This Virtue may be taken by a child character who has the Gift…" (the descriptor line `ArMDE:3419` reads "*Free. Social Status*"). Acceptance by a magus and troupe approval are table decisions with nothing on the sheet to check |
| `flaw.suppressed_gift` | — | `Has(virtue.the_gift)` | `ArMDE:6805` "The character has The Gift but cannot access its power" |

`incompatible_with` rather than a `Nor` prerequisite for Failed Apprentice
because that is how this catalogue already states a flat "may not have The
Gift" — `virtue.devil_child`, `virtue.faerie_doctor`, `virtue.nephilim` and
`virtue.spirit_votary` all do, with `virtue.the_gift` listing each back. The one
`Nor` in the data (`flaw.offensive_to_beings`, `ArMDE:6530`) is there because that
rule is *conditional* ("unless you have the Gentle Gift"), which an
incompatibility cannot express; `ArMDE:3845`'s is not.

**Save impact: none, and none owed.** This is ruleset shape, not save shape, so
`SCHEMA_VERSION` stays 16 and there is no migration. An existing save holding
`flaw.suppressed_gift` (or Apprentice, or Gentle Gift) without
`virtue.the_gift` now reports `prereq_not_met`. That is the accepted precedent
of open-to-dos row 17(c): saves store choices, the engine only reports, and one
selection clears it.

#### A creation phase may carry a condition (A2/D56)
> "He knows Hermetic magic and can cast spells and enchant items like other
> magi. He is not a member of the Order of Hermes, however."
> — `ArMDE:5641-5650` (Abandoned Apprentice)

D56 splits the old single `is_magus` flag into *Hermetically trained*
(`hermetically_trained`) and *member of the Order* (`order_member`) —
`docs/vf-audit/design-a0-is-magus-split.md` has the full design. A1 landed the
two profile booleans, the `Prereq::HermeticallyTrained`/`Prereq::OrderMember`
variants, and the entity-level union `is_hermetically_trained` (profile flag
OR a selection carrying `Effect::ConfersHermeticTraining`,
`effective/hermetic_training.rs`). A2 makes the wizard's/tab list's own phase
list follow the same union rather than the bare profile flag, mirroring
*A category rule may carry a condition* exactly: `EntityTypeProfile::creation_phases`
is now `Vec<PhaseRule>` (`types.rs`) — `#[serde(untagged)]` over either a bare
phase slug or `{ phase, when: <Prereq> }` — so every existing rules file loads
and re-emits unchanged.

**Semantics, resolved once.** A phase is in force **iff** `when` is absent or
evaluates to `Tri::True`; `Tri::False`/`Tri::Unknown` both leave it out of
force, the same convention `categories_in_force` established. The single
resolution point is `phases_in_force` (`validation/selections.rs`), exposed to
`arm-app` and, through `effective_dto.rs::EffectiveScores.phases_in_force`, to
the frontend — `ui/src/App.svelte`'s tab list intersects its static tab
metadata against this resolved list instead of re-deriving `isMagus`-shaped
booleans client-side. `completeness.rs::completeness` reads the same resolved
list, so a phase not yet in force is never reported "incomplete" either.

**Shipped data.** The magus profile's phases are unchanged (every entry stays
the unconditional `Always` form — all of them are true for a magus regardless).
The companion profile (`rules/core/character_types.json`) gains two new
conditional entries, `{"phase": "arts", "when": {"kind": "hermetically_trained"}}`
and the same for `spells`, inserted after `abilities` (matching the magus's own
order). **No shipped item carries `Effect::ConfersHermeticTraining` yet** —
`flaw.abandoned_apprentice` gains it only in slice D3, together with the
truncated-apprenticeship XP shape it funds — so today's plain companion sees
neither phase, exactly as before; the conditional path is proved with
test-only ruleset fixtures (`effective/hermetic_training.rs`,
`effective_dto.rs`, `crates/arm-app/tests/commands.rs`).

**Load gate.** A `PhaseRule::when` sits at the identical trust boundary as a
`CategoryRule::when`, so `ruleset/integrity.rs::validate_phase_rule_conditions`
walks it through the same `validate_prereq_refs` a category rule's condition
passes — a dangling ref fails the load rather than silently resolving to
`Tri::Unknown` (not-in-force) with nothing naming the typo. Pinned by
`a_conditional_creation_phase_referencing_an_unknown_item_fails_the_load`
(`ruleset.rs`).

**Save impact: none.** `Entity` carries no `creation_phases`-shaped field —
this is ruleset shape (rules data, keyed by `ruleset.version`), not a saved
`Selection` — so no `SCHEMA_VERSION` bump.

#### Two Flaws the book indexes under General were magus-only (open-to-dos row 13)
> **Hermetic** — "Only characters with The Gift can take these Virtues and Flaws,
> and some are only applicable to Hermetic magi who have already completed their
> training." — `ArMDE:2880`

The `hermetic` category gates on **The Gift, not magus-hood**. `ArMDE:2882` goes
further and lets a troupe read Hermetic Flaws as Supernatural Flaws for a
sufficiently supernatural character. `ArMDE:2840`: a companion "may not take Hermetic
Virtues and Flaws, **unless you have The Gift**". `ArMDE:2829`: grogs may not, and
`ArMDE:2830` bars them from The Gift entirely.

Two Flaws the book also indexes under General shipped `["hermetic"]` alone, and
`hermetic` is on the forbidden list of the grog, companion and mythic-companion
profiles — so only a magus could take them. That was **wrong output**:

> **Offensive to (Beings)** *Minor, Hermetic and General* (`ArMDE:6525`) — "You may not
> take this Flaw more than once, characters who are Offensive to more than one
> kind of being should take Magical Air instead. **Characters with The Gift may
> take this Flaw only if they have the Gentle Gift**, which makes this type of
> being react to them negatively while others are unaffected. **Characters with
> Magical Air may not take it at all.**" — `ArMDE:6530`

> **Unbearable to (Beings)** *Minor, Hermetic or General* (`ArMDE:6892`) — "**Only
> characters with The Gift or Magical Air may take this Flaw, and it cannot be
> combined with the Blatant Gift.**" — `ArMDE:6895`

`ArMDE:6530` restricting *the Gifted case* is the proof: the unGifted case is the
default, so the Flaw cannot be magi-only.

**Fixed as data, not as a second category.** Both are `categories: ["general"]`
in `rules/core/virtues_flaws.json`, and the eligibility the `hermetic` category
was enforcing **by accident** is now stated explicitly. A third item from the
same block of the same page had the eligibility gate missing outright:

| item | `prerequisites` | `incompatible_with` | source |
|---|---|---|---|
| `flaw.unbearable_to_beings` | `Any([Has(virtue.the_gift), Has(flaw.magical_air)])` | `[flaw.blatant_gift]` | `ArMDE:6895` |
| `flaw.offensive_to_beings` | `Any([Nor([Has(virtue.the_gift)]), Has(virtue.gentle_gift)])` — "unGifted, or Gifted with the Gentle Gift" | `[flaw.magical_air]` | `ArMDE:6530` |
| `virtue.inoffensive_to_beings` | `Any([Has(virtue.the_gift), Has(flaw.magical_air)])` | — | `ArMDE:4139` "UnGifted characters may take this Virtue only if they have the Flaw Magical Air." |

`Nor`'s wire tag is `"none"` (see the `Prereq` doc comment in `types.rs`) — it is
the boolean NOR, not "no prerequisite". `validate_incompatibility_symmetry`
requires the reverse edges, so `flaw.blatant_gift` and `flaw.magical_air` carry
theirs; they are behaviourally inert but mandatory at load.

**`hermetic` must NOT be added back as a second category**, and this is the
reason the obvious "completion" of the dual-category data is a trap:
`effective::has_the_gift` (`effective/gift_confidence.rs`) defines "has The Gift"
as the profile's `gift_id` **or any selection carrying any category in
`gift_categories`** — and every shipped profile sets
`gift_categories: ["hermetic"]`. That is right for Suppressed Gift, which *is* a
Gift Flaw (`ArMDE:6805`). It would be flatly wrong here: an unGifted companion taking
Offensive to (Beings) would count as Gifted, changing `validate_gift_policy` and
silently handing him the Gift's free Supernatural-Ability slot via
`supernatural_free_slots` (`ArMDE:2874`). Pinned by
`a_companion_holding_offensive_to_beings_is_not_gifted` in
`tests/data_integrity.rs`, so a later "completion" fails loudly.

**Resolved (row 18): the regression above is repaired by splitting the field in
two.** `gift_categories` used to serve **three** masters — Gift detection, the
free-slot grant, and `validate_house`'s "a magus should take at least one
Hermetic Flaw" guideline (`ArMDE:2860`). Only the first two forced dropping
`hermetic`, so the third became collateral: a magus whose only Hermetic Flaw was
one of these two — a legal build; a magus without the Blatant Gift may take
Unbearable (`ArMDE:6895` — "Only characters with The Gift or Magical Air may take this
Flaw, and it cannot be combined with the Blatant Gift"), and a Gentle-Gift magus
may take Offensive (`ArMDE:6530`) — was told he had none, though the
book lists both in its Hermetic Flaws index (`ArMDE:5445`, `ArMDE:5455`).

Two fields now carry the two meanings, and `hermetic` is still **not** a
membership category on either Flaw:

| field | question | reader |
|---|---|---|
| `PointItem::index_categories` | *Under which headings does the book's own index file this entry, beyond its descriptor's categories?* — provenance | `validate_house` **only** |
| `EntityTypeProfile::hermetic_flaw_categories` | *Which categories does this type's `ArMDE:2860` guideline count?* — `["hermetic"]` on the magus profile, empty elsewhere | `validate_house` **only** |

> You should take at least one Hermetic Flaw
> — `ArMDE:2860` (a bullet under
> `#### Magi`, `ArMDE:2853`)

`validate_house` counts a selected Flaw when the profile's
`hermetic_flaw_categories` meets the item's `categories` **or** its
`index_categories`. Everything else stays blind to `index_categories`: Gift
detection, the free-slot grant, permitted/forbidden categories, the category
caps, grant constraints, `PointItem::categories_for`, `items_by_category`, the UI
pickers and the Markdown export's Type cell. That is a leak guard, not a
convention — `gift_detection_ignores_index_categories` and
`index_categories_are_invisible_to_every_membership_surface`
(`tests/data_integrity.rs`) fail loudly if any of them starts reading it.

The guideline reads the item's whole `categories` list, **not** the taken-as
narrowed `categories_for`. Both lists are provenance for this question — "does
the book list this Flaw as Hermetic?" — so a player's presentation choice must
not strike an entry off the book's own Hermetic index.

Four shipped items carry `index_categories`, all naming `hermetic`, all
hand-verified against the `### Hermetic, Minor` block at `ArMDE:5417` (Flaws) and
`ArMDE:3087` (Virtues). The frozen `INDEX_CATEGORY_ITEMS` table in
`tests/data_integrity.rs` pins the set with the index line each was verified at:

| item | descriptor | membership `categories` | indexed under Hermetic at |
|---|---|---|---|
| `flaw.offensive_to_beings` | *Minor, Hermetic and General* `ArMDE:6525` | `["general"]` | `ArMDE:5445` |
| `flaw.primogeniture_lineage` | *Minor, Story and Hermetic* `ArMDE:6635` | `["story"]` | `ArMDE:5447` |
| `flaw.unbearable_to_beings` | *Minor, Hermetic or General* `ArMDE:6892` | `["general"]` | `ArMDE:5455` |
| `virtue.inoffensive_to_beings` | *Minor, General and Hermetic* `ArMDE:4134` | `["general"]` | `ArMDE:3110` |

`virtue.inoffensive_to_beings` is a **Virtue**, so it can never satisfy a
Hermetic-*Flaw* guideline; it carries the heading because the field records what
the book's index does, and a provenance record that is populated only where some
consumer happens to need it is not provenance.

Load-time integrity (`ruleset::integrity::validate_index_categories`) rejects a
repeated heading and rejects a heading the item already carries in `categories`
— the field records only the divergence, and a slug in both lists would let a
reader satisfy itself from either and blur the line. Unlike `categories`, whose
order is the descriptor's own emphasis, `index_categories` is canonically sorted
by `PointItem::normalize`: an index has no authored order.

`SCHEMA_VERSION` is untouched (16). Both fields are **ruleset** shape, not save
shape, so there is no migration.

**Known divergence, recorded rather than resolved: id-based vs category-based
Giftedness.** `Prereq::Has(virtue.the_gift)` is **id**-based; the engine's own
`has_the_gift` is `gift_id` **or** any selection in a `gift_categories`
category. The two can disagree in both directions:

- A Suppressed-Gift companion is Gifted by the engine's reckoning (and by
  `ArMDE:6805`, still suffering the Gift's social penalties) yet satisfies
  Offensive's `Nor([Has(the_gift)])` arm without the Gentle Gift, and fails
  Unbearable's Gift arm. Both readings diverge from a literal `ArMDE:6530` / `ArMDE:6895`.
- A magus can satisfy `gift_policy: required` through the **category** arm alone
  — any `hermetic` selection with no `virtue.the_gift` row — and then sees
  `prereq_not_met` on `virtue.gentle_gift` and `flaw.blatant_gift`, which gate
  on the **id** (`Has(virtue.the_gift)`). Told he is Gifted by one validator and
  not Gifted by the next.

**They ought to agree, and the source says which way.** `ArMDE:2858` ("You must take
The Gift *and* the Hermetic Magus Social Status Virtue") and `ArMDE:2870` ("all magi
must have this Virtue") make The Gift a specific **row**, not a category. And
`ArMDE:2880` — "Only characters with The Gift can take these Virtues and Flaws" —
states a *requirement on* Hermetic items, not an implication *from* holding one;
the category arm reads it backwards. So the correct shape is: Giftedness is
`gift_id` alone, and `ArMDE:2880` is expressed as `Has(virtue.the_gift)`
prerequisites on the Hermetic items themselves (B5 already did exactly that for
`flaw.suppressed_gift`, `virtue.gentle_gift`, `flaw.blatant_gift` and
`virtue.apprentice`, which is what makes the category arm redundant for them).

Not done here: dropping the category arm changes Gift detection, the
`supernatural_free_slots` grant and `validate_gift_policy` for every profile,
and needs a `Has(virtue.the_gift)` audit across every `hermetic` item — a rules
sweep, not a consequence of this field split. Recorded as owed.

Relatedly, `ArMDE:2840`'s "unless you have The Gift" conditional is now modelled for
the companion profile (see *A category rule may carry a condition*), and its
`when` leaf is deliberately the **id** `Has(virtue.the_gift)` for exactly this
reason: a category condition would license itself.

#### The Gift policy — required / forbidden by type
> "all magi must have this Virtue" ... "Grogs can never have The Gift".

- Source: `ArMDE:2868-2877` (The Gift),
  `ArMDE:2858` (magi must take The Gift + Hermetic Magus status), `ArMDE:2293` and
  `ArMDE:4067-4069` (only magi may take the Hermetic Magus Social Status).
- Implementation: `crates/arm-rules/src/validation/selections.rs` — `validate_gift_policy` (:2012).
  The Gift policy is independent of the `hermetically_trained`/`order_member` flags
  (an unGifted Redcap is a companion; a Gifted hedge wizard is not Hermetically trained).

#### Prerequisite evaluation (meta-mechanic)
- The tri-state `Prereq` evaluator (`crates/arm-rules/src/validation/prereq.rs` —
  `evaluate_prereq`, :421) is engine infrastructure, not a single rulebook passage. It
  enforces book requirements expressed as data, e.g. "all magi must take the
  Hermetic Magus Social Status" (`ArMDE:2293`), encoded as a `Prereq` on the relevant
  items.

#### Category-ranging Prereq/Effect + ability prohibitions (B1/D21/D41) — `Prereq::HasCategory`, `Effect::ForbidsAbilityCategory`/`ForbidsItemCategory`/`ForbidsAbilities`, `CategoryCap.min`/`min_hard`
> "The character must have a Social Status Virtue dictating his place within
> the university." (Rector/Proctor)

> "You are completely unable to learn a certain class of Abilities... This may
> be Martial Abilities, or a more limited set of the others." (Ability Block)

> "...all Personality Traits must be between +1 and -1... The character may
> have no other Personality Flaws or Virtues or Flaws that grant Personality
> Traits." (Weak Personality)

> "You may not take Bargain, Charm, Etiquette, Folk Ken, Guile, Intrigue, or
> Leadership as beginning Abilities, but you may learn them in play."
> (Sheltered Upbringing)

- Source: `ArMDE:6671-6674` (Rector/Proctor),
  `ArMDE:5651-5654` (Ability Block), `ArMDE:7076-7079` (Weak Personality),
  `ArMDE:6721-6724` (Sheltered Upbringing), `ArMDE:2816` (Social Status floor —
  "must take one").
- **One design slice, five findings.** F-502/F-427/F-355/F-542/F-511 all
  range over a category or a fixed id list rather than a single named item,
  which `Prereq`/`Effect` could not express before B1. Designed together in
  `docs/vf-audit/design-b0-ranging-and-predicates.md` (D21, D41) rather than
  landed as five incompatible spellings. (D40's own authorization residual —
  Feral Upbringing's `RestrictsAbilityCategoryToAbilities` whitelist, and
  D60.2's stacking rule for it — was withdrawn by D63/B1c: no shipped data
  ever used the variant, so it was removed as YAGNI. Feral Upbringing's
  wilderness list is D2's, `docs/vf-audit/design-d0-xp-modes.md`'s
  replacement-pool mechanism, not this slice's.)
- **`Prereq::HasCategory(String)`** (`types.rs`) — the category-ranging twin of
  `Has`: satisfied when the entity holds (bought or granted) at least one item
  whose in-force category matches. Evaluated via `PrereqCtx.held_categories`
  (`validation/prereq.rs::PrereqCtx::build`, folded from bought ++ granted
  selections' `categories_for`), a definite True/False, never Unknown — an
  item's own category is static, unlike a fact the entity has not yet
  supplied. `flaw.rector`'s prerequisite is this slice's worked example (data
  is X5/B2's, not B1's).
- **`Effect::ForbidsAbilityCategory { category: AbilityCategory }`** (Ability
  Block, F-355) and **`Effect::ForbidsItemCategory { category: String }`**
  (Weak Personality clause 1, F-542) are the twin *forbid* half D21 asks for —
  one concept (a category-scoped prohibition), realized as an Ability-axis/
  item-axis sibling pair on the same precedent as
  `AbilityScoreGrant`/`CharacteristicScoreDelta`, since `AbilityCategory` is a
  closed 5-value enum while `PointItem::categories` is free-form (one untagged
  type across both would be ambiguous at the wire — both legally contain the
  bare string `"general"`).
- **`Effect::ForbidsAbilities { abilities: BTreeSet<Id> }`** (Sheltered
  Upbringing, F-511) is a third, id-list sub-shape — the passage names seven
  specific Abilities, not a whole category. **This engine draws no
  in-play/beginning distinction at all** — `Entity::ability_scores` carries no
  life-stage-block or timing field, and no restricted-XP pool (including the
  LaterLife block, `effective/xp.rs::magus_later_life_pool`) is tracked per
  bought score either, so "beginning Abilities" and "the whole of
  `entity.ability_scores`" are the same set by construction — not a
  simplification, the honest shape of what this app (which builds the
  character as of saga-start only) can express.
- **All three forbid effects are consumed by one grant-aware validator**:
  `validation/selections.rs::validate_category_effect_prohibitions`
  (:274). Grant-aware on both sides (D2): the forbidding item's effect applies
  bought or granted, and the forbidden target (an Ability held via
  `entity.ability_scores` or an `AbilityScoreGrant`/`AbilityScoreGrantParam`
  floor, or another V/F selection) is read the same way — closing the F-466
  reachability trap `validate_incompatibilities`/`validate_forbidden_categories`
  are deliberately bought-only about (B15). **No soft/advisory forbid exists**
  — every violation is a hard error (`ability_forbidden_by_effect` /
  `category_forbidden_by_effect`), matching `Effect::ForbidsAbilitySpecialties`'s
  precedent. Distinct from `validate_forbidden_categories` (a PROFILE's own
  category list, not an ITEM's authored prohibition against another) — see
  both functions' doc comments for the cross-reference.
- **`CategoryCap.min`/`min_hard`** (`types.rs`, additive on the existing
  ceiling-only struct) give ArMDE:2816's "must take one Social Status" a
  floor mechanism — `max`/`hard` alone can only bound from above. Enforced in
  `validation/caps.rs::validate_caps`, alongside the existing ceiling
  (`too_many_<category>_<flaws|virtues>`), as the mirror-shaped
  `too_few_<category>_<flaws|virtues>` (or the `major_only` form), severity
  following `min_hard` independently of the ceiling's own `hard` — D41 needs
  floor=hard, ceiling=soft on the SAME `social_status` row. The `social_status`
  data row itself is B2/D41's, not B1's; no shipped cap sets `min` yet.
- **Load-time integrity**: `Prereq::HasCategory`/`Effect::ForbidsItemCategory`'s
  category must be declared by at least one point item
  (`ruleset/integrity.rs::category_declared_by_some_item`, :2359, shared by
  `validate_prereq_refs` :2236 and `validate_effect_refs` :2774) —
  deliberately NOT the same as `validate_type_profile_refs`'s documented
  non-check of a type profile's category fields (those name a legitimately
  forward-declared category with no item yet; these sit on an item's own
  prerequisites/effects and claim the catalogue as it stands). Every
  `ForbidsAbilities` ability id must resolve. `CategoryCap.min > max` is
  rejected as unsatisfiable, and `min_hard` with `min` absent is rejected as
  meaningless (`ruleset/integrity.rs::validate_category_cap_floors`, :836).
- Fluent: `issue-category_forbidden_by_effect` (args `$item`/`$category`/`$other`),
  `issue-ability_forbidden_by_effect` (args `$ability`/`$other`) — both locales.
- Tests: `crates/arm-rules/tests/b1_category_and_ability_prohibitions.rs`
  (hand-authored fixtures; the real catalogue entries this note names are
  Phase 3 data work, not B1's).

### Data-driven rule values — `rules/core/character_types.json`

The point budgets and caps per character type are data, not Rust. Each value's
source:

| Type | Value | Source |
|------|-------|--------|
| grog | `virtue_points: 3`, `flaw_points: 3` | `ArMDE:2295`, `ArMDE:2824-2830`, `ArMDE:1009` |
| grog | `max_major_virtues: 0`, `max_major_flaws: 0` | `ArMDE:2824-2830` ("may not take Major Virtues or Flaws"), `ArMDE:1009` |
| grog | `max_minor_flaws: 3` | `ArMDE:1009` ("no more than three Minor Flaws") |
| grog | `flaw_category_caps`: personality major_only/hard `max: 0`; personality `max: 1`; story `max: 0` (soft); `permitted_categories` includes `story` | grogs take one Minor Personality Flaw, no Major Flaws; Story Flaws "should not" (`ArMDE:1009`, `ArMDE:2826`), so since Norbert's 2026-10-03 ruling (F10) a grog's Story Flaw draws only the `too_many_story_flaws` warning, never `category_not_permitted` `ArMDE:2824-2830` |
| grog | `forbidden_categories` includes `hermetic`; `gift_policy: forbidden` | `ArMDE:2829` ("You may not take Hermetic Virtues and Flaws"), `ArMDE:2830` ("You may not take The Gift"), `ArMDE:2876` ("Grogs can never have The Gift"), `ArMDE:1009` ("grogs can never have The Gift") |
| grog | `permitted_categories` includes `supernatural`, and `forbidden_categories` does **not** | **Removed as unsourced** — the entry it replaces forbade `supernatural`, and no passage supports that. `ArMDE:2822-2830` is the grog guidelines in full (up to 3 points of Flaws and an equal number of Virtues; must take one Social Status; should not take Story Flaws; not more than one Personality Flaw; may not take Major Virtues or Flaws; may not take Hermetic Virtues and Flaws; may not take The Gift) and Supernatural appears nowhere in it; `ArMDE:1009` likewise; the `### Supernatural` prose (`ArMDE:2958-2962`) explains realm association and Warping immunity and sets no character-type restriction. **Both halves had to go**: permitting is ANY, so removing only the forbid would have left every single-category Supernatural item refused with `category_not_permitted` — a change that looks like a fix and does nothing. `hermetic` stays forbidden (`ArMDE:2829`, row above). What still bounds a grog here is sourced: `ArMDE:2828`'s Major cap (`max_major_virtues`/`max_major_flaws: 0`), which catches every Major Supernatural item and so does most of the real work; `ArMDE:2830`'s Gift policy, untouched because Gift detection reads `gift_categories: ["hermetic"]`, so a Supernatural Virtue never confers The Gift; `ArMDE:2824`'s 3-point budget; and `ArMDE:2826`'s Story cap, which warns on the two *Story, Supernatural* Flaws (a soft cap since F10, 2026-10-03) |
| companion | `virtue_points: 10`, `flaw_points: 10` | `ArMDE:2297`, `ArMDE:2834-2840` |
| companion | `hermetic` is **permitted when** `Has(virtue.the_gift)` and **forbidden when** `Nor([Has(virtue.the_gift)])` | `ArMDE:2840` ("You may not take Hermetic Virtues and Flaws, unless you have The Gift (this would be highly unusual)"). The conditional is now modelled — see *A category rule may carry a condition* below. **Both halves carry the condition**, because permitting is ANY and forbidding is EVERY: relaxing only the forbid would have left every single-category Hermetic item refused with `category_not_permitted`, the same trap the grog `supernatural` row records. The two conditions are exact complements, so behaviour for an unGifted companion is unchanged (both issues still fire) and only the Gifted case moves |
| companion | `max_major_virtues: null`, `max_major_flaws: null` (no count cap) | no Major-count cap for companions in the book |
| companion | `max_minor_flaws: 5` | `ArMDE:2774`, `ArMDE:2835` |
| companion | `flaw_category_caps`: personality major_only/hard `max: 1`; personality `max: 2`; story `max: 1` | `ArMDE:2820`, `ArMDE:2838` (Major Personality hard); `ArMDE:2820`/`ArMDE:2976` (Personality total); `ArMDE:2818`/`ArMDE:2837` (Story) |
| companion | `creation_phases` includes `arts`/`spells` **when** `hermetically_trained` | `ArMDE:5641-5650` (Abandoned Apprentice: "he knows Hermetic magic and can cast spells" though not the profile's magus). The conditional is now modelled — see *A creation phase may carry a condition* below. No shipped item carries the conferring effect yet (D3), so a plain companion sees neither phase today |
| magus | `virtue_points: 10`, `flaw_points: 10` | `ArMDE:2303` ("Like companions, magi may take up to ten points of Flaws, and the same number of points of Virtues"), `ArMDE:2855` ("up to 10 points of Flaws, and an equal number of points of Virtues") |
| magus | `max_minor_flaws: 5` | `ArMDE:2856` ("may not have more than 5 Minor Flaws") |
| magus | `hermetically_trained: true`, `order_member: true`, `gift_policy: required`, `required_traits: [virtue.hermetic_magus]` | `ArMDE:2858` ("must take The Gift and the Hermetic Magus Social Status Virtue"), `ArMDE:2293` (only magi may take the Hermetic Magus Status) |
| magus | `flaw_category_caps`: personality major_only/hard `max: 1`; personality `max: 2`; story `max: 1` | `ArMDE:2862` ("should not take more than two Personality Flaws, and may not take more than one Major Personality Flaw"); `ArMDE:2861` ("should not take more than one Story Flaw") |
| magus | `virtue_category_caps`: hermetic major_only/hard `max: 1` | `ArMDE:2857` ("may not have more than one Major Hermetic Virtue") — see the Houses section |
| magus | `hermetic_flaw_categories: [hermetic]`, and **no other profile carries the key** | `ArMDE:2860` ("You should take at least one Hermetic Flaw"), a bullet under `#### Magi` (`ArMDE:2853`). Its own field rather than a second read of `gift_categories`: the guideline asks what the BOOK lists as Hermetic, Gift detection asks what the CHARACTER is, and the two Beings Flaws answer those differently — see *Resolved (row 18)* |
| mythic_companion | `virtue_points: 20`, `flaw_points: 10`, `virtue_points_per_flaw_point: 2` | `ArMDE:2638` ("up to ten points of Flaws, and each point of Flaws is worth two points of Virtues. This produces a maximum of 21 points of Virtues and 10 points of Flaws") |
| mythic_companion | `forbidden_categories: [hermetic]`, `gift_policy: forbidden` | `ArMDE:2637` (Mythic Companion status Virtues "are incompatible … with The Gift"); generated as Companions `ArMDE:2635` |
| mythic_companion | `permitted_categories` includes `mythic_companion`, and no other profile's does | `ArMDE:2637` ("All Mythic Companions take a Free Virtue which specifies their status … are not available to grogs"); each status Virtue makes its bearer a Mythic Companion (`ArMDE:3673`, `ArMDE:3823`, `ArMDE:4596`, `ArMDE:5008`) — see the V/F category note |

Landed for `magus` in M4/4b (Houses; see the Houses section): the `≤1 Major
Hermetic Virtue` cap (`ArMDE:2857`) via `virtue_category_caps`, the free Minor House
Virtue (`ArMDE:2859`) via the derived-grant model, and the "≥1 Hermetic Flaw"
guideline (`ArMDE:2860`) via the `missing_hermetic_flaw` warning. The Mythic Companion's free Minor status
Virtue (`ArMDE:2638`, raising the balanced max from 20 to 21) is M5 catalogue data;
the `virtue_points: 20` ceiling here is the balanced maximum without it.

#### Virtue/Flaw funding rate — `virtue_points_per_flaw_point`
Each Flaw point funds one Virtue point by default; Mythic Companions fund two.
Modelled as the data-driven `PointBudget.virtue_points_per_flaw_point` (default
1), applied in `validation/balance.rs::validate_balance` (the `unbalanced_virtues` check
compares virtue points against `flaw_points * virtue_points_per_flaw_point`).
Source: ArMDE:2638.

#### Resolved: companion `max_major_virtues`
Earlier data set companion `max_major_virtues: 1`. The book's "no more than one
Major Hermetic Virtue" (`ArMDE:2857`) is a **magus** rule, and companions may not
take Hermetic Virtues at all (`ArMDE:2834-2840`); the book defines **no** cap on a
companion's count of Major Virtues. The value was therefore corrected to `null`
(no cap).

### Characteristics

#### Eight Characteristics
> "There are eight Characteristics in Ars Magica, each representing one of a
> given character's inborn attributes."

- Source: `ArMDE:1023-1025`.
- Implementation: `crates/arm-rules/src/characteristics.rs` — `Characteristic`
  enum (Int, Per, Str, Sta, Pre, Com, Dex, Qik).

#### Point-buy cost table + seven starting points — `rules/core/characteristics.json`
> "Characteristics are bought on the following table. You start with seven points
> to spend." Printed table: +3→6, +2→3, +1→1, 0→0, −1→Gain 1, −2→Gain 3, −3→Gain 6.

- Source: `ArMDE:2340-2354` (printed
  table), `ArMDE:4105` (the +3 base cap "unless you take … Great Characteristic").
- Data: `rules/core/characteristics.json` (`start_points: 7`, `costs`,
  `base_max: 3`, `base_min: -3`). The rulebook's "Gain N" rows
  are encoded as **negative** cost (`Gain 1` → `-1`, etc.) — an extraction sign
  convention. The table is **exactly the seven printed rows**, ±3, and that is
  also the buy range: `base_max`/`base_min` equal the table bounds.

  *Corrected 2026-09-15 (open-todos row 32 / GitHub issue #4).* The file
  previously carried four **invented** rows (+4→10, +5→15, −4→Gain 10, −5→Gain
  15) justified as "continuing the table's own triangular progression". The
  rulebook prints no cost for ±4 or ±5, and that absence is the tell: those rows
  existed only to pay for a cap-shift reading of Great/Poor (Characteristic) that
  the passages do not support (see below). They are gone, and with them
  `effective_max`/`effective_min` — the buy range has one tier again. (The age →
  max-Ability-score bands live in `rules/core/abilities.json`, not here — see
  "Age → max Ability score" below.)

  *`aging_floor` removed 2026-09-15 (open-todos row 44, Norbert's decision).* The
  same correction one size down. A third limit survived the row-32 pass because it
  had a job — clamping an aged-down score, and giving the
  `excessive_aging_reduction` warning a threshold — but `ArMDE:16579` names no
  floor for aging drops at all, so the limit stated a rule the book does not: that
  a character decrepit with age can be no weaker than a freshly-built grog may
  start. Moving it further out (briefly −10) only made the invention harder to
  see. It is deleted, with the warning that depended on it; the derived score needs
  no clamp because `aging_points` is a `u8` and each drop costs more than the last,
  so the arithmetic terminates on its own. See **Aging (M6/6b6)** below.
- Implementation: `crates/arm-rules/src/characteristics.rs` —
  `CharacteristicRules` (`cost_for`, `total_cost`, `min_score`, `max_score`,
  `base_max_score`, `base_min_score`); enforced in
  `validation/scores.rs` — `validate_characteristics` (:35) (off-table
  out-of-range error, above-cap / below-floor errors against the buy range,
  overspent error, points-unspent warning). The out-of-range error is what a save
  written against the old invented rows now trips — see *Save compatibility*
  below. See the Great/Poor (Characteristic) layer below.

### Abilities

#### Ability XP advancement table ("ABILITY To Buy") — `rules/core/abilities.json`
> Advancement Table, "ABILITY To Buy" column: total XP to reach a score from
> zero — 1→5, 2→15, 3→30, … (triangular 5·n·(n+1)/2), through 20→1050.

- Source: `ArMDE:2406-2427` (header
  `ArMDE:2406`, data rows `ArMDE:2408-2427`).
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
> Supernatural) at `ArMDE:7177-7268`; alphabetical descriptions at `ArMDE:7269-7789`.

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
  (e.g. Awareness `ArMDE:7325-7328`, Magic Theory `ArMDE:7646-7649`). The five categories
  are the book's Ability types (`ArMDE:7177-7268`), taken from each entry's trailing
  `(Type)` label.
- Data: 78 abilities covering all five categories, including the full
  early-childhood restricted list (`ArMDE:2378`: Area Lore, Athletics, Awareness,
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
  > `ArMDE:4157`.

  This is an **independent per-ability property, not derived from `category`**:
  it spans General (e.g. (Area) Lore), Academic, Arcane, and all Supernatural
  abilities. Notably Penetration and most combat/General abilities are *not*
  asterisked. The flag is set from the
  `*` on each ability's `####` heading (`ArMDE:7273-7786`); `Ability.requires_training`
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
  resolve against it) in `ruleset/integrity.rs`; `validate_abilities` in `validation/scores.rs` (:315).

### Arts

#### Art XP advancement table ("ART To Buy") — `rules/core/arts.json`
> Advancement Table, "ART To Buy" column: total XP to reach a score from zero —
> 1→1, 2→3, 3→6, 4→10, 5→15, … (triangular n·(n+1)/2), through 20→210. Cheaper
> than the Ability column (which is 5× these values).

- Source: `ArMDE:2406-2427` (header
  `ArMDE:2406`, data rows `ArMDE:2408-2427` — the "ART To Buy" / "To Raise" columns).
- Data: `rules/core/arts.json` `advancement` array (scores 1-20).
- Implementation: reuses `crates/arm-rules/src/ability.rs` — `AdvancementTable`
  (the same generic table type as Abilities). Surfaced on `Ruleset` as
  `art_advancement`. A character stores whole bought Art scores
  (`Entity::art_scores`) priced from this table.
- **Shared XP pool.** Abilities and Arts are bought from **one** bank
  (`Entity::xp_pool`): the rules give apprenticeship experience as a single pool
  the magus splits freely between Arts and Abilities ("These experience points
  can be spent on Arts or Abilities", `ArMDE:2429-2433`). So there is no Art-specific
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

- Source: `ArMDE:8833-8982` — chapter
  intro `ArMDE:8833`, Techniques `ArMDE:8845-8897`, Forms `ArMDE:8899-8982`. Each Art entry cites
  its own description line range (e.g. Creo `ArMDE:8847-8863`, Ignem `ArMDE:8941-8945`).
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
  `ruleset/integrity.rs`; `validate_arts` in `validation/scores.rs` (:655).

### Effect layer (score-boosting Virtues, limit-shifting Virtues/Flaws)

Two `Effect` kinds, both **data-driven** (a `PointItem` declares `effects` and the
target ability/characteristic is named by the selection's parameter value — the
engine hardcodes no Virtue/Flaw IDs):

- `ability_bonus` — adds to an ability's *effective* score (bought + bonus,
  always computed, never stored), used whenever the Ability is used, and for display.
  It does **not** count toward a minimum-score prerequisite (`AbilityMin`,
  `AbilityCategoryScoreMin`; likewise `art_bonus` for `ArtMin`/`AnyArtMin`): Puissant
  adds its bonus "whenever you use it" (`ArMDE:4816, :4820`), and meeting a minimum
  is not a use (`ArMDE:4389`). Those prerequisites test the *held* score — the bought
  score or a granted floor (Second Sight 1, `ArMDE:4890`) — in
  `validation/prereq.rs::PrereqCtx::build` (D83.5, Norbert 2026-10-03; consistent
  with the magus-minimums general ruling below). Pinned by
  `tests/r4_prereq_score_basis.rs`.
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

- Source: `ArMDE:4814-4816`.
- Data: `rules/core/virtues_flaws.json` `virtue.puissant_ability` —
  `effects: [{ ability_bonus, param: "ability", amount: 2 }]`; `max_per_target`
  defaults to 1 ("only once for a given Ability").
- **Targets one ability instance.** The selection stores the ability id under
  `ability`; for a *parameterized* ability ((Area) Lore) it also stores the
  instance value (the area/language) under the ability's own param key, e.g.
  `params: { ability: "ability.area_lore", area: "Brandenburg" }`. Each (Area)
  Lore is a distinct Ability (`ArMDE:4816`), so "Puissant Brandenburg Lore" boosts
  that row alone, not "Berlin Lore".
- Implementation: `effective/ability.rs::ability_bonus(.., parameter)` matches
  `(ability, parameter)` — a plain ability by id, a parameterized one only when
  the selection names the same instance; a selection missing the instance key
  matches nothing. `ability_bonuses` returns a per-instance `Vec<AbilityBonus>`
  (serialized directly to the frontend by `arm-app::effective_dto`). The bonus no
  longer reaches `AbilityMin`: since D83.5 a prerequisite minimum tests the held
  score (strongest bought instance or granted floor), never a Puissant bonus.
- `ability_bonuses` iterates the **union of the Ability catalogue and the bought
  instances**, deduped on `(id, parameter)` (Issue 17 — the Ability twin of the
  Art fix below). The catalogue half surfaces a Puissant on a plain ability at 0
  bought points, which a bought-rows-only list hid entirely: a character with
  Puissant Magic Theory and no Magic Theory row saw nothing on the Abilities
  surface. The bought half is what the Art code needs no equivalent of — every
  catalogue entry carries `parameter: None`, and the match is exact, so iterating
  definitions alone would report 0 for Puissant "(Area) Lore: Brandenburg" on a
  bought Brandenburg row. A Puissant on a parameterized ability therefore surfaces
  only once its instance is bought (an unnamed instance scores 0 by the rule
  above; a named-but-unbought one is in neither half) — reported instead by
  `ability_bonus_dangling_target`, below.
- Validation: `validate_parameters` makes the expected param-key set
  target-aware — a parameterized ability target also expects its instance key
  (else `missing_param`; a stray instance key on a plain target is
  `unexpected_param`). `validate_ability_bonus_targets` flags
  `ability_bonus_dangling_target` when the targeted `(ability, parameter)` is
  neither among the character's bought abilities nor granted a floor
  (`granted_ability_floor` — Second Sight "confers the Ability Second Sight 1",
  `ArMDE:4890`; tests `tests/bve_s4_granted_ability_bonus_target.rs`) (e.g. the
  ability was later removed).
  The rule names no precondition — "choose one Ability", not "one Ability you
  have" (`ArMDE:4814-4816`) — so the target may be picked before the score exists and
  the finding is filed on `CreationPhase::Abilities`, the step where the ability
  is bought, not on the Virtues/Flaws step that names it.
- **Grant-aware since D2** (`docs/vf-audit/decisions.md`): `validate_ability_bonus_targets`
  scans *effective* selections (bought ++ House/Mythic-type/`grants_selection`
  grants) for the Virtue itself, so a granted Puissant Ability (e.g. Bonisagus's
  free choice between Puissant Magic Theory and Puissant Intrigue,
  `ArMDE:2270-2283`) dangles exactly like a bought one when its target is
  unheld. Distinct from review finding B1's bought-only exemption for the
  incompatibility/forbidden-trait checks: those ask whether the character may
  hold the Virtue at all, this asks whether its target is in a legal state — a
  grant answers the first question but says nothing about the second.
  `examples/magus_sample.json`'s Bonisagus had picked Puissant Intrigue with no
  bought Intrigue row; D2's fix surfaced it as a genuine (if minor) data defect,
  fixed by giving the character a base Intrigue score.

#### Puissant (Art) — +3 to one Art
> "You add 3 to the value of one Art whenever you use it. This means all totals in
> which the score of the Art is part of the total. … You may take this Virtue
> twice, for two different Arts."

- Source: `ArMDE:4818-4820`.
- Data: `rules/core/virtues_flaws.json` `virtue.puissant_art` —
  `category: hermetic`, `art`-domain param, `effects: [{ art_bonus, param: "art",
  amount: 3 }]`; `max_per_target` defaults to 1 ("once for a given Art"). Taking
  it for two Arts is two selections with different targets (repeatable by the
  parameterized-item rule). Being Hermetic, only magus profiles permit it. The
  "twice" is a cap on the *total* number of copies, which the model cannot
  express (`max_per_target` is per-target); see *Selection multiplicity* above.
- The `art_bonus` effect adds to an Art's *effective* score (Arts are not
  parameterized, so the target is matched by id alone). Implementation:
  `effective/art.rs::art_bonus`, `effective_art_score`, `art_bonuses` (serialized to
  the frontend by `arm-app::effective_dto`). `validation/scores.rs` folds the bonus into the
  Art score map so `ArtMin` is met by the boosted score.
- `art_bonuses` iterates the **full Art catalogue** (not just bought
  `art_scores`), emitting any nonzero effective-over-bought delta. A Puissant Art
  (or Elemental Magic form boost) applies at 0 bought points, but the UI drops an
  Art's row at score 0, so gating the list on `art_scores` would hide the badge
  until the first point is bought (Issue 13). Iterating the catalogue surfaces the
  bonus at bought-0 and naturally dedupes any duplicate bought rows.

#### Great (Characteristic) — a free +1, to a maximum of +5
> "You may raise any Characteristic that already has a score of at least +3 by
> one point, to no more than +5 … You may take this Virtue twice for the same
> Characteristic, and for more than one Characteristic."

Great Characteristic **grants the point**. The passage's verb is *raise*, and its
object is the Characteristic — the same grammar as Giant Blood's "You also gain
+1 to both Strength and Stamina" (`ArMDE:3977`), which this engine already models
as a free `characteristic_score_delta`. `ArMDE:4105` agrees rather than
conflicts: +3 is the cap on the **bought** score, and the Virtue is what carries
the character past it — by granting the point, not by unlocking a purchase. The
decisive evidence is negative: the point-buy table (`ArMDE:2346-2354`) has seven
rows, +3 through −3, and **prices no +4 or +5**. A cap-shift reading needs a cost
the book never prints.

*Corrected 2026-09-15 (open-todos row 32 / GitHub issue #4). This entry
previously asserted "Great Characteristic grants no free point" and modelled the
Virtue as a buy-cap shift, which required the four invented cost rows recorded
above. Both are gone.*

- Source: `ArMDE:3987-3989` (and
  `ArMDE:4105`).
- Data: `rules/core/virtues_flaws.json` `virtue.great_characteristic` —
  `characteristic`-domain param; `effects: [{ characteristic_score_delta_param,
  param: "characteristic", amount: 1 }]`; `max_per_target: 2`. The buy cap (+3)
  is `base_max` in `rules/core/characteristics.json`.
- Implementation: `effective/characteristic.rs::characteristic_score_bonus` sums
  the delta (its param resolved through the selection) alongside the fixed-target
  `characteristic_score_delta`; `effective/characteristic.rs::effective_characteristic_score`
  adds it to the bought score.
  `validation/scores.rs::validate_characteristic_delta_preconditions` flags a
  target whose **bought** score is below the base cap
  (`characteristic_max_base_too_low`) — the "≥ +3" precondition is
  parameter-relative, derived from `base_max` by the amount's sign, so it lives
  on the effect, not in the static `Prereq`. That gate is unchanged by the
  correction: it always read the bought score.
- **Grant-aware since D2** (`docs/vf-audit/decisions.md`): the scan is over
  *effective* selections (bought ++ House/Mythic-type/`grants_selection`
  grants) for the Virtue/Flaw itself, so a granted Great/Poor Characteristic is
  held to the "already at ±3" precondition exactly like a bought one — the
  precondition's own **target** comparison stays bought-only regardless, since
  "already has a score of at least +3" (`ArMDE:3989`) describes the
  Characteristic's bought state, not how the Virtue was acquired. Distinct from
  review finding B1's bought-only exemption for the incompatibility/
  forbidden-trait checks (see the Puissant Ability entry above for the same
  distinction, spelled out once).
- **"to no more than +5" needs no clamp of its own.** It falls out arithmetically:
  the bought score tops out at `base_max` (+3) and `max_per_target: 2` allows at
  most two grants, so +5 is the maximum reachable. Deliberately **no ceiling is
  imposed** on the effective score, because `ArMDE:3977` explicitly allows Giant
  Blood to push those same Characteristics to +6 ("This bonus may raise your
  scores in those Characteristics as high as +6"); a +5 clamp would break it.
  Pinned by `tests/data_integrity.rs::giant_blood_over_two_greats_reaches_plus_six`.

#### Poor (Characteristic) — a free −1, to a minimum of −5
> "lower one which is already −3 or lower by one point … You may take this Flaw
> twice for a single Characteristic, lowering it to −5, and multiple times for
> different Characteristics."

The exact sign-mirror of Great: a `flaw`, `amount: -1`, which *lowers the score
itself* for free.

*This was the worse half of the row-32 defect.* Modelled as a buy-floor shift, it
needed the invented −4 row, which refunded **10** points where the table's own
progression gives 6 — so the Flaw paid the player twice, once in Flaw points and
once in Characteristic points.

- Source: `ArMDE:6598-6600`.
- Data: `rules/core/virtues_flaws.json` `flaw.poor_characteristic` —
  `characteristic`-domain param; `effects: [{ characteristic_score_delta_param,
  param: "characteristic", amount: -1 }]`; `max_per_target: 2`. The buy floor
  (−3) is `base_min` in `characteristics.json`.
- Implementation: as Great above, sign-mirrored —
  `characteristic_score_bonus` sums the negative delta, and
  `validate_characteristic_delta_preconditions` flags a target whose bought score
  is above the base floor (`characteristic_min_base_too_high`). Grant-aware
  since D2, same as Great above.

#### Save compatibility after the row-32 correction
A save stores **choices, not resolved values**, so a character built under the
old table still carries whatever bought score the player entered — `Str: 4`, say,
alongside a `virtue.great_characteristic`. Under the corrected rules that bought
value is off-table, and `validate_characteristics` reports
`characteristic_out_of_range` naming the Characteristic and the legal range.

*Deliberately no migration fold.* The obvious one — rewrite bought 4 → 3 and let
the Virtue supply the point — is lossless only for the single-Great case. With
**two** Greats the free delta is not optional, so an old bought +4 would become
an effective +5: a different character. With **no** Great (a hand-edited or
already-invalid save) there is nothing to fold at all. A fold that is correct in
one shape and invents rules choices in the others is worse than a loud, precise
error the player resolves in one click, so the file is left byte-for-byte as
written until they do. `SCHEMA_VERSION` is unchanged: no shape moved.

#### Selection multiplicity — `max_per_target`
> "A Virtue or Flaw may be taken more than once only if the description
> explicitly allows it. Most Virtues and Flaws may only be taken once."

- Source: `ArMDE:2814`.

The per-`(item, params)` selection cap. `validate_duplicate_selections`
(`validation/selections.rs`, :909) errors `duplicate_selection` when a target's count exceeds the
item's `max_per_target` (default 1; Great Characteristic 2). This generalizes the
former hardcoded "at most once" rule and enforces both "Puissant once per
Ability" (`ArMDE:4816`) and "Great twice per Characteristic" (`ArMDE:3989`). Effect
integrity (`ruleset/integrity.rs::validate_effect_refs`) rejects at load any effect whose
`param` is undeclared or whose domain mismatches the effect kind.

Two independent shapes of "repeatable" therefore exist, and an item may need
both:

1. **Repeats with a different target each time** — modelled by a *parameter*.
   Two selections with different `params` are different duplicate keys, so they
   never collide; `max_per_target` stays 1 ("once for a given Ability", `ArMDE:3374`,
   `ArMDE:4816`). Items already covered this way: `virtue.affinity_ability`,
   `virtue.affinity_art`, `virtue.puissant_ability`, `virtue.puissant_art`,
   `virtue.extractor_of_form_vis` (`ArMDE:3781`),
   `virtue.master_of_form_creatures` (`ArMDE:4465`), `virtue.student_of_realm`
   (`ArMDE:5054`), `virtue.ways_of_the_land` (`ArMDE:5233`),
   `flaw.careless_with_ability` (`ArMDE:5775`),
   `flaw.flawed_parma_magica` (`ArMDE:6144`),
   `flaw.limited_magic_resistance` (`ArMDE:6348`).
2. **Repeats with no target at all** — the item carries no parameter, so every
   copy shares the one empty duplicate key and only `max_per_target` can permit
   the repeat. This is the case GitHub issue 3 reported against Improved
   Characteristics.

**"No stated ceiling" is encoded as `max_per_target: 255` (`u8::MAX`).** The
model has no "unlimited" sentinel and does not need one: a repeatable Virtue
costs at least one point per copy and the largest V/F budget in the rules is 21
Virtue / 10 Flaw points (`ArMDE:2638`), so 255 is unreachable by any legal build and
behaves exactly as "no limit" without overloading the field's meaning. A stated
ceiling is encoded literally (Quiet Magic 2, `ArMDE:4826`).

Items whose descriptor states no ceiling, each with the line that says so — all
carry `max_per_target: 255` in `rules/core/virtues_flaws.json`:

| Item | Line | Rule text (abridged) |
|---|---|---|
| `virtue.demonic_might` | `ArMDE:3665` | "may take this Virtue more than once, though it can account for no more than half of the character's total Virtues" |
| `virtue.demonic_powers` | `ArMDE:3669` | "may also take this Virtue more than once, though it can account for no more than half of the character's total Virtues" |
| `virtue.focus_power` | `ArMDE:3903` | "may be taken more than once, and the points gained may be combined" |
| `virtue.greater_power` | `ArMDE:4021` | "more than once, and the levels added together" |
| `virtue.improved_characteristics` | `ArMDE:4105` | "You may take this Virtue multiple times." |
| `virtue.lesser_power` | `ArMDE:4283` | "more than once, and the levels added together" |
| `virtue.magic_items` | `ArMDE:4349` | "you may take it more than once, though no single effect in any of your items can be greater than Level 30" |
| `virtue.mastered_spells` | `ArMDE:4474` | "You may take this Virtue multiple times." |
| `virtue.mentored_by_demons` | `ArMDE:4498` | "may purchase this Virtue multiple times, and gain 50 further experience points each time" |
| `virtue.minor_enchantments` | `ArMDE:4534` | "more than once: add the total levels together" |
| `virtue.personal_power` | `ArMDE:4724` | "more than once, and the levels added together" |
| `virtue.ritual_power` | `ArMDE:4874` | "more than once, and the levels added together" |
| `virtue.special_circumstances` | `ArMDE:5000` | "more than once, but you only gain a +3 bonus even if more than one set of circumstances applies" |
| `virtue.strong_angelic_heritage` | `ArMDE:5030` | "multiple times. Each additional time … increases by thirty the number of levels of holy powers" |
| `virtue.withstand_casting` | `ArMDE:5265` | "more than once, and withstand 1 Fatigue level for each level of the Virtue" |
| `flaw.vulnerable_casting` | `ArMDE:6997` | "may have, or acquire, this Flaw more than once, losing 1 extra Fatigue level for each level" |

Each of these is a **level-stack**: identical repeats are exactly what the
passage grants (numeric pools combine, or Fatigue/level counts add), so no
target parameter is needed and none carries one. D10 (below) additionally
requires each to carry an explicit `max_total: 255` — with no parameter, every
copy shares one duplicate key, so `max_total`'s own default would otherwise
silently override the ceiling this table states.

**Four more items used to sit in this table and no longer do**:
`virtue.greater_immunity` (`ArMDE:4015`, "with a **different immunity** each
time"), `flaw.deteriorating_power` (`ArMDE:5948`, "if the character has more
than one Power" — implicitly a different one per copy), `virtue.social_contacts`
(`ArMDE:4990`, "each time specifying a **different** social group"), and
`flaw.vulnerable_magic` (`ArMDE:7009`, "so long as a **different** condition is
specified for each"). Each of these is **vary-the-target**, not level-stack: an
identical second copy is not what the passage grants. D10 (Q4) left each with no
target parameter and no `max_per_target`, so each fell to the plain default of 1
(once), the safe interim state.

**Slice Q4b (D9 part 1) gave each its target parameter, applied early rather
than waiting for the catalogue-wide X6 sweep** — a real character can legally
take one of these twice with a different target, which the Q4 interim wrongly
refused. `flaw.deteriorating_power` is the exact shape
`flaw.slow_power`/`flaw.restricted_power`/`virtue.variable_power` already use
— a free-text `power` parameter checked against `entity.powers` — so it now
joins them in the `PER_POWER_ITEMS` table (`data_integrity.rs`), pinned by
`shipped_per_power_items_carry_a_power_target` and its neighbouring tests. The
other three name nothing else the sheet tracks, so each carries its own plain
free-text parameter instead: `flaw.vulnerable_magic`'s `condition`,
`virtue.greater_immunity`'s `hazard`, and `virtue.social_contacts`'s
`social_group` — all `domain: "text"`, `max_total: 255` ("no ceiling the rules
state"), `max_per_target` at D10's default of 1 (a second copy naming the same
target still collides). Pinned by `TEXT_TARGET_PARAM_ITEMS` and
`shipped_text_target_param_items_carry_their_target_param` /
`text_target_param_items_repeat_across_targets_but_never_within_one`
(`data_integrity.rs`). An older save holding one of the four with no value
now reports `missing_param` — see `docs/open-todos.md`'s "What an older save
still reports on open".

`flaw.flawed_parma_magica` (`ArMDE:6144`, "may purchase this Flaw more than once
**for different Forms**") and `flaw.limited_magic_resistance` (`ArMDE:6348`,
"multiple times, **for multiple Forms**") used to sit in that table and no longer
do. Each names a **Form**, so each is shape 1 above rather than shape 2: the
`form` parameter puts the Form into the duplicate key, the **default**
`max_per_target` of 1 states the real ceiling ("once per Form"), and (since D10,
below) an explicit `max_total: 255` states the other half ("any number of
different Forms") — before D10 this was the field's own default and needed no
declaration; D10 inverted that default, so it is now stated. The 255
`max_per_target` they shipped with predated the parameter and was
over-permissive — it also allowed a second, identical copy naming the *same*
Form, which neither descriptor grants. Pinned by `data_integrity.rs` —
`the_form_scoped_magic_resistance_flaws_name_their_form` and
`a_form_scoped_mr_flaw_repeats_across_forms_but_never_within_one`. An
EXPLICIT `max_per_value` cap is deliberately **not** added: with a single
parameter, `max_per_target`'s own key already *is* the Form, so a stated cap
would be one ceiling spelled twice — D10's per-value default of 1 already
agrees with `max_per_target: 1` and needs no override here.

`flaw.incompatible_arts` (`ArMDE:6290-6292`, "may be taken repeatedly with
different combinations") used to sit in the no-stated-ceiling table above and
no longer does (D81.8, `docs/vf-audit/decisions.md`). Its two combinations
used to be uncomputed text with no parameter, so copies could not be told
apart and were not capped — the one stated exception in the table's old row.
D81.8 gives each copy four real parameters (`technique_1`, `form_1`,
`technique_2`, `form_2`, domains `technique`/`form`), so copies ARE
distinguishable again and the item falls back to the **default**
`max_per_target` of 1 (an identical pair of combinations collides), while an
explicit `max_total: 255` keeps "any number of DIFFERENT combinations"
unlimited, exactly the shape `flawed_parma_magica`/`limited_magic_resistance`
above already use for a single Form. The twist here is that the duplicate key
cannot be the plain params tuple: the rulebook's two combinations are an
UNORDERED pair, so restating them with which one sits in `technique_1`/`form_1`
versus `technique_2`/`form_2` swapped is the same copy, not a second one.
`PointItem::unordered_param_groups` (`[["form_1","technique_1"],
["form_2","technique_2"]]`) names the two groups whose values
`validation/selections.rs::validate_duplicate_selections` re-keys by
`ParameterDomain` (`"technique"`/`"form"`) and compares as an unordered SET,
so the swap collides as intended. Pinned by
`crates/arm-rules/tests/d81_incompatible_arts.rs`.

**An incomplete group never collides** (`tmp/review-incompat.json` #1): an old
save may legally hold 2+ copies of this Flaw from before D81.8 added the four
parameters, each with all of them absent. `validation/selections.rs::duplicate_key`
used to skip a missing key silently while still building the rest of the role
map, so two such copies collapsed to the IDENTICAL empty `DuplicateKey` and
raised a spurious `duplicate_selection` on top of the expected four
`missing_param`s per copy. `duplicate_key` now returns `Option<DuplicateKey>`,
via the shared `group_role_map` helper: a group missing any of its keys makes
the WHOLE key `None`, and such a selection is excluded from
`validate_duplicate_selections`'s count entirely — it can never collide with
anything, complete or not. Items with no `unordered_param_groups` are
unaffected (the `None` path is only reachable when `groups` is non-empty).

**D81.15 (`docs/vf-audit/decisions.md`): a spell that touches a barred
combination is a creation-time ERROR.** "You may not use these Arts together
**even if one or both are requisites**" (`ArMDE:6292`) is not a quiet
grid/spell-list marker — the player cannot legally cast the spell at all, so
it is a blocking error, not a warning. `effective/spell.rs::barred_combinations`
folds every held Incompatible Arts copy's two `(Technique, Form)` groups
(bought-plus-granted) into a set; `effective/spell.rs::spell_touches_barred_combination`
tests a spell against that set as the cross product of `{primary technique} ∪
{Technique-class requisites}` × `{primary form} ∪ {Form-class requisites}` —
the same "a requisite for both its Technique and Form" reading
(`ArMDE:12309-12311`) `derived/casting.rs::fold_requisite` already applies to
the numeric Casting Total. `validation/magus.rs::validate_spell_incompatible_arts`
raises `ValidationIssue::CODE_SPELL_USES_INCOMPATIBLE_ARTS`
(`"spell_uses_incompatible_arts"`) per spell. Pinned by
`crates/arm-rules/tests/d81_incompatible_arts.rs`.

**D81.16 (`docs/vf-audit/decisions.md`): a copy's own groups must be pairwise
distinct.** One Incompatible Arts copy naming the SAME `(Technique, Form)`
pair in both of its two groups halves the Flaw's intended restriction
("two combinations", `ArMDE:6292`) while paying the same Minor cost. A new
`validate_param_groups_distinct` (`validation/selections.rs`) reuses
`group_role_map` to canonicalize every one of an item's declared groups within
ONE selection and errors (`CODE_PARAM_GROUPS_NOT_DISTINCT`,
`"param_groups_not_distinct"`) when two resolve to the identical role map —
generic over `unordered_param_groups`, with no item id named in the check
itself. An incomplete group is exempt from the comparison (same reasoning as
the duplicate-key fix above): `missing_param` already reports the gap.

**D83.1 (amends D81.8): two copies may not share a combination.** Verbatim
(`ArMDE:6292`): "For some reason you are completely unable to use two
combinations of Techniques and Forms. For example, you may be unable to use
Intellego Herbam and Intellego Animal. You may not use these Arts together even
if one or both are requisites. This Flaw may be taken repeatedly with different
combinations, but may not be combined with a Deficiency (see page 125)."
"Different combinations" bars a second copy from naming ANY pair the first
names, in either position — not only the whole pair of pairs D81.8 was built
as (try-out finding 21). `validation/selections.rs::validate_param_groups_shared_across_copies`
compares each copy's complete groups (`group_role_map`) against every other
copy's and raises `CODE_PARAM_GROUP_SHARED_ACROSS_COPIES`
(`"param_group_shared_across_copies"`, error) once per offending copy pair —
generic over `unordered_param_groups`, no item id in the check. A pair whose
whole canonical tuple is equal is skipped: that is `duplicate_selection`'s
finding, so one mistake draws one finding. A single Art may recur across
copies (finding 20): `validate_per_value_cap` skips a key inside a group, whose
repeat rule is the group's, so D10's default `max_per_value: 1` no longer
refuses copy 2's Creo. Pinned by `crates/arm-rules/tests/d81_incompatible_arts.rs`.

**D81.8, the exported name pairs each group.** The Markdown sheet used to
append the four Arts in key order, `Incompatible Arts (Ignem, Aquam, Creo,
Perdo)`, which hides which two combinations are barred. Now
`export/resolve.rs::template_extras` prints one entry per `unordered_param_groups`
group, Technique before Form (`export/resolve.rs::group_member_rank`):
`Incompatible Arts (Creo Ignem, Perdo Aquam)`, the same pairing as the in-app
`ParameterPicker.svelte::paramGroups`. Ungrouped values trail in key order, so an
item with no groups exports byte-identically. Pinned (EN, DE, granted table, and
an ungrouped guard) by `crates/arm-rules/tests/d81_export_incompatible_arts.rs`.

Items with a stated ceiling of two: `virtue.great_characteristic` (`ArMDE:3989`),
`virtue.quiet_magic` ("You may take this Virtue twice, and eliminate the penalty
altogether", `ArMDE:4826`), `flaw.poor_characteristic` (`ArMDE:6600`),
`flaw.weak_characteristics` (`ArMDE:7058`) — all `max_per_target: 2`. The two
parameterized ones, `great_characteristic` and `poor_characteristic`, ALSO
declare their own `characteristic` parameter's `max_per_value: 255`: without it,
D10's per-value default of 1 would wrongly reject the legal "twice for the same
Characteristic" repeat as a second copy naming one value — see *Selection
multiplicity — `max_total`* below for why the unparameterized two
(`quiet_magic`, `weak_characteristics`) need the analogous `max_total: 2` fix
instead.

#### Selection multiplicity — `max_total`
> "You may take this Virtue twice, for two different Arts."

- Source: `ArMDE:3378` (Affinity with
  Art), `ArMDE:4820` (Puissant Art).

The per-item, ALL-targets selection cap — distinct from `max_per_target`
above, which only catches copies sharing one identical `(item, params)`
target. `validate_total_selection_cap` (`validation/selections.rs`) counts
every copy of an item across every distinct parameter target, over the folded
bought-plus-granted selection list (`effective.rs`) — so a House-granted copy
counts against the same ceiling as one the player buys — and errors when the
count exceeds `max_total`.

Two shapes need it, and neither was expressible with `max_per_target` alone:

1. **"Twice, for two different Arts."** One copy per Art is legal
   (`max_per_target: 1`), but the item as a whole may only appear twice
   (`max_total: 2`). Without the total cap, a build could legally take
   Affinity/Puissant copies for three or more different Arts, since each Art
   is a distinct duplicate key.
2. **Forbidden repetition with a per-copy target.** `virtue.inoffensive_to_beings`,
   `flaw.offensive_to_beings`, `flaw.unbearable_to_beings`, and
   `flaw.fish_out_of_water_terrain` each carry a `being`/`terrain` parameter, so
   two selections naming two different targets are two different duplicate
   keys and do not collide under `max_per_target` — even though the rulebook
   forbids taking the item more than once at all, full stop. `max_total: 1` is
   what actually enforces "not more than once"; `max_per_target` is left at its
   default of 1 too (no repeat at the *same* target either), so both fields are
   set on these four items and both are load-bearing. (The three `being`
   parameters are `enumerated` and the `terrain` one is `text` — see *Enumerated
   parameter domain* below — but that distinction is irrelevant here: both
   domains make the target part of the duplicate key, which is the only property
   `max_total` is compensating for.)

**D10 (Norbert, 2026-09-21): "if only once, it should be only once" — the
default is inverted.**
> "A Virtue or Flaw may be taken more than once only if the description
> explicitly allows it. Most Virtues and Flaws may only be taken once."

- Source: `ArMDE:2814`.

Before D10, absent `max_total` meant `u8::MAX` — "no stated ceiling" — which is
the exact inverse of the book's own default. `default_max_total` (`types.rs`)
now returns 1: **absent means once**, and a repeat must be declared, the same
inversion `ParameterDef::max_per_value` gets below. An item whose descriptor
explicitly grants repeats across different targets must now say so with an
explicit `max_total: 255` (unlimited) or a literal number; the machinery
itself did not change — `validate_total_selection_cap` still enforces
whatever value the field holds, grant-aware exactly as before — only the
value the field holds when nothing is written changed. **No `SCHEMA_VERSION`
bump**: the save format is untouched, only what validation says about a given
count. A save written before D10 that legitimately held several copies of an
item the new default now caps at 1 is not migrated — it loads, and
`Enforced` blocks / `Advisory` warns / `Silent` suppresses, same as any other
finding.

A stated ceiling is encoded literally (Quiet Magic 2, `ArMDE:4826`).

Items whose descriptor caps the TOTAL number of copies, each with the line
that says so — all carry `max_total` in `rules/core/virtues_flaws.json`:

| Item | Line | Rule text (abridged) |
|---|---|---|
| `virtue.inoffensive_to_beings` | `ArMDE:4139` | "You may not take this Virtue more than once" |
| `flaw.offensive_to_beings` | `ArMDE:6530` | "You may not take this Flaw more than once" |
| `flaw.unbearable_to_beings` | `ArMDE:6897` | "You may not take this Flaw more than once" |
| `flaw.fish_out_of_water_terrain` | `ArMDE:6132` | "This Flaw may only be taken once, because taking it more than once makes it less serious, rather than more" |
| `virtue.affinity_art` | `ArMDE:3378` | "You may take this Virtue twice, for two different Arts" |
| `virtue.puissant_art` | `ArMDE:4820` | "You may take this Virtue twice, for two different Arts" |
| `flaw.false_power` | `ArMDE:6096` | "in each subsequent instance as a Minor Flaw rather than a Major one" — only the FIRST instance is this (Major) entry; see the section below |

**Every level-stack item in the `max_per_target` "no stated ceiling" table
above now ALSO carries the matching `max_total` (255, or 2 for `quiet_magic` /
`weak_characteristics`)**, for the reason given there: with no parameter,
`max_per_target` and `max_total` govern the identical set of copies, so
D10's new `max_total` default of 1 would otherwise silently override a
higher `max_per_target` the data still states. Pinned by
`shipped_repeatable_items_carry_their_rulebook_ceiling`'s `max_total`
assertion (`data_integrity.rs`).

**Q4's declared-repeaters sweep — the 42 entries D10 measured as riding the
old unlimited default (`jq`: `parameters != null and max_total == null`),
each read against its own passage.** 18 explicitly permit repeats across
different targets and now carry `max_total: 255`:

| Item | Line | Rule text (abridged) |
|---|---|---|
| `virtue.affinity_ability` | `ArMDE:3374` | "may take it again for different Abilities" |
| `virtue.puissant_ability` | `ArMDE:4816` | "may take it more than once for different Abilities" |
| `virtue.extractor_of_form_vis` | `ArMDE:3781` | "may be taken multiple times (once for each Form)" |
| `virtue.master_of_form_creatures` | `ArMDE:4465` | "this Virtue may be taken multiple times, once for each Form" |
| `virtue.student_of_realm` | `ArMDE:5054` | "may take this Virtue multiple times, for a different realm each time" |
| `virtue.ways_of_the_land` | `ArMDE:5233` | "may choose this Virtue multiple times, for different types of terrain" |
| `virtue.great_characteristic` | `ArMDE:3989` | "twice for the same Characteristic, and for more than one Characteristic" — `max_per_target: 2` stays, this states the OTHER axis |
| `virtue.learn_ability_from_mistakes` | `ArMDE:4243` | "may take this Virtue several times, once for each Ability chosen" |
| `virtue.folk_magic` | `ArMDE:3919` | "pick this Virtue more than once… category"; see the `max_per_value` fix below for its two axes |
| `virtue.variable_power` | `ArMDE:5205` | "may be taken more than once, if the character has more than one power" |
| `flaw.careless_with_ability` | `ArMDE:5775` | "may be taken more than once; each time, it applies to a different Ability" |
| `flaw.false_power_minor` | `ArMDE:6096` | "may be taken multiple times, once for each appropriate Supernatural Virtue" |
| `flaw.flawed_parma_magica` | `ArMDE:6144` | see above |
| `flaw.limited_magic_resistance` | `ArMDE:6348` | see above |
| `flaw.necessary_realm_aura_for_ability` | `ArMDE:6482` | "once for any particular Ability" — unlimited across abilities; see the `max_per_value` fix below for its `realm` axis |
| `flaw.poor_characteristic` | `ArMDE:6600` | "twice for a single Characteristic… and multiple times for different Characteristics" — `max_per_target: 2` stays, this states the OTHER axis |
| `flaw.restricted_power` | `ArMDE:6689` | "may be taken once for each power the character possesses" |
| `flaw.slow_power` | `ArMDE:6761` | "may be taken more than once, if the character has multiple powers, but not more than once for a single power" |

**24 do not**, and stay at the new default (1) — no data change. Verified
directly against the passage rather than assumed, since the pre-D10 measurement
only counted which entries rode the unlimited default, not which of them the
book actually licenses to repeat: `flaw.anchored_to_the_land`,
`flaw.bound_to_realm`, `flaw.bound_to_role_role`, `flaw.deficient_form`,
`flaw.deficient_technique`, `flaw.form_monstrosity`,
`flaw.hunger_for_form_magic`, `flaw.magical_being_companion`,
`flaw.realm_stigmatic`, `flaw.servant_of_the_land` (confirms F-522's fix),
`virtue.academic_concentration_subject`, `virtue.alluring_to_beings`,
`virtue.aptitude_for_sin`, `virtue.cautious_with_ability`,
`virtue.doctor_in_faculty`, `virtue.enchanting_ability`,
`virtue.imbued_with_the_spirit_of_form`, `virtue.land_regio_network`,
`virtue.mythic_blood`, `virtue.perfect_eye_for_commodity`,
`virtue.voice_of_the_land`. **Three of these correct an assumption the D10
ruling and the M0 measurement both named as a "classic repeater" — verifying
rather than trusting turned up the opposite of what was expected:**
`virtue.deft_form` (`ArMDE:3645-3648`) states no repeat permission anywhere in
the book (checked its heading, TOC entry, index, and every other-book
cross-reference — none states it may be taken more than once), and
`virtue.major_magical_focus` / `virtue.minor_magical_focus`
(`ArMDE:4405`) are affirmatively **forbidden** to repeat: "A character can have
only one Magical Focus, either major or minor, regardless of the source of the
focus" — which is also why the two are mutually `incompatible_with` each other.
Verdict table: `tmp/q4-verdicts.md` (gitignored; the durable record is this
section and the tests it is pinned by).

#### Selection multiplicity — `max_per_value` (one key, not the whole tuple)
> "A character may take this Flaw once for any particular Ability." — Necessary
> (Realm) Aura for (Ability), `ArMDE:6482`.

- Source: `ArMDE:6482` (the descriptor runs `ArMDE:6480-6487`).
- Engine: `ParameterDef::max_per_value` (`types.rs`), enforced by
  `validation/selections.rs::validate_per_value_cap` (issue code
  `too_many_for_param_value`, `ValidationIssue::CODE_TOO_MANY_FOR_PARAM_VALUE`);
  load-time shape in `ruleset/integrity.rs::validate_parameter_defs`; messages
  `issue-too_many_for_param_value` (`locales/en|de/main.ftl`).
- Data: `rules/core/virtues_flaws.json` —
  `flaw.necessary_realm_aura_for_ability`'s **`ability`** parameter,
  `"max_per_value": 1`. Its `realm` parameter carries `"max_per_value": 255`
  (D10): `ArMDE:6482` caps repeats on the Ability alone, and nothing limits how
  many Realms the Flaw may be held across, but D10's new per-value default of 1
  would otherwise wrongly reject two copies that share a Realm while naming
  different Abilities. `virtue.folk_magic`'s `category` and `realm` parameters
  carry the same explicit `"max_per_value": 255` for the identical reason — see
  *Enumerated parameter domain* below, and `ArMDE:3919`'s "you can align it to
  the same Realm as before or pick a different one".
  `virtue.focus_power`'s `focus` parameter carries `"max_per_value": 255`
  (D83.2, try-out finding 25): "This Virtue may be taken more than once, and the
  points gained may be combined." (`ArMDE:3903`), and D81.9 lets copies share a
  scope, so two copies on ONE focus draw no repeat finding of any code.
- Sweep (D83.2) of the other repeatable (`max_total` 255) text-parameter items,
  which keep the default cap of 1 because their descriptors demand a different
  value per copy: `flaw.restricted_power` ("once for each power the character
  possesses", `ArMDE:6689`), `flaw.slow_power` ("not more than once for a single
  power", `ArMDE:6761`), `flaw.vulnerable_magic` ("so long as a different
  condition is specified for each", `ArMDE:7009`), `virtue.greater_immunity`
  ("with a different immunity each time", `ArMDE:4015`), `virtue.social_contacts`
  ("each time specifying a different social group", `ArMDE:4990`),
  `virtue.variable_power` ("it only applies once to a single power",
  `ArMDE:5205`), `virtue.ways_of_the_land` ("for different types of terrain",
  `ArMDE:5233`). Pending Norbert, also left at 1: `flaw.deteriorating_power`
  (`ArMDE:5948`), `virtue.potent_magic_major`/`_minor` (`ArMDE:4742`),
  `virtue.special_circumstances` (`ArMDE:5000`).
- **Free text compares folded (D83.2).** A `text`-domain value and a
  parameterized Ability's instance text are compared by
  `catalogue.rs::fold_free_text` (case-folded, trimmed, inner whitespace
  collapsed) in both `validate_duplicate_selections` and `validate_per_value_cap`,
  via `validation/selections.rs::comparable_params`, so "Fire", "fire" and
  " Fire  " are one value and no cap is dodged by retyping. Ids (`enumerated`,
  `ability`, `art`, …) and `multi_ref` sets compare exactly. Stored values stay
  as typed; the finding names the first copy's own spelling. Tests:
  `tests/r2_text_param_compare.rs`,
  `focus_power_may_be_taken_twice_for_the_same_focus` and
  `focus_power_twice_for_one_focus_spelled_differently_is_still_legal`
  (`tests/x6c_label_parameters.rs`).
- Tests: `two_copies_may_not_share_a_capped_parameter_value`,
  `copies_naming_different_values_of_a_capped_parameter_are_clean`,
  `two_instances_of_one_parameterized_ability_are_two_targets`,
  `two_copies_naming_one_instance_of_a_parameterized_ability_trip_the_cap`,
  `a_parameter_with_no_stated_cap_defaults_to_once_per_value`,
  `an_identical_repeat_stays_the_duplicate_selections_finding`,
  `an_identical_repeat_draws_exactly_one_finding`,
  `legal_identical_repeats_still_count_toward_the_per_value_cap`,
  `granted_copies_count_toward_the_per_value_cap`,
  `the_per_value_finding_names_the_key_and_the_value`,
  `the_per_value_finding_names_the_instance_it_counted`,
  `the_per_value_finding_carries_an_empty_instance_for_a_plain_ability`,
  `an_explicit_unlimited_per_value_cap_is_a_sentinel_and_not_the_number_255`
  (`validation/mod.rs`),
  `a_per_value_cap_of_zero_is_rejected`,
  `across_copies_constraints_on_a_spell_parameter_are_rejected`,
  `across_copies_constraints_on_a_point_item_parameter_are_accepted` (`ruleset.rs`),
  `necessary_aura_is_taken_once_for_any_particular_ability`
  (`tests/data_integrity.rs`), and `names the per-parameter-value cap in both
  locales, naming the value` (`ui/src/lib/i18n.test.ts`).

The third and narrowest of the three multiplicity axes, and the only one that
binds **one named parameter key** rather than an item-wide grouping:

| Axis | Field | Groups by |
|---|---|---|
| one identical target | `PointItem::max_per_target` | `(item_ref, params)` — the whole tuple |
| copies in total | `PointItem::max_total` | `item_ref` alone |
| per named value | `ParameterDef::max_per_value` | `(item_ref, one key's target)` |

**The cap is per Ability *target*, not per Ability id.** An `ability`-domain
parameter aimed at a *parameterized* Ability names its target with two keys —
the ability id under the parameter's own key, plus the instance under the
Ability's (`{ ability: "ability.craft", craft: "Carpentry" }`), the pair
`validate_selection_parameters` makes mandatory. Craft (Carpentry) and Craft
(Blacksmith) are two different **Abilities**, and `ArMDE:6484` explicitly
contemplates this Flaw "applied to Craft or Profession Abilities", so counting
both as `ability.craft` rejected a character the rules permit — an error, which
blocks the guided wizard in Enforced mode. The catalogue's seven parameterized
Abilities (`rules/core/abilities.json`: Area Lore, Craft, Dead Language, Living
Language, Mystery Cult Lore, Organization Lore, Profession) were all affected.
The composition lives once, in
`validation/selections.rs::ability_instance`, shared with
`validate_ability_bonus_targets` — the same `(ability, instance)` pair
`Entity::ability_scores` rows are keyed by and the frontend's
`usedAbilityTargets` composes.

**The finding names the target it counted.** Because the count is taken per
`(ability, instance)`, naming the Ability alone left the two halves of the
sentence disagreeing: with Craft (Carpentry) twice and Craft (Blacksmith) once,
"selected 2 times for Craft" sat beside three Craft rows. So the issue carries
the instance as its own `parameter` arg — spelled as
`validate_ability_bonus_targets` spells it, and emitted unconditionally (empty
for an Ability that takes none). It is a **qualifier, not a label**: no Fluent
message interpolates it, because where an instance sits inside an Ability's name
is that Ability's localized template and differs per language (`{area} Lore` vs
`{area}-Kunde`, `Craft: {craft}` vs `Handwerk: {craft}`). The frontend folds the
pair into one name through `derive.ts::abilityDisplayName`, declared once in
`ABILITY_INSTANCE_ARG` beside `ENUM_ARG_FLUENT_PREFIX`.

**Why the two older axes cannot state it.** The Flaw declares two parameters,
`realm` and `ability`, and `ArMDE:6482` caps repeats on the Ability alone —
nothing in `ArMDE:6480-6487` limits how many Realms a character may hold the
Flaw across. So `(ability: awareness, realm: divine)` and `(ability: awareness,
realm: faerie)` are different tuples, collide in no `max_per_target` key, and
before this axis both validated clean: the character held the Flaw twice for one
Ability and had spent Flaw points the rules do not allow. `max_total` cannot help
either — grouping by `item_ref` alone, it can say "twice overall" but never "twice
per Ability".

**Why the gap survived so long.** The cap mechanism predates multi-parameter
items. For every single-parameter item it was designed against, "one copy per
target" and "one copy per named value" are the same sentence, because the tuple
*is* the value. Only a second parameter separates them, and the catalogue ships
exactly two multi-parameter items — this Flaw and `virtue.folk_magic`, whose own
`ArMDE:3919` explicitly *permits* the divergence ("you can align it to the same
Realm as before or pick a different one"). So this is the one entry where the
divergence is a defect rather than a rule.

**The cap lives on the parameter, not on the item.** An item-level key→max map
would repeat a key name `ParameterDef` already owns, and a typo in it would sit
in `rules/` looking enforced; here the cap *is* on the key, so naming a
parameter the item does not declare is unrepresentable rather than merely
rejected. It also sits beside `at_most_one_of`, the other per-key constraint
judged across an item's copies. Load-time integrity rejects the two authoring
slips the shape still permits. `max_per_value: 0` forbids every value the
parameter could name and leaves the item unfillable — the mirror of the
`at_most_one_of` group that excludes nothing. And either constraint declared on
a **spell** parameter is rejected outright: both are read only by validators
that resolve a `ruleset.point_items` entry, while a spell's one parameter value
is checked alone by `validation/magus.rs::validate_spell_parameter`, so a cap
there would sit in `rules/` looking enforced and enforce nothing. That gate keys
on the declaring record rather than on the parameter's domain — see
`ruleset/integrity.rs::CopiesJudgedTogether` — which is what distinguishes it
from the `require_categories` / `require_possessed` / `require_power` gates
beside it.

**D10 inverted this field's default too: absent = 1 (once), not `u8::MAX`.**
Before D10, absent meant "no stated ceiling"; now a repeat along one value
must be declared explicitly with `max_per_value: 255` (the sentinel for "the
rulebook states no limit" — see `necessary_realm_aura_for_ability`'s `realm`
axis and `folk_magic`'s two axes above). The **enforcement** side did not
change: the validator tests for the `u8::MAX` sentinel rather than comparing
against it, so a crafted save holding 256 copies naming one EXPLICITLY
unlimited value cannot trip a cap the rules never state. That distinction
remains load-bearing: a save is a declared hostile-input surface, so the
sentinel is checked explicitly — which also bounds the counting to parameters
that actually declare (now: explicitly override) a cap.

**`max_per_target` stays, at its default of 1.** The two are not redundant,
because `validate_per_value_cap` counts only the copies its neighbour did not
already report: each distinct tuple contributes at most `max_per_target` copies
to the per-value count. At the default ceiling of 1, two copies with an
identical `(realm, ability)` tuple are `max_per_target`'s finding and only its
finding, while copies that differ in Realm are this axis's and only this one.
One mistake draws one finding — the same division of labour
`validate_possessed_param_targets` keeps with the same neighbour. Removing the
default `max_per_target` would legalise an exact duplicate row; raising it would
too.

**Why "at most `max_per_target`" and not "one copy per tuple".** An
unconditional collapse to one copy per distinct tuple looks equivalent and is,
but only while the ceiling is 1. Above it there is no neighbouring finding to
defer to: at `max_per_target: 2`, `validate_duplicate_selections` passes two
identical copies, and a collapse would fold them into a single tuple so the
per-value cap saw one copy and reported nothing — a stated cap silently
unenforced. Catalogue shape is **data, never code**, so reaching that state is a
data-only edit to `rules/core/virtues_flaws.json`; many shipped items already
carry `max_per_target: 255` and three carry 2. Capping each tuple's contribution
at the ceiling keeps both halves: the excess above it is the duplicate check's
finding and is not counted twice, and everything at or below it is a legal
repeat nobody else reports and so must count here.

**This message names its value, unlike `exclusive_param_values`.** The
exclusion message deliberately names no value (see *Realm parameter domain*
below): a group's members can span domains, and a per-domain argument family
would make Fluent throw wherever a domain does not supply one. Here there is
exactly one offending value, in exactly one domain — the parameter's own — and
"taken twice for something" is not a finding a player can act on, so `value`
is an argument and the message reads "twice for Awareness". The label comes from
the frontend's existing `resolveIssueArgValue`, which already turns a rules id
into its localized name, so no new label machinery was needed.

**Still unmodelled, and recorded rather than approximated:** the same entry's
sibling restriction "You may not take Student of (Realm) and Puissant Ability
for the same Lore" (`ArMDE:5054`) is a cross-**item** constraint over a parameter
value. `max_per_value` binds one item's own copies and cannot express it; no
mechanism relates two *different* items' parameter values at all.

#### Selection multiplicity — False Power's subsequent copies are Minor
> "This Flaw may be taken multiple times, once for each appropriate
> Supernatural Virtue that the character possesses, but in each subsequent
> instance as a Minor Flaw rather than a Major one."

- Source: `ArMDE:6096` (the repeat
  sentence; the entry runs `ArMDE:6080-6096`, and both catalogue entries carry
  `source.lines = [6080, 6096]`, D30.2's "ends on the last non-blank body
  line" convention).
- Data: `rules/core/virtues_flaws.json` — `flaw.false_power` (Major,
  `max_total: 1`) and `flaw.false_power_minor` (Minor,
  `prerequisites: { kind: has, value: flaw.false_power }`, no
  `incompatible_with`). Both carry the `virtue` target parameter and the default
  `max_per_target: 1`; see *False Power names the Virtue it taints* below. Text
  in `rules/i18n/en|de/virtues_flaws.json`.
- Tests: `false_power_ships_as_a_coexisting_major_plus_minor_pair`,
  `a_second_major_false_power_is_capped`,
  `a_minor_false_power_without_the_major_is_a_missing_prerequisite`,
  `false_power_taken_three_times_costs_three_plus_one_plus_one`,
  `a_granted_major_false_power_satisfies_the_minor_prerequisite`
  (`crates/arm-rules/tests/data_integrity.rs`; the Major also sits in
  `TOTAL_CAP_ITEMS`).

`magnitude` is a property of the **catalogue entry**, never of a selection, so a
single repeatable entry cannot change price between copies — a second copy of a
Major entry is charged 3 points, not the 1 the book asks for. The rule is
therefore expressed as a **pair of entries**, the same shape the `*Major or
Minor*` items use: the first instance is `flaw.false_power` (Major, 3 points,
capped at one copy by `max_total: 1`), and every subsequent instance is
`flaw.false_power_minor` (Minor, 1 point, freely repeatable). One Major plus two
Minors therefore costs 3 + 1 + 1 = 5 Flaw points, which is exactly what `ArMDE:6096`
prices. Both entries carry `tainted: true` (the descriptor is *Major,
Supernatural, Tainted*, `ArMDE:6081`), so every copy feeds the half-of-Flaw-points
Tainted cap.

**The two must coexist, so they are NOT mutually `incompatible_with`.** This is
the one place the pair diverges from the dual-magnitude convention above: a
Major/Minor *variant* pair (Amorphous, Magical Focus) offers a choice of one
magnitude, whereas False Power's Minor copies exist only *after* the Major one.
The `has` prerequisite is what encodes that ordering, and because
`validate_prerequisites` is handed the folded grant list, a **granted** Major
(e.g. a warping-owed Major Flaw slot filled with False Power, `ArMDE:16561`)
satisfies it just as a bought one does.

**Why the ids are asymmetric (`flaw.false_power` + `flaw.false_power_minor`),
and why a later sweep must not "tidy" them into `_major`/`_minor`.** Two
independent reasons, either one sufficient:

1. **Saves.** `flaw.false_power` is already shipped and held by existing saves.
   Saves store choices, not resolved values, so renaming the id would orphan
   every row that holds it — for cosmetic symmetry only.
2. **The load would fail.** `validate_magnitude_variant_exclusivity`
   (`ruleset/integrity.rs`) detects a variant pair purely by id shape: for every
   item whose name ends in `_major` (or starts with `major_`) it looks up the
   `_minor` sibling and, when **both** exist, *requires* each to list the other
   in `incompatible_with`, failing the ruleset load otherwise. Naming this pair
   `flaw.false_power_major` / `flaw.false_power_minor` would therefore demand a
   mutual incompatibility that would make the rule unimplementable — the two
   entries must be selectable together. The asymmetric ids are what keep the pair
   outside that check.

**The rest of `ArMDE:6096` is enforced separately**, by the target parameter both
entries now carry — see *False Power names the Virtue it taints* immediately
below. Between them the two sections cover the whole sentence: the per-copy
magnitude change here, the per-Virtue target there.

#### False Power names the Virtue it taints (B9)
> "This Flaw may be taken multiple times, once for each appropriate
> Supernatural Virtue that the character possesses, but in each subsequent
> instance as a Minor Flaw rather than a Major one. Also note that this Flaw
> cannot apply to Supernatural Virtues that are affiliated to the Infernal realm
> in the first place, and the troupe may not allow it to apply to Virtues
> derived from the Divine."

> "This Flaw can apply to Supernatural Virtues that define the character's
> background, like Faerie Blood, Diedne Magic, or even The Gift."

- Source: `ArMDE:6096` (the repeat,
  possession and Infernal clauses) and `ArMDE:6082` (which Virtues count).
- Data: `rules/core/virtues_flaws.json` — both False Power entries declare
  `parameters: [{ key: "virtue", domain: "item", require_categories:
  ["supernatural"], allow_ids: ["virtue.diedne_magic", "virtue.the_gift"],
  require_possessed: true, forbid_tainted: true }]` (D34, C2 —
  `require_categories` used to read `["hermetic", "special", "supernatural"]`;
  see *The whitelist: D34* below), and the Minor entry's `max_per_target`
  dropped from `255` to the default `1`. Label `param-label-virtue` in
  `locales/en|de/main.ftl`.
- Engine: `ParameterDef::require_possessed` / `::forbid_tainted` /
  `::allow_ids` (`types.rs`);
  `validation/selections.rs::validate_possessed_param_targets` (possession and
  the one-claim-per-Virtue rule) and `::param_value_resolves` (the Tainted
  narrowing and the `allow_ids` OR); load gates in
  `ruleset/integrity.rs::validate_parameter_defs` and `::validate_allow_ids`
  (the catalogue half, D34).
- Frontend: `ParameterDef.require_possessed` / `.forbid_tainted` /
  `.allow_ids` (`ui/src/lib/types.ts`), `itemOptionsFor` + `heldItemRefs`
  (`ui/src/lib/components/ParameterPicker.svelte`).
- Tests: `false_power_names_the_supernatural_virtue_it_taints`,
  `false_power_admits_the_named_ids_but_not_an_arbitrary_other_hermetic_virtue`,
  `false_power_cannot_taint_a_virtue_the_character_lacks`,
  `false_power_cannot_taint_an_already_infernal_virtue`,
  `a_major_and_a_minor_false_power_cannot_taint_the_same_virtue`,
  `a_sufi_taken_as_social_status_is_still_a_possessed_false_power_target`
  (`tests/data_integrity.rs`);
  `a_require_possessed_target_the_character_lacks_is_flagged`,
  `an_unpossessed_target_is_filed_on_the_virtues_flaws_step`,
  `a_house_granted_target_counts_as_possessed`,
  `two_different_items_cannot_claim_the_same_possessed_target`,
  `two_items_claiming_different_possessed_targets_are_clean`,
  `a_param_without_require_possessed_admits_an_unheld_target`,
  `a_tainted_item_is_outside_a_forbid_tainted_domain`,
  `a_tainted_item_resolves_when_forbid_tainted_is_not_declared`
  (`validation/mod.rs`);
  `require_possessed_and_forbid_tainted_default_false_and_are_omitted_when_false`
  (`types.rs`); `possession_and_taint_flags_on_a_non_item_param_are_rejected`,
  `possession_and_taint_flags_load_on_an_item_param`,
  `allow_ids_on_a_non_item_param_is_rejected`,
  `allow_ids_member_naming_an_unknown_item_is_rejected`,
  `allow_ids_member_resolving_loads_clean` (`ruleset.rs`); and
  `offers only Virtues the character holds when the parameter requires
  possession` / `counts a granted row as possessed in the picker` /
  `additionally offers whitelisted ids the category alone would refuse (D34)`
  (`ui/src/lib/components/ParameterPicker.test.ts`).

**The whitelist: D34.** `ArMDE:6096` says
"Supernatural Virtue", but `ArMDE:6082` names three examples outright — "Faerie
Blood, Diedne Magic, or even The Gift" — and in this catalogue those carry
`supernatural`, `hermetic` and `special` respectively. A bare
`require_categories: ["supernatural"]` would have refused two Virtues the source
explicitly permits, which is wrong rules output — but widening the CATEGORY
(the entry's original `["hermetic", "special", "supernatural"]`) admits every
OTHER Virtue either category carries too, 56 Hermetic Virtues where the book
names one, which is over-permission just as wrong. D34's ruling: keep the
category axis at `["supernatural"]` alone (Faerie Blood needs no further
entry — it is already `supernatural`) and admit the other two examples by a
**one-id whitelist**, `allow_ids: ["virtue.diedne_magic", "virtue.the_gift"]`
— `ParameterDef::allow_ids`, additive to `require_categories`: a value
resolves if EITHER test passes (`validation::selections::param_value_resolves`).
`ArMDE:6082`'s "like" is an open list and the whitelist closes it, so a future
background-defining Hermetic Virtue must be added by hand — the deliberate
trade: a missing id is visible the moment someone looks for it, where the 55
wrongly-admitted ones were invisible.

**Possession is `Prereq::Has`'s notion, not a second one.** The check reads
`validation::prereq::PrereqCtx::present_ids` — bought selections ++ granted rows
— hoisted to `pub(crate)` by B1 for exactly this. A House-granted, Mythic-type
or warping-filled Supernatural Virtue is genuinely held, so it is a legal
target, and the engine keeps one definition of "held" rather than two that can
drift.

**A `taken_as` reading is not consulted, deliberately.** Sufi is "either as a
Minor Social Status Virtue or a Minor Supernatural Virtue" (`ArMDE:5083`). A Sufi
taken as Social Status is still a Virtue the character *possesses*, so False
Power may name it, and `require_categories` admits it through its Supernatural
membership either way — B8 already established that a parameter value is a bare
`Id` naming an item, not a `Selection` of one, so `PointItem::categories_for`
cannot be reached from the domain half. Making the *possession* half stricter
than the *domain* half would put two different answers to "is this Virtue
Supernatural for this character" inside one parameter. One lenient answer, with
the troupe adjudicating the rest, is the better trade;
`a_sufi_taken_as_social_status_is_still_a_possessed_false_power_target` pins it.

**Why a second code rather than `max_per_target`.** "**Once** for each
appropriate Supernatural Virtue" forbids two copies naming one Virtue.
`max_per_target` cannot express it: its duplicate key is `(item_ref, params)`,
and `flaw.false_power` / `flaw.false_power_minor` are different ids, so a Major
and a Minor both naming Second Sight collide in no key at all. Hence
`param_target_already_claimed`, raised by the same validator against the
**later** selection — one decision, one finding. A repeat of the *same* id is
left to `max_per_target` (now 1 on both entries), or one mistake would draw two
findings.

**Why the Tainted refusal is `unknown_param_value` and not a code of its own.**
"Cannot apply to Supernatural Virtues that are affiliated to the Infernal realm
in the first place" is a statement about the *catalogue entry*, not about the
character: `PointItem::tainted` already records exactly that affiliation
(`ArMDE:2998-3002`). It needs no `&Entity`, so it narrows the domain in
`param_value_resolves` on `require_categories`' precedent — the narrowing IS the
domain — while `require_possessed` genuinely cannot answer without the entity
and therefore gets codes of its own.

**Filed on `virtues_flaws`, and why that is safe here.**
`validate_ability_bonus_targets` files its dangling-target error on
`CreationPhase::Abilities` even though the offending value is a Virtue's
parameter, because Puissant Ability's target is bought on a *later* step and
filing on V/F blocked a step that could not offer the remedy. Both remedies for
this issue — name a different Virtue, or buy the one named — are on the V/F step
itself, so same-step filing cannot deadlock the wizard. Do not "align" the two
validators' phases; that reintroduces the deadlock.

**Deliberately NOT implemented: the Divine clause.** The same sentence ends "and
the troupe may not allow it to apply to Virtues derived from the Divine"
(`ArMDE:6096`). *May not allow* is troupe discretion, not a rule the engine can
decide — there is no Divine-affiliation flag on a Virtue in the first place, and
inventing one would encode a ruling the book leaves open. Recorded here rather
than implemented.

**The name says which Virtue.** Both i18n names gained a `{virtue}`
placeholder — "False Power (Major): {virtue}" / "Falsche Macht (Groß):
{virtue}" — because the defect this row records is precisely that the copies
"do not name which Supernatural Virtue they taint". Without it the in-app row
would read "False Power (Minor)" three times over: `selectionDisplayName`
(`ui/src/lib/derive.ts`) fills placeholders and has no extras-append path. The
Markdown sheet would have shown the Virtue either way — `Doc::fill_template`
appends a value the template never mentions — so the placeholder is what keeps
app and sheet reading identically. Pinned by
`both_locales_name_the_virtue_a_false_power_taints`.

**Save impact.** An existing save holding a paramless False Power row now raises
`missing_param` on open, and the Minor entry's tightened `max_per_target` can
raise `duplicate_selection` on a save with two unnamed Minor copies. No data is
lost — saves store choices and the engine only reports — and the player clears
both by naming the Virtue each copy taints. No migration: `SCHEMA_VERSION` stays
16, because nothing about the save *shape* changed.

#### Enumerated parameter domain — a closed list the rulebook prints
> "He can only create spells in one narrow area, which must be one of the
> following four options" — Folk Magic, `ArMDE:3909`, the options printed
> `ArMDE:3911-3917`; and "You may pick this Virtue more than once, to acquire
> expertise in a different category of spells" — `ArMDE:3919`.

> "This Virtue is associated with one of five classes of beings: animals, divine
> beings, faeries, demons, or magical creatures." — Inoffensive to (Beings),
> `ArMDE:4135`.

> "This Flaw is associated with one of six classes of beings: animals, mundane
> humans, divine beings, faeries, demons, or magical creatures." — Offensive to
> (Beings), `ArMDE:6526`.

> "This Flaw is associated with one of three classes of beings: mundane humans,
> demons, or divine beings." — Unbearable to (Beings), `ArMDE:6893`.

- Source: `ArMDE:3909`, `ArMDE:3911-3917`,
  `ArMDE:3919`, `ArMDE:4135`, `ArMDE:6526`, `ArMDE:6893`.
- Engine: `ParameterDomain::Enumerated` and `ParameterDef.values`
  (`crates/arm-rules/src/types.rs`); resolution in
  `validation/selections.rs::param_value_resolves`; the load-time shape check in
  `ruleset/integrity.rs::validate_parameter_defs`.
- Data: `rules/core/virtues_flaws.json` — `virtue.folk_magic` (`category`),
  `virtue.inoffensive_to_beings`, `flaw.offensive_to_beings`,
  `flaw.unbearable_to_beings` (`being`). Value labels in
  `rules/i18n/en|de/virtues_flaws.json`; the picker's control name is the Fluent
  `param-label-category` / `param-label-being`.
- Tests: `shipped_enumerated_params_declare_exactly_their_book_values`,
  `a_value_outside_an_enumerated_list_does_not_resolve`,
  `every_declared_enumerated_value_resolves`,
  `folk_magic_repeats_across_categories_but_never_within_one`,
  `folk_magic_repeats_along_either_axis_and_never_across_the_excluded_realms`,
  `every_enumerated_value_id_has_english_and_german_text`,
  `fish_out_of_water_keeps_a_free_text_terrain`
  (`crates/arm-rules/tests/data_integrity.rs`, table `ENUMERATED_PARAM_ITEMS`).

Four items take a target the rulebook prints **in full**. Free text could not
express that: it accepts any string, so "dragons" was as legal as "demons" and
the book's own list lived nowhere the engine could see it. The domain is
therefore the list itself — declared on the parameter, in the rules JSON:

```json
"parameters": [{ "key": "category", "type": "ref", "domain": "enumerated",
                 "values": ["folk_magic.abjuration", "folk_magic.divination",
                            "folk_magic.evil_eye", "folk_magic.healing"] }]
```

A value outside `values` raises the **existing** `unknown_param_value` — it
simply does not resolve in its domain, which is what that code already means, so
there is no new issue code and no new Fluent key. Because the check lives in
`validate_selection_parameters`, it covers House/Mythic open-grant picks and
Warping fills for free, exactly as it covers a bought row.

The value ids are slug-style and never translated (`being.mundane_humans`,
`folk_magic.evil_eye`); their user-facing labels are ordinary
`rules/i18n/<lang>/` entries, so the picker and the Markdown export both render
a localized name and never the slug.

**The three being lists are different subsets of one another** — five classes,
six, and three — which is why the enumeration is declared per *parameter* rather
than once globally under the shared `being` key.

**Why Folk Magic has no ceiling at all — and why, since D10, that has to be
SAID rather than left absent.** `ArMDE:3919` grants the repeat "to acquire
expertise in a *different* category", so the item carries **no
`max_per_target`** (the default of 1 states nothing more than "not the
identical tuple twice", which is exactly right — see below). It used to follow
that the ceiling was the length of the category list — a further copy had to
repeat a category, which `validate_duplicate_selections` rejects. The **realm
axis** (see the Row 12 section below) retired that reading: the same sentence
lets each copy "align it to the same Realm as before or pick a different one",
so two copies may share a category as long as their Realms differ, and the
duplicate key `(item_ref, params)` covers both axes at once. No number is
written anywhere still, and `max_per_target: 4` remains rejected for the
original reason — it states a number the data already carries, and it would
still permit two copies naming the same target.

**D10 changes what "no ceiling" costs to state, not what it means.** Before
D10, an absent `max_total` and an absent `max_per_value` on both parameters
already meant unlimited, so nothing needed writing. D10 inverts both defaults
to 1 (once), so all three now have to be **explicit**: `max_total: 255` at
the item level (any number of copies, across every category/realm
combination), and `max_per_value: 255` on BOTH `category` and `realm` (either
axis may repeat identically, so long as the OTHER axis differs — "two copies
may legitimately share a category" as long as their Realms differ, and vice
versa). Omitting any of the three would silently reintroduce a cap
`ArMDE:3919` does not state.

**Load-time integrity** (`validate_parameter_defs`, applied to point items *and*
spells, since both hold `ParameterDef`s and both resolve through the same
`param_value_resolves`): an `enumerated` parameter must declare a non-empty,
duplicate-free `values` list, and every *other* domain must declare none. The
second half is the one that is easy to miss — a `values` list on a `text`
parameter is read by nothing, so it would look like an enforced restriction in
the data and silently not be one. Both failures name the offending item id and
parameter key.

**Why `flaw.fish_out_of_water_terrain` is NOT one of these.** Its terrain list
ends "…, etc." (`ArMDE:6130`), so the rulebook means the set to be open. Free text is
the correct encoding there, and `fish_out_of_water_keeps_a_free_text_terrain`
pins it against a future sweep that "finishes the job".

**Save impact, and the migration that now absorbs it.** Tightening `text` →
`enumerated` invalidated the free-text values older saves hold: a v0.2.x save
carrying `being: "Demons"` raised `unknown_param_value`, and one carrying
`virtue.folk_magic` raises `missing_param` because the parameter is new. That was
accepted when the domains landed and is no longer, because 0.3 ships to players
already running v0.2.0 — "open your character, get four errors" is not a release.

- **The `being` labels are migrated.** `migration.rs::fold_legacy_being_params`
  (called from `migration.rs::load_entity_migrating`) maps the fifteen labels a
  v0.2.x player could have typed — six classes × the two shipped languages,
  taken from `rules/i18n/en|de/virtues_flaws.json`, plus the three
  German dative forms `ArMDE:4135` prints (see below); all cross-checked against
  `ArMDE:4135` / `ArMDE:6526` / `ArMDE:6893` and their German mirrors — onto the
  `being.*` ids, case- and whitespace-insensitively. The earlier objection that
  "an English word list misses every German-typed save" is answered by carrying
  both locales; the table is frozen in Rust rather than read from the i18n files
  because the migration point holds no ruleset, and because editing a label later
  must not change what an old save migrates *to*. The strings are **matched, never
  rendered**, so no user-facing string is hardcoded.
- **`SCHEMA_VERSION` neither moves nor could.** These were *ruleset* changes, so
  nothing in a save distinguishes the two eras. The fold is therefore value-driven
  and idempotent, and it deliberately does not stamp the version: a value already
  equal to an id is not a key in the table, so a second load is a no-op, and
  `a_migrated_save_is_byte_stable_across_a_save_load_save_cycle` pins that a
  migrated save does not churn on every open.
- **Nothing unrecoverable is faked.** Folk Magic's `category` and `realm` and the
  three per-power Flaws' `power` were never *stored*, so there is nothing to
  migrate from; each stays exactly one `missing_param` naming its item and key,
  which the player clears with one pick. `virtue.alluring_to_beings` and
  `flaw.magical_being_companion` carry a `being` parameter that is still `text`,
  and are excluded from the fold for that reason.
- **One class this does not and must not fix.** A save whose bought Puissant Arts
  plus a House grant exceed `max_total` reports `too_many_selections`. That is a
  genuine rules violation the engine was previously blind to, not a format
  problem, and it survives migration —
  `a_genuine_too_many_selections_survives_the_being_migration` pins it. So
  "v0.2.x saves open clean" is true of the format changes and false of that one.
- Tests: `a_v0_2_x_save_migrates_its_typed_being_values_in_both_languages`,
  `an_already_migrated_being_value_is_left_alone`,
  `a_being_param_that_is_still_free_text_is_never_folded`,
  `an_unrecognised_being_value_is_left_exactly_as_typed`,
  `a_choice_the_old_save_never_stored_is_not_invented`,
  `migrating_a_v0_2_x_save_twice_changes_nothing`,
  `a_migrated_save_is_byte_stable_across_a_save_load_save_cycle`
  (`crates/arm-rules/src/migration.rs`);
  `a_v0_2_x_saves_typed_being_values_resolve_after_migration`,
  `the_choices_a_v0_2_x_save_never_stored_stay_one_actionable_issue_each`,
  `a_genuine_too_many_selections_survives_the_being_migration`
  (`crates/arm-rules/tests/data_integrity.rs`); and the unsaved-changes guard's
  half, `opens a migrated legacy save clean…` (`ui/src/App.client.test.ts`) —
  migration happens in Rust before the entity crosses IPC, so `open()`'s baseline
  snapshot is already post-migration and a migrated save opens **not** dirty.

**Both the nominative and the dative fold, and why the table has fifteen keys and
not twelve.** The uninflected-standalone-label convention governs what the app
*renders*; this table governs what a player *typed*. German `ArMDE:4135` — the
Inoffensive entry — prints its classes in the **dative** ("…verbunden: Tieren,
göttlichen Wesen, Feen, Dämonen oder magischen Kreaturen"), so a player filling
the old free-text box while reading that page copied an inflected form straight
off it. Those three forms are therefore keys too: `Tieren`, `göttlichen Wesen`,
`magischen Kreaturen`. They are transcribed, not declined — `Feen` and `Dämonen`
are identical in the dative and are already present, and no `sterblichen Menschen`
key exists because the source never prints one (`ArMDE:6526` and `ArMDE:6893`, the two
entries where that class is legal, both print the nominative). The English lists
(`ArMDE:4135`, `ArMDE:6526`, `ArMDE:6893`) fold onto the six English i18n labels exactly, so they
add no key.

`every_frozen_being_label_is_distinct_and_is_never_an_id`
(`crates/arm-rules/src/migration.rs`) asserts the two properties the whole fold
rests on over whatever the table holds: no two keys collapse onto one another
under the case/whitespace folding, and no key equals a `being.*` id. So growing
the table cannot silently introduce an ambiguity or break idempotency.

**One German wording deliberately left out.** `ArMDE:4141` renders the mundane-humans
class as *"gewöhnliche Menschen"* rather than the *"sterbliche Menschen"* of
`ArMDE:6526`/`ArMDE:6893` — a different lexical choice, not an inflection. It is not a key,
because it appears only inside the Inoffensive entry, in the sentence stating that
that class is **not available** there, and `being.mundane_humans` is not among
`virtue.inoffensive_to_beings`'s five declared values. Folding it could therefore
only turn one visible issue into another, and the two items where the class *is*
legal never print it.

**What this does NOT model — the residual gap, stated plainly.** Folk Magic has
a *second* choice axis: each copy also aligns to a `(Realm) Lore`, freely
re-chosen per copy, "although a character cannot have access to both the Divine
and Infernal Realms" (`ArMDE:3919`, the tail of the same sentence that grants the
repeat). The `category` parameter is the right and sufficient fix for the copy
cap; the realm axis and its Divine/Infernal exclusion stay unmodelled.

**Adjacent, adjudicated, and deliberately not fixed here.** The book gives
Inoffensive "General **and** Hermetic" (`ArMDE:4134`), Offensive "Hermetic **and**
General" (`ArMDE:6525`) and Unbearable "Hermetic **or** General" (`ArMDE:6892`), yet all
three ship with a single category. That is not an oversight of the dual-category
sweep so much as a class the sweep never covered: `5729e6d` states "The rulebook
gives four core Virtues and Flaws two categories each" and names Sufi, Visions,
Raised from the Dead and Suppressed Gift — all four written with a **comma**
(`*Minor, Social Status, Supernatural*`). Every descriptor joined by *and*/*or*
was left alone, and there are five of them: `ArMDE:4134`, `ArMDE:5882` (Curse of Slander,
"General or Supernatural"), `ArMDE:6525`, `ArMDE:6635` (Primogeniture Lineage, "Story and
Hermetic") and `ArMDE:6892`. Fixing three of the five here would leave the same claim
false and would not settle what *or* even means (Curse of Slander's "General or
Supernatural" reads as an either/or origin, not membership in both). So the
categories are untouched and the whole family is recorded as one to-do.

#### Realm parameter domain, and mutually exclusive values (open-to-dos row 12)
> "The character is capable of performing very minor acts of magic through his
> knowledge of scraps of occult lore. Choose one (Realm) Lore that is the key
> Ability for this magic, he may learn this Ability at Character Creation even
> if he is normally unable to take Arcane Abilities. The choice of (Realm) Lore
> also determines which supernatural realm his magic is aligned to for the
> purposes of aura modifiers." — Folk Magic, `ArMDE:3909`.

> "You may pick this Virtue more than once, to acquire expertise in a different
> category of spells. Each time you choose this Virtue, you can align it to the
> same Realm as before or pick a different one, although a character cannot have
> access to both the Divine and Infernal Realms." — `ArMDE:3919`.

- Source: `ArMDE:3909`, `ArMDE:3919`.
- Engine: `ParameterDomain::Realm` and `Realm::ALL` / `Realm::id` /
  `Realm::from_id` (`crates/arm-rules/src/types.rs`); resolution in
  `validation/selections.rs::param_value_resolves`; `ParameterDef::at_most_one_of`
  (`types.rs`) enforced by `validation/selections.rs::validate_exclusive_param_values`
  (issue code `exclusive_param_values`); load-time shape in
  `ruleset/integrity.rs::Ruleset::validate_at_most_one_of`; the export label in
  `export/resolve.rs::Doc::taxonomy_label`.
- Data: `rules/core/virtues_flaws.json` — `virtue.folk_magic`'s second
  parameter, `{ "key": "realm", "domain": "realm", "at_most_one_of":
  [["realm.divine", "realm.infernal"]] }`. Labels are the Fluent `realm-<id>`
  family (`locales/en|de/main.ftl`), which already shipped for Might; the
  control's name is the existing `param-label-realm`. The item's display name
  is a two-token template naming both axes — `rules/i18n/en/virtues_flaws.json`
  `"Folk Magic {category}, {realm}"` and `rules/i18n/de/virtues_flaws.json`
  `"Volksmagie {category}, {realm}"`.
- Tests: `a_realm_id_round_trips_through_from_id` (`types.rs`),
  `realm_domain_param_resolves_only_against_the_four_realms`,
  `two_copies_may_not_name_two_values_the_data_keeps_apart`
  (`validation/mod.rs`), `values_on_a_realm_param_are_rejected`,
  `an_at_most_one_of_member_outside_its_domain_is_rejected`,
  `an_at_most_one_of_group_of_one_is_rejected` (`ruleset.rs`),
  `a_realm_param_value_is_localized_rather_than_printed_as_its_slug`
  (`export.rs`), `folk_magic_records_the_realm_its_magic_is_aligned_to`,
  `folk_magic_repeats_along_either_axis_and_never_across_the_excluded_realms`
  (`tests/data_integrity.rs`), plus the frontend's
  `ParameterPicker realm domain (row 12)` (`ui/src/lib/components/ParameterPicker.test.ts`),
  `paramValueUsage > counts only rows that agree on the OTHER parameters`,
  `selectionDisplayName > names the Realm a Folk Magic copy is aligned to
  (English)` / `(German)`, `selectionDisplayName > renders an unfilled Folk
  Magic row without nesting the parameter hints`
  (`ui/src/lib/derive.test.ts`) and `names the exclusive-values finding in both
  locales, without naming a Realm` (`ui/src/lib/i18n.test.ts`).

**The Realm is stored, not the (Realm) Lore Ability.** `ArMDE:3909` names a *(Realm)
Lore* and the Core Rules print no closed list of them, so an `enumerated` domain
had nothing to enumerate and free text would accept anything. The sentence's own
mechanical payload is the **Realm** — "determines which supernatural realm his
magic is aligned to for the purposes of aura modifiers" — and the four Realms
*are* closed, already modelled as `Realm`, and already labelled by the
`realm-<id>` Fluent family the Might score reads. So the parameter records the
Realm, and `realm.<slug>` resolves through `Realm::from_id` exactly as
`characteristic.<slug>` resolves through `Characteristic::from_id`. No catalogue
and no declared `values`: the enum *is* the registry, which is why a `values`
list on a realm parameter is rejected at load by the same branch that rejects
one on `text`.

**The Divine/Infernal exclusion is data.** `at_most_one_of` is a list of value
*groups*, of which at most one member may be named across all copies of the
item. Folk Magic declares one group, `{realm.divine, realm.infernal}`, and no
realm id appears anywhere in Rust — a supplement (or a second realm-axis Virtue)
declares its own groups and the engine needs no edit. Each group is a
`BTreeSet`, so the declaration is order-free and canonical output is stable, and
load-time integrity requires every member to resolve *in the parameter's own
domain* (through the very `param_value_resolves` validation uses, so the two
notions of "resolves" cannot drift) and every group to name at least two values,
since "at most one of {Divine}" excludes nothing.

**Why `exclusive_param_values` names no value.** The message reports the item,
the parameter and how many group members are in play. A value's *label* depends
on its domain — a Realm is a Fluent `realm-<id>`, an enumerated id is a
rules-i18n name — so naming the values would need a per-domain argument family,
and Fluent throws on a message variable a caller does not supply. Spelling the
realms into the string instead would freeze one item's group into text that
every other item's group would then read wrongly, which is exactly what putting
the constraint in data avoids.

**Out of scope, and a different shape: the per-Realm effect restrictions.**
`ArMDE:3915` ("*Healing:* … Infernal Lore cannot be used to produce this type of
effect") and `ArMDE:3917` ("*Evil Eye:* … Divine Lore cannot be used to produce this
type of effect") are **cross-parameter constraints within one copy** — this
copy's realm against this copy's spell category. `at_most_one_of` excludes
values *across* copies and cannot express them; stretching it to try would make
one copy of Folk Magic Healing aligned to the Infernal look identical to two
copies naming Divine and Infernal, which are different rules with different
remedies. They stay unimplemented and are recorded here rather than approximated.

**A second axis retires the "one copy per category" reading.** Before the realm
axis, a second copy of Folk Magic had to differ in its spell category or be a
duplicate. `ArMDE:3919` says otherwise — "align it to the same Realm as before or
pick a different one" — so two copies may share a category and differ only in
Realm. The engine already had this right: `validate_duplicate_selections` keys
on `(item_ref, params)`, the whole tuple. The **picker** did not, and greyed out
a category another copy held regardless of Realm, refusing a legal build; the
frontend's `paramValueUsage` now takes the editing row's other parameter values
and counts only rows that agree on them, which is the same duplicate key. For a
single-parameter item there is nothing to disagree about and nothing changes.

**The Realm is named on the row, not only in the picker.** A parameter reaches a
display name only where the item's own template *mentions* it, so a Realm
recorded against a `"Folk Magic {category}"` template was visible in the picker
and on the exported sheet and nowhere the player actually reads a chosen Virtue.
The template therefore names both axes in apposition — "Folk Magic Healing,
Divine" / "Volksmagie Heilung, Das Göttliche" — and the frontend's
`selectionDisplayName` (`ui/src/lib/derive.ts`) resolves the stored
`realm.<slug>` through the `realm-<id>` Fluent family, the counterpart of the
export's `Doc::taxonomy_label`, so no surface can print the slug.

Two constraints fixed the wording. The Realm token carries **no literal
parentheses**: an unfilled slot renders the hint "(Realm)", so a
`"… ({realm})"` template would render the nested "((Realm))" the three
per-power items needed `name_unfilled` to escape — and a freshly added row is
exactly the unfilled case. And no noun follows the Realm label, because German
takes it uninflected from the glossary
(`rules/source/de/translation-tables/sphären-mächte.md:18-21`: Magie, Fee, Das
Göttliche, Das Infernale); "… Göttliche Sphäre" would need the inflected
adjective the label does not carry. Since the sheet appends an unmentioned
parameter in parentheses, the exported string changed shape with the template,
from "Folk Magic Healing (Divine)" to "Folk Magic Healing, Divine".

**Save impact — accepted, not migrated.** `ParameterDef` and `ParameterDomain`
are *ruleset* shape, not save shape, so `SCHEMA_VERSION` neither moves nor could
(no save distinguishes the two eras) and there is no migration to write. An
existing save holding Folk Magic reports `missing_param` for the newly declared
`realm`, exactly as it already does for `category` — the standing policy, since
a placeholder would invent someone's rules choice.

#### The other four realm parameters (open-to-dos row 24)

B7 above gave the project `ParameterDomain::Realm` and wired Folk Magic to it.
Four shipped items still typed their realm as free text; each one's own entry
names the closed four-value list in so many words, so all four are now `realm`
too.

> "Choose the realm (Divine, Faerie, Infernal, or Magic) to which the character
> is bound when you take the Flaw." — Bound to (Realm), `ArMDE:5733`.

> "Due to some connection with a given supernatural realm, the absence of a
> given supernatural aura has a pronounced effect upon the character's ability
> to focus on certain tasks. … A character may take this Flaw once for any
> particular Ability." — Necessary (Realm) Aura for (Ability), `ArMDE:6482`.

> "Pick one of the four Realms of Power; whenever he enters an aura of strength
> 4 or more aligned to that realm …" — (Realm) Stigmatic, `ArMDE:6656`.

> "You have been trained in the mystical aspects of one of the four realms of
> power (Divine, Faerie, Infernal, or Magic) … You may take this Virtue multiple
> times, for a different realm each time." — Student of (Realm), `ArMDE:5054`.

- Source: `ArMDE:5733`, `ArMDE:6482`, `ArMDE:6656`, `ArMDE:5054`.
- Data: `rules/core/virtues_flaws.json` — `flaw.bound_to_realm`,
  `flaw.necessary_realm_aura_for_ability` (whose second, `ability`-domain
  parameter is untouched), `flaw.realm_stigmatic` and `virtue.student_of_realm`
  each carry `{ "key": "realm", "type": "ref", "domain": "realm" }`. No
  `values` list (the `Realm` enum IS the registry) and no `at_most_one_of` —
  see below.
- Engine: unchanged. `param_value_resolves`
  (`validation/selections.rs`) already resolves the domain through
  `Realm::from_id`, the picker already has a `realm` branch, and
  `export/resolve.rs::Doc::taxonomy_label` already labels it — so this is a
  data-only change, which is the point of keeping catalogue shape in JSON.
- Tests: `no_shipped_realm_parameter_is_free_text`,
  `a_free_text_realm_from_an_older_save_is_reported_in_the_players_own_words`
  (`crates/arm-rules/tests/data_integrity.rs`);
  `resolves a parameter domain arg through its param-domain Fluent key`
  `takes the German Realm label in apposition, never inflected`,
  `names an unfilled German row as the rulebook heads it`
  (`ui/src/lib/derive.test.ts`); `labels every parameter domain in both
  locales`, `shows the typed value back, and no domain slug`
  (`ui/src/lib/i18n.test.ts`).

**What an older save does — reported, never rewritten.** A save written while
these were free text holds a typed word ("Faerie", "the Divine") in the slot,
and `Realm::from_id` resolves none of it. **No migration touches it.** Saves
store choices, not resolved values; mapping a word onto a Realm would be
guessing someone's rules choice, and blanking it would destroy the only record
of what they meant. The engine reports instead, through the **existing**
`unknown_param_value` — the same code an unresolvable `item` or `ability` ref
has always raised, and the only parameter finding whose args carry the offending
**`value`**. That argument is the whole mechanism: the player is shown the words
they typed, beside the picker the domain change gives them, and one pick clears
it. A *blank* realm still reports `missing_param` as before
(`param_value_is_blank` runs first), which is the right split — nothing typed is
a choice not yet made, something typed is a choice that no longer resolves.
`SCHEMA_VERSION` does not move and could not: `ParameterDomain` is *ruleset*
shape, not save shape, so no save distinguishes the two eras. This is B7's
"Save impact — accepted, not migrated" paragraph applied to four more items.

**The finding's `domain` argument is now a word.** `unknown_param_value`
interpolates the `ParameterDomain` the value failed in, and the engine emits the
enum's serialized name. Nothing labelled it, so German read "hat unbekannten
realm-Wert" — an English slug inside a German sentence, and the violation
CLAUDE.md names explicitly. E2 added the `param-domain-<id>` Fluent family (one
key per variant, both locales) and routes the argument through it in
`resolveIssueArgValue`'s `ENUM_ARG_FLUENT_PREFIX` (`ui/src/lib/derive.ts`),
exactly as `category` is routed. Both messages were reworded to place the domain
in a trailing parenthesis rather than as the head of a compound
(`{ $domain }-Wert`), because the word arrives from data and cannot be compounded
or inflected reliably. `Record<ParameterDomain, true>` in the i18n test makes a
future unlabelled variant a type error.

**The German names had to be rewritten, and that is the change with the most
user-visible bite.** A typed word inflects however the player typed it; a picked
Realm arrives as the fixed `realm-<id>` label, and two of the German four are
noun phrases carrying their own article — *Das Göttliche*, *Das Infernale*
(`rules/source/de/translation-tables/sphären-mächte.md:18-21`). Every German
template put the token somewhere that demands agreement, so the moment the
domain changed they rendered ungrammatical German: *"Gebunden an Das
Göttliche"*, *"Student der Das Göttliche"*, *"Das Göttliche-Stigmatisierter"*,
*"Notwendige Das Göttliche-Aura für …"*. The fix is Folk Magic's own pattern
from B7 — the label stands in **apposition after a comma**, uninflected, and no
noun follows it:

| id | `rules/i18n/de/virtues_flaws.json` `name` | `name_unfilled` |
|---|---|---|
| `flaw.bound_to_realm` | `Gebunden an eine Sphäre, {realm}` | `Gebunden an (Sphäre)` |
| `flaw.necessary_realm_aura_for_ability` | `Notwendige Aura für {ability}, {realm}` | — (two params; see below) |
| `flaw.realm_stigmatic` | `Stigmatisierter, {realm}` | `(Sphäre)-Stigmatisierter` |
| `virtue.student_of_realm` | `Student einer Sphäre, {realm}` | `Student der (Sphäre)` |

`name_unfilled` carries the German rulebook's own heading (`Basisregeln.md:5731`,
`Basisregeln.md:6654`, `Basisregeln.md:5052` — the German file mirrors the
English line-for-line, so these are the same entries as the citations above) so a
freshly added row reads as the book prints it instead of
repeating "Sphäre" either side of the comma. It is **not** given to Necessary
(Realm) Aura: it fires only when *no* placeholder is filled, so on a row with an
Ability chosen and no Realm it would not fire anyway, and the plain template
already renders cleanly there. **Parentheses were considered and rejected** for
the same reason B7 rejected them — the unfilled hint is itself "(Sphäre)", so a
`"… ({realm})"` template renders the nested "((Sphäre))", and on a two-parameter
item `name_unfilled` cannot rescue the half-filled case.

The **English** names needed no change: the English realm labels are bare
adjectives (Magic, Faerie, Divine, Infernal), so "Bound to Divine", "Divine
Stigmatic", "Necessary Divine Aura for Awareness" and "Student of Divine" all
read as the book heads them. This is the asymmetry B7 recorded and it is why the
two locales' templates now differ in shape.

**No `at_most_one_of` on any of the four, and that is a finding, not an
omission.** Folk Magic carries one because `ArMDE:3919` states it *inside Folk
Magic's own entry* — "although a character cannot have access to both the Divine
and Infernal Realms" — as part of that Virtue's repeat rule. It is not a general
statement about characters, and none of these four repeats the restriction.
Inventing it for Student of (Realm) would forbid a build the book permits.

**Two multiplicity restrictions these entries state, and what became of them.**

- *Student of (Realm)* — "You may take this Virtue multiple times, for a
  different realm each time" (`ArMDE:5054`) is already expressed exactly by the
  defaults: `max_per_target` 1 over the duplicate key `(item_ref, params)`,
  whose only parameter is the realm, plus no `max_total`. Nothing to add. It is
  already listed under "Repeats with a different target each time" above.
- *Necessary (Realm) Aura for (Ability)* — "A character may take this Flaw once
  for any particular Ability" (`ArMDE:6482`) caps repeats on **one parameter
  key**, not on the whole tuple. `max_per_target` keys on all parameters at
  once, so two copies naming the same Ability in different Realms collide in no
  key and pass today. Expressing it needs a per-key cap the `ParameterDef` model
  does not have — engine work, not data — so it is **recorded, not invented**
  (`docs/open-todos.md`). The same entry's "You may not take Student of (Realm)
  and Puissant Ability for the same Lore" (`ArMDE:5054`) is a cross-*item*
  constraint over a parameter value and is likewise unmodelled.

#### D42 — every Supernatural entry's realm association (`docs/vf-audit/decisions.md` D42/D70/D74)

> "All Supernatural Virtues and Flaws are associated with one of the four
> realms … this should be the choice if the character concept does not
> suggest another option. Some Virtues are always associated with other
> realms, such as Faerie Blood and Strong Faerie Blood, which are always
> associated with Faerie. A Virtue's description notes if it is limited in
> this way." — `ArMDE:2960`.

> "Tainted Virtues and Flaws are associated with the Infernal realm …
> Supernatural abilities granted by Tainted Virtues or Flaws are always
> Infernal powers." — `ArMDE:3000`.

- Source: `ArMDE:2960`, `ArMDE:3000`; per-entry fixes at `ArMDE:3486` (Bee
  King), `ArMDE:3581` (Commanding Aura), `ArMDE:3507` (Blood of the
  Nephilim), `ArMDE:3661` (Demonic Blood's cross-entry bar, which is what
  fixes Demonic Might/Powers), `ArMDE:4177` (Kassalan Exorcism),
  `ArMDE:5026` (Strong Angelic Heritage), `ArMDE:6648` (Raised from the
  Dead), `ArMDE:6979` (Viaticarus), `ArMDE:6406` (Manifest Sin's
  Divine/Infernal subset), `ArMDE:3486` calibration note also covers
  `ArMDE:4361` (Magical Blood) and `ArMDE:6456` (Monstrous Blood — same
  origin-stated shape as Faerie Blood).
- Data model: `types.rs::RealmAssociation` — `Fixed`/`Default`/`Subset`/
  `FromParam`, an optional field on `types.rs::PointItem::realm_association`.
  `None` for the ~90 Supernatural entries the book leaves free. A `tainted:
  true` item (`ArMDE:3000`) resolves Infernal by that rule alone, ahead of
  this field, so the nine Tainted-tagged entries in the 17-fixed tally carry
  no `realm_association` of their own (double-storing the same fact could
  silently drift). `types.rs::Entity::concept_realm: Option<Realm>` is the
  concept-phase default source (D42: "a default source and nothing else" —
  never the character's own realm).
- Data: `rules/core/virtues_flaws.json` — Fixed: `virtue.faerie_blood`,
  `virtue.strong_faerie_blood`, `virtue.bee_king` (Faerie); `virtue.
  kassalan_exorcism`, `virtue.magical_blood`, `flaw.monstrous_blood` (Magic);
  `virtue.strong_angelic_heritage`, `virtue.blood_of_the_nephilim`, `flaw.
  viaticarus`, `virtue.commanding_aura`, `flaw.raised_from_the_dead`
  (Divine); `virtue.demonic_might`, `virtue.demonic_powers` (Infernal).
  Default: `virtue.hex`, `flaw.cursed_guile` (Infernal); `virtue.
  spiritual_pact`, `flaw.warped_by_magic` (Magic); `virtue.sufi` (Divine).
  Subset: `flaw.manifest_sin` (`{divine, infernal}`). FromParam (reusing the
  entry's own `realm` parameter, D74 Q3): `flaw.bound_to_realm`, `flaw.
  realm_stigmatic`, `flaw.necessary_realm_aura_for_ability`, `virtue.
  folk_magic`.
- Engine: the override is stored under a selection param key of its own,
  `types.rs::REALM_OVERRIDE_PARAM_KEY` (`"association"`) — not
  `"realm"`, which four of the entries above already use for a different (or,
  for Folk Magic, the same) value. It is admitted as a legal key but never a
  *required* one (`validation/selections.rs::validate_selection_parameters`),
  so an old save with none of this stays clean. `effective/realm.rs::
  resolve_realm` is the pure resolver (override → entry association →
  concept → `Realm::Magic`); `validation/realm.rs::validate_realm_
  associations` turns its warning into `realm_changed_default` /
  `realm_unset_subset`, and separately flags a garbage override
  (`unknown_param_value`, the same code any other unresolvable parameter
  value raises) or one naming a realm outside a `Subset` entry's list
  (`realm_override_invalid`).
- Export: `export/sections.rs::Doc::item_rows` appends the resolved realm to
  the existing text cell for a qualifying selection only
  (`effective/realm.rs::item_has_realm_association`) — no new column, so an
  entity holding no Supernatural entry renders byte-identical to before this
  existed.
- Tests: `crates/arm-rules/tests/d42_realms.rs` (resolution chain, each
  association class, the garbage-override safety net, old-save
  compatibility); `crates/arm-rules/tests/export_golden.rs` (`a_fixed_
  supernatural_entrys_realm_rides_the_text_cell`, `a_non_supernatural_
  entrys_text_cell_gets_no_realm_line`).
- A *granted* copy's realm (Strong Faerie Blood's "faerie eyes" Second Sight,
  Faerie Doctor's Dowsing, mythic-type grants) is a separate mechanism — see
  "D74.4/Row 55" below.

#### D74.4/Row 55 — a granted copy's realm override (`docs/open-todos.md` row 55; `docs/vf-audit/decisions.md` D74.4)

> "Second, you have faerie eyes. This gives you the Virtue Second Sight (see
> page 106) at no cost…" — `ArMDE:5038` (Strong Faerie Blood).

> "The following types of Mythic Companions cover all the supernatural
> realms." — `ArMDE:2641` (Devil Child=Infernal, Faerie Doctor=Faerie,
> Nephilim=Divine, Spirit Votary=Magic, by the book's own naming).

A granted copy of a Supernatural entry carried no realm of its own: D42's
resolver treats a granted `Selection` exactly like a bought one, so the
item's own `realm_association` (usually `None` — Second Sight and Dowsing are
both book-open entries) decided it, silently dropping the stated per-GRANT
association. D74.4: "granted copies carry a realm only where stated, and
mythic-type grants default to the type's realm" — per-grant, not per-item,
since the same item (Second Sight) resolves differently depending on which
Virtue granted it (Strong Faerie Blood → Faerie; bought outright → the plain
chain).

- Data model: an optional `realm: Option<Realm>` field on
  `types.rs::Effect::GrantsSelection` (the nested-grant effect a Virtue/Flaw
  carries) and `grant.rs::Grant::Fixed` (the direct grant a House or Mythic
  Companion type's own `grants` list carries) — two separate fields because
  `vf_granted_selections` only scans **bought** selections, so a mythic-type's
  marker Virtue (`virtue.faerie_doctor`/`virtue.spirit_votary`) is never
  itself bought when granted via the type, and its own nested
  `GrantsSelection` effect never fires for that path; the type's `Grant::Fixed`
  entry grants the Supernatural item directly instead. Both paths need the
  value when the marker Virtue is bought standalone too (`ArMDE:2704`: "A
  character can be a faerie doctor without being a Mythic Companion").
- Data: `rules/core/virtues_flaws.json` — `virtue.strong_faerie_blood`'s
  `grants_selection` of `virtue.second_sight` carries `realm: "faerie"`;
  `virtue.faerie_doctor`'s `grants_selection` of `virtue.dowsing` carries
  `realm: "faerie"`; `virtue.spirit_votary`'s `grants_selection` of
  `virtue.second_sight` carries `realm: "magic"`. `rules/core/
  mythic_companion_types.json` — `mythic_type.faerie_doctor`'s direct
  `Grant::Fixed` of `virtue.dowsing` carries `realm: "faerie"`;
  `mythic_type.spirit_votary`'s of `virtue.second_sight` carries
  `realm: "magic"`. No other grant (House or mythic-type) needed the field —
  every other Supernatural grant target is already `Fixed`/`Default` at the
  item level (Demonic Might/Powers, Strong Angelic Heritage), or is not
  Supernatural at all (Templar Commander's Brother-Knight/Temporal
  Influence).
- Engine: `types.rs::stamp_realm_override` writes the stated realm
  onto the granted `Selection`'s own `REALM_OVERRIDE_PARAM_KEY` param
  (`"association"` — the exact key a player's own override uses), called from
  `grant.rs::resolve_grant`'s `Grant::Fixed` arm and
  `effective.rs::vf_granted_selections`'s `Effect::GrantsSelection` arm. This
  is the whole mechanism: `effective/realm.rs::resolve_realm` itself needs
  **no change**, since it already reads that param off any `Selection`,
  bought or granted — reusing the existing override chain rather than adding
  a parallel one. A grant that states nothing (`realm: None`) is a no-op, so
  the granted copy resolves through the plain chain exactly like a bought
  one (Templar Commander, unaffected).
- Load-time integrity: `realm: Option<Realm>` is a typed enum field (mirroring
  `Effect::MightGrant::realm`), so serde itself rejects an invalid realm id at
  `Ruleset::from_sources` — no separate `ruleset/integrity.rs` check needed.
- DTO: `arm_app::effective_dto::EffectiveScores::granted_realm_associations`
  (`Vec<ResolvedRealmEntry>`, mirrored in `ui/src/lib/types.ts`) — the
  granted-row counterpart to `realm_associations`, `index`-keyed into
  `granted_selections` rather than `Entity::selections`. `fixed` is always
  `true`: a granted row has no stable `entity.selections` index for the
  player to attach an override control to, unlike a bought row's. Consumed by
  `export/sections.rs::Doc::item_rows` for free (it already calls
  `resolve_realm` on the granted table exactly like the bought one); the live
  Svelte V/F editor does not yet render a granted row's realm at all
  (`docs/open-todos.md` row 55's own note) — left for a follow-up slice.
- Tests: `crates/arm-rules/tests/row55_grant_realm.rs` (the stamping chain,
  per source, the mythic-type cases, the unaffected/bought-selection guards,
  load-time integrity); `crates/arm-rules/tests/export_golden.rs`
  (`a_granted_supernatural_entrys_realm_rides_the_granted_table_text_cell`);
  `crates/arm-app/src/effective_dto.rs` tests
  (`granted_realm_associations_surfaces_a_stated_per_grant_realm`,
  `granted_realm_associations_is_empty_for_a_bought_only_entity`).
- **Not modelled**: the live Svelte V/F editor does not show a granted row's
  realm (bought rows do, via the existing `realm-fixed`/`<select>` control).
  Recorded as an open question, not invented.

#### Selection multiplicity — one copy per named power
> "This Flaw may be taken once for each power the character possesses."

- Source: `ArMDE:6689` (Restricted
  Power), `ArMDE:6761` (Slow Power, "may be taken more than once, if the character
  has multiple powers, but not more than once for a single power"), `ArMDE:5205`
  (Variable Power, "may be taken more than once, if the character has more than
  one power, but it only applies once to a single power").
- Data: `rules/core/virtues_flaws.json` — `flaw.restricted_power`,
  `flaw.slow_power`, `virtue.variable_power`.
- Tests: `shipped_per_power_items_carry_a_power_target`,
  `per_power_items_repeated_on_one_power_are_duplicates`,
  `per_power_items_repeated_across_powers_are_clean`
  (`crates/arm-rules/tests/data_integrity.rs`, table `PER_POWER_ITEMS`).

All three say the same thing: repeat freely across *different* powers, never
twice on the *same* one. That is neither `max_per_target: 255` (which permits
stacking every copy on one power) nor `max_total: 1` (which forbids the repeat
the book grants), so it needs a recorded per-copy **target**. Each item
therefore declares a single free-text parameter

```json
"parameters": [{ "key": "power", "type": "ref", "domain": "text" }]
```

and leaves `max_per_target` at its default of 1 and `max_total` absent
(`u8::MAX`). The power name becomes part of the `(item_ref, params)` duplicate
key (`validation/selections.rs`), so two copies naming the same power collide
as `duplicate_selection` while copies naming different powers are distinct keys
under no ceiling. **No engine change was needed** — the `text` domain and the
existing duplicate key already express the rule.

**Why the target is free text and not `domain: "item"`.** A "power" is an
*instance* of one of the Power Virtues Restricted Power enumerates — "those
granted by the Focus Power, Greater Power, Lesser Power, Personal Power, and
Ritual Power Virtues" (`ArMDE:6689`; Variable Power says "(Greater, Lesser,
Personal, or Ritual)", `ArMDE:5201`). Those Virtues are themselves unparameterized
and repeatable (`virtue.greater_power`, `max_per_target: 255`), so a magus with
three Greater Powers holds three *indistinguishable* rows. An `item` domain
would name the power Virtue, wrongly capping him at one Restricted Power across
all three.

**What this does NOT enforce — the residual gap, stated plainly.**

1. **The typed name used to be unverified. It no longer is.** Nothing checked
   that the string named a power the character possesses, and the recorded fix
   here was to give the five Power Virtues a free-text `name` parameter of their
   own. That fix is **wrong and was not taken**: a copy of a Power Virtue is not
   a power (`ArMDE:4021`, `ArMDE:4283`, `ArMDE:4874` all add copies' levels together to make
   *several* powers or one stronger one), so a per-copy name would encode a 1:1
   correspondence the book denies. What the registry always was is
   `Entity::powers` — see *Power Virtues fund the power-levels budget (B10)*
   below, which wires the Virtues to that budget and validates the `power`
   parameter against `entity.powers[].name` (`require_power` →
   `power_dangling_target`).
2. **The duplicate key is byte-for-byte, but the values are trimmed.** Identity
   is exact `(item_ref, params)` equality, and a `Text` value that trims to
   nothing now raises `missing_param` rather than being accepted — see
   *Parameter-value identity: trimmed, never case-folded (row 10)*. So
   "Wolf Shape " and "Wolf Shape" are the same target, while "wolf shape" is a
   different one: case is deliberately not folded, and `validate_power_targets`
   makes the same choice when matching a `power` parameter against the power
   rows, so the engine has exactly one notion of free-text identity.

So the cap stops an honest mistake — the player who takes Slow Power twice for
one power without noticing — not a determined evasion. That is a real
improvement over the previous state (where the same-power repeat was not
detectable at all), and it is the whole of what is claimed.

**Save impact.** An existing save holding a paramless `flaw.slow_power`,
`flaw.restricted_power` or `virtue.variable_power` row now raises
`missing_param` on open. No data is lost — saves store choices and the engine
only reports — and the player clears it by naming the power.

Repeat rules the data model cannot express (deliberately left unenforced rather
than approximated):

- **"Once for each Supernatural Virtue the character possesses" used to sit in
  this list.** False Power's copies (`ArMDE:6096`) now each name the Virtue they
  taint, through an `item` parameter carrying `require_categories`,
  `require_possessed` and `forbid_tainted` — see *False Power names the Virtue
  it taints (B9)* above. Nothing about the repeat rule is deferred any more; the
  one clause left unimplemented there is the Divine one, which the book itself
  leaves to the troupe.
- **Proportional per-item caps used to sit in this list.** Demonic Might /
  Demonic Powers "can account for no more than half of the character's total
  Virtues" (`ArMDE:3665`, `ArMDE:3669`) is now expressed — the ratio is data
  (`max_share_of_kind`) and `validate_share_of_kind_cap` warns on it; see
  *Demonic Might / Demonic Powers — the half-of-Virtues ratio* above. Nothing
  about it is deferred any more. What is a judgement rather than a gap — reading
  "total Virtues" as Virtue *points* rather than a headcount, which is why the
  check warns instead of blocking — is recorded there.
- **"A different X each time" where X is free text.** Greater Immunity's
  immunity, Social Contacts' social group and Vulnerable Magic's condition are
  not recorded, so distinctness is not enforced. These items record no target at
  all today; adding a `Text` parameter would enforce it but would invalidate
  existing saves whose selections carry no such parameter. The three **per-power
  caps** used to sit in this bullet; they are now expressed — see *Selection
  multiplicity — one copy per named power* above — and adding their `power`
  parameter did carry exactly the save cost described here, which is the price
  of closing the gap rather than an argument against it.
- **Folk Magic's realm alignment.** The category axis is now expressed — see
  *Enumerated parameter domain — a closed list the rulebook prints* above, which
  is also where the three (Beings) items' closed lists live. What is still not
  modelled is Folk Magic's *second* axis: each copy also aligns to a `(Realm)
  Lore`, re-choosable per copy, except that "a character cannot have access to
  both the Divine and Infernal Realms" (`ArMDE:3919`). Expressing it needs a second
  parameter plus a cross-copy exclusion rule, and is not done.

Repeated copies stack through the normal effect sum — `for_each_effect!` walks
every selection, so two Improved Characteristics yield
`characteristic_points_granted == 6` and two Demonic Powers yield
`power_levels_budget == 40`. Covered by
`tests/data_integrity.rs::repeated_selections_stack_their_effects`,
`::repeated_selections_are_not_reported_as_duplicates`,
`::shipped_repeatable_items_carry_their_rulebook_ceiling` and
`::shipped_once_only_items_stay_non_repeatable`.

#### Power Virtues fund the power-levels budget (B10)
> "The character has a supernatural power that he can activate at will. If you
> take the Virtue once, this is a single power, equivalent to a Formulaic
> Hermetic spell with a level of 50 or lower. You may also spend levels
> one-for-one to give the power Penetration; otherwise, it has a Penetration of
> zero."

- Source: `ArMDE:4019` (Greater Power,
  **50** levels + the Penetration clause), `ArMDE:4281` (Lesser Power, "total levels
  of 25 or lower", same Penetration clause), `ArMDE:4716` (Personal Power, **25**),
  `ArMDE:4872` (Ritual Power, **25**, "a Ritual Hermetic spell with a level of 25 or
  lower").
- Data: `rules/core/virtues_flaws.json` — `virtue.greater_power`,
  `virtue.lesser_power`, `virtue.personal_power`, `virtue.ritual_power` each gain
  `effects: [{ "type": "power_levels", "amount": … }]`, and with it the
  `creation_effect` classification every effect-bearing entry must carry.
- Implementation: `effective::power_levels_budget` (unchanged — it already sums
  `Effect::PowerLevels`), `effective::powers_used` (now sums Penetration too),
  `validation/might.rs::validate_powers` (`over_power_levels`).
- Tests: `shipped_power_virtues_fund_the_power_levels_budget`,
  `focus_power_funds_no_power_levels_because_its_pool_is_points`
  (`crates/arm-rules/tests/data_integrity.rs`, table `POWER_LEVEL_ITEMS`),
  `powers_used_spends_penetration_from_the_same_pool_as_level`
  (`effective.rs`).

`Entity::powers` was already the named-power registry and already budgeted; what
was missing was the connection. Before this, a character could buy Greater Power
and have a budget of 0, so every power he entered read as `over_power_levels`.
The budget's only funders were the three Might Virtues (Demonic Blood 30,
Demonic Powers +20, Strong Angelic Heritage 30), which is a *different* source
and stacks rather than double-counts: a Might Score is not itself a grant —
`power_levels_budget` starts at 0 and sums `Effect::PowerLevels` alone — so a
demon-blooded companion who also buys Lesser Power legitimately has 30 + 25.

**Focus Power is deliberately excluded from *this* budget** — and E1 built the
one it does belong to (see *Focus Power's own point pool* below). "This Virtue
grants a pool of 25 points… It costs 2 points to raise the maximum level of
effect by 1, and 1 point to raise the Penetration by 1" (`ArMDE:3899`). Those 25
are a different currency: they buy at most 12 levels, or 25 Penetration, or a
mix. Adding them to a level-denominated budget would let a Focus Power pay for 25
levels the Virtue cannot buy — wrong rules output, so `virtue.focus_power`
carries no `power_levels` effect and a control test
(`focus_power_funds_no_power_levels_because_its_pool_is_points`) still pins that
it never gains one. What it gains instead is `Effect::FocusPoints`, a second
currency with its own budget, its own spend rate and its own over-spend code.

**Penetration is charged against the same budget** — `SupernaturalPower` gains
`penetration: u16`. `ArMDE:4021` makes the arithmetic explicit: two copies of Greater
Power give 100 levels, spent as "a power with a level of 60 and a Penetration of
0, and a second power with a level and Penetration of 20 each" — 60 + 0 + 20 +
20 = 100. `powers_used` summing levels alone reported 80 and let the player spend
20 levels the book had already spent. The field is additive
(`#[serde(default, skip_serializing_if = "is_zero_u16")]`), so `SCHEMA_VERSION`
stays **16** and there is no migration: an older save reads 0, and a power with
no Penetration serializes exactly as before.

**Recorded, not enforced: the one-power-per-Virtue default.** "By default, there
should be one power per Virtue, as this Virtue is intended for powers that are
individually significant. … However, the troupe may allow the character to take
more powers if they have a strong thematic link" (`ArMDE:4021`). That is explicit
troupe discretion, so the *number* of `powers` rows is unconstrained; only their
total level (plus Penetration) is. Note also that `ArMDE:4724` gives Personal Power
the same "may be taken more than once, and the levels added together" clause as
Greater and Lesser Power, despite `ArMDE:4716` calling it "a single power" — so no
Power Virtue is one-copy-one-power, and none of them is modelled as such.

**The `power` parameter is now validated against the registry.** Restricted
Power, Slow Power and Variable Power keep their free-text `power` parameter and
add `require_power: true`; `validation/selections.rs::validate_power_targets`
raises `power_dangling_target` when the typed name matches no
`entity.powers[].name`. It is filed on `CreationPhase::Review` — the phase that
owns "Might and powers", and therefore the step that can offer the fix —
following `validate_ability_bonus_targets`'s rule rather than
`validate_possessed_param_targets`'s, because powers are entered *after* the V/F
step and filing there would deadlock the wizard. Matching is exact on trimmed
strings, case not folded, per *Parameter-value identity: trimmed, never
case-folded (row 10)*. The picker deliberately keeps a text input rather than a
select over the held powers, for the same reason: at V/F time there may be no
powers yet, and a select would be a dead end. Load-time integrity rejects
`require_power` on any domain but `text`.

**Save impact — accepted, not migrated.** Two visible changes on opening an old
save. A character holding a Power Virtue now has a *budget* where he had none,
so an existing `over_power_levels` error may simply clear. A character holding
Restricted/Slow/Variable Power whose `power` names something not in his `powers`
list now reports `power_dangling_target`; no migration can answer it, because
the engine cannot know whether the player meant to add the power or mistyped the
name. No data is lost either way — saves store choices and the engine only
reports.

**Markdown export.** The being's own powers table gained a Penetration column
(`export/magic.rs::power_rows`, key `power-penetration-label`), so the sheet
reconciles with the budget bar the app showed. The familiar's invested-powers
table keeps the two-column `leveled_rows`: those powers are charged against no
budget at all (`ArMDE:10866`) and no surface sets their Penetration, so a third column
there would be a row of zeros implying a field that does not exist.

#### Focus Power's own point pool (E1)
> "This Virtue grants a pool of 25 points. The maximum level of effect and
> Penetration both start at zero. It costs 2 points to raise the maximum level of
> effect by 1, and 1 point to raise the Penetration by 1. Thus, 25 points can
> allow a maximum level of 10 with a Penetration of 5, or a maximum level of 5
> with a Penetration of 15, or combinations in between. The power has an
> Initiative score equal to the character's Quickness – the maximum magnitude of
> the effect. The character may create any effect within the scope of the power,
> up to the level of the effect."

- Source: `ArMDE:3895-3897` (Focus Power, *Major, Supernatural*), `ArMDE:3899`
  (the 25-point pool, the 2:1 / 1:1 rates, the two worked splits, Initiative),
  `ArMDE:3901` (the Fatigue bands), `ArMDE:3903` ("This Virtue may be taken more
  than once, and the points gained may be combined"). The magnitude the
  Initiative formula names is the book's own general rule: "Spells also have a
  magnitude, which is equal to the level divided by five, rounded up"
  (`ArMDE:9097`).
- Data: `rules/core/virtues_flaws.json` — `virtue.focus_power` gains
  `effects: [{ "type": "focus_points", "amount": 25 }]` and, with it, the
  `creation_effect` classification every effect-bearing entry must carry. **The
  25 lives only here**, never in Rust.
- Implementation: `types.rs::Effect::FocusPoints` (the grant),
  `effective/gift_confidence.rs::focus_points_budget` (base 0, summed — copies
  combine), `effective/gift_confidence.rs::focus_points_used`
  (`2 × max_level + penetration`), `validation/might.rs::validate_focus_powers`
  (`over_focus_points`, Fluent `issue-over_focus_points`),
  `derived/focus_power.rs::focus_power_lines` (magnitude, Initiative, Fatigue),
  `export/magic.rs::focus_power_rows` (its own table).
- Tests: `focus_power_grants_a_twenty_five_point_pool_that_copies_combine` and
  `a_focus_power_spends_two_points_per_level_and_one_per_penetration`
  (`effective.rs`), `focus_powers_are_checked_against_their_own_pool`
  (`validation/mod.rs`), `focus_power_lines_derive_magnitude_initiative_and_fatigue`
  (`derived.rs`), `focus_powers_print_their_own_table_with_initiative_and_fatigue`
  (`export.rs`),
  `shipped_focus_power_funds_a_twenty_five_point_pool_that_copies_combine`
  (`tests/data_integrity.rs`).

**Why a separate `Entity.focus_powers` list rather than a flag on
`SupernaturalPower`.** The two hold different quantities. A `SupernaturalPower`'s
`level` is the level of a power that was *made*, spent one-for-one out of the
level budget; a Focus Power's is a **ceiling** — "the maximum level of effect",
raised at 2 points a time, with "the character may create any effect within the
scope of the power, up to the level of the effect" (`ArMDE:3899`). The field is
therefore named `max_level`, and keeping the lists apart makes it *structurally*
impossible for a focus power to reach `powers_used` — the stronger form of the
same guarantee B10 got by leaving Focus Power out of `Effect::PowerLevels`.
A character with no Focus Power is provably unaffected: `powers_used` and
`power_levels_budget` are untouched, and `focus_points_used` of an empty list is
0 against a budget of 0.

**Initiative and the Fatigue bands are display-only** (`derived/focus_power.rs`,
surfaced as `DerivedTotals.focus_powers` and in the export table). Initiative is
`Quickness − magnitude`, against the same aging-adjusted Quickness every other
play stat uses. The Fatigue cost is 1 / 2 / 3 levels for ≤25 / 26–50 / 51–75
(`ArMDE:3901`); **above 75 the rulebook says nothing**, so the read-out reports
"unstated" through `focus-power-fatigue-unstated` rather than continuing the
pattern into a fourth band nothing sourced.

**Recorded, not implemented: the cross-book guidance of `ArMDE:3905`.** "This
Virtue may be associated with any supernatural realm… you may want to base the
power on a different system of supernatural powers… remember that the levels of
effects may work on different scales for different systems, and you may want to
change the cost of a level of effect." That is explicit troupe/other-book
guidance — a *variable* points-per-level rate keyed to a power system this app
does not model — so the engine implements the core rate (2 points per level) and
nothing else. The realm association itself is likewise unmodelled: a focus power
carries no Realm field.

**Schema.** `Entity.focus_powers` is additive
(`#[serde(default, skip_serializing_if = "Vec::is_empty")]`), as is
`FocusPower::penetration` (`is_zero_u16`), so `SCHEMA_VERSION` stays **17** and
there is no migration: an older save reads an empty list and writes identical
bytes. `Entity::normalize` sorts the list, like every other.

#### Affinity with (Ability) — creation XP counts for half again
> "All Advancement Totals for one Ability are increased by half, rounded up, as
> are any experience points you put in that Ability at character creation. … If
> you take this Virtue for an Ability, you may exceed the normal age-based cap
> during character generation … by two points for that Ability."

The "counts as 1½×" rule is modelled as a cost reduction: to reach a score whose
table cost is `T`, the XP *charged* is the smallest `c` with `ceil(c·3/2) ≥ T`,
i.e. `charged = floor((T−1)·2/3) + 1` for `T > 0` (and `0` for `T = 0`). The cap
exemption is read off the effect's presence (age cap itself is M4/4e).

- Source: `ArMDE:3372-3374`.
- Data: `rules/core/virtues_flaws.json` `virtue.affinity_ability` —
  `category: general`, `ability`-domain param, `effects: [{ affinity_ability_cost,
  param: "ability", counts_as_num: 3, counts_as_den: 2 }]`.
- Implementation: `effective/xp.rs::charged_cost` (the `floor(den·(T−1)/num) + 1`
  arithmetic, verified against the worked example below) + `ability_affinity`,
  folded into `effective/xp.rs::xp_allocation` and so into
  `validation/magus.rs::validate_xp_pool` (:982). **Not** the simpler
  `ceil(T·den/num)`, which looks equivalent and agrees with it on the worked
  example below, but overcharges by one XP whenever `T·den mod num` falls
  strictly between `0` and `den` — row 47 / V/F-audit F-547, fixed after
  `docs/book-template-conformance.md` § S2 caught it against the Specialist grog
  template's own printed arithmetic (Single Weapon 7, `T = 140`: the correct
  charge is 93, not 94).

#### Affinity with (Art) — creation XP counts for half again
> "Your Advancement Totals for one Hermetic Art are increased by one half, rounded
> up. At character creation, any experience points you put into that Art are also
> increased by one half (rounded up) … You may take this Virtue twice, for two
> different Arts."

Worked example (`ArMDE:2443`): "He spends 37 points on Perdo, which his affinity turns
into 56 points, so that he has Perdo 10 (1)" — Perdo 10 needs 55 on the Art table,
and `charged_cost(55, 3/2) = floor(2·54/3) + 1 = 37` (which happens to equal
`ceil(55·2/3)` too — see the caveat above). (Perdo is a Technique/Art; the
identical rule applies to Abilities but against the 5×-larger Ability table.)

- Source: `ArMDE:3376-3378`, example
  `ArMDE:2443`.
- Data: `rules/core/virtues_flaws.json` `virtue.affinity_art` — `category:
  hermetic`, `art`-domain param, `effects: [{ affinity_art_cost, param: "art",
  counts_as_num: 3, counts_as_den: 2 }]`.
- Implementation: `effective/xp.rs::charged_cost` + `art_affinity`, via
  `xp_allocation`.

**The ratio itself is validated at load.** `counts_as_num`/`counts_as_den` (and
`grants_spell_mastery`'s `advancement_num`/`advancement_den`, the same ratio under
other names) are ruleset-authored numbers that reach the `charged_cost`
arithmetic unfiltered, so a hand-authored `0` denominator would price every score
under the Affinity at **0 XP** — silently, in the player's favour, with the
character still validating clean in Enforced mode.
`ruleset/integrity.rs::validate_item_ratios` therefore rejects a zero on
either side of all **five** ratio-bearing effects, naming the offending item, as
`validate_item_share` does for `max_share_of_kind`. A numerator *above* the
denominator stays legal here — unlike a share, that is the ordinary case (3/2,
5/4, 2/1), because the ratio reduces a cost rather than capping a count.

The fifth is `LocalityAbilityCapFraction` (Foreign Upbringing's 1/2 cap
narrowing, `ArMDE:6160`), added in round 4 — it is **not** an Affinity, which is
why the validator is no longer named for one. A zero breaks it the opposite way:
its use site (`effective/reputation_and_caps.rs::ability_age_cap`) guards with
`den > 0` and skips the whole narrowing, so a zero denominator leaves the Flaw
silently inert while its points still count, and a zero numerator caps every
locality-dependent Ability at 0 and rejects a character the rules allow. Each
family therefore carries its own message body rather than an Affinity-shaped one.
`charged_cost` additionally treats a degenerate ratio as full price, so an
unvalidated ruleset fails safe rather than free. Tests:
`an_affinity_with_a_zero_denominator_fails_the_load_naming_the_item` and its
numerator / Art / group / mastery siblings, plus `ordinary_affinity_ratios_load`
(`ruleset.rs`); `affinity_charged_cost_matches_perdo_example` and
`charged_cost_is_the_smallest_charge_that_reaches_the_table_cost` (`effective.rs`).

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

- Source: `ArMDE:3711-3713` (Educated), `ArMDE:5227-5229` (Warrior), `ArMDE:4806-4808`
  (Privileged Upbringing).
- Data: `rules/core/virtues_flaws.json` `virtue.educated` /`virtue.warrior` /
  `virtue.privileged_upbringing` — `effects: [{ restricted_ability_xp, amount: 50,
  abilities | categories }]`. The `50` lives here.
- Implementation: `effective/xp.rs::xp_allocation` builds a bipartite **max-flow**
  feasibility graph (general pool + one node per restricted pool → eligible spends
  → sink). A greedy assignment is incorrect under overlapping eligibility
  (Educated's academic ids overlap Privileged's `academic` category), so flow is
  used. `validation/magus.rs::validate_xp_pool` (:982) reports `not_enough_xp` (with
  `shortfall`) and `restricted_xp_unspent` (warning, naming the granting item
  through `origin_kind`/`origin` — see the life-stage section for why the pool has to
  be named). `not_enough_xp` and `xp_solve_bound_exceeded` are owned by Abilities and
  list the other steps that spend the pool in `also_phases` (Arts, Spells — from
  `effective/xp.rs::shared_pool_phases`, read off `SpendKind`), so the overspend
  shows on every XP step (tryout-findings-2026-10-03 #9b, #11). Presentation only;
  the computation is unchanged.
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
- **Overall total (W3, tryout-findings-2026-10-03 #10/#11).** `XpAllocation::total_supply`
  is every experience point the character has: `general_pool` plus each flow pool's
  `amount` — the restricted ability-XP pools, the life-stage blocks and the
  Spell-Mastery pool (which `restricted` never surfaces) — summed saturating, like
  `total_demand`. It is a read-out, not a rule: no validation reads it. The XP bar's
  overall chip reads "spent `total_demand` of `total_supply`" (`EffectiveScores::xp_total_supply`,
  `XpBar.svelte`, Fluent `xp-total`), shown on the Experience, Abilities, Arts and
  Spells tabs/steps. `max_flow` is deliberately not M: it equals `total_demand`
  whenever the spend is legal. `not_enough_xp` keeps `pool = max_flow`, the fundable
  amount, which is the honest number for an overspend (it can be below
  `total_supply` when a restricted pool is left unused).
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
    *size* is thoroughly sourced (`ArMDE:2213` 75 + 45, `ArMDE:2214` 15/yr, `ArMDE:2215` 240 for
    apprenticeship, `ArMDE:2216` 30/yr after the Gauntlet), but **no passage in
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

#### Church Upbringing — a restricted-XP earmark of the NORMAL budget (D13)
> "The player must spend 25 experience points from the normal budget on Artes
> Liberales, Latin, Music, Organization Lore: Church, or Theology. Unless the
> character has a Virtue that permits it, no other experience points may be
> spent on Academic Abilities."

A **third** XP mode, distinct from Educated/Warrior/Privileged Upbringing above:
those are *additive* grants (new points on top of the budget); this earmarks 25
of the budget the character *already has*, narrowing only which Abilities that
slice may fund. Modelled with `RestrictedAbilityXp.from_normal_budget: bool`
rather than a fourth pool shape, so the flow-solve's existing two-phase
restricted-pool preference (see above) funds it deterministically instead of
inventing a `general → pool` edge that competes with ordinary general spends on
equal footing (`docs/vf-audit/design-d0-xp-modes.md` § 2).

- Source: `ArMDE:5789-5791`.
- Data: `rules/core/virtues_flaws.json` `flaw.church_upbringing` — `classification:
  creation_effect`, `effects: [{ restricted_ability_xp, amount: 25,
  from_normal_budget: true, abilities: [ability.artes_liberales, ability.music,
  ability.theology_christian], instances: [dead_language:language.latin,
  organization_lore:organization.church] }]`. The `25` lives here. Latin and
  Organization Lore: Church are instance-scoped (D14/D48's existing mechanism,
  same as Educated's Latin above); `organization.church`
  (`rules/core/parameter_catalogues.json` `catalogue.organization`) is new,
  sourced at `ArMDE:5791` (the only place the rulebook names it).
- Implementation: `effective/xp.rs::general_pool_and_bonus` subtracts every
  `from_normal_budget: true` grant's `amount` from the general pool's own base,
  in every funding branch (life-stage or `xp_pool`), and
  `restricted_ability_xp_pools` funds the earmark as an ordinary `SOURCE`-fed
  `FlowPool` of that same size — total capacity is unchanged, only reassigned.
  **Capped, not unconditional**: when the earmark exceeds the general pool it
  would draw from, the amount actually shifted is `amount.min(base_general)`,
  never more than the character has — otherwise a young or otherwise
  general-XP-poor character's total budget would *rise* by taking the Flaw,
  which is exactly the ceiling D13 forbids.
- Authorization comes free, exactly as it does for Educated/Warrior/Privileged
  above: `ability_authorizations`'s existing `RestrictedAbilityXp` arm folds
  `abilities`/`instances` regardless of `from_normal_budget`, so the earmark's
  five named entries are authorized and every *other* Academic Ability stays
  gated by `categories_requiring_virtue` — clause 2 of the passage, with no
  second effect.
- Validation: **zero new validator code.** `CODE_RESTRICTED_XP_UNSPENT`
  (`validation/magus.rs`, see above) already warns on any under-spent
  `FlowPool`; an earmark that cannot be fully absorbed (its eligible Abilities
  capped below 25, or the general pool itself too small) is lost, not refunded
  to general, and surfaced only by that existing warning — Norbert's resolution
  of `docs/vf-audit/design-d0-xp-modes.md` § 6.1 (2026-09-28): D13 does not rule
  on the shortfall case, and a refund mechanism would reopen the over-funding
  risk the earmark exists to close.
- The Flaw's own escape clause ("unless the character has a Virtue that
  permits it") is **not modelled** — text only, in `description`, both locales.

#### Feral Upbringing, Redcap, Lone Redcap — replacing a life-stage block (D40, D2)

A **fourth** XP mode, distinct from the additive grants and the D13 earmark
above: these three REPLACE a named life-stage block's own normal grant with a
different total and eligibility, rather than adding to it. The pre-D2 data
shipped all three as additive `RestrictedAbilityXp` grants stacked on top of
the standard block, over-funding the character (F-428, F-439). Modelled as
`Effect::ReplacesLifeStageXp { stage, amount, abilities, categories, years }`,
naming a `LifeStageBlock` (`ChildhoodSpread` or `Apprenticeship`), consumed in
`effective/xp.rs::build_flow_pools` rather than `restricted_ability_xp_pools`
(`docs/vf-audit/design-d0-xp-modes.md` § 3).

> "You grew up in the wilderness … You may only choose beginning Abilities
> that you could have learned in the wilds. In particular, you may not start
> with a score in a Language. In your first five years you gain 120
> experience points, which must be split between (Area) Lore, Animal
> Handling, Athletics, Awareness, Brawl, Hunt, Stealth, Survival, and Swim."

- Source: `ArMDE:6110-6113` (Feral Upbringing).
- Data: `rules/core/virtues_flaws.json` `flaw.feral_upbringing` —
  `effects: [{ replaces_life_stage_xp, stage: childhood_spread, amount: 120,
  abilities: [the nine names above] }]`. `years` absent (0): a flat block
  replacement never carves a span.
- Implementation: `build_flow_pools` skips BOTH ordinary childhood pools
  (native-language and spread) when a `ChildhoodSpread`-stage replacement is
  present, and pushes exactly ONE `FlowPool` from the replacement's own
  `amount`/`abilities`, tagged `origin: LifeStage{block: ChildhoodSpread}` —
  the SAME tag the ordinary spread pool uses (reuses `XpBar.svelte`'s existing
  "spread-only" display branch, zero new UI). No native-language pool at all,
  matching "may not start with a score in a Language" — structural, not merely
  "nobody set one" (proved by
  `a_feral_upbringing_companion_with_a_native_language_set_still_gets_no_native_pool`,
  `crates/arm-rules/src/effective.rs`).
- **D63 (Norbert, 2026-09-28):** the wilderness list and "no Language" rule
  bind ONLY this 120-XP first-five-years pool, never later life — a Feral
  character still funds Latin (or any other Ability) from ordinary later-life
  general XP, and Church Upbringing's earmark (D13/D1) composes with it
  unchanged, capped against that same general pool.
- **Pool-mode scoping (Norbert, 2026-09-28):** the replacement fires only
  under `AbilityFunding::LifeStages` (`build_flow_pools`'s life-stage-budget
  branch) — under pool/direct-entry funding there is no childhood block to
  replace, so the character's typed `xp_pool` is the sole authority and the
  Flaw grants nothing extra. See
  `pool_funding_grants_no_replacement_pool_for_feral_or_redcap`
  (`tests/data_integrity.rs`).

> "You are trained in a similar manner to magi, and may take Academic,
> Arcane, and Martial Abilities during character generation. You have spent
> fifteen years as an apprentice, and gained a total of 300 experience points
> in those fifteen years. … You must spend two seasons per year delivering
> messages for the Order."

- Source: Redcap `ArMDE:4842-4851, :4848`; Lone Redcap `ArMDE:4319-4326,
  :4321` ("You still begin with 300 experience points for your fifteen years
  spent as an apprentice").
- Data: `rules/core/virtues_flaws.json` `virtue.redcap` (adds
  `replaces_life_stage_xp` alongside its existing `item_level_budget: 50`) and
  `virtue.lone_redcap` (replaces its old additive `restricted_ability_xp`) —
  both `{ replaces_life_stage_xp, stage: apprenticeship, amount: 300,
  years: 15, categories: [academic, arcane, general, martial, supernatural] }`.
  Redcaps are companions (ArMDE:2279), never magi, so this is the ONLY place
  either carries an apprenticeship-shaped block.
- Implementation: `life_stage.rs::extra_apprenticeship_years` (`pub(crate)`)
  reads the largest `years` named by any effective `ReplacesLifeStageXp{
  stage: Apprenticeship, ..}` — 0 when none. `budget()` feeds this into a
  SEPARATE local, `later_life_carve_years =
  magus_apprenticeship_years.max(extra_apprenticeship_years(..))`, which goes
  ONLY into `later_life_years(gauntlet_age, later_life_carve_years)` — never
  into `LifeStageBudget.apprenticeship_years` itself, which stays sourced from
  a real magus's own block alone (its doc comment's "15 for a magus, 0 for
  anyone else" holds for a Redcap too). `build_flow_pools` then pushes the
  300-XP pool as an ordinary `SOURCE`-fed `FlowPool`
  (`origin: LifeStage{block: Apprenticeship}`) — never `general`, since
  ArMDE:4848 excludes Arts. A 25-year-old Redcap/Lone Redcap therefore totals
  120 (childhood) + 300 (apprenticeship-shaped) + 75 (5 later-life years ×
  15/year) = 495, never 720 (F-439's over-funded shape) or 420 (Redcap's own
  previously-unfunded shape).
- **Wealthy/Poor (Norbert, 2026-09-28):** the 300 XP is a FIXED figure, never
  scaled by `later_life_rate` — only the ordinary later-life years after the
  carve are, e.g. a Lone Redcap with Wealthy gets 5 years × 20/year = 100, not
  15/year. Confirmed by
  `lone_redcap_apprenticeship_is_flat_and_later_life_uses_wealthys_rate`.
- **Age check (Norbert, 2026-09-28):** a Redcap or Lone Redcap younger than
  `childhood.years` plus the carved years (5 + 15 = 20 against the shipped
  ruleset) raises `CODE_LIFE_STAGE_AGE_BEFORE_TRUNCATION` (error, `Experience`
  phase, `age`/`min` args) — the SAME code the truncated-apprenticeship
  carrier below uses, reused rather than forked, though the two carriers'
  `min` formulas differ (`childhood.years + carved_years` here;
  `truncated_apprenticeship_start(..) + years_completed` below — mutually
  exclusive in the catalogue, so `validate_life_stage_age_meets_minimum`
  (`validation/life_stage.rs`) tries the truncated-apprenticeship formula
  first and falls back to this one). Fluent:
  `issue-life_stage_age_before_truncation`, both locales.

#### Abandoned Apprentice — truncated apprenticeship, funding two later-life spans (D56, D62, D64, D3)

> "Decide at what age the character was abandoned. Create the character as a
> regular apprentice up until that age, and then give him experience points
> based on his age and other Virtues for his life past being abandoned. If
> the character knows the Parma Magica, he must join the Order or be slain."

- Source: `ArMDE:5641-5650`.
- **D56/D62 (Norbert):** the character is Hermetically trained by selection,
  not by profile — `is_magus` splits into *trained*/*Order member*
  (`effective/hermetic_training.rs`) — and the parameter he records is
  `years_completed` (1..=`apprenticeship.years - 1`), not an age.
- **D64 (Norbert, 2026-09-28):** the ordinary later-life years split into TWO
  spans around the truncated block, not one merged span (superseding an
  earlier draft of this design): the years BEFORE it stay Abilities-only
  (the Arts are not open yet); the years AFTER it may fund Arts as well
  (`"he knows Hermetic magic"`, `ArMDE:5643` — his Arts are already open).
- Data: `rules/core/virtues_flaws.json` `flaw.abandoned_apprentice` —
  `parameters: [{ key: years_completed, type: { number: { min: 1, max: 14 } },
  domain: number }]`, `effects: [{ confers_hermetic_training_if,
  param: years_completed }, { truncated_apprenticeship_xp,
  param: years_completed }]`, `advisory_prerequisites: { none: [{ ability_min,
  ability: ability.parma_magica, score: 1 }] }` (Parma advisory, see below).
  `rules/core/life_stages.json` `apprenticeship.truncated_xp_per_year: 16`,
  `truncated_spell_levels_per_year: 8` — FIXED, derived from `ArMDE:2435`'s
  240 XP / 120 spell levels over 15 years (`decisions.md` D56, "not to be
  reopened as a house rule").
- **`Effect::ConfersHermeticTrainingIf { param }`** — the conditional sibling
  of the bare `ConfersHermeticTraining` marker (same "Foo"/"FooParam"
  precedent as `AbilityScoreGrant`/`AbilityScoreGrantParam`): true only once
  the OWNING selection's own `param` resolves to ANY value — a presence
  test. Without this, `is_hermetically_trained` would flip true the instant
  the Flaw is picked, before `years_completed` is answered, funding nothing
  (F1). `effective/hermetic_training.rs::entity_confers_hermetic_training`
  gains a second match arm for it, the bare marker's seven existing sites
  untouched.
- **`Effect::TruncatedApprenticeshipXp { param }`** — funds the GENERAL pool
  (unlike `ReplacesLifeStageXp`, which replaces a RESTRICTED block), at
  `truncated_xp_per_year`/`truncated_spell_levels_per_year` times the
  resolved `years_completed`, `saturating_mul`'d (F3 — matches
  `ScaledRestrictedAbilityXp`'s own precedent, so a crafted `years_completed`
  at `u32::MAX` cannot panic under `overflow-checks = true`).
- **The two later-life spans, and the "start" they pivot on:**
  `life_stage.rs::LifeStageRules::truncated_apprenticeship_start` — the
  Gauntlet age a real apprenticeship would have reached (the plan's own
  `gauntlet_age`, else `apprenticeship.default_gauntlet_age`) minus
  `apprenticeship.years` — deliberately NOT clamped to the character's own
  age (unlike a real magus's Gauntlet age): this is a hypothetical milestone
  he never reached. Worked example (age 20, `years_completed` 7, shipped
  data): `start = 25 - 15 = 10`. Pre-span `10 - childhood.years(5) = 5` years
  × 15/yr = 75 XP, reuses the EXISTING `later_life_xp`/
  `LifeStageBlock::LaterLife` restricted-pool plumbing verbatim (no new tag).
  Post-span `20 - (10 + 7) = 3` years × 15/yr = 45 XP, folded into
  `general_pool_and_bonus` alongside `truncated_training_xp` (16×7=112) —
  `general_pool = 157`, never the pre-D64 merged 120-all-restricted shape.
  `LifeStageBudget` gains `truncated_training_years/xp/spell_levels` and
  `truncated_training_post_span_years/xp`.
- **Age check:** `life_stage::truncated_apprentice_years_completed` (the
  presence-gated `years_completed` reader, shared by `budget()` and the
  validator so the two can never derive different figures) feeds
  `validate_life_stage_age_meets_minimum` a `min = start + years_completed`
  branch, ahead of the Redcap-shaped one above — `CODE_LIFE_STAGE_AGE_BEFORE_
  TRUNCATION`, error, `Experience` phase. A 15-year-old with 7 years
  completed is refused (`min: 17`, the OLD childhood-only formula's `12`
  would have missed this entirely); a 6-year-old with 14 is refused
  (`min: 24`).
- **Integrity (mandatory, not a nice-to-have):** `years_completed`'s
  authored `max` must equal `apprenticeship.years - 1` exactly
  (`ruleset/integrity.rs::validate_truncated_apprenticeship_param`) — "taken
  from the ruleset rather than hardcoded" (Norbert). An item declaring
  `TruncatedApprenticeshipXp`/`ConfersHermeticTrainingIf` in a ruleset
  shipping NO `apprenticeship` block at all is rejected outright, naming the
  item — not silently skipped.
- **Parma advisory:** `advisory_prerequisites: Nor([AbilityMin{
  ability.parma_magica, 1}])` — a HEDGED restriction (F-550/Q8's existing
  machinery), warning only when Parma Magica IS known (`ArMDE:5647`: "if the
  character knows the Parma Magica, he must join the Order or be slain" —
  the danger is in knowing it unjoined, never in lacking it).
  `Prereq::Has(ability.parma_magica)` — an earlier design draft's own
  example — is WRONG for this: `Has` resolves against selected point items
  (Virtues/Flaws), never `Entity::ability_scores`; `AbilityMin` is the
  variant that reads scores.
- Tests: `crates/arm-rules/src/effective/hermetic_training.rs`,
  `crates/arm-rules/src/effective/xp.rs`, `crates/arm-rules/src/validation/
  {authorization,life_stage}.rs`, `crates/arm-rules/tests/data_integrity.rs`.

#### Improved Characteristics — +3 Characteristic-buy points
> "You have an additional three points to spend on buying Characteristics … You
> may take this Virtue multiple times."

- Source: `ArMDE:4103-4105`.
- Data: `rules/core/virtues_flaws.json` `virtue.improved_characteristics` —
  `effects: [{ characteristic_points, amount: 3 }]`, `max_per_target: 255`. The
  `3` lives here; the repeat allowance ("multiple times", no stated ceiling) is
  the `max_per_target` — see *Selection multiplicity* above. Copies stack: two
  grant 6 points.
- Implementation: `effective/characteristic.rs::characteristic_points_granted` sums the grants;
  `validation/scores.rs::validate_characteristics` (:35) budget = `start_points + granted`. The
  per-characteristic +3 *cap* is unchanged (only Great Characteristic widens it).

#### Weak Characteristics — −3 Characteristic-buy points (`characteristic_points`, signed)
> "You have three fewer points to spend buying Characteristics … You may take
> this Flaw twice, leaving you with only one point to spend."

- Source: `ArMDE:7056-7058`.
- Data: `flaw.weak_characteristics` — `effects: [{ characteristic_points, amount:
  -3 }]`, `max_per_target: 2`. `Effect::CharacteristicPoints.amount` is `i8`
  (signed) and `characteristic_points_granted` returns `i32`, so Weak nets against
  Improved (+3 and −3 cancel). The budget clamps naturally via the cost check.

#### Giant Blood / Large / Dwarf / Small Frame — Size + free Characteristic bonus
> Giant Blood: "Your Size is +2 … gain +1 to both Strength and Stamina. This
> bonus may raise your scores … as high as +6." Dwarf: "Your Size is −2 … −1 to
> each of Strength and Stamina … as low as −6." Large: Size +1. Small Frame: Size −1.
> Each lists the other three as mutually exclusive.

- Source: `ArMDE:3975-3978` (Giant Blood), `ArMDE:4229-4231` (Large), `ArMDE:5996-5998` (Dwarf),
  `ArMDE:6767-6769` (Small Frame).
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

- Source: `ArMDE:7019-7021`.
- Data: `flaw.warped_by_magic` — `effects: [{ warping_grant, points: 5 }]`. No `score`
  key (D77.3): a stored score would be an unread second copy of the same fact the
  point total already derives, and could only ever disagree with it.
- Implementation: `Effect::WarpingGrant { points }` → `effective/warping.rs::warping`
  (derived `(score, points)`, base 0 each, summed across grants — never stored, like
  Confidence). Surfaced as `EffectiveScores.warping_{score,points}` and shown on the
  sheet via Fluent `warping-label`/`warping-readout` (DE "Verzerrung", per the
  glossary). **Deferred (separate machinery):** the free unbalanced Minor Flaw
  (nested-grant, B6) and the "spend XP on Magic Lore" purchase permission are not
  part of this numeric effect.

#### Linguist — group Affinity over all Languages (`group_affinity_cost`)
> "All Advancement Totals for any Language are increased by a quarter, rounded up
> … Both Living and Dead languages are augmented with this Virtue."

- Source: `ArMDE:4315-4317`.
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
> (`ceil(21/2) = 11`); 10 XP → 5 each. (`ArMDE:3731-3737`.)

- Source: `ArMDE:3731-3737`.
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
> (`ArMDE:9516`), one of the six Ability types (`ArMDE:7143`, `ArMDE:7163-7165`), bought from the
> Ability advancement table "(Ability + 1) x 5" (`ArMDE:15952`; To-Buy table 1=5, 2=15,
> 3=30, 4=50, 5=75 at `ArMDE:15956-15979`). Mastered Spells: "You have fifty experience
> points to spend on mastering spells that you know … You may take this Virtue
> multiple times." Flawless Magic: "All your spells start with a score of 1 in the
> corresponding Spell Mastery Ability … all your Advancement Totals for Spell
> Mastery Abilities are doubled."

- Source: `ArMDE:9516`, `ArMDE:7143`,
  `ArMDE:7163-7165` (mastery is an Ability type); `ArMDE:15952`, `ArMDE:15956-15979` (Ability
  To-Buy table); `ArMDE:4471-4474` (Mastered Spells); `ArMDE:3887-3889` (Flawless Magic).
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
  the same floor + Affinity charge in `derive.ts::spellMasteryXpSpent`, driven by
  `EffectiveScores.spell_mastery_{xp,floor}` and `spell_mastery_advancement_affinity`
  — the authored `[num, den]` pair, not a "doubled" flag, so the UI charges the same
  reduced cost for any ratio the catalogue authors (full-audit round 2, V2);
  the SpellPicker mastery spinner shows for every magus (buyable from the general
  pool). "You may take this Virtue multiple times" (`ArMDE:4474`) states no ceiling, so
  `virtue.mastered_spells` carries `max_per_target: 255` — see *Selection
  multiplicity* above. (It formerly sat at the default 1, which blocked the repeat.)

#### Mastered Spell Special Abilities — choosable per-spell mastery options (`spell_mastery_abilities`)
> "For every level in the Mastery Ability, the maga may also choose one special
> ability, which applies only to that mastered spell. Thus, a maga with a Mastery
> Score of two for a spell has two special abilities for that spell." (`ArMDE:9524-9526`)
> The catalogue of fourteen options is at `ArMDE:9528-9592`: Adaptive, Ceremonial, Fast,
> Imperturbable, Magic Resistance, Multiple, Obfuscated, Penetration, Precise,
> Quick, Quiet, Rebuttal, Still Casting, and Unravelling. Most are once-per-spell;
> Precise (`ArMDE:9570-9572`), Quick (`ArMDE:9574-9576`), and Quiet Casting (`ArMDE:9578-9580`)
> "may take this ability multiple times for the same spell".

- Source: `ArMDE:9524-9526` (one special
  ability per Mastery level); `ArMDE:9528-9592` (the catalogue); `ArMDE:9572`, `ArMDE:9576`,
  `ArMDE:9580` (Precise/Quick/Quiet are repeatable). German source
  `Ars Magica Definitive Edition Basisregeln.md:9524-9592` mirrors it line-for-line.
> Quiet Casting (`ArMDE:9578-9580`): "A maga may take this ability twice" — a hard
> ceiling of 2, tighter than the unlimited "multiple times" Precise/Quick Casting get.
> Ceremonial Casting (`ArMDE:9534`), Fast Casting (`ArMDE:9540`), and Quick Casting
> (`ArMDE:9576`) "may not be taken for Ritual spells"; Multiple Casting (`ArMDE:9560`)
> is the explicit converse, "may be taken for Ritual spells".

- Data: catalogue in `rules/core/spell_mastery_abilities.json` — each entry an `id`
  (`spell_mastery_ability.<slug>`) + a `repeatable` bool (true only for Precise/
  Quick/Quiet Casting) + an optional `max_count: u8` (only Quiet Casting, `2`) +
  a `forbidden_for_ritual` bool (true only for Ceremonial/Fast/Quick Casting) +
  `source`. Names/descriptions in
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
  integrity — an unknown chosen id fails (`CODE_UNKNOWN_MASTERY_ABILITY`); (d) a
  `max_count`-bearing ability (Quiet Casting) chosen more times than its cap, even
  though it is also `repeatable` (`CODE_TOO_MANY_OF_MASTERY_ABILITY`); (e) a
  `forbidden_for_ritual` ability chosen for a spell whose `ritual` flag is set
  (`CODE_MASTERY_ABILITY_FORBIDDEN_FOR_RITUAL`). Load-time
  integrity checks each catalogue entry's `source` range. UI: per-spell add/remove
  picker in `SpellPicker.svelte` (store `addMasteryAbilityAt`/`removeMasteryAbilityAt`),
  greying non-repeatable already-chosen options and hiding the add control once the
  count reaches the effective mastery.

#### Templar Commander — fixed nested free Virtue grant (`grants_selection`)
> "This Virtue also grants the Temporal Influence Minor Virtue … This Virtue
> includes the effects of the Brother-Knight Virtue. … he is a wellknown figure
> and has a Reputation of level 3 in his area."

- Source: `ArMDE:5113-5116`.
- Data: `virtue.templar_commander` — `grants_selection: [virtue.brother_knight,
  virtue.temporal_influence]`, plus (F-312, D11/Q5) `grants_reputation{local, 3}`
  for the "Reputation of level 3 in his area" clause, carried by no effect
  before this fix.
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

- Source: `ArMDE:4347-4349` (Magic
  Items), `ArMDE:4842-4846` (Redcap).
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
  `EffectiveScores.item_level_used`), and `validation/might.rs::validate_devices` (:123) emits
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
  adds the aura (ArMDE:9089); every Lab Total takes it as a plain addend — "your
  basic Lab Total is: Technique + Form + Intelligence + Magic Theory + Aura
  Modifier" (ArMDE:10276-10278) — with no gate on a nonzero aura (see M5.5a).
- **Familiar cords** — `Entity.familiar: Option<Familiar { name, cord_gold,
  cord_silver, cord_bronze: u8 }>`. Gold = −botch dice; Silver = +Personality /
  mental resistance; **Bronze = +Soak & aging-resistance** (feeds 5i Soak /
  longevity). Source: `ArMDE:10840-10844`.
- **Talisman** — `Entity.talisman: Option<Talisman>` (see M5.5b below). Originally
  a flat `talisman_attunements: Vec<TalismanAttunement>`; schema 14 moved it under
  the item.
- **Longevity Ritual** — `Entity.longevity_ritual: Option<LongevityRitual { source:
  LongevitySource (SelfMade|External), bonus: Option<i8>, focus: String }>`. The
  bonus is **player-entered for both sources** and stored; `None` means "not entered
  yet", never a claimed 0 (see M5.5a for why it is not derived). `focus` is the
  ritual's culminating focus, free text. Source:
  `ArMDE:10662` (formula), `ArMDE:10656`
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
  Source: `ArMDE:10603-10625`
  (`ArMDE:10605` identity, `ArMDE:10607` one-at-a-time).
- **Identity** — `Talisman.description: String`, free text: shape and material are
  open-ended (the Shape and Material Bonuses Table is not a closed catalogue in
  this model), so there is no id to reference.
- **Attunements** — `Talisman.attunements: Vec<TalismanAttunement { description,
  bonus: i8 }>`, unchanged in shape, only re-homed. "you may also open your
  talisman to one kind of magic attunement, based on the shape and material of the
  talisman, every time you prepare it for enchantment or instill an effect"
  (`ArMDE:10623`); "only the highest bonus applies. They apply to Casting Scores for
  Ritual, Formulaic and Spontaneous magic, but they do not apply to Magic
  Resistance or any laboratory activities" (`ArMDE:10625`). This corrects the M5/5e
  citation, which named no lines at all.
  **The bonus is stored but still unread** — nothing in `derived.rs` adds it to a
  Casting Total (`casting_totals` never looks at `talisman.attunements`;
  `talisman_capacity` is the only talisman consumer), in the same sense as
  `lab_enchanting` being *collected but unread* in the V/F effect table below. This
  is a decision, not an omission: the attunement's subject is free text
  (`TalismanAttunement.description`), so the engine cannot tell which cell of the
  casting grid an attunement enhances, "only the highest bonus applies" needs that
  same judgement to pick a winner, and the bonus applies only "when the magus is
  touching the talisman" (`ArMDE:10625`) — a moment of play the model does not represent.
  So it is a situational modifier the player applies at the table, exactly like
  Soak's Form bonus (entered 0, see the *Soak* row). Computing it would require
  either a closed attunement catalogue keyed to spell Techniques/Forms or a
  per-total "touching my talisman" toggle; both are out of M5.5b's scope.
- **Instilled effects** — `Talisman.effects: Vec<TalismanEffect { name, level: u16 }>`.
  "When a magus instills effects into a talisman, he gets a +5 bonus to his Lab
  Total" (`ArMDE:10621`). Deliberately **not** a reused `EnchantedDevice`: the two carry
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
Format* (`ArMDE:17787-17827`) so a save reads like the printed creature entry:

> A familiar is a beast that a magus befriends and then magically bonds with,
> instilling the beast with magical powers in the process … Though a familiar is
> very close to the magus who creates it, it always has its own will, and is not
> under the control of the magus. — `ArMDE:10770`

Source for the whole chapter: `ArMDE:10766-10892`.

- **`animal: String`** (free text) — the kind of beast. "The first step in getting a
  familiar is finding an animal with inherent magic" (`ArMDE:10774`). Deliberately **not**
  named `species`: the rules reserve *Species* for the Imaginem term (the sensory
  image a thing sheds), and the German glossary makes the same reservation
  (`translation-tables/grundbegriffe.md:112`), so "Spezies" is the wrong label too.
- **`might: Option<MightScore>`** — the familiar's own Magic Might + Realm. "the
  beast is likely to have a Magic Might score, which may be assigned based on the
  scores of comparable magical creatures" (`ArMDE:10774`); the Creature Format prints it
  as the `(Realm) Might:` line (`ArMDE:17791`). It is the **familiar's** Might: no Virtue
  grant stacks on top, which is why the UI labels it with its own Fluent key rather
  than reusing `might-score-label` ("Base Might Score").
- **`characteristics: BTreeMap<Characteristic, i8>`** — the creature's own
  Characteristics ("A list of the characteristics and values", `ArMDE:17793`), signed and
  **never bought from the magus's Characteristic points**. A 0 score is **pruned by
  `Familiar::normalize()`**, and `skip_serializing_if` then omits the map once it is
  empty — so an explicit 0 and an absent entry write identical bytes, as canonical
  serialization requires. Pruning (rather than keeping a stored 0) is safe because
  these scores are display-only: no engine read-out or validation reads them, so a
  deliberately-entered 0 carries no mechanical meaning an absent entry lacks. Pinned
  by `entity_normalize_prunes_zero_familiar_characteristics`.
- **`size: i8`** — signed and commonly **negative**; a raven is Size -4
  (`ArMDE:17829-17856`, the Size examples table). It lowers the bonding level: "If the
  familiar has negative Size, this reduces the level for the enchantment" (`ArMDE:10824`).
  Every displayed sign is the ASCII hyphen-minus, per the project convention.
- **`personality_traits: Vec<PersonalityTrait>`** — `ArMDE:17807`. Reuses the existing
  value struct; kept sorted by `Familiar::normalize()`.
- **`powers: Vec<SupernaturalPower>`** — the powers invested in the bond
  (`ArMDE:10862-10884`), charged against **no** budget (`ArMDE:10866`, see the derived row).
  Kept sorted by `Familiar::normalize()`.
- The three cords are unchanged, only re-ordered to sit where the statblock puts
  them (after Personality Traits, before Powers). Source: `ArMDE:10840-10844`. A score
  above the +5 maximum (`ArMDE:10836`) is **clamped by `Familiar::normalize()`**, the same
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
(`ArMDE:17793`), but a **bound** familiar is not such a creature: "If it did not
previously have human intelligence, it gains it, with a score of –3" (`ArMDE:10854`) —
an ordinary Intelligence entry. The app only ever stores *bound* familiars, so
`Characteristic` stays the fixed eight the rules name (`ArMDE:1023-1025`). Displaying an
unbound creature's score as "Cun" would be a **display-only** affordance, and it is
deferred until the app models unbound creatures at all.

**Deferred statblock lines** (out of M5.5c scope by decision): the familiar's
Abilities (`ArMDE:17819`), Qualities, Virtues/Flaws (`ArMDE:17805`) and the
Combat / Soak / Fatigue / Wound statlines (`ArMDE:17811-17817`). These are the creature
lines the app does not yet enter for *any* creature, so adding them for the familiar
alone would be a lone special case.

**The bond's own grants are surfaced, never auto-applied.**

> The familiar binding gives both the magus and the familiar the Minor Virtue True
> Friend, relating to the other half of the partnership. Thus, they also gain
> Personality Traits of Loyal (partner) +3. — `ArMDE:10852`

> If it did not previously have human intelligence, it gains it, with a score of
> –3. — `ArMDE:10854`

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
(`ArMDE:10866`). `DerivedTotalsPanel.svelte` gains a familiar-bond section modelled on the
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
German rulebook passage rather than invented: **Kordelpunkte** (from `ArMDE:10836`
"verteile die Punkte … auf die drei Kordeln … Kordelwerte"), **Bindungsstufe** (from
the `ArMDE:10828` header "STUFE DER VERTRAUTENSBINDUNG"; the tables' `Vertrautenbindung`
spelling wins over the header's linking -s-), **Investierte Kräfte/Kraftstufen**
(from `ArMDE:10864-10866` "Kräfte in die Vertrautenbindung investieren" — deliberately
*not* "Eingebettete Effekte", which `labor-fortschritt.md:23` assigns to the
talisman's *Instilling Effects*), and **Laborsumme für die Bindung** (phrase shape
from `ArMDE:10818`). Every displayed negative sign is the ASCII hyphen-minus, including
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
  `ArMDE:16579`.
  - The former manual `aging_reductions` map was **removed** (schema 9 → 10):
    modelling aging fully from `aging_points` per the rule made a separate
    stored-drops field redundant and a divergence risk.
    `migration.rs::load_entity_migrating` migrates old saves by folding any
    legacy `aging_reductions[c] = R` into `aging_points[c]` as the *minimal*
    point total that reproduces `R` drops under the derived rule
    (`effective/warping.rs::minimal_aging_points_for_drops`, fed the actual
    pre-aging score — the same cost sequence the live `aging_drops` walks, ruling
    F-A below), reporting which
    Characteristics were migrated. Because every aging point counts toward
    Decrepitude — including those "lost" to a drop — the fold also corrects the old
    model's Decrepitude under-count. A legacy map that cannot deserialize fails the
    load loudly rather than folding in nothing (see the M5.5b migration note).
- **Warping points** — `Entity.warping_points: u32`: accrued Warping Points.
- **Twilight scars** — `Entity.twilight_scars: Vec<TwilightScar { description }>`
  (free-text; `TwilightScar` derives `Ord`, so `Entity::normalize()` sorts them for
  zero-noise diffs). Source: `ArMDE:9731`, `ArMDE:9743`.
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

  Source: `ArMDE:1155`. **No longer a
  pure annotation.** M5/D2 shipped it as one because nothing computed aging rolls;
  since M6/6b6 `aging::resolve_year` seeds and advances it per `ArMDE:16577` (see
  **Aging (M6/6b6)** below). It stays directly editable — a hand-entered figure is
  never overwritten — and it is never an *input* to a roll: the modifier "depends
  on the character's **actual, not apparent**, age" (`ArMDE:16577`).
- **Warping effect** — `Entity.warping_effect: String` (skip-if-empty), a
  free-text flavor note for how the character's Warping manifests (Issue D).
  > "This Minor Flaw should reflect the predominant source of the Warping Points."

  Source: `ArMDE:16547-16561` (### Effects
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

- Source: `ArMDE:16551-16561`.
- Implementation: `effective/warping.rs::warping_owed(entity, ruleset) -> WarpingOwed`
  (`{ minor_flaws, minor_supernatural_virtues, major_flaws }`), driven by the pure
  threshold `WarpingOwed::from_score`: Minor Flaw at Score 1, a second at 3
  (`minor_flaws` cap 2); a supernatural Minor Virtue at 5; `major_flaws =
  score.saturating_sub(5)` (Score 6 → 1, 7 → 2, …). The Hermetically trained
  (`is_hermetically_trained` — D56/A0's union of the profile's
  `hermetically_trained` flag with any selection carrying
  `Effect::ConfersHermeticTraining`, e.g. the Abandoned Apprentice Flaw) owe
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
  troupe, **not** specially handled here (the guard keys only on
  `is_hermetically_trained`).
  (ii) The 16559 clause that the supernatural Minor Virtue "stops any further gain
  of points from living in a strong aura of the same type" is a post-creation /
  in-play effect and is **NOT** modeled.
- **Decrepitude effect** — `Entity.decrepitude_effect: String` (skip-if-empty):
  free-text overall aging/decrepitude narrative. Pure annotation — Decrepitude
  itself is DERIVED from `aging_points` (see above). It is the **cumulative** account
  of what age has done to the character, distinct from the aging log's per-year
  one-liners below, so the UI renders it as a multi-line field like its analogue
  `Entity.warping_effect` (guided-creation-review-2026-08 #26). Source:
  `ArMDE:16563-16577` (## Aging).
- **Aging log** — `Entity.aging_log: Vec<AgingLogEntry>` (skip-if-empty). `year`
  is the **first** field so the derived `Ord` sorts the log chronologically via
  `Entity::normalize()`. Shipped in D2 as `{ year: i32, effect: String }` — per-year
  free-text outcomes, a pure annotation. **M6/6b6 widened it** into the full record
  of a resolved aging year and made `year` optional (`SCHEMA_VERSION` 14 → 15); the
  free text stays authoritative for a hand-written entry. See **Aging (M6/6b6)**
  below. Source: `ArMDE:16563-16577`
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
  Source: `ArMDE:16617`.
- **Warping (unified)** — `warping_points_total(entity, ruleset) =
  entity.warping_points + Σ WarpingGrant.points`, then `warping_score =
  advancement.score_for_xp(points_total)` (cumulative 5/15/30/50/75: 15 points →
  **Warping Score 2**). The two warping sources are routed through **one** function:
  `warping()` now returns `(warping_score, warping_points_total)`. `WarpingGrant`
  carries no `score` field at all (D77.3 dropped it): a stored score could only ever
  be an unread second copy of the same fact the point total derives, never anything
  the engine could additionally need. Source: `ArMDE:16464-16475`; grant at
  `ArMDE:7019-7021`.

**Aging lowers derived, not creation.** The drops are DERIVED from the accrued
points by `effective/warping.rs::aging_drops(entity, char)`: once the points **exceed** the
absolute value of the (already aged-down) score the Characteristic drops by one and
the points reset, so the simulation consumes `|score| + 1` points per drop over the
lifetime total. Worked examples encoded as tests (`ArMDE:16613`): a Communication of +2
drops on its **3rd** aging point; a Stamina of −3 on its **4th**.
`effective_characteristic_after_aging(entity, ruleset, char) = (bought + delta) − drops`,
**unfloored**, where `delta` is every free `CharacteristicScoreDelta` bonus (Giant
Blood +1 Str/Sta, Dwarf −1, Great/Poor (Characteristic), the Blood Virtues).
*Decision (ruling F-A, Norbert 2026-10-03, reversing the earlier bought-score
reading):* the threshold is the absolute value of the **actual** Characteristic,
`bought + delta` (`effective_characteristic_score`), and it is that score which drops.
`ArMDE:16579` names one noun, "the Characteristic", for both, and a free delta raises
the Characteristic itself (`ArMDE:3989`: "You may raise any Characteristic … by one
point"). So Great (Stamina) twice over a bought +3 is +5, and four aging points do not
drop it; Poor (Stamina) twice under −3 is −5 and first drops on the 6th point. The delta
is a one-time raise and is not re-clamped against the aged score, so nothing is counted
twice. The live derivation and the legacy migration fold share one cost sequence
(`effective/warping.rs::drops_forced_by_aging_points` /
`effective/warping.rs::minimal_aging_points_for_drops`). This is what DERIVED / play stats consume (5i);
creation-legality validators keep reading the **un-aged bought score** from
`entity.characteristics`, so entering an aged-down character can never
retroactively make its point-buy illegal. Source: `ArMDE:16579`, `ArMDE:16613`,
`ArMDE:3989`.

**No floor, and therefore no warning about one.** `ArMDE:16579` gives the drop
condition and names **no minimum** for an aged Characteristic, so the engine states
none either: the drops run to the arithmetic, and a decrepit character may end up
far weaker than any character could be *built*. The result is bounded by the data
rather than by a clamp — `Entity::aging_points` is a `u8` per Characteristic and
each successive drop costs one point more than the last, so the drops terminate on
their own (255 points on a −3 Stamina buy 19 of them and no more). Pinned by
`tests/data_integrity.rs` —
`aging_drops_run_to_the_arithmetic_with_no_invented_floor`.

*Removed 2026-09-15 (open-todos row 44, Norbert's decision).* A
`characteristics.json` key `aging_floor` clamped the aged score (at −5, briefly at
−10), and `validation/aging.rs` emitted an `excessive_aging_reduction` warning when
the drops passed it. Both are gone, along with the Fluent key
`issue-excessive_aging_reduction` in both locales and
`CharacteristicRules::aging_floor_score`. The clamp was an engine invention no
passage supports, and the warning was worse: it existed *only* because the clamp
did, so it warned the player about a perfectly legal character. "The warning needs
a threshold to fire against" was offered as a reason to keep the floor, which is
the argument eating its own tail — a warning can only exist where a rule does.

**Validation (advisory, single path).** `validation/aging.rs::validate_aging` (:54)
emits no per-Characteristic finding at all. (An earlier
`aging_points_force_drop` note announcing each auto-applied drop was removed as
validation noise — the drop is automatic and already reflected in the effective
score, so it is not an entry problem worth flagging.) The entity-wide findings
below are what remains.

`validate_aging` also emits the entity-wide **warning**
`aging_rolls_pending` (arg `age`, phase `aging`) when the character has
reached `ruleset.aging()?.first_roll_age()` and its `aging_log` is empty — the rolls
the rules owe before play have not been made (`ArMDE:2232`, `ArMDE:16565`; the threshold is a
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
Warping Points and twilight scars, while the aging points per Characteristic sit on
the Aging tab (`AgingPanel.svelte` → `AgingRecordPanel.svelte`; they were on Details
until the Slice 3 tab split). Between them they show the engine-computed
Decrepitude / Warping **scores** and the aging-lowered Characteristics (never
recomputed in JS). The Characteristic drop is applied automatically once the accrued
points exceed the score, and the derived totals are where the player sees it —
manual-testing-findings #21 removed the `aging-points-note` that said so in words.
Fluent keys en/de: identity + aging block (`identity-*`, `aging-*`,
`warping-points-label`, `twilight-*`, `decrepitude-{label,readout}`).

#### True Faith — special derived score (`true_faith_grant`)
> "You have a True Faith score of 1 and can gain more."

- Source: `ArMDE:5169-5171`.
- Data: `virtue.true_faith` — `effects: [{ true_faith_grant, score: 1 }]`.
- Implementation: `Effect::TrueFaithGrant { score }` → `effective/might.rs::true_faith`
  (derived, base 0 + Σ, like Warping). True Faith is a special score with its own
  rules (Core p.419), **not** a Supernatural Ability, so it is not modelled via
  `ability_score_grant`. Surfaced as `EffectiveScores.true_faith_score` and shown on
  the sheet (Fluent `true-faith-label/readout`, DE "Wahrer Glaube" per glossary).
  `flaw`/`virtue.relic` grants a *possessed* holy item with True Faith 1 — the
  character's own score stays 0 — so Relic is item-possession, left structural.
- **Also feeds Magic Resistance as a floor** (D37, F-329): `derived/casting.rs::magic_resistance`
  reads `true_faith` and takes `max(ordinary per-Form total, score × 10)` —
  see the Magic Resistance row above.

#### Second Sight / Premonitions — free starting Ability score (`ability_score_grant`)
> Second Sight: "Choosing this Virtue confers the Ability Second Sight 1."
> Premonitions: "Choosing this Virtue confers the Ability Premonitions 1."

A free bought-score *floor*, costing no XP: effective score = `max(bought, grant)
+ bonuses`. The target Ability is fixed by the Virtue (not player-chosen), so the
effect stores the ability id directly (`ability`), not a selection parameter.

- Source: `ArMDE:4888-4890` (Second Sight), `ArMDE:4788-4790` (Premonitions).
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
  bought score) is unaffected. Source: `ArMDE:2639` (a parenthetical inside
  the Mythic Companions bullet list).

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
  and the *category-restricted* magus cap "≤1 Major Hermetic Virtue" (`ArMDE:2857`)
  via the data-driven `virtue_category_caps`. See the **Houses** section above.
- **M4/4f (V/F effect families):** *done* — Affinity with Ability/Art (XP-cost
  modifier), restricted XP-grant pools (Educated/Warrior/Privileged Upbringing),
  Improved Characteristics (+3 point-buy pool), and `ability_score_grant`
  starting-score effects. See the effect-layer subsections above.
- **M6 (guided wizard):** *done* — the life-stage XP acquisition (early childhood
  75+45 xp `ArMDE:2378`; later life 15/20/10 xp/yr `ArMDE:2390-2394`; age→max-score cap
  `ArMDE:2368-2374`) in **6b2**, the Sample Childhood packages (`ArMDE:2380-2388` —
  catalogue, engine and picker, see **Sample Childhood packages** below) in
  **6b3**, the magus apprenticeship and post-apprenticeship Art-XP flow
  (`ArMDE:2433-2471`) in **6b4**/**6b5**, the aging engine for characters over 35
  (`ArMDE:16563-16617`, see **Aging (M6/6b6)** below) in **6b6**, and the Crisis of
  `ArMDE:16619-16640` — which this bullet once excluded — in **6b7**. The
  age→max-score *cap* itself is enforced as direct-entry validation in M4/4e; only
  the XP *acquisition* and aging *rolls* are M6.

### Houses (magus-only)

Every magus belongs to one of the twelve Hermetic Houses. A House confers a free
benefit — a Virtue that "you need not balance with a Flaw" (`ArMDE:2859`). The save
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

- Source: `ArMDE:2270-2283` (the House
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
  in `validation/magus.rs` (:33). `Bonisagus`/`Mercere`/`Flambeau` reuse the generic
  `virtue.puissant_ability` / `virtue.puissant_art` (target via param) — no new
  Puissant items.

#### The free House Virtue is budget-free and uncapped
> "You receive one free Minor Virtue from your choice of House, which you need
> not balance with a Flaw."

- Source: `ArMDE:2859`.
- Implementation: the derived grant model. `effective.rs::selections_for_effects`
  returns `entity.selections ++ granted_selections` so a granted Virtue's effects
  (e.g. a Mystery Ability floor) participate, but `compute_balance` and
  `validate_caps` stay on `entity.selections` (bought only) — commented as the
  line that makes grants free and uncapped. So a granted Virtue never costs points
  and never trips a virtue-count cap, and Ex Miscellanea's three grants are "in
  addition to the normal allowance" (`ArMDE:2273`) purely by construction.

#### ≤1 Major Hermetic Virtue (magus cap) — `virtue_category_caps`
> "You may not have more than one Major Hermetic Virtue."

- Source: `ArMDE:2857`.
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

#### Social Status — mandatory floor, soft ceiling (D41/B2)
> "All characters must take one Social Status, and may only take more than one
> if the descriptions of the Virtues or Flaws explicitly note that they are
> compatible."

- Source: `ArMDE:2816`.
- `CategoryCap` gains `min`/`min_hard` (a floor, additive to the existing
  ceiling-only `max`/`hard`) and `both_kinds` (counts Virtues AND Flaws
  sharing the category, regardless of which array the row lives in — the
  book's own Social Status entries are a mix of both, e.g. `virtue.gentleman`
  and `flaw.outlaw`; `false` by default preserves every OTHER shipped cap's
  kind-scoped behavior).
- Data: every profile in `rules/core/character_types.json` —
  `virtue_category_caps: [{ "category": "social_status", "max": 1, "hard":
  false, "min": 1, "min_hard": true, "both_kinds": true }]`. D41 rules the
  floor hard (an absolute "must") and the ceiling soft, and explicitly rejects
  a `compatible_with` list — it would model none of ArMDE:4325/4614/4441's
  three different exception shapes and would look authoritative while
  guessing.
- Implementation: `validation/caps.rs::validate_caps` (:22)'s existing
  category-cap loop, extended with the floor half (code
  `too_few_[major_]<category>_<flaws|virtues>`, severity following `min_hard`
  rather than `hard`) and, on a ceiling breach, naming the first two matched
  entries in `args` (`item`/`other`, sorted) so the warning shows what was
  paired rather than a bare count. Fluent keys
  `issue-too_few_social_status_virtues` / `issue-too_many_social_status_virtues`
  in both locales.
- `virtue.male_guild_sponsor` (`ArMDE:4439-4442`) needs a SEPARATE guild
  Social Status: `prerequisites: { "kind": "has_category", "value":
  "social_status" }`, evaluated via `validation/prereq.rs::PrereqCtx::evaluate_for_item`,
  which excludes the asking item's OWN contributed categories — otherwise the
  prereq would be trivially satisfied by the item's own `social_status`
  category. See `docs/vf-audit/decisions.md` D41 and
  `docs/vf-audit/design-b0-ranging-and-predicates.md`'s 2026-09-27
  amendments.

#### Predicate-valued exclusions (B3/D23/D33) — `ItemPredicate`, `ParameterDef::exclude_if`, `PointItem::excluded_if_holds`
> "Any Flaw that is only appropriate to Hermetic Magic (for example,
> Deficient Technique or Unstructured Caster) cannot be taken with this
> Flaw." (Flawed Powers, imported-Flaw constraint)

> "...can not have the Poor Flaw or any other Flaw that grants a Bad
> Reputation." (University Dean)

> "...or Virtues or Flaws that grant Personality Traits." (Weak Personality,
> second clause)

- Source: `ArMDE:6146-6148` (Flawed Powers), `ArMDE:6923-6926` (University
  Dean), `ArMDE:7076-7079` (Weak Personality, second clause).
- **The gap.** The book excludes some Flaws by DESCRIPTION — "grants a Bad
  Reputation", "only appropriate to Hermetic Magic" — where
  `incompatible_with`/`Effect::ForbidsItemCategory` can only exclude by id or
  by category (D23). C0 § 7 designed the vocabulary but did not build it; B3
  builds the enum, both consumer fields, both consumers, and the load-time
  checks in the same slice (`docs/vf-audit/design-b0-ranging-and-predicates.md`
  § 1's correction).
- **`ItemPredicate`** (`types.rs`) — a closed, snake_case-tagged enum:
  `Trained` (D12's intrinsic/trained classification, reads `PointItem::trained`
  — populated wholesale by D12's classification pass, X3, not this slice;
  every shipped item is `false` until then), `GrantsReputation` (derivable:
  carries an `Effect::GrantsReputation`), `GrantsPersonalityTrait` (derivable:
  carries the new, FIELDLESS `Effect::GrantsPersonalityTrait` marker —
  deliberately no `name`/`value`, since a trait's wording is exactly the
  translatable string `rules/core/` may never carry; the entry's own i18n
  `description` already states it, and D3 keeps the value itself free text on
  the entity). `ItemPredicate::holds_for` is the ONE evaluator both consumers
  below share, so the two mechanisms cannot drift on what a predicate means.
- **`ParameterDef::exclude_if: Option<ItemPredicate>`** (D33) narrows an
  `item`-domain parameter by predicate, additive to `require_categories`/
  `allow_ids`/`forbid_tainted`: a candidate resolves only if it does NOT
  satisfy the predicate. Flawed Powers' own fix is a D9/D14 instance too (a
  stated choice nobody recorded) — its own JSON is Phase 3's, once B3's
  machinery exists. Enforced by
  `validation/selections.rs::param_value_resolves`'s `Item` domain arm,
  reported as the existing `unknown_param_value` (the narrowing IS the
  domain, on `forbid_tainted`'s own precedent).
- **`PointItem::excluded_if_holds: Vec<ItemPredicate>`** (D23) — a
  ONE-DIRECTIONAL point-item-level exclusion: this item is illegal while ANY
  OTHER effective (bought or granted) selection satisfies one of these
  predicates. `flaw.university_dean`'s worked example:
  `excluded_if_holds: ["grants_reputation"]` (`Poor` itself stays a plain
  `incompatible_with` id — it does not "grant a Reputation", it is the OTHER
  named exclusion). Enforced by
  `validation/selections.rs::validate_excluded_if_holds` (grant-aware on both
  sides, D2 — closing the F-466 reachability trap
  `validate_incompatibilities`/`validate_forbidden_categories` are
  deliberately bought-only about, B15, the same reach
  `validate_category_effect_prohibitions` already gives D21's prohibitions —
  see both functions' doc comments for the cross-reference). New error code
  `excluded_by_predicate` (args `item`/`other`/`predicate`).
- **Load-time integrity**: `exclude_if` is rejected on any domain but `item`
  (`ruleset/integrity.rs::validate_parameter_defs`, on `allow_ids`'s own
  precedent); `excluded_if_holds`/`exclude_if` values need no further check —
  `ItemPredicate` is a closed enum, serde-checked at parse time.
- Fluent: `issue-excluded_by_predicate` — both locales.
- Tests: `crates/arm-rules/tests/b3_predicate_exclusions.rs` (hand-authored
  fixtures with invented ids; the real catalogue entries this note names —
  `flaw.flawed_powers`, `flaw.university_dean`, `flaw.weak_personality` — are
  Phase 3 data work, not B3's).

#### Parameter-gated effects (B4/Q-51) — `Effect::CharacteristicScoreDeltaParam.gate`/`GrantsReputation.gate`, Magical Blood's Magic Human clause
> "In addition, she receives a minor physical advantage appropriate to one of
> the four different types of magic beings (magic animals, magic humans,
> magic spirits, and magic things), the one that is associated with the
> character's background. ... *Magic Human:* The character may increase one
> of his Characteristics by 1, but not above +3. ... The character also has a
> positive Reputation at level 3 among others of his bloodline."

- Source: `ArMDE:4359-4372` (Magical Blood, full passage).
- **The gap.** An `Effect` fires for every copy of its owning item regardless
  of a parameter's value, so encoding Magic Human's Characteristic/Reputation
  clauses directly would wrongly apply them to Magic Animal/Spirit/Thing too
  (the F-45/F-20 shape). The missing machinery is "this effect applies only
  when parameter X holds value Y" — parameter-**gated**, distinct from
  parameter-**valued** (`characteristic_score_delta_param`'s own `param`,
  which already existed).
- **`gate: Option<ParamGate>`** added directly to `Effect::CharacteristicScoreDeltaParam`
  and `Effect::GrantsReputation` (`types.rs`) — the SAME embedded-gate idiom
  C0/C1 already established for `AbilityRef::Scoped`/`CategoryRef::Scoped`/
  `AbilityBonusGated`, not a new wrapper variant (`docs/vf-audit/design-b0-ranging-and-predicates.md`
  Revision 3 rejects a generic `Effect::Gated` wrapper on YAGNI grounds: Q-51
  names exactly two concrete carriers, so a direct field does the identical
  job with no `Box`, no second closed enum). `#[serde(default,
  skip_serializing_if = "Option::is_none")]` — additive, byte-compatible, no
  `SCHEMA_VERSION` bump (`Effect` lives in ruleset JSON, not in saves).
  `ParamGate::holds` is `pub(crate)` (was private to `types.rs`) so the two
  real consumer arms below, which live in different modules, can call it.
- **Consumer arms**: `effective/characteristic.rs::characteristic_score_bonus`
  and `effective/reputation_and_caps.rs::reputation_grants` each guard their
  existing match arm with `gate.as_ref().is_none_or(|g| g.holds(selection))` —
  an absent gate (every OTHER carrier) still applies unconditionally.
- **Load-time integrity**: `gate.param` on either variant is validated by the
  SAME `ruleset/integrity.rs::validate_param_gate` C1 already built for
  `AbilityRef`/`CategoryRef`'s own gate (dangling param, or a `MultiRef`
  param, which has no single value to gate on) — not a new invention, wired
  in `validate_effect_refs`.
- **Data**: `rules/core/virtues_flaws.json` `virtue.magical_blood` gains a
  `bloodline` (`enumerated`, values `bloodline.magic_animal`/`magic_human`/
  `magic_spirit`/`magic_thing`, localized in both locales like `being.*`/
  `folk_magic.*`) and a `characteristic` (`characteristic`-domain, player's
  choice — ArMDE:4367 names Strength/Stamina/Presence as illustrative
  examples, not a restriction) parameter, plus two effects gated on
  `{ "param": "bloodline", "equals": "bloodline.magic_human" }`:
  `characteristic_score_delta_param` (+1) and `grants_reputation` (score 3,
  `kind` absent — "among others of his bloodline" fits no fixed
  `ReputationType`, matching Famous's player-chosen-type precedent). The
  pre-existing `aging_mod` effect (-1 Aging rolls, shared by all four
  bloodlines) is untouched; classification stays `in_play_effect` — D46's
  "classification follows what is computed, never where" does not
  distinguish which `Effect` family an item may mix. Magic Lore's
  authorization and the Animal/Spirit/Thing sub-type bonuses stay
  uncomputed, but their prose now lives in `description` in both locales,
  verbatim from the source — `uncomputed_clauses.rs`'s own
  `no_swept_entry_drops_an_uncomputed_mechanical_clause` screen is what
  requires that (a passage stating a mechanical rule needs the SAME in
  displayed text), and its passing is what let this entry's row leave
  `PENDING_DROPPED_CLAUSE` entirely rather than merely be reworded.
- **Gap 1 — `characteristic` must not be required for Magic Animal/Spirit/
  Thing.** `ParameterDef.required_if: Option<ParamGate>` (`types.rs`) — a
  declared parameter is only REQUIRED (raises `missing_param`) when the
  OWNING selection's own gate holds; `None` (the default) means
  unconditionally required, as before this field existed. Consumed by
  `validation/selections.rs::validate_selection_parameters`: the "expected"
  set (governs `unexpected_param`) is unchanged, but a SEPARATE "required"
  set (governs `missing_param`) drops a key whose `required_if` gate does not
  hold. `virtue.magical_blood`'s `characteristic` parameter carries
  `required_if: { "param": "bloodline", "equals": "bloodline.magic_human" }`
  — an older save with neither param reports `missing_param` for `bloodline`
  only; once `bloodline=magic_human` is chosen, `characteristic` becomes
  required in turn. Load-time integrity: the SAME `validate_param_gate`
  reused again, wired at the point-item call site. UI:
  `ParameterPicker.svelte` hides the control entirely while a `required_if`
  gate does not hold (`ui/src/lib/types.ts::ParamGate`, mirrored onto both
  `ParameterDef.required_if` and `Effect`'s two `gate` fields, which the
  Rust-side field additions had left unmirrored — corrected in the same
  slice).
- **Gap 2 — "but not above +3" (ArMDE:4367).** First cut wrongly inferred the
  rule from `gate.is_some()`: "gated ⇒ no must-already-be-at-cap precondition,
  and clamp to the base cap" ties Magic Human's WITHIN-the-cap shape to the
  mere presence of a gate, so a future GATED effect with Great-Characteristic
  semantics (raising ABOVE the cap) would silently inherit the wrong rule.
  Corrected (coordinator review, post-B4) to explicit data:
  **`CharacteristicDeltaCap`** (`types.rs`, closed 2-variant enum,
  `#[serde(default)]` on the `cap` field it names) — `AboveBase` (the
  default: every entry shipped before this field keeps its exact behavior,
  byte-identical, no `SCHEMA_VERSION` bump) is Great/Poor Characteristic's own
  shape (requires the bought score already at the cap/floor, result left
  uncapped); `WithinBase` is Magical Blood's Magic Human shape (no such
  precondition, contribution clamped instead). `gate` and `cap` are
  independent axes: `gate` governs WHETHER a delta applies at all;
  `cap` governs HOW its contribution is computed once it does — a hypothetical
  GATED `AboveBase` delta keeps the old precondition exactly like an ungated
  one, and an UNGATED `WithinBase` delta clamps with no gate at all.
  `validate_characteristic_delta_preconditions` (`scores.rs`) skips its
  precondition only on `cap: WithinBase` (not on `gate.is_some()` — the
  `gate` field was removed from `EffectTarget::CharacteristicParamDelta`
  entirely once nothing there read it any more); `characteristic_score_bonus`
  (`effective/characteristic.rs`) reads `cap` to choose between the raw
  `amount` and `capped_characteristic_delta_contribution`'s clamp:
  `(base_max - bought).clamp(0, amount)` for a positive `amount` (zero once
  bought is already at the cap), sign-mirrored against `base_min` for a
  negative one (no shipped entry uses that direction yet). Falls back to the
  raw `amount`, unclamped, when the ruleset carries no `characteristic_rules`
  at all (lean test fixtures). `virtue.magical_blood`'s
  `characteristic_score_delta_param` effect carries both `gate` and
  `cap: "within_base"`. Mirrored in `ui/src/lib/types.ts`'s `cap?:
  'above_base' | 'within_base'`, kept `effect-parity.test.ts` green.
- Tests: `crates/arm-rules/tests/b4_parameter_gated_effects.rs` — hand-authored
  fixture (gate held/not held, both variants) plus a shipped-data pair
  against the real `virtue.magical_blood` entry; two load-time-refusal tests
  (dangling gate param, `multi_ref` gate param) reusing `validate_param_gate`;
  a serde round-trip pinning that `gate: None`/default `cap` serializes
  byte-identically to the pre-B4 shape; gap 1's missing-param-scoping pair
  (Magic Animal never asked, Magic Human still asked); gap 2's cap-clamp pair
  (already-at-+3 does not overshoot, below-+3 does not misfire the
  Great-Characteristic-shaped precondition) PLUS the decoupling pair (a
  GATED `AboveBase` delta keeps the old precondition; an UNGATED `WithinBase`
  delta clamps) proving `cap` and `gate` are independent.
  `ui/src/lib/components/ParameterPicker.test.ts` — an SSR render trio for
  `required_if` (gate held shows the control, gate not held or the gate
  parameter unfilled hides it).

#### ≥1 Hermetic Flaw (magus guideline)
> You should take at least one Hermetic Flaw

- Source: `ArMDE:2860` — verbatim, a
  bullet under `#### Magi` (`ArMDE:2853`), hence no closing period.
- A "should", so a **soft warning** — `validation/magus.rs::validate_house` (:33) emits
  `missing_hermetic_flaw` when a magus has no selected Flaw counting as Hermetic.
- "Counting as Hermetic" is two data lookups, never a hardcoded `"hermetic"`:
  the profile's **`hermetic_flaw_categories`** (`["hermetic"]` on the magus
  profile only) met against the item's `categories` **or** its
  `index_categories`. It is deliberately NOT `gift_categories` and NOT the
  taken-as-narrowed `categories_for` — see *Resolved (row 18)* in the Hermetic
  category section for why all three distinctions are load-bearing.

#### `validate_house` — specialisation resolution
- `house_choice_unresolved` (error): a `Choice` pick missing or not among its
  `options`, or an `Open` pick missing.
- `house_grant_constraint` (error): an `Open` pick violating its `GrantConstraint`
  (kind / magnitude / require- / forbid-categories, all read from data), or
  demanding a different House than the character's (see "The four Outer-Mystery
  Virtues confer House membership" below).
- `house_unset` (warning): a magus with no House chosen.

#### House-granted Virtue/Ability definitions
The named Virtues a House grants, and the Mystery Abilities their
`ability_score_grant` seeds at score 1, are all defined in the Core Rules
(this edition folds the House virtues into the core V/F list). Each new
`rules/core/virtues_flaws.json` item and `rules/core/abilities.json` entry cites:

| Item | Kind | Source (Ars Magica - Definitive Edition (Core Rules).md) |
|------|------|------------------------|
| `virtue.the_enigma` (grants `ability.enigmatic_wisdom` 1) | Minor, Hermetic | `ArMDE:3759-3761` |
| `virtue.faerie_magic` (grants `ability.faerie_magic` 1) | Minor, Hermetic | `ArMDE:3825-3827` |
| `virtue.heartbeast` (grants `ability.heartbeast` 1) | Minor, Hermetic | `ArMDE:4059-4061` |
| `virtue.verditius_magic` (no creation-number effect) | Minor, Hermetic | `ArMDE:5215-5217` |
| `virtue.hermetic_prestige` (Reputation → M5, no creation effect) | Minor, Hermetic | `ArMDE:4071-4073` |
| `virtue.minor_magical_focus` (in-play casting, no creation effect) | Minor, Hermetic | `ArMDE:4536-4542` |
| `virtue.self_confident` (Confidence → M5, no creation effect) | Minor, General | `ArMDE:4900-4902` |
| `ability.intrigue` | General | `ArMDE:7590-7592` |
| `ability.enigmatic_wisdom` (requires_training) | Arcane | `ArMDE:7454-7455` |
| `ability.faerie_magic` (requires_training) | Arcane | `ArMDE:7478-7479` |
| `ability.heartbeast` (requires_training) | Arcane | `ArMDE:7501-7502` |

#### Merinita's conditional Warping Point
> "Any magus in this House without a faerie-related Virtue or Flaw has a
> Warping Point, inflicted to allow initiation into the Mystery."

- Source: `ArMDE:2280` — the second sentence of Merinita's own cell in the
  House benefit table (above), distinct from the Faerie Magic grant the first
  sentence states.
- D81.4/D81.14 (`docs/vf-audit/decisions.md`): a conditional point, not a
  `Grant` — applies only to a magus of a House carrying a
  `conditional_warping` clause (Merinita only, today) who holds no
  "faerie-related" Virtue or Flaw. D81.14 rules "faerie-related" as EITHER an
  entry whose realm resolves to Faerie via the existing realm-resolution chain
  (Faerie Blood, Strong Faerie Blood, and Bound to (Realm) / Realm Stigmatic /
  Necessary Realm Aura for Ability / Folk Magic when their own `realm`
  parameter names Faerie), OR an explicit `faerie_related: true` data flag on
  three entries outside the realm system entirely: Faerie Friend
  (`ArMDE:6052-6054`), Faerie Upbringing (`ArMDE:6056-6058`), Susceptibility to
  Faerie Power (`ArMDE:6819-6821`). `virtue.faerie_magic` itself — the first
  sentence's grant, which every Merinita magus holds by construction — carries
  neither, so the clause is never vacuous.
- Implementation: `effective/warping.rs::has_faerie_related_vf` (the
  predicate, scanning bought ∪ non-warping-fill granted selections — the same
  base `warping_points_for_owed` uses, so a player cannot fill an owed-warping
  Flaw slot with a faerie-related pick to dodge this very point) and
  `effective/warping.rs::house_conditional_warping_points` (the data-driven
  gate: reads `entity`'s House's own `conditional_warping` field — `points`
  unless `unless` holds, 0 for a House with no such field at all), folded
  additively into both `warping_points_total` and `warping_points_for_owed`
  via `saturating_add` — a genuine accrued Warping Point, not a Merinita-only
  side channel. Generic over House: `house_conditional_warping_points` names
  no House id, so a second House's analogous Mystery clause is a
  `rules/core/houses.json` change (architecture review finding, 2026-10-02 —
  the original implementation hardcoded `house.merinita` by id).
- Data: `house.rs::House::conditional_warping` (`ConditionalWarping { points,
  unless: WarpingExemption }`); `house.merinita`'s entry in
  `rules/core/houses.json` sets `{ "points": 1, "unless": "faerie_related_vf" }`.
  `WarpingExemption` is a closed enum naming the one predicate shape the
  rulebook states today (YAGNI) — a future House with a differently-shaped
  exemption adds a variant, not a branch. `rules/core/virtues_flaws.json` —
  `faerie_related: true` added to `flaw.faerie_friend`,
  `flaw.faerie_upbringing`, `flaw.susceptibility_to_faerie_power`. No new
  field on `virtue.faerie_magic` or `virtue.faerie_doctor` (mythic-companion-
  only category, unreachable by any Hermetic magus profile) — see D81.14.
- Tests: `crates/arm-rules/tests/d81_merinita_warping.rs` — the no-faerie-V/F
  case (1 point), the House gate (non-Merinita owes nothing), additive
  stacking with pre-existing accrued points, the two realm-resolved and three
  explicitly-flagged exemptions, and the Bound-to-(Magic)-realm forward guard
  (does NOT exempt).

#### The four Outer-Mystery Virtues confer House membership
> "You have been initiated into the Outer Mystery of the Heartbeast (see page
> 233), **and thus are a member of House Bjornaer**. You start with the Ability
> Heartbeast 1."

- Source: `ArMDE:4059-4061` (Heartbeast
  → Bjornaer), `ArMDE:3759-3761` (The Enigma → Criamon), `ArMDE:3825-3827` (Faerie Magic →
  Merinita), `ArMDE:5215-5217` (Verditius Magic → Verditius). These are the **only**
  four V/F descriptors in the core rules carrying the "and thus are a member of
  House X" clause; the other eight Houses' free Virtues make no such claim and
  carry no House prerequisite (`virtue.hermetic_prestige`,
  `virtue.self_confident`, `virtue.minor_magical_focus`, the two Puissants).
- Data: `rules/core/virtues_flaws.json` — each of the four carries
  `"prerequisites": { "kind": "house", "value": "house.<x>" }`. The House ids come
  from `rules/core/houses.json`; load-time integrity
  (`ruleset/integrity.rs::validate_prereq_refs`) rejects an unresolvable one, so
  the shipped catalogue can only be loaded alongside the House registry.
- Implementation, bought rows: nothing new — `validation/prereq.rs` already
  evaluates `Prereq::House` against `Entity::house`. A magus of another House who
  *buys* one gets `prereq_not_met`; a magus with **no House chosen yet** gets the
  `prereq_unevaluated` warning, because an absent House is genuinely Unknown, not
  a failure. The House's own *granted* row is never prereq-checked
  (`validate_prerequisites` walks `entity.selections` only) and would satisfy the
  prerequisite anyway, so Bjornaer's free Heartbeast stays legal.
- Implementation, **open grants**: `grant.rs::open_pick_satisfies` also refuses a
  pick whose prerequisite `Prereq::conflicts_with_house` rules out for the
  character's own House. Without it, Jerbiton's `jerbiton_minor_virtue` and Ex
  Miscellanea's `ex_misc_minor_virtue` — both open menus for a Minor (Hermetic)
  Virtue, which all four of these are — would offer them, and a grant pick is
  never prerequisite-checked, so a Jerbiton could hold a Heartbeast with no
  complaint at all. Only `House` leaves filter: a `Has`/`AbilityMin`/… prerequisite
  stays on the menu, since those resolve as the build progresses and excluding
  them would be an order-dependent exclusion rather than the error-that-resolves
  model used everywhere else. `ui/src/lib/derive.ts::eligibleForConstraint`
  mirrors the same predicate (`houseOnlyValue`) so the picker and the validator
  agree; `house` is a required argument there so no picker can silently omit it.

All four Mystery Virtues (The Enigma, Faerie Magic, Heartbeast, Verditius Magic)
are **Minor** — the free House Virtue is Minor per `ArMDE:2859`, so none of them can
trip the ≤1-Major-Hermetic-Virtue cap even when granted. Tremere's own Magical
Focus field is book-given, not freeform: "Minor Magical Focus(certamen)\*"
(`ArMDE:2064`, the sample Tremere magus's Virtues-and-Flaws line) and the House table's "Minor
Magical Focus (certamen)." (`ArMDE:2281`) both name it, so House Tremere's grant
carries `"params": { "focus": "certamen" }` on its
`fixed virtue.minor_magical_focus` (X10(a), D65 row N5). Ex Miscellanea's Minor
Hermetic Virtue is player-chosen (`open`), not a fixed item, so there is no
invented `virtue.ex_misc_minor_hermetic`.

---

## Mythic Companion types (M4/4d)

A Mythic Companion picks a **type** (Devil Child, Faerie Doctor, Nephilim, Spirit
Votary) that grants a free "status" Virtue **plus** a free Minor Virtue (both
point-free grants, via the shared `grant.rs` model), imposes a required V/F
package that counts against the budget normally. It never changes the *size* of
the budget. Data: `rules/core/mythic_companion_types.json`
+ `rules/i18n/<lang>/mythic_companion_types.json`; engine `mythic_companion.rs`
(`MythicCompanionType`, `RequiredFlaw`); validation `validate_mythic_type`, with
the budget itself left to `validate_balance`. The general rules are Core
Rules.md `ArMDE:2635-2639`; the V/F guidelines `ArMDE:2842-2851`.

#### General mechanism & per-type budgets
> `ArMDE:2637` "All Mythic Companions take a Free Virtue which specifies their status.
> These Virtues are incompatible with each other, and with The Gift, and are not
> available to grogs." `ArMDE:2638` "You gain a free Minor Virtue… you may take up to
> ten points of Flaws, and each point of Flaws is worth two points of Virtues…
> Most Mythic Companion Virtues require you to take some particular Virtues and
> Flaws, these count against your maximum."

The four status Virtues carry a symmetric `incompatible_with` web (each other +
`virtue.the_gift`). The free status Virtue's category is `mythic_companion` — the
index heading these four are listed under (`ArMDE:3329-3334`), and one the
`mythic_companion` profile alone permits, which is how "not available to grogs"
(`ArMDE:2637`) is enforced; the free Minor and required Virtues keep their own book
categories. The balance ceilings come from the type profile alone —
`EffectiveBudget` (`validation/balance.rs`, :86): `flaw_ceiling = flaw_points`,
`virtue_ceiling = virtue_points`, `funded = flaw·rate` (rate = 2), i.e. 10 F / 20 V
for every Mythic Companion. A type carries no bonus points of any kind, so the
check is the same one every other character type gets.

| Type | free status | free Minor | required Virtues (budgeted) | required Flaw (default) | bonus | Source |
|------|-------------|-----------|------------------------------|--------------------------|-------|--------|
| Devil Child | `virtue.devil_child` | Demonic Might **or** Powers | Demonic Blood, Puissant (Guile) | Tragic Life | **none** | `ArMDE:2643-2666`, `RoP:I:4908-4924` |
| Faerie Doctor | `virtue.faerie_doctor` | Dowsing | Wise One, Curse-Throwing | Faerie Friend, Dutybound | **none** | `ArMDE:2668-2704` |
| Nephilim | `virtue.nephilim` | Strong Angelic Heritage | Blood of the Nephilim, Greater Immunity, Great Sta, Great Str, Improved Characteristics, Sense Holiness | — (5 F fund the +10 V) | **none** | `ArMDE:2714-2739` |
| Spirit Votary | `virtue.spirit_votary` | Second Sight | Spiritual Pact (+ 1 Major/3 Minor Supernatural, **advisory**) | Pagan | **none** | `ArMDE:2741-2764` + `ArMDE:2638` |

**No type carries a budget bonus. Every Mythic Companion gets 10 Flaw points and
20 budgeted Virtue points, plus one uncharged free Minor Virtue — the "21 points
of Virtues for 10 points of Flaws" of `ArMDE:2638` and `RoP:I:4912`.** A type's
compulsory package changes how that allowance is *spent*, never its size. **This
is settled and closed: see `docs/vf-audit/decisions.md` D32, which lists the five
readings that have been tried and rejected. Do not re-derive it.**

The two sentences that look like bonuses are not:

- `ArMDE:2664` / `RoP:I:4924` — "three more points of Virtues at no cost … and an
  additional seven points of Flaws" (Devil Child). `RoP:I:4912` states the
  21-for-10 maximum **twelve lines above this sentence, in the same book, about
  the same type**, so neither number extends the cap; they describe how it is
  reached (the compulsory Tragic Life is 3 of the 10, leaving 7). The book's own
  worked Devil Child, Malachi at `RoP:I:4942`, carries **exactly 10** points of
  Flaws.
- Spirit Votary's **7** is the *unspent remainder* of the same ten: its required
  Virtues cost **6** budgeted points — Spiritual Pact (Major, 3) plus "either one
  more Major Supernatural Virtue or three Minor Supernatural Virtues" (3) at
  `ArMDE:2750-2751` — and its required Flaw, Pagan (Major, 3) at `ArMDE:2756`,
  funds exactly those 6 at the 2:1 rate, leaving `10 − 3 = 7`. A remainder inside
  the ten; adding it to the ten counts it twice. *Realms of Power: Magic*
  `RoP:M:5486` states the same 7 and means the same thing.

**Withdrawn.** This section previously read *"Verified maxed budget: flaw 17,
virtue 20 + 14 + 3 = 37"*. No such decision was ever taken — it was a misreading
recorded as a ruling, it shipped as `bonus_flaw_points: 7` on two types, and it
contradicted the Spirit Votary derivation printed directly beneath it.
- **Nephilim** has no bonus: its "5 points of Flaws to pay for these virtues"
  (`ArMDE:2731`) is a consequence of funding 10 pts of required Virtues at 2:1 within
  the base 10 F / 20 V, not a budget change.
- The `20` and the `21` are **not an inconsistency** — they are two different
  quantities in one sentence. `ArMDE:2638` (and `RoP:I:4912`) gives `21` as the
  **total** points of Virtues: `20` budgeted (10 Flaw points × the 2:1 rate) plus
  the **free Minor Virtue**, which is a point-free grant that never consumes
  budget (same treatment as the free House Virtue). `ArMDE:2844`'s `20` is the
  budgeted figure alone. The ceiling the engine enforces is `20`. This section
  previously called it an inconsistency and "resolved" it; that framing was wrong
  (D32, rejected reading 3).
- **Required-package enforcement is advisory** (`mythic_required_trait_missing`
  warning): the rules permit "a suitable substitute agreed with the troupe" for
  the required Flaws (`ArMDE:2660`, `ArMDE:2689`, `ArMDE:2754`), so a missing/substituted slot
  never hard-blocks. Required Virtues are matched by full `Selection` (ref +
  params), so parameterized/duplicated requirements are exact — Nephilim's two
  distinct Great Characteristics (`great_characteristic` with
  `characteristic.sta` / `.str`) and Puissant Guile (`puissant_ability` +
  `ability.guile`) — an approximation matching only the seeded default target.
- Spirit Votary's open "1 Major OR 3 Minor Supernatural" requirement is left
  **advisory** (unenforced) for M4 — a candidate for a future "minimum category
  points" rule (M5).

#### The type is mandatory, and the only source of its status Virtue (D83.7)
> `ArMDE:2846` "- You must take the Free Virtue defining which type of Mythic
> Companion you are"
>
> `ArMDE:2637` "- All Mythic Companions take a Free Virtue which specifies their
> status. These Virtues are incompatible with each other, and with The Gift, and are
> not available to grogs."

- **Unset type → error.** `validate_mythic_type` (`validation/magus.rs`) reports
  `mythic_type_unset` as an **error** under the `mythic_type` phase. It was a warning
  before, and no reason for that was ever recorded. The book says "must". The guided
  wizard gates only on errors in the *current* step. The Type step is the second step
  of the `mythic_companion` profile's `creation_phases` and is unconditional, so the
  error blocks only leaving that step, where the type is chosen.
- **Status Virtues are grant-only** (Norbert, 2026-10-03: "Devil Child and all the
  other Vs for myth comps should only be granted by selecting the appropriate mythic
  companion type. not selectable by just anyone."). The four status Virtues carry
  `"mythic_status": true` in `rules/core/virtues_flaws.json` (`PointItem::mythic_status`).
  `validate_mythic_status_not_bought` (`validation/magus.rs`) reports every bought copy
  (one in `entity.selections`) as the error `mythic_status_virtue_bought`
  (`virtues_flaws` phase), for every character type. The copy the type grants is the
  legal one. The V/F picker (`VirtueFlawTab.svelte`) greys the row out in every
  validation mode, with the reason `vf-blocked-mythic-status`.
- The types' **other** free Virtues (Dowsing, Strong Angelic Heritage, Second Sight)
  are not flagged and stay buyable.
- Load-time integrity (`ruleset/integrity.rs::validate_mythic_status_granted`): with
  mythic types loaded, every `mythic_status` item must be some type's `fixed` grant,
  or it could never be held. `tests/r6_mythic_status_virtue.rs` also checks the
  shipped data: the flag marks exactly the `mythic_companion` category, and each type
  grants exactly one flagged Virtue.

#### New V/F & Ability definitions (structural; core effects M5, supplement effects M9)
Per CLAUDE.md each is sourced from the authoritative Markdown by book. The
supernatural **Might** effects (Infernal/Divine Might + power levels) for Demonic
Blood/Might/Powers and Strong Angelic Heritage are now **wired** (see Supernatural
Might & Magic Resistance below); the core in-play details (e.g. the Greater
Immunity target) are M5/5b.

> **Provenance correction.** Eleven of these Virtues — Blood of the Nephilim,
> Curse-Throwing, Demonic Blood, Demonic Might, Demonic Powers, Devil Child,
> Faerie Doctor, Nephilim, Spirit Votary, Spiritual Pact, Strong Angelic Heritage —
> were originally cited to a *Realms of Power* volume, where they are indeed also
> printed. But every one of them is **reprinted in the core rules**, in the Virtues
> and Flaws chapter, and English core is the source of truth for IDs and
> provenance. Their `source` in `rules/core/virtues_flaws.json` and every citation
> below now names *Ars Magica - Definitive Edition (Core Rules).md*, bracketing the
> real core entry (heading line through the line before the next heading, the
> convention used throughout that file). Locked by
> `core_rules_virtues_cite_the_core_rules_file` in
> `crates/arm-rules/tests/data_integrity.rs`. Note that **Curse-Throwing appears
> twice** in the core rules: the Virtue at `ArMDE:3625` (*Major, Supernatural*) and the
> Supernatural **Ability** at `ArMDE:7396`; `virtue.curse_throwing` cites the former,
> `ability.curse_throwing` the latter. The *Realms of Power* citations that remain
> in this section carry no mechanic the core rules lack: `RoP:I:4908-4924` states
> the Devil Child's "21 points of Virtues for 10 points of Flaws" maximum that
> `ArMDE:2638` already gives generally, and `RoP:M:5486` restates the 7 that falls
> out of it. Neither is a budget bonus (see above, and D32).

| Item | Kind | Source |
|------|------|--------|
| `virtue.devil_child` | Special/Free, Social Status | `ArMDE:3671-3674` |
| `virtue.demonic_blood` | Major, Supernatural (Tainted) | `ArMDE:3649-3662` |
| `virtue.demonic_might` (req. Demonic Blood) | Minor, Supernatural | `ArMDE:3663-3666` |
| `virtue.demonic_powers` (req. Demonic Blood) | Minor, Supernatural | `ArMDE:3667-3670` |
| `flaw.tragic_life` | **Major, Story** (Tainted) | `ArMDE:6855-6870` |
| `virtue.faerie_doctor` | Special/Free, Social Status | `ArMDE:3821-3824` |
| `virtue.dowsing` (grants `ability.dowsing` 1) | Minor, Supernatural | `ArMDE:3703-3706` |
| `virtue.curse_throwing` (grants `ability.curse_throwing` 1) | Major, Supernatural | `ArMDE:3625-3628` |
| `virtue.wise_one` | Minor, Social Status | `ArMDE:5257-5260` |
| `flaw.faerie_friend` | Minor, Story | `ArMDE:6052-6055` |
| `flaw.dutybound` | Minor, Personality | `ArMDE:5992-5995` |
| `virtue.nephilim` | Free, Social Status | `ArMDE:4594-4597` |
| `virtue.blood_of_the_nephilim` | Major, Supernatural | `ArMDE:3504-3518` |
| `virtue.strong_angelic_heritage` (req. Blood of the Nephilim) | Minor, Supernatural | `ArMDE:5022-5031` |
| `virtue.greater_immunity` (carries a `hazard` text param since Q4b/D9 part 1; "Disease" is an in-play target) | Major, Supernatural | `ArMDE:4009-4016` |
| `virtue.sense_holiness_and_unholiness` (grants `ability.sense_holiness_and_unholiness` 1) | Minor, Supernatural | `ArMDE:4926-4929` |
| `virtue.spirit_votary` | Free, Supernatural (→ Social Status) | `ArMDE:5006-5009` |
| `virtue.spiritual_pact` | Major, Supernatural | `ArMDE:5010-5021` |
| `flaw.pagan` (Major or Minor; seeded Major) | Major, Personality | `ArMDE:6570-6573` |
| `ability.curse_throwing` (requires_training) | Supernatural | `ArMDE:7396-7430` |
| `ability.dowsing` (requires_training) | Supernatural | `ArMDE:7439-7442` |
| `ability.sense_holiness_and_unholiness` (requires_training) | Supernatural | `ArMDE:7720-7723` |

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
(`validation/magus.rs`) enforces two sourced constraints, both gated on
`is_hermetically_trained` (D56/A0 — not just a real magus, but any entity whose
selections confer training, e.g. the Abandoned Apprentice Flaw).

**Spell-levels budget — 120 at creation.**

> `ArMDE:2215-2216` "**Hermetic Magi Only: Apprenticeship.** … Take 120 levels of
> spells, of no higher level than Technique + Form + Intelligence + Magic Theory
> +3."
> `ArMDE:2435` "The fifteen years of apprenticeship give the character 240 experience
> points, and 120 levels of spells."

Encoded as `EntityTypeProfile.spell_levels: 120` on the magus profile
(`rules/core/character_types.json`; JSON has no comments, so the value's
provenance lives here). The sum of the chosen spells' levels must not exceed the
effective budget → `over_spell_levels`. Value lives in data; the engine never
hardcodes 120.

`ArMDE:2435`'s **other** number — the 240 experience points of the same sentence — lives in
`rules/core/life_stages.json` instead, on the apprenticeship block: see
**Apprenticeship — 240 experience points across Arts and Abilities (M6/6b4)** in the
*Life-stage experience* section below, which records why the two halves of one
sentence are deliberately split across two files. `ApprenticeshipRules` carries no
`spell_levels` field, so this profile value stays the single source.

The base budget is selected in one place, `effective::spell_levels_base` — the
per-character `Entity::spell_levels_override` (an optional stored *choice*, an
app affordance, not a rulebook mechanic) when set, otherwise the profile's
`spell_levels`. Both the effective payload (`arm-app/src/effective_dto.rs`) and the
`over_spell_levels` validator (`validation/magus.rs`) call it, so the displayed
and validated budgets cannot diverge. `spell_levels_budget` then adds the
Skilled/Weak Parens `Effect::SpellLevels` modifiers on top of that base — and, for
a magus generated some years out of apprenticeship, the levels it took out of its
post-Gauntlet points (`ArMDE:2471`): see **Life as a magus after the Gauntlet — 30 points
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
deliberate and was searched for while reviewing: `ArMDE:2215` grants the levels ("Take 120
levels of spells") and **no passage anywhere in `rules/source/en/` states that
unlearned levels are lost, wasted or forfeit.** Its sibling
`restricted_xp_unspent` *may* say "will be wasted" because a restricted pool is
earmarked to a named list and buys nothing outside it — the reading recorded under
**Early childhood** below, off `ArMDE:2378` — but that backing does not extend here.
The **absence** of a waste rule is the whole reason for the bare count — so a later
pass that "improves" the copy into "will be wasted" would be inventing a rule, which
`CLAUDE.md`'s provenance rule prohibits. `ui/src/lib/i18n.test.ts` pins it in both
locales.

**The base is offered for editing in direct entry only (Slice 8 / #19).** `ArMDE:2215`'s
"Take 120 levels of spells" is a flat grant with no in-rules variation, and every
variation the rules *do* allow already reaches the budget elsewhere — Skilled/Weak
Parens as `Effect::SpellLevels`, and the post-Gauntlet split as `ArMDE:2471`'s levels. So
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

**Per-spell cap — Technique + Form + Intelligence + Magic Theory + 3 + flat `lab_total_mod` (D1), with requisites folded, the Magical Focus doubling (X11b, D81.5) and Potent Magic only within its marked field (D83.3).**

> `ArMDE:2465` "The highest level spell you can learn is equal to Technique + Form +
> Intelligence + Magic Theory +3 … If the spell has requisites … they apply to
> this total as well. This is the appropriate Lab Total, assuming an aura modifier
> of +3, and thus any Virtues and Flaws your character has apply to this total if
> they would apply to a Lab Total in play."

`spell_level_cap(entity, ruleset, technique, form, requisites, range_beyond_touch,
marks)` (`effective/spell.rs`; `marks: SpellMarks` carries the spell's
`within_focus` / `within_potent_field` markers) computes it from the effective Art scores
(folded against `requisites` — below), the **effective** Intelligence (after
aging drops and free deltas such as Great (Intelligence),
`effective_characteristic_after_aging` — the same reader the Lab Total uses, since
`ArMDE:2465` calls the cap "the appropriate Lab Total"; tests
`tests/spell_cap_effective_intelligence.rs`), and
effective Magic Theory → a spell above it emits `spell_level_exceeds_cap`
(validation, in `validation/magus.rs`). The grid function `spell_level_caps`
→ `EffectiveScores.spell_level_caps` (`effective_dto.rs`) has no specific spell
at a Te/Fo cell (it may host several, with different or no requisites), so it
calls the same function with `requisites: &[]`, `within_focus: false`,
`within_potent_field: false` — a no-op fold, unchanged from before X11b.

**X11b/D81.5 closes the approximation the previous paragraph used to record
here.** `ArMDE:2465`'s own second sentence — "If the spell has requisites …
they apply to this total as well" — and `ArMDE:12313`'s "Requisites listed
with a spell's statistics apply when you are learning, inventing, or casting
that spell" put learning (this cap) on the same footing as casting, which
`derived/casting.rs::fold_requisite` already folds (X11, `b5ee82a`). That exact
function is **reused, not duplicated**: `spell_level_cap` calls
`crate::derived::casting::fold_requisite` directly (made `pub(crate)` for this),
so a Puissant Art bonus (ArMDE:4820), several same-class requisites folding to
their group's lowest (ArMDE:12311), and the Elemental Magic exception
(ArMDE:3737 — "you use the primary Form to calculate totals, even if the
requisite is lower"; generic "totals" wording, so it governs this Lab-Total-
shaped cap too, per :2465's own closing sentence calling the cap itself "the
appropriate Lab Total") all behave identically on both totals. A requisite
Art that is itself Deficient halves the cap even when it does not numerically
bind the fold (ArMDE:12311's closing sentence) — the same
[`deficient_arts`](`effective/art.rs`) fold both totals already shared is
simply also checked against `requisites`, so the two can never disagree about
which Arts are deficient.

**Magical Focus doubling (ArMDE:4403).** "If a spell has requisites, the
lowest applicable score may be one of the requisites, rather than one of the
primary Arts" — `within_focus` adds the lower of the two *already-folded*
scores to the cap before either halving, so a requisite that won the fold is
automatically eligible with no separate case. `within_focus` is the caller's
own claim (`SpellSelection::within_focus`, X10c) — the engine cannot match a
free-text Magical Focus theme to a spell (MAG8), so the player decides, and
`validate_spell_level_cap` reads `sel.within_focus` back, the same way
`spell_casting_total` already reads it for the Casting Total.

**Correction (review finding, `tmp/review-d81.json` #1): the doubling is now
also gated on actually holding a Magical Focus right now.** The marker alone
used to be enough — `spell_level_cap` applied the doubling for any
`within_focus: true`, with no check that the entity still held a Magical
Focus Virtue. That made the gate on the *cap* strictly weaker than the gate
the picker uses to offer the "add within focus" action in the first place
(`spell_caps`'s own `has_magical_focus` check, above), so removing the Focus
Virtue after marking a spell left an illegally over-cap spell with no
`spell_level_exceeds_cap` error — reachable through plain Virtue removal in
the normal UI, not only a hand-edited save. `spell_level_cap` now reads
`within_focus && has_magical_focus(entity, ruleset)` — the SAME predicate
`spell_caps` already calls, so the enforcement path and the picker's own
gate can never disagree again. The marker itself is never scrubbed (saves
store choices); only its *effect* here is conditional on the Virtue still
being held. `derived/casting.rs::spell_casting_total` gets the identical
gate, against its own already-computed `InPlayMods::has_focus`, for the same
reason — the Casting Total a known spell shows was inflating the same way.
Tests: `crates/arm-rules/tests/focus_marker_needs_focus.rs`.

**D81.17 (`docs/vf-audit/decisions.md`): the now-inert marker also WARNS.**
The correction above leaves the cap/Casting-Total figures right but gives the
player no visible sign that their `within_focus` marker stopped doing
anything once the Magical Focus Virtue was removed. `validate_spells`
(`validation/magus.rs`) calls a new `validate_spell_focus_marker_without_focus`
for every known spell: `sel.within_focus && !has_magical_focus(entity,
ruleset)` raises a non-blocking warning (`CODE_SPELL_WITHIN_FOCUS_WITHOUT_MAGICAL_FOCUS`,
`"spell_within_focus_without_magical_focus"`, `args` carries `spell`), reading
`effective::has_magical_focus` — made `pub(crate)` for this — the SAME
predicate `spell_level_cap`/`spell_casting_total` already gate the doubling
on, so the warning can never disagree with which spells the marker actually
affects. The mark stays saved; only the player's visibility into its
inertness changes. Tests: `crates/arm-rules/tests/focus_marker_needs_focus.rs`.

**The in-play Lab Total (`derived/lab.rs::lab_totals`) still does not fold
requisites.** `ArMDE:12313` separates "learning" (this cap, now folded) from
"inventing" (the in-play Lab Total family, still unfolded) — a genuinely
separate change (Lab Text similarity bonuses, arcane experimentation,
ArMDE:10746's "lowest of several simultaneous activities" rule), not started
by this slice.

**DTO: `EffectiveScores.spell_caps` (new, X11b).** The Te/Fo-keyed grid
(`spell_level_caps`/`SpellLevelCap`) cannot represent a per-spell fold: two
spells sharing a Te/Fo pair but different requisites can now genuinely report
different caps, the same reason `spell_casting_total` exists beside
`casting_totals`. `spell_caps(entity, ruleset)` (`effective/spell.rs`) returns
one `SpellCap { spell, cap, within_focus_cap }` row per **catalogue** spell
(not just known ones, so the picker can grey an unlearned spell too):
`within_focus_cap` is `Some` only when the entity holds a Magical Focus at
all (`has_magical_focus`), independent of whether any spell is marked
`within_focus` yet. Both grids are kept — the Te/Fo grid still backs
`SpellTab.svelte`'s cross-Te/Fo matrix display, while the picker's per-spell
grey/offer logic should read `spell_caps` for any spell carrying requisites.
Tests: `crates/arm-rules/tests/requisite_level_cap.rs` (engine fold + focus),
`crates/arm-app/tests/spell_caps_dto.rs` (DTO row).

**Casting Total — requisite folding (X11, `derived/casting.rs::fold_requisite`).**

> `ArMDE:12309` "You must use the lesser of your score in the requisite and
> your score in the spell's main Technique or Form — Technique if the
> requisite is a Technique, Form if the requisite is a Form."

> `ArMDE:12311` "Sometimes a spell has a requisite for both its Technique and
> Form. You must use the lowest in each case. And, if several requisites
> apply to the same primary Art... your effective score is the lowest of the
> group. Furthermore, any Deficiencies you have with an Art apply when you
> use that Art as a requisite."

**Reused by the per-spell cap (X11b, above), not duplicated:** `fold_requisite` is
`pub(crate)` so `effective/spell.rs::spell_level_cap` calls it directly.

`fold_requisite` reduces the effective Technique/Form score `formulaic_casting_score`
uses to the lowest of the primary and every same-class requisite — bonus-inclusive
on every side (`effective_art_score`), so Puissant Art (`ArMDE:4820`) needs no
special case. `InPlayMods::deficient` additionally checks the spell's
requisites, not only its primary Technique/Form, per `ArMDE:12311`'s closing
sentence. `Effect::ElementalMagic`'s pooled Forms (`InPlayMods::elemental_forms`)
exempt a Form requisite from the fold when both it and the primary Form are
elemental (`ArMDE:3737`), so a magus with Elemental Magic always uses the
primary elemental Form. Reached by `spell_casting_total` and `penetration`
(both resolve a concrete spell); the Technique×Form grid (`casting_totals`)
passes no requisites at all, since one cell may host several spells with
different or no requisites of their own. Deliberately does **not** reach Magic
Resistance, vis-boosting, or any other "effects that affect spells based on
their Arts" (`ArMDE:12313`) — those read the primary Arts only and call neither
`fold_requisite` nor `formulaic_casting_score`. Tests:
`crates/arm-rules/tests/requisite_casting_total.rs`; resolves `docs/book-template-
conformance.md`'s MAG4 (was "(c) the book", now "(a) the engine, fixed") and
updates MAG7's own best-reachable figure (still "(c) the book").

**Stat-line guard (`tests/spell_stat_line.rs`).** Every entry in
`rules/core/spells.json` is cross-checked against the Technique, Form, Level,
Range, Duration, Target, Ritual and requisites the book actually states at the
entry's own cited `source.lines`. The guard reads a `Reg:` line the same as a
`Req:` one: the book misspells "Req:" as "Reg:" in four places — `ArMDE:13222`
(Rain of Stones), `ArMDE:14016` (inline, no separating comma — Thaumaturgical
Transformation of Plants to Iron), `ArMDE:15448` (Obliteration of the Metallic
Barrier) and `ArMDE:15545` (Hands of the Grasping Earth) — and the data's
extraction pass had evidently looked only for `Req:`, so all four requisites
were missing until this guard caught them (spell-guard Phase 1/2, 2026-10-02).
One entry, `spell.piercing_the_magical_veil`, is a deliberate exception to the
guard rather than a passing comparison: its citation (`ArMDE:1745`) is a
sample character's spell-list bullet, not a stat block, so its level/R/D/T
are an acknowledged inference (D68 item 6) rather than a parseable passage —
see `RULED_EXCEPTIONS` in the test file.

**D1 (`docs/vf-audit/decisions.md`): every flat `LabTotalMod` effect (Inventive
Genius +3, Creative Block −3, Weak Scholar −6, Adept Laboratory Student +6,
Aristotelian Training +1, Cyclic Magic ±3, Potent Magic Major/Minor +3/+6 —
nine entries total) applies to this cap, its own book condition deliberately
ignored.** (Potent Magic since D83.3 only for a spell marked within its field —
see below.) Every one of the nine is individually conditional in the source text
(a Laboratory Text, a season, a chosen focus, …); D4 resolves those conditions
for the *in-play* Lab Total (`derived/lab.rs::lab_totals`), but this cap does
not model lab situations at all — it is only a ceiling on which spells may be
*chosen*, never a number printed as a play result, so the generous
condition-free reading is acceptable here **and here only** (D1's ruling was
withdrawn once and re-taken on this corrected, unconditional-everywhere
reading — do not re-introduce a conditional subset). `effective/spell.rs::lab_total_mod`
sums the flat total; `derived.rs::in_play_mods` reuses the identical function
for its own `lab_mod` addend, so the two can never compute this sum
differently. Test: `lab_total_mod_applies_flat_to_the_spell_level_cap`
(`effective.rs`).

**D83.3 amends D1 for Potent Magic: its bonus counts only for a spell marked
within its field** (Norbert, after-deadline answer 3).

> `ArMDE:4742` "The maga's magic is particularly attuned to a narrow field, much as
> in a Magical Focus. The benefits of Potent Magic are compatible with a Magical
> Focus, unlike a Magical Focus, unlike a Magical Focus, a maga may have more than
> one area of Potent Magic, although only one Potent Magic Virtue applies to any
> single activity."
>
> `ArMDE:4744` "Potent Magic provides the maga with a bonus in her field of magic,
> and permits her to devise Potent spells that gain a casting bonus from the
> sympathetic magic in shapes and materials. Potent Magic can be taught as an
> alternative to a Magical Focus."
>
> `ArMDE:4746` "Minor Potent Magic covers the same narrow fields as a Minor Magical
> Focus, and grants a +3 bonus to Lab Totals and Casting Score."
>
> `ArMDE:4748` "Major Potent Magic covers the same wide fields as a Major Magical
> Focus, and grants a +6 bonus to Lab Totals and Casting Score."
>
> (The doubled "unlike a Magical Focus" is in the source file as is.)

`effective/spell.rs::lab_total_mod` now leaves out every carrier scoped
`within_potent_field_only` (data: `virtue.potent_magic_major` +6,
`virtue.potent_magic_minor` +3 in `rules/core/virtues_flaws.json`). Those carriers
enter `spell_level_cap` only when `within_potent_field` is set **and**
`effective/spell.rs::has_potent_magic` holds. That guard reads the data scope, not an
item id, so a stale marker adds nothing (mirroring `has_magical_focus`). Across
carriers it takes the **larger** bonus, not the sum: the same
`derived.rs::in_play_lab_total_mod_within_potent_field` fold the in-play Lab Total
reads (D79). The other D1 carriers stay flat. The order is unchanged: the flat term,
the focus doubling and the Potent bonus all sum into `base` before any halving.
`validate_spell_level_cap` passes `sel.within_potent_field`.

`spell_caps` adds `within_potent_field_cap` (when Potent Magic is held) and
`within_focus_and_potent_field_cap` (when both a Magical Focus and Potent Magic are
held), serialized only when present. The combined figure is an engine figure because
the halvings floor the sum. The plain `cap` never includes Potent Magic.

Tests: `tests/r3_potent_magic_spell_cap.rs`, `tests/r3_potent_magic_spell_cap_fields.rs`,
`composite_spell_five_dimensions.rs::level_cap_without_the_potent_field_mark_drops_the_potent_bonus`,
and `arm-app/tests/spell_caps_dto.rs::effective_scores_surface_the_within_potent_field_caps`.

**A Deficient Art halves the cap, because the cap *is* a Lab Total.** The closing
sentence of `ArMDE:2465` above is what makes the per-spell cap subject to every Virtue
and Flaw that touches a Lab Total in play, and a Deficiency is one of those:

> `ArMDE:5911` (Deficient Form, *Minor, Hermetic*) "Almost all totals (including
> Casting Totals and Lab Totals, but excluding Magic Resistance) to which a
> particular Form is added are halved."

> `ArMDE:5915` (Deficient Technique, *Major, Hermetic*) "All totals, including Lab
> and Casting totals, including a particular Technique are halved."

So `spell_level_cap` halves the summed total whenever the spell's Technique or Form
is deficient — **once** for the pair, however many of its two Arts are deficient,
which is the same reading `lab_totals` (`derived/lab.rs`) applies through
`InPlayMods::deficient`. Both read one fold, `deficient_arts` (`effective/art.rs`),
so the creation-time cap and the in-play Lab Totals can never disagree about which
Arts are deficient. The halving **floors** (`ArMDE:547` — "if it does not, round
down"), via `div_euclid(2)` as `halve` (`derived.rs`) does: this cap is legitimately
negative for a beginning magus, and a truncating `/ 2` would round −1 up to 0.
Tests: `deficient_technique_halves_the_spell_level_cap`,
`deficient_form_halves_the_spell_level_cap`,
`two_deficient_arts_on_one_pair_halve_the_cap_once`,
`a_negative_spell_level_cap_halves_downwards` (`effective.rs`);
`deficient_technique_halves_the_per_spell_cap`,
`a_deficiency_in_another_art_leaves_the_cap_unhalved` (`validation/mod.rs`).

**D28 (`docs/vf-audit/decisions.md`): the cap becomes range-aware for
Short-Ranged Magic, and it lowers a cap that was legal before.**

> `ArMDE:6739` (Short-Ranged Magic, *Major, Hermetic*) "Halve your Casting
> Totals whenever you are not touching the target of the spell. Halve your Lab
> Total when designing an effect or spell that has a range greater than Touch,
> including Eye."

Unlike D1's nine, this condition **is** a creation-time fact: the spell's own
`range` (`rules/core/spells.json`), not an in-play circumstance — which is why
D1 does not decide it and D28 rules separately. `Effect::HalvesSpellCapBeyondTouch`
(`types.rs`) is a bare marker, data-driven like `ForbidsRitualCasting`; no id is
hardcoded. `effective/spell.rs::range_beyond_touch` is the by-**name** whitelist
the halving reads — Eye, Voice, Sight, Arcane Connection, **not** Personal or
Touch — deliberately not derived from `SpellRange`'s difficulty ordering: Eye is
the *same* RDT difficulty as Touch, and the book names it explicitly for exactly
that reason ("greater than Touch, **including Eye**" would be redundant to state
if Eye already fell out of "greater than" by magnitude).

**Order of operations.** The D1 flat term and (X11b) the within-focus double
both sum into `base` first (they are part of what the Lab Total *is*), then
the two conditional halvings — Deficient Art and this one — apply. Neither
passage orders the two halvings relative to each other, and the order between
them is provably immaterial: both are a plain `div_euclid(_, 2)`, and floor
division by 2 twice equals floor division by 4 regardless of which comes
first (the same reasoning `derived/lab.rs::creo_corpus_lab_total` already
applies to its own Deficient/Difficult-Longevity stack).

**Re-keyed.** `SpellLevelCap` (`effective/spell.rs`) gains `range_beyond_touch:
bool` alongside `technique`/`form`; `spell_level_caps` now emits **two** rows per
Te/Fo pair (crossed with both range classes) instead of one. The picker
(`ui/src/lib/derive.ts`'s `nonTakeableReason` / `isDisabled`, fed by
`SpellTab.svelte`'s `capByTeFo` map) keys its lookup on
`` `${technique} ${form} ${beyond_touch}` ``, computing a candidate spell's own
`beyond_touch` from its `range` via `spellRangeBeyondTouch` (`derive.ts`).

**The whitelist stayed engine-side even after the picker moved on (D82.2).**
`SpellRange` is a fixed taxonomy, so the UI must not re-hardcode which
variants count as beyond Touch — but since D81.5's `capBySpell`
(`SpellTab.svelte`) replaced the picker's Te/Fo-keyed grey/offer logic with
the per-spell `spell_caps` DTO, the classification it greys a candidate spell
BY is read off that spell's own `SpellCap.range_beyond_touch`
(mirrors `effective::spell.rs::SpellLevelCap`), computed engine-side from
`effective::range_beyond_touch` — never recomputed in TypeScript.
`Ruleset.ranges_beyond_touch`, the `SpellRange::ALL`-derived set this
paragraph used to describe surfacing to `nonTakeableReason`/`isDisabled` via a
`spellRangeBeyondTouch` lookup, had no remaining reader once that move landed;
it was removed from the serialized `Ruleset` and the TS type (D82.2). The
predicate it was derived from, `effective::range_beyond_touch`, is unchanged
and remains the sole whitelist.

**It lowers a cap, so a save legal today can become invalid — same treatment as
D10:** `Enforced` blocks it, `Advisory` warns, no migration and no schema
change. `validate_spell_level_cap` (`validation/magus.rs`) computes
`range_beyond_touch` from the resolved spell's own `range` and passes it to
`spell_level_cap`, so the enforcement path and the surfaced picker cap can never
disagree.

**The catalogue entry stays `in_play_effect`, and it now needs a `description`
in both locales (D20).** `flaw.short_ranged_magic` carries two clauses: the
Lab-Total half is now computed (this section), but the Casting-Total half
("whenever you are not touching the target of the spell") is a per-cast
circumstance the engine cannot resolve at character-generation time and is not
modelled by any effect — `special_casting_mod: circumstantial` already carries
that, and D20 obliges the full rule to reach the player as text regardless, so
`rules/i18n/{en,de}/virtues_flaws.json` now state both sentences in
`description` (the existing `summary` already carried the Casting-Total
sentence alone).

Tests: `short_ranged_magic_halves_the_cap_only_beyond_touch`,
`without_short_ranged_magic_the_range_flag_changes_nothing`,
`spell_level_caps_expose_te_fo_int_mt_plus_three` (`effective.rs`).

**General spells.**

> `ArMDE:12349-12353` "Some spells are General spells (abbreviated to Gen), which
> means that they may be learned at any level … different levels of a General
> level spell are still different spells."

A General spell's catalogue `level` is `None`; the learned level is the
per-character `SpellSelection.level`. Identity (and the dedup key) is
`(spell, level, parameter)`; an unresolved General spell (no chosen level) warns
(`spell_level_unresolved`) and is excluded from the budget sum.

**Parametrized spells — one version per Hermetic Form.**

> `ArMDE:15791-15794` "**Wizard's Boost (Form)** … There are ten versions of this
> spell, one for each Hermetic Form." (Also **Mirror of Opposition (form)**
> `ArMDE:15776-15779`, **Wizard's Reach (Form)** `ArMDE:15801-15804`, and **Unravelling the
> Fabric of (Form)** `ArMDE:15843-15846`.)

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

> `ArMDE:12283` "Formulaic and Spontaneous spells may not have Year duration"
> `ArMDE:12284` "Formulaic and Spontaneous spells may not have Boundary target. They
> may have Vision target, if they are magical sense spells."
> `ArMDE:12285` "Formulaic and Spontaneous spells may not have a level greater than 50.
> (Note that they may have a level of 50, but not 51 or higher.)"
> `ArMDE:12290` "If the spell is a Momentary Creo spell creating a lasting thing, it
> must be a Ritual."
> `ArMDE:12293` "Ritual spells are always at least level 20, even if the level
> calculation would make them lower."
> `ArMDE:12055` "**Year:** … A spell with this duration must be ritual."
> `ArMDE:12077` "**Boundary:** … A spell with this target must be a Ritual."
> `ArMDE:12099` "**Vision**: … unlike Boundary, it does not require Ritual magic."
> `ArMDE:12039`/`ArMDE:12115` A Momentary Creo spell creating a lasting thing must be a
> Ritual (the Creo/Momentary interaction).

Two-level enforcement:

- **Load-time** (`Ruleset::validate_spell_refs`, `ruleset/integrity.rs`): for a fixed
  `level`, ritual ⇒ `level ≥ 20`, non-ritual ⇒ `level ≤ 50`; a non-ritual spell
  may not have `duration = Year` or `target = Boundary`, nor be a Momentary Creo
  spell with `creates_lasting`. Vision target is exempt from the Boundary rule.
- **Per-entity** (`validate_spell_ritual_legality`, `validation/magus.rs`, :621, called
  from `validate_spells`, V51 split it into a named sub-check): the *resolved* learned
  level (General chosen level or fixed) must obey the same ≥20 / ≤50 bounds — a
  violation emits `spell_ritual_legality` (`CODE_SPELL_RITUAL_LEGALITY`). This
  bites for General spells whose chosen level is illegal; fixed-level spells are
  already caught at load.

#### Rigid Magic — a known Ritual is an advisory warning, never a block
> "You cannot use vis when you cast spells. Thus, you cannot increase your
> spell rolls or cast Ritual magic. You can use vis in the laboratory, or to
> refresh a Longevity Ritual."

**Ruling: D27** (`docs/vf-audit/decisions.md`). `ArMDE:6697` forbids **casting**
Ritual magic, not knowing it — the spell list here models spells *known*, so
selecting a Ritual stays legal. A magus may have learned one before acquiring
the Flaw, and a Ritual he cannot cast is still worth holding (teach it, copy it
out). Blocking would enforce something the book does not say; the engine only
warns.

- Source: `ArMDE:6695-6698`, rule `ArMDE:6697`.
- Data: `rules/core/virtues_flaws.json` `flaw.rigid_magic` —
  `effects: [{ forbids_ritual_casting }]`; reclassified `narrative` →
  `creation_effect` (D46: classification follows what is computed). The two
  remaining clauses this does not compute — the spell-roll penalty and the
  laboratory/Longevity-Ritual exception, both in-play rather than
  creation-time facts — reach the player as `description` text in both locales
  (D20), carrying the full passage rather than only the summary's first
  sentence.
- Implementation: `validation/magus.rs::validate_ritual_casting_restriction`,
  called from `validate_spells` per known spell. Data-driven, like
  `flaw.unspecialized`'s `forbids_ability_specialties` (below, "Selection
  multiplicity" area): any item carrying `Effect::ForbidsRitualCasting`
  triggers `ritual_casting_restricted` (`ValidationIssue::warning`, never
  `::error`) on every held spell whose catalogue entry sets
  `spell.rs::Spell::ritual`. Reads *effective* selections (bought ++
  House/Mythic-type/`grants_selection` grants, D2), not `entity.selections`
  alone — the Flaw may be granted.

**Budget-modifier Virtues/Flaws (Skilled/Weak Parens).** Two Hermetic V/F modify
the apprenticeship grant, each via *two* `Effect`s
(`spell_levels` ± and `general_xp` ±, both signed, summed and clamped at 0):

| Item | magnitude/category | effects | source |
|------|--------------------|---------|--------|
| `virtue.skilled_parens` | Minor, Hermetic | `spell_levels +30`, `general_xp +60` | `ArMDE:4964-4966` |
| `flaw.weak_parens` | Minor, Hermetic | `spell_levels -30`, `general_xp -60` | `ArMDE:7072-7074` |

> `ArMDE:4966` (Skilled Parens) "You gain an additional 60 experience points and 30
> spell levels during apprenticeship."
> `ArMDE:7074` (Weak Parens) "You gain 60 fewer experience points and 30 fewer spell
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
spell list, extracted from the **Spells chapter** (`ArMDE:12385-15941`, from
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

**Roll tables inside a spell body are not description prose.** Three entries
interleave a Markdown table with their prose — *The Shadow of Life Renewed*
(`ArMDE:13442-13457`, table `ArMDE:13447-13455`), *Mists of Change* (`ArMDE:13633-13652`, table
`ArMDE:13638-13649`, closing paragraph `ArMDE:13651`) and *Visions of the Infernal
Terrors* (`ArMDE:15176-15188`, table `ArMDE:15180-15186`):

> `ArMDE:13638-13639` "| Roll | Animal | | ---- | ----…"

The extractor skips those rows when building `description` and keeps collecting
the prose that follows, so *Mists of Change* retains its closing paragraph. The
`description` is prose the UI renders as prose and cannot lay out a table; the
authoritative tabular text stays in `rules/source/<lang>/`, and the entry's
`source` range still spans the table so provenance is complete. Witness:
`no_spell_description_carries_markdown_table` in `tests/data_integrity.rs`. Two
descriptions legitimately end without sentence punctuation because the source
lines do (`ArMDE:14676` "…more distinct<br>", `ArMDE:15171` "…in their nostrils<br>"), so
the guard keys on the pipe character, never on end punctuation.

**An Art-class parenthetical in a spell name is a parameter slot.** Four names
end (or, in German, embed) a bare Art-class word — *Mirror of Opposition (form)*
`ArMDE:15776`, *Wizard's Boost (Form)* `ArMDE:15791`, *Wizard's Reach (Form)* `ArMDE:15801`,
*Unravelling the Fabric of (Form)* `ArMDE:15843` — each standing for ten spells, one
per Hermetic Form:

> `ArMDE:15793` "There are ten versions of this spell, one for each Hermetic Form."

The extractor recognises the closed set of class words (the `art_type` values in
`rules/core/arts.json`, plus their German spellings, since the translation table
writes the same marker in German — "Das (Form)-Gefüge auflösen") and emits both
halves itself: `"parameters": [{ "key": "form", "type": "ref", "domain": "form" }]`
in `rules/core/spells.json` (`Spell::parameters`, a `Vec<ParameterDef>` in
`spell.rs`) and the matching
`({form})` placeholder in the localized name in both i18n files.

**`spell.piercing_the_magical_veil` (X10(d), D65 row N5, D68.6) is a hand-added
exception to the 5d extraction, not a run of `extract_spells.py`.** The Criamon's
own spell list names it directly:

> `ArMDE:1745` "Piercing the Magical Veil (InVi 20/+18) (see Piercing the Faerie
> Veil)"

— Technique Intellego, Form Vim, level 20 (the Casting Total, +18, cross-checks
against her printed Arts with no adjustment needed). The link target,
`#piercing-the-faerie-veil`, is `spell.piercing_the_faerie_veil`'s own entry
(`ArMDE:15705-15710`); its closing line only **names** the Magical/Divine/
Infernal variants without printing a statblock for any of them:

> `ArMDE:15709` "There are separate but related spells for Divine, Magical and
> Infernal regiones."

So Range/Duration/Target are **copied from that sibling** (`personal` /
`concentration` / `vision`) as a recorded inference, per D68.6 — not a value the
book states for the Magical veil itself. The `description` in both i18n files is
correspondingly minimal (states the "see Piercing the Faerie Veil" cross-
reference and that the book prints no separate effect text), to satisfy
`data_integrity.rs::english_i18n_covers_all_spells` /
`german_i18n_covers_all_spells` (every catalogued spell needs a description)
without inventing rules text the source does not give. German name from the
line-parallel German source at the same `ArMDE:1745`: "Den magischen Schleier
durchdringen".

---

## Equipment: weapons / shields / armor (M5/5h)

The equipment catalogue (`equipment.rs` + `rules/core/equipment.json`) defines
weapons, shields, and armor as language-neutral records with per-row provenance.
This slice stores the character's choices (`Entity::equipment`, a `Vec<EquipmentSlot
{ item, loadout }>`) and validates references; it does **not** compute combat
totals, Soak, or Encumbrance — that lands in the derived-totals slice (5i), which
consumes these rows. The data shape is deliberately rich enough for 5i.

**Loadout state (K5, F1 —
`docs/vf-audit/design-f0-book-template-engine.md` § 2a).** `EquipmentSlot.loadout`
is a closed three-state enum (`LoadoutState`: `Stowed` / `Carried` / `Wielded`),
replacing the K2-era `equipped: bool`, which conflated two independent facts one
boolean cannot express. The Knight's own statblock forces the distinction: his
five printed Combat rows include the great sword (`ArMDE:1470-1471`) while his
printed Encumbrance ("2 (3)", `ArMDE:1484`) counts only his wielded set — the
book wants the spare weapon to yield a Combat row **and** contribute no Load.
`Stowed` (the default) yields neither; `Carried` yields a Combat row but no Load;
`Wielded` yields both (K2's old `equipped: true` behavior, unchanged). Shields
stay gated on `Wielded` alone — no book template shows a "carried but not
readied" shield modifying anything. `SCHEMA_VERSION` 19 → 20 for the field's
shape move (bool → enum); `migration.rs::fold_legacy_equipment_loadout` folds a
legacy `equipped: true`/`false` into `loadout: "wielded"`/`"stowed"` per element,
dispatching on `loadout`'s absence, never the claimed version. `types.rs::
LoadoutState` / `types.rs::EquipmentSlot::loadout`.

**Armor Table** (each material split into partial / full rows; full is `n/a` for
Quilted/Fur and Heavy Leather):

> `ArMDE:16944-16949` "| Quilted/Fur | 1 | 2 | n/a | n/a … | Chain Mail | 6 | 4 | 9 | 6 |"

`Armor { protection, load }` — Prot is the Soak bonus, Load feeds Encumbrance.
10 armor rows in `equipment.json`.

**Melee Weapon Statistics** (Ability, Init, Atk, Dfn, Dam, Str, Load; shields are
rows in this table but modeled as their own `Shield` type):

> `ArMDE:16959-16986` "| Dodge | Brawl | 0 | n/a | 0 | n/a | n/a | 0 … | Warhammer |
> Great | 0 | +6 | 0 | +12 | +2 | 3 |"

`ArMDE:16988` names the "Ability" column ("The Weapon Ability needed to use this
weapon"). `validate_weapon_refs` (in `ruleset/integrity.rs`) requires each weapon's
`ability` to resolve to a Martial Ability, or an Ability flagged
`combat_ability` in data — Brawl (the combat Ability for unarmed/improvised
weapons, which the rules class as General). 25 melee weapons + 3 shields.

**`weapon.grapple`** (X10(d), D65 row N5, D68.5) is not a Melee Weapon
Statistics row: the human table above prints no Grapple line at all. Its
figures are the bestiary's **Natural Weapons Table** instead, which the book
itself says use "Combat Statistics ... calculated as normal" (`ArMDE:18551`) —
the same formula this engine already applies to a magus:

> `ArMDE:18561` "| Grapple | 0 | 0 | 0 | n/a |"

`Init 0, Atk 0, Dfn 0, Dam n/a`, Ability Brawl, Load 0, `body_attack: true`
(D66) — like Dodge/Fist/Kick, Grapple has no separate mounted-combat twin
printed anywhere for anyone, so `data_integrity.rs::
body_attack_is_set_on_exactly_the_body_attacks`'s `EXPECTED` list now
names all four. 26 melee weapons + 3 shields (33 weapons total).

**`Ability.combat_ability` data flag** (`rules/core/abilities.json`, set on
`ability.brawl`; consumed by `validate_weapon_refs`): Brawl is the non-Martial
combat Ability, so the engine drives the weapon-Ability exception from this flag
rather than a hardcoded `ability.brawl` slug (a ruleset that slugs unarmed combat
differently just sets the flag).

> `ArMDE:7337-7340` "**Brawl** Fighting hand-to-hand without weapons, or with the
> sorts of improvised weapons you just pick up, including knives. Brawl is also
> the Ability used to dodge attacks if you have no Martial Abilities."

**Missile Weapon Statistics** (adds a Range column; Thrown-Ability rows are
`WeaponKind::Thrown`, Bow-Ability rows `Missile`):

> `ArMDE:17005-17011` "| Axe, Throwing | Thrown | 0 | +2 | 0 | +6 | 5 | 0 | 1 … | Bow,
> Short | Bow | –1 | +3 | 0 | +6 | 15 | –1 | 2 |"

7 missile-table weapons (33 weapons total, including `weapon.grapple` above).

**`n/a` cells** are `Option::None`: Dodge has no Attack/Damage; the body attacks
(Dodge/Fist/Grapple/Kick) have no minimum-Strength — distinct from a real `0`
(Fist's `+0` Attack). `min_strength` for a weapon and a shield are met separately
(`ArMDE:16997`).

**Weapon + shield combine** — a weapon+shield combatant **adds both** rows'
modifiers (computed in 5i):

> `ArMDE:16656` "If the character is using a weapon and a shield, add together the
> modifiers of the weapon and the shield to get the final modifier."

**Encumbrance** (the Load→Burden table; Encumbrance = `max(0, Burden − max(0,Str))`)
is defined at `ArMDE:17103-17123` and **computed in 5i**, not here:

> `ArMDE:17109-17123` "| Total Load | Burden | | 0 | 0 | | 1 | 1 | … | 55 | 10 |"

`validate_equipment` (in `validation/equipment.rs`, :16) emits `unknown_equipment` (error) for a
slot whose id resolves to no catalogue row, and `equipment_min_strength` (advisory
warning) when a **Wielded** weapon/shield's min-Strength exceeds the character's
(aged) Strength — never blocking, since wielding an over-heavy weapon is a
storyguide call. A merely Carried weapon (K5) triggers no such warning: it is
not the one actually being used. German equipment names follow
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
| Casting Score | `ArMDE:9089` | Technique + Form + Stamina − Encumbrance + Aura |
| Cast types | `ArMDE:9103-9145` | Formulaic = score; Ritual = score + Artes Liberales + Philosophiae; Spont fatiguing = ÷2; non-fatiguing = ÷5 |
| Method Caster | `ArMDE:4524-4527` | +3 flat, Formulaic/Ritual scope only |
| Non-standard casting | `ArMDE:9236-9245` | Words/Gestures penalties (Formulaic/Spont, not Ritual): no voice −10, no gestures −5. The penalty is **to the Casting Score** (`ArMDE:9236` "Increased subtlety gives a penalty to the casting score", repeated at `ArMDE:9247`), so it is summed with the other Casting-Score terms **before** the Deficient-Art halving: `silent` = `halve(score + residual voice penalty)`, not `halve(score) + penalty`. Round 4 (Gerda 1) corrected the ordering, which this row previously documented the wrong way round. Per cell, `NonStandardCasting` exposes `silent`, `still` and `silent_and_still`; residuals clamp at 0 |
| Quiet Magic | `ArMDE:4822-4826` | reduces the no-voice penalty +5 per casting (soft voice → 0, no voice → −5; a second casting eliminates it) |
| Subtle Magic | `ArMDE:5073-5076` | reduces the no-gesture penalty +5 (no gestures → 0) |
| Deft Form | `ArMDE:3645-3648` | casting in the named Form suffers **no** non-standard voice/gesture penalty (both residuals 0 for that Form's cells) |
| Magical Focus | `ArMDE:4399-4422` | within focus, add the **lower** applicable Art again (per-`(Te,Fo)` `within_focus` value; applicability is user-judged, never auto-detected) |
| Deficient Art | `ArMDE:5909-5915` | totals adding that Technique/Form **halved** (Form excludes Magic Resistance) |
| Lab Total | `ArMDE:10276-10278`, `ArMDE:4151-4154` | Int + Magic Theory + Technique + Form + Aura + flat LabTotalMod (+ focus / halving as casting). "**YOUR BASIC LAB TOTAL IS: Technique + Form + Intelligence + Magic Theory + Aura Modifier**" (`ArMDE:10276`); `ArMDE:4151-4154` is Inventive Genius, the flat `LabTotalMod`. (Corrects the earlier `ArMDE:4143-4154`, which is the Inspirational / Intuition / Inventive Genius Virtue block, not the formula.) |
| Penetration | `ArMDE:9159-9161` | per known spell: Casting Total − Level + Penetration score |
| Weak Magic | `ArMDE:7064-7067` | halves Penetration **after** subtracting level (not the casting total) |
| Magic Resistance | `ArMDE:9390-9398` | per Form: Form + 5 × Parma Magica (Form-base rule `ArMDE:9390`, Parma "five times" `ArMDE:9398`). Both Flaws that modify it are scoped to **one named Form**, carried as a `form` parameter on the selection: **Limited Magic Resistance** (`ArMDE:6346-6349`) drops that Form's own bonus ("no bonus from **one of** your Form scores"), and **Flawed Parma Magica** (`ArMDE:6142-6145`) halves the **Parma addend alone** against that Form — `MR = form_bonus + halve(5 × Parma)`. Only the Parma addend, because the Flaw's subject is "Your Parma Magica" while `ArMDE:9396` puts the rest of the resistance on "a maga's Form scores", which a defective Parma does not produce and cannot reduce. Worked example: Ignem 10, Parma 3, Flawed Parma (Ignem) → Ignem MR = 10 + halve(15) = **17**, every other Form unchanged at its score + 15. A Might base is never halved by it: Might and Parma do not stack and the higher is the base (RoP:M:1472, `ArMDE:2627`), and the comparison is made against that Form's own (possibly halved) Parma figure, so the halving only ever moves the Parma side of it. **A True Faith Score is a third, independent source and a floor, not an addend** (D37, F-329): "a True Faith Score gains Magic Resistance equal to this score multiplied by ten" (`ArMDE:17611`), and per-Form MR = `max(form_bonus + base, true_faith_score × 10)` — competing with the Form's **whole** total, since (unlike Might, `RoP:M:1472`) no passage says the Form bonus is compatible with it, and "these totals do not stack … you simply use the higher total" (`ArMDE:2627`). Being a separate source, the floor is untouched by Flawed Parma Magica or Limited Magic Resistance and so can override either's reduction on the Form it names: Ignem 10, Parma 3, Flawed Parma (Ignem) = 17, but True Faith 2 (floor 20) raises Ignem to **20**, and every other Form (ordinarily 15) to 20 as well, since the floor is blanket. A Faith Point alone (no True Faith Score) grants nothing (`ArMDE:17611`, same sentence). No new `Effect` — `effective/might.rs::true_faith` already sums `Effect::TrueFaithGrant`, so this is a consumer change; when the floor wins, the breakdown is replaced by a single `true_faith` addend (Fluent `derived-addend-true_faith`, en/de). `derived/casting.rs::magic_resistance` |
| Longevity (stored) | `ArMDE:10662`, `ArMDE:10668`, `ArMDE:10670` | the aging bonus is the **player-entered** `LongevityRitual.bonus`, passed through for **both** sources; `entered: false` marks an unfilled field so a placeholder 0 is never read as a claim. Bronze cord noted for aging-resistance (`ArMDE:10840-10844`), via `cord_score` so it respects the +5 maximum (`ArMDE:10836`) and matches the Soak and cord-cost figures |
| Longevity hint | `ArMDE:10662`, `ArMDE:10276-10278`, `ArMDE:17658`, `ArMDE:5909-5915`, `ArMDE:5962-5964` | self-made only: `LongevityHint { lab_total, suggested_bonus, halved }` — Creo Corpus Lab Total (Int + Magic Theory + Creo + Corpus + Aura + flat LabTotalMod), halved by a Deficient Creo/Corpus and again by Difficult Longevity Ritual, then `suggested_bonus = ceil(lab_total / 5)` floored at 0. **Read-only guidance** — never written into the entity. `derived/lab.rs::suggested_longevity_bonus` / `creo_corpus_lab_total` |
| Masterpiece | `ArMDE:4476-4479`, `ArMDE:10410`, `ArMDE:7060-7063` | magus with the Masterpiece Virtue (`Effect::MasterpieceItem` marker) surfaces a **read-only** lesser-enchanted-item cap = **best base `(Te,Fo)` Lab Total ÷ 2** (the lesser-enchantment rule caps single-season instillation at Lab Total ≥ 2×effect level, `ArMDE:10410`; vis costs ignored per the Virtue). The best cell is picked by (and the cap built from) `LabTotal.enchanting`, not the plain `total`: designing the item is "creating" an enchanted item, so a **Weak Enchanter** magus's halved figure (`ArMDE:7060-7063`) is "the regular rules for construction of such a device" (`ArMDE:4476-4479`) for him too — `enchanting` equals `total` for everyone else, so the formula is unchanged for a magus without the Flaw (round 3, G1: the un-halved `total` used to leak through here, doubling the cap). No focus doubling. The engine does **not** create the device or spend an item-level budget — the player still enters the actual lesser enchanted item by hand under Magic Items; this is guidance only. `masterpiece_item_cap` / `DerivedTotals.masterpiece` |
| Talisman capacity | `ArMDE:10619`, `ArMDE:4347-4349`, `ArMDE:4842-4850` | magus **with a talisman** surfaces a **read-only** enchantment capacity in pawns of Vim vis: "The maximum number of pawns of Vim vis that may be used to prepare a talisman is equal to the sum of the magus's highest Technique and highest Form" (`ArMDE:10619`). Taken from the per-Art maxima of **effective** scores (`effective_art_score`, so Puissant Art folds in), **not** the best `lab_totals` pair — a Deficient Art halves *totals*, never the score, and a discriminating test pins that. Ties go to the alphabetically first Art (`Ruleset::art_ids_of` is sorted); a magus with no bought Arts still reads out, at 0 pawns. **Non-goal**: instilled `TalismanEffect` levels are charged against **no** budget — `item_level_budget` comes only from the Redcap-only Virtues (Magic Items "You must be a Redcap to take this Virtue", `ArMDE:4347-4349`; Redcap's fifty starting levels `ArMDE:4842-4850`, which also states "You may not take The Gift", `ArMDE:4850`), so it can never fund a magus's talisman, and the talisman's real limit is this vis capacity, which the model cannot enforce (it holds no vis stock). `derived/familiar.rs::talisman_capacity` / `DerivedTotals.talisman_capacity` |
| Familiar bonding level | `ArMDE:10824`, `ArMDE:10828` | magus **with a familiar** surfaces a **read-only** bonding level = **Magic Might + 25 + 5 × Size**: "The level for the enchantment is equal to 25 plus the familiar's Magic Might plus 5 times its Size. If the familiar has negative Size, this reduces the level for the enchantment" (`ArMDE:10824`), restated as "**FAMILIAR BONDING LEVEL: Familiar's Magic Might + 25 + (5 x Size)**" (`ArMDE:10828`). Size is signed and commonly negative, so the level routinely drops *below* 25 — the book's own worked example (Size -2, Might 10 → level 25) is a named test. A familiar with **no entered Might** contributes 0 rather than suppressing the read-out; the panel says "no Magic Might entered" instead. `derived/familiar.rs::familiar_binding_level` |
| Familiar bonding Lab Total | `ArMDE:10818`, `ArMDE:10822`, `ArMDE:10826`, `ArMDE:10824` | the **ordinary Lab Total shape** — "any appropriate Technique + any appropriate Form + Int + Magic Theory + Aura Modifier" (`ArMDE:10818`), restated as "**FAMILIAR BONDING LAB TOTAL**" (`ArMDE:10826`) — so `lab_totals()` is **reused** and the best `(Te,Fo)` cell taken with `max_by_key`, the same max-over-grid reuse as Masterpiece. Which Arts are *appropriate* to a given beast is prose the engine cannot evaluate, and "Any magus should be able to find an animal that he can bind with his best Technique and Form" (`ArMDE:10822`), so the best cell is the honest figure. **Unlike Masterpiece, a focus applies here**: "Puissant Arts and foci may apply to this" (`ArMDE:10818`) — so `lab_total_within_focus` is surfaced as a **separate conditional** figure the UI labels as such (whether *this* familiar falls inside the focus's narrow field is a troupe judgment); Puissant Arts need no separate figure, `effective_art_score` folds them in. `lab_total_reaches_level` reports "A magus can only bind a familiar if his Lab Total equals or exceeds this level" (`ArMDE:10824`). `derived/familiar.rs::familiar_readout` / `FamiliarBinding` |
| Familiar cord cost | `ArMDE:10836` | cord scores 0…+5 cost **0 / 5 / 15 / 30 / 50 / 75** points: "a strength of +1 requires 5 points, a score of +2 requires 15 points, a score of +3 requires 30 points, a score of +4 requires 50 points, and a score of +5 (the maximum) requires 75 points" (`ArMDE:10836`). The curve is a fixed 5-entry rule constant, so it is a Rust `const CORD_COST_TABLE` (precedent: `LOAD_TABLE` for Encumbrance) with this row as its provenance home. The same line also fixes the **+5 maximum** ("rated from 0 to +5 … a score of +5 (the maximum)"), which is stated in **one** place in the engine: `pub const MAX_CORD_SCORE: u8 = 5` in `types.rs`, beside the `Familiar` type whose fields it bounds. Two clamps read it and neither restates the number. (1) **On read** — `derived.rs::cord_score(raw)`, which **every** cord consumer routes through: `cord_points_spent` (the table index), `soak`'s `bronze_cord` addend, and `longevity_bonus`'s `bronze_cord` note. Cord fields are plain `u8`, so a hand-edited or legacy save can carry any value up to 255; unclamped, one entered number produced three contradictory figures (75 points spent / +255 Soak / +255 aging-resistance) and a raw table index would panic inside the `derived_totals` command and kill the read-out panel. (2) **On store** — `Familiar::normalize` clamps the three fields, so an out-of-range value **self-heals on the next save**, exactly as a zero Characteristic is pruned there and for the same canonical-serialization reason: since every consumer clamps, `cord_bronze: 255` and `cord_bronze: 5` are the same statement to the engine, yet they serialize differently and *display* differently (the panel renders the raw 255 in an input declaring `max="5"`, beside read-outs computed from 5) — a disagreement the user cannot resolve and that saving would otherwise perpetuate forever. UI input bounds are a separate, non-durable layer and do not replace either clamp. Tests: `cord_points_spent_clamps_a_score_above_the_curve`, `an_out_of_range_bronze_cord_reads_the_same_for_every_consumer`, `entity_normalize_clamps_out_of_range_familiar_cords`. `cord_points_within_lab_total` reports "The total cost of the cords you buy cannot exceed the magus's Lab Total" (`ArMDE:10836`). `types.rs::MAX_CORD_SCORE` + `Familiar::normalize` / `derived.rs::cord_score` / `derived/familiar.rs::cord_points_spent` |
| Familiar invested powers | `ArMDE:10866`, `ArMDE:10862-10884` | the total level of the powers invested in the bond, summed **for information only**: "there is no limit to the number of powers which may be invested in a familiar" (`ArMDE:10866`). So — unlike a being's own `Entity.powers`, which `power_levels_budget` bounds — there is **no budget bar** and no issue to raise, and the UI must not render one. Vis costs (`ArMDE:10882`, one pawn per ten levels) are out of scope: the model holds no vis stock. `derived/familiar.rs::familiar_invested_power_levels` |
| Combat | `ArMDE:16658-16670` | Init = Qik + WpnInit − Enc + CombatMod; Attack = Dex + Ability + WpnAtk + CombatMod; Defense = Qik + Ability + WpnDef + CombatMod; Damage = Str + WpnDam + CombatMod |
| Weapon+shield | `ArMDE:16656`, `ArMDE:7746`, `ArMDE:1317`, `ArMDE:1467-1472` | "If the character is using a weapon and a shield, add together the modifiers of the weapon and the shield to get the final modifier" (`ArMDE:16656`). A shield is raised and dropped at will, so **each Carried-or-Wielded one-handed weapon emits TWO `CombatLine`s** (K5) while any **Wielded** shield is present: the with-shield line first (every Wielded shield's Init/Atk/Def mods added; all their ids listed in `CombatLine.shields`, since multiple shields stay **summed** into one pseudo-shield), then the bare line (`shields` empty, shield mods zeroed). The two differ **only** by the shield modifiers — Encumbrance and the specialization bonus are shield-independent, the latter explicitly so (`ArMDE:7746`, see below). With-shield-first mirrors the book's own statblocks, which print the weapon-and-shield lines ahead of the rest (`ArMDE:1467-1472` "- Long sword and heater shield (mounted): … - Great sword (mounted): …"). The renderers label the with-shield line `<weapon> <joiner> <shield>`; the joiner is the **translatable** `derived-combat-shield-joiner` Fluent key, not a hardcoded `&`, because the book alternates `&` (`ArMDE:1317` "- Axe & Heater Shield: Init +1, Attack +17, Defense +15, Damage +8") with "and" (`ArMDE:1468`) and the German rulebook writes `&` too. With no Wielded shield the output is unchanged: one bare line per weapon. `derived/combat.rs::combat_totals` / `export/sections.rs::combat_line_name` / `derive.ts::combatRowLabel` |
| Two-handed weapon | `ArMDE:7494`, `ArMDE:17008-17013` | a two-handed weapon (`Weapon.two_handed`) cannot be paired with a shield, so it receives **no** shield Init/Atk/Def mods (the shield still counts toward Load, `ArMDE:17107`) and therefore emits exactly **one** line — there is no second way to wield it. The 9 Great-Weapon melee weapons ("Fighting with a weapon which requires two hands to use", `ArMDE:7494`) + both bows are flagged in `rules/core/equipment.json`. The bows come from the missile table's asterisked rows and its footnote (`ArMDE:17008-17013`, i.e. `| Sling* …` through `ArMDE:17013` "\* Requires two free hands to load and fire."), **not** from the Bows Ability entry at `ArMDE:7333-7334` or the `Bow, Long:` flavor note at `ArMDE:17099`, neither of which says anything about two hands. The **Sling** shares that asterisk but stays unflagged: it is a thrown weapon and keeps the status-quo shield handling. (`ArMDE:17017` is the missile table's "Atk:" column legend — not a citation for this rule.) `derived/combat.rs::combat_totals` |
| Shield + two-handed advisory | `ArMDE:7494`, `ArMDE:17063`, `ArMDE:16975` | advisory `shield_with_two_handed_weapon` warning when a **Wielded** shield accompanies **only** two-handed Wielded weapon(s) — its dropped modifiers otherwise look like a bug (buckler prose `ArMDE:17063`; shield table "Single" Ability column `ArMDE:16975`). Non-blocking. `validation/equipment.rs` |
| Specialization +1 | `ArMDE:7122`, `ArMDE:7139`, `ArMDE:7746` | when `EquipmentSlot.specialization_applies` is set AND the weapon's combat Ability carries a non-empty specialty, the Ability acts "as if your score were one level higher" (`ArMDE:7122`, Single Weapon longsword example) for **Attack and Defense only** (Damage/Init do not use the Ability); "Add +1 when using an Ability's specialization" (`ArMDE:7139`). The bonus is **shield-independent** and so applies identically to the with-shield and the bare line: a Single Weapon specialty is "any one weapon or shield, which covers using that weapon with any shield or none, and that shield with any weapon" (`ArMDE:7746`). `derived/combat.rs::specialization_bonus` |
| Unspecialized forbids the specialty itself | `ArMDE:6943-6946` | `flaw.unspecialized` (`ArMDE:6945`) — "The character does not have any specialties for any of her Abilities" — is a claim about what the character *has*, not what he *rolls*, so the specialization bonus above is deliberately left alone (teaching it to return 0 would still leave the sheet printing a specialty the character may not have). The fix is data-driven: `Effect::ForbidsAbilitySpecialties` (a bare marker, no fields), shipped only on `flaw.unspecialized`, and `specialty_forbidden` errors on any Ability row carrying a non-empty `specialty` while an item carrying that effect is held — read against `effective_selections` (bought ++ granted) since no shipped profile currently grants it but a future one should be caught identically, and any future item carrying the same effect works with zero code changes. Row 47 / V/F-audit F-524; classification moved `narrative` → `creation_effect` (D46: classification follows what is computed) — this is the entry's own fix, not the catalogue-wide reclassification pass (`docs/vf-audit/phase-2-plan.md` slice X2). No `description` is added in either locale: the shipped `summary` already carries the whole one-sentence rule verbatim (§ 3.9). `validation/scores.rs::validate_ability_specialty_permitted` |
| Enc-exempt (unconditional) | `ArMDE:17105`, `ArMDE:17107` | Attack/Defense are Encumbrance-penalized **only** when the Encumbrance is *not* largely weapons + armor; Init is **always** penalized (`ArMDE:16658`). Here the waiver is **unconditional**: all modelled Load is combat gear by construction, so Attack/Defense never take the penalty and only Init does. Load is "listed in the Armor and Weapons tables" (`ArMDE:17107`), and the engine mirrors that — `equipment.rs::EquipmentCatalogue` holds exactly `weapons`, `shields` and `armor`, `load` is declared on exactly those three structs, and `equipment_load` yields 0 for anything else. No `Entity` can therefore carry non-combat Load. Earlier revisions encoded a majority test ("largely" read as combat-gear Load ≥ half of total Load); both of its sums ranged over the same items, so it was identically true and has been removed rather than left as a decision in shape only. The interpretation question returns the day a non-combat load-bearing item is modelled — which requires a new catalogue collection, not a data edit. `derived/combat.rs::combat_totals` |
| Soak | `ArMDE:16666` | Stamina + Armor Protection + SoakMod (Tough +3) + Bronze cord; Form bonus situational (entered 0). The Bronze-cord addend goes through `cord_score` (the +5 maximum, `ArMDE:10836`) so it cannot disagree with the cord-cost or Longevity read-outs |
| Encumbrance | `ArMDE:17103-17123`, `ArMDE:1484` | Burden from Load table `[0,1,3,6,10,15,21,28,36,45,55]→[0..10]`; Enc = `max(0, Burden − max(0,Str))`. **Only `Wielded` gear contributes Load** (K5) — a Stowed **or Carried** spare weapon is inert. The rule says to total "the Load that a character is carrying" (`ArMDE:17107`) and never defines carried-but-not-wielded, so the book's own worked characters settle it: the Knight template lists full chain mail, long sword, heater shield **and** a great sword yet prints "Encumbrance: 2 (3)" (`ArMDE:1484`). Burden 3 is Load 6-9, i.e. the *wielded* set (1+2+6 = 9); all four total 11, which is Burden 4. His alternate loadout reaches the same Burden (great sword 2 + chain 6 = 8), which is why one printed Encumbrance serves all four of his Combat rows. The engine totalled everything carried until 2026-09-23, when building the Knight as a fixture exposed the disagreement — one point of Init on every row, Atk/Def untouched per the waiver two rows above, which is what isolated Load as the sole term in dispute. Before K5 (F1), one boolean could not express "prints a Combat row but contributes no Load", so the Knight's fixture left the spare great sword unequipped entirely — a wrong-but-plausible workaround that cost the great sword's own Combat row (`ArMDE:1470-1471`). K5's `loadout: Carried` state fixes that: the great sword now carries `loadout: "carried"` and prints its row while staying out of Load. `derived/combat.rs::encumbrance`; tests `derived.rs::only_equipped_gear_counts_toward_load`, `derived.rs::a_carried_weapon_yields_a_combat_row_with_no_load` and `book_templates.rs::the_knight_matches_the_book` |
| Fatigue | `ArMDE:17127-17129` | Weary −1, Tired −3, Dazed −5 ("Each Fatigue level above Winded has a penalty", `ArMDE:17129` — Winded itself takes none), adjusted by HealthMod fatigue delta |
| Wounds | `ArMDE:17167-17191` | Size unit `u = max(1, Size+5)`; Light 1..u, Medium u+1..2u, Heavy 2u+1..3u, Incap 3u+1..4u, Dead 4u+1.. ; penalties −1/−3/−5 adjusted by HealthMod wound delta |
| Decrepitude / Warping | `ArMDE:16617`, `ArMDE:16464-16475` | **reused** from `effective/warping.rs` (`decrepitude_score`, `warping_score`), not reimplemented |

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
+7 (`ArMDE:2573`, `ArMDE:2488`); casting total with Encumbrance + Focus (base vs within-focus, Method
Caster +3); Deficient Technique halving; per-Form Magic Resistance = Form + 5×Parma;
Flawed Parma halving the Parma addend against its own Form only (Ignem 10 + Parma
3 → 17, other Forms untouched, a Might base never halved); Limited Magic
Resistance dropping its own Form's bonus only; per-spell penetration + Weak Magic; weapon+shield combat line;
Soak with Tough + Bronze cord; Encumbrance from Load; wound ranges Size 0 / +1;
Enduring Constitution penalty reduction; Decrepitude 17→2 & Warping 15→2 via the
reused functions; purity.

**Non-standard casting is computed** (no longer surfaced-only). The three penalty
relievers — Quiet Magic (`quiet_words`), Subtle Magic (`subtle_gestures`), Deft
Form (`deft_form`, now Form-parameterized) — are folded by `in_play_mods` into a
voice reduction, a gesture reduction, and a Deft-Form set, and each casting cell
carries a `NonStandardCasting` variant (`silent` / `still` / `silent_and_still`,
residuals clamped at 0). Every **other** `SpecialCastingMod` kind (spontaneous-magic
variants, circumstantial halvings, doubled aura penalties) stays surfaced-only.

**Surfaced-only families** (study / conditional-casting /
wound-recovery / conditional MR: `AdvancementMod`, the non-computed
`SpecialCastingMod` kinds, `AbilityRollMod`/`AbilityRollModParam` (B5/F-489: the
fixed-target and parameter-relative twins alike), the `HealthTrack::{FatigueRoll,
CastingFatigue, Recovery}` tracks, and the `MagicResistanceMod`
kinds that are neither `no_form_bonus` nor `halved_parma`
— `ModifierFamily::MagicResistance` — aura_bonus, the two realm
susceptibilities and Weak Magic Resistance's conditional Penetration waiver) are
**listed** as labelled `SurfacedModifier`s, not folded into a
simulated number, because the app does not simulate those subsystems.

**`AgingMod` left this list in M6/6b6.** `derived.rs` still lists it as a standing
modifier, but it is no longer surfaced-*only*: `aging.rs` consumes it — see
**Aging (M6/6b6)**, and the `AgingMod` row of the in-play effect table below.

**D45/F-423 — a surfaced row names its source.** Before D45, six shipped Flaws
(`flaw.corrupted_spells`, `flaw.deleterious_circumstances`,
`flaw.disjointed_magic`, `flaw.environmental_magic_condition`,
`flaw.short_ranged_magic`, `flaw.the_constant_expression`) all carrying
`special_casting_mod { circumstantial }` rendered as the byte-identical row
"Special casting: Circumstantial", with nothing to tell a character holding
two of them apart. `SurfacedModifier` gains `source: Option<Id>`, populated at
every push site **inside `in_play_mods`** with the granting item's own id
(`item.id.clone()`) — `MagicResistance`, `Aging`, `Advancement`,
`SpecialCasting` and `AbilityRoll`. The frontend renders it through the label
map as the item's localized name (`DerivedSurfacedModifiersSection.svelte`,
`derived-surfaced-source`), never the raw id.

`ModifierFamily::HealthRoll` is deliberately excluded, and stays `None`: unlike
the other five, its push site sits in `surfaced_modifiers` rather than
`in_play_mods`, reading back `InPlayMods::health_mods` — a `BTreeMap` that
already SUMS every contributing selection's amount into one number before this
family is built. There is no single item left to name by then (Q-81's
twelve-row `health_mod` census found more than one Virtue/Flaw commonly
sharing a track), so attributing one would be inventing an answer the data
does not have. De-duplicating identical rows into one line listing every
contributor was considered and rejected (D45): naming the source is what fixes
the anonymity defect, and collapsing rows is a separate UI call.

#### M5.5a — Longevity Ritual: stored value + live hint

M5/5i **derived** the self-made aging bonus from the current Creo Corpus Lab Total.
That was wrong, and this slice replaces it with a stored player-entered value plus a
read-only suggestion.

**Why the bonus cannot be derived.** The formula is "+1 bonus for every five points
or fraction of Creo Corpus Lab Total" (`ArMDE:10662`) — but the Lab Total in that sentence
is the one the *creating* magus had in the season the ritual was made. The rules
never restate this as a standalone sentence; it follows from two passages, so it is
recorded here as an **inference from quoted text**, not a quoted rule:

> If you reinvent the ritual to take advantage of increased Art scores, you can
> choose not to use extra vis. — `ArMDE:10670`

> A Longevity Ritual's effect lasts until you suffer an aging crisis […] After this,
> the ritual loses its effectiveness and the focus must be repeated. — `ArMDE:10668`

Reinvention is what captures raised Arts, and a failed ritual is repeated *unchanged*.
Deriving the bonus live therefore let a Creo increase or a move to a stronger aura
silently rewrite a past event. So `LongevityRitual.bonus: Option<i8>` is now entered
and stored for **both** sources (`None` = not entered yet, never a claimed 0), and
`LongevityBonus.entered` lets the UI distinguish an unfilled field from a deliberate
0. The derivation survives only as `LongevityHint` beside the input.

**`LongevityRitual.focus`** (free text, additive — no schema bump) records the
ritual's culminating focus:

> The ritual takes a season, and culminates in some sort of focus, which is
> appropriate to the magus in question. — `ArMDE:10656`

The same line carries the consequence the UI surfaces as a note — "the magus becomes
permanently sterile" (`ArMDE:10656`). The rules attach no mechanics to either, so both are
free text / a note; nothing is validated.

**The `aura != 0` gate is deleted.** 5i suppressed the whole bonus unless
`entity.aura != 0`, which has no source support: the Aura Modifier is a plain addend
in the Lab Total (`ArMDE:10276-10278`), `lab_totals()` has never gated on it, and a
zero-aura location is explicitly unhindered —

> The mundane has no aura rating — in fact, it is the absence of aura, so powers used
> there function without hindrance. — `ArMDE:17658`

A zero-aura magus now gets a hint; a negative aura simply lowers it.

**Two halvings now apply to the hint** (`creo_corpus_lab_total` returns
`(total, halved)`):

> Almost all totals (including Casting Totals and Lab Totals, but excluding Magic
> Resistance) to which a particular Form is added are halved. — Deficient Form,
> `ArMDE:5909-5911`; Deficient Technique likewise, `ArMDE:5913-5915`

> Anyone (including yourself) creating a Longevity Ritual for you must halve their
> Lab Total. — Difficult Longevity Ritual, `ArMDE:5962-5964`

This is the **first read of `HalvableTotal::LabLongevity`**: before M5.5a
`flaw.difficult_longevity_ritual` was collected by `in_play_mods` and moved no
number.

**That the two halvings compound is an inference, not a quoted rule.** Each Flaw
says to halve the Lab Total and neither carves out the other, but no passage states
the interaction. Order is pinned base → Deficient → Difficult and is numerically
immaterial, since the two are the same operation and `halve()` floors
(`ArMDE:547`), so halving twice is `floor(base / 4)` whichever Flaw is named
first. `halved` is a single flag: the UI
marks the hint as halved without claiming which Flaw did it.

`suggested_bonus = ceil(lab_total / 5)`, floored at 0 — "every five points or
fraction" has no meaning below one point, and a negative Lab Total must not suggest a
negative bonus. Worked example on the shipped ruleset: Lab Total 35 → +7, the book's
own sheet line (`ArMDE:2573`; the same magus's lab season at `ArMDE:2488` shows the vis cost
too).

**Note on the Flaw's cited range.** `rules/core/virtues_flaws.json` records
`flaw.difficult_longevity_ritual` as `ArMDE:5962-5965`; line 5965 is blank, so the
accurate inclusive range is **`ArMDE:5962-5964`**, which is what this section and the
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
cannot make (which Arts suit the beast, `ArMDE:10818`; whether a Magical Focus covers it)
and on vis, which the model does not hold (`ArMDE:10830`, `ArMDE:10882`). So the engine reports
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
`age_ability_cap(entity, ruleset)` expose it to the engine's own consumers, which
apply the cap during validation so the UI never re-hardcodes it. A ruleset that
ships no bands cannot enforce the cap (returns `None`).

> `ArMDE:2366-2374` "Your character's age determines the maximum score … | under 30 |
> 5 | | 30-35 | 6 | | 36-40 | 7 | | 41-45 | 8 | | 46+ | 9 |"

`validate_abilities` flags a bought Ability above this cap (`ability_above_age_cap`).
An Ability carrying an Affinity may exceed it **by +2** — not without limit:

> `ArMDE:3374` (Affinity with (Ability)) "you may exceed the normal age-based cap
> during character generation … by two points for that Ability."

**D29: one resolution point.** `ability_age_cap` (per-ability,
`effective/reputation_and_caps.rs`) folds the age band with EVERY Virtue/Flaw
override — the locality-dependent halving above, the Affinity +2, and the
full waiver below — so `validate_abilities` (and any future UI surface) reads
this one function rather than re-deriving an override beside it. It used to:
the Affinity +2 was applied a second time, in the validator, alongside a call
to this function that did not know about it — invisible so long as nothing
else read the function directly, but exactly the shape that cannot express
Savantism's favored Ability (general cap **lowers** to 3, the one favored
Ability **raises** to 6 above an age cap of 5 — a validator beside the age
rule can lower but never raise). Savantism itself stays unshipped (F-510); D29
only fixes the resolution point.

**F-194: Mentored by Demons waives the cap outright.**
`Effect::WaivesAbilityAgeCap` is a bare marker (no target — the passage names
no Ability list) that makes the per-ability resolution return `None` (no cap
to enforce) for every Ability the character holds:

> `ArMDE:4498` "Characters trained by demons may exceed the maximum skill
> level for a given age provided by the character creation rules."

`virtue.mentored_by_demons` carries it alongside its existing 50-XP grant.
Data-driven: the engine hardcodes no virtue id, so a house rule or future
book's item carrying the same effect is caught with zero code changes.

**The Gift → one free Supernatural Ability.** A Supernatural Ability normally
requires a granting Virtue (an `ability_score_grant` effect seeds it — Second
Sight, etc.). The Gift lets a Gifted **non-magus** take one such Ability with no
Virtue; a **magus** gets none (his free supernatural ability is Hermetic magic):

> `ArMDE:2874` "Characters who have The Gift may start play with a single Supernatural
> Ability, without having to take any other Virtue … The ability to cast Hermetic
> magic is the single supernatural ability possessed by Hermetic magi in virtue
> of The Gift".

`effective::supernatural_free_slots` = `(has_the_gift && !is_hermetically_trained ? 1 : 0, used)`;
`validate_supernatural_abilities` errors on uncovered Supernatural abilities beyond
the free allowance (`supernatural_ability_requires_virtue`). **Companion
`gift_policy` is `allowed`** (was `forbidden`) so a Gifted companion is legal
(`ArMDE:2872` "companions should only have The Gift if they are intended to become
magi, or … other magical traditions"); grog/mythic stay `forbidden`.
**Approximation:** "covered" = "has an `ability_score_grant` floor", a proxy for
"has a granting Virtue" — exact for the current seed; `ability.animal_ken` has no
granting Virtue, so it is takeable only via the free slot.

**Confidence** (derived, never stored: type-profile default + `ConfidenceBonus`
effects, via `effective::confidence`):

> `ArMDE:2524` "Companions and Magi start with a Confidence Score of 1 and 3 Confidence
> Points … Grogs do not have Confidence Points."

`character_types.json` sets `confidence_score:1`/`confidence_points:3` on
companion/magus/mythic; grog omits (0/0). **Self-Confident** (`ArMDE:4900-4902`, Minor
General) → `confidence_bonus {score:1, points:2}` (raising the default to 2/5).

**Personality Traits** (`Entity.personality_traits`, `validate_personality_traits`):

> `ArMDE:2500-2503` "attach a value between -3 and +3 … a Major Personality Flaw
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
Hermetic / **Academic**, `ArMDE:1091-1101`; `ArMDE:1097` names Local as "the most basic"
type with the others alongside):

> `ArMDE:2514` "Characters only start with a Reputation if they choose a Virtue or Flaw
> that grants one, but all characters can develop them in play."

`Effect::GrantsReputation {kind: Option<ReputationType>, score}` authorizes one
starting Reputation; `validate_reputations` errors on any reputation beyond the
grants of its kind (`reputation_not_granted`). A grant with `kind == None` is a
**player-chosen-type** wildcard (Famous, `ArMDE:3861-3863` — "Choose … one type"): it
authorizes one Reputation of *any* type; concrete-kind grants authorize only that
type (a Reputation consumes a matching concrete slot first, then a wildcard). Seed
granters: **Infamous** (`ArMDE:6310-6312`, `{local, 4}`) and **Black Sheep**
(`ArMDE:5703-5705`, `{local, 2}`). **Correction (M5/5a):** the earlier "only Local
granters exist in Core" claim was wrong. **Hermetic Prestige** *is* in Core at
`ArMDE:4071-4073` (Hermetic Reputation level **4** — the Darius `ArMDE:2518` example's 3 is
an errata slip; the Virtue text's 4 is authoritative). The full set of
reputation-granting `creation_effect` V/F is wired in slice 5a-wire (see below).
`ReputationType::Academic` was added because six scholastic Social-Status Virtues
(Baccalaureus, Cathedral School Master, Doctor in (Faculty), Magister in
Artibus/Medicina, Failed Student) confer an "Academic Reputation" — a named type
in the source. Fluent `reputation-type-academic` (en `Academic`, de `Akademisch`).

`reputation_grants` (`effective/reputation_and_caps.rs`) returns one
`ReputationGrant { source, reputation_type, score }` per `GrantsReputation`
effect, where `source` is the id of the granting Virtue/Flaw — provenance for the
UI, not a rule value, so a granted row can say *which* V/F put it there. The app
layer passes each grant through unchanged (`reputation_grants_for_ui`,
`arm-app/src/effective_dto.rs`) and the panel renders one row per grant, a
`<select>` over `Ruleset::reputation_type_order` for the wildcard. It used to
**flatten** a wildcard into one entry per type, which offered four slots where
`validate_reputations` allows one; the counts now agree.

---

## Virtue/Flaw classification (M5/5a)

Every entry in `rules/core/virtues_flaws.json` carries a required `classification`
field (engine enum `Classification`, serde `snake_case`; no serde default — an
unclassified entry fails to load, and `tests/data_integrity.rs::every_vf_is_classified`
guards the acceptance criterion). This slice adds **no mechanical effects**; it is a
data-tagging + provenance pass whose output finalizes the slice-4/5b `Effect`
variant set and the slice-5a-wire creation-number wiring. The four disjoint
classes:

- **`narrative`** — no mechanical creation number, no in-play/derived-total
  effect, **and no mechanical clause in the cited passage at all**. Personality,
  Story, and most Social-Status V/F are narrative by design; they are **never**
  given an invented effect. No source citation is required for a narrative item.
- **`uncomputed_rule`** — the cited passage states a real mechanical rule, but one
  that is genuinely uncomputable at character-generation time, so the engine models
  nothing and the **displayed rules text is the rule's only carrier**. Like
  `narrative` it carries no `effects`; unlike `narrative` the rulebook did say
  something. The recurring shapes are **botch dice** (a table-time change to how a
  stress roll is rolled, not a sheet property), **scene- or activity-contingent
  modifiers** (terrain, time of day, what the character is doing), and **GM
  judgement / open-ended magnitudes** ("-3 or greater").
- **`creation_effect`** — changes a character-creation number/state (starting
  scores, XP grants, Confidence, Size/characteristic deltas, reputation grants,
  spell-levels, item-level budget, True Faith / Might scores, a free starting
  Supernatural Ability score, …). Includes every entry that already carries
  `effects`.
- **`in_play_effect`** — no creation-number change, but modifies an in-play/derived
  total the engine computes in slice 5i (casting / lab / penetration / magic
  resistance / combat / soak / study / aging-longevity). An entry the engine *does*
  compute something for stays `in_play_effect` even when its passage additionally
  carries an uncomputable clause — `flaw.hobbled` (ArMDE:6260-6263) encodes its
  combat penalties and also doubles botch dice, and the encoded half is what decides
  the class.

Counts over the shipped catalogue as of the 2026-09-19 phrase-screen re-sweep —
a snapshot for orientation, **deliberately not asserted in any test**
(`CLAUDE.md` → "Catalogue size is data, never code"): **narrative 335**,
**uncomputed_rule 102**, **creation_effect 125**
(36 wired via `effects` before 5a; 68 more wired in 5a-wire; **11 deferred** —
see the 5a-wire section for the itemized deferrals),
**in_play_effect 93** (all wired in 5b). Total 655. `narrative` will keep falling
as later blocks are swept; the two classes trade one-for-one.

### Why the fourth class exists

`narrative` originally carried two incompatible meanings at once. Its own
definition said "no derived-total effect" (a statement about **the engine**) and
"pure personality, story, or social-status flavor" (a statement about **the
rulebook**), and those came apart on any entry the engine cannot compute but the
book still gives a rule for. `flaw.clumsy` (ArMDE:5797-5800) is the clearest case:
its passage says "you are at -3 in all related rolls" and "roll an extra botch
die", which is emphatically not flavour, yet it was `narrative` because no `Effect`
variant fits a botch-dice change.

The consequence was that a **dropped rule and genuine flavour were
indistinguishable in the data**, so no guard could detect the former. Worse, the
V/F `summary` convention stops at the first sentence, so whether such a clause
reached the user at all depended on where the author happened to put the full stop
— `virtue.all_according_to_plan` and `flaw.careless_sorcerer` kept their botch
clause purely because it landed in sentence one, while 16 sibling entries lost
theirs. `uncomputed_rule` makes the distinction explicit and per-entry, which is
what makes the clause reachable by a guard.

The botch family reclassified `narrative` → `uncomputed_rule` (18 entries; the five
botch-carrying entries that already had `effects` stayed `in_play_effect`):
`virtue.all_according_to_plan` (ArMDE:3384-3387), `virtue.cautious_sorcerer`
(ArMDE:3555-3558), `virtue.cautious_with_ability` (ArMDE:3559-3562),
`virtue.jack_of_all_trades` (ArMDE:4155-4158), `virtue.light_touch`
(ArMDE:4307-4310), `flaw.careless_sorcerer` (ArMDE:5769-5772),
`flaw.careless_with_ability` (ArMDE:5773-5776), `flaw.clumsy` (ArMDE:5797-5800),
`flaw.cursed_guile` (ArMDE:5889-5892), `flaw.deaf` (ArMDE:5901-5904),
`flaw.evil_eye` (ArMDE:6036-6039), `flaw.fish_out_of_water_terrain`
(ArMDE:6126-6133), `flaw.jinxed` (ArMDE:6322-6325), `flaw.twilight_prone`
(ArMDE:6879-6882), `flaw.uncontrollable_strength` (ArMDE:6907-6910),
`flaw.unpredictable_magic` (ArMDE:6935-6938), `flaw.waster_of_vis`
(ArMDE:7052-7055), `flaw.weird_magic` (ArMDE:7098-7101).

`flaw.jinxed` is worth noting as the one that states the *absence* of a botch
change ("need not roll any extra botch dice"). That is still a rule — a
clarification a player needs — so it classifies the same way.

### The Flaws-block sweep (2026-09-15) — 49 further reclassifications

The botch family above was one shape of dropped rule. The **signed-modifier**
shape is far larger, and the core rulebook's whole Flaws block (ArMDE:5639-7113)
was read entry by entry to clear it. 49 Flaws moved `narrative` →
`uncomputed_rule`; each now carries the full cited passage as `description` in
both shipped locales, so the rule the engine does not compute at least reaches
the player:

`flaw.anchored_to_the_land` (ArMDE:5667-5670), `flaw.arthritis`
(ArMDE:5679-5682), `flaw.blatant_gift` (ArMDE:5711-5714),
`flaw.blatant_magical_air` (ArMDE:5715-5718), `flaw.broken_vessel`
(ArMDE:5753-5756), `flaw.brutal_artist` (ArMDE:5757-5760), `flaw.castratus`
(ArMDE:5777-5780), `flaw.clumsy_magic` (ArMDE:5801-5804),
`flaw.corrupted_abilities` (ArMDE:5847-5852), `flaw.craving_for_travel`
(ArMDE:5869-5872), `flaw.devoted_parent` (ArMDE:5950-5953), `flaw.disfigured`
(ArMDE:5980-5983), `flaw.environmental_sensitivity` (ArMDE:6024-6027),
`flaw.fickle_nature` (ArMDE:6122-6125), `flaw.flashbacks` (ArMDE:6134-6141),
`flaw.fury` (ArMDE:6194-6197), `flaw.gullible` (ArMDE:6222-6225),
`flaw.hunchback` (ArMDE:6272-6275), `flaw.hunger_for_form_magic`
(ArMDE:6276-6279), `flaw.inconstant_magic` (ArMDE:6298-6301),
`flaw.independent_craftsman` (ArMDE:6302-6305), `flaw.indiscreet`
(ArMDE:6306-6309), `flaw.inscribed_shadow` (ArMDE:6318-6321),
`flaw.lingering_injury` (ArMDE:6350-6353), `flaw.lycanthrope`
(ArMDE:6370-6377), `flaw.magic_addiction` (ArMDE:6378-6381),
`flaw.manifest_sin` (ArMDE:6396-6407), `flaw.missing_ear` (ArMDE:6430-6433),
`flaw.mute` (ArMDE:6472-6475), `flaw.necessary_realm_aura_for_ability`
(ArMDE:6480-6487), `flaw.night_terrors` (ArMDE:6488-6495), `flaw.nocturnal`
(ArMDE:6504-6507), `flaw.offensive_to_beings` (ArMDE:6524-6533),
`flaw.poor_concentration` (ArMDE:6602-6605), `flaw.primitive_equipment`
(ArMDE:6630-6633), `flaw.raised_in_the_gutter` (ArMDE:6650-6653),
`flaw.repellent` (ArMDE:6679-6682), `flaw.rolling_stone` (ArMDE:6699-6702),
`flaw.sleep_disorder` (ArMDE:6745-6750), `flaw.social_handicap`
(ArMDE:6771-6774), `flaw.surgical_empiricus` (ArMDE:6811-6814),
`flaw.susceptibility_to_sunlight` (ArMDE:6827-6830), `flaw.unbearable_to_beings`
(ArMDE:6891-6898), `flaw.uncertain_faith` (ArMDE:6899-6906),
`flaw.uninspirational` (ArMDE:6919-6922), `flaw.unlucky` (ArMDE:6927-6930),
`flaw.vengeful_powers` (ArMDE:6959-6976), `flaw.warped_senses`
(ArMDE:7027-7051), `flaw.weak_personality` (ArMDE:7076-7079).

`flaw.missing_ear` is the one that gained no `description`: its passage is a
single sentence that its `summary` already carries in full, so a `description`
would be a byte-identical duplicate. Same reading as
`flaw.susceptibility_to_divine_power` in the previous pass.

**Four of these are not simple modifiers, and flattening them would misstate the
rule.** Worth naming because the class is called "uncomputed *rule*", not
"uncomputed penalty":

- `flaw.weak_personality` (ArMDE:7078) is a **cap plus a roll ceiling**: "all
  Personality Traits must be between +1 and -1", and "treat any roll above 6 as
  merely 6". Only the trailing "-1 or worse … modifier" is a penalty at all.
- `flaw.uninspirational` (ArMDE:6921) is likewise a **cap on two
  Characteristics** — "His Presence and Communication may not be greater than 0"
  — beside its -3.
- `flaw.fickle_nature` (ArMDE:6124) is a **creation-time grant**, not a penalty:
  "Select a Personality Trait at +4, and its opposite at +4."
- `flaw.lingering_injury` (ArMDE:6352) is a **formula**: the penalty is
  multiplied by `1 + Decrepitude Score`, so it is not a fixed number at all.

All four are `uncomputed_rule` rather than `creation_effect`/`in_play_effect`
because the engine has no representation for any of them: Personality Traits are
free-form user entries on `types.rs::Entity::personality_traits` with no
granting effect and no cap validation, and no `Effect` variant multiplies by
Decrepitude. Inventing one is what `CLAUDE.md` → "Rules provenance" forbids.
These are the four best candidates if a later slice wants to *compute*
something here — see `docs/open-todos.md`.

**Two entries in the block trip the guard's screen and are correctly
`narrative`:** `flaw.overconfident_major` and `flaw.overconfident_minor`
(ArMDE:6562-6565). Their passage contains the word "botch", but as a bare verb
inside a roleplaying instruction ("If you actually botch, you come up with some
rationalization"); it states nothing about botch dice and no number. They are
recorded with their reading in
`tests/uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`, and
`exempted_entries_still_trip_the_screen` stops that row outliving its reason.

With the block clean, the third assertion of the uncomputed-clause guard —
*"a `narrative` entry whose cited passage carries a mechanical token has dropped
a rule"* — **landed green**, scoped to this block via
`tests/uncomputed_clauses.rs::SWEPT_BLOCKS`, mirroring the incremental-roots
pattern `rulebook_citations.rs` uses. The Virtues block (ArMDE:3360-5282) is not
swept and is deliberately outside the scope.

### The phrase-screen re-sweep (2026-09-19) — 19 further reclassifications

Both sweeps above were run with a two-token detector: a **signed number** and a
**botch term**. The books state plenty of mechanics in neither. A cap ("up to 12
human-sized animals"), a target number ("against an Ease Factor of 15"), a
formula ("equal to a tenth of his Creo Vim Lab Total"), a rounding direction, an
absolute ("cannot die as a result of wounds or old age"), a step counted in
magnitudes — each is as mechanical as a `+3`, and every one of them read as pure
flavour. So "the Flaws block is clean" was true only of the shapes the screen
could see: **thirteen** of the nineteen entries below sit inside that block.

`tests/uncomputed_clauses.rs::MECHANICAL_PHRASES` adds the third screen, and both
blocks were re-read under it. The reclassified entries, each now carrying the
full cited passage as `description` in both shipped locales:

`flaw.bound_to_realm` (ArMDE:5731-5734), `flaw.curse_of_slander`
(ArMDE:5881-5884), `flaw.the_falling_evil` (ArMDE:6076-6079),
`flaw.flawed_powers` (ArMDE:6146-6149), `flaw.raised_from_the_dead`
(ArMDE:6646-6649), `flaw.realm_stigmatic` (ArMDE:6654-6658),
`flaw.servant_of_the_land` (ArMDE:6717-6720), `flaw.stigmatic_catalyst`
(ArMDE:6783-6786), `flaw.university_dean` (ArMDE:6923-6926), `flaw.viaticarus`
(ArMDE:6977-6984), `flaw.vulnerable_to_folk_tradition` (ArMDE:7011-7014),
`virtue.capo` (ArMDE:3545-3548), `virtue.command_animals` (ArMDE:3575-3578),
`virtue.death_prophecy` (ArMDE:3639-3644), `virtue.enduring_magic`
(ArMDE:3755-3758), `virtue.exotic_casting` (ArMDE:3775-3778),
`virtue.extractor_of_form_vis` (ArMDE:3779-3782),
`virtue.flexible_formulaic_magic` (ArMDE:3891-3894), `virtue.folk_magic`
(ArMDE:3907-3920).

**Shapes worth naming, because none of them is a modifier:**

- `virtue.command_animals` (ArMDE:3577) is a **hard cap on a count** — "the
  character may command up to 12 human-sized animals" — with an explicitly
  open-ended clause beneath it for smaller animals.
- `virtue.death_prophecy` (ArMDE:3641) is an **absolute**: "You heal normally,
  but cannot die as a result of wounds or old age." It overrides two systems the
  engine does model (wounds, aging) and carries no number at all.
- `virtue.extractor_of_form_vis` (ArMDE:3781) is a **formula with a rounding
  direction** — "a number of pawns of Vis equal to a tenth of his Creo Vim (Form)
  Lab Total (round up)".
- `virtue.enduring_magic` (ArMDE:3757) is a **die-typed multiplier**: "The
  storyguide secretly rolls a simple die; multiply the spell's normal duration by
  the number rolled."
- `virtue.flexible_formulaic_magic` (ArMDE:3893) is a **parameter-swap rule**
  priced in magnitudes: raise or lower the casting level by one magnitude to
  shift one of Range/Duration/Target category/Target size by one step.
- `flaw.university_dean` (ArMDE:6925) and `flaw.flawed_powers` (ArMDE:6148) are
  **selection prerequisites the engine does not express** — a minimum age of 40
  and a Flaw exclusion in the first, "at least one Major Supernatural Virtue" and
  a Hermetic-Flaw exclusion in the second.
- `flaw.servant_of_the_land` (ArMDE:6719) states a **budget exemption**: the
  granted Prohibition "does not count toward the character's total number of
  Virtues and Flaws".
- `virtue.folk_magic` (ArMDE:3919) is a whole **sub-system** — a Casting Total of
  `(Stamina + (Realm) Lore + Aura modifier + stress die) / 2`, a Fatigue cost per
  cast, and a 15-minute preparation minimum.

All nineteen stay `uncomputed_rule` rather than gaining an effect: no `Effect`
variant expresses a cap on commanded animals, an exemption from the V/F budget, a
minimum age, or a duration multiplier, and inventing one is what `CLAUDE.md` →
"Rules provenance" forbids.

**Four entries trip the widened screen and are correctly `narrative`**, recorded
with their readings in `tests/uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`:
`flaw.horrifying_appearance_snake_legs` (ArMDE:6264-6267, "two or more
snake-like tails" counts tails), `flaw.primogeniture_lineage` (ArMDE:6634-6637 —
its one real rule, House Verditius only, is already *computed* by the entry's
`prerequisites`; see the Prereq section above), and `flaw.true_love_major` /
`flaw.true_love_minor` (ArMDE:6871-6878, whose "equal to or better than the
player character" is the book choosing which magnitude applies — a choice the
catalogue already encodes as two entries).

The screen also needs a **left word boundary**, without which "or more" reads
itself out of "f|or more| details" and flags every entry pointing at a
supplement; `virtue.factor` (ArMDE:3793-3796) and `virtue.fidai`
(ArMDE:3877-3882) arrived that way and are not reclassified.

### D67 — the classification guard also reads `prerequisites`/`incompatible_with` (2026-09-29)

The acceptance guard above (`tests/data_integrity.rs::every_vf_is_classified`)
originally read only `effects` (plus a type profile's `required_traits`/
`forbidden_traits`) to decide whether an entry was "computed". D46
(`docs/vf-audit/decisions.md`) already held that classification follows *what*
is computed, never *where* — but the guard's own `is_computed` check was
narrower than that ruling: an entry whose only enforced constraint was a
`prerequisites` or `incompatible_with` field (a selection constraint, not an
`Effect`) still read as "computes nothing" and so could never be
`creation_effect`/`in_play_effect` under the guard, only `narrative` or
`uncomputed_rule`. D67 widens `is_computed` to read all four sources
(`effects`, `prerequisites`, `incompatible_with`, profile trait references) and
resolves the resulting edge case explicitly: **an entry may be
`uncomputed_rule` even while something else about it is computed** —
`uncomputed_rule` is exempt from the "must compute something" /
"must compute nothing" split the guard applies to the other three classes.
This is what lets `virtue.the_gift` (computed via the grog profile's
`forbidden_traits`, per D46's own worked example) and `virtue.devil_child`
(computed via its `incompatible_with`) both classify `uncomputed_rule` for a
genuinely dropped clause — the free-Virtue grant in `devil_child`'s case, the
ArMDE:2870-2876 penalties clause in `the_gift`'s — without the guard treating
the entry's other, already-enforced constraint as a contradiction.

### Roadmap corrections applied here

- **Wealthy / Poor are `narrative`.** Core defines only advancement-*season*
  effects for them (`ArMDE:5235-5237`, `ArMDE:6594-6596`); there is no creation XP-per-year
  figure, so no invented creation effect is assigned (the season effect is an M6
  life-stage concern).
- **Hermetic Prestige `creation_effect`** — `ArMDE:4071-4073`, a Hermetic
  Reputation at level **4** (the `ArMDE:2518` Darius example's 3 is an errata slip; the
  Virtue text's 4 is authoritative). The stale "not in Core / Local-only" claim in
  the Reputations section above is corrected.
- **Famous `creation_effect`** — `ArMDE:3861-3863`, a **player-chosen-type**
  Reputation at level 4 (5a-wire may add a player-selected `kind` param to
  `GrantsReputation`).

### D12/D68 — the trained gate on Hermetic Virtues and Flaws (X3, 2026-09-29)

Every `hermetic`-category entry is either **intrinsic** — it operates on The
Gift itself (`flaw.blatant_gift`, `virtue.gentle_gift`, `flaw.suppressed_gift`,
the only three exceptions) — or **trained**: it operates on a Technique, Form,
spell, Casting/Lab Total, Parma Magica, Arcane Connection, certámen, or
Twilight, none of which exist before Hermetic training (D12, D68.1). Training
is stated twice per entry, with a guard
(`tests/x3_trained_gate.rs::trained_flag_and_gate_always_agree`) that the two
never disagree: `PointItem::trained: true`, plus a `prerequisites` leaf —
`{"kind": "hermetically_trained"}`, or `{"kind": "order_member"}` where the
passage names the Order, a House, or the Gauntlet (D68.2, D68.12) — the latter
combined with any existing House leaf under `all[...]` (e.g.
`virtue.clan_ilfetu`, `virtue.the_enigma`). No `Prereq` a shipped entry once
spelled as `Has(virtue.hermetic_magus)` survives this pass (D12.3/D56); every
site normalizes to `HermeticallyTrained`.

| Value | Source |
|---|---|
| D12 reading of: every Hermetic entry except the three Gift ones requires Hermetic training | `ArMDE:2870` ("A character with The Gift... may take Hermetic Virtues and Flaws which relate to intrinsic ability rather than background or training"), `ArMDE:2880` ("some are only applicable to Hermetic magi who have already completed their training") — the "every" is D12's reading of the two passages together, not a sentence the book prints |

X3a lands the gate on the 55 Hermetic Virtues (`rules/core/virtues_flaws.json`);
the 64 Hermetic Flaws follow in X3b/X3c (`tmp/x3-scope.md`). X3b (33 Flaws,
`flaw.bound_casting_tools`…`flaw.necessary_condition`) and X3c (31 Flaws,
`flaw.painful_magic`…`flaw.weird_magic`) land the same `trained`+gate pair on
every remaining Hermetic Flaw, including the folded House leaves
`flaw.brutal_artist`/House Jerbiton, `flaw.consumed_casting_tools`/House
Verditius (X3b), and `flaw.spontaneous_casting_tools`/House Verditius (X3c),
and normalize `flaw.deficient_technique`'s prerequisite from
`Has(virtue.hermetic_magus)` to `HermeticallyTrained` (D12.3/D56). Every
Hermetic-category entry now lands in exactly one of the five buckets
`tests/x3_trained_gate.rs::hermetic_catalogue_is_fully_classified_by_x3a_or_pending`
checks. Where the added `prerequisites` leaf newly makes a still-`narrative`
entry "computed" per D67, the id is added to
`tests/data_integrity.rs::PENDING_D67_CLASSIFICATION` (23 entries) rather than
reclassified here — the same shape X3a already used for
`virtue.side_effect`/`virtue.tethered_magic`; reclassifying each from its own
passage is X2's routed work.

### D69/X7b-e — row 42 compute verdicts (2026-09-29)

Eight `docs/open-todos.md` row-42 entries move from `uncomputed_rule`-with-no-
wiring to `uncomputed_rule`-with-partial-effects (D67: an entry stays
`uncomputed_rule` whenever any stated clause is left uncomputed, whatever else
it computes). Full per-entry citation and D58 reasoning:
`tmp/x7be-verdicts.md`; engine design sketch: `tmp/x7be-handover.md`.

- **Uninspirational** (`ArMDE:6919-6922`) — "His Presence and Communication may
  not be greater than 0" is a new `Effect::CharacteristicMax { characteristic,
  max }`, folded by `effective/characteristic.rs::characteristic_cap`, which
  became **entity-aware** (previously ruleset-global only). Two instances (Pre,
  Com), both `max: 0`. The `max` binds the **actual** score (bve S1,
  2026-10-03): `characteristic_cap` subtracts the free delta
  (`characteristic_score_bonus`) from it, so Sidhe Faerie Blood's +1 Presence
  caps the bought score at -1 and Monstrous Blood's -1 lets it reach +1; the
  base +3 cap is never shifted. `validation/scores.rs::validate_characteristics`
  also checks the unbought Characteristics (no map entry, bought 0), so a free
  +1 on an untouched Presence is flagged. The four named Abilities' -3
  ("Leadership, Charm, Intrigue, Etiquette") reuse the existing
  `Effect::AbilityRollMod`. The "Personality Rolls" clause has no Ability to
  attach to and stays text.
- **Weak Personality** (`ArMDE:7076-7079`) — "all Personality Traits must be
  between +1 and -1" is a new `Effect::PersonalityTraitRange { max }`, read by
  `validation/scores.rs::validate_personality_traits` as a per-entity override
  that REPLACES (not composes with) the universal ±3/Major-Personality-Flaw-±6
  scheme. "No other Personality Flaws or Virtues/Flaws that grant Personality
  Traits" is data-only: `excluded_if_holds: ["grants_personality_trait"]` plus
  `Effect::ForbidsItemCategory { category: "personality" }` — both already-built
  machinery whose doc comments (`types.rs`) name this entry as the intended
  consumer. The roll-ceiling-at-6 clause stays text.
- **Fickle Nature** (`ArMDE:6122-6124`) — "Select a Personality Trait at +4,
  and its opposite at +4" is a new `Effect::RequiresPersonalityTraitPair
  { value }`, read by the new
  `validation/scores.rs::validate_personality_trait_pairs`, which requires at
  least two distinct `personality_traits` entries at exactly `value`. New
  issue code `fickle_nature_trait_pair_missing`. The "opposite" pairing itself
  is unverified free text (D61's shape — the passage's own list is
  illustrative, not closed).
- **Lingering Injury** (`ArMDE:6350-6352`) — a "–1 penalty to all rolls
  involving physical activity" that "worsens with age, so multiply whatever
  the penalty is by 1 + (Decrepitude Score)" is a new
  `Effect::DecrepitudeScaledRollMod { amount }` and a new
  `ModifierFamily::PhysicalActivity`, folded in `derived.rs::in_play_mods` as
  `amount * (1 + decrepitude_score)`. The aggravated (-3) alternative stays
  text: whether a wound was caused by a botch is not tracked anywhere on
  `Entity`.
- **Servant of the (Land)** (`ArMDE:6717-6720`) — "the character has the Minor
  Personality Flaw: Prohibition, but this does not count toward the
  character's total number of Virtues and Flaws" is the existing
  `Effect::GrantsSelection { items: ["flaw.prohibition"] }` (a granted item is
  already free of the point budget by construction) — no new engine
  capability. Stays `uncomputed_rule`, not `creation_effect`: the passage's
  curse-on-failure clause is a GM-adjudicated in-play consequence with no fixed
  sheet number.
- **University Dean** (`ArMDE:6923-6926`) — "must have the Virtue Doctor in
  (Faculty), be at least 40 years old, and can not have the Poor Flaw or any
  other Flaw that grants a Bad Reputation" is a new `Prereq::AgeMin(u32)`
  (`ArMDE:6923-6926`, evaluated against `Entity::age`; unset age is
  `Tri::Unknown`, mirroring `Prereq::House`), combined under `all[...]` with
  the existing `Prereq::Has(virtue.doctor_in_faculty)`; `incompatible_with:
  ["flaw.poor"]` (added symmetrically to `flaw.poor` too, per the
  incompatibility-symmetry guard) and `excluded_if_holds: ["grants_reputation"]`
  — the latter two are data-only against already-built machinery.
- **Flawed Powers** (`ArMDE:6146-6149`) — "must have at least one Major
  Supernatural Virtue to take this Flaw" is a new `Prereq::HasCategoryAtMagnitude
  { category, magnitude, item_kind }`, the magnitude/kind-filtered twin of
  `Prereq::HasCategory` (D68.4's scheduling note: X7b-e owns this
  prerequisite; X4 keeps only the separate `requires_hermetic_arts` import
  filter). `item_kind` is required because `categories` is shared free-form
  vocabulary between Virtues and Flaws — `flaw.raised_from_the_dead` is
  itself a Major `supernatural`-category Flaw, so a kind-blind test would be
  wrongly satisfied by it.
- **Raised from the Dead** (`ArMDE:6646-6649`, D69.1) — "at least three
  Warping points, plus one Warping point for every year that has passed since
  you were resurrected" is a new `Effect::WarpingGrantParam { param,
  base_points }` (D64's Abandoned Apprentice `years_completed` precedent): a
  new `years_since_resurrection` number parameter, folded by
  `effective/warping.rs::warping_grant_points_in` as `base_points + years`
  (years defaults to 0 when unanswered). "A level 4 reputation in the area
  where the miracle occurred" reuses the existing `Effect::GrantsReputation
  { kind: local, score: 4 }`. The ongoing "+1 Warping point every year you
  continue living" clause is NOT creation-time (an accrual, not a constant)
  and stays text.

| Value | Source |
|---|---|
| Uninspirational's Presence/Communication cap of 0 | `ArMDE:6919-6922` |
| Uninspirational's four named Abilities' -3 | `ArMDE:6919-6922` |
| Weak Personality's ±1 trait range | `ArMDE:7076-7079` |
| Fickle Nature's matched-pair value (+4) | `ArMDE:6122-6124` |
| Lingering Injury's -1 base, ×(1 + Decrepitude Score) | `ArMDE:6350-6352` |
| University Dean's age floor (40) | `ArMDE:6923-6926` |
| Flawed Powers' Major-Supernatural-Virtue prerequisite | `ArMDE:6146-6149` |
| Raised from the Dead's 3-point Warping floor + 1/year + level-4 Reputation | `ArMDE:6646-6649` |

### D44/D68.8/D69/D23/D33 — X4 incompatibilities, twin-pair guard, predicates (2026-09-30)

Full per-pair citations: `tmp/x4-verdicts.md`. Phase 2 (this pass) turns each
Phase-1 finding into data or the one small engine gap it needed.

**Wealthy/Poor closed set (F-340 family) + F-242.** Seven entries state "you
may not take the Wealthy Virtue or the Poor Flaw" (or Wealthy alone), and
`virtue.redcap` separately excludes The Gift on the same passage — all data-only,
symmetric `incompatible_with` pairs.

| Entry | Excludes | Source |
|---|---|---|
| `virtue.almogavar` | Wealthy, Poor | `ArMDE:3406` |
| `virtue.mendicant_friar` | Wealthy, Poor | `ArMDE:4494` |
| `virtue.redcap` | Wealthy, Poor, The Gift | `ArMDE:4850` |
| `virtue.turb_trained` | Wealthy, Poor | `ArMDE:5181` |
| `virtue.perfectus` | Wealthy | `ArMDE:4634` |
| `flaw.branded_criminal` | Wealthy | `ArMDE:5751` |
| `flaw.outcast` | Wealthy | `ArMDE:6540` |

`virtue.priest`'s parish-priest ban on Poor (`ArMDE:4800`, `ArMDE:4802`) is
conditional on holding the office and stays **text only** (D68.7).

**D44 plain two-sided pairs.** Twenty flat, unconditional `incompatible_with`
pairs stated in so many words, data-only:

| Pair | Source |
|---|---|
| `virtue.demonic_blood` ↔ `virtue.unaging` / `flaw.age_quickly` | `ArMDE:3661` |
| `virtue.forgettable_face` ↔ `virtue.venus_blessing` / `virtue.inspirational` | `ArMDE:3931` (Curse of Venus explicitly exempted, same sentence) |
| `virtue.strong_faerie_blood` ↔ `virtue.faerie_blood` | `ArMDE:5044` |
| `virtue.withstand_casting` ↔ `flaw.vulnerable_casting` | `ArMDE:5269` |
| `flaw.blatant_gift` ↔ `flaw.blatant_magical_air` | `ArMDE:5717` (added to the existing array alongside `unbearable_to_beings`/`gentle_gift`) |
| `flaw.bound_magic` ↔ `virtue.harnessed_magic` | `ArMDE:5729` |
| `flaw.ceremonial_spontaneous_magic` ↔ `flaw.difficult_spontaneous_magic` / `flaw.weak_spontaneous_magic` | `ArMDE:5783` (Difficult ↔ Weak themselves stay compatible — `ArMDE:5970`/`ArMDE:7088` explicitly permit combining those two) |
| `flaw.failed_student` ↔ `virtue.doctor_in_faculty` | `ArMDE:6074` |
| `flaw.night_terrors` ↔ `flaw.sleep_disorder` | `ArMDE:6492` |
| `flaw.uncertain_faith` ↔ `virtue.true_faith` | `ArMDE:6905` |
| `virtue.blood_of_the_nephilim` ↔ `virtue.the_gift`, `virtue.true_faith`, `virtue.giant_blood`, `virtue.mythic_blood`, `virtue.faerie_blood`, `flaw.age_quickly`, `flaw.lycanthrope` | `ArMDE:3517` |

**Blood of the Nephilim's category and Size clauses (F-23 residue).**
`ArMDE:3517`'s "Hermetic Virtues or Flaws" is a category, not a closed id list
(CLAUDE.md's catalogue-size invariant): `effects: [{"type":
"forbids_item_category", "category": "hermetic"}]`, the same
`Effect::ForbidsItemCategory`/`validate_category_effect_prohibitions` machinery
Weak Personality already uses. The same passage's "Virtues or Flaws that affect
your Size, such as Giant..." is open-ended (D69.5): rather than hand-enumerate a
closed list, a new `ItemPredicate::AffectsSize` ranges over every item carrying
`Effect::SizeDelta` — the five carriers already in the catalogue (`virtue.giant_blood`,
`virtue.large`, `flaw.small_frame`, `flaw.dwarf`, and Blood of the Nephilim's own
+1) are the complete sweep, verified: no other entry changes Size any other way.
`virtue.blood_of_the_nephilim` gains `excluded_if_holds: ["affects_size"]`
(Giant Blood is *also* covered by the flat D44 pair above; Dwarf/Small
Frame/Large are reached only through this predicate). The character-type
clause ("Magi and Grogs may not take this Virtue") and the Realms of Power
Methods/Powers clause are out of scope here — `Prereq::IsGrog`/X5, and not a
`virtues_flaws.json` entry, respectively.

**No Sense of Direction excludes bought Well Traveled, not granted (F-466).**
`ArMDE:6502` "incompatible with the Well Traveled Virtue" — but `virtue.lone_redcap`
GRANTS `virtue.well_traveled` for free (`Effect::GrantsSelection`). Since
`validate_incompatibilities` reads **bought-only** selections on both sides
(unlike `validate_excluded_if_holds`/`validate_category_effect_prohibitions`,
which are deliberately grant-aware, B15), a plain symmetric `incompatible_with`
pair is exactly the right shape here — it fires when Well Traveled is bought
directly and stays silent when it is only Lone Redcap's incidental grant. (An
earlier Phase-1 sketch proposed a `Prereq::Nor(Has(well_traveled))` instead,
reasoning from D23's *opposite* case — an exclusion that must ALSO reach a
granted item, which is what `Prereq::Has` resolving bought-and-granted rows is
for. That would have wrongly blocked Lone Redcap too; corrected here.)

**University Dean excludes Bad-Reputation Flaws but not its own Virtue
(F-526/Q-137 engine fix).** The data (`excluded_if_holds: ["grants_reputation"]`
on `flaw.university_dean`, `incompatible_with: ["flaw.poor"]` symmetric) already
shipped from X7b-e's row-42 pass (see above). What was missing:
`ItemPredicate::GrantsReputation::holds_for` matched ANY
`Effect::GrantsReputation`, Virtue or Flaw — so `virtue.doctor_in_faculty`
(University Dean's own required prerequisite, which grants an *academic*
Reputation) wrongly excluded itself. Fixed by adding `item.kind ==
ItemKind::Flaw` to the predicate, matching the passage's own wording ("any
OTHER FLAW that grants a Bad Reputation").

**Flawed Powers' import filter is a new predicate, not an incompatibility
(Q-138/D33/D68.4).** `ArMDE:6148` "Any Flaw that is only appropriate to
Hermetic Magic (for example, Deficient Technique or Unstructured Caster)
cannot be taken with this Flaw" restricts what `flaw.flawed_powers` may
**import**, not what its holder may also hold in her own right (D33). All four
candidate entries (`flaw.deficient_technique`, `flaw.unstructured_caster`,
`flaw.restriction`, `flaw.necessary_condition`) are `trained: true` (D12's
classification), so `ItemPredicate::Trained` cannot distinguish the two
excluded from the two importable — D68.4 amends D33's original plan and adds a
**new** `PointItem::requires_hermetic_arts` flag + `ItemPredicate::RequiresHermeticArts`,
set `true` only on Deficient Technique and Unstructured Caster.
`flaw.flawed_powers` gains a `parameters` entry (`hermetic_flaw`, domain
`item`, `require_categories: ["hermetic"]`, `exclude_if:
"requires_hermetic_arts"`) — the same `ParameterDef::exclude_if` shape
`flaw.false_power` already uses for its own narrowing, D33's intended home for
this constraint all along.

**The Major/Minor twin-pair guard becomes data-driven, not blanket (D68.8).**
`ArMDE:2814` is not a blanket source for every `_major`/`_minor` exclusion —
each pair needs its OWN passage, and a pair without one loses its exclusion.
33 candidate pairs ship both sides; `ruleset::integrity::validate_magnitude_variant_exclusivity`
used to FORCE every one of them mutually `incompatible_with` or fail to load.
A new `PointItem::skip_magnitude_variant_guard` flag (set on both sides of an
exempted pair) lets the guard skip a pair entirely rather than force it:

| Verdict | Pairs | Treatment |
|---|---|---|
| Sourced, keep hard-blocked | `major_magical_focus`/`minor_magical_focus` | unchanged — `ArMDE:4405` |
| Entailed (D44), keep hard-blocked | Outsider, True Love (Flaw), True Friend (Flaw, X2t), Amorphous, Magian Lineage | unchanged — each side's OWN text contradicts the other (`ArMDE:6554`/`ArMDE:6556`, `ArMDE:6877`, `ArMDE:6877`, `ArMDE:3412`, `ArMDE:4345`) |
| Unsourced, REMOVE | the 26 personality Flaws (Ambitious … Wrathful) + Potent Magic | `incompatible_with` deleted, `skip_magnitude_variant_guard: true` added to both sides; Potent Magic's removal is itself sourced — `ArMDE:4742` "a maga may have more than one area of Potent Magic" |
| Hedged (D16), convert to advisory | Beloved Rival | `incompatible_with` deleted, `skip_magnitude_variant_guard: true` added; `advisory_prerequisites: {"kind":"none","value":[{"kind":"has","value":"flaw.beloved_rival_<sibling>"}]}` added on each side instead — `ArMDE:5697` "the troupe **may** allow the character to take both" |

The 26 personality pairs (one row each in `rules/core/virtues_flaws.json`,
citations in `tmp/x4-verdicts.md` § 2): Ambitious (`ArMDE:5663`), Avaricious
(`ArMDE:5683`), Compassionate (`ArMDE:5809`), Compulsion (`ArMDE:5813`), Compulsive Lying
(`ArMDE:5817`), Depraved (`ArMDE:5936`), Driven (`ArMDE:5988`), Envious (`ArMDE:6016`), Gender
Nonconforming (`ArMDE:6202`), Generous (`ArMDE:6206`), Greedy (`ArMDE:6214`), Hatred (`ArMDE:6236`),
Higher Purpose (`ArMDE:6256`), Lecherous (`ArMDE:6334`), Meddler (`ArMDE:6422`), Obsessed
(`ArMDE:6520`), Optimistic (`ArMDE:6534`), Overconfident (`ArMDE:6562`), Oversensitive
(`ArMDE:6566`), Pious (`ArMDE:6586`), Proud (`ArMDE:6642`), Rebellious (`ArMDE:6659`), Reckless
(`ArMDE:6663`), Vow (`ArMDE:6989`), Weakness (`ArMDE:7090`), Wrathful (`ArMDE:7106`) — each a
generic "*Major or Minor, Personality*" header with no passage stating the two
magnitudes exclude each other.

**The same-choice constraint (D69.6) — new engine machinery.** "You may not
take Student of (Realm) and Puissant Ability for the same Lore" (`ArMDE:5054`)
and "This Virtue is incompatible with the Virtue Puissant Artes Liberales"
(`ArMDE:3364`) both forbid two selections from resolving to the SAME target
rather than forbidding the pair outright — neither a flat `incompatible_with`
(both sides are parameterized; only a shared target conflicts) nor an
`ItemPredicate` (which tests a PROPERTY of the other item, never a specific
parameter VALUE) can express this. New shape:
`PointItem::same_choice_exclusions: Vec<SameChoiceExclusion>`, each entry
naming the other item + its `other_param` key, and THIS item's own target —
either `fixed_target` (Academic Concentration always means Artes Liberales,
regardless of its unrelated `subject` parameter) or `this_param` mapped
through a `via: BTreeMap<Id, Id>` (Student of Realm's `realm` parameter, mapped
to the Lore Ability that realm trains: `realm.divine` → `ability.dominion_lore`,
`realm.faerie` → `ability.faerie_lore`, `realm.infernal` → `ability.infernal_lore`,
`realm.magic` → `ability.magic_lore`). New validator
`validation::selections::validate_same_choice_exclusions` (grant-aware on both
sides, on `validate_excluded_if_holds`'s own precedent), new issue code
`same_choice_conflict` (`item`, `other`, `target` args; Fluent
`issue-same_choice_conflict` in en/de). Referential integrity
(`ruleset::integrity::validate_same_choice_exclusion`) checks `other`/`other_param`
resolve, exactly one of `this_param`/`fixed_target` is set, and `this_param`
(when set) is a parameter this item itself declares.

F-02/F-298 out of scope for a DIFFERENT reason than D69.6 solved: those two
were flagged as needing a same-**parameter-value** constraint against
`virtue.puissant_ability`'s OWN parameterization pattern before D69.6 existed
to build it; once built, both landed as this mechanism's first two consumers.

| Value | Source |
|---|---|
| Student of (Realm)'s realm→Lore Ability mapping (divine/faerie/infernal/magic) | `ArMDE:5054`, cross-referenced against `virtue.student_of_realm`'s own `ability_bonus_gated` targets |
| Academic Concentration (Subject) always targets Artes Liberales | `ArMDE:3364` |

**Not decided in this slice.** F-02 (`virtue.academic_concentration_subject`)
and F-298 (`virtue.student_of_realm`) are now BUILT (above); nothing from X4
remains undecided except what `tmp/x4-verdicts.md` § 5/§ 6 already routed
elsewhere (F-459 to numeric-effect-composition, F-23's character-type clause to
X5/`Prereq::IsGrog`).

### X7b-d — data/classification fixes landed without RULES.md rows (commit 8c2a252, 2026-09-30 follow-up)

`tmp/x7bd-handover.md`/`tmp/x7bd-verdicts.md` fixed these `corrections.md` §
3.9/3.10 findings in Phase 2 but the RULES.md rows were time-boxed out; added
here by the consuming slice.

| Finding | Entry | Fix | Source |
|---|---|---|---|
| F-21 | `virtue.blood_of_the_nephilim` | added `{"type":"aging_mod","kind":"aging_roll","amount":-5}` ("receive a –5 to Aging Rolls") | `ArMDE:3513` |
| F-49 | `virtue.demonic_blood` | added `{"type":"aging_mod","kind":"no_aging"}` + `{"type":"aging_mod","kind":"no_apparent_aging"}` ("she does not show the effects of aging; any Aging Points acquired do not get applied to her Characteristics", pairing `virtue.unaging`'s own precedent) | `ArMDE:3659` |
| F-208 | `virtue.nephilim` | added `{"type":"grants_selection","items":["virtue.strong_angelic_heritage"]}` ("You receive the Strong Angelic Heritage Virtue free"); reclassified `uncomputed_rule` → `creation_effect` | `ArMDE:4594-4597` |
| F-243/F-342 | `virtue.redcap` | added `{"type":"grants_selection","items":["virtue.well_traveled"]}` to the existing `effects` array ("you have the Well-Traveled Virtue (page 116) at no cost") | `ArMDE:4848` |
| F-249 | `virtue.ripper` | added `{"type":"power_levels","amount":70}` (a PeAn(He) 25 + a PeAn 45 effect, `power_levels` = 25+45); reclassified `narrative` → `creation_effect` | `ArMDE:4866-4869` |
| F-406 | `flaw.a_deal_with_the_devil` | added `{"type":"grants_selection","items":["flaw.plagued_by_supernatural_entity"]}` ("This Flaw includes the effects of Plagued By Supernatural Entity"); reclassified `narrative` → `creation_effect`; full verbatim `description` added both locales (D5) | `ArMDE:5905-5908` |
| F-463 | `flaw.magical_air` | added `incompatible_with: ["virtue.the_gift"]` ("You may not take this Flaw if you actually do have The Gift", `ArMDE:6382-6385`); the imported-by-reference Gifted social-penalty clause (`ArMDE:8751`) added to `description` both locales; reclassified `narrative` → `uncomputed_rule` | `ArMDE:6382-6385` |
| F-77 | `virtue.faerie_raised_magic` | added a second `{"type":"special_casting_mod","kind":"spell_improvisation"}` ("This Virtue also includes the Virtue Spell Improvisation") | `ArMDE:3829-3842` (`ArMDE:3839`) |
| F-205 | `virtue.mythic_blood` | free hereditary Minor Personality Flaw clause added to `description` both locales, verbatim per the full cited passage (`ArMDE:4573-4589`); reclassified `in_play_effect` → `uncomputed_rule` (D67: an entry with any stated rule computed nowhere is `uncomputed_rule`) | `ArMDE:4573-4589` |
| F-449 | `flaw.low_tolerance` | `derived/combat.rs::fatigue_levels` — Low Tolerance's `delta` now applies only to tiers whose base penalty is already nonzero (Weary/Tired/Dazed); Fresh/Winded stay 0 regardless of sign ("Each Fatigue level above Winded has a penalty", `ArMDE:17129` — Winded itself takes none) | `ArMDE:6366-6369`, `ArMDE:17129` |
| F-462 | `flaw.missing_eye` | added weapon-scoped `combat_mod` at −3 for each ranged weapon (`weapon.bow_long`/`bow_short`/`sling`/`javelin`/`axe_throwing`/`knife_thrown`/`stone`), mirroring `flaw.lame`'s scoped-delta pattern; the unscoped −1 stays for melee | `ArMDE:6434-6437` |
| F-306 | (confidence engine) | `effective/gift_confidence.rs::confidence` skips `ConfidenceBonus` entirely when the profile's Confidence base is 0/0 — the general fix for any Confidence-less profile ("Grogs do not have Confidence Points", `ArMDE:2522`; the 1-score/3-point Companion/Magus default is `ArMDE:2524`) | `ArMDE:2522-2524` |
| D4 | `virtue.potent_magic_major`/`_minor` | `derived.rs`/`derived/lab.rs::lab_totals` split: Potent Magic's +6/+3 moved out of the unconditional `total` into a separate figure. **Superseded by D79**: that figure was `within_focus` (gated on holding a Magical Focus, the wrong Virtue) — D79 gives Potent Magic its own `within_potent_field` figure instead, on both Lab and Casting Totals; see the "Flat lab-total bonus/penalty" bullet's "D79 correction" below | `ArMDE:4740-4781` (`ArMDE:4746` Minor +3, `ArMDE:4748` Major +6) |
| F-256 | `virtue.relic`/`virtue.powerful_relic` | new `Effect::RelicTrueFaith { score }`, NOT consumed by `effective::true_faith` (so a Relic no longer moves the bearer's own True Faith Score/MR floor). The relic-as-item mechanic itself (`ArMDE:17607-17623`: a Faith Points pool usable as Confidence, and a separate MR the relic grants its bearer) is NOT implemented — the effect is surfaced-only today | corrections.md § 3.10 |
| (n/a) | `types.rs::HealthTrack::CastingFatigue` | doc comment's sign convention corrected to "positive = fewer levels lost" (Withstand Casting +1; Vulnerable Casting/Painful Magic negative); Fluent label `derived-detail-casting_fatigue` reworded to "Casting fatigue resistance" (en) so a positive number reads as a resistance, not a cost | (label/doc only, no rulebook value) |

F-89 (`virtue.ferocity`) stays correctly unfixed here — cross-slice blocked on
F-556/X5's character-type gate (D58: "animals only" becomes permanently
unselectable by every human type once that `Prereq` lands); inventing a
standalone gate for this one entry would pre-empt that slice's design.

### D79.2 — F-256 follow-up: relic/powerful_relic compose the Relics bearer clause (2026-10-01)

`docs/vf-audit/decisions.md` D79.2, applying `docs/vf-audit/design-x7-relic-and-ct-mirror.md`
§ 1. The F-256 row above (X7b-d) left the relic-as-item mechanic
"surfaced-only"; this closes that gap data-side, without new engine
machinery (D70/X6-0: "the relic stub may land before X7b-d's F-256" —
the stub, not full machinery, per the Talisman-attunement precedent
`types.rs::TalismanAttunement`).

| Finding | Entry | Fix | Source |
|---|---|---|---|
| D79.2 | `virtue.relic` | `description` (both locales) composed via `COMPOSED_DESCRIPTIONS` (`x2_reclassification.rs`) with the "Relics" section's heading, intro paragraph and FAITH clause (own entry text unchanged); `classification` `creation_effect` → `uncomputed_rule` (D67: the bearer-MR/Confidence clause is stated, computed nowhere); `Effect::RelicTrueFaith { score: 1 }` unchanged | `ArMDE:4852-4855, :17619-17623` |
| D79.2 | `virtue.powerful_relic` | same composition and reclassification; `Effect::RelicTrueFaith { score: 3 }` unchanged | `ArMDE:4782-4787, :17619-17623` |

Divine Might and Scourging the Infernal (`ArMDE:17624-17627`) are the relic's
own defenses, not a rule about the bearer's sheet, and stay out of scope — the
composed range stops at `ArMDE:17623`. No new `Effect`, no `Entity`/save-format
change, no `SCHEMA_VERSION` bump.

### X2g — rows 318-374 (ArMDE:6382-6708, `tmp/x2g-verdicts.md`)

Thirteen reclassifications and seven description-only additions, none adding a
new computed `Effect` (all keep whatever effects/prerequisites they already
had):

| Entry | Change | Source |
|---|---|---|
| `flaw.magical_being_companion` | `narrative` → `uncomputed_rule`; full passage as `description` (the Magic Might formula "10 – Size") | `ArMDE:6386-6391` |
| `flaw.master_of_none` | `narrative` → `uncomputed_rule`; full passage as `description` (lost-XP rule) | `ArMDE:6418-6421` |
| `flaw.monastic_vows_hermetic` | `narrative` → `uncomputed_rule`; full passage as `description` ("cannot own vis"/"cannot marry"); `prerequisites: hermetically_trained` unchanged | `ArMDE:6450-6453` |
| `flaw.motion_sickness` | `narrative` → `uncomputed_rule`; full passage as `description` (doubled fatigue loss, 2-level minimum) | `ArMDE:6468-6471` |
| `flaw.necessary_condition` | `narrative` → `uncomputed_rule`; full passage as `description`; `prerequisites: hermetically_trained` unchanged | `ArMDE:6476-6479` |
| `flaw.no_hands` | `narrative` → `uncomputed_rule`; full passage as `description` ("– 5" penalty to Casting Scores, en dash + space + digit reproduced byte-for-byte per the verbatim checker) | `ArMDE:6496-6499` |
| `flaw.restriction` | `narrative` → `uncomputed_rule`; full passage as `description`; `prerequisites: hermetically_trained` unchanged | `ArMDE:6691-6694` |
| `flaw.prohibition` | `narrative` → `uncomputed_rule` (D8: `categories:["supernatural"]`); classification only — the shipped `summary` already states the whole rule | `ArMDE:6638-6641` |
| `flaw.restricted_power` | `narrative` → `uncomputed_rule` (D8); full passage as `description` (the summary stopped before the ceremony/limited-target mechanism) | `ArMDE:6687-6690` |
| `flaw.oath_of_fealty` | `narrative` → `uncomputed_rule` (D50: a hard eligibility rule no `Prereq`/`incompatible_with` enforces); full passage as `description` | `ArMDE:6512-6515` |
| `flaw.regular` | `narrative` → `uncomputed_rule` (D62 names Regular explicitly as staying text); full passage as `description` | `ArMDE:6675-6678` |
| `flaw.savantism` | `creation_effect` → `uncomputed_rule` (D67/F-510: the two `ability_score_cap_*` effects (X6b) compute the score caps, but halved starting XP, halved future Advancement Totals and the +3-not-+1 specialization roll are computed nowhere); full 2-paragraph passage as `description`; effects/parameters unchanged | `ArMDE:6703-6708` |
| `flaw.primogeniture_lineage` | `narrative` → `creation_effect` (D67: the one real clause, "can only be taken by magi of House Verditius", is already computed via `prerequisites: all(order_member, house.verditius)`); no `description` — nothing else in the passage states a rule | `ArMDE:6634-6637` |
| `flaw.magical_fascination` | classification unchanged (`creation_effect`); `description` added (the "score of 1 (but no more)" Faerie/Magic Lore cap, alongside the existing `ability_authorization` clause) | `ArMDE:6392-6395` |
| `flaw.monstrous_blood` | classification unchanged (`in_play_effect`); `description` added covering all four sub-branches (Magic Animal/Human/Spirit/Thing) — only the Magic Human branch is computed (`characteristic_score_delta_param`/`grants_reputation`, X6b), the other three stay text | `ArMDE:6454-6467` |
| `flaw.obese` | classification unchanged (`in_play_effect`); `description` added stating the non-Fatigue "-1 to rolls that involve moving quickly or gracefully" clause alongside the existing `health_mod`/fatigue_roll effect | `ArMDE:6516-6519` |
| `flaw.outlaw` | classification unchanged (`creation_effect`); `description` added stating the Reputation-2 and Martial-Abilities-authorization clauses (both already computed, neither previously displayed) plus the advisory Outlaw-follower sentence | `ArMDE:6542-6545` |
| `flaw.outlaw_leader` | classification unchanged (`creation_effect`); `description` added, same shape as `flaw.outlaw` (Reputation-3 local, Martial-Abilities authorization, and the already-landed `is_grog` Nor-prerequisite, X5) | `ArMDE:6546-6549` |
| `flaw.painful_magic` | classification unchanged (`in_play_effect`); `description` added stating the "though you do not suffer any physical damage from pain" clarifying clause alongside the existing `health_mod`/`casting_fatigue` effect | `ArMDE:6574-6577` |
| `flaw.poor_eyesight` | classification unchanged (`in_play_effect`) — **D61/OQ-4 overturn**: the attack/defense −3 stays computed via two `combat_mod` effects, but the broader "rolls involving sight" penalty (a table call, like Poor Hearing/Sharp Ears/Keen Vision) stays text; `description` added stating it | `ArMDE:6606-6609` |

Two `uncomputed_clauses.rs` mechanical-token screen misses this slice's own
newly-swept entries tripped are fixed with narrowly-scoped `S2_IDIOM`
additions rather than the bare high-collision verb ("must obey"/"must
perform" each hit an unrelated entry — `virtue.apprentice`/`flaw.vow` — so the
idiom is scoped to the fuller phrase instead): `"forbidden"`/`"verboten"`,
`"restrictions of your prohibition"`/`"befolgen"`, `"must spend"`/`"aufwenden"`,
`"special ceremony"`/`muss…durchführen`, and `"überhaupt keine"` (DE only;
the EN mirror already matches the existing bare `"cannot"`). `flaw.painful_magic`
gets a `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` row (the one Fatigue level in
pain is stated in prose, not as a digit — a screen vocabulary gap, not a
dropped rule) and `flaw.primogeniture_lineage` moves from `NO_RULE_DESPITE_TOKEN`
to the same list (its own reasoning was always an argument for `creation_effect`,
not for staying `narrative`).

### X2h + X2t — rows 375-432 (ArMDE:6709-7109, `tmp/x2h-verdicts.md`, `tmp/x2t-handover.md`)

The last Flaws block (`flaw.secretive` through `flaw.wrathful_minor`), plus
D60.3's True Friend twin. 21 reclassifications (`narrative` →
`uncomputed_rule`, full cited passage as `description`), 5 more
`in_play_effect` → `uncomputed_rule` (D20, classification-only — four already
stated their rule in `summary`), 8 classification-unchanged
description-completions, and one classification *up*-grade
(`narrative` → `creation_effect`) that a concurrent slice's premise had
assumed would stay `narrative`:

| Entry | Change | Source |
|---|---|---|
| `flaw.sheltered_upbringing` | `narrative` → `uncomputed_rule`; full passage as `description` (the seven forbidden beginning Abilities) | `ArMDE:6721-6724` |
| `flaw.short_lived_magic` | `narrative` → `uncomputed_rule`; full passage as `description` (the year→moon/moon→sun/sun→Diameter duration table) | `ArMDE:6729-6732` |
| `flaw.slow_caster` | `narrative` → `uncomputed_rule`; full passage as `description` (two-round casting, with the fast-cast/Mastered/Muto Vim/Ritual exceptions) | `ArMDE:6755-6758` |
| `flaw.spontaneous_casting_tools` | `narrative` → `uncomputed_rule`; full passage as `description` — the Verditius-only eligibility is already computed (`prerequisites: all(order_member, house.verditius)`, a concurrent slice), but the casting-tools clause itself is a separate, still-uncomputed rule | `ArMDE:6779-6782` |
| `flaw.stockade_parma_magica` | `narrative` → `uncomputed_rule`; full passage as `description` (cannot suppress Parma once erected) | `ArMDE:6787-6790` |
| `flaw.stuck_in_your_ways` | `narrative` → `uncomputed_rule`; full passage as `description` (the lower-of Ability/Covenant Lore roll substitution) | `ArMDE:6791-6794` |
| `flaw.study_requirement` | `narrative` → `uncomputed_rule`; full passage as `description` (must study in presence of the Art; may take both Study Bonus and Study Requirement) | `ArMDE:6795-6798` |
| `flaw.suppressed_gift` | `narrative` → `uncomputed_rule`; full 3-paragraph passage as `description` (cannot perform Hermetic magic/improve Arts/Parma Magica while Arts still grant MR and the social penalty persists) | `ArMDE:6803-6810` |
| `flaw.unnatural_magic` | `narrative` → `uncomputed_rule`; full passage as `description` (Creo rituals have no permanent effect; cannot extract vis via Creo) | `ArMDE:6931-6934` |
| `flaw.unstructured_caster` | `narrative` → `uncomputed_rule`; full passage as `description` (all Formulaic cast as Ritual incl. vis need; no Ritual spells at all) | `ArMDE:6947-6950` |
| `flaw.vulnerable_magic` | `narrative` → `uncomputed_rule`; full 2-paragraph passage as `description` (dispelling conditions; repeatable per distinct condition; not combinable with Restrictions/Necessary Conditions) | `ArMDE:7005-7010` |
| `flaw.warped_magic` | `narrative` → `uncomputed_rule`; full passage as `description` (side-effect intensity scales with spell level) — survives its own blanket `hermetically_trained` gate (X3b/X3c) the same way `flaw.short_lived_magic`/`flaw.slow_caster` do | `ArMDE:7023-7026` |
| `flaw.tainted_with_evil` | `narrative` → `uncomputed_rule`; full passage as `description` (gaining a positive Reputation is impossible) | `ArMDE:6843-6846` |
| `flaw.wanderlust` | `narrative` → `uncomputed_rule`; full passage as `description` (two nonconsecutive seasons/year cap, D62's stays-text shape) | `ArMDE:7015-7018` |
| `flaw.unruly_air` | `narrative` → `uncomputed_rule` (D8, `categories:["supernatural"]`); full passage as `description` (Magic Resistance holders are not influenced) | `ArMDE:6939-6942` |
| `flaw.visions` | `narrative` → `uncomputed_rule` (D8); full passage as `description` (visions come purely at the storyguide's discretion) | `ArMDE:6985-6988` |
| `flaw.slow_power` | `narrative` → `uncomputed_rule` (D8); full passage as `description` (repeatable but not more than once per single power — `max_per_target` still unenforced) | `ArMDE:6759-6762` |
| `flaw.susceptibility_to_warping` | `narrative` → `uncomputed_rule` (D8); full 3-paragraph passage as `description` (one extra Warping Point per Realm already gaining one that year) | `ArMDE:6831-6838` |
| `flaw.vow_major`/`flaw.vow_minor` | `narrative` → `uncomputed_rule`; same shared passage as `description` on both (the guaranteed-atonement clause; the Major variant's own "vow to DO something, not refrain" constraint) | `ArMDE:6989-6992` |
| `flaw.tragic_life` | `narrative` → `uncomputed_rule` (OQ-6 overturn of its own `NO_RULE_DESPITE_TOKEN` row — that row addressed only the "cannot" token, missing ArMDE:6859's separate sinful-Personality-Trait creation-time instruction); full 5-paragraph passage (incl. the five-sources-of-hope bullet list) as `description` | `ArMDE:6855-6870` |
| `flaw.short_ranged_magic` | `in_play_effect` → `uncomputed_rule` (D20/OQ-2: a numerically-correct surfaced effect with no text stating which two halvings apply is not enough); classification-only — already had a full verbatim `description` from an earlier slice | `ArMDE:6737-6740` |
| `flaw.susceptibility_to_divine_power`/`_faerie_power`/`_infernal_power`, `flaw.weak_magic_resistance` | `in_play_effect` → `uncomputed_rule` (D20, same "number right, text missing" shape); classification-only — each `summary` already states its own figures | `ArMDE:6815-6818`, `ArMDE:6819-6822`, `ArMDE:6823-6826`, `ArMDE:7068-7071` |
| `flaw.short_of_breath` | classification unchanged (`in_play_effect`); `description` added (the −3 Stamina-roll-to-avoid-fatigue penalty, already fully computed via `health_mod`) | `ArMDE:6733-6736` |
| `flaw.vulnerable_casting` | classification unchanged (`in_play_effect`); `description` added, full 4-paragraph passage (the extra-Fatigue-loss-per-instance rule stays computed; the missing text was the Vulnerable/Withstand resolution ORDER) | `ArMDE:6993-7004` |
| `flaw.usurer` | classification unchanged (`creation_effect`); `description` added (F-544: the "ten pounds of silver" interest-income figure, alongside the already-computed Reputation grant) | `ArMDE:6951-6954` |
| `flaw.warped_by_magic` | classification unchanged (`creation_effect`); `description` added (F-544: the "spend experience points on Magic Lore" creation-time grant, alongside the already-computed Warping/`ability_authorization` effects) | `ArMDE:7019-7022` |
| `flaw.weak_enchanter` | classification unchanged (`in_play_effect`); `description` added (F-544: apply the Deficiency first, then halve the remainder — the ordering the bare `magic_total_halving` effect doesn't state) | `ArMDE:7060-7063` |
| `flaw.weak_magic` | classification unchanged (`in_play_effect`); `description` added (F-544: halve Penetration Total AFTER subtracting spell level/Arcane Connection adjustments, not halve the Casting Total) | `ArMDE:7064-7067` |
| `flaw.weak_scholar` | classification unchanged (`in_play_effect`); `description` added (F-544: the −6 penalty applies specifically to Lab Totals from others' Lab Texts) | `ArMDE:7080-7083` |
| `flaw.weak_spontaneous_magic` | classification unchanged (`in_play_effect`); `description` added (F-544: ceremonial casting remains available despite the Casting Score halving) | `ArMDE:7084-7089` |
| `flaw.true_love_major`/`flaw.true_love_minor` | **`narrative` → `creation_effect`** (D67, same shape as `flaw.primogeniture_lineage`, X2g): the one genuinely mechanical clause — the Major/Minor magnitude choice — is already fully computed via the two-entry split plus mutual `incompatible_with`; the rest is Story-Flaw fiction. No `description` owed. This overturns a concurrent slice's premise that True Love would stay `narrative` (flagged in `tmp/x2h-verdicts.md` for X2t) | `ArMDE:6871-6878` |

D60.3 (X2t): `virtue.true_friend_pc`, `flaw.true_friend_major`,
`flaw.true_friend_minor` are new ids copying their True Love twins'
mechanics, categories, classification, and source *verbatim* — there is no
separate `#### True Friend` heading in either language, only a rename
sentence inside True Love's own passage in each (`ArMDE:5177` for the PC
Virtue, `ArMDE:6875` for the Flaw pair). Per the True Love reclassification
above, the Flaw pair copies the FINAL (`creation_effect`) classification, not
the `narrative` one an earlier premise assumed. `virtue.true_friend_pc`'s
`description` is `virtue.true_love_pc`'s own, copied byte-for-byte —
including its own "This Virtue may be renamed 'True Friend'" sentence, which
reads as redundant once the entry is itself named that, but is what
verbatim fidelity means here (no separate passage exists to draw different
text from). `rules/i18n/de/source_anchors.json` gets matching rows for all
three (same anchors as their twins: `wahre-liebe`, `wahre-liebe-sc`), and
`x4_incompatibilities.rs::ENTAILED_TWIN_PAIRS` gets the True Friend pair
alongside True Love's own (same D44 entailment, same cited passage).

Nine `uncomputed_clauses.rs` mechanical-token screen misses this slice's
reclassifications tripped are fixed with narrowly-scoped `S2_IDIOM`
additions, several needed in both locales for different reasons per language
(a German verb conjugation the existing stem missed, a subordinate clause's
verb-final word order, or simply a phrase with no mirror yet): `"two
rounds"`/`"zwei Runden"` (slow_caster), `"the lower of"`/`"niedrigeren
Wert"` (stuck_in_your_ways), `"additional Warping Point"`/`"zusätzlichen
Verzerrungspunkt"` (susceptibility_to_warping), `"not influenced"`/bounded
`"nicht … beeinflusst"` (unruly_air), `"purely at the storyguide's
discretion"`/`"ausschließlich nach dem Ermessen des Spielleiters"` (visions —
narrowed to the fuller phrase specifically because the bare "storyguide's
discretion"/"Ermessen des Spielleiters" also hits three unrelated,
already-settled narrative entries: `flaw.demonic_familiar`, `flaw.favors`,
`virtue.latent_magic_ability`), `"rather than refrain"`/`"anstatt etwas zu
unterlassen"` (vow), `"according to the level of the spell"`/`"zunehmender
Intensität"` (warped_magic), `"are not affected"` EN-only (short_lived_magic
— the existing `"not affected by"` idiom requires an object this passage
doesn't have), `"not more than"` EN-only (slow_power — the base list only has
"no more than"), `"may not learn"` EN-only (unstructured_caster), `"ten
pounds of silver"`/`"zehn Pfund Silber"` (usurer), `"experience points on
Magic Lore"`/`"Erfahrungspunkte auf Magiekunde"` (warped_by_magic), a DE-only
`\bhalbier` stem widening the existing `\bhalbiert` idiom to cover the
imperative/2nd-person conjugations `short_ranged_magic`/`weak_magic` actually
use, a DE-only `"nicht als Anfangsfertigkeiten nehmen"` (sheltered_upbringing
— the German modal/negation gap exceeds `DE_MODAL_NICHT`'s 40-character
bound), a DE-only `"kann nur von … gewählt werden"` (spontaneous_casting_tools
— neither existing "kann nur für"/"darf nur von" idiom covers this verb
pairing), and a DE-only `"keine alternative … kann"` (tragic_life — the
subordinate clause's modal verb sits AFTER "keine", the reverse of the
existing bounded-gap idiom's word order; narrowed to "keine alternative"
specifically because the bare form also closed `PENDING_DROPPED_CLAUSE`'s
`virtue.redcap` row for an unrelated reason).

`tests/fixtures/s1_before_offenders.json` drops its `flaw.true_love_major`/
`flaw.true_love_minor` rows: both were recorded `true` (tripped the screen)
back when the pair was still `narrative`, and `regex_screen_never_loses_an_s1_recorded_flag`
only tracks entries `compute_s1_offender_set` still classifies `narrative`
today — once reclassified, the pair leaves that population entirely, which
is a data change this slice made deliberately, not a screen regression.

### In-play effect families (definitive input to slice 4 / 5b)

- **Magical Focus (major/minor)** — `virtue.major_magical_focus` (ArMDE:4399-4422), `virtue.minor_magical_focus` (ArMDE:4536-4557), `virtue.mythic_blood` (ArMDE:4573-4589)
- **Flat casting-total bonus/penalty** — `virtue.method_caster` (ArMDE:4524-4527), `flaw.poor_formulaic_magic` (ArMDE:6610-6613), `flaw.afflicted_tongue` (ArMDE:5655-5658), `virtue.life_boost` (ArMDE:4295-4298), `virtue.leper_magus` (ArMDE:4249-4252) (`flaw.corrupted_spells`, ArMDE:5859-5864, left this list in Phase 2 C5c/D15; `virtue.cyclic_magic_positive`/`flaw.cyclic_magic_negative`/`virtue.special_circumstances`/`virtue.ways_of_the_land` left it in F2/D61 — see **Berserk / Ways of the Land / Cyclic Magic / Special Circumstances** below)
- **Spontaneous-magic casting modifier** — `flaw.weak_spontaneous_magic` (ArMDE:7084-7089), `virtue.diedne_magic` (ArMDE:3675-3682), `virtue.faerie_raised_magic` (ArMDE:3829-3842), `virtue.spell_improvisation` (ArMDE:5002-5005), `virtue.life_linked_spontaneous_magic` (ArMDE:4299-4306)
- **Art-halving (Technique / Form)** — `flaw.deficient_technique` (ArMDE:5913-5915), `flaw.deficient_form` (ArMDE:5909-5912)
- **Circumstantial casting/lab penalty (surfaced)** — `flaw.deleterious_circumstances` (ArMDE:5917-5920), `flaw.environmental_magic_condition` (ArMDE:6020-6023), `flaw.short_ranged_magic` (ArMDE:6737-6740), `flaw.disjointed_magic` (ArMDE:5972-5975), `flaw.the_constant_expression` (ArMDE:5821-5838)

  **X2f Phase 2/D20/D67 correction:** three of these five —
  `flaw.deleterious_circumstances`, `flaw.disjointed_magic` and
  `flaw.environmental_magic_condition` — are `uncomputed_rule`, not
  `in_play_effect`. D20 rules that a bare `special_casting_mod: circumstantial`
  label with no number is a surfaced-only effect that tells the player
  nothing, so the rule's only real carrier is `description`; D67 lets the
  entry keep the `special_casting_mod` effect it still computes regardless of
  the classification. `flaw.short_ranged_magic` and
  `flaw.the_constant_expression` are unaffected by this slice.
- **Doubled aura penalties (surfaced)** — `flaw.susceptibility_to_divine_power` (ArMDE:6815-6818), `special_casting_mod { doubled_aura_penalty }`

  > `ArMDE:6817` — "You are especially sensitive to the Dominion and suffer twice
  > the normal penalties (such as **spellcasting modifiers and botch dice**) to your
  > magic when in a Divine aura."

  Magic Resistance is never mentioned, and the round-5 audit moved this Flaw off
  `magic_resistance_mod` for exactly that reason — it had been filed there on the
  strength of the shared "Susceptibility to …" name, while only its two siblings'
  passages support it (see **Magic-resistance modifier** below). The Aura Modifier
  is a term of the Casting Score (`ArMDE:9089` — "CASTING SCORE: Technique + Form
  + Stamina - Encumbrance + Aura Modifier"), so the mechanic is
  casting-side; it is **surfaced, not simulated**, because the engine models
  neither an aura of a foreign realm nor botch dice, and the rule carries no number
  of its own — it doubles whatever the scene's aura rating happens to be. It gets
  its own kind rather than reusing `circumstantial` because the passage states a
  precise mechanic (a *doubling*, of the *aura's* penalties) that "Circumstantial"
  would throw away in the one place the player reads it. The value the engine
  cannot compute lives in the Flaw's own rules text in `rules/i18n/<lang>/`.
- **Flat lab-total bonus/penalty** — `virtue.adept_laboratory_student` (ArMDE:3368-3371), `virtue.inventive_genius` (ArMDE:4151-4154), `flaw.creative_block` (ArMDE:5873-5876), `flaw.weak_scholar` (ArMDE:7080-7083), `virtue.potent_magic_major` (ArMDE:4740-4781), `virtue.potent_magic_minor` (ArMDE:4740-4781), `virtue.cyclic_magic_positive` (ArMDE:3635-3638), `flaw.cyclic_magic_negative` (ArMDE:5893-5896)

  **X7a/D4 correction**: "flat" here means only "an unconditional `LabTotalMod`
  effect in the JSON, read by D1's `effective::lab_total_mod` for
  `spell_level_cap`." As of X7a it is no longer true for the **in-play** Lab
  Total grid: `adept_laboratory_student`, `weak_scholar` and
  `cyclic_magic_positive` never apply there (D4 — their condition cannot hold
  at character generation), and `cyclic_magic_negative` applies there unless
  its `cycle` parameter is `cycle.seasonal` (D52). Only `inventive_genius` and
  `creative_block` stay flat in **both** consumers. See
  `derived.rs::in_play_lab_total_mod`.

  **X7a-refactor correction**: which carriers are excluded/gated is now DATA on
  the `LabTotalMod` effect itself, never an id the engine hardcodes.
  `Effect::LabTotalMod` (`types.rs`) carries a `scope: LabTotalModScope`
  (`in_play_grid` default / `within_potent_field_only` (D79; named
  `within_focus_only` before it) / `never_at_creation`) and an
  optional `suppressed_when: ParamGate`; `adept_laboratory_student`,
  `weak_scholar` and `cyclic_magic_positive` set
  `scope: "never_at_creation"`, and `cyclic_magic_negative` sets
  `suppressed_when: { "param": "cycle", "equals": "cycle.seasonal" }`
  (`rules/core/virtues_flaws.json`). `derived.rs::in_play_lab_total_mod` reads
  these fields off each carrier's effect rather than matching against an id
  list; `lab_total_mod_carriers_match_the_d4_table` is the enumeration test
  that still stands in for an exhaustive-match safeguard.

  **X7b-d/D4 correction (8c2a252)**: both Potent Magic entries no longer stay
  "flat" either. `ArMDE:4746`/`ArMDE:4748` ("Minor Potent Magic covers the same
  narrow fields as a Minor Magical Focus, and grants a +3 bonus to Lab Totals
  and Casting Score" / Major, +6) is a within-field-only bonus, not an
  unconditional one. D1's `spell_level_cap` fold is unaffected (it still reads
  `lab_total_mod` unconditionally, matching every other carrier — see the
  `LabTotalMod` row).

  **D79 correction**: the within-field gate is **Potent Magic's own field, not
  a Magical Focus**, and this entry's `scope` and doc comments were renamed
  accordingly — `WithinFocusOnly`/`"within_focus_only"` is now
  `WithinPotentFieldOnly`/`"within_potent_field_only"`
  (`LabTotalModScope`, `types.rs`). Before D79, `in_play_lab_total_mod_within_focus`
  folded this amount into `derived/lab.rs::lab_totals`'s `within_focus` figure
  — i.e. gated on holding a **Magical Focus**, the wrong Virtue, and leaking
  the bonus into a different figure whenever both were held. D79 gives it its
  own figure instead: `in_play_lab_total_mod_within_potent_field`/
  `InPlayMods::lab_mod_within_potent_field` (MAX across carriers, not sum —
  ArMDE:4742, "only one Potent Magic Virtue applies to any single activity")
  feed `LabTotal::within_potent_field` alone, gated on
  `InPlayMods::has_potent_magic`, independent of `has_focus`. The **Casting
  Total mirror**, flagged above as "not yet split", also landed under D79:
  `Effect::CastingTotalMod` gained `potent_field_only: bool`
  (`#[serde(default)]`, orthogonal to the existing cast-type `scope`), both
  Potent Magic entries set it `true`, and `derived/casting.rs::casting_totals`
  folds it into a separate `InPlayMods::casting_mods_within_potent_field`
  (also MAX, not sum) feeding `CastingTotal::within_potent_field` — never the
  unconditional `casting_mods` fold that the base and within-focus figures
  read. `derived/casting.rs::spell_casting_total` combines
  `SpellSelection::within_focus` and the new
  `SpellSelection::within_potent_field` independently via
  `formulaic_casting_score`'s `focus`/`potent` parameters, so a spell marked
  under both gets the Magical-Focus doubling **and** the Potent Magic bonus
  together. Tests: `crates/arm-rules/tests/d79_potent_magic.rs`.

  `virtue.aristotelian_training` (ArMDE:3440-3443) is no longer in this list:
  X7a deleted its `lab_total_mod` effect entirely (D4 — its condition can
  never be satisfied by anything this app models, not even D1's generous flat
  fold), so the +1 is now purely textual in `description`.

  **Two items were listed here and belong to neither family.** `flaw.disjointed_magic`
  carries `special_casting_mod { circumstantial }`, never a `lab_total_mod`, and
  `ArMDE:5974` states no Lab-Total number at all — it is filed under
  `SpecialCastingMod` below, which is its only correct home.
  `flaw.the_constant_expression` carried `lab_total_mod −3` until the round-4 audit,
  but the passage its `source` range points at gives no Lab-Total number either:

  > `ArMDE:5831` — "The constant expression of magic also makes laboratory work
  > inherently risky. Any laboratory the maga works in is treated as having a free
  > Flaw providing a **Safety penalty of –3** (see page 288)."

  > `ArMDE:11576` — "The Safety score **subtracts its value from the number of botch
  > dice** on all lab activities."

  Safety is one of the eight laboratory Characteristics (`ArMDE:11502`), not a term of
  the Lab Total, so the Flaw makes lab work botch-prone and imposes no Lab-Total
  penalty whatsoever. The engine models no laboratory and has no botch-dice concept,
  so the rule is **recorded, not simulated**: the item carries
  `special_casting_mod { circumstantial }` (surfaced) and its own rules text carries
  the −3 Safety plus the Warping-Score botch dice on Ritual/Ceremonial casting
  (`ArMDE:5829`). Inventing a `safety_mod` variant with no consumer would be
  implementing a subsystem the app does not have, which is the opposite of what
  `CLAUDE.md` → "Rules backed by source, never memory" asks for. The machine-checkable
  guard that catches this class is
  `rules_source_provenance.rs::every_guarded_effect_cites_a_passage_that_names_its_own_mechanic`.
- **Lab-total halving** — `flaw.weak_enchanter` (ArMDE:7060-7063), `flaw.difficult_longevity_ritual` (ArMDE:5962-5965)
- **Ritual effective-level bonus** — `virtue.mercurian_magic` (ArMDE:4514-4523)
- **Penetration-total modifier** — `flaw.weak_magic` (ArMDE:7064-7067)
- **Magic-resistance modifier** — `flaw.flawed_parma_magica` (ArMDE:6142-6145), `flaw.limited_magic_resistance` (ArMDE:6346-6349), `flaw.weak_magic_resistance` (ArMDE:7068-7071), `flaw.susceptibility_to_faerie_power` (ArMDE:6819-6822), `flaw.susceptibility_to_infernal_power` (ArMDE:6823-6826), `virtue.commanding_aura` (ArMDE:3579-3596)

  **Weak Magic Resistance halves nothing** — corrected in the round-5 audit, where
  it had been carrying `magic_total_halving { magic_resistance }` and was therefore
  halving the carrier's Magic Resistance on **every Form, unconditionally**:

  > `ArMDE:7070` — "Any form of Magic Resistance you generate is much weaker under
  > relatively common circumstances which are fairly easy for an opponent to
  > utilize, such as when you are wet or facing away from the caster of the spell.
  > **If the conditions are met, do not subtract the level of the effect from the
  > casting total before calculating Penetration.**"

  The word "halve" does not appear. Normal Penetration is the Casting Total less
  the spell level (`ArMDE:7066` — "after subtracting the spell level and making any
  adjustments for the use of Arcane Connections"), so the Flaw makes an **attacker**
  skip that subtraction: the size of the effect is the level of the *incoming*
  spell and the trigger is a scene call. Neither is knowable from this character's
  sheet, and the carrier's own Magic Resistance score does not change at all — so
  **no number on the sheet may differ** because of this Flaw. The book settles the
  reading itself, glossing Clan Ilfetu's Secret Name mystery as letting someone
  "need not subtract the spell level from the Penetration total of any spells cast
  against the target, **much like the Weak Magic Resistance Flaw**" (`ArMDE:9912`).
  It is therefore encoded `magic_resistance_mod { conditional_penetration_waiver }`
  and **surfaced, not computed** — the same shape `aura_bonus` and the two realm
  susceptibilities already use for a Magic Resistance rule the flat per-Form figure
  cannot carry. Pinned by
  `data_integrity.rs::weak_magic_resistance_changes_no_number_on_the_sheet`.

  **The two realm susceptibilities are scoped, and that is why they are surfaced.**
  `ArMDE:6821` halves Magic Resistance "including Parma Magica, **against faerie
  effects**" and `ArMDE:6825` gives "only half your normal Magic Resistance score
  **against infernal effects**". Both scopes name the *attacker's* realm, which the
  flat per-Form number has no axis for, so folding them in would halve resistance
  against every attacker — wrong output. They stay `magic_resistance_mod`s listed
  at amount 0, with the halving, the Stamina roll (`ArMDE:6821` and `ArMDE:6825`) and the
  Infernal illness (-1 on all rolls) carried in each Flaw's own rules text in
  `rules/i18n/<lang>/`. The rolls are **deliberately not modelled**: they resolve in
  play at the moment an aura is entered and produce no sheet value.
  Pinned by
  `data_integrity.rs::the_realm_scoped_susceptibilities_are_surfaced_and_halve_no_flat_total`.

  **The two Form-scoped Flaws are scoped too, and that is why they are
  *computed per Form* rather than blanket.** Both used to apply to all ten Forms
  at once, and repeat purchases were indistinguishable from the first, because
  neither entry declared the Form the rulebook scopes it to (row 35 of
  `docs/open-todos.md`). `ArMDE:6144` gives Flawed Parma Magica "only half the
  normal Magic Resistance **against a certain Form** … more than once for
  different Forms", and `ArMDE:6348` gives Limited Magic Resistance "no bonus
  from **one of** your Form scores … multiple times, for multiple Forms". Both
  now carry a `form` parameter (`ParameterDomain::Form`) and both encode as a
  `magic_resistance_mod` with `param: "form"` — `no_form_bonus` and
  `halved_parma` — folded per Form by `derived.rs::in_play_mods` into
  `no_form_bonus_forms` / `halved_parma_forms`, exactly as Deft Form's
  `deft_forms` set already works.

  **Flawed Parma halves the Parma addend, not the total.** `ArMDE:6144`'s
  subject is "**Your Parma Magica**", and `ArMDE:9396` states the other half of
  the resistance arises "from a maga's **Form scores**" — a defective Parma does
  not produce that number and cannot reduce it. So against the named Form
  `MR = form_bonus + halve(5 × Parma)`, and `HalvableTotal::MagicResistance`
  (which meant "halve this **whole** total") was **deleted**: its only data user
  was this Flaw, its only consumer `derived/casting.rs::magic_resistance`, and
  `flaw.weak_magic_resistance` had already left it in the round-5 audit. A Might
  base is never halved — the comparison that picks Might over Parma
  (RoP:M:1472, `ArMDE:2627`) is made against that Form's own Parma figure, so
  the halving only ever moves the Parma side of it. Pinned by
  `derived.rs::flawed_parma_halves_only_the_parma_contribution_against_its_own_form`,
  `flawed_parma_never_halves_a_might_base` and
  `limited_magic_resistance_drops_the_form_bonus_of_its_own_form_only`.

  **Limited Magic Resistance's "caught without your Parma" clause is not
  modelled**, deliberately: whether the Parma is up is a scene fact, not a sheet
  number, and the fallback it states (Magic Resistance 0) is what the character
  already reads at Form 0 with no Parma. It rides in the Flaw's own rules text in
  both locales instead.
- **Flat Soak bonus/penalty** — `virtue.tough` (ArMDE:5145-5147), `flaw.frail` (ArMDE:6190-6193)
- **Wound/fatigue penalty delta** — `virtue.enduring_constitution` (ArMDE:3751-3754), `flaw.low_tolerance` (ArMDE:6366-6369), `flaw.painful_magic` (ArMDE:6574-6577), `flaw.vulnerable_casting` (ArMDE:6993-7004), `virtue.withstand_casting` (ArMDE:5261-5282), `flaw.obese` (ArMDE:6516-6519), `flaw.short_of_breath` (ArMDE:6733-6736), `virtue.long_winded` (ArMDE:4327-4330)
- **Wound-recovery modifier** — `flaw.fragile_constitution` (ArMDE:6186-6189), `virtue.rapid_convalescence` (ArMDE:4834-4837)
- **Combat total modifier (atk/def/init/dam)** — `flaw.hobbled` (ArMDE:6260-6263), `flaw.lame` (ArMDE:6330-6333), `flaw.missing_hand` (ArMDE:6438-6441), `flaw.missing_eye` (ArMDE:6434-6437), `flaw.poor_eyesight` (ArMDE:6606-6609), `flaw.palsied_hands` (ArMDE:6578-6581), `flaw.slow_reflexes` (ArMDE:6763-6766), `virtue.lightning_reflexes` (ArMDE:4311-4314), `virtue.fast_caster` (ArMDE:3865-3868) (`virtue.berserk`, ArMDE:3500-3503, left this list in F2/D61 — see **Berserk / Ways of the Land / Cyclic Magic / Special Circumstances** below)

  **What "combat rolls" / "combat scores" was read to mean.** Several of these
  Flaws penalize "combat rolls" or "combat scores" without naming the totals.
  ArMDE:16656 names five — "Initiative, Attack, Defense, Damage, and Soak" — so
  the phrase needed a decision, taken 2026-09-14 and recorded here (it closes
  row 37 of `docs/open-todos.md`).

  **The reading: the totals that take a Combat Ability — Attack and Defense.**
  The book's own formulas draw the line. ArMDE:16660 gives ATTACK TOTAL =
  Dexterity + **Combat Ability** + Weapon Attack Modifier + Stress Die and
  ArMDE:16662 gives DEFENSE TOTAL = Quickness + **Combat Ability** + Weapon
  Defense Modifier + Stress Die. ArMDE:16658's INITIATIVE TOTAL = Quickness +
  Weapon Initiative Modifier - Encumbrance + Stress Die carries no Combat
  Ability at all, and neither does ArMDE:16664's DAMAGE TOTAL (Strength +
  Weapon Damage Modifier + Attack Advantage) or ArMDE:16666's SOAK TOTAL
  (Stamina + Armor Protection). So a penalty on "combat" as a *skill at
  fighting* lands on exactly Attack and Defense; Initiative, Damage and Soak
  measure speed, brawn and armor, and are untouched. This is the same argument
  the `flaw.palsied_hands` paraphrase exemption in
  `rules_source_provenance.rs::PARAPHRASE_EXEMPTIONS` already makes from that
  Flaw's "including weapon skills" wording — the reading here simply applies it
  to the Flaws that say "combat" without saying "skill".

  Per item:

  - `flaw.hobbled` (ArMDE:6262) — "Her Dodge and other combat rolls are
    penalized by -6": -6 Attack, -6 Defense. Dodge *is* a Defense (ArMDE:16959
    gives it a Dfn column and no Atk), so naming it adds no total the -6 does
    not already reach, and the figure is uniform so no per-weapon scope is
    needed. The same entry's "you roll double the normal botch dice in combat
    situations" is **not modelled**: no `Effect` variant carries botch dice, and
    the app computes sheet totals rather than resolving rolls. Recorded as the
    narrowed row 37 in `docs/open-todos.md`.
  - `flaw.lame` (ArMDE:6332) — "-6 penalty on rolls involving moving quickly or
    with agility, **-3 on Dodge, and -1 on other combat scores**". The -6 is a
    roll made in play, not a sheet total, so it is not modelled. The other two
    figures are both encoded, because Dodge is a row of the weapon table using
    Brawl (ArMDE:16959) and so has a Defense Total of its own that genuinely
    differs from a weapon's: -1 Attack and -1 Defense unscoped, plus a -3
    Defense scoped to `weapon.dodge`. The scoped figure **replaces** the
    unscoped one on that weapon — which is what "**other** combat scores" says —
    rather than adding to it, so the Dodge line reads -3 and not -4. Encoded via
    `Effect::CombatMod`'s `weapon` field (`types.rs::Effect::CombatMod`), folded
    by `derived.rs::in_play_mods` as a delta over the unscoped sum so a second
    Flaw's unscoped penalty still applies on the Dodge line, and consumed by
    `derived/combat.rs::combat_totals`. Pinned by
    `derived.rs::weapon_scoped_combat_mod_replaces_general_on_that_weapon_only`.
    Deliberately **not** encoded as a flat -3 Defense: that would assert the
    Dodge penalty against weapon-parry defence, a different computed total the
    source says nothing about.
  - `flaw.missing_hand` (ArMDE:6440) — "Climbing, **combat**, and other
    activities normally requiring both hands are at a penalty of -3 or greater":
    -3 Attack, -3 Defense. It says "combat", so the reading above reaches both
    Combat-Ability totals. ArMDE:16656 describes the standard configuration as
    "using a weapon and a shield", which needs both hands, so combat as the book
    normally describes it does require them. "-3 **or greater**" is open-ended;
    the engine takes the **floor**, -3, because that is the only figure the
    source fixes and anything beyond it is a storyguide ruling. Corrected
    2026-09-14 from Attack-only, which also removed a sibling asymmetry in which
    a severed hand penalized fewer totals than the milder `flaw.palsied_hands`.
  - `flaw.palsied_hands` (ArMDE:6580) — "All rolls involving holding or wielding
    an object are made at -2, including weapon skills": -2 Attack, -2 Defense.
    The same entry's "must roll an extra botch die when casting a spell" is
    **not modelled**, for the same reason as Hobbled's botch clause.

  All four are pinned by
  `data_integrity.rs::the_combat_roll_flaws_penalize_the_combat_ability_totals`,
  which also asserts that none of them touches Initiative.
- **Study source-quality / advancement modifier** — `virtue.apt_student` (ArMDE:3422-3425), `virtue.book_learner` (ArMDE:3519-3522), `virtue.free_study` (ArMDE:3937-3940), `virtue.good_teacher` (ArMDE:3971-3974), `virtue.independent_study` (ArMDE:4115-4118), `virtue.study_bonus` (ArMDE:5056-5072), `flaw.unimaginative_learner` (ArMDE:6915-6918), `flaw.poor_student` (ArMDE:6626-6628), `flaw.incomprehensible` (ArMDE:6294-6297), `virtue.secondary_insight` (ArMDE:4892-4895), `flaw.loose_magic` (ArMDE:6354-6357)
- **Aging / longevity modifier** — `flaw.age_quickly` (ArMDE:5659-5662), `flaw.baneful_circumstances` (ArMDE:5687-5690), `flaw.monstrous_blood` (ArMDE:6454-6467), `virtue.bee_king` (ArMDE:3484-3499), `virtue.faerie_blood` (ArMDE:3797-3820), `virtue.magical_blood` (ArMDE:4359-4372), `virtue.unaging` (ArMDE:5187-5190), `flaw.bound_to_role_role` (ArMDE:5735-5748), `flaw.leprosy` (ArMDE:6338-6341), `flaw.poor_living_conditions` (ArMDE:6618-6621), `virtue.mild_aging` (ArMDE:4528-4531), `virtue.magian_lineage_major` (ArMDE:4339-4346), `virtue.magian_lineage_minor` (ArMDE:4339-4346)
- **Non-standard-casting penalty removal (Deft/Quiet/Subtle)** — **computed** into per-cell `NonStandardCasting` variants (`derived.rs`), not surfaced-only. Base Words/Gestures penalties `ArMDE:9236-9245` (no voice −10, no gestures −5). `virtue.quiet_magic` (ArMDE:4822-4826, +5 voice per casting, second casting eliminates), `virtue.subtle_magic` (ArMDE:5073-5076, +5 gesture), `virtue.deft_form` (ArMDE:3645-3648, Form-parameterized, waives both for that Form). Residuals clamp at 0.
- **Flat ability-total bonus (Concentration)** — `virtue.academic_concentration_subject` (ArMDE:3362-3367)


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
`ruleset::validate_effect_refs`. The **one-focus-per-magus** limit (ArMDE:4542) is
a validation rule `validate_magical_focus` (issue code `multiple_magical_foci`,
Fluent `issue-multiple_magical_foci` in en/de) that **counts** the `MagicalFocus`
effect across selections + grants — so it also catches two Minor Foci with
distinct descriptors, which pairwise `incompatible_with` could not.

> `ArMDE:4405` "A character can have only one Magical Focus, either major or minor,
> regardless of the source of the focus." (Restated at `ArMDE:4542`.)

Because a magus may hold only one Magical Focus of **either** magnitude,
`virtue.major_magical_focus` and `virtue.minor_magical_focus` are marked mutually
`incompatible_with` in `rules/core/virtues_flaws.json` (ArMDE:4405) — the same
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
E2E: `ui/e2e/specs/companion-editor.e2e.js`'s `mutually exclusive Virtues/Flaws`.

| Variant | Family / representative V/F | Source | 5i |
|---|---|---|---|
| `MagicalFocus { param(Text), major }` | Magical Focus — major/minor/mythic_blood | ArMDE:4399-4422, 4536-4542, 4573-4589 | computed |
| `CastingTotalMod { amount, scope }` | Flat casting bonus/penalty — method_caster (+3 formulaic_ritual), poor_formulaic_magic (−5 formulaic), afflicted_tongue, cyclic_magic ±3, special_circumstances, ways_of_the_land, potent_magic | ArMDE:4524-4527, 6610-6613, 5655-5658, 3635-3638, 5893-5896, 4998-5001, 5231-5234, 4740-4781 | computed; conditional ones folded **unconditionally** (no toggle exists), surfaced per scope as the `casting_mod_formulaic`/`_ritual`/`_spontaneous` addends |
| `LabTotalMod { amount, scope, suppressed_when }` | Flat lab bonus/penalty — adept_laboratory_student (+6), inventive_genius (+3), creative_block (−3), weak_scholar (−6), cyclic_magic (±3), potent_magic. Flat only for `spell_level_cap` (D1), which ignores `scope`/`suppressed_when` entirely; the in-play grid reads them per-entry instead of hardcoding ids (X7a-refactor: `scope: LabTotalModScope` is `in_play_grid`/`within_potent_field_only` (D79; `within_focus_only` before it)/`never_at_creation`; `suppressed_when: ParamGate` excludes the amount while it holds), see the "Flat lab-total bonus/penalty" bullet above and `derived.rs::in_play_lab_total_mod` (X7a). `virtue.aristotelian_training`'s `+1` (ArMDE:3440-3443) was deleted (X7a/D4) and no longer carries this effect. **X7c (D46/D67, Phase 2)**: `inventive_genius` and `creative_block` each state a second, still-uncomputed experimentation-dice clause (+6 / roll twice as many dice); both reclassify to `uncomputed_rule` carrying the whole passage in `description`, while keeping this flat modifier. | ArMDE:3368-3371, 4151-4154, 5873-5876, 7080-7083, 3635-3638, 5893-5896, 4740-4781 | computed |
| `flaw.cyclic_magic_negative`'s `parameters` — `cycle` (Enumerated: `cycle.solar`/`cycle.lunar`/`cycle.seasonal`) | D9/D52/X7a: "attuned to some cycle of nature (solar, lunar, or seasonal, for example)" (ArMDE:3637, mirrored at :5893-5896 for the Flaw) is a stated choice D9 obliges recording. The entry's own `LabTotalMod::suppressed_when: { "param": "cycle", "equals": "cycle.seasonal" }` (X7a-refactor; formerly a hardcoded check in `derived.rs::in_play_lab_total_mod`) suppresses the −3 in the in-play grid when `cycle.seasonal` is selected (D52 — a seasonal cycle is exactly as uncertain at creation as the Virtue's bonus); solar/lunar/absent apply the flat −3. `virtue.cyclic_magic_positive` needs no parameter — its in-play Lab answer is "no" regardless of cycle type. i18n: `param-label-cycle` (`.ftl`), `cycle.solar`/`cycle.lunar`/`cycle.seasonal` (`rules/i18n/<lang>/virtues_flaws.json`) | ArMDE:3635-3638, 5893-5896 | computed (lab_mod only; D1's `spell_level_cap` fold ignores it, matching every other carrier) |
| `DeficientArt { param(Technique\|Form) }` | Art-halving — deficient_technique, deficient_form | ArMDE:5913-5915, 5909-5912 | computed |
| `MagicTotalHalving { total }` | Halve spont casting / lab-enchant / lab-longevity / penetration — weak_spontaneous_magic, weak_enchanter, difficult_longevity_ritual, weak_magic. Two items have left this family: weak_magic_resistance in the round-5 audit (`ArMDE:7070` halves nothing), and flawed_parma_magica with row 35 (it halves one *addend* against one *Form*, which is not a whole total — see **Magic-resistance modifier** above). `HalvableTotal::MagicResistance` was deleted with the second of them | ArMDE:7084-7089, 7060-7063, 5962-5964, 7064-7067 | **computed** (round-2 audit finding GD3 closed the last gap): `spontaneous_casting`, `penetration`, `lab_longevity` (since M5.5a), and now `lab_enchanting` too — folded into the new `LabTotal.enchanting` field in `derived/lab.rs::lab_totals` (Deficiency first, then this halving, per `ArMDE:7060-7063`'s own stated order). Round 3 (G1) wired `enchanting` into `masterpiece_item_cap` too — the one remaining consumer of a Lab Total that used to read `total` instead — and into the frontend `LabTotal` type / `DerivedTotalsPanel` (G2) |
| `SoakMod { amount }` | Flat Soak — tough (+3), frail (−3), berserk (+2) | ArMDE:5145-5147, 6190-6193, 3500-3503 | computed |
| `CombatMod { amount, target, weapon }` | Combat init/atk/def — hobbled, lame, missing_hand, missing_eye, poor_eyesight, palsied_hands, slow_reflexes, lightning_reflexes, fast_caster | ArMDE:6260-6263, 6330-6333, 6438-6441, 6434-6437, 6606-6609, 6578-6581, 6763-6766, 4311-4314, 3865-3868 | computed, **not labelled** — `CombatLine` carries no `addends` at all. `weapon` restricts a figure to one weapon's lines and **replaces** the same item's unscoped figure there (only `flaw.lame`'s -3 on `weapon.dodge`, ArMDE:6332); an unresolvable weapon fails referential integrity. `virtue.berserk` (ArMDE:3500-3503) used to be the one **conditional** carrier here ("while berserk", folded unconditionally) — F2 (D61/D15) deletes its three effects outright and reclassifies it `uncomputed_rule` instead of surfacing the condition; see **Berserk / Ways of the Land / Cyclic Magic / Special Circumstances** below |
| `HealthMod { track, amount }` | Wound/fatigue penalty (enduring_constitution, low_tolerance), fatigue rolls (obese, short_of_breath, long_winded), casting-fatigue (painful_magic, vulnerable_casting, withstand_casting), recovery (fragile_constitution, rapid_convalescence) | ArMDE:3751-3754, 6366-6369, 6516-6519, 6733-6736, 4327-4330, 6574-6577, 6993-7004, 5261-5282, 6186-6189, 4834-4837 | wound/fatigue computed; fatigue-roll/casting-fatigue/recovery surfaced |
| `MagicResistanceMod { kind, param }` | MR modifiers — limited_magic_resistance (no_form_bonus), flawed_parma_magica (halved_parma), susceptibility faerie/infernal, commanding_aura & special_circumstances (aura_bonus), weak_magic_resistance (conditional_penetration_waiver). `param` names the selection key carrying the **Form** the modifier is scoped to; it is set on the first two kinds and absent on the rest, which name no Form | ArMDE:6346-6349, 6142-6145, 6819-6826, 3579-3596, 7068-7071 | **no_form_bonus and halved_parma computed** (folded into the flat per-Form MR number in `magic_resistance`, each against the one Form its own copy names — a copy naming no Form applies to none, and `missing_param` asks for the choice rather than the engine guessing it); the four conditional/situational kinds (aura_bonus, susceptible_faerie/infernal, conditional_penetration_waiver) are surfaced as `ModifierFamily::MagicResistance` (amount 0) — each carries a scope the flat per-Form figure has no axis for (a realm, an aura, a scene condition plus the incoming spell's level), so listing them keeps them from being silently dropped *and* from being applied where the book does not apply them. `susceptible_divine` was retired in the round-5 audit: `ArMDE:6817` never mentions Magic Resistance, and the Flaw now carries `special_casting_mod { doubled_aura_penalty }` |
| `AgingMod { kind, amount }` | Aging/longevity — age_quickly, baneful_circumstances, monstrous_blood (−1), bee_king, faerie_blood (−1), magical_blood (−1), strong_faerie_blood (−3), unaging, bound_to_role, leprosy, poor_living_conditions, mild_aging, magian_lineage major/minor | ArMDE:5659-5662, 5687-5690, 6454-6467, 3484-3499, 3797-3820, 4359-4372, 5032-5047, 5187-5190, 5735-5748, 6338-6341, 6618-6621, 4528-4531, 4339-4346 | **computed since M6/6b6**: `aging_roll` and `longevity_bonus` move the AGING TOTAL, `living_conditions` moves the modifier it subtracts, `no_apparent_aging` gates the apparent age and `no_aging` gates the Characteristic drop. Three items stay surfaced-only, each for a stated reason — age_quickly and baneful_circumstances (amount 0; schedule rules, not modifiers) and any `decrepitude` amount (no shipped item carries one). **The two immunities are separate tags**: bee_king carries `no_apparent_aging` alone, bound_to_role `no_aging` alone, unaging both — see **Aging (M6/6b6)**. **X7c (D46/D67, Phase 2)**: age_quickly's doubled aging-roll cadence, baneful_circumstances' Fatigue/healing/Might block, magian_lineage minor/major's shared +3 disease-resistance bonus, and strong_faerie_blood's age-50 (not 35) aging-roll onset all reach the player nowhere as numbers; all four ids reclassify to `uncomputed_rule` carrying the whole passage in `description`, while keeping their existing `AgingMod`/other effects unchanged |
| `AdvancementMod { source, amount?, factor? }` | Study/teaching — apt_student (+5 taught), book_learner (+3 book), free_study (+3 vis), good_teacher (+5 teaching, +3 authoring), independent_study, study_bonus, secondary_insight, unimaginative_learner, poor_student, incomprehensible (teaching ×½, authoring ×½), loose_magic (spell_mastery ×½) | ArMDE:3422-3425, 3519-3522, 3937-3940, 3971-3974, 4115-4118, 5056-5072, 4892-4895, 6915-6918, 6626-6628, 6294-6297, 6354-6357 | surfaced (app does not simulate advancement); exactly one of `amount`/`factor` is present, load-validated (D55/Q6) |
| `SpecialCastingMod { kind, param? }` | Casting-style quirks — deft_form (Form-parameterized), quiet_magic, subtle_magic, diedne_magic, faerie_raised_magic, life_linked_spontaneous_magic, spell_improvisation, mercurian_magic, life_boost, leper_magus, circumstantial halvings (deleterious_circumstances, environmental_magic_condition, short_ranged_magic, disjointed_magic, the_constant_expression), and doubled_aura_penalty (susceptibility_to_divine_power). **`corrupted_spells` left this family in Phase 2 C5c (D15)**: its `special_casting_mod { circumstantial }` is deleted — the ±3 is on an *Ability/Casting roll a GM judges selfish-or-sinful*, which this family cannot express any more precisely than any other uncomputed rule, so D15 reclassifies all three Corrupted entries `uncomputed_rule` instead (see **Corrupted Abilities/Arts/Spells** below) | ArMDE:3645-3648, 4822-4826, 5073-5076, 9236-9245, 3675-3682, 3829-3842, 4299-4306, 5002-5005, 4514-4523, 4295-4298, 4249-4252, 5917-5920, 6020-6023, 6737-6740, 5972-5975, 5821-5838, 6815-6818 | **deft_form/quiet_magic/subtle_magic computed** into per-cell `NonStandardCasting` (silent/still/silent_and_still); all other kinds surfaced (conditional penalties). `deft_form`'s `param` names the affected Form and is load-validated (`validate_effect_refs`, `ParameterDomain::Form` required) exactly as `DeficientArt`'s param, so a missing/wrong-domain key fails loudly instead of silently voiding the waiver in `in_play_mods` |
| `AbilityRollMod { ability, amount }` | Fixed-target Ability-roll modifier (B5/F-489; D61 withdrew the original worked example, poor_hearing, as a sense-conditioned false carrier — "rolls involving hearing" also hits non-hearing Awareness rolls and misses non-Awareness hearing rolls, so it stays `uncomputed_rule` text): the entry itself names one Ability with no further condition — poor_concentration (Concentration, −3), inconstant_magic (Finesse, −3), clumsy_magic (Finesse, −3) | ArMDE:6602-6605, :6298-6301, :5801-5804 | surfaced |
| `AbilityRollModParam { param(Text), amount }` | Ability-roll bonus in a subject the player names (renamed from `AbilityRollMod`, B5/F-489 — the naming convention's `...Param` = parameter-relative twin) — academic_concentration_subject (+3) | ArMDE:3362-3367 | surfaced |

**Modeling notes / accepted approximations** (each surfaced in 5i's labelled
read-out, so precision is not lost to the player): `weak_spontaneous_magic` maps
to `MagicTotalHalving { spontaneous_casting }`; the book rule is "you always
divide your Casting Score by five" (`ArMDE:7084-7086`) — the Flaw removes the
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
**E2 (V/F audit Q-32 / F-91)**: `good_teacher`'s second row used to read
`{ source: "book", amount: 3 }` for *"Add three to the Quality of any books
that you write"* (ArMDE:3973) — the mis-directed row, since `book` is the
reader-side shape `book_learner`/`study_bonus` use ("Learning from a book"),
not the character's own authorship. `AdvancementSource` gained a tenth
variant, `Authoring`, and `good_teacher` now carries
`{ source: "authoring", amount: 3 }` instead.

**D55 (Q6, V/F audit Q-113): `amount: 0` retired as a "halved" marker — a
factor instead.** `RULES.md`'s own note above used to read *"`incomprehensible`
/`loose_magic` halve advancement (amount 0 marker; surfaced)"* — the overload
D55 targets: `amount: 0` meant both *"no magnitude carried"* (every other
surfaced family, still true) and *"halve this source's Advancement Total"*
(these two entries only), and the UI's `{#if m.amount !== 0}` guard read the
second case as the first, so the player saw a labelled row with nothing beside
it. `Effect::AdvancementMod` now carries `amount: Option<i8>` **and**
`factor: Option<AdvancementFactor>`, exactly one of which is `Some` — enforced
at load by `ruleset/integrity.rs::validate_advancement_mod_shape`, which fails
naming the item and source for either *neither present* (a meaningless row) or
*both present* (contradictory: an addend and a total-multiplying factor are
different operations, and no shipped entry states both in one clause).

**The representation is a small enum, `AdvancementFactor { Half }`, not a
`num`/`den` pair.** Every stated `AdvancementMod` factor in the core rules is a
halving (Incomprehensible ArMDE:6296, Loose Magic ArMDE:6354-6357) — contrast
`Effect::GrantsSpellMastery`'s `advancement_num`/`advancement_den`, which
genuinely needs a fraction because Flawless Magic *doubles* a *different*
total (ArMDE:3889). A fraction pair can express a zero denominator or a
reversed ratio, which is exactly what `validate_item_ratios`'s long doc comment
(above) exists to catch at load for the four ratio-bearing effects that need
it; an enum carrying only the one value the rulebook states cannot express that
nonsense at all, so no such validator is needed for this field. A second stated
factor later is a new variant, and the exhaustive `match` in every consumer —
`derived.rs`'s push site, `crates/arm-app/tests/commands.rs`'s
`every_advancement_factor` Fluent-coverage tripwire — turns that into a compile
error until it is handled, exactly as a new `AdvancementSource` already does.

**Rounding: down, per the core book's own stated default.** Neither passage
gives its halving a rounding direction of its own (Incomprehensible: "must
halve their Advancement Total"; Loose Magic: "Your Advancement Total is halved
whenever you try to Master spells"), so ArMDE:547 governs: *"The rules for Ars
Magica sometimes involve division. In most cases, a rule specifies whether you
should round up or down, but if it does not, round down."* This is a
documented decision, not a computed one — the app does not simulate
Advancement Totals (see this row's own last column), so there is no numeric
`apply()` to round; a future slice that does compute one rounds down.

**Order relative to an additive bonus on the same source: additive first, then
the factor, per the book's own three-stage model** (ArMDE:15989,
`**ADVANCEMENT TOTAL: Source Quality + Bonus from Virtues - Penalty from
Flaws**`; ArMDE:15997-16001's three steps — Source Quality, then the Advancement
Total, then experience points). Good Teacher's/Apt Student's bonuses act at the
Source-Quality stage, which feeds additively into the Advancement Total's own
formula; Incomprehensible and Loose Magic both name their halving target as
*"their Advancement Total"* — the already-assembled figure, one stage later.
So a character holding both an additive-source-quality entry and a
halving-total entry on the same source would sum every stated bonus/penalty
first, then apply the factor to that sum — not the reverse. This is stated
here for the record: nothing in this codebase combines same-source rows into
one number today (surfaced-only, see this row's last column), so the order has
no live consumer yet; it binds the day one is built.

**`incomprehensible` needs two factor rows, not one — the authoring axis Q-32
introduced.** ArMDE:6296: *"Anyone trying to learn **from you or from a book
you have written** must halve their Advancement Total…"* — a `teaching` row
(being taught by this character) and an `authoring` row (studying from a book
this character wrote), each `{ factor: "half" }`. `loose_magic` gets the same
factor on its one `spell_mastery` row. Fluent: the factor renders through its
own `derived-factor-<slug>` key (`DerivedSurfacedModifiersSection.svelte`),
distinct from `derived-detail-<slug>`'s source-name namespace — a row carrying
a factor names both a source and a multiplier, and confusing the two keys would
silently collide two unrelated slugs (e.g. `teaching` is both a source name and
would-be factor name).

Conditional `CastingTotalMod`/`CombatMod`/`LabTotalMod` amounts (cyclic magic,
special circumstances, etc.) are folded **unconditionally** into the printed
totals. **X7b-d (8c2a252) moved two examples out of this generalization**:
Potent Magic's Lab/Casting bonus is no longer folded into the unconditional
total at all — it lands only in the separate `within_focus` figure (D4, see
the "Flat lab-total bonus/penalty" row) — and Missing Eye's ranged penalty is
no longer a flat −3 folded into every total; it is a weapon-scoped `combat_mod`
that replaces the unscoped figure only for the seven ranged weapons it names
(`flaw.lame`'s own scoped-delta precedent), computed rather than folded flat.
This row previously claimed they
were "shown as toggleable/labelled addends in 5i rather than always-on numbers";
round 4 (Gerda 5) established that no toggle exists anywhere in `crates/` or
`ui/`, and the always-on fold is the decision that actually shipped. What is
true:

- `LabTotalMod` — folded, and **labelled** as the `lab_mod` addend
  (`derived/lab.rs::lab_totals`). **X7a/D4 correction**: as of X7a, this is no
  longer true for every carrier. `effective/spell.rs::lab_total_mod` (D1)
  still folds every carrier unconditionally for `spell_level_cap` only; the
  in-play `lab_mod` addend above is folded by a **separate** function,
  `derived.rs::in_play_lab_total_mod`, which excludes
  `virtue.adept_laboratory_student`, `flaw.weak_scholar` and
  `virtue.cyclic_magic_positive` (their condition never holds at character
  generation) and gates `flaw.cyclic_magic_negative` (no penalty when
  its `cycle` parameter is `cycle.seasonal`, D52) — **X7a-refactor**: reading
  the entry's own `scope: LabTotalModScope`/`suppressed_when: ParamGate`
  fields (`types.rs`) rather than matching against a hardcoded id list.
  Inventive Genius and Creative Block are unaffected. **X7b-d (8c2a252), D79**:
  both Potent Magic entries are no longer unaffected either — both set
  `scope: "within_potent_field_only"` (D79; `"within_focus_only"` before it —
  X7a-refactor named it after the wrong Virtue), so `in_play_lab_total_mod`
  excludes them and their +3/+6 lands only in the separate
  `within_potent_field` field via `in_play_lab_total_mod_within_potent_field`,
  gated on `InPlayMods::has_potent_magic` rather than on holding a Magical
  Focus — see the "Flat lab-total bonus/penalty" and `LabTotalMod` rows below.
- `CastingTotalMod` — folded, and labelled since round 4 as the three per-scope
  `casting_mod_*` addends (`derived/casting.rs::CastingTotal`); before that it
  was folded into the printed figure with nothing on screen accounting for it.
  **D79**: Potent Magic's two entries additionally set `potent_field_only:
  true`, which routes them to a **separate** fold
  (`InPlayMods::casting_mods_within_potent_field`, MAX not sum per
  ArMDE:4742) feeding `CastingTotal::within_potent_field` instead of the
  labelled `casting_mod_*` addends above, which explain only the
  unconditional base/within-focus figures.
- `CombatMod` — folded, and **not** surfaced at all: `CombatLine` has no
  `addends` field, so a conditional combat modifier is invisible in the
  breakdown.

The condition itself is carried only by the item's own rules text; the engine
applies every one of these amounts always. A per-condition toggle is **not
implemented** and is not claimed here — if it is wanted it is queued work, and
this document records implemented properties only.


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
default). True Faith → **X7b-d (8c2a252) correction**: Relic/Powerful Relic no
longer carry `true_faith_grant` — a new `relic_true_faith` effect replaces it
on both, and the bearer's own True Faith Score stays 0 (see **Relic /
Powerful Relic** below). Size →
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

**Deferred (2), source-not-present or no clean creation number:**
- **Savantism, Corrupted Arts** — Savantism
  (ArMDE:6703) *halves* starting XP (multiplicative; no variant); Corrupted Arts (ArMDE:5853) has no
  creation XP figure (its ±3 casting swing / ±5 Art xp are in-play). (Elemental
  Magic, :3731, is now implemented in slice 5c — see its section above. Simple
  Student, ArMDE:4958, was the third deferred entry here — it is now wired,
  D35/Phase 2 C3 — see **Simple Student's parameter-scaled XP grant** below.
  Corrupted Arts's "are in-play" half is also now stale: Phase 2 C5c/D15
  removes even the in-play reading — see **Corrupted Abilities/Arts/Spells &
  Enchanting (Ability) (F-63)** below.)

#### Student of (Realm) — missed by the 5a-wire pass, fixed in the audit-fix round

> You have been trained in the mystical aspects of one of the four realms of
> power (Divine, Faerie, Infernal, or Magic) … You may take that Lore at
> character generation even if you cannot learn other Arcane Abilities.

- Source: `ArMDE:5052-5055`.
- **Why this was missed by 5a-wire:** the 5a-wire pass above only scanned
  entries already tagged `creation_effect` for a missing `Effect`. This item
  was mis-tagged `narrative` from the M5/5a classification pass, so it was
  invisible to that scan and shipped with no `effects` at all — a magus or
  companion taking this Virtue got no mechanical benefit whatsoever.
- Data (as of this fix): `rules/core/virtues_flaws.json` `virtue.student_of_realm`
  — reclassified `narrative` → `creation_effect` and given
  `effects: [{ "type": "ability_authorization", "abilities":
  ["ability.dominion_lore", "ability.faerie_lore", "ability.infernal_lore",
  "ability.magic_lore"] }]` — a static, unconditional `Vec<Id>`, since
  `Effect::AbilityAuthorization` was not yet parameter-relative and the Virtue's
  `realm` parameter names a **Realm**, not a Lore Ability, so it could not be
  bound to one specific Lore the way `Effect::AbilityBonus`'s `param` mechanism
  requires. This shipped two known gaps, recorded at the time as follow-ups:
  the `+2` bonus on the chosen Lore was not implemented, and the authorization
  was permissive rather than exact — taking Student of (Divine) also nominally
  authorized buying Faerie/Infernal/Magic Lore at creation (P4 in
  `docs/book-template-conformance.md`).
- **Both gaps are fixed by Phase 2 C1** (`docs/vf-audit/design-c0-parameter-model.md`
  § 3): see **Access to Academic / Arcane / Martial Abilities** below,
  "Student of (Realm)'s gated bonus doubles as its own authorization" — the
  `AbilityBonusGated` effect (targets gated on `realm`) both applies the `+2`
  and narrows authorization to the one Lore the chosen realm names, replacing
  the plain `ability_authorization` effect shown above entirely. Regression
  coverage: `crates/arm-rules/tests/book_templates.rs` (`the_merinita_matches_the_book`,
  `the_priest_matches_the_book`, `the_witch_matches_the_book`, each now
  asserting the `X+2` printed score) and
  `crates/arm-rules/src/validation/authorization.rs`'s
  `student_of_realm_authorizes_only_the_chosen_realms_lore`. The Puissant-
  Ability-for-the-same-Lore incompatibility the passage also states remains
  unmodelled — out of this fix's scope, not silently dropped.

#### Supernatural Might & Magic Resistance (Core Rules; general MR rule from Realms of Power: Magic)

A supernatural being has a **Might Score** aligned to one **Realm** (`Realm::{Magic,
Faerie, Divine, Infernal}`). The general rule:

> "Magic Might gives the character innate Magic Resistance equal to its Might
> Score, and this does not stack with other forms of resistance … they must use
> either their Parma or their Might for their base Magic Resistance."
> — *RoP:M:1472* (general; also ArMDE:2623-2631, :2627 "these
> totals do not stack … use the higher total").

**MR-from-Might formula** (`derived::magic_resistance`): per Form, `total = Form
bonus + max(5 × Parma, Might Score)`. Might and Parma do **not** stack — the higher
is the base (labelled `might` or `parma` addend); the Form bonus is compatible with
either. A pure being (no Parma, Art 0) therefore gets a flat blanket MR = Might
Score on every Form. `derived_totals` now surfaces Magic Resistance for a
Might-being, not only a magus.

**Data model.** `Entity.might: Option<MightScore{realm, score}>` is the base the
player enters (may be 0); `Entity.powers: Vec<SupernaturalPower{name, level,
penetration}>` are free-text powers charged against a power-levels budget
(mirroring `devices` vs `item_level_budget` — the engine is not a power
*designer*); `penetration` is charged against the *same* budget as `level`, and
the four core Power Virtues fund it — see *Power Virtues fund the power-levels
budget (B10)*. Two additive `Effect`
variants (SCHEMA_VERSION unchanged at 9 — both `#[serde(default)]`, old saves load):
- `Effect::MightGrant{realm, score}` — summed (same Realm) on top of the entered
  base by `effective::effective_might`; a `score` 0 grant establishes the Realm
  without adding points.
- `Effect::PowerLevels{amount}` — summed by `effective::power_levels_budget`.

**The four V/F grants (verified against source):**

| Virtue | Grant | Source (file:line) |
|--------|-------|--------------------|
| `virtue.demonic_blood` (Major) | Infernal Might **5** + **30** power levels | ArMDE:3651, :3653 |
| `virtue.demonic_might` (Minor, req. Demonic Blood) | Infernal Might **+2** | ArMDE:3665 |
| `virtue.demonic_powers` (Minor, req. Demonic Blood) | **+20** power levels | ArMDE:3669 |
| `virtue.strong_angelic_heritage` (Minor, req. Blood of the Nephilim) | Divine Might = **age ÷ 20** (entered by hand) + **30** power levels | ArMDE:5026, :5028 |

Effective Might = entered base (may be 0/None) + Σ same-Realm `MightGrant` scores.
So Demonic Blood alone → Infernal Might 5; with Demonic Might → 7. Strong Angelic
Heritage's Divine Might is age-derived (no fixed constant), so it grants
`might_grant{divine,0}` (establishing the Realm + MR) plus its 30 power levels; the
player enters the age÷20 base. Validation: `validate_powers` (used ≤ budget →
`over_power_levels` error, mirroring devices); `validate_might` (base Realm vs
granted Realm → `might_realm_mismatch` warning).

The per-item audit that fed this wiring follows (source line-ranges retained).

**Confidence:**
- `flaw.low_self_esteem` (ArMDE:6362-6365) — Confidence (no score/points)
- `virtue.ferocity` (ArMDE:3873-3876) — Confidence score 1 points 3

**Free nested V/F grant:**
- `virtue.devil_child` (ArMDE:3671-3674) — grants free Minor Virtue
- `virtue.faerie_doctor` (ArMDE:3821-3824) — grants free Virtue (Dowsing)
- `virtue.nephilim` (ArMDE:4594-4597) — grants free Virtue

**Free starting Supernatural Ability score:**
- `virtue.animal_ken` (ArMDE:3414-3417) — free starting Supernatural Ability score
- `virtue.corpse_magic` (ArMDE:3605-3608) — free starting Supernatural Ability score
- `virtue.crafters_healing` (ArMDE:3617-3620) — free starting Supernatural Ability score
- `virtue.embitterment` (ArMDE:3739-3742) — free starting Supernatural Ability score
- `virtue.enchanting_ability` (ArMDE:3747-3750) — free starting Supernatural Ability score (Phase 2 C5c/F-63: moved from a plain, unconditional `ability_score_grant` to the parameter-bound `ability_score_grant_param`, so the floor applies only at the player's chosen `medium` instance — see **Corrupted Abilities/Arts/Spells & Enchanting (Ability) (F-63)** below)
- `virtue.entrancement` (ArMDE:3767-3770) — free starting Supernatural Ability score
- `virtue.font_of_knowledge` (ArMDE:3921-3924) — free starting Supernatural Ability score
- `virtue.hex` (ArMDE:4075-4078) — free starting Supernatural Ability score
- `virtue.induction` (ArMDE:4119-4122) — free starting Supernatural Ability score
- `virtue.magic_sensitivity` (ArMDE:4351-4354) — free starting Supernatural Ability score
- `virtue.persona` (ArMDE:4710-4713) — free starting Supernatural Ability score
- `virtue.sense_passions` (ArMDE:4930-4933) — free starting Supernatural Ability score
- `virtue.shapeshifter` (ArMDE:4946-4949) — free starting Supernatural Ability score
- `virtue.spirit_votary` (ArMDE:5006-5009) — grants free Virtue (Second Sight)
- `virtue.strong_faerie_blood` (ArMDE:5032-5047) — free starting Second Sight ability
- `virtue.summon_animals` (ArMDE:5085-5088) — free starting Supernatural Ability score
- `virtue.whistle_up_the_wind` (ArMDE:5243-5246) — free starting Supernatural Ability score
- `virtue.wilderness_sense` (ArMDE:5247-5250) — free starting Supernatural Ability score

**Item-level budget:**
- (Magic Items / Redcap wire `item_level_budget`; see slice 5e.)

**Might / power budget (wired, this slice):**
- `virtue.demonic_blood` (ArMDE:3651, :3653) — `might_grant{infernal,5}` + `power_levels{30}`
- `virtue.demonic_might` (ArMDE:3665) — `might_grant{infernal,2}` (adds +2)
- `virtue.demonic_powers` (ArMDE:3669) — `power_levels{20}`
- `virtue.strong_angelic_heritage` (ArMDE:5026, :5028) — `might_grant{divine,0}` + `power_levels{30}` (Divine Might = age÷20 entered by hand; grant establishes Realm)

**Reputation grant** (D11/Q5, 2026-09-26: `score` is now enforced — exact by
default, a range only where the grant states `max_score`. `validate_reputations`
raises `reputation_score_out_of_range` for a stored Reputation whose score
falls outside its grant; `docs/vf-audit/decisions.md` § D11,
`corrections.md` § 3.11):
- `flaw.apostate` (ArMDE:5675-5678) — reputation grant bad score 4
- `flaw.excommunicate` (ArMDE:6044-6047) — reputation grant ecclesiastical
  score 3 (F-408: carried by no effect before this fix)
- `flaw.failed_journeyman` (ArMDE:6060-6063) — reputation grant bad score 2
- `flaw.failed_master` (ArMDE:6064-6067) — reputation grant bad score 4
- `flaw.failed_monk` (ArMDE:6068-6071) — reputation grant poor score 2
- `flaw.failed_student` (ArMDE:6072-6075) — reputation grant academic score 2
- `flaw.feral_scent` (ArMDE:6106-6109) — reputation grant negative score 2
- `flaw.gabai` (ArMDE:6198-6201) — reputation grant negative score 2
- `flaw.hedge_wizard` (ArMDE:6240-6243) — reputation grant hermetic score 3
- `flaw.infamous` (ArMDE:6310-6312) — reputation grant player-chosen score 4
  (F-450: no `kind` — the passage names no audience, and the shipped
  `kind: "local"` was a hardcoded invention; its twin `virtue.famous` already
  ships the wildcard for the same shape)
- `flaw.infamous_master` (ArMDE:6314-6317) — reputation grant hermetic score 3
- `flaw.outlaw` (ArMDE:6542-6545) — reputation grant player-chosen score 2
  (F-484: `kind` dropped — the passage names no audience)
- `flaw.outlaw_leader` (ArMDE:6546-6549) — reputation grant score 3
- `flaw.outsider_major` (ArMDE:6550-6561) — reputation grant local score 1,
  max_score 3 (F-486: the one entry in the catalogue the book states a range
  for, "a bad Reputation of level 1 to 3" — was a fixed, invented score 3)
- `flaw.outsider_minor` (ArMDE:6550-6561) — reputation grant local score 1,
  max_score 3 (F-486, same range: ":6556 says 'You **still** have the bad
  Reputation'" — was a fixed, invented score 1)
- `flaw.usurer` (ArMDE:6951-6954) — reputation grant poor score 4
- `virtue.baccalaureus` (ArMDE:3470-3475) — XP grant 90 + reputation grant academic score 1
- `virtue.bard` (ArMDE:3476-3479) — reputation grant local score 1
- `virtue.cathedral_school_master` (ArMDE:3549-3554) — XP grant 240 + reputation grant academic score 2
- `virtue.doctor_in_faculty` (ArMDE:3683-3698) — XP grant 300 + reputation grant academic score 3
- `virtue.famous` (ArMDE:3861-3864) — reputation grant player-chosen score 4
- `virtue.frightful_presence` (ArMDE:3941-3950) — reputation grant player-chosen
  score 2 (F-80: "an appropriate Reputation … at a score of 2 among those you
  have affected" — carried by no effect before this fix)
- `virtue.hermetic_prestige` (ArMDE:4071-4073) — reputation grant hermetic score 4
- `virtue.lone_redcap` (ArMDE:4319-4326) — XP grant 300 + grants Virtue + reputation grant poor score 2
- `virtue.magister_in_artibus` (ArMDE:4385-4394) — XP grant 240 + reputation grant academic score 2
- `virtue.magister_in_medicina` (ArMDE:4395-4398) — XP grant 300 + reputation grant academic score 3
- `virtue.master_bard` (ArMDE:4457-4462) — XP grant 240 + reputation grant local score 3
- `virtue.physician_of_salerno` (ArMDE:4732-4735) — XP grant 50 + reputation 2. **X7c (Q-55, D46/D67)**: the granted Reputation travels with the character ("carries the reputation of the school with him"), so its `kind` is the player-chosen wildcard (omitted, not `local`) — matching `flaw.infamous`'s (F-450) shape
- `virtue.protection` (ArMDE:4810-4813) — reputation grant player-chosen score 3
  (F-235: "a Reputation (good or bad, your choice) of level 3" — carried by no
  effect before this fix)
- `virtue.rosh_beth_din` (ArMDE:4878-4883) — XP grant 50 + reputation grant
  player-chosen score 2 + grants Virtue (F-254: `kind` dropped — "applies
  across his country" is wider than Local and not
  Ecclesiastical/Hermetic/Academic either, so the shipped `kind: "local"` was
  wrong)
- `virtue.senior_bard` (ArMDE:4904-4909) — XP grant 90 + reputation grant local score 2
- `virtue.senior_clergy` (ArMDE:4910-4921) — reputation grant score 4
- `virtue.templar_office_holder` (ArMDE:5121-5124) — reputation grant score 2
- `virtue.templar_prestige` (ArMDE:5125-5128) — reputation grant player-chosen
  score 4 (F-316: "a Reputation of level 4 within the Templars" — an
  organization, not one of the four fixed Reputation types, so wildcard;
  carried by no effect and classified `narrative` before this fix)

**Size/characteristic delta:**
- `virtue.blood_of_the_nephilim` (ArMDE:3509, :3511) — size delta + Dominion Lore

**True Faith score — SUPERSEDED (X7b-d, 8c2a252, F-256):** these two no longer
carry `true_faith_grant`. Both now carry the new `Effect::RelicTrueFaith
{ score }` instead — the relic's own score (3 / 1), never folded into the
bearer's `effective::true_faith()` or MR floor. See **Relic / Powerful
Relic** below.
- `virtue.powerful_relic` (ArMDE:4782-4787) — relic True Faith score 3
- `virtue.relic` (ArMDE:4852-4855) — relic True Faith score 1

**XP grant:**
- `flaw.corrupted_arts` (ArMDE:5853-5858) — grants XP swing at creation + situational casting (**superseded, Phase 2 C5c/D15**: it never actually carried an effect at any point — see **Corrupted Abilities/Arts/Spells & Enchanting (Ability) (F-63)** below, which reclassifies it `uncomputed_rule` with no effects at all, same as its two siblings)
- `flaw.feral_upbringing` (ArMDE:6110-6113) — XP grant 120
- `flaw.savantism` (ArMDE:6703-6708) — halves starting XP
- `virtue.arcane_lore` (ArMDE:3430-3435) — XP grant 50
- `virtue.clan_ilfetu` (ArMDE:3563-3566) — XP grant 50
- `virtue.craft_guild_training` (ArMDE:3613-3616) — XP grant 50, Organization Lore of the guild scoped via its `guild` parameter (`instances`, not the unscoped `abilities` list). **X7c**: `virtue.educated_vernacular` (ArMDE:3727-3729) gets the same fix — its "Organization Lore of the character's company" is moved from the unscoped `abilities` list to a parameter-bound `instances` entry keyed on a new `company` parameter, mirroring this shape exactly
- `virtue.elemental_magic` (ArMDE:3731-3738) — Art XP distribution at creation (implemented, slice 5c: `Effect::ElementalMagic` XP-space Art boost — see section above)
- `virtue.falconer` (ArMDE:3847-3852) — XP grant 50
- `virtue.forge_companion` (ArMDE:3925-3928) — XP grant 50
- `virtue.hermetic_experience` (ArMDE:4063-4066) — XP grant 50
- `virtue.ineslemen` (ArMDE:4123-4126) — XP grant 50 + grants Minor Flaw
- `virtue.marshal` (ArMDE:4449-4456) — XP grant 50
- `virtue.master_of_kennels` (ArMDE:4467-4470) — XP grant 50
- `virtue.mentored_by_demons` (ArMDE:4496-4499) — XP grant 50
- `virtue.schooled_in_crime` (ArMDE:4884-4887) — XP grant 50
- `virtue.shadchan` (ArMDE:4934-4939) — XP grant 50
- `virtue.simple_student` (ArMDE:4958-4963) — parameter-scaled XP grant, 30 xp
  per finished year (1-2 years, 60 xp cap) — see **Simple Student's
  parameter-scaled XP grant (D35, Phase 2 C3)** below
- `virtue.trained_assassin` (ArMDE:5153-5156) — XP grant 50
- `virtue.venditor` (ArMDE:5207-5210) — XP grant 50


## Life-stage experience (M6/6b2) — `life_stage.rs`

Abilities are bought with experience earned in blocks, not from one bank:

> Abilities represent a character's learned abilities. For grogs and companions
> they are acquired in two blocks: early childhood, and later life. For magi, there
> are two more periods to consider: apprenticeship, and life as a magus after that.

- Source: `ArMDE:2364`.
- Implementation: `crates/arm-rules/src/life_stage.rs`. **All four periods are
  modelled**: early childhood, later life, apprenticeship (**M6/6b4**), and life as a
  magus after the Gauntlet — `ArMDE:2216`/`ArMDE:2471`'s 30 points per year (**M6/6b5**). Each
  has a section below.
- **So a magus may be built through its life stages, at its Gauntlet or long past
  it.** Its later life runs only "until apprenticeship" (`ArMDE:2214`), so
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

- Source: `ArMDE:2378`.
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
- **Childhood is granted unconditionally, so it does not wait for an age.** `ArMDE:2378`
  gives the 75 + 45 "in the first five years of life" with no further condition;
  only later life is counted in years up to an age (`ArMDE:2392`). `budget` therefore
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

- Source: `ArMDE:2380-2388` (heading
  `ArMDE:2380`, the "can be taken … for yourself, as well" sentence `ArMDE:2382`, the five
  packages `ArMDE:2384-2388`, one per line). Pricing off the Ability advancement table
  at `ArMDE:2406-2427` (the shared "ABILITY To Buy" column, whose own provenance is the
  **Abilities** section above).
- Data: `rules/core/childhoods.json` — all five packages of `ArMDE:2384-2388`, one
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
  authority here (the mirror is line-for-line, so `ArMDE:2384` is the same package in
  both languages).
  The app reads the file in `load_ruleset_from_dir` (`crates/arm-app/src/ruleset_io.rs`)
  alongside the other `core/*.json`, with the same "an empty file means the ruleset
  ships none" idiom `life_stages.json` uses — so a ruleset may legitimately offer no
  packages, per `ArMDE:2382`. `childhoods.json` is the ninth `i18n/<lang>/` file the
  localized ruleset merges, and it is required in every language's directory.
- Implementation: `crates/arm-rules/src/childhood.rs` — `ChildhoodPackage` /
  `ChildhoodEntry`, with `spread_xp` and `native_xp` pricing a package against
  `AdvancementTable::xp_for_score`, and `slots` naming the parameters a UI must
  ask for.
- **A package is a shortcut, never a restriction.** `ArMDE:2382` says the packages
  *can* be taken and explicitly keeps hand-spending open ("you can spend the 45
  experience points for yourself, as well"), so no character type requires one and
  taking none is not a validation issue. What a package may buy is unchanged: the
  closed eleven-ability spread list of `ArMDE:2378` above.
- **Every package prices to exactly 45 + 75.** Verified against the advancement
  table for all five: spreads 15+15+15 (Athletic), 15+5+5+5+15 (Exploring),
  15+15+15 (Mischievous), 15+15+15 (Social), 5+5+15+5+15 (Traveling) = **45**
  each, and every `Native Language 5` = **75**. That identity is why a package
  needs no budget of its own — it is one way of spending the two childhood blocks
  `ArMDE:2378` already grants, so `spread_xp`/`native_xp` exist to *check* the shipped
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
  `ArMDE:2378` spread list; packages shipped without life-stage rules are rejected,
  since nothing could price them; and `source` ranges are not inverted. All
  errors accumulate, so a broken file reports every problem at once.
- Engine reading where the text is terse: "Native Language 5" names no language,
  because the language is the character's own choice — so the entry carries a
  `native` flag and the chosen language lives in `LifeStagePlan::native_language`.
  Likewise "Area A Lore / Area B Lore" are two instances of one parameterized
  Ability, distinguished by an entry `slot` key (`area_a`, `area_b`) rather than by
  id; Traveling's plain "Living Language 1" is the spread's second language, the
  `ArMDE:2378` "other than the character's native language", and is told apart from the
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
  deliberately **not** cross-checked against those rows — `ArMDE:2382` ("Note that you can
  spend the 45 experience points for yourself, as well") leaves a taken package open
  to adjustment, so scores that no longer match the package are legal rather than an
  error. The parameterized slot values are not stored either: they persist as the
  rows' own `parameter` values, and a second copy could only diverge from them. The
  field is additive (`serde(default, skip_serializing_if)`), so `SCHEMA_VERSION`
  stays 14 and no migration is needed.
- **A stored package does not narrow the 45-point pool.** Taking one leaves the
  spread pool's eligibility exactly as `ArMDE:2378` sets it — the closed eleven-ability
  list — rather than restricting it to the abilities the package names. No passage
  forbids the other eight once a package is taken, and `ArMDE:2382` invites precisely
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
- **The spread's second language may not be the native one.** `ArMDE:2378` lists what the
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
  stays uncross-checked, per `ArMDE:2382` above.
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

- Source: `ArMDE:2392`.
- Data: `rules/core/life_stages.json` → `later_life.xp_per_year` 15.
- Implementation: `life_stage.rs` — `LaterLifeRules`, `later_life_years`
  (stop age − childhood years − apprenticeship years, where the stop age is the
  character's own age, or a magus's Gauntlet age) and `budget`. **For a grog or
  companion later life is the general pool**, since it funds anything the character
  may learn. For a magus it is neither the whole span nor the general pool — see
  **Pre-apprenticeship experience buys Abilities only** below.
- The Virtue requirement this passage restates for Academic/Arcane/Martial
  Abilities (`ArMDE:2315` names Educated / Arcane Lore / Warrior) **landed in M6/6b2b** —
  see **Access to Academic / Arcane / Martial Abilities** below. Supernatural keeps
  its own stricter check (`supernatural_ability_requires_virtue`).

#### Apprenticeship — 240 experience points across Arts and Abilities (M6/6b4)

> The fifteen years of apprenticeship give the character 240 experience points, and
> 120 levels of spells. These experience points can be spent on Arts or Abilities,
> including Arcane, Academic, and Martial Abilities. Note that magi can only spend
> experience points on Arcane, Academic and Martial Abilities before apprenticeship
> if they have a Virtue which allows them to do so. A sensible division is to spend
> 120 experience points on Abilities and 120 on Arts.

- Source: `ArMDE:2435` (heading
  `#### Magus Only — Apprenticeship` at `ArMDE:2433`), the four periods at `ArMDE:2364` ("For
  magi, there are two more periods to consider: apprenticeship, and life as a magus
  after that"), and the Darius example at `ArMDE:2439-2449` — whose master "picks 10 as a
  nice, round number" for the start of apprenticeship (`ArMDE:2402`) and who spends "his
  last 5 exp on Parma Magica 1" "just before Gauntlet" (`ArMDE:2449`).
- Data: `rules/core/life_stages.json` → `apprenticeship`: `years` 15, `xp` 240,
  `default_gauntlet_age` 25 (below), plus the two Ability lists (below). Canonically
  sorted, so `apprenticeship` leads the file.
- Implementation: `life_stage.rs` — `ApprenticeshipRules`,
  `LifeStageRules::apprenticeship_of` (the block for a magus, `None` for anyone else,
  read off the profile's `hermetically_trained` flag alone — deliberately
  **not** the entity-level union, D56/A0: an Abandoned Apprentice never
  completed a Gauntlet) and `budget`, which reports
  `apprenticeship_years` / `apprenticeship_xp` as a fourth block. `total()` sums all
  four.
- **The 120 spell levels are deliberately NOT here.** They ship as
  `EntityTypeProfile.spell_levels: 120` on the magus profile
  (`rules/core/character_types.json`), whose single selector is
  `effective::spell_levels_base` — see **Spell-levels budget — 120 at creation** in
  the *Hermetic spells* section above, which cites this same `ArMDE:2435`. The two numbers
  of one sentence live in two files on purpose: the spell budget belongs to the
  character *type* (a per-character `spell_levels_override` may replace it), while the
  240 belongs to the life-stage block. Duplicating either would create a second place
  to edit it.
- **Apprenticeship is the general pool.** Two independent facts force it. (a) `ArMDE:2435`
  lets this experience buy "Arts or Abilities", and only the general pool may fund an
  Art — `pool_covers` returns false for every `(Ability pool, Art spend)` pair
  (`effective/xp.rs`). (b) `Effect::GeneralXp` **already means apprenticeship
  experience**: Skilled Parens grants "an additional 60 experience points … during
  apprenticeship" (`ArMDE:4966`) and Weak Parens "60 fewer … from apprenticeship"
  (`ArMDE:7074`), and `general_xp_bonus` is applied to the general pool alone. Assigning
  apprenticeship anywhere else would move that ±60 onto a child's money.
  `xp_allocation` therefore selects `base_general` in exactly one place:
  `apprenticeship_xp + post_gauntlet_xp` for a magus (the years after the Gauntlet buy
  Arts too, `ArMDE:2471` — see **Life as a magus after the Gauntlet** below), later life for
  anyone else, the typed `xp_pool` without a plan.
- **No `general_xp` field on `LifeStageBudget`.** The pool the solve funds from is
  base + bonus (240 at the Gauntlet, 300 with Skilled Parens, more once the
  post-Gauntlet experience joins it); a budget field holding only the base
  would disagree with it and would have to be excluded from `total()`. The real pool
  is surfaced instead, as `EffectiveScores.xp_general_pool` (`arm-app/effective_dto.rs`).
- **`apprenticeship_start_age` is not stored.** Apprenticeship is fifteen fixed years,
  so the span before it follows from the age the character was gauntleted at:
  `later_life_years = gauntlet_age − childhood.years − apprenticeship.years`. Verified
  against `ArMDE:2402`, where a boy apprenticed at 10 has "75 experience points to spend from
  those five years" — exactly what a magus gauntleted at 25 earns here, whatever its age
  now. In M6/6b4 the Gauntlet age *was* the age, because a magus was generated standing
  at its Gauntlet; **M6/6b5** stores it as `LifeStagePlan::gauntlet_age` and counts the
  years after it separately — see the next section. An absent value read as the age
  itself until **Default Gauntlet age** (below) gave it the rulebook's own baseline
  instead. `SCHEMA_VERSION` is unchanged (14) throughout and no save migrates.
- Load-time gate (an engine invariant, not a sourced rule): a ruleset that declares a
  `hermetically_trained` profile **and** ships life-stage rules must declare an apprenticeship
  block — `Ruleset::validate_apprenticeship_refs`, gated exactly like
  `validate_engine_required_roles`. Without the block such a ruleset would cost a
  magus as a companion, counting every year to its age. **This replaces the 6b2
  runtime refusal** (`life_stage_magus_guided_unsupported`, now deleted): the limit was
  never a property of a character, it was missing data.

#### Default Gauntlet age — 25, the baseline a blank field reads as

> These templates are of a stereotypical member of each House, 25 years old and just
> out of apprenticeship.

- Source: `ArMDE:1601` (the Magus Templates
  preamble, under `### Magus Templates` at `ArMDE:1599`). This is the **strongest** citation
  available: it is the only line that states the number 25 and ties it to "just out of
  apprenticeship" in one sentence, rather than leaving it to be added up. Two passages
  corroborate it and neither replaces it — the Darius example apprentices the boy at
  10 ("he picks 10 as a nice, round number", `ArMDE:2402`) which plus "the fifteen years of
  apprenticeship" (`ArMDE:2435`) lands on 25, and the same example then counts his years as
  a magus "from 26 to 33" (`ArMDE:2486`), i.e. forward from a Gauntlet at 25. The templates
  themselves carry `Age: 25 (25)` (e.g. `ArMDE:1609`).
- Data: `rules/core/life_stages.json` → `apprenticeship.default_gauntlet_age` 25.
- Implementation: `life_stage.rs` — `ApprenticeshipRules::default_gauntlet_age`, read
  by `LifeStageRules::budget` as `plan.gauntlet_age.or(block.default_gauntlet_age)`,
  then clamped to `Entity::age` exactly as a stored value is. Surfaced to the frontend
  through the whole `life_stages` block (`ui/src/lib/types.ts` →
  `ApprenticeshipRules.default_gauntlet_age`), where
  `LifeStagePanel.svelte` shows `min(default, age)` as the Gauntlet-age field's
  **placeholder** — so the empty field states what the engine will do with it.
- **Why it is a default and not a prefill.** Nothing is written into
  `LifeStagePlan::gauntlet_age`, so the save still stores only the player's own choice
  and an untouched field leaves the entity — and the unsaved-changes dirty flag —
  alone. A prefill would also have to be written *before* the age is known, and a
  stored 25 against an age of 22 typed later would raise
  `life_stage_gauntlet_age_after_age`, a blocking error the blank field never
  produced.
- **No new finding becomes reachable.** `life_stage_gauntlet_age_after_age` tests
  `plan.gauntlet_age` — the stored `Option`, still `None` here — so the baseline
  cannot trip it. `life_stage_age_before_gauntlet` compares
  `LifeStageBudget::gauntlet_age` against `minimum_gauntlet_age()` (20), and the clamp
  keeps that value at `min(25, age)`: for `age < 25` it is the age, exactly what the
  validator saw before, and for `age >= 25` it is 25, which clears the floor. The
  lab-season and spell-level ceilings both *rise* with the post-Gauntlet years the
  baseline grants, so they refuse strictly less than before.
- **Optional, so nothing migrates.** `None` for a ruleset that states no baseline,
  which then keeps the pre-existing reading (a plan with no Gauntlet age means the
  magus stands at its Gauntlet). `SCHEMA_VERSION` is untouched: this is ruleset data,
  not save data.

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

- Source: `ArMDE:2216` (the character-creation
  step), `ArMDE:2471` (the rate) and `ArMDE:2482` (lab work), under the heading
  `#### Magus Only — After Apprenticeship` at `ArMDE:2467` — the fourth of the periods
  `ArMDE:2364` names: "apprenticeship, and life as a magus after that".
- Data: `rules/core/life_stages.json` → `post_apprenticeship`: `points_per_year` 30
  (`ArMDE:2471`), `lab_season_cost` 10 and `max_charged_lab_seasons_per_year` 3 (`ArMDE:2482`).
  Canonically sorted, so the block closes the file after `later_life`.
- Implementation: `life_stage.rs` — `PostApprenticeshipRules`, carried by
  `LifeStageRules::post_apprenticeship` as an additive `Option` with
  `skip_serializing_if`, exactly like `apprenticeship`: `ArMDE:2364` calls these "two
  **more** periods", so a ruleset with no Hermetic magi ships neither block and writes
  neither key. The player's three choices live on `LifeStagePlan` (`gauntlet_age`,
  `post_gauntlet_lab_seasons`, `post_gauntlet_spell_levels`), the derived figures on
  `LifeStageBudget` (`gauntlet_age`, `post_gauntlet_years`, `post_gauntlet_points`,
  `post_gauntlet_spell_levels`, `post_gauntlet_xp`).
- **Points, not experience points.** `ArMDE:2471` makes each point fungible — "an experience
  point in an Art or Ability or one level of spell" — and the player decides which each
  one becomes. Hence `points_per_year` where apprenticeship says `xp`: `ArMDE:2435` grants
  its 240 experience and 120 spell levels as two separate, non-interchangeable numbers,
  which is why those two live in two files (above) while this one number does not split.
- **Transcription trust gate:** `lab_season_cost × max_charged_lab_seasons_per_year`
  must equal `points_per_year`, and `lab_season_cost` may not be 0 —
  `Ruleset::validate_post_apprenticeship_rules`. The identity is not tidiness, it **is**
  `ArMDE:2482`: the deduction runs "to a minimum of 0 if three or four seasons are spent", so
  three seasons at 10 have to cancel the yearly 30 exactly. A year that overshot would
  have lab work take points it never granted; one that fell short would still pay a
  magus who spent the whole year in the lab. Same idiom as re-pricing the
  apprenticeship's `recommended_xp` off the advancement table. The **fourth** season is
  free because `ArMDE:2482` has already reached 0 by the third — hence `max_charged`, not a
  cap on seasons.
- Load-time gate (an engine invariant, not a sourced rule): a ruleset that declares a
  `hermetically_trained` profile **and** ships life-stage rules must declare the
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
  seasons             = min(plan.post_gauntlet_lab_seasons, 4 × post_gauntlet_years)
  charged_seasons     = (seasons ÷ 4) × max_charged_lab_seasons_per_year
                        + min(seasons mod 4, max_charged_lab_seasons_per_year)
                        # F1 (Norbert 2026-10-03): packed into full lab years
  post_gauntlet_points      = post_gauntlet_years × points_per_year
                              − charged_seasons × lab_season_cost
  post_gauntlet_spell_levels = min(plan.post_gauntlet_spell_levels, post_gauntlet_points)
  post_gauntlet_xp           = post_gauntlet_points − post_gauntlet_spell_levels
  ```

  Every step saturates, so no stored value can underflow a figure; a stored total that
  the years cannot pay for is a validation finding, not a negative budget.
  `LifeStageBudget::total()` adds **`post_gauntlet_xp` only** — a level of spell is not
  experience (`ArMDE:2471` has the player split the points), and folding it in would spend it
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
  - **Read for a magus only, and clamped to the age.** `ArMDE:2216` is "**Hermetic Magi Only
    (Optional):** Years after apprenticeship", so the stored age is read for a character
    whose profile serves an apprenticeship and ignored on any other plan. Gating on the
    *points* being zero instead would let a hand-edited companion plan carrying
    `gauntlet_age: 25` at age 60 silently lose 35 later-life years (525 experience
    points). It is clamped to the age because Advisory and Silent validation do not
    block a Gauntlet later than the character's own.
- **`post_gauntlet_lab_seasons` is one total of *charged* seasons, not a per-year list.**
  The deduction stops at the third season of a year (`ArMDE:2482`), so every legal per-year
  distribution totals at most `3 × years`, every total in that range is realizable, and
  all of them cost the same — one number is lossless. It must count *charged* seasons:
  sixteen seasons actually worked cost 0 points across four years but 30 across five, so
  a single total of worked seasons could not tell those apart.
- **The post-Gauntlet experience joins the *general* pool — it is not a block of its
  own.** `effective/xp.rs`'s `xp_allocation` selects `base_general` as
  `apprenticeship_xp + post_gauntlet_xp` for a Hermetically trained entity
  (`is_hermetically_trained`, D56/A0 — saturating), leaving the untrained arm and the
  plan-less `xp_pool` arm untouched. Three sourced facts force
  the general pool rather than a restricted one:
  - `ArMDE:2216` — "Divide 30 points per year between experience points in **Arts**,
    experience points in Abilities, and levels of spells" — and `ArMDE:2471` — "Each point
    can be an experience point in an **Art** or Ability or one level of spell". Only
    the general pool may fund an Art (`pool_covers` returns false for every
    `(Ability pool, Art spend)` pair), so a restricted pool could not express this.
  - The Academic/Arcane/Martial gate does not narrow them: `ArMDE:2435` restricts what a
    magus may spend "**before** apprenticeship", and `ArMDE:7151` puts the years after it on
    the permitted side — "Magi without a specific Virtue may only buy Academic Abilities
    during or after apprenticeship". The whole-character waiver of `ArMDE:7151` already
    applies to a magus, so nothing further is needed.
  - Consequently **no** `LifeStageBlock` variant, **no** `RestrictedXpPool`, and **no**
    `xp-pool-<slug>` Fluent key were added: the block that funds anything is the general
    pool, which is the one block with no slug. Its size is surfaced as
    `EffectiveScores.xp_general_pool`, exactly as apprenticeship's was.
  - Later life is untouched by this and stays the restricted, Abilities-only pool of
    `ArMDE:2435`'s "before apprenticeship" clause however many years the magus has lived
    since (`post_gauntlet_years_leave_later_life_restricted`).
- **The spell levels are *additive* to the profile's 120 — not a second budget.**
  `effective/spell.rs`'s `life_stage_spell_levels(entity, ruleset)` (the budget's
  `post_gauntlet_spell_levels`; 0 without a plan or without the block) is folded into
  `spell_levels_budget`, which is `base + spell_levels_bonus +
  life_stage_spell_levels`, clamped as before. Apprenticeship's "120 levels of spells"
  (`ArMDE:2435`) are the magus profile's `spell_levels` in
  `rules/core/character_types.json`, which is what `spell_levels_base` selects; the
  post-Gauntlet levels are the player's chosen slice of the fungible 30 points a year
  (`ArMDE:2471`) — the same reason `post_gauntlet_xp + post_gauntlet_spell_levels ==
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
  - **App payload:** `EffectiveScores` (`arm-app/effective_dto.rs`) carries the three
    parts of the budget separately — `spell_levels_profile_base`,
    `spell_levels_bonus` and `spell_levels_life_stage`, whose identity is
    `base + bonus + life_stage == spell_levels_budget` — so the spell-levels bar can
    label each rather than showing an unexplained total, the way the XP bar lists its
    extra pools beside the general pool. Mirrored by hand in `ui/src/lib/types.ts`
    (`EffectiveScores`, `LifeStagePlan`'s three choices, and the new
    `PostApprenticeshipRules` on `LifeStageRules`), pinned by
    `every_life_stage_field_is_mirrored_in_the_frontend_types`. The views that render
    them are **M6/6b5b**.
- **Five findings, and the reason the clamps stay.** The first four are
  `validate_post_gauntlet_choices` in `validation/life_stage.rs`, all
  `error`/`experience`, all magus-gated because `ArMDE:2216` is "**Hermetic Magi Only
  (Optional)**" and on any other plan the values are ignored outright:
  - `life_stage_gauntlet_age_after_age` (`gauntlet_age`, `age`) — a Gauntlet in the
    character's future.
  - `life_stage_lab_seasons_out_of_range` (`seasons`, `max`, `years`) — more lab
    seasons than the `4 × post_gauntlet_years` the span holds (`ArMDE:2482`, "three or
    four seasons"; F1, Norbert 2026-10-03). The book's Darius (`ArMDE:2486`, :2488; 9
    years, one full lab year) is worth 240, because the stored total is read as packed
    into full lab years by `charged_lab_seasons` in `crates/arm-rules/src/life_stage.rs`.
    Recording seasons per year is planned; it needs a save-format change. The message
    names `max` as the seasons the years hold, never as seasons that can be charged.
  - `life_stage_lab_seasons_without_years` (`seasons`) — the same rule (`ArMDE:2482`) on a
    plan with **no** post-Gauntlet year, which is a different fault told a different
    way. The branch is on `post_gauntlet_years == 0`, not on the ceiling being 0: a
    zero span means the seasons have nowhere to happen, and the per-year charging
    limit the sibling message used to recite explains nothing about it. Both
    messages were shortened at the same time — the "the third already takes the whole
    30, so a fourth is free" gloss belonged to the field's own
    `life-stage-lab-seasons-hint`, not to a finding, and manual-testing-findings #21
    then removed that hint too: the charging rule is in the rulebook, and the engine
    still enforces it.
  - `life_stage_spell_level_split_exceeds_points` (`levels`, `points`) — a spell-level
    share larger than the points the years granted (`ArMDE:2471`).
  - The fifth is `aging_rolls_pending` (warning, `aging`), which the
    post-Gauntlet years make reachable at all: a magus generated years out of
    apprenticeship is routinely over 35, and "a character over the age of 35 must make
    aging rolls before the game begins" (`ArMDE:2232`, `ArMDE:16565`). It is emitted from
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
  `ArMDE:2435`) is compared against `LifeStageBudget::gauntlet_age` — the resolved value
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

- Source: `ArMDE:2213-2216` (the numbered
  creation summary, steps 5-8) and `ArMDE:2364` (the same four blocks named as periods, in
  the same order). Early childhood's two figures are **one** block, not two:
  `ArMDE:2378` grants "75 experience points in their native language … and 45 experience
  points to divide between" the spread, in one sentence about "the first five years of
  life". The span later life covers is worked through by the Darius example at
  `ArMDE:2402` — apprenticed at 10 after a childhood ending at 5, he "has 75 experience
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
  life, and `ArMDE:2213-2216`/`ArMDE:2364` state them in the order they are lived. Rendered with
  later life last, its label read as the years *after* the Gauntlet — a chronology
  asserted wrongly, which is a misstatement of `ArMDE:2214`'s "until apprenticeship for
  magi" rather than an aesthetic complaint.
- **"After the Gauntlet", not "As a magus"** (`ArMDE:2216`, "Years after apprenticeship").
  The block is driven by `LifeStagePlan::gauntlet_age`, so naming it for the Gauntlet
  ties the label to the field that moves it. Renamed in both bars —
  `xp-pool-block-after-gauntlet` and `spell-levels-post-gauntlet` — and **only** on
  those two labels: the six strings that say "years as a magus" are prose describing
  the span, and `ArMDE:2216` calls the same period "life as a magus after that" (`ArMDE:2364`),
  so they read correctly and keep it.
- German: `Frühe Kindheit`, `Späteres Leben`, `Lehrlingszeit` and
  `Nach der Lehrlingsprüfung`, from the mirrored German lines
  `Ars Magica Definitive Edition Basisregeln.md:2213-2216` (heading `ArMDE:2376`, `ArMDE:2390`)
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

- Source: `ArMDE:2213-2216` — the numbered
  creation sequence, and the decisive statement of *which* period may buy Arts: step 6
  (`ArMDE:2214`) says "any **Abilities**", step 7 (`ArMDE:2215`) "between Hermetic **Arts** and
  … Abilities". Restated for the block itself at `ArMDE:2392`.
- Implementation: `effective/xp.rs` — `xp_allocation` pushes later life as a
  **restricted** `PoolEligibility::Ability` pool for a Hermetically trained
  entity (`is_hermetically_trained(entity, ruleset, profile) && budget.later_life_xp
  > 0`, D56/A0 — the same shape of guard the mastery pool uses), so Arts
  fall out for free: an Ability pool never covers an Art spend. `LifeStageBlock` gains
  `LaterLife` (slug `later_life`, labelled `xp-pool-later_life` in both locales) so the
  bar can name the row.
- **The reason, not just the wording:** a magus's later life *ends where
  apprenticeship begins* (`ArMDE:2214`, "until apprenticeship for magi"), so it is the span
  in which the character is a child not yet taken as an apprentice — and has no Arts at
  all. Funding an Art from it would be funding it before the Gift was ever opened.
- Where an experienced magus's Arts actually come from: `ArMDE:2216`/`ArMDE:2471`'s "For every
  year, the magus gets 30 points", the years *after* apprenticeship — modelled in
  **M6/6b5**, whose experience joins the **general** pool precisely because only that
  pool may fund an Art. So later life stays Abilities-only however many years the
  magus has lived since its Gauntlet
  (`post_gauntlet_years_leave_later_life_restricted`), and a magus is no longer built
  standing at its Gauntlet: the age past it is spent through
  `LifeStagePlan::gauntlet_age`. Nothing in the source bounds the apprenticeship start
  age — Darius's master "picks 10 as a nice, round number" (`ArMDE:2402`) — so no such
  bound is enforced.

#### Pre-apprenticeship experience may not buy Arcane, Academic or Martial Abilities (M6/6b4)

> Note that magi can only spend experience points on Arcane, Academic and Martial
> Abilities before apprenticeship if they have a Virtue which allows them to do so.

- Source: `ArMDE:2435` (second sentence),
  with `ArMDE:2215`'s "any nonSupernatural Abilities (or Supernatural Abilities, if the
  magus has the relevant Virtue)" for apprenticeship itself. The Darius example
  reasons exactly this way about a pre-apprenticeship purchase: Order of Hermes Lore
  "It's a **general** Ability, so he can" (`ArMDE:2402`).
- Implementation: the later-life pool's eligibility (`effective/xp.rs`,
  `xp_allocation`) — `categories` is `AbilityCategory::ALL` minus
  `ruleset.categories_requiring_virtue()` plus whatever the character's Virtues
  authorize, and `abilities` is the authorized ids. Both come from
  `effective::ability_authorizations`, moved here from `validation/authorization.rs` in
  this slice so the ownership check and the pool read one function. (Direction matters:
  `validation` already depends on `effective`, so calling the other way would invert
  the layering.)
- **No double-reporting with `validate_ability_authorization`.** That validator gates
  *owning* a gated Ability and exempts magi whole-character (`ArMDE:7151`, "or if they are
  magi"); the pool decides *whose money* pays. A magus who overspends its
  apprenticeship gets `not_enough_xp`, never `ability_category_requires_virtue`. The
  two mechanisms are disjoint by construction.
- **Supernatural stays in the pool's category set and legalizes nothing.** Access to
  each Supernatural Ability is granted per Ability (`ArMDE:2315`), which
  `validate_supernatural_abilities` enforces for magi as well — so an unauthorized one
  is already an error and funding it here changes nothing. Excluding it would only
  produce a second, differently-worded complaint about the same row.
- **Deliberate asymmetry: later life stays the general pool for a non-magus.** A grog
  or companion is gated by an error on the character
  (`ability_category_requires_virtue`), so its money needs no restriction; a magus's
  category gate is waived whole-character, so the pool is the only place the "before
  apprenticeship" half of `ArMDE:2435` can live. This is also where `ArMDE:7151`'s finer
  distinction now bites — see **Access to Academic / Arcane / Martial Abilities**
  below, whose "not modelled" note this slice narrows.

#### Hermetic minimum Abilities — Parma Magica 1, Magic Theory 1, Latin 1 (M6/6b4)

> Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory 1,
> Latin 1. Characters with lower scores would not be admitted to the Order.

- Source: `ArMDE:2437`.
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
  (`arm-app/effective_dto.rs`) hands the whole checklist to the frontend — so a finding
  and the checklist a UI shows cannot disagree.
- **An error, and unconditional.** "Would not be admitted to the Order" is a hard bar,
  and `ArMDE:2437` says nothing about how the experience was earned, so a magus built from a
  flat `xp_pool` is held to it exactly as a guided one is. That is why the validator
  lives in `validation/magus.rs` and not in `validation/life_stage.rs`, which returns
  early without a life-stage plan.
- **L2 (try-out finding 8, D83.12): "Latin 1" is Latin.** The old reading matched by
  Ability id, so a magus whose only dead language was Greek passed `ArMDE:2437`; it was
  kept because languages were free text with no catalogue. CV and L1a since gave Dead
  Language its own catalogue (Latin, Hebrew, Gothic; `ArMDE:7432`), so both Latin rows
  now carry `"parameter": "language.latin"` and the check reads the bought instance
  through `catalogue.rs::instance_is`, the one "is this instance language X" helper:
  - a `Catalogued` value counts when its id is `language.latin`;
  - typed text counts when it equals (trimmed, case-folded) Latin's name in any locale
    the rules ship ("latin", "Latein"). The load fold already converts such text on
    reopening; for the session the app attaches every shipped locale's catalogue names
    to its ruleset (`Ruleset::with_catalogue_names`, filled by
    `arm-app/ruleset_io.rs::catalogue_names_in_every_locale`). A ruleset with no names
    attached matches ids only. Text spelling the id ("language.latin") never counts.
  - Integrity: a requirement's `parameter` on a catalogued Ability must be a value of
    its own catalogue, or the load fails naming it
    (`integrity.rs::check_value_in_own_catalogue`).
  - Pinned by `latin_is_matched_by_catalogue_value_not_by_ability_id` (it replaces
    `latin_is_matched_by_ability_id_not_by_instance`) and
    `tests/l2_language_requirements.rs`, `tests/l2_typed_language_in_session.rs`, and
    `arm-app/tests/l2_language_requirements.rs`.
  The earlier "the widening is permanent, a language catalogue is rejected" ruling
  (guided-creation review #32) was superseded by CV/D59 and is withdrawn here.
- **The requirement carries the rules' own wording as a label.**
  `AbilityRequirement::exemplar` holds a **language-neutral slug** — `"exemplar":
  "latin"` — on the two sites where the rules demand Latin by name:
  `rules/core/life_stages.json` `minimum_abilities`
  (`dead_language ≥ 1`, `ArMDE:2437`) and `recommended_abilities` (`≥ 4`, `ArMDE:2455`).
  (The scholarly-language expectation lists its languages instead since L2; see its own
  section.) Its **translated text lives in
  `rules/i18n/<lang>/abilities.json`** under `exemplar.latin` (`Latin` / `Latein`, the
  latter as the German rulebook uses it at the mirrored `ArMDE:2437`), so no translatable
  string enters the mechanics file. `MagusMinimumAbility` carries it through to the
  frontend and the validator emits it as an **optional** `exemplar` arg, so the
  minimums row and the `issue-magus_minimum_ability` /
  `issue-magus_recommended_ability` messages read identically:
  *"Latin 1 (Dead Language) is not met"*. The exemplar **heads** the requirement so
  the score follows it directly, exactly as `ArMDE:2437` states it, and the Ability it is
  bought as trails the score as a note (L2: it said "(any Dead Language)" while the
  check was id-only); it used to read *"Dead Language (e.g. Latin) 1"*, which put the
  example between the Ability and its score and buried the demand
  (guided-creation-review-2026-08 #12). The one shared label path is
  `requirementAbilityLabel` + `requirementExemplarNote` in `ui/src/lib/derive.ts`,
  joined by the `requirement-exemplar` Fluent string and a `qualifier` message
  variable.
- **The exemplar slug is a LABEL KEY, not a referential-integrity `ref`.** It resolves
  against no catalogue, so the loader deliberately does not check it (documented at
  `ruleset/integrity.rs::validate_apprenticeship_refs`) while the requirement's
  `ability` **is** a ref and still fails loudly, and so is its `parameter` on a
  catalogued Ability (L2). Pinned by
  `an_exemplar_slug_is_not_treated_as_a_referential_integrity_ref`, which also proves
  the test is not vacuous by showing a bogus `ability` still rejected. Its i18n
  coverage in both locales is pinned by `the_exemplar_slug_resolves_in_both_locales`,
  and its presence on the two sites by
  `the_magus_minimum_dead_language_requirement_names_its_exemplar`.
- **The score tested is the BOUGHT one**, not the effective one:
  `effective_ability_score` returns 2 for a magus with a Puissant Parma Magica and no
  Parma row at all, and `ArMDE:2437`'s "scores" cannot mean a Virtue's +2 to *use*. The same
  `entry.score >= min_score` test `validate_academic_language` applies. General ruling:
  the age caps constrain the bought score, and the minimums test it. Pinned by
  `puissant_parma_magica_does_not_admit_a_magus_to_the_order`. Virtue/Flaw
  prerequisite minimums follow the same ruling (D83.5; see the Effect layer's
  `ability_bonus` bullet), except that they also count a granted floor.
- **A minimum age of 20 follows** from the same block: childhood (5) plus
  apprenticeship (15), `LifeStageRules::minimum_gauntlet_age`. A younger magus with a
  life-stage plan gets `life_stage_age_before_gauntlet` (error, `experience`, args
  `age`/`min`) **instead of** `life_stage_age_before_childhood` — one wrong age, one
  finding, under the code that describes it truthfully.
- **The minimum set is deliberately not re-priced** the way the recommended one is:
  `ArMDE:2437` states no total, so a pricing check could only compare the engine to itself.

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

- Source: `ArMDE:2451-2461` (heading
  `ArMDE:2451`, the four Abilities `ArMDE:2453-2459`, the total `ArMDE:2461`), with the consequence
  half of `ArMDE:2437`: "A character without a Latin score of least 4 and an Artes
  Liberales score of at least 1 is unable to read the books of the Order… A Magic
  Theory score of below 3 is weak, and, in particular, means that the magus cannot set
  up his own laboratory."
- Data: `rules/core/life_stages.json` → `apprenticeship.recommended_abilities` +
  `recommended_xp: 90`.
- Implementation: the same `magus_minimum_abilities` rows, tagged
  `AbilityRequirementKind::Recommended`, reported by
  `validate_magus_minimum_abilities` as `magus_recommended_ability` (**warning**,
  `abilities`, args `ability`/`min`/`score` plus an optional `exemplar` — the Latin 4
  row states one, `ArMDE:2455`). Since L2 the Latin 4 row also names
  `"parameter": "language.latin"` and is matched exactly like the minimum above.
- **A warning, not an error**, because `ArMDE:2451` calls the list *recommended* and the
  consequences `ArMDE:2437` spells out describe a weak magus, not an illegal one — unlike
  `ArMDE:2437`'s own three, which decide admission.
- **The 90 is a re-pricing trust gate.** `Ruleset::validate_apprenticeship_refs`
  prices the list off the Ability advancement table and refuses any other total:
  5 (`ArMDE:2408`) + 50 (`ArMDE:2411`) + 30 (`ArMDE:2410`) + 5 = 90 exactly. A mistyped score fails
  the load instead of shipping a recommendation the rulebook never costed — the gate
  `validate_childhood_packages` applies to childhood's 45 and 75. The literals are also
  asserted from outside the data, in
  `tests/data_integrity.rs::shipped_apprenticeship_carries_the_2435_and_2437_numbers`,
  so an edit moving list and total together cannot pass in silence.
- Parma Magica 1 appears in **both** lists (`ArMDE:2437` and `ArMDE:2459`), so it legitimately
  produces two checklist rows — one required, one recommended. Not a duplicate.
- **Not modelled:** `ArMDE:2459`'s "(should be no higher if the magus is just out of
  apprenticeship)". It is advice about what apprenticeship *teaches* — "this Ability is
  normally the last thing taught" (`ArMDE:2437`) — and the engine has no per-stage
  attribution for a bought score to hang a maximum on, the same limit recorded for
  `ArMDE:7151` below.

#### Wealthy / Poor — the rate, and who may take them

> Characters with the Wealthy Virtue get 20 experience points per year, while
> characters with the Poor Flaw get 10 experience points per year. Note that only
> companions can take this Virtue or Flaw.

- Source: `ArMDE:2394`; the items
  themselves at `ArMDE:5235-5238` (Wealthy) and `ArMDE:6594-6596` (Poor, which repeats "In
  particular, this Flaw is not available to magi").
- Data: `rules/core/virtues_flaws.json` — `virtue.wealthy` and `flaw.poor` each
  carry `later_life_xp_rate` (20 / 10). **`flaw.poor` was missing from the
  catalogue entirely before this milestone** and is added here; German name "Arm"
  (`rules/source/de/translation-tables/tugenden-fehler.md:445`, corroborated by the
  German source at the mirrored `ArMDE:6594-6596`).
- Implementation: `Effect::LaterLifeXpRate` (`types.rs`), read by
  `LifeStageRules::later_life_rate`. The effect **replaces** the base rate rather
  than adjusting it, because the passage states the whole rate. Engine reading where
  the text is silent: if several selections ever name a rate, the lowest applies —
  nothing ranks them, so this is the conservative and deterministic choice.
- **D47/X7a — Guild Apprentice suppresses both, until the journeyman rank.**
  `ArMDE:4041-4044`: "The character is not able to benefit from either the Poor
  Flaw or the Wealthy Virtue … until he moves to the journeyman rank."
  `virtue.guild_apprentice` carries `Effect::SuppressesLaterLifeXpRate`
  (`types.rs`; X7a-refactor — formerly a hardcoded check for
  `virtue.guild_apprentice`'s id in `LifeStageRules::later_life_rate`, now
  data on the entry itself, `classification` moved `uncomputed_rule` →
  `in_play_effect`). `later_life_rate` checks the entity's selections for any
  item carrying that effect first and, if present, returns the ruleset's base
  rate outright — skipping the `LaterLifeXpRate` fold entirely rather than
  adding an `incompatible_with` (D47 rejects a general "nullify any effect"
  mechanism, hence the narrow, single-purpose marker effect rather than a
  generic suppression variant). Both `virtue.wealthy` and
  `virtue.guild_apprentice` state the interaction in `description` (both
  locales); test: `x7a_lab_rows.rs::guild_apprentice_suppresses_wealthys_later_life_rate`.
  **Cross-cutting note (X7a-refactor):** this reclassification makes
  `crates/arm-rules/tests/x2_reclassification.rs`'s `RECLASSIFY_WITH_DESCRIPTION`
  entry for `virtue.guild_apprentice` (which still asserts `UncomputedRule`)
  stale — that file is owned by a different slice (X2g) and was not edited
  here; the row needs removing there.
- **Eligibility is a `Prereq` on the entry, not three profile lists (D38, F-339).**
  Before this milestone the "only companions" line was enforced twice
  explicitly (`magus` and `mythic_companion` each listed both ids in
  `forbidden_traits`) and once by accident (a grog takes no Major Flaw at all,
  so Poor was merely out of reach, not stated — F-339's asymmetry). Making
  either entry Minor, or giving grogs a Major allowance, would have silently
  stopped enforcing a rule the book states, since nothing asserted the rule
  itself, only its side effect.

  Data: `rules/core/virtues_flaws.json` — `virtue.wealthy` and `flaw.poor` each
  carry `"prerequisites": { "kind": "is_companion" }`. The two `forbidden_traits`
  entries on `magus`/`mythic_companion` are removed — one statement of the rule
  instead of three. Evaluated by `validation/prereq.rs::evaluate_prereq`
  against `Prereq::IsCompanion`.

  **"Companion" is a profile-level class flag, not the exact type id
  `companion` — and Mythic Companion is inside it, not outside (owner's
  ruling, reversing this section's own earlier reading).**
  `rules/core/character_types.json` gives both `companion` and
  `mythic_companion` `"is_companion": true`; `magus` and `grog` leave it unset
  (default false).
  `Prereq::IsCompanion` (`types.rs`) reads `EntityTypeProfile::is_companion`
  exactly as `HermeticallyTrained`/`OrderMember` read their own profile flags
  (D56/A0) — never the profile's `id` — so a **future** companion-like profile
  joins this audience by setting the flag in its own JSON entry alone, with
  **zero** change to `virtue.wealthy`, `flaw.poor`, `virtue.magical_mount`, or
  any other item that already carries the prerequisite. This was chosen over
  the alternative, `Any([IsCompanion-by-id, CharacterType(mythic_companion)])`
  spelled out per item, because that form requires editing every such item's
  `prerequisites` again each time a new companion-like profile ships — the
  flag is the single point of extension the alternative is not.

  This reverses the milestone's first-round reading (recorded here until now):
  "`ArMDE:2394` says 'only companions', ... a mythic companion *is* a companion
  in the rules' sense but is a distinct profile here, so it is forbidden too."
  Norbert's clarification is unambiguous: **mythic companions are companions
  too**, for every place the rules say "companion" — this passage's "only
  companions" (D38) and `virtue.magical_mount`'s "companion or magus-level
  character" (F-553, below) alike. Tests:
  `a_companion_or_mythic_companion_may_take_wealthy_or_poor`,
  `a_magus_or_grog_may_not_take_wealthy_or_poor`
  (`tests/data_integrity.rs`).

#### Magical Mount — companion or magus-level character (F-553)

> In this case, only a companion or magus-level character can take this
> Virtue.
> — ArMDE:4375 (entry `ArMDE:4373-4376`)

Unencoded before this milestone: the entry carried no prerequisite and no
profile forbade it, so a grog could take it (F-553).

Data: `rules/core/virtues_flaws.json` — `virtue.magical_mount` carries
`"prerequisites": { "kind": "any", "value": [{ "kind": "is_companion" },
{ "kind": "order_member" } ] }`. The first disjunct is the same `IsCompanion`
flag as Wealthy/Poor above (a mythic companion counts, per Norbert's ruling).

**"Magus-level character" is undefined in the passage.** Read conservatively
as `Prereq::OrderMember` — full membership in the Order of Hermes, true only
of the `magus` profile today (`character_types.json`) — because nothing in the
text supports widening it further, and inventing a permission the book does
not state is the wrong direction to be wrong in. This was weighed against
reusing `OrderMember` unqualified per
`docs/vf-audit/design-a0-is-magus-split.md` § "Notes for E1": `virtue.redcap`
(ArMDE:4844) is a full Order member with no Hermetic training, built as a
companion in this app, which is exactly who "magus-level" is *not* reaching
for. The concern does not bite here — `order_member` is a **profile-only**
fact (`PrereqCtx::build`, D56/A0: no entity-level override), and a
Redcap-flavored companion's profile is `companion`, whose `order_member` is
`false`; such a character is admitted only through the `is_companion` disjunct,
never through `OrderMember`. Should `order_member` ever gain an entity-level
override (the open risk `design-a0-is-magus-split.md` § 1 already records),
this expression needs re-review — recorded here so that review has somewhere
to start.

Tests: `magical_mount_requires_companion_or_order_member`,
`a_companion_mythic_companion_or_magus_may_take_magical_mount`,
`a_grog_may_not_take_magical_mount` (`tests/data_integrity.rs`).

#### Access to Academic / Arcane / Martial Abilities (M6/6b2b)

> There are two exceptions. One is that a character must have a Virtue to buy
> Academic, Arcane, Martial, or Supernatural Abilities at character creation.
> Educated, Arcane Lore, and Warrior, respectively, are the easiest options for the
> first three groups, although other Virtues (and some Flaws) also grant access to
> some of these Abilities.

- Source: `ArMDE:2315`, restated for the
  later-life block at `ArMDE:2392` ("as long as the character has a Virtue that permits
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
  `ability_authorization: [{ ability: ability.dead_language, instance: { literal: "latin" } }]`
  for "You may take Latin at character creation" (`ArMDE:5867`). **Phase 2 C1
  (F-349/F-16x fix):** an id-only proxy previously permitted any dead language,
  not Latin alone — `AbilityRef`'s `instance` field (`types.rs::AbilityRef`)
  closes that: `ability_authorizations()` (`effective/xp.rs`) resolves the
  literal against the bought `AbilityScore::parameter`, so only the instance
  named actually resolves. Other access-granting Virtues are a data addition,
  never a code change.
- **Exclusive-choice and gated entries (W2/F-42/F-317/D14 shape 2, Phase 2
  C1).** `Effect::AbilityAuthorization.abilities`/`.categories` are
  `Vec<AbilityRef>`/`Vec<CategoryRef>` (`types.rs`), each optionally carrying a
  `ParamGate { param, equals }`: an entry counts toward the authorized set only
  when the OWNING selection's own parameter equals the gated value —
  conditional list membership, not value substitution (see
  `docs/vf-audit/design-c0-parameter-model.md` § 3 for why a single
  parameter-relative binding cannot express "either/or, not both" or a
  fixed-Lore-list bonus). Three entries use it:
  - `virtue.wise_one` — "You may take either Arcane or Academic Abilities, but
    not both, at character creation" (`ArMDE:5259`). A `study` parameter
    (`domain: enumerated`, values `ability_category.academic`/`ability_category.arcane`)
    gates two `CategoryRef`s. Choosing `academic` leaves the `arcane` entry's
    gate false, so it contributes nothing — the naive fix (authorizing both
    categories unconditionally) would have been an over-permission worse than
    the original bug, since it is silent.
  - `virtue.custos` — "either Martial, Academic, or Arcane Abilities. If you
    choose Martial or Arcane Abilities, you may still learn to speak Latin"
    (`ArMDE:3629-3634`). Three gated `CategoryRef`s (`study` ∈
    `{ability_category.academic, ability_category.arcane, ability_category.martial}`)
    plus one **ungated** `AbilityRef` (`ability.dead_language`, instance
    `latin`) — gated and ungated entries coexist in one list with no
    special-casing. The Latin entry is unconditional rather than only firing
    for Martial/Arcane study, since an Academic choice already covers Latin
    through its own category authorization; the passage's wording just
    explains why the two non-Academic choices need the carve-out.
  - `virtue.templar_specialist` — "one restricted group of Abilities... such
    as Academic or Martial Abilities" (`ArMDE:5133-5136`). **The open-set
    decision this note owes** (`docs/vf-audit/design-c0-parameter-model.md`
    § 3 leaves three readings open): both "such as"-hedged clauses in the
    passage — "such as craftsmen, blacksmiths, artisans, notaries, squires,
    soldiers, scribes, or translators" and "such as Academic or Martial
    Abilities" — give **examples**, not an exhaustive enumeration, unlike Wise
    One's closed "either...or" and Custos's closed "either...or...or".
    Deriving a closed set from the named *roles* does not survive scrutiny:
    the engine has no "craft" `AbilityCategory` to map craftsman/blacksmith/
    artisan onto (Craft Abilities are `general`, which needs no authorization
    at all — a role the passage lists as an example of a *restricted* group
    would then authorize nothing), so a role-based reading is not merely
    interpretive narrowing, it is internally inconsistent with the engine's
    own taxonomy. **Chosen instead: the categories the engine actually gates**
    — `study` ∈ `{ability_category.academic, ability_category.arcane,
    ability_category.martial}`, matching `rules/core/abilities.json`'s
    `categories_requiring_virtue` exactly (not `AbilityCategory::ALL` minus
    `supernatural`, C1's first draft): the passage says "one **restricted**
    group of Abilities", and `general` needs no Virtue to buy at all
    (`ArMDE:2315`), so offering it as a "restricted group" choice would
    authorize nothing and be a meaningless option — the same reasoning that
    rules out a role-based reading, applied to the category axis instead of
    the role axis. There is no clean data-model way to *derive* this list
    from `categories_requiring_virtue` at authoring time — the two live in
    separate JSON files with no cross-reference mechanism between an
    `Enumerated` parameter's `values` and another file's global list — so the
    three ids are hand-authored here and must be kept in sync by hand if
    `categories_requiring_virtue` ever changes; `data_integrity.rs`'s
    `ENUMERATED_PARAM_ITEMS` pins the current set, so a future edit to either
    file that breaks the correspondence is caught by name, not silently. This
    also makes Templar Specialist's set literally identical to Custos's own
    three categories (`virtue.custos`, ArMDE:3629-3634) — coincidental in the
    rulebook's own wording, not engineered, but confirms the reading is not
    arbitrarily narrow. The third option (`uncomputed_rule` with free `text`)
    was rejected because the mechanical shape (an exclusive Ability-category
    authorization) is not genuinely open the way a free-text descriptor is —
    Wise One and Custos already show the shape closes to a handful of
    categories, this entry's set is the same size as Custos's, just reached by
    a different reading of the passage.
- **Student of (Realm)'s gated bonus doubles as its own authorization
  (row 50(a), Phase 2 C1 — see below, "Supernatural Might & Magic
  Resistance" section neighbour `AbilityBonusGated`).** `virtue.student_of_realm`
  no longer carries a separate `ability_authorization` effect; its single
  `ability_bonus_gated` effect's four gated targets (one per realm Lore) are
  folded into the authorized-Ability set the same way an `AbilityAuthorization`
  entry is — a competence bonus tied to one Ability instance is itself
  permission to own it, since the bonus could never apply to an Ability the
  character may not buy. This is what implements the passage's own "You may
  take that Lore at character generation even if you cannot learn other
  Arcane Abilities" (`ArMDE:5054`) for **that** Lore only, fixing the previous
  unconditional four-Lore authorization (P4 in
  `docs/book-template-conformance.md`).
- **Supernatural is deliberately excluded** from this check: `ArMDE:2315` says access "is
  granted by a separate Virtue" per Ability, which the stricter, pre-existing
  `validate_supernatural_abilities` / `supernatural_ability_requires_virtue` already
  enforces. Folding it in here would double-report.
- **Magi are exempt**: `ArMDE:7151` ("Beginning characters may only purchase Academic
  Abilities if they are specifically permitted to through the purchase of a Virtue,
  **or if they are magi**") and `ArMDE:2435`, where apprenticeship experience may go on
  "Arcane, Academic, and Martial Abilities". Read from `is_hermetically_trained`
  (D56/A0's union of the profile flag with any selection carrying
  `Effect::ConfersHermeticTraining`), never a type id. `ArMDE:7151`'s finer "Magi without a specific Virtue may only buy
  Academic Abilities **during or after** apprenticeship" is now modelled **for a
  guided magus** (M6/6b4): the restriction is not on the Ability but on the money, so
  a magus's pre-apprenticeship experience simply cannot fund those categories — see
  **Pre-apprenticeship experience may not buy Arcane, Academic or Martial Abilities**
  above. It remains unmodelled for a magus funded from a flat `xp_pool`, which carries
  no per-stage attribution at all; there the exemption stays whole-character. The
  ownership check here is unchanged either way, and the two never double-report: a
  shortfall is `not_enough_xp`, never `ability_category_requires_virtue`.

#### Simple Student's parameter-scaled XP grant (D35, Phase 2 C3)

> The character is a university student who has not yet taken a degree. He is
> typically between 14 and 16 years old and somewhere along his university
> program. He receives 30 experience points per finished year that he can
> apply to Latin or Artes Liberales. If he has finished his second year of
> studies, he is in the liminal position of either applying for work or
> continuing his education.

- Source: `ArMDE:4958-4963`, rule at `ArMDE:4960`.
- **The cap is 2 finished years (60 XP), derived from the catalogue, not
  invented.** The rate — 30 XP per finished year — is a *family* mechanic
  shared with three other Virtues whose year counts and totals are stated
  outright: Baccalaureus Artium (3 years, 90 XP, `ArMDE:3472`), Magister in
  Artibus (8 years, 240 XP, `ArMDE:4389`), Doctor in Faculty (10 years, 300 XP,
  `ArMDE:3687`). A Simple Student's **third** finished year completes the
  Baccalaureus, a different Virtue with its own 90 XP, so 2 is the ceiling —
  and `ArMDE:4960` says as much ("If he has finished his **second** year...").
  An age formula (`age - 15`) was considered and rejected: it breaks on the
  perpetual student (decisions.md D35).
- Data: `rules/core/virtues_flaws.json` `virtue.simple_student` declares
  `parameters: [{ key: "years", type: { number: { min: 1, max: 2 } }, domain:
  "number" }]` and `effects: [{ type: "scaled_restricted_ability_xp", param:
  "years", per_unit: 30, abilities: ["ability.artes_liberales", { ability:
  "ability.dead_language", instance: { literal: "latin" } }] }]`. The Dead
  Language entry uses D14's literal-instance form (the same fix
  `flaw.covenant_upbringing` already carries) so the pool funds Latin
  specifically, never Ancient Greek — Simple Student was never one of D14's
  named carriers, but it needs the same fix the moment it is written. Only the
  three other family members' year counts stay fixed constants (90/240/300);
  parameterizing them for symmetry would be wrong, since the book states
  their totals outright.
- Engine: `ParamType::Number { min, max }` and `ParameterDomain::Number`
  (`types.rs`) — the domain is the redundant half of the pair and carries no
  resolution logic of its own (see its own doc comment); the bound lives on
  `ParamType::Number` and is read in
  `validation/selections.rs::param_value_resolves`'s `Number` arm. The grant
  itself is `Effect::ScaledRestrictedAbilityXp { param, per_unit, abilities,
  categories }` (`types.rs`) — the parameter-scaled sibling of
  `Effect::RestrictedAbilityXp`, on the `CharacteristicScoreDelta`/
  `CharacteristicScoreDeltaParam` "Foo"/"FooParam" precedent.
  `effective/xp.rs::restricted_ability_xp_pools` reads the selection's own
  `years` value, multiplies by `per_unit`, and resolves each `AbilityRef` into
  an `AbilityInstanceRef` (`instances`, not `abilities` — an unscoped entry's
  `parameter: None` already means "any instance", so Artes Liberales at any
  instance and Dead Language at the Latin instance coexist in the same list
  with no special-casing). No pool at all until a legal `years` value is
  filled in — the same "a choice not yet made" reading `missing_param` gives
  elsewhere. `ability_authorizations()` also folds this effect's `abilities`/
  `categories` into the authorized set, exactly like `RestrictedAbilityXp` —
  an earmark is itself permission.
- Load-time integrity (`ruleset/integrity.rs::validate_parameter_defs`):
  `ParamType::Number` and `ParameterDomain::Number` must pair (either half
  without the other fails the load), and `min > max` fails the load — the
  numeric-range mirror of `max_per_value: 0`'s "unfillable" rejection.
  `::validate_effect_refs` requires a `scaled_restricted_ability_xp`'s `param`
  to resolve to a **`Number`**-domain parameter on the same item (not `Ref`) —
  a scaled-XP `param` must resolve to a count, never an id or a set — and
  every named ability to resolve, on `AbilityAuthorization`'s own
  `validate_gated_ability_refs` precedent. `::validate_param_gate` rejects a
  `ParamGate` naming a `Number`-domain parameter outright: gating on numeric
  equality is a different, unaddressed feature no current ruling needs.
- Frontend: `ParamType`/`ParameterDomain` TS mirrors
  (`ui/src/lib/types.ts`) gain `{ number: { min, max } }`/`'number'`, guarded
  against the Rust source by `ui/src/lib/param-type-parity.test.ts`.
  `ParameterPicker.svelte` gains a `number` domain branch — a bounded
  `<input type="number" min max>`, the first numeric parameter control (every
  other domain renders a `<select>` or free-text `<input>`). Label
  `param-label-years` in `locales/en|de/main.ftl`, alongside the
  `param-domain-number` entry `unknown_param_value` needs.

#### D48 — restricted pools that fund an Ability *instance* (Phase 2 C4)

> Marshal: "A marshal receives 50 extra experience points at character
> generation to spend on the abilities Animal Handling, Etiquette, Hunt,
> Latin, Profession: Marshal and Ride" (`ArMDE:4455`). Master Bard: "an extra
> 240 experience points to spend on Art of Memory, Profession: Storyteller,
> Profession: Poet, any Area Lore, any Organization Lore, Faerie Lore, or
> Magic Lore" (`ArMDE:4461`).

- Source: see the per-entry list below; each line cites its own passage.
- **Ruling (`docs/vf-audit/decisions.md` D48):** `Effect::RestrictedAbilityXp`
  gains `instances: Vec<AbilityRef>` (D14's literal/bound-instance form, reused
  rather than a second mechanism), and pool eligibility becomes the **union**
  of `abilities` (id, any instance), `categories`, and `instances` (id **and**
  a specific instance) — replacing the previous "when `instances` is non-empty
  it is the ONLY test", which cannot express Master Bard's one pool funding
  five whole Abilities **and** two specific Profession instances at once.
  `effective/xp.rs::pool_covers`'s `Ability` arm is now
  `instances.any(matches) || abilities.contains(ability) || categories.contains(category)`
  (minus `exclude`, checked first, unchanged).
- **One resolution path, not two.** `effective/xp.rs::resolve_ability_refs`
  (new) filters a `&[AbilityRef]` by `active_for(selection)` and resolves each
  to an `AbilityInstanceRef` — the exact fold `ability_authorizations`'s
  `AbilityAuthorization`/`ScaledRestrictedAbilityXp`/`AbilityBonusGated` arms
  and `RestrictedAbilityXp`'s own new `instances` fold, plus
  `restricted_ability_xp_pools`'s `RestrictedAbilityXp`/`ScaledRestrictedAbilityXp`
  arms, all now call — so "what may I own" (`ability_authorizations`) and
  "what may this pool fund" (`restricted_ability_xp_pools`) cannot drift apart
  on gate/instance semantics. Behaviour-preserving: the full suite is green
  before and after this refactor (no data changed by it alone).
- **An earmarked instance is itself permission**, same reasoning as an
  earmarked id/category: `ability_authorizations`'s `RestrictedAbilityXp` arm
  now also folds `instances` into the authorized set, each entry authorizing
  only the instance it names — Marshal funding Profession: Marshal no longer
  authorizes owning Profession: Sailor for free either.
- Load-time integrity (`ruleset/integrity.rs::validate_effect_refs`): a
  `restricted_ability_xp`'s `instances` gets the same
  `validate_gated_ability_refs` check `AbilityAuthorization.abilities` already
  has (every named ability resolves; a dangling gate/`Bound` param is
  rejected; `instance` and `gate` may not name the same param key).
- **The 12-pool sweep (measurements.md § 8 row 12), each passage re-read
  directly and confirmed — none rejected:**
  - `virtue.marshal` (`ArMDE:4449-4456`) — "Latin, Profession: Marshal" scoped;
    Animal Handling/Etiquette/Hunt/Ride stay unscoped.
  - `virtue.master_of_kennels` (`ArMDE:4467-4470`) — same shape, "Profession:
    Master of Kennels".
  - `virtue.master_bard` (`ArMDE:4457-4462`) — "Profession: Storyteller,
    Profession: Poet" scoped; Area Lore/Art of Memory/Faerie Lore/Magic
    Lore/Organization Lore stay unscoped. **Regression tests**
    (`tests/d48_instance_pools.rs`): `amount` stays **240** (Faerie Lore 9 +
    Magic Lore 2 = 225 + 15 = 240 XP, fully funded with no general pool at
    all; +1 more XP of demand overflows), and Faerie Lore/Magic Lore stay
    eligible.
  - `virtue.senior_bard` (`ArMDE:4904-4909`) — same six-Ability/two-Profession
    shape as Master Bard, amount 90 (unchanged).
  - `virtue.falconer` (`ArMDE:3847-3852`) — "Latin, Profession: Falconer"
    scoped; Animal Handling/Area Lore/Etiquette/Hunt/Ride stay unscoped.
  - `virtue.craft_guild_training` (`ArMDE:3613-3616`) — "any Craft or
    Profession Abilities, Bargain, or Organization Lore: **Guild**" — only
    Organization Lore is instance-scoped; Craft and Profession stay unscoped
    ("any"), matching the passage's own wording. "Guild" itself is not a
    catalogue value (CV3, `docs/vf-audit/design-cv-catalogued-values.md` § 1.1):
    the character's own guild is local and specific, unlike the ten universal
    values below, so the item declares its own `guild` text parameter
    (ArMDE:3615) and its pool's Organization Lore instance is `Bound` to it —
    the same mechanism `virtue.forge_companion`'s `craft` parameter already
    uses (see that entry below), applied here to a pool-funding entry instead
    of an XP-space Art boost.
  - `virtue.educated` (`ArMDE:3711-3713`) — "Latin and Artes Liberales" — Latin
    scoped, Artes Liberales unscoped (not itself parameterized). Reuses the
    literal-instance form `flaw.covenant_upbringing` (C1) already established,
    rather than inventing a second one.
  - `virtue.baccalaureus` (`ArMDE:3470-3475`) — same "Latin and Artes
    Liberales" shape, amount 90.
  - `virtue.hermetic_experience` (`ArMDE:4063-4066`) — "Order of Hermes Lore,
    Magic Lore, or Latin" — Order of Hermes Lore (`order_of_hermes`) and Latin
    scoped; Magic Lore (not parameterized) stays unscoped.
  - `virtue.clan_ilfetu` (`ArMDE:3563-3566`) — "House Bjornaer Lore, Magic Lore
    … and Gothic" — House Bjornaer Lore (`house_bjornaer`) and Gothic scoped;
    Magic Lore stays unscoped. The specialty "(with a specialty in the Great
    Beasts)" is not modelled — specialties are free text on the bought score,
    orthogonal to instance scoping.
  - `virtue.rosh_beth_din` (`ArMDE:4878-4883`) — "the required Abilities"
    resolves against the Virtue's own prerequisite: "scores of at least 5 in
    **Hebrew**, Rabbinic Law, and Theology: Judaism" — Hebrew scoped; Rabbinic
    Law and Theology: Judaism are fixed, non-parameterized ids and stay
    unscoped.
  - `virtue.forge_companion` (`ArMDE:3925-3928`, F-74) — "raise the particular
    Crafts her master practices." **No existing mechanism records "her
    master"** — the engine has no cross-character relationship at all, so
    which Craft(s) a Verditius's unGifted craftsman may fund cannot be *read
    off* anything; it is a fact only the player can supply. **Modelled as a
    new `text`-domain parameter** (`craft`), read via `AbilityRef.instance:
    { param: "craft" }` — the same `Bound`-instance form C0 designed for F-63
    (`AbilityScoreGrantParam`, C5c), applied here to a pool-funding entry
    instead of a floor grant. **This narrows the passage's plural "Crafts" to
    one recorded choice**, same as any other Virtue capped at one selection
    (Forge Companion is Social Status, D41): a master who practices more than
    one Craft cannot have all of them funded by a single `craft` value. That
    is a real, stated limitation, not a silent one — recorded here rather than
    left for someone to rediscover — and it is still strictly better than
    today's unscoped `ability.craft` (funds literally any Craft in the
    catalogue, matching no master at all).
  - **Not instance-scoped, confirmed by the same re-read**:
    `flaw.feral_upbringing` ("(Area) Lore", i.e. any), `virtue.schooled_in_crime`,
    `virtue.shadchan`, `virtue.venditor` — each names its Abilities plainly,
    with no "the" / specific-instance wording.
- **D10, no migration.** Instance scoping narrows ownership, so a save that
  spent Marshal's/Master of Kennels'/etc. points on the wrong Profession (or
  Educated's on a non-Latin dead language) becomes invalid — reported (a new
  `not_enough_xp` shortfall, or `ability_category_requires_virtue` if that was
  the character's only academic authorization) and blocked, never silently
  migrated. Swept `examples/` and every book-template fixture for a holder:
  only `crates/arm-rules/tests/fixtures/book_templates/companion_witch.json`
  holds one of the twelve (`virtue.educated`) with a Dead Language score — its
  `parameter` was `"Latin"` (capitalised), which no longer matches the literal
  `"latin"` this fix and `flaw.covenant_upbringing`/`virtue.custos` (C1) all
  use; fixed to lowercase in the fixture (the language spoken is unchanged,
  only its stored slug), restoring `the_witch_matches_the_book`'s previously
  green result. No other fixture or `examples/` save holds any of the twelve.
- **CV3, literal instances become catalogue ids**
  (`docs/vf-audit/design-cv-catalogued-values.md`). The ten rulebook words the
  pool sweep above names as literal instances (`latin`, `gothic`, `hebrew`,
  `falconer`, `marshal`, `storyteller`, `poet`, `master_of_kennels`,
  `house_bjornaer`, `order_of_hermes`) are now the catalogue ids CV1/CV2 shipped
  (`language.latin`, …, `organization.order_of_hermes`) — comparing a
  case/language-independent id rather than an exact-cased rulebook word (D14's
  original defect). Load-time integrity
  (`ruleset/integrity.rs::validate_literal_instance`) fails loudly if a
  `Literal` instance on a `catalogued: true` Ability names an id outside that
  Ability's own catalogue. **`companion_witch.json`'s Dead Language score holds
  the interim value `"language.latin"`, not the human-typed `"Latin"`** — under
  `AbilityScore.parameter`'s still-plain `Option<String>` (pre-CV4), a `Literal`
  only matches by exact string equality, so the id is the only value that
  keeps `the_witch_matches_the_book` green. **CV4 must restore `"Latin"`**
  once `AbilityParameterValue`/name-matching and the migration fold exist (§
  5.3 of the design note) — CV4's fold matches *names*, not ids, so the
  human-typed word is the value CV4's own fixture set expects (see its
  own § 5.7 test-obligation list, which already spells out this exact
  fixture and value).
- **CV3, Bound/Link once-only.** Load-time integrity
  (`ruleset/integrity.rs::validate_param_value_ref`) now also rejects a
  `Bound`-instance-declaring item whose `max_total` allows more than one copy:
  `(item_ref, param)` is the only handle a Bound source (or, later, a Link
  target, CV5) can name, so two copies would be ambiguous by construction, not
  merely at runtime (design note § 7). A no-op for every item shipped today —
  `virtue.forge_companion` and `virtue.craft_guild_training` both already
  default to `max_total: 1` (D10).
- **CV4, `AbilityScore.parameter` widens to `AbilityParameterValue`**
  (`types.rs::AbilityParameterValue`) — `Catalogued { id }` / `Linked { item,
  param }` (type only; nothing produces or resolves one until CV5) / `Text
  { text }`, `#[serde(untagged, deny_unknown_fields)]` so a value naming keys
  from more than one variant fails the whole load rather than silently
  matching the first structural fit. `migration.rs::wrap_legacy_ability_parameters`
  rewraps a pre-CV4 bare-string `parameter` into `{"text": …}` before the typed
  parse, unconditionally (not gated on the claimed `schema_version`);
  `migration.rs::fold_catalogue_matching` then upgrades a `Text` value into
  `Catalogued { id }` where it case-insensitively, trimmed-ly spells out a
  catalogue entry's name in either locale (§ 5.3), reporting both the
  resolved and the unresolved cases on `LoadedEntity`. `SCHEMA_VERSION` 17 →
  18. **`companion_witch.json`'s Dead Language score is restored to the
  human-typed `"Latin"`**, exactly as the CV3 note above anticipated — the
  fold now resolves it to `language.latin` at load, so
  `the_witch_matches_the_book` stays green against the player-typed word, not
  the interim id.
- **CV4, Literal-only matching (design § 4 rule 1).** A `ParamValue::Literal`
  instance is satisfied ONLY by a bought `AbilityParameterValue::Catalogued`
  with the matching id — never by `Text` holding the identical letters, which
  is exactly D14's original defect restated one layer up (a stray case/
  language mismatch used to silently fail; now a `Text` value can never pass
  at all, catalogued or not). Implemented in
  `effective/xp.rs::AbilityInstanceRef::satisfied_by` and
  `effective/xp.rs::AuthorizedAbility::covers` (shared via
  `effective/xp.rs::instance_satisfied`), which both distinguish a
  `Literal`-derived restriction (`requires_catalogued: true`) from a `Bound`-
  or plain-text-derived one — Bound's own structural/content matching (design
  § 4 rule 2) is CV5 work and keeps the pre-CV4 plain-string comparison until
  then. `effective/ability.rs`'s `AbilityBonusGated` target comparison is
  **not** yet rewritten to this rule — no shipped `ability_bonus_gated` effect
  declares an `instance` restriction, so the gap is latent, not live, exactly
  as design § 4.2 records for the Bound/Link ambiguity guard. **Still true
  after CV5** (see CV5's own entry below): CV5 rewired every shipped Bound/Link
  site (`RestrictedAbilityXp`/`ScaledRestrictedAbilityXp`/`AbilityAuthorization`/
  `AbilityBonusGated`'s OWN `resolve_ability_refs`-derived `instances`, in
  `ability_authorizations`/`restricted_ability_xp_pools`), but NOT this one
  bonus-comparison branch, which reads a plain `parameter: Option<&str>` match
  key rather than the typed `AbilityParameterValue` and would need its own,
  wider threading through every `ability_bonus` caller — deferred until a book
  ships a gated `AbilityBonusGated` target with an `instance` restriction to
  actually exercise it.
- **CV5, Bound/Link matching (design § 4), the ambiguity guard (§ 4.1), the
  dangling/ambiguous-link fold (§ 5.4), and the unlink operation (§ 5.5).**
  `effective.rs::resolve_link` resolves a `ParamValue::Bound` source or an
  `AbilityParameterValue::Linked` target's `(item, param)` against effective
  (bought ∪ granted, D2) selections: zero occurrences → `Dangling`; exactly one
  → `Resolved`; more than one → `Ambiguous` (carrying the bought copy's value
  only when exactly one occurrence is bought — "bought beats granted", §
  4.1). `effective/xp.rs::resolve_ability_refs` calls it for every
  `ParamValue::Bound` instance instead of reading `selection.params` directly,
  so `AbilityInstanceRef`/`AuthorizedAbility` gain `bound_source: Option<(Id,
  String)>` and `ambiguous: bool`. Matching
  (`AbilityInstanceRef::satisfied_by`/`AuthorizedAbility::covers`, shared via
  the new `effective/xp.rs::bound_instance_satisfied`) checks `ambiguous`
  first (satisfies nothing, full stop), then applies rule 1 (a bought `Linked`
  naming the SAME `(item, param)` — no string comparison) or rule 2 (a bought
  `Text`/`Catalogued` whose content equals the Bound source's current value,
  case-folded and trimmed via the now-`pub(crate)` `catalogue::fold_name` —
  the SAME fold the load-time catalogue-matching fold uses, so the two can
  never disagree on "does this text spell out this value"). A bought
  `Catalogued` compares its raw id (not a localized name) against the Bound
  source's text — full both-locale name resolution at this layer would need
  `catalogue_names` threaded into `effective`/`validation`, which no shipped
  Bound-declaring item's data exercises (guild/craft are always free text),
  so it is deferred rather than built for a case nothing reaches.
  `validation/scores.rs::validate_ability_parameter_link` emits
  `issue-ambiguous_bound_parameter` for a bought `Linked` value that resolves
  ambiguous — telling the player WHY a link stopped funding, since the
  matching layer already silently treats it as satisfying nothing.
  `migration.rs::fold_dangling_and_ambiguous_links` runs unconditionally on
  every load (value-driven, no version signal, exactly like
  `trim_all_selection_params`): a `Linked` value that resolves `Dangling` or
  `Resolved(None)` folds to empty `Text`; one that resolves `Ambiguous` folds
  to `Text` holding the provenance fallback; both are reported on
  `LoadedEntity::dangling_links`/`ambiguous_links` (`(ability, item, param)`
  triples). `effective.rs::unlink_ability_parameters` is the same conversion
  as an engine-owned operation the UI must call BEFORE splicing out a removed
  Virtue/Flaw selection, so "last resolvable value" is still readable —
  **wiring the Virtue/Flaw removal flow to call it is CV7's job, not CV5's**
  (CV5 ships it engine-only, per the design note's own slice table). Since a
  link can only ever target a once-only (`max_total <= 1`, § 7), never-granted
  (§ 4.2) bought selection, the only two paths that can actually orphan a link
  are the player removing that exact Virtue/Flaw selection, or clearing its
  own parameter (the guild/craft text) back to empty — there is no third way
  to lose a link target. `export/resolve.rs::Doc::ability_param_value`'s
  `Linked` arm now calls `resolve_link` too (§ 6.4's "one engine function"),
  showing the ambiguity fallback rather than a blank cell — never a raw id or
  `(item, param)` pair. Tests: `crates/arm-rules/tests/cv5_bound_link_matching.rs`,
  `cv5_link_fold_and_unlink.rs`; `data_integrity.rs`'s
  `no_bound_or_link_declaring_item_is_ever_granted` (design § 4.2's mitigation,
  passes vacuously today, mirroring the C0 §3 precedent). No `SCHEMA_VERSION`
  bump: `Linked` already shipped inside CV4's 17 → 18 (design § 3.2), and
  adding resolution logic on top is purely additive for existing v18 data.
- **CV6, `AbilityParameterOptions` (design § 6.3, § 11 item 2).** Engine-built
  parameter-picker options, one entry per Ability that is either catalogued or
  offers at least one link target to the character — `effective/
  parameter_options.rs::ability_parameter_options`. `catalogued` reads
  straight off `Ruleset::parameter_catalogues()` in catalogue order (ids only,
  independent of the character); `linked` reuses CV5's `ability_authorizations`
  output (`AuthorizedAbility::bound_source`/`ambiguous`) rather than
  re-scanning effects, filtering out `ambiguous` entries per design § 4.1 ("an
  ambiguous source is not offered as a link target"), sorted/deduplicated via
  the new `LinkTarget`'s derived `Ord` for a deterministic picker order. The
  conditional hint (§ 11 item 2, replacing an earlier "static, always shown"
  recommendation) is `true` only when a bought `Text` value on that ability
  fails to satisfy at least one SCOPED (`Literal`- or non-ambiguous `Bound`-
  derived) `AuthorizedAbility` entry from one of the character's own items —
  `AuthorizedAbility::covers` (made `pub(crate)`, previously private) is the
  shared test, so the hint can never disagree with what actually funds/
  authorizes. An unscoped entry (no instance restriction at all) contributes
  nothing to the hint, and neither does an ambiguous one (nothing typed could
  satisfy it anyway). `EffectiveScores::ability_parameter_options`
  (`arm-app/src/effective_dto.rs`) carries it over IPC; the TS mirror
  (`AbilityParameterOptions`/`LinkTarget`, `ui/src/lib/types.ts`) is NOT yet
  added to the TS `EffectiveScores` interface — the existing
  `effective-scores-consumers.test.ts` guard requires a real shipped consumer
  for every `EffectiveScores` field, and CV6 ships no UI at all (CV7 wires the
  picker and adds the field + its first consumer together, rather than this
  slice adding an orphaned field to dodge that guard). Tests:
  `crates/arm-rules/tests/cv6_ability_parameter_options.rs`;
  `arm-app/src/effective_dto.rs::tests::effective_scores_surfaces_ability_parameter_options`;
  `ui/src/lib/ability-parameter-options-parity.test.ts` (Rust struct fields vs.
  the TS mirror, text-diffed like `param-type-parity.test.ts`).
- **CV7, the picker (design § 6.1/§ 6.2/§ 6.3/§ 6.4, § 5.5).** Pulls forward
  three pieces the design note's own slice table had scoped to CV8 — a live
  picker cannot ship without them without either rendering a raw/humanized id
  or silently breaking bonus/floor matching the moment it writes a `Linked`
  value, so the note's CV7/CV8 rows were revised to match what actually
  shipped here (§ 10 of the design note records the correction).
  - `LinkTarget` gains `resolved: Option<String>` (`effective/
    parameter_options.rs`) — carries the ALREADY-COMPUTED
    `AuthorizedAbility::instance` value through, not a second resolution, so
    the picker can label a link target ("Follows «Craft Guild Training»:
    Smiths' Guild of Verdi") without its own lookup.
  - `arm-app::ruleset_io::merge_catalogue_display_names` reads the ACTIVE
    language's `i18n/<lang>/parameter_catalogue.json`
    (`arm_rules::parse_catalogue_names`, `catalogue.rs`) and merges each
    catalogue value's own-language name into `LocalizedRuleset.i18n` at load —
    the SAME map every other id's display name already lives in (design §
    2.3), so `ui/src/lib/derive.ts::abilityParamDisplay`'s `Catalogued` arm
    resolves through it exactly like `displayName` does elsewhere, falling
    back to `humanizeCatalogueId` only when a name is genuinely absent.
    Best-effort like `load_catalogue_names_from_dir`'s own `.ok()`: a missing
    or malformed name file does not fail the whole ruleset load.
  - `ui/src/lib/derive.ts::sameParam`/`normalizeParam`/`resolvedLinksFrom`
    (design § 6.2) — the UI's structural, same-entity comparison for a
    `.parameter` value, resolving a `Linked` value to its CURRENT text via
    `resolvedLinks` (every offered link target's `LinkTarget.resolved`,
    flattened once per `derive()` pass) rather than comparing `(item, param)`
    pairs. `AbilityTab.svelte`'s `bonusOf`/`settledScoreOf` both route through
    it now, replacing the CV4-era `abilityParamKey` identity comparison there.
  - The combo box itself: `AbilityTab.svelte`'s parameter `<select>`, built
    entirely from `store.effective.ability_parameter_options` (catalogue
    values, then link targets, then "Other…" — one engine-decided order).
    Choosing a catalogue entry writes `{id}`; choosing a link target writes
    `{item, param}`; choosing "Other…" reveals the existing free-text escape.
    Choosing any entry REPLACES the stored value outright — there is no
    "keep both" state. A `Linked` value's own row shows a "follows the
    Virtue" indicator (`ability-param-follows`, both locales) naming the
    source and its current text, or a visibly distinct
    `ability-param-unresolved` indicator when the source cannot currently be
    resolved (removed, or ambiguous — `issue-ambiguous_bound_parameter`
    explains why). The conditional hint (`options.hint`) renders as
    `ability-param-hint` when set.
  - The removal/clear flow: `arm_rules::unlink_ability_parameters` (CV5) now
    has an IPC command, `commands::unlink_ability_parameters`
    (`arm-app/src/commands.rs`, guarded like every other ruleset-needing
    command, delegating to `ruleset_io::unlink_ability_parameters_loaded`),
    registered in `main.rs`. `ui/src/lib/state.svelte.ts`'s
    `AppStore.removeSelectionAt`/`setParamAt` call it BEFORE applying the
    change whenever the affected item is currently offered as a link source
    (`#isLinkSource`, a latency optimization over the engine's own correct
    no-op, not a second copy of its decision) — `removeSelectionAt`
    unconditionally, `setParamAt` only when clearing an existing value to
    empty (a non-empty edit is a rename a live link must keep tracking, D59
    point 2). Both methods are now `async`; the synchronous fast path (the
    common case, a non-linking item) still resolves within the same tick, so
    `state.svelte.test.ts`'s existing synchronous assertions needed no
    changes.
  - Tests: `crates/arm-rules/tests/cv7_link_target_resolved.rs`;
    `crates/arm-app/tests/commands.rs`'s
    `load_ruleset_localizes_catalogue_value_names_in_{english,german}` and
    `unlink_ability_parameters_converts_a_linked_ability_score_to_text`;
    `ui/src/lib/derive.test.ts`'s `sameParam`/`normalizeParam`/
    `resolvedLinksFrom` suites; `ui/src/lib/state.svelte.test.ts`'s
    `setAbilityParameterValueAt` and the two unlink-flow `describe` blocks;
    `AbilityTab.test.ts`/`AbilityTab.client.test.ts`'s "parameter picker
    (CV7)" suites (localized names never a raw id as visible text, the
    indicators, the hint, keyboard-operable native `<select>`, the live
    removal conversion).
- Tests: `simple_student_scales_its_restricted_pool_by_finished_years`,
  `simple_student_funds_latin_but_not_another_dead_language`
  (`tests/data_integrity.rs`); `a_number_type_paired_with_a_non_number_domain_fails_the_load`,
  `a_number_domain_paired_with_a_non_number_type_fails_the_load`,
  `a_number_parameter_with_min_greater_than_max_fails_the_load`,
  `ordinary_number_parameter_loads`,
  `a_scaled_restricted_ability_xp_param_not_of_type_number_fails_the_load`,
  `a_scaled_restricted_ability_xp_param_naming_an_undeclared_key_fails_the_load`,
  `ordinary_scaled_restricted_ability_xp_effect_loads`,
  `a_gate_naming_a_number_domain_parameter_fails_the_load` (`ruleset.rs`);
  `ParameterPicker.test.ts` (ssr rendering), `ParameterPicker.client.test.ts`
  (bounded clamping), `param-type-parity.test.ts`.

#### Corrupted Abilities/Arts/Spells & Enchanting (Ability) (D9 part 3, D15, F-63, Phase 2 C5c)

> Corrupted Abilities: "Any use of a corrupted Ability is an unholy act, which
> can be sensed by Divine Powers. You may only take this Flaw once, though you
> can choose to have it affect multiple Abilities if you wish" (`ArMDE:5847-5852`).
> Corrupted Arts: "Any use of a corrupted Art taints the character's magic,
> causing it to appear unholy. You may only take this Flaw once, though it can
> affect multiple Arts" (`ArMDE:5853-5858`). Corrupted Spells: "Any use of a
> corrupted spell is tainted and appears unholy. You may only take this Flaw
> once, though it can affect as many of the character's spells as you wish"
> (`ArMDE:5859-5864`). Enchanting (Ability): "Choosing this Virtue confers the
> Ability Enchanting (Ability) 1" (`ArMDE:3747-3750`).

- Source: `ArMDE:5847-5852` (Corrupted Abilities), `ArMDE:5853-5858` (Corrupted
  Arts), `ArMDE:5859-5864` (Corrupted Spells), `ArMDE:3747-3750` (Enchanting
  (Ability)).
- **Ruling (`docs/vf-audit/decisions.md` D15):** the three Corrupted entries
  are one mechanic (a book-repeated ±3 roll bonus/±5 XP swing on a GM's
  selfish-or-sinful judgement call), so they get one treatment —
  `uncomputed_rule`, no effects at all, with the full passage in `description`
  in both locales. `flaw.corrupted_arts` moves off `creation_effect` (it never
  actually carried an effect); `flaw.corrupted_spells` moves off
  `in_play_effect` and **loses its `special_casting_mod { circumstantial }`**
  — the only thing any of the three computed — since the modifier's condition
  ("selfish or sinful" vs. "neutral or selfless") is a table judgement no
  engine decides, and unifying upward would need an effect naming an Ability
  roll and a Casting Total together, which the effect vocabulary does not
  support (and D15 does not ask for). `flaw.corrupted_abilities` was already
  `uncomputed_rule` and is the model the other two move to.
- **D9 part 3 — the `multi_ref` parameter (§ 8, `design-c0-parameter-model.md`):**
  each entry declares one `targets` parameter, `type: "multi_ref"`, over the
  domain its own passage names — `ability` (Corrupted Abilities), `art`
  (Corrupted Arts), `spell` (Corrupted Spells, resolved against the character's
  OWN `entity.spells`, per `ParameterDomain::Spell`'s possession-scoped
  reading — there is no "any spell in the rules" reading the 30-level
  prerequisite would even permit). The classification says the engine computes
  nothing; the parameter still records which targets the player chose, so the
  choice round-trips through save/load and prints on the Markdown export —
  `uncomputed_rule` and a multi-valued parameter are both correct here, for
  different reasons (D15's own note on the interaction).
- **F-63 — `Effect::AbilityScoreGrantParam` (§ 2, `design-c0-parameter-model.md`):**
  `virtue.enchanting_ability` moves from a bare `ability_score_grant` (which
  ignored its own declared parameter entirely — the shipped `"ability"`/
  `domain: ability` parameter let the player choose ANY Supernatural Ability,
  never read by the effect, and the floor applied unconditionally) to
  `ability_score_grant_param { ability: "ability.enchanting", instance: {
  "param": "medium" }, amount: 1 }`, paired with a new `medium` parameter,
  `domain: text` (the passage's "even craftwork" hedges its list as
  illustrative, not exhaustive — D9's "text only where the choice is genuinely
  open"). `ability.enchanting` itself (`rules/core/abilities.json`) gains
  `"parameter": "medium"`, so the bought Ability row and the Virtue's grant
  share the same instance axis.
  - **Wired through the existing D59/CV Bound/Link model, not a second path**
    (orchestrator ruling, Phase 2 go-ahead): `effective/xp.rs::resolve_instance`
    is factored out of `resolve_ability_refs` (identical Bound/Literal/plain
    resolution, now shared by both the `Vec<AbilityRef>` fold and this single-
    instance grant) and reused by three call sites — `ability_authorizations`'s
    new `AbilityScoreGrantParam` arm (below), and
    `effective/ability.rs::granted_ability_floor`/`ability_score_floors` (whose
    `if parameter.is_some() { return 0; }` early return previously made a
    parameter-bound floor grant impossible at all). `AbilityFloor` gains a
    `parameter: Option<String>` field (mirrors `AbilityBonus::parameter`) so
    the UI badge lands on the chosen medium's row, never on every instance —
    `AbilityTab.svelte::floorOf` and `derive.ts::unboughtModifiedAbilities`
    updated to match on `(ability, parameter)` via `sameParam`, exactly like
    the existing bonus path.
  - **Correction to this note's own design doc, dated 2026-09-27 (Phase 2
    go-ahead ruling):** `design-c0-parameter-model.md` § 1a row 12
    (`effective/xp.rs::ability_authorizations`) originally read
    `AbilityScoreGrantParam` as a no-op there, "a floor grant is not an
    authorization path" — but `AbilityScoreGrant`, this variant's own
    unparameterized sibling, is NOT a no-op at that same site (it inserts an
    `AuthorizedAbility`, "a free score in an Ability is permission to have
    it"). The row was stale, not the code: `AbilityScoreGrantParam` gets a
    **real** arm too, scoped to the resolved instance via `resolve_instance`.
    This is also what lets `ability_parameter_options` (CV) offer the Virtue's
    own `medium` parameter as a LINK target for the Ability's picker
    (`AuthorizedAbility::bound_source`) instead of making the player retype
    the medium as free text.
- Tests: `crates/arm-rules/tests/c5c_corrupted_and_enchanting.rs` (13 cases:
  the three entries' shipped `multi_ref` parameters and domains, the D15
  reclassification and effect removal, both locales' descriptions, a
  duplicate-target collision, Corrupted Spells' possession-scoped resolution,
  the F-63 floor applying only at the chosen medium and nowhere else, and the
  authorization fix via a `Linked` bought instance);
  `effective/xp.rs::ability_authorizations_reads_only_the_three_permission_granting_effects`
  (unchanged — still exercises only the pre-existing three; the new fourth
  case is `c5c_corrupted_and_enchanting.rs`'s own);
  `ui/src/lib/components/AbilityTab.test.ts`'s "parameter-scoped floors
  (F-63)"; `ui/src/lib/derive.test.ts`'s `unboughtModifiedAbilities` floor
  case.

#### The scholarly-language expectation for Academic Abilities (M6/6b2b)

> Academic Abilities require formal training. … In addition, learning an Academic
> Knowledge normally requires a Latin, Greek, Hebrew, or Arabic score of at least 3,
> depending on the region of Europe you are from. For most characters, Latin 3 is
> required.

> In other areas of the world, Arabic, Greek and Hebrew fill similar functions,
> although of these only Hebrew is a dead language.

- Source: `ArMDE:7151`, `ArMDE:7432` (L2, ruling F5, D83.12).
- Data: `rules/core/abilities.json` → `scholarly_language`:
  `{ min_score: 3, languages: [ { ability: ability.dead_language, values: [language.latin,
  language.hebrew] }, { ability: ability.living_language, values: [language.greek,
  language.arabic] } ] }`. Greek and Arabic are Living Language values because
  `ArMDE:7432` says only Hebrew of the three is dead. Values are in book order within
  each Ability, which is the order the warning lists them.
- Implementation: `validation/authorization.rs` — `validate_academic_language`: any
  bought instance of a listed Ability naming a listed value (through
  `catalogue.rs::instance_is`, so typed names count) at bought score ≥ `min_score`
  satisfies it. The listed Abilities do not themselves trigger it. Emits
  `academic_ability_without_scholarly_language` (args `languages`, the value ids
  ", "-joined in data order, and `min`); the UI names them "Latin, Hebrew, Greek or
  Arabic" (`derive.ts`, `requirement-language-list-*`).
- Load checks: `ruleset/parse.rs::check_scholarly_language` (each Ability resolves and
  is parameterized; no empty list) and
  `ruleset/integrity.rs::validate_scholarly_language_values` (each Ability is
  catalogued, each value is in its own catalogue, else the load fails naming it).
- A **warning**, not an error, because the passage hedges twice ("normally",
  "depending on the region of Europe"); the engine cannot know a saga's region, so any
  of the four satisfies it.

#### Foreign Upbringing halves locality-dependent caps (M6/6b2c)

> The maximum scores at character creation for locality-dependent Abilities like
> Language, Area Lore, or Organization Lore, as well as some social Abilities, are
> half (round up) that which his age normally allows.

- Source: `ArMDE:6160`.
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
standing at its Gauntlet (`life_stage_age_before_gauntlet`, `ArMDE:2435`) — an unchosen
native language, and a chosen one with no bought score (a warning — the points are
merely unspent). Only the two age bars are sourced.

#### The guided wizard defaults to life-stage funding, direct entry stays on the pool (M6/D2)

Not a rule change — `LifeStageRules::budget` and every validator above are untouched
— but the product decision that finally *wires up* the age-derived budget this whole
section computes, which sat behind `AbilityFunding::Pool` (every character's default)
until a new wizard character explicitly switched to it. The owner's report: guided
creation should hand the player a character funded by its life history, not a pool
sitting at 0.

**Scoped to the wizard, not `Entity::new`.** `ui/src/lib/state.svelte.ts`'s
`#instantiateCharacter(typeId, abilityFunding)` takes the mode as a parameter,
defaulting to `'pool'` — {@link createCharacter} (direct entry) takes that default
as-is, `startWizard` passes `'life_stages'`. The engine's own `Entity::new` and its
`#[default]` on `AbilityFunding` are unchanged: direct entry is the "I already know
the numbers" mode, where a hand-typed total is the point, and nothing about a
directly-entered character's defaults moves.

**Allocates an EMPTY plan together with the mode, exactly like a manual switch.**
`#instantiateCharacter` sets `life_stages = {}` whenever it defaults to
`'life_stages'` — the same pairing `setAbilityFunding` already does for a manual
switch (`ui/src/lib/state.svelte.ts`). This is not cosmetic: every engine read of
the plan (`LifeStageRules::budget`, `validate_life_stage_plan`) is gated on the
plan's mere *presence*, by design predating this slice — so a magus's age, typed on
`concept` (see below) before the player ever opens `experience`, would raise no
`life_stage_age_before_gauntlet`/`life_stage_age_unset` finding at all without a
plan already sitting there to read. An earlier version of this slice tried leaving
the plan uncreated and having the individual setters (`setNativeLanguage`,
`#setPlanCount`) conjure it lazily on first write instead; the flat-out gap that
design left — a magus whose player types an age but never happens to touch a
plan-specific field first sails past every Gauntlet-age check with zero feedback —
surfaced immediately in the `e2e/specs/magus-apprenticeship.e2e.js` run, which is
exactly the failure this paragraph is now the record of.

**A merely-allocated plan is still not a choice, though.** `completeness.rs`'s
`Experience` criterion changed from `life_stages.is_some()` to
`life_stages.as_ref().is_some_and(|plan| *plan != LifeStagePlan::default())` — it
now reads the plan's *content*, not its bare presence — precisely mirroring how the
`VirtuesFlaws` arm above it already filters a profile's own forced selections
(`is_mandatory_trait`) out of "engaged": a default the player did not choose is not
engagement, whichever field carries it. Without this refinement, EVERY fresh wizard
character would read the Experience step as finished the instant it is created,
before the player has named it, aged it or touched anything at all — reopening
exactly the "0 is always a deliberate entry" trap the flat-pool side of this same
disjunction was built to avoid. `completeness.rs`'s
`a_life_stage_funded_character_with_an_empty_plan_is_still_incomplete` pins the
three-way shape: no plan is incomplete, an allocated-but-empty plan is *still*
incomplete, and a plan carrying one real field (a Gauntlet age, say) finishes the
step. The pre-existing `a_stored_life_stage_plan_finishes_the_experience_step` was
updated in the same change to store a plan with a native language rather than a bare
`LifeStagePlan::default()`, since a default plan no longer finishes anything.

**The step-order question this raises, and why it was already answered.** A
life-stage budget derived from a null age is empty, so this default is only sound if
the age is captured before the `experience` phase in every character type's
`creation_phases` (`rules/core/character_types.json`). It already is: Slice 12
(#24, guided-creation-review-2026-08) moved the age field onto the FIRST phase every
profile declares — `concept` — via `AgeFields` mounted in `ConceptStep.svelte`,
specifically because the age used to live on the `aging` step, three phases past
where life-stage funding needed it. This slice rides that fix rather than repeating
it, and the eager plan allocation above is what makes riding it sufficient: the age
typed on `concept` has a plan waiting for it by the time `budget()`/
`validate_life_stage_plan` next run, on the very same character, whichever step is
current.

**What did NOT change: `LifeStageBudget::total()` stays unwired.** The owner also
asked, and rejected, prefilling `Entity::xp_pool` from the age — see this section's
own doc comment on `total()` above: the blocks fund different, non-fungible things
(childhood's 75 buys only the native language, its 45 only the closed spread list, a
magus's later life is Abilities-only), so summing them into one scalar pool would
misrepresent what the character may spend it on. Defaulting the *mode*, not summing
the *budget*, is what this slice does instead.

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

The unspent-block warning and the 75-point pool read `ArMDE:2378` the same way, which is
a requirement rather than a coincidence: both key on
`childhood.native_language_ability` at the chosen instance, scoring above 0
(`validate_life_stage_plan` and `native_language_instance` in `effective/xp.rs`). A
looser test — any parameterized Ability whose parameter equals the language — would
let an `Area Lore (German)` declare the block spent while the pool, which funds one
instance of one id, paid for nothing of it.

## Aging (M6/6b6) — `aging.rs`

The yearly roll of `## Aging` (`ArMDE:16563-16617`), from the threshold that owes it to
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

**The Crisis is resolved, never survived.** The engine totals it (`ArMDE:16621`), reads
the row (`ArMDE:16624-16632`), records it on the year (`resolve_year`'s Crisis leg) and
reports what surviving would take (`ArMDE:16628-16638`). It never throws the Stamina die,
never resolves the doctor's Medicine roll (`ArMDE:16634`) and never kills — `ArMDE:16617`'s
death at Decrepitude 5 ships as a datum nothing acts on. See *Recorded gaps* at the
end of this section.

#### Age at which aging rolls become due — over 35
> "Characters begin aging in the Winter after they turn 35. Every year, a
> character must roll on the aging table."

> "The first thing to bear in mind is that a character over the age of 35 must
> make aging rolls (see page 392) before the game begins."

- Source: `ArMDE:16565` (the threshold),
  `ArMDE:2232` (the rolls are owed before play begins).
- **Data**: `rules/core/aging.json` → `start_age: 35`. It was a cited Rust
  constant (`AGING_ROLLS_START_AGE`) until this slice created the file; the
  constant is gone, and a ruleset shipping no aging block stands the subsystem
  down rather than letting the engine supply a fallback number.
- Implementation: `AgingRules::first_roll_age()` (`aging.rs`) does the one
  deliberate `+1` — "the Winter **after** they turn 35" falls in the 36th year, and
  `ArMDE:2232`'s "over the age of 35" agrees — and
  `validation/aging.rs::report_pending_aging_rolls` emits
  `aging_rolls_pending` (warning) for a character who has reached it with
  an empty `aging_log`. `ArMDE:2496`'s "each year from the age of 35" is advice inside a
  worked advancement example, disposed of in `first_roll_age()`'s doc comment rather
  than ignored.
- The schedule itself is `aging::aging_schedule(entity, ruleset) -> Vec<AgingYear
  { age, year, recorded }>` — `first_roll_age()..=age`, pairing each age with its
  calendar year (`birth_year + age`, `None` without a birth year) and with whether
  the log already records it. Empty at or under the threshold, with no age entered,
  and under a ruleset shipping no aging rules.
- **Maximum age 500 — an app limit, NOT a rule** (slice A1, Norbert 2026-10-03,
  after-deadline answer 7; robustness finding F2). No rulebook passage sets a maximum
  age, so this row cites none. The schedule builds one row per year, so an unbounded
  typed or crafted age froze or OOM-crashed the app. Data:
  `rules/core/aging.json` `max_age: 500` → `AgingRules::max_age` (optional; an aging
  block stating none applies no cap), surfaced on the Ruleset the UI receives.
  Enforced by: `aging_schedule` (never walks past it), `age_in_saga_year` (a derived
  age clamps at it), `migration.rs::clamp_ages_to_max_age` (a save's `age` and
  `apparent_age` clamp to it, and a `birth_year` earlier than `saga_year - max_age`
  clamps in step; value-driven and version-free like the aura clamp, so the next save
  writes the clamped values), the integrity check (a `max_age` below
  `first_roll_age()` fails the load), and the UI (age and apparent-age inputs capped
  at it, birth-year input floored at `saga_year - max_age`). Guards:
  `tests/a1_max_age.rs`.
- **A Longevity Ritual holder under 35 is not scheduled** — a decision, not an
  oversight. `ArMDE:16575`'s "should roll on the table no matter what his age" is
  unbounded downward, nothing records *when* the ritual was made, and those rolls
  are clamped so they can never cost a point. The obligation the app enforces is
  `ArMDE:2232`'s, which is age-gated with no ritual clause. `aging_total` still computes
  a pre-35 roll correctly for a caller that asks for one.

#### The AGING TOTAL
> "**AGING TOTAL: Stress die (no botch) + age/10 (round up)**
> **\- Living Conditions modifier**
> **\- Longevity Ritual modifier**"

> "As a high roll generally indicates more serious effects of age, a high Longevity
> Ritual modifier and a high Living Conditions modifier both indicate longer life."

> "The modifier to rolls depends on the character's actual, not apparent, age."

- Source: `ArMDE:16567-16569` (the
  formula), `ArMDE:16571` (the sign convention), `ArMDE:16577` (actual, not apparent, age).
- **Data**: `rules/core/aging.json` → `age_divisor: 10` — the 10 of "age/10 (round
  up)". Rounding **up** steps the term on the first year of each decade, not the
  last: 30 scores 3, 31 already scores 4 (`AgingRules::age_modifier`).
- Implementation: `aging::aging_total(entity, ruleset, age, die) ->
  Option<AgingTotal>`, which returns every term (`age`, `die`, `age_modifier`,
  `living_conditions`, `longevity_bonus`, `trait_modifier`, `uncapped_total`,
  `total`, `capped_by_longevity`) rather than a bare number, so a sheet can show the
  arithmetic without re-deriving it. `age` is a **parameter**, not read off the
  entity: `ArMDE:2232`'s pre-play catch-up walks every owed year and each uses *that*
  year's age.
- **The signs, which are easy to get backwards.** `ArMDE:16571` works only because the
  formula *subtracts* the two named modifiers, so both are stored with the book's
  own printed sign and negated exactly once, in `aging_total`:
  - `Effect::AgingMod { kind: living_conditions }` and the table rows are
    **SUBTRACTED**. Mild Aging's `+1` (`ArMDE:4530`) therefore *lowers* the total, and
    Poor Living Conditions' `-1` (`ArMDE:6620`) *raises* it.
  - The Longevity Ritual bonus is **SUBTRACTED** (`ArMDE:10662`, `ArMDE:10672`).
  - `Effect::AgingMod { kind: aging_roll }` is a **different quantity and is ADDED
    with its stored sign**: Faerie Blood's `-1` (`ArMDE:3801`) lowers the total directly,
    Strong Faerie Blood's `-3` (`ArMDE:5036`) by three. This is not the same convention
    as the two named modifiers, and reading it as one would invert the sign of the
    six shipped items that carry a non-zero `aging_roll` amount (five at `-1`, plus
    Strong Faerie Blood's `-3`); the guard is
    `mild_aging_and_poor_living_conditions_move_the_total_in_opposite_directions`
    plus `faerie_blood_lowers_the_aging_total_by_one` and
    `strong_faerie_blood_lowers_the_aging_total_by_three` in `data_integrity.rs`.
  - `Effect::AgingMod { kind: longevity_bonus }` moves the ritual term, and only for
    a character who actually holds a ritual — a modifier to a bonus that does not
    exist is meaningless.

App/UI: `EffectiveScores` (`arm-app/src/effective_dto.rs`) gains
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
holding Faerie Blood (`ArMDE:3801`) it printed "+4 (age) 0 (living conditions) 0
(Longevity Ritual) = stress die +3" — a rules readout that did not add up. Displayed
by `ui/src/lib/components/AgingSchedulePanel.svelte`, whose sibling
`aging-total-parts` in `AgingRollCalculator.svelte` has always carried the term (off
`AgingTotal::trait_modifier`); both strings now word it identically.
`longevity_clamp_active` is deliberately **not** `AgingTotal::capped_by_longevity`:
that one says a particular roll was cut down, this one that the `ArMDE:16575` clamp
stands over the character at all.

#### The `ArMDE:16575` Longevity Ritual clamp — it clamps the TOTAL, not the die
> "A character under the influence of a Longevity Ritual should roll on the table no
> matter what his age, but treats all rolls of 10 or more as rolls of 9 until he
> reaches the age of 35. His apparent age may be younger than his actual age, but he
> is at no risk of actually aging before any other characters."

- Source: `ArMDE:16575`.
- **Data**: `rules/core/aging.json` → `longevity_clamp: { max_total: 9, until_age:
  35 }` — the 9, not the 10, because it is the value the roll *becomes*.
- Implementation: `aging::aging_total` applies `total =
  uncapped_total.min(max_total)` when the character holds a ritual **and** `age <
  until_age`. A ceiling, never a floor: an uncapped total of 2 stays 2.
  `capped_by_longevity` compares rather than assuming, so it never claims a cap on a
  total the clamp did not touch.
- **The ruling, recorded because a project note had it backwards.** "Rolls of 10 or
  more" is the **total**, not the die:
  1. The formula block those rolls feed is headed "AGING TOTAL" (`ArMDE:16567`) and the
     table's index column "Aging Roll" (`ArMDE:16597`) — the clamp's *rolls* and the
     table's *roll* are the same quantity.
  2. 10 is meaningful only in the table's index space: it is exactly where the first
     aging-point row begins (`ArMDE:16601`). On a stress die 10 is nothing at all.
  3. Decisive: a 34-year-old average peasant with the weakest legal ritual (+1 — "+1
     bonus for every five points **or fraction** of Creo Corpus Lab Total",
     `ArMDE:10662`) would under the die reading score `9 + ⌈34/10⌉ - 1 = 12` and take an
     Aging Point, while a ritual-less peer of the same age rolls nothing at all. The
     die reading makes a Longevity Ritual strictly *worse than nothing* in exactly
     the case the sentence calls safe. Under the total reading both halves of the
     sentence fall out: no points, and a good ritual drops the total below 3 so the
     apparent age does not advance either.
- **The one-year seam is the text's, not a bug.** `start_age` (35, `ArMDE:16565`) and
  `longevity_clamp.until_age` (35, `ArMDE:16575`) are two numbers from two different
  sentences that happen to coincide. "Until he reaches the age of 35" stops the clamp
  *at* 35 while rolls are owed only from 36, so at exactly 35 a ritual-holder rolls
  **unclamped** and a character without one does not roll at all. `validate_aging_rules`
  deliberately does **not** gate `start_age == until_age`: asserting equality would
  invent a relationship the rules never state.
- **The load-time trust gate**: `longevity_clamp.max_total < outcomes[0].min`.
  `ArMDE:16575` states the clamp's *purpose* — "no risk of actually aging" — and that is
  true if and only if the ceiling sits strictly below the first row that costs Aging
  Points. The shipped `9 < 10` is therefore an identity derivable from two
  sentences, not a number to transcribe and hope for (the same idiom as re-pricing
  the apprenticeship's `recommended_xp`). A `max_total: 10` against a table starting
  at 10 fails the load, naming `ArMDE:16575`.

#### The Living Conditions table
> "| Living Conditions | Modifier |"

- Source: `ArMDE:16581-16594` — the header
  at `ArMDE:16581-16582`, the ten rows at `ArMDE:16583-16592`, the footnote at `ArMDE:16594`.
- **Data**: `rules/core/aging.json` → `living_conditions`, ten rows, each carrying
  its own one-line `source`. Ids are sorted canonically in the file, so the file
  order is *not* the book's; the `source` line is what pairs a row with its
  rulebook line:

  | id | modifier | `:line` | cumulative |
  |---|---|---|---|
  | `living_condition.wealthy_or_healthy_location` | +2 | `ArMDE:16583` | |
  | `living_condition.typical_summer_or_autumn_covenant_magus` | +2 | `ArMDE:16584` | |
  | `living_condition.typical_summer_or_autumn_covenant_mundane` | +1 | `ArMDE:16585` | |
  | `living_condition.typical_spring_or_winter_covenant_magus` | +1 | `ArMDE:16586` | |
  | `living_condition.average_peasant` | 0 | `ArMDE:16587` | |
  | `living_condition.live_in_a_leper_colony` | -1 | `ArMDE:16588` | yes |
  | `living_condition.work_in_a_bad_air_trade` | -1 | `ArMDE:16589` | yes |
  | `living_condition.work_in_a_mine` | -1 | `ArMDE:16590` | yes |
  | `living_condition.poor_or_unhealthy_location_typical_town` | -2 | `ArMDE:16591` | yes |
  | `living_condition.leper` | -2 | `ArMDE:16592` | yes |

  Names live in `rules/i18n/{en,de}/aging.json`, keyed by id. The German names come
  from the **rulebook body** at the mirrored `ArMDE:16583-16592`, not from the glossary —
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
  "Average peasant 0" (`ArMDE:16587`), so a character naming no condition already has a
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

- Source: `ArMDE:16594`; the table itself
  at `ArMDE:16581-16592`. The footnote is only worth writing because the *unmarked* rows
  are not cumulative: they name mutually exclusive situations (`ArMDE:16583` "Wealthy, or
  healthy location" against `ArMDE:16587` "Average peasant"; the four covenant rows are
  graded alternatives for one covenant), so at most one of them applies.
- **Data**: `rules/core/aging.json` → `living_conditions[].cumulative`, `true` on
  exactly the five asterisked rows (`ArMDE:16588-16592`).
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

- Source: `ArMDE:16597-16611` — the header
  at `ArMDE:16597-16598`, the two apparent-aging rows at `ArMDE:16599-16600`, the eleven
  effect rows at `ArMDE:16601-16611`; the player's choice at `ArMDE:16615`.
- **Data**: `rules/core/aging.json` → `outcomes`, eleven rows, each with its own
  `source` line and a serde-tagged `effect` so an unknown kind is a **load**
  failure rather than a silently ignored row:

  | totals | effect | `:line` |
  |---|---|---|
  | 10-12 | `any_characteristic`, 1 point | `ArMDE:16601` |
  | 13 | `next_decrepitude_level_and_crisis` | `ArMDE:16602` |
  | 14 | `named_characteristics`, 1 point, `[qik]` | `ArMDE:16603` |
  | 15 | `named_characteristics`, 1 point, `[sta]` | `ArMDE:16604` |
  | 16 | `named_characteristics`, 1 point, `[per]` | `ArMDE:16605` |
  | 17 | `named_characteristics`, 1 point, `[pre]` | `ArMDE:16606` |
  | 18 | `named_characteristics`, 1 point, `[str, sta]` | `ArMDE:16607` |
  | 19 | `named_characteristics`, 1 point, `[dex, qik]` | `ArMDE:16608` |
  | 20 | `named_characteristics`, 1 point, `[com, pre]` | `ArMDE:16609` |
  | 21 | `named_characteristics`, 1 point, `[int, per]` | `ArMDE:16610` |
  | 22+ | `next_decrepitude_level_and_crisis` (open-ended) | `ArMDE:16611` |

  The book's **"Prs"** (`ArMDE:16606`, `ArMDE:16609`) is `Characteristic::Pre`, slug `"pre"`.
  A named row gives **each** Characteristic it names a point — "1 Aging Point in Str
  and Sta" (`ArMDE:16607`) is one point each, not one divided between them.
- **The "3 or more" row is a threshold, not a row.** `ArMDE:16599-16600` ("2 or less — No
  apparent aging" / "3 or more — Apparent age increases by one year") **overlaps**
  every effect row: a 20 both ages the appearance and costs two Characteristics a
  point. Modelling it as two more table rows would make the rows non-exclusive and
  let the table disagree with itself, so it ships as a single number,
  `apparent_age_increase_min: 3`, asked of every total. `ArMDE:16577` states it as a
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
  "increases as an Ability" (`ArMDE:16617`), so the count is the advancement curve's price
  for the next score less the points already accrued, floored at 1 (sitting exactly
  on a boundary must still cost *something*). `points: None` **only** when the curve
  cannot price that score (`AdvancementTable` tops out) — reported as unpriceable,
  never silently costed at 0.
- **"In any Characteristic*s*" is plural** (`ArMDE:16602`, `ArMDE:16611`), so the points may be
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

- Source: `ArMDE:16567-16617`; `ArMDE:16579`
  (the points accumulate), `ArMDE:16577` (the apparent age), `ArMDE:16615` (the player's
  choice), `ArMDE:16617` (Decrepitude follows the points).
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
  derived from `aging_points` through `decrepitude_score` (`ArMDE:16617`) and is never
  written down beside them.
- **Every refusal is a refusal to write**, never a silent adjustment: `AgingError::{
  NoAgingRules, YearAlreadyRecorded, DistributionMismatch, DistributionNotOpen,
  AwardUnpriceable, YearNotRecorded }`. The distribution must sum to exactly the
  points the row left open, and a row that names its own Characteristics accepts no
  distribution at all. Plain data — no issue codes, no Fluent keys — like
  `ChildhoodRejection`.
- **Seeding the apparent age.** `Entity.apparent_age` is `None` on most characters.
  The first year that needs it seeds it at `start_age` (35 — nothing has happened to
  the appearance before the first owed roll, `ArMDE:16577`) and every year after only
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
die input carries `min="0"` and deliberately **no `max`** (`ArMDE:16567`'s exploding
stress die), the total is shown broken into the terms `AgingTotal` already reports,
and the distributor renders one number input per Characteristic and refuses to
submit until it sums to exactly the points the row left open — the UI half of
`ArMDE:16602`/`ArMDE:16611`'s plural "in any Characteristic**s**". The year and die live in
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

- Source: `ArMDE:16577` (one year per
  year), `ArMDE:5189` (Unaging's aside).
- Implementation: `validation/aging.rs::report_apparent_age` emits
  `apparent_age_above_age` (**warning**, args `apparent_age` / `age`) when both are
  entered and the apparent age is higher. A warning, not an error: `ArMDE:5189` states it
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

- Source: `ArMDE:5189` (Unaging), `ArMDE:5743`
  (Bound to (Role)), `ArMDE:3488` (Bee King).
- **The sources describe two independent facts**, and the shipped data used to
  collapse them into one `no_aging` tag — which made the Bee King entry simply
  wrong. Bound to (Role)'s explicit *but* at `ArMDE:5743` is the sentence that proves
  they are separable at all:

  | item | Characteristics do not drop | apparent age does not advance | tags |
  |---|---|---|---|
  | `virtue.unaging` (`ArMDE:5189`) | yes | yes | `no_aging` + `no_apparent_aging` |
  | `flaw.bound_to_role_role` (`ArMDE:5743`) | yes | **no** | `no_aging` |
  | `virtue.bee_king` (`ArMDE:3488`) | **no** | yes | `no_apparent_aging` |

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
    **This is new in 6b6**: `aging_drops` ignored `ArMDE:5189` until the engine started
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

- Source: `ArMDE:5036`.
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
lives on the `migration.rs::SCHEMA_VERSION` constant; the frontend mirror in
`ui/src/lib/state.svelte.ts` is pinned equal to it by test.

Knock-on: `export.rs` now prints an **undated** log entry as a plain bullet rather
than an empty bold label (`an_undated_aging_log_entry_prints_its_effect_without_a_year_label`)
— a consequence of the optional `year`, not a formatting preference. The Markdown
sheet also carries the two new records: `Doc::write_living_conditions` opens the
annotation block with the chosen `Entity.living_conditions`, localized through
`rules/i18n/<lang>/aging.json` (never the slug), because they are a **stored choice**
and a standing term of the aging total (`ArMDE:16567-16569`, table `ArMDE:16581-16594`); and
`Doc::aging_log_entry` appends each logged year's `die` and `total` (`ArMDE:16567`) after
its free text, so an exported sheet records what produced the year's outcome. The
resolved conditions *modifier* is deliberately not printed — it is derived from the
ids plus the character's Virtues and Flaws, and the sheet records choices.

#### Recorded gaps — deliberately not implemented in 6b6
Each is findable here so it is not rediscovered later as a bug. Several were closed by
6b7 and say so in place, rather than being deleted: a closed gap is worth more as a
record of *how* it closed than as a blank. As of the end of milestone 6 the ones still
genuinely open are **Strong Faerie Blood's start-at-fifty**, **Might-holders' immunity
to aging**, **Age Quickly / Baneful Circumstances' schedule rules**, **`ArMDE:16575`'s
discretionary closing clause**, and **crisis survival itself** — the Stamina roll, the
doctor's Medicine roll and death, all out of scope by design.

- **Strong Faerie Blood's start-at-fifty** (`ArMDE:5036`). "You start making aging rolls
  at the age of **fifty**, rather than the normal 35" — the **-3 ships**, the altered
  start age does not. A carrier is still scheduled from `first_roll_age()`. It would
  need a per-trait override of `AgingRules::start_age`, machinery no other shipped
  item asks for. Recorded on `aging_schedule`'s doc comment as well.
- **Might-holders' immunity to aging**, which `ArMDE:5689` presupposes — "he must make an
  additional Aging roll even if he is normally immune to aging because of a Longevity
  Ritual **or Might Score**" — and which mythic companions can hold
  (`entity.might`). The clause states the immunity in passing rather than granting
  it, so no rule is implemented from it; the schedule takes no notice of `might`.
- **The Bronze Cord** (`ArMDE:10844`, "and to rolls to resist aging") — *no longer a gap;
  delivered in 6b7.* It is still deliberately **not** folded into the aging total:
  an aging roll is not a roll one passes or fails, so the referent of "resist aging"
  is the *crisis survival* roll, and `ArMDE:16636` ("Virtues that affect aging rolls do
  not affect crisis survival rolls") keeps the two roll families apart on purpose.
  The slice that revisited it against the crisis rules is 6b7, and it landed on that
  reading: the cord reaches `crisis_survival`'s `modifier_total` as a
  `CrisisModifierSource::BronzeCord` term (`aging.rs`), through the shared
  `derived::bronze_cord_bonus` accessor so the +5 maximum of `ArMDE:10836` keeps its one
  home, and it moves no term of `aging_total`.
  `the_bronze_cord_reaches_crisis_survival_and_never_the_aging_total` (`aging.rs`)
  asserts both directions in one test, and
  `virtues_that_modify_aging_rolls_do_not_affect_crisis_survival_rolls`
  (`data_integrity.rs`) locks the converse against the shipped catalogue.
- **Age Quickly** (`ArMDE:5661`) and **Baneful Circumstances** (`ArMDE:5689`) both ship
  `aging_mod` amount **0**, deliberately. Their real mechanics are *schedule* rules —
  a doubled rate and effective age ("you make two aging rolls each year"), and a
  conditional extra roll — not modifiers to any one roll's total, and this slice
  implements neither schedule. Both stay visible in the surfaced-modifier read-out.
  `age_quickly_contributes_nothing_to_the_total_and_stays_surfaced`
  (`data_integrity.rs`) is a regression test whose doc comment exists so nobody
  "fixes" the 0 into a number.
- **`ArMDE:16575`'s closing clause** — "At the player's and storyguide's discretion, this
  may also apply to characters with modifiers to the aging roll from other sources."
  Explicitly discretionary, so the clamp is **not** extended to non-ritual modifier
  holders unilaterally; `aging_total` gates it on `longevity_ritual.is_some()`.
- **The Crisis** (`ArMDE:16619-16632`) — *no longer a gap; delivered in 6b7.* What 6b6 left
  as a flag `resolve_year` stopped at is now read, resolved and recorded. The serde
  shape (`CrisisRules` /
  `CrisisRow` / `CrisisOutcome` / `CrisisSeverity` / `CrisisDie` / `CrisisAttendant`
  in `aging.rs`, plus `AgingRules.crisis` and the two Decrepitude thresholds of
  `ArMDE:16617` — all optional, so an aging block with no crisis key loads unchanged);
  the **shipped data** below and the load gates of `validate_crisis_rules`; the
  CRISIS TOTAL of `ArMDE:16621` with `ArMDE:16619`'s Decrepitude-first ordering; the survival
  read-out of `ArMDE:16628-16638` with the doctor's allowance (`ArMDE:16634`) and `ArMDE:16636`'s
  wall; the table look-up (`resolve_crisis_row`); `crisis_preview`, which composes all
  four into one reading; and the **write-back** — `resolve_year`'s Crisis leg with
  `ArMDE:16619`'s ordering, the four fields a resolved Crisis records on `AgingLogEntry`,
  and `ArMDE:16573`'s spent Longevity Ritual as an `AgingNote`. The subsections at the end
  of this section carry the provenance for each.
  Survival itself stays out for good: the engine gives the total, the row, the Ease
  Factor and the Creo Corpus level, but never rolls the Stamina die, never resolves
  the attendant's Medicine roll, and never kills — so the `fatal_decrepitude_score`
  of `ArMDE:16617` ships as a datum nothing in the engine acts on. Two consequences of the
  write-back reached the app edge in **6b7c**: `aging_apply` now carries the Simple
  Die through to `resolve_year`, and `aging_preview` answers with the Crisis the
  year *would* write. The preview does that by resolving the year in memory and
  keeping only the reading — never by calling `crisis_preview` on the character as
  it stands, which `ArMDE:16619` makes one Decrepitude short (the year's own award is
  the increase that comes first), so a bare read would show 14 where the log then
  records 15. The Markdown sheet caught up in the same slice: `export.rs`'s
  `aging_log_crisis` prints the row (through the rules i18n), its severity (through
  `crisis-severity-<slug>`, now declared in `LABEL_KEYS`), the Simple Die and the
  CRISIS TOTAL — and says in words when a Crisis is owed and unrolled, which is a
  third state the sheet could not previously tell from no Crisis at all. The golden
  fixture carries one of each.
- **Leprosy's Heavy Wound at a Crisis** (`ArMDE:6340`) — "whenever she undergoes an Aging
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
    survive an aging crisis." (`ArMDE:4530`) — **one sentence, two mechanics, two
    destinations.** This is the proof case for `ArMDE:16636`, "Virtues that affect aging
    rolls do not affect crisis survival rolls": that general prohibition governs the
    +1, which is an aging-roll modifier and so stays out of the crisis, while the +3
    is a *specific* grant to the survival roll and survives the general rule. Guards:
    `mild_aging_carries_both_halves_of_4530` and
    `a_crisis_survival_modifier_never_reaches_the_aging_total` (`data_integrity.rs`),
    the latter being `ArMDE:16636`'s converse — a survival-roll grant is not an aging-roll
    modifier either.
  - **`flaw.leprosy`** gains `crisis_heavy_wound 0` beside its `living_conditions -2`:
    "whenever she undergoes an Aging Crisis (page 392) the leper sustains a Heavy
    Wound in addition to any other result" (`ArMDE:6340`). Guard:
    `leprosy_carries_its_crisis_wound_beside_its_living_conditions_penalty`.
- **Covenant-derived Living Conditions.** The four covenant rows are chosen by hand
  today; a covenant will hand over ids in a later milestone.

#### The Crisis Table — shipped values (M6/6b7, data only)

> "| Crisis Roll | Result |" … "| 8 or less | Bedridden for a week |"
> — ArMDE:16624-16632

This subsection records where the *numbers* in `rules/core/aging.json` come from,
because JSON carries no comments — plus the two items below, which are provenance the
load gates deliberately do **not** encode. The mechanics that read them — the CRISIS
TOTAL of `ArMDE:16621`, the table look-up, the survival read-out and the refusals — are the
`#####` subsections that follow, all delivered in 6b7. (This paragraph promised them as
a placeholder until 6b8d noticed they had arrived.)

| Datum | Value | Source line |
|---|---|---|
| `frail_decrepitude_score` | 4 | `ArMDE:16617` |
| `fatal_decrepitude_score` | 5 | `ArMDE:16617` |
| `crisis.die` | 1–10 (Simple Die) | `ArMDE:474` |
| `crisis.attendant` | `ability.medicine`, `int`, EF 6, botch `-3` | `ArMDE:16634` |
| `crisis.bedridden_week` | ≤ 8, bedridden | `ArMDE:16626` |
| `crisis.bedridden_month` | 9–14, bedridden | `ArMDE:16627` |
| `crisis.minor_illness` | 15, EF 3, CrCo20 | `ArMDE:16628` |
| `crisis.serious_illness` | 16, EF 6, CrCo25 | `ArMDE:16629` |
| `crisis.major_illness` | 17, EF 9, CrCo30 | `ArMDE:16630` |
| `crisis.critical_illness` | 18, EF 12, CrCo35 | `ArMDE:16631` |
| `crisis.terminal_illness` | 19+, **no EF**, CrCo40 | `ArMDE:16632` |

Three things a later sweep must not undo:

1. **`crisis.rows` ships in BAND order, not id order — a deliberate exception to the
   project's canonical-serialization rule.** The rows must tile the integers
   contiguously with the open-below row (`ArMDE:16626`) first and the open-above row
   (`ArMDE:16632`) last, which is what the load gates check and what the look-up walks. An
   id-alphabetical sort leaves every value intact and silently breaks all of it.
   Everything else in the file — `living_conditions` above all — stays id-sorted.
   `shipped_crisis_table_carries_the_16626_to_16632_rows` (`data_integrity.rs`)
   asserts the band order so the exception has a test, not just a paragraph.
   **`rules/i18n/en/aging.json` and `rules/i18n/de/aging.json` mirror the same BAND
   order for their `crisis.*` keys**, not their own separate id-alphabetical order —
   full-audit finding V43 caught the two layers disagreeing (i18n was
   alphabetical while core was band-ordered), so a die-roll table read top to
   bottom as "the crisis ladder" told a different story depending on which file
   you opened. Band order was kept (not flipped to id order) because it is the
   one a player actually reads the table in, matching the rulebook's own
   presentation; canonical-serialization's zero-noise-diff goal does not
   outweigh that for a table that is read as a ladder. `living_condition.*` keys
   in both i18n files stay id-sorted, matching core.
   `crisis_row_i18n_order_matches_core_band_order` (`data_integrity.rs`) asserts
   the two i18n files agree with core's order.
   **The other order-meaningful array in the rules data is `PointItem.categories`**
   (`rules/core/virtues_flaws.json`), which holds a descriptor's categories in the
   descriptor's own order so that element 0 is the primary — see the "Full core
   Virtue/Flaw catalogue" section. Same exemption, same reasoning: sorting it
   loses information rather than noise.
2. **Terminal carries no `ease_factor` at all.** `ArMDE:16632` offers no Stamina roll —
   "CrCo40 required to survive" — so the field is absent rather than set to an
   unbeatable number. `Option<i32>` says "no roll"; a 99 would say "roll and lose".
3. **`botch_penalty` ships signed (`-3`)**, matching "the character must subtract 3
   from the survival roll" (`ArMDE:16634`) as the roll takes it: the survival read-out
   **adds** it. (The doc-vs-data disagreement recorded here while the field was
   unread — the doc comment then described a positive magnitude — was settled in
   favour of the signed datum when the survival read-out landed. The field's doc
   comment now states `-3` as stored-and-added, the one sign convention this file
   keeps.)

##### The `+5` between the table and the Creo Corpus guidelines

> "| 15 | … • Resolve a minor aging crisis |" … "| 35 | … • Resolve a terminal aging
> crisis … |"
> — ArMDE:13372-13376

The Creo Corpus guidelines price a minor / serious / major / critical / terminal
aging crisis at **15 / 20 / 25 / 30 / 35**, exactly **5 below** the Crisis Table's
own **20 / 25 / 30 / 35 / 40** (`ArMDE:16628-16632`) — the `+1` Touch magnitude of a
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
   by exactly 3 and 5 down `ArMDE:16628-16632`, but the rulebook never states that
   relation. Gating it would refuse a legitimate house table, and it is the same
   class of invention as `start_age == longevity_clamp.until_age`, which slice 6b6
   rejected for the aging block. What *is* gated is the direction the source does
   state — "The level of spell required depends on the severity of the crisis, as
   noted on the table" (`ArMDE:16638`) — so severity, required Ritual level and Ease
   Factor must each climb strictly down the illness rows, by any step.
2. **`ritual_level == guideline + 5`.** See the offset above: an inference, not a
   stated rule, and one of the two tables is not loaded at all.

##### The CRISIS TOTAL adds the Decrepitude **that year** raised

> "**Crisis:** Increase the character's Decrepitude first, and then roll on the Crisis
> Table." … "**CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score**"
> — ArMDE:16619, :16621

`crisis_total` (`aging.rs`) adds those three terms and no others. Its Decrepitude term
is **as of the crisis year**, not the character's score today: `ArMDE:16619` puts the
year's own increase *first*, and the aging module is deliberately order-independent
(`resolve_year` refuses nothing but a year already recorded), so a player may roll 36,
carry on through 37-40, and resolve 36's Crisis afterwards. A live
`effective::decrepitude_score` read would charge that Crisis with four later years'
Aging Points. `decrepitude_points_as_of` therefore takes the lifetime total
(`ArMDE:16617` — "Every Aging Point also counts as an experience point towards
Decrepitude") less the points of every `aging_log` entry whose `age` is **strictly**
greater, so the crisis year's own award stays in and an undated legacy entry
subtracts nothing. `the_crisis_total_reads_the_decrepitude_that_year_raised_not_todays`
is the regression test.

No trait modifier reaches this total: `ArMDE:16621` names three terms, and `ArMDE:16636` —
"Virtues that affect aging rolls do not affect crisis survival rolls" — refuses the
analogy with `aging_total` for the roll that follows. The Simple Die's 1–10 range
(`ArMDE:474`) is an input affordance a UI applies; the engine totals whatever die it is
given rather than refusing one.

##### Which row a CRISIS TOTAL lands on — `resolve_crisis_row`

> "| Crisis Roll | Result |" … "| 8 or less | Bedridden for a week |" … "| 19+ |
> **Terminal illness**. CrCo40 required to survive. |"
> — ArMDE:16624-16632

`resolve_crisis_row` (`aging.rs`) is the Aging Roll table's `resolve_outcome` twin,
and deliberately the simpler of the two. An aging row means nothing until it is read
against the character — "sufficient Aging Points … to reach the next level in
Decrepitude" (`ArMDE:16602`) is a count the table does not print — whereas a crisis row
already says everything it does. So the look-up takes no `Entity` and returns the
`CrisisRow` itself: the caller needs the **id** as much as the `CrisisOutcome`,
because the row's display text ("Bedridden for a week") lives in
`rules/i18n/<lang>/aging.json` keyed by that id and never in the engine.

Band membership is `CrisisRow::covers`, inclusive at both ends, with an absent bound
meaning an open end — below for `ArMDE:16626`, above for `ArMDE:16632`. Because
`validate_crisis_rules` refuses at load any table that leaves a gap or an overlap
between those two open ends, a `None` from a ruleset that loaded means the ruleset
ships **no Crisis Table**, never that the total fell off the table.
`a_crisis_total_lands_on_the_row_whose_band_covers_it` (`aging.rs`) walks both open
ends and every band between them.

##### One Crisis read whole — `crisis_preview`

> "**Crisis:** Increase the character's Decrepitude first, and then roll on the Crisis
> Table." … "Creo Corpus magic can postpone a crisis, or resolve it if cast as a
> Momentary Ritual."
> — ArMDE:16619, :16638

`crisis_preview` (`aging.rs`) composes `crisis_total`, `resolve_crisis_row` and
`crisis_survival` into the one value a caller needs from `(entity, ruleset, age,
die)`: the CRISIS TOTAL with its three terms, the **id** of the row it lands on, that
row's `CrisisOutcome`, and the survival read-out where one applies. Each of the three
stays the single home of its own rule; what the composition adds is the *pairing* —
the row is looked up against the total this call computed and the survival read-out
against the outcome that row carries, so no caller can pair a total with the wrong
row. The row travels as an id because its text ("Bedridden for a week") is i18n data,
never engine prose.

`survival` is `None` for `CrisisOutcome::Bedridden` (`ArMDE:16626`, `ArMDE:16627`): a week or a
month in bed is time, not a roll, and an empty read-out would read as "survivable on
a 0".

It **writes nothing**. `resolve_year` remains the aging subsystem's single writer;
`ArMDE:16619`'s "increase the character's Decrepitude first" is honoured by *reading* the
score as of the crisis year (`decrepitude_points_as_of`), not by raising anything
here. `the_crisis_preview_composes_the_total_the_row_and_the_survival_roll`
(`aging.rs`) asserts the character is byte-identical after a preview, and
`a_crisis_preview_leaks_no_aging_roll_modifier_into_either_half` re-pins `ArMDE:16636`
through the composed path — a read-out holding the total and the survival roll in one
value is a second place an aging-roll modifier could leak into either.
`the_shipped_crisis_table_answers_a_total_end_to_end` (`data_integrity.rs`) walks the
same path against the **shipped** `rules/core/aging.json` through the crate's public
surface, which is where the attendant of `ArMDE:16634` actually ships.

##### The Crisis leg of the single writer — `resolve_year`

> "**Crisis:** Increase the character's Decrepitude first, and then roll on the Crisis
> Table."
> — ArMDE:16619

`resolve_year` (`aging.rs`) gains the leg that makes a Crisis part of the character.
`AgingYearRequest` grows `crisis_die: Option<i32>` and `AgingYearResult` grows
`crisis: Option<CrisisPreview>`; everything the leg reads is `crisis_preview`'s, so no
crisis rule is implemented twice.

- **The order is the rule.** A row carrying a Crisis awards its Aging Points like any
  other row, and **that award IS `ArMDE:16619`'s increase**. The Crisis is then read off the
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
  error either — whether a Crisis happened is `ArMDE:16602`/`ArMDE:16611`'s call, never the
  player's, and nothing is written from the unused die
  (`a_crisis_die_never_invents_a_crisis_the_table_did_not_call_for`).
- **`ArMDE:16636` gets a third pin**, and this is the worst of the three places it could
  leak: one call now computes the AGING TOTAL, which *does* take the trait modifiers,
  and the CRISIS TOTAL, which takes none, off **one** character — the shape that
  invites someone to reuse a modifier between them. A character wearing Faerie Blood,
  Poor Living Conditions and Mild Aging is driven through the whole write-back in
  `a_resolved_crisis_year_leaks_no_aging_roll_modifier_into_the_crisis` (`aging.rs`):
  the aging half takes the `-1`, the CRISIS TOTAL is still the three terms of
  `ArMDE:16621`, and only Mild Aging's `+3` reaches the survival read-out the year hands
  back.
- **`revert_year` needed nothing**, and that is a claim rather than an omission: the
  Crisis is recorded inside the year's own entry and the entry is what the revert
  removes, so a Crisis year still restores the save byte for byte — the Longevity
  Ritual included, which costs nothing precisely because the leg reports it spent
  instead of deleting it
  (`reverting_a_resolved_crisis_year_leaves_the_character_byte_identical`).
- **Walked against the shipped catalogue**, not only against fixtures:
  `a_shipped_crisis_year_is_written_into_the_character_and_reverts_exactly`
  (`data_integrity.rs`) drives a companion of 40 through a 9 (`13`, the `ArMDE:16602` row),
  five Aging Points to Decrepitude 1, a Simple Die of 10 and so a CRISIS TOTAL of 15 —
  the shipped minor illness with its Ease Factor 3, CrCo20 and the `ArMDE:16634` doctor,
  which only the real `rules/core/aging.json` ships — and then back off byte for byte.
- App/UI: **the die reaches the engine from the command edge.** This bullet
  described a stopgap — `ruleset_io::aging_apply_loaded` passing `crisis_die: None`
  — that 6b7c removed in the same slice; both `aging_apply_loaded` and
  `aging_preview_loaded` now take `crisis_die: Option<i32>` from the
  `aging_apply` / `aging_preview` commands and hand it to `resolve_year`. A die
  given for a year the table sent to no Crisis is simply unused: whether a Crisis
  happened is `ArMDE:16602`/`ArMDE:16611`'s call, never the player's.

##### The Longevity Ritual a Crisis spends — reported, never deleted

> "A Longevity Ritual is effective until the character suffers a crisis. When the
> crisis occurs, the ritual assures that the character survives, but its power is
> spent, and the focal ritual must be performed again (see page 261)."
> — ArMDE:16573

`AgingYearResult` gains `notes: Vec<AgingNote>`, and `AgingNote::LongevityRitualSpent`
is the one variant (`aging.rs`). A tagged enum rather than a message, on the
`AgingError` / `ChildhoodRejection` precedent: the engine hardcodes no user-facing
string, so the caller maps the variant through Fluent, and an exhaustive `match` makes
a second note a compile error at every reader until it has been rendered.

**`Entity.longevity_ritual` is not touched.** The ritual is a *stored choice* holding a
player-entered bonus and the focus that "must be repeated" if the ritual is performed
again (`ArMDE:10668`); an engine that cleared it would destroy both, and would make the year
unrevertible into the bargain. Performing the focal ritual again is a season's work the
player records, not an inference the sheet makes. (The sentence's other half — "the
ritual assures that the character survives" — is likewise **not** implemented as an
automatic survival: the engine resolves no survival roll at all, for anyone.)

The note follows the **Crisis**, not the Crisis *roll*: "when the crisis occurs" is the
aging row's doing (`ArMDE:16602`, `ArMDE:16611`) and the Simple Die only decides how bad it was,
so a Crisis owed and unrolled spends the ritual too.
`a_crisis_spends_the_longevity_ritual_and_never_deletes_it` (`aging.rs`) pins all four
cases — rolled, unrolled, no Crisis, no ritual.

##### What a resolved Crisis records — and why `SCHEMA_VERSION` stays 15

> "**CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score**" … "| Crisis
> Roll | Result |"
> — ArMDE:16621, :16624-16632

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
  is a bedridden row (`ArMDE:16626`, `ArMDE:16627`) — time, not an illness. The writer takes both
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
   itself at `ArMDE:43` — *"Negativer Modifikator: wird vom Alterungswurf subtrahiert."* —
   and its rows at `ArMDE:47-56` accordingly print the negation of the book's own column
   (Wohlhabend −2 where `ArMDE:16583` prints +2; Aussätziger +2 where `ArMDE:16592` prints -2).
   That is a **convention difference, not a defect**, but it means the glossary must
   **never** source `rules/core/aging.json`'s numbers. The shipped modifiers come
   from the English `ArMDE:16583-16592`, and the German *names* from the mirrored German
   rulebook body at the same line numbers — not from this table.
2. **Its `ArMDE:67` narrows "3 or more" to "3–9", which _is_ a divergence** from `ArMDE:16600`
   ("3 or more") and from the German rulebook's own mirrored line. Under the book's
   reading the apparent-age row overlaps every effect row, which is exactly why the
   engine models it as `apparent_age_increase_min` rather than as a table row; under
   the glossary's "3–9" a total of 15 would age the Characteristic but not the face.
   The core file is right and the glossary line is the outlier.
3. **The Crisis Table's German severities are a false friend — do not "correct"
   them.** German **_Schwere_ Erkrankung = _Major_ illness** (`ArMDE:16630`) and **_Ernste_
   Erkrankung = _Serious_ illness** (`ArMDE:16629`), which is the opposite of the English
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

**A spell's Arts and level are one short code, requisites included (I4, D83.11).**
The rulebook writes a requisite in parentheses after the Art it belongs with — a
Technique requisite after the Technique, a Form requisite after the Form, several
comma-separated — and the level a space after the code:

> Cr(Re)Ig 30 (Base 5, +1 Touch, +2 Sun, +1 requisite, +1 constant effect) — `ArMDE:19301`
>
> ReAq(Co) 30 (Base 5, +1 Touch, +2 Sun, +1 requisite, +1 constant effect) — `ArMDE:19166`
>
> MuTe(Aq, Co, An) 25 (Base 3, +2 Voice, +2 affect metal, +2 affect humans and animals) — `ArMDE:19241`

`LocalizedRuleset::spell_code` (`ruleset.rs`) is the one place that composes the code,
without the level: the Art abbreviations are localized rules data
(`LocalizedRuleset::abbreviation`, from `rules/i18n/<lang>/arts.json` — Latin, the same
in every locale; an Art shipping none falls back to its display name), the requisites
are the spell's `requisites` ids in data order (`rules/core/spells.json`), each placed
by its catalogue `art_type`. Phantasmal Fire ("Req: Ignem", `ArMDE:14554-14558`) reads
`CrIm(Ig)`, Coat of Flame ("Req: Rego", `ArMDE:14262-14266`) `Cr(Re)Ig`, Fog of
Confusion ("Req: Imaginem, Rego", `ArMDE:13241-13245`) `Mu(Re)Au(Im)`. The codes are
serialized per spell as `LocalizedRuleset.spell_codes`, so the Spells tab shows the
engine's code and only appends the level (`derive.ts::spellCodeWithLevel`). The export
(`export/sections.rs::spell_code`) prints `<code> <resolved level>` (`CrIm(Ig) 20`); an
unresolved General level takes the localized marker in the level's place
(`MuVi General`), and a spell no catalogue holds leaves the cell empty rather than
printing a half-written code. The book itself is not uniform — `ArMDE:4403` writes
"MuAn (Ig)" with a space — and the code follows the stat-line form above.

**A spell marked within a Magical Focus or a Potent Magic field says so** (after-deadline
answer 4). The mark (`SpellSelection::within_focus`, `ArMDE:4399-4422`;
`SpellSelection::within_potent_field`, `ArMDE:4740-4748`) is what makes such a spell
above the plain level cap legal (D81.5), so the code cell appends the localized
`export-spell-within-focus` ("· Focus" / "· Fokus") and `export-spell-within-potent-field`
("· Potent") markers, Focus first, one space apart: `CrIg 20 · Focus · Potent`. No new
column (D73.2 forbids a Casting Total column, and the mark needs none).

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

## Guided wizard step copy (M6/6b8b) — REMOVED

Each wizard step used to open with two or three sentences saying what is decided
there (`wizard-guidance-<phase>`, one Fluent line per `CreationPhase`, with the
per-type Flaw-cap clauses appended on `virtues_flaws`). The whole family — the
per-phase lines, `wizard-guidance-<category>-flaw-cap`,
`wizard-guidance-hermetic-flaw`, and `ui/src/lib/derive.ts`'s `wizardGuidance`,
`GUIDANCE_ARGS`, `guidanceNotes` and `flawCapNotes` — was deleted by
**manual-testing-findings #21**: the player has the rulebook open, and a
paragraph above every step's input surface cost real content its height.

This section is kept as the record of what went and why, because the copy made
rules claims and this file is where those claims were sourced. Nothing in the app
renders any `wizard-guidance-*` key today; `ui/src/lib/i18n.test.ts` asserts the
whole prefix is absent from both locales, so a reintroduced key fails loudly
rather than becoming an unsourced rules statement.

Every rules figure the copy interpolated is still on screen, on the surface that
acts on it rather than in a sentence: the Characteristic point buy on the
Characteristics step's own counter, and the Virtue/Flaw budget on the Virtues &
Flaws balance bar. The Flaw caps are reported by the validator
(`too_many_<category>_flaws`), as described under *Category caps* above.

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
> — ArMDE:1073-1075

The character-sheet listing at `ArMDE:1165` names Personality Traits unconditionally as
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
  character to. Its two statements — the Virtue/Flaw budget numbers (`ArMDE:2209-2211`,
  `ArMDE:2295`, `ArMDE:2303`, `ArMDE:2844`) and the Gift policy line (`ArMDE:2224`) — moved to the
  character banner for a time and were then removed outright by
  **manual-testing-findings #3**, along with the keys that carried them
  (`character-type-explainer`, `character-type-budget`,
  `character-type-gift-required|forbidden|optional`). Neither fact was lost: the
  budget is on the Virtues & Flaws balance bar, which is the surface that spends it,
  and the Gift policy is enforced by the engine — a forbidden Gift is an error, a
  required one is granted automatically. The banner
  (`ui/src/lib/components/CharacterBanner.svelte`) now names the type and nothing
  else.
- **`experience` added, immediately before `abilities`.** The blocks that fund a
  character — early childhood, later life, and for a magus apprenticeship and the
  years after it (`ArMDE:2364`, `ArMDE:2213-2216`) — are chosen and priced on their own step
  now, instead of as a ~620px preamble on the step that spends them. Every
  `life_stage_*` and `childhood_*` code, plus `restricted_xp_unspent`, is filed under
  `experience` for the M6/6b1a reason: a finding belongs to the phase whose *input
  surface* owns the offending value. `magus_minimum_ability`,
  `magus_recommended_ability`, `not_enough_xp`, `xp_solve_bound_exceeded`,
  `duplicate_ability` and the `ability_*` codes stay on `abilities`.

`CreationPhase::ALL` therefore still has twelve members; the magus flow is ten
declared phases plus the wizard's synthetic `review`.

#### `Prereq::IsGrog` — the grog audience twin of `IsCompanion` (D68.9)

> "Grogs may not take this Virtue." (Temporal Influence, `ArMDE:5139`)
> "Grogs may not take this Flaw." (Outlaw Leader, `ArMDE:6548`)
> "This Flaw may only be taken by grogs." (Bound to Role (Role), `ArMDE:5747`)

Three entries name grogs as an audience — two excluding them, one admitting only
them — with no engine gate before this milestone.

- Data: `rules/core/character_types.json` — `grog` gains `"is_grog": true`
  (every other profile leaves it unset, default `false`, matching `is_companion`'s
  own shape).
- Implementation: `Prereq::IsGrog` (`types.rs`, beside `IsCompanion`), evaluated
  by `validation/prereq.rs::evaluate_prereq` against
  `PrereqCtx::build`'s new `is_grog: Option<bool>` fact — profile-only, no
  entity-level override, exactly mirroring `is_companion`/`trained`/`order`
  (D56/A0). `ruleset/integrity.rs::validate_prereq_refs` lists it among the
  bare-marker variants (nothing to resolve referentially).
- Data: `virtue.temporal_influence` and `flaw.outlaw_leader` each carry
  `"prerequisites": { "kind": "none", "value": [{ "kind": "is_grog" }] }`
  (`Nor` — "not a grog"); `flaw.bound_to_role_role` carries
  `"prerequisites": { "kind": "is_grog" }` directly ("must be a grog").
- UI mirror: `ui/src/lib/types.ts`'s `Prereq` union gains `{ kind: 'is_grog' }`
  and `EntityTypeProfile.is_grog?: boolean`; `ui/src/lib/derive.ts::houseOnlyValue`
  lists `'is_grog'` among the undecided kinds. Parity enforced by the existing
  `prereq-parity.test.ts`, which diffs the Rust `Prereq` enum against the TS
  union textually — adding `IsGrog` to `types.rs` without the TS/derive.ts twin
  fails that test.
- Tests: `x5_prerequisites.rs` — `grog_is_refused_temporal_influence`,
  `grog_is_refused_outlaw_leader`, `companion_is_refused_bound_to_role`.

#### Blood of the Nephilim — companions only (F-24/D38)

> "Magi and Grogs may not take this Virtue."
> — `ArMDE:3517` (entry `ArMDE:3504-3518`)

"Only companions (and mythic companions) may" is exactly the already-built
`Prereq::IsCompanion` (Wealthy/Poor, above) — no engine change, only the missing
gate.

- Data: `rules/core/virtues_flaws.json` — `virtue.blood_of_the_nephilim` gains
  `"prerequisites": { "kind": "is_companion" }`.
- Test: `x5_prerequisites.rs::blood_of_the_nephilim_is_refused_to_magus_and_grog_but_legal_for_companion`.

#### The Order-audience Story Flaws — Tormenting Master, Vendetta, Hermetic Patron (D68.2)

> "This Flaw is only applicable to magi." (Tormenting Master, `ArMDE:6851-6854`,
> the Gauntlet named one sentence earlier)
> "This Flaw is generally restricted to magi of House Verditius." (Vendetta,
> `ArMDE:6955-6958`)
> "You must be a Redcap or magus to take this Flaw." (Hermetic Patron,
> `ArMDE:6248-6255`)

All three name the Order of Hermes' membership (magus, or — for Hermetic
Patron — a Redcap) as the audience, not Hermetic training: an Abandoned
Apprentice (trained, never Gauntleted, no Order membership) must stay refused
Tormenting Master, which is exactly what `Prereq::OrderMember` (not
`HermeticallyTrained`) gives.

- Data: `flaw.tormenting_master` gains `"prerequisites": { "kind": "order_member" }`
  and reclassifies `narrative` → `creation_effect` (D67: a prerequisite the
  engine checks counts as computed).
- Data: `flaw.vendetta` gains the SAME hard `"prerequisites": { "kind":
  "order_member" }`, alongside its existing (untouched) `advisory_prerequisites:
  { kind: house, value: house.verditius }` — the House half stays a hedge
  (D16), only the unhedged magus half becomes a hard gate. Reclassifies
  `narrative` → `creation_effect`.
- Data: `flaw.hermetic_patron` gains `"prerequisites": { "kind": "any", "value":
  [{ "kind": "has", "value": "virtue.redcap" }, { "kind": "has", "value":
  "virtue.lone_redcap" }, { "kind": "order_member" }] }` and reclassifies
  `narrative` → `creation_effect`.
- Tests: `x5_prerequisites.rs` —
  `tormenting_master_requires_order_membership`,
  `vendetta_house_half_stays_advisory_and_magus_half_becomes_a_hard_gate`,
  `hermetic_patron_requires_redcap_or_magus`.

#### Rector/Proctor and Male Guild Sponsor — closed lists, not a bare Social Status (D68.10/Q-X5-2)

> "The character must have a Social Status Virtue dictating his place within
> the university." (Rector/Proctor, `ArMDE:6673`)
> "The character must select a separate guild Social Status Virtue as well as
> this free Virtue." (Male Guild Sponsor, `ArMDE:4441`)

Both previously read (or would have read) a bare `has_category: social_status`
leaf — vacuous, since every profile already requires *some* Social Status
(D41), so ANY Social Status Virtue (a Merchant, say) would satisfy either
clause without actually naming a university post or a guild rank.

- Data: `flaw.rector` gains `"prerequisites": { "kind": "any", "value":
  [...] }` over seven university-affiliated Social Status Virtues, swept
  directly against each entry's own passage: `virtue.baccalaureus`
  (`ArMDE:3470-3474`, "a three-year program at a university"),
  `virtue.beadle` (`ArMDE:3480-3482`, "employed by the university"),
  `virtue.doctor_in_faculty` (`ArMDE:3683-3691`, "graduated from one of the
  higher faculties of a university"), `virtue.magister_in_artibus`
  (`ArMDE:4385-4393`, "incepted Master of Arts in one of the universities"),
  `virtue.magister_in_medicina` (`ArMDE:4395-4397`, "the same benefits as
  Doctor in (Faculty)"), `virtue.simple_student` (`ArMDE:4958-4962`, "a
  university student"), `virtue.university_grammar_teacher`
  (`ArMDE:5195-5197`, "employed by a university"). Explicitly excluded:
  `virtue.cathedral_school_master` (`ArMDE:3551`, "typically not a university
  man"), `virtue.jurist` (`ArMDE:4165`, "not necessarily university
  trained"), `virtue.nuntius` (`ArMDE:4604`, "employed by a university
  nation" but "not necessarily educated", a distinct claim from holding
  university standing). Reclassifies `narrative` → `creation_effect`.
- Data: `virtue.male_guild_sponsor`'s existing `has_category: social_status`
  leaf is REPLACED by `"prerequisites": { "kind": "any", "value": [{ "kind":
  "has", "value": "virtue.guild_apprentice" }, { "kind": "has", "value":
  "virtue.journeyman" }, { "kind": "has", "value": "virtue.guild_master" },
  { "kind": "has", "value": "virtue.senior_master" }, { "kind": "has",
  "value": "virtue.guild_dean" }] }` — the five guild-rank Social Status
  Virtues the catalogue carries. Classification (`uncomputed_rule`) already
  reflected a computed prerequisite and is unchanged.
- Tests: `x5_prerequisites.rs` —
  `rector_without_a_university_status_is_refused_and_with_one_is_legal`,
  `male_guild_sponsor_without_a_guild_status_is_refused_and_with_one_is_legal`.

#### D51's worked instances — Mercurian Magic, Leper Magus, Mythic Blood

> "All known members of the Mercurian lineage also have the Minor Flaw
> Ceremonial Spontaneous Magic." — Mercurian Magic, `ArMDE:4522`

"Also have" states a prerequisite, not a bundled effect: `virtue.mercurian_magic`'s
`"prerequisites": { "kind": "hermetically_trained" }` becomes `"prerequisites":
{ "kind": "all", "value": [{ "kind": "hermetically_trained" }, { "kind": "has",
"value": "flaw.ceremonial_spontaneous_magic" }] }`.

> "This Virtue can only be bought if the character also has the Leprosy Flaw
> … granting the Life Boost Minor Virtue." — Leper Magus, `ArMDE:4249-4252`

Two separate corrections. First, "can only be bought if… also has" is a
prerequisite `virtue.leper_magus` was missing: its existing three-way
`"prerequisites": { "kind": "all", "value": [order_member, house(house.tytalus)]
}` (already landed by X3) gains a third conjunct, `{ "kind": "has", "value":
"flaw.leprosy" }`. Second, "granting the Life Boost Minor Virtue" is a GRANT,
not a duplicated effect: the entry's inlined `{ "type": "special_casting_mod",
"kind": "life_boost" }` — which duplicated `virtue.life_boost`'s own effect
rather than actually holding that Virtue — is replaced by `{ "type":
"grants_selection", "items": ["virtue.life_boost"] }`.

> "the character also gains a Minor Personality Flaw representing an inherited
> trait from her heritage… both at no extra cost." — Mythic Blood, `ArMDE:4588`
> (cf. the Personality-Flaw soft cap of 2, `ArMDE:2820`)

The hardest of the three: an OPEN grant naming no specific Flaw id, so nothing
exists to hold via `Effect::GrantsSelection` (no id to grant, `Prereq::Has`, or
export by name) — yet it must still count toward the Personality-Flaw category
cap, while remaining free of the point budget (D68.11).

- New machinery, once (D68.11): `Effect::GrantsCategoryCount { category,
  magnitude, item_kind }` (`types.rs`, beside `GrantsSelection`) — an id-less
  phantom item this entity is treated as holding one more of, for
  `validate_caps`'s per-category ceiling/floor counts only. Read from BOUGHT
  selections alone, exactly like every OTHER cap input
  (`validation/caps.rs::validate_caps`'s existing "House-granted items are
  exempt" scope): `push_category_cap_issues` now folds in every bought
  selection carrying a matching `GrantsCategoryCount`, naming that selection's
  own id in the paired-entry message (there being no id of its own to name).
  Every OTHER exhaustive `Effect` fold in the engine (`effective.rs`'s
  `irrelevant_effect_variants!()` macro and its seven call sites, `derived.rs`,
  `effective/spell.rs` ×3, `validation/scores.rs` ×2, `validation/mod.rs`,
  `ruleset/integrity.rs::validate_effect_refs`) gained a no-op arm — it
  contributes to nothing else (no score, no Affinity ratio, no in-play mod, no
  referential check).
- Data: `virtue.mythic_blood` gains a second effect, `{ "type":
  "grants_category_count", "category": "personality", "magnitude": "minor",
  "item_kind": "flaw" }`, alongside its existing `magical_focus` effect.
- **The Magical Focus exclusion needs NO change.** `tmp/x3-scope.md`'s note
  that Mythic Blood should gain `incompatible_with` against both Magical Focus
  Virtues (`ArMDE:4405`, "only one Magical Focus… regardless of the source")
  is already true today by a better mechanism:
  `validation/selections.rs::validate_magical_focus` counts
  `Effect::MagicalFocus` across every bought-PLUS-granted selection, and Mythic
  Blood's bundled focus already carries that effect. Adding `incompatible_with`
  on top would be a second, redundant mechanism for the same rule (D37).
- Tests: `x5_prerequisites.rs` —
  `mercurian_magic_requires_ceremonial_spontaneous_magic`,
  `leper_magus_requires_leprosy`,
  `leper_magus_grants_life_boost_via_grants_selection`,
  `mythic_blood_open_flaw_grant_counts_toward_personality_cap`,
  `mythic_blood_and_a_standalone_magical_focus_already_trip_the_one_focus_rule`
  (SANITY).

#### Shamash and Sofer — require Educated (Hebrew) (F-270/F-283)

> "the character must have the Educated (Hebrew) Virtue" — Shamash,
> `ArMDE:4944`; Sofer, `ArMDE:4996` (same sentence)

Both entries carried the requirement in prose only.

- Data: `virtue.shamash` and `virtue.sofer` each gain `"prerequisites": {
  "kind": "has", "value": "virtue.educated_hebrew" }` and reclassify
  `narrative` → `creation_effect`.
- Tests: `x5_prerequisites.rs::shamash_requires_educated_hebrew`,
  `sofer_requires_educated_hebrew`.

#### Ferocity — animals-only, so unselectable by every buildable type (F-89/Q-11/D58)

> "Like companion and magus characters, this character has Confidence
> points… take 3 Confidence points and a Confidence Score of 1."
> — `ArMDE:3873-3876`

The passage's "animals only" descriptor (shared with `virtue.domestic_animal`
and `flaw.companion_animal`) has no engine model — animal characters are a
deliberate non-goal (D58) — and on a companion/magus its `confidence_bonus`
Effect used to double-count what the type profile already grants for free
(Confidence Score 2/Points 6 instead of the book's unmodified 1/3). Per Q-11's
settled ruling, the fix is the SAME shape F-556 gives `virtue.domestic_animal`:
gate the entry so no buildable human type can select it at all, rather than
inventing a partial arithmetic patch.

- Data: `virtue.ferocity` gains `"prerequisites": { "kind": "none", "value":
  [{ "kind": "is_grog" }, { "kind": "is_companion" }, { "kind":
  "order_member" }] }` — `Nor` over the three profile flags that between them
  cover every shipped type (grog via `is_grog`; companion and mythic_companion
  via `is_companion`; magus via `order_member`), so the Virtue is refused to
  all four.
- This also resolves, rather than requiring a code change for, the review
  finding that `effective/gift_confidence.rs::confidence`'s base-0 gate
  (added for F-306, `virtue.self_confident`'s own grog case) zeroes Ferocity's
  grant for a grog: since a grog can no longer legally hold Ferocity at all,
  that gate's behavior on it is unreachable in the normal build flow, and the
  gate's own job (denying a *delta* Effect any purchase on a base-0 profile)
  remains correct for `virtue.self_confident`.
- Test: `x7bd_wrong_numbers.rs::f89_ferocity_does_not_double_the_companion_profiles_own_confidence_base`,
  RE-SCOPED from an arithmetic assertion (now moot — the state it tested is no
  longer reachable through `validate()`) to an unselectability check across
  all four shipped types.

---

## Engine framework (book-agnostic, no rulebook source)

These checks are structural integrity, not Ars Magica rules, and intentionally
carry no source citation:

- Incompatibility symmetry (`ruleset/integrity.rs` — `validate_incompatibility_symmetry`)
- Required/forbidden traits (`validation/selections.rs` — `validate_required_traits` (:1241),
  `validate_forbidden_traits` (:1262))
- Entity-kind applicability, parameter validation, duplicate-selection detection
  (`validation/selections.rs` — `validate_entity_kind_applicability` (:619),
  `validate_parameters` (:1462), `validate_duplicate_selections` (:909))
- `Prereq` nesting depth bound, `PREREQ_MAX_DEPTH = 32` (K8; `types.rs`, next
  to the `Prereq` enum) — a robustness limit against a pathologically deep
  boolean-expression tree from a crafted or corrupted `rules/` directory,
  enforced at load (`ruleset/integrity.rs` — `validate_prereq_refs`) and, as
  defense in depth, at evaluation (`validation/prereq.rs` —
  `evaluate_prereq`)

### Provenance is guarded, not merely conventional (B11)

Three tests keep the citations in this file and in the tree honest, because
line numbers rot silently and a stale one is worse than none — it sends a
reader confidently to the wrong passage.

- **Outward, into the rulebooks.** `tests/rules_source_provenance.rs` walks
  every `source: { file, lines }` block in `rules/core/*.json` and asserts the
  named English file exists and the range brackets real, non-blank content.
- **Inward, into Rust.** `tests/rules_md_citations.rs` —
  `every_implementation_site_citation_in_rules_md_points_at_the_named_item`
  parses this file's `` `symbol` (:NNN) `` implementation-site citations and
  asserts each cited line actually defines the symbol it names. The B11 sweep
  found **15 of them stale**, every one a function that had simply moved; the
  failure message now names the line the symbol is really on, so the fix is
  mechanical. A floor of 20 parsed citations keeps the test from silently
  degrading to a no-op if the citation wording is ever reworked.
- **No code cites this file by line.** `no_source_comment_cites_rules_md_by_line_number`
  bans `RULES.md:<line>` in Rust comments outright. RULES.md gained over a
  thousand lines in Phase B alone, so such a citation is wrong within days —
  all three that existed pointed at unrelated sections. **Cite a section
  heading instead**; headings are stable and greppable.

Deliberately *not* guarded: the rulebook citations in this file's own prose.
A bare `` `ArMDE:2860` `` inherits its book from the surrounding section, so
resolving one needs a heuristic, and the only defect a bounds check could catch
(a line past EOF) is not the defect that actually happens — an off-by-one,
which lands on a real line and passes. Those are verified by reading.

### Parameter-value identity: trimmed, never case-folded (row 10)

Also framework, and also uncited — no passage says how a typed descriptor is
compared. But the comparison decides **rules output**, so it is recorded here.

A selection's identity is `(item_ref, params)` compared **byte-for-byte** in four
places: `validate_duplicate_selections` (`validation/selections.rs`, whose
`BTreeMap` key is the whole params map), `options.contains(pick)` in
`grant.rs`, `entity.selections.contains(req)` for a mythic type's required
Virtues (`validation/magus.rs`), and `Entity::normalize`'s `selections.sort()`.
So `"Wolf Shape"`, `"wolf shape"` and `"Wolf Shape "` were three distinct
targets, and the per-power cap counted them separately.

**The decision: trim, do not case-fold.** Trimming makes
`ParameterDomain::Text`'s own documentation true ("any non-empty value is
legal") and costs nothing a player intended — leading and trailing space was
never part of a choice. Case-folding is a judgement the rulebook does not ask
for, and two Powers a player deliberately capitalised differently are theirs to
distinguish. `virtue.puissant_ability`-style id domains are trimmed too, because
no domain has a legal value with an edge of whitespace.

**Where it happens:** at **load**, in `load_entity_migrating`
(`migration.rs::trim_all_selection_params`) — covering the bought list plus
`house_choices` / `mythic_choices` / `warping_choices`, exactly like the `being`
fold — and at every frontend write path (`AppStore.setParamAt`,
`setAbilityBonusTarget`, `setArtBonusTarget`, and `ParameterPicker`'s
`onTypeText`, which already did). **Not** in `Entity::normalize`: normalize runs
on every save and sorts selections by `(ref, params)`, so trimming there would
reorder the rows of a file the player had only opened, against the
zero-noise-diff convention. On load the reordering settles once.

**A blank value is a missing choice, not an unknown one.**
`param_value_resolves` now rejects an empty or whitespace-only `Text` value, and
both parameter validators (`validate_selection_parameters`, and
`validate_spell_parameter` for a `SpellSelection`) treat a blank value in **any**
domain as if the key were absent — so it raises `missing_param` naming the box to
fill, not `unknown_param_value`, which renders the offending value and would
print nothing at all. This also makes an unnamed Power read identically whether
the save stored a blank `power` or no `power` key, which is what the v0.2.x
migration wants. No new issue code and no new Fluent key.

Tests: `a_blank_text_param_reads_as_a_missing_choice` (`validation/mod.rs`),
`load_trims_the_whitespace_around_every_param_value` and
`trimming_params_at_load_is_idempotent_and_byte_stable` (`migration.rs`),
`a_padded_ability_roll_mod_subject_surfaces_trimmed` (`derived.rs` — the
`text`-domain parameter of `Effect::AbilityRollModParam` reaches the sheet verbatim),
plus `setParamAt` / `setAbilityBonusTarget` trim cases in
`ui/src/lib/state.svelte.test.ts` and the dirty-flag guard in
`ui/src/App.client.test.ts`.

### An `item` parameter narrowed to a category (B8) — `ParameterDef::require_categories`

Framework, **not** a rule: no passage is cited because none is implemented here.
This is a mechanism the data can now use — the vocabulary for "this parameter
names a *Virtue of category X*" rather than any point item in the catalogue —
and the shipped catalogue did not use it when it landed. Verified at the time of
writing: **no `rules/core/*.json` entry declared an `item`-domain parameter at
all** (the domains in use were `ability`, `art`, `category`, `characteristic`,
`enumerated`, `form`, `realm`, `technique`, `text`), so nothing shipped changed
behaviour and every `rules/` file re-serialized byte-identically. B9 supplied the
first user: the two False Power entries.

- Engine: `ParameterDef::require_categories` (`types.rs`), enforced in
  `validation/selections.rs::param_value_resolves` via
  `item_matches_required_categories`; load-time gates in
  `ruleset/integrity.rs::validate_parameter_defs` (domain) and
  `ruleset/integrity.rs::Ruleset::validate_require_categories` (catalogue).
- Frontend: `ParameterDef.require_categories` (`ui/src/lib/types.ts`) and
  `itemOptionsFor` (`ui/src/lib/components/ParameterPicker.svelte`).
- Tests: `require_categories_defaults_to_empty_and_is_omitted_when_empty`,
  `require_categories_serializes_sorted` (`types.rs`);
  `an_item_value_outside_require_categories_is_flagged`,
  `an_item_param_without_require_categories_admits_any_item`,
  `a_multi_category_item_satisfies_require_categories_by_any_of_its_categories`,
  `index_categories_do_not_satisfy_require_categories` (`validation/mod.rs`);
  `require_categories_on_a_non_item_param_is_rejected`,
  `require_categories_naming_an_uncatalogued_category_is_rejected`,
  `require_categories_naming_a_catalogued_category_loads`,
  `a_spell_parameter_obeys_the_require_categories_rule` (`ruleset.rs`); and
  `offers only items of the required category when the parameter narrows the
  domain` / `leaves an un-narrowed item-domain parameter offering the whole
  registry` (`ui/src/lib/components/ParameterPicker.test.ts`).

**The deliberate mirror of `GrantConstraint::require_categories`.** An open
grant's pick is already narrowed by a set of categories with non-empty-
intersection semantics (`grant.rs`). A parameter narrowed to a category is the
same question asked of a different carrier, so it takes the same field name and
the same semantics rather than inventing a parallel concept ("category_filter",
an allow *and* deny pair, a single category). `forbid_categories` is **not**
mirrored: nothing needs it yet, and YAGNI.

**No new issue code.** A value naming an item outside the required categories
raises the existing `unknown_param_value`. The precedent is exact:
`ParameterDomain::Technique` narrows the Art catalogue to one Art class, and a
Form named where a Technique is required is reported as an unknown *technique*
value, not with a code of its own — because the narrowing IS the domain. So
there is no new Fluent message in either locale, and
`every_validation_code_has_a_fluent_key_in_each_locale` (`arm-app`) has nothing
new to cover.

**Which notion of "category" — membership, whole list.** B2's
`PointItem::categories_for` narrows a *selection* of a multi-category item to
the one category it was `taken_as`. It is **not** consulted here, and cannot be:
a parameter value is a bare `Id` naming an **item**, not a `Selection` of one,
so there is no `params` map from which to read a `taken_as` choice — the
character need not even hold the item, and the picker offering the value has no
selection of it to inspect. Where no `taken_as` choice is available, the answer
is the whole `categories` list, which is what `categories_for` itself falls back
to and what `items_by_category` and every browsing surface already use. A
multi-category item therefore satisfies the requirement through *any* of its
categories. B6's `index_categories` stays invisible, as at every other
membership surface: it records where the book's index files an entry, never what
the entry *is*.

**Both load-time gates, and why each is loud.** `require_categories` on any
domain but `item` is rejected: nothing else resolves against the point-item
catalogue, so the list would be read by no one while looking enforced in the
data — the reasoning `values_on_a_realm_param_are_rejected` established. And a
required category that **no point item carries** is rejected: it admits nothing,
so every value the parameter could name raises `unknown_param_value` and the
declaring item is unfillable — the mirror of `at_most_one_of`'s "a group of one
excludes nothing". Note this is deliberately *stricter* than the treatment of a
type profile's `permitted_categories`, which are **not** checked against the
catalogue because categories are an open, forward-declared namespace: a profile
naming a not-yet-extracted category merely permits nothing extra, which is
harmless and forward-looking, whereas a parameter narrowed to an empty category
takes a working item away. Both gates apply to spells as well as point items,
since both carry `ParameterDef`s.

**Canonical serialization** comes from the type: a `BTreeSet<String>`, so the
authoring order of the JSON list never survives into the output, and
`skip_serializing_if` keeps the empty default — every parameter shipped today —
out of the file entirely.

**The picker narrows with it.** `ParameterPicker`'s `item` branch offered the
whole registry; with `require_categories` it offers only matching items, the
same way the `technique`/`form` branches offer one Art class. Offering an item
the engine will refuse as `unknown_param_value` is offering an illegal choice.

**The other half landed in B9:** whether the named item is one the character
actually *possesses*. That question needs `&Entity`, which `param_value_resolves`
does not have and deliberately does not take; this half needs only `&Ruleset`
and `&ParameterDef`. B9 added `ParameterDef::require_possessed` and a validator
of its own for it, plus `forbid_tainted` — which, being another entity-free
narrowing, folded into `param_value_resolves` beside `require_categories`. Both
are described under *False Power names the Virtue it taints (B9)*, and False
Power is now the first shipped entry to declare an `item`-domain parameter, so
the "nothing shipped uses it" note above is history rather than current fact.

### The saga year (Slice 12, #25; moved onto the document by C8) — `validation/saga.rs`

**One rules value, wrapped in an editing aid.** The saga year — the calendar year
the troupe's saga stands in — is **not** a rule, so it is not in the ruleset. It *is*
character-document state, as of C8: `Entity::saga_year` (schema 17). It used to live
in the app-settings file owned by `arm-app` instead, and that was a defect of the
top-severity class this project names — **wrong rules output**. A storyguide running
a 1220 Rhine saga and a 1197 Iberia saga had a single machine-global number, right
for one of them; opening a character from the other reported the wrong derived age
and could raise a spurious `saga_year_before_birth_year`. A saga year is a property
of the saga, so it travels in the save.

The accepted cost, recorded with the decision: advancing a saga by a year means
touching each character. The named-sagas model (a list of sagas, each with a year,
referenced by id) was considered and deliberately not built.

What `settings.json` keeps is only the year a **new** document is stamped with —
key `default_saga_year` (`crates/arm-app/src/settings.rs`, commands `read_settings` /
`write_settings`; it shares that file with the UI language, the palette and the
validation mode, and the writer is read-modify-write so choosing one never destroys
another). The key was `saga_year` before C8 and is read through a serde `alias`, so
an existing settings file keeps the year its owner chose.

`arm-rules` still gains **no** filesystem dependency: the engine holds only the
arithmetic and the one advisory, receives the year as a plain argument, and takes the
*migration* default as a parameter too (`load_entity_migrating(json,
default_saga_year)`) rather than reading one — `crates/arm-app/src/ruleset_io.rs`'s
`load_entity_from_path` is what passes the user's configured value in.

The **default** year IS a rules value, and is the only cited thing here:

> `ArMDE:597` "That domination persists until the present day, 1220."

Corroborated at `ArMDE:364` ("much like the Europe of 1220, the middle ages") and `ArMDE:440`
("Much like medieval Europe in 1220"). Encoded as
`arm_rules::DEFAULT_SAGA_YEAR = 1220` in `validation/saga.rs`, applied by
`settings::read_settings` whenever there is no stored value, by `Entity::new`, and by
`Entity::saga_year`'s `serde(default)` for a hand-edited schema-17 file that drops
the key — the fallback of last resort, behind both the document's own year and the
user's configured default. It is the only one of the four settings whose default is
resolved in Rust at all: the other three are UI-layer choices whose defaults live in
the frontend store, and `read_settings` reports them honestly as unset rather than
inventing a value. `ui/src/lib/state.svelte.ts` mirrors the constant by hand (its
module-scope placeholder entity needs a year before any IPC can answer), pinned by
`the_frontend_mirrors_the_engine_default_saga_year`.

**What it derives, and what it deliberately does not.** `age` and `birth_year` are
both already stored on the entity, so the saga year adds no *derived* state. It is
the reference the two are linked against while the user types:
`age_in_saga_year(saga_year, birth_year)` and
`birth_year_in_saga_year(saga_year, age)`, exposed as the `derive_age` /
`derive_birth_year` commands so the frontend computes none of it itself. Editing
either half rewrites the other and dirties the document; **editing the saga year
rewrites neither of them.** That is a deliberate refusal, not an omission: advancing a
character by N years requires an aging roll, Living Conditions and any Longevity
Ritual applied *per year* (see *Aging*, above), so a silent recompute would fabricate
ages that skipped their rolls. Saga progression is a separate, explicit feature.

It *does* dirty the document, as of C8 — the year is stored state now, part of what an
unsaved file would lose, so the unsaved-changes guard has to see it move. Before C8 it
dirtied nothing, because it was a preference. The settings dialog's
`default_saga_year` control still dirties nothing, for exactly that reason.

The derivation shares the calendar-year approximation the aging engine already makes
(`aging.rs`, calendar year = `birth_year + age`): birthdays within the year are
ignored, identically in both directions.

**The one finding.** `birth_year` is `i32` and `age` is `u32`, so a saga year *before*
the birth year would underflow. It clamps the derived age to 0 and emits
`saga_year_before_birth_year` — a **warning**, phase `concept`, args `saga_year` and
`birth_year`, localized as `issue-saga_year_before_birth_year` in both locales. Like
the three `childhood_slot_*` rows it is listed in the `ValidationIssue` contract table
but is **not** emitted by `validate`: only the derivation raises it, and that stayed
true through C8 — the year now reaches the engine as entity data, but `validate` was
not given a new rule to enforce with it, so the advisory still comes only from
`age_in_saga_year`. It carries no rulebook citation — no passage forbids an impossible
date; the clamp exists because the type does.

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
| `house_specialisation` | the House is recorded | `ArMDE:2859` "You receive one free Minor Virtue from your choice of House" |
| `mythic_type` | the Mythic Companion type is recorded | — |
| `personality_reputations` | a Personality Trait or a Reputation is recorded | — |
| `aging` | the age is recorded (every other reading on the step is taken against it). Since Slice 12 (#24) the age is *entered* on the `concept` step, so this phase reads as engaged before it is opened — which is correct: the choice it needs has been made | — |
| `review` | always — the closing look at the whole character holds no choices of its own | — |

The House row is the only one resting on a rule, and it is the same choice the
`house_unset` warning is about (see *`validate_house` — specialisation
resolution*); the report merely observes that the choice has not been made, and
still does not require it.

### Catalogued parameter values (CV1/CV2) — `catalogue.rs`

Fixes the D14 literal-instance defect: matching a rules-authored
`ParamValue::Literal` (e.g. an Educated Virtue's Latin exemplar) against a
player-typed `AbilityScore.parameter` used to be plain `==` on two strings, so
a correct answer in the wrong case or language ("Latein" for a German player)
silently failed to authorize or fund. See
`docs/vf-audit/design-cv-catalogued-values.md` (Revision 4) for the full
design. CV1 shipped the catalogue data, its i18n names, and their standalone
load-time integrity; CV2 wires all of it into the real `Ruleset` load path and
adds `Ability.catalogued` — the `AbilityScore`-side matching fix itself (Bound/
Link resolution) lands in CV3 onward.

- Data: `rules/core/parameter_catalogues.json` — four catalogues
  (`catalogue.language_dead`, `catalogue.language_living`,
  `catalogue.organization`, `catalogue.profession`; the language list split by
  L1a, below),
  each value citing the ArMDE passage that names it as a fixed, universal
  value rather than a character-specific one (the design note § 1.1 explains
  why "Organization Lore: Guild" was rejected as a catalogue candidate on
  exactly this ground).
- i18n: `rules/i18n/{en,de}/parameter_catalogue.json` — `{ "names": [{"id",
  "name"}, …] }`, one entry per catalogue value, both locales. German names
  follow `CLAUDE.md`'s translation-table precedence: `orden-tribunale.md:16`
  confirms "House Bjornaer"; no table exists for language/profession names
  (checked `islamische-begriffe.md`, `juedische-begriffe.md`, `fertigkeiten.md`),
  so those fall through to the DE rulebook, which mirrors the English file
  line-for-line (design note § 9).
- Implementation: `crates/arm-rules/src/catalogue.rs` — `Catalogue`,
  `CatalogueValue`, `parse_parameter_catalogues_file` (raw parse) and
  `parameter_catalogue_integrity_errors` (the shared checks below), which
  `load_parameter_catalogues` composes into the CV1 standalone entry point
  and `ruleset/parse.rs::{parse_sources,check_duplicate_ids,assemble_ruleset}`
  (CV2) compose into the real `Ruleset::from_sources` load path — one
  implementation, not two that could disagree. `load_catalogue_names` stays
  standalone by design: it needs BOTH locales at once, which
  `Ruleset`/`LocalizedRuleset` never carry together.
- `Ability.catalogued: bool` (`ability.rs`) — `true` for `ability.dead_language`,
  `ability.living_language`, `ability.profession`, `ability.organization_lore`
  (design note § 1.3; matches the four catalogues above by parameter key).
  `ability.craft`/`ability.area_lore`/`ability.mystery_cult_lore` stay
  uncatalogued. `catalogued: true` does **not** forbid free text: a picker
  offers the catalogue's values plus an "Other…" escape (CV6/CV7), and an
  unrecognised typed value simply stays `Text`, never guessed.
- **L1a — Dead and Living Language draw on separate catalogues** (try-out
  finding 6, Norbert 2026-10-03; amends D59). "In other areas of the world,
  Arabic, Greek and Hebrew fill similar functions, although of these only
  Hebrew is a dead language" (ArMDE:7432); "Gothic, the dead language that the
  House uses for all of its rituals" (ArMDE:3565). Data:
  `catalogue.language_dead` = `language.gothic/hebrew/latin`,
  `catalogue.language_living` = `language.arabic/aramaic/greek/persian`, each
  value in exactly one list; value ids and their i18n names unchanged.
  `rules/core/abilities.json`: `ability.dead_language` / `ability.living_language`
  name theirs with `"catalogue"`. Implementation: `Ability.catalogue:
  Option<Id>` read only through `ability.rs::Ability::catalogue_id` (the field,
  else `catalogue.<parameter key>`), which all four readers call:
  `effective/parameter_options.rs::catalogued_values`,
  `migration.rs::fold_catalogue_matching`,
  `ruleset/integrity.rs::validate_catalogued_abilities` (a named catalogue
  must exist) and `validate_literal_instance` (a literal must sit in the
  ability's own catalogue). Pool literals follow their value: Educated
  (Islamic) "Arabic, Persian, Greek, Latin" (ArMDE:3721) funds Living Language
  Arabic/Greek/Persian and Dead Language Latin; Educated (Hebrew) "Hebrew,
  Aramaic" (ArMDE:3725) funds Dead Language Hebrew and Living Language Aramaic.
  Turb Trained's `language` choice ("whichever single dead language the magi
  speak", ArMDE:5181; Q-X6-2) narrows to the dead list. Saves holding a living
  value under Dead Language are not moved here (that is slice L1b). Tests:
  `tests/l1a_language_catalogues.rs`.
- **L1b — pre-split saves move their languages** (try-out finding 6, decisions
  C5; `SCHEMA_VERSION` 21 → 22). A save migration, not a rule: it carries no
  rulebook number of its own and follows L1a's lists (ArMDE:7432, :3565).
  `migration.rs::move_values_to_their_catalogue` runs only when the file's RAW
  `schema_version` is below 22, read in `load_entity_migrating` before any fold
  stamps it. An instance whose value (a `Catalogued` id, or text naming a value
  in EN or DE) is outside its own Ability's catalogue, but inside exactly one
  catalogue of a sibling Ability with the same parameter key, moves there as
  that `Catalogued` id. It keeps its score, banked XP and specialty. No Ability
  id is hard-coded; on shipped data only Dead ↔ Living Language qualify. On a
  collision the greater `(score, banked_xp)` is kept whole (C5a), and on a full
  tie the instance already in place stays. Each move is reported once through
  `LoadedEntity::moved_ability_parameters`, then `OpenedDocument` in arm-app,
  then `derive.ts::movedAbilityParameterNotice` in the UI. The version is
  stamped only when something moved. A save at 22 or later is never moved:
  `validation/scores.rs::validate_ability_parameter_in_catalogue` reports a
  `Catalogued` id outside its Ability's catalogue as
  `ability_parameter_outside_catalogue`. Turb Trained's recorded `language`
  choice is never rewritten ("whichever single dead language the magi speak",
  ArMDE:5181): a living one is reported as `unknown_param_value`. Tests:
  `tests/l1b_language_save_migration.rs`, `tests/l1b_language_move_notice.rs`.
- Load-time integrity: catalogue ids and value ids unique, both sorted by id,
  each catalogue non-empty; every value has both an `en` and a `de` name; no
  two values within one catalogue collide under trimmed, case-folded
  comparison across the union of their `en`/`de` names (all enforced in
  `Ruleset::from_sources`'s pre-integrity pass, CV2). A `catalogued` Ability
  must declare a `parameter` and that key must name a catalogue that exists
  (`Ruleset::validate_catalogued_abilities`, `ruleset/integrity.rs` — a
  referential check, so it runs post-assembly like every other cross-catalogue
  reference, not in the pre-integrity pass).
- arm-app: `core/parameter_catalogues.json` is a `REQUIRED_CORE_FILES` entry
  (`ruleset_io.rs`) read by `load_ruleset_from_dir` like every other core file
  (empty file = the ruleset ships no catalogues). `load_catalogue_names_from_dir`
  reads `i18n/{en,de}/parameter_catalogue.json` — **both, always**, regardless
  of the requested UI language — against the loaded ruleset's own catalogues;
  composes with the portable-fallback directory `pick_rules_dir` resolves,
  exactly like `load_ruleset_from_dir` does.

| id | EN name | DE name | Source |
|---|---|---|---|
| `language.arabic` | Arabic | Arabisch | `ArMDE:3719-3721` |
| `language.aramaic` | Aramaic | Aramäisch | `ArMDE:3723-3725` |
| `language.gothic` | Gothic | Gotisch | `ArMDE:3563-3565` |
| `language.greek` | Greek | Griechisch | `ArMDE:3719-3721` |
| `language.hebrew` | Hebrew | Hebräisch | `ArMDE:3723-3725` |
| `language.latin` | Latin | Latein | `ArMDE:3711-3713` |
| `language.persian` | Persian | Persisch | `ArMDE:3719-3721` |
| `organization.house_bjornaer` | House Bjornaer | Haus Bjornaer | `ArMDE:3563-3565` |
| `organization.order_of_hermes` | Order of Hermes | Orden des Hermes | `ArMDE:4063-4065` |
| `profession.falconer` | Falconer | Falkner | `ArMDE:3847-3852` |
| `profession.marshal` | Marshal | Marschall | `ArMDE:4449-4453` |
| `profession.master_of_kennels` | Master of Kennels | Meister der Hundezwinger | `ArMDE:4467-4470` |
| `profession.merchant` | Merchant | Kaufmann | `ArMDE:3727-3729` |
| `profession.poet` | Poet | Dichter | `ArMDE:4456-4461` |
| `profession.storyteller` | Storyteller | Geschichtenerzähler | `ArMDE:4456-4461` |

Tests: `crates/arm-rules/tests/parameter_catalogues.rs` — CV1's fixture-based
loading, both-locale names, duplicate-id and cross-locale-collision failures,
plus `shipped_catalogues_and_names_load_clean` against the real shipped files.
`crates/arm-rules/tests/parameter_catalogues_ruleset.rs` — CV2's real-load-path
wiring (`Ruleset::from_sources` exposes catalogues, a broken catalogue fails
the real load, the four listed Abilities read as `catalogued`, and the two
`Ability.catalogued` integrity gaps). `crates/arm-app/tests/commands.rs` —
`load_catalogue_names_reads_both_locales_from_the_shipped_rules_dir` and
`load_catalogue_names_works_through_the_portable_fallback_directory`.

---

#### Mounted Combat (K3, Phase 2 Group F, `design-f0-book-template-engine.md`)

> "A mounted character adds his Ride score, to a maximum of +3, to his Attack
> and Defense Totals, due to higher position and control of a large animal."
> (`ArMDE:16837-16839`, full passage; heading `ArMDE:16837`.)

- **`Entity.mounted: bool`** (additive, `serde(default)`, no `SCHEMA_VERSION`
  bump — byte-compatible with every existing save, absent/false ≡ today's
  behavior) records the player's toggle. `combat_totals` appends a **second,
  mounted** `CombatLine` per existing line, adding `min(Ride, 3)` to Attack and
  Defense only — Initiative and Damage are untouched, per the passage. The
  twins are appended **after** the full on-foot set, in the same relative
  order as the line each doubles (a filter + map over the already-built
  vector, then `extend`), never interleaved with it.
- **The Fist/Kick/Dodge exclusion is a template match, not a stated rule.**
  The passage above names no weapon and no exception at all. The one place a
  mounted twin is actually checkable against the book — the Knight's own
  statblock (`ArMDE:1467-1472`) — prints mounted rows for both his weapon
  lines but **not** for Fist. D66 (`docs/vf-audit/decisions.md`, Norbert,
  2026-09-29) rules that the engine reproduce that template via an
  **explicit, purpose-named catalogue field**, `Weapon::body_attack: bool`
  (`rules/core/equipment.json`, set on `weapon.dodge`/`weapon.fist`/
  `weapon.grapple`/`weapon.kick` only — `weapon.grapple` added by X10(d), D68.5,
  the same unmounted-body-attack template) — never `min_strength` or any other field whose
  documented purpose is unrelated (the exact "rules meaning inferred from an
  unrelated field" mistake D52 already named once). Reproducing a template's
  presentation, deliberately, is one of this project's recorded "(d)
  unsettled, resolved as a presentation choice" outcomes (alongside K3's own
  mounted-twin scoping and K5's `LoadoutState`), not a mechanic the Mounted
  Combat passage itself contains.
- **Integrity check** (`data_integrity.rs::body_attack_is_set_on_exactly_the_body_attacks`):
  a structural invariant on named items (CLAUDE.md: "never exact catalogue
  totals") — exactly `weapon.dodge`/`weapon.fist`/`weapon.grapple`/`weapon.kick`
  carry `body_attack: true`, no other shipped weapon does. (The test's own name
  still says "three"; X10(d) adds the fourth to its `EXPECTED` list without
  renaming it.)
- **The Knight, reproduced exactly** (`ArMDE:1467-1472`): with the fixture's
  `mounted: true` and Ride 5 (capped at +3), `combat_totals` returns all eight
  lines the book's five-row block implies once the engine's own "dropped
  shield" bare line (K5) is accounted for — long sword+shield and the great
  sword each gain a mounted twin; Fist stays singular. Long sword+shield
  (mounted): Atk +17, Def +17; great sword (mounted): Atk +16, Def +13 — both
  exact. `book_templates.rs::the_knight_matches_the_book`.
- UI: a whole-character "Mounted" checkbox (`EquipmentTab.svelte`,
  `data-testid="mounted-toggle"`) → `state.svelte.ts::AppStore.setMounted`.
  Fluent `mounted-label` (en/de); `derived-combat-mounted-suffix` (en/de,
  reserved for the "(mounted)" row-name suffix — not yet consumed by
  `export/sections.rs::combat_line_name` or the UI's mirror, since neither
  needed changing for the derived figures themselves to be correct; wiring
  the suffix in is cosmetic follow-up, not gated on anything here).
- Source: `ArMDE:16837-16839`. `derived/combat.rs::combat_totals` /
  `types.rs::Entity::mounted` / `derived/combat.rs::CombatLine::mounted` /
  `equipment.rs::Weapon::body_attack`.

#### Berserk / Ways of the Land / Cyclic Magic / Special Circumstances (D61, D15, Phase 2 Group F)

> Berserk: "While berserk, you get +2 to Attack and Soak scores, but suffer a
> -2 penalty to Defense." (`ArMDE:3500-3503`.) Ways of the Land: "You get a +3
> bonus to all rolls, including combat and Casting Scores, that directly
> involve that area and its inhabitants" (`ArMDE:5231-5234`). Cyclic Magic
> (positive): "At those times, you receive a +3 bonus to all Casting Scores.
> The bonus also applies to Lab Totals if the positive part of the cycle
> covers the whole season." (`ArMDE:3635-3638`.) Cyclic Magic (negative): the
> mirrored -3 penalty (`ArMDE:5893-5896`). Special Circumstances: "gaining a
> +3 bonus to your Casting Scores and Magic Resistance"
> (`ArMDE:4998-5001`).

- **The defect these four/five ids shared:** each bonus/penalty is stated as
  conditional in its own passage ("while berserk", "that directly involve that
  area", "at those times", "in certain uncommon situations") but was folded
  into the engine's totals **unconditionally** — the printed book templates
  disagreed (the Berserker's Soak/Attack/Defense, the Bjornaer's and the
  Mercere's Casting Totals all print the **not-active** figure). No template
  anywhere prints the boosted/penalized state as a second row the way K3's
  Mounted Combat does, so D58's "extra row where the template prints one" is
  not in tension with deleting rather than surfacing (`docs/vf-audit/
  design-f0-book-template-engine.md` § 1a).
- **Ruling (D61, invoking D15's already-shipped `flaw.corrupted_spells`
  precedent):** delete the conditional effect(s) outright — no new `Effect`
  field, no `ModifierFamily`, no surfaced row — and reclassify per what
  remains (`Classification`'s own rule, `types.rs:224-227`: an entry the
  engine computes *something* for is `in_play_effect` even if its passage also
  states an uncomputable clause; zero remaining effects makes it
  `uncomputed_rule`).
  - **`virtue.berserk`**: all three effects deleted (`combat_mod` ×2,
    `soak_mod`) → `uncomputed_rule`. The Berserker's printed Soak/Pole Axe/Kick
    figures (Soak +9, +13/+7, +6/+4, `ArMDE:1212-1215`) are now exact.
  - **`virtue.ways_of_the_land`**: its one `casting_total_mod` deleted →
    `uncomputed_rule`. Its `description` (both locales) already stated the
    full passage before this change and needed no authoring. The Bjornaer's
    printed Casting Totals (`ArMDE:1643-1650`) are now exact.
  - **`virtue.cyclic_magic_positive` / `flaw.cyclic_magic_negative`**: only
    the `casting_total_mod` clause is deleted; the `lab_total_mod` (X7a's
    separate Lab Total correctness question, since **resolved** — see the
    "Flat lab-total bonus/penalty" bullet and the `LabTotalMod`/`cycle`
    parameter rows above) is untouched by *this* deletion, so both entries
    **stay** `in_play_effect`. The two `COMPUTED_ENTRY_COVERS_WHOLE_
    PASSAGE` exemption rows in `uncomputed_clauses.rs` that used to read "both
    stated bonuses... are computed via two effects" are **removed** (not left
    stale) — that guard matches by id only and never re-checks its own prose
    claim, so a stale row would silence it forever. Removal alone (with the
    new `description`, below) satisfies `no_swept_entry_drops_an_uncomputed_
    mechanical_clause` without any exemption at all.
  - **`virtue.special_circumstances`**: only its `casting_total_mod` is
    deleted; `magic_resistance_mod { aura_bonus }` (a pre-existing, already-
    correct D58 family) is untouched, so it too **stays** `in_play_effect`.
  - The Mercere's printed Casting Totals (`ArMDE:1992-1996`, +26/+35) are now
    exact — all three conditional carriers (Cyclic ± and Special
    Circumstances) used to apply their casting bonus/penalty at once, by day
    and by night, in a storm and out of one, for a net unconditional +3.
- **Descriptions added, both locales** (`rules/i18n/{en,de}/virtues_flaws.json`):
  `virtue.berserk` (its `summary` alone states no number, so a `description`
  is mandatory for `every_uncomputed_rule_entry_states_its_rule_in_every_locale`),
  `virtue.cyclic_magic_positive`, `flaw.cyclic_magic_negative` (both needed
  one for `no_swept_entry_drops_an_uncomputed_mechanical_clause` once their
  exemption rows were removed), and `virtue.special_circumstances` (not guard-
  mandatory — its `summary` already states the +3 — added anyway for
  consistency with the other four). While adding
  `virtue.cyclic_magic_positive`'s DE `description`, its DE `summary` was
  found truncated mid-word ("Deine Magie ist auf einen Naturzyklus abgestimmt
  (z.") — a pre-existing extraction defect unrelated to F2, fixed in the same
  change since it sat in the exact entry being edited.
- **Row 45 (X1) is explicitly untouched here:** Berserk's missing
  `ability_authorization` (Martial Abilities at character creation,
  `ArMDE:3502`) is a *different* defect (a missing effect, not an
  unconditional one) and belongs to X1's own assignment, not this fix.
  Likewise Ways of the Land's missing combat-roll clause
  (`ArMDE:5233`, "+3 bonus to all rolls, including combat...") needs no
  effect of its own now that the Virtue is `uncomputed_rule` — the clause is
  already covered by the same `description` that carries the Casting clause.
- Source: `ArMDE:3500-3503`, `ArMDE:5231-5234`, `ArMDE:3635-3638`,
  `ArMDE:5893-5896`, `ArMDE:4998-5001`.
  `crates/arm-rules/tests/f2_conditional_modifiers.rs` (classification/effects/
  description, all five ids); `uncomputed_clauses.rs::no_swept_entry_drops_an_
  uncomputed_mechanical_clause`; `book_templates.rs::the_berserker_matches_the_book`
  / `the_bjornaer_matches_the_book` / `the_mercere_matches_the_book`.

  **Amended by X1/D43** (below): Berserk's `ability_authorization` (Martial
  Abilities, `ArMDE:3500-3503`) is a permanent grant, so it survives this
  fix's deletions and D46 keeps the entry `creation_effect`, not
  `uncomputed_rule` — `f2_conditional_modifiers.rs::berserk_loses_every_
  conditional_effect_and_stays_a_creation_effect` supersedes the test named
  above.

#### Row 45 (X1) — the `ability_authorization` backfill + D43 (`docs/vf-audit/decisions.md`)

**D43's ruling.** A `RestrictedAbilityXp`/`ScaledRestrictedAbilityXp`/
`ReplacesLifeStageXp` pool permits spending **its own** earmarked points on the
Abilities/categories it names, and nothing more — general XP on the same
Ability/category still needs an explicit `Effect::AbilityAuthorization`. The
old `ability_authorizations` folded both into one unconditional set, so a
restricted pool silently authorized spending general XP on the same scope too.
Privileged Upbringing is category-scoped and states this directly
(`ArMDE:4806-4808`: "You may not... buy Academic or Martial Abilities with your
normal pool of experience points unless you have another Virtue or Flaw
permitting that"). Hermetic Experience is **Ability-scoped**, not
category-scoped, and words the same restriction over its own three named
Abilities (`ArMDE:4065`: "you have an additional 50 experience points to spend
on Order of Hermes Lore, Magic Lore, or Latin. You cannot spend other
experience points on Magic Lore or Latin unless the character has another
Virtue or Flaw permitting this").

**Engine (`crates/arm-rules/src/effective/xp.rs`,
`crates/arm-rules/src/validation/authorization.rs`,
`crates/arm-rules/src/effective/parameter_options.rs`):**

- `xp.rs::AbilityAuthorizations` replaces the old bare
  `(BTreeSet<AuthorizedAbility>, BTreeSet<AbilityCategory>)` tuple with two
  pairs of sets: `explicit_abilities`/`explicit_categories` (an
  `AbilityAuthorization`, `AbilityScoreGrant`/`AbilityScoreGrantParam`, or
  `AbilityBonusGated` target — permission that holds regardless of which XP
  funds the spend) and `pool_abilities`/`pool_categories`
  (`RestrictedAbilityXp`/`ScaledRestrictedAbilityXp`/`ReplacesLifeStageXp` —
  permission scoped to that pool's own points).
- `xp.rs::ability_is_authorized` is the one shared predicate D43 asks not to
  be forked: `(category, ability, parameter, categories_set, abilities_set) ->
  bool`. `validation/authorization.rs::validate_ability_authorization` (the
  OWNERSHIP check) calls it twice — once per half, OR'd — unchanged in effect
  from before D43. `xp.rs::build_spends` (the FUNDING check) calls it once,
  against the explicit half alone.
- `xp.rs::Spend` gained `general_eligible: bool`, true for every Art/Mastery
  spend and for an Ability spend that is ungated, hermetically-trained-exempt,
  or explicitly authorized. `xp.rs::build_capacity_matrix`'s GENERAL→spend edge
  is `spend.general_eligible ? spend.cost : 0` (was an unconditional
  `spend.cost`) — a spend only a pool permits gets pool edges only, so demand
  above the pool surfaces as the ordinary `not_enough_xp` shortfall, never
  `ability_category_requires_virtue` (existence stays legal either way).
- `xp.rs::magus_later_life_pool` reads the explicit set only — a pool-implied
  permission (Warrior's own Martial-XP earmark) does not widen what a magus's
  unrestricted later-life block may fund.
- `effective/parameter_options.rs::ability_parameter_options` unions both
  halves — owning-at-all vs. funding-from-general is a funding question, not a
  picker one.

**Data — 62 entries backfilled with an `ability_authorization` effect** (the
verdict table's `docs/vf-audit/corrections.md` § 3.2/§ 3.2a "primary auth" +
"class+auth" findings, all in `rules/core/virtues_flaws.json`; most also
reclassify `narrative`/`uncomputed_rule` → `creation_effect` since the entry
now computes something, D46):

*Category grant* — `virtue.alim` (academic, `ArMDE:3380-3383`),
`virtue.almogaten` (martial, `ArMDE:3396-3403`), `virtue.almogavar` (martial,
`ArMDE:3404-3409`), `virtue.archieunuch` (academic, `ArMDE:3436-3439`), `virtue.beadle`
(academic, `ArMDE:3480-3483`), `virtue.berserk` (martial, `ArMDE:3500-3503`),
`virtue.brother_chaplain` (academic, `ArMDE:3529-3532`), `virtue.brother_knight`
(academic+martial, `ArMDE:3533-3536`), `virtue.brother_sergeant` (martial,
`ArMDE:3537-3540`), `virtue.bureaucrat` (academic, `ArMDE:3541-3544`), `virtue.clerk`
(academic, `ArMDE:3571-3574`), `virtue.educated` (academic, `ArMDE:3711-3713`),
`virtue.eunuch` (academic, `ArMDE:3771-3774`), `virtue.failed_apprentice`
(academic+arcane+martial, `ArMDE:3843-3846`), `flaw.branded_criminal` (martial,
`ArMDE:5749-5752`), `virtue.fidai` (martial, `ArMDE:3877-3882`), `virtue.guild_dean`
(academic, `ArMDE:4045-4048`), `virtue.guild_master` (academic, `ArMDE:4049-4052`),
`virtue.knight` (martial, `ArMDE:4195-4198`), `virtue.lasiq` (martial,
`ArMDE:4233-4236`), `virtue.marshal` (martial, `ArMDE:4449-4456`), `virtue.master_bard`
(arcane, `ArMDE:4457-4462`), `virtue.master_of_kennels` (martial, `ArMDE:4467-4470`),
`virtue.mazdean_priest` (academic, `ArMDE:4480-4487`), `virtue.mendicant_friar`
(academic, `ArMDE:4488-4495`), `virtue.mercenary_captain` (martial,
`ArMDE:4500-4505`), `virtue.notary` (academic, `ArMDE:4598-4601`), `virtue.perfectus`
(academic, `ArMDE:4632-4634`), `virtue.prestigious_student` (academic,
`ArMDE:4792-4795`), `virtue.priest` (academic, `ArMDE:4796-4805`), `virtue.redcap`
(academic+arcane+martial, `ArMDE:4842-4851`), `virtue.religious` (academic,
`ArMDE:4856-4861`), `virtue.rosh_beth_din` (academic, `ArMDE:4878-4883`),
`virtue.senior_clergy` (academic, `ArMDE:4910-4921`), `virtue.senior_master`
(academic, `ArMDE:4922-4925`), `virtue.templar_administrator` (academic,
`ArMDE:5109-5112`), `virtue.town_magistrate` (academic, `ArMDE:5149-5152`),
`virtue.troubadour` (academic, `ArMDE:5157-5164`), `virtue.turb_trained` (martial
half only — the dead-language half needs a player-chosen `language` parameter,
deferred, `ArMDE:5179-5182`), `virtue.venditor` (academic, `ArMDE:5207-5210`),
`flaw.failed_monk` (academic, `ArMDE:6068-6071`), `flaw.outlaw` (martial,
`ArMDE:6542-6545`), `flaw.outlaw_leader` (martial, `ArMDE:6546-6549`).

*Single-Ability grant* — `virtue.blood_of_the_nephilim` (dominion_lore,
`ArMDE:3504-3518`), `virtue.demonic_blood` (infernal_lore, `ArMDE:3649-3662`),
`virtue.faerie_blood` (faerie_lore, `ArMDE:3797-3820`),
`virtue.familiarity_with_the_fae` (faerie_lore, `ArMDE:3857-3860`),
`virtue.magical_blood` (magic_lore, `ArMDE:4359-4372`),
`virtue.master_of_form_creatures` (magic_lore, `ArMDE:4463-4466`),
`virtue.strong_faerie_blood` (faerie_lore, `ArMDE:5032-5047`),
`flaw.diabolic_past` (infernal_lore, `ArMDE:5958-5961`), `flaw.faerie_friend`
(faerie_lore, `ArMDE:6052-6055`), `flaw.faerie_upbringing` (faerie_lore,
`ArMDE:6056-6059`), `flaw.monstrous_blood` (magic_lore, `ArMDE:6454-6467`),
`flaw.warped_by_magic` (magic_lore, `ArMDE:7019-7022`),
`flaw.imagined_folk_tradition_vulnerability` (faerie_lore, `ArMDE:6280-6283`),
`flaw.magical_fascination` (faerie_lore+magic_lore, `ArMDE:6392-6395`),
`flaw.pagan` (faerie_lore+magic_lore, `ArMDE:6570-6573`).

*Id-form (named Abilities, not a whole category)* —
`virtue.lupus_the_wolf` (artes_liberales + Latin, `ArMDE:4335-4338`),
`virtue.simple_student` (artes_liberales + Latin, `ArMDE:4958-4963`),
`virtue.university_grammar_teacher` (artes_liberales + Latin,
`ArMDE:5195-5198`), `virtue.jurist` (artes_liberales, civil_and_canon_law, Latin,
`ArMDE:4163-4168`), `virtue.sufi` (theology_islam, islamic_law, dominion_lore,
`ArMDE:5077-5084`), `virtue.mamluk` (theology_islam id + martial category,
`ArMDE:4443-4448`), `virtue.senior_bard` (all four realm Lores,
`ArMDE:4904-4909`).

*Realm-gated (D14-shape-2, same `ParamGate` `virtue.student_of_realm` already
uses)* — `virtue.folk_magic`: one `ability_authorization` naming all four Realm
Lores, each gated `realm == <its realm>` (`ArMDE:3907-3920`).

**Five entries beyond the verdict table, added after `book_templates.rs`
regressions surfaced they needed the same fix:** `virtue.warrior` (martial,
"You may acquire Martial Abilities during character creation", `ArMDE:5227-
5229`, in addition to its existing 50xp pool), `virtue.arcane_lore` (arcane,
"You may take Arcane Abilities during character generation", `ArMDE:3430-3435`,
likewise), `virtue.cathedral_school_master` (academic, "He may learn any
Academic Ability", `ArMDE:3549-3554`), `virtue.magister_in_artibus` (academic, "You
may buy Academic Abilities during character generation", `ArMDE:4385-4394`),
`virtue.mentored_by_demons` (academic+arcane+martial, "Students of demons may
also have Abilities that are usually restricted to suitable backgrounds" —
read as authorizing the three gated categories, `ArMDE:4496-4499`).

**The Educated family (§ 3.2a) — four new entries, `ArMDE:3715-3729`.**
`virtue.educated_bardic` (`ArMDE:3715-3717`) states only a 50xp pool (Art of Memory,
Profession: Storyteller/Poet, any Area/Organization Lore) and no separate
"may purchase Academic Abilities" sentence — under D43 it carries **no**
`ability_authorization`, the clean minimal case the ruling exists for: owning
Art of Memory stays legal only up to what the pool funds
(`d43_educated_bardic_refuses_spending_beyond_its_pool`). Its three siblings
each state the broader sentence too, so each gets
`ability_authorization{categories:[academic]}` alongside its own pool:
`virtue.educated_islamic` (`ArMDE:3719-3721`), `virtue.educated_hebrew`
(`ArMDE:3723-3725`, its "Characters from Iberia or the East may also spend some of
these points on Arabic" clause is stated in `description`/`summary` rather
than computed — the condition is regional, not a fact the engine tracks),
`virtue.educated_vernacular` (`ArMDE:3727-3729`). Since L1a, the Islamic and Hebrew
pools' Arabic/Greek/Persian/Aramaic instances target `ability.living_language`
(see "L1a" under Catalogued parameter values). German heading anchors added to
`rules/i18n/de/source_anchors.json` (`gebildet-bardisch`, `gebildet-islamisch`,
`gebildet-hebräisch`, `gebildet-weltlich`), same lines as the English
(`ArMDE`-parallel German source).

**`virtue.well_traveled`** (`ArMDE:5239-5242`, was `narrative` with no
effects): "fifty bonus experience points to spend on living languages, Area
Lores, and Bargain, Carouse, Charm, Etiquette, Folk Ken, or Guile" — every
target is `general` category (ungated), so no `ability_authorization` is
needed, only the missing `restricted_ability_xp` pool + reclass to
`creation_effect`. **Follow-on fixture fix:** `companion_priest.json`'s
`xp_pool` moved 590 → 540 — the priest holds several of Well-Traveled's pool
targets, and `xp.rs::two_phase_max_flow` fills restricted pools before
GENERAL, so those 50 points now come from the pool rather than GENERAL; the
fixture's total demand (590) is unchanged but GENERAL's own share of it is
540, and leaving it at 590 left 50 points genuinely unspent
(`general_xp_unspent`).

**`virtue.custos`/`virtue.covenfolk` ↔ `virtue.wealthy`/`flaw.poor`
(row 45, `ArMDE:3629-3634`/`ArMDE:3609-3612`):** "you may not take the Wealthy
Virtue or Poor Flaw" / "You may not take the Wealthy Major Virtue or the Poor
Major Flaw" — `incompatible_with` added on all four entries (symmetric edges).

**Deferred, recorded rather than fixed here (each a `PENDING_DROPPED_CLAUSE`
row in `uncomputed_clauses.rs`, citing the deferring slice):** the Wealthy/
Poor `incompatible_with` gap on `virtue.almogavar`/`virtue.mendicant_friar`/
`virtue.perfectus`/`virtue.turb_trained`/`flaw.branded_criminal` and
`virtue.priest`'s conditional parish-priest Poor-Flaw prohibition (F-340's
family, X4); `virtue.turb_trained`'s open dead-language parameter (X6/D9p1);
`virtue.town_magistrate`'s Ability-3 prerequisite (F-322); `flaw.
magical_fascination`'s "score of 1, but no more" cap (D3-inexpressible).

**Tests:** `crates/arm-rules/tests/x1_authorization_family.rs` (new, 13
tests — the contract for this slice); `book_templates.rs` (DISAGREEMENTS F1/
K1/P1/W1/B1 resolved: `the_female_scholar_matches_the_book`,
`the_knight_matches_the_book`, `the_priest_matches_the_book`,
`the_witch_matches_the_book`, `the_berserker_matches_the_book` now assert
`codes(&[])`); `f2_conditional_modifiers.rs::berserk_loses_every_
conditional_effect_and_stays_a_creation_effect` (D46 re-scope, above);
`uncomputed_clauses.rs` (~60 `PENDING_MECHANICAL_CLASSIFICATION` rows removed,
now `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE`, or `PENDING_DROPPED_CLAUSE` for the
genuine residual gaps above); `rules_source_provenance.rs` (the four new DE
anchors).

---

## X6a — rule-driving parameter engine additions (`docs/vf-audit/design-x6-parameters.md` § 1)

Engine only, **no data** — the 16 rule-driving entries themselves land in X6b.
Every field below is additive on `Effect`/`ParameterDef`/`PointItem` (ruleset
JSON, not saves), so no `SCHEMA_VERSION` bump (D70/Q-X6-4).

**e1 — `gate: Option<ParamGate>` on `SoakMod`/`CharacteristicScoreDelta`/
`AbilityRollMod`, plus `MagicResistanceMod::amount`.** Same idiom as
`CharacteristicScoreDeltaParam`/`GrantsReputation`'s own `gate`: an inactive
gate contributes nothing.

- Engine: `types.rs::Effect::SoakMod`, `Effect::CharacteristicScoreDelta`,
  `Effect::AbilityRollMod`, `Effect::MagicResistanceMod` (each gains `gate`;
  `MagicResistanceMod` also gains `amount: i8`). Consumers: `derived.rs::in_play_mods`
  (`SoakMod`, `AbilityRollMod`, `MagicResistanceMod`'s `AuraBonus` arm — folded
  into a new `InPlayMods::aura_bonus` field), `effective/characteristic.rs::characteristic_score_bonus`
  (`CharacteristicScoreDelta`).
- Tests: `crates/arm-rules/tests/x6a_parameter_engine.rs`, `mod e1_gated_effects`.

**e2 — Commanding Aura's flat MR (plumbing only; relic composition is still
open).** Source: ArMDE:3583, ArMDE:17653, ArMDE:2627 (the book's
"relic absent" composition). `derived/casting.rs::magic_resistance` folds
`mods.aura_bonus` against the ordinary/True-Faith floor via `max()`. The
"relic present" branch (adds instead of competing) needs a `relic_mr`
mechanism, which does not exist yet; until then this is always the
relic-absent path. **X7b-d (8c2a252/F-256) landed only half of the relic
split**: `Effect::RelicTrueFaith` stops a Relic/Powerful Relic from inflating
the bearer's OWN True Faith Score/MR floor, but the relic-as-item mechanic
this note describes (a Magic Resistance the relic itself grants its bearer,
`ArMDE:17607-17623`) is the still-open half — `relic_mr` remains unbuilt.

- Tests: `x6a_parameter_engine.rs::e1_gated_effects::magic_resistance_mod_aura_bonus_applies_when_gate_met`
  (tagged e1/e2 in its own doc comment).

**e3 — Ability-category/id narrowing on `ParameterDef`.** `require_ability_categories:
BTreeSet<AbilityCategory>` (non-empty-intersection, `Ability` domain only,
mirrors `require_categories`) and `forbid_ids: BTreeSet<Id>` (subtractive
mirror of `allow_ids`). Both raise the existing `unknown_param_value` — the
narrowing IS the domain, no code of its own.

- Engine: `types.rs::ParameterDef`; enforced in
  `validation/selections.rs::param_value_resolves` (`Ability` arm); load-time
  gates in `ruleset/integrity.rs::validate_parameter_defs` (domain must be
  `ability`).
- Tests: `x6a_parameter_engine.rs`, `mod e3_ability_category_narrowing`.

**e4 — `exact_count: Option<u8>` on `MultiRef`.** New code
`wrong_param_count` (`issue-wrong_param_count = { $item } names { $count }
value(s) for { $key }, but exactly { $expected } are required.`, both locales).
Counts DISTINCT members of the selection's `BTreeSet<Id>`, so a repeated value
never inflates the count.

- Engine: `types.rs::ParameterDef::exact_count`; enforced in
  `validation/selections.rs::validate_selection_parameters` (the `Multi`/`true`
  arm); load-time gates in `ruleset/integrity.rs::validate_parameter_defs`
  (`multi_ref`-only, rejects `0`).
- Fluent: `issue-wrong_param_count` (`locales/en/main.ftl`, `locales/de/main.ftl`).
- Contract table row: `validation/mod.rs::ValidationIssue` doc comment.
- Tests: `x6a_parameter_engine.rs`, `mod e4_multi_ref_exact_count`.

**e5 — the two XP-scope validators.** *Ability Block:* `Effect::ForbidsAbilityCategoryParam
{ param }`, the parameter-relative sibling of B1's fixed `ForbidsAbilityCategory`,
consumed by the SAME grant-aware validator
(`validation/selections.rs::validate_category_effect_prohibitions`), gaining
an arm that resolves `selection.params[param]` against the new
`ParameterDomain::AbilityCategory` (closed 5-member enum, `AbilityCategory::from_id`,
`ability.rs`; labelled through the already-shipped `ability-category-<slug>`
Fluent family — no new i18n). *Restricted Learning:* `RestrictedAbilityXp::abilities_param:
Option<String>` names a `multi_ref`/`ability` parameter whose resolved set is a
FOURTH eligibility source unioned with `abilities`/`categories`/`instances`
(D48 extended). New validator `validation/selections.rs::validate_ability_xp_scope`
fires only when at least one effective selection carries an entry naming
`abilities_param` (an ordinary earmark — Educated, Warrior, Privileged
Upbringing — never sets it and stays unaffected); it checks every scored
Ability against the union. New code `ability_outside_restricted_scope`
(`issue-ability_outside_restricted_scope = { $item } restricts experience to
{ $allowed }; { $ability } is outside that list.`, both locales).

- Engine: `types.rs::Effect::ForbidsAbilityCategoryParam`,
  `Effect::RestrictedAbilityXp::abilities_param`, `types.rs::ParameterDomain::AbilityCategory`;
  `ability.rs::AbilityCategory::from_id`; `validation/selections.rs::validate_ability_xp_scope`,
  wired into `validation/mod.rs::validate`; export label:
  `export/resolve.rs::Doc::taxonomy_label` (`AbilityCategory` arm).
- Fluent: `issue-ability_outside_restricted_scope` (both locales).
- Contract table row: `validation/mod.rs::ValidationIssue` doc comment.
- Tests: `x6a_parameter_engine.rs`, `mod e5_xp_scope_validators`.

**e6 — Savantism through D29's single resolution point.** `Effect::AbilityScoreCapOverrideParam
{ param, max }` (the favored Ability caps at `max` INSTEAD OF the age band —
may raise, not just lower) and `Effect::AbilityScoreCapAllExcept { param, max }`
(every OTHER Ability caps at `max`, lowering the band). Both fold into
`ability_age_cap` — never a second check beside it, per D29 — with the
override taking precedence (returns outright) over the all-except clamp.

- Engine: `types.rs::Effect::AbilityScoreCapOverrideParam`,
  `Effect::AbilityScoreCapAllExcept`; folded in
  `effective/reputation_and_caps.rs::ability_age_cap`.
- No new Fluent key (existing cap-violation plumbing reads the resolved cap).
- Tests: `x6a_parameter_engine.rs`, `mod e6_savantism_caps`.

**e7 — Warped Senses' conditional incompatibility (surfaced by this pass, not
in `tmp/x6-scope.md`'s own e1-e8 list).** `PointItem::conditional_incompatible_with:
Vec<ConditionalIncompatibility>`, `ConditionalIncompatibility { gate: ParamGate,
forbids: BTreeSet<Id> }` — a per-VALUE extension of the flat
`incompatible_with`, active only when `gate` holds for the declaring
selection. Consumed by `validation/prereq.rs::validate_incompatibilities` as
one more forbidden-id source per selection, reusing the existing
`incompatible` code and its pair-dedup — D58 rules this a hard error even
though the -2 penalty itself stays text (D61).

- Engine: `types.rs::PointItem::conditional_incompatible_with`,
  `types.rs::ConditionalIncompatibility`; consumed in
  `validation/prereq.rs::validate_incompatibilities`; load-time gates in
  `ruleset/integrity.rs::validate_point_items` (gate resolves on the SAME
  item via `validate_param_gate`; every forbidden id is a real point item).
- No new Fluent key.
- Tests: `x6a_parameter_engine.rs`, `mod e7_conditional_incompatibility`.

**Not this slice:** the 16 rule-driving entries' own JSON data (X6b); the
relic/True-Faith composition split (F-256, X7b-d); Commanding Aura stacking,
Turb Trained's dead-language catalogue dependency, and Special Circumstances'
+3 (design note § 5, Norbert's answers recorded in `decisions.md` D70).

---

## X6b — the 16 rule-driving entries' data (`docs/vf-audit/design-x6-parameters.md` § 2)

Data only — every field below rides X6a's already-shipped `Effect`/`ParameterDef`/
`PointItem` machinery; no engine change. Tests:
`crates/arm-rules/tests/x6b_parameter_data.rs`, one behavioral test (or a
required/negative pair) per entry, against the SHIPPED `rules/core/*.json`.

- **`virtue.commanding_aura`** — `rank` enumerated {`archbishop`,
  `cardinal_legatus`, `king`, `legatus_missus`, `pope`}. Ten gated effects
  (one `magic_resistance_mod`/`soak_mod` pair per rank): Pope MR 25/Soak +5,
  Cardinal or Legatus a Latere MR 20/+4, Legatus Missus MR 15/+3, Archbishop
  MR 10/+2 (ArMDE:3585-3591), King MR 10/+2 (ArMDE:17651, D70 Q-X6-1). The
  MR bonuses ADD across active sources, per e2's existing `max()`/sum fold.
  The wife rule (ArMDE:17652) and lay-ruler cross-reference (ArMDE:3595)
  stay text. **Classification stays `uncomputed_rule`** — D20/X2a already
  pinned this (`x2_reclassification.rs::commanding_aura_reclassifies_and_states_its_eight_figures`),
  so the gated effects sit alongside the existing eight-figure description,
  not instead of it — the same shape `flaw.repellent`/
  `virtue.special_circumstances` below already use.
- **`flaw.savantism`** — `favored` (`ability` domain). Two effects fold into
  `ability_age_cap` (e6): `ability_score_cap_override_param{max:6}` on the
  favored Ability (overrides the age band, ArMDE:6705), `ability_score_cap_all_except{max:3}`
  on every other (lowers it). The halved starting XP, halved Advancement
  Totals, and the +3 specialization roll stay handed to X7b-d's F-510 — not
  this slice's concern (removed from `PENDING_D46_CLASSIFICATION` in
  `data_integrity.rs` and narrowed in `uncomputed_clauses.rs`'s
  `PENDING_DROPPED_CLAUSE`, both now that the cap half is computed).
- **`flaw.restricted_learning`** — `abilities` (`multi_ref`/`ability`,
  `exact_count: 5`, ArMDE:6685). `restricted_ability_xp{amount:0,
  categories:[supernatural], abilities_param:"abilities"}` (e5): the five
  named Abilities plus any Supernatural Ability a Virtue grants are the ONLY
  funding targets. Reclassified `narrative` → `creation_effect` (D67; row
  removed from `uncomputed_clauses.rs`'s `PENDING_MECHANICAL_CLASSIFICATION`);
  gained a full `description` in both locales (the prior entry carried only
  `summary`, and the looser creation_effect-scoped screen still needs the
  rule stated).
- **`virtue.faerie_blood` / `virtue.strong_faerie_blood`** — shared `heritage`
  enumerated {`bee_king`, `custom`, `dwarf`, `goblin`, `satyr`, `sidhe`,
  `spinnen`, `undine`} (E+, ArMDE:3805-3819/:5032-5047 — "or create a similar
  one"). Sidhe: `characteristic_score_delta{characteristic.pre, amount:1,
  gate}` (ArMDE:3815). Dwarf: `ability_roll_mod{ability.craft, amount:1,
  gate}` (ArMDE:3809, surfaced-only). Goblin: `ability_roll_mod{ability.stealth,
  amount:1, gate}` (ArMDE:3811, "+1 bonus on all totals involving stealth" —
  RC review-C item 3, formerly left text in error; this one names a single
  Ability, exactly Dwarf's shape). Satyr/Spinnen/Undine/Bee King/custom stay
  text (D61) — Satyr/Undine's own totals name no
  fixed target the engine's vocabulary can bind to. **Known gap, flagged for
  Norbert:** Sidhe's own text caps at "+1 to Presence, but not to more than
  +3" (ArMDE:3815), but `CharacteristicScoreDelta` (unlike its
  `CharacteristicScoreDeltaParam` sibling) carries no `cap` field — only a
  fixed-target, always-additive delta, the same shape Great
  Characteristic/Giant Blood use to legitimately EXCEED +3. Adding a `cap`
  field to `CharacteristicScoreDelta` is a small, engine-side, additive
  change (mirroring `CharacteristicDeltaCap` onto the fixed-target variant)
  that X6b's data-only scope does not cover; until it lands, a Sidhe
  character whose bought Presence is already +3 shows +4, one over the
  book's stated ceiling. Strong Faerie Blood additionally carries an
  unconditional `quirk` (`text`, ArMDE:5042 — "Choose one physical quirk").
- **`flaw.monstrous_blood`** — reuses Magical Blood's own `bloodline`
  enumerated {`custom`, `magic_animal`, `magic_human`, `magic_spirit`,
  `magic_thing`} (same four background types, ArMDE:6454-6467 vs. Magical
  Blood's ArMDE:4359-4372) plus a `characteristic` (`characteristic` domain,
  `required_if: bloodline==magic_human`). Magic Human:
  `characteristic_score_delta_param{amount:-1, gate, cap:within_base}` +
  `grants_reputation{score:3, gate}` — B4's Magical Blood pattern, sign
  flipped (decrease, not increase) and the Reputation framed as "poor"
  (ArMDE:6462). Magic Animal/Spirit/Thing's own penalties/powers stay text
  (no fixed Characteristic/roll target to bind to).
- **`flaw.ability_block`** — a three-parameter either/or, since
  `ParamGate` has no OR/disjunction and `class`'s domain must be exactly
  `ability_category` (no room for a `custom` sentinel inside a closed
  5-member enum, unlike the Enumerated-domain E+ entries above): `scope`
  enumerated {`category`, `custom`} (always required) selects the branch;
  `class` (`ability_category` domain, `required_if: scope==category`)
  drives `forbids_ability_category_param{param:"class"}` (e5); `custom`
  (`text`, `required_if: scope==custom`) stays permanently text (D70: no
  closed domain to validate a free-text Ability list against, ArMDE:5653
  states no count to bound it). Reclassified `uncomputed_rule` →
  `creation_effect` (D67; its existing full-passage `description` in both
  locales already satisfied the looser screen, so no text change needed).
- **`flaw.vengeful_powers`** — `taken_as` (`category` domain,
  {`hermetic`, `story`}, `max_total: 1`) records which reading applies
  (ArMDE:6975: "may be taken as a Hermetic Flaw... more commonly associated
  with Supernatural Abilities"); `categories` widened to `["story",
  "hermetic"]`. Data only — category membership is read by X3's trained
  gate / House credit through the existing `categories_for` machinery, no
  new effect. **Added to `x3_trained_gate.rs`'s `PENDING_HERMETIC_FLAWS`**
  (not gated `trained`/`prerequisites`): those fields apply to the WHOLE
  item unconditionally, and there is no per-value conditional `Prereq`, so
  gating it would wrongly block the ordinary Story reading for a non-magus.
  A real per-value gate is a future engine change, not this slice's.
- **`virtue.potent_magic_major` / `_minor`** — `field` (`text`,
  `max_total: 255` — "more than one area of Potent Magic", ArMDE:4742, which
  also bounds the +3/+6 to at most one applying Virtue — see D79 below). Data
  only: the +3/+6 bonus's Lab Total half was X7b-d's D4 fix (each entry's
  `LabTotalMod::scope: within_potent_field_only` +
  `in_play_lab_total_mod_within_potent_field`; X7a-refactor replaced the
  original `derived.rs::D4_WITHIN_FOCUS_ONLY` id list with this data field;
  D79 renamed the scope from `within_focus_only` once its Casting Total mirror
  exposed that the gate was never about Magical Focus at all), coordinated
  with, not duplicated by, this slice.
- **`virtue.special_circumstances`** — `circumstance` (`text`). Closes
  F-541/F-287 (the duplicate-copy inversion): `max_per_target: 255` removed
  (reverts to the default 1, now meaningful since the parameter makes two
  identical copies an exact-tuple duplicate), `max_total: 255` kept
  explicit. Moved from `data_integrity.rs`'s `UNLIMITED_REPEAT_ITEMS` to its
  `TEXT_TARGET_PARAM_ITEMS`. **Classification and the existing
  `magic_resistance_mod{aura_bonus}` effect (amount 0, inert) stay
  untouched** — `f2_conditional_modifiers.rs::special_circumstances_keeps_only_magic_resistance_mod`
  pins `uncomputed_rule`; the +3 stays surfaced-only text (D45/D58/D70: no
  "circumstance is active" toggle exists).
- **`virtue.performance_magic`** — `ability` (`ability` domain,
  `require_ability_categories:[general]`, `max_total: 255` — "distinct
  Virtues for each possible Ability", ArMDE:4648). **Corrected, RC review-C
  item 4a/4b: the category narrowing above was WRONG.** Martial is not
  excluded — ArMDE:4676/:4684 explicitly list Bows/Great Weapon/Single
  Weapon/Thrown Weapon/Brawl as legal Performance Magic choices (with a
  combat-casting restriction, not a ban) — so `require_ability_categories`
  widens to `[general, martial]`. General alone also let through
  `ability.living_language` (catalogued `general`, not `academic` — only
  `ability.dead_language` is), which ArMDE:4646 excludes by name ("not
  ... Language"); `forbid_ids: [ability.living_language]` now excludes it,
  the same mechanism `virtue.magian_lineage_major` uses for True Names. No
  effect — the picker/validator domain is the whole of D9's obligation here;
  classification stays `uncomputed_rule`. See "RC review-C fixes" below for
  the full citation and tests.
- **`virtue.magian_lineage_major`** — `abilities` (`multi_ref`/`ability`,
  `require_ability_categories:[arcane, supernatural]`,
  `forbid_ids:[ability.true_names]`, `exact_count: 3`, ArMDE:4345). No
  catalogued `ability.true_names` exists yet to prove `forbid_ids`
  behaviorally against (pinned as a direct data assertion,
  `x6b_parameter_data.rs::magian_lineage_major_data_forbids_true_names`);
  the field is still correct and future-proof the moment that Ability is
  added. The connected-XP rule (halving between the three) stays text
  (in-play, D61). Classification stays `uncomputed_rule` (unchanged;
  already carried its own `aging_mod` effect at this classification before
  X6b).
- **`flaw.repellent`** — `feature` enumerated {`custom`, `dark_sight`,
  `natural_weapons`, `scales`} (Q-X6-3/D70). Scales:
  `soak_mod{amount:3, gate}` — "a scaled character might have a Soak bonus
  of +3" (ArMDE:6681), the one computable minor advantage (D58.1: if the
  sheet shows it, compute it). Natural weapons (melee use) and dark sight
  (see in the dark) stay text — neither names a fixed total the engine's
  vocabulary can bind to. **Classification stays `uncomputed_rule`** — a
  hard constraint (`tests/fixtures/s1_before_offenders.json`'s
  `en/de flaw.repellent: true` entries require the id to remain
  `uncomputed_rule`, else `uncomputed_clauses.rs::regex_screen_never_loses_an_s1_recorded_flag`
  regresses), the same shape Commanding Aura/Special Circumstances already
  established: a gated, computed effect can sit on an `uncomputed_rule`
  entry when the REST of the passage (the -6 trust penalty here) stays
  uncomputed.
- **`virtue.turb_trained`** — `language` enumerated over the dead-language
  catalogue's values (`language.gothic/hebrew/latin`; D70/Q-X6-2: "a parameter
  over the catalogue's dead languages", ArMDE:5181 "whichever single dead
  language the magi speak" — the full seven-value list until L1a split it). The
  `ability_authorization` effect gains `abilities:
  [{ability: ability.dead_language, instance: {param: "language"}}]`
  alongside its existing `categories: [martial]` — D14 shape 2, the
  param-bound sibling of Custos's fixed-Latin literal. Gained a full
  `description` in both locales (previously `summary`-only; needed once the
  entry started tripping `no_swept_entry_drops_an_uncomputed_mechanical_clause`'s
  looser "does displayed text state a rule" check).
- **`flaw.warped_senses`** — one merged `affliction` enumerated parameter
  covering ten leaf values (`sensitive_hearing/_sight/_smell/_taste`,
  `sensitive_to_cold/_heat`, `weak_hearing/_sight/_smell/_taste`,
  ArMDE:7029-7037), `max_total: 255` — **not** the design note's literal
  "form + sense" two-parameter split. `ParamGate` carries a single
  `equals`, with no OR; two parameters would need "sense required_if
  form ∈ {sensitive, weak}", which no existing gate shape expresses within
  this data-only slice. One combined value per leaf choice sidesteps the
  gap entirely (no engine change, and a simpler single-dropdown UI to
  boot). Four `conditional_incompatible_with` entries (e7), gated on the
  merged value directly: Sensitive Sight forbids Blind; Weak Sight forbids
  Blind AND Keen Vision (ArMDE:7031, :7033); Sensitive Hearing forbids
  Deaf; Weak Hearing forbids Deaf AND Sharp Ears — both "inadvisable" and
  "incompatible" phrasings become hard errors per D58's absolute reading.
  **Same-item pairing now enforced too (RC review-C item 2).** The passage's
  OWN same-copy pairing (Weak Sight forbids a SEPARATE copy holding Sensitive
  Sight; Weak Hearing forbids a separate copy holding Sensitive Hearing,
  ArMDE:7033) was formerly left unbuilt — `conditional_incompatible_with.forbids`
  names OTHER items' ids, and the validator's `selected_ids` collapses every
  copy of one parameterized item to a single id, so a self-referencing
  `forbids` would have wrongly also caught every LEGAL pair of copies
  (Sensitive to Cold + Sensitive to Heat). The new
  `forbids_same_item_values` field (see "RC review-C fixes" below) closes
  this without that trap. The -2 penalty stays text (D61); classification
  stays `uncomputed_rule` (a hard constraint, same `s1_before_offenders.json`
  mechanism as Repellent).

**Fixture and cross-file fallout** (every new REQUIRED parameter making an
old bare selection newly report `missing_param`, per D70/Q-X6-4):
`crates/arm-rules/tests/fixtures/book_templates/{magus_mercere,magus_merinita,magus_verditius}.json`
gained the book's own stated values (Mercere's Special Circumstances "during
a storm", ArMDE:1966; Merinita's Strong Faerie Blood "(Undine)", ArMDE:2014,
plus an invented, mechanically-inert `quirk` the book's own archetype
summary does not state; Verditius's Faerie Blood "(Dwarf)", ArMDE:2165).
`core_type_conformance.rs`'s grog/companion fixtures gained
`scope`/`class` for their bare `flaw.ability_block` selections.
`x1_authorization_family.rs`'s `virtue.turb_trained` row gained a
`dead_language_param()` helper (the `ParamValue::Bound` sibling of the
existing `latin()` literal) alongside its `categories: [Martial]`
expectation.

**Fluent:** twelve new `param-label-*` keys (`rank`, `favored`, `abilities`,
`heritage`, `quirk`, `scope`, `class`, `custom`, `feature`, `affliction`,
`field`, `circumstance` — both locales); no new issue codes (e1-e7's plumbing
already covers every new effect/validator this data exercises).

**i18n:** every new enumerated value id, both locales
(`rules/i18n/{en,de}/virtues_flaws.json`), German terms taken from
`rules/source/de/translation-tables/tugenden-fehler.md` (Bee King, the
Sensitive/Weak (Sense) family) where tabled, else the line-parallel German
rulebook (Commanding Aura's ranks, Faerie Blood's remaining heritages, Turb
Trained's own passage) per standing rule — `Spinnen-Blut` is used over the
German rulebook's own heading spelling `Stinnen-Blut` (ArMDE:3817, whose body
text says "Spinnen" three times against the heading's one "Stinnen";
`tmp/x6-scope.md`'s own `heritage.spinnen` slug agrees). `language.*`'s seven
values are ALSO
duplicated into `virtues_flaws.json`'s own i18n (redundant with
`parameter_catalogue.json`, where they already lived) because
`every_enumerated_value_id_has_english_and_german_text` resolves names
through `virtues_flaws.json` alone, not the app's own
`merge_catalogue_display_names` — the same "second, hand-authored copy, no
completeness guardrail" shape `ability_category.*`'s pre-existing duplication
already has (`derive.ts::displayName`'s own doc comment).

---

#### Ability-minimum / cross-Virtue prerequisites — F-30, F-56, F-145, F-169, F-172, F-183, F-223, F-252, F-261, F-322, F-324 (X5b, `docs/vf-audit/corrections.md` § 3.6)

Eleven entries state an eligibility gate — an Ability score floor, a
cross-Virtue requirement, or both. All eleven are pure data: every gate is
expressible with a `Prereq` variant that already existed
(`All`/`Any`/`Nor`/`Has`/`AbilityMin`), so no engine change landed with this
slice.

- **The id-level proxy for a parameterized Ability applies again, at four more
  sites.** `virtue.cathedral_school_master`, `virtue.doctor_in_faculty`,
  `virtue.magister_in_artibus`, `virtue.magister_in_medicina` and
  `virtue.rosh_beth_din` each state a Latin or Hebrew minimum — an instance of
  the parameterized `ability.dead_language` — and `AbilityMin` matches by
  Ability id alone, ignoring `AbilityScore::parameter` entirely (see "Hermetic
  minimum Abilities" above, which documents this as PERMANENT). Encoding
  `ability_min ability.dead_language 5` for these is that same shipped
  precedent applied to a second family, not a fresh judgement.
- **`virtue.master_bard`'s Profession clause is enforced too (D70).** Norbert,
  2026-09-29: "Master Bard's Profession clause is enforced as 'some Profession
  at 5', the id-level check already used for the Latin minimums." So the gate
  is `all[ability_min(ability.profession, 5), any[the four Lores at 5]]`,
  matching `ArMDE:4457-4462`.
- **`virtue.doctor_in_faculty`'s third clause stays text (F-56).** "The
  Ability that correlates to his faculty degree" depends on this entry's own
  open `faculty` parameter, which no `Prereq` variant can bind to — only two
  of the three stated minima (Artes Liberales, Latin) are encoded.
- **`virtue.license_of_absence` reclassifies `uncomputed_rule` →
  `creation_effect` (D67).** Its only two stated rules ("only a Priest may
  take it", "never a Senior Clergy") are now both captured by
  `all[has(virtue.priest), none[has(virtue.senior_clergy)]]` — per D67, "an
  entry whose only stated rule is such a constraint is `creation_effect`."
- **Three age formulas stay `description` text, not `Prereq`** — none reads
  `Entity::age`, and two are formulas a flat floor cannot express anyway:
  `virtue.cathedral_school_master`/`virtue.magister_in_artibus`
  ("30/25 − Int", F-29/F-171, out of this family's scope), and
  `virtue.rosh_beth_din` ("30 − Int", F-252, in scope — the shipped
  `description` now states it verbatim in both locales).
  `virtue.senior_bard`'s flat "minimum age of 22" (F-261) is likewise
  text-only here since `Prereq::AgeMin` was not yet on `main` at this slice's
  HEAD; a later pass should reconsider it once `AgeMin` ships (it cannot help
  Rosh Beth Din's formula either way).
- **`virtue.physician_of_salerno`'s "must be able to take Academic Abilities"
  (F-223) is not a `Prereq` at all** — it gates on whether a *category* is
  authorized, which no `Prereq` variant tests, and the entry's own
  `restricted_ability_xp` effect grants exactly that permission, so a
  same-entry gate would be circular. `description` text in both locales.
- **`virtue.town_magistrate`** (F-322): `any[ability_min(civil_and_canon_law,
  3), ability_min(common_law, 3)]`, `ArMDE:5149-5152`.
- **`virtue.trained_assassin`** (F-324): `any[has(virtue.fidai),
  has(virtue.lasiq)]` — the passage names no Virtue directly ("one of the
  Social Status Virtues of the Nizaris"), resolved to the two Nizari
  Social-Status Virtues in the catalogue, `ArMDE:5153-5156`.
- Data: `rules/core/virtues_flaws.json`, all eleven entries above. Text-only
  additions: `rules/i18n/{en,de}/virtues_flaws.json` for
  `virtue.physician_of_salerno`, `virtue.rosh_beth_din`,
  `virtue.senior_bard`.
- Tests: `crates/arm-rules/tests/x5b_ability_minimums.rs`.

---

#### Banked Ability/Art XP and the per-spell within-focus marker (X10b/X10c, D65 N5(b)/(c), D73)

> "**Abilities:** All of the character's Abilities, in alphabetical order. The
> format is Ability X(Z) (specialization), where X is the score in the
> Ability and Z is the number of experience points acquired towards the next
> level." (ArMDE:1177.) "**Arts:** The character's scores in the Hermetic
> Arts, in the format Art X (Z), where X is the score and Z the number of
> experience points acquired towards the next level." (ArMDE:1179.) "When you
> cast a spell or generate a Lab Total within your focus, add the lowest
> applicable Art score twice." (ArMDE:4403.)

- **X10b**: `AbilityScore`/`ArtScore` gain `banked_xp: u32` — the book's own
  "Z", raw table-XP a score has already earned toward the next point,
  distinct from both the charged XP and `Entity::xp_pool`.
  `#[serde(default, skip_serializing_if = "is_zero")]`, appended last so a
  pre-X10b save (every field at 0) sorts and round-trips byte-identically.
  **Spend loops**: banked XP joins the raw table total *before* Affinity
  reduction, in both the Ability loop and the Art loop of `build_spends` —
  taxed exactly like any other XP spent toward the score, not added on top
  of an already-charged figure. `saturating_add` closes the hostile-input
  path (a crafted `banked_xp: u32::MAX`): no panic.
- **Validation (new, warning)**: `banked_xp_at_or_above_next_level` fires
  when `banked_xp` is at or above the raw-table delta to the next score —
  that figure names a score the character should already have, mis-recorded,
  not a crash — or at any `banked_xp > 0` sitting at the ceiling score (no
  next row to bank toward). Severity warning, matching
  `ValidationIssue::CODE_GENERAL_XP_UNSPENT` (D73.1 — an error was
  considered and rejected).
- **X10c**: `SpellSelection::within_focus: bool`, same shape
  (`skip_serializing_if` on false). The player's own claim that a known
  spell falls within the character's Magical Focus — free-text
  `Effect::MagicalFocus.param` cannot supply this (MAG8's capability gap
  stands; the engine still cannot *derive* membership, only record the
  claim). `spell_casting_total` selects the one figure the book prints
  beside a known spell: the cell's within-focus formulaic figure when the
  flag is set AND the entity currently holds a Magical Focus, else the base
  figure. **Correction (review finding, `tmp/review-d81.json` #1):** this
  used to read the flag alone, so a Focus Virtue removed after a spell was
  marked left the figure inflated with no Focus to justify it — reachable
  via plain Virtue removal through the normal UI, not only a hand-edited
  save as originally claimed here. Now gated on the same `InPlayMods::has_focus`
  `casting_totals`'s own grid cell already gates its `within_focus` figure on,
  so the picker's per-spell figure and the grid cell can never disagree. A
  pure selector over the two numbers `casting_totals` already computes, so
  Potent Magic (which folds into the within-focus figure alone, never the
  base total) comes along automatically.
- **Export**: the book's own "X (Z)" notation — `write_abilities`/
  `write_arts` append `" ({banked_xp})"` to the score cell only when
  `banked_xp > 0`, so every export written before X10b stays byte-identical.
  **D73.2: no Casting Total column** — the Markdown export prints none for
  any spell today, and X10c does not add one; the marker and the in-app
  totals are the whole of this slice.
- **Saves / `SCHEMA_VERSION`**: no bump (D73.3) — both fields are ordinary
  `serde(default, skip_serializing_if)` additions to an existing Vec-item
  struct, identical in shape to `mastery_abilities`/`from_normal_budget`,
  both shipped bump-free; an old save omits the key, defaults to
  `0`/`false`, and round-trips byte-identically.
- **Book-template conformance**: Guernicus's `art.intellego`
  (`banked_xp: 5`, `xp_pool: 435`) and the Specialist's `ability.bows`
  (`banked_xp: 2`) go from "exact only because the pool happens to
  under-fund the difference" to actually exact (§§ MAG12, S2,
  `docs/book-template-conformance.md`). The MAG8 quartet's per-spell
  `within_focus` marks land on exactly the spells whose printed Casting
  Total matches the cell's within-focus figure (Ex Miscellanea: Wall of
  Protecting Stone, The Crystal Dart, Rock of Viscid Clay, Earth that Breaks
  No More — four spells, verified against the printed figures directly, not
  the two an earlier paraphrase suggested; Flambeau: all five; Mercere:
  Clouds of Rain and Thunder, Clouds of Summer Snow — two, not the one the
  same paraphrase suggested; Tremere: none). The pre-existing MAG4/MAG7
  disagreements (neither candidate figure matches the book) are left
  `within_focus: false` — X10c cannot fix a figure neither candidate
  reaches.
- Fluent: `issue-banked_xp_at_or_above_next_level`, `art-banked-xp-label`,
  `ability-banked-xp-label`, `spell-within-focus-label` (both locales; the UI
  wiring itself is a separate port step).
- Source: `ArMDE:1177-1179` (the "X (Z)" notation), `ArMDE:4399-4422` (Major
  Magical Focus). `types.rs::AbilityScore`, `types.rs::ArtScore`,
  `types.rs::SpellSelection`, `effective/xp.rs::build_spends`,
  `validation/scores.rs::validate_ability_banked_xp`,
  `validation/scores.rs::validate_art_banked_xp`,
  `derived/casting.rs::spell_casting_total`, `export/sections.rs::score_cell`.
  Tests: `crates/arm-rules/tests/x10bc_banked_xp_and_within_focus.rs`,
  `crates/arm-rules/tests/book_templates.rs`. The live-Focus gate
  (`tmp/review-d81.json` #1, above): `crates/arm-rules/tests/focus_marker_needs_focus.rs`.

---

#### RC review-C fixes — Goblin stealth, Performance Magic's Ability filter, Warped Senses' same-copy pairing, F-556 (`tmp/rc-verdicts.md`, `tmp/rc-handover.md`)

Four confirmed defects from the 2026-09-30c review pass, one of them (F-556)
needing a new engine capability D38 had already called for.

> "Goblin Blood: ... you get a +1 bonus on all totals involving stealth."
> — `ArMDE:3811`
>
> "You may not choose any Language, Supernatural, Academic, or Arcane
> Ability." — Performance Magic, `ArMDE:4646`. "While Brawl and Martial
> Abilities may be used in Performance Magic..." — `ArMDE:4684`
>
> "Weak Sight is incompatible with Sensitive Sight, Keen Vision, and Blind
> and you cannot take Weak Hearing with Sensitive Hearing, Sharp Ears, or
> Deaf." — Warped Senses, `ArMDE:7033`

**Goblin heritage's stealth bonus (item 3).** `virtue.faerie_blood` and
`virtue.strong_faerie_blood` each gain `{ "type": "ability_roll_mod",
"ability": "ability.stealth", "amount": 1, "gate": { "param": "heritage",
"equals": "heritage.goblin" } }` — the same shape Dwarf Blood's Craft bonus
already had. Data only.

- Test: `rc_review_c_fixes.rs::goblin_heritage_grants_a_stealth_roll_bonus`.

**Performance Magic's Ability filter (items 4a/4b).** `require_ability_categories`
widens from `[general]` to `[general, martial]` (ArMDE:4676/:4684 explicitly
list Bows/Great Weapon/Single Weapon/Thrown Weapon/Brawl as legal, with a
combat-casting restriction, not a ban), and `forbid_ids: [ability.living_language]`
is added (ArMDE:4646 excludes Language, but `ability.living_language` is
catalogued `general` — only `ability.dead_language` is `academic` — so the
category filter alone did not exclude it; `forbid_ids` is the same mechanism
`virtue.magian_lineage_major` already uses for True Names). **Corrects this
file's own prior claim** at the "Performance Magic" entry above ("General is
the one category left... IS the exclusion") — that reading was wrong against
the primary source; Martial was never excluded.

- Tests: `rc_review_c_fixes.rs::performance_magic_accepts_a_martial_ability`,
  `::performance_magic_rejects_a_language_ability`.

**Warped Senses' same-copy pairing (item 2, new engine capability).** The
cross-item half of ArMDE:7033 (Weak Sight excludes Blind/Keen Vision, Weak
Hearing excludes Deaf/Sharp Ears) was already encoded; the SAME-copy half
(Weak Sight excludes a SEPARATE copy holding Sensitive Sight; Weak Hearing
excludes a separate copy holding Sensitive Hearing) had no engine shape —
`conditional_incompatible_with.forbids` names OTHER items' ids, and
`validate_incompatibilities`'s `selected_ids: BTreeSet<&Id>` collapses every
copy of ONE parameterized item to a single id, so a self-referencing `forbids`
would wrongly also catch every LEGAL pair of copies (Sensitive to Cold +
Sensitive to Heat).

- Implementation: `types.rs::ConditionalIncompatibility` gains
  `forbids_same_item_values: BTreeSet<Id>` — while `gate` holds for the
  declaring selection, an OTHER selection of the SAME item whose own value of
  `gate.param` is one of these is also forbidden (both copies read the SAME
  parameter key, so no second field is needed to name it).
  `validation/prereq.rs::validate_incompatibilities` gains a second loop,
  keyed on same-item-id + differing param value, reading `entity.selections`
  directly (not `selected_ids`) since the id collapse is exactly what this
  shape needs to see past. The pair-dedup key degenerates to `(id, id)` for a
  same-item pairing, so at most one `incompatible` issue is ever raised per
  item regardless of how many colliding copies exist.
- Load-time integrity: `ruleset/integrity.rs::validate_forbids_same_item_values`
  resolves each `forbids_same_item_values` entry against the SAME parameter
  domain `gate.equals` is already checked against (both the declaring copy and
  the copy it forbids are instances of one parameter).
- Data: `flaw.warped_senses`'s `weak_sight` and `weak_hearing` conditional
  entries each gain `forbids_same_item_values` naming their Sensitive
  counterpart.
- Tests: `x6a_parameter_engine.rs::e7_conditional_incompatibility` —
  `weak_sight_excludes_a_second_copy_holding_sensitive_sight`,
  `two_copies_of_the_same_value_carry_no_incompatibility` (synthetic
  ruleset); `rc_review_c_fixes.rs::warped_senses_forbids_pairing_weak_sight_with_sensitive_sight`,
  `::warped_senses_forbids_pairing_weak_hearing_with_sensitive_hearing`
  (shipped ruleset). No new Fluent key — reuses `issue-incompatible`
  (`$item`/`$other`), which for a same-item pairing resolves both to the same
  localized name.

**F-556 — `virtue.domestic_animal` takeable by a human (item 9, new `Prereq`
variant, D75).** ArMDE:3701 opens "The character is an animal who is the
property of a covenant or character", and animal characters are a deliberate
non-goal (D58: no Cunning characteristic, no animal profile) — so the entry
must be gated so NO current (human) character type can select it, without
inventing animal machinery. D38 had already called for "a `Prereq` that can
name a character type" for the opposite need (narrowing TO a type); D75
settles that this is the SAME capability, used here to gate AWAY FROM every
type at once.

- Implementation: `Prereq::CharacterType(Id)` (`types.rs`) — evaluated
  against the entity's own type profile id
  (`validation/prereq.rs::PrereqCtx::type_profile_id`, profile-only like
  `IsCompanion`/`IsGrog`: unknown when the profile cannot be resolved, never a
  definite answer either way). **Deliberately NOT referentially checked**
  against the type-profile registry at load
  (`ruleset/integrity.rs::validate_prereq_refs`) — unlike `Prereq::House`,
  this variant's whole point is to name an id NO profile carries.
- Data: `virtue.domestic_animal` gains `"prerequisites": { "kind":
  "character_type", "value": "character_type.domestic_animal" }` — an id no
  shipped profile carries, so no human character type can ever satisfy it,
  forward-compatible if an animal profile is ever added (against D58's present
  non-goal). Reclassifies `narrative` → `creation_effect` (D67: a prerequisite
  the engine checks counts as computed).
- UI mirror: `ui/src/lib/types.ts`'s `Prereq` union gains `{ kind:
  'character_type'; value: string }`; `ui/src/lib/derive.ts::houseOnlyValue`
  lists `'character_type'` among the undecided kinds. Parity enforced by the
  existing `prereq-parity.test.ts`.
- `docs/vf-audit/corrections.md`'s F-556 row marked fixed.
- Test: `rc_review_c_fixes.rs::f556_domestic_animal_is_refused_for_a_human_character_type`;
  `validation/prereq.rs`'s own unit tests —
  `evaluate_prereq_character_type_true_when_id_matches`,
  `::evaluate_prereq_character_type_false_when_id_differs`,
  `::evaluate_prereq_character_type_unknown_when_type_unresolved`.

**Fluent:** none new — both fixes reuse existing issue codes
(`prereq_not_met`, `incompatible`).

**CLAUDE.md:** the `Prereq` quick-reference block gains `CharacterType(Id)`.

---

## X6c — the label-only parameters + all 3 retypes (`tmp/x6c-verdicts.md`, D80)

Data only, same convention as X6b: one behavioral test per entry against the
SHIPPED `rules/core/*.json`, in `crates/arm-rules/tests/x6c_label_parameters.rs`.
None of these drives a computed rule (D9 still records the choice for the
sheet/export). D80 settled the file's three open questions (Rector takes no
parameter; Demonic Familiar's role is free text; Fida'i/Lasiq's cover status
is optional) — see `d80_optional_parameter.rs` for the one genuine engine
addition this slice needed: `ParameterDef::required: bool` (default `true`),
consumed by `validation::selections::validate_selection_parameters`'s
required-set computation and guarded at load by
`ruleset::integrity.rs` (rejects `required: false` combined with
`required_if` on the same parameter).

- **Required `text` parameters (18), each purely a recorded label with no
  catalogue to resolve against:** `virtue.greater_purifying_touch` → `disease`
  (ArMDE:4027-4030); `virtue.lesser_purifying_touch` → `illness`
  (ArMDE:4287-4290); `virtue.lesser_immunity` → `hazard` (ArMDE:4275-4278);
  `virtue.troupe_upbringing` → `area` (ArMDE:5165-5168); `virtue.focus_power` →
  `focus` (ArMDE:3895-3906); `flaw.baneful_circumstances` → `circumstance`
  (ArMDE:5687-5690); `flaw.deleterious_circumstances` → `circumstance`
  (ArMDE:5917-5920); `flaw.environmental_magic_condition` → `condition`
  (ArMDE:6020-6023); `flaw.environmental_sensitivity` → `feature`
  (ArMDE:6024-6027); `flaw.restriction` → `condition` (ArMDE:6691-6694);
  `flaw.necessary_condition` → `action` (ArMDE:6476-6479);
  `flaw.supernatural_nuisance` → `kind` (ArMDE:6799-6802); `flaw.poor_memory` →
  `kind` (ArMDE:6622-6625); `flaw.lycanthrope` → `predator` (ArMDE:6370-6377);
  `virtue.paid_rights` → `right` (ArMDE:4606-4615); `virtue.templar_office_holder`
  → `position` (ArMDE:5121-5124); `flaw.curse_of_slander` → `section`
  (ArMDE:5881-5884), ADDITIVE alongside its already-shipped `taken_as` category
  parameter — unchanged; `flaw.demonic_familiar` → `role` (ArMDE:5926-5931,
  D80.2: "at the storyguide's discretion... SUCH AS a warder, teacher, or
  paramour" is open-ended, not the closed 4-value enumeration the scope note
  first proposed).
- **Enumerated label parameters (4), each a genuinely closed set the book
  states:** `virtue.lesser_benediction` → `benediction` E+custom
  {`custom`, `gift_of_the_gab`, `green_fingers`, `pricking_thumbs`,
  `unusually_fecund`} (ArMDE:4253-4274 sidebar — four named examples, "should
  be comparable to other Minor Virtues" keeps it open); `virtue.indescribable_face`
  → `form` {`distracting_prop`, `forgettable`}, closed at exactly two
  (ArMDE:4107-4114: "select which form... his character has"); `virtue.alim` →
  `rank` {`major_figure`, `minor_official`} (ArMDE:3380-3383 — the named
  examples mu'adhdhin/imam/mufti/qadi are "such as", so stay out of the enum);
  `virtue.cyclic_magic_positive` → `cycle` {`lunar`, `seasonal`, `solar`},
  reusing the SAME three values `flaw.cyclic_magic_negative` already shipped
  (ArMDE:3635-3638 vs. :5893-5896) — the virtue carried no parameter at all
  before this slice.
- **`flaw.rector` — D80.1, no parameter at all.** ArMDE:6673: "the
  representative leader of his faculty or nation... depending on whether he
  is a master or a student. ... The character must have a Social Status
  Virtue dictating his place within the university" — that required
  prerequisite Virtue already decides faculty vs nation, so there is nothing
  left for a parameter to record. No data change; its `summary` in both
  locales already states the rule verbatim (confirmed, not edited), and D67
  keeps `creation_effect` correct (the `prerequisites` constraint is itself
  engine-enforced).
- **`virtue.fidai` / `virtue.lasiq` — D80.3, one OPTIONAL `item` parameter.**
  `cover` (`domain: item`, `require_categories: ["social_status"]`,
  `required: false`) — ArMDE:3881, :4235: "pretending to have some other social
  status, which you should choose" names an actual Social Status Virtue/Flaw,
  not free text, and `require_categories` narrows to exactly that without a
  new mechanism. `required: false` is the ONE exception to D70's "every new
  parameter is required": the cover applies only while away from home on a
  mission, a fact the engine cannot observe, so it is never reported missing.
- **Retypes, text → enumerated (3 of 3 landed):**
  `virtue.alluring_to_beings`'s `being` (ArMDE:3388-3395 — "one of three
  classes of beings: mundane animals, faeries, or magical beings") now
  resolves against the SAME `being.animals`/`being.faeries`/
  `being.magical_creatures` values `flaw.offensive_to_beings` et al. already
  ship — zero new i18n. `virtue.doctor_in_faculty`'s `faculty`
  (ArMDE:3683-3692 — "in medicine, civil or canon law, or theology") now
  resolves against `faculty.law`/`faculty.medicine`/`faculty.theology`,
  mirroring the Academic Abilities the book ties each faculty to
  (`ability.civil_and_canon_law`, `ability.medicine`,
  `ability.theology_christian`/`_islam`/`_judaism` — the faculty parameter
  itself stays a 3-way axis, not split by creed). `virtue.academic_concentration_subject`'s
  `subject` (ArMDE:3362-3367, :7310 — "one of the seven subjects of Artes
  Liberales": Trivium `grammar`, `logic`, `rhetoric`; Quadrivium `arithmetic`,
  `geometry`, `astronomy`, `music`) now resolves against the seven new
  `subject.*` values. Old free text in any of the three fails
  `unknown_param_value`, never migrated (Q-X6-4/D70), no schema bump.
- **The engine fix the `subject` retype needed.** `Effect::AbilityRollModParam`'s
  declared parameter was hardcoded to `ParameterDomain::Text` in
  `ruleset::integrity.rs::validate_effect_refs`, which rejected at LOAD time a
  ruleset giving it `domain: enumerated`. Widened via a new helper,
  `ruleset::integrity.rs::validate_ability_roll_mod_param_effect` (mirroring
  `validate_deficient_art_effect`'s own Technique-or-Form dual acceptance a
  few hundred lines above), to accept EITHER `Text` or `Enumerated` — a third
  domain still fails load, with a message naming both accepted domains.
  `Effect::MagicalFocus` (Magical Focus's own free-text descriptor) is
  UNCHANGED, Text-only: nothing asks for an enumerated Magical Focus field.
  Reds in `crates/arm-rules/tests/x6c_academic_concentration_domain.rs`.
- **The UI fix.** `derived.rs::surfaced_modifiers` still passes the
  parameter's raw value straight into `SurfacedModifier::detail`, and
  `ui/src/lib/components/DerivedSurfacedModifiersSection.svelte::detailLabel`
  still renders `detail` for the `ability_roll` family whenever `m.ability` is
  `None` — but now through `derive.ts::selectionParamLabel` (newly exported),
  the SAME generic rules-i18n-id-or-verbatim resolver every other
  selection-parameter value already uses, rather than unconditionally raw. An
  enumerated value (`subject.logic`) resolves to its localized rules-i18n
  name; a genuinely free-text value (no rules-i18n entry) still renders
  exactly as entered — so neither the old-save free-text case nor Focus
  Power's typed field changes behavior. Confirmed this is the only rendering
  surface: the Markdown export (`crates/arm-rules/src/export/`) never reads
  `surfaced_modifiers` at all, and `DerivedSurfacedModifiersSection.svelte` is
  the sole consumer of `DerivedTotals.surfaced_modifiers` in `ui/src`, shared
  by every creation mode (there is no separate wizard-preview copy).

**Fluent (`locales/{en,de}/main.ftl`):** 10 new `param-label-*` keys —
`disease`, `illness`, `benediction`, `action`, `kind`, `section`, `predator`,
`right`, `cover`, `position`. 13 others (`being`, `faculty`, `hazard`, `form`,
`area`, `focus`, `cycle`, `circumstance`, `condition`, `feature`, `role`,
`rank`, `subject`) are reused as-is.

**Rules-i18n (`rules/i18n/{en,de}/virtues_flaws.json`):** `name` entries added
for `benediction.*` (5), `faculty.*` (3), `form.*` (2), `rank.major_figure`/
`rank.minor_official` (2, inserted into the existing `rank.*` block),
`subject.*` (7: `arithmetic`, `astronomy`, `geometry`, `grammar`, `logic`,
`music`, `rhetoric` — EN verbatim from ArMDE:7310, :7319; DE from the
SAME lines of the line-mirrored German core rulebook, since no translation
table covers these seven terms individually and D31 only overrides the
rulebook when a table row is actually in dispute). No new entries for
`being.*` (fully reused).

---

## D81.1-3 — the night audits' exclusions (`tmp/incompat-audit.md`, D81)

Three gaps the incompatibility/prerequisite sweep found, none previously
documented. Behavioral tests in `crates/arm-rules/tests/d81_exclusions.rs`.

- **Incompatible Arts excludes a Deficiency** (`flaw.incompatible_arts`,
  `ArMDE:6292`) — "may not be combined with a Deficiency (see page 125)" is a
  flat, symmetric `incompatible_with` toward `flaw.deficient_technique`
  (`ArMDE:5913-5915`) and `flaw.deficient_form` (`ArMDE:5909-5911`), added to
  all three entries (D44's own convention: every pair's `incompatible_with`
  is declared on both sides).
- **Characteristic-floor prerequisite — a new `Prereq::CharacteristicMin
  { characteristic, score }`.** `virtue.supernatural_beauty` (`ArMDE:5095`)
  and `flaw.envied_beauty` (`ArMDE:6014`): "a character lacking a positive
  Presence score may not have this Virtue/Flaw" → Presence ≥ 1.
  `flaw.uncontrollable_strength` (`ArMDE:6909`): "may not be taken if the
  character's Strength is below 0" → Strength ≥ 0. One variant covers both
  floors (score 1 or 0) rather than shipping two near-identical ones.
  Evaluated against the entity's effective score
  (`effective/characteristic.rs::effective_characteristic_score`) for every
  Characteristic. **D83.4 (Norbert 2026-10-03, amends D81.2's original
  "unset is Unknown"):** a Characteristic with no entry in
  `Entity::characteristics` is a real 0 plus its free deltas — the UI deletes
  the entry at 0, so "never set" and "0" are one state. So Supernatural Beauty
  with Presence unset is `prereq_not_met`; Uncontrollable Strength with
  Strength unset is legal, and refused under Dwarf's free -1 (`ArMDE:5998`).
  `prereq_unevaluated` no longer arises from this variant (only from an
  unresolvable id, which load-time integrity rejects). Referential integrity
  requires `characteristic` to resolve via `Characteristic::from_id`. Pinned
  by `tests/r4_prereq_score_basis.rs` and `d81_exclusions.rs`.
- **Broken Vessel — two new variants composed under `Prereq::Any`, not one
  hard-coded predicate.** `flaw.broken_vessel` (`ArMDE:5755`): "Characters may
  only take this Flaw if they have at least one Supernatural Ability or Art
  normally improved through experience points" is a disjunction over TWO
  different kinds of catalogue entry, so CLAUDE.md's data-driven rule (the
  engine must not assume which category a rule names) rules out a single
  `HasSupernaturalAbilityOrArt`-shaped variant — that was the Phase-1 draft,
  replaced before Phase 2 landed. Instead:
  - `Prereq::AbilityCategoryScoreMin { category, score }` — holds an Ability
    of the named [`AbilityCategory`] at held score ≥ `score` (bought or
    granted, never Puissant — D83.5). Unlike
    `Prereq::HasCategory`'s open `PointItem::categories` vocabulary, the
    Ability-category taxonomy is closed (five fixed variants), so load-time
    integrity requires `category` to resolve via
    `AbilityCategory::from_slug` — a new bare-slug parse (no
    `ability_category.` prefix) alongside the existing `from_id`.
  - `Prereq::AnyArtMin { score }` — holds ANY Hermetic Art at bought score
    ≥ `score` (never Puissant Art — D83.5). Every entry in the Art registry IS a Hermetic Art (no other
    kind exists in this engine), so no category filter is needed.
  - Both are static (never `Unknown`), matching `Prereq::HasCategory`'s own
    nature: an Ability/Art the entity does not hold simply scores 0.
  - `flaw.broken_vessel`'s data:
    `Any([AbilityCategoryScoreMin{category:"supernatural",score:1},
    AnyArtMin{score:1}])`.

| Value | Source |
|---|---|
| Supernatural Beauty's/Envied Beauty's Presence floor (1) | `ArMDE:5095, :6014` |
| Uncontrollable Strength's Strength floor (0) | `ArMDE:6909` |
| Broken Vessel's Supernatural-Ability/Art floor (1) | `ArMDE:5755` |

---

## Other available books

The following English sources are present in `rules/source/en/`. Add a section
above (mirroring the Core Rules layout) when mechanics from a book are implemented.

**Nothing implemented yet:**

- Ars Magica 5e - Houses of Hermes - Mystery Cults.md (HoH:MC)
- Ars Magica 5e - Houses of Hermes - Societates.md (HoH:S)
- Ars Magica 5e - Houses of Hermes - True Lineages.md (HoH:TL)
- Ars Magica 5e - Magic - Hedge Magic (Revised).md (HM:RE)
- Ars Magica 5e - Realms of Power - Faerie.md (RoP:F)
- Ars Magica 5e - Realms of Power - The Divine (Revised).md (RoP:D)
- Ars Magica 5e - Realms of Power - The Infernal.md (RoP:I)

**Cited only for what the core rules do not state:**

- Ars Magica 5e - Realms of Power - Magic.md (RoP:M) — the general Might-vs-Parma
  Magic Resistance rule (`RoP:M:1472`). Nothing else. It also restates Spirit
  Votary's **7** at `RoP:M:5486`, but that number is the unspent remainder of the
  core Flaw allowance rather than anything the core rules omit (D32), so no
  mechanic rests on it.

The four Mythic Companion types of M4/4d used to be listed here as the reason all
four *Realms of Power* volumes were cited. That was wrong provenance, not a real
dependency: the Virtues those types are built from (Devil Child, Faerie Doctor,
Nephilim, Spirit Votary, and the Demonic/Angelic/Spirit-Pact Virtues they grant
and require) are **reprinted in the core rules**, which is the source of truth, so
all eleven now cite *Ars Magica - Definitive Edition (Core Rules).md*. See the
provenance-correction note under **Mythic Companion types (M4/4d)** above for the
per-item line ranges.
