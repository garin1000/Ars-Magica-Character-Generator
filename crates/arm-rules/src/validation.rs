use crate::ruleset::Ruleset;
use crate::types::{Entity, Id, ItemKind, Magnitude, Prereq, ValidationMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub context: Option<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub fn validate(entity: &Entity, ruleset: &Ruleset) -> ValidationResult {
    let mut issues = Vec::new();

    let type_profile = ruleset.character_types.get(&entity.type_id);

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

    validate_balance(entity, ruleset, type_profile, &mut issues);
    validate_caps(entity, ruleset, type_profile, &mut issues);
    validate_prerequisites(entity, ruleset, &mut issues);
    validate_incompatibilities(entity, ruleset, &mut issues);
    validate_forbidden_categories(entity, ruleset, type_profile, &mut issues);

    ValidationResult { issues }
}

fn validate_balance(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&crate::types::CharacterTypeProfile>,
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
    type_profile: Option<&crate::types::CharacterTypeProfile>,
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
            && !evaluate_prereq(prereq, &selected_ids, entity)
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

fn evaluate_prereq(prereq: &Prereq, selected_ids: &[&Id], _entity: &Entity) -> bool {
    match prereq {
        Prereq::All(children) => children
            .iter()
            .all(|c| evaluate_prereq(c, selected_ids, _entity)),
        Prereq::Any(children) => children
            .iter()
            .any(|c| evaluate_prereq(c, selected_ids, _entity)),
        Prereq::None(children) => !children
            .iter()
            .any(|c| evaluate_prereq(c, selected_ids, _entity)),
        Prereq::Has(id) => selected_ids.contains(&id),
        Prereq::House(_) => false,
        Prereq::AbilityMin { .. } => false,
        Prereq::ArtMin { .. } => false,
        Prereq::IsMagus => false,
    }
}

fn validate_incompatibilities(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let selected_ids: Vec<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        for incompat_id in &item.incompatible_with {
            if selected_ids.contains(&incompat_id) {
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

fn validate_forbidden_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&crate::types::CharacterTypeProfile>,
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
            "creation_phases": ["concept", "virtues_flaws", "abilities"]
          }
        ]"#;

        Ruleset::from_json("arm5-core", "2024.1", items, types).unwrap()
    }

    fn make_entity(type_id: &str, selections: Vec<Selection>) -> Entity {
        Entity {
            schema_version: 1,
            ruleset: RulesetRef {
                id: "arm5-core".into(),
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
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.gentle_gift"),
                sel("virtue.hermetic_magus"),
                sel("virtue.the_gift"),
                sel("virtue.keen_vision"),
                sel("virtue.large"),
                sel("virtue.tough"),
                sel("virtue.puissant_ability"),
                sel("virtue.puissant_ability"),
                sel("virtue.puissant_ability"),
                sel("virtue.puissant_ability"),
                sel("virtue.puissant_ability"),
                sel("virtue.puissant_ability"),
                sel("virtue.puissant_ability"),
                sel("virtue.puissant_ability"),
            ],
        );

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
        let rs = test_ruleset();
        let entity = make_entity(
            "companion",
            vec![
                sel("virtue.the_gift"),
                sel("virtue.hermetic_magus"),
                sel("virtue.gentle_gift"),
                sel("virtue.gentle_gift"),
            ],
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

        let json = serde_json::to_string_pretty(&entity).unwrap();
        let roundtripped: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity, roundtripped);
    }
}
