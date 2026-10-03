//! After-deadline answer 5 (`tmp/questions-after-deadline.md`, 2026-10-03): a
//! Virtue that confers an Ability at a fixed score says so in its summary, in
//! both locales — the shape F-65 gave Dowsing and Curse-Throwing ("Confers the
//! Dowsing Ability at 1." / „Verleiht die Fertigkeit Wünschelrutengehen auf 1.“).
//!
//! Walked from the shipped data, never listed: every item carrying an
//! `ability_score_grant` or `ability_score_grant_param` effect, with the score
//! read from that effect's own `amount`. So a Virtue added later as a data-only
//! change is held to the same rule the day it lands.
//!
//! The summaries are app-authored text, not the verbatim rulebook description,
//! so the granted score may be stated in the house style. The phrasings accepted
//! are the ones the shipped summaries already use: "at N" / "at a score of N" and
//! „auf N“ / „mit einem Wert von N“.

use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::{Effect, Id};

fn shipped_ruleset() -> Ruleset {
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
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
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

fn localized_en() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        shipped_ruleset(),
        &[
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
            include_str!("../../../rules/i18n/en/abilities.json"),
            include_str!("../../../rules/i18n/en/arts.json"),
            include_str!("../../../rules/i18n/en/houses.json"),
            include_str!("../../../rules/i18n/en/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/en/spells.json"),
            include_str!("../../../rules/i18n/en/spell_mastery_abilities.json"),
            include_str!("../../../rules/i18n/en/equipment.json"),
            include_str!("../../../rules/i18n/en/aging.json"),
        ],
    )
    .expect("shipped English rules text loads")
}

fn localized_de() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        shipped_ruleset(),
        &[
            include_str!("../../../rules/i18n/de/virtues_flaws.json"),
            include_str!("../../../rules/i18n/de/abilities.json"),
            include_str!("../../../rules/i18n/de/arts.json"),
            include_str!("../../../rules/i18n/de/houses.json"),
            include_str!("../../../rules/i18n/de/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/de/spells.json"),
            include_str!("../../../rules/i18n/de/spell_mastery_abilities.json"),
            include_str!("../../../rules/i18n/de/equipment.json"),
            include_str!("../../../rules/i18n/de/aging.json"),
        ],
    )
    .expect("shipped German rules text loads")
}

/// Every item that confers an Ability at a fixed score, with that score.
/// The third field records whether the grant is the parameter-relative
/// variant, so the test can prove that variant was walked too.
fn ability_grants(ruleset: &Ruleset) -> Vec<(Id, u8, bool)> {
    let mut grants = Vec::new();
    for item in ruleset.items() {
        for effect in &item.effects {
            match effect {
                Effect::AbilityScoreGrant { amount, .. } => {
                    grants.push((item.id.clone(), *amount, false));
                }
                Effect::AbilityScoreGrantParam { amount, .. } => {
                    grants.push((item.id.clone(), *amount, true));
                }
                _ => {}
            }
        }
    }
    grants
}

/// Whether `summary` states the score `amount` through one of `phrases`: the
/// phrase must start a word, and the number must not run on into more digits
/// ("at 1" must not be satisfied by "at 10").
fn states_score(summary: &str, phrases: &[&str], amount: u8) -> bool {
    phrases.iter().any(|phrase| {
        let needle = format!("{phrase} {amount}");
        summary.match_indices(&needle).any(|(at, _)| {
            let starts_word = summary[..at]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric());
            let ends_number = summary[at + needle.len()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_ascii_digit());
            starts_word && ends_number
        })
    })
}

const EN_PHRASES: &[&str] = &["at", "at a score of"];
const DE_PHRASES: &[&str] = &["auf", "mit einem Wert von"];

#[test]
fn every_ability_granting_virtue_states_the_granted_score_in_both_locales() {
    let en = localized_en();
    let de = localized_de();
    let grants = ability_grants(&en.ruleset);
    // A sweep over an empty set would pass vacuously, and one that never met
    // the parameter-relative variant would not prove Enchanting (Ability).
    assert!(!grants.is_empty(), "no ability_score_grant effects found");
    assert!(
        grants.iter().any(|(_, _, param)| *param),
        "no ability_score_grant_param effect found"
    );

    let mut missing = Vec::new();
    for (id, amount, _) in &grants {
        for (lang, ruleset, phrases) in [("en", &en, EN_PHRASES), ("de", &de, DE_PHRASES)] {
            let summary = ruleset.summary(id).unwrap_or("");
            if !states_score(summary, phrases, *amount) {
                missing.push(format!("{lang} {id} (score {amount}): {summary}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "these summaries do not state the granted Ability score:\n{}",
        missing.join("\n")
    );
}

/// The helper itself: a phrase inside a longer word, or a number running on
/// into more digits, is not a statement of the score.
#[test]
fn states_score_matches_only_the_whole_phrase_and_number() {
    assert!(states_score(
        "Confers the Dowsing Ability at 1.",
        EN_PHRASES,
        1
    ));
    assert!(states_score(
        "… of Corpse Magic, at a score of 1.",
        EN_PHRASES,
        1
    ));
    assert!(states_score(
        "Verleiht die Fertigkeit Herztier auf 1.",
        DE_PHRASES,
        1
    ));
    assert!(!states_score(
        "Confers the Dowsing Ability at 10.",
        EN_PHRASES,
        1
    ));
    assert!(!states_score("Find that 1 thing.", EN_PHRASES, 1));
    assert!(!states_score("Confers the Dowsing Ability.", EN_PHRASES, 1));
}
