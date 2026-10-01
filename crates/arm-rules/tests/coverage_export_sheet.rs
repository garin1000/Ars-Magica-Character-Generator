//! Coverage slice (D76 follow-up): exercises `export/sections.rs`,
//! `export/magic.rs` and `export/resolve.rs` code paths `export_golden.rs`'s
//! single richly-populated fixture never reaches.
//!
//! `export_golden.rs`'s `golden_magus` is deliberately stuffed with every
//! field so the fixture touches every "something to show" branch of the
//! renderer. That leaves its mirror image untested: what the sheet looks like
//! for an entity with *nothing* in a given slot (the "omit the section"
//! branches), a handful of real but rare shapes (a claimed-but-blank
//! talisman, a being's own Supernatural Powers, an unrolled-severity Crisis
//! row, a taxonomy-domain Virtue/Flaw parameter, a `multi_ref` parameter
//! value) that the golden fixture happens not to carry.
//!
//! Duplicates `export_golden.rs`'s small `shipped_ruleset`/`synthetic_labels`
//! helpers rather than importing them, so this file stays self-contained and
//! two coverage/review passes touching export tests concurrently do not
//! collide on one shared file.

use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::{BTreeMap, BTreeSet};

/// The whole shipped ruleset, localized with the shipped English rules text.
/// Mirrors `export_golden.rs::shipped_ruleset`.
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
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
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

