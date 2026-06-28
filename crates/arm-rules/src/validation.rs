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
use crate::ruleset::Ruleset;
use crate::types::{
    Effect, Entity, EntityKind, EntityTypeProfile, GiftPolicy, Id, ItemKind, Magnitude,
    ParameterDomain, PointItem, Prereq, ValidationMode,
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
/// | `not_enough_xp` | error | `spent`, `pool` |
/// | `ability_parameter_required` | error | `ability` |
/// | `ability_score_out_of_range` | error | `ability`, `score`, `max` |
/// | `ability_bonus_dangling_target` | error | `item`, `ability`, `parameter` |
///
/// † The per-category flaw caps emit a code derived from the
/// `flaw_category_caps` entry's category slug (`too_many_<category>_flaws`, or
/// `too_many_major_<category>_flaws` when the cap is `major_only`); severity
/// follows the cap's `hard` flag. The shipped `personality`/`story` caps thus
/// produce `too_many_major_personality_flaws` (error),
/// `too_many_personality_flaws` (warning), and `too_many_story_flaws`
/// (warning); a new category requires its matching `issue-<code>` Fluent key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Error or warning.
    pub severity: IssueSeverity,
    /// Stable machine key for this issue. The UI derives the Fluent message id
    /// as `issue-<code>` (e.g. code `over_budget_virtues` →
    /// `issue-over_budget_virtues`).
    pub code: String,
    /// Interpolation values for the localized message, keyed by argument name.
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
    validate_prerequisites(entity, ruleset, type_profile, &selected_ids, &mut issues);
    validate_incompatibilities(entity, ruleset, &selected_ids, &mut issues);
    validate_permitted_categories(entity, ruleset, type_profile, &mut issues);
    validate_forbidden_categories(entity, ruleset, type_profile, &mut issues);
    validate_required_traits(type_profile, &selected_ids, &mut issues);
    validate_forbidden_traits(type_profile, &selected_ids, &mut issues);
    validate_parameters(entity, ruleset, &mut issues);
    validate_ability_bonus_targets(entity, ruleset, &mut issues);
    validate_gift_policy(entity, ruleset, type_profile, &mut issues);

    // Characteristics and Abilities are character-only concerns; a covenant has
    // neither. Gate them on the entity kind so the engine respects EntityKind
    // rather than relying on a covenant happening to carry no such data.
    if entity.entity_kind == EntityKind::Character {
        validate_characteristics(entity, ruleset, &mut issues);
        validate_characteristic_limit_preconditions(entity, ruleset, &mut issues);
        validate_abilities(entity, ruleset, &mut issues);
        validate_arts(entity, ruleset, &mut issues);
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

    if virtue_points > profile.budget.virtue_points as i32 {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_BUDGET_VIRTUES,
            args([
                ("points", virtue_points.to_string()),
                ("budget", profile.budget.virtue_points.to_string()),
            ]),
            None,
        ));
    }

    if flaw_points > profile.budget.flaw_points as i32 {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_BUDGET_FLAWS,
            args([
                ("points", flaw_points.to_string()),
                ("budget", profile.budget.flaw_points.to_string()),
            ]),
            None,
        ));
    }

    // Each flaw point funds `virtue_points_per_flaw_point` virtue points (1 for
    // most types; 2 for Mythic Companions). Source: Core Rules.md:2638.
    let funded_virtue_points = flaw_points * profile.budget.virtue_points_per_flaw_point as i32;
    if virtue_points > funded_virtue_points {
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

    // --- Data-driven per-category flaw caps ---
    //
    // Each cap names its flaw category as data, so the engine never hardcodes a
    // category slug. A `hard` cap is a blocking error; otherwise a non-blocking
    // warning (the book marks the Personality/Story guidelines as
    // troupe-overridable). The issue code is derived from the category slug as
    // `too_many_<category>_flaws` (or `too_many_major_<category>_flaws` when the
    // cap is Major-only), so the Fluent key follows the category by convention —
    // no slug is baked into the engine. The shipped `personality`/`story` caps
    // thus map onto the existing Fluent keys without a hardcoded mapping.
    for cap in &profile.budget.flaw_category_caps {
        let n = count(&|i| {
            i.kind == ItemKind::Flaw
                && i.category == cap.category
                && (!cap.major_only || i.magnitude == Magnitude::Major)
        });
        if n <= cap.max as usize {
            continue;
        }

        let code = if cap.major_only {
            format!("too_many_major_{}_flaws", cap.category)
        } else {
            format!("too_many_{}_flaws", cap.category)
        };
        let cap_args = count_args(n, cap.max);

        if cap.hard {
            issues.push(ValidationIssue::error(&code, cap_args, None));
        } else {
            issues.push(ValidationIssue::warning(&code, cap_args, None));
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
    // plus any virtue bonus (Puissant Ability +2). `AbilityMin` thresholds are
    // checked against the effective score so a boosted ability satisfies them.
    let mut ability_scores: BTreeMap<&Id, u8> = BTreeMap::new();
    for a in &entity.ability_scores {
        // Per-instance bonus (Puissant targets one (ability, parameter)); an
        // `AbilityMin` is keyed by id, so the strongest instance wins.
        let bonus =
            crate::effective::ability_bonus(entity, ruleset, &a.ability, a.parameter.as_deref());
        let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = ability_scores.entry(&a.ability).or_insert(0);
        *entry = (*entry).max(effective);
    }

    // Effective score per Art: max bought score plus any virtue bonus (Puissant
    // Art +3). `ArtMin` thresholds are checked against the effective score.
    let mut art_scores: BTreeMap<&Id, u8> = BTreeMap::new();
    for a in &entity.art_scores {
        let bonus = crate::effective::art_bonus(entity, ruleset, &a.art);
        let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = art_scores.entry(&a.art).or_insert(0);
        *entry = (*entry).max(effective);
    }

    let ctx = PrereqCtx {
        selected_ids,
        is_magus,
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
    selected_ids: &'a BTreeSet<&'a Id>,
    is_magus: Option<bool>,
    ability_scores: &'a BTreeMap<&'a Id, u8>,
    art_scores: &'a BTreeMap<&'a Id, u8>,
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
            if ctx.selected_ids.contains(id) {
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
        // No house metadata on the entity yet: genuinely unknown (Phase 4).
        Prereq::House(_) => (Tri::Unknown, true),
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
/// declared parameter keys (no missing, no extra). Param values are accepted
/// as-is for `ability`/`art` domains (no registry yet); `item`-domain values
/// are resolved against the ruleset's point items.
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
                Effect::AbilityBonus { param, .. } => param,
                // Art bonuses are not parameterized instances; their target is
                // resolved by validate_parameters (domain check). Characteristic
                // limits are handled elsewhere.
                Effect::CharacteristicLimit { .. } | Effect::ArtBonus { .. } => continue,
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

    let has_gift_id = match profile.gift_id {
        Some(ref gift_id) => entity.selections.iter().any(|s| s.item_ref == *gift_id),
        None => false,
    };

    let has_gift_category = !profile.gift_categories.is_empty()
        && entity.selections.iter().any(|s| {
            ruleset
                .point_items
                .get(&s.item_ref)
                .is_some_and(|item| profile.gift_categories.contains(&item.category))
        });

    // Symmetric: both branches use the same definition of "has the Gift".
    let has_gift = has_gift_id || has_gift_category;

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
    let budget = rules.start_points as i32;
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
                Effect::AbilityBonus { .. } | Effect::ArtBonus { .. } => continue,
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

/// Total XP an entity has committed across Abilities and Arts, each priced from
/// its own advancement table. A score the table cannot price (off-table, already
/// flagged by [`validate_abilities`] / [`validate_arts`]) contributes 0 rather
/// than being counted at a wrong price.
fn xp_spent(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let abilities: u32 = entity
        .ability_scores
        .iter()
        .filter_map(|a| ruleset.advancement.xp_for_score(a.score))
        .sum();
    let arts: u32 = entity
        .art_scores
        .iter()
        .filter_map(|a| ruleset.art_advancement.xp_for_score(a.score))
        .sum();
    abilities + arts
}

/// Validates the shared experience pool: Abilities and Arts are bought from one
/// bank (`Entity::xp_pool`), so their combined cost may not exceed it.
/// Overspending is reported as an error rather than blocked: direct-entry allows
/// the illegal state and surfaces it (the M4 wizard blocks the spend up front).
fn validate_xp_pool(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let spent = xp_spent(entity, ruleset);
    if spent > entity.xp_pool {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_NOT_ENOUGH_XP,
            args([
                ("spent", spent.to_string()),
                ("pool", entity.xp_pool.to_string()),
            ]),
            None,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::characteristics::Characteristic;
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(warning_codes.contains(&"prereq_unevaluated"));
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
}
