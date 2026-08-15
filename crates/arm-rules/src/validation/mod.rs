//! Entity validation: the single evaluation path.
//!
//! [`validate`] always computes every rule result and returns them as
//! [`ValidationIssue`]s; [`ValidationMode`] governs enforcement at the caller
//! level, not which checks run. The set of
//! emittable issue codes and their interpolation args is documented as a
//! contract on [`ValidationIssue`] for the Fluent frontend.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::characteristics::Characteristic;
use crate::grant::{Grant, open_pick_satisfies};
use crate::ruleset::Ruleset;
use crate::types::{
    CategoryCap, CreationPhase, Effect, Entity, EntityKind, EntityTypeProfile, GiftPolicy, Id,
    ItemKind, Magnitude, ParameterDomain, PointItem, Prereq, Selection, ValidationMode,
};

mod aging;
mod authorization;
mod balance;
mod caps;
mod equipment;
mod life_stage;
mod magus;
mod might;
mod prereq;
mod scores;
mod selections;
mod warping;

use aging::*;
use authorization::*;
use balance::*;
use caps::*;
use equipment::*;
use life_stage::*;
use magus::*;
use might::*;
use prereq::*;
use scores::*;
use selections::*;
use warping::*;

pub use balance::{Balance, PointCeilings, compute_balance, effective_point_ceilings};
pub use life_stage::childhood_rejection_issues;

/// Whether a validation issue blocks (`Error`) or merely advises (`Warning`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// A rule violation that makes the entity illegal.
    Error,
    /// A non-blocking advisory (e.g. a prerequisite that cannot be evaluated yet).
    Warning,
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueSeverity::Error => f.write_str("error"),
            IssueSeverity::Warning => f.write_str("warning"),
        }
    }
}

/// A single validation finding.
///
/// Carries NO user-facing prose: `code` is a stable machine key (also the
/// Fluent message id the UI localizes), and `args` carries the interpolation
/// values (offending ids, counts, budgets). The UI renders `code` + `args`
/// against its `.ftl` catalogue.
///
/// # Issue-code contract (the Fluent contract for the frontend)
///
/// Every `code` the engine can emit, with the creation phase it is attributed to
/// and the `args` keys it carries. The UI must define an `issue-<code>` message
/// for each (see the workspace test
/// `every_validation_code_has_a_fluent_key_in_each_locale`), and the guided wizard
/// filters each step's findings on the phase — so both columns are contract, not
/// commentary, and both are asserted against the source by
/// `contract_table_phase_column_covers_the_creation_phases` and
/// `every_issue_emit_site_names_a_phase_the_table_lists`.
///
/// A code listed under several phases is emitted from several places over
/// different subject kinds; its phase is the caller's, not the code's.
///
/// Three rows describe a **rejected command input** rather than an entity state:
/// `childhood_slot_unfilled`, `childhood_slot_is_native_language`, and
/// `childhood_slot_duplicate_value` come only from [`childhood_rejection_issues`],
/// when applying a Sample Childhood package fails, and never from [`validate`] — a
/// stored character cannot *hold* an unanswered slot, because a rejected
/// application writes nothing. They are listed here all the same: the UI localizes
/// them through the same `issue-<code>` catalogue.
///
/// | `code` | severity | phase | `args` keys |
/// |--------|----------|-------|-------------|
/// | `unknown_type` | error | review | `type_id` |
/// | `unknown_ref` | error | virtues_flaws | `item` |
/// | `wrong_entity_kind` | error | virtues_flaws | `item`, `entity_kind` |
/// | `duplicate_selection` | error | virtues_flaws | `item`, `count`, `max` |
/// | `over_budget_virtues` | error | virtues_flaws | `points`, `budget` |
/// | `over_budget_flaws` | error | virtues_flaws | `points`, `budget` |
/// | `unbalanced_virtues` | error | virtues_flaws | `virtue_points`, `flaw_points` |
/// | `too_many_major_virtues` | error | virtues_flaws | `count`, `max` |
/// | `too_many_major_flaws` | error | virtues_flaws | `count`, `max` |
/// | `too_many_minor_flaws` | error | virtues_flaws | `count`, `max` |
/// | `too_many_tainted_virtues` | warning | virtues_flaws | `tainted`, `total` |
/// | `too_many_tainted_flaws` | warning | virtues_flaws | `tainted`, `total` |
/// | `too_many_major_<category>_flaws`† | error or warning | virtues_flaws | `count`, `max` |
/// | `too_many_<category>_flaws`† | error or warning | virtues_flaws | `count`, `max` |
/// | `too_many_major_<category>_virtues`† | error or warning | virtues_flaws | `count`, `max` |
/// | `too_many_<category>_virtues`† | error or warning | virtues_flaws | `count`, `max` |
/// | `prereq_not_met` | error | virtues_flaws | `item` |
/// | `prereq_unevaluated` | warning | virtues_flaws | `item` |
/// | `incompatible` | error | virtues_flaws | `item`, `other` |
/// | `category_not_permitted` | error | virtues_flaws | `item`, `category` |
/// | `forbidden_category` | error | virtues_flaws | `item`, `category` |
/// | `missing_required_trait` | error | virtues_flaws | `item` |
/// | `forbidden_trait` | error | virtues_flaws | `item` |
/// | `missing_param` | error | virtues_flaws, house_specialisation, mythic_type, spells, review | `item`, `key` |
/// | `unexpected_param` | error | virtues_flaws, house_specialisation, mythic_type, review | `item`, `key` |
/// | `unknown_param_value` | error | virtues_flaws, house_specialisation, mythic_type, spells, review | `item`, `key`, `value`, `domain` |
/// | `gift_required` | error | virtues_flaws | (none) |
/// | `gift_forbidden` | error | virtues_flaws | (none) |
/// | `characteristic_out_of_range` | error | characteristics | `characteristic`, `score`, `min`, `max` |
/// | `characteristic_above_cap` | error | characteristics | `characteristic`, `score`, `cap` |
/// | `characteristic_below_floor` | error | characteristics | `characteristic`, `score`, `floor` |
/// | `characteristic_max_base_too_low` | error | characteristics | `item`, `characteristic`, `base`, `min` |
/// | `characteristic_min_base_too_high` | error | characteristics | `item`, `characteristic`, `base`, `max` |
/// | `characteristic_overspent` | error | characteristics | `cost`, `points` |
/// | `characteristic_points_unspent` | warning | characteristics | `cost`, `points` |
/// | `unknown_ability` | error | abilities | `ability` |
/// | `duplicate_ability` | error | abilities | `ability`, `count` |
/// | `not_enough_xp` | error | abilities | `spent`, `pool`, `shortfall` |
/// | `restricted_xp_unspent` | warning | abilities | `amount`, `used`, `unspent`, `origin_kind`, `origin` |
/// | `ability_category_requires_virtue` | error | abilities | `ability`, `category` |
/// | `academic_ability_without_scholarly_language` | warning | abilities | `ability`, `min` |
/// | `life_stage_xp_pool_conflict` | error | abilities | `xp_pool` |
/// | `life_stage_age_unset` | error | abilities | (none) |
/// | `life_stage_age_before_childhood` | error | abilities | `age`, `min` |
/// | `life_stage_age_before_gauntlet` | error | abilities | `age`, `min` |
/// | `life_stage_gauntlet_age_after_age` | error | abilities | `gauntlet_age`, `age` |
/// | `life_stage_lab_seasons_out_of_range` | error | abilities | `seasons`, `max`, `years` |
/// | `life_stage_spell_level_split_exceeds_points` | error | abilities | `levels`, `points` |
/// | `life_stage_native_language_unset` | error | abilities | (none) |
/// | `life_stage_native_language_missing_score` | warning | abilities | `language` |
/// | `magus_minimum_ability` | error | abilities | `ability`, `min`, `score` |
/// | `magus_recommended_ability` | warning | abilities | `ability`, `min`, `score` |
/// | `childhood_package_unknown` | error | abilities | `package` |
/// | `childhood_slot_unfilled` | error | abilities | `ability`, `key`, `slot` |
/// | `childhood_slot_is_native_language` | error | abilities | `ability`, `key`, `slot`, `language` |
/// | `childhood_slot_duplicate_value` | error | abilities | `ability`, `key`, `slot`, `other_slot`, `value` |
/// | `ability_parameter_required` | error | abilities | `ability` |
/// | `ability_score_out_of_range` | error | abilities | `ability`, `score`, `max` |
/// | `ability_bonus_dangling_target` | error | virtues_flaws | `item`, `ability`, `parameter` |
/// | `unknown_art` | error | arts | `art` |
/// | `duplicate_art` | error | arts | `art`, `count` |
/// | `art_score_out_of_range` | error | arts | `art`, `score`, `max` |
/// | `house_choice_unresolved` | error | house_specialisation | `house`, `choice_key` |
/// | `house_grant_constraint` | error | house_specialisation | `house`, `choice_key`, `item` |
/// | `house_unset` | warning | house_specialisation | (none) |
/// | `missing_hermetic_flaw` | warning | virtues_flaws | (none) |
/// | `mythic_type_unset` | warning | mythic_type | (none) |
/// | `mythic_choice_unresolved` | error | mythic_type | `mythic_type`, `choice_key` |
/// | `mythic_grant_constraint` | error | mythic_type | `mythic_type`, `choice_key`, `item` |
/// | `mythic_required_trait_missing` | warning | virtues_flaws | `item` |
/// | `unknown_spell` | error | spells | `spell` |
/// | `duplicate_spell` | error | spells | `spell`, `count` |
/// | `spell_level_unresolved` | warning | spells | `spell` |
/// | `over_spell_levels` | error | spells | `used`, `budget`, `over` |
/// | `spell_level_exceeds_cap` | error | spells | `spell`, `level`, `cap` |
/// | `ability_above_age_cap` | error | abilities | `ability`, `score`, `cap`, `age` |
/// | `supernatural_ability_requires_virtue` | error | abilities | `ability` |
/// | `personality_trait_out_of_range` | error | personality_reputations | `name`, `value`, `max` |
/// | `reputation_not_granted` | error | personality_reputations | `kind`, `content` |
/// | `over_item_level` | error | review | `used`, `budget`, `over` |
/// | `multiple_magical_foci` | error | virtues_flaws | `count` |
/// | `spell_ritual_legality` | error | spells | `spell`, `level` |
/// | `unknown_mastery_ability` | error | spells | `spell`, `ability` |
/// | `too_many_mastery_abilities` | error | spells | `spell`, `chosen`, `mastery` |
/// | `duplicate_mastery_ability` | error | spells | `spell`, `ability`, `count` |
/// | `over_power_levels` | error | review | `used`, `budget`, `over` |
/// | `might_realm_mismatch` | warning | review | `base`, `granted` |
/// | `excessive_aging_reduction` | warning | review | `characteristic`, `reduction`, `min` |
/// | `life_stage_aging_rolls_pending` | warning | review | `age` |
/// | `unknown_living_condition` | error | review | `condition` |
/// | `living_conditions_conflict` | error | review | `condition`, `other` |
/// | `apparent_age_above_age` | warning | review | `apparent_age`, `age` |
/// | `unknown_equipment` | error | review | `item` |
/// | `equipment_min_strength` | warning | review | `item`, `required`, `strength` |
/// | `shield_with_two_handed_weapon` | warning | review | (none) |
/// | `warping_owed_minor_flaws` | warning | review | `count` |
/// | `warping_owed_supernatural_virtues` | warning | review | `count` |
/// | `warping_owed_major_flaws` | warning | review | `count` |
/// | `warping_fill_constraint` | error | review | `choice_key`, `item` |
/// | `warping_fill_ineligible` | error | review | `choice_key`, `item` |
/// | `warping_fill_excess` | error | review | `choice_key` |
///
/// † The per-category caps emit a code derived from the `flaw_category_caps` /
/// `virtue_category_caps` entry's category slug: `too_many_<category>_flaws` /
/// `too_many_<category>_virtues` (or the `too_many_major_<category>_…` form when
/// the cap is `major_only`); severity follows the cap's `hard` flag and the phase
/// is always `virtues_flaws`. The shipped `personality`/`story` flaw caps thus
/// produce `too_many_major_personality_flaws` (error), `too_many_personality_flaws`
/// (warning), and `too_many_story_flaws` (warning), and the magus `hermetic`
/// virtue cap produces `too_many_major_hermetic_virtues` (error); a new category
/// requires its matching `issue-<code>` Fluent key.
///
/// Two phases appear in no row: `concept` (free text — nothing to violate) and
/// `type` (fixed at creation; a bad one is `unknown_type`, a `review` finding).
/// The test pins that list, so the first code filed under either must update it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Error or warning.
    pub severity: IssueSeverity,
    /// Stable machine key for this issue. The UI derives the Fluent message id
    /// as `issue-<code>` (e.g. code `over_budget_virtues` →
    /// `issue-over_budget_virtues`).
    pub code: String,
    /// The creation phase whose input surface owns the offending value — the step
    /// the guided wizard blocks on, and where the user can actually fix it. Set
    /// per emit site rather than derived from `code`, because several codes
    /// (`missing_param`, `prereq_not_met`, `unknown_param_value`, …) are emitted
    /// over different subject kinds from different modules. Findings no creation
    /// phase owns (equipment, Might, Warping, aging, an unknown type) carry
    /// [`CreationPhase::Review`].
    ///
    /// Always emitted, for the same reason as `args`.
    pub phase: CreationPhase,
    /// Interpolation values for the localized message, keyed by argument name.
    /// Always emitted, even when empty (no `skip_serializing_if`): a
    /// `ValidationResult` is a transient frontend payload, never a git-tracked
    /// file, so the zero-noise-diff rule does not apply; and the UI's
    /// `issue.args` contract (spread into the Fluent interpolation) relies on the
    /// field being a defined object rather than possibly absent.
    #[serde(default)]
    pub args: BTreeMap<String, String>,
    /// The item id this issue is about, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Id>,
}

impl ValidationIssue {
    /// The closed set of fixed issue codes the engine emits, exposed as symbols
    /// so consumers reference a const rather than re-typing a string literal.
    /// The serialized `code` stays a plain `String` (these consts ARE its
    /// values); the per-category flaw-cap codes
    /// (`too_many_<category>_flaws` / `too_many_major_<category>_flaws`) are
    /// derived from data at runtime and so deliberately have no const here (see
    /// the issue-code contract table above).
    pub const CODE_UNKNOWN_TYPE: &'static str = "unknown_type";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_UNKNOWN_REF: &'static str = "unknown_ref";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_WRONG_ENTITY_KIND: &'static str = "wrong_entity_kind";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_DUPLICATE_SELECTION: &'static str = "duplicate_selection";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_OVER_BUDGET_VIRTUES: &'static str = "over_budget_virtues";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_OVER_BUDGET_FLAWS: &'static str = "over_budget_flaws";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_UNBALANCED_VIRTUES: &'static str = "unbalanced_virtues";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_TOO_MANY_MAJOR_VIRTUES: &'static str = "too_many_major_virtues";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_TOO_MANY_MAJOR_FLAWS: &'static str = "too_many_major_flaws";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_TOO_MANY_MINOR_FLAWS: &'static str = "too_many_minor_flaws";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: more than half a
    /// character's Virtue points are Tainted (Core:2998-3002).
    pub const CODE_TOO_MANY_TAINTED_VIRTUES: &'static str = "too_many_tainted_virtues";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: more than half a
    /// character's Flaw points are Tainted (Core:2998-3002).
    pub const CODE_TOO_MANY_TAINTED_FLAWS: &'static str = "too_many_tainted_flaws";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_PREREQ_NOT_MET: &'static str = "prereq_not_met";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_PREREQ_UNEVALUATED: &'static str = "prereq_unevaluated";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_INCOMPATIBLE: &'static str = "incompatible";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CATEGORY_NOT_PERMITTED: &'static str = "category_not_permitted";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_FORBIDDEN_CATEGORY: &'static str = "forbidden_category";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_MISSING_REQUIRED_TRAIT: &'static str = "missing_required_trait";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_FORBIDDEN_TRAIT: &'static str = "forbidden_trait";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_MISSING_PARAM: &'static str = "missing_param";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_UNEXPECTED_PARAM: &'static str = "unexpected_param";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_UNKNOWN_PARAM_VALUE: &'static str = "unknown_param_value";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_GIFT_REQUIRED: &'static str = "gift_required";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_GIFT_FORBIDDEN: &'static str = "gift_forbidden";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CHARACTERISTIC_OUT_OF_RANGE: &'static str = "characteristic_out_of_range";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CHARACTERISTIC_OVERSPENT: &'static str = "characteristic_overspent";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CHARACTERISTIC_POINTS_UNSPENT: &'static str = "characteristic_points_unspent";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_MULTIPLE_MAGICAL_FOCI: &'static str = "multiple_magical_foci";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_UNKNOWN_ABILITY: &'static str = "unknown_ability";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_DUPLICATE_ABILITY: &'static str = "duplicate_ability";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_NOT_ENOUGH_XP: &'static str = "not_enough_xp";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_ABILITY_PARAMETER_REQUIRED: &'static str = "ability_parameter_required";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_ABILITY_SCORE_OUT_OF_RANGE: &'static str = "ability_score_out_of_range";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_UNKNOWN_ART: &'static str = "unknown_art";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_DUPLICATE_ART: &'static str = "duplicate_art";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_ART_SCORE_OUT_OF_RANGE: &'static str = "art_score_out_of_range";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CHARACTERISTIC_ABOVE_CAP: &'static str = "characteristic_above_cap";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CHARACTERISTIC_BELOW_FLOOR: &'static str = "characteristic_below_floor";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CHARACTERISTIC_MAX_BASE_TOO_LOW: &'static str =
        "characteristic_max_base_too_low";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_CHARACTERISTIC_MIN_BASE_TOO_HIGH: &'static str =
        "characteristic_min_base_too_high";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`].
    pub const CODE_ABILITY_BONUS_DANGLING_TARGET: &'static str = "ability_bonus_dangling_target";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a restricted XP pool — a
    /// grant (Educated/Warrior/Privileged) or a life-stage block — has experience the
    /// character left unspent on its eligible Abilities; the rules waste it.
    /// `origin_kind` (`item` | `life_stage`) plus `origin` (the item id or the block
    /// slug) say *which* pool, since a life-stage character has several.
    pub const CODE_RESTRICTED_XP_UNSPENT: &'static str = "restricted_xp_unspent";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: an Ability whose category
    /// the rules gate behind a Virtue (Academic/Arcane/Martial) is bought without
    /// one. Supernatural has its own per-Ability rule
    /// (`supernatural_ability_requires_virtue`).
    pub const CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE: &'static str =
        "ability_category_requires_virtue";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: an Academic Ability is
    /// bought without a scholarly language at 3+, which the rules normally require.
    pub const CODE_ACADEMIC_ABILITY_WITHOUT_SCHOLARLY_LANGUAGE: &'static str =
        "academic_ability_without_scholarly_language";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the character carries both
    /// a life-stage plan and a directly-entered experience pool. They are
    /// alternative ways of funding the same purchases, so both together would let
    /// the character spend twice.
    pub const CODE_LIFE_STAGE_XP_POOL_CONFLICT: &'static str = "life_stage_xp_pool_conflict";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the character's age falls
    /// inside the childhood block, so it cannot have lived a year of later life.
    pub const CODE_LIFE_STAGE_AGE_BEFORE_CHILDHOOD: &'static str =
        "life_stage_age_before_childhood";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a magus is younger than
    /// childhood plus apprenticeship, so it cannot yet stand at its Gauntlet
    /// (Core Rules.md:2435). The magus-specific counterpart of
    /// [`ValidationIssue::CODE_LIFE_STAGE_AGE_BEFORE_CHILDHOOD`]; one wrong age
    /// produces one of the two, never both.
    pub const CODE_LIFE_STAGE_AGE_BEFORE_GAUNTLET: &'static str = "life_stage_age_before_gauntlet";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the stored Gauntlet age is
    /// later than the character's own age, putting the Gauntlet in its future
    /// (Core Rules.md:2216). [`crate::life_stage::LifeStageRules::budget`] clamps the
    /// value to the age so no figure underflows; this names the fault the clamp
    /// would otherwise absorb in silence.
    pub const CODE_LIFE_STAGE_GAUNTLET_AGE_AFTER_AGE: &'static str =
        "life_stage_gauntlet_age_after_age";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: more lab seasons are
    /// charged against the post-Gauntlet years than those years can hold. Only three
    /// seasons a year cost anything — the deduction runs "to a minimum of 0 if three
    /// or four seasons are spent on lab work" (Core Rules.md:2482) — so the ceiling
    /// is `max_charged_lab_seasons_per_year × post_gauntlet_years`.
    /// [`crate::life_stage::LifeStageRules::budget`] caps the total there, making the
    /// surplus free rather than an error of its own.
    pub const CODE_LIFE_STAGE_LAB_SEASONS_OUT_OF_RANGE: &'static str =
        "life_stage_lab_seasons_out_of_range";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: more of a magus's yearly
    /// post-Gauntlet points are taken as levels of spells than those years granted.
    /// "Each point can be an experience point in an Art or Ability or one level of
    /// spell" (Core Rules.md:2471) splits points that exist;
    /// [`crate::life_stage::LifeStageRules::budget`] holds the stored split to them.
    /// Filed under `abilities`, the phase whose input surface owns the value, not
    /// `spells`, which merely spends the budget it feeds.
    pub const CODE_LIFE_STAGE_SPELL_LEVEL_SPLIT_EXCEEDS_POINTS: &'static str =
        "life_stage_spell_level_split_exceeds_points";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a life-stage plan with no
    /// age, so the later-life block — the one that counts years — cannot be earned.
    /// Childhood is granted regardless (Core Rules.md:2378), which is why this is a
    /// finding of its own rather than the absence of a budget.
    pub const CODE_LIFE_STAGE_AGE_UNSET: &'static str = "life_stage_age_unset";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: no native language chosen,
    /// leaving childhood's largest experience block with nothing to buy.
    pub const CODE_LIFE_STAGE_NATIVE_LANGUAGE_UNSET: &'static str =
        "life_stage_native_language_unset";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a native language is
    /// chosen but no matching Living Language score is bought, so its experience is
    /// unspent.
    pub const CODE_LIFE_STAGE_NATIVE_LANGUAGE_MISSING_SCORE: &'static str =
        "life_stage_native_language_missing_score";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a magus falls short of an
    /// Ability the Order demands — "Magi must have the following minimum Abilities:
    /// Parma Magica 1, Magic Theory 1, Latin 1. Characters with lower scores would not
    /// be admitted to the Order." (Core Rules.md:2437.) One finding per unmet
    /// requirement, and unconditional on how the experience was funded.
    pub const CODE_MAGUS_MINIMUM_ABILITY: &'static str = "magus_minimum_ability";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a magus falls short of one
    /// of the `#### Hermetic Magi Recommended Minimum Abilities` (Core Rules.md:2451-2461)
    /// — advice about a weak magus, not an illegal one.
    pub const CODE_MAGUS_RECOMMENDED_ABILITY: &'static str = "magus_recommended_ability";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the Sample Childhood
    /// package recorded on the life-stage plan names an id the loaded ruleset does
    /// not ship — a dangling reference, since the taken package is persisted.
    pub const CODE_CHILDHOOD_PACKAGE_UNKNOWN: &'static str = "childhood_package_unknown";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a Sample Childhood
    /// package's parameterized entry was left unanswered, so the row it would write
    /// has no parameter to be told apart by. A **command-input** code
    /// ([`childhood_rejection_issues`]), never a [`validate`] finding.
    pub const CODE_CHILDHOOD_SLOT_UNFILLED: &'static str = "childhood_slot_unfilled";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a Sample Childhood
    /// package's language slot was answered with the character's own native
    /// language, which the childhood spread may not buy (Core:2378). A
    /// **command-input** code, never a [`validate`] finding.
    pub const CODE_CHILDHOOD_SLOT_IS_NATIVE_LANGUAGE: &'static str =
        "childhood_slot_is_native_language";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: two slots of one Ability
    /// were answered with the same value, so their rows would merge and the second
    /// entry's experience would vanish. A **command-input** code, never a
    /// [`validate`] finding.
    pub const CODE_CHILDHOOD_SLOT_DUPLICATE_VALUE: &'static str = "childhood_slot_duplicate_value";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a magus's House Choice
    /// grant has no pick, or a pick that is not one of the offered options.
    pub const CODE_HOUSE_CHOICE_UNRESOLVED: &'static str = "house_choice_unresolved";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a magus's House Open
    /// grant pick violates the grant's constraint (kind/magnitude/category).
    pub const CODE_HOUSE_GRANT_CONSTRAINT: &'static str = "house_grant_constraint";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a magus has no
    /// Hermetic House (a "should", not a hard rule).
    pub const CODE_HOUSE_UNSET: &'static str = "house_unset";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a magus has taken no
    /// Hermetic Flaw (the rules recommend at least one).
    pub const CODE_MISSING_HERMETIC_FLAW: &'static str = "missing_hermetic_flaw";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a mythic-companion
    /// type has not been chosen yet (its free status/Minor Virtue + package are
    /// unresolved).
    pub const CODE_MYTHIC_TYPE_UNSET: &'static str = "mythic_type_unset";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a Mythic Companion
    /// type's Choice/Open grant has no pick, or one not among the offered
    /// options.
    pub const CODE_MYTHIC_CHOICE_UNRESOLVED: &'static str = "mythic_choice_unresolved";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a Mythic Companion
    /// type's Open grant pick violates the grant's constraint.
    pub const CODE_MYTHIC_GRANT_CONSTRAINT: &'static str = "mythic_grant_constraint";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a Mythic Companion
    /// type's required Virtue/Flaw (or a suitable substitute) is not selected (a
    /// "should", overridable by troupe agreement — never a hard block).
    pub const CODE_MYTHIC_REQUIRED_TRAIT_MISSING: &'static str = "mythic_required_trait_missing";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a chosen spell's id does
    /// not resolve against the spell catalogue.
    pub const CODE_UNKNOWN_SPELL: &'static str = "unknown_spell";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the same spell (at the
    /// same level) is listed more than once.
    pub const CODE_DUPLICATE_SPELL: &'static str = "duplicate_spell";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a General spell has no
    /// chosen level yet, so it is excluded from the spell-levels budget.
    pub const CODE_SPELL_LEVEL_UNRESOLVED: &'static str = "spell_level_unresolved";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the sum of the chosen
    /// spells' levels exceeds the magus's spell-levels budget (Core:2215-2216).
    pub const CODE_OVER_SPELL_LEVELS: &'static str = "over_spell_levels";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a spell's level exceeds
    /// Technique + Form + Intelligence + Magic Theory + 3 (Core:2465).
    pub const CODE_SPELL_LEVEL_EXCEEDS_CAP: &'static str = "spell_level_exceeds_cap";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a spell's resolved learned
    /// level violates the ritual level bounds — a ritual learned below level 20, or a
    /// non-ritual learned above level 50 (Core:12279-12295, :12283).
    pub const CODE_SPELL_RITUAL_LEGALITY: &'static str = "spell_ritual_legality";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a chosen Spell Mastery
    /// special ability id does not resolve against the mastery-ability catalogue.
    pub const CODE_UNKNOWN_MASTERY_ABILITY: &'static str = "unknown_mastery_ability";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the number of Spell Mastery
    /// special abilities chosen for a spell exceeds its effective mastery score —
    /// one may be chosen per mastery level (Core:9524-9526).
    pub const CODE_TOO_MANY_MASTERY_ABILITIES: &'static str = "too_many_mastery_abilities";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a non-repeatable Spell
    /// Mastery special ability is chosen more than once for the same spell. Only
    /// Precise, Quick, and Quiet Casting may repeat (Core:9572, :9576, :9580).
    pub const CODE_DUPLICATE_MASTERY_ABILITY: &'static str = "duplicate_mastery_ability";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a bought Ability exceeds
    /// the character's age-based maximum (Core:2366-2376; Affinity raises it +2).
    pub const CODE_ABILITY_ABOVE_AGE_CAP: &'static str = "ability_above_age_cap";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a Supernatural Ability is
    /// held with no granting Virtue and no free Gift slot (Core:2874).
    pub const CODE_SUPERNATURAL_ABILITY_REQUIRES_VIRTUE: &'static str =
        "supernatural_ability_requires_virtue";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a Personality Trait is
    /// outside ±3 (or beyond the ±6 allowance a Major Personality Flaw grants).
    pub const CODE_PERSONALITY_TRAIT_OUT_OF_RANGE: &'static str = "personality_trait_out_of_range";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a starting Reputation is
    /// not backed by a granting Virtue/Flaw (Core:2514).
    pub const CODE_REPUTATION_NOT_GRANTED: &'static str = "reputation_not_granted";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the total level of the
    /// character's enchanted devices exceeds the item-level budget the character's
    /// Virtues grant (Magic Items +25, Redcap 50; Core:4347-4349, :4842-4846).
    pub const CODE_OVER_ITEM_LEVEL: &'static str = "over_item_level";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: the total level of the
    /// being's supernatural powers exceeds the power-levels budget its Might
    /// Virtues grant (Demonic Blood 30, Demonic Powers +20; RoP:Infernal:4122,
    /// :4142). A power with no granting Virtue (budget 0) is flagged, mirroring
    /// enchanted devices.
    pub const CODE_OVER_POWER_LEVELS: &'static str = "over_power_levels";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: the entity's base Might
    /// Realm disagrees with the Realm its Might Virtues grant (a supernatural being
    /// belongs to exactly one Realm; Core:2623-2625).
    pub const CODE_MIGHT_REALM_MISMATCH: &'static str = "might_realm_mismatch";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a Characteristic's
    /// completed aging/Decrepitude reductions would push its effective score below
    /// the rules effective minimum (−5). Advisory — the engine still clamps the
    /// derived score at the floor (Core:16579).
    pub const CODE_EXCESSIVE_AGING_REDUCTION: &'static str = "excessive_aging_reduction";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a character over 35 has
    /// no aging rolls recorded, and "a character over the age of 35 must make aging
    /// rolls ... before the game begins" (Core:2232). Advisory: the rolls happen at
    /// the table, so the engine can only say they are owed.
    pub const CODE_LIFE_STAGE_AGING_ROLLS_PENDING: &'static str = "life_stage_aging_rolls_pending";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a chosen Living Condition
    /// resolves against no row of the Living Conditions table (Core:16581-16594).
    /// The modifier computation skips such an id, so without this the aging total
    /// would be silently wrong.
    pub const CODE_UNKNOWN_LIVING_CONDITION: &'static str = "unknown_living_condition";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: two or more chosen Living
    /// Conditions are mutually exclusive. Only the asterisked rows "are cumulative
    /// with each other" (Core:16594); the rest describe one situation each.
    pub const CODE_LIVING_CONDITIONS_CONFLICT: &'static str = "living_conditions_conflict";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: the apparent age exceeds
    /// the actual age, which aging cannot produce — it advances the apparent age by
    /// at most one year per year (Core:16577). Advisory, because `:5189` puts it as
    /// a *should* with an explicit escape for a character who is not basically
    /// human.
    pub const CODE_APPARENT_AGE_ABOVE_AGE: &'static str = "apparent_age_above_age";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: an equipment slot names an
    /// id that does not resolve to any catalogue weapon, shield, or armor
    /// (Core:16944-17011).
    pub const CODE_UNKNOWN_EQUIPMENT: &'static str = "unknown_equipment";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: an equipped weapon or
    /// shield's minimum-Strength requirement exceeds the character's Strength
    /// (Core:16997). Advisory — the character may still carry/wield it, at the
    /// storyguide's discretion, so this never blocks.
    pub const CODE_EQUIPMENT_MIN_STRENGTH: &'static str = "equipment_min_strength";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a shield is equipped
    /// alongside only two-handed weapon(s), so its Attack/Defense modifiers are
    /// dropped (a two-handed weapon cannot be used with a shield). Advisory — the
    /// shield still counts toward Load, and the character may carry it, so this
    /// never blocks (Core:7494, :17063, :16975).
    pub const CODE_SHIELD_WITH_TWO_HANDED_WEAPON: &'static str = "shield_with_two_handed_weapon";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a non-magus owes more
    /// Minor Flaws from Warping than it has chosen fills for (Core:16553-16557).
    pub const CODE_WARPING_OWED_MINOR_FLAWS: &'static str = "warping_owed_minor_flaws";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a non-magus owes a
    /// supernatural Minor Virtue from Warping it has not chosen yet (Core:16559).
    pub const CODE_WARPING_OWED_SUPERNATURAL_VIRTUES: &'static str =
        "warping_owed_supernatural_virtues";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a non-magus owes more
    /// Major Flaws from Warping than it has chosen fills for (Core:16561).
    pub const CODE_WARPING_OWED_MAJOR_FLAWS: &'static str = "warping_owed_major_flaws";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a chosen warping-owed
    /// fill does not match the owed slot's kind/magnitude/category (or its id does
    /// not resolve) — e.g. a Major Flaw where a Minor is owed (Core:16553-16561).
    pub const CODE_WARPING_FILL_CONSTRAINT: &'static str = "warping_fill_constraint";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a chosen warping-owed
    /// fill carries an [`crate::types::Effect::WarpingGrant`] and is ineligible —
    /// folding its Warping Points back would self-amplify the owed count (the
    /// recursion guard).
    pub const CODE_WARPING_FILL_INELIGIBLE: &'static str = "warping_fill_ineligible";
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Error: a stored warping fill is
    /// keyed to a slot the character does not owe (exceeds the owed count).
    pub const CODE_WARPING_FILL_EXCESS: &'static str = "warping_fill_excess";

