use arm_rules::AbilityCategory;
use arm_rules::Characteristic;
use arm_rules::effective_art_score;
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
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
    })
    .unwrap()
}

/// The full shipped ruleset including the equipment catalogue — the equipment
/// tests need Abilities loaded so each weapon's combat Ability resolves.
fn load_ruleset_with_equipment() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: None,
        mythic_types: None,
        spells: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
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

#[test]
fn shipped_equipment_loads_and_exposes_accessors() {
    let rs = load_ruleset_with_equipment();
    // Catalogue size is data, not code: assert representative items are present
    // and read correctly, never exact totals.
    let sword = rs
        .weapon(&Id::new("weapon.sword_long"))
        .expect("weapon.sword_long present");
    assert_eq!(sword.attack_mod, Some(4));
    assert_eq!(sword.damage_mod, Some(6));
    assert_eq!(sword.ability, Id::new("ability.single_weapon"));
    assert!(sword.range.is_none(), "melee weapon has no range");
    // A missile weapon carries a Range and uses Bows.
    let bow = rs
        .weapon(&Id::new("weapon.bow_long"))
        .expect("weapon.bow_long present");
    assert_eq!(bow.range, Some(30));
    assert_eq!(bow.ability, Id::new("ability.bows"));
    // Dodge has no attack/damage/min-Strength (n/a cells).
    let dodge = rs.weapon(&Id::new("weapon.dodge")).expect("dodge present");
    assert_eq!(dodge.attack_mod, None);
    assert_eq!(dodge.min_strength, None);
    // Shields and armor resolve through their own accessors.
    assert_eq!(rs.shield(&Id::new("shield.heater")).unwrap().defense_mod, 3);
    assert_eq!(
        rs.armor_item(&Id::new("armor.chain_mail_full"))
            .unwrap()
            .protection,
        9
    );
    assert!(rs.weapon_count() > 0 && rs.shield_count() > 0 && rs.armor_count() > 0);
}

/// A weapon whose combat `ability` names something that is neither Martial nor
/// Brawl is rejected at load (referential-integrity trust gate).
#[test]
fn weapon_with_non_combat_ability_rejected_at_load() {
    let equipment = r#"{ "weapons": [
      { "id": "weapon.bad", "kind": "melee", "init_mod": 0, "defense_mod": 0,
        "load": 1, "ability": "ability.awareness" }
    ] }"#;
    let err = Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: None,
        houses: None,
        mythic_types: None,
        spells: None,
        equipment: Some(equipment),
        characteristics: None,
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("not a combat Ability"),
        "expected combat-ability rejection, got: {err}"
    );
}

/// A weapon naming a wholly unknown ability id is also rejected at load.
#[test]
fn weapon_with_unknown_ability_rejected_at_load() {
    let equipment = r#"{ "weapons": [
      { "id": "weapon.bad", "kind": "melee", "init_mod": 0, "defense_mod": 0,
        "load": 1, "ability": "ability.nonexistent" }
    ] }"#;
    let err = Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: None,
        houses: None,
        mythic_types: None,
        spells: None,
        equipment: Some(equipment),
        characteristics: None,
    })
    .unwrap_err();
    assert!(
        err.to_string().contains("unknown ability"),
        "expected unknown-ability rejection, got: {err}"
    );
}

