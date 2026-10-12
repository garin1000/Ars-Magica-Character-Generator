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
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::MightGrant { realm, score } = effect {
            grants.push((*realm, *score));
        }
    });
    grants
}

/// The being's **effective Might Score**, or `None` if it is not a supernatural
/// being (no base Might and no [`Effect::MightGrant`]). The Realm comes from the
/// entity's base Might if entered, else from its Might Virtue grants; the score is
/// the entered base (may be 0) plus every same-Realm grant. Demonic Blood grants
/// Infernal Might 5, Demonic Might +2 → effective 7. Source: RoP:M:1368-1370;
/// RoP:I:4180, :4196.
pub fn effective_might(entity: &Entity, ruleset: &Ruleset) -> Option<MightScore> {
    let grants = might_grants(entity, ruleset);
    let realm = entity
        .might
        .map(|m| m.realm)
        .or_else(|| grants.first().map(|(realm, _)| *realm))?;
    let base = entity.might.map(|m| m.score).unwrap_or(0);
    // K5 (round 1 full audit, 2026-10-12): `saturating_add`, not `.sum()`/`+` —
    // a crafted save's granting selections are an uncapped `Vec`, and
    // `[profile.release] overflow-checks = true` (root `Cargo.toml`, K9) turns
    // a plain overflow into a process abort before the `u8::try_from(...)
    // .unwrap_or(u8::MAX)` fallback below ever gets a chance to run.
    let granted: u32 = grants
        .iter()
        .filter(|(r, _)| *r == realm)
        .map(|(_, s)| u32::from(*s))
        .fold(0u32, u32::saturating_add);
    let score = u8::try_from(u32::from(base).saturating_add(granted)).unwrap_or(u8::MAX);
    Some(MightScore { realm, score })
}

/// The character's derived True Faith Score: base 0 plus every
/// [`Effect::TrueFaithGrant`] (True Faith Virtue → 1), summed and clamped to
/// `u8`. Derived, never stored. Source: ArMDE:5240-5242.
pub fn true_faith(entity: &Entity, ruleset: &Ruleset) -> u8 {
    let mut score = 0u32;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if let Effect::TrueFaithGrant { score: s } = effect {
            // K5 (round 1 full audit, 2026-10-12): `saturating_add`, not `+=` —
            // see `effective_might`'s identical note just above.
            score = score.saturating_add(u32::from(*s));
        }
    });
    u8::try_from(score).unwrap_or(u8::MAX)
}
