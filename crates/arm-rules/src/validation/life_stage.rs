//! The life-stage experience plan: age, native language, and its exclusivity with
//! the directly-entered experience pool.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

use crate::childhood::ChildhoodRejection;

/// Validates a character built through its life stages.
///
/// Nothing here applies to a directly-entered character (no
/// [`Entity::life_stages`]), which keeps buying Abilities from
/// [`Entity::xp_pool`] exactly as before. With a plan:
///
/// - `life_stage_xp_pool_conflict`: a raw pool alongside the plan. The two are
///   alternative ways of funding the same purchases, so carrying both would let a
///   character spend the derived budget *and* a typed pool.
/// - `life_stage_age_unset`: no age at all, so the later-life block — the one
///   counted in years — cannot be earned. Childhood is granted regardless
///   ([`crate::life_stage::LifeStageRules::budget`]), so this is a finding of its
///   own rather than a missing budget.
/// - `life_stage_age_before_childhood`: an age inside the childhood block, which
///   earns childhood's experience but cannot have lived any later-life year.
/// - `life_stage_native_language_unset`: no native language chosen, so the
///   childhood's largest block (75 points) has nothing it may be spent on.
/// - `life_stage_native_language_missing_score`: a native language chosen but no
///   matching Living Language row bought, so those points are unspent.
/// - `childhood_package_unknown`: the recorded Sample Childhood package names an id
///   the loaded ruleset does not ship.
/// - `life_stage_magus_guided_unsupported`: a magus with a plan at all — its
///   experience comes in four periods, of which this engine models two (`:2364`).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2364, :2378, :2392.
pub(crate) fn validate_life_stage_plan(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
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

    // "For grogs and companions they are acquired in two blocks: early childhood,
    // and later life. For magi, there are two more periods to consider:
    // apprenticeship, and life as a magus after that."
    // (Core Rules.md:2364.) The engine models the two blocks, so a magus's later
    // life — which runs only until apprenticeship — cannot be counted to its age
    // here without over-granting, all the more so because Abilities and Arts draw
    // one shared pool. The combination is refused until apprenticeship exists
    // (M6/6b4) rather than computed wrongly; a magus keeps the flat pool, which is
    // fully functional. The guided UI hides the choice, so this catches a
    // hand-edited save.
    if type_profile.is_some_and(|profile| profile.is_magus) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_LIFE_STAGE_MAGUS_GUIDED_UNSUPPORTED,
            CreationPhase::Abilities,
            args([]),
            None,
        ));
    }

    // The later-life block is "15 experience points per year" up to the character's
    // age (Core Rules.md:2392), so an unset age leaves it uncountable — said plainly
    // here rather than left to surface as a shortfall on rows the guided flow may
    // itself have written before an age was typed.
    if entity.age.is_none() {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_LIFE_STAGE_AGE_UNSET,
            CreationPhase::Abilities,
            args([]),
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
            // "75 experience points in their native language" (Core Rules.md:2378)
            // names one Ability — `childhood.native_language_ability` — at one
            // instance, so "bought" is a row for exactly that id whose parameter is
            // this language, scoring above 0. Testing the parameter alone would let
            // an `Area Lore (German)` pass while the 75-point pool, which keys on the
            // id (`native_language_instance` in `effective.rs`), funds none of it.
            let bought = entity.ability_scores.iter().any(|score| {
                score.ability == rules.childhood.native_language_ability
                    && score.parameter.as_deref() == Some(language.as_str())
                    && score.score > 0
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

    // The package a player took is *stored* (`LifeStagePlan::childhood_package`),
    // so a save can name one the loaded ruleset does not ship — a dangling
    // reference like any other, reported rather than quietly ignored. What the
    // package granted is not re-checked: the Abilities are ordinary bought rows
    // (Core Rules.md:2382 keeps a taken package open to adjustment).
    if let Some(package) = &plan.childhood_package
        && ruleset.childhood(package).is_none()
    {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_CHILDHOOD_PACKAGE_UNKNOWN,
            CreationPhase::Abilities,
            args([("package", package.to_string())]),
            Some(package.clone()),
        ));
    }
}

