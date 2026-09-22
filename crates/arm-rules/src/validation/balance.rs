//! Virtue/Flaw point balance and budget ceilings.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Enforces the two halves of the points rule:
///
/// 1. Flaw points stay within the type's budget (and virtue points within
///    theirs as a clear-message backstop).
/// 2. Virtues must be funded by Flaws: spent virtue points may not exceed the
///    flaw points granted. A character with 10 virtue points and 0 flaw points
///    is over budget on neither total yet is illegal — Players "start with no
///    points for buying Virtues and Flaws, and thus must take Flaws if they
///    want Virtues."
///
/// Source: ArMDE:2774 ("must take
/// Flaws if they want Virtues"), :2297 (companions), :2303 (magi) — "up to ten
/// points of Flaws, and the same number of points of Virtues". The per-type
/// point totals themselves are data in `rules/core/character_types.json` (see
/// RULES.md).
pub(crate) fn validate_balance(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let Balance {
        virtue_points,
        flaw_points,
    } = compute_balance(entity, ruleset);

    let budget = effective_budget(profile);

    if virtue_points > budget.virtue_ceiling {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_BUDGET_VIRTUES,
            CreationPhase::VirtuesFlaws,
            args([
                ("points", virtue_points.to_string()),
                ("budget", budget.virtue_ceiling.to_string()),
            ]),
            None,
        ));
    }

    if flaw_points > budget.flaw_ceiling {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_OVER_BUDGET_FLAWS,
            CreationPhase::VirtuesFlaws,
            args([
                ("points", flaw_points.to_string()),
                ("budget", budget.flaw_ceiling.to_string()),
            ]),
            None,
        ));
    }

    if virtue_points > budget.funded(flaw_points) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_UNBALANCED_VIRTUES,
            CreationPhase::VirtuesFlaws,
            args([
                ("virtue_points", virtue_points.to_string()),
                ("flaw_points", flaw_points.to_string()),
            ]),
            None,
        ));
    }
}

/// The virtue/flaw point ceilings the balance check enforces: the type profile's
/// own budget, and nothing else. No character type — Mythic Companion included —
/// carries bonus points on top of it, so `flaw_ceiling = flaw_points`,
/// `virtue_ceiling = virtue_points`, and `funded = flaw · rate`. Every Mythic
/// Companion gets 10 Flaw / 20 budgeted Virtue points whatever its type; its
/// compulsory package changes how that allowance is spent, never its size
/// (ArMDE:2638, RoP:I:4912 — settled
/// in `docs/vf-audit/decisions.md` D32, pinned by
/// `tests/core_type_conformance.rs`).
pub(crate) struct EffectiveBudget {
    pub(crate) virtue_ceiling: i32,
    pub(crate) flaw_ceiling: i32,
    pub(crate) rate: i32,
}

impl EffectiveBudget {
    /// Virtue points fundable by `flaw_points` taken.
    pub(crate) fn funded(&self, flaw_points: i32) -> i32 {
        flaw_points * self.rate
    }
}

pub(crate) fn effective_budget(profile: &EntityTypeProfile) -> EffectiveBudget {
    EffectiveBudget {
        virtue_ceiling: profile.budget.virtue_points as i32,
        flaw_ceiling: profile.budget.flaw_points as i32,
        rate: profile.budget.virtue_points_per_flaw_point as i32,
    }
}

/// The effective virtue/flaw point ceilings for an entity's type — the type
/// profile's budget. Serializes like its sibling result type `Balance` as
/// `{ "virtue_ceiling": N, "flaw_ceiling": N }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointCeilings {
    /// The highest total virtue points the type permits.
    pub virtue_ceiling: u32,
    /// The highest total flaw points the type permits.
    pub flaw_ceiling: u32,
}

/// The effective virtue/flaw point ceilings for the entity's type, for the
/// frontend's balance display. `None` when the type profile is unknown. Keeps the
/// budget numbers engine-authoritative rather than recomputed in TS.
pub fn effective_point_ceilings(entity: &Entity, ruleset: &Ruleset) -> Option<PointCeilings> {
    let profile = ruleset.profile(&entity.type_id)?;
    let b = effective_budget(profile);
    Some(PointCeilings {
        virtue_ceiling: b.virtue_ceiling.max(0) as u32,
        flaw_ceiling: b.flaw_ceiling.max(0) as u32,
    })
}

/// The accumulated virtue and flaw point totals for an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    /// Total points spent on positive items (virtues, boons).
    pub virtue_points: i32,
    /// Total points granted by negative items (flaws, hooks).
    pub flaw_points: i32,
}

/// Computes the total virtue and flaw points for an entity.
/// Unknown item refs are skipped.
pub fn compute_balance(entity: &Entity, ruleset: &Ruleset) -> Balance {
    let mut virtue_points: i32 = 0;
    let mut flaw_points: i32 = 0;

    for selection in &entity.selections {
        if let Some(item) = ruleset.point_items.get(&selection.item_ref) {
            let pts = item.magnitude.points() as i32;
            if item.kind.is_positive() {
                virtue_points += pts;
            } else {
                flaw_points += pts;
            }
        }
    }

    Balance {
        virtue_points,
        flaw_points,
    }
}
