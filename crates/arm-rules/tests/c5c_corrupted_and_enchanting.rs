//! C5c (`docs/vf-audit/design-c0-parameter-model.md` § 8's worked examples,
//! § 2, § 10; `docs/vf-audit/decisions.md` D15): the three Corrupted entries
//! get D9 part 3's `multi_ref` parameter and D15's one-treatment reclassification,
//! and `virtue.enchanting_ability` gets F-63's `AbilityScoreGrantParam` so its
//! floor grant lands on the player's chosen medium instance, not unconditionally.
//!
//! **F-42 (Custos) and F-317 (Templar Specialist) are NOT this slice's job** —
//! design-c0 § 8 deliberately narrows the plan's C5c grouping: both already
//! resolved in C1 via the exclusive-choice gate (verified directly against the
//! shipped `rules/core/virtues_flaws.json`: `virtue.custos` and
//! `virtue.templar_specialist` already carry a gated `ability_authorization`).
//!
//! RED-CHECKPOINT PHASE 1: `Effect::AbilityScoreGrantParam` exists as a
//! signature-only stub (every exhaustive `match Effect` site treats it as a
//! documented no-op; only `docs/vf-audit/design-c0-parameter-model.md` § 1a's
//! `AbilityScoreGrant`-classified sites are stubbed here, per this slice's
//! phase-1 mandate). No shipped `rules/core/*.json` data has been touched yet —
//! every test below is expected to fail until phase 2 edits the data and wires
//! the real resolution logic (`effective/ability.rs::granted_ability_floor`/
//! `ability_score_floors`, `validation/prereq.rs`'s `AbilityMin` reading).

use arm_rules::migration::load_entity_migrating;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Classification, Effect, Entity, EntityKind, Id, ParamType,
    ParameterDomain, RulesetRef, Selection, SelectionParamValue, SpellSelection,
};
use arm_rules::validation::{ValidationIssue, validate};
use arm_rules::{ModifierFamily, effective_ability_score, surfaced_modifiers};
use std::collections::{BTreeMap, BTreeSet};

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

/// The full shipped ruleset — every test below exercises the real
/// `rules/core/virtues_flaws.json` entries, so referential integrity needs
/// every registry those 655 entries collectively touch (Arts, Houses,
/// spells), on `data_integrity.rs::load_ruleset_with_spells`'s precedent.
fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(SHIPPED_HOUSES),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("the shipped ruleset loads")
}

fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn codes(entity: &Entity, ruleset: &Ruleset) -> BTreeSet<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

fn multi_selection(item: &str, targets: BTreeSet<Id>) -> Selection {
    Selection {
        item_ref: Id::new(item),
        params: BTreeMap::from([("targets".to_string(), SelectionParamValue::Multi(targets))]),
    }
}

// ---------------------------------------------------------------------------
// D9 part 3 / D15: the three Corrupted entries declare `multi_ref` parameters
// over the right domain, as data-integrity tests on the shipped catalogue.
// ---------------------------------------------------------------------------

/// § 8's worked example, verified against the shipped data rather than a
/// fixture: each Corrupted entry must declare a `multi_ref` parameter over the
/// domain its own passage names (Abilities, Arts, spells). RED today: none of
/// the three ships a `parameters` array at all.
#[test]
fn each_corrupted_entry_declares_its_multi_ref_parameter() {
    let rs = load_ruleset();
    for (id, expected_domain) in [
        ("flaw.corrupted_abilities", ParameterDomain::Ability),
        ("flaw.corrupted_arts", ParameterDomain::Art),
        ("flaw.corrupted_spells", ParameterDomain::Spell),
    ] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} ships"));
        let param = item.parameters.first().unwrap_or_else(|| {
            panic!("{id} must declare a parameter (D9 part 3's multi-valued choice)")
        });
        assert_eq!(
            param.param_type,
            ParamType::MultiRef,
            "{id}'s parameter must be multi_ref (\"can affect multiple...\")"
        );
        assert_eq!(
            param.domain, expected_domain,
            "{id}'s parameter must resolve against {expected_domain}"
        );
    }
}

