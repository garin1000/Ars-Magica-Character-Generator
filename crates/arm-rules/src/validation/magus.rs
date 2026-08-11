//! Magus concerns: House, Mythic type, spells, and the shared XP pool.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;
use crate::types::SpellSelection;

/// Validates a magus's Hermetic House and its specialisation picks. Runs only
/// for a magus type (`is_magus`); no other type has a House.
///
/// - A magus with no House gets a soft `house_unset` warning — belonging to a
///   House is a "should" the troupe can waive, not a hard rule.
/// - Each `Choice` grant's pick (keyed by `choice_key`) must be present and one
///   of the offered options, else `house_choice_unresolved`.
/// - Each `Open` grant's pick must be present (else `house_choice_unresolved`)
///   and satisfy the grant's declarative `GrantConstraint` (kind, magnitude,
///   category allow/deny lists), else `house_grant_constraint`.
/// - A magus with no Flaw in a Hermetic category gets a soft
///   `missing_hermetic_flaw` warning. "Hermetic" is the type's `gift_categories`
///   (data), so no category slug is hardcoded.
///
/// The picks are validated here rather than as ordinary selections because the
/// derived grant is never stored on `entity.selections`; the grant option refs
/// are integrity-checked at load (see `validate_house_refs`).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2855-2861 (the free
/// House Virtue and the recommendation to take a Hermetic Flaw).
pub(crate) fn validate_house(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    // Houses are a magus-only concern; a non-magus (or an unknown type) has none.
    let Some(profile) = type_profile else {
        return;
    };
    if !profile.is_magus {
        return;
    }

    // A magus should take at least one Hermetic Flaw. "Hermetic" is the type's
    // declared gift category (data), so no slug is hardcoded here; skip the
    // guideline entirely when the type names no gift category.
    if !profile.gift_categories.is_empty() {
        let has_hermetic_flaw = entity.selections.iter().any(|s| {
            ruleset.point_items.get(&s.item_ref).is_some_and(|item| {
                item.kind == ItemKind::Flaw && profile.gift_categories.contains(&item.category)
            })
        });
        if !has_hermetic_flaw {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_MISSING_HERMETIC_FLAW,
                // Detected here, but fixed by taking a Hermetic Flaw on the V/F
                // step.
                CreationPhase::VirtuesFlaws,
                args([]),
                None,
            ));
        }
    }

    // No House: a soft warning, and there are no grants to resolve.
    let Some(house_id) = &entity.house else {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_HOUSE_UNSET,
            CreationPhase::HouseSpecialisation,
            args([]),
            None,
        ));
        return;
    };

    // An unknown House id resolves to no grants; nothing further to check.
    let Some(house) = ruleset.house(house_id) else {
        return;
    };

    let unresolved = |choice_key: &str| {
        ValidationIssue::error(
            ValidationIssue::CODE_HOUSE_CHOICE_UNRESOLVED,
            CreationPhase::HouseSpecialisation,
            args([
                ("house", house_id.to_string()),
                ("choice_key", choice_key.to_string()),
            ]),
            None,
        )
    };

    for grant in &house.grants {
        match grant {
            // A fixed grant carries no player choice, so nothing to validate.
            Grant::Fixed { .. } => {}
            Grant::Choice {
                choice_key,
                options,
            } => {
                let pick = entity.house_choices.get(choice_key);
                if !pick.is_some_and(|p| options.contains(p)) {
                    issues.push(unresolved(choice_key));
                }
            }
            Grant::Open {
                choice_key,
                constraint,
            } => {
                let Some(pick) = entity.house_choices.get(choice_key) else {
                    issues.push(unresolved(choice_key));
                    continue;
                };
                if !open_pick_satisfies(pick, constraint, ruleset) {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_HOUSE_GRANT_CONSTRAINT,
                        CreationPhase::HouseSpecialisation,
                        args([
                            ("house", house_id.to_string()),
                            ("choice_key", choice_key.clone()),
                            ("item", pick.item_ref.to_string()),
                        ]),
                        Some(pick.item_ref.clone()),
                    ));
                }
                // An open pick of a parameterized Virtue must name its parameter,
                // exactly like a bought selection. (A `Choice` pick carries the
                // params the ruleset data declares, so it needs no check.)
                validate_selection_parameters(
                    pick,
                    ruleset,
                    CreationPhase::HouseSpecialisation,
                    issues,
                );
            }
        }
    }
}

