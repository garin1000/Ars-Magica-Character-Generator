//! Prerequisite, incompatibility, and boolean-expression evaluation.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;
use crate::types::AbilityParameterValue;

/// Tri-state outcome of evaluating a prerequisite expression.
///
/// `pub(crate)`: sibling validators outside this module (B5's conditional
/// category rules, B9's possessed-target check) call [`PrereqCtx::evaluate`],
/// whose return type this appears in, so it must be at least as visible as
/// that method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tri {
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
    ctx: &PrereqCtx,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if let Some(ref prereq) = item.prerequisites {
            // B2/ArMDE:4441: an item's own prerequisite excludes its own
            // contribution to `HasCategory` — see `evaluate_for_item`'s doc
            // comment.
            let (outcome, depended_on_unknown) = ctx.evaluate_for_item(prereq, &selection.item_ref);
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

        // A HEDGED restriction (F-550/D16/Q-115): evaluated by the same
        // tri-state machinery, on its OWN tree so it can never interact with
        // the hard one under All/Any/Nor. Only a definite False is worth
        // surfacing — an Unknown means the hedge cannot yet be resolved (e.g.
        // no House set yet), and D16 hedges to a warning on a stated
        // violation, not to a nag about missing data the way the hard tree's
        // `prereq_unevaluated` does.
        if let Some(ref advisory) = item.advisory_prerequisites {
            let (outcome, _) = ctx.evaluate_for_item(advisory, &selection.item_ref);
            if outcome == Tri::False {
                issues.push(ValidationIssue::warning(
                    ValidationIssue::CODE_ADVISORY_PREREQ_NOT_MET,
                    CreationPhase::VirtuesFlaws,
                    args([("item", selection.item_ref.to_string())]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}

/// The read-only context a prerequisite is evaluated against: which items are
/// selected, whether the entity is Hermetically trained and/or an Order
/// member, and the effective Ability/Art score
/// maps the `AbilityMin`/`ArtMin` thresholds compare against. Bundled so the
/// recursive evaluator and its fold helper take one context rather than a long
/// positional argument list.
///
/// `pub(crate)`: built once in `validation::validate` (beside the `granted` /
/// `effective_selections` locals it is derived from) and passed by reference
/// into `validate_prerequisites`. Sibling validators in other modules build
/// their own via [`PrereqCtx::build`] — B5's conditional category rules and
/// B9's "is this Virtue actually possessed" check both need to evaluate a
/// `Prereq` against the same entity without duplicating this fold.
pub(crate) struct PrereqCtx<'a> {
    /// The grants-inclusive id set (bought selections ++ House-granted rows) that
    /// `Prereq::Has` tests against — NOT the bought-only `selected_ids` used by
    /// the forbidden-trait / incompatibility checks (review finding B1).
    ///
    /// `pub(crate)`: B9's possessed-target validator reads this set directly
    /// (a House-granted Supernatural Virtue counts as possessed) rather than
    /// re-deriving it.
    pub(crate) present_ids: BTreeSet<&'a Id>,
    /// `Prereq::HermeticallyTrained`'s fact: `None` when the type profile is
    /// missing (→ `Tri::Unknown`), otherwise the union of the profile's
    /// `hermetically_trained` flag with `entity_confers_hermetic_training`
    /// (D56/A0).
    trained: Option<bool>,
    /// `Prereq::OrderMember`'s fact: `None` when the type profile is missing,
    /// otherwise the profile's `order_member` flag alone — no entity-level
    /// override (D56/A0).
    order: Option<bool>,
    /// `Prereq::IsCompanion`'s fact: `None` when the type profile is missing,
    /// otherwise the profile's `is_companion` flag alone — no entity-level
    /// override, mirroring `trained`/`order` (D38).
    is_companion: Option<bool>,
    /// The entity's own Hermetic House, if any. `Prereq::House` compares against
    /// it: matching → True, differing → False, absent → Unknown (mirrors how
    /// `trained`/`order` yield Unknown when the profile is missing).
    house: Option<&'a Id>,
    ability_scores: BTreeMap<Id, u8>,
    art_scores: BTreeMap<Id, u8>,
    /// `Prereq::HasCategory`'s fact (B1/D21/F-502): every in-force category
    /// ([`PointItem::categories_for`]) of every bought-OR-granted item this
    /// entity holds — the grant-aware twin of
    /// [`crate::validation::selections::categories_in_force`]'s bought-only
    /// fold. `flaw.rector`'s "must have a Social Status Virtue"
    /// (ArMDE:6671-6674) is satisfied by a
    /// House-granted Social Status exactly as by a bought one, the same reach
    /// `present_ids` already gives `Has`.
    ///
    /// Keyed on category, valued on the `item_ref`s that contribute it (not a
    /// bare `BTreeSet<String>`) so [`evaluate_for_item`](PrereqCtx::evaluate_for_item)
    /// can ask "does some item OTHER than the one asking hold this category" —
    /// B2/ArMDE:4441: `virtue.male_guild_sponsor` is itself `social_status`, so
    /// a self-blind `HasCategory` on its own prerequisite would be trivially
    /// satisfied by itself and never actually require the SEPARATE guild
    /// status the book demands.
    held_categories: BTreeMap<String, BTreeSet<Id>>,
    /// `Prereq::AgeMin`'s fact (D69/X7b-e row 42): the entity's own
    /// [`Entity::age`], `None` when unset (genuinely unknown, mirroring
    /// `trained`/`order`/`house`).
    age: Option<u32>,
    /// `Prereq::HasCategoryAtMagnitude`'s fact (D69/X7b-e row 42): every
    /// bought-OR-granted item's in-force categories, keyed by `(category,
    /// kind)` and valued on the magnitude each contributing `item_ref`
    /// carries — the magnitude- and kind-aware twin of
    /// [`Self::held_categories`]. Kept as its own map (rather than widening
    /// `held_categories`) so `Prereq::HasCategory`'s existing kind-blind
    /// query is unaffected.
    held_categories_by_kind: BTreeMap<(String, ItemKind), BTreeMap<Id, Magnitude>>,
}

impl<'a> PrereqCtx<'a> {
    /// Builds the context once from an entity/ruleset pair: folds bought +
    /// virtue-boosted Ability and Art scores, and unions bought selections
    /// with `granted` rows into the `Has`-satisfying id set.
    ///
    /// This is the effective-score folding that used to run inside
    /// `validate_prerequisites` on every call; hoisting it here lets
    /// `validation::validate` build one `PrereqCtx` and share it with
    /// `validate_prerequisites` and any later sibling validator, instead of
    /// re-folding per caller.
    pub(crate) fn build(
        entity: &'a Entity,
        ruleset: &Ruleset,
        type_profile: Option<&EntityTypeProfile>,
        selected_ids: &BTreeSet<&'a Id>,
        granted: &'a [Selection],
    ) -> Self {
        // `type_profile.map` short-circuits to `None` (→ `Tri::Unknown`) when
        // the profile itself cannot be resolved, exactly mirroring how the old
        // single `is_magus` built — the OR-check only ever runs once a profile
        // *is* in hand. See `docs/vf-audit/design-a0-is-magus-split.md` § 1.
        let trained: Option<bool> = type_profile.map(|p| {
            p.hermetically_trained
                || crate::effective::entity_confers_hermetic_training(entity, ruleset)
        });
        let order: Option<bool> = type_profile.map(|p| p.order_member);
        let is_companion: Option<bool> = type_profile.map(|p| p.is_companion);

        // Effective score per ability: the max bought score (a parameterized
        // ability may appear more than once with different specialties; the
        // highest wins) plus any virtue bonus (Puissant Ability +2, which now
        // includes a House-granted Puissant via the combined selection list).
        // `AbilityMin` thresholds are checked against the effective score so a
        // boosted ability satisfies them. Keyed by owned `Id` so House-granted
        // ability *floors* (below) can be folded in even for abilities that
        // were never bought.
        let mut ability_scores: BTreeMap<Id, u8> = BTreeMap::new();
        for a in &entity.ability_scores {
            // Per-instance bonus (Puissant targets one (ability, parameter));
            // an `AbilityMin` is keyed by id, so the strongest instance wins.
            let bonus = crate::effective::ability_bonus(
                entity,
                ruleset,
                &a.ability,
                a.parameter
                    .as_ref()
                    .and_then(AbilityParameterValue::match_key),
            );
            let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
            let entry = ability_scores.entry(a.ability.clone()).or_insert(0);
            *entry = (*entry).max(effective);
        }
        // A free ability-score floor from an `AbilityScoreGrant` effect —
        // including a House-granted Mystery Ability (Bjornaer → Heartbeast 1)
        // — counts toward `AbilityMin` even with no bought row, so fold each
        // granted floor in.
        for floor in crate::effective::ability_score_floors(entity, ruleset) {
            let bonus = crate::effective::ability_bonus(entity, ruleset, &floor.ability, None);
            let effective = (floor.floor + bonus).clamp(0, i32::from(u8::MAX)) as u8;
            let entry = ability_scores.entry(floor.ability).or_insert(0);
            *entry = (*entry).max(effective);
        }

        // Effective score per Art: max bought score plus any virtue bonus
        // (Puissant Art +3, including a House-granted Puissant). `ArtMin`
        // thresholds are checked against the effective score.
        let mut art_scores: BTreeMap<Id, u8> = BTreeMap::new();
        for a in &entity.art_scores {
            let bonus = crate::effective::art_bonus(entity, ruleset, &a.art);
            let effective = (i32::from(a.score) + bonus).clamp(0, i32::from(u8::MAX)) as u8;
            let entry = art_scores.entry(a.art.clone()).or_insert(0);
            *entry = (*entry).max(effective);
        }

        // `Prereq::Has` resolves against bought AND granted rows (a granted
        // Heartbeast/Dowsing satisfies `Has(...)`), so build a grants-inclusive
        // id set spanning House and Mythic-Companion-type grants (`granted`,
        // computed once by the caller — see `super::validate`). This is
        // deliberately distinct from the bought-only `selected_ids` that the
        // forbidden-trait / incompatibility validators use — grants must
        // never reach those (review finding B1).
        let mut present_ids: BTreeSet<&Id> = selected_ids.iter().copied().collect();
        for g in granted {
            present_ids.insert(&g.item_ref);
        }

        // `Prereq::HasCategory`'s grants-inclusive category set (B1/D21):
        // every bought-OR-granted selection's in-force categories, read the
        // taken-as-aware way `categories_for` already resolves for the
        // bought-only profile-level gates. Attributed per contributing
        // `item_ref` (B2) so a self-excluding lookup can tell "held by this
        // item alone" from "held by some OTHER item too".
        let mut held_categories: BTreeMap<String, BTreeSet<Id>> = BTreeMap::new();
        let mut held_categories_by_kind: BTreeMap<(String, ItemKind), BTreeMap<Id, Magnitude>> =
            BTreeMap::new();
        for selection in entity.selections.iter().chain(granted.iter()) {
            if let Some(item) = ruleset.point_items.get(&selection.item_ref) {
                for category in item.categories_for(&selection.params) {
                    held_categories
                        .entry(category.clone())
                        .or_default()
                        .insert(selection.item_ref.clone());
                    held_categories_by_kind
                        .entry((category.clone(), item.kind))
                        .or_default()
                        .insert(selection.item_ref.clone(), item.magnitude);
                }
            }
        }

        PrereqCtx {
            present_ids,
            trained,
            order,
            is_companion,
            house: entity.house.as_ref(),
            ability_scores,
            art_scores,
            held_categories,
            age: entity.age,
            held_categories_by_kind,
        }
    }

    /// Evaluates a prerequisite expression to a tri-state against this
    /// context. Thin wrapper over the free recursive [`evaluate_prereq`]
    /// starting at depth 1 (the top level of the expression tree).
    ///
    /// Excludes no item's own contribution: correct for a PROFILE-level gate
    /// (`CategoryRule.when`) that belongs to no single selection. An item's
    /// OWN `prerequisites`/`advisory_prerequisites` must go through
    /// [`Self::evaluate_for_item`] instead (B2).
    pub(crate) fn evaluate(&self, prereq: &Prereq) -> (Tri, bool) {
        evaluate_prereq(prereq, self, 1, None)
    }

    /// Evaluates a prerequisite as `item_ref`'s OWN statement about itself
    /// (`PointItem::prerequisites`/`advisory_prerequisites`) — B2/ArMDE:4441.
    ///
    /// A prerequisite states what the REST of the character must hold, not
    /// what the item itself trivially supplies. `Prereq::HasCategory`
    /// therefore excludes `item_ref`'s own contributed categories: without
    /// this, an item whose own category equals the category it asks about
    /// (`virtue.male_guild_sponsor` is itself `social_status`) would be
    /// self-satisfied and could never actually require holding a SEPARATE
    /// item of that category. Every other `Prereq` variant is unaffected —
    /// `Has(id)` etc. never risk this trap, since no shipped item names
    /// itself.
    pub(crate) fn evaluate_for_item(&self, prereq: &Prereq, item_ref: &Id) -> (Tri, bool) {
        evaluate_prereq(prereq, self, 1, Some(item_ref))
    }
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
fn evaluate_prereq(
    prereq: &Prereq,
    ctx: &PrereqCtx,
    depth: usize,
    excluding: Option<&Id>,
) -> (Tri, bool) {
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
        Prereq::All(children) => fold_children(
            children,
            ctx,
            depth,
            excluding,
            Tri::False,
            Tri::False,
            Tri::True,
        ),
        Prereq::Any(children) => fold_children(
            children,
            ctx,
            depth,
            excluding,
            Tri::True,
            Tri::True,
            Tri::False,
        ),
        Prereq::Nor(children) => fold_children(
            children,
            ctx,
            depth,
            excluding,
            Tri::True,
            Tri::False,
            Tri::True,
        ),
        Prereq::Has(id) => {
            if ctx.present_ids.contains(id) {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // Enforced against `is_hermetically_trained` (D56/A0's union of the
        // profile flag with any selection carrying
        // `Effect::ConfersHermeticTraining`), independent of gift_policy.
        Prereq::HermeticallyTrained => match ctx.trained {
            Some(true) => (Tri::True, false),
            Some(false) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
        // Enforced against the profile's explicit `order_member` flag alone —
        // no entity-level override (D56/A0).
        Prereq::OrderMember => match ctx.order {
            Some(true) => (Tri::True, false),
            Some(false) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
        // Enforced against the profile's `is_companion` flag alone (D38): a
        // narrower audience stated on the entry rather than duplicated across
        // every type profile's `forbidden_traits`.
        Prereq::IsCompanion => match ctx.is_companion {
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
        // Static (an item's own category never depends on missing data), so
        // always a definite True/False, never Unknown — matching `Has`'s own
        // shape (design § 3a site 2). Self-excluding (B2/ArMDE:4441): when
        // evaluated for a specific item (`excluding`), an item contributing
        // ONLY via that item's own selection does not count — see
        // `PrereqCtx::evaluate_for_item`'s doc comment.
        Prereq::HasCategory(category) => {
            let held = ctx
                .held_categories
                .get(category.as_str())
                .is_some_and(|contributors| match excluding {
                    Some(self_id) => contributors.iter().any(|id| id != self_id),
                    None => !contributors.is_empty(),
                });
            if held {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
        // D69/X7b-e row 42: an unset age is genuinely unknown (mirrors
        // `House`'s own `None` → `Tri::Unknown`), never a definite failure.
        Prereq::AgeMin(min) => match ctx.age {
            Some(age) if age >= *min => (Tri::True, false),
            Some(_) => (Tri::False, false),
            None => (Tri::Unknown, true),
        },
        // D69/X7b-e row 42/D68.4: static (like `HasCategory`, never Unknown),
        // self-excluding the same way (B2).
        Prereq::HasCategoryAtMagnitude {
            category,
            magnitude,
            item_kind,
        } => {
            let held = ctx
                .held_categories_by_kind
                .get(&(category.clone(), *item_kind))
                .is_some_and(|contributors| {
                    contributors.iter().any(|(id, held_magnitude)| {
                        *held_magnitude >= *magnitude
                            && match excluding {
                                Some(self_id) => id != self_id,
                                None => true,
                            }
                    })
                });
            if held {
                (Tri::True, false)
            } else {
                (Tri::False, false)
            }
        }
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
    excluding: Option<&Id>,
    trigger: Tri,
    short_circuit: Tri,
    all_known: Tri,
) -> (Tri, bool) {
    let mut depended = false;
    let mut saw_unknown = false;
    for child in children {
        let (outcome, dep) = evaluate_prereq(child, ctx, depth + 1, excluding);
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

        // X6a/e7: a per-VALUE incompatibility, active only while its own
        // gate holds for THIS selection — Warped Senses' sight-only clause
        // forbids Keen Vision only when `sense` names sight (D58, ArMDE:7029-7037:
        // a hard error even though the -2 penalty stays text). Reported
        // through the same `incompatible`/pair-dedup machinery as the flat
        // list above.
        for conditional in &item.conditional_incompatible_with {
            if !conditional.gate.holds(selection) {
                continue;
            }
            for incompat_id in &conditional.forbids {
                if selected_ids.contains(incompat_id) {
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
            present_ids,
            trained: None,
            order: None,
            is_companion: None,
            house: None,
            ability_scores,
            art_scores,
            held_categories: BTreeMap::new(),
            age: None,
            held_categories_by_kind: BTreeMap::new(),
        };

        // A single leaf, but evaluated as though it were already past the
        // depth limit — proves the guard fires on `depth`, not on actually
        // walking a deep tree (which would defeat the point of testing this
        // in isolation from the load-time guard).
        let (outcome, depended_on_unknown) = evaluate_prereq(
            &Prereq::HermeticallyTrained,
            &ctx,
            PREREQ_MAX_DEPTH + 1,
            None,
        );
        assert_eq!(outcome, Tri::Unknown);
        assert!(depended_on_unknown);
    }

    #[test]
    fn evaluate_prereq_at_exactly_the_depth_limit_still_evaluates_normally() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids,
            trained: Some(true),
            order: None,
            is_companion: None,
            house: None,
            ability_scores,
            art_scores,
            held_categories: BTreeMap::new(),
            age: None,
            held_categories_by_kind: BTreeMap::new(),
        };

        let (outcome, depended_on_unknown) =
            evaluate_prereq(&Prereq::HermeticallyTrained, &ctx, PREREQ_MAX_DEPTH, None);
        assert_eq!(outcome, Tri::True);
        assert!(!depended_on_unknown);
    }

    /// B1: a sibling validator (B5's category rules, B9's possessed-target
    /// check) needs to build a `PrereqCtx` and evaluate a `Prereq` without
    /// going through `validate_prerequisites`. This is the RED for that
    /// constructor + evaluate method: it fails to *compile* until
    /// `PrereqCtx::build` and `PrereqCtx::evaluate` exist.
    ///
    /// A `Has` target present only in `granted` (never bought into
    /// `entity.selections`) must still evaluate `Tri::True` — the whole point
    /// of the grants-inclusive `present_ids` set (see the doc comment on
    /// `PrereqCtx::present_ids`).
    #[test]
    fn prereq_ctx_is_constructible_by_a_sibling_validator() {
        let ruleset = Ruleset::from_json("test", "1", "[]", "[]").unwrap();
        let entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            crate::RulesetRef::new(Id::new("test"), "1"),
        );
        let selected_ids: BTreeSet<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();
        let granted = vec![Selection::new(Id::new("virtue.heartbeast"))];

        let ctx = PrereqCtx::build(&entity, &ruleset, None, &selected_ids, &granted);

        let (outcome, _) = ctx.evaluate(&Prereq::Has(Id::new("virtue.heartbeast")));
        assert_eq!(outcome, Tri::True);
    }

    /// D38/F-553: `Prereq::IsCompanion` reads the profile's own `is_companion`
    /// flag, independent of `trained`/`order` — true for the plain `companion`
    /// profile.
    #[test]
    fn evaluate_prereq_is_companion_true_when_profile_flag_is_set() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids,
            trained: None,
            order: None,
            is_companion: Some(true),
            house: None,
            ability_scores,
            art_scores,
            held_categories: BTreeMap::new(),
            age: None,
            held_categories_by_kind: BTreeMap::new(),
        };

        let (outcome, depended_on_unknown) = evaluate_prereq(&Prereq::IsCompanion, &ctx, 1, None);
        assert_eq!(outcome, Tri::True);
        assert!(!depended_on_unknown);
    }

    /// A resolved profile whose flag is unset is definitely False — the fact
    /// is the flag, not any particular type id (the shipped `magus`/`grog`
    /// profiles leave it unset; whether `mythic_companion` sets it is a data
    /// question, covered in `tests/data_integrity.rs`).
    #[test]
    fn evaluate_prereq_is_companion_false_when_profile_flag_is_unset() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids,
            trained: None,
            order: None,
            is_companion: Some(false),
            house: None,
            ability_scores,
            art_scores,
            held_categories: BTreeMap::new(),
            age: None,
            held_categories_by_kind: BTreeMap::new(),
        };

        let (outcome, depended_on_unknown) = evaluate_prereq(&Prereq::IsCompanion, &ctx, 1, None);
        assert_eq!(outcome, Tri::False);
        assert!(!depended_on_unknown);
    }

    /// An entity whose type profile cannot be resolved leaves the fact
    /// genuinely unknown, mirroring `HermeticallyTrained`/`OrderMember`.
    #[test]
    fn evaluate_prereq_is_companion_unknown_when_type_unresolved() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids,
            trained: None,
            order: None,
            is_companion: None,
            house: None,
            ability_scores,
            art_scores,
            held_categories: BTreeMap::new(),
            age: None,
            held_categories_by_kind: BTreeMap::new(),
        };

        let (outcome, depended_on_unknown) = evaluate_prereq(&Prereq::IsCompanion, &ctx, 1, None);
        assert_eq!(outcome, Tri::Unknown);
        assert!(depended_on_unknown);
    }

    /// B1/D21/F-502: `Prereq::HasCategory` is a definite True when
    /// `held_categories` names it — never Unknown, matching `Has`'s own
    /// static shape (an item's own category never depends on missing data).
    #[test]
    fn evaluate_prereq_has_category_true_when_held() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids,
            trained: None,
            order: None,
            is_companion: None,
            house: None,
            ability_scores,
            art_scores,
            held_categories: BTreeMap::from([(
                "social_status".to_string(),
                BTreeSet::from([Id::new("virtue.some_status")]),
            )]),
            age: None,
            held_categories_by_kind: BTreeMap::new(),
        };

        let (outcome, depended_on_unknown) =
            evaluate_prereq(&Prereq::HasCategory("social_status".into()), &ctx, 1, None);
        assert_eq!(outcome, Tri::True);
        assert!(!depended_on_unknown);
    }

    /// The refusal half: a category not in `held_categories` is a definite
    /// False.
    #[test]
    fn evaluate_prereq_has_category_false_when_not_held() {
        let (present_ids_owned, ability_scores, art_scores) = empty_ctx();
        let present_ids: BTreeSet<&Id> = present_ids_owned.iter().collect();
        let ctx = PrereqCtx {
            present_ids,
            trained: None,
            order: None,
            is_companion: None,
            house: None,
            ability_scores,
            art_scores,
            held_categories: BTreeMap::new(),
            age: None,
            held_categories_by_kind: BTreeMap::new(),
        };

        let (outcome, depended_on_unknown) =
            evaluate_prereq(&Prereq::HasCategory("social_status".into()), &ctx, 1, None);
        assert_eq!(outcome, Tri::False);
        assert!(!depended_on_unknown);
    }

    /// `PrereqCtx::build` folds `held_categories` grant-aware (bought OR
    /// granted), the same reach `present_ids` gives `Has` — a category held
    /// only via a granted row (never bought) must still satisfy
    /// `HasCategory`.
    #[test]
    fn prereq_ctx_build_folds_held_categories_from_granted_selections_too() {
        let items = r#"[
          { "id": "virtue.some_status", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"] },
          { "id": "flaw.filler_personality", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
        ]"#;
        let ruleset = Ruleset::from_json("test", "1", items, "[]").unwrap();
        let entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            crate::RulesetRef::new(Id::new("test"), "1"),
        );
        let selected_ids: BTreeSet<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();
        let granted = vec![Selection::new(Id::new("virtue.some_status"))];

        let ctx = PrereqCtx::build(&entity, &ruleset, None, &selected_ids, &granted);

        let (outcome, _) = ctx.evaluate(&Prereq::HasCategory("social_status".into()));
        assert_eq!(outcome, Tri::True);
    }

    /// B2/ArMDE:4441: `Prereq::HasCategory`, evaluated via
    /// [`PrereqCtx::evaluate_for_item`] as an item's own prerequisite about
    /// ITSELF, excludes that item's own contribution. An item whose own
    /// category is `social_status` and whose own prerequisite is
    /// `HasCategory("social_status")` — exactly `virtue.male_guild_sponsor`'s
    /// shape — must fail when held ALONE (nothing else supplies the
    /// category), and must succeed once a SECOND, distinct `social_status`
    /// item is held too.
    #[test]
    fn evaluate_for_item_excludes_the_asking_items_own_category() {
        let items = r#"[
          { "id": "virtue.self_ref_status", "kind": "virtue", "classification": "narrative",
            "magnitude": "free", "categories": ["social_status"], "entity_kinds": ["character"] },
          { "id": "virtue.other_status", "kind": "virtue", "classification": "narrative",
            "magnitude": "minor", "categories": ["social_status"], "entity_kinds": ["character"] },
          { "id": "flaw.filler_personality", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] }
        ]"#;
        let ruleset = Ruleset::from_json("test", "1", items, "[]").unwrap();
        let prereq = Prereq::HasCategory("social_status".into());
        let item_ref = Id::new("virtue.self_ref_status");

        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            crate::RulesetRef::new(Id::new("test"), "1"),
        );
        entity.selections = vec![Selection::new(item_ref.clone())];

        // Held ALONE: no OTHER item contributes `social_status`, so the
        // asking item's own row must not satisfy its own prerequisite.
        let selected_ids: BTreeSet<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();
        let ctx = PrereqCtx::build(&entity, &ruleset, None, &selected_ids, &[]);
        let (outcome, _) = ctx.evaluate_for_item(&prereq, &item_ref);
        assert_eq!(
            outcome,
            Tri::False,
            "held alone, the asking item's own category must not satisfy its own HasCategory"
        );

        // Holding a SECOND, distinct `social_status` item satisfies it.
        entity
            .selections
            .push(Selection::new(Id::new("virtue.other_status")));
        let selected_ids: BTreeSet<&Id> = entity.selections.iter().map(|s| &s.item_ref).collect();
        let ctx = PrereqCtx::build(&entity, &ruleset, None, &selected_ids, &[]);
        let (outcome, _) = ctx.evaluate_for_item(&prereq, &item_ref);
        assert_eq!(
            outcome,
            Tri::True,
            "a SEPARATE social_status item must satisfy the asking item's own HasCategory"
        );
    }
}