#[test]
fn english_i18n_covers_all_equipment() {
    let rs = load_ruleset_with_equipment();
    let i18n_en = include_str!("../../../rules/i18n/en/equipment.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_en).unwrap();
    for id in rs
        .weapons()
        .map(|w| &w.id)
        .chain(rs.shields().map(|s| &s.id))
        .chain(rs.armor().map(|a| &a.id))
    {
        assert!(
            loc.display_name(id).is_some(),
            "English i18n missing equipment '{id}'"
        );
    }
}

#[test]
fn german_i18n_covers_all_equipment() {
    let rs = load_ruleset_with_equipment();
    let i18n_de = include_str!("../../../rules/i18n/de/equipment.json");
    let loc = LocalizedRuleset::new(rs.clone(), i18n_de).unwrap();
    for id in rs
        .weapons()
        .map(|w| &w.id)
        .chain(rs.shields().map(|s| &s.id))
        .chain(rs.armor().map(|a| &a.id))
    {
        assert!(
            loc.display_name(id).is_some(),
            "German i18n missing equipment '{id}'"
        );
    }
}

/// An [`EquipmentSlot`] referencing an unknown catalogue id errors in validate;
/// an over-heavy equipped weapon warns (advisory, non-blocking); normalize sorts.
#[test]
fn validate_equipment_unknown_ref_and_min_strength() {
    let rs = load_ruleset_with_equipment();
    let mut e = entity("grog", vec![]);
    // Unknown id → error.
    e.equipment = vec![EquipmentSlot {
        item: Id::new("weapon.nonexistent"),
        equipped: true,
    }];
    let result = validate(&e, &rs);
    assert!(
        result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "unknown equipment id must error"
    );

    // A Warhammer (min-Strength +2) equipped by a Strength −1 grog warns, not errors.
    e.characteristics.insert(Characteristic::Str, -1);
    e.equipment = vec![EquipmentSlot {
        item: Id::new("weapon.warhammer"),
        equipped: true,
    }];
    let result = validate(&e, &rs);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "equipment_min_strength"
                && i.severity == arm_rules::IssueSeverity::Warning),
        "over-heavy equipped weapon must warn"
    );
    assert!(
        !result.issues.iter().any(|i| i.code == "unknown_equipment"),
        "a known weapon must not report unknown_equipment"
    );
}

#[test]
fn normalize_sorts_equipment() {
    let mut e = entity("grog", vec![]);
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.warhammer"),
            equipped: false,
        },
        EquipmentSlot {
            item: Id::new("armor.chain_mail_full"),
            equipped: true,
        },
    ];
    e.normalize();
    assert_eq!(e.equipment[0].item, Id::new("armor.chain_mail_full"));
    assert_eq!(e.equipment[1].item, Id::new("weapon.warhammer"));
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

/// The shipped Elemental Magic (Core:3731-3737) redistributes Art-XP over the four
/// elemental Forms against the real Arts catalogue: each Form gains half (rounded
/// up) of every other Form's table-XP. With Ignem/Auram/Terram at score 6 (21 XP)
/// and Aquam at score 4 (10 XP), the boosted effective scores are 9/9/9 and 8,
/// while a non-elemental Form (Corpus) at score 6 is untouched.
#[test]
fn shipped_elemental_magic_redistributes_art_xp() {
    let rs = load_ruleset_with_spells();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.elemental_magic"))],
    );
    let row = |art: &str, score: u8| ArtScore {
        art: Id::new(art),
        score,
    };
    e.art_scores = vec![
        row("art.aquam", 4),
        row("art.auram", 6),
        row("art.ignem", 6),
        row("art.terram", 6),
        row("art.corpus", 6),
    ];
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.aquam")), 8);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.auram")), 9);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.ignem")), 9);
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.terram")), 9);
    // A non-elemental Form is never touched by the redistribution.
    assert_eq!(effective_art_score(&e, &rs, &Id::new("art.corpus")), 6);
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

