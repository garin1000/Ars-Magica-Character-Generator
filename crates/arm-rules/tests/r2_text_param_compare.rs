//! R2 (try-out finding 25, D81.9 → D83.2) — free-TEXT parameter values are
//! compared case- and whitespace-insensitively by every check that weighs an
//! item's copies against each other: `"fire"`, `"Fire"` and `" Fire  "` are one
//! value, so a per-target or per-value cap can no longer be dodged by retyping
//! the same word. Only `text`-domain values fold; a registry id (`enumerated`,
//! `ability`, …) and a `multi_ref` set still compare exactly, because an id is
//! not something the player types.
//!
//! Minimal in-test rulesets (`Ruleset::from_sources`), so the engine behaviour
//! is pinned independently of which shipped items declare which caps. The
//! shipped Focus Power half lives in `x6c_label_parameters.rs`.

use std::collections::{BTreeMap, BTreeSet};

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;

/// Every issue code that judges an item's copies against each other.
const REPEAT_CODES: [&str; 3] = [
    "duplicate_selection",
    "too_many_selections",
    "too_many_for_param_value",
];

/// Fixture items. Each repeats freely in total (`max_total: 255`), so
/// `too_many_selections` never fires and the other two repeat checks are
/// observed alone.
///
/// - `virtue.text_once` — one free-text key at the defaults: once per target
///   (`max_per_target` 1) and once per value (`max_per_value` 1), the shape of
///   most shipped repeatable text-parameter items.
/// - `flaw.text_per_value` — Necessary-Aura-shaped: a capped free-text key plus
///   a free realm axis, so two copies can differ in their tuple yet share the
///   text value.
/// - `virtue.enumerated_ids` / `virtue.ability_ids` / `virtue.multi_ids` —
///   registry ids that differ only by case, which must stay two values.
const ITEMS: &str = r#"[
  { "id": "virtue.text_once", "kind": "virtue", "classification": "narrative",
    "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
    "max_total": 255,
    "parameters": [{ "key": "subject", "type": "ref", "domain": "text" }] },
  { "id": "flaw.text_per_value", "kind": "flaw", "classification": "narrative",
    "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
    "max_total": 255,
    "parameters": [
      { "key": "subject", "type": "ref", "domain": "text" },
      { "key": "realm", "type": "ref", "domain": "realm", "max_per_value": 255 }
    ] },
  { "id": "virtue.enumerated_ids", "kind": "virtue", "classification": "narrative",
    "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
    "max_total": 255,
    "parameters": [{ "key": "kind", "type": "ref", "domain": "enumerated",
      "values": ["kind.fire", "kind.Fire"] }] },
  { "id": "virtue.ability_ids", "kind": "virtue", "classification": "narrative",
    "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
    "max_total": 255,
    "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }] },
  { "id": "virtue.multi_ids", "kind": "virtue", "classification": "narrative",
    "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
    "max_total": 255,
    "parameters": [{ "key": "targets", "type": "multi_ref", "domain": "ability" }] },
  { "id": "flaw.ability_per_value", "kind": "flaw", "classification": "narrative",
    "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
    "max_total": 255,
    "parameters": [
      { "key": "ability", "type": "ref", "domain": "ability", "max_per_value": 1 },
      { "key": "realm", "type": "ref", "domain": "realm", "max_per_value": 255 }
    ] },
  { "id": "flaw.personality_filler", "kind": "flaw", "classification": "narrative",
    "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
]"#;

const TYPES: &str = r#"[
  { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
    "permitted_categories": ["general", "personality"], "creation_phases": [] }
]"#;

const ABILITIES: &str = r#"{ "abilities": [
  { "id": "ability.lore", "category": "general" },
  { "id": "ability.Lore", "category": "general" },
  { "id": "ability.craft", "category": "general", "parameter": "craft" }
] }"#;

fn ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: ITEMS,
        type_profiles: TYPES,
        abilities: Some(ABILITIES),
        ..RulesetSources::default()
    })
    .expect("the R2 fixture ruleset loads")
}

fn entity(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("test"), "1"),
    );
    e.selections = selections;
    e
}

