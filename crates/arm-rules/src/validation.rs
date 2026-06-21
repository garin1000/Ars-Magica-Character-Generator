use serde::{Deserialize, Serialize};

use crate::ruleset::Ruleset;
use crate::types::{Entity, Id, ItemKind, Magnitude, Prereq, ValidationMode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub context: Option<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        !self
            .issues
            .iter()
            .any(|i| i.severity == IssueSeverity::Error)
    }

    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Error)
            .collect()
    }

    /// Returns all issues with [`IssueSeverity::Warning`].
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Warning)
            .collect()
    }

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
/// incompatibilities, categories, traits, and gift policy.
pub fn validate(entity: &Entity, ruleset: &Ruleset) -> ValidationResult {
    let mut issues = Vec::new();

    let type_profile = ruleset.type_profiles.get(&entity.type_id);

    if type_profile.is_none() {
        issues.push(ValidationIssue {
            severity: IssueSeverity::Error,
            code: "unknown_type".into(),
            message: format!("unknown type_id: '{}'", entity.type_id),
            context: None,
        });
    }

    for selection in &entity.selections {
        if !ruleset.point_items.contains_key(&selection.item_ref) {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "unknown_ref".into(),
                message: format!("unknown item reference: '{}'", selection.item_ref),
                context: Some(selection.item_ref.clone()),
            });
        }
    }

    validate_entity_kind_applicability(entity, ruleset, &mut issues);
    validate_duplicate_selections(entity, ruleset, &mut issues);
    validate_balance(entity, ruleset, type_profile, &mut issues);
    validate_caps(entity, ruleset, type_profile, &mut issues);
    validate_prerequisites(entity, ruleset, &mut issues);
    validate_incompatibilities(entity, ruleset, &mut issues);
    validate_permitted_categories(entity, ruleset, type_profile, &mut issues);
    validate_forbidden_categories(entity, ruleset, type_profile, &mut issues);
    validate_required_traits(entity, type_profile, &mut issues);
    validate_forbidden_traits(entity, type_profile, &mut issues);
    validate_gift_policy(entity, ruleset, type_profile, &mut issues);

    ValidationResult { issues }
}

fn validate_balance(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&crate::types::EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let (virtue_points, flaw_points) = compute_balance(entity, ruleset);

    if virtue_points > profile.budget.virtue_points as i32 {
        issues.push(ValidationIssue {
            severity: IssueSeverity::Error,
            code: "over_budget_virtues".into(),
            message: format!(
                "virtue points ({virtue_points}) exceed budget ({})",
                profile.budget.virtue_points
            ),
            context: None,
        });
    }

    if flaw_points > profile.budget.flaw_points as i32 {
        issues.push(ValidationIssue {
            severity: IssueSeverity::Error,
            code: "over_budget_flaws".into(),
            message: format!(
                "flaw points ({flaw_points}) exceed budget ({})",
                profile.budget.flaw_points
            ),
            context: None,
        });
    }
}

/// Computes the total virtue and flaw points for an entity.
/// Returns (virtue_points, flaw_points).
pub fn compute_balance(entity: &Entity, ruleset: &Ruleset) -> (i32, i32) {
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

    (virtue_points, flaw_points)
}

fn validate_caps(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&crate::types::EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let count_major = |kind: ItemKind| -> usize {
        entity
            .selections
            .iter()
            .filter(|s| {
                ruleset
                    .point_items
                    .get(&s.item_ref)
                    .is_some_and(|item| item.kind == kind && item.magnitude == Magnitude::Major)
            })
            .count()
    };

    if let Some(max) = profile.budget.max_major_virtues {
        let count = count_major(ItemKind::Virtue);
        if count > max as usize {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "too_many_major_virtues".into(),
                message: format!("too many Major Virtues ({count}, max {max})"),
                context: None,
            });
        }
    }

    if let Some(max) = profile.budget.max_major_flaws {
        let count = count_major(ItemKind::Flaw);
        if count > max as usize {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "too_many_major_flaws".into(),
                message: format!("too many Major Flaws ({count}, max {max})"),
                context: None,
            });
        }
    }
}

fn validate_prerequisites(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let selected_ids: Vec<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if let Some(ref prereq) = item.prerequisites
            && !evaluate_prereq(prereq, &selected_ids, &selection.item_ref, issues)
        {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "prereq_not_met".into(),
                message: format!("'{}' has unmet prerequisites", selection.item_ref),
                context: Some(selection.item_ref.clone()),
            });
        }
    }
}