/// Localizes the [`ChildhoodRejection`]s a failed Sample Childhood package
/// application produced.
///
/// These are **command-input** findings, not entity state, which is why they are
/// mapped here on demand instead of being emitted by [`validate`]: applying a
/// package is all-or-nothing (`childhood::apply_package` writes nothing when it
/// rejects), so no stored character can ever *hold* an unanswered or colliding
/// slot for [`validate`] to find. The rejection describes the form the player just
/// submitted, and it is gone as soon as the form is corrected.
///
/// The emit sites live in `validation/` all the same, so the contract-table and
/// phase scanners keep seeing every `(code, phase)` pair the frontend must
/// localize.
///
/// Two variants reuse an existing code rather than inventing one:
/// [`ChildhoodRejection::NativeLanguageUnset`] is exactly the plan's own
/// `life_stage_native_language_unset`, and
/// [`ChildhoodRejection::UnknownPackage`] the stored plan's
/// `childhood_package_unknown` — the same fact, whether it arrives as a command or
/// out of a save.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2378, :2380-2388.
pub fn childhood_rejection_issues(
    rejections: &[ChildhoodRejection],
    ruleset: &Ruleset,
) -> Vec<ValidationIssue> {
    rejections
        .iter()
        .map(|rejection| childhood_rejection_issue(rejection, ruleset))
        .collect()
}

/// One rejection as a localizable issue.
///
/// `slot`/`other_slot` are carried for **field targeting only** — the machine keys
/// a UI highlights the offending input by. The message names the field by its
/// Ability and the `key` below instead, because a slot key is a slug and a slug is
/// never shown to a user.
fn childhood_rejection_issue(rejection: &ChildhoodRejection, ruleset: &Ruleset) -> ValidationIssue {
    match rejection {
        ChildhoodRejection::UnknownPackage { package } => ValidationIssue::error(
            ValidationIssue::CODE_CHILDHOOD_PACKAGE_UNKNOWN,
            CreationPhase::Abilities,
            args([("package", package.to_string())]),
            Some(package.clone()),
        ),
        ChildhoodRejection::NativeLanguageUnset => ValidationIssue::error(
            ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_UNSET,
            CreationPhase::Abilities,
            args([]),
            None,
        ),
        ChildhoodRejection::SlotUnfilled { slot, ability } => ValidationIssue::error(
            ValidationIssue::CODE_CHILDHOOD_SLOT_UNFILLED,
            CreationPhase::Abilities,
            args([
                ("ability", ability.to_string()),
                ("key", parameter_key(ability, ruleset)),
                ("slot", slot.clone()),
            ]),
            Some(ability.clone()),
        ),
        ChildhoodRejection::SlotIsNativeLanguage {
            slot,
            ability,
            language,
        } => ValidationIssue::error(
            ValidationIssue::CODE_CHILDHOOD_SLOT_IS_NATIVE_LANGUAGE,
            CreationPhase::Abilities,
            args([
                ("ability", ability.to_string()),
                ("key", parameter_key(ability, ruleset)),
                ("slot", slot.clone()),
                ("language", language.clone()),
            ]),
            Some(ability.clone()),
        ),
        ChildhoodRejection::DuplicateSlotValue {
            slot,
            other_slot,
            ability,
            value,
        } => ValidationIssue::error(
            ValidationIssue::CODE_CHILDHOOD_SLOT_DUPLICATE_VALUE,
            CreationPhase::Abilities,
            args([
                ("ability", ability.to_string()),
                ("key", parameter_key(ability, ruleset)),
                ("slot", slot.clone()),
                ("other_slot", other_slot.clone()),
                ("value", value.clone()),
            ]),
            Some(ability.clone()),
        ),
    }
}

