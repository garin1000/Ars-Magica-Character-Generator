// Hand-mirrored from the `arm-rules` serde types. Kept minimal: only the shapes
// that cross the Tauri boundary for Milestone 2. Codegen (ts-rs/specta) is out
// of scope here.

export type Magnitude = 'free' | 'minor' | 'major';
export type ItemKind = 'virtue' | 'flaw' | 'boon' | 'hook';
export type EntityKind = 'character' | 'covenant';
export type ValidationMode = 'enforced' | 'advisory' | 'silent';
export type IssueSeverity = 'error' | 'warning';

// A phase of character creation (`CreationPhase`). Two things speak it: the
// character type's ordered `creation_phases`, which the guided wizard walks, and
// every validation issue, which names the phase whose input surface owns the
// offending value. `review` is the wizard's terminal step — the findings no
// creation phase owns (equipment, Might, Warping) plus a last look at the whole
// character. Labels always go through the `phase-<slug>` Fluent key; a Rust test
// pins this union against `CreationPhase::ALL`.
export type CreationPhase =
  | 'concept'
  | 'type'
  | 'characteristics'
  | 'virtues_flaws'
  | 'abilities'
  | 'arts'
  | 'spells'
  | 'house_specialisation'
  | 'mythic_type'
  | 'personality_reputations'
  | 'aging'
  | 'review';

// Closed enums in the engine (`ParamType` / `ParameterDomain`), serialized as
// their snake_case names.
export type ParamType = 'ref';
export type ParameterDomain =
  | 'ability'
  | 'art'
  | 'technique'
  | 'form'
  | 'characteristic'
  | 'item'
  | 'text';

export interface ParameterDef {
  key: string;
  type: ParamType;
  domain: ParameterDomain;
}

// Mechanical effect a virtue/flaw applies. `ability_bonus` adds to an ability's
// effective score (Puissant Ability +2); `characteristic_limit` shifts a
// characteristic's buy limit (Great Characteristic +1 raises the cap, Poor
// Characteristic −1 lowers the floor). The target is named by the selection's
// `param` value.
export type Effect =
  | { type: 'ability_bonus'; param: string; amount: number }
  | { type: 'characteristic_limit'; param: string; amount: number }
  | { type: 'art_bonus'; param: string; amount: number }
  | { type: 'affinity_ability_cost'; param: string; counts_as_num: number; counts_as_den: number }
  | { type: 'affinity_art_cost'; param: string; counts_as_num: number; counts_as_den: number }
  | { type: 'restricted_ability_xp'; amount: number; abilities?: string[]; categories?: string[] }
  | {
      type: 'group_affinity_cost';
      abilities: string[];
      counts_as_num: number;
      counts_as_den: number;
    }
  | { type: 'characteristic_points'; amount: number }
  | { type: 'ability_score_grant'; ability: string; amount: number }
  | { type: 'spell_levels'; amount: number }
  | { type: 'general_xp'; amount: number }
  | { type: 'confidence_bonus'; score: number; points: number }
  | { type: 'warping_grant'; score: number; points: number }
  | { type: 'true_faith_grant'; score: number }
  | { type: 'item_level_budget'; amount: number }
  | { type: 'spell_mastery_xp'; amount: number }
  | {
      type: 'grants_spell_mastery';
      score: number;
      advancement_num?: number;
      advancement_den?: number;
    }
  | { type: 'grants_selection'; items: string[] }
  | { type: 'size_delta'; amount: number }
  | { type: 'characteristic_score_delta'; characteristic: string; amount: number }
  | { type: 'grants_reputation'; kind?: ReputationType; score: number }
  // M5/5b in-play effects (consumed by the derived-totals read-out, slice 5i).
  | { type: 'magical_focus'; param: string; major: boolean }
  | { type: 'casting_total_mod'; amount: number; scope: CastingScope }
  | { type: 'lab_total_mod'; amount: number }
  | { type: 'deficient_art'; param: string }
  | { type: 'magic_total_halving'; total: HalvableTotal }
  | { type: 'soak_mod'; amount: number }
  | { type: 'combat_mod'; amount: number; target: CombatStat }
  | { type: 'health_mod'; track: HealthTrack; amount: number }
  | { type: 'magic_resistance_mod'; kind: MagicResistanceEffect }
  | { type: 'aging_mod'; kind: AgingEffect; amount: number }
  | { type: 'advancement_mod'; source: AdvancementSource; amount: number }
  | { type: 'special_casting_mod'; kind: SpecialCasting }
  | { type: 'ability_roll_mod'; param: string; amount: number }
  // Elemental Magic (5c): creation-time Art-XP redistribution over the four
  // elemental Forms. Surfaced through the effective art bonus, not rendered raw.
  | { type: 'elemental_magic'; forms: string[] };

// M5/5b scalar enums mirroring the engine (rendered via Fluent in slice 5i).
export type CastingScope = 'all' | 'formulaic' | 'ritual' | 'formulaic_ritual' | 'spontaneous';
export type HalvableTotal =
  | 'spontaneous_casting'
  | 'lab_enchanting'
  | 'lab_longevity'
  | 'penetration'
  | 'magic_resistance';
export type CombatStat = 'initiative' | 'attack' | 'defense' | 'damage';
export type HealthTrack =
  | 'fatigue_penalty'
  | 'wound_penalty'
  | 'fatigue_roll'
  | 'casting_fatigue'
  | 'recovery';
export type MagicResistanceEffect =
  | 'no_form_bonus'
  | 'aura_bonus'
  | 'susceptible_divine'
  | 'susceptible_faerie'
  | 'susceptible_infernal';
export type AgingEffect =
  | 'aging_roll'
  | 'longevity_bonus'
  | 'no_aging'
  | 'no_apparent_aging'
  | 'decrepitude'
  | 'living_conditions'
  | 'crisis_survival'
  | 'crisis_heavy_wound';
export type AdvancementSource =
  | 'taught'
  | 'book'
  | 'vis'
  | 'practice'
  | 'adventure'
  | 'insight'
  | 'teaching'
  | 'spell_mastery'
  | 'all';
export type SpecialCasting =
  | 'quiet_words'
  | 'subtle_gestures'
  | 'deft_form'
  | 'diedne'
  | 'faerie_raised'
  | 'life_linked_spontaneous'
  | 'spell_improvisation'
  | 'mercurian'
  | 'life_boost'
  | 'circumstantial';

// The audience a Reputation reaches (a fixed rules taxonomy, rendered via Fluent
// `reputation-type-<id>`, never as a raw slug).
export type ReputationType = 'local' | 'ecclesiastical' | 'hermetic' | 'academic';

// Prerequisite expression tree. Adjacently tagged by the engine: every variant
// is a uniform object carrying a `kind` discriminant, with any payload under
// `value` (the unit variant `is_magus` has no `value`).
export type Prereq =
  | { kind: 'all'; value: Prereq[] }
  | { kind: 'any'; value: Prereq[] }
  | { kind: 'none'; value: Prereq[] }
  | { kind: 'has'; value: string }
  | { kind: 'house'; value: string }
  | { kind: 'ability_min'; value: { ability: string; score: number } }
  | { kind: 'art_min'; value: { art: string; score: number } }
  | { kind: 'is_magus' };

// How a V/F impacts a character mechanically (M5 slice 5a). Mirrors the engine's
// `Classification`. Required on every PointItem.
export type Classification = 'narrative' | 'creation_effect' | 'in_play_effect';

export interface PointItem {
  id: string;
  kind: ItemKind;
  magnitude: Magnitude;
  category: string;
  classification: Classification;
  // Descriptor "Type" tag: a Tainted (Infernal-associated) V/F. Omitted when false.
  tainted?: boolean;
  // Entity kinds this item may be selected for. Omitted when empty, which the
  // engine reads as "any kind".
  entity_kinds?: EntityKind[];
  prerequisites?: Prereq;
  // Items that may not be selected alongside this one. Symmetric (the engine
  // rejects a ruleset whose declarations are one-sided) and omitted when empty.
  incompatible_with?: string[];
  parameters?: ParameterDef[];
  effects?: Effect[];
  // Max selections per (id, params) target. Omitted when the default (1).
  max_per_target?: number;
}

