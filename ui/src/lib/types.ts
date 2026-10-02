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
  | 'characteristics'
  | 'virtues_flaws'
  | 'experience'
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
export type ParamType =
  | 'ref'
  // D35: a bounded integer count (Simple Student's 1-2 finished years).
  // Externally-tagged, matching the Rust struct variant's own wire form
  // (`{ "number": { "min": 1, "max": 2 } }`) — never a bare string, so it can
  // never be confused with `'ref'`.
  | { number: { min: number; max: number } }
  // D9 part 3 (C5a, `docs/vf-audit/design-c0-parameter-model.md` § 8): an
  // open-ended SET of ids (the three Corrupted entries: "you can choose to
  // have it affect multiple Abilities", ArMDE:5851 et al.). Type-level parity
  // only in this slice — no picker control exists yet (C5b's job).
  | 'multi_ref';
export type ParameterDomain =
  | 'ability'
  | 'art'
  | 'technique'
  | 'form'
  | 'characteristic'
  | 'item'
  | 'enumerated'
  // Value is one of the declaring item's OWN `categories` — records which
  // single reading of a multi-category Virtue/Flaw was chosen (Sufi, "either
  // as a Minor Social Status Virtue or a Minor Supernatural Virtue",
  // ArMDE:5083). Same shape as
  // `enumerated` (the domain IS `values`), but the picker labels options
  // through `category-<id>` rather than `displayName`.
  | 'category'
  // Value is one of the four Realms as `realm.<slug>` — Folk Magic's magic "is
  // aligned to" one of them (ArMDE:3909). A closed engine taxonomy, so the
  // menu is `REALMS` and the
  // labels come from the `realm-<id>` Fluent family, never from rules i18n
  // (a bare realm slug has no entry there) and never from a declared list.
  | 'realm'
  | 'text'
  // D35: a bounded integer count — the redundant half of the `ParamType`
  // pair (see the Rust `ParameterDomain::Number` doc comment); it carries no
  // resolution logic here either, only the `param-domain-number` label
  // `unknown_param_value` needs.
  | 'number'
  // C5a: a spell id, resolved against the OWNING character's OWN learned
  // spells (`Entity.spells`), never the ruleset's whole spell catalogue —
  // Corrupted Spells (ArMDE:5859-5863). In practice only ever paired with
  // `multi_ref`.
  | 'spell'
  // X6a/e5: one of the closed 5-member AbilityCategory enum, as
  // `ability_category.<slug>` — no catalogue, no declared `values`, exactly
  // like `realm` above. Labelled through the already-shipped
  // `ability-category-<slug>` Fluent family.
  | 'ability_category';

export interface ParameterDef {
  key: string;
  type: ParamType;
  domain: ParameterDomain;
  // The closed list of legal value ids, for the `enumerated` domain only — the
  // domain IS its list, so it travels with the parameter rather than naming a
  // catalogue. Omitted from the JSON for every other domain, where the engine
  // rejects it at load.
  values?: string[];
  // Groups of values of which at most ONE may be named across all copies of the
  // declaring item — Folk Magic's "a character cannot have access to both the
  // Divine and Infernal Realms" (ArMDE:3919). Enforced by the engine
  // (`exclusive_param_values`); which values exclude each other is data, so no
  // id is named in the frontend either.
  at_most_one_of?: string[][];
  // How many of the item's copies may name one and the same value for THIS key.
  // Necessary (Realm) Aura for (Ability) is taken "once for any particular
  // Ability" (ArMDE:6482), a cap the whole-tuple `max_per_target` cannot state.
  // D10: omitted means the engine's default of 1 (once); an entry that
  // legitimately repeats one value carries an explicit 255 (Folk Magic's
  // `realm` axis, ArMDE:3919). Enforced by the engine
  // (`too_many_for_param_value`); like `at_most_one_of`, the picker does not
  // pre-empt it — the finding names the key and the value.
  max_per_value?: number;
  // Narrows the `item` domain to a category: the point item the value names must
  // carry at least one of these, so the picker offers only those. Membership
  // (`PointItem.categories`) and nothing else — a value names an ITEM, not a
  // selection of one, so there is no `taken_as` reading to narrow against, and
  // `index_categories` is provenance. Absent on every other domain, where the
  // engine rejects it at load.
  require_categories?: string[];
  // A one-id whitelist, additive to `require_categories`: a value resolves if
  // EITHER test passes. False Power's target is `require_categories:
  // ['supernatural']` plus `allow_ids: ['virtue.diedne_magic',
  // 'virtue.the_gift']` — the two Virtues ArMDE:6082 names outright but whose
  // OWN category ('hermetic', 'special') the domain does not otherwise reach.
  // Absent on every other domain, where the engine rejects it at load (D34).
  allow_ids?: string[];
  // Narrows the `ability` domain to a non-empty intersection with these
  // AbilityCategory values (X6a/e3, the `ability`-domain mirror of
  // `require_categories`). Absent on every other domain, where the engine
  // rejects it at load.
  require_ability_categories?: string[];
  // Subtracts these ids from an `ability` domain's resolution — the
  // subtractive mirror of `allow_ids` (X6a/e3: Magian Lineage Major excludes
  // True Names). Absent on every other domain, where the engine rejects it
  // at load.
  forbid_ids?: string[];
  // The `item` the value names must be one the character HOLDS — bought or
  // granted, the engine's grants-inclusive `present_ids`. False Power is taken
  // "once for each appropriate Supernatural Virtue that the character
  // possesses" (ArMDE:6096), and the same flag makes each held Virtue
  // claimable once. Absent on every other domain, where the engine rejects it
  // at load.
  require_possessed?: boolean;
  // The `item` the value names may not be Tainted: "this Flaw cannot apply to
  // Supernatural Virtues that are affiliated to the Infernal realm in the first
  // place" (ArMDE:6096).
  forbid_tainted?: boolean;
  // Narrows the `item` domain by DESCRIPTION rather than category: a value
  // resolves only if the named item does NOT satisfy this predicate (D33,
  // `flaw.flawed_powers`'s "only appropriate to Hermetic Magic" import
  // constraint). Absent on every other domain, where the engine rejects it
  // at load. Not yet consumed by any picker — the resolved option list an
  // IPC command hands the frontend already reflects it server-side.
  exclude_if?: ItemPredicate;
  // This parameter is only REQUIRED when the OWNING selection's own gate
  // holds (B4/Q-51) — Magical Blood's `characteristic` is meaningless for
  // Magic Animal/Spirit/Thing, so it is required only when `bloodline`
  // equals `bloodline.magic_human`. `ParameterPicker` reads this to hide the
  // control entirely when the gate does not hold, matching the engine's own
  // `missing_param` relaxation (`validation/selections.rs`).
  required_if?: ParamGate;
  // The exact number of DISTINCT values a `multi_ref` selection must name
  // (X6a/e4: Restricted Learning, "choose five Abilities"). Absent on any
  // other `type`, where the engine rejects it at load.
  exact_count?: number;
}

// Names a parameter this item declares and the literal value that activates
// something conditional on it — mirrors the engine's `ParamGate` (C0/C1),
// read directly off the OWNING selection's own `params[param]`.
export interface ParamGate {
  param: string;
  equals: string;
}

// A value fixed at authoring time, or read from the OWNING selection's own
// parameter at evaluation time (D14's two forms) — mirrors the engine's
// `ParamValue` (`#[serde(untagged)]`, distinguished by which field is
// present: `{ literal }` vs `{ param }`, never a bare string).
export type ParamValue = { literal: string } | { param: string };

// An Ability id inside an `Effect::RestrictedAbilityXp`/`AbilityAuthorization`
// list, carrying D14's two constraints: optionally restricted to ONE
// instance of a parameterized Ability, and/or active only when the OWNING
// selection's own gate holds. Mirrors the engine's `AbilityRef`
// (`#[serde(untagged)]`): a bare string for the common unconstrained case, or
// a scoped object.
export type AbilityRef = string | { ability: string; instance?: ParamValue; gate?: ParamGate };

// One entry of `PointItem.conditional_incompatible_with` (X6a/e7): while
// `gate` holds for the declaring item's own selection, every id in `forbids`
// becomes incompatible with it. Mirrors the engine's `ConditionalIncompatibility`.
export interface ConditionalIncompatibility {
  gate: ParamGate;
  forbids: string[];
  // The same-copy twin of `forbids` (RC review-C item 2, ArMDE:7033): while
  // `gate` holds for the declaring selection, an OTHER selection of the SAME
  // item whose own value of `gate.param` equals one of these is also
  // forbidden — "Weak Sight is incompatible with Sensitive Sight", two copies
  // of `flaw.warped_senses` reading the same `affliction` parameter key.
  // Omitted when empty (the common case: only Weak Sight/Weak Hearing declare
  // this).
  forbids_same_item_values?: string[];
}

