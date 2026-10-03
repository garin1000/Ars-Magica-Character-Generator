//! R4 (`tmp/bve-sweep.md` S2 and F-B, Norbert's rulings 2026-10-03): which
//! score a minimum-score prerequisite compares against.
//!
//! **S2 — `Prereq::CharacteristicMin`.** A Characteristic with no stored entry
//! is a real 0, plus every free delta a Virtue or Flaw puts on it. The UI
//! deletes the entry at 0, so "never touched" and "set to 0" cannot be told
//! apart, and D81.2's "unset is Unknown" turned a refusal into a mere
//! `prereq_unevaluated` warning (and warned on legal characters).
//! ArMDE:5095 (Supernatural Beauty): "A character lacking a positive Presence
//! score may not have this Virtue." ArMDE:6909 (Uncontrollable Strength):
//! "This Flaw may not be taken if the character's Strength is below 0."
//! ArMDE:5998 (Dwarf): "You take a -1 penalty to each of Strength and
//! Stamina".
//!
//! **F-B — `Prereq::AbilityMin` / `Prereq::ArtMin`.** A Virtue's minimum tests
//! the score the character HOLDS (bought, or conferred by a grant such as
//! Second Sight 1, ArMDE:4890), not a bonus that applies only when the Ability
//! or Art is used. ArMDE:4816 (Puissant Ability): "add 2 to its value whenever
//! you use it". ArMDE:4820 (Puissant Art): "You add 3 to the value of one Art
//! whenever you use it." ArMDE:4389 (Magister in Artibus): "must have scores
//! of at least 5 in Latin and Artes Liberales" — holding a degree is not a use.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use arm_rules::{Characteristic, effective_ability_score, effective_characteristic_score};
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
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

fn companion(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn sel_with(id: &str, params: &[(&str, &str)]) -> Selection {
    Selection::with_params(
        Id::new(id),
        params
            .iter()
            .map(|&(key, value)| (key.to_string(), Id::new(value)))
            .collect::<BTreeMap<_, _>>(),
    )
}

/// The issue codes raised against `item_id`'s own prerequisite — scoped by
/// `context` so an unrelated issue on another selection cannot satisfy or
/// spoil an assertion.
fn prereq_codes_for(e: &Entity, rs: &Ruleset, item_id: &str) -> Vec<String> {
    let item = Id::new(item_id);
    validate(e, rs)
        .issues
        .into_iter()
        .filter(|issue| issue.context.as_ref() == Some(&item))
        .map(|issue| issue.code)
        .filter(|code| code == "prereq_not_met" || code == "prereq_unevaluated")
        .collect()
}

// ---------------------------------------------------------------------------
// S2 — an unset Characteristic is 0 plus free deltas.
// ---------------------------------------------------------------------------

#[test]
fn unset_presence_is_zero_so_supernatural_beauty_is_refused() {
    let rs = shipped_ruleset();
    let e = companion(vec![sel("virtue.supernatural_beauty")]);
    assert!(
        !e.characteristics.contains_key(&Characteristic::Pre),
        "precondition: Presence was never set"
    );

    let codes = prereq_codes_for(&e, &rs, "virtue.supernatural_beauty");
    assert_eq!(
        codes,
        vec!["prereq_not_met".to_string()],
        "ArMDE:5095: an unset Presence is 0, which is not positive, so Supernatural \
         Beauty must be refused outright (an error, not an unevaluated warning)"
    );
}

#[test]
fn unset_strength_is_zero_so_uncontrollable_strength_is_legal() {
    let rs = shipped_ruleset();
    let e = companion(vec![sel("flaw.uncontrollable_strength")]);
    assert!(
        !e.characteristics.contains_key(&Characteristic::Str),
        "precondition: Strength was never set"
    );

    let codes = prereq_codes_for(&e, &rs, "flaw.uncontrollable_strength");
    assert!(
        codes.is_empty(),
        "ArMDE:6909: an unset Strength is 0, which is not below 0, so Uncontrollable \
         Strength is legal with no prerequisite issue at all, got: {codes:?}"
    );
}

#[test]
fn unset_strength_with_dwarf_counts_its_free_delta_and_refuses_uncontrollable_strength() {
    let rs = shipped_ruleset();
    let e = companion(vec![sel("flaw.dwarf"), sel("flaw.uncontrollable_strength")]);
    assert!(
        !e.characteristics.contains_key(&Characteristic::Str),
        "precondition: Strength was never set"
    );
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Str),
        -1,
        "precondition: Dwarf's free -1 (ArMDE:5998) on an unset Strength"
    );

    let codes = prereq_codes_for(&e, &rs, "flaw.uncontrollable_strength");
    assert_eq!(
        codes,
        vec!["prereq_not_met".to_string()],
        "ArMDE:6909: an unset Strength under Dwarf is 0 - 1 = -1, below 0, so \
         Uncontrollable Strength must be refused"
    );
}

