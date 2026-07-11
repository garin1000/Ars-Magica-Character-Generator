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
    CategoryCap, Effect, Entity, EntityKind, EntityTypeProfile, GiftPolicy, Id, ItemKind,
    Magnitude, ParameterDomain, PointItem, Prereq, ValidationMode,
};

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
/// Every `code` the engine can emit, with the `args` keys it carries. The UI
/// must define an `issue-<code>` message for each (see the workspace test
/// `every_validation_code_has_a_fluent_key_in_each_locale`).
///
/// | `code` | severity | `args` keys |
/// |--------|----------|-------------|
/// | `unknown_type` | error | `type_id` |
/// | `unknown_ref` | error | `item` |
/// | `wrong_entity_kind` | error | `item`, `entity_kind` |
/// | `duplicate_selection` | error | `item`, `count`, `max` |
/// | `over_budget_virtues` | error | `points`, `budget` |
/// | `over_budget_flaws` | error | `points`, `budget` |
/// | `unbalanced_virtues` | error | `virtue_points`, `flaw_points` |
/// | `too_many_major_virtues` | error | `count`, `max` |
/// | `too_many_major_flaws` | error | `count`, `max` |
/// | `too_many_minor_flaws` | error | `count`, `max` |
/// | `too_many_major_<category>_flaws`† | error or warning | `count`, `max` |
/// | `too_many_<category>_flaws`† | error or warning | `count`, `max` |
/// | `too_many_major_<category>_virtues`† | error or warning | `count`, `max` |
/// | `too_many_<category>_virtues`† | error or warning | `count`, `max` |
/// | `prereq_not_met` | error | `item` |
/// | `prereq_unevaluated` | warning | `item` |
/// | `incompatible` | error | `item`, `other` |
/// | `category_not_permitted` | error | `item`, `category` |
/// | `forbidden_category` | error | `item`, `category` |
/// | `missing_required_trait` | error | `item` |
/// | `forbidden_trait` | error | `item` |
/// | `missing_param` | error | `item`, `key` |
/// | `unexpected_param` | error | `item`, `key` |
/// | `unknown_param_value` | error | `item`, `key`, `value`, `domain` |
/// | `gift_required` | error | (none) |
/// | `gift_forbidden` | error | (none) |
/// | `characteristic_out_of_range` | error | `characteristic`, `score`, `min`, `max` |
/// | `characteristic_above_cap` | error | `characteristic`, `score`, `cap` |
/// | `characteristic_below_floor` | error | `characteristic`, `score`, `floor` |
/// | `characteristic_max_base_too_low` | error | `item`, `characteristic`, `base`, `min` |
/// | `characteristic_min_base_too_high` | error | `item`, `characteristic`, `base`, `max` |
/// | `characteristic_overspent` | error | `cost`, `points` |
/// | `characteristic_points_unspent` | warning | `cost`, `points` |
/// | `unknown_ability` | error | `ability` |
/// | `duplicate_ability` | error | `ability`, `count` |
/// | `not_enough_xp` | error | `spent`, `pool`, `shortfall` |
/// | `restricted_xp_unspent` | warning | `amount`, `used`, `unspent` |
/// | `ability_parameter_required` | error | `ability` |
/// | `ability_score_out_of_range` | error | `ability`, `score`, `max` |
/// | `ability_bonus_dangling_target` | error | `item`, `ability`, `parameter` |
/// | `unknown_art` | error | `art` |
/// | `duplicate_art` | error | `art`, `count` |
/// | `art_score_out_of_range` | error | `art`, `score`, `max` |
/// | `house_choice_unresolved` | error | `house`, `choice_key` |
/// | `house_grant_constraint` | error | `house`, `choice_key`, `item` |
/// | `house_unset` | warning | (none) |
/// | `missing_hermetic_flaw` | warning | (none) |
/// | `mythic_type_unset` | warning | (none) |
/// | `mythic_choice_unresolved` | error | `mythic_type`, `choice_key` |
/// | `mythic_grant_constraint` | error | `mythic_type`, `choice_key`, `item` |
/// | `mythic_required_trait_missing` | warning | `item` |
/// | `unknown_spell` | error | `spell` |
/// | `duplicate_spell` | error | `spell`, `count` |
/// | `spell_level_unresolved` | warning | `spell` |
/// | `over_spell_levels` | error | `used`, `budget`, `over` |
/// | `spell_level_exceeds_cap` | error | `spell`, `level`, `cap` |
/// | `ability_above_age_cap` | error | `ability`, `score`, `cap`, `age` |
/// | `supernatural_ability_requires_virtue` | error | `ability` |
/// | `personality_trait_out_of_range` | error | `name`, `value`, `max` |
/// | `reputation_not_granted` | error | `kind`, `content` |
///
/// † The per-category caps emit a code derived from the `flaw_category_caps` /
/// `virtue_category_caps` entry's category slug: `too_many_<category>_flaws` /
/// `too_many_<category>_virtues` (or the `too_many_major_<category>_…` form when
/// the cap is `major_only`); severity follows the cap's `hard` flag. The shipped
/// `personality`/`story` flaw caps thus produce `too_many_major_personality_flaws`
/// (error), `too_many_personality_flaws` (warning), and `too_many_story_flaws`
/// (warning), and the magus `hermetic` virtue cap produces
/// `too_many_major_hermetic_virtues` (error); a new category requires its
/// matching `issue-<code>` Fluent key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Error or warning.
    pub severity: IssueSeverity,
    /// Stable machine key for this issue. The UI derives the Fluent message id
    /// as `issue-<code>` (e.g. code `over_budget_virtues` →
    /// `issue-over_budget_virtues`).
    pub code: String,
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
    /// See [`ValidationIssue::CODE_UNKNOWN_TYPE`]. Warning: a restricted XP grant
    /// (Educated/Warrior/Privileged) has experience the character left unspent on
    /// its eligible Abilities; the rules waste it.
    pub const CODE_RESTRICTED_XP_UNSPENT: &'static str = "restricted_xp_unspent";
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

    /// Builds an issue with the given severity, code, args, and context.
    pub fn new(
        severity: IssueSeverity,
        code: &str,
        args: BTreeMap<String, String>,
        context: Option<Id>,
    ) -> Self {
        Self {
            severity,
            code: code.to_string(),
            args,
            context,
        }
    }

    /// Builds an error-severity issue.
    pub fn error(code: &str, args: BTreeMap<String, String>, context: Option<Id>) -> Self {
        Self::new(IssueSeverity::Error, code, args, context)
    }

    /// Builds a warning-severity issue.
    pub fn warning(code: &str, args: BTreeMap<String, String>, context: Option<Id>) -> Self {
        Self::new(IssueSeverity::Warning, code, args, context)
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
        validate_xp_pool(entity, ruleset, &mut issues);
    }

    ValidationResult { issues }
}

/// Emits `unknown_type` when the entity's `type_id` has no matching profile.
fn validate_known_type(
    entity: &Entity,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    if type_profile.is_none() {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_UNKNOWN_TYPE,
            args([("type_id", entity.type_id.to_string())]),
            None,
        ));
    }
}

/// Emits `unknown_ref` for each selection whose item id is not in the ruleset.
fn validate_known_refs(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    for selection in &entity.selections {
        if !ruleset.point_items.contains_key(&selection.item_ref) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_REF,
                args([("item", selection.item_ref.to_string())]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

/// Enforces the two halves of the points rule:
///
/// 1. Flaw points stay within the type's budget (and virtue points within
///    theirs as a clear-message backstop).
/// 2. Virtues must be funded by Flaws: spent virtue points may not exceed the
///    flaw points granted. A character with 10 virtue points and 0 flaw points
///    is over budget on neither total yet is illegal — Players "start with no
///    points for buying Virtues and Flaws, and thus must take Flaws if they
///    want Virtues."
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2774 ("must take
/// Flaws if they want Virtues"), :2297 (companions), :2303 (magi) — "up to ten
/// points of Flaws, and the same number of points of Virtues". The per-type
/// point totals themselves are data in `rules/core/character_types.json` (see
/// RULES.md).
fn validate_balance(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let Balance {
        virtue_points,
        flaw_points,
    } = compute_balance(entity, ruleset);

    let budget = effective_budget(entity, ruleset, profile);

    if virtue_points > budget.virtue_ceiling {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_BUDGET_VIRTUES,
            args([
                ("points", virtue_points.to_string()),
                ("budget", budget.virtue_ceiling.to_string()),
            ]),
            None,
        ));
    }

    if flaw_points > budget.flaw_ceiling {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_BUDGET_FLAWS,
            args([
                ("points", flaw_points.to_string()),
                ("budget", budget.flaw_ceiling.to_string()),
            ]),
            None,
        ));
    }

    if virtue_points > budget.funded(flaw_points) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_UNBALANCED_VIRTUES,
            args([
                ("virtue_points", virtue_points.to_string()),
                ("flaw_points", flaw_points.to_string()),
            ]),
            None,
        ));
    }
}

/// The virtue/flaw point ceilings the balance check enforces, folding in the
/// selected Mythic Companion type's per-type bonus points on top of the
/// profile's base budget. For any non-mythic type (no `mythic_type`, or a type
/// carrying no bonuses) both bonuses are 0 and this reduces **exactly** to the
/// profile's own budget — `flaw_ceiling = flaw_points`,
/// `virtue_ceiling = virtue_points`, `funded = flaw · rate`.
///
/// The extra Flaw points each still fund virtue points at the type's rate, so
/// they raise the virtue ceiling by `bonus_flaw · rate` (not just the flaw
/// ceiling); `bonus_free_virtue_points` is unfunded headroom that also lifts the
/// funded floor. Source: Core Rules.md:2664 (Devil Child +3 free V / +7 F);
/// Realms of Power - Magic.md:5486 (Spirit Votary +7 F).
struct EffectiveBudget {
    virtue_ceiling: i32,
    flaw_ceiling: i32,
    bonus_free_virtue_points: i32,
    rate: i32,
}