// One entry of `PointItem.same_choice_exclusions` (D69.6): the declaring item
// may not be held alongside a selection of `other` that resolves `other_param`
// to the SAME target this item resolves to — "You may not take Student of
// (Realm) and Puissant Ability for the same Lore" (ArMDE:5054). The target is
// either `fixed_target` (set regardless of this item's own parameters), or —
// when `this_param` is set — this item's OWN `this_param` value mapped
// through `via`. Exactly one of `this_param`/`fixed_target` is ever set.
// Mirrors the engine's `SameChoiceExclusion` (`types.rs::SameChoiceExclusion`).
export interface SameChoiceExclusion {
  other: string;
  other_param: string;
  this_param?: string;
  via?: Record<string, string>;
  fixed_target?: string;
}

// A property-based test over a point item, for an exclusion the rulebook
// states by description rather than by id (`PointItem.incompatible_with`) or
// category (`Effect.forbids_item_category`) — D23/D33/D68.4/D69.5. Mirrors the
// engine's `ItemPredicate`, a closed 5-member enum
// (`item-predicate-parity.test.ts` guards the two sides against drift).
export type ItemPredicate =
  | 'trained'
  | 'grants_reputation'
  | 'grants_personality_trait'
  | 'requires_hermetic_arts'
  | 'affects_size';

// Mechanical effect a virtue/flaw applies. `ability_bonus` adds to an ability's
// effective score (Puissant Ability +2); `characteristic_score_delta_param`
// grants a free point to a characteristic (Great Characteristic +1, Poor
// Characteristic -1) without touching the bought score, which stays inside the
// printed +-3 point-buy table. The target is named by the selection's `param`
// value.
export type Effect =
  | { type: 'ability_bonus'; param: string; amount: number }
  | {
      type: 'characteristic_score_delta_param';
      param: string;
      amount: number;
      // B4/Q-51: applies only when the OWNING selection's own gate holds
      // (Magical Blood's Magic Human clause). Absent for every other carrier
      // (Great/Poor Characteristic), which apply unconditionally.
      gate?: ParamGate;
      // Which precondition/clamp shape this delta follows — data, never
      // inferred from whether `gate` is present (coordinator review, post-B4).
      // `above_base` (the default, omitted from the JSON) is Great/Poor
      // Characteristic's own uncapped shape; `within_base` (Magical Blood's
      // Magic Human clause) clamps to the base cap/floor instead. Mirrors the
      // engine's `CharacteristicDeltaCap`.
      cap?: 'above_base' | 'within_base';
    }
  | { type: 'art_bonus'; param: string; amount: number }
  | { type: 'affinity_ability_cost'; param: string; counts_as_num: number; counts_as_den: number }
  | { type: 'affinity_art_cost'; param: string; counts_as_num: number; counts_as_den: number }
  | {
      type: 'restricted_ability_xp';
      amount: number;
      abilities?: string[];
      categories?: string[];
      // D48: specific ability instances this pool also funds (Marshal's
      // Profession: Marshal, Master Bard's Profession: Storyteller/Poet) —
      // a union with `abilities`/`categories`, never "instances-only".
      instances?: AbilityRef[];
      // X6a/e5: names a `multi_ref`/`ability`-domain parameter on the SAME
      // item whose resolved set is a further eligibility source, unioned with
      // `abilities`/`categories` (Restricted Learning's five player-named
      // Abilities, ArMDE:6685).
      abilities_param?: string;
      // D13: this grant earmarks part of the GENERAL pool rather than adding
      // to it — `false` (the default, omitted from the JSON) preserves every
      // existing carrier's additive meaning (Educated, Warrior, Privileged
      // Upbringing).
      from_normal_budget?: boolean;
    }
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
  // No `score` field (D77.3): the Warping Score is always derived from the
  // point total alone, never authored directly, so a stored score could only
  // ever be an unread, disagreeable second copy of the same fact.
  | { type: 'warping_grant'; points: number }
  // D69.1: Raised from the Dead's parameterized sibling of `warping_grant` —
  // `base_points` unconditionally, plus one per unit named by the OWNING
  // selection's `params[param]` (ArMDE:6646-6649).
  | { type: 'warping_grant_param'; param: string; base_points: number }
  | { type: 'true_faith_grant'; score: number }
  // F-256: a Relic's own True Faith Score (ArMDE:17607), not the bearer's.
  // Surfaced-only — nothing here renders it directly.
  | { type: 'relic_true_faith'; score: number }
  | { type: 'item_level_budget'; amount: number }
  | { type: 'spell_mastery_xp'; amount: number }
  | {
      type: 'grants_spell_mastery';
      score: number;
      advancement_num?: number;
      advancement_den?: number;
    }
  | { type: 'grants_selection'; items: string[] }
  // D68.11: an id-less category-cap count (Mythic Blood's hereditary
  // Personality Flaw, ArMDE:4588) — no real point item, so nothing for a
  // picker to render; read only by the derived category-cap counts.
  | { type: 'grants_category_count'; category: string; magnitude: Magnitude; item_kind: ItemKind }
  | { type: 'size_delta'; amount: number }
  | {
      type: 'characteristic_score_delta';
      characteristic: string;
      amount: number;
      // X6a/e1: applies only when the OWNING selection's own gate holds
      // (Faerie Blood's Sidhe clause). Absent for every unconditional carrier.
      gate?: ParamGate;
    }
  // D69/X7b-e (row 42, Uninspirational): lowers the BUY CAP of a fixed
  // Characteristic while this selection is in effect, rather than adding a
  // free delta — the sign-mirror of `characteristic_score_delta` above.
  | { type: 'characteristic_max'; characteristic: string; max: number }
  | {
      type: 'grants_reputation';
      kind?: ReputationType;
      score: number;
      max_score?: number;
      // Same meaning as `characteristic_score_delta_param`'s own `gate` (B4/Q-51).
      gate?: ParamGate;
    }
  // M5/5b in-play effects (consumed by the derived-totals read-out, slice 5i).
  | { type: 'magical_focus'; param: string; major: boolean }
  | {
      type: 'casting_total_mod';
      amount: number;
      scope: CastingScope;
      // D79: counted only within the maga's own Potent Magic field, never
      // unconditionally — orthogonal to `scope` (the cast-TYPE axis). Mirrors
      // the Lab Total side's `within_potent_field_only` scope. Omitted
      // (false) for every carrier that predates D79.
      potent_field_only?: boolean;
    }
  | {
      type: 'lab_total_mod';
      amount: number;
      // D4/X7a: where this amount counts in the in-play Lab Total grid —
      // data, never an id the app hardcodes. `in_play_grid` (the default,
      // omitted from the JSON) applies flat; `within_potent_field_only`
      // (Potent Magic, D79 — named `within_focus_only` before it, when it was
      // wrongly gated on holding a Magical Focus instead of Potent Magic)
      // counts only within the maga's own Potent Magic field;
      // `never_at_creation` (Adept Laboratory Student, Weak Scholar) never
      // counts at all. Mirrors the engine's `LabTotalModScope`.
      scope?: 'in_play_grid' | 'within_potent_field_only' | 'never_at_creation';
      // D52: this amount is excluded from the in-play grid while the OWNING
      // selection's own gate holds (Cyclic Magic (Negative)'s seasonal
      // cycle).
      suppressed_when?: ParamGate;
    }
  | { type: 'deficient_art'; param: string }
  | { type: 'magic_total_halving'; total: HalvableTotal }
  | {
      type: 'soak_mod';
      amount: number;
      // X6a/e1: applies only when the OWNING selection's own gate holds
      // (Repellent's "scales" branch). Absent for every unconditional carrier.
      gate?: ParamGate;
    }
  // `weapon` scopes the modifier to one weapon's combat line, for a rule that
  // singles a weapon out — Lame's -3 applies to Dodge (`ArMDE:6332`), which is a
  // Brawling Weapons row rather than an Ability (`ArMDE:16959`), so the scope is
  // per-weapon. Absent means the modifier applies to every line.
  | { type: 'combat_mod'; amount: number; target: CombatStat; weapon?: string }
  | { type: 'health_mod'; track: HealthTrack; amount: number }
  // `param` names the selection parameter carrying the Form the modifier is
  // scoped to, for the two kinds the rulebook scopes that way. Absent for the
  // realm- and scene-conditional kinds, which name no Form.
  | {
      type: 'magic_resistance_mod';
      kind: MagicResistanceEffect;
      param?: string;
      // X6a/e1-e2: the flat Magic Resistance bonus (Commanding Aura's rank
      // figure). Absent (0) for every kind that carries no number of its own.
      amount?: number;
      // Applies only when the OWNING selection's own gate holds (Commanding
      // Aura's `rank` enumeration). Absent for every unconditional carrier.
      gate?: ParamGate;
    }
  | { type: 'aging_mod'; kind: AgingEffect; amount: number }
  // Exactly one of `amount`/`factor` is present (D55; load-time validated,
  // `ruleset/integrity.rs::validate_advancement_mod_shape`): `amount` is a
  // flat signed modifier, `factor` multiplies the source's Advancement Total
  // instead (Incomprehensible, Loose Magic — both halve, never an amount).
  | {
      type: 'advancement_mod';
      source: AdvancementSource;
      amount?: number;
      factor?: AdvancementFactor;
    }
  | { type: 'special_casting_mod'; kind: SpecialCasting }
  // B5/F-489: fixed target, named directly by the entry (Poor Hearing: -3 to
  // Awareness) — the parameter-relative shape below carries the OLD
  // `ability_roll_mod` tag's original meaning under its renamed tag.
  | {
      type: 'ability_roll_mod';
      ability: string;
      amount: number;
      // X6a/e1: applies only when the OWNING selection's own gate holds
      // (Faerie Blood's Dwarf clause). Absent for every unconditional carrier.
      gate?: ParamGate;
    }
  | { type: 'ability_roll_mod_param'; param: string; amount: number }
  // D69/X7b-e row 42 (Lingering Injury): a roll penalty over an unenumerated
  // category of rolls, scaled by 1 + Decrepitude Score. Surfaced-only —
  // nothing here renders it directly.
  | { type: 'decrepitude_scaled_roll_mod'; amount: number }
  // Elemental Magic (5c): creation-time Art-XP redistribution over the four
  // elemental Forms. Surfaced through the effective art bonus, not rendered raw.
  | { type: 'elemental_magic'; forms: string[] }
  // Unspecialized: forbids a specialty on any Ability. A creation-time
  // constraint the validator enforces (`specialty_forbidden`), not a score or
  // in-play modifier — nothing here renders it directly.
  | { type: 'forbids_ability_specialties' }
  // Rigid Magic (D27): forbids casting Ritual magic. Knowing a Ritual stays
  // legal (spells known, not spells castable) — the validator only advises
  // (`ritual_casting_restricted`), never blocks. A creation-time constraint,
  // not a score or in-play modifier — nothing here renders it directly.
  | { type: 'forbids_ritual_casting' }
  // Mentored by Demons (D29/F-194): waives the age→Ability-score cap outright,
  // for every Ability — a creation-time constraint the engine's age-cap
  // resolution point folds in; nothing here renders it directly.
  | { type: 'waives_ability_age_cap' }
  // X6a/e6 (Savantism, ArMDE:6705-6706): the Ability named by the OWNING
  // selection's `params[param]` caps at `max` INSTEAD OF the age-band figure.
  | { type: 'ability_score_cap_override_param'; param: string; max: number }
  // Savantism's sibling clause: every OTHER Ability (all but the one named by
  // the SAME item's `param`) caps at `max`, lowering the otherwise-applicable
  // age band.
  | { type: 'ability_score_cap_all_except'; param: string; max: number }
  // B1 (D21/F-355, F-542, F-511): category/ability prohibitions. All three
  // are creation-time constraints a dedicated validator enforces; nothing
  // here renders any of them directly.
  | { type: 'forbids_ability_category'; category: string }
  // X6a/e5: the parameter-relative sibling — the forbidden category is named
  // by the OWNING selection's own `params[param]` rather than fixed by the
  // item (Ability Block, ArMDE:5651-5654).
  | { type: 'forbids_ability_category_param'; param: string }
  | { type: 'forbids_item_category'; category: string }
  | { type: 'forbids_abilities'; abilities: string[] }
  // D69/X7b-e row 42 (Weak Personality): tightens the universal +-3
  // Personality Trait range to `max` while this selection is in effect.
  | { type: 'personality_trait_range'; max: number }
  // D69/X7b-e row 42 (Fickle Nature): requires at least two distinct
  // Personality Trait entries at exactly `value` while this selection is in
  // effect.
  | { type: 'requires_personality_trait_pair'; value: number };