/// D15: all three become `uncomputed_rule` with no effects at all —
/// `flaw.corrupted_arts` moves off `creation_effect`, and
/// `flaw.corrupted_spells` moves off `in_play_effect` and loses its
/// `special_casting_mod`. RED today: `corrupted_arts` is `creation_effect` and
/// `corrupted_spells` is `in_play_effect` carrying that effect.
#[test]
fn all_three_corrupted_entries_are_uncomputed_rule_with_no_effects() {
    let rs = load_ruleset();
    for id in [
        "flaw.corrupted_abilities",
        "flaw.corrupted_arts",
        "flaw.corrupted_spells",
    ] {
        let item = rs
            .item(&Id::new(id))
            .unwrap_or_else(|| panic!("{id} ships"));
        assert_eq!(
            item.classification,
            Classification::UncomputedRule,
            "{id} must be uncomputed_rule (D15: one mechanic, one treatment)"
        );
        assert!(
            item.effects.is_empty(),
            "{id} must carry no effects (D15): found {:?}",
            item.effects
        );
    }
}

/// D15 obligation 4: all three carry the FULL passage in `description`, both
/// locales — `corrupted_abilities` already does; `corrupted_arts` and
/// `corrupted_spells` today carry only a one-sentence `summary`.
#[test]
fn all_three_corrupted_entries_carry_a_description_in_both_locales() {
    for (lang, json) in [
        (
            "en",
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
        ),
        (
            "de",
            include_str!("../../../rules/i18n/de/virtues_flaws.json"),
        ),
    ] {
        let parsed: serde_json::Value = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("rules/i18n/{lang}/virtues_flaws.json is valid JSON: {e}"));
        for id in [
            "flaw.corrupted_abilities",
            "flaw.corrupted_arts",
            "flaw.corrupted_spells",
        ] {
            let description = parsed
                .get(id)
                .and_then(|entry| entry.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or("");
            assert!(
                !description.trim().is_empty(),
                "{id} must carry a non-empty description in {lang} (D15/D5)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Functional: a character choosing several targets validates; a duplicate
// naming the same set (in a different build order) collides.
// ---------------------------------------------------------------------------

/// A companion choosing Corrupted Abilities for two named Abilities must not
/// trip `missing_param`/`unexpected_param`/`param_wrong_shape`/
/// `unknown_param_value` — RED today because the shipped item declares no
/// `parameters` at all, so `targets` is an undeclared, `unexpected_param` key.
#[test]
fn corrupted_abilities_selection_naming_two_targets_validates_cleanly() {
    let rs = load_ruleset();
    let selection = multi_selection(
        "flaw.corrupted_abilities",
        BTreeSet::from([Id::new("ability.awareness"), Id::new("ability.brawl")]),
    );
    let found = codes(&entity("companion", vec![selection]), &rs);

    for bad_code in [
        ValidationIssue::CODE_MISSING_PARAM,
        ValidationIssue::CODE_UNEXPECTED_PARAM,
        ValidationIssue::CODE_PARAM_WRONG_SHAPE,
        ValidationIssue::CODE_UNKNOWN_PARAM_VALUE,
    ] {
        assert!(
            !found.contains(bad_code),
            "naming two real Abilities must not raise {bad_code}, found: {found:?}"
        );
    }
}

/// "You may only take this Flaw once, though it can affect multiple Abilities"
/// (ArMDE:5851) — a second copy naming the SAME set, built in a different
/// order, must collide under the default `max_per_target: 1`. RED today for
/// the same reason as above (no parameter declared, so no duplicate-target key
/// can be built from it at all).
#[test]
fn a_second_copy_naming_the_same_set_is_refused_as_duplicate() {
    let rs = load_ruleset();
    let first = multi_selection(
        "flaw.corrupted_abilities",
        BTreeSet::from([Id::new("ability.brawl"), Id::new("ability.awareness")]),
    );
    let second = multi_selection(
        "flaw.corrupted_abilities",
        BTreeSet::from([Id::new("ability.awareness"), Id::new("ability.brawl")]),
    );
    let found = codes(&entity("companion", vec![first, second]), &rs);

    assert!(
        found.contains(ValidationIssue::CODE_DUPLICATE_SELECTION),
        "two copies naming the same target set must collide, found: {found:?}"
    );
}

/// Corrupted Spells (ArMDE:5859-5863) resolves its `targets` against the
/// character's OWN learned spells, never the ruleset's whole catalogue: naming
/// a spell the magus never learned must raise `unknown_param_value`. RED
/// today: no parameter is declared at all, so `targets` is `unexpected_param`,
/// never a spell-domain resolution failure.
#[test]
fn corrupted_spells_refuses_an_unlearned_spell() {
    let rs = load_ruleset();
    let mut e = entity(
        "magus",
        vec![multi_selection(
            "flaw.corrupted_spells",
            BTreeSet::from([Id::new("spell.pilum_of_fire")]),
        )],
    );
    // The character has learned no spells at all.
    e.spells = Vec::new();
    let found = codes(&e, &rs);

    assert!(
        found.contains(ValidationIssue::CODE_UNKNOWN_PARAM_VALUE),
        "naming an unlearned spell must raise unknown_param_value, found: {found:?}"
    );
}

/// The positive half: a spell the character HAS learned must resolve cleanly
/// against `targets`. RED today for the same reason as the refusal case above
/// (the parameter does not exist yet to resolve anything).
#[test]
fn corrupted_spells_accepts_a_learned_spell() {
    let rs = load_ruleset();
    let mut e = entity(
        "magus",
        vec![multi_selection(
            "flaw.corrupted_spells",
            BTreeSet::from([Id::new("spell.pilum_of_fire")]),
        )],
    );
    e.spells = vec![SpellSelection {
        spell: Id::new("spell.pilum_of_fire"),
        level: None,
        mastery: None,
        parameter: None,
        mastery_abilities: Vec::new(),
    }];
    let found = codes(&e, &rs);

    for bad_code in [
        ValidationIssue::CODE_MISSING_PARAM,
        ValidationIssue::CODE_UNEXPECTED_PARAM,
        ValidationIssue::CODE_PARAM_WRONG_SHAPE,
        ValidationIssue::CODE_UNKNOWN_PARAM_VALUE,
    ] {
        assert!(
            !found.contains(bad_code),
            "a learned spell must resolve cleanly, found {bad_code} in: {found:?}"
        );
    }
}

/// D15 obligation 2: `flaw.corrupted_spells` loses its `special_casting_mod`
/// effect, so it must no longer surface a `SpecialCasting`-family modifier at
/// all. RED today: the shipped entry still carries
/// `{ "type": "special_casting_mod", "kind": "circumstantial" }`, so this
/// currently DOES surface.
#[test]
fn corrupted_spells_no_longer_surfaces_a_special_casting_modifier() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![Selection::new(Id::new("flaw.corrupted_spells"))],
    );
    let surfaced = surfaced_modifiers(&e, &rs);
    assert!(
        !surfaced
            .iter()
            .any(|m| m.family == ModifierFamily::SpecialCasting),
        "flaw.corrupted_spells must no longer contribute a SpecialCasting-family \
         surfaced modifier (D15 deletes special_casting_mod), found: {surfaced:?}"
    );
}

// ---------------------------------------------------------------------------
// F-63: `virtue.enchanting_ability` grants the Enchanting floor only at the
// player's chosen medium instance. A fixture ruleset, not shipped data — the
// shipped `virtue.enchanting_ability`/`ability.enchanting` entries have not
// been touched in this phase (see this file's own header comment); the
// AbilityScoreGrantParam type/mechanism is exercised directly instead, on
// C5a's precedent of not depending on this slice's own data landing first.
// ---------------------------------------------------------------------------

/// Mirrors the shipped `virtue.enchanting_ability` shape ONCE FIXED (D2's
/// design § 2): a `medium` parameter of domain `text`, and an
/// `ability_score_grant_param` effect whose `instance` is bound to it.
fn ruleset_with_enchanting_ability_shape() -> Ruleset {
    let items = r#"[
      { "id": "virtue.tester_enchanting", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["supernatural"], "entity_kinds": ["character"],
        "parameters": [{ "key": "medium", "type": "ref", "domain": "text" }],
        "effects": [{ "type": "ability_score_grant_param", "ability": "ability.tester_enchanting",
          "instance": { "param": "medium" }, "amount": 1 }] },
      { "id": "flaw.personality_filler", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
    ]"#;
    let types = r#"[{
      "id": "small_budget",
      "budget": { "virtue_points": 10, "flaw_points": 10 },
      "permitted_categories": ["supernatural", "personality"],
      "creation_phases": []
    }]"#;
    // `ability.tester_enchanting` is parameterized on `medium` — the same
    // shape the shipped `ability.enchanting` needs once F-63 lands (currently
    // `rules/core/abilities.json`'s `ability.enchanting` declares no
    // `parameter` at all, so the real entry cannot resolve an instance either;
    // flagged as a phase-2 data gap in this slice's report).
    let abilities = r#"{ "abilities": [
      { "id": "ability.tester_enchanting", "category": "supernatural", "parameter": "medium" }
    ] }"#;
    Ruleset::from_sources(RulesetSources {
        id: "t",
        version: "1",
        point_items: items,
        type_profiles: types,
        abilities: Some(abilities),
        ..RulesetSources::default()
    })
    .expect("the enchanting-ability fixture ruleset loads")
}

