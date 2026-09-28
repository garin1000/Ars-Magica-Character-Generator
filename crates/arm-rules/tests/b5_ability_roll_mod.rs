//! B5 (`docs/vf-audit/design-b0-ranging-and-predicates.md` § 1/§ 2/§ 3b row 11/
//! § 5/§ 8, F-489).
//!
//! `Effect::AbilityRollMod` (parameter-relative: Academic Concentration's
//! free-text subject) is renamed to `Effect::AbilityRollModParam`, and the
//! base name `Effect::AbilityRollMod { ability, amount }` now names a FIXED
//! Ability for a roll-only modifier, for a passage that names one Ability
//! with no further condition. Same naming-collision convention
//! `AbilityScoreGrant`/`AbilityScoreGrantParam` and
//! `CharacteristicScoreDelta`/`CharacteristicScoreDeltaParam` already use.
//!
//! **D61 correction (`docs/vf-audit/decisions.md`, 2026-09-28): the mechanism's
//! worked example is NOT Poor Hearing.** B5's original design picked
//! `flaw.poor_hearing` ("Subtract 3 from rolls involving hearing",
//! ArMDE:6614-6617) as its worked example, which is wrong: "rolls involving
//! hearing" is conditioned on a SENSE, exactly like Corrupted's "selfish or
//! sinful" (D15) — a table judgement, not a fixed Ability, since it hits
//! Awareness rolls made by sight or smell just as wrongly as it misses
//! non-Awareness hearing rolls. Sense/situation-conditioned modifiers stay
//! `uncomputed_rule` text (see `shipped_poor_hearing_stays_textual_per_d61`
//! below, and its twin `shipped_convoluted_mind_stays_textual_per_d61` —
//! Convoluted Mind's "+3 on all Infernal Lore rolls **to determine what a
//! demon will do**" narrows the same way).
//!
//! The mechanism's real, genuine carriers are three Flaws whose passage names
//! one Ability with NO further condition: `flaw.poor_concentration` (-3
//! Concentration, ArMDE:6602-6605), `flaw.inconstant_magic` (-3 Finesse,
//! ArMDE:6298-6301), and `flaw.clumsy_magic` (-3 Finesse, ArMDE:5801-5804).
//!
//! Ruleset-JSON-only rename (design § 6): no `SCHEMA_VERSION` bump. The one
//! known parameter-relative carrier, `virtue.academic_concentration_subject`,
//! is updated in the same commit (`rules/core/virtues_flaws.json`) so it
//! keeps resolving under the new `ability_roll_mod_param` tag.
//!
//! Both variants are surfaced-only (5i): neither ever perturbs a bought or
//! effective Ability score.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityScore, Classification, Entity, EntityKind, Id, RulesetRef, Selection,
};
use arm_rules::{ModifierFamily, effective_ability_score, surfaced_modifiers};
use serde_json::Value;
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
        // D3: `flaw.abandoned_apprentice` now carries `TruncatedApprenticeshipXp`,
        // which requires an `apprenticeship` block to bound its parameter
        // against — the shipped ruleset always ships one.
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("the shipped ruleset must load")
}

// --- (e) D61: sense/situation-conditioned modifiers stay text-only ---------

/// Shared assertion for a D61 sense/situation-conditioned entry: it must
/// carry NO computed effect at all — `uncomputed_rule`, no surfaced
/// modifier — and its rule must reach the player as `description` text, in
/// every shipped locale, containing the given phrase verbatim.
fn assert_shipped_uncomputed_roll_text(id: &str, checks: &[(&str, &str, &str)]) {
    let rs = load_shipped_ruleset();
    let item = rs
        .item(&Id::new(id))
        .unwrap_or_else(|| panic!("{id} must be in the shipped catalogue"));
    assert_eq!(
        item.classification,
        Classification::UncomputedRule,
        "D61: a sense/situation-conditioned modifier is a table judgement, so {id} must be \
         classified uncomputed_rule, not {:?}",
        item.classification
    );

    let entity = companion(vec![Selection::new(Id::new(id))]);
    let s = surfaced_modifiers(&entity, &rs);
    assert!(
        !s.iter().any(|m| m.source == Some(Id::new(id))),
        "D61: {id} must surface NO computed modifier at all (the rule is text-only) — \
         surfaced: {s:?}"
    );

    for (lang, i18n_json, rule_phrase) in checks {
        let entries: BTreeMap<String, Value> =
            serde_json::from_str(i18n_json).expect("i18n virtues_flaws.json is valid JSON");
        let entry = entries
            .get(id)
            .unwrap_or_else(|| panic!("{id} must have an i18n entry"));
        let description = entry
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(
            description.contains(rule_phrase),
            "D61: {lang}/{id}'s `description` must carry the rule verbatim (expected to \
             contain {rule_phrase:?}) — description: {description:?}"
        );
    }
}

