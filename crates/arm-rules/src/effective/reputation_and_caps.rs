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
/// (e.g. Famous). Serializes as
/// `{ "source": <item id>, "reputation_type": <type>|null, "score": N }`.
///
/// `source` is the id of the Virtue/Flaw whose [`Effect::GrantsReputation`]
/// produced this grant. Without it a granted Reputation cannot say why it
/// exists — the player sees "Ecclesiastical 4" with no hint that Apostate put
/// it there. Carrying an owned [`Id`] is why this type is not `Copy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReputationGrant {
    /// The Virtue/Flaw that granted this Reputation slot.
    pub source: Id,
    /// The Reputation type the grant fixes, or `None` when player-chosen.
    pub reputation_type: Option<ReputationType>,
    /// The starting Reputation score the grant confers, exact unless
    /// `max_score` states a range.
    pub score: u8,
    /// The upper bound of a stated range (D11/Q5); `None` means exact.
    pub max_score: Option<u8>,
}

/// The Reputation grants a character holds (one per [`Effect::GrantsReputation`]),
/// authorizing starting Reputations. Source: ArMDE:2512-2514.
pub fn reputation_grants(entity: &Entity, ruleset: &Ruleset) -> Vec<ReputationGrant> {
    let mut grants = Vec::new();
    for_each_effect!(entity, ruleset, |selection, effect| {
        if let Effect::GrantsReputation {
            kind,
            score,
            max_score,
        } = effect
        {
            grants.push(ReputationGrant {
                source: selection.item_ref.clone(),
                reputation_type: *kind,
                score: *score,
                max_score: *max_score,
            });
        }
    });
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
/// Source: ArMDE:2874.
pub fn supernatural_free_slots(
    entity: &Entity,
    ruleset: &Ruleset,
    profile: &EntityTypeProfile,
) -> SupernaturalFreeSlots {
    // Bare profile rename only (compiler-forced by D56/A0's `is_magus` split).
    // This site is not named by any A1 sub-slice in
    // `docs/vf-audit/design-a0-is-magus-split.md` § 5 — flagged there as an
    // orphan needing a home before the union (`is_hermetically_trained`) is
    // wired in; until then this stays a same-behavior rename.
    let total = if has_the_gift(entity, ruleset, profile) && !profile.hermetically_trained {
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
/// age band table (ArMDE:2366-2374). Data, not hardcoded: the bands live
/// in `rules/core/abilities.json` (`age_ability_caps`) and are surfaced via
/// `EffectiveScores` so the UI never re-hardcodes the table. `None` when the
/// ruleset ships no age caps.
///
/// This is the band alone — no override folded in yet. [`ability_age_cap`] is
/// the per-ability resolution point (D29) that folds every Virtue/Flaw
/// override over this base; callers wanting the enforced cap for one Ability
/// should read that function, not this one.
pub fn age_max_ability_score(ruleset: &Ruleset, age: u32) -> Option<u8> {
    ruleset.age_ability_caps().max_ability_score(age)
}

/// The character's base age → Ability-score cap, if `age` is set and the ruleset
/// ships an age band table.
pub fn age_ability_cap(entity: &Entity, ruleset: &Ruleset) -> Option<u8> {
    age_max_ability_score(ruleset, entity.age?)
}

/// Whether any held item waives the age → Ability-score cap outright
/// (`Effect::WaivesAbilityAgeCap`; Mentored by Demons, ArMDE:4498, F-194). The
/// waiver applies to every Ability — the passage names no list — so this is a
/// single yes/no fact about the entity, not a per-ability one.
fn ability_age_cap_waived(entity: &Entity, ruleset: &Ruleset) -> bool {
    let mut waived = false;
    for_each_effect!(entity, ruleset, |_selection, effect| {
        if matches!(effect, Effect::WaivesAbilityAgeCap) {
            waived = true;
        }
    });
    waived
}

/// **D29: the single resolution point for an Ability's maximum score.** Folds
/// the age band (`age_ability_cap`) with EVERY Virtue/Flaw override that
/// bears on it — a full waiver (Mentored by Demons, ArMDE:4498), the
/// locality-dependent halving (Foreign Upbringing, ArMDE:6160), and the
/// Affinity +2 (ArMDE:3374) — so a second check beside this one (as
/// `validate_ability_age_cap` used to run for Affinity) can no longer
/// disagree with it: there is nowhere else left to ask. Both the validator and
/// any future UI surface read this function, never `age_ability_cap` +
/// their own override logic.
///
/// > The maximum scores at character creation for locality-dependent Abilities like
/// > Language, Area Lore, or Organization Lore, as well as some social Abilities,
/// > are half (round up) that which his age normally allows.
///
/// Source: ArMDE:6160 (Foreign
/// Upbringing). Which Abilities count as locality-dependent is catalogue data
/// (`locality_dependent`), because the passage's "as well as some social Abilities"
/// is deliberately open — the engine enforces the flag it is given rather than
/// guessing which social Abilities a saga counts.
///
/// The fraction rounds **up**, per the passage. Several such flaws would compose by
/// applying the smallest resulting cap, though no shipped Flaw pairs with another.
///
/// `None` means no cap applies at all — a ruleset shipping no age bands, or a
/// full waiver — so callers must not treat it as "very low" or "very high".
pub fn ability_age_cap(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &Id,
    parameter: Option<&str>,
) -> Option<u8> {
    let base = age_ability_cap(entity, ruleset)?;
    if ability_age_cap_waived(entity, ruleset) {
        return None;
    }
    let mut cap = base;
    if ruleset
        .ability(ability)
        .is_some_and(|def| def.locality_dependent)
    {
        for_each_effect!(entity, ruleset, |_selection, effect| {
            if let Effect::LocalityAbilityCapFraction { num, den } = effect
                && *den > 0
            {
                // Ceiling division: "half (round up)".
                let numerator = u32::from(base) * u32::from(*num) + u32::from(*den) - 1;
                let fractioned = u8::try_from(numerator / u32::from(*den)).unwrap_or(base);
                cap = cap.min(fractioned);
            }
        });
    }
    if ability_affinity(entity, ruleset, ability, parameter).is_some() {
        // ArMDE:3374 — "exceed the normal age-based cap … by two points", not
        // without limit, so this is an addend, never a second waiver.
        cap = cap.saturating_add(2);
    }
    Some(cap)
}

#[cfg(test)]
mod age_cap_resolution_point_tests {
    use super::*;
    use crate::types::{Entity, EntityKind, Id, RulesetRef, Selection};
    use crate::{Ruleset, RulesetSources};
    use std::collections::BTreeMap;

    const ITEMS: &str = r#"[
      { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
        "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
      { "id": "virtue.affinity_ability", "kind": "virtue", "classification": "narrative",
        "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
        "parameters": [{ "key": "ability", "type": "ref", "domain": "ability" }],
        "effects": [{ "type": "affinity_ability_cost", "param": "ability", "counts_as_num": 3, "counts_as_den": 2 }] }
    ]"#;
    const TYPES: &str = r#"[
      { "id": "companion", "budget": { "virtue_points": 10, "flaw_points": 10 },
        "permitted_categories": ["general", "personality"], "creation_phases": [] }
    ]"#;
    // Age caps: 5 under 30.
    const ABILITIES: &str = r#"{
      "age_ability_caps": [ { "max_age": 29, "max_score": 5 }, { "max_score": 9 } ],
      "abilities": [ { "id": "ability.brawl", "category": "general" } ]
    }"#;

    fn rs() -> Ruleset {
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

    fn companion_with_affinity(age: u32) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.age = Some(age);
        let mut params = BTreeMap::new();
        params.insert("ability".to_string(), Id::new("ability.brawl"));
        entity.selections = vec![Selection::with_params(
            Id::new("virtue.affinity_ability"),
            params,
        )];
        entity
    }

    /// D29: `ability_age_cap` is documented as the resolution point for an
    /// Ability's maximum score, yet `validate_ability_age_cap` (`validation/
    /// scores.rs`) applies the Affinity +2 override BESIDE it rather than
    /// through it — the "two consumers disagree" shape D29 forbids. At age 25
    /// (base cap 5), an Affinity-bearing Ability is legal up to 7; the
    /// resolution point must say so.
    #[test]
    fn the_resolution_point_folds_the_affinity_override_the_validator_applies() {
        let ruleset = rs();
        let entity = companion_with_affinity(25);
        assert_eq!(
            ability_age_cap(&entity, &ruleset, &Id::new("ability.brawl"), None),
            Some(7),
        );
    }

    /// F-194: Mentored by Demons waives the age cap outright. Modelled
    /// data-driven (any item carrying `Effect::WaivesAbilityAgeCap`), not by a
    /// hardcoded virtue id.
    #[test]
    fn a_waiver_effect_removes_the_cap_entirely() {
        let items = r#"[
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "categories": ["personality"], "entity_kinds": ["character"] },
          { "id": "virtue.trained_beyond_years", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"],
            "effects": [{ "type": "waives_ability_age_cap" }] }
        ]"#;
        let ruleset = Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: items,
            type_profiles: TYPES,
            abilities: Some(ABILITIES),
            ..RulesetSources::default()
        })
        .unwrap();
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.age = Some(20);
        entity.selections = vec![Selection::new(Id::new("virtue.trained_beyond_years"))];
        assert_eq!(
            ability_age_cap(&entity, &ruleset, &Id::new("ability.brawl"), None),
            None,
            "a full waiver leaves no cap to enforce",
        );
    }
}