fn evaluate_prereq(
    prereq: &Prereq,
    selected_ids: &[&Id],
    context_id: &Id,
    issues: &mut Vec<ValidationIssue>,
) -> bool {
    match prereq {
        Prereq::All(children) => children
            .iter()
            .all(|c| evaluate_prereq(c, selected_ids, context_id, issues)),
        Prereq::Any(children) => children
            .iter()
            .any(|c| evaluate_prereq(c, selected_ids, context_id, issues)),
        Prereq::None(children) => !children
            .iter()
            .any(|c| evaluate_prereq(c, selected_ids, context_id, issues)),
        Prereq::Has(id) => selected_ids.contains(&id),
        // TODO: Implement when Entity carries house metadata.
        Prereq::House(house_id) => {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "prereq_unevaluated".into(),
                message: format!(
                    "'{}': House prerequisite '{house_id}' cannot be evaluated yet",
                    context_id
                ),
                context: Some(context_id.clone()),
            });
            true
        }
        // TODO: Implement when Entity carries ability/art scores.
        Prereq::AbilityMin { ability, score } => {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "prereq_unevaluated".into(),
                message: format!(
                    "'{}': AbilityMin prerequisite '{ability}' >= {score} cannot be evaluated yet",
                    context_id
                ),
                context: Some(context_id.clone()),
            });
            true
        }
        Prereq::ArtMin { art, score } => {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "prereq_unevaluated".into(),
                message: format!(
                    "'{}': ArtMin prerequisite '{art}' >= {score} cannot be evaluated yet",
                    context_id
                ),
                context: Some(context_id.clone()),
            });
            true
        }
        // TODO: Implement by checking entity metadata (e.g. type_id or
        // selected social status) once the magus detection strategy is decided.
        Prereq::IsMagus => {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                code: "prereq_unevaluated".into(),
                message: format!(
                    "'{}': IsMagus prerequisite cannot be evaluated yet",
                    context_id
                ),
                context: Some(context_id.clone()),
            });
            true
        }
    }
}

fn validate_incompatibilities(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let selected_ids: Vec<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();
    let mut reported: std::collections::BTreeSet<(&Id, &Id)> = std::collections::BTreeSet::new();

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        for incompat_id in &item.incompatible_with {
            if selected_ids.contains(&incompat_id) {
                // Only report when item_ref < incompat_id to avoid duplicate reports.
                let pair = if selection.item_ref < *incompat_id {
                    (&selection.item_ref, incompat_id)
                } else {
                    (incompat_id, &selection.item_ref)
                };
                if reported.insert(pair) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        code: "incompatible".into(),
                        message: format!(
                            "'{}' is incompatible with '{incompat_id}'",
                            selection.item_ref
                        ),
                        context: Some(selection.item_ref.clone()),
                    });
                }
            }
        }
    }
}

fn validate_permitted_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&crate::types::EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    if profile.permitted_categories.is_empty() {
        return;
    }

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if !profile.permitted_categories.contains(&item.category) {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "category_not_permitted".into(),
                message: format!(
                    "'{}' belongs to category '{}' which is not permitted",
                    selection.item_ref, item.category
                ),
                context: Some(selection.item_ref.clone()),
            });
        }
    }
}

fn validate_forbidden_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&crate::types::EntityTypeProfile>,
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
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "forbidden_category".into(),
                message: format!(
                    "'{}' belongs to forbidden category '{}'",
                    selection.item_ref, item.category
                ),
                context: Some(selection.item_ref.clone()),
            });
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
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "wrong_entity_kind".into(),
                message: format!(
                    "'{}' is not valid for entity kind '{:?}'",
                    selection.item_ref, entity.entity_kind
                ),
                context: Some(selection.item_ref.clone()),
            });
        }
    }
}

