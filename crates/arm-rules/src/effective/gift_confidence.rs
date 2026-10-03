//! The Gift (whether an entity has it) and Confidence, plus the enchanted-item
//! and supernatural-power level budgets a being's Virtues confer — four small,
//! independent "how much may this being spend/hold" queries with no
//! mechanical relationship to Might or Warping. Split out of
//! `might_warping.rs` (Viktor's V1 architecture finding, round 2: that file
//! was itself a leftover bucket of ~10 unrelated domains after the round-1
//! `effective.rs` split); pure code motion, no behavior change.

use super::*;

/// Whether the entity "has The Gift" per its type profile: a selection matching
/// the profile's `gift_id`, or one carrying *any* category in `gift_categories`
/// — a descriptor's secondary category counts, so Suppressed Gift ("*Major,
/// Hermetic, Story*") is recognised as Hermetic here.
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
    // Taken-as aware, via `PointItem::categories_for`: a selection recording
    // which category it was taken as counts as Gift-bearing only if THAT
    // category is a Gift category — no shipped item pairs a `taken_as` param
    // with a `hermetic` category yet, but a future one must not silently gift
    // a character through a reading the player did not choose.
    let by_category = !profile.gift_categories.is_empty()
        && entity.selections.iter().any(|s| {
            ruleset.point_items.get(&s.item_ref).is_some_and(|item| {
                item.categories_for(&s.params)
                    .iter()
                    .any(|c| profile.gift_categories.contains(c))
            })
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
/// stored. Source: ArMDE:2520-2526, :4900-4902.
pub fn confidence(
    base_score: u8,
    base_points: u8,
    entity: &Entity,
    ruleset: &Ruleset,
) -> Confidence {
    let mut score = i32::from(base_score);
    let mut points = i32::from(base_points);
    // A profile with no Confidence base at all (the grog, ArMDE:1161/:2522)
    // has no Confidence track for a ConfidenceBonus effect to add to — a
    // Virtue delta cannot conjure a Confidence Score/Points a type profile
    // denies outright.
    if base_score != 0 || base_points != 0 {
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
    }
    let clamp = |n: i32| u8::try_from(n.max(0)).unwrap_or(u8::MAX);
    Confidence {
        score: clamp(score),
        points: clamp(points),
    }
}

/// The character's derived enchanted-device level budget: base 0 plus every
/// [`Effect::ItemLevelBudget`] (Magic Items +25, Redcap 50), summed. Source:
/// ArMDE:4347-4349, :4842-4846.
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
/// side of the item-level budget bar. Summed across every device. Source:
/// ArMDE:4347-4349.
pub fn item_level_used(entity: &Entity) -> u32 {
    // Saturating, not `sum()`: the rows come from the save, and enough maxed
    // `u16` levels overflow `u32` (a panic in release).
    entity
        .devices
        .iter()
        .fold(0u32, |sum, d| sum.saturating_add(u32::from(d.level)))
}

/// The character's derived power-levels budget: base 0 plus every
/// [`Effect::PowerLevels`] grant (Demonic Blood 30, Demonic Powers +20, Strong
/// Angelic Heritage 30), summed. The being's `powers` are charged against it,
/// mirroring [`item_level_budget`]. Source: RoP:I:4122, :4142; RoP:D:1977.
pub fn power_levels_budget(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::PowerLevels { amount } = effect {
            total += u32::from(*amount);
        }
    });
    total
}

/// The character's Focus Power point pool: base 0 plus every
/// [`Effect::FocusPoints`] grant, summed. Source: ArMDE:3899, :3903.
pub fn focus_points_budget(entity: &Entity, ruleset: &Ruleset) -> u32 {
    let mut total = 0u32;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::FocusPoints { amount } = effect {
            total += u32::from(*amount);
        }
    });
    total
}

/// The Focus Power points the character's `focus_powers` consume — the "used"
/// side of the focus-points pool.
///
/// "It costs 2 points to raise the maximum level of effect by 1, and 1 point to
/// raise the Penetration by 1. Thus, 25 points can allow a maximum level of 10
/// with a Penetration of 5, or a maximum level of 5 with a Penetration of 15, or
/// combinations in between" (`ArMDE:3899`) — hence `2 × max_level + penetration`,
/// which both of the book's worked splits satisfy exactly.
///
/// Charged against [`focus_points_budget`] alone: a focus power never touches the
/// level budget [`powers_used`] spends, and vice versa. Source: `ArMDE:3899`.
pub fn focus_points_used(entity: &Entity) -> u32 {
    entity
        .focus_powers
        .iter()
        .map(|p| 2 * u32::from(p.max_level) + u32::from(p.penetration))
        // Saturating, as in `item_level_used`: each row fits easily, but the save
        // controls how many rows there are.
        .fold(0u32, u32::saturating_add)
}

/// The total power level the being's `powers` consume — the "used" side of the
/// power-levels budget bar.
///
/// Level **and** Penetration, because they are spent from one pool: "You may also
/// spend levels one-for-one to give the power Penetration; otherwise, it has a
/// Penetration of zero" (ArMDE:4019).
/// The book's worked example (`ArMDE:4021`) spends the 100 levels of two Greater Powers
/// as "a power with a level of 60 and a Penetration of 0, and a second power with
/// a level and Penetration of 20 each" — 60 + 0 + 20 + 20 = 100. Counting the
/// levels alone would report 80 and let the player buy 20 levels already spent.
///
/// Source: RoP:I:4122; ArMDE:4019, :4021.
pub fn powers_used(entity: &Entity) -> u32 {
    entity
        .powers
        .iter()
        .map(|p| u32::from(p.level) + u32::from(p.penetration))
        // Saturating, as in `item_level_used`: the save controls the row count.
        .fold(0u32, u32::saturating_add)
}