#[test]
fn unset_presence_with_magical_blood_counts_its_free_delta_as_one() {
    let rs = shipped_ruleset();
    let e = companion(vec![
        sel_with(
            "virtue.magical_blood",
            &[
                ("bloodline", "bloodline.magic_human"),
                ("characteristic", "characteristic.pre"),
            ],
        ),
        sel("virtue.supernatural_beauty"),
    ]);
    assert!(
        !e.characteristics.contains_key(&Characteristic::Pre),
        "precondition: Presence was never set"
    );
    assert_eq!(
        effective_characteristic_score(&e, &rs, Characteristic::Pre),
        1,
        "precondition: Magical Blood (Magic Human, Presence) adds a free +1 to an unset Presence"
    );

    let codes = prereq_codes_for(&e, &rs, "virtue.supernatural_beauty");
    assert!(
        codes.is_empty(),
        "ArMDE:5095: an unset Presence plus Magical Blood's free +1 is a positive 1, \
         so Supernatural Beauty is legal with no prerequisite issue, got: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// F-B — Ability/Art minimums test the held score, not Puissant.
// ---------------------------------------------------------------------------

#[test]
fn magister_in_artibus_with_puissant_artes_liberales_below_five_is_refused() {
    let rs = shipped_ruleset();
    let mut e = companion(vec![
        sel("virtue.magister_in_artibus"),
        sel_with(
            "virtue.puissant_ability",
            &[("ability", "ability.artes_liberales")],
        ),
    ]);
    e.ability_scores = vec![
        AbilityScore::new(Id::new("ability.dead_language"), 5),
        AbilityScore::new(Id::new("ability.artes_liberales"), 3),
    ];
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.artes_liberales"), None),
        5,
        "precondition: bought 3 + Puissant 2 is an effective 5"
    );

    let codes = prereq_codes_for(&e, &rs, "virtue.magister_in_artibus");
    assert_eq!(
        codes,
        vec!["prereq_not_met".to_string()],
        "ArMDE:4389 + ArMDE:4816: Puissant applies 'whenever you use it', and holding a \
         degree is not a use, so a bought Artes Liberales 3 does not meet the minimum of 5"
    );
}

#[test]
fn magister_in_artibus_with_bought_five_and_puissant_is_legal() {
    let rs = shipped_ruleset();
    let mut e = companion(vec![
        sel("virtue.magister_in_artibus"),
        sel_with(
            "virtue.puissant_ability",
            &[("ability", "ability.artes_liberales")],
        ),
    ]);
    e.ability_scores = vec![
        AbilityScore::new(Id::new("ability.dead_language"), 5),
        AbilityScore::new(Id::new("ability.artes_liberales"), 5),
    ];

    let codes = prereq_codes_for(&e, &rs, "virtue.magister_in_artibus");
    assert!(
        codes.is_empty(),
        "ArMDE:4389: a bought Artes Liberales 5 meets the minimum (Puissant on top is \
         harmless), got: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// F-B extended to Broken Vessel (orchestrator's Q1 ruling): ArMDE:5755, "at
// least one Supernatural Ability or Art normally improved through experience
// points", is a holding test too, so `AbilityCategoryScoreMin`/`AnyArtMin`
// read the same held score (bought or granted, never Puissant).
// ---------------------------------------------------------------------------

#[test]
fn broken_vessel_is_not_satisfied_by_puissant_on_an_unscored_supernatural_ability() {
    let rs = shipped_ruleset();
    let mut e = companion(vec![
        sel("flaw.broken_vessel"),
        sel_with("virtue.puissant_ability", &[("ability", "ability.dowsing")]),
    ]);
    e.ability_scores = vec![AbilityScore::new(Id::new("ability.dowsing"), 0)];
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.dowsing"), None),
        2,
        "precondition: bought 0 + Puissant 2 is an effective 2"
    );

    let codes = prereq_codes_for(&e, &rs, "flaw.broken_vessel");
    assert_eq!(
        codes,
        vec!["prereq_not_met".to_string()],
        "ArMDE:5755 + ArMDE:4816: a Dowsing score of 0 lifted only by Puissant is not a \
         held Supernatural Ability"
    );
}

#[test]
fn broken_vessel_is_not_satisfied_by_puissant_art_on_an_unscored_art() {
    let rs = shipped_ruleset();
    let mut e = companion(vec![
        sel("flaw.broken_vessel"),
        sel_with("virtue.puissant_art", &[("art", "art.creo")]),
    ]);
    e.type_id = Id::new("magus");
    e.art_scores = vec![ArtScore::new(Id::new("art.creo"), 0)];

    let codes = prereq_codes_for(&e, &rs, "flaw.broken_vessel");
    assert_eq!(
        codes,
        vec!["prereq_not_met".to_string()],
        "ArMDE:5755 + ArMDE:4820: a Creo score of 0 lifted only by Puissant Art is not a \
         held Art"
    );
}

#[test]
fn broken_vessel_is_satisfied_by_a_granted_supernatural_ability() {
    let rs = shipped_ruleset();
    let e = companion(vec![sel("flaw.broken_vessel"), sel("virtue.second_sight")]);
    assert!(e.ability_scores.is_empty(), "precondition: nothing bought");

    let codes = prereq_codes_for(&e, &rs, "flaw.broken_vessel");
    assert!(
        codes.is_empty(),
        "ArMDE:4890: Second Sight confers the Ability Second Sight 1, a held Supernatural \
         Ability, got: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// F-B with a test ruleset: a grant floor counts, Puissant on it does not, and
// the same holds for Arts. No shipped `AbilityMin` names a granted Ability and
// no shipped item carries an `ArtMin`, so these need their own fixture.
// ---------------------------------------------------------------------------

const FIXTURE_ITEMS: &str = r#"[
  { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major",
    "categories": ["personality"], "entity_kinds": ["character"] },
  { "id": "virtue.grants_awareness", "kind": "virtue", "classification": "creation_effect", "magnitude": "minor",
    "categories": ["general"], "entity_kinds": ["character"],
    "effects": [{ "type": "ability_score_grant", "ability": "ability.awareness", "amount": 1 }] },
  { "id": "virtue.puissant_ability", "kind": "virtue", "classification": "creation_effect", "magnitude": "minor",
    "categories": ["general"], "entity_kinds": ["character"],
    "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
    "effects": [{ "type": "ability_bonus", "param": "ability", "amount": 2 }] },
  { "id": "virtue.puissant_art", "kind": "virtue", "classification": "creation_effect", "magnitude": "minor",
    "categories": ["general"], "entity_kinds": ["character"],
    "parameters": [{ "key": "art", "type": "ref", "domain": "art" }],
    "effects": [{ "type": "art_bonus", "param": "art", "amount": 3 }] },
  { "id": "virtue.needs_awareness_1", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
    "categories": ["general"], "entity_kinds": ["character"],
    "prerequisites": { "kind": "ability_min", "value": { "ability": "ability.awareness", "score": 1 } } },
  { "id": "virtue.needs_awareness_3", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
    "categories": ["general"], "entity_kinds": ["character"],
    "prerequisites": { "kind": "ability_min", "value": { "ability": "ability.awareness", "score": 3 } } },
  { "id": "virtue.needs_creo_5", "kind": "virtue", "classification": "narrative", "magnitude": "minor",
    "categories": ["general"], "entity_kinds": ["character"],
    "prerequisites": { "kind": "art_min", "value": { "art": "art.creo", "score": 5 } } }
]"#;

const FIXTURE_TYPES: &str = r#"[{
  "id": "companion",
  "budget": { "virtue_points": 10, "flaw_points": 10 },
  "permitted_categories": ["general"],
  "creation_phases": []
}]"#;

const FIXTURE_ABILITIES: &str = r#"{ "abilities": [
  { "id": "ability.awareness", "category": "general" }
] }"#;