/// Validates a Mythic Companion's chosen *type* (Devil Child, Faerie Doctor, …):
/// its free-Virtue grants resolve and its required V/F package is present. Gated
/// on the profile's `has_mythic_type` capability flag (never a hardcoded type
/// id), mirroring how [`validate_house`] gates on `is_magus`.
///
/// - No type chosen → `mythic_type_unset` warning (a "should", not a hard rule).
/// - Each `Choice`/`Open` grant pick is resolved from `entity.mythic_choices`
///   (identical machinery to House grants); a missing/off-menu pick →
///   `mythic_choice_unresolved`, an Open pick violating its constraint →
///   `mythic_grant_constraint`.
/// - The required package (fixed Virtues + each required Flaw's default OR a
///   "suitable substitute agreed with the troupe") is checked against the bought
///   `entity.selections`; a missing slot → a non-blocking
///   `mythic_required_trait_missing` warning, so Enforced mode never hard-blocks
///   a legal-with-substitute build. Required Virtues match by full `Selection`
///   (ref + params) so parameterized/duplicated requirements — Nephilim's two
///   distinct Great Characteristics, Puissant Guile — are matched precisely.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2635-2639, 2842-2851.
pub(crate) fn validate_mythic_type(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    // Mythic types are a mythic-companion-only concern; gated on the capability
    // flag so no type id is hardcoded here.
    let Some(profile) = type_profile else {
        return;
    };
    if !profile.has_mythic_type {
        return;
    }

    // No type chosen: a soft warning, and there are no grants/package to resolve.
    let Some(type_id) = &entity.mythic_type else {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_MYTHIC_TYPE_UNSET,
            CreationPhase::MythicType,
            args([]),
            None,
        ));
        return;
    };

    // An unknown type id resolves to nothing; nothing further to check.
    let Some(mtype) = ruleset.mythic_type(type_id) else {
        return;
    };

    // --- Free-Virtue grant picks (Choice/Open), mirroring validate_house. ---
    let unresolved = |choice_key: &str| {
        ValidationIssue::error(
            ValidationIssue::CODE_MYTHIC_CHOICE_UNRESOLVED,
            CreationPhase::MythicType,
            args([
                ("mythic_type", type_id.to_string()),
                ("choice_key", choice_key.to_string()),
            ]),
            None,
        )
    };
    for grant in &mtype.grants {
        match grant {
            Grant::Fixed { .. } => {}
            Grant::Choice {
                choice_key,
                options,
            } => {
                let pick = entity.mythic_choices.get(choice_key);
                if !pick.is_some_and(|p| options.contains(p)) {
                    issues.push(unresolved(choice_key));
                }
            }
            Grant::Open {
                choice_key,
                constraint,
            } => {
                let Some(pick) = entity.mythic_choices.get(choice_key) else {
                    issues.push(unresolved(choice_key));
                    continue;
                };
                if !open_pick_satisfies(pick, constraint, ruleset) {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_MYTHIC_GRANT_CONSTRAINT,
                        CreationPhase::MythicType,
                        args([
                            ("mythic_type", type_id.to_string()),
                            ("choice_key", choice_key.clone()),
                            ("item", pick.item_ref.to_string()),
                        ]),
                        Some(pick.item_ref.clone()),
                    ));
                }
                // Same parameter checks the House Open grant applies.
                validate_selection_parameters(pick, ruleset, CreationPhase::MythicType, issues);
            }
        }
    }

    // --- Required package (non-blocking warnings; substitutes allowed). ---
    let missing = |item: &Id| {
        ValidationIssue::warning(
            ValidationIssue::CODE_MYTHIC_REQUIRED_TRAIT_MISSING,
            // The mythic type demands it, but the trait is bought on the V/F step.
            CreationPhase::VirtuesFlaws,
            args([("item", item.to_string())]),
            Some(item.clone()),
        )
    };
    // Fixed required Virtues: matched by full Selection (ref + params).
    for req in &mtype.required_virtues {
        if !entity.selections.contains(req) {
            issues.push(missing(&req.item_ref));
        }
    }
    // Required Flaws: the default, or any bought selection satisfying the
    // substitute constraint (kind/magnitude/category) — the troupe-substitute
    // allowance.
    for flaw in &mtype.required_flaws {
        let satisfied = entity
            .selections
            .iter()
            .any(|s| open_pick_satisfies(s, &flaw.constraint, ruleset));
        if !satisfied {
            issues.push(missing(&flaw.default.item_ref));
        }
    }
}

