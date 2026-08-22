//! Supernatural Might and True Faith: two independent Realm-adjacent scores a
//! being may hold. Split out of `might_warping.rs` (Viktor's V1 architecture
//! finding, round 2 — that file was a leftover bucket of ~10 unrelated
//! domains after the round-1 `effective.rs` split); pure code motion, no
//! behavior change.

use super::*;

/// Every [`Effect::MightGrant`] a being's Virtues confer, as `(realm, score)`
/// pairs (selections + derived grants).
fn might_grants(entity: &Entity, ruleset: &Ruleset) -> Vec<(Realm, u8)> {
    let mut grants = Vec::new();
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::MightGrant { realm, score } = effect {
                grants.push((*realm, *score));
            }
        }
    }
    grants
}

/// The being's **effective Might Score**, or `None` if it is not a supernatural
/// being (no base Might and no [`Effect::MightGrant`]). The Realm comes from the
/// entity's base Might if entered, else from its Might Virtue grants; the score is
/// the entered base (may be 0) plus every same-Realm grant. Demonic Blood grants
/// Infernal Might 5, Demonic Might +2 → effective 7. Source: Ars Magica 5e -
/// Realms of Power - Magic.md:1470-1472; Ars Magica 5e - Realms of Power -
/// The Infernal.md:4120, :4136.
pub fn effective_might(entity: &Entity, ruleset: &Ruleset) -> Option<MightScore> {
    let grants = might_grants(entity, ruleset);
    let realm = entity
        .might
        .map(|m| m.realm)
        .or_else(|| grants.first().map(|(realm, _)| *realm))?;
    let base = entity.might.map(|m| m.score).unwrap_or(0);
    let granted: u32 = grants
        .iter()
        .filter(|(r, _)| *r == realm)
        .map(|(_, s)| u32::from(*s))
        .sum();
    let score = u8::try_from(u32::from(base) + granted).unwrap_or(u8::MAX);
    Some(MightScore { realm, score })
}

/// The character's derived True Faith Score: base 0 plus every
/// [`Effect::TrueFaithGrant`] (True Faith Virtue → 1), summed and clamped to
/// `u8`. Derived, never stored. Source: Ars Magica - Definitive Edition (Core
/// Rules).md:5169-5171.
pub fn true_faith(entity: &Entity, ruleset: &Ruleset) -> u8 {
    let mut score = 0u32;
    for selection in selections_for_effects(entity, ruleset).iter() {
        let Some(item) = ruleset.point_items.get(&selection.item_ref) else {
            continue;
        };
        for effect in &item.effects {
            if let Effect::TrueFaithGrant { score: s } = effect {
                score += u32::from(*s);
            }
        }
    }
    u8::try_from(score).unwrap_or(u8::MAX)
}