const FIXTURE_ARTS: &str = r#"{
  "advancement": [
    { "score": 1, "total_xp": 1 }, { "score": 2, "total_xp": 3 },
    { "score": 3, "total_xp": 6 }, { "score": 4, "total_xp": 10 },
    { "score": 5, "total_xp": 15 }, { "score": 6, "total_xp": 21 }
  ],
  "arts": [ { "id": "art.creo", "art_type": "technique" } ]
}"#;

fn fixture_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: FIXTURE_ITEMS,
        type_profiles: FIXTURE_TYPES,
        abilities: Some(FIXTURE_ABILITIES),
        arts: Some(FIXTURE_ARTS),
        ..RulesetSources::default()
    })
    .expect("R4 fixture ruleset loads")
}

#[test]
fn a_grant_floor_satisfies_an_ability_min() {
    let rs = fixture_ruleset();
    let e = companion(vec![
        sel("virtue.grants_awareness"),
        sel("virtue.needs_awareness_1"),
    ]);
    assert!(e.ability_scores.is_empty(), "precondition: nothing bought");

    let codes = prereq_codes_for(&e, &rs, "virtue.needs_awareness_1");
    assert!(
        codes.is_empty(),
        "a conferred score (ArMDE:4890's 'confers the Ability Second Sight 1' shape) is a \
         held score and meets AbilityMin 1, got: {codes:?}"
    );
}

