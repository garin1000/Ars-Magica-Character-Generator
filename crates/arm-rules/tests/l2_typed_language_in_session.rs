//! L2, item 3 (Norbert's decision on try-out finding 8): typed text that equals a
//! catalogue name, case-insensitively and trimmed, counts as that language —
//! **in the session too**, not only after a save is reopened.
//!
//! The load path already converts such text to the catalogue id
//! (`migration.rs::fold_catalogue_matching`). But a player who picks "Other…" and
//! types "Latin" holds `{ "text": "Latin" }` until the document is reopened, and
//! every validation in between runs on that text. The engine's `Ruleset` carries
//! no names (they live in `rules/i18n/`), so the app attaches both locales'
//! catalogue names to the ruleset it loads, and the one matching helper compares
//! typed text against them. Every check that asks "is this instance language X"
//! (magus minimum, Academic requirement) then answers the same in-session as after
//! a reload. A ruleset with no names attached matches ids only
//! (`l2_language_requirements.rs::without_catalogue_names_typed_text_is_no_catalogue_value`).
//!
//! Red-checkpoint protocol, phase 1: `Ruleset::with_catalogue_names` does not
//! exist yet, so this file fails to compile — the accepted first red. Phase 2
//! adds the bare method first and shows the assertions failing.

use std::collections::BTreeMap;

use arm_rules::life_stage::AbilityRequirementKind;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityParameterValue, AbilityScore, Entity, Id};
use arm_rules::{magus_minimum_abilities, validate};

const DEAD: &str = "ability.dead_language";
const LIVING: &str = "ability.living_language";
const ACADEMIC_WARNING: &str = "academic_ability_without_scholarly_language";

fn full_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

/// The shipped ruleset with both locales' catalogue names attached, as the app
/// loads it.
fn session_ruleset() -> Ruleset {
    let ruleset = full_ruleset();
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    let names: BTreeMap<Id, Vec<String>> =
        arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
            .expect("catalogue names load");
    ruleset.with_catalogue_names(names)
}

fn score(ability: &str, text: &str, score: u8) -> AbilityScore {
    let mut row = AbilityScore::new(Id::new(ability), score);
    row.parameter = Some(AbilityParameterValue::text(text));
    row
}

fn character(scores: Vec<AbilityScore>) -> Entity {
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.ability_scores = scores;
    entity
}

fn latin_minimum_met(entity: &Entity, ruleset: &Ruleset) -> bool {
    magus_minimum_abilities(entity, ruleset)
        .into_iter()
        .find(|row| {
            row.ability == Id::new(DEAD) && row.requirement == AbilityRequirementKind::Required
        })
        .expect("the Latin minimum is on the checklist")
        .met
}

/// Typed "Latin" in any case and padding, or the German "Latein" under an
/// English UI, meets the Latin minimum in the session; "Old Norse" does not.
#[test]
fn typed_latin_meets_the_minimum_in_session() {
    let rs = session_ruleset();
    for name in ["Latin", "latin", "  LATIN ", "Latein"] {
        let magus = character(vec![score(DEAD, name, 1)]);
        assert!(
            latin_minimum_met(&magus, &rs),
            "typed {name:?} under Dead Language is Latin"
        );
    }
    let magus = character(vec![score(DEAD, "Old Norse", 1)]);
    assert!(!latin_minimum_met(&magus, &rs), "Old Norse is not Latin");
}

/// The typed name is matched only within the instance's own catalogue: "Latin"
/// typed under Living Language is not Dead Language: Latin.
#[test]
fn typed_latin_under_living_language_is_not_the_dead_language_latin() {
    let rs = session_ruleset();
    let magus = character(vec![score(LIVING, "Latin", 4)]);
    assert!(!latin_minimum_met(&magus, &rs));
}

/// The Academic requirement reads typed names the same way: "griechisch" under
/// Living Language at 3 is Greek 3.
#[test]
fn typed_greek_satisfies_the_academic_requirement_in_session() {
    let rs = session_ruleset();
    let mut scores = vec![AbilityScore::new(Id::new("ability.artes_liberales"), 1)];
    scores.push(score(LIVING, "griechisch", 3));
    let entity = character(scores);
    let warnings = validate(&entity, &rs)
        .issues
        .into_iter()
        .filter(|issue| issue.code == ACADEMIC_WARNING)
        .count();
    assert_eq!(warnings, 0, "typed Greek 3 satisfies ArMDE:7151");
}