impl EffectiveBudget {
    /// Virtue points fundable by `flaw_points` taken, plus the free headroom.
    fn funded(&self, flaw_points: i32) -> i32 {
        flaw_points * self.rate + self.bonus_free_virtue_points
    }
}

fn effective_budget(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: &EntityTypeProfile,
) -> EffectiveBudget {
    let rate = profile.budget.virtue_points_per_flaw_point as i32;
    // Only a mythic-capable profile applies a type's bonus points — a stray
    // `mythic_type` on some other profile (hand-edited save) must not inflate its
    // budget (validate conditionally, mirroring `validate_mythic_type`'s gate).
    let (bonus_flaw, bonus_free_virtue) = profile
        .has_mythic_type
        .then_some(entity.mythic_type.as_ref())
        .flatten()
        .and_then(|id| ruleset.mythic_type(id))
        .map(|t| {
            (
                t.bonus_flaw_points as i32,
                t.bonus_free_virtue_points as i32,
            )
        })
        .unwrap_or((0, 0));
    EffectiveBudget {
        virtue_ceiling: profile.budget.virtue_points as i32 + bonus_flaw * rate + bonus_free_virtue,
        flaw_ceiling: profile.budget.flaw_points as i32 + bonus_flaw,
        bonus_free_virtue_points: bonus_free_virtue,
        rate,
    }
}

/// The effective virtue/flaw point ceilings `(virtue, flaw)` for the entity's
/// type — the profile's base budget plus any Mythic Companion type bonus — for
/// the frontend's balance display (so the bar shows a Devil Child's 37/17, not
/// the base 20/10). `None` when the type profile is unknown. Keeps the budget
/// numbers engine-authoritative rather than recomputed in TS.
pub fn effective_point_ceilings(entity: &Entity, ruleset: &Ruleset) -> Option<(u32, u32)> {
    let profile = ruleset.profile(&entity.type_id)?;
    let b = effective_budget(entity, ruleset, profile);
    Some((b.virtue_ceiling.max(0) as u32, b.flaw_ceiling.max(0) as u32))
}

/// The accumulated virtue and flaw point totals for an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    /// Total points spent on positive items (virtues, boons).
    pub virtue_points: i32,
    /// Total points granted by negative items (flaws, hooks).
    pub flaw_points: i32,
}

/// Computes the total virtue and flaw points for an entity.
/// Unknown item refs are skipped.
pub fn compute_balance(entity: &Entity, ruleset: &Ruleset) -> Balance {
    let mut virtue_points: i32 = 0;
    let mut flaw_points: i32 = 0;

    for selection in &entity.selections {
        if let Some(item) = ruleset.point_items.get(&selection.item_ref) {
            let pts = item.magnitude.points() as i32;
            if item.kind.is_positive() {
                virtue_points += pts;
            } else {
                flaw_points += pts;
            }
        }
    }

    Balance {
        virtue_points,
        flaw_points,
    }
}

/// Enforces per-type caps on the *count* of items (distinct from the point
/// budget). The caps themselves are data in the type profile; whether a cap is
/// a hard rule (error) or a soft guideline (warning) is fixed by the rulebook
/// and encoded here per cap. A cap left `None`/absent imposes no limit.
///
/// Per-category flaw caps (Personality, Story, ...) are data in the profile's
/// `flaw_category_caps`: each entry names its category, so no category slug is
/// hardcoded in the engine.
///
/// Source: grogs may take no Major Virtues or Flaws (the `max_major_*` count
/// caps) at Ars Magica - Definitive Edition (Core Rules).md:2824-2830; ≤5 Minor
/// Flaws (central) at :2774, grogs ≤3 at :1009; ≤1 Major Personality Flaw at
/// :2820; ≤2 Personality Flaws (soft) at :2820/:2976; ≤1 Story Flaw (soft) at
/// :2818, grogs none at :1009. See RULES.md.
fn validate_caps(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    // Counts selections whose resolved point item matches `pred`.
    let count = |pred: &dyn Fn(&PointItem) -> bool| -> usize {
        entity
            .selections
            .iter()
            .filter(|s| ruleset.point_items.get(&s.item_ref).is_some_and(pred))
            .count()
    };

    let count_args = |n: usize, max: u8| args([("count", n.to_string()), ("max", max.to_string())]);

    // --- Hard caps ("may not ...") → blocking errors ---

    if let Some(max) = profile.budget.max_major_virtues {
        let n = count(&|i| i.kind == ItemKind::Virtue && i.magnitude == Magnitude::Major);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_TOO_MANY_MAJOR_VIRTUES,
                count_args(n, max),
                None,
            ));
        }
    }

    if let Some(max) = profile.budget.max_major_flaws {
        let n = count(&|i| i.kind == ItemKind::Flaw && i.magnitude == Magnitude::Major);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_TOO_MANY_MAJOR_FLAWS,
                count_args(n, max),
                None,
            ));
        }
    }

    if let Some(max) = profile.budget.max_minor_flaws {
        let n = count(&|i| i.kind == ItemKind::Flaw && i.magnitude == Magnitude::Minor);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_TOO_MANY_MINOR_FLAWS,
                count_args(n, max),
                None,
            ));
        }
    }

    // --- Data-driven per-category caps ---
    //
    // Each cap names its category as data, so the engine never hardcodes a
    // category slug. A `hard` cap is a blocking error; otherwise a non-blocking
    // warning (the book marks the Personality/Story guidelines as
    // troupe-overridable). The issue code is derived from the category slug as
    // `too_many_<category>_<noun>` (or `too_many_major_<category>_<noun>` when
    // the cap is Major-only), so the Fluent key follows the category by
    // convention — no slug is baked into the engine. Counts `entity.selections`
    // only, so House-granted items (which never enter the bought list) are
    // exempt — Bjornaer's Major Hermetic Heartbeast cannot trip a virtue cap.
    //
    // Source: Ars Magica - Definitive Edition (Core Rules).md:2855-2861.
    let mut push_category_cap_issues = |caps: &[CategoryCap], kind: ItemKind, noun: &str| {
        for cap in caps {
            let n = count(&|i| {
                i.kind == kind
                    && i.category == cap.category
                    && (!cap.major_only || i.magnitude == Magnitude::Major)
            });
            if n <= cap.max as usize {
                continue;
            }

            let code = if cap.major_only {
                format!("too_many_major_{}_{}", cap.category, noun)
            } else {
                format!("too_many_{}_{}", cap.category, noun)
            };
            let cap_args = count_args(n, cap.max);

            if cap.hard {
                issues.push(ValidationIssue::error(&code, cap_args, None));
            } else {
                issues.push(ValidationIssue::warning(&code, cap_args, None));
            }
        }
    };

    push_category_cap_issues(&profile.budget.flaw_category_caps, ItemKind::Flaw, "flaws");
    push_category_cap_issues(
        &profile.budget.virtue_category_caps,
        ItemKind::Virtue,
        "virtues",
    );
}

/// Warns when more than half the Virtue points a character has taken are Tainted
/// (and likewise for Flaws). The rulebook frames this as a "should" ("no more
/// than half a character's Virtues should be tainted, and similarly for Flaws"),
/// so it is a non-blocking warning; the limit is measured against the points
/// actually taken, not the type's budget. Free items contribute 0 points and so
/// never affect the ratio.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2998-3002.
fn validate_tainted_cap(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let (mut tainted_virtue, mut total_virtue) = (0i32, 0i32);
    let (mut tainted_flaw, mut total_flaw) = (0i32, 0i32);
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        let pts = item.magnitude.points() as i32;
        if item.kind.is_positive() {
            total_virtue += pts;
            if item.tainted {
                tainted_virtue += pts;
            }
        } else {
            total_flaw += pts;
            if item.tainted {
                tainted_flaw += pts;
            }
        }
    }

    let mut warn = |tainted: i32, total: i32, code: &str| {
        // "No more than half": tainted may equal half but not exceed it. The
        // integer form `2·tainted > total` sidesteps any rounding choice.
        if tainted * 2 > total {
            issues.push(ValidationIssue::warning(
                code,
                args([
                    ("tainted", tainted.to_string()),
                    ("total", total.to_string()),
                ]),
                None,
            ));
        }
    };
    warn(tainted_virtue, total_virtue, "too_many_tainted_virtues");
    warn(tainted_flaw, total_flaw, "too_many_tainted_flaws");
}