/// Acceptance criterion for M5 slice 5a: every shipped Virtue/Flaw carries a
/// `classification`. The field is required (no serde default), so an unclassified
/// entry would already fail `load_ruleset()`; this test additionally asserts the
/// catalogue is non-trivial and that all three classes are actually used, so the
/// classification pass can never silently collapse to a single bucket.
#[test]
fn every_vf_is_classified() {
    let rs = load_ruleset();
    let mut narrative = 0usize;
    let mut creation = 0usize;
    let mut in_play = 0usize;
    for item in rs.items() {
        match item.classification {
            Classification::Narrative => narrative += 1,
            Classification::CreationEffect => creation += 1,
            Classification::InPlayEffect => in_play += 1,
        }
    }
    // Catalogue size is data, not code: assert only that the catalogue is large
    // and every class is represented, never exact per-class totals.
    assert!(
        narrative + creation + in_play > 600,
        "expected the full V/F catalogue to load"
    );
    assert!(narrative > 0, "some V/F must be narrative");
    assert!(creation > 0, "some V/F must be creation_effect");
    assert!(in_play > 0, "some V/F must be in_play_effect");
    // An entry carrying `effects` is mechanical, never narrative: it either
    // changes a creation number (creation_effect) or modifies an in-play/derived
    // total (in_play_effect, M5/5b). Narrative items are never given an effect.
    for item in rs.items() {
        if !item.effects.is_empty() {
            assert_ne!(
                item.classification,
                Classification::Narrative,
                "{} carries effects so must not be narrative",
                item.id
            );
        }
    }
    // M5/5b acceptance: every in_play_effect V/F is wired to at least one
    // derived-total Effect variant (the 93-item in-play audit is fully wired).
    for item in rs.items() {
        if item.classification == Classification::InPlayEffect {
            assert!(
                !item.effects.is_empty(),
                "{} is in_play_effect but carries no effect",
                item.id
            );
        }
    }
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

// --- M5 slice 5b: in-play effect variants ---

/// Helper: the set of issue codes `validate` emits for an entity.
fn issue_codes(entity: &Entity, rs: &Ruleset) -> Vec<String> {
    validate(entity, rs)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

/// A magus may hold at most one Magical Focus (Core Rules.md:4542): two Minor
/// Foci (distinct descriptors, so not a duplicate selection) trip the
/// `multiple_magical_foci` rule, which counts the `MagicalFocus` effect rather
/// than relying on pairwise incompatibility (which cannot catch two Minors).
#[test]
fn two_magical_foci_are_rejected() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![
            Selection::with_params(
                Id::new("virtue.minor_magical_focus"),
                BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
            ),
            Selection::with_params(
                Id::new("virtue.minor_magical_focus"),
                BTreeMap::from([("focus".into(), Id::new("weather"))]),
            ),
        ],
    );
    assert!(
        issue_codes(&e, &rs).contains(&"multiple_magical_foci".to_string()),
        "two foci must be rejected"
    );
}

/// A single Magical Focus is legal — the one-focus rule does not fire.
#[test]
fn one_magical_focus_is_allowed() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
        )],
    );
    assert!(
        !issue_codes(&e, &rs).contains(&"multiple_magical_foci".to_string()),
        "one focus must be allowed"
    );
}

/// Deficient Form's parameter is Form-domain, so targeting a Technique (art.creo)
/// fails parameter resolution — the art-class restriction the slice requires.
#[test]
fn deficient_form_cannot_target_a_technique() {
    let rs = load_ruleset_with_spells();
    let e = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.creo"))]),
        )],
    );
    assert!(
        issue_codes(&e, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Form targeting a Technique must be rejected"
    );
    // A Form target (art.ignem) resolves cleanly.
    let ok = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        )],
    );
    assert!(
        !issue_codes(&ok, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Form targeting a Form must be accepted"
    );
}

/// Deficient Technique's parameter is Technique-domain, so targeting a Form
/// (art.ignem) fails; a Technique (art.creo) resolves.
#[test]
fn deficient_technique_cannot_target_a_form() {
    let rs = load_ruleset_with_spells();
    let bad = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("technique".into(), Id::new("art.ignem"))]),
        )],
    );
    assert!(
        issue_codes(&bad, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Technique targeting a Form must be rejected"
    );
    let ok = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.deficient_technique"),
            BTreeMap::from([("technique".into(), Id::new("art.creo"))]),
        )],
    );
    assert!(
        !issue_codes(&ok, &rs).contains(&"unknown_param_value".to_string()),
        "Deficient Technique targeting a Technique must be accepted"
    );
}

