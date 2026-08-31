//! Prerequisite, incompatibility, and boolean-expression evaluation.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

/// Tri-state outcome of evaluating a prerequisite expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tri {
    /// Definitely satisfied.
    True,
    /// Definitely unsatisfied.
    False,
    /// Cannot be evaluated with the data currently on the entity.
    Unknown,
}

pub(crate) fn validate_prerequisites(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let is_magus = type_profile.map(|p| p.is_magus);

    // Effective score per ability: the max bought score (a parameterized ability
    // may appear more than once with different specialties; the highest wins)
    // plus any virtue bonus (Puissant Ability +2, which now includes a
    // House-granted Puissant via the combined selection list). `AbilityMin`
    // thresholds are checked against the effective score so a boosted ability
    // satisfies them. Keyed by owned `Id` so House-granted ability *floors*
    // (below) can be folded in even for abilities that were never bought.
    let mut ability_scores: BTreeMap<Id, u8> = BTreeMap::new();
    for a in &entity.ability_scores {
        // Per-instance bonus (Puissant targets one (ability, parameter)); an
        // `AbilityMin` is keyed by id, so the strongest instance wins.
        let bonus =
            crate::effective::ability_bonus(entity, ruleset, &a.ability, a.parameter.as_deref());
        let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = ability_scores.entry(a.ability.clone()).or_insert(0);
        *entry = (*entry).max(effective);
    }
    // A free ability-score floor from an `AbilityScoreGrant` effect — including a
    // House-granted Mystery Ability (Bjornaer → Heartbeast 1) — counts toward
    // `AbilityMin` even with no bought row, so fold each granted floor in.
    for floor in crate::effective::ability_score_floors(entity, ruleset) {
        let bonus = crate::effective::ability_bonus(entity, ruleset, &floor.ability, None);
        let effective = (floor.floor + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = ability_scores.entry(floor.ability).or_insert(0);
        *entry = (*entry).max(effective);
    }

    // Effective score per Art: max bought score plus any virtue bonus (Puissant
    // Art +3, including a House-granted Puissant). `ArtMin` thresholds are
    // checked against the effective score.
    let mut art_scores: BTreeMap<Id, u8> = BTreeMap::new();
    for a in &entity.art_scores {
        let bonus = crate::effective::art_bonus(entity, ruleset, &a.art);
        let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
        let entry = art_scores.entry(a.art.clone()).or_insert(0);
        *entry = (*entry).max(effective);
    }

    // `Prereq::Has` resolves against bought AND granted rows (a granted
    // Heartbeast/Dowsing satisfies `Has(...)`), so build a grants-inclusive id
    // set spanning House and Mythic-Companion-type grants. This is deliberately
    // distinct from the bought-only `selected_ids` that the forbidden-trait /
    // incompatibility validators use — grants must never reach those (review
    // finding B1).
    let granted = crate::effective::entity_grants(entity, ruleset);
    let mut present_ids: BTreeSet<&Id> = selected_ids.iter().copied().collect();
    for g in &granted {
        present_ids.insert(&g.item_ref);
    }

    let ctx = PrereqCtx {
        present_ids: &present_ids,
        is_magus,
        house: entity.house.as_ref(),
        ability_scores: &ability_scores,
        art_scores: &art_scores,
    };
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if let Some(ref prereq) = item.prerequisites {
            let (outcome, depended_on_unknown) = evaluate_prereq(prereq, &ctx, 1);
            match outcome {
                Tri::False => {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_PREREQ_NOT_MET,
                        CreationPhase::VirtuesFlaws,
                        args([("item", selection.item_ref.to_string())]),
                        Some(selection.item_ref.clone()),
                    ));
                }
                Tri::Unknown if depended_on_unknown => {
                    issues.push(ValidationIssue::warning(
                        ValidationIssue::CODE_PREREQ_UNEVALUATED,
                        CreationPhase::VirtuesFlaws,
                        args([("item", selection.item_ref.to_string())]),
                        Some(selection.item_ref.clone()),
                    ));
                }
                _ => {}
            }
        }
    }
}

