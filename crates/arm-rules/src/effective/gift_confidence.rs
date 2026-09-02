//! The Gift (whether an entity has it) and Confidence, plus the enchanted-item
//! and supernatural-power level budgets a being's Virtues confer — four small,
//! independent "how much may this being spend/hold" queries with no
//! mechanical relationship to Might or Warping. Split out of
//! `might_warping.rs` (Viktor's V1 architecture finding, round 2: that file
//! was itself a leftover bucket of ~10 unrelated domains after the round-1
//! `effective.rs` split); pure code motion, no behavior change.

use super::*;

/// Whether the entity "has The Gift" per its type profile: a selection matching
/// the profile's `gift_id`, or one whose item category is in `gift_categories`.
/// Shared with `validate_gift_policy` so both use one definition.
pub(crate) fn has_the_gift(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: &EntityTypeProfile,
) -> bool {
    let by_id = profile
        .gift_id
        .as_ref()
        .is_some_and(|gid| entity.selections.iter().any(|s| &s.item_ref == gid));
    let by_category = !profile.gift_categories.is_empty()
        && entity.selections.iter().any(|s| {
            ruleset
                .point_items
                .get(&s.item_ref)
                .is_some_and(|item| profile.gift_categories.contains(&item.category))
        });
    by_id || by_category
}

/// A character's effective Confidence: the derived Confidence Score and the
/// Confidence Points backing it. Serializes like its sibling result types
/// (`Balance`, `AbilityBonus`) as `{ "score": N, "points": N }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confidence {
    /// The Confidence Score (spent per die roll).
    pub score: u8,
    /// The Confidence Points available to refresh the score.
    pub points: u8,
}

/// The character's effective Confidence: the type profile's base plus every
/// [`Effect::ConfidenceBonus`], clamped at 0. Confidence is derived, never
/// stored. Source: Ars Magica - Definitive Edition (Core Rules).md:2520-2526,
/// 4900-4902.
pub fn confidence(
    base_score: u8,
    base_points: u8,
    entity: &Entity,
    ruleset: &Ruleset,
) -> Confidence {
    let mut score = i32::from(base_score);
    let mut points = i32::from(base_points);
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::ConfidenceBonus {
            score: s,
            points: p,
        } = effect
        {
            score += i32::from(*s);
            points += i32::from(*p);
        }
    });
    let clamp = |n: i32| u8::try_from(n.max(0)).unwrap_or(u8::MAX);
    Confidence {
        score: clamp(score),
        points: clamp(points),
    }
}

/// The character's derived enchanted-device level budget: base 0 plus every
/// [`Effect::ItemLevelBudget`] (Magic Items +25, Redcap 50), summed. Source:
/// Ars Magica - Definitive Edition (Core Rules).md:4347-4349, :4842-4846.
pub fn item_level_budget(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::ItemLevelBudget { amount } = effect {
            total += u32::from(*amount);
        }
    });
    total
}

/// The total enchanted-device level the entity's `devices` consume — the "used"
/// side of the item-level budget bar. Summed across every device. Source: Ars
/// Magica - Definitive Edition (Core Rules).md:4347-4349.
pub fn item_level_used(entity: &Entity) -> u32 {
    entity.devices.iter().map(|d| u32::from(d.level)).sum()
}

/// The character's derived power-levels budget: base 0 plus every
/// [`Effect::PowerLevels`] grant (Demonic Blood 30, Demonic Powers +20, Strong
/// Angelic Heritage 30), summed. The being's `powers` are charged against it,
/// mirroring [`item_level_budget`]. Source: Ars Magica 5e - Realms of Power -
/// The Infernal.md:4122, :4142; Ars Magica 5e - Realms of Power - The Divine
/// (Revised).md:1977.
pub fn power_levels_budget(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::PowerLevels { amount } = effect {
            total += u32::from(*amount);
        }
    });
    total
}

/// The total power level the being's `powers` consume — the "used" side of the
/// power-levels budget bar. Source: Ars Magica 5e - Realms of Power - The
/// Infernal.md:4122.
pub fn powers_used(entity: &Entity) -> u32 {
    entity.powers.iter().map(|p| u32::from(p.level)).sum()
}
