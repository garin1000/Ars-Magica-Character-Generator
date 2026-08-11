//! Selection legality: categories, traits, parameters, foci, and Gift policy.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;

pub(crate) fn validate_permitted_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    // An empty permitted list means "no category restriction".
    if profile.permitted_categories.is_empty() {
        return;
    }

    for selection in &entity.selections {
        // The profile's own gift is governed solely by `validate_gift_policy`
        // (required/allowed/forbidden). Exempt it here: it is contradictory for a
        // profile to mandate a trait via `gift_policy` yet reject its category
        // (The Gift is `special`), so gifted profiles need not whitelist it.
        if profile.gift_id.as_ref() == Some(&selection.item_ref) {
            continue;
        }

        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if !profile.permitted_categories.contains(&item.category) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_CATEGORY_NOT_PERMITTED,
                CreationPhase::VirtuesFlaws,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("category", item.category.clone()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

pub(crate) fn validate_forbidden_categories(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    if profile.forbidden_categories.is_empty() {
        return;
    }

    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if profile.forbidden_categories.contains(&item.category) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_FORBIDDEN_CATEGORY,
                CreationPhase::VirtuesFlaws,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("category", item.category.clone()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

pub(crate) fn validate_entity_kind_applicability(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };

        if !item.entity_kinds.is_empty() && !item.entity_kinds.contains(&entity.entity_kind) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_WRONG_ENTITY_KIND,
                CreationPhase::VirtuesFlaws,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("entity_kind", entity.entity_kind.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

pub(crate) fn validate_duplicate_selections(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut seen: BTreeMap<(&Id, &BTreeMap<String, Id>), usize> = BTreeMap::new();

    for selection in &entity.selections {
        let key = (&selection.item_ref, &selection.params);
        *seen.entry(key).or_insert(0) += 1;
    }

    for ((item_ref, _params), count) in &seen {
        // Selections are grouped by (item_ref, params): two selections of the
        // same parameterized item with DIFFERENT params are distinct targets and
        // do not collide here. An item may be taken up to `max_per_target` times
        // for the same target (default 1; Great Characteristic allows 2).
        let max = ruleset
            .point_items
            .get(*item_ref)
            .map_or(1, |item| usize::from(item.max_per_target));
        if *count <= max {
            continue;
        }
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_DUPLICATE_SELECTION,
            CreationPhase::VirtuesFlaws,
            args([
                ("item", item_ref.to_string()),
                ("count", count.to_string()),
                ("max", max.to_string()),
            ]),
            Some((*item_ref).clone()),
        ));
    }
}

pub(crate) fn validate_required_traits(
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    for required_id in &profile.required_traits {
        if !selected_ids.contains(required_id) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_MISSING_REQUIRED_TRAIT,
                CreationPhase::VirtuesFlaws,
                args([("item", required_id.to_string())]),
                Some(required_id.clone()),
            ));
        }
    }
}

pub(crate) fn validate_forbidden_traits(
    type_profile: Option<&EntityTypeProfile>,
    selected_ids: &BTreeSet<&Id>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    for forbidden_id in &profile.forbidden_traits {
        if selected_ids.contains(forbidden_id) {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_FORBIDDEN_TRAIT,
                CreationPhase::VirtuesFlaws,
                args([("item", forbidden_id.to_string())]),
                Some(forbidden_id.clone()),
            ));
        }
    }
}

/// Whether a parameter `value` resolves against its `domain`'s registry: `Item`
/// → point items, `Ability` → the ability catalogue, `Art` → the art catalogue,
/// `Technique`/`Form` → the art catalogue *and* the required art class (so
/// Deficient Technique cannot target a Form; Core Rules.md:5909-5915),
/// `Characteristic` → [`Characteristic::from_id`], `Text` → always (no registry).
/// Shared by virtue/flaw parameter validation and spell parameter validation.
pub(crate) fn param_value_resolves(ruleset: &Ruleset, domain: ParameterDomain, value: &Id) -> bool {
    match domain {
        ParameterDomain::Item => ruleset.point_items.contains_key(value),
        ParameterDomain::Ability => ruleset.abilities.contains_key(value),
        ParameterDomain::Characteristic => Characteristic::from_id(value).is_some(),
        ParameterDomain::Art => ruleset.arts.contains_key(value),
        ParameterDomain::Technique => ruleset
            .arts
            .get(value)
            .is_some_and(|a| a.art_type == crate::art::ArtType::Technique),
        ParameterDomain::Form => ruleset
            .arts
            .get(value)
            .is_some_and(|a| a.art_type == crate::art::ArtType::Form),
        ParameterDomain::Text => true,
    }
}

