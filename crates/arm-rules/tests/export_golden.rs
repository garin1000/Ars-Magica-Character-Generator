//! Golden-fixture test for the Markdown character export (M5.6a).
//!
//! Renders a fully-populated magus against the **real shipped ruleset** — the same
//! `rules/core` + `rules/i18n/en` files the app loads — and compares the result byte
//! for byte with a checked-in fixture. So this covers what the unit tests in
//! `export.rs` cannot: that the formatter still produces a sane whole document when
//! the catalogue is the real one, and that the document does not drift unnoticed.
//!
//! The label map is **synthetic** (`key → key`), never parsed out of
//! `locales/en/main.ftl`: the engine links no Fluent parser, and a fixture built from
//! real English wording would churn on every copy tweak in the UI. What the fixture
//! pins is the document's *structure* and its *numbers*; the wording is 5.6c's
//! business.

use arm_rules::Characteristic;
use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::{BTreeMap, BTreeSet};

/// The whole shipped ruleset, localized with the shipped English rules text.
fn shipped_ruleset() -> LocalizedRuleset {
    let ruleset = Ruleset::from_sources(RulesetSources {
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
    })
    .expect("the shipped ruleset loads");
    LocalizedRuleset::from_merged(
        ruleset,
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
    .expect("the shipped English rules text loads")
}

/// Every declared chrome key resolved to itself — see the module docs for why the
/// fixture is deliberately not localized.
fn synthetic_labels() -> BTreeMap<String, String> {
    LABEL_KEYS
        .iter()
        .map(|key| ((*key).to_string(), (*key).to_string()))
        .collect()
}

/// A magus built entirely from **real** catalogue ids, touching every section the
/// formatter can emit.
fn golden_magus() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.name = "Marcus of Bonisagus".to_string();
    e.description = "A bookish theorist".to_string();
    e.concept = "Seeker after lost Hermetic lore".to_string();
    e.gender = "male".to_string();
    e.birth_year = Some(1194);
    e.sigil = "the smell of old parchment".to_string();
    e.covenant_name = "Semita Errabunda".to_string();
    e.parens = "Vittoria of Bonisagus".to_string();
    e.house = Some(Id::new("house.bonisagus"));
    e.age = Some(35);
    e.apparent_age = Some(30);
    e.aura = 3;
    for (characteristic, score) in [
        (Characteristic::Int, 3),
        (Characteristic::Per, 1),
        (Characteristic::Str, 1),
        (Characteristic::Sta, 2),
        (Characteristic::Pre, -1),
        (Characteristic::Com, 1),
        (Characteristic::Dex, 1),
        (Characteristic::Qik, 1),
    ] {
        e.characteristics.insert(characteristic, score);
    }
    e.characteristic_descriptions
        .insert(Characteristic::Int, "quick-witted".to_string());
    e.selections = vec![
        Selection::new(Id::new("virtue.the_gift")),
        Selection::new(Id::new("virtue.hermetic_magus")),
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.magic_theory"))]),
        ),
        Selection::with_params(
            Id::new("virtue.minor_magical_focus"),
            BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
        ),
        Selection::new(Id::new("virtue.warrior")),
        Selection::new(Id::new("flaw.blatant_gift")),
    ];
    // House Bonisagus grants a free Puissant Ability; the pick is Intrigue, distinct
    // from the point-bought Puissant Magic Theory above. Off-budget, so the sheet
    // lists it without it moving the balance.
    e.house_choices = BTreeMap::from([(
        "bonisagus_puissant".to_string(),
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.intrigue"))]),
        ),
    )]);
    e.xp_pool = 240;
    e.ability_scores = vec![
        AbilityScore {
            ability: Id::new("ability.area_lore"),
            score: 2,
            specialty: Some("legends".to_string()),
            parameter: Some("Provence".to_string()),
        },
        AbilityScore {
            ability: Id::new("ability.artes_liberales"),
            score: 1,
            specialty: None,
            parameter: None,
        },
        AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 2,
            specialty: Some("searching".to_string()),
            parameter: None,
        },
        AbilityScore {
            ability: Id::new("ability.magic_theory"),
            score: 4,
            specialty: None,
            parameter: None,
        },
        AbilityScore {
            ability: Id::new("ability.parma_magica"),
            score: 2,
            specialty: None,
            parameter: None,
        },
        AbilityScore {
            ability: Id::new("ability.single_weapon"),
            score: 4,
            specialty: Some("long sword".to_string()),
            parameter: None,
        },
    ];
    e.art_scores = vec![
        // Scores chosen so the whole spend fits the 240-point pool: 232 from the
        // general pool (Abilities 100 + Arts 117 + Spell Mastery 15) plus Single
        // Weapon's 50 from Warrior's Martial-only pool.
        ArtScore {
            art: Id::new("art.creo"),
            score: 8,
        },
        ArtScore {
            art: Id::new("art.rego"),
            score: 5,
        },
        ArtScore {
            art: Id::new("art.corpus"),
            score: 5,
        },
        ArtScore {
            art: Id::new("art.ignem"),
            score: 8,
        },
        ArtScore {
            art: Id::new("art.vim"),
            score: 5,
        },
    ];
    e.spells = vec![SpellSelection {
        spell: Id::new("spell.pilum_of_fire"),
        level: None,
        mastery: Some(2),
        parameter: None,
        mastery_abilities: vec![Id::new("spell_mastery_ability.penetration")],
    }];
    e.equipment = vec![
        EquipmentSlot {
            item: Id::new("weapon.sword_long"),
            equipped: true,
            specialization_applies: true,
        },
        EquipmentSlot {
            item: Id::new("shield.round"),
            equipped: true,
            specialization_applies: false,
        },
        EquipmentSlot {
            item: Id::new("armor.leather_scale_partial"),
            equipped: false,
            specialization_applies: false,
        },
    ];
    e.personality_traits = vec![
        PersonalityTrait {
            name: "Curious".to_string(),
            value: 3,
        },
        PersonalityTrait {
            name: "Brash".to_string(),
            value: -2,
        },
    ];
    e.reputations = vec![Reputation {
        kind: ReputationType::Hermetic,
        score: 2,
        content: "a promising theoretician".to_string(),
    }];
    e.devices = vec![EnchantedDevice {
        name: "Ring of Seeing".to_string(),
        level: 15,
    }];
    e.talisman = Some(Talisman {
        description: "an ash staff shod with silver".to_string(),
        attunements: vec![TalismanAttunement {
            description: "to ward off flame".to_string(),
            bonus: 3,
        }],
        effects: vec![TalismanEffect {
            name: "Lamp Without Flame".to_string(),
            level: 10,
        }],
    });
    e.longevity_ritual = Some(LongevityRitual {
        source: LongevitySource::SelfMade,
        bonus: Some(7),
        focus: "a draught of gold and silver".to_string(),
    });
    e.familiar = Some(Familiar {
        name: "Corvus".to_string(),
        animal: "a raven".to_string(),
        might: Some(MightScore {
            realm: Realm::Magic,
            score: 10,
        }),
        characteristics: BTreeMap::from([(Characteristic::Int, 2), (Characteristic::Qik, 4)]),
        size: -4,
        personality_traits: vec![PersonalityTrait {
            name: "Loyal".to_string(),
            value: 3,
        }],
        cord_gold: 2,
        cord_silver: 1,
        cord_bronze: 0,
        powers: vec![SupernaturalPower {
            name: "Wings of the Storm".to_string(),
            level: 20,
        }],
    });
    e.warping_points = 6;
    e.warping_effect = "his shadow lags a heartbeat behind".to_string();
    e.twilight_scars = vec![TwilightScar {
        description: "his eyes reflect no candlelight".to_string(),
    }];
    e.aging_points = BTreeMap::from([(Characteristic::Pre, 5)]);
    e.decrepitude_effect = "a persistent cough each winter".to_string();
    // Two cumulative rows off the shipped table, so the fixture pins that a stored
    // choice reaches the sheet through the rules i18n rather than as its slug.
    e.living_conditions = BTreeSet::from([
        Id::new("living_condition.work_in_a_mine"),
        Id::new("living_condition.live_in_a_leper_colony"),
    ]);
    // One hand-written year and one the engine resolved, so the fixture covers both
    // the free-text entry and the widened one carrying its die and total.
    e.aging_log = vec![
        AgingLogEntry {
            year: Some(1220),
            effect: "an apparent aging crisis, weathered".to_string(),
            ..AgingLogEntry::default()
        },
        AgingLogEntry {
            year: Some(1229),
            age: Some(35),
            die: Some(9),
            total: Some(13),
            living_conditions: BTreeSet::from([Id::new("living_condition.work_in_a_mine")]),
            points: BTreeMap::from([(Characteristic::Pre, 5)]),
            crisis: true,
            ..AgingLogEntry::default()
        },
    ];
    e.normalize();
    e
}

#[test]
fn a_fully_populated_magus_matches_the_golden_document() {
    let rendered = character_markdown(&golden_magus(), &shipped_ruleset(), &synthetic_labels());
    let expected = include_str!("fixtures/magus_export.md");
    assert_eq!(
        rendered, expected,
        "the rendered document drifted from tests/fixtures/magus_export.md"
    );
}

#[test]
fn rendering_the_same_magus_twice_is_byte_identical() {
    let ruleset = shipped_ruleset();
    let entity = golden_magus();
    let labels = synthetic_labels();
    assert_eq!(
        character_markdown(&entity, &ruleset, &labels),
        character_markdown(&entity, &ruleset, &labels)
    );
}
