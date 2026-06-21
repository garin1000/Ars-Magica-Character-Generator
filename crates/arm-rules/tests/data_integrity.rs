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
    assert!(rs.point_items.len() >= 5, "should have seed V/F data");
    assert!(
        rs.character_types.len() >= 1,
        "should have at least one character type"
    );
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
            id: "arm5-core".into(),
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

    let (v, f) = compute_balance(&entity, &rs);
    assert_eq!(v, 1);
    assert_eq!(f, 1);
}

#[test]
fn save_load_roundtrip_with_canonical_output() {
    let entity = Entity {
        schema_version: 1,
        ruleset: RulesetRef {
            id: "arm5-core".into(),
            version: "2024.1".into(),
        },
        entity_kind: EntityKind::Character,
        type_id: Id::new("companion"),
        selections: vec![
            Selection {
                item_ref: Id::new("virtue.puissant_ability"),
                params: BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            },
            Selection {
                item_ref: Id::new("flaw.poor_student"),
                params: BTreeMap::new(),
            },
        ],
    };

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
            id: "arm5-core".into(),
            version: "2024.1".into(),
        },
        entity_kind: EntityKind::Character,
        type_id: Id::new("grog"),
        selections: vec![Selection {
            item_ref: Id::new("virtue.keen_vision"),
            params: BTreeMap::new(),
        }],
    };

    let result = validate(&entity, &rs);
    assert!(
        result.is_valid(),
        "grog with one minor virtue should be valid: {:?}",
        result.issues
    );
}

#[test]
fn grog_over_budget() {
    let rs = load_ruleset();

    let entity = Entity {
        schema_version: 1,
        ruleset: RulesetRef {
            id: "arm5-core".into(),
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