fn validate_duplicate_selections(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut seen: std::collections::BTreeMap<
        (&Id, &std::collections::BTreeMap<String, Id>),
        usize,
    > = std::collections::BTreeMap::new();

    for selection in &entity.selections {
        let key = (&selection.item_ref, &selection.params);
        *seen.entry(key).or_insert(0) += 1;
    }

    for ((item_ref, _params), count) in &seen {
        if *count <= 1 {
            continue;
        }
        let is_parameterized = ruleset
            .point_items
            .get(item_ref)
            .is_some_and(|item| !item.parameters.is_empty());
        if is_parameterized {
            // For parameterized items, duplicates with identical params are flagged.
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "duplicate_selection".into(),
                message: format!(
                    "'{}' selected {count} times with identical parameters",
                    item_ref
                ),
                context: Some((*item_ref).clone()),
            });
        } else {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "duplicate_selection".into(),
                message: format!("'{}' selected {count} times", item_ref),
                context: Some((*item_ref).clone()),
            });
        }
    }
}

fn validate_required_traits(
    entity: &Entity,
    type_profile: Option<&crate::types::EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let selected_ids: std::collections::BTreeSet<&Id> =
        entity.selections.iter().map(|s| &s.item_ref).collect();

    for required_id in &profile.required_traits {
        if !selected_ids.contains(required_id) {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "missing_required_trait".into(),
                message: format!("required trait '{}' is not selected", required_id),
                context: Some(required_id.clone()),
            });
        }
    }
}

fn validate_forbidden_traits(
    entity: &Entity,
    type_profile: Option<&crate::types::EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let selected_ids: std::collections::BTreeSet<&Id> =
        entity.selections.iter().map(|s| &s.item_ref).collect();

    for forbidden_id in &profile.forbidden_traits {
        if selected_ids.contains(forbidden_id) {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                code: "forbidden_trait".into(),
                message: format!("trait '{}' is forbidden for this type", forbidden_id),
                context: Some(forbidden_id.clone()),
            });
        }
    }
}

