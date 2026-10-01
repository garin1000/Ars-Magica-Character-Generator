//! `ability.enchanting`'s granted/bought label doubles its medium instead of
//! naming it (`tmp/x8a-handover.md` § E; traced via
//! `export/resolve.rs::Doc::fill_template`).
//!
//! `virtue.enchanting_ability` grants `ability.enchanting` at the player's
//! chosen `medium` (`Effect::AbilityScoreGrantParam`, `instance: { "param":
//! "medium" }`), and the ability itself can also be bought directly with a
//! `medium` parameter (`rules/core/abilities.json` declares
//! `"parameter": "medium"`). But `ability.enchanting`'s own shipped i18n
//! `name` — `"Enchanting (Ability)"` / `"Bezaubernde (Fertigkeit)"` — carries
//! no `{medium}` placeholder at all, unlike `(Area) Lore`/`Living Language`'s
//! parameterized templates. Two call sites reach this template with the
//! chosen medium in hand, and each shows a different symptom:
//!
//! * The BOUGHT row (`export/sections.rs::Doc::write_abilities`) threads a
//!   non-empty `{"medium": "Music"}` values map into
//!   [`arm_rules::export::character_markdown`]'s `fill_template`. Since the
//!   template has no `{medium}` key to consume, the "unconsumed value" branch
//!   appends it in a SECOND, redundant parenthetical: `"Enchanting (Ability)
//!   (Music)"` / `"Bezaubernde (Fertigkeit) (Music)"`.
//! * The GRANTED-floor row (`export/sections.rs::Doc::granted_ability_rows`)
//!   never threaded `AbilityFloor::parameter` into the values map at all (a
//!   second, pre-existing defect uncovered while tracing this one — the floor
//!   struct gained its `parameter` field for F-63, but this call site was
//!   never updated to read it), so the medium is silently DROPPED instead of
//!   doubled: the row name is bare `"Enchanting (Ability)"` and the effective
//!   score was read for the wrong (parameter-less) instance.
//!
//! Both are fixed together: `ability.enchanting`'s own `name` gains a
//! `{medium}` token (mirroring `virtue.enchanting_ability`'s own fixed D57
//! apposition form in German, since the same free-text/gender-agreement
//! problem applies to the Ability's name too), and `granted_ability_rows` now
//! threads `floor.parameter` through to both `Doc::parameterized_name` and
//! `effective_ability_score`.

use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::{BTreeMap, BTreeSet};

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
    .expect("the shipped ruleset loads")
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
    .expect("the shipped English rules text loads")
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
    .expect("the shipped German rules text loads")
}

