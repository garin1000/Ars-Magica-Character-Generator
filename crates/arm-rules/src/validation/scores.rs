//! Characteristics, Abilities, Arts, personality traits, and reputations.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;
use crate::ruleset::ENGINE_REQUIRED_CATEGORY_PERSONALITY;

/// Validates Characteristic point-buy: each score must be a legal table value
/// and within the characteristic's per-target buy range, and the total cost must
/// not exceed the starting points (over = error, under = a non-blocking "points
/// unspent" warning, mirroring the V/F balance rule). No-op when the ruleset
/// ships no characteristic rules.
///
/// The buy range is the base ±3 by default, widened upward by Great
/// (Characteristic) and downward by Poor (Characteristic) (see
/// [`characteristic_cap`](crate::effective::characteristic_cap) /
/// [`characteristic_floor`](crate::effective::characteristic_floor)). The cost
/// table itself spans the absolute ±5 range so the higher/lower scores can be
/// priced; without the virtue/flaw they are legal table values but above the cap
/// / below the floor.
///
/// The point-spend check is skipped entirely when the character has no
/// Characteristics set: an untouched step is not yet under-spent, so a fresh
/// character is not nagged. Out-of-range scores are always flagged.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2340-2354 (the cost
/// table and the seven starting points), :4105 (the +3 base cap), :3987-3989
/// (Great's +5), :6598-6600 (Poor's −5). The numbers themselves are data in
/// `rules/core/characteristics.json` (see RULES.md).
pub(crate) fn validate_characteristics(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(rules) = ruleset.characteristic_rules() else {
        return;
    };
    let (Some(min), Some(max)) = (rules.min_score(), rules.max_score()) else {
        return;
    };

    for (&characteristic, &score) in &entity.characteristics {
        if !rules.is_legal_score(score) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CHARACTERISTIC_OUT_OF_RANGE,
                CreationPhase::Characteristics,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("score", score.to_string()),
                    ("min", min.to_string()),
                    ("max", max.to_string()),
                ]),
                None,
            ));
            continue;
        }
        // A legal table value still has to sit within the range that this
        // character's Great/Poor (Characteristic) choices open for the target.
        let cap = crate::effective::characteristic_cap(entity, ruleset, characteristic);
        let floor = crate::effective::characteristic_floor(entity, ruleset, characteristic);
        if i32::from(score) > cap {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CHARACTERISTIC_ABOVE_CAP,
                CreationPhase::Characteristics,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("score", score.to_string()),
                    ("cap", cap.to_string()),
                ]),
                None,
            ));
        } else if i32::from(score) < floor {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CHARACTERISTIC_BELOW_FLOOR,
                CreationPhase::Characteristics,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("score", score.to_string()),
                    ("floor", floor.to_string()),
                ]),
                None,
            ));
        }
    }

    // Don't evaluate the point spend before the user has touched the step.
    if entity.characteristics.is_empty() {
        return;
    }

    let cost = rules.total_cost(&entity.characteristics);
    // Improved Characteristics (+3 each, stackable) raises the buy budget above
    // the ruleset's base start_points.
    let granted = crate::effective::characteristic_points_granted(entity, ruleset);
    let budget = i32::from(rules.start_points) + granted;
    if cost > budget {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_CHARACTERISTIC_OVERSPENT,
            CreationPhase::Characteristics,
            args([("cost", cost.to_string()), ("points", budget.to_string())]),
            None,
        ));
    } else if cost < budget {
        issues.push(ValidationIssue::warning(
            ValidationIssue::CODE_CHARACTERISTIC_POINTS_UNSPENT,
            CreationPhase::Characteristics,
            args([("cost", cost.to_string()), ("points", budget.to_string())]),
            None,
        ));
    }
}