// One ability-score bonus, targeting a single ability instance. A parameterized
// ability ((Area) Lore) is identified by (ability, parameter); `parameter` is
// absent for a plain ability. Mirrors the engine's `AbilityBonus`.
export interface AbilityBonus {
  ability: string;
  parameter?: string | null;
  bonus: number;
}

// One Art-score bonus (e.g. Puissant Art +3). Arts are not parameterized, so the
// target is identified by id alone. Mirrors the engine's `ArtBonus`.
export interface ArtBonus {
  art: string;
  bonus: number;
}

// The maximum learnable spell level for one Technique/Form combination
// (Te + Fo + Int + Magic Theory + 3). Mirrors the engine's `SpellLevelCap`.
export interface SpellLevelCap {
  technique: string;
  form: string;
  cap: number;
}

// Whether a demanded Ability score is one the Order enforces or one the rulebook
// merely recommends. Mirrors the engine's `AbilityRequirementKind`.
export type AbilityRequirementKind = 'required' | 'recommended';

// One row of a magus's Hermetic-minimums checklist: what is demanded, what the
// character bought, and whether that satisfies it. Mirrors the engine's
// `MagusMinimumAbility`. `score` is the BOUGHT score — a Virtue's +2 to use is not
// training the Order can examine — and `parameter` narrows the demand to one
// instance, unset throughout the shipped data.
export interface MagusMinimumAbility {
  ability: string;
  parameter?: string;
  min_score: number;
  score: number;
  met: boolean;
  requirement: AbilityRequirementKind;
}

// A free effective-score bonus to a Characteristic (Giant Blood +1 Str/Sta,
// Dwarf -1). Mirrors the engine's `CharacteristicBonus`.
export interface CharacteristicBonus {
  characteristic: Characteristic;
  bonus: number;
}

// A free starting-score floor a virtue grants to an ability (e.g. Second Sight →
// Second Sight 1). Mirrors the engine's `AbilityFloor`.
export interface AbilityFloor {
  ability: string;
  floor: number;
}

// One restricted experience pool (Educated/Warrior/Privileged) with how much of
// it the allocation consumes. Mirrors the engine's `RestrictedXpPool`. Eligible
// by ability id OR ability category; empty arrays are omitted by the engine.
export interface RestrictedXpPool {
  amount: number;
  used: number;
  abilities?: string[];
  categories?: string[];
  // Where the pool came from, so the bar can name it. A V/F grant is labelled with
  // the item's own localized name; a life-stage block through `xp-pool-<block>`,
  // since a life stage is not an item and has no i18n entry.
  origin: XpPoolOrigin;
}

// A block of life-stage experience that funds purchases on its own terms
// (`LifeStageBlock`). Apprenticeship is absent on purpose: whichever block funds
// anything the character may learn is the general pool and needs no slug, and for a
// magus that is apprenticeship. Later life is here because for a magus it is
// restricted — Abilities only, never an Art.
export type LifeStageBlock = 'childhood_native_language' | 'childhood_spread' | 'later_life';

export type XpPoolOrigin =
  | { kind: 'item'; item: string }
  | { kind: 'life_stage'; block: LifeStageBlock };

// The experience a character's life stages earn, block by block (`LifeStageBudget`).
// Derived, never stored: `LifeStagePlan` holds the choices and this follows from
// them plus the age.
export interface LifeStageBudget {
  childhood_native_xp: number;
  childhood_spread_xp: number;
  later_life_years: number;
  later_life_rate: number;
  later_life_xp: number;
  // Years of apprenticeship served (15 for a magus, 0 for anyone else).
  apprenticeship_years: number;
  // Experience from apprenticeship (240 for a magus, 0 for anyone else) — the base
  // of the general pool, which may buy Arts as well as Abilities. The pool actually
  // funded from is `EffectiveScores.xp_general_pool` (this plus Skilled/Weak Parens).
  apprenticeship_xp: number;
  // The age the character was gauntleted at — its own age while it stands at its
  // Gauntlet, and for anyone who serves no apprenticeship.
  gauntlet_age: number;
  // Years lived after the Gauntlet (age - gauntlet_age), 0 for everyone else.
  post_gauntlet_years: number;
  // What those years grant, lab work deducted (30 per year, -10 per charged lab
  // season). Points, not experience: each is an experience point OR a spell level,
  // so this is the sum of the two fields below and funds nothing on its own.
  post_gauntlet_points: number;
  // How many of those points the player took as levels of spells.
  post_gauntlet_spell_levels: number;
  // The rest of them, which are experience.
  post_gauntlet_xp: number;
}

// A character's life-stage choices — never its resolved numbers (see
// `LifeStageBudget`). Its presence switches Ability funding from the typed
// `xp_pool` to the derived life-stage blocks.
export interface LifeStagePlan {
  native_language?: string;
  // The Sample Childhood package the player took — a record of the decision, not
  // something derived from: the Abilities it grants live in `ability_scores` as
  // ordinary bought rows. Omitted for a childhood divided by hand.
  childhood_package?: string;
  // The age the magus was gauntleted at — the one post-Gauntlet number stored, with
  // every other figure derived from it. Omitted means the character stands AT its
  // Gauntlet, which is how a magus was built before the field existed.
  gauntlet_age?: number;
  // Lab seasons charged against the yearly 30 points, totalled across every
  // post-Gauntlet year (-10 each, and the deduction stops at the third season of any
  // year, so a fourth is free). Omitted when zero.
  post_gauntlet_lab_seasons?: number;
  // How many of the post-Gauntlet points the player took as levels of spells rather
  // than experience; the rest are experience. Omitted when zero.
  post_gauntlet_spell_levels?: number;
}

// One row of the Living Conditions table (`rules/core/aging.json`): how a
// character's circumstances modify the aging total. A HIGHER modifier means a
// longer life, because the total subtracts it. Its display name lives in
// `rules/i18n/<lang>/aging.json`, keyed by this id — never render the id.
export interface LivingCondition {
  id: string;
  modifier: number;
  // "Modifiers marked with an asterisk are cumulative with each other" — the
  // unstarred rows are alternatives. Absent (rather than false) for a plain row.
  cumulative?: boolean;
}

// One row of the Aging Roll table: a band of totals, inclusive on both ends, and
// what landing in it costs. `max` is absent for the open-ended top row.
export interface AgingRow {
  min: number;
  max?: number | null;
  effect: AgingRowEffect;
}

export type AgingRowEffect =
  | { type: 'any_characteristic'; points: number }
  | { type: 'named_characteristics'; points: number; characteristics: Characteristic[] }
  | { type: 'next_decrepitude_level_and_crisis' };

// One row of the Crisis Table: a band of CRISIS TOTALs, inclusive on both ends,
// and what landing in it costs. `min` is absent for the open-ended bottom row and
// `max` for the open-ended top one, so every total lands somewhere. Its display
// name lives in `rules/i18n/<lang>/aging.json`, keyed by this id.
export interface CrisisRow {
  id: string;
  min?: number | null;
  max?: number | null;
  outcome: CrisisOutcome;
}

// The Crisis Table as the ruleset ships it, plus the die it is rolled on
// ("a zero counts as ten") and the doctor the rules permit to attend. All three
// are data, so a UI can offer the legal die and name the attendant's Ability
// without hardcoding either.
export interface CrisisRules {
  rows: CrisisRow[];
  die?: { min: number; max: number } | null;
  attendant?: {
    ability: string;
    characteristic: Characteristic;
    ease_factor: number;
    botch_penalty: number;
  } | null;
}

// The aging tables as the ruleset ships them. Absent for a ruleset with no aging
// file, which stands the whole subsystem down.
export interface AgingRules {
  start_age: number;
  age_divisor: number;
  apparent_age_increase_min: number;
  longevity_clamp?: { max_total: number; until_age: number } | null;
  living_conditions: LivingCondition[];
  outcomes: AgingRow[];
  // Absent for an aging block that ships no Crisis Table, which stands the Crisis
  // down exactly as an absent aging block stands the whole subsystem down.
  crisis?: CrisisRules | null;
}

