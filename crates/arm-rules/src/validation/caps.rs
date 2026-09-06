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
    //
    // V66: the three hard caps used to be three copy-pasted blocks, differing
    // only in which budget field, (kind, magnitude) predicate, and issue code
    // each checked — a bug fixed in one was one bug fixed in three. One loop
    // over this table now drives all three; each row is still a compile-time
    // constant (the `ValidationIssue::CODE_*` referenced directly, not built
    // by `format!` like the data-driven per-category caps below), so a typo'd
    // code still fails to compile.
    let hard_caps: [(Option<u8>, ItemKind, Magnitude, &str); 3] = [
        (
            profile.budget.max_major_virtues,
            ItemKind::Virtue,
            Magnitude::Major,
            ValidationIssue::CODE_TOO_MANY_MAJOR_VIRTUES,
        ),
        (
            profile.budget.max_major_flaws,
            ItemKind::Flaw,
            Magnitude::Major,
            ValidationIssue::CODE_TOO_MANY_MAJOR_FLAWS,
        ),
        (
            profile.budget.max_minor_flaws,
            ItemKind::Flaw,
            Magnitude::Minor,
            ValidationIssue::CODE_TOO_MANY_MINOR_FLAWS,
        ),
    ];
    for (max, kind, magnitude, code) in hard_caps {
        let Some(max) = max else { continue };
        let n = count(&|i| i.kind == kind && i.magnitude == magnitude);
        if n > max as usize {
            issues.push(ValidationIssue::error(
                code,
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
            // An item counts against the cap when it *carries* the capped
            // category, primary or secondary: Suppressed Gift is "*Major,
            // Hermetic, Story*" (Ars Magica - Definitive Edition (Core
            // Rules).md:6803-6804), so it is a Story Flaw for the Story cap just
            // as much as it is a Hermetic one.
            let n = count(&|i| {
                i.kind == kind
                    && i.has_category(&cap.category)
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

/// Warns when the copies of one item account for more of its own kind's point
/// total than its descriptor allows — the ratio each item states as data in
/// [`PointItem::max_share_of_kind`].
///
/// Modelled on [`validate_tainted_cap`], which implements the same sentence
/// shape for the Tainted guideline, and identical to it in three respects:
/// points rather than headcount, the rounding-free integer comparison, and
/// **warning** severity. It differs in one: it takes the **folded** selection
/// list (bought ++ granted) rather than `entity.selections`, because a granted
/// copy is still a copy — Devil Child hands out a free Demonic Might or Demonic
/// Powers (Ars Magica - Definitive Edition (Core Rules).md:3673). See RULES.md
/// for why that divergence from the Tainted precedent is deliberate.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3665 (Demonic Might)
/// and :3669 (Demonic Powers) — "no more than half of the character's total
/// Virtues", read as points, which is an interpretation (see RULES.md) and the
/// second reason this is a warning rather than an error.
pub(crate) fn validate_share_of_kind_cap(
    selections: &[Selection],
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let (mut total_virtue, mut total_flaw) = (0i64, 0i64);
    // Points held per share-capped item; ordered so the issue order is stable.
    let mut capped_points: BTreeMap<&Id, i64> = BTreeMap::new();

    for selection in selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        let pts = item.magnitude.points() as i64;
        if item.kind.is_positive() {
            total_virtue += pts;
        } else {
            total_flaw += pts;
        }
        if item.max_share_of_kind.is_some() {
            *capped_points.entry(&selection.item_ref).or_insert(0) += pts;
        }
    }

    for (item_ref, points) in capped_points {
        let Some(item) = ruleset.point_items.get(item_ref) else {
            continue;
        };
        let Some(share) = item.max_share_of_kind else {
            continue;
        };
        let total = if item.kind.is_positive() {
            total_virtue
        } else {
            total_flaw
        };
        // "No more than <share>": the item's copies may equal the share but not
        // exceed it. The integer form `part · denominator > total · numerator`
        // sidesteps any rounding choice, and with 1/2 it is the `2·part > total`
        // the Tainted cap already uses.
        if points * i64::from(share.denominator) <= total * i64::from(share.numerator) {
            continue;
        }
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_TOO_LARGE_SHARE,
            CreationPhase::VirtuesFlaws,
            args([
                ("item", item_ref.to_string()),
                ("points", points.to_string()),
                ("total", total.to_string()),
            ]),
            Some(item_ref.clone()),
        ));
    }
}
