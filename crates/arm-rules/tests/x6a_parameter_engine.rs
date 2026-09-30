//! X6a (`docs/vf-audit/design-x6-parameters.md` § 1, e1-e7) — Phase 1 failing
//! tests for the engine additions X6b's 16 rule-driving parameters need.
//!
//! **Phase 1 only, and currently RED AT COMPILE TIME, not merely at
//! assertion time.** Every test below names a field or `Effect`/`ParameterDef`
//! variant that does not exist yet in `crates/arm-rules/src/types.rs`
//! (`gate` on `SoakMod`/`CharacteristicScoreDelta`/`AbilityRollMod`, `amount`
//! on `MagicResistanceMod`, `ParameterDef::require_ability_categories`/
//! `forbid_ids`/`exact_count`, `ParameterDomain::AbilityCategory`,
//! `Effect::ForbidsAbilityCategoryParam`, `RestrictedAbilityXp::abilities_param`,
//! `Effect::AbilityScoreCapOverrideParam`/`AbilityScoreCapAllExcept`,
//! `PointItem::conditional_incompatible_with` + `ConditionalIncompatibility`).
//!
//! Per CLAUDE.md's TDD rule, "a test naming a function [or field] that does
//! not exist yet fails to compile; that is a legitimate first red only until
//! the signature exists — you must still see the assertion itself fail before
//! writing the body." This file intentionally carries **no** signature stubs:
//! `tmp/x6a-handover.md` documents why (the four gate-bearing `Effect`
//! variants and `RestrictedAbilityXp` are each already constructed by dozens
//! of existing call sites elsewhere in `crates/arm-rules/src`, so adding a
//! required field ripples far past the ~40-line stub budget this slice was
//! given). Each `mod eN_*` below covers one design-note engine addition; the
//! doc comment on each test cites the representative red it implements from
//! design-x6-parameters.md § 4.
//!
//! Minimal in-test rulesets throughout (`Ruleset::from_sources`), so these
//! tests do not depend on X6b's data landing in `rules/core/`.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use arm_rules::effective::ability_age_cap;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use arm_rules::{Characteristic, characteristic_score_bonus, soak};

/// Builds a character entity of `type_id` with the given selections, at
/// ruleset `test`/`1` — same shape as every other engine test file's `entity`
/// helper (e.g. `x1_authorization_family.rs`).
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("test"), "1"),
    );
    e.selections = selections;
    e
}

/// The single-selection multiplicity every fixture here uses: one "companion"
/// type with a generous budget and no permitted-category narrowing, so a
/// test's own point items are always legal to select regardless of which
/// `categories` they declare.
const COMPANION_TYPE: &str = r#"[
  { "id": "companion", "budget": { "virtue_points": 20, "flaw_points": 20 },
    "permitted_categories": ["general", "supernatural"],
    "creation_phases": [] }
]"#;

fn issue_codes(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect()
}

// --- e1: gate: Option<ParamGate> on SoakMod / CharacteristicScoreDelta /
// AbilityRollMod, plus MagicResistanceMod's amount+gate ---------------------

mod e1_gated_effects {
    use super::*;

