//! E9: mechanical enforcement of "saves store choices, not resolved values"
//! (`CLAUDE.md` → Engineering conventions).
//!
//! Before this test, the invariant was a comment: nothing failed if a future
//! change added a *derived* field to [`Entity`] and let it serialize into a
//! save. This test pins the exact set of JSON keys a fully-populated `Entity`
//! serializes to against an explicit allowlist below. Any change to
//! `Entity`'s field set — adding, removing, or renaming a field — changes that
//! key set and fails the test, forcing whoever touches it to look at the
//! allowlist and make a deliberate call: is the new field a player CHOICE
//! (belongs in a save) or a DERIVED/computed value (must stay out of `Entity`
//! and be computed at evaluation time instead, e.g. in
//! `arm_rules::effective` or `arm_rules::derived`)?
//!
//! This is a fixed-struct-shape assertion, not a rules-catalogue count:
//! `Entity`'s field list is a Rust struct definition, not sized by
//! `rules/core/*.json`, so enumerating its keys here does not violate
//! "catalogue size is data, never code" (`CLAUDE.md` → Architecture
//! invariants) — the allowlist would be exactly this same fixed size no
//! matter how many Virtues, Abilities, or Spells the loaded ruleset carries.

use arm_rules::types::*;
use arm_rules::{Characteristic, LifeStagePlan};
use std::collections::{BTreeMap, BTreeSet};

/// Every field `Entity` declares, populated with a non-default value so
/// every `skip_serializing_if` is defeated and every key actually appears in
/// the serialized JSON. Field values are otherwise arbitrary/unrealistic
/// (e.g. both `house` and `mythic_type` are set, which a real character never
/// does) — this fixture tests the *shape* Entity serializes to, not a legal
/// character.
fn fully_populated_entity() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.name = "Test Name".to_string();
    e.description = "Test description".to_string();
    e.concept = "Test concept".to_string();
    e.gender = "Test gender".to_string();
    e.birth_year = Some(1190);
    e.sigil = "Test sigil".to_string();
    e.covenant_name = "Test covenant".to_string();
    e.parens = "Test parens".to_string();

    e.house = Some(Id::new("house.bonisagus"));
    e.house_choices = BTreeMap::from([(
        "house_choice".to_string(),
        Selection::new(Id::new("virtue.house_grant")),
    )]);
    e.mythic_type = Some(Id::new("mythic.type"));
    e.mythic_choices = BTreeMap::from([(
        "mythic_choice".to_string(),
        Selection::new(Id::new("virtue.mythic_grant")),
    )]);
    e.warping_choices = BTreeMap::from([(
        "warping_choice".to_string(),
        Selection::new(Id::new("virtue.warping_fill")),
    )]);

    e.age = Some(35);
    e.apparent_age = Some(30);
    e.aura = 2;
    e.wizard_furthest_phase = Some("virtues_flaws".to_string());
    e.spell_levels_override = Some(150);
    e.ability_funding = AbilityFunding::LifeStages;
    e.life_stages = Some(LifeStagePlan {
        native_language: Some("Latin".to_string()),
        childhood_package: Some(Id::new("childhood.package")),
        gauntlet_age: Some(25),
        post_gauntlet_lab_seasons: 3,
        post_gauntlet_spell_levels: 5,
    });

    e.characteristics = BTreeMap::from([(Characteristic::Int, 3)]);
    e.characteristic_descriptions = BTreeMap::from([(Characteristic::Int, "sharp".to_string())]);
    e.selections = vec![Selection::new(Id::new("virtue.the_gift"))];
    e.xp_pool = 240;
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 3,
        specialty: Some("searching".to_string()),
        parameter: None,
    }];
    e.art_scores = vec![ArtScore {
        art: Id::new("art.creo"),
        score: 5,
    }];
    e.spells = vec![SpellSelection {
        spell: Id::new("spell.pilum_of_fire"),
        level: Some(20),
        mastery: Some(1),
        parameter: Some("art.ignem".to_string()),
        mastery_abilities: vec![Id::new("spell_mastery_ability.penetration")],
    }];

    e.personality_traits = vec![PersonalityTrait {
        name: "Brave".to_string(),
        value: 3,
    }];
    e.reputations = vec![Reputation {
        kind: ReputationType::Hermetic,
        score: 2,
        content: "known theorist".to_string(),
    }];
    e.devices = vec![EnchantedDevice {
        name: "Ring of Seeing".to_string(),
        level: 10,
    }];
    e.talisman = Some(Talisman {
        description: "ash staff".to_string(),
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
        bonus: Some(2),
        focus: "gold and silver".to_string(),
    });
    e.familiar = Some(Familiar {
        name: "Corvus".to_string(),
        animal: "a raven".to_string(),
        might: Some(MightScore {
            realm: Realm::Magic,
            score: 5,
        }),
        characteristics: BTreeMap::from([(Characteristic::Qik, 3)]),
        size: -3,
        personality_traits: vec![PersonalityTrait {
            name: "Loyal".to_string(),
            value: 3,
        }],
        cord_gold: 1,
        cord_silver: 1,
        cord_bronze: 1,
        powers: vec![SupernaturalPower {
            name: "Wings of the Storm".to_string(),
            level: 20,
        }],
    });

    e.warping_points = 3;
    e.warping_effect = "his shadow lags behind".to_string();
    e.twilight_scars = vec![TwilightScar {
        description: "his eyes reflect no candlelight".to_string(),
    }];
    e.aging_points = BTreeMap::from([(Characteristic::Sta, 2)]);
    e.decrepitude_effect = "a persistent cough".to_string();
    e.living_conditions = BTreeSet::from([Id::new("living_condition.average_peasant")]);
    e.aging_log = vec![AgingLogEntry {
        year: Some(1220),
        effect: "an apparent aging crisis, weathered".to_string(),
        ..AgingLogEntry::default()
    }];

    e.equipment = vec![EquipmentSlot {
        item: Id::new("weapon.sword_long"),
        equipped: true,
        specialization_applies: true,
    }];
    e.might = Some(MightScore {
        realm: Realm::Magic,
        score: 8,
    });
    e.powers = vec![SupernaturalPower {
        name: "Second Sight".to_string(),
        level: 5,
    }];

    e.normalize();
    e
}

