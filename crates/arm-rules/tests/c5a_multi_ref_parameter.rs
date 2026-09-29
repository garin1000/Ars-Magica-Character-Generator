//! C5a (`docs/vf-audit/design-c0-parameter-model.md` § 8, § 9, § 10): the
//! engine-only half of D9 part 3's multi-valued parameter — `ParamType::MultiRef`,
//! `ParameterDomain::Spell`, and the load-time integrity/validation reds that
//! must exist before the three Corrupted entries (C5c) or the multi-select
//! picker (C5b) touch any of this.
//!
//! RED-CHECKPOINT PHASE 1: no enforcement logic has been wired in yet (see
//! `validation::selections::validate_selection_parameters`, which still just
//! `continue`s past a `Multi` value). These tests pin the CURRENTLY MISSING
//! behavior and are expected to fail until phase 2 wires the checks in — see
//! this crate's own module-level doc comment on TDD discipline
//! (`CLAUDE.md`'s "RED must fail for the right reason").

use arm_rules::migration::load_entity_migrating;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Entity, EntityKind, Id, RulesetRef, Selection, SelectionParamValue};
use arm_rules::validation::{ValidationIssue, validate};
use std::collections::{BTreeMap, BTreeSet};

/// One Flaw (`flaw.tester`) declaring a `targets` parameter of type
/// `multi_ref`/domain `ability`, plus an ability catalogue with two entries —
/// the minimal fixture every test below builds an entity against. Modelled
/// directly on the shipped `flaw.corrupted_abilities` shape (D15, § 8's worked
/// example), but under a fixture id so this file does not depend on C5c's
/// data landing first.
fn ruleset_with_multi_ref_flaw() -> Ruleset {
    let items = r#"[
      { "id": "flaw.tester", "kind": "flaw", "classification": "uncomputed_rule",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [{ "key": "targets", "type": "multi_ref", "domain": "ability" }] },
      { "id": "flaw.personality_filler", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
    ]"#;
    let types = r#"[{
      "id": "small_budget",
      "budget": { "virtue_points": 10, "flaw_points": 10 },
      "permitted_categories": ["general", "personality"],
      "creation_phases": []
    }]"#;
    let abilities = r#"{ "abilities": [
      { "id": "ability.awareness", "category": "general" },
      { "id": "ability.brawl", "category": "martial" }
    ] }"#;
    Ruleset::from_sources(RulesetSources {
        id: "t",
        version: "1",
        point_items: items,
        type_profiles: types,
        abilities: Some(abilities),
        ..RulesetSources::default()
    })
    .expect("the multi_ref fixture ruleset loads")
}

fn entity_with_selections(selections: Vec<Selection>) -> Entity {
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("small_budget"),
        RulesetRef::new(Id::new("t"), "1"),
    );
    entity.selections = selections;
    entity
}

fn codes(entity: &Entity, ruleset: &Ruleset) -> BTreeSet<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

/// § 8's second migration red: a save whose `targets` value is the OLD,
/// pre-`multi_ref` single-scalar shape (a plain `Single(Id)`) on a key the
/// ruleset now declares `multi_ref` must be reported as `param_wrong_shape`
/// — a wrong SHAPE, distinct from an unmade choice (`missing_param`) and from
/// an out-of-domain value (`unknown_param_value`). Currently RED: nothing
/// checks `param.param_type == MultiRef` against the stored value's shape,
/// so today this scalar quietly resolves as an ordinary `ability` value (it
/// IS a real ability id) and raises nothing at all.
#[test]
fn a_scalar_value_where_multi_ref_is_declared_yields_wrong_shape_not_missing() {
    let ruleset = ruleset_with_multi_ref_flaw();
    let selection = Selection::with_params(
        Id::new("flaw.tester"),
        BTreeMap::from([("targets".to_string(), Id::new("ability.awareness"))]),
    );
    let entity = entity_with_selections(vec![selection]);
    let found = codes(&entity, &ruleset);

    assert!(
        found.contains(ValidationIssue::CODE_PARAM_WRONG_SHAPE),
        "a scalar value under a multi_ref-declared key must raise param_wrong_shape, found: {found:?}"
    );
    assert!(
        !found.contains(ValidationIssue::CODE_MISSING_PARAM),
        "a present (if wrongly-shaped) value must not be reported as an unmade choice: {found:?}"
    );
}