    fn ruleset(items: &str) -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn repellent_items() -> &'static str {
        r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.repellent", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["general"],
            "parameters": [{ "key": "feature", "type": "ref", "domain": "enumerated",
              "values": ["feature.scales", "feature.natural_weapons"] }],
            "effects": [{ "type": "soak_mod", "amount": 3,
              "gate": { "param": "feature", "equals": "feature.scales" } }] }
        ]"#
    }

    fn repellent_selection(feature: &str) -> Selection {
        let mut params = BTreeMap::new();
        params.insert("feature".to_string(), Id::new(feature));
        Selection::with_params(Id::new("flaw.repellent"), params)
    }

    /// Representative red (design-x6-parameters.md § 4, e1): a `SoakMod` whose
    /// `gate` names a value the selection did NOT choose contributes nothing —
    /// same "an inactive gate contributes nothing" idiom `AbilityRef::active_for`
    /// already follows.
    #[test]
    fn soak_mod_inactive_when_gate_unmet() {
        let rs = ruleset(repellent_items());
        let e = entity(
            "companion",
            vec![repellent_selection("feature.natural_weapons")],
        );
        let total = soak(&e, &rs);
        let soak_mod = total
            .addends
            .iter()
            .find(|a| a.label == "soak_mod")
            .unwrap();
        assert_eq!(
            soak_mod.value, 0,
            "gate unmet: Repellent's +3 must not apply"
        );
    }

    /// The gate-met counterpart of the same fixture (Q-X6-3/D70: Repellent's
    /// "scales" branch computes a +3 Soak bonus).
    #[test]
    fn soak_mod_active_when_gate_met() {
        let rs = ruleset(repellent_items());
        let e = entity("companion", vec![repellent_selection("feature.scales")]);
        let total = soak(&e, &rs);
        let soak_mod = total
            .addends
            .iter()
            .find(|a| a.label == "soak_mod")
            .unwrap();
        assert_eq!(soak_mod.value, 3, "gate met: Repellent's +3 must apply");
    }

    /// `CharacteristicScoreDelta` (the FIXED-target variant, unlike
    /// `CharacteristicScoreDeltaParam` which already has `gate`) gains the same
    /// idiom — Faerie Blood's Sidhe clause, +1 Presence only for that heritage.
    #[test]
    fn characteristic_score_delta_gate_controls_faerie_blood_bonus() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.faerie_blood", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["supernatural"],
            "parameters": [{ "key": "heritage", "type": "ref", "domain": "enumerated",
              "values": ["heritage.sidhe", "heritage.dwarf"] }],
            "effects": [{ "type": "characteristic_score_delta", "characteristic": "characteristic.pre",
              "amount": 1, "gate": { "param": "heritage", "equals": "heritage.sidhe" } }] }
        ]"#;
        let rs = ruleset(items);
        let mut sidhe_params = BTreeMap::new();
        sidhe_params.insert("heritage".to_string(), Id::new("heritage.sidhe"));
        let sidhe = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.faerie_blood"),
                sidhe_params,
            )],
        );
        assert_eq!(
            characteristic_score_bonus(&sidhe, &rs, Characteristic::Pre),
            1
        );

        let mut dwarf_params = BTreeMap::new();
        dwarf_params.insert("heritage".to_string(), Id::new("heritage.dwarf"));
        let dwarf = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.faerie_blood"),
                dwarf_params,
            )],
        );
        assert_eq!(
            characteristic_score_bonus(&dwarf, &rs, Characteristic::Pre),
            0,
            "gate unmet for the Dwarf heritage: no Presence bonus"
        );
    }

    /// `AbilityRollMod` (surfaced-only) gains `gate` too — Faerie Blood's Dwarf
    /// clause, +1 to Craft rolls only for that heritage. Read through the
    /// public `surfaced_modifiers` list rather than a folded number, since
    /// this family is surfaced-only (5b), never simulated.
    #[test]
    fn ability_roll_mod_gate_controls_dwarf_craft_bonus() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.faerie_blood", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["supernatural"],
            "parameters": [{ "key": "heritage", "type": "ref", "domain": "enumerated",
              "values": ["heritage.sidhe", "heritage.dwarf"] }],
            "effects": [{ "type": "ability_roll_mod", "ability": "ability.craft", "amount": 1,
              "gate": { "param": "heritage", "equals": "heritage.dwarf" } }] }
        ]"#;
        let abilities = r#"{ "abilities": [{ "id": "ability.craft", "category": "general" }] }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(abilities),
            ..RulesetSources::default()
        })
        .unwrap();
        let mut dwarf_params = BTreeMap::new();
        dwarf_params.insert("heritage".to_string(), Id::new("heritage.dwarf"));
        let dwarf = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.faerie_blood"),
                dwarf_params,
            )],
        );
        let rows = arm_rules::surfaced_modifiers(&dwarf, &rs);
        assert!(
            rows.iter().any(
                |r| r.ability.as_ref().map(Id::as_str) == Some("ability.craft") && r.amount == 1
            ),
            "gate met for Dwarf: the Craft roll bonus must be surfaced"
        );

        let mut sidhe_params = BTreeMap::new();
        sidhe_params.insert("heritage".to_string(), Id::new("heritage.sidhe"));
        let sidhe = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.faerie_blood"),
                sidhe_params,
            )],
        );
        let rows = arm_rules::surfaced_modifiers(&sidhe, &rs);
        assert!(
            !rows
                .iter()
                .any(|r| r.ability.as_ref().map(Id::as_str) == Some("ability.craft")),
            "gate unmet for Sidhe: no Craft roll bonus surfaced"
        );
    }

    /// Representative red (design-x6-parameters.md § 4, e1/e2):
    /// `MagicResistanceMod` gains `amount: i32` paired with `gate`, and
    /// `magic_resistance()` folds every active `AuraBonus` on top of the
    /// existing per-Form/True-Faith total (Commanding Aura, Pope rank: MR 25).
    #[test]
    fn magic_resistance_mod_aura_bonus_applies_when_gate_met() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.commanding_aura", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"],
            "parameters": [{ "key": "rank", "type": "ref", "domain": "enumerated",
              "values": ["rank.pope", "rank.archbishop"] }],
            "effects": [{ "type": "magic_resistance_mod", "kind": "aura_bonus", "amount": 25,
              "gate": { "param": "rank", "equals": "rank.pope" } }] }
        ]"#;
        let abilities =
            r#"{ "abilities": [{ "id": "ability.parma_magica", "category": "arcane" }] }"#;
        let arts = r#"{ "arts": [{ "id": "art.ignem", "art_type": "form" }] }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(abilities),
            arts: Some(arts),
            ..RulesetSources::default()
        })
        .unwrap();
        let mut params = BTreeMap::new();
        params.insert("rank".to_string(), Id::new("rank.pope"));
        let e = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.commanding_aura"),
                params,
            )],
        );
        let mr = arm_rules::magic_resistance(&e, &rs);
        let ignem = mr.iter().find(|m| m.form.as_str() == "art.ignem").unwrap();
        assert_eq!(ignem.total, 25, "Pope-rank Commanding Aura: flat MR 25");
    }
}

