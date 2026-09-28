//! B3 (`docs/vf-audit/design-b0-ranging-and-predicates.md` § 1/§ 2/§ 8,
//! D23/D33) — RED-checkpoint phase 1 tests.
//!
//! `ItemPredicate`, `ParameterDef::exclude_if`, `PointItem::trained`, and
//! `PointItem::excluded_if_holds` are declared (this commit), but NOTHING
//! reads them yet: `validation::selections::param_value_resolves` does not
//! consult `exclude_if`, no validator scans `excluded_if_holds`, and
//! `ruleset::integrity` does not reject `exclude_if` on a non-`item` domain.
//! Every behavior asserted below therefore fails today for that reason, and
//! goes green only once phase 2 wires the real resolver/validator logic
//! behind these fields.
//!
//! Fixtures are hand-authored throughout, using INVENTED ids even where a
//! worked example in the design note names a real catalogue entry
//! (`flaw.university_dean`, `flaw.weak_personality`) — the real entries'
//! own JSON edits are Phase 3 data work, not B3's (design § 7).

use arm_rules::ruleset::Ruleset;
use arm_rules::types::{Entity, EntityKind, Id, RulesetRef, Selection};
use arm_rules::validation::{ValidationIssue, validate};
use std::collections::BTreeMap;

/// `ruleset/integrity.rs::validate_engine_required_categories`: ANY
/// non-empty point-item catalogue must carry at least one
/// `personality`-category entry. Unrelated to B3, but a fixture whose
/// `items` array is non-empty and carries no such entry fails to load for
/// that unrelated reason, which would mask the ONE gap each test below
/// actually means to exercise. A bare filler item, never selected.
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

fn sel(item_ref: &str) -> Selection {
    Selection::new(Id::new(item_ref))
}

fn has_code(issues: &[ValidationIssue], code: &str) -> bool {
    issues.iter().any(|i| i.code == code)
}

// --- (a) D33/Q-138: ParameterDef::exclude_if -------------------------------
//
// `flaw.flawed_powers`'s own shape (ArMDE:6146-6148, already verified in
// D33): a parameter naming an imported Major Hermetic Flaw, refusing any
// candidate for which `ItemPredicate::Trained` holds. `flaw.deficient_technique`
// (categories: hermetic, trained: true) is the excluded candidate;
// `flaw.some_other_hermetic_flaw` (trained: false, the default) is the
// control that must stay legal.