/// The blank-value precedent already covers this (`param_value_is_blank`
/// treats `Multi(empty set)` as blank, exactly like an empty text box) — this
/// pins that the SAME rule holds once the key is genuinely declared
/// `multi_ref` in the ruleset, not just hypothetically. Expected to already be
/// GREEN: `param_value_is_blank` was written generically in C0b, before this
/// slice's ruleset shape existed to exercise it.
#[test]
fn an_empty_multi_set_is_reported_as_missing_param() {
    let ruleset = ruleset_with_multi_ref_flaw();
    let selection = Selection {
        item_ref: Id::new("flaw.tester"),
        params: BTreeMap::from([(
            "targets".to_string(),
            SelectionParamValue::Multi(BTreeSet::new()),
        )]),
    };
    let entity = entity_with_selections(vec![selection]);
    let found = codes(&entity, &ruleset);

    assert!(
        found.contains(ValidationIssue::CODE_MISSING_PARAM),
        "an empty set must be treated as a choice not yet made: {found:?}"
    );
    assert!(
        !found.contains(ValidationIssue::CODE_PARAM_WRONG_SHAPE),
        "an empty set is the right SHAPE (Multi), just blank: {found:?}"
    );
}

/// § 8: `{A,B}` and `{B,A}` are the same value by construction (`BTreeSet`),
/// so serialization is already canonical (sorted, deduplicated) with no
/// separate step. Already GREEN — a property of the type chosen in C0b, not
/// new code this slice adds.
#[test]
fn multi_values_built_in_either_order_serialize_identically_and_sorted() {
    let a = SelectionParamValue::Multi(BTreeSet::from([
        Id::new("ability.brawl"),
        Id::new("ability.awareness"),
    ]));
    let b = SelectionParamValue::Multi(BTreeSet::from([
        Id::new("ability.awareness"),
        Id::new("ability.brawl"),
    ]));
    assert_eq!(
        a, b,
        "the same set built in either order must compare equal"
    );

    let json = serde_json::to_string(&a).unwrap();
    assert_eq!(
        json, r#"["ability.awareness","ability.brawl"]"#,
        "a Multi value must serialize sorted"
    );
    assert_eq!(serde_json::to_string(&b).unwrap(), json);
}

/// § 8's `max_per_target` worked claim: two selections of the SAME item naming
/// the SAME set (built in different orders) collide as duplicates under the
/// item's default `max_per_target: 1` — "you may only take this Flaw once,
/// though it can affect multiple Abilities" (ArMDE:5851-style wording). Already
/// GREEN in principle (duplicate detection already keys on `(item_ref,
/// params)`, and `Multi`'s `BTreeSet` equality needs no extra canonicalization)
/// — this test proves the claim end-to-end through the real validator, not
/// just at the type level.
#[test]
fn two_selections_naming_the_same_set_in_different_order_collide_as_duplicates() {
    let ruleset = ruleset_with_multi_ref_flaw();
    let first = Selection {
        item_ref: Id::new("flaw.tester"),
        params: BTreeMap::from([(
            "targets".to_string(),
            SelectionParamValue::Multi(BTreeSet::from([
                Id::new("ability.brawl"),
                Id::new("ability.awareness"),
            ])),
        )]),
    };
    let second = Selection {
        item_ref: Id::new("flaw.tester"),
        params: BTreeMap::from([(
            "targets".to_string(),
            SelectionParamValue::Multi(BTreeSet::from([
                Id::new("ability.awareness"),
                Id::new("ability.brawl"),
            ])),
        )]),
    };
    let entity = entity_with_selections(vec![first, second]);
    let found = codes(&entity, &ruleset);

    assert!(
        found.contains(ValidationIssue::CODE_DUPLICATE_SELECTION),
        "two copies naming the same set (in either build order) must collide under \
         max_per_target: 1, found: {found:?}"
    );
}