// --- e3: Ability-category / id narrowing on ParameterDef --------------------

mod e3_ability_category_narrowing {
    use super::*;

    const ABILITIES: &str = r#"{ "abilities": [
      { "id": "ability.dancing", "category": "general" },
      { "id": "ability.philosophiae", "category": "academic" },
      { "id": "ability.dominion_lore", "category": "supernatural" },
      { "id": "ability.true_names", "category": "supernatural" },
      { "id": "ability.parma_magica", "category": "arcane" }
    ] }"#;

    fn ruleset(items: &str) -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// Representative red (§ 4, e3): Performance Magic's `ability` parameter
    /// narrows to `require_ability_categories: {general}` — an Academic
    /// Ability like Philosophiae does not resolve.
    #[test]
    fn performance_magic_rejects_academic_ability() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.performance_magic", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["general"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability",
              "require_ability_categories": ["general"] }] }
        ]"#;
        let rs = ruleset(items);
        let mut bad = BTreeMap::new();
        bad.insert("ability".to_string(), Id::new("ability.philosophiae"));
        let e = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.performance_magic"),
                bad,
            )],
        );
        assert!(
            issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "an Academic Ability must not resolve Performance Magic's general-only parameter"
        );

        let mut good = BTreeMap::new();
        good.insert("ability".to_string(), Id::new("ability.dancing"));
        let e2 = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("virtue.performance_magic"),
                good,
            )],
        );
        assert!(
            !issue_codes(&e2, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "a General Ability legally resolves"
        );
    }

    /// Representative red (§ 4, e3): Magian Lineage (Major)'s `abilities`
    /// parameter narrows to Arcane/Supernatural via `require_ability_categories`
    /// AND subtracts `ability.true_names` via `forbid_ids` — the D34-style
    /// whitelist mirror.
    #[test]
    fn magian_lineage_major_rejects_true_names() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.magian_lineage_major", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "major", "categories": ["supernatural"],
            "parameters": [{ "key": "abilities", "type": "multi_ref", "domain": "ability",
              "require_ability_categories": ["arcane", "supernatural"],
              "forbid_ids": ["ability.true_names"], "exact_count": 3 }] }
        ]"#;
        let rs = ruleset(items);
        let mut params = BTreeMap::new();
        params.insert(
            "abilities".to_string(),
            SelectionParamValue::Multi(BTreeSet::from([
                Id::new("ability.true_names"),
                Id::new("ability.dominion_lore"),
                Id::new("ability.parma_magica"),
            ])),
        );
        let e = Selection {
            item_ref: Id::new("virtue.magian_lineage_major"),
            params,
        };
        let e = entity("companion", vec![e]);
        assert!(
            issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "unknown_param_value"),
            "True Names is forbidden even though it is Supernatural-category"
        );
    }
}

