//! Spell levels and Spell Mastery: the levels budget (base + Skilled/Weak
//! Parens + post-Gauntlet life-stage levels), per-Technique/Form level caps,
//! and the Spell-Mastery XP pool/floor/Affinity chain (Flawless Magic). Split
//! out of `effective.rs` (Viktor's V4 architecture finding — 93 free functions
//! across 7 unrelated domains in one file); pure code motion, no behavior
//! change.

use super::*;

/// Sums the [`Effect::SpellLevels`] amounts across the entity's selections (may
/// be negative; Skilled Parens +30, Weak Parens −30).
///
/// Surfaced on its own (not only folded into [`spell_levels_budget`]) so the
/// spell-levels bar can show the editable base beside a labelled V/F bonus,
/// mirroring how the XP bar lists extra pools beside the general one.
pub fn spell_levels_bonus(entity: &Entity, ruleset: &Ruleset) -> i64 {
    sum_signed_effect(entity, ruleset, |e| match e {
        Effect::SpellLevels { amount } => Some(*amount),
        // Exhaustive so adding an Effect variant is a compile error here, not a
        // silently-ignored contribution to the spell-levels budget.
        Effect::AbilityBonus { .. }
        | Effect::CharacteristicLimit { .. }
        | Effect::ArtBonus { .. }
        | Effect::AffinityAbilityCost { .. }
        | Effect::AffinityArtCost { .. }
        | Effect::RestrictedAbilityXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
        | Effect::GeneralXp { .. }
        | Effect::LaterLifeXpRate { .. }
        | Effect::AbilityAuthorization { .. }
        | Effect::LocalityAbilityCapFraction { .. }
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
        | Effect::ElementalMagic { .. } => None,
    })
}

/// Sums the [`Effect::GeneralXp`] amounts across the entity's selections (may be
/// negative; Skilled Parens +60, Weak Parens −60).
///
/// `pub(crate)` (not just module-private) because
/// [`xp_allocation`](crate::effective::xp_allocation) (the `xp` domain) folds
/// this into the general pool's size.
pub(crate) fn general_xp_bonus(entity: &Entity, ruleset: &Ruleset) -> i64 {
    sum_signed_effect(entity, ruleset, |e| match e {
        Effect::GeneralXp { amount } => Some(*amount),
        // Exhaustive so adding an Effect variant is a compile error here, not a
        // silently-ignored contribution to the general XP pool.
        Effect::AbilityBonus { .. }
        | Effect::CharacteristicLimit { .. }
        | Effect::ArtBonus { .. }
        | Effect::AffinityAbilityCost { .. }
        | Effect::AffinityArtCost { .. }
        | Effect::RestrictedAbilityXp { .. }
        | Effect::CharacteristicPoints { .. }
        | Effect::AbilityScoreGrant { .. }
        | Effect::SpellLevels { .. }
        // The later-life RATE is not a pool bonus: it multiplies out into the
        // life-stage budget (see `life_stage::LifeStageRules::later_life_budget`),
        // which then becomes the general pool. Adding it here would double-count.
        | Effect::LaterLifeXpRate { .. }
        | Effect::AbilityAuthorization { .. }
        | Effect::LocalityAbilityCapFraction { .. }
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
        | Effect::ElementalMagic { .. } => None,
    })
}

/// Sums a signed per-selection effect amount across everything that feeds the
/// effective layer (selections + derived grants).
fn sum_signed_effect(
    entity: &Entity,
    ruleset: &Ruleset,
    pick: impl Fn(&Effect) -> Option<i16>,
) -> i64 {
    let mut total: i64 = 0;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Some(amount) = pick(effect) {
                total += i64::from(amount);
            }
        }
    }
    total
}

/// The base spell-levels budget BEFORE Skilled/Weak Parens modifiers: the
/// per-character [`Entity::spell_levels_override`] when set, otherwise the type
/// profile's `spell_levels` (120 for a magus; 0 for a type with no profile).
/// Factored so the effective payload and the validator select the base
/// identically and can never diverge (Issue 11).
// Source: Ars Magica - Definitive Edition (Core Rules).md:2215-2216, :2435
pub fn spell_levels_base(entity: &Entity, profile: Option<&EntityTypeProfile>) -> u32 {
    entity
        .spell_levels_override
        .unwrap_or_else(|| profile.map(|p| p.spell_levels).unwrap_or(0))
}

