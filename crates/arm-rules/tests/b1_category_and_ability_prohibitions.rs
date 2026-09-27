//! B1 (`docs/vf-audit/design-b0-ranging-and-predicates.md`, D21/F-355/F-542/
//! F-511, D40 residual, D41's floor mechanism, F-502) — RED-checkpoint phase 1
//! tests.
//!
//! Every behavior asserted here is, as of this commit, a documented no-op stub
//! (`Prereq::HasCategory` in `validation/prereq.rs::evaluate_prereq`, the four
//! new `Effect` variants in `ruleset/integrity.rs::validate_effect_refs`, and
//! `CategoryCap.min`/`min_hard` unconsumed by `validation/caps.rs`). These
//! tests MUST fail today, for the reason each doc comment states, and go green
//! only once phase 2 wires the real evaluator/validator logic behind them.
//!
//! Fixtures are hand-authored throughout — the real catalogue entries this
//! note names (`flaw.ability_block`, `flaw.rector`, `flaw.weak_personality`,
//! `flaw.sheltered_upbringing`, `flaw.feral_upbringing`) are Phase 3's data
//! work (design note § 7), not B1's.

use arm_rules::ruleset::Ruleset;
use arm_rules::types::{AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection};
use arm_rules::validation::{IssueSeverity, ValidationIssue, validate};

/// `ruleset/integrity.rs::validate_engine_required_categories`: ANY non-empty
/// point-item catalogue must carry at least one `personality`-category entry
/// (the engine's Major-Personality-Flaw rule dereferences it unconditionally).
/// Unrelated to B1, but every fixture below whose `items` array is non-empty
/// must satisfy it, or that check's own failure masks the ONE gap each test
/// actually means to exercise. A bare filler item, never selected.
const PERSONALITY_FILLER: &str = r#"{ "id": "flaw.filler_personality", "kind": "flaw",
  "classification": "narrative", "magnitude": "minor", "categories": ["personality"],
  "entity_kinds": ["character"] }"#;

fn companion(selections: Vec<Selection>, ability_scores: Vec<AbilityScore>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("test"), "1"),
    );
    e.selections = selections;
    e.ability_scores = ability_scores;
    e
}

fn ability_score(ability: &str, score: u8) -> AbilityScore {
    AbilityScore {
        ability: Id::new(ability),
        score,
        specialty: None,
        parameter: None,
    }
}

fn sel(item_ref: &str) -> Selection {
    Selection::new(Id::new(item_ref))
}

fn has_code(issues: &[ValidationIssue], code: &str) -> bool {
    issues.iter().any(|i| i.code == code)
}

// --- (a) D21/F-355: Effect::ForbidsAbilityCategory ------------------------

fn ruleset_with_ability_block() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "flaw.ability_block", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "forbids_ability_category", "category": "martial" }} ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let abilities = r#"{ "abilities": [
      { "id": "ability.single_weapon", "category": "martial" },
      { "id": "ability.awareness", "category": "general" }
    ] }"#;
    Ruleset::from_json_with_abilities("test", "1", &items, types, abilities)
        .expect("hand-authored fixture must load")
}

