//! D92 (`docs/vf-audit/decisions.md`, Norbert 2026-10-19; manual-testing-findings-2026-10-18
//! #16): a known spell whose level now exceeds `effective/spell.rs::spell_level_cap`
//! ONLY because an aging drop lowered Intelligence since it was learned is a
//! WARNING, not an ERROR — a known spell is not unlearned when its creator
//! ages; the cap limits what can be *learned* (ArMDE:2541, "limited as
//! before"). `validate_spell_level_cap` (`validation/magus.rs`) now also
//! computes the cap with PRE-aging Intelligence
//! (`effective/spell.rs::spell_level_cap_pre_aging`); when the spell's level
//! is still within THAT cap, the issue code is
//! `CODE_SPELL_LEVEL_EXCEEDS_CAP_DUE_TO_AGING` instead of
//! `CODE_SPELL_LEVEL_EXCEEDS_CAP`. A spell that would be over the cap even
//! before any aging stays the plain error.

use arm_rules::characteristics::Characteristic;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{IssueSeverity, ValidationIssue, validate};

/// The shipped core ruleset with its spell catalogue replaced by a single
/// synthetic spell — same technique `requisite_level_cap.rs::ruleset_with_only_spell`
/// uses, duplicated here because integration test binaries cannot share
/// private helpers across files.
fn ruleset_with_only_spell(spell_json: &str) -> Ruleset {
    let spells_file = format!(r#"{{"spells": [{spell_json}]}}"#);
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
        spells: Some(&spells_file),
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
    .expect("single-synthetic-spell ruleset loads")
}

/// A magus with Cr 5, Ig 5, no Magic Theory, bought Intelligence `bought_int`
/// and no other lab modifiers — so the cap is `5 + 5 + Int + 0 + 3`.
fn creo_ignem_magus(bought_int: i8) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.characteristics.insert(Characteristic::Int, bought_int);
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 5),
        ArtScore::new(Id::new("art.ignem"), 5),
    ];
    e
}

/// The `spell_level_exceeds_cap`/`spell_level_exceeds_cap_due_to_aging` issue
/// for `spell_id`, if any.
fn cap_issue(entity: &Entity, ruleset: &Ruleset, spell_id: &str) -> Option<ValidationIssue> {
    validate(entity, ruleset).issues.into_iter().find(|issue| {
        issue.context.as_ref() == Some(&Id::new(spell_id))
            && (issue.code == ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP
                || issue.code == ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP_DUE_TO_AGING)
    })
}

#[test]
fn a_spell_over_cap_only_because_of_an_aging_intelligence_drop_is_a_warning() {
    // Bought Int 3: the first aging drop costs |3| + 1 = 4 points (ArMDE:16658).
    // Pre-aging cap: 5 + 5 + 3 + 0 + 3 = 16. After one drop (Int -> 2):
    // post-aging cap: 5 + 5 + 2 + 0 + 3 = 15. A spell at level 16 is legal
    // before aging (16 <= 16) but illegal after (16 > 15) — caused ONLY by aging.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_aging_cap_warning", "technique": "art.creo",
             "form": "art.ignem", "level": 16 }"#,
    );
    let mut e = creo_ignem_magus(3);
    e.aging_points.insert(Characteristic::Int, 4);
    e.spells = vec![SpellSelection::new(Id::new("spell.test_aging_cap_warning"))];

    assert_eq!(
        arm_rules::effective_characteristic_after_aging(&e, &ruleset, Characteristic::Int),
        2,
        "precondition: 4 aging points force exactly one drop on a bought Int of 3"
    );

    let issue = cap_issue(&e, &ruleset, "spell.test_aging_cap_warning")
        .expect("level 16 exceeds the post-aging cap of 15");
    assert_eq!(
        issue.code,
        ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP_DUE_TO_AGING,
        "D92: the spell is within the pre-aging cap (16), so the violation is caused only by aging"
    );
    assert_eq!(issue.severity, IssueSeverity::Warning);
    assert_eq!(issue.args.get("cap").cloned(), Some("15".to_string()));
    assert_eq!(issue.args.get("level").cloned(), Some("16".to_string()));
}

#[test]
fn a_spell_over_cap_even_before_aging_stays_an_error() {
    // Same aged magus (post-aging cap 15, pre-aging cap 16), but the spell's
    // level (20) exceeds BOTH — not caused only by aging.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_aging_cap_error", "technique": "art.creo",
             "form": "art.ignem", "level": 20 }"#,
    );
    let mut e = creo_ignem_magus(3);
    e.aging_points.insert(Characteristic::Int, 4);
    e.spells = vec![SpellSelection::new(Id::new("spell.test_aging_cap_error"))];

    let issue = cap_issue(&e, &ruleset, "spell.test_aging_cap_error")
        .expect("level 20 exceeds both the pre- and post-aging caps");
    assert_eq!(issue.code, ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP);
    assert_eq!(issue.severity, IssueSeverity::Error);
    assert_eq!(issue.args.get("cap").cloned(), Some("15".to_string()));
}

#[test]
fn an_unaged_magus_over_cap_still_gets_the_plain_error() {
    // No aging points at all: the pre- and post-aging caps are identical (16),
    // so a spell above it (17) must still be the ordinary error. Regression
    // guard — D92 must not demote a plain over-cap violation with no aging
    // involved at all.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_no_aging_cap", "technique": "art.creo",
             "form": "art.ignem", "level": 17 }"#,
    );
    let mut e = creo_ignem_magus(3);
    e.spells = vec![SpellSelection::new(Id::new("spell.test_no_aging_cap"))];

    let issue = cap_issue(&e, &ruleset, "spell.test_no_aging_cap")
        .expect("level 17 exceeds the un-aged cap of 16");
    assert_eq!(issue.code, ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP);
    assert_eq!(issue.severity, IssueSeverity::Error);
    assert_eq!(issue.args.get("cap").cloned(), Some("16".to_string()));
}