/// The levels of spells a magus took out of its years past the Gauntlet — the
/// player's chosen slice of "30 points per year", where "Each point can be an
/// experience point in an Art or Ability or **one level of spell**" (`:2471`).
///
/// 0 for a character with no life-stage plan, and for a ruleset shipping no
/// `post_apprenticeship` block — the rate is data, so with no block there is
/// nothing to grant.
// Source: Ars Magica - Definitive Edition (Core Rules).md:2216, :2471.
pub fn life_stage_spell_levels(entity: &Entity, ruleset: &Ruleset) -> u32 {
    ruleset
        .life_stages()
        .and_then(|rules| rules.budget(entity, ruleset))
        .map_or(0, |budget| budget.post_gauntlet_spell_levels)
}

/// The magus's effective spell-levels budget: the base ([`spell_levels_base`])
/// plus any [`Effect::SpellLevels`] modifiers and the levels its post-Gauntlet
/// years bought ([`life_stage_spell_levels`]), clamped at 0.
///
/// The post-Gauntlet term is **additive, not a second budget.** Apprenticeship's
/// "120 levels of spells" (`:2435`) are the type profile's `spell_levels` and are
/// what `base` selects; these are the player's chosen slice of the fungible "30
/// points per year" (`:2471`), which is also why `post_gauntlet_xp` and
/// `post_gauntlet_spell_levels` always sum to `post_gauntlet_points`.
///
/// Folded in **here**, in the one selector both `validate_spells` and the
/// `EffectiveScores` payload call, so the `over_spell_levels` finding and the
/// spell-levels bar can never disagree about what the budget is.
///
/// [`Entity::spell_levels_override`] still replaces the *profile base* only, and
/// the post-Gauntlet levels stay on top of it. Deliberate: the override is the flat
/// flow's escape hatch, and it is not made exclusive with a life-stage plan the way
/// [`Entity::xp_pool`] is.
// Source: Ars Magica - Definitive Edition (Core Rules).md:2216, :2435, :2471.
pub fn spell_levels_budget(base: u32, entity: &Entity, ruleset: &Ruleset) -> u32 {
    clamp_to_u32(
        i64::from(base)
            + spell_levels_bonus(entity, ruleset)
            + i64::from(life_stage_spell_levels(entity, ruleset)),
    )
}

/// The maximum level a magus may learn of a spell of the given Technique/Form:
/// the sum of Technique, Form, Intelligence, Magic Theory and 3 (Ars Magica - Definitive Edition (Core Rules).md:2465),
/// using effective Art/Ability scores. Returns an `i64` (small or negative for a
/// beginning magus). Requisite-Art reduction is a lab-total nuance out of scope.
/// Single source of truth: both the validation cap and the UI-surfaced cap read
/// this, so the two can never diverge.
// Source: Ars Magica - Definitive Edition (Core Rules).md:2465
pub fn spell_level_cap(entity: &Entity, ruleset: &Ruleset, technique: &Id, form: &Id) -> i64 {
    let tech = i64::from(effective_art_score(entity, ruleset, technique));
    let form = i64::from(effective_art_score(entity, ruleset, form));
    let int = i64::from(
        entity
            .characteristics
            .get(&Characteristic::Int)
            .copied()
            .unwrap_or(0),
    );
    let magic_theory = i64::from(effective_ability_score(
        entity,
        ruleset,
        &Id::new(crate::ruleset::ID_MAGIC_THEORY),
        None,
    ));
    tech + form + int + magic_theory + 3
}

/// A per-Technique/Form spell-level cap, surfaced to the frontend so the spell
/// picker can grey a spell whose level exceeds the magus's cap without
/// recomputing the derivation in JS. Serializes as
/// `{ "technique": "<id>", "form": "<id>", "cap": N }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellLevelCap {
    /// The Technique-class Art id (e.g. `art.creo`).
    pub technique: Id,
    /// The Form-class Art id (e.g. `art.ignem`).
    pub form: Id,
    /// The maximum learnable level for this Te/Fo combination (may be negative
    /// for a beginning magus).
    pub cap: i64,
}

/// The [`spell_level_cap`] for every Technique × Form combination in the Art
/// catalogue, sorted canonically by `(technique, form)`. The picker keys these
/// by the pair to look up a candidate spell's cap. One entry per combo (a spell's
/// cap depends only on its Te/Fo, never its level).
pub fn spell_level_caps(entity: &Entity, ruleset: &Ruleset) -> Vec<SpellLevelCap> {
    // `art_ids_of` guarantees the sort the canonical (technique, form) order needs.
    let techniques = ruleset.art_ids_of(crate::art::ArtType::Technique);
    let forms = ruleset.art_ids_of(crate::art::ArtType::Form);
    let mut caps = Vec::with_capacity(techniques.len() * forms.len());
    for technique in &techniques {
        for form in &forms {
            caps.push(SpellLevelCap {
                technique: technique.clone(),
                form: form.clone(),
                cap: spell_level_cap(entity, ruleset, technique, form),
            });
        }
    }
    caps
}