/// Validates a magus's Hermetic House and its specialisation picks. Runs only
/// for a magus type (`is_magus`); no other type has a House.
///
/// - A magus with no House gets a soft `house_unset` warning — belonging to a
///   House is a "should" the troupe can waive, not a hard rule.
/// - Each `Choice` grant's pick (keyed by `choice_key`) must be present and one
///   of the offered options, else `house_choice_unresolved`.
/// - Each `Open` grant's pick must be present (else `house_choice_unresolved`)
///   and satisfy the grant's declarative `GrantConstraint` (kind, magnitude,
///   category allow/deny lists), else `house_grant_constraint`.
/// - A magus with no Flaw in a Hermetic category gets a soft
///   `missing_hermetic_flaw` warning. "Hermetic" is the type's `gift_categories`
///   (data), so no category slug is hardcoded.
///
/// The picks are validated here rather than as ordinary selections because the
/// derived grant is never stored on `entity.selections`; the grant option refs
/// are integrity-checked at load (see `validate_house_refs`).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2855-2861 (the free
/// House Virtue and the recommendation to take a Hermetic Flaw).
fn validate_house(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    // Houses are a magus-only concern; a non-magus (or an unknown type) has none.
    let Some(profile) = type_profile else {
        return;
    };
    if !profile.is_magus {
        return;
    }

    // A magus should take at least one Hermetic Flaw. "Hermetic" is the type's
    // declared gift category (data), so no slug is hardcoded here; skip the
    // guideline entirely when the type names no gift category.
    if !profile.gift_categories.is_empty() {
        let has_hermetic_flaw = entity.selections.iter().any(|s| {
            ruleset.point_items.get(&s.item_ref).is_some_and(|item| {
                item.kind == ItemKind::Flaw && profile.gift_categories.contains(&item.category)
            })
        });
        if !has_hermetic_flaw {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_MISSING_HERMETIC_FLAW,
                args([]),
                None,
            ));
        }
    }

    // No House: a soft warning, and there are no grants to resolve.
    let Some(house_id) = &entity.house else {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_HOUSE_UNSET,
            args([]),
            None,
        ));
        return;
    };

    // An unknown House id resolves to no grants; nothing further to check.
    let Some(house) = ruleset.house(house_id) else {
        return;
    };

    let unresolved = |choice_key: &str| {
        ValidationIssue::error(
            ValidationIssue::CODE_HOUSE_CHOICE_UNRESOLVED,
            args([
                ("house", house_id.to_string()),
                ("choice_key", choice_key.to_string()),
            ]),
            None,
        )
    };

    for grant in &house.grants {
        match grant {
            // A fixed grant carries no player choice, so nothing to validate.
            Grant::Fixed { .. } => {}
            Grant::Choice {
                choice_key,
                options,
            } => {
                let pick = entity.house_choices.get(choice_key);
                if !pick.is_some_and(|p| options.contains(p)) {
                    issues.push(unresolved(choice_key));
                }
            }
            Grant::Open {
                choice_key,
                constraint,
            } => {
                let Some(pick) = entity.house_choices.get(choice_key) else {
                    issues.push(unresolved(choice_key));
                    continue;
                };
                if !open_pick_satisfies(pick, constraint, ruleset) {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_HOUSE_GRANT_CONSTRAINT,
                        args([
                            ("house", house_id.to_string()),
                            ("choice_key", choice_key.clone()),
                            ("item", pick.item_ref.to_string()),
                        ]),
                        Some(pick.item_ref.clone()),
                    ));
                }
            }
        }
    }
}

/// Validates a Mythic Companion's chosen *type* (Devil Child, Faerie Doctor, …):
/// its free-Virtue grants resolve and its required V/F package is present. Gated
/// on the profile's `has_mythic_type` capability flag (never a hardcoded type
/// id), mirroring how [`validate_house`] gates on `is_magus`.
///
/// - No type chosen → `mythic_type_unset` warning (a "should", not a hard rule).
/// - Each `Choice`/`Open` grant pick is resolved from `entity.mythic_choices`
///   (identical machinery to House grants); a missing/off-menu pick →
///   `mythic_choice_unresolved`, an Open pick violating its constraint →
///   `mythic_grant_constraint`.
/// - The required package (fixed Virtues + each required Flaw's default OR a
///   "suitable substitute agreed with the troupe") is checked against the bought
///   `entity.selections`; a missing slot → a non-blocking
///   `mythic_required_trait_missing` warning, so Enforced mode never hard-blocks
///   a legal-with-substitute build. Required Virtues match by full `Selection`
///   (ref + params) so parameterized/duplicated requirements — Nephilim's two
///   distinct Great Characteristics, Puissant Guile — are matched precisely.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2635-2639, 2842-2851.
fn validate_mythic_type(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    // Mythic types are a mythic-companion-only concern; gated on the capability
    // flag so no type id is hardcoded here.
    let Some(profile) = type_profile else {
        return;
    };
    if !profile.has_mythic_type {
        return;
    }

    // No type chosen: a soft warning, and there are no grants/package to resolve.
    let Some(type_id) = &entity.mythic_type else {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_MYTHIC_TYPE_UNSET,
            args([]),
            None,
        ));
        return;
    };

    // An unknown type id resolves to nothing; nothing further to check.
    let Some(mtype) = ruleset.mythic_type(type_id) else {
        return;
    };

    // --- Free-Virtue grant picks (Choice/Open), mirroring validate_house. ---
    let unresolved = |choice_key: &str| {
        ValidationIssue::error(
            ValidationIssue::CODE_MYTHIC_CHOICE_UNRESOLVED,
            args([
                ("mythic_type", type_id.to_string()),
                ("choice_key", choice_key.to_string()),
            ]),
            None,
        )
    };
    for grant in &mtype.grants {
        match grant {
            Grant::Fixed { .. } => {}
            Grant::Choice {
                choice_key,
                options,
            } => {
                let pick = entity.mythic_choices.get(choice_key);
                if !pick.is_some_and(|p| options.contains(p)) {
                    issues.push(unresolved(choice_key));
                }
            }
            Grant::Open {
                choice_key,
                constraint,
            } => {
                let Some(pick) = entity.mythic_choices.get(choice_key) else {
                    issues.push(unresolved(choice_key));
                    continue;
                };
                if !open_pick_satisfies(pick, constraint, ruleset) {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_MYTHIC_GRANT_CONSTRAINT,
                        args([
                            ("mythic_type", type_id.to_string()),
                            ("choice_key", choice_key.clone()),
                            ("item", pick.item_ref.to_string()),
                        ]),
                        Some(pick.item_ref.clone()),
                    ));
                }
            }
        }
    }

    // --- Required package (non-blocking warnings; substitutes allowed). ---
    let missing = |item: &Id| {
        ValidationIssue::warning(
            ValidationIssue::CODE_MYTHIC_REQUIRED_TRAIT_MISSING,
            args([("item", item.to_string())]),
            Some(item.clone()),
        )
    };
    // Fixed required Virtues: matched by full Selection (ref + params).
    for req in &mtype.required_virtues {
        if !entity.selections.contains(req) {
            issues.push(missing(&req.item_ref));
        }
    }
    // Required Flaws: the default, or any bought selection satisfying the
    // substitute constraint (kind/magnitude/category) — the troupe-substitute
    // allowance.
    for flaw in &mtype.required_flaws {
        let satisfied = entity
            .selections
            .iter()
            .any(|s| open_pick_satisfies(s, &flaw.constraint, ruleset));
        if !satisfied {
            issues.push(missing(&flaw.default.item_ref));
        }
    }
}

/// Tri-state outcome of evaluating a prerequisite expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tri {
    /// Definitely satisfied.
    True,
    /// Definitely unsatisfied.
    False,
    /// Cannot be evaluated with the data currently on the entity.
    Unknown,
}

fn validate_prerequisites(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let is_magus = type_profile.map(|p| p.is_magus);

    // Effective score per ability: the max bought score (a parameterized ability
    // may appear more than once with different specialties; the highest wins)
    // plus any virtue bonus (Puissant Ability +2, which now includes a
    // House-granted Puissant via the combined selection list). `AbilityMin`
    // thresholds are checked against the effective score so a boosted ability
    // satisfies them. Keyed by owned `Id` so House-granted ability *floors*
    // (below) can be folded in even for abilities that were never bought.
    let mut ability_scores: BTreeMap<Id, u8> = BTreeMap::new();
    for a in &entity.ability_scores {
        // Per-instance bonus (Puissant targets one (ability, parameter)); an
        // `AbilityMin` is keyed by id, so the strongest instance wins.
        let bonus =
            crate::effective::ability_bonus(entity, ruleset, &a.ability, a.parameter.as_deref());
        let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = ability_scores.entry(a.ability.clone()).or_insert(0);
        *entry = (*entry).max(effective);
    }
    // A free ability-score floor from an `AbilityScoreGrant` effect — including a
    // House-granted Mystery Ability (Bjornaer → Heartbeast 1) — counts toward
    // `AbilityMin` even with no bought row, so fold each granted floor in.
    for floor in crate::effective::ability_score_floors(entity, ruleset) {
        let bonus = crate::effective::ability_bonus(entity, ruleset, &floor.ability, None);
        let effective = (floor.floor + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = ability_scores.entry(floor.ability).or_insert(0);
        *entry = (*entry).max(effective);
    }

    // Effective score per Art: max bought score plus any virtue bonus (Puissant
    // Art +3, including a House-granted Puissant). `ArtMin` thresholds are
    // checked against the effective score.
    let mut art_scores: BTreeMap<Id, u8> = BTreeMap::new();
    for a in &entity.art_scores {
        let bonus = crate::effective::art_bonus(entity, ruleset, &a.art);
        let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = art_scores.entry(a.art.clone()).or_insert(0);
        *entry = (*entry).max(effective);
    }

    // `Prereq::Has` resolves against bought AND granted rows (a granted
    // Heartbeast/Dowsing satisfies `Has(...)`), so build a grants-inclusive id
    // set spanning House and Mythic-Companion-type grants. This is deliberately
    // distinct from the bought-only `selected_ids` that the forbidden-trait /
    // incompatibility validators use — grants must never reach those (review
    // finding B1).
    let granted = crate::effective::entity_grants(entity, ruleset);
    let mut present_ids: BTreeSet<&Id> = selected_ids.iter().copied().collect();
    for g in &granted {
        present_ids.insert(&g.item_ref);
    }

    let ctx = PrereqCtx {
        present_ids: &present_ids,
        is_magus,
        house: entity.house.as_ref(),
        ability_scores: &ability_scores,
        art_scores: &art_scores,
    };
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if let Some(ref prereq) = item.prerequisites {
            let (outcome, depended_on_unknown) = evaluate_prereq(prereq, &ctx);
            match outcome {
                Tri::False => {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_PREREQ_NOT_MET,
                        args([("item", selection.item_ref.to_string())]),
                        Some(selection.item_ref.clone()),
                    ));
                }
                Tri::Unknown if depended_on_unknown => {
                    issues.push(ValidationIssue::warning(
                        ValidationIssue::CODE_PREREQ_UNEVALUATED,
                        args([("item", selection.item_ref.to_string())]),
                        Some(selection.item_ref.clone()),
                    ));
                }
                _ => {}
            }
        }
    }
}