/// D61 (`docs/vf-audit/decisions.md`, 2026-09-28): B5's worked example above
/// was wrong. "Subtract 3 from rolls involving hearing" (ArMDE:6614-6617) is a
/// modifier conditioned on a SENSE, like Corrupted's "selfish or sinful"
/// (D15) — a table judgement, not a fixed-Ability roll modifier: it hits
/// Awareness rolls made by sight or smell just as wrongly as it misses
/// non-Awareness hearing rolls (a Perception roll, following speech).
#[test]
fn shipped_poor_hearing_stays_textual_per_d61() {
    assert_shipped_uncomputed_roll_text(
        "flaw.poor_hearing",
        &[
            (
                "en",
                include_str!("../../../rules/i18n/en/virtues_flaws.json"),
                "rolls involving hearing",
            ),
            (
                "de",
                include_str!("../../../rules/i18n/de/virtues_flaws.json"),
                "die das Hören beinhalten",
            ),
        ],
    );
}

/// D61 (coordinator correction, 2026-09-28): `virtue.convoluted_mind`
/// (ArMDE:3601-3604) is NOT a genuine carrier despite naming Infernal Lore
/// outright — "+3 bonus on all Infernal Lore rolls **to determine what a
/// demon will do**" narrows the modifier to a subset of Infernal Lore rolls,
/// exactly Poor Hearing's error: Infernal Lore has uses beyond predicting a
/// demon's action, and none of those other uses get the +3. Stays
/// `uncomputed_rule` text, already shipped correctly — this pins it so a
/// future edit cannot silently turn it into a fixed `ability_roll_mod`.
#[test]
fn shipped_convoluted_mind_stays_textual_per_d61() {
    assert_shipped_uncomputed_roll_text(
        "virtue.convoluted_mind",
        &[
            (
                "en",
                include_str!("../../../rules/i18n/en/virtues_flaws.json"),
                "Infernal Lore rolls to determine what a demon will do",
            ),
            (
                "de",
                include_str!("../../../rules/i18n/de/virtues_flaws.json"),
                "Infernalkunde-Würfe, um zu bestimmen, was ein Dämon tun wird",
            ),
        ],
    );
}

// --- (f) F-489's genuine class-A carriers (coordinator correction, Phase 1b)

/// Shared assertion for a genuine class-A carrier of the fixed-target
/// `ability_roll_mod` shape: the passage names one Ability with no further
/// condition, unlike Poor Hearing's and Convoluted Mind's sense/situation-
/// conditioned family (D61). Checks three things: the entry is
/// `in_play_effect`; it surfaces the modifier in the structured `ability`
/// field; and it never perturbs the bought or effective score of the Ability
/// it modifies (surfaced-only, design § 1).
fn assert_shipped_roll_mod_carrier(id: &str, ability: &str, amount: i32) {
    let rs = load_shipped_ruleset();
    let item = rs
        .item(&Id::new(id))
        .unwrap_or_else(|| panic!("{id} must be in the shipped catalogue"));
    assert_eq!(
        item.classification,
        Classification::InPlayEffect,
        "{id}: a roll modifier naming one Ability with no further condition is computed, so \
         classification must be in_play_effect, not {:?}",
        item.classification
    );

    let mut entity = companion(vec![Selection::new(Id::new(id))]);
    let s = surfaced_modifiers(&entity, &rs);
    assert!(
        s.iter().any(|m| m.family == ModifierFamily::AbilityRoll
            && m.ability == Some(Id::new(ability))
            && m.amount == amount
            && m.source == Some(Id::new(id))),
        "{id} must surface {amount:+} to {ability} rolls in the structured `ability` field \
         — surfaced: {s:?}"
    );

    entity.ability_scores.push(AbilityScore {
        ability: Id::new(ability),
        score: 3,
        specialty: None,
        parameter: None,
    });
    let bought = entity.ability_scores[0].score;
    let effective = effective_ability_score(&entity, &rs, &Id::new(ability), None);
    assert_eq!(
        bought, 3,
        "{id} must never touch the bought score of {ability}"
    );
    assert_eq!(
        effective, 3,
        "{id} is a roll-only modifier and must never change the bought/effective score of \
         {ability} — effective: {effective}"
    );
}

/// `flaw.poor_concentration` (ArMDE:6602-6605): "The character has a -3
/// penalty to Concentration rolls." — unconditional, names Concentration
/// outright.
#[test]
fn shipped_poor_concentration_surfaces_minus_three_to_concentration() {
    assert_shipped_roll_mod_carrier("flaw.poor_concentration", "ability.concentration", -3);
}

/// `flaw.inconstant_magic` (ArMDE:6298-6301): "The character suffers a -3
/// penalty to all Finesse rolls." — unconditional; the rest of the passage
/// describes unrelated narrative effects on the character's spells, not a
/// condition on this modifier.
#[test]
fn shipped_inconstant_magic_surfaces_minus_three_to_finesse() {
    assert_shipped_roll_mod_carrier("flaw.inconstant_magic", "ability.finesse", -3);
}

/// `flaw.clumsy_magic` (ArMDE:5801-5804): "You receive a -3 penalty to any
/// rolls involving Finesse." — unconditional; the passage's other clause (an
/// aiming roll of 0 auto-botching) is a separate rule about a different
/// mechanic (the botch die), not a condition on this penalty.
#[test]
fn shipped_clumsy_magic_surfaces_minus_three_to_finesse() {
    assert_shipped_roll_mod_carrier("flaw.clumsy_magic", "ability.finesse", -3);
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