/// The read-only context a prerequisite is evaluated against: which items are
/// selected, whether the type is a magus, and the effective Ability/Art score
/// maps the `AbilityMin`/`ArtMin` thresholds compare against. Bundled so the
/// recursive evaluator and its fold helper take one context rather than a long
/// positional argument list.
struct PrereqCtx<'a> {
    /// The grants-inclusive id set (bought selections ++ House-granted rows) that
    /// `Prereq::Has` tests against — NOT the bought-only `selected_ids` used by
    /// the forbidden-trait / incompatibility checks (review finding B1).
    present_ids: &'a BTreeSet<&'a Id>,
    is_magus: Option<bool>,
    /// The entity's own Hermetic House, if any. `Prereq::House` compares against
    /// it: matching → True, differing → False, absent → Unknown (mirrors how
    /// `is_magus` yields Unknown when the profile is missing).
    house: Option<&'a Id>,
    ability_scores: &'a BTreeMap<Id, u8>,
    art_scores: &'a BTreeMap<Id, u8>,
}

/// Evaluates a prerequisite to a tri-state. Returns the outcome plus whether an
/// unevaluable leaf actually influenced the result (so a warning is only worth
/// emitting when the answer genuinely hinges on missing data).
///
/// `depth` is 1 at the top-level prerequisite and increments once per
/// `All`/`Any`/`Nor` nesting level. Past [`PREREQ_MAX_DEPTH`] this treats the
/// expression as unevaluable rather than recursing further — K8 defense in
/// depth. This should be unreachable in practice: any ruleset whose
/// prerequisites nest that deep is rejected at load by
/// `Ruleset::validate_prereq_refs` (see that function's doc), so this branch
/// exists only to degrade gracefully rather than overflow the stack should a
/// `Prereq` tree ever reach evaluation some other way.
fn evaluate_prereq(prereq: &Prereq, ctx: &PrereqCtx, depth: usize) -> (Tri, bool) {
    if depth > PREREQ_MAX_DEPTH {
        return (Tri::Unknown, true);
    }
    match prereq {
        // The three quantifiers share one tri-state fold over their children,
        // differing only in: which child outcome short-circuits, what the
        // expression then evaluates to, and the value when every child is known
        // and none triggered the short-circuit.
        //   All (AND): trigger on False  -> short-circuit False; all-known -> True
        //   Any (OR) : trigger on True   -> short-circuit True;  all-known -> False
        //   Nor      : trigger on True   -> short-circuit False; all-known -> True
        // In every case a surviving Unknown makes the whole expression Unknown.
        Prereq::All(children) => {
            fold_children(children, ctx, depth, Tri::False, Tri::False, Tri::True)
        }
        Prereq::Any(children) => {
            fold_children(children, ctx, depth, Tri::True, Tri::True, Tri::False)
        }
        Prereq::Nor(children) => {
            fold_children(children, ctx, depth, Tri::True, Tri::False, Tri::True)
        }
        Prereq::Has(id) => {
            if ctx.present_ids.contains(id) {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // IsMagus is enforced against the profile's explicit `is_magus` flag (a
        // Hermetic-Magus-status type), independent of gift_policy.
        Prereq::IsMagus => match ctx.is_magus {
            Some(true) => (Tri::True, false),
            Some(false) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
        // AbilityMin compares against the entity's max *effective* score for
        // that ability (bought score plus virtue bonuses such as Puissant
        // Ability), as supplied by the caller. An ability the entity does not
        // have counts as score 0, so any positive threshold is False.
        Prereq::AbilityMin { ability, score } => {
            let have = ctx.ability_scores.get(ability).copied().unwrap_or(0);
            if have >= *score {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // ArtMin compares against the entity's max *effective* Art score (bought
        // plus Puissant Art). An Art the entity does not have counts as 0.
        Prereq::ArtMin { art, score } => {
            let have = ctx.art_scores.get(art).copied().unwrap_or(0);
            if have >= *score {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // House matches against the entity's own house: a known house that
        // matches is True, a known house that differs is False, and no house at
        // all (non-magus or an unset magus) is genuinely Unknown.
        Prereq::House(id) => match ctx.house {
            Some(h) if h == id => (Tri::True, false),
            Some(_) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
    }
}

/// Tri-state fold shared by the `All`/`Any`/`Nor` quantifiers (see the call
/// sites for the per-quantifier parameterization).
///
/// Walks the children once: if any child evaluates to `trigger`, the whole
/// expression short-circuits to `short_circuit` (a definite True/False, so its
/// dependency flag is irrelevant downstream and reported as `false`). Otherwise,
/// a surviving `Unknown` makes the result `Unknown` (carrying whether that
/// hinged on genuinely missing data); if every child is known, the result is
/// `all_known`.
fn fold_children(
    children: &[Prereq],
    ctx: &PrereqCtx,
    depth: usize,
    trigger: Tri,
    short_circuit: Tri,
    all_known: Tri,
) -> (Tri, bool) {
    let mut depended = false;
    let mut saw_unknown = false;
    for child in children {
        let (outcome, dep) = evaluate_prereq(child, ctx, depth + 1);
        if outcome == trigger {
            return (short_circuit, false);
        }
        if outcome == Tri::Unknown {
            saw_unknown = true;
            depended |= dep;
        }
    }
    if saw_unknown {
        (Tri::Unknown, depended)
    } else {
        (all_known, false)
    }
}

pub(crate) fn validate_incompatibilities(
    entity: &Entity,
    ruleset: &Ruleset,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut reported: BTreeSet<(&Id, &Id)> = BTreeSet::new();

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        for incompat_id in &item.incompatible_with {
            if selected_ids.contains(incompat_id) {
                // Normalize the pair order so a mutual incompatibility is
                // reported exactly once.
                let pair = if selection.item_ref < *incompat_id {
                    (&selection.item_ref, incompat_id)
                } else {
                    (incompat_id, &selection.item_ref)
                };
                if reported.insert(pair) {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_INCOMPATIBLE,
                        CreationPhase::VirtuesFlaws,
                        args([
                            ("item", selection.item_ref.to_string()),
                            ("other", incompat_id.to_string()),
                        ]),
                        Some(selection.item_ref.clone()),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> (BTreeSet<Id>, BTreeMap<Id, u8>, BTreeMap<Id, u8>) {
        (BTreeSet::new(), BTreeMap::new(), BTreeMap::new())
    }

    /// K8 defense-in-depth, unit-tested directly against the module-private
    /// `evaluate_prereq` (unreachable from outside `validation::prereq`,
    /// hence this in-module test rather than one alongside the others in
    /// `validation::tests`). The load-time guard
    /// (`Ruleset::validate_prereq_refs`, tested in `ruleset.rs`) means a real
    /// `Ruleset` can never carry a `Prereq` this deep, so the only way to
    /// exercise this branch is to call `evaluate_prereq` with a `depth`
    /// starting above the limit directly, exactly as this test does.
    #[test]
    fn evaluate_prereq_treats_over_depth_as_unknown_instead_of_recursing() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids: &present_ids,
            is_magus: None,
            house: None,
            ability_scores: &ability_scores,
            art_scores: &art_scores,
        };

        // A single leaf, but evaluated as though it were already past the
        // depth limit — proves the guard fires on `depth`, not on actually
        // walking a deep tree (which would defeat the point of testing this
        // in isolation from the load-time guard).
        let (outcome, depended_on_unknown) =
            evaluate_prereq(&Prereq::IsMagus, &ctx, PREREQ_MAX_DEPTH + 1);
        assert_eq!(outcome, Tri::Unknown);
        assert!(depended_on_unknown);
    }

    #[test]
    fn evaluate_prereq_at_exactly_the_depth_limit_still_evaluates_normally() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids: &present_ids,
            is_magus: Some(true),
            house: None,
            ability_scores: &ability_scores,
            art_scores: &art_scores,
        };

        let (outcome, depended_on_unknown) =
            evaluate_prereq(&Prereq::IsMagus, &ctx, PREREQ_MAX_DEPTH);
        assert_eq!(outcome, Tri::True);
        assert!(!depended_on_unknown);
    }
}
