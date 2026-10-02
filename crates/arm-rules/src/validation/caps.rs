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
/// caps) at ArMDE:2824-2830; ≤5 Minor
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

    // Counts selections whose resolved point item (and that selection's own
    // params, so a taken-as-aware predicate can read `s.params`) matches `pred`.
    let count = |pred: &dyn Fn(&PointItem, &Selection) -> bool| -> usize {
        entity
            .selections
            .iter()
            .filter(|s| {
                ruleset
                    .point_items
                    .get(&s.item_ref)
                    .is_some_and(|item| pred(item, s))
            })
            .count()
    };

    // The same match, but naming which items matched rather than merely
    // counting them (D41/B2): a category-cap ceiling warning must name the
    // paired entries, not just say "2 of 1". Sorted so a pair reports in a
    // stable order across runs.
    let matching_ids = |pred: &dyn Fn(&PointItem, &Selection) -> bool| -> Vec<Id> {
        let mut ids: Vec<Id> = entity
            .selections
            .iter()
            .filter(|s| {
                ruleset
                    .point_items
                    .get(&s.item_ref)
                    .is_some_and(|item| pred(item, s))
            })
            .map(|s| s.item_ref.clone())
            .collect();
        ids.sort();
        ids
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
        let n = count(&|i, _s| i.kind == kind && i.magnitude == magnitude);
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
    // Source: ArMDE:2855-2861.
    let mut push_category_cap_issues = |caps: &[CategoryCap], kind: ItemKind, noun: &str| {
        for cap in caps {
            // An item counts against the cap when it *carries* the capped
            // category, primary or secondary: Suppressed Gift is "*Major,
            // Hermetic, Story*" (ArMDE:6803-6804), so it is a Story Flaw for the Story cap just
            // as much as it is a Hermetic one.
            //
            // Taken-as aware, via `PointItem::categories_for`: a Sufi taken as
            // Social Status must not count against a Supernatural cap, and vice
            // versa — `ArMDE:5083` is a choice between the two readings, not both at
            // once.
            let mut matched = matching_ids(&|i, s| {
                (cap.both_kinds || i.kind == kind)
                    && i.categories_for(&s.params)
                        .iter()
                        .any(|c| c == &cap.category)
                    && (!cap.major_only || i.magnitude == Magnitude::Major)
            });

            // D68.11: fold in `Effect::GrantsCategoryCount` on BOUGHT
            // selections (Mythic Blood's hereditary Personality Flaw) — an
            // id-less phantom item, so the granting selection's own id
            // stands in for it in the paired-entry message. Bought-only,
            // matching this whole function's "House-granted items are
            // exempt" scope: the effect is defined to be read only from
            // `entity.selections`, never a granted row.
            for selection in &entity.selections {
                let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
                    continue;
                };
                let counts = item.effects.iter().any(|effect| {
                    matches!(
                        effect,
                        Effect::GrantsCategoryCount { category, magnitude, item_kind }
                            if (cap.both_kinds || *item_kind == kind)
                                && category == &cap.category
                                && (!cap.major_only || *magnitude == Magnitude::Major)
                    )
                });
                if counts {
                    matched.push(selection.item_ref.clone());
                }
            }
            matched.sort();
            let n = matched.len();

            if n > cap.max as usize {
                let code = if cap.major_only {
                    format!("too_many_major_{}_{}", cap.category, noun)
                } else {
                    format!("too_many_{}_{}", cap.category, noun)
                };
                // D41: a soft ceiling (a second Social Status, say) must name
                // the paired entries so the player can see what was paired —
                // not category-specific, so any ceiling breach gets this for
                // free. Only the first two matches are named (every shipped
                // ceiling caps at 1 or 2 today, so a third is not yet
                // reachable; naming more would need a third arg key).
                let mut cap_args = count_args(n, cap.max);
                if let [first, second, ..] = matched.as_slice() {
                    cap_args.insert("item".to_string(), first.to_string());
                    cap_args.insert("other".to_string(), second.to_string());
                }

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

            // B1/D21/F-427/D41: the floor half, additive to the ceiling
            // above and independent of it (no `continue` between them — a
            // malformed cap could in principle trip both, though the
            // load-time `min <= max` check makes that unreachable in
            // practice). ArMDE:2816's "must take one Social Status" is the
            // first data user (B2/D41); `min` stays `None` for every OTHER
            // shipped cap, so this is a no-op there.
            if let Some(min) = cap.min
                && n < min as usize
            {
                let code = if cap.major_only {
                    format!("too_few_major_{}_{}", cap.category, noun)
                } else {
                    format!("too_few_{}_{}", cap.category, noun)
                };
                let floor_args = args([("count", n.to_string()), ("min", min.to_string())]);

                if cap.min_hard {
                    issues.push(ValidationIssue::error(
                        &code,
                        CreationPhase::VirtuesFlaws,
                        floor_args,
                        None,
                    ));
                } else {
                    issues.push(ValidationIssue::warning(
                        &code,
                        CreationPhase::VirtuesFlaws,
                        floor_args,
                        None,
                    ));
                }
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
/// Source: ArMDE:2998-3002.
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
/// Powers (ArMDE:3673). See RULES.md
/// for why that divergence from the Tainted precedent is deliberate.
///
/// Source: ArMDE:3665 (Demonic Might)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ruleset::Ruleset;
    use crate::types::{Entity, EntityKind, RulesetRef};

    /// No shipped profile combines a category-cap floor (`min`) with
    /// `min_hard: false` (a soft guideline) or with `major_only: true` (the
    /// "major" issue-code wording) — the only shipped floor
    /// (`social_status`, `rules/core/character_types.json`) is `min_hard:
    /// true, major_only` absent. Both branches
    /// (`validation/caps.rs::push_category_cap_issues`) are real and
    /// reachable, just untested with real data, so this builds a minimal
    /// synthetic ruleset carrying both combinations on one data-only
    /// `test_status` category — no production code changes.
    fn ruleset_with_test_status_caps() -> Ruleset {
        let items = r#"[
          { "id": "virtue.test_status_small", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "categories": ["test_status"] },
          { "id": "virtue.test_status_large", "kind": "virtue", "classification": "narrative",
            "magnitude": "major", "categories": ["test_status"] },
          { "id": "flaw.personality_filler", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"] }
        ]"#;
        let profiles = r#"[
          {
            "id": "test_type",
            "creation_phases": ["virtues_flaws"],
            "budget": {
              "virtue_points": 10,
              "flaw_points": 10,
              "virtue_category_caps": [
                { "category": "test_status", "max": 5, "min": 1, "min_hard": false },
                { "category": "test_status", "max": 5, "min": 1, "min_hard": true, "major_only": true }
              ]
            }
          }
        ]"#;
        Ruleset::from_json("t", "1", items, profiles).expect("synthetic ruleset loads")
    }

    fn entity_with(selections: Vec<Selection>) -> Entity {
        let mut e = Entity::new(
            EntityKind::Character,
            Id::new("test_type"),
            RulesetRef::new(Id::new("t"), "1"),
        );
        e.selections = selections;
        e
    }

    fn issues_for(ruleset: &Ruleset, selections: Vec<Selection>) -> Vec<ValidationIssue> {
        let profile = ruleset.profile(&Id::new("test_type")).unwrap();
        let entity = entity_with(selections);
        let mut issues = Vec::new();
        validate_caps(&entity, ruleset, Some(profile), &mut issues);
        issues
    }

    #[test]
    fn a_soft_category_floor_below_minimum_raises_a_warning_not_an_error() {
        let ruleset = ruleset_with_test_status_caps();
        let issues = issues_for(&ruleset, vec![]);

        let soft_floor = issues
            .iter()
            .find(|i| i.code == "too_few_test_status_virtues")
            .expect("the min_hard: false floor must still raise its issue when unmet");
        assert_eq!(
            soft_floor.severity,
            IssueSeverity::Warning,
            "min_hard: false must raise a WARNING, not an error — got {soft_floor:?}"
        );
    }

    #[test]
    fn a_major_only_category_floor_uses_the_major_wording_and_is_an_error() {
        let ruleset = ruleset_with_test_status_caps();
        let issues = issues_for(&ruleset, vec![]);

        let major_floor = issues
            .iter()
            .find(|i| i.code == "too_few_major_test_status_virtues")
            .expect("major_only: true must derive the 'too_few_major_<category>' code");
        assert_eq!(
            major_floor.severity,
            IssueSeverity::Error,
            "min_hard: true must raise an ERROR — got {major_floor:?}"
        );
    }

    #[test]
    fn a_major_only_floor_counts_only_major_magnitude_items() {
        let ruleset = ruleset_with_test_status_caps();
        // A MINOR test_status virtue clears the plain (non-major_only) floor,
        // which counts any magnitude, but must leave the major_only floor
        // unmet, since it counts only Major-magnitude items.
        let issues = issues_for(
            &ruleset,
            vec![Selection::new(Id::new("virtue.test_status_small"))],
        );

        assert!(
            !issues
                .iter()
                .any(|i| i.code == "too_few_test_status_virtues"),
            "a minor test_status virtue must clear the plain floor — issues: {issues:?}"
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code == "too_few_major_test_status_virtues"),
            "a MINOR item must not satisfy the major_only floor — issues: {issues:?}"
        );
    }

    #[test]
    fn a_major_only_floor_is_cleared_by_a_major_magnitude_item() {
        let ruleset = ruleset_with_test_status_caps();
        let issues = issues_for(
            &ruleset,
            vec![Selection::new(Id::new("virtue.test_status_large"))],
        );

        assert!(
            !issues
                .iter()
                .any(|i| i.code == "too_few_major_test_status_virtues"),
            "a MAJOR test_status virtue must clear the major_only floor — issues: {issues:?}"
        );
    }
}
