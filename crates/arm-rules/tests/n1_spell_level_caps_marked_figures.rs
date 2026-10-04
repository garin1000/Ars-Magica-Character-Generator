//! N1 (try-out 2026-10-04): the Te/Fo grid (`spell_level_caps`) carries the
//! same marked-cap figures the per-spell rows (`spell_caps`) already carry, so
//! the Spells tab's group-header tooltip can show the Magical Focus and Potent
//! Magic caps beside the plain one.
//!
//! Definitions pinned here:
//! - `SpellLevelCap::within_focus_cap` — the grid cell's cap with the Magical
//!   Focus marker; `Some` only while a Magical Focus is held.
//! - `SpellLevelCap::within_potent_field_cap` — the cap with the Potent Magic
//!   marker; `Some` only while Potent Magic is held.
//! - `SpellLevelCap::within_focus_and_potent_field_cap` — both markers; `Some`
//!   only while both Virtues are held.
//!
//! Neither Virtue is scoped per Te/Fo in the data (its focus/field is free
//! text), so the figures appear on every grid row while the Virtue is held.
//! Each figure is `spell_level_cap` with the matching `SpellMarks` — one
//! computation shared with the per-spell path, never a second formula.
//!
//! Fixture as in `r3_potent_magic_spell_cap_fields.rs`: Creo 12 / Ignem 15,
//! plain cap 30, Magical Focus doubling + 12, Major Potent Magic + 6.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{SpellLevelCap, SpellMarks, spell_caps, spell_level_cap, spell_level_caps};
use std::collections::BTreeMap;

const SPELL_ID: &str = "spell.test_n1_grid";
const SPELL_JSON: &str = r#"{ "id": "spell.test_n1_grid", "technique": "art.creo",
     "form": "art.ignem", "level": 50, "requisites": [] }"#;