/// In-play effects (Tough/Soak, Method Caster, Enduring Constitution, a Magical
/// Focus, Deficient Form) never perturb creation-legality totals: adding them
/// leaves the XP allocation, characteristic caps, and effective ability scores
/// exactly as they were. They cost/grant only the point-balance their magnitude
/// dictates, computed elsewhere.
#[test]
fn in_play_effects_do_not_perturb_creation_totals() {
    use arm_rules::{characteristic_cap, effective_ability_score, xp_allocation};
    let rs = load_ruleset_with_spells();

    let mut base = entity("magus", vec![]);
    base.xp_pool = 15;
    base.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 3,
        specialty: None,
        parameter: None,
    }];
    base.characteristics = BTreeMap::from([(Characteristic::Int, 2)]);

    let mut with_effects = base.clone();
    with_effects.selections = vec![
        Selection::new(Id::new("virtue.tough")),
        Selection::new(Id::new("virtue.method_caster")),
        Selection::new(Id::new("virtue.enduring_constitution")),
        Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            BTreeMap::from([("focus".into(), Id::new("necromancy"))]),
        ),
        Selection::with_params(
            Id::new("flaw.deficient_form"),
            BTreeMap::from([("form".into(), Id::new("art.ignem"))]),
        ),
    ];

    assert_eq!(
        xp_allocation(&base, &rs).total_demand,
        xp_allocation(&with_effects, &rs).total_demand,
        "in-play effects must not change XP demand"
    );
    assert_eq!(
        characteristic_cap(&base, &rs, Characteristic::Int),
        characteristic_cap(&with_effects, &rs, Characteristic::Int),
        "in-play effects must not change characteristic caps"
    );
    assert_eq!(
        effective_ability_score(&base, &rs, &Id::new("ability.awareness"), None),
        effective_ability_score(&with_effects, &rs, &Id::new("ability.awareness"), None),
        "in-play effects must not change effective ability scores"
    );
}

// --- M5 slice 5a-wire: creation-effect wiring on the shipped catalogue ---

/// Helper: does an entity's reputation set raise `reputation_not_granted`?
fn reputation_ungranted(entity: &Entity, rs: &Ruleset) -> bool {
    issue_codes(entity, rs).contains(&"reputation_not_granted".to_string())
}

/// Builds a companion holding a single virtue/flaw (no params) plus one
/// player-declared Reputation of `kind`/`score`, to check the grant authorizes it.
fn companion_with_reputation(item: &str, kind: ReputationType, score: u8) -> Entity {
    let mut e = entity("companion", vec![Selection::new(Id::new(item))]);
    e.reputations = vec![Reputation {
        kind,
        score,
        content: "test".into(),
    }];
    e
}

#[test]
fn shipped_reputation_granters_authorize_their_kind() {
    let rs = load_ruleset();
    // Hermetic Prestige → a Hermetic Reputation at 4 (Core:4071-4073).
    let hp = companion_with_reputation("virtue.hermetic_prestige", ReputationType::Hermetic, 4);
    assert!(
        !reputation_ungranted(&hp, &rs),
        "Hermetic Prestige grants Hermetic"
    );
    // Baccalaureus → an Academic Reputation (Core:3472).
    let bac = companion_with_reputation("virtue.baccalaureus", ReputationType::Academic, 1);
    assert!(
        !reputation_ungranted(&bac, &rs),
        "Baccalaureus grants Academic"
    );
    // A Local reputation is NOT authorized by Hermetic Prestige alone.
    let wrong = companion_with_reputation("virtue.hermetic_prestige", ReputationType::Local, 4);
    assert!(
        reputation_ungranted(&wrong, &rs),
        "Hermetic Prestige does not grant Local"
    );
}

