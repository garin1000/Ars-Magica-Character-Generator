//! R3 (after-deadline answer 3, amends D1): the NEW surface the slice adds —
//! `spell_level_cap`'s `within_potent_field` input, and `spell_caps`'s
//! `within_potent_field_cap` and `within_focus_and_potent_field_cap` figures
//! for the picker. The behaviour through the existing surface is pinned by
//! `r3_potent_magic_spell_cap.rs`.
//!
//! Definitions pinned here:
//! - `SpellCap::within_potent_field_cap` — the cap with the spell marked
//!   within the Potent field (and not within focus); `Some` only while the
//!   entity holds a Lab-Total carrier scoped `within_potent_field_only`
//!   (Potent Magic), `None` otherwise — same shape as `within_focus_cap`.
//! - `SpellCap::within_focus_and_potent_field_cap` — the cap with BOTH
//!   markers; `Some` only while the entity holds a Magical Focus AND Potent
//!   Magic. It is a separate engine figure, not something the UI may add up
//!   from the other two: the halvings (Deficient Art, Short-Ranged Magic)
//!   floor-divide the SUM, so `cap + doubling + bonus` is not derivable from
//!   the two single-marker figures (see
//!   `the_combined_figure_is_not_derivable_from_the_single_marker_figures`).
//!
//! Fixture as in `r3_potent_magic_spell_cap.rs`: Creo 12 / Ignem 15, plain
//! cap 30, Magical Focus doubling + 12, Major Potent Magic + 6.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{SpellCap, spell_caps, spell_level_cap};
use std::collections::BTreeMap;

fn ruleset_with_only_spell(spell_json: &str) -> Ruleset {
    let spells_file = format!(r#"{{"spells": [{spell_json}]}}"#);
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
        spells: Some(&spells_file),
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
    .expect("single-synthetic-spell ruleset loads")
}

const SPELL_ID: &str = "spell.test_r3_potent";
const SPELL_JSON: &str = r#"{ "id": "spell.test_r3_potent", "technique": "art.creo",
     "form": "art.ignem", "level": 50, "requisites": [] }"#;

fn param(item: &str, key: &str, value: &str) -> Selection {
    Selection::with_params(
        Id::new(item),
        BTreeMap::from([(key.to_string(), Id::new(value))]),
    )
}

fn potent_magic_major() -> Selection {
    param("virtue.potent_magic_major", "field", "fire")
}

fn major_magical_focus() -> Selection {
    param("virtue.major_magical_focus", "focus", "fire")
}

fn deficient_form_ignem() -> Selection {
    param("flaw.deficient_form", "form", "art.ignem")
}

fn magus(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 12),
        ArtScore::new(Id::new("art.ignem"), 15),
    ];
    e.selections = selections;
    e
}

fn row(e: &Entity, ruleset: &Ruleset) -> SpellCap {
    spell_caps(e, ruleset)
        .into_iter()
        .find(|row| row.spell == Id::new(SPELL_ID))
        .expect("the synthetic spell has a spell_caps row")
}

/// `spell_level_cap` for Creo/Ignem, no requisites, Touch-or-nearer.
fn cap(e: &Entity, ruleset: &Ruleset, within_focus: bool, within_potent_field: bool) -> i64 {
    spell_level_cap(
        e,
        ruleset,
        &Id::new("art.creo"),
        &Id::new("art.ignem"),
        &[],
        false,
        within_focus,
        within_potent_field,
    )
}

// --- spell_level_cap's new input ---------------------------------------------

#[test]
fn spell_level_cap_adds_potent_magic_only_when_marked() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let e = magus(vec![potent_magic_major()]);
    assert_eq!(cap(&e, &ruleset, false, false), 30, "unmarked: no +6");
    assert_eq!(cap(&e, &ruleset, false, true), 36, "marked: +6");
}

#[test]
fn spell_level_cap_ignores_a_stale_potent_field_mark() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let e = magus(vec![]);
    assert_eq!(
        cap(&e, &ruleset, false, true),
        30,
        "marked but no Potent Magic Virtue held: the mark adds nothing"
    );
}

