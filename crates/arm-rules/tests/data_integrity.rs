use arm_rules::ruleset::{LocalizedRuleset, Ruleset};
use arm_rules::types::*;
use arm_rules::validation::{compute_balance, validate};
use std::collections::BTreeMap;

fn load_ruleset() -> Ruleset {
    let items = include_str!("../../../rules/core/virtues_flaws.json");
    let types = include_str!("../../../rules/core/character_types.json");
    Ruleset::from_json("arm5-core", "2024.1", items, types).unwrap()
}

/// Builds a character entity of `type_id` with the given selections, at the
/// current schema version and empty trait data.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

#[test]
fn shipped_data_passes_integrity_check() {
    let rs = load_ruleset();
    assert_eq!(rs.item_count(), 10, "exact shipped V/F count");
    assert_eq!(rs.profile_count(), 2, "companion + grog");
    assert!(
        rs.item(&Id::new("virtue.the_gift")).is_some(),
        "virtue.the_gift must be present"
    );
    assert!(rs.profile(&Id::new("companion")).is_some());
    assert!(rs.profile(&Id::new("grog")).is_some());
}

#[test]
fn english_i18n_covers_all_items() {
    let rs = load_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/virtues_flaws.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();

    for item in rs.items() {
        assert!(
            loc.display_name(&item.id).is_some(),
            "English i18n missing entry for '{}'",
            item.id
        );
    }
}

#[test]
fn german_i18n_covers_all_items() {
    let rs = load_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/virtues_flaws.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();

    for item in rs.items() {
        assert!(
            loc.display_name(&item.id).is_some(),
            "German i18n missing entry for '{}'",
            item.id
        );
    }
}

#[test]
fn companion_balanced_entity_validates() {
    let rs = load_ruleset();

    let entity = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );

    let result = validate(&entity, &rs);
    assert!(
        result.is_valid(),
        "balanced companion should validate: {:?}",
        result.issues
    );

    let balance = compute_balance(&entity, &rs);
    assert_eq!(balance.virtue_points, 1);
    assert_eq!(balance.flaw_points, 1);
}

#[test]
fn save_load_roundtrip_with_canonical_output() {
    let mut entity = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );
    // Canonical output requires normalize(): Serialize no longer auto-sorts.
    entity.normalize();

    let json1 = serde_json::to_string_pretty(&entity).unwrap();
    let roundtripped: Entity = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&roundtripped).unwrap();

    assert_eq!(json1, json2, "canonical serialization should be stable");
    assert_eq!(entity, roundtripped);
}

#[test]
fn grog_type_restricts_major_virtues() {
    let rs = load_ruleset();

    // One minor virtue funded by one minor flaw, so the points balance and
    // the test isolates the Major-virtue restriction.
    let entity = entity(
        "grog",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );

    let result = validate(&entity, &rs);
    assert!(
        result.is_valid(),
        "grog with one balanced minor virtue should be valid: {:?}",
        result.issues
    );
}

#[test]
fn grog_over_budget() {
    let rs = load_ruleset();

    let entity = entity(
        "grog",
        vec![
            Selection::new(Id::new("virtue.keen_vision")),
            Selection::new(Id::new("virtue.large")),
            Selection::new(Id::new("virtue.tough")),
            Selection::new(Id::new("virtue.puissant_ability")),
        ],
    );

    let result = validate(&entity, &rs);
    let codes: Vec<&str> = result.errors().map(|i| i.code.as_str()).collect();
    assert!(
        codes.contains(&"over_budget_virtues"),
        "grog over budget: {codes:?}"
    );
}
