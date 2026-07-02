// Hand-mirrored from the `arm-rules` serde types. Kept minimal: only the shapes
// that cross the Tauri boundary for Milestone 2. Codegen (ts-rs/specta) is out
// of scope here.

export type Magnitude = 'free' | 'minor' | 'major';
export type ItemKind = 'virtue' | 'flaw' | 'boon' | 'hook';
export type EntityKind = 'character' | 'covenant';
export type ValidationMode = 'enforced' | 'advisory' | 'silent';
export type IssueSeverity = 'error' | 'warning';

// Closed enums in the engine (`ParamType` / `ParameterDomain`), serialized as
// their snake_case names.
export type ParamType = 'ref';
export type ParameterDomain = 'ability' | 'art' | 'characteristic' | 'item';

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
  | { type: 'characteristic_points'; amount: number }
  | { type: 'ability_score_grant'; ability: string; amount: number };

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

export interface PointItem {
  id: string;
  kind: ItemKind;
  magnitude: Magnitude;
  category: string;
  entity_kinds: EntityKind[];
  prerequisites?: Prereq;
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
  xp_total_demand: number;
  xp_general_used: number;
  restricted_xp_pools: RestrictedXpPool[];
  characteristic_points_granted: number;
  ability_score_floors: AbilityFloor[];
}

// Per-category flaw count cap. The category is data, so the engine hardcodes no
// slug; `major_only`/`hard` default to false and are omitted from JSON then.
export interface FlawCategoryCap {
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
  flaw_category_caps?: FlawCategoryCap[];
}

export interface EntityTypeProfile {
  id: string;
  budget: PointBudget;
  permitted_categories: string[];
  forbidden_categories: string[];
  // Whether this character type is a Hermetic magus. Omitted from JSON when
  // false (the common case), so optional here.
  is_magus?: boolean;
  creation_phases: string[];
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

// `Ruleset` serializes its maps as JSON objects keyed by id.
export interface Ruleset {
  id: string;
  version: string;
  point_items: Record<string, PointItem>;
  type_profiles: Record<string, EntityTypeProfile>;
  // Present from schema with abilities/characteristics loaded; optional so older
  // shapes still type-check.
  abilities?: Record<string, Ability>;
  advancement?: AbilityXpRow[];
  // Present from schema with arts loaded; optional so older shapes still
  // type-check. `art_advancement` reuses the Ability XP-row shape.
  arts?: Record<string, Art>;
  art_advancement?: AbilityXpRow[];
  characteristic_rules?: CharacteristicRules | null;
  // Derived taxonomy surfaced by the engine so the UI never re-hardcodes the
  // magnitude point weights or the ability-category / art-type order. Source of
  // truth is the Rust `Magnitude::points` / `AbilityCategory::ALL` / `ArtType::ALL`.
  magnitude_points: Record<Magnitude, number>;
  ability_category_order: AbilityCategory[];
  art_type_order?: ArtType[];
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
  selections: Selection[];
  // Chosen Characteristic scores (point-buy). Omitted when empty.
  characteristics?: Record<Characteristic, number>;
  // Optional free-text description per Characteristic (sheet flavor). Omitted empty.
  characteristic_descriptions?: Partial<Record<Characteristic, string>>;
  // Whole bought Ability scores. Omitted when empty.
  ability_scores?: AbilityScore[];
  // Total XP available to spend on Abilities AND Arts — one shared bank. Spent
  // is derived (ability + art cost), leftover is the banked XP. Omitted when zero.
  xp_pool?: number;
  // Whole bought Art scores (magi only). Priced against the shared xp_pool.
  // Omitted when empty.
  art_scores?: ArtScore[];
  // The Hermetic House (magi only). Stores only the choice; the free House
  // Virtue is derived engine-side, never persisted. Omitted when unset.
  house?: string | null;
  // House specialisation picks, keyed by each grant's choice_key. Omitted empty.
  house_choices?: Record<string, Selection>;
}

export interface ValidationIssue {
  severity: IssueSeverity;
  // Stable machine key, also the Fluent message id the UI localizes.
  code: string;
  // Interpolation values for the localized message, keyed by argument name.
  args: Record<string, string>;
  context?: string | null;
}

export interface ValidationResult {
  issues: ValidationIssue[];
}

// Tauri command errors are rejected as a tagged object whose `kind`
// discriminates the variant. `io`/`serialize` carry a `message`; `ruleset`
// preserves the engine's own discriminant (`ruleset_kind`: 'parse' |
// 'integrity') and the individual violation messages (`errors`).
export type AppError =
  | { kind: 'io'; message: string }
  | { kind: 'ruleset'; ruleset_kind: 'parse' | 'integrity'; errors: string[] }
  | { kind: 'not_loaded' }
  | { kind: 'serialize'; message: string };
