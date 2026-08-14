//! Aging and its effect on characteristic and ability scores.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Validates a directly-entered aged character's aging state (advisory). Aging
/// drops are DERIVED from [`Entity::aging_points`] (Core Rules.md:16579); this
/// only surfaces informational notes, never blocking errors:
///
/// - `excessive_aging_reduction`: the derived drops would push a Characteristic's
///   effective score below the rules effective minimum (−5). The derived score is
///   clamped regardless; this only flags an implausible entry.
/// - `life_stage_aging_rolls_pending`: the character is over 35 and no aging roll
///   is recorded, so the rolls the rules owe before play have not been made.
///
/// (An earlier `aging_points_force_drop` note announcing each auto-applied drop
/// was removed as validation noise — the drop is automatic and already reflected
/// in the effective score, so it is not an entry problem worth flagging.)
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

        // The aged-down score would fall below the rules floor (clamped anyway).
        if let Some(min) = effective_min
            && aged < min
        {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION,
                CreationPhase::Review,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("reduction", drops.to_string()),
                    ("min", min.to_string()),
                ]),
                None,
            ));
        }
    }

    report_pending_aging_rolls(entity, issues);
}

/// The age past which the rules owe aging rolls: "Characters begin aging in the
/// Winter after they turn 35."
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16565.
///
/// A cited constant in Rust rather than rules data because there is no
/// `rules/core/aging.json` yet; slice 6b6 introduces that file and this threshold
/// moves into it (see RULES.md → *Hardcoded engine values*).
const AGING_ROLLS_START_AGE: u32 = 35;

/// Emits `life_stage_aging_rolls_pending` for a character older than
/// [`AGING_ROLLS_START_AGE`] whose aging log is empty.
///
/// "The first thing to bear in mind is that a character over the age of 35 must
/// make aging rolls (see page 392) before the game begins."
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2232.
///
/// The rule is about *any* character, however it was built, which is why this
/// lives here and not in `validate_life_stage_plan` — that one returns early for a
/// character with no plan, and a directly-entered magus of 60 owes the rolls just
/// as much as a guided one.
///
/// "Over the age of 35" is strict: aging begins "in the Winter after they turn
/// 35" (`:16565`), so 35 owes nothing and 36 owes the first roll.
///
/// The recorded [`Entity::aging_log`] — not [`Entity::aging_points`] — settles the
/// finding: a roll can legitimately produce no aging points, so a well-rolled
/// character would otherwise be nagged forever.
///
/// Filed under [`CreationPhase::Review`] because no aging phase exists yet; slice
/// 6b6 adds the `Aging` variant to [`CreationPhase`] and moves this finding onto
/// it.
fn report_pending_aging_rolls(entity: &Entity, issues: &mut Vec<ValidationIssue>) {
    let Some(age) = entity.age else {
        return;
    };
    if age <= AGING_ROLLS_START_AGE || !entity.aging_log.is_empty() {
        return;
    }

    issues.push(ValidationIssue::warning(
        ValidationIssue::CODE_LIFE_STAGE_AGING_ROLLS_PENDING,
        CreationPhase::Review,
        args([("age", age.to_string())]),
        None,
    ));
}