fn fixture_entity(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("small_budget"),
        RulesetRef::new(Id::new("t"), "1"),
    );
    e.selections = selections;
    e
}

/// F-63's positive case: the floor applies at the CHOSEN medium instance.
/// RED today: `granted_ability_floor` only matches `Effect::AbilityScoreGrant`
/// (the fixed-target variant), so `AbilityScoreGrantParam` contributes nothing
/// and the floor stays 0.
#[test]
fn enchanting_ability_grants_the_floor_at_the_chosen_medium() {
    let rs = ruleset_with_enchanting_ability_shape();
    let selection = Selection::with_params(
        Id::new("virtue.tester_enchanting"),
        BTreeMap::from([("medium".to_string(), Id::new("storytelling"))]),
    );
    let e = fixture_entity(vec![selection]);

    assert_eq!(
        effective_ability_score(
            &e,
            &rs,
            &Id::new("ability.tester_enchanting"),
            Some("storytelling")
        ),
        1,
        "the floor must apply to the chosen medium instance"
    );
}

/// F-63's negative case, the one the finding is actually about: the grant
/// applies ONLY to the player's chosen medium, never to every instance of the
/// Ability — a DIFFERENT medium the character never chose must stay at 0. RED
/// today for the same reason as above (the stub never floors anything).
#[test]
fn enchanting_ability_does_not_grant_the_floor_at_a_different_medium() {
    let rs = ruleset_with_enchanting_ability_shape();
    let selection = Selection::with_params(
        Id::new("virtue.tester_enchanting"),
        BTreeMap::from([("medium".to_string(), Id::new("storytelling"))]),
    );
    let e = fixture_entity(vec![selection]);

    assert_eq!(
        effective_ability_score(
            &e,
            &rs,
            &Id::new("ability.tester_enchanting"),
            Some("dance")
        ),
        0,
        "an unchosen medium instance must not receive the floor"
    );
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.tester_enchanting"), None),
        0,
        "the parameter-less instance must not receive the floor either"
    );
}

