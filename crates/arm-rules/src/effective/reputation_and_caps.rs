//! Reputation grants, the Gift's free Supernatural-Ability slots, and the
//! age-based Ability-score caps — three independent "what may this character
//! start with or reach" queries with no mechanical relationship to Might or
//! Warping. Split out of `might_warping.rs` (Viktor's V1 architecture
//! finding, round 2 — that file was a leftover bucket of ~10 unrelated
//! domains after the round-1 `effective.rs` split); pure code motion, no
//! behavior change.

use super::*;

/// A Reputation a character's Virtue/Flaw authorizes them to start with. A
/// `reputation_type` of `None` means the grant leaves the type to the player
/// (e.g. Famous). Serializes as `{ "reputation_type": <type>|null, "score": N }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationGrant {
    /// The Reputation type the grant fixes, or `None` when player-chosen.
    pub reputation_type: Option<ReputationType>,
    /// The starting Reputation score the grant confers.
    pub score: u8,
}

/// The Reputation grants a character holds (one per [`Effect::GrantsReputation`]),
/// authorizing starting Reputations. Source: Ars Magica - Definitive Edition
/// (Core Rules).md:2512-2514.
pub fn reputation_grants(entity: &Entity, ruleset: &Ruleset) -> Vec<ReputationGrant> {
    let mut grants = Vec::new();
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::GrantsReputation { kind, score } = effect {
                grants.push(ReputationGrant {
                    reputation_type: *kind,
                    score: *score,
                });
            }
        }
    }
    grants
}

/// A character's Gift-granted free Supernatural-Ability slots: how many the Gift
/// confers and how many the entity currently consumes. Serializes like its
/// sibling result types as `{ "total": N, "used": N }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupernaturalFreeSlots {
    /// The number of free Supernatural-Ability slots the Gift confers.
    pub total: u8,
    /// The Supernatural abilities the entity holds that no granting Virtue covers.
    pub used: u8,
}

/// The Gift's free Supernatural-Ability slots. A Gifted non-magus gets one free
/// slot; a magus gets none (his free ability is Hermetic magic itself). `used`
/// counts the Supernatural abilities the entity holds that no granting Virtue
/// covers (a granting Virtue seeds an `ability_score_grant` floor).
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2874.
pub fn supernatural_free_slots(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: &EntityTypeProfile,
) -> SupernaturalFreeSlots {
    let total = if has_the_gift(entity, ruleset, profile) && !profile.is_magus {
        1
    } else {
        0
    };
    let covered: BTreeSet<Id> = ability_score_floors(entity, ruleset)
        .into_iter()
        .map(|f| f.ability)
        .collect();
    let used = entity
        .ability_scores
        .iter()
        .filter(|a| {
            ruleset
                .ability(&a.ability)
                .is_some_and(|ab| ab.category == AbilityCategory::Supernatural)
        })
        .filter(|a| !covered.contains(&a.ability))
        .count();
    SupernaturalFreeSlots {
        total,
        used: u8::try_from(used).unwrap_or(u8::MAX),
    }
}

/// The base age → maximum-Ability-score cap for `age`, read from the ruleset's
/// age band table (Ars Magica - Definitive Edition (Core Rules).md:2366-2374). Data, not hardcoded: the bands live
/// in `rules/core/abilities.json` (`age_ability_caps`) and are surfaced via
/// `EffectiveScores` so the UI never re-hardcodes the table. `None` when the
/// ruleset ships no age caps. An Ability with an Affinity may exceed this by +2
/// (applied in validation).
pub fn age_max_ability_score(ruleset: &Ruleset, age: u32) -> Option<u8> {
    ruleset.age_ability_caps().max_ability_score(age)
}

/// The character's base age → Ability-score cap, if `age` is set and the ruleset
/// ships an age band table.
pub fn age_ability_cap(entity: &Entity, ruleset: &Ruleset) -> Option<u8> {
    age_max_ability_score(ruleset, entity.age?)
}

/// The age cap for ONE ability, after any Virtue/Flaw that narrows it for
/// locality-dependent Abilities.
///
/// > The maximum scores at character creation for locality-dependent Abilities like
/// > Language, Area Lore, or Organization Lore, as well as some social Abilities,
/// > are half (round up) that which his age normally allows.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:6160 (Foreign
/// Upbringing). Which Abilities count as locality-dependent is catalogue data
/// (`locality_dependent`), because the passage's "as well as some social Abilities"
/// is deliberately open — the engine enforces the flag it is given rather than
/// guessing which social Abilities a saga counts.
///
/// The fraction rounds **up**, per the passage. Several such flaws would compose by
/// applying the smallest resulting cap, though no shipped Flaw pairs with another.
pub fn ability_age_cap(entity: &Entity, ruleset: &Ruleset, ability: &Id) -> Option<u8> {
    let base = age_ability_cap(entity, ruleset)?;
    if !ruleset
        .ability(ability)
        .is_some_and(|def| def.locality_dependent)
    {
        return Some(base);
    }
    let mut cap = base;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::LocalityAbilityCapFraction { num, den } = effect
                && *den > 0
            {
                // Ceiling division: "half (round up)".
                let numerator = u32::from(base) * u32::from(*num) + u32::from(*den) - 1;
                let fractioned = u8::try_from(numerator / u32::from(*den)).unwrap_or(base);
                cap = cap.min(fractioned);
            }
        }
    }
    Some(cap)
}