/// A minimal hand-written save naming `flaw.tester`'s `targets` parameter as
/// `targets_json` verbatim, so a caller can pass a bare string, a small array,
/// or a hostile shape.
fn minimal_entity_json(targets_json: &str) -> String {
    format!(
        r#"{{
          "schema_version": 18,
          "ruleset": {{ "id": "t", "version": "1" }},
          "entity_kind": "character",
          "type_id": "small_budget",
          "selections": [{{ "ref": "flaw.tester", "params": {{ "targets": {targets_json} }} }}]
        }}"#
    )
}

/// § 8's trust-boundary concern: the save file the user opens is the hostile
/// -input surface (`CLAUDE.md`), so a crafted array with thousands of
/// duplicate entries must load without panicking and must still dedupe down
/// to the distinct ids — `BTreeSet` deserialization already gives this for
/// free (C0b), so this is expected to already be GREEN; it is pinned here
/// because C5a is the slice that first makes this shape reachable through a
/// real ruleset-declared `multi_ref` parameter.
#[test]
fn a_large_duplicate_laden_array_loads_without_panicking_and_dedupes() {
    let ruleset = ruleset_with_multi_ref_flaw();
    let names = BTreeMap::new();
    let mut ids: Vec<&str> = Vec::new();
    for _ in 0..5000 {
        ids.push("\"ability.awareness\"");
        ids.push("\"ability.brawl\"");
    }
    let targets_json = format!("[{}]", ids.join(","));
    let json = minimal_entity_json(&targets_json);
    let loaded = load_entity_migrating(&json, arm_rules::DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect("a large duplicate-laden array must load without panicking");
    let selection = &loaded.entity.selections[0];
    assert_eq!(
        selection.params.get("targets"),
        Some(&SelectionParamValue::Multi(BTreeSet::from([
            Id::new("ability.awareness"),
            Id::new("ability.brawl"),
        ]))),
        "10000 duplicate entries must dedupe down to the two distinct ids"
    );
}

/// A `Multi` value must round-trip through a second load/save cycle to a
/// stable fixed point, exactly like every other field `migration.rs`'s own
/// `a_migrated_save_is_byte_stable_across_a_save_load_save_cycle` pins.
/// Expected to already be GREEN (no new fold changes the shape).
#[test]
fn a_multi_ref_selection_reloads_idempotently() {
    let ruleset = ruleset_with_multi_ref_flaw();
    let names = BTreeMap::new();
    let json = minimal_entity_json(r#"["ability.brawl","ability.awareness","ability.brawl"]"#);
    let mut first = load_entity_migrating(&json, arm_rules::DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect("loads")
        .entity;
    first.normalize();
    let first_bytes = serde_json::to_string(&first).unwrap();

    let mut second =
        load_entity_migrating(&first_bytes, arm_rules::DEFAULT_SAGA_YEAR, &ruleset, &names)
            .expect("reloads")
            .entity;
    second.normalize();
    let second_bytes = serde_json::to_string(&second).unwrap();

    assert_eq!(first_bytes, second_bytes, "a reload must be byte-stable");
}

/// The bump row itself (§ 10: C5a is "18 → 19", CV having already claimed
/// 17 → 18 ahead of it) — phase 2 moved `selection_param_value.rs`'s own
/// `c0b_does_not_bump_schema_version` and
/// `crates/arm-app/tests/commands.rs::the_frontend_mirrors_the_engine_schema_version`'s
/// TS-side mirror (`ui/src/lib/state.svelte.ts::SCHEMA_VERSION`) to 19 alongside
/// this constant, per `SCHEMA_VERSION`'s own doc comment on why the bump is a
/// pure version marker with no fold. The constant has since moved once more, to
/// 20 for F1's unrelated `EquipmentSlot::loadout` move (K5) — this assertion
/// tracks the current value, not C5a's own contribution to it (still 19 → one
/// bump, same as it always was).
#[test]
fn c5a_bumps_schema_version_to_19() {
    assert_eq!(
        arm_rules::migration::SCHEMA_VERSION,
        20,
        "C5a's own bump (§ 10) still stands at 18 -> 19; later bumps (F1/K5) move \
         the constant further, which this assertion tracks"
    );
}