#[test]
fn puissant_on_a_grant_floor_does_not_lift_it_over_an_ability_min() {
    let rs = fixture_ruleset();
    let e = companion(vec![
        sel("virtue.grants_awareness"),
        sel_with(
            "virtue.puissant_ability",
            &[("ability", "ability.awareness")],
        ),
        sel("virtue.needs_awareness_3"),
    ]);
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.awareness"), None),
        3,
        "precondition: granted 1 + Puissant 2 is an effective 3"
    );

    let codes = prereq_codes_for(&e, &rs, "virtue.needs_awareness_3");
    assert_eq!(
        codes,
        vec!["prereq_not_met".to_string()],
        "ArMDE:4816: the held score is the granted 1; Puissant's +2 applies only when the \
         Ability is used, so AbilityMin 3 is not met"
    );
}

#[test]
fn puissant_art_does_not_count_toward_an_art_min() {
    let rs = fixture_ruleset();
    let mut e = companion(vec![
        sel_with("virtue.puissant_art", &[("art", "art.creo")]),
        sel("virtue.needs_creo_5"),
    ]);
    e.xp_pool = 100;
    e.art_scores = vec![ArtScore::new(Id::new("art.creo"), 3)];

    let codes = prereq_codes_for(&e, &rs, "virtue.needs_creo_5");
    assert_eq!(
        codes,
        vec!["prereq_not_met".to_string()],
        "ArMDE:4820: Puissant Art adds 3 'whenever you use it'; a bought Creo 3 does not \
         meet ArtMin 5 even though its effective value is 6"
    );
}

#[test]
fn a_bought_art_at_the_minimum_satisfies_an_art_min() {
    let rs = fixture_ruleset();
    let mut e = companion(vec![sel("virtue.needs_creo_5")]);
    e.xp_pool = 100;
    e.art_scores = vec![ArtScore::new(Id::new("art.creo"), 5)];

    let codes = prereq_codes_for(&e, &rs, "virtue.needs_creo_5");
    assert!(
        codes.is_empty(),
        "a bought Creo 5 meets ArtMin 5, got: {codes:?}"
    );
}