/// Enforces the parameter-relative precondition on `characteristic_limit`
/// effects: a limit-shift may only be taken on a characteristic whose *base*
/// (bought) score is already at the limit being extended. Great (Characteristic,
/// positive amount) needs base ≥ the base cap (+3); Poor (Characteristic,
/// negative amount) needs base ≤ the base floor (−3). The threshold is derived
/// from the ruleset's base cap/floor by the sign of the amount, so no per-effect
/// number is stored. This is parameter-relative (it constrains whichever
/// characteristic the selection targets), so it lives here rather than in the
/// static [`Prereq`] tree.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3987-3989 (Great,
/// "already … at least +3"), :6598-6600 (Poor, "already −3 or lower").
pub(crate) fn validate_characteristic_limit_preconditions(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(rules) = ruleset.characteristic_rules() else {
        return;
    };
    let base_max = rules.base_max_score();
    let base_min = rules.base_min_score();
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-skipped precondition check.
            let (amount, target, base) = match effect {
                Effect::CharacteristicLimit { param, amount } => {
                    let Some(target) = selection
                        .params
                        .get(param)
                        .and_then(Characteristic::from_id)
                    else {
                        continue; // unresolved param value is reported by validate_parameters
                    };
                    let base = entity.characteristics.get(&target).copied().unwrap_or(0);
                    (*amount, target, base)
                }
                Effect::AbilityBonus { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::AbilityAuthorization { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SpellMasteryXp { .. }
                | Effect::GrantsSpellMastery { .. }
                | Effect::GrantsSelection { .. }
                | Effect::ItemLevelBudget { .. }
                | Effect::MasterpieceItem
                | Effect::TrueFaithGrant { .. }
                | Effect::WarpingGrant { .. }
                | Effect::SizeDelta { .. }
                | Effect::CharacteristicScoreDelta { .. }
                | Effect::GroupAffinityCost { .. }
                | Effect::GrantsReputation { .. }
                | Effect::MightGrant { .. }
                | Effect::PowerLevels { .. }
                // M5/5b in-play effects: consumed by derived.rs (5i). They carry
                // no ability/characteristic creation target to check here.
                | Effect::MagicalFocus { .. }
                | Effect::CastingTotalMod { .. }
                | Effect::LabTotalMod { .. }
                | Effect::DeficientArt { .. }
                | Effect::MagicTotalHalving { .. }
                | Effect::SoakMod { .. }
                | Effect::CombatMod { .. }
                | Effect::HealthMod { .. }
                | Effect::MagicResistanceMod { .. }
                | Effect::AgingMod { .. }
                | Effect::AdvancementMod { .. }
                | Effect::SpecialCastingMod { .. }
                | Effect::AbilityRollMod { .. }
                // Elemental Magic carries no ability/characteristic creation target.
                | Effect::ElementalMagic { .. } => continue,
            };
            if amount > 0 {
                if let Some(cap) = base_max
                    && i32::from(base) < i32::from(cap)
                {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_CHARACTERISTIC_MAX_BASE_TOO_LOW,
                        // The Virtue names the limit, but the value that violates
                        // it is the Characteristic score.
                        CreationPhase::Characteristics,
                        args([
                            ("item", selection.item_ref.to_string()),
                            ("characteristic", target.to_string()),
                            ("base", base.to_string()),
                            ("min", cap.to_string()),
                        ]),
                        Some(selection.item_ref.clone()),
                    ));
                }
            } else if amount < 0
                && let Some(floor) = base_min
                && i32::from(base) > i32::from(floor)
            {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_CHARACTERISTIC_MIN_BASE_TOO_HIGH,
                    CreationPhase::Characteristics,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("characteristic", target.to_string()),
                        ("base", base.to_string()),
                        ("max", floor.to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}

/// Validates Ability scores: every referenced ability must resolve against the
/// catalogue, no (ability, parameter) pair may appear twice, a parameterized
/// ability must carry a parameter value, and the total XP the bought scores cost
/// may not exceed the character's `xp_pool`.
///
/// A parameterized ability (e.g. `(Area) Lore`) is identified by its instance
/// `parameter` (the area / language), so a character may hold several; plain
/// abilities have no parameter and are deduped by id (one instance).
///
/// The age cap is deferred to M4. XP cost per score comes from the advancement
/// table (`AdvancementTable::xp_for_score`); a non-zero score with no table row
/// is off-table and flagged `ability_score_out_of_range` (mirroring the
/// characteristic range check), so an illegal score is never silently priced at
/// 0 XP. The XP a score costs is summed against the shared pool by
/// [`validate_xp_pool`], not here.
pub(crate) fn validate_abilities(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut seen: BTreeMap<(&Id, Option<&str>), u32> = BTreeMap::new();
    // The highest score the advancement table prices. A ruleset that ships no
    // advancement table has no legal score range to check against, so off-table
    // range checking is skipped.
    let max_score = ruleset.advancement.max_score();

    for entry in &entity.ability_scores {
        match ruleset.abilities.get(&entry.ability) {
            None => issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_ABILITY,
                CreationPhase::Abilities,
                args([("ability", entry.ability.to_string())]),
                Some(entry.ability.clone()),
            )),
            Some(ability) => {
                // A parameterized ability needs its value supplied (which Area?).
                if ability.parameter.is_some()
                    && entry.parameter.as_deref().is_none_or(str::is_empty)
                {
                    issues.push(ValidationIssue::error(
                        ValidationIssue::CODE_ABILITY_PARAMETER_REQUIRED,
                        CreationPhase::Abilities,
                        args([("ability", entry.ability.to_string())]),
                        Some(entry.ability.clone()),
                    ));
                }
            }
        }
        // The advancement table covers the legal score range. A non-zero score
        // with no table row is off-table (illegal) — flag it rather than silently
        // pricing it at 0 XP, so direct-entry illegal states surface here instead
        // of relying on the UI to keep them out (mirrors characteristic range
        // checking).
        // A non-zero score with no table row is off-table (illegal) — flag it
        // rather than silently pricing it at 0 XP, so direct-entry illegal states
        // surface here (mirrors characteristic range checking).
        if ruleset.advancement.xp_for_score(entry.score).is_none()
            && let Some(max) = max_score
        {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_ABILITY_SCORE_OUT_OF_RANGE,
                CreationPhase::Abilities,
                args([
                    ("ability", entry.ability.to_string()),
                    ("score", entry.score.to_string()),
                    ("max", max.to_string()),
                ]),
                Some(entry.ability.clone()),
            ));
        }
        // Age → max-Ability-score cap (Core:2366-2374). An Ability carrying an
        // Affinity may exceed it by +2 (Core:3374), not without limit. The cap is
        // read from the ruleset's age band table; a ruleset that ships none cannot
        // enforce it, so the check is skipped.
        if let Some(age) = entity.age
            && let Some(base_cap) = crate::effective::age_max_ability_score(ruleset, age)
        {
            let mut cap = u32::from(base_cap);
            if crate::effective::ability_affinity(
                entity,
                ruleset,
                &entry.ability,
                entry.parameter.as_deref(),
            )
            .is_some()
            {
                cap += 2;
            }
            if u32::from(entry.score) > cap {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_ABILITY_ABOVE_AGE_CAP,
                    CreationPhase::Abilities,
                    args([
                        ("ability", entry.ability.to_string()),
                        ("score", entry.score.to_string()),
                        ("cap", cap.to_string()),
                        ("age", age.to_string()),
                    ]),
                    Some(entry.ability.clone()),
                ));
            }
        }
        let key = (&entry.ability, entry.parameter.as_deref());
        *seen.entry(key).or_insert(0) += 1;
    }

    for ((ability, _parameter), count) in seen {
        if count > 1 {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_DUPLICATE_ABILITY,
                CreationPhase::Abilities,
                args([
                    ("ability", ability.to_string()),
                    ("count", count.to_string()),
                ]),
                Some(ability.clone()),
            ));
        }
    }
}