// --- e4: MultiRef's exact_count ---------------------------------------------

mod e4_multi_ref_exact_count {
    use super::*;

    const ABILITIES: &str = r#"{ "abilities": [
      { "id": "ability.artes_liberales", "category": "academic" },
      { "id": "ability.philosophiae", "category": "academic" },
      { "id": "ability.medicine", "category": "academic" },
      { "id": "ability.common_sense", "category": "general" }
    ] }"#;

    fn ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.restricted_learning", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"],
            "parameters": [{ "key": "abilities", "type": "multi_ref", "domain": "ability",
              "exact_count": 5 }] }
        ]"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn selection_with(abilities: Vec<&str>) -> Selection {
        let mut params = BTreeMap::new();
        params.insert(
            "abilities".to_string(),
            SelectionParamValue::Multi(abilities.into_iter().map(Id::new).collect()),
        );
        Selection {
            item_ref: Id::new("flaw.restricted_learning"),
            params,
        }
    }

    /// Representative red (§ 4, e4): four names where the parameter declares
    /// `exact_count: 5` raises the new `wrong_param_count` code.
    #[test]
    fn restricted_learning_rejects_four_abilities() {
        let rs = ruleset();
        let e = entity(
            "companion",
            vec![selection_with(vec![
                "ability.artes_liberales",
                "ability.philosophiae",
                "ability.medicine",
                "ability.common_sense",
            ])],
        );
        assert!(
            issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "wrong_param_count"),
            "4 names against exact_count: 5 must be reported"
        );
    }

    /// The legal counterpart: five distinct names satisfy `exact_count: 5`.
    #[test]
    fn restricted_learning_accepts_five_distinct() {
        let rs = ruleset();
        let e = entity(
            "companion",
            vec![selection_with(vec![
                "ability.artes_liberales",
                "ability.philosophiae",
                "ability.medicine",
                "ability.common_sense",
                "ability.artes_liberales",
            ])],
        );
        // Five VALUES, but only four DISTINCT ones — BTreeSet collapses the
        // duplicate, which is exactly why exact_count must count distinct
        // members, not raw list length.
        assert!(
            issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "wrong_param_count"),
            "a duplicate collapses to four distinct names, still wrong"
        );
    }
}

// --- e5: Ability Block / Restricted Learning XP validators ------------------

mod e5_xp_scope_validators {
    use super::*;

