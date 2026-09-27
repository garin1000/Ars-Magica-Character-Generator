//! Which Ability categories a character is *allowed* to buy at creation.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;
use crate::ability::AbilityCategory;

/// Validates that every bought Ability belongs to a category the character is
/// permitted to learn.
///
/// > There are two exceptions. One is that a character must have a Virtue to buy
/// > Academic, Arcane, Martial, or Supernatural Abilities at character creation.
/// > Educated, Arcane Lore, and Warrior, respectively, are the easiest options for
/// > the first three groups, although other Virtues (and some Flaws) also grant
/// > access to some of these Abilities.
///
/// Source: ArMDE:2315, restated for the
/// later-life block at `ArMDE:2392`.
///
/// **Supernatural is not checked here.** It has its own, stricter rule — access is
/// granted per Ability rather than per category (`ArMDE:2315`: "In most cases, access to
/// each Supernatural Ability is granted by a separate Virtue") — which
/// [`validate_supernatural_abilities`] already enforces. Which categories this
/// function gates is therefore rules *data*
/// (`abilities.json` → `categories_requiring_virtue`), not a hardcoded list.
///
/// **The Hermetically trained are exempt**, per `ArMDE:7151` ("Beginning characters
/// may only purchase Academic Abilities if they are specifically permitted to
/// through the purchase of a Virtue, **or if they are magi**") and `ArMDE:2435`,
/// where apprenticeship experience may be spent on "Arcane, Academic, and Martial
/// Abilities". The exemption is read off `is_hermetically_trained` (D56/A0's union
/// of the profile flag with any selection carrying
/// [`Effect::ConfersHermeticTraining`], e.g. the Abandoned Apprentice Flaw), never a
/// type id — a stray Arcane Ability on a companion with no such Virtue and no such
/// selection is still gated.
///
/// The exemption is whole-character on purpose, and the other half of `ArMDE:2435` —
/// "magi can only spend experience points on Arcane, Academic and Martial Abilities
/// **before** apprenticeship if they have a Virtue which allows them to do so" — is
/// enforced elsewhere and is **not** unenforced: for a guided magus it is a property
/// of the money, not of the Ability, so it lives in the pool
/// ([`crate::effective::xp_allocation`] builds later life as an Abilities-only pool
/// that excludes the gated categories). A magus may *own* these Abilities; a
/// shortfall there is `not_enough_xp`, never `ability_category_requires_virtue`. The
/// two mechanisms are disjoint, which is what keeps a single purchase from being
/// reported twice.
///
/// A Virtue grants access two ways, and both count: an explicit
/// [`Effect::AbilityAuthorization`], or any [`Effect::RestrictedAbilityXp`] pool —
/// experience earmarked for a category is evidence the category is permitted, which
/// is what makes Warrior (Martial XP) and Arcane Lore (Arcane XP) work without
/// further data.
pub(crate) fn validate_ability_authorization(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let gated = ruleset.categories_requiring_virtue();
    if gated.is_empty() {
        return;
    }
    // A Hermetically trained entity may buy the restricted categories with no
    // further Virtue.
    if crate::effective::is_hermetically_trained(entity, ruleset, type_profile) {
        return;
    }

    let (authorized_abilities, authorized_categories) =
        crate::effective::ability_authorizations(entity, ruleset);

    for entry in &entity.ability_scores {
        let Some(ability) = ruleset.ability(&entry.ability) else {
            continue; // `unknown_ability` covers this.
        };
        if !gated.contains(&ability.category) {
            continue;
        }
        if authorized_categories.contains(&ability.category)
            || crate::effective::authorizes_instance(
                &authorized_abilities,
                &entry.ability,
                entry.parameter.as_ref(),
            )
        {
            continue;
        }
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE,
            CreationPhase::Abilities,
            // `ability_category`, not `category`: the latter names the Virtue/Flaw
            // grouping category in `category_not_permitted`/`forbidden_category`
            // (`selections.rs`), a disjoint taxonomy with its own label family. One
            // arg name for both taxonomies left this side unlabelled in the UI.
            args([
                ("ability", entry.ability.to_string()),
                ("ability_category", ability.category.to_string()),
            ]),
            Some(entry.ability.clone()),
        ));
    }
}