/// The `parameter` key the slot's Ability asks for — `area`, `language` — which the
/// UI turns into a field label (`param-label-<key>`). The rejection carries only
/// the Ability id, so the key is looked up rather than guessed from the slot name.
///
/// Empty only for an Ability the ruleset does not know or does not parameterize,
/// which load-time integrity already rules out for every slotted package entry
/// (`Ruleset::validate_childhood_packages`): the message then names the Ability
/// alone rather than printing a slug.
fn parameter_key(ability: &Id, ruleset: &Ruleset) -> String {
    ruleset
        .ability(ability)
        .and_then(|known| known.parameter.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::childhood::ChildhoodRejection;
    use crate::life_stage::LifeStagePlan;
    use crate::types::{AbilityScore, Entity, EntityKind, Id, RulesetRef};
    use crate::validation::{
        IssueSeverity, ValidationIssue, ValidationResult, childhood_rejection_issues, validate,
    };
    use crate::{CreationPhase, Ruleset, RulesetSources};

    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] }
    ]"#;
    const TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality"], "creation_phases": [] },
      { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality"], "is_magus": true,
        "creation_phases": [] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 },
        { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 },
        { "score": 5, "total_xp": 75 }
      ],
      "categories_requiring_virtue": ["academic", "arcane", "martial"],
      "abilities": [
        { "id": "ability.area_lore", "category": "general", "parameter": "area" },
        { "id": "ability.living_language", "category": "general", "parameter": "language" },
        { "id": "ability.swim", "category": "general" },
        { "id": "ability.artes_liberales", "category": "academic" },
        { "id": "ability.philosophiae", "category": "academic" },
        { "id": "ability.magic_theory", "category": "arcane" },
        { "id": "ability.parma_magica", "category": "arcane" },
        { "id": "ability.penetration", "category": "arcane" }
      ]
    }"#;
    /// The apprenticeship block carries **empty** requirement lists: the load-time
    /// gate only demands that a magus ruleset declare the block, and the abilities
    /// const above ships no dead language, so filling the lists here would drag an
    /// unrelated catalogue change into these tests.
    const LIFE_STAGES: &str = r#"{
      "apprenticeship": {
        "years": 15,
        "xp": 240,
        "minimum_abilities": [],
        "recommended_abilities": [],
        "recommended_xp": 0
      },
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.swim", "ability.living_language"]
      },
      "later_life": { "xp_per_year": 15 }
    }"#;
    /// One shipped package, priced to the blocks above: spread 30 + 15 = 45,
    /// native language 75 (Core Rules.md:2378).
    const CHILDHOODS: &str = r#"{ "packages": [
      { "id": "childhood.shipped",
        "entries": [
          { "ability": "ability.living_language", "score": 5, "native": true },
          { "ability": "ability.living_language", "score": 2, "slot": "language" },
          { "ability": "ability.swim", "score": 3 }
        ] }
    ] }"#;

    fn rs() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            life_stages: Some(LIFE_STAGES),
            childhoods: Some(CHILDHOODS),
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
            ..LifeStagePlan::default()
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

    /// Later life is counted in years up to an age (Core Rules.md:2392), so a plan
    /// with no age has no later-life block to earn. That is worth saying out loud:
    /// the guided flow can write childhood rows before an age is typed, and without
    /// this the only symptom would be a shortfall the player did not cause.
    #[test]
    fn a_plan_without_an_age_is_an_error() {
        let mut entity = planned(25);
        entity.age = None;
        let result = validate(&entity, &rs());
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_LIFE_STAGE_AGE_UNSET)
            .expect("the missing age is reported");
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.phase, CreationPhase::Abilities);
        assert!(issue.args.is_empty(), "args: {:?}", issue.args);

        // With an age it is silent, and a direct-entry character never sees it.
        assert!(
            !codes(&validate(&planned(25), &rs())).contains(&"life_stage_age_unset".to_string())
        );
        let mut direct = planned(25);
        direct.life_stages = None;
        direct.age = None;
        assert!(!codes(&validate(&direct, &rs())).contains(&"life_stage_age_unset".to_string()));
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

    /// "75 experience points in their native language" (Core Rules.md:2378) names
    /// **one** Ability: the childhood's `native_language_ability`. An Area Lore
    /// whose area happens to be spelled like the language is a different Ability,
    /// and the 75-point pool — which keys on that id — cannot fund a point of it, so
    /// it must not silence the warning. The validator and the pool have to read the
    /// same rule the same way.
    #[test]
    fn an_area_lore_named_after_the_language_does_not_satisfy_the_native_language() {
        let mut entity = planned(25);
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.area_lore"),
            parameter: Some("German".into()),
            score: 2,
            specialty: None,
        }];

        let result = validate(&entity, &rs());
        assert!(
            codes(&result)
                .contains(&ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_MISSING_SCORE.into()),
            "issues: {:?}",
            codes(&result)
        );
    }

    /// A row at 0 buys nothing, so it leaves the 75 points as unspent as no row at
    /// all — the whole point of the warning.
    #[test]
    fn a_native_language_row_at_zero_does_not_satisfy_the_native_language() {
        let mut entity = planned(25);
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.living_language"),
            parameter: Some("German".into()),
            score: 0,
            specialty: None,
        }];

        let result = validate(&entity, &rs());
        assert!(
            codes(&result)
                .contains(&ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_MISSING_SCORE.into()),
            "issues: {:?}",
            codes(&result)
        );
    }

    /// The taken package is persisted, so a save can name one the loaded ruleset
    /// does not ship — a dangling reference, reported like any other. A package the
    /// ruleset does ship is silent.
    #[test]
    fn a_stored_unknown_childhood_package_is_an_error() {
        let mut entity = planned(25);
        entity.life_stages = Some(LifeStagePlan {
            native_language: Some("German".into()),
            childhood_package: Some(Id::new("childhood.nonesuch")),
        });

        let result = validate(&entity, &rs());
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_CHILDHOOD_PACKAGE_UNKNOWN)
            .expect("the unknown package is reported");
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.phase, CreationPhase::Abilities);
        assert_eq!(
            issue.args.get("package").map(String::as_str),
            Some("childhood.nonesuch")
        );
        assert_eq!(issue.context, Some(Id::new("childhood.nonesuch")));

        entity.life_stages = Some(LifeStagePlan {
            native_language: Some("German".into()),
            childhood_package: Some(Id::new("childhood.shipped")),
        });
        assert!(
            !codes(&validate(&entity, &rs()))
                .contains(&ValidationIssue::CODE_CHILDHOOD_PACKAGE_UNKNOWN.into())
        );
    }

    /// A magus does not earn its experience in two blocks: "For grogs and companions
    /// they are acquired in two blocks: early childhood, and later life. For magi,
    /// there are two more periods to consider: apprenticeship, and life as a magus
    /// after that." (Core Rules.md:2364). `later_life_years` counts every year after
    /// childhood, so for a magus it swallows apprenticeship and life as a magus —
    /// and since Abilities and Arts draw one shared pool, those years would fund
    /// Arts too. Until the two further periods are modelled (M6/6b4) the combination
    /// is refused rather than computed.
    #[test]
    fn a_magus_may_not_be_built_through_its_life_stages_yet() {
        let mut entity = planned(25);
        entity.type_id = Id::new("magus");

        let result = validate(&entity, &rs());
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == ValidationIssue::CODE_LIFE_STAGE_MAGUS_GUIDED_UNSUPPORTED)
            .expect("a magus carrying a life-stage plan is refused");
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.phase, CreationPhase::Abilities);
        assert!(issue.args.is_empty(), "args: {:?}", issue.args);

        // A companion is exactly the case the two blocks describe, and a magus with
        // no plan keeps the flat pool it always had.
        let unaffected = |entity: &Entity| {
            assert!(
                !codes(&validate(entity, &rs()))
                    .contains(&"life_stage_magus_guided_unsupported".to_string())
            );
        };
        unaffected(&planned(25));
        let mut magus = planned(25);
        magus.type_id = Id::new("magus");
        magus.life_stages = None;
        magus.xp_pool = 240;
        unaffected(&magus);
    }

    /// Childhood grants two separately-restricted blocks (Core Rules.md:2378), so a
    /// life-stage character can leave both unspent and would otherwise receive two
    /// warnings a reader cannot tell apart. Each names its own block, and the
    /// unspent-45 case — the common one, since the 75 buys one language — says
    /// `childhood_spread` rather than merely "restricted".
    #[test]
    fn an_unspent_life_stage_block_names_which_block_it_is() {
        let origins = |entity: &Entity| -> Vec<(String, String)> {
            validate(entity, &rs())
                .issues
                .iter()
                .filter(|i| i.code == ValidationIssue::CODE_RESTRICTED_XP_UNSPENT)
                .map(|i| {
                    (
                        i.args.get("origin_kind").cloned().unwrap_or_default(),
                        i.args.get("origin").cloned().unwrap_or_default(),
                    )
                })
                .collect()
        };

        // The bought German 5 spends the whole 75; only the spread is left.
        assert_eq!(
            origins(&planned(25)),
            vec![("life_stage".to_string(), "childhood_spread".to_string())]
        );

        // Nothing bought: both blocks are unspent, and the two warnings are
        // distinguishable.
        let mut entity = planned(25);
        entity.ability_scores.clear();
        assert_eq!(
            origins(&entity),
            vec![
                (
                    "life_stage".to_string(),
                    "childhood_native_language".to_string()
                ),
                ("life_stage".to_string(), "childhood_spread".to_string()),
            ]
        );
    }

    /// A magus's pre-apprenticeship experience is a pool of its own, so leaving it
    /// unspent is reported like any other wasted block — and **names the block**,
    /// because a magus now has three of them and three warnings differing only in
    /// their numbers say nothing about where to go and spend.
    ///
    /// A **regression lock**: this falls out of `origin_args` reading
    /// `LifeStageBlock`'s `Display`, which is precisely why the pool carries an origin
    /// at all.
    #[test]
    fn unspent_pre_apprenticeship_experience_names_the_later_life_block() {
        let mut magus = planned(25);
        magus.type_id = Id::new("magus");
        // Academic, so only apprenticeship's 240 may fund it — the 75 of later life
        // stays untouched.
        magus.ability_scores.push(AbilityScore {
            ability: Id::new("ability.artes_liberales"),
            parameter: None,
            score: 3,
            specialty: None,
        });

        let origins: Vec<(String, String)> = validate(&magus, &rs())
            .issues
            .iter()
            .filter(|i| i.code == ValidationIssue::CODE_RESTRICTED_XP_UNSPENT)
            .map(|i| {
                (
                    i.args.get("origin_kind").cloned().unwrap_or_default(),
                    i.args.get("origin").cloned().unwrap_or_default(),
                )
            })
            .collect();
        assert!(
            origins.contains(&("life_stage".to_string(), "later_life".to_string())),
            "origins: {origins:?}"
        );
    }

    /// A rejected package application describes **command input**, not entity
    /// state, so `validate()` never produces these codes: they are localized here
    /// instead. Each carries what a message needs — the Ability, the `parameter`
    /// key its label is drawn from (`area` for an Area Lore slot) — plus the
    /// machine slot key the UI highlights but never prints.
    #[test]
    fn childhood_rejections_map_to_localizable_issues() {
        let issues = childhood_rejection_issues(
            &[
                ChildhoodRejection::UnknownPackage {
                    package: Id::new("childhood.nonesuch"),
                },
                ChildhoodRejection::NativeLanguageUnset,
                ChildhoodRejection::SlotUnfilled {
                    slot: "area_b".into(),
                    ability: Id::new("ability.area_lore"),
                },
                ChildhoodRejection::SlotIsNativeLanguage {
                    slot: "language".into(),
                    ability: Id::new("ability.living_language"),
                    language: "German".into(),
                },
                ChildhoodRejection::DuplicateSlotValue {
                    slot: "area_b".into(),
                    other_slot: "area_a".into(),
                    ability: Id::new("ability.area_lore"),
                    value: "Bavaria".into(),
                },
            ],
            &rs(),
        );

        let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
        assert_eq!(
            codes,
            vec![
                ValidationIssue::CODE_CHILDHOOD_PACKAGE_UNKNOWN,
                // A missing native language is the plan's own finding, so it reuses
                // the code `validate()` already emits for it.
                ValidationIssue::CODE_LIFE_STAGE_NATIVE_LANGUAGE_UNSET,
                ValidationIssue::CODE_CHILDHOOD_SLOT_UNFILLED,
                ValidationIssue::CODE_CHILDHOOD_SLOT_IS_NATIVE_LANGUAGE,
                ValidationIssue::CODE_CHILDHOOD_SLOT_DUPLICATE_VALUE,
            ]
        );
        assert!(
            issues
                .iter()
                .all(|i| i.severity == IssueSeverity::Error && i.phase == CreationPhase::Abilities),
            "every rejection is a blocking abilities-phase error: {issues:?}"
        );

        let keys =
            |issue: &ValidationIssue| -> Vec<String> { issue.args.keys().cloned().collect() };

        assert_eq!(keys(&issues[0]), vec!["package"]);
        assert_eq!(
            issues[0].args.get("package").map(String::as_str),
            Some("childhood.nonesuch")
        );

        assert!(issues[1].args.is_empty());

        assert_eq!(keys(&issues[2]), vec!["ability", "key", "slot"]);
        assert_eq!(
            issues[2].args.get("ability").map(String::as_str),
            Some("ability.area_lore")
        );
        assert_eq!(issues[2].args.get("key").map(String::as_str), Some("area"));
        assert_eq!(
            issues[2].args.get("slot").map(String::as_str),
            Some("area_b")
        );
        assert_eq!(issues[2].context, Some(Id::new("ability.area_lore")));

        assert_eq!(keys(&issues[3]), vec!["ability", "key", "language", "slot"]);
        assert_eq!(
            issues[3].args.get("key").map(String::as_str),
            Some("language")
        );
        assert_eq!(
            issues[3].args.get("language").map(String::as_str),
            Some("German")
        );

        assert_eq!(
            keys(&issues[4]),
            vec!["ability", "key", "other_slot", "slot", "value"]
        );
        assert_eq!(issues[4].args.get("key").map(String::as_str), Some("area"));
        assert_eq!(
            issues[4].args.get("other_slot").map(String::as_str),
            Some("area_a")
        );
        assert_eq!(
            issues[4].args.get("value").map(String::as_str),
            Some("Bavaria")
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
