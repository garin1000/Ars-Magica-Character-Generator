//! Phase 1 (analysis + RED tests only) of folding spell requisites into the
//! Casting Total — see `tmp/requisites-handover.md` for the verbatim rule
//! quotes, the book-template conformance table (which shows this gap is the
//! root cause of the previously-filed MAG4 "book disagreement"), and the
//! design Phase 2 implements against. `Spell::requisites` is stored
//! (`crates/arm-rules/src/spell.rs`) but today `derived/casting.rs`'s
//! `formulaic_casting_score` never reads it — every test below pins the
//! correct, rules-derived figure and is RED against today's engine unless
//! its own comment says otherwise.
//!
//! Source: ArMDE:12309-12313 (`### Requisites`, the base min rule),
//! :12311 (several requisites / Deficient Art as a requisite), :4403
//! (Major Magical Focus — "the lowest applicable score may be one of the
//! requisites"), :3737 (Elemental Magic — primary elemental Form wins over
//! an elemental requisite even if lower), :4820 (Puissant Art — compare
//! bonus-inclusive scores).

use arm_rules::derived::spell_casting_total;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::BTreeMap;

/// The shipped core ruleset — same set of files `book_templates.rs::full_ruleset`
/// loads; duplicated because integration test binaries cannot share private
/// helpers (convention already established by `x10bc_banked_xp_and_within_focus.rs`,
/// `x5b_ability_minimums.rs`).
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

/// A ruleset identical to [`full_ruleset`] except its spell catalogue is replaced
/// by a single synthetic spell, for the one test (`two_form_requisites_in_the_same_category`)
/// that needs a shape (two requisites of the same Art class) no shipped spell has.
/// Every other catalogue (Arts, Virtues/Flaws, …) stays the real shipped data, so
/// `virtue.puissant_art`-style selections still resolve normally if a test needs them.
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

fn entity() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

fn art(id: &str, score: u8) -> ArtScore {
    ArtScore::new(Id::new(id), score)
}

/// The Formulaic Casting Total `spell_casting_total` reaches for `spell_id` on
/// `e`, with `within_focus` as given. Every case here leaves `within_potent_field`
/// false — D79's Potent Magic interaction is untouched by this task.
fn casting(e: &Entity, ruleset: &Ruleset, spell_id: &str, within_focus: bool) -> i32 {
    let mut sel = SpellSelection::new(Id::new(spell_id));
    sel.within_focus = within_focus;
    spell_casting_total(&sel, e, ruleset)
        .unwrap_or_else(|| panic!("{spell_id} must be in the catalogue"))
}

// --- Base min rule: ArMDE:12309 ---------------------------------------------
//
// "You must use the lesser of your score in the requisite and your score in
// the spell's main Technique or Form — Technique if the requisite is a
// Technique, Form if the requisite is a Form."

#[test]
fn technique_requisite_lower_than_the_technique_reduces_the_total() {
    // spell.obliteration_of_the_metallic_barrier: Pe/Te, requisite art.rego
    // (Technique-class). Pe 10, Te 10, Re 3 (lower than Pe) -> use Re's 3 in
    // Pe's place: 3 + 10 = 13. Today's engine ignores the requisite entirely
    // and reaches 10 + 10 = 20.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.perdo", 10),
        art("art.terram", 10),
        art("art.rego", 3),
    ];
    assert_eq!(
        casting(
            &e,
            &ruleset,
            "spell.obliteration_of_the_metallic_barrier",
            false
        ),
        13,
        "ArMDE:12309: a lower Technique requisite must replace the main Technique score"
    );
}

#[test]
fn form_requisite_lower_than_the_form_reduces_the_total() {
    // spell.rain_of_stones: Mu/Au, requisite art.terram (Form-class). Mu 10,
    // Au 10, Te 2 (lower than Au) -> use Te's 2 in Au's place: 10 + 2 = 12.
    // Today's engine reaches 10 + 10 = 20.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 10),
        art("art.auram", 10),
        art("art.terram", 2),
    ];
    assert_eq!(
        casting(&e, &ruleset, "spell.rain_of_stones", false),
        12,
        "ArMDE:12309: a lower Form requisite must replace the main Form score"
    );
}

