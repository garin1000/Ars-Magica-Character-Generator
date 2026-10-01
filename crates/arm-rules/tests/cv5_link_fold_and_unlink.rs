//! CV5 — the dangling/ambiguous-link fold (design § 5.4) and the engine-owned
//! unlink operation (design § 5.5), `docs/vf-audit/design-cv-catalogued-values.md`.
//! Also covers § 6.4's display resolution: one engine function resolves a
//! `Linked` value to its current text, and every reader (export included)
//! must use it — no raw id, no raw `(item, param)` pair, ever rendered.
//!
//! Red-checkpoint protocol, phase 1: `crate::effective::resolve_link` and
//! `unlink_ability_parameters` are no-op stubs, `LoadedEntity`'s new
//! `dangling_links`/`ambiguous_links` fields are always empty, and
//! `export/resolve.rs::ability_param_value`'s `Linked` arm still renders empty
//! (CV4 left it that way on purpose) — every assertion below that expects a
//! real fold, conversion, or resolved display text is expected to fail on its
//! own assertion, never to panic. Phase 2 wires the real resolution in.

use std::collections::{BTreeMap, BTreeSet};

use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::migration::load_entity_migrating;
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
    SelectionParamValue,
};
use arm_rules::{DEFAULT_SAGA_YEAR, unlink_ability_parameters};

fn full_ruleset() -> Ruleset {
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
        spell_mastery_abilities: None,
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

/// The shipped catalogue names, both locales — `load_entity_migrating`'s
/// dependency since CV4.
fn catalogue_names(ruleset: &Ruleset) -> BTreeMap<Id, Vec<String>> {
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

fn localized_ruleset() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        full_ruleset(),
        &[
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
            include_str!("../../../rules/i18n/en/abilities.json"),
            include_str!("../../../rules/i18n/en/arts.json"),
            include_str!("../../../rules/i18n/en/houses.json"),
            include_str!("../../../rules/i18n/en/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/en/spells.json"),
            include_str!("../../../rules/i18n/en/equipment.json"),
            include_str!("../../../rules/i18n/en/aging.json"),
        ],
    )
    .expect("the shipped English rules text loads")
}

/// Every declared chrome key resolved to itself, plus the catalogue-derived
/// families `export.rs` deliberately excludes from `LABEL_KEYS` — mirrors
/// `export_golden.rs`'s own `synthetic_labels`, so `character_markdown` never
/// errors on a missing label regardless of which entity is rendered.
fn synthetic_labels(rs: &LocalizedRuleset) -> BTreeMap<String, String> {
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

/// A companion holding Craft Guild Training once (`guild` set) and one
/// Organization Lore row `Linked` to it.
fn companion_with_guild_link(guild: &str) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.xp_pool = 0;
    e.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new(guild))]),
    )];
    let mut a = AbilityScore::new(Id::new("ability.organization_lore"), 1);
    a.parameter = Some(AbilityParameterValue::Linked {
        item: Id::new("virtue.craft_guild_training"),
        param: "guild".into(),
    });
    e.ability_scores = vec![a];
    e
}

/// A minimal companion save with a `Linked` Organization Lore row but NO
/// selection of the target item at all — dangling from the moment it is
/// opened, never mind what the app did to produce it.
fn companion_json_with_dangling_link() -> String {
    r#"{
      "schema_version": 18,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "companion",
      "ability_scores": [
        { "ability": "ability.organization_lore", "score": 1,
          "parameter": { "item": "virtue.craft_guild_training", "param": "guild" } }
      ]
    }"#
    .to_string()
}

/// The Abilities section's own slice of the export, isolated from the
/// Virtues/Flaws section that precedes it (`character_markdown` writes
/// `write_virtues_flaws` immediately before `write_abilities`) — Craft Guild
/// Training's OWN `guild` parameter cell always shows the raw typed text
/// there (§ 6's `param_value` passthrough, unrelated to this test), so a
/// substring search over the WHOLE document would pass even if the Ability
/// row's own `Linked` resolution rendered nothing. Only this slice proves the
/// Ability row itself resolved the link.
fn abilities_section(md: &str) -> &str {
    let start = md
        .find("abilities-title")
        .expect("the export must carry an Abilities section for this entity");
    &md[start..]
}

