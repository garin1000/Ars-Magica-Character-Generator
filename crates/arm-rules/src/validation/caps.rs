//! Per-type and tainted count caps on selections.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Enforces per-type caps on the *count* of items (distinct from the point
/// budget). The caps themselves are data in the type profile; whether a cap is
/// a hard rule (error) or a soft guideline (warning) is fixed by the rulebook
/// and encoded here per cap. A cap left `None`/absent imposes no limit.
///
/// Per-category flaw caps (Personality, Story, ...) are data in the profile's
/// `flaw_category_caps`: each entry names its category, so no category slug is
/// hardcoded in the engine.
///
/// Source: grogs may take no Major Virtues or Flaws (the `max_major_*` count
/// caps) at Ars Magica - Definitive Edition (Core Rules).md:2824-2830; ≤5 Minor
/// Flaws (central) at :2774, grogs ≤3 at :1009; ≤1 Major Personality Flaw at
/// :2820; ≤2 Personality Flaws (soft) at :2820/:2976; ≤1 Story Flaw (soft) at
/// :2818, grogs none at :1009. See RULES.md.
pub(crate) fn validate_caps(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    // Counts selections whose resolved point item matches `pred`.
    let count = |pred: &dyn Fn(&PointItem) -> bool| -> usize {
        entity
            .selections
            .iter()
            .filter(|s| ruleset.point_items.get(&s.item_ref).is_some_and(pred))
            .count()
    };

    let count_args = |n: usize, max: u8| args([("count", n.to_string()), ("max", max.to_string())]);

    // --- Hard caps ("may not ...") → blocking errors ---

    if let Some(max) = profile.budget.max_major_virtues {
        let n = count(&|i| i.kind == ItemKind::Virtue && i.magnitude == Magnitude::Major);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_TOO_MANY_MAJOR_VIRTUES,
                CreationPhase::VirtuesFlaws,
                count_args(n, max),
                None,
            ));
        }
    }

    if let Some(max) = profile.budget.max_major_flaws {
        let n = count(&|i| i.kind == ItemKind::Flaw && i.magnitude == Magnitude::Major);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_TOO_MANY_MAJOR_FLAWS,
                CreationPhase::VirtuesFlaws,
                count_args(n, max),
                None,
            ));
        }
    }

    if let Some(max) = profile.budget.max_minor_flaws {
        let n = count(&|i| i.kind == ItemKind::Flaw && i.magnitude == Magnitude::Minor);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_TOO_MANY_MINOR_FLAWS,
                CreationPhase::VirtuesFlaws,
                count_args(n, max),
                None,
            ));
        }
    }

    // --- Data-driven per-category caps ---
    //
    // Each cap names its category as data, so the engine never hardcodes a
    // category slug. A `hard` cap is a blocking error; otherwise a non-blocking
    // warning (the book marks the Personality/Story guidelines as
    // troupe-overridable). The issue code is derived from the category slug as
    // `too_many_<category>_<noun>` (or `too_many_major_<category>_<noun>` when
    // the cap is Major-only), so the Fluent key follows the category by
    // convention — no slug is baked into the engine. Counts `entity.selections`
    // only, so House-granted items (which never enter the bought list) are
    // exempt — Bjornaer's Major Hermetic Heartbeast cannot trip a virtue cap.
    //
    // Source: Ars Magica - Definitive Edition (Core Rules).md:2855-2861.
    let mut push_category_cap_issues = |caps: &[CategoryCap], kind: ItemKind, noun: &str| {
        for cap in caps {
            let n = count(&|i| {
                i.kind == kind
                    && i.category == cap.category
                    && (!cap.major_only || i.magnitude == Magnitude::Major)
            });
            if n <= cap.max as usize {
                continue;
            }

            let code = if cap.major_only {
                format!("too_many_major_{}_{}", cap.category, noun)
            } else {
                format!("too_many_{}_{}", cap.category, noun)
            };
            let cap_args = count_args(n, cap.max);

            if cap.hard {
                issues.push(ValidationIssue::error(
                    &code,
                    CreationPhase::VirtuesFlaws,
                    cap_args,
                    None,
                ));
            } else {
                issues.push(ValidationIssue::warning(
                    &code,
                    CreationPhase::VirtuesFlaws,
                    cap_args,
                    None,
                ));
            }
        }
    };

    push_category_cap_issues(&profile.budget.flaw_category_caps, ItemKind::Flaw, "flaws");
    push_category_cap_issues(
        &profile.budget.virtue_category_caps,
        ItemKind::Virtue,
        "virtues",
    );
}

/// Warns when more than half the Virtue points a character has taken are Tainted
/// (and likewise for Flaws). The rulebook frames this as a "should" ("no more
/// than half a character's Virtues should be tainted, and similarly for Flaws"),
/// so it is a non-blocking warning; the limit is measured against the points
/// actually taken, not the type's budget. Free items contribute 0 points and so
/// never affect the ratio.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2998-3002.
pub(crate) fn validate_tainted_cap(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let (mut tainted_virtue, mut total_virtue) = (0i32, 0i32);
    let (mut tainted_flaw, mut total_flaw) = (0i32, 0i32);
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        let pts = item.magnitude.points() as i32;
        if item.kind.is_positive() {
            total_virtue += pts;
            if item.tainted {
                tainted_virtue += pts;
            }
        } else {
            total_flaw += pts;
            if item.tainted {
                tainted_flaw += pts;
            }
        }
    }

    let mut warn = |tainted: i32, total: i32, code: &str| {
        // "No more than half": tainted may equal half but not exceed it. The
        // integer form `2·tainted > total` sidesteps any rounding choice.
        if tainted * 2 > total {
            issues.push(ValidationIssue::warning(
                code,
                CreationPhase::VirtuesFlaws,
                args([
                    ("tainted", tainted.to_string()),
                    ("total", total.to_string()),
                ]),
                None,
            ));
        }
    };
    warn(
        tainted_virtue,
        total_virtue,
        ValidationIssue::CODE_TOO_MANY_TAINTED_VIRTUES,
    );
    warn(
        tainted_flaw,
        total_flaw,
        ValidationIssue::CODE_TOO_MANY_TAINTED_FLAWS,
    );
}