// One year of a character's aging schedule: the age the roll is owed at, the
// calendar year it falls in (absent without a birth year) and whether the aging
// log already records it.
export interface AgingScheduleYear {
  age: number;
  year?: number | null;
  recorded: boolean;
}

// The die-independent half of a character's aging. Every field is a pure function
// of the character and the rules, which is why it rides on the always-recomputed
// effective scores: the stress die is the player's and never touches the entity,
// so the roll itself is a separate command.
export interface AgingReadout {
  // The first age owing a roll (36) and the age aging begins after (35) — read
  // out of the rules, never printed as a literal.
  first_roll_age: number;
  begins_after_age: number;
  schedule: AgingScheduleYear[];
  // schedule.length and how many of those years are already recorded, so the UI
  // never counts a rules-defined set itself.
  rolls_owed: number;
  rolls_recorded: number;
  // ceil(age/10) at the ACTUAL age; 0 when no age is entered.
  age_modifier: number;
  // The two modifiers the AGING TOTAL subtracts (a high one means a longer life).
  living_conditions_modifier: number;
  longevity_modifier: number;
  // Whether a Longevity Ritual's under-35 clamp stands over this character — a
  // standing predicate, unlike AgingTotal.capped_by_longevity, which says a
  // particular roll was cut down.
  longevity_clamp_active: boolean;
  // The whole non-die half of the total, so the UI adds only what the player typed.
  fixed_total: number;
}

// Score effects for the current entity, computed by the engine. Ability bonuses
// are per-instance (only non-zero ones present). Art bonuses are per-Art (only
// non-zero ones present). Characteristic caps/floors are the per-characteristic
// buy limits (Great/Poor Characteristic widen them), present for all eight. The
// XP fields are the authoritative spend after Affinity and restricted-pool
// allocation — the UI must not recompute spend without them.
export interface EffectiveScores {
  ability_bonuses: AbilityBonus[];
  art_bonuses: ArtBonus[];
  characteristic_caps: Partial<Record<Characteristic, number>>;
  characteristic_floors: Partial<Record<Characteristic, number>>;
  /** Engine-authoritative point-buy cost of the Characteristics, gains netted
   *  against spends. Never recompute this here — a second copy of the point-buy
   *  table in `derive.ts` was audit finding VA1. Optional only because the payload
   *  is absent until the first `effectiveScores` call returns. */
  characteristic_points_used?: number;
  xp_total_demand: number;
  xp_general_used: number;
  // The most demand the pools can fund; equals xp_total_demand iff legal, so
  // xp_total_demand - xp_max_flow is the overspend. Required to show a negative
  // Available: xp_general_used is a flow capped by the pool, so pool -
  // xp_general_used can never go negative however far the spend overshoots.
  xp_max_flow: number;
  // The general pool itself: the typed `xp_pool` for a directly-entered character,
  // and for one built through its life stages the block that may fund anything —
  // apprenticeship plus the years past the Gauntlet for a magus, later life for
  // anyone else — plus the Skilled/Weak
  // Parens adjustment. Engine-authoritative, because no stored field holds it.
  xp_general_pool: number;
  // The signed Virtue/Flaw contribution folded into `xp_general_pool` (Skilled
  // Parens +60, Weak Parens -60), reported on its own so the bar can name it
  // beside the base the player typed instead of leaving the two unexplained.
  xp_general_bonus: number;
  restricted_xp_pools: RestrictedXpPool[];
  // The life-stage experience blocks, or null for a directly-entered character
  // (where `xp_pool` is the authority).
  life_stage: LifeStageBudget | null;
  // The die-independent half of the character's aging, or null when the ruleset
  // ships no aging rules at all.
  aging: AgingReadout | null;
  characteristic_points_granted: number;
  ability_score_floors: AbilityFloor[];
  // Derived Size (base 0; Large +1, Giant Blood +2, Small Frame -1, Dwarf -2).
  size: number;
  // Free effective-score bonuses to Characteristics (Giant Blood +1 Str/Sta, Dwarf -1).
  characteristic_bonuses: CharacteristicBonus[];
  // Effective Characteristic score after aging drops AND free virtue deltas, keyed
  // by characteristic — only the entries that differ from the bought score (the UI
  // falls back to the bought score for the rest; the engine owns the floor clamp).
  characteristic_effective: Partial<Record<Characteristic, number>>;
  // Aging-drop count per characteristic (only non-zero entries), for the tooltip.
  characteristic_aging_drops: Partial<Record<Characteristic, number>>;
  // Virtue/Flaw Selections the entity's House grants (derived, never persisted),
  // in the House's declared grant order, so the V/F view renders them read-only.
  granted_selections: Selection[];
  // Effective virtue/flaw point ceilings (base budget + Mythic Companion type
  // bonus), so the balance bar shows the true budget (Devil Child 37/17).
  virtue_budget: number;
  flaw_budget: number;
  // The magus's effective spell-levels budget (base + Skilled/Weak Parens + the
  // levels its post-Gauntlet years bought) and how many levels the chosen spells
  // consume — the spell bar. The base is the per-character spell_levels_override
  // when set, else the type profile's base.
  spell_levels_budget: number;
  // The type profile's base spell-levels budget (120 for a magus), so the
  // override field's placeholder shows the data-driven default (never a literal).
  spell_levels_profile_base: number;
  // The V/F contribution alone (Skilled Parens +30, Weak Parens -30; signed, 0
  // when none). The budget's three parts — profile base, this, and the life-stage
  // levels below — are surfaced separately because they come from three different
  // rules and the bar labels each, the way the XP bar lists its extra pools; the
  // identity is base + bonus + life_stage === spell_levels_budget.
  spell_levels_bonus: number;
  // The levels of spells the magus's years past its Gauntlet bought — its chosen
  // slice of the fungible 30 points a year, added ON TOP of the profile base (which
  // is apprenticeship's 120). 0 for a magus standing at its Gauntlet.
  spell_levels_life_stage: number;
  spell_levels_used: number;
  // Per-Technique/Form maximum learnable spell level (Te + Fo + Int + Magic
  // Theory + 3), so the picker greys a spell above the magus's cap. Empty for a
  // non-magus. Engine-authoritative; the UI only reads it, never recomputes it.
  spell_level_caps: SpellLevelCap[];
  // The Hermetic minimum-Ability checklist: what the Order demands (Core:2437) and
  // what the rulebook recommends (Core:2451-2461), each with the character's bought
  // score and whether it suffices. Empty for a non-magus, exactly like
  // `spell_level_caps` — admission to the Order is a magus's concern alone.
  // Engine-authoritative: the same reading the magus_minimum_ability /
  // magus_recommended_ability findings come from.
  magus_minimum_abilities: MagusMinimumAbility[];
  // Derived Confidence (type default + V/F); 0/0 for grogs.
  confidence_score: number;
  confidence_points: number;
  // The Gift's free Supernatural-Ability slots (1 for a Gifted non-magus, else 0)
  // and how many are used, so the ability picker greys unavailable ones.
  supernatural_free_total: number;
  supernatural_free_used: number;
  // The character's age → max-Ability-score cap (base, pre-Affinity). Null when
  // age is unset.
  age_ability_cap?: number | null;
  // Reputation grants the character's V/F confer (kind + score), so the UI only
  // offers a Reputation add-control when one exists.
  reputation_grants: ReputationGrant[];
  // Derived Warping: the unified total of stored Warping Points plus any granted
  // by V/F, with the score derived by inverting the advancement curve (15 points
  // 2). 0/0 when there is no Warping. Engine-authoritative; never recomputed here.
  warping_score: number;
  warping_points: number;
  // The off-budget Virtues/Flaws a non-magus owes from its Warping Score
  // ("Effects of Warping", Core:16547-16561): per-kind owed counts for the
  // read-out. All zero for magi (exempt — Twilight instead) and characters owing
  // nothing. Engine-authoritative.
  warping_owed: WarpingOwed;
  // One OPEN grant per owed warping slot (stable choice_key + the constraint its
  // fill must satisfy), so the UI renders one picker per slot. Empty for magi and
  // characters owing nothing.
  warping_owed_grants: Grant[];
  // Derived Decrepitude Score: the sum of accrued aging points across all
  // Characteristics inverted through the advancement curve (17 points 2); 0 when
  // there are no aging points. Engine-authoritative; never recomputed here.
  decrepitude_score: number;
  // Derived True Faith Score granted by V/F (True Faith 1); 0 when none.
  true_faith_score: number;
  // Derived starting enchanted-device level budget (Magic Items +25, Redcap 50);
  // 0 when none.
  item_level_budget: number;
  // Total device level the entity's devices consume — the "used" side of the
  // item-level budget bar (engine-authoritative; never recomputed in JS).
  item_level_used: number;
  // Derived Spell-Mastery XP pool (Mastered Spells +50 each); 0 when none.
  spell_mastery_xp: number;
  // Mastery-score floor every known spell gets (Flawless Magic 1); 0 = none.
  spell_mastery_floor: number;
  // Whether a Virtue doubles Spell-Mastery Advancement Totals (Flawless Magic),
  // halving each mastery point's XP cost, so the mastery accounting matches the
  // engine's charge.
  spell_mastery_advancement_doubled: boolean;
  // The being's effective Might Score + Realm (base + same-Realm grants), or null
  // for an ordinary character. Engine-authoritative; never recomputed in JS.
  might: MightScore | null;
  // Derived power-levels budget the being's Might Virtues grant; 0 when none.
  power_levels_budget: number;
  // Total power level the being's powers consume — the "used" side of the bar.
  power_levels_used: number;
}

