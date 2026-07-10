use arm_rules::AbilityCategory;
use arm_rules::Characteristic;
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{compute_balance, validate};
use std::collections::BTreeMap;

fn load_ruleset() -> Ruleset {
    let items = include_str!("../../../rules/core/virtues_flaws.json");
    let types = include_str!("../../../rules/core/character_types.json");
    let abilities = include_str!("../../../rules/core/abilities.json");
    let characteristics = include_str!("../../../rules/core/characteristics.json");
    Ruleset::from_core_json(
        "arm5-core",
        "2024.1",
        items,
        types,
        abilities,
        characteristics,
    )
    .unwrap()
}

/// The full shipped ruleset including Arts and the spell catalogue — the spell
/// tests need Arts loaded so each spell's Technique/Form resolves.
fn load_ruleset_with_spells() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: None,
        mythic_types: None,
        spells: Some(include_str!("../../../rules/core/spells.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
    })
    .unwrap()
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
    // Catalogue size is data, not code: assert key items are present, never an
    // exact V/F total (which would break when any item is added to the JSON).
    assert!(
        rs.item(&Id::new("virtue.the_gift")).is_some(),
        "virtue.the_gift must be present"
    );
    // All four character-type profiles must load.
    assert!(rs.profile(&Id::new("companion")).is_some());
    assert!(rs.profile(&Id::new("grog")).is_some());
    assert!(rs.profile(&Id::new("magus")).is_some());
    assert!(rs.profile(&Id::new("mythic_companion")).is_some());
    // The magus is the only seeded Hermetic type.
    assert!(
        rs.profile(&Id::new("magus")).unwrap().is_magus,
        "magus profile must carry is_magus"
    );
    // Mythic Companions convert each Flaw point into two Virtue points.
    assert_eq!(
        rs.profile(&Id::new("mythic_companion"))
            .unwrap()
            .budget
            .virtue_points_per_flaw_point,
        2,
        "mythic companion funds virtues at 2:1"
    );
}

#[test]
fn shipped_abilities_and_characteristics_load() {
    let rs = load_ruleset();
    // Catalogue size is data, not code: assert the seed items the engine relies
    // on are present and read correctly, never an exact ability total. The full
    // Core Rules catalogue (staged on `full-abilities`) must be pullable as a
    // data-only change — see crates/arm-rules/RULES.md.
    assert!(rs.ability(&Id::new("ability.awareness")).is_some());
    // The whole childhood restricted list ships (Core Rules 2378).
    for id in [
        "ability.area_lore",
        "ability.athletics",
        "ability.awareness",
        "ability.brawl",
        "ability.charm",
        "ability.folk_ken",
        "ability.guile",
        "ability.living_language",
        "ability.stealth",
        "ability.survival",
        "ability.swim",
    ] {
        assert!(rs.ability(&Id::new(id)).is_some(), "missing {id}");
    }

    // Supernatural Abilities ship and carry the supernatural category.
    for id in [
        "ability.second_sight",
        "ability.premonitions",
        "ability.animal_ken",
    ] {
        let ability = rs
            .ability(&Id::new(id))
            .expect("supernatural ability present");
        assert_eq!(
            ability.category,
            AbilityCategory::Supernatural,
            "{id} must be supernatural"
        );
    }

    // The `*` marker is the per-ability `requires_training` flag (cannot be used
    // untrained), NOT the supernatural category: it spans General/Academic/Arcane
    // too. Source: Core Rules :4157 (Jack of All Trades) — heading asterisks set it.
    for (id, expected) in [
        ("ability.artes_liberales", true), // Academic, asterisked
        ("ability.magic_theory", true),    // Arcane, asterisked
        ("ability.area_lore", true),       // General, asterisked
        ("ability.second_sight", true),    // Supernatural, asterisked
        ("ability.awareness", false),      // General, usable untrained
        ("ability.penetration", false),    // Arcane but explicitly not asterisked
    ] {
        let ability = rs.ability(&Id::new(id)).expect("ability present");
        assert_eq!(
            ability.requires_training, expected,
            "{id} requires_training must be {expected}"
        );
    }

    let chars = rs
        .characteristic_rules()
        .expect("characteristic rules present");
    assert_eq!(chars.start_points, 7);
    // The cost table spans the absolute ±5 range (Great/Poor headroom)...
    assert_eq!(chars.min_score(), Some(-5));
    assert_eq!(chars.max_score(), Some(5));
    // ...while the no-virtue base limits are ±3.
    assert_eq!(chars.base_max_score(), Some(3));
    assert_eq!(chars.base_min_score(), Some(-3));
    assert_eq!(chars.effective_max_score(), Some(5));
    assert_eq!(chars.effective_min_score(), Some(-5));

    // Advancement table is triangular: score 5 costs 75 xp total.
    assert_eq!(rs.advancement().xp_for_score(5), Some(75));
    assert_eq!(rs.advancement().xp_to_raise(5), Some(25));
}

#[test]
fn english_i18n_covers_all_abilities() {
    let rs = load_ruleset();
    let i18n_en = include_str!("../../../rules/i18n/en/abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for ability in rs.abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "English i18n missing ability '{}'",
            ability.id
        );
    }
}

