//! Warping: the off-budget Virtues/Flaws a non-magus character owes from its
//! Warping Score ("Effects of Warping", Ars Magica - Definitive Edition (Core Rules).md:16547-16561).
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and the
//! `ValidationIssue` issue-code contract.

use super::*;
use crate::effective::{
    WARPING_MAJOR_FLAW_KEY, WARPING_MINOR_FLAW_KEY, WARPING_SUPERNATURAL_VIRTUE_KEY,
    item_carries_warping_grant, warping_owed, warping_owed_grants,
};

/// Validates a non-magus character's chosen fills for the Virtues/Flaws it owes
/// from Warping. Mirrors [`validate_house`]/[`validate_mythic_type`] — the owed
/// grants are OPEN grants ([`warping_owed_grants`]) whose picks live in
/// `entity.warping_choices`, resolved off the creation V/F budget.
///
/// - Hermetic magi are exempt (Warping gives them Wizard's Twilight instead,
///   Ars Magica - Definitive Edition (Core Rules).md:16551) → the section is skipped entirely, owing zero.
/// - An unfilled owed slot → a non-blocking advisory per kind
///   (`warping_owed_minor_flaws` / `_supernatural_virtues` / `_major_flaws`) so a
///   still-incomplete build is flagged, never hard-blocked.
/// - A chosen fill keyed to a slot the character does not owe → `warping_fill_excess`.
/// - A chosen fill carrying [`crate::types::Effect::WarpingGrant`] →
///   `warping_fill_ineligible` (the recursion guard: it must never re-feed the
///   Warping Score that decides how many V/F are owed).
/// - A chosen fill of the wrong kind/magnitude/category, or an unresolvable id →
///   `warping_fill_constraint`.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16547-16561.
pub(crate) fn validate_warping(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    // Warping-owed V/F are a non-magus concern; magi are exempt (Twilight), and an
    // unknown type cannot be checked. `warping_owed` also returns zero for magi, so
    // this gate mainly avoids emitting advisories for them.
    let Some(profile) = type_profile else {
        return;
    };
    if profile.is_magus {
        return;
    }

    let grants = warping_owed_grants(entity, ruleset);
    // The stable choice_key of every slot the character owes.
    // `warping_owed_grants` emits only `Grant::Open`, so the non-Open arms are
    // unreachable by construction; they are listed explicitly (rather than a `_`
    // wildcard) to stay exhaustive over the closed `Grant` set.
    let owed_keys: BTreeSet<&str> = grants
        .iter()
        .filter_map(|grant| match grant {
            Grant::Open { choice_key, .. } => Some(choice_key.as_str()),
            Grant::Fixed { .. } | Grant::Choice { .. } => None,
        })
        .collect();

    // A stored fill keyed to a slot the character does not owe exceeds the owed
    // count (e.g. the score dropped after the pick was made).
    for choice_key in entity.warping_choices.keys() {
        if !owed_keys.contains(choice_key.as_str()) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_WARPING_FILL_EXCESS,
                CreationPhase::Review,
                args([("choice_key", choice_key.clone())]),
                None,
            ));
        }
    }

    // Each owed slot with a pick: reject a WarpingGrant-carrying item (recursion
    // guard) or a pick that violates the slot's constraint.
    for grant in &grants {
        let Grant::Open {
            choice_key,
            constraint,
        } = grant
        else {
            continue;
        };
        let Some(pick) = entity.warping_choices.get(choice_key) else {
            continue;
        };
        if item_carries_warping_grant(&pick.item_ref, ruleset) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_WARPING_FILL_INELIGIBLE,
                CreationPhase::Review,
                args([
                    ("choice_key", choice_key.clone()),
                    ("item", pick.item_ref.to_string()),
                ]),
                Some(pick.item_ref.clone()),
            ));
            continue;
        }
        if !open_pick_satisfies(pick, constraint, ruleset) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_WARPING_FILL_CONSTRAINT,
                CreationPhase::Review,
                args([
                    ("choice_key", choice_key.clone()),
                    ("item", pick.item_ref.to_string()),
                ]),
                Some(pick.item_ref.clone()),
            ));
        }
        // A parameterized fill ("Enchanting (Ability)") is only really chosen once
        // its parameter names a target, so the pick gets the same parameter checks
        // a bought selection gets.
        // A Warping fill lives on no creation step, so its parameter problems are
        // fixable only in the finished character.
        validate_selection_parameters(pick, ruleset, CreationPhase::Review, issues);
    }

    // Advisory: still owe more than chosen, per kind. A slot counts as filled once
    // it has any pick (constraint validity is handled by the errors above).
    let owed = warping_owed(entity, ruleset);
    let unfilled = |prefix: &str, count: u8| -> u8 {
        (0..count)
            .filter(|i| !entity.warping_choices.contains_key(&format!("{prefix}{i}")))
            .count() as u8
    };

    let remaining_minor_flaws = unfilled(WARPING_MINOR_FLAW_KEY, owed.minor_flaws);
    if remaining_minor_flaws > 0 {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_WARPING_OWED_MINOR_FLAWS,
            CreationPhase::Review,
            args([("count", remaining_minor_flaws.to_string())]),
            None,
        ));
    }
    let remaining_virtues = unfilled(
        WARPING_SUPERNATURAL_VIRTUE_KEY,
        owed.minor_supernatural_virtues,
    );
    if remaining_virtues > 0 {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_WARPING_OWED_SUPERNATURAL_VIRTUES,
            CreationPhase::Review,
            args([("count", remaining_virtues.to_string())]),
            None,
        ));
    }
    let remaining_major_flaws = unfilled(WARPING_MAJOR_FLAW_KEY, owed.major_flaws);
    if remaining_major_flaws > 0 {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_WARPING_OWED_MAJOR_FLAWS,
            CreationPhase::Review,
            args([("count", remaining_major_flaws.to_string())]),
            None,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntityKind, RulesetRef, Selection};
    use crate::validation::{compute_balance, validate};
    use pretty_assertions::assert_eq;

    /// A companion-only ruleset with a Minor Flaw, a Major Flaw, a supernatural
    /// Minor Virtue, and the `WarpingGrant`-carrying `warped_by_magic` (also the
    /// score driver). No Arts → the magus-role integrity gate is skipped.
    fn warping_ruleset() -> Ruleset {
        let items = r#"[
          { "id": "virtue.the_gift", "kind": "virtue", "classification": "narrative",
            "magnitude": "free", "category": "special", "entity_kinds": ["character"] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "personality", "entity_kinds": ["character"] },
          { "id": "flaw.clumsy", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "general", "entity_kinds": ["character"] },
          { "id": "flaw.dark_secret", "kind": "flaw", "classification": "narrative",
            "magnitude": "major", "category": "story", "entity_kinds": ["character"] },
          { "id": "virtue.second_sight", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "category": "supernatural", "entity_kinds": ["character"] },
          { "id": "virtue.enchanting_ability", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "category": "supernatural", "entity_kinds": ["character"],
            "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }] },
          { "id": "flaw.warped_by_magic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "supernatural", "entity_kinds": ["character"],
            "effects": [{ "type": "warping_grant", "score": 1, "points": 5 }] }
        ]"#;
        let types = r#"[
          { "id": "companion", "budget": { "virtue_points": 3, "flaw_points": 3 },
            "permitted_categories": ["special", "personality", "general", "story", "supernatural"],
            "creation_phases": [] }
        ]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }, { "score": 6, "total_xp": 105 },
            { "score": 7, "total_xp": 140 }
          ],
          "abilities": [{ "id": "ability.awareness", "category": "general" }]
        }"#;
        Ruleset::from_json_with_abilities("arm5-core", "2024.1", items, types, abilities).unwrap()
    }

    fn companion(points: u32) -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        e.warping_points = points;
        e
    }

    fn fill(e: &mut Entity, key: &str, item: &str) {
        e.warping_choices
            .insert(key.to_string(), Selection::new(Id::new(item)));
    }

    fn fill_with_params(e: &mut Entity, key: &str, item: &str, params: &[(&str, &str)]) {
        let params = params
            .iter()
            .map(|(k, v)| ((*k).to_string(), Id::new(*v)))
            .collect();
        e.warping_choices.insert(
            key.to_string(),
            Selection::with_params(Id::new(item), params),
        );
    }

    fn codes(result: &crate::validation::ValidationResult) -> Vec<String> {
        result.issues.iter().map(|i| i.code.clone()).collect()
    }

    #[test]
    fn owing_a_minor_flaw_with_no_fill_warns() {
        let rs = warping_ruleset();
        let e = companion(5); // Warping Score 1 → owes one Minor Flaw.
        let result = validate(&e, &rs);
        assert!(
            codes(&result).contains(&"warping_owed_minor_flaws".to_string()),
            "an unfilled owed Minor Flaw should warn: {:?}",
            result.issues
        );
    }

    #[test]
    fn owing_a_supernatural_virtue_with_no_fill_warns() {
        let rs = warping_ruleset();
        // Warping Score 5 (75 pts) → owes a supernatural Minor Virtue (:16559).
        let e = companion(75);
        let result = validate(&e, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == "warping_owed_supernatural_virtues")
            .expect("an unfilled owed supernatural Virtue should warn");
        assert_eq!(
            issue.args.get("count").map(String::as_str),
            Some("1"),
            "the advisory should report the one owed supernatural Virtue"
        );
    }

    #[test]
    fn owing_a_major_flaw_with_no_fill_warns() {
        let rs = warping_ruleset();
        // Warping Score 6 (105 pts) → owes a Major Flaw beyond the Score-5 set
        // (:16561: a Major Flaw at Score 6 and every point thereafter).
        let e = companion(105);
        let result = validate(&e, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == "warping_owed_major_flaws")
            .expect("an unfilled owed Major Flaw should warn");
        assert_eq!(
            issue.args.get("count").map(String::as_str),
            Some("1"),
            "the advisory should report the one owed Major Flaw"
        );
    }

    #[test]
    fn a_valid_minor_flaw_fill_clears_the_advisory() {
        let rs = warping_ruleset();
        let mut e = companion(5);
        fill(&mut e, "warping.minor_flaw.0", "flaw.clumsy");
        let result = validate(&e, &rs);
        assert!(
            !codes(&result).contains(&"warping_owed_minor_flaws".to_string()),
            "a filled owed slot must not warn: {:?}",
            result.issues
        );
    }

    #[test]
    fn wrong_magnitude_fill_is_an_error() {
        let rs = warping_ruleset();
        let mut e = companion(5); // owes a Minor Flaw
        // A Major Flaw where a Minor is owed.
        fill(&mut e, "warping.minor_flaw.0", "flaw.dark_secret");
        let result = validate(&e, &rs);
        assert!(
            codes(&result).contains(&"warping_fill_constraint".to_string()),
            "a wrong-magnitude fill should error: {:?}",
            result.issues
        );
    }

    #[test]
    fn unknown_fill_id_is_a_constraint_error() {
        let rs = warping_ruleset();
        let mut e = companion(5);
        fill(&mut e, "warping.minor_flaw.0", "flaw.does_not_exist");
        let result = validate(&e, &rs);
        assert!(
            codes(&result).contains(&"warping_fill_constraint".to_string()),
            "an unresolvable fill id should error: {:?}",
            result.issues
        );
    }

    #[test]
    fn warping_grant_item_as_a_fill_is_ineligible() {
        let rs = warping_ruleset();
        let mut e = companion(5);
        // warped_by_magic satisfies (Minor Flaw) but carries WarpingGrant — the
        // recursion guard rejects it.
        fill(&mut e, "warping.minor_flaw.0", "flaw.warped_by_magic");
        let result = validate(&e, &rs);
        assert!(
            codes(&result).contains(&"warping_fill_ineligible".to_string()),
            "a WarpingGrant fill should be ineligible: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_fill_keyed_to_an_unowed_slot_is_excess() {
        let rs = warping_ruleset();
        let mut e = companion(0); // owes nothing
        fill(&mut e, "warping.minor_flaw.0", "flaw.clumsy");
        let result = validate(&e, &rs);
        assert!(
            codes(&result).contains(&"warping_fill_excess".to_string()),
            "a fill with nothing owed should be excess: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_chosen_owed_flaw_fill_is_off_budget() {
        // The companion's flaw budget is 3 (one Minor). A warping-owed Minor Flaw
        // fill must NOT spend from that budget — compute_balance reads bought
        // selections only, and no over_budget_flaws fires.
        let rs = warping_ruleset();
        let mut e = companion(5);
        fill(&mut e, "warping.minor_flaw.0", "flaw.clumsy");
        assert_eq!(
            compute_balance(&e, &rs).flaw_points,
            0,
            "an owed warping fill must not spend the flaw budget"
        );
        assert!(
            !codes(&validate(&e, &rs)).contains(&"over_budget_flaws".to_string()),
            "an owed warping fill must not trip the flaw budget"
        );
    }

    /// A parameterized fill ("Enchanting (Ability)") needs its parameter chosen:
    /// the owed slot is only really filled once the target is named, so the same
    /// `missing_param` check bought selections get applies to the pick.
    #[test]
    fn a_parameterized_owed_fill_missing_its_param_errors() {
        let rs = warping_ruleset();
        // Warping Score 5 (75 pts) → owes a supernatural Minor Virtue slot.
        let mut e = companion(75);
        fill(
            &mut e,
            "warping.supernatural_virtue.0",
            "virtue.enchanting_ability",
        );
        let result = validate(&e, &rs);
        assert!(
            codes(&result).contains(&ValidationIssue::CODE_MISSING_PARAM.to_string()),
            "a parameterized owed fill with no param should error missing_param: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_parameterized_owed_fill_with_unknown_param_value_errors() {
        let rs = warping_ruleset();
        let mut e = companion(75);
        fill_with_params(
            &mut e,
            "warping.supernatural_virtue.0",
            "virtue.enchanting_ability",
            &[("ability", "ability.nope")],
        );
        let result = validate(&e, &rs);
        assert!(
            codes(&result).contains(&ValidationIssue::CODE_UNKNOWN_PARAM_VALUE.to_string()),
            "an owed fill whose param value is not in the catalogue should error: {:?}",
            result.issues
        );
    }

    #[test]
    fn a_parameterized_owed_fill_with_resolving_param_is_clean() {
        let rs = warping_ruleset();
        let mut e = companion(75);
        fill_with_params(
            &mut e,
            "warping.supernatural_virtue.0",
            "virtue.enchanting_ability",
            &[("ability", "ability.awareness")],
        );
        let result = validate(&e, &rs);
        let param_codes: Vec<&String> = result
            .issues
            .iter()
            .map(|i| &i.code)
            .filter(|c| c.ends_with("_param") || c.as_str() == "unknown_param_value")
            .collect();
        assert!(
            param_codes.is_empty(),
            "a resolving param on an owed fill should raise no parameter issue: {param_codes:?}"
        );
    }

    #[test]
    fn a_fully_filled_owed_set_is_clean() {
        let rs = warping_ruleset();
        // Warping Score 5 (75 pts) → owes 2 Minor Flaws + 1 supernatural Minor
        // Virtue; fill them all with legal, eligible picks.
        let mut e = companion(75);
        fill(&mut e, "warping.minor_flaw.0", "flaw.clumsy");
        fill(&mut e, "warping.minor_flaw.1", "flaw.clumsy");
        fill(
            &mut e,
            "warping.supernatural_virtue.0",
            "virtue.second_sight",
        );
        let result = validate(&e, &rs);
        let warping_codes: Vec<&String> = result
            .issues
            .iter()
            .map(|i| &i.code)
            .filter(|c| c.starts_with("warping_"))
            .collect();
        assert!(
            warping_codes.is_empty(),
            "a fully, legally filled owed set should raise no warping issues: {warping_codes:?}"
        );
    }
}