fn ruleset() -> Ruleset {
    let spells_file = format!(r#"{{"spells": [{SPELL_JSON}]}}"#);
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

/// The Creo/Ignem grid row of the given range class.
fn grid_row(e: &Entity, ruleset: &Ruleset, range_beyond_touch: bool) -> SpellLevelCap {
    spell_level_caps(e, ruleset)
        .into_iter()
        .find(|row| {
            row.technique == Id::new("art.creo")
                && row.form == Id::new("art.ignem")
                && row.range_beyond_touch == range_beyond_touch
        })
        .expect("the grid has a Creo/Ignem row per range class")
}

#[test]
fn grid_row_has_no_marked_figures_without_either_virtue() {
    let ruleset = ruleset();
    let r = grid_row(&magus(vec![]), &ruleset, false);
    assert_eq!(r.cap, 30);
    assert_eq!(r.within_focus_cap, None, "no Magical Focus held");
    assert_eq!(r.within_potent_field_cap, None, "no Potent Magic held");
    assert_eq!(r.within_focus_and_potent_field_cap, None, "neither held");
}

#[test]
fn grid_row_has_a_within_focus_cap_when_a_magical_focus_is_held() {
    let ruleset = ruleset();
    let r = grid_row(&magus(vec![major_magical_focus()]), &ruleset, false);
    assert_eq!(r.cap, 30, "the plain cap stays unmarked");
    assert_eq!(r.within_focus_cap, Some(42), "30 + the doubled Creo 12");
    assert_eq!(r.within_potent_field_cap, None, "no Potent Magic held");
    assert_eq!(r.within_focus_and_potent_field_cap, None, "needs both");
}

#[test]
fn grid_row_has_a_within_potent_field_cap_when_potent_magic_is_held() {
    let ruleset = ruleset();
    let r = grid_row(&magus(vec![potent_magic_major()]), &ruleset, false);
    assert_eq!(r.cap, 30, "the plain cap excludes Potent Magic");
    assert_eq!(r.within_focus_cap, None, "no Magical Focus held");
    assert_eq!(
        r.within_potent_field_cap,
        Some(36),
        "30 + Major Potent Magic's 6"
    );
    assert_eq!(r.within_focus_and_potent_field_cap, None, "needs both");
}

#[test]
fn grid_row_has_all_three_marked_figures_when_both_virtues_are_held() {
    let ruleset = ruleset();
    let r = grid_row(
        &magus(vec![major_magical_focus(), potent_magic_major()]),
        &ruleset,
        false,
    );
    assert_eq!(r.cap, 30);
    assert_eq!(r.within_focus_cap, Some(42));
    assert_eq!(r.within_potent_field_cap, Some(36));
    assert_eq!(r.within_focus_and_potent_field_cap, Some(48));
}

#[test]
fn every_grid_row_figure_is_spell_level_cap_with_the_matching_marks() {
    // Both range classes, every Te/Fo pair, with Short-Ranged Magic held so
    // the beyond-Touch rows really differ: each figure must be exactly the
    // shared `spell_level_cap` with the matching markers.
    let ruleset = ruleset();
    let e = magus(vec![
        major_magical_focus(),
        potent_magic_major(),
        Selection::new(Id::new("flaw.short_ranged_magic")),
    ]);
    let marked = |row: &SpellLevelCap, within_focus: bool, within_potent_field: bool| {
        spell_level_cap(
            &e,
            &ruleset,
            &row.technique,
            &row.form,
            &[],
            row.range_beyond_touch,
            SpellMarks {
                within_focus,
                within_potent_field,
            },
        )
    };
    for row in spell_level_caps(&e, &ruleset) {
        assert_eq!(row.cap, marked(&row, false, false), "{row:?}");
        assert_eq!(
            row.within_focus_cap,
            Some(marked(&row, true, false)),
            "{row:?}"
        );
        assert_eq!(
            row.within_potent_field_cap,
            Some(marked(&row, false, true)),
            "{row:?}"
        );
        assert_eq!(
            row.within_focus_and_potent_field_cap,
            Some(marked(&row, true, true)),
            "{row:?}"
        );
    }
    let beyond = grid_row(&e, &ruleset, true);
    assert_eq!(
        beyond.cap, 15,
        "Short-Ranged Magic halves the beyond-Touch row"
    );
    assert_eq!(beyond.within_focus_and_potent_field_cap, Some(24));
}

#[test]
fn grid_row_matches_the_per_spell_row_of_a_requisite_free_spell() {
    // A requisite-free Touch-or-nearer spell folds nothing, so its per-spell
    // figures and its grid cell's figures must agree.
    let ruleset = ruleset();
    let e = magus(vec![major_magical_focus(), potent_magic_major()]);
    let grid = grid_row(&e, &ruleset, false);
    let spell = spell_caps(&e, &ruleset)
        .into_iter()
        .find(|row| row.spell == Id::new(SPELL_ID))
        .expect("the synthetic spell has a spell_caps row");
    assert_eq!(grid.cap, spell.cap);
    assert_eq!(grid.within_focus_cap, spell.within_focus_cap);
    assert_eq!(grid.within_potent_field_cap, spell.within_potent_field_cap);
    assert_eq!(
        grid.within_focus_and_potent_field_cap,
        spell.within_focus_and_potent_field_cap
    );
}

#[test]
fn grid_row_serializes_the_marked_figures_only_when_present() {
    let ruleset = ruleset();
    let none =
        serde_json::to_string(&grid_row(&magus(vec![]), &ruleset, false)).expect("serializes");
    assert!(
        !none.contains("within_"),
        "absent figures must not be written: {none}"
    );
    let both = serde_json::to_string(&grid_row(
        &magus(vec![major_magical_focus(), potent_magic_major()]),
        &ruleset,
        false,
    ))
    .expect("serializes");
    assert!(both.contains("\"within_focus_cap\":42"), "{both}");
    assert!(both.contains("\"within_potent_field_cap\":36"), "{both}");
    assert!(
        both.contains("\"within_focus_and_potent_field_cap\":48"),
        "{both}"
    );
}
