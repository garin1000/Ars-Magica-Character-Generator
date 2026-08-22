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
/// storyguide call, not an illegal creation state (Ars Magica - Definitive Edition (Core Rules).md:16997). Armor carries no
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
                CreationPhase::Review,
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
                CreationPhase::Review,
                args([
                    ("item", item.to_string()),
                    ("required", required.to_string()),
                    ("strength", strength.to_string()),
                ]),
                Some(item.clone()),
            ));
        }
    }

    warn_shield_with_two_handed_weapon(entity, ruleset, issues);
}

/// Advisory: a shield equipped alongside **only** two-handed weapon(s) has its
/// combat modifiers silently dropped (a two-handed weapon cannot be paired with a
/// shield, Ars Magica - Definitive Edition (Core Rules).md:7494), which looks like a bug. Raised only when at least one
/// shield and at least one weapon are equipped and **no** equipped weapon is
/// one-handed — a one-handed weapon makes the shield usable, so no advisory.
/// Non-blocking: the shield still counts toward Load (Ars Magica - Definitive Edition (Core Rules).md:17063, :16975).
fn warn_shield_with_two_handed_weapon(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let equipped_shield = entity
        .equipment
        .iter()
        .any(|s| s.equipped && ruleset.shield(&s.item).is_some());
    if !equipped_shield {
        return;
    }
    let equipped_weapons: Vec<_> = entity
        .equipment
        .iter()
        .filter(|s| s.equipped)
        .filter_map(|s| ruleset.weapon(&s.item))
        .collect();
    let all_two_handed =
        !equipped_weapons.is_empty() && equipped_weapons.iter().all(|w| w.two_handed);
    if all_two_handed {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_SHIELD_WITH_TWO_HANDED_WEAPON,
            CreationPhase::Review,
            args([]),
            None,
        ));
    }
}