/// Validates a magus's spell list: every referenced spell must resolve; the same
/// spell at the same level may not appear twice (different General levels are
/// different spells, Core:12353); a General spell with no chosen level is excluded
/// from the budget and warned; the sum of chosen levels must not exceed the
/// effective spell-levels budget (Core:2215-2216, 2435); and no spell's level may
/// exceed Technique + Form + Intelligence + Magic Theory + 3 (Core:2465).
///
/// The budget and per-spell cap apply only to magi (`profile.is_magus`); a stray
/// spell on a non-magus is ref- and dedup-checked only (spells are magus-only).
pub(crate) fn validate_spells(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let is_magus = type_profile.is_some_and(|p| p.is_magus);
    // Identity is (spell, resolved level, parameter): a parameterized meta-magic
    // Vim spell may be taken once per distinct target (Form) (Core:12353,
    // Core Rules.md:15791-15794).
    let mut seen: BTreeMap<(&Id, Option<u32>, Option<&String>), u32> = BTreeMap::new();

    for sel in &entity.spells {
        let Some(spell) = ruleset.spell(&sel.spell) else {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_SPELL,
                CreationPhase::Spells,
                args([("spell", sel.spell.to_string())]),
                Some(sel.spell.clone()),
            ));
            continue;
        };
        let resolved = crate::effective::resolved_spell_level(sel, ruleset);
        // A General spell (catalogue level None) with no chosen level cannot be
        // budgeted yet — warn, don't block.
        if spell.level.is_none() && sel.level.is_none() {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_SPELL_LEVEL_UNRESOLVED,
                CreationPhase::Spells,
                args([("spell", sel.spell.to_string())]),
                Some(sel.spell.clone()),
            ));
        }

        // A parameterized spell (meta-magic Vim spell whose target (Form) is a
        // selection) requires a chosen value that resolves to the declared
        // domain. Display + identity only — it does NOT change the spell's own
        // Technique/Form (Core Rules.md:15791-15794). Mirrors the virtue/flaw
        // parameter checks in `validation::selections`.
        if let Some(def) = spell.parameters.first() {
            match &sel.parameter {
                None => issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_MISSING_PARAM,
                    CreationPhase::Spells,
                    args([("item", sel.spell.to_string()), ("key", def.key.clone())]),
                    Some(sel.spell.clone()),
                )),
                Some(value) => {
                    let value_id = Id::new(value.as_str());
                    if !super::selections::param_value_resolves(ruleset, def.domain, &value_id) {
                        issues.push(ValidationIssue::error(
                            ValidationIssue::CODE_UNKNOWN_PARAM_VALUE,
                            CreationPhase::Spells,
                            args([
                                ("item", sel.spell.to_string()),
                                ("key", def.key.clone()),
                                ("value", value.clone()),
                                ("domain", def.domain.to_string()),
                            ]),
                            Some(sel.spell.clone()),
                        ));
                    }
                }
            }
        }

        *seen
            .entry((&sel.spell, resolved, sel.parameter.as_ref()))
            .or_insert(0) += 1;

        // Ritual level bounds apply to the resolved learned level regardless of
        // budget: a ritual must be learned at level >= 20, a non-ritual at <= 50
        // (Core Rules.md:12279-12295, :12283). For fixed-level spells this is
        // already enforced at load; it bites here for General spells whose chosen
        // level is illegal.
        if let Some(level) = resolved {
            let ritual_too_low = spell.ritual && level < 20;
            let non_ritual_too_high = !spell.ritual && level > 50;
            if ritual_too_low || non_ritual_too_high {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_SPELL_RITUAL_LEGALITY,
                    CreationPhase::Spells,
                    args([
                        ("spell", sel.spell.to_string()),
                        ("level", level.to_string()),
                    ]),
                    Some(sel.spell.clone()),
                ));
            }
        }

        if is_magus && let Some(level) = resolved {
            let cap =
                crate::effective::spell_level_cap(entity, ruleset, &spell.technique, &spell.form);
            if i64::from(level) > cap {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP,
                    CreationPhase::Spells,
                    args([
                        ("spell", sel.spell.to_string()),
                        ("level", level.to_string()),
                        ("cap", cap.max(0).to_string()),
                    ]),
                    Some(sel.spell.clone()),
                ));
            }
        }

        validate_spell_mastery_abilities(sel, entity, ruleset, issues);
    }

    for ((spell, _level, _param), count) in seen {
        if count > 1 {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_DUPLICATE_SPELL,
                CreationPhase::Spells,
                args([("spell", spell.to_string()), ("count", count.to_string())]),
                Some(spell.clone()),
            ));
        }
    }

    if is_magus {
        let base = crate::effective::spell_levels_base(entity, type_profile);
        let budget = crate::effective::spell_levels_budget(base, entity, ruleset);
        let used = crate::effective::spell_levels_used(entity, ruleset);
        if used > budget {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_OVER_SPELL_LEVELS,
                CreationPhase::Spells,
                args([
                    ("used", used.to_string()),
                    ("budget", budget.to_string()),
                    ("over", (used - budget).to_string()),
                ]),
                None,
            ));
        }
    }
}