    /// Builds an issue with the given severity, code, phase, args, and context.
    pub fn new(
        severity: IssueSeverity,
        code: &str,
        phase: CreationPhase,
        args: BTreeMap<String, String>,
        context: Option<Id>,
    ) -> Self {
        Self {
            severity,
            code: code.to_string(),
            phase,
            args,
            context,
        }
    }

    /// Builds an error-severity issue.
    pub fn error(
        code: &str,
        phase: CreationPhase,
        args: BTreeMap<String, String>,
        context: Option<Id>,
    ) -> Self {
        Self::new(IssueSeverity::Error, code, phase, args, context)
    }

    /// Builds a warning-severity issue.
    pub fn warning(
        code: &str,
        phase: CreationPhase,
        args: BTreeMap<String, String>,
        context: Option<Id>,
    ) -> Self {
        Self::new(IssueSeverity::Warning, code, phase, args, context)
    }
}

/// Convenience: build an `args` map from `(name, Display value)` pairs.
fn args<const N: usize>(pairs: [(&str, String); N]) -> BTreeMap<String, String> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// The outcome of validating an entity: a flat list of issues.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// All findings, errors and warnings intermixed in detection order.
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Builds a result from a list of issues.
    pub fn new(issues: Vec<ValidationIssue>) -> Self {
        Self { issues }
    }

    /// Returns `true` if there are no error-severity issues.
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }

    /// Iterates all issues with [`IssueSeverity::Error`].
    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> + '_ {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
    }

    /// Iterates all issues with [`IssueSeverity::Warning`].
    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> + '_ {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
    }

    /// Applies a [`ValidationMode`]: `Enforced` keeps issues as-is, `Advisory`
    /// downgrades every issue to a warning, `Silent` clears all issues.
    pub fn apply_mode(self, mode: ValidationMode) -> Self {
        match mode {
            ValidationMode::Enforced => self,
            ValidationMode::Advisory => ValidationResult {
                issues: self
                    .issues
                    .into_iter()
                    .map(|mut i| {
                        i.severity = IssueSeverity::Warning;
                        i
                    })
                    .collect(),
            },
            ValidationMode::Silent => ValidationResult { issues: vec![] },
        }
    }
}

/// Validates an Entity against a Ruleset, checking balance, caps, prerequisites,
/// incompatibilities, categories, traits, parameters, and gift policy.
///
/// When `entity.type_id` does not resolve to a type profile, an `unknown_type`
/// error is emitted and all profile-dependent sub-validators (balance, caps,
/// prerequisites' `is_magus` resolution, categories, traits, gift policy) are
/// skipped because they have no profile to check against. A result containing
/// only `unknown_type` therefore does NOT imply the rest of the entity is legal.
pub fn validate(entity: &Entity, ruleset: &Ruleset) -> ValidationResult {
    let mut issues = Vec::new();

    let type_profile = ruleset.type_profiles.get(&entity.type_id);

    // Computed once and shared by membership-test sub-validators.
    let selected_ids: BTreeSet<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();

    validate_known_type(entity, type_profile, &mut issues);
    validate_known_refs(entity, ruleset, &mut issues);
    validate_entity_kind_applicability(entity, ruleset, &mut issues);
    validate_duplicate_selections(entity, ruleset, &mut issues);
    validate_balance(entity, ruleset, type_profile, &mut issues);
    validate_caps(entity, ruleset, type_profile, &mut issues);
    validate_tainted_cap(entity, ruleset, &mut issues);
    validate_prerequisites(entity, ruleset, type_profile, &selected_ids, &mut issues);
    validate_incompatibilities(entity, ruleset, &selected_ids, &mut issues);
    validate_permitted_categories(entity, ruleset, type_profile, &mut issues);
    validate_forbidden_categories(entity, ruleset, type_profile, &mut issues);
    validate_required_traits(type_profile, &selected_ids, &mut issues);
    validate_forbidden_traits(type_profile, &selected_ids, &mut issues);
    validate_parameters(entity, ruleset, &mut issues);
    validate_ability_bonus_targets(entity, ruleset, &mut issues);
    validate_magical_focus(entity, ruleset, &mut issues);
    validate_gift_policy(entity, ruleset, type_profile, &mut issues);
    validate_house(entity, ruleset, type_profile, &mut issues);
    validate_mythic_type(entity, ruleset, type_profile, &mut issues);

    // Characteristics and Abilities are character-only concerns; a covenant has
    // neither. Gate them on the entity kind so the engine respects EntityKind
    // rather than relying on a covenant happening to carry no such data.
    if entity.entity_kind == EntityKind::Character {
        validate_characteristics(entity, ruleset, &mut issues);
        validate_characteristic_limit_preconditions(entity, ruleset, &mut issues);
        validate_abilities(entity, ruleset, &mut issues);
        validate_arts(entity, ruleset, &mut issues);
        validate_spells(entity, ruleset, type_profile, &mut issues);
        validate_supernatural_abilities(entity, ruleset, type_profile, &mut issues);
        validate_personality_traits(entity, ruleset, &mut issues);
        validate_reputations(entity, ruleset, &mut issues);
        validate_devices(entity, ruleset, &mut issues);
        validate_powers(entity, ruleset, &mut issues);
        validate_might(entity, ruleset, &mut issues);
        validate_equipment(entity, ruleset, &mut issues);
        validate_aging(entity, ruleset, &mut issues);
        validate_xp_pool(entity, ruleset, &mut issues);
        validate_life_stage_plan(entity, ruleset, type_profile, &mut issues);
        validate_magus_minimum_abilities(entity, ruleset, &mut issues);
        validate_ability_authorization(entity, ruleset, type_profile, &mut issues);
        validate_academic_language(entity, ruleset, &mut issues);
        validate_warping(entity, ruleset, type_profile, &mut issues);
    }

    ValidationResult { issues }
}

/// Emits `unknown_type` when the entity's `type_id` has no matching profile.
pub(crate) fn validate_known_type(
    entity: &Entity,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    if type_profile.is_none() {
        // No creation phase can fix this: the type is chosen once, at creation.
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_UNKNOWN_TYPE,
            CreationPhase::Review,
            args([("type_id", entity.type_id.to_string())]),
            None,
        ));
    }
}