// --- Derived play-stat totals (M5/5i), mirrored from `arm_rules::derived`.
// Read-only: the panel renders these numbers and computes no mechanics itself.
// Every `label`/`level`/`family`/`detail` is a stable slug mapped through a
// Fluent `derived-*` key — never rendered raw.

// A labelled term in a breakdown.
export interface Addend {
  label: string;
  value: number;
}

// A Lab Total for one (Technique, Form) grid cell.
export interface LabTotal {
  technique: string;
  form: string;
  addends: Addend[];
  total: number;
  within_focus?: number | null;
  deficient: boolean;
  // `total` (Deficient-halved), halved again for Weak Enchanter — the figure
  // to use when creating or investigating an enchanted item. Equal to `total`
  // for anyone without the Flaw. Mirrored from the Rust `LabTotal.enchanting`
  // field (`crates/arm-rules/src/derived/lab.rs`).
  enchanting: number;
}

// The within-focus counterparts of a CastingTotal's four cast types.
export interface CastingWithinFocus {
  focus_art: number;
  formulaic: number;
  ritual: number;
  spontaneous_fatiguing: number;
  spontaneous_non_fatiguing: number;
}

// The non-standard-casting (silent / still) variants of a cell's Formulaic total.
export interface NonStandardCasting {
  voice_penalty: number;
  gesture_penalty: number;
  silent: number;
  still: number;
  silent_and_still: number;
  deft_form: boolean;
}

// A Casting Total for one (Technique, Form) cell, split into four cast types.
export interface CastingTotal {
  technique: string;
  form: string;
  addends: Addend[];
  ritual_addends: Addend[];
  formulaic: number;
  ritual: number;
  spontaneous_fatiguing: number;
  spontaneous_non_fatiguing: number;
  within_focus?: CastingWithinFocus | null;
  non_standard: NonStandardCasting;
  deficient: boolean;
}

// A per-known-spell Penetration line.
export interface PenetrationLine {
  spell: string;
  // Chosen parameter of a parameterized spell (the target (Form)), disambiguating
  // two instances of one spell id. Absent for ordinary spells.
  parameter?: string | null;
  level: number;
  casting_total: number;
  penetration_ability: number;
  total: number;
  within_focus?: number | null;
  weak_magic: boolean;
}

// A per-Form Magic Resistance line.
export interface MagicResistance {
  form: string;
  addends: Addend[];
  total: number;
}

// The Encumbrance read-out.
export interface EncumbranceTotal {
  load: number;
  burden: number;
  total: number;
}

// One way of wielding an equipped weapon. A one-handed weapon carried with a shield
// yields two lines: one with the shield modifiers folded in (`shields` naming them)
// and one bare (`shields` absent).
export interface CombatLine {
  weapon: string;
  shields?: string[];
  ability: string;
  initiative: number;
  attack?: number | null;
  defense: number;
  damage?: number | null;
  range?: number | null;
}

// The Soak read-out.
export interface SoakTotal {
  addends: Addend[];
  total: number;
}

// A Fatigue level and its penalty.
export interface FatigueLevel {
  level: string;
  penalty: number;
}

// A wound band with its inclusive range and per-wound penalty.
export interface WoundRange {
  level: string;
  min: number;
  max?: number | null;
  penalty?: number | null;
}

// What a Longevity Ritual made *today* would be worth — a suggestion beside the
// entered-bonus input, never the stored value. Self-made rituals only.
export interface LongevityHint {
  lab_total: number;
  suggested_bonus: number;
  halved: boolean;
}

// The Longevity Ritual read-out. `bonus` is the stored, player-entered value for
// both sources; `entered` false means it is a placeholder 0, not a claim.
export interface LongevityBonus {
  source: LongevitySource;
  bonus: number;
  entered: boolean;
  bronze_cord: number;
  hint?: LongevityHint | null;
}

// The Masterpiece lesser enchanted item cap (best Lab Total / 2).
export interface MasterpieceCap {
  technique: string;
  form: string;
  lab_total: number;
  cap: number;
}

// The talisman's enchantment capacity in pawns of Vim vis: the magus's highest
// Technique + highest Form. Both contributing scores travel with the sum so the
// panel renders the derivation without arithmetic in JS.
export interface TalismanCapacity {
  technique: string;
  form: string;
  technique_score: number;
  form_score: number;
  pawns: number;
}

// The magus's side of the familiar bonding season: the best bonding Lab Total and
// how it compares with what the bond needs. `lab_total_within_focus` is present
// only when the magus holds a Magical Focus, and is CONDITIONAL — whether this
// familiar falls inside the focus's narrow field is a troupe judgment.
export interface FamiliarBinding {
  technique: string;
  form: string;
  lab_total: number;
  lab_total_within_focus?: number | null;
  lab_total_reaches_level: boolean;
  cord_points_within_lab_total: boolean;
}

// The familiar bonding read-out. Read-only guidance: no validation issue is ever
// raised from any of it, and `invested_power_levels` is compared against no budget
// (there is no limit on powers invested in a familiar), so no budget bar is drawn.
export interface FamiliarReadout {
  binding_level: number;
  cord_points_spent: number;
  invested_power_levels: number;
  binding: FamiliarBinding;
}

// A surfaced-only modifier (listed, not simulated).
export interface SurfacedModifier {
  family: string;
  detail: string;
  amount: number;
}

// The full read-only play-stat read-out returned by the `derived_totals` command.
export interface DerivedTotals {
  is_magus: boolean;
  lab_totals: LabTotal[];
  casting_totals: CastingTotal[];
  penetration: PenetrationLine[];
  magic_resistance: MagicResistance[];
  longevity?: LongevityBonus | null;
  masterpiece?: MasterpieceCap | null;
  talisman_capacity?: TalismanCapacity | null;
  familiar?: FamiliarReadout | null;
  combat: CombatLine[];
  soak: SoakTotal;
  encumbrance: EncumbranceTotal;
  fatigue: FatigueLevel[];
  wounds: WoundRange[];
  size: number;
  decrepitude_score: number;
  warping_score: number;
  warping_points: number;
  surfaced_modifiers: SurfacedModifier[];
}

export interface ReputationGrant {
  kind: ReputationType;
  score: number;
}

// Per-category flaw count cap. The category is data, so the engine hardcodes no
// slug; `major_only`/`hard` default to false and are omitted from JSON then.
// Shared by flaw and virtue category caps; the item kind counted is fixed by
// which budget list the cap lives in.
export interface CategoryCap {
  category: string;
  max: number;
  major_only?: boolean;
  hard?: boolean;
}

export interface PointBudget {
  virtue_points: number;
  flaw_points: number;
  // How many virtue points each flaw point funds. Omitted from JSON when 1 (the
  // common case), so optional here; Mythic Companions carry 2.
  virtue_points_per_flaw_point?: number;
  max_major_virtues?: number | null;
  max_major_flaws?: number | null;
  max_minor_flaws?: number | null;
  flaw_category_caps?: CategoryCap[];
  virtue_category_caps?: CategoryCap[];
}