/// Every declared chrome key resolved to itself, plus the catalogue-derived
/// families `export.rs` excludes from [`LABEL_KEYS`]. Mirrors
/// `export_golden.rs::synthetic_labels`.
fn synthetic_labels() -> BTreeMap<String, String> {
    let rs = shipped_ruleset();
    let mut keys: BTreeSet<String> = LABEL_KEYS.iter().map(|k| k.to_string()).collect();
    for profile in rs.ruleset.profiles() {
        keys.insert(format!("type-{}", profile.id));
    }
    for item in rs.ruleset.items() {
        for category in &item.categories {
            keys.insert(format!("category-{category}"));
        }
        for param in &item.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    for ability in rs.ruleset.abilities() {
        if let Some(parameter) = &ability.parameter {
            keys.insert(format!("param-label-{parameter}"));
        }
    }
    for spell in rs.ruleset.spells() {
        for param in &spell.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    keys.into_iter().map(|k| (k.clone(), k)).collect()
}

fn bare_entity(kind: EntityKind, type_id: &str) -> Entity {
    let mut e = Entity::new(
        kind,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.normalize();
    e
}

/// A freshly created character (every collection empty, every optional field
/// unset) renders none of the "something to show" sections — only the title
/// and, because it is still a Character, the body-constant Fatigue/Wound
/// tracks. The grog profile is used deliberately: it is the one shipped
/// profile that grants NO baseline Confidence at all (`ArMDE:1161/:2522`,
/// see `confidence()`'s own doc comment), so this also exercises
/// `write_confidence`'s all-zero early return — unreachable from any other
/// profile, which all grant a non-zero baseline.
///
/// Exercises the early-return branches in `write_identity`,
/// `write_characteristics`, `write_virtues_flaws`, `write_abilities`,
/// `write_arts`, `write_spells`, `write_equipment`, `write_combat`,
/// `write_soak`, `write_encumbrance`, `write_personality_traits`,
/// `write_reputations`, `write_confidence`, `write_supernatural`,
/// `write_magic_items` and `write_annotations` — none of which
/// `export_golden.rs`'s fully-populated fixture ever takes, since it sets a
/// value in every one of these slots.
#[test]
fn a_freshly_created_character_omits_every_optional_section() {
    let entity = bare_entity(EntityKind::Character, "grog");
    let rendered = character_markdown(&entity, &shipped_ruleset(), &synthetic_labels())
        .expect("a bare entity with real ids still resolves every chrome key");

    for absent_key in [
        "identity-label",
        "characteristics-title",
        "tab-virtues-flaws",
        "abilities-title",
        "tab-arts",
        "tab-spells",
        "tab-equipment",
        "derived-section-combat",
        "derived-section-soak",
        "derived-section-encumbrance",
        "personality-label",
        "reputations-label",
        "confidence-label",
        "tab-supernatural",
        "tab-possessions",
        "aging-label",
    ] {
        assert!(
            !rendered.contains(absent_key),
            "expected no {absent_key} section for a bare entity, got:\n{rendered}"
        );
    }
    // Still a Character: the Fatigue/Wound tracks are body constants, not
    // gated on emptiness or on the profile.
    assert!(rendered.contains("derived-section-fatigue"));
    assert!(rendered.contains("derived-section-wounds"));
}

/// A Covenant entity whose `type_id` names no character-type profile at all
/// (the ordinary case: covenants are not profiled in
/// `character_types.json`) shows neither Confidence (profile lookup fails,
/// `write_confidence`'s own early return at a *different* point than the
/// all-zero-Confidence case above) nor the Fatigue/Wound tracks
/// (`write_health_tracks`'s `entity_kind != Character` gate).
#[test]
fn a_covenant_entity_has_neither_confidence_nor_health_tracks() {
    let entity = bare_entity(EntityKind::Covenant, "covenant");
    let mut labels = synthetic_labels();
    labels.insert("type-covenant".to_string(), "type-covenant".to_string());
    let rendered = character_markdown(&entity, &shipped_ruleset(), &labels)
        .expect("a bare entity with real ids still resolves every chrome key");

    assert!(!rendered.contains("confidence-label"));
    assert!(!rendered.contains("derived-section-fatigue"));
    assert!(!rendered.contains("derived-section-wounds"));
}

/// A talisman the player has claimed but left otherwise blank (no
/// description, no attunements, no effects — exactly `Talisman::default()`)
/// renders no Talisman subsection at all, rather than an empty heading.
/// `golden_magus`'s talisman always carries a description and an effect, so
/// this all-empty shape is never exercised there.
#[test]
fn a_claimed_but_blank_talisman_renders_no_talisman_section() {
    let mut entity = bare_entity(EntityKind::Character, "magus");
    entity.talisman = Some(Talisman::default());
    entity.normalize();
    let rendered = character_markdown(&entity, &shipped_ruleset(), &synthetic_labels())
        .expect("a bare entity with real ids still resolves every chrome key");

    assert!(
        !rendered.contains("talisman-label"),
        "a blank talisman should not open its own section, got:\n{rendered}"
    );
}

/// The character's OWN Supernatural Powers — as opposed to a familiar's
/// invested powers, which `golden_magus` does carry — render with the
/// three-column Name/Level/Penetration table `power_rows` builds, never the
/// familiar's two-column shape. No existing export test sets
/// `Entity::powers` at all.
#[test]
fn the_characters_own_supernatural_powers_render_with_penetration() {
    let mut entity = bare_entity(EntityKind::Character, "magus");
    entity.powers = vec![SupernaturalPower {
        name: "Gift of Fiery Breath".to_string(),
        level: 20,
        penetration: 5,
    }];
    entity.normalize();
    let rendered = character_markdown(&entity, &shipped_ruleset(), &synthetic_labels())
        .expect("a bare entity with real ids still resolves every chrome key");

    assert!(rendered.contains("tab-supernatural"));
    assert!(rendered.contains("supernatural-powers-label"));
    assert!(rendered.contains("Gift of Fiery Breath"));
    assert!(rendered.contains("20"));
    assert!(rendered.contains("power-penetration-label"));
}

/// A resolved Crisis whose row carries no severity rank (`crisis.bedridden_week`
/// — "a week in bed is time, not an illness", per `aging_log_crisis`'s own
/// doc comment) prints the row's bare name, with none of the
/// `crisis-severity-<slug>` text a ranked illness would add. Every aging-log
/// entry in `golden_magus` that carries a `crisis_row` also carries a
/// `crisis_severity`, so the no-severity arm is otherwise never reached.
#[test]
fn an_unranked_crisis_row_prints_without_a_severity() {
    let mut entity = bare_entity(EntityKind::Character, "grog");
    entity.aging_log = vec![AgingLogEntry {
        year: Some(1200),
        age: Some(20),
        crisis: true,
        crisis_row: Some(Id::new("crisis.bedridden_week")),
        crisis_severity: None,
        ..AgingLogEntry::default()
    }];
    entity.normalize();
    let rendered = character_markdown(&entity, &shipped_ruleset(), &synthetic_labels())
        .expect("a bare entity with real ids still resolves every chrome key");

    assert!(rendered.contains("Bedridden for a week"));
    assert!(
        !rendered.contains("crisis-severity-"),
        "an unranked Crisis row must not print a severity label, got:\n{rendered}"
    );
}

/// A Virtue/Flaw parameter whose *domain* is a fixed engine taxonomy — not a
/// catalogue id and not free text — is labelled through its own `category-`
/// / `ability-category-` Fluent family (`taxonomy_label`), never printed as
/// raw text. `golden_magus`'s own parameterized selections (Puissant
/// Ability's `ability`, Magical Focus's `focus`) are Ability/Text domain, so
/// neither the `Category` nor the `AbilityCategory` arm is reached by the
/// golden fixture.
#[test]
fn a_category_domain_virtue_flaw_parameter_renders_through_its_taxonomy_label() {
    let mut entity = bare_entity(EntityKind::Character, "companion");
    entity.selections = vec![Selection {
        item_ref: Id::new("flaw.curse_of_slander"),
        params: BTreeMap::from([(
            "taken_as".to_string(),
            SelectionParamValue::Single(Id::new("general")),
        )]),
    }];
    entity.normalize();
    let rendered = character_markdown(&entity, &shipped_ruleset(), &synthetic_labels())
        .expect("a bare entity with real ids still resolves every chrome key");

    assert!(
        rendered.contains("category-general"),
        "a Category-domain parameter value should resolve through `category-<id>`, got:\n{rendered}"
    );
}

/// Same shape for the `AbilityCategory` domain (the closed 5-member
/// taxonomy), which is otherwise never exercised through the full export
/// pipeline at all.
#[test]
fn an_ability_category_domain_parameter_renders_through_its_taxonomy_label() {
    let mut entity = bare_entity(EntityKind::Character, "companion");
    entity.selections = vec![Selection {
        item_ref: Id::new("flaw.ability_block"),
        params: BTreeMap::from([
            (
                "scope".to_string(),
                SelectionParamValue::Single(Id::new("scope.category")),
            ),
            (
                "class".to_string(),
                SelectionParamValue::Single(Id::new("ability_category.martial")),
            ),
        ]),
    }];
    entity.normalize();
    let rendered = character_markdown(&entity, &shipped_ruleset(), &synthetic_labels())
        .expect("a bare entity with real ids still resolves every chrome key");

    assert!(
        rendered.contains("ability-category-martial"),
        "an AbilityCategory-domain parameter value should resolve through \
         `ability-category-<slug>`, got:\n{rendered}"
    );
}

/// A `multi_ref` parameter value (`SelectionParamValue::Multi`) is skipped by
/// `param_display_values` rather than guessed at (C5b's job, per its own doc
/// comment) — the selection still renders, just with that slot showing its
/// unfilled placeholder label instead of a list of names. No existing export
/// test ever constructs a `Multi` value; `Selection::with_params` can only
/// build `Single` ones.
#[test]
fn a_multi_ref_parameter_value_is_skipped_rather_than_rendered() {
    let mut entity = bare_entity(EntityKind::Character, "companion");
    entity.selections = vec![Selection {
        item_ref: Id::new("flaw.corrupted_abilities"),
        params: BTreeMap::from([(
            "targets".to_string(),
            SelectionParamValue::Multi(BTreeSet::from([
                Id::new("ability.awareness"),
                Id::new("ability.athletics"),
            ])),
        )]),
    }];
    entity.normalize();
    let rendered = character_markdown(&entity, &shipped_ruleset(), &synthetic_labels())
        .expect("a bare entity with real ids still resolves every chrome key");

    // The row still renders (no panic, the Flaw's own name resolves); the
    // skipped Multi value is dropped rather than guessed at — neither the raw
    // `ability.*` ids nor the bare `targets` key reach the sheet. (Its name
    // template declares no `{targets}` placeholder, so there is no slot label
    // to show either — the value is simply absent, which is the behavior
    // under test.)
    assert!(rendered.contains("Corrupted Abilities"));
    assert!(!rendered.contains("ability.awareness"));
    assert!(!rendered.contains("ability.athletics"));
    assert!(!rendered.contains("targets"));
}
