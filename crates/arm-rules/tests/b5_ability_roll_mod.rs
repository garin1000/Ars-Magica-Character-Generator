//! B5 (`docs/vf-audit/design-b0-ranging-and-predicates.md` § 1/§ 2/§ 3b row 11/
//! § 5/§ 8, F-489).
//!
//! `Effect::AbilityRollMod` (parameter-relative: Academic Concentration's
//! free-text subject) is renamed to `Effect::AbilityRollModParam`, and the
//! base name `Effect::AbilityRollMod { ability, amount }` now names a FIXED
//! Ability for a roll-only modifier — Poor Hearing's "Subtract 3 from rolls
//! involving hearing" (ArMDE:6614-6617), which the parameter-relative shape
//! cannot express (there is no parameter to read the target Ability from; the
//! Flaw itself fixes it). Same naming-collision convention `AbilityScoreGrant`/
//! `AbilityScoreGrantParam` and `CharacteristicScoreDelta`/
//! `CharacteristicScoreDeltaParam` already use.
//!
//! Ruleset-JSON-only rename (design § 6): no `SCHEMA_VERSION` bump. The one
//! known carrier, `virtue.academic_concentration_subject`, is updated in the
//! same commit (`rules/core/virtues_flaws.json`) so it keeps resolving under
//! the new `ability_roll_mod_param` tag.
//!
//! Both variants are surfaced-only (5i): neither ever perturbs a bought or
//! effective Ability score.
//!
//! RED-CHECKPOINT PROTOCOL, phase 1: `Effect::AbilityRollMod`'s fixed-target
//! shape is declared but every consuming match arm is a documented no-op
//! (`derived.rs::in_play_mods`, `ruleset/integrity.rs::validate_effect_refs`) —
//! deliberately, so the tests below fail for the right reason (missing
//! behavior, not a compile error or an unrelated cascade).

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection};
use arm_rules::{ModifierFamily, effective_ability_score, surfaced_modifiers};
use std::collections::BTreeMap;

/// `ruleset/integrity.rs::validate_engine_required_categories`: any non-empty
/// point-item catalogue must carry at least one `personality`-category entry.
/// Mirrors `b4_parameter_gated_effects.rs`'s own `PERSONALITY_FILLER`.
const PERSONALITY_FILLER: &str = r#"{ "id": "flaw.filler_personality", "kind": "flaw",
  "classification": "narrative", "magnitude": "minor", "categories": ["personality"],
  "entity_kinds": ["character"] }"#;

const COMPANION_TYPE: &str = r#"[
  { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
    "creation_phases": ["virtues_flaws"] }
]"#;

const ABILITIES_JSON: &str = r#"{ "abilities": [
  { "id": "ability.awareness", "category": "general" },
  { "id": "ability.athletics", "category": "general" }
] }"#;

fn companion(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("test"), "1"),
    );
    e.selections = selections;
    e
}

// --- (a) hand-authored fixture: the fixed-target shape, Poor-Hearing-like ---

fn ruleset_with_fixed_roll_mod() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "flaw.tester_poor_hearing", "kind": "flaw", "classification": "in_play_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [
          {{ "type": "ability_roll_mod", "ability": "ability.awareness", "amount": -3 }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    Ruleset::from_json_with_abilities("test", "1", &items, COMPANION_TYPE, ABILITIES_JSON)
        .expect("hand-authored fixture must load")
}

/// The fixed-target shape surfaces for the ability it names. Fails today:
/// `Effect::AbilityRollMod` has no `ability` field to deserialize yet (still
/// the parameter-relative shape), so the fixture itself fails to load; once
/// the field exists, `derived.rs`'s stub arm still surfaces nothing.
///
/// Coordinator review (post-phase-1): the target reaches the UI through a
/// STRUCTURED `SurfacedModifier::ability: Option<Id>` field, never through the
/// free-text `detail` — a raw catalogue id must never sit in a field the UI
/// renders verbatim (CLAUDE.md). `detail` stays reserved for a parameter-based
/// row's player-typed subject (Academic Concentration).
#[test]
fn fixed_ability_roll_mod_surfaces_for_the_named_ability() {
    let rs = ruleset_with_fixed_roll_mod();
    let entity = companion(vec![Selection::new(Id::new("flaw.tester_poor_hearing"))]);
    let s = surfaced_modifiers(&entity, &rs);
    assert!(
        s.iter().any(|m| m.family == ModifierFamily::AbilityRoll
            && m.ability == Some(Id::new("ability.awareness"))
            && m.amount == -3
            && m.source == Some(Id::new("flaw.tester_poor_hearing"))),
        "a fixed ability_roll_mod naming ability.awareness must surface -3 for it, in the \
         structured `ability` field — surfaced: {s:?}"
    );
}