/// The read-only context a prerequisite is evaluated against: which items are
/// selected, whether the type is a magus, and the effective Ability/Art score
/// maps the `AbilityMin`/`ArtMin` thresholds compare against. Bundled so the
/// recursive evaluator and its fold helper take one context rather than a long
/// positional argument list.
struct PrereqCtx<'a> {
    /// The grants-inclusive id set (bought selections ++ House-granted rows) that
    /// `Prereq::Has` tests against — NOT the bought-only `selected_ids` used by
    /// the forbidden-trait / incompatibility checks (review finding B1).
    present_ids: &'a BTreeSet<&'a Id>,
    is_magus: Option<bool>,
    /// The entity's own Hermetic House, if any. `Prereq::House` compares against
    /// it: matching → True, differing → False, absent → Unknown (mirrors how
    /// `is_magus` yields Unknown when the profile is missing).
    house: Option<&'a Id>,
    ability_scores: &'a BTreeMap<Id, u8>,
    art_scores: &'a BTreeMap<Id, u8>,
}

/// Evaluates a prerequisite to a tri-state. Returns the outcome plus whether an
/// unevaluable leaf actually influenced the result (so a warning is only worth
/// emitting when the answer genuinely hinges on missing data).
fn evaluate_prereq(prereq: &Prereq, ctx: &PrereqCtx) -> (Tri, bool) {
    match prereq {
        // The three quantifiers share one tri-state fold over their children,
        // differing only in: which child outcome short-circuits, what the
        // expression then evaluates to, and the value when every child is known
        // and none triggered the short-circuit.
        //   All (AND): trigger on False  -> short-circuit False; all-known -> True
        //   Any (OR) : trigger on True   -> short-circuit True;  all-known -> False
        //   Nor      : trigger on True   -> short-circuit False; all-known -> True
        // In every case a surviving Unknown makes the whole expression Unknown.
        Prereq::All(children) => fold_children(children, ctx, Tri::False, Tri::False, Tri::True),
        Prereq::Any(children) => fold_children(children, ctx, Tri::True, Tri::True, Tri::False),
        Prereq::Nor(children) => fold_children(children, ctx, Tri::True, Tri::False, Tri::True),
        Prereq::Has(id) => {
            if ctx.present_ids.contains(id) {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // IsMagus is enforced against the profile's explicit `is_magus` flag (a
        // Hermetic-Magus-status type), independent of gift_policy.
        Prereq::IsMagus => match ctx.is_magus {
            Some(true) => (Tri::True, false),
            Some(false) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
        // AbilityMin compares against the entity's max *effective* score for
        // that ability (bought score plus virtue bonuses such as Puissant
        // Ability), as supplied by the caller. An ability the entity does not
        // have counts as score 0, so any positive threshold is False.
        Prereq::AbilityMin { ability, score } => {
            let have = ctx.ability_scores.get(ability).copied().unwrap_or(0);
            if have >= *score {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // ArtMin compares against the entity's max *effective* Art score (bought
        // plus Puissant Art). An Art the entity does not have counts as 0.
        Prereq::ArtMin { art, score } => {
            let have = ctx.art_scores.get(art).copied().unwrap_or(0);
            if have >= *score {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // House matches against the entity's own house: a known house that
        // matches is True, a known house that differs is False, and no house at
        // all (non-magus or an unset magus) is genuinely Unknown.
        Prereq::House(id) => match ctx.house {
            Some(h) if h == id => (Tri::True, false),
            Some(_) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
    }
}

/// Tri-state fold shared by the `All`/`Any`/`Nor` quantifiers (see the call
/// sites for the per-quantifier parameterization).
///
/// Walks the children once: if any child evaluates to `trigger`, the whole
/// expression short-circuits to `short_circuit` (a definite True/False, so its
/// dependency flag is irrelevant downstream and reported as `false`). Otherwise,
/// a surviving `Unknown` makes the result `Unknown` (carrying whether that
/// hinged on genuinely missing data); if every child is known, the result is
/// `all_known`.
fn fold_children(
    children: &[Prereq],
    ctx: &PrereqCtx,
    trigger: Tri,
    short_circuit: Tri,
    all_known: Tri,
) -> (Tri, bool) {
    let mut depended = false;
    let mut saw_unknown = false;
    for child in children {
        let (outcome, dep) = evaluate_prereq(child, ctx);
        if outcome == trigger {
            return (short_circuit, false);
        }
        if outcome == Tri::Unknown {
            saw_unknown = true;
            depended |= dep;
        }
    }
    if saw_unknown {
        (Tri::Unknown, depended)
    } else {
        (all_known, false)
    }
}

fn validate_incompatibilities(
    entity: &Entity,
    ruleset: &Ruleset,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut reported: BTreeSet<(&Id, &Id)> = BTreeSet::new();

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        for incompat_id in &item.incompatible_with {
            if selected_ids.contains(incompat_id) {
                // Normalize the pair order so a mutual incompatibility is
                // reported exactly once.
                let pair = if selection.item_ref < *incompat_id {
                    (&selection.item_ref, incompat_id)
                } else {
                    (incompat_id, &selection.item_ref)
                };
                if reported.insert(pair) {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_INCOMPATIBLE,
                        args([
                            ("item", selection.item_ref.to_string()),
                            ("other", incompat_id.to_string()),
                        ]),
                        Some(selection.item_ref.clone()),
                    ));
                }
            }
        }
    }
}

fn validate_permitted_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    // An empty permitted list means "no category restriction".
    if profile.permitted_categories.is_empty() {
        return;
    }

    for selection in &entity.selections {
        // The profile's own gift is governed solely by `validate_gift_policy`
        // (required/allowed/forbidden). Exempt it here: it is contradictory for a
        // profile to mandate a trait via `gift_policy` yet reject its category
        // (The Gift is `special`), so gifted profiles need not whitelist it.
        if profile.gift_id.as_ref() == Some(&selection.item_ref) {
            continue;
        }

        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if !profile.permitted_categories.contains(&item.category) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CATEGORY_NOT_PERMITTED,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("category", item.category.clone()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

fn validate_forbidden_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    if profile.forbidden_categories.is_empty() {
        return;
    }

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if profile.forbidden_categories.contains(&item.category) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_FORBIDDEN_CATEGORY,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("category", item.category.clone()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

fn validate_entity_kind_applicability(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if !item.entity_kinds.is_empty() && !item.entity_kinds.contains(&entity.entity_kind) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_WRONG_ENTITY_KIND,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("entity_kind", entity.entity_kind.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

fn validate_duplicate_selections(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut seen: BTreeMap<(&Id, &BTreeMap<String, Id>), usize> = BTreeMap::new();

    for selection in &entity.selections {
        let key = (&selection.item_ref, &selection.params);
        *seen.entry(key).or_insert(0) += 1;
    }

    for ((item_ref, _params), count) in &seen {
        // Selections are grouped by (item_ref, params): two selections of the
        // same parameterized item with DIFFERENT params are distinct targets and
        // do not collide here. An item may be taken up to `max_per_target` times
        // for the same target (default 1; Great Characteristic allows 2).
        let max = ruleset
            .point_items
            .get(*item_ref)
            .map_or(1, |item| usize::from(item.max_per_target));
        if *count <= max {
            continue;
        }
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_DUPLICATE_SELECTION,
            args([
                ("item", item_ref.to_string()),
                ("count", count.to_string()),
                ("max", max.to_string()),
            ]),
            Some((*item_ref).clone()),
        ));
    }
}

fn validate_required_traits(
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    for required_id in &profile.required_traits {
        if !selected_ids.contains(required_id) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_MISSING_REQUIRED_TRAIT,
                args([("item", required_id.to_string())]),
                Some(required_id.clone()),
            ));
        }
    }
}

fn validate_forbidden_traits(
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    for forbidden_id in &profile.forbidden_traits {
        if selected_ids.contains(forbidden_id) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_FORBIDDEN_TRAIT,
                args([("item", forbidden_id.to_string())]),
                Some(forbidden_id.clone()),
            ));
        }
    }
}

/// Validates that each selection of a parameterized item supplies exactly the
/// declared parameter keys (no missing, no extra) and that each provided value
/// resolves against its domain's registry: `item` → point items, `ability` →
/// the ability catalogue, `art` → the art catalogue, `characteristic` →
/// [`Characteristic::from_id`]. A value that does not resolve emits
/// `unknown_param_value`.
fn validate_parameters(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        let declared: BTreeSet<&str> = item.parameters.iter().map(|p| p.key.as_str()).collect();
        let provided: BTreeSet<&str> = selection.params.keys().map(String::as_str).collect();

        // A parameter targeting a PARAMETERIZED ability also expects the instance
        // discriminator, supplied under the target ability's own param key
        // ((Area) Lore → "area"). So Puissant on (Area) Lore needs both keys; on a
        // plain ability the instance key would be an unexpected extra.
        let mut expected = declared.clone();
        for param in &item.parameters {
            if matches!(param.domain, ParameterDomain::Ability)
                && let Some(target) = selection.params.get(&param.key)
                && let Some(ability) = ruleset.abilities.get(target)
                && let Some(instance_key) = ability.parameter.as_deref()
            {
                expected.insert(instance_key);
            }
        }

        for missing in expected.difference(&provided) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_MISSING_PARAM,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("key", missing.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }

        for extra in provided.difference(&expected) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNEXPECTED_PARAM,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("key", extra.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }

        // Resolve values for domains that have a registry (Item -> point items,
        // Ability -> ability catalogue, Art -> art catalogue).
        for param in &item.parameters {
            let Some(value) = selection.params.get(&param.key) else {
                continue; // missing already reported above
            };
            let resolves = match param.domain {
                ParameterDomain::Item => ruleset.point_items.contains_key(value),
                ParameterDomain::Ability => ruleset.abilities.contains_key(value),
                ParameterDomain::Characteristic => Characteristic::from_id(value).is_some(),
                ParameterDomain::Art => ruleset.arts.contains_key(value),
            };
            if !resolves {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_UNKNOWN_PARAM_VALUE,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("key", param.key.clone()),
                        ("value", value.to_string()),
                        ("domain", param.domain.to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}

/// Validates that every ability-bonus effect (e.g. Puissant Ability +2) targets
/// an ability instance the character actually holds. The target is
/// `(ability, parameter)`: for a parameterized ability ((Area) Lore) the instance
/// value is read from the selection's matching key, so Puissant "Brandenburg Lore"
/// must have a bought Brandenburg Lore row. A dangling target (e.g. the ability was
/// removed) means the +2 attaches to nothing, so flag it. Effect-driven — no virtue
/// id is hardcoded.
fn validate_ability_bonus_targets(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-skipped target check.
            let param = match effect {
                // Affinity reduces the cost of buying one ability, so it too must
                // target a held instance — the cost break attaches to nothing
                // otherwise, exactly like a dangling Puissant.
                Effect::AbilityBonus { param, .. } | Effect::AffinityAbilityCost { param, .. } => {
                    param
                }
                // Art bonuses are not parameterized instances; their target is
                // resolved by validate_parameters (domain check). Characteristic
                // limits are handled elsewhere. AbilityScoreGrant *creates* the
                // score, so it needs no pre-existing bought row. The XP-pool and
                // characteristic-budget grants carry no target.
                Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicScoreDelta { .. }
                | Effect::GrantsReputation { .. } => continue,
            };
            let Some(target) = selection.params.get(param) else {
                continue; // missing ability key already reported by validate_parameters
            };
            // The instance discriminator, if the target ability is parameterized.
            let instance = ruleset
                .abilities
                .get(target)
                .and_then(|a| a.parameter.as_deref())
                .and_then(|key| selection.params.get(key).map(Id::as_str));
            let has_instance = entity
                .ability_scores
                .iter()
                .any(|a| &a.ability == target && a.parameter.as_deref() == instance);
            if !has_instance {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_ABILITY_BONUS_DANGLING_TARGET,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("ability", target.to_string()),
                        ("parameter", instance.unwrap_or("").to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}

/// Enforces the type's Gift policy (required / allowed / forbidden). The policy
/// per type is data; this is the mechanism the book's Gift rules map onto.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2868-2877 (The Gift:
/// "all magi must have this Virtue"; "Grogs can never have The Gift"); magi must
/// take The Gift at :2858; only magi may take the Hermetic Magus Social Status
/// at :2293 and :4067-4069.
fn validate_gift_policy(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let Some(policy) = profile.gift_policy else {
        return;
    };

    // Shared with the Supernatural free-slot computation so both use one
    // definition of "has The Gift".
    let has_gift = crate::effective::has_the_gift(entity, ruleset, profile);

    match policy {
        GiftPolicy::Required => {
            if !has_gift {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_GIFT_REQUIRED,
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Forbidden => {
            if has_gift {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_GIFT_FORBIDDEN,
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Allowed => {}
    }
}

/// Validates Characteristic point-buy: each score must be a legal table value
/// and within the characteristic's per-target buy range, and the total cost must
/// not exceed the starting points (over = error, under = a non-blocking "points
/// unspent" warning, mirroring the V/F balance rule). No-op when the ruleset
/// ships no characteristic rules.
///
/// The buy range is the base ±3 by default, widened upward by Great
/// (Characteristic) and downward by Poor (Characteristic) (see
/// [`characteristic_cap`](crate::effective::characteristic_cap) /
/// [`characteristic_floor`](crate::effective::characteristic_floor)). The cost
/// table itself spans the absolute ±5 range so the higher/lower scores can be
/// priced; without the virtue/flaw they are legal table values but above the cap
/// / below the floor.
///
/// The point-spend check is skipped entirely when the character has no
/// Characteristics set: an untouched step is not yet under-spent, so a fresh
/// character is not nagged. Out-of-range scores are always flagged.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2340-2354 (the cost
/// table and the seven starting points), :4105 (the +3 base cap), :3987-3989
/// (Great's +5), :6598-6600 (Poor's −5). The numbers themselves are data in
/// `rules/core/characteristics.json` (see RULES.md).
fn validate_characteristics(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let Some(rules) = ruleset.characteristic_rules() else {
        return;
    };
    let (Some(min), Some(max)) = (rules.min_score(), rules.max_score()) else {
        return;
    };

    for (&characteristic, &score) in &entity.characteristics {
        if !rules.is_legal_score(score) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CHARACTERISTIC_OUT_OF_RANGE,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("score", score.to_string()),
                    ("min", min.to_string()),
                    ("max", max.to_string()),
                ]),
                None,
            ));
            continue;
        }
        // A legal table value still has to sit within the range that this
        // character's Great/Poor (Characteristic) choices open for the target.
        let cap = crate::effective::characteristic_cap(entity, ruleset, characteristic);
        let floor = crate::effective::characteristic_floor(entity, ruleset, characteristic);
        if i32::from(score) > cap {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CHARACTERISTIC_ABOVE_CAP,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("score", score.to_string()),
                    ("cap", cap.to_string()),
                ]),
                None,
            ));
        } else if i32::from(score) < floor {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CHARACTERISTIC_BELOW_FLOOR,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("score", score.to_string()),
                    ("floor", floor.to_string()),
                ]),
                None,
            ));
        }
    }

    // Don't evaluate the point spend before the user has touched the step.
    if entity.characteristics.is_empty() {
        return;
    }

    let cost = rules.total_cost(&entity.characteristics);
    // Improved Characteristics (+3 each, stackable) raises the buy budget above
    // the ruleset's base start_points.
    let granted = crate::effective::characteristic_points_granted(entity, ruleset);
    let budget = i32::from(rules.start_points) + granted;
    if cost > budget {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_CHARACTERISTIC_OVERSPENT,
            args([("cost", cost.to_string()), ("points", budget.to_string())]),
            None,
        ));
    } else if cost < budget {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_CHARACTERISTIC_POINTS_UNSPENT,
            args([("cost", cost.to_string()), ("points", budget.to_string())]),
            None,
        ));
    }
}