/// Validates Hermetic Art scores (mirrors [`validate_abilities`], minus the
/// parameter logic — Arts are not parameterized): every referenced Art must
/// resolve against the catalogue, no Art may appear twice, and every bought score
/// must be priced by the Art advancement table. The XP a score costs is summed
/// against the shared pool by [`validate_xp_pool`], not here.
pub(crate) fn validate_arts(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let mut seen: BTreeMap<&Id, u32> = BTreeMap::new();
    // The highest score the Art advancement table prices. A ruleset that ships no
    // Art advancement table has no legal score range to check against, so
    // off-table range checking is skipped.
    let max_score = ruleset.art_advancement.max_score();

    for entry in &entity.art_scores {
        if !ruleset.arts.contains_key(&entry.art) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_ART,
                CreationPhase::Arts,
                args([("art", entry.art.to_string())]),
                Some(entry.art.clone()),
            ));
        }
        // A non-zero score with no table row is off-table (illegal) — flag it
        // rather than silently pricing it at 0 XP.
        if ruleset.art_advancement.xp_for_score(entry.score).is_none()
            && let Some(max) = max_score
        {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_ART_SCORE_OUT_OF_RANGE,
                CreationPhase::Arts,
                args([
                    ("art", entry.art.to_string()),
                    ("score", entry.score.to_string()),
                    ("max", max.to_string()),
                ]),
                Some(entry.art.clone()),
            ));
        }
        *seen.entry(&entry.art).or_insert(0) += 1;
    }

    for (art, count) in seen {
        if count > 1 {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_DUPLICATE_ART,
                CreationPhase::Arts,
                args([("art", art.to_string()), ("count", count.to_string())]),
                Some(art.clone()),
            ));
        }
    }
}

