//! X8c (`docs/open-todos.md` row 41; D65 N1, `docs/vf-audit/decisions.md`) —
//! the Markdown export today prints a V/F's NAME only
//! (`export/sections.rs::Doc::item_rows`). D65 N1 rules that the sheet must
//! also carry rules TEXT: an `uncomputed_rule` entry shows `description ?? summary`,
//! and a computed entry (`creation_effect`/`in_play_effect`) shows its `summary`.
//!
//! Phase 1 only — these tests are expected to be RED until `item_rows` grows a
//! text column/cell. They use REAL shipped ids so the fixture proves the i18n
//! text actually reaches the sheet, not a synthetic stand-in:
//!
//! - `flaw.arthritis` (`uncomputed_rule`) has a `description` strictly longer
//!   than its `summary` (the summary is the description's first sentence), so
//!   asserting the FULL description string appears proves the description was
//!   used, not the shorter summary.
//! - `flaw.missing_ear` (`uncomputed_rule`) carries NO `description` at all in
//!   the shipped i18n — only `summary` — so the fallback half of `?? ` is
//!   exercised for real, on real data.
//! - `virtue.puissant_ability` (`creation_effect`, i.e. computed) has a
//!   `summary` worded completely differently from its `description`, so
//!   asserting the summary appears AND a description-only sentence does not
//!   proves the computed branch prints the summary, never the description.

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

/// Every chrome key resolved to itself — the export/sections.rs machinery only
/// needs SOME string back for a label lookup; the rules TEXT under test comes
/// from the real per-language i18n catalogue merged above, not from this map.
fn synthetic_labels(rs: &LocalizedRuleset) -> BTreeMap<String, String> {
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
    keys.into_iter().map(|k| (k.clone(), k)).collect()
}

fn entity_with(selection: Selection) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = vec![selection];
    e.normalize();
    e
}

fn puissant_awareness() -> Selection {
    Selection::with_params(
        Id::new("virtue.puissant_ability"),
        BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
    )
}

// --- EN ----------------------------------------------------------------

#[test]
fn uncomputed_rule_with_description_exports_the_description_en() {
    let ruleset = localized_en();
    let labels = synthetic_labels(&ruleset);
    let entity = entity_with(Selection::new(Id::new("flaw.arthritis")));
    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("flaw.arthritis is a real shipped id");

    let description = ruleset
        .description(&Id::new("flaw.arthritis"))
        .expect("flaw.arthritis carries a description in the shipped EN i18n");
    assert!(
        rendered.contains(description),
        "an uncomputed_rule V/F with a description must export the FULL \
         description text (D65 N1); got:\n{rendered}"
    );
}

#[test]
fn uncomputed_rule_without_description_exports_the_summary_en() {
    let ruleset = localized_en();
    assert!(
        ruleset.description(&Id::new("flaw.missing_ear")).is_none(),
        "fixture premise: flaw.missing_ear must carry no description in the \
         shipped EN i18n"
    );
    let labels = synthetic_labels(&ruleset);
    let entity = entity_with(Selection::new(Id::new("flaw.missing_ear")));
    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("flaw.missing_ear is a real shipped id");

    let summary = ruleset
        .summary(&Id::new("flaw.missing_ear"))
        .expect("flaw.missing_ear carries a summary in the shipped EN i18n");
    assert!(
        rendered.contains(summary),
        "an uncomputed_rule V/F with NO description must fall back to its \
         summary (D65 N1's `description ?? summary`); got:\n{rendered}"
    );
}

#[test]
fn computed_entry_exports_its_summary_en() {
    let ruleset = localized_en();
    let labels = synthetic_labels(&ruleset);
    let entity = entity_with(puissant_awareness());
    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("virtue.puissant_ability is a real shipped id");

    let summary = ruleset
        .summary(&Id::new("virtue.puissant_ability"))
        .expect("virtue.puissant_ability carries a summary in the shipped EN i18n");
    assert!(
        rendered.contains(summary),
        "a computed (creation_effect) V/F must export its summary (D65 N1); \
         got:\n{rendered}"
    );
    assert!(
        !rendered.contains("You may only take this Virtue once for a given Ability"),
        "a computed V/F must NOT export its full description, only its \
         summary; got:\n{rendered}"
    );
}

// --- DE ------------------------------------------------------------------

#[test]
fn uncomputed_rule_with_description_exports_the_description_de() {
    let ruleset = localized_de();
    let labels = synthetic_labels(&ruleset);
    let entity = entity_with(Selection::new(Id::new("flaw.arthritis")));
    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("flaw.arthritis is a real shipped id");

    let description = ruleset
        .description(&Id::new("flaw.arthritis"))
        .expect("flaw.arthritis carries a description in the shipped DE i18n");
    assert!(
        rendered.contains(description),
        "an uncomputed_rule V/F with a description must export the FULL \
         German description text (D65 N1); got:\n{rendered}"
    );
}

#[test]
fn uncomputed_rule_without_description_exports_the_summary_de() {
    let ruleset = localized_de();
    assert!(
        ruleset.description(&Id::new("flaw.missing_ear")).is_none(),
        "fixture premise: flaw.missing_ear must carry no description in the \
         shipped DE i18n"
    );
    let labels = synthetic_labels(&ruleset);
    let entity = entity_with(Selection::new(Id::new("flaw.missing_ear")));
    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("flaw.missing_ear is a real shipped id");

    let summary = ruleset
        .summary(&Id::new("flaw.missing_ear"))
        .expect("flaw.missing_ear carries a summary in the shipped DE i18n");
    assert!(
        rendered.contains(summary),
        "an uncomputed_rule V/F with NO description must fall back to its \
         summary in German too (D65 N1); got:\n{rendered}"
    );
}

#[test]
fn computed_entry_exports_its_summary_de() {
    let ruleset = localized_de();
    let labels = synthetic_labels(&ruleset);
    let entity = entity_with(puissant_awareness());
    let rendered = character_markdown(&entity, &ruleset, &labels)
        .expect("virtue.puissant_ability is a real shipped id");

    let summary = ruleset
        .summary(&Id::new("virtue.puissant_ability"))
        .expect("virtue.puissant_ability carries a summary in the shipped DE i18n");
    assert!(
        rendered.contains(summary),
        "a computed (creation_effect) V/F must export its summary in German \
         too (D65 N1); got:\n{rendered}"
    );
    assert!(
        !rendered
            .contains("Du kannst diese Tugend für eine bestimmte Fertigkeit nur einmal wählen"),
        "a computed V/F must NOT export its full German description, only its \
         summary; got:\n{rendered}"
    );
}
