//! Which of a character's creation phases the player has not engaged with yet.
//!
//! Legal is not the same as finished. Validation reports what the *rules* forbid,
//! and an untouched phase breaks no rule: a character with no Abilities bought and
//! a magus with no House chosen are both perfectly legal. The guided wizard
//! therefore needs a second, weaker reading of the same character — "has anything
//! been recorded here yet?" — so an empty step can be *shown* as unfinished
//! without being blocked.
//!
//! This is product behaviour, not a rule: no passage in the sources says an
//! untouched phase is incomplete, and none is cited for the criteria below. The one
//! criterion that does rest on the rules — a magus has a House
//! (Ars Magica - Definitive Edition (Core Rules).md:2859) — cites it at its arm of
//! the match.
//!
//! Deliberately NOT a [`ValidationIssue`](crate::validation::ValidationIssue) of
//! any severity. Every gate in the wizard is phrased over issues (Next blocks on an
//! `error` in the current phase, Finish on any `error` anywhere), so an issue-shaped
//! verdict would be one severity change away from gating. A separate report can
//! only ever be read as information.

use serde::{Deserialize, Serialize};

use crate::ruleset::Ruleset;
use crate::types::{CreationPhase, Entity, EntityTypeProfile, GiftPolicy, Id};

/// The phases of an entity's own creation flow that hold no choices yet.
///
/// Scoped to the phases the entity's type profile declares, in that declared
/// order — the order the wizard's rail walks them, so a consumer can list them
/// without re-sorting. A phase the profile never declares (a grog has no `arts`
/// step) is not reported, and neither is the wizard's synthetic `review` step.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletenessReport {
    /// The declared phases with nothing recorded, in the profile's own order.
    #[serde(default)]
    pub incomplete_phases: Vec<CreationPhase>,
}

impl CompletenessReport {
    /// Whether `phase` is one of the phases still untouched.
    pub fn is_incomplete(&self, phase: CreationPhase) -> bool {
        self.incomplete_phases.contains(&phase)
    }
}

/// Reports which of the entity's declared creation phases the player has not
/// engaged with yet.
///
/// Empty when the entity's `type_id` resolves to no profile: without a profile
/// there is no declared flow to be part-way through, and the missing type is
/// already an `unknown_type` error.
pub fn completeness(entity: &Entity, ruleset: &Ruleset) -> CompletenessReport {
    let Some(profile) = ruleset.type_profiles.get(&entity.type_id) else {
        return CompletenessReport::default();
    };

    CompletenessReport {
        incomplete_phases: profile
            .creation_phases
            .iter()
            .copied()
            .filter(|phase| !phase_is_engaged(*phase, entity, profile))
            .collect(),
    }
}