/// Validates that each selection of a parameterized item supplies exactly the
/// declared parameter keys (no missing, no extra) and that each provided value
/// resolves against its domain's registry: `item` → point items, `ability` →
/// the ability catalogue, `art` → the art catalogue, `characteristic` →
/// [`Characteristic::from_id`]. A value that does not resolve emits
/// `unknown_param_value`.
pub(crate) fn validate_parameters(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        validate_selection_parameters(selection, ruleset, CreationPhase::VirtuesFlaws, issues);
    }
}

/// The parameter checks for ONE selection: declared-vs-provided keys
/// (`missing_param` / `unexpected_param`) and each value's domain resolution
/// (`unknown_param_value`).
///
/// Shared by bought selections ([`validate_parameters`]) and the *derived* picks
/// that never live on `entity.selections` — House / Mythic-type Open grants and
/// the Warping-owed fills — so "{form} Monstrosity" chosen for an open grant is
/// held to the same standard as one bought on the V/F tab.
///
/// `phase` is the caller's, not this function's: the same three codes are fixed on
/// the V/F step for a bought selection, on the House or Mythic-type step for an
/// open grant, and only in the finished character for a Warping fill.
pub(crate) fn validate_selection_parameters(
    selection: &Selection,
    ruleset: &Ruleset,
    phase: CreationPhase,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
        return;
    };

    let declared: BTreeSet<&str> = item.parameters.iter().map(|p| p.key.as_str()).collect();
    let provided: BTreeSet<&str> = selection.params.keys().map(String::as_str).collect();

    // A parameter targeting a PARAMETERIZED ability also expects the instance
    // discriminator, supplied under the target ability's own param key
    // ((Area) Lore → "area"). So Puissant on (Area) Lore needs both keys; on a
    // plain ability the instance key would be an unexpected extra.
    let mut expected = declared.clone();
    for param in &item.parameters {
        if matches!(param.domain, ParameterDomain::Ability)
            && let Some(target) = selection.params.get(&param.key)
            && let Some(ability) = ruleset.abilities.get(target)
            && let Some(instance_key) = ability.parameter.as_deref()
        {
            expected.insert(instance_key);
        }
    }

    for missing in expected.difference(&provided) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_MISSING_PARAM,
            phase,
            args([
                ("item", selection.item_ref.to_string()),
                ("key", missing.to_string()),
            ]),
            Some(selection.item_ref.clone()),
        ));
    }

    for extra in provided.difference(&expected) {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_UNEXPECTED_PARAM,
            phase,
            args([
                ("item", selection.item_ref.to_string()),
                ("key", extra.to_string()),
            ]),
            Some(selection.item_ref.clone()),
        ));
    }

    // Resolve values for domains that have a registry (Item -> point items,
    // Ability -> ability catalogue, Art -> art catalogue).
    for param in &item.parameters {
        let Some(value) = selection.params.get(&param.key) else {
            continue; // missing already reported above
        };
        let resolves = param_value_resolves(ruleset, param.domain, value);
        if !resolves {
            issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_PARAM_VALUE,
                phase,
                args([
                    ("item", selection.item_ref.to_string()),
                    ("key", param.key.clone()),
                    ("value", value.to_string()),
                    ("domain", param.domain.to_string()),
                ]),
                Some(selection.item_ref.clone()),
            ));
        }
    }
}