#[test]
fn shipped_famous_authorizes_any_reputation_kind() {
    let rs = load_ruleset();
    // Famous (Core:3861-3863): player chooses the type — any single type is legal.
    for kind in ReputationType::ALL {
        let e = companion_with_reputation("virtue.famous", kind, 4);
        assert!(
            !reputation_ungranted(&e, &rs),
            "Famous authorizes a {kind} Reputation",
        );
    }
    // But only ONE: two reputations exceed the single wildcard grant.
    let mut two = companion_with_reputation("virtue.famous", ReputationType::Local, 4);
    two.reputations.push(Reputation {
        kind: ReputationType::Hermetic,
        score: 4,
        content: "second".into(),
    });
    assert!(
        reputation_ungranted(&two, &rs),
        "Famous grants only one Reputation"
    );
}

#[test]
fn shipped_supernatural_virtues_grant_starting_score() {
    use arm_rules::effective_ability_score;
    let rs = load_ruleset();
    for (item, ability) in [
        ("virtue.animal_ken", "ability.animal_ken"),
        ("virtue.shapeshifter", "ability.shapeshifter"),
        ("virtue.enchanting_ability", "ability.enchanting"),
        ("virtue.wilderness_sense", "ability.wilderness_sense"),
    ] {
        let e = entity("companion", vec![Selection::new(Id::new(item))]);
        assert_eq!(
            effective_ability_score(&e, &rs, &Id::new(ability), None),
            1,
            "{item} grants {ability} at 1",
        );
    }
    // Strong Faerie Blood grants the Second Sight *Virtue* for free (Core:5038),
    // which in turn floors Second Sight at 1.
    let sfb = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.strong_faerie_blood"))],
    );
    assert_eq!(
        effective_ability_score(&sfb, &rs, &Id::new("ability.second_sight"), None),
        1,
        "Strong Faerie Blood grants Second Sight via a nested Virtue grant",
    );
}

#[test]
fn shipped_xp_granters_add_restricted_pool() {
    use arm_rules::restricted_xp_pools;
    let rs = load_ruleset();
    // Arcane Lore → +50 XP restricted to Arcane abilities (Core:3432).
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.arcane_lore"))],
    );
    let pools = restricted_xp_pools(&e, &rs);
    assert!(
        pools
            .iter()
            .any(|p| p.amount == 50 && p.categories.contains(&AbilityCategory::Arcane)),
        "Arcane Lore grants a 50-xp Arcane-restricted pool",
    );
    // Feral Upbringing → 120 XP on a fixed ability list (Core:6112).
    let fu = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.feral_upbringing"))],
    );
    assert!(
        restricted_xp_pools(&fu, &rs)
            .iter()
            .any(|p| p.amount == 120),
        "Feral Upbringing grants a 120-xp restricted pool",
    );
}

#[test]
fn shipped_confidence_true_faith_and_size_granters() {
    use arm_rules::{confidence, size, true_faith};
    let rs = load_ruleset();
    // Ferocity → +1 Confidence Score / +3 Points (Core:3875) over the base.
    let fer = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.ferocity"))],
    );
    assert_eq!(confidence(1, 3, &fer, &rs), (2, 6), "Ferocity adds 1/3");
    // Low Self-Esteem → removes the standard 1/3 Confidence (Core:6364).
    let lse = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.low_self_esteem"))],
    );
    assert_eq!(
        confidence(1, 3, &lse, &rs),
        (0, 0),
        "Low Self-Esteem zeroes Confidence"
    );
    // Relic → True Faith 1 (Core:4854); Powerful Relic → 3 (Core:4783).
    let relic = entity("companion", vec![Selection::new(Id::new("virtue.relic"))]);
    assert_eq!(true_faith(&relic, &rs), 1);
    let prelic = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.powerful_relic"))],
    );
    assert_eq!(true_faith(&prelic, &rs), 3);
    // Blood of the Nephilim → Size +1 (Divine:1945).
    let bon = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.blood_of_the_nephilim"))],
    );
    assert_eq!(
        size(&bon, &rs),
        1,
        "Blood of the Nephilim raises Size to +1"
    );
}
