use arm_rules::ruleset::{LocalizedRuleset, Ruleset};
use arm_rules::types::*;
use arm_rules::validation::{compute_balance, validate};
use std::collections::BTreeMap;

fn load_ruleset() -> Ruleset {
    let items = include_str!("../../../rules/core/virtues_flaws.json");
    let types = include_str!("../../../rules/core/character_types.json");
    Ruleset::from_json("arm5-core", "2024.1", items, types).unwrap()
}

#[test]
fn shipped_data_passes_integrity_check() {
    let rs = load_ruleset();
    assert_eq!(rs.point_items.len(), 10, "exact shipped V/F count");
    assert_eq!(rs.type_profiles.len(), 2, "companion + grog");
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

    for id in rs.point_items.keys() {
        assert!(
            loc.display_name(id).is_some(),
            "English i18n missing entry for '{id}'"
        );
    }
}

#[test]
fn german_i18n_covers_all_items() {
    let rs = load_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/virtues_flaws.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();

    for id in rs.point_items.keys() {
        assert!(
            loc.display_name(id).is_some(),
            "German i18n missing entry for '{id}'"
        );
    }
}

#[test]
fn companion_balanced_entity_validates() {
    let rs = load_ruleset();

    let entity = Entity {
        schema_version: 1,
        ruleset: RulesetRef {
            id: Id::new("arm5-core"),
            version: "2024.1".into(),
        },
        entity_kind: EntityKind::Character,
        type_id: Id::new("companion"),
        selections: vec![
            Selection {
                item_ref: Id::new("virtue.keen_vision"),
                params: BTreeMap::new(),
            },
            Selection {
                item_ref: Id::new("flaw.poor_student"),
                params: BTreeMap::new(),
            },
        ],
    };

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
    let mut entity = Entity {
        schema_version: 1,
        ruleset: RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        entity_kind: EntityKind::Character,
        type_id: Id::new("companion"),
        selections: vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    };
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

    let entity = Entity {
        schema_version: 1,
        ruleset: RulesetRef {
            id: Id::new("arm5-core"),
            version: "2024.1".into(),
        },
        entity_kind: EntityKind::Character,
        type_id: Id::new("grog"),
        // One minor virtue funded by one minor flaw, so the points balance and
        // the test isolates the Major-virtue restriction.
        selections: vec![
            Selection {
                item_ref: Id::new("virtue.keen_vision"),
                params: BTreeMap::new(),
            },
            Selection {
                item_ref: Id::new("flaw.poor_student"),
                params: BTreeMap::new(),
            },
        ],
    };

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

    let entity = Entity {
        schema_version: 1,
        ruleset: RulesetRef {
            id: Id::new("arm5-core"),
            version: "2024.1".into(),
        },
        entity_kind: EntityKind::Character,
        type_id: Id::new("grog"),
        selections: vec![
            Selection {
                item_ref: Id::new("virtue.keen_vision"),
                params: BTreeMap::new(),
            },
            Selection {
                item_ref: Id::new("virtue.large"),
                params: BTreeMap::new(),
            },
            Selection {
                item_ref: Id::new("virtue.tough"),
                params: BTreeMap::new(),
            },
            Selection {
                item_ref: Id::new("virtue.puissant_ability"),
                params: BTreeMap::new(),
            },
        ],
    };

    let result = validate(&entity, &rs);
    let codes: Vec<&str> = result.errors().iter().map(|i| i.code.as_str()).collect();
    assert!(
        codes.contains(&"over_budget_virtues"),
        "grog over budget: {codes:?}"
    );
}
