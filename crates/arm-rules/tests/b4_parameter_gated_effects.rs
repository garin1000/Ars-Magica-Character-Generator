//! B4 (`docs/vf-audit/design-b0-ranging-and-predicates.md` § 1/§ 2/§ 3c/§ 5/
//! § 8, Q-51).
//!
//! Revision 3 dropped the `Effect::Gated` wrapper: `gate: Option<ParamGate>`
//! is declared directly on `Effect::CharacteristicScoreDeltaParam` and
//! `Effect::GrantsReputation`, and `ParamGate::holds` is `pub(crate)`.
//! `characteristic_score_bonus` (`effective/characteristic.rs`) and
//! `reputation_grants` (`effective/reputation_and_caps.rs`) each guard their
//! match arm with `gate.as_ref().is_none_or(|g| g.holds(selection))`;
//! `ruleset::integrity::validate_effect_refs` checks `gate.param` on both
//! variants via the SAME `validate_param_gate` C1 built for `AbilityRef`/
//! `CategoryRef`'s own gate.
//!
//! Fixtures (a)-(b) use INVENTED ids, on B1/B3's own precedent — the real
//! `virtue.magical_blood` entry in `rules/core/virtues_flaws.json` carries the
//! `bloodline`/`characteristic` parameters and the two gated effects
//! (Magic Human's clause, ArMDE:4367), landed by B4 directly per design § 7.
//! Fixture (c) loads the REAL shipped ruleset to prove that data is in place.
//!
//! Two follow-up fixes, both test-first (coordinator review, post-B4):
//! **Gap 1** — `characteristic` must not be required for Magic Animal/Spirit/
//! Thing, which the clause never reads; fixed via
//! `ParameterDef::required_if: Option<ParamGate>`, consumed by
//! `validate_selection_parameters`'s separate "required" set. **Gap 2** —
//! "but not above +3" (ArMDE:4367): a corrected SECOND pass — the first cut
//! wrongly inferred "no precondition, clamp instead" from `gate.is_some()`,
//! which would have silently misclassified a future GATED effect with
//! Great-Characteristic semantics (raising ABOVE the cap). Fixed via explicit
//! data instead: `CharacteristicDeltaCap` (`AboveBase` default, `WithinBase`
//! for Magical Blood) on the `cap` field, read by
//! `validate_characteristic_delta_preconditions` (skips its precondition only
//! on `WithinBase`) and `characteristic_score_bonus` (clamps only on
//! `WithinBase`, via `capped_characteristic_delta_contribution`) — never on
//! whether a `gate` happens to be present.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Entity, EntityKind, Id, RulesetRef, Selection};
use arm_rules::validation::{ValidationIssue, validate};
use arm_rules::{
    Characteristic, CharacteristicDeltaCap, Effect, characteristic_score_bonus,
    effective_characteristic_score, reputation_grants,
};
use std::collections::BTreeMap;

/// `ruleset/integrity.rs::validate_engine_required_categories`: ANY non-empty
/// point-item catalogue must carry at least one `personality`-category entry.
/// Unrelated to B4, but every hand-authored fixture below must satisfy it, or
/// that check's own failure masks the ONE gap each test actually means to
/// exercise.
const PERSONALITY_FILLER: &str = r#"{ "id": "flaw.filler_personality", "kind": "flaw",
  "classification": "narrative", "magnitude": "minor", "categories": ["personality"],
  "entity_kinds": ["character"] }"#;

const COMPANION_TYPE: &str = r#"[
  { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
    "creation_phases": ["virtues_flaws"] }
]"#;

/// The shipped ruleset's own base cap/floor (`rules/core/characteristics.json`,
/// trimmed to the two fields `validate_characteristic_delta_preconditions`
/// reads) — `Ruleset::from_json` omits `characteristics` entirely, which
/// would make that precondition's `let Some(rules) = ... else { return }`
/// early-exit mask the very behavior the gap-2 decoupling tests below mean to
/// exercise.
const CHARACTERISTICS_JSON: &str = r#"{ "start_points": 7, "base_max": 3, "base_min": -3,
  "costs": [{ "score": 3, "cost": 6 }, { "score": 0, "cost": 0 }, { "score": -3, "cost": -6 }] }"#;

