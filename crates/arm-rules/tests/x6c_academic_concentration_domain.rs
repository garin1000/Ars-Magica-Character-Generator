//! X6c academic_concentration_subject retype — Phase 1 reds for the engine
//! fix that unblocks it (`tmp/ac-handover.md`, `tmp/x6c-handover.md`'s "A
//! discovered, out-of-scope blocker" section, `crates/arm-rules/RULES.md`'s
//! "X6c" entry).
//!
//! `Effect::AbilityRollModParam`'s declared parameter is hardcoded to
//! `ParameterDomain::Text` in
//! `ruleset::integrity.rs::validate_effect_refs` (the `(param, expected,
//! kind)` tuple match), which rejects at LOAD time any ruleset that gives the
//! effect's parameter `domain: enumerated` instead — this is what blocked
//! retyping `virtue.academic_concentration_subject`'s `subject` parameter to
//! the seven Artes Liberales (F-03, ArMDE:3362-3367, :7310). The fix widens
//! that single hardcoded expectation to accept EITHER `Text` or `Enumerated`
//! (mirroring `validate_deficient_art_effect`'s existing Technique-or-Form
//! dual acceptance, a few hundred lines above the same match), not a removal
//! of the check — a THIRD domain (e.g. `ability`) must still fail load.
//!
//! Minimal in-test rulesets throughout (`Ruleset::from_sources`), mirroring
//! `x6a_parameter_engine.rs`'s convention, since the shipped
//! `rules/core/virtues_flaws.json` still carries `domain: text` for the real
//! `virtue.academic_concentration_subject` until Phase 2 re-applies the
//! enumerated data.

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;

/// Same shape as every other X-series test file's minimal type profile (e.g.
/// `x6a_parameter_engine.rs::COMPANION_TYPE`): one generous, unnarrowed
/// "companion" type so a test's own point items are always legal to select.
const COMPANION_TYPE: &str = r#"[
  { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
    "permitted_categories": ["general"], "creation_phases": [] }
]"#;

fn entity(selections: Vec<Selection>) -> Entity {
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

/// An `ability_roll_mod_param` effect naming an ENUMERATED-domain parameter
/// must load cleanly, not just a text-domain one — the engine fix the
/// subject retype needs. Also pins the usual enumerated behavior once loaded:
/// a listed value resolves, and a free-text/unlisted value does not (D70/
/// Q-X6-4: no migration of an old save's free text).
#[test]
fn ability_roll_mod_param_accepts_an_enumerated_domain_parameter() {
    let items = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.academic_concentration_subject", "kind": "virtue",
        "classification": "in_play_effect", "magnitude": "minor",
        "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [{ "key": "subject", "type": "ref", "domain": "enumerated",
          "values": ["subject.grammar", "subject.logic"] }],
        "effects": [{ "type": "ability_roll_mod_param", "param": "subject", "amount": 3 }] }
    ]"#;
    let rs = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: items,
        type_profiles: COMPANION_TYPE,
        ..RulesetSources::default()
    })
    .expect(
        "an ability_roll_mod_param effect naming an enumerated-domain parameter \
         must load, not only a text-domain one — this is the engine fix the \
         academic_concentration_subject retype needs",
    );

    let resolved = entity(vec![Selection::with_params(
        Id::new("virtue.academic_concentration_subject"),
        BTreeMap::from([("subject".to_string(), Id::new("subject.logic"))]),
    )]);
    assert!(
        !issue_codes(&resolved, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "subject.logic must resolve against the now-enumerated subject parameter"
    );

    let old_free_text = entity(vec![Selection::with_params(
        Id::new("virtue.academic_concentration_subject"),
        BTreeMap::from([("subject".to_string(), Id::new("Logic"))]),
    )]);
    assert!(
        issue_codes(&old_free_text, &rs)
            .iter()
            .any(|c| c == "unknown_param_value"),
        "Q-X6-4/D70: an old save's free-text value ('Logic') must NOT be \
         migrated once the parameter is enumerated — it reports unknown_param_value"
    );
}

/// The flip side: widening the check to Text-or-Enumerated must not turn it
/// into "anything goes" — an `ability_roll_mod_param` effect naming a
/// parameter of a THIRD domain (neither text nor enumerated) must still fail
/// load, with a clear error naming the offending item and BOTH accepted
/// domains.
#[test]
fn ability_roll_mod_param_rejects_a_non_text_non_enumerated_domain() {
    let items = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.bogus_academic_concentration", "kind": "virtue",
        "classification": "in_play_effect", "magnitude": "minor",
        "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [{ "key": "subject", "type": "ref", "domain": "ability" }],
        "effects": [{ "type": "ability_roll_mod_param", "param": "subject", "amount": 3 }] }
    ]"#;
    let err = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: items,
        type_profiles: COMPANION_TYPE,
        ..RulesetSources::default()
    })
    .unwrap_err();
    let message = format!("{err}");
    assert!(
        message.contains("virtue.bogus_academic_concentration")
            && message.contains("ability_roll_mod_param")
            && message.contains("text")
            && message.contains("enumerated"),
        "load error must name the offending item/effect and say both 'text' and \
         'enumerated' were acceptable, got: {message}"
    );
}