export interface EntityTypeProfile {
  id: string;
  budget: PointBudget;
  // Category allow/deny lists. Both are omitted when empty (the magus profile
  // forbids nothing, so it carries no `forbidden_categories` key); an absent
  // permit list means "no restriction".
  permitted_categories?: string[];
  forbidden_categories?: string[];
  // Item ids that must / may never be selected. Omitted from JSON when empty.
  required_traits?: string[];
  forbidden_traits?: string[];
  // Whether this character type is a Hermetic magus. Omitted from JSON when
  // false (the common case), so optional here.
  is_magus?: boolean;
  // Whether this type chooses a Mythic Companion type (free status/Minor Virtue
  // + required package). Capability flag parallel to is_magus. Omitted when false.
  has_mythic_type?: boolean;
  // The magus's starting spell-levels budget (120). Omitted from JSON when 0
  // (every non-magus type), so optional here.
  spell_levels?: number;
  // Starting Confidence Score / Points (companions/magi/mythic = 1/3, grog 0).
  // Omitted from JSON when 0.
  confidence_score?: number;
  confidence_points?: number;
  // The Gift policy and the id representing The Gift. Omitted when not applicable.
  gift_policy?: 'required' | 'allowed' | 'forbidden';
  gift_id?: string;
  // Ordered: the guided wizard walks these in sequence. Typed in the engine too,
  // so a phase string it has no variant for fails the ruleset load.
  creation_phases: CreationPhase[];
}

export interface I18nEntry {
  name: string;
  summary?: string | null;
  description?: string | null;
  // Two-letter Art abbreviation (e.g. "Cr"); present only for Arts.
  abbreviation?: string | null;
  // Example specialties (e.g. an Ability's example specializations). Omitted when empty.
  specialties?: string[];
}

// The eight Characteristics, serialized as their snake_case names.
export type Characteristic = 'int' | 'per' | 'str' | 'sta' | 'pre' | 'com' | 'dex' | 'qik';

// Canonical Characteristic order (matches the engine's `Characteristic::ALL`).
export const CHARACTERISTICS: Characteristic[] = [
  'int',
  'per',
  'str',
  'sta',
  'pre',
  'com',
  'dex',
  'qik',
];

export type AbilityCategory = 'general' | 'academic' | 'arcane' | 'martial' | 'supernatural';

export interface Ability {
  id: string;
  category: AbilityCategory;
  // For a parameterized ability ((Area) Lore, (Living Language), …): the key of
  // the player-supplied value, naming the {key} placeholder in the localized name
  // and the `param-label-<key>` Fluent label. Absent for plain abilities.
  parameter?: string | null;
  // "Asterisked" in the rulebook: cannot be used without at least one experience
  // point in it (no untrained roll). Independent of category; drives the trailing
  // `*` marker. Absent/false for abilities usable untrained.
  requires_training?: boolean;
}

// One row of the Ability XP advancement table ("ABILITY To Buy" column).
export interface AbilityXpRow {
  score: number;
  total_xp: number;
}

// The two classes of Hermetic Art, serialized as their snake_case names.
export type ArtType = 'technique' | 'form';

export interface Art {
  id: string;
  art_type: ArtType;
}

// A spell in the catalogue. `level` is a fixed number, or omitted for a General
// spell (learned at a per-character level). Technique/Form are Art ids.
export interface Spell {
  id: string;
  technique: string;
  form: string;
  level?: number | null;
  requisites?: string[];
  // A Ritual spell must be learned at level >= 20 (its minimum learnable level).
  // Present (true) only for rituals; absent = ordinary spell (minimum level 1).
  ritual?: boolean;
  // Selection parameters this spell requires (a meta-magic Vim spell whose target
  // (Form) is a selection declares a single `form`-domain parameter). Empty/absent
  // for ordinary spells. The chosen value is display + identity only and does NOT
  // change the spell's own Technique/Form.
  parameters?: ParameterDef[];
}

// A choosable Spell Mastery special ability from the catalogue. Name/description
// live in the i18n map, keyed by `id`.
export interface SpellMasteryAbility {
  id: string;
  // Whether this ability may be taken multiple times for the same spell (Precise,
  // Quick, Quiet Casting). Absent = false (once per spell).
  repeatable?: boolean;
}

// A spell the character knows. `level` is set only for a General spell (the
// chosen level); for a fixed spell the catalogue level is authoritative.
export interface SpellSelection {
  spell: string;
  level?: number | null;
  // Bought Spell Mastery Ability score (spent from the mastery-XP pool);
  // omitted/0 = unmastered. Effective mastery = max(this, mastery floor).
  mastery?: number | null;
  // Chosen value for a parameterized spell (the target (Form) of a meta-magic Vim
  // spell, an Art id like `art.ignem`). Part of the spell's identity: the same base
  // spell may be taken once per distinct parameter. Absent for ordinary spells.
  parameter?: string | null;
  // Chosen Spell Mastery special abilities for this spell, each a
  // `spell_mastery_ability.*` id. One may be chosen per effective mastery level; a
  // repeatable ability (Precise/Quick/Quiet Casting) may appear more than once, so
  // this may hold duplicates. Absent/empty when none are chosen.
  mastery_abilities?: string[];
}

// A named Personality Trait with a value in ±3 (±6 for a Major Personality Flaw).
export interface PersonalityTrait {
  name: string;
  value: number;
}

// A starting Reputation (only legal when a V/F grants one).
export interface Reputation {
  kind: ReputationType;
  score: number;
  content: string;
}

// A starting enchanted device (magi). `level` is charged against the item-level
// budget the character's Magic Items / Redcap Virtues grant.
export interface EnchantedDevice {
  name: string;
  level: number;
}

// The four supernatural Realms a being's Might can be aligned to.
export type Realm = 'magic' | 'faerie' | 'divine' | 'infernal';

// Canonical Realm order (matches the engine's `Realm` enum declaration order). A
// fixed rules taxonomy, so it lives here beside CHARACTERISTICS rather than being
// re-hardcoded per component; always rendered through Fluent (`realm-<id>`),
// never as a raw slug.
export const REALMS: Realm[] = ['magic', 'faerie', 'divine', 'infernal'];

// A supernatural being's base Might Score + Realm (Virtue grants add on top).
export interface MightScore {
  realm: Realm;
  score: number;
}

// A supernatural power a Might-being holds. `level` is charged against the
// power-levels budget the being's Might Virtues grant (like a device vs item level).
export interface SupernaturalPower {
  name: string;
  level: number;
}

// A magus's familiar: the magical beast itself plus the three bond cords. Field
// order follows the rulebook's Creature Format, so a save reads like a statblock.
// Everything here is the FAMILIAR's own — its Characteristics are not bought from
// the magus's Characteristic points, and `might` is not the character's `might`.
// Every field but `name` is omitted from JSON at its default.
export interface Familiar {
  name: string;
  // The kind of beast, free text ("raven"). Not `species` — the rules reserve that
  // word for the Imaginem term.
  animal?: string;
  // The familiar's own Magic Might + Realm; null/absent when not entered.
  might?: MightScore | null;
  // The familiar's eight Characteristics, signed. A 0 score is absent.
  characteristics?: Partial<Record<Characteristic, number>>;
  // The creature's Size — signed, and commonly negative (a raven is -4).
  size?: number;
  personality_traits?: PersonalityTrait[];
  cord_gold?: number;
  cord_silver?: number;
  cord_bronze?: number;
  // Powers invested in the bond. Charged against NO budget, unlike the character's
  // own `powers` — there is no limit on powers invested in a familiar.
  powers?: SupernaturalPower[];
}

// A talisman attunement: a free-text descriptor and the bonus it confers. Only
// the highest applicable bonus applies, and only to Casting Scores.
export interface TalismanAttunement {
  description: string;
  bonus: number;
}

// An effect instilled in a talisman. Deliberately not an EnchantedDevice: a
// talisman's effects are charged against no budget, while a device's level is
// charged against the item-level budget its Virtues grant.
export interface TalismanEffect {
  name: string;
  level: number;
}