/// Control: the fixed target names ONLY the one Ability it declares — a
/// different Ability (never named by this entry) must never show up under
/// the same family's structured field.
#[test]
fn fixed_ability_roll_mod_does_not_surface_for_a_different_ability() {
    let rs = ruleset_with_fixed_roll_mod();
    let entity = companion(vec![Selection::new(Id::new("flaw.tester_poor_hearing"))]);
    let s = surfaced_modifiers(&entity, &rs);
    assert!(
        !s.iter().any(|m| m.family == ModifierFamily::AbilityRoll
            && m.ability == Some(Id::new("ability.athletics"))),
        "a fixed ability_roll_mod naming ability.awareness must never surface for \
         ability.athletics — surfaced: {s:?}"
    );
}

/// Surfaced-only (design § 1): the fixed-target modifier must never perturb
/// the bought OR effective Ability score it names.
#[test]
fn fixed_ability_roll_mod_does_not_change_bought_or_effective_ability_score() {
    let rs = ruleset_with_fixed_roll_mod();
    let mut entity = companion(vec![Selection::new(Id::new("flaw.tester_poor_hearing"))]);
    entity.ability_scores.push(AbilityScore {
        ability: Id::new("ability.awareness"),
        score: 3,
        specialty: None,
        parameter: None,
    });
    let bought = entity.ability_scores[0].score;
    let effective = effective_ability_score(&entity, &rs, &Id::new("ability.awareness"), None);
    assert_eq!(
        bought, 3,
        "the bought score itself must never be touched by this effect"
    );
    assert_eq!(
        effective, 3,
        "a fixed ability_roll_mod must never change the bought/effective Ability score \
         it targets — effective: {effective}"
    );
}

// --- (b) load-time integrity: a dangling fixed `ability` must be refused ---

/// A fixed `ability_roll_mod` naming an Ability outside the catalogue must
/// fail to load, exactly like `AbilityScoreGrant`'s own fixed-`ability` check.
/// Fails today: no such check exists yet for this variant (documented stub in
/// `ruleset/integrity.rs::validate_effect_refs`).
#[test]
fn ruleset_load_rejects_a_dangling_fixed_ability_on_ability_roll_mod() {
    let items = format!(
        r#"[
      {{ "id": "flaw.tester_dangling_roll_mod", "kind": "flaw", "classification": "in_play_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [
          {{ "type": "ability_roll_mod", "ability": "ability.no_such_ability", "amount": -3 }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let result =
        Ruleset::from_json_with_abilities("test", "1", &items, COMPANION_TYPE, ABILITIES_JSON);
    assert!(
        result.is_err(),
        "a fixed ability_roll_mod naming an unknown ability must fail to load"
    );
}

// --- (c) the shipped `flaw.poor_hearing` (F-489's worked example) ----------

fn load_shipped_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("the shipped ruleset must load")
}

/// A data red (design § 7/§ 8's B5 row): `flaw.poor_hearing` (ArMDE:6614-6617)
/// ships `narrative` with no effects today. Fails until B5's own worked
/// example authors the effect and reclassifies the entry to `in_play_effect`.
/// Checks the STRUCTURED `ability` field (coordinator review), not `detail`.
#[test]
fn shipped_poor_hearing_surfaces_minus_three_to_awareness() {
    let rs = load_shipped_ruleset();
    let entity = companion(vec![Selection::new(Id::new("flaw.poor_hearing"))]);
    let s = surfaced_modifiers(&entity, &rs);
    assert!(
        s.iter().any(|m| m.family == ModifierFamily::AbilityRoll
            && m.ability == Some(Id::new("ability.awareness"))
            && m.amount == -3
            && m.source == Some(Id::new("flaw.poor_hearing"))),
        "flaw.poor_hearing (ArMDE:6614-6617) must surface -3 to Awareness rolls once its \
         fixed-target ability_roll_mod effect is authored, in the structured `ability` field \
         — surfaced: {s:?}"
    );
}

// --- (d) regression: the renamed param-based shape keeps working ----------

/// `virtue.academic_concentration_subject` (ArMDE:3362-3367) must still
/// resolve its +3 subject bonus after the `ability_roll_mod` ->
/// `ability_roll_mod_param` rename — the ONE known carrier the rename
/// touches (design § 1/§ 8), updated in the same commit as the Rust rename.
#[test]
fn shipped_academic_concentration_still_resolves_under_the_renamed_param_tag() {
    let rs = load_shipped_ruleset();
    let entity = companion(vec![Selection::with_params(
        Id::new("virtue.academic_concentration_subject"),
        BTreeMap::from([("subject".into(), Id::new("theology"))]),
    )]);
    let s = surfaced_modifiers(&entity, &rs);
    assert!(
        s.iter().any(|m| m.family == ModifierFamily::AbilityRoll
            && m.detail == "theology"
            && m.amount == 3
            && m.source == Some(Id::new("virtue.academic_concentration_subject"))
            // The structured field is reserved for the FIXED-target shape
            // (coordinator review) — a parameter-based row must never
            // populate it, even once the fixed shape is fully wired.
            && m.ability.is_none()),
        "virtue.academic_concentration_subject must still resolve its +3 subject bonus under \
         the renamed ability_roll_mod_param tag, with the structured `ability` field absent \
         — surfaced: {s:?}"
    );
}
