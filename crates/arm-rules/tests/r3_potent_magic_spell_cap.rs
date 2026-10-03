//! R3 (after-deadline answer 3, amends D1 — `docs/vf-audit/decisions.md`):
//! Potent Magic's Lab Total bonus counts toward a spell's creation-time level
//! cap **only** for a spell the player has marked
//! `SpellSelection::within_potent_field`, exactly as the Casting Total already
//! reads that marker (D79). The other eight D1 carriers stay flat and
//! condition-free.
//!
//! ArMDE:4742: "The maga's magic is particularly attuned to a narrow field,
//! much as in a Magical Focus. [...] a maga may have more than one area of
//! Potent Magic, although only one Potent Magic Virtue applies to any single
//! activity." ArMDE:4744: "Potent Magic provides the maga with a bonus in her
//! field of magic". ArMDE:4746/:4748: Minor grants "+3 bonus to Lab Totals and
//! Casting Score", Major "+6".
//!
//! Every test here goes through the EXISTING public surface (`validate`'s
//! `spell_level_exceeds_cap` issue and `spell_caps`'s plain `cap`), so each
//! one compiles against today's engine and fails, where it fails, on its
//! assertion. The tests that need the new `spell_caps` fields live in
//! `r3_potent_magic_spell_cap_fields.rs`.
//!
//! Fixture: one synthetic spell, Creo/Ignem level 50 (far above every cap
//! below, so `validate` always reports the cap it computed), and a magus with
//! Creo 12 / Ignem 15, Int 0, Magic Theory 0. Plain cap = 12 + 15 + 0 + 0 + 3
//! = 30. Magical Focus doubles the lower Art: + min(12, 15) = + 12.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::spell_caps;
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, validate};
use std::collections::BTreeMap;

/// The shipped core ruleset with a single synthetic spell (same
/// `ruleset_with_only_spell` pattern `composite_spell_five_dimensions.rs`
/// uses; duplicated because integration test binaries cannot share helpers).
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

/// Creo 12 + Ignem 15 + Int 0 + Magic Theory 0 + 3.
const PLAIN_CAP: i64 = 30;
/// Magical Focus doubling: + min(Creo 12, Ignem 15).
const FOCUS_DOUBLING: i64 = 12;

fn ruleset() -> Ruleset {
    ruleset_with_only_spell(SPELL_JSON)
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

fn potent_magic_minor() -> Selection {
    param("virtue.potent_magic_minor", "field", "water")
}

fn major_magical_focus() -> Selection {
    param("virtue.major_magical_focus", "focus", "fire")
}

/// The magus, holding `selections`, with the synthetic spell chosen and its
/// two markers set as given.
fn magus(selections: Vec<Selection>, within_focus: bool, within_potent_field: bool) -> Entity {
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
    let mut sel = SpellSelection::new(Id::new(SPELL_ID));
    sel.within_focus = within_focus;
    sel.within_potent_field = within_potent_field;
    e.spells = vec![sel];
    e
}

/// The cap `validate` reports in the spell's `spell_level_exceeds_cap` issue.
fn validated_cap(e: &Entity, ruleset: &Ruleset) -> i64 {
    let result = validate(e, ruleset);
    let issue = result
        .issues
        .iter()
        .find(|i| {
            i.code == ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP
                && i.context.as_ref() == Some(&Id::new(SPELL_ID))
        })
        .unwrap_or_else(|| {
            panic!(
                "the level-50 spell must exceed every cap in this file; issues {:?}",
                result.issues
            )
        });
    issue
        .args
        .get("cap")
        .expect("cap arg present")
        .parse()
        .expect("cap arg is a number")
}

/// The plain `cap` `spell_caps` surfaces to the picker for the synthetic spell.
fn picker_cap(e: &Entity, ruleset: &Ruleset) -> i64 {
    spell_caps(e, ruleset)
        .into_iter()
        .find(|row| row.spell == Id::new(SPELL_ID))
        .expect("the synthetic spell has a spell_caps row")
        .cap
}

// --- Unmarked: Potent Magic is NOT in the cap --------------------------------

#[test]
fn an_unmarked_spell_does_not_get_major_potent_magic_in_its_cap() {
    let ruleset = ruleset();
    let e = magus(vec![potent_magic_major()], false, false);
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP,
        "answer 3 / ArMDE:4744: Potent Magic's bonus is 'in her field of magic' only, so an \
         unmarked spell's cap must not include its +6"
    );
}

#[test]
fn an_unmarked_spell_does_not_get_minor_potent_magic_in_its_cap() {
    let ruleset = ruleset();
    let e = magus(vec![potent_magic_minor()], false, false);
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP,
        "an unmarked spell's cap must not include Minor Potent Magic's +3"
    );
}