// A magus's talisman: his personal enchanted item. `description` is its
// shape/material identity (free text). Its capacity is derived, never stored
// (see TalismanCapacity). Both lists are omitted from JSON when empty.
export interface Talisman {
  description?: string;
  attunements?: TalismanAttunement[];
  effects?: TalismanEffect[];
}

// A Twilight Scar: a minor magical trait a magus acquires from Twilight
// (free-text; no mechanical number).
export interface TwilightScar {
  description: string;
}

// One year of a character's aging log. A resolved year carries the whole roll —
// the die the player typed, the total it made, the conditions in force and the
// points awarded — which is what lets the engine undo the year exactly. A
// hand-written entry carries only `effect`, which stays authoritative for it.
//
// `year` is the CALENDAR year and is absent for a character with no birth year
// (and for a hand-written entry naming none), so every reader must handle its
// absence rather than render it. `year` is first so the engine sorts the log
// chronologically; undated entries sort first.
export interface AgingLogEntry {
  year?: number | null;
  age?: number | null;
  effect: string;
  die?: number | null;
  total?: number | null;
  living_conditions?: string[];
  points?: Partial<Record<Characteristic, number>>;
  apparent_age_increased?: boolean;
  // Whether the row DEMANDED a Crisis. A `true` with no `crisis_row` is a Crisis
  // owed and not yet rolled.
  crisis?: boolean;
  // What the Crisis Table was asked and what it answered, once the Crisis is
  // rolled: the player's Simple Die, the CRISIS TOTAL it made, the row id (whose
  // text lives in the rules i18n, never here) and that row's severity. A row with
  // no severity is a bedridden one — time, not an illness.
  crisis_die?: number | null;
  crisis_total?: number | null;
  crisis_row?: string | null;
  crisis_severity?: CrisisSeverity | null;
}

// How bad a crisis illness is, ascending. Mirrors the Rust `CrisisSeverity`;
// rendered through Fluent, never as a raw slug.
export type CrisisSeverity = 'minor' | 'serious' | 'major' | 'critical' | 'terminal';

// What one row of the Crisis Table does to the character. Bedridden is time
// rather than a roll, so it carries no numbers at all; `ease_factor` is absent
// for the Terminal row, which offers NO Stamina roll — not an unbeatable one.
export type CrisisOutcome =
  | { type: 'bedridden' }
  | {
      type: 'illness';
      severity: CrisisSeverity;
      ease_factor?: number | null;
      ritual_level: number;
    };

// Where a Longevity Ritual comes from (rendered via Fluent, never as a raw slug).
export type LongevitySource = 'self_made' | 'external';

// A magus's Longevity Ritual: a stored record of a past event. `bonus` is
// player-entered for BOTH sources — the number was fixed by the Lab Total of the
// season the ritual was made, so nothing here is derived; null means "not entered
// yet", never a claimed 0. `focus` is the ritual's culminating focus (free text).
export interface LongevityRitual {
  source: LongevitySource;
  bonus?: number | null;
  focus?: string;
}

// Which weapon table a Weapon comes from (rendered via Fluent `weapon-kind-<id>`,
// never as a raw slug). Thrown and missile weapons carry a Range.
export type WeaponKind = 'melee' | 'missile' | 'thrown';

// A catalogue weapon. Attack/Damage/min-Strength are absent for the n/a cells
// (Dodge has no attack/damage; body attacks have no min-Strength). All modifiers
// are signed; combat totals are computed downstream (slice 5i), never in JS.
export interface Weapon {
  id: string;
  kind: WeaponKind;
  init_mod: number;
  attack_mod?: number | null;
  defense_mod: number;
  damage_mod?: number | null;
  min_strength?: number | null;
  load: number;
  range?: number | null;
  ability: string;
}

// A catalogue shield. Its modifiers add to the wielded weapon's line (slice 5i).
export interface Shield {
  id: string;
  init_mod: number;
  attack_mod: number;
  defense_mod: number;
  load: number;
  min_strength: number;
}

// A catalogue armor row (one per material and coverage). Protection is the Soak
// bonus; Load feeds Encumbrance (slice 5i).
export interface Armor {
  id: string;
  protection: number;
  load: number;
}

// A piece of equipment the character carries: a reference to a catalogue weapon,
// shield, or armor id, plus whether it is currently equipped (wielded/worn).
export interface EquipmentSlot {
  item: string;
  equipped?: boolean;
  // Whether this weapon's combat Ability specialization applies to it, granting
  // +1 to the weapon's Attack and Defense (Core:7122, :7139). Additive/optional.
  specialization_applies?: boolean;
}

export interface CharacteristicCost {
  score: number;
  cost: number;
}

export interface CharacteristicRules {
  start_points: number;
  costs: CharacteristicCost[];
  // The no-virtue buy cap/floor (±3). The spinner falls back to these until the
  // engine's entity-dependent caps/floors (which Great/Poor Characteristic
  // widen) arrive. Omitted by rulesets that don't distinguish them.
  base_max?: number | null;
  base_min?: number | null;
}

// The three structural classes of Hermetic House, serialized as their
// snake_case names. Flavor/grouping only — drives no mechanics.
export type LineageType = 'true_lineage' | 'mystery_cult' | 'societas';

// Constraint on an open, player-chosen House grant (Jerbiton's free Minor
// Virtue, Ex Miscellanea's Major non-Hermetic Virtue). Declarative; empty
// category lists are omitted by the engine. Mirrors the engine's `GrantConstraint`.
export interface GrantConstraint {
  kind: ItemKind;
  magnitude?: Magnitude;
  require_categories?: string[];
  forbid_categories?: string[];
}

// The off-budget Virtues/Flaws a non-magus character owes from its Warping Score
// ("Effects of Warping", Core:16547-16561). Mirrors the engine's `WarpingOwed`.
export interface WarpingOwed {
  minor_flaws: number;
  minor_supernatural_virtues: number;
  major_flaws: number;
}

// One thing a type-linked profile (House or Mythic Companion type) grants at
// creation, internally tagged on `kind`. `fixed` gives a set Virtue (with any
// fixed params); `choice` offers a menu of Selections keyed by `choice_key`;
// `open` is a player-chosen Virtue/Flaw the `constraint` bounds. Mirrors the
// engine's `Grant`.
export type Grant =
  | { kind: 'fixed'; item: string; params?: Record<string, string> }
  | { kind: 'choice'; choice_key: string; options: Selection[] }
  | { kind: 'open'; choice_key: string; constraint: GrantConstraint };

// A Hermetic House. Display name/description live in the rules i18n map, keyed
// by `id` (like Arts) — not in Fluent. Mirrors the engine's `House` (the
// provenance `source` field is not surfaced to the UI).
export interface House {
  id: string;
  lineage_type: LineageType;
  grants?: Grant[];
}

// A required Flaw a Mythic Companion type imposes: the rules-specified `default`
// plus the `constraint` a "suitable substitute agreed with the troupe" must
// satisfy. Mirrors the engine's `RequiredFlaw`.
export interface RequiredFlaw {
  default: Selection;
  constraint: GrantConstraint;
}

// A Mythic Companion type (Devil Child, Faerie Doctor, Nephilim, Spirit Votary).
// `grants` are point-free (free status + free Minor Virtue); `required_virtues`
// and each `required_flaws[].default` count against the budget. Mirrors the
// engine's `MythicCompanionType`; name/description live in the rules i18n map.
export interface MythicCompanionType {
  id: string;
  grants?: Grant[];
  required_virtues?: Selection[];
  required_flaws?: RequiredFlaw[];
  bonus_flaw_points?: number;
  bonus_free_virtue_points?: number;
}

// The life-stage experience rules: the blocks a character's Abilities are bought
// from before play (`rules/core/life_stages.json`). Mirrors the engine's
// `LifeStageRules`.
export interface LifeStageRules {
  // The magus's apprenticeship, when the ruleset ships one (a ruleset with no
  // Hermetic magi ships none, so the block is optional).
  apprenticeship?: ApprenticeshipRules;
  childhood: ChildhoodRules;
  later_life: LaterLifeRules;
  // The years a magus lives after its Gauntlet, when the ruleset ships them —
  // optional for the same reason apprenticeship is.
  post_apprenticeship?: PostApprenticeshipRules;
}