/// Emits `unknown_ref` for each selection whose item id is not in the ruleset.
pub(crate) fn validate_known_refs(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        if !ruleset.point_items.contains_key(&selection.item_ref) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_REF,
                CreationPhase::VirtuesFlaws,
                args([("item", selection.item_ref.to_string())]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::characteristics::Characteristic;
    use crate::grant::GrantConstraint;
    use crate::ruleset::RulesetSources;
    use crate::types::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    fn test_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {
            "id": "virtue.the_gift",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "free",
            "category": "special",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.hermetic_magus",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "free",
            "category": "social_status",
            "entity_kinds": ["character"],
            "prerequisites": { "kind": "has", "value": "virtue.the_gift" }
          },
          {
            "id": "virtue.gentle_gift",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "major",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "prerequisites": { "kind": "has", "value": "virtue.hermetic_magus" },
            "incompatible_with": ["flaw.blatant_gift"]
          },
          {
            "id": "flaw.blatant_gift",
            "kind": "flaw",
            "classification": "narrative",
            "magnitude": "major",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "prerequisites": { "kind": "has", "value": "virtue.the_gift" },
            "incompatible_with": ["virtue.gentle_gift"]
          },
          {
            "id": "virtue.puissant_ability",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }]
          },
          {
            "id": "flaw.poor_student",
            "kind": "flaw",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.keen_vision",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.large",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.tough",
            "kind": "virtue",
            "classification": "narrative",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          }
        ]"#;

        let types = r#"[
          {
            "id": "companion",
            "budget": {
              "virtue_points": 10,
              "flaw_points": 10,
              "max_major_virtues": 1
            },
            "permitted_categories": ["general", "social_status", "supernatural"],
            "forbidden_categories": ["hermetic"],
            "required_traits": [],
            "forbidden_traits": [],
            "gift_policy": "forbidden",
            "gift_id": "virtue.the_gift",
            "gift_categories": ["hermetic"],
            "creation_phases": ["concept", "virtues_flaws", "abilities"]
          }
        ]"#;

        let abilities =
            r#"{ "abilities": [{ "id": "ability.awareness", "category": "general" }] }"#;
        Ruleset::from_json_with_abilities("arm5-core", "2024.1", items, types, abilities).unwrap()
    }

    fn make_entity(type_id: &str, selections: Vec<Selection>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new(type_id),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.selections = selections;
        entity
    }

    fn sel(item_ref: &str) -> Selection {
        Selection::new(Id::new(item_ref))
    }

    /// The houses the `Prereq::House` composition tests reference. They exist
    /// only so the ruleset passes load integrity (which now resolves House
    /// refs); the tests still exercise House as an *unevaluable* leaf because the
    /// entity carries no `house`, so eval yields `Unknown`.
    const TEST_HOUSES: &str = r#"{ "houses": [
        { "id": "house.flambeau", "lineage_type": "societas" },
        { "id": "house.x", "lineage_type": "societas" },
        { "id": "house.bjornaer", "lineage_type": "mystery_cult" }
    ] }"#;

    /// Builds a test ruleset that registers [`TEST_HOUSES`], so `Prereq::House`
    /// refs resolve at load (mirrors `Ruleset::from_json` otherwise).
    fn rs_with_houses(items: &str, types: &str) -> Ruleset {
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: None,
            arts: None,
            houses: Some(TEST_HOUSES),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    // --- Phase 4 (step 8): House-granted Virtues in the two id-sets ----------

    /// Abilities the grant-bearing test Houses seed. Registered so the
    /// `AbilityMin` prereq ref and the `AbilityScoreGrant` effect resolve at load.
    // Carries the five engine-required Hermetic abilities alongside Heartbeast:
    // the magus type profile below makes `validate_engine_required_roles` demand
    // them of any ruleset that ships an abilities catalogue.
    const GRANT_ABILITIES: &str = r#"{ "abilities": [
        { "id": "ability.heartbeast", "category": "supernatural", "requires_training": true },
        { "id": "ability.artes_liberales", "category": "academic" },
        { "id": "ability.magic_theory", "category": "arcane" },
        { "id": "ability.parma_magica", "category": "arcane" },
        { "id": "ability.penetration", "category": "arcane" },
        { "id": "ability.philosophiae", "category": "academic" }
    ] }"#;

    /// A House (Bjornaer) whose fixed grant is the (Major, Hermetic) Heartbeast,
    /// which also seeds the Heartbeast Ability at 1.
    const GRANT_HOUSES: &str = r#"{ "houses": [
        { "id": "house.bjornaer", "lineage_type": "mystery_cult",
          "grants": [ { "kind": "fixed", "item": "virtue.heartbeast" } ] }
    ] }"#;

    /// Items the two-set-split tests use: the granted Heartbeast, two virtues
    /// gated on it (by `Has` and by `AbilityMin`), and one mutually incompatible
    /// with it (for the B1 guard).
    const GRANT_TEST_ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
        { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.heartbeast", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"],
          "effects": [{ "type": "ability_score_grant", "ability": "ability.heartbeast", "amount": 1 }],
          "incompatible_with": ["virtue.foe_of_heartbeast"] },
        { "id": "virtue.needs_heartbeast", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "prerequisites": { "kind": "has", "value": "virtue.heartbeast" } },
        { "id": "virtue.needs_heartbeast_ability", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "prerequisites": { "kind": "ability_min", "value": { "ability": "ability.heartbeast", "score": 1 } } },
        { "id": "virtue.foe_of_heartbeast", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "incompatible_with": ["virtue.heartbeast"] }
    ]"#;

    /// A magus profile permitting the test categories (`is_magus` so it is a
    /// legal House-bearer).
    const GRANT_MAGUS_TYPE: &str = r#"[{
        "id": "magus",
        "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "hermetic", "special", "social_status"],
        "is_magus": true,
        "creation_phases": []
    }]"#;

    fn rs_with_grant_houses(items: &str, types: &str) -> Ruleset {
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(GRANT_ABILITIES),
            arts: None,
            houses: Some(GRANT_HOUSES),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    #[test]
    fn has_prereq_satisfied_by_a_house_granted_virtue() {
        // A Bjornaer magus is granted Heartbeast; a bought virtue whose
        // prerequisite is `Has(virtue.heartbeast)` is satisfied by that grant,
        // though Heartbeast was never bought (it lives in `present_ids`, not
        // `selected_ids`).
        let rs = rs_with_grant_houses(GRANT_TEST_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.needs_heartbeast")]);
        entity.house = Some(Id::new("house.bjornaer"));

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "granted Heartbeast should satisfy Has(virtue.heartbeast): {:?}",
            result.issues
        );
    }

    #[test]
    fn ability_min_prereq_satisfied_by_a_house_granted_ability_floor() {
        // Bjornaer's Heartbeast seeds the Heartbeast Ability at 1, which meets an
        // `AbilityMin(ability.heartbeast, 1)` prerequisite with no bought score.
        let rs = rs_with_grant_houses(GRANT_TEST_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.needs_heartbeast_ability")]);
        entity.house = Some(Id::new("house.bjornaer"));

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "granted Heartbeast Ability floor should satisfy AbilityMin(1): {:?}",
            result.issues
        );
    }

    #[test]
    fn house_grant_does_not_trip_an_incompatibility() {
        // B1 guard: grants stay out of the bought-only set the incompatibility
        // check uses. Buying foe_of_heartbeast while granted the mutually
        // incompatible Heartbeast must NOT fire `incompatible`.
        let rs = rs_with_grant_houses(GRANT_TEST_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.foe_of_heartbeast")]);
        entity.house = Some(Id::new("house.bjornaer"));

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"incompatible".to_string()),
            "a granted virtue must not collide with a bought one: {:?}",
            result.issues
        );
    }

    #[test]
    fn house_grant_does_not_trip_a_forbidden_trait() {
        // B1 guard: a profile forbidding Heartbeast must not flag the House
        // *grant* of it — forbidden-trait checks bought rows only.
        let types = r#"[{
            "id": "magus",
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["general", "hermetic", "special", "social_status"],
            "forbidden_traits": ["virtue.heartbeast"],
            "is_magus": true,
            "creation_phases": []
        }]"#;
        let rs = rs_with_grant_houses(GRANT_TEST_ITEMS, types);
        let mut entity = make_entity("magus", vec![]);
        entity.house = Some(Id::new("house.bjornaer"));

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"forbidden_trait".to_string()),
            "a granted virtue must not trip forbidden_trait: {:?}",
            result.issues
        );
    }

    // --- Phase 4 (step 9): grants are free of the point budget & count caps ---

    #[test]
    fn house_granted_virtue_is_free_of_the_point_budget() {
        // Regression lock: `compute_balance` iterates bought `entity.selections`
        // only, so Bjornaer's granted (Major, 3-point) Heartbeast contributes
        // nothing to the virtue-point tally. A leak into the combined list would
        // read 3 here.
        let rs = rs_with_grant_houses(GRANT_TEST_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![]);
        entity.house = Some(Id::new("house.bjornaer"));

        let balance = compute_balance(&entity, &rs);
        assert_eq!(
            balance.virtue_points, 0,
            "a House-granted Virtue must not spend from the point budget"
        );
    }

    #[test]
    fn house_granted_major_virtue_does_not_count_toward_the_major_virtue_cap() {
        // Regression lock: a profile capping Major Virtues at 0 must not fire on
        // Bjornaer's *granted* Major Heartbeast — `validate_caps` counts bought
        // rows only, so grants are uncapped by construction.
        let types = r#"[{
            "id": "magus",
            "budget": { "virtue_points": 10, "flaw_points": 10, "max_major_virtues": 0 },
            "permitted_categories": ["general", "hermetic", "special", "social_status"],
            "is_magus": true,
            "creation_phases": []
        }]"#;
        let rs = rs_with_grant_houses(GRANT_TEST_ITEMS, types);
        let mut entity = make_entity("magus", vec![]);
        entity.house = Some(Id::new("house.bjornaer"));

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"too_many_major_virtues".to_string()),
            "a granted Major Virtue must not trip the Major-Virtue count cap: {:?}",
            result.issues
        );
    }

    // --- Phase 4 (step 10): per-category *virtue* count caps -----------------

    /// Two Major Hermetic virtues plus the granted-Heartbeast fixture, so a
    /// magus can buy two distinct Major Hermetic Virtues (to trip the cap) while
    /// Bjornaer's grant supplies a third that must stay exempt.
    const CAP_ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
        { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.gentle_gift", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"] },
        { "id": "virtue.mythic_blood", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"] },
        { "id": "virtue.heartbeast", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"] }
    ]"#;

    /// A magus profile capping Major Hermetic Virtues at 1 (hard), mirroring the
    /// shipped `≤1 Major Hermetic Virtue` rule (Core Rules.md:2855-2861).
    const CAP_MAGUS_TYPE: &str = r#"[{
        "id": "magus",
        "budget": { "virtue_points": 30, "flaw_points": 30,
          "virtue_category_caps": [
            { "category": "hermetic", "max": 1, "major_only": true, "hard": true } ] },
        "permitted_categories": ["general", "hermetic", "special", "social_status"],
        "is_magus": true,
        "creation_phases": []
    }]"#;

    #[test]
    fn two_bought_major_hermetic_virtues_trip_the_virtue_category_cap() {
        let rs = rs_with_grant_houses(CAP_ITEMS, CAP_MAGUS_TYPE);
        let entity = make_entity(
            "magus",
            vec![sel("virtue.gentle_gift"), sel("virtue.mythic_blood")],
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"too_many_major_hermetic_virtues".to_string()),
            "two bought Major Hermetic Virtues should trip the cap: {:?}",
            result.issues
        );
    }

    #[test]
    fn house_granted_major_hermetic_virtue_alone_does_not_trip_the_virtue_category_cap() {
        // One bought Major Hermetic Virtue is legal (cap is 1); Bjornaer's
        // granted Major Hermetic Heartbeast is exempt, so the pair must NOT trip
        // the cap even though two Major Hermetic Virtues are "present".
        let rs = rs_with_grant_houses(CAP_ITEMS, CAP_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.gentle_gift")]);
        entity.house = Some(Id::new("house.bjornaer"));

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"too_many_major_hermetic_virtues".to_string()),
            "a granted Major Hermetic Virtue must not count toward the cap: {:?}",
            result.issues
        );
    }

    // --- Tainted V/F half-budget point cap (Core Rules.md:2998-3002) ---------

    /// Items carrying the Tainted flag: two Major tainted virtues (3 pts each),
    /// an untainted Major virtue, and two Major tainted flaws. The cap is
    /// half of the points *actually taken* on each side, so an untainted virtue
    /// balances a tainted one.
    const TAINTED_ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
        { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.tainted_a", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "supernatural", "entity_kinds": ["character"], "tainted": true },
        { "id": "virtue.tainted_b", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "supernatural", "entity_kinds": ["character"], "tainted": true },
        { "id": "virtue.plain", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "flaw.tainted_c", "kind": "flaw", "classification": "narrative", "magnitude": "major",
          "category": "story", "entity_kinds": ["character"], "tainted": true },
        { "id": "flaw.tainted_d", "kind": "flaw", "classification": "narrative", "magnitude": "major",
          "category": "story", "entity_kinds": ["character"], "tainted": true }
    ]"#;

    /// A companion profile permitting the Tainted-cap test categories.
    const TAINTED_TYPE: &str = r#"[{
        "id": "companion",
        "budget": { "virtue_points": 30, "flaw_points": 30 },
        "permitted_categories": ["general", "supernatural", "special", "story"],
        "creation_phases": []
    }]"#;

    #[test]
    fn tainted_field_defaults_false_and_parses_true() {
        let rs = rs_with_houses(TAINTED_ITEMS, TAINTED_TYPE);
        assert!(
            rs.point_items
                .get(&Id::new("virtue.tainted_a"))
                .unwrap()
                .tainted
        );
        assert!(
            !rs.point_items
                .get(&Id::new("virtue.the_gift"))
                .unwrap()
                .tainted
        );
    }

    #[test]
    fn tainted_virtue_points_over_half_of_taken_warn() {
        // Taken: 2 Major tainted (6) + 0 untainted → tainted is all of it,
        // which is more than half, so it warns.
        let rs = rs_with_houses(TAINTED_ITEMS, TAINTED_TYPE);
        let entity = make_entity(
            "companion",
            vec![sel("virtue.tainted_a"), sel("virtue.tainted_b")],
        );
        let result = validate(&entity, &rs);
        assert!(
            all_codes(&result).contains(&"too_many_tainted_virtues".to_string()),
            "all-tainted Virtue points (over half of those taken) should warn: {:?}",
            result.issues
        );
    }

    #[test]
    fn tainted_flaw_points_over_half_of_taken_warn() {
        let rs = rs_with_houses(TAINTED_ITEMS, TAINTED_TYPE);
        let entity = make_entity(
            "companion",
            vec![sel("flaw.tainted_c"), sel("flaw.tainted_d")],
        );
        let result = validate(&entity, &rs);
        assert!(
            all_codes(&result).contains(&"too_many_tainted_flaws".to_string()),
            "all-tainted Flaw points (over half of those taken) should warn: {:?}",
            result.issues
        );
    }

    #[test]
    fn tainted_at_exactly_half_of_taken_is_clean() {
        // Taken: 1 Major tainted (3) + 1 Major untainted (3) = 6 total,
        // 3 tainted = exactly half → "no more than half" holds, no warning.
        let rs = rs_with_houses(TAINTED_ITEMS, TAINTED_TYPE);
        let entity = make_entity(
            "companion",
            vec![sel("virtue.tainted_a"), sel("virtue.plain")],
        );
        let result = validate(&entity, &rs);
        assert!(
            !all_codes(&result).contains(&"too_many_tainted_virtues".to_string()),
            "tainted points at exactly half of those taken must not warn: {:?}",
            result.issues
        );
    }

    // --- Phase 4 (step 11): validate_house -----------------------------------

    /// Items for the House-validation tests: a minor and a major Virtue (for the
    /// open-grant magnitude constraint), a Puissant Art (a Choice option), a
    /// non-Hermetic Flaw and a Hermetic Flaw (for the ≥1-Hermetic-Flaw guideline).
    const HOUSE_ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
        { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.puissant_art", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "parameters": [{ "key": "art", "type": "ref", "domain": "art" }] },
        { "id": "virtue.self_confident", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "virtue.great_characteristic", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "parameters": [{ "key": "characteristic", "type": "ref", "domain": "characteristic" }] },
        { "id": "virtue.wealthy", "kind": "virtue", "classification": "narrative", "magnitude": "major",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "flaw.driven", "kind": "flaw", "classification": "narrative", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "flaw.deficient_technique", "kind": "flaw", "classification": "narrative", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"] }
    ]"#;

    /// Two Houses exercising the player-choice grant kinds: Flambeau's Choice
    /// between two Puissant Arts, and Jerbiton's Open Minor Virtue.
    const HOUSE_VALIDATE_HOUSES: &str = r#"{ "houses": [
        { "id": "house.flambeau", "lineage_type": "societas",
          "grants": [ { "kind": "choice", "choice_key": "flambeau_puissant", "options": [
            { "ref": "virtue.puissant_art", "params": { "art": "art.perdo" } },
            { "ref": "virtue.puissant_art", "params": { "art": "art.ignem" } }
          ] } ] },
        { "id": "house.jerbiton", "lineage_type": "societas",
          "grants": [ { "kind": "open", "choice_key": "jerbiton_virtue",
            "constraint": { "kind": "virtue", "magnitude": "minor" } } ] }
    ] }"#;

    /// A magus profile whose Hermetic gift category tells `missing_hermetic_flaw`
    /// which category is Hermetic, permitting the test categories.
    const HOUSE_MAGUS_TYPE: &str = r#"[{
        "id": "magus",
        "budget": { "virtue_points": 30, "flaw_points": 30 },
        "permitted_categories": ["general", "hermetic", "special", "social_status"],
        "is_magus": true,
        "gift_categories": ["hermetic"],
        "creation_phases": []
    }]"#;

    fn rs_for_house_validation() -> Ruleset {
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: HOUSE_ITEMS,
            type_profiles: HOUSE_MAGUS_TYPE,
            abilities: None,
            arts: None,
            houses: Some(HOUSE_VALIDATE_HOUSES),
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    /// A magus in the given House, carrying one Hermetic Flaw so the
    /// `missing_hermetic_flaw` guideline stays quiet unless a test wants it.
    fn magus_with_house(house: &str) -> Entity {
        let mut entity = make_entity("magus", vec![sel("flaw.deficient_technique")]);
        entity.house = Some(Id::new(house));
        entity
    }

    fn puissant_art(art: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".into(), Id::new(art))]),
        )
    }

    #[test]
    fn magus_without_a_house_gets_the_house_unset_warning() {
        let rs = rs_for_house_validation();
        let entity = make_entity("magus", vec![sel("flaw.deficient_technique")]);

        let result = validate(&entity, &rs);
        assert!(
            warning_codes(&result).contains(&"house_unset".to_string()),
            "a magus with no House should warn house_unset: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_choice_grant_with_no_pick_is_unresolved() {
        let rs = rs_for_house_validation();
        let entity = magus_with_house("house.flambeau");

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"house_choice_unresolved".to_string()),
            "an unpicked Choice grant should error house_choice_unresolved: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_choice_grant_pick_off_the_menu_is_unresolved() {
        let rs = rs_for_house_validation();
        let mut entity = magus_with_house("house.flambeau");
        // Puissant Creo is not one of the offered options (Perdo / Ignem).
        entity
            .house_choices
            .insert("flambeau_puissant".to_string(), puissant_art("art.creo"));

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"house_choice_unresolved".to_string()),
            "an off-menu Choice pick should error house_choice_unresolved: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_resolved_choice_grant_has_no_unresolved_error() {
        let rs = rs_for_house_validation();
        let mut entity = magus_with_house("house.flambeau");
        entity
            .house_choices
            .insert("flambeau_puissant".to_string(), puissant_art("art.ignem"));

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"house_choice_unresolved".to_string()),
            "a valid Choice pick must not error house_choice_unresolved: {:?}",
            result.issues
        );
    }

    #[test]
    fn an_open_grant_with_no_pick_is_unresolved() {
        let rs = rs_for_house_validation();
        let entity = magus_with_house("house.jerbiton");

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"house_choice_unresolved".to_string()),
            "an unpicked Open grant should error house_choice_unresolved: {:?}",
            result.issues
        );
    }

    #[test]
    fn an_open_grant_pick_violating_its_constraint_is_an_error() {
        let rs = rs_for_house_validation();
        let mut entity = magus_with_house("house.jerbiton");
        // Jerbiton's grant demands a Minor Virtue; Wealthy is Major.
        entity
            .house_choices
            .insert("jerbiton_virtue".to_string(), sel("virtue.wealthy"));

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"house_grant_constraint".to_string()),
            "a Major Virtue pick should violate the Minor-Virtue constraint: {:?}",
            result.issues
        );
    }

    #[test]
    fn an_open_grant_pick_satisfying_its_constraint_has_no_error() {
        let rs = rs_for_house_validation();
        let mut entity = magus_with_house("house.jerbiton");
        // Self-Confident is a Minor Virtue, satisfying the constraint.
        entity
            .house_choices
            .insert("jerbiton_virtue".to_string(), sel("virtue.self_confident"));

        let result = validate(&entity, &rs);
        let codes = codes(&result);
        assert!(
            !codes.contains(&"house_grant_constraint".to_string())
                && !codes.contains(&"house_choice_unresolved".to_string()),
            "a Minor Virtue pick must satisfy the Open grant: {:?}",
            result.issues
        );
    }

    /// An Open grant pick of a parameterized Virtue must name its parameter — the
    /// grant is unresolved in practice while "(Characteristic)" is still blank, so
    /// the pick gets the same parameter checks a bought selection gets.
    #[test]
    fn an_open_house_pick_missing_its_param_errors() {
        let rs = rs_for_house_validation();
        let mut entity = magus_with_house("house.jerbiton");
        entity.house_choices.insert(
            "jerbiton_virtue".to_string(),
            sel("virtue.great_characteristic"),
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&ValidationIssue::CODE_MISSING_PARAM.to_string()),
            "a parameterized Open pick with no param should error missing_param: {:?}",
            result.issues
        );
    }

    #[test]
    fn an_open_house_pick_with_resolving_param_is_clean() {
        let rs = rs_for_house_validation();
        let mut entity = magus_with_house("house.jerbiton");
        entity.house_choices.insert(
            "jerbiton_virtue".to_string(),
            Selection::with_params(
                Id::new("virtue.great_characteristic"),
                BTreeMap::from([("characteristic".into(), Id::new("characteristic.per"))]),
            ),
        );

        let result = validate(&entity, &rs);
        let param_codes: Vec<&String> = result
            .issues
            .iter()
            .map(|i| &i.code)
            .filter(|c| c.ends_with("_param") || c.as_str() == "unknown_param_value")
            .collect();
        assert!(
            param_codes.is_empty(),
            "a resolving param on an Open pick should raise no parameter issue: {param_codes:?}"
        );
    }

    #[test]
    fn open_pick_satisfies_rejects_every_way_a_pick_can_violate_its_constraint() {
        use std::collections::BTreeSet;
        let rs = rs_for_house_validation();
        // Baseline: a Minor Virtue (self_confident, category "general") satisfies
        // a Minor-Virtue constraint whose allow-list includes its category.
        let ok = GrantConstraint {
            kind: ItemKind::Virtue,
            magnitude: Some(Magnitude::Minor),
            require_categories: BTreeSet::from(["general".to_string()]),
            forbid_categories: BTreeSet::new(),
        };
        assert!(open_pick_satisfies(&sel("virtue.self_confident"), &ok, &rs));

        // 1. Unresolvable pick: the item id is not in the ruleset.
        assert!(!open_pick_satisfies(
            &sel("virtue.does_not_exist"),
            &ok,
            &rs
        ));

        // 2. Wrong kind: a Virtue pick against a Flaw constraint.
        let wants_flaw = GrantConstraint {
            kind: ItemKind::Flaw,
            magnitude: None,
            require_categories: BTreeSet::new(),
            forbid_categories: BTreeSet::new(),
        };
        assert!(!open_pick_satisfies(
            &sel("virtue.self_confident"),
            &wants_flaw,
            &rs
        ));

        // 3. Category absent from a non-empty require list.
        let wants_hermetic = GrantConstraint {
            kind: ItemKind::Virtue,
            magnitude: None,
            require_categories: BTreeSet::from(["hermetic".to_string()]),
            forbid_categories: BTreeSet::new(),
        };
        assert!(!open_pick_satisfies(
            &sel("virtue.self_confident"),
            &wants_hermetic,
            &rs
        ));

        // 4. Category on the forbid list.
        let forbids_general = GrantConstraint {
            kind: ItemKind::Virtue,
            magnitude: None,
            require_categories: BTreeSet::new(),
            forbid_categories: BTreeSet::from(["general".to_string()]),
        };
        assert!(!open_pick_satisfies(
            &sel("virtue.self_confident"),
            &forbids_general,
            &rs
        ));
    }

    #[test]
    fn a_magus_with_no_hermetic_flaw_is_warned() {
        let rs = rs_for_house_validation();
        let mut entity = make_entity("magus", vec![sel("flaw.driven")]);
        entity.house = Some(Id::new("house.jerbiton"));
        entity
            .house_choices
            .insert("jerbiton_virtue".to_string(), sel("virtue.self_confident"));

        let result = validate(&entity, &rs);
        assert!(
            warning_codes(&result).contains(&"missing_hermetic_flaw".to_string()),
            "a magus whose only Flaw is non-Hermetic should warn missing_hermetic_flaw: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_magus_with_a_hermetic_flaw_is_not_warned() {
        let rs = rs_for_house_validation();
        let mut entity = make_entity("magus", vec![sel("flaw.deficient_technique")]);
        entity.house = Some(Id::new("house.jerbiton"));
        entity
            .house_choices
            .insert("jerbiton_virtue".to_string(), sel("virtue.self_confident"));

        let result = validate(&entity, &rs);
        assert!(
            !warning_codes(&result).contains(&"missing_hermetic_flaw".to_string()),
            "a magus with a Hermetic Flaw must not warn missing_hermetic_flaw: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_non_magus_gets_no_house_warnings() {
        // A companion is not a House-bearer, so neither house_unset nor
        // missing_hermetic_flaw applies even with no House and no Hermetic Flaw.
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("flaw.poor_student")]);

        let result = validate(&entity, &rs);
        let warnings = warning_codes(&result);
        assert!(
            !warnings.contains(&"house_unset".to_string())
                && !warnings.contains(&"missing_hermetic_flaw".to_string()),
            "House guidelines must not apply to a non-magus: {:?}",
            result.issues
        );
    }

    fn puissant(ability: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".into(), Id::new(ability))]),
        )
    }

    fn great(characteristic: Characteristic) -> Selection {
        Selection::with_params(
            Id::new("virtue.great_characteristic"),
            BTreeMap::from([("characteristic".into(), characteristic.id())]),
        )
    }

    fn poor(characteristic: Characteristic) -> Selection {
        Selection::with_params(
            Id::new("flaw.poor_characteristic"),
            BTreeMap::from([("characteristic".into(), characteristic.id())]),
        )
    }

    fn codes(result: &ValidationResult) -> Vec<String> {
        result.errors().map(|i| i.code.clone()).collect()
    }

    /// All issue codes (errors AND warnings), for tests asserting on warnings.
    fn all_codes(result: &ValidationResult) -> Vec<String> {
        result.issues.iter().map(|i| i.code.clone()).collect()
    }

    #[test]
    fn valid_entity_passes() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![sel("virtue.keen_vision"), sel("flaw.poor_student")],
        );

        let result = validate(&entity, &rs);
        assert!(result.is_valid(), "issues: {:?}", result.issues);
    }

    /// A magus profile permitting the categories the device tests use, with the
    /// Magic Items Virtue granting a +25 item-level budget.
    const DEVICE_ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
        { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.magic_items", "kind": "virtue", "classification": "creation_effect", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "effects": [{ "type": "item_level_budget", "amount": 25 }] }
    ]"#;

    fn device(name: &str, level: u16) -> EnchantedDevice {
        EnchantedDevice {
            name: name.into(),
            level,
        }
    }

    #[test]
    fn devices_within_item_level_budget_pass() {
        let rs = rs_with_houses(DEVICE_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity(
            "magus",
            vec![sel("virtue.the_gift"), sel("virtue.magic_items")],
        );
        entity.devices = vec![device("Wand", 15), device("Ring", 10)]; // 25 == budget
        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&ValidationIssue::CODE_OVER_ITEM_LEVEL.to_string()),
            "issues: {:?}",
            result.issues
        );
    }

    #[test]
    fn devices_over_item_level_budget_error() {
        let rs = rs_with_houses(DEVICE_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity(
            "magus",
            vec![sel("virtue.the_gift"), sel("virtue.magic_items")],
        );
        entity.devices = vec![device("Wand", 20), device("Ring", 10)]; // 30 > 25
        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&ValidationIssue::CODE_OVER_ITEM_LEVEL.to_string()));
    }

    #[test]
    fn device_with_no_budget_grant_error() {
        let rs = rs_with_houses(DEVICE_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.the_gift")]);
        entity.devices = vec![device("Wand", 5)]; // budget 0
        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&ValidationIssue::CODE_OVER_ITEM_LEVEL.to_string()));
    }

    /// A profile whose Demonic Blood Virtue grants Infernal Might 5 + 30 power
    /// levels (RoP:Infernal:4120-4122).
    const MIGHT_ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
        { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.demonic_blood", "kind": "virtue", "classification": "creation_effect", "magnitude": "major",
          "category": "supernatural", "entity_kinds": ["character"],
          "effects": [
            { "type": "might_grant", "realm": "infernal", "score": 5 },
            { "type": "power_levels", "amount": 30 }
          ] }
    ]"#;

    fn power(name: &str, level: u16) -> SupernaturalPower {
        SupernaturalPower {
            name: name.into(),
            level,
        }
    }

    #[test]
    fn powers_within_power_levels_budget_pass() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.demonic_blood")]);
        entity.powers = vec![power("Curse", 20), power("Shape", 10)]; // 30 == budget
        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&ValidationIssue::CODE_OVER_POWER_LEVELS.to_string()),
            "issues: {:?}",
            result.issues
        );
    }

    #[test]
    fn powers_over_power_levels_budget_error() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.demonic_blood")]);
        entity.powers = vec![power("Curse", 25), power("Shape", 10)]; // 35 > 30
        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&ValidationIssue::CODE_OVER_POWER_LEVELS.to_string()));
    }

    #[test]
    fn might_realm_mismatch_warns() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.demonic_blood")]);
        // Grant realm is Infernal, but the entity's base Might claims Magic.
        entity.might = Some(MightScore {
            realm: Realm::Magic,
            score: 2,
        });
        let result = validate(&entity, &rs);
        assert!(
            warning_codes(&result)
                .contains(&ValidationIssue::CODE_MIGHT_REALM_MISMATCH.to_string())
        );
    }

    /// A dangling selection (unresolvable `item_ref`, legal in direct/unchecked
    /// entry) ordered BEFORE the Might-granting Virtue must NOT suppress the
    /// realm-agreement check: the granted Realm is still found and the mismatch
    /// still warns. Regression guard for the early-`?`-abort bug.
    #[test]
    fn dangling_selection_before_might_virtue_still_warns() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let entity_selections = vec![
            sel("virtue.does_not_exist"), // dangling ref, ordered first
            sel("virtue.demonic_blood"),  // grants Infernal Might
        ];
        let mut entity = make_entity("magus", entity_selections);
        // Grant realm is Infernal, but the entity's base Might claims Magic.
        entity.might = Some(MightScore {
            realm: Realm::Magic,
            score: 2,
        });
        let result = validate(&entity, &rs);
        assert!(
            warning_codes(&result)
                .contains(&ValidationIssue::CODE_MIGHT_REALM_MISMATCH.to_string()),
            "a dangling selection before the Might Virtue must not disable the \
             realm check: {:?}",
            result.issues
        );
    }

    /// A being with a base Might but NO Might-granting Virtue exercises the
    /// `ruleset_might_grant_realm` None path: with no granted Realm to compare
    /// against, the realm-agreement check is skipped and no mismatch is raised,
    /// even though the base Realm differs from what a grant would supply.
    #[test]
    fn might_without_a_granting_virtue_raises_no_realm_mismatch() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        // No Demonic Blood → nothing grants Might, yet a base Might is entered.
        let mut entity = make_entity("magus", vec![]);
        entity.might = Some(MightScore {
            realm: Realm::Magic,
            score: 3,
        });
        let result = validate(&entity, &rs);
        assert!(
            !warning_codes(&result)
                .contains(&ValidationIssue::CODE_MIGHT_REALM_MISMATCH.to_string()),
            "with no Might-granting Virtue there is nothing to cross-check: {:?}",
            result.issues
        );
    }

    /// The over-budget power error carries the real used / budget / over values.
    #[test]
    fn over_power_levels_reports_used_budget_over() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.demonic_blood")]);
        entity.powers = vec![power("Curse", 25), power("Shape", 10)]; // 35 > 30
        let result = validate(&entity, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_OVER_POWER_LEVELS)
            .expect("over_power_levels present");
        assert_eq!(issue.args.get("used").map(String::as_str), Some("35"));
        assert_eq!(issue.args.get("budget").map(String::as_str), Some("30"));
        assert_eq!(issue.args.get("over").map(String::as_str), Some("5"));
    }

    /// A power on a being with no Might-granting Virtue is charged against a 0
    /// budget and flagged over (mirrors the device-budget check).
    #[test]
    fn power_without_granting_virtue_is_over_zero_budget() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![]); // no Demonic Blood → budget 0
        entity.powers = vec![power("Curse", 5)];
        let result = validate(&entity, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_OVER_POWER_LEVELS)
            .expect("over_power_levels present");
        assert_eq!(issue.args.get("budget").map(String::as_str), Some("0"));
        assert_eq!(issue.args.get("used").map(String::as_str), Some("5"));
        assert_eq!(issue.args.get("over").map(String::as_str), Some("5"));
    }

    /// The realm-mismatch warning carries the base and granted realm slugs.
    #[test]
    fn might_realm_mismatch_reports_base_and_granted() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![sel("virtue.demonic_blood")]);
        entity.might = Some(MightScore {
            realm: Realm::Magic,
            score: 2,
        });
        let result = validate(&entity, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_MIGHT_REALM_MISMATCH)
            .expect("might_realm_mismatch present");
        assert_eq!(issue.args.get("base").map(String::as_str), Some("magic"));
        assert_eq!(
            issue.args.get("granted").map(String::as_str),
            Some("infernal")
        );
    }

    /// **Invariant (M5.5c).** The familiar's Characteristics are its own creature
    /// statblock, not bought from the magus's Characteristic points
    /// (Core:17793 — a creature's Characteristics are simply "a list of the
    /// characteristics and values"). Because they are nested in `Entity.familiar`
    /// and never in `Entity.characteristics`, `validate_characteristics` cannot
    /// see them: a familiar whose Characteristics would cost far more than the
    /// starting budget changes no issue at all.
    #[test]
    fn familiar_characteristics_never_enter_the_magus_point_buy() {
        let rs = aging_ruleset();
        let mut entity = make_entity("companion", vec![]);
        // Exactly the 7 starting points: Str 3 (6) + Qik 1 (1).
        entity.characteristics.insert(Characteristic::Str, 3);
        entity.characteristics.insert(Characteristic::Qik, 1);
        let before = all_codes(&validate(&entity, &rs));

        entity.familiar = Some(Familiar {
            name: "Corax".into(),
            // Eight maxed Characteristics — 48 points if they were ever charged.
            characteristics: Characteristic::ALL.into_iter().map(|c| (c, 3)).collect(),
            ..Default::default()
        });
        let after = all_codes(&validate(&entity, &rs));
        assert_eq!(
            before, after,
            "the familiar's Characteristics must not touch the magus's point-buy"
        );
    }

    /// **Invariant (M5.5c).** The familiar's Magic Might belongs to the familiar,
    /// so it can never trip [`validate_might`]'s realm-agreement warning (which
    /// reads only `Entity.might`), nor charge the being's power-levels budget, nor
    /// make the magus himself look like a Might-being (which would offer the
    /// non-magus Might tab).
    #[test]
    fn familiar_might_never_triggers_the_realm_mismatch_warning() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        // Demonic Blood grants Infernal Might, so a *base* Might of another Realm
        // would warn — the familiar's Faerie Might must not.
        let mut entity = make_entity("magus", vec![sel("virtue.demonic_blood")]);
        entity.might = None;
        entity.familiar = Some(Familiar {
            name: "Corax".into(),
            might: Some(MightScore {
                realm: Realm::Faerie,
                score: 25,
            }),
            powers: vec![power("Mental communication", 400)],
            ..Default::default()
        });
        let result = validate(&entity, &rs);
        assert!(
            !all_codes(&result).contains(&ValidationIssue::CODE_MIGHT_REALM_MISMATCH.to_string()),
            "issues: {:?}",
            result.issues
        );
        assert!(
            !all_codes(&result).contains(&ValidationIssue::CODE_OVER_POWER_LEVELS.to_string()),
            "the familiar's bond-invested powers are charged against no budget: {:?}",
            result.issues
        );

        // And with no Might Virtue at all, a familiar with Might leaves the magus
        // a plain magus — `effective_might` stays `None`.
        let mut plain = make_entity("magus", vec![]);
        plain.familiar = entity.familiar.clone();
        assert_eq!(crate::effective::effective_might(&plain, &rs), None);
    }

    /// **Guidance-only contract (M5.5c).** Every familiar read-out is read-only
    /// guidance, like `masterpiece_item_cap`: a familiar pushed to every extreme the
    /// model allows — maximum cords 5/5/5 (225 points, far past any Lab Total),
    /// hundreds of levels of bond-invested powers, and a *Faerie* Might on a
    /// Hermetic magus — raises **zero** validation issues, of any severity.
    #[test]
    fn fully_populated_familiar_raises_no_issues() {
        let rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut entity = make_entity("magus", vec![]);
        entity.familiar = Some(Familiar {
            name: "Corax".into(),
            animal: "raven".into(),
            might: Some(MightScore {
                realm: Realm::Faerie,
                score: 30,
            }),
            characteristics: Characteristic::ALL.into_iter().map(|c| (c, 5)).collect(),
            size: -4,
            personality_traits: vec![PersonalityTrait {
                name: "Loyal (Marcus)".into(),
                value: 3,
            }],
            cord_gold: 5,
            cord_silver: 5,
            cord_bronze: 5,
            powers: vec![power("Speech", 200), power("Shapechanging", 250)],
        });
        let with_familiar = validate(&entity, &rs);

        entity.familiar = None;
        let without = validate(&entity, &rs);
        assert_eq!(
            all_codes(&with_familiar),
            all_codes(&without),
            "no familiar read-out may ever produce a ValidationIssue"
        );
    }

    /// The smallest loadable aging block: the threshold the pending-rolls finding
    /// reads (`start_age: 35`), the Living Conditions rows the unknown-id and
    /// conflict checks resolve against — three non-cumulative alternatives and two
    /// asterisked rows that stack (`:16594`) — and a single open-ended outcome row,
    /// which is all `Ruleset::validate_aging_rules`' tiling gate needs. The shipped
    /// table is asserted in `tests/data_integrity.rs`, not restated here.
    const AGING_RULES: &str = r#"{
          "start_age": 35,
          "age_divisor": 10,
          "apparent_age_increase_min": 3,
          "living_conditions": [
            { "id": "living_condition.average_peasant", "modifier": 0 },
            { "id": "living_condition.leper", "modifier": -2, "cumulative": true },
            { "id": "living_condition.typical_summer_or_autumn_covenant_magus", "modifier": 2 },
            { "id": "living_condition.wealthy_or_healthy_location", "modifier": 2 },
            { "id": "living_condition.work_in_a_mine", "modifier": -1, "cumulative": true }
          ],
          "outcomes": [
            { "min": 10, "effect": { "type": "any_characteristic", "points": 1 } }
          ]
        }"#;

    /// A minimal ruleset carrying characteristic rules (effective range ±5), so the
    /// aging-reduction floor check has a minimum to compare against, plus
    /// [`AGING_RULES`] — every aging finding reads its numbers off the ruleset, so
    /// a fixture without them would leave its test asserting nothing.
    fn aging_ruleset() -> Ruleset {
        ruleset_with_aging(Some(AGING_RULES))
    }

    /// The same fixture with `aging` set as given, so a test can witness what a
    /// ruleset shipping no aging rules at all does: stand the subsystem down.
    fn ruleset_with_aging(aging: Option<&'static str>) -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
            "category": "special", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["special"], "forbidden_categories": [], "required_traits": [],
            "forbidden_traits": [], "gift_policy": "forbidden", "gift_id": "virtue.the_gift",
            "gift_categories": [], "creation_phases": ["concept"] }
        ]"#;
        let characteristics = r#"{
          "start_points": 7, "base_max": 3, "base_min": -3,
          "effective_max": 5, "effective_min": -5,
          "costs": [
            { "score": 3, "cost": 6 }, { "score": 2, "cost": 3 }, { "score": 1, "cost": 1 },
            { "score": 0, "cost": 0 }, { "score": -1, "cost": -1 }, { "score": -2, "cost": -3 },
            { "score": -3, "cost": -6 }
          ]
        }"#;
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: Some(characteristics),
            life_stages: None,
            childhoods: None,
            aging,
        })
        .unwrap()
    }

    /// A plausible aged character whose accrued points have not yet forced a drop
    /// (points within score magnitude) raises no aging advisory.
    #[test]
    fn plausible_aging_state_has_no_warnings() {
        let rs = aging_ruleset();
        let mut entity = make_entity("companion", vec![]);
        entity.characteristics.insert(Characteristic::Str, 3);
        entity.aging_points.insert(Characteristic::Str, 1); // 1 ≤ |3|, no drop
        let result = validate(&entity, &rs);
        let codes = all_codes(&result);
        assert!(!codes.contains(&ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION.to_string()));
    }

    /// Derived drops that would drive the effective Characteristic below the rules
    /// floor warn (non-blocking); the score is clamped regardless.
    #[test]
    fn excessive_aging_reduction_warns() {
        let rs = aging_ruleset();
        let mut entity = make_entity("companion", vec![]);
        entity.characteristics.insert(Characteristic::Str, 0);
        // From a 0 score, 21 points force 6 drops (1+2+3+4+5+6) → −6 < −5.
        entity.aging_points.insert(Characteristic::Str, 21);
        let result = validate(&entity, &rs);
        assert!(
            all_codes(&result)
                .contains(&ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION.to_string())
        );
        // Advisory only — never an error.
        assert!(
            result.errors().next().is_none(),
            "issues: {:?}",
            result.issues
        );
    }

    /// Accrued points that force a drop (but stay above the floor) raise NO aging
    /// advisory — the auto-applied drop is not surfaced as validation noise — and
    /// never block. The excessive-reduction warning only fires below the floor.
    #[test]
    fn aging_points_forcing_a_drop_raise_no_note() {
        let rs = aging_ruleset();
        let mut entity = make_entity("companion", vec![]);
        entity.characteristics.insert(Characteristic::Sta, 1);
        entity.aging_points.insert(Characteristic::Sta, 3); // 3 > |1| → drops, but above −5
        let result = validate(&entity, &rs);
        assert!(
            !all_codes(&result)
                .contains(&ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION.to_string())
        );
        assert!(
            result.errors().next().is_none(),
            "issues: {:?}",
            result.issues
        );
    }

    /// "A character over the age of 35 must make aging rolls ... before the game
    /// begins" (Core Rules.md:2232) is a rule about *any* character, not only one
    /// built through its life stages — so the finding cannot hang off the
    /// life-stage plan, which a directly-entered character does not carry. This
    /// fixture has no plan at all, and the warning still fires.
    #[test]
    fn a_character_over_thirty_five_owes_aging_rolls_without_a_life_stage_plan() {
        let rs = aging_ruleset();
        let mut entity = make_entity("companion", vec![]);
        entity.age = Some(40);
        assert!(entity.life_stages.is_none(), "no plan, by construction");

        let result = validate(&entity, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_LIFE_STAGE_AGING_ROLLS_PENDING)
            .expect("the owed aging rolls are reported");
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.phase, CreationPhase::Review);
        assert_eq!(issue.args.get("age").map(String::as_str), Some("40"));
        // Advisory only — an unrolled character is unfinished, not illegal.
        assert!(
            result.errors().next().is_none(),
            "issues: {:?}",
            result.issues
        );
    }

    /// "Characters begin aging in the Winter **after** they turn 35"
    /// (Core Rules.md:16565), and the rule reads "over the age of 35"
    /// (`:2232`) — so 35 owes nothing and 36 owes the first roll. The exact
    /// boundary, pinned on both sides.
    #[test]
    fn the_first_aging_roll_is_owed_at_thirty_six_not_thirty_five() {
        let rs = aging_ruleset();
        let owed = |age: u32| {
            let mut entity = make_entity("companion", vec![]);
            entity.age = Some(age);
            all_codes(&validate(&entity, &rs))
                .contains(&ValidationIssue::CODE_LIFE_STAGE_AGING_ROLLS_PENDING.to_string())
        };

        assert!(!owed(35), "a character of 35 has not yet begun aging");
        assert!(owed(36), "the first roll is owed the year after 35");
    }

    /// A character with no age entered cannot be over 35, so it owes nothing yet.
    #[test]
    fn a_character_without_an_age_owes_no_aging_rolls() {
        let rs = aging_ruleset();
        let entity = make_entity("companion", vec![]);
        assert!(
            !all_codes(&validate(&entity, &rs))
                .contains(&ValidationIssue::CODE_LIFE_STAGE_AGING_ROLLS_PENDING.to_string())
        );
    }

    /// A recorded aging log means the rolls were made, whatever they produced. It
    /// is the log — not the accrued points — that settles them: a roll can
    /// legitimately produce no aging points at all, so keying on the points would
    /// nag a character that had rolled well.
    #[test]
    fn a_recorded_aging_log_settles_the_owed_rolls() {
        let rs = aging_ruleset();
        let mut entity = make_entity("companion", vec![]);
        entity.age = Some(60);
        entity.aging_log = vec![crate::types::AgingLogEntry {
            year: Some(1220),
            effect: "No apparent aging.".into(),
            ..Default::default()
        }];
        assert!(entity.aging_points.is_empty(), "rolled, but gained nothing");

        assert!(
            !all_codes(&validate(&entity, &rs))
                .contains(&ValidationIssue::CODE_LIFE_STAGE_AGING_ROLLS_PENDING.to_string()),
            "a logged roll settles the finding"
        );
    }

    /// The set of chosen Living Conditions, as a canonical [`BTreeSet`].
    fn living_conditions<const N: usize>(ids: [&str; N]) -> BTreeSet<Id> {
        ids.into_iter().map(Id::new).collect()
    }

    /// The standard referential check, applied to the Living Conditions table:
    /// `living_conditions_modifier` deliberately *skips* an id the table does not
    /// carry (the engine has one evaluation path and always produces a number), so
    /// without this finding a typo would silently make the aging total wrong.
    /// One finding per unknown id, like `unknown_ability` and `unknown_spell`.
    #[test]
    fn an_unknown_living_condition_is_reported() {
        let rs = aging_ruleset();
        let mut entity = make_entity("companion", vec![]);
        entity.living_conditions = living_conditions([
            "living_condition.average_peasant",
            "living_condition.nonesuch",
            "living_condition.no_such_row_either",
        ]);

        let result = validate(&entity, &rs);
        let reported: Vec<&str> = result
            .issues
            .iter()
            .filter(|i| i.code == ValidationIssue::CODE_UNKNOWN_LIVING_CONDITION)
            .map(|i| i.args.get("condition").map(String::as_str).unwrap_or(""))
            .collect();
        assert_eq!(
            reported,
            vec![
                "living_condition.no_such_row_either",
                "living_condition.nonesuch"
            ],
            "one finding per unknown id, in canonical order: {:?}",
            result.issues
        );

        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_UNKNOWN_LIVING_CONDITION)
            .expect("the unresolvable id is reported");
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.phase, CreationPhase::Review);
    }

    /// A ruleset shipping no aging rules stands the whole subsystem down — there
    /// is no table for an id to be unknown against, so nothing is reported.
    #[test]
    fn a_ruleset_without_aging_rules_reports_no_unknown_living_condition() {
        let rs = ruleset_with_aging(None);
        let mut entity = make_entity("companion", vec![]);
        entity.living_conditions = living_conditions(["living_condition.nonesuch"]);
        assert!(
            !all_codes(&validate(&entity, &rs))
                .contains(&ValidationIssue::CODE_UNKNOWN_LIVING_CONDITION.to_string())
        );
    }

    /// "Modifiers marked with an asterisk are cumulative with each other"
    /// (Core Rules.md:16594) — which is only worth saying because the *rest* are
    /// alternatives. A character cannot be both "Wealthy, or healthy location" and
    /// "Average peasant", and the covenant rows are graded versions of one
    /// situation, so at most one non-cumulative row may be chosen.
    ///
    /// Exactly ONE finding naming the first two offenders in canonical id order,
    /// not one per pair: three mutually exclusive rows are a single mistake to fix,
    /// and a pair explosion would report it three times.
    #[test]
    fn two_non_cumulative_living_conditions_conflict() {
        let rs = aging_ruleset();
        let conflicts = |ids: BTreeSet<Id>| {
            let mut entity = make_entity("companion", vec![]);
            entity.living_conditions = ids;
            validate(&entity, &rs)
                .issues
                .into_iter()
                .filter(|i| i.code == ValidationIssue::CODE_LIVING_CONDITIONS_CONFLICT)
                .collect::<Vec<_>>()
        };

        // The five asterisked rows stack, so two of them are legal.
        assert!(
            conflicts(living_conditions([
                "living_condition.leper",
                "living_condition.work_in_a_mine",
            ]))
            .is_empty(),
            "the cumulative rows stack with each other (:16594)"
        );

        // A cumulative row plus one baseline is the table's own normal case.
        assert!(
            conflicts(living_conditions([
                "living_condition.leper",
                "living_condition.average_peasant",
            ]))
            .is_empty(),
            "one non-cumulative row alongside a cumulative one is legal"
        );

        // Three exclusive alternatives: one finding, naming the first two.
        let found = conflicts(living_conditions([
            "living_condition.average_peasant",
            "living_condition.typical_summer_or_autumn_covenant_magus",
            "living_condition.wealthy_or_healthy_location",
        ]));
        assert_eq!(
            found.len(),
            1,
            "one finding, not a pair explosion: {found:?}"
        );
        let issue = &found[0];
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.phase, CreationPhase::Review);
        assert_eq!(
            issue.args.get("condition").map(String::as_str),
            Some("living_condition.average_peasant")
        );
        assert_eq!(
            issue.args.get("other").map(String::as_str),
            Some("living_condition.typical_summer_or_autumn_covenant_magus")
        );
    }

    /// "Otherwise, the character's apparent age increases by one year"
    /// (Core Rules.md:16577) — at most one year per year lived, so absent a
    /// supernatural reason the apparent age cannot outrun the actual one, and a
    /// higher figure is a transposed entry.
    ///
    /// A WARNING, not an error: the closest explicit text is Unaging's aside — "You
    /// may choose your apparent age freely, although if you are basically human it
    /// **should be** less than or equal to your actual age" (`:5189`) — a *should*,
    /// carrying its own escape for a character who is not basically human.
    #[test]
    fn an_apparent_age_above_the_actual_age_is_reported() {
        let rs = aging_ruleset();
        let reported = |apparent_age: Option<u32>, age: Option<u32>| {
            let mut entity = make_entity("companion", vec![]);
            entity.age = age;
            entity.apparent_age = apparent_age;
            validate(&entity, &rs)
                .issues
                .into_iter()
                .find(|i| i.code == ValidationIssue::CODE_APPARENT_AGE_ABOVE_AGE)
        };

        let issue = reported(Some(50), Some(40)).expect("the crossed ages are reported");
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.phase, CreationPhase::Review);
        assert_eq!(
            issue.args.get("apparent_age").map(String::as_str),
            Some("50")
        );
        assert_eq!(issue.args.get("age").map(String::as_str), Some("40"));

        assert!(
            reported(Some(40), Some(40)).is_none(),
            "equal ages are exactly what :5189 permits"
        );
        assert!(
            reported(Some(30), Some(40)).is_none(),
            "a younger appearance is the ordinary case"
        );
        assert!(
            reported(Some(50), None).is_none(),
            "with no actual age there is nothing to compare against"
        );
        assert!(
            reported(None, Some(40)).is_none(),
            "with no apparent age there is nothing to compare"
        );
    }

    /// The pending-rolls threshold is a rules NUMBER, so it comes from
    /// `rules/core/aging.json` via `AgingRules::first_roll_age()` and from nowhere
    /// else. A ruleset shipping no aging rules emits nothing — the subsystem stands
    /// down rather than the engine inventing a fallback constant.
    #[test]
    fn the_pending_aging_rolls_finding_reads_its_threshold_from_the_ruleset() {
        let owed = |rs: &Ruleset, age: u32| {
            let mut entity = make_entity("companion", vec![]);
            entity.age = Some(age);
            all_codes(&validate(&entity, rs))
                .contains(&ValidationIssue::CODE_LIFE_STAGE_AGING_ROLLS_PENDING.to_string())
        };

        // No aging rules: no threshold, so nothing is owed at any age.
        let standing_down = ruleset_with_aging(None);
        assert!(
            !owed(&standing_down, 60),
            "with no aging rules the engine has no threshold to enforce"
        );

        // The shipped threshold: "over the age of 35" is strict (`:2232`, `:16565`).
        let rs = aging_ruleset();
        assert!(!owed(&rs, 35), "a character of 35 has not yet begun aging");
        assert!(owed(&rs, 36), "the first roll is owed the year after 35");

        // Move the number in the data and the finding moves with it.
        let late = ruleset_with_aging(Some(
            r#"{
              "start_age": 49,
              "age_divisor": 10,
              "apparent_age_increase_min": 3,
              "outcomes": [
                { "min": 10, "effect": { "type": "any_characteristic", "points": 1 } }
              ]
            }"#,
        ));
        assert!(!owed(&late, 49), "the data's start age owes nothing itself");
        assert!(owed(&late, 50), "and the year after it owes the first roll");
    }

    #[test]
    fn over_budget_virtues() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "small_budget",
          "budget": { "virtue_points": 1, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("small_budget", vec![sel("virtue.a"), sel("virtue.b")]);

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"over_budget_virtues".to_string()),
            "should flag over budget: {:?}",
            codes(&result)
        );
        // args carry the structured values.
        let issue = result
            .errors()
            .find(|i| i.code == "over_budget_virtues")
            .unwrap();
        assert_eq!(issue.args.get("points"), Some(&"2".to_string()));
        assert_eq!(issue.args.get("budget"), Some(&"1".to_string()));
    }

    #[test]
    fn issue_code_consts_match_emitted_codes() {
        // The emitted `code` string equals the public associated const, so
        // consumers can reference the symbol instead of a bare literal.
        assert_eq!(
            ValidationIssue::CODE_OVER_BUDGET_VIRTUES,
            "over_budget_virtues"
        );
        assert_eq!(ValidationIssue::CODE_UNKNOWN_TYPE, "unknown_type");

        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "small_budget",
          "budget": { "virtue_points": 1, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("small_budget", vec![sel("virtue.a"), sel("virtue.b")]);
        let result = validate(&entity, &rs);
        assert!(
            result
                .errors()
                .any(|i| i.code == ValidationIssue::CODE_OVER_BUDGET_VIRTUES),
            "emitted code should equal the const: {:?}",
            codes(&result)
        );
    }

    /// The phase the issue carrying `code` is attributed to.
    fn phase_of(result: &ValidationResult, code: &str) -> CreationPhase {
        result
            .issues
            .iter()
            .find(|i| i.code == code)
            .unwrap_or_else(|| {
                panic!(
                    "expected an issue with code '{code}', got {:?}",
                    codes(result)
                )
            })
            .phase
    }

    /// Every submodule attributes its findings to the phase whose input surface
    /// owns the offending value — not to the phase that happens to detect it. This
    /// is what lets the wizard show a step only its own findings and block Next on
    /// them; an issue filed under the wrong phase would either nag on a step that
    /// cannot fix it or hide on one the user has left behind.
    ///
    /// The codes no creation phase owns (equipment, Might, Warping, aging, and an
    /// entity whose whole type is unknown) go to the terminal `Review` phase.
    #[test]
    fn every_module_attributes_its_issues_to_a_phase() {
        // mod.rs — an unknown type is not fixable on any step: the type is chosen
        // once, at creation.
        let rs = test_ruleset();
        let unknown = validate(&make_entity("nope", vec![]), &rs);
        assert_eq!(
            phase_of(&unknown, ValidationIssue::CODE_UNKNOWN_TYPE),
            CreationPhase::Review
        );

        // balance.rs + prereq.rs + selections.rs — everything about a chosen
        // Virtue or Flaw is fixed on the V/F step.
        let vf = validate(
            &make_entity("companion", vec![sel("virtue.gentle_gift")]),
            &rs,
        );
        assert_eq!(
            phase_of(&vf, ValidationIssue::CODE_PREREQ_NOT_MET),
            CreationPhase::VirtuesFlaws
        );
        assert_eq!(
            phase_of(&vf, ValidationIssue::CODE_FORBIDDEN_CATEGORY),
            CreationPhase::VirtuesFlaws
        );
        let unbalanced = validate(
            &make_entity("companion", vec![sel("virtue.keen_vision")]),
            &rs,
        );
        assert_eq!(
            phase_of(&unbalanced, ValidationIssue::CODE_UNBALANCED_VIRTUES),
            CreationPhase::VirtuesFlaws
        );

        // caps.rs — the dynamic per-category cap codes are V/F findings too.
        let caps_types = r#"[{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10,
                      "flaw_category_caps": [{ "category": "personality", "max": 0 }] },
          "permitted_categories": ["personality"],
          "creation_phases": []
        }]"#;
        let caps_rs = Ruleset::from_json(
            "test",
            "1",
            r#"[{ "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
                  "magnitude": "major", "category": "personality", "entity_kinds": ["character"] }]"#,
            caps_types,
        )
        .unwrap();
        let capped = validate(
            &make_entity("companion", vec![sel("flaw.optimistic")]),
            &caps_rs,
        );
        assert_eq!(
            phase_of(&capped, "too_many_personality_flaws"),
            CreationPhase::VirtuesFlaws
        );

        // scores.rs — characteristics, abilities and personality traits each own
        // their own step, though one module validates all three.
        let aging_rs = aging_ruleset();
        let mut overspent = make_entity("companion", vec![]);
        overspent.characteristics.insert(Characteristic::Str, 3);
        overspent.characteristics.insert(Characteristic::Sta, 3);
        overspent.personality_traits = vec![PersonalityTrait {
            name: "Brave".into(),
            value: 99,
        }];
        let scores = validate(&overspent, &aging_rs);
        assert_eq!(
            phase_of(&scores, ValidationIssue::CODE_CHARACTERISTIC_OVERSPENT),
            CreationPhase::Characteristics
        );
        assert_eq!(
            phase_of(
                &scores,
                ValidationIssue::CODE_PERSONALITY_TRAIT_OUT_OF_RANGE
            ),
            CreationPhase::PersonalityReputations
        );

        let mut unknown_ability = make_entity("companion", vec![]);
        unknown_ability.ability_scores = vec![ability("ability.nonesuch", 1)];
        assert_eq!(
            phase_of(
                &validate(&unknown_ability, &rs),
                ValidationIssue::CODE_UNKNOWN_ABILITY
            ),
            CreationPhase::Abilities
        );

        // aging.rs and equipment.rs — surfaces the wizard has no step for.
        let mut aged = make_entity("companion", vec![]);
        aged.characteristics.insert(Characteristic::Str, 0);
        aged.aging_points.insert(Characteristic::Str, 21);
        assert_eq!(
            phase_of(
                &validate(&aged, &aging_rs),
                ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION
            ),
            CreationPhase::Review
        );

        // might.rs — likewise Review: Might and its powers live in the editor.
        let might_rs = rs_with_houses(MIGHT_ITEMS, GRANT_MAGUS_TYPE);
        let mut over_powers = make_entity("magus", vec![sel("virtue.demonic_blood")]);
        over_powers.powers = vec![power("Curse", 25), power("Shape", 10)];
        assert_eq!(
            phase_of(
                &validate(&over_powers, &might_rs),
                ValidationIssue::CODE_OVER_POWER_LEVELS
            ),
            CreationPhase::Review
        );
    }

    /// The whole `validation/` source, test modules stripped, so a scan sees only
    /// the emit sites that ship.
    fn production_validation_source() -> String {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/validation");
        let mut src = String::new();
        for entry in std::fs::read_dir(dir).expect("read validation dir") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let file = std::fs::read_to_string(&path).expect("read validation source");
            let production = match file.find("#[cfg(test)]") {
                Some(at) => &file[..at],
                None => &file[..],
            };
            src.push_str(production);
            src.push('\n');
        }
        src
    }

    /// `CODE_*` const name → its string value, read out of the source.
    fn issue_code_consts(src: &str) -> BTreeMap<String, String> {
        let marker = concat!("const ", "CODE_");
        src.split(marker)
            .skip(1)
            .filter_map(|seg| {
                let name = format!("CODE_{}", seg.split(':').next()?.trim());
                let start = seg.find('"')? + 1;
                let end = seg[start..].find('"')? + start;
                Some((name, seg[start..end].to_string()))
            })
            .collect()
    }

    /// The contract table as `code → the phases its row lists`.
    fn contract_table_phases(src: &str) -> BTreeMap<String, Vec<String>> {
        src.lines()
            .filter_map(|line| {
                let line = line.trim_start().strip_prefix("///")?.trim();
                // A row ends with `|` too, so drop the trailing empty cell.
                let cells: Vec<&str> = line
                    .strip_prefix('|')?
                    .split('|')
                    .filter(|c| !c.trim().is_empty())
                    .collect();
                if cells.len() < 4 {
                    return None;
                }
                // Skip the header separator (`|---|---|…`).
                if cells.iter().all(|c| c.trim().starts_with('-')) {
                    return None;
                }
                let code = cells[0].trim().trim_matches('`').trim_end_matches('†');
                // Skip the header row itself.
                if code == "code" {
                    return None;
                }
                let phases = cells[2]
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect::<Vec<_>>();
                if phases.is_empty() {
                    return None;
                }
                Some((code.to_string(), phases))
            })
            .collect()
    }

    /// Phases no issue code can name, because they hold no rule the engine
    /// checks: the concept is free text and the type is fixed at creation (a bad
    /// type is `unknown_type`, which is a `review` finding). Asserted below, so
    /// the first code filed under either forces this note to be updated.
    const PHASES_WITH_NO_CODES: [CreationPhase; 2] = [CreationPhase::Concept, CreationPhase::Type];

    /// The contract table's phase column is a real part of the frontend contract —
    /// the wizard filters steps on it — so it may not drift from the emit sites or
    /// invent a phase the engine has no variant for.
    #[test]
    fn contract_table_phase_column_covers_the_creation_phases() {
        let src = production_validation_source();
        let rows = contract_table_phases(&src);
        assert!(
            rows.len() >= 40,
            "expected to find the contract table rows, found {}",
            rows.len()
        );

        let known: BTreeSet<String> = CreationPhase::ALL.iter().map(|p| p.to_string()).collect();
        let mut used = BTreeSet::new();
        for (code, phases) in &rows {
            for phase in phases {
                assert!(
                    known.contains(phase),
                    "row `{code}` names '{phase}', which is not a CreationPhase"
                );
                used.insert(phase.clone());
            }
        }

        let unused: BTreeSet<String> = known.difference(&used).cloned().collect();
        let expected: BTreeSet<String> =
            PHASES_WITH_NO_CODES.iter().map(|p| p.to_string()).collect();
        assert_eq!(
            unused, expected,
            "the phases with no issue codes changed; update PHASES_WITH_NO_CODES and the table's footnote"
        );
    }

    /// Every place the engine builds an issue must name a phase the code's table
    /// row lists. This is the check that keeps attribution honest: the table is
    /// documentation, and documentation drifts unless something reads it.
    #[test]
    fn every_issue_emit_site_names_a_phase_the_table_lists() {
        let src = production_validation_source();
        let consts = issue_code_consts(&src);
        let rows = contract_table_phases(&src);

        let mut sites = 0;
        let mut derived_code_sites = 0;
        let mut forwarded_phase_sites = 0;
        // Split on the three constructors; each segment starts inside the call's
        // argument list, where the code and the phase are the leading arguments.
        for seg in src
            .split("ValidationIssue::error(")
            .skip(1)
            .chain(src.split("ValidationIssue::warning(").skip(1))
            .chain(src.split("ValidationIssue::new(").skip(1))
        {
            let window = &seg[..seg.len().min(600)];
            sites += 1;

            let phase = window
                .find("CreationPhase::")
                .map(|at| {
                    window[at + "CreationPhase::".len()..]
                        .chars()
                        .take_while(char::is_ascii_alphanumeric)
                        .collect::<String>()
                })
                .map(|variant| {
                    CreationPhase::ALL
                        .into_iter()
                        .find(|p| format!("{p:?}") == variant)
                        .unwrap_or_else(|| panic!("emit site names unknown phase '{variant}'"))
                        .to_string()
                });

            let code = window.find("CODE_").map(|at| {
                let name: String = window[at..]
                    .chars()
                    .take_while(|c| c.is_ascii_uppercase() || *c == '_' || c.is_ascii_digit())
                    .collect();
                consts
                    .get(&name)
                    .unwrap_or_else(|| panic!("emit site names unknown const '{name}'"))
                    .clone()
            });

            match (code, phase) {
                (Some(code), Some(phase)) => {
                    let listed = rows
                        .get(&code)
                        .unwrap_or_else(|| panic!("code `{code}` has no contract table row"));
                    assert!(
                        listed.contains(&phase),
                        "an emit site files `{code}` under '{phase}', which its table row ({listed:?}) does not list"
                    );
                }
                // The per-category cap codes are built with `format!`, so the site
                // carries no const; the table's `†` footnote covers them.
                (None, Some(_)) => derived_code_sites += 1,
                // A site that forwards its caller's phase (the shared parameter
                // checks) must be a code the table lists under several phases —
                // that is exactly why the phase is a parameter there.
                (Some(code), None) => {
                    forwarded_phase_sites += 1;
                    let listed = rows
                        .get(&code)
                        .unwrap_or_else(|| panic!("code `{code}` has no contract table row"));
                    assert!(
                        listed.len() > 1,
                        "`{code}` forwards its caller's phase but its row lists only {listed:?}"
                    );
                }
                (None, None) => panic!("an issue emit site names neither a code nor a phase"),
            }
        }

        // 96 shipping sites today (every literal inside a test module is stripped).
        // A floor, not an equality, so adding a validator is not a failing test —
        // but a scanner that stops matching is.
        assert!(
            sites >= 96,
            "expected to find the issue emit sites, found {sites}"
        );
        assert!(
            derived_code_sites >= 2,
            "expected the format!-built category-cap sites, found {derived_code_sites}"
        );
        assert!(
            forwarded_phase_sites >= 3,
            "expected the shared parameter-check sites, found {forwarded_phase_sites}"
        );
    }

    #[test]
    fn every_issue_code_const_is_documented_in_the_contract_table() {
        // Guards the `ValidationIssue` doc-comment contract table against drift:
        // every `CODE_*` const declared anywhere in the `validation/` module must
        // have a matching row in the table, so a newly-added code can't silently
        // ship undocumented.
        // Dynamic per-category cap codes are built with `format!`, not consts, and
        // are covered by the table's `†` footnote rather than a literal row.
        // Scans the whole `validation/` directory (mod.rs + submodules) so a code
        // introduced in any submodule is still checked against the table.
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/validation");
        let mut src = String::new();
        for entry in std::fs::read_dir(dir).expect("read validation dir") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                src.push_str(&std::fs::read_to_string(&path).expect("read validation source"));
                src.push('\n');
            }
        }
        let src = src.as_str();
        // Extract the string value of every issue-code const declaration (handles
        // multi-line declarations: the first quoted string after it wins).
        let marker = concat!("const ", "CODE_");
        let codes: Vec<String> = src
            .split(marker)
            .skip(1)
            .filter_map(|seg| {
                let start = seg.find('"')? + 1;
                let end = seg[start..].find('"')? + start;
                Some(seg[start..end].to_string())
            })
            .collect();
        assert!(
            codes.len() >= 40,
            "expected to find the issue-code consts, found {}",
            codes.len()
        );
        for code in &codes {
            let row = format!("| `{code}` |");
            assert!(
                src.contains(&row),
                "issue code `{code}` has no row in the ValidationIssue contract table"
            );
        }
    }

    #[test]
    fn missing_prerequisite() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.gentle_gift")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn prerequisite_met() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.the_gift"),
                sel("virtue.hermetic_magus"),
                sel("virtue.gentle_gift"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "should have no prereq issues: {:?}",
            result.issues
        );
    }

    #[test]
    fn incompatible_pair() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.the_gift"),
                sel("virtue.hermetic_magus"),
                sel("virtue.gentle_gift"),
                sel("flaw.blatant_gift"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"incompatible".to_string()));
    }

    #[test]
    fn mutual_incompatibility_reported_once() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.the_gift"),
                sel("virtue.hermetic_magus"),
                sel("virtue.gentle_gift"),
                sel("flaw.blatant_gift"),
            ],
        );

        let result = validate(&entity, &rs);
        let incompat_count = result.errors().filter(|i| i.code == "incompatible").count();
        assert_eq!(
            incompat_count, 1,
            "A<->B mutual incompatibility should fire exactly once"
        );
    }

    #[test]
    fn cap_exceeded_major_virtues() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.major_a", "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.major_b", "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "capped_type",
          "budget": { "virtue_points": 10, "flaw_points": 10, "max_major_virtues": 1 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity(
            "capped_type",
            vec![sel("virtue.major_a"), sel("virtue.major_b")],
        );

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"too_many_major_virtues".to_string()));
    }

    /// A ruleset with one minor flaw, one major personality flaw, one minor
    /// personality flaw, one story flaw, and enough minor virtues to balance —
    /// plus a type carrying every new cap. Used by the cap tests below.
    fn caps_ruleset(types: &str) -> Ruleset {
        let items = r#"[
          {"id": "flaw.minor_a", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.minor_b", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.pers_major", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"]},
          {"id": "flaw.pers_minor_a", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "personality", "entity_kinds": ["character"]},
          {"id": "flaw.pers_minor_b", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "personality", "entity_kinds": ["character"]},
          {"id": "flaw.story_a", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "story", "entity_kinds": ["character"]},
          {"id": "flaw.story_b", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "story", "entity_kinds": ["character"]},
          {"id": "virtue.v1", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.v2", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.v3", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        Ruleset::from_json("test", "1", items, types).unwrap()
    }

    fn warning_codes(result: &ValidationResult) -> Vec<String> {
        result.warnings().map(|i| i.code.clone()).collect()
    }

    #[test]
    fn unbalanced_virtues_more_virtues_than_flaws() {
        // Two virtue points, no flaw points: legal on both budgets yet unfunded.
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![sel("virtue.keen_vision"), sel("virtue.large")],
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"unbalanced_virtues".to_string()),
            "virtues exceeding flaws must be flagged: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn balanced_virtues_pass() {
        // Equal virtue and flaw points: balanced and legal.
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![sel("virtue.keen_vision"), sel("flaw.poor_student")],
        );

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"unbalanced_virtues".to_string()),
            "balanced character should not be flagged: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn mythic_flaw_points_fund_double_virtues() {
        // Mythic Companions: each Flaw point funds two Virtue points. Two virtue
        // points on one flaw point is balanced here, but unbalanced at the
        // default 1:1 rate. Source: Core Rules.md:2638.
        let types = r#"[{
          "id": "mythic",
          "budget": { "virtue_points": 20, "flaw_points": 10, "virtue_points_per_flaw_point": 2 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "mythic",
            vec![sel("virtue.v1"), sel("virtue.v2"), sel("flaw.minor_a")],
        );

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"unbalanced_virtues".to_string()),
            "2 virtue points funded by 1 flaw point at the 2:1 rate is balanced: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn mythic_virtues_beyond_double_are_unbalanced() {
        // Three virtue points on one flaw point exceeds even the 2:1 funding.
        let types = r#"[{
          "id": "mythic",
          "budget": { "virtue_points": 20, "flaw_points": 10, "virtue_points_per_flaw_point": 2 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "mythic",
            vec![
                sel("virtue.v1"),
                sel("virtue.v2"),
                sel("virtue.v3"),
                sel("flaw.minor_a"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"unbalanced_virtues".to_string()),
            "3 virtue points exceed 1 flaw point's 2:1 funding: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn too_many_minor_flaws() {
        let types = r#"[{
          "id": "capped",
          "budget": { "virtue_points": 10, "flaw_points": 10, "max_minor_flaws": 1 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "capped",
            vec![
                sel("flaw.minor_a"),
                sel("flaw.minor_b"),
                sel("virtue.v1"),
                sel("virtue.v2"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"too_many_minor_flaws".to_string()),
            "two minor flaws over a cap of one: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn minor_flaws_at_cap_pass() {
        let types = r#"[{
          "id": "capped",
          "budget": { "virtue_points": 10, "flaw_points": 10, "max_minor_flaws": 5 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "capped",
            vec![sel("flaw.minor_a"), sel("flaw.minor_b"), sel("virtue.v1")],
        );

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"too_many_minor_flaws".to_string()),
            "two minor flaws under a cap of five: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn minor_flaws_no_cap_imposes_no_limit() {
        // No max_minor_flaws on the type: any number is allowed.
        let types = r#"[{
          "id": "uncapped",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "uncapped",
            vec![
                sel("flaw.minor_a"),
                sel("flaw.minor_b"),
                sel("virtue.v1"),
                sel("virtue.v2"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"too_many_minor_flaws".to_string()),
            "absent cap should not limit minor flaws: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn too_many_major_personality_flaws_is_error() {
        // pers_major (3) is a Major Personality Flaw; cap of zero forbids it.
        let types = r#"[{
          "id": "capped",
          "budget": {
            "virtue_points": 10,
            "flaw_points": 10,
            "flaw_category_caps": [
              { "category": "personality", "max": 0, "major_only": true, "hard": true }
            ]
          },
          "permitted_categories": ["general", "personality"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "capped",
            vec![
                sel("flaw.pers_major"),
                sel("virtue.v1"),
                sel("virtue.v2"),
                sel("virtue.v3"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"too_many_major_personality_flaws".to_string()),
            "major personality flaw over a cap of zero (hard error): {:?}",
            codes(&result)
        );
    }

    #[test]
    fn too_many_personality_flaws_is_warning() {
        // Two personality flaws over a soft cap of one → non-blocking warning.
        let types = r#"[{
          "id": "capped",
          "budget": {
            "virtue_points": 10,
            "flaw_points": 10,
            "flaw_category_caps": [{ "category": "personality", "max": 1 }]
          },
          "permitted_categories": ["general", "personality"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "capped",
            vec![
                sel("flaw.pers_minor_a"),
                sel("flaw.pers_minor_b"),
                sel("virtue.v1"),
                sel("virtue.v2"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            warning_codes(&result).contains(&"too_many_personality_flaws".to_string()),
            "should warn, not error: warnings {:?}, errors {:?}",
            warning_codes(&result),
            codes(&result)
        );
        assert!(
            !codes(&result).contains(&"too_many_personality_flaws".to_string()),
            "soft guideline must not be a blocking error"
        );
    }

    #[test]
    fn too_many_story_flaws_is_warning() {
        let types = r#"[{
          "id": "capped",
          "budget": {
            "virtue_points": 10,
            "flaw_points": 10,
            "flaw_category_caps": [{ "category": "story", "max": 1 }]
          },
          "permitted_categories": ["general", "story"],
          "creation_phases": []
        }]"#;
        let rs = caps_ruleset(types);
        let entity = make_entity(
            "capped",
            vec![
                sel("flaw.story_a"),
                sel("flaw.story_b"),
                sel("virtue.v1"),
                sel("virtue.v2"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            warning_codes(&result).contains(&"too_many_story_flaws".to_string()),
            "two story flaws over a soft cap of one: warnings {:?}",
            warning_codes(&result)
        );
        assert!(!codes(&result).contains(&"too_many_story_flaws".to_string()));
    }

    #[test]
    fn forbidden_category() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.the_gift"),
                sel("virtue.hermetic_magus"),
                sel("virtue.gentle_gift"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"forbidden_category".to_string()),
            "companion can't take hermetic: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn validation_mode_enforced_preserves_errors() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.gentle_gift")]);

        let result = validate(&entity, &rs).apply_mode(ValidationMode::Enforced);
        assert!(!result.is_valid());
    }

    #[test]
    fn validation_mode_advisory_downgrades_to_warnings() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.gentle_gift")]);

        let result = validate(&entity, &rs).apply_mode(ValidationMode::Advisory);
        assert!(result.is_valid(), "advisory mode should have no errors");
        assert!(!result.issues.is_empty(), "should still have warnings");
        assert!(
            result
                .issues
                .iter()
                .all(|i| i.severity == IssueSeverity::Warning)
        );
    }

    #[test]
    fn validation_mode_silent_clears_all() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.gentle_gift")]);

        let result = validate(&entity, &rs).apply_mode(ValidationMode::Silent);
        assert!(result.is_valid());
        assert!(result.issues.is_empty());
    }

    #[test]
    fn balance_computation() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.keen_vision"),
                sel("virtue.large"),
                sel("flaw.poor_student"),
            ],
        );

        let balance = compute_balance(&entity, &rs);
        assert_eq!(balance.virtue_points, 2, "two minor virtues = 2 points");
        assert_eq!(balance.flaw_points, 1, "one minor flaw = 1 point");
    }

    #[test]
    fn unknown_selection_ref() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.nonexistent")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"unknown_ref".to_string()));
    }

    #[test]
    fn compute_balance_skips_unknown_ref_no_panic() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![sel("virtue.keen_vision"), sel("virtue.does_not_exist")],
        );
        // Only the known minor virtue counts; unknown ref is skipped silently.
        let balance = compute_balance(&entity, &rs);
        assert_eq!(
            balance,
            Balance {
                virtue_points: 1,
                flaw_points: 0
            }
        );
    }

    #[test]
    fn entity_save_canonical_roundtrip() {
        let mut entity = make_entity(
            "companion",
            vec![
                sel("virtue.puissant_ability"),
                sel("flaw.poor_student"),
                sel("virtue.keen_vision"),
            ],
        );
        entity.normalize();

        let json1 = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json1).unwrap();
        assert_eq!(entity, roundtripped);

        let json2 = serde_json::to_string_pretty(&roundtripped).unwrap();
        assert_eq!(json1, json2, "canonical serialization should be stable");
    }

    #[test]
    fn issue_without_context_omits_field_and_roundtrips() {
        let issue = ValidationIssue::error(
            "over_budget_virtues",
            CreationPhase::VirtuesFlaws,
            BTreeMap::new(),
            None,
        );

        let json = serde_json::to_string(&issue).unwrap();
        assert!(
            !json.contains("context"),
            "a None context must be omitted from JSON: {json}"
        );
        assert!(
            json.contains(r#""phase":"virtues_flaws""#),
            "the phase is always emitted: {json}"
        );

        let roundtripped: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(issue, roundtripped);

        // Deserialization must also accept JSON that omits `context` entirely.
        // `phase` is required — the frontend filters steps on it, so an issue
        // without one would be silently ungated rather than merely unlabelled.
        let without_context =
            r#"{"severity":"error","code":"over_budget_virtues","phase":"virtues_flaws"}"#;
        let parsed: ValidationIssue = serde_json::from_str(without_context).unwrap();
        assert_eq!(parsed.context, None);
        assert_eq!(parsed.phase, CreationPhase::VirtuesFlaws);
    }

    #[test]
    fn public_constructors_build_issues_and_results() {
        let issue = ValidationIssue::new(
            IssueSeverity::Warning,
            "too_many_story_flaws",
            CreationPhase::VirtuesFlaws,
            BTreeMap::new(),
            None,
        );
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.code, "too_many_story_flaws");
        assert_eq!(issue.phase, CreationPhase::VirtuesFlaws);

        let result = ValidationResult::new(vec![
            ValidationIssue::error("unknown_type", CreationPhase::Review, BTreeMap::new(), None),
            issue,
        ]);
        assert!(!result.is_valid(), "an error makes the result invalid");
        assert_eq!(result.errors().count(), 1);
        assert_eq!(result.warnings().count(), 1);
        assert!(ValidationResult::default().issues.is_empty());
    }

    #[test]
    fn unknown_type_id() {
        let rs = test_ruleset();
        let entity = make_entity("nonexistent_type", vec![]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"unknown_type".to_string()));
        let issue = result.errors().find(|i| i.code == "unknown_type").unwrap();
        assert_eq!(
            issue.args.get("type_id"),
            Some(&"nonexistent_type".to_string())
        );
    }

    #[test]
    fn over_budget_flaws() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "flaw.a", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.b", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "small_flaw_budget",
          "budget": { "virtue_points": 10, "flaw_points": 1 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("small_flaw_budget", vec![sel("flaw.a"), sel("flaw.b")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"over_budget_flaws".to_string()));
    }

    #[test]
    fn too_many_major_flaws() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "flaw.major_a", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.major_b", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "capped_flaws",
          "budget": { "virtue_points": 10, "flaw_points": 10, "max_major_flaws": 1 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity(
            "capped_flaws",
            vec![sel("flaw.major_a"), sel("flaw.major_b")],
        );

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"too_many_major_flaws".to_string()));
    }

    fn all_prereq_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "all", "value": [{"kind": "has", "value": "virtue.a"}, {"kind": "has", "value": "virtue.b"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        Ruleset::from_json("test", "1", items, types).unwrap()
    }

    #[test]
    fn prereq_all_satisfied() {
        let rs = all_prereq_ruleset();
        let entity = make_entity(
            "test_type",
            vec![sel("virtue.a"), sel("virtue.b"), sel("virtue.c")],
        );

        let result = validate(&entity, &rs);
        assert!(!codes(&result).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn prereq_all_unsatisfied() {
        let rs = all_prereq_ruleset();
        let entity = make_entity("test_type", vec![sel("virtue.a"), sel("virtue.c")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"prereq_not_met".to_string()));
    }

    fn any_prereq_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "any", "value": [{"kind": "has", "value": "virtue.a"}, {"kind": "has", "value": "virtue.b"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        Ruleset::from_json("test", "1", items, types).unwrap()
    }

    #[test]
    fn prereq_any_satisfied() {
        let rs = any_prereq_ruleset();
        let entity = make_entity("test_type", vec![sel("virtue.a"), sel("virtue.c")]);

        let result = validate(&entity, &rs);
        assert!(!codes(&result).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn prereq_any_unsatisfied() {
        let rs = any_prereq_ruleset();
        let entity = make_entity("test_type", vec![sel("virtue.c")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"prereq_not_met".to_string()));
    }

    fn none_prereq_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "none", "value": [{"kind": "has", "value": "virtue.a"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        Ruleset::from_json("test", "1", items, types).unwrap()
    }

    #[test]
    fn prereq_none_satisfied() {
        let rs = none_prereq_ruleset();
        let entity = make_entity("test_type", vec![sel("virtue.b")]);

        let result = validate(&entity, &rs);
        assert!(!codes(&result).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn prereq_none_unsatisfied() {
        let rs = none_prereq_ruleset();
        let entity = make_entity("test_type", vec![sel("virtue.a"), sel("virtue.b")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"prereq_not_met".to_string()));
    }

    // --- Tri-state composition with unevaluable leaves ---

    #[test]
    fn prereq_none_with_unknown_leaf_does_not_false_positive() {
        // None([House]) must NOT collapse to a spurious failure: an unevaluable
        // leaf yields Unknown, so no prereq_not_met error fires.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "none", "value": [{"kind": "house", "value": "house.flambeau"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = rs_with_houses(items, types);
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "unknown leaf under None must not produce a false failure"
        );
        let warnings: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(warnings.contains(&"prereq_unevaluated"));
    }

    #[test]
    fn prereq_any_satisfied_does_not_warn_on_unknown_sibling() {
        // Any([Has(a)=true, House=unknown]) short-circuits to True; the
        // unknown sibling must NOT produce a warning since the result does not
        // depend on it.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "any", "value": [{"kind": "has", "value": "virtue.a"}, {"kind": "house", "value": "house.flambeau"}]}},
          {"id": "flaw.x", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.y", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = rs_with_houses(items, types);
        // Two minor virtues balanced by two minor flaws so the test isolates
        // prerequisite-warning behaviour, not the points balance.
        let entity = make_entity(
            "test_type",
            vec![
                sel("virtue.a"),
                sel("virtue.b"),
                sel("flaw.x"),
                sel("flaw.y"),
            ],
        );

        let result = validate(&entity, &rs);
        assert!(result.is_valid(), "issues: {:?}", result.issues);
        let warnings: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            !warnings.contains(&"prereq_unevaluated"),
            "satisfied Any must not warn about its unknown sibling: {warnings:?}"
        );
    }

    #[test]
    fn prereq_all_with_known_failure_does_not_warn() {
        // All([Has(missing)=false, House=unknown]) -> False; report
        // prereq_not_met, NOT an unevaluated warning.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.dep", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "all", "value": [{"kind": "has", "value": "virtue.dep"}, {"kind": "house", "value": "house.x"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        // virtue.dep exists but is NOT selected, so Has(virtue.dep) is False.
        let rs = rs_with_houses(items, types);
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"prereq_not_met".to_string()));
        let warnings: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            !warnings.contains(&"prereq_unevaluated"),
            "known failure should not also warn: {warnings:?}"
        );
    }

    #[test]
    fn prereq_any_no_true_branch_with_unknown_resolves_to_unevaluated() {
        // Any([Has(dep)=false, House=unknown]) -> no True branch, one Unknown
        // branch -> Unknown that depends on the unevaluable leaf. Exercises the
        // Any Unknown-resolution path: no prereq_not_met, a prereq_unevaluated
        // warning instead.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.dep", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "any", "value": [{"kind": "has", "value": "virtue.dep"}, {"kind": "house", "value": "house.flambeau"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        // virtue.dep is NOT selected, so Has(virtue.dep) is False.
        let rs = rs_with_houses(items, types);
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "Any with an unknown branch must not fail outright: {:?}",
            codes(&result)
        );
        let warnings: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            warnings.contains(&"prereq_unevaluated"),
            "unresolved Any should warn: {warnings:?}"
        );
    }

    #[test]
    fn prereq_all_satisfied_leaf_with_unknown_sibling_warns() {
        // All([Has(a)=true, House=unknown]) -> Unknown that genuinely hinges on
        // the unevaluable leaf: no prereq_not_met error, but a
        // prereq_unevaluated warning must fire.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "all", "value": [{"kind": "has", "value": "virtue.a"}, {"kind": "house", "value": "house.flambeau"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = rs_with_houses(items, types);
        let entity = make_entity("test_type", vec![sel("virtue.a"), sel("virtue.b")]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "satisfied leaf + unknown sibling must not fail: {:?}",
            codes(&result)
        );
        let warnings: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            warnings.contains(&"prereq_unevaluated"),
            "result hinges on unknown leaf, should warn: {warnings:?}"
        );
    }

    #[test]
    fn prereq_house_produces_unevaluated_warning() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "house", "value": "house.bjornaer"}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = rs_with_houses(items, types);
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(warning_codes.contains(&"prereq_unevaluated"));
    }

    #[test]
    fn prereq_house_met_when_entity_belongs_to_that_house() {
        // House(bjornaer) is satisfied when the entity's own house matches: the
        // leaf is now evaluable (True), so no error and no unevaluated warning.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "house", "value": "house.bjornaer"}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = rs_with_houses(items, types);
        let mut entity = make_entity("test_type", vec![sel("virtue.a")]);
        entity.house = Some(Id::new("house.bjornaer"));

        let result = validate(&entity, &rs);
        let all_codes = codes(&result);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            !all_codes.contains(&"prereq_not_met".to_string()),
            "matching house satisfies the prereq: {all_codes:?}"
        );
        assert!(
            !warning_codes.contains(&"prereq_unevaluated"),
            "an evaluable house leaf must not warn: {warning_codes:?}"
        );
    }

    #[test]
    fn prereq_house_not_met_when_entity_belongs_to_a_different_house() {
        // The entity is in house.x but the virtue requires house.bjornaer: the
        // leaf is a definite False, so a hard prereq error fires (no warning).
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "house", "value": "house.bjornaer"}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = rs_with_houses(items, types);
        let mut entity = make_entity("test_type", vec![sel("virtue.a")]);
        entity.house = Some(Id::new("house.x"));

        let result = validate(&entity, &rs);
        let all_codes = codes(&result);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            all_codes.contains(&"prereq_not_met".to_string()),
            "a different house is a definite failure: {all_codes:?}"
        );
        assert!(
            !warning_codes.contains(&"prereq_unevaluated"),
            "a definite False must not warn: {warning_codes:?}"
        );
    }

    /// A ruleset whose `virtue.a` requires Awareness 3. Returns (ruleset).
    fn ability_min_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "ability_min", "value": {"ability": "ability.awareness", "score": 3}}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let abilities =
            r#"{ "abilities": [{ "id": "ability.awareness", "category": "general" }] }"#;
        Ruleset::from_json_with_abilities("test", "1", items, types, abilities).unwrap()
    }

    #[test]
    fn ability_min_met_when_bought_score_at_or_above_threshold() {
        let rs = ability_min_ruleset();
        let mut entity = make_entity("test_type", vec![sel("virtue.a")]);
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3,
            specialty: None,
            parameter: None,
        }];

        let result = validate(&entity, &rs);
        // The lone minor virtue is unbalanced (no funding flaw), so the entity is
        // not fully valid; what matters here is that the prereq itself is met.
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "Awareness 3 satisfies the min: {result:?}"
        );
    }

    #[test]
    fn ability_min_not_met_when_bought_score_below_threshold() {
        let rs = ability_min_ruleset();
        let mut entity = make_entity("test_type", vec![sel("virtue.a")]);
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: None,
            parameter: None,
        }];

        let result = validate(&entity, &rs);
        let codes: Vec<String> = codes(&result);
        assert!(codes.contains(&"prereq_not_met".to_string()), "{codes:?}");
    }

    #[test]
    fn ability_min_absent_ability_counts_as_zero_and_fails() {
        let rs = ability_min_ruleset();
        // No ability scores at all -> Awareness counts as 0 -> below 3 -> error,
        // NOT an unevaluated warning (AbilityMin is now decidable).
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let error_codes = codes(&result);
        assert!(
            error_codes.contains(&"prereq_not_met".to_string()),
            "{error_codes:?}"
        );
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(!warning_codes.contains(&"prereq_unevaluated"));
    }

    /// A ruleset with Puissant Ability (+2), Great Characteristic (raises the buy
    /// cap, up to 2/characteristic), Poor Characteristic (lowers the buy floor),
    /// a virtue requiring Awareness 3, the characteristic table extended to ±5
    /// with base limits ±3 and effective limits ±5, and a funding flaw. For the
    /// characteristic-limit validation tests.
    fn effective_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.requires_awareness_3", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "ability_min", "value": {"ability": "ability.awareness", "score": 3}}},
          {"id": "virtue.puissant_ability", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "ability", "type": "ref", "domain": "ability"}],
           "effects": [{"type": "ability_bonus", "param": "ability", "amount": 2}]},
          {"id": "virtue.great_characteristic", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "characteristic", "type": "ref", "domain": "characteristic"}],
           "effects": [{"type": "characteristic_limit", "param": "characteristic", "amount": 1}],
           "max_per_target": 2},
          {"id": "virtue.improved_characteristics", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "effects": [{"type": "characteristic_points", "amount": 3}]},
          {"id": "flaw.poor_characteristic", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "characteristic", "type": "ref", "domain": "characteristic"}],
           "effects": [{"type": "characteristic_limit", "param": "characteristic", "amount": -1}],
           "max_per_target": 2},
          {"id": "flaw.f", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let abilities = r#"{ "abilities": [
          { "id": "ability.awareness", "category": "general" },
          { "id": "ability.stealth", "category": "general" },
          { "id": "ability.area_lore", "category": "general", "parameter": "area" }
        ] }"#;
        let characteristics = r#"{
          "start_points": 7,
          "base_max": 3, "base_min": -3, "effective_max": 5, "effective_min": -5,
          "costs": [
            { "score": 5, "cost": 15 }, { "score": 4, "cost": 10 },
            { "score": 3, "cost": 6 }, { "score": 2, "cost": 3 }, { "score": 1, "cost": 1 },
            { "score": 0, "cost": 0 },
            { "score": -1, "cost": -1 }, { "score": -2, "cost": -3 }, { "score": -3, "cost": -6 },
            { "score": -4, "cost": -10 }, { "score": -5, "cost": -15 }
          ]
        }"#;
        Ruleset::from_core_json("test", "1", items, types, abilities, Some(characteristics))
            .unwrap()
    }

    #[test]
    fn ability_min_met_via_puissant_bonus() {
        let rs = effective_ruleset();
        let mut entity = make_entity(
            "companion",
            vec![
                sel("virtue.requires_awareness_3"),
                puissant("ability.awareness"),
            ],
        );
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 1,
            specialty: None,
            parameter: None,
        }];
        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"prereq_not_met".to_string()),
            "Awareness base 1 + Puissant +2 = 3 meets the min: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn ability_min_not_met_at_base_one_without_bonus() {
        let rs = effective_ruleset();
        let mut entity = make_entity("companion", vec![sel("virtue.requires_awareness_3")]);
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 1,
            specialty: None,
            parameter: None,
        }];
        assert!(codes(&validate(&entity, &rs)).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn score_four_without_great_is_above_cap_not_off_table() {
        // +4 is a legal table value (it must be priced), but the default cap is
        // +3, so buying it without Great Characteristic is "above cap", NOT
        // "out of range".
        let rs = effective_ruleset();
        let mut entity = companion_entity_eff();
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 4)]);
        let c = codes(&validate(&entity, &rs));
        assert!(c.contains(&"characteristic_above_cap".to_string()), "{c:?}");
        assert!(
            !c.contains(&"characteristic_out_of_range".to_string()),
            "{c:?}"
        );
    }

    #[test]
    fn score_six_is_off_table_out_of_range() {
        let rs = effective_ruleset();
        let mut entity = companion_entity_eff();
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 6)]);
        assert!(
            codes(&validate(&entity, &rs)).contains(&"characteristic_out_of_range".to_string())
        );
    }

    #[test]
    fn score_four_with_one_great_is_within_cap() {
        // base 4 (≥ +3 precondition met) + one Great → cap +4, score +4 fits.
        let rs = effective_ruleset();
        let mut entity = make_entity("companion", vec![great(Characteristic::Str)]);
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 4)]);
        let c = codes(&validate(&entity, &rs));
        assert!(
            !c.contains(&"characteristic_above_cap".to_string()),
            "{c:?}"
        );
        assert!(
            !c.contains(&"characteristic_out_of_range".to_string()),
            "{c:?}"
        );
        assert!(
            !c.contains(&"characteristic_max_base_too_low".to_string()),
            "{c:?}"
        );
    }

    #[test]
    fn score_five_needs_two_greats() {
        let rs = effective_ruleset();
        // One Great only opens the cap to +4, so +5 is still above cap...
        let mut one = make_entity("companion", vec![great(Characteristic::Str)]);
        one.characteristics = BTreeMap::from([(Characteristic::Str, 5)]);
        assert!(
            codes(&validate(&one, &rs)).contains(&"characteristic_above_cap".to_string()),
            "one Great should leave +5 above cap"
        );
        // ...two Greats open it to +5.
        let mut two = make_entity(
            "companion",
            vec![great(Characteristic::Str), great(Characteristic::Str)],
        );
        two.characteristics = BTreeMap::from([(Characteristic::Str, 5)]);
        assert!(
            !codes(&validate(&two, &rs)).contains(&"characteristic_above_cap".to_string()),
            "two Greats should allow +5"
        );
    }

    #[test]
    fn great_on_base_two_is_max_base_too_low() {
        let rs = effective_ruleset();
        let mut entity = make_entity("companion", vec![sel("flaw.f"), great(Characteristic::Str)]);
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 2)]);
        assert!(
            codes(&validate(&entity, &rs)).contains(&"characteristic_max_base_too_low".to_string())
        );
    }

    #[test]
    fn score_minus_four_without_poor_is_below_floor_not_off_table() {
        let rs = effective_ruleset();
        let mut entity = companion_entity_eff();
        entity.characteristics = BTreeMap::from([(Characteristic::Qik, -4)]);
        let c = codes(&validate(&entity, &rs));
        assert!(
            c.contains(&"characteristic_below_floor".to_string()),
            "{c:?}"
        );
        assert!(
            !c.contains(&"characteristic_out_of_range".to_string()),
            "{c:?}"
        );
    }

    #[test]
    fn score_minus_five_needs_two_poors() {
        let rs = effective_ruleset();
        // One Poor only opens the floor to −4, so −5 is still below floor...
        let mut one = make_entity("companion", vec![poor(Characteristic::Qik)]);
        one.characteristics = BTreeMap::from([(Characteristic::Qik, -5)]);
        assert!(
            codes(&validate(&one, &rs)).contains(&"characteristic_below_floor".to_string()),
            "one Poor should leave −5 below floor"
        );
        // ...two Poors open it to −5.
        let mut two = make_entity(
            "companion",
            vec![poor(Characteristic::Qik), poor(Characteristic::Qik)],
        );
        two.characteristics = BTreeMap::from([(Characteristic::Qik, -5)]);
        assert!(
            !codes(&validate(&two, &rs)).contains(&"characteristic_below_floor".to_string()),
            "two Poors should allow −5"
        );
    }

    #[test]
    fn poor_on_base_minus_two_is_min_base_too_high() {
        let rs = effective_ruleset();
        let mut entity = make_entity("companion", vec![sel("flaw.f"), poor(Characteristic::Qik)]);
        entity.characteristics = BTreeMap::from([(Characteristic::Qik, -2)]);
        assert!(
            codes(&validate(&entity, &rs))
                .contains(&"characteristic_min_base_too_high".to_string())
        );
    }

    #[test]
    fn great_characteristic_twice_same_is_allowed() {
        let rs = effective_ruleset();
        let mut entity = make_entity(
            "companion",
            vec![great(Characteristic::Str), great(Characteristic::Str)],
        );
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
        assert!(!codes(&validate(&entity, &rs)).contains(&"duplicate_selection".to_string()));
    }

    #[test]
    fn great_characteristic_thrice_same_is_error() {
        let rs = effective_ruleset();
        let mut entity = make_entity(
            "companion",
            vec![
                great(Characteristic::Str),
                great(Characteristic::Str),
                great(Characteristic::Str),
            ],
        );
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 3)]);
        assert!(codes(&validate(&entity, &rs)).contains(&"duplicate_selection".to_string()));
    }

    #[test]
    fn improved_characteristics_raises_the_buy_budget() {
        let rs = effective_ruleset();
        // Str 4 costs 10; the base budget is 7, so this overspends by 3 — unless
        // Improved Characteristics (+3) lifts the budget to 10.
        let spend = BTreeMap::from([(Characteristic::Str, 4)]);
        let mut bare = make_entity("companion", vec![]);
        bare.characteristics = spend.clone();
        assert!(codes(&validate(&bare, &rs)).contains(&"characteristic_overspent".to_string()));

        let mut improved = make_entity("companion", vec![sel("virtue.improved_characteristics")]);
        improved.characteristics = spend;
        let codes = codes(&validate(&improved, &rs));
        assert!(
            !codes.contains(&"characteristic_overspent".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn puissant_same_ability_twice_is_error() {
        let rs = effective_ruleset();
        let entity = make_entity(
            "companion",
            vec![puissant("ability.awareness"), puissant("ability.awareness")],
        );
        assert!(codes(&validate(&entity, &rs)).contains(&"duplicate_selection".to_string()));
    }

    #[test]
    fn puissant_two_different_abilities_is_allowed() {
        let rs = effective_ruleset();
        let entity = make_entity(
            "companion",
            vec![puissant("ability.awareness"), puissant("ability.stealth")],
        );
        assert!(!codes(&validate(&entity, &rs)).contains(&"duplicate_selection".to_string()));
    }

    /// Puissant on a parameterized ability instance: ability id + the instance
    /// value under the ability's own param key (`area`).
    fn puissant_instance(ability: &str, key: &str, value: &str) -> Selection {
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([
                ("ability".into(), Id::new(ability)),
                (key.into(), Id::new(value)),
            ]),
        )
    }

    fn lore_score(area: &str, score: u8) -> AbilityScore {
        AbilityScore {
            ability: Id::new("ability.area_lore"),
            score,
            specialty: None,
            parameter: Some(area.to_string()),
        }
    }

    #[test]
    fn puissant_dangling_when_target_ability_not_held() {
        let rs = effective_ruleset();
        // Puissant Awareness but the character never bought Awareness.
        let entity = make_entity("companion", vec![puissant("ability.awareness")]);
        assert!(
            codes(&validate(&entity, &rs)).contains(&"ability_bonus_dangling_target".to_string()),
            "a Puissant whose target ability is absent should dangle: {:?}",
            codes(&validate(&entity, &rs))
        );
    }

    #[test]
    fn puissant_plain_target_held_is_not_dangling() {
        let rs = effective_ruleset();
        let mut entity = make_entity("companion", vec![puissant("ability.awareness")]);
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: None,
            parameter: None,
        }];
        assert!(
            !codes(&validate(&entity, &rs)).contains(&"ability_bonus_dangling_target".to_string()),
        );
    }

    #[test]
    fn puissant_parameterized_requires_the_instance_key() {
        let rs = effective_ruleset();
        // (Area) Lore is parameterized: naming only the ability id, not the area,
        // is a missing param.
        let mut entity = make_entity("companion", vec![puissant("ability.area_lore")]);
        entity.ability_scores = vec![lore_score("Brandenburg", 2)];
        assert!(
            codes(&validate(&entity, &rs)).contains(&"missing_param".to_string()),
            "a parameterized Puissant target needs its instance key: {:?}",
            codes(&validate(&entity, &rs))
        );
    }

    #[test]
    fn puissant_parameterized_matching_instance_is_clean() {
        let rs = effective_ruleset();
        let mut entity = make_entity(
            "companion",
            vec![puissant_instance(
                "ability.area_lore",
                "area",
                "Brandenburg",
            )],
        );
        entity.ability_scores = vec![lore_score("Brandenburg", 2)];
        let cs = codes(&validate(&entity, &rs));
        assert!(
            !cs.contains(&"missing_param".to_string())
                && !cs.contains(&"unexpected_param".to_string())
                && !cs.contains(&"ability_bonus_dangling_target".to_string()),
            "a Puissant naming a held instance should be clean: {cs:?}"
        );
    }

    #[test]
    fn puissant_parameterized_dangles_when_that_instance_absent() {
        let rs = effective_ruleset();
        // Targets Brandenburg Lore, but only Berlin Lore is held.
        let mut entity = make_entity(
            "companion",
            vec![puissant_instance(
                "ability.area_lore",
                "area",
                "Brandenburg",
            )],
        );
        entity.ability_scores = vec![lore_score("Berlin", 2)];
        assert!(
            codes(&validate(&entity, &rs)).contains(&"ability_bonus_dangling_target".to_string()),
        );
    }

    #[test]
    fn stray_instance_key_on_plain_target_is_unexpected() {
        let rs = effective_ruleset();
        // Awareness is plain; an extra `area` key is not expected.
        let mut entity = make_entity(
            "companion",
            vec![puissant_instance(
                "ability.awareness",
                "area",
                "Brandenburg",
            )],
        );
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: None,
            parameter: None,
        }];
        assert!(codes(&validate(&entity, &rs)).contains(&"unexpected_param".to_string()));
    }

    fn companion_entity_eff() -> Entity {
        make_entity("companion", vec![])
    }

    /// A ruleset carrying the canonical characteristic cost table + a couple of
    /// abilities, for the characteristic/ability validator tests.
    fn traits_ruleset() -> Ruleset {
        let types = r#"[{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 },
            { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }
          ],
          "abilities": [
            { "id": "ability.awareness", "category": "general" },
            { "id": "ability.living_language", "category": "general" },
            { "id": "ability.area_lore", "category": "general", "parameter": "area" }
          ]
        }"#;
        let characteristics = r#"{
          "start_points": 7,
          "costs": [
            { "score": 3, "cost": 6 }, { "score": 2, "cost": 3 }, { "score": 1, "cost": 1 },
            { "score": 0, "cost": 0 },
            { "score": -1, "cost": -1 }, { "score": -2, "cost": -3 }, { "score": -3, "cost": -6 }
          ]
        }"#;
        Ruleset::from_core_json("test", "1", "[]", types, abilities, Some(characteristics)).unwrap()
    }

    fn companion_entity() -> Entity {
        make_entity("companion", vec![])
    }

    #[test]
    fn characteristics_balanced_spend_is_valid() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        // Int +3 (6) + Per +1 (1) = 7 = start points.
        entity.characteristics =
            BTreeMap::from([(Characteristic::Int, 3), (Characteristic::Per, 1)]);
        let result = validate(&entity, &rs);
        assert!(result.is_valid(), "exactly 7 points spent: {result:?}");
    }

    #[test]
    fn characteristics_overspend_is_error() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        // Int +3 (6) + Per +2 (3) = 9 > 7.
        entity.characteristics =
            BTreeMap::from([(Characteristic::Int, 3), (Characteristic::Per, 2)]);
        assert!(codes(&validate(&entity, &rs)).contains(&"characteristic_overspent".to_string()));
    }

    #[test]
    fn untouched_characteristics_produce_no_points_warning() {
        let rs = traits_ruleset();
        let entity = companion_entity(); // no characteristics set
        let result = validate(&entity, &rs);
        let warnings: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            !warnings.contains(&"characteristic_points_unspent"),
            "a fresh character should not be nagged: {warnings:?}"
        );
    }

    #[test]
    fn characteristics_underspend_is_warning() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.characteristics = BTreeMap::from([(Characteristic::Int, 1)]); // costs 1 < 7
        let result = validate(&entity, &rs);
        assert!(result.is_valid(), "underspend does not block");
        let warnings: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            warnings.contains(&"characteristic_points_unspent"),
            "{warnings:?}"
        );
    }

    #[test]
    fn characteristic_out_of_range_is_error() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 4)]); // table max is 3
        assert!(
            codes(&validate(&entity, &rs)).contains(&"characteristic_out_of_range".to_string())
        );
    }

    #[test]
    fn unknown_ability_is_error() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.nonexistent"),
            score: 2,
            specialty: None,
            parameter: None,
        }];
        assert!(codes(&validate(&entity, &rs)).contains(&"unknown_ability".to_string()));
    }

    #[test]
    fn covenant_entity_skips_character_only_validation() {
        // A covenant carries neither Characteristics nor Abilities. Even if stray
        // character data is present, the engine must not run the character-only
        // validators for a covenant-kind entity (gating is on EntityKind, not on
        // data emptiness). The same data on a Character entity DOES get flagged.
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.characteristics = BTreeMap::from([(Characteristic::Str, 4)]); // out of range
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.nonexistent"),
            score: 2,
            specialty: None,
            parameter: None,
        }];

        // As a character, both stray fields are flagged.
        let char_codes = codes(&validate(&entity, &rs));
        assert!(char_codes.contains(&"characteristic_out_of_range".to_string()));
        assert!(char_codes.contains(&"unknown_ability".to_string()));

        // As a covenant, the character-only validators are skipped entirely.
        entity.entity_kind = EntityKind::Covenant;
        let cov_codes = codes(&validate(&entity, &rs));
        assert!(
            !cov_codes.contains(&"characteristic_out_of_range".to_string()),
            "covenant must not get characteristic validation: {cov_codes:?}"
        );
        assert!(
            !cov_codes.contains(&"unknown_ability".to_string()),
            "covenant must not get ability validation: {cov_codes:?}"
        );
    }

    #[test]
    fn duplicate_ability_same_specialty_is_error() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.awareness"),
                score: 2,
                specialty: None,
                parameter: None,
            },
            AbilityScore {
                ability: Id::new("ability.awareness"),
                score: 3,
                specialty: None,
                parameter: None,
            },
        ];
        assert!(codes(&validate(&entity, &rs)).contains(&"duplicate_ability".to_string()));
    }

    #[test]
    fn ability_xp_within_pool_is_valid() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        // Awareness 3 costs 30 xp; pool of 40 covers it.
        entity.xp_pool = 40;
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3,
            specialty: None,
            parameter: None,
        }];
        assert!(
            !codes(&validate(&entity, &rs)).contains(&"not_enough_xp".to_string()),
            "30 xp spent within a 40 pool"
        );
    }

    #[test]
    fn overspending_ability_xp_is_error() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        // Awareness 3 costs 30 xp; pool of 10 is not enough.
        entity.xp_pool = 10;
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3,
            specialty: None,
            parameter: None,
        }];
        assert!(codes(&validate(&entity, &rs)).contains(&"not_enough_xp".to_string()));
    }

    /// A minimal ruleset carrying both an Ability and an Art registry (each with a
    /// triangular advancement table, scores 1-5) for the shared-pool tests. The
    /// companion profile permits the general category so the entity has no
    /// unrelated findings.
    fn arts_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "gift_policy": "forbidden",
          "gift_id": "virtue.the_gift",
          "creation_phases": []
        }]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ],
          "abilities": [ { "id": "ability.awareness", "category": "general" } ]
        }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }
          ],
          "arts": [
            { "id": "art.creo", "art_type": "technique" },
            { "id": "art.ignem", "art_type": "form" }
          ]
        }"#;
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn art_xp_within_pool_is_valid() {
        let rs = arts_ruleset();
        let mut e = make_entity("companion", vec![]);
        // Creo 5 (15) + Ignem 3 (6) = 21 xp; the shared pool of 21 is enough.
        e.xp_pool = 21;
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 5,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 3,
            },
        ];
        let found = codes(&validate(&e, &rs));
        assert!(!found.contains(&"not_enough_xp".to_string()), "{found:?}");
    }

    #[test]
    fn overspending_art_xp_is_error() {
        let rs = arts_ruleset();
        let mut e = make_entity("companion", vec![]);
        // Creo 5 costs 15 xp; the shared pool of 10 is not enough.
        e.xp_pool = 10;
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 5,
        }];
        assert!(codes(&validate(&e, &rs)).contains(&"not_enough_xp".to_string()));
    }

    #[test]
    fn abilities_and_arts_share_one_pool() {
        // Awareness 2 (15) + Creo 3 (6) = 21 drawn from the same bank; a pool of
        // 20 overspends by 1, a pool of 21 does not.
        let rs = arts_ruleset();
        let mut e = make_entity("companion", vec![]);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: None,
            parameter: None,
        }];
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 3,
        }];
        e.xp_pool = 20;
        assert!(codes(&validate(&e, &rs)).contains(&"not_enough_xp".to_string()));
        e.xp_pool = 21;
        assert!(!codes(&validate(&e, &rs)).contains(&"not_enough_xp".to_string()));
    }

    /// Arts ruleset plus the Phase-3 restricted-pool and Affinity virtues, with
    /// Latin/Artes-Liberales (academic) and Awareness (general) abilities.
    fn restricted_xp_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.educated", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "effects": [{ "type": "restricted_ability_xp", "amount": 50, "abilities": ["ability.latin", "ability.artes_liberales"] }]},
          {"id": "virtue.affinity_art", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
           "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
           "effects": [{ "type": "affinity_art_cost", "param": "art", "counts_as_num": 3, "counts_as_den": 2 }]}
        ]"#;
        let types = r#"[{
          "id": "companion",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general", "hermetic"],
          "gift_policy": "forbidden",
          "gift_id": "virtue.the_gift",
          "creation_phases": []
        }]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ],
          "abilities": [
            { "id": "ability.latin", "category": "academic" },
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.awareness", "category": "general" }
          ]
        }"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }
          ],
          "arts": [ { "id": "art.creo", "art_type": "technique" } ]
        }"#;
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: Some(arts),
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn restricted_xp_left_unspent_warns() {
        // Educated grants 50 XP restricted to Latin/Artes Liberales; with none of
        // it spent, the rules waste it — a non-blocking warning, not an error.
        let rs = restricted_xp_ruleset();
        let mut e = make_entity("companion", vec![sel("virtue.educated")]);
        e.xp_pool = 0;
        let result = validate(&e, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_RESTRICTED_XP_UNSPENT)
            .unwrap_or_else(|| panic!("{:?}", warning_codes(&result)));
        assert_eq!(issue.severity, IssueSeverity::Warning);
        // The pool names its granting item, so the UI can localize "Educated" out of
        // the ruleset's own i18n rather than saying "restricted" twice over.
        assert_eq!(
            issue.args.get("origin_kind").map(String::as_str),
            Some("item")
        );
        assert_eq!(
            issue.args.get("origin").map(String::as_str),
            Some("virtue.educated")
        );
        assert!(!codes(&result).contains(&"not_enough_xp".to_string()));
    }

    #[test]
    fn restricted_xp_unspent_does_not_fire_when_eligible_spend_fills_it() {
        // A3: an eligible Artes Liberales spend the restricted pool fully covers
        // must consume the restricted pool (not the general pool), so no
        // restricted_xp_unspent warning fires despite a large general pool.
        let rs = restricted_xp_ruleset();
        let mut e = make_entity("companion", vec![sel("virtue.educated")]);
        e.xp_pool = 100;
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.artes_liberales"),
            score: 4, // 50 xp = the whole Educated pool
            specialty: None,
            parameter: None,
        }];
        let result = validate(&e, &rs);
        assert!(
            !warning_codes(&result).contains(&"restricted_xp_unspent".to_string()),
            "{:?}",
            warning_codes(&result)
        );
        assert!(!codes(&result).contains(&"not_enough_xp".to_string()));
    }

    #[test]
    fn restricted_pool_cannot_fund_an_ineligible_ability() {
        // Educated's 50 can only buy Latin/Artes Lib; Awareness (general) must come
        // from the general pool, which is empty → overspend.
        let rs = restricted_xp_ruleset();
        let mut e = make_entity("companion", vec![sel("virtue.educated")]);
        e.xp_pool = 0;
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2, // 15 xp
            specialty: None,
            parameter: None,
        }];
        assert!(codes(&validate(&e, &rs)).contains(&"not_enough_xp".to_string()));
    }

    #[test]
    fn affinity_makes_an_otherwise_overspent_art_fit() {
        let rs = restricted_xp_ruleset();
        let creo5 = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 5, // table cost 15
        }];
        // Without Affinity, Creo 5 costs 15 and a pool of 10 overspends.
        let mut bare = make_entity("companion", vec![]);
        bare.xp_pool = 10;
        bare.art_scores = creo5.clone();
        assert!(codes(&validate(&bare, &rs)).contains(&"not_enough_xp".to_string()));
        // With Affinity with Creo, the charged cost drops to ceil(15·2/3)=10 → fits.
        let mut affined = make_entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.affinity_art"),
                BTreeMap::from([("art".into(), Id::new("art.creo"))]),
            )],
        );
        affined.xp_pool = 10;
        affined.art_scores = creo5;
        assert!(!codes(&validate(&affined, &rs)).contains(&"not_enough_xp".to_string()));
    }

    #[test]
    fn unknown_art_is_flagged() {
        let rs = arts_ruleset();
        let mut e = make_entity("companion", vec![]);
        e.xp_pool = 100;
        e.art_scores = vec![ArtScore {
            art: Id::new("art.made_up"),
            score: 1,
        }];
        assert!(codes(&validate(&e, &rs)).contains(&"unknown_art".to_string()));
    }

    #[test]
    fn duplicate_art_is_flagged() {
        let rs = arts_ruleset();
        let mut e = make_entity("companion", vec![]);
        e.xp_pool = 100;
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 2,
            },
            ArtScore {
                art: Id::new("art.creo"),
                score: 3,
            },
        ];
        assert!(codes(&validate(&e, &rs)).contains(&"duplicate_art".to_string()));
    }

    #[test]
    fn off_table_art_score_is_out_of_range() {
        let rs = arts_ruleset(); // Art table tops out at score 5
        let mut e = make_entity("companion", vec![]);
        e.xp_pool = 1000;
        e.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 9,
        }];
        let found = codes(&validate(&e, &rs));
        assert_eq!(
            found
                .iter()
                .filter(|c| *c == "art_score_out_of_range")
                .count(),
            1,
            "exactly one art_score_out_of_range issue: {found:?}"
        );
    }

    #[test]
    fn off_table_ability_score_is_out_of_range() {
        let rs = traits_ruleset(); // advancement table tops out at score 3
        let mut entity = companion_entity();
        entity.xp_pool = 1000; // generous, so the only finding is the range error
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 9, // far above the table max
            specialty: None,
            parameter: None,
        }];
        let found = codes(&validate(&entity, &rs));
        assert_eq!(
            found
                .iter()
                .filter(|c| *c == "ability_score_out_of_range")
                .count(),
            1,
            "exactly one ability_score_out_of_range issue: {found:?}"
        );
    }

    #[test]
    fn in_range_ability_score_is_not_out_of_range() {
        let rs = traits_ruleset(); // advancement table tops out at score 3
        let mut entity = companion_entity();
        entity.xp_pool = 1000;
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3, // the table's max — in range
            specialty: None,
            parameter: None,
        }];
        assert!(
            !codes(&validate(&entity, &rs)).contains(&"ability_score_out_of_range".to_string()),
            "an in-range score must not be flagged"
        );
    }

    #[test]
    fn raising_an_ability_with_no_pool_is_error() {
        let rs = traits_ruleset();
        let mut entity = companion_entity(); // xp_pool defaults to 0
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 1,
            specialty: None,
            parameter: None,
        }];
        assert!(codes(&validate(&entity, &rs)).contains(&"not_enough_xp".to_string()));
    }

    #[test]
    fn selected_ability_at_score_zero_costs_no_xp() {
        let rs = traits_ruleset();
        let mut entity = companion_entity(); // no pool
        // Selecting an ability without raising it (score 0) spends nothing.
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 0,
            specialty: None,
            parameter: None,
        }];
        assert!(
            !codes(&validate(&entity, &rs)).contains(&"not_enough_xp".to_string()),
            "a score-0 ability is free"
        );
    }

    #[test]
    fn same_ability_different_parameter_is_allowed() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.xp_pool = 100;
        // Two languages: same catalogue row, distinct parameter values.
        entity.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.living_language"),
                score: 5,
                specialty: None,
                parameter: Some("German".into()),
            },
            AbilityScore {
                ability: Id::new("ability.living_language"),
                score: 1,
                specialty: None,
                parameter: Some("Latin".into()),
            },
        ];
        assert!(
            !codes(&validate(&entity, &rs)).contains(&"duplicate_ability".to_string()),
            "distinct parameters are not duplicates"
        );
    }

    #[test]
    fn duplicate_ability_same_parameter_is_error() {
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.xp_pool = 100;
        entity.ability_scores = vec![
            AbilityScore {
                ability: Id::new("ability.living_language"),
                score: 5,
                specialty: None,
                parameter: Some("German".into()),
            },
            AbilityScore {
                ability: Id::new("ability.living_language"),
                score: 2,
                specialty: None,
                parameter: Some("German".into()),
            },
        ];
        assert!(codes(&validate(&entity, &rs)).contains(&"duplicate_ability".to_string()));
    }

    #[test]
    fn parameterized_ability_without_value_is_error() {
        // traits_ruleset marks ability.area_lore as parameterized.
        let rs = traits_ruleset();
        let mut entity = companion_entity();
        entity.xp_pool = 100;
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.area_lore"),
            score: 1,
            specialty: None,
            parameter: None,
        }];
        assert!(codes(&validate(&entity, &rs)).contains(&"ability_parameter_required".to_string()));
    }

    /// A ruleset whose `virtue.a` requires Creo ≥ 5, with an Art registry so the
    /// ArtMin prereq is genuinely evaluable.
    fn art_min_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "art_min", "value": {"art": "art.creo", "score": 5}}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let arts = r#"{
          "advancement": [
            { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
            { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
            { "score": 5, "total_xp": 15 }, { "score": 6, "total_xp": 21 }
          ],
          "arts": [ { "id": "art.creo", "art_type": "technique" } ]
        }"#;
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: Some("{}"),
            arts: Some(arts),
            ..Default::default()
        })
        .unwrap()
    }

    #[test]
    fn prereq_art_min_is_evaluated_against_effective_score() {
        let rs = art_min_ruleset();

        // Creo 5 satisfies the threshold: no prereq error or unevaluated warning.
        let mut met = make_entity("test_type", vec![sel("virtue.a")]);
        met.xp_pool = 100;
        met.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 5,
        }];
        let result = validate(&met, &rs);
        assert!(!codes(&result).contains(&"prereq_not_met".to_string()));
        assert!(
            !result
                .warnings()
                .any(|i| i.code.as_str() == "prereq_unevaluated"),
            "ArtMin should be enforced, not unevaluated"
        );

        // Creo 4 falls short: prereq_not_met fires (it is evaluated, not deferred).
        let mut unmet = make_entity("test_type", vec![sel("virtue.a")]);
        unmet.xp_pool = 100;
        unmet.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 4,
        }];
        assert!(codes(&validate(&unmet, &rs)).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn prereq_is_magus_satisfied_on_magus_type() {
        // A profile flagged `is_magus: true` satisfies IsMagus: no warning, no error.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "is_magus"}}
        ]"#;
        let types = r#"[{
          "id": "magus_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "is_magus": true,
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("magus_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            !warning_codes.contains(&"prereq_unevaluated"),
            "IsMagus should be enforced, not unevaluated: {warning_codes:?}"
        );
        assert!(!codes(&result).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn prereq_is_magus_fails_on_non_magus_type() {
        // A profile flagged `is_magus: false` makes IsMagus False: prereq_not_met fires.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "is_magus"}}
        ]"#;
        let types = r#"[{
          "id": "grog_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "is_magus": false,
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("grog_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"prereq_not_met".to_string()),
            "magus-only item must fail on a non-magus type: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn prereq_is_magus_unknown_without_profile() {
        // No matching type profile -> IsMagus is Unknown -> warning.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "is_magus"}}
        ]"#;
        let types = r#"[{
          "id": "some_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        // Entity references a type id with no matching profile.
        let entity = make_entity("no_such_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(warning_codes.contains(&"prereq_unevaluated"));
    }

    #[test]
    fn prereq_is_magus_independent_of_gift_ungifted_redcap() {
        // An unGifted Redcap is modeled as a companion-style profile: NOT a
        // magus AND the Gift is forbidden. IsMagus must still fail, proving the
        // flag is decoupled from gift_policy.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "is_magus"}}
        ]"#;
        let types = r#"[{
          "id": "ungifted_redcap",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "is_magus": false,
          "gift_policy": "forbidden",
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("ungifted_redcap", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"prereq_not_met".to_string()),
            "an unGifted Redcap is not a magus: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn prereq_is_magus_independent_of_gift_gifted_hedge_wizard() {
        // A Gifted hedge wizard HAS The Gift but is NOT a magus. Having the Gift
        // selected must not make IsMagus pass: prereq_not_met still fires.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "is_magus"}}
        ]"#;
        let types = r#"[{
          "id": "hedge_wizard",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general", "special"],
          "is_magus": false,
          "gift_policy": "allowed",
          "gift_id": "virtue.the_gift",
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity(
            "hedge_wizard",
            vec![sel("virtue.the_gift"), sel("virtue.a")],
        );

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"prereq_not_met".to_string()),
            "having The Gift does not make a magus: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn prereq_is_magus_satisfied_on_magus_type_regardless_of_gift_fields() {
        // Sanity: the `is_magus` flag drives IsMagus, not the gift fields. A
        // magus profile with gift_policy=required and the Gift selected passes.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "is_magus"}}
        ]"#;
        let types = r#"[{
          "id": "magus_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general", "special"],
          "is_magus": true,
          "gift_policy": "required",
          "gift_id": "virtue.the_gift",
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("magus_type", vec![sel("virtue.the_gift"), sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(
            !warning_codes.contains(&"prereq_unevaluated"),
            "IsMagus should be enforced: {warning_codes:?}"
        );
        assert!(!codes(&result).contains(&"prereq_not_met".to_string()));
    }

    #[test]
    fn wrong_entity_kind() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.char_only", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "standard_covenant",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let mut entity = make_entity("standard_covenant", vec![sel("virtue.char_only")]);
        entity.entity_kind = EntityKind::Covenant;

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"wrong_entity_kind".to_string()));
        // entity_kind rendered via Display (snake_case), not Debug.
        let issue = result
            .errors()
            .find(|i| i.code == "wrong_entity_kind")
            .unwrap();
        assert_eq!(issue.args.get("entity_kind"), Some(&"covenant".to_string()));
    }

    #[test]
    fn empty_entity_kinds_valid_for_any_kind() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.universal", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": []}
        ]"#;
        let types = r#"[{
          "id": "standard_covenant",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let mut entity = make_entity("standard_covenant", vec![sel("virtue.universal")]);
        entity.entity_kind = EntityKind::Covenant;

        let result = validate(&entity, &rs);
        assert!(!codes(&result).contains(&"wrong_entity_kind".to_string()));
    }

    #[test]
    fn profile_mandated_gift_is_exempt_from_permitted_categories() {
        // Regression: a profile that REQUIRES The Gift (via gift_policy) but does
        // not list the Gift's `special` category in permitted_categories must not
        // reject the very trait it mandates. The Gift is governed solely by
        // validate_gift_policy; the category check exempts the profile's gift_id.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "magus_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general", "hermetic"],
          "is_magus": true,
          "gift_policy": "required",
          "gift_id": "virtue.the_gift",
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("magus_type", vec![sel("virtue.the_gift")]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"category_not_permitted".to_string()),
            "a profile's mandated gift must be exempt from the category check: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn empty_permitted_categories_permits_any() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.weird", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "obscure", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "unrestricted",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("unrestricted", vec![sel("virtue.weird")]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"category_not_permitted".to_string()),
            "empty permitted_categories should allow any category: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn empty_forbidden_categories_forbids_nothing() {
        // Mirror of empty_permitted_categories_permits_any: a profile with no
        // forbidden_categories must not flag any selection, whatever its
        // category.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.hermetic_thing", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "unrestricted",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("unrestricted", vec![sel("virtue.hermetic_thing")]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"forbidden_category".to_string()),
            "empty forbidden_categories should forbid nothing: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn gift_policy_required_without_gift() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "magus_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general", "special"],
          "gift_policy": "required",
          "gift_id": "virtue.the_gift",
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("magus_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"gift_required".to_string()));
    }

    #[test]
    fn gift_policy_required_with_gift() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "special", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "magus_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general", "special"],
          "gift_policy": "required",
          "gift_id": "virtue.the_gift",
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("magus_type", vec![sel("virtue.the_gift")]);

        let result = validate(&entity, &rs);
        assert!(!codes(&result).contains(&"gift_required".to_string()));
    }

    #[test]
    fn gift_policy_required_via_category() {
        // Required gift, no gift_id, gift_categories=[hermetic], a hermetic
        // selection satisfies it (symmetry: category counts for Required too).
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.parma", "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "magus_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["hermetic"],
          "gift_policy": "required",
          "gift_categories": ["hermetic"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();

        // With a hermetic selection: satisfied.
        let entity = make_entity("magus_type", vec![sel("virtue.parma")]);
        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"gift_required".to_string()),
            "hermetic category should satisfy Required gift: {:?}",
            codes(&result)
        );

        // Without it: fails.
        let empty = make_entity("magus_type", vec![]);
        let result = validate(&empty, &rs);
        assert!(codes(&result).contains(&"gift_required".to_string()));
    }

    #[test]
    fn gift_policy_forbidden_with_gift() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.the_gift")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"gift_forbidden".to_string()));
    }

    #[test]
    fn gift_policy_forbidden_via_category() {
        // Forbidden gift via gift_categories=[hermetic] with NO gift_id: a
        // hermetic selection must still trip gift_forbidden (category path).
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.parma", "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "no_gift_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["hermetic"],
          "gift_policy": "forbidden",
          "gift_categories": ["hermetic"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("no_gift_type", vec![sel("virtue.parma")]);

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"gift_forbidden".to_string()),
            "hermetic category should trip forbidden gift: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn gift_policy_forbidden_via_category_does_not_fire_on_non_matching_category() {
        // Forbidden gift, no gift_id, gift_categories=[hermetic]. A selection
        // whose category is NOT hermetic must NOT trip gift_forbidden: the
        // category path's negative branch.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.mundane", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "no_gift_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "gift_policy": "forbidden",
          "gift_categories": ["hermetic"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("no_gift_type", vec![sel("virtue.mundane")]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"gift_forbidden".to_string()),
            "a non-hermetic selection must not trip forbidden gift: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn gift_policy_required_without_gift_id() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "no_gift_id_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "gift_policy": "required",
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("no_gift_id_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"gift_required".to_string()));
    }

    #[test]
    fn missing_required_trait() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.mandatory", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "strict_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "required_traits": ["virtue.mandatory"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("strict_type", vec![]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"missing_required_trait".to_string()));
    }

    #[test]
    fn forbidden_trait_selected() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.banned", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "restricted_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "forbidden_traits": ["virtue.banned"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("restricted_type", vec![sel("virtue.banned")]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"forbidden_trait".to_string()));
    }

    #[test]
    fn duplicate_selection_non_parameterized() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![sel("virtue.keen_vision"), sel("virtue.keen_vision")],
        );

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"duplicate_selection".to_string()));
    }

    #[test]
    fn duplicate_selection_parameterized() {
        let rs = test_ruleset();
        let param_sel = Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
        );
        let entity = make_entity("companion", vec![param_sel.clone(), param_sel]);

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"duplicate_selection".to_string()));
    }

    #[test]
    fn parameterized_item_with_different_params_not_duplicate() {
        let rs = test_ruleset();
        let a = Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
        );
        let b = Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".into(), Id::new("ability.brawl"))]),
        );
        let entity = make_entity("companion", vec![a, b]);

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"duplicate_selection".to_string()),
            "same item with different params is legal: {:?}",
            codes(&result)
        );
    }

    // --- Parameter validation ---

    #[test]
    fn missing_required_param() {
        let rs = test_ruleset();
        // puissant_ability declares an `ability` param but none provided.
        let entity = make_entity("companion", vec![sel("virtue.puissant_ability")]);

        let result = validate(&entity, &rs);
        assert!(
            codes(&result).contains(&"missing_param".to_string()),
            "should flag missing param: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn unexpected_param() {
        let rs = test_ruleset();
        // keen_vision has no params; supplying one is an error.
        let entity = make_entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.keen_vision"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            )],
        );

        let result = validate(&entity, &rs);
        assert!(codes(&result).contains(&"unexpected_param".to_string()));
    }

    #[test]
    fn well_formed_param_passes() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            )],
        );

        let result = validate(&entity, &rs);
        assert!(
            !codes(&result).contains(&"missing_param".to_string())
                && !codes(&result).contains(&"unexpected_param".to_string()),
            "well-formed ability param should pass: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn item_domain_param_value_resolved() {
        // A parameterized item whose domain is `item` must resolve its value
        // against the point-item registry.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.target", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.linked", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "linked", "type": "ref", "domain": "item"}]}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();

        // Good value.
        let good = make_entity(
            "test_type",
            vec![Selection::with_params(
                Id::new("virtue.linked"),
                BTreeMap::from([("linked".into(), Id::new("virtue.target"))]),
            )],
        );
        assert!(
            !codes(&validate(&good, &rs)).contains(&"unknown_param_value".to_string()),
            "valid item-domain value should resolve"
        );

        // Bad value.
        let bad = make_entity(
            "test_type",
            vec![Selection::with_params(
                Id::new("virtue.linked"),
                BTreeMap::from([("linked".into(), Id::new("virtue.ghost"))]),
            )],
        );
        assert!(
            codes(&validate(&bad, &rs)).contains(&"unknown_param_value".to_string()),
            "unknown item-domain value should be flagged"
        );
    }

    #[test]
    fn text_domain_param_accepts_any_free_text_value() {
        // A `text` domain is a free-text slot (e.g. Aptitude for (Sin)): any
        // non-empty value the player types is legal — no registry resolution.
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.aptitude", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "sin", "type": "ref", "domain": "text"}]}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let e = make_entity(
            "test_type",
            vec![Selection::with_params(
                Id::new("virtue.aptitude"),
                BTreeMap::from([("sin".into(), Id::new("Pride"))]),
            )],
        );
        assert!(
            !codes(&validate(&e, &rs)).contains(&"unknown_param_value".to_string()),
            "a free-text value must not be resolved against any registry"
        );
    }

    #[test]
    fn art_domain_param_value_resolves_against_registry() {
        // Art-domain parameter values are now resolved against the Art catalogue:
        // a real Art passes, a made-up one raises `unknown_param_value`.
        let items = r#"[
          { "id": "virtue.puissant_art", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
            "category": "general", "entity_kinds": ["character"],
            "parameters": [{"key": "art", "type": "ref", "domain": "art"}],
            "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
            "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let arts = r#"{ "arts": [ { "id": "art.creo", "art_type": "technique" } ] }"#;
        let rs = Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: types,
            abilities: Some("{}"),
            arts: Some(arts),
            ..Default::default()
        })
        .unwrap();

        let good = make_entity(
            "test_type",
            vec![Selection::with_params(
                Id::new("virtue.puissant_art"),
                BTreeMap::from([("art".into(), Id::new("art.creo"))]),
            )],
        );
        assert!(
            !codes(&validate(&good, &rs))
                .contains(&ValidationIssue::CODE_UNKNOWN_PARAM_VALUE.to_string()),
            "a real Art-domain value must resolve: {:?}",
            codes(&validate(&good, &rs))
        );

        let bad = make_entity(
            "test_type",
            vec![Selection::with_params(
                Id::new("virtue.puissant_art"),
                BTreeMap::from([("art".into(), Id::new("art.totally_made_up"))]),
            )],
        );
        assert!(
            codes(&validate(&bad, &rs))
                .contains(&ValidationIssue::CODE_UNKNOWN_PARAM_VALUE.to_string()),
            "an unknown Art-domain value must be flagged: {:?}",
            codes(&validate(&bad, &rs))
        );
    }

    #[test]
    fn compute_balance_major_and_free() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          {"id": "virtue.major", "kind": "virtue", "classification": "narrative", "magnitude": "major", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.free", "kind": "virtue", "classification": "narrative", "magnitude": "free", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.minor", "kind": "flaw", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity(
            "test_type",
            vec![sel("virtue.major"), sel("virtue.free"), sel("flaw.minor")],
        );

        let balance = compute_balance(&entity, &rs);
        assert_eq!(
            balance.virtue_points, 3,
            "one major virtue = 3 pts, one free = 0 pts"
        );
        assert_eq!(balance.flaw_points, 1, "one minor flaw = 1 pt");
    }

    #[test]
    fn compute_balance_empty() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![]);

        let balance = compute_balance(&entity, &rs);
        assert_eq!(balance.virtue_points, 0);
        assert_eq!(balance.flaw_points, 0);
    }

    #[test]
    fn entity_normalize_sorts_selections() {
        let mut entity = make_entity(
            "companion",
            vec![
                sel("virtue.tough"),
                sel("flaw.poor_student"),
                sel("virtue.keen_vision"),
            ],
        );

        entity.normalize();

        let refs: Vec<&str> = entity
            .selections
            .iter()
            .map(|s| s.item_ref.as_str())
            .collect();
        assert_eq!(
            refs,
            vec!["flaw.poor_student", "virtue.keen_vision", "virtue.tough"]
        );
    }

    #[test]
    fn empty_entity_validates() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![]);

        let result = validate(&entity, &rs);
        assert!(
            result.is_valid(),
            "empty entity should be valid: {:?}",
            result.issues
        );
    }

    #[test]
    fn issue_args_carry_offending_ids() {
        let rs = test_ruleset();

        // unknown_ref: args should contain the bad ID under "item".
        let entity = make_entity("companion", vec![sel("virtue.nonexistent")]);
        let result = validate(&entity, &rs);
        let unknown_ref = result.errors().find(|i| i.code == "unknown_ref").unwrap();
        assert_eq!(
            unknown_ref.args.get("item"),
            Some(&"virtue.nonexistent".to_string())
        );

        // unknown_type: args should contain the bad type_id.
        let entity = make_entity("bogus_type", vec![]);
        let result = validate(&entity, &rs);
        let unknown_type = result.errors().find(|i| i.code == "unknown_type").unwrap();
        assert_eq!(
            unknown_type.args.get("type_id"),
            Some(&"bogus_type".to_string())
        );
    }

    #[test]
    fn validation_result_errors_vs_warnings() {
        let result = ValidationResult {
            issues: vec![
                ValidationIssue::error("err1", CreationPhase::Review, BTreeMap::new(), None),
                ValidationIssue::warning("warn1", CreationPhase::Review, BTreeMap::new(), None),
                ValidationIssue::error("err2", CreationPhase::Review, BTreeMap::new(), None),
            ],
        };

        assert_eq!(result.errors().count(), 2);
        assert_eq!(result.warnings().count(), 1);
        assert!(!result.is_valid());
    }

    #[test]
    fn apply_mode_advisory_preserves_issue_count() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.gentle_gift")]);

        let result = validate(&entity, &rs);
        let original_count = result.issues.len();
        assert!(original_count > 0, "should have issues to test with");

        let advisory = validate(&entity, &rs).apply_mode(ValidationMode::Advisory);
        assert_eq!(advisory.issues.len(), original_count);
    }

    // --- Mythic Companion types: effective budget + validate_mythic_type -----

    /// A ruleset with a mythic-companion profile (`has_mythic_type`), a plain
    /// companion profile (base 10/10), and two types: Devil Child (+3 free V,
    /// +7 F) and Faerie Doctor (no bonus).
    fn mythic_ruleset() -> Ruleset {
        const ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
            { "id": "virtue.devil_child", "kind": "virtue", "classification": "narrative", "magnitude": "free",
              "category": "social_status", "entity_kinds": ["character"] },
            { "id": "virtue.demonic_might", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "virtue.demonic_powers", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "virtue.demonic_mark", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
              "category": "supernatural", "entity_kinds": ["character"],
              "parameters": [{ "key": "characteristic", "type": "ref", "domain": "characteristic" }] },
            { "id": "virtue.demonic_blood", "kind": "virtue", "classification": "narrative", "magnitude": "major",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "flaw.tragic_life", "kind": "flaw", "classification": "narrative", "magnitude": "major",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "flaw.other_supernatural", "kind": "flaw", "classification": "narrative", "magnitude": "major",
              "category": "supernatural", "entity_kinds": ["character"] }
        ]"#;
        const PROFILES: &str = r#"[
            { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
              "creation_phases": [] },
            { "id": "mythic_companion", "has_mythic_type": true,
              "budget": { "virtue_points": 20, "flaw_points": 10, "virtue_points_per_flaw_point": 2 },
              "permitted_categories": ["general", "supernatural", "social_status"],
              "creation_phases": [] }
        ]"#;
        const TYPES: &str = r#"{ "types": [
            { "id": "mythic_type.devil_child",
              "grants": [
                { "kind": "fixed", "item": "virtue.devil_child" },
                { "kind": "choice", "choice_key": "devil_child_might", "options": [
                  { "ref": "virtue.demonic_might" }, { "ref": "virtue.demonic_powers" } ] } ],
              "required_virtues": [ { "ref": "virtue.demonic_blood" } ],
              "required_flaws": [ { "default": { "ref": "flaw.tragic_life" },
                "constraint": { "kind": "flaw", "magnitude": "major", "require_categories": ["supernatural"] } } ],
              "bonus_flaw_points": 7, "bonus_free_virtue_points": 3 },
            { "id": "mythic_type.faerie_doctor" },
            { "id": "mythic_type.open_child",
              "grants": [
                { "kind": "open", "choice_key": "open_child_virtue",
                  "constraint": { "kind": "virtue", "magnitude": "minor", "require_categories": ["supernatural"] } } ] }
        ] }"#;
        Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: ITEMS,
            type_profiles: PROFILES,
            abilities: None,
            arts: None,
            houses: None,
            mythic_types: Some(TYPES),
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
            life_stages: None,
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    fn mythic_entity(type_id: &str) -> Entity {
        let mut e = make_entity("mythic_companion", vec![]);
        e.mythic_type = Some(Id::new(type_id));
        e
    }

    #[test]
    fn effective_budget_folds_devil_child_bonuses() {
        let rs = mythic_ruleset();
        let profile = rs.profile(&Id::new("mythic_companion")).unwrap();
        let e = mythic_entity("mythic_type.devil_child");
        let b = effective_budget(&e, &rs, profile);
        // 20 + 7·2 + 3 = 37; 10 + 7 = 17; funded(17) = 17·2 + 3 = 37; the free
        // headroom funds 3 virtue points with no flaws.
        assert_eq!(b.virtue_ceiling, 37);
        assert_eq!(b.flaw_ceiling, 17);
        assert_eq!(b.funded(17), 37);
        assert_eq!(b.funded(0), 3);
    }

    #[test]
    fn effective_budget_is_base_when_type_has_no_bonus() {
        let rs = mythic_ruleset();
        let profile = rs.profile(&Id::new("mythic_companion")).unwrap();
        let e = mythic_entity("mythic_type.faerie_doctor");
        let b = effective_budget(&e, &rs, profile);
        assert_eq!(b.virtue_ceiling, 20);
        assert_eq!(b.flaw_ceiling, 10);
        assert_eq!(b.funded(10), 20); // rate 2, no free headroom
        assert_eq!(b.funded(0), 0);
    }

    #[test]
    fn effective_budget_reduces_to_profile_for_non_mythic() {
        let rs = mythic_ruleset();
        let profile = rs.profile(&Id::new("companion")).unwrap();
        let e = make_entity("companion", vec![]); // no mythic_type
        let b = effective_budget(&e, &rs, profile);
        assert_eq!(b.virtue_ceiling, 10);
        assert_eq!(b.flaw_ceiling, 10);
        assert_eq!(b.funded(10), 10); // rate 1
        assert_eq!(b.funded(0), 0);
    }

    #[test]
    fn effective_point_ceilings_surface_the_display_budget() {
        let rs = mythic_ruleset();
        // Devil Child: 37 V / 17 F (base 20/10 + 7·2 + 3 free / +7 F).
        let devil = mythic_entity("mythic_type.devil_child");
        assert_eq!(
            effective_point_ceilings(&devil, &rs),
            Some(PointCeilings {
                virtue_ceiling: 37,
                flaw_ceiling: 17
            })
        );
        // Faerie Doctor (no bonus) and a plain companion stay at their base.
        let faerie = mythic_entity("mythic_type.faerie_doctor");
        assert_eq!(
            effective_point_ceilings(&faerie, &rs),
            Some(PointCeilings {
                virtue_ceiling: 20,
                flaw_ceiling: 10
            })
        );
        let companion = make_entity("companion", vec![]);
        assert_eq!(
            effective_point_ceilings(&companion, &rs),
            Some(PointCeilings {
                virtue_ceiling: 10,
                flaw_ceiling: 10
            })
        );
    }

    #[test]
    fn unchosen_mythic_type_warns() {
        let rs = mythic_ruleset();
        let e = make_entity("mythic_companion", vec![]); // has_mythic_type but no type
        let codes = all_codes(&validate(&e, &rs));
        assert!(codes.contains(&ValidationIssue::CODE_MYTHIC_TYPE_UNSET.to_string()));
    }

    #[test]
    fn missing_required_package_warns_and_clears_when_present() {
        let rs = mythic_ruleset();
        // Devil Child chosen, Might/Powers picked, but no required package bought.
        let mut e = mythic_entity("mythic_type.devil_child");
        e.mythic_choices.insert(
            "devil_child_might".to_string(),
            Selection::new(Id::new("virtue.demonic_might")),
        );
        let before = all_codes(&validate(&e, &rs));
        assert!(before.contains(&ValidationIssue::CODE_MYTHIC_REQUIRED_TRAIT_MISSING.to_string()));

        // Buy Demonic Blood + Tragic Life → the required-package warning clears.
        e.selections = vec![sel("virtue.demonic_blood"), sel("flaw.tragic_life")];
        let after = all_codes(&validate(&e, &rs));
        assert!(!after.contains(&ValidationIssue::CODE_MYTHIC_REQUIRED_TRAIT_MISSING.to_string()));
    }

    #[test]
    fn a_substitute_flaw_satisfies_the_required_flaw() {
        let rs = mythic_ruleset();
        let mut e = mythic_entity("mythic_type.devil_child");
        // A different Major Supernatural flaw substitutes for Tragic Life.
        e.selections = vec![sel("virtue.demonic_blood"), sel("flaw.other_supernatural")];
        let codes = all_codes(&validate(&e, &rs));
        // The required-flaw slot is satisfied by the substitute (no missing warning
        // referencing tragic_life); only the still-unbought status choice, if any,
        // would surface — Demonic Blood + a Major Supernatural flaw cover the
        // package.
        assert!(!codes.contains(&ValidationIssue::CODE_MYTHIC_REQUIRED_TRAIT_MISSING.to_string()));
    }

    #[test]
    fn unresolved_mythic_choice_is_an_error() {
        let rs = mythic_ruleset();
        // Devil Child with the Might/Powers choice unset.
        let e = mythic_entity("mythic_type.devil_child");
        let codes = all_codes(&validate(&e, &rs));
        assert!(codes.contains(&ValidationIssue::CODE_MYTHIC_CHOICE_UNRESOLVED.to_string()));
    }

    #[test]
    fn unresolved_open_grant_reports_its_choice_key() {
        let rs = mythic_ruleset();
        // Open Child's open grant left unpicked → the unresolved-pick error names
        // the grant's choice_key (the Grant::Open absent-pick wiring).
        let e = mythic_entity("mythic_type.open_child");
        let issue = validate(&e, &rs)
            .issues
            .into_iter()
            .find(|i| i.code == ValidationIssue::CODE_MYTHIC_CHOICE_UNRESOLVED)
            .expect("unresolved mythic choice present");
        assert_eq!(
            issue.args.get("mythic_type").map(String::as_str),
            Some("mythic_type.open_child")
        );
        assert_eq!(
            issue.args.get("choice_key").map(String::as_str),
            Some("open_child_virtue")
        );
    }

    #[test]
    fn open_grant_pick_violating_its_constraint_errors_with_args() {
        let rs = mythic_ruleset();
        // Open Child requires a Minor Supernatural *virtue*; a flaw pick violates
        // the constraint → CODE_MYTHIC_GRANT_CONSTRAINT carrying type/key/item.
        let mut e = mythic_entity("mythic_type.open_child");
        e.mythic_choices.insert(
            "open_child_virtue".to_string(),
            Selection::new(Id::new("flaw.optimistic")),
        );
        let issue = validate(&e, &rs)
            .issues
            .into_iter()
            .find(|i| i.code == ValidationIssue::CODE_MYTHIC_GRANT_CONSTRAINT)
            .expect("mythic grant constraint violation present");
        assert_eq!(
            issue.args.get("mythic_type").map(String::as_str),
            Some("mythic_type.open_child")
        );
        assert_eq!(
            issue.args.get("choice_key").map(String::as_str),
            Some("open_child_virtue")
        );
        assert_eq!(
            issue.args.get("item").map(String::as_str),
            Some("flaw.optimistic")
        );
    }

    /// The Mythic-type Open grant validates its pick's parameters too, exactly as
    /// the House Open grant does (same machinery, same issue codes).
    #[test]
    fn an_open_mythic_pick_missing_its_param_errors() {
        let rs = mythic_ruleset();
        let mut e = mythic_entity("mythic_type.open_child");
        e.mythic_choices.insert(
            "open_child_virtue".to_string(),
            Selection::new(Id::new("virtue.demonic_mark")),
        );
        let codes = all_codes(&validate(&e, &rs));
        assert!(
            codes.contains(&ValidationIssue::CODE_MISSING_PARAM.to_string()),
            "a parameterized mythic Open pick with no param should error: {codes:?}"
        );
    }

    #[test]
    fn mythic_bonus_ignored_for_non_mythic_profile() {
        // A stray mythic_type on a plain companion (hand-edited save) must not
        // inflate its budget: effective_budget gates the type's bonuses on the
        // profile's `has_mythic_type`, so the companion budget stays 10/10.
        let rs = mythic_ruleset();
        let mut e = make_entity("companion", vec![]);
        e.mythic_type = Some(Id::new("mythic_type.devil_child"));
        let profile = rs.profile(&Id::new("companion")).unwrap();
        let b = effective_budget(&e, &rs, profile);
        assert_eq!(b.flaw_ceiling, 10);
        assert_eq!(b.virtue_ceiling, 10);
        assert_eq!(b.rate, 1);
    }

    // --- Spells -----------------------------------------------------------

    const SPELL_ITEMS: &str = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
        { "id": "virtue.skilled_parens", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "effects": [ { "type": "spell_levels", "amount": 30 },
                       { "type": "general_xp", "amount": 60 } ] }
    ]"#;
    const SPELL_ARTS: &str = r#"{ "arts": [
        { "id": "art.creo", "art_type": "technique" },
        { "id": "art.corpus", "art_type": "form" },
        { "id": "art.ignem", "art_type": "form" },
        { "id": "art.rego", "art_type": "technique" },
        { "id": "art.vim", "art_type": "form" }
    ] }"#;
    // The magus profile + Arts catalogue trigger the engine-required-role check,
    // so every engine-required Hermetic ability must be present.
    const SPELL_ABILITIES: &str = r#"{ "abilities": [
        { "id": "ability.artes_liberales", "category": "academic" },
        { "id": "ability.living_language", "category": "general", "parameter": "language" },
        { "id": "ability.magic_theory", "category": "arcane" },
        { "id": "ability.parma_magica", "category": "arcane" },
        { "id": "ability.penetration", "category": "arcane" },
        { "id": "ability.philosophiae", "category": "academic" }
    ] }"#;
    const SPELL_CATALOGUE: &str = r#"{ "spells": [
        { "id": "spell.pilum_of_fire", "technique": "art.creo", "form": "art.ignem", "level": 20 },
        { "id": "spell.ball_of_abysmal_flame", "technique": "art.creo", "form": "art.ignem", "level": 35 },
        { "id": "spell.aegis_of_the_hearth", "technique": "art.rego", "form": "art.vim", "ritual": true },
        { "id": "spell.general_ward", "technique": "art.rego", "form": "art.vim" },
        { "id": "spell.wizards_boost_form", "technique": "art.rego", "form": "art.vim",
          "parameters": [{ "key": "form", "type": "ref", "domain": "form" }] }
    ] }"#;
    // Two once-per-spell abilities and one repeatable, enough to exercise the
    // count-cap, repeatable, and unknown-id mastery rules.
    const SPELL_MASTERY_CATALOGUE: &str = r#"{ "abilities": [
        { "id": "spell_mastery_ability.penetration" },
        { "id": "spell_mastery_ability.fast_casting" },
        { "id": "spell_mastery_ability.quiet_casting", "repeatable": true }
    ] }"#;
    // Life stages so a magus can be taken past its Gauntlet; the `post_apprenticeship`
    // block is obligatory once an `is_magus` profile ships life-stage rules.
    const SPELL_LIFE_STAGES: &str = r#"{
        "apprenticeship": { "years": 15, "xp": 240, "minimum_abilities": [],
                            "recommended_abilities": [], "recommended_xp": 0 },
        "childhood": { "years": 5, "native_language_ability": "ability.living_language",
                       "native_language_xp": 75, "spread_xp": 45,
                       "spread_abilities": ["ability.living_language"] },
        "later_life": { "xp_per_year": 15 },
        "post_apprenticeship": { "lab_season_cost": 10,
                                 "max_charged_lab_seasons_per_year": 3,
                                 "points_per_year": 30 }
    }"#;
    // spell_levels 50 keeps the budget small enough to trip in tests.
    const SPELL_MAGUS_TYPE: &str = r#"[
        { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["hermetic"], "is_magus": true,
          "spell_levels": 50, "creation_phases": [] },
        { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
          "creation_phases": [] }
    ]"#;

    fn spell_rs() -> Ruleset {
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: SPELL_ITEMS,
            type_profiles: SPELL_MAGUS_TYPE,
            abilities: Some(SPELL_ABILITIES),
            arts: Some(SPELL_ARTS),
            houses: None,
            mythic_types: None,
            spells: Some(SPELL_CATALOGUE),
            spell_mastery_abilities: Some(SPELL_MASTERY_CATALOGUE),
            equipment: None,
            characteristics: None,
            life_stages: Some(SPELL_LIFE_STAGES),
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    fn spell(id: &str, level: Option<u8>) -> SpellSelection {
        SpellSelection {
            spell: Id::new(id),
            level,
            mastery: None,
            parameter: None,
            mastery_abilities: Vec::new(),
        }
    }

    /// A spell selection carrying a chosen parameter (e.g. the target `(Form)` of
    /// a meta-magic Vim spell).
    fn spell_param(id: &str, level: Option<u8>, parameter: &str) -> SpellSelection {
        SpellSelection {
            spell: Id::new(id),
            level,
            mastery: None,
            parameter: Some(parameter.to_string()),
            mastery_abilities: Vec::new(),
        }
    }

    /// A magus whose chosen spell levels stay within budget (and each within its
    /// per-spell cap, given high Arts) validates clean.
    #[test]
    fn spells_within_budget_and_cap_are_clean() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        // Cr 20 / Ig 20 / Int 3 / MT 5 → cap 51, so Pilum (20) is legal.
        e.art_scores = vec![
            ArtScore {
                art: Id::new("art.creo"),
                score: 20,
            },
            ArtScore {
                art: Id::new("art.ignem"),
                score: 20,
            },
        ];
        e.characteristics.insert(Characteristic::Int, 3);
        e.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.magic_theory"),
            score: 5,
            specialty: None,
            parameter: None,
        }];
        e.xp_pool = 1000; // cover the Art/MT costs so no not_enough_xp noise
        e.spells = vec![spell("spell.pilum_of_fire", None)];
        let codes = all_codes(&validate(&e, &rs));
        assert!(
            !codes
                .iter()
                .any(|c| c.starts_with("spell") || c.starts_with("over_spell")),
            "expected no spell issues, got {codes:?}"
        );
    }

    /// The sum of chosen spell levels exceeding the budget emits over_spell_levels.
    #[test]
    fn spells_over_budget_flag_over_spell_levels() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        // Budget 50; Pilum(20)+Ball(35)=55 > 50.
        e.spells = vec![
            spell("spell.pilum_of_fire", None),
            spell("spell.ball_of_abysmal_flame", None),
        ];
        assert!(all_codes(&validate(&e, &rs)).contains(&"over_spell_levels".to_string()));
    }

    /// The levels of spells a magus took out of its post-Gauntlet points raise the
    /// same budget `over_spell_levels` is measured against: "Divide 30 points per year
    /// between … levels of spells" (Core Rules.md:2216). Gauntleted at 25 and now 60,
    /// this maga banked 370 of its 1050 points as spell levels, so its budget is the
    /// profile's 50 plus 370 — and the finding turns on exactly one level.
    #[test]
    fn post_gauntlet_spell_levels_raise_the_budget_the_finding_measures() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.age = Some(60);
        e.life_stages = Some(crate::life_stage::LifeStagePlan {
            gauntlet_age: Some(25),
            post_gauntlet_spell_levels: 370,
            ..crate::life_stage::LifeStagePlan::default()
        });
        // 210 + 210 = 420, exactly the budget.
        e.spells = vec![
            spell("spell.general_ward", Some(210)),
            spell("spell.aegis_of_the_hearth", Some(210)),
        ];
        assert!(!all_codes(&validate(&e, &rs)).contains(&"over_spell_levels".to_string()));

        // One level more and it fires.
        e.spells[1].level = Some(211);
        assert!(all_codes(&validate(&e, &rs)).contains(&"over_spell_levels".to_string()));
    }

    /// Skilled Parens's +30 raises the budget from 50 to 80, making 55 legal.
    #[test]
    fn skilled_parens_raises_the_spell_budget() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![sel("virtue.skilled_parens")]);
        e.spells = vec![
            spell("spell.pilum_of_fire", None),
            spell("spell.ball_of_abysmal_flame", None),
        ];
        assert!(!all_codes(&validate(&e, &rs)).contains(&"over_spell_levels".to_string()));
    }

    /// A spell above Tech+Form+Int+MagicTheory+3 emits spell_level_exceeds_cap.
    #[test]
    fn spell_above_per_spell_cap_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        // Zero Arts/Int/MT → cap = 3; Pilum (20) exceeds it.
        e.spells = vec![spell("spell.pilum_of_fire", None)];
        assert!(all_codes(&validate(&e, &rs)).contains(&"spell_level_exceeds_cap".to_string()));
    }

    /// A General spell with no chosen level warns and is excluded from the budget.
    #[test]
    fn general_spell_without_level_warns_and_is_free() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![spell("spell.aegis_of_the_hearth", None)];
        let codes = all_codes(&validate(&e, &rs));
        assert!(codes.contains(&"spell_level_unresolved".to_string()));
        assert!(!codes.contains(&"over_spell_levels".to_string()));
    }

    /// A parameterized spell taken with a chosen Form validates, and the same base
    /// spell with a *different* Form is a distinct instance (not a duplicate).
    /// Source: Core Rules.md:15791-15794 (one version per Hermetic Form).
    #[test]
    fn parametrized_spell_distinct_per_form() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![
            spell_param("spell.wizards_boost_form", Some(10), "art.ignem"),
            spell_param("spell.wizards_boost_form", Some(10), "art.corpus"),
        ];
        let codes = all_codes(&validate(&e, &rs));
        assert!(!codes.contains(&"duplicate_spell".to_string()), "{codes:?}");
        assert!(!codes.contains(&"missing_param".to_string()), "{codes:?}");
        assert!(
            !codes.contains(&"unknown_param_value".to_string()),
            "{codes:?}"
        );
    }

    /// The same parameterized spell with the SAME Form twice is a duplicate.
    #[test]
    fn parametrized_spell_same_form_is_duplicate() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![
            spell_param("spell.wizards_boost_form", Some(10), "art.ignem"),
            spell_param("spell.wizards_boost_form", Some(10), "art.ignem"),
        ];
        assert!(all_codes(&validate(&e, &rs)).contains(&"duplicate_spell".to_string()));
    }

    /// A parameterized spell taken with no chosen parameter is flagged missing_param.
    #[test]
    fn parametrized_spell_without_parameter_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![spell("spell.wizards_boost_form", Some(10))];
        assert!(all_codes(&validate(&e, &rs)).contains(&"missing_param".to_string()));
    }

    /// A parameterized spell whose chosen value is outside the parameter's domain
    /// (a Technique where a Form is required) is flagged unknown_param_value.
    #[test]
    fn parametrized_spell_wrong_domain_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![spell_param(
            "spell.wizards_boost_form",
            Some(10),
            "art.creo",
        )];
        assert!(all_codes(&validate(&e, &rs)).contains(&"unknown_param_value".to_string()));
    }

    /// The same spell at the same level twice emits duplicate_spell.
    #[test]
    fn duplicate_spell_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![
            spell("spell.pilum_of_fire", None),
            spell("spell.pilum_of_fire", None),
        ];
        assert!(all_codes(&validate(&e, &rs)).contains(&"duplicate_spell".to_string()));
    }

    /// Spells on a non-magus are ref-checked but never budget/cap-checked.
    #[test]
    fn spells_on_non_magus_are_not_budgeted() {
        let rs = spell_rs();
        let mut e = make_entity("companion", vec![]);
        e.spells = vec![
            spell("spell.pilum_of_fire", None),
            spell("spell.ball_of_abysmal_flame", None),
        ];
        let codes = all_codes(&validate(&e, &rs));
        assert!(!codes.contains(&"over_spell_levels".to_string()));
        assert!(!codes.contains(&"spell_level_exceeds_cap".to_string()));
    }

    /// A mastered spell carrying one special ability per effective mastery level,
    /// all catalogue-known and non-repeating, raises no mastery-ability issue.
    /// Source: Core Rules.md:9524-9526.
    #[test]
    fn mastery_abilities_within_score_are_clean() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.xp_pool = 1000;
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: Some(2),
            parameter: None,
            mastery_abilities: vec![
                Id::new("spell_mastery_ability.penetration"),
                Id::new("spell_mastery_ability.fast_casting"),
            ],
        }];
        let codes = all_codes(&validate(&e, &rs));
        assert!(
            !codes.iter().any(|c| c.contains("mastery_abilit")),
            "expected no mastery-ability issues, got {codes:?}"
        );
    }

    /// More special abilities than the spell's effective mastery score is an error
    /// — one may be chosen per mastery level (Core Rules.md:9524-9526).
    #[test]
    fn too_many_mastery_abilities_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.xp_pool = 1000;
        // Mastery 1 permits one ability; two are chosen.
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: Some(1),
            parameter: None,
            mastery_abilities: vec![
                Id::new("spell_mastery_ability.penetration"),
                Id::new("spell_mastery_ability.fast_casting"),
            ],
        }];
        assert!(all_codes(&validate(&e, &rs)).contains(&"too_many_mastery_abilities".to_string()));
    }

    /// A non-repeatable ability chosen twice for the same spell is an error, even
    /// when the count fits the mastery score (Core Rules.md:9528-9592).
    #[test]
    fn duplicate_non_repeatable_mastery_ability_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.xp_pool = 1000;
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: Some(2),
            parameter: None,
            mastery_abilities: vec![
                Id::new("spell_mastery_ability.penetration"),
                Id::new("spell_mastery_ability.penetration"),
            ],
        }];
        assert!(all_codes(&validate(&e, &rs)).contains(&"duplicate_mastery_ability".to_string()));
    }

    /// A repeatable ability (Quiet Casting) chosen twice for the same spell is
    /// legal (Core Rules.md:9580).
    #[test]
    fn repeatable_mastery_ability_twice_is_clean() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.xp_pool = 1000;
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: Some(2),
            parameter: None,
            mastery_abilities: vec![
                Id::new("spell_mastery_ability.quiet_casting"),
                Id::new("spell_mastery_ability.quiet_casting"),
            ],
        }];
        assert!(!all_codes(&validate(&e, &rs)).contains(&"duplicate_mastery_ability".to_string()));
    }

    /// A chosen mastery-ability id absent from the catalogue fails loudly.
    #[test]
    fn unknown_mastery_ability_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.xp_pool = 1000;
        e.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: Some(1),
            parameter: None,
            mastery_abilities: vec![Id::new("spell_mastery_ability.does_not_exist")],
        }];
        assert!(all_codes(&validate(&e, &rs)).contains(&"unknown_mastery_ability".to_string()));
    }

    /// A General ritual spell learned below level 20 is a ritual-legality error
    /// (Core Rules.md:12279-12295).
    #[test]
    fn ritual_spell_learned_below_20_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![spell("spell.aegis_of_the_hearth", Some(15))];
        assert!(all_codes(&validate(&e, &rs)).contains(&"spell_ritual_legality".to_string()));
    }

    /// A General non-ritual spell learned above level 50 is a ritual-legality error
    /// (Core Rules.md:12283).
    #[test]
    fn non_ritual_spell_learned_above_50_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![spell("spell.general_ward", Some(55))];
        assert!(all_codes(&validate(&e, &rs)).contains(&"spell_ritual_legality".to_string()));
    }

    /// A General ritual spell learned at exactly level 20 is legal.
    #[test]
    fn ritual_spell_learned_at_20_is_clean() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![spell("spell.aegis_of_the_hearth", Some(20))];
        assert!(!all_codes(&validate(&e, &rs)).contains(&"spell_ritual_legality".to_string()));
    }

    /// An unknown spell id is an error.
    #[test]
    fn unknown_spell_is_flagged() {
        let rs = spell_rs();
        let mut e = make_entity("magus", vec![]);
        e.spells = vec![spell("spell.does_not_exist", None)];
        assert!(all_codes(&validate(&e, &rs)).contains(&"unknown_spell".to_string()));
    }

    // --- Phase 7: age cap, Supernatural gate, Personality, Reputations -------

    const P7_ITEMS: &str = r#"[
        { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.second_sight", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "supernatural", "entity_kinds": ["character"],
          "effects": [{ "type": "ability_score_grant", "ability": "ability.second_sight", "amount": 1 }] },
        { "id": "virtue.affinity_awareness", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
          "effects": [{ "type": "affinity_ability_cost", "param": "ability", "counts_as_num": 3, "counts_as_den": 2 }] },
        { "id": "virtue.self_confident", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "effects": [{ "type": "confidence_bonus", "score": 1, "points": 2 }] },
        { "id": "flaw.infamous", "kind": "flaw", "classification": "narrative", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "effects": [{ "type": "grants_reputation", "kind": "local", "score": 4 }] },
        { "id": "flaw.major_personality", "kind": "flaw", "classification": "narrative", "magnitude": "major",
          "category": "personality", "entity_kinds": ["character"] }
    ]"#;
    // Carries the age → max-Ability-score bands (Core:2366-2374), so the age-cap
    // checks below are exercised against ruleset data.
    const P7_ABILITIES: &str = r#"{ "advancement": [
        { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
        { "score": 7, "total_xp": 140 }, { "score": 8, "total_xp": 180 } ],
        "age_ability_caps": [
          { "max_age": 29, "max_score": 5 },
          { "max_age": 35, "max_score": 6 },
          { "max_age": 40, "max_score": 7 },
          { "max_age": 45, "max_score": 8 },
          { "max_score": 9 } ],
        "abilities": [
        { "id": "ability.awareness", "category": "general" },
        { "id": "ability.second_sight", "category": "supernatural", "requires_training": true },
        { "id": "ability.animal_ken", "category": "supernatural", "requires_training": true },
        { "id": "ability.artes_liberales", "category": "academic" },
        { "id": "ability.magic_theory", "category": "arcane" },
        { "id": "ability.parma_magica", "category": "arcane" },
        { "id": "ability.penetration", "category": "arcane" },
        { "id": "ability.philosophiae", "category": "academic" } ] }"#;
    const P7_TYPES: &str = r#"[
        { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
          "gift_policy": "allowed", "gift_id": "virtue.the_gift",
          "confidence_score": 1, "confidence_points": 3, "creation_phases": [] },
        { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
          "is_magus": true, "gift_policy": "required", "gift_id": "virtue.the_gift",
          "confidence_score": 1, "confidence_points": 3, "creation_phases": [] },
        { "id": "grog", "budget": { "virtue_points": 3, "flaw_points": 3 },
          "creation_phases": [] }
    ]"#;

    const P7_CHARACTERISTICS: &str = r#"{
        "start_points": 7,
        "costs": [ { "score": 0, "cost": 0 } ]
      }"#;

    fn p7_rs() -> Ruleset {
        Ruleset::from_sources(crate::ruleset::RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: P7_ITEMS,
            type_profiles: P7_TYPES,
            abilities: Some(P7_ABILITIES),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: Some(P7_CHARACTERISTICS),
            life_stages: None,
            childhoods: None,
            aging: None,
        })
        .unwrap()
    }

    fn ability(id: &str, score: u8) -> AbilityScore {
        AbilityScore {
            ability: Id::new(id),
            score,
            specialty: None,
            parameter: None,
        }
    }

    #[test]
    fn ability_over_age_cap_is_flagged() {
        let rs = p7_rs();
        let mut e = make_entity("companion", vec![]);
        e.age = Some(25); // cap 5
        e.ability_scores = vec![ability("ability.awareness", 6)];
        assert!(all_codes(&validate(&e, &rs)).contains(&"ability_above_age_cap".to_string()));
    }

    #[test]
    fn ability_at_age_cap_is_clean() {
        let rs = p7_rs();
        let mut e = make_entity("companion", vec![]);
        e.age = Some(40); // cap 7
        e.ability_scores = vec![ability("ability.awareness", 7)];
        assert!(!all_codes(&validate(&e, &rs)).contains(&"ability_above_age_cap".to_string()));
    }

    #[test]
    fn affinity_raises_the_age_cap_by_two_but_no_more() {
        let rs = p7_rs();
        let affinity = Selection::with_params(
            Id::new("virtue.affinity_awareness"),
            BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
        );
        // Age 25 → base cap 5; Affinity lifts it to 7. Score 7 is legal, 8 is not.
        let mut ok = make_entity("companion", vec![affinity.clone()]);
        ok.age = Some(25);
        ok.ability_scores = vec![ability("ability.awareness", 7)];
        assert!(!all_codes(&validate(&ok, &rs)).contains(&"ability_above_age_cap".to_string()));

        let mut over = make_entity("companion", vec![affinity]);
        over.age = Some(25);
        over.ability_scores = vec![ability("ability.awareness", 8)];
        assert!(all_codes(&validate(&over, &rs)).contains(&"ability_above_age_cap".to_string()));
    }

    #[test]
    fn supernatural_ability_without_gift_or_virtue_is_flagged() {
        let rs = p7_rs();
        let mut e = make_entity("companion", vec![]);
        e.ability_scores = vec![ability("ability.animal_ken", 2)];
        assert!(
            all_codes(&validate(&e, &rs))
                .contains(&"supernatural_ability_requires_virtue".to_string())
        );
    }

    #[test]
    fn gifted_companion_gets_one_free_supernatural_ability() {
        let rs = p7_rs();
        let mut e = make_entity("companion", vec![sel("virtue.the_gift")]);
        e.ability_scores = vec![ability("ability.animal_ken", 2)];
        // One uncovered supernatural ability fits the single free Gift slot.
        assert!(
            !all_codes(&validate(&e, &rs))
                .contains(&"supernatural_ability_requires_virtue".to_string())
        );
        // A second uncovered one exceeds the allowance.
        e.ability_scores.push(ability("ability.second_sight", 2));
        assert!(
            all_codes(&validate(&e, &rs))
                .contains(&"supernatural_ability_requires_virtue".to_string())
        );
    }

    #[test]
    fn magus_gets_no_free_supernatural_slot() {
        let rs = p7_rs();
        // A magus must select The Gift (required) but still gets 0 free slots.
        let mut e = make_entity("magus", vec![sel("virtue.the_gift")]);
        e.ability_scores = vec![ability("ability.animal_ken", 2)];
        assert!(
            all_codes(&validate(&e, &rs))
                .contains(&"supernatural_ability_requires_virtue".to_string())
        );
    }

    #[test]
    fn granting_virtue_covers_its_supernatural_ability() {
        let rs = p7_rs();
        // Second Sight (virtue) grants the ability; holding it needs no free slot.
        let mut e = make_entity("companion", vec![sel("virtue.second_sight")]);
        e.ability_scores = vec![ability("ability.second_sight", 2)];
        assert!(
            !all_codes(&validate(&e, &rs))
                .contains(&"supernatural_ability_requires_virtue".to_string())
        );
    }

    #[test]
    fn personality_trait_over_three_needs_a_major_personality_flaw() {
        let rs = p7_rs();
        let mut e = make_entity("companion", vec![]);
        e.personality_traits = vec![PersonalityTrait {
            name: "Brave".into(),
            value: 4,
        }];
        assert!(
            all_codes(&validate(&e, &rs)).contains(&"personality_trait_out_of_range".to_string())
        );
        // A Major Personality Flaw lifts one trait to ±6.
        e.selections = vec![sel("flaw.major_personality")];
        assert!(
            !all_codes(&validate(&e, &rs)).contains(&"personality_trait_out_of_range".to_string())
        );
    }

    #[test]
    fn personality_trait_beyond_six_is_always_illegal() {
        let rs = p7_rs();
        let mut e = make_entity("companion", vec![sel("flaw.major_personality")]);
        e.personality_traits = vec![PersonalityTrait {
            name: "Wrathful".into(),
            value: 7,
        }];
        assert!(
            all_codes(&validate(&e, &rs)).contains(&"personality_trait_out_of_range".to_string())
        );
    }

    #[test]
    fn reputation_without_a_grant_is_flagged() {
        let rs = p7_rs();
        let mut e = make_entity("companion", vec![]);
        e.reputations = vec![Reputation {
            kind: ReputationType::Local,
            score: 4,
            content: "dragon slayer".into(),
        }];
        assert!(all_codes(&validate(&e, &rs)).contains(&"reputation_not_granted".to_string()));
        // Infamous grants a Local reputation, covering it.
        e.selections = vec![sel("flaw.infamous")];
        assert!(!all_codes(&validate(&e, &rs)).contains(&"reputation_not_granted".to_string()));
    }
}
