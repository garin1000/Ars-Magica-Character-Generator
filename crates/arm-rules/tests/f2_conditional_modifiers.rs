//! F2's conditional-modifier fix (design-f0-book-template-engine.md § 1a/§ 2c,
//! D61 invoking D15's already-shipped `flaw.corrupted_spells` precedent):
//! Berserk and Ways of the Land lose every effect and reclassify to
//! `uncomputed_rule`; Cyclic Magic (both ids) and Special Circumstances lose
//! only their unconditional Casting-Total clause and keep their other,
//! untouched effect, classification unchanged. Both changes must reach the
//! player through `description` in every locale where the guards in
//! `uncomputed_clauses.rs` require it.
//!
//! RED CHECKPOINT: this file pins the FINAL shape of the five catalogue
//! entries. `rules/core/virtues_flaws.json` and the two `rules/i18n/<lang>/
//! virtues_flaws.json` files have not been edited yet (phase 2 data work), so
//! every test below fails against the shipped catalogue until that data
//! change lands.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Classification, Effect, Id};

/// The shipped core ruleset, loaded exactly as `book_templates.rs` does.
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

/// B2/D61: the printed Berserker statblock is the NOT-berserk figure, and no
/// template anywhere shows the "+2/-2/+2 while berserk" state — "are you
/// currently berserk" is exactly D61's uncomputable shape. D15's shipped
/// precedent (`flaw.corrupted_spells`) is delete + reclassify, not surface.
#[test]
fn berserk_loses_every_effect_and_becomes_uncomputed_rule() {
    let rs = full_ruleset();
    let item = rs
        .item(&Id::new("virtue.berserk"))
        .expect("virtue.berserk must ship");
    assert_eq!(
        item.classification,
        Classification::UncomputedRule,
        "no template ever prints a boosted Berserk figure, so no effect survives (D61/D15)"
    );
    assert!(
        item.effects.is_empty(),
        "the combat_mod x2 + soak_mod trio must be deleted outright, not surfaced"
    );
}

/// MAG1: Bjornaer's printed Casting Totals are the unconditional base — no
/// dual-row precedent for the boosted figure either.
#[test]
fn ways_of_the_land_loses_its_effect_and_becomes_uncomputed_rule() {
    let rs = full_ruleset();
    let item = rs
        .item(&Id::new("virtue.ways_of_the_land"))
        .expect("virtue.ways_of_the_land must ship");
    assert_eq!(item.classification, Classification::UncomputedRule);
    assert!(
        item.effects.is_empty(),
        "the casting_total_mod must be deleted"
    );
}

/// MAG1 (Mercere): the printed Casting Totals are base/unboosted for the
/// Casting term; the Lab term (X7a's separate, still-open problem) is
/// untouched and keeps this entry mechanically active.
#[test]
fn cyclic_magic_positive_keeps_only_lab_total_mod() {
    let rs = full_ruleset();
    let item = rs
        .item(&Id::new("virtue.cyclic_magic_positive"))
        .expect("virtue.cyclic_magic_positive must ship");
    assert_eq!(
        item.classification,
        Classification::InPlayEffect,
        "the surviving Lab Total effect keeps this entry mechanically active (X7a, untouched)"
    );
    assert!(
        !item
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CastingTotalMod { .. })),
        "the unconditional Casting clause must be deleted (D61)"
    );
    assert!(
        item.effects
            .iter()
            .any(|e| matches!(e, Effect::LabTotalMod { amount: 3 })),
        "the Lab Total clause is X7a's problem and must stay untouched here"
    );
}

/// Mirror of the positive Virtue, same reasoning.
#[test]
fn cyclic_magic_negative_keeps_only_lab_total_mod() {
    let rs = full_ruleset();
    let item = rs
        .item(&Id::new("flaw.cyclic_magic_negative"))
        .expect("flaw.cyclic_magic_negative must ship");
    assert_eq!(item.classification, Classification::InPlayEffect);
    assert!(
        !item
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CastingTotalMod { .. })),
        "the unconditional Casting clause must be deleted (D61)"
    );
    assert!(
        item.effects
            .iter()
            .any(|e| matches!(e, Effect::LabTotalMod { amount: -3 })),
        "the Lab Total clause is X7a's problem and must stay untouched here"
    );
}

/// MAG1 (Mercere): same base-only printed figure for the Casting term;
/// `aura_bonus` is a pre-existing, already-correct D58 family, untouched.
#[test]
fn special_circumstances_keeps_only_magic_resistance_mod() {
    let rs = full_ruleset();
    let item = rs
        .item(&Id::new("virtue.special_circumstances"))
        .expect("virtue.special_circumstances must ship");
    assert_eq!(item.classification, Classification::InPlayEffect);
    assert!(
        !item
            .effects
            .iter()
            .any(|e| matches!(e, Effect::CastingTotalMod { .. })),
        "the unconditional Casting clause must be deleted (D61)"
    );
    assert!(
        item.effects
            .iter()
            .any(|e| matches!(e, Effect::MagicResistanceMod { .. })),
        "the pre-existing D58 aura_bonus family must stay untouched"
    );
}

// --- Descriptions in both locales (D61's obligation that the dropped clause
// still reaches the player) ---

const EN_VF: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_VF: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

fn has_description(json: &str, id: &str) -> bool {
    let entries: serde_json::Value =
        serde_json::from_str(json).expect("rules/i18n virtues_flaws.json is valid JSON");
    entries[id]["description"]
        .as_str()
        .is_some_and(|s| !s.trim().is_empty())
}

/// `uncomputed_clauses.rs::every_uncomputed_rule_entry_states_its_rule_in_every_locale`
/// requires a `description` here: Berserk's current `summary` alone states no
/// number.
#[test]
fn berserk_states_its_rule_in_both_locales() {
    assert!(
        has_description(EN_VF, "virtue.berserk"),
        "EN description missing — the summary alone states no number"
    );
    assert!(
        has_description(DE_VF, "virtue.berserk"),
        "DE description missing"
    );
}

/// `uncomputed_clauses.rs::no_swept_entry_drops_an_uncomputed_mechanical_clause`
/// needs the now-uncomputed Casting clause carried by `description` once the
/// two `COMPUTED_ENTRY_COVERS_WHOLE_PASSAGE` exemption rows for these ids are
/// removed (§ 2c).
#[test]
fn cyclic_magic_entries_state_their_casting_clause_in_both_locales() {
    for id in ["virtue.cyclic_magic_positive", "flaw.cyclic_magic_negative"] {
        assert!(has_description(EN_VF, id), "{id}: EN description missing");
        assert!(has_description(DE_VF, id), "{id}: DE description missing");
    }
}

/// Not guard-mandatory (the `summary` already states the +3 number), but the
/// design note recommends a `description` here too for consistency with the
/// other four reclassified entries.
#[test]
fn special_circumstances_gains_a_description_for_consistency() {
    assert!(has_description(EN_VF, "virtue.special_circumstances"));
    assert!(has_description(DE_VF, "virtue.special_circumstances"));
}