// Life as a magus after the Gauntlet: what a year out of apprenticeship is worth
// and what a season of lab work costs against it. Mirrors the engine's
// `PostApprenticeshipRules`. Counted in POINTS, not experience: each point is an
// experience point in an Art or Ability OR one level of spell, and the player
// decides which.
export interface PostApprenticeshipRules {
  // What one season of lab work costs the year that holds it (10 points).
  lab_season_cost: number;
  // How many lab seasons in one year actually cost anything (3) — the deduction
  // stops at 0, so a fourth season in the same year is free.
  max_charged_lab_seasons_per_year: number;
  // The points one year out of apprenticeship grants (30).
  points_per_year: number;
}

// Apprenticeship: fifteen years and 240 experience points, plus the Abilities the
// Order demands and the ones it recommends. Mirrors the engine's
// `ApprenticeshipRules`. The 120 spell levels are NOT here — they are the magus
// type profile's `spell_levels`.
export interface ApprenticeshipRules {
  // Abilities without which a magus "would not be admitted to the Order".
  minimum_abilities: AbilityRequirement[];
  // The recommended package, priced by `recommended_xp`.
  recommended_abilities: AbilityRequirement[];
  recommended_xp: number;
  xp: number;
  years: number;
}

// An Ability score a rule demands. Mirrors the engine's `AbilityRequirement`;
// `parameter` narrows the demand to one instance of a parameterized Ability and is
// unset throughout the shipped data (the match is by Ability id).
export interface AbilityRequirement {
  ability: string;
  min_score: number;
  parameter?: string;
}

// Early childhood: a fixed block of years granting a native language plus a
// restricted spread of the Abilities a child picks up. Mirrors the engine's
// `ChildhoodRules`.
export interface ChildhoodRules {
  years: number;
  // The Ability the native-language experience buys — data, not a hardcoded slug,
  // so a ruleset naming its language Ability differently still resolves.
  native_language_ability: string;
  // Experience for the native language alone, spendable on nothing else.
  native_language_xp: number;
  // Experience to divide between `spread_abilities`.
  spread_xp: number;
  // The closed list the spread may be spent on (a Rust set, so an id array).
  spread_abilities: string[];
}

// Later life: the base experience per year from the end of childhood to the
// character's age. Mirrors the engine's `LaterLifeRules`; the rate this character
// actually earns (Wealthy 20 / Poor 10 replace it) arrives already resolved as
// `LifeStageBudget.later_life_rate`.
export interface LaterLifeRules {
  xp_per_year: number;
}

// One Ability score a Sample Childhood package grants. Mirrors the engine's
// `ChildhoodEntry`.
export interface ChildhoodEntry {
  ability: string;
  // The whole Ability score the package brings this Ability to.
  score: number;
  // For a parameterized Ability, the key the player's value is supplied under
  // (`area_a`, `area_b`, `language`) — what tells two entries of the same Ability
  // apart. Omitted for an entry that needs nothing asked.
  slot?: string;
  // Whether this entry is the package's native language, funded by the childhood's
  // native-language block rather than the spread. Omitted = false. Which language
  // it is is the player's choice (`LifeStagePlan.native_language`).
  native?: boolean;
}

// A Sample Childhood package: a ready-made Ability spread a player may take
// instead of dividing the childhood experience by hand. Name/description live in
// the rules i18n map, keyed by `id` (like Arts and Houses) — not in Fluent.
// `entries` keep their authored file order, which is the order a UI must ask for
// the parameterized ones in. Mirrors the engine's `ChildhoodPackage` (the
// provenance `source` field is not surfaced to the UI).
export interface ChildhoodPackage {
  id: string;
  entries: ChildhoodEntry[];
}

// `Ruleset` serializes its maps as JSON objects keyed by id.
export interface Ruleset {
  id: string;
  version: string;
  point_items: Record<string, PointItem>;
  type_profiles: Record<string, EntityTypeProfile>;
  // Present from schema with houses loaded; optional so older shapes still
  // type-check. Keyed by House id (e.g. `house.bjornaer`).
  houses?: Record<string, House>;
  // Keyed by Mythic Companion type id (e.g. `mythic_type.devil_child`).
  mythic_companion_types?: Record<string, MythicCompanionType>;
  // Present from schema with abilities/characteristics loaded; optional so older
  // shapes still type-check.
  abilities?: Record<string, Ability>;
  advancement?: AbilityXpRow[];
  // Present from schema with arts loaded; optional so older shapes still
  // type-check. `art_advancement` reuses the Ability XP-row shape.
  arts?: Record<string, Art>;
  art_advancement?: AbilityXpRow[];
  // Present from schema with spells loaded; optional so older shapes still
  // type-check. Keyed by spell id (e.g. `spell.pilum_of_fire`).
  spells?: Record<string, Spell>;
  // Present from schema with the Spell Mastery special-ability catalogue loaded;
  // optional so older shapes still type-check. Keyed by ability id
  // (e.g. `spell_mastery_ability.penetration`).
  spell_mastery_abilities?: Record<string, SpellMasteryAbility>;
  // Present from schema with equipment loaded; optional so older shapes still
  // type-check. Keyed by catalogue id (`weapon.*`, `shield.*`, `armor.*`).
  weapons?: Record<string, Weapon>;
  shields?: Record<string, Shield>;
  armor?: Record<string, Armor>;
  characteristic_rules?: CharacteristicRules | null;
  // The life-stage experience rules. Absent for a ruleset shipping no life-stages
  // file, which leaves the typed `xp_pool` the only source of experience.
  life_stages?: LifeStageRules;
  // Sample Childhood packages keyed by package id (e.g. `childhood.athletic`). The
  // engine always sends the map — empty for a ruleset shipping none — but it stays
  // optional here like the other catalogues above, so older shapes still type-check.
  childhoods?: Record<string, ChildhoodPackage>;
  // The aging tables (Living Conditions + Aging Roll). Absent for a ruleset that
  // ships no aging file, which stands the whole aging subsystem down.
  aging?: AgingRules | null;
  // Derived taxonomy surfaced by the engine so the UI never re-hardcodes the
  // magnitude point weights or the ability-category / art-type order. Source of
  // truth is the Rust `Magnitude::points` / `AbilityCategory::ALL` / `ArtType::ALL`.
  magnitude_points: Record<Magnitude, number>;
  ability_category_order: AbilityCategory[];
  art_type_order?: ArtType[];
  // The minimum level a Ritual spell may be learned at, derived from the Rust
  // `spell::RITUAL_MIN_LEVEL` constant (populated at construction and
  // re-derived on every deserialize, never trusted from input JSON), so the UI
  // reads the Ritual floor from engine data instead of re-hardcoding it (VA2).
  // Optional like `art_type_order` above (not `magnitude_points`/
  // `ability_category_order`, present since the type's introduction): a live
  // engine payload always sends it, but typing it required would force every
  // existing hand-built `Ruleset` test fixture across the tree to add a field
  // unrelated to what each of those tests exercises.
  ritual_min_level?: number;
  // The rules-legal range for `Entity.aura`, mirrored from the Rust
  // `types::AURA_MODIFIER_MIN`/`MAX` constants (populated at construction and
  // re-derived on every deserialize, never trusted from input JSON), so the
  // aura number inputs (DerivedTotalsPanel, MagicPossessions) bound themselves
  // from engine data instead of re-hardcoding -50/10 (round 3, Task 3).
  // Optional like `ritual_min_level` above, for the same reason.
  aura_modifier_min?: number;
  aura_modifier_max?: number;
}

export interface LocalizedRuleset {
  ruleset: Ruleset;
  i18n: Record<string, I18nEntry>;
}

export interface RulesetRef {
  id: string;
  version: string;
}

export interface Selection {
  ref: string;
  params?: Record<string, string>;
}

// A whole bought Ability score with an optional free-text specialty. Keyed by
// (ability, specialty): the same parameterized ability may appear more than once.
export interface AbilityScore {
  ability: string;
  score: number;
  specialty?: string | null;
  // Player-supplied value for a parameterized ability (e.g. the area for
  // (Area) Lore). Part of the instance identity, so several can coexist.
  parameter?: string | null;
}