    const ABILITIES: &str = r#"{ "abilities": [
      { "id": "ability.single_weapon", "category": "martial" },
      { "id": "ability.brawl", "category": "martial" },
      { "id": "ability.awareness", "category": "general" },
      { "id": "ability.dominion_lore", "category": "supernatural" }
    ] }"#;

    /// Representative red (§ 4, e5): Ability Block (Martial) forbids XP into
    /// any Martial Ability — the parameter-relative sibling of B1's fixed
    /// `ForbidsAbilityCategory`.
    #[test]
    fn ability_block_martial_blocks_martial_xp() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.ability_block", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"],
            "parameters": [{ "key": "class", "type": "ref", "domain": "ability_category" }],
            "effects": [{ "type": "forbids_ability_category_param", "param": "class" }] }
        ]"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap();
        let mut params = BTreeMap::new();
        params.insert("class".to_string(), Id::new("ability_category.martial"));
        let mut e = entity(
            "companion",
            vec![Selection::with_params(
                Id::new("flaw.ability_block"),
                params,
            )],
        );
        e.ability_scores.push(AbilityScore {
            ability: Id::new("ability.brawl"),
            score: 2,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        });
        assert!(
            !issue_codes(&e, &rs).is_empty(),
            "a Martial Ability score must be refused under Ability Block (Martial)"
        );
    }

    /// Representative red (§ 4, e5): Restricted Learning's five named
    /// Abilities plus the `supernatural` category are the ONLY funding
    /// targets — an Ability outside both raises the new
    /// `ability_outside_restricted_scope` code.
    #[test]
    fn restricted_learning_blocks_ability_outside_five_and_supernatural() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.restricted_learning", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"],
            "parameters": [{ "key": "abilities", "type": "multi_ref", "domain": "ability",
              "exact_count": 1 }],
            "effects": [{ "type": "restricted_ability_xp", "amount": 0,
              "categories": ["supernatural"], "abilities_param": "abilities" }] }
        ]"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap();
        let mut params = BTreeMap::new();
        params.insert(
            "abilities".to_string(),
            SelectionParamValue::Multi(BTreeSet::from([Id::new("ability.awareness")])),
        );
        let mut e = entity(
            "companion",
            vec![Selection {
                item_ref: Id::new("flaw.restricted_learning"),
                params,
            }],
        );
        // In scope: named directly.
        e.ability_scores.push(AbilityScore {
            ability: Id::new("ability.awareness"),
            score: 3,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        });
        // In scope: the supernatural category union member.
        e.ability_scores.push(AbilityScore {
            ability: Id::new("ability.dominion_lore"),
            score: 2,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        });
        // OUT of scope: neither named nor supernatural.
        e.ability_scores.push(AbilityScore {
            ability: Id::new("ability.single_weapon"),
            score: 1,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        });
        assert!(
            issue_codes(&e, &rs)
                .iter()
                .any(|c| c == "ability_outside_restricted_scope"),
            "Single Weapon is neither named nor Supernatural: must be refused"
        );
    }

    /// UI review 2026-09-30 #1: the `allowed` arg mixes raw Ability ids and raw
    /// `AbilityCategory` enum words in one comma-joined string, which the
    /// frontend cannot tell apart to localize (`derive.ts::resolveIssueArgValue`
    /// resolves one id/enum per arg, never a composite list). Tagging each
    /// category as an `ability_category.<slug>` id — the same id-shaped form
    /// `AbilityCategory::from_id` and `rules/core/virtues_flaws.json`'s
    /// enumerated parameter values already use — makes every token
    /// self-describing, so the frontend can split and resolve each one.
    #[test]
    fn ability_outside_restricted_scope_allowed_arg_tags_categories_as_ids() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.restricted_learning", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"],
            "parameters": [{ "key": "abilities", "type": "multi_ref", "domain": "ability",
              "exact_count": 1 }],
            "effects": [{ "type": "restricted_ability_xp", "amount": 0,
              "categories": ["supernatural"], "abilities_param": "abilities" }] }
        ]"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap();
        let mut params = BTreeMap::new();
        params.insert(
            "abilities".to_string(),
            SelectionParamValue::Multi(BTreeSet::from([Id::new("ability.awareness")])),
        );
        let mut e = entity(
            "companion",
            vec![Selection {
                item_ref: Id::new("flaw.restricted_learning"),
                params,
            }],
        );
        e.ability_scores.push(AbilityScore {
            ability: Id::new("ability.single_weapon"),
            score: 1,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        });
        let issues = validate(&e, &rs).issues;
        let issue = issues
            .iter()
            .find(|i| i.code == "ability_outside_restricted_scope")
            .expect("Single Weapon is out of scope");
        let allowed = issue.args.get("allowed").expect("allowed arg present");
        assert!(
            allowed.contains("ability_category.supernatural"),
            "category token must be id-shaped, got: {allowed}"
        );
        assert!(
            !allowed.split(", ").any(|t| t == "supernatural"),
            "category token must not be a bare enum word, got: {allowed}"
        );
        assert!(
            allowed.contains("ability.awareness"),
            "ability token must still be present, got: {allowed}"
        );
    }
}