#[test]
fn the_picker_plain_cap_excludes_potent_magic() {
    // `spell_caps`'s plain `cap` is the UNMARKED figure: the picker greys a
    // spell by it, and offers the marked figures separately.
    let ruleset = ruleset();
    let e = magus(vec![potent_magic_major()], false, false);
    assert_eq!(
        picker_cap(&e, &ruleset),
        PLAIN_CAP,
        "spell_caps().cap is the unmarked cap and must not include Potent Magic's +6"
    );
}

// --- Marked + Virtue held: +6 / +3 -------------------------------------------

#[test]
fn a_marked_spell_gets_major_potent_magic_in_its_cap() {
    let ruleset = ruleset();
    let e = magus(vec![potent_magic_major()], false, true);
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP + 6,
        "ArMDE:4748: a spell marked within the Potent field gets Major Potent Magic's +6"
    );
}

#[test]
fn a_marked_spell_gets_minor_potent_magic_in_its_cap() {
    let ruleset = ruleset();
    let e = magus(vec![potent_magic_minor()], false, true);
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP + 3,
        "ArMDE:4746: a spell marked within the Potent field gets Minor Potent Magic's +3"
    );
}

#[test]
fn only_one_potent_magic_virtue_applies_to_a_marked_spell() {
    // ArMDE:4742: "only one Potent Magic Virtue applies to any single
    // activity" — the larger applies (+6), never the sum (+9). Same MAX the
    // in-play totals already take (D79).
    let ruleset = ruleset();
    let e = magus(
        vec![potent_magic_major(), potent_magic_minor()],
        false,
        true,
    );
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP + 6,
        "a Major + a Minor Potent Magic Virtue must add only the larger bonus to the cap"
    );
}

#[test]
fn two_potent_magic_virtues_add_nothing_to_an_unmarked_spell() {
    let ruleset = ruleset();
    let e = magus(
        vec![potent_magic_major(), potent_magic_minor()],
        false,
        false,
    );
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP,
        "an unmarked spell gets no Potent Magic bonus however many such Virtues are held"
    );
}

// --- Stale mark: the Virtue is gone, the marker adds nothing ----------------

#[test]
fn a_stale_potent_field_mark_adds_nothing_once_the_virtue_is_removed() {
    // Saves store choices, so the marker survives the Virtue's removal; its
    // effect on the cap must not (same rule as a stale `within_focus`).
    let ruleset = ruleset();
    let e = magus(vec![], false, true);
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP,
        "a within_potent_field mark without any Potent Magic Virtue must leave the cap flat"
    );
}

// --- Both markers ------------------------------------------------------------

#[test]
fn a_spell_marked_both_gets_the_focus_doubling_and_the_potent_bonus() {
    let ruleset = ruleset();
    let e = magus(
        vec![major_magical_focus(), potent_magic_major()],
        true,
        true,
    );
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP + FOCUS_DOUBLING + 6,
        "ArMDE:4742 ('compatible with a Magical Focus'): both markers stack — 30 + 12 + 6"
    );
}

#[test]
fn a_spell_marked_within_focus_only_does_not_get_the_potent_bonus() {
    let ruleset = ruleset();
    let e = magus(
        vec![major_magical_focus(), potent_magic_major()],
        true,
        false,
    );
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP + FOCUS_DOUBLING,
        "the within_focus marker alone must not bring in Potent Magic's bonus"
    );
}

#[test]
fn a_spell_marked_within_potent_field_only_does_not_get_the_focus_doubling() {
    let ruleset = ruleset();
    let e = magus(
        vec![major_magical_focus(), potent_magic_major()],
        false,
        true,
    );
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP + 6,
        "the within_potent_field marker alone must not bring in the Magical Focus doubling"
    );
}

// --- The other D1 carriers stay flat -----------------------------------------

#[test]
fn inventive_genius_still_applies_flat_to_an_unmarked_spell() {
    // D1 stands for the other eight carriers: Inventive Genius's +3 reaches
    // every spell's cap, marked or not, alongside an unmarked Potent Magic.
    let ruleset = ruleset();
    let e = magus(
        vec![
            Selection::new(Id::new("virtue.inventive_genius")),
            potent_magic_major(),
        ],
        false,
        false,
    );
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP + 3,
        "D1: Inventive Genius stays flat (+3); only Potent Magic is gated by the marker"
    );
}

#[test]
fn weak_scholar_still_applies_flat_to_the_cap() {
    // A `never_at_creation`-scoped carrier (D4) is still flat under D1: only
    // the `within_potent_field_only` scope is gated by the marker.
    let ruleset = ruleset();
    let e = magus(
        vec![Selection::new(Id::new("flaw.weak_scholar"))],
        false,
        false,
    );
    assert_eq!(
        validated_cap(&e, &ruleset),
        PLAIN_CAP - 6,
        "D1: Weak Scholar's -6 stays flat on the cap"
    );
}