/// Whether the player has recorded anything this phase owns.
///
/// The match is exhaustive on purpose: a new [`CreationPhase`] must not silently
/// count as finished, so adding one is a compile error until its criterion is
/// written. Each criterion reads a *stored choice* off the entity — never a
/// derived value and never a validation finding — so "the player has been here"
/// means the same thing whichever way the character was built.
fn phase_is_engaged(phase: CreationPhase, entity: &Entity, profile: &EntityTypeProfile) -> bool {
    match phase {
        // Any identity field at all: the step's inputs are all free text and the
        // rules ask for none of them, so filling in a single one is engagement.
        CreationPhase::Concept => has_identity_text(entity),
        // A score actually distributed. An all-zero spread is what an untouched
        // point-buy looks like, and spending nothing is legal but not a choice.
        CreationPhase::Characteristics => entity.characteristics.values().any(|&score| score != 0),
        // A selection the profile did not force. A magus is seeded with The Gift
        // and Hermetic Magus at creation, so counting every selection would report
        // the step as finished before it is opened.
        CreationPhase::VirtuesFlaws => entity
            .selections
            .iter()
            .any(|selection| !is_mandatory_trait(&selection.item_ref, profile)),
        // Either funding mode, recorded with substance. The step's whole job is to
        // say where a character's experience comes from, and it can be answered two
        // ways: a life-stage plan (the age the years are priced from, the native
        // language, the childhood package) or a typed flat pool.
        //
        // Both have to count. The mode itself is not stored — it is inferred from
        // the plan's *absence* (`abilityFunding`, `state.svelte.ts`), and absence is
        // also what an untouched character looks like — so keying only on the plan
        // would report the step untouched forever for every pool-funded character,
        // which is the default. `xp_pool` defaults to 0, so a nonzero pool is a
        // deliberate entry rather than a leftover. #29 replaces the inference with a
        // stored discriminator; this stays correct either way.
        CreationPhase::Experience => entity.life_stages.is_some() || entity.xp_pool > 0,
        CreationPhase::Abilities => !entity.ability_scores.is_empty(),
        CreationPhase::Arts => !entity.art_scores.is_empty(),
        CreationPhase::Spells => !entity.spells.is_empty(),
        // "You receive one free Minor Virtue from your choice of House"
        // (Ars Magica - Definitive Edition (Core Rules).md:2859) — a magus has a
        // House to choose, and that choice is the one thing this step stores. The
        // specialisation picks are not required for the step to count as visited:
        // a House offering no player choice leaves none to make.
        CreationPhase::HouseSpecialisation => entity.house.is_some(),
        CreationPhase::MythicType => entity.mythic_type.is_some(),
        CreationPhase::PersonalityReputations => {
            !entity.personality_traits.is_empty() || !entity.reputations.is_empty()
        }
        // The age. Everything else on the step — the aging rolls owed, the Living
        // Conditions modifier, the Longevity Ritual's bonus — is read against it,
        // so without an age the step cannot even say what it wants.
        CreationPhase::Aging => entity.age.is_some(),
        // The closing look at the whole character. It holds no choices of its own,
        // so it can never be untouched.
        CreationPhase::Review => true,
    }
}

/// Whether any of the concept step's free-text identity fields carries a value.
fn has_identity_text(entity: &Entity) -> bool {
    !entity.name.is_empty()
        || !entity.description.is_empty()
        || !entity.concept.is_empty()
        || !entity.gender.is_empty()
        || entity.birth_year.is_some()
        || !entity.sigil.is_empty()
        || !entity.covenant_name.is_empty()
        || !entity.parens.is_empty()
}

