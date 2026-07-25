//! Supernatural-being Might, powers, and invested devices.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Sanity-checks a being's Might: its entered base Realm must agree with the Realm
/// its Might Virtues grant (a being belongs to exactly one Realm; Core:2623-2625).
/// A warning, never a block — the troupe may be modelling an unusual creature.
pub(crate) fn validate_might(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(base) = entity.might else {
        return;
    };
    let Some(granted) = ruleset_might_grant_realm(entity, ruleset) else {
        return;
    };
    if granted != base.realm {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_MIGHT_REALM_MISMATCH,
            args([
                ("base", base.realm.to_string()),
                ("granted", granted.to_string()),
            ]),
            None,
        ));
    }
}

/// The Realm of the first [`Effect::MightGrant`] the being's Virtues confer, if
/// any. Used only for the [`validate_might`] realm-agreement sanity check.
fn ruleset_might_grant_realm(entity: &Entity, ruleset: &Ruleset) -> Option<crate::types::Realm> {
    for selection in crate::effective::selections_for_effects(entity, ruleset).iter() {
        // A dangling selection ref (legal in direct/unchecked entry) must not
        // abort the whole scan — skip it, mirroring the sibling validators
        // (`effective::spell_mastery_floor` et al.).
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::MightGrant { realm, .. } = effect {
                return Some(*realm);
            }
        }
    }
    None
}

/// Validates a supernatural being's powers: the total power level may not exceed
/// the power-levels budget its Might Virtues grant (Demonic Blood 30, Demonic
/// Powers +20). A power on a being with no granting Virtue (budget 0) is flagged,
/// mirroring [`validate_devices`]. Source: RoP:Infernal:4122, :4142.
pub(crate) fn validate_powers(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let used = crate::effective::powers_used(entity);
    let budget = crate::effective::power_levels_budget(entity, ruleset);
    if used > budget {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_POWER_LEVELS,
            args([
                ("used", used.to_string()),
                ("budget", budget.to_string()),
                ("over", (used - budget).to_string()),
            ]),
            None,
        ));
    }
}

/// Validates a character's starting enchanted devices: the total device level may
/// not exceed the item-level budget the character's Virtues grant (Magic Items
/// +25, Redcap 50). A device requires budget, so a device on a character with no
/// granting Virtue (budget 0) is flagged — consistent with how a starting
/// Reputation requires a granting Virtue. Source: Core Rules.md:4347-4349,
/// :4842-4846.
pub(crate) fn validate_devices(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let used = crate::effective::item_level_used(entity);
    let budget = crate::effective::item_level_budget(entity, ruleset);
    if used > budget {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_ITEM_LEVEL,
            args([
                ("used", used.to_string()),
                ("budget", budget.to_string()),
                ("over", (used - budget).to_string()),
            ]),
            None,
        ));
    }
}