// M5/5b scalar enums mirroring the engine (rendered via Fluent in slice 5i).
export type CastingScope = 'all' | 'formulaic' | 'ritual' | 'formulaic_ritual' | 'spontaneous';
// Magic Resistance is deliberately absent: the one Flaw that halved it, Flawed
// Parma Magica, halves one *addend* of it (the Parma contribution) against one
// *Form*, which is neither a whole total nor blanket. It is the
// `halved_parma` MagicResistanceEffect instead.
export type HalvableTotal =
  | 'spontaneous_casting'
  | 'lab_enchanting'
  | 'lab_longevity'
  | 'penetration';
export type CombatStat = 'initiative' | 'attack' | 'defense' | 'damage';
export type HealthTrack =
  | 'fatigue_penalty'
  | 'wound_penalty'
  | 'fatigue_roll'
  | 'casting_fatigue'
  | 'recovery';
export type MagicResistanceEffect =
  // `no_form_bonus` and `halved_parma` are scoped to the ONE Form the selection
  // names, carried in `param`; both fold into the flat per-Form Magic Resistance
  // number rather than being listed, so neither carries a `derived-detail-*` key.
  | 'no_form_bonus'
  | 'halved_parma'
  | 'aura_bonus'
  | 'susceptible_faerie'
  | 'susceptible_infernal'
  | 'conditional_penetration_waiver';
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
  | 'authoring'
  | 'spell_mastery'
  | 'all';
// The multiplier an `advancement_mod` factor applies (D55). A single value
// today (every stated factor in the core rules is a halving); a new one is a
// new union member, which every `Record<AdvancementFactor, …>` consumer turns
// into a type error until it is handled.
export type AdvancementFactor = 'half';
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
  | 'circumstantial'
  | 'doubled_aura_penalty';

// The audience a Reputation reaches (a fixed rules taxonomy, rendered via Fluent
// `reputation-type-<id>`, never as a raw slug).
export type ReputationType = 'local' | 'ecclesiastical' | 'hermetic' | 'academic';

// Prerequisite expression tree. Adjacently tagged by the engine: every variant
// is a uniform object carrying a `kind` discriminant, with any payload under
// `value` (the unit variants `hermetically_trained`/`order_member` have no
// `value`). D56/A0 sub-slice 4 split the old `is_magus` unit variant into
// these two independent facts. `is_companion` (D38/F-553) reads a profile-level
// class flag (true for `companion` and `mythic_companion` — "mythic companions
// are companions too"), so a narrower audience ("only companions can take
// this") lives on the entry rather than duplicated across every type
// profile's forbidden traits, and a future companion-like profile joins by
// setting the flag alone. `is_grog` (D68.9) is the audience twin of
// `is_companion`, true only for `grog` today. `has_category` (D21/F-502) is the category-ranging
// twin of `has`: the entity must hold (bought or granted) at least one item
// whose in-force category is `value`. `age_min` (D69/X7b-e, University Dean)
// requires the entity's own age to be at least `value` years. `has_category_at_magnitude`
// (D69/X7b-e/D68.4, Flawed Powers) is `has_category`'s magnitude- and kind-filtered
// twin: the entity must hold at least one item of `item_kind` whose in-force
// category is `category`, at or above `magnitude`. `character_type` (D38/D75,
// F-556) requires the entity's own type profile id to equal `value` literally
// (unlike `is_companion`/`is_grog`, which read a profile FLAG) —
// `virtue.domestic_animal` gates on an id no shipped profile carries, so no
// human character type can ever satisfy it. `characteristic_min` (D81.2)
// requires the entity's effective score in the named Characteristic to be at
// least `score`; a Characteristic the entity has not SET is unknown, not a
// definite failure, mirroring `age_min`'s own unset handling.
// `ability_category_score_min` (D81.3, Broken Vessel's "Supernatural Ability"
// half) requires an Ability of the named category at effective score at
// least `score` — a free-form category string like `has_category`'s, but
// ranging over the closed Ability-category taxonomy rather than
// `PointItem.categories`'s open vocabulary. `any_art_min` (D81.3, Broken
// Vessel's "... or Art" half) requires ANY Hermetic Art at effective score at
// least `score`.
export type HasCategoryAtMagnitudeValue = {
  category: string;
  magnitude: Magnitude;
  item_kind: ItemKind;
};