/// Whether the profile forces this trait on every character of its type — the
/// declared `required_traits` plus The Gift where the type demands it. Mirrors the
/// frontend's `mandatoryTraitRefs`, which is what seeds them at creation.
fn is_mandatory_trait(item_ref: &Id, profile: &EntityTypeProfile) -> bool {
    if profile.required_traits.contains(item_ref) {
        return true;
    }
    profile.gift_policy == Some(GiftPolicy::Required) && profile.gift_id.as_ref() == Some(item_ref)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::characteristics::Characteristic;
    use crate::life_stage::LifeStagePlan;
    use crate::types::{
        AbilityScore, ArtScore, EntityKind, PersonalityTrait, Reputation, ReputationType,
        RulesetRef, Selection, SpellSelection,
    };
    use crate::validation::validate;
    use crate::{Ruleset, RulesetSources};

    const ITEMS: &str = r#"[
      { "id": "virtue.keen_vision", "kind": "virtue", "classification": "narrative",
        "magnitude": "minor", "category": "general", "entity_kinds": ["character"] },
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] },
      { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative",
        "magnitude": "free", "category": "special", "entity_kinds": ["character"] },
      { "id": "virtue.hermetic_magus", "kind": "virtue", "classification": "narrative",
        "magnitude": "free", "category": "social_status", "entity_kinds": ["character"] }
    ]"#;

    const TYPES: &str = r#"[
      { "id": "grog", "budget": { "virtue_points": 3, "flaw_points": 3 },
        "permitted_categories": ["general", "personality", "social_status", "special"],
        "creation_phases": ["concept", "characteristics", "virtues_flaws",
                            "experience", "abilities", "aging"] },
      { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality", "social_status", "special"],
        "is_magus": true, "gift_policy": "required", "gift_id": "virtue.the_gift",
        "required_traits": ["virtue.hermetic_magus"],
        "creation_phases": ["concept", "characteristics", "house_specialisation",
                            "virtues_flaws", "experience", "abilities", "arts", "spells",
                            "personality_reputations", "aging"] },
      { "id": "mythic_companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["mythic_type"] }
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

    fn character(type_id: &str) -> Entity {
        Entity::new(
            EntityKind::Character,
            Id::new(type_id),
            RulesetRef::new(Id::new("test"), "1"),
        )
    }

    fn magus() -> Entity {
        let mut entity = character("magus");
        entity.selections = vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("virtue.hermetic_magus")),
        ];
        entity
    }

    fn incomplete(entity: &Entity) -> Vec<CreationPhase> {
        completeness(entity, &rs()).incomplete_phases
    }

    #[test]
    fn an_untouched_character_is_incomplete_in_every_phase_that_takes_a_choice() {
        assert_eq!(
            incomplete(&character("grog")),
            vec![
                CreationPhase::Concept,
                CreationPhase::Characteristics,
                CreationPhase::VirtuesFlaws,
                CreationPhase::Experience,
                CreationPhase::Abilities,
                CreationPhase::Aging,
            ],
            "every declared step of a fresh character still takes a choice"
        );
    }

    #[test]
    fn a_stored_life_stage_plan_finishes_the_experience_step() {
        let mut entity = character("grog");
        assert!(incomplete(&entity).contains(&CreationPhase::Experience));
        entity.life_stages = Some(LifeStagePlan::default());
        assert!(!incomplete(&entity).contains(&CreationPhase::Experience));
    }

    /// The other funding mode counts too. A plan is not the only thing this step
    /// records: under flat funding the choice is a typed [`Entity::xp_pool`], and
    /// a character with one has engaged the step just as much as one with a plan.
    /// Without this, every pool-funded character — the default, since the mode is
    /// inferred from the plan's *absence* — would report the step untouched
    /// forever, which is both wrong and noise on the rail.
    #[test]
    fn a_typed_experience_pool_finishes_the_experience_step() {
        let mut entity = character("grog");
        assert!(incomplete(&entity).contains(&CreationPhase::Experience));
        entity.xp_pool = 45;
        assert!(!incomplete(&entity).contains(&CreationPhase::Experience));
    }

    #[test]
    fn the_report_follows_the_profiles_declared_order() {
        let phases = incomplete(&magus());
        let house = phases
            .iter()
            .position(|p| *p == CreationPhase::HouseSpecialisation);
        let vf = phases
            .iter()
            .position(|p| *p == CreationPhase::VirtuesFlaws);
        assert!(
            house < vf,
            "the magus profile declares the House step before Virtues & Flaws: {phases:?}"
        );
    }

    #[test]
    fn a_phase_the_profile_never_declares_is_never_reported() {
        let phases = incomplete(&character("grog"));
        for phase in [
            CreationPhase::Arts,
            CreationPhase::Spells,
            CreationPhase::HouseSpecialisation,
            CreationPhase::MythicType,
            CreationPhase::PersonalityReputations,
        ] {
            assert!(
                !phases.contains(&phase),
                "a grog declares no {phase} step, so it cannot be incomplete: {phases:?}"
            );
        }
    }

    #[test]
    fn the_closing_review_step_is_never_incomplete() {
        // No profile may declare it, and the wizard appends it; either way it holds
        // no choices, so nothing about a character can leave it untouched.
        assert!(!incomplete(&magus()).contains(&CreationPhase::Review));
    }

    #[test]
    fn an_unresolved_type_reports_nothing() {
        let entity = character("nonesuch");
        assert!(completeness(&entity, &rs()).incomplete_phases.is_empty());
    }

    #[test]
    fn a_name_finishes_the_concept_step() {
        let mut entity = character("grog");
        entity.name = "Otto".to_string();
        assert!(!incomplete(&entity).contains(&CreationPhase::Concept));
    }

    #[test]
    fn any_single_identity_field_finishes_the_concept_step() {
        let mut entity = character("grog");
        entity.birth_year = Some(1194);
        assert!(!incomplete(&entity).contains(&CreationPhase::Concept));
    }

    #[test]
    fn characteristics_left_at_zero_are_not_a_choice() {
        let mut entity = character("grog");
        entity.characteristics.insert(Characteristic::Str, 0);
        assert!(
            incomplete(&entity).contains(&CreationPhase::Characteristics),
            "an all-zero spread is what an untouched point-buy looks like"
        );

        entity.characteristics.insert(Characteristic::Str, -1);
        assert!(!incomplete(&entity).contains(&CreationPhase::Characteristics));
    }

    #[test]
    fn the_traits_a_type_forces_do_not_finish_the_virtues_flaws_step() {
        // A magus is seeded with The Gift and Hermetic Magus before the step opens.
        assert!(incomplete(&magus()).contains(&CreationPhase::VirtuesFlaws));

        let mut chosen = magus();
        chosen
            .selections
            .push(Selection::new(Id::new("virtue.keen_vision")));
        assert!(!incomplete(&chosen).contains(&CreationPhase::VirtuesFlaws));
    }

    #[test]
    fn a_bought_ability_finishes_the_abilities_step() {
        let mut entity = character("grog");
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.awareness"),
            parameter: None,
            score: 2,
            specialty: None,
        }];
        assert!(!incomplete(&entity).contains(&CreationPhase::Abilities));
    }

    #[test]
    fn a_bought_art_finishes_the_arts_step() {
        let mut entity = magus();
        entity.art_scores = vec![ArtScore {
            art: Id::new("art.creo"),
            score: 5,
        }];
        assert!(!incomplete(&entity).contains(&CreationPhase::Arts));
    }

    #[test]
    fn a_known_spell_finishes_the_spells_step() {
        let mut entity = magus();
        entity.spells = vec![SpellSelection {
            spell: Id::new("spell.pilum_of_fire"),
            level: None,
            mastery: None,
            mastery_abilities: Vec::new(),
            parameter: None,
        }];
        assert!(!incomplete(&entity).contains(&CreationPhase::Spells));
    }

    #[test]
    fn a_chosen_house_finishes_the_house_step() {
        let mut entity = magus();
        assert!(incomplete(&entity).contains(&CreationPhase::HouseSpecialisation));

        entity.house = Some(Id::new("house.bonisagus"));
        assert!(!incomplete(&entity).contains(&CreationPhase::HouseSpecialisation));
    }

    #[test]
    fn a_chosen_mythic_type_finishes_the_mythic_type_step() {
        let mut entity = character("mythic_companion");
        assert!(incomplete(&entity).contains(&CreationPhase::MythicType));

        entity.mythic_type = Some(Id::new("mythic.faerie_noble"));
        assert!(!incomplete(&entity).contains(&CreationPhase::MythicType));
    }

    #[test]
    fn either_a_trait_or_a_reputation_finishes_the_personality_step() {
        let mut with_trait = magus();
        with_trait.personality_traits = vec![PersonalityTrait {
            name: "Brave".to_string(),
            value: 3,
        }];
        assert!(!incomplete(&with_trait).contains(&CreationPhase::PersonalityReputations));

        let mut with_reputation = magus();
        with_reputation.reputations = vec![Reputation {
            kind: ReputationType::Local,
            score: 2,
            content: "healer".to_string(),
        }];
        assert!(!incomplete(&with_reputation).contains(&CreationPhase::PersonalityReputations));
    }

    #[test]
    fn an_age_finishes_the_aging_step() {
        let mut entity = character("grog");
        entity.age = Some(25);
        assert!(!incomplete(&entity).contains(&CreationPhase::Aging));
    }

    /// The whole point of the report: it describes a character the rules have
    /// nothing to say about, so it can never be mistaken for a gate.
    #[test]
    fn a_character_can_be_legal_and_incomplete_at_once() {
        let entity = character("grog");
        let result = validate(&entity, &rs());
        assert!(
            result.is_valid(),
            "an untouched grog breaks no rule: {:?}",
            result.errors().collect::<Vec<_>>()
        );
        assert!(!incomplete(&entity).is_empty());
    }
}
