use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ruleset::Ruleset;
use crate::types::{
    Entity, EntityTypeProfile, GiftPolicy, Id, ItemKind, Magnitude, PointItem, Prereq,
    ValidationMode,
};

/// Whether a validation issue blocks (`Error`) or merely advises (`Warning`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// A rule violation that makes the entity illegal.
    Error,
    /// A non-blocking advisory (e.g. a prerequisite that cannot be evaluated yet).
    Warning,
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
/// | `duplicate_selection` | error | `item`, `count` |
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
    validate_duplicate_selections(entity, &mut issues);
    validate_balance(entity, ruleset, type_profile, &mut issues);
    validate_caps(entity, ruleset, type_profile, &mut issues);
    validate_prerequisites(entity, ruleset, type_profile, &selected_ids, &mut issues);
    validate_incompatibilities(entity, ruleset, &selected_ids, &mut issues);
    validate_permitted_categories(entity, ruleset, type_profile, &mut issues);
    validate_forbidden_categories(entity, ruleset, type_profile, &mut issues);
    validate_required_traits(type_profile, &selected_ids, &mut issues);
    validate_forbidden_traits(type_profile, &selected_ids, &mut issues);
    validate_parameters(entity, ruleset, &mut issues);
    validate_gift_policy(entity, ruleset, type_profile, &mut issues);

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
            "unknown_type",
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
                "unknown_ref",
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
            "over_budget_virtues",
            args([
                ("points", virtue_points.to_string()),
                ("budget", profile.budget.virtue_points.to_string()),
            ]),
            None,
        ));
    }

    if flaw_points > profile.budget.flaw_points as i32 {
        issues.push(ValidationIssue::error(
            "over_budget_flaws",
            args([
                ("points", flaw_points.to_string()),
                ("budget", profile.budget.flaw_points.to_string()),
            ]),
            None,
        ));
    }

    if virtue_points > flaw_points {
        issues.push(ValidationIssue::error(
            "unbalanced_virtues",
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
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2857 ("You may not
/// have more than one Major Hermetic Virtue", magi); grogs may take no Major
/// Virtues or Flaws at :2824-2830; ≤5 Minor Flaws (central) at :2774, grogs ≤3
/// at :1009; ≤1 Major Personality Flaw at :2820; ≤2 Personality Flaws (soft) at
/// :2820/:2976; ≤1 Story Flaw (soft) at :2818, grogs none at :1009. See
/// RULES.md.
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
                "too_many_major_virtues",
                count_args(n, max),
                None,
            ));
        }
    }

    if let Some(max) = profile.budget.max_major_flaws {
        let n = count(&|i| i.kind == ItemKind::Flaw && i.magnitude == Magnitude::Major);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                "too_many_major_flaws",
                count_args(n, max),
                None,
            ));
        }
    }

    if let Some(max) = profile.budget.max_minor_flaws {
        let n = count(&|i| i.kind == ItemKind::Flaw && i.magnitude == Magnitude::Minor);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                "too_many_minor_flaws",
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

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if let Some(ref prereq) = item.prerequisites {
            let (outcome, depended_on_unknown) = evaluate_prereq(prereq, selected_ids, is_magus);
            match outcome {
                Tri::False => {
                    issues.push(ValidationIssue::error(
                        "prereq_not_met",
                        args([("item", selection.item_ref.to_string())]),
                        Some(selection.item_ref.clone()),
                    ));
                }
                Tri::Unknown if depended_on_unknown => {
                    issues.push(ValidationIssue::warning(
                        "prereq_unevaluated",
                        args([("item", selection.item_ref.to_string())]),
                        Some(selection.item_ref.clone()),
                    ));
                }
                _ => {}
            }
        }
    }
}