/// The type itself: `Effect::AbilityScoreGrantParam` round-trips through serde
/// under the `ability_score_grant_param` tag, matching the "Foo"/"FooParam"
/// naming convention every other pair in this enum already uses. A pure
/// type-level sanity check for the phase-1 stub — expected to already be
/// GREEN (it only proves the variant exists and (de)serializes, not that
/// anything resolves it).
#[test]
fn ability_score_grant_param_round_trips_through_json() {
    let effect = Effect::AbilityScoreGrantParam {
        ability: Id::new("ability.tester_enchanting"),
        instance: Some(arm_rules::types::ParamValue::Bound {
            param: "medium".to_string(),
        }),
        amount: 1,
    };
    let json = serde_json::to_string(&effect).unwrap();
    assert_eq!(
        json,
        r#"{"type":"ability_score_grant_param","ability":"ability.tester_enchanting","instance":{"param":"medium"},"amount":1}"#
    );
    let round_tripped: Effect = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, effect);
}

/// A hand-written save is this app's trust boundary (`CLAUDE.md`): an old save
/// holding a Corrupted entry from before this slice — no `targets` param at
/// all — must load without panicking and must report `missing_param` (the
/// standing policy for an older save opened under new-and-stricter rules,
/// same as D10's). Expected to already be GREEN once the ruleset declares the
/// parameter (no migration folds anything here; this only pins the load-time
/// behavior a human reads `docs/open-todos.md`'s list against).
#[test]
fn an_older_save_with_no_targets_param_reports_missing_param_once_declared() {
    let rs = load_ruleset();
    let names = BTreeMap::new();
    let json = r#"{
      "schema_version": 19,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "companion",
      "selections": [{ "ref": "flaw.corrupted_abilities" }]
    }"#;
    let loaded = load_entity_migrating(json, arm_rules::DEFAULT_SAGA_YEAR, &rs, &names)
        .expect("an older save with no targets param must load without panicking");
    let found = codes(&loaded.entity, &rs);
    assert!(
        found.contains(ValidationIssue::CODE_MISSING_PARAM),
        "an older save with no targets param must report missing_param once the ruleset \
         declares it, found: {found:?}"
    );
}

