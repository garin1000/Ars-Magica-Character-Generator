//! BUG-1 (`tmp/export-audit.md`; D81.8) — the Markdown export prints a
//! `flaw.incompatible_arts` copy's four Arts unpaired, in `BTreeMap` key order
//! (`form_1, form_2, technique_1, technique_2`):
//! `Incompatible Arts (Ignem, Aquam, Creo, Perdo)`. That reads equally as
//! CrAq + PeIg, so the sheet cannot say which two combinations are barred —
//! the Flaw's entire content (ArMDE:6290-6292).
//!
//! The in-app picker already pairs them through
//! `PointItem::unordered_param_groups`, Technique before Form
//! (`ParameterPicker.svelte::paramGroups`). The sheet must say the same:
//! `Incompatible Arts (Creo Ignem, Perdo Aquam)` — members of one group joined
//! by a space (Technique, then Form), groups joined by the list separator.
//!
//! The regression guard pins an ungrouped multi-parameter item
//! (`flaw.ability_block`) to today's exact output, so the fix cannot reorder
//! the extras of an item that declares no groups.

use arm_rules::export::character_markdown;
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::BTreeMap;

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
    .expect("shipped core ruleset loads")
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
    .expect("shipped English rules text loads")
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
    .expect("shipped German rules text loads")
}

/// Every chrome key resolved to itself, except the list separator, which is set
/// to the real `,` both `locales/*/main.ftl` ship (`restricted-xp-list-separator`),
/// so the asserted strings are exactly what the app prints, and the one
/// taxonomy label the regression guard renders.
fn labels(rs: &LocalizedRuleset) -> BTreeMap<String, String> {
    let mut keys: std::collections::BTreeSet<String> = arm_rules::export::LABEL_KEYS
        .iter()
        .map(|k| k.to_string())
        .collect();
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
    let mut labels: BTreeMap<String, String> = keys.into_iter().map(|k| (k.clone(), k)).collect();
    labels.insert("restricted-xp-list-separator".to_string(), ",".to_string());
    labels.insert(
        "ability-category-martial".to_string(),
        "Martial".to_string(),
    );
    labels
}

/// One copy barring Creo Ignem and Perdo Aquam.
fn creo_ignem_and_perdo_aquam() -> Selection {
    Selection::with_params(
        Id::new("flaw.incompatible_arts"),
        BTreeMap::from([
            ("technique_1".to_string(), Id::new("art.creo")),
            ("form_1".to_string(), Id::new("art.ignem")),
            ("technique_2".to_string(), Id::new("art.perdo")),
            ("form_2".to_string(), Id::new("art.aquam")),
        ]),
    )
}

fn magus() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

fn magus_with(selection: Selection) -> Entity {
    let mut e = magus();
    e.selections = vec![selection];
    e.normalize();
    e
}

/// The name cell (first column) of the first table row, at or after line
/// `from`, whose name starts with `name_prefix`.
fn name_cell(rendered: &str, from: usize, name_prefix: &str) -> String {
    let row_start = format!("| {name_prefix}");
    let row = rendered
        .lines()
        .skip(from)
        .find(|line| line.starts_with(&row_start))
        .unwrap_or_else(|| panic!("no table row starting {row_start:?} in:\n{rendered}"));
    row.trim_start_matches("| ")
        .split(" | ")
        .next()
        .expect("a table row has a first cell")
        .to_string()
}

// --- (a) EN, bought copy -------------------------------------------------

#[test]
fn incompatible_arts_exports_each_combination_as_a_technique_form_pair_en() {
    let ruleset = localized_en();
    let entity = magus_with(creo_ignem_and_perdo_aquam());
    let rendered = character_markdown(&entity, &ruleset, &labels(&ruleset))
        .expect("flaw.incompatible_arts and the four Arts are real shipped ids");

    assert_eq!(
        name_cell(&rendered, 0, "Incompatible Arts"),
        "Incompatible Arts (Creo Ignem, Perdo Aquam)",
        "the two barred combinations must be exported as Technique-Form pairs \
         (D81.8), not as four loose Arts in key order"
    );
}

// --- (b) DE, bought copy -------------------------------------------------

#[test]
fn incompatible_arts_exports_each_combination_as_a_technique_form_pair_de() {
    let ruleset = localized_de();
    let entity = magus_with(creo_ignem_and_perdo_aquam());
    let rendered = character_markdown(&entity, &ruleset, &labels(&ruleset))
        .expect("flaw.incompatible_arts and the four Arts are real shipped ids");

    assert_eq!(
        name_cell(&rendered, 0, "Unvereinbare Künste"),
        "Unvereinbare Künste (Creo Ignem, Perdo Aquam)",
        "the German sheet must pair the barred combinations too (D81.8)"
    );
}

// --- (c) regression guard: an ungrouped item is untouched ----------------

/// `flaw.ability_block` declares three parameters (`scope`, `class`, `custom`)
/// and no `unordered_param_groups`; its name has no placeholders, so every value
/// is an extra. Today they trail in key order (`class` before `scope`), which is
/// NOT the declared order — so this pins that an item without groups keeps
/// exactly today's bytes, whatever the fix does for grouped items.
#[test]
fn an_ungrouped_multi_parameter_item_exports_exactly_as_before() {
    let ruleset = localized_en();
    let item = ruleset
        .ruleset
        .item(&Id::new("flaw.ability_block"))
        .expect("flaw.ability_block is a shipped item");
    assert!(
        item.unordered_param_groups.is_empty(),
        "fixture premise: flaw.ability_block declares no unordered_param_groups"
    );
    let mut entity = magus();
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
    let rendered = character_markdown(&entity, &ruleset, &labels(&ruleset))
        .expect("flaw.ability_block and its values are real shipped ids");

    assert_eq!(
        name_cell(&rendered, 0, "Ability Block"),
        "Ability Block (Martial, Whole Category)",
    );
}

// --- (d) a copy in the granted table -------------------------------------

/// The granted table is served by the same `export/sections.rs::Doc::item_rows`
/// as the bought one, so a copy held through a House's open grant must pair
/// too. Ex Miscellanea's `ex_misc_major_flaw` slot asks for a *Major* Hermetic
/// Flaw and Incompatible Arts is Minor, so validation would flag this pick; the
/// export renders what the save holds regardless (Advisory/Silent modes, a
/// hand-edited file), and no shipped open grant admits a Minor Hermetic Flaw.
#[test]
fn a_granted_incompatible_arts_copy_exports_its_pairs_in_the_granted_table() {
    let ruleset = localized_en();
    let mut entity = magus();
    entity.house = Some(Id::new("house.ex_miscellanea"));
    entity.house_choices = BTreeMap::from([(
        "ex_misc_major_flaw".to_string(),
        creo_ignem_and_perdo_aquam(),
    )]);
    entity.normalize();
    let rendered = character_markdown(&entity, &ruleset, &labels(&ruleset))
        .expect("house.ex_miscellanea and the granted flaw are real shipped ids");

    let granted_heading = rendered
        .lines()
        .position(|line| line.starts_with('#') && line.contains("export-granted"))
        .unwrap_or_else(|| panic!("no granted table heading in:\n{rendered}"));
    assert_eq!(
        name_cell(&rendered, granted_heading, "Incompatible Arts"),
        "Incompatible Arts (Creo Ignem, Perdo Aquam)",
        "a granted copy is rendered by the same item_rows and must pair too"
    );
}
