//! Magus concerns: House, Mythic type, spells, and the shared XP pool.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;
use crate::effective::XpPoolOrigin;
use crate::life_stage::{AbilityRequirementKind, magus_minimum_abilities};
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

    for outcome in grant_pick_outcomes(&house.grants, &entity.house_choices, ruleset) {
        match outcome {
            GrantPickOutcome::Resolved => {}
            GrantPickOutcome::Unresolved { choice_key } => issues.push(ValidationIssue::error(
                ValidationIssue::CODE_HOUSE_CHOICE_UNRESOLVED,
                CreationPhase::HouseSpecialisation,
                args([
                    ("house", house_id.to_string()),
                    ("choice_key", choice_key.to_string()),
                ]),
                None,
            )),
            GrantPickOutcome::OpenPick {
                choice_key,
                pick,
                satisfies_constraint,
            } => {
                if !satisfies_constraint {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_HOUSE_GRANT_CONSTRAINT,
                        CreationPhase::HouseSpecialisation,
                        args([
                            ("house", house_id.to_string()),
                            ("choice_key", choice_key.to_string()),
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

/// One `Grant`'s outcome against the player's stored picks — everything
/// [`grant_pick_outcomes`] can determine without knowing which issue code or
/// `CreationPhase` the caller files it under. Deliberately carries no
/// [`ValidationIssue`]: the emit sites stay in [`validate_house`] and
/// [`validate_mythic_type`] with their own literal `CODE_*`/`CreationPhase`
/// constants, which is what keeps
/// `every_issue_emit_site_names_a_phase_the_table_lists`'s static scan (source
/// text, not runtime values) able to verify each code/phase pair against the
/// contract table.
enum GrantPickOutcome<'a> {
    /// `Fixed` (no player choice), or a `Choice` pick present and on-menu.
    /// Nothing to report.
    Resolved,
    /// A `Choice` pick absent/off-menu, or an `Open` pick absent.
    Unresolved { choice_key: &'a str },
    /// An `Open` pick is present: the caller must run the same parameter
    /// checks a bought selection gets, and — when `satisfies_constraint` is
    /// `false` — also report the constraint violation.
    OpenPick {
        choice_key: &'a str,
        pick: &'a Selection,
        satisfies_constraint: bool,
    },
}

/// Resolves one `grants` list's `Choice`/`Open` picks against `picks`, in
/// order — the walk [`validate_house`] and [`validate_mythic_type`] used to
/// hand-roll almost identically (V50: a bug fixed on one side had to be
/// separately remembered on the other). Both now read this one list and
/// report each [`GrantPickOutcome`] with their own issue code/phase, so the
/// walk itself — which pick a `Choice`/`Open` grant resolves to, and whether
/// an `Open` pick satisfies its constraint — exists exactly once.
fn grant_pick_outcomes<'a>(
    grants: &'a [Grant],
    picks: &'a BTreeMap<String, Selection>,
    ruleset: &Ruleset,
) -> Vec<GrantPickOutcome<'a>> {
    grants
        .iter()
        .map(|grant| match grant {
            Grant::Fixed { .. } => GrantPickOutcome::Resolved,
            Grant::Choice {
                choice_key,
                options,
            } => {
                let pick = picks.get(choice_key);
                if pick.is_some_and(|p| options.contains(p)) {
                    GrantPickOutcome::Resolved
                } else {
                    GrantPickOutcome::Unresolved { choice_key }
                }
            }
            Grant::Open {
                choice_key,
                constraint,
            } => match picks.get(choice_key) {
                None => GrantPickOutcome::Unresolved { choice_key },
                Some(pick) => GrantPickOutcome::OpenPick {
                    choice_key,
                    pick,
                    satisfies_constraint: open_pick_satisfies(pick, constraint, ruleset),
                },
            },
        })
        .collect()
}

/// Validates the Abilities the Order demands of every magus.
///
/// > Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory 1,
/// > Latin 1. Characters with lower scores would not be admitted to the Order.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2437, with the
/// `#### Hermetic Magi Recommended Minimum Abilities` of `:2451-2461` as the second,
/// advisory half.
///
/// - `magus_minimum_ability` (error) — one per unmet minimum. "Would not be admitted
///   to the Order" is a hard bar, and the passage is **unconditional on the funding
///   mode**, which is why this lives here rather than in `validation/life_stage.rs`:
///   that validator returns early without a life-stage plan, while a magus built from
///   a flat experience pool is held to `:2437` just the same.
/// - `magus_recommended_ability` (warning) — one per unmet recommendation. `:2451`
///   calls them *recommended*, and the consequences `:2437` spells out ("unable to
///   read the books of the Order", "cannot set up his own laboratory") describe a weak
///   magus, not an illegal one.
///
/// The checklist itself comes from [`magus_minimum_abilities`], the single reading of
/// both passages — so this validator and the checklist a UI shows cannot disagree.
/// Nothing is enforced for a non-magus, nor for a ruleset that states no minimums:
/// the requirements are data (`rules/core/life_stages.json`), never a list in Rust.
pub(crate) fn validate_magus_minimum_abilities(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for row in magus_minimum_abilities(entity, ruleset) {
        if row.met {
            continue;
        }
        let mut args = args([
            ("ability", row.ability.to_string()),
            ("min", row.min_score.to_string()),
            ("score", row.score.to_string()),
        ]);
        // The rules say "Latin 1" while the check is "any Dead Language 1", so the
        // finding carries the rules' own exemplar and the UI names it beside the
        // Ability. Emitted only where the data states one, so a requirement without
        // an exemplar reads exactly as before.
        if let Some(exemplar) = &row.exemplar {
            args.insert("exemplar".to_string(), exemplar.clone());
        }
        let context = Some(row.ability.clone());
        issues.push(match row.requirement {
            AbilityRequirementKind::Required => ValidationIssue::error(
                ValidationIssue::CODE_MAGUS_MINIMUM_ABILITY,
                CreationPhase::Abilities,
                args,
                context,
            ),
            AbilityRequirementKind::Recommended => ValidationIssue::warning(
                ValidationIssue::CODE_MAGUS_RECOMMENDED_ABILITY,
                CreationPhase::Abilities,
                args,
                context,
            ),
        });
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
    for outcome in grant_pick_outcomes(&mtype.grants, &entity.mythic_choices, ruleset) {
        match outcome {
            GrantPickOutcome::Resolved => {}
            GrantPickOutcome::Unresolved { choice_key } => issues.push(ValidationIssue::error(
                ValidationIssue::CODE_MYTHIC_CHOICE_UNRESOLVED,
                CreationPhase::MythicType,
                args([
                    ("mythic_type", type_id.to_string()),
                    ("choice_key", choice_key.to_string()),
                ]),
                None,
            )),
            GrantPickOutcome::OpenPick {
                choice_key,
                pick,
                satisfies_constraint,
            } => {
                if !satisfies_constraint {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_MYTHIC_GRANT_CONSTRAINT,
                        CreationPhase::MythicType,
                        args([
                            ("mythic_type", type_id.to_string()),
                            ("choice_key", choice_key.to_string()),
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
/// different spells, Ars Magica - Definitive Edition (Core Rules).md:12353); a General spell with no chosen level is excluded
/// from the budget and warned; the sum of chosen levels must not exceed the
/// effective spell-levels budget (Ars Magica - Definitive Edition (Core Rules).md:2215-2216, 2435, and the levels bought out of
/// the years past the Gauntlet, `:2471`); and no spell's level may
/// exceed Technique + Form + Intelligence + Magic Theory + 3 (Ars Magica - Definitive Edition (Core Rules).md:2465).
///
/// The budget and per-spell cap apply only to magi (`profile.is_magus`); a stray
/// spell on a non-magus is ref- and dedup-checked only (spells are magus-only).
///
/// V51: this used to be one ~175-line function doing all 7 checks inline.
/// Split into named sub-checks — one per job, matching the granularity
/// [`validate_spell_mastery_abilities`] and [`validate_xp_pool`] already use
/// in this file — so each check reads and tests on its own; this function is
/// now purely the orchestration (loop over selections, dispatch each
/// per-spell check, then the two aggregate checks). Pure code motion: same
/// issue codes, same `issues` ordering (per-spell checks in selection order,
/// then duplicate detection, then the budget check — pinned by
/// `spell_issues_are_emitted_per_spell_then_duplicate_then_budget` in
/// `validation/mod.rs`).
pub(crate) fn validate_spells(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let is_magus = type_profile.is_some_and(|p| p.is_magus);
    // Identity is (spell, resolved level, parameter): a parameterized meta-magic
    // Vim spell may be taken once per distinct target (Form) (Ars Magica - Definitive Edition (Core Rules).md:12353,
    // Ars Magica - Definitive Edition (Core Rules).md:15791-15794).
    let mut seen: BTreeMap<(&Id, Option<u32>, Option<&String>), u32> = BTreeMap::new();

    for sel in &entity.spells {
        let Some(spell) = validate_spell_ref(sel, ruleset, issues) else {
            continue;
        };
        let resolved = crate::effective::resolved_spell_level(sel, ruleset);

        validate_spell_level_unresolved(sel, spell, issues);
        validate_spell_parameter(sel, spell, ruleset, issues);

        *seen
            .entry((&sel.spell, resolved, sel.parameter.as_ref()))
            .or_insert(0) += 1;

        validate_spell_ritual_legality(sel, spell, resolved, issues);
        if is_magus {
            validate_spell_level_cap(entity, ruleset, sel, spell, resolved, issues);
        }
        validate_spell_mastery_abilities(sel, entity, ruleset, issues);
    }

    validate_duplicate_spells(seen, issues);

    if is_magus {
        validate_spell_levels_budget(entity, ruleset, type_profile, issues);
    }
}

/// Job 1/7: resolves `sel`'s spell against the catalogue; on failure reports
/// `unknown_spell` and returns `None` (`validate_spells` stops there for that
/// selection — every other check needs a resolved [`crate::spell::Spell`]).
fn validate_spell_ref<'a>(
    sel: &SpellSelection,
    ruleset: &'a Ruleset,
    issues: &mut Vec<ValidationIssue>,
) -> Option<&'a crate::spell::Spell> {
    let spell = ruleset.spell(&sel.spell);
    if spell.is_none() {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_UNKNOWN_SPELL,
            CreationPhase::Spells,
            args([("spell", sel.spell.to_string())]),
            Some(sel.spell.clone()),
        ));
    }
    spell
}

/// Job 2/7: a General spell (catalogue level `None`) with no chosen level
/// cannot be budgeted yet — warn, don't block.
fn validate_spell_level_unresolved(
    sel: &SpellSelection,
    spell: &crate::spell::Spell,
    issues: &mut Vec<ValidationIssue>,
) {
    if spell.level.is_none() && sel.level.is_none() {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_SPELL_LEVEL_UNRESOLVED,
            CreationPhase::Spells,
            args([("spell", sel.spell.to_string())]),
            Some(sel.spell.clone()),
        ));
    }
}

/// Job 3/7: a parameterized spell (meta-magic Vim spell whose target (Form) is
/// a selection) requires a chosen value that resolves to the declared domain.
/// Display + identity only — it does NOT change the spell's own
/// Technique/Form (Ars Magica - Definitive Edition (Core Rules).md:15791-15794).
/// Mirrors the virtue/flaw parameter checks in `validation::selections`.
fn validate_spell_parameter(
    sel: &SpellSelection,
    spell: &crate::spell::Spell,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(def) = spell.parameters.first() else {
        return;
    };
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

/// Job 5/7: ritual level bounds apply to the resolved learned level regardless
/// of budget: a ritual must be learned at level >= `RITUAL_MIN_LEVEL`, a
/// non-ritual at <= 50. For fixed-level spells this is already enforced at
/// load; it bites here for General spells whose chosen level is illegal.
///
/// The floor comes from the shared constant rather than a literal: the UI
/// used to restate it too (audit finding VA2), and one number in two places
/// is one number that can drift.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:12279-12295,
/// :12285 ("Formulaic and Spontaneous spells may not have a level greater
/// than 50" — the exact non-Ritual ceiling this checks). An earlier
/// version of this comment cited :12283 (the Year-duration restriction,
/// unrelated), then a later pass "corrected" it to :12291 (a discretionary
/// note that spectacular effects "will normally be over level 50, and thus
/// Rituals anyway" — a design rationale, not the numeric rule itself).
/// :12285 is the line that actually states the "> 50" ceiling.
fn validate_spell_ritual_legality(
    sel: &SpellSelection,
    spell: &crate::spell::Spell,
    resolved: Option<u32>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(level) = resolved else {
        return;
    };
    // `RITUAL_MIN_LEVEL` is `u8` (it also feeds the ruleset surface the UI
    // reads); resolved spell levels are `u32`. Widen explicitly rather than
    // cast.
    let ritual_min = u32::from(crate::spell::RITUAL_MIN_LEVEL);
    let ritual_too_low = spell.ritual && level < ritual_min;
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

/// Job 6/7: no spell's level may exceed Technique + Form + Intelligence +
/// Magic Theory + 3 (Ars Magica - Definitive Edition (Core Rules).md:2465).
/// Magi only — the caller gates on `is_magus`.
fn validate_spell_level_cap(
    entity: &Entity,
    ruleset: &Ruleset,
    sel: &SpellSelection,
    spell: &crate::spell::Spell,
    resolved: Option<u32>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(level) = resolved else {
        return;
    };
    let cap = crate::effective::spell_level_cap(entity, ruleset, &spell.technique, &spell.form);
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

/// Job 4/7: the same spell at the same resolved level (and, for a
/// parameterized meta-magic Vim spell, the same parameter) may not appear
/// twice — different General levels are different spells
/// (Ars Magica - Definitive Edition (Core Rules).md:12353). Runs once, after
/// the per-spell loop has built `seen`.
fn validate_duplicate_spells(
    seen: BTreeMap<(&Id, Option<u32>, Option<&String>), u32>,
    issues: &mut Vec<ValidationIssue>,
) {
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
}

/// Job 7/7: the sum of chosen spell levels must not exceed the effective
/// spell-levels budget (Ars Magica - Definitive Edition (Core Rules).md:2215-2216,
/// 2435, and the levels bought out of the years past the Gauntlet, `:2471`).
/// Magi only — the caller gates on `is_magus`.
fn validate_spell_levels_budget(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
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
    } else if used < budget {
        // guided-creation-review-2026-08 #30, the spell-levels half. Same
        // budget, same step, opposite direction — and read off the same
        // `spell_levels_budget` above, so the pair cannot disagree about what
        // the budget is.
        //
        // Factual, exactly as `general_xp_unspent`. "Take 120 levels of spells"
        // (Ars Magica - Definitive Edition (Core Rules).md:2215) grants the
        // levels; the source nowhere says unused levels are lost, so the
        // message reports the count and asserts nothing further.
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_SPELL_LEVELS_UNSPENT,
            CreationPhase::Spells,
            args([
                ("used", used.to_string()),
                ("budget", budget.to_string()),
                ("unspent", (budget - used).to_string()),
            ]),
            None,
        ));
    }
}

/// Validates the Spell Mastery special abilities chosen for one spell:
///
/// - Every chosen id must resolve against the mastery-ability catalogue
///   (referential integrity — an unknown id fails loudly, CLAUDE.md).
/// - The count of chosen abilities may not exceed the spell's *effective* mastery
///   score: for every level in the Mastery Ability the maga may choose one
///   special ability (Ars Magica - Definitive Edition (Core Rules).md:9524-9526).
/// - A non-repeatable ability may be chosen only once for the same spell; only
///   Precise, Quick, and Quiet Casting may repeat (Ars Magica - Definitive Edition (Core Rules).md:9572, :9576, :9580).
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

    // One special ability per effective mastery level (Ars Magica - Definitive Edition (Core Rules).md:9524-9526).
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
    // This is a malformed-input rejection, not a rules judgment, but it stays
    // paired with the computation it protects rather than living beside the
    // referential-integrity checks at the top of `validate`
    // (`validate_known_refs` and friends): those check that the entity's own
    // references resolve, a property every other validator can then assume,
    // whereas this one exists solely to gate the flow solve itself and has no
    // meaning apart from it.
    //
    // `checked_xp_allocation` (audit finding K1, round 2) is the single place
    // the node-count bound is checked; this used to recompute
    // `xp_solve_scale`/`MAX_XP_SOLVE_NODES` independently right before calling
    // the (then-`pub`) `xp_allocation` — two places asserting the same bound
    // with nothing keeping them in sync, which is exactly how K1 happened
    // (two *other* callers grew directly against the unguarded function).
    // Folding both steps into one call removes that duplication.
    let allocation = match crate::effective::checked_xp_allocation(entity, ruleset) {
        Ok(allocation) => allocation,
        Err(bound) => {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_XP_SOLVE_BOUND_EXCEEDED,
                // The pool spans Abilities and Arts, but the Abilities step is
                // where the XP bar lives, matching `not_enough_xp` below.
                CreationPhase::Abilities,
                args([
                    ("nodes", bound.nodes.to_string()),
                    ("limit", bound.limit.to_string()),
                    ("spends", bound.spends.to_string()),
                    ("pools", bound.flow_pools.to_string()),
                ]),
                None,
            ));
            return;
        }
    };
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
    } else if allocation.general_used < allocation.general_pool {
        // guided-creation-review-2026-08 #30: the underspend counterpart of the
        // error above. Its sibling on the restricted pools has existed all along,
        // so leaving 200 general points in hand was the one budget the engine said
        // nothing about while a single unspent Characteristic point warned.
        //
        // The GENERAL remainder alone. The restricted pools are reported one by one
        // in the loop below, and summing them in here as well would tell a
        // life-stage character about the same points twice.
        //
        // `else if`, not a second `if`: an infeasible allocation's `general_used` is
        // whatever the flow solve could place, so an overspent character would
        // otherwise be told it has spent too much AND has points left over.
        //
        // The wording this feeds is deliberately just a count. The pool's own size
        // is a rule (Ars Magica - Definitive Edition (Core Rules).md:2213-2216 —
        // 75 + 45 in childhood, 15 a year in later life, 240 for apprenticeship, 30
        // a year after the Gauntlet), but NOTHING in the source says unspent general
        // experience is lost. `restricted_xp_unspent` may say "wasted" because
        // childhood's blocks are spend-or-lose; this one may not, and must not.
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_GENERAL_XP_UNSPENT,
            // The Abilities step, where the XP bar lives — the same step
            // `not_enough_xp` above is filed under, since it is the same budget.
            CreationPhase::Abilities,
            args([
                ("pool", allocation.general_pool.to_string()),
                ("used", allocation.general_used.to_string()),
                (
                    "unspent",
                    (allocation.general_pool - allocation.general_used).to_string(),
                ),
            ]),
            None,
        ));
    }
    for pool in &allocation.restricted {
        if pool.used < pool.amount {
            let (origin_kind, origin) = origin_args(&pool.origin);
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_RESTRICTED_XP_UNSPENT,
                // A restricted pool comes from the funding plan (a life-stage block
                // or a Virtue's grant), so the step that can change what it is worth
                // is the `experience` one — the Abilities step spends it but cannot
                // resize it.
                CreationPhase::Experience,
                args([
                    ("amount", pool.amount.to_string()),
                    ("used", pool.used.to_string()),
                    ("unspent", (pool.amount - pool.used).to_string()),
                    ("origin_kind", origin_kind.to_string()),
                    ("origin", origin),
                ]),
                None,
            ));
        }
    }
}