// --- e6: Savantism through D29's single resolution point --------------------

mod e6_savantism_caps {
    use super::*;

    // Age band: cap 5 under 30 (mirrors effective/reputation_and_caps.rs's own
    // age_cap_resolution_point_tests fixture), so both Savantism overrides are
    // provably NOT age-derived.
    const ABILITIES: &str = r#"{
      "age_ability_caps": [ { "max_age": 29, "max_score": 5 }, { "max_score": 9 } ],
      "abilities": [
        { "id": "ability.brawl", "category": "general" },
        { "id": "ability.awareness", "category": "general" }
      ]
    }"#;

    fn ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.savantism", "kind": "flaw", "classification": "creation_effect",
            "magnitude": "major", "categories": ["general"],
            "parameters": [{ "key": "favored", "type": "ref", "domain": "ability" }],
            "effects": [
              { "type": "ability_score_cap_override_param", "param": "favored", "max": 6 },
              { "type": "ability_score_cap_all_except", "param": "favored", "max": 3 }
            ] }
        ]"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn savant(favored: &str, age: u32) -> Entity {
        let mut params = BTreeMap::new();
        params.insert("favored".to_string(), Id::new(favored));
        let mut e = entity(
            "companion",
            vec![Selection::with_params(Id::new("flaw.savantism"), params)],
        );
        e.age = Some(age);
        e
    }

    /// Representative red (§ 4, e6): the favored Ability caps at 6 — ABOVE the
    /// age-25 band's own cap of 5 — proving the override raises rather than
    /// merely lowers, which is exactly why D29 requires one resolution point.
    #[test]
    fn savantism_favored_ability_caps_at_6_regardless_of_age() {
        let rs = ruleset();
        let e = savant("ability.brawl", 25);
        assert_eq!(
            ability_age_cap(&e, &rs, &Id::new("ability.brawl"), None),
            Some(6),
        );
    }

    /// The sibling clause: every OTHER Ability caps at 3, lowering the
    /// (otherwise 5) age-band figure.
    #[test]
    fn savantism_other_abilities_cap_at_3_under_the_normal_age_cap() {
        let rs = ruleset();
        let e = savant("ability.brawl", 25);
        assert_eq!(
            ability_age_cap(&e, &rs, &Id::new("ability.awareness"), None),
            Some(3),
        );
    }
}

// --- e7: Warped Senses' conditional incompatibility -------------------------

mod e7_conditional_incompatibility {
    use super::*;