/// Warns when an Academic Ability is bought without a scholarly language at the
/// score the rules expect.
///
/// > In addition, learning an Academic Knowledge normally requires a Latin, Greek,
/// > Hebrew, or Arabic score of at least 3, depending on the region of Europe you
/// > are from. For most characters, Latin 3 is required.
///
/// Source: ArMDE:7151.
///
/// A **warning**, not an error, for two reasons the passage itself gives: "normally"
/// and "depending on the region", so a saga may legitimately differ; and which
/// languages qualify is regional, so the data names the ability
/// (`scholarly_language_ability`) and minimum score rather than enumerating Latin,
/// Greek, Hebrew and Arabic — any instance of that ability at the minimum satisfies
/// it, which is as close as the engine can get without deciding a saga's region.
pub(crate) fn validate_academic_language(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(requirement) = ruleset.scholarly_language_requirement() else {
        return;
    };
    let has_academic = entity.ability_scores.iter().any(|entry| {
        ruleset
            .ability(&entry.ability)
            .is_some_and(|ability| ability.category == AbilityCategory::Academic)
            && entry.ability != requirement.ability
    });
    if !has_academic {
        return;
    }
    let satisfied = entity
        .ability_scores
        .iter()
        .any(|entry| entry.ability == requirement.ability && entry.score >= requirement.min_score);
    if !satisfied {
        let mut args = args([
            ("ability", requirement.ability.to_string()),
            ("min", requirement.min_score.to_string()),
        ]);
        // "For most characters, Latin 3 is required" (`ArMDE:7151`): the check stays any
        // instance of the dead language, but the finding names the rules' exemplar.
        if let Some(exemplar) = &requirement.exemplar {
            args.insert("exemplar".to_string(), exemplar.clone());
        }
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_ACADEMIC_ABILITY_WITHOUT_SCHOLARLY_LANGUAGE,
            CreationPhase::Abilities,
            args,
            None,
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{
        AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
    };
    use crate::validation::{ValidationIssue, ValidationResult, validate};
    use crate::{CreationPhase, Ruleset, RulesetSources};

    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.warrior", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "restricted_ability_xp", "amount": 50, "categories": ["martial"] }] },
      { "id": "virtue.covenant_upbringing", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [{ "type": "ability_authorization",
          "abilities": [{ "ability": "ability.dead_language", "instance": { "literal": "latin" } }] }] },
      { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative",
        "magnitude": "free", "categories": ["special"], "entity_kinds": ["character"] },
      { "id": "virtue.hermetic_magus", "kind": "virtue", "classification": "narrative",
        "magnitude": "free", "categories": ["social_status"], "entity_kinds": ["character"] },
      { "id": "flaw.test_confers_training", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"],
        "effects": [{ "type": "confers_hermetic_training" }] },
      { "id": "virtue.wise_one", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"],
        "parameters": [{ "key": "study", "type": "ref", "domain": "enumerated",
          "values": ["academic", "arcane"] }],
        "effects": [{ "type": "ability_authorization",
          "categories": [
            { "category": "academic", "gate": { "param": "study", "equals": "academic" } },
            { "category": "arcane", "gate": { "param": "study", "equals": "arcane" } }
          ] }] },
      { "id": "virtue.student_of_realm", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [{ "key": "realm", "type": "ref", "domain": "realm" }],
        "effects": [{ "type": "ability_bonus_gated", "amount": 2,
          "targets": [
            { "ability": "ability.magic_lore", "gate": { "param": "realm", "equals": "realm.magic" } },
            { "ability": "ability.infernal_lore", "gate": { "param": "realm", "equals": "realm.infernal" } }
          ] }] }
    ]"#;
    const TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality", "social_status", "special"],
        "creation_phases": [] },
      { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality", "social_status", "special"],
        "hermetically_trained": true, "order_member": true, "creation_phases": [] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [ { "score": 1, "total_xp": 5 }, { "score": 3, "total_xp": 30 } ],
      "categories_requiring_virtue": ["academic", "arcane", "martial"],
      "abilities": [
        { "id": "ability.awareness", "category": "general" },
        { "id": "ability.single_weapon", "category": "martial" },
        { "id": "ability.magic_theory", "category": "arcane" },
        { "id": "ability.artes_liberales", "category": "academic" },
        { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
        { "id": "ability.parma_magica", "category": "arcane" },
        { "id": "ability.penetration", "category": "arcane" },
        { "id": "ability.philosophiae", "category": "academic" },
        { "id": "ability.magic_lore", "category": "arcane" },
        { "id": "ability.infernal_lore", "category": "arcane" }
      ]
    }"#;

    fn rs() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// A companion/magus with one parameterized ability score, for the
    /// instance-restriction tests below (the plain [`character`] helper always
    /// leaves `parameter: None`).
    /// `parameter` is wrapped as `Catalogued`, not `Text`: this helper backs
    /// the instance-restriction gate tests below, which name a
    /// `ParamValue::Literal` (design § 4 rule 1) — satisfied ONLY by a
    /// `Catalogued` id match, never by `Text` holding the identical letters.
    fn character_with_parameterized_ability(
        type_id: &str,
        selections: Vec<&str>,
        ability: &str,
        score: u8,
        parameter: Option<&str>,
    ) -> Entity {
        let mut entity = character(type_id, selections, vec![]);
        entity.ability_scores.push(AbilityScore {
            ability: Id::new(ability),
            parameter: parameter.map(|id| AbilityParameterValue::Catalogued { id: Id::new(id) }),
            score,
            specialty: None,
        });
        entity
    }

    /// A companion/magus holding ONE selection with a single parameter value
    /// set (Wise One's `study`, Student of (Realm)'s `realm`), for the
    /// exclusive-choice gate tests below.
    fn character_with_param(
        type_id: &str,
        item_ref: &str,
        param_key: &str,
        param_value: &str,
        abilities: Vec<(&str, u8)>,
    ) -> Entity {
        let mut entity = character(type_id, vec![], abilities);
        entity.selections = vec![Selection::with_params(
            Id::new(item_ref),
            std::collections::BTreeMap::from([(param_key.to_string(), Id::new(param_value))]),
        )];
        entity
    }

    fn character(type_id: &str, selections: Vec<&str>, abilities: Vec<(&str, u8)>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new(type_id),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.selections = selections
            .into_iter()
            .map(|r| Selection::new(Id::new(r)))
            .collect();
        entity.ability_scores = abilities
            .into_iter()
            .map(|(id, score)| AbilityScore {
                ability: Id::new(id),
                parameter: None,
                score,
                specialty: None,
            })
            .collect();
        entity.xp_pool = 500;
        entity
    }

    fn codes(result: &ValidationResult) -> Vec<String> {
        result.issues.iter().map(|i| i.code.clone()).collect()
    }

    fn gate_issues(result: &ValidationResult) -> Vec<&ValidationIssue> {
        result
            .issues
            .iter()
            .filter(|i| i.code == ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE)
            .collect()
    }

    /// A General Ability needs no Virtue.
    #[test]
    fn a_general_ability_needs_no_virtue() {
        let entity = character("companion", vec![], vec![("ability.awareness", 3)]);
        assert!(gate_issues(&validate(&entity, &rs())).is_empty());
    }

    /// "a character must have a Virtue to buy Academic, Arcane, Martial … Abilities"
    /// (ArMDE:2315).
    ///
    /// The category arrives under `ability_category`, NOT `category`: the latter is
    /// the Virtue/Flaw grouping category `category_not_permitted` and
    /// `forbidden_category` emit (`selections.rs`), a disjoint taxonomy the UI
    /// labels through a different Fluent family. One arg name serving both meant
    /// the Ability side resolved against the V/F labels and printed its own slug.
    #[test]
    fn a_martial_ability_without_a_virtue_is_an_error() {
        let entity = character("companion", vec![], vec![("ability.single_weapon", 3)]);
        let result = validate(&entity, &rs());
        let issue = gate_issues(&result).first().copied().expect("gated");
        assert_eq!(
            issue.args.get("ability_category").map(String::as_str),
            Some("martial")
        );
        assert_eq!(issue.args.get("category"), None);
        assert_eq!(issue.phase, CreationPhase::Abilities);
    }

    /// Warrior earmarks Martial experience, which is itself permission — otherwise
    /// its own grant could never be spent.
    #[test]
    fn an_xp_granting_virtue_authorizes_its_category() {
        let entity = character(
            "companion",
            vec!["virtue.warrior"],
            vec![("ability.single_weapon", 3)],
        );
        assert!(gate_issues(&validate(&entity, &rs())).is_empty());
    }

    /// …but only its own category: Warrior does not open Arcane.
    #[test]
    fn a_virtue_authorizes_only_what_it_names() {
        let entity = character(
            "companion",
            vec!["virtue.warrior"],
            vec![("ability.magic_theory", 3)],
        );
        assert_eq!(gate_issues(&validate(&entity, &rs())).len(), 1);
    }

    /// A Virtue may permit an Ability without granting experience for it, which is
    /// what `ability_authorization` is for.
    #[test]
    fn an_authorizing_virtue_permits_one_ability_without_granting_xp() {
        let entity = character_with_parameterized_ability(
            "companion",
            vec!["virtue.covenant_upbringing"],
            "ability.dead_language",
            3,
            Some("latin"),
        );
        assert!(gate_issues(&validate(&entity, &rs())).is_empty());
        // It permits that Ability alone, not the whole Academic category.
        let broader = character(
            "companion",
            vec!["virtue.covenant_upbringing"],
            vec![("ability.artes_liberales", 3)],
        );
        assert_eq!(gate_issues(&validate(&broader, &rs())).len(), 1);
    }

    /// F-349/F-16x's actual fix (`docs/vf-audit/design-c0-parameter-model.md`
    /// § 1, § 4): "You may take Latin at character creation" (`ArMDE:5867`)
    /// authorizes Latin, not any dead language — the id-only proxy silently
    /// over-permitted every instance of `ability.dead_language`. Once the
    /// effect carries an `instance` restriction, Latin is still authorized but
    /// a different dead language must be gated exactly like any other
    /// Academic Ability.
    #[test]
    fn an_authorizing_virtue_with_an_instance_restriction_refuses_a_different_instance() {
        let latin = character_with_parameterized_ability(
            "companion",
            vec!["virtue.covenant_upbringing"],
            "ability.dead_language",
            3,
            Some("latin"),
        );
        assert!(
            gate_issues(&validate(&latin, &rs())).is_empty(),
            "issues: {:?}",
            codes(&validate(&latin, &rs()))
        );

        let greek = character_with_parameterized_ability(
            "companion",
            vec!["virtue.covenant_upbringing"],
            "ability.dead_language",
            3,
            Some("greek"),
        );
        assert_eq!(gate_issues(&validate(&greek, &rs())).len(), 1);
    }

    /// W2/F-349's exclusive-choice gate (`ArMDE:5259`: "You may take either
    /// Arcane or Academic Abilities, but not both, at character creation").
    /// The naive fix — authorizing both categories unconditionally — is
    /// exactly the over-permission F-349 warns against: choosing Academic
    /// must still refuse an Arcane Ability.
    #[test]
    fn wise_one_with_one_study_choice_refuses_the_other() {
        let academic = character_with_param(
            "companion",
            "virtue.wise_one",
            "study",
            "academic",
            vec![("ability.artes_liberales", 3)],
        );
        assert!(
            gate_issues(&validate(&academic, &rs())).is_empty(),
            "issues: {:?}",
            codes(&validate(&academic, &rs()))
        );

        // Academic study must NOT also authorize Arcane Abilities.
        let wrong = character_with_param(
            "companion",
            "virtue.wise_one",
            "study",
            "academic",
            vec![("ability.magic_theory", 3)],
        );
        assert_eq!(gate_issues(&validate(&wrong, &rs())).len(), 1);
    }

    /// Student of (Realm)'s gated `ability_bonus_gated` targets double as
    /// authorization (row 50(a), "even if you cannot learn other Arcane
    /// Abilities") — but ONLY for the chosen realm's own Lore. Student of
    /// Magic must refuse Infernal Lore.
    #[test]
    fn student_of_realm_authorizes_only_the_chosen_realms_lore() {
        let magic = character_with_param(
            "companion",
            "virtue.student_of_realm",
            "realm",
            "realm.magic",
            vec![("ability.magic_lore", 3)],
        );
        assert!(
            gate_issues(&validate(&magic, &rs())).is_empty(),
            "issues: {:?}",
            codes(&validate(&magic, &rs()))
        );

        let wrong = character_with_param(
            "companion",
            "virtue.student_of_realm",
            "realm",
            "realm.magic",
            vec![("ability.infernal_lore", 3)],
        );
        assert_eq!(gate_issues(&validate(&wrong, &rs())).len(), 1);
    }

    /// "…or if they are magi" (`ArMDE:7151`), and apprenticeship experience may go on
    /// "Arcane, Academic, and Martial Abilities" (`ArMDE:2435`). Read off the profile
    /// flag, never a type id.
    #[test]
    fn a_magus_needs_no_virtue_for_the_restricted_categories() {
        let entity = character(
            "magus",
            vec!["virtue.the_gift", "virtue.hermetic_magus"],
            vec![
                ("ability.magic_theory", 3),
                ("ability.artes_liberales", 3),
                ("ability.single_weapon", 3),
            ],
        );
        assert!(
            gate_issues(&validate(&entity, &rs())).is_empty(),
            "issues: {:?}",
            codes(&validate(&entity, &rs()))
        );
    }

    /// D56/A0's sub-slice-1 headline case: a companion (untrained profile)
    /// holding a **test-only fixture** selection that carries
    /// `Effect::ConfersHermeticTraining` is authorized for the restricted
    /// categories with no further Virtue — exactly like a magus — because
    /// `is_hermetically_trained` unions the entity-level fact in. Today's
    /// profile-only gate would refuse him (`ability_category_requires_virtue`);
    /// the real `flaw.abandoned_apprentice` is untouched by this fixture and
    /// stays refused until D3 attaches the effect to its shipped data.
    #[test]
    fn abandoned_apprentice_shaped_test_fixture_is_authorized_for_arcane_abilities_without_a_further_virtue()
     {
        let entity = character(
            "companion",
            vec!["flaw.test_confers_training"],
            vec![("ability.magic_theory", 3)],
        );
        assert!(
            gate_issues(&validate(&entity, &rs())).is_empty(),
            "issues: {:?}",
            codes(&validate(&entity, &rs()))
        );

        // Without the fixture effect, the same companion is refused — the gate
        // genuinely switched on, it did not stand down for every companion.
        let untrained = character("companion", vec![], vec![("ability.magic_theory", 3)]);
        assert_eq!(gate_issues(&validate(&untrained, &rs())).len(), 1);
    }

    /// A ruleset that names no gated categories cannot enforce the rule, so the
    /// check stands down rather than inventing the list.
    #[test]
    fn a_ruleset_without_the_data_enforces_nothing() {
        // Same catalogue, minus the `categories_requiring_virtue` list. (The
        // engine-required Hermetic abilities stay, since the magus profile above
        // demands them of any ruleset shipping a catalogue.)
        let abilities = ABILITIES.replace(
            r#""categories_requiring_virtue": ["academic", "arcane", "martial"],"#,
            "",
        );
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(&abilities),
            ..RulesetSources::default()
        })
        .unwrap();
        let entity = character("companion", vec![], vec![("ability.single_weapon", 3)]);
        assert!(gate_issues(&validate(&entity, &rs)).is_empty());
    }
}