/// Design § 6.4: one engine function resolves a `Linked` value to its current
/// display text, and export must use it — never a raw id, never a raw
/// `(item, param)` pair, never a blank cell where a value plainly exists.
#[test]
fn export_shows_the_linked_values_current_resolved_text() {
    let localized = localized_ruleset();
    let labels = synthetic_labels(&localized);
    let entity = companion_with_guild_link("Smiths' Guild of Verdi");

    let md = character_markdown(&entity, &localized, &labels).expect("export succeeds");
    let abilities = abilities_section(&md);
    assert!(
        abilities.contains("Smiths' Guild of Verdi"),
        "the Ability row must show the Linked value's CURRENT resolved text (design § 6.4), \
         not an empty cell — Abilities section:\n{abilities}"
    );
    assert!(
        !abilities.contains("virtue.craft_guild_training"),
        "the Ability row must never render the raw (item, param) pair or a bare id — \
         Abilities section:\n{abilities}"
    );
}

/// Design § 3.2: "a link never has a separate display name, it always shows
/// what the Virtue currently says" — renaming the Virtue's own value must
/// rename every linked Ability row's resolved display too.
#[test]
fn renaming_the_virtues_value_renames_the_resolved_display() {
    let localized = localized_ruleset();
    let labels = synthetic_labels(&localized);
    let mut entity = companion_with_guild_link("Smiths' Guild of Verdi");

    entity.selections[0].params.insert(
        "guild".to_string(),
        SelectionParamValue::Single(Id::new("Journeyman's Lodge")),
    );
    let md = character_markdown(&entity, &localized, &labels).expect("export succeeds");
    let abilities = abilities_section(&md);
    assert!(
        abilities.contains("Journeyman's Lodge") && !abilities.contains("Smiths' Guild of Verdi"),
        "renaming the Virtue's own 'guild' value must rename the linked Ability row's \
         display too, not keep showing the old text — Abilities section:\n{abilities}"
    );
}

/// Design § 5.4: a `Linked` value naming an item held ZERO times among
/// effective selections is dangling. It must be folded to `Text` (empty, since
/// the target was never held at all) and reported so the player is told,
/// exactly like an unmatched free-text value.
#[test]
fn a_dangling_link_is_folded_to_text_and_reported_on_load() {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    let json = companion_json_with_dangling_link();
    let loaded = load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect("a dangling link still loads, never a hard failure");

    assert_eq!(
        loaded.entity.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("")),
        "a dangling link (target never held) must fold to empty Text, keeping no phantom value"
    );
    assert!(
        loaded.dangling_links.contains(&(
            Id::new("ability.organization_lore"),
            Id::new("virtue.craft_guild_training"),
            "guild".to_string(),
        )),
        "the dangling link must be reported so the player is told, not left to discover a \
         silently broken authorization later: {:?}",
        loaded.dangling_links
    );
}

/// Design § 5.5: the engine-owned unlink operation converts every `Linked`
/// row targeting the removed item into `Text` holding its LAST resolvable
/// value — called BEFORE the caller actually splices the selection out, so
/// that value is still readable.
#[test]
fn unlink_operation_converts_linked_rows_to_text_with_the_last_value() {
    let ruleset = full_ruleset();
    let mut entity = companion_with_guild_link("Smiths' Guild of Verdi");

    let converted = unlink_ability_parameters(
        &mut entity,
        &ruleset,
        &Id::new("virtue.craft_guild_training"),
    );

    assert_eq!(
        converted,
        vec![Id::new("ability.organization_lore")],
        "the operation must report which Abilities it converted"
    );
    assert_eq!(
        entity.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("Smiths' Guild of Verdi")),
        "removing the target Virtue must convert the link to Text holding its LAST \
         resolvable value"
    );
}