#[test]
fn spell_level_cap_with_both_marks_halves_the_sum_once() {
    // Order of operations (ArMDE:2465, :4403, :5911): the doubling and the
    // bonus both sum into the Lab Total first, then the Deficient Form halves
    // it — floor((30 + 12 + 6) / 2) = 24.
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let e = magus(vec![
        major_magical_focus(),
        potent_magic_major(),
        deficient_form_ignem(),
    ]);
    assert_eq!(cap(&e, &ruleset, true, true), 24);
}

// --- spell_caps' new picker figures ------------------------------------------

#[test]
fn spell_caps_has_no_potent_figures_without_potent_magic() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let r = row(&magus(vec![major_magical_focus()]), &ruleset);
    assert_eq!(r.cap, 30);
    assert_eq!(r.within_focus_cap, Some(42));
    assert_eq!(
        r.within_potent_field_cap, None,
        "no Potent Magic held, so the picker has no within-Potent-field figure"
    );
    assert_eq!(
        r.within_focus_and_potent_field_cap, None,
        "the combined figure needs both Virtues"
    );
}

#[test]
fn spell_caps_has_a_within_potent_field_cap_when_potent_magic_is_held() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let r = row(&magus(vec![potent_magic_major()]), &ruleset);
    assert_eq!(r.cap, 30, "the plain cap excludes Potent Magic");
    assert_eq!(r.within_focus_cap, None, "no Magical Focus held");
    assert_eq!(r.within_potent_field_cap, Some(36));
    assert_eq!(
        r.within_focus_and_potent_field_cap, None,
        "the combined figure needs both Virtues"
    );
}

#[test]
fn spell_caps_has_all_three_marked_figures_when_both_virtues_are_held() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let r = row(
        &magus(vec![major_magical_focus(), potent_magic_major()]),
        &ruleset,
    );
    assert_eq!(r.cap, 30);
    assert_eq!(r.within_focus_cap, Some(42), "focus only: no +6");
    assert_eq!(
        r.within_potent_field_cap,
        Some(36),
        "potent only: no doubling"
    );
    assert_eq!(r.within_focus_and_potent_field_cap, Some(48));
}

#[test]
fn the_combined_figure_is_not_derivable_from_the_single_marker_figures() {
    // Creo 13 / Ignem 14, Major Magical Focus (+13), Minor Potent Magic (+3),
    // Deficient Form (halves after summing, ArMDE:5911, floored ArMDE:547):
    // plain 30 -> 15, focus 43 -> 21, potent 33 -> 16, both 46 -> 23.
    // Adding up the single-marker figures (21 + 16 - 15 = 22) would be wrong
    // by one, which is why the combined figure is an engine figure.
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let mut e = magus(vec![
        major_magical_focus(),
        param("virtue.potent_magic_minor", "field", "fire"),
        deficient_form_ignem(),
    ]);
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 13),
        ArtScore::new(Id::new("art.ignem"), 14),
    ];
    let r = row(&e, &ruleset);
    assert_eq!(r.cap, 15);
    assert_eq!(r.within_focus_cap, Some(21));
    assert_eq!(r.within_potent_field_cap, Some(16));
    assert_eq!(r.within_focus_and_potent_field_cap, Some(23));
}

#[test]
fn spell_cap_serializes_the_new_figures_only_when_present() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let none = serde_json::to_string(&row(&magus(vec![]), &ruleset)).expect("serializes");
    assert!(
        !none.contains("within_potent_field_cap") && !none.contains("within_focus_and"),
        "absent figures must not be written: {none}"
    );
    let both = serde_json::to_string(&row(
        &magus(vec![major_magical_focus(), potent_magic_major()]),
        &ruleset,
    ))
    .expect("serializes");
    assert!(both.contains("\"within_potent_field_cap\":36"), "{both}");
    assert!(
        both.contains("\"within_focus_and_potent_field_cap\":48"),
        "{both}"
    );
}