fn ruleset_with_flawed_powers() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "flaw.flawed_powers_like", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "major", "categories": ["supernatural"], "entity_kinds": ["character"],
        "parameters": [ {{ "key": "imported_flaw", "type": "ref", "domain": "item",
          "require_categories": ["hermetic"], "exclude_if": "trained" }} ] }},
      {{ "id": "flaw.trained_hermetic_flaw", "kind": "flaw", "classification": "in_play_effect",
        "magnitude": "major", "categories": ["hermetic"], "entity_kinds": ["character"],
        "trained": true }},
      {{ "id": "flaw.untrained_hermetic_flaw", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    Ruleset::from_json("test", "1", &items, COMPANION_TYPE)
        .expect("hand-authored fixture must load")
}

fn with_imported_flaw(imported: &str) -> Selection {
    Selection::with_params(
        Id::new("flaw.flawed_powers_like"),
        BTreeMap::from([("imported_flaw".into(), Id::new(imported))]),
    )
}

/// The § 8(a) red: naming a `trained` candidate under an `exclude_if:
/// "trained"` parameter must be refused as an unknown/out-of-domain value.
/// Fails today because `param_value_resolves`'s `Item` domain arm does not
/// consult `exclude_if` at all — the candidate still resolves via
/// `require_categories` alone.
#[test]
fn exclude_if_trained_refuses_a_trained_candidate() {
    let ruleset = ruleset_with_flawed_powers();
    let entity = companion(vec![with_imported_flaw("flaw.trained_hermetic_flaw")]);
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(&result.issues, ValidationIssue::CODE_UNKNOWN_PARAM_VALUE),
        "a `trained` candidate under `exclude_if: \"trained\"` must be refused \
         (B3 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// Control: an untrained candidate (the default, `trained` absent) stays a
/// legal value. Passes today (nothing excludes it yet) and must keep
/// passing once phase 2 wires the real check.
#[test]
fn exclude_if_trained_leaves_an_untrained_candidate_alone() {
    let ruleset = ruleset_with_flawed_powers();
    let entity = companion(vec![with_imported_flaw("flaw.untrained_hermetic_flaw")]);
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(&result.issues, ValidationIssue::CODE_UNKNOWN_PARAM_VALUE),
        "an untrained candidate must never be refused by `exclude_if: \
         \"trained\"` — issues: {:?}",
        result.issues
    );
}

/// § 5 integrity: `exclude_if` on a non-`item` domain must fail to load —
/// mirrors `allow_ids`/`require_categories`'s own domain check. Fails today
/// because `validate_parameter_defs` does not read `exclude_if` at all.
#[test]
fn ruleset_load_rejects_exclude_if_on_a_non_item_domain() {
    let items = format!(
        r#"[
      {{ "id": "flaw.bad_exclude_if_domain", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [ {{ "key": "target", "type": "ref", "domain": "ability",
          "exclude_if": "trained" }} ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let abilities = r#"{ "abilities": [ { "id": "ability.awareness", "category": "general" } ] }"#;
    let result = Ruleset::from_json_with_abilities("test", "1", &items, COMPANION_TYPE, abilities);
    assert!(
        result.is_err(),
        "`exclude_if` on a non-item domain must fail to load \
         (B3 phase 2 not yet wired)"
    );
}

// --- (b) D23/F-466: PointItem::excluded_if_holds ----------------------------
//
// `flaw.university_dean`'s own shape (Q-137, ArMDE:6923-6926, already
// verified in D23): illegal alongside ANY other effective Flaw carrying
// `Effect::GrantsReputation`, bought or GRANTED (closing the F-466
// reachability trap `validate_incompatibilities` is deliberately
// bought-only about).

fn ruleset_with_university_dean_like() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "flaw.reputation_excluder", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["story"], "entity_kinds": ["character"],
        "excluded_if_holds": ["grants_reputation"] }},
      {{ "id": "flaw.reputation_grantor", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "grants_reputation", "score": 1, "max_score": 3 }} ] }},
      {{ "id": "virtue.grantor_patron", "kind": "virtue", "classification": "narrative",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "grants_selection", "items": ["flaw.reputation_grantor"] }} ] }},
      {{ "id": "flaw.unrelated", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    Ruleset::from_json("test", "1", &items, COMPANION_TYPE)
        .expect("hand-authored fixture must load")
}

/// The § 8(b) red, bought-vs-bought: both the excluding item and the
/// Reputation-granting Flaw bought directly must raise the new exclusion
/// code. Fails today — nothing reads `excluded_if_holds` at all.
#[test]
fn excluded_if_holds_blocks_a_bought_reputation_grantor() {
    let ruleset = ruleset_with_university_dean_like();
    let entity = companion(vec![
        sel("flaw.reputation_excluder"),
        sel("flaw.reputation_grantor"),
    ]);
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(&result.issues, ValidationIssue::CODE_EXCLUDED_BY_PREDICATE),
        "a bought Reputation-granting Flaw must be refused alongside the \
         excluding item (B3 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// The § 8(b) red, grant-aware half (closes F-466): the Reputation grantor
/// arrives ONLY via `virtue.grantor_patron`'s `grants_selection`, never
/// bought directly. Design § 4 states this must be reached the same way as
/// the bought case. Fails today for the same reason.
#[test]
fn excluded_if_holds_blocks_a_granted_reputation_grantor_too() {
    let ruleset = ruleset_with_university_dean_like();
    let entity = companion(vec![
        sel("flaw.reputation_excluder"),
        sel("virtue.grantor_patron"),
    ]);
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(&result.issues, ValidationIssue::CODE_EXCLUDED_BY_PREDICATE),
        "a GRANTED Reputation-granting Flaw must be refused exactly like a \
         bought one (B3 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// Control: an unrelated Flaw (no `GrantsReputation`) never trips the
/// exclusion. Passes today and must keep passing once phase 2 wires the
/// real check.
#[test]
fn excluded_if_holds_leaves_an_unrelated_item_alone() {
    let ruleset = ruleset_with_university_dean_like();
    let entity = companion(vec![sel("flaw.reputation_excluder"), sel("flaw.unrelated")]);
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(&result.issues, ValidationIssue::CODE_EXCLUDED_BY_PREDICATE),
        "an unrelated Flaw must never trip a `grants_reputation` exclusion — \
         issues: {:?}",
        result.issues
    );
}

// --- (c) F-542 clause 2: both routes on one entry ---------------------------
//
// `flaw.weak_personality`'s own shape: BOTH `forbids_item_category` (B1,
// already real) and `excluded_if_holds: ["grants_personality_trait"]` (B3,
// new) sit on the SAME entry. A second `personality` Flaw fails via the
// category route; an unrelated Virtue granting a Personality Trait fails
// via the predicate route.

fn ruleset_with_weak_personality_like() -> Ruleset {
    let items = r#"[
      { "id": "flaw.weak_personality_like", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"],
        "effects": [ { "type": "forbids_item_category", "category": "personality" } ],
        "excluded_if_holds": ["grants_personality_trait"] },
      { "id": "flaw.bad_temper_like", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.trait_grantor", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ { "type": "grants_personality_trait" } ] },
      { "id": "flaw.unrelated", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"] }
    ]"#;
    Ruleset::from_json("test", "1", items, COMPANION_TYPE).expect("hand-authored fixture must load")
}

/// The category route: already real via B1's own
/// `validate_category_effect_prohibitions` (`forbids_item_category`) — NOT a
/// B3 red, included to confirm the two routes coexist cleanly on one entry.
#[test]
fn weak_personality_category_route_already_blocks_a_second_personality_flaw() {
    let ruleset = ruleset_with_weak_personality_like();
    let entity = companion(vec![
        sel("flaw.weak_personality_like"),
        sel("flaw.bad_temper_like"),
    ]);
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_CATEGORY_FORBIDDEN_BY_EFFECT
        ),
        "a second personality Flaw must be refused via B1's existing route — \
         issues: {:?}",
        result.issues
    );
}

/// The § 8(c) red, predicate route: an unrelated Virtue carrying
/// `Effect::GrantsPersonalityTrait` must be refused via
/// `excluded_if_holds: [\"grants_personality_trait\"]`, even though it
/// carries no `personality` category of its own. Fails today — nothing
/// reads `excluded_if_holds` at all.
#[test]
fn weak_personality_predicate_route_blocks_an_unrelated_trait_grantor() {
    let ruleset = ruleset_with_weak_personality_like();
    let entity = companion(vec![
        sel("flaw.weak_personality_like"),
        sel("virtue.trait_grantor"),
    ]);
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(&result.issues, ValidationIssue::CODE_EXCLUDED_BY_PREDICATE),
        "an unrelated Virtue granting a Personality Trait must be refused via \
         the predicate route (B3 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// Control: an unrelated Flaw with neither the category nor the predicate
/// trips neither code.
#[test]
fn weak_personality_leaves_an_unrelated_item_alone() {
    let ruleset = ruleset_with_weak_personality_like();
    let entity = companion(vec![
        sel("flaw.weak_personality_like"),
        sel("flaw.unrelated"),
    ]);
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(
            &result.issues,
            ValidationIssue::CODE_CATEGORY_FORBIDDEN_BY_EFFECT
        ) && !has_code(&result.issues, ValidationIssue::CODE_EXCLUDED_BY_PREDICATE),
        "an unrelated Flaw must trip neither Weak-Personality route — \
         issues: {:?}",
        result.issues
    );
}

// --- serde round-trip: ItemPredicate ---------------------------------------

/// `ItemPredicate` is a closed, snake_case-tagged enum — a bare parse/
/// round-trip check per D33's own "the enum is shared" design (§ 7).
/// Exercised indirectly through JSON already above, but pinned directly
/// here so a future rename of one tag is caught by name.
#[test]
fn item_predicate_json_tags_round_trip() {
    use arm_rules::types::ItemPredicate;
    for (json, expected) in [
        (r#""trained""#, ItemPredicate::Trained),
        (r#""grants_reputation""#, ItemPredicate::GrantsReputation),
        (
            r#""grants_personality_trait""#,
            ItemPredicate::GrantsPersonalityTrait,
        ),
    ] {
        let parsed: ItemPredicate = serde_json::from_str(json).unwrap_or_else(|e| {
            panic!("failed to parse {json}: {e}");
        });
        assert_eq!(parsed, expected, "unexpected parse for {json}");
        let rendered = serde_json::to_string(&parsed).unwrap();
        assert_eq!(rendered, json, "round-trip mismatch for {json}");
    }
}