/// The § 8(a) red: buying a Martial Ability alongside Ability Block's
/// `forbids_ability_category` effect must be refused. Fails today because
/// `validate_category_effect_prohibitions` (design § 4) does not exist yet —
/// nothing in `validate()` reads the new `Effect` variant at all.
#[test]
fn forbids_ability_category_blocks_a_bought_martial_ability() {
    let ruleset = ruleset_with_ability_block();
    let entity = companion(
        vec![sel("flaw.ability_block")],
        vec![ability_score("ability.single_weapon", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "a Martial Ability held alongside Ability Block's forbid must be refused \
         (B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// Control case: a General Ability must never trip a Martial-only forbid.
/// Passes today (nothing forbids anything yet) and must keep passing once
/// phase 2 wires the real check.
#[test]
fn forbids_ability_category_leaves_an_unrelated_category_alone() {
    let ruleset = ruleset_with_ability_block();
    let entity = companion(
        vec![sel("flaw.ability_block")],
        vec![ability_score("ability.awareness", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "a General Ability must never trip a Martial-only forbid — issues: {:?}",
        result.issues
    );
}

// --- (b) F-502/D21: Prereq::HasCategory -----------------------------------

fn ruleset_with_rector() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "flaw.rector", "kind": "flaw", "classification": "narrative",
        "magnitude": "major", "categories": ["story"], "entity_kinds": ["character"],
        "prerequisites": {{ "kind": "has_category", "value": "social_status" }} }},
      {{ "id": "virtue.some_status", "kind": "virtue", "classification": "narrative",
        "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    Ruleset::from_json("test", "1", &items, types).expect("hand-authored fixture must load")
}

/// The § 8(b) red: `flaw.rector` with no Social Status selection must fail
/// `prereq_not_met`. Fails today because `Prereq::HasCategory`'s evaluator
/// arm is a documented `Tri::Unknown` stub — it emits `prereq_unevaluated`
/// (a warning), never the hard `prereq_not_met` error this asserts.
#[test]
fn has_category_prereq_fails_without_a_matching_category() {
    let ruleset = ruleset_with_rector();
    let entity = companion(vec![sel("flaw.rector")], vec![]);
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(&result.issues, ValidationIssue::CODE_PREREQ_NOT_MET),
        "flaw.rector with no Social Status selection must fail prereq_not_met \
         (B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// The positive half: holding a Social Status item must satisfy the
/// prerequisite cleanly (`Tri::True`), not merely leave it unevaluated. Fails
/// today for the same stub reason as above — an unconditional `Tri::Unknown`
/// still raises `prereq_unevaluated` even when the entity actually holds a
/// qualifying category.
#[test]
fn has_category_prereq_is_satisfied_by_a_matching_category() {
    let ruleset = ruleset_with_rector();
    let entity = companion(vec![sel("flaw.rector"), sel("virtue.some_status")], vec![]);
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(&result.issues, ValidationIssue::CODE_PREREQ_NOT_MET),
        "flaw.rector WITH a Social Status selection must not fail prereq_not_met — \
         issues: {:?}",
        result.issues
    );
    assert!(
        !has_code(&result.issues, ValidationIssue::CODE_PREREQ_UNEVALUATED),
        "flaw.rector WITH a Social Status selection must evaluate satisfied, not \
         merely unevaluated (B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

// --- (c) F-427/D41: CategoryCap.min/min_hard ------------------------------

fn ruleset_with_social_status_floor() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "virtue.status_a", "kind": "virtue", "classification": "narrative",
        "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"] }},
      {{ "id": "virtue.status_b", "kind": "virtue", "classification": "narrative",
        "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10,
          "virtue_category_caps": [
            { "category": "social_status", "max": 1, "hard": false, "min": 1, "min_hard": true }
          ] },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    Ruleset::from_json("test", "1", &items, types).expect("hand-authored fixture must load")
}

/// The § 8(c) red, floor half: zero Social Status selections against a
/// `min: 1, min_hard: true` cap must raise a hard error. Fails today because
/// `validate_caps` (`validation/caps.rs`) never reads `CategoryCap.min` at
/// all — the fields exist on the struct but nothing consumes them yet.
#[test]
fn category_cap_floor_is_a_hard_error_when_unmet() {
    let ruleset = ruleset_with_social_status_floor();
    let entity = companion(vec![], vec![]);
    let result = validate(&entity, &ruleset);
    let floor_errors: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code.starts_with("too_few_") && i.severity == IssueSeverity::Error)
        .collect();
    assert!(
        !floor_errors.is_empty(),
        "zero Social Status selections against a hard min:1 floor must raise an \
         error (B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// The § 8(c) control, ceiling half: a SECOND Social Status against
/// `max: 1, hard: false` must raise only a warning, never an error — this
/// half already works today (the ceiling machinery is unchanged by B1), so
/// it documents the cap's other half rather than adding a new red.
#[test]
fn category_cap_ceiling_stays_a_warning_when_a_floor_is_also_present() {
    let ruleset = ruleset_with_social_status_floor();
    let entity = companion(vec![sel("virtue.status_a"), sel("virtue.status_b")], vec![]);
    let result = validate(&entity, &ruleset);
    let ceiling_issues: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code.starts_with("too_many_social_status"))
        .collect();
    assert!(
        ceiling_issues
            .iter()
            .all(|i| i.severity == IssueSeverity::Warning),
        "a second Social Status must warn, never error — issues: {:?}",
        result.issues
    );
}

// --- § 5 integrity checks: dangling refs must fail to load ----------------

/// `Prereq::HasCategory` naming a category no point item declares must fail
/// to load. Fails today (loads clean) because `validate_prereq_refs`'s new
/// arm is a documented no-op stub.
#[test]
fn ruleset_load_rejects_a_dangling_has_category_prereq() {
    let items = format!(
        r#"[
      {{ "id": "flaw.rector", "kind": "flaw", "classification": "narrative",
        "magnitude": "major", "categories": ["story"], "entity_kinds": ["character"],
        "prerequisites": {{ "kind": "has_category", "value": "no_such_category" }} }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let result = Ruleset::from_json("test", "1", &items, types);
    assert!(
        result.is_err(),
        "a has_category prereq naming a category NO point item declares must fail \
         to load (B1 phase 2 not yet wired)"
    );
}

/// `Effect::ForbidsItemCategory` naming a category no point item declares
/// must fail to load, the item-axis twin of the prereq check above. Fails
/// today because `validate_effect_refs`'s new arm is a documented no-op stub.
#[test]
fn ruleset_load_rejects_a_dangling_forbids_item_category() {
    let items = r#"[
      { "id": "flaw.weak_personality", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"],
        "effects": [ { "type": "forbids_item_category", "category": "no_such_category" } ] }
    ]"#;
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let result = Ruleset::from_json("test", "1", items, types);
    assert!(
        result.is_err(),
        "a forbids_item_category naming a category NO point item declares must \
         fail to load (B1 phase 2 not yet wired)"
    );
}

/// `Effect::ForbidsAbilities` naming an ability the catalogue does not
/// declare must fail to load. Fails today for the same stub reason.
#[test]
fn ruleset_load_rejects_a_dangling_forbids_abilities_target() {
    let items = r#"[
      { "id": "flaw.sheltered_upbringing", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"],
        "effects": [ { "type": "forbids_abilities", "abilities": ["ability.no_such_ability"] } ] }
    ]"#;
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let abilities = r#"{ "abilities": [ { "id": "ability.bargain", "category": "general" } ] }"#;
    let result = Ruleset::from_json_with_abilities("test", "1", items, types, abilities);
    assert!(
        result.is_err(),
        "a forbids_abilities entry naming an ability the catalogue lacks must \
         fail to load (B1 phase 2 not yet wired)"
    );
}

/// `Effect::RestrictsAbilityCategoryToAbilities` naming an ability the
/// catalogue does not declare must fail to load. Fails today for the same
/// stub reason.
#[test]
fn ruleset_load_rejects_a_dangling_restricts_ability_category_target() {
    let items = format!(
        r#"[
      {{ "id": "flaw.feral_upbringing", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "restricts_ability_category_to_abilities",
          "category": "general", "allowed": ["ability.no_such_ability"] }} ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let abilities = r#"{ "abilities": [ { "id": "ability.awareness", "category": "general" } ] }"#;
    let result = Ruleset::from_json_with_abilities("test", "1", &items, types, abilities);
    assert!(
        result.is_err(),
        "a restricts_ability_category_to_abilities entry naming an ability the \
         catalogue lacks must fail to load (B1 phase 2 not yet wired)"
    );
}

/// `Effect::RestrictsAbilityCategoryToAbilities.allowed` naming an ability
/// whose OWN category differs from the effect's stated `category` must fail
/// to load — a whitelist entry outside its own stated category can never
/// apply, so it is silent dead data otherwise (design § 5). Fails today for
/// the same stub reason.
#[test]
fn ruleset_load_rejects_an_allowed_ability_outside_its_stated_category() {
    let items = format!(
        r#"[
      {{ "id": "flaw.feral_upbringing", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "restricts_ability_category_to_abilities",
          "category": "general", "allowed": ["ability.parma_magica"] }} ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    // `ability.parma_magica` is Supernatural, not General — a real category
    // mismatch, not a dangling id.
    let abilities =
        r#"{ "abilities": [ { "id": "ability.parma_magica", "category": "supernatural" } ] }"#;
    let result = Ruleset::from_json_with_abilities("test", "1", &items, types, abilities);
    assert!(
        result.is_err(),
        "an allowed ability outside the effect's own stated category must fail \
         to load (B1 phase 2 not yet wired)"
    );
}

/// `CategoryCap.min > max` is an unsatisfiable range and must fail to load.
/// Fails today because no integrity check reads `CategoryCap.min` at all.
/// `items` is deliberately empty — `validate_engine_required_categories` is
/// gated on the catalogue being non-empty (see `PERSONALITY_FILLER`'s doc
/// comment), so an empty one needs no filler and stays the minimal fixture.
#[test]
fn ruleset_load_rejects_a_category_cap_floor_above_its_own_ceiling() {
    let items = "[]";
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10,
          "virtue_category_caps": [
            { "category": "social_status", "max": 1, "hard": false, "min": 2, "min_hard": true }
          ] },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let result = Ruleset::from_json("test", "1", items, types);
    assert!(
        result.is_err(),
        "a CategoryCap with min > max is unsatisfiable and must fail to load \
         (B1 phase 2 not yet wired)"
    );
}

/// `CategoryCap.min_hard: true` with `min` absent is meaningless (a hardness
/// flag with no floor to be hard about) and must fail to load. Fails today
/// for the same reason as the previous test. `items` is empty for the same
/// reason as above.
#[test]
fn ruleset_load_rejects_min_hard_with_no_min() {
    let items = "[]";
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10,
          "virtue_category_caps": [
            { "category": "social_status", "max": 1, "hard": false, "min_hard": true }
          ] },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let result = Ruleset::from_json("test", "1", items, types);
    assert!(
        result.is_err(),
        "min_hard with no min is meaningless and must fail to load \
         (B1 phase 2 not yet wired)"
    );
}

// --- D: F-542 clause 1 (Weak Personality) — Effect::ForbidsItemCategory ---
//
// Grant-awareness (design § 4, D2): the validator reads EFFECTIVE selections
// on both sides — whether the forbidding item is itself in effect (bought OR
// granted) and whether the forbidden target is present at all (bought OR
// granted). `forbids_item_category_blocks_a_granted_personality_item_too`
// below is the answer to "is a granted forbidden item refused, or advisory?":
// § 4 states no soft/advisory forbid-effect exists at all ("No B-group
// finding needs a *soft* forbid-effect, so none is designed") — every forbid
// is a hard error by construction, bought or granted alike.

fn ruleset_with_weak_personality() -> Ruleset {
    let items = r#"[
      { "id": "flaw.weak_personality", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"],
        "effects": [ { "type": "forbids_item_category", "category": "personality" } ] },
      { "id": "flaw.bad_temper", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.patron", "kind": "virtue", "classification": "narrative",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ { "type": "grants_selection", "items": ["flaw.bad_temper"] } ] },
      { "id": "flaw.poor_student", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"] }
    ]"#;
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    Ruleset::from_json("test", "1", items, types).expect("hand-authored fixture must load")
}

/// The plain bought-vs-bought case: a second `personality` Flaw bought
/// alongside Weak Personality must be refused. Fails today — nothing in
/// `validate()` reads `Effect::ForbidsItemCategory` at all.
#[test]
fn forbids_item_category_blocks_a_second_bought_personality_item() {
    let ruleset = ruleset_with_weak_personality();
    let entity = companion(
        vec![sel("flaw.weak_personality"), sel("flaw.bad_temper")],
        vec![],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_CATEGORY_FORBIDDEN_BY_EFFECT
        ),
        "a second bought personality Flaw must be refused (B1 phase 2 not yet \
         wired) — issues: {:?}",
        result.issues
    );
}

/// Grant-aware half: `flaw.bad_temper` arrives via `virtue.patron`'s
/// `grants_selection`, never bought directly — the forbidden target is
/// present bought-OR-granted (design § 4), so this must refuse exactly like
/// the bought case above, as a hard error, never merely advisory (§ 4: no
/// soft forbid-effect is designed at all). Fails today for the same reason.
#[test]
fn forbids_item_category_blocks_a_granted_personality_item_too() {
    let ruleset = ruleset_with_weak_personality();
    let entity = companion(
        vec![sel("flaw.weak_personality"), sel("virtue.patron")],
        vec![],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_CATEGORY_FORBIDDEN_BY_EFFECT
        ),
        "a GRANTED personality Flaw must be refused exactly like a bought one \
         (B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// Control: an unrelated category stays allowed. Passes today (nothing
/// forbids anything yet) and must keep passing once phase 2 wires the real
/// check.
#[test]
fn forbids_item_category_leaves_an_unrelated_category_alone() {
    let ruleset = ruleset_with_weak_personality();
    let entity = companion(
        vec![sel("flaw.weak_personality"), sel("flaw.poor_student")],
        vec![],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(
            &result.issues,
            ValidationIssue::CODE_CATEGORY_FORBIDDEN_BY_EFFECT
        ),
        "an unrelated (general) category must never trip a personality-only \
         forbid — issues: {:?}",
        result.issues
    );
}

// --- E: F-511 (Sheltered Upbringing) — Effect::ForbidsAbilities -----------
//
// "You may not take [these] as beginning Abilities, but you may learn them
// in play" (ArMDE:6721-6724). **The engine draws no in-play/beginning
// distinction at all**: `Entity::ability_scores` (`AbilityScore { ability,
// score, specialty, parameter }`) carries no life-stage-block or timing
// field, and no restricted-XP pool (including the LaterLife block,
// `effective/xp.rs::magus_later_life_pool`) is tracked per bought score
// either — `checked_xp_allocation`'s max-flow solve funds scores from pools
// without recording which pool paid for which score. There is nothing in
// this app's data model representing actual post-creation ("in play") XP at
// all — it builds the character AS OF the start of the saga, full stop. So
// `ForbidsAbilities` has no narrower subset to exempt: it applies to the
// WHOLE of `entity.ability_scores`, uniformly, forever — not a phase-1
// simplification, the honest shape of what this app can express.

fn ruleset_with_sheltered_upbringing() -> Ruleset {
    let items = r#"[
      { "id": "flaw.sheltered_upbringing", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"],
        "effects": [ { "type": "forbids_abilities", "abilities": ["ability.bargain", "ability.charm"] } ] },
      { "id": "virtue.mentor_grant", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ { "type": "ability_score_grant", "ability": "ability.bargain", "amount": 1 } ] }
    ]"#;
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let abilities = r#"{ "abilities": [
      { "id": "ability.bargain", "category": "general" },
      { "id": "ability.charm", "category": "general" },
      { "id": "ability.awareness", "category": "general" }
    ] }"#;
    Ruleset::from_json_with_abilities("test", "1", items, types, abilities)
        .expect("hand-authored fixture must load")
}

/// Buying a listed Ability at creation must be refused. Fails today —
/// nothing in `validate()` reads `Effect::ForbidsAbilities` at all.
#[test]
fn forbids_abilities_blocks_a_bought_listed_ability() {
    let ruleset = ruleset_with_sheltered_upbringing();
    let entity = companion(
        vec![sel("flaw.sheltered_upbringing")],
        vec![ability_score("ability.bargain", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "a bought listed Ability must be refused (B1 phase 2 not yet wired) — \
         issues: {:?}",
        result.issues
    );
}

/// Control: an unlisted Ability is fine. Passes today and must keep passing.
#[test]
fn forbids_abilities_leaves_an_unlisted_ability_alone() {
    let ruleset = ruleset_with_sheltered_upbringing();
    let entity = companion(
        vec![sel("flaw.sheltered_upbringing")],
        vec![ability_score("ability.awareness", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "an unlisted Ability must never trip the forbid — issues: {:?}",
        result.issues
    );
}

/// Grant-aware half, ability axis: `ability.bargain` arrives ONLY as a free
/// floor from `virtue.mentor_grant`'s `Effect::AbilityScoreGrant` — it never
/// appears in `entity.ability_scores` at all. Design § 4's "an Ability score
/// bought, or another V/F selection... (bought OR granted)" reaches this
/// case too, so it must refuse exactly like the bought case, as a hard
/// error (§ 4: no soft forbid-effect exists). Fails today for the same
/// reason.
#[test]
fn forbids_abilities_blocks_a_granted_ability_score_floor_too() {
    let ruleset = ruleset_with_sheltered_upbringing();
    let entity = companion(
        vec![sel("flaw.sheltered_upbringing"), sel("virtue.mentor_grant")],
        vec![],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "a GRANTED Ability floor must be refused exactly like a bought score \
         (B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

// --- F: D40 residual (Feral Upbringing) — RestrictsAbilityCategoryToAbilities
//
// D60.2: the whitelist applies to BEGINNING Abilities only, and two stacked
// whitelists intersect. The "creation-only, in-play untouched" half carries
// the SAME caveat as § E above — restated, not re-argued: this engine has no
// data representation of post-creation ("in play") Ability acquisition at
// all, so there is no narrower creation-only subset to express; the
// restriction necessarily applies to the whole of `entity.ability_scores`.

fn ruleset_with_feral_upbringing() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "flaw.feral_upbringing", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "restricts_ability_category_to_abilities",
          "category": "general", "allowed": ["ability.awareness", "ability.athletics"] }} ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let abilities = r#"{ "abilities": [
      { "id": "ability.awareness", "category": "general" },
      { "id": "ability.athletics", "category": "general" },
      { "id": "ability.bargain", "category": "general" }
    ] }"#;
    Ruleset::from_json_with_abilities("test", "1", &items, types, abilities)
        .expect("hand-authored fixture must load")
}

/// A beginning Ability in the restricted category NOT on the whitelist must
/// be refused. Fails today — nothing in `validate()` reads
/// `Effect::RestrictsAbilityCategoryToAbilities` at all.
#[test]
fn restricts_ability_category_blocks_an_unlisted_ability_in_the_restricted_category() {
    let ruleset = ruleset_with_feral_upbringing();
    let entity = companion(
        vec![sel("flaw.feral_upbringing")],
        vec![ability_score("ability.bargain", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "an unlisted general Ability must be refused under the whitelist \
         (B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}

/// Control: a listed Ability is fine. Passes today and must keep passing.
#[test]
fn restricts_ability_category_allows_a_listed_ability() {
    let ruleset = ruleset_with_feral_upbringing();
    let entity = companion(
        vec![sel("flaw.feral_upbringing")],
        vec![ability_score("ability.awareness", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "a whitelisted Ability must never trip the restriction — issues: {:?}",
        result.issues
    );
}

fn ruleset_with_two_stacked_restrictions() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "flaw.feral_upbringing", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "restricts_ability_category_to_abilities",
          "category": "general", "allowed": ["ability.awareness", "ability.athletics"] }} ] }},
      {{ "id": "flaw.second_restriction", "kind": "flaw", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "effects": [ {{ "type": "restricts_ability_category_to_abilities",
          "category": "general", "allowed": ["ability.athletics", "ability.bargain"] }} ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let types = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "creation_phases": ["virtues_flaws"] }
    ]"#;
    let abilities = r#"{ "abilities": [
      { "id": "ability.awareness", "category": "general" },
      { "id": "ability.athletics", "category": "general" },
      { "id": "ability.bargain", "category": "general" }
    ] }"#;
    Ruleset::from_json_with_abilities("test", "1", &items, types, abilities)
        .expect("hand-authored fixture must load")
}

/// D60.2's stacking rule, kept half: `ability.athletics` is on BOTH
/// whitelists (`{awareness, athletics}` ∩ `{athletics, bargain}` =
/// `{athletics}`), so it must stay legal under two stacked restrictions.
/// Fails today for the same reason as every other test in this file — no
/// consumer exists yet, so this passes VACUOUSLY today (no error either way)
/// and must keep passing, meaningfully, once phase 2 wires intersection.
#[test]
fn restricts_ability_category_stacked_restrictions_keep_the_common_ability() {
    let ruleset = ruleset_with_two_stacked_restrictions();
    let entity = companion(
        vec![sel("flaw.feral_upbringing"), sel("flaw.second_restriction")],
        vec![ability_score("ability.athletics", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        !has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "an Ability on BOTH stacked whitelists must stay legal — issues: {:?}",
        result.issues
    );
}

/// D60.2's stacking rule, excluded half: `ability.awareness` is on the FIRST
/// whitelist only — the intersection excludes it, so two stacked
/// restrictions must be STRICTER than either alone. Fails today because
/// nothing reads the effect at all (no error is raised for anything yet).
#[test]
fn restricts_ability_category_stacked_restrictions_exclude_an_ability_on_only_one_list() {
    let ruleset = ruleset_with_two_stacked_restrictions();
    let entity = companion(
        vec![sel("flaw.feral_upbringing"), sel("flaw.second_restriction")],
        vec![ability_score("ability.awareness", 3)],
    );
    let result = validate(&entity, &ruleset);
    assert!(
        has_code(
            &result.issues,
            ValidationIssue::CODE_ABILITY_FORBIDDEN_BY_EFFECT
        ),
        "an Ability on only ONE of two stacked whitelists must be refused \
         (intersection, not union; B1 phase 2 not yet wired) — issues: {:?}",
        result.issues
    );
}
