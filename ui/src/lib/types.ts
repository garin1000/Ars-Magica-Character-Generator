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
  | { type: 'grants_spell_mastery'; score: number }
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
  | { type: 'ability_roll_mod'; param: string; amount: number };

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
  | 'decrepitude'
  | 'living_conditions';
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
  // Derived Size (base 0; Large +1, Giant Blood +2, Small Frame -1, Dwarf -2).
  size: number;
  // Free effective-score bonuses to Characteristics (Giant Blood +1 Str/Sta, Dwarf -1).
  characteristic_bonuses: CharacteristicBonus[];
  // Virtue/Flaw Selections the entity's House grants (derived, never persisted),
  // in the House's declared grant order, so the V/F view renders them read-only.
  granted_selections: Selection[];
  // Effective virtue/flaw point ceilings (base budget + Mythic Companion type
  // bonus), so the balance bar shows the true budget (Devil Child 37/17).
  virtue_budget: number;
  flaw_budget: number;
  // The magus's effective spell-levels budget (profile base + Skilled/Weak
  // Parens) and how many levels the chosen spells consume — the spell bar.
  spell_levels_budget: number;
  spell_levels_used: number;
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
  // Derived Warping Score / Points granted by V/F (Warped by Magic 1 / 5); 0/0
  // when nothing grants Warping.
  warping_score: number;
  warping_points: number;
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
  permitted_categories: string[];
  forbidden_categories: string[];
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

// A spell in the catalogue. `level` is a fixed number, or omitted for a General
// spell (learned at a per-character level). Technique/Form are Art ids.
export interface Spell {
  id: string;
  technique: string;
  form: string;
  level?: number | null;
  requisites?: string[];
}

// A spell the character knows. `level` is set only for a General spell (the
// chosen level); for a fixed spell the catalogue level is authoritative.
export interface SpellSelection {
  spell: string;
  level?: number | null;
  // Bought Spell Mastery Ability score (spent from the mastery-XP pool);
  // omitted/0 = unmastered. Effective mastery = max(this, mastery floor).
  mastery?: number | null;
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

// A magus's familiar and its three bond-cord scores. Omitted cords default to 0.
export interface Familiar {
  name: string;
  cord_gold?: number;
  cord_silver?: number;
  cord_bronze?: number;
}

// A talisman attunement: a free-text descriptor and the bonus it confers.
export interface TalismanAttunement {
  description: string;
  bonus: number;
}

// Where a Longevity Ritual comes from (rendered via Fluent, never as a raw slug).
export type LongevitySource = 'self_made' | 'external';

// A magus's Longevity Ritual. `self_made` leaves `bonus` unset (computed
// downstream); `external` carries a player-entered bonus.
export interface LongevityRitual {
  source: LongevitySource;
  bonus?: number | null;
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
  // The spells the character knows (magi only). Each consumes the spell-levels
  // budget. Omitted when empty.
  spells?: SpellSelection[];
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
  // Talisman attunements (magi). Omitted when empty.
  talisman_attunements?: TalismanAttunement[];
  // The magus's Longevity Ritual. Omitted when none.
  longevity_ritual?: LongevityRitual | null;
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