export type CharacteristicMinValue = { characteristic: string; score: number };

export type AbilityCategoryScoreMinValue = { category: string; score: number };

export type Prereq =
  | { kind: 'all'; value: Prereq[] }
  | { kind: 'any'; value: Prereq[] }
  | { kind: 'none'; value: Prereq[] }
  | { kind: 'has'; value: string }
  | { kind: 'house'; value: string }
  | { kind: 'ability_min'; value: { ability: string; score: number } }
  | { kind: 'art_min'; value: { art: string; score: number } }
  | { kind: 'hermetically_trained' }
  | { kind: 'order_member' }
  | { kind: 'is_companion' }
  | { kind: 'is_grog' }
  | { kind: 'has_category'; value: string }
  | { kind: 'age_min'; value: number }
  | { kind: 'has_category_at_magnitude'; value: HasCategoryAtMagnitudeValue }
  | { kind: 'character_type'; value: string }
  | { kind: 'characteristic_min'; value: CharacteristicMinValue }
  | { kind: 'ability_category_score_min'; value: AbilityCategoryScoreMinValue }
  | { kind: 'any_art_min'; value: { score: number } };

// How a V/F impacts a character mechanically (M5 slice 5a). Mirrors the engine's
// `Classification`. Required on every PointItem.
//
// `narrative` and `uncomputed_rule` both mean "the engine computes nothing"; they
// differ in whether the *rulebook* stated a rule. `uncomputed_rule` marks an entry
// whose passage carries a real mechanical clause the engine cannot compute at
// character-generation time (botch dice, scene-contingent modifiers, GM
// judgement), so the displayed rules text is that rule's only carrier.
export type Classification = 'narrative' | 'uncomputed_rule' | 'creation_effect' | 'in_play_effect';

