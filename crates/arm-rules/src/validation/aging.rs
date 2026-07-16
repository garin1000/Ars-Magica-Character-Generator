//! Aging and its effect on characteristic and ability scores.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Validates a directly-entered aged character's aging state (advisory). Aging
/// drops are DERIVED from [`Entity::aging_points`] (Core Rules.md:16579); this
/// only surfaces informational notes, never blocking errors:
///
/// - `aging_points_force_drop`: a Characteristic's accrued points have forced one
///   or more drops, which the engine has already applied to its effective score.
///   An informational note (the drop is automatic, not an entry mistake).
/// - `excessive_aging_reduction`: the derived drops would push a Characteristic's
///   effective score below the rules effective minimum (−5). The derived score is
///   clamped regardless; this only flags an implausible entry.
///
/// Reads the un-aged bought score plus the derived drops; it never touches the
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

    let effective_min = ruleset
        .characteristic_rules()
        .and_then(|r| r.effective_min_score())
        .map(i32::from);

    for (characteristic, points) in &entity.aging_points {
        if *points == 0 {
            continue;
        }
        let drops = crate::effective::aging_drops(entity, *characteristic);
        if drops == 0 {
            continue;
        }
        let aged = bought(characteristic) - i32::try_from(drops).unwrap_or(i32::MAX);

        // Informational: the accrued points have auto-applied one or more drops.
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_AGING_POINTS_FORCE_DROP,
            args([
                ("characteristic", characteristic.to_string()),
                ("points", points.to_string()),
                ("drops", drops.to_string()),
                ("score", aged.to_string()),
            ]),
            None,
        ));

        // The aged-down score would fall below the rules floor (clamped anyway).
        if let Some(min) = effective_min
            && aged < min
        {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("reduction", drops.to_string()),
                    ("min", min.to_string()),
                ]),
                None,
            ));
        }
    }
}