// A whole bought Art score (magi only). Arts are not parameterized and carry no
// specialty, so an Art is identified by id alone.
export interface ArtScore {
  art: string;
  score: number;
}

export interface Entity {
  schema_version: number;
  ruleset: RulesetRef;
  entity_kind: EntityKind;
  type_id: string;
  // The chosen Virtues/Flaws. Omitted when empty (a character with none at all —
  // a bare grog — carries no key), so every read must be defensive.
  selections?: Selection[];
  // Chosen Characteristic scores (point-buy). Omitted when empty.
  characteristics?: Record<Characteristic, number>;
  // Optional free-text description per Characteristic (sheet flavor). Omitted empty.
  characteristic_descriptions?: Partial<Record<Characteristic, string>>;
  // Whole bought Ability scores. Omitted when empty.
  ability_scores?: AbilityScore[];
  // Total XP available to spend on Abilities AND Arts — one shared bank. Spent
  // is derived (ability + art cost), leftover is the banked XP. Omitted when zero.
  xp_pool?: number;
  // The character's life-stage choices, when it is built through them (childhood +
  // later life) rather than by typing `xp_pool` directly. Mutually exclusive with a
  // non-zero `xp_pool`. Omitted for a directly-entered character.
  life_stages?: LifeStagePlan;
  // Whole bought Art scores (magi only). Priced against the shared xp_pool.
  // Omitted when empty.
  art_scores?: ArtScore[];
  // The spells the character knows (magi only). Each consumes the spell-levels
  // budget. Omitted when empty.
  spells?: SpellSelection[];
  // Optional per-character override of the type profile's base spell-levels
  // budget (a stored choice). When set it REPLACES the profile base; Skilled/Weak
  // Parens modifiers still add on top. Omitted when unset (use the profile base).
  spell_levels_override?: number | null;
  // The Hermetic House (magi only). Stores only the choice; the free House
  // Virtue is derived engine-side, never persisted. Omitted when unset.
  house?: string | null;
  // House specialisation picks, keyed by each grant's choice_key. Omitted empty.
  house_choices?: Record<string, Selection>;
  // The Mythic Companion type (mythic companions only). Stores only the choice;
  // free status/Minor Virtue derived engine-side. Omitted when unset.
  mythic_type?: string | null;
  // Mythic type grant picks, keyed by each grant's choice_key. Omitted empty.
  mythic_choices?: Record<string, Selection>;
  // The character's age (drives the age → max-Ability-score cap). Omitted unset.
  age?: number | null;
  // The character's apparent age. Seeded at 35 and advanced by one for every
  // resolved aging roll whose total reached the threshold; a hand-entered value is
  // never re-seeded, only advanced. Omitted when unset.
  apparent_age?: number | null;
  // Named Personality Traits. Omitted when empty.
  personality_traits?: PersonalityTrait[];
  // Starting Reputations (each backed by a granting V/F). Omitted when empty.
  reputations?: Reputation[];
  // The realm aura modifier the magus starts under (signed). Omitted when 0.
  aura?: number;
  // Starting enchanted devices (magi); each level is charged against the
  // item-level budget. Omitted when empty.
  devices?: EnchantedDevice[];
  // The magus's familiar and bond cords. Omitted when none.
  familiar?: Familiar | null;
  // The magus's talisman — identity, attunements, instilled effects. Omitted when
  // none (a magus may have at most one). Replaced the flat `talisman_attunements`
  // list in schema 14; legacy saves are folded in by the engine on load.
  talisman?: Talisman | null;
  // The magus's Longevity Ritual. Omitted when none.
  longevity_ritual?: LongevityRitual | null;
  // Accrued aging points per Characteristic — the lifetime total. Their sum is
  // Decrepitude XP; the Characteristic drops they force are derived by the engine
  // (never stored) and lower the effective/derived score, never the bought score
  // creation-legality reads. Omitted when empty.
  aging_points?: Partial<Record<Characteristic, number>>;
  // Accrued Warping Points (summed with grant points, inverted to the score by
  // the engine). Omitted when 0.
  warping_points?: number;
  // The player's chosen fills for the off-budget Virtues/Flaws a non-magus owes
  // from its Warping Score, keyed by each owed slot's stable choice_key (see
  // EffectiveScores.warping_owed_grants). Like house_choices/mythic_choices these
  // resolve to derived selections engine-side and never touch the V/F budget.
  // Omitted when empty. Mirrors the engine's Entity.warping_choices.
  warping_choices?: Record<string, Selection>;
  // Free-text description of how the character's Warping manifests (the
  // source-reflecting Flaw from "Effects of Warping"). A pure annotation — NOT a
  // Flaw selection, so it never counts against the creation V/F budget. Omitted
  // when empty.
  warping_effect?: string;
  // Twilight Scars (free-text). Omitted when empty.
  twilight_scars?: TwilightScar[];
  // Free-text narrative of the character's overall aging / decrepitude (pure
  // annotation, no mechanic). Omitted when empty.
  decrepitude_effect?: string;
  // Per-year aging-roll log. A resolved year carries the whole roll and is what
  // the engine reverts; a hand-written entry carries only its free text. Omitted
  // when empty.
  aging_log?: AgingLogEntry[];
  // The Living Conditions the character lives under, as ids into the aging
  // catalogue — stored choices, never the resolved modifier. Sorted, and omitted
  // when empty (an empty set IS the table's "Average peasant 0").
  living_conditions?: string[];
  // Identity / flavor fields (free-text, no mechanical effect). Omitted when empty.
  name?: string;
  // Short one-line tagline shown under the name in the header banner.
  description?: string;
  // Longer free-text character concept (edited in the Details tab).
  concept?: string;
  gender?: string;
  birth_year?: number | null;
  sigil?: string;
  covenant_name?: string;
  parens?: string;
  // Carried weapons, shields, and armor (references to catalogue ids). Combat
  // totals, Soak, and Encumbrance are derived downstream (slice 5i). Omitted empty.
  equipment?: EquipmentSlot[];
  // A supernatural being's base Might Score + Realm (grog/companion/mythic
  // companion with a Might Virtue). Omitted for ordinary characters.
  might?: MightScore | null;
  // The being's supernatural powers; each level is charged against the
  // power-levels budget its Might Virtues grant. Omitted when empty.
  powers?: SupernaturalPower[];
}

export interface ValidationIssue {
  severity: IssueSeverity;
  // Stable machine key, also the Fluent message id the UI localizes.
  code: string;
  // The creation phase whose input surface owns the offending value — what the
  // wizard filters each step's findings on. Always present.
  phase: CreationPhase;
  // Interpolation values for the localized message, keyed by argument name.
  args: Record<string, string>;
  context?: string | null;
}

// Which of the character's declared creation phases hold no choices yet, in the
// profile's own order. Deliberately not shaped like a ValidationIssue: it carries
// no severity, so nothing that gates on `error` can ever see it.
export interface CompletenessReport {
  incomplete_phases: CreationPhase[];
}

export interface ValidationResult {
  issues: ValidationIssue[];
  // Always present on an engine payload; optional here because a result built
  // from a bare issue list (and every test fixture) legitimately carries none,
  // which reads as "nothing to report".
  completeness?: CompletenessReport;
}

// Tauri command errors are rejected as a tagged object whose `kind`
// discriminates the variant. `io`/`serialize` carry a `message`; `ruleset`
// preserves the engine's own discriminant (`ruleset_kind`: 'parse' |
// 'integrity') and the individual violation messages (`errors`).
export type AppError =
  | { kind: 'io'; message: string }
  | { kind: 'ruleset'; ruleset_kind: 'parse' | 'integrity'; errors: string[] }
  | { kind: 'not_loaded' }
  | { kind: 'serialize'; message: string }
  // A Markdown export could not resolve a chrome key or catalogue id to display
  // text. `missing` carries one entry per offense, so every fix needed is
  // reported in one round trip. Mirrors `AppError::Export` in
  // `crates/arm-app/src/error.rs`; the banner localizes it through
  // `error-export`, never by rendering the kind or the ids themselves.
  | { kind: 'export'; missing: string[] };