/// Validates that every ability-bonus effect (e.g. Puissant Ability +2) targets
/// an ability instance the character actually holds. The target is
/// `(ability, parameter)`: for a parameterized ability ((Area) Lore) the instance
/// value is read from the selection's matching key, so Puissant "Brandenburg Lore"
/// must have a bought Brandenburg Lore row. A dangling target (e.g. the ability was
/// removed) means the +2 attaches to nothing, so flag it. Effect-driven — no virtue
/// id is hardcoded.
pub(crate) fn validate_ability_bonus_targets(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    for selection in &entity.selections {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            // Exhaustive match so adding an Effect variant is a compile error
            // here, not a silently-skipped target check.
            let param = match effect {
                // Affinity reduces the cost of buying one ability, so it too must
                // target a held instance — the cost break attaches to nothing
                // otherwise, exactly like a dangling Puissant.
                Effect::AbilityBonus { param, .. } | Effect::AffinityAbilityCost { param, .. } => {
                    param
                }
                // Art bonuses are not parameterized instances; their target is
                // resolved by validate_parameters (domain check). Characteristic
                // limits are handled elsewhere. AbilityScoreGrant *creates* the
                // score, so it needs no pre-existing bought row. The XP-pool and
                // characteristic-budget grants carry no target.
                Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
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
            let Some(target) = selection.params.get(param) else {
                continue; // missing ability key already reported by validate_parameters
            };
            // The instance discriminator, if the target ability is parameterized.
            let instance = ruleset
                .abilities
                .get(target)
                .and_then(|a| a.parameter.as_deref())
                .and_then(|key| selection.params.get(key).map(Id::as_str));
            let has_instance = entity
                .ability_scores
                .iter()
                .any(|a| &a.ability == target && a.parameter.as_deref() == instance);
            if !has_instance {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_ABILITY_BONUS_DANGLING_TARGET,
                    // The offending value is the Virtue's target parameter, so the
                    // fix is on the V/F step (retarget it) even though the missing
                    // half is an Ability.
                    CreationPhase::VirtuesFlaws,
                    args([
                        ("item", selection.item_ref.to_string()),
                        ("ability", target.to_string()),
                        ("parameter", instance.unwrap_or("").to_string()),
                    ]),
                    Some(selection.item_ref.clone()),
                ));
            }
        }
    }
}

/// Enforces the "one Magical Focus per magus" limit (Core Rules.md:4542) by
/// counting [`Effect::MagicalFocus`] across everything that feeds the effective
/// layer (bought selections plus House / Mythic-type grants, e.g. Mythic Blood's
/// bundled Minor Focus). More than one Focus is illegal. This counts the *effect*
/// rather than using pairwise `incompatible_with`, so it also catches two Minor
/// Foci with different descriptors (distinct selections that no incompatibility
/// pair would flag). Effect-driven — no virtue id is hardcoded.
pub(crate) fn validate_magical_focus(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut foci = 0usize;
    for selection in crate::effective::selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        foci += item
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::MagicalFocus { .. }))
            .count();
    }
    if foci > 1 {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_MULTIPLE_MAGICAL_FOCI,
            CreationPhase::VirtuesFlaws,
            args([("count", foci.to_string())]),
            None,
        ));
    }
}

/// Enforces the type's Gift policy (required / allowed / forbidden). The policy
/// per type is data; this is the mechanism the book's Gift rules map onto.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2868-2877 (The Gift:
/// "all magi must have this Virtue"; "Grogs can never have The Gift"); magi must
/// take The Gift at :2858; only magi may take the Hermetic Magus Social Status
/// at :2293 and :4067-4069.
pub(crate) fn validate_gift_policy(
    entity: &Entity,
    ruleset: &Ruleset,
    type_profile: Option<&EntityTypeProfile>,
    issues: &mut Vec<ValidationIssue>,
) {
    let Some(profile) = type_profile else {
        return;
    };

    let Some(policy) = profile.gift_policy else {
        return;
    };

    // Shared with the Supernatural free-slot computation so both use one
    // definition of "has The Gift".
    let has_gift = crate::effective::has_the_gift(entity, ruleset, profile);

    match policy {
        GiftPolicy::Required => {
            if !has_gift {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_GIFT_REQUIRED,
                    CreationPhase::VirtuesFlaws,
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Forbidden => {
            if has_gift {
                issues.push(ValidationIssue::error(
                    ValidationIssue::CODE_GIFT_FORBIDDEN,
                    CreationPhase::VirtuesFlaws,
                    BTreeMap::new(),
                    None,
                ));
            }
        }
        GiftPolicy::Allowed => {}
    }
}