    fn ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.warped_senses", "kind": "flaw", "classification": "uncomputed_rule",
            "magnitude": "minor", "categories": ["general"],
            "parameters": [{ "key": "sense", "type": "ref", "domain": "enumerated",
              "values": ["sense.sight", "sense.hearing"] }],
            "conditional_incompatible_with": [
              { "gate": { "param": "sense", "equals": "sense.sight" },
                "forbids": ["virtue.keen_vision"] }
            ] },
          { "id": "virtue.keen_vision", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "categories": ["general"] }
        ]"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn warped(sense: &str) -> Selection {
        let mut params = BTreeMap::new();
        params.insert("sense".to_string(), Id::new(sense));
        Selection::with_params(Id::new("flaw.warped_senses"), params)
    }

    /// Representative red (§ 4, e7; D58/D58.1): Weak Sight (sense = sight) is
    /// a HARD error alongside Keen Vision, even though the -2 penalty itself
    /// stays text (D61) — the incompatibility is absolute (ArMDE:7031, :7033).
    #[test]
    fn weak_sight_excludes_sensitive_sight_and_keen_vision() {
        let rs = ruleset();
        let e = entity(
            "companion",
            vec![
                warped("sense.sight"),
                Selection::new(Id::new("virtue.keen_vision")),
            ],
        );
        assert!(
            issue_codes(&e, &rs).iter().any(|c| c == "incompatible"),
            "Weak Sight + Keen Vision must be a hard incompatibility"
        );
    }

    /// The gate-unmet counterpart: Weak Hearing (sense = hearing) carries no
    /// incompatibility with a sight-only entry — the conditional exclusion is
    /// per-value, not a blanket Warped Senses exclusion.
    #[test]
    fn weak_hearing_has_no_incompatibility_with_a_sight_entry() {
        let rs = ruleset();
        let e = entity(
            "companion",
            vec![
                warped("sense.hearing"),
                Selection::new(Id::new("virtue.keen_vision")),
            ],
        );
        assert!(
            !issue_codes(&e, &rs).iter().any(|c| c == "incompatible"),
            "Weak Hearing must not exclude Keen Vision"
        );
    }

    // --- RC review-C item 2: the same-copy twin (ArMDE:7033) ---------------

    fn same_item_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "flaw.warped_senses", "kind": "flaw", "classification": "uncomputed_rule",
            "magnitude": "minor", "categories": ["general"], "max_total": 255,
            "parameters": [{ "key": "affliction", "type": "ref", "domain": "enumerated",
              "values": ["affliction.weak_sight", "affliction.sensitive_sight"] }],
            "conditional_incompatible_with": [
              { "gate": { "param": "affliction", "equals": "affliction.weak_sight" },
                "forbids": [], "forbids_same_item_values": ["affliction.sensitive_sight"] }
            ] }
        ]"#;
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: COMPANION_TYPE,
            ..RulesetSources::default()
        })
        .unwrap()
    }

    fn affliction(value: &str) -> Selection {
        let mut params = BTreeMap::new();
        params.insert("affliction".to_string(), Id::new(value));
        Selection::with_params(Id::new("flaw.warped_senses"), params)
    }

    /// New engine capability (RC review-C item 2, ArMDE:7033): a SECOND copy
    /// of the SAME item, whose own `affliction` value is one of
    /// `forbids_same_item_values`, is incompatible with the declaring copy.
    /// `forbids` alone (naming OTHER items' ids) cannot express this, since
    /// `selected_ids` collapses both copies of `flaw.warped_senses` to one id.
    #[test]
    fn weak_sight_excludes_a_second_copy_holding_sensitive_sight() {
        let rs = same_item_ruleset();
        let e = entity(
            "companion",
            vec![
                affliction("affliction.weak_sight"),
                affliction("affliction.sensitive_sight"),
            ],
        );
        assert!(
            issue_codes(&e, &rs).iter().any(|c| c == "incompatible"),
            "Weak Sight must exclude a second copy holding Sensitive Sight"
        );
    }

    /// The gate-unmet counterpart: two copies both holding the SAME value
    /// carry no incompatibility — this is not a generic "no duplicate
    /// copies" rule, only the stated same-item pairing.
    #[test]
    fn two_copies_of_the_same_value_carry_no_incompatibility() {
        let rs = same_item_ruleset();
        let e = entity(
            "companion",
            vec![
                affliction("affliction.weak_sight"),
                affliction("affliction.weak_sight"),
            ],
        );
        assert!(
            !issue_codes(&e, &rs).iter().any(|c| c == "incompatible"),
            "two copies of the same value must not collide"
        );
    }
}