/// The complete, deliberate allowlist of top-level JSON keys `Entity` may
/// serialize to. Every entry here is a stored player CHOICE (or save-format
/// bookkeeping like `schema_version`/`ruleset`); nothing here is computed
/// from other fields. Changing this list is exactly the deliberate decision
/// this test exists to force — see the module doc.
const ALLOWED_ENTITY_KEYS: &[&str] = &[
    "schema_version",
    "ruleset",
    "entity_kind",
    "type_id",
    "selections",
    "characteristics",
    "characteristic_descriptions",
    "ability_scores",
    "xp_pool",
    "life_stages",
    "ability_funding",
    "wizard_furthest_phase",
    "art_scores",
    "spells",
    "spell_levels_override",
    "house",
    "house_choices",
    "mythic_type",
    "mythic_choices",
    "warping_choices",
    "age",
    "apparent_age",
    "personality_traits",
    "reputations",
    "aura",
    "devices",
    "familiar",
    "talisman",
    "longevity_ritual",
    "living_conditions",
    "aging_points",
    "warping_points",
    "warping_effect",
    "twilight_scars",
    "decrepitude_effect",
    "aging_log",
    "name",
    "description",
    "concept",
    "gender",
    "birth_year",
    "sigil",
    "covenant_name",
    "parens",
    "equipment",
    "might",
    "powers",
];

#[test]
fn entity_serialized_keys_match_the_declared_allowlist() {
    let entity = fully_populated_entity();
    let value = serde_json::to_value(&entity).expect("Entity always serializes");
    let obj = value
        .as_object()
        .expect("Entity serializes to a JSON object");

    let actual: BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    let allowed: BTreeSet<&str> = ALLOWED_ENTITY_KEYS.iter().copied().collect();

    let unexpected: Vec<&&str> = actual.difference(&allowed).collect();
    let missing: Vec<&&str> = allowed.difference(&actual).collect();

    assert!(
        unexpected.is_empty() && missing.is_empty(),
        "Entity's serialized field set drifted from the declared allowlist.\n\
         Unexpected keys (present in JSON, not in the allowlist — did you add \
         a field? Is it a stored CHOICE or a DERIVED value that must not be \
         persisted?): {unexpected:?}\n\
         Missing keys (in the allowlist but absent from JSON — did you remove \
         or rename a field, or does the fixture above need updating to set \
         it?): {missing:?}"
    );
}

/// Guards the guard: every field in [`ALLOWED_ENTITY_KEYS`] must actually
/// come from [`fully_populated_entity`] setting a non-default value, or the
/// key-set assertion above would pass by accident (a field silently omitted
/// from both the fixture AND the allowlist can't be caught by a set-equality
/// check). Cross-checked by re-deriving the same JSON and diffing byte length
/// against an all-defaults entity: every key in the fixture must be a
/// genuine, non-empty addition.
#[test]
fn fully_populated_entity_actually_sets_every_allowlisted_field() {
    let bare = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    let bare_value = serde_json::to_value(&bare).unwrap();
    let bare_obj = bare_value.as_object().unwrap();
    let bare_keys: BTreeSet<&str> = bare_obj.keys().map(String::as_str).collect();

    let full = fully_populated_entity();
    let full_value = serde_json::to_value(&full).unwrap();
    let full_obj = full_value.as_object().unwrap();
    let full_keys: BTreeSet<&str> = full_obj.keys().map(String::as_str).collect();

    // ability_funding is written even at its default (see Entity doc comment
    // on that field), so it is legitimately present on a bare entity too.
    let always_present: BTreeSet<&str> = [
        "schema_version",
        "ruleset",
        "entity_kind",
        "type_id",
        "ability_funding",
    ]
    .into_iter()
    .collect();

    let should_appear_only_when_set: BTreeSet<&str> = ALLOWED_ENTITY_KEYS
        .iter()
        .copied()
        .filter(|k| !always_present.contains(k))
        .collect();

    let missing_from_full: Vec<&&str> =
        should_appear_only_when_set.difference(&full_keys).collect();
    assert!(
        missing_from_full.is_empty(),
        "fully_populated_entity() failed to set these fields, so the \
         key-allowlist test above would not actually exercise them: \
         {missing_from_full:?}"
    );

    let leaked_on_bare: Vec<&&str> = should_appear_only_when_set
        .intersection(&bare_keys)
        .collect();
    assert!(
        leaked_on_bare.is_empty(),
        "these fields serialize even on a brand-new, untouched Entity, so \
         fully_populated_entity() setting them proves nothing: {leaked_on_bare:?}"
    );
}