#[test]
fn requisite_higher_than_both_primary_arts_leaves_the_total_unchanged() {
    // spell.the_crystal_dart: Mu/Te, requisite art.rego (Technique-class). Mu
    // 5, Te 8, Re 20 (higher than Mu) -> the lesser-of rule picks Mu's own 5,
    // so the total is unaffected either way: 5 + 8 = 13. NOTE: this assertion
    // already holds against TODAY's unfolded engine too (10+... no fold run
    // at all still reaches the same arithmetic when the requisite never binds),
    // so this one is NOT currently red — it is a forward confirmation that
    // Phase 2's fold must leave this case alone, not a bug this file catches.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 5),
        art("art.terram", 8),
        art("art.rego", 20),
    ];
    assert_eq!(
        casting(&e, &ruleset, "spell.the_crystal_dart", false),
        13,
        "ArMDE:12309: a higher requisite must never raise or otherwise change the total"
    );
}

// --- Several requisites: ArMDE:12311 ----------------------------------------

#[test]
fn two_form_requisites_in_the_same_category_use_the_lowest_of_the_group() {
    // "if several requisites apply to the same primary Art (for example, if
    // there are two Form requisites), your effective score is the lowest of
    // the group." No shipped spell carries two same-class requisites, so this
    // uses a synthetic one-spell ruleset: Cr/Ig primary, requisites Aquam(7)
    // and Auram(4), both Form-class. Cr 10 (untouched, no Technique
    // requisite), Ig 10, Aq 7, Au 4 -> the Form side folds to the group's
    // lowest (4): 10 + 4 = 14. Today's engine reaches 10 + 10 = 20.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_two_form_requisites", "technique": "art.creo",
             "form": "art.ignem", "level": 20,
             "requisites": ["art.aquam", "art.auram"] }"#,
    );
    let mut e = entity();
    e.art_scores = vec![
        art("art.creo", 10),
        art("art.ignem", 10),
        art("art.aquam", 7),
        art("art.auram", 4),
    ];
    assert_eq!(
        casting(&e, &ruleset, "spell.test_two_form_requisites", false),
        14,
        "ArMDE:12311: two requisites in the same class must fold to the lowest of the group"
    );
}

#[test]
fn requisites_for_both_technique_and_form_are_folded_independently() {
    // "Sometimes a spell has a requisite for both its Technique and Form. You
    // must use the lowest in each case." spell.fog_of_confusion: Mu/Au,
    // requisites [art.imaginem (Form), art.rego (Technique)]. Mu 10, Au 10,
    // Im 3 (lower, Form side), Re 2 (lower, Technique side) -> folded
    // Technique 2 + folded Form 3 = 5. Today's engine reaches 10 + 10 = 20.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 10),
        art("art.auram", 10),
        art("art.imaginem", 3),
        art("art.rego", 2),
    ];
    assert_eq!(
        casting(&e, &ruleset, "spell.fog_of_confusion", false),
        5,
        "ArMDE:12311: a Technique requisite and a Form requisite fold independently"
    );
}

// --- Deficient Art as a requisite: ArMDE:12311 (closing sentence) -----------

#[test]
fn deficient_art_on_a_requisite_only_art_still_halves_the_total() {
    // "Furthermore, any Deficiencies you have with an Art apply when you use
    // that Art as a requisite." spell.obliteration_of_the_metallic_barrier:
    // Pe/Te, requisite art.rego. Pe 10, Te 10, Re 15 (HIGHER than Pe, so the
    // min-rule alone would leave the total unaffected: 10 + 10 = 20) — but
    // Rego is Deficient (flaw.deficient_technique), and using a Deficient Art
    // as a requisite still halves the whole total (ArMDE:5911/:5915's halving
    // reaches a requisite Art, not only a primary one): halve(20) = 10.
    // Today's `InPlayMods::deficient` only looks at the spell's own primary
    // Technique/Form, never its requisites, so today's engine reaches 20
    // unhalved.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.perdo", 10),
        art("art.terram", 10),
        art("art.rego", 15),
    ];
    e.selections = vec![Selection::with_params(
        Id::new("flaw.deficient_technique"),
        BTreeMap::from([("technique".to_string(), Id::new("art.rego"))]),
    )];
    assert_eq!(
        casting(
            &e,
            &ruleset,
            "spell.obliteration_of_the_metallic_barrier",
            false
        ),
        10,
        "ArMDE:12311: a Deficient requisite Art halves the total even when it does not numerically bind"
    );
}