/// Validates that every held Supernatural Ability is legal: it must be covered by
/// a granting Virtue (an `ability_score_grant` floor) or fit within the Gift's
/// free slot (one for a Gifted non-magus, none for a magus). Uncovered instances
/// beyond the free allowance emit `supernatural_ability_requires_virtue`
/// (deterministic by sorted id). Source: Core Rules.md:2874.
pub(crate) fn validate_supernatural_abilities(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };
    let free_total = crate::effective::supernatural_free_slots(entity, ruleset, profile).total;
    // A granting Virtue seeds an `ability_score_grant` floor; such abilities are
    // "covered" and never consume the free slot.
    let floors: std::collections::BTreeSet<Id> =
        crate::effective::ability_score_floors(entity, ruleset)
            .into_iter()
            .map(|f| f.ability)
            .collect();
    let mut uncovered: Vec<&Id> = entity
        .ability_scores
        .iter()
        .filter(|a| {
            ruleset
                .abilities
                .get(&a.ability)
                .is_some_and(|ab| ab.category == crate::ability::AbilityCategory::Supernatural)
        })
        .map(|a| &a.ability)
        .filter(|id| !floors.contains(*id))
        .collect();
    uncovered.sort();
    uncovered.dedup();
    // The first `free_total` uncovered abilities occupy the free Gift slot(s); the
    // rest require a granting Virtue.
    for ability in uncovered.into_iter().skip(usize::from(free_total)) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_SUPERNATURAL_ABILITY_REQUIRES_VIRTUE,
            // Fixable from either side, but the stored offending value is the
            // Ability score, so it belongs to the Abilities step.
            CreationPhase::Abilities,
            args([("ability", ability.to_string())]),
            Some(ability.clone()),
        ));
    }
}