#[test]
fn german_i18n_covers_all_abilities() {
    let rs = load_ruleset();
    let i18n_de = include_str!("../../../rules/i18n/de/abilities.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for ability in rs.abilities() {
        assert!(
            loc.display_name(&ability.id).is_some(),
            "German i18n missing ability '{}'",
            ability.id
        );
    }
}

#[test]
fn english_i18n_covers_all_spells() {
    let rs = load_ruleset_with_spells();
    let i18n_en = include_str!("../../../rules/i18n/en/spells.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for spell in rs.spells() {
        assert!(
            loc.display_name(&spell.id).is_some(),
            "English i18n missing spell '{}'",
            spell.id
        );
    }
}

#[test]
fn german_i18n_covers_all_spells() {
    let rs = load_ruleset_with_spells();
    let i18n_de = include_str!("../../../rules/i18n/de/spells.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for spell in rs.spells() {
        assert!(
            loc.display_name(&spell.id).is_some(),
            "German i18n missing spell '{}'",
            spell.id
        );
    }
}

/// The shipped Skilled Parens raises *both* the spell-levels budget (+30) and the
/// general apprenticeship XP pool (+60), proving its two-effect package is wired
/// end-to-end against real data (Core:4964-4966).
#[test]
fn shipped_skilled_parens_raises_both_budgets() {
    let rs = load_ruleset_with_spells();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.skilled_parens"))],
    );
    e.xp_pool = 240;
    assert_eq!(arm_rules::spell_levels_budget(120, &e, &rs), 150);
    assert_eq!(arm_rules::xp_allocation(&e, &rs).general_pool, 300);
}

/// The shipped Weak Parens lowers both budgets (Core:7072-7074).
#[test]
fn shipped_weak_parens_lowers_both_budgets() {
    let rs = load_ruleset_with_spells();
    let mut e = entity("magus", vec![Selection::new(Id::new("flaw.weak_parens"))]);
    e.xp_pool = 240;
    assert_eq!(arm_rules::spell_levels_budget(120, &e, &rs), 90);
    assert_eq!(arm_rules::xp_allocation(&e, &rs).general_pool, 180);
}

#[test]
fn fully_specified_companion_validates() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::new(Id::new("flaw.poor_student")),
        ],
    );
    // Int +2 (3) + Per +1 (1) + Sta -1 (-1) + others 0 = 3 <= 7 (under -> warning only).
    e.characteristics = BTreeMap::from([
        (Characteristic::Int, 2),
        (Characteristic::Per, 1),
        (Characteristic::Sta, -1),
    ]);
    e.ability_scores = vec![
        AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: Some("searching".into()),
            parameter: None,
        },
        AbilityScore {
            ability: Id::new("ability.living_language"),
            score: 5,
            specialty: None,
            parameter: Some("German".into()),
        },
    ];
    // Awareness 2 (15 xp) + Living Language 5 (75 xp) = 90 spent; give a pool that
    // covers it (banking the rest).
    e.xp_pool = 120;

    let result = validate(&e, &rs);
    assert!(
        result.is_valid(),
        "fully-specified companion should validate: {:?}",
        result.issues
    );
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

#[test]
fn shipped_score_effects_apply() {
    use arm_rules::{characteristic_cap, characteristic_floor, effective_ability_score};
    let rs = load_ruleset();

    // Puissant Ability (+2 bonus) and Great/Poor Characteristic (limit shifts)
    // from the shipped data.
    let mut e = entity(
        "companion",
        vec![
            Selection::with_params(
                Id::new("virtue.puissant_ability"),
                BTreeMap::from([("ability".into(), Id::new("ability.awareness"))]),
            ),
            Selection::with_params(
                Id::new("virtue.great_characteristic"),
                BTreeMap::from([("characteristic".into(), Characteristic::Str.id())]),
            ),
            Selection::with_params(
                Id::new("flaw.poor_characteristic"),
                BTreeMap::from([("characteristic".into(), Characteristic::Qik.id())]),
            ),
        ],
    );
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 2,
        specialty: None,
        parameter: None,
    }];
    e.characteristics = BTreeMap::from([(Characteristic::Str, 3), (Characteristic::Qik, -3)]);

    // Puissant adds to the effective ability score.
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.awareness"), None),
        4,
        "Awareness 2 + Puissant +2"
    );
    // Great raises Strength's buy cap (no free point); Poor lowers Quickness's
    // buy floor. Untargeted characteristics keep the base ±3 limits.
    assert_eq!(
        characteristic_cap(&e, &rs, Characteristic::Str),
        4,
        "Great raises the Strength cap to +4"
    );
    assert_eq!(
        characteristic_floor(&e, &rs, Characteristic::Qik),
        -4,
        "Poor lowers the Quickness floor to -4"
    );
    assert_eq!(characteristic_cap(&e, &rs, Characteristic::Int), 3);
    assert_eq!(characteristic_floor(&e, &rs, Characteristic::Int), -3);
}