// ---------------------------------------------------------------------------
// Phase 2 ruling (orchestrator, "PHASE 2 go-ahead"): design-c0 § 1a row 12 is
// STALE, not the code — `Effect::AbilityScoreGrant` already authorizes ("a
// free score in an Ability is permission to have it",
// `effective/xp.rs::ability_authorizations`), so `AbilityScoreGrantParam` must
// authorize too, scoped to the RESOLVED instance, reusing the existing D59/CV
// Bound/Link model rather than a second path.
// ---------------------------------------------------------------------------

/// A category the mechanism actually gates (`arcane`), NOT Enchanting's real
/// `supernatural` category — `validate_ability_authorization`'s own doc
/// comment excludes Supernatural from this specific check ("has its own,
/// stricter rule ... enforced by `validate_supernatural_abilities`"), so a
/// supernatural fixture would pass this assertion either way and prove
/// nothing about the fix. `arcane` genuinely exercises
/// `ability_authorizations`'s new arm.
fn ruleset_with_gated_enchanting_shape() -> Ruleset {
    let items = r#"[
      { "id": "virtue.tester_enchanting_link", "kind": "virtue", "classification": "creation_effect",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [{ "key": "medium", "type": "ref", "domain": "text" }],
        "effects": [{ "type": "ability_score_grant_param", "ability": "ability.tester_enchanting_gated",
          "instance": { "param": "medium" }, "amount": 1 }] },
      { "id": "flaw.personality_filler", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
    ]"#;
    let types = r#"[{
      "id": "small_budget",
      "budget": { "virtue_points": 10, "flaw_points": 10 },
      "permitted_categories": ["general", "arcane", "personality"],
      "creation_phases": []
    }]"#;
    let abilities = r#"{
      "advancement": [{ "score": 1, "total_xp": 5 }],
      "abilities": [
        { "id": "ability.tester_enchanting_gated", "category": "arcane", "parameter": "medium" }
      ],
      "categories_requiring_virtue": ["arcane"]
    }"#;
    Ruleset::from_sources(RulesetSources {
        id: "t2",
        version: "1",
        point_items: items,
        type_profiles: types,
        abilities: Some(abilities),
        ..RulesetSources::default()
    })
    .expect("the gated enchanting-shape fixture ruleset loads")
}

/// The ruling-1 RED: a character holding Enchanting Ability's granted medium
/// instance — via the D59/CV `Linked` model (ruling 2), not a re-typed string
/// — must raise NO `ability_category_requires_virtue` finding for that bought
/// score. RED today: `AbilityScoreGrantParam` is a no-op in
/// `ability_authorizations`, so nothing authorizes `ability.tester_enchanting_gated`
/// at all and the gated `arcane` category blocks it.
#[test]
fn enchanting_style_grant_authorizes_the_linked_bought_instance() {
    let rs = ruleset_with_gated_enchanting_shape();
    let selection = Selection::with_params(
        Id::new("virtue.tester_enchanting_link"),
        BTreeMap::from([("medium".to_string(), Id::new("storytelling"))]),
    );
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("small_budget"),
        RulesetRef::new(Id::new("t2"), "1"),
    );
    e.selections = vec![selection];
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.tester_enchanting_gated"),
        parameter: Some(AbilityParameterValue::Linked {
            item: Id::new("virtue.tester_enchanting_link"),
            param: "medium".to_string(),
        }),
        score: 1,
        specialty: None,
    }];

    let found = codes(&e, &rs);
    assert!(
        !found.contains(ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE),
        "a bought score LINKED to the granting Virtue's own medium parameter must not need \
         its own separate authorization, found: {found:?}"
    );
}