/// Enforces the parameter-relative precondition on `characteristic_limit`
/// effects: a limit-shift may only be taken on a characteristic whose *base*
/// (bought) score is already at the limit being extended. Great (Characteristic,
/// positive amount) needs base ≥ the base cap (+3); Poor (Characteristic,
/// negative amount) needs base ≤ the base floor (−3). The threshold is derived
/// from the ruleset's base cap/floor by the sign of the amount, so no per-effect
/// number is stored. This is parameter-relative (it constrains whichever
/// characteristic the selection targets), so it lives here rather than in the
/// static [`Prereq`] tree.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3987-3989 (Great,
/// "already … at least +3"), :6598-6600 (Poor, "already −3 or lower").
fn validate_characteristic_limit_preconditions(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(rules) = ruleset.characteristic_rules() else {
        return;
    };
    let base_max = rules.base_max_score();
    let base_min = rules.base_min_score();
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-skipped precondition check.
            let (amount, target, base) = match effect {
                Effect::CharacteristicLimit { param, amount } => {
                    let Some(target) = selection
                        .params
                        .get(param)
                        .and_then(Characteristic::from_id)
                    else {
                        continue; // unresolved param value is reported by validate_parameters
                    };
                    let base = entity.characteristics.get(&target).copied().unwrap_or(0);
                    (*amount, target, base)
                }
                Effect::AbilityBonus { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicScoreDelta { .. }
                | Effect::GrantsReputation { .. } => continue,
            };
            if amount > 0 {
                if let Some(cap) = base_max
                    && i32::from(base) < i32::from(cap)
                {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_CHARACTERISTIC_MAX_BASE_TOO_LOW,
                        args([
                            ("item", selection.item_ref.to_string()),
                            ("characteristic", target.to_string()),
                            ("base", base.to_string()),
                            ("min", cap.to_string()),
                        ]),
                        Some(selection.item_ref.clone()),
                    ));
                }
            } else if amount < 0
                && let Some(floor) = base_min
                && i32::from(base) > i32::from(floor)
            {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_CHARACTERISTIC_MIN_BASE_TOO_HIGH,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("characteristic", target.to_string()),
                        ("base", base.to_string()),
                        ("max", floor.to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}

/// Validates Ability scores: every referenced ability must resolve against the
/// catalogue, no (ability, parameter) pair may appear twice, a parameterized
/// ability must carry a parameter value, and the total XP the bought scores cost
/// may not exceed the character's `xp_pool`.
///
/// A parameterized ability (e.g. `(Area) Lore`) is identified by its instance
/// `parameter` (the area / language), so a character may hold several; plain
/// abilities have no parameter and are deduped by id (one instance).
///
/// The age cap is deferred to M4. XP cost per score comes from the advancement
/// table (`AdvancementTable::xp_for_score`); a non-zero score with no table row
/// is off-table and flagged `ability_score_out_of_range` (mirroring the
/// characteristic range check), so an illegal score is never silently priced at
/// 0 XP. The XP a score costs is summed against the shared pool by
/// [`validate_xp_pool`], not here.
fn validate_abilities(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let mut seen: BTreeMap<(&Id, Option<&str>), u32> = BTreeMap::new();
    // The highest score the advancement table prices. A ruleset that ships no
    // advancement table has no legal score range to check against, so off-table
    // range checking is skipped.
    let max_score = ruleset.advancement.max_score();

    for entry in &entity.ability_scores {
        match ruleset.abilities.get(&entry.ability) {
            None => issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_ABILITY,
                args([("ability", entry.ability.to_string())]),
                Some(entry.ability.clone()),
            )),
            Some(ability) => {
                // A parameterized ability needs its value supplied (which Area?).
                if ability.parameter.is_some()
                    && entry.parameter.as_deref().is_none_or(str::is_empty)
                {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_ABILITY_PARAMETER_REQUIRED,
                        args([("ability", entry.ability.to_string())]),
                        Some(entry.ability.clone()),
                    ));
                }
            }
        }
        // The advancement table covers the legal score range. A non-zero score
        // with no table row is off-table (illegal) — flag it rather than silently
        // pricing it at 0 XP, so direct-entry illegal states surface here instead
        // of relying on the UI to keep them out (mirrors characteristic range
        // checking).
        // A non-zero score with no table row is off-table (illegal) — flag it
        // rather than silently pricing it at 0 XP, so direct-entry illegal states
        // surface here (mirrors characteristic range checking).
        if ruleset.advancement.xp_for_score(entry.score).is_none()
            && let Some(max) = max_score
        {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_ABILITY_SCORE_OUT_OF_RANGE,
                args([
                    ("ability", entry.ability.to_string()),
                    ("score", entry.score.to_string()),
                    ("max", max.to_string()),
                ]),
                Some(entry.ability.clone()),
            ));
        }
        // Age → max-Ability-score cap (Core:2366-2376). An Ability carrying an
        // Affinity may exceed it by +2 (Core:3374), not without limit.
        if let Some(age) = entity.age {
            let mut cap = u32::from(crate::effective::age_max_ability_score(age));
            if crate::effective::ability_affinity(
                entity,
                ruleset,
                &entry.ability,
                entry.parameter.as_deref(),
            )
            .is_some()
            {
                cap += 2;
            }
            if u32::from(entry.score) > cap {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_ABILITY_ABOVE_AGE_CAP,
                    args([
                        ("ability", entry.ability.to_string()),
                        ("score", entry.score.to_string()),
                        ("cap", cap.to_string()),
                        ("age", age.to_string()),
                    ]),
                    Some(entry.ability.clone()),
                ));
            }
        }
        let key = (&entry.ability, entry.parameter.as_deref());
        *seen.entry(key).or_insert(0) += 1;
    }

    for ((ability, _parameter), count) in seen {
        if count > 1 {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_DUPLICATE_ABILITY,
                args([
                    ("ability", ability.to_string()),
                    ("count", count.to_string()),
                ]),
                Some(ability.clone()),
            ));
        }
    }
}

