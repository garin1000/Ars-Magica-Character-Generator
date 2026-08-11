//! The life-stage experience plan: age, native language, and its exclusivity with
//! the directly-entered experience pool.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Validates a character built through its life stages.
///
/// Nothing here applies to a directly-entered character (no
/// [`Entity::life_stages`]), which keeps buying Abilities from
/// [`Entity::xp_pool`] exactly as before. With a plan:
///
/// - `life_stage_xp_pool_conflict`: a raw pool alongside the plan. The two are
///   alternative ways of funding the same purchases, so carrying both would let a
///   character spend the derived budget *and* a typed pool.
/// - `life_stage_age_before_childhood`: an age inside the childhood block, which
///   earns childhood's experience but cannot have lived any later-life year.
/// - `life_stage_native_language_unset`: no native language chosen, so the
///   childhood's largest block (75 points) has nothing it may be spent on.
/// - `life_stage_native_language_missing_score`: a native language chosen but no
///   matching Living Language row bought, so those points are unspent.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2378, :2392.
pub(crate) fn validate_life_stage_plan(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(plan) = &entity.life_stages else {
        return;
    };
    let Some(rules) = ruleset.life_stages() else {
        // A plan against a ruleset that ships no life stages: the entity's own
        // `unknown_*` refs are reported elsewhere, and there is no rule here to
        // check it against.
        return;
    };

    if entity.xp_pool > 0 {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_LIFE_STAGE_XP_POOL_CONFLICT,
            CreationPhase::Abilities,
            args([("xp_pool", entity.xp_pool.to_string())]),
            None,
        ));
    }

    if let Some(age) = entity.age
        && age < rules.childhood.years
    {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_LIFE_STAGE_AGE_BEFORE_CHILDHOOD,
            CreationPhase::Abilities,
            args([
                ("age", age.to_string()),
                ("min", rules.childhood.years.to_string()),
            ]),
            None,
        ));
    }

    match &plan.native_language {
        None => issues.push(ValidationIssue::error(
            ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_UNSET,
            CreationPhase::Abilities,
            args([]),
            None,
        )),
        // Suppressed while the language is set but unspent — that is the warning
        // below, not this error.
        Some(language) => {
            // The native language is a Living Language instance, so "bought" means a
            // row for that ability whose parameter value is this language.
            let bought = entity.ability_scores.iter().any(|score| {
                score.parameter.as_deref() == Some(language.as_str())
                    && ruleset
                        .ability(&score.ability)
                        .is_some_and(|ability| ability.parameter.is_some())
            });
            if !bought {
                issues.push(ValidationIssue::warning(
                    ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_MISSING_SCORE,
                    CreationPhase::Abilities,
                    args([("language", language.clone())]),
                    None,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::life_stage::LifeStagePlan;
    use crate::types::{AbilityScore, Entity, EntityKind, Id, RulesetRef};
    use crate::validation::{ValidationIssue, ValidationResult, validate};
    use crate::{CreationPhase, Ruleset, RulesetSources};

    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] }
    ]"#;
    const TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality"], "creation_phases": [] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [ { "score": 1, "total_xp": 5 }, { "score": 5, "total_xp": 75 } ],
      "abilities": [
        { "id": "ability.living_language", "category": "general", "parameter": "language" },
        { "id": "ability.swim", "category": "general" }
      ]
    }"#;
    const LIFE_STAGES: &str = r#"{
      "childhood": {
        "years": 5,
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.swim", "ability.living_language"]
      },
      "later_life": { "xp_per_year": 15 }
    }"#;

    fn rs() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            life_stages: Some(LIFE_STAGES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// A companion built through its life stages, with a native language bought.
    fn planned(age: u32) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.age = Some(age);
        entity.life_stages = Some(LifeStagePlan {
            native_language: Some("German".into()),
        });
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.living_language"),
            parameter: Some("German".into()),
            score: 5,
            specialty: None,
        }];
        entity
    }

    fn codes(result: &ValidationResult) -> Vec<String> {
        result.issues.iter().map(|i| i.code.clone()).collect()
    }

    #[test]
    fn a_well_formed_plan_raises_nothing() {
        let issues = codes(&validate(&planned(25), &rs()));
        assert!(
            !issues.iter().any(|c| c.starts_with("life_stage_")),
            "issues: {issues:?}"
        );
    }

    /// The two funding models are alternatives, so both at once is an error rather
    /// than a silent double budget.
    #[test]
    fn a_plan_beside_a_typed_pool_conflicts() {
        let mut entity = planned(25);
        entity.xp_pool = 120;
        let result = validate(&entity, &rs());
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_LIFE_STAGE_XP_POOL_CONFLICT)
            .expect("the conflict is reported");
        assert_eq!(issue.args.get("xp_pool").map(String::as_str), Some("120"));
        assert_eq!(issue.phase, CreationPhase::Abilities);
    }

    #[test]
    fn an_age_inside_childhood_is_an_error() {
        let result = validate(&planned(3), &rs());
        assert!(
            codes(&result).contains(&ValidationIssue::CODE_LIFE_STAGE_AGE_BEFORE_CHILDHOOD.into()),
            "issues: {:?}",
            codes(&result)
        );
    }

    #[test]
    fn a_plan_without_a_native_language_is_an_error() {
        let mut entity = planned(25);
        entity.life_stages = Some(LifeStagePlan::default());
        let result = validate(&entity, &rs());
        assert!(
            codes(&result).contains(&ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_UNSET.into())
        );
    }

    /// Choosing a language but never buying it leaves 75 points unspendable — worth
    /// saying, but not illegal: a character may be mid-build.
    #[test]
    fn an_unbought_native_language_only_warns() {
        let mut entity = planned(25);
        entity.ability_scores.clear();
        let result = validate(&entity, &rs());
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_MISSING_SCORE)
            .expect("the unspent block is reported");
        assert_eq!(issue.severity, crate::validation::IssueSeverity::Warning);
        assert_eq!(
            issue.args.get("language").map(String::as_str),
            Some("German")
        );
    }

    /// A directly-entered character is untouched by any of this: no plan, no
    /// life-stage findings, and its typed pool stays legal.
    #[test]
    fn direct_entry_is_left_alone() {
        let mut entity = planned(25);
        entity.life_stages = None;
        entity.xp_pool = 120;
        let issues = codes(&validate(&entity, &rs()));
        assert!(
            !issues.iter().any(|c| c.starts_with("life_stage_")),
            "issues: {issues:?}"
        );
    }
}
