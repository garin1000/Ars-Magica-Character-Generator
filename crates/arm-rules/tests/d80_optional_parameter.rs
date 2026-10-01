//! D80 (`docs/vf-audit/decisions.md`) point 3 — Fida'i/Lasiq's "cover social
//! status" is the ONE exception to D70's "every new parameter is required":
//! it applies only while the character is away from home on a mission
//! (ArMDE:4235), which the engine cannot know, so the choice is recorded when
//! made and never forced. No existing mechanism expresses "never required,
//! unconditionally" — `ParameterDef::required_if` only relaxes requiredness
//! CONDITIONALLY on another parameter in the SAME item, and D70 §3 already
//! ruled out repurposing it for severity. This adds the minimal field:
//! `ParameterDef::required: bool`, default `true` (every parameter shipped
//! before this field existed stays unconditionally required, byte-identical
//! JSON), `false` unconditionally exempts the key from `missing_param`
//! regardless of `required_if`.
//!
//! Hand-authored fixtures, on `b4_parameter_gated_effects.rs`'s own
//! precedent — the real `virtue.fidai`/`virtue.lasiq` data lands in
//! `rules/core/virtues_flaws.json` directly per X6c's data phase.

use arm_rules::ruleset::Ruleset;
use arm_rules::types::{Entity, EntityKind, Id, ParameterDef, RulesetRef, Selection};
use arm_rules::validation::validate;
use std::collections::BTreeMap;

/// `ruleset/integrity.rs::validate_engine_required_categories`: any non-empty
/// point-item catalogue must carry at least one `personality`-category entry
/// — unrelated to this fixture's own point, but required for it to load.
const PERSONALITY_FILLER: &str = r#"{ "id": "flaw.filler_personality", "kind": "flaw",
  "classification": "narrative", "magnitude": "minor", "categories": ["personality"],
  "entity_kinds": ["character"] }"#;

const COMPANION_TYPE: &str = r#"[
  { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
    "creation_phases": ["virtues_flaws"] }
]"#;

fn companion(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("test"), "1"),
    );
    e.selections = selections;
    e
}

fn issue_codes(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

/// A `text`-domain parameter declared `required: false`: an entity holding
/// the item with NO value for that key must never report `missing_param`.
#[test]
fn a_parameter_declared_required_false_never_reports_missing_param() {
    let items = format!(
        r#"[
      {{ "id": "virtue.tester_optional_cover", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"],
        "parameters": [
          {{ "key": "cover", "type": "ref", "domain": "text", "required": false }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let rs = Ruleset::from_json("test", "1", &items, COMPANION_TYPE)
        .expect("required: false must be accepted at load");

    let bare = companion(vec![Selection::new(Id::new(
        "virtue.tester_optional_cover",
    ))]);
    assert!(
        !issue_codes(&bare, &rs).iter().any(|c| c == "missing_param"),
        "D80.3: an unfilled `required: false` parameter must never report missing_param"
    );

    let filled = companion(vec![Selection::with_params(
        Id::new("virtue.tester_optional_cover"),
        BTreeMap::from([("cover".to_string(), Id::new("a travelling merchant"))]),
    )]);
    assert!(
        !issue_codes(&filled, &rs)
            .iter()
            .any(|c| c == "missing_param" || c == "unknown_param_value"),
        "a filled `required: false` parameter must still resolve cleanly once given"
    );
}

/// `required: false` together with `required_if` on the SAME parameter is an
/// authoring contradiction (one of the two would be silently ignored) and
/// must fail to load.
#[test]
fn ruleset_load_rejects_required_false_combined_with_required_if() {
    let items = format!(
        r#"[
      {{ "id": "virtue.tester_contradictory_required", "kind": "virtue",
        "classification": "creation_effect", "magnitude": "minor",
        "categories": ["social_status"], "entity_kinds": ["character"],
        "parameters": [
          {{ "key": "scope", "type": "ref", "domain": "enumerated",
            "values": ["scope.a", "scope.b"] }},
          {{ "key": "cover", "type": "ref", "domain": "text", "required": false,
            "required_if": {{ "param": "scope", "equals": "scope.a" }} }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let result = Ruleset::from_json("test", "1", &items, COMPANION_TYPE);
    assert!(
        result.is_err(),
        "required: false combined with required_if on the same parameter must fail to load"
    );
}

/// Serde round-trip: `required: true` (the default) must serialize exactly
/// as the pre-D80 shape did — no shipped `rules/core/*.json` entry gains a
/// byte of diff from this field's addition.
#[test]
fn required_true_serializes_byte_identically_to_today() {
    let param = ParameterDef::new(
        "ability",
        arm_rules::types::ParamType::Ref,
        arm_rules::types::ParameterDomain::Ability,
    );
    let json = serde_json::to_string(&param).unwrap();
    assert!(
        !json.contains("required"),
        "a default (required: true) ParameterDef must omit the field entirely, got {json}"
    );
}