/// Validates Personality Traits: `|value|` never exceeds 6, and at most one trait
/// per selected Major Personality Flaw may exceed ±3 (a Major Personality Flaw is
/// represented by a single ±6 trait; others stay ±3). Source: Core Rules.md:2500-2503.
pub(crate) fn validate_personality_traits(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    // The personality category slug is referenced directly here, deliberately.
    // Unlike caps.rs — whose per-category caps are driven by profile data that
    // *happens* to name categories, so no slug is baked in — this is a universal
    // core rule keyed specifically to the Personality Flaw category (Core:2500-2503,
    // :2820): each Major Personality Flaw permits exactly one ±6 trait. Categories
    // are free-form data slugs with no registry or per-item metadata that marks
    // "this is the personality category", and the rule is not per-profile, so there
    // is no non-speculative data field to drive it from — adding a dedicated
    // ruleset/profile field solely for this one rule would be YAGNI over-engineering.
    // The slug is not an unvalidated literal, though: it is the engine-required,
    // load-validated category `ENGINE_REQUIRED_CATEGORY_PERSONALITY`, whose presence
    // `Ruleset::validate_integrity` enforces for any ruleset shipping a V/F
    // catalogue (mirroring the ENGINE_REQUIRED_ABILITIES/ARTS role checks), so a
    // ruleset that renamed or dropped it fails loudly at load rather than silently
    // counting zero here.
    let major_personality_flaws = entity
        .selections
        .iter()
        .filter(|s| {
            ruleset.point_items.get(&s.item_ref).is_some_and(|item| {
                item.category == ENGINE_REQUIRED_CATEGORY_PERSONALITY
                    && item.magnitude == Magnitude::Major
            })
        })
        .count();

    // Traits are sorted by name (normalize) for a deterministic "excess" choice.
    let mut traits: Vec<&crate::types::PersonalityTrait> =
        entity.personality_traits.iter().collect();
    traits.sort_by(|a, b| a.name.cmp(&b.name));
    let mut over_three_budget = major_personality_flaws;
    for trait_ in traits {
        let magnitude = trait_.value.unsigned_abs();
        if magnitude > 6 {
            issues.push(personality_out_of_range(trait_, 6));
        } else if magnitude > 3 {
            if over_three_budget > 0 {
                over_three_budget -= 1;
            } else {
                issues.push(personality_out_of_range(trait_, 3));
            }
        }
    }
}

fn personality_out_of_range(trait_: &crate::types::PersonalityTrait, max: i8) -> ValidationIssue {
    ValidationIssue::error(
        ValidationIssue::CODE_PERSONALITY_TRAIT_OUT_OF_RANGE,
        CreationPhase::PersonalityReputations,
        args([
            ("name", trait_.name.clone()),
            ("value", trait_.value.to_string()),
            ("max", max.to_string()),
        ]),
        None,
    )
}

/// Validates that every starting Reputation is backed by a granting Virtue/Flaw:
/// the count of reputations of each `kind` must not exceed the grants of that kind
/// (`Effect::GrantsReputation`). A player-chosen-kind grant (`kind == None`, e.g.
/// Famous) is a wildcard authorizing one Reputation of *any* type; a Reputation
/// consumes a matching concrete-kind slot first, falling back to a wildcard slot.
/// Excess reputations emit `reputation_not_granted`. Source: Core Rules.md:2514.
pub(crate) fn validate_reputations(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    use crate::types::ReputationType;
    let mut remaining: BTreeMap<ReputationType, usize> = BTreeMap::new();
    let mut wildcard: usize = 0;
    for grant in crate::effective::reputation_grants(entity, ruleset) {
        match grant.reputation_type {
            Some(kind) => *remaining.entry(kind).or_insert(0) += 1,
            None => wildcard += 1,
        }
    }
    for reputation in &entity.reputations {
        let slot = remaining.entry(reputation.kind).or_insert(0);
        if *slot > 0 {
            *slot -= 1;
        } else if wildcard > 0 {
            wildcard -= 1;
        } else {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_REPUTATION_NOT_GRANTED,
                CreationPhase::PersonalityReputations,
                args([
                    ("kind", reputation.kind.to_string()),
                    ("content", reputation.content.clone()),
                ]),
                None,
            ));
        }
    }
}