/// The learned level of a chosen spell: the catalogue's fixed level, or — for a
/// **General** spell — the per-character chosen level. `None` if the spell is
/// unknown to the catalogue, or a General spell has no chosen level yet.
pub fn resolved_spell_level(sel: &SpellSelection, ruleset: &Ruleset) -> Option<u32> {
    let spell = ruleset.spell(&sel.spell)?;
    match spell.level {
        Some(fixed) => Some(u32::from(fixed)),
        None => sel.level.map(u32::from),
    }
}

/// The character's Spell-Mastery XP pool: the sum of every
/// [`Effect::SpellMasteryXp`] (Mastered Spells +50, stackable). A restricted pool
/// spent only on per-spell Spell Mastery Abilities. Source: Ars Magica -
/// Definitive Edition (Core Rules).md:4471-4474.
pub fn spell_mastery_xp(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::SpellMasteryXp { amount } = effect {
                total += u32::from(*amount);
            }
        }
    }
    total
}

/// The mastery-score floor every known spell receives from
/// [`Effect::GrantsSpellMastery`] (Flawless Magic → 1). The highest floor wins.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3887-3889.
pub fn spell_mastery_floor(entity: &Entity, ruleset: &Ruleset) -> u8 {
    let mut floor = 0u8;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::GrantsSpellMastery { score, .. } = effect {
                floor = floor.max(*score);
            }
        }
    }
    floor
}

/// The Advancement-Total multiplier applying to *every* Spell Mastery Ability, as
/// an Affinity "counts as num/den of itself" ([`Effect::GrantsSpellMastery`]'s
/// doubling: Flawless Magic → `(2, 1)`, halving the XP charged). `None` when no
/// grant reduces the cost. The most generous multiplier wins, like any Affinity.
/// Source: Ars Magica - Definitive Edition (Core Rules).md:3889.
pub fn spell_mastery_advancement_affinity(entity: &Entity, ruleset: &Ruleset) -> Option<(u8, u8)> {
    let selections = selections_for_effects(entity, ruleset);
    let found = selections.iter().flat_map(|selection| {
        let item = ruleset.point_items.get(&selection.item_ref);
        item.into_iter()
            .flat_map(|item| &item.effects)
            .filter_map(|effect| match effect {
                Effect::GrantsSpellMastery {
                    advancement_num,
                    advancement_den,
                    ..
                    // A larger num/den is a genuine reduction; the identity 1/1
                    // (a plain floor grant) contributes no Affinity.
                } if u32::from(*advancement_num) > u32::from(*advancement_den) => {
                    Some((*advancement_num, *advancement_den))
                }
                // A plain floor grant (identity multiplier) or any other effect
                // contributes no advancement Affinity. Enumerated so a new Effect
                // variant is a compile error here until it is classified.
                Effect::GrantsSpellMastery { .. }
                | Effect::AffinityAbilityCost { .. }
                | Effect::AbilityBonus { .. }
                | Effect::CharacteristicLimit { .. }
                | Effect::ArtBonus { .. }
                | Effect::AffinityArtCost { .. }
                | Effect::RestrictedAbilityXp { .. }
                | Effect::CharacteristicPoints { .. }
                | Effect::AbilityScoreGrant { .. }
                | Effect::SpellLevels { .. }
                | Effect::GeneralXp { .. }
                | Effect::LaterLifeXpRate { .. }
                | Effect::AbilityAuthorization { .. }
                | Effect::LocalityAbilityCapFraction { .. }
                | Effect::ConfidenceBonus { .. }
                | Effect::SpellMasteryXp { .. }
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
                | Effect::ElementalMagic { .. } => None,
            })
    });
    best_affinity(found)
}

/// The effective Spell Mastery score of one chosen spell: the higher of its
/// bought mastery and the granted floor (Flawless Magic auto-masters at 1).
pub fn effective_spell_mastery(sel: &SpellSelection, entity: &Entity, ruleset: &Ruleset) -> u8 {
    sel.mastery
        .unwrap_or(0)
        .max(spell_mastery_floor(entity, ruleset))
}

/// Total spell levels the entity's chosen spells consume. Unresolved General
/// spells (no chosen level) and unknown spells contribute 0.
pub fn spell_levels_used(entity: &Entity, ruleset: &Ruleset) -> u32 {
    entity
        .spells
        .iter()
        .filter_map(|s| resolved_spell_level(s, ruleset))
        .sum()
}