/// Every declared chrome key resolved to itself, plus the catalogue-derived
/// families `export.rs` deliberately excludes from [`LABEL_KEYS`] — mirrors
/// `export_golden.rs`'s `synthetic_labels`, duplicated here rather than shared
/// so this file stays a self-contained unit the concurrent rules/tests agent
/// never needs to touch.
fn synthetic_labels(localized: &LocalizedRuleset) -> BTreeMap<String, String> {
    let mut keys: BTreeSet<String> = LABEL_KEYS.iter().map(|k| k.to_string()).collect();
    for profile in localized.ruleset.profiles() {
        keys.insert(format!("type-{}", profile.id));
    }
    for item in localized.ruleset.items() {
        for category in &item.categories {
            keys.insert(format!("category-{category}"));
        }
        for param in &item.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    for ability in localized.ruleset.abilities() {
        if let Some(parameter) = &ability.parameter {
            keys.insert(format!("param-label-{parameter}"));
        }
    }
    for spell in localized.ruleset.spells() {
        for param in &spell.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    keys.into_iter().map(|k| (k.clone(), k)).collect()
}

fn companion() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

fn markdown(localized: &LocalizedRuleset, entity: &Entity) -> String {
    character_markdown(entity, localized, &synthetic_labels(localized))
        .expect("the fixture entity exports with no missing labels")
}

/// Just the Abilities table's own section, so a match against "Enchanting Music"
/// cannot be satisfied by the Virtues/Flaws table's row of the same text (the
/// Virtue's own name, already fixed by X8a/D57 — a different row this file does
/// not touch).
fn abilities_section(doc: &str) -> &str {
    let start = doc
        .find("## abilities-title")
        .expect("the document must carry an Abilities section");
    let rest = &doc[start..];
    let end = rest[1..].find("\n## ").map_or(rest.len(), |i| i + 1);
    &rest[..end]
}

// ---------------------------------------------------------------------------
// The BOUGHT row: a non-empty values map reaches a template with no matching
// placeholder, so `fill_template`'s unconsumed-value branch doubles the medium
// in a second parenthetical.
// ---------------------------------------------------------------------------

fn companion_with_bought_enchanting(medium: &str) -> Entity {
    let mut e = companion();
    let mut a = AbilityScore::new(Id::new("ability.enchanting"), 1);
    a.parameter = Some(AbilityParameterValue::text(medium));
    e.ability_scores = vec![a];
    e
}

#[test]
fn bought_enchanting_ability_row_names_the_medium_once_english() {
    let localized = localized_en();
    let entity = companion_with_bought_enchanting("Music");
    let doc = markdown(&localized, &entity);
    let section = abilities_section(&doc);
    assert!(
        section.contains("| Enchanting Music |"),
        "F-63 follow-up: the bought ability.enchanting row must read \"Enchanting \
         Music\", not double the medium in a second parenthetical; got:\n{section}"
    );
    assert!(
        !section.contains("(Music)"),
        "the medium must not appear in its own trailing parenthetical; got:\n{section}"
    );
}

#[test]
fn bought_enchanting_ability_row_names_the_medium_once_german() {
    let localized = localized_de();
    let entity = companion_with_bought_enchanting("Music");
    let doc = markdown(&localized, &entity);
    let section = abilities_section(&doc);
    assert!(
        section.contains("| Bezaubernde Fertigkeit, Music |"),
        "F-63 follow-up (D57): the bought ability.enchanting row must stand the \
         free-text medium in apposition, matching virtue.enchanting_ability's own \
         fixed German name, not double it in a second parenthetical; got:\n{section}"
    );
    assert!(
        !section.contains("(Music)"),
        "the medium must not appear in its own trailing parenthetical; got:\n{section}"
    );
}

// ---------------------------------------------------------------------------
// The GRANTED-floor row: `AbilityFloor::parameter` was never threaded into
// the values map at all, so the medium is silently dropped (not doubled).
// ---------------------------------------------------------------------------

fn companion_with_enchanting_virtue(medium: &str) -> Entity {
    let mut e = companion();
    e.selections = vec![Selection::with_params(
        Id::new("virtue.enchanting_ability"),
        BTreeMap::from([("medium".to_string(), Id::new(medium))]),
    )];
    e
}

#[test]
fn granted_enchanting_ability_floor_row_names_the_chosen_medium_english() {
    let localized = localized_en();
    let entity = companion_with_enchanting_virtue("Music");
    let doc = markdown(&localized, &entity);
    let section = abilities_section(&doc);
    assert!(
        section.contains("| Enchanting Music |"),
        "the floor row ability.enchanting gets from virtue.enchanting_ability must \
         name the chosen medium (AbilityFloor::parameter was never read here before \
         this fix) rather than the bare, unfilled generic name; got:\n{section}"
    );
    assert!(
        section.contains("| Enchanting Music |  | 0 | 1 |"),
        "AbilityFloor::parameter must also reach effective_ability_score, or the \
         floor for the WRONG (parameter-less) instance is read and the effective \
         column renders blank/0 instead of the granted 1; got:\n{section}"
    );
}

#[test]
fn granted_enchanting_ability_floor_row_names_the_chosen_medium_german() {
    let localized = localized_de();
    let entity = companion_with_enchanting_virtue("Music");
    let doc = markdown(&localized, &entity);
    let section = abilities_section(&doc);
    assert!(
        section.contains("| Bezaubernde Fertigkeit, Music |  | 0 | 1 |"),
        "the floor row ability.enchanting gets from virtue.enchanting_ability must \
         name the chosen medium, in apposition (D57), and show the granted \
         effective score of 1; got:\n{section}"
    );
}