/// Validates that every held Supernatural Ability is legal: it must be covered by
/// a granting Virtue (an `ability_score_grant` floor) or fit within the Gift's
/// free slot (one for a Gifted non-magus, none for a magus). Uncovered instances
/// beyond the free allowance emit `supernatural_ability_requires_virtue`
/// (deterministic by sorted id). Source: Core Rules.md:2874.
fn validate_supernatural_abilities(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };
    let (free_total, _used) = crate::effective::supernatural_free_slots(entity, ruleset, profile);
    // A granting Virtue seeds an `ability_score_grant` floor; such abilities are
    // "covered" and never consume the free slot.
    let floors: std::collections::BTreeSet<Id> =
        crate::effective::ability_score_floors(entity, ruleset)
            .into_iter()
            .map(|f| f.ability)
            .collect();
    let mut uncovered: Vec<&Id> = entity
        .ability_scores
        .iter()
        .filter(|a| {
            ruleset
                .abilities
                .get(&a.ability)
                .is_some_and(|ab| ab.category == crate::ability::AbilityCategory::Supernatural)
        })
        .map(|a| &a.ability)
        .filter(|id| !floors.contains(*id))
        .collect();
    uncovered.sort();
    uncovered.dedup();
    // The first `free_total` uncovered abilities occupy the free Gift slot(s); the
    // rest require a granting Virtue.
    for ability in uncovered.into_iter().skip(usize::from(free_total)) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_SUPERNATURAL_ABILITY_REQUIRES_VIRTUE,
            args([("ability", ability.to_string())]),
            Some(ability.clone()),
        ));
    }
}

/// Validates Personality Traits: `|value|` never exceeds 6, and at most one trait
/// per selected Major Personality Flaw may exceed ±3 (a Major Personality Flaw is
/// represented by a single ±6 trait; others stay ±3). Source: Core Rules.md:2500-2503.
fn validate_personality_traits(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let major_personality_flaws = entity
        .selections
        .iter()
        .filter(|s| {
            ruleset.point_items.get(&s.item_ref).is_some_and(|item| {
                item.category == "personality" && item.magnitude == Magnitude::Major
            })
        })
        .count();

    // Traits are sorted by name (normalize) for a deterministic "excess" choice.
    let mut traits: Vec<&crate::types::PersonalityTrait> =
        entity.personality_traits.iter().collect();
    traits.sort_by(|a, b| a.name.cmp(&b.name));
    let mut over_three_budget = major_personality_flaws;
    for trait_ in traits {
        let magnitude = trait_.value.unsigned_abs();
        if magnitude > 6 {
            issues.push(personality_out_of_range(trait_, 6));
        } else if magnitude > 3 {
            if over_three_budget > 0 {
                over_three_budget -= 1;
            } else {
                issues.push(personality_out_of_range(trait_, 3));
            }
        }
    }
}

fn personality_out_of_range(trait_: &crate::types::PersonalityTrait, max: i8) -> ValidationIssue {
    ValidationIssue::error(
        ValidationIssue::CODE_PERSONALITY_TRAIT_OUT_OF_RANGE,
        args([
            ("name", trait_.name.clone()),
            ("value", trait_.value.to_string()),
            ("max", max.to_string()),
        ]),
        None,
    )
}

/// Validates that every starting Reputation is backed by a granting Virtue/Flaw:
/// the count of reputations of each `kind` must not exceed the grants of that kind
/// (`Effect::GrantsReputation`). Excess reputations emit `reputation_not_granted`.
/// Source: Core Rules.md:2514.
fn validate_reputations(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    use crate::types::ReputationType;
    let mut remaining: BTreeMap<ReputationType, usize> = BTreeMap::new();
    for (kind, _score) in crate::effective::reputation_grants(entity, ruleset) {
        *remaining.entry(kind).or_insert(0) += 1;
    }
    for reputation in &entity.reputations {
        let slot = remaining.entry(reputation.kind).or_insert(0);
        if *slot > 0 {
            *slot -= 1;
        } else {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_REPUTATION_NOT_GRANTED,
                args([
                    ("kind", reputation.kind.to_string()),
                    ("content", reputation.content.clone()),
                ]),
                None,
            ));
        }
    }
}

/// Validates Hermetic Art scores (mirrors [`validate_abilities`], minus the
/// parameter logic — Arts are not parameterized): every referenced Art must
/// resolve against the catalogue, no Art may appear twice, and every bought score
/// must be priced by the Art advancement table. The XP a score costs is summed
/// against the shared pool by [`validate_xp_pool`], not here.
fn validate_arts(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let mut seen: BTreeMap<&Id, u32> = BTreeMap::new();
    // The highest score the Art advancement table prices. A ruleset that ships no
    // Art advancement table has no legal score range to check against, so
    // off-table range checking is skipped.
    let max_score = ruleset.art_advancement.max_score();

    for entry in &entity.art_scores {
        if !ruleset.arts.contains_key(&entry.art) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_ART,
                args([("art", entry.art.to_string())]),
                Some(entry.art.clone()),
            ));
        }
        // A non-zero score with no table row is off-table (illegal) — flag it
        // rather than silently pricing it at 0 XP.
        if ruleset.art_advancement.xp_for_score(entry.score).is_none()
            && let Some(max) = max_score
        {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_ART_SCORE_OUT_OF_RANGE,
                args([
                    ("art", entry.art.to_string()),
                    ("score", entry.score.to_string()),
                    ("max", max.to_string()),
                ]),
                Some(entry.art.clone()),
            ));
        }
        *seen.entry(&entry.art).or_insert(0) += 1;
    }

    for (art, count) in seen {
        if count > 1 {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_DUPLICATE_ART,
                args([("art", art.to_string()), ("count", count.to_string())]),
                Some(art.clone()),
            ));
        }
    }
}

