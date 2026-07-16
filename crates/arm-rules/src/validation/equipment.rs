//! Mundane equipment constraints.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Validates the character's carried equipment. Each [`EquipmentSlot`] must name a
/// catalogue weapon, shield, or armor id (`unknown_equipment`, error). For an
/// **equipped** weapon or shield whose minimum-Strength requirement exceeds the
/// character's (aged) Strength, an advisory `equipment_min_strength` warning is
/// raised — never blocking, since carrying/wielding an over-heavy weapon is a
/// storyguide call, not an illegal creation state (Core:16993). Armor carries no
/// minimum-Strength requirement. Combat totals, Soak, and Encumbrance are computed
/// downstream (slice 5i), not here.
pub(crate) fn validate_equipment(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let strength = crate::effective::effective_characteristic_after_aging(
        entity,
        ruleset,
        Characteristic::Str,
    );
    for slot in &entity.equipment {
        let item = &slot.item;
        // Resolve the id against exactly one of the three catalogues, and (for an
        // equipped weapon/shield) capture its min-Strength for the advisory.
        let min_strength: Option<i32> = if let Some(weapon) = ruleset.weapon(item) {
            weapon.min_strength.map(i32::from)
        } else if let Some(shield) = ruleset.shield(item) {
            Some(i32::from(shield.min_strength))
        } else if ruleset.armor_item(item).is_some() {
            None
        } else {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_EQUIPMENT,
                args([("item", item.to_string())]),
                Some(item.clone()),
            ));
            continue;
        };

        if slot.equipped
            && let Some(required) = min_strength
            && required > strength
        {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_EQUIPMENT_MIN_STRENGTH,
                args([
                    ("item", item.to_string()),
                    ("required", required.to_string()),
                    ("strength", strength.to_string()),
                ]),
                Some(item.clone()),
            ));
        }
    }
}