fn validate_gift_policy(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&crate::types::EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    use crate::types::GiftPolicy;

    let Some(profile) = type_profile else {
        return;
    };

    let Some(policy) = profile.gift_policy else {
        return;
    };

    let has_gift = match profile.gift_id {
        Some(ref gift_id) => entity.selections.iter().any(|s| s.item_ref == *gift_id),
        None => false,
    };

    let has_gift_category = if profile.gift_categories.is_empty() {
        false
    } else {
        entity.selections.iter().any(|s| {
            ruleset
                .point_items
                .get(&s.item_ref)
                .is_some_and(|item| profile.gift_categories.contains(&item.category))
        })
    };

    match policy {
        GiftPolicy::Required => {
            if !has_gift {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    code: "gift_required".into(),
                    message: "The Gift is required for this entity type".into(),
                    context: None,
                });
            }
        }
        GiftPolicy::Forbidden => {
            if has_gift || has_gift_category {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    code: "gift_forbidden".into(),
                    message: "The Gift is forbidden for this entity type".into(),
                    context: None,
                });
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
            "prerequisites": { "has": "virtue.the_gift" }
          },
          {
            "id": "virtue.gentle_gift",
            "kind": "virtue",
            "magnitude": "major",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "prerequisites": { "has": "virtue.hermetic_magus" },
            "incompatible_with": ["flaw.blatant_gift"]
          },
          {
            "id": "flaw.blatant_gift",
            "kind": "flaw",
            "magnitude": "major",
            "category": "hermetic",
            "entity_kinds": ["character"],
            "prerequisites": { "has": "virtue.the_gift" },
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
            ruleset: RulesetRef {
                id: Id::new("arm5-core"),
                version: "2024.1".into(),
            },
            entity_kind: EntityKind::Character,
            type_id: Id::new(type_id),
            selections,
        }
    }

    fn sel(item_ref: &str) -> Selection {
        Selection {
            item_ref: Id::new(item_ref),
            params: BTreeMap::new(),
        }
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
        // Use a type profile with a small budget (1 pt) so 2 minor virtues exceed it
        // without triggering unrelated errors.
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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"over_budget_virtues"),
            "should flag over budget: {codes:?}"
        );
    }

    #[test]
    fn missing_prerequisite() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.gentle_gift")]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(codes.contains(&"prereq_not_met"), "codes: {codes:?}");
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
        let prereq_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.code == "prereq_not_met")
            .collect();
        assert!(
            prereq_issues.is_empty(),
            "should have no prereq issues: {prereq_issues:?}"
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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(codes.contains(&"incompatible"), "codes: {codes:?}");
    }

    #[test]
    fn cap_exceeded_major_virtues() {
        // Create a type profile that allows hermetic, has max_major_virtues=1,
        // and select 2 different major virtues.
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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"too_many_major_virtues"),
            "codes: {codes:?}"
        );
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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"forbidden_category"),
            "companion can't take hermetic: {codes:?}"
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

        let (v, f) = compute_balance(&entity, &rs);
        assert_eq!(v, 2, "two minor virtues = 2 points");
        assert_eq!(f, 1, "one minor flaw = 1 point");
    }

    #[test]
    fn unknown_selection_ref() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.nonexistent")]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(codes.contains(&"unknown_ref"), "codes: {codes:?}");
    }

    #[test]
    fn entity_save_canonical_roundtrip() {
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.puissant_ability"),
                sel("flaw.poor_student"),
                sel("virtue.keen_vision"),
            ],
        );

        let json1 = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json1).unwrap();

        // Selections are sorted during serialization, so roundtripped
        // selections will be in sorted order.
        let mut expected = entity.clone();
        expected.normalize();
        assert_eq!(expected, roundtripped);

        // Re-serializing the roundtripped entity produces identical JSON.
        let json2 = serde_json::to_string_pretty(&roundtripped).unwrap();
        assert_eq!(json1, json2, "canonical serialization should be stable");
    }

    // --- Finding #1: unknown_type_id ---

    #[test]
    fn unknown_type_id() {
        let rs = test_ruleset();
        let entity = make_entity("nonexistent_type", vec![]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"unknown_type"),
            "should flag unknown type_id: {codes:?}"
        );
    }

    // --- Finding #2: over_budget_flaws ---

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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"over_budget_flaws"),
            "should flag over budget flaws: {codes:?}"
        );
    }

    // --- Finding #3: too_many_major_flaws ---

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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"too_many_major_flaws"),
            "should flag too many major flaws: {codes:?}"
        );
    }

    // --- Finding #4: prereq_all ---

    #[test]
    fn prereq_all_satisfied() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"all": [{"has": "virtue.a"}, {"has": "virtue.b"}]}}
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
            vec![sel("virtue.a"), sel("virtue.b"), sel("virtue.c")],
        );

        let result = validate(&entity, &rs);
        let errors = result.errors();
        let prereq_errors: Vec<_> = errors
            .iter()
            .filter(|i| i.code == "prereq_not_met")
            .collect();
        assert!(
            prereq_errors.is_empty(),
            "all prereqs met: {prereq_errors:?}"
        );
    }

    #[test]
    fn prereq_all_unsatisfied() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"all": [{"has": "virtue.a"}, {"has": "virtue.b"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        // Only virtue.a present, missing virtue.b
        let entity = make_entity("test_type", vec![sel("virtue.a"), sel("virtue.c")]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"prereq_not_met"),
            "All prereq not fully satisfied: {codes:?}"
        );
    }

    // --- Finding #5: prereq_any ---

    #[test]
    fn prereq_any_satisfied() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"any": [{"has": "virtue.a"}, {"has": "virtue.b"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        // Only virtue.a present — Any requires at least one
        let entity = make_entity("test_type", vec![sel("virtue.a"), sel("virtue.c")]);

        let result = validate(&entity, &rs);
        let errors = result.errors();
        let prereq_errors: Vec<_> = errors
            .iter()
            .filter(|i| i.code == "prereq_not_met")
            .collect();
        assert!(
            prereq_errors.is_empty(),
            "any prereq satisfied: {prereq_errors:?}"
        );
    }

    #[test]
    fn prereq_any_unsatisfied() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.c", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"any": [{"has": "virtue.a"}, {"has": "virtue.b"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        // Neither virtue.a nor virtue.b present
        let entity = make_entity("test_type", vec![sel("virtue.c")]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"prereq_not_met"),
            "any prereq not satisfied: {codes:?}"
        );
    }

    // --- Finding #6: prereq_none ---

    #[test]
    fn prereq_none_satisfied() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"none": [{"has": "virtue.a"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        // virtue.a is absent, so None([Has(A)]) is satisfied
        let entity = make_entity("test_type", vec![sel("virtue.b")]);

        let result = validate(&entity, &rs);
        let errors = result.errors();
        let prereq_errors: Vec<_> = errors
            .iter()
            .filter(|i| i.code == "prereq_not_met")
            .collect();
        assert!(
            prereq_errors.is_empty(),
            "none prereq satisfied (A absent): {prereq_errors:?}"
        );
    }

    #[test]
    fn prereq_none_unsatisfied() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"]},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"none": [{"has": "virtue.a"}]}}
        ]"#;
        let types = r#"[{
          "id": "test_type",
          "budget": { "virtue_points": 10, "flaw_points": 10 },
          "permitted_categories": ["general"],
          "creation_phases": []
        }]"#;
        let rs = Ruleset::from_json("test", "1", items, types).unwrap();
        // virtue.a IS present, so None([Has(A)]) fails
        let entity = make_entity("test_type", vec![sel("virtue.a"), sel("virtue.b")]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"prereq_not_met"),
            "none prereq unsatisfied (A present): {codes:?}"
        );
    }

    // --- Finding #7: prereq_house/ability_min/art_min/is_magus produce warnings ---

    #[test]
    fn prereq_house_produces_unevaluated_warning() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"house": "house.bjornaer"}}
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
        let warning_codes: Vec<&str> = result.warnings().iter().map(|i| i.code.as_str()).collect();
        assert!(
            warning_codes.contains(&"prereq_unevaluated"),
            "House prereq should produce unevaluated warning: {warning_codes:?}"
        );
    }

    #[test]
    fn prereq_ability_min_produces_unevaluated_warning() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"ability_min": {"ability": "ability.awareness", "score": 3}}}
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
        let warning_codes: Vec<&str> = result.warnings().iter().map(|i| i.code.as_str()).collect();
        assert!(
            warning_codes.contains(&"prereq_unevaluated"),
            "AbilityMin prereq should produce unevaluated warning: {warning_codes:?}"
        );
    }

    #[test]
    fn prereq_art_min_produces_unevaluated_warning() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": {"art_min": {"art": "art.creo", "score": 5}}}
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
        let warning_codes: Vec<&str> = result.warnings().iter().map(|i| i.code.as_str()).collect();
        assert!(
            warning_codes.contains(&"prereq_unevaluated"),
            "ArtMin prereq should produce unevaluated warning: {warning_codes:?}"
        );
    }

    #[test]
    fn prereq_is_magus_produces_unevaluated_warning() {
        let items = r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"],
           "prerequisites": "is_magus"}
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
        let warning_codes: Vec<&str> = result.warnings().iter().map(|i| i.code.as_str()).collect();
        assert!(
            warning_codes.contains(&"prereq_unevaluated"),
            "IsMagus prereq should produce unevaluated warning: {warning_codes:?}"
        );
    }

    // --- Finding #8: wrong_entity_kind ---

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
        // Covenant entity selects a character-only item
        let mut entity = make_entity("standard_covenant", vec![sel("virtue.char_only")]);
        entity.entity_kind = EntityKind::Covenant;

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"wrong_entity_kind"),
            "covenant selecting character-only item: {codes:?}"
        );
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
        let errors = result.errors();
        let kind_errors: Vec<_> = errors
            .iter()
            .filter(|i| i.code == "wrong_entity_kind")
            .collect();
        assert!(
            kind_errors.is_empty(),
            "empty entity_kinds should be valid for any kind: {kind_errors:?}"
        );
    }

    // --- Finding #9: gift_policy_required ---

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
        // Entity without The Gift
        let entity = make_entity("magus_type", vec![sel("virtue.a")]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"gift_required"),
            "gift required but missing: {codes:?}"
        );
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
        let errors = result.errors();
        let gift_errors: Vec<_> = errors
            .iter()
            .filter(|i| i.code == "gift_required")
            .collect();
        assert!(
            gift_errors.is_empty(),
            "gift present, no error expected: {gift_errors:?}"
        );
    }

    // --- Finding #10: gift_policy_forbidden ---

    #[test]
    fn gift_policy_forbidden_with_gift() {
        let rs = test_ruleset();
        // Companion has gift_policy "forbidden" and gift_id "virtue.the_gift"
        let entity = make_entity("companion", vec![sel("virtue.the_gift")]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"gift_forbidden"),
            "gift forbidden but present: {codes:?}"
        );
    }

    #[test]
    fn gift_policy_required_without_gift_id() {
        // Type profile has gift_policy Required but no gift_id configured.
        // Since gift_id is None, has_gift is always false, so gift_required fires.
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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"gift_required"),
            "gift required but gift_id not set: {codes:?}"
        );
    }

    // --- Finding #11: required_traits ---

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
        // Entity missing the required trait
        let entity = make_entity("strict_type", vec![]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"missing_required_trait"),
            "should flag missing required trait: {codes:?}"
        );
    }

    // --- Finding #12: forbidden_traits ---

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
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"forbidden_trait"),
            "should flag forbidden trait: {codes:?}"
        );
    }

    // --- Finding #13: duplicate_selection_non_parameterized ---

    #[test]
    fn duplicate_selection_non_parameterized() {
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![sel("virtue.keen_vision"), sel("virtue.keen_vision")],
        );

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"duplicate_selection"),
            "two identical non-param selections: {codes:?}"
        );
    }

    // --- Finding #14: duplicate_selection_parameterized ---

    #[test]
    fn duplicate_selection_parameterized() {
        let rs = test_ruleset();
        let param_sel = Selection {
            item_ref: Id::new("virtue.puissant_ability"),
            params: BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
        };
        let entity = make_entity("companion", vec![param_sel.clone(), param_sel]);

        let result = validate(&entity, &rs);
        let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
        assert!(
            codes.contains(&"duplicate_selection"),
            "two param selections with same params: {codes:?}"
        );
    }

    // --- Finding #15: compute_balance_major_and_free ---

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

        let (v, f) = compute_balance(&entity, &rs);
        assert_eq!(v, 3, "one major virtue = 3 pts, one free = 0 pts");
        assert_eq!(f, 1, "one minor flaw = 1 pt");
    }

    // --- Finding #16: compute_balance_empty ---

    #[test]
    fn compute_balance_empty() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![]);

        let (v, f) = compute_balance(&entity, &rs);
        assert_eq!(v, 0);
        assert_eq!(f, 0);
    }

    // --- Finding #17: entity_normalize ---

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

    // --- Finding #18: empty_entity_validates ---

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

    // --- Finding #19: error_message_content ---

    #[test]
    fn error_message_content() {
        let rs = test_ruleset();

        // unknown_ref: message should contain the bad ID
        let entity = make_entity("companion", vec![sel("virtue.nonexistent")]);
        let result = validate(&entity, &rs);
        let errors = result.errors();
        let unknown_ref = errors.iter().find(|i| i.code == "unknown_ref").unwrap();
        assert!(
            unknown_ref.message.contains("virtue.nonexistent"),
            "unknown_ref message should contain the ID: {}",
            unknown_ref.message
        );

        // unknown_type: message should contain the bad type_id
        let entity = make_entity("bogus_type", vec![]);
        let result = validate(&entity, &rs);
        let errors = result.errors();
        let unknown_type = errors.iter().find(|i| i.code == "unknown_type").unwrap();
        assert!(
            unknown_type.message.contains("bogus_type"),
            "unknown_type message should contain the type ID: {}",
            unknown_type.message
        );
    }

    // --- Finding #20: validation_result_errors_vs_warnings ---

    #[test]
    fn validation_result_errors_vs_warnings() {
        let result = ValidationResult {
            issues: vec![
                ValidationIssue {
                    severity: IssueSeverity::Error,
                    code: "err1".into(),
                    message: "an error".into(),
                    context: None,
                },
                ValidationIssue {
                    severity: IssueSeverity::Warning,
                    code: "warn1".into(),
                    message: "a warning".into(),
                    context: None,
                },
                ValidationIssue {
                    severity: IssueSeverity::Error,
                    code: "err2".into(),
                    message: "another error".into(),
                    context: None,
                },
            ],
        };

        assert_eq!(result.errors().len(), 2);
        assert_eq!(result.warnings().len(), 1);
        assert!(!result.is_valid());

        assert!(
            result
                .errors()
                .iter()
                .all(|i| i.severity == IssueSeverity::Error)
        );
        assert!(
            result
                .warnings()
                .iter()
                .all(|i| i.severity == IssueSeverity::Warning)
        );
    }

    // --- Finding #30: apply_mode preserves issue count in Advisory ---

    #[test]
    fn apply_mode_advisory_preserves_issue_count() {
        let rs = test_ruleset();
        let entity = make_entity("companion", vec![sel("virtue.gentle_gift")]);

        let result = validate(&entity, &rs);
        let original_count = result.issues.len();
        assert!(original_count > 0, "should have issues to test with");

        let advisory = validate(&entity, &rs).apply_mode(ValidationMode::Advisory);
        assert_eq!(
            advisory.issues.len(),
            original_count,
            "advisory mode should preserve issue count"
        );
    }
}