fn companion(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("test"), "1"),
    );
    e.selections = selections;
    e
}

// --- (a)/(b) hand-authored fixture: both gated variants on one entry --------
//
// Mirrors design § 2's own worked example for `virtue.magical_blood`'s Magic
// Human clause: a `bloodline` parameter gates BOTH a
// `characteristic_score_delta_param` (whose own `param` names a SECOND,
// player-chosen parameter, "characteristic") and a `grants_reputation`.

fn ruleset_with_gated_blood_like() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "virtue.tester_gated_blood", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["supernatural"], "entity_kinds": ["character"],
        "parameters": [
          {{ "key": "bloodline", "type": "ref", "domain": "enumerated",
            "values": ["magic_human", "magic_animal"] }},
          {{ "key": "characteristic", "type": "ref", "domain": "characteristic" }}
        ],
        "effects": [
          {{ "type": "characteristic_score_delta_param", "param": "characteristic", "amount": 1,
            "gate": {{ "param": "bloodline", "equals": "magic_human" }} }},
          {{ "type": "grants_reputation", "score": 3,
            "gate": {{ "param": "bloodline", "equals": "magic_human" }} }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    Ruleset::from_json("test", "1", &items, COMPANION_TYPE)
        .expect("hand-authored fixture must load")
}

fn with_bloodline(bloodline: &str) -> Selection {
    Selection::with_params(
        Id::new("virtue.tester_gated_blood"),
        BTreeMap::from([
            ("bloodline".into(), Id::new(bloodline)),
            ("characteristic".into(), Id::new("characteristic.str")),
        ]),
    )
}

/// Characteristic half, gate HELD: the bonus applies.
#[test]
fn characteristic_score_delta_param_applies_when_gate_holds() {
    let rs = ruleset_with_gated_blood_like();
    let entity = companion(vec![with_bloodline("magic_human")]);
    let bonus = characteristic_score_bonus(&entity, &rs, Characteristic::Str);
    assert_eq!(
        bonus, 1,
        "bloodline=magic_human must grant the +1 Characteristic bonus its gate names"
    );
}

/// Characteristic half, gate NOT held: the bonus must not apply.
#[test]
fn characteristic_score_delta_param_does_not_apply_when_gate_does_not_hold() {
    let rs = ruleset_with_gated_blood_like();
    let entity = companion(vec![with_bloodline("magic_animal")]);
    let bonus = characteristic_score_bonus(&entity, &rs, Characteristic::Str);
    assert_eq!(
        bonus, 0,
        "bloodline=magic_animal must NOT grant Magic Human's +1 Characteristic bonus — bonus: {bonus}"
    );
}

/// Reputation half, gate HELD: the grant applies.
#[test]
fn grants_reputation_applies_when_gate_holds() {
    let rs = ruleset_with_gated_blood_like();
    let entity = companion(vec![with_bloodline("magic_human")]);
    let grants = reputation_grants(&entity, &rs);
    assert!(
        grants.iter().any(|g| g.score == 3),
        "bloodline=magic_human must grant the level-3 Reputation its gate names — grants: {grants:?}"
    );
}

/// Reputation half, gate NOT held: the grant must not apply.
#[test]
fn grants_reputation_does_not_apply_when_gate_does_not_hold() {
    let rs = ruleset_with_gated_blood_like();
    let entity = companion(vec![with_bloodline("magic_animal")]);
    let grants = reputation_grants(&entity, &rs);
    assert!(
        grants.is_empty(),
        "bloodline=magic_animal must NOT grant Magic Human's level-3 Reputation — grants: {grants:?}"
    );
}

// --- (c) the shipped `virtue.magical_blood` ---------------------------------
//
// ArMDE:4359-4372 (Magical Blood, verified directly): only the Magic Human
// sub-type states a determinate mechanic — "may increase one of his
// Characteristics by 1, but not above +3 ... also has a positive Reputation
// at level 3 among others of his bloodline" (ArMDE:4367). The shipped entry
// now carries `bloodline`/`characteristic` parameters and the two effects
// gated on `bloodline=bloodline.magic_human`; Magic Lore's authorization and
// the Animal/Spirit/Thing sub-type bonuses stay uncomputed
// (`crates/arm-rules/tests/uncomputed_clauses.rs`'s own `PENDING_DROPPED_CLAUSE`
// row for this id) — their prose lives in `description` in both locales.

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

/// `bloodline=magic_human` must gain both the Characteristic bonus and the
/// Reputation.
#[test]
fn shipped_magical_blood_magic_human_gains_characteristic_bonus_and_reputation() {
    let rs = load_shipped_ruleset();
    let entity = companion(vec![Selection::with_params(
        Id::new("virtue.magical_blood"),
        BTreeMap::from([
            ("bloodline".into(), Id::new("bloodline.magic_human")),
            ("characteristic".into(), Id::new("characteristic.str")),
        ]),
    )]);
    let bonus = characteristic_score_bonus(&entity, &rs, Characteristic::Str);
    let grants = reputation_grants(&entity, &rs);
    assert_eq!(
        bonus, 1,
        "Magic Human's +1 Characteristic (ArMDE:4367) must be granted on the shipped \
         entry — bonus: {bonus}"
    );
    assert!(
        grants.iter().any(|g| g.score == 3),
        "Magic Human's level-3 Reputation (ArMDE:4367) must be granted on the shipped \
         entry — grants: {grants:?}"
    );
}

/// Control: another bloodline must never receive Magic Human's bonus/grant.
#[test]
fn shipped_magical_blood_other_bloodline_gains_neither() {
    let rs = load_shipped_ruleset();
    let entity = companion(vec![Selection::with_params(
        Id::new("virtue.magical_blood"),
        BTreeMap::from([("bloodline".into(), Id::new("bloodline.magic_animal"))]),
    )]);
    let bonus = characteristic_score_bonus(&entity, &rs, Characteristic::Str);
    let grants = reputation_grants(&entity, &rs);
    assert_eq!(
        bonus, 0,
        "a non-Magic-Human bloodline must never receive the Characteristic bonus"
    );
    assert!(
        grants.is_empty(),
        "a non-Magic-Human bloodline must never receive the Reputation grant — grants: {grants:?}"
    );
}

// --- load-time integrity: reusing C1's ParamGate check ----------------------
//
// Design § 5: `gate.param` on either variant is validated by the SAME
// `validate_param_gate` C1 already built for `AbilityRef`/`CategoryRef`'s own
// gate — not a new invention.

/// A `characteristic_score_delta_param` gate naming a parameter the SAME item
/// never declares must fail to load.
#[test]
fn ruleset_load_rejects_a_dangling_gate_param_on_characteristic_score_delta_param() {
    let items = format!(
        r#"[
      {{ "id": "virtue.tester_gated_blood", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["supernatural"], "entity_kinds": ["character"],
        "parameters": [
          {{ "key": "characteristic", "type": "ref", "domain": "characteristic" }}
        ],
        "effects": [
          {{ "type": "characteristic_score_delta_param", "param": "characteristic", "amount": 1,
            "gate": {{ "param": "no_such_param", "equals": "magic_human" }} }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let result = Ruleset::from_json("test", "1", &items, COMPANION_TYPE);
    assert!(
        result.is_err(),
        "a characteristic_score_delta_param gate naming an undeclared parameter must fail to load"
    );
}

/// A `grants_reputation` gate naming a `multi_ref` parameter must fail to
/// load — C1's own rule (a gate reads exactly ONE value at fold time) applies
/// identically here.
#[test]
fn ruleset_load_rejects_a_multi_ref_gate_param_on_grants_reputation() {
    let items = format!(
        r#"[
      {{ "id": "virtue.tester_gated_blood2", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["supernatural"], "entity_kinds": ["character"],
        "parameters": [
          {{ "key": "targets", "type": "multi_ref", "domain": "ability" }}
        ],
        "effects": [
          {{ "type": "grants_reputation", "score": 3,
            "gate": {{ "param": "targets", "equals": "ability.awareness" }} }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    let abilities = r#"{ "abilities": [ { "id": "ability.awareness", "category": "general" } ] }"#;
    let result = Ruleset::from_json_with_abilities("test", "1", &items, COMPANION_TYPE, abilities);
    assert!(
        result.is_err(),
        "a grants_reputation gate naming a multi_ref parameter must fail to load"
    );
}

// --- serde round-trip: `gate: None` is byte-identical to the pre-B4 shape ---

/// An absent gate must serialize exactly as the pre-B4 shape did — the
/// `#[serde(default, skip_serializing_if = "Option::is_none")]` stub must not
/// perturb any existing shipped JSON.
#[test]
fn characteristic_score_delta_param_gate_none_serializes_byte_identically_to_today() {
    let effect = Effect::CharacteristicScoreDeltaParam {
        param: "characteristic".into(),
        amount: 1,
        gate: None,
        cap: CharacteristicDeltaCap::AboveBase,
    };
    let rendered = serde_json::to_string(&effect).expect("Effect always serializes");
    assert_eq!(
        rendered,
        r#"{"type":"characteristic_score_delta_param","param":"characteristic","amount":1}"#,
        "an absent gate and a default (AboveBase) cap must serialize byte-identically to the \
         pre-B4 shape"
    );
}

/// Same guarantee for `GrantsReputation`.
#[test]
fn grants_reputation_gate_none_serializes_byte_identically_to_today() {
    let effect = Effect::GrantsReputation {
        kind: None,
        score: 3,
        max_score: None,
        gate: None,
    };
    let rendered = serde_json::to_string(&effect).expect("Effect always serializes");
    assert_eq!(
        rendered, r#"{"type":"grants_reputation","score":3}"#,
        "an absent gate must serialize byte-identically to the pre-B4 shape"
    );
}

// --- older-save standing policy: `missing_param` ----------------------------
//
// `virtue.magical_blood` now declares two parameters no earlier save could
// ever have stored. `missing_param` is the standing, generic policy for a
// declared-but-unfilled parameter (`validation/selections.rs::validate_selection_parameters`)
// — no per-item wiring needed — but this pins the specific new instance
// against a regression, on the same precedent as the Corrupted-entries and
// Form-scoped-Flaw rows already in `docs/open-todos.md`'s "What an older
// save still reports on open" list.

/// A `virtue.magical_blood` selection written before B4 (no `bloodline`/
/// `characteristic` params at all, since neither was ever stored) must report
/// `missing_param` for `bloodline` — but NOT (yet) for `characteristic`, which
/// only becomes relevant once `bloodline=magic_human` is chosen (see
/// `required_if` below). Two unfilled params reported at once would ask the
/// player to fill a field that means nothing until the FIRST choice narrows
/// which bloodline they have.
#[test]
fn an_older_magical_blood_selection_with_no_params_reports_missing_param() {
    let rs = load_shipped_ruleset();
    let entity = companion(vec![Selection::new(Id::new("virtue.magical_blood"))]);
    let issues = validate(&entity, &rs).issues;
    let missing_param_keys: Vec<&str> = issues
        .iter()
        .filter(|i| i.code == ValidationIssue::CODE_MISSING_PARAM)
        .filter_map(|i| i.args.get("key").map(String::as_str))
        .collect();
    assert_eq!(
        missing_param_keys,
        vec!["bloodline"],
        "an older virtue.magical_blood selection with neither parameter stored must \
         report missing_param for bloodline only — issues: {issues:?}"
    );
}

/// The gap-1 fix: a Magic Animal/Spirit/Thing character must never be asked
/// to fill a Characteristic parameter that clause never uses. Fails today —
/// `characteristic` is declared unconditionally, so `validate_selection_parameters`
/// requires it regardless of `bloodline`'s value.
#[test]
fn magic_animal_bloodline_does_not_require_the_characteristic_param() {
    let rs = load_shipped_ruleset();
    let entity = companion(vec![Selection::with_params(
        Id::new("virtue.magical_blood"),
        BTreeMap::from([("bloodline".into(), Id::new("bloodline.magic_animal"))]),
    )]);
    let issues = validate(&entity, &rs).issues;
    let missing_characteristic = issues.iter().any(|i| {
        i.code == ValidationIssue::CODE_MISSING_PARAM
            && i.args.get("key").map(String::as_str) == Some("characteristic")
    });
    assert!(
        !missing_characteristic,
        "bloodline=magic_animal must never require the characteristic param — issues: {issues:?}"
    );
}

/// The other half: once `bloodline=magic_human` IS chosen, `characteristic`
/// becomes genuinely required (the clause needs to know which Characteristic
/// to raise). Control — must stay green once `required_if` lands, since this
/// is exactly what `required_if` MUST still enforce, not merely relax.
#[test]
fn magic_human_bloodline_still_requires_the_characteristic_param() {
    let rs = load_shipped_ruleset();
    let entity = companion(vec![Selection::with_params(
        Id::new("virtue.magical_blood"),
        BTreeMap::from([("bloodline".into(), Id::new("bloodline.magic_human"))]),
    )]);
    let issues = validate(&entity, &rs).issues;
    let missing_characteristic = issues.iter().any(|i| {
        i.code == ValidationIssue::CODE_MISSING_PARAM
            && i.args.get("key").map(String::as_str) == Some("characteristic")
    });
    assert!(
        missing_characteristic,
        "bloodline=magic_human must still require the characteristic param — issues: {issues:?}"
    );
}

// --- gap 2 (ArMDE:4367): "but not above +3" ---------------------------------

/// The gap-2 fix: Magic Human's +1 must not push an already-capped
/// Characteristic past the printed base cap (+3). Fails today —
/// `characteristic_score_bonus` adds the gated delta unconditionally, with no
/// ceiling of its own (unlike Great Characteristic, which explicitly has none
/// because ArMDE:3977/Giant Blood deliberately overshoot it).
#[test]
fn magic_human_characteristic_bonus_does_not_exceed_the_base_cap() {
    let rs = load_shipped_ruleset();
    let mut entity = companion(vec![Selection::with_params(
        Id::new("virtue.magical_blood"),
        BTreeMap::from([
            ("bloodline".into(), Id::new("bloodline.magic_human")),
            ("characteristic".into(), Id::new("characteristic.str")),
        ]),
    )]);
    entity.characteristics.insert(Characteristic::Str, 3);
    let effective = effective_characteristic_score(&entity, &rs, Characteristic::Str);
    assert!(
        effective <= 3,
        "Magic Human's +1 must not push an already-capped Characteristic past +3 \
         (ArMDE:4367) — effective: {effective}"
    );
}

/// The other direction: ArMDE:4367 states no precondition requiring the
/// Characteristic to ALREADY be at the cap (unlike Great Characteristic's own
/// "already has a score of at least +3", ArMDE:3987) — a Magic Human
/// character with Strength at +1 must be able to take the Virtue and reach
/// +2 cleanly, not be told the Characteristic is already too low. Fails today
/// if `validate_characteristic_delta_preconditions` (Great Characteristic's
/// own precondition) misfires on this UNRELATED gated effect too, since
/// `effect_target` does not distinguish a gated delta from an ungated one.
#[test]
fn magic_human_below_the_base_cap_does_not_require_a_pre_existing_plus_three() {
    let rs = load_shipped_ruleset();
    let mut entity = companion(vec![Selection::with_params(
        Id::new("virtue.magical_blood"),
        BTreeMap::from([
            ("bloodline".into(), Id::new("bloodline.magic_human")),
            ("characteristic".into(), Id::new("characteristic.str")),
        ]),
    )]);
    entity.characteristics.insert(Characteristic::Str, 1);
    let issues = validate(&entity, &rs).issues;
    assert!(
        !issues
            .iter()
            .any(|i| i.code == ValidationIssue::CODE_CHARACTERISTIC_MAX_BASE_TOO_LOW),
        "Magic Human's +1 must not require the Characteristic to already be at the \
         base cap — ArMDE:4367 states no such precondition — issues: {issues:?}"
    );
}

// --- coordinator review, post-B4: `cap` is DATA, never inferred from `gate` ---
//
// The rule ("no precondition, clamp instead") must live on
// `CharacteristicDeltaCap`, not on the mere presence of a `gate` — a future
// gated effect with Great-Characteristic semantics (raising ABOVE the cap)
// must not silently inherit Magical Blood's clamp merely for being gated, and
// an UNGATED `within_base` delta must still clamp. Both hand-authored fixtures
// below use INVENTED ids, decoupled from the real `virtue.magical_blood` shape
// entirely, to prove the two axes (`gate`, `cap`) are independent.

fn ruleset_with_gated_above_base_delta() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "virtue.tester_gated_above_base", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [
          {{ "key": "axis", "type": "ref", "domain": "enumerated", "values": ["on", "off"] }},
          {{ "key": "characteristic", "type": "ref", "domain": "characteristic" }}
        ],
        "effects": [
          {{ "type": "characteristic_score_delta_param", "param": "characteristic", "amount": 1,
            "gate": {{ "param": "axis", "equals": "on" }} }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: &items,
        type_profiles: COMPANION_TYPE,
        characteristics: Some(CHARACTERISTICS_JSON),
        ..RulesetSources::default()
    })
    .expect("hand-authored fixture must load")
}

/// A GATED delta with no `cap` (defaults to `AboveBase`) must keep the OLD
/// Great-Characteristic-shaped precondition — the gate governs only WHETHER
/// the effect applies, never HOW. Fails if the engine still infers "no
/// precondition" from `gate.is_some()` instead of reading `cap`.
#[test]
fn gated_delta_without_within_base_keeps_the_old_precondition() {
    let rs = ruleset_with_gated_above_base_delta();
    let entity = companion(vec![Selection::with_params(
        Id::new("virtue.tester_gated_above_base"),
        BTreeMap::from([
            ("axis".into(), Id::new("on")),
            ("characteristic".into(), Id::new("characteristic.str")),
        ]),
    )]);
    let issues = validate(&entity, &rs).issues;
    assert!(
        issues
            .iter()
            .any(|i| i.code == ValidationIssue::CODE_CHARACTERISTIC_MAX_BASE_TOO_LOW),
        "a gated AboveBase delta (the default) must still require the Characteristic \
         to already be at the base cap, exactly like an ungated one — issues: {issues:?}"
    );
}

fn ruleset_with_ungated_within_base_delta() -> Ruleset {
    let items = format!(
        r#"[
      {{ "id": "virtue.tester_ungated_within_base", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [
          {{ "key": "characteristic", "type": "ref", "domain": "characteristic" }}
        ],
        "effects": [
          {{ "type": "characteristic_score_delta_param", "param": "characteristic", "amount": 1,
            "cap": "within_base" }}
        ] }},
      {PERSONALITY_FILLER}
    ]"#
    );
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: &items,
        type_profiles: COMPANION_TYPE,
        characteristics: Some(CHARACTERISTICS_JSON),
        ..RulesetSources::default()
    })
    .expect("hand-authored fixture must load")
}

/// An UNGATED `within_base` delta must still clamp — `cap` alone drives the
/// clamp, with no gate required at all. Fails if the clamp is only ever
/// reached through the gated branch.
#[test]
fn ungated_within_base_delta_clamps() {
    let rs = ruleset_with_ungated_within_base_delta();
    let mut entity = companion(vec![Selection::with_params(
        Id::new("virtue.tester_ungated_within_base"),
        BTreeMap::from([("characteristic".into(), Id::new("characteristic.str"))]),
    )]);
    entity.characteristics.insert(Characteristic::Str, 3);
    let effective = effective_characteristic_score(&entity, &rs, Characteristic::Str);
    assert!(
        effective <= 3,
        "an ungated within_base delta must clamp to the base cap even with no gate \
         at all — effective: {effective}"
    );
}