// --- Magical Focus doubling may pick a requisite: ArMDE:4403 ---------------

#[test]
fn magical_focus_doubling_uses_the_folded_lowest_score_including_a_requisite() {
    // "If a spell has requisites, the lowest applicable score may be one of
    // the requisites, rather than one of the primary Arts." Same Arts as the
    // base Technique-requisite test (Pe 10, Te 10, Re 3) but cast within a
    // Major Magical Focus: the folded Technique (3, from the Rego requisite)
    // is also the score the focus doubles. common = 3 + 10 = 13; focus adds
    // min(3, 10) = 3; total = 16. Today's engine folds nothing, so its
    // focus_art is min(10, 10) = 10 and it reaches 10 + 10 + 10 = 30.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.perdo", 10),
        art("art.terram", 10),
        art("art.rego", 3),
    ];
    e.selections = vec![Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new("stone"))]),
    )];
    assert_eq!(
        casting(
            &e,
            &ruleset,
            "spell.obliteration_of_the_metallic_barrier",
            true
        ),
        16,
        "ArMDE:4403: the within-focus double must use the folded (requisite-aware) lowest score"
    );
}

// --- Elemental Magic exception: ArMDE:3737 ----------------------------------

#[test]
fn elemental_magic_uses_the_primary_form_even_when_an_elemental_requisite_is_lower() {
    // "if a spell with one of these Forms as its primary Form has another
    // element as a requisite, you use the primary Form to calculate totals,
    // even if the requisite is lower." spell.rain_of_stones: Mu/Au, requisite
    // art.terram — both Auram and Terram are elemental Forms
    // (virtue.elemental_magic's own `forms` list). Mu 10, Au 10 (bought), Te 2
    // (bought). NOTE: Elemental Magic's own XP-space redistribution
    // (`effective/art.rs::elemental_form_bonus`) lifts Terram's EFFECTIVE
    // score to 7 here (half of Auram's 55 XP, rounded up, added to Terram's
    // own 3 XP, re-resolved against the table) — still lower than Auram's 10,
    // so the exception is still the only thing standing between 20 and the
    // plain-fold answer of 17 (10 + 7).
    //
    // NOT currently red: today's engine folds no requisite at all (for any
    // spell, Elemental Magic or not), so it already reaches 10 + 10 = 20 —
    // the same figure the exception requires, for an unrelated reason. This
    // is a forward guard: Phase 2 must special-case Elemental Magic so this
    // stays at 20 once the plain fold (which alone would pull it down to 17)
    // is implemented.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 10),
        art("art.auram", 10),
        art("art.terram", 2),
    ];
    e.selections = vec![Selection::new(Id::new("virtue.elemental_magic"))];
    assert_eq!(
        casting(&e, &ruleset, "spell.rain_of_stones", false),
        20,
        "ArMDE:3737: Elemental Magic must use the primary elemental Form, never a lower elemental requisite"
    );
}

// --- Puissant Art is compared bonus-inclusive: ArMDE:4820 -------------------

#[test]
fn puissant_art_bonus_is_included_before_comparing_against_the_requisite() {
    // "include the bonus from Puissant Art with that Art when calculating
    // which Art is higher. If the Puissant Art is higher, the bonus does not
    // apply to the requisite." spell.obliteration_of_the_metallic_barrier:
    // Pe/Te, requisite art.rego. Pe bought 5 + Puissant Art (+3) = effective
    // 8; Re (requisite) 6 — between the bought 5 and the effective 8. The
    // comparison must use the PUISSANT-INCLUSIVE 8 against 6, folding to 6 (not
    // the bought 5): 6 + 10 = 16. Today's engine does not fold at all and
    // reaches the unfolded effective 8 + 10 = 18.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.perdo", 5),
        art("art.terram", 10),
        art("art.rego", 6),
    ];
    e.selections = vec![Selection::with_params(
        Id::new("virtue.puissant_art"),
        BTreeMap::from([("art".to_string(), Id::new("art.perdo"))]),
    )];
    assert_eq!(
        casting(
            &e,
            &ruleset,
            "spell.obliteration_of_the_metallic_barrier",
            false
        ),
        16,
        "ArMDE:4820: the requisite comparison must use the Puissant-inclusive primary score"
    );
}