fn sel(item: &str, params: &[(&str, &str)]) -> Selection {
    Selection::with_params(
        Id::new(item),
        params
            .iter()
            .map(|(k, v)| (k.to_string(), Id::new(*v)))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn multi_sel(item: &str, key: &str, ids: &[&str]) -> Selection {
    Selection {
        item_ref: Id::new(item),
        params: BTreeMap::from([(
            key.to_string(),
            SelectionParamValue::Multi(ids.iter().map(|id| Id::new(*id)).collect::<BTreeSet<_>>()),
        )]),
    }
}

/// The repeat-check codes raised, in order, one entry per finding.
fn repeat_findings(e: &Entity) -> Vec<String> {
    validate(e, &ruleset())
        .issues
        .into_iter()
        .map(|i| i.code)
        .filter(|c| REPEAT_CODES.contains(&c.as_str()))
        .collect()
}

// --- free text folds: one value, so one finding ------------------------------

/// "fire" and "Fire" are one subject: two copies on a once-per-target item are
/// a duplicate, reported ONCE (by `max_per_target`, the whole-tuple check — the
/// per-value check must not report the same mistake a second time).
#[test]
fn text_values_differing_only_in_case_are_one_duplicate() {
    let e = entity(vec![
        sel("virtue.text_once", &[("subject", "fire")]),
        sel("virtue.text_once", &[("subject", "Fire")]),
    ]);
    assert_eq!(repeat_findings(&e), vec!["duplicate_selection".to_string()]);
}

/// Leading and trailing padding is not part of the value.
#[test]
fn text_values_differing_only_in_padding_are_one_duplicate() {
    let e = entity(vec![
        sel("virtue.text_once", &[("subject", "Fire")]),
        sel("virtue.text_once", &[("subject", " Fire  ")]),
    ]);
    assert_eq!(repeat_findings(&e), vec!["duplicate_selection".to_string()]);
}

/// Inner runs of whitespace collapse to one space: "hot fire" and "Hot   Fire"
/// are one subject.
#[test]
fn text_values_differing_only_in_inner_whitespace_are_one_duplicate() {
    let e = entity(vec![
        sel("virtue.text_once", &[("subject", "hot fire")]),
        sel("virtue.text_once", &[("subject", "Hot   Fire")]),
    ]);
    assert_eq!(repeat_findings(&e), vec!["duplicate_selection".to_string()]);
}

/// Genuinely different text stays different: no finding.
#[test]
fn different_text_values_are_two_targets() {
    let e = entity(vec![
        sel("virtue.text_once", &[("subject", "fire")]),
        sel("virtue.text_once", &[("subject", "water")]),
    ]);
    assert_eq!(repeat_findings(&e), Vec::<String>::new());
}

/// The per-value axis folds too: two copies whose tuples differ (two Realms)
/// but whose capped text value is one subject spelled two ways exceed
/// `max_per_value` 1 — exactly one `too_many_for_param_value`.
#[test]
fn per_value_cap_counts_case_variants_as_one_value() {
    let e = entity(vec![
        sel(
            "flaw.text_per_value",
            &[("subject", "fire"), ("realm", "realm.divine")],
        ),
        sel(
            "flaw.text_per_value",
            &[("subject", " FIRE "), ("realm", "realm.magic")],
        ),
    ]);
    assert_eq!(
        repeat_findings(&e),
        vec!["too_many_for_param_value".to_string()]
    );
}

/// Two copies whose WHOLE tuples fold to the same one are `max_per_target`'s
/// finding alone: the per-value check must group tuples by their folded form
/// too, or a case variant would be counted as a second tuple and the one
/// mistake would draw two findings.
#[test]
fn folded_identical_tuples_draw_one_finding_not_two() {
    let e = entity(vec![
        sel(
            "flaw.text_per_value",
            &[("subject", "fire"), ("realm", "realm.divine")],
        ),
        sel(
            "flaw.text_per_value",
            &[("subject", "Fire"), ("realm", "realm.divine")],
        ),
    ]);
    assert_eq!(repeat_findings(&e), vec!["duplicate_selection".to_string()]);
}

// --- a parameterized Ability's instance text folds too (R2 Q-R2-2) -----------

/// Craft (Carpentry) and Craft ( carpentry ) are one Ability: the instance
/// key is free text the player types, so a per-Ability cap (Necessary Aura's
/// "once for any particular Ability", ArMDE:6482) counts them as one target.
#[test]
fn ability_instance_text_case_variants_are_one_ability_per_value() {
    let e = entity(vec![
        sel(
            "flaw.ability_per_value",
            &[
                ("ability", "ability.craft"),
                ("craft", "Carpentry"),
                ("realm", "realm.divine"),
            ],
        ),
        sel(
            "flaw.ability_per_value",
            &[
                ("ability", "ability.craft"),
                ("craft", " carpentry "),
                ("realm", "realm.magic"),
            ],
        ),
    ]);
    assert_eq!(
        repeat_findings(&e),
        vec!["too_many_for_param_value".to_string()]
    );
}

/// The same pair under ONE Realm is a whole-tuple duplicate: one finding,
/// `max_per_target`'s, not a second from the per-value axis.
#[test]
fn ability_instance_text_case_variants_in_one_tuple_are_one_duplicate() {
    let e = entity(vec![
        sel(
            "flaw.ability_per_value",
            &[
                ("ability", "ability.craft"),
                ("craft", "Carpentry"),
                ("realm", "realm.divine"),
            ],
        ),
        sel(
            "flaw.ability_per_value",
            &[
                ("ability", "ability.craft"),
                ("craft", "carpentry"),
                ("realm", "realm.divine"),
            ],
        ),
    ]);
    assert_eq!(repeat_findings(&e), vec!["duplicate_selection".to_string()]);
}

/// Control: two genuinely different Crafts stay two Abilities.
#[test]
fn different_ability_instances_stay_two_abilities() {
    let e = entity(vec![
        sel(
            "flaw.ability_per_value",
            &[
                ("ability", "ability.craft"),
                ("craft", "Carpentry"),
                ("realm", "realm.divine"),
            ],
        ),
        sel(
            "flaw.ability_per_value",
            &[
                ("ability", "ability.craft"),
                ("craft", "Blacksmith"),
                ("realm", "realm.magic"),
            ],
        ),
    ]);
    assert_eq!(repeat_findings(&e), Vec::<String>::new());
}

// --- ids do not fold ---------------------------------------------------------

/// An `enumerated` value is an id, compared exactly: two ids that differ only
/// by case are two values, so neither repeat check fires.
#[test]
fn enumerated_ids_differing_in_case_stay_distinct() {
    let e = entity(vec![
        sel("virtue.enumerated_ids", &[("kind", "kind.fire")]),
        sel("virtue.enumerated_ids", &[("kind", "kind.Fire")]),
    ]);
    assert_eq!(repeat_findings(&e), Vec::<String>::new());
}

/// An `ability`-domain value is an id, compared exactly.
#[test]
fn ability_ids_differing_in_case_stay_distinct() {
    let e = entity(vec![
        sel("virtue.ability_ids", &[("ability", "ability.lore")]),
        sel("virtue.ability_ids", &[("ability", "ability.Lore")]),
    ]);
    assert_eq!(repeat_findings(&e), Vec::<String>::new());
}

/// A `multi_ref` set of ids is compared exactly.
#[test]
fn multi_ref_ids_differing_in_case_stay_distinct() {
    let e = entity(vec![
        multi_sel("virtue.multi_ids", "targets", &["ability.lore"]),
        multi_sel("virtue.multi_ids", "targets", &["ability.Lore"]),
    ]);
    assert_eq!(repeat_findings(&e), Vec::<String>::new());
}

/// Control for the three above: the SAME id twice is still a duplicate, so
/// "no finding" there means "two values", not "the check is off".
#[test]
fn identical_ids_are_still_a_duplicate() {
    let e = entity(vec![
        sel("virtue.enumerated_ids", &[("kind", "kind.fire")]),
        sel("virtue.enumerated_ids", &[("kind", "kind.fire")]),
    ]);
    assert_eq!(repeat_findings(&e), vec!["duplicate_selection".to_string()]);
}
