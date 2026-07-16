//! Aging and its effect on characteristic and ability scores.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Validates a directly-entered aged character's aging state (advisory). Aging is
/// derived by the guided flow in M6; M5 only makes the raw state enterable, so both
/// findings here are **warnings**, never blocking:
///
/// - `excessive_aging_reduction`: a Characteristic's completed drops
///   ([`Entity::aging_reductions`]) would push its effective score below the rules
///   effective minimum (−5). The derived score is clamped regardless; this only
///   flags an implausible entry.
/// - `aging_points_force_drop`: a Characteristic's accrued points
///   ([`Entity::aging_points`]) exceed the magnitude of its aged-down score, which
///   per the rules would already have forced a drop and reset. Kept non-blocking
///   because a character may be entered mid-accrual.
///
/// Reads the un-aged bought score plus the reductions; it never touches the
/// point-buy budget check (which is what keeps aging from perturbing creation
/// legality). Source: Core Rules.md:16579.
pub(crate) fn validate_aging(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let bought = |c: &Characteristic| {
        entity
            .characteristics
            .get(c)
            .copied()
            .map_or(0i32, i32::from)
    };
    let reduction = |c: &Characteristic| {
        entity
            .aging_reductions
            .get(c)
            .copied()
            .map_or(0i32, i32::from)
    };

    let effective_min = ruleset
        .characteristic_rules()
        .and_then(|r| r.effective_min_score())
        .map(i32::from);

    if let Some(min) = effective_min {
        for (characteristic, drop) in &entity.aging_reductions {
            if *drop == 0 {
                continue;
            }
            if bought(characteristic) - i32::from(*drop) < min {
                issues.push(ValidationIssue::warning(
                    ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION,
                    args([
                        ("characteristic", characteristic.to_string()),
                        ("reduction", drop.to_string()),
                        ("min", min.to_string()),
                    ]),
                    None,
                ));
            }
        }
    }

    for (characteristic, points) in &entity.aging_points {
        if *points == 0 {
            continue;
        }
        let aged = bought(characteristic) - reduction(characteristic);
        if u32::from(*points) > aged.unsigned_abs() {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_AGING_POINTS_FORCE_DROP,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("points", points.to_string()),
                    ("score", aged.to_string()),
                ]),
                None,
            ));
        }
    }
}