export interface PointItem {
  id: string;
  kind: ItemKind;
  magnitude: Magnitude;
  // Every grouping category the rulebook descriptor lists, in its own order.
  // Mirrors the engine's `PointItem` (`crates/arm-rules/src/types.rs`), which
  // rejects an empty list at load. All of them are equally real: membership tests
  // (filters, grant constraints) read the whole list, and so does the Available
  // picker's grouping — the book's own indexes list a dual-category item under
  // both headings. `categories[0]` carries no rules meaning; it is only a
  // tie-break where exactly one bucket is structurally required (a Selected row,
  // whose removal is addressed by index) or one label is (the badge order, the
  // `category_not_permitted` message parameter).
  categories: string[];
  // Headings the book's own INDEX files this entry under beyond its membership
  // `categories`. Provenance, not membership — the engine's
  // `validate_house` Hermetic-Flaw guideline is its only reader, and no UI
  // surface may render, filter, group or badge by it (it is a raw slug and it
  // is not what the character *is*). Omitted from JSON when empty, which is the
  // case for all but a handful of entries.
  index_categories?: string[];
  classification: Classification;
  // Descriptor "Type" tag: a Tainted (Infernal-associated) V/F. Omitted when false.
  tainted?: boolean;
  // D42/D70/D74: how this entry's realm association resolves beyond the plain
  // override/concept/Magic chain. Omitted for every Supernatural entry the
  // book leaves free, and for a Tainted entry (which resolves Infernal from
  // `tainted` alone, ArMDE:3000, never storing the same fact twice).
  realm_association?: RealmAssociation;
  // Entity kinds this item may be selected for. Omitted when empty, which the
  // engine reads as "any kind".
  entity_kinds?: EntityKind[];
  prerequisites?: Prereq;
  // A HEDGED prerequisite ("generally", "normally" restricted) — F-550/D16.
  // Evaluated the same way as `prerequisites`, but a failure is reported as
  // the `advisory_prereq_not_met` warning rather than the `prereq_not_met`
  // error; unevaluable stays silent (no `..._unevaluated` twin). Omitted when
  // absent, which is every entry except `flaw.vendetta`.
  advisory_prerequisites?: Prereq;
  // Items that may not be selected alongside this one. Symmetric (the engine
  // rejects a ruleset whose declarations are one-sided) and omitted when empty.
  incompatible_with?: string[];
  // X6a/e7: while `gate` holds for THIS item's own selection, every id in
  // `forbids` becomes incompatible with it — a per-VALUE extension of the flat
  // `incompatible_with` above (Warped Senses excludes Keen Vision only when
  // its `sense` param names sight). Omitted when empty. Mirrors the engine's
  // `PointItem::conditional_incompatible_with`.
  conditional_incompatible_with?: ConditionalIncompatibility[];
  // This item is illegal while ANY OTHER effective (bought or granted)
  // selection satisfies one of these predicates (D23/B3) — one-directional,
  // unlike `incompatible_with`. Omitted when empty.
  excluded_if_holds?: ItemPredicate[];
  // D69.6: this item may not be held alongside another selection that
  // resolves to the SAME target as this one (Student of (Realm) vs Puissant
  // Ability for the same Lore; Academic Concentration (Artes Liberales) vs
  // Puissant Artes Liberales). Declared one-directionally, like
  // `excluded_if_holds` above. Omitted when empty. Mirrors the engine's
  // `PointItem::same_choice_exclusions`.
  same_choice_exclusions?: SameChoiceExclusion[];
  parameters?: ParameterDef[];
  // D81.8/Q3: groups of this item's own parameter keys that together name ONE
  // unordered "combination" for duplicate-detection purposes (Incompatible
  // Arts: `[["technique_1","form_1"],["technique_2","form_2"]]`). Omitted for
  // every item except Incompatible Arts today. Duplicate detection itself is
  // engine-side only; `ParameterPicker.svelte::paramGroups` is the one
  // frontend reader (review-ui-today finding 2), reordering an item's
  // `parameters` for DISPLAY — Technique before Form within each group — since
  // the JSON array itself must stay alphabetical (canonical serialization).
  // Mirrors the engine's `PointItem::unordered_param_groups`
  // (`crates/arm-rules/src/types.rs`).
  unordered_param_groups?: string[][];
  // D81.14: an entry Merinita's conditional Warping Point (ArMDE:2280) reads
  // as "faerie-related" without carrying a Faerie realm association (Faerie
  // Friend, Faerie Upbringing, Susceptibility to Faerie Power). Omitted when
  // false. No frontend reader: the Warping Point is computed engine-side.
  // Mirrors `types.rs::PointItem::faerie_related`.
  faerie_related?: boolean;
  effects?: Effect[];
  // Max selections per (id, params) target. Omitted when the default (1).
  max_per_target?: number;
  // Max copies of this item TOTAL, across every distinct parameter target,
  // counting granted copies — distinct from `max_per_target`, which caps copies
  // sharing one identical (id, params) target. D10: omitted means the
  // engine's default of 1 (once), not "no ceiling" — an item that legitimately
  // repeats across targets carries an explicit 255. See
  // `derive.ts::atMaxTotalRefs`'s doc comment. Mirrors the engine's
  // `PointItem::max_total` (`crates/arm-rules/src/types.rs`).
  max_total?: number;
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

// The maximum learnable spell level for one Technique/Form/range-class
// combination (Te + Fo + Int + Magic Theory + 3, plus any flat lab_total_mod
// (D1), further halved for a Short-Ranged-Magic holder when
// `range_beyond_touch` is set (D28)). Mirrors the engine's `SpellLevelCap` —
// two rows per Technique/Form pair, one per range class.
export interface SpellLevelCap {
  technique: string;
  form: string;
  range_beyond_touch: boolean;
  cap: number;
}

// A per-CATALOGUE-SPELL level cap (D81.5/X11b): unlike `SpellLevelCap`'s
// Te/Fo/range-keyed grid, this folds the spell's OWN requisites, so two
// spells sharing a Te/Fo pair but different requisites can report different
// caps — the per-spell cap the picker greys a spell BY. Mirrors the engine's
// `SpellCap`. `within_focus_cap` is the same cap with the Magical Focus
// doubling (ArMDE:4403) applied, present only when the entity holds a
// Magical Focus at all; the picker's "add within focus" action offers
// itself only when a candidate level fits `within_focus_cap` but not `cap` —
// the engine cannot match a spell to a player's free-text focus itself, so
// the player decides via that action.
export interface SpellCap {
  spell: string;
  cap: number;
  within_focus_cap?: number;
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
  // One example instance the rules themselves name ("latin"), as a language-neutral
  // slug labelled through `exemplar.<slug>` in the rules i18n. Present because the
  // enforced check is deliberately wider than the rules' letter — the rules say
  // "Latin 1", the engine can only demand any Dead Language. See
  // `requirementAbilityLabel` in derive.ts.
  exemplar?: string;
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
// Second Sight 1). Mirrors the engine's `AbilityFloor`. `parameter` (F-63,
// Enchanting Ability) names the ONE instance a parameter-bound grant applies
// to — absent for a plain, unparameterized grant.
export interface AbilityFloor {
  ability: string;
  parameter?: string;
  floor: number;
}

// A once-only Virtue/Flaw item's own parameter, Bound to one Ability in a
// currently-active effect the character holds. Mirrors the engine's
// `LinkTarget` (design-cv-catalogued-values.md § 6.3).
export interface LinkTarget {
  item: string;
  param: string;
  // The item's OWN current value for `param` (design § 6.4), resolved by the
  // engine so the picker never looks this up itself — `null` when the
  // parameter is declared but not yet filled in.
  resolved: string | null;
}

// The engine-built parameter-picker options for one catalogued-or-linkable
// Ability — the UI must not derive any of this itself. Mirrors the engine's
// `AbilityParameterOptions` (design-cv-catalogued-values.md § 6.3). `catalogued`
// holds ids only — the UI localizes the name through the i18n it already has,
// never rendering a raw id. `hint` (§ 11 item 2) is set only when a bought
// free-text value on this ability leaves a Literal or Bound instance unmet;
// the UI never queries the ruleset to decide this itself. Consumed by
// `AbilityTab.svelte`'s combo box (CV7).
export interface AbilityParameterOptions {
  ability: string;
  catalogued: string[];
  linked: LinkTarget[];
  hint: boolean;
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
// (`LifeStageBlock`). A REAL magus's own apprenticeship is absent on purpose:
// whichever block funds anything the character may learn is the general pool and
// needs no slug, and for a magus that is apprenticeship. Later life is here because
// for a magus it is restricted — Abilities only, never an Art. `apprenticeship` here
// is a DIFFERENT thing (D40/D2): an apprenticeship-SHAPED restricted pool for a
// character who is not hermetically trained by profile (Redcap, Lone Redcap) — it
// does not fund an Art either, so it needs a slug the real magus block does not.
export type LifeStageBlock =
  | 'childhood_native_language'
  | 'childhood_spread'
  | 'later_life'
  | 'apprenticeship';

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
  // Years of a truncated apprenticeship this character completed before
  // abandonment (D56/D62/D3) — 0 for anyone not carrying such a selection,
  // magi included. Distinct from apprenticeship_years above, which is real
  // magus apprenticeship only.
  truncated_training_years: number;
  // Experience the truncated years grant, folded into the general pool
  // alongside apprenticeship_xp/post_gauntlet_xp.
  truncated_training_xp: number;
  // Spell levels the truncated years grant, folded alongside
  // post_gauntlet_spell_levels.
  truncated_training_spell_levels: number;
  // D64: years lived AFTER the truncated block, up to the character's own
  // age — 0 for anyone not funding a truncated apprenticeship.
  truncated_training_post_span_years: number;
  // Experience the post-span years grant — GENERAL (Arts or Abilities
  // alike), unlike the pre-span's Abilities-only later_life_xp above.
  truncated_training_post_span_xp: number;
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
  // The Virtue/Flaw aging-roll modifier, ADDED with its stored sign. The book's
  // three-line formula does not name it; `fixed_total` has always included it, so
  // the formula read-out needs it to add up (#22).
  trait_modifier: number;
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
  // Virtue/Flaw Selections granted onto the entity (derived, never persisted), in
  // declared grant order, so the V/F view renders them read-only. Fed by the
  // engine's `entity_grants` (`crates/arm-app/src/ruleset_io.rs`), which folds
  // together House grants, Mythic Companion type grants, `grants_selection`
  // grants (a bought item granting a further free item), and warping grants —
  // NOT House grants alone.
  granted_selections: Selection[];
  // The character type's virtue/flaw point ceilings, so the balance bar shows the
  // budget without recomputing it here.
  virtue_budget: number;
  flaw_budget: number;
  // The virtue/flaw points actually spent — the engine's own compute_balance
  // figure (the same one the Markdown export and the over-budget/unbalanced-
  // Virtues validation issues already read), so the balance bar never
  // re-derives it. Engine-authoritative; never recomputed here.
  virtue_points: number;
  flaw_points: number;
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
  // D81.5: the per-catalogue-spell level cap, folding each spell's own
  // requisites and, when the entity holds a Magical Focus, the focus-doubled
  // figure — what the picker's per-spell grey/offer logic reads, since only
  // this folds a candidate spell's own requisites (`spell_level_caps` above
  // stays the Te/Fo-keyed grid, a requisite-free baseline figure). Empty for
  // a non-magus, exactly like `spell_level_caps`. Engine-authoritative.
  spell_caps: SpellCap[];
  // The Hermetic minimum-Ability checklist: what the Order demands (ArMDE:2437) and
  // what the rulebook recommends (ArMDE:2451-2461), each with the character's bought
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
  // Reputation grants the character's V/F confer (kind + score), so the UI only
  // offers a Reputation add-control when one exists.
  reputation_grants: ReputationGrant[];
  // Derived Warping: the unified total of stored Warping Points plus any granted
  // by V/F, with the score derived by inverting the advancement curve (15 points
  // 2). 0/0 when there is no Warping. Engine-authoritative; never recomputed here.
  warping_score: number;
  warping_points: number;
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
  // The Affinity on every Spell-Mastery Advancement Total, as the authored
  // [num, den] pair ("counts as num/den of itself"; Flawless Magic [2, 1], which
  // halves each mastery point's XP cost); null when no grant reduces the cost.
  // The ratio crosses rather than a "doubled" flag so the mastery accounting
  // matches the engine's charge for ANY pair the catalogue authors.
  spell_mastery_advancement_affinity: [number, number] | null;
  // The being's effective Might Score + Realm (base + same-Realm grants), or null
  // for an ordinary character. Engine-authoritative; never recomputed in JS.
  might: MightScore | null;
  // Derived power-levels budget the being's Might Virtues grant; 0 when none.
  power_levels_budget: number;
  // Total power level the being's powers consume — the "used" side of the bar.
  power_levels_used: number;
  // Derived Focus Power point pool (25 per copy of the Virtue); 0 when none.
  focus_points_budget: number;
  // Points the focus powers consume: 2 per level of effect + 1 per Penetration.
  focus_points_used: number;
  // The creation phases actually in force for THIS entity, in the profile's own
  // declared order — the type profile's `creation_phases`, each conditional
  // entry resolved against the entity's own selections (A2/D56). The tab list
  // (`App.svelte`) intersects its static tab metadata against this list rather
  // than re-deriving "is this trained/an Order member" from the bare profile
  // flags itself.
  phases_in_force: CreationPhase[];
  // CV7: the Ability parameter picker's engine-built option list, one entry per
  // catalogued-or-linkable Ability (design § 6.3). The UI must not derive this
  // itself — see `AbilityParameterOptions`'s own doc comment.
  ability_parameter_options: AbilityParameterOptions[];
  // D42/D70/D74: the resolved realm for every bought selection that carries
  // one, engine-authoritative — the UI must not re-implement `resolve_realm`'s
  // chain here. Empty for an entity holding no such entry.
  realm_associations: ResolvedRealmEntry[];
  // Row 55 (D74.4): the resolved realm for every GRANTED selection
  // (`EffectiveScores.granted_selections`) that carries one — a House, a
  // Mythic Companion type, or a `grants_selection` Virtue/Flaw grant can
  // state its own realm independent of the granted item's. `index` here is
  // NOT a position in `granted_selections` — it is the 0-based count of
  // earlier entries in `granted_selections` sharing the same `item_ref` (the
  // first copy of a possibly-repeated grant is 0, the second 1, …), because
  // that count survives being re-filtered into a Virtues/Flaws column
  // (`derive.ts::grantedSelectionsForSide`) the way an absolute list
  // position would not. `fixed` is always `true`: a granted row has no
  // stable `entity.selections` index for the player to attach an override
  // control to. Empty for an entity holding none.
  granted_realm_associations: ResolvedRealmEntry[];
}

// One selection's resolved D42 realm association. `fixed` means the book
// states it outright (or the item is Tainted): the V/F row shows it
// read-only rather than offering an override control. Mirrors the engine's
// `arm_app::effective_dto::ResolvedRealmEntry`.
export interface ResolvedRealmEntry {
  // For `EffectiveScores.realm_associations`: the selection's position in
  // `Entity.selections` — a repeatable item (Folk Magic) can appear more than
  // once, so this is the row key, not `item_ref` alone. For
  // `EffectiveScores.granted_realm_associations` this is a DIFFERENT
  // numbering — see that field's own comment.
  index: number;
  item_ref: string;
  realm: Realm;
  fixed: boolean;
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
  // D79: the Potent-Magic-field figure, independent of `within_focus` — `null`
  // unless the character holds a Potent Magic Virtue. Before D79, Potent
  // Magic's bonus wrongly folded into `within_focus` (gated on a Magical
  // Focus instead); the two are now separate figures, non-null independently.
  within_potent_field?: number | null;
  deficient: boolean;
  // `total` (Deficient-halved), halved again for Weak Enchanter — the figure
  // to use when creating or investigating an enchanted item. Equal to `total`
  // for anyone without the Flaw. Mirrored from the Rust `LabTotal.enchanting`
  // field (`crates/arm-rules/src/derived/lab.rs`).
  enchanting: number;
  // D81.8: whether this cell's `(Technique, Form)` pair is one of a held
  // Incompatible Arts Flaw's two barred combinations (ArMDE:6290-6292). The
  // numeric totals above are still computed and must NOT be read as 0 when
  // this is true — 0 is a legitimate Lab Total and would be indistinguishable
  // from this. Mirrors the Rust `LabTotal.unusable` field.
  unusable: boolean;
}

// The within-focus counterparts of a CastingTotal's four cast types.
export interface CastingWithinFocus {
  focus_art: number;
  formulaic: number;
  ritual: number;
  spontaneous_fatiguing: number;
  spontaneous_non_fatiguing: number;
}

// The within-Potent-Magic-field counterparts of a CastingTotal's four cast
// types (D79). Unlike CastingWithinFocus there is no doubled Art — Potent
// Magic is a flat bonus, not a doubling.
export interface CastingWithinPotentField {
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
  // The flat `CastingTotalMod` reaching each cast type, one labelled addend per
  // scope (`casting_mod_formulaic` / `_ritual` / `_spontaneous`), always present
  // including at 0. Separate from `addends` because the modifier is per scope
  // while `addends` is the one breakdown shared by all four cast types — Method
  // Caster's +3 reaches Formulaic and Ritual but not Spontaneous. Mirrored from
  // the Rust `CastingTotal.casting_mod_addends` field
  // (`crates/arm-rules/src/derived/casting.rs`).
  casting_mod_addends: Addend[];
  formulaic: number;
  ritual: number;
  spontaneous_fatiguing: number;
  spontaneous_non_fatiguing: number;
  within_focus?: CastingWithinFocus | null;
  // D79: the Potent-Magic-field variants, independent of `within_focus` —
  // gates the UI's second per-spell toggle. `null` unless the character holds
  // a Potent Magic Virtue; non-null independently of whether `within_focus`
  // is also non-null (a character may hold either Virtue, both, or neither).
  within_potent_field?: CastingWithinPotentField | null;
  non_standard: NonStandardCasting;
  deficient: boolean;
  // D81.8: whether this cell's `(Technique, Form)` pair is one of a held
  // Incompatible Arts Flaw's two barred combinations (ArMDE:6290-6292), same
  // reasoning as `LabTotal.unusable`. A requisite spell may still be unusable
  // through this cell without the cell itself being flagged — see
  // `DerivedTotals.spell_casting_unusable`, which the per-spell figure reads
  // instead. Mirrors the Rust `CastingTotal.unusable` field.
  unusable: boolean;
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
  // True for the mounted twin of this line (K3): Attack/Defense add
  // min(Ride, 3). Never part of a save — CombatLine is a derived read-out.
  mounted?: boolean;
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

// One Focus Power's derived figures. Display-only: the magnitude of the maximum
// level of effect (ArMDE:9097), the Initiative it gives (Quickness − that
// magnitude, ArMDE:3899), and the Fatigue levels activating it costs — null above
// level 75, where the rulebook states no cost (ArMDE:3901).
export interface FocusPowerLine {
  name: string;
  max_level: number;
  penetration: number;
  magnitude: number;
  initiative: number;
  fatigue_levels?: number | null;
}

// A surfaced-only modifier (listed, not simulated).
export interface SurfacedModifier {
  family: string;
  detail: string;
  amount: number;
  // Set only for an Advancement row backed by a `factor` (D55): the row is
  // multiplicative, and `amount` carries no meaning for it. Absent for every
  // other family, so `amount: 0` keeps its original "no magnitude" reading
  // for them (e.g. Unaging's mode-toggle rows).
  factor?: AdvancementFactor;
  // The id of the Virtue/Flaw/other item that produced this row (D45,
  // F-423): rendered through the label map as its localized display name,
  // never the raw id, so two carriers of the same family+detail pair (two
  // Flaws both granting `special_casting: circumstantial`) read as two
  // distinguishable lines. Absent only for `health_roll`: the engine sums
  // every contributing selection's amount into one number before this
  // family is surfaced, so there is no single item left to name.
  source?: string;
  // The FIXED Ability a fixed-target `ability_roll_mod` row targets
  // (coordinator review, post-B5-phase-1): resolved through the same
  // ruleset-i18n path `abilityLabel` uses for a bought Ability's own name —
  // never rendered from `detail`, which stays reserved for a parameter-based
  // row's free-text subject (Academic Concentration). Absent for every other
  // family, and for the parameter-relative `ability_roll` row.
  ability?: string;
}

// The full read-only play-stat read-out returned by the `derived_totals` command.
export interface DerivedTotals {
  // D56/A0: renamed from `is_magus`, and no longer just a real magus — reads
  // true for any entity `is_hermetically_trained` (union with an entity-level
  // selection effect), e.g. a future Abandoned Apprentice test fixture.
  hermetically_trained: boolean;
  lab_totals: LabTotal[];
  casting_totals: CastingTotal[];
  // D79: per-known-spell Casting Total, index-aligned with `entity.spells` —
  // the engine's own `spell_casting_total` computation (combines both the
  // within-focus and within-potent-field markers directly, which a
  // client-side reconstruction from the three grid figures above cannot do
  // correctly under Deficient-Art halving: halve(a) + halve(b) != halve(a + b)).
  // `null` for a row whose spell id is absent from the catalogue. The UI must
  // read this rather than re-derive the figure itself.
  spell_casting_totals: (number | null)[];
  // D81.8: whether each known spell touches one of a held Incompatible Arts
  // Flaw's two barred `(Technique, Form)` combinations — as its primary Arts
  // or only through a requisite ("even if one or both are requisites",
  // ArMDE:6292). Index-aligned with `spell_casting_totals` and
  // `entity.spells`, for the same reason that field is; a parallel `Vec`
  // rather than widening `spell_casting_totals`'s element, so every existing
  // reader of that field stays untouched. Mirrors the Rust
  // `DerivedTotals.spell_casting_unusable` field.
  spell_casting_unusable: boolean[];
  penetration: PenetrationLine[];
  magic_resistance: MagicResistance[];
  longevity?: LongevityBonus | null;
  masterpiece?: MasterpieceCap | null;
  talisman_capacity?: TalismanCapacity | null;
  familiar?: FamiliarReadout | null;
  focus_powers?: FocusPowerLine[];
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

// A Reputation slot a Virtue/Flaw opens. One grant = one row in the Reputations
// panel, so there is nothing to press twice. `kind` is null for a
// player-chosen-type grant (Famous), which renders as a type `<select>`;
// `source` is the id of the granting Virtue/Flaw, shown through its i18n name so
// the row can say WHY it is there. `max_score` is the upper bound of a stated
// range (D11/Q5) — absent (undefined) means the score is exact. Outsider is
// the one grant in the catalogue that states a range ("a bad Reputation of
// level 1 to 3", ArMDE:6554).
export interface ReputationGrant {
  source: string;
  kind: ReputationType | null;
  score: number;
  max_score?: number;
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

// One entry on a profile's `permitted_categories` / `forbidden_categories`
// list. Mirrors the engine's untagged `CategoryRule`: a bare slug is
// unconditional, and the object form applies only while `when` holds
// (`ArMDE:2840` — a companion may take Hermetic Virtues and Flaws "unless you have
// The Gift"). Enforcement lives entirely in the engine; the frontend never
// re-implements the resolution, and never renders either the slug or the
// condition as a label.
export type CategoryRule = string | { category: string; when: Prereq };

// One entry on a profile's `creation_phases` list. Mirrors `CategoryRule`
// exactly (A2/D56): a bare phase is unconditional, and the object form applies
// only while `when` holds (the Arts/Spells phases for a type that may be
// Hermetically trained by selection rather than by profile, e.g. the
// Abandoned Apprentice companion). The frontend never re-implements the
// resolution — it reads `EffectiveScores.phases_in_force`, the engine's own
// already-resolved list, rather than evaluating `when` itself.
export type PhaseRule = CreationPhase | { phase: CreationPhase; when: Prereq };

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
  permitted_categories?: CategoryRule[];
  forbidden_categories?: CategoryRule[];
  // Item ids that must / may never be selected. Omitted from JSON when empty.
  required_traits?: string[];
  forbidden_traits?: string[];
  // Whether this character type is Hermetically trained. Omitted from JSON
  // when false (the common case), so optional here. D56/A0 split the old
  // `is_magus` into this and `order_member`; this field alone has no
  // equivalent of the engine's entity-level union
  // (`is_hermetically_trained`) — that reaches the frontend separately, as
  // `DerivedTotals.hermetically_trained` (sub-slice 6). What is still missing
  // is a resolved, phases-shaped equivalent for tab *visibility* — see
  // `docs/vf-audit/design-a0-is-magus-split.md` § 6 (A2).
  hermetically_trained?: boolean;
  // Whether this character type is a full member of the Order of Hermes. The
  // other half of the old `is_magus` flag; every shipped profile sets it equal
  // to `hermetically_trained` today.
  order_member?: boolean;
  // Whether this character type counts as a companion for the `is_companion`
  // Prereq (D38): true for `companion` and `mythic_companion` — "mythic
  // companions are companions too" — a capability flag parallel to
  // hermetically_trained, never the exact type id. Omitted when false.
  is_companion?: boolean;
  // Whether this character type counts as a grog for the `is_grog` Prereq
  // (D68.9): true only for `grog` today. A capability flag parallel to
  // is_companion, never the exact type id. Omitted when false.
  is_grog?: boolean;
  // Whether this type chooses a Mythic Companion type (free status/Minor Virtue
  // + required package). Capability flag parallel to hermetically_trained. Omitted when false.
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
  // The V/F categories that count as carrying The Gift (the magus profile names
  // `hermetic`). Omitted from JSON when empty, so optional here.
  gift_categories?: string[];
  // The V/F categories whose Flaws satisfy the "at least one Hermetic Flaw"
  // guideline (`ArMDE:2860`); `hermetic` on the magus profile only. A separate field
  // from `gift_categories` on purpose — "is this character Gifted?" and "does
  // this Flaw count as Hermetic?" are different questions with, for the two
  // Beings Flaws, different answers. No frontend code reads either field today;
  // when the guided wizard advises on Hermetic Flaws it must key off THIS one,
  // exactly as the engine's `missing_hermetic_flaw` does.
  hermetic_flaw_categories?: string[];
  // Ordered: the guided wizard walks these in sequence. Typed in the engine too,
  // so a phase string it has no variant for fails the ruleset load. Each entry
  // is a `PhaseRule` (A2/D56): a bare phase, or one conditional on `when`.
  creation_phases: PhaseRule[];
}

export interface I18nEntry {
  name: string;
  // The name to show when a `{token}` template holds no instance yet. Opt-in and
  // normally absent: the generic "(Ability)" hint is right for almost every
  // template. Only a name carrying BOTH a token and a parenthetical literal doubles
  // ("{language} (Dead Language)" + "(Language)"), and those two entries name their
  // unfilled form here. See `displayName` in derive.ts.
  name_unfilled?: string | null;
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
  // The spell's Range (RDT chart) — 'personal' | 'touch' | 'eye' | 'voice' |
  // 'sight' | 'arcane_connection'. Mirrors the engine's `SpellRange`. Drives
  // Short-Ranged Magic's beyond-Touch cap halving (D28), folded server-side
  // into this spell's own `EffectiveScores.spell_caps` row (D81.5) — nothing
  // in the UI re-derives it from this field.
  range?: string | null;
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
  // A hard ceiling on picks per spell, tighter than plain `repeatable`. Set only
  // for Quiet Casting (2); absent = no ceiling beyond `repeatable`.
  max_count?: number;
  // Whether this ability may not be chosen for a Ritual spell (Ceremonial, Fast,
  // Quick Casting). Absent = false.
  forbidden_for_ritual?: boolean;
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
  // X10c: the player's own claim that this known spell falls within the
  // character's Magical Focus (ArMDE:4399-4422) — free-text `focus` can't
  // supply this (MAG8's capability gap). Absent/false = not claimed. Feeds
  // `derived/casting.rs::spell_casting_total`'s selector; harmless if the
  // character holds no Focus.
  within_focus?: boolean;
  // D79: the player's own claim that this known spell falls within the
  // character's Potent Magic field (ArMDE:4740-4748) — same shape as
  // `within_focus` and independent of it, since the two free-text themes
  // need not coincide. Absent/false = not claimed. Harmless if the character
  // holds no Potent Magic Virtue.
  within_potent_field?: boolean;
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

// D42/D70/D74: how a Supernatural PointItem's realm association resolves
// beyond the plain override/concept/Magic chain. Mirrors the engine's
// `RealmAssociation` (`crates/arm-rules/src/types.rs`), tagged by `kind`.
export type RealmAssociation =
  | { kind: 'fixed'; realm: Realm }
  | { kind: 'default'; realm: Realm }
  | { kind: 'subset'; realms: Realm[] }
  | { kind: 'from_param'; key: string };

// A supernatural being's base Might Score + Realm (Virtue grants add on top).
export interface MightScore {
  realm: Realm;
  score: number;
}

// A supernatural power a Might-being holds. `level` is charged against the
// power-levels budget the being's Might Virtues grant (like a device vs item level).
// `penetration` is charged against the SAME budget — levels are spent on it
// one-for-one (ArMDE:4019) — and is absent from the save when unspent.
export interface SupernaturalPower {
  name: string;
  level: number;
  penetration?: number;
}

// A Focus Power, bought from the Focus Power Virtue's own 25-point pool: 2 points
// per point of `max_level`, 1 per point of `penetration` (ArMDE:3899). Deliberately
// NOT a SupernaturalPower — `max_level` is the *maximum level of effect* the
// character may create, a ceiling, where a SupernaturalPower's `level` is a level
// that was spent out of the level budget. `penetration` is absent when unspent.
export interface FocusPower {
  name: string;
  max_level: number;
  penetration?: number;
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
  // True for an unarmed strike (Dodge/Fist/Kick) — K3's mounted-twin gate
  // reads this alone (D66), never min_strength or any other field.
  body_attack?: boolean;
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

// How a piece of equipment is currently carried (K5). Replaces the K2-era
// `equipped: boolean`, which conflated two independent facts: whether the slot
// yields a Combat row, and whether it contributes Load. A `Carried` weapon (the
// Knight's own spare great sword) yields a Combat row but no Load; `Wielded` is
// K2's old `equipped: true` behavior unchanged; `Stowed` is the default.
export type LoadoutState = 'stowed' | 'carried' | 'wielded';

// A piece of equipment the character carries: a reference to a catalogue weapon,
// shield, or armor id, plus how it is currently carried (K5).
export interface EquipmentSlot {
  item: string;
  loadout?: LoadoutState; // omitted = 'stowed'
  // Whether this weapon's combat Ability specialization applies to it, granting
  // +1 to the weapon's Attack and Defense (ArMDE:7122, :7139). Additive/optional.
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
// and each `required_flaws[].default` count against the budget. A type never
// changes the size of that budget — every Mythic Companion gets the same 10 Flaw
// / 20 Virtue points. Mirrors the engine's `MythicCompanionType`;
// name/description live in the rules i18n map.
export interface MythicCompanionType {
  id: string;
  grants?: Grant[];
  required_virtues?: Selection[];
  required_flaws?: RequiredFlaw[];
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
  // The age a plan naming no Gauntlet age is read at — the rules' own baseline
  // magus, "25 years old and just out of apprenticeship". Data, so the Gauntlet-age
  // field's placeholder says what blank means without hardcoding a number. Absent
  // for a ruleset stating no baseline, where blank still means "stands at its
  // Gauntlet".
  default_gauntlet_age?: number;
  // Abilities without which a magus "would not be admitted to the Order".
  minimum_abilities: AbilityRequirement[];
  // The recommended package, priced by `recommended_xp`.
  recommended_abilities: AbilityRequirement[];
  recommended_xp: number;
  xp: number;
  years: number;
  // D56/D3: the truncated apprenticeship's fixed per-year rates (16 XP, 8
  // spell levels), derived from xp/years and the magus profile's own
  // spell_levels/years — FIXED, not a house rule.
  truncated_xp_per_year: number;
  truncated_spell_levels_per_year: number;
}

// An Ability score a rule demands. Mirrors the engine's `AbilityRequirement`;
// `parameter` narrows the demand to one instance of a parameterized Ability and is
// unset throughout the shipped data (the match is by Ability id).
export interface AbilityRequirement {
  ability: string;
  // A label key, not a ref: one example the rules name in prose, resolved through
  // `exemplar.<slug>` in the rules i18n. See `MagusMinimumAbility.exemplar`.
  exemplar?: string;
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
  // Reputation types in book order, mirrored from the Rust `ReputationType::ALL`,
  // so the wildcard-grant `<select>` offers the engine's taxonomy instead of a
  // list re-hardcoded in Svelte. Optional like `art_type_order`, for the same
  // reason: a live payload always sends it, but requiring it would force every
  // hand-built `Ruleset` test fixture to grow a field it does not exercise.
  reputation_type_order?: ReputationType[];
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
  // C0b (docs/vf-audit/design-c0-parameter-model.md): a value is either a
  // single string (every value this engine produces today) or a string array
  // (D9 part 3's multi-valued parameter — not yet producible; C5b is the
  // first slice with a picker that writes one). Wire-compatible: a plain
  // string still parses exactly as before.
  params?: Record<string, string | string[]>;
}

// A parameterized Ability's player-supplied value (CV4, D14;
// `docs/vf-audit/design-cv-catalogued-values.md` § 3): three disjoint shapes,
// discriminated structurally by which key is present — mirrors
// `arm_rules::types::AbilityParameterValue`. `Linked` is written and resolved
// by the picker's combo box (CV7, `AbilityTab.svelte`).
export type AbilityParamValue = { id: string } | { item: string; param: string } | { text: string };

// A whole bought Ability score with an optional free-text specialty. Keyed by
// (ability, specialty): the same parameterized ability may appear more than once.
export interface AbilityScore {
  ability: string;
  score: number;
  specialty?: string | null;
  // Player-supplied value for a parameterized ability (e.g. the area for
  // (Area) Lore). Part of the instance identity, so several can coexist.
  parameter?: AbilityParamValue | null;
  // X10b: XP already banked toward the next score, in raw table-XP currency —
  // not charged XP, not the entity's `xp_pool`. The book's own "X (Z)"
  // notation (ArMDE:1177): X = `score`, Z = this field. Absent/0 = none.
  banked_xp?: number;
}

// A whole bought Art score (magi only). Arts are not parameterized and carry no
// specialty, so an Art is identified by id alone.
export interface ArtScore {
  art: string;
  score: number;
  // X10b: same shape as `AbilityScore.banked_xp` (ArMDE:1179's "Art X (Z)").
  banked_xp?: number;
}

/**
 * Where a character's Ability/Art experience comes from: a typed `xp_pool` (direct
 * entry) or the blocks its life stages earn (the guided flow). The slugs mirror
 * `arm_rules::AbilityFunding`'s snake_case serde names exactly.
 *
 * Lives here rather than in `state.svelte.ts` because `Entity` carries it, and
 * `types.ts` must not import from the store (that would be a cycle). The store
 * re-exports it, so `import { type AbilityFunding } from '../state.svelte'` keeps
 * working.
 */
export type AbilityFunding = 'pool' | 'life_stages';

export interface Entity {
  schema_version: number;
  ruleset: RulesetRef;
  entity_kind: EntityKind;
  type_id: string;
  // The chosen Virtues/Flaws. Omitted when empty (a character with none at all —
  // a bare grog — carries no key), so every read must be defensive.
  selections?: Selection[];
  // Chosen Characteristic scores (point-buy). Omitted when empty.
  characteristics?: Partial<Record<Characteristic, number>>;
  // Optional free-text description per Characteristic (sheet flavor). Omitted empty.
  characteristic_descriptions?: Partial<Record<Characteristic, string>>;
  // Whole bought Ability scores. Omitted when empty.
  ability_scores?: AbilityScore[];
  // Total XP available to spend on Abilities AND Arts — one shared bank. Spent
  // is derived (ability + art cost), leftover is the banked XP. Omitted when zero.
  xp_pool?: number;
  // The character's life-stage choices (childhood + later life), read only under
  // `ability_funding: 'life_stages'`. Omitted when no plan has ever been recorded.
  //
  // Presence is NOT the funding mode — it was until schema 16, and is not now. A
  // plan kept beside pool funding is inert data the player typed and may come back
  // to, so ask `store.abilityFunding` for the mode and this key only for a plan's
  // contents.
  life_stages?: LifeStagePlan;
  // Which of `xp_pool` / `life_stages` funds this character's Abilities and Arts.
  //
  // REQUIRED, deliberately, unlike almost every other field here. Tauri commands
  // receive an entity through plain serde, never through `load_entity_migrating`,
  // so the load-time migration does not run on an IPC payload: an omitted key
  // defaults to `Pool` on the Rust side, `LifeStageRules::budget` returns `None`,
  // and every life-stage pool silently disappears — no error, no finding, just a
  // character whose experience evaporated. The engine writes the key even when it
  // holds its default for the same reason (absence is the pre-16 signal), so the
  // frontend must always send it too.
  ability_funding: AbilityFunding;
  // The furthest guided-wizard phase this character reached, as a raw phase slug.
  // UI/document state that happens to live on the entity because it has to survive
  // a save: nothing validates it and nothing derives from it, and it may hold a
  // slug this build no longer declares (resolve it leniently against the profile's
  // `creation_phases`; unresolvable is treated exactly like absent). Omitted when
  // no wizard progress has been recorded, which is what an editor-built character
  // looks like.
  wizard_furthest_phase?: string;
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
  // D42: the concept's default realm — a default SOURCE for every Supernatural
  // entry's realm, never the character's own realm and never mechanical on its
  // own (concept_realm is free, like every other identity field). Omitted when
  // unset, in which case each entry resolves through its own override, else
  // Magic. See `realm_association` on `PointItem`.
  concept_realm?: Realm | null;
  // The calendar year the saga this document was built for stands in (C8, schema
  // 17). Required, and always written even at its default: `load_entity_migrating`
  // dispatches on the key's ABSENCE to fill a pre-17 save from the configured
  // default, so an omitted key is a migration trigger rather than a tidy elision.
  // It was a machine-global app setting until C8, which made it wrong for every
  // saga but one on a storyguide's machine; `settings.json` now keeps only the
  // default a NEW document starts at.
  saga_year: number;
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
  // The character's Focus Powers, charged against the separate Focus Power point
  // pool (ArMDE:3899). Omitted when empty.
  focus_powers?: FocusPower[];
  // Whether the character is currently fighting mounted (K3): derived combat
  // totals add a second, mounted line per weapon that is not a body attack,
  // adding min(Ride, 3) to Attack and Defense. Omitted = false.
  mounted?: boolean;
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

/**
 * An age the engine derived from a saga year and a birth year
 * (`arm_rules::AgeInSagaYear`, guided-creation-review-2026-08 #25).
 *
 * `issues` is empty for any pair that can really happen; a saga year before the
 * birth year clamps `age` to 0 and advises why.
 */
export interface AgeInSagaYear {
  age: number;
  issues: ValidationIssue[];
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
  | { kind: 'export'; missing: string[] }
  // The native application menu could not be built or installed — a
  // window-system failure, not a file one, so it carries its own kind and its
  // own `error-menu` message rather than borrowing `error-io`'s.
  | { kind: 'menu'; message: string };