/// Validates a magus's spell list: every referenced spell must resolve; the same
/// spell at the same level may not appear twice (different General levels are
/// different spells, Core:12353); a General spell with no chosen level is excluded
/// from the budget and warned; the sum of chosen levels must not exceed the
/// effective spell-levels budget (Core:2215-2216, 2435); and no spell's level may
/// exceed Technique + Form + Intelligence + Magic Theory + 3 (Core:2465).
///
/// The budget and per-spell cap apply only to magi (`profile.is_magus`); a stray
/// spell on a non-magus is ref- and dedup-checked only (spells are magus-only).
fn validate_spells(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let is_magus = type_profile.is_some_and(|p| p.is_magus);
    let mut seen: BTreeMap<(&Id, Option<u32>), u32> = BTreeMap::new();

    for sel in &entity.spells {
        let Some(spell) = ruleset.spell(&sel.spell) else {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_SPELL,
                args([("spell", sel.spell.to_string())]),
                Some(sel.spell.clone()),
            ));
            continue;
        };
        let resolved = crate::effective::resolved_spell_level(sel, ruleset);
        // A General spell (catalogue level None) with no chosen level cannot be
        // budgeted yet — warn, don't block.
        if spell.level.is_none() && sel.level.is_none() {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_SPELL_LEVEL_UNRESOLVED,
                args([("spell", sel.spell.to_string())]),
                Some(sel.spell.clone()),
            ));
        }
        *seen.entry((&sel.spell, resolved)).or_insert(0) += 1;

        if is_magus && let Some(level) = resolved {
            let cap = spell_level_cap(entity, ruleset, spell);
            if i64::from(level) > cap {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP,
                    args([
                        ("spell", sel.spell.to_string()),
                        ("level", level.to_string()),
                        ("cap", cap.max(0).to_string()),
                    ]),
                    Some(sel.spell.clone()),
                ));
            }
        }
    }

    for ((spell, _level), count) in seen {
        if count > 1 {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_DUPLICATE_SPELL,
                args([("spell", spell.to_string()), ("count", count.to_string())]),
                Some(spell.clone()),
            ));
        }
    }

    if is_magus {
        let base = type_profile.map(|p| p.spell_levels).unwrap_or(0);
        let budget = crate::effective::spell_levels_budget(base, entity, ruleset);
        let used = crate::effective::spell_levels_used(entity, ruleset);
        if used > budget {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_OVER_SPELL_LEVELS,
                args([
                    ("used", used.to_string()),
                    ("budget", budget.to_string()),
                    ("over", (used - budget).to_string()),
                ]),
                None,
            ));
        }
    }
}

/// The maximum level a magus may learn of a spell: the sum of Technique, Form,
/// Intelligence, Magic Theory and 3 (Core:2465), using effective Art/Ability
/// scores. Returns an `i64` (small or negative for a beginning magus).
/// Requisite-Art reduction is a lab-total nuance out of M4 scope.
fn spell_level_cap(entity: &Entity, ruleset: &Ruleset, spell: &crate::spell::Spell) -> i64 {
    let tech = i64::from(crate::effective::effective_art_score(
        entity,
        ruleset,
        &spell.technique,
    ));
    let form = i64::from(crate::effective::effective_art_score(
        entity,
        ruleset,
        &spell.form,
    ));
    let int = i64::from(
        entity
            .characteristics
            .get(&Characteristic::Int)
            .copied()
            .unwrap_or(0),
    );
    let magic_theory = i64::from(crate::effective::effective_ability_score(
        entity,
        ruleset,
        &Id::new("ability.magic_theory"),
        None,
    ));
    tech + form + int + magic_theory + 3
}