/// Validates the Spell Mastery special abilities chosen for one spell:
///
/// - Every chosen id must resolve against the mastery-ability catalogue
///   (referential integrity — an unknown id fails loudly, CLAUDE.md).
/// - The count of chosen abilities may not exceed the spell's *effective* mastery
///   score: for every level in the Mastery Ability the maga may choose one
///   special ability (Core:9524-9526).
/// - A non-repeatable ability may be chosen only once for the same spell; only
///   Precise, Quick, and Quiet Casting may repeat (Core:9572, :9576, :9580).
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:9524-9592.
fn validate_spell_mastery_abilities(
    sel: &SpellSelection,
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    if sel.mastery_abilities.is_empty() {
        return;
    }

    // One special ability per effective mastery level (Core:9524-9526).
    let effective = crate::effective::effective_spell_mastery(sel, entity, ruleset);
    if sel.mastery_abilities.len() > usize::from(effective) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_TOO_MANY_MASTERY_ABILITIES,
            CreationPhase::Spells,
            args([
                ("spell", sel.spell.to_string()),
                ("chosen", sel.mastery_abilities.len().to_string()),
                ("mastery", effective.to_string()),
            ]),
            Some(sel.spell.clone()),
        ));
    }

    // Referential integrity + per-ability occurrence counts (unknown ids are not
    // counted, so an unknown id is never also reported as a duplicate).
    let mut counts: BTreeMap<&Id, u32> = BTreeMap::new();
    for ability_id in &sel.mastery_abilities {
        if ruleset.spell_mastery_ability(ability_id).is_some() {
            *counts.entry(ability_id).or_insert(0) += 1;
        } else {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_MASTERY_ABILITY,
                CreationPhase::Spells,
                args([
                    ("spell", sel.spell.to_string()),
                    ("ability", ability_id.to_string()),
                ]),
                Some(sel.spell.clone()),
            ));
        }
    }

    // A non-repeatable ability may not appear more than once for the same spell.
    for (ability_id, count) in counts {
        let repeatable = ruleset
            .spell_mastery_ability(ability_id)
            .is_some_and(|a| a.repeatable);
        if count > 1 && !repeatable {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_DUPLICATE_MASTERY_ABILITY,
                CreationPhase::Spells,
                args([
                    ("spell", sel.spell.to_string()),
                    ("ability", ability_id.to_string()),
                    ("count", count.to_string()),
                ]),
                Some(sel.spell.clone()),
            ));
        }
    }
}

/// Validates the experience pools: Abilities and Arts are bought from the shared
/// general bank (`Entity::xp_pool`) plus any restricted grants (Educated/Warrior/
/// Privileged), each Affinity-reduced. Feasibility is a max-flow solve over the
/// general pool + restricted pools; an infeasible allocation overspends. Reported
/// as an error rather than blocked: direct-entry allows the illegal state and
/// surfaces it (the M4 wizard blocks the spend up front). Leftover restricted XP
/// the rules waste raises a non-blocking warning.
pub(crate) fn validate_xp_pool(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let allocation = crate::effective::xp_allocation(entity, ruleset);
    if allocation.total_demand > allocation.max_flow {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_NOT_ENOUGH_XP,
            // The pool spans Abilities and Arts, but the Abilities step is where
            // the XP bar lives and where most of the spending happens.
            CreationPhase::Abilities,
            args([
                ("spent", allocation.total_demand.to_string()),
                // The demand now draws several pools (general + restricted-ability +
                // Spell-Mastery), so `pool` is the total the allocation *can* fund
                // (`max_flow`), not the raw general pool — keeping `spent - pool ==
                // shortfall` accurate across all funding sources.
                ("pool", allocation.max_flow.to_string()),
                (
                    "shortfall",
                    (allocation.total_demand - allocation.max_flow).to_string(),
                ),
            ]),
            None,
        ));
    }
    for pool in &allocation.restricted {
        if pool.used < pool.amount {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_RESTRICTED_XP_UNSPENT,
                CreationPhase::Abilities,
                args([
                    ("amount", pool.amount.to_string()),
                    ("used", pool.used.to_string()),
                    ("unspent", (pool.amount - pool.used).to_string()),
                ]),
                None,
            ));
        }
    }
}
