//! Whether an entity's Hermetic training is real — whether declared by its
//! type profile or conferred by a selection (D56/A0's `is_magus` split).
//! Sibling to `effective/gift_confidence.rs::has_the_gift`: a second
//! necessary-but-not-sufficient precondition alongside it (D24). Pure code
//! addition, no behavior change for any profile shipping today — the union
//! only diverges from the bare profile flag once an entity actually holds a
//! selection carrying [`Effect::ConfersHermeticTraining`].

use super::*;

/// Pure selection fold, independent of the type profile: true if any of the
/// entity's present bought selections, or non-warping grants
/// ([`entity_grants_base`]), carries [`Effect::ConfersHermeticTraining`] — e.g.
/// the Abandoned Apprentice Flaw (ArMDE:5641-5650), "you have most of the
/// skills and knowledge of a fully trained magus, but you were never able to
/// complete your training". See [`is_hermetically_trained`] for the fact every
/// "trained" production site should actually call.
///
/// Deliberately **not** the full [`selections_for_effects`] (which also folds
/// in owed Warping fills, via [`entity_grants`]): this function is called from
/// inside `warping_owed` (through [`is_hermetically_trained`]), and
/// `warping_owed` is itself what decides which fills are owed —
/// `entity_grants` → `warping_granted_selections` → `warping_owed_grants` →
/// `warping_owed` would be a direct cycle. `entity_grants_base` is the same
/// recursion-safe set `warping_points_for_owed` already uses for exactly this
/// reason; no shipped or plausible Warping-owed fill carries this effect
/// anyway, so nothing is lost by excluding fills here.
pub fn entity_confers_hermetic_training(entity: &Entity, ruleset: &Ruleset) -> bool {
    let mut present = entity.selections.clone();
    present.extend(entity_grants_base(entity, ruleset));
    present.iter().any(|selection| {
        ruleset
            .point_items
            .get(&selection.item_ref)
            .is_some_and(|item| {
                item.effects
                    .iter()
                    .any(|effect| matches!(effect, Effect::ConfersHermeticTraining))
            })
    })
}