/// Evaluates a prerequisite to a tri-state. Returns the outcome plus whether an
/// unevaluable leaf actually influenced the result (so a warning is only worth
/// emitting when the answer genuinely hinges on missing data).
fn evaluate_prereq(
    prereq: &Prereq,
    selected_ids: &BTreeSet<&Id>,
    is_magus: Option<bool>,
) -> (Tri, bool) {
    match prereq {
        Prereq::All(children) => {
            // AND: any False -> False; else any Unknown -> Unknown; else True.
            let mut depended = false;
            let mut saw_unknown = false;
            for child in children {
                let (outcome, dep) = evaluate_prereq(child, selected_ids, is_magus);
                match outcome {
                    Tri::False => return (Tri::False, dep),
                    Tri::Unknown => {
                        saw_unknown = true;
                        depended |= dep;
                    }
                    Tri::True => {}
                }
            }
            if saw_unknown {
                (Tri::Unknown, depended)
            } else {
                (Tri::True, false)
            }
        }
        Prereq::Any(children) => {
            // OR: any True -> True; else any Unknown -> Unknown; else False.
            let mut depended = false;
            let mut saw_unknown = false;
            for child in children {
                let (outcome, dep) = evaluate_prereq(child, selected_ids, is_magus);
                match outcome {
                    Tri::True => return (Tri::True, false),
                    Tri::Unknown => {
                        saw_unknown = true;
                        depended |= dep;
                    }
                    Tri::False => {}
                }
            }
            if saw_unknown {
                (Tri::Unknown, depended)
            } else {
                (Tri::False, false)
            }
        }
        Prereq::None(children) => {
            // NOR: any True -> False; else any Unknown -> Unknown; else True.
            let mut depended = false;
            let mut saw_unknown = false;
            for child in children {
                let (outcome, dep) = evaluate_prereq(child, selected_ids, is_magus);
                match outcome {
                    Tri::True => return (Tri::False, false),
                    Tri::Unknown => {
                        saw_unknown = true;
                        depended |= dep;
                    }
                    Tri::False => {}
                }
            }
            if saw_unknown {
                (Tri::Unknown, depended)
            } else {
                (Tri::True, false)
            }
        }
        Prereq::Has(id) => {
            if selected_ids.contains(id) {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // IsMagus is enforced against the profile's explicit `is_magus` flag (a
        // Hermetic-Magus-status type), independent of gift_policy.
        Prereq::IsMagus => match is_magus {
            Some(true) => (Tri::True, false),
            Some(false) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
        // No house/ability/art metadata on the entity yet: genuinely unknown.
        Prereq::House(_) | Prereq::AbilityMin { .. } | Prereq::ArtMin { .. } => {
            (Tri::Unknown, true)
        }
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
                        "incompatible",
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
                "category_not_permitted",
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
                "forbidden_category",
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
                "wrong_entity_kind",
                args([
                    ("item", selection.item_ref.to_string()),
                    ("entity_kind", entity.entity_kind.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

fn validate_duplicate_selections(entity: &Entity, issues: &mut Vec<ValidationIssue>) {
    let mut seen: BTreeMap<(&Id, &BTreeMap<String, Id>), usize> = BTreeMap::new();

    for selection in &entity.selections {
        let key = (&selection.item_ref, &selection.params);
        *seen.entry(key).or_insert(0) += 1;
    }

    for ((item_ref, _params), count) in &seen {
        if *count <= 1 {
            continue;
        }
        // Selections are de-duplicated by (item_ref, params): two selections of
        // the same parameterized item with DIFFERENT params are legal and do
        // not collide here. Only identical (ref + params) pairs are flagged.
        issues.push(ValidationIssue::error(
            "duplicate_selection",
            args([("item", item_ref.to_string()), ("count", count.to_string())]),
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
                "missing_required_trait",
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
                "forbidden_trait",
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

        for missing in declared.difference(&provided) {
            issues.push(ValidationIssue::error(
                "missing_param",
                args([
                    ("item", selection.item_ref.to_string()),
                    ("key", missing.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }

        for extra in provided.difference(&declared) {
            issues.push(ValidationIssue::error(
                "unexpected_param",
                args([
                    ("item", selection.item_ref.to_string()),
                    ("key", extra.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }

        // Resolve values for domains that have a registry.
        for param in &item.parameters {
            let Some(value) = selection.params.get(&param.key) else {
                continue; // missing already reported above
            };
            if param.domain.resolves_against_items() && !ruleset.point_items.contains_key(value) {
                issues.push(ValidationIssue::error(
                    "unknown_param_value",
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
                    "gift_required",
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Forbidden => {
            if has_gift {
                issues.push(ValidationIssue::error(
                    "gift_forbidden",
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Allowed => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        Ruleset::from_json("arm5-core", "2024.1", items, types).unwrap()
    }

    fn make_entity(type_id: &str, selections: Vec<Selection>) -> Entity {
        Entity {
            schema_version: 1,
            ruleset: RulesetRef::new(Id::new("arm5-core"), "2024.1"),
            entity_kind: EntityKind::Character,
            type_id: Id::new(type_id),
            selections,
        }
    }

    fn sel(item_ref: &str) -> Selection {
        Selection::new(Id::new(item_ref))
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

    #[test]
    fn prereq_ability_min_produces_unevaluated_warning() {
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(warning_codes.contains(&"prereq_unevaluated"));
    }

    #[test]
    fn prereq_art_min_produces_unevaluated_warning() {
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
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        let entity = make_entity("test_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let warning_codes: Vec<&str> = result.warnings().map(|i| i.code.as_str()).collect();
        assert!(warning_codes.contains(&"prereq_unevaluated"));
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