/// The `(origin_kind, origin)` pair that names an unspent pool: which sort of thing
/// granted it, and that thing's machine name — an item id, or a life-stage block
/// slug ([`LifeStageBlock`]'s `Display`).
///
/// Naming the pool is the whole reason [`RestrictedXpPool::origin`] exists: a
/// life-stage character leaves childhood's two blocks (75 and 45) unspent
/// independently, and two warnings differing only in their numbers cannot tell a
/// reader which block to go and spend.
///
/// Deliberately **not** a message and not a Fluent key: the engine carries no
/// user-facing string, so the frontend resolves an `item` through the ruleset's own
/// i18n and a `life_stage` through its `xp-pool-<block>` catalogue.
fn origin_args(origin: &XpPoolOrigin) -> (&'static str, String) {
    match origin {
        XpPoolOrigin::Item { item } => ("item", item.to_string()),
        XpPoolOrigin::LifeStage { block } => ("life_stage", block.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection};
    use crate::validation::{IssueSeverity, ValidationIssue, ValidationResult, validate};
    use crate::{CreationPhase, Ruleset, RulesetSources};

    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] }
    ]"#;
    const TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality"], "creation_phases": [] },
      { "id": "magus", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality"], "is_magus": true,
        "creation_phases": [] }
    ]"#;
    const ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 },
        { "score": 3, "total_xp": 30 },
        { "score": 4, "total_xp": 50 }
      ],
      "abilities": [
        { "id": "ability.artes_liberales", "category": "academic" },
        { "id": "ability.dead_language", "category": "academic", "parameter": "language" },
        { "id": "ability.living_language", "category": "general", "parameter": "language" },
        { "id": "ability.magic_theory", "category": "arcane" },
        { "id": "ability.parma_magica", "category": "arcane" },
        { "id": "ability.penetration", "category": "arcane" },
        { "id": "ability.philosophiae", "category": "academic" },
        { "id": "ability.swim", "category": "general" }
      ]
    }"#;
    /// The shipped Hermetic requirements: the three minimums of Ars Magica - Definitive Edition (Core Rules).md:2437 and
    /// the four recommendations of `:2451-2461`, priced to 90 (5 + 50 + 30 + 5).
    const LIFE_STAGES: &str = r#"{
      "apprenticeship": {
        "years": 15,
        "xp": 240,
        "minimum_abilities": [
          { "ability": "ability.dead_language", "min_score": 1 },
          { "ability": "ability.magic_theory", "min_score": 1 },
          { "ability": "ability.parma_magica", "min_score": 1 }
        ],
        "recommended_abilities": [
          { "ability": "ability.artes_liberales", "min_score": 1 },
          { "ability": "ability.dead_language", "min_score": 4 },
          { "ability": "ability.magic_theory", "min_score": 3 },
          { "ability": "ability.parma_magica", "min_score": 1 }
        ],
        "recommended_xp": 90
      },
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": ["ability.swim"]
      },
      "later_life": { "xp_per_year": 15 },
      "post_apprenticeship": {
        "lab_season_cost": 10,
        "max_charged_lab_seasons_per_year": 3,
        "points_per_year": 30
      }
    }"#;

    fn rs() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            life_stages: Some(LIFE_STAGES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// The same catalogue with no life-stage rules at all — a ruleset that states no
    /// minimums, so none are enforced. (A ruleset shipping life stages *and* magi is
    /// obliged to declare an apprenticeship, so this is the only shape that can lack
    /// the requirements.)
    fn rs_without_life_stages() -> Ruleset {
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: ITEMS,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// A directly-entered character of `type_id` with the given bought scores. No
    /// life-stage plan: `:2437` is unconditional on how the experience was funded.
    fn character(type_id: &str, scores: Vec<(&str, Option<&str>, u8)>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new(type_id),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.xp_pool = 500;
        entity.ability_scores = scores
            .into_iter()
            .map(|(ability, parameter, score)| AbilityScore {
                ability: Id::new(ability),
                parameter: parameter.map(str::to_string),
                score,
                specialty: None,
            })
            .collect();
        entity
    }

    fn issues_with(result: &ValidationResult, code: &str) -> Vec<ValidationIssue> {
        result
            .issues
            .iter()
            .filter(|issue| issue.code == code)
            .cloned()
            .collect()
    }

    /// > Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory
    /// > 1, Latin 1. Characters with lower scores would not be admitted to the Order.
    ///
    /// Source: Ars Magica - Definitive Edition (Core Rules).md:2437. An **error**, and
    /// unconditional: it says nothing about how the experience was earned, so a magus
    /// built from a flat pool is held to it exactly as a guided one is. One finding per
    /// unmet requirement, each naming the Ability it is about.
    #[test]
    fn a_magus_below_the_minimum_abilities_is_an_error() {
        let rs = rs();
        let result = validate(&character("magus", vec![]), &rs);

        let errors = issues_with(&result, ValidationIssue::CODE_MAGUS_MINIMUM_ABILITY);
        assert_eq!(
            errors
                .iter()
                .map(|issue| {
                    (
                        issue.args.get("ability").cloned().unwrap_or_default(),
                        issue.args.get("min").cloned().unwrap_or_default(),
                        issue.args.get("score").cloned().unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                ("ability.dead_language".to_string(), "1".into(), "0".into()),
                ("ability.magic_theory".to_string(), "1".into(), "0".into()),
                ("ability.parma_magica".to_string(), "1".into(), "0".into()),
            ]
        );
        for issue in &errors {
            assert_eq!(issue.severity, IssueSeverity::Error);
            assert_eq!(issue.phase, CreationPhase::Abilities);
            assert_eq!(
                issue.context.as_ref().map(Id::to_string),
                issue.args.get("ability").cloned(),
                "the finding targets the Ability row: {issue:?}"
            );
        }

        // The recommended set is advice, so falling short of it only warns.
        let warnings = issues_with(&result, ValidationIssue::CODE_MAGUS_RECOMMENDED_ABILITY);
        assert_eq!(warnings.len(), 4);
        for issue in &warnings {
            assert_eq!(issue.severity, IssueSeverity::Warning);
            assert_eq!(issue.phase, CreationPhase::Abilities);
        }

        // A magus meeting the minimums raises neither code for them; the Darius
        // package (`:2441`, `:2449`) meets all seven rows.
        let darius = character(
            "magus",
            vec![
                ("ability.dead_language", Some("Latin"), 4),
                ("ability.magic_theory", None, 4),
                ("ability.artes_liberales", None, 3),
                ("ability.parma_magica", None, 1),
            ],
        );
        let result = validate(&darius, &rs);
        assert!(
            issues_with(&result, ValidationIssue::CODE_MAGUS_MINIMUM_ABILITY).is_empty(),
            "{:?}",
            issues_with(&result, ValidationIssue::CODE_MAGUS_MINIMUM_ABILITY)
        );
        assert!(issues_with(&result, ValidationIssue::CODE_MAGUS_RECOMMENDED_ABILITY).is_empty());

        // A companion is not admitted to the Order in the first place.
        let result = validate(&character("companion", vec![]), &rs);
        assert!(issues_with(&result, ValidationIssue::CODE_MAGUS_MINIMUM_ABILITY).is_empty());
        assert!(issues_with(&result, ValidationIssue::CODE_MAGUS_RECOMMENDED_ABILITY).is_empty());

        // A ruleset that states no minimums enforces none — the minimums are data.
        let result = validate(&character("magus", vec![]), &rs_without_life_stages());
        assert!(issues_with(&result, ValidationIssue::CODE_MAGUS_MINIMUM_ABILITY).is_empty());
        assert!(issues_with(&result, ValidationIssue::CODE_MAGUS_RECOMMENDED_ABILITY).is_empty());
    }

    /// K3 layer 1: a hostile save can carry an unbounded `ability_scores` array
    /// (deserialized straight off disk, no length cap in `types.rs`), which
    /// would otherwise force `xp_allocation` to build a multi-hundred-MB `n x n`
    /// matrix. `validate_xp_pool` must refuse before ever calling it, with a
    /// structured issue naming the counts — not the solver's `assert!`, which
    /// stays only as the unbypassable backstop (layer 2) for any caller that
    /// skips validation.
    #[test]
    fn a_pathological_number_of_ability_scores_is_rejected_before_the_solver_runs() {
        let rs = rs();
        let scores: Vec<(&str, Option<&str>, u8)> =
            vec![("ability.artes_liberales", None, 1); 3000];
        let entity = character("companion", scores);

        let result = validate(&entity, &rs);

        let errors = issues_with(&result, ValidationIssue::CODE_XP_SOLVE_BOUND_EXCEEDED);
        assert_eq!(errors.len(), 1, "{:?}", result.issues);
        let issue = &errors[0];
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(issue.phase, CreationPhase::Abilities);
        assert_eq!(issue.args.get("nodes").cloned(), Some("3003".to_string()));
        assert_eq!(issue.args.get("limit").cloned(), Some("2048".to_string()));
        assert_eq!(issue.args.get("spends").cloned(), Some("3000".to_string()));
        assert_eq!(issue.args.get("pools").cloned(), Some("0".to_string()));

        // The ordinary xp-pool feasibility check must not have run at all — no
        // `not_enough_xp` noise stacked on top of an entity refused outright.
        assert!(issues_with(&result, ValidationIssue::CODE_NOT_ENOUGH_XP).is_empty());
    }

    /// The bound guards a hostile save's array sizes, never a legal character's:
    /// an implausibly long-lived archmage with hundreds of bought scores — an
    /// order of magnitude below the solve bound — must still validate normally.
    #[test]
    fn a_legal_but_extreme_character_is_not_rejected_by_the_solve_bound() {
        let rs = rs();
        let scores: Vec<(&str, Option<&str>, u8)> = vec![("ability.artes_liberales", None, 1); 500];
        let mut entity = character("companion", scores);
        entity.xp_pool = 100_000;

        let result = validate(&entity, &rs);

        assert!(issues_with(&result, ValidationIssue::CODE_XP_SOLVE_BOUND_EXCEEDED).is_empty());
    }

    /// A ruleset carrying `pool_count` distinct restricted-XP-granting virtues
    /// eligible for the fixture's one real ability — puts node mass on
    /// `flow_pools` rather than `spends` so the exact-boundary tests below stay
    /// fast. The flow solve's cost is dominated by the number of augmenting
    /// BFS calls, which tracks `spends` (the sink-side bottleneck) and not
    /// `flow_pools`; a spends-heavy fixture at `n` near 2048 measurably takes
    /// on the order of a minute in a debug build (see
    /// `effective/xp.rs`'s `assert!` comment), which a boundary test must
    /// avoid.
    fn rs_with_dead_pools(pool_count: usize) -> Ruleset {
        let mut items = String::from(ITEMS.trim_end().trim_end_matches(']'));
        for i in 0..pool_count {
            items.push(',');
            items.push_str(&format!(
                r#"{{"id":"virtue.dead_pool_{i}","kind":"virtue","classification":"narrative","magnitude":"minor","category":"general","effects":[{{"type":"restricted_ability_xp","amount":10,"abilities":["ability.artes_liberales"]}}]}}"#
            ));
        }
        items.push(']');
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: &items,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            life_stages: Some(LIFE_STAGES),
            ..RulesetSources::default()
        })
        .unwrap()
    }

    /// A companion holding one selection per dead pool plus a few bought
    /// scores — paired with [`rs_with_dead_pools`].
    fn character_with_pools_and_scores(pool_count: usize, score_count: usize) -> Entity {
        let mut entity = character(
            "companion",
            vec![("ability.artes_liberales", None, 1); score_count],
        );
        entity.xp_pool = 1_000_000;
        entity.selections = (0..pool_count)
            .map(|i| Selection::new(Id::new(format!("virtue.dead_pool_{i}"))))
            .collect();
        entity
    }

    /// E4 (round-2 test-verification finding): `MAX_XP_SOLVE_NODES`'s exact
    /// boundary was untested — the two tests above use node counts far from
    /// the limit (3003 and 503), so an off-by-one in `node_count >
    /// MAX_XP_SOLVE_NODES` (e.g. `>=` instead of `>`) would silently reject a
    /// legal character at exactly the bound. Exactly at the bound must
    /// validate clean.
    #[test]
    fn exactly_at_the_solve_bound_is_not_rejected() {
        let pools = crate::effective::MAX_XP_SOLVE_NODES - 3 - 5;
        let rs = rs_with_dead_pools(pools);
        let entity = character_with_pools_and_scores(pools, 5);

        let result = validate(&entity, &rs);

        assert!(issues_with(&result, ValidationIssue::CODE_XP_SOLVE_BOUND_EXCEEDED).is_empty());
    }

    /// The mirror of the above: one node past the bound must still be
    /// rejected, naming the exact counts.
    #[test]
    fn one_node_past_the_solve_bound_is_rejected() {
        let rs = rs();
        let scores: Vec<(&str, Option<&str>, u8)> =
            vec![("ability.artes_liberales", None, 1); 2046];
        let entity = character("companion", scores);

        let result = validate(&entity, &rs);

        let errors = issues_with(&result, ValidationIssue::CODE_XP_SOLVE_BOUND_EXCEEDED);
        assert_eq!(errors.len(), 1, "{:?}", result.issues);
        assert_eq!(
            errors[0].args.get("nodes").cloned(),
            Some("2049".to_string())
        );
        assert_eq!(
            errors[0].args.get("limit").cloned(),
            Some("2048".to_string())
        );
    }
}