/// Single source of truth for "is this entity's Hermetic training real,
/// whether by profile or by selection?" A union of the type profile's own
/// [`EntityTypeProfile::hermetically_trained`] flag (true for the magus
/// profile, false for every other type today) and
/// [`entity_confers_hermetic_training`] (true only for an entity holding a
/// training-conferring selection, e.g. the Abandoned Apprentice).
///
/// Every "trained" production site is obliged to call this instead of reading
/// `profile.hermetically_trained` directly — **except** the documented
/// **profile-only** sites (`life_stage.rs::LifeStageRules::apprenticeship_of`,
/// `validation/life_stage.rs::validate_life_stage_plan`), which must NOT union
/// in the entity-level fact: an Abandoned Apprentice's truncated
/// apprenticeship must not be held to `apprenticeship.minimum_abilities`
/// (D56) — reading the unioned fact there would be exactly the bug D56
/// forbids — and **except** `validation/prereq.rs::PrereqCtx`, which threads a
/// genuine three-valued `Option<bool>` this `bool`-returning helper cannot
/// express. See `docs/vf-audit/design-a0-is-magus-split.md` § 1.
pub fn is_hermetically_trained(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: Option<&EntityTypeProfile>,
) -> bool {
    profile.is_some_and(|p| p.hermetically_trained)
        || entity_confers_hermetic_training(entity, ruleset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CreationPhase, Entity, EntityKind, Id, RulesetRef, Selection};
    use crate::{Ruleset, RulesetSources};

    const ITEMS: &str = r#"[
      { "id": "flaw.test_confers_training", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "major", "categories": ["story"], "entity_kinds": ["character"],
        "effects": [{ "type": "confers_hermetic_training" }] },
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
    ]"#;
    const TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["personality", "story"], "creation_phases": [] },
      { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["personality", "story"],
        "hermetically_trained": true, "order_member": true, "creation_phases": [] }
    ]"#;

    fn rs() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn character(type_id: &str, selections: Vec<&str>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new(type_id),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.selections = selections
            .into_iter()
            .map(|r| Selection::new(Id::new(r)))
            .collect();
        entity
    }

    #[test]
    fn entity_confers_hermetic_training_is_false_with_no_such_selection() {
        let entity = character("companion", vec!["flaw.optimistic"]);
        assert!(!entity_confers_hermetic_training(&entity, &rs()));
    }

    #[test]
    fn entity_confers_hermetic_training_is_true_when_a_selection_carries_the_effect() {
        let entity = character("companion", vec!["flaw.test_confers_training"]);
        assert!(entity_confers_hermetic_training(&entity, &rs()));
    }

    #[test]
    fn is_hermetically_trained_is_true_for_the_magus_profile_with_no_selections_at_all() {
        // Load-bearing per the design note: a fresh magus, before any V/F
        // selection, must still read as trained (the profile term alone).
        let entity = character("magus", vec![]);
        let ruleset = rs();
        let profile = ruleset.profile(&Id::new("magus"));
        assert!(is_hermetically_trained(&entity, &ruleset, profile));
    }

    #[test]
    fn is_hermetically_trained_unions_in_the_entity_level_effect_for_a_non_magus_profile() {
        let entity = character("companion", vec!["flaw.test_confers_training"]);
        let ruleset = rs();
        let profile = ruleset.profile(&Id::new("companion"));
        assert!(is_hermetically_trained(&entity, &ruleset, profile));
    }

    #[test]
    fn is_hermetically_trained_is_false_for_a_non_magus_profile_with_no_such_selection() {
        let entity = character("companion", vec!["flaw.optimistic"]);
        let ruleset = rs();
        let profile = ruleset.profile(&Id::new("companion"));
        assert!(!is_hermetically_trained(&entity, &ruleset, profile));
    }

    #[test]
    fn is_hermetically_trained_is_false_with_no_profile_and_no_conferring_selection() {
        let entity = character("companion", vec!["flaw.optimistic"]);
        let ruleset = rs();
        assert!(!is_hermetically_trained(&entity, &ruleset, None));
    }

    // --- `phases_in_force` (A2/D56 § 6): the conditional-phase mechanism that
    // resolves `EntityTypeProfile::creation_phases` the same way
    // `categories_in_force` resolves `permitted_categories`. Test-fixture-only —
    // per the design's sub-slice ordering, the shipped
    // `flaw.abandoned_apprentice` entry gains `ConfersHermeticTraining` only in
    // slice D3, not here.

    const PHASE_TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["personality", "story"],
        "creation_phases": [
          "concept",
          { "phase": "arts", "when": { "kind": "hermetically_trained" } },
          { "phase": "spells", "when": { "kind": "hermetically_trained" } }
        ] }
    ]"#;

    fn phase_rs() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: PHASE_TYPES,
            ..RulesetSources::default()
        })
        .unwrap()
    }

    #[test]
    fn phases_in_force_omits_a_conditional_phase_whose_condition_is_unmet() {
        let entity = character("companion", vec!["flaw.optimistic"]);
        assert_eq!(
            crate::validation::phases_in_force(&entity, &phase_rs()),
            vec![CreationPhase::Concept]
        );
    }

    #[test]
    fn phases_in_force_includes_a_conditional_phase_once_a_selection_confers_training() {
        // The Abandoned-Apprentice-shaped test fixture: trained by selection,
        // not by the profile flag (the companion profile here sets no
        // `hermetically_trained: true` at all).
        let entity = character("companion", vec!["flaw.test_confers_training"]);
        assert_eq!(
            crate::validation::phases_in_force(&entity, &phase_rs()),
            vec![
                CreationPhase::Concept,
                CreationPhase::Arts,
                CreationPhase::Spells
            ]
        );
    }

    #[test]
    fn phases_in_force_preserves_the_profile_s_declared_order() {
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": ["personality", "story"],
            "creation_phases": [
              { "phase": "arts", "when": { "kind": "hermetically_trained" } },
              "concept"
            ] }
        ]"#;
        let ruleset = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: types,
            ..RulesetSources::default()
        })
        .unwrap();
        let entity = character("companion", vec!["flaw.test_confers_training"]);
        assert_eq!(
            crate::validation::phases_in_force(&entity, &ruleset),
            vec![CreationPhase::Arts, CreationPhase::Concept],
            "declared order, not enum/canonical order"
        );
    }

    #[test]
    fn phases_in_force_is_empty_without_a_resolvable_profile() {
        let entity = character("no_such_type", vec![]);
        assert_eq!(
            crate::validation::phases_in_force(&entity, &phase_rs()),
            Vec::<CreationPhase>::new()
        );
    }
}