/// Validates the experience pools: Abilities and Arts are bought from the shared
/// general bank (`Entity::xp_pool`) plus any restricted grants (Educated/Warrior/
/// Privileged), each Affinity-reduced. Feasibility is a max-flow solve over the
/// general pool + restricted pools; an infeasible allocation overspends. Reported
/// as an error rather than blocked: direct-entry allows the illegal state and
/// surfaces it (the M4 wizard blocks the spend up front). Leftover restricted XP
/// the rules waste raises a non-blocking warning.
fn validate_xp_pool(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let allocation = crate::effective::xp_allocation(entity, ruleset);
    if allocation.total_demand > allocation.max_flow {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_NOT_ENOUGH_XP,
            args([
                ("spent", allocation.total_demand.to_string()),
                ("pool", entity.xp_pool.to_string()),
                (
                    "shortfall",
                    (allocation.total_demand - allocation.max_flow).to_string(),
                ),
            ]),
            None,
        ));
    }
    for pool in &allocation.restricted {
        if pool.used < pool.amount {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_RESTRICTED_XP_UNSPENT,
                args([
                    ("amount", pool.amount.to_string()),
                    ("used", pool.used.to_string()),
                    ("unspent", (pool.amount - pool.used).to_string()),
                ]),
                None,
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
          {
            "id": "virtue.the_gift",
            "kind": "virtue",
            "magnitude": "free",
            "category": "special",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.hermetic_magus",
            "kind": "virtue",
            "magnitude": "free",
            "category": "social_status",
            "entity_kinds": ["character"],
            "prerequisites": { "kind": "has", "value": "virtue.the_gift" }
          },
          {
            "id": "virtue.gentle_gift",
            "kind": "virtue",
            "magnitude": "major",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "prerequisites": { "kind": "has", "value": "virtue.hermetic_magus" },
            "incompatible_with": ["flaw.blatant_gift"]
          },
          {
            "id": "flaw.blatant_gift",
            "kind": "flaw",
            "magnitude": "major",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "prerequisites": { "kind": "has", "value": "virtue.the_gift" },
            "incompatible_with": ["virtue.gentle_gift"]
          },
          {
            "id": "virtue.puissant_ability",
            "kind": "virtue",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }]
          },
          {
            "id": "flaw.poor_student",
            "kind": "flaw",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.keen_vision",
            "kind": "virtue",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.large",
            "kind": "virtue",
            "magnitude": "minor",
            "category": "general",
            "entity_kinds": ["character"]
          },
          {
            "id": "virtue.tough",
            "kind": "virtue",
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
            characteristics: None,
        })
        .unwrap()
    }

    // --- Phase 4 (step 8): House-granted Virtues in the two id-sets ----------

    /// Abilities the grant-bearing test Houses seed. Registered so the
    /// `AbilityMin` prereq ref and the `AbilityScoreGrant` effect resolve at load.
    const GRANT_ABILITIES: &str = r#"{ "abilities": [
        { "id": "ability.heartbeast", "category": "supernatural", "requires_training": true }
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
        { "id": "virtue.the_gift", "kind": "virtue", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.heartbeast", "kind": "virtue", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"],
          "effects": [{ "type": "ability_score_grant", "ability": "ability.heartbeast", "amount": 1 }],
          "incompatible_with": ["virtue.foe_of_heartbeast"] },
        { "id": "virtue.needs_heartbeast", "kind": "virtue", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "prerequisites": { "kind": "has", "value": "virtue.heartbeast" } },
        { "id": "virtue.needs_heartbeast_ability", "kind": "virtue", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "prerequisites": { "kind": "ability_min", "value": { "ability": "ability.heartbeast", "score": 1 } } },
        { "id": "virtue.foe_of_heartbeast", "kind": "virtue", "magnitude": "minor",
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
            characteristics: None,
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
        { "id": "virtue.the_gift", "kind": "virtue", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.gentle_gift", "kind": "virtue", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"] },
        { "id": "virtue.mythic_blood", "kind": "virtue", "magnitude": "major",
          "category": "hermetic", "entity_kinds": ["character"] },
        { "id": "virtue.heartbeast", "kind": "virtue", "magnitude": "major",
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
        { "id": "virtue.the_gift", "kind": "virtue", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.tainted_a", "kind": "virtue", "magnitude": "major",
          "category": "supernatural", "entity_kinds": ["character"], "tainted": true },
        { "id": "virtue.tainted_b", "kind": "virtue", "magnitude": "major",
          "category": "supernatural", "entity_kinds": ["character"], "tainted": true },
        { "id": "virtue.plain", "kind": "virtue", "magnitude": "major",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "flaw.tainted_c", "kind": "flaw", "magnitude": "major",
          "category": "story", "entity_kinds": ["character"], "tainted": true },
        { "id": "flaw.tainted_d", "kind": "flaw", "magnitude": "major",
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
        { "id": "virtue.the_gift", "kind": "virtue", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.puissant_art", "kind": "virtue", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "parameters": [{ "key": "art", "type": "ref", "domain": "art" }] },
        { "id": "virtue.self_confident", "kind": "virtue", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "virtue.wealthy", "kind": "virtue", "magnitude": "major",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "flaw.optimistic", "kind": "flaw", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"] },
        { "id": "flaw.deficient_technique", "kind": "flaw", "magnitude": "major",
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
            characteristics: None,
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
        let mut entity = make_entity("magus", vec![sel("flaw.optimistic")]);
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

    #[test]
    fn over_budget_virtues() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.major_a", "kind": "virtue", "magnitude": "major", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.major_b", "kind": "virtue", "magnitude": "major", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "flaw.minor_a", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.minor_b", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.pers_major", "kind": "flaw", "magnitude": "major", "category": "personality", "entity_kinds": ["character"]},
          {"id": "flaw.pers_minor_a", "kind": "flaw", "magnitude": "minor", "category": "personality", "entity_kinds": ["character"]},
          {"id": "flaw.pers_minor_b", "kind": "flaw", "magnitude": "minor", "category": "personality", "entity_kinds": ["character"]},
          {"id": "flaw.story_a", "kind": "flaw", "magnitude": "minor", "category": "story", "entity_kinds": ["character"]},
          {"id": "flaw.story_b", "kind": "flaw", "magnitude": "minor", "category": "story", "entity_kinds": ["character"]},
          {"id": "virtue.v1", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.v2", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.v3", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
        let issue = ValidationIssue::error("over_budget_virtues", BTreeMap::new(), None);

        let json = serde_json::to_string(&issue).unwrap();
        assert!(
            !json.contains("context"),
            "a None context must be omitted from JSON: {json}"
        );

        let roundtripped: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(issue, roundtripped);

        // Deserialization must also accept JSON that omits `context` entirely.
        let without_context = r#"{"severity":"error","code":"over_budget_virtues"}"#;
        let parsed: ValidationIssue = serde_json::from_str(without_context).unwrap();
        assert_eq!(parsed.context, None);
    }

    #[test]
    fn public_constructors_build_issues_and_results() {
        let issue = ValidationIssue::new(
            IssueSeverity::Warning,
            "too_many_story_flaws",
            BTreeMap::new(),
            None,
        );
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.code, "too_many_story_flaws");

        let result = ValidationResult::new(vec![
            ValidationIssue::error("unknown_type", BTreeMap::new(), None),
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
          {"id": "flaw.a", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.b", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "flaw.major_a", "kind": "flaw", "magnitude": "major", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.major_b", "kind": "flaw", "magnitude": "major", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "any", "value": [{"kind": "has", "value": "virtue.a"}, {"kind": "house", "value": "house.flambeau"}]}},
          {"id": "flaw.x", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.y", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.dep", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.dep", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.requires_awareness_3", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"kind": "ability_min", "value": {"ability": "ability.awareness", "score": 3}}},
          {"id": "virtue.puissant_ability", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "ability", "type": "ref", "domain": "ability"}],
           "effects": [{"type": "ability_bonus", "param": "ability", "amount": 2}]},
          {"id": "virtue.great_characteristic", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "characteristic", "type": "ref", "domain": "characteristic"}],
           "effects": [{"type": "characteristic_limit", "param": "characteristic", "amount": 1}],
           "max_per_target": 2},
          {"id": "virtue.improved_characteristics", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "effects": [{"type": "characteristic_points", "amount": 3}]},
          {"id": "flaw.poor_characteristic", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "parameters": [{"key": "characteristic", "type": "ref", "domain": "characteristic"}],
           "effects": [{"type": "characteristic_limit", "param": "characteristic", "amount": -1}],
           "max_per_target": 2},
          {"id": "flaw.f", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
        Ruleset::from_core_json("test", "1", items, types, abilities, characteristics).unwrap()
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
        Ruleset::from_core_json("test", "1", "[]", types, abilities, characteristics).unwrap()
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
          {"id": "virtue.the_gift", "kind": "virtue", "magnitude": "free", "category": "special", "entity_kinds": ["character"]}
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
        Ruleset::from_core_json_with_arts("test", "1", items, types, abilities, arts, "").unwrap()
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
          {"id": "virtue.the_gift", "kind": "virtue", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.educated", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "effects": [{ "type": "restricted_ability_xp", "amount": 50, "abilities": ["ability.latin", "ability.artes_liberales"] }]},
          {"id": "virtue.affinity_art", "kind": "virtue", "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"],
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
        Ruleset::from_core_json_with_arts("test", "1", items, types, abilities, arts, "").unwrap()
    }

    #[test]
    fn restricted_xp_left_unspent_warns() {
        // Educated grants 50 XP restricted to Latin/Artes Liberales; with none of
        // it spent, the rules waste it — a non-blocking warning, not an error.
        let rs = restricted_xp_ruleset();
        let mut e = make_entity("companion", vec![sel("virtue.educated")]);
        e.xp_pool = 0;
        let result = validate(&e, &rs);
        assert!(
            warning_codes(&result).contains(&"restricted_xp_unspent".to_string()),
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
        Ruleset::from_core_json_with_arts("test", "1", items, types, "{}", arts, "").unwrap()
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.the_gift", "kind": "virtue", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.the_gift", "kind": "virtue", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
          {"id": "virtue.char_only", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.universal", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": []}
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
          {"id": "virtue.the_gift", "kind": "virtue", "magnitude": "free", "category": "special", "entity_kinds": ["character"]}
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
          {"id": "virtue.weird", "kind": "virtue", "magnitude": "minor", "category": "obscure", "entity_kinds": ["character"]}
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
          {"id": "virtue.hermetic_thing", "kind": "virtue", "magnitude": "minor", "category": "hermetic", "entity_kinds": ["character"]}
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
          {"id": "virtue.the_gift", "kind": "virtue", "magnitude": "free", "category": "special", "entity_kinds": ["character"]},
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.the_gift", "kind": "virtue", "magnitude": "free", "category": "special", "entity_kinds": ["character"]}
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
          {"id": "virtue.parma", "kind": "virtue", "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"]}
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
          {"id": "virtue.parma", "kind": "virtue", "magnitude": "major", "category": "hermetic", "entity_kinds": ["character"]}
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
          {"id": "virtue.mundane", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.mandatory", "kind": "virtue", "magnitude": "free", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.banned", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
          {"id": "virtue.target", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.linked", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
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
    fn art_domain_param_value_resolves_against_registry() {
        // Art-domain parameter values are now resolved against the Art catalogue:
        // a real Art passes, a made-up one raises `unknown_param_value`.
        let items = r#"[{
          "id": "virtue.puissant_art", "kind": "virtue", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "parameters": [{"key": "art", "type": "ref", "domain": "art"}],
          "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }]
        }]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let arts = r#"{ "arts": [ { "id": "art.creo", "art_type": "technique" } ] }"#;
        let rs =
            Ruleset::from_core_json_with_arts("test", "1", items, types, "{}", arts, "").unwrap();

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
          {"id": "virtue.major", "kind": "virtue", "magnitude": "major", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.free", "kind": "virtue", "magnitude": "free", "category": "general", "entity_kinds": ["character"]},
          {"id": "flaw.minor", "kind": "flaw", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]}
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
                ValidationIssue::error("err1", BTreeMap::new(), None),
                ValidationIssue::warning("warn1", BTreeMap::new(), None),
                ValidationIssue::error("err2", BTreeMap::new(), None),
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
            { "id": "virtue.devil_child", "kind": "virtue", "magnitude": "free",
              "category": "social_status", "entity_kinds": ["character"] },
            { "id": "virtue.demonic_might", "kind": "virtue", "magnitude": "minor",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "virtue.demonic_powers", "kind": "virtue", "magnitude": "minor",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "virtue.demonic_blood", "kind": "virtue", "magnitude": "major",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "flaw.tragic_life", "kind": "flaw", "magnitude": "major",
              "category": "supernatural", "entity_kinds": ["character"] },
            { "id": "flaw.other_supernatural", "kind": "flaw", "magnitude": "major",
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
            { "id": "mythic_type.faerie_doctor" }
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
            characteristics: None,
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
        assert_eq!(effective_point_ceilings(&devil, &rs), Some((37, 17)));
        // Faerie Doctor (no bonus) and a plain companion stay at their base.
        let faerie = mythic_entity("mythic_type.faerie_doctor");
        assert_eq!(effective_point_ceilings(&faerie, &rs), Some((20, 10)));
        let companion = make_entity("companion", vec![]);
        assert_eq!(effective_point_ceilings(&companion, &rs), Some((10, 10)));
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
        { "id": "virtue.skilled_parens", "kind": "virtue", "magnitude": "minor",
          "category": "hermetic", "entity_kinds": ["character"],
          "effects": [ { "type": "spell_levels", "amount": 30 },
                       { "type": "general_xp", "amount": 60 } ] }
    ]"#;
    const SPELL_ARTS: &str = r#"{ "arts": [
        { "id": "art.creo", "art_type": "technique" },
        { "id": "art.ignem", "art_type": "form" },
        { "id": "art.rego", "art_type": "technique" },
        { "id": "art.vim", "art_type": "form" }
    ] }"#;
    const SPELL_ABILITIES: &str = r#"{ "abilities": [
        { "id": "ability.magic_theory", "category": "arcane" }
    ] }"#;
    const SPELL_CATALOGUE: &str = r#"{ "spells": [
        { "id": "spell.pilum_of_fire", "technique": "art.creo", "form": "art.ignem", "level": 20 },
        { "id": "spell.ball_of_abysmal_flame", "technique": "art.creo", "form": "art.ignem", "level": 35 },
        { "id": "spell.aegis_of_the_hearth", "technique": "art.rego", "form": "art.vim" }
    ] }"#;
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
            characteristics: None,
        })
        .unwrap()
    }

    fn spell(id: &str, level: Option<u8>) -> SpellSelection {
        SpellSelection {
            spell: Id::new(id),
            level,
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
        { "id": "virtue.the_gift", "kind": "virtue", "magnitude": "free",
          "category": "special", "entity_kinds": ["character"] },
        { "id": "virtue.second_sight", "kind": "virtue", "magnitude": "minor",
          "category": "supernatural", "entity_kinds": ["character"],
          "effects": [{ "type": "ability_score_grant", "ability": "ability.second_sight", "amount": 1 }] },
        { "id": "virtue.affinity_awareness", "kind": "virtue", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
          "effects": [{ "type": "affinity_ability_cost", "param": "ability", "counts_as_num": 3, "counts_as_den": 2 }] },
        { "id": "virtue.self_confident", "kind": "virtue", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "effects": [{ "type": "confidence_bonus", "score": 1, "points": 2 }] },
        { "id": "flaw.infamous", "kind": "flaw", "magnitude": "minor",
          "category": "general", "entity_kinds": ["character"],
          "effects": [{ "type": "grants_reputation", "kind": "local", "score": 4 }] },
        { "id": "flaw.major_personality", "kind": "flaw", "magnitude": "major",
          "category": "personality", "entity_kinds": ["character"] }
    ]"#;
    const P7_ABILITIES: &str = r#"{ "advancement": [
        { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
        { "score": 7, "total_xp": 140 }, { "score": 8, "total_xp": 180 } ],
        "abilities": [
        { "id": "ability.awareness", "category": "general" },
        { "id": "ability.second_sight", "category": "supernatural", "requires_training": true },
        { "id": "ability.animal_ken", "category": "supernatural", "requires_training": true } ] }"#;
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
            characteristics: None,
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
